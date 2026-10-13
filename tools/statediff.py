#!/usr/bin/env python3
"""statediff.py — 状态向量差分器（层次二工具，迭代 72）。

用法：
  python3 tools/statediff.py <a_trace> <rust_state> [--start N] [--limit N]

<a_trace>   ：oracle trace（TICK 块 + FLD 行，字段名经 fields.py 映射）
<rust_state>：state_dump.rs 导出文件（TICK 行 + `name = value` 行）

输出：每字段首分歧 tick + 值对；仅在一侧出现的 tick 也计数。
字段跳过：walkPathBuffer（Rust 活跃路径 vs Java 100 槽陈旧尾，语义不同）。
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "reference" / "oracle"))
import fields as F  # noqa: E402

SKIP = {"walkPathBuffer"}


def parse_trace(path: str, name_of_idx: dict) -> dict:
    """→ {tick: {name: value}}（含数组整串）。"""
    out: dict[int, dict[str, str]] = {}
    cur = None
    for line in open(path, encoding="utf-8", errors="replace"):
        mt = re.match(r"TICK (\d+)", line)
        if mt:
            cur = int(mt.group(1))
            out.setdefault(cur, {})
            continue
        mf = re.match(r"\s*FLD (\d{3}) (\S+) (.+)", line)
        if mf and cur is not None:
            idx = int(mf.group(1))
            name = name_of_idx.get(idx)
            if name:
                out[cur][name] = mf.group(3).strip()
    return out


def parse_rust(path: str) -> dict:
    """→ {tick: {name: value}}。"""
    out: dict[int, dict[str, str]] = {}
    cur = None
    for line in open(path, encoding="utf-8", errors="replace"):
        mt = re.match(r"TICK (\d+)", line)
        if mt:
            cur = int(mt.group(1))
            out.setdefault(cur, {})
            continue
        mm = re.match(r"(\w+) = (.*)", line)
        if mm and cur is not None:
            out[cur][mm.group(1)] = mm.group(2)
    return out


def main() -> int:
    args = sys.argv[1:]
    if len(args) < 2:
        print(__doc__)
        return 2
    start = 0
    if "--start" in args:
        start = int(args[args.index("--start") + 1])
        args = args[:args.index("--start")] + args[args.index("--start") + 2:]
    a = parse_trace(args[0], F.build_map())
    r = parse_rust(args[1])

    # 只比较两侧共有的 tick 与字段名；值文本完全一致才算一致。
    common = sorted(set(a) & set(r))
    if start:
        common = [t for t in common if t >= start]
    first_div: dict[str, tuple] = {}
    div_ticks: set[int] = set()
    checked = 0
    for t in common:
        names = set(a[t]) | set(r[t]) - set(SKIP)
        for n in sorted(names - SKIP):
            va, vr = a[t].get(n), r[t].get(n)
            if va is None or vr is None:
                continue  # 单侧缺失（对齐/容量语义），跳过
            checked += 1
            if va != vr and n not in first_div:
                first_div[n] = (t, va[:80], vr[:80])
                div_ticks.add(t)
    print(f"比对 tick 数={len(common)}  字段比对数={checked}  分歧字段数={len(first_div)}")
    for n, (t, va, vr) in sorted(first_div.items(), key=lambda kv: kv[1][0]):
        print(f"  {n}: 首分歧 T{t}  A={va}  R={vr}")
    if div_ticks:
        print("分歧 tick:", sorted(div_ticks)[:40])
    return 0 if not first_div else 1


if __name__ == "__main__":
    sys.exit(main())
