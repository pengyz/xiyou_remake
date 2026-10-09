#!/bin/bash
# 安装 git hooks（幂等）
set -e
cd "$(dirname "$0")/.."
chmod +x gates/hooks/pre-commit gates/hooks/commit-msg
ln -sf ../../gates/hooks/pre-commit .git/hooks/pre-commit
ln -sf ../../gates/hooks/commit-msg .git/hooks/commit-msg
echo "hooks installed:"
ls -la .git/hooks/pre-commit .git/hooks/commit-msg
