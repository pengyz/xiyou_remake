#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""等值单测（TDD）：data/tables/*.json 表值 == 机械提取值。

分层：
① 第二侧自钉（extractor 提取值 vs 手读 a.java 字面锚）——防提取器自身漂移；
② 表存在性（生成前 = 红）；
③ 等值比对（comparator 逐值三元组对账，错位逐行）；
④ 规则形抽样（transition_overrides / size_dispatch 手读字面）；
⑤ --check 确定性复跑同哈希。

跑法（stdlib unittest，无第三方依赖——本机 pytest 断言改写损坏故弃用）：
  python3 -m unittest discover -s tools/table-evidence/tests -v
"""
from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parents[1]
ROOT = TOOLS_DIR.parents[1]
sys.path.insert(0, str(TOOLS_DIR))

from comparator import Comparator  # noqa: E402
from extractor import (DEFAULT_JAVA, FIXTURE_WIDTH, extract,  # noqa: E402
                       extract_anim_offset_table, extract_anim_row_by_type,
                       extract_mark_des_removed, extract_mark_removed,
                       extract_resource_image_counts, extract_resource_names,
                       extract_seg67_pieces, extract_size_dispatch,
                       extract_type_frame_width, read_lines)

TABLES = ROOT / "data" / "tables"

_LINES = read_lines(DEFAULT_JAVA)
_DATA = extract(DEFAULT_JAVA)


def _load(name: str) -> dict:
    path = TABLES / name
    if not path.exists():
        raise AssertionError(
            f"表缺: {path}（红=未回填，跑 tools/table-evidence/cli.py generate）")
    return json.loads(path.read_text(encoding="utf-8"))


def _anchors(entry: dict, fact: str) -> list[dict]:
    return [a for a in entry.get("evidence", [])
            if isinstance(a.get("value"), dict) and fact in a["value"]]


# ---------------------------------------------------------------- ① 第二侧自钉

class TestSecondSide(unittest.TestCase):
    """extractor 提取值 vs 手读 a.java 字面（钉第二侧，防提取器漂移）。"""

    def test_anim_row_by_type(self):
        got = extract_anim_row_by_type(_LINES)
        self.assertEqual(len(got["values"]), 89)
        for idx, val, ln in ((0, 0, 499), (10, 2, 509), (41, 3, 540), (44, 4, 543),
                             (45, 2, 544), (64, 5, 563), (74, 2, 573), (88, 0, 587)):
            item = got["values"][idx]
            self.assertEqual((item["value"], item["line"]), (val, ln), f"index={idx}")

    def test_type_frame_width(self):
        got = extract_type_frame_width(_LINES)
        self.assertEqual(len(got["values"]), 88)
        for idx, val, ln in ((0, 32, 590), (4, 45, 594), (9, 104, 599), (87, 36, 677)):
            item = got["values"][idx]
            self.assertEqual((item["value"], item["line"]), (val, ln), f"index={idx}")

    def test_anim_offset_table(self):
        got = extract_anim_offset_table(_LINES)
        self.assertEqual(len(got["rows"]), 11)
        self.assertEqual(got["rows"][0], {"index": 0, "value": [0], "line": 486})
        self.assertEqual(got["rows"][1], {"index": 1, "value": [0, 1, 0, 2], "line": 487})
        self.assertEqual(got["rows"][10], {"index": 10, "value": [2, 3, 4], "line": 496})

    def test_containers_450_451_erratum(self):
        """名序值行=451、张数表值行=453（survey「450/451」条目的机械实锤勘误）。"""
        names = extract_resource_names(_LINES)
        counts = extract_resource_image_counts(_LINES)
        self.assertEqual([v["value"] for v in names["values"]], [
            "sflogo", "mapbg", "map", "actor", "sptmap", "sptprop", "sptarm", "sptenemy1",
            "ui", "xtq", "menu", "intro", "face", "sptenemy2", "end", "load"])
        self.assertTrue(all(v["line"] == 451 for v in names["values"]))
        self.assertEqual([v["value"] for v in counts["values"]],
                         [8, 1, 12, 4, 13, 23, 10, 20, 25, 6, 2, 2, 12, 20, 1, 2])
        self.assertTrue(all(v["line"] == 453 for v in counts["values"]))

    def test_seg67(self):
        got = extract_seg67_pieces(_LINES)
        self.assertEqual(len(got["rows"]), 9)
        self.assertEqual(got["rows"][0], {"index": 0, "value": [0, 0, 0, 33, 56], "line": 683})
        self.assertEqual(got["rows"][8], {"index": 8, "value": [4, 28, 0, 24, 26], "line": 691})

    def test_size_dispatch(self):
        got = extract_size_dispatch(_LINES)
        arms = {lbl: arm for arm in got["arms"] for lbl in arm["labels"]}
        a69 = {s["field"]: s["value"] for s in arms[69]["stmts"] if s["stmt"] == "assign"}
        a72 = {s["field"]: s["value"] for s in arms[72]["stmts"] if s["stmt"] == "assign"}
        dflt = {s["field"]: s["value"] for s in arms["default"]["stmts"]
                if s["stmt"] == "assign"}
        fb = {s["field"]: s["value"] for s in got["fallback_stmts"] if s["stmt"] == "assign"}
        self.assertEqual(a69, {"sprite_w": 96, "sprite_h": 32})
        self.assertEqual(a72, {"sprite_w": 32, "sprite_h": 45})
        self.assertEqual(dflt, {"sprite_w": "this.typeFrameWidth[var1]",
                                "sprite_h": "var3.getHeight()"})
        self.assertEqual(fb, {"sprite_w": 32, "sprite_h": 32})

    def test_mark_removed(self):
        got = extract_mark_removed(_LINES)
        self.assertEqual(got["state"], 1)
        arms = {tuple(sorted(a["labels"])): a for a in got["arms"]}
        cat1 = arms[(1,)]
        sub = {tuple(sorted(s["labels"])): s for s in cat1["switch"]["arms"]}
        f_123 = {s["field"]: s["value"] for s in sub[(1, 2, 3)]["stmts"] if s["stmt"] == "assign"}
        f_11 = {s["field"]: s["value"] for s in sub[(11,)]["stmts"] if s["stmt"] == "assign"}
        self.assertEqual(f_123, {"sprite_w": 45, "sprite_h": 56, "anim_row": 6, "anim_frame": 0})
        self.assertEqual(f_11, {"sprite_w": 45, "sprite_h": 57, "anim_row": 2, "anim_frame": 0})
        f_24 = {s["field"]: s["value"] for s in arms[(2, 4)]["stmts"] if s["stmt"] == "assign"}
        self.assertEqual(f_24, {"sprite_w": 45, "sprite_h": 56, "anim_row": 10, "anim_frame": 0})
        f_8 = {s["field"]: s["value"] for s in arms[(8, 16, 32)]["stmts"] if s["stmt"] == "assign"}
        self.assertEqual(f_8, {"sprite_w": 27, "sprite_h": 29, "anim_row": 6, "anim_frame": 0})
        self.assertIn("buildMinimap", [c["callee"] for c in cat1["stmts"] if c["stmt"] == "call"])

    def test_mark_des_removed(self):
        got = extract_mark_des_removed(_LINES)
        self.assertEqual(got["state"], 2)
        arms = {tuple(sorted(a["labels"])): a for a in got["arms"]}
        f = {s["field"]: s["value"] for s in arms[(2, 8, 16, 32)]["stmts"] if s["stmt"] == "assign"}
        self.assertEqual(f, {"sprite_w": 27, "sprite_h": 29, "anim_row": 8, "anim_frame": 0})
        self.assertEqual([s["stmt"] for s in arms[(1,)]["stmts"]], ["return"])

    def test_fixture_is_migration_source(self):
        """迁入源核对：type_width_px.txt == typeFrameWidth（88 项）。"""
        self.assertTrue(FIXTURE_WIDTH.exists())
        fix = [int(x) for x in FIXTURE_WIDTH.read_text(encoding="utf-8").split()]
        ext = [v["value"] for v in extract_type_frame_width(_LINES)["values"]]
        self.assertEqual(fix, ext)


# ---------------------------------------------------------------- ② ③ 表 + 等值

class TestTablesExist(unittest.TestCase):
    def test_tables_exist(self):
        for name in ("sprite_defs.json", "anim_defs.json", "sheet_index.json"):
            self.assertTrue((TABLES / name).exists(), f"表缺: data/tables/{name}")


class TestEquivalence(unittest.TestCase):
    def test_equivalence_zero_mismatches(self):
        """主判据：表值==机械提取值，错位逐行报。"""
        if not (TABLES / "sprite_defs.json").exists():
            self.fail("表缺 data/tables/（红=未回填）")
        errors = Comparator().run_all()
        self.assertFalse(errors, "\n".join(str(e) for e in errors))

    def test_core_triple_counts(self):
        sprite = _load("sprite_defs.json")
        anim = _load("anim_defs.json")
        sheet = _load("sheet_index.json")
        n_anim = sum(len(_anchors(t, "anim_row")) for t in sprite["sprite_defs"])
        n_width = sum(len(_anchors(t, "type_frame_width")) for t in sprite["sprite_defs"])
        n_rows = sum(len(_anchors(r, "frames")) for r in anim["anim_defs"])
        n_names = len(_anchors(sheet, "name"))
        n_counts = len(_anchors(sheet, "count"))
        self.assertEqual((n_anim, n_width, n_rows, n_names, n_counts),
                         (89, 88, 11, 16, 16))
        # 三键三元组形：{index, value:{事实名: 值}, src}
        for t in sprite["sprite_defs"][:3]:
            for a in t["evidence"]:
                self.assertEqual(set(a), {"index", "value", "src"})


# ---------------------------------------------------------------- ④ 规则形抽样

class TestRuleShapes(unittest.TestCase):
    def test_transition_rules_spot_check(self):
        sprite = _load("sprite_defs.json")
        rules = {r["on"]: r for r in sprite["transition_overrides"]}
        mr = rules["mark_removed"]
        self.assertEqual(mr["state"], 1)
        cat1 = mr["by_category"]["1"]
        self.assertEqual(cat1["by_type"]["1|2|3"],
                         {"sprite_w": 45, "sprite_h": 56, "anim_row": 6, "anim_frame": 0})
        self.assertEqual(cat1["by_type"]["11"],
                         {"sprite_w": 45, "sprite_h": 57, "anim_row": 2, "anim_frame": 0})
        self.assertIs(cat1["build_minimap"], True)
        self.assertEqual(mr["by_category"]["2|4"],
                         {"sprite_w": 45, "sprite_h": 56, "anim_row": 10, "anim_frame": 0})
        self.assertEqual(mr["by_category"]["8|16|32"],
                         {"sprite_w": 27, "sprite_h": 29, "anim_row": 6, "anim_frame": 0})
        md = rules["mark_des_removed"]
        self.assertEqual(md["state"], 2)
        self.assertEqual(md["by_category"]["2|8|16|32"],
                         {"sprite_w": 27, "sprite_h": 29, "anim_row": 8, "anim_frame": 0})
        self.assertEqual(md["by_category"]["1"], {"noop": True, "returns": True})
        br = rules["bump_reveal"]["by_type"]
        self.assertEqual(br["6"]["anim_row"], 9)
        self.assertEqual(br["6"]["state"], 2)
        self.assertEqual(br["12"]["type_id"], 11)
        self.assertEqual(br["12"]["anim_row"], 9)

    def test_size_dispatch_spot_check(self):
        sprite = _load("sprite_defs.json")
        sd = sprite["size_dispatch"]
        self.assertEqual((sd["by_type"]["69"]["frame_w"], sd["by_type"]["69"]["frame_h"]),
                         (96, 32))
        self.assertEqual((sd["by_type"]["72"]["frame_w"], sd["by_type"]["72"]["frame_h"]),
                         (32, 45))
        self.assertEqual(sd["by_type"]["67"]["load_packed_images"], 9)
        self.assertEqual((sd["fallback"]["frame_w"], sd["fallback"]["frame_h"]), (32, 32))
        self.assertEqual(sd["default"]["frame_w_from"], "type_frame_width")
        self.assertEqual(sd["default"]["frame_h_from"], "image_height")

    def test_density_and_schema_fields(self):
        sprite = _load("sprite_defs.json")
        anim = _load("anim_defs.json")
        sheet = _load("sheet_index.json")
        for doc in (sprite, anim, sheet):
            self.assertEqual(doc["schema_version"], 2, "schema 版本字段=2（解析器 SCHEMA_VERSION）")
        for t in sprite["sprite_defs"]:
            self.assertEqual(t["density"], 1)
        self.assertTrue(sprite["quirks_in_code"],
                        "quirks_in_code 留痕槽不得为空（§2.1 禁静默留码）")
        # anim_defs 全局节拍（a.java:6833/6850-6851）
        for r in anim["anim_defs"]:
            self.assertEqual(r["step_ticks"], 2)
            self.assertEqual(r["loop_mode"], "cycle")
        # sheet_index 扁平形（§2.1）：16 名序字符串 + 张数表 + slots 对象
        self.assertEqual(len(sheet["containers"]), 16)
        self.assertTrue(all(isinstance(c, str) for c in sheet["containers"]))
        self.assertEqual(sheet["container_counts"],
                         [8, 1, 12, 4, 13, 23, 10, 20, 25, 6, 2, 2, 12, 20, 1, 2])
        self.assertIsInstance(sheet["slots"], dict)


# ---------------------------------------------------------------- ⑤ 确定性

class TestDeterminism(unittest.TestCase):
    def test_check_cli_same_hash(self):
        if not (TABLES / "sprite_defs.json").exists():
            self.fail("表缺 data/tables/（红=未回填）")
        outs = []
        for flag in ("check", "--check"):
            r = subprocess.run([sys.executable, str(TOOLS_DIR / "cli.py"), flag],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
            self.assertIn("[check] ✓ 确定性复跑同哈希", r.stdout)
            outs.append([l for l in r.stdout.splitlines()
                         if l.startswith("[check] ✓ ") and ".json" in l])
        self.assertEqual(outs[0], outs[1], "两次 --check 哈希输出必须一致")


if __name__ == "__main__":
    unittest.main(verbosity=2)
