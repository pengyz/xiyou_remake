# 重构行为面覆盖矩阵（refactor-plan §7 条件 3/4 实施前置）

> 角色：porter-arms（第 0 步金样基线批）。本文 = **行为面 → 测试名** 对照矩阵：
> 每面（槽三禁令/条件取参/恢复臂 quirk/UNKNOWN 停摆）逐项列**钉住它的测试**；
> 无测试的面标 **缺口**（第 1/2 步动布局前必须补 RED 测）。
> 配套：[refactor-plan.md](refactor-plan.md) §7 六条件、
> [refactor-plan-review.md](refactor-plan-review.md) A1-A6、
> 金样基线 `data/status/baselines/state-dump-golden.json`（55 场景）。

## 0. 主判据三件（refactor-plan-review A1 换枪）

| 判据 | 落点 | 状态 |
|---|---|---|
| ① state_dump 金样逐字节等价 | `data/status/baselines/state-dump-golden.json` + `python3 tools/state_baseline.py --check` | ✅ 55 场景 HEAD 基线已固化（含 prologue 长窗 9224 拍） |
| ② statediff 名集**相等**断言 | `tools/statediff.py`（漏字段=红，tick 集不等=红） | ✅ 2026-10-16 修复（原交集口径静默绿） |
| ③ 行为面覆盖矩阵 | 本文 | ✅ 框架立枪（缺口见各节） |
| （附证）44/44 A==B==C | `gates/cli.py scenarios`（Java 管线未动） | 降级附证，不进重构验证列 |

**行序冻结声明（A2）**：`state_dump.rs dump_scene` 现行 push 序即金样字节序——
Entity/TraceState 迁移**适配其序，禁按 FLD 序重排**；FLD 行=Java 反射序（布局迁移
天然不影响），两机制分列不混谈。契约源 = `python3 reference/oracle/fields.py
--contract`（411 真槽 + 49 dump 项全映射，生成式非手抄）。

## 1. 槽表三禁令（Entity 迁移红线，refactor-plan 条件 3）

| # | 行为面 | 钉住测试 | 状态 |
|---|---|---|---|
| 1.1 | 槽编码 **1-based u8**（`entity.rs` spawn `(idx+1)` 语义；槽表存 idx+1） | `entity_golden::spawn_registers_cell_slot_and_stair_anchor`、`entity_golden::relocate_slot_moves_registration_from_start_cell` | ✅ |
| 1.2 | **槽位永不复用**（entityCount 单调 append-only 至换层重建；swap_remove/重编号=静默坏槽） | `entity_golden::remove_marks_solid_and_lookup_skips`（removed 留置形态）、`fixture_frames::fx_bug_ces70_slot_frames_match`（重复登记留置实证） | ⚠️ **缺口**：无"remove 后 entityCount 不回退/槽不复用"的显式断言——第 1 步动布局前补 RED 测 |
| 1.3 | **六写点一致性收口**（spawn/removeGroup/fix_slot/detach/relocate/m_108 → 双写点同一收口函数） | `spawn_registers_cell_slot_and_stair_anchor`（spawn）、`remove_all_of_type_matches_des_negative`（removeGroup 族）、`relocate_slot_moves_registration_from_start_cell`（m_108） | ⚠️ **缺口**：fix_slot/detach 两写点无独立测（现由帧对拍间接覆盖）——收口重构前补 |
| 1.4 | 恢复臂 **entityVisible 按循环下标回填 quirk**（a.java:9638，struct 化禁顺手修正） | `fixture_frames::fx_bug_ces70_slot_frames_match`（恢复路径槽序钉） | ✅（帧级）；⚠️ 下标语义无字段级断言 |
| 1.5 | **sort_by_y 平局序**（严格小于 ⇒ 等值保序；禁 unstable sort） | 无（仅帧对拍间接覆盖；`entity::sort_by_y` 无任何直接测） | ❌ **缺口**：补"5 实体同 y 保序"RED 测后方可动排序实现 |

## 2. 条件取参清单（游标契约，refactor-plan 条件 4）

| # | 行为面 | 钉住测试 | 状态 |
|---|---|---|---|
| 2.1 | MOV var55>0 **续取 3 参**（type/cx/cy + x/y 共 5 参） | `preset_cases::preset_floor2_mov_frames_match`、`preset_floor40_mov_frames_match`、`fixture_frames::fx-scriptwalk-floor3`（生产面） | ✅ |
| 2.2 | MOV var55=0 三参（玩家行）+ findPath 门 | `script_golden::mov_player_path_gates_walk_phase` | ✅ |
| 2.3 | MOV-miss **尾递归**（a.java:8302 实体未命中 ⇒ 递归取下一条） | 无 | ❌ **缺口**：补 RED 测（游标位移断言）后动命令模式 |
| 2.4 | DES **负参分支**（负=全型清除 / 正=定点移除） | `entity_golden::remove_all_of_type_matches_des_negative` + `scene.rs::remove_des` 组 | ✅ |
| 2.5 | GUT **尾递归**（a.java:8437 `loadLevelScript(n)` 链） | `script_golden::gut_flags_and_jumps` | ✅ |
| 2.6 | GIN 尾递归（a.java:8440 executeScriptInstruction 自调） | `script_golden::gin_tail_recurses_into_next_instruction` | ✅ |
| 2.7 | TAK 游标推进 + 页尽续走 | `script_golden::tak_page_advance_and_exhaust_resumes_script` | ✅ |

## 3. UNKNOWN 停摆（R-TODO-1b，命令模式红线）

| # | 行为面 | 钉住测试 | 状态 |
|---|---|---|---|
| 3.1 | **真未知 op 静默停摆**（游标不动/不改状态；禁"解析失败仍消费 token"） | `scene.rs::unknown_op_silently_stalls_cursor` | ✅ |
| 3.2 | MVS 族（参数低字节配对/误配对零效/双路堆叠/多对钉格/无怪 82 点） | `mvs_pairs_by_param_low_byte_transfers_and_consumes`、`mvs_mismatched_pair_key_leaves_world_untouched`、`mvs_two_way_transfer_piles_monsters_at_first_slot_cell`、`mvs_multi_pair_pinning_scans_and_targets_each_pair_cell`、`mvs_pair_without_monsters_leaves_82_points` | ✅ 5 测全 |

## 4. 恢复臂 quirk（save/load 往返）

| # | 行为面 | 钉住测试 | 状态 |
|---|---|---|---|
| 4.1 | save blob 布局逐字节（bool×4 + N + N×i32 + i32×3 头） | `scene.rs::save_blob_layout_matches_java_and_roundtrips` | ✅ |
| 4.2 | entityCount==0 ⇒ 存 null（a.java:9383-9384） | `scene.rs::save_floor_state_null_only_when_entity_table_empty` | ✅ |
| 4.3 | 读档 scriptLineFlags 前 scriptLineCount 项 + floor2 特例（a.java:9285-9291/9643-9646） | `scene.rs::save_roundtrip_restores_equipment`（装备面）+ `fixture_frames::fx-bug-ces70-slot` | ⚠️ 特例路径无独立测（帧级间接） |
| 4.4 | addItemToItemStack 写 itemUseTargetType（a.java:7732） | `scene.rs::add_item_to_stack_writes_item_use_target_type` | ✅ |
| 4.5 | 槽表 1-based 回读/实体可见性下标回填 | 见 1.1/1.4 | ⚠️ 同上 |

## 5. 双面分列（harness 面 vs 生产面，DC-9 漂移前科）

| 面 | 覆盖 | 状态 |
|---|---|---|
| harness 面（script.rs ScriptEngine） | `script_golden` 全族（2.1-2.7/3.x 单测） | ✅ |
| 生产面（execute_script_instruction） | fx 夹具 23 场景 + preset_cases 35 场景 + gameplay-floor1（帧/状态面） | ✅ |
| **桥漏道具加成类双面漂移** | 无专测 | ❌ **缺口**：第 2 步命令模式迁移时补"同脚本 harness/生产双跑一致"测 |

## 6. 缺口汇总（第 1/2 步动布局前必补 RED 测）

1. **1.2 槽不复用显式断言**（remove 后 entityCount 单调 + removed 槽留置形态字段级）
2. **1.3 fix_slot/detach 独立测**（收口重构的对照源）
3. **1.5 sort_by_y 平局保序**（5 实体同 y 序列）
4. **2.3 MOV-miss 尾递归**（游标位移断言）
5. **4.3 恢复特例路径独立测**（floor2 scriptLineFlags[26]）
6. **5.x 双面同脚本一致测**（DC-9 类漂移防线）

> 补测优先级按重构步序：第 1 步（Entity 迁移）前补 1.2/1.3/1.5；第 2 步（命令模式）
> 前补 2.3/5.x。4.3 可随特例批（restore_rules 配置化）一并落。
