#!/usr/bin/env python3
"""provision_tests — 再生 cargo test 消费的本地夹具（assets/raw + oracle trace）。

背景（迭代 55）：assets/raw 与 reference/oracle/_out 均不入库（.gitignore），
而 rust-tests 门禁的 replay/preset 测试逐帧对拍这两类夹具——CI checkout
无夹具 ⇒ cargo test 即刻 panic ⇒ gates CI 自工作流建立以来 100+ 次全红。

本脚本使门禁**自供给**（CI 与本地完全等效，无 CI 特例跳过）：
  1. assets/raw 缺失 → 调 tools/extract-assets/extract.py（original/ 在库，秒级）；
  2. 扫描 crates/**.rs 中 `reference/oracle/_out/<label>/trace.txt` 的读取点，
     对缺失的 label 按约定 `A-<stem>` ↔ `scenarios/<stem>.txt` 再生
     （frames=off 省 PNG；dumpStride=8 与现行生成口径一致；FRAME sha
     与 stride 无关，对拍语义不变）。
已存在的 trace 一律不动（可再生但就地有效；再生是确定性等价）。
"""
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
CRATES = ROOT / "crates"
OUT = HERE / "_out"
SCENARIOS = HERE / "scenarios"

REF_RE = re.compile(r'join\("reference/oracle/_out/([A-Za-z0-9_-]+)/trace\.txt"\)')
TICKS_RE = re.compile(r"^#\s*ticks:\s*(\d+)", re.M)
PRESET_RE = re.compile(r"^#\s*preset:\s*(\S+)", re.M)


def scan_required_labels():
    labels = set()
    for rs in CRATES.rglob("*.rs"):
        labels |= set(REF_RE.findall(rs.read_text(encoding="utf-8")))
    return sorted(labels)


def scenario_for(label: str) -> Path:
    """label → 场景脚本（约定 A-<stem>；其余 label 无场景映射即报错）。"""
    if not label.startswith("A-"):
        raise SystemExit(f"provision: label {label} 不符合 A-<stem> 约定，无法映射场景")
    f = SCENARIOS / f"{label[2:]}.txt"
    if not f.exists():
        raise SystemExit(f"provision: {label} 的场景脚本缺失: {f}")
    return f


def main() -> int:
    # 1) 原始资产层
    if not (ROOT / "assets/raw/l0.png").exists():
        import subprocess
        r = subprocess.run([sys.executable, str(ROOT / "tools/extract-assets/extract.py")],
                           capture_output=True, text=True, encoding="utf-8", errors="replace")
        if r.returncode != 0:
            print(r.stdout[-1500:] + r.stderr[-1500:], file=sys.stderr)
            return 2
        print("[provision] assets/raw 已再生（extract-assets）")
    else:
        print("[provision] assets/raw 已存在")

    # 2) oracle trace 夹具
    import importlib.util
    spec = importlib.util.spec_from_file_location("oracle_run", HERE / "run.py")
    run = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(run)

    missing = []
    for label in scan_required_labels():
        if not (OUT / label / "trace.txt").exists():
            missing.append(label)
    if not missing:
        print(f"[provision] oracle trace 齐备（{len(scan_required_labels())} 个 label 无缺失）")
        return 0

    print(f"[provision] 缺失 trace {len(missing)} 个: {', '.join(missing)}")
    classes = run.build_shim_host()
    orig = run.ensure_game_classes()
    run.ttransform_variants(orig, None)

    specs = []
    for label in missing:
        scen = scenario_for(label)
        text = scen.read_text(encoding="utf-8")
        m = TICKS_RE.search(text)
        ticks = int(m.group(1)) if m else 150
        pm = PRESET_RE.search(text)
        preset = str(SCENARIOS / pm.group(1)) if pm and (SCENARIOS / pm.group(1)).exists() else None
        specs.append((classes, run.OUT / "variantA", label, ticks, scen, preset))

    # 批并行（4/批）：单 JVM ~1GB 内，2 核 CI 也不至于抖动。
    # preset 逐 spec 携带（6 元组）——批级共享曾致预置静默丢失（迭代 55 教训：
    # 症状 = 带 preset 场景全部自 T151 槽载入拍起帧分歧，无 preset 场景全吻合）。
    for i in range(0, len(specs), 4):
        batch = specs[i:i + 4]
        res = run.run_variants_parallel(
            [(c, v, l, t, s, p) for (c, v, l, t, s, p) in batch],
            frames="off", dump_stride=8)
        for (c, v, label, ticks, scen, preset) in batch:
            rc, d = res[label]
            if rc != 0 or not (d / "trace.txt").exists():
                print(f"[provision] {label} 再生失败 rc={rc}", file=sys.stderr)
                return 2
            print(f"[provision] {label} 已再生（{ticks} ticks, preset={Path(preset).name if preset else '-'}）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
