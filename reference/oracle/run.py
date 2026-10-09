#!/usr/bin/env python3
"""run.py — reference/oracle 差分 trace 宿主（编排层）。

用法：
  python3 reference/oracle/run.py             # 完整差分（gates reference 门禁入口）
  python3 reference/oracle/run.py --selftest  # 快速确定性自测（小 tick 预算）
  python3 reference/oracle/run.py --ticks N   # 指定 tick 预算

流程：
 1. 构建 shim + host（javac → _out/classes）；
 2. 变体准备：A = 原始版 class（original JAR 原样字节），B = t1 无歧义版
    （tools/rename-pipeline 重映射 class）；两者施加同一 T-变换
    （虚拟时钟/虚拟线程，见 ttransform.py）→ _out/variantA|B；
 3. 各自以固定输入脚本 + 固定虚拟时钟运行 oracle.host.Runner → trace.txt；
 4. 断言 ① A 两次运行逐字节一致（确定性）② A vs B 逐字节一致
    （机械重命名语义等价的运行时证明）；
 5. 产出 reference/oracle/_diff/report.md + api-surface.md。

产出约束：report 内容确定性（无时间戳/耗时），复跑逐字节一致；
trace/截图在 _out/（gitignore，不入库）。
退出码：0 全部通过；1 任一断言失败；2 构建/运行基础设施错误。
"""
import hashlib
import re
import shutil
import struct
import subprocess
import sys
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
OUT = HERE / "_out"
DIFF = HERE / "_diff"
JAR = ROOT / "original/囧囧西游-大闹天宫.jar"
SEED_CLASSES = ("a", "CMidlet")
DEFAULT_SCRIPT = HERE / "script-default.txt"
RENAME_OUT = ROOT / "analysis/rename-pipeline/renamed"
REMAP_TABLE = ROOT / "data/naming/remap-table.json"

sys.path.insert(0, str(HERE))
import ttransform  # noqa: E402


def sh(cmd, **kw):
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, **kw)
    return r


def sha256_file(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def log(msg):
    print(f"[oracle] {msg}")


# ---------------------------------------------------------------- 构建

def ensure_game_classes():
    """A 变体源：original JAR 内 class 原样字节（红线：只读）。
    B 变体源：tools/rename-pipeline 重映射产物（analysis/ 可再生工作区）。"""
    orig = OUT / "orig"
    orig.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(JAR) as z:
        for c in SEED_CLASSES:
            data = z.read(f"{c}.class")
            (orig / f"{c}.class").write_bytes(data)
    if not (RENAME_OUT / "a.class").exists() or not (RENAME_OUT / "CMidlet.class").exists():
        log(f"{RENAME_OUT} 缺失，运行 tools/rename-pipeline/run.py 再生…")
        r = sh([sys.executable, "tools/rename-pipeline/run.py"])
        if r.returncode != 0:
            log("rename-pipeline 再生失败:\n" + r.stdout[-2000:] + r.stderr[-2000:])
            sys.exit(2)
    return orig


def build_shim_host():
    classes = OUT / "classes"
    classes.mkdir(parents=True, exist_ok=True)
    sources = sorted(str(p.relative_to(ROOT)) for p in
                     list((ROOT / "reference/shim/src").rglob("*.java"))
                     + list((ROOT / "reference/oracle/src").rglob("*.java")))
    r = sh(["javac", "-encoding", "UTF-8", "-nowarn", "-d", str(classes)] + sources)
    if r.returncode != 0:
        log("javac 失败:\n" + r.stdout[-4000:] + r.stderr[-4000:])
        sys.exit(2)
    return classes


def ttransform_variants(orig: Path):
    results = {}
    for label, srcdir in (("A", orig), ("B", RENAME_OUT)):
        dst = OUT / f"variant{label}"
        dst.mkdir(parents=True, exist_ok=True)
        counts = {}
        for c in SEED_CLASSES:
            data = (srcdir / f"{c}.class").read_bytes()
            out, cnt = ttransform.transform(data, label=f"{label}:{c}")
            (dst / f"{c}.class").write_bytes(out)
            for k, v in cnt.items():
                counts[k] = counts.get(k, 0) + v
        results[label] = {
            "src": srcdir,
            "counts": counts,
            "sha": {c: sha256_file(srcdir / f"{c}.class") for c in SEED_CLASSES},
            "sha_t": {c: sha256_file(dst / f"{c}.class") for c in SEED_CLASSES},
        }
    return results


def verify_t_equivalence(orig: Path):
    """javap 归一化等价：T-变换前后，把 System/Thread 与 VTime/VThread 视为同一符号后
    逐字节相同 ⇒ 变换只动了引用 owner，指令/行号/异常表零改动。"""
    ok = True
    diffs = {}
    for label, src in (("A", orig), ("B", RENAME_OUT)):
        for c in SEED_CLASSES:
            before = sh(["javap", "-c", "-p", "-l", str(src / f"{c}.class")]).stdout
            after = sh(["javap", "-c", "-p", "-l", str(OUT / f"variant{label}" / f"{c}.class")]).stdout
            norm = [("java/lang/System", "oracle/vt/VTime"), ("java/lang/Thread", "oracle/vt/VThread")]
            b, a = before, after
            for old, new in norm:
                b = b.replace(old, "TGT")
                a = a.replace(new, "TGT")
            if b != a:
                ok = False
                diffs[f"{label}:{c}"] = first_diff_line(b, a)
    return ok, diffs


def first_diff_line(a: str, b: str):
    la, lb = a.splitlines(), b.splitlines()
    for i, (x, y) in enumerate(zip(la, lb)):
        if x != y:
            return f"line {i + 1}: {x!r} != {y!r}"
    return f"length differs {len(la)} vs {len(lb)}"


# ---------------------------------------------------------------- 运行

def run_variant(classes: Path, variant_dir: Path, label: str, ticks: int, script: Path):
    out_dir = OUT / label
    if out_dir.exists():
        shutil.rmtree(out_dir)
    out_dir.mkdir(parents=True)
    cp = f"{classes}:{variant_dir}:{JAR}"
    cmd = [
        "java", "-Djava.awt.headless=true", "-cp", cp,
        f"-Doracle.out={out_dir}",
        f"-Doracle.max={ticks}",
        f"-Doracle.script={script}",
        f"-Doracle.label={label}",
        "-Doracle.watchdog=300000",
        "oracle.host.Runner",
    ]
    r = sh(cmd)
    (out_dir / "stdout.txt").write_text(r.stdout, encoding="utf-8")
    (out_dir / "stderr.txt").write_text(r.stderr, encoding="utf-8")
    return r.returncode, out_dir


def trace_stats(trace: Path):
    ticks = 0
    frames = []
    for line in trace.read_text(encoding="utf-8").splitlines():
        if line.startswith("TICK "):
            ticks += 1
        elif line.startswith("FRAME sha="):
            frames.append(line.split("sha=")[1].split()[0])
    return ticks, frames


def canonicalize_stderr(text: str, remap: dict) -> str:
    """把堆栈里的原始混淆成员名映射到规范化名（两变体对齐后再比）。
    只处理 `at <class>.<member>(` 行，避免误替换游戏文本。"""
    def sub(m):
        cls, mem = m.group(1), m.group(2)
        return f"at {cls}.{remap.get((cls, mem), mem)}("
    return re.sub(r"at ([\w/$]+)\.([\w$<>]+)\(", sub, text)


def load_remap():
    if not REMAP_TABLE.exists():
        return {}
    import json
    data = json.loads(REMAP_TABLE.read_text(encoding="utf-8"))
    out = {}
    for s in data.get("symbols", []):
        if s.get("renamed"):
            out[(s["class"], s["old"])] = s["new"]
    return out


# ---------------------------------------------------------------- 报告

def write_report(res: dict):
    lines = []
    a = lines.append
    a("# reference/oracle 差分报告（自动生成）")
    a("")
    a("> 由 `python3 reference/oracle/run.py` 生成；**内容确定性**（无时间戳/耗时），"
      "复跑逐字节一致。trace 与截图在 `_out/`（gitignore），本报告只存结论与哈希。")
    a("")
    a("## 1. 结论")
    a("")
    a("| 判定项 | 结果 | 证据 |")
    a("|--------|------|------|")
    a(f"| 确定性：A1 vs A2 同输入 trace 逐字节一致 | **{res['det']}** | "
      f"trace sha256 `{res['sha_a1'][:16]}…` vs `{res['sha_a2'][:16]}…` |")
    a(f"| 语义等价：原始版(A) vs 无歧义版(B) 同输入 trace 逐字节一致 | **{res['eq']}** | "
      f"trace sha256 `{res['sha_a1'][:16]}…` vs `{res['sha_b1'][:16]}…` |")
    a(f"| stderr 对照（符号规范化后，辅助证据） | {res['stderr_cmp']} | 见 §4 |")
    a(f"| T-变换 javap 归一化等价 | **{res['t_ok']}** | System/Thread→VTime/VThread 归一化后逐字节相同 |")
    a(f"| 运行成功（退出码 0 / tick 数 > 0） | **{res['run_ok']}** | A1 ticks={res['ticks_a1']}, B1 ticks={res['ticks_b1']} |")
    a("")
    a("## 2. 对比对象")
    a("")
    a("| 变体 | 来源 | a.class sha256 | CMidlet.class sha256 |")
    a("|------|------|----------------|----------------------|")
    a(f"| A 原始版 | `original/囧囧西游-大闹天宫.jar`（原样字节） | `{res['sha_orig_a']}` | `{res['sha_orig_c']}` |")
    a(f"| B 无歧义版 | `analysis/rename-pipeline/renamed/`（t1 重映射产物） | `{res['sha_ren_a']}` | `{res['sha_ren_c']}` |")
    a("")
    a("两个变体施加**同一 T-变换**（常量池 Class 项重定向，见 §3）后运行。"
      "seed 版 `reference/seed/a.java` 是 CFR 投影（存在重复成员名与 "
      "`this = v3` 等不可编译结构），按 AGENTS.md §1「字节码是最终真相」，"
      "运行时 A 变体取 JAR 内原始字节码。")
    a("")
    a("## 3. T-变换（harness 适配，非游戏语义变更）")
    a("")
    a("原版依赖墙钟与真实线程，直接跑 trace 不可复现。T-变换把 "
      "`java/lang/System`（currentTimeMillis/gc/out）与 `java/lang/Thread`"
      "（&lt;init&gt;/start/sleep/yield）的常量池 Class 项重定向到 "
      "`oracle/vt/VTime`、`oracle/vt/VThread`（虚拟时钟 + 协作调度），"
      "指令字节/LineNumberTable/异常表零改动（javap 归一化等价验证，"
      "归一化即把上述两组 owner 视为同一符号）。**A/B 施加同一变换**，"
      "故差分结论对变换不变。")
    a("")
    a("| 变体 | 重定向引用计数 |")
    a("|------|----------------|")
    for label in ("A", "B"):
        counts = res["t_counts"][label]
        s = ", ".join(f"{k}→{v}" for k, v in sorted(counts.items())) or "(none)"
        a(f"| {label} | {s} |")
    a("")
    a("## 4. 确定性策略（oracle policy，两端一致）")
    a("")
    a("- **虚拟时钟**：`currentTimeMillis` 纯读；时间只在 `yield`(+1ms) 与 "
      "`sleep`(+ms) 推进。原版帧限速忙等（a.java:2424-2425）因此每帧确定性地"
      "空转 frameBudget 次后退出。")
    a("- **虚拟线程**：`Thread.start()` 仅登记，宿主在 `startApp` 返回后授权；"
      "同一时刻只有一个虚拟线程执行游戏代码，切换点在 start 授权/yield/sleep/线程结束"
      "（FIFO）。消除 JVM 调度不确定性。")
    a("- **输入投递**：固定脚本 `script-default.txt`，在 tick 边界"
      "（`serviceRepaints()`，a.java:2422）投递 keyPressed/keyReleased。")
    a("- **网络**：`Connector.open` 一律抛 IOException（确定性离线），"
      "驱动原版失败分支（a.java:8321 catch → -1）。")
    a("- **RMS**：内存实现（进程内静态 Map），跨运行为空。")
    a("- **音频**：Player/VolumeControl 状态机 stub，无声音输出，调用记入 EVT 流。")
    a("- **随机**：`java.util.Random` 保留原实现，seed 来自虚拟时钟 ⇒ 确定。")
    a("- **字形/字体**：shim 自定义度量 + 确定性示意字形（非设备字库）。"
      "文字布局证据看 trace 内 `OP drawString(...)` 操作流；像素哈希用于差分，"
      "不用于设备视觉保真（L4）。")
    a("")
    a("## 5. trace 结构与统计")
    a("")
    a("```")
    a("PRE ...")
    a("TICK 0001 vt=<虚拟毫秒>")
    a("FRAME sha=<像素sha256> w=240 h=320 png=frame-0001.png")
    a("  OPS count=<n>        # 绘制操作流（含 drawString 文本，≤256 条 + 总数）")
    a("  FLD <序号> <类型描述符> <值>   # 状态向量：class 文件声明序，不含符号名")
    a("INPUT press(-5)")
    a("OUT <System.out 行>")
    a("EVT <媒体/网络/线程事件>")
    a("END reason=<停止原因> ticks=<n>")
    a("```")
    a("")
    a(f"- tick 数：A1={res['ticks_a1']} A2={res['ticks_a2']} B1={res['ticks_b1']}")
    a(f"- trace 行数：A1={res['lines_a1']} B1={res['lines_b1']}")
    a(f"- 帧哈希链 sha256（全部 tick 的 FRAME 行摘要）：A1=`{res['frame_chain_a1'][:16]}…`，"
      f"B1=`{res['frame_chain_b1'][:16]}…`")
    a(f"- 末 3 帧像素哈希（A1）：{res['last_frames_a1']}")
    a(f"- 末 3 帧像素哈希（B1）：{res['last_frames_b1']}")
    a("")
    a("## 6. stderr 对照（辅助）")
    a("")
    a("异常堆栈含被重命名的成员符号；对照前经 `data/naming/remap-table.json` 规范化"
      "（`at <class>.<old>(` → 规范名），只处理堆栈行、不触碰游戏文本。")
    a("")
    a(f"- A1 stderr 行数={res['stderr_lines_a']}，B1 stderr 行数={res['stderr_lines_b']}")
    a(f"- 规范化后比对：**{res['stderr_cmp']}**")
    a("")
    a("## 7. API 覆盖")
    a("")
    a(f"见 [api-surface.md](api-surface.md)：MIDP/Nokia 引用 {res['api_shim']} 条全部由 "
      f"`reference/shim/` 提供，缺失 {res['api_missing']} 条；JDK 引用 {res['api_jdk']} 条存档。")
    a("")
    a("## 8. 已知局限（后续阶段处理）")
    a("")
    a("- 字形为示意图案、字体度量为 shim 常量 ⇒ 截图非设备像素级保真"
      "（Stage A 的 L4 视觉门禁在 P4 建立视觉比对器后另行标定）。")
    a("- 协作调度的线程交错点不同于真机并发语义；差分两端同一策略，"
      "未来 Rust 移植需复刻 oracle 策略（本文件是策略权威描述）。")
    a("- 网络永久离线、RMS 跨运行为空；联网/存档场景脚本待 P3 后补充。")
    a("- 本阶段差分只覆盖「机械重命名语义等价」；参考版 vs Rust 的 trace 差分"
      "（L1-L3）待 game-core/game-oracle 就绪后接入。")
    a("")
    (DIFF / "report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    log(f"报告 → {DIFF.relative_to(ROOT)}/report.md")


# ---------------------------------------------------------------- 主流程

def main():
    args = sys.argv[1:]
    selftest = "--selftest" in args
    ticks = 150
    if "--ticks" in args:
        ticks = int(args[args.index("--ticks") + 1])
    elif selftest:
        ticks = 40
    script = DEFAULT_SCRIPT
    if "--script" in args:
        script = Path(args[args.index("--script") + 1]).resolve()

    if not JAR.exists():
        log(f"缺 {JAR}")
        return 2
    DIFF.mkdir(parents=True, exist_ok=True)
    OUT.mkdir(parents=True, exist_ok=True)

    log("1/6 构建 shim + host …")
    classes = build_shim_host()
    log("2/6 变体准备（原始版 / t1 无歧义版）…")
    orig = ensure_game_classes()
    tinfo = ttransform_variants(orig)

    log("3/6 T-变换等价验证（javap 归一化）…")
    t_ok, t_diffs = verify_t_equivalence(orig)
    if not t_ok:
        log("T-变换 javap 归一化比对失败: " + repr(t_diffs))
        return 2

    log(f"4/6 运行 A1/A2/B1（tick 预算 {ticks}）…")
    rc_a1, dir_a1 = run_variant(classes, OUT / "variantA", "A1", ticks, script)
    rc_a2, dir_a2 = run_variant(classes, OUT / "variantA", "A2", ticks, script)
    if selftest:
        sha1, sha2 = sha256_file(dir_a1 / "trace.txt"), sha256_file(dir_a2 / "trace.txt")
        ok = rc_a1 == 0 and rc_a2 == 0 and sha1 == sha2
        log(f"[selftest] A1 rc={rc_a1} A2 rc={rc_a2}")
        log(f"[selftest] trace sha256 A1={sha1[:16]}… A2={sha2[:16]}… → "
            f"{'PASS 逐字节一致' if sha1 == sha2 else 'FAIL 不一致'}")
        t1, f1 = trace_stats(dir_a1 / "trace.txt")
        log(f"[selftest] ticks={t1} 末帧={f1[-1] if f1 else '(none)'}")
        return 0 if ok else 1

    rc_b1, dir_b1 = run_variant(classes, OUT / "variantB", "B1", ticks, script)
    log("5/6 比对 …")
    sha_a1, sha_a2 = sha256_file(dir_a1 / "trace.txt"), sha256_file(dir_a2 / "trace.txt")
    sha_b1 = sha256_file(dir_b1 / "trace.txt")
    det = "PASS" if (rc_a1 == 0 and rc_a2 == 0 and sha_a1 == sha_a2) else "FAIL"
    eq = "PASS" if (rc_a1 == 0 and rc_b1 == 0 and sha_a1 == sha_b1) else "FAIL"

    ticks_a1, frames_a1 = trace_stats(dir_a1 / "trace.txt")
    ticks_a2, _ = trace_stats(dir_a2 / "trace.txt")
    ticks_b1, frames_b1 = trace_stats(dir_b1 / "trace.txt")

    remap = load_remap()
    err_a = canonicalize_stderr((dir_a1 / "stderr.txt").read_text(encoding="utf-8"), remap)
    err_b = canonicalize_stderr((dir_b1 / "stderr.txt").read_text(encoding="utf-8"), remap)
    stderr_cmp = "PASS（规范化后一致）" if err_a == err_b else "FAIL（规范化后仍有差异）"

    # API 面核对
    r = sh([sys.executable, "reference/oracle/api_surface.py",
            "--game", str(orig / "a.class"), "--game", str(orig / "CMidlet.class"),
            "--classes", str(classes), "--out", str(DIFF / "api-surface.md")])
    log("api_surface: " + r.stdout.strip())
    api_missing = int(re.search(r"missing=(\d+)", r.stdout).group(1)) if r.returncode in (0, 1) else -1
    api_shim = int(re.search(r"shim=(\d+)", r.stdout).group(1)) if r.returncode in (0, 1) else -1
    api_jdk = int(re.search(r"jdk=(\d+)", r.stdout).group(1)) if r.returncode in (0, 1) else -1

    def frame_chain(frames):
        return hashlib.sha256("\n".join(frames).encode()).hexdigest()

    def lines_of(p):
        return len(p.read_text(encoding="utf-8").splitlines())

    run_ok = "PASS" if (rc_a1 == 0 and rc_a2 == 0 and rc_b1 == 0
                        and ticks_a1 > 0 and ticks_b1 > 0 and api_missing == 0) else "FAIL"

    log("6/6 写报告 …")
    write_report({
        "det": det, "eq": eq, "run_ok": run_ok, "t_ok": "PASS",
        "stderr_cmp": stderr_cmp,
        "sha_a1": sha_a1, "sha_a2": sha_a2, "sha_b1": sha_b1,
        "sha_orig_a": sha256_file(orig / "a.class"),
        "sha_orig_c": sha256_file(orig / "CMidlet.class"),
        "sha_ren_a": sha256_file(RENAME_OUT / "a.class"),
        "sha_ren_c": sha256_file(RENAME_OUT / "CMidlet.class"),
        "t_counts": {k: v["counts"] for k, v in tinfo.items()},
        "ticks_a1": ticks_a1, "ticks_a2": ticks_a2, "ticks_b1": ticks_b1,
        "lines_a1": lines_of(dir_a1 / "trace.txt"), "lines_b1": lines_of(dir_b1 / "trace.txt"),
        "frame_chain_a1": frame_chain(frames_a1), "frame_chain_b1": frame_chain(frames_b1),
        "last_frames_a1": ", ".join(f"`{f}`" for f in frames_a1[-3:]) or "(none)",
        "last_frames_b1": ", ".join(f"`{f}`" for f in frames_b1[-3:]) or "(none)",
        "stderr_lines_a": len(err_a.splitlines()), "stderr_lines_b": len(err_b.splitlines()),
        "api_shim": api_shim, "api_missing": api_missing, "api_jdk": api_jdk,
    })

    ok = det == "PASS" and eq == "PASS" and run_ok == "PASS" and stderr_cmp.startswith("PASS")
    log(f"== 结果：确定性={det} 等价={eq} 运行={run_ok} stderr={stderr_cmp} ==")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
