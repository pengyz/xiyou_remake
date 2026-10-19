#!/usr/bin/env python3
"""modern_frame_baseline.py — modern 帧自证基线（task-25 ⑤，font-metrics §5 协议）。

**夹具协议**（viewport V3 同款）：modern 期望仅源自 **modern 预设运行** 的参照帧——
禁从 classic 期望派生（防洗期望）；自证冻结（非与 classic 差分等价，2c 自证基线同款）。

用法：
  python3 tools/modern_frame_baseline.py          # 固化：modern 运行产帧 → sha 冻结
  python3 tools/modern_frame_baseline.py --check  # 校验：重跑比对基线 sha（漂移=红）

基线 = data/status/baselines/modern-frame-baseline.json（canonical 状态×帧 sha）。
状态集=headless 自验通道五面（boot/title/intro/game/menu）——字体观感主战场。
"""
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "data/status/baselines/modern-frame-baseline.json"
BIN = ROOT / "target/release/xiyou"

# canonical 状态（modern 预设运行；帧点=复现性稳态窗）
STATES = [
    # (键, 参数表)——headless 自验通道五面
    ("boot", ["--frames", "60"]),
    ("title", ["--frames", "90"]),
    ("intro", ["--frames", "280", "--headless-newgame"]),
    ("game", ["--frames", "900", "--headless-newgame"]),
    ("menu", ["--frames", "200", "--inject", "95:-4,100:-4,120:-5"]),
]


def record(out_dir: Path) -> dict:
    """modern 预设运行产帧（自证来源=modern 渲染自身，禁 classic 派生）。"""
    entries: dict[str, dict] = {}
    for key, args in STATES:
        png = out_dir / f"{key}.png"
        cmd = [str(BIN), "--preset", "modern", "--headless-shot", str(png)] + args
        r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
        if r.returncode != 0 or not png.exists():
            print(f"[RED] {key} 帧产出失败 rc={r.returncode}", file=sys.stderr)
            sys.exit(1)
        b = png.read_bytes()
        entries[key] = {
            "sha256": hashlib.sha256(b).hexdigest(),
            "bytes": len(b),
        }
    return entries


def main() -> int:
    check = "--check" in sys.argv
    if not BIN.exists():
        print("[RED] 先构建：cargo build --release -p game-desktop", file=sys.stderr)
        return 1
    with tempfile.TemporaryDirectory(prefix="modern-frames-") as td:
        entries = record(Path(td))
    if not entries:
        print("[RED] 无 modern 帧产出", file=sys.stderr)
        return 1

    if check:
        if not MANIFEST.exists():
            print("[RED] 基线缺失（先固化）", file=sys.stderr)
            return 1
        old = json.loads(MANIFEST.read_text(encoding="utf-8"))["states"]
        drift = []
        for k in sorted(set(old) | set(entries)):
            if k not in old:
                drift.append(f"+新增 {k}")
            elif k not in entries:
                drift.append(f"-缺失 {k}")
            elif old[k]["sha256"] != entries[k]["sha256"]:
                drift.append(f"~字节漂移 {k}: {old[k]['sha256'][:12]}→{entries[k]['sha256'][:12]}")
        if drift:
            print("[RED] modern 帧基线漂移（modern 运行 ≠ 自证冻结基线）：")
            for d in drift:
                print("  " + d)
            return 1
        print(f"[GREEN] modern 帧基线逐字节等价：{len(entries)} 状态 sha 全同")
        return 0

    doc = {
        "schema": "modern-frame-baseline-1",
        "purpose": (
            "task-25 ⑤ modern 帧自证基线（font-metrics §5 夹具协议）：modern 期望仅源自 "
            "modern 预设运行——禁从 classic 期望派生（防洗期望）；自证冻结非差分等价。"
        ),
        "probe": "python3 tools/modern_frame_baseline.py --check",
        "states": entries,
    }
    MANIFEST.parent.mkdir(parents=True, exist_ok=True)
    MANIFEST.write_text(json.dumps(doc, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"[OK] modern 帧基线固化：{len(entries)} 状态 → {MANIFEST.relative_to(ROOT)}")
    for k, e in sorted(entries.items()):
        print(f"  {k}: {e['sha256'][:16]}… {e['bytes']}B")
    return 0


if __name__ == "__main__":
    sys.exit(main())
