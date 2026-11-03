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
                         ("crates/game-core/src/scene_menu.rs", "MenuState"),
                         ("crates/game-core/src/scene_paint.rs", "RenderState")]:
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
    """【v6·终章度量】GameScene/域成员直写扫描：
    - 匹配 self./sc./scene./sc2. 收件 + 域成员形（self.<domain>.<field>）；
    - **他型同名字段排除**（BootMachine/TitleMachine 家文件的 self. 写=他型自有面，
      非 GameScene 跨域直写——paint.rs/title.rs/menu_family.rs/logo_anim.rs）；
    - 判定口径：字段直写文件数 >1 = 跨域直写（终章清零目标）。
    """
    pat = re.compile(r"\b(?:self|sc|scene|sc2)\.(?:(?:persist|tap|dialog|menu|render)\.)?([a-z_][a-z0-9_]*)\s*(?:=[^=]|\+=|-=|\|=|&=|\*=|/=|\.push\(|\.clear\(|\.pop\(|\.insert\(|\.remove\()")
    OTHER_HOME = {"paint.rs", "title.rs", "menu_family.rs", "logo_anim.rs",
                  "game_view.rs", "render.rs"}  # 【终章簇4】clip_w/h=GameView/RenderSurface 自有字段（他型混计排除核对留痕）
    out = collections.defaultdict(set)
    # 【v6.1 口径定案】生产面（src/）直写=域访问面；测试面（tests//scene_tests/
    # =夹具播种）不计——lead "~15 处"口径即生产面（终章清零目标域）。
    for f in glob.glob(str(ROOT / "crates/game-core/src/*.rs")) \
           + glob.glob(str(ROOT / "crates/game-oracle/src/*.rs")) + glob.glob(str(ROOT / "shell/*/src/*.rs")):
        base = Path(f).name
        txt = Path(f).read_text(encoding="utf-8")
        for m in pat.finditer(txt):
            pre = txt[max(0, m.start()-6):m.start()]
            if base in OTHER_HOME and not re.search(r"\b(sc|scene|sc2)\.$", pre):
                continue  # 他型 self. 写排除
            out[m.group(1)].add(base)
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
        print(f"[报告·全口径] 多文件直写字段={len(cross)}（含 shell/oracle 收件面）")
        # 【终章定案口径】生产面 self. 直写（lead 2026-11-03 "~15 处"口径=终值判据）
        # 直写=赋值/复合赋值（=,+=,-=,|=,&=,*=,/=）；self.<f>.method()=域对象操作
        # （undo.push 等=接口调用非直写——lead"编排可调接口"口径）不计
        selfpat = re.compile(r"\bself\.(?:(?:persist|tap|dialog|menu|render)\.)?([a-z_][a-z0-9_]*)\s*(?:=[^=]|\+=|-=|\|=|&=|\*=|/=)")
        OTHER = {"paint.rs", "title.rs", "menu_family.rs", "logo_anim.rs", "game_view.rs", "render.rs"}
        selfsites = collections.defaultdict(set)
        for f in glob.glob(str(ROOT / "crates/game-core/src/*.rs")):
            b = Path(f).name
            if b in OTHER:
                continue
            for m in selfpat.finditer(Path(f).read_text(encoding="utf-8")):
                selfsites[m.group(1)].add(b)
        selfcross = [(f, sorted(selfsites.get(f, [])), owners.get(f)) for f in fields
                     if len(selfsites.get(f, [])) > 1]
        print(f"[报告·定案口径] 生产面 self 直写跨文件字段={len(selfcross)}（**终章清零判据**）")
        for f, fs, dom in selfcross:
            print(f"  {f} ({dom}): {fs}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
