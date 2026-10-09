#!/bin/bash
# tools/decompile.sh — 薄包装（POSIX 兼容入口）。
# 全部逻辑在 tools/decompile.py（跨平台纯 Python，不依赖 curl/unzip/diff）；
# Windows 请直接用：python3 tools/decompile.py
exec python3 "$(cd "$(dirname "$0")" && pwd)/decompile.py" "$@"
