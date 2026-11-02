# 对抗复核 B：scene.rs 模块拆解方案 — 契约/迁移安全攻击

> 日期：2026-11-02 ｜ 复核员：对抗复核员 B（remake-verifier 纪律，契约安全视角）
> 攻击对象：[2026-11-02-scene-module-decomposition-plan.md](../../superpowers/plans/2026-11-02-scene-module-decomposition-plan.md)（v1）
> 判定图例：🔴 红（方案缺项/不可接受，必须修订）｜🟡 黄（可接受但有缺口）｜🟢 绿（充分）
> 本文档每节完成后即写回；结论区分"已证实 / 假设"。

## 攻击点总览

| # | 攻击点 | 判定 | 修订条数 |
|---|---|---|---|
| 1 | TraceState 契约充分性 | 🟡（吸收力已实证🟢，命名/覆盖/步进纪律缺） | 3 |
| 2 | 游标语义雷区 | 🟡（RED 先行已满足且有实绩，钉死集未入方案、变体薄） | 3 |
| 3 | 迁移期双轨风险 | 🔴（Q5 前提过时+真双面未接+档 B 新两处真相风险） | 3 |
| 4 | 验证矩阵完备性 | 🟡（三件对但计数失实、漏两个完备性守卫、域测边界缺） | 3 |
| 5 | 回退策略 | 🔴（缺回退设计=红，蓝图已有 v1 丢失） | 2 |

> 跨项发现 R0（方案级红）：§0/§3 问题基线陈旧——见汇总裁定。

## 1. TraceState 契约充分性

**判定：🟡** —— "吸收内部布局变化"的机制成立且有 A 级实证先例；但方案
"trace_snapshot（步 0 已落）"名实不符、双机制混谈回退了 A2 已裁决的更正、
dump 外字段与域步进纪律未写。

### 已证实（A 级）

- **名实不符**：全仓 `fn trace_snapshot|struct TraceState`（*.rs）**零命中**。
  实际落点 = `crates/game-oracle/src/state_dump.rs::dump_scene(&GameScene)`（49 项
  dump：38 标量 + 9 实体数组 + enemyBaseGold + walkPathBuffer）+ 契约件
  `data/status/baselines/trace-contract.json`（schema `TraceState-contract-1`，
  `reference/oracle/fields.py --contract` 生成式 411 真槽+49 dump 项全映射）+
  行序冻结 `state-dump-golden.json` 的 `line_order_freeze` 声明。
- **吸收力实证**：SoA→struct 全量迁移已一次落毕（commit `313375b`，Entity 11 字段
  +427 站点，**金样零漂移**）——dump_scene 按 `entities.items` 映射重建 9 条 SoA
  整表（state_dump.rs:107-133 明注"push 序/值序/尾默认逐字节保持"）。域外移同理：
  变的是取数路径，行序/值序被契约冻结面锁死 ⇒ **oracle dump 拆解后成立**。
- **双机制分列**（refactor-plan-review A2 已裁决）：FLD 413 槽=Java 反射序
  （StateDump.java 侧），Rust 拆解**零触碰**（"布局迁移天然不影响"）；Rust 侧
  49 项 push 序才是布局敏感面。方案"trace 契约=TraceState"措辞把契约重新套回
  FLD 坐标系 = **回退 A2 更正**（A2 原话"双机制混谈"）。
- **完备性守卫已存在**：statediff 名集**相等**断言（2026-10-16 修复交集静默绿，
  refactor-behavior-matrix §0 ②✅）；金样实数 = **56 场景**（state-dump-golden.json
  `scenarios` 实测 len=56，非方案所称 47）。
- **dump 外字段**（49/413 覆盖）：script_cursor、dialog_char（打字机游标 UTF-16）、
  battle_enemy_hp、route_*、menu_* 等不入逐字节比对，只受帧哈希+场景行为间接保护。

### 修订建议

- **R1.1** "trace_snapshot（步 0 已落）"改为实名引用三件：state_dump::dump_scene
  适配层 + fields.py --contract 生成式契约 + line_order_freeze；**禁按蓝图旧名再造
  TraceState struct**（=第三面，gotcha_dual-porting-surfaces-drift 禁令）。
- **R1.2** 红线补一句双机制分列："FLD 倾印 Java 侧不受拆解影响；Rust 侧 49 项
  push 序冻结，域外移只准改取数路径，**禁增删 dump 项**（49 项集合变更=换考题，
  需用户确认+基线重固化）"。
- **R1.3** 执行序档 B 步 5 每域外移加检查项："dump_scene 取数路径适配对照随
  commit + `python3 tools/state_baseline.py --check` 全绿（名集相等断言兜底漏项）"。

## 2. 游标语义雷区（scriptCursor/lastDelimPos）

**判定：🟡** —— "RED 测先行"原则不仅满足、且有真抓实绩；但方案未引用钉死测试集，
双面 parity 变体覆盖薄，lastDelimPos 两面结构差异未提示。

### 已证实（A 级）

- **parse/execute 分离已落**（commit `8a455dd` 金样零漂移）：scene_script.rs
  `parse_op`（纯解析，逐 op 计算**推进后游标**，条件取参清单：MOV 5 参续取/
  DES 正负参分支/ROS 子码条件取参/GUT 无游标 load 重置）+ `execute_op`（统一写
  `self.script_cursor = op.cursor()` → 逐 op 独立函数）。
- **机器证明四件已在**：① 金样逐字节零漂移（state_baseline）② **双面游标对照**
  `script_surfaces_op_dispatch_cursor_parity`（16 op 各一行 + 未知 op 两面同停摆，
  entity_route_tests.rs:215-262）③ trace 锚定游标断言（script_golden.rs t578
  cursor=12 / t1002=24 / t1202=43）④ MOV-miss 尾递归游标位移断言
  （`mov_miss_tail_recurses_into_next_instruction`——行为矩阵 2.3 缺口已关）。
- **测试族有效性实绩**：SWD 游标 orig+5/+4 off-by-one 由两面对照测揪出并修复
  （script.rs:375 注记"5.x 两面对照测揪出"）——A 级证据。
- **薄弱点**：parity 表每 op 仅一行简单参——条件变体（MOV 3/5 参、DES 负参、
  ROS 1-6/其余子码、SWD/MVS 无参=标签后 1 位 a.java:8591）不在 parity 表内；
  组合序列游标轨迹仅两处（MOV+ROS、MOV+TAK，entity_route_tests.rs:183/207）。
- **lastDelimPos 结构差异**：生产面已内联为局部终位（parse_script_int 返回值
  e/e1/e2），harness 面仍是 `state.last_delim_pos` 字段（script.rs:73）——两面
  中间态形态不同，对照测只钉终值 cursor 不钉中间态。

### RED 测设计建议（机器钉死"游标推进逐字节等价"）

- **R2.1** parity 表扩变体行：MOV 3 参/5 参、DES 正/负、ROS 1..6+其余、SWD/MVS
  （游标=标签后 1 位）、GUT（load 后游标重置 0）——每变体断言 (op,参数)→精确
  cursor 值，双面对照沿用现形态（分派/游标层 parity，效果层归 fx 夹具+金样，
  两层分列不混谈）。
- **R2.2** 序列级轨迹断言：固定脚本行序列逐拍 (script_cursor, currentScriptIndex,
  dialogPhase) 三元组轨迹哈希入**独立基线文件**（不入金样，避免换考题）；
  l2_prologue.rs:102 已监视 cursor 变化点，可升格为全场景轨迹。
- **R2.3** 红线写入分歧定位判据：任何游标分歧必须报"首个分歧点（tick n, op,
  旧值→新值）"（diff-case 模板），无首分歧点不下结论；script_ops.rs 归位时禁
  把终位"中间态字段化"（如新增 GameScene 字段暂存 lastDelimPos）= 造新状态面。

## 3. 迁移期双轨风险（SoA vs Entity struct）

**判定：🔴** —— Q5 提问前提已过时（并存期不存在）；真正的双面风险（script.rs/
camera.rs harness 面）方案红线未接；档 B"域自有状态切片"缺唯一归属表 = 亲手造
新两处真相（项目有两次实锤前科）。

### 已证实（A 级）

- **Q5 前提过时**：SoA 与 Entity struct **并存期不存在**——commit `313375b`
  一次性全量切换（427 站点金样零漂移）；现行代码仅 `EntityTable.items` 一源
  （scene.rs:262），state_dump 已按 items 重建整表。方案 §0"SoA 散列数组横穿
  全部域"、§3 步 2"Entity 聚合补完（SoA→struct）"均属陈旧表述。
- **真双面风险在别处且未入方案**：gotcha_dual-porting-surfaces-drift——
  executeScriptInstruction 生产面（scene_script.rs）vs script.rs `ScriptEngine`
  +`HostCtx` harness 面、camera.rs 第二套相机 vs GameView；**DC-9 桥漏道具二倍
  加成 = 漂移已发生的实例**。refactor-plan 条件 6 要求"harness 面随迁移同步或
  显式冻结"，v1 红线无此条。
- **两处真相前科两次实锤**：pattern_dual-copy-algorithms-sync-fix（迭代 26 修
  全量版 `\c` 转义、迭代 30 限量版同 bug 复发）+ DC-9——双写/双拷贝共存一拍即
  有漏改面，"编译全绿、两边测试各自绿"仍漂移。

### 迁移策略建议（裁决 Q5）

- **R3.1 一次性切，禁双写对照期**——先例 `313375b` 即此策略（SoA→struct 对照表
  随 commit+金样逐字节）。双写对照=漂移温床（知识库双件实锤）；若某步必须过渡，
  对照面只读且过夜即删，禁止两个可写真相并存。
- **R3.2** 档 B 前置**字段唯一归属表**：每字段→唯一域（唯一可写点）；dump_scene/
  trace 适配层=只读旁路；禁双写、禁双读缓存。表随方案冻结、随域步进维护，
  与 restore_rules 的"双写点同一收口函数"纪律（refactor-plan 条件 3）同款。
- **R3.3** 红线补 harness 面条款（继承 refactor-plan 条件 6）："script.rs/
  camera.rs harness 面随生产面迁移同步或显式冻结（标注'非生产路径'）；新逻辑
  禁造第三面"；档 A script_ops.rs 归位=move-only，不得借归位重写 harness 面。

## 4. 验证矩阵完备性

**判定：🟡** —— 主判据三件方向正确，但方案红线计数失实、漏掉两个完备性守卫
（statediff 名集相等、行为面覆盖矩阵），域测边界未定义（确有沦为实现耦合测的
风险）。

### 已证实（A 级）

- **计数失实**：金样实数 = **56 场景**（state-dump-golden.json `scenarios` 实测
  len=56）；帧哈希夹具 = **31 个** `*.frames.txt`；方案"47+35"无出处可复现
  （蓝图 refactor-plan 写 44/44、行为矩阵写 55 场景、实数 56——三处三个数）。
- **主判据三件**（refactor-plan-review A1 换枪，refactor-behavior-matrix §0）：
  ① state_dump 金样逐字节（`tools/state_baseline.py --check`）② statediff
  **名集相等**（漏字段=红，2026-10-16 修复）③ 行为面覆盖矩阵（面→测试名，
  缺口显式标 ❌）。方案红线只写"金样 47+35 帧哈希+trace 逐字节"——**漏 ②③**，
  且"trace 逐字节"未指明对象（A==B==C 44/44 已降级附证，主判据是 Rust 侧
  state_dump 金样）。
- **域测边界（判定依据）**：域级独立单测只能钉"新模块自洽"；与 Java 的等价性
  **只有差分能判**（AGENTS.md §1"裁判是差分测试，不是'看起来对'"）。实现耦合
  测的判别法：断言能否指回 a.java 行号行为规则或 trace 观测值——指不回=白盒
  搬运实现细节，重构即碎、且给假信心（"域测全绿"≠零行为差）。

### 修订建议

- **R4.1** 域测边界规则入方案：域测只准断言**语义不变量**（可指回 a.java 行号
  锚/trace tick 值；script_golden 的 trace 锚定断言=合规范式）；禁断言内部布局/
  私有路径/字段序（归 line_order_freeze 与对照表管）。
- **R4.2** 完成判据分层写死："等价性=差分三件（金样逐字节/名集相等/帧哈希）+
  行为面矩阵无缺口；域测=分歧定位器（§4 三问定位层用），不进等价判据列"。
- **R4.3** 红线改写为可执行命令并引用实数："`python3 tools/state_baseline.py
  --check`（56 场景逐字节）+ `python3 gates/cli.py trace`（statediff 名集相等）+
  fixture_frames 帧哈希（31 夹具）+ 行为面矩阵复核（每域外移必须补行）"；
  禁裸数字，计数一律注明基线文件与复现命令（AGENTS.md §3 证据纪律）。

## 5. 回退策略

**判定：🔴** —— 方案缺回退单元/判据/分歧处置（用户预判成立："方案缺回退设计=
红"）。且这是对上位蓝图的**回退**：refactor-plan §5 执行表本有"回退=单 commit
revert"列，v1 把它丢了。

### 已证实（A 级）

- 方案 §2 仅"每步独立 commit+金样重跑"：无回退单元、无"金样重跑"的可执行命令、
  无中途语义分歧的处置路径。
- 蓝图对照：refactor-plan §5 表逐行带"回退 | 单 commit revert"——v1 未继承。
- 步 3（命令模式）实际已落（`8a455dd` 零漂移）；"中途发现语义分歧"是对未来
  script_ops.rs 归位步的同一道题，纪律同适用。

### 回退设计建议

- **R5.1 回退单元 = 整步 commit 的 revert**：步 3 归位中发现语义分歧（游标/16 op
  逐值）⇒ 整体 revert step-3 commit 至 step-2 全绿 checkpoint，**禁跨步攒补丁
  fix-forward**（双面漂移前科：修一处漏一面）；根因落前步（parse 口径）则逐步
  revert 3→2 至最近全绿点，按 §4 三问定位层后再动。
- **R5.2 checkpoint 判据写死**：每步完成 = commit + `tools/state_baseline.py
  --check` + `gates/cli.py trace` + 帧哈希**全绿输出摘要贴 commit message**
  （"未贴出的运行结果不存在"）；"金样重跑"替换为上述命令实名。大步（档 B 域
  外移）建议 worktree 隔离（using-git-worktrees 纪律），revert 粒度=步。

## 汇总裁定

**R0（方案级红，跨项）：§0/§3 问题基线陈旧。** 已证实：SoA→struct（`313375b`）、
ScriptOp parse/execute 分离（`8a455dd`）、restore_rules 表（scene.rs:685-748
`RESTORE_RULES` + scene_persist.rs:401 `apply_restore_rules`）**均已落地**；
§3 步 2-4 描述的是已完成工作，§0 的"425 行 switch/SoA 散列数组"是拆解前形态。
真实剩余工作 = **文件归位**（script_ops.rs / restore_rules.rs 建档 move-only +
entity 域操作补完）+ 档 B 域外移。方案须先重基线（§3 改写为"已落资产盘点+
剩余清单"）再执行，否则执行者会重做 RED 循环、或"重新拆分"churn 已冻结语义。

**计数**：攻击点 红 2（#3、#5）/ 黄 3（#1、#2、#4）/ 绿 0；+方案级红 1（R0）。
**修订 15 条**：R0 + R1.1-R1.3 + R2.1-R2.3 + R3.1-R3.3 + R4.1-R4.3 + R5.1-R5.2。

**给裁决人的一句话**：契约面（dump 适配+行序冻结）吸收力已被 SoA→struct 实证，
本方案真正的危险不在契约，而在 ①把已落地的工作当待办（R0）②真双面
（script.rs/camera.rs）不入红线（R3.3）③无回退单元（R5.1）。
