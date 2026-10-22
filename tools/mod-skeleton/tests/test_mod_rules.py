#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""mod 骨架四规则最小测试（TDD：先红后绿，红证据入 commit message）。

survey §2.2 两份制四规则写死：
  规则 1  mod 禁覆盖保真 id（id 命名空间隔离：mod 型号 ≥ 1000 或前缀）
  规则 2  金样运行禁 mod 表（断言）
  规则 3  保真表内容哈希闸护航（哈希入 gates——非荣誉制）
  规则 4  mod 独立基线（不进金样闸）

附带覆盖零改码链演练两失败面的加载语义（坏外键/quirk 型走 Q-00x 路径）。
跑法（stdlib unittest，无第三方依赖）：
  python3 -m unittest discover -s tools/mod-skeleton/tests -v
"""
from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

MOD_DIR = Path(__file__).resolve().parents[1]
ROOT = MOD_DIR.parents[1]
sys.path.insert(0, str(MOD_DIR))

from loader import (  # noqa: E402
    MOD_ANIM_PREFIX,
    MOD_TYPE_ID_MIN,
    ModBaseline,
    ModReject,
    assert_golden_run_clean,
    load_fidelity,
    load_mod_table,
    verify_fidelity_hashes,
)

TABLES = ROOT / "data" / "tables"


def _mod_table(tmp: Path, sprite_rows: list[dict], anim_rows: list[dict] | None = None,
               quirks: list[dict] | None = None, resource_rows: list[dict] | None = None,
               mod_id: str = "fake-enemy-pack") -> Path:
    """写一张 mod 表到 tmp/mod_table.json（骨架形：schema_version+mod_id+四桶）。"""
    doc = {
        "schema_version": 2,
        "mod_id": mod_id,
        "sprite_defs": sprite_rows,
        "anim_defs": anim_rows or [],
        "quirks_in_code": quirks or [],
        "resource_rows": resource_rows or [],
    }
    p = tmp / "mod_table.json"
    p.write_text(json.dumps(doc, ensure_ascii=False), encoding="utf-8")
    return p


def _good_row(**over) -> dict:
    """域内正例行：型号 mod 命名空间 1000、anim_profile 复用保真 anim_row_0、图集复用。"""
    row = {
        "type_id": MOD_TYPE_ID_MIN,
        "sheet": "sptenemy1",
        "sheet_index": 3,
        "frame_w": 32,
        "frame_h": 45,
        "render_category": 32,
        "anim_row": 0,
        "type_frame_width": 32,
        "anim_profile": "anim_row_0",
        "assembly_profile": None,
        "insert_block": False,
        "floor_category_override": [],
        "density": 1,
    }
    row.update(over)
    return row


class Rule1IdNamespace(unittest.TestCase):
    """规则 1：mod 禁覆盖保真 id（≥1000 或前缀）——两份制命名空间隔离。"""

    def setUp(self):
        self.fidelity = load_fidelity(TABLES)

    def test_mod_type_id_namespace_accepts(self):
        with tempfile.TemporaryDirectory() as d:
            p = _mod_table(Path(d), [_good_row()])
            bundle = load_mod_table(p, self.fidelity)
        self.assertEqual(bundle.mod_id, "fake-enemy-pack")
        self.assertEqual([r["type_id"] for r in bundle.sprite_rows], [MOD_TYPE_ID_MIN])

    def test_rejects_fidelity_id_override(self):
        """mod 行 type_id=72（保真冻结 id）= 禁覆盖保真 id，拒载。"""
        with tempfile.TemporaryDirectory() as d:
            p = _mod_table(Path(d), [_good_row(type_id=72)])
            with self.assertRaises(ModReject) as cm:
                load_mod_table(p, self.fidelity)
        self.assertEqual(cm.exception.kind, "DomainOverflow")
        self.assertIn("禁覆盖保真", cm.exception.detail)

    def test_rejects_out_of_domain_type_id(self):
        """域外型 id=89：既非保真域 0..=88 也非 mod 命名空间 ≥1000，拒载。"""
        with tempfile.TemporaryDirectory() as d:
            p = _mod_table(Path(d), [_good_row(type_id=89)])
            with self.assertRaises(ModReject) as cm:
                load_mod_table(p, self.fidelity)
        self.assertEqual(cm.exception.kind, "DomainOverflow")
        self.assertIn("89", cm.exception.detail)

    def test_mod_anim_id_prefix_rule(self):
        """mod 动画 id 须 mod: 前缀；无前缀=撞保真命名空间=拒载。"""
        with tempfile.TemporaryDirectory() as d:
            good = _mod_table(Path(d), [_good_row(anim_profile="mod:walk")],
                              anim_rows=[{"id": "mod:walk", "frames": [0], "step_ticks": 2,
                                          "loop_mode": "cycle"}])
            bundle = load_mod_table(good, self.fidelity)
            self.assertEqual(bundle.anim_rows[0]["id"], "mod:walk")
        with tempfile.TemporaryDirectory() as d:
            bad = _mod_table(Path(d), [_good_row(anim_profile="walk_3frame")],
                             anim_rows=[{"id": "walk_3frame", "frames": [0], "step_ticks": 2,
                                         "loop_mode": "cycle"}])
            with self.assertRaises(ModReject) as cm:
                load_mod_table(bad, self.fidelity)
        self.assertEqual(cm.exception.kind, "DomainOverflow")

    def test_mod_anim_id_collision_with_fidelity(self):
        """mod 动画 id 直接抄保真 anim_row_0 = 覆盖保真 id，拒载。"""
        with tempfile.TemporaryDirectory() as d:
            p = _mod_table(Path(d), [_good_row()],
                           anim_rows=[{"id": "anim_row_0", "frames": [0], "step_ticks": 2,
                                       "loop_mode": "cycle"}])
            with self.assertRaises(ModReject) as cm:
                load_mod_table(p, self.fidelity)
        self.assertEqual(cm.exception.kind, "DomainOverflow")
        self.assertIn("禁覆盖保真", cm.exception.detail)


class Rule2GoldenRunBansMod(unittest.TestCase):
    """规则 2：金样运行禁 mod 表（断言）——保真面运行=纯保真表。"""

    def setUp(self):
        self.fidelity = load_fidelity(TABLES)

    def test_golden_run_clean_with_fidelity_tables(self):
        assert_golden_run_clean(
            [TABLES / "sprite_defs.json", TABLES / "anim_defs.json",
             TABLES / "sheet_index.json"], self.fidelity)

    def test_golden_run_rejects_mod_table(self):
        with tempfile.TemporaryDirectory() as d:
            p = _mod_table(Path(d), [_good_row()])
            with self.assertRaises(AssertionError) as cm:
                assert_golden_run_clean(
                    [TABLES / "sprite_defs.json", p], self.fidelity)
        self.assertIn("禁 mod 表", str(cm.exception))


class Rule3FidelityHashGuard(unittest.TestCase):
    """规则 3：保真表内容哈希闸护航——哈希不匹配=拒载（非荣誉制）。"""

    def _tmp_tables(self, d: Path) -> Path:
        t = d / "tables"
        t.mkdir()
        for name in ("sprite_defs.json", "anim_defs.json", "sheet_index.json"):
            (t / name).write_text("{}", encoding="utf-8")
        return t

    def _baseline(self, d: Path, tables: Path, tamper: bool = False) -> Path:
        import hashlib
        entries = {}
        for name in ("sprite_defs.json", "anim_defs.json", "sheet_index.json"):
            h = hashlib.sha256((tables / name).read_bytes()).hexdigest()
            if tamper and name == "sprite_defs.json":
                h = "0" * 64
            entries[name] = h
        b = d / "hash_baseline.json"
        b.write_text(json.dumps({"tables": entries}), encoding="utf-8")
        return b

    def test_hash_guard_passes_on_match(self):
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            t = self._tmp_tables(d)
            b = self._baseline(d, t)
            verify_fidelity_hashes(t, b)  # 不抛=护航通过

    def test_hash_guard_rejects_drift(self):
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            t = self._tmp_tables(d)
            b = self._baseline(d, t, tamper=True)
            with self.assertRaises(ModReject) as cm:
                verify_fidelity_hashes(t, b)
        self.assertEqual(cm.exception.kind, "FidelityHashDrift")

    def test_mod_load_requires_hash_guard(self):
        """护航是硬前置：哈希漂移时命名空间都建不起来，mod 通道无从谈起。"""
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            t = self._tmp_tables(d)
            b = self._baseline(d, t, tamper=True)
            with self.assertRaises(ModReject) as cm:
                fidelity = load_fidelity(t, hash_baseline=b)
                load_mod_table(_mod_table(d, [_good_row()]), fidelity)
        self.assertEqual(cm.exception.kind, "FidelityHashDrift")


class Rule4ModIndependentBaseline(unittest.TestCase):
    """规则 4：mod 独立基线（不进金样闸）——mod 表哈希单独立账，保真哈希闸不背 mod。"""

    def test_mod_baseline_roundtrip_and_independence(self):
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            p = _mod_table(d, [_good_row()])
            base = ModBaseline(d / "mod_baseline.json")
            base.update([p])
            base.verify([p])  # 不抛=一致
            # 独立性：mod 基线只记 mod 表；保真哈希基线不含 mod 条目
            payload = json.loads((d / "mod_baseline.json").read_text(encoding="utf-8"))
            self.assertEqual(list(payload["mod_tables"]), ["mod_table.json"])
            fid_base = d / "fid_baseline.json"
            fid_base.write_text(json.dumps({"tables": {
                "sprite_defs.json": "x", "anim_defs.json": "y",
                "sheet_index.json": "z"}}), encoding="utf-8")
            fid_keys = set(json.loads(fid_base.read_text(encoding="utf-8"))["tables"])
            self.assertFalse(fid_keys & set(payload["mod_tables"]),
                             "mod 表不得进保真哈希闸基线（不进金样闸）")

    def test_mod_baseline_detects_drift(self):
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            p = _mod_table(d, [_good_row()])
            base = ModBaseline(d / "mod_baseline.json")
            base.update([p])
            p.write_text(p.read_text(encoding="utf-8") + " ", encoding="utf-8")
            with self.assertRaises(ModReject) as cm:
                base.verify([p])
        self.assertEqual(cm.exception.kind, "ModBaselineDrift")


class FailureFaces(unittest.TestCase):
    """零改码链演练两失败面的加载语义（第三面=域外型，见 Rule1）。"""

    def setUp(self):
        self.fidelity = load_fidelity(TABLES)

    def test_bad_foreign_key_rejected(self):
        """坏外键：anim_profile 孤儿=拒载（§2.3）。"""
        with tempfile.TemporaryDirectory() as d:
            p = _mod_table(Path(d), [_good_row(anim_profile="anim_row_nope")])
            with self.assertRaises(ModReject) as cm:
                load_mod_table(p, self.fidelity)
        self.assertEqual(cm.exception.kind, "BadForeignKey")
        self.assertIn("anim_row_nope", cm.exception.detail)

    def test_quirk_shaped_row_requires_q00x(self):
        """quirk 型（frame_h=null=运行时图高，Q-001/Q-003 族）须走 Q-00x 路径登记，
        无 quirk_ref=拒载（§2.1 禁静默留码）。"""
        with tempfile.TemporaryDirectory() as d:
            p = _mod_table(Path(d), [_good_row(frame_h=None)])
            with self.assertRaises(ModReject) as cm:
                load_mod_table(p, self.fidelity)
        self.assertEqual(cm.exception.kind, "QuirkPathRequired")
        self.assertIn("Q-00", cm.exception.detail)

    def test_quirk_registered_row_accepted(self):
        """登记 Q-101（quirk_id/reason/src/red_test_id 四字段留痕）+ quirk_ref ⇒ 走 Q-00x 路径接受。"""
        quirk = {"quirk_id": "Q-101",
                 "reason": "假想新敌人型高=运行时图高（Q-001/Q-003 族）纯表规则装不下",
                 "src": "a.java:6318", "red_test_id": "drill-quirk-q101"}
        with tempfile.TemporaryDirectory() as d:
            p = _mod_table(Path(d), [_good_row(frame_h=None, quirk_ref="Q-101")],
                           quirks=[quirk])
            bundle = load_mod_table(p, self.fidelity)
        self.assertEqual(bundle.quirk_rows[0]["quirk_id"], "Q-101")

    def test_unknown_field_rejected(self):
        """unknown-field=拒载（第 5 类镜像，防拼写静默）。"""
        with tempfile.TemporaryDirectory() as d:
            p = _mod_table(Path(d), [_good_row(anim_profil="typo")])
            with self.assertRaises(ModReject) as cm:
                load_mod_table(p, self.fidelity)
        self.assertEqual(cm.exception.kind, "UnknownField")


if __name__ == "__main__":
    unittest.main()
