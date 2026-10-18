#!/usr/bin/env python3
"""package_win.py —— Windows 分发打包规则（2026-10-18 用户需求：给朋友一个压缩包）。

打包规则（docs/spec/distribution.md 同步）：
  dist/xiyou-win/
    xiyou.exe            # shell/win 二进制（cargo build --release）
    assets/raw/**        # 游戏资源（extract.py 产物，137 文件）
    assets/audio/**      # BGM OGG（5 文件）
    README.txt           # 运行/操作/存档说明（本脚本生成）
  → dist/xiyou-win-<git短sha或日期>.zip（deflate）

**分发红线（2026-10-18 用户指正）**：原版字节码与清单不得入包——`*.class`
（a.class/CMidlet.class）与 `*.MF` 是代码/工程面、运行时零读取；包内只留
运行时媒体（容器/图/MID）。assets/raw 内保留它们仅供 reference/oracle
工具链，打包步骤黑名单剔除 + 把关自检（出现即打包失败）。

原理：游戏运行时默认取 **cwd 相对** `assets/raw` 与 `assets/audio`（main.rs 参数区），
存档默认 `exe目录\save`（decision_save-path-shell-injection）——故包内保持
exe 与 assets/ 同级，双击即跑，零配置。朋友机器需 Windows（winmm 系统自带）；
中文字体走 %WINDIR%\Fonts 系统字体，无需捆绑。

用法：python tools/package_win.py [--out dist] [--skip-build]
"""
from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BIN = "xiyou.exe"


def git_rev() -> str:
    try:
        r = subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT,
                           capture_output=True, text=True, encoding="utf-8", errors="replace")
        return r.stdout.strip() or "unknown"
    except OSError:
        return "unknown"


def build() -> Path:
    print("[package] cargo build --release -p game-desktop …")
    r = subprocess.run(["cargo", "build", "--release", "-p", "game-desktop"], cwd=ROOT)
    if r.returncode != 0:
        sys.exit("[package] 构建失败（cargo 非 0）")
    exe = ROOT / "target" / "release" / "xiyou.exe"
    if not exe.exists():
        sys.exit(f"[package] 缺二进制 {exe}")
    return exe


def stage(out: Path, exe: Path) -> Path:
    name = "xiyou-win"
    stage_dir = out / name
    if stage_dir.exists():
        # 暂存目录可能被资源管理器/杀软短暂占用（WinError 32）——换名自适应，
        # 不让打包因锁崩溃（2026-10-18 实测）。
        try:
            shutil.rmtree(stage_dir)
        except PermissionError:
            for i in range(2, 50):
                cand = out / f"{name}.staging{i}"
                if not cand.exists():
                    stage_dir = cand
                    break
            else:
                sys.exit("[package] 暂存目录全被占用，清理 dist/ 后重试")
    stage_dir.mkdir(parents=True, exist_ok=True)

    shutil.copy2(exe, stage_dir / BIN)

    # 分发红线：原版字节码/工程面绝不入包——`*.class`（a.class/CMidlet.class）、
    # **META-INF/ 整目录**（JAR 清单/签名，2026-10-18 用户指正：目录本身即剔除）。
    # 运行时零读取；assets/raw 内保留仅供 reference/oracle 工具链。
    EXCLUDE_SUFFIX = {".class", ".mf"}
    EXCLUDE_DIRS = {"META-INF"}

    for sub in ("assets/raw", "assets/audio"):
        src = ROOT / sub
        if not src.is_dir():
            sys.exit(f"[package] 缺 {sub}（先跑 tools/extract-assets/extract.py）")
        dst = stage_dir / sub
        shutil.copytree(src, dst,
                        ignore=shutil.ignore_patterns("*.class", "*.MF", "*.mf", *EXCLUDE_DIRS))
        for bad in list(dst.rglob("*")):
            if bad.is_dir() and bad.name.upper() in EXCLUDE_DIRS:
                shutil.rmtree(bad)
            elif bad.is_file() and bad.suffix.lower() in EXCLUDE_SUFFIX:
                bad.unlink()

    # 把关自检：包内出现 class/MF/META-INF = 直接失败（防黑名单被绕过）
    leaked = [p for p in stage_dir.rglob("*")
              if p.is_file() and (p.suffix.lower() in EXCLUDE_SUFFIX
                                  or any(part.upper() in EXCLUDE_DIRS for part in p.parts))]
    if leaked:
        sys.exit(f"[package] 分发红线泄漏：{[str(p) for p in leaked]}")

    n_raw = sum(1 for _ in (stage_dir / "assets/raw").rglob("*") if _.is_file())
    n_au = sum(1 for _ in (stage_dir / "assets/audio").rglob("*") if _.is_file())

    # 一键解锁（去 MOTW 下载标记——SmartScreen 硬拦截的主因；2026-10-18 实机反馈）
    (stage_dir / "解除锁定.bat").write_bytes((
        "@echo off\r\n"
        "cd /d %~dp0\r\n"
        "echo 正在解除下载标记（Microsoft Defender SmartScreen 拦截主因）...\r\n"
        "powershell -NoProfile -Command \"Get-ChildItem -Recurse -File | Unblock-File\"\r\n"
        "echo 完成！现在双击 xiyou.exe 启动（如仍见提示，点【更多信息】→【仍要运行】）。\r\n"
        "pause\r\n").encode("gbk", errors="replace")
    )

    (stage_dir / "README.txt").write_text(
        "《囧囧西游-大闹天宫》重制版（Windows）\n"
        "========================================\n\n"
        "运行：双击 xiyou.exe（请保持 assets 文件夹与 exe 同级、勿移动）。\n"
        "首次启动若弹【Windows 已保护你的电脑】：点【更多信息】→【仍要运行】即可；\n"
        "点过一两次后系统记住选择，之后不再提示（无签名程序的正常流程）。\n"
        "（若整个文件夹是网上下载的，可先双击【解除锁定.bat】去掉下载标记。）\n\n"
        "操作：方向键=移动 / -5(Enter)=确认 / -6、-7=软键（左右菜单键）。\n"
        "存档：自动存于 exe 同级 save\\ 文件夹（可整体拷走续档）。\n"
        "音频：BGM 取自 assets\\audio；无声或嫌吵可用参数 --mute 启动。\n"
        "高级参数：--assets <dir>（资源目录）/ --save <dir>（存档目录）\n"
        "         --audio <dir>（BGM 目录）/ --audio-device <name>。\n\n"
        f"包内容：xiyou.exe + assets/raw（{n_raw} 文件）+ assets/audio（{n_au} 文件）。\n"
        "说明：游戏资产源自原版 JAR（SNOWFISH 2010），本包仅供私人分享试玩。\n",
        encoding="utf-8",
    )
    return stage_dir


def zipit(stage_dir: Path, out: Path, rev: str) -> Path:
    zpath = out / f"xiyou-win-{rev}.zip"
    if zpath.exists():
        zpath.unlink()
    with zipfile.ZipFile(zpath, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for p in sorted(stage_dir.rglob("*")):
            if p.is_file():
                z.write(p, p.relative_to(stage_dir.parent).as_posix())
    return zpath


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="dist")
    ap.add_argument("--skip-build", action="store_true")
    a = ap.parse_args()

    out = (ROOT / a.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    exe = ROOT / "target" / "release" / "xiyou.exe" if a.skip_build else build()
    stage_dir = stage(out, exe)
    rev = git_rev()
    z = zipit(stage_dir, out, rev)
    mb = z.stat().st_size / 1024 / 1024
    print(f"[package] ✅ {z}（{mb:.1f} MB）")
    print(f"[package] 暂存目录 {stage_dir}（可先本地双击自测再发）")


if __name__ == "__main__":
    main()
