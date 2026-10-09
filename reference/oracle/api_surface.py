#!/usr/bin/env python3
"""api_surface.py — 参考版外部 API 面 vs shim 覆盖核对。

扫描 a.class / CMidlet.class 常量池中 owner 位于 javax/microedition/ 与
com/nokia/ 的 Fieldref/Methodref/InterfaceMethodref，逐条在 shim 编译产物
（oracle/_out/classes）中做「类存在 + 成员沿继承链可解析」检查，产出
reference/oracle/_diff/api-surface.md。java/* 引用仅列账（由 JDK 提供）。
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from ttransform import CP, UTF8, CLASS, NAT, FIELDREF, METHODREF, IMETHODREF  # noqa: E402

SHIM_PREFIXES = ("javax/microedition/", "com/nokia/")


def member_refs(cp: CP):
    out = []
    for e in cp.entries:
        if e is None:
            continue
        tag, payload, _, _ = e
        if tag in (FIELDREF, METHODREF, IMETHODREF):
            import struct
            cls_idx, nat_idx = struct.unpack_from(">HH", payload, 1)
            t2, payload2, _, _ = cp.entries[nat_idx]
            assert t2 == NAT
            name_idx, desc_idx = struct.unpack_from(">HH", payload2, 1)
            owner = cp.utf8(cp.class_name_idx(cls_idx))
            kind = {FIELDREF: "field", METHODREF: "method", IMETHODREF: "imethod"}[tag]
            out.append((owner, cp.utf8(name_idx), cp.utf8(desc_idx), kind))
    return out


def parse_class(data: bytes):
    import struct
    cp = CP(data)
    this_idx, super_idx = struct.unpack_from(">HH", data, cp.cp_end + 2)
    # 跳过 access(2) this(2) super(2) interfaces_count(2) + interfaces
    p = cp.cp_end + 2 + 2 + 2 + 2
    (if_count,) = struct.unpack_from(">H", data, cp.cp_end + 6)
    ifaces = []
    for i in range(if_count):
        (idx,) = struct.unpack_from(">H", data, p)
        ifaces.append(cp.utf8(cp.class_name_idx(idx)))
        p += 2

    def read_members(pos):
        (count,) = struct.unpack_from(">H", data, pos)
        pos += 2
        members = set()
        for _ in range(count):
            _acc, name_idx, desc_idx, attr_count = struct.unpack_from(">HHHH", data, pos)
            pos += 8
            members.add((cp.utf8(name_idx), cp.utf8(desc_idx)))
            for _ in range(attr_count):
                (ln,) = struct.unpack_from(">I", data, pos + 2)
                pos += 6 + ln
        return members, pos

    fields, p = read_members(p)
    methods, p = read_members(p)
    return {
        "this": cp.utf8(cp.class_name_idx(this_idx)),
        "super": cp.utf8(cp.class_name_idx(super_idx)) if super_idx else None,
        "ifaces": ifaces,
        "fields": fields,
        "methods": methods,
    }


def load_shim(classes_dir: Path):
    table = {}
    for f in sorted(classes_dir.rglob("*.class")):
        info = parse_class(f.read_bytes())
        table[info["this"]] = info
    return table


def resolve(table, owner, name, desc, kind, seen=None):
    if owner not in table:
        return None  # 类不存在
    info = table[owner]
    members = info["methods"] if kind != "field" else info["fields"]
    if (name, desc) in members:
        return True
    seen = seen or set()
    if owner in seen:
        return False
    seen.add(owner)
    for up in [info["super"]] + info["ifaces"]:
        if up and resolve(table, up, name, desc, kind, seen):
            return True
    return False


def main():
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("--game", action="append", required=True, help="游戏 class 文件（原始版）")
    ap.add_argument("--classes", required=True, help="shim+host 编译输出目录")
    ap.add_argument("--out", required=True, help="输出 markdown")
    args = ap.parse_args()

    table = load_shim(Path(args.classes))
    refs = {}
    for g in args.game:
        for owner, name, desc, kind in member_refs(CP(Path(g).read_bytes())):
            refs[(owner, name, desc, kind)] = True

    shim_rows, jdk_rows, missing = [], [], []
    for (owner, name, desc, kind) in sorted(refs):
        if not owner.startswith(SHIM_PREFIXES):
            jdk_rows.append((owner, name, desc, kind))
            continue
        ok = resolve(table, owner, name, desc, kind)
        if ok:
            shim_rows.append((owner, name, desc, kind))
        else:
            missing.append((owner, name, desc, kind))

    lines = []
    lines.append("# API 面核对（自动生成，确定性）")
    lines.append("")
    lines.append("扫描对象：`a.class` + `CMidlet.class` 常量池全部 Fieldref/Methodref/"
                 "InterfaceMethodref；shim 覆盖判定 = 类存在且成员沿继承链可解析"
                 "（解析的是 reference/shim/ 编译产物）。")
    lines.append("")
    lines.append(f"- 引用总数（去重）：{len(refs)}")
    lines.append(f"- MIDP/Nokia API（shim 提供）：{len(shim_rows)}，缺失 {len(missing)}")
    lines.append(f"- JDK API（java/*，未列入 shim 责任）：{len(jdk_rows)}")
    lines.append("")
    lines.append("## shim 覆盖清单（MIDP/Nokia）")
    lines.append("")
    lines.append("| # | owner | kind | name | desc |")
    lines.append("|---|-------|------|------|------|")
    for i, (owner, name, desc, kind) in enumerate(shim_rows, 1):
        lines.append(f"| {i} | `{owner}` | {kind} | `{name}` | `{desc}` |")
    lines.append("")
    if missing:
        lines.append("## ✗ 缺失（必须补齐或显式登记）")
        lines.append("")
        lines.append("| owner | kind | name | desc |")
        lines.append("|-------|------|------|------|")
        for owner, name, desc, kind in missing:
            lines.append(f"| `{owner}` | {kind} | `{name}` | `{desc}` |")
    else:
        lines.append("## ✓ 缺失 0 条")
    lines.append("")
    lines.append("## JDK 引用（存档）")
    lines.append("")
    lines.append("| owner | kind | name | desc |")
    lines.append("|-------|------|------|------|")
    for owner, name, desc, kind in jdk_rows:
        lines.append(f"| `{owner}` | {kind} | `{name}` | `{desc}` |")
    lines.append("")
    Path(args.out).write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"api-surface: refs={len(refs)} shim={len(shim_rows)} missing={len(missing)} jdk={len(jdk_rows)}")
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
