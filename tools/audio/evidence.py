#!/usr/bin/env python3
"""evidence.py — BGM 选轨语义的运行时取证（A 级 trace 证据再生入口）。

在 oracle 变体 A（原始字节码 + T-变换，reference/oracle/run.py 同一管线）上运行
`tools/audio/evidence/*.txt` 输入脚本，逐 tick 全量转储（dumpStride=1），输出
`reference/oracle/_out/EV-<stem>/trace.txt`（gitignore，可再生），并打印音频相关
时间线：EVT（Manager shim 的 createPlayer/start/close… 事件）+ gameMode/soundEnabled/
volumeLevel/f_byte_28/f_bool_28/anyKeyPlayed 字段变更。

不修改 reference/（只 import run.py 的构建/运行函数）；脚本头部 `# preset:` 路径相对
本脚本所在目录解析。docs/spec/audio.md 的证据表引用本工具产出的 tick 号。

用法：
  python3 tools/audio/evidence.py            # 全部取证脚本
  python3 tools/audio/evidence.py toggle-exit
"""
from __future__ import annotations

import importlib.util
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
ORACLE = ROOT / "reference/oracle"
EVID = HERE / "evidence"

AUDIO_FIELDS = ["gameMode", "soundEnabled", "volumeLevel", "f_byte_28", "f_bool_28",
                "anyKeyPlayed"]


def load_oracle():
    spec = importlib.util.spec_from_file_location("oracle_run", ORACLE / "run.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def load_fields():
    spec = importlib.util.spec_from_file_location("oracle_fields", ORACLE / "fields.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def header(script: Path, key: str):
    for line in script.read_text(encoding="utf-8").splitlines()[:30]:
        m = re.match(rf"#\s*{key}:\s*(\S+)", line)
        if m:
            return m.group(1)
    return None


def timeline(trace: Path, idx_of: dict) -> list:
    """[(tick, [events…], {field: value 变更})]——仅含有变化的 tick。

    归属规则：EVT 行由 postTick(n) 的 Runner.flushEvents() 先于 `TICK n` 行刷出
    （TickHooks.postTick），事件发生于 logic#n 或 preTick(n) 的 keyPressed ⇒ 归属 tick n；
    FLD 转储在 `TICK n` 之后（同一 postTick），反映 logic#n + preTick(n) 投递后的状态。"""
    wanted = {v: k for k, v in idx_of.items()}
    rows, cur, pending, last = [], None, [], {}
    for line in trace.read_text(encoding="utf-8").splitlines():
        if line.startswith("EVT "):
            ev = line[4:]
            if ev not in ("thread-start", "thread-exit"):
                pending.append(ev)
        elif line.startswith("TICK "):
            if cur and (cur[1] or cur[2]):
                rows.append(cur)
            cur = (int(line.split()[1]), pending, {})
            pending = []
        elif line.startswith("  FLD ") and cur is not None:
            # "  FLD 375 boolean true" → ['', '', 'FLD', '375', 'boolean', 'true']
            parts = line.split(" ", 5)
            try:
                idx = int(parts[3])
            except (IndexError, ValueError):
                continue
            if idx in wanted:
                val = parts[5] if len(parts) > 5 else ""
                name = wanted[idx]
                if last.get(name) != val:
                    cur[2][name] = val
                    last[name] = val
    if cur and (cur[1] or cur[2]):
        rows.append(cur)
    return rows


def print_timeline(label: str, trace: Path, idx_of: dict):
    for tick, evts, ch in timeline(trace, idx_of):
        kv = " ".join(f"{k}={v}" for k, v in ch.items())
        ev = ("  EVT[" + "; ".join(evts) + "]") if evts else ""
        print(f"  T{tick:04d} {kv}{ev}")


def main(argv) -> int:
    oracle = load_oracle()
    fields = load_fields()
    fmap = {v: k for k, v in fields.build_map().items()}
    idx_of = {name: fmap[name] for name in AUDIO_FIELDS}

    if argv[:1] == ["--trace"]:
        # 只读既有 trace（如 --scenarios 产出的 A-menu-sweep）；EVT 逐 tick 精确，
        # 字段变更受该 trace 的 dumpStride 采样粒度约束（默认 8 + 输入 tick）。
        for p in argv[1:]:
            print(f"== {p}")
            print_timeline(p, Path(p), idx_of)
        return 0

    scripts = sorted(EVID.glob("*.txt"))
    if argv:
        scripts = [s for s in scripts if s.stem in argv]
    if not scripts:
        print("无取证脚本", file=sys.stderr)
        return 2

    classes = oracle.build_shim_host()
    orig = oracle.ensure_game_classes()
    oracle.ttransform_variants(orig, None)
    rc_all = 0
    for script in scripts:
        ticks = oracle.parse_scenario_ticks(script)
        preset = header(script, "preset")
        preset_path = str((script.parent / preset).resolve()) if preset else None
        label = f"EV-{script.stem}"
        rc, out = oracle.run_variant(classes, oracle.OUT / "variantA", label, ticks, script,
                                     frames="off", preset=preset_path, dump_stride=1)
        trace = out / "trace.txt"
        print(f"== {label}（{ticks} ticks, rc={rc}, trace={trace.relative_to(ROOT)}）")
        if rc != 0:
            rc_all = 1
        print_timeline(label, trace, idx_of)
    return rc_all


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
