#!/usr/bin/env python3
"""state_baseline.py — state_dump 金样基线固化（refactor-plan §7 条件 1/2 实施前置）。

用法：
  python3 tools/state_baseline.py            # 重跑全场景 dump 并固化 sha 清单
  python3 tools/state_baseline.py --check    # 重跑并与入档清单逐字节核对（重构前后主判据）

机制：
  1. 跑 game-oracle 三个 GameScene 重放面（fixture_frames / preset_cases /
     gameplay_floor1），STATE_OUT=<tmp> 时各测试逐拍导出 state_dump（TICK 块 +
     dump_scene 行）到 `<tmp>/<场景>.state.txt`；
  2. 每场景 sha256 入 data/status/baselines/state-dump-golden.json。

**行序冻结声明（A2）**：dump_scene 现行 push 序（state_dump.rs dump_scene 起始）即
金样字节序——重构（Entity struct/命令模式）**禁按 FLD 序重排**；FLD 行=Java 反射序、
布局迁移天然不影响，两机制分列不混谈。任何行序变更=金样红，属预期（需 lead 裁）。

覆盖口径（2026-10-16）：fx 夹具（fixture_frames label）+ scenarios/ 的 Rust 重放
（preset_cases label）+ gameplay-floor1 窗口。boot_replay/walk_replay_floor1/
probe_floor2 为机级/原语测试（不驱 GameScene）不在 state_dump 面；l2_prologue 为
脚本引擎测（mock harness）同不在——此二者非缺口，是**面不适用**。
"""
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "data/status/baselines/state-dump-golden.json"
TARGETS = ["fixture_frames", "preset_cases", "gameplay_floor1"]


def run_dumps(out_dir: Path) -> int:
    env = dict(**__import__("os").environ, STATE_OUT=str(out_dir))
    cmd = ["cargo", "test", "--release", "-p", "game-oracle"]
    for t in TARGETS:
        cmd += ["--test", t]
    r = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True, text=True)
    sys.stderr.write(r.stderr[-2000:])
    return r.returncode


def collect(out_dir: Path) -> dict:
    entries = {}
    for f in sorted(out_dir.glob("*.state.txt")):
        b = f.read_bytes()
        ticks = b.count(b"\nTICK ") + (1 if b.startswith(b"TICK ") else 0)
        entries[f.stem] = {
            "sha256": hashlib.sha256(b).hexdigest(),
            "bytes": len(b),
            "ticks": ticks,
        }
    return entries


def main() -> int:
    check = "--check" in sys.argv
    with tempfile.TemporaryDirectory(prefix="state-golden-") as td:
        out_dir = Path(td)
        rc = run_dumps(out_dir)
        if rc != 0:
            print(f"[RED] 重放面未全绿 rc={rc}——基线拒绝固化", file=sys.stderr)
            return 1
        entries = collect(out_dir)
    if not entries:
        print("[RED] 无 state dump 产出（STATE_OUT 挂点缺失？）", file=sys.stderr)
        return 1

    if check:
        old = json.loads(MANIFEST.read_text(encoding="utf-8"))
        old_e, new_e = old["scenarios"], entries
        drift = []
        for k in sorted(set(old_e) | set(new_e)):
            if k not in old_e:
                drift.append(f"+新增 {k}")
            elif k not in new_e:
                drift.append(f"-缺失 {k}")
            elif old_e[k]["sha256"] != new_e[k]["sha256"]:
                drift.append(f"~字节漂移 {k}: {old_e[k]['sha256'][:12]}→{new_e[k]['sha256'][:12]}")
        if drift:
            print("[RED] 金样漂移（重构后 dump ≠ HEAD 基线）：")
            for d in drift[:40]:
                print("  " + d)
            return 1
        print(f"[GREEN] 金样逐字节等价：{len(new_e)} 场景 sha 全同")
        return 0

    doc = {
        "schema": "state-dump-golden-1",
        "purpose": "重构前 HEAD 基线（refactor-plan §7 条件 1）：第 1/2 步逐字节对照面",
        "line_order_freeze": (
            "dump_scene 现行 push 序为金样字节序（A2）：TraceState/Entity 迁移适配其序，"
            "禁按 FLD 序重排；FLD 行=Java 反射序两机制分列不混谈。"
        ),
        "probe": "python3 tools/state_baseline.py --check",
        "scenarios": entries,
    }
    MANIFEST.parent.mkdir(parents=True, exist_ok=True)
    MANIFEST.write_text(json.dumps(doc, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    total = sum(e["bytes"] for e in entries.values())
    print(f"[OK] 金样基线固化：{len(entries)} 场景 / {total} 字节 → {MANIFEST.relative_to(ROOT)}")
    for k, e in sorted(entries.items()):
        print(f"  {k}: {e['sha256'][:16]}… {e['bytes']}B {e['ticks']}ticks")
    return 0


if __name__ == "__main__":
    sys.exit(main())
