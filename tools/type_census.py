#!/usr/bin/env python3
"""type_census — spriteN 类型普查（类型闭包账本的派生覆盖判据，type-closure.md §6）。

对指定层（或全部 1-54 层）解析 original/ jar 内 spriteN 的记录类型集，
与已覆盖类型集（--covered，默认账本 ✅/📦 集合的近似：命令行逗号给出）
做差，输出每层的「新增类型」——无增量的层即"普查派生覆盖"。

用法：
  python3 tools/type_census.py 41 44            # 指定层
  python3 tools/type_census.py --all            # 全层扫描
  python3 tools/type_census.py --all --covered 1,3,4,5,60,61,62
"""
from __future__ import annotations

import re
import struct
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
JAR = ROOT / "original" / "囧囧西游-大闹天宫.jar"

# extra 长度表（docs/findings/resource-formats.md；crates/game-data/src/sprite.rs 同源）
EXTRA_LEN_2 = {7, 8, 76, 82}
EXTRA_LEN_1 = {4, 5, 9, 57, 59, 70, 71, 72, 73, 77, 78, 81, 83}


def extra_len(t: int) -> int:
    if t in EXTRA_LEN_2:
        return 2
    if t in EXTRA_LEN_1:
        return 1
    return 0


def census(zf: zipfile.ZipFile, floor: int) -> dict[int, int]:
    """spriteN 记录的类型 → 出现次数。"""
    data = zf.read(f"sprite{floor}")
    count = struct.unpack_from("<H", data, 0)[0] | (struct.unpack_from("<H", data, 2)[0] << 16)
    pos = 4
    types: dict[int, int] = {}
    for _ in range(count):
        t = data[pos]
        pos += 1
        x = struct.unpack_from("<H", data, pos)[0]
        pos += 2
        y = struct.unpack_from("<H", data, pos)[0]
        pos += 2
        pos += extra_len(t)
        types[t] = types.get(t, 0) + 1
    assert pos == len(data), f"sprite{floor}: consumed {pos} != {len(data)}"
    return types


def main() -> int:
    args = sys.argv[1:]
    all_floors = "--all" in args
    floors = (
        list(range(1, 55))
        if all_floors
        else [int(a) for a in args if a.isdigit()]
    )
    covered: set[int] = set()
    if "--covered" in args:
        i = args.index("--covered")
        covered = {int(x) for x in args[args.index("--covered") + 1].split(",") if x}
    with zipfile.ZipFile(JAR) as zf:
        for n in floors:
            types = census(zf, n)
            delta = sorted(set(types) - covered)
            flag = "增量" if delta else "派生覆盖"
            print(
                f"floor{n:02d} [{flag}] types={sorted(types)}"
                + (f" delta={delta}" if delta else "")
            )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
