#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""零改码链演练实跑（survey §3 步 5=验收闸；失败面设计 3/3④）。

命题（survey §3 v2 降格口径）：**同图集复用、消费替换完成后域内零改码**——
加一个假想新敌人 = 纯表+资源行（sprite_defs 加行 + anim_profile 复用 + 资源行复用
保真图集容器/槽位），**验证零 Rust 改动**（`git status --porcelain crates/` 为空）。

两份制下的加行位置（§2.2）：保真三表被内容哈希闸冻结（gate `table-content-hash`，
变更=勘误流程+金样重过）⇒ 新内容进 **mod 通道**（型号 id ≥ 1000 命名空间），
保真表一字不动、金样运行禁 mod 表（规则 2）。

演练步骤（每步断言，任一失败 ⇒ 退出码 1）：
  [0] 前置护航：保真三表 sha256 == gate `table-content-hash` 基线（规则 3）
  [0b] 零改码基线：git 工作树 crates/ 干净（diff+status 皆空）
  [1] 正例：假想新敌人 mod 行（sprite_defs 加行+anim_profile 复用+资源行）⇒ 加载通过；
      真解析器（crates/game-data defs.rs）保真三表原样可载（零改码链起点）
  [2] 失败面 A 域外型：保真 sprite_defs 直接加行 id=89 ⇒ 真解析器域溢出拒载（域外三界①）；
      mod 通道 id=89 ⇒ 命名空间拒载
  [3] 失败面 B 坏外键：anim_profile 孤儿 ⇒ mod 通道拒载；保真侧同错 ⇒ 真解析器坏外键拒载
  [4] 失败面 C quirk 型：frame_h=null（运行时图高，Q-001/Q-003 族）无登记 ⇒
      走 Q-00x 路径拒载（§2.1 禁静默留码）；登记 Q-101 四字段+quirk_ref ⇒ 通过
  [5] 规则 2：金样运行清单混入 mod 表 ⇒ 断言触发
  [6] 规则 4：mod 独立基线立账/复验；保真哈希基线不含 mod 条目（不进金样闸）
  [7] 收尾：保真三表哈希仍绿 + git crates/ 仍空 = 零 Rust 改动实证

用法：python3 tools/zero-code-drill/drill.py
"""
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

DRILL_DIR = Path(__file__).resolve().parent
ROOT = DRILL_DIR.parents[1]
sys.path.insert(0, str(ROOT / "tools" / "mod-skeleton"))

from loader import (  # noqa: E402
    HASH_BASELINE,
    ModBaseline,
    ModReject,
    assert_golden_run_clean,
    load_fidelity,
    load_mod_table,
    verify_fidelity_hashes,
)

TABLES = ROOT / "data" / "tables"
GOLDEN_TABLES = [TABLES / "sprite_defs.json", TABLES / "anim_defs.json",
                 TABLES / "sheet_index.json"]

_HARNESS_MAIN = r'''
use std::env;
use std::fs;
use game_data::defs::{self, DefBundle};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("");
    let res: Result<String, String> = match mode {
        "parse-sprite" => {
            let src = fs::read_to_string(&args[2]).expect("read sprite");
            defs::parse_sprite_defs(&src).map(|_| "OK".to_string()).map_err(|e| e.to_string())
        }
        "bundle" => {
            let s = fs::read_to_string(&args[2]).expect("read sprite");
            let a = fs::read_to_string(&args[3]).expect("read anim");
            let h = fs::read_to_string(&args[4]).expect("read sheet");
            DefBundle::load(&s, &a, &h, &[]).map(|_| "OK".to_string()).map_err(|e| e.to_string())
        }
        _ => { eprintln!("usage: harness parse-sprite <f> | bundle <sprite> <anim> <sheet>"); std::process::exit(2); }
    };
    match res {
        Ok(s) => println!("{s}"),
        Err(e) => println!("REJECT {e}"),
    }
}
'''

_RESULTS: list[tuple[bool, str]] = []


def check(ok: bool, label: str) -> bool:
    _RESULTS.append((bool(ok), label))
    print(f"  [{'✓' if ok else '✗'}] {label}")
    return bool(ok)


class DrillFailure(AssertionError):
    pass


# ---------------------------------------------------------------- 真解析器 harness

class RealParser:
    """在临时目录跑真解析器（crates/game-data `defs.rs`，零依赖）——
    演练的拒载断言以**真加载层**为准，不镜像自证。"""

    def __init__(self, workdir: Path):
        self.proj = workdir / "harness"
        self.proj.mkdir(parents=True, exist_ok=True)
        (self.proj / "Cargo.toml").write_text(
            "[package]\n"
            'name = "zero-code-drill-harness"\n'
            'version = "0.0.0"\n'
            'edition = "2021"\n'
            "[dependencies]\n"
            f'game-data = {{ path = "{ROOT / "crates" / "game-data"}" }}\n'
            "[[bin]]\n"
            'name = "harness"\n'
            'path = "main.rs"\n', encoding="utf-8")
        (self.proj / "main.rs").write_text(_HARNESS_MAIN, encoding="utf-8")

    def run(self, *args: str) -> str:
        r = subprocess.run(
            ["cargo", "run", "--offline", "--quiet", "--manifest-path",
             str(self.proj / "Cargo.toml"), "--", *args],
            capture_output=True, text=True, encoding="utf-8", errors="replace",
            cwd=self.proj, timeout=900)
        out = (r.stdout or "").strip()
        if r.returncode != 0:
            raise DrillFailure(
                f"真解析器 harness 运行失败 rc={r.returncode}\n"
                f"stdout: {out[-400:]}\nstderr: {(r.stderr or '')[-800:]}")
        return out


# ---------------------------------------------------------------- 演练件

def _mod_table_doc(sprite_rows, anim_rows=None, quirks=None, resource_rows=None) -> dict:
    return {
        "schema_version": 2,
        "mod_id": "fake-enemy-pack",
        "sprite_defs": sprite_rows,
        "anim_defs": anim_rows or [],
        "quirks_in_code": quirks or [],
        "resource_rows": resource_rows or [],
    }


def _new_enemy_row(**over) -> dict:
    """假想新敌人（mod 命名空间 1000）：纯表行——anim_profile 复用保真 `anim_row_0`，
    图集/槽位复用保真 `sptenemy1`/3（同图集复用=域内，survey §3）。"""
    row = {
        "type_id": 1000,
        "sheet": "sptenemy1",
        "sheet_index": 3,
        "frame_w": 32,
        "frame_h": 45,
        "render_category": 32,
        "anim_row": 0,
        "type_frame_width": 32,
        "anim_profile": "anim_row_0",   # anim_profile 复用（域内：ANIM 表 89 行不动）
        "assembly_profile": None,
        "insert_block": False,
        "floor_category_override": [],
        "density": 1,
    }
    row.update(over)
    return row


def _write(path: Path, doc: dict) -> Path:
    path.write_text(json.dumps(doc, ensure_ascii=False), encoding="utf-8")
    return path


def _git_crates_dirty() -> str:
    """返回 crates/ 的 git 改动描述（空串=零改动）；diff + status 双查（含未追踪）。"""
    parts = []
    for cmd in (["git", "diff", "--", "crates/"], ["git", "status", "--porcelain", "--", "crates/"]):
        r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True,
                           encoding="utf-8", errors="replace")
        out = (r.stdout or "").strip()
        if out:
            parts.append(f"{' '.join(cmd[1:3])}: {out[:300]}")
    return " | ".join(parts)


def main() -> int:
    print("== 零改码链演练实跑（survey §3 步 5=验收闸）==")
    with tempfile.TemporaryDirectory(prefix="zero-code-drill-") as td:
        tmp = Path(td)

        # [0] 规则 3 护航
        print("[0] 前置护航：保真表内容哈希闸（非荣誉制）")
        try:
            hashes = verify_fidelity_hashes(TABLES, HASH_BASELINE)
            check(True, f"保真三表 sha256 == 闸基线（{len(hashes)} 表，"
                        f"基线 {HASH_BASELINE.relative_to(ROOT)}）")
        except ModReject as e:
            check(False, f"保真表哈希护航失败：{e}")
            return _finish()

        # [0b] 零改码基线
        print("[0b] 零改码基线：git crates/ 干净")
        dirty = _git_crates_dirty()
        check(not dirty, f"git diff/status crates/ 为空（{dirty or '干净'}）")

        fidelity = load_fidelity(TABLES, hash_baseline=HASH_BASELINE)
        parser = RealParser(tmp)

        # [1] 正例
        print("[1] 正例：假想新敌人=纯表+资源行（sprite_defs 加行+anim_profile 复用）")
        mod_doc = _mod_table_doc(
            [_new_enemy_row()],
            resource_rows=[{"container": "sptenemy1", "slot": 3,
                            "source": "assets/mods/fake-enemy.png"}])
        mod_path = _write(tmp / "fake_enemy_mod.json", mod_doc)
        try:
            bundle = load_mod_table(mod_path, fidelity)
            check(bundle.sprite_rows[0]["type_id"] == 1000
                  and bundle.sprite_rows[0]["anim_profile"] == "anim_row_0",
                  "mod 通道接受假想新敌人（id 1000；anim_profile 复用 anim_row_0）")
        except ModReject as e:
            check(False, f"正例 mod 行被拒：{e}")
        ok = parser.run("bundle", str(TABLES / "sprite_defs.json"),
                        str(TABLES / "anim_defs.json"), str(TABLES / "sheet_index.json"))
        check(ok == "OK", f"真解析器保真三表原样可载（零改码链起点）→ {ok[:60]}")

        # [2] 失败面 A：域外型 id>=89
        print("[2] 失败面 A 域外型：保真 sprite_defs 直接加行 id=89 ⇒ 拒载（域外三界①）")
        sprite_doc = json.loads((TABLES / "sprite_defs.json").read_text(encoding="utf-8"))
        rogue = json.loads(json.dumps(sprite_doc["sprite_defs"][0]))
        rogue["type_id"] = 89
        sprite_doc["sprite_defs"].append(rogue)
        sandbox_sprite = _write(tmp / "sprite_defs_domain.json", sprite_doc)
        out = parser.run("parse-sprite", str(sandbox_sprite))
        check(out.startswith("REJECT") and "域溢出拒载" in out and "0..=88" in out,
              f"真解析器域溢出拒载（值 89 超域 0..=88）→ {out[:100]}")
        try:
            load_mod_table(_write(tmp / "mod_domain.json",
                                  _mod_table_doc([_new_enemy_row(type_id=89)])), fidelity)
            check(False, "mod 通道 id=89 应拒载")
        except ModReject as e:
            check(e.kind == "DomainOverflow" and "89" in e.detail,
                  f"mod 通道域外型拒载（id=89 非 ≥1000 命名空间）→ {e.detail[:80]}")

        # [3] 失败面 B：坏外键
        print("[3] 失败面 B 坏外键：anim_profile 孤儿 ⇒ 拒载（§2.3）")
        try:
            load_mod_table(_write(tmp / "mod_fk.json", _mod_table_doc(
                [_new_enemy_row(anim_profile="anim_row_nope")])), fidelity)
            check(False, "mod 通道坏外键应拒载")
        except ModReject as e:
            check(e.kind == "BadForeignKey" and "anim_row_nope" in e.detail,
                  f"mod 通道坏外键拒载 → {e.detail[:80]}")
        fk_sprite = json.loads(json.dumps(sprite_doc))
        fk_sprite["sprite_defs"] = fk_sprite["sprite_defs"][:89]  # 撤 A 面的越域行
        fk_sprite["sprite_defs"][0] = json.loads(json.dumps(fk_sprite["sprite_defs"][0]))
        fk_sprite["sprite_defs"][0]["anim_profile"] = "anim_row_nope"
        out = parser.run("bundle", str(_write(tmp / "sprite_defs_fk.json", fk_sprite)),
                         str(TABLES / "anim_defs.json"), str(TABLES / "sheet_index.json"))
        check(out.startswith("REJECT") and "坏外键拒载" in out and "anim_row_nope" in out,
              f"真解析器坏外键拒载（孤儿 anim_profile）→ {out[:100]}")

        # [4] 失败面 C：quirk 型走 Q-00x 路径
        print("[4] 失败面 C quirk 型：frame_h=null（运行时图高）⇒ 走 Q-00x 路径")
        try:
            load_mod_table(_write(tmp / "mod_quirk.json", _mod_table_doc(
                [_new_enemy_row(frame_h=None)])), fidelity)
            check(False, "quirk 型无登记应拒载")
        except ModReject as e:
            check(e.kind == "QuirkPathRequired" and "Q-00" in e.detail,
                  f"quirk 型拒载（走 Q-00x 路径登记，禁静默留码）→ {e.detail[:100]}")
        quirk = {"quirk_id": "Q-101",
                 "reason": "假想新敌人型高=运行时图高（Q-001/Q-003 族，纯表规则装不下）",
                 "src": "a.java:6318", "red_test_id": "drill-quirk-q101"}
        try:
            bundle = load_mod_table(_write(tmp / "mod_quirk_ok.json", _mod_table_doc(
                [_new_enemy_row(frame_h=None, quirk_ref="Q-101")], quirks=[quirk])), fidelity)
            check(bundle.quirk_rows and bundle.quirk_rows[0]["quirk_id"] == "Q-101",
                  "登记 Q-101（quirk_id/reason/src/red_test_id）+quirk_ref ⇒ 走 Q-00x 路径通过")
        except ModReject as e:
            check(False, f"登记后仍拒载：{e}")

        # [5] 规则 2：金样运行禁 mod 表
        print("[5] 规则 2：金样运行禁 mod 表（断言）")
        assert_golden_run_clean(GOLDEN_TABLES, fidelity)
        check(True, "金样运行清单（纯保真三表）通过")
        try:
            assert_golden_run_clean(GOLDEN_TABLES + [mod_path], fidelity)
            check(False, "金样混入 mod 表应断言触发")
        except AssertionError as e:
            check("禁 mod 表" in str(e), f"金样混入 mod 表 ⇒ 断言触发 → {str(e)[:80]}")

        # [6] 规则 4：mod 独立基线
        print("[6] 规则 4：mod 独立基线（不进金样闸）")
        base = ModBaseline(tmp / "mod_baseline.json")
        base.update([mod_path])
        base.verify([mod_path])
        payload = json.loads((tmp / "mod_baseline.json").read_text(encoding="utf-8"))
        fid_keys = set(json.loads(HASH_BASELINE.read_text(encoding="utf-8"))["tables"])
        check(set(payload["mod_tables"]) == {mod_path.name}
              and not (fid_keys & set(payload["mod_tables"])),
              "mod 基线独立立账；保真哈希闸基线不含 mod 条目")

        # [7] 收尾：保真哈希仍绿 + crates/ 零改动
        print("[7] 收尾：零 Rust 改动实证")
        try:
            verify_fidelity_hashes(TABLES, HASH_BASELINE)
            check(True, "演练后保真三表哈希仍绿（保真表一字未动）")
        except ModReject as e:
            check(False, f"演练后保真表哈希漂移：{e}")
        dirty = _git_crates_dirty()
        check(not dirty, f"git diff/status crates/ 仍为空（{dirty or '零 Rust 改动'}）")

    return _finish()


def _finish() -> int:
    n_ok = sum(1 for ok, _ in _RESULTS if ok)
    n_all = len(_RESULTS)
    verdict = "PASS" if n_ok == n_all else "FAIL"
    print(f"== 演练汇总：{n_ok}/{n_all} 通过 —— {verdict}（域内零改码链；域外三界=拒载/登记）==")
    return 0 if verdict == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())
