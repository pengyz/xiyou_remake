#!/usr/bin/env python3
"""extract-assets — 从 original/ 固化 JAR 再生派生资产（assets/ 一律可再生，AGENTS.md §2）。

产出（全部在 assets/ 下，不入库）：
  assets/raw/   JAR 全部条目原样解包（含资源与 class）
  assets/png/   packed PNG 剥掉 2 字节头后的标准 PNG（魔数校验）
  assets/manifest.json  清单与计数（与 data/status/baselines/baseline.json 对账）

音频 .mid → .ogg 转换未实现（P3 音频阶段需要 fluidsynth，见 tools/README.md）。
红线：本工具只读 original/，修正逻辑写在这里，永不改原始归档。
"""
import json
import re
import struct
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
JAR = ROOT / "original/囧囧西游-大闹天宫.jar"
OUT = ROOT / "assets"
PNG_MAGIC = b"\x89PNG"


def main() -> int:
    if not JAR.exists():
        print(f"JAR 缺失: {JAR}", file=sys.stderr)
        return 1
    raw, png = OUT / "raw", OUT / "png"
    raw.mkdir(parents=True, exist_ok=True)
    png.mkdir(parents=True, exist_ok=True)

    manifest = {"entries": 0, "classes": 0, "maplv": 0, "sprite": 0,
                "packed_png": [], "mid": [], "text": []}
    with zipfile.ZipFile(JAR) as z:
        for name in z.namelist():
            manifest["entries"] += 1   # 口径 = zip 全部条目（含目录条目），与 baseline.json 对账
            data = z.read(name)
            dst = raw / name
            if name.endswith("/"):          # zip 目录条目（如 META-INF/）按目录处理
                dst.mkdir(parents=True, exist_ok=True)
                continue
            dst.parent.mkdir(parents=True, exist_ok=True)  # zip 条目可含子目录
            dst.write_bytes(data)
            if name.endswith(".class"):
                manifest["classes"] += 1
            if re.fullmatch(r"maplv\d+", name):
                manifest["maplv"] += 1
            if re.fullmatch(r"sprite\d+", name):
                manifest["sprite"] += 1
            # packed PNG：2 字节头 + 标准 PNG（考证见 docs/findings/resource-formats.md）
            off = data.find(PNG_MAGIC)
            if off == 2:
                (png / f"{name}.png").write_bytes(data[2:])
                w, h = struct.unpack(">II", data[2 + 16:2 + 24])
                manifest["packed_png"].append(
                    {"name": name, "header": data[:2].hex(), "size": len(data), "png": f"{w}x{h}"})
            if name.endswith(".mid"):
                manifest["mid"].append(name)

    (OUT / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"entries={manifest['entries']} classes={manifest['classes']} "
          f"maplv={manifest['maplv']} sprite={manifest['sprite']} "
          f"packed_png={len(manifest['packed_png'])} mid={len(manifest['mid'])}")
    print(f"输出: {raw} · {png} · {OUT / 'manifest.json'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
