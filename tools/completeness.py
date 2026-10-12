#!/usr/bin/env python3
"""tools/completeness.py — 实现完整度测量（方法论见 docs/spec/completeness.md）。

原则：测试全绿 ≠ 实现完整。完整度 = 已验证行为（分子，登记带证据）÷ 参考系
全集（分母，从仓库机械导出）。本工具只计算**机械可导出**的分母与场景清单；
"已验证等级"（L0-L4）由人在 completeness.md 登记、每迭代语义回流时更新，
工具对其做存在性校验（引用的 scenario/trace 必须存在）。

用法：
  python3 tools/completeness.py            # 打印机械分母报告（确定性输出）
  python3 tools/completeness.py --check    # 校验 completeness.md 登记行的证据指针存在
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEOBF = ROOT / "reference/src/deobf/a.java"
REMAP = ROOT / "data/naming/remap-table.json"
SCEN = ROOT / "reference/oracle/scenarios"
FIX = ROOT / "crates/game-oracle/tests/fixtures"
SPEC_DOC = ROOT / "docs/spec/completeness.md"

# DSL 指令码表（权威：docs/spec/p3-script-interpreter.md §指令语义）。
# MOT/SKY 是 RMS 店名前缀、非指令——源码串扫描会误报，以此白名单为准。
DSL_OPCODES = ["ADD", "CES", "DES", "END", "GIN", "GLV", "GUT",
               "LAY", "MOV", "RES", "ROS", "SEE", "SMS", "SWD", "TAK"]

TOTAL_FLOORS = 55  # maplv 0-54（assets-manifest 门禁口径）


def paint_run_arms() -> dict:
    """提取 deobf a.java 中 paint()/run() 的**模式 switch 顶层非空 case 臂**。

    锚定 `switch (this.gameMode) {` 行并做花括号配平——排除方法内嵌套 switch
    （如菜单导航的 case 52/56）与空 fallthrough 组（paint case 23-99、
    run case 15/17 空操作组合法为空）。方法语义权威 = docs/findings/state-machine.md。
    """
    lines = Path(DEOBF).read_text(encoding="utf-8").split("\n")

    def switch_line(anchor: str) -> int:
        return next(i for i, l in enumerate(lines) if l.strip().startswith(anchor))

    def top_arms(switch_idx: int) -> tuple[list[int], list[int]]:
        depth, started = 0, False
        cur: list[int] = []
        acc: list[str] = []
        arms: dict[int, list[str]] = {}
        for i in range(switch_idx, len(lines)):
            l = lines[i]
            if not started:
                if re.search(r"switch \(this\.gameMode\) \{", l):
                    started = True
                    depth = l.count("{") - l.count("}")
                continue
            depth += l.count("{") - l.count("}")
            if depth <= 0:
                break
            m = re.match(r"\s*case (\d+):", l)
            if m and depth == 1:
                cur, acc = [int(m.group(1))], []
                continue
            if re.match(r"\s*default:", l) and depth == 1:
                cur, acc = [], []
                continue
            if cur:
                acc.append(l)
                if re.match(r"\s*(break|return)\b|label\d+;", l):
                    for c in cur:
                        arms[c] = list(acc)
                    cur, acc = [], []
        nonempty = sorted(c for c, ls in arms.items()
                          if any(re.sub(r"//.*", "", x).strip() not in ("", "break;") for x in ls))
        return nonempty, sorted(arms)

    p_ne, _ = top_arms(switch_line("switch (this.gameMode) {"))
    # run() 的模式 switch 是同文件第二处 `switch (this.gameMode)`
    idxs = [i for i, l in enumerate(lines) if l.strip().startswith("switch (this.gameMode) {")]
    r_ne, _ = top_arms(idxs[-1])
    return {"paint_nonempty": p_ne, "run_nonempty": r_ne}


def symbol_counts() -> dict:
    rt = json.loads(REMAP.read_text(encoding="utf-8"))
    kinds = {}
    for s in rt["symbols"]:
        kinds[s["kind"]] = kinds.get(s["kind"], 0) + 1
    return {"total": len(rt["symbols"]), "by_kind": kinds,
            "renamed": rt["counts"]["renamed"], "kept": rt["counts"]["kept"]}


def rust_modes() -> dict:
    out = {"scene": [], "paint": [], "boot": []}
    for key, path in [("scene", "crates/game-core/src/scene.rs"),
                      ("paint", "crates/game-core/src/paint.rs")]:
        t = Path(ROOT / path).read_text(encoding="utf-8")
        out[key] = sorted(set(int(m) for m in re.findall(r"(?:game_mode|self\.mode) == (\d+)", t)))
    return out


def scenario_inventory() -> list[dict]:
    rows = []
    for f in sorted(SCEN.glob("*.txt")):
        head = f.read_text(encoding="utf-8").splitlines()[:30]
        cover = next((m.group(1) for l in head
                      if (m := re.match(r"#\s*cover:\s*(.+)", l))), "")
        preset = next((m.group(1) for l in head
                       if (m := re.match(r"#\s*preset:\s*(\S+)", l))), "")
        ticks = next((int(m.group(1)) for l in head
                      if (m := re.match(r"#\s*ticks:\s*(\d+)", l))), 0)
        rows.append({"scenario": f.name, "ticks": ticks,
                     "preset": preset or None, "cover": cover})
    return rows


def floor_fixtures() -> list[int]:
    return sorted(int(m.group(1)) for p in FIX.glob("maplv*.bin")
                  if (m := re.match(r"maplv(\d+)\.bin", p.name)))


def main() -> int:
    check = "--check" in sys.argv
    arms = paint_run_arms()
    syms = symbol_counts()
    modes = rust_modes()
    scen = scenario_inventory()
    floors = floor_fixtures()

    print("# 完整度机械分母（确定性输出；方法论见 docs/spec/completeness.md）")
    print()
    print("## A. 符号面（remap-table，机械）")
    print(f"- 总符号 {syms['total']} = 字段 {syms['by_kind'].get('field', 0)} + 方法 {syms['by_kind'].get('method', 0)}"
          f"（重命名 {syms['renamed']}，保留 {syms['kept']}）")
    sym = symbol_surface()
    tot = sym["total"]
    print(f"- applied 方法分子（Rust 覆盖）：fn 直证 {len(sym['fn_direct'])}/{tot}，"
          f"内联候选 {len(sym['inline_candidate'])}（人工确认），未移植候选 {len(sym['missing_candidate'])}")
    print(f"  未移植候选清单：{' '.join(sym['missing_candidate'])}")
    print()
    print("## B. 模式面（paint/run 非空 case 臂，机械提取）")
    print(f"- paint() 非空臂 {len(arms['paint_nonempty'])}: {arms['paint_nonempty']}")
    print(f"- run()   非空臂 {len(arms['run_nonempty'])}: {arms['run_nonempty']}")
    print(f"- Rust 显式判等 mode：scene {modes['scene']} / paint {modes['paint']}")
    print()
    print("## C. 内容面")
    print(f"- DSL 指令码 {len(DSL_OPCODES)}: {'/'.join(DSL_OPCODES)}")
    print(f"- 楼层夹具已备 {len(floors)}/{TOTAL_FLOORS}: {floors}")
    print()
    print("## D. 场景面（scenarios/*.txt）")
    for r in scen:
        flag = "preset✓" if r["preset"] else "boot  "
        print(f"- {r['scenario']:<24} {flag} {r['ticks']:>5}t  {r['cover'][:60]}")

    if check:
        missing = []
        if SPEC_DOC.exists():
            doc = SPEC_DOC.read_text(encoding="utf-8")
            for ref in re.findall(r"scenario:`([^`]+)`", doc):
                if not (SCEN / ref).exists():
                    missing.append(ref)
            for ref in re.findall(r"trace:`([^`]+)`", doc):
                if not (ROOT / "reference/oracle/_out" / ref / "trace.txt").exists():
                    missing.append(ref)
        if missing:
            print("证据指针缺失:", missing)
            return 1
        print("证据指针校验通过")
    return 0


# ---------- A 维分子：applied 方法 → Rust 三档（fn 直证/内联候选/未移植） ----------

LEDGER = ROOT / "data/naming/ledger.jsonl"


def symbol_surface() -> dict:
    """applied 方法的 Rust 覆盖三档。四信号：语义名/蛇形名/机械名前缀/证据行号引用。
    信号 4 命中 = 内联移植候选（人工确认）；全部未命中 = 未移植候选（人工终审）。"""
    import glob
    led = [json.loads(l) for l in open(LEDGER, encoding="utf-8")]
    methods = [d for d in led if d.get("status") == "applied" and d.get("kind") == "method"]

    def snake(camel):
        return re.sub(r"(?<!^)(?=[A-Z])", "_", camel).lower()

    rust = ""
    for f in glob.glob(str(ROOT / "crates/*/src/**/*.rs"), recursive=True):
        try:
            rust += open(f, encoding="utf-8").read()
        except OSError:
            pass

    fn_direct, inline, missing = [], [], []
    for d in methods:
        n = d["new"]
        m = re.search(r"m_(\d+)", d.get("ref", ""))
        mech = f"m_{m.group(1)}" if m else None
        pats = [rf"fn {n}\b", rf"fn {snake(n)}\b"]
        if mech:
            pats.append(rf"fn \w*{mech}")
        if any(re.search(p, rust) for p in pats):
            fn_direct.append(n)
            continue
        ev_lines = re.findall(r"a\.java:(\d+)", d.get("evidence", ""))
        if ev_lines and any(l in rust for l in ev_lines):
            inline.append(n)
        else:
            missing.append(n)
    return {"fn_direct": fn_direct, "inline_candidate": inline, "missing_candidate": missing,
            "total": len(methods)}


if __name__ == "__main__":
    sys.exit(main())
