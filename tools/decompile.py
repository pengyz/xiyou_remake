#!/usr/bin/env python3
"""decompile.py — 从 original/ 一键再生反编译投影（CFR 0.152，sha256 钉死）。

用途：交叉验证 `reference/seed/a.java` 的不可变性；新 class 反编译。
病灶层三问备忘：产物是"字节码的投影"，不是真相源；seed 的权威性由
`gates/cli.py reference-seed-integrity` 保护，本工具只负责再生/对比。

跨平台（Windows/macOS/Linux）：纯 Python 标准库（zipfile/urllib/difflib/subprocess），
不依赖 curl/unzip/diff/sha256sum；`tools/decompile.sh` 只是 POSIX 薄包装。
CFR 副本的定位与校验见 [`cfr_env`](cfr_env.py)（vendor 入库 → analysis 缓存 → urllib 下载）。

用法：
  python3 tools/decompile.py            # 等价 bash tools/decompile.sh
"""
from __future__ import annotations

import subprocess
import sys
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
sys.path.insert(0, str(HERE))

from cfr_env import ensure_cfr  # noqa: E402

JAR = ROOT / "original/囧囧西游-大闹天宫.jar"
EX = ROOT / "analysis/ex"
DECOMP = ROOT / "analysis/decomp"
SEED = ROOT / "reference/seed/a.java"
CLASSES = ("a.class", "CMidlet.class")


def main() -> int:
    if not JAR.exists():
        print(f"original JAR 缺失: {JAR}", file=sys.stderr)
        return 1
    cfr = ensure_cfr(ROOT)
    EX.mkdir(parents=True, exist_ok=True)
    DECOMP.mkdir(parents=True, exist_ok=True)

    with zipfile.ZipFile(JAR) as z:
        for name in CLASSES:
            (EX / name).write_bytes(z.read(name))

    for name in CLASSES:
        r = subprocess.run(
            ["java", "-jar", str(cfr), str(EX / name), "--outputdir", str(DECOMP)],
            capture_output=True, text=True)
        if r.returncode != 0:
            print(f"CFR 反编译失败 {name}:\n{r.stdout[-2000:]}\n{r.stderr[-2000:]}",
                  file=sys.stderr)
            return 1

    print("== 与 reference/seed/ 对比（一致 = seed 未漂移）==")
    regenerated = DECOMP / "a.java"
    if not regenerated.exists():
        print(f"反编译产物缺失: {regenerated}", file=sys.stderr)
        return 1
    if regenerated.read_bytes() == SEED.read_bytes():
        print("a.java: 与 seed 一致 ✓")
        return 0
    print("a.java: 与 seed 不一致 ✗（立即调查，见 AGENTS.md §4）", file=sys.stderr)
    import difflib
    diff = difflib.unified_diff(
        SEED.read_text(encoding="utf-8").splitlines(keepends=True),
        regenerated.read_text(encoding="utf-8").splitlines(keepends=True),
        fromfile="reference/seed/a.java", tofile="analysis/decomp/a.java")
    sys.stderr.writelines(list(diff)[:80])
    return 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as e:  # noqa: BLE001 —— 工具入口统一失败出口
        print(f"decompile FAIL: {e}", file=sys.stderr)
        sys.exit(1)
