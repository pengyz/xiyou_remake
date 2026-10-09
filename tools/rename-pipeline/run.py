#!/usr/bin/env python3
"""run.py — 机械反混淆管线 v1：签名唯一化重映射 + javap 等价验证（一键、幂等）。

用法：
  python3 tools/rename-pipeline/run.py            # 构建：重映射 + CFR 再生无歧义版 Java
  python3 tools/rename-pipeline/run.py --verify   # 构建 + javap 指令级等价验证与报告

产物：
  analysis/rename-pipeline/orig/*.class       original/ JAR 提取的原始 class（只读输入）
  analysis/rename-pipeline/renamed/*.class    重映射后 class（可再生中间产物）
  reference/src/deobf/*.java                  CFR 再生的无歧义版 Java（入库）
  data/naming/remap-table.json                全量 old→new 符号映射表（入库）
  tools/rename-pipeline/_verify/              javap 等价验证报告（--verify，入库）
  analysis/rename-pipeline/verify/            javap 原始/归一化反汇编（可再生）

确定性变换（见 renamer.py 命名方案）：字段 f_<type>_<NN>、方法 m_<NNN>。
不做语义命名；语义命名之后叠加在 data/naming/ledger.jsonl（本管线留空）。
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
sys.path.insert(0, str(HERE))

import classfile as cfmod          # noqa: E402
import renamer                     # noqa: E402
import verifier                    # noqa: E402

JAR = ROOT / "original/囧囧西游-大闹天宫.jar"
CFR = ROOT / "analysis/cfr.jar"
CFR_URL = "https://repo1.maven.org/maven2/org/benf/cfr/0.152/cfr-0.152.jar"
WORK = ROOT / "analysis/rename-pipeline"
ORIG, RENAMED, JAVAPDIR = WORK / "orig", WORK / "renamed", WORK / "verify"
DEOBF = ROOT / "reference/src/deobf"
NAMING = ROOT / "data/naming"
REMAP = NAMING / "remap-table.json"
VERIFY_OUT = HERE / "_verify"

OBF_NAME = re.compile(r"^[a-zA-Z]{1,2}$")
JAVA_KEYWORDS = {"if", "for", "do", "else", "try", "catch", "finally", "return", "switch",
                 "case", "default", "break", "continue", "throw", "new", "this", "super",
                 "null", "true", "false", "synchronized", "instanceof", "assert", "while"}
# CFR 输出的类体第一层声明恰为 4 空格缩进且下一行首非空格（方法体 8+ 空格）
SRC_DECL = re.compile(
    r"^    (?=\S)(?:public|protected|private|static|final|abstract|native|synchronized|"
    r"transient|volatile|strictfp| )*[\w.$/\[\]<>, ]+?\s+(\w+)\s*[;(=]")


def log(msg: str):
    print(f"[rename-pipeline] {msg}")


def sha256_bytes(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def run(cmd, **kw):
    r = subprocess.run(cmd, capture_output=True, text=True, **kw)
    if r.returncode != 0:
        raise RuntimeError(f"命令失败: {' '.join(map(str, cmd))}\n{r.stdout[-2000:]}\n{r.stderr[-2000:]}")
    return r


def ensure_cfr():
    if CFR.exists():
        return
    log(f"下载 CFR 0.152 … {CFR_URL}")
    run(["curl", "-sL", "-o", str(CFR), CFR_URL])
    if not CFR.exists() or CFR.stat().st_size == 0:
        raise RuntimeError("CFR 下载失败（离线环境请预置 analysis/cfr.jar）")


# ---------- 构建 ----------

def build() -> dict:
    if not JAR.exists():
        raise RuntimeError(f"original JAR 缺失: {JAR}")
    ensure_cfr()

    for d in (ORIG, RENAMED, JAVAPDIR):
        d.mkdir(parents=True, exist_ok=True)
    DEOBF.mkdir(parents=True, exist_ok=True)

    # ① 提取全部 game class（只读 original/，解包到 analysis/）
    with zipfile.ZipFile(JAR) as z:
        class_entries = sorted(n for n in z.namelist() if n.endswith(".class"))
        for name in class_entries:
            (ORIG / Path(name).name).write_bytes(z.read(name))
    log(f"提取 {len(class_entries)} 个 class: {', '.join(class_entries)}")

    # ② 解析 + 建映射 + 重写
    classes = {}
    for name in class_entries:
        p = ORIG / Path(name).name
        cff = cfmod.parse(p.read_bytes())
        classes[cff.class_name(cff.this_idx)] = cff
    records = renamer.build_rename_map(classes)
    for i, rec in enumerate(records, 1):
        rec["id"] = f"R-{i:04d}"
    renamer.apply_rename(classes, records)
    problems = renamer.consistency_check(classes, records)
    if problems:
        raise RuntimeError("重写后一致性自检失败:\n  " + "\n  ".join(problems))

    for cname, cff in classes.items():
        (RENAMED / f"{cname}.class").write_bytes(cfmod.serialize(cff))
    n_renamed = sum(1 for r in records if r["renamed"])
    log(f"重映射成员 {n_renamed}/{len(records)} 个，一致性自检通过")

    # ③ 映射表落 data/naming/remap-table.json
    REMAP.parent.mkdir(parents=True, exist_ok=True)
    table = {
        "schema": "rename-pipeline/remap-table/v1",
        "tool": "tools/rename-pipeline/run.py",
        "source_jar": "original/囧囧西游-大闹天宫.jar",
        "source_jar_sha256": sha256_bytes(JAR.read_bytes()),
        "scheme": {
            "field": "f_<type>_<NN> —— type 记号（int/byte/bool/…/类简单名；数组 +_arr/_arr2…）+ 类内同 type 序号(2 位)",
            "method": "m_<NNN> —— 类内声明序号(3 位)",
            "rule": "仅改混淆名 ^[a-zA-Z]{1,2}$（本 JAR 混淆器产出空间：a..z/A..Z/aa..cj/aA..bZ）"
                    "且 private/static（不可能 override 外部 API）的成员；"
                    "构造器/<clinit>/MIDP override 等保留原名（renamed=false）",
            "deterministic": "序号取 class 文件声明序；同一 JAR 多次运行输出逐字节一致（幂等）",
        },
        "counts": {
            "classes": len(classes),
            "symbols": len(records),
            "renamed": n_renamed,
            "kept": len(records) - n_renamed,
        },
        "semantic_naming": "data/naming/ledger.jsonl（本表为机械层，语义命名叠加其上）",
        "evidence": "tools/rename-pipeline/_verify/report.md（javap 指令级等价验证）",
        "symbols": records,
    }
    REMAP.write_text(json.dumps(table, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    log(f"映射表 → {REMAP.relative_to(ROOT)}（{len(records)} 条）")

    # ④ CFR 再生无歧义版 Java
    for old in DEOBF.glob("*.java"):
        old.unlink()
    class_files = [str(RENAMED / f"{c}.class") for c in sorted(classes)]
    run(["java", "-jar", str(CFR), *class_files, "--outputdir", str(DEOBF)])
    sources = sorted(DEOBF.glob("*.java"))
    log(f"CFR 再生 {len(sources)} 个 Java: {', '.join(p.name for p in sources)}")

    # ⑤ 产物自检：无混淆名单字母/短名残留
    problems = check_outputs(classes, sources)
    if problems:
        raise RuntimeError("产物自检失败:\n  " + "\n  ".join(problems))
    log("产物自检通过：字段/方法名全部唯一化，无混淆短名残留")
    return {"classes": classes, "records": records, "sources": sources}


def check_outputs(classes: dict, sources: list) -> list:
    """产物自检（A 级：javap 声明 + Java 源声明双路扫描）。"""
    problems = []
    # (a) javap -p 扫描重映射 class 的成员声明
    for cname in sorted(classes):
        out = run(["javap", "-p", str(RENAMED / f"{cname}.class")]).stdout
        for line in out.splitlines():
            m = verifier.DECL_LINE.match(line) or verifier.CTOR_DECL.match(line)
            if not m:
                continue
            name = m.group(1)
            if name == cname or name in ("<init>", "<clinit>"):
                continue
            if OBF_NAME.fullmatch(name):
                problems.append(f"{cname} 成员残留混淆名: {name}")
    # (b) CFR 源码扫描：类体第一层（4 空格缩进）声明
    for src in sources:
        cls = src.stem
        for line in src.read_text(encoding="utf-8").splitlines():
            m = SRC_DECL.match(line)
            if not m:
                continue
            name = m.group(1)
            if name == cls or name in JAVA_KEYWORDS:
                continue  # 构造器 / 控制流残影
            if OBF_NAME.fullmatch(name):
                problems.append(f"{src.relative_to(ROOT)} 成员残留混淆名: {name} @ {line.strip()[:60]}")
        text = src.read_text(encoding="utf-8")
        if "Duplicate member names" in text:
            problems.append(f"{src.relative_to(ROOT)} 仍有 CFR 歧义告警注释")
    return problems


def locals_note(sources: list) -> str:
    """统计 CFR 生成的单字母局部变量名（透明披露用，不改源码）。"""
    decl = re.compile(r"^ {8,}(?:final\s+)?[\w.$\[\]]+\s+([a-zA-Z])\s*(?:=|;)")
    cnt = {}
    for src in sources:
        for line in src.read_text(encoding="utf-8").splitlines():
            m = decl.match(line)
            if m:
                cnt[m.group(1)] = cnt.get(m.group(1), 0) + 1
    if not cnt:
        return "CFR 输出中无单字母局部变量名"
    detail = "、".join(f"`{k}`×{v}" for k, v in sorted(cnt.items()))
    return f"CFR 输出中存在单字母局部变量名 {detail}（共 {sum(cnt.values())} 处）"


# ---------- 验证 ----------

def verify(build_state: dict):
    classes = build_state["classes"]
    records = build_state["records"]
    VERIFY_OUT.mkdir(parents=True, exist_ok=True)

    # 反向映射（按类）：new → old
    reverse = {}
    for rec in records:
        if rec["renamed"]:
            reverse.setdefault(rec["class"], {})[rec["new"]] = rec["old"]

    report_rows, method_rows, all_ok = [], [], True
    for cname in sorted(classes):
        before_raw = run(["javap", "-c", "-p", "-l", str(ORIG / f"{cname}.class")]).stdout
        after_raw = run(["javap", "-c", "-p", "-l", str(RENAMED / f"{cname}.class")]).stdout
        (JAVAPDIR / f"{cname}.before.javap.txt").write_text(before_raw, encoding="utf-8")
        (JAVAPDIR / f"{cname}.after.javap.txt").write_text(after_raw, encoding="utf-8")

        before = verifier.javap_normalize(before_raw, reverse, cname)
        after = verifier.javap_normalize(after_raw, reverse, cname)
        (JAVAPDIR / f"{cname}.before.norm.txt").write_text(before, encoding="utf-8")
        (JAVAPDIR / f"{cname}.after.norm.txt").write_text(after, encoding="utf-8")

        diff = verifier.diff_texts(before, after,
                                   f"{cname} (改名前, 归一化)", f"{cname} (改名后, 归一化+符号还原)")
        (VERIFY_OUT / f"diff-{cname}.txt").write_text(diff, encoding="utf-8")
        ok = (diff == "")
        all_ok = all_ok and ok

        mb = verifier.split_members(before)
        ma = verifier.split_members(after)
        old_names = [x[1] for x in verifier.split_members(before_raw)]
        new_names = [x[1] for x in verifier.split_members(after_raw)]
        per_method = []
        for i, ((hb, nb, tb, ib), (ha, na, ta, ia)) in enumerate(zip(mb, ma)):
            per_method.append((old_names[i], new_names[i], ib, ia, tb == ta))
        if len(mb) != len(ma) or len(old_names) != len(new_names):
            all_ok = False
        method_rows.append((cname, per_method))

        n_instr = sum(x[2] for x in per_method)
        report_rows.append({
            "class": cname,
            "members": len(mb),
            "instructions": n_instr,
            "sha_before": verifier.sha256_text(before),
            "sha_after": verifier.sha256_text(after),
            "diff_lines": diff.count("\n"),
            "equal": ok,
        })

    write_report(records, report_rows, method_rows, all_ok,
                 locals_note=locals_note(build_state["sources"]))

    # 幂等 manifest：全部产物 sha256（不含 manifest 自身）
    outputs = [REMAP.relative_to(ROOT)] + \
              [p.relative_to(ROOT) for p in sorted(DEOBF.glob("*.java"))] + \
              [p.relative_to(ROOT) for p in sorted(VERIFY_OUT.glob("*")) if p.name != "manifest.json"] + \
              [p.relative_to(ROOT) for p in sorted(JAVAPDIR.glob("*"))]
    manifest = {str(p): sha256_bytes((ROOT / p).read_bytes()) for p in outputs}
    (VERIFY_OUT / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if not all_ok:
        raise RuntimeError("javap 等价验证失败：归一化后仍有差异，见 _verify/diff-*.txt")
    log("javap 等价验证通过：归一化后逐字节一致（仅符号名变化）")
    log(f"报告 → {VERIFY_OUT.relative_to(ROOT)}/report.md")


def write_report(records, report_rows, method_rows, all_ok, locals_note):
    n_renamed = sum(1 for r in records if r["renamed"])
    kept = [r for r in records if not r["renamed"]]
    lines = []
    a = lines.append
    a("# javap 指令级等价验证报告 — 机械反混淆管线 v1")
    a("")
    a(f"- 结论：**{'PASS — 重映射仅改变符号名，语义不变' if all_ok else 'FAIL'}**")
    a("- 复现：`python3 tools/rename-pipeline/run.py --verify`（确定性输出，无时间戳 ⇒ 幂等）")
    a("- 输入：`original/囧囧西游-大闹天宫.jar`（只读）→ `analysis/rename-pipeline/orig/*.class`")
    a("- 对比对象：`analysis/rename-pipeline/orig/` vs `analysis/rename-pipeline/renamed/*.class`")
    a(f"- 映射表：`data/naming/remap-table.json`（{len(records)} 条符号，改名 {n_renamed} 条）")
    a("")
    a("## 验证方法（为什么这能证明“仅符号名变化”）")
    a("")
    a("1. `javap -c -p -l` 分别反汇编改名前/后 class，逐指令文本落盘"
      "（`analysis/rename-pipeline/verify/<class>.{before,after}.javap.txt`）；")
    a("2. 归一化两份文本：常量池槽号 `#NNN → #<cp>`（重映射以常量池**末尾追加**新 "
      "Utf8/NameAndType 的方式实施，槽号位移是布局而非语义）；")
    a("3. 对改名后文本做**符号还原**：按映射表把新名（f_*/m_*）替换回旧名。替换只发生在"
      "成员声明行与 `// Field|Method|InterfaceMethod` 注释中，字符串常量等不动；"
      "被限定引用（owner.name:desc）按 owner 查表，非限定引用按当前类查表；")
    a("4. 两份归一化文本**逐字节相同**（diff 0 行）⇒ 指令序列、每条指令操作数指向的"
      "具体成员（owner+name+desc）、行号表、异常表、成员签名、访问标志全部一致 —— "
      "差异只剩符号名本身。")
    a("")
    a("同时跑三重自检（失败即退出码非 0）：① 重写后无任何 Fieldref/Methodref 仍指向旧 "
      "(owner,name,desc)；② 类内成员名（字段+方法合并）唯一；③ 产物无混淆短名残留"
      "（javap 声明 + CFR 源码双路扫描）。")
    a("")
    a("## 逐类结果")
    a("")
    a("| class | 成员数 | 指令数 | 归一化 sha256(前) | 归一化 sha256(后) | diff 行数 | 结论 |")
    a("|---|---|---|---|---|---|---|")
    for r in report_rows:
        a(f"| {r['class']} | {r['members']} | {r['instructions']} | `{r['sha_before'][:16]}…` "
          f"| `{r['sha_after'][:16]}…` | {r['diff_lines']} | {'一致 ✓' if r['equal'] else '不一致 ✗'} |")
    a("")
    a("完整 diff（空 = 无差异）：`tools/rename-pipeline/_verify/diff-<class>.txt`。")
    a("")
    for cname, per_method in method_rows:
        a(f"## {cname} 逐成员对比（{len(per_method)} 个成员）")
        a("")
        a("| # | 旧名 | 新名 | 指令数(前) | 指令数(后) | 归一化块一致 |")
        a("|---|---|---|---|---|---|")
        for i, (old, new, ib, ia, eq) in enumerate(per_method, 1):
            a(f"| {i} | {old} | {new} | {ib} | {ia} | {'✓' if eq else '✗'} |")
        a("")
    a("## 保留原名的成员（old == new，共 %d 个）" % len(kept))
    a("")
    a("| class | kind | 名称 | signature | 保留原因 |")
    a("|---|---|---|---|---|")
    for r in kept:
        reason = ("构造器/静态初始化" if r["old"] in ("<init>", "<clinit>")
                  else "MIDP API override/实现（改名会改变虚分派语义）")
        a(f"| {r['class']} | {r['kind']} | {r['old']} | `{r['signature']}` | {reason} |")
    a("")
    a("## 产物自检")
    a("")
    a("- 字段/方法声明扫描（javap -p + CFR 源码双路）：混淆短名（`^[a-zA-Z]{1,2}$`）残留 "
      "**0 个**；类内字段+方法名全局唯一。")
    a(f"- CFR 局部变量名说明：{locals_note}。字节码无 LocalVariableTable，"
      "局部变量名是 CFR 按类型启发式生成的**投影层命名**（方法作用域内唯一、无歧义），"
      "不属于符号重映射范围；本目录 Java 保持 CFR 原样输出以便与 "
      "`bash tools/decompile.sh` 的再生产流程同构。")
    a("")
    a("## 备注")
    a("")
    a("- 未改名成员（构造器 / `<clinit>` / `paint` `run` `keyPressed` 等 MIDP API override）"
      "在映射表中 `renamed=false`、old == new：override 不得改名，否则虚分派语义改变。")
    a("- `data/naming/ledger.jsonl` 保持为空 —— 语义命名留给台账驱动的后续批次，"
      "叠加在本映射表之上。")
    a("- 本报告不含时间戳：同一 JAR 重跑输出逐字节一致（见 `_verify/manifest.json`）。")
    (VERIFY_OUT / "report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


# ---------- 入口 ----------

def main() -> int:
    ap = argparse.ArgumentParser(description="机械反混淆管线 v1（签名唯一化重映射 + javap 等价验证）")
    ap.add_argument("--verify", action="store_true",
                    help="构建后执行 javap 指令级等价验证并归档报告到 tools/rename-pipeline/_verify/")
    args = ap.parse_args()
    state = build()
    if args.verify:
        verify(state)
    print("[rename-pipeline] OK")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as e:  # noqa: BLE001 —— 工具入口统一失败出口
        print(f"[rename-pipeline] FAIL: {e}", file=sys.stderr)
        sys.exit(1)
