# game-core 求解器直调门面提案（方案 B：批1 直调，拒绝自建 23 枚举转移表）

> 2026-11-09。**用户 2026-11-09 裁定方案 B，批1 前过对抗复核**：求解器批1 走**直调 game-core**，拒绝自建 23 枚举转移表的重复逻辑。本件即求解器实施计划 §3.1 后门条款所称「另立提案」（[solver-implementation-plan.md:224](../plans/2026-11-09-solver-implementation-plan.md) 实测原话：「若对拍暴露必须直调的需求，另立提案改 game-core 暴露 `pub` 门面，**不混入本计划批次**。」），同步消解计划风险 R7（同文件 :414「**ScriptOp parse/execute 私有**（`script_ops.rs:401,553`）…若必须直调，另立提案改 game-core（不混入本批）」）。
> **只立文档：本提案不改任何代码、不改计划**；实施切片（§4）须待本件过对抗复核通过后另行派工。**先经对抗复核，通过后派工。**
> 全部行号 = 2026-11-09 在当前 HEAD `fac2c65`（分支 modern）上 sed/grep 实测；`git diff df4e3ea..HEAD -- crates/game-core/src/script_ops.rs crates/game-core/src/scene_script.rs crates/game-core/src/scene.rs crates/game-core/src/subsystem/script_host.rs` 为**空**——任务书所记 df4e3ea 起这四个文件未变，两侧行号一致（A 级验证，见 §7 表）。

## 0. 裁定与边界

- **裁定**（2026-11-09）：批1 不走计划 :295 原案「`script_abs.rs`：把 23 个 `ScriptOp` 变体映射为 `fn(&mut SolverState, &SolverInput) -> ()` 转移函数表（**自建，不依赖 crate 私有的 parse_op/execute_op**）」，改走**直调 game-core**（方案 B）。
- **本提案解决什么**：给出直调所需的最小暴露面、三问预检、验证闸、执行切片，并声明对拍闸保留。
- **本提案不解决什么**：批1 实现本身（`script_abs.rs` 尚未建，`crates/game-solver/src/` 现只有 `bin/solver.rs`、`ingest.rs`、`lib.rs`、`state.rs`）；不改求解器计划文档；不改任何门禁。
- **红线不变**（AGENTS.md:36）：game-core「**无 I/O、确定性**：只用虚拟时钟与 GameRng；禁止引用平台 crate」——暴露可见性不触碰该红线的任何一条；无头先例 = game-oracle 全无头跑（金样 74 场景）。

## 1. 暴露面最小集（问题 1）

### 1.1 结论：开 3 个转发 fn，零个既有符号改动可见性

门面 = 新文件 `crates/game-core/src/scene_solver_api.rs`，挂 **scene 模块树内**（scene.rs 加一行 `#[path]` 挂载），对外导出 `pub mod solver_api`。3 个转发 fn 全部是**纯委托**（一行转发到既有 `pub(super)` 方法），底层 `script_ops.rs` / `scene_script.rs` **一个字符不改**——纯新增 diff，零行为风险。

| # | 门面转发 fn（拟新增，pub） | 委托到（现状，实测行号） | 现状签名 |
|---|---|---|---|
| 1 | `pub fn parse_op(line: &str, script_cursor0: usize) -> Option<ScriptOp>` | `script_ops.rs:401` | `pub(super) fn parse_op(line: &str, script_cursor0: usize) -> Option<ScriptOp>` |
| 2 | `pub fn execute_op(scene: &mut GameScene, op: ScriptOp)` | `script_ops.rs:553` | `pub(super) fn execute_op(&mut self, op: ScriptOp)`（&mut self = GameScene） |
| 3 | `pub fn execute_script_instruction(scene: &mut GameScene)` | `scene_script.rs:14` | `pub(super) fn execute_script_instruction(&mut self)` |

**为什么是 3 个而不是 2 个**（parse_op + execute_op 不够）：

- `execute_script_instruction` 承载**行级语义三件套**，求解器若只拿 parse+execute 自己拼行循环，就必须复刻这三件＝回到方案 B 要拒绝的重复逻辑：
  1. **行尾收尾依赖私有 fn**：`scene_script.rs:16-24` 行尾守卫 `if script_cursor0 + 3 > line.len() {` 分支内调 `set_dialog_phase(4)`、置 `script_line_flags[cur]`（`cur != 32` 特判在 :20-21）、`lock_camera_on(0)` —— 而 `lock_camera_on` 是**私有**的（`scene.rs:3191` `fn lock_camera_on(&mut self, target: i32)`，无 pub，实测）。外部无法复刻收尾而不改可见性，改它 = 扩暴露面（违反本件最小集）。
  2. **未知 op 静默停摆语义**：`scene_script.rs:29` `let Some(op) = Self::parse_op(&line, script_cursor0) else {` → 停摆 return（原版 a.java:8594-8595）。注意 `parse_op` 本身**无守卫**：`script_ops.rs:402` `let op = &line[script_cursor0..script_cursor0 + 3];` 越界会 panic——守卫只在 `scene_script.rs:16-17`。求解器绕过行驱动直接裸调 `parse_op` 需自带守卫，否则踩 panic 面。
  3. **GIN 同拍尾递归宿主**：`script_ops.rs:587-589` execute_op 末尾 `if matches!(step, …Recurse) { self.execute_script_instruction(); // GIN 同拍尾递归（a.java:8435-8436）`——execute_op 内部已会回调 execute_script_instruction，暴露它使门面语义自洽（转发链不悬空）。
- 反过来，**execute_op 也不能省**：它是 23 臂 match（`script_ops.rs:560-586`）→ `op_*` helper（22 个 `pub(super)`，实测 `grep -c "pub(super) fn op_" = 22`，第 23 变体 RosNoop 无 helper 且 :584 空臂）的总入口，A 类状态变换（gold/实体/hp）全在此（§1.3）。
- **状态读取不需要新符号**（全部已 pub，见 §1.4），**checkpoint 五元组回放**（计划批5 :382「五元组（血/攻/防/三色钥匙/金币，字段源 `combat.rs:51`）」）读的字段已可跨 crate 访问。

### 1.2 为什么门面必须挂 scene 树内（可见性物理约束）

- `script_ops.rs` 经 `scene.rs:55-56` `#[path = "script_ops.rs"] mod script_ops;` 挂载，是 **scene 的私有子模块**——其中 `pub(super)` 的语义是 `pub(in scene)`，**只在 scene 模块树内可见**；`lib.rs` 层（及 `game-solver`）看不到。
- 因此转发 fn 必须声明在 scene 的后代模块里（新文件 + scene.rs:43-44 同风格 `#[path]` 挂载，scene.rs:55-56 旁），不能放 lib.rs 层。`lib.rs:23` `pub mod scene;` 已是 pub，门面路径 = `game_core::scene::solver_api::*`，消费端可达。
- 这也是本方案「**底层不动**」的关键：把 `pub(super)` 直接改成 `pub` 也能达到目的，但那是改既有符号（diff 触及解释器核心行，需解释为何改、改后谁可见）；转发 fn 方案让既有符号可见性**保持原样**，审计面收敛到一个新文件。

### 1.3 ScriptHost trait 面评估：免费，但**不充分**（不能单独作为暴露面）

任务点名要实测的「P2 批5 已有 ScriptHost::execute_op 可能就是免费门面」——实测结论：

- **确实免费且已 pub**：`script_host.rs:48` `pub trait ScriptHost {` / `:49` `fn execute_op(&mut self, ctx: &mut ScriptCtx, op: ScriptOp) -> ScriptStep;`（pub trait 的方法即跨 crate 可调）；`script_host.rs:61` `pub struct ScriptResolver;`；`subsystem/mod.rs:25` `pub mod script_host;`。求解器可直接 `impl ScriptHost` 或直接调 `ScriptResolver`。
- **但只覆盖「脚本域写」**：`ScriptResolver` 实现体（`script_host.rs:71-95`）只做 `*ctx.script_cursor = op.cursor()`（:74，游标推进）、Gut 旗标写、Ces delay 写、Gin → `ScriptStep::Recurse` 信号——**不碰 gold/实体/hp**。
- **A 类状态变换全在 GameScene 侧**：`script_ops.rs:560-586` 的 match 23 臂逐臂调 `self.op_*`（`op_tak:60` `op_add:120` `op_res:101` `op_mov:175` `op_des_all:208` …共 22 个 `pub(super)` helper）。这些 helper 全是 GameScene 方法，`ScriptResolver` 摸不到。
- **且没有注入缝**：`script_ops.rs:556-559` execute_op 内部**硬编码自建** `ScriptResolver`（`let mut r = crate::subsystem::script_host::ScriptResolver; r.execute_op(&mut ctx, op)`），不接受外部 host 注入——求解器即使自己 impl 一个 ScriptHost，GameScene 也永远不会调它。
- **结论**：ScriptHost 面保留原样（已 pub，零改动）；`execute_op` 必须经 GameScene 面转发（门面 #2）。求解器若只需「游标推进 + 脚本域写」可免费直用 ScriptResolver，但批1 转移需求（A 类变换）必须走门面。

### 1.4 明确不改的符号（零改动清单，全部实测已 pub）

| 符号 | 现状锚（实测） | 备注 |
|---|---|---|
| `ScriptOp` 枚举（23 变体） | `script_ops.rs:6` `pub enum ScriptOp`；出口 `scene.rs:60` `pub use script_ops::ScriptOp;`、`script_host.rs:31` `pub use crate::scene::ScriptOp;` | 已跨 crate 可见 |
| `ScriptHost` / `ScriptResolver` / `ScriptCtx` / `ScriptStep` | `script_host.rs:48` / `:61` / `:34`（字段全 pub）/ `:55` | 批5 已落的 trait 面，免费 |
| 状态字段 | `scene.rs:282` `pub struct GameScene`；`:411-419` `pub script_line / script_cursor / script_delay / script_line_flags / current_script_index` | checkpoint 读取源 |
| 五元组/背包 | `combat.rs:51` `pub struct PlayerCombat`（hp:52 / gold:58 / yellow_keys·blue_keys·red_keys / item_types 全 pub） | 批5 :382 五元组源 |
| 实体表 | `entity.rs:57` `pub struct EntityTable` | 实体集合对比源 |
| 装载入口 | `scene_persist.rs:41` `load_floor` / `:269` `load_floor_data` / `:501` `pub fn load_level_script(&mut self, idx: i32)` | 计划 :147 集成路径 |
| 对拍 harness 面 | `script.rs:154` `pub struct ScriptEngine<'d>` / `:162 pub fn new` / `:270 pub fn execute_script_instruction` | 计划 :302 对拍 (b) 侧 |
| 测试构造 | `lib.rs:29-33` `#[doc(hidden)] pub mod scene_test_support`；`scene.rs:4624-4626` | 跨 crate 无头构造先例 |
| 22 个 `op_*` helper | `script_ops.rs:60-316`，全 `pub(super)` | **不开**——经 execute_op 间接可达，逐个开=暴露面爆炸 |

### 1.5 消费端（批1）如何接

- `crates/game-solver/Cargo.toml` deps **已含 game-core**（实测），无需改依赖。
- 批1 的 `script_abs.rs`：`use game_core::scene::solver_api::{parse_op, execute_op, execute_script_instruction};` + 读 §1.4 已 pub 字段。计划 :302 对拍的 (b) 侧（`ScriptEngine` / `load_level_script`）不变。

## 2. 三问预检（AGENTS.md §4 :70-77，动手前三问）

1. **问题在哪一层？**——**可见性层**（Rust 模块可见性：`pub(super)` = `pub(in scene)`，scene 私有子模块的方法跨 crate 不可达），**非行为层**。转发 fn 不修改任何既有行的逻辑、签名、调用点；行为面 diff = 0。不是反编译投影错、不是格式理解错、不是门禁错。
2. **消费者实际读哪份数据？**——`crates/game-solver` 批1 `script_abs.rs` 读：(a) 脚本文本行（运行时真相源 = `crates/game-core/tests/fixtures/level_scripts.txt` 68 槽，经 `scene.rs:252-253` `from_fixtures` 同源语料，计划 :131 实测 3609 字节/68 行）；(b) `GameScene` 状态字段与 `PlayerCombat` 五元组（§1.4 表，全已 pub）；(c) 转移后状态回读做对拍断言。不读 `original/`、不读 `reference/`、不读转换产物。
3. **证据是什么级别？**——A 级：所有行号为 sed/grep 实测输出（引文行号 = 实测行号，防引锚漂移）；可见性结论有 `git diff df4e3ea..HEAD`（四文件空）+ 模块挂载链实测（scene.rs:55-56 私有 mod）双证；门禁现状为 `gates/cli.py status` 实跑输出（PASS=13 FAIL=0 SKIP=1 TOLERATED=0）。无 C 级内容进入结论。

## 3. 风险与闸（问题 3）

### 3.1 「改 pub 不改行为」必须过的闸（合入前提，缺一不许合）

| # | 闸 | 命令 | 判据 | 依据 |
|---|---|---|---|---|
| G1 | 总闸 | `python3 gates/cli.py check` | FAIL=0（13/13；实测 status：`PASS=13 FAIL=0 SKIP=1 TOLERATED=0`，`SKIP=visual-diff` 显式打印） | AGENTS.md §5 :79-91；计划 :191 |
| G2 | 金样 74 零漂移 | `python3 tools/state_baseline.py --check` | `[GREEN] 金样逐字节等价：74 场景 sha 全同` | 计划 :184/:383 |
| G3 | 差分 trace | `python3 gates/cli.py trace` | L1–L3 无回退 | AGENTS.md :90「改 game-data/game-core 后」必跑 |
| G4 | ratchet 不回退 | 随 G1 | 求解器新增测试不得使 ratchet 回退（现状 total=1022 / 基线 978） | 计划 :23 全局约束、:191 |
| G5 | 签名锁 | 新增测试（§4 commit2） | 门面 3 个 fn 签名被编译期钉死，漂移即红 | 本件 §1.1 |

### 3.2 「pub 化后被求解器以外误用」的风险与约束

- **风险**：门面一旦 `pub`，任何依赖 game-core 的 crate（game-desktop / game-wasm / game-oracle / 测试）都可调；若语义理解有偏（如把 execute_op 当普通状态机步进滥用、绕过行驱动裸调 parse_op 触发 :402 越界 panic），错误会扩散到门面之外。
- **推荐约束（三件套，本提案采纳）**：
  1. **窄门面模块收口**（推荐，理由见 §1.2）：唯一入口 `scene::solver_api`，3 个 fn 之外无第二条直调路径——审计面 = 一个新文件；比「直接把 `pub(super)` 改 `pub`」多一个文件的代价，换来零既有行改动 + 单点审计。
  2. **`#[doc(hidden)]` + 模块契约 doc 注释**：`#[doc(hidden)]` 不进 rustdoc 公开文档（仓内先例：`lib.rs:29-33` scene_test_support「非生产 API」同款处置）；模块 doc 写明「唯一合法消费者 = game-solver 批1；新消费者须先改本提案」。
  3. **签名锁测试**（G5）：把 §1.1 表的 3 个签名以 `let _: fn(...) = ...` 形式编译期钉死，签名漂移即测试红——防「顺手改签名」破坏消费端。
- **替代案（不推荐）**：直接 `pub(super)` → `pub` + doc 注释。diff 更小（改 3 个词），但：改的是解释器核心行（触发「为何改」审计）；无收口（22 个 op_* 的可见性边界模糊化）；无 doc(hidden) 护栏。仅当对抗复核认定「转发层多余」时降级采用，且须仍过 G1-G4 + 补签名锁。

## 4. 执行切片（问题 4，过审后实施，2 个小 commit）

- **Commit 1 —— 门面**：新建 `crates/game-core/src/scene_solver_api.rs`（3 个纯转发 fn + `#[doc(hidden)]` + 模块契约 doc），`scene.rs` 加一行 `#[path = "scene_solver_api.rs"] pub mod solver_api;`（挂载风格与 scene.rs:43-44/:55-56 一致）。**diff = 新文件 + 1 行挂载**，底层零改动 → 直接过 G1-G4。
- **Commit 2 —— 签名锁测试**：G5 签名锁（3 个 `let _: fn(...)` 编译期断言）+ 门面冒烟（parse 一条 fixture 语料 → execute → 回读字段），落 game-solver 或 game-core 测试侧。
- **批1 消费（本提案过审后）**：`script_abs.rs` 从自建 23 枚举转移表改为直调门面（`parse_op` + `execute_op` + 行驱动 `execute_script_instruction`）；**对拍闸不删、降级为回归校验**（§5）。计划 :295「自建，不依赖 crate 私有的 parse_op/execute_op」原案句由本裁定作废，计划文档本身不改（纪律：不改计划；修订走计划的正常变更流程，不混入本件）。

## 5. 对拍不删（问题 5：构造性同语义 ≠ 免检）

- **原则**：直调是**构造性**同语义（求解器与现解释器共用同一实现，语义由同一份代码保证），但构造性保证**不豁免**验证——门面可能**接错**（转发到错的 fn、参数序错、守卫语义与裸调口径不一致、行驱动循环与原版停摆语义分叉），对拍是唯一能抓住「接错」的闸。
- **保留方式**：计划 §4 批1 的对拍判据**原样保留、一字不改**——
  - 计划 :302 对拍句：构造同一 fixture 行 → (a) 经求解器门面直调路径、(b) 经 `ScriptEngine`（script.rs:154，pub）或 `load_level_script`（scene_persist.rs:501，pub）→ 断言可观测状态一致（hp/gold/keys/实体集合）；
  - 计划 :307-311 判据块照旧：`grep -c 对拍 ≥ 1`（存在性闸，防空过）+ fixture 68 槽逐行 0 差异 + `gates/cli.py check` FAIL=0；
  - 变化仅一处：(a) 侧从「求解器自建转移表」换成「门面直调」——(b) 侧（独立实现）**不动**，故对拍仍是**异实现互证**而非自证。
- **仓内先例**（构造性共享仍对拍的旁证）：`scene_tests/entity_route_tests.rs:230` `script_surfaces_op_dispatch_cursor_parity()`——同一 `GameScene` 面上 parse/execute 游标一致性仍单列一条对拍测试。
- **批5 衔接**：对拍抓「接错」，批5 :382 五元组判别器抓「分叉」（求解器预测 vs 真引擎实测），两闸正交，均不因直调删除。

## 6. 待复核攻击面（自测清单，复核者逐条验）

1. **暴露面是否真的最小？**——门面是否只有 §1.1 的 3 个 fn？有没有顺手把 22 个 `op_*`（script_ops.rs:60-316）或 `lock_camera_on`（scene.rs:3191）也开了？有没有改底层 `pub(super)` → `pub`（应零改动）？
2. **有没有顺手改行为？**——`git diff` 应只见新文件 + scene.rs 一行挂载；`script_ops.rs`/`scene_script.rs`/`script_host.rs` 必须零 diff。
3. **ScriptHost 面结论是否成立？**——复核 §1.3：`script_host.rs:49` 是否真 pub、`ScriptResolver`（:71-95）是否真的只写脚本域、`script_ops.rs:556-559` 是否真无注入缝（外部 impl ScriptHost 会不会被 GameScene 调用）。
4. **execute_script_instruction 进暴露面是否过度？**——若认为 3 个太多，请指出求解器如何在**不复刻** `scene_script.rs:16-24`（依赖私有 `lock_camera_on` scene.rs:3191）的前提下完成行尾收尾与未知 op 停摆。
5. **行号锚实测性**——抽查本件 ≥6 个行号 sed/grep 复跑，核对与引文一致（引锚漂移是惯犯；另核 `git diff df4e3ea..HEAD` 四文件为空的声明）。
6. **门面挂载位置是否正确**——`solver_api` 必须在 scene 树内（scene.rs 挂载），若放 lib.rs 层则 `pub(super)` 转发**编译不过**（可见性物理约束）；复核者可尝试编译验证。
7. **签名锁是否真锁**——临时改门面 fn 签名，G5 测试必须变红。
8. **对拍闸是否保留**——计划 :302/:307-311 判据是否原样（尤其 (b) 侧独立实现未被换成门面自证）。
9. **门禁是否全绿**——G1-G4 五项输出是否齐全（13/13、金样 74、trace、ratchet、SKIP=visual-diff 显式）。
10. **纪律合规**——本件是否只新建这一个文档（`git status` 应只见该 md 新增）；文档头是否注明「用户 2026-11-09 裁定方案 B，批1 前过对抗复核」；未过对抗复核不推送。

## 7. 证据指针表（全部 sed/grep 实测于 HEAD fac2c65）

| # | 事实 | 锚（实测行号） | 级别 |
|---|---|---|---|
| E1 | `parse_op` 现状签名与可见性 | `crates/game-core/src/script_ops.rs:401` | A |
| E2 | `execute_op` 现状签名（&mut GameScene） | `crates/game-core/src/script_ops.rs:553` | A |
| E3 | execute_op = 硬编码 ScriptResolver + match 23 臂 + GIN 尾递归 | `script_ops.rs:554-559`（自建无注入缝）、`:560-586`（match）、`:587-589`（尾递归 :588） | A |
| E4 | 22 个 `op_*` helper 全 `pub(super)`（第 23 变体 RosNoop 无 helper，`:542` parse 兜底、`:584` 空臂） | `script_ops.rs:60-316`（grep 计数=22） | A |
| E5 | `parse_op` 体无守卫（越界 panic 面） | `script_ops.rs:401-405`（`:402` 切片） | A |
| E6 | 行驱动三件套：行尾收尾/未知 op 停摆/execute 调用 | `scene_script.rs:16-24`（守卫+收尾）、`:29`（parse let-else 停摆）、`:35`（execute_op） | A |
| E7 | 收尾依赖私有 `lock_camera_on` | `scene.rs:3191`（无 pub） | A |
| E8 | 模块挂载链（pub(super)=pub(in scene)，门面须挂 scene 树） | `scene.rs:55-56`（script_ops）、`:43-44`（scene_script）、`lib.rs:23`（pub mod scene） | A |
| E9 | ScriptHost 面 pub 但只写脚本域 | `script_host.rs:48/49`（trait+方法）、`:61`（Resolver）、`:71-95`（实现体）、`subsystem/mod.rs:25` | A |
| E10 | ScriptOp 已跨 crate pub | `script_ops.rs:6`、`scene.rs:60`、`script_host.rs:31` | A |
| E11 | 状态/五元组/实体字段全 pub | `scene.rs:282/:411-419`、`combat.rs:51`、`entity.rs:57` | A |
| E12 | 装载与对拍 harness 面已 pub | `scene_persist.rs:501`、`script.rs:154/:162/:270` | A |
| E13 | doc(hidden) 仓内先例 | `lib.rs:29-33`、`scene.rs:4624-4626` | A |
| E14 | 计划后门原话 / R7 风险行 | `plans/2026-11-09-solver-implementation-plan.md:224` / `:414` | A |
| E15 | 计划批1 原案、对拍句、判据块 | 同文件 `:295` / `:302` / `:307-311` | A |
| E16 | 门禁现状与金样 74 | `gates/cli.py status` 实跑（PASS=13 FAIL=0 SKIP=1 TOLERATED=0）；计划 `:184/:191` | A |
| E17 | 三问预检出处 | `AGENTS.md:70`（§4 标题）、`:75-77`（三问原文）、`:90`（改 game-core 后跑 trace）、`:36`（game-core 红线） | A |
| E18 | 对拍先例 | `scene_tests/entity_route_tests.rs:230` | A |
| E19 | fixture 真相源 68 槽 | `scene.rs:252-253`（include_str）、计划 `:131` | A |
| E20 | 批5 五元组判别器（字段源 combat.rs:51） | 计划 `:382/:458`（A11） | A |
