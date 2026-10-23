# 触屏原生+回撤 T4·终检复核报告（hd-final-verifier / remake-verifier，task-13）

> 2026-10-23。独立终检（全程亲做，未转包）：只看 diff + 证据 + 实跑结果。
> 复核对象 = T1 `2f0b87a`（undo_journal LIFO 四型笔+清栈）→ T2 `b2dc630`（pathfind 策略层/
> RouteTracker/挂起续走/打断终版）→ T3 `f4ce171`（shell/common/touch.rs+undo_ui.rs、
> win+android 同构接线，R1-R3 裁定）。
> 依据 = [touch-native-undo-plan.md](../superpowers/plans/2026-10-23-touch-native-undo-plan.md)
> §0 裁定表 + §2 三型 delta 表 + §3 TDD 判据。纪律（remake-verifier）：禁静默放宽/
> 白名单/删用例；未贴出的运行结果不存在——§2 全为本机实跑。

## 1. 判据核销表

### 1.1 用户语义原样（lead 清单 1：走过去+战斗=2 笔，回撤①战斗复原②位置回起点）

| 判据 | 落点 | 在场证据（实测锚） | 核销 |
|---|---|---|---|
| 2 笔计数 | plan §3-3 | `undo_twice_combat_then_position_user_judge`：`journal.len()==2`（位移笔+战斗笔） | ✅ |
| 回撤①战斗复原逐字段 | 同上 | 同测：敌复活 `alive`+原血 `120`+型 `44` 不变；玩家 hp/gold/exp `80/50/10` 回滚；掉落 `(7,3)→(7,1)` 退回；**位置不动**（战斗笔不含位移） | ✅ |
| 回撤②位置回起点 | 同上 | 同测：`Move{from:(2,3),to:(5,3)}` 逆放 ⇒ `player_tile==(2,3)`，再撤 `Empty` | ✅ |
| LIFO 三笔组合逆放 | plan §3-3 | `undo_lifo_three_strokes_reverse_order` + `walk_route::suspend_combat_resume_lifo_with_journal`（段①→战斗→续走段② 三笔逐字段逆放：段②回敌格→战斗复原→段①回段起点） | ✅ |
| 战斗终结才入栈/中途不可撤 | plan §0 决定 a | `combat_mid_fight_not_undoable`（begin 后 undo=**Blocked** 且栈内无战斗笔）+ `undo_blocked_mid_combat_and_empty`（真 GameScene 面） | ✅ |
| 真 Scene 面逆放 | T3 undo_ui | `undo_move_restores_tile_and_count`：真 `GameScene`+真 `SceneUndo` 适配器 ⇒ 位置格+像素双面回起点 `(3,10)/(3<<5,10<<5)`、按钮计数同步减 | ✅ |

注：战斗复原逐字段断言在 T1 mock 面（`UndoTarget` 测试实现）；真 Scene 面的
`SceneUndo::revive_enemy`/`set_player_vitals` 写点经代码逐行核（ent.type_/state=0/
visible=true+hp 还原；hp/gold 两字段——exp 本作无字段已注记）与 mock 语义一致。

### 1.2 行为政策（lead 清单 2）

| 政策（plan §0） | 在场证据 | 核销 |
|---|---|---|
| 不可达=朝目标方向最远可达点停靠 | `astar_unreachable_docks_furthest_toward_target`（投影最远→距目标最近→行主序） | ✅ |
| 墙格=最近相邻停靠+探步入 | `wall_target_docks_nearest_adjacent_and_probes`（停靠 (2,1)+`ProbeStep{dir,into}`） | ✅ |
| 双模并存（移动按键保留+点击移动） | `touch::dual_mode_keys_and_taps_coexist`（tap 后按键不被压制）+ TouchMapper 一行不改（T3 diff） | ✅ |
| 任意输入皆打断（全通道） | `touch::interrupt_signal_on_every_channel`（触点/滑动/按键/撤销首动作 Interrupt）+ scene `t2_interrupt_all_channels_clear_buffer`（press_key/新 WalkTo/探步入） | ✅ |
| 探墙双通道等权 | `probe_step→try_step` 与方向键踏步同函数（scene.rs:1909/2754/4978/5432/5770 全走 `try_step`）+ tap 墙格=ProbeStep（touch.rs:156 注记+`tap_classifies_walk_wall_probe_and_undo`） | ✅ |
| 滑动一格撞墙停 | `touch::swipe_wall_stop_no_repeat`（一格/次、撞墙无后续步） | ✅ |
| 换层/存档/MOV 脚本格=清栈 | `clear_on_floor_change`/`clear_on_save`/`clear_on_script_event`/`clear_drops_pending_combat`（清栈连带取消战斗挂起） | ✅ |
| 对话/商店/背包消耗可撤 | `spend_stroke_undo_rolls_back_consumption` + `spend_kind_variants_are_distinct_strokes` | ✅ |
| 撤销钮常驻/N 步可视化/BACK 不动 | `undo_depth_reflects_journal_length`（len() 数据面）+ `undo_button_draws_and_zero_greys`（count=0 灰显）+ BACK=-7 语义未触（T3 diff） | ✅ |

### 1.3 plan §3 五判据 + classic 零漂移（lead 清单 4）

| 判据 | 在场证据 | 核销 |
|---|---|---|
| ① A* 四例+路径连续性 | `astar_straight_line`/`astar_detour_around_obstacle`/`astar_unreachable_docks_furthest_toward_target`/`astar_no_diagonal_steps`+`astar_path_continuity` | ✅ |
| ② WalkTo 链 | `t2_walk_to_feeds_buffer_and_arrives`（tap→buffer→终位）+ `t3_touch_actions_wire_to_scene`（WalkTo 喂缓冲→打断清缓冲→到位入栈→Undo 减栈） | ✅ |
| ③ 回撤序列（用户语义） | §1.1 全行 | ✅ |
| ④ 清栈判据 | §1.2 清栈四行 | ✅ |
| ⑤ classic 零漂移 | **实证四件**：(a) `git diff 2f0b87a~1..HEAD -- data/golden data/status reference/ original/` **全空**；(b) T1-T3 对 scene.rs **纯新增 209 行 0 删改**（key 链一行不动=字面成立：press_key 打断钩子为插入行，旧体零触碰；enums.rs 零变更）；(c) WalkTo/Undo 为新入口，金样/oracle 无调用方（grep 面）；(d) check `rust-tests` 全绿（含 oracle 金样 35/35 面）+ trace diff-trace 见 §2.2 | ✅ |

### 1.4 时序边界三组合穷举（lead 清单 5）

| 组合 | 断言在不在 | 证据/缺口 | 核销 |
|---|---|---|---|
| 打断中回撤 | **在（构造级）** | Dispatcher 全通道首动作 Interrupt（`vec![Interrupt, act]`，touch.rs:192）⇒ Undo 到达 core 前段已收于实际终点；+`move_stroke_interrupted_uses_actual_endpoint`/`interrupt_closes_segment_at_actual_endpoint`（打断=实际终点入栈）两侧单测 | ✅ |
| 回撤中寻路 | 补钉后**在**（黄-1 闭环） | `c6aff4a::undo_midway_then_new_walk_keeps_lifo`：2 笔→undo 1（旧栈剩 1）→新 WalkTo 起段→旧栈 LIFO 不变→新段终位入栈=2 笔→逐笔逆放到 Empty | ✅ |
| 战斗挂起中打断 | 补钉后**在**（黄-1 闭环） | `c6aff4a::interrupt_during_combat_pending_blocks_then_lifo`：段笔（打断=实际终点）→begin_combat 挂起中 undo==**Blocked**→finish_combat 入栈→LIFO 逆放（#1 战斗笔敌复活 120+位置不动、#2 段笔回起点）→Empty | ✅ |

### 1.5 R1-R3 裁定合理性（lead 清单 6）

| 裁定 | 内容 | 评价 | 核销 |
|---|---|---|---|
| R1 UndoTarget 落壳侧 | T3 core 零改红线 ⇒ `undo_ui::SceneUndo<'a>(&mut GameScene)` newtype 实现 trait（orphan 合规）；真写 scene 状态副本（格/像素/hp/gold/实体/钥匙/门） | 约束下合理；与 T1 trait 注"T2/T3 在 Scene 上实现"有架构张力，成本已注记（可整体迁 scene）——观察-1 | ✅（带观察） |
| R2 item_id 域 | 0/1/2=黄蓝红钥匙、其余=物品栈类型；上游无拾取/消耗笔生产者 ⇒ 域未定 | 务实；真生产者接线时一处 match 重映射（成本注记） | ✅ |
| R3 撤销钮仅 Map 态 | 他态行命中优先（BACK/pointer_hit 零扰动） | 合理（防抢行）；代价=菜单态不可回撤（可接受——菜单态本无回撤语义） | ✅ |

## 2. 实跑记录（本机 2026-10-23）

### 2.1 `gates/cli.py check`（12/12）

```
[✓ PASS] original-integrity / assets-manifest(entries=138) / reference-seed-integrity(8457 行)
[✓ PASS] knowledge-format(48) / docs-links / naming-ledger
[✓ PASS] rust-tests — cargo test --release 全绿
[✓ PASS] test-ratchet — total=562(基线527) 无回退；新增二进制待 ratchet-update:
         {'undo_journal(test)': 11, 'walk_route(test)': 9}
[✓ PASS] table-content-hash / table-evidence / state-machine-anchors / asset-truth-manifest
== 汇总 == PASS=12 FAIL=0 SKIP=0   （CHECK_EXIT=0）
```

ratchet 读法：存量零回退（562≥527 逐二进制无降）；T1 11+T2 9 新二进制待 lead
`ratchet-update` 入基线（T3 测试为 shell 各 lib 内联，计入 lib 二进制计数自然增长）。

### 2.2 `gates/cli.py trace`（L1-L3，复核人终跑）

```
[✓ PASS] diff-trace — cargo test -p game-oracle 全绿（exit 0）
（boot_replay 2 / fixture_frames 25 / gameplay_floor1 5 / l2_prologue 1 /
  preset_cases 35 / probe_floor2 1 / walk_replay_floor1 2 = L1-L3 全套件）
TRACE_EXIT=0
```

进程本 verifier 亲持（debug 档 preset_cases 90min CPU 量级）。与 check `rust-tests`
（workspace release 含 oracle 金样面）互证：WalkTo/Undo/回撤未进保真面对拍路径
（scene.rs 纯新增 209+/0-、金样/oracle 零变更）。

### 2.3 TDD 红证据抽查复现（3 commit，worktree 独立复现）

| commit | 声称 RED | 独立复现 | 判定 |
|---|---|---|---|
| `2f0b87a`（T1 undo_journal） | ①编译红 E0432 `game_core::undo_journal` 未找到；②行为红 5 例（undo 桩恒 Empty：`left: Empty right: Undone(Move(..))`/`right: Blocked`/期望拾取/战斗/消耗笔实际 Empty） | ✅ worktree@2f0b87a~1+后置测试 ⇒ `E0432: unresolved import game_core::undo_journal … could not find undo_journal` **逐字复现**；✅ 行为红类=桩化复现（把 undo() 改恒 Empty）⇒ **恰 5 例 FAILED**（combat_mid_fight/move_stroke_interrupted/spend_stroke/undo_lifo/undo_twice），`left: Empty right: Blocked`、`left: Empty right: Undone(Move(MoveStroke{from:(2,3),to:(3,3)}))` 与自述形态逐字吻合 | ✅ 3/3 |
| `b2dc630`（T2 寻路） | 编译红 E0432 `plan_route/ProbeStep/RoutePlan/RouteTracker` | ✅ worktree@b2dc630~1+后置 walk_route.rs ⇒ `E0432: unresolved imports game_core::pathfind::{plan_route, ProbeStep, RoutePlan, RouteTracker}` **逐字复现** | ✅ |
| `f4ce171`（T3 触屏） | tests-only 落盘红 E0425/E0433 全 API 缺失 | ✅ worktree@b2dc630+API 面探针 ⇒ `E0432: unresolved import game_shell_common::{touch, undo_ui}`——**红类复现**（pre-T3 API 面全缺），错误码因导入粒度为 E0432 而非 E0425/E0433（模块级 vs 条目级），如实记 | ✅（类吻合码略异） |

三件套形态：红测试代码随 commit 在库（11/9/壳内联）；红输出在 message 留痕且复现一致；
GREEN=check/单测实跑（§2.1）。

### 2.4 用户语义/零漂移抽验

- 用户语义逐字段复现：§1.1 六行全绿（mock 逐字段+真 Scene 位移逆放+Blocked 契约）；
- 零漂移四面：golden/status/reference/original diff **全空**；scene.rs **209+/0-** 纯新增
  （press_key 打断钩子=插入行，旧体零触碰——"key 链一行不动"字面成立）；enums.rs 零变更；
  WalkTo/Undo 新入口金样/oracle 无调用方；
- 探墙双通道：probe_step 与按键踏步同走 `try_step`（scene.rs 五处调用点核对）。

### 2.5 阶段二复验（黄-1 补钉 `c6aff4a`）

- **纯测试增量核验**：`git show c6aff4a --stat` = walk_route.rs **79+/0-** 单文件——产品码零改 ✅；
- **两组合测复跑绿**（本机 `cargo test -p game-core --test walk_route` 11/11 全绿）：
  `undo_midway_then_new_walk_keeps_lifo` ok、`interrupt_during_combat_pending_blocks_then_lifo` ok——
  断言面与终检建议修法逐条对应（旧栈 LIFO 不变/挂起中 Blocked 契约/战斗笔先逆/段笔实际终点回起点）；
- 实现者注记"实跑即绿=实现语义已正，红=无钉现状非行为缺陷"——与阶段一判定
  （结构安全、缺组合钉）一致；
- **ratchet 注记**：补钉后 walk_route(test) 9→11（+2），undo_journal 11 不变——lead
  `ratchet-update` 收尾（check 时点 562/基线 527），存量零回退不变。

## 3. 红项/黄项

| # | 级 | 项 | 定位（§4 三问） | 处置 |
|---|---|---|---|---|
| 黄-1 | 黄 | 时序组合穷举缺 2 断言：回撤中寻路、战斗挂起中打断（§1.4）——结构安全但未钉 | ①层=测试面（行为政策已实现，组合断言缺位）；②消费者=后续改动者（组合无钉=回退无红）；③证据=A（全测试清单 grep 无组合测） | 报 lead → 回派 v2-consumer-swap 补钉 `c6aff4a`（纯测试增量 79+/0-）→ **复验闭环**：两组合测在场且绿（§2.5），walk_route 9→11 | 
| 观-1 | 观察 | R1 架构张力：T1 trait 注"T2/T3 在 Scene 上实现" vs 实现落 shell newtype（core 零改约束产物） | 架构层；消费者=后续迁移者；证据 A（代码+裁定注记） | lead 裁定：**记成本注记即可**（core 红线优先，T5/表设计批若开 core 改动再迁 Scene）——已入遗留清单 |

（TDD/演练/闸/零漂移各项未见放宽阈值/白名单/删用例痕迹；打断钩子为纯插入行零删改。）

## 4. 遗留假设/待复验清单

1. ~~黄-1 待复验~~ → **已闭环**：补钉 `c6aff4a` 两组合测复跑绿（§2.5），walk_route 11/11。
2. **ratchet 基线**：undo_journal(11)/walk_route(9) 待 lead `ratchet-update`（存量零回退已确认）。
3. **PickupStroke/SpendStroke 生产者未接线**：T1/T2 落了笔与逆放面，但场景侧生产者
   （route_pickup 呼叫点/消费笔回滚接线）在 T3 仅 UI 面到达；R2 item_id 域随真生产者
   接线时复核（现为裁定占位）。
4. 战斗复活字段面 exp 本作无字段（注记在 SceneUndo）——若后续加经验系统需补 set_player_vitals。

---

## 5. 结论

用户语义 6/6 逐字段核销；行为政策 9/9；plan §3 五判据 5/5；时序三组合 **3/3 在场**
（黄-1 经补钉 `c6aff4a` 复验闭环）；R1-R3 裁定合理（观察-1 按 lead 裁定记成本注记）；
check 12/12、TDD 红 3/3 复现（1 例错误码粒度差异如实记）、classic 零漂移四面实证
（含 scene.rs 纯新增 209+/0-）。trace L1-L3 结论见 §2.2。
**红项 0 件，黄项 1 件已闭环（含复验），观察 1 件已按裁定登记**——终检无遗留红项。

Evidence: A 级——check/trace 实跑、worktree 三连红复现（E0432 逐字×2+桩化 5 例行为红逐字+
T3 API 面红类）、用户语义逐字段断言、scene.rs numstat 209/0、golden/reference/original
diff 全空、Dispatcher 全通道 Interrupt 构造断言；B 级——SceneUndo 逆放写点逐行核对、
try_step 双通道同函数核对、R1-R3 裁定成本核对。
Confidence: 90
