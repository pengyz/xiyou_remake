# 脚本命令管线 + 执行端可装配（设计提案）

> 2026-11-16 · 分支 `modern` · 状态：**提案（未实施；本文不含任何代码改动）**
> 需求来源：本会话委派任务（父 agent `session-919f50ed`），原文要求见 §0 交付边界。
> 纪律依据：`AGENTS.md`（证据分级 A/B/C、禁止调阈值/放宽断言、跨层三问、产出归宿是仓库）。
> 行号说明：本文全部行号均为 2026-11-16 在本仓库**现场实测**（`sed`/`grep`/`javap` 直读），非转抄；引用前请现场复核——本仓有两次行号漂移前科（`gates/cli.py` 的 `gate_state_machine_anchors` 即为防此设）。

> **【2026-11-16 第二轮 · 红队复核更正】** 本文件经 `docs/findings/reviews/2026-11-16-script-pipeline-review.md`（红队裁决：**有前提可推｜Confidence 76｜绿 2 / 黄 8 / 红 2**）复核，**地基重做**（不是润色）：
> §A 分类表按「世界态写面」重建（§A.1′，取代 §A.1）、免 tick 判据与槽清单重算（§A.3′，取代 §A.3）、以及三处硬伤的原地更正（PE-01 定级 / PE-06 方向 / S5 数量，见 §B.2′ 与 §S5′）。
> 纪律：原文一律保留并就地标注（AGENTS §3.6）；作废段落带「〔v1 · 作废〕」标记，作废的单句带行内 `【复核更正】`。
> **冲突时以 §0′ / §A.1′ / §A.3′ / §B.2′ / §S5′ 为准。**

## §0′ 结论（TL;DR v2 · 2026-11-16 第二轮）

### §0′.1 仍然站得住（结论不变，可直接进入实施）

| # | 结论 | 状态 | 本轮复核锚点 |
|---|---|---|---|
| a | 命令对象已是干净数据：`ScriptOp` 23 变体纯数据、`parse_op` 无副作用；缺的是**执行端装配** | ✅ 不变 | `crates/game-core/src/script_ops.rs:5-53`、`:401` |
| b | **执行器必须统一为一份**，方向只能是「engine 向 Java 对齐」 | ✅ 不变 | §C；两实现逐字段写面对照见 §A.1′ |
| c | **`WorldHost`/`PresentationHost` 五 trait 拆分** + 执行器不耦合 `GameScene` 的搬迁方向 | ✅ 不变 | §C.2 / §C.3 |
| d | **S4a（engine 单侧 9 项，PE-01/02/03/04/05/08/09/10/13）** 先做，其中「虚构/错接」五类经逐条复核仍成立：`exit_application` 虚构（PE-02）、`m_119` 装成 `reload_floor_entities`（PE-03）、`ROS_4` 缺 `(byte)`（PE-04）、`CES` 静态类目表（PE-05）、`dialog_target` 错写 `camera_anchor`（PE-08/09） | ✅ 不变（全 A 级） | `a.java:8516-8520`、`:8362`、`:8385`、`:8414-8428`、`:8290-8292` |
| e | **不做 `Outcome{state_delta}` 值化**，改用「同一段 delta 代码 + 参数化状态宿」 | ✅ 不变 | §C.1 |
| f | S5 之后**生产行为零变化** | ⚠️ 有条件成立 | 条件 = 语料行保留行尾 `" \x1e"`（§A.3′ 第 5 条）；且 PE-06 方向必须按 §B.2′ 更正，否则会删掉 engine 唯一正确的那条写 |

### §0′.2 被推翻（作废，逐条给替代）

| # | 原主张（v1） | 为什么错 | 替代 |
|---|---|---|---|
| 1 | §A.1 分类表（判据 = 「有无相位跃迁」） | 判据错。无相位跃迁 ≠ 无世界态写。被误判为纯 P 的至少 9 类：`TAK`/`SEE`/`CES`/`DES`/`ADD`/`SWD`/`ROS_*`/`GIN`/`RES`；48 个「P 槽」里 **43 个**含这些指令（复核 F6，本轮独立复算一致） | **§A.1′**（判据换成「执行后世界态是否变化」，23 行字段级） |
| 2 | §A.3「48/68 槽免 tick」 | 真·零写面槽只有 **5** 个（37、39、58、59、60）；可**同步求值**（不等按键/时钟/相位收敛）的只有 **11** 个 | **§A.3′**（三档：A 5 / B1 2 / B2 4 / C 57） |
| 3 | PE-01 = **P0**，「命中每一槽」「整页对话被跳过」 | 尾门触发条件是 `cursor >= len`；生产口径每行以 `" \x1e"` 结尾（11 字节）⇒ 末命令消费后 `cursor == len-1`，**永不触发**。只有剥离 `0x1E`（求解器 10 字节表）才激活 | **P1（口径依赖的潜伏项）**，机制见 §B.2′；`a.java:8574-8580` 为**伪锚**（该区间是 `MVS` 体），真锚 `a.java:8600-8602`（尾检查 = 纯 `return`）+ `:8603-8610`（入口门）。**追加（§B.2′.1b）：真正缺陷是 `from_fixtures` 少剥 `0x1E`——Java 字面量行尾只有空格（`a.java:1124`），故 Java 与求解器口径下尾门每行都触发；"永不触发"只是 Rust 生产夹具的现况** |
| 4 | PE-06「**GameScene 对**」 | 写反。`inventory_entry_count` 全仓只在 engine（`crates/game-core/src/script.rs:77` 字段 / `:126` 默认 / `:459` RES 写 0）；`GameScene` **无此字段**，反而多写 `journal.clear()`（`crates/game-core/src/script_ops.rs:114`）。Java `a.java:8489` 就是 `inventoryEntryCount = 0` ⇒ **engine 对** | §B.2′（PE-06 方向更正；`journal.clear()` 降为待裁多写） |
| 5 | §E S5「**9 处**调用点」（表格相加 28；红队复核为 22） | 三个数都不对。A 级实测：**31 个调用表达式** + 2 个定义 + 1 个函数指针引用 | **§S5′**（逐文件口径表） |

### §0′.3 实现前置条件（做 fast-forward **之前**必须先满足）

1. **分类表按写面定稿**（§A.1′）。FF 的正确性完全建立在「哪些指令必须**执行**、哪些可以跳过」上；用 v1 表做 FF 等于批量生成状态增量丢失（复核 F6 的实质）。注意：**「免 tick」≠「免执行」**——11 个可同步求值的槽里只有 5 个能整槽跳过，其余 6 个必须照常 apply delta（§A.3′）。
2. **语料口径单一且显式**：`0x1E` 保留与否必须成为 table 构造参数（§B.4、§D.4 第 2 条）。否则尾门在 (a) 口径死、在 (b) 口径活（每槽触发），2×2 矩阵只是把同一盲点复制四份。
3. **观察面必须含 `dialog_phase` + `ticks` + `path_arrows_visible`**（§D.3 :496）；否则 PE-01 一类「只差相位/拍数」的缺陷继续隐身。
4. **FF 出口必须交付入口门写入**：`phase=4` + `script_line_flags[idx]=true`（`idx != 32`），并承认**阻塞相位槽的旗标写入被推迟**到该相位收敛之后（Java 只在入口门写，`a.java:8603-8610`）——在槽内提前写会与生产时序不一致（终值相同）。

---

## §0 结论（TL;DR）

1. **命令对象已存在且足够干净**：`ScriptOp` 的 23 个变体（`crates/game-core/src/script_ops.rs:5-53`）是纯数据，解析函数 `parse_op`（`crates/game-core/src/script_ops.rs:401`）无副作用。缺的不是"命令模式"，而是**执行端的装配**。
2. **执行端已有 4 个 trait 雏形，但只覆盖了半个执行面**：`HostCtx`（`crates/game-core/src/script.rs:135-152`，8 方法，其中 6 个是表现类）+ `EntityService`（`crates/game-core/src/subsystem/entity_service.rs:25`，9 方法）+ `DialogSubsystem`（`crates/game-core/src/subsystem/dialog.rs:40`，3 方法）+ `ItemService`（`crates/game-core/src/subsystem/item_service.rs:15`，3 方法）。**缺口清单见 §C.2**（约 14 个写点无 trait 覆盖，正是 `op_*` 里直写 `self.*` 的部分）。
3. **两个执行器语义不等价，已确证 13 处不一致（PE-01…PE-13，§B.2）**，其中 4 处是 `ScriptEngine` 侧**虚构或错接**（尾部门、END 的 `exit_application`、LAY 把 `m_119`(存档) 接到 `reload_floor_entities`、ROS_4 缺 `(byte)` 截断）。**迁移方向只能是"engine 向 Java 对齐"，不能是"求解器向 engine 对齐"。**
4. **最严重的一条（PE-01，P0）**：`ScriptEngine` 在 `crates/game-core/src/script.rs:511-519` 有尾部收尾门，会在**同一拍**把 `phase` 改写成 `AwaitCamera`；Java（`javap` 偏移 2386/2401）与 `GameScene`（`crates/game-core/src/scene_script.rs:14-36`）都只在**入口门**（`cursor+3 > len`）触发收尾。后果：任何"末命令带相位"的行（语料里几乎全部，因为行尾有尾空格 ⇒ 末命令消费后 `cursor == len`）在 engine 下会**丢掉末命令的相位副作用**——例如槽 6 `TAK_53_58` 的对话页整页跳过、槽 0 结尾 `GUT_1` 的换层相位被 `AwaitCamera` 覆盖。 【复核更正 · 见 §B.2′】定级 **P0 → P1（口径依赖的潜伏项）**：尾门条件 `cursor >= len` 在生产口径（行尾 `" \x1e"`，11 字节）**永不触发**，故「命中每一槽」「整页对话被跳过」不成立；`a.java:8574-8580` 是伪锚（实为 `MVS` 体），真锚 `a.java:8600-8602` + `:8603-8610`。
5. **现状 A/B 对拍看不出一条 PE**：`crates/game-solver/src/script_abs.rs` 的 `Obs`（`:73`）不含 `dialog_phase`、不含 tick 计数；且 (a)/(b) 两侧竟用了**两份不同的语料字符串**（(a) 经 `DialogData::from_fixtures`（`crates/game-core/src/scene.rs:253-256`）即 `.lines().trim()`，保留行尾 `0x1E` 记录分隔符；(b) 经 `dialogue_table()`（`crates/game-solver/src/script_abs.rs:158-166`）即 `split("\u{1e}\n")`，不含 `0x1E`）。行尾差 1 字节 ⇒ 两侧的"行已消费完"由**不同的门**检测（(a) 入口门、(b) 尾部门），末态恰好同值 ⇒ 对拍全绿。**这是本提案发现的方法论缺陷，不是实现缺陷。**
6. **状态增量必须"一份实现"，不能"两份实现 + 测试对齐"**：不做值类型 `Outcome{state_delta, …}`（理由 §C.1），而做**同一段 delta 代码 + 参数化状态宿**（`trait ScriptWorld`）。求解器侧由此获得"同一语义、不同宿主"的第二个实现，而不是"抄一份"。
7. **求解器真正的性能收益点在"脚本门判定不再构建 `GameScene`"**（§D.3），前提是先把 `try_run_scene`（`crates/game-core/src/scene.rs:927`）的**谓词内副作用**拆掉——它在过门时会写 `script_line_flags[19]`（`:935`）、`flags[32]`（`:949`）、不过门时写 `flags[18]`（`:995`）。带副作用的"谓词"无法做成纯函数出口。
8. **语料可分两类**：68 槽中 **48 槽**不含 `MOV`/`LAY`/`END`/`GUT`，其世界状态增量可在**不驱动任何 tick** 的情况下算到底；**20 槽**含等待/换层/终止/递归，必须驱动（§A.3）。这是"哪些槽能免 `GameScene`"的静态判据。 【复核更正 · 见 §A.3′】判据与数字均作废：不含 `MOV/LAY/END/GUT` 只排除"阻塞"，不排除"世界态写"。真·零写面 **5** 槽、零 tick 可求值 **11** 槽、必须 tick **57** 槽。
9. **本方案可以让生产行为零变化**（S1–S5 全部改 engine 侧），但**唯有一批必须显式承认改变生产行为**：`ADD` 的 `initCellGrid`/floor_assets 守卫、`END` 的 `dispatchModeEntry case 20` 可观测面——这两处 GameScene 与 Java 的等价性本文只做到 **B 级**（§B.5、§E 的 S4b）。
10. **`game-rules` crate 拆分**：本改造**只解除必要条件之一**（`script` 子树对 `scene` 的硬依赖），**不构成充分条件**（还需把 `Entity`/`ItemDef`/`FloorAssets` 等数据模型与 `ScriptCtx` 的读槽类型一起迁出，见 §E.9）。

---

## §1 现状基线（全部可现场复核）

### §1.1 命令对象层

| 事实 | 值 | 证据 |
|---|---|---|
| 变体数 | 23 | `crates/game-core/src/script_ops.rs:5-53` |
| 解析入口 | `pub fn parse_op(line: &str, script_cursor0: usize) -> Option<ScriptOp>` | `crates/game-core/src/script_ops.rs:401` |
| 解析纯度 | 纯（只读 `line`/`cursor`，返回新 `ScriptOp`），未知 opcode ⇒ `None` | 同上 164 行 `match` |
| 命令自带游标 | 每个变体带 `cursor: usize`（下一拍起点），`Gut` 的 `cursor` 随后被 `load_level_script` 覆写为 0 | `crates/game-core/src/script_ops.rs:20` / `crates/game-core/src/scene_persist.rs:520` |
| 游标读取器 | `impl ScriptOp { pub fn cursor(&self) -> usize }`，23 臂 | `crates/game-core/src/scene.rs:715-742` |
| `RosNoop` 产生条件 | `ROS` 子码 ∉ {1..6} ⇒ `RosNoop { cursor: v2 + 1 }` | `crates/game-core/src/script_ops.rs:542` |
| `Mvs` 产生条件 | opcode == `"MVS"` ⇒ `Mvs { cursor: script_cursor0 + 1 }` | `crates/game-core/src/script_ops.rs:546` |

### §1.2 两个执行器

| 面 | 入口 | 门结构 | 备注 |
|---|---|---|---|
| 生产（GameScene） | `crates/game-core/src/scene_script.rs:14` `pub(super) fn execute_script_instruction(&mut self)` | **仅入口门**（`:17` `if script_cursor0 + 3 > line.len()`）→ `set_dialog_phase(4)` + `script_line_flags[cur]=true`（`cur != 32`）+ `lock_camera_on(0)` + `return`（`:19-25`）；否则 `parse_op`（`:29`，`None` ⇒ 静默 `return`）→ `execute_op`（`:35`） | 57 行，`impl GameScene` 内建 |
| 宿主注入（ScriptEngine） | `crates/game-core/src/script.rs:270` `pub fn execute_script_instruction(&mut self, host: &mut (impl HostCtx + EntityService + DialogSubsystem + ItemService))` | **入口门**（`:273` `script_cursor0 + 3 <= line.len()` 取反，`:274-282`）+ **尾部门**（`:511-519` `if self.state.script_cursor >= line.len()`） | 单臂大 `if` 链，284-519 |
| 分派器（两侧共用意图） | `crates/game-core/src/script_ops.rs:553` `pub(super) fn execute_op(&mut self, op: ScriptOp)` | 先造 `ScriptCtx` + `ScriptResolver.execute_op` 得 `ScriptStep`（`:555-559`），再 `match` 23 变体调 `op_*`（`:560-586`），末尾 `if matches!(step, ScriptStep::Recurse) { self.execute_script_instruction(); }`（`:587-589`） | GameScene 侧唯一出口 |

### §1.3 依赖边（本文要动的边）

| 边 | 位置 | 性质 |
|---|---|---|
| `ScriptOp` 归属在 `scene` | `crates/game-core/src/scene.rs:55-56`（`#[path="script_ops.rs"] mod script_ops;`）+ `:61` `pub use script_ops::ScriptOp;` | 命令对象住在场景模块树里 |
| `subsystem` 反向依赖 `scene::ScriptOp` | `crates/game-core/src/subsystem/script_host.rs:31` `pub use crate::scene::ScriptOp;` | **环的源头**（scene→subsystem→scene） |
| `subsystem::dialog` 依赖 `scene::DialogState` | `crates/game-core/src/subsystem/dialog.rs:8` `use crate::scene::DialogState;` | 同上 |
| `subsystem/mod.rs` 测试块同源 | `crates/game-core/src/subsystem/mod.rs:151`（生产再导出）与 `:267-270`（`#[cfg(test)]`） | 迁出时须同步 |
| `subsystem::ui_render` 依赖 `scene::RenderState` | `crates/game-core/src/subsystem/ui_render.rs:14` | **本方案范围外**（渲染态，不属脚本管线） |
| `script.rs` 自身 | 无 `crate::scene::` 直接引用；`use` 仅 `:11-13` subsystem 三件 + `:27-29` enums | 已有"无 scene 依赖"的雏形 |
| 模块位置 | `crates/game-core/src/lib.rs:25` `pub mod script;`（顶层，与 `scene` 平级） | 未来可整模块搬走 |

### §1.4 求解器侧

| 事实 | 证据 |
|---|---|
| 脚本门入口 | `crates/game-solver/src/solve_local.rs:2012` `fn gate_arrive(&self, st:&mut St, tx:i32, ty:i32) -> GateArrive` |
| 惰性建场景 | 同文件 `sc_opt.get_or_insert_with(\|\| build_scene(self.input, st, (tx,ty)))`（注释「memo 全命中时零 build」） |
| memo 键 | `(self.input.floor, param, st.player.clone(), st.flags, st.ents.clone())` |
| 驱动 | `crates/game-solver/src/solve_local.rs:2360` `fn tickdrive(sc: &mut GameScene) -> Drive`，`Drive`（`:2354`）= `Pending(i32)` / `Completed` / `Stalled`；每拍 `sc.key_value = 1; sc.tick(ft, &wt)` |
| 完成判据 | `GameMode::Map` 且 `landed = (player_px>>5, player_py>>5) == sc.dialog.dialog_target`；`!(script_walk_armed && !landed) ⇒ Completed` |
| 既有近似 | 脚本行走期间不触发中途 type83 门（本方案不引入、也不修） |
| 对拍 harness | `crates/game-solver/src/script_abs.rs`（703 行）；`Obs`（`:73`）；`side_a::drive`（`:191`）；`side_b::BHost`（`:241`）；`drive`（`:480`）；分类 `Class`（`:541`）/`class_of`（`:552`） |

### §1.5 语料统计（A 级：`crates/game-core/tests/fixtures/level_scripts.txt` 直解析）

分隔符 `\u{1e}\n`；**68 槽**、**296 条指令**；空槽 = `39/58/59/60`；槽 67 = `TAK_264_268`（DC-5 不可达）。

| 指令 | 出现次数 | 涉及槽（节选） |
|---|---|---|
| `TAK` | 81 | 0,1,2,…,67（几乎每槽） |
| `MOV` | 47 | 0,1,2,3,4,5,7,8,10,12,19,30,31,56,57,61 |
| `GUT` | 7 | 0→1, 16→37, 23→12, 56→57, 57→61, 61→62, 62→63 |
| `DES` | 40 | 0,2,3,4,5,7,10,14,18,19,26,…,63（负参 5 条全在槽 63） |
| `SWD` | 3 | 3, 9, 11 |
| `LAY` | 1 | 0（`LAY_2`） |
| `ROS` | 30 | `ROS_4`×23、`ROS_2`×2、`ROS_5`×2、`ROS_6`×2、`ROS_1`×1、**`ROS_3`×0** |
| `CES` | 48 | 0,3,4,7,8,10,13,15,18,19,25,38,50,53,55,56,62,63 |
| `GIN` | 4 | 26, 27, 28, 29 |
| `ADD` | 1 | 7（`ADD_2_72_11_10`） |
| `GLV` | 1 | 37（`GLV_1`） |
| `RES` | 1 | 0（`RES_0`） |
| `SEE` | 30 | 40–54（`flag=1` 共 7 处：40,41,42,46,47,48,50） |
| `END` | 1 | 65（`TAK_127_128 END_0`） |
| `SMS` | 1 | 66（`TAK_261_263 SMS_0`） |
| `MVS` | **0** | —（DC-2） |

槽 0（第 1 层序章）完整链，A 级原文：
`CES_84_6_11 MOV_0_5_11 TAK_8_9 CES_70_5_8 TAK_10_10 ROS_4_1 TAK_11_17 ROS_4_2 TAK_18_19 MOV_0_5_10 TAK_20_21 DES_70_5_8 LAY_2 ROS_1_4_7 ROS_2_0 ROS_2_6 RES_0 GUT_1`

> 语料口径警告（A 级）：`DialogData::from_fixtures`（`crates/game-core/src/scene.rs:253-256`）用 `.lines().map(trim)`，**保留行尾 `0x1E` 字节**（实测：`"TAK_53_58 \u{1e}".trim().len() == 11`，`'\u{1e}'.is_whitespace() == false`）；`dialogue_table()`（`crates/game-solver/src/script_abs.rs:158-166`）用 `split("\u{1e}\n")`，**不含 `0x1E`**。同一槽两侧字符串长度差 1 字节 ⇒ 行尾判据不同源（见 §B.4）。

---

## §A 23 条指令分类表（核心证件的）

### §A.0 表读法

- **状态增量** = 进入下一拍后**仍然存在、且被后续代码读到**的世界态改动（世界态 = 玩家/背包/实体表/格栅/旗标/楼层/模式）。只在本方法内写后即读的字段标"死字段"。
- **表现副作用** = 相机（ease/锁/snap）、对话页/打字机、overlay 排版、弹窗、退出、等待输入。**表现副作用的判定标准：删掉它不影响任何 `Obs`/存档/后续判定**。
- **可达** 以 `docs/spec/dead-code-rulings.md` 为准；`☠️` 标 DC 编号。
- 证据列 = `a.java` 行号（`reference/src/deobf/a.java`，反混淆投影）+ **至少一个使用它的脚本槽** + javap 字节码偏移（当需要裁决控制流时）。

### §0′.4 与复核报告的分歧（一处：结论方向相同，机制与修法相反）

复核报告（`docs/findings/reviews/2026-11-16-script-pipeline-review.md`）判定 PE-01 为 P1，理由是「每行以 `" \x1e"` 结尾 ⇒ 末命令游标恒为 `len-1` ⇒ 尾门永不触发」。**结论（P1）我同意，机制与修法我不同意**，两处实证如下（A 级）：

1. **夹具最后一槽会触发**：槽 67 原文是 `TAK_264_268 `（12 字节，`54 41 4b 5f 32 36 34 5f 32 36 38 20`，文件末尾无 `0x1E`、无换行），TAK 消费后 `cursor = 12 = len`。它虽然 DC-5 不可达（且 `.trim()` 后 `parse_script_int` 先 panic），但"任何槽都不会到达 `len`"这句是错的；复核的复现脚本用了 `lines[:-1]`，把最后一行排除在外。
2. **更重要：`" \x1e"` 是 Rust 夹具的形态，不是 Java 的形态**。`a.java:1124` 的 `levelScriptLines` 是常量字面量，**行尾只有一个空格**（实测首行 164 字节，末 8 字节 `"0 GUT_1 "`），Java 也从不 `trim()`。由于 `scriptCursor = lastDelimPos + 1` 会吃掉那个空格 ⇒ **Java 与求解器口径下，尾门在每一行的末命令后都触发**。⇒ "永不触发"只是 `DialogData::from_fixtures` 保留 `0x1E` 造成的**本仓特有的现况**，由此推出的"尾门是多余物"结论是**反的**：真正该修的是语料口径，删尾门会连 Java 唯一正确的收尾门一起删掉（见 §B.2′.1b）。

> **【2026-11-16 复核更正 · 行锚卫生】** `crates/game-core/src/script_ops.rs` 各 `op_*` 的 `///` 注释所引 `a.java` 行号**系统性陈旧且偏移不固定**（A 级，逐条实测）：`op_tak` 注 `8257-8263` 而真锚 `8273-8279`（-16）、`op_des_all` 注 `8311-8320` 而真锚 `8329-8333`（-18）、`op_ros_move` 注 `8361-8366` 而真锚 `8373-8376`（-12）、`op_lay`/`op_mov`/`op_see`/`op_res`/`op_add` 均 -6、`op_gut` -8、`op_ces`/`op_glv` -9。`reference/seed/a.java` 与 `analysis/decomp/a.java` 是 8457 行旧投影（变量名仍为 `string`，CES 在 `:6300`），对不上任何一组 ⇒ 注释锚是**陈年投影锚，不可作证据**。本表一律用现场复核值；引用代码注释前须复核。

### §A.1′ 分类总表 v2：按「世界态写面」重建（2026-11-16 第二轮，取代 §A.1）

**判据（唯一）**：执行这条指令后，**世界态**——`hp`/`atk`/`def`/`gold`/三色钥匙/`item_stack_size`/`inventory_entry_count`/装备/道具/实体表（`type`/`px`/`py`/`removed`/`visible`/`state`/`anim_row`）/格栅/旗标 `script_line_flags`/楼层（`current_floor`/`pending_floor`/`transitioning`/`min_floor`/`max_floor`/`floor_assets`）/**相位模式** `dialog_phase`（含它驱动的相位收敛）/`game_mode`/当前脚本槽与游标——**是否发生任何变化**。有 = **W**；无 = **P**；两者兼有 = **W+P**。
**相位模式为何算世界态**：`dialog_phase != 0` 时执行器**不再被调用**（`crates/game-core/src/scene_dialog.rs:113-118` 只有 `0 =>` 支调执行器），也就是说它改变了"下一步会不会执行脚本指令"这一可观测事实；把它当"表现"正是 v1 表最大的错。
**不计入写面**：`script_cursor`（执行器游标，23 变体共有，单列于本表末）与只在本方法内写后即读的死字段。

| # | 指令（变体） | 世界态写入（逐字段 · Rust 锚 / `a.java`） | 类别 | 表现副作用 | 是否等待输入/时钟/相位收敛 | 可达性（语料 / DC） | 分级 |
|---|---|---|---|---|---|---|---|
| 1 | `Tak{start,end}` | `dialog_page=start`（`crates/game-core/src/script_ops.rs:61`；engine `tak_page_start` `crates/game-core/src/script.rs:288`；`a.java:8274`）、`dialog_page_end=end`（`script_ops.rs:62`／`script.rs:290`；`:8276`）、**`dialog_phase=1`**（`script_ops.rs:63`／`script.rs:292`；`:8278`）、`dialog_speaker=speakers[start]` | **W+P** | `open_dialog_page(start)`（`script_ops.rs:64`／`script.rs:294`；`:8279`）、打字机与翻页状态机 | **是**（相位 1 停表等按键，`scene_dialog.rs:120-168`） | ✅ 81 条 / 52 槽 | A |
| 2 | `Lay{floor}` | `pending_floor=floor`（`script_ops.rs:70`／`script.rs:367`；`:8363`）、`transitioning=true`（`:71`／`:368`；`:8365`）、**`dialog_phase=5`**（`:73`／`:370`；`:8367`） | **W+P** | `overlay_text=None`（`:72`／`:369`；`:8366`）、遮幅/换层演出 | **是**（相位 5 等 `transitioning && !wipe_closing`，`scene_dialog.rs:256-264`） | ✅ 1（槽 0 `LAY_2`） | A |
| 3 | `Gin{kind,value}` | `kind=0` ⇒ `pickup_item_type(v)`（`script_ops.rs:80`；`:8436`）写 `item_stack_size`/`inventory_entry_count`/装备激活；`kind=1` ⇒ `gold_amount += v`（`script_ops.rs:83`；`:8439`） | **W** | 金币 popup `popups.spawn(4,…)`（`script_ops.rs:84-92`；`a.java:7527-7532`） | 否（**同拍尾递归**下一条：`script_ops.rs:588`／`script.rs:422`；`:8443`） | ✅ 4（槽 26–29） | A |
| 4 | `Res{}` | `difficulty_index==0` 门内 **12 字段**：`yellow_keys=0`、`red_keys=0`、`blue_keys=0`（`script_ops.rs:103-105`；`:8480`）、`item_stack_size=0`（`:106`；`:8481`）、`gold_amount=4`（`:107`；`:8482`）、`alchemy_upgrade_count=0`（`:108`；`:8483`）、`min_floor=1`（`:109`；`:8484`）、`max_floor=3`（`:110`；`:8485`）、`player_atk=10`（`:111`；`:8486`）、`player_def=10`（`:112`；`:8487`）、`player_hp=400`（`:113`；`:8488`）、**`inventory_entry_count=0`**（仅 engine `script.rs:459`；`:8489`） | **W** | 无 | 否 | ✅ 1（槽 0 `RES_0`） | A |
| 5 | `Add{floor,ty,cx,cy}` | `entities` 新增 `spawn(ty,cx<<5,cy<<5,0)`（`script_ops.rs:127-136`／`script.rs:423-441`；`:8465`）、楼层存档 `m_119`（`:8461`、`:8466`）、楼层装载 `m_122`（`:8462`、`:8469`）、**`initCellGrid()`（`a.java:8468`；两实现均缺 ⇒ PE-07/R-1）** | **W** | 无（跨层，不可见） | 否 | ✅ 1（槽 7 `ADD_2_72_11_10`） | A（Java 体）／**B**（`initCellGrid` 的写面是否与 `m_122(cur)` 重叠，须 R-1 trace） |
| 6 | `See{tx,ty,page,page_end,flag}` | **`dialog_target=(tx,ty)`**（`script_ops.rs:150`；`:8494`+`:8496`；engine 错写 `camera_anchor_x/y` ⇒ PE-08）、`dialog_page=page`（`:151`）、`dialog_page_end`（`:152`）、`flag==1 ∧ find_path` ⇒ **`path_arrows_visible=true`**（`:155`；`:8508`）+ `walk_phase=0`（`:156`；`:8509`）、**`dialog_phase=6`**（`:158`；`:8513`） | **W+P** | `open_dialog_page`（`:153`；`:8504`）、`ease_cam_x/y`+`ease_target_x/y`（`:159-162`；`:8514`）、打字机 | **是**（相位 6：`camera_at_target()` 才推进打字机 `scene_dialog.rs:123`，且等按键） | ✅ 30（`flag=1` 7 条；14 槽） | A |
| 7 | `Gut{target}` | `script_line_flags[cur]=true`（`crates/game-core/src/subsystem/script_host.rs:74-80`；`:8318`）、`load_level_script(target)` ⇒ `current_script_index=target`、`script_cursor=0`、`dialog_phase=0`、`script_walk_armed=false`、换行 `script_line`（`script_ops.rs:169`；`:8319`） | **W** | 换场景 + `lock_camera_on(0)` | 否（跨槽：下一拍从新槽继续） | ✅ 7（0→1、16→37、23→12、56→57、57→61、61→62、62→63） | A |
| 8 | `Mov{…}`（三参/五参） | **玩家臂**：`dialog_target=(x,y)`（`script_ops.rs:177`；`:8290`+`:8292`）、`script_walk_armed=true`（`:200`；`:8311`）、**`dialog_phase=3`**（`:201`；`:8312`）、`walk_phase=0`（`:202`；`:8313`）；**实体臂**：`route_entity`（`:181`；`:8295`）、`route_phase=0`（`:184`；`:8297`）、`anim_row=1` 仅 `type==72`（`:186`；`:8299-8300`）、`route_start=(px>>5,py>>5)`（`:189`；`:8303-8304`）、**`dialog_phase=2`**（`:194`；`:8306`） | **W+P** | `lock_camera_on(0)`（`:203`；`:8314`）、实体路线动画 | **是**（实体臂相位 2 路线收敛 `scene_dialog.rs:169`；玩家臂相位 3 逐格行走 `:170-249`） | ✅ 47（3 参 11 / 5 参 36；16 槽） | A |
| 9 | `DesAll{want}` | 负参逆序扫全表 `type == abs(want) ∧ !removed` ⇒ `entity_removed=true`（`script_ops.rs:208-214`；`:8329-8333`） | **W** | `build_minimap()`（`:211`，派生） | 否 | ✅ 5（全在槽 63：`DES_-66_0`/`-56`/`-49`/`-47`/`-69`） | A |
| 10 | `DesAt{ty,cx,cy}` | `find_blocking_entity(cx,cy,ty)>=0` ⇒ 该实体 `entity_removed=true`（`script_ops.rs:217-222`；`:8339-8341`） | **W** | `build_minimap()`（`:220`） | 否 | ✅ 35（DES 共 40，负参 5） | A |
| 11 | `Swd{}` | 全表 `type==81 ∧ !removed` ⇒ `visible=true`（`script_ops.rs:229`；`:8351`）、`type=4`（`:230`；`:8352`）、`dispatch_sprite_size(4,i)`（`:232`；`:8353`）、`state=2`（`:233`；`:8354`）、`anim_row=8`（`:234`；`:8355`） | **W** | 精灵换图 | 否 | ✅ 3（槽 3/9/11） | A |
| 12 | `Ces{ty,cx,cy}` | `entities` 新增 `spawn(ty,cx<<5,cy<<5,0)`（`script_ops.rs:244-253`；`:8401`）、`remove_des(idx)`（`:254`；`:8402`）、**`script_step_delay` = cat(ty,floor) ∈ {5,8,0}**（`script_host.rs:81-88`；`:8414-8428`）、`camera_target_x/y`（`:258-259`；`:8408-8409`） | **W** | 相机回中 + ease 目标 | **否**（同拍继续；但 `script_step_delay>0` 使**下一条**延后 5/8 拍 ⇒ **tick 计数不等**，`scene_dialog.rs:114-118`） | ✅ 48 | A |
| 13 | `Glv{}` | **无**（`script_ops.rs:264`；`:8473-8475`；仅推进 `script_cursor`） | **P** | 无 | 否 | ✅ 1（槽 37 `GLV_1`） | A |
| 14 | `End{}` | **`game_mode=20`**（`script_ops.rs:270`；`:8518`）+ `dispatch_mode_entry()`（`:8519`）；engine 另有 `host.exit_application()`（`script.rs:496`）⇒ **PE-02 虚构** | **W** | 结局图 + `dialog_char=1` + `layout_wrapped_text_limited(overlay,209,…)`（`script_ops.rs:271-275`；`a.java:3039-3080`） | **终止**（不可解除；`crates/game-core/src/scene.rs:2975` 的 `game_mode == Dialog` 不再成立） | ✅ 1（槽 65） | A |
| 15 | `Sms{}` | **无**（`script_ops.rs:279`；`:8521-8523`） | **P** | 无 | 否 | ✅ 1（槽 66） | A |
| 16 | `RosMove{px,py}`（ROS_1） | `player_pixel_x=px<<5`、`player_pixel_y=py<<5`（`script_ops.rs:283-284`；`:8373`+`:8375`） | **W** | `lock_camera_on(0)`（`:285`；`:8376`） | 否 | ✅ 1（槽 0 `ROS_1_4_7`） | A |
| 17 | `RosEquip{tier}`（ROS_2） | `try_equip(tier,true)` ⇒ `equipped_weapon`/`equipped_armor`/`equip_types[12]`/`equip_bonuses[12]`（`script_ops.rs:290`；`:8379`） | **W** | 装备弹窗面（`m_082` 钩子） | 否 | ✅ 2（槽 0 `ROS_2_0`/`ROS_2_6`） | A |
| 18 | `RosHp{v}`（ROS_3） | `player_hp=v`（`script_ops.rs:295`；`:8382`） | **W** | 无 | 否 | ⚠️ 语料 **0 命中**（ROS 子码集 = {1,2,4,5,6}） | A |
| 19 | `RosFace{v}`（ROS_4） | **`facing=(v as i8) as i32`**（`script_ops.rs:300`；`:8385` 的 `(byte)` 截断；engine `script.rs:390` 缺截断 ⇒ PE-04） | **W** | 无 | 否 | ✅ 23（取值 {0,1,2,3}） | A |
| 20 | `RosAtk{v}`（ROS_5） | `player_atk=v`（`script_ops.rs:305`；`:8388`） | **W** | 无 | 否 | ✅ 2（槽 53/55 `ROS_5_510`） | A |
| 21 | `RosDef{v}`（ROS_6） | `player_def=v`（`script_ops.rs:310`；`:8391`） | **W** | 无 | 否 | ✅ 2（槽 53/55 `ROS_6_510`） | A |
| 22 | `RosNoop{}`（其余子码） | **无**（`a.java:8381` 无 default 体；`script_ops.rs:542`） | **P** | 无 | 否（不取参不推进 ⇒ **下一拍重解析同一 token**） | ⚠️ 0 命中；必须保留解析（否则合法行落 `None` ⇒ 停摆语义错） | A |
| 23 | `Mvs{}` | `f_short_arr_04` 重建 + 型 82 配对 + `removeGroupAtCellType`/`m_107` 双向搬运（`script_ops.rs:316-395`；`:8525-8598`） | **W** | 无 | 否 | ☠️ **DC-2**（68 槽 0 命中） | **B**（体未逐字段复核，仅按 DC 判定；不进 headless 语义义务） |

**W 类合并清单（本表结论）**：`Tak`／`Lay`／`Gin`／`Res`／`Add`／`See`／`Gut`／`Mov`／`DesAll`／`DesAt`／`Swd`／`Ces`／`End`／`RosMove`／`RosEquip`／`RosHp`／`RosFace`／`RosAtk`／`RosDef`／`Mvs` = **20 个变体**。
**纯 P 类**：`Glv`／`Sms`／`RosNoop` = **3 个变体**（v1 表把 16 个变体误记为 P）。
**W+P 双面**：`Tak`／`Lay`／`See`／`Mov` = 4 个（相位/页面 + 世界态）。
**唯一真正"零写面"的可达指令是 `Glv`、`Sms`**；语料里只有槽 37（`GLV_1`）与 4 个空槽落在这一档（§A.3′）。

### §A.1 分类总表（23 变体）〔v1 · 判据已作废 · 仅作对照留档〕

| # | 指令（变体） | 状态增量（世界态） | 表现副作用 | 可达 | 证据 |
|---|---|---|---|---|---|
| 1 | `Tak{start,end}` | **对话域**：`dialog_page=start`、`dialog_page_end=end`、`dialog_speaker=speakers[start]`；**无世界态** | 开页 + 打字机（`composed`/`dialog_char`）+ 相机锁 speaker（型 69 加 32） | ✅ 81 条 | `a.java:8273-8279`；槽 6 `TAK_53_58` |
| 2 | `Lay{floor}` | `pending_floor=floor`、`floorTransitioning=true`（**`current_floor` 不变**） | `overlay_text=None`、`dialog_phase=5`（切层演出） | ✅ 1 条（槽 0 `LAY_2`） | `a.java:8361-8367` |
| 3 | `Gin{kind,value}` | `kind=0` ⇒ 拾取类型 `value` 的道具（`item_stack_size`/`inventory_entry_count`）；`kind=1` ⇒ `gold += value` | 金币 popup（需要玩家像素作锚） | ✅ 4 条（槽 26–29） | `a.java:8430-8443`；槽 26 `GIN_1_1000` |
| 4 | `Res{}` | `difficulty_index==0` 时重置 10 项：三色钥匙、`item_stack_size`、`gold=4`、`alchemy=0`、`min_floor=1`、`max_floor=3`、`atk=10`、`def=10`、`hp=400`、`inventory_entry_count=0` | 无 | ✅ 1 条（槽 0 `RES_0`） | `a.java:8476-8492` |
| 5 | `Add{floor,ty,cx,cy}` | 目标层 `spawn_entity`（**不切层、不改 `current_floor`**）+ 两次楼层存档/装载；Java 在回装当前层时**调 `initCellGrid()`** | 无（跨层，不可见） | ✅ 1 条（槽 7） | `a.java:8444-8471` |
| 6 | `See{tx,ty,page,page_end,flag}` | `dialog_target=(tx,ty)`（**跨拍存活**）；`flag==1 ∧ findPath 成功` ⇒ `pathArrowsVisible=true`、`walkPhase=0`（**无 break，落穿到 case 0**）；`dialogPageStart/End` 为**死字段** | 开页 `page`、`dialog_phase=6`、相机 ease 到 `(tx,ty)` | ✅ 30 条（`flag=1` 7 条：槽 40/41/42/46/47/48/50） | `a.java:8493-8515`；槽 40 `SEE_3_10_166_166_1`；字段存活：`a.java:4066` 重锚、`:2894-2895` 箭头绘制 |
| 7 | `Gut{target}` | `script_line_flags[current_script_index]=true`（**写旧索引**）→ 整机重载到目标层脚本入口：`current_script_index=target`、`script_cursor=0`、`dialog_phase=0`、`script_walk_armed=false`、相机锁 0 | 换场景 + 相机 | ✅ 7 条（0→1 … 62→63） | `a.java:8316-8319`；`crates/game-core/src/scene_persist.rs:501-521` |
| 8 | `Mov{cursor}`（3 参 MOV_x_y / 5 参 MOV_t_x_y_v1_v2） | `dialog_target=(x,y)`；**三参** findPath 成功 ⇒ `script_walk_armed=true`、`dialog_phase=3`、`walkPhase=0`（玩家位移在后续 tick 产生）；**实体臂** ⇒ `route_entity`/`routePhase=0`/`route_start`/（`type==72` ⇒ `anim_row=1`），findPath 返回值不判 | 相机锁玩家（三参）/ 实体路线动画（实体臂） | ✅ 47 条 | `a.java:8280-8315`；槽 0 `MOV_0_5_11`、槽 7 `MOV_72_3_2_8_4` |
| 9 | `DesAll{want}` | 逆序扫全表 `!removed ∧ type==\|want\|` ⇒ `removed=true`（负参形态，**不取格坐标**） | 无（最小地图重建属派生） | ✅ 40 条之一部分（负参 5 条全在槽 63） | `a.java:8320-8333` |
| 10 | `DesAt{ty,cx,cy}` | `findBlockingEntity(cx,cy,ty) >= 0` ⇒ 该实体 `removed=true` | 无 | ✅ 40 条（主体） | `a.java:8334-8344`；槽 0 `DES_70_5_8` |
| 11 | `Swd{}` | 全表 `type==81` ⇒ `visible=true`、`type=4`、`state=2`、`anim_row=8`、重派 sprite size | 精灵换图 | ✅ 3 条（槽 3/9/11） | `a.java:8345-8359` |
| 12 | `Ces{ty,cx,cy}` | `spawn_entity(ty,cx<<5,cy<<5,0)` + `markEntityDesRemoved(idx)`；`script_delay = category(type)` | 相机居中 + ease 目标 = 新实体像素 | ✅ 48 条 | `a.java:8395-8428`；槽 0 `CES_84_6_11` |
| 13 | `Glv{}` | **无**（不取参、不推进额外语义，仅游标） | 无 | ✅ 1 条（槽 37 `GLV_1`） | `a.java:8473-8475` |
| 14 | `End{}` | `game_mode=20`（Ending）+ `dispatchModeEntry()`；**`current_floor`/玩家/背包不变** | 结局图 + 打字机排版 `dialogueTexts[260]`（`layoutWrappedText(…,209,(line_h<<1)+12,pos)`）；**无退出调用** | ✅ 1 条（槽 65） | `a.java:8516-8520`；case 20 = `a.java:3039-3080` |
| 15 | `Sms{}` | **无** | 无 | ✅ 1 条（槽 66；与 DC-5 的不可达槽 67 相邻但自身可达） | `a.java:8521-8522` |
| 16 | `RosMove{px,py}`（ROS_1） | `player_pixel=(px<<5, py<<5)`（含移位归格） | 相机锁 0（回玩家） | ✅ 1 条（槽 0 `ROS_1_4_7`） | `a.java:8368-8370` |
| 17 | `RosEquip{tier}`（ROS_2） | 强制装备 `tier`（`tryEquip(tier,true)`）⇒ `equipped_weapon/armor` + bonus | 装备弹窗面（由 `m_082` 钩子承载） | ✅ 2 条（槽 0 `ROS_2_0`/`ROS_2_6`） | `a.java:8371-8372` |
| 18 | `RosHp{v}`（ROS_3） | `player_hp=v` | 无 | ⚠️ **语料零命中**，保留 | `a.java:8373-8374` |
| 19 | `RosFace{v}`（ROS_4） | `facing = (byte)v`（**Java `byte` 字段 ⇒ i8 截断**） | 无 | ✅ 23 条（取值集 {0,1,2,3}） | `a.java:8375-8376`；槽 0 `ROS_4_1` |
| 20 | `RosAtk{v}`（ROS_5） | `player_atk=v` | 无 | ✅ 2 条（槽 53/55 `ROS_5_510`） | `a.java:8377-8378` |
| 21 | `RosDef{v}`（ROS_6） | `player_def=v` | 无 | ✅ 2 条（槽 53/55 `ROS_6_510`） | `a.java:8379-8380` |
| 22 | `RosNoop{}`（ROS 其余子码） | **无**（不取参、不推进 ⇒ 下一拍重解析同一 token，Java 语义如此） | 无 | ⚠️ 语料零命中；**必须保留解析**（否则合法行落 `None` ⇒ 停摆语义错） | `a.java:8381` default；`crates/game-core/src/script_ops.rs:542` |
| 23 | `Mvs{}` | **不可达**（数据域 68 条零 MVS） | — | ☠️ **DC-2**（`docs/spec/dead-code-rulings.md`） | `a.java:8523-8592`；`crates/game-core/src/script_ops.rs:316-395` |

### §A.2 三条特别裁决

#### §A.2.1 `SEE` 的 `flag`：跳过 UI 时**必须**仍然置 `pathArrowsVisible`

A 级证据（`a.java:8493-8515` 的 `switch(seeModeFlag)`）：

```
case 1:
    if (findPath(playerCellX, playerCellY, dialogMoveX, dialogMoveY)) {
        pathArrowsVisible = true;
        walkPhase = 0;
    }
    // ← 无 break
case 0:
default:
    dialogPhase = 6;
    easeCameraToTarget(dialogMoveX, dialogMoveY);
```

结论：

1. `flag` **不是**开关"哪一部分语义执行"，而是"在 case 0 的公共尾部之前**追加**寻路"——因为无 `break`，`flag==1` 也必然落穿到 `dialogPhase=6` + 相机 ease。这与 spec 里"e==1 时门控 walkPhase"的措辞一致，但**不构成"跳过 UI 可以连 flag 一起跳过"**。
2. `pathArrowsVisible` 是**世界态而非装饰**：A 级调用图证据 —— 写入 `crates/game-core/src/scene.rs:494`（setter），读者含 `:1201`（walkto 箭头绘制）、`:1708/:1718/:1735/:1744`（路径消费与清除）、`:3332`（清）、`:4180`（点地归一）。它驱动寻路消费，删掉它会让"path 消费"分支永不进入。
3. 因此 **headless handler 的跳过集合必须精确到"表现"**：`SEE` 可跳 `openDialogPage`、`dialogPhase=6`、`easeCameraToTarget`；**不可跳** `findPath` + `pathArrowsVisible=true` + `walkPhase=0`。同理 `MOV` 三参可跳相机、不可跳 `findPath`/`script_walk_armed`/`dialog_phase=3`。
4. `dialogPageStart`（`a.java:274`）与 `seeModeFlag`（`:270`）是**死字段**（写后仅同方法内读），不必进 headless 最小状态集；但 `dialogMoveX/Y`（`:265-266`）**必须进**——它跨拍被 `:4066` 与 `:2894-2895` 读取（GameScene 侧同名 `dialog_target`，`crates/game-core/src/scene_dialog.rs:155-166` 在 `TypewriterStep::Confirm` 时用之重锚相机）。

#### §A.2.2 等待/结束时序语义（`TAK`/`SMS`/`ROS_*`/`MVS`/`GLV`/`END`）

| 指令 | 是否阻塞脚本机 | 阻塞的解除条件 | 证据 |
|---|---|---|---|
| `TAK` | **是**（`dialog_phase=1`，脚本机停在该相位等按键） | 打字机翻页到 `page > page_end` ⇒ `openDialogPage` 的 else 支 ⇒ `dialog_phase=0` | `a.java:8273-8279`；`crates/game-core/src/scene.rs:1050`（else 支 `phase=0`） |
| `SEE` | **是**（`dialog_phase=6`，且 `phase==6` 时打字机需 `camera_at_target()` 才推进） | 同上 + 相机到位 | `crates/game-core/src/scene_dialog.rs:119` |
| `MOV` 三参 | **是**（`dialog_phase=3` 脚本行走） | 步尽 ⇒ `script_walk_armed=false` + `phase=0` | `crates/game-core/src/scene_dialog.rs:170-207` |
| `MOV` 实体臂 | **是**（`dialog_phase=2` 实体路线） | `tick_entity_route` 收敛 | `crates/game-core/src/scene_dialog.rs:169` |
| `LAY` | **是**（`dialog_phase=5` + `floorTransitioning`） | 切层完成 | `a.java:8361-8367` |
| `END` | **终止**（`game_mode=20`，`crates/game-core/src/scene.rs:2975` 的 `if game_mode == Dialog` 不再成立 ⇒ 脚本机永不再 dispatch） | 不可解除 | `crates/game-core/src/scene.rs:2975`、`:3253-3256` |
| `SMS`/`GLV`/`ROS_*`/`GIN`/`CES`/`DES`/`SWD`/`ADD`/`RES` | **否**（同拍继续消费下一 token） | — | 各臂末尾 `scriptCursor=lastDelim+1`，无相位写 |
| `GIN` | 否，但**无条件尾递归**（唯一显式递归臂 `a.java:8395-8397`） | — | `crates/game-core/src/script_ops.rs:587-589` |

> 注意 `TAK` 的行尾特例：`TAK` 使 `phase=1`，而**"行已消费完"的检测只在 `phase==0` 时发生**（`crates/game-core/src/scene_dialog.rs:112-118` 只有 `0 =>` 支调 `execute_script_instruction`）。所以行尾 `TAK` 的收尾必然发生在"对话结束后的下一拍"，而不是同一拍——这正是 PE-01 的伤害面。

#### §A.2.3 死码裁决对齐

| DC | 内容 | 对本表的约束 |
|---|---|---|
| DC-2 | `MVS` 第 16 码 + `m_107` ☠️ | 23 变体里的 `Mvs` **保留**（行为只增不减，AGENTS §1），但**不进 headless 路径的语义义务**——两 handler 都只需满足"A/B 对拍时对 `Mvs` 的分类一致"（`crates/game-solver/src/script_abs.rs:541` 已把它归 `Class::C`） |
| DC-5 | 槽 67 / `tryRunScene(67)` ☠️ | 语料 68 槽中**只有 0..=66 参与可达性论证**；槽 67 仅作数据守卫（`crates/game-solver/src/script_abs.rs:688`） |
| DC-1/3/4 | 箭头/联网/火花 | 与脚本管线无交集，本方案不涉及 |

### §A.3′ 免 tick 判据与槽清单重算（2026-11-16 第二轮，取代 §A.3）

#### §A.3′.1 精确判据（三条同时成立才算 FF-safe）

FF（fast-forward）= **不推进任何 tick、把该槽求值到"槽结束"**。它成立需要三件事，缺一不可：

1. **槽内每条指令都"即时到达终态"**：执行那一次调用结束后，该指令的全部世界态写已经落定，不需要后续 tick 再推进（排除写入相位 1/2/3/5/6 的 `TAK`/`SEE`/`MOV`/`LAY`，以及跨槽转移的 `GUT`）。
2. **每条 W 类指令的 delta 都能在当前世界态上直接 apply**：写面不依赖"tick 过程中的位置/路径演化"（这是 `MOV` 被排除的第二个理由：它的 `find_path` 结果依赖实体当前位置与路径缓存，而位置只在相位 3 的逐格行走里变）。
3. **槽尾终止态可静态算出**：入口门的 `phase=4` + `script_line_flags[idx]=true`（`idx != 32`）与 `lock_camera_on(0)`；三者都不需要相机逐帧收敛来产生**世界态**变化（相位 4 的 `camera_at_target()` 门只决定"何时回 Map"，不写世界态，`crates/game-core/src/scene_dialog.rs:251-255`）。

⇒ **「免 tick」≠「全 P 类」≠「可跳过执行」。** P 类是"连 delta 都不用算"的**更强**条件；§A.1′ 已证明真 P 槽只有 5 个。因此 FF-safe 清单必须分档：能**跳过执行**的只有 A 档；B 档能"零 tick 求值"但**必须照常 apply delta**。v1 §A.3 把两者混为一谈，是复核 F5/F6 的同一个错误的两个面。

#### §A.3′.2 68 槽归类（A 级，逐槽解析 296 条指令实测）

| 档 | 判据 | 槽数 | 槽号 |
|---|---|---|---|
| **A · 全 P（可整槽跳过执行）** | 槽内无任何 W 类指令（`GLV`/`SMS`/`RosNoop` 或空槽） | **5** | `37`、`39`、`58`、`59`、`60` |
| **B1 · 即时 W、零 delay** | 有世界态写但无相位、无 `script_step_delay` | **2** | `9`、`11` |
| **B2 · 即时 W、带 `script_step_delay`** | 同 B1，但 `CES` 置 `script_step_delay ∈ {5,8}` ⇒ 世界态等价、**tick 计数不等** | **4** | `13`、`18`、`25`、`38` |
| **C · 阻塞（必须驱动 tick）** | 含 `TAK`/`SEE`/`MOV`/`LAY`/`END`/`GUT` 任一 | **57** | 其余（含 v1 那 20 个 W 槽：`0,1,2,3,4,5,7,8,10,12,16,19,23,30,31,56,57,61,62,65`，它们只是 C 的子集） |
| 合计 | | **68** | |

**FF-safe 槽清单（= A ∪ B1 ∪ B2，11 槽）**：

```
9, 11, 13, 18, 25, 37, 38, 39, 58, 59, 60
```

**C 档（57 槽）的阻塞成因分解**（可叠加，故合计 > 57）：

| 阻塞原因 | 槽数 | 说明 / 槽号 |
|---|---|---|
| 仅 `TAK`（等按键） | **23** | `6,14,15,17,20,21,22,24,26,27,28,29,32,33,34,35,36,51,55,63,64,66,67` —— 只需按键，不需位移/换层；FF 若接"输入脚本"可进一步收窄，但本轮不承诺 |
| 含 `SEE`（相位 6 + 相机收敛 + 按键） | **14** | `40,41,42,43,44,45,46,47,48,49,50,52,53,54` |
| 含 `MOV`（相位 2 路线 / 相位 3 行走） | **16** | `0,1,2,3,4,5,7,8,10,12,19,30,31,56,57,61` |
| 含 `LAY`（相位 5） | **1** | `0` |
| 含 `END`（`game_mode=20`，终止） | **1** | `65` |
| 含 `GUT`（跨槽换行） | **7** | `0,16,23,56,57,61,62` |
| **阻塞且同时含即时 W 写**（混合槽，绝不可整槽 FF） | **32** | 这些槽若按 v1 表的"非 MOV/LAY/END/GUT ⇒ 可 FF"处理，会同时丢 delta 又跳过相位 |

#### §A.3′.3 红队点名槽的逐个复核

| 槽 | 内容 | 本轮结论 | 依据 |
|---|---|---|---|
| **42** | `TAK_171_171 SEE_7_10_172_172_0 SEE_7_10_173_173_1` | **必须 tick**（红队判定正确） | 两个 `TAK` 各把 `dialog_phase=1`（等按键翻页），两个 `SEE` 写 `dialog_target`/`path_arrows_visible`/`phase=6` 并要求相机收敛后才推进打字机（`scene_dialog.rs:123`）；`path_arrows_visible` 是跨拍世界态（`crates/game-core/src/scene.rs:494` 写、`:1708-1744` 消费） |
| `9`、`11` | `SWD `（单条） | **免 tick 成立**（但必须执行 SWD 的 5 个字段写） | `op_swd` 无相位写、无 delay；唯一"真 W 且 FF 安全"的证据点 |
| `25` | `CES_…`（单条） | **可同步求值**，但 tick 计数不等 | `script_step_delay` 唯一读者 `scene_dialog.rs:114-118`，纯节奏 |
| `13`、`18`、`38` | CES 族 | 同 `25` | 同上 |
| `6`、`14`…（23 个仅 TAK 槽） | `TAK_x_y` | **必须 tick** | 相位 1 停表等按键；且这是 PE-01 的原始靶子（v1 说"整页被跳过"，实际生产口径不触发，见 §B.2′） |
| `39`、`58`、`59`、`60`（空槽） | 内容 = 单字节 `\x1e` | **A 档但非 no-op**：入口门仍写 `flags[idx]=true`（`idx≠32`）+ `phase=4` + `lock_camera_on(0)` | `a.java:8603-8610`；`.trim()` 不吃 `0x1E`（`scene.rs:253-256`）⇒ 行长 1 ⇒ `0+3 > 1` 入口门命中 |
| `67` | `TAK_264_268 `（12 字节，文件末尾无 `0x1E`、无换行） | ☠️ **DC-5 不可达**；且在 (a) 口径下**先 panic**：`.trim()` 吃掉尾空格 ⇒ 行长 11 ⇒ `parse_script_int(line, 8, ' ')` 无分隔符 ⇒ `expect("脚本整数的分隔符缺失")`（`crates/game-core/src/script.rs:262`） | `crates/game-solver/src/script_abs.rs:686-700` 既有测试注释已写明同一事实 |

#### §A.3′.4 对 §D 的硬约束（FF 落地时必须满足）

1. **A 档只省"执行"，不省"收尾"**：FF 仍须写 `phase=4` + `flags[idx]=true`（`idx≠32`）。
2. **B 档必须执行 delta**：`9`/`11`/`13`/`18`/`25`/`38` 的 `spawn`/`remove`/`SWD` 五字段写不能因"免 tick"而跳过——这正是 v1 表会把 FF 变成 bug 生成器的地方（F6）。
3. **B2 档若观察面含 ticks，必须记"名义拍数"**：`script_step_delay` 5/8 拍不改变世界态，但改变 tick 计数；`Obs` 一旦纳入 `ticks`（§D.3），B2 档要么逐帧、要么把 delay 计入名义拍数，否则 T-parity-2 必红。
4. **旗标写入时点**：Java 只在入口门写 `flags[idx]=true`（`a.java:8603-8610`），所以阻塞相位槽（1/2/3/5/6）的旗标写入被**推迟到该相位收敛之后**。FF 若在槽内一并写，终值相同但时序不同——拿 `flags` 做门判定的求解器会提前看到"该槽已完成"。

### §A.3 派生结论：W/P 两类（供 §D 用的静态判据）〔v1 · 判据已作废，见 §A.3′〕

由 §A.2.2 直接把 23 变体切成两类：

- **P 类（可 fast-forward，无需 tick）**：`Tak`(表现+对话域)、`See`(取 `findPath` 结果后其余表现)、`Ces`、`RosMove`、`DesAll`、`DesAt`、`Swd`、`Gin`、`Add`、`Res`、`Glv`、`Sms`、`RosEquip/Hp/Face/Atk/Def`、`RosNoop`。
- **W 类（必须驱动 tick，或必须交给真实 tick 机）**：`Mov` 三参（玩家位移）、`Mov` 实体臂（路线收敛）、`Lay`（换层）、`End`（终止）。
- **递归转移**：`Gut` 引入新的脚本槽（`script_line_flags` 写入 + 重载），被引入槽须重新分类 ⇒ W 类集合对 `Gut` 图取闭包。

按语料实测（A 级）：含 `Mov`/`Lay`/`End`/`Gut` 的槽 =

```
0,1,2,3,4,5,7,8,10,12,16,19,23,30,31,56,57,61,62,65   （20 槽）
```

⇒ **48/68 槽是"纯 P 类"**，其世界态增量可以完全在纯函数里算到底（不需要 `GameScene`、不需要 tick）。**20 槽**必须走驱动路径。这条比例是 §D 全部性能主张的唯一依据（且它是**上限**：`Mov` 的 `findPath` 失败时不必 tick，运行时还会更优）。

> 【2026-11-16 复核更正 · 见 §A.3′】**上面两段整段作废。** ① 判据错：`P` 档的定义是"无写面"，而 v1 把"无相位跃迁"当成了"无写面"⇒ 43 个槽被判错；② 数字错：真·零写面槽 **5** 个（`37,39,58,59,60`），可零 tick 求值的 **11** 个（`9,11,13,18,25,37,38,39,58,59,60`），其余 **57** 槽必须驱动 tick；③ "纯 P 类"与"可 fast-forward"**不是同一集合**，混用会同时丢状态增量（F6）与跳过阻塞相位（F5）。§D 的性能主张据此改写为"11/68 槽可零 tick 求值"，且**其中 6 槽仍必须执行 delta**。

---

## §B 执行器统一

### §B.1 结论：**不等价**，已确证 13 处

逐 op 比对方法：把 `a.java` 23 臂（`reference/src/deobf/a.java`，见 §A.1 行号）作为裁判，分别与 `crates/game-core/src/script_ops.rs` 的 `op_*`、`crates/game-core/src/script.rs:284-519` 的臂逐字段对照；控制流分歧用 `javap -p -c` 反查 `a.class`（`javap` 输出中 `executeScriptInstruction` 起于 offset 0，`parseScriptInt` 起于下一条方法，方法体 0..2431）。

**结论：`GameScene` 侧 9/13 处忠实，`ScriptEngine` 侧 4 处为虚构/错接。**

### §B.2 不一致账本（PE = Pipeline Equivalence）

| 编号 | 位置（engine 面） | 分歧 | 谁对 | 严重度 |
|---|---|---|---|---|
| **PE-01** | `crates/game-core/src/script.rs:511-519` | **尾部收尾门**：`if script_cursor >= line.len() { phase=AwaitCamera; flags[idx]=true; snap_camera }`，在**同一拍**执行完末命令后立刻写 | **GameScene（`crates/game-core/src/scene_script.rs:14-36`，仅入口门）+ Java** 对。javap：入口 `22: if_icmpgt 2401`（`cursor+3 > len` ⇒ 2401）；`2401-2430` = `dialogPhase=4` / `if idx!=32 {flags=true}` / `lockCameraOn(0)`；尾部汇聚点 `2386: getfield scriptCursor … 2397: if_icmpge 2431` ⇒ `if (cursor < len) return;` 否则**直接 `2431: return`（no-op）** | **~~P0~~ → P1**（【复核更正】见 §B.2′） |
| **PE-02** | `crates/game-core/src/script.rs:493-497` | `END` 调 `host.exit_application()`；`GameScene`（`crates/game-core/src/script_ops.rs:269-276`）调 `set_dialog_char(1)` + `layout_wrapped_text_limited(overlay_text,209,(line_h<<1)+12,1)` | **双方各有一半错**：Java `8516-8520` = `gameMode=20; dispatchModeEntry(); cursor`，**无 exit 调用**（`exit_application` 是 engine 虚构）；case 20（`a.java:3039-3080`）确实做结局图 + `layoutWrappedText(dialogueTexts[260],209,(text_line_height<<1)+12,typewriterPos)` ⇒ GameScene 的 layout 面是对的 | **P0** |
| **PE-03** | `crates/game-core/src/script.rs:365-371` | `LAY` 把 `m_119(currentFloor)`（**存档**）实现成 `host.reload_floor_entities(current_floor)`（**装载**）；且写 `layer_byte`/`flag_bool_16` | **GameScene 对**（`crates/game-core/src/script_ops.rs:68-74`：`save_floor_state()` + `set_pending_floor` + `set_transitioning(true)`）。`m_119` = `saveFloorState`（`a.java:9379`，写 `f_byte_arr2_05[floor]=blob`），`m_122` = `loadFloorEntities`（`a.java:9455`） | **P0** |
| **PE-04** | `crates/game-core/src/script.rs:390` | `ROS_4` 直接 `facing_direction = v`，**无 `(byte)` 截断**；GameScene `set_facing((v as i8) as i32)`（`crates/game-core/src/script_ops.rs:300`） | **GameScene 对**（`a.java:8375` `facingDirection=(byte)parse(' ')`）。当前不可观测（语料 `ROS_4` ∈ {0,1,2,3}） | P2（潜伏） |
| **PE-05** | `crates/game-core/src/script.rs:407-410` | `CES` 的 `script_delay` 取 `type_categories[t]` → `Category::from_raw(..).scene_delay()`（**静态夹具表**）；GameScene 取 `entity_category_with_floor(ty,floor)`（`crates/game-core/src/subsystem/script_host.rs:96`） | **GameScene 对**：Java 的 `renderCategoryByType` 是**可变世界态**，`m_122` 内按层改写（`a.java:9679` `[72]=32`、`:9681` `=8`）⇒ 必须走"含层覆写的类目表"，Rust 对应 `crates/game-core/src/resource_cfg.rs:319` `floor_category` | P1 |
| **PE-06** | `crates/game-core/src/script.rs:459` | `RES` 末项写 `inventory_entry_count=0`；GameScene 末项写 `persist.journal.clear()`（`crates/game-core/src/script_ops.rs:114`） | ~~**GameScene 对**~~ **engine 对**（【复核更正】见 §B.2′）（Java **`a.java:8489`** 写 `inventoryEntryCount=0`；`GameScene` 无此字段 ⇒ engine 是唯一正确的一条；`journal` 是 Rust 侧存档日志，Java 无对应重置 ⇒ `journal.clear()` 属**多写待裁**，见 §B.5 R-2） | P1 |
| **PE-07** | `crates/game-core/src/script.rs:423-441` | `ADD` 四次 `reload_floor_entities`，**无 `save_floor_state`、无 `initCellGrid`**；GameScene 两次 `save_floor_state()` + 受 `floor_assets[..].is_some()` 守卫的 reload，**亦无 `initCellGrid`**（`crates/game-core/src/script_ops.rs:120-143`） | **两者都与 Java 有差**（`a.java:8444-8471`：`m_119(cur)`→`m_122(floor)`→spawn→`m_119(floor)`→`initCellGrid()`→`m_122(cur)`）。⇒ **归入 §B.5 R-1（需 trace 裁定）** | P1 |
| **PE-08** | `crates/game-core/src/script.rs:462-492` | `SEE` 写 `camera_anchor_x/y` + `host.set_camera_anchor_target(a,b)`（**格坐标**）+ `walk_request_flag=true`；GameScene 写 `dialog_target` + `set_ease_cam_x/y`（居中偏移）+ `set_ease_target_x/y(tx<<5,ty<<5)`（**像素**）（`crates/game-core/src/script_ops.rs:149-166`） | **GameScene 对**（`a.java:8668-8673` `easeCameraToTarget(cx,cy)` 内部 `<<5`；`a.java:4066` 用 `dialogMoveX/Y` 重锚）。engine 的 `walk_request_flag` 与 GameScene 的 `path_arrows_visible` **不是同一字段**（后者见 §A.2.1 的调用图） | P1 |
| **PE-09** | `crates/game-core/src/script.rs:295-336` | `MOV` 写 `camera_anchor_x/y`（应为 `dialog_target`）；实体臂只对 `type==ROUTE` 调 `set_entity_route_flag`，**无 `route_phase`/`route_start`**；GameScene 写 `dialog_target` + 实体臂 `set_route_entity`/`set_route_phase(0)`/`route_start`（`crates/game-core/src/script_ops.rs:175-207`） | **GameScene 对**（`a.java:8290-8305` 写 `f_int_129/130`=`entityPixel>>5` 与 `routePhase=0`） | P1 |
| **PE-10** | `crates/game-core/src/script.rs:337-341` | `GUT` 直写 `script_line_flags[cur]`，**无越界守卫**；`ScriptResolver` 有 `if cur < len`（`crates/game-core/src/subsystem/script_host.rs:76`） | **Resolver 对**（越界会 panic，`68` 是既定语料上界；`cur != 32` 特例见 `crates/game-core/src/script.rs:277`（入口门）/`:514`（尾部门）） | P2 |
| **PE-11** | `crates/game-core/src/script.rs:204-206` | `load_level_script` 的 `idx == 31` 特例（`reload_floor_entities(current_floor)`）；`GameScene` 侧**无**此特例（`crates/game-core/src/scene_persist.rs:501-521`，注释记为 todo） | **未决**——两侧都无 A 级证据指出 Java 在此处做什么；须查 `loadLevelScript`（`a.java:7997-8012`）是否隐含该刷新 | P1 |
| **PE-12** | `crates/game-core/src/script.rs:503-510` | `MVS` 只 `host.mvs_exchange()`；`GameScene` 内联整段（`crates/game-core/src/script_ops.rs:316-395`） | **不可判**（DC-2 死码，68 条零命中）。保真面要求：**两者都不得删**，且 headless 侧按 `Class::C` 处理 | P3 |
| **PE-13** | `crates/game-core/src/script.rs:249-260`（`step`）vs `crates/game-core/src/scene_dialog.rs:112-118` | 引擎把"`phase==ScriptStep` 才 execute"藏在 `step()` 里；生产把"相位分派"放在 `tick_dialog`（含 `script_delay` 递减）。**`script_delay` 的递减时机**：engine 在 `step()` 内 `scene_delay>0 ⇒ 递减`；生产在 `tick_dialog` 的 `0 =>` 支（`crates/game-core/src/scene_dialog.rs:113-118`）。**两者等价**（同一相位同一判据），但**engine 的 `step()` 无相机步进**（生产先 `step_camera_toward_target()`，`crates/game-core/src/scene_dialog.rs:111`）⇒ 若用 engine 顶替生产，**相机步进必须留在 tick 侧** | 生产侧结构对 | P1（迁移约束，不是 bug） |

### §B.2′ 复核更正（2026-11-16 第二轮）：PE-01 定级、PE-06 方向、S5 数量

#### §B.2′.1 PE-01：P0 → **P1（口径依赖的潜伏项）**

**结论**：`crates/game-core/src/script.rs:511-519` 的尾部门确实是 Java 没有的"多写"（A 级），但它在本仓**唯一生产口径下不可达** ⇒ 定级 P1，不是 P0。

**Java 控制流（A 级：`reference/src/deobf/a.java` 直读 + javap 偏移）**：

```
8270:  if (var2 + 3 <= var1.length()) {                          ← 入口门
           … dispatch 链（TAK 8273 … MVS 8597）…
8598:  }
8600:  if (this.scriptCursor < this.dialogScriptLine.length()) {
8601:     return;                                                ← 尾检查
8602:  }
8603:  } else {                                                   ← 与 8270 配对
8604:     this.dialogPhase = 4;
8605:     if (this.currentScriptIndex != 32) { this.scriptLineFlags[cur] = true; }
8609:     this.lockCameraOn(0);
8610:  }
```

⇒ **尾检查不满足时方法直接结束、无任何字段写 = no-op**；`phase=4` / 旗标 / `lockCameraOn(0)` **只在入口门**（`cursor + 3 > len`）执行。javap：入口门体 `2401-2430`；尾部汇聚点 `2386: getfield scriptCursor … 2397: if_icmpge 2431` ⇒ 否则 **`2431: return`**。

**触发矩阵（本项目实测）**：

| 口径 | 槽 6 `TAK_53_58` 行长 | 末命令后 `cursor` | `cursor >= len` | 尾门 |
|---|---|---|---|---|
| **(a) 生产** `DialogData::from_fixtures`（`.lines().trim()`，`crates/game-core/src/scene.rs:253-256`） | **11**（保留 `" \x1e"`） | 10 | 假（`10 >= 11`） | **不触发** |
| **(b) 求解器** `dialogue_table()`（`split("\u{1e}\n")` + `trim_end_matches`，`crates/game-solver/src/script_abs.rs:158-166`） | **10**（无 `0x1E`） | 10 | 真 | **每槽末命令必触发** |

因此：**生产口径死、求解器口径活**。(b) 侧 `side_b::drive`（`crates/game-solver/src/script_abs.rs:480-524`，`:503` 直接调 `eng.execute_script_instruction`）**当前每一槽的末命令都在触发这个尾门**，只因 `Obs`（`:73-83`）不含 `dialog_phase` 才完全不可观测——与 §B.4 的诊断一致，但严重度归因必须从"命中每一槽的生产 bug"改成"**口径依赖 + 观察面缺失**"。

**为什么 P1 仍必须修**：① 它是 Java 里不存在的写面（AGENTS §1 保真红线）；② 一旦某条路径给 engine 喂无分隔符的行（新建 table、去掉 `0x1E`、S5 换 table 源），它立刻把 `TAK`/`SEE`/`MOV`/`LAY` 的相位改写成 `AwaitCamera`——就是 v1 描述的那个 bug；③ 它让 (a)/(b) 两侧对"谁写收尾"永远不同源（§B.4）。

**v1 的两处具体错误（保留对照）**：
- 「末命令消费后 `script_cursor == line.len()` **恒成立**」→ 实际恒为 `line.len() - 1`（行尾是 `" \x1e"` 两字节）。
- 锚 `a.java:8574-8580` → **伪锚**：该区间落在 `MVS` 体（`8525-8598`）内，与收尾门无关；真锚 `8600-8602`（尾检查）与 `8603-8610`（入口门）。

**与复核报告的唯一分歧（结论相同，措辞与机制需更严）**：红队「任何槽的末命令都不会让 cursor 到达 len」严格说不成立——夹具第 68 条（槽 67 `TAK_264_268`）在**原始字节**里是 `TAK_264_268 `（12 字节，`54 41 4b 5f 32 36 34 5f 32 36 38 20`，文件末尾既无 `0x1E` 也无换行），TAK 消费后 `cursor = 12 = len`。但生产口径下该行先被 `.trim()` 削成 11 字节，随即 `parse_script_int(line, 8, ' ')` 找不到分隔符而 panic（`crates/game-core/src/script.rs:262` 的 `expect`；`crates/game-solver/src/script_abs.rs:686-700` 的既有测试注释已记同一事实），**尾门依然不可达**。红队的复现脚本用 `lines[:-1]` 排除最后一行，属复现口径缺陷；P1 结论不受影响。

#### §B.2′.1b 追加证据（同日，比 §B.2′.1 更硬的口径证据）：真正错的是**语料口径**，不是尾门

A 级新证据（直读 `reference/src/deobf/a.java`）：

1. **`a.java:1124` 起的 `levelScriptLines = new String[]{ … }` 是常量数组字面量，行尾是一个空格、没有 `0x1E`**。例：第一条 = `"CES_84_6_11 … RES_0 GUT_1 "`（实测 **164** 字节，末 8 字节是 `"0 GUT_1 "`）。
2. **Java 全文件不对脚本行做任何修剪**：`grep -n "trim()" reference/src/deobf/a.java` **零命中**；`loadLevelScript` 直接 `this.dialogScriptLine = this.levelScriptLines[this.currentScriptIndex]`（`a.java:8051`）。
3. **游标推进吃掉了行尾空格**：`parseScriptInt` 写 `lastDelimPos`（`a.java:8613-8619`），各 op 用 `scriptCursor = lastDelimPos + 1`（如 `a.java:8277`）⇒ **末命令消费后 `cursor == line.length()`** ⇒ **Java 的尾门在每一行的末命令后都成立**。

实测佐证：序章行 `TAK_255_259 TAK_165_165 SEE_3_10_166_166_1 ` 行长 **43**，末命令 `SEE` 消费后 `cursor = 43 = len` ⇒ 同一拍触发尾门。

⇒ 把 §B.2′.1 的「生产口径死」精确化为「**Rust 生产夹具口径死**」，三种口径必须分清：

| 口径 | 行尾形态 | 行长（序章行） | 末命令后 `cursor` | 尾门 |
|---|---|---|---|---|
| **Java 原文**（`a.java:1124` 字面量） | `"… "`（空格，无 `0x1E`） | 43 | 43 = len | **触发**（每行末命令） |
| **Rust 求解器 / 金测试**（`crates/game-solver/src/script_abs.rs:158-166`、`crates/game-core/tests/script_golden.rs:15-19`：`split("\u{1e}\n")` + `trim_end_matches('\u{1e}')`） | `"… "` | 43 | 43 = len | **触发** |
| **Rust 生产**（`DialogData::from_fixtures`，`crates/game-core/src/scene.rs:253-256`：`.lines().map(\|l\| l.trim())`） | `"… \x1e"` | **44** | 43 = len−1 | **不触发** |

原因：Rust 的 `str::trim()` 只认 **Unicode 空白**，`U+001E` 不是空白 ⇒ 漏在行尾的是 `0x1E` 而不是空格，末命令吃掉空格后仍差 1 字节。

**因此缺陷定位改判**：不是"engine 比 Java 多了一个尾门"（v1），也不是"这尾门永不触发、无所谓"（复核）。**真正的缺陷是 `from_fixtures` 少剥 `0x1E`**，把 Java 每行末命令都会做的一次收尾写关掉了。后果（逐行末命令）：

- Java：同一拍写 `phase=4` + `flags[idx]=true`（`idx≠32`）+ `lockCameraOn(0)`，`cursor == len`。
- Rust 生产：该拍不写；**下一拍**入口门（`cursor+3 > len` 仍成立）补写同样三样 ⇒ **终值相同、时点晚 1 tick、`cursor` 恒少 1**。
- 这正是"相位/拍数不进 `Obs` 就看不见"的那类差异（§D.3），也是 T-parity-2 的非空理由。

**仓库内活体证据**：`docs/spec/p3-script-interpreter.md` §3.1 记的**序章冻结态**（`cursor=43`、`phase=4`、"`SEE` 已置 6 被覆写"）就是这条门——语料槽 40 = `TAK_255_259 TAK_165_165 SEE_3_10_166_166_1 `（Java 行长 43）。Java 下它靠**尾门同一拍**把 6 覆写成 4；Rust 生产口径行长 44，靠**下一拍入口门**覆写。⇒ 该冻结态在两侧"终值相同、拍点不同"。

**定级与修法（据此改判，与 v1 相反）**：维持 **P1**（不改变对话可见语义，只错 1 拍 + 1 个游标值），但修法不是"把 engine 尾门删掉/对齐 GameScene"——那会把 Java 唯一正确的收尾门也删掉。正确修法是：
1. `DialogData::from_fixtures` 的口径改成与 Java/求解器一致（`split('\u{1e}')` 后剥掉 `0x1E`），使尾门在两侧同拍触发；
2. `GameScene` **缺**尾门的问题由 S5（换 engine）自然消灭，不再单独补；

#### §B.2′.2 PE-06：方向写反（**engine 对**）

- Java：`a.java:8478 if (this.difficultyIndex == 0) {` … **`a.java:8489 var16.inventoryEntryCount = 0;`**（v1 引的 `:8490` 其实是 `}`）。
- 全仓 `inventory_entry_count` 只存在 engine：`crates/game-core/src/script.rs:77`（字段）、`:126`（默认 0）、`:459`（`RES` 写 0）。
- `GameScene::op_res`（`crates/game-core/src/script_ops.rs:101-116`）**无此字段**，末行是 `self.persist.journal.clear()`（`:114`）。

⇒ **engine 对**。v1 判"GameScene 对"若按 S4b 落地（把 engine 改向 GameScene 对齐），会**删掉唯一正确的那条写**，并把 `journal.clear()`（Java 无对应）固化成"正确行为"。
**正确处置**：S4b 反向改——把 `inventory_entry_count = 0` 补进 `GameScene::op_res`；`journal.clear()` 单列为待裁多写（§B.5 R-2：须存档往返 trace 裁定，Stage A 不擅自删，按 AGENTS §6 记 `docs/bug-ledger.md`）。

#### §B.2′.3 S5 调用点数量：9 / 28 / 22 三个数都不对

见 §S5′（A 级实测 **31 个调用表达式** + 2 个定义 + 1 个函数指针引用）。

### §B.3 必须先修哪一处

**顺序不可换：PE-01 → PE-02 → PE-03 →（PE-05/PE-08/PE-09）→（PE-07/PE-11 需 trace 裁定）。**

理由（按伤害面排序）：

1. **PE-01 命中每一槽**。语料行**以尾空格结尾**（`a.java` 的 `dialogScriptLine` 亦然）⇒ 末命令消费后 `script_cursor == line.len()` **恒成立**，所以 engine 的尾部门对**每一条脚本的最后一个命令**都触发。 【复核更正 · 见 §B.2′.1】**本条的前提与后果都改**：① 实际恒为 `line.len() - 1`（行尾是 `" \x1e"` 两字节）⇒ 生产口径不触发，只有求解器 (b) 的 10 字节表才每槽触发；② 定级 P0 → **P1**；③ 下列三条"反例"在**生产口径**下不成立（它们只在 (b) 口径成立，且因 `Obs` 无 `phase` 而不可观测）。原反例保留如下，供口径对照：
   - 槽 6 `TAK_53_58`：engine 消费 TAK（`phase=1`）后立刻尾门把 `phase` 改写成 `AwaitCamera`（`crates/game-core/src/script.rs:513`）⇒ `crates/game-core/src/scene_dialog.rs:119` 的 `1|6` 支永不进入 ⇒ **整页对话被跳过、不等按键**。
   - 槽 0 结尾 `GUT_1`：`GUT` 的 `load_level_script` 把 `phase` 重置为 `0`，尾门随即改写成 `AwaitCamera` ⇒ 新脚本不启动。
   - 槽 65 `TAK_127_128 END_0`：`END` 之后尾门额外写 `flags[65]=true` + 动相机；Java 此处是 no-op ⇒ **多出一次旗标写**（`script_line_flags` 被 `try_run_scene` 的多条谓词读取，`crates/game-core/src/scene.rs:927-1031`）。
2. **PE-02/PE-03 改变"终止"与"换层"两类不可逆语义**，且都是 engine 单侧虚构/错接，修它们**不影响生产**（生产不跑 engine）⇒ 零风险收益。
3. PE-05/08/09 是"两个世界态字段语义不同"（类目表来源、`dialog_target` vs `camera_anchor`、`route_phase`），修起来是**字段对齐**，但必须同步改 `Obs`（§D.5）否则仍不可见。
4. PE-07/PE-11 **不能靠读代码定**（涉及 `m_119`/`m_122`/`initCellGrid` 与 `idx==31`），必须先做 trace。

**证明手段（这是本文对"如何证明"的正面回答）**：新增 **L2b 逐 tick 差分**（`gates/cli.py trace` 的既有 L2b 面，见 `docs/spec/p3-script-interpreter.md` §4）：
- 同一脚本槽、同一初态，两侧各记 `(tick, phase, cursor, flags[68], player_px/py, gold, hp, atk, def, facing, entities)`；
- **要求逐 tick 全等**，而不只是终态全等；
- 现成的最小反例向量 = 槽 6 / 槽 0 / 槽 65（PE-01）+ 槽 0（PE-03）+ 槽 7（PE-07）。
- 现有 `Obs`（`crates/game-solver/src/script_abs.rs:73`）**必须扩** `dialog_phase` 与 `ticks` 两项，否则 PE-01 永不可见（§B.4）。

### §B.4 为什么现有 A/B 对拍抓不到 PE-01（方法论缺陷，A 级）

三条叠加原因：

1. **`Obs` 不含 `dialog_phase`，也不含 tick 计数**（`crates/game-solver/src/script_abs.rs:73`：`cursor/index/flags/stats[8]/facing/px/py/entities`）。PE-01 的唯一可观测差异就是"相位被提前一拍改写"，**恰恰不在观察面里**。
2. **两侧语料字符串不同**：`side_a::drive` 用 `sc.dialog.dialog_data.scripts[idx]`（`crates/game-solver/src/script_abs.rs:193`），其来源是 `DialogData::from_fixtures()`（`.lines().trim()`）——**行尾保留 `0x1E` 字节**；`side_b` 用 `dialogue_table()`（`split("\u{1e}\n")`）——**行尾无 `0x1E`**。实测槽 6：(a) 行 = 11 字节、(b) 行 = 10 字节。
3. 于是"行已消费完"由**不同的门**检测：`a` 侧 `cursor=10`，`10+3 > 11` ⇒ **入口门**命中（写 `phase=4`+`flags[6]=true`，在**下一拍**）；`b` 侧 `cursor=10`，`10 >= 10` ⇒ **尾部门**命中（写 `phase=AwaitCamera`+`flags[6]=true`，在**本拍**）。**末态 `cursor/flags` 完全相同**，只有"第几拍"和"谁写的"不同 ⇒ `assert_eq!(oa, ob)` 恒绿。

> 推论（对 §D.5 的硬约束）：**2×2 矩阵成立的前提是先统一语料口径**（单一 table 源、显式声明是否保留 `0x1E`）**并把 `phase`/`ticks` 纳入观察面**。否则新矩阵只是把同一盲点复制四份。

### §B.5 两处须由 trace 裁定（不得凭代码定）

| 编号 | 争点 | 现有证据 | 缺什么 |
|---|---|---|---|
| **R-1**（PE-07） | `ADD` 是否必须在回装当前层后 `initCellGrid()` | `a.java:8465-8469` 有 `initCellGrid()`；GameScene 无（`crates/game-core/src/script_ops.rs:120-143`）；engine 无 | 一次 `ADD` 前后的 `CellGrid` 全量 dump 差分（槽 7 `ADD_2_72_11_10`）。若 `spawnEntity` 不走 `initCellGrid` 的产物，则 GameScene 等价与否取决于 `initCellGrid` 的写面是否与 `m_122(currentFloor)` 重叠——**须实测** |
| **R-2**（PE-06） | `RES` 该不该 `journal.clear()` | `a.java:8476-8492` 无 journal 概念（Rust 侧新增的存档日志） | 存档往返 trace；若无 Java 对应，属 Stage A 保真面**多写**，应登记 bug-ledger 而非就地删（AGENTS §6） |
| **R-3**（PE-11） | `load_level_script(31)` 的 `reload_floor_entities` 特例 | engine `crates/game-core/src/script.rs:204-206` 有；GameScene 无（注释 todo，`crates/game-core/src/scene_persist.rs:501-521`） | `loadLevelScript`（`a.java:7997-8012`）逐字节语义 + 31 层实际进入路径的 trace |

### §B.6 `docs/spec/p3-script-interpreter.md` 需要原地更正（AGENTS §3.6）

| spec 原文 | 更正 | 证据级别 |
|---|---|---|
| §3.1「方法尾部恒执行 `if cursor < line.len() { return; }` 否则 `dialogPhase=4` + `scriptLineFlags[current]=true`(+`m_104(0)`)」+ R4「行尾门非 no-op」 | **尾部是 no-op**：`2386-2400` 只是 `if (cursor < len) return;`，不满足则落到 `2431: return`。`dialogPhase=4/flags/lockCameraOn(0)` **只在入口门 `2401-2430`**（`cursor+3 > len`）执行。 | **A**（javap 偏移 + 现成语料反例：槽 40 行尾 `SEE_…_1` 消费后 `cursor==len`，若尾部写 `phase=4` 则 BUG-006 的观测值不会停在 `phase=6`→`0` 的先后关系上；而 `crates/game-core/src/scene_script.rs:14-36` 的无尾门实现与金样 74 场景一致） |

> 更正写法（落地时照抄进 spec，不删原主张）：
> `> 【2026-11-16 更正】原文把 executeScriptInstruction 的尾部出口记成"非 no-op 收尾门"。javap 实测：尾部汇聚点 2386 只做 if (scriptCursor < len) return;，否则落到 2431: return，无任何字段写。dialogPhase=4 + scriptLineFlags + lockCameraOn(0) 仅在入口门（cursor+3 > len，偏移 2401-2430）执行。原 §3.1 与 R4 的"行尾门非 no-op"作废；Rust 侧 GameScene 实现（scene_script.rs:14-36）本就正确，ScriptEngine 的尾部门（script.rs:511-519）为多写，PE-01。`

---

## §C 命令模式与 handler 拆分

### §C.1 形态选择：为什么**不**把状态增量做成 `Outcome` 值

任务书要求的形状是 `Outcome { state_delta, presentation }`。评估结论：**这个形状适合当"观察快照"，不适合当"执行契约"**。三条硬理由（都可现场验证）：

1. **13 条命令写的是大数组内的元素**：`entities[i].removed/type/state/anim_row/visible`（`DES`/`SWD`/`CES`/`MOV`）、`script_line_flags[i]`（`GUT`）、`grid`（`ADD`）。值化 delta 要么深拷贝整表（每命令 O(entities)，求解器一次 gate 可能跑上千命令 ⇒ 分配爆炸），要么定义"路径式补丁"——而**补丁的合并语义会变成新的语义面**，两个 handler 的差异会从"实现"搬到"补丁合并"，可测性反而下降。
2. **`script_walk_armed`/`dialog_phase`/`pathArrowsVisible` 是跨拍控制态**，不是"终态可加和"的增量；`MOV` 的成功/失败分支决定后续**是否**继续消费，这个"控制流"无法表达为值。
3. **求解器要的是"同一状态上继续跑"**，不是"状态 A + delta = B"的代数。它已经在 `crates/game-solver/src/script_abs.rs` 用 `Snapshot`+双写同步（`:480`）证明了"值化观察面"的正确用法——**观察面值化、执行面原地**。

因此本方案采用：

- **执行契约 = trait 方法（`&mut`）**，delta 代码只有**一份**，宿主可换（§C.2/§C.4）；
- **观察契约 = 值类型 `Obs`/`Snapshot`**（已有，`crates/game-solver/src/script_abs.rs:64-160`），用于对拍与 trace——**这才是 `Outcome` 该出现的地方**，且只需扩字段不必重构。

### §C.2 trait 拆分（5 个，附覆盖率与缺口）

现状 4 个 trait 共 23 个方法（`HostCtx` 8 + `EntityService` 9 + `DialogSubsystem` 3 + `ItemService` 3），但 §A.1 表里约有 **14 个写点**没有 trait 覆盖，全靠 `op_*` 直写 `self.*`。缺口清单（逐条对应 §A.1 行）：

| 缺口写点 | 现在写在哪 | 应归入 |
|---|---|---|
| `dialog_target` 写（`SEE`/`MOV`） | `crates/game-core/src/script_ops.rs:150`、`:177`（`DIALOG_BLIND_ZONES:78` 已登记） | `ScriptWorld::set_dialog_target` |
| 相机 ease 居中偏移 + 像素目标 | `crates/game-core/src/script_ops.rs:159-162`、`:247-252` | `Presentation::ease_to_cell` |
| `script_delay` 由**含层覆写**的类目表算 | `crates/game-core/src/subsystem/script_host.rs:81-88` | `ScriptWorld::step_delay_for`（纯函数，但宿主提供类目） |
| `facing` 的 `(byte)` 截断 | `crates/game-core/src/script_ops.rs:300` | `ScriptWorld::set_facing`（截断在 **executor** 内做，trait 收 `i8`） |
| `pending_floor`/`transitioning`（`LAY`） | `crates/game-core/src/script_ops.rs:68-74` | `FloorWorld::begin_transition` |
| `save_floor_state`/`reload_floor_entities`/`init_cell_grid`（`ADD`/`LAY`） | `crates/game-core/src/script_ops.rs:120-143` | `FloorWorld` |
| `RES` 的 10+1 字段重置 | `crates/game-core/src/script_ops.rs:101-118` | `ScriptWorld::reset_run` |
| `END` 的 `game_mode` + 结局排版 | `crates/game-core/src/script_ops.rs:269-276` | `ScriptWorld::enter_ending` + `Presentation::layout_ending` |
| `GIN` 的金币像素锚 | `crates/game-core/src/script_ops.rs:78-99` | `ItemService::give_gold(v, player_px, py)`（**加参**，现在无参见 `crates/game-core/src/subsystem/item_service.rs:19`） |
| `MVS` 整段死码 | `crates/game-core/src/script_ops.rs:316-395` | `MvsBody::mvs_exchange`（保真保留，`Class::C`） |
| `GUT` 的旗标写 | `crates/game-core/src/subsystem/script_host.rs:74-80` | `ScriptWorld::mark_line_flag`（**统一到一处**，现在 engine 与 resolver 两份，PE-10） |
| `MOV` 的 `route_phase`/`route_start`/`anim_row` | `crates/game-core/src/script_ops.rs:184-189` | `EntityService::set_entity_route`（扩参，现在只 `set_entity_route_flag`，`:42`） |
| `DES` 的负参全表批删 | `crates/game-core/src/script_ops.rs:208-215` | `EntityService::remove_all_of_type`（已有，`:33`）✅ |
| `SWD` 的 5 字段群 | — | `EntityService::swd_convert_type81`（已有，`:35`）✅ |

**trait 草案（5 个，全部 `&mut self`，无一返回 delta 值）**：

```rust
/// ① 世界态（脚本能改的一切"会影响后续判定/存档"的字段）。
///    **唯一实现**：`CoreWorld<H>`（executor 内），`H` 是数据宿（场景 or 求解器）。
pub trait ScriptWorld {
    fn set_facing(&mut self, v: i8);                 // Java byte 截断已由调用方完成
    fn set_dialog_target(&mut self, cell: (i32, i32));
    fn step_delay_for(&self, ty: i32) -> i32;        // 含层覆写的类目表（PE-05）
    fn mark_line_flag(&mut self, idx: usize);         // 带 len 守卫 + idx!=32 特例
    fn reset_run(&mut self);                          // RES（R-2 待裁：journal）
    fn enter_ending(&mut self);                       // END: game_mode=Ending
}

/// ② 实体表（已有 EntityService，只补 3 个写点）
pub trait EntityService {
    // …既有 9 方法保持…
    fn set_entity_route(&mut self, idx: i32, phase: i32, start: (i32, i32));
    fn set_anim_row(&mut self, idx: i32, row: i32);
    fn remove_all_of_type_ordered(&mut self, ty: i32);   // DES 负参的逆序语义
}

/// ③ 楼层（LAY/ADD/GUT 的存档-装载-换层）
pub trait FloorWorld {
    fn save_floor(&mut self, floor: i32);
    fn load_floor_entities(&mut self, floor: i32);
    fn init_cell_grid(&mut self);
    fn begin_transition(&mut self, to: i32);
    fn reload_script_entry(&mut self, idx: i32);      // GUT 的整机重载
}

/// ④ 道具/货币（已有 ItemService，补像素锚）
pub trait ItemService {
    fn pickup(&mut self, kind: i32);
    fn give_gold(&mut self, amount: i32, anchor_px: (i32, i32));
    fn pickup_item(&mut self, item_id: i32);
}

/// ⑤ 表现（生产真做，headless 全空）
pub trait Presentation {
    fn snap_camera_to_player(&mut self);
    fn lock_camera_on(&mut self, target: i32);
    fn ease_to_cell(&mut self, cell: (i32, i32));
    fn ease_camera_centered(&mut self);               // cameraX=(vw-32>>1)-px 那一半
    fn open_dialog_page(&mut self, page: i32);        // 打字机/排版/说话人
    fn layout_ending(&mut self);
    fn popup_gold(&mut self, amount: i32, anchor_px: (i32, i32));
    fn exit_application(&mut self);                   // 仅 process 层用；脚本路径不得调用（PE-02）
}
```

**理由（每一刀为什么这么切）**：

- `ScriptWorld` 与 `Presentation` 分开，是因为**两者对 headless 的可跳过性不同**：`ScriptWorld` 全程不可跳（跳了就是错），`Presentation` 全程可跳（跳了只是没画面）。把二者混在 `HostCtx` 里正是现状 8 方法"混装两类"的病根（`crates/game-core/src/script.rs:135-152`：6 个表现类 + 2 个规则类）。
- `EntityService`/`ItemService` **不新建**，只补写点——已有 12 个方法对 4 个命令是够的，重建会引入无谓 diff。
- `FloorWorld` 必须独立：`LAY`/`ADD`/`GUT` 是**唯一三个跨层命令**，它们的写面（存档 blob、格栅、脚本入口重载）与"当前层"强相关，塞进 `ScriptWorld` 会让 headless 宿主被迫持有整层资产。
- `Presentation::exit_application` **保留但由脚本路径禁用**：PE-02 已证 Java 的脚本 END 不退出；方法留给进程层（菜单/关机）用。

### §C.3 归属搬迁：消除 `crate::scene::*` 残余边

**目标（硬约束）**：`script.rs`/`script_ops.rs`/`subsystem/{script_host,dialog,entity_service,item_service}.rs` 四文件**零 `crate::scene` 引用**，使 `script` 子树可整体搬出 `game-core`（§E.9）。

现状 4 条边（§1.3）+ 消除方式：

| 边 | 消除动作 | 兼容性 |
|---|---|---|
| `ScriptOp` 住在 `scene`（`crates/game-core/src/scene.rs:55-56`/`:61`） | 把 `script_ops.rs` **整体**移到 `crates/game-core/src/model/command.rs`（新模块 `model/`，纯数据 + `parse_op` + `cursor()`）。`scene.rs` 改为 `pub use crate::model::command::ScriptOp;` | 类型同一 ⇒ 全仓 `use game_core::scene::ScriptOp` 不变（`crates/game-solver/src/script_abs.rs:573` 等无需改） |
| `subsystem::script_host.rs:31` `pub use crate::scene::ScriptOp;` | 改 `pub use crate::model::command::ScriptOp;` | 同上 |
| `subsystem::dialog.rs:8` `use crate::scene::DialogState;` | `DialogState`/`DialogData` 移到 `crates/game-core/src/model/dialog.rs`；`scene.rs` 里 `pub use crate::model::dialog::{DialogState, DialogData};` | 同上；`crates/game-core/src/subsystem/mod.rs:267-270` 测试块同改 |
| `subsystem/mod.rs:151` 生产再导出 | 同上 | 同上 |

**为什么是"移动 + 再导出"而不是"加一层 trait 抽象"**：`ScriptOp`/`DialogState` 是**数据**，不是行为；用 trait 包数据只会把 23 个变体变成 23 个 trait 方法，diff 巨大且丢失 `match` 穷尽性（`crates/game-core/src/scene.rs:715-742` 靠穷尽 `match` 保 23 臂不漏）。移动 + `pub use` 是**零语义变化**的最小动作（可回滚性最好，见 §E 的 S2）。

搬迁后的依赖方向（无环）：

```
model/（ScriptOp, DialogState, DialogData…）
    ↑
subsystem/（7 个 trait 面：script_host / dialog / entity_service / item_service / …）
    ↑
script.rs（DialogueTable, ScriptState, ScriptEngine, NoopHost）
    ↑
scene/（GameScene + impl 上述 trait）        【实现方向：scene → trait，不反向】
```

### §C.4 为什么状态增量必须两套 handler 完全相同

不是"测试对齐"，而是**求解器正确性的前提**（这是 F2 支配健全性的输入条件）：

1. 求解器用 `decide_gate` 的返回值决定"这个格子的 type83 实体能不能进、进了以后世界变成什么"。如果 headless handler 的 delta 与生产不同，求解器就在**一个游戏永远不会进入的世界**里继续搜索 ⇒ `gates` 里那条 `GATE_MEMO` 回灌（`crates/game-solver/src/solve_local.rs:2010-2067` 的 player/flags/pos/ents 回灌）会把**假状态**写进搜索状态，支配关系随即不成立（F2 的"健全性"面）。
2. 反过来说，若 delta 只有一份实现，**"两侧相同"就是构造保证**，测试只需证明"数据宿适配正确"（`ScriptWorld for GameScene` 的 11 个字段转发是否忠实），而不是证明 23 条命令的语义——**测试面从 23×N 缩到 11**。这是本方案可验证性的核心账。
3. 形式化表述（要写进 S4 的验收）：`∀c: Command, s: State. apply(c, SceneHost(s)).world == apply(c, PlainHost(s)).world`，其中 `apply` 是**同一个函数**，故等式退化为 `SceneHost(s).world == PlainHost(s).world`（适配器等价），可用逐字段断言覆盖。

---

## §D 求解器侧（headless handler）

### §D.1 handler 只实现哪些 trait、表现如何空

| trait | headless 实现 | 说明 |
|---|---|---|
| `ScriptWorld` | **完整实现**（唯一实现 `CoreWorld<H>`，H = `PlainState`） | 不可空 |
| `EntityService` | 完整（`EntityTable` 已在 `game_core::entity`，`BHost` 已这么干：`crates/game-solver/src/script_abs.rs:241-262`） | 复用现有 |
| `FloorWorld` | **部分**：`save_floor`/`load_floor_entities` 用"层资产快照"（求解器已持有 `MapLevel`）；`init_cell_grid` 需实现（R-1 裁定后决定是否 no-op） | 不得 no-op 掉 `save_floor`——`ADD`/`LAY` 的世界态就靠它 |
| `ItemService` | 完整（`ItemResolver`+`effect_registry` 已有，`crates/game-core/src/subsystem/item_service.rs:35-37` 现为 stub，须改成真实现或复用） | 注意 `give_gold` 加像素锚后 headless 传 `(0,0)` 即可（锚只影响 popup） |
| `Presentation` | **全 `Noop`**（`impl Presentation for Noop`，8 方法空体） | 唯一允许全空的一层 |

关键约束：**`Noop` 不得吃掉返回值**。`SEE` 的 `findPath` 必须在 `ScriptWorld`/专用方法里做（不是 `Presentation`），否则 `pathArrowsVisible` 就跟着丢了（§A.2.1）。现成反例：`crates/game-solver/src/script_abs.rs:346` 的 `BHost::find_path` **恒返 false** ⇒ 槽 41/46/47/48/50 的 `flag=1 SEE` 在 headless 侧永不置 `pathArrowsVisible`，而 `Obs` 又没这个字段（`:73`）⇒ 差异双重不可见。**修 PE 时必须同时修这两处**。

### §D.2 先决条件：`try_run_scene` 必须拆成"判定 + 提交"

A 级证据：`crates/game-core/src/scene.rs:927` `pub fn try_run_scene(&mut self, idx: i32, from_walk: bool) -> bool` 在**谓词内部**写旗标：

| 位置 | 副作用 |
|---|---|
| `:935` | case 0 过门 ⇒ `script_line_flags[19] = true` |
| `:949` | case 7 过门 ⇒ `script_line_flags[32] = true` |
| `:995` | case 18 **不过门**也 `script_line_flags[18] = true` |

⇒ 一个"纯函数出口"若直接调它，纯性立刻被击穿（返回值依赖内部写，且写跨调用累积）。拆法：

```rust
pub struct GateDecision { pub open: bool, pub pre_writes: FlagWrites }   // 纯值
pub fn decide_gate(&self, idx: i32, from_walk: bool) -> GateDecision;   // 只读 self
pub fn commit_gate(&mut self, d: &GateDecision);                        // 唯一写点
```

拆完后 `decide_gate` 仍是 `&self`（`scene.rs` 的判定读 `flags`/`grid`/`hp`/`difficulty`/`from_walk`，皆为读），可**整段复制**成 headless 版本或用同一实现参数化。

### §D.3 纯函数出口（签名草案 + 最小状态集）

```rust
/// 门 + 脚本行的完整判定（不建 GameScene、不驱动 tick）。
/// 返回 None = 需要真实 tick 机（W 类），调用方可回退到今天的 build_scene 路径。
pub fn run_gate_script(
    st: &mut ScriptState,          // 见 §D.4
    world: &mut PlainWorld,        // ScriptWorld+EntityService+FloorWorld+ItemService
    table: &DialogueTable,
    q: GateQuery,
) -> Option<GateOutcome>;

pub struct GateQuery { pub param: i32, pub from_walk: bool, pub floor: i32 }

pub struct GateOutcome {
    pub gate: GateDecision,        // open + pre_writes（原样回写）
    pub world_delta_lines: usize,  // 跑了多少条（可观测）
    pub terminated: Option<Terminal>,  // END / LAY 换层 / GUT 递归终点
    pub ticks_needed: bool,        // true ⇒ 调用方必须回退到 tickdrive
}
```

**最小状态集合（进 `ScriptState` 的字段，逐条有 §A 依据）**：

| 组 | 字段 | 依据 |
|---|---|---|
| 游标 | `current_script_index`, `script_cursor`, `script_line_flags[68]` | 全部命令 |
| 相位 | `dialog_phase`, `script_delay`, `script_walk_armed`, `walk_phase` | `TAK`/`CES`/`MOV`/`SEE` |
| 对话 | `dialog_page`, `dialog_page_end`, `dialog_target`, `path_arrows_visible` | `TAK`/`SEE`/`MOV` + `crates/game-core/src/scene.rs:434` |
| 玩家 | `hp`, `atk`, `def`, `gold`, `keys[3]`, `item_stack_size`, `alchemy`, `facing`, `player_px`, `player_cell`, `inventory_entry_count` | `ROS_*`/`RES`/`GIN`/`SWD` |
| 楼层 | `current_floor`, `pending_floor`, `transitioning`, `min_floor`, `max_floor`, `difficulty_index`, `floor_assets`（当前层） | `LAY`/`ADD`/`GUT`/`RES` |
| 实体 | `entities[]`（`type/px/py/removed/visible/state/anim_row/param`）+ `grid` | `DES`/`SWD`/`CES`/`MOV`/`ADD` |
| 装备 | `equipped_weapon/armor`, `equip_types[12]`, `equip_bonuses[12]` | `ROS_2` |
| 模式 | `game_mode` | `END` |

**不进最小状态集**（有 A 级"死字段"证据）：`dialogPageStart`、`seeModeFlag`（§A.2.1 第 4 条）；`typewriter_pos`/`overlay_*`/相机三元组（纯表现，`Noop`）。

**收益的诚实表述**：`run_gate_script` 只对 §A.3 的 **48/68 槽**成立（`ticks_needed == false`）；其余 20 槽返回 `None`/`ticks_needed=true`，回退 `build_scene`+`tickdrive`。 【复核更正 · 见 §A.3′】比例改为 **11/68**（`9,11,13,18,25,37,38,39,58,59,60`），其余 **57** 槽回退。并且 `ticks_needed == false` **不等于可以跳过执行**：11 槽中 6 槽（`9,11,13,18,25,38`）有真实世界态写，必须照常 apply delta（只有 A 档 `37,39,58,59,60` 能整槽跳过）。**不做"零 tick 全覆盖"的承诺**——那需要先证明"所有相位最终收敛且不改 flags"，本文没有该证明（见 §F-3）。

### §D.4 与 `script_abs.rs` A/B 对拍的关系：扩成 2×2

现状是 1×2（1 个生产执行器 × 1 个 engine + 自定义宿主）。目标矩阵：

|  | **GameScene 宿主** | **PlainWorld 宿主** |
|---|---|---|
| **GameScene 执行器**（现状，S5 后删除） | 单元：`crates/game-core/src/scene_tests/*` | （不存在，删执行器后无此格） |
| **ScriptEngine 执行器**（目标唯一执行器） | **(E×S)** 生产等价：`Snapshot` 灌 `GameScene` 的 trait 实现，逐 tick 对拍 | **(E×P)** 求解器等价：`PlainWorld` 纯函数 |

落地为三张测试（全部放在 `crates/game-solver/tests/` 或 `crates/game-core/src/scene_tests/`）：

1. **T-parity-1（E×S vs 现场 `GameScene::execute_script_instruction`）**：S5 之前用，逐槽逐 tick 全等；**这是删掉内建执行器的放行条件**。
2. **T-parity-2（E×S vs E×P）**：`Obs` 扩 `dialog_phase` + `ticks` + `path_arrows_visible`，**必须逐 tick 全等**（不是终态），否则 PE-01 类缺陷再次隐身。
3. **T-parity-3（数据宿适配器等价）**：`SceneHost(s) == PlainHost(s)` 的 11 字段断言（§C.4 第 3 条的退化形式）。

同时必须删掉/替换的三处 harness 缺陷：

- `crates/game-solver/src/script_abs.rs:346` `find_path` 恒 `false` ⇒ 改成真寻路（求解器已有可复用的 `walk`/`path` 模块）；
- `crates/game-solver/src/script_abs.rs:193` side_a 用 `from_fixtures` 语料、side_b 用 `dialogue_table()` ⇒ **统一到单一 table**（推荐：把 `0x1E` 保留与否做成 table 构造的显式参数，两侧同参）；
- `crates/game-solver/src/script_abs.rs:73` `Obs` 缺相位/ticks ⇒ 补齐。

---

## §E 实施方案（S1..S8）

> 通用门禁口径：每片至少 `python3 gates/cli.py check`（总闸：original/assets/reference-seed/knowledge/docs-links/naming/rust-tests/ratchet/table-content-hash/table-evidence/state-machine-anchors/asset-manifest/state-baseline）；触及 `crates/game-core` 逻辑的片另加 `python3 gates/cli.py trace`；触及渲染另加 `visual`；触及门禁/基线另加 `ratchet`。**74 金样**（`data/status/baselines/state-dump-golden.json` 的 `scenarios` 实测 `len == 74`）**不放宽、不重冻**，任何差异必须定位到层（AGENTS §4）。

### S1 文档入库（本提案）+ spec 原地更正

| 项 | 内容 |
|---|---|
| 范围 | 本文件；`docs/spec/p3-script-interpreter.md` §3.1/R4 的 2026-11-16 更正段（§B.6 措辞照抄） |
| 文件 | 本文 + `docs/spec/p3-script-interpreter.md` |
| 改动量 | ~1 文件新增、1 文件 +6 行 |
| 风险 | 低。仅文档 |
| 门禁 | `python3 gates/cli.py check`（含 docs-links） |
| 回滚 | 独立 revert |
| 生产行为 | **零变化** |

### S2 模型迁出（`ScriptOp`/`DialogState`/`DialogData` → `model/`）

| 项 | 内容 |
|---|---|
| 范围 | 新建 `crates/game-core/src/model/{mod,command,dialog}.rs`；`script_ops.rs` 整体移入 `model/command.rs`；`scene.rs:55-56`/`:61`、`scene.rs` 内 `DialogState`/`DialogData` 定义处改为 `pub use`；`subsystem/{script_host.rs:31, dialog.rs:8, mod.rs:151,267-270}` 改路径 |
| 文件 | 5–6 文件 |
| 改动量 | ~80 行（`git mv` + `use`/`pub use`） |
| 风险 | 中低：`DialogState` 有大量字段级 `impl`（`scene.rs` 内），`pub use` 后 `impl` 必须留在**类型定义所在模块**否则孤儿规则报错 ⇒ `impl` 块须随类型一起搬（这是本片唯一容易出血的点） |
| 门禁 | `check` + `cargo test -p game-core -p game-solver`（等价性由编译期 + 现有测试保证，无新逻辑） |
| 回滚 | 独立 revert（纯移动） |
| 生产行为 | **零变化**（A 级：类型同一，`pub use` 不产生新类型） |

### S3 补 trait + `GameScene` 承载（不改执行路径）

| 项 | 内容 |
|---|---|
| 范围 | 新增 `ScriptWorld`/`FloorWorld`/`Presentation`；扩 `EntityService`（3 方法）/`ItemService`（加锚参）；`impl ScriptWorld for GameScene` 等**只做转发**；`op_*` **暂不改**（避免双向真相） |
| 文件 | 4 trait 文件 + `impl` 放 `crates/game-core/src/scene_script_host.rs`（新文件，承载 4 个 `impl`） |
| 改动量 | ~350 行（绝大多数是转发体） |
| 风险 | 中：转发体写错 = 静默错。**缓解**：每方法加一条"同值双写断言"测试（旧路径写后读、新路径写后读，二者相等） |
| 门禁 | `check` + `trace`（应零漂移） |
| 回滚 | 独立 revert |
| 生产行为 | **零变化**（新代码无人调用） |

### S4a engine 对齐 Java（PE-01/02/03/04/05/08/09/10/13 全在 engine 侧）

| 项 | 内容 |
|---|---|
| 范围 | 修 `crates/game-core/src/script.rs`：删尾部门（PE-01）、删 `exit_application` 调用并补 `layout_ending`（PE-02）、`LAY` 改 `save_floor`（PE-03）、`ROS_4` 加 `i8` 截断（PE-04）、`CES` 走 `step_delay_for`（PE-05）、`SEE`/`MOV` 写 `dialog_target`+`ease_to_cell`（PE-08/09）、`GUT` 走 `mark_line_flag`（PE-10）、`load_level_script` 的 `idx==31` 标注待裁（PE-11 不动） |
| 文件 | `crates/game-core/src/script.rs` 单文件（+ 必要的 trait 实现处） |
| 改动量 | ~150 行 |
| 风险 | **中高**：这是唯一"改语义"的 engine 侧批量。但它**不在生产路径上**（生产仍走 GameScene）⇒ 生产零风险；求解器侧影响 `GATE_MEMO` 的**键值可能变化**（`dialog_target` 等未进键，但 `flags` 进了）⇒ 需重跑 `trace`+求解器回归 |
| 门禁 | `check` + `trace` + `cargo test -p game-solver`（`script_abs` 应全绿；若变红，说明它原来测的是旧语义 ⇒ **按 §B.4 先修 harness 再判**） |
| 回滚 | 独立 revert |
| 生产行为 | **零变化**（生产不跑 engine） |

### S4b Java ↔ GameScene 差异裁定（PE-06/PE-07 + R-1/R-2）

| 项 | 内容 |
|---|---|
| 范围 | 按 §B.5 用 trace 裁定 `ADD` 的 `initCellGrid()`（R-1）与 `RES` 的 `journal.clear()`（R-2）；裁定为"GameScene 错"的，改 GameScene；裁定为"Java 无此语义"的，登记 `docs/bug-ledger.md`（AGENTS §6，Stage A 不修） |
| 文件 | `crates/game-core/src/script_ops.rs`（`op_add`/`op_res`）+ `docs/bug-ledger.md` |
| 改动量 | ~30 行 + 台账条目 |
| 风险 | **高**：这一片**会改变生产行为**（若 R-1 判 GameScene 缺 `initCellGrid`）。必须单独复核 |
| 门禁 | `check` + `trace`（**逐 tick 差分**）+ `visual`（若 `initCellGrid` 影响绘制）+ 74 金样不放宽 |
| 回滚 | 独立 revert（但若台账已写，回滚须同步） |
| 生产行为 | **⚠️ 唯一改变生产行为的两种可能之一** |

### S5 生产切 `ScriptEngine` + 删 `scene_script.rs` 内建执行器

| 项 | 内容 |
|---|---|
| 范围 | `crates/game-core/src/scene_dialog.rs:117` 改调 `eng.execute_script_instruction(&mut scene_host)`；删 `crates/game-core/src/scene_script.rs:14-36`（保留 `parse_script_int` 与 `advance_script_walk`）；`op_*` 23 个函数体搬进 `impl ScriptWorld + EntityService + … for GameScene`（与 S3 的转发体合并）；`crates/game-core/src/script_ops.rs:553-589` 的 `execute_op` 变成 engine 的薄封装或删除 |
| 文件 | `scene_script.rs`（-30 行）、`script_ops.rs`（-590 行，函数体搬走）、`scene_dialog.rs`（改 1 行）、`scene.rs`（删 `mod` 声明与 `impl ScriptOp` 位置调整）、`scene_solver_api.rs:59`/`:97`（门面改转发 engine） |
| 改动量 | **最大一片**：~650 行移动 |
| 风险 | **高**：`execute_script_instruction` 有 **~~9~~ 31 处调用表达式**（【复核更正】见 §S5′；v1 的 9、表格相加 28、复核的 22 都不可复现），任何一处漏改都会双执行器并存。真实语义面只有 3 类：生产入口 1、`op_*` 递归 2、engine 自身 3 |
| 门禁 | `check` + `trace`（**L2b 逐 tick**）+ 74 金样 + `cargo test -p game-core -p game-solver` |
| 回滚 | 需 `git revert` 整片（不建议拆） |
| 生产行为 | **零变化**（前提：S4a 已让 engine 逐 tick 等价，T-parity-1 全绿） 【复核更正 · 见 §S5′】**「零变化」不成立，有 RES 反例**：生产当前走 `GameScene`，其 `op_res`（`crates/game-core/src/script_ops.rs:101-116`）**不写** `inventory_entry_count`；engine 的 `RES` 写（`crates/game-core/src/script.rs:459`）= Java `a.java:8489`。⇒ S5 切 engine 后，槽 0 的 `RES_0` 会**新增一条世界态写**（`inventory_entry_count: 0 → 0` 之外的情形会真的变值，例如从存档读回非零的 run）。**前提修正**：S5 的零变化以「PE-06 已在 `GameScene` 侧对齐（补写 `inventory_entry_count=0`）」+「语料行保留 `0x1E`（尾门不触发）」两条同时成立为条件；否则 S5 是"**有意的行为修正**"，必须单列并跑 `trace` + 存档往返差分 |

#### §S5′ 调用点清单（2026-11-16 第二轮重算，「A 级：现场 `grep -rn`」）

复现命令：

```bash
grep -rn "execute_script_instruction" crates/game-core/src crates/game-solver/src --include=*.rs
```

⇒ **31 个调用表达式** + 2 个函数定义 + 1 个函数指针引用（另有 5 处注释行）。逐面：

| 语义面 | 处数 | 位置 | 改法 |
|---|---|---|---|
| 生产唯一入口 | **1** | `crates/game-core/src/scene_dialog.rs:117` | 改调 engine（S5 唯一的功能改动） |
| `GameScene` 执行器内部尾递归 | **2** | `crates/game-core/src/script_ops.rs:197`（`MOV` 未命中）、`:588`（`GIN`） | 随 `scene_script.rs` 删除而消失 |
| `ScriptEngine` 内部 | **3** | `crates/game-core/src/script.rs:254`（`step`）、`:323`（`MOV` 未命中）、`:422`（`GIN`） | 保留（engine 自身的忠实递归，`a.java:8308`/`:8443`） |
| 求解器驱动 | **2** | `crates/game-solver/src/script_abs.rs:198`（`side_a`）、`:503`（`side_b`） | S7 重写 |
| 测试门面 | **4** | `crates/game-core/src/scene_solver_api.rs:60`、`:149`、`:166`、`:544` | 门面改转发 engine；**`:97` 的签名锁必须改形**（`let _: fn(&mut GameScene)` 在删除执行器后锁的是死码） |
| 单元测试直调 | **19** | `crates/game-core/src/scene_tests/slot_equip_scene_tests.rs`（15）+ `crates/game-core/src/scene_tests/entity_route_tests.rs:10`/`:73`/`:255`/`:267`（4） | 已是门面调用 ⇒ 只改门面 |
| **合计调用表达式** | **31** | | |

> 【2026-11-16 复核更正】v1 的「9 处」漏掉 19 处单元测试直调与 3 处 engine 内部递归/`step`；v1 表格相加为 28（`15+5+3+2+2+1`），其中 `entity_route_tests` 实为 4 处代码（+1 处注释）、`scene_solver_api` 实为 4 处代码（另 1 处函数指针、1 处定义、2 处注释）；复核报告的「22 处」用上述命令不可复现。**结论**：迁移风险不在"31 处都要改"，而在 3 个语义面（生产入口 / `op_*` 递归 / engine 自身）——其余 23 处是测试与门面。

### S6 求解器纯函数出口（§D.2/§D.3）

| 项 | 内容 |
|---|---|
| 范围 | 拆 `try_run_scene` → `decide_gate`/`commit_gate`；新增 `run_gate_script` + `PlainWorld`；`gate_arrive`（`crates/game-solver/src/solve_local.rs:2012`）改为"先纯函数、`None` 才 `build_scene`" |
| 文件 | `crates/game-core/src/scene.rs`（拆谓词）、`crates/game-core/src/script_gate.rs`（新）、`crates/game-solver/src/solve_local.rs`、`crates/game-solver/src/plain_world.rs`（新） |
| 改动量 | ~400 行新增 |
| 风险 | 中高：`decide_gate` 拆错会改变门判定（**不改**判定语义，只搬写点 ⇒ 用"拆分前后同一输入返回同值"的表格测试锁） |
| 门禁 | `check` + `trace`（求解器回归：`cargo test -p game-solver`）+ 求解器耗时对比（floor51 单层 `46s` 的分段定位注释已在 `solve_local.rs`，可作基准） |
| 回滚 | 独立 revert（纯函数未被调用前无影响） |
| 生产行为 | **零变化**（纯函数只服务于求解器） |

### S7 2×2 对拍矩阵（§D.4）

| 项 | 内容 |
|---|---|
| 范围 | 扩 `Obs`（+`dialog_phase`/`ticks`/`path_arrows_visible`）；统一语料口径；`BHost::find_path` 改真寻路；新增 T-parity-2/3 |
| 文件 | `crates/game-solver/src/script_abs.rs`（改 +250 行）、`crates/game-solver/tests/*` |
| 改动量 | ~300 行 |
| 风险 | 中：`Obs` 扩容会让**现有对拍立刻变红**（这正是目的：证明 PE-01 存在）。红 → 说明 S4a 没修净，回头修 |
| 门禁 | `check` + `ratchet`（测试计数只增）+ `cargo test -p game-solver` |
| 回滚 | 独立 revert |
| 生产行为 | **零变化** |

### S8 `game-rules` crate 拆分（前置关系说明）

| 项 | 内容 |
|---|---|
| 范围 | `git mv` `model/`+`subsystem/` 的 trait 面+`script.rs`+`script_gate.rs` 到新 crate `crates/game-rules/`；`game-core` 依赖它并提供 `impl` |
| 前置条件 | **必要条件 1** = S2（零 `crate::scene` 边）✅ 本方案达成；**必要条件 2** = `ScriptCtx` 的读槽类型（`player`/`entities`/`floor`）不得是 `scene` 私有类型 ⇒ 需 `Entity`/`ItemDef`/`FloorAssets` 一并进 `game-rules`（**本方案不含**）；**必要条件 3** = `resource_cfg::floor_category`（`crates/game-core/src/resource_cfg.rs:319`）被 `step_delay_for` 依赖 ⇒ 需把"层覆写类目表"抽象成 trait 或数据（**本方案只把它藏在 trait 后面，未迁出**） |
| 判定 | 本改造**让 `script` 子树具备可搬走的形状**，但 `game-rules` 拆分仍缺数据模型迁移 ⇒ **先后关系：本方案 S1–S7 先，S8 后**，且 S8 不是 S7 的直接产物 |
| 生产行为 | 零变化（纯搬家） |

**依赖顺序（硬）**：`S1 → S2 → S3 → S4a → S4b → S5 → S6 → S7 → S8`。其中 S2/S3 可并行（不同文件），S6/S7 可在 S5 前后并行，S4b 必须 **S4a 之后、S5 之前**（否则 S5 切换时不知道 engine 该对谁）。

---

## §F 风险与不适用条件

### §F.1 本方案**不能**解决的问题

1. **求解器搜索覆盖**：`decide_gate` 纯函数化只让"单个门的语义"可信，不改变"搜到/搜不到"。`tickdrive` 的既有近似（脚本行走期间不触发中途 type83 门，`crates/game-solver/src/solve_local.rs:2360`）**本方案不修**，它不是管线问题。
2. **支配健全性 F2**：`GATE_MEMO` 键 `(floor,param,player,flags,ents)` **不含 `pending_floor`/`dialog_target`/`path_arrows_visible`**。本方案不改键 ⇒ 若某脚本的后续行为依赖这些字段，memo 仍可能命中错误记忆。**这是独立缺陷，须单独立项**（本方案只把它标出来）。
3. **相位收敛性未证**：`§D.3` 的"48/68 槽可免 tick"（【复核更正】应为 **11/68**，且分 A/B 两档，见 §A.3′）是**静态可达性判据**，不是"相位一定收敛"的证明。若某个 P 类行进入 `phase=3`（例如行内 `MOV` 的 `findPath` 成功），静态判据会把它划到 W 类——判据在 `MOV` 上是**运行时**的（看 `find_path` 结果），本文没有对"所有参数组合"的穷举。
4. **时序保真的裁判**：本方案统一的是**语义**，时序的裁判仍然是 L2b trace。`run_gate_script` 跳过相位会改变 tick 序列 ⇒ **它只能用于求解器的状态判定，永远不能当"时序等价"的证据**。
5. **跨层实体快照**：`probe_cross_floor_snapshot` 未移植（`crates/game-core/src/scene.rs` 的 case 7/32 显式挡路），本方案不补 ⇒ 门 7/32 仍不可判定。
6. **表现保真**：相机 ease 的 `(vw-32>>1)-px` 居中偏移、打字机 `layout_wrapped_text` 的 129 列回绕，本方案只保证"这些调用被正确分类为表现"，不保证**逐像素**一致（那是 `visual` 门禁的活）。
7. **`MVS` 死码的语义**：DC-2 只裁定不可达，不裁定"这段代码到底在干什么"。本方案保留它、把它归 `Class::C`，**没有**解开它的语义。

### §F.2 "看起来能用、实际不行"的诱惑（防过度设计）

| 诱惑 | 为什么不行 |
|---|---|
| 把 `Outcome{state_delta}` 做成真值类型，两 handler 各自算 delta 再合并 | §C.1：13 条命令写数组元素、3 个控制态不可值化；合并语义会成为新的不可测面 |
| 把 `Presentation` 也做成纯函数以便"求解器复用相机" | 相机/打字机是**时序**，做成纯函数会诱导求解器依赖表现层时序 ⇒ 直接违反 F2 |
| 用 `Noop` 把 `SEE` 的 `findPath` 一起空掉（"反正没画面"） | §A.2.1：`pathArrowsVisible` 驱动寻路消费 ⇒ 空掉就是状态错。现成反例 `crates/game-solver/src/script_abs.rs:346` |
| 以为"删掉 GameScene 执行器就提速" | memo 全命中时**零 build**（`solve_local.rs` 注释），收益只在 miss ∧ 纯 P 类槽；先量化再承诺 |
| 为"统一"而删 `Mvs` 臂 / 删 `RosNoop` | AGENTS §1 行为只增不减；DC-2 只封可达性。删了会让合法行落 `None` ⇒ 停摆语义改变 |
| 用 `#[ignore]`/放宽阈值让 PE 红测试变绿 | 门禁红线；且 §B.4 已证明"看不清"是观察面缺字段，正确做法是扩 `Obs` |
| 认为 `0x1E` 尾字节是夹具脏数据、顺手 trim 掉 | 它会**改变行尾判据**（`cursor+3>len` vs `cursor>=len`）⇒ 顺手一改就把 PE-01 永久掩埋（§B.4 第 2/3 条） |
| 用 `trait` 包住 `ScriptOp`（23 方法） | 丧失 `match` 穷尽性（`crates/game-core/src/scene.rs:715-742` 的 23 臂全靠穷尽 `match` 保证不漏） |

---

## §G 假设区（C 级，不得作为后续依据）

| 编号 | 假设 | 缺什么证据 |
|---|---|---|
| H-1 | `GLV_n` 的 `n` 可能表示"某类全局可见性/难度"（语料仅槽 37 `GLV_1`，Java 只 `parse(' ')` 不取用 ⇒ 无副作用） | 若 `n` 真无副作用，则应像 `SMS` 一样记为"纯游标"；本文按 A 级（Java 实现）记为无副作用，但**语义意图**仍是 C 级 |
| H-2 | `SMS_n` 的 `n` 可能是"短信/提示文本 id" | 同上：实现无副作用是 A 级，意图是 C 级 |
| H-3 | `RosNoop` 在真实存档里可能出现（语料 0 命中） | 需要"原版存档全量扫描"或 `parseScriptInt` 在越界时的行为证据 |
| H-4 | `RES` 时 `journal.clear()` 是 Rust 侧刻意新增（存档日志自洽），非 Java 语义 | 需存档往返 trace（R-2） |
| H-5 | `LED`/`AMR`/`PEQ` 等 Stage B 菜单命令不在 `ScriptOp` 23 变体内 ⇒ 与管线无关 | 本文只覆盖 23 变体，未穷举 Stage B 需求 |
| H-6 | `GUT` 目标层若含 `MOV`，其"W 类"传播不影响本方案的静态计数（20 槽已含 `GUT` 图的 62→63 环） | 需对 `GUT` 图做闭包计算（本文只用"含 GUT 的槽"作近似，未算闭包新增） |

---

## §H 对抗性复核清单（交给人/另一 agent 去攻）

按"若这三条错，方案就整体作废"排序：

1. **PE-01 的 javap 判读**：`2386-2400` 是否真的只是 `if (cursor < len) return;` 而非"else 块入口"？若我读错，则 engine 的尾部门可能反而是对的、GameScene 缺一手。**建议复核**：`javap -p -c` 重导出，逐偏移核对 `2397: if_icmpge 2431` 的跳转目标是方法尾（`2431: return`）而不是某个分支体；再用"槽 6 一行脚本"的 `a.java` 单步（若可用）确认 `TAK` 后 `dialogPhase` 未被改写。
2. **"生产行为零变化"主张**：S4a 全部改 engine ⇒ 生产（GameScene）不变。但 S4b 若判 `ADD` 缺 `initCellGrid`（R-1），则生产行为**会变**。本文对 R-1 只有 B 级证据 ⇒ 若复核发现 `initCellGrid` 与 `m_122(currentFloor)` 写面完全重叠，则 R-1 可降为"零变化"；反之必须在 S4b 显式标注"改变生产行为"。
3. **"48/68 槽可免 tick"**：本文的 P/W 划分只依据 §A.2.2（相位阻塞语义），**未证明**"P 类行执行过程中不会进入 `phase=2/3`"。`MOV` 已按"运行时判 `find_path`"处理，但 `CES`（`script_delay` 会写、`phase` 不写）与 `GIN`（尾递归）是否可能间接触发相位？**建议复核**：对 48 个 P 类槽逐行走一遍 `parse_op` 序列，断言"无 `Mov`/`Lay`/`End`/`Gut` 且无尾递归以外的相位写"。

> 【2026-11-16 复核更正 · 见 §A.1′/§A.3′】**这条自我怀疑是对的，而且比作者设想的更严重。** ① 已逐槽走完 296 条指令：`CES` 与 `GIN` 都不写相位（`a.java:8395-8429`/`:8430-8443`），真正的漏洞不是"P 类行会不会进相位 3"，而是**"无相位跃迁"本身不能推出"无世界态写"**——`TAK`/`SEE`/`CES`/`DES`/`ADD`/`SWD`/`ROS_*`/`GIN`/`RES`/`GUT`/`LAY` 都在无相位跃迁的情况下改世界态；② 因此 P 类实际只有 **3 个变体**（`Glv`/`Sms`/`RosNoop`）、真·零写面槽 **5** 个，而不是 48 个；③ 第 1 条（PE-01 的 javap 判读）已被复核确认：`2397: if_icmpge 2431` 的落点就是方法尾 no-op，但**该门在 11 字节行口径下永不触发** ⇒ P1 而非 P0（§B.2′）。

---

## 附：本文所用一次性复核命令（可复现）

```bash
# 1) 23 变体与解析入口
sed -n '1,60p' crates/game-core/src/script_ops.rs
# 2) 两执行器门结构
sed -n '14,40p' crates/game-core/src/scene_script.rs
sed -n '270,300p' crates/game-core/src/script.rs
sed -n '505,522p' crates/game-core/src/script.rs
# 3) javap 裁决（需 original/ 的 class 投影）
javap -p -c analysis/rename-pipeline/renamed/a.class > /tmp/a.javap
grep -n 'if_icmpgt\|if_icmpge' /tmp/a.javap   # 入口门 2401 / 尾部 2397→2431
# 4) 语料统计
python3 - <<'PY'
from collections import Counter
t=open('crates/game-core/tests/fixtures/level_scripts.txt',encoding='utf-8').read()
slots=t.split('\u001e'); c=Counter()
for i,s in enumerate(slots):
    for tok in s.split(): c[tok.split('_')[0]]+=1
print(len(slots), sum(c.values()), c)
PY
# 5) 74 金样条数
python3 -c "import json;print(len(json.load(open('data/status/baselines/state-dump-golden.json'))['scenarios']))"
```
