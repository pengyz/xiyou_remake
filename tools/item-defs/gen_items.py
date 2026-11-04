#!/usr/bin/env python3
"""gen_items.py — items.json 生成器（DDDSA P1-1；V2 逐值锚重生机制）。

取数根（五类 SoA 源——禁手抄，值逐字迁）：
① name：crates/game-core/tests/fixtures/object_type_names.txt（行序=型号）
② desc：reference/src/deobf/a.java itemDescriptions（906-929）+equipDescriptions
③ effect/params：crates/game-core/src/item_effect.rs ITEM_EFFECTS+pickup dispatch
   语义（a.java 7537-7648 锚）——本表=语义映射（生成式单一源）
④ equip_types/equip_bonuses：crates/game-core/src/scene_persist.rs 真源行
⑤ icon：entity 图引用（sprite_defs 外键——icon={"entity_type": t}）

输出：data/tables/items.json（id<1000 保真表行；行序=id 升序）。
确定性：同输入同输出（无时间戳/无随机）。
"""
from __future__ import annotations
import json, re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
NAMES = ROOT / "crates/game-core/tests/fixtures/object_type_names.txt"
AOBJ = ROOT / "reference/src/deobf/a.java"
OUT = ROOT / "data/tables/items.json"


def java_string_array(src: str, marker: str) -> list[str]:
    """抽 deobf a.java 中 `marker` 后第一个 String[] 字面量的逐串值。"""
    i = src.index(marker)
    j = src.index("{", i)
    depth = 0; k = j
    while True:
        depth += src[k].count("{") - src[k].count("}")
        if depth == 0 and k > j:
            break
        k += 1
    body = src[j:k + 1]
    return re.findall(r'"((?:[^"\\]|\\.)*)"', body)


def main() -> int:
    src = AOBJ.read_text(encoding="utf-8", errors="replace")
    names = [l.strip() for l in NAMES.read_text(encoding="utf-8").splitlines()]
    descs = java_string_array(src, "this.itemDescriptions = new String[]")
    equips = java_string_array(src, "equipDescriptions")
    # equip_types/bonuses 真源行（scene_persist 硬编码——行锚 2026-11-04 核）
    sp = (ROOT / "crates/game-core/src/scene_persist.rs").read_text(encoding="utf-8")
    et = re.search(r"equip_types: \[([^\]]+)\]", sp).group(1)
    eb = re.search(r"equip_bonuses: \[([^\]]+)\]", sp).group(1)
    equip_types = [int(x) for x in et.split(",")]
    equip_bonuses = [int(x) for x in eb.split(",")]

    def desc_idx(t: int) -> int:
        return 20 if t == 85 else 21 if t == 86 else t - 13

    def tier_of(t: int) -> int:
        return equip_types.index(t)

    # effect/params 语义映射（③——ITEM_EFFECTS 九 case+a.java pickup dispatch 全组）
    EFFECT = {
        13: ("OpenPanel", {"panel": "EnemyInfo", "page": 0}, "a.java:7794-7797"),
        14: ("OpenPanel", {"panel": "RecordBrowser", "page": 0}, "a.java:7798-7800"),
        15: ("TutorialSeenStack", {}, "a.java:7538-7539 case15 落穿入栈"),
        16: ("RemoveAdjacent", {"target": 10, "msg": "你必须面对三昧真火再使用它。"}, "a.java:7807-7811"),
        17: ("RemoveAdjacent", {"target": 11, "msg": "你必须面对一堵墙使用"}, "a.java:7807-7811"),
        18: ("RemoveAllType", {"target": 11}, "a.java:7812-7825"),
        19: ("Heal", {"mul": 74, "div": 10, "msg_fmt": "增加了{}血量"}, "a.java:7826-7831"),
        20: ("MirrorTeleport", {}, "a.java:7832-7850"),
        21: ("ChangeFloorUp", {}, "a.java:7863-7866"),
        22: ("Passive", {"kind": "AntiNarcissusDamage"}, "描述面：对自恋神仙伤害加倍（道具栏被动）"),
        23: ("Passive", {"kind": "BindingRope"}, "描述面：捆仙绳（道具栏被动）"),
        24: ("Passive", {"kind": "DamageReflect"}, "描述面：伤害反弹类被动"),
        25: ("Passive", {"kind": "LuckyCoinGoldDouble"}, "a.java killAdjacent 金币加倍=find_item(25)"),
        26: ("Key", {"color": "yellow"}, "a.java:7556-7558 yellowKeyCount++"),
        27: ("Key", {"color": "red"}, "a.java:7559-7561 redKeyCount++"),
        28: ("Key", {"color": "blue"}, "a.java:7562-7564 blueKeyCount++"),
        29: ("StatUp", {"stat": "atk"}, "a.java:7565-7572 playerAtk++/floorTier"),
        30: ("StatUp", {"stat": "def"}, "a.java:7573-7580 playerDef++/floorTier"),
        31: ("HpUp", {"mul": 50}, "a.java:7581-7584 applyHpDelta(50*tier)"),
        32: ("HpUp", {"mul": 200}, "a.java:7585-7587 applyHpDelta(200*tier)（落穿空标至 return 无害）"),
        85: ("RemoveAllType", {"target": 1}, "a.java:7873-7884 全表 markEntityRemoved(型1)"),
        86: ("KillAdjacent4", {}, "a.java:7876-7886 四邻 killAdjacentAt"),
    }

    rows = []
    # 可入栈 22 件：13..32+85/86
    for t in list(range(13, 33)) + [85, 86]:
        key, params, anchor = EFFECT[t]
        d = descs[desc_idx(t)]
        rows.append({
            "id": t, "name": names[t], "icon": {"entity_type": t},
            "desc": d, "pickup_msg": d,
            "effect": {"key": key, "params": params},
            "evidence": {"name": f"object_type_names.txt:{t+1}", "desc": f"a.java:906-929 idx{desc_idx(t)}", "effect": anchor},
        })
    # 装备 10 件：33..40+79/80（tier 档描述）
    for t in [33, 34, 35, 36, 37, 38, 39, 40, 79, 80]:
        tier = tier_of(t)
        rows.append({
            "id": t, "name": names[t], "icon": {"entity_type": t},
            "desc": equips[tier] if tier < len(equips) else "",
            "pickup_msg": equips[tier] if tier < len(equips) else "",
            "effect": {"key": "Equip", "params": {"tier": tier, "bonus": equip_bonuses[tier]}},
            "evidence": {"name": f"object_type_names.txt:{t+1}", "desc": f"equipDescriptions[{tier}]（a.java:7633 区）", "effect": f"a.java:7632-7648 tryEquip（equip_types[{tier}]）"},
        })
    rows.sort(key=lambda r: r["id"])
    doc = {
        "schema_version": 2,
        "note": "DDDSA P1-1 ItemDef 聚合（id<1000=保真表）；生成器 tools/item-defs/gen_items.py（禁手抄逐值迁）",
        "rows": rows,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(doc, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"items.json → {OUT}（{len(rows)} 行）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
