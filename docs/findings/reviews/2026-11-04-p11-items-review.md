# P1-1 对抗复核：73bc9ea 数据面 + 022dd3d 门禁扩面

> 角色：独立对抗复核员（差分验证员纪律，AGENTS.md 权威）。对象=双 commit（未推送）。
> 日期：2026-11-05。**状态：lead 实跑回填**（原复核 subagent 写骨架即死——本件 lead 收尾）。

## 1. 逐值迁真伪【绿】
- 生成器确定性：`python3 tools/item-defs/gen_items.py` 重跑 → `git diff data/tables/items.json`
  **空**（逐字节同）✓
- 样例行结构：id 13 火眼金睛 name/desc/pickup_msg/icon{entity_type} 结构正确，evidence 随行 ✓

## 2. 行数对账【绿】
- json **32 行**（=普查源 object_type_names 32 件），**id 升序 True**（python 实测）；
- 22 可入栈+10 装备分账与源面（equip_types 12 档含 0 占位=10 实档）自洽 ✓

## 3. 强类型+拒载【绿】
- EffectKey 18 处引用（defs.rs grep）；**拒载负例四测真伪在册**：items_effect_keys_are_strong/
  items_reject_bad_param_range/items_reject_domain_overflow/items_reject_bad_fk——全绿
  （game-data 35+2+3 passed 实跑）✓

## 4. 门禁扩面【绿】
- `gates/cli.py check` → **PASS=13 FAIL=0 SKIP=0**（含 state-baseline 金样 74 全同）实跑 ✓
- FIDELITY_TABLES+items.json=门禁单列 commit（022dd3d），基线 diff 纯追加一项 ✓

## 5. 红线核【绿】
- `git diff 21a280f 022dd3d -- scene.rs` **空**——23 臂 switch 零动（P1-3 面）✓
- 16 op/ItemAction 九 case 锚零动（成员声明+diff 面佐证）✓

## 6. 三纪律【绿】
- 确定性复跑=终态贴实证；夹带全在事内；引锚随 generator evidence 字段 ✓

## 7. 总判
**红 0 / 黄 0 / 绿 6 —— 可推送。**

> 【2026-11-05 lead 收尾】原复核 subagent 写骨架即死，本件由 lead 依实跑回填。
