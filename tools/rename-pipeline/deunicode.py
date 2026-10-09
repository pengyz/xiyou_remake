#!/usr/bin/env python3
"""deunicode.py — deobf 源码中文可读化：字面量内非 ASCII `\\uXXXX` 机械解码（t12）。

规则（确定性、幂等；只改源文本呈现，运行期字符串值零变化）：
  1. 只处理**字符串/字符字面量内部** —— 注释/标识符/其余源码一律不动，
     且解码只产出 ≥ 0x00A0 的字符（永远不是引号/反斜杠）⇒ 源码词法结构零变化；
  2. 码点 ≥ 0x00A0 的 `\\uXXXX` 解码为 UTF-8 明文（含全部中文）；
     ASCII 范围转义（< 0x00A0，如 `\\u0022`/`\\u005C`/`\\u0000`）一律保留，避免词法歧义；
  3. 反向 [`encode_java_literals`]（CFR 规则：非 ASCII 一律 `\\uXXXX` 小写 4 位、
     非 BMP 用 UTF-16 代理对）：回环 `encode(decode(x)) == x` 逐字节成立
     （run.py 构建时硬校验），即「解码产物值不变」的可复算证明。

边界处理与 Java 词法一致：字面量内 `\\\\`（转义反斜杠）等普通转义原样搬运；
`\\\\uXXXX`（反杠被转义）不是 unicode 转义，按字面文本保留。
"""
from __future__ import annotations

import re

# unicode 转义：\u + 4 位 hex
_UNI = re.compile(r"\\u([0-9a-fA-F]{4})")
# CJK 统一表意（常量池对照抽样用）
CJK = re.compile(r"[\u4e00-\u9fff]")


def _decode_inner(inner: str):
    """字面量内部解码，返回 (新文本, 解码字符数, 保留的转义数)。"""
    out, kept, decoded, i, n = [], 0, 0, 0, len(inner)
    while i < n:
        if inner[i] == "\\":
            m = _UNI.match(inner, i)
            if m:
                cp = int(m.group(1), 16)
                end = m.end()
                if 0xD800 <= cp <= 0xDBFF:  # UTF-16 代理对 → 非 BMP 单字符
                    m2 = _UNI.match(inner, end)
                    if m2 and 0xDC00 <= int(m2.group(1), 16) <= 0xDFFF:
                        cp = 0x10000 + ((cp - 0xD800) << 10) + (int(m2.group(1), 16) - 0xDC00)
                        end = m2.end()
                if cp >= 0x00A0:
                    out.append(chr(cp))
                    decoded += 1
                else:
                    out.append(inner[i:end])  # ASCII 转义 / 孤立代理：原样保留
                    kept += 1
                i = end
            else:
                out.append(inner[i:i + 2])    # \\ \" \n \' 等普通转义原样（含行尾孤杠）
                kept += 1
                i += 2
        else:
            out.append(inner[i])
            i += 1
    return "".join(out), decoded, kept


def _encode_inner(inner: str) -> str:
    """字面量内部（CFR 规则）：非 ASCII 字符 → \\uXXXX 小写 4 位（非 BMP 用代理对）。"""
    out = []
    for ch in inner:
        cp = ord(ch)
        if cp >= 0x80:
            if cp >= 0x10000:
                v = cp - 0x10000
                out.append(f"\\u{0xD800 + (v >> 10):04x}\\u{0xDC00 + (v & 0x3FF):04x}")
            else:
                out.append(f"\\u{cp:04x}")
        else:
            out.append(ch)
    return "".join(out)


def map_literals(text: str, fn):
    """对每个字符串("…")/字符('…')字面量内容应用 fn(kind, inner) → new_inner。

    注释与字面量外文本原样；返回 (新文本, 处理的字面量数)。
    """
    out, i, n, count = [], 0, len(text), 0
    while i < n:
        c = text[i]
        if text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            out.append(text[i:j])
            i = j
        elif text.startswith("/*", i):
            j = text.find("*/", i + 2)
            j = n if j < 0 else j + 2
            out.append(text[i:j])
            i = j
        elif c in ('"', "'"):
            j, closed = i + 1, False
            while j < n:
                if text[j] == "\\":
                    j += 2
                    continue
                if text[j] == c:
                    j += 1
                    closed = True
                    break
                j += 1
            if closed:
                out.append(c + fn(c, text[i + 1:j - 1]) + c)
            else:
                out.append(text[i:min(j, n)])  # 未闭合字面量：原样（不处理）
            count += 1
            i = j
        else:
            out.append(c)
            i += 1
    return "".join(out), count


def decode_java_literals(text: str):
    """解码：返回 (新文本, 字面量数, 解码字符数, 保留转义数)。"""
    totals = {"decoded": 0, "kept": 0}

    def fn(_kind, inner):
        new, decoded, kept = _decode_inner(inner)
        totals["decoded"] += decoded
        totals["kept"] += kept
        return new

    out, n_literals = map_literals(text, fn)
    return out, n_literals, totals["decoded"], totals["kept"]


def encode_java_literals(text: str) -> str:
    """反向编码（CFR 规则）：回环校验 encode(decode(x)) == x 用。"""
    out, _ = map_literals(text, lambda _kind, inner: _encode_inner(inner))
    return out


def string_literal_values(text: str) -> list:
    """提取字符串字面量（"…"，不含字符字面量）的内容列表，供常量池 UTF-8 对照抽样。"""
    values = []

    def fn(kind, inner):
        if kind == '"':
            values.append(inner)
        return inner

    map_literals(text, fn)
    return values
