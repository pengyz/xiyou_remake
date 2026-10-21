#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""比对器：data/tables/*.json 表值 == extractor 机械提取值（R2 第二侧）。

- 每值一行断言（逐值锚三元组逐条对账），错位**逐行报**（index/表值/提取值/源行）；
- 判定逻辑独立于 generator（不 import generator）——防同源互证；
- 附带迁入源核对：crates/game-core/tests/fixtures/type_width_px.txt == typeFrameWidth。

三元组形：`{index, value:{事实名: 值}, src}`（三键；value 结构化）。
"""
from __future__ import annotations

import json
from pathlib import Path

from extractor import (DEFAULT_JAVA, FIXTURE_WIDTH, ROOT, TYPE_HI, TYPE_LO,
                       ExtractionError, extract)

TABLES_DIR = ROOT / "data" / "tables"
FILE_SPRITE = "sprite_defs.json"
FILE_ANIM = "anim_defs.json"
FILE_SHEET = "sheet_index.json"

CORE_TRIPLE_COUNT = 89 + 88 + 11 + 16 + 16  # 89 anim_row + 88 type_frame_width + 11 帧序行
# + 16 名序 + 16 张数 = 220


class Mismatch:
    def __init__(self, check: str, index, table_value, extract_value, src):
        self.check = check
        self.index = index
        self.table_value = table_value
        self.extract_value = extract_value
        self.src = src

    def __str__(self):
        return (f"[{self.check}] index={self.index} 表值={self.table_value!r} "
                f"提取值={self.extract_value!r} src={self.src}")


def _anchor(entry: dict, fact: str) -> dict:
    """取 entry.evidence 里 value 含事实名 fact 的三元组（无则 {}）。"""
    for e in entry.get("evidence", []):
        val = e.get("value")
        if isinstance(val, dict) and fact in val:
            return e
    return {}


def _anchor_fact(e: dict, fact: str):
    val = e.get("value")
    return val.get(fact) if isinstance(val, dict) else None


class Comparator:
    def __init__(self, java_path: Path = DEFAULT_JAVA, tables_dir: Path = TABLES_DIR):
        self.java_path = Path(java_path)
        self.tables_dir = Path(tables_dir)
        self.data = extract(self.java_path)
        self.errors: list[Mismatch] = []

    def _load(self, name: str) -> dict:
        path = self.tables_dir / name
        if not path.exists():
            raise FileNotFoundError(f"表缺失: {path}（先跑 generate 回填）")
        return json.loads(path.read_text(encoding="utf-8"))

    def _eq(self, check: str, index, table_value, extract_value, src) -> None:
        if table_value != extract_value:
            self.errors.append(Mismatch(check, index, table_value, extract_value, src))

    @staticmethod
    def _find_anchor(doc: dict, fact: str, index: int) -> dict:
        for e in doc.get("evidence", []):
            val = e.get("value")
            if isinstance(val, dict) and fact in val and e.get("index") == index:
                return e
        return {}

    # ------------------------------------------------------------- 入口
    def run_all(self) -> list[Mismatch]:
        sprite = self._load(FILE_SPRITE)
        anim = self._load(FILE_ANIM)
        sheet = self._load(FILE_SHEET)
        self.check_schema(sprite, anim, sheet)
        self.check_anim_rows(anim)
        self.check_types(sprite)
        self.check_containers(sheet)
        self.check_slots(sheet)
        self.check_assembly(sheet)
        self.check_size_dispatch(sprite)
        self.check_transition_overrides(sprite)
        self.check_quirks(sprite)
        self.check_fixture()
        self.check_triple_counts(sprite, anim, sheet)
        return self.errors

    # ------------------------------------------------------------- 各面
    def check_schema(self, sprite: dict, anim: dict, sheet: dict) -> None:
        for name, doc in (("sprite_defs", sprite), ("anim_defs", anim),
                          ("sheet_index", sheet)):
            self._eq("schema_version", name, doc.get("schema_version"), 2,
                     "解析器 SCHEMA_VERSION（§2.3 schema 版本字段）")
        for t in sprite.get("sprite_defs", []):
            self._eq("density", t.get("type_id"), t.get("density"), 1, "density 默认 1")

    def check_anim_rows(self, anim: dict) -> None:
        rows = anim.get("anim_defs", [])
        self._eq("anim_defs 行数", "-", len(rows), 11, "a.java:485-497")
        src_rows = self.data["anim_offset_table"]["rows"]
        for i, row in enumerate(rows):
            src = src_rows[i] if i < len(src_rows) else {"value": None, "line": None}
            self._eq("anim.frames", i, row.get("frames"), src["value"], f"a.java:{src['line']}")
            self._eq("anim.step_ticks", i, row.get("step_ticks"), 2,
                     "a.java:6833 (frameCounter&1 全局节拍)")
            self._eq("anim.loop_mode", i, row.get("loop_mode"), "cycle",
                     "a.java:6850-6851 (default 臂归零)")
            ev = _anchor(row, "frames")
            self._eq("anim 三元组.value.frames", i, _anchor_fact(ev, "frames"),
                     src["value"], f"a.java:{src['line']}")
            self._eq("anim 三元组.index", i, ev.get("index"), i, f"a.java:{src['line']}")
            self._eq("anim 三元组.src", i, ev.get("src"), f"a.java:{src['line']}",
                     f"a.java:{src['line']}")

    def check_types(self, sprite: dict) -> None:
        types = sprite.get("sprite_defs", [])
        self._eq("sprite_defs 行数", "-", len(types), TYPE_HI - TYPE_LO + 1, "型域 0..88")
        anim_vals = self.data["anim_row_by_type"]["values"]
        width_vals = {v["index"]: v for v in self.data["type_frame_width"]["values"]}
        cat_vals = self.data["render_category"]["values"]
        ins_vals = {v["index"]: v for v in self.data["no_insert_type_flags"]["values"]}
        img_vals = {v["index"]: v for v in self.data["type_image_map"]["values"]}
        names = {v["index"]: v["value"] for v in self.data["resource_names"]["values"]}
        floor = self.data["floor_category_override"]

        for t, entry in enumerate(types):
            self._eq("type_id", t, entry.get("type_id"), t, "行序")
            # ① 89 项 animRowByType
            a_src = anim_vals[t]
            self._eq("anim_row", t, entry.get("anim_row"), a_src["value"],
                     f"a.java:{a_src['line']}")
            ev = _anchor(entry, "anim_row")
            self._eq("anim_row 三元组.value", t, _anchor_fact(ev, "anim_row"), a_src["value"],
                     f"a.java:{a_src['line']}")
            self._eq("anim_row 三元组.src", t, ev.get("src"), f"a.java:{a_src['line']}",
                     f"a.java:{a_src['line']}")
            self._eq("anim_row 三元组.index", t, ev.get("index"), t, f"a.java:{a_src['line']}")
            self._eq("anim_profile 外键", t, entry.get("anim_profile"),
                     f"anim_row_{a_src['value']}", "anim_defs FK")
            # ② 88 项 typeFrameWidth
            w_src = width_vals.get(t)
            exp_w = None if w_src is None else w_src["value"]
            self._eq("type_frame_width", t, entry.get("type_frame_width"), exp_w,
                     f"a.java:{w_src['line'] if w_src else '-'}")
            if w_src is not None:
                ev = _anchor(entry, "type_frame_width")
                self._eq("type_frame_width 三元组.value", t,
                         _anchor_fact(ev, "type_frame_width"),
                         w_src["value"], f"a.java:{w_src['line']}")
                self._eq("type_frame_width 三元组.src", t, ev.get("src"),
                         f"a.java:{w_src['line']}", f"a.java:{w_src['line']}")
            # ③ render_category
            c_src = cat_vals[t]
            self._eq("render_category", t, entry.get("render_category"), c_src["value"],
                     f"a.java:{c_src['line']}")
            # ④ insert_block
            i_src = ins_vals.get(t)
            self._eq("insert_block", t, entry.get("insert_block"),
                     i_src["value"] if i_src else False,
                     f"a.java:{i_src['line'] if i_src else 6246}")
            # ⑤ 型→图集映射
            img = img_vals[t]
            if img["via"] == "none":
                self._eq("sheet", t, entry.get("sheet"), None, "无映射")
                self._eq("sheet_index", t, entry.get("sheet_index"), None, "无映射")
            else:
                target = img_vals[img["alias_of"]] if img["via"] == "alias" else img
                self._eq("sheet", t, entry.get("sheet"),
                         names[target["container_index"]], f"a.java:{img['line']}")
                self._eq("sheet_index", t, entry.get("sheet_index"), target["slot"],
                         f"a.java:{img['line']}")
            # ⑥ 派发尺寸（独立推导）
            self.check_effective_size(t, entry)
            # ⑦ floor 覆写
            if t == floor["type"]:
                self._eq("floor_category_override", t, entry.get("floor_category_override"),
                         [{"floor": 50, "category": floor["override_category"]}],
                         f"a.java:{floor['line']}")
            else:
                self._eq("floor_category_override", t,
                         entry.get("floor_category_override"), [], "-")
            # ⑧ assembly_profile 挂点
            self._eq("assembly_profile", t, entry.get("assembly_profile"),
                     "seg67" if t == 67 else None, "a.java:682-697")

    def check_effective_size(self, t: int, entry: dict) -> None:
        """dispatchSpriteSize 独立推导（不借 generator 的代码）。"""
        sd = self.data["size_dispatch"]
        arms = {lbl: arm for arm in sd["arms"] for lbl in arm["labels"]}
        fallback = {a["field"]: a["value"] for a in sd["fallback_stmts"]
                    if a["stmt"] == "assign"}

        def assigns(arm):
            return {s["field"]: s["value"] for s in arm["stmts"] if s["stmt"] == "assign"}

        img = self.data["type_image_map"]["values"][t]
        w_item = {v["index"]: v for v in self.data["type_frame_width"]["values"]}.get(t)
        if t == 69:
            got = assigns(arms[69])
            exp = (got["sprite_w"], got["sprite_h"])
        elif t == 72:
            got = assigns(arms[72])
            exp = (got["sprite_w"], got["sprite_h"])
        elif t == 67 or img["via"] == "none":
            exp = (fallback["sprite_w"], fallback["sprite_h"])
        else:
            got = assigns(arms["default"])
            h = None if "getHeight" in str(got["sprite_h"]) else got["sprite_h"]
            exp = (w_item["value"] if w_item else None, h)
        self._eq("frame_w", t, entry.get("frame_w"), exp[0],
                 "dispatchSpriteSize a.java:6300-6325")
        self._eq("frame_h", t, entry.get("frame_h"), exp[1],
                 "dispatchSpriteSize a.java:6300-6325")

    def check_containers(self, sheet: dict) -> None:
        containers = sheet.get("containers", [])
        names = self.data["resource_names"]["values"]
        counts = self.data["resource_image_counts"]["values"]
        self._eq("containers 数", "-", len(containers), 16, "a.java:450-453")
        for i, name in enumerate(containers):
            self._eq("container.name", i, name, names[i]["value"], f"a.java:{names[i]['line']}")
        self._eq("container_counts", "-", sheet.get("container_counts"),
                 [c["value"] for c in counts], f"a.java:{counts[0]['line']}")
        for i in range(16):
            ev_n = self._find_anchor(sheet, "name", i)
            ev_c = self._find_anchor(sheet, "count", i)
            self._eq("name 三元组.value", i, _anchor_fact(ev_n, "name"),
                     names[i]["value"], f"a.java:{names[i]['line']}")
            self._eq("name 三元组.src", i, ev_n.get("src"), f"a.java:{names[i]['line']}",
                     f"a.java:{names[i]['line']}")
            self._eq("count 三元组.value", i, _anchor_fact(ev_c, "count"),
                     counts[i]["value"], f"a.java:{counts[i]['line']}")
            self._eq("count 三元组.src", i, ev_c.get("src"), f"a.java:{counts[i]['line']}",
                     f"a.java:{counts[i]['line']}")

    def check_slots(self, sheet: dict) -> None:
        """槽位占用表 == 型→图集映射独立归并。"""
        imgs = self.data["type_image_map"]["values"]
        exp: dict[tuple[int, int], list[int]] = {}
        for rec in imgs:
            if rec["via"] == "direct":
                target = rec
            elif rec["via"] == "alias":
                target = imgs[rec["alias_of"]]
            else:
                continue
            exp.setdefault((target["container_index"], target["slot"]), []).append(rec["index"])
        got = {}
        for slot in sheet.get("slots", {}).get("claims", []):
            got[(slot.get("container_index"), slot.get("slot"))] = \
                sorted(slot.get("used_by_types", []))
        self._eq("slots 槽位集合", "-", sorted(got), sorted(exp), "a.java:9455-9491")
        for key in sorted(set(exp) | set(got)):
            self._eq(f"slots {key}", "-", got.get(key), sorted(exp.get(key, [])),
                     "a.java:9455-9491")

    def check_assembly(self, sheet: dict) -> None:
        seg = sheet.get("assembly_profiles", {}).get("seg67", {})
        pieces = self.data["seg67_pieces"]["rows"]
        frames = self.data["seg67_frames"]["rows"]
        self._eq("seg67.pieces 数", "-", len(seg.get("pieces", [])), 9, "a.java:682-692")
        self._eq("seg67.frames 行数", "-", len(seg.get("frames", [])), 3, "a.java:693-697")
        for i, row in enumerate(pieces):
            self._eq("seg67.piece", i, seg.get("pieces", [None] * 9)[i], row["value"],
                     f"a.java:{row['line']}")
            ev = self._find_anchor(sheet, "seg67_piece", i)
            self._eq("seg67.piece 三元组.value", i, _anchor_fact(ev, "seg67_piece"),
                     row["value"], f"a.java:{row['line']}")
            self._eq("seg67.piece 三元组.src", i, ev.get("src"), f"a.java:{row['line']}",
                     f"a.java:{row['line']}")
        for i, row in enumerate(frames):
            flat = [x for seg4 in seg.get("frames", [[None] * 5] * 3)[i] for x in seg4]
            self._eq("seg67.frame_row", i, flat, row["value"], f"a.java:{row['line']}")
            ev = self._find_anchor(sheet, "seg67_frame_row", i)
            self._eq("seg67.frame_row 三元组.value", i, _anchor_fact(ev, "seg67_frame_row"),
                     row["value"], f"a.java:{row['line']}")
            self._eq("seg67.frame_row 三元组.src", i, ev.get("src"), f"a.java:{row['line']}",
                     f"a.java:{row['line']}")

    def check_size_dispatch(self, sprite: dict) -> None:
        sd_tab = sprite.get("size_dispatch", {})
        sd = self.data["size_dispatch"]
        arms = {lbl: arm for arm in sd["arms"] for lbl in arm["labels"]}
        fallback = {a["field"]: a["value"] for a in sd["fallback_stmts"]
                    if a["stmt"] == "assign"}
        self._eq("size_dispatch.fallback.frame_w", "-",
                 sd_tab.get("fallback", {}).get("frame_w"), fallback["sprite_w"],
                 "a.java:6323")
        self._eq("size_dispatch.fallback.frame_h", "-",
                 sd_tab.get("fallback", {}).get("frame_h"), fallback["sprite_h"],
                 "a.java:6324")
        for t in (67, 69, 72):
            arm = arms[t]
            got = {s["field"]: s["value"] for s in arm["stmts"] if s["stmt"] == "assign"}
            entry = sd_tab.get("by_type", {}).get(str(t), {})
            if got:
                self._eq(f"size_dispatch.by_type[{t}].frame_w", t,
                         entry.get("frame_w"), got["sprite_w"], f"a.java:{arm['line']}")
                self._eq(f"size_dispatch.by_type[{t}].frame_h", t,
                         entry.get("frame_h"), got["sprite_h"], f"a.java:{arm['line']}")
        self._eq("size_dispatch.default.frame_w_from", "-",
                 sd_tab.get("default", {}).get("frame_w_from"), "type_frame_width",
                 "a.java:6317")
        self._eq("size_dispatch.default.frame_h_from", "-",
                 sd_tab.get("default", {}).get("frame_h_from"), "image_height",
                 "a.java:6318")

    def check_transition_overrides(self, sprite: dict) -> None:
        rules = {r.get("on"): r for r in sprite.get("transition_overrides", [])}

        # ① mark_removed
        mr = self.data["mark_removed"]
        rule = rules.get("mark_removed", {})
        self._eq("mark_removed.state", "-", rule.get("state"), mr["state"],
                 f"a.java:{mr['decl_line']}")
        bc = rule.get("by_category", {})
        self._eq("mark_removed 臂数", "-", len(bc), len(mr["arms"]),
                 f"a.java:{mr['decl_line']}")
        for arm in mr["arms"]:
            key = "|".join(str(x) for x in sorted(arm["labels"]))
            got = bc.get(key)
            self._eq(f"mark_removed[{key}] 存在", "-", got is not None, True,
                     f"a.java:{arm['line']}")
            if got is None:
                continue
            if "switch" in arm:
                by_type = got.get("by_type", {})
                for sub in arm["switch"]["arms"]:
                    skey = "|".join(str(x) for x in sorted(sub["labels"]))
                    exp = {s["field"]: s["value"] for s in sub["stmts"]
                           if s["stmt"] == "assign"}
                    self._eq(f"mark_removed[{key}].by_type[{skey}]", "-",
                             by_type.get(skey), exp, f"a.java:{sub['line']}")
                calls = [c["callee"] for c in arm["stmts"] if c["stmt"] == "call"]
                self._eq(f"mark_removed[{key}].build_minimap", "-",
                         got.get("build_minimap"), "buildMinimap" in calls,
                         f"a.java:{arm['line']}")
            else:
                exp = {s["field"]: s["value"] for s in arm["stmts"] if s["stmt"] == "assign"}
                self._eq(f"mark_removed[{key}]", "-", got, exp, f"a.java:{arm['line']}")

        # ② mark_des_removed
        md = self.data["mark_des_removed"]
        rule = rules.get("mark_des_removed", {})
        self._eq("mark_des_removed.state", "-", rule.get("state"), md["state"],
                 f"a.java:{md['decl_line']}")
        bc = rule.get("by_category", {})
        for arm in md["arms"]:
            key = "|".join(str(x) for x in sorted(arm["labels"]))
            got = bc.get(key)
            exp = {s["field"]: s["value"] for s in arm["stmts"] if s["stmt"] == "assign"}
            if not exp:
                exp = {"noop": True, "returns": True}
            self._eq(f"mark_des_removed[{key}]", "-", got, exp, f"a.java:{arm['line']}")

        # ③ bump_reveal
        br = self.data["bump_reveal"]
        rule = rules.get("bump_reveal", {})
        by_type = rule.get("by_type", {})
        for arm in br["arms"]:
            key = "|".join(str(x) for x in sorted(arm["labels"]))
            got = by_type.get(key)
            self._eq(f"bump_reveal[{key}] 存在", "-", got is not None, True,
                     f"a.java:{arm['line']}")
            if got is None:
                continue
            exp = {s["field"]: s["value"] for s in arm["stmts"] if s["stmt"] == "assign"}
            for field, val in exp.items():
                self._eq(f"bump_reveal[{key}].{field}", "-", got.get(field), val,
                         f"a.java:{arm['line']}")

    def check_quirks(self, sprite: dict) -> None:
        quirks = sprite.get("quirks_in_code", [])
        if not quirks:
            self.errors.append(Mismatch("quirks_in_code 留痕", "-", "空", "至少 1 条",
                                        "§2.1 禁静默留码"))
        for q in quirks:
            for field in ("quirk_id", "reason", "src", "red_test_id"):
                if not q.get(field):
                    self.errors.append(Mismatch(f"quirks[{q.get('quirk_id', '?')}].{field}",
                                                "-", q.get(field), "非空", "§2.1 留痕槽"))

    def check_fixture(self) -> None:
        """迁入源核对：type_width_px.txt 夹具 == typeFrameWidth 提取值。"""
        if not FIXTURE_WIDTH.exists():
            return
        fix = [int(x) for x in FIXTURE_WIDTH.read_text(encoding="utf-8").split()]
        ext = [v["value"] for v in self.data["type_frame_width"]["values"]]
        self._eq("type_width_px.txt 迁入源", "-", fix, ext,
                 "crates/game-core/tests/fixtures")

    def check_triple_counts(self, sprite: dict, anim: dict, sheet: dict) -> None:
        def count_fact(entries, fact):
            return sum(1 for e in entries for a in e.get("evidence", [])
                       if isinstance(a.get("value"), dict) and fact in a["value"])

        n_anim = count_fact(sprite.get("sprite_defs", []), "anim_row")
        n_width = count_fact(sprite.get("sprite_defs", []), "type_frame_width")
        n_rows = count_fact(anim.get("anim_defs", []), "frames")
        n_names = sum(1 for a in sheet.get("evidence", [])
                      if isinstance(a.get("value"), dict) and "name" in a["value"])
        n_counts = sum(1 for a in sheet.get("evidence", [])
                       if isinstance(a.get("value"), dict) and "count" in a["value"])
        got = n_anim + n_width + n_rows + n_names + n_counts
        self._eq("保真表三元组计数", "-", got, CORE_TRIPLE_COUNT,
                 f"89+88+11+16+16={CORE_TRIPLE_COUNT}（逐值锚生成式）")
        self._eq("三元组.anim_row", "-", n_anim, 89, "a.java:498-588")
        self._eq("三元组.type_frame_width", "-", n_width, 88, "a.java:589-678")
        self._eq("三元组.frames", "-", n_rows, 11, "a.java:485-497")
        self._eq("三元组.name", "-", n_names, 16, "a.java:451")
        self._eq("三元组.count", "-", n_counts, 16, "a.java:453")


def compare(java_path: Path = DEFAULT_JAVA, tables_dir: Path = TABLES_DIR) -> list[Mismatch]:
    return Comparator(java_path, tables_dir).run_all()


if __name__ == "__main__":
    import sys
    try:
        errors = compare()
    except (FileNotFoundError, ExtractionError) as exc:
        print(f"[compare] 无法比对: {exc}")
        sys.exit(2)
    if errors:
        print(f"[compare] {len(errors)} 处错位：")
        for e in errors:
            print("  " + str(e))
        sys.exit(1)
    print("[compare] 表值 == 机械提取值，全部等值 ✓")
