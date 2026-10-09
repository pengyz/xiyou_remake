#!/usr/bin/env python3
"""cfr_env.py — CFR 0.152 反编译器的定位、完整性校验与获取（跨平台，纯 Python）。

单一真相源：CFR 的版本、下载地址、sha256 与查找策略都只在这里维护
（`tools/decompile.py` 与 `tools/rename-pipeline/run.py` 共用）。

查找顺序（每一步都校验 sha256，**绝不使用未校验副本**）：
  1. `tools/vendor/cfr-0.152.jar` —— 入库的依赖副本（首选：全新 clone 即离线可用）；
  2. `analysis/cfr.jar` —— 历史缓存位置（兼容旧工作区，不入库）；
  3. Python `urllib` 下载到 `tools/vendor/cfr-0.152.jar`（**不依赖 curl**，Windows 亦可）。

校验失败 ⇒ 抛 [`CfrIntegrityError`]，提示手动放置（vendor）或走下载兜底；
本模块下载的文件校验失败即删除，不污染缓存。
"""
from __future__ import annotations

import hashlib
import urllib.error
import urllib.request
from pathlib import Path

CFR_VERSION = "0.152"
CFR_FILE_NAME = f"cfr-{CFR_VERSION}.jar"
CFR_URL = f"https://repo1.maven.org/maven2/org/benf/cfr/{CFR_VERSION}/{CFR_FILE_NAME}"
# CFR 0.152 完整 sha256：本地实算 + 与 Maven Central cfr-0.152.jar.sha256 交叉核对一致
# （t9-F2 供应链校验；变更必须重新与上游核对，禁止只改其一）
CFR_SHA256 = "f686e8f3ded377d7bc87d216a90e9e9512df4156e75b06c655a16648ae8765b2"
CFR_SHA1 = "48ef4892cfe8feffddbbd0ff077735140557db74"  # 上游 .sha1，交叉核对用


class CfrIntegrityError(RuntimeError):
    """CFR 副本缺失/校验失败（拒绝使用未校验的反编译器）。"""


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def vendor_path(root: Path) -> Path:
    """入库依赖副本路径（tools/vendor/cfr-0.152.jar）。"""
    return root / "tools" / "vendor" / CFR_FILE_NAME


def _verified(path: Path) -> Path:
    got = sha256_file(path)
    if got != CFR_SHA256:
        raise CfrIntegrityError(
            f"CFR sha256 校验失败：{path}\n"
            f"  期望 {CFR_SHA256}\n  实际 {got}\n"
            f"拒绝使用未校验的反编译器。请手动下载 CFR {CFR_VERSION}（{CFR_URL}）核对哈希后"
            f"放到 {path}（或经审计的 vendor 副本 tools/vendor/{CFR_FILE_NAME}）；"
            f"离线/缓存用法见 docs/knowledge/reference_cfr_decompiler.md")
    return path


def ensure_cfr(root: Path, log=print) -> Path:
    """返回校验通过的 cfr.jar 路径；缺则按 vendor → 缓存 → 下载 的顺序获取。"""
    vendor = vendor_path(root)
    cache = root / "analysis" / "cfr.jar"

    if vendor.exists():
        return _verified(vendor)
    if cache.exists():
        return _verified(cache)

    vendor.parent.mkdir(parents=True, exist_ok=True)
    log(f"[cfr_env] 下载 CFR {CFR_VERSION} … {CFR_URL}")
    try:
        with urllib.request.urlopen(CFR_URL, timeout=60) as resp, open(vendor, "wb") as out:
            out.write(resp.read())
    except (urllib.error.URLError, OSError) as e:
        if vendor.exists():
            vendor.unlink()
        raise CfrIntegrityError(
            f"CFR 下载失败（{e}）。离线环境请手动下载 CFR {CFR_VERSION}（{CFR_URL}）"
            f"核对 sha256={CFR_SHA256} 后放到 {vendor}；"
            f"用法见 docs/knowledge/reference_cfr_decompiler.md") from e
    try:
        return _verified(vendor)
    except CfrIntegrityError:
        vendor.unlink()  # 清掉未校验的下载物，不污染缓存
        raise
