# game-core 求解器直调门面提案（方案 B）对抗复核报告

> 被审件：`docs/superpowers/proposals/2026-11-09-game-core-solver-facade-proposal.md`（commit `31ac853`，docs-only，未推送）
> 复核方式：对抗复核（只读；唯一写 = 本报告；不改代码、不改提案、不推送）；逐面 sed/grep/git 实测，证据 A/B 级；纪律 = AGENTS.md
> 日期：2026-11-10 · HEAD=`31ac853`（modern，领先 origin/modern 6 = 未推送，与任务书一致）· 基线 `df4e3ea`

## 总判：**需修订后放行**（1 红点、8 黄点；方案本体实测可执行，无结构性伤害）

方案本体（3 个纯转发 fn / ScriptHost 不充分论证 / 三件复刻论证 / 五闸 / 2-commit 切片 / 对拍保留原则）逐面实测**成立**：暴露面无漏开无开多、「不充分」论证为真、三件复刻中两件实为必需、行号锚 20 条抽测 19 中、`git diff df4e3ea..31ac853` 四文件确为空、commit `31ac853` 恰 1 文件零夹带、门禁 status 实跑复现 `PASS=13 FAIL=0 SKIP=1 TOLERATED=0`。

**唯一红点 R1**：§5「(b) 侧独立实现不动，故对拍仍是异实现互证而非自证」是一条**有条件且未被钉死的承诺**——门面打开前 (b) 实际只能走 `ScriptEngine`（GameScene 执行路径 pub(super) 不可达），门面打开后 (b) 可被写成 `load_level_script + solver_api::execute_script_instruction`，与 (a) 同一实现 → 对拍恒过、`grep -c 对拍 ≥ 1` 存在性闸抓不到，G1-G4/G5 也全抓不到（金样/trace 不经过门面，签名锁只锁签名不锁转发目标）。修复 = §5 加一句话钉死 (b)；**修订合入前不得派工批1**。

置信度：**高（≈90%）**——锚、git、门禁、可见性静态推导全部实测可复现；编译面与全量 check 未实测（见盲区）。

---

## 面1 · 暴露面最小性 —— **PASS（3 个够用，无漏开、无开多）**

**判据**：读计划 `docs/superpowers/plans/2026-11-09-solver-implementation-plan.md` 批1（:291-314）推导符号消费清单，逐个对提案 §1.1 的 3 个转发 fn；再查有没有批1 仍需自建（漏开）或提案多开。

**实测**：批1 四项内容（:295 script_abs.rs / :296-300 A/B/C 分类 / :301 分类单测 / :302 对拍）的符号消费为：

| 批1 需求 | 所需符号 | 实测现状 | 提案覆盖 |
|---|---|---|---|
| 23 变体解析（分类单测逐 op 驱动） | `parse_op` | `script_ops.rs:401` `pub(super)`（跨 crate 不可达） | 门面 #1 ✓ |
| A 类状态变换执行（GIN/ADD/DES/CES/MOV/ROS_*/LAY/SWD/RES/GUT） | `execute_op`（23 臂 match → 22 `op_*`，`grep -c`=22，`script_ops.rs:60-316`） | `script_ops.rs:553` `pub(super)` | 门面 #2 ✓ |
| 行驱动对拍（guard+收尾+停摆） | `execute_script_instruction` | `scene_script.rs:14` `pub(super)` | 门面 #3 ✓ |
| 状态读取（checkpoint/对拍断言） | `GameScene` 字段 `scene.rs:411-419`、`PlayerCombat combat.rs:51-60`、`EntityTable entity.rs:57` | 全 `pub`，跨 crate 可达 | 无需开 ✓ |
| 装载/(b) 侧 | `load_level_script scene_persist.rs:501`、`ScriptEngine script.rs:154/162/270`、测试构造 `lib.rs:29-33` | 全 `pub`（含 doc(hidden) 先例） | 无需开 ✓ |
| 依赖 | `crates/game-solver/Cargo.toml` 已含 `game-core = { path = "../game-core" }` | 实测已含 | 无需改 ✓ |

- **无漏开**：上述清单之外，批1 不再消费任何 game-core 私有符号（词法 `lex_instr_stream`/`parse_guts_line` 属 game-data，批0 ingest 已落，不属本提案暴露面）。
- **无开多**：22 个 `op_*`（`script_ops.rs:60-316`，全 `pub(super)`）、私有 `lock_camera_on`（`scene.rs:3191`）均未开；§1.4 明列「零改动清单」且实测全部已 pub / 不动。3 个 fn 各有独立消费者（分类单测 = #1+#2；行驱动对拍 = #3），无冗余。
- **旁证（#3 非过度）**：行尾收尾写 `script_line_flags[cur]`（`scene_script.rs:20-22`），该字段是脚本触发门的实读输入（`scene.rs:930-1029` 十余处 `gate &= ...flags[...]`）——收尾语义对批3 可达性/批5 回放**是状态相关的**，不是纯展示。

## 面2 · 「ScriptHost 不充分」论证真伪 —— **PASS（结论为真；1 处行锚瑕疵 = Y1）**

**判据**：`script_host.rs:71-95` 逐行读，确认不写 gold/实体/hp；`script_ops.rs:556-559` 确认硬编码无注入缝。若论证错 → 3-fn 面过度 = 红。

**实测**：
- `script_host.rs:48` `pub trait ScriptHost` / `:49` `fn execute_op(&mut self, ctx, op) -> ScriptStep` 确为 pub（跨 crate 可调）；`:61 pub struct ScriptResolver`；`subsystem/mod.rs:25 pub mod script_host` ✓。
- `ScriptResolver::execute_op` 实现体实测 `script_host.rs:71`（fn）→ 92（fn 闭）/93（impl 闭），体只做四件：`*ctx.script_cursor = op.cursor()`（**实测 :72**，提案写 :74 = Y1）、Gut 旗标写（:74-79）、Ces delay 写（:81-88）、Gin → `ScriptStep::Recurse`（:89）；其余臂 `_ => Advance`。**确实不碰 gold/实体/hp** ✓。
- `script_ops.rs:556-559` 实测硬编码：`let mut ctx = self.script_ctx();`（556）`let mut r = crate::subsystem::script_host::ScriptResolver;`（557）`r.execute_op(&mut ctx, op)`（558）——**无任何注入缝**；外部 impl ScriptHost 的确永远不会被 GameScene 调用 ✓。
- A 类变换确在 `GameScene` 侧：`script_ops.rs:560-586` 23 臂逐臂 `self.op_*`（op_tak:60 / op_res:101 / op_add:120 / op_mov:175 / op_des_all:208 …），`ScriptResolver` 摸不到 ✓。

**裁定**：论证成立，ScriptHost 面免费但不充分，3-fn 面**非过度暴露**。

## 面3 · 三件复刻论证 —— **PASS（2 实 1 弱，不影响结论）**

| 三件 | 实测 | 是否真在路径上 / 绕过是否漂移 | 裁定 |
|---|---|---|---|
| ① 行尾收尾依赖私有 `lock_camera_on` | `scene.rs:3191` `fn lock_camera_on(&mut self, target: i32)` **无 pub（私有）**；调用点 `scene_script.rs:24`（守卫分支内，`:17 if script_cursor0 + 3 > line.len()` → `:19 set_dialog_phase(4)` → `:21-22 flags[cur]`（cur!=32）→ `:24 lock_camera_on(0)`） | 真在 `execute_script_instruction` 路径上；求解器复刻收尾 = 要么改可见性（扩面）要么跳过（`script_line_flags` 是触发门输入 → 真漂移面） | **实** ✓ |
| ② 未知 op 静默停摆 | `scene_script.rs:29` `let Some(op) = Self::parse_op(...) else { ... return; }`（a.java:8594-8595），且注释明令「禁止 3+N 跳字节续行」 | 真在路径上；绕过需自行复刻 let-else + 不推进游标语义 | **实** ✓（一行但属语义） |
| ③ GIN 同拍尾递归宿主 | `script_ops.rs:587-589`，调用行实测 `:588 self.execute_script_instruction();` | 在 `execute_op` **内部**——只要开 #2，尾递归经 game-core 内部固有调用自动生效，**不以门面暴露 #3 为前提** | **弱**（见 Y6：「语义自洽」修辞，非必要性） |

**裁定**：①② 支撑 #3 必要性成立（求解器绕过行驱动真会漂移）；③ 论证措辞偏强但无实害。#3 不判定为开多。

## 面4 · panic 风险 —— **处置写清且基本合理（黄 Y7：冒烟未盖前置）**

**判据**：`parse_op` 体是否无守卫、转发是否带守卫、提案处置是否写清且合理。

**实测**：
- `script_ops.rs:401` fn、`:402` `let op = &line[script_cursor0..script_cursor0 + 3];` —— 体**无守卫**，越界/非字符边界即 panic ✓（提案自认属实）；守卫只在 `scene_script.rs:17`（`script_cursor0 + 3 > line.len()`），经行驱动路径调用时 `len ≥ cursor+3` 切片必界内 → **行驱动路径不可达 panic**。
- 转发 fn 设计为**纯转发、不带守卫**（§1.1 纯委托承诺）→ 裸调 `parse_op` 坏游标 = panic。
- fixture 语料 `crates/game-core/tests/fixtures/level_scripts.txt` 实测**非 ASCII 字符 = 0**（全行 ASCII，无字符边界 panic 面）；求解器运行期行源 = fixture（well-formed）+ 自身维护游标 → 实际崩面小，且批1 分类单测中 panic = 测试红，非无头事故。
- 提案处置写在两处：§1.1 第 2 条「求解器绕过行驱动直接裸调 `parse_op` 需自带守卫，否则踩 panic 面」；§3.2 风险 1「绕过行驱动裸调 parse_op 触发 :402 越界 panic」。**写清了**。

**裁定**：合理（守卫留在唯一权威路径，转发层零语义漂移，前置条件写明）。残余黄点 Y7：§4 Commit2 冒烟（parse→execute→回读）**未覆盖**守卫前置用例，建议 facade 模块 doc 写死前置条件「`script_cursor0 + 3 <= line.len()`，否则 panic；行驱动请走 #3」+ 冒烟补一条行尾收尾断言。

## 面5 · 三问预检 + 风险闸 —— **PASS（三问真答且可核；五闸可执行；1 替代案漏点 = Y5）**

- **三问真答**（出处实测 `AGENTS.md:70`（§4 标题）/`:75-77`（三问原文）/`:90`（改 game-core 跑 trace）/`:36`（红线）✓）：
  1. 层 = **可见性层**（`pub(super)`=`pub(in scene)`，跨 crate 不可达）非行为层——与实测模块挂载链（`scene.rs:55-56` 私有 mod、`lib.rs:23 pub mod scene`）相符 ✓；
  2. 消费者 = game-solver `script_abs.rs` 读 fixture（`scene.rs:252-253` include_str 同源、计划 `:131` 3609B/68 行）+ pub 字段 + 回读对拍——逐项实测为真 ✓；
  3. 证据 = A 级（sed/grep 实测 + `git diff df4e3ea..31ac853` 四文件空——**复核实测确为空** + status 实跑）✓。
- **五闸可执行**：G1 `gates/cli.py check`、G2 `tools/state_baseline.py --check`（基线 `state-dump-golden.json` 实测 `scenarios` 计数 **=74** ✓）、G3 `gates/cli.py trace`、G4 ratchet 随 G1、G5 签名锁（`let _: fn(...)` 编译期钉 3 条签名，机制成立）。`python3 gates/cli.py status` 复核实跑输出 `PASS=13 FAIL=0 SKIP=1 TOLERATED=0`（SKIP=visual-diff 显式打印）——**与提案 E16 逐字相符** ✓。
- **签名锁锁什么**：锁 §1.1 的 3 个转发 fn 签名（编译期 fn 指针断言）——具体、可执行；但**不锁转发目标**（同签名换实现仍绿，见 Y4）。
- **doc(hidden) 是否足够 / 漏了什么**：`#[doc(hidden)]` 只是文档层护栏（先例实测 `lib.rs:29-33`、`scene.rs:4624-4626` 同款）；任何依赖 crate 仍可调。提案对比了「直接 pub」替代案但**漏了 cargo feature gate**（`crates/game-core/Cargo.toml` 实测无 `[features]` 段）这一编译期强选项，也未给 doc(hidden) 相对 feature 的取舍理由 → Y5（不判红：窄模块收口 + 签名锁 + 对拍已构成三层约束，feature 的代价是 Cargo 改动与 workspace feature unification）。

## 面6 · 锚实测 —— **20 条抽测：19 中、1 错（Y1）、1 边界宽（Y1b）；四文件零 diff 声明为真**

- **六条声明行号全中**：`parse_op script_ops.rs:401`（体 :402 切片 ✓）/ `execute_op :553`（:556-559 硬编码 ✓、:560-586 match 23 臂 ✓、:587-589 尾递归 :588 ✓、:584 RosNoop 空臂 ✓）/ `execute_script_instruction scene_script.rs:14`（守卫 :17、收尾至 :24、停摆 :29、execute :35 ✓）/ `lock_camera_on scene.rs:3191` 无私有 ✓ / `script_host.rs:71-95 实现体`（fn 实测 :71，**闭于 :92/impl :93，71-95 略宽** = Y1b）/ `script_ops.rs:556-559` ✓。
- **其余抽测全中**：`op_*` 计数 `grep -c "pub(super) fn op_" = 22` 且首 `op_tak:60` 末 `op_mvs:316`；`script_ops.rs:6 pub enum ScriptOp`、`:542 RosNoop 兜底`；挂载链 `scene.rs:55-56/:43-44/:60`、`lib.rs:23`；`scene.rs:282/:411-419/:252-253/:4624-4626`、`lib.rs:29-33`；`combat.rs:51-60`（hp:52 gold:58 三色钥匙:55-57 item_types:60）、`entity.rs:57`；`scene_persist.rs:41/269/501`、`script.rs:154/162/270`；`entity_route_tests.rs:230`；`AGENTS.md:36/70/75-77/90`；计划 `:23/:131/:184/:191/:224/:295/:302/:307-311/:382/:414/:458` 逐行命中。
- **唯一错锚（Y1）**：§1.3/E9 称游标写 `*ctx.script_cursor = op.cursor()` 在 `:74` ——**实测 :72**（:74 是 `ScriptOp::Gut` 臂头）。结论（只写脚本域）不受影响。
- **行号两侧一致声明实测为真**：`git diff df4e3ea..31ac853 -- script_ops.rs scene_script.rs scene.rs subsystem/script_host.rs` = **空**（`git diff --stat` 无输出）；`df4e3ea..fac2c65` 同四文件亦空。提案头写「实测于 HEAD `fac2c65`」而当前 HEAD=`31ac853`——其间 `0dc9832` 只动 `intro.rs` 文档（commit message 实证），四文件未动，锚仍有效。
- 信息性偏差（Y2）：G4 写「现状 total=1022」（引计划 :23 旧数），status 实跑现值 **total=1029**（基线 978 不变）；仅影响文字，不影响闸判据。

## 面7 · 对拍保留 —— **引文一字相符 ✓；但 (b) 侧二义性 = 红点 R1**

- **原文相符**：计划 `:302` 对拍句、`:307-311` 判据块（307 script_abs 存在性 / 308 分类单测 / 309 `grep -c 对拍 ≥1` / 310 fixture 68 槽 0 差异 / 311 `gates/cli.py check` FAIL=0）实测与提案 §5 引文**逐字一致**，提案承诺「一字不删」与计划原文相符 ✓；(b) 侧在计划原文中 = "`ScriptEngine`（script.rs:154，pub）**或** GameScene `load_level_script`（scene_persist.rs:501，pub）" ✓。
- **红点 R1（(b) 可被门面污染 → 对拍自证空过）**：
  - 门面打开**前**：GameScene 的执行方法 `execute_op/execute_script_instruction` 均 `pub(super)`（`script_ops.rs:553`、`scene_script.rs:14` 实测），(b) 用 GameScene 路径**物理不可执行** → (b) 实际只能 `ScriptEngine`（实测 `script.rs` 不 `use` ScriptOp/GameScene，仅 :172 注释提及——确为独立实现）→ 原计划「异实现互证」成立。
  - 门面打开**后**：(b) 可写成 `load_level_script` + `solver_api::execute_script_instruction`，与 (a) **同一份实现** → 对拍恒等通过；`grep -c 对拍 ≥1`（:309）是存在性闸抓不到；G1-G4 抓不到（金样/trace 不经过门面）；G5 只锁签名不锁转发目标（Y4）。此时 §5「(b) 侧不动，故对拍仍是异实现互证而非自证」的承诺**失守**，而该闸是提案自认唯一能抓「接错」的闸（§5 原则段）。
  - **修复（一句话）**：§5 明写「(b) 侧唯一合法路径 = `ScriptEngine`（script.rs:154）；**禁止以门面 solver_api 路径充当 (b)**，违者对拍判无效」。
  - 严重度判红依据：AGENTS.md §1「裁判是差分测试，不是看起来对」；闸空过 = 提案核心承诺（对拍保留）实质失效，且静默。

## 面8 · 夹带 —— **PASS（零夹带）**

- `git show 31ac853 --stat`：**恰 1 文件** `docs/superpowers/proposals/2026-11-09-game-core-solver-facade-proposal.md`，`149 insertions(+)`，无计划/代码改动 ✓。
- `git status --porcelain` 过滤 `??` 后**无任何已跟踪文件修改**（工作树干净；未跟踪仅 `android/build/` 与既往 review 文档）✓。
- `git status -sb` = `modern...origin/modern [领先 6]` = 未推送，与任务书「未推送」一致 ✓。
- 提案自述「只立文档不改代码不改计划」与实况相符 ✓。

## 面9 · 可执行性（2-commit 切片） —— **PASS（自洽；1 锚漂移提示 = Y3）**

- **Commit 1**（§4）：新建 `crates/game-core/src/scene_solver_api.rs`（实测该文件不存在、全仓无 `solver_api` 引用 = 确为新文件）+ `scene.rs` 加一行 `#[path = "scene_solver_api.rs"] pub mod solver_api;`（挂载风格与 `scene.rs:43-44/:55-56` 实测一致）。**转发 fn 写在新文件、确实会动 scene.rs（1 行）**。
- **与「底层 pub(super) 零改动」自洽**：「底层」= `script_ops.rs`/`scene_script.rs`/`subsystem/script_host.rs`（一个字符不改 ✓）；scene.rs 挂载行是**提案明文披露的**（§1.1/§4），非夹带。提案头「四文件零改动」是**历史声明**（df4e3ea..HEAD 锚一致性，实测为真），非未来承诺——两者不矛盾。
- **可见性推导正确（B 级静态推理）**：转发 fn 声明在 scene 后代模块内 → 可触达 `pub(super)`=`pub(in scene)` 的三方法；`ScriptOp` 经 `scene.rs:60 pub use` 可引；若放 lib.rs 层，`scene.execute_op(...)` 因方法可见性 `pub(in scene)` 而**编译不过**——与提案 §1.2/攻击点 6 一致。**编译未实测**（复核约束：不改代码）→ 归盲区，实施 Commit 1 第一步即验证。
- **Commit 2**（§4）：签名锁（3 条 `let _: fn(...)` 编译期断言）+ 门面冒烟——可照做；落 game-solver 或 game-core 测试侧均不触碰底层三文件 ✓。
- **Y3**：Commit 1 插挂载行后，`scene.rs` 全部行锚（`:3191 lock_camera_on`、`:282`、`:411-419`、`:60`、`:252-253`、`:4624-4626`）将 +1~2 漂移；提案未提示（与其自设 §6.5 防引锚漂移标准相悖）。建议：实施 commit message 记录漂移量，或提案补一句「实施后 scene.rs 锚 +N」。

## 面10 · 攻击面自测清单逐点裁定（提案 §6 十点）

| # | 攻击点 | 复核裁定 |
|---|---|---|
| 1 | 暴露面是否真最小 | **过**：设计面恰 3 fn（面1）；22 `op_*`/`lock_camera_on` 未开、零既有符号可见性改动——提案文本与实测相符（尚未实施，实施后按 §6.1 复验） |
| 2 | 有无顺手改行为 | **过**：`31ac853` 恰 1 文档文件；工作树零已跟踪修改；实施承诺 = 新文件 + scene.rs 1 行，底层三文件零 diff |
| 3 | ScriptHost 结论是否成立 | **过**（面2）：48/49 pub、71-92 只写脚本域、556-559 硬编码无注入缝，全实测为真；瑕疵 = Y1 锚 :74→:72 |
| 4 | #3 是否过度 | **不过（判非过度）**：收尾写 `script_line_flags` 且该字段被触发门 `scene.rs:930-1029` 实读（状态相关）；`lock_camera_on` 实测私有且在收尾路径；停摆 :29 在路径上。唯一弱论据 = GIN 尾递归（Y6，不改变裁定） |
| 5 | 行号锚实测性 | **过（带瑕疵）**：抽测 20+ 条，19 中、1 错（:74→:72）、1 略宽（71-95→fn71-92）；四文件空 diff 声明实测为真 |
| 6 | 挂载位置 | **过（静态推理）**：必须 scene 树内（可见性物理约束，推导正确）；编译验证归盲区（约束不改码），实施首步补验 |
| 7 | 签名锁是否真锁 | **机制成立、待实施验证**：`let _: fn(...)` 编译期钉签名，改签名必红；残余 = 不锁转发目标（Y4，靠 R1 修复后的对拍兜底） |
| 8 | 对拍闸是否保留 | **红 = R1**：引文一字相符，但 (b) 侧「ScriptEngine 或 load_level_script」二义在门面打开后可退化为同实现自证；须钉死 (b)=ScriptEngine |
| 9 | 门禁是否全绿 | **过**：status 实跑复现 `PASS=13 FAIL=0 SKIP=1 TOLERATED=0`（SKIP=visual-diff 显式）；金样基线 scenarios=74 实计数；G1 全量 check/G2 --check 未重跑（盲区，见下） |
| 10 | 纪律合规 | **过**：恰 1 文档 commit、头部含「用户 2026-11-09 裁定方案 B，批1 前过对抗复核」（提案 :3）、未推送（领先 6） |

---

## 红点清单（合入/派工前必修）

- **R1（面7/攻击点 8）**：§5 未钉死 (b) 侧唯一合法路径 = `ScriptEngine`（script.rs:154），存在「门面路径充当 (b) → 对拍同实现自证空过、全闸抓不到」的静默失效通道。**修法**：§5 增一句「(b) 侧唯一合法路径 = ScriptEngine；禁止以 solver_api 门面路径充当 (b)，违者对拍判无效」。修订合入前不派工批1。

## 黄点清单（不阻断，建议随修订或实施处理）

- **Y1** `script_host.rs` 行锚：§1.3/E9 游标写标 `:74`，实测 **:72**（:74 = Gut 臂头）；E9「71-95 实现体」实为 fn 71-92 / impl 闭 93。结论不受影响，按「引锚=实测行号」纪律改正。
- **Y2** G4「现状 total=1022」与实测 status `total=1029` 不符（引的是计划 :23 旧数）；信息性，改字即可。
- **Y3** Commit 1 挂载行使 scene.rs 全部行锚 +1~2 漂移，提案未提示（与自身 §6.5 防漂移标准相悖）；实施 commit message 记录或提案补注。
- **Y4** G5 签名锁只锁签名不锁转发目标（同签名换实现仍绿）；残余风险靠 R1 修复后的对拍兜底——R1 不修则此残余无兜底。
- **Y5** §3.2 替代案只对比「直接 pub」，漏 cargo feature gate（game-core 现无 `[features]`）这一编译期强选项，且未给 doc(hidden) 相对 feature 的取舍理由；可不采纳，但应补一句理由。
- **Y6** §1.1 第 3 条 GIN 尾递归论证偏强：尾递归在 `execute_op` 内部（`script_ops.rs:588`），开 #2 即自动生效，不以暴露 #3 为前提；#3 必要性实由第 1、2 条支撑（均实测成立）。建议措辞降为「配套」而非「不开则复刻」。
- **Y7** §4 Commit2 冒烟（parse→execute→回读）未覆盖守卫前置；建议 facade 模块 doc 写死前置条件 `script_cursor0 + 3 <= line.len()`（否则 panic；行驱动走 #3）+ 冒烟补行尾收尾断言。
- **Y8** 计划 `:295/:224/:414` 与裁定矛盾的文本保留（提案声明走计划正常变更流程，合规）；建议批1 派工单显式引用本提案，防未来读者照 :295 自建转移表。

## 盲区（本复核未能覆盖）

1. **未编译验证门面**（复核约束：只写本报告、不改代码）——挂载语法/可见性为 B 级静态推理；实施 Commit 1 首步即为编译验证。
2. **未重跑全量 `gates/cli.py check`**（只实跑了只读 `status`，13/13 已复现）；G1 判据的全量执行留给实施闸。
3. **未执行 `tools/state_baseline.py --check`**（只验证基线文件存在且 scenarios=74 计数）；G2 实跑留给实施闸。
4. **G5 签名锁红测未执行**（门面未实现）；攻击点 7 的「改签名必红」是机制推断。
5. **R1 是文档级风险预测**：批1 实现者实际如何写 (a)/(b) 未发生；修复的必要性以「闸空过不可观测」为据。
6. 批1 是否仍需 game-data 词法（计划 :295 `lex_instr_stream` 复用）与门面的关系未深究——不属本提案暴露面，留批1 复核。

## 证据索引（本报告全部实测于 HEAD=`31ac853`，分支 modern）

- git：`git show 31ac853 --stat`（1 文件 149+）、`git diff df4e3ea..31ac853 -- <四文件>`（空）、`git status -sb`（领先 6 未推送）、`git status --porcelain`（零已跟踪修改）
- 代码：`crates/game-core/src/script_ops.rs:6/60-316/401-402/542/553-589`、`crates/game-core/src/scene_script.rs:14/17-35`、`crates/game-core/src/scene.rs:43-60/252-253/282/411-419/930-1029/3191/4624-4626`、`crates/game-core/src/subsystem/script_host.rs:31/48-49/55/61/71-93`、`crates/game-core/src/subsystem/mod.rs:25`、`crates/game-core/src/lib.rs:23/29-33`、`crates/game-core/src/combat.rs:51-60`、`crates/game-core/src/entity.rs:57`、`crates/game-core/src/scene_persist.rs:41/269/501`、`crates/game-core/src/script.rs:154/162/270`、`crates/game-core/src/scene_tests/entity_route_tests.rs:230`、`crates/game-solver/Cargo.toml`（deps 已含 game-core）
- 计划/纪律：`docs/superpowers/plans/2026-11-09-solver-implementation-plan.md:23/131/184/191/224/291-314/382/414/458`、`AGENTS.md:36/70/75-77/90`
- 门禁/基线：`python3 gates/cli.py status` → `PASS=13 FAIL=0 SKIP=1 TOLERATED=0`（SKIP=visual-diff 显式）；`data/status/baselines/state-dump-golden.json` scenarios=74（实计数）
- 语料：`crates/game-core/tests/fixtures/level_scripts.txt` 非 ASCII 计数 = 0
