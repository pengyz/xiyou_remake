#!/usr/bin/env python3
"""classfile.py — 最小 class 文件读写器（常量池手术级符号重命名专用）。

设计（对应 AGENTS.md §4 三问）：
- 问题在哪一层：字节码符号层。混淆器让同一类内多个字段/方法共用 a/b/c 等名字
  （JVM 按 name+desc 解析合法，Java 源码层歧义）。本模块只改“名字”，不碰指令。
- 消费者读的是哪个文件：original/ JAR 解出的 .class；Code 属性字节原样搬运。
- 证据级别：A（class 文件字节结构，工具输出可复现）。

重写策略：常量池**末尾追加**新 Utf8/NameAndType 条目，只把
  ① 字段/方法声明的 name_index、
  ② Fieldref/Methodref/InterfaceMethodref 的 name_and_type_index
改指向新条目。所有既有索引不变 ⇒ Code/LineNumberTable 等属性零改动。
绝不在原地改 Utf8 —— Utf8 "a" 可能被类名/字符串常量/局部变量表共享。
"""
from __future__ import annotations

import struct
from dataclasses import dataclass, field as dc_field

# 常量池 tag（JVM 规范 §4.4，2/13/14 未使用）
UTF8 = 1
INTEGER, FLOAT, LONG, DOUBLE = 3, 4, 5, 6
CLASS, STRING = 7, 8
FIELDREF, METHODREF, IFACEREF = 9, 10, 11
NAMEANDTYPE = 12
METHODHANDLE, METHODTYPE = 15, 16
DYNAMIC, INVOKEDYNAMIC = 17, 18
MODULE, PACKAGE = 19, 20

_REF_TAGS = (FIELDREF, METHODREF, IFACEREF)
_WIDE_TAGS = (LONG, DOUBLE)


class ClassFormatError(Exception):
    pass


@dataclass
class Member:
    access: int
    name_idx: int
    desc_idx: int
    attributes: list  # [(attr_name_idx, raw_bytes)]


@dataclass
class ClassFile:
    major: int
    minor: int
    cp: list                 # 1-based: cp[i] = (tag, payload) 或 None（long/double 占位）
    access: int
    this_idx: int
    super_idx: int
    interfaces: list
    fields: list
    methods: list
    attributes: list
    # 末尾追加的新条目登记（tag, payload）→ 已有索引；写回时并入常量池
    pending: list = dc_field(default_factory=list)

    # ---------- 常量池查询（含末尾追加区）----------

    def entry(self, idx: int):
        if 0 < idx < len(self.cp):
            e = self.cp[idx]
            if e is None:
                raise ClassFormatError(f"cp[{idx}] 是 wide 空槽")
            return e
        off = idx - self.cp_base()
        if 0 <= off < len(self.pending):
            return self.pending[off]
        raise ClassFormatError(f"cp[{idx}] 越界（池槽位 {len(self.cp)}，追加 {len(self.pending)}）")

    def utf8(self, idx: int) -> str:
        tag, payload = self.entry(idx)
        if tag != UTF8:
            raise ClassFormatError(f"cp[{idx}] tag={tag} 不是 Utf8")
        return payload.decode("utf-8", "replace")

    def class_name(self, idx: int) -> str:
        tag, payload = self.entry(idx)
        if tag != CLASS:
            raise ClassFormatError(f"cp[{idx}] tag={tag} 不是 Class")
        return self.utf8(payload)

    def nat(self, idx: int):
        tag, payload = self.entry(idx)
        if tag != NAMEANDTYPE:
            raise ClassFormatError(f"cp[{idx}] tag={tag} 不是 NameAndType")
        return payload  # (name_idx, desc_idx)

    def ref(self, idx: int):
        tag, payload = self.entry(idx)
        if tag not in _REF_TAGS:
            raise ClassFormatError(f"cp[{idx}] tag={tag} 不是 Fieldref/Methodref")
        return payload  # (class_idx, nat_idx)

    # ---------- 追加登记（写回时并入常量池末尾）----------

    def intern(self, tag: int, payload):
        """登记一条待追加常量池条目（幂等：同 tag+payload 复用同一新索引）。"""
        for off, (t, p) in enumerate(self.pending):
            if t == tag and p == payload:
                return self.cp_base() + off
        self.pending.append((tag, payload))
        return self.cp_base() + len(self.pending) - 1

    def cp_base(self) -> int:
        """追加区首索引 = 现有槽位数 + 1（含 long/double 的空槽）。"""
        return len(self.cp)  # cp 是 1-based 列表，len 含占位 None，恰好等于下一可用索引

    def intern_utf8(self, s: str) -> int:
        return self.intern(UTF8, s.encode("utf-8"))

    def intern_nat(self, name_idx: int, desc_idx: int) -> int:
        return self.intern(NAMEANDTYPE, (name_idx, desc_idx))


# ---------- 解析 ----------

class _Reader:
    def __init__(self, data: bytes):
        self.d = data
        self.p = 0

    def take(self, n: int) -> bytes:
        if self.p + n > len(self.d):
            raise ClassFormatError("class 文件截断")
        b = self.d[self.p:self.p + n]
        self.p += n
        return b

    def u1(self):
        return self.take(1)[0]

    def u2(self):
        return struct.unpack(">H", self.take(2))[0]

    def u4(self):
        return struct.unpack(">I", self.take(4))[0]


def parse(data: bytes) -> ClassFile:
    r = _Reader(data)
    if r.u4() != 0xCAFEBABE:
        raise ClassFormatError("magic 不是 CAFEBABE")
    minor, major = r.u2(), r.u2()
    cp_count = r.u2()
    cp = [None] * cp_count
    i = 1
    while i < cp_count:
        tag = r.u1()
        if tag == UTF8:
            n = r.u2()
            cp[i] = (UTF8, r.take(n))
        elif tag in (INTEGER, FLOAT):
            cp[i] = (tag, r.take(4))
        elif tag in _WIDE_TAGS:
            cp[i] = (tag, r.take(8))
            i += 1                      # 空槽
        elif tag == CLASS or tag == STRING or tag == METHODTYPE or tag in (MODULE, PACKAGE):
            cp[i] = (tag, r.u2())
        elif tag in _REF_TAGS or tag == NAMEANDTYPE or tag in (DYNAMIC, INVOKEDYNAMIC):
            cp[i] = (tag, (r.u2(), r.u2()))
        elif tag == METHODHANDLE:
            cp[i] = (tag, (r.u1(), r.u2()))
        else:
            raise ClassFormatError(f"未知常量池 tag={tag} @ cp[{i}]")
        i += 1

    cf = ClassFile(minor=minor, major=major, cp=cp,
                   access=r.u2(), this_idx=r.u2(), super_idx=r.u2(),
                   interfaces=[r.u2() for _ in range(r.u2())],
                   fields=[], methods=[], attributes=[])

    def read_attrs():
        out = []
        for _ in range(r.u2()):
            name_idx, length = r.u2(), r.u4()
            out.append((name_idx, r.take(length)))
        return out

    for _ in range(r.u2()):
        cf.fields.append(Member(r.u2(), r.u2(), r.u2(), read_attrs()))
    for _ in range(r.u2()):
        cf.methods.append(Member(r.u2(), r.u2(), r.u2(), read_attrs()))
    cf.attributes = read_attrs()
    if r.p != len(data):
        raise ClassFormatError(f"尾部残留 {len(data) - r.p} 字节")
    return cf


# ---------- 回写 ----------

def serialize(cf: ClassFile) -> bytes:
    out = bytearray()
    out += struct.pack(">IHH", 0xCAFEBABE, cf.minor, cf.major)

    entries = [e for e in cf.cp[1:] if e is not None] + list(cf.pending)
    # 重新编号：long/double 仍占两槽
    cp_count = 1
    for tag, _ in entries:
        cp_count += 2 if tag in _WIDE_TAGS else 1
    out += struct.pack(">H", cp_count)

    def enc(tag, payload):
        if tag == UTF8:
            return bytes([tag]) + struct.pack(">H", len(payload)) + payload
        if tag in (INTEGER, FLOAT):
            return bytes([tag]) + payload
        if tag in _WIDE_TAGS:
            return bytes([tag]) + payload
        if tag in (CLASS, STRING, METHODTYPE, MODULE, PACKAGE):
            return bytes([tag]) + struct.pack(">H", payload)
        if tag in _REF_TAGS or tag == NAMEANDTYPE or tag in (DYNAMIC, INVOKEDYNAMIC):
            return bytes([tag]) + struct.pack(">HH", *payload)
        if tag == METHODHANDLE:
            return bytes([tag, payload[0]]) + struct.pack(">H", payload[1])
        raise ClassFormatError(f"无法编码 tag={tag}")

    for tag, payload in entries:
        out += enc(tag, payload)

    out += struct.pack(">HHH", cf.access, cf.this_idx, cf.super_idx)
    out += struct.pack(">H", len(cf.interfaces))
    for x in cf.interfaces:
        out += struct.pack(">H", x)

    def write_member(m: Member):
        out.extend(struct.pack(">HHH", m.access, m.name_idx, m.desc_idx))
        out.extend(struct.pack(">H", len(m.attributes)))
        for name_idx, raw in m.attributes:
            out.extend(struct.pack(">HI", name_idx, len(raw)))
            out.extend(raw)

    out += struct.pack(">H", len(cf.fields))
    for m in cf.fields:
        write_member(m)
    out += struct.pack(">H", len(cf.methods))
    for m in cf.methods:
        write_member(m)
    out += struct.pack(">H", len(cf.attributes))
    for name_idx, raw in cf.attributes:
        out += struct.pack(">HI", name_idx, len(raw))
        out += raw
    return bytes(out)
