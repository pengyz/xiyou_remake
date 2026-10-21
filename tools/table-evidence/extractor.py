#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""机械提取器（R2 第二侧）：从 deobf a.java 逐值提取保真表 + 行号锚。

设计纪律（survey §4 R2）：
- 只认字面文本模式（声明行 + 数组字面量 + switch 臂），不认语义猜测；
- 每个值携带 1-based 源行号（逐值锚三元组的 src）；
- 提取不完整即抛 ExtractionError，绝不静默缺项；
- 本模块是"第二侧"——表值的正确性由「表 == 本提取」等值断言判定，
  防手抄互证同源错误（人手抄值进表 ⇒ 比对器逐行报错位）。

被 generator.py / comparator.py / tests 共用；可独立运行打印提取摘要。
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_JAVA = ROOT / "reference" / "src" / "deobf" / "a.java"
FIXTURE_WIDTH = ROOT / "crates" / "game-core" / "tests" / "fixtures" / "type_width_px.txt"

TYPE_LO, TYPE_HI = 0, 88  # 型号域 0..88（survey §2.3 域溢出拒载）


class ExtractionError(RuntimeError):
    """机械提取失败（模式不匹配/形状不符）——不许静默降级。"""


# ---------------------------------------------------------------- 基础工具

def read_lines(java_path: Path = DEFAULT_JAVA) -> list[str]:
    return Path(java_path).read_text(encoding="utf-8").splitlines()


def _find_line(lines: list[str], pattern: str, what: str) -> int:
    """返回首个匹配行的 0-based 下标。"""
    rx = re.compile(pattern)
    for i, line in enumerate(lines):
        if rx.search(line):
            return i
    raise ExtractionError(f"找不到 {what}（模式 {pattern!r}）")


def _collect_rows(lines: list[str], decl_idx: int, what: str) -> list[tuple[int, str]]:
    """声明行之后到 `};` 之前的行，返回 [(1-based line_no, text)]。"""
    rows: list[tuple[int, str]] = []
    for i in range(decl_idx + 1, len(lines)):
        if lines[i].strip().startswith("};"):
            return rows
        rows.append((i + 1, lines[i]))
    raise ExtractionError(f"{what} 数组未闭合 @line {decl_idx + 1}")


def _ints(text: str) -> list[int]:
    return [int(x) for x in re.findall(r"-?\d+", text)]


def _method_body(lines: list[str], decl_idx: int, what: str) -> list[tuple[int, str]]:
    """方法体（含声明行）到配对右括号为止，[(1-based line_no, text)]。"""
    depth = 0
    body: list[tuple[int, str]] = []
    for i in range(decl_idx, len(lines)):
        text = lines[i]
        body.append((i + 1, text))
        depth += text.count("{") - text.count("}")
        if depth == 0:
            return body
    raise ExtractionError(f"{what} 方法体未闭合 @line {decl_idx + 1}")


def _eval_index_expr(expr: str, var: str, val: int) -> int:
    """求值 `var4 - 1` / `var24 - 41 - 20` 这类纯整数下标表达式。"""
    sub = re.sub(r"\b" + re.escape(var) + r"\b", str(val), expr)
    if not re.fullmatch(r"[\d+\- ]+", sub):
        raise ExtractionError(f"下标表达式无法机械求值: {expr!r}")
    return eval(sub, {"__builtins__": {}}, {})  # noqa: S307 — 字符集已白名单化


# ---------------------------------------------------------------- 保真表（值+行锚）

def extract_anim_offset_table(lines: list[str]) -> dict:
    """ANIM_OFFSET_TABLE（a.java:485-497）：11 行帧序。"""
    d = _find_line(lines, r"this\.animOffsetTable = new int\[\]\[\]\{", "animOffsetTable")
    rows = []
    for idx, (ln, text) in enumerate(_collect_rows(lines, d, "animOffsetTable")):
        vals = _ints(text)
        if not vals:
            raise ExtractionError(f"animOffsetTable 第 {idx} 行无数值 @line {ln}")
        rows.append({"index": idx, "value": vals, "line": ln})
    return {"decl_line": d + 1, "rows": rows}


def extract_anim_row_by_type(lines: list[str]) -> dict:
    """animRowByType（a.java:498-588）：89 项（型 0..88）。"""
    d = _find_line(lines, r"this\.animRowByType = new byte\[\]\{", "animRowByType")
    values = []
    for idx, (ln, text) in enumerate(_collect_rows(lines, d, "animRowByType")):
        vals = _ints(text)
        if len(vals) != 1:
            raise ExtractionError(f"animRowByType 第 {idx} 项非单值 @line {ln}")
        values.append({"index": idx, "value": vals[0], "line": ln})
    return {"decl_line": d + 1, "values": values}


def extract_type_frame_width(lines: list[str]) -> dict:
    """typeFrameWidth（a.java:589-678）：88 项（型 0..87）。"""
    d = _find_line(lines, r"this\.typeFrameWidth = new byte\[\]\{", "typeFrameWidth")
    values = []
    for idx, (ln, text) in enumerate(_collect_rows(lines, d, "typeFrameWidth")):
        vals = _ints(text)
        if len(vals) != 1:
            raise ExtractionError(f"typeFrameWidth 第 {idx} 项非单值 @line {ln}")
        values.append({"index": idx, "value": vals[0], "line": ln})
    return {"decl_line": d + 1, "values": values}


def extract_resource_names(lines: list[str]) -> dict:
    """resourceNames（声明 a.java:450，值行 451）：16 容器名序。"""
    d = _find_line(lines, r"this\.resourceNames = new String\[\]\{", "resourceNames")
    values = []
    for idx, (ln, text) in enumerate(_collect_rows(lines, d, "resourceNames")):
        names = re.findall(r'"([^"]*)"', text)
        for name in names:
            values.append({"index": len(values), "value": name, "line": ln})
    return {"decl_line": d + 1, "values": values}


def extract_resource_image_counts(lines: list[str]) -> dict:
    """resourceImageCounts（a.java:453 单行）：16 容器张数表。

    注意锚定勘误（survey「450/451 错位」条目）：机械实测——
    名序值行 = 451（声明 450）、张数表赋值行 = **453**。三元组以机械行号为准。
    """
    d = _find_line(lines, r"this\.resourceImageCounts = new int\[\]\{", "resourceImageCounts")
    text = lines[d]
    payload = text[text.index("{") + 1:text.rindex("}")]
    values = []
    for idx, tok in enumerate(x.strip() for x in payload.split(",")):
        if not re.fullmatch(r"\d+", tok):
            raise ExtractionError(f"resourceImageCounts 第 {idx} 项非整数 @line {d + 1}")
        values.append({"index": idx, "value": int(tok), "line": d + 1})
    return {"decl_line": d + 1, "values": values}


def extract_seg67_pieces(lines: list[str]) -> dict:
    """f_byte_arr2_00（a.java:682-692）：9 片件 (sheet, srcX, srcY, w, h)。"""
    d = _find_line(lines, r"this\.f_byte_arr2_00 = new byte\[\]\[\]\{", "f_byte_arr2_00")
    rows = []
    for idx, (ln, text) in enumerate(_collect_rows(lines, d, "f_byte_arr2_00")):
        vals = _ints(text)
        if len(vals) != 5:
            raise ExtractionError(f"SEG67 片件 {idx} 非 5 元组 @line {ln}")
        rows.append({"index": idx, "value": vals, "line": ln})
    return {"decl_line": d + 1, "rows": rows}


def extract_seg67_frames(lines: list[str]) -> dict:
    """f_byte_arr2_01（a.java:693-697）：3 帧 × 5 段 × 4 字节（每行 20 值）。"""
    d = _find_line(lines, r"this\.f_byte_arr2_01 = new byte\[\]\[\]\{", "f_byte_arr2_01")
    rows = []
    for idx, (ln, text) in enumerate(_collect_rows(lines, d, "f_byte_arr2_01")):
        vals = _ints(text)
        if len(vals) != 20:
            raise ExtractionError(f"SEG67 段帧 {idx} 非 20 值 @line {ln}")
        rows.append({"index": idx, "value": vals, "line": ln})
    return {"decl_line": d + 1, "rows": rows}


def extract_no_insert_type_flags(lines: list[str]) -> dict:
    """noInsertTypeFlags（a.java:679）：13 项（型 0..12），其余型默认 false。"""
    d = _find_line(lines, r"this\.noInsertTypeFlags = new boolean\[\]\{", "noInsertTypeFlags")
    text = lines[d]
    payload = text[text.index("{") + 1:text.rindex("}")]
    values = []
    for idx, tok in enumerate(x.strip() for x in payload.split(",")):
        if tok not in ("true", "false"):
            raise ExtractionError(f"noInsertTypeFlags 第 {idx} 项非布尔 @line {d + 1}")
        values.append({"index": idx, "value": tok == "true", "line": d + 1})
    return {"decl_line": d + 1, "values": values}


# ---------------------------------------------------------------- 逐型派生事实

def extract_render_category(lines: list[str]) -> dict:
    """renderCategoryByType 初始化（a.java:6243-6298 initEntityTables）。

    循环赋值（1..12→1 等）+ 逐型覆盖（76→1 等）；未赋值型（0/88）= byte[] 默认 0。
    """
    d = _find_line(lines, r"private void initEntityTables\(\)", "initEntityTables")
    body = _method_body(lines, d, "initEntityTables")
    out: dict[int, dict] = {}
    loop: tuple[str, int, int] | None = None
    for ln, text in body:
        m = re.search(r"for \(int (\w+) = (\d+); \1 <= (\d+); \1\+\+\)", text)
        if m:
            loop = (m.group(1), int(m.group(2)), int(m.group(3)))
            continue
        m = re.search(r"for \(int (\w+) = (\d+); \1 < (\d+); \1\+\+\)", text)
        if m:
            loop = (m.group(1), int(m.group(2)), int(m.group(3)) - 1)
            continue
        m = re.search(r"this\.renderCategoryByType\[(\w+)\] = (\d+);", text)
        if not m:
            continue
        idx_tok, val = m.group(1), int(m.group(2))
        if loop and idx_tok == loop[0]:
            for t in range(loop[1], loop[2] + 1):
                out[t] = {"value": val, "line": ln}
        elif re.fullmatch(r"\d+", idx_tok):
            out[int(idx_tok)] = {"value": val, "line": ln}
        else:
            raise ExtractionError(f"renderCategoryByType 下标语句无法机械解析 @line {ln}")
    values = []
    for t in range(TYPE_LO, TYPE_HI + 1):
        item = out.get(t, {"value": 0, "line": None})
        values.append({"index": t, "value": item["value"], "line": item["line"]})
    return {"decl_line": d + 1, "values": values}


def extract_type_image_map(lines: list[str]) -> dict:
    """m_122 头部 型→图集映射（a.java:9455-9491）：(容器, 槽) 或别名。

    只收 `entityTypeImage[X] = ...` 赋值；读取不动。别名（12→11、71→70）保留
    via=alias 并记录被别名型。
    """
    d = _find_line(lines, r"private void m_122\(int var1\)", "m_122")
    body = _method_body(lines, d, "m_122")
    out: dict[int, dict] = {}
    loop: tuple[str, int, int] | None = None
    rx = re.compile(
        r"entityTypeImage\[(\w+)\] = \w+\.(f_Image_arr2_00|entityTypeImage)"
        r"\[(\d+)\](?:\[(.+?)\])?;")
    for ln, text in body:
        m = re.search(r"for \(int (\w+) = (\d+); \1 <= (\d+); \1\+\+\)", text)
        if m:
            loop = (m.group(1), int(m.group(2)), int(m.group(3)))
            continue
        m = re.search(r"for \(int (\w+) = (\d+); \1 < (\d+); \1\+\+\)", text)
        if m:
            loop = (m.group(1), int(m.group(2)), int(m.group(3)) - 1)
            continue
        m = rx.search(text)
        if not m:
            continue
        idx_tok, rhs_kind, first_idx, slot_expr = m.groups()
        if loop and idx_tok == loop[0]:
            targets = list(range(loop[1], loop[2] + 1))
        elif re.fullmatch(r"\d+", idx_tok):
            targets = [int(idx_tok)]
        else:
            raise ExtractionError(f"entityTypeImage 下标语句无法机械解析 @line {ln}")
        for t in targets:
            if rhs_kind == "entityTypeImage":
                if slot_expr is not None:
                    raise ExtractionError(f"别名右值形状异常 @line {ln}")
                out[t] = {"via": "alias", "alias_of": int(first_idx), "line": ln}
            else:
                if slot_expr is None:
                    raise ExtractionError(f"图集右值缺槽下标 @line {ln}")
                slot = _eval_index_expr(slot_expr, idx_tok, t) if idx_tok in slot_expr \
                    else _eval_index_expr(slot_expr, idx_tok, t)
                out[t] = {"via": "direct", "container_index": int(first_idx),
                          "slot": slot, "line": ln}
    values = []
    for t in range(TYPE_LO, TYPE_HI + 1):
        item = out.get(t)
        if item is None:
            values.append({"index": t, "via": "none", "line": None})
        else:
            rec = {"index": t, "via": item["via"], "line": item["line"]}
            if item["via"] == "alias":
                rec["alias_of"] = item["alias_of"]
            else:
                rec["container_index"] = item["container_index"]
                rec["slot"] = item["slot"]
            values.append(rec)
    return {"decl_line": d + 1, "values": values}


# ---------------------------------------------------------------- switch 臂机械解析

_FIELD_MAP = {
    "entitySpriteW": "sprite_w",
    "entitySpriteH": "sprite_h",
    "entityAnimRow": "anim_row",
    "entityAnimFrame": "anim_frame",
    "entityState": "state",
    "entityType": "type_id",
    "entityVisible": "visible",
}


def _parse_stmt(ln: int, text: str) -> dict | None:
    """把一条 Java 语句解析成机械事实（无法解析返回 None=原样留痕）。"""
    s = text.strip()
    if not s or s in ("{", "}") or s.startswith("//"):
        return None
    m = re.match(r"this\.(\w+)\[(\w+)\] = (.+);$", s)
    if m and m.group(1) in _FIELD_MAP:
        rhs = m.group(3).strip()
        if re.fullmatch(r"-?\d+", rhs):
            val: int | str | bool = int(rhs)
        elif rhs == "true":
            val = True
        elif rhs == "false":
            val = False
        else:
            val = rhs  # 符号右值（typeFrameWidth[var1] / var3.getHeight() ...）
        return {"stmt": "assign", "field": _FIELD_MAP[m.group(1)], "value": val, "line": ln}
    m = re.match(r"this\.(\w+)\((.*)\);$", s)
    if m:
        args = [a.strip() for a in m.group(2).split(",") if a.strip()]
        call = {"stmt": "call", "callee": m.group(1), "line": ln}
        if all(re.fullmatch(r"-?\d+", a) for a in args):
            call["args"] = [int(a) for a in args]
        else:
            call["args"] = args
        return call
    if s in ("return;", "break;", "continue;"):
        return {"stmt": s[:-1], "line": ln}
    return {"stmt": "raw", "text": s, "line": ln}


def _parse_switch_arms(body: list[tuple[int, str]], switch_ln: int) -> list[dict]:
    """解析一个 switch 的臂（支持嵌套 switch）。

    臂形：{labels, line, stmts[], switch?{var, line, arms[]}}。
    - case/default 标签行归入当前层 pending，遇语句即开新臂；
    - 嵌套 `switch (var) {` 开新层并挂到当前臂；
    - 括号深度回落到层以下时弹层。
    """
    depths_before = []
    depths_after = []
    depth = 0
    for _, text in body:
        depths_before.append(depth)
        depth += text.count("{") - text.count("}")
        depths_after.append(depth)
    try:
        start = next(i for i, (ln, _) in enumerate(body) if ln == switch_ln)
    except StopIteration:
        raise ExtractionError(f"switch 行 {switch_ln} 不在方法体内") from None

    root_case_level = depths_before[start] + 1
    root = {"case_level": root_case_level, "pending": [], "cur": None, "arms": []}
    stack = [root]

    def _flush_pending(ctx: dict, ln: int) -> dict:
        if ctx["cur"] is None or ctx["pending"]:
            ctx["cur"] = {"labels": [], "stmts": [], "line": ln}
            ctx["arms"].append(ctx["cur"])
        if ctx["pending"]:
            ctx["cur"]["labels"].extend(ctx["pending"])
            ctx["pending"] = []
        return ctx["cur"]

    for i in range(start + 1, len(body)):
        ln, text = body[i]
        d_before = depths_before[i]
        ctx = stack[-1]
        case_level = ctx["case_level"]
        stripped = text.strip()

        if stripped and d_before >= case_level:
            rest = stripped
            head = re.match(r"\s*((?:\bcase\s+-?\d+\s*:|\bdefault\s*:)+)", text)
            if head:
                for tok in re.findall(r"case\s+(-?\d+)\s*:", head.group(1)):
                    ctx["pending"].append(int(tok))
                if re.search(r"\bdefault\s*:", head.group(1)):
                    ctx["pending"].append("default")
                rest = text[head.end():].strip()
            nested = re.search(r"\bswitch\s*\((.+?)\)\s*\{", rest)
            if nested:
                arm = _flush_pending(ctx, ln)
                arm["switch"] = {"var": nested.group(1).strip(), "line": ln, "arms": []}
                stack.append({"case_level": d_before + 1, "pending": [], "cur": None,
                              "arms": arm["switch"]["arms"]})
                continue
            if rest:
                stmt = _parse_stmt(ln, rest)
                if stmt is not None:
                    arm = _flush_pending(ctx, ln)
                    arm["stmts"].append(stmt)

        # 括号回落弹层（`}` 闭合行本身无内容）
        while len(stack) > 1 and depths_after[i] < stack[-1]["case_level"]:
            stack.pop()
        if depths_after[i] < root_case_level:
            break

    for ctx in stack:
        if ctx["pending"]:
            arm = {"labels": list(ctx["pending"]), "stmts": [], "line": body[-1][0]}
            ctx["arms"].append(arm)
    return root["arms"]


def _switch_body(lines: list[str], decl_pattern: str, what: str,
                 switch_pattern: str) -> tuple[list[tuple[int, str]], int]:
    d = _find_line(lines, decl_pattern, what)
    body = _method_body(lines, d, what)
    for ln, text in body:
        if re.search(switch_pattern, text):
            return body, ln
    raise ExtractionError(f"{what} 内找不到目标 switch（{switch_pattern}）")


def extract_size_dispatch(lines: list[str]) -> dict:
    """dispatchSpriteSize（a.java:6300-6325）：型→尺寸派发（含 fallback 臂）。"""
    body, sw = _switch_body(
        lines, r"private void dispatchSpriteSize\(int var1, int var2\)",
        "dispatchSpriteSize", r"switch \(var1\)")
    arms = _parse_switch_arms(body, sw)
    # switch 后的 fallback 语句（6323-6324）：取 switch 闭合之后、方法收尾之前
    depths = []
    depth = 0
    for _, text in body:
        depths.append(depth)
        depth += text.count("{") - text.count("}")
    start = next(i for i, (ln, _) in enumerate(body) if ln == sw)
    d0 = depths[start]
    end = next(i for i in range(start, len(body)) if i > start and depths[i] <= d0)
    fallback = [s for ln, text in body[end + 1:]
                if (s := _parse_stmt(ln, text))]
    return {"decl_line": body[0][0], "arms": arms, "fallback_stmts": fallback}


def extract_mark_removed(lines: list[str]) -> dict:
    """markEntityRemoved（a.java:6349-6387）：state=1 路径（entity.rs:392-426 同构）。"""
    body, sw = _switch_body(
        lines, r"private void markEntityRemoved\(int var1\)",
        "markEntityRemoved", r"switch \(this\.renderCategoryByType")
    arms = _parse_switch_arms(body, sw)
    m = re.search(r"this\.entityState\[var1\] = (\d+);", body[1][1])
    state = int(m.group(1)) if m else None
    return {"decl_line": body[0][0], "switch_line": sw, "state": state, "arms": arms}


def extract_mark_des_removed(lines: list[str]) -> dict:
    """markEntityDesRemoved（a.java:6332-6347）：state=2 路径（entity.rs:428- 同构）。"""
    body, sw = _switch_body(
        lines, r"private void markEntityDesRemoved\(int var1\)",
        "markEntityDesRemoved", r"switch \(this\.renderCategoryByType")
    arms = _parse_switch_arms(body, sw)
    m = re.search(r"this\.entityState\[var1\] = (\d+);", body[1][1])
    state = int(m.group(1)) if m else None
    return {"decl_line": body[0][0], "switch_line": sw, "state": state, "arms": arms}


def extract_floor_category_override(lines: list[str]) -> dict:
    """载入层 类目覆写（a.java:9678-9682）：floor==50 ⇒ renderCategoryByType[72]=8。"""
    d = _find_line(lines, r"if \(var1 != 50\) \{", "floor50 类目覆写")
    window = lines[d:d + 6]
    assigns = []
    for k, text in enumerate(window):
        m = re.search(r"this\.renderCategoryByType\[(\d+)\] = (\d+);", text)
        if m:
            assigns.append({"type": int(m.group(1)), "value": int(m.group(2)),
                            "line": d + 1 + k})
    if len(assigns) != 2 or {a["type"] for a in assigns} != {72}:
        raise ExtractionError(f"floor50 类目覆写臂形状异常 @line {d + 1}")
    # 形态：if (var1 != 50) { =C_default } else { =C_override }——先出现的=非 50 默认值
    normal, override = assigns[0], assigns[1]
    return {
        "condition": "currentFloor == 50",
        "type": normal["type"],
        "default_category": normal["value"],
        "override_category": override["value"],
        "line": override["line"],
    }


def extract_bump_reveal(lines: list[str]) -> dict:
    """interactWithCell 型 switch（a.java:5380-…）的显形/变形臂：
    型 6 显形（5410-5417）、型 12 撞击墙变形（5449-5457）——state=2 + 行 9。"""
    body, sw = _switch_body(
        lines, r"private boolean interactWithCell\(int var1, int var2\)",
        "interactWithCell", r"switch \(var5\) \{")
    arms = _parse_switch_arms(body, sw)
    wanted = {}
    for arm in arms:
        labels = set(arm["labels"])
        if 6 in labels:
            wanted[6] = arm
        if 12 in labels:
            wanted[12] = arm
    if set(wanted) != {6, 12}:
        raise ExtractionError(f"显形臂不全（找到 {sorted(wanted)}）")
    return {"decl_line": body[0][0], "switch_line": sw, "arms": [wanted[6], wanted[12]]}


# ---------------------------------------------------------------- 汇总

def extract(java_path: Path = DEFAULT_JAVA) -> dict:
    lines = read_lines(java_path)
    data = {
        "java_path": str(java_path),
        "anim_offset_table": extract_anim_offset_table(lines),
        "anim_row_by_type": extract_anim_row_by_type(lines),
        "type_frame_width": extract_type_frame_width(lines),
        "resource_names": extract_resource_names(lines),
        "resource_image_counts": extract_resource_image_counts(lines),
        "seg67_pieces": extract_seg67_pieces(lines),
        "seg67_frames": extract_seg67_frames(lines),
        "no_insert_type_flags": extract_no_insert_type_flags(lines),
        "render_category": extract_render_category(lines),
        "type_image_map": extract_type_image_map(lines),
        "size_dispatch": extract_size_dispatch(lines),
        "mark_removed": extract_mark_removed(lines),
        "mark_des_removed": extract_mark_des_removed(lines),
        "floor_category_override": extract_floor_category_override(lines),
        "bump_reveal": extract_bump_reveal(lines),
    }
    _self_check(data)
    return data


def _self_check(data: dict) -> None:
    """提取形状断言（域尺寸/表长）——形状错立即炸，不流到生成侧。"""
    if len(data["anim_row_by_type"]["values"]) != TYPE_HI - TYPE_LO + 1:
        raise ExtractionError("animRowByType 项数 != 89")
    if len(data["type_frame_width"]["values"]) != TYPE_HI - TYPE_LO:
        raise ExtractionError("typeFrameWidth 项数 != 88")
    if len(data["anim_offset_table"]["rows"]) != 11:
        raise ExtractionError("animOffsetTable 行数 != 11")
    if len(data["resource_names"]["values"]) != 16:
        raise ExtractionError("resourceNames 项数 != 16")
    if len(data["resource_image_counts"]["values"]) != 16:
        raise ExtractionError("resourceImageCounts 项数 != 16")
    if len(data["seg67_pieces"]["rows"]) != 9:
        raise ExtractionError("SEG67 片件数 != 9")
    if len(data["seg67_frames"]["rows"]) != 3:
        raise ExtractionError("SEG67 段帧行数 != 3")


if __name__ == "__main__":
    import json
    data = extract(Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_JAVA)
    print(json.dumps(data, ensure_ascii=False, indent=2))
