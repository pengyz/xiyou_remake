# 分派形态普查 v2：加敌人/加物品的成本解剖与三层效果架构（修订版）

> 【2026-10-17 v2 修订】本版按对抗复核 [dispatch-shape-review.md](dispatch-shape-review.md)
> 7 条件逐条吸收（有条件通过 → 条件全部落回）。v1 主张的更正在正文原地标注（【v2 更正】），
> 历史主张保留不删。**基线 = `efc6fe5` 后树**（v1 自称"01:50 树"但该树已含迁移批产物——A1 伤，
> 现行号以函数锚为主）。纯普查未改任何代码。
>
> v2 修订记录：① A1 重基线（restore_rules/ScriptOp/DC-9 已落项标 DONE，工时按剩余项重估）
> ② A2 口径统一（"一处"=逻辑编辑点，验证面分列）③ A3/A5 schema 修正（respawn 归 restore_rules
> 唯一宿主；钩子=表索引+层条件的 hook 表）④ B1 EnemyDef 字段补全 ⑤ B2 枚举补 15/16/29-32
> ⑥ B5 口径修正（新内容 DSL/新机制 Rust）⑦ B6 script_trigger 四触点 ⑧ C1 覆盖域清单替代百分比、
> C2 DSL 静态校验器进 gates、C3 接缝改语义动作层 ⑨ D1+E2 保真表/mod 表两份制、E1 evidence 列
> +门禁、D2 四口径归源、D3 墙钟重估。

## 0. 结论（先读这节）——三层效果架构 + 两份制红线

**用户核心诉求：加一种敌人/一个物品 = 数据表配，不动代码。** 机制类处置三层：

| 层 | 承载 | 覆盖域（C1 可证伪口径，见 §3.3） | 新增成本 |
|---|---|---|---|
| **第一层·表参数** | `enemy_defs` / `item_defs`（§4，**保真表/mod 表两份制**） | 现有 17 类效果枚举 × 现有行为维度组合（§3 逐项清单为分母） | **纯表加行**（含 evidence 列） |
| **第二层·DSL 挂脚本**（新**内容**默认手段） | GUTS 16 op（`OPCODES` game-data/script.rs:25）+ `ScriptOp` 23 变体（scene.rs:699） | **叙事/生成/走位/对话类**：场景、怪群行为、剧情链、道具行为链 | 表行 `script_trigger`+`script_ref` + DSL 脚本一段 |
| **第三层·外部引擎接缝**（可选后期） | 语义动作层 trait `ScriptRuntime`（C3，见 §4.4） | DSL 表现力实证不足时 | 引擎实现+动作集扩展 |

**【v2 更正】B5 口径**：v1 写"新机制默认走 DSL"——错。GUTS **无条件/循环/计时器/算术**
（16 op 封闭），数值规则/冷却/光环类表达不了。正确口径：**新内容（场景/怪群/剧情/行为链）
默认 DSL；新机制（新效果语义/数值规则）仍 Rust 或第三层**。表行挂 script_ref 时脚本内行为
不受表审计约束——该边界写进表 schema 文档（B5）。

**现状 vs 目标态（A2 口径：一处 = 逻辑编辑点〔文件级〕，验证面分列）**：

| | 现状（efc6fe5 树实测） | 目标态 |
|---|---|---|
| 加第 N 个敌人（机制复用） | **逻辑编辑点 8-10**（§1 重数）+ 验证面（图鉴文案 1 + 场景/夹具 1 + 金样 1） | 第一层 **1 行表 + 资源 1 行 + evidence 列**；行为扩展走第二层 |
| 加第 N 个物品（效果类复用） | **逻辑编辑点 4-6**（§2 重数）+ 验证面（文案 1 + 金样 1） | 第一层 **1 行表 + 文案 1 行** |

**【v2 新增】红线第 4 条（D1+E2 裁定）——保真表 / mod 表两份制**：
- **保真表** = 原版字节码常量镜像（族表数值/trait 位/钩子号/restore 规则/D4 quirk）——
  **不可改**（仅 evidence 驱动的勘误，同证伪更正文化）、每行带 `evidence: a.java:行号`（E1）、
  **金样冻结语义 = 保真表逐字节**（`state-dump-golden.json` 55 场景锁死）；
- **mod 表** = 用户扩展层（Stage B 新增≠原版）——可改、**不进金样闸**（独立基线或豁免）；
- 修改权裁定：保真表改动 = 勘误流程（Evidence/Confidence 必填，门禁按 data/naming 体例）；
  mod 表 = 用户自由。**旧档语义**（A4）：Stage A 保真表值恒等于原版常量 ⇒ 旧档无影响；
  mod 表对旧档的行为 = Stage B 显式选择项。

其余三条红线沿 v1：① game-core 确定性红线（脚本引擎也须确定性）② 保真核心与扩展缝隔离
③ 双解释器教训 → 第三层落地时"第三解释器"漂移面进 gotcha 防线。

---

## 1. 敌人新增成本解剖（A1 重基于 efc6fe5 树）

**【v2 更正】**：v1 "#9 恢复/存档特例=代码（restore 路径硬编码）"、"respawn_rule 待入表"系
双计——`efc6fe5`（10-17 01:26）**restore_rules 已配置化**（scene.rs:630-700：
`RestoreAction` 三形态 SpawnIfAbsent/ParamRewrite/SwapSpawn + `RESTORE_RULES` floor 2/12/39
入表，84 respawn 特例 = floor2 规则行 `SpawnIfAbsent{flag:26, find:(11,4,84), spawn:(84,352,128,0)}`）。
同批 `8a455dd` ScriptOp 命令模式已落（parse_op/execute_op 拆分）、`efc6fe5` DC-9 死桥已删。

以"加新敌人型 N（机制复用）"为想定，逐处列改动点（**A2 口径：逻辑编辑点；DONE=已落项**）：

| # | 改动点 | 现状形态（efc6fe5 树） | 逻辑编辑点 | 机制复用时 | 有特例时 |
|---|---|---|---|---|---|
| 1 | 族表 HP/ATK/DEF/GOLD | `fixture!` 4 件（main.rs:458-465） | 4 | 加行 ×4 | 同左 |
| 2 | sprite 尺寸 | `fixture!("type_width_px.txt")` | 1 | 加行 | 同左 |
| 3 | sprite 图集/动画 | jar 资源 + `dispatch_sprite_size`/`dispatch_anim`（entity.rs match） | 1（资源）+0-1（代码） | 资源加图 | 新 profile ⇒ 加臂 |
| 4 | **render_category 主轴**（B1） | `entity_category`（scene.rs:5909）= floor 特例（72@50）→ `entity::render_category` **区间 match**——**分派主轴**，新型落区间外（88+）无臂命中 | 0（区间内） | 0 | 域外新型 ⇒ **改区间 match** |
| 5 | **trait 位** | `trait_flags` match（`49\|53\|74=>1, 69=>2`） | 0-1 | 0 | 加型号行 |
| 6 | 战斗臂 | interact_with_cell cat8 共享臂（predict/tick_battle 复用） | 0 | **0** | cat8 臂内 if 特例 |
| 7 | **击杀钩子**（A5 重取证） | **表索引间接**：`BOSS_TYPE_ORDER=[75,74,68,67,69,71]`（scene.rs:128），kill 处 `rposition` 取位 → `spawn_boss_event(bi)`（:5179-5181）；**层条件特例**：`73⇒event(6)` 仅 floor40（:5189）、`69⇒event(7)` 仅 floor35（:5260） | 0-1 | 0 | 改钩子表（§4.2） |
| 8 | 恢复特例 | **DONE**（restore_rules 已入表，见上更正） | 0 | 0 | 加 restore_rules 行 |
| 9 | 渲染特例 | paint_entities cat8 臂 93 行（型67 五段 `SEG67_PIECES` 表已常量/83 剧情点混入） | 0 | 0 | cat8 臂内特例 |
| 10 | 图鉴名/文案 | build_encyclopedia 文案 | 1 | 加行 | 同左 |

**重数（A2 口径）**：机制复用新敌人 = **8-10 逻辑编辑点**（族表 4 + 尺寸 1 + 资源 1 + 图鉴 1
+ 若 category 域外/新 profile/钩子/特例则 +1 各）——**【v2 更正】v1"4-7 处"按文件级口径低估**
（漏 render_category 图集/insert/anim/图鉴散点，见 B1 反例）。验证面分列：金样 1（state-dump
55 场景基线）+ 场景/夹具 1 + 单测若干。

**痛点重述**：纯数值新敌人的代码面已接近目标（族表/尺寸已 fixture 化、restore 已表化、
战斗/渲染 cat8 共享）；剩余硬编码 = **trait_flags / 钩子层条件 / 渲染特例 / render_category 域外**
4 类。

## 2. 物品新增成本解剖（A1 重基线）

| # | 改动点 | 现状形态 | 逻辑编辑点 | 复用效果类时 | 新效果语义时 |
|---|---|---|---|---|---|
| 1 | 拾取分派 | `pickup_item_type` 11 臂区间——**【v2 更正 B2】区间臂内藏特例**：15 筋斗云入包同时 `self.tutorial_seen = true`（scene.rs:5570，f_bool_12 置旗） | 0-1 | 落区间 ⇒ 0 | 加臂 |
| 2 | 使用效果臂 | `activate_item` 10 臂（`16\|17` **共臂**〔v1 漏标〕、20 臂 28 行内 floor40 禁用特例） | 1 | 加分支/新臂 | 加臂 |
| 3 | 被动散点 | 25 金翻倍 ×2（`find_item(25)` 掉落点）；23/24 trait 联动（combat.rs 规则本体） | 0-1/点 | 新被动 ⇒ 加散点 | 同左 |
| 4 | 数值规则（B2 补） | **29-32 药水=floorTier 缩放规则型**（scene.rs:5576-5590：29/30 `floor≤10 ? +1 : +floorTier()`；31/32 `50/200 × floorTier()`）——非定值 | 0 | 入表需 `scale: FloorTier\|Fixed` | 规则型 |
| 5 | 文案表 | `item_description` D[22] const（含使用次数文案） | 1 | 加行 | 同左 |
| 6 | 文案索引 | `item_desc_index`（`85→20, 86→21, t-13` 映射硬编码） | 1 | 改映射 | 同左 |

**重数**：复用效果类 = **4-6 逻辑编辑点** + 验证面（文案/金样）。痛点：#1 区间臂内嵌特例、
#2 每道具一分支、#3 被动散点、#4 规则型数值、#5/#6 文案硬编码。

## 3. 效果类型穷举（B2 补全 + C1 覆盖域清单）

### 3.1 物品效果（17 类，补 v1 遗漏 3 处）

| effect 语义 | 代表 | 参数化形态 | 归层 |
|---|---|---|---|
| 状态显示 | 13 | REVEAL（无参） | 一 |
| 记录浏览 | 14 | PANEL{mode} | 一 |
| 凿墙（**16\|17 共臂**） | 16/17 | REMOVE_FACING_TYPE{target:11} | 一 |
| 全清墙 | 18 | REMOVE_ALL_TYPE{11} | 一 |
| 回血 | 19 | HEAL{value, scale: Fixed} | 一 |
| 瞬移 | 20 | TELEPORT_MIRROR{floor_ban:[40]} | 一 |
| 楼梯 | 21/22 | STAIR{±1} | 一 |
| 全屏攻击 | 86 | KILL_ADJACENT_ALL | 一 |
| 开门特权 | 85 | OPEN_ALL_TYPE{门型} | 一 |
| **拾取置旗（v1 漏）** | 15 | SET_FLAG{tutorial_seen}（pickup 内嵌） | 一 |
| **数值规则（v1 漏）** | 29-32 | STAT_DELTA{stat, base, scale: FloorTier\|Fixed} | 一 |
| 被动金加倍 | 25 | Passive{kind:GoldX, multiplier}（B4：**倍率参数化**，不写死 ×2） | 一 |
| 被动攻加倍 | 23/24 | Passive{kind:TraitX, multiplier, trait_bits 联动} | 一 |
| 药水即时 | 29-32 | 同 STAT_DELTA（pickup 触发） | 一 |
| 装备档位 | 33-40/79/80 | EQUIP{tier} | 一 |
| 钥匙 | 26-28 | KEY{color} | 一 |
| 道具栈/次数 | 通用 | Uses{Infinite\|Count(n)} | 一 |

### 3.2 敌人行为维度（不变 8 项；trait/钩子/恢复/渲染特例入表形态见 §4）

### 3.3 C1 覆盖域清单（替代不可证伪的"90%"）

**分母 = 可枚举清单**（下列逐项归层，新增需求照此表登记）：
- task-2/3 取证集：7 道具（15/17/18/19/23/24/25）+ 5 boss（67/68/74/84/87）→ 12 项全归
  第一层（15 置旗/17 凿墙/18 全清/19 回血/23/24 被动/25 被动；67/74/68/84/87 钩子+特例入表）；
- 现有 17 类效果 × 现有行为维度组合 → 第一层；
- floor53 怪群/scene26-31 门控族类新内容 → 第二层 DSL；
- 未知新机制（数值规则/光环/冷却等 DSL 表达不了的）→ 第三层或 Rust。
**表述**：不再用百分比；承诺=「§3.1/§3.2 枚举域内纯表加行；枚举外新内容走 DSL；
枚举外新机制走第三层/扩 ScriptOp（对照性评审）」。

## 4. 目标形态：表 schema v2（按 7 条件修正）

### 4.1 EnemyDef（B1 补全字段）

```rust
pub struct EnemyDef {
    pub type_id: i32,
    // 数值（保真表=原版常量镜像，逐行 evidence）
    pub hp: i32, pub atk: i32, pub def: i32, pub gold: i32,
    pub trait_bits: u8,
    // 分派主轴与渲染面（B1 补全——v1 缺口）
    pub render_category: u8,        // entity_category 主轴（域外新型 = 改码，丑话入档）
    pub sprite_sheet: i32,          // 型→图集映射（entityTypeImage 表）
    pub insert_block: bool,         // blocksPlayerInsert / noInsertTypeFlags
    pub anim_row: i32,              // animRowByType 真值
    pub width_profile: i32, pub anim_profile: i32,
    pub encyclopedia_name: Option<usize>,   // 图鉴文案下标
    // 行为（B3 语义类型簇，禁瘦枚举装厚语义）
    pub battle_kind: BattleKind,
    pub hooks: Vec<BossHook>,       // A5：多钩子+层条件，见 §4.2
    pub restore_ref: Option<usize>, // A3：一事实一宿主——指向 RESTORE_RULES 行，
                                    // respawn 语义归 restore_rules（SpawnIfAbsent 三要素已装下）
    pub render_kind: RenderKind,
    pub script_trigger: Option<ScriptTrigger>,  // B6：触发语义 + 脚本 id
    pub evidence: &'static str,     // E1：a.java 行号锚（保真表必填）
}
```

### 4.2 BossHook（A5：装下 BOSS_TYPE_ORDER 间接 + 层条件）

```rust
pub enum BossHook {
    ByOrderIndex,                   // rposition(BOSS_TYPE_ORDER) 取号（67/69/71/74/75 族）
    FloorWhen { event: i32, floor: i32 },  // 73⇒event(6)@40、69⇒event(7)@35 层条件特例
}
```
**【v2 更正】** v1 `hook_on_kill: Option<i32>` 装不下真实结构（钩子号=表索引间接+层条件追加），
回填前逐钩子从代码重新取证（钩子对照表带行号：BOSS_TYPE_ORDER scene.rs:128、
rposition :5179-5181、73⇒6 :5189、69⇒7 :5260）。

### 4.3 ItemDef（B2/B4/B6 修正）

```rust
pub struct ItemDef {
    pub type_id: i32,
    pub pickup_kind: PickupKind,
    pub effect: Option<Effect>,          // §3.1 全 17 类
    pub passive: Vec<Passive>,           // B4：Passive{kind, multiplier}——倍率参数化
    pub conditions: Vec<Condition>,
    pub desc_idx: usize, pub uses: Uses,
    pub script_trigger: Option<ScriptTrigger>,  // B6
    pub evidence: &'static str,
}
pub struct Passive { pub kind: PassiveKind, pub multiplier: i32 }   // ×2 不再写死枚举名
```

### 4.4 ScriptTrigger（B6 触发语义 + C3 接缝）

```rust
pub enum ScriptTrigger {
    OnSpawn(ScriptId),   // 相位契约：spawn 后单次装载，禁重入
    OnBump(ScriptId),    // bump 触发；重入语义=显式（BUG-012 script0 重入环=反例实弹）
    OnKill(ScriptId),    // 击杀收尾（与 hooks 可并存：hooks=事件号，script=行为链）
    OnActivate(ScriptId),// 使用触发（mode11 对话语境执行——上下文契约入文档）
}
```
每 trigger 定义相位契约（重入允许性/失败回滚/中断语义）——**BUG-012 教训固化**。

**C3 接缝改语义动作层**（解 v1 死锁）：第三层 trait 不以 ScriptOp 为参数形（否则新机制须先扩
ScriptOp=碰保真体），改语义动作层：

```rust
pub trait ScriptRuntime {          // 接缝版本化随动作集演进
    fn execute_action(&mut self, action: SemanticAction) -> ActionResult;
}
// SemanticAction = Spawn/Move/Give/SetFlag/ChangeFloor/…（op→动作解析留在保真解释器内）
```

### 4.5 两份制落地（D1/E1/E2）

- 保真表：`game-core/src/fidelity_tables.rs`（或 data/ 但金样锁死）——修改权=仅 evidence 驱动
  勘误；**门禁**：表值变更 ⇒ commit 必带 Evidence/Confidence（data/naming 体例，E1）+
  金样 55 场景须重过；每行 `evidence` 列带 a.java 锚。
- mod 表：Stage B `mod_tables.rs` 运行时叠放——不进金样闸，独立基线。
- 金样冻结语义改写：**冻结 = 保真表逐字节**（state-dump-golden.json 55 场景 = 保真表镜像）。

## 5. 分派形态普查背景（v1 数据保留 + 更正）

- 121 match 三形实测（复核"未找到反例"）：厚 case 树 12 处/瘦分派 ~60 处/表驱动候选=§4 来源。
  复核建议附扫描脚本——复现命令（可入 tools/）：
  `python3 - <<'PY' … match scrutinee/臂数/单臂行数扫描器（v1 执笔所用）… PY`（逻辑见本报告
  git 历史 v1 版；后续固化进 tools/ 时补全）。
- **【v2 更正】** v1 §5"execute_script_instruction 425 行不拆"——`8a455dd` 已拆 parse_op/
  execute_op 命令模式（16 op 逐 op 独立函数，金样零漂移）。"不拆 switch 为对照体"精神保留，
  现状=已按命令模式重构。
- 状态机 26 处不拆建议：复核无反例，保留。

## 6. 实施顺序与工期（D3 重估：含金样墙钟）

**【v2 更正】** v1"4-5 天"低估（D3：金样单跑实测 460s+，每散点 RED→GREEN×全量回归=小时级墙钟）。
按"点数 × 回归墙钟"重估，步 3 拆每散点一步一闸：

| 步 | 内容 | 工期（含墙钟） | 依赖 |
|---|---|---|---|
| 0 | 钩子对照表逐项取证（A5 前置）+ 表 schema 定稿评审 | 0.5 天 | — |
| 1 | 保真表落地+机械回填（evidence 列逐行；表==旧硬编码等值单测**含缺行/越界/未知 type 三类负例**〔D2〕） | 1.5 天 | 步 0 |
| 2 | mod 表骨架 + 金样冻结语义改写（55 场景=保真表逐字节） | 1 天 | 步 1 |
| 3 | 消费替换（每散点一步一闸：trait_flags→钩子→渲染特例→render_category；每点 RED→GREEN+金样 460s+scenarios） | **3-4 天**（≥4 点 × 小时级回归墙钟） | 步 1/2 |
| 4 | 被动集中（loot_modifiers，25 金翻倍×2） | 1 天 | 步 2 |
| 5 | DSL 静态校验器进 gates（C2：op 词表/参数个数/脚本号引用/flag 号域 + 表行 schema 校验） | 1 天 | 步 1 |
| 6 | 全量回归（state-dump 金样 55 + oracle scenarios 45 + ratchet + gates 8/8） | 1 天 | 步 3/4/5 |

**总估计 9-10 天**（v1 的 4-5 天按 D3 作废；步 6 第三层按需另计）。

**D2 场景数四口径归源**（钉死，回归承诺以四口径全过为准）：

| 口径 | 数 | 归源 |
|---|---|---|
| oracle 场景套件 | **45** | `reference/oracle/scenarios/*.txt` 实测（gates scenarios 时点通过数随新增涨） |
| method_reach 快照 | 44 | completeness.md:749 重测口径（快照刷新前的覆盖数，需随批重刷） |
| v1"44 场景闸" | 44 | = 上一行同源（普查引用的旧数） |
| state-dump 金样 | **55** | `data/status/baselines/state-dump-golden.json`（refactor-behavior-matrix.md:8，含 prologue 长窗 9224 拍）——**回归主闸** |

---

## 7. 条件吸收对照表（复核 7 条 → 本版落点）

| 条件 | 落点 |
|---|---|
| 1 重基线（A1） | §1/§2 重数 + DONE 标注 + §6 工期重估 |
| 2 口径（A2） | §0 口径定义（逻辑编辑点/验证面分列）+ §1/§2 重数 + D2 四口径表 |
| 3 schema（A1/A3/A5/B1/B3/B4） | §4.1 字段补全 / §4.2 BossHook / restore_ref 归宿主 / Passive 参数化 |
| 4 flags[26]（A3） | restore_rules 唯一宿主（SpawnIfAbsent 三要素现成）+ 禁裸 flag 号 |
| 5 两份制（D1/E2） | §0 红线第 4 条 + §4.5 |
| 6 evidence 列+门禁（E1） | §4.1/§4.3 evidence 字段 + §4.5 门禁（data/naming 体例） |
| 7 DSL 校验器+口径+接缝（C2/B5/C3） | §6 步 5 + §0 B5 口径更正 + §4.4 语义动作层 |

> 7 条件全部落回。待用户拍板实施。
