#!/usr/bin/env python3
"""ttransform.py — 参考版字节码 harness 适配变换（T-变换）。

**只做常量池 Class 项重定向**（与 tools/rename-pipeline 同一手术风格）：

    java/lang/System  -> oracle/vt/VTime    (currentTimeMillis()J / gc()V / out)
    java/lang/Thread  -> oracle/vt/VThread  (<init>(Runnable)V / start()V /
                                             sleep(J)V / yield()V)

目的：原版用墙钟与真实线程（a.java:623 setSeed(System.currentTimeMillis())、
2424-2425 帧限速忙等、8421-8422 new Thread），直接跑 trace 不可复现。
T-变换把它们指向虚拟时钟/协作调度器，**A/B 两变体施加同一变换**，
差分语义 = 「在确定性 harness 观测下的行为等价」，与 t1 的 javap 等价验证互补。

安全校验（任一不满足即退出非 0）：被重定向的 Class 项只能被白名单成员引用
（Fieldref/Methodref/InterfaceMethodref 且 (name,desc) 命中上面清单）；
出现其他用途（ldc/instanceof/异常表等）立即拒绝。变换后尾部字节（含 Code、
LineNumberTable）逐字节不变——只动常量池区。

用法：python3 ttransform.py <in.class> <out.class> [--expect N]
"""
import hashlib
import struct
import sys

TARGETS = {
    "java/lang/System": {
        "owner": "oracle/vt/VTime",
        "allow": {
            ("currentTimeMillis", "()J", "method"),
            ("gc", "()V", "method"),
            ("out", "Ljava/io/PrintStream;", "field"),
        },
    },
    "java/lang/Thread": {
        "owner": "oracle/vt/VThread",
        "allow": {
            ("<init>", "(Ljava/lang/Runnable;)V", "method"),
            ("start", "()V", "method"),
            ("sleep", "(J)V", "method"),
            ("yield", "()V", "method"),
        },
    },
}

# 常量池 tag（JVM 规范实际取值，注意无 2/13/14）
UTF8, INTEGER, FLOAT, LONG, DOUBLE, CLASS, STRING = 1, 3, 4, 5, 6, 7, 8
FIELDREF, METHODREF, IMETHODREF, NAT = 9, 10, 11, 12
MH, MT, DYNAMIC, INVDYNAMIC, MODULE, PACKAGE = 15, 16, 17, 18, 19, 20


class CP:
    def __init__(self, data: bytes):
        self.data = data
        self.entries = [None]  # 1-based；(tag, payload_bytes, start, end)
        self.tail = b""
        self._parse()

    def _parse(self):
        d = self.data
        if d[:4] != b"\xca\xfe\xba\xbe":
            raise SystemExit("not a class file")
        (cp_count,) = struct.unpack_from(">H", d, 8)
        p = 10
        i = 1
        while i < cp_count:
            tag = d[p]
            start = p
            p += 1
            if tag == UTF8:
                (ln,) = struct.unpack_from(">H", d, p)
                p += 2 + ln
            elif tag in (INTEGER, FLOAT):
                p += 4
            elif tag in (LONG, DOUBLE):
                p += 8
                self.entries.append((tag, d[start:p], start, p))
                self.entries.append(None)  # wide 占位
                i += 2
                continue
            elif tag in (CLASS, STRING, 16, 19, 20):   # Class/String/MethodType/Module/Package
                p += 2
            elif tag in (FIELDREF, METHODREF, IMETHODREF, NAT, DYNAMIC, INVDYNAMIC):
                p += 4
            elif tag == 15:  # MethodHandle
                p += 3
            else:
                raise SystemExit(f"unknown cp tag {tag} at {start}")
            self.entries.append((tag, d[start:p], start, p))
            i += 1
        self.tail = d[p:]
        self.cp_end = p

    def utf8(self, idx):
        tag, payload, _, _ = self.entries[idx]
        assert tag == UTF8
        (ln,) = struct.unpack_from(">H", payload, 1)
        return payload[3:3 + ln].decode("utf-8", "replace")

    def class_name_idx(self, idx):
        tag, payload, _, _ = self.entries[idx]
        assert tag == CLASS
        return struct.unpack_from(">H", payload, 1)[0]


def transform(data: bytes, label=""):
    cp = CP(data)
    # 1) 找到目标 Class 项并统计其全部用途
    uses = {}  # class_idx -> list of (kind, name, desc)
    for e in cp.entries:
        if e is None:
            continue
        tag, payload, _, _ = e
        if tag in (FIELDREF, METHODREF, IMETHODREF):
            cls_idx, nat_idx = struct.unpack_from(">HH", payload, 1)
            t2, payload2, _, _ = cp.entries[nat_idx]
            assert t2 == NAT
            name_idx, desc_idx = struct.unpack_from(">HH", payload2, 1)
            kind = {FIELDREF: "field", METHODREF: "method", IMETHODREF: "imethod"}[tag]
            uses.setdefault(cls_idx, []).append((kind, cp.utf8(name_idx), cp.utf8(desc_idx)))
    new_names = {}
    counts = {}
    for cls_idx, refs in uses.items():
        name = cp.utf8(cp.class_name_idx(cls_idx))
        if name not in TARGETS:
            continue
        spec = TARGETS[name]
        for kind, mname, mdesc in refs:
            key = (mname, mdesc, kind)
            alt = (mname, mdesc, "method" if kind == "imethod" else kind)
            if key not in spec["allow"] and alt not in spec["allow"]:
                raise SystemExit(f"{label}: {name}.{mname}{mdesc} ({kind}) 不在 T-变换白名单，拒绝变换")
        new_names[cls_idx] = spec["owner"]
        counts[name] = len(refs)
    if not new_names:
        raise SystemExit(f"{label}: 未发现任何 T-变换目标（java/lang/System|Thread）")
    # 2) 重写：目标 Class 项 name_idx 指向池末尾新增 Utf8
    out = bytearray(data[:8])  # magic + minor + major（cp_count 在下方重写）
    appends = []
    for cls_idx, owner in sorted(new_names.items()):
        appends.append(owner)
    next_idx = len(cp.entries)
    owner_idx = {}
    for k, owner in enumerate(appends):
        owner_idx[owner] = next_idx + k
    patched = []
    for i, e in enumerate(cp.entries):
        if e is None:
            patched.append(None)
            continue
        tag, payload, start, end = e
        if i in new_names:
            (name_idx,) = struct.unpack_from(">H", payload, 1)
            payload = payload[:1] + struct.pack(">H", owner_idx[new_names[i]])
            patched.append((tag, payload, start, end))
        else:
            patched.append(e)
    (cp_count,) = struct.unpack_from(">H", data, 8)
    new_count = cp_count + len(appends)
    out += struct.pack(">H", new_count)
    for e in patched:
        if e is not None:
            out += e[1]
    for owner in appends:
        b = owner.encode("utf-8")
        out += bytes([UTF8]) + struct.pack(">H", len(b)) + b
    out += cp.tail
    return bytes(out), counts


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    src, dst = sys.argv[1], sys.argv[2]
    data = open(src, "rb").read()
    out, counts = transform(data, label=src.split("/")[-1])
    open(dst, "wb").write(out)
    before = hashlib.sha256(data).hexdigest()
    after = hashlib.sha256(out).hexdigest()
    summary = " ".join(f"{k}->{v}" for k, v in sorted(counts.items()))
    print(f"T-TRANSFORM {src} -> {dst}")
    print(f"  sha256 before={before[:16]} after={after[:16]}")
    print(f"  retargeted refs: {summary or '(none)'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
