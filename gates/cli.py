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
  python3 gates/cli.py reference   # 参考版回归（P1 后启用）
  python3 gates/cli.py trace       # 差分 L1-L3（P3 后启用）
  python3 gates/cli.py visual      # 视觉 L4（P3 后启用）
  python3 gates/cli.py all         # 全量
"""
import hashlib
import json
import re
import subprocess
import sys
import zipfile
from pathlib import Path

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
                  "; ".join(bad) if bad else f"{len(sums.read_text().splitlines())} 个归档校验通过")


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
    r = subprocess.run(["git", "ls-files", "*.md"], cwd=ROOT, capture_output=True, text=True)
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
    r = subprocess.run(["cargo", "test", "--workspace"], cwd=ROOT, capture_output=True, text=True)
    return report("rust-tests", PASS if r.returncode == 0 else FAIL,
                  "cargo test 全绿" if r.returncode == 0 else r.stdout[-200:])


def gate_reference():
    """参考版回归：改 reference/ 后 trace 必须逐 tick 不变（P1 oracle 就绪后启用）。"""
    if not (ROOT / "reference/oracle").exists():
        return report("reference-regression", SKIP, "参考版 oracle 未就绪（P1 交付后启用）")
    r = subprocess.run(["python3", "reference/oracle/run.py"], cwd=ROOT, capture_output=True, text=True)
    return report("reference-regression", PASS if r.returncode == 0 else FAIL, r.stdout[-200:])


def gate_trace():
    """差分 L1-L3：参考版 vs Rust 同输入逐 tick 比对（P4 启用）。"""
    if not (ROOT / "crates/game-oracle/Cargo.toml").exists():
        return report("diff-trace", SKIP, "game-oracle 未就绪（P4 启动后启用）")
    r = subprocess.run(["cargo", "test", "-p", "game-oracle"], cwd=ROOT, capture_output=True, text=True)
    return report("diff-trace", PASS if r.returncode == 0 else FAIL, r.stdout[-200:])


def gate_visual():
    """视觉 L4：截图 diff（白名单外逐像素一致）。"""
    if not (ROOT / "gates/visual_diff.py").exists():
        return report("visual-diff", SKIP, "视觉比对器未就绪（P4 启动后启用）")
    r = subprocess.run(["python3", "gates/visual_diff.py"], cwd=ROOT, capture_output=True, text=True)
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
    statuses = [gate_original(), gate_assets_manifest(), gate_knowledge_format(),
                gate_docs_links(), gate_naming_ledger(), gate_rust_tests(),
                gate_reference(), gate_trace(), gate_visual()]
    print("== 汇总 ==")
    print(f"PASS={statuses.count(PASS)} FAIL={statuses.count(FAIL)} "
          f"SKIP={statuses.count(SKIP)} TOLERATED={statuses.count(TOLERATE)}")
    return 1 if FAIL in statuses else 0


def cmd_check():
    print("== 总闸 gates/cli.py check ==")
    statuses = [gate_original(), gate_assets_manifest(), gate_knowledge_format(),
                gate_docs_links(), gate_naming_ledger(), gate_rust_tests()]
    # SKIP 项必须显式出现在输出（上面已打印），不得静默
    print("== 汇总 ==")
    print(f"PASS={statuses.count(PASS)} FAIL={statuses.count(FAIL)} SKIP={statuses.count(SKIP)}")
    return 1 if FAIL in statuses else 0


def cmd_all():
    print("== 全量门禁 ==")
    statuses = [gate_original(), gate_assets_manifest(), gate_knowledge_format(),
                gate_docs_links(), gate_naming_ledger(), gate_rust_tests(),
                gate_reference(), gate_trace(), gate_visual()]
    print(f"PASS={statuses.count(PASS)} FAIL={statuses.count(FAIL)} SKIP={statuses.count(SKIP)}")
    return 1 if FAIL in statuses else 0


COMMANDS = {
    "status": cmd_status,
    "check": cmd_check,
    "all": cmd_all,
    "original": lambda: 1 if gate_original() == FAIL else 0,
    "reference": lambda: 1 if gate_reference() == FAIL else 0,
    "trace": lambda: 1 if gate_trace() == FAIL else 0,
    "visual": lambda: 1 if gate_visual() == FAIL else 0,
}

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "status"
    if cmd not in COMMANDS:
        print(__doc__)
        sys.exit(2)
    sys.exit(COMMANDS[cmd]())
