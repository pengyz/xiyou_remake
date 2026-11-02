# 对抗复核 C：scene.rs 模块拆解方案 v1 —— 实施可行性攻击

> 角色：对抗复核员 C（实施可行性视角，remake-verifier 纪律）
> 对象：[docs/superpowers/plans/2026-11-02-scene-module-decomposition-plan.md](../../superpowers/plans/2026-11-02-scene-module-decomposition-plan.md)
> 日期：2026-11-02 ｜ 判定：**红**（执行序 5 步中 4 步已落地，方案把已付工作当待办；唯一未动的档 B 状态切片无估算）
> 修订计数：**9 条**（R1–R9，见 §6）；附加实测事故 1 起：金样基线当前红（4 新增场景未固化，R9）
> 实测口径：全部数据来自本日实跑（命令随条标注）；未跑的一律标"未实测"。

## 0. 结论速览

| # | 攻击点 | 判定 | 一句话 |
|---|---|---|---|
| 1 | 工作量/顺序 | **红** | §3 执行序步 0/2/3/4 **均已完成**（commit 实证）；真正未做的档 B 域状态切片被压缩成"步 5 一句话"，无估算 |
| 2 | 已落资产盘点 | **黄** | 六件资产完成度 90–100%，"保留+重归类"合理；但档 B 目标树与既有文件名（scene_paint/menu/menu_family）**对不上**，缺 menu 域 |
| 3 | 测试策略 | **黄** | 金样主判据被方案混谈（Java 47 场景≠Rust 拆解回归面）；"实现耦合测"红线可守，需补 3 条测试规矩；**实测金样基线当前红**（R9） |
| 4 | 并行/串行 | **黄** | 不必全仓冻结；需 scene.rs 写域所有权表 + 步级冻结窗（小时级）+ rebase 协议 |
| 5 | 完成判据 | **红** | 方案无验收定义；本补 6 条可测判据（含"新增道具不改 GameScene"grep 闸——**现状违反**） |

## 1. 工作量 / 顺序现实性（§3 执行序 5 步）

### 1.1 逐步实测：4/5 步已付讫

`git log` 实证（A 级证据，命令：`git log --oneline --grep=...` + `wc -l`）：

| §3 步 | 方案表述 | 实际状态 | 证据（commit / 行数） |
|---|---|---|---|
| 步 0 | "（已落）：trace_snapshot 映射层" | **已落，但名字不对**：不存在 `TraceState`/`trace_snapshot` 类型（`grep -rn "TraceState\|trace_snapshot" crates/` = 0 命中）；实体是 game-oracle `state_dump.rs`（142 行）逐字段映射+行序冻结，契约是**隐式 push 序**而非类型 | `crates/game-oracle/src/state_dump.rs:101-137`；`tools/state_baseline.py` A2 行序冻结声明 |
| 步 2 | "Entity 聚合补完（SoA→struct 对照表随 commit）" | **完成**。SoA→struct 全量迁移（Entity 11 字段 + 427 站点，金样零漂移）+ 六写点收口 + 层存档表操作并入。scene.rs 残留 `entityType[]` 类字样仅 5 处**注释**（entityTypeImage=类型→精灵映射，另一概念）；余量≈0 | `313375b`（10-16 全量迁移）、`76cb6c2`（六写点收口）、`5894bfb`（文件合流）；`wc -l entity.rs`=621 |
| 步 3 | "ScriptOp 命令模式（parse/execute 分离，游标 RED 先行）" | **完成**。16 op 已拆 parse_op/execute_op（ScriptOp::Tak/Lay/Gin…，游标随变体携带，UNKNOWN→None 停摆）；游标 RED 测（MOV-miss 尾递归钉）+ 双面漂移防线已建 | `8a455dd`（ScriptOp 命令模式 金样零漂移）、`d6a9cef`（第 2 步前置两测）；`scene_script.rs:14-41,192` |
| 步 4 | "restore_rules 表驱动" | **完成**。RestoreAction 枚举（SpawnIfAbsent/ParamRewrite/SwapSpawn）+ RESTORE_RULES 表（floor 2/12/39）+ DOOR13_HI + apply_restore_rules | `efc6fe5`（特例批收官 floor2/12/39 入表+DC-9 死桥删除）；`scene.rs:685-750`、`scene_persist.rs:401` |
| 步 5 | "（档 B 追加）combat/persist/render_adapter 逐域外移+接口定义" | **唯一未启动**，且是全程最大头 | 见 1.3 |

**文件搬家也是 5 步不是 4 步**：今天（11-02）task-15 步①–⑤ 连落（tests/paint/menu/persist/script，全部 move-only），
scene.rs 5755→**5526 行**（实测 `wc -l`）+ scene_script.rs 242 行。方案 §4 "已落 4 步"清单漏了步⑤ scene_script.rs。

### 1.2 顺序对不对

风险序（Entity 数据→命令模式→特例表→域切片）与 refactor-plan 一致，**方向没错**；
但作为"执行序"已失去意义——它列的是**已完工清单**。真实剩余序应为：

1. 先裁决 §5.1 深度（档 A/档 B/渐进）——完成判据依深度而变（见 §5）；
2. 裁决 combat 切分线、menu 域归属（档 B 树里根本没有 menu 味的位置，见 §2.3）；
3. 再排档 B 状态切片迁移（真·剩余工作）。

### 1.3 哪步被低估：步 5（档 B 状态切片）

"逐域外移+接口定义"一笔带过，掩盖了三种完全不同的工作：

- **现状**：5 个 scene_*.rs 都是 `impl GameScene` 块搬家（move-only）——文件变了，域没变；
  GameScene 仍是中央状态体（scene.rs 151 个 fn，含 op_* 16 臂、item 簇 `activate_item/use_item_stack/
  item_desc*`、battle `tick_battle/predict_hp_loss`、菜单/相机/相机/overlay）；**GameScene 177 个 pub 字段**
  （awk 区间计数实测）——档 B"各域自有状态切片"要动的就是这 177 个字段的归属。
- **档 B 要求**："各域自有状态切片（域间经接口/命令交互）"= 把 GameScene 字段搬进域 struct + 改调用约定。
  参照系：SoA→struct 单字段面迁移就花了 427 站点（313375b），状态切片是**多字段面×域间接口**，量级同或更大。
- **est.**：refactor-plan §6 对已付部分估 3-4 天、§7 修正 5-7 天；剩余档 B 我估 **8-15 天**（combat/persist/
  render/menu 四域×状态切片+接口+域级测试回指），档 A 残余（把 op_* 实现从 scene.rs 挪进 script_ops、
  Entity 补文档）**1-2 天**。方案对步 5 给的是 0 天——这是本次攻击的最大工作量失真。

### 1.4 "Entity 聚合补完"余量估计（点名项）

grep 实测：`crates/game-core/src/scene.rs` 无 SoA 平行数组残留（`entityType|entityPixel|entityState[|`
等 5 命中全为注释）；EntityTable/CellGrid 已是域类型；state_dump 已改走 struct 闭包。
**余量 = 0 行代码**；只剩方案要求的"对照表随 commit"——该表已存在于 refactor-plan §2（11 字段对照），
补个链接即可。步 2 应从执行序删除或改标"已付讫·仅补文档"。

## 2. 已落资产盘点（逐项实测完成度 + 处置提议裁定）

### 2.1 六件资产台账

| 资产 | 规模（wc -l） | 落地 commit | 完成度 | 方案处置（§4 保留+重归类）裁定 |
|---|---|---|---|---|
| scene_tests/（12 文件） | 6099 | `f419b68` 步① move-only 6123↔6123 | 100% | **合理**。但注意：测试内 178 处直读点（entities.items/cell_type/.count，grep 计数）分布于 5 文件，slot_equip(2112)+t2_walkto(2678) 两块是重仓——**档 B 状态切片时这些测试要重指到域接口**，回指成本未入任何估算 |
| scene_paint.rs（paint* 15 簇） | 895 | `78822d4` 步② | 100%（搬完） | **合理但与档 B 冲突**：档 B 又要把 paint* 并入 render_adapter.rs = **第二次搬家**。且仓库已有 paint.rs（BootMachine 等）+ render.rs——三个渲染名目再添一个必乱。**修订：一步到位定终名**（建议 scene_paint.rs 直接改名 render_adapter.rs 或就地更名注记），禁止"搬完再搬" |
| scene_menu.rs | 976 | `bf780ac` 步③ | 100%（搬完） | **合理，但目标树缺口**：档 A/B 的目标架构**没有 menu 域**；且 menu_family.rs（2128 行，modes 8/15/16/17/22）已独立存在。两 menu 文件+title.rs/intro.rs/logo_anim.rs 的归属必须在拆解前裁决，否则"拆完了"没有收敛点 |
| scene_persist.rs（装载/存档/换层） | 599 | `838f5fb` 步④ | 100%（搬完）≈档 B persist 域的外移部分 | **合理**。剩"状态切片+接口"即档 B persist 的全部实质工作 |
| scene_script.rs（16 op） | 242 | `5111171` 步⑤（**方案漏列**） | 搬完+命令模式已落（8a455dd） | **方案 §4 清单过期**，补第 5 件。op_* 执行体仍在 scene.rs（16 臂）——档 A script_ops 的剩余=把执行体挪出 |
| Entity/EntityTable/CellGrid | 621 | 313375b/76cb6c2/5894bfb | ~100% | 表驱动与六写点收口已入；cellSlotTable 独立红线（方案 §2）与 CellGrid 现状一致 ✓ |
| trace 契约 | 142 | state_baseline 基线 `data/status/baselines/state-dump-golden.json` | 100%（隐式契约） | 见 1.1：不叫 TraceState。**修订：补一行"契约=dump_scene push 序（A2 冻结）"的正式命名**，否则"TraceState 契约冻结"是空头支票（无类型可冻结） |
| restore_rules | scene.rs:685-750 | `efc6fe5` | 100% | 方案步 4 该删（已付讫） |

### 2.2 拆解与已落资产会不会打架

- move-only 纪律与域化**不冲突**（金样零漂移的搬运可以随时做）；冲突在**重复搬家**：
  scene_paint→render_adapter、scene_persist→persist 改名、scene_script→script_ops 改名。
  每改一次名，测试/文档/知识库指针全要跟着走。**修订：拆解方案先出"终名表"（现名→终名→唯一一次搬运）**。
- `impl GameScene` 块搬家 ≠ 真模块：域化后这些 fn 的 `&mut self` 要改成域方法+参数面，
  等于把 6099 行测试+151 个 fn 的调用约定再动一遍——这是"拆解会不会和已落冲突"的真答案：
  **行为不冲突，签名大面积重排**，测试回指是主要成本。

### 2.3 档 B 目标树缺口（攻击 §1 档 B）

`combat.rs`（PlayerCombat）、`paint.rs`、`menu_family.rs`、`title.rs`、`intro.rs`、`logo_anim.rs`、
`popup.rs`、`tap_walk.rs`、`walk.rs`、`undo_journal.rs`……已有一票域文件；档 B 树只新增 6 个名字，
既不收编它们也不画边界。**"拆完了长什么样"必须先回答：既有 25 个 game-core 文件与 6 域的映射关系**，
否则完成判据无从谈起（见 §5）。

## 3. 测试策略与验证预算

### 3.1 先纠一个主判据混谈（verifier 纪律红线）

方案 §2 红线写"金样 47 场景+35 帧哈希+trace 逐字节"。实测拆开是**两套不同裁判**：

| 裁判 | 命令 | 实际验证对象 | 对拆解回归的效力 |
|---|---|---|---|
| Java 47 场景 A==B==C | `python3 reference/oracle/run.py --scenarios`（47 个 *.txt，实测计数；32 preset/5 md 不算） | **纯 Java 三方**（A 原始字节码/B 重映射/C javac），`run.py` 不跑 Rust（refactor-plan §7.1 早有更正） | **间接**：只保证 oracle 侧不漂，抓不住 Rust 拆解错 |
| state_dump 金样 | `python3 tools/state_baseline.py --check`（重跑 game-oracle 三重放面 fixture_frames/preset_cases/gameplay_floor1，逐字节对 `data/status/baselines/state-dump-golden.json`） | **Rust 侧逐 tick 状态金样**（A2 行序冻结） | **主判据**——拆解步骤真正要跑的 |
| game-oracle 差分 | `cargo test -p game-oracle`（gate_trace） | 参考版 vs Rust 逐 tick | 主判据之二 |
| 单元 | `cargo test -p game-core`（实测 321 测试 13.30s 全绿） | 域行为 | 内层 |

**修订 R3**：红线措辞改为"state_dump 金样 + game-oracle 差分为主判据，Java 47 场景为 oracle 附证"
（refactor-plan §7.1"主判据换枪"决议在方案 v1 里被回退了——这是裁判口径回退，须改）。
另一实测缺口：**`state_baseline.py --check` 没挂进 gates/cli.py**（仅记录在金样 manifest 的 probe 字段），
"每步金样重跑"目前靠人肉自觉——**修订 R4：挂 gate 或写进每步 commit 检查单**。

### 3.2 域级独立单测怎么不制造"实现耦合测"（用户红线）

现状风险实测：scene_tests 178 处直读 `entities.items/.cell_type/.count`；
refactor-plan §7.6 早列过"测试内 SoA 直读点清单（probe_floor2 手算 cellTypeGrid sha 等）"。
拆域后**测字段布局**的用例会成片变红（行为没变、布局变了）——这就是实现耦合测。三条规矩（建议入方案红线）：

1. **域单测只断言域 API 的可观测结果**（"对实体 X 执行 op_mov 后 px==N"），禁止断言私有布局/调用序列/中间缓存；
2. **布局敏感的唯一合法面 = state_dump 金样**（契约层），其余测试借用它当出口，不再各测各的布局；
3. **判定法**：一个行为保持的重构让某用例变红 ⇒ 该用例=实现耦合测，改写为结果断言，禁止改断言迁就实现。

成本侧（实测）：`cargo test -p game-core` 321 测试 13.3s（含编译 44.9s 冷启）——内层验证便宜，
每 commit 都跑可行；金样/场景套件贵（§3.3），按步跑不按 commit 跑。

### 3.3 验证预算实测（2026-11-02 实跑）

| 项 | 命令 | 实测耗时 | 结果 |
|---|---|---|---|
| 金样 47 场景（Java 三方） | `time python3 reference/oracle/run.py --scenarios` | **1082.53 s ≈ 18.0 min** | PASS 47/47（sha 三方全同） |
| Rust 主判据金样 | `time python3 tools/state_baseline.py --check` | **239.77 s ≈ 4.0 min**（含 release 编译，温跑更短） | **RED**：`+新增 fx-boss68b / fx-chang-e-floor1-taps / fx-item20b / fx-record-arrows` 4 场景未入基线（见下） |
| 单元 | `cargo test -p game-core` | 13.3 s（321 测试） | 全绿 |

**预算结论**：每步金样重跑若指 state_baseline --check（4 min）⇒ 剩余 ~10 步全程 ~40 min，完全可行；
若指 47 场景套件（18 min/次）⇒ 光验证 ~3 h 起，挤压实现时间。**修订 R5 定稿**：每 commit=单测（13 s 级）；
每步=state_baseline --check；步末/里程碑=47 场景 + `gates all`。

**实测抓到的新问题（R9）**：state_baseline --check 当前**红**——fixture_frames.rs 新增 4 个夹具
（fx-boss68b:829 / fx-item20b:743 / fx-record-arrows:360 / fx-chang-e-floor1-taps:1377）未跟金样固化 commit
（基线最后动账 07d1ac3 只加了 story-fullrun）。这不是行为漂移（属"+新增"），是**基线纪律滑坡**：
--check 未挂 gate（R4）⇒ 新增测试可以不固化金样就落库。拆解动手前必须先收口（重固化+单独 commit 说明，
沿 07d1ac3 先例），否则"每步金样重跑"没有绿起点，第一条红线就无从对照。

## 4. 并行 / 串行与写域冲突预案

**结论：不必全仓冻结**。拆解写域 = scene.rs + scene_*.rs + entity.rs（game-core 内）；
实测近期 feature 流量主要在 game-desktop/shell（ebe041a 标题态侧功能钮）、docs/kb、android——与拆解写域天然错开。

冲突预案（建议入方案 §3 附录）：

1. **写域所有权表**：拆解分支独占 `crates/game-core/src/scene*.rs` + `entity.rs`；
   feature 分支禁改上述文件（要改就排队：等当前步 commit 落地后 rebase 再改）。
2. **步级冻结窗**：每步是小时级 move-only/签名重排，冻结窗=该步开工到 commit；
   不存在"拆解期全仓禁 feature"的必要。
3. **rebase 协议**：feature 分支每步 commit 后立即 rebase（move-only 用 `git log --follow`/
   rename 检测消化）；冲突高发点=scene.rs 的 press_key/handle_field_input/tick（输入与状态机，
   dual-control 系列计划与拆解重叠最深）。
4. **长 feature 走 worktree**（git worktree 隔离），合入前以 state_baseline --check 对账。
5. **红线复述**：任何冲突解决禁止以"顺手改行为"收场（D4/零行为差）；冲突解决后金样必须重跑。

## 5. 完成判据（补写：可维护性验收定义）

方案 v1 没有"拆完了长什么样"。补 6 条**可测**判据（依 §5.1 深度裁决取档 A/档 B 子集）：

| # | 判据 | 可测断言（怎么验） | 现状（实测） |
|---|---|---|---|
| C1 | 行为零差 | `state_baseline --check` 金样逐字节 + `cargo test -p game-oracle` 全绿 + 47 场景 PASS（附证）——**贴实际输出** | 基线在；--check 未挂 gate（R4） |
| C2 | **新增道具不再改 GameScene** | grep 闸：`scene.rs`（协调器）中 `ItemKind::`/`item_\d`/道具名 match 臂计数 **== 0**，入 gates 只降不升 ratchet | **现状违反**：activate_item/use_item_stack/item_desc_index/item_description 等在 scene.rs（:2777,2998,3024,4936）——判据从违反→清零即验收 |
| C3 | 新增 op 不改执行面 | `parse_op` 新增 op ⇒ 只触 scene_script（parse）+script_ops（execute 臂）；scene.rs `op_*` 计数 ratchet 清零 | op_* 16 臂还在 scene.rs（步 5 余量） |
| C4 | 特例配置化 | floor 特例字面量（`floor == 2/12/39` switch）在 scene.rs 计数 == 0，全部走 RESTORE_RULES | **已达标**（efc6fe5）——保留为回归断言 |
| C5 | GameScene 减薄 | scene.rs 行数 ratchet：5526 → 目标（档 A ≤4000 / 档 B ≤1500，"只持域实例"字段数 ≤6） | 5526 行 / 151 fn / **GameScene 177 个 pub 字段**（awk 实测）——状态切片的真实工作量标尺 |
| C6 | 域测独立性 | 每域 ≥1 个**不构造 GameScene** 的单测文件（script_ops 可对 parse_op 纯函数直测）；scene_tests 直读点 178 → 只经域 API | 0 个（scene_tests 全部构造 GameScene） |
| C7 | 验收演练 | 演练"新增一件 dummy 道具 + 一个 dummy op + 一个 dummy 层特例"，diff 触达文件清单不含协调器（C2/C3/C4 的活体验收） | 未做 |

**注意**：C2 是用户可感知的"拆完了"定义——"新增道具不再改 GameScene"当前**不成立**（scene.rs 仍持道具逻辑），
这正是档 B/档 A 深度裁决的实质差别：档 A 不含 item 域 ⇒ C2 在档 A 下只能部分达成。
**修订 R6：深度裁决（§5.1）必须和完成判据绑定**，否则"拆完了"在两个深度下是两个答案。

## 6. 修订汇总（8 条）

| # | 严重度 | 修订建议 | 证据 |
|---|---|---|---|
| R1 | **高** | §3 执行序重写：删/标已付讫步 0/2/3/4（各配 commit 号），剩余序 = 深度裁决→边界裁决→档 B 状态切片（估 8-15 天）/档 A 残余（1-2 天） | 1.1 表 |
| R2 | **高** | 补"终名表"（现名→终名，每文件只搬一次）：scene_paint→render_adapter 或就地定名；scene_menu/menu_family/title/intro 归属裁决进 §5 待裁问题 | 2.2/2.3 |
| R3 | **高** | 主判据换枪回归：红线改为 state_dump 金样+game-oracle 差分为主判据，Java 47 场景=oracle 附证（refactor-plan §7.1 决议不得回退） | 3.1 |
| R4 | 中 | `state_baseline.py --check` 挂 gates（或强制入每步 commit 检查单） | 3.1；grep gates/cli.py 无引用 |
| R5 | 中 | 验证预算分层（实测定稿）：每 commit=单测 13s；每步=state_baseline --check 4min（~10 步合计 ~40min）；步末/里程碑=47 场景 18min+`gates all`——"每步金样重跑"明确指前者 | 3.3 实测表 |
| R6 | **高** | §5.1 深度裁决与完成判据绑定：档 A 不能承诺 C2（道具不改 GameScene）；承诺什么写什么 | §5 C2/C6 |
| R7 | 中 | 测试三规矩入红线（结果断言/布局只留金样面/耦合测判定法）；档 B 切片预算补 scene_tests 178 直读点回指成本（slot_equip 2112+t2_walkto 2678 行重仓） | 3.2 |
| R8 | 低 | "TraceState"改名或正式立类型：现状契约=dump_scene push 序（A2），方案"TraceState 契约冻结"无类型可依 | 1.1 |
| R9 | **高** | **动手前置**：金样基线当前红（fixture_frames 4 新增场景未固化）——先重固化+单独 commit 说明（07d1ac3 先例），拆解才有绿起点 | 3.3 实测 `state_baseline.py --check` 输出 |

### 遗留假设（C 级，未证实）

- 档 B 8-15 天估算基于 SoA→struct 427 站点迁移的类比外推（B/C 级之间）——切片实际点数未逐一清点。
- feature 流量与拆解写域"天然错开"基于近 30 commit 观察，dual-control 后续计划若落 scene.rs 输入层会提高冲突面。

### 实测附录（命令与输出摘要，2026-11-02）

```
wc -l scene.rs=5526 entity.rs=621 scene_menu.rs=976 scene_paint.rs=895
      scene_persist.rs=599 scene_script.rs=242 scene_tests/*=6099
GameScene pub 字段 = 177（awk '/pub struct GameScene/,/^}/' 计数）
ls reference/oracle/scenarios/*.txt = 47（32 preset/5 md 另计）
cargo test -p game-core: 321 tests 319 pass 2 ignore, 13.30s（含编译 44.9s）
scene_tests 内部直读点（entities.items|cell_type|.count）= 178 处 / 5 文件
grep TraceState|trace_snapshot crates/ = 0 命中（契约=state_dump.rs push 序）
time run.py --scenarios: ELAPSED 1082.53s ≈ 18.0 min，PASS 47/47
time state_baseline.py --check: ELAPSED 239.77s ≈ 4.0 min，RED
  （+新增 fx-boss68b / fx-chang-e-floor1-taps / fx-item20b / fx-record-arrows 未入基线）
```
