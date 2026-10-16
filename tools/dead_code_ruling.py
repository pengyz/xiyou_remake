#!/usr/bin/env python3
"""dead_code_ruling.py — 覆盖矩阵白项 b 类（不触达）的字节码级机械裁决（2026-10-15，判据①收口）。

对 docs/coverage-matrix.md §4 的 b 类 10 方法逐项**机械复核**「正常执行永不进入」：
裁决书正文见 docs/spec/dead-code-rulings.md；cover_matrix.py 渲染时调用本模块，复核失败
即在账面打 ⚠ 且不得记 ☠️。

输入（全部可再生、确定性）：
  javap -p -c -s analysis/rename-pipeline/renamed/{a,CMidlet}.class
  （改名类：方法/字段名即矩阵名；原版 JAR 仅含这两个 class——assets-manifest 门禁 classes=2）

方法（保守的上近似，从「全部代码存活、字段值域 ⊤」出发逐轮收缩至不动点）：
  1. 解析 javap：方法 → 指令（偏移/操作码/操作数/注释）、switch 表、异常表 → CFG
     （顺序/条件/goto/switch 边 + 异常边：try 区间内每条指令 → handler）。
  2. 字段值域：只看**存活代码中可达**的 putfield F 写点；值产生指令（putfield 的直接前驱，
     且 putfield 本身不是跳转目标）∈ {iconst/bipush/sipush/ldc int 常量, getfield G（拷贝 G
     的值域）}，任何其他形态 ⇒ 值域(F) = ⊤（不据此剪枝）。值域恒含 JVM 缺省值 0。
  3. 守卫剪边：`getfield F; tableswitch/lookupswitch` 中可能值都到不了的目标边、
     `getfield F; ifeq/ifne/iflt/…` 与 `getfield F; 常量; if_icmpXX` 中永不发生的分支边
     为死边（守卫指令是跳转目标 ⇒ 栈值可能另有来源 ⇒ 不剪）。外加 MVS 引理（字符串数据域，
     见 mvs_lemma）给出的死边。
  4. 方法内可达性：CFG 去死边后自偏移 0 BFS。方法存活 = 入口点（a 的非 private 方法 =
     MIDP 平台回调/构造器、CMidlet 全部方法、static{}）∪ 存活方法中**可达**调用点的被调者。
  5. 2→4 迭代至不动点。每一步都是真实执行的上近似（死边只在守卫条件对上近似值域都不成立时
     才剪），故终态判「死」的方法在任何执行中都不会被调用。

前提（闭世界）：CLDC 1.0 无反射；私有方法只能被类 a 自身调用；a 的字段只经这两个 class 的
putfield 写入（CMidlet 对 a 的字段无 putfield——本工具同样扫描并校验）。

用法：
  python3 tools/dead_code_ruling.py            # 裁决报告（markdown，逐裁决 证据链）
  python3 tools/dead_code_ruling.py --json     # 机器可读
  python3 tools/dead_code_ruling.py --check    # 任一裁决对象未被证死 ⇒ 退出码 1
"""
from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from collections import defaultdict, deque
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RENAMED = ROOT / "analysis/rename-pipeline/renamed"
CLASSES = ("a", "CMidlet")

# 裁决对象（docs/spec/dead-code-rulings.md 的 DC-n；cover_matrix TRIAGE b 类逐条引用）
RULINGS: dict[str, dict] = {
    "DC-1": {"title": "道具轮盘左右箭头（walkPhase 3 臂）", "methods": ["drawLeftTriangleArrow"]},  # 裁决时名 m_093（dc470b2 N-0443 改名）
    "DC-2": {"title": "DSL 第 16 码 MVS（脚本文本源封闭）", "methods": ["m_107"]},
    "DC-3": {"title": "联网簇（gameMode 99 臂 + f_bool_31 自门控）",
             "methods": ["m_148", "m_149", "m_150", "m_151", "m_152", "m_153", "m_154"]},
    "DC-4": {"title": "火花层（paint gameMode 18 臂）", "methods": ["addSpark"]},
}

SWITCH = {"tableswitch", "lookupswitch"}
IF1 = {"ifeq": lambda v: v == 0, "ifne": lambda v: v != 0, "iflt": lambda v: v < 0,
       "ifge": lambda v: v >= 0, "ifgt": lambda v: v > 0, "ifle": lambda v: v <= 0}
ICMP = {"if_icmpeq": lambda a, b: a == b, "if_icmpne": lambda a, b: a != b,
        "if_icmplt": lambda a, b: a < b, "if_icmpge": lambda a, b: a >= b,
        "if_icmpgt": lambda a, b: a > b, "if_icmple": lambda a, b: a <= b}
OTHER_COND = {"if_acmpeq", "if_acmpne", "ifnull", "ifnonnull"}
GOTO = {"goto", "goto_w"}
TERM = {"return", "ireturn", "lreturn", "freturn", "dreturn", "areturn", "athrow"}
INVOKE = {"invokevirtual", "invokespecial", "invokestatic", "invokeinterface"}

INS_RE = re.compile(r"^\s+(\d+): (\w+)\s*(.*?)\s*$")
CASE_RE = re.compile(r"^\s+(-?\d+|default): (\d+)\s*$")
EXC_RE = re.compile(r"^\s+(\d+)\s+(\d+)\s+(\d+)\s+(\S.*?)\s*$")
FIELD_REF = re.compile(r"Field (?:([\w/$]+)\.)?([\w$]+):(\S+)")
METHOD_REF = re.compile(r"(?:Interface)?Method (?:([\w/$]+)\.)?\"?([\w$<>]+)\"?:(\(\S*)")


@dataclass
class Ins:
    off: int
    op: str
    arg: str
    cmt: str
    cases: dict = field(default_factory=dict)   # switch：键（int | "default"）→ 目标偏移


@dataclass
class Method:
    cls: str
    name: str
    header: str
    private: bool
    desc: str = ""
    ins: list = field(default_factory=list)
    exc: list = field(default_factory=list)     # (from, to, handler, type)
    succ: dict = field(default_factory=dict)    # off → [(dst, label)]
    jt: set = field(default_factory=set)        # 非顺序边的目标偏移（跳转目标）
    idx: dict = field(default_factory=dict)     # off → 指令下标

    @property
    def key(self) -> tuple:
        return (self.cls, self.name, self.desc)

    @property
    def label(self) -> str:
        return self.name if self.cls == "a" else f"{self.cls}.{self.name}"


def sha256(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def javap(cls: str) -> str:
    r = subprocess.run(["javap", "-p", "-c", "-s", str(RENAMED / f"{cls}.class")],
                       capture_output=True, text=True)
    if r.returncode != 0:
        print(f"javap {cls} 失败：{r.stderr[-400:]}（先跑 python3 tools/rename-pipeline/run.py）",
              file=sys.stderr)
        sys.exit(2)
    return r.stdout


def parse_class(cls: str, text: str) -> list[Method]:
    out: list[Method] = []
    cur: Method | None = None
    lines = text.split("\n")
    i = 0
    in_exc = False
    while i < len(lines):
        l = lines[i]
        if l.startswith("  ") and not l.startswith("   ") and l.rstrip().endswith(";"):
            s = l.strip()
            in_exc = False
            cur = None
            if s == "static {};":
                cur = Method(cls, "<clinit>", s, private=False)
            elif "(" in s:
                head = s.split("(")[0].split()
                name = "<init>" if head[-1] == cls else head[-1]
                cur = Method(cls, name, s, private="private" in head)
            if cur is not None:
                out.append(cur)
            i += 1
            continue
        if cur is None:
            i += 1
            continue
        s = l.strip()
        if s.startswith("descriptor:"):
            cur.desc = s.split(":", 1)[1].strip()
        elif s == "Exception table:":
            in_exc = True
            i += 2  # 跳过表头 "from to target type"
            continue
        elif in_exc:
            m = EXC_RE.match(l)
            if m:
                cur.exc.append((int(m.group(1)), int(m.group(2)), int(m.group(3)), m.group(4)))
            else:
                in_exc = False
        else:
            m = INS_RE.match(l)
            if m:
                rest = m.group(3)
                arg, _, cmt = rest.partition("//")
                ins = Ins(int(m.group(1)), m.group(2), arg.strip(), cmt.strip())
                cur.ins.append(ins)
                if ins.op in SWITCH:
                    i += 1
                    while i < len(lines) and lines[i].strip() != "}":
                        c = CASE_RE.match(lines[i])
                        if c:
                            k = c.group(1)
                            ins.cases["default" if k == "default" else int(k)] = int(c.group(2))
                        i += 1
        i += 1
    for m in out:
        build_cfg(m)
    return out


def build_cfg(m: Method) -> None:
    offs = [x.off for x in m.ins]
    m.idx = {o: k for k, o in enumerate(offs)}
    nxt = {offs[k]: offs[k + 1] for k in range(len(offs) - 1)}
    succ = defaultdict(list)
    for x in m.ins:
        if x.op in SWITCH:
            by_t = defaultdict(set)
            for k, t in x.cases.items():
                by_t[t].add(k)
            for t, ks in sorted(by_t.items()):
                succ[x.off].append((t, ("sw", frozenset(ks, ))))
        elif x.op in GOTO:
            succ[x.off].append((int(x.arg), ("goto",)))
        elif x.op in IF1 or x.op in ICMP or x.op in OTHER_COND:
            succ[x.off].append((int(x.arg), ("jump",)))
            succ[x.off].append((nxt[x.off], ("fall",)))
        elif x.op in TERM:
            pass
        elif x.off in nxt:
            succ[x.off].append((nxt[x.off], ("next",)))
    for f, t, h, _ in m.exc:
        for o in offs:
            if f <= o < t:
                succ[o].append((h, ("exc",)))
    m.succ = succ
    m.jt = {d for es in succ.values() for d, lab in es if lab[0] not in ("next", "fall")}


# ---------------------------------------------------------------- 引用解析

def field_of(x: Ins, cls: str) -> str | None:
    m = FIELD_REF.search(x.cmt)
    if not m:
        return None
    return f"{m.group(1) or cls}.{m.group(2)}"


def callee_of(x: Ins, cls: str) -> tuple | None:
    m = METHOD_REF.search(x.cmt)
    if not m:
        return None
    return (m.group(1) or cls, m.group(2), m.group(3))


def const_of(x: Ins) -> int | None:
    if x.op == "iconst_m1":
        return -1
    if x.op.startswith("iconst_"):
        return int(x.op[7:])
    if x.op in ("bipush", "sipush"):
        return int(x.arg)
    if x.op in ("ldc", "ldc_w"):
        m = re.match(r"int (-?\d+)$", x.cmt)
        return int(m.group(1)) if m else None
    return None


# ---------------------------------------------------------------- 字段值域

TOP = None  # 值域未知（⊤）


def field_writes(methods: dict, reach: dict, live: set, rt: dict | None = None) -> dict:
    """字段 → [(方法标签, 偏移, 产生指令描述, 常量|('copy',G)|('top',描述), 是否计入)]。
    rt：save_roundtrip_lemma 成立时把其 site 写点改判为 ('copy', F) 自拷贝。"""
    w = defaultdict(list)
    for k, m in methods.items():
        for j, x in enumerate(m.ins):
            if x.op != "putfield":
                continue
            f = field_of(x, m.cls)
            counted = k in live and x.off in reach[k]
            prev = m.ins[j - 1] if j > 0 else None
            if prev is None or x.off in m.jt:
                src = ("top", "putfield 是跳转目标/无前驱")
                desc = "?"
            else:
                c = const_of(prev)
                desc = f"{prev.op} {prev.arg}".strip()
                if c is not None:
                    src = c
                elif prev.op in ("getfield", "getstatic") and field_of(prev, m.cls):
                    src = ("copy", field_of(prev, m.cls))
                    desc = f"{prev.op} {field_of(prev, m.cls).split('.')[-1]}"
                else:
                    src = ("top", desc)
            if rt and rt.get("ok") and rt.get("site") == (m.label, x.off):
                src = ("copy", rt["field"])
                desc = "readByte←writeByte(getfield difficultyIndex)〔存档往返引理 L3-L4 自拷贝〕"
            w[f].append((m.label, x.off, desc, src, counted))
    return w


def value_sets(writes: dict) -> dict:
    """字段 → 值域（frozenset，含缺省 0）或 TOP；拷贝写点按被拷字段值域闭包（最小不动点）。"""
    consts, copies, top = defaultdict(set), defaultdict(set), set()
    for f, ws in writes.items():
        for _, _, _, src, counted in ws:
            if not counted:
                continue
            if isinstance(src, int):
                consts[f].add(src)
            elif src[0] == "copy":
                copies[f].add(src[1])
            else:
                top.add(f)
    vals: dict = {f: (TOP if f in top else set(consts[f]) | {0}) for f in writes}
    changed = True
    while changed:
        changed = False
        for f, gs in copies.items():
            if vals.get(f) is TOP:
                continue
            for g in gs:
                vg = vals.get(g, {0})
                if vg is TOP:
                    vals[f] = TOP
                    changed = True
                    break
                if not vg <= vals[f]:
                    vals[f] |= vg
                    changed = True
    return {f: (v if v is TOP else frozenset(v)) for f, v in vals.items()}


def vals_of(vals: dict, f: str):
    """未出现在写点表的字段 = 从未被写 = 恒为缺省 0。"""
    return vals.get(f, frozenset({0}))


# ---------------------------------------------------------------- 守卫与剪边

def guard(m: Method, j: int) -> tuple | None:
    """第 j 条指令（switch/条件跳转）的字段守卫：('fld', F) | ('cmp', F, K, 顺序)。"""
    x = m.ins[j]
    if j == 0 or x.off in m.jt:
        return None
    p1 = m.ins[j - 1]
    if x.op in SWITCH or x.op in IF1:
        if p1.op == "getfield" and field_of(p1, m.cls):
            return ("fld", field_of(p1, m.cls))
        return None
    if x.op in ICMP and j >= 2:
        p2 = m.ins[j - 2]
        if const_of(p1) is not None and p2.op == "getfield" and p1.off not in m.jt:
            return ("cmp", field_of(p2, m.cls), const_of(p1), "vk")
        if j >= 3 and p1.op == "getfield" and p2.op.startswith("aload") and const_of(m.ins[j - 3]) is not None \
                and p1.off not in m.jt and p2.off not in m.jt:
            return ("cmp", field_of(p1, m.cls), const_of(m.ins[j - 3]), "kv")
    return None


def dead_edges(m: Method, vals: dict, extra: dict) -> dict:
    """死边 → 原因（守卫字段、可能值、该边对应的键/分支）。"""
    dead = {}
    for j, x in enumerate(m.ins):
        g = guard(m, j)
        if g is None:
            continue
        V = vals_of(vals, g[1])
        if V is TOP:
            continue
        if x.op in SWITCH:
            live_t = {x.cases.get(v, x.cases.get("default")) for v in V}
            for t, lab in m.succ[x.off]:
                if lab[0] == "sw" and t not in live_t:
                    dead[(x.off, t, lab)] = {"kind": x.op, "field": g[1], "keys": sorted(map(str, lab[1])),
                                             "values": sorted(V)}
        else:
            if x.op in IF1:
                outs = {IF1[x.op](v) for v in V}
            else:
                K = g[2]
                outs = {ICMP[x.op](v, K) if g[3] == "vk" else ICMP[x.op](K, v) for v in V}
            for t, lab in m.succ[x.off]:
                if (lab[0] == "jump" and True not in outs) or (lab[0] == "fall" and False not in outs):
                    dead[(x.off, t, lab)] = {"kind": x.op, "field": g[1], "branch": lab[0],
                                             "cmp": g[2] if g[0] == "cmp" else None, "values": sorted(V)}
    dead.update(extra)
    return dead


def reachable(m: Method, dead) -> set:
    seen, q = {0}, deque([0])
    while q:
        o = q.popleft()
        for t, lab in m.succ.get(o, []):
            if (o, t, lab) in dead or t in seen:
                continue
            seen.add(t)
            q.append(t)
    return seen


# ---------------------------------------------------------------- MVS 引理（字符串数据域）

def mvs_lemma(methods: dict) -> dict:
    """executeScriptInstruction 中 `aload_3; ldc "MVS"; String.equals; ifeq L` 的相等分支（fall）
    在任何执行中都不发生——逐步机械校验：
      L1 局部 3 的全部 astore 为 aconst_null 或 `aload_1; iload_2; iload_2; iconst_3; iadd;
         String.substring(II)`（3 字符窗口取自参数 1）；方法内无 astore_1（参数不被改写）。
      L2 executeScriptInstruction 的全部调用点：首参（String）由 `getfield dialogScriptLine` 提供。
      L3 dialogScriptLine 的全部 putfield：值为 `getfield levelScriptLines; …; aaload`。
      L4 levelScriptLines 唯一 putfield 在 <init>，值为 `anewarray String` + N×(dup, 下标, ldc 串,
         aastore)；全部 getfield levelScriptLines 之后紧随 arraylength 或（下标）aaload（无元素写）。
      L5 这 N 条脚本串均不含子串 "MVS"。
    ⇒ 比较串 ∈ 脚本串的 3 字符子串 ∪ {null} ∌ "MVS" ⇒ 相等分支死。"""
    res = {"ok": False, "steps": [], "edge": None}
    esi = [m for m in methods.values() if m.cls == "a" and m.name == "executeScriptInstruction"]
    if len(esi) != 1:
        res["steps"].append(f"executeScriptInstruction 方法数 = {len(esi)}（应为 1）")
        return res
    m = esi[0]
    site = None
    for j in range(len(m.ins) - 3):
        a, b, c, d = m.ins[j:j + 4]
        if b.op in ("ldc", "ldc_w") and b.cmt == "String MVS" and "String.equals" in c.cmt and d.op in ("ifeq", "ifne"):
            site = (j, a, d)
    if site is None:
        res["steps"].append("未找到 `ldc \"MVS\"; String.equals; ifeq/ifne` 比较点")
        return res
    j, a, d = site
    ok = True
    if a.op != "aload_3" or any(m.ins[k].off in m.jt for k in range(j + 1, j + 4)):
        res["steps"].append(f"L0 ✗ 比较前驱 {a.op}（应为 aload_3 且比较序列内无跳转目标）")
        ok = False
    stores = [(k, x) for k, x in enumerate(m.ins) if x.op == "astore_3" or (x.op == "astore" and x.arg == "3")]
    bad, recv = [], []
    for k, x in stores:
        p = [y.op for y in m.ins[max(0, k - 6):k]]
        if p[-1:] == ["aconst_null"]:
            continue
        if len(p) == 6 and p == ["aload_1", "iload_2", "iload_2", "iconst_3", "iadd", "invokevirtual"] \
                and "String.substring:(II)" in m.ins[k - 1].cmt \
                and not any(m.ins[q].off in m.jt for q in range(k - 5, k + 1)):
            recv.append(m.ins[k - 6].off)
            continue
        bad.append(f"@{x.off}")
    # 混淆器复用局部槽：参数 1 在方法后段被 astore_1 改写（int/引用复用）——要求任何 astore_1
    # 都到达不了 substring 的接收者装载点（CFG 前向可达，含异常边），即该处局部 1 恒为参数
    a1 = [x.off for x in m.ins if x.op == "astore_1" or (x.op == "astore" and x.arg == "1")]
    hit = set()
    for o in a1:
        seen, q = set(), deque(t for t, _ in m.succ.get(o, []))
        while q:
            t = q.popleft()
            if t in seen:
                continue
            seen.add(t)
            q.extend(u for u, _ in m.succ.get(t, []))
        hit |= seen & set(recv)
    l1 = not bad and recv and not hit
    res["steps"].append(f"L1 {'✓' if l1 else '✗'} 局部 3 的引用写入 {len(stores)} 处"
                        f"（@{', @'.join(str(x.off) for _, x in stores)}）= aconst_null / `aload_1; iload_2; iload_2; "
                        f"iconst_3; iadd; String.substring(II)`（接收者 aload_1 @{', @'.join(map(str, recv))}）；"
                        f"astore_1 {len(a1)} 处（局部槽复用）均不能前向到达该接收者 ⇒ 局部 1 = 参数（脚本行）"
                        + (f"；异常 {bad}" if bad else "") + (f"；可达接收者的 astore_1 → {sorted(hit)}" if hit else ""))
    ok &= bool(l1)
    calls = []
    for k2, mm in methods.items():
        for jj, x in enumerate(mm.ins):
            if x.op in INVOKE and callee_of(x, mm.cls) == ("a", "executeScriptInstruction", m.desc):
                # 调用形态须恰为 `aload; [aload; getfield dialogScriptLine]; [aload; getfield int]; invoke`
                w = mm.ins[jj - 5:jj] if jj >= 5 else []
                src = (len(w) == 5 and w[2].op == "getfield" and field_of(w[2], mm.cls) == "a.dialogScriptLine"
                       and w[1].op.startswith("aload") and w[3].op.startswith("aload") and w[4].op == "getfield"
                       and w[4].cmt.endswith(":I") and not any(y.off in mm.jt for y in w[2:] + [x]))
                calls.append((mm.label, x.off, src))
    bad = [c for c in calls if not c[2]]
    res["steps"].append(f"L2 {'✓' if calls and not bad else '✗'} executeScriptInstruction 调用点 {len(calls)} 处"
                        f"（{', '.join(f'{c[0]}@{c[1]}' for c in calls)}）首参均为 getfield dialogScriptLine"
                        + (f"；异常 {bad}" if bad else ""))
    ok &= bool(calls) and not bad
    dsl = []
    for mm in methods.values():
        for jj, x in enumerate(mm.ins):
            if x.op == "putfield" and field_of(x, mm.cls) == "a.dialogScriptLine":
                w = mm.ins[max(0, jj - 4):jj]
                good = (w and w[-1].op == "aaload" and x.off not in mm.jt
                        and any(y.op == "getfield" and field_of(y, mm.cls) == "a.levelScriptLines" for y in w))
                dsl.append((mm.label, x.off, good))
    res["steps"].append(f"L3 {'✓' if dsl and all(g for *_, g in dsl) else '✗'} dialogScriptLine 写点 {len(dsl)} 处"
                        f"（{', '.join(f'{a}@{b}' for a, b, _ in dsl)}）= levelScriptLines[…]（aaload）")
    ok &= bool(dsl) and all(g for *_, g in dsl)
    puts, gets, scripts = [], [], []
    for mm in methods.values():
        for jj, x in enumerate(mm.ins):
            f = field_of(x, mm.cls) if x.op in ("putfield", "getfield") else None
            if f != "a.levelScriptLines":
                continue
            if x.op == "getfield":
                after = [y.op for y in mm.ins[jj + 1:jj + 5]]
                read_only = after[:1] == ["arraylength"] or (after[2:3] == ["aaload"] and "aastore" not in after)
                gets.append((mm.label, x.off, read_only))
                continue
            # 回溯数组构造：anewarray String … putfield（javap 把空串注释成 `// String`）
            k = jj - 1
            strs = []
            while k >= 0:
                y = mm.ins[k]
                if y.op == "anewarray":
                    break
                if y.op in ("ldc", "ldc_w") and (y.cmt == "String" or y.cmt.startswith("String ")):
                    strs.append(y.cmt[len("String "):] if y.cmt != "String" else "")
                elif y.op in ("ldc", "ldc_w"):
                    strs.append(None)  # 非字符串常量 ⇒ 形态校验失败
                k -= 1
            shape_ok = (k >= 0 and "class java/lang/String" in mm.ins[k].cmt and mm.label == "<init>"
                        and None not in strs and x.off not in mm.jt)
            n = const_of(mm.ins[k - 1]) if k >= 1 else None
            ast = [q for q in range(k, jj) if mm.ins[q].op == "aastore"]
            # 每个 aastore 的值都是紧邻的 ldc 字符串常量（数组元素只可能是这些常量）
            vals_const = all(mm.ins[q - 1].op in ("ldc", "ldc_w") and mm.ins[q - 1].cmt.startswith("String")
                             for q in ast)
            puts.append((mm.label, x.off, shape_ok and vals_const and n == len(strs) == len(ast)))
            scripts = list(reversed(strs))
    res["steps"].append(f"L4 {'✓' if len(puts) == 1 and all(p[2] for p in puts) and all(g[2] for g in gets) else '✗'} "
                        f"levelScriptLines putfield {len(puts)} 处（{', '.join(f'{a}@{b}' for a, b, _ in puts)}：anewarray "
                        f"String[{len(scripts)}] + {len(scripts)}×ldc 串）；getfield {len(gets)} 处"
                        f"（{', '.join(f'{a}@{b}' for a, b, _ in gets)}）均为 arraylength/aaload 读")
    ok &= len(puts) == 1 and all(p[2] for p in puts) and all(g[2] for g in gets)
    hit = [i for i, s in enumerate(scripts) if s is None or "MVS" in s]
    res["steps"].append(f"L5 {'✓' if scripts and not hit else '✗'} {len(scripts)} 条脚本串含 \"MVS\" 子串 "
                        f"{len(hit)} 条" + (f"（脚本 {hit}）" if hit else ""))
    ok &= bool(scripts) and not hit
    res["n_scripts"] = len(scripts)
    if ok:
        eq_branch = "fall" if d.op == "ifeq" else "jump"
        for t, lab in m.succ[d.off]:
            if lab[0] == eq_branch:
                res["edge"] = (m.key, (d.off, t, lab), {"kind": "MVS 引理", "field": "String.equals(\"MVS\")",
                                                        "branch": eq_branch, "values": ["false"]})
    res["ok"] = ok and res["edge"] is not None
    res["site"] = f"executeScriptInstruction@{d.off}（{d.op} {d.arg}）"
    return res


# ---------------------------------------------------------------- 存档往返归纳引理（DC-5 数据域前提）

ROUNDTRIP_FIELD = "a.difficultyIndex"


def save_roundtrip_lemma(methods: dict) -> dict:
    """difficultyIndex 的存档往返归纳引理（MVS 引理同型 L 系机械校验）：

      L1 putfield F 全写点清点：恒定形态×2（均 0）+ 非恒定恰 1 处，产生链 =
         `getfield f_DataInputStream_00; invokevirtual DataInputStream.readByte()B`。
      L2 readByte 接收流的血统：f_DataInputStream_00 的非空 putfield 全部 =
         `new ByteArrayInputStream(getfield f_byte_arr_27); new DataInputStream(...)`；
         f_byte_arr_27 的非空 putfield 全部 = `RecordStore.getRecord(...)`。
      L3 槽 1 配对 + 自拷贝：loadFloorSave 中**最早**的流读恰落进 putfield F；对偶
         saveFloorState（同 "MOT_L" 库）中**最早**的流写 = `writeByte(getfield F)`。
      L4 记录库写口封闭：store 名 "MOT_L" 的唯一写者 = saveFloorState（其余写口
         MOT_IF / SKY_WAR 与 loadFloorSave 的 getRecord 不同库）。
      L5 归纳：F 全写点 = {0, 0, 自拷贝} ⇒ 读回值 ∈ F 历史值集；初值 0（JVM 缺省 +
         <init> 写 0）⇒ F ≡ 0（跨会话持久由同一归纳覆盖：存档里存过的任何值都是
         某次存档时刻的字段值）。

    成立 ⇒ 非恒定写点按 ('copy', F) 计入 value_sets，得 {0}。"""
    res = {"ok": False, "steps": [], "site": None, "field": ROUNDTRIP_FIELD}
    F = ROUNDTRIP_FIELD

    # ---- L1 putfield 写点清点
    sites = []
    for k, m in methods.items():
        for j, x in enumerate(m.ins):
            if x.op == "putfield" and field_of(x, m.cls) == F:
                sites.append((m, j, x, m.ins[j - 1] if j else None))
    consts = [(m, x, const_of(prev)) for m, j, x, prev in sites
              if prev is not None and const_of(prev) is not None]
    nonconst = [(m, j, x, prev) for m, j, x, prev in sites
                if prev is None or const_of(prev) is None]
    ok1 = (len(sites) == 3 and len(consts) == 2 and all(c == 0 for _, _, c in consts)
           and len(nonconst) == 1)
    site_desc = []
    if ok1:
        m, j, x, prev = nonconst[0]
        p2 = m.ins[j - 2] if j >= 2 else None
        ok1 = (prev is not None and prev.op == "invokevirtual"
               and "DataInputStream.readByte" in prev.cmt
               and p2 is not None and p2.op == "getfield"
               and field_of(p2, m.cls) == "a.f_DataInputStream_00"
               and x.off not in m.jt and prev.off not in m.jt and p2.off not in m.jt)
        res["site"] = (m.label, x.off)
        site_desc = [f"{m.label}@{x.off}"]
    res["steps"].append(
        f"L1 {'✓' if ok1 else '✗'} putfield {F} 共 {len(sites)} 处 = 恒定 {len(consts)} 处"
        f"（{', '.join(f'{m.label}@{x.off}={c}' for m, x, c in consts) or '无'}）+ 非恒定 {len(nonconst)} 处"
        f"（{', '.join(site_desc) or '异常'}：`getfield f_DataInputStream_00; invokevirtual readByte()B` 且链内无跳转目标）")

    # ---- L2 流血统
    ok2 = True
    d2 = []
    for m in methods.values():
        for j, x in enumerate(m.ins):
            if x.op != "putfield":
                continue
            f = field_of(x, m.cls)
            if f == "a.f_DataInputStream_00":
                w = m.ins[max(0, j - 7):j]
                if w and w[-1].op == "aconst_null":
                    continue
                good = (any(y.op == "new" for y in w) and any(y.op == "dup" for y in w)
                        and any(y.op == "getfield" and field_of(y, m.cls) == "a.f_byte_arr_27" for y in w)
                        and any(y.op == "invokespecial" and "DataInputStream.\"" in y.cmt for y in w))
                ok2 &= good
                d2.append(f"{m.label}@{x.off}{'✓' if good else '✗'}")
            elif f == "a.f_byte_arr_27":
                w = m.ins[max(0, j - 4):j]
                if w and w[-1].op == "aconst_null":
                    continue
                good = any(y.op == "invokevirtual" and "RecordStore.getRecord" in y.cmt for y in w)
                ok2 &= good
                d2.append(f"{m.label}@{x.off}{'✓' if good else '✗'}")
    res["steps"].append(
        f"L2 {'✓' if ok2 and d2 else '✗'} 流血统：f_DataInputStream_00 / f_byte_arr_27 非空 putfield "
        f"{len(d2)} 处均为 `new ByteArrayInputStream(getfield f_byte_arr_27)` / `RecordStore.getRecord` 形态"
        f"（{', '.join(d2)}）")

    # ---- L3 槽 1 配对 + 自拷贝；L4 写口封闭
    ok3 = ok4 = False
    d3 = d4 = ""
    if ok1 and res["site"]:
        site_lbl, site_off = res["site"]
        load_m = next(m for m in methods.values() if m.label == site_lbl)
        stream_read = re.compile(r"DataInputStream\.read(Byte|Short|Int|Boolean|)\:")
        read_idx = [j for j, y in enumerate(load_m.ins)
                    if y.op == "invokevirtual" and stream_read.search(y.cmt)]
        putf_idx = [y.off for y in load_m.ins].index(site_off)
        first_read = min(read_idx, default=-1)
        ok3 = (first_read >= 0 and putf_idx >= 1
               and load_m.ins[first_read].off == load_m.ins[putf_idx - 1].off)
        d3 = (f"loadFloorSave 首流读@{load_m.ins[first_read].off if first_read >= 0 else '?'}"
              f" → putfield {F.split('.')[-1]}@{site_off}")
        # 对偶 save 方法：store 名 MOT_L 且写记录
        def motl_writers() -> list:
            out = []
            for m2 in methods.values():
                names2 = {y.cmt.split("String ", 1)[1] for y in m2.ins
                          if y.op in ("ldc", "ldc_w") and y.cmt.startswith("String ")}
                if any(n.startswith("MOT_L") for n in names2) and any(
                        y.op == "invokevirtual" and ("RecordStore.addRecord" in y.cmt
                                                     or "RecordStore.setRecord" in y.cmt)
                        for y in m2.ins):
                    out.append(m2)
            return out

        wms = motl_writers()
        ok4 = len(wms) == 1
        if wms:
            save_m = wms[0]
            write_idx = [j for j, y in enumerate(save_m.ins)
                         if y.op == "invokevirtual" and "DataOutputStream.write" in y.cmt]
            fw = min(write_idx, default=-1)
            if fw >= 0:
                wwin = save_m.ins[max(0, fw - 4):fw]
                self_copy = (any(y.op == "getfield" and field_of(y, save_m.cls) == F for y in wwin)
                             and "writeByte" in save_m.ins[fw].cmt)
                ok3 &= self_copy
                d3 += f"；{save_m.label} 首流写@{save_m.ins[fw].off} = writeByte(getfield {F.split('.')[-1]})"
                d3 += " 自拷贝" if self_copy else " ✗非自拷贝"
            d4 = f"MOT_L 库写者 = {[m2.label for m2 in wms]}（唯一）；其余写口 MOT_IF / SKY_WAR 不同库"
    res["steps"].append(f"L3 {'✓' if ok3 else '✗'} 槽 1 配对 + 自拷贝：{d3}")
    res["steps"].append(f"L4 {'✓' if ok4 else '✗'} 记录库写口封闭：{d4}")

    ok5 = ok1 and ok2 and ok3 and ok4
    res["steps"].append(
        f"L5 {'✓' if ok5 else '✗'} 归纳：{F.split('.')[-1]} 全写点 = {{0, 0, 自拷贝}} ⇒ 读回值 ∈ "
        f"字段历史值集；初值 0（<init> 写 0 + JVM 缺省）⇒ ≡ 0（跨会话持久同归纳：存档值 = "
        f"某次存档时刻的字段值）⇒ 非恒定写点按 ('copy', {F}) 计")
    res["ok"] = ok5
    return res


def script_corpus(methods: dict) -> list[str]:
    """levelScriptLines 语料（MVS 引理 L4 同型回溯）：anewarray String + N×ldc 串。"""
    out: list[str] = []
    for m in methods.values():
        for jj, x in enumerate(m.ins):
            if x.op != "putfield" or field_of(x, m.cls) != "a.levelScriptLines":
                continue
            k = jj - 1
            strs = []
            while k >= 0:
                y = m.ins[k]
                if y.op == "anewarray":
                    break
                if y.op in ("ldc", "ldc_w") and y.cmt.startswith("String"):
                    strs.append(y.cmt[len("String "):] if y.cmt != "String" else "")
                k -= 1
            out = list(reversed(strs))
    return out


def scene67_proof(an: dict) -> dict:
    """DC-5 路径级证明：scene67（TAK_264_268）不可达。

      P1 值域（依赖存档往返引理）：值域(a.difficultyIndex) = {0}。
      P2 tryRunScene 的 tableswitch 键 67 段 = `… getfield difficultyIndex; ifle L;
         iconst_1; goto L2; L: iconst_0; L2: iand; istore_3`，且 ifle 的 fall 边
         （→iconst_1）在死边表 ⇒ 该段恒 iand(_,0)=0 ⇒ tryRunScene(67,·) 恒返回 0。
      P3 loadLevelScript 全调用点清点：常量实参集 ∌ 67；动态实参（entityParam）各点
         均被同参 tryRunScene 的 ifeq 守卫跳过（目标 > 调用点）；解析实参（GUT 臂）
         语料目标集 ∌ 67。
      ⇒ loadLevelScript(67) 无可达调用点 ⇒ script67 永不装入 ⇒ TAK_264_268 死。"""
    res = {"ok": False, "steps": []}
    F = ROUNDTRIP_FIELD
    methods = an["methods"]

    # ---- P1
    V = vals_of(an["vals"], F)
    p1 = V == frozenset({0})
    res["steps"].append(f"P1 {'✓' if p1 else '✗'} 值域({F}) = "
                        + ("{0}（存档往返引理生效）" if p1 else f"{V if V is not TOP else '⊤'}（引理未生效）"))

    # ---- P2
    p2 = False
    d2 = "未找到"
    trk = [k for k in methods if k[1] == "tryRunScene"]
    if len(trk) == 1:
        m = methods[trk[0]]
        sw = next((x for x in m.ins if x.op in SWITCH and 67 in x.cases), None)
        hit = None
        for x in m.ins:
            if x.op == "ifle" and guard(m, m.idx[x.off]) == ("fld", F):
                hit = x  # case67 = `> 0` 编译形态 ifle（case16 的 `== 0` 为 ifne）
        if sw is not None and hit is not None:
            fi = m.idx[hit.off] + 1
            fall_ins = m.ins[fi]
            jump_ins = m.ins[m.idx[int(hit.arg)]]
            fall_edge = (hit.off, fall_ins.off, ("fall",))
            in_dead = fall_edge in an["dead"][trk[0]]
            p2 = (fall_ins.op == "iconst_1" and jump_ins.op == "iconst_0" and in_dead)
            d2 = (f"tableswitch 键 67→@{sw.cases[67]}；ifle@{hit.off} fall→{fall_ins.op}@{fall_ins.off} "
                  f"{'∈' if in_dead else '∉'}死边表，jump→{jump_ins.op}@{jump_ins.off}"
                  f"（恒 0 ⇒ iand ⇒ var3=0 ⇒ ireturn 0）")
    res["steps"].append(f"P2 {'✓' if p2 else '✗'} tryRunScene(67) 恒 false：{d2}")

    # ---- P3 loadLevelScript 调用点清点
    p3 = True
    d3 = []
    consts: set = set()
    corpus_gut: set = set()
    for s in script_corpus(methods):
        corpus_gut |= {int(g) for g in re.findall(r"GUT_(-?\d+)", s)}
    dynamic_ok = 0
    for k, m in methods.items():
        for j, x in enumerate(m.ins):
            if x.op not in INVOKE or callee_of(x, m.cls) != ("a", "loadLevelScript", "(I)Z"):
                continue
            prev = m.ins[j - 1]
            c = const_of(prev)
            if c is not None:
                consts.add(c)
                continue
            w = m.ins[max(0, j - 4):j]
            if (len(w) >= 3 and w[-1].op == "saload" and w[-2].op.startswith("iload")
                    and w[-3].op == "getfield" and field_of(w[-3], m.cls) == "a.entityParam"):
                guarded = False
                for q in range(j - 1, max(-1, j - 30), -1):
                    y = m.ins[q]
                    if y.op in INVOKE and callee_of(y, m.cls) == ("a", "tryRunScene", "(IZ)Z"):
                        wn = m.ins[q + 1] if q + 1 < len(m.ins) else None
                        if wn is not None and wn.op in ("ifeq", "ifne") and int(wn.arg) > x.off:
                            guarded = True
                        break
                p3 &= guarded
                dynamic_ok += 1 if guarded else 0
                d3.append(f"{m.label}@{x.off} 动态(entityParam) "
                          f"{'✓同参 tryRunScene 守卫' if guarded else '✗无守卫'}")
                continue
            if m.name == "executeScriptInstruction":
                win = m.ins[max(0, j - 12):j]
                gut_shape = (any(y.op == "bastore" for y in win)
                             and any(y.op == "invokespecial" and "parseScriptInt" in y.cmt
                                     for y in m.ins))
                okc = gut_shape and 67 not in corpus_gut
                p3 &= okc
                d3.append(f"{m.label}@{x.off} 解析(GUT) {'✓' if okc else '✗'} "
                          f"语料 GUT 目标={sorted(corpus_gut)}")
                continue
            p3 = False
            d3.append(f"{m.label}@{x.off} ✗ 未知实参形态 `{prev.op}`")
    p3 = p3 and 67 not in consts and dynamic_ok == 2
    res["steps"].append(
        f"P3 {'✓' if p3 else '✗'} loadLevelScript 实参清点：常量={sorted(consts)}（∌67）；"
        f"动态点 {dynamic_ok}/2 受同参 tryRunScene 守卫；GUT 语料目标={sorted(corpus_gut)}（∌67）")
    res["steps"].append("     " + "；".join(d3))
    res["ok"] = p1 and p2 and p3
    return res


# ---------------------------------------------------------------- 不动点

def analyze() -> dict:
    methods: dict = {}
    shas = {}
    for c in CLASSES:
        for m in parse_class(c, javap(c)):
            methods[m.key] = m
        shas[c] = sha256(RENAMED / f"{c}.class")
    entries = {k for k, m in methods.items() if m.cls != "a" or not m.private}
    lemma = mvs_lemma(methods)
    rt = save_roundtrip_lemma(methods)  # DC-5 数据域前提（difficultyIndex 存档往返）
    extra = defaultdict(dict)
    if lemma["ok"]:
        mk, e, why = lemma["edge"]
        extra[mk][e] = why
    live = set(methods)
    reach = {k: {x.off for x in m.ins} for k, m in methods.items()}
    vals = value_sets(field_writes(methods, reach, live, rt))
    for it in range(1, 100):
        dead = {k: dead_edges(m, vals, extra.get(k, {})) for k, m in methods.items()}
        reach = {k: reachable(m, dead[k]) for k, m in methods.items()}
        new_live, q = set(), deque(sorted(entries))
        while q:
            k = q.popleft()
            if k in new_live or k not in methods:
                continue
            new_live.add(k)
            m = methods[k]
            for x in m.ins:
                if x.op in INVOKE and x.off in reach[k]:
                    c = callee_of(x, m.cls)
                    if c in methods and c not in new_live:
                        q.append(c)
        writes = field_writes(methods, reach, new_live, rt)
        new_vals = value_sets(writes)
        if new_live == live and new_vals == vals:
            break
        live, vals = new_live, new_vals
    return {"methods": methods, "entries": entries, "live": live, "vals": vals, "dead": dead,
            "reach": reach, "writes": writes, "lemma": lemma, "rt": rt, "iterations": it, "sha": shas}


def callers_of(an: dict, name: str, cls: str = "a") -> list[tuple[str, int]]:
    """字节码调用图：类 cls 的方法 name（任意描述符）的全部调用点 (调用者标签, 偏移)。"""
    out = []
    for k, m in an["methods"].items():
        for x in m.ins:
            if x.op in INVOKE:
                c = callee_of(x, m.cls)
                if c and c[0] == cls and c[1] == name:
                    out.append((m.label, x.off))
    return sorted(out)


def explain_site(an: dict, caller, site_off: int) -> list[dict]:
    """调用点不可达的充分切断边：**仅剪这一条**死边（其余边全部恢复）即令调用点不可达
    ⇒ 该守卫单独就足以证明此调用点死（多条均充分时全部列出）。"""
    m = an["methods"][caller]
    out = []
    for e, why in an["dead"][caller].items():
        if site_off not in reachable(m, {e: why}):
            out.append({"edge": f"@{e[0]}→{e[1]}", **why})
    return out


def ledger_aliases() -> dict[str, str]:
    """data/naming/ledger.jsonl 的 method 记录（applied）：旧引用名（ref 末段）→ 现名。裁决对象以裁决时名字
    登记，改名批次落地后经此映射到现名（不因改名静默失效）。"""
    p = ROOT / "data/naming/ledger.jsonl"
    out = {}
    if not p.exists():
        return out
    for l in p.read_text(encoding="utf-8").splitlines():
        if not l.strip():
            continue
        d = json.loads(l)
        if d.get("kind") == "method" and d.get("status") == "applied":
            tail = d.get("ref", "").split(".")[-1]
            if tail and tail != d.get("new"):
                out[tail] = d["new"]
    return out


def rulings(an: dict | None = None) -> dict:
    """裁决结果（cover_matrix.py 调用）：{DC-n: {ok, methods: {现名: {dead, sites:[…], alias_of?}}}}。"""
    an = an or analyze()
    by_name = defaultdict(list)
    for k, m in an["methods"].items():
        if m.cls == "a":
            by_name[m.name].append(k)
    alias = ledger_aliases()
    callers = defaultdict(list)
    for k, m in an["methods"].items():
        for x in m.ins:
            if x.op in INVOKE:
                c = callee_of(x, m.cls)
                if c in an["methods"]:
                    callers[c].append((k, x.off))
    out = {}
    for rid, spec in RULINGS.items():
        ms = {}
        for orig in spec["methods"]:
            name = orig if orig in by_name or orig not in alias else alias[orig]
            ks = by_name.get(name, [])
            if len(ks) != 1:
                ms[name] = {"dead": False, "error": f"方法 {name} 在改名类中出现 {len(ks)} 次"}
                continue
            k = ks[0]
            sites = []
            for ck, off in callers[k]:
                cm = an["methods"][ck]
                if ck not in an["live"]:
                    sites.append({"caller": cm.label, "off": off, "why": "调用方法本身已证死"})
                elif off not in an["reach"][ck]:
                    sites.append({"caller": cm.label, "off": off, "why": "调用点不可达",
                                  "cut": explain_site(an, ck, off)})
                else:
                    sites.append({"caller": cm.label, "off": off, "why": "可达（裁决不成立）"})
            ms[name] = {"dead": k not in an["live"], "entry": k in an["entries"], "sites": sites}
            if name != orig:
                ms[name]["alias_of"] = orig
        out[rid] = {"title": spec["title"], "ok": all(v["dead"] for v in ms.values()), "methods": ms}
    return out


def field_evidence(an: dict, f: str) -> str:
    ws = an["writes"].get(f, [])
    V = vals_of(an["vals"], f)
    groups = defaultdict(list)
    for meth, off, desc, src, counted in ws:
        groups[(desc, counted)].append(f"{meth}@{off}")
    parts = []
    for (desc, counted), sites in sorted(groups.items(), key=lambda kv: (not kv[0][1], kv[0][0])):
        tag = "" if counted else "〔死代码内，不计〕"
        parts.append(f"{desc}×{len(sites)}{tag}")
    vs = "⊤" if V is TOP else "{" + ",".join(map(str, sorted(V))) + "}"
    return f"{f.split('.')[-1]}：putfield {len(ws)} 处 = " + "，".join(parts) + f" ⇒ 值域 {vs}（含缺省 0）"


def report(an: dict, rs: dict, proof: dict | None = None) -> str:
    proof = proof or {"ok": False, "steps": ["（未计算）"]}
    L = []
    a = L.append
    a("# 白项 b 类死代码裁决：字节码机械复核报告（tools/dead_code_ruling.py）")
    a("")
    a(f"- 输入：renamed a.class sha256={an['sha']['a'][:16]}… / CMidlet.class sha256={an['sha']['CMidlet'][:16]}…；"
      f"方法 {len(an['methods'])}（入口点 {len(an['entries'])}：a 的非 private 方法 + CMidlet 全部 + static{{}}）")
    a(f"- 不动点：{an['iterations']} 轮收敛；存活方法 {len(an['live'])} / 证死 {len(an['methods']) - len(an['live'])}")
    deadset = sorted(m.label for k, m in an["methods"].items() if k not in an["live"])
    a(f"- 证死方法全集：{', '.join(deadset)}")
    a("")
    guard_fields = set()
    for rid, r in rs.items():
        a(f"## {rid} {r['title']}：{'✓ 证死' if r['ok'] else '✗ 未证死'}")
        a("")
        for name, v in r["methods"].items():
            if "error" in v:
                a(f"- {name}：✗ {v['error']}")
                continue
            alias = f"（裁决时名 {v['alias_of']}）" if v.get("alias_of") else ""
            a(f"- **{name}**{alias}：{'死' if v['dead'] else '活'}；调用点 {len(v['sites'])}"
              + ("（入口点！）" if v["entry"] else ""))
            for s in v["sites"]:
                cut = ""
                for c in s.get("cut", []):
                    guard_fields.add(c["field"])
                    if c["kind"] in SWITCH:
                        cut += f"；切断 = {c['kind']}({c['field'].split('.')[-1]}){c['edge']} 键{{{','.join(c['keys'])}}}"
                    elif c["kind"] == "MVS 引理":
                        cut += f"；切断 = {c['field']} 相等分支{c['edge']}（MVS 引理）"
                    else:
                        cut += f"；切断 = {c['kind']}({c['field'].split('.')[-1]}){c['edge']} {c['branch']} 分支"
                a(f"  - {s['caller']}@{s['off']}：{s['why']}{cut}")
        a("")
    a("## 守卫字段值域（存活代码写点，putfield 前驱产生指令）")
    a("")
    for f in sorted(guard_fields - {"String.equals(\"MVS\")"}):
        a(f"- {field_evidence(an, f)}")
        srcs = sorted({src[1] for _, _, _, src, counted in an["writes"].get(f, [])
                       if isinstance(src, tuple) and src[0] == "copy" and counted})
        for g in srcs:
            a(f"  - 拷贝源 {field_evidence(an, g)}")
    a("")
    a("## MVS 引理（DC-2 的数据域前提）")
    a("")
    a(f"- 比较点：{an['lemma'].get('site', '?')}；结论：{'✓ 相等分支死' if an['lemma']['ok'] else '✗ 不成立'}")
    for s in an["lemma"]["steps"]:
        a(f"- {s}")
    a("")
    a("## 存档往返归纳引理（DC-5 的数据域前提）")
    a("")
    a(f"- 结论：{'✓ 非恒定写点改判自拷贝 ⇒ 值域(difficultyIndex) = {0}' if an['rt']['ok'] else '✗ 不成立（值域 ⊤，DC-5 不可证）'}")
    for s in an["rt"]["steps"]:
        a(f"- {s}")
    a(f"- 值域落点：{field_evidence(an, ROUNDTRIP_FIELD)}")
    a("")
    a("## DC-5 scene67 路径证明（混沌引子 TAK_264_268 不可达）")
    a("")
    a(f"- 结论：{'✓ 证死' if proof['ok'] else '✗ 未证死'}")
    for s in proof["steps"]:
        a(f"- {s}")
    a("")
    return "\n".join(L) + "\n"


def main() -> int:
    an = analyze()
    rs = rulings(an)
    proof = scene67_proof(an)  # DC-5 路径级证明（依赖存档往返引理）
    if "--json" in sys.argv:
        print(json.dumps({"sha": an["sha"], "iterations": an["iterations"], "rulings": rs,
                          "roundtrip": an["rt"], "scene67": proof,
                          "lemma": {k: v for k, v in an["lemma"].items() if k != "edge"}},
                         ensure_ascii=False, indent=1, default=str))
    elif "--check" not in sys.argv:
        print(report(an, rs, proof), end="")
    bad = [rid for rid, r in rs.items() if not r["ok"]]
    if not proof["ok"]:
        bad.append("DC-5")
    if "--check" in sys.argv:
        print("死代码裁决复核：" + ("全部成立 " + "/".join(list(rs) + ["DC-5"]) if not bad
                                else f"不成立 {bad}"))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
