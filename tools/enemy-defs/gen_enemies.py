#!/usr/bin/env python3
"""gen_enemies.py — enemies.json 生成器（DDDSA P1-6；V2 逐值锚重生机制）。

取数根（deobf a.java 逐字迁——禁手抄）：
① enemyBaseHp/Atk/Def/Gold（a.java:1044-1130 区 4×35 行）；
② difficultyMultipliers（a.java:2236 区 5 项=全局表非 per-type）；
③ traitFlagsByType（a.java:6266-6267 稀疏赋值：49/53/74=1、69=2——行为键）。
输出：data/tables/enemies.json（35 行 id=41..75=SoA 横索引 t-41 的型域；
行数对账=源表长 35——2026-11-06 实测校正 lead 89 数=域宽误）。确定性输出。
"""
from __future__ import annotations
import json, re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
AOBJ = ROOT / "reference/src/deobf/a.java"
OUT = ROOT / "data/tables/enemies.json"


def java_int_array(src: str, name: str) -> list[int]:
    i = src.index(f"this.{name} = new int[]")
    j = src.find("{", i)
    depth = 0; k = j
    while True:
        depth += src[k].count("{") - src[k].count("}")
        if depth == 0 and k > j:
            break
        k += 1
    return [int(x) for x in re.findall(r"\d+", src[j:k])]


def main() -> int:
    src = AOBJ.read_text(encoding="utf-8", errors="replace")
    hp = java_int_array(src, "enemyBaseHp")
    atk = java_int_array(src, "enemyBaseAtk")
    deff = java_int_array(src, "enemyBaseDef")
    gold = java_int_array(src, "enemyBaseGold")
    mult = java_int_array(src, "difficultyMultipliers")
    n = len(hp)
    assert len(atk) == len(deff) == len(gold) == n, "四表长度不齐"
    # traitFlagsByType（a.java:6266-6267 稀疏赋值逐字）
    traits = {t: 1 for t in (49, 53, 74)}
    traits[69] = 2
    rows = []
    for i in range(n):
        tid = 41 + i
        rows.append({
            "id": tid,
            "hp": hp[i],
            "atk": atk[i],
            "def": deff[i],
            "gold": gold[i],
            "behavior": {"trait_flags": traits.get(tid, 0)},
            "evidence": {
                "hp": f"a.java:1044-1130 enemyBaseHp[{i}]",
                "atk": f"enemyBaseAtk[{i}]",
                "def": f"enemyBaseDef[{i}]",
                "gold": f"enemyBaseGold[{i}]",
                "behavior": "a.java:6266-6267 traitFlagsByType（稀疏：49/53/74=1、69=2）",
            },
        })
    doc = {
        "schema_version": 2,
        "note": "DDDSA P1-6 EnemyDef 聚合（per-type 属性束；生成器 tools/enemy-defs/gen_enemies.py 逐值迁）；difficulty_multipliers=全局表非 per-type（a.java:2236）",
        "difficulty_multipliers": mult,
        "rows": rows,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(doc, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"enemies.json → {OUT}（{len(rows)} 行；difficulty {len(mult)} 项）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
