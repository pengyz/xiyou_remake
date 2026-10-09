#!/usr/bin/env python3
"""verifier.py — javap 指令级等价验证：证明重映射“仅符号名变化、语义不变”。

方法（A 级硬证据，可复现）：
  1. javap -c -p -l 分别输出改名前 / 改名后 class 的逐指令反汇编；
  2. 归一化：把常量池槽号 #NNN → #<cp>（追加 Utf8/NameAndType 必然挤动槽号，
     槽号是布局而非语义）；对“改名后”输出按映射表把新符号名**还原**为旧名；
  3. 若两份归一化文本逐字节相同 ⇒ 指令序列、操作数所指的具体成员、行号表、
     异常表、签名、访问标志全部一致，唯一差异就是符号名。

还原替换是**上下文敏感**的：只在 成员声明行 与 `// Field|Method|InterfaceMethod`
注释 里替换，字符串常量等其他文本不动。被限定的引用（owner.name:desc）按 owner
查映射，非限定引用按当前类查映射。
"""
from __future__ import annotations

import difflib
import hashlib
import re
from collections import defaultdict

INSTR_LINE = re.compile(r"^\s+\d+:\s+\S+")
DECL_LINE = re.compile(
    r"^  (?=\S)(?:public|protected|private|static|final|abstract|native|synchronized|"
    r"transient|volatile|strictfp| |\t)*[\w.$/\[\]<>, ]+?\s+(\w+)\s*[;(]")
CTOR_DECL = re.compile(r"^  (?=\S)\S.*\b(\w+)\s*\([^)]*\)\s*;?\s*\{?\s*$")
REF_COMMENT = re.compile(r"(Field|Method|InterfaceMethod) (.*)$")
CP_SLOT = re.compile(r"#\d+")


def javap_normalize(text: str, reverse_map_by_class: dict, cur_class: str) -> str:
    """归一化 javap 输出；reverse_map_by_class: {class: {new_name: old_name}}。"""
    out = []
    for line in text.splitlines():
        line = CP_SLOT.sub("#<cp>", line)
        # javap 注释固定为行内首个 "// " 之后的内容；只认 Field|Method|InterfaceMethod 引用
        idx = line.find("// ")
        m = REF_COMMENT.match(line[idx + 3:]) if idx != -1 else None
        if m:
            kind, rest = m.group(1), m.group(2)
            owner, name, tail = _split_ref(rest)
            new = _unrename(reverse_map_by_class, owner or cur_class, name)
            if new != name:
                # 仅在确有改名时重建 rest（构造器等带引号名不在映射表内 ⇒ 原样保留）
                rest = (owner + "." if owner else "") + new + tail
                line = line[:idx + 3] + kind + " " + rest
        else:
            d = DECL_LINE.match(line)
            if d:
                old = d.group(1)
                new = _unrename(reverse_map_by_class, cur_class, old)
                if new != old:
                    line = line[:d.start(1)] + new + line[d.end(1):]
        out.append(line)
    return "\n".join(out) + "\n"


def _split_ref(rest: str):
    """"a.b:(II)V" → ("a", "b", ":(II)V")；"a:(II)V" → ("", "a", ":(II)V")。"""
    if ":" in rest:
        head, desc = rest.split(":", 1)
        tail = ":" + desc
    else:
        head, tail = rest, ""
    # <init>/<clinit> 带引号：a."<init>" → name = <init>
    quoted = re.fullmatch(r"(.*)\.\"([^\"]+)\"", head)
    if quoted:
        return quoted.group(1), quoted.group(2), tail
    if "." in head:
        owner, name = head.rsplit(".", 1)
        return owner, name, tail
    return "", head, tail


def _unrename(maps: dict, owner: str, name: str) -> str:
    m = maps.get(owner)
    if m and name in m:
        return m[name]
    return name


def split_members(text: str):
    """javap 输出 → [(header_line, name, block_text, instr_count)]。"""
    lines = text.splitlines()
    members, cur, cur_name, cur_header = [], [], None, None
    for line in lines:
        if line.startswith("  ") and not line.startswith("    ") and line.strip():
            if cur_name is not None:
                members.append((cur_header, cur_name, "\n".join(cur), _count_instr(cur)))
            cur, cur_name, cur_header = [line], _decl_name(line), line
        else:
            if cur_name is not None:
                cur.append(line)
    if cur_name is not None:
        members.append((cur_header, cur_name, "\n".join(cur), _count_instr(cur)))
    return members


def _decl_name(line: str) -> str:
    m = DECL_LINE.match(line)
    if m:
        return m.group(1)
    m = CTOR_DECL.match(line)
    if m:
        return m.group(1)
    return line.strip().rstrip(";{").split()[-1]


def _count_instr(lines) -> int:
    return sum(1 for l in lines if INSTR_LINE.match(l))


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def diff_texts(a: str, b: str, fromfile: str, tofile: str) -> str:
    return "".join(difflib.unified_diff(
        a.splitlines(keepends=True), b.splitlines(keepends=True),
        fromfile=fromfile, tofile=tofile))
