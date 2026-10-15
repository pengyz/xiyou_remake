#!/usr/bin/env python3
"""method_reach.py — 参考版 Java 方法触达测量（覆盖矩阵 v2 的 A 级数据源）。

对 reference/oracle/scenarios/*.txt 的每个场景，以 JDWP 调试模式运行 oracle
宿主（与 run.py --scenarios 同一 Runner 命令行：同脚本、同 preset、同 tick
预算、同 T-变换），由 JDI 客户端 tools/method_reach/MethodReach.java 在类 `a`
每个方法首位置下一次性断点，记录每个方法的**首次命中 tick**（t_hit，语义见
MethodReach.java 头注释）。断点只观测不改写；本工具再把测得运行的 FRAME sha
序列与现存 A-trace（reference/oracle/_out/A-<场景>/trace.txt，即 Rust 对拍用例
实际消费的 trace）逐 tick 对照——一致才证明"测到的触达 = 被 Rust 全帧验证的
那次执行"。

变体：B（analysis/rename-pipeline/renamed 改名类 + 同一 T-变换）——方法名即矩阵
名；A==B==C 由 `gates/cli.py scenarios` 保证，FRAME 对照再逐场景复核一次。

工作区：analysis/method-reach/（可再生，不入库；与 reference/oracle/_out 隔离，
不干扰并行的 reference/scenarios 门禁）。产物：tools/method_reach/snapshot.json
（确定性内容，入库；tools/cover_matrix.py 渲染矩阵的输入）。2026-10-15 全量重测
44 场景与前任快照逐字段一致（可复现性实证）。

相位：相位起点 = 1 ∪ 使用该场景 trace 的对拍用例窗口起点；相位 p 只记到下一相位起点
前 ⇒ 全程首次命中 = 各相位并集取最小 t_hit（full_run_hits）。

夹具场景（2026-10-15 起）：crates/*/tests/fixtures/<stem>.txt（带 `# ticks:`/`# preset:` 头）且有
同名 `<stem>.frames.txt`（tools/frames_fixture.py 产出的 Java 参照帧 = Rust 帧夹具用例的期望值）
⇒ 同样测量；FRAME 对照改与夹具帧比（快照 a_trace.source=fixture，不读 _out/）。只认已入 git 索引的
夹具（并行任务的在制夹具不进全量/--check；可 --only 点名测量）。

用法：
  python3 tools/method_reach.py                    # 全部场景（并行 --jobs，缺省 4）
  python3 tools/method_reach.py --only a b         # 只测指定场景 stem，合并入既有快照
  python3 tools/method_reach.py --rev REV --only a # git 历史版场景（脚本+预置）按现行快照同预算重测，
                                                   # 与现行快照逐方法对比（只历史/只现行/t_hit 变/FRAME 链
                                                   # 异同）→ analysis/method-reach/rev-<sha>/<场景>.json；
                                                   # 不入快照。`--rev HEAD` = 自测（应全同）
  python3 tools/method_reach.py --check            # 只校验快照与场景脚本/预置/改名类 sha 一致
退出码：0 成功；1 FRAME 对照失配或运行失败；2 基础设施错误。
"""
from __future__ import annotations

import concurrent.futures
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WORK = ROOT / "analysis/method-reach"
SCEN = ROOT / "reference/oracle/scenarios"
A_OUT = ROOT / "reference/oracle/_out"
JAR = ROOT / "original/囧囧西游-大闹天宫.jar"
RENAMED = ROOT / "analysis/rename-pipeline/renamed"
CLIENT_SRC = ROOT / "tools/method_reach/MethodReach.java"
SNAPSHOT = ROOT / "tools/method_reach/snapshot.json"

sys.path.insert(0, str(ROOT / "reference/oracle"))
sys.path.insert(0, str(ROOT / "tools"))


def sha256_bytes(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def sha256_file(p: Path) -> str:
    return sha256_bytes(p.read_bytes())


def log(msg: str) -> None:
    print(f"[method_reach] {msg}", flush=True)


def scenario_header(f: Path) -> tuple[int, Path | None]:
    ticks, preset = 150, None
    for line in f.read_text(encoding="utf-8").splitlines()[:30]:
        m = re.match(r"#\s*ticks:\s*(\d+)", line)
        if m:
            ticks = int(m.group(1))
        m = re.match(r"#\s*preset:\s*(\S+)", line)
        if m:
            preset = f.parent / m.group(1)
    return ticks, preset


def frame_shas(trace: Path) -> list[str]:
    out = []
    with trace.open(encoding="utf-8", errors="replace") as fh:
        for line in fh:
            if line.startswith("FRAME sha="):
                out.append(line[10:].split()[0])
    return out


def fixture_scenarios(tracked_only: bool = False) -> dict[str, Path]:
    """夹具场景（2026-10-15 起）：crates/*/tests/fixtures/<stem>.txt 带 `# ticks:` 头且有同名
    `<stem>.frames.txt`（tools/frames_fixture.py 产出的 Java 参照帧）。帧对照改以夹具帧为准
    （= Rust 用例实际比对的期望值），不读 _out/。stem 不得与 scenarios/ 重名。
    tracked_only：只认已入 git 索引的夹具（并行任务未入库的在制夹具不进快照/校验——
    快照与入库树一一对应；在制夹具可用 --only 显式点名测量）。"""
    tracked = None
    if tracked_only:
        r = subprocess.run(["git", "ls-files", "--", "crates"], capture_output=True, text=True, cwd=str(ROOT))
        tracked = set(r.stdout.split()) if r.returncode == 0 else None
    out = {}
    for f in sorted(ROOT.glob("crates/*/tests/fixtures/*.txt")):
        if f.name.endswith(".frames.txt") or not f.with_suffix(".frames.txt").exists():
            continue
        if not any(re.match(r"#\s*ticks:\s*\d+", l) for l in f.read_text(encoding="utf-8").splitlines()[:40]):
            continue
        if tracked is not None and str(f.relative_to(ROOT)) not in tracked:
            continue
        if (SCEN / f.name).exists() or f.stem in out:
            raise SystemExit(f"夹具场景 stem 冲突：{f.relative_to(ROOT)}")
        out[f.stem] = f
    return out


def fixture_frames(f: Path) -> list[str]:
    """`<stem>.frames.txt` 的 FRAME sha 列（逐 TICK，注释行跳过）。"""
    out = []
    for line in f.read_text(encoding="utf-8").splitlines():
        if line.strip() and not line.startswith("#"):
            out.append(line.split()[1])
    return out


def end_line(trace: Path) -> str:
    last = ""
    with trace.open(encoding="utf-8", errors="replace") as fh:
        for line in fh:
            if line.startswith("END "):
                last = line.strip()
    return last


# ---------------------------------------------------------------- 构建

def build() -> dict:
    """shim+host / T-变换 B / JDI 客户端，全部落 WORK（与 _out 隔离）。"""
    import ttransform  # reference/oracle/ttransform.py（只读复用，同一变换）

    WORK.mkdir(parents=True, exist_ok=True)
    classes = WORK / "classes"
    classes.mkdir(exist_ok=True)
    sources = sorted(str(p) for p in list((ROOT / "reference/shim/src").rglob("*.java"))
                     + list((ROOT / "reference/oracle/src").rglob("*.java")))
    r = subprocess.run(["javac", "-encoding", "UTF-8", "-nowarn", "-d", str(classes)] + sources,
                       capture_output=True, text=True)
    if r.returncode != 0:
        log("shim/host javac 失败:\n" + r.stderr[-3000:])
        sys.exit(2)
    if not (RENAMED / "a.class").exists():
        log(f"缺 {RENAMED}/a.class —— 先跑 python3 tools/rename-pipeline/run.py")
        sys.exit(2)
    variant = WORK / "variantB"
    variant.mkdir(exist_ok=True)
    for c in ("a", "CMidlet"):
        data, _ = ttransform.transform((RENAMED / f"{c}.class").read_bytes(), label=f"B:{c}")
        (variant / f"{c}.class").write_bytes(data)
    jdi = WORK / "jdi"
    jdi.mkdir(exist_ok=True)
    r = subprocess.run(["javac", "-encoding", "UTF-8", "-nowarn", "-d", str(jdi), str(CLIENT_SRC)],
                       capture_output=True, text=True)
    if r.returncode != 0:
        log("MethodReach javac 失败:\n" + r.stderr[-3000:])
        sys.exit(2)
    # ASCII 路径的 JAR 链接（只读引用 original/，避免非 ASCII 参数经 JDI 转发的编码风险）
    jar_link = WORK / "game.jar"
    if jar_link.is_symlink() or jar_link.exists():
        jar_link.unlink()
    jar_link.symlink_to(JAR)
    return {"classes": classes, "variant": variant, "jdi": jdi, "jar": jar_link}


# ---------------------------------------------------------------- 测量

def a_trace_budget(stem: str) -> int | None:
    """A-trace 的 tick 预算（首行 `PRE max=N`）。Rust 全帧用例窗口 = A-trace 帧数，
    而部分 A-trace 生成时预算 ≠ 场景头（route_gen 系 +100 拍：floor10/10b/17）——
    测量按 A-trace 预算跑，才覆盖用例实际比对的全部帧。"""
    p = A_OUT / f"A-{stem}" / "trace.txt"
    if not p.exists():
        return None
    with p.open(encoding="utf-8", errors="replace") as fh:
        m = re.match(r"PRE max=(\d+)", fh.readline())
    return int(m.group(1)) if m else None


def measure(stem: str, env: dict, arms: list[int], script: Path | None = None,
            a_trace: Path | None = None, run_tag: str = "run", ticks: int | None = None,
            expect: Path | None = None) -> dict:
    """测一个场景。缺省 = 现行 scenarios/<stem>.txt + 其 A-trace；`script`/`a_trace` 显式
    给出时用于测历史版本（--rev：git show 导出到 WORK 的旧脚本/预置，不入快照）；
    `ticks` 显式给出时覆盖 tick 预算（--rev 取现行快照同预算，保证对比同窗）。
    `expect` = 夹具场景的 `<stem>.frames.txt`：帧对照以夹具帧为准（a_trace.source=fixture）。"""
    f = script if script is not None else SCEN / f"{stem}.txt"
    ticks_header, preset = scenario_header(f)
    if a_trace is None and script is None:
        a_trace = A_OUT / f"A-{stem}" / "trace.txt"
    if ticks is None:
        ticks = (a_trace_budget(stem) if script is None else None) or ticks_header
    run_dir = WORK / run_tag / stem
    run_dir.mkdir(parents=True, exist_ok=True)
    for p in run_dir.iterdir():
        p.unlink()
    target = [
        "-Djava.awt.headless=true",
        "-cp", f"{env['classes']}:{env['variant']}:{env['jar']}",
        f"-Doracle.out={run_dir}",
        f"-Doracle.max={ticks}",
        f"-Doracle.script={f}",
        f"-Doracle.label=R-{stem}",
        f"-Doracle.watchdog={max(600000, ticks * 300)}",
        "-Doracle.frames=off",
        "-Doracle.dumpStride=8",
    ]
    if preset is not None:
        target.append(f"-Doracle.preset={preset}")
    target.append("oracle.host.Runner")
    reach_txt = run_dir / "reach.txt"
    cmd = ["java", "-cp", str(env["jdi"]), "MethodReach", "--out", str(reach_txt),
           "--arm-at", ",".join(str(a) for a in arms), "--"] + target
    r = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT))
    if r.returncode != 0 or not reach_txt.exists():
        return {"error": f"MethodReach rc={r.returncode}: {r.stderr[-800:]}"}

    phases: dict[str, dict[str, int]] = {}
    cur = None
    exit_code = None
    for line in reach_txt.read_text(encoding="utf-8").splitlines():
        if line.startswith("PHASE "):
            cur = phases.setdefault(line.split()[1], {})
        elif line.startswith("HIT "):
            _, t, key = line.split(" ", 2)
            cur[key] = int(t)
        elif line.startswith("EXIT "):
            exit_code = int(line.split()[1])
    trace = run_dir / "trace.txt"
    mine = frame_shas(trace)
    res = {
        "ticks_header": ticks_header,
        "ticks_run": ticks,
        "preset": preset.name if preset else None,
        "script_sha256": sha256_file(f),
        "preset_sha256": sha256_file(preset) if preset else None,
        "exit": exit_code,
        "end": end_line(trace),
        "frames": len(mine),
        "frame_chain": sha256_bytes("\n".join(mine).encode()),
        "phases": {k: dict(sorted(v.items())) for k, v in sorted(phases.items(), key=lambda kv: int(kv[0]))},
    }
    a_trace_ok = a_trace is not None and a_trace.exists()
    if expect is not None or a_trace_ok:
        theirs = fixture_frames(expect) if expect is not None else frame_shas(a_trace)
        n = min(len(mine), len(theirs))
        first_diff = next((i + 1 for i in range(n) if mine[i] != theirs[i]), None)
        res["a_trace"] = {
            "path": str((expect if expect is not None else a_trace).relative_to(ROOT)),
            "frames": len(theirs),
            "frame_chain": sha256_bytes("\n".join(theirs).encode()),
            # 一致 = 前缀全同且长度相等；prefix = 较短一侧全同（A-trace 预算与脚本头不一致时）
            "match": "equal" if (first_diff is None and len(mine) == len(theirs))
                     else ("prefix" if first_diff is None else f"diverge@T{first_diff}"),
        }
        if expect is not None:
            res["a_trace"]["source"] = "fixture"
            res["script"] = str(f.relative_to(ROOT))
    else:
        res["a_trace"] = None
    # 产物瘦身：trace 只用于 FRAME 对照，测完即删（场景 trace 动辄百 MB）
    trace.unlink()
    return res


def arms_for(stem: str) -> list[int]:
    """相位起点 = 1 ∪ 使用该场景 trace 的对拍用例窗口起点（cover_matrix.TESTS 机械派生）。"""
    import cover_matrix
    starts = {1}
    for t in cover_matrix.collect_tests():
        if t.get("trace_stem") == stem and t.get("window"):
            starts.add(t["window"][0])
    return sorted(starts)


def full_run_hits(s: dict) -> dict[str, int]:
    """全程首次命中 = 各相位并集取最小 t_hit（相位 p 只记到下一相位起点前，见 MethodReach.java）。"""
    first: dict[str, int] = {}
    for ph in s.get("phases", {}).values():
        for key, th in ph.items():
            if key not in first or th < first[key]:
                first[key] = th
    return first


def renamed_meta() -> dict:
    return {c: sha256_file(RENAMED / f"{c}.class") for c in ("a", "CMidlet")}


def main() -> int:
    args = sys.argv[1:]
    jobs = int(args[args.index("--jobs") + 1]) if "--jobs" in args else 4
    fx_all = fixture_scenarios()
    fx = fixture_scenarios(tracked_only=True)
    untracked = sorted(set(fx_all) - set(fx))
    if untracked:
        log(f"未入库夹具场景（不进全量/校验，可 --only 点名测量）：{', '.join(untracked)}")
    stems_all = sorted([p.stem for p in SCEN.glob("*.txt")] + list(fx))
    snap = json.loads(SNAPSHOT.read_text(encoding="utf-8")) if SNAPSHOT.exists() else {}

    def script_of(stem: str) -> Path:
        return fx_all.get(stem, SCEN / f"{stem}.txt")

    if "--check" in args:
        bad = []
        if snap.get("meta", {}).get("renamed_sha256") != renamed_meta():
            bad.append("改名类 sha 与快照不符（需重测全部场景）")
        for stem in stems_all:
            s = snap.get("scenarios", {}).get(stem)
            if s is None:
                bad.append(f"{stem}: 快照缺失")
                continue
            ticks, preset = scenario_header(script_of(stem))
            if s["script_sha256"] != sha256_file(script_of(stem)):
                bad.append(f"{stem}: 场景脚本已变")
            if (s["preset_sha256"] or None) != (sha256_file(preset) if preset else None):
                bad.append(f"{stem}: 预置已变")
            if stem in fx:
                exp = fixture_frames(fx[stem].with_suffix(".frames.txt"))
                if (s.get("a_trace") or {}).get("frame_chain") != sha256_bytes("\n".join(exp).encode()):
                    bad.append(f"{stem}: 夹具帧已变")
        for b in bad:
            log("STALE " + b)
        log("快照一致" if not bad else f"快照陈旧 {len(bad)} 项")
        return 0 if not bad else 1

    stems = stems_all
    if "--only" in args:
        i = args.index("--only") + 1
        stems = []
        while i < len(args) and not args[i].startswith("--"):
            stems.append(args[i])
            i += 1

    if "--rev" in args:
        # 历史场景版本测量（取证用，不入快照）：git show <rev>:场景脚本/预置 → WORK/rev-<rev>/，
        # 测完与现行快照逐方法对比（只在历史版/只在现行版触达、t_hit 变化、FRAME 链异同）
        rev = args[args.index("--rev") + 1]
        sha = subprocess.run(["git", "rev-parse", "--short", f"{rev}^{{commit}}"], capture_output=True,
                             text=True, cwd=str(ROOT))
        if sha.returncode != 0:
            log(f"无法解析 rev {rev!r}")
            return 2
        tag = sha.stdout.strip()
        env = build()
        out_dir = WORK / f"rev-{tag}"
        out_dir.mkdir(parents=True, exist_ok=True)
        cur = snap.get("scenarios", {})
        measured, failed = 0, 0
        for stem in stems:
            src = f"reference/oracle/scenarios/{stem}.txt"
            text = subprocess.run(["git", "show", f"{tag}:{src}"], capture_output=True,
                                  text=True, encoding="utf-8", cwd=str(ROOT))
            if text.returncode != 0:
                log(f"  {tag}:{src} 不存在——跳过")
                continue
            sf = out_dir / f"{stem}.txt"
            sf.write_text(text.stdout, encoding="utf-8")
            _, preset = scenario_header(sf)
            if preset is not None:
                rel = preset.relative_to(out_dir).as_posix()
                pr = subprocess.run(["git", "show", f"{tag}:reference/oracle/scenarios/{rel}"],
                                    capture_output=True, text=True, encoding="utf-8", cwd=str(ROOT))
                if pr.returncode != 0:
                    log(f"  {tag}:{stem} 预置 {rel} 取不到——跳过")
                    failed += 1
                    continue
                preset.parent.mkdir(parents=True, exist_ok=True)
                preset.write_text(pr.stdout, encoding="utf-8")
            res = measure(stem, env, [1], script=sf, run_tag=f"rev-{tag}/run",
                          ticks=cur.get(stem, {}).get("ticks_run"))
            if "error" in res:
                log(f"  {tag}:{stem}: ERROR {res['error']}")
                failed += 1
                continue
            measured += 1
            mine = full_run_hits(res)
            now = full_run_hits(cur.get(stem, {}))
            diff = {
                "rev": tag,
                "script_same_as_worktree": res["script_sha256"] == (sha256_file(SCEN / f"{stem}.txt")
                                                                    if (SCEN / f"{stem}.txt").exists() else None),
                "frame_chain_same_as_snapshot": (res["frame_chain"] == cur[stem]["frame_chain"]
                                                 if stem in cur else None),
                "only_rev": sorted(k for k in mine if k not in now),
                "only_now": sorted(k for k in now if k not in mine),
                "t_hit_changed": {k: [mine[k], now[k]] for k in sorted(mine) if k in now and mine[k] != now[k]},
            }
            res["vs_snapshot"] = diff
            (out_dir / f"{stem}.json").write_text(json.dumps(res, ensure_ascii=False, indent=1) + "\n",
                                                  encoding="utf-8")
            log(f"  {tag}:{stem}: frames={res['frames']}（现行 {cur.get(stem, {}).get('frames', '—')}）"
                f" hits={len(mine)}（现行 {len(now)}）只历史 {len(diff['only_rev'])} / 只现行 "
                f"{len(diff['only_now'])} / t_hit 变 {len(diff['t_hit_changed'])}"
                f" → {(out_dir / (stem + '.json')).relative_to(ROOT)}")
        if measured == 0:
            log("没有测成任何历史场景")
            return 2
        return 1 if failed else 0

    env = build()
    log(f"测量 {len(stems)} 场景（jobs={jobs}）…")
    results: dict[str, dict] = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as ex:
        futs = {ex.submit(measure, s, env, arms_for(s),
                          script=fx_all.get(s),
                          expect=fx_all[s].with_suffix(".frames.txt") if s in fx_all else None): s
                for s in stems}
        for fu in concurrent.futures.as_completed(futs):
            s = futs[fu]
            res = fu.result()
            results[s] = res
            if "error" in res:
                log(f"  {s}: ERROR {res['error']}")
            else:
                at = res["a_trace"]["match"] if res["a_trace"] else "无 A-trace"
                n = len(full_run_hits(res))
                log(f"  {s}: frames={res['frames']} {res['end']} hits={n} A-trace={at}")

    scen = dict(snap.get("scenarios", {}))
    if snap.get("meta", {}).get("renamed_sha256") != renamed_meta():
        scen = {}  # 改名类变了：旧数据作废
    scen.update(results)
    out = {
        "meta": {
            "tool": "tools/method_reach.py + tools/method_reach/MethodReach.java（JDI 一次性断点）",
            "variant": "B = analysis/rename-pipeline/renamed + reference/oracle/ttransform.py",
            "renamed_sha256": renamed_meta(),
            "t_hit": "Canvas.hooks.tick + 1 = 首个可能反映该次执行的 TICK 记录号",
        },
        "scenarios": dict(sorted(scen.items())),
    }
    SNAPSHOT.write_text(json.dumps(out, ensure_ascii=False, indent=1, sort_keys=False) + "\n",
                        encoding="utf-8")
    log(f"快照 → {SNAPSHOT.relative_to(ROOT)}")
    failed = [s for s, r in results.items() if "error" in r or
              (r.get("a_trace") and r["a_trace"]["match"].startswith("diverge"))]
    if failed:
        log(f"失配/失败：{failed}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
