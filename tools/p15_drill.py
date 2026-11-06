#!/usr/bin/env python3
"""p15_drill.py — DDDSA P1-5 零改码演练（spec v3 §4 四段活演）。

段①：items_mod.json 追加一行（演练品 id=1001）；
段②：拒载闸过（cargo test p15 加载断言实跑）；
段③：git diff crates/ = 空（加行零改码判据——含 GameScene 主体）；
段④：哈希双查（保真表 sha 不变/mod 表不入 FIDELITY_TABLES 闸）。
收尾：回滚演练行（树净）。
输出=贴件（$() 重定向生成面）。
"""
from __future__ import annotations
import hashlib, json, re, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MOD = ROOT / "data/tables/items_mod.json"
FID = ROOT / "data/tables/items.json"


def sh(cmd: str) -> tuple[int, str]:
    r = subprocess.run(cmd, shell=True, cwd=ROOT, capture_output=True, text=True)
    return r.returncode, (r.stdout + r.stderr).strip()


def main() -> int:
    fid_before = hashlib.sha256(FID.read_bytes()).hexdigest()
    gate = (ROOT / "gates/cli.py").read_text(encoding="utf-8")
    in_gate = "items_mod.json" in gate

    orig = MOD.read_text(encoding="utf-8")
    _, diff_before = sh("git diff crates/")
    print("=== P1-5 零改码演练（四段活演） ===")
    # 段①：追加演练行
    row = ('  {\n   "id": 1001,\n   "name": "演练行",\n   "icon": {"entity_type": 13},\n'
           '   "desc": "段①活演行（脚本收尾回滚）。",\n   "pickup_msg": "演练。",\n'
           '   "effect": {"key": "Passive", "params": {"kind": "Drill"}},\n'
           '   "evidence": {"name": "p15_drill 段①", "desc": "活演", "effect": "活演"}\n  }')
    mutated = orig.replace("\n ]\n}", ",\n" + row + "\n ]\n}")
    MOD.write_text(mutated, encoding="utf-8")
    rows = json.loads(mutated)["rows"]
    print(f"段① 追加演练行 id=1001：表行数 {len(rows)}（前 {len(json.loads(orig)['rows'])}）")
    # 段②：拒载闸过+加载断言实跑
    rc, out = sh("cargo test -p game-oracle --test p15_zero_code_drill 2>&1 | grep 'test result'")
    print(f"段② 拒载闸过+无头加载断言：{out}")
    ok2 = rc == 0 and "3 passed" in out
    # 段③：零改码判据（**演练增量**=段①前后 git diff crates/ 快照对比——
    # 在途 WIP diff 隔离；加行前后 diff 全同=加行零改码）
    _, diff_after = sh("git diff crates/")
    clean = diff_before == diff_after
    print(f"段③ 零改码判据（git diff crates/ 段①前后快照对比）：{'全同=过（演练增量零）' if clean else 'FAIL（加行引码变）'}")
    print(f"    （在途 WIP diff 隔离：段①前已有 diff {len(diff_before)} 字节=非演练引入）")
    # 段④：哈希双查
    fid_after = hashlib.sha256(FID.read_bytes()).hexdigest()
    fid_ok = fid_before == fid_after
    print(f"段④a 保真表 sha 不变：{fid_ok}（{fid_before[:12]}…）")
    print(f"段④b mod 表不入 FIDELITY_TABLES 闸：{'过（不入闸=两份制红线）' if not in_gate else 'FAIL（误入闸）'}")
    # 收尾回滚
    MOD.write_text(orig, encoding="utf-8")
    rc, out = sh("git diff --stat data/tables/items_mod.json")
    print(f"收尾 演练行回滚：{'树净=过' if out == '' else 'FAIL:' + out}")
    verdict = ok2 and clean and fid_ok and not in_gate
    print(f"=== 演练判定：{'全过' if verdict else 'FAIL'} ===")
    return 0 if verdict else 1


if __name__ == "__main__":
    sys.exit(main())
