#!/usr/bin/env python3
"""fields.py — trace FLD 索引 ↔ 字段语义名 映射器。

oracle trace 的状态向量按 **class 字段声明序（0 基）** 输出 `FLD NNN <type> <value>`，
不含符号名。本工具从 rename-pipeline 的 renamed class（javap -p 字段序）生成映射，
使 trace 可语义化阅读（如 gameMode=FLD010、playerHp=FLD074、playerPixelX=FLD083）。

用法：
  python3 reference/oracle/fields.py                    # 列出全量映射
  python3 reference/oracle/fields.py gameMode playerHp  # 按名查索引
  python3 reference/oracle/fields.py --index 010 074    # 按索引查名
  python3 reference/oracle/fields.py --timeline <trace.txt> [字段…]
                                                        # 打印 gameMode（默认）+指定字段的
                                                        # 变更时间线（语义化读 trace 的入口）

确定性：映射只依赖 renamed class 的字段声明序（管线幂等 ⇒ 映射稳定）。
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
RENAMED = ROOT / "analysis/rename-pipeline/renamed/a.class"

# 高频语义字段（--timeline 默认集）
TIMELINE_DEFAULTS = ["gameMode", "playerHp", "currentFloor", "playerPixelX", "playerPixelY"]


def field_names() -> list:
    out = subprocess.run(["javap", "-p", str(RENAMED)], capture_output=True, text=True).stdout
    names = []
    for line in out.splitlines():
        l = line.strip()
        if not l or "(" in l or l.startswith(("Compiled from", "public final class", "class ")):
            continue
        names.append(l.split()[-1].rstrip(";"))
    return names


def build_map() -> dict:
    names = field_names()
    return {i: n for i, n in enumerate(names)}


def timeline(trace: Path, wants: list):
    m = build_map()
    inv = {v: k for k, v in m.items()}
    sel = {}
    for w in wants:
        if w not in inv:
            print(f"未知字段: {w}（可用：见 fields.py 全量列表）", file=sys.stderr)
            sys.exit(2)
        sel[inv[w]] = (w, None)
    ticks = {i: (w, None) for i, (w, _) in sel.items()}
    cur = 0
    print(f"# tick | " + " | ".join(w for w, _ in sel.values()))
    rows = []
    last = {}
    for line in open(trace, encoding="utf-8", errors="replace"):
        mt = re.match(r"TICK (\d+)", line)
        if mt:
            cur = int(mt.group(1))
            continue
        for i, (w, _) in sel.items():
            mm = re.search(rf"FLD {i:03d} (?:int|byte|short|long|boolean) (\S+)", line)
            if mm:
                v = mm.group(1)
                if last.get(w) != v:
                    last[w] = v
                    rows.append((cur, w, v))
    for t, w, v in rows:
        print(f"{t:5d} | {w} = {v[:60]}")


def main() -> int:
    args = sys.argv[1:]
    m = build_map()
    if not args:
        for i, n in sorted(m.items()):
            print(f"FLD {i:03d} = {n}")
        return 0
    if args[0] == "--index":
        inv = {v: k for k, v in m.items()}
        for idx in args[1:]:
            print(f"FLD {int(idx):03d} = {m.get(int(idx), '?')}")
        return 0
    if args[0] == "--timeline":
        if len(args) < 2:
            print("用法: --timeline <trace.txt> [字段…]", file=sys.stderr)
            return 2
        wants = args[2:] or TIMELINE_DEFAULTS
        timeline(Path(args[1]), wants)
        return 0
    inv = {v: k for k, v in m.items()}
    for name in args:
        print(f"{name} = FLD {inv.get(name, '?'):03d}" if name in inv else f"{name}: 未找到")
    return 0


if __name__ == "__main__":
    sys.exit(main())
