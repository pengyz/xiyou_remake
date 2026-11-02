#!/usr/bin/env python3
"""field_ownership.py — GameScene 字段唯一归属对账（task-15 V2.0 域切纪律件）。

用法：
  python3 tools/field_ownership.py --check    # 归属表完整性：覆盖 struct 全字段+唯一归属
  python3 tools/field_ownership.py --report   # 写点分布报告（域切进度度量：跨界写计数）

表 = data/status/field-ownership.json（v3 §1 域切纪律：每字段唯一归属——防"新两处真相"）。
双写定义：同字段直接写点（self.<f> = / += / .push 等）落在**非归属域**文件（跨界写）。
"""
from __future__ import annotations
import json, re, sys, glob, collections
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TABLE = ROOT / "data/status/field-ownership.json"
SCENE = ROOT / "crates/game-core/src/scene.rs"


def struct_fields() -> list:
    lines = SCENE.read_text(encoding="utf-8").splitlines()
    si = next(i for i, l in enumerate(lines) if "pub struct GameScene" in l)
    depth = 0; k = si; started = False
    while k < len(lines):
        depth += lines[k].count("{") - lines[k].count("}")
        if "{" in lines[k]:
            started = True
        if started and depth == 0:
            break
        k += 1
    out = []
    for l in lines[si:k]:
        m = re.match(r"\s+(pub(\([^)]*\))? )?([a-z_][a-z0-9_]*):", l)
        if m and not l.strip().startswith("//"):
            out.append(m.group(3))
    # 域切片成员（PersistState 等）同入归属面（2026-11-02 勘误：私有字段+切片面补全）
    for fname, sname in [("crates/game-core/src/scene_persist.rs", "PersistState"),
                         ("crates/game-core/src/tap_walk.rs", "TapState"),
                         ("crates/game-core/src/scene_dialog.rs", "DialogState"),
                         ("crates/game-core/src/scene_menu.rs", "MenuState")]:
        in_s = False
        for l in (ROOT / fname).read_text(encoding="utf-8").splitlines():
            if l.startswith("pub struct " + sname):
                in_s = True
                continue
            if in_s:
                if l.startswith("}"):
                    break
                m = re.match(r"\s+(pub(\([^)]*\))? )?([a-z_][a-z0-9_]*):", l)
                if m and not l.strip().startswith("//"):
                    out.append(m.group(3))
    return out


def write_sites() -> dict:
    pat = re.compile(r"self\.([a-z_][a-z0-9_]*)\s*(?:=[^=]|\+=|-=|\|=|&=|\*=|/=|\.push\(|\.clear\(|\.pop\(|\.insert\(|\.remove\()")
    out = collections.defaultdict(set)
    for f in glob.glob(str(ROOT / "crates/game-core/src/*.rs")) + glob.glob(str(ROOT / "crates/game-core/src/scene_tests/*.rs")):
        for m in pat.finditer(Path(f).read_text(encoding="utf-8")):
            out[m.group(1)].add(Path(f).name)
    return out


def main() -> int:
    fields = struct_fields()
    tab = json.loads(TABLE.read_text(encoding="utf-8"))["fields"]
    owners = {}
    bad = []
    for e in tab:
        if e["field"] in owners:
            bad.append(f"双归属：{e['field']}（{owners[e['field']]} vs {e['domain']}）")
        owners[e["field"]] = e["domain"]
    missing = [f for f in fields if f not in owners]
    extra = [f for f in owners if f not in fields]
    if missing:
        bad.append(f"表缺字段 {len(missing)}：{missing[:8]}")
    if extra:
        bad.append(f"表多余字段 {len(extra)}：{extra[:8]}")
    if "--check" in sys.argv or "--report" not in sys.argv:
        if bad:
            print("[RED] 归属表：")
            for b in bad:
                print("  " + b)
            return 1
        print(f"[GREEN] 归属表 {len(owners)}/{len(fields)} 字段唯一归属（零双写/零缺口）")
    if "--report" in sys.argv:
        sites = write_sites()
        cross = [(f, sorted(sites.get(f, [])), owners.get(f)) for f in fields
                 if len(sites.get(f, [])) > 1]
        print(f"[报告] 多文件直写字段={len(cross)}（域切进度度量——每域收口后应降）")
        for f, fs, dom in cross[:15]:
            print(f"  {f} ({dom}): {fs}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
