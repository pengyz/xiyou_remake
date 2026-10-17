# 对抗复核：dispatch-shape-survey.md（敌方视角·lead 拍板前最后一关）

> 复核人：interact-evidence（敌方）。对象：[dispatch-shape-survey.md](dispatch-shape-survey.md)
> （216 行，自称"2026-17 01:50 树"）。方法：逐面攻击 + 现场反证（git log/代码/台账/语料实测），
> 每攻击点 =「风险/反例/建议修正」。**总判定见文末。**
> 复核时间：2026-10-17 02:05-02:40。

## 总判定：**有条件通过**（架构方向对，但现状数字与表 schema 不可直接作实施依据）

条件清单（7 条，全部满足前不得开工）：见文末 §判定。核心三要害：
**① 现状描述与自称树基线矛盾（特例批已落，普查未反映）；② 表 schema 字段与语义类型不够装 4 特例
（含 flags[26] 双重角色与 BOSS 钩子层条件）；③ 表=代码还是配置未定义 ⇒ 金样冻结语义悬空。**

---

## A 面：现状数字可信度

### A1 现状描述与自称树基线矛盾（严重）
- **风险**：普查自称行号基准 = "2026-17 01:50 树"，但该树上**迁移步 1-3 的部分内容已经落地**：
  `efc6fe5`（10-17 **01:26**）"特例批收官——restore_rules 配置化（floor2/12/39 入表）"；
  `8a455dd`（10-17 **00:55**）"ScriptOp 命令模式——execute_script_instruction 16 op 拆 parse/execute"。
  普查仍写 "#9 恢复/存档特例=代码（restore 路径硬编码）"、"§3.2 恢复特例…入表（respawn_rule 字段）"、
  "§5 execute_script_instruction 425 行 **不拆**（task-22 已拍板）"。
- **反例**：scene.rs:630 `// ===== restore_rules（refactor-plan §4 特例批）：m_122 恢复分支尾的层特例 =====`、
  scene.rs:1452 `fn apply_restore_rules`、scene.rs:7733-7740 测试钉
  `floor2_restore_respawns_84_only_while_script26_pending`——84 respawn 特例已在 restore_rules 配置里。
- **建议修正**：重基于 `efc6fe5` 树重出 §1/§2/§3 的"现状形态"列（已落项标 DONE 并删迁移步重复内容）；
  工时估计按"剩余项"重估（否则步 1-3 双计）。**本条不满足，4-5 天估计作废。**

### A2 "改动点"口径未统一（严重）
- **风险**：三种口径混用——§0"4-7 处"（fixture×4 + 尺寸表 + sprite + …）、§1 表 11 行、
  §1 小结"4-5 行 fixture + 资源图，代码零改动"；"处"到底=文件？=逻辑编辑点？=行？
  测试/文档/夹具/金样全部未入计数（加一个新敌人若入图鉴/进地图场景，还牵图鉴文案、
  场景夹具、金样新增——这些是每敌必付的验证面成本）。
- **反例**：§1 表本身列了 11 个改动点（其中"不动"7 个），与"4-7 处"对不上任何一个口径；
  加新型实际面 = 4 族表 fixture + 1 尺寸表 + 1 sprite 资源 + category/image/anim/insert 散点
  （见 B1）+ 图鉴 + 验证面 ⇒ 无特例也 ≥8 逻辑编辑点。
- **建议修正**：定义口径 = "逻辑编辑点（文件级）"，主数与验证面（金样/单测/文档）**分列两栏**；
  "4-7/3-5"按新口径重算。

### A3 flags[26] 双重角色未建模（严重，与 B3 连带）
- **风险**：`respawn_rule: ScriptGated{flag: 26}` 把 flag **索引**当裸参数——但 flag26 同时是
  `scriptLineFlags[26]`（script 26 完成标记，行尾收尾门写入）；"script 号 == flag 号"是隐式约定，
  表里裸存 `flag: 26` 表达不了"该位由哪个脚本的完成态写入"，未来复用他位会静默错配。
- **反例**：84 respawn 真实规则 = **三要素**：spawn(84, 352, 128) 具体三参 + `!flags[26]` +
  空间守卫 `findBlockingEntity(11,4,84) < 0`（语义钉在 scene.rs:7733-7740 测试与 a.java
  m_122 恢复分支尾层特例）；`RespawnRule::ScriptGated{flag}` 签名只装下三要素之一。
- **建议修正**：要么 `respawn_rule` 重设计为 `{spawn: (type,x,y), when: ScriptUnvisited(ScriptId),
  and: SpatialGuard(...)` 语义类型簇；要么**删掉 enemy_defs.respawn_rule 字段**、承认
  restore_rules（已落地）是唯一宿主、表内只存 restore_rule 行引用——一事实一宿主。

### A4 save/restore 与表的同步问题未问（中）
- **风险**：迁移路径只字未提存档侧——RMS 快照存实体状态，表值改动后**旧档恢复按新表跑** =
  行为漂移面；且 restore_rules 本身已是配置，表改动与 restore_rules 的先后语义（谁覆盖谁）未定义。
- **反例**：BUG-012/restore 系 quirk 都是"D4 保真不许改"的值——一旦表成为"可改配置"，
  旧档语义与新表冲突时无裁定规则。
- **建议修正**：显式声明"Stage A 表值恒等于原版常量 ⇒ 旧档无影响"；定义表版本与存档版本策略
  （Stage B mod 表对旧档的行为 = 显式选择项）。

### A5 击杀钩子"旧值"三方不一致（中）
- **风险**：机械回填的前提 = "旧硬编码值可信"，但钩子号在三处说法不一：
  普查"69→(4)/70→(5)/67→(3)"；task-3 boss 取证"67→3、71→5"；代码实际结构 =
  `BOSS_TYPE_ORDER.iter().rposition(...)` **表索引间接** + 层条件特例
  （`73 ⇒ spawn_boss_event(6)` 仅 floor 40，`69 ⇒ spawn_boss_event(7)` 仅 floor 35，
  scene.rs:5177-5190、5258-5261）。
- **反例**：`hook_on_kill: Option<i32>` **装不下**"按 BOSS_TYPE_ORDER 位置取号 + floor==40/35 追加"。
- **建议修正**：回填前逐钩子从代码重新取证（产出钩子对照表带行号）；`hook_on_kill` 字段重设计为
  `hooks: Vec<{event: i32, when: Option<FloorEq>}>` 或显式引用 BOSS_TYPE_ORDER。

---

## B 面：表设计草案

### B1 EnemyDef 缺关键字段（严重）
- **风险**：分派**主轴** render_category 不在结构里（entity_category scene.rs:5909 硬编码路由
  战斗/渲染/交互三族臂）；图集映射（型→sheet，Java entityTypeImage 表）、blocksPlayerInsert/
  noInsertTypeFlags、animRowByType 真值、图鉴名全缺。`width_profile/anim_profile` 只存"档位引用"，
  真值仍在旧表/代码。
- **反例**：新型 id 落在 entity_category 的区间 match 之外（如 88+）⇒ 无臂命中——加一行表根本跑不起来。
- **建议修正**：补 `render_category / sprite_sheet / insert_block / anim_row / encyclopedia_name`
  字段；或显式定义"表覆盖域 = type_id 落在现有区间内，域外新型仍需改码"（把丑话写进承诺）。

### B2 14 类效果枚举不穷尽（严重）
- **反例 1**：**15 筋斗云不在枚举**——pickup 时 `if t == 15 { self.tutorial_seen = true; }`
  （scene.rs:5569-5571，f_bool_12）+ 入包，是"拾取置旗"类效果，14 类里没有；
  而且它就硬编码在 pickup 区间臂内部——§2"落入现有区间 ⇒ 不动"掩盖了这个内嵌特例。
- **反例 2**：16 与 17 共臂（§2 自己写了 `16|17`），§3.1 凿墙行只记"17 金勺子"。
- **反例 3**：29-32 药水数值是 **floorTier 缩放规则**不是纯参数（scene.rs:5580 注释：
  "29/30 floor≤10 +1 否则 +floorTier()；31/32 50/200×floorTier()…迭代 31 实测 +50 应为 +100"）——
  表达力要装"缩放公式"而非定值。
- **建议修正**：枚举补"拾取置旗"（15）与共臂标注（16|17）；HEAL/数值类字段带
  `scale: FloorTier | Fixed` 修饰。

### B3 特例 4 点入表后表达力不足（严重，A3/A5 同源）
- **风险**：respawn_rule（A3 三要素装不下）、hook_on_kill（A5 层条件装不下）、
  render_kind=MultiSegment{seg_table}（67 五段表进表——表中套表，evidence 锚怎么挂？）、
  battle_kind=68 bump 战态（战态机参数是什么？）都是"瘦枚举装厚语义"。
- **建议修正**：4 特例先各写一版**完整语义签名**再定表结构；装不下的显式归"第二层脚本/代码"
  并在表里留 reference 字段（一事实一宿主）。

### B4 字段耦合与倍率硬编码（中）
- **风险**：`Passive::GoldX2 / TraitX2{bits}` 把 **×2 倍率写死在枚举名**——倍率没参数化；
  "持有 23/24 且敌 trait 位匹配 ⇒ ×2"的**规则本体**仍在 combat.rs（表只参数化"谁触发"）。
  trait_bits（敌侧）与 effect/passive（物侧）的联动约定未写 schema 注释。
- **建议修正**：`Passive { kind, multiplier }`；联动规则文档化 + 单测钉规则（不只钉值）。

### B5 扩展天花板：DSL 绕表 = 两层脱节（严重）
- **风险**：script_ref 挂的 GUTS 脚本可做表看不到的一切（CES/DES/GIN/换层/传怪…）——
  表行不再描述行为，审表不可见行为；"1-2 行表"的真实适用域被悄悄换成"纯数值怪"。
- **反例**：普查自己承认纯数值路径"今天已接近目标态"（§1 小结）——表的核心增量只剩
  收敛 4 特例散点；把"新行为走 DSL"算进"表驱动收益"是口径通胀。且 **GUTS 无条件/循环/
  计时器/算术**（16 op 封闭，见语料普查 evidence-todos T-1）——"新机制默认走 DSL"只有
  **叙事/生成/走位/对话**类成立，数值规则、冷却、光环类新机制根本表达不了。
- **建议修正**：口径改为"**新内容**（场景/怪群/剧情/道具行为链）默认 DSL；**新机制**仍 Rust
  或第三层"；表行挂 script_ref 时强制附触发语义（B6），并在表 schema 文档写明"脚本内行为
  不受表审计约束"的边界。

### B6 script_ref 触发语义未定义（严重）
- **风险**：字段只有 `Option<ScriptId>`——on spawn？on bump？on kill？on tick？item 的
  activate→script 执行上下文是什么（GUTS 线在 mode11 对话状态机里跑）？
- **反例**：BUG-012 的 script0 重入环（case83 bump→loadLevelScript 重入死循环）就是
  "脚本触发语义没设计"的实弹——把 GUTS 线当函数挂进表，重入/中断/相位语义原样带入。
- **建议修正**：`script_ref` 拆 `script_trigger: OnSpawn|OnBump|OnKill|OnActivate + ScriptId`，
  并对每种 trigger 定义相位契约（是否允许重入、失败回滚）。

---

## C 面：三层边界

### C1 "90% 表覆盖"分母未定义（严重）
- **风险**：~90% 不可证伪——分母是未来新增？现有散点？语料场景？
- **建议修正**：给分母（例："task-2/3 取证的 7 道具 5 boss + Stage B 已知需求清单"逐项标注
  归层）或删百分比改"覆盖域清单"。

### C2 "第二层 DSL 零代码"口径 = 零编译期保障（严重）
- **风险**：GUTS 解析全在运行期（parseScriptInt 直接 Integer.parseInt，坏脚本=NumberFormatException/
  越界/死循环），无静态检查；"零 Rust 代码"实为"**零 Rust 编译期校验**"。
- **反例**：BUG-012 = 脚本语义坑的现成案例（重入环）；写错的用户脚本只会 runtime 炸。
- **建议修正**：DSL 静态校验器（op 词表/参数个数/脚本号引用存在/flag 号域）进 gates；
  表行 schema 校验同批。

### C3 第三层接缝 trait 形状锁死演进（中）
- **风险**：`ScriptRuntime::execute(op)->…` 以 **ScriptOp 为参数形** ⇒ 引擎只能当"现有 DSL
  的超集解释器"；真新机制要先扩 ScriptOp = 碰保真解释器（唯一红线面）——死锁：
  绕过 ScriptOp 就得改表结构，用 ScriptOp 就得碰保真体。
- **建议修正**：接缝取**语义动作层**（spawn/move/give/set_flag/change_floor…），
  op→动作的解析留在保真解释器内；接缝版本化随动作集演进。

---

## D 面：实施顺序与金样

### D1 表=代码还是配置未定义 ⇒ 金样冻结语义悬空（严重）
- **风险**：用户诉求"数据表配"= 表要**用户可改**；但金样 ratchet 冻结 55/44 场景行为——
  表改一个 HP 数字 = 金样红（这是预期行为变化还是回退？）。表若放 game-core const =
  改表=改代码重编译（诉求落空）；放 data/ = 大门禁语义未定义。
- **反例**：现有 fixture（enemybasehp.txt 等）已是 data——但它们的改动同样踩金样；
  普查未讨论这个已存在的张力。
- **建议修正**：**两份制**——保真表（原版常量镜像，进代码或 data 但金样锁死 + evidence 锚）
  与 mod 表（运行时叠放，金样显式豁免/单独基线）；金样冻结语义改写为"冻结=保真表逐字节"。

### D2 消费替换的验证面：等值单测与场景数口径（严重）
- **风险**：机械回填的"表==旧硬编码"等值单测若只测命中行，**default 兜底语义差**漏网
  （match default=0 vs 表查缺行=panic）；且场景数三口径打架：普查"44 场景闸"、
  completeness.md:749"44 场景"、scenarios/*.txt 实测 **45** 个、lead 口径"55 场景金样"。
- **建议修正**：等值单测含缺行/越界型/未知 type 三类负例；回归口径以 provision 清单钉死并
  写进 §6（数字不对齐，"0.5 天全量回归"就是空头支票）。

### D3 4-5 天估计无依据（中）
- **风险**：步 3"消费替换 ~1-2 天"要覆盖 ≥4 散点 × RED→GREEN × 每点全量回归
  （金样单跑实测 460s+，44/45 场景 × 每点 = 小时级墙钟）；步 5"0.5 天"未含回填错误返工。
- **建议修正**：按"点数 × 回归墙钟"重估（建议 ×1.5-2）；步 3 拆"每散点一步一闸"。

### D4 迁移路径与已落批竞速（=A1 连带）
- **建议修正**：迁移路径三步对 efc6fe5/8a455dd/d6a9cef 已完成项打勾后重排。

---

## E 面：保真红线（D4）

### E1 表值=原版常量的对照性断裂（严重）
- **风险**：现在硬编码常量带 a.java 行号注释（对照体纪律）；入表后行号锚消失，表值变成
  "没有出处的数字"；"表=可改配置"直接顶撞 D4"原版 bug 保真保留/只增不减"。
- **反例**：restore_rules/像素字面量（case 39）这类 D4 项一旦入表，"顺手修复"的诱惑没有
  任何机制拦阻（代码里至少有 review diff，表里是一行数字）。
- **建议修正**：表行必带 `evidence: a.java:行号`（+ BUG-xxx 引用）；门禁按 data/naming 体例
  校验"表值变更 ⇒ Evidence/Confidence 必填"；保真表数值变更须重过金样审批。

### E2 "可改配置 vs 不可改事实"张力未裁定（严重）
- **风险**：同 D1——若不裁定，Stage B 的"表化便利"会持续吸走 Stage A 的保真约束。
- **建议修正**：保真表/mutable mod 表两份制写进 §0 红线（第 4 条）；保真表的修改权 =
  仅 evidence 驱动的勘误（同证伪更正文化），mod 表不得进金样闸。

---

## 未命中面留痕（核过、未找到反例/仅轻微风险）

1. **§5 普查数据**（121 match/三形分布/厚度表）：抽查 paint_entities cat8 厚臂、activate_item
   10 臂、interact_with_cell cat 分派，量级与描述相符——**未发现反例**；轻微风险："python 扫描"
   脚本未附，数字不可复查，建议附扫描脚本或一句复现命令。
2. **状态机"不拆"建议**：与 gotcha（双移植面漂移/对照体保护）一致，**无反例**。
3. **红线 1/2（确定性/保真隔离）**：表述正确，第三层"不预接"克制得当，**无反例**。
4. **DC-9 删除（efc6fe5）**：与本方案"消灭散点"方向一致，**无反例**。
5. **拆分收益矩阵**（§5 表）：定性合理，**无反例**（前提是 D3 工时重估后重排优先级）。

---

## 判定

**有条件通过** ——三层架构（表/DSL/接缝）与项目纪律（确定性、保真隔离、双面教训）相容，
方向正确；但**现状数字、表 schema、金样语义三处不达标**，直接开工会把 A1/A2 的双计与
B1/B3 的表达力缺口带进实施。**条件清单**（全部满足方可开工）：

1. **重基线**：基于 `efc6fe5` 树重出"现状形态"列（restore_rules/ScriptOp 已落项标 DONE），
   迁移步重排，工时按剩余项重估；
2. **口径**：改动点 = 逻辑编辑点（文件级）+ 验证面分列；场景数 44/45/55 钉一口径；
3. **schema 补强**：EnemyDef 补 render_category/sprite_sheet/insert_block/anim_row/图鉴名；
   respawn_rule 语义化或删除归 restore_rules；hook_on_kill 装下 BOSS_TYPE_ORDER+层条件；
   script_ref 拆 trigger；Passive 倍率参数化；枚举补 15/16 与 floorTier 缩放；
4. **flags[26] 双重角色**显式建模（ScriptVisitedFlag(ScriptId) 语义类型，禁裸 flag 号）；
5. **表身份裁定**：保真表 vs mod 表两份制 + 金样冻结语义改写（冻结=保真表逐字节）；
6. **表行 evidence 列 + 门禁**（Evidence/Confidence 必填，data/naming 体例）；
7. **DSL 静态校验器**进 gates；"零 Rust 代码"口径改为"零 Rust 的叙事/生成/走位类内容"；
   第三层接缝改语义动作层。

> 全部条件落回 dispatch-shape-survey.md（或其实施计划）后，本复核自动转"通过"；
> 争议条目（如 B5 口径、D1 两份制）由用户/lead 裁定后留痕。
