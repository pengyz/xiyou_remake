#!/usr/bin/env python3
"""gates/cli.py — 本项目门禁单一真相源（AGENTS.md §5）。

设计原则（移植自 PAL3Decomp 的 pipeline/cli.py 教训）：
- SKIP / TOLERATED 必须显式打印，绝不静默通过；
- 基线 ratchet 只升不降，基线文件是 data/status/baselines/baseline.json；
- 门禁失败先按 AGENTS.md §4 定位层，禁止调阈值掩盖。

用法：
  python3 gates/cli.py status      # 只读预检（动手前必跑）
  python3 gates/cli.py check       # 总闸（pre-commit 调用）
  python3 gates/cli.py original    # JAR 完整性
  python3 gates/cli.py reference-seed  # D3 锚点 reference/seed/a.java 完整性
  python3 gates/cli.py reference   # 参考版回归（P1 后启用）
  python3 gates/cli.py trace       # 差分 L1-L3（P3 后启用）
  python3 gates/cli.py visual      # 视觉 L4（P3 后启用）
  python3 gates/cli.py all         # 全量
  python3 gates/cli.py ratchet         # 测试计数 ratchet 校验（只许增不许减）
  python3 gates/cli.py ratchet-update  # 重建测试计数基线（commit 须写明原因）
"""
import hashlib
import json
import re
import subprocess
import sys
import zipfile
from pathlib import Path

# Windows 适配（2026-10-14）：中文控制台 GBK 编码无法输出 ✓/✗ 门禁符号
# （U+2713/2717 实测 UnicodeEncodeError 崩在 report()），统一 stdout/stderr
# 为 UTF-8；管道/文件/CI 消费端均按 UTF-8 解码，与工具链其余部分同口径。
for _s in (sys.stdout, sys.stderr):
    try:
        _s.reconfigure(encoding="utf-8", errors="replace")
    except (AttributeError, ValueError):
        pass  # 非标准流（重定向包装等）保持原状，不该让门禁因打印而崩

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "data/status/baselines/baseline.json"
PASS, FAIL, SKIP, TOLERATE = "PASS", "FAIL", "SKIP", "TOLERATED"
VALID_TYPES = {"architecture", "gotcha", "pattern", "debug", "decision", "reference"}


def report(name, status, detail=""):
    tag = {"PASS": "✓ PASS", "FAIL": "✗ FAIL", "SKIP": "- SKIP", "TOLERATED": "~ TOLERATED"}[status]
    print(f"[{tag}] {name}" + (f" — {detail}" if detail else ""))
    return status


# ---------- 基础设施 ----------

def load_baseline():
    if not BASELINE.exists():
        return {}
    return json.loads(BASELINE.read_text(encoding="utf-8"))


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


# ---------- 各门禁 ----------

def gate_original():
    """original/ 完整性：sha256 对照 SHA256SUMS（红线：任何字节修改 = 严重违规）。"""
    sums = ROOT / "original/SHA256SUMS"
    if not sums.exists():
        return report("original-integrity", FAIL, "original/SHA256SUMS 缺失")
    bad = []
    for line in sums.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        digest, name = line.split(maxsplit=1)
        p = ROOT / "original" / name.strip().lstrip("*")
        if not p.exists():
            bad.append(f"{name} 缺失")
        elif sha256(p) != digest:
            bad.append(f"{name} sha256 不匹配")
    return report("original-integrity", FAIL if bad else PASS,
                  "; ".join(bad) if bad else f"{len(sums.read_text(encoding='utf-8').splitlines())} 个归档校验通过")


def gate_assets_manifest():
    """资产清单 ratchet：JAR 内容必须与基线一致（138 条目/2 class/55 maplv/55 sprite）。"""
    base = load_baseline().get("assets")
    jar = ROOT / "original/囧囧西游-大闹天宫.jar"
    if not base:
        return report("assets-manifest", FAIL, "基线缺失：data/status/baselines/baseline.json")
    with zipfile.ZipFile(jar) as z:
        names = z.namelist()
    cur = {
        "entries": len(names),
        "classes": sum(1 for n in names if n.endswith(".class")),
        "maplv": sum(1 for n in names if re.fullmatch(r"maplv\d+", n)),
        "sprite": sum(1 for n in names if re.fullmatch(r"sprite\d+", n)),
    }
    drift = {k: (base[k], cur[k]) for k in cur if base.get(k) != cur[k]}
    return report("assets-manifest", FAIL if drift else PASS,
                  f"回退 {drift}" if drift else f"与基线一致 {cur}")


def gate_knowledge_format():
    """知识库格式：frontmatter 必填字段 + type 合法 + 文件名前缀 = type。"""
    kdir = ROOT / "docs/knowledge"
    bad = []
    for p in sorted(kdir.glob("*.md")):
        if p.name in ("README.md", "MEMORY.md"):
            continue
        text = p.read_text(encoding="utf-8")
        m = re.match(r"^---\n(.*?)\n---\n", text, re.S)
        if not m:
            bad.append(f"{p.name}: 缺 frontmatter")
            continue
        fm = dict(re.findall(r"^(\w+):\s*(.+)$", m.group(1), re.M))
        for key in ("name", "description", "type", "created", "sources"):
            if not fm.get(key):
                bad.append(f"{p.name}: 缺 {key}")
        t = fm.get("type", "")
        if t and t not in VALID_TYPES:
            bad.append(f"{p.name}: type 非法 {t}")
        if t and not p.name.startswith(t + "_"):
            bad.append(f"{p.name}: 文件名前缀应为 {t}_")
    return report("knowledge-format", FAIL if bad else PASS,
                  "; ".join(bad[:5]) if bad else f"{len(list(kdir.glob('*.md')))-2} 条目合规")


def _tracked_md_files():
    """全部 git 追踪的 .md 文件（覆盖面以仓库实际内容为准，不靠手写目录清单维护）。"""
    r = subprocess.run(["git", "ls-files", "*.md"], cwd=ROOT, capture_output=True,
                       text=True, encoding="utf-8", errors="replace")
    if r.returncode != 0:
        # git 不可用时退化为静态目录清单，保证门禁仍可运行
        files = [ROOT / "AGENTS.md", ROOT / "CLAUDE.md", ROOT / "README.md"]
        for sub in ("docs", "prompts", "gates", "data", "original", "reference"):
            files += [p for p in (ROOT / sub).rglob("*.md")]
        return files
    return [ROOT / line for line in r.stdout.splitlines() if line.strip()]


def gate_docs_links():
    """文档链接：全部 git 追踪 .md 文件的相对链接必须指向存在的文件（覆盖整个仓库，不限 docs/prompts）。"""
    missing = []
    md_files = _tracked_md_files()
    for p in md_files:
        if not p.exists():
            continue
        for target in re.findall(r"\[[^\]]*\]\(([^)]+)\)", p.read_text(encoding="utf-8")):
            if re.match(r"^(https?://|mailto:|#)", target):
                continue
            target = target.split("#")[0].strip()
            if not target:
                continue
            if not (p.parent / target).resolve().exists():
                missing.append(f"{p.relative_to(ROOT)} → {target}")
    return report("docs-links", FAIL if missing else PASS,
                  "; ".join(missing[:5]) if missing else "全部相对链接有效")


def gate_reference_seed_integrity():
    """D3 锚点完整性：reference/seed/a.java 行数+sha256 必须对照基线不变（AGENTS.md §2 reference/ 红线）。

    seed/a.java 是全部差分验证的不可变投影基准（历史 CFR 反编译产物，2026-10-10
    引擎已切换 Vineflower，seed 作为冻结件保留；工作参考版是 reference/src/deobf/），本身不受
    original-integrity（校验 original/ 下 jar）覆盖，之前无任何门禁保护，
    可被意外改动而不触发任何 FAIL。本门禁补上这个缺口。
    """
    seed = ROOT / "reference/seed/a.java"
    base = load_baseline().get("decomp")
    if not seed.exists():
        return report("reference-seed-integrity", FAIL, "reference/seed/a.java 缺失（D3 锚点红线）")
    if not base or not base.get("seed_a_java_sha256"):
        return report("reference-seed-integrity", FAIL,
                      "基线缺失 decomp.seed_a_java_sha256：data/status/baselines/baseline.json")
    cur_sha = sha256(seed)
    cur_lines = len(seed.read_text(encoding="utf-8").splitlines())
    bad = []
    if cur_sha != base["seed_a_java_sha256"]:
        bad.append(f"sha256 不匹配 期望={base['seed_a_java_sha256'][:12]}… 实际={cur_sha[:12]}…")
    if base.get("seed_a_java_lines") and cur_lines != base["seed_a_java_lines"]:
        bad.append(f"行数不匹配 期望={base['seed_a_java_lines']} 实际={cur_lines}")
    return report("reference-seed-integrity", FAIL if bad else PASS,
                  "; ".join(bad) if bad else f"{cur_lines} 行，sha256 对照基线通过")


def gate_naming_ledger():
    """命名台账：每条 rename 必须有 evidence/confidence（AGENTS.md §3）。"""
    ledger = ROOT / "data/naming/ledger.jsonl"
    if not ledger.exists():
        return report("naming-ledger", SKIP, "台账尚不存在（P1 启动后启用）")
    bad = []
    for i, line in enumerate(ledger.read_text(encoding="utf-8").splitlines(), 1):
        line = line.strip()
        if not line:
            continue
        try:
            rec = json.loads(line)
        except json.JSONDecodeError as e:
            bad.append(f"行{i}: JSON 无效 {e}")
            continue
        for key in ("id", "old", "new", "evidence", "confidence"):
            if not rec.get(key) and rec.get(key) != 0:
                bad.append(f"行{i}: 缺 {key}")
        ev = str(rec.get("evidence", ""))
        if ev and not re.search(r"(reference/|original/|analysis/|0x[0-9a-fA-F]+|:\d+)", ev):
            bad.append(f"行{i}: evidence 缺可复现指针")
    return report("naming-ledger", FAIL if bad else PASS,
                  "; ".join(bad[:5]) if bad else "全部记录带证据指针")


def gate_rust_tests():
    """Rust 工作区测试（crates/ 存在后启用）。"""
    crates = ROOT / "crates"
    if not any(crates.glob("*/Cargo.toml")):
        return report("rust-tests", SKIP, "crates/ 尚无 Cargo 工程（P3 启动后启用）")
    # 夹具自供给（迭代 55）：assets/raw 与 oracle trace 均不入库，CI checkout
    # 缺失曾致 cargo test 即刻 panic（gates CI 100+ 次全红根因）。缺则再生，
    # 齐备则零成本（扫描+stat）。供给失败 ⇒ 门禁 FAIL（不静默跳过）。
    prov = subprocess.run([sys.executable, str(ROOT / "reference/oracle/provision_tests.py")],
                          cwd=ROOT, capture_output=True,
                          text=True, encoding="utf-8", errors="replace")
    prov_lines = [l for l in prov.stdout.splitlines() if l.startswith("[provision]")]
    prov_note = "；".join(prov_lines[-6:]) if prov.returncode == 0 else \
        "夹具供给失败: " + (prov.stdout[-400:] + prov.stderr[-400:])
    if prov.returncode != 0:
        return report("rust-tests", FAIL, prov_note)
    # --release：与差分/移植工作同档（缓存常热）。迭代 39 实测 debug 档每次
    # check 付 ~390s 近全量重编译（开发全程只构建 release）；溢出检查由
    # [profile.release] overflow-checks=true 保持（Cargo.toml）。
    r = subprocess.run(["cargo", "test", "--workspace", "--release"],
                       cwd=ROOT, capture_output=True,
                       text=True, encoding="utf-8", errors="replace")
    ok = r.returncode == 0
    if ok:
        msg = "cargo test --release 全绿"
    else:
        # cargo 的编译/测试错误走 **stderr**——只入 stdout 会把 CI 失败原因
        # 截断成不可见（迭代 57 起 CI 红根因排查受阻，迭代 60 补修）
        msg = (r.stdout[-200:] + " | STDERR: " + r.stderr[-400:]) if (r.stdout or r.stderr) else "cargo test 无输出"
    if not ok and prov_note:
        msg += "；" + prov_note
    elif ok and prov_note and "已再生" in prov_note:
        msg += "；" + prov_note
    return report("rust-tests", PASS if ok else FAIL, msg)


# ---------- 测试计数 ratchet（迭代 27，防嵌套 #[test] 假绿） ----------

RATCHET = ROOT / "data/status/baselines/rust-test-counts.json"


def collect_test_counts():
    """逐测试二进制清点用例数。返回 {key: count}；key = "<target>(<kind>)"。
    cargo test 对嵌套 #[test]（libtest 不可见）静默 0 收录——只有计数 ratchet 能抓
    （docs/knowledge/gotcha_nested-test-silently-skipped.md，preset_help 实证）。"""
    r = subprocess.run(["cargo", "test", "--workspace", "--release", "--no-run",
                        "--message-format=json"],
                       cwd=ROOT, capture_output=True,
                       text=True, encoding="utf-8", errors="replace")
    if r.returncode != 0:
        return None
    seen = {}
    for line in r.stdout.splitlines():
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get("reason") != "compiler-artifact" or not msg.get("profile", {}).get("test"):
            continue
        exe = msg.get("executable")
        target = msg.get("target", {})
        kinds = [k for k in target.get("kind", []) if k in ("lib", "bin", "test")]
        if not exe or not kinds:
            continue
        seen[f"{target.get('name', '?')}({kinds[0]})"] = exe
    counts = {}
    for key, exe in sorted(seen.items()):
        lr = subprocess.run([exe, "--list", "--format", "terse"],
                            capture_output=True,
                            text=True, encoding="utf-8", errors="replace")
        n = sum(1 for l in lr.stdout.splitlines() if l.endswith(": test"))
        counts[key] = counts.get(key, 0) + n
    return counts


def gate_test_ratchet():
    """测试计数 ratchet：每测试二进制用例数只许增不许减。
    减少 ⇒ 有测试被删除或嵌套化静默失效（iteration 26 前车）。"""
    if not (ROOT / "crates").exists():
        return report("test-ratchet", SKIP, "crates/ 不存在")
    cur = collect_test_counts()
    if cur is None:
        return report("test-ratchet", FAIL, "cargo test --no-run 失败")
    if not RATCHET.exists():
        return report("test-ratchet", FAIL,
                      f"基线缺失 {RATCHET}（python3 gates/cli.py ratchet-update 生成）")
    basec = json.loads(RATCHET.read_text(encoding="utf-8")).get("binaries", {})
    drops = []
    for k, b in basec.items():
        c = cur.get(k)
        if c is None:
            drops.append(f"{k}: 二进制消失(基线{b})")
        elif c < b:
            drops.append(f"{k}: {b}->{c}")
    if drops:
        return report("test-ratchet", FAIL,
                      "用例数回退（疑似删除/嵌套化静默失效）: " + "; ".join(drops))
    newbins = {k: v for k, v in cur.items() if k not in basec}
    detail = (f"total={sum(cur.values())}(基线{sum(basec.values())}) 无回退"
              + (f"；新增二进制待 ratchet-update: {newbins}" if newbins else ""))
    return report("test-ratchet", PASS, detail)


def cmd_ratchet_update():
    """重建测试计数基线（基线更新须在 commit 正文写明原因——AGENTS.md §2）。"""
    cur = collect_test_counts()
    if cur is None:
        print("cargo test --no-run 失败")
        return 1
    payload = {
        "comment": "每测试二进制用例数 ratchet（只许增不许减）。减少 = 测试被删除或"
                   "嵌套化静默失效（gotcha_nested-test-silently-skipped）。"
                   "更新基线必须在 commit 正文写明原因与影响。",
        "binaries": dict(sorted(cur.items())),
        "total": sum(cur.values()),
    }
    RATCHET.parent.mkdir(parents=True, exist_ok=True)
    RATCHET.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"基线已更新: total={payload['total']} binaries={len(cur)} -> {RATCHET}")
    return 0


def gate_reference():
    """参考版回归：改 reference/ 后 trace 必须逐 tick 不变（P1 oracle 就绪后启用）。"""
    if not (ROOT / "reference/oracle").exists():
        return report("reference-regression", SKIP, "参考版 oracle 未就绪（P1 交付后启用）")
    r = subprocess.run([sys.executable, "reference/oracle/run.py"], cwd=ROOT,
                       capture_output=True, text=True, encoding="utf-8", errors="replace")
    return report("reference-regression", PASS if r.returncode == 0 else FAIL, r.stdout[-200:])




def gate_scenarios():
    """深场景套件：scenarios/ 每个输入脚本 A==B==C 三方对拍（重，milestone 级）。"""
    scen = ROOT / "reference/oracle/scenarios"
    if not scen.exists() or not list(scen.glob("*.txt")):
        return report("scenario-suite", SKIP, "reference/oracle/scenarios/ 无场景脚本")
    r = subprocess.run([sys.executable, "reference/oracle/run.py", "--scenarios"], cwd=ROOT,
                       capture_output=True, text=True, encoding="utf-8", errors="replace")
    tail = r.stdout.strip().splitlines()
    return report("scenario-suite", PASS if r.returncode == 0 else FAIL,
                  tail[-1] if tail else r.stdout[-200:])

def gate_trace():
    """差分 L1-L3：参考版 vs Rust 同输入逐 tick 比对（P4 启用）。"""
    if not (ROOT / "crates/game-oracle/Cargo.toml").exists():
        return report("diff-trace", SKIP, "game-oracle 未就绪（P4 启动后启用）")
    r = subprocess.run(["cargo", "test", "-p", "game-oracle"], cwd=ROOT,
                       capture_output=True, text=True, encoding="utf-8", errors="replace")
    return report("diff-trace", PASS if r.returncode == 0 else FAIL, r.stdout[-200:])


def gate_visual():
    """视觉 L4：截图 diff（白名单外逐像素一致）。"""
    if not (ROOT / "gates/visual_diff.py").exists():
        return report("visual-diff", SKIP, "视觉比对器未就绪（P4 启动后启用）")
    r = subprocess.run([sys.executable, "gates/visual_diff.py"], cwd=ROOT,
                       capture_output=True, text=True, encoding="utf-8", errors="replace")
    return report("visual-diff", PASS if r.returncode == 0 else FAIL, r.stdout[-200:])


# ---------- 入口 ----------

def cmd_status():
    print("== 门禁状态预检（只读）==")
    print(f"ROOT: {ROOT}")
    base = load_baseline()
    if base:
        print(f"基线: {json.dumps(base.get('assets', base), ensure_ascii=False)}")
    else:
        print("基线: 缺失")
    # 场景套件（gate_scenarios）不进 status：重跑动辄数十分钟，只在 all/子命令执行
    statuses = [gate_original(), gate_assets_manifest(), gate_reference_seed_integrity(),
                gate_knowledge_format(), gate_docs_links(), gate_naming_ledger(), gate_rust_tests(),
                gate_test_ratchet(),
                gate_reference(), gate_trace(), gate_visual()]
    print("== 汇总 ==")
    print(f"PASS={statuses.count(PASS)} FAIL={statuses.count(FAIL)} "
          f"SKIP={statuses.count(SKIP)} TOLERATED={statuses.count(TOLERATE)}")
    return 1 if FAIL in statuses else 0


def gate_state_machine_anchors():
    """状态机锚校验（件 C/D 挂点）：key-code-contract §5 锚表 + 边数对照
    （TRANSFER_EDGES vs state-machine-edges.md 表）——防行号/注释漂移（前科两次）。"""
    deobf = ROOT / "reference" / "src" / "deobf" / "a.java"
    if not deobf.exists():
        return report("state-machine-anchors", SKIP, "deobf a.java 缺失（跳过锚校验）")
    lines = deobf.read_text(encoding="utf-8", errors="replace").splitlines()
    # 锚表：(行号, 期望 token)——key-code-contract.md §5
    anchors = [
        (4276, "keyValue = 0"),   # 尾清锚（run 内）
        (5296, "keyValue = 0"),   # 尾清锚（BootMachine 侧）
        (5302, "keyValue = 0"),   # 尾清锚（BootMachine 侧）
        (3415, "gameMode = 17"),  # 菜单激活写点（mode 17 入口）
        (3420, "gameMode = 22"),  # 菜单激活写点（mode 22 入口）
        (902, "2, 3, 3, 3, 6, 3, 3, 6"),  # f_byte_arr_13 表常量
    ]
    bad = []
    for lineno, tok in anchors:
        lo, hi = max(0, lineno - 3), min(len(lines), lineno + 2)
        window = "\n".join(lines[lo:hi])
        if tok not in window:
            bad.append(f"a.java:{lineno} 期望 {tok!r} 未命中（±2 邻域）")
    # 边数对照：scene.rs TRANSFER_EDGES vs edges 文档表
    scene = ROOT / "crates" / "game-core" / "src" / "scene.rs"
    edges_doc = ROOT / "docs" / "spec" / "state-machine-edges.md"
    if scene.exists() and edges_doc.exists():
        code_edges = scene.read_text(encoding="utf-8").count("(GameMode::")
        doc_rows = sum(1 for l in edges_doc.read_text(encoding="utf-8").splitlines()
                       if l.startswith("| ") and l[2:3].isdigit())
        # TRANSFER_EDGES 常量内的 (GameMode:: 计数（表外也有——取常量段）
        seg = scene.read_text(encoding="utf-8")
        i = seg.find("pub static TRANSFER_EDGES")
        j = seg.find("];", i)
        code_edges = sum(1 for l in seg[i:j].splitlines() if l.strip().startswith("(GameMode::")) if i >= 0 else -1
        if code_edges != doc_rows:
            bad.append(f"边数不一致：TRANSFER_EDGES {code_edges} vs edges 文档 {doc_rows}（件 D 维护义务）")
    return report("state-machine-anchors", FAIL if bad else PASS,
                  f"锚 {len(anchors)} + 边数对照；" + ("；".join(bad) if bad else "全命中"))


def cmd_check():
    print("== 总闸 gates/cli.py check ==")
    statuses = [gate_original(), gate_assets_manifest(), gate_reference_seed_integrity(),
                gate_knowledge_format(), gate_docs_links(), gate_naming_ledger(), gate_rust_tests(),
                gate_test_ratchet(), gate_state_machine_anchors()]
    # SKIP 项必须显式出现在输出（上面已打印），不得静默
    print("== 汇总 ==")
    print(f"PASS={statuses.count(PASS)} FAIL={statuses.count(FAIL)} SKIP={statuses.count(SKIP)}")
    return 1 if FAIL in statuses else 0


def cmd_all():
    print("== 全量门禁 ==")
    statuses = [gate_original(), gate_assets_manifest(), gate_reference_seed_integrity(),
                gate_knowledge_format(), gate_docs_links(), gate_naming_ledger(), gate_rust_tests(),
                gate_test_ratchet(),
                gate_reference(), gate_scenarios(), gate_trace(), gate_visual()]
    print(f"PASS={statuses.count(PASS)} FAIL={statuses.count(FAIL)} SKIP={statuses.count(SKIP)}")
    return 1 if FAIL in statuses else 0


COMMANDS = {
    "status": cmd_status,
    "check": cmd_check,
    "all": cmd_all,
    "original": lambda: 1 if gate_original() == FAIL else 0,
    "reference-seed": lambda: 1 if gate_reference_seed_integrity() == FAIL else 0,
    "scenarios": gate_scenarios,
    "reference": lambda: 1 if gate_reference() == FAIL else 0,
    "trace": lambda: 1 if gate_trace() == FAIL else 0,
    "visual": lambda: 1 if gate_visual() == FAIL else 0,
    "ratchet": lambda: 1 if gate_test_ratchet() == FAIL else 0,
    "ratchet-update": cmd_ratchet_update,
}

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "status"
    if cmd not in COMMANDS:
        print(__doc__)
        sys.exit(2)
    sys.exit(COMMANDS[cmd]())
