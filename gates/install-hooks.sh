#!/bin/bash
# 安装 git hooks（幂等）
set -e
cd "$(dirname "$0")/.."
chmod +x gates/hooks/pre-commit gates/hooks/commit-msg
for h in pre-commit commit-msg; do
    rm -f ".git/hooks/$h"
    # 一律用副本（2026-10-14 Windows 适配）：符号链接跨环境不可靠——WSL 的 ln -s
    # 产出 Linux 专用 reparse tag，Windows 侧 git 报 "cannot spawn
    # .git/hooks/pre-commit: No such file or directory"（实测）。
    # 代价：gates/hooks/* 变更后需重跑本脚本同步。
    cp "gates/hooks/$h" ".git/hooks/$h"
done
echo "hooks installed:"
ls -la .git/hooks/pre-commit .git/hooks/commit-msg
