#!/usr/bin/env python3
"""frames_fixture.py — 帧夹具（fixture-frames）管线：Java 参照帧嵌入测试夹具，Rust 用例运行期不读 _out/。

背景：provision_tests 只认 `reference/oracle/scenarios/<stem>.txt` ↔ `_out/A-<stem>` 的 trace 引用
（docs/knowledge/gotcha_provision-trace-labels-a-stem-only.md）；夹具场景（场景脚本放在
crates/*/tests/fixtures/）无从供给 ⇒ 期望帧须嵌入夹具。先例：crates/game-core/tests/slot_equip_scene.rs。

夹具格式（`<stem>.frames.txt`）：注释头（`# 源 trace：…/A-<stem>/trace.txt sha256=…` 等）+ 逐 TICK
`tick FRAME-sha [INPUT]`（INPUT = trace 的 `INPUT press(k)`/`release(k)`）。cover_matrix.py 以
注释头的 `A-<stem>` 认领场景，并把夹具帧链 sha256（"\\n".join(sha)）与 method_reach 快照的
frame_chain 逐字节比对——一致才把该用例的窗口计入验证。

子命令：
  extract TRACE --stem STEM --out OUT [--script SCRIPT]
      由既有 A 变体 trace 抽取夹具（如 reference/oracle/_out/A-preset-floor5/trace.txt）。
  gen SCRIPT [--frames all|off]
      夹具场景（脚本带 `# ticks:`/`# preset:` 头）在隔离工作区 analysis/frames-fixture/ 跑 A/B/C
      三变体（同 run.py --scenarios：shim+host、T-变换、frames/dumpStride=8，headless），
      三方 trace sha256 一致才由 A 抽取 `<脚本>.frames.txt`。不触碰 reference/oracle/_out/。
      `--frames all` 额外保留 A 的逐帧 PNG（取证/选格用，可再生，不入库）。
  check FRAMES [--script SCRIPT]
      夹具自洽：TICK 连续、INPUT 列 = 脚本事件。
  preset KEY=VAL …
      生成 RMS 预置文本（MOT_IF 槽0 + MOT_L0 + SKY_WAR；字段布局 = a.java loadSlotTable
      8950-8996 / loadFloorSave 9244-9319 / m_000 case 21 4450-4479），打印到 stdout。
      键：floor x y facing hp atk def yk bk rk gold weapon armor items=型:次,型:次 flags=i,j
"""
from __future__ import annotations

import hashlib
import re
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WORK = ROOT / "analysis/frames-fixture"
JAR = ROOT / "original/囧囧西游-大闹天宫.jar"
RENAMED = ROOT / "analysis/rename-pipeline/renamed"
VARIANT_C = ROOT / "analysis/build/deobf"
CLASSES = ("a", "CMidlet")

sys.path.insert(0, str(ROOT / "reference/oracle"))


def log(msg: str) -> None:
    print(f"[frames_fixture] {msg}", flush=True)


def sha256_file(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def scenario_header(f: Path) -> tuple[int, Path | None]:
    ticks, preset = 150, None
    for line in f.read_text(encoding="utf-8").splitlines()[:40]:
        m = re.match(r"#\s*ticks:\s*(\d+)", line)
        if m:
            ticks = int(m.group(1))
        m = re.match(r"#\s*preset:\s*(\S+)", line)
        if m:
            preset = f.parent / m.group(1)
    return ticks, preset


def script_events(f: Path) -> list[str]:
    out = []
    for line in f.read_text(encoding="utf-8").splitlines():
        s = line.strip()
        if not s or s.startswith("#"):
            continue
        t, k, kind = s.split()[:3]
        out.append(f"{int(t)} {kind}({int(k)})")
    return out


def trace_frames(trace: Path) -> list[tuple[int, str, str | None]]:
    """trace → [(tick, FRAME sha, INPUT)]（INPUT 行在 TICK 块内、FRAME 行之后）。"""
    recs: list[list] = []
    with trace.open(encoding="utf-8", errors="replace") as fh:
        for line in fh:
            if line.startswith("TICK "):
                recs.append([int(line.split()[1]), None, None])
            elif line.startswith("FRAME sha=") and recs:
                recs[-1][1] = line[10:].split()[0]
            elif line.startswith("INPUT ") and recs:
                recs[-1][2] = line[6:].strip()
    bad = [r[0] for r in recs if r[1] is None]
    if bad:
        raise SystemExit(f"trace {trace} 有 {len(bad)} 拍缺 FRAME（首个 T{bad[0]}）")
    return [tuple(r) for r in recs]


def word_of(variants: str) -> str:
    """对拍集合的口径词（单方/双方/三方/多方）——头注记用（2026-10-16）。"""
    return {1: "单方", 2: "双方", 3: "三方"}.get(len(variants.split("/")), "多方")


def frame_chain(shas: list[str]) -> str:
    """与 tools/method_reach.py 快照 frame_chain 同式。"""
    return hashlib.sha256("\n".join(shas).encode()).hexdigest()


def write_fixture(recs, out: Path, stem: str, trace: Path, note: str, variants: str = "A") -> None:
    """注释头第 2 行 = 源 trace 实际路径（含 `A-<stem>/trace.txt`——cover_matrix 据此认领场景）。

    头行变体口径随对拍集合（单/双/三方）——2026-10-16 起不再硬编码"A 变体"单方言。
    """
    shas = [s for _, s, _ in recs]
    word = {1: "单方", 2: "双方", 3: "三方"}.get(len(variants.split("/")), "多方")
    lines = [
        f"# {stem} Java 参照帧：{word}（{variants}）trace 逐 TICK 抽取（列 = tick FRAME-sha [INPUT]）。",
        f"# 源 trace：{trace.relative_to(ROOT)} sha256={sha256_file(trace)}",
        f"# 帧数 {len(recs)}；帧链 sha256={frame_chain(shas)}（= tools/method_reach 快照 frame_chain 口径）",
        f"# {note}",
    ]
    for t, s, i in recs:
        lines.append(f"{t:04d} {s}" + (f" {i}" if i else ""))
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    log(f"夹具 → {out.relative_to(ROOT)}（{len(recs)} 帧，链 {frame_chain(shas)[:16]}…）")


def read_fixture(p: Path) -> tuple[dict, list[tuple[int, str, str | None]]]:
    head, recs = {}, []
    for line in p.read_text(encoding="utf-8").splitlines():
        if line.startswith("#"):
            m = re.search(r"A-([\w-]+)/trace\.txt", line)
            if m:
                head["stem"] = m.group(1)
            continue
        if not line.strip():
            continue
        parts = line.split()
        recs.append((int(parts[0]), parts[1], parts[2] if len(parts) > 2 else None))
    return head, recs


def check(frames: Path, script: Path | None) -> list[str]:
    head, recs = read_fixture(frames)
    bad = []
    if "stem" not in head:
        bad.append("注释头缺 `A-<stem>/trace.txt` 源声明")
    if [t for t, _, _ in recs] != list(range(1, len(recs) + 1)):
        bad.append("TICK 不连续")
    if script is not None:
        ev = script_events(script)
        got = [f"{t} {i}" for t, _, i in recs if i]
        if ev != got:
            bad.append(f"INPUT 列与脚本事件不一致（脚本 {len(ev)} / 夹具 {len(got)}）")
        ticks, _ = scenario_header(script)
        if ticks != len(recs):
            bad.append(f"帧数 {len(recs)} ≠ 脚本 # ticks: {ticks}")
    return bad


# ---------------------------------------------------------------- gen：隔离工作区三变体

def build() -> dict:
    import ttransform  # reference/oracle/ttransform.py（只读复用，同一 T-变换）

    WORK.mkdir(parents=True, exist_ok=True)
    classes = WORK / "classes"
    if classes.exists():
        shutil.rmtree(classes)
    classes.mkdir()
    sources = sorted(str(p) for p in list((ROOT / "reference/shim/src").rglob("*.java"))
                     + list((ROOT / "reference/oracle/src").rglob("*.java")))
    r = subprocess.run(["javac", "-encoding", "UTF-8", "-nowarn", "-d", str(classes)] + sources,
                       capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit("shim/host javac 失败:\n" + r.stderr[-3000:])
    srcs = {"A": None, "B": RENAMED}
    if (VARIANT_C / "a.class").exists():
        srcs["C"] = VARIANT_C
    else:
        log(f"变体 C 缺（{VARIANT_C}）——只做 A==B")
    variants = {}
    with zipfile.ZipFile(JAR) as z:
        for v, src in srcs.items():
            d = WORK / f"variant{v}"
            d.mkdir(exist_ok=True)
            for c in CLASSES:
                data = z.read(f"{c}.class") if src is None else (src / f"{c}.class").read_bytes()
                out, _ = ttransform.transform(data, label=f"{v}:{c}")
                (d / f"{c}.class").write_bytes(out)
            variants[v] = d
    return {"classes": classes, "variants": variants}


def gen(script: Path, frames: str) -> int:
    script = script.resolve()
    stem = script.stem
    ticks, preset = scenario_header(script)
    if preset is not None and not preset.exists():
        raise SystemExit(f"预置缺失：{preset}")
    env = build()
    procs = {}
    for v, vdir in env["variants"].items():
        out = WORK / "runs" / f"{v}-{stem}"
        if out.exists():
            shutil.rmtree(out)
        out.mkdir(parents=True)
        cmd = ["java", "-Djava.awt.headless=true", "-cp", f"{env['classes']}:{vdir}:{JAR}",
               f"-Doracle.out={out}", f"-Doracle.max={ticks}", f"-Doracle.script={script}",
               f"-Doracle.label={v}-{stem}", f"-Doracle.watchdog={max(300000, ticks * 75)}",
               f"-Doracle.frames={frames if v == 'A' else 'off'}", "-Doracle.dumpStride=8"]
        if preset is not None:
            cmd.append(f"-Doracle.preset={preset}")
        cmd.append("oracle.host.Runner")
        procs[v] = (subprocess.Popen(cmd, cwd=str(ROOT), stdout=(out / "stdout.txt").open("w"),
                                     stderr=(out / "stderr.txt").open("w")), out)
    shas = {}
    for v, (p, out) in procs.items():
        rc = p.wait()
        tr = out / "trace.txt"
        if rc != 0 or not tr.exists():
            log(f"变体 {v} rc={rc}：{(out / 'stderr.txt').read_text()[-600:]}")
            return 1
        shas[v] = sha256_file(tr)
    log("三方 trace sha256：" + "，".join(f"{v}={s[:12]}…" for v, s in shas.items()))
    if len(set(shas.values())) != 1:
        log("A/B/C 不一致——不写夹具")
        return 1
    a_trace = procs["A"][1] / "trace.txt"
    recs = trace_frames(a_trace)
    variants = "/".join(shas)
    note = (f"{word_of(variants)}（{variants}）trace 逐字节一致；frames=off dumpStride=8；"
            f"再生：python3 tools/frames_fixture.py gen {script.relative_to(ROOT)}")
    write_fixture(recs, script.with_suffix(".frames.txt"), stem, a_trace, note, variants)
    bad = check(script.with_suffix(".frames.txt"), script)
    for b in bad:
        log("✗ " + b)
    if frames == "all":
        log(f"A 帧 PNG：{procs['A'][1].relative_to(ROOT)}/frame-NNNN.png")
    return 1 if bad else 0


# ---------------------------------------------------------------- preset：RMS 预置构造

def preset_text(kv: dict) -> str:
    g = lambda k, d: int(kv.get(k, d))  # noqa: E731
    floor = g("floor", 1)
    hp, atk, de = g("hp", 500), g("atk", 20000), g("def", 30)
    weapon, armor = g("weapon", 0), g("armor", 0)
    items = [tuple(int(v) for v in p.split(":")) for p in kv.get("items", "").split(",") if p]
    flags = {int(v) for v in kv.get("flags", "").split(",") if v}

    def be(n: int, w: int) -> str:
        return (n & ((1 << (8 * w)) - 1)).to_bytes(w, "big").hex()

    # MOT_IF（loadSlotTable a.java:8950-8996）：槽0 有效 + 层/武器/甲 + 7×int（hp/atk/def/…）；槽1-5 无效
    mot_if = "01" + be(floor, 1) + be(weapon, 1) + be(armor, 1) + be(hp, 4) + be(atk, 4) + be(de, 4) \
        + "00" * 16 + "00" * 5
    # MOT_L0（loadFloorSave a.java:9244-9319；crates/game-core/src/save.rs parse_floor_save 同序）
    body = (be(g("difficulty", 0), 1) + be(floor, 1) + be(g("min", 1), 1) + be(g("max", floor), 1)
            + be(g("facing", 0), 1) + be(weapon, 1) + be(armor, 1) + be(g("x", 0), 2) + be(g("y", 0), 2)
            + be(hp, 4) + be(atk, 4) + be(de, 4) + be(g("yk", 1), 2) + be(g("bk", 0), 2) + be(g("rk", 0), 2)
            + be(g("gold", 100), 4) + be(g("alchemy", 0), 2) + be(0, 2))
    body += "".join("01" if i in flags else "00" for i in range(128))
    body += be(len(items), 1) + "".join(be(t, 1) + be(u, 1) for t, u in items)
    body += "0000" * 56
    # SKY_WAR（m_000 case 21 a.java:4450-4479 读 / saveGlobalProgress 8914-8948 写：bool×4 + 计数 7 +
    # 7×int + 3×int）：逐字沿用既有 preset（scenarios/preset-floor41.preset）的选项/全局旗标，
    # 末 int 低 16 位 = `sky`（缺省沿用 0x23）
    sky = SKY_WAR_BASE[:-4] + be(g("sky", 0x23), 2)
    return f"MOT_IF@1={mot_if}\nMOT_L0@1={body}\nSKY_WAR@1={sky}\n"


SKY_WAR_BASE = "000100000700000004000000020000000200000002000000020000000200000002000000330000000100000023"


def main() -> int:
    a = sys.argv[1:]
    if not a:
        print(__doc__)
        return 2
    cmd = a[0]

    def opt(name, default=None):
        return a[a.index(name) + 1] if name in a else default

    if cmd == "extract":
        trace = Path(a[1]).resolve()
        recs = trace_frames(trace)
        out = Path(opt("--out")).resolve()
        stem = opt("--stem")
        note = (f"既有 A-trace 抽取（不重跑 Java）；再生：python3 tools/frames_fixture.py extract "
                f"reference/oracle/_out/A-{stem}/trace.txt --stem {stem} --out {out.relative_to(ROOT)}")
        write_fixture(recs, out, stem, trace, note)
        sc = opt("--script")
        bad = check(out, Path(sc) if sc else None)
        for b in bad:
            log("✗ " + b)
        return 1 if bad else 0
    if cmd == "gen":
        return gen(Path(a[1]), opt("--frames", "off"))
    if cmd == "check":
        sc = opt("--script")
        bad = check(Path(a[1]), Path(sc) if sc else None)
        log("夹具自洽" if not bad else "；".join(bad))
        return 1 if bad else 0
    if cmd == "preset":
        kv = dict(x.split("=", 1) for x in a[1:])
        sys.stdout.write(preset_text(kv))
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main())
