#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""生成器：把 extractor 的机械提取结果落成 data/tables/ 三表（含逐值锚三元组）。

schema 对齐（2026-10-21 与 crates/game-data/src/defs.rs 契约合流）：
- `schema_version` = 2（int，= 解析器 SCHEMA_VERSION）；
- 封套键：sprite_defs.json={schema_version, sprite_defs, transition_overrides,
  size_dispatch, quirks_in_code}；anim_defs.json={schema_version, anim_defs}；
  sheet_index.json={schema_version, containers, container_counts, slots,
  assembly_profiles, evidence}；
- 逐值锚三元组 = **三键** `{index, value, src}`（value 结构化=事实名→值，禁手抄）；
- density 字段（默认 1）+ quirks_in_code 留痕 + assembly_profile（SEG67 表中表）。

表里的一切**数值**都来自 extractor（a.java 字面），本模块只做结构映射；
comparator 独立复核（防手抄互证同源错误，R2）。
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

from extractor import ROOT, extract

TABLES_DIR = ROOT / "data" / "tables"
SCHEMA_VERSION = 2
DENSITY_DEFAULT = 1
# 动画步进节拍（a.java:6833 `(frameCounter & 1) != 0` ⇒ 每 2 tick 步进一帧）
ANIM_STEP_TICKS = 2
# 行末回卷（a.java:6850-6851 default 臂 entityAnimFrame = 0 ⇒ cycle）
ANIM_LOOP_MODE = "cycle"

FILE_SPRITE = "sprite_defs.json"
FILE_ANIM = "anim_defs.json"
FILE_SHEET = "sheet_index.json"


# ---------------------------------------------------------------- 小工具

def _json_bytes(obj) -> bytes:
    """确定性序列化：固定缩进/插入序/UTF-8/尾换行。"""
    return (json.dumps(obj, ensure_ascii=False, indent=2) + "\n").encode("utf-8")


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _assigns(arm: dict) -> dict:
    return {st["field"]: st["value"] for st in arm.get("stmts", []) if st["stmt"] == "assign"}


def _calls(arm: dict) -> list[dict]:
    return [st for st in arm.get("stmts", []) if st["stmt"] == "call"]


def _line_span(*lines: int | None) -> str:
    ls = sorted(l for l in lines if l)
    return f"a.java:{ls[0]}" if len(ls) == 1 else f"a.java:{ls[0]}-{ls[-1]}"


def _arm_key(labels: list) -> str:
    return "|".join(str(x) for x in sorted(labels))


def _anchor(index: int, fact: str, value, src: str) -> dict:
    """三键逐值锚三元组：{index, value:{事实名: 值}, src}。"""
    return {"index": index, "value": {fact: value}, "src": src}


# ---------------------------------------------------------------- 派生（全部来自提取）

def derive_effective_size(data: dict, t: int) -> dict:
    """dispatchSpriteSize 派发结果（a.java:6300-6325）逐型求值。

    返回 {frame_w, frame_h, kind}；kind=explicit|image_height|fallback。
    frame_h=null ⇒ 运行时图高（image_height）。
    """
    sd = data["size_dispatch"]
    fallback = {a["field"]: a["value"] for a in sd["fallback_stmts"] if a["stmt"] == "assign"}
    by_label = {lbl: arm for arm in sd["arms"] for lbl in arm["labels"]}
    if t == 69 or t == 72:
        got = _assigns(by_label[t])
        return {"frame_w": got["sprite_w"], "frame_h": got["sprite_h"], "kind": "explicit"}
    if t == 67:
        # 仅 loadPackedImages(9) 后 break → 落 switch 后 fallback（32×32）
        return {"frame_w": fallback["sprite_w"], "frame_h": fallback["sprite_h"],
                "kind": "fallback"}
    if data["type_image_map"]["values"][t]["via"] == "none":
        return {"frame_w": fallback["sprite_w"], "frame_h": fallback["sprite_h"],
                "kind": "fallback"}
    got = _assigns(by_label["default"])
    w = got["sprite_w"]
    if isinstance(w, str):  # this.typeFrameWidth[var1] → 保真表值
        w = derive_type_frame_width(data, t)
    h = got["sprite_h"]
    if isinstance(h, str) and "getHeight" in h:  # var3.getHeight() → 运行时图高
        h = None
    return {"frame_w": w, "frame_h": h, "kind": "image_height"}


def derive_type_frame_width(data: dict, t: int) -> int | None:
    """保真表 typeFrameWidth[t]（型 88 无表项 → null）。"""
    vals = {v["index"]: v for v in data["type_frame_width"]["values"]}
    item = vals.get(t)
    return None if item is None else item["value"]


# ---------------------------------------------------------------- sprite_defs

def build_sprite_defs(data: dict) -> dict:
    anim = {v["index"]: v for v in data["anim_row_by_type"]["values"]}
    widths = {v["index"]: v for v in data["type_frame_width"]["values"]}
    cats = {v["index"]: v for v in data["render_category"]["values"]}
    inserts = {v["index"]: v for v in data["no_insert_type_flags"]["values"]}
    imgs = {v["index"]: v for v in data["type_image_map"]["values"]}
    names = {v["index"]: v["value"] for v in data["resource_names"]["values"]}
    floor_ov = data["floor_category_override"]

    types = []
    for t in range(0, 89):
        a_row = anim[t]
        w_item = widths.get(t)
        cat = cats[t]
        ins = inserts.get(t)
        img = imgs[t]
        eff = derive_effective_size(data, t)

        if img["via"] == "alias":
            target = imgs[img["alias_of"]]
            sheet = names[target["container_index"]]
            sheet_index = target["slot"]
        elif img["via"] == "direct":
            sheet = names[img["container_index"]]
            sheet_index = img["slot"]
        else:
            sheet = None
            sheet_index = None

        evidence = [
            _anchor(t, "anim_row", a_row["value"], f"a.java:{a_row['line']}"),
        ]
        if w_item is not None:
            evidence.append(_anchor(t, "type_frame_width", w_item["value"],
                                    f"a.java:{w_item['line']}"))
        if img["via"] != "none":
            ref = {"via": img["via"]}
            if img["via"] == "alias":
                ref["alias_of"] = img["alias_of"]
            else:
                ref["container_index"] = img["container_index"]
                ref["slot"] = img["slot"]
            evidence.append(_anchor(t, "sheet_ref", ref, f"a.java:{img['line']}"))
        evidence.append(_anchor(t, "render_category", cat["value"],
                                f"a.java:{cat['line']}" if cat["line"]
                                else "a.java:6245 (byte[] 默认 0)"))
        evidence.append(_anchor(t, "insert_block", ins["value"] if ins else False,
                                f"a.java:{ins['line']}" if ins
                                else "a.java:6246 (blocksPlayerInsert 默认 false)"))

        entry = {
            "type_id": t,
            "sheet": sheet,
            "sheet_index": sheet_index,
            "frame_w": eff["frame_w"],
            "frame_h": eff["frame_h"],
            "render_category": cat["value"],
            "anim_row": a_row["value"],
            "type_frame_width": None if w_item is None else w_item["value"],
            "anim_profile": f"anim_row_{a_row['value']}",
            "assembly_profile": "seg67" if t == 67 else None,
            "insert_block": ins["value"] if ins else False,
            "floor_category_override": (
                [{"floor": 50, "category": floor_ov["override_category"]}]
                if t == floor_ov["type"] else []),
            "density": DENSITY_DEFAULT,
            "evidence": evidence,
        }
        if t == floor_ov["type"]:
            entry["evidence"].append(_anchor(
                t, "floor_category_override",
                {"floor": 50, "category": floor_ov["override_category"],
                 "default_category": floor_ov["default_category"]},
                f"a.java:{floor_ov['line']}"))
        types.append(entry)

    return {
        "schema_version": SCHEMA_VERSION,
        "sprite_defs": types,
        "transition_overrides": _build_transition_overrides(data),
        "size_dispatch": _build_size_dispatch(data),
        "quirks_in_code": _build_quirks(data),
    }


def _build_size_dispatch(data: dict) -> dict:
    sd = data["size_dispatch"]
    by_label = {lbl: arm for arm in sd["arms"] for lbl in arm["labels"]}
    fallback = {a["field"]: a["value"] for a in sd["fallback_stmts"] if a["stmt"] == "assign"}

    def by_type_entry(t: int) -> dict:
        arm = by_label[t]
        got = _assigns(arm)
        load = [c for c in _calls(arm) if c["callee"] == "loadPackedImages"]
        entry: dict = {}
        if got:
            entry["frame_w"] = got["sprite_w"]
            entry["frame_h"] = got["sprite_h"]
        else:
            entry["frame_w"] = fallback["sprite_w"]
            entry["frame_h"] = fallback["sprite_h"]
            entry["via"] = "break_to_fallback"
        if load:
            entry["load_packed_images"] = load[0]["args"][0]
        entry["src"] = _line_span(*[st["line"] for st in arm["stmts"]])
        return entry

    dflt = _assigns(by_label["default"])
    return {
        "default": {"frame_w_from": "type_frame_width",
                    "frame_h_from": "image_height",
                    "src": _line_span(*[st["line"] for st in by_label["default"]["stmts"]])},
        "fallback": {"frame_w": fallback["sprite_w"], "frame_h": fallback["sprite_h"],
                     "when": "image == null（default 臂 var3==null 落空）或 break 臂",
                     "src": _line_span(*[a["line"] for a in sd["fallback_stmts"]])},
        "by_type": {str(t): by_type_entry(t) for t in (67, 69, 72)},
    }


def _build_transition_overrides(data: dict) -> list[dict]:
    out = []

    # ① mark_removed（state=1）：a.java:6349-6387 / entity.rs:392-426
    mr = data["mark_removed"]
    by_category = {}
    for arm in mr["arms"]:
        key = _arm_key(arm["labels"])
        if "switch" in arm:  # 类目 1：内层按型分派（1|2|3 / 11）
            by_type = {_arm_key(sub["labels"]): _assigns(sub)
                       for sub in arm["switch"]["arms"]}
            entry = {"by_type": by_type, "otherwise": {"noop": True}}
            if any(c["callee"] == "buildMinimap" for c in _calls(arm)):
                entry["build_minimap"] = True
            by_category[key] = entry
        else:
            by_category[key] = _assigns(arm)
    out.append({
        "on": "mark_removed",
        "state": mr["state"],
        "by_category": by_category,
        "otherwise": {"noop": True},
        "evidence": f"a.java:{mr['decl_line']}-6387 (markEntityRemoved); "
                    f"entity.rs:392-426 (remove)",
    })

    # ② mark_des_removed（state=2）：a.java:6332-6347 / entity.rs:428-437
    md = data["mark_des_removed"]
    by_category = {}
    for arm in md["arms"]:
        key = _arm_key(arm["labels"])
        got = _assigns(arm)
        by_category[key] = got if got else {"noop": True, "returns": True}
    out.append({
        "on": "mark_des_removed",
        "state": md["state"],
        "by_category": by_category,
        "otherwise": {"noop": True},
        "evidence": f"a.java:{md['decl_line']}-6347 (markEntityDesRemoved); "
                    f"entity.rs:428-437 (remove_des)",
    })

    # ③ bump_reveal（显形行 9 / 型 12 变形）：a.java:5410-5417 / 5449-5457
    br = data["bump_reveal"]
    by_type = {}
    for arm in br["arms"]:
        key = _arm_key(arm["labels"])
        entry = _assigns(arm)
        calls = [c["callee"] for c in _calls(arm)]
        if "dispatchSpriteSize" in calls:
            entry["redispatch"] = ["dispatchSpriteSize", "dispatchAnimFields"]
            entry["block_step"] = True  # var6 = false（a.java:5456）
        entry["src"] = _line_span(*[st["line"] for st in arm["stmts"] if st["stmt"] != "raw"])
        by_type[key] = entry
    out.append({
        "on": "bump_reveal",
        "by_type": by_type,
        "evidence": "a.java:5410-5417 (型6 显形) / 5449-5457 (型12 变形); "
                    "scene.rs:5001-5103",
    })
    return out


def _build_quirks(data: dict) -> list[dict]:
    """装不下规则形的残余特例显式登记（survey §2.1 禁静默留码）。"""
    sd = data["size_dispatch"]
    load_lines = [st["line"] for arm in sd["arms"] for st in arm["stmts"]
                  if st["stmt"] == "call" and st["callee"] == "loadPackedImages"]
    return [
        {"quirk_id": "Q-001",
         "reason": "型 12 撞击墙 bump 变形：type→11 后须按**运行时图高**重派尺寸"
                   "（dispatchSpriteSize(11)）——图高是运行时事实，纯表规则装不下，保留代码路径",
         "src": "a.java:5449-5457",
         "red_test_id": "task7-three-beat-t12-morph"},
        {"quirk_id": "Q-002",
         "reason": "dispatchSpriteSize case 67/69 附带 loadPackedImages(9) 加载副作用"
                   "（非尺寸语义，规则形只承载尺寸值）",
         "src": _line_span(*load_lines),
         "red_test_id": "task7-size-dispatch-load-side-effect"},
        {"quirk_id": "Q-003",
         "reason": "typeFrameWidth 仅 88 项（型 0..87）而型号域 0..88：型 88 无表项且无图，"
                   "走 dispatchSpriteSize fallback 32×32（不读表）",
         "src": f"a.java:{data['type_frame_width']['decl_line']}-678 / 6322-6324",
         "red_test_id": "task7-type88-fallback"},
        {"quirk_id": "Q-004",
         "reason": "显形行 9 的型 12 臂含 `var6=false`（bump 阻挡）+ `continue`（续扫）控制流，"
                   "规则形以 block_step 标志承载阻挡，续扫语义留在消费侧代码",
         "src": "a.java:5456-5457",
         "red_test_id": "task7-three-beat-t12-morph"},
        {"quirk_id": "Q-005",
         "reason": "anim_defs.step_ticks=2（全局节拍 a.java:6833 frameCounter&1）与 "
                   "loop_mode=cycle（a.java:6850-6851 default 臂归零）**非行级表**——"
                   "按行回填会发明原版没有的行级语义，schema 字段保留但取全局机械值",
         "src": "a.java:6831-6857 (advanceEntityFrames)",
         "red_test_id": "task7-anim-step-ticks-global"},
    ]


# ---------------------------------------------------------------- anim_defs

def build_anim_defs(data: dict) -> dict:
    rows = []
    for row in data["anim_offset_table"]["rows"]:
        rows.append({
            "id": f"anim_row_{row['index']}",
            "frames": row["value"],
            "step_ticks": ANIM_STEP_TICKS,
            "loop_mode": ANIM_LOOP_MODE,
            "evidence": [_anchor(row["index"], "frames", row["value"],
                                 f"a.java:{row['line']}")],
        })
    return {"schema_version": SCHEMA_VERSION, "anim_defs": rows}


# ---------------------------------------------------------------- sheet_index

def build_sheet_index(data: dict) -> dict:
    names = data["resource_names"]["values"]
    counts = data["resource_image_counts"]["values"]
    imgs = data["type_image_map"]["values"]

    # 槽位占用表（sheet 侧；型侧在 sprite_defs.sheet/sheet_index——一事实一双视角，比对器核双写）
    claims: dict[tuple[int, int], dict] = {}
    for rec in imgs:
        if rec["via"] == "direct":
            target = rec
        elif rec["via"] == "alias":
            target = imgs[rec["alias_of"]]
        else:
            continue
        key = (target["container_index"], target["slot"])
        ent = claims.setdefault(key, {"types": [], "srcs": []})
        ent["types"].append(rec["index"])
        ent["srcs"].append(rec["line"])
    slots = [{"container_index": ci,
              "container": names[ci]["value"],
              "slot": slot,
              "used_by_types": sorted(claims[(ci, slot)]["types"]),
              "src": _line_span(*claims[(ci, slot)]["srcs"])}
             for (ci, slot) in sorted(claims)]

    evidence = []
    for i, (n, c) in enumerate(zip(names, counts)):
        evidence.append(_anchor(i, "name", n["value"], f"a.java:{n['line']}"))
        evidence.append(_anchor(i, "count", c["value"], f"a.java:{c['line']}"))

    def _segs(flat: list[int]) -> list[list[int]]:
        return [flat[i:i + 4] for i in range(0, 20, 4)]

    pieces = [r["value"] for r in data["seg67_pieces"]["rows"]]
    frames = [_segs(r["value"]) for r in data["seg67_frames"]["rows"]]
    for r in data["seg67_pieces"]["rows"]:
        evidence.append(_anchor(r["index"], "seg67_piece", r["value"], f"a.java:{r['line']}"))
    for r in data["seg67_frames"]["rows"]:
        evidence.append(_anchor(r["index"], "seg67_frame_row", r["value"],
                                f"a.java:{r['line']}"))

    return {
        "schema_version": SCHEMA_VERSION,
        "containers": [n["value"] for n in names],
        "container_counts": [c["value"] for c in counts],
        "slots": {"claims": slots},
        "assembly_profiles": {
            "seg67": {
                "target_type": 67,
                "sheet_container": "xtq",
                "pieces": pieces,
                "frames": frames,
            }
        },
        "evidence": evidence,
    }


# ---------------------------------------------------------------- 汇总

def build_tables(data: dict | None = None) -> dict[str, bytes]:
    if data is None:
        data = extract()
    return {
        FILE_SPRITE: _json_bytes(build_sprite_defs(data)),
        FILE_ANIM: _json_bytes(build_anim_defs(data)),
        FILE_SHEET: _json_bytes(build_sheet_index(data)),
    }


def write_tables(data: dict | None = None) -> dict[str, str]:
    tables = build_tables(data)
    TABLES_DIR.mkdir(parents=True, exist_ok=True)
    hashes = {}
    for name, blob in tables.items():
        (TABLES_DIR / name).write_bytes(blob)
        hashes[name] = _sha256(blob)
    return hashes


if __name__ == "__main__":
    for name, h in write_tables().items():
        print(f"{h}  data/tables/{name}")
