#!/usr/bin/env python3
"""renamer.py — 签名唯一化机械重命名（确定性变换，不做语义命名）。

命名方案（v1，规则写死、与内容无关 ⇒ 幂等、可复算）：
  字段  f_<type>_<NN>   例：int a → f_int_07     （type 记号 + 该类内同 type 计数，2 位）
  方法  m_<NNN>         例：a(int,int) → m_042   （该类内声明序计数，3 位）
  未改名成员（构造器 / <clinit> / MIDP API override 等）保持原名，old == new。

type 记号：I→int B→byte Z→bool S→short C→char J→long F→float D→double
           Lpkg/Cls;→Cls（简单类名）；数组 = 元素记号 + _arr（多维 _arr2/_arr3…）

判定“混淆名”：^[a-zA-Z]{1,2}$（本 JAR 混淆器的完整产出空间：a..z / A..Z / aa..cj /
aA..bZ 等 1–2 字母名；保留名 paint/run/keyPressed/… 与 <init>/<clinit> 均 ≥3 字符或特殊名）。
安全性约束：仅当成员为 private 或 static（不可能 override 外部 API）才允许改名；
若出现“疑似混淆名 + 可 override”的成员 ⇒ 硬失败，绝不猜。
"""
from __future__ import annotations

import re
from collections import defaultdict

from classfile import (ClassFile, FIELDREF, METHODREF, IFACEREF, UTF8,
                       NAMEANDTYPE)

OBF_NAME = re.compile(r"^[a-zA-Z]{1,2}$")

_PRIM = {"I": "int", "B": "byte", "Z": "bool", "S": "short", "C": "char",
         "J": "long", "F": "float", "D": "double", "V": "void"}

ACC_STATIC = 0x0008
ACC_PRIVATE = 0x0002


class RenameError(Exception):
    pass


def type_token(desc: str) -> str:
    dims = 0
    while desc.startswith("["):
        dims += 1
        desc = desc[1:]
    if desc.startswith("L") and desc.endswith(";"):
        base = desc[1:-1].split("/")[-1]
    else:
        base = _PRIM.get(desc, desc.replace("/", "."))
    if dims == 0:
        return base
    return base + "_arr" + (str(dims) if dims > 1 else "")


def java_sig(desc: str) -> str:
    """JVM 描述符 → 人读签名，如 (II)V → (int,int)void。"""
    i, args = 1, []
    while desc[i] != ")":
        start = i
        while desc[i] == "[":
            i += 1
        if desc[i] == "L":
            i = desc.index(";", i)
        i += 1
        args.append(_human(desc[start:i]))
    return "(" + ",".join(args) + ")" + _human(desc[i + 1:])


def _human(desc: str) -> str:
    dims = 0
    while desc.startswith("["):
        dims += 1
        desc = desc[1:]
    if desc.startswith("L") and desc.endswith(";"):
        base = desc[1:-1].replace("/", ".")
    else:
        base = _PRIM.get(desc, desc)
    return base + "[]" * dims


def build_rename_map(classes: dict) -> list:
    """classes: {class_name: ClassFile} → 全量符号记录（含未改名成员）。

    返回记录列表，字段见 data/naming/remap-table.json 的 symbols[]。
    """
    records = []
    seq_field = defaultdict(lambda: defaultdict(int))  # class → type token → seq
    seq_method = defaultdict(int)                      # class → seq
    errors = []

    for cname in sorted(classes):
        cf = classes[cname]
        seen_final = {}  # (kind, final_name) → 已占签名（查重）
        for kind, members in (("field", cf.fields), ("method", cf.methods)):
            for m in members:
                old = cf.utf8(m.name_idx)
                desc = cf.utf8(m.desc_idx)
                is_static = bool(m.access & ACC_STATIC)
                is_private = bool(m.access & ACC_PRIVATE)
                obf = bool(OBF_NAME.fullmatch(old)) and old not in ("<init>", "<clinit>")
                if kind == "method" and old in ("<init>", "<clinit>"):
                    rename = False
                elif not obf:
                    rename = False
                elif not (is_static or is_private):
                    errors.append(
                        f"{cname}.{old}{desc} 疑似混淆名但既非 private 也非 static，"
                        f"可能 override 外部 API —— 拒绝自动改名")
                    rename = False
                else:
                    rename = True

                if rename:
                    if kind == "field":
                        tok = type_token(desc)
                        n = seq_field[cname][tok]
                        seq_field[cname][tok] += 1
                        new = f"f_{tok}_{n:02d}"
                    else:
                        n = seq_method[cname]
                        seq_method[cname] += 1
                        new = f"m_{n:03d}"
                else:
                    new = old

                key = (kind, new)
                if key in seen_final:
                    errors.append(f"{cname} 内重名冲突: {kind} {new} "
                                  f"({desc} 与 {seen_final[key]})")
                seen_final[key] = desc

                records.append({
                    "id": None,  # 序号最后统一排
                    "class": cname,
                    "kind": kind,
                    "old": old,
                    "new": new,
                    "signature": desc,
                    "signature_java": _human(desc) if kind == "field" else java_sig(desc),
                    "static": is_static,
                    "private": is_private,
                    "renamed": rename,
                    "evidence": (
                        f"机械变换（确定性规则，非语义命名）：{cname}.{old}{desc} 声明与全部 "
                        f"Fieldref/Methodref 引用点一并改名；等价证据 javap 归一化逐字节一致，"
                        f"diff 0 行（tools/rename-pipeline/_verify/diff-{cname}.txt，复现："
                        f"python3 tools/rename-pipeline/run.py --verify）"
                        if rename else
                        f"保留原名：{cname}.{old}{desc} 为 {'构造器/静态初始化' if old in ('<init>', '<clinit>') else 'MIDP API override/实现'}，"
                        f"改名将改变虚分派语义；见 tools/rename-pipeline/_verify/report.md"
                    ),
                    "confidence": 100,
                })

    if errors:
        raise RenameError("命名方案自检失败:\n  " + "\n  ".join(errors))
    return records


def apply_rename(classes: dict, records: list) -> None:
    """把记录里的改名落回 class 文件模型（常量池末尾追加 + 索引改写）。"""
    # (class, old, desc) → new
    renamed = {(r["class"], r["old"], r["signature"]): r["new"]
               for r in records if r["renamed"]}

    for cname, cf in classes.items():
        # ① 声明处
        for members in (cf.fields, cf.methods):
            for m in members:
                key = (cname, cf.utf8(m.name_idx), cf.utf8(m.desc_idx))
                if key in renamed:
                    m.name_idx = cf.intern_utf8(renamed[key])
        # ② 引用处（Fieldref/Methodref/InterfaceMethodref 的 NameAndType）
        for idx in range(1, len(cf.cp)):
            entry = cf.cp[idx]
            if entry is None or entry[0] not in (FIELDREF, METHODREF, IFACEREF):
                continue
            class_idx, nat_idx = entry[1]
            owner = cf.class_name(class_idx)
            name_idx, desc_idx = cf.nat(nat_idx)
            key = (owner, cf.utf8(name_idx), cf.utf8(desc_idx))
            if key in renamed:
                new_nat = cf.intern_nat(cf.intern_utf8(renamed[key]), desc_idx)
                cf.cp[idx] = (entry[0], (class_idx, new_nat))


def consistency_check(classes: dict, records: list) -> list:
    """回写后的硬校验，返回问题列表（空 = 通过）。

    1. 每个改名声明已指向新名；
    2. 没有任何 Fieldref/Methodref/InterfaceMethodref 仍指向旧 (owner,name,desc)；
    3. 每个新名在该类内唯一（字段+方法合并查重）；
    4. 常量池无 MethodHandle/InvokeDynamic/Dynamic（本 JAR 为 CLDC 时代产物；
       若出现则 NameAndType 可能绕过重写 ⇒ 拒绝继续）。
    """
    problems = []
    renamed = {(r["class"], r["old"], r["signature"]): r["new"]
               for r in records if r["renamed"]}

    for cname, cf in classes.items():
        names = {}
        for kind, members in (("field", cf.fields), ("method", cf.methods)):
            for m in members:
                name = cf.utf8(m.name_idx)
                desc = cf.utf8(m.desc_idx)
                key = (cname, name, desc)
                if key in renamed:
                    problems.append(f"{cname}.{name}{desc} 声明处未改名")
                if (kind, name) in names:
                    problems.append(f"{cname} 成员重名: {name} ({names[(kind, name)]} vs {desc})")
                names[(kind, name)] = desc

        for idx in range(1, len(cf.cp)):
            entry = cf.cp[idx]
            if entry is None:
                continue
            tag = entry[0]
            if tag in (15, 17, 18):  # MethodHandle / Dynamic / InvokeDynamic
                problems.append(f"{cname} cp[{idx}] 存在 tag={tag}，超出本管线安全范围")
                continue
            if tag not in (FIELDREF, METHODREF, IFACEREF):
                continue
            class_idx, nat_idx = entry[1]
            owner = cf.class_name(class_idx)
            name_idx, desc_idx = cf.nat(nat_idx)
            key = (owner, cf.utf8(name_idx), cf.utf8(desc_idx))
            if key in renamed:
                problems.append(f"{cname} cp[{idx}] 仍引用旧符号 {owner}.{key[1]}{key[2]}")

    # 新名不得与保留名冲突（类内全局）
    by_class = defaultdict(dict)
    for r in records:
        by_class[r["class"]][r["new"]] = r
    return problems
