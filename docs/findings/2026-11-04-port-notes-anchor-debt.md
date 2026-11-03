# PORT_NOTES/校验锚失效预存债清单（2026-11-04 销账建档·批2 复核红2更正版）

> DDDSA P1-a2 批2 账②。**算术（实测口径）**：原 ⚠ 17 = 销 2 + 登 14（锚失效）
> + 1（TRIAGE 不符另类）。销 2=addItemToItemStack（`self.overlay_kind_item`
> →`self.menu.set_overlay_kind_item(t)` 锚串实改）+initFloorStates
> （`scene.saved_floor_states`→`scene.persist.` 前缀锚串实改）——此二条已从
> ⚠ 清单消失（grep 实证）。**行 3 归登记**（仅加括注未使锚匹配=仍报⚠——
> 复核红2 裁定）。
> **异族四类**（黄③更正：原"同族多锚"标法错——实为异族；Q-2 族分自洽更正：
> TRIAGE 第 2 条=「修正暴露」不符类拆出另账）：**PORT_NOTES 8 / TRIAGE 1 /
> YELLOW_NOTES 4 / ARM_GAPS 1 = 登 14（锚失效）+1 另类（TRIAGE 不符）**。

## 登记面（14 锚失效实名——⚠ 清单 grep 原文逐条）

| # | 族 | 条目 | 失效锚（原文） | 处置 |
|---|---|---|---|---|
| 1 | PORT_NOTES | initFloorStates | `saved_floor_states: vec![None; 56]` | 登记（括注已加，锚仍失配） |
| 2 | PORT_NOTES | easeCameraToTarget | `// m_103：ease 相机目标 = 目标格` | 登记 |
| 3 | PORT_NOTES | easeCameraToTarget | `// m_103：ease 目标 = SEE 目标格` | 登记 |
| 4 | PORT_NOTES | m_120 | `fn save_floor_state(&mut self)` | 登记（迁 scene_persist.rs） |
| 5 | PORT_NOTES | restoreFloorState | `u16::from_be_bytes([d[*p], d[*p + 1]])` | 登记 |
| 6 | PORT_NOTES | resetNewGameState | `difficulty_index: 0` | 登记 |
| 7 | PORT_NOTES | saveGlobalProgress | ported 缺 Rust 文件锚 | 登记 |
| 8 | PORT_NOTES | startLoadTask | `pub fn new(target: i32, call_m000: bool, steps: &[i8]) -> LoadProgress` | 登记 |
| 9 | TRIAGE | saveGlobalProgress | `saveGlobalProgress（RMS 侧，无像素效应；game-core 无 I/O）` | 登记 |
| 10 | YELLOW_NOTES | m_107 | `fn m_107` | 登记 |
| 11 | YELLOW_NOTES | m_107 | `mvs_pairs_by_param_low_byte_transfers_and_consumes` | 登记 |
| 12 | YELLOW_NOTES | m_107 | `mvs_mismatched_pair_key_leaves_world_untouched` | 登记 |
| 13 | YELLOW_NOTES | m_107 | `mvs_two_way_transfer_piles_monsters_at_first_slot_cell` | 登记 |
| 14 | ARM_GAPS | interactWithCell | `_ => {} // 76/82：后续批次` | 登记 |

## 附带预存债（批3 复核随批登记）

| # | 位点 | 现象 | 处置 |
|---|---|---|---|
| A1 | scene.rs:4257-4259 三处 | 锚漂移（复核件点名——具体锚值待 sed 实文核） | 登记（下批引锚核对时逐条销） |

## 另类 1（非锚失效）

- TRIAGE「修正暴露」与机械复算不符：漏登 ['m_101','m_134','showNotify']；
  多登 ['closeRecordStoreAndGc','deleteRecordStoreQuietly']——登记（校验器
  口径差待裁）。

**登记口径**：锚失效=字面锚 grep 失配（task-15/终章迁移连带）——非行为面。
销账随 coverage 校验器锚文件多路径支持批。排除面口径全列=史料/哈希/原地
注记/伪命中。
