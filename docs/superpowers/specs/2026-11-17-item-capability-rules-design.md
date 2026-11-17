# 道具能力以规则层为真值的设计（S0：封闭原语集 + 可序列化缝）

- 日期：2026-11-17
- 状态：设计待审（brainstorming 架构级路径；批准后进入 writing-plans）
- 相关：`docs/findings/2026-11-17-solver-map-vs-mechanism.md`（机制/判据/题面三层）、
  `docs/superpowers/proposals/2026-11-17-solver-correctness-remediation.md`（5 红修复方案）
- 用户裁定（本会话）：① "新道具不应该导致求解器修改，规则要从规则层库直接取"；
  ② 方案档位选 **C**（先封闭原语集 A，但类型做成可序列化，留开放 DSL 的缝）；
  ③ 快照**必须能 dump**；④ 不认识的道具/原语**必须明确报错**，不许跳过。

## 1 问题

求解器目前把道具语义**手抄**在自己身上，且抄得不全：

| 现状 | 证据 |
|---|---|
| 求解器完全没建模破障（17 金勺子 / 18 玄明石） | `grep -rn "RemoveAdjacent\|RemoveAllType" crates/game-solver/` 零命中 |
| 求解器把墙（ty11）当永久阻挡 | `crates/game-solver/src/solve_local.rs` 通行判定无破障分支 |
| 求解器硬编码道具 ID | 17/18/20/21/22/25 均以字面量出现 |
| 同一"某格能否进入"有两套口径 | `crates/game-solver/src/doors.rs:115` 只豁免 goal/entry；`crates/game-solver/src/solve_local.rs:1262` 豁免 `stair_like`(6\|7\|8) |
| 规则层的机制特例被埋在实现里 | `crates/game-core/src/item_effect.rs:116-118`：`if self.persist.floor == 40 { "本层不能直接瞬移" }` |

后果不是"少算一点"，而是**结论方向错**：把可破的墙判成永久阻挡，会把可绕的门判成必经，
再把这类错结论喂给 ROI 决策。

## 2 目标与非目标

**目标**
1. 道具语义只有一份真值：game-core 的执行面；求解器**只**通过导出的快照消费语义。
2. 新增**道具实例**（用已有原语）时，求解器零改动，且有一条用例能证明这一点。
3. 出现未知道具/未知原语时，求解器**拒绝出结论**（`model_broken`），不沉默跳过。
4. 快照可 dump 成 JSON 并作为基线被门禁比对，任何语义漂移都可见。

**非目标（本次不做，见 §10）**
- 装备（33-40、79-80）的 tier/bonus 属性建模；
- 开放规则 DSL / 效果程序解释器（B 档，只留缝）；
- 道具使用的**价值/ROI** 评估（属于求解器判据，不属于规则层）；
- 随机地图生成器本体。

## 3 契约分层

```
执行面（唯一真值）         crates/game-core/src/item_effect.rs
  ITEM_EFFECTS + ItemAction + activate_item()
        │  推导：ItemAction → Capability（穷尽 match，编译期强制）
        ▼
语义面（唯一导出）         crates/game-core/src/rules/caps.rs
  Capability / CapDef / CapException / CapsSnapshot / cap_snapshot()
        │  dump / 比对
        ▼
账单面（基线）             data/status/baselines/caps_snapshot.json
        │  消费
        ▼
消费面（求解器）           crates/game-solver/  —— 只吃 &CapsSnapshot
```

铁律：
1. 执行面是唯一真值；语义面**不得**手写第二份语义声明。
2. 语义面只能由执行面推导；新增 `ItemAction` 变体必须迫使归类（编译期）。
3. dump 是**账单**不是真值：变更必须单独 commit 并写明意图（同 ratchet 规矩）。

## 4 数据类型

```rust
#[non_exhaustive]
pub enum Capability {
    Obstacle { scope: ObstacleScope, target: i32 },   // 17 Adjacent / 18 WholeFloor，target=11(墙)
    Relocate { mode: RelocateMode },                  // 20 Mirror / 21 FloorUp / 22 FloorDown
    Gain { resource: Resource, amount: Amount },      // 金/血/钥匙/道具
    Stat { kind: StatKind, amount: Amount },          // 29/30 攻防永久提升
    Passive { kind: PassiveKind },                    // 25 LuckyCoinGoldDouble
}

pub enum ObstacleScope { Adjacent, WholeFloor }
pub enum RelocateMode { Mirror, FloorUp, FloorDown }
pub enum Resource { Gold, Hp, Key(KeyColor), Item(i32) }
pub enum KeyColor { Yellow, Red, Blue }
pub enum StatKind { Atk, Def, MaxHp }
pub enum PassiveKind { LuckyCoinGoldDouble }

/// 数值形态：法则层必须能表达"随属性缩放"，不能只装常数。
pub enum Amount {
    Const(i32),
    StatScaled { from: (StatKind, StatKind), num: i32, den: i32 }, // 19: (atk+def)*74/10
}

pub struct CapDef {
    pub item: i32,
    pub caps: Vec<Capability>,
    pub charges: u32,               // 使用次数（数据表结构化字段，见 §5.2）
    pub target_required: bool,      // 例：17 必须相邻有墙
    pub exceptions: Vec<CapException>,
    pub evidence: String,           // A 级出处（文件:行 或 复核报告）
}

pub struct CapException { pub cap: Capability, pub floor: Option<i32>, pub reason: String }

pub struct CapsSnapshot { pub schema_version: u32, pub items: Vec<CapDef> }

/// 预算键：Capability 的「不带参分类」，用于 BTreeMap 计数（带参形式不能当键）。
pub enum CapKind {
    ObstacleAdjacent, ObstacleWholeFloor,
    MirrorTeleport, FloorUp, FloorDown,
    GainGold, GainHp, GainKey, GainItem,
    StatAtk, StatDef, StatMaxHp,
    PassiveLuckyCoin,
}
```

序列化：`serde` 派生，字段名稳定；`schema_version` 变更即破坏性变更，旧 dump 必须重生成。

范围：快照覆盖**使用类道具 + 被动**；装备属性不进来（它们来自 `items.json` 的 tier/bonus 字段，
没有对应的 `ItemAction`，混入会破坏"执行面推导"这条铁律）。

## 5 派生与护栏

### 5.1 派生：两路输入

| 输入 | 来源 | 提供 |
|---|---|---|
| 执行面 | `ITEM_EFFECTS` + `ItemAction` | 动作**种类** |
| 数据表 | `data/tables/items.json`（effect params、新增 charges） | 动作**参数**、**次数** |

`capability_of(action: ItemAction) -> Capability` 使用**穷尽 match**：新增动作变体 ⇒ 编译失败 ⇒ 强制归类。

### 5.2 数据表改动：`charges` 结构化

现状：使用次数只存在于 `desc` 中文文本（`\\c00ff00[使用次数：3次]`）。解析文案取次数不可接受。

改动：`data/tables/items.json` 的每行新增 `"charges": <u32>`，取值证据=desc 原文逐条对照
（20→3；17/18/21/22→1；无"使用次数"标注的道具→1）。缺字段即视为 `MissingCharges` 报错（§6）。

### 5.3 四条护栏

1. **编译期**：见 5.1。
2. **运行期**：求解器遇到数据中的道具，`snapshot.def(item) == None` 且它既非装备也不在
   `NO_EFFECT_ITEMS`（显式白名单，位于 `rules/caps.rs`，每项带理由）⇒ `model_broken`，带 item id 与首次出现层号。
3. **账单基线**：`data/status/baselines/caps_snapshot.json`；门禁比对当前 dump 与基线，
   不一致即 FAIL。
4. **注入式证明**（关键）：求解器**不读全局单例**，而是接收 `&CapsSnapshot` 参数。用例
   `新增道具_用已有原语_求解器零改动`：构造只含 `item 9999 → Obstacle{Adjacent, target:11}, charges:2`
   的假快照，断言求解器将其当作"破墙预算 2"使用。该用例不触碰求解器源码即可通过 ⇒ 证明
   求解器只认原语、不认 ID。

## 6 失败语义

```rust
pub enum ModelDefect {
    UnknownItem { item: i32, floor: i32 },
    UnclassifiedAction { action: &'static str },
    MissingCharges { item: i32 },
    ContradictoryDecl { item: i32, reason: String },
}
// 输出：status = "model_broken", reasons: Vec<ModelDefect>
```

范围界定（重要）：**装备（33-40、79-80）与纯描述道具不算 `UnknownItem`**——它们没有 `ItemAction`，
按 §2 非目标处理。判定方式：`items.json` 中该行的 `category`（或等价字段）为装备即豁免；豁免逻辑也放在
`rules/caps.rs`，不散落在求解器里。

规则：
- CLI 退出码 **2**（区别于 0 成功、1 参数错误）；
- `model_broken` 时**不输出**任何结论数值（`max_hp`/`min_kills`/`necessary`/`reachable` 均不出现，
  只保留 `status` 与 `reasons`）；
- `query --all` 在**开搜之前**做全量预检（所有层的所有道具），避免"算了四分钟才报错"；
- `solve-local` / `doors` / `keys` 同规矩。

## 7 求解器消费

```rust
Budget::from_inventory(inv: &Inventory, snap: &CapsSnapshot) -> BTreeMap<CapKind, u32>;
edges_from_caps(snap: &CapsSnapshot, level: &LevelSpec, pos: (i32,i32)) -> Vec<MoveEdge>;
```

- 预算维度由快照推导，不写死 `mirror/floor_up/floor_down` 字段；
- 能力边全部由快照生成，删除 17/18/20/21/22 的硬编码分支；
- 求解器源码中不得出现道具 ID 字面量（grep 粗筛 + §5.3 注入式证明；不做成强门禁以免误伤测试）。

## 8 迁移切片

每片单独 commit；S0-0..S0-2 要求 74 金样 sha 全同。

### S0-0 语义面投影（零行为）
- 文件：`crates/game-core/src/rules/mod.rs`、`rules/caps.rs`；`combat.rs`/`enums.rs` 不动。
- 判据：`caps_every_action_classified`（穷尽性由编译保证，用例断言表覆盖 20/21/22/17/18/25/19/29-32）、
  `caps_snapshot_has_no_duplicate_item`。
- 预期：dump 包含 10 个道具定义（17/18/19/20/21/22/25/29/30/31/32）。
- 风险：低；回滚＝删文件。

### S0-1 账单与护栏
- 文件：`data/tables/items.json`（+charges）、`crates/game-solver/src/bin/solver.rs`（`caps --out`）、
  `data/status/baselines/caps_snapshot.json`、`gates/`（新增 `caps-snapshot` 检查项）。
- 判据：`caps_snapshot_matches_baseline`、`caps_unknown_item_reports_model_broken`、
  `caps_missing_charges_reports_model_broken`。
- 预期：gates 由 13 项变 14 项，全 PASS；74 金样 sha 全同。
- 风险：中（数据表加字段会触及表哈希门禁，须同步更新该哈希基线并写意图）。

### S0-2 求解器改吃快照
- 文件：`crates/game-solver/src/{solve_local.rs,solve_global.rs,keyflow.rs,doors.rs,bin/solver.rs}`。
- 判据：`新增道具_用已有原语_求解器零改动`、`s0b2_破障边与旧硬编码等价`（17/18 场景对照）、
  `s0b2_上行符边存在`（f45 (2,10)）。
- 预期（确定性判据，二选一，不许模糊）：
  a. 若三查询与当前逐值相同 ⇒ 记录"零漂移"，并将 f45 上行符边标为"已建模但未影响该查询面"；
  b. 若不同 ⇒ 必须逐项列出差异字段（goal 数、hp、金峰、买红次数…）并在文档中给出机制解释；
  **禁止**以"看起来合理"为由接受变化。
- 风险：高；回滚＝单笔 revert。

### S0-3 预算化可达性
- 文件：`crates/game-solver/src/reach.rs`（新）、`doors.rs`。
- 判据：`单格破墙预算1可通`、`两格破墙预算1不可通`、`钥匙计数`、`楼梯终审豁免地形`（23 层上行梯
  坐落在不可走地形上，这条豁免承重）。
- 预期：doors 必经门由 33 → **32**（budget=B0：不可破障、楼梯为终点）。

### S0-4 分层图与能力边
- 文件：`reach.rs`、`crates/game-core/src/item_effect.rs`（f40 硬编码挪出）、`rules/caps.rs`（CapException）。
- 判据：`相形变位_可落在门后并触发交互`、`f40_无镜像边`（data 驱动的例外）、
  `上行符_f45到f46`、`下行符_无来源仍建模`。
- 预期：f40 特例成为 `CapException{cap: Relocate(Mirror), floor: 40}`；`item_effect.rs` 读它。

### S0-5 判据口径与撤回
- 文件：`doors.rs`、`keyflow.rs`、相关 findings 文档。
- 判据：`necessary @ budget` 输出含预算说明；翻转阈值表存在。
- 预期：撤回清单 8 条落文档（必经 33→**32**；同色必要 180→**176**（172 黄/4 红）；红钥 6→**5+1 条件可得**；
  撤「界内无解」；撤「瓶颈=f12 差 33 金」；撤 Q3 结论；f29 破墙仍不通；红钥供给串行余额更新）。
- 风险：中；口径类改动必须成对改文档。

## 9 验证与门禁

- 每片 TDD 先红；
- S0-0..S0-2：74 金样 sha 全同（零行为漂移证据）；
- 变异自证 3 条：删 `capability_of` 一条臂 ⇒ 编译失败（即证明）；删运行期护栏 ⇒ 护栏用例红；
  删基线比对 ⇒ 门禁红；
- `python3 gates/cli.py check` 全程 PASS，S0-1 后为 14 项；
- ratchet 随之更新并在 commit 说明基线影响的理由。

## 10 明确不做

| 项 | 理由 |
|---|---|
| 装备属性建模 | 无对应 `ItemAction`，来源是 `items.json` 字段，属另一条契约 |
| 效果程序 IR / 开放 DSL | B 档；本设计的 `Capability` 是可序列化枚举，升级时外部接口不变 |
| 道具 ROI 估值 | 判据层职责，规则层只提供"能做什么、花多少" |
| 随机地图生成器 | 依赖本契约的 dump，后续单独设计 |

## 11 假设与待证

1. **假设**：所有现役道具效果都能落入五类原语。若 S0-0 出现装不下的 `ItemAction`，则需新增原语
   （属"新机制"，需走 §5.3 的归类流程，并记入下一批）。
2. **假设**：`charges` 与 `desc` 文本一致。S0-1 逐条对照建表，若发现文本与实际消耗不符，
   以**执行面实际消耗次数**为准并登记 findings。
3. **待证**：`RemoveAllType{11}` 是否只作用于当前层（desc 说"当前层所有的墙"，须用 game-core
   实现位置复核并记证据）。

## 12 附录：现状证据（A 级）

- `crates/game-core/src/item_effect.rs`：`ITEM_EFFECTS` 含 `(20, &[ItemAction::MirrorTeleport])`、
  `(21, &[ItemAction::ChangeFloorUp])`、`(22, &[ItemAction::ChangeFloorDown])`（:42-44）；
  `activate_item` 注释「新增道具不改此主体——ITEM_EFFECTS 加行即可」。
- 同文件 `:115-118` 镜像瞬移落点 `(wide-1-x, high-1-y)` 且 **floor==40 拒绝**；
  `:140-148` 上行符 `change_floor(floor+1)`、下行符 `change_floor(floor-1)`。
- `data/tables/items.json`：17 `RemoveAdjacent{target:11}`（"你必须面对一堵墙使用"）；
  18 `RemoveAllType{target:11}`（"震开当前层所有的墙"）；20 次数 3；21/22 次数 1；25 `Passive`。
- 数据可得性（54 层普查）：20 **0 处**、21 仅 f45 (2,10)、22 **0 处**。
- `crates/game-solver/src/solve_local.rs:1262`：`if !(self.walkable(x,y) || stair_like) { return Cell::Blocked }`；
  `crates/game-solver/src/doors.rs:115` 只豁免 goal/entry ⇒ 双口径。
- `crates/game-core/src/scene.rs:3674-3676`：楼梯实体豁免地形终审（23 层的上行梯坐落在不可走地形上，
  该豁免承重）。
