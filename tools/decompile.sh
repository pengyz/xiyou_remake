#!/bin/bash
# tools/decompile.sh — 从 original/ 一键再生反编译投影（CFR 0.152 pinned）
# 用途：交叉验证 reference/seed/a.java 的不可变性；新 class 反编译。
# 病灶层三问备忘：产物是"字节码的投影"，不是真相源；seed 的权威性由
# gates/cli.py reference-seed-integrity 保护，本工具只负责再生/对比。
set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
JAR="$ROOT/original/囧囧西游-大闹天宫.jar"
CFR="$ROOT/analysis/cfr.jar"
CFR_URL="https://repo1.maven.org/maven2/org/benf/cfr/0.152/cfr-0.152.jar"

mkdir -p "$ROOT/analysis/ex" "$ROOT/analysis/decomp"
unzip -oq "$JAR" 'a.class' 'CMidlet.class' -d "$ROOT/analysis/ex"

if [ ! -f "$CFR" ]; then
    echo "下载 CFR 0.152 …"
    curl -sL -o "$CFR" "$CFR_URL"
fi

java -jar "$CFR" "$ROOT/analysis/ex/a.class" --outputdir "$ROOT/analysis/decomp" >/dev/null
java -jar "$CFR" "$ROOT/analysis/ex/CMidlet.class" --outputdir "$ROOT/analysis/decomp" >/dev/null

echo "== 与 reference/seed/ 对比（一致 = seed 未漂移）=="
diff -q "$ROOT/analysis/decomp/a.java" "$ROOT/reference/seed/a.java" \
    && echo "a.java: 与 seed 一致 ✓" \
    || { echo "a.java: 与 seed 不一致 ✗（立即调查，见 AGENTS.md §4）"; exit 1; }
