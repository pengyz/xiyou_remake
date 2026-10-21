#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""机械对照器 CLI（survey §4 R2 三件之一）。

用法（工作目录任意）：
  python3 tools/table-evidence/cli.py extract     # 打印机械提取摘要（第二侧）
  python3 tools/table-evidence/cli.py generate    # 生成式回填 data/tables/*.json
  python3 tools/table-evidence/cli.py compare     # 表值 == 提取值（错位逐行报）
  python3 tools/table-evidence/cli.py check       # compare + 确定性复跑同哈希
  python3 tools/table-evidence/cli.py --check     # 同 check

退出码：0=全绿；1=错位/哈希漂移；2=提取失败或表缺失。
"""
from __future__ import annotations

import hashlib
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from comparator import Comparator, compare  # noqa: E402
from extractor import DEFAULT_JAVA, ExtractionError, extract  # noqa: E402
from generator import build_tables, write_tables  # noqa: E402


def cmd_extract(java: Path) -> int:
    data = extract(java)
    print(f"[extract] 第二侧 = {java}")
    print(f"  animRowByType : {len(data['anim_row_by_type']['values'])} 项 "
          f"（a.java:{data['anim_row_by_type']['decl_line']}-588）")
    print(f"  typeFrameWidth: {len(data['type_frame_width']['values'])} 项 "
          f"（a.java:{data['type_frame_width']['decl_line']}-678）")
    print(f"  animOffsetTable: {len(data['anim_offset_table']['rows'])} 行 "
          f"（a.java:{data['anim_offset_table']['decl_line']}-497）")
    print(f"  resourceNames : {len(data['resource_names']['values'])} 项 "
          f"（值行 a.java:{data['resource_names']['values'][0]['line']}）")
    print(f"  resourceImageCounts: {len(data['resource_image_counts']['values'])} 项 "
          f"（值行 a.java:{data['resource_image_counts']['values'][0]['line']}）")
    print(f"  SEG67 片件/段帧: {len(data['seg67_pieces']['rows'])} / "
          f"{len(data['seg67_frames']['rows'])}（a.java:682-697）")
    print(f"  mark_removed 臂数: {len(data['mark_removed']['arms'])}；"
          f"mark_des_removed: {len(data['mark_des_removed']['arms'])}")
    return 0


def cmd_generate(java: Path) -> int:
    hashes = write_tables(extract(java))
    for name, h in hashes.items():
        print(f"[generate] {h}  data/tables/{name}")
    return 0


def cmd_compare(java: Path) -> int:
    try:
        errors = compare(java)
    except FileNotFoundError as exc:
        print(f"[compare] {exc}")
        return 2
    if errors:
        print(f"[compare] ✗ {len(errors)} 处错位：")
        for e in errors:
            print("  " + str(e))
        return 1
    print("[compare] ✓ 表值 == 机械提取值（等值）")
    return 0


def cmd_check(java: Path) -> int:
    """compare + 确定性复跑：内存重建两次与盘上三表三方同哈希。"""
    rc = cmd_compare(java)
    if rc != 0:
        return rc
    data = extract(java)
    rebuilt = build_tables(data)
    again = build_tables(extract(java))  # 第二次独立复跑
    ok = True
    for name, blob in sorted(rebuilt.items()):
        h1 = hashlib.sha256(blob).hexdigest()
        h2 = hashlib.sha256(again[name]).hexdigest()
        on_disk = (Path(__file__).resolve().parents[2] / "data" / "tables" / name).read_bytes()
        h3 = hashlib.sha256(on_disk).hexdigest()
        mark = "✓" if h1 == h2 == h3 else "✗"
        if h1 != h2 or h1 != h3:
            ok = False
        print(f"[check] {mark} {name} 复跑1={h1[:16]} 复跑2={h2[:16]} 盘上={h3[:16]}")
    if not ok:
        print("[check] ✗ 确定性复跑哈希不一致")
        return 1
    print("[check] ✓ 确定性复跑同哈希 + 等值全绿")
    return 0


def main(argv: list[str]) -> int:
    args = [a for a in argv if not a.startswith("-")]
    flags = [a for a in argv if a.startswith("-")]
    java = Path(args[0]) if args else DEFAULT_JAVA
    cmd = args[1] if len(args) > 1 else (args[0] if args and Path(args[0]).suffix == ".java" else "")
    if args and Path(args[0]).suffix == ".java":
        java = Path(args[0])
        cmd = args[1] if len(args) > 1 else ""
    elif args:
        java = DEFAULT_JAVA
        cmd = args[0]
    if cmd in ("check",) or "--check" in flags:
        return cmd_check(java)
    if cmd == "extract":
        return cmd_extract(java)
    if cmd == "generate":
        return cmd_generate(java)
    if cmd == "compare":
        return cmd_compare(java)
    print(__doc__)
    return 2


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv[1:]))
    except ExtractionError as exc:
        print(f"[extract] ✗ 机械提取失败: {exc}")
        sys.exit(2)
