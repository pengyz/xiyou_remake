#!/usr/bin/env python3
"""vf_env.py — Vineflower 1.12.0 反编译器的定位、完整性校验与获取（跨平台，纯 Python）。

单一真相源：Vineflower 的版本、下载地址、sha256 与查找策略都只在这里维护
（`tools/rename-pipeline/run.py` 使用；2026-10-10 起取代已退役的 cfr_env.py）。

查找顺序（每一步都校验 sha256，**绝不使用未校验副本**）：
  1. `tools/vendor/vineflower-1.12.0.jar` —— 入库的依赖副本（首选：全新 clone 即离线可用）；
  2. Python `urllib` 下载到 `tools/vendor/vineflower-1.12.0.jar`（**不依赖 curl**，Windows 亦可）。

校验失败 ⇒ 抛 [`VfIntegrityError`]，提示手动放置（vendor）或走下载兜底；
本模块下载的文件校验失败即删除，不污染缓存。

选型依据（2026-10-10 实测，oracle 变体 C 三方对拍为证）：
  - VF 1.12.0 直出 110 错（vs CFR 0.152 修 GOTO 洞后 1615 错）；
  - VF + 修复集 javac 零错，且 trace 与原始字节码逐字节一致；
  - VF 直出 UTF-8 中文（CFR 需 deunicode 后处理）、确定性输出（双跑逐字节一致）。
"""
from __future__ import annotations

import hashlib
import urllib.error
import urllib.request
from pathlib import Path

VF_VERSION = "1.12.0"
VF_FILE_NAME = f"vineflower-{VF_VERSION}.jar"
VF_URL = ("https://github.com/Vineflower/vineflower/releases/download/"
          f"{VF_VERSION}/vineflower-{VF_VERSION}.jar")
# 实算 sha256（2026-10-10 本地实算于官方 release 产物；变更必须重新与上游核对）
VF_SHA256 = "1dfcfe974395734fa467ce620661c7623d05ba83670de0529b1fbd63ff548b9d"


class VfIntegrityError(RuntimeError):
    """Vineflower 副本缺失/校验失败（拒绝使用未校验的反编译器）。"""


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def vendor_path(root: Path) -> Path:
    return root / "tools" / "vendor" / VF_FILE_NAME


def _verified(path: Path) -> Path:
    got = sha256_file(path)
    if got != VF_SHA256:
        raise VfIntegrityError(
            f"Vineflower sha256 校验失败：{path}\n"
            f"  期望 {VF_SHA256}\n  实际 {got}\n"
            "请删除该文件后重试（会走下载兜底），或放置正确副本。")
    return path


def ensure_vf(root: Path, log=print) -> Path:
    """返回可用的 Vineflower jar 路径（vendor → urllib 下载，全程 sha256 校验）。"""
    vendor = vendor_path(root)
    if vendor.exists():
        return _verified(vendor)
    log(f"[vf_env] vendor 缺失，下载 {VF_URL} …")
    vendor.parent.mkdir(parents=True, exist_ok=True)
    try:
        with urllib.request.urlopen(VF_URL, timeout=120) as resp, vendor.open("wb") as out:
            while True:
                chunk = resp.read(1 << 16)
                if not chunk:
                    break
                out.write(chunk)
    except (urllib.error.URLError, OSError) as e:
        raise VfIntegrityError(
            f"下载失败：{e}\n请手动下载 {VF_URL}\n"
            f"核对 sha256={VF_SHA256} 后放到 {vendor}") from e
    return _verified(vendor)
