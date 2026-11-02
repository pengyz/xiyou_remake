# 对抗复核 A（架构纯度）：scene.rs 真模块拆解方案 v1

> 2026-11-02。角色=对抗复核员 A（remake-verifier 纪律，架构纯度视角）。
> 攻击对象：[2026-11-02-scene-module-decomposition-plan.md](../../superpowers/plans/2026-11-02-scene-module-decomposition-plan.md)（下称"方案 v1"）。
> 上位蓝图：[refactor-plan.md](../../spec/refactor-plan.md)（2026-10-16 用户裁定：领域聚合+命令模式）。
> 被拆对象实况：`crates/game-core/src/scene.rs` 5,526 行（文件拆分后核心残留），另有
> scene_menu/scene_paint/scene_persist/scene_script/controls 五个 `impl GameScene` 卫星文件。
> 证据纪律：判定带行号/计数（grep 可复现）；假设标注【假设】。

## 0. 证据基线（取证命令与快照）

**复现命令**（均在仓库根执行）：
- 字段族×文件互访密度：python 计数正则族（combat=battle_\*/player/enemies/equip；entity=entities/grid；
  walk=walk_phase/walk_step_count/step_progress/walk_path_buffer…；persist=saved_\*/global/journal；
  script=script_\*/dialog_\*；visual=overlay_\*/popups/动画相位；menu=menu_\*/slot/options/item_\*/encyc_/alchemy_；
  route=route_\*）对 `crates/game-core/src/*.rs` 逐文件计数 → §1 矩阵。
- 方法级跨域密度：按 fn 行界切 scene.rs 方法体逐族计数 → §1 方法表。
- 现状核对：`grep -rn "enum ScriptOp"|"RESTORE_RULES"|"impl GameScene"` + entity.rs/scene_persist.rs 读取。

**结构快照**：GameScene ≈150 字段（scene.rs:260-610）横跨 12+ 状态簇；151 个 fn；`impl GameScene`
分布于 scene.rs(908, 5279)/scene_menu/scene_paint/scene_persist/scene_script/controls 六文件。

**§1 字段族×文件互访矩阵**（hits = 字段族正则命中数）：

| 文件 | combat | entity | walk | persist | script | visual | menu | route | self 合计 |
|---|---|---|---|---|---|---|---|---|---|
| scene.rs | 228 | 247 | 145 | 16 | 104 | 216 | 176 | 44 | 1838 |
| scene_paint.rs | 19 | 15 | 16 | 1 | 5 | 55 | 1 | 3 | 371 |
| scene_persist.rs | 29 | 14 | 9 | 11 | 23 | 34 | 27 | 5 | 132 |
| scene_menu.rs | 33 | 2 | 23 | 2 | 61 | 46 | 205 | 2 | 612 |
| scene_script.rs | 0 | 0 | 10 | 0 | 24 | 0 | 0 | 0 | 43 |
| controls.rs / tap_walk.rs | 0 | 1 | 5 | 0 | 9 | 54 | 8 | 0 | 54 |
| combat.rs（纯模块） | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |

**方法级跨域 TOP**（scene.rs，Fam=触达字段族数）：

| 方法 | 行 | 行数 | Fam | hits | 分解 |
|---|---|---|---|---|---|
| interact_with_cell | :4025 | 309 | **7** | 208 | combat:67 entity:61 walk:36 visual:30 menu:9 script:3 route:2 |
| tick_entity_route | :2521 | 123 | 6 | 88 | entity:33 combat:17 route:21 walk:13 |
| tick | :3490 | 306 | 6 | 63 | visual:35 walk:15 |
| apply_merchant_offer | :3242 | 173 | 4 | 48 | combat:18 entity:8 visual:12 menu:10 |
| tick_battle | :4358 | 211 | 3 | 61 | combat:34 entity:12 walk:5 visual:7 |
| apply_step_cell_effects | :4611 | 215 | 3 | 77 | entity:26 walk:25 combat:16 visual:9 |
| activate_item | :2777 | 122 | 3 | 15 | combat:2 entity:6 visual:7 |
| pickup_item_type | :4837 | 70 | 3 | 15 | combat:8 script:2 visual:5 |
| use_item_stack | :2998 | 26 | 1 | 12 | combat:12 |

## 1. 攻击点一：档 A vs 档 B 裁决

**证据读法**：
1. 纯计算面**已脱离**：combat.rs 171 行仅 2 处 self（在 GameScene 侧薄封装），predict_battle_hp_loss/
   effective_attack_vs_type/consume_key_for_door/gain_gold 已是参数化纯函数——档 B "combat 域（战斗计算）"
   的主体**已存在**，再建=重复。
2. 编排面**真纠缠**：tick_battle 跨 3 域、apply_step_cell_effects 跨 3 域、interact_with_cell 跨 7 域
   （208 hits）。档 B "GameScene 传参不传整引用"对 interact_with_cell 意味着 7 个状态切片入参——
   参数面爆炸，与红线"too-many-args 6 处永久保留"（方案 §2）正面冲突。
3. 低耦合域**已成形**：scene_persist.rs（self 合计仅 132）集中持久面且 serialize_slot_blob 已 &self 纯化；
   scene_paint.rs 集中 paint* 簇。二者已是"事实域文件"，缺的只是状态切片归属声明。

**裁定：渐进**——
- **档 A 现在做，但按"域 API 化"做**：script_ops/entity/restore_rules 的**结构件已落地**（见 §5 盲点 1），
  剩余工作=op_\*/实体域方法/GameScene 散方法改走域 API，而不是再搬家一次（搬家=被用户否决的文件拆分）。
- **档 B 不整体采纳，逐域再评**：persist/render 两个"已成形域"补状态归属即可（低险）；
  **combat 域只搬纯计算（已落）+道具效果表（见 §2），战斗编排（tick_battle/interact_with_cell）留协调器**——
  它们就是协调逻辑本身，强行域化=把 God 类换成 God 参数列表。
- 档 B 的"可测性"主张（域级单测不构造全 GameScene）须设验收判据（见修订 R9），否则域化=自欺。

## 2. 攻击点二：域边界正确性

**六域逐项**：
| 域 | 判定 | 证据 |
|---|---|---|
| entity | ✅ 边界正确 | entity.rs 已参数化（spawn/remove/sort_by_y 只吃 table/grid），CellGrid 独立红线已满足 |
| script_ops | ⚠️ 命名过窄 | 见下"漏域③"：应扩为 script 域（含 VM 态+对话相位机） |
| restore_rules | ⚠️ 归位错位 | 表在 scene.rs:726、执行器 apply_restore_rules 在 scene_persist.rs:401——分居两文件；且执行器读 self.images（restore 域漏渲染依赖，spawn 需 img_h） |
| combat | ⚠️ 切分线（答 Q3） | 数值/规则=combat.rs 纯函数（已落）；战斗会话簇（battle_\* 8 字段+battle_undo_snap，scene.rs:548）=留协调器【假设：量小，独立成域收益<接口成本】；战斗 UI（show_battle_digits/battle_digit_by_type 预测显示）归 UI 域 |
| persist | ✅ 低耦合可收口 | scene_persist self 合计 132；saved_floor_states/saved_slot_blobs/global 已独立字段 |
| render | ❌ 只读前提不成立 | scene_paint.rs:102-105 写 strip_frame、:248-250 写 backdrop_scroll、:329 advance_bob 写 bob_offset、advance_open_group_paint :5193 写 open_anim_tick——**paint 推进相位是原版行为**，render_adapter 定义必须改（修订 R3） |

**跨域归属裁定**：
- **道具使用（三入口：GIN 拾取 op_gin/pickup_item_type、use_item_stack、try_equip）**→ **独立 item/effect 域**。
  证据：activate_item :2777 同时改 combat(Hp/atk/def)、entity(移门/移墙 :2804-2820)、overlay 反馈，
  甚至**跳模式**（t=13/14 转 EnemyInfo/RecordBrowser）；效果表驱动仿 RestoreAction enum 即可，
  模式跳转走协调器回调。挂 entity 或 script_ops 都会把三入口之一变成跨域调用。
- **菜单族** → **独立 UI 域，不归 render**。证据：menu_\*/slot/options/item\_\*/encyc\_ 30+ 字段；
  scene_menu.rs menu:205+script:61+combat:33，有自己的状态机（tick_menu4/7/9/15/16/19+
  两段式选中 ui_selected，scene.rs:558）与输入（select_row :5423）；menu_family.rs 已有
  SlotSelect/OptionList/OverlayEngine 结构体。render 只是 paint_\* 消费面。

**漏域清单**：① item/effect 域（上）② menu/UI 域（上）③ dialog/script-VM——tick_dialog 错居
scene_menu.rs:6，dialogPhase 相位机（0/1/2/3/4/6）+typewriter+分页是脚本 VM 呈现态，应归 script 域
（script_ops 改名 script）④ tap/input 子机——tap_walk.rs 628 行+tap_\* 11 字段（scene.rs:556-573）
+ui_selected/ui_frame，方案只字未提⑤ popup/audio/camera/walk 已有准域模块（PopupRing/BgmState/
CameraState/WalkState）——六域图须声明归属（建议：即存准域，不重划）。
**错域清单**：restore_rules 表执行器分居（上）；tick_dialog 错居 scene_menu；activate_item 内含模式跳转
（效果域污染协调器职责，剥回调）。

## 3. 攻击点三：协调器残留

GameScene 降级后的**合理残留**（裁定=保留，不硬拆）：
1. **模式状态机**：game_mode+transition_to(:1084)+dispatchModeEntry+tick(:3490) 主编排——协调器本职；
2. **输入分发**：press_key/release_key(:3440/3467)+perform_action(:1560，45 行零域字段命中=纯分发)
   +controls.rs 键→Action 映射——薄层，留协调器；tap 子机（tap_walk）例外，量大可独立（§2 漏域④）；
3. **walkPhase 状态机（walkPhase/walkPathBuffer/stepProgress）=不独立成域**：它是 tick 主相位机本身，
   战斗（walk:5）/交互（walk:4）/脚本行走（advance_script_walk）都消费其相位——拆出=循环依赖。
   但注意：walk_path_buffer/walkto_arrows/walkto_draining+route_\*+undo **已被 RouteTracker
   （pathfind）+UndoJournal（undo_journal.rs）承接**，是已落的"寻路段落域"；
4. **undo journal 已是独立域**（UndoTarget trait，undo_journal.rs:55-59；跨 combat/route/merchant/item
   记账）——方案 v1 未提，须补入域图；
5. battle_\* 会话簇（8 字段+battle_undo_snap）：留协调器【假设】，若后续膨胀再升 battle 会话域。

## 4. 攻击点四：命令模式粒度（Q4 裁定）

**裁定：逐 op 函数（现状 enum ScriptOp + match 分发 + op_\* 独立函数）成立；trait 对象否决；Q4 关闭。**
证据：
1. **16 op 是固定集合**：锚定参考版 a.java:8220-8568（p3-script-interpreter.md §3"16 opcode"）；
   超集扩展=改参考版行为=红线禁区。parse_op 16 标签外 ⇒ None ⇒ 静默停摆（scene_script.rs:29-34，
   R-TODO-1b）——扩展性诉求在本项目不成立（D2/D3：保真期行为只增不减自参考版）。
2. 现状已是目标形态：ScriptOp enum（scene.rs:827）+parse_op/execute_op（scene_script.rs:41-221）
   +op_\* 独立函数（scene.rs:2021-2357），游标逐 op 计算（cursor 字段）——refactor-plan §3 原案落地。
3. trait 对象的 vtable 间接无可测性收益（op 级单测直接调 op_\* 即可）；trait 的正确用武之地=
   **host 依赖注入**（script.rs HostCtx :129-166 已示范）与 UndoTarget，不是 op 分发。
4. 遗留真问题不是粒度而是**双解释器**：script.rs（harness 面，HostCtx）与 scene_script.rs（生产面）
   双实现 16 op（DC-9 桥漏道具加成=双面漂移前科）——见 §5 盲点 4。

## 5. 攻击点五：v1 方案盲点

1. **【最大盲点】执行序与仓库现状错位**：方案 §3 步 2-4 列为待做，实际**结构件已落**——
   Entity struct/EntityTable（entity.rs:18-56 自注"SoA→AoS，refactor-plan §2 第 1 步"）、
   ScriptOp 命令模式（scene.rs:827+scene_script.rs:41-221，注释锚"refactor-plan §3 第 2 步"）、
   RESTORE_RULES 表驱动（scene.rs:726+scene_persist.rs:401-458）。真实剩余=**域 API 化**
   （op_\*/实体方法/散方法改走域接口）。不改写则实施者会把"再搬一次家"当成果——正是用户否决的形态。
2. **paint 副作用保真红线缺失**：§2 红线只钉 scriptCursor；Java paint 推进动画相位（strip_frame/
   backdrop_scroll/bob_offset/open_anim_tick——行号见 §2 render 行）是原版行为，render_adapter 或
   "相位搬 tick"必须逐帧调用序等价+RED 测，方案未覆盖。
3. **双解释器收敛缺席**：op 外移只动生产面 ⇒ harness 面（script.rs）漂移风险上升（DC-9 前科）；
   refactor-plan §7.6 遗漏件⑤"双解释器收敛另立任务"，方案 v1 须至少写"op 双面同步策略"。
4. **测试直读面**：scene_tests/ 共 5,280 行（t2_walkto 2,678+slot_equip 2,112）经 from_fixtures
   （scene.rs:230，7 域 116 hits）直填 GameScene 字段——域自有切片会撕裂测试构造；
   refactor-plan §7.6 遗漏件③"测试内 SoA 直读点清单"仍未清。
5. **已存准域缺席**（§2 漏域⑤）：undo_journal/RouteTracker/tap_walk/popup/audio/camera/walk/
   menu_family 结构体——六域图不与之对账，实施时边界争议会重开。
6. **档 B 参数化矛盾**："传参不传整引用"与"too-many-args 6 处永久保留"互斥；应定为
   **传域实例**（EntityTable/CellGrid 参数化模式已示范），禁止散参化。
7. **Q5 过时**：SoA→AoS 已完成，"SoA 字段与 Entity struct 并存期"不存在；应改问
   "GameScene 散字段→域切片双轨期"策略（trace_snapshot 已隔离 dump 面，主要风险在测试构造面）。

## 6. 判定表 + 修订清单

### 判定表

| # | 攻击点 | 判定 | 要点 |
|---|---|---|---|
| 1 | 档 A vs 档 B | **渐进** | A=域 API 化收尾（结构件已落）；B 整体过度（interact_with_cell 7 域 208 hits 参数面爆炸），只取 persist/render 收口+combat 纯计算（已落） |
| 2 | 域边界 | 六域有漏有错 | 漏 item/effect、menu/UI、dialog-VM、tap 四域；错位：restore_rules 分居、tick_dialog 错居、render"只读"不成立 |
| 3 | 协调器残留 | 合理残留=模式机+输入分发+walkPhase 标量相位+battle 会话簇 | walk 状态机不独立；undo journal 已独立域（方案漏提） |
| 4 | Q4 命令粒度 | **逐 op 函数**（现状落地）；trait 对象否决 | 16 op 锚定参考版固定集合；trait 用在 host 注入（HostCtx/UndoTarget）而非 op 分发 |
| 5 | 盲点 | 7 项 | 执行序错位（最大）/paint 副作用/双解释器/测试直读/准域缺席/参数化矛盾/Q5 过时 |

### 修订清单（改哪节 → 改成什么）

| # | 改哪节 | 改成什么 |
|---|---|---|
| R1 | §3 执行序步 2-4 | 标注"结构件已落"（entity.rs:18-56 / scene_script.rs:41-221 / scene.rs:726+scene_persist.rs:401）；步骤改写为"域 API 化"：op_\* 与实体域方法/散方法改走域接口，restore_rules 表+执行器归位同文件 |
| R2 | §1 档 A | 补边界判据：script_ops.rs ≠ impl GameScene 搬家（=被否决的文件拆分）；op 函数签名改走域 API，逐 op 耦合面登记（op_mvs entity:28 / op_res combat:8 / op_mov 5 域） |
| R3 | §1 档 B render_adapter + §2 红线 | 改为"读面+相位推进面"双面定义并加红线：paint 写 strip_frame/backdrop_scroll/bob/open_anim_tick（scene_paint.rs:102/248/329、:5193）为原版行为；相位搬 tick 须逐帧调用序等价+RED 测，否则 render 域砍到只读面 |
| R4 | §1 六域图 | 补漏域：item/effect（三入口表驱动，仿 RestoreAction）、menu/UI（独立非 render 子域）、script 域扩含 dialog-VM（tick_dialog 自 scene_menu.rs:6 迁回）、tap/input 子机；并声明与 undo_journal/RouteTracker/PopupRing/BgmState/CameraState/WalkState 准域的关系（不重划） |
| R5 | §5 Q4 | 写入裁定：逐 op 函数成立（16 op 锚定 a.java:8220-8568 固定集合，扩展=改参考版=禁区）；trait 对象否决；trait 只用于 host 注入（HostCtx/UndoTarget 先例）。Q4 关闭 |
| R6 | §5 Q5 | 改写：SoA→AoS 已完成（entity.rs Entity struct），双轨期命题作废；改为"GameScene 散字段→域切片双轨"并给策略（dump 面 trace_snapshot 已隔离；测试构造面 from_fixtures 清单化） |
| R7 | §2 红线 | 补两条：① 测试直读保护（scene_tests/ 5,280 行 from_fixtures 直填字段，域切片须保构造面或适配层）② 档 B"传参不传整引用"改口径为"传域实例"（EntityTable/CellGrid 模式），禁散参化制造新 too-many-args |
| R8 | §1 协调器定义 + §5 Q3 | 写明残留清单：game_mode 相位机+dispatchModeEntry+tick 编排+输入分发（perform_action 纯分发）+walkPhase 标量相位+battle_\* 会话簇（留，量大再升域）；Q3 切分线=纯计算 combat.rs（已落）/会话编排留协调器/战斗 UI 归 UI 域 |
| R9 | §3 执行序 | 每步追加收敛判据："域级单测不构造全 GameScene"（档 B 可测性主张的验收测试）；达不到=退回文件搬家判定，不算域化 |
| R10 | §4 已落 4 步处置 | 同意保留+重归类"代码组织整理"；但补注：apply_restore_rules（scene_persist.rs:401）与 RESTORE_RULES（scene.rs:726）分居=错位，随 R1 归位 |

**统计**：修订 10 条（R1-R10）；档位裁定=渐进；Q4=逐 op 函数关闭；Q5=命题作废改写。
