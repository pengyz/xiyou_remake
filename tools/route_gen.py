#!/usr/bin/env python3
"""route_gen — 手写路线的走步填充与校验（非规划器：航点由人拟定）。

输入：层号 + 航点序列（bump 目标/门/普通格），输出 oracle 输入脚本。
- bump/门航点 = 原地朝向动作（不移动）；格航点 = 逐步走过去。
- 自动 BFS 填充相邻航点间的走步（只走自由格），逐步做三重校验
  （连续性/可走/键向）——floor8/14 教训的固化。

用法：
  python3 tools/route_gen.py 47 --spawn 10,11 --yellow 3 --ticks-out 150 -- \
      bump 63@11,6 -- bump 62@9,5 ...
  航点语法：
    x,y          普通格（走过去）
    bump 型@x,y  战斗/交互 bump（该格为实体格，从邻格朝向撞）
    door x,y     黄门（bump 开 + 走入，消耗一把黄钥）
"""
from __future__ import annotations

import struct
import sys
import zipfile
from collections import deque
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
JAR = ROOT / "original" / "囧囧西游-大闹天宫.jar"
EXTRA_LEN_2 = {7, 8, 76, 82}
EXTRA_LEN_1 = {4, 5, 9, 57, 59, 70, 71, 72, 73, 77, 78, 81, 83}
KEY = {(0, -1): "-1", (0, 1): "-2", (-1, 0): "-3", (1, 0): "-4"}


def extra_len(t: int) -> int:
    return 2 if t in EXTRA_LEN_2 else (1 if t in EXTRA_LEN_1 else 0)


def main() -> int:
    args = sys.argv[1:]
    floor = int(args[0])
    spawn = tuple(int(v) for v in args[args.index("--spawn") + 1].split(","))
    yellow = 1
    if "--yellow" in args:
        yellow = int(args[args.index("--yellow") + 1])
    suffix = ""
    if "--out" in args:
        suffix = args[args.index("--out") + 1]
    tail = args.index("--") + 1

    zf = zipfile.ZipFile(JAR)
    m = zf.read(f"maplv{floor}")
    wide = struct.unpack_from("<H", m, 0)[0] >> 1
    g1 = m[4:4 + wide * 13 * 4]
    walk = [int(x) for x in (ROOT / "crates/game-core/tests/fixtures/tile_walkability.txt").read_text(encoding="utf-8").split()]

    def wk(y, x):
        t = g1[y * wide * 4 + x * 2]
        return 0 <= x < 13 and 0 <= y < 13 and t < len(walk) and walk[t] == 1

    # 实体表：门（黄开）/敌（战后可踏）/其他障碍
    d = zf.read(f"sprite{floor}")
    count = struct.unpack_from("<H", d, 0)[0] | (struct.unpack_from("<H", d, 2)[0] << 16)
    pos = 4
    doors, enemies, obstacles = set(), set(), set()
    for _ in range(count):
        t = d[pos]; pos += 1
        x = struct.unpack_from("<H", d, pos)[0]; pos += 2
        y = struct.unpack_from("<H", d, pos)[0]; pos += 2
        pos += extra_len(t)
        cx, cy = (x + 16) >> 5, (y + 16) >> 5
        if t == 1:
            doors.add((cx, cy))
        elif 41 <= t <= 79:
            enemies.add((cx, cy))
        elif t in (2, 3, 4, 5, 81, 84, 86, 87):
            obstacles.add((cx, cy))

    def passable(x, y):
        return wk(y, x) and ((x, y) not in obstacles or (x, y) in enemies or (x, y) in doors)

    def bfs(a, b):
        seen = {a: None}
        q = deque([a])
        while q:
            cur = q.popleft()
            if cur == b:
                path = []
                while cur != a:
                    path.append(cur)
                    cur = seen[cur]
                return path[::-1]
            x, y = cur
            for dx, dy in ((0, -1), (0, 1), (-1, 0), (1, 0)):
                n = (x + dx, y + dy)
                if n not in seen and passable(*n):
                    seen[n] = cur
                    q.append(n)
        return None

    # 解析航点（shell 会把 "bump x,y" 按空格拆开——按序配对）
    waypoints = []
    toks = args[tail:]
    i = 0
    while i < len(toks):
        tok = toks[i]
        if tok in ("bump", "door") and i + 1 < len(toks):
            tname, xy = (toks[i + 1].split("@", 1) if tok == "bump" else ("1", toks[i + 1]))
            waypoints.append((tok, int(tname), (int(xy.split(",")[0]), int(xy.split(",")[1]))))
            i += 2
            continue
        xy = tok
        waypoints.append(("cell", 0, (int(xy.split(",")[0]), int(xy.split(",")[1]))))
        i += 1

    # 展开：bump = 走到邻格 + 朝向动作；door = 邻格朝向 bump + 走入
    actions = []  # (kind, key, cell)
    cur = spawn
    keys_left = yellow
    for wname, tname, cell in waypoints:
        if wname == "cell":
            path = bfs(cur, cell)
            if path is None:
                print(f"!! 航点 {cell} 不可达 from {cur}", file=sys.stderr)
                return 2
            for c in path:
                actions.append(("step", KEY[(c[0] - cur[0], c[1] - cur[1])], c))
                cur = c
            continue
        if wname == "door":
            if keys_left <= 0:
                print(f"!! 黄钥不足 @ {cell}", file=sys.stderr)
                return 2
            keys_left -= 1
        # bump/door：从「目标的邻格」朝向撞；已在邻格则原地，否则 BFS 过去
        cand = [(cell[0] + dx, cell[1] + dy) for dx, dy in ((0, -1), (0, 1), (-1, 0), (1, 0))
                if wk(cell[1] + dy, cell[0] + dx)
                and ((cell[0] + dx, cell[1] + dy) not in obstacles
                     or (cell[0] + dx, cell[1] + dy) in enemies
                     or (cell[0] + dx, cell[1] + dy) == cur)]
        if cur in cand:
            actions.append((wname, KEY[(cell[0] - cur[0], cell[1] - cur[1])], cell))
            if wname == "bump":
                cur = cell
            continue
        placed = None
        for a in cand:
            path = bfs(cur, a)
            if path is not None:
                k = KEY[(cell[0] - a[0], cell[1] - a[1])]
                placed = (path, a, k)
                break
        if placed is None:
            print(f"!! {wname} {tname}@{cell} 邻格不可达 from {cur}", file=sys.stderr)
            return 2
        path, a, k = placed
        for c in path:
            actions.append(("step", KEY[(c[0] - cur[0], c[1] - cur[1])], c))
            cur = c
        actions.append((wname, k, cell))
        if wname == "bump":
            cur = cell  # 战后 restep

    # 发脚本
    lines = [
        "# ticks: 0", f"# preset: preset-floor{floor}.preset",
        f"# cover: route_gen 生成（航点手拟，走步 BFS 填充 + 三重校验）",
    ]
    lines += ["70 -6 press", "72 -6 release", "100 -2 press", "102 -2 release",
              "150 -5 press", "152 -5 release", "200 -5 press", "202 -5 release"]
    t = 300
    for kind, k, cell in actions:
        lines.append(f"{t} {k} press")
        lines.append(f"{t + 2} {k} release")
        t += {"bump": 400, "door": 200, "step": 60}[kind]
    total = t + 150
    lines[0] = f"# ticks: {total}"
    out = Path(f"reference/oracle/scenarios/preset-floor{floor}{suffix}.txt")
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"OK actions={len(actions)} ticks={total} -> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
