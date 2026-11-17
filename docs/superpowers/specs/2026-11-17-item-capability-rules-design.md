# 道具能力规则层设计（spec，v2 —— 对抗复核后重写）

- 状态：**v2 可进实施计划**（v1 于 2026-11-17 13f1d7c 落盘，经对抗复核定级「不可实施 as written」，本节 §0 逐条更正）
- 目标：求解器判断任何道具效果时，**只看规则层给出的能力快照**，不自己写一份；新增道具不改求解器
- 边界：只覆盖「背包道具的主动能力」；战斗/拾取/装备的账各自有主人（§7）

---

## §0 更正记录（2026-11-17，A 级行锚实测）

v1 的四处核心事实与仓库现状相反，以下每条给出**原主张 → 更正后事实 → 证据**。历史主张保留于此表，不删除。

| # | v1 原主张 | 更正后事实 | 证据 |
|---|---|---|---|
| C1 | `ITEM_EFFECTS` 含 25/29/30/31/32，可投影出 `Passive{LuckyCoinGoldDouble}` 与 `Stat` 类 | 表里**只有 11 行**：13/14/16/17/18/85/19/86/20/21/22。**没有 25、29-32**；反过来 v1 漏了 13/14/16/85/86 | `crates/game-core/src/item_effect.rs:33-45` |
| C2 | 参数可从 `data/tables/items.json` 的 `effect` 字段取 | 参数本来就在代码里：`ItemAction` 载荷（`RemoveAdjacent{target,msg}` / `Heal{mul,div}` 等）；items.json 是**下游副本**（`tools/item-defs/gen_items.py:57-80` 生成）；且 `crates/game-solver/src/ingest.rs:3-5` 已立红线「items.json 22-25 的 effect 禁作求解器效果源（BUG-053）」 | `item_effect.rs:9-30`、`ingest.rs:3-5` |
| C3 | 使用次数只在中文 desc 文本里，需给 items.json 加 `charges` 字段 | 次数在规则层：`ITEM_USES` 按 `id-13` 索引（85→20、86→21），**-1 = 无限**：13/14/15/16/23/24/25 无限、20 为 3、其余 1；消费判定 `(uses as i8) > 0`。给 items.json 加字段不是「只动哈希」——`ITEM_ROW_KEYS` 7 项白名单 + `Reject::UnknownField` ⇒ **加载期拒载** | `crates/game-core/src/effect_registry.rs:201-207`、`crates/game-core/src/scene.rs:2392`、`crates/game-data/src/defs.rs:568-580` |
| C4 | 求解器只有「待补的硬编码若干处」，注入 item 9999 可覆盖全部 | 求解器实际硬编码：`QOL_ITEMS=[15,21,23]`、筋斗云 `contains(&15)`、符 `[(21,1,false),(22,-1,true)]` ⇒ **注入一个假道具完全覆盖不到这些分支**；v1 还主动放弃了强门禁 | `crates/game-solver/src/state.rs:40`、`solve_local.rs:1945`、`solve_local.rs:1997` |
| C5 | S0-3 预期「必经门 33→32」 | 32 来自**另一份未入库提案**的 `cell_gate`（ty6 喷泉格当通道），与破墙能力无关；今日实测仍是 **33（29 黄 + 4 红）** | `docs/superpowers/proposals/2026-11-17-solver-correctness-remediation.md:235`、`./target/release/solver doors` 实测 |
| C6 | 同色必要「180→176 是将来会变」 | **176 = 172 黄 + 4 红，今天就是这个值**；「180」是文档把 summary 总数 176 当黄门数的笔误 | `docs/findings/2026-11-17-mandatory-doors.md:27` |
| C7 | 破障能力对当前结论有实质影响 | 可得性实测：**17 = 世界 0 处且商店无**（不可获得）；18 = 世界 0 处、仅 f47 商店 ¤4000；20 = 0 处；21 = 仅 f45 (2,10) 1 枚；22 = 0 处；25 = 0 处。⇒ 破障能力在当前数据上几乎不改结论，价值是**未来可扩展性 + 死局可解释性** | `tools/type_census.py`、`data/tables/shop_goods.json` 第 17 行 |
| C8 | `HpUp` 是「当前 HP × floor_tier」 | 是**固定增量 × floor_tier**：31 = 50×tier、32 = 200×tier；另有归零地板规则（`hp <= -d` ⇒ `hp = 1` 且 `d = 0`，不发浮层）。更正的重点不是「按当前 HP」，而是**不能写成 `Const(50)`**——f45 时必须是 250 | `crates/game-core/src/effect_registry.rs:259-281`、`:9-10` |

**由此确立的纪律**：语义面**只允许**读 `ItemAction` 载荷 + `ITEM_USES`；禁止读 `items.json` 的 `effect`/`desc`（BUG-053 红线）；禁止为了「结构好看」给数据表加字段——加载器会拒载。

---

## §1 范围（重定）

**在范围内**：8 类主动能力，全部来自 11 行 `ITEM_EFFECTS`：

| id | 名 | `ItemAction` | 能力语义 |
|---|---|---|---|
| 16 | 芭蕉扇 | `RemoveAdjacent{target:10}` | 移除相邻三昧真火 |
| 17 | 金勺子 | `RemoveAdjacent{target:11}` | 移除相邻墙（**需面对墙**） |
| 18 | 玄明石 | `RemoveAllType{target:11}` | 移除**全层**墙 |
| 85 | 黄金钥匙 | `RemoveAllType{target:1}` | 移除**全层黄门**——**不是破墙**，v1 归类错误 |
| 19 | 千年月桂露 | `Heal{mul:74,div:10}` | `(atk+def)*74/10` 加血 |
| 86 | 日月无极钹 | `KillAdjacent4` | 杀四邻 |
| 20 | 相形变位 | `MirrorTeleport` | 移至 `(w-1-x, h-1-y)`；**f40 硬禁**；落点先 `interact_with_cell` |
| 21/22 | 上行/下行符 | `ChangeFloorUp/Down` | `change_floor(f±1)`，起跳不耗步 |
| 13/14 | 火眼金睛/生死簿 | `OpenPanel` | 纯 UI，**不投影能力**（记 `NoCapability`） |

**明确不在范围内**（各有主人，v1 错误地把它们塞进本 spec）：
- **25 幸运币**：被动在战斗结算里（`combat::kill_reward` 的 `find_item(25)` 双倍），C6d 已单源化；不属「主动能力」，不进快照。
- **29-32 丹**：走 `PICKUP_ARM_TABLE` 的 `StatUp`/`HpUp`（拾取即生效、不进背包能力面）。
- **26-28 钥匙 / 33-40、79-80 装备**：各有独立账（钥匙计数、`Equip` 的 tier+bonus），不进快照。

---

## §2 四层单向依赖（不变，但来源改正）

```
执行面  crates/game-core/src/item_effect.rs
        ItemAction 载荷 + ITEM_EFFECTS 表（唯一真值）
   │  穷尽 match（新增 ItemAction 变体 ⇒ 编译失败）
   ▼
语义面  crates/game-core/src/rules/caps.rs（新增）
        Capability / CapDef / CapException / CapsSnapshot / cap_snapshot()
   │  dump（可序列化）
   ▼
账单面  data/status/baselines/caps_snapshot.json（门禁 13→14 项：漂移即 FAIL）
   │  只读参数注入
   ▼
消费面  crates/game-solver（只收 &CapsSnapshot，不读任何全局单例、不读 items.json）
```

**为什么必须单向**：求解器读 items.json 就等于让「数据表」当第二真值——这正是批5 漏击杀金币、批6 重写赏金公式的同一类错误（两份实现靠人同步）。

---

## §3 语义面：能力枚举

```rust
pub enum Capability {
    /// 移除障碍：scope 决定范围，target 是被移除的实体型
    Obstacle { scope: OblivionScope, target: i32 },
    /// 位移：镜像瞬移 / 上行 / 下行
    Relocate(RelocateKind),
    /// 直接杀四邻
    KillAdjacent,
    /// 加血（公式参数来自 ItemAction::Heal）
    Heal { mul: i32, div: i32 },
    /// 纯 UI，不进预算
    NoCapability,
}
pub enum OblivionScope { Adjacent, WholeFloor }
pub enum RelocateKind { Mirror, FloorUp, FloorDown }

pub struct CapDef {
    pub item: i32,
    pub capability: Capability,
    /// 次数：-1 = 无限（与 ITEM_USES 同语义，禁止用 u32）
    pub uses: i32,
    /// 前置/例外（f40 禁用镜像），从 item_effect.rs 的硬编码搬来
    pub exception: Option<CapException>,
}
pub struct CapException { pub floor_forbidden: Option<i32> }

/// 预算键：同 (kind, target) 同桶——18 拆墙与 85 开门**不同桶**（v1 把 target 丢了）
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum CapKey { Obstacle { scope: OblivionScope, target: i32 }, Relocate(RelocateKind), KillAdjacent, Heal }
```

**次数语义**（v1 用 `charges: u32` 是错的）：`ITEM_USES` 直接给 `i32`，`-1` 表示无限；投影时**原样搬运**，禁止「无标注按 1」——那会把 7 件无限道具降为一次性（行为级错数据）。

**编译期穷尽**：`fn capability_of(a: &ItemAction) -> Capability` 用穷尽 `match`；新增 `ItemAction` 变体即编译失败。

---

## §4 快照与数值

```rust
pub struct CapsSnapshot { pub rows: Vec<CapDef>, pub schema: u32 }
pub struct Amount { pub kind: AmountKind, pub value: i32 }
pub enum AmountKind { Const, TierScaled }   // 19 是 StatScaled（(atk+def)*mul/div），见下
```

- 19 加血量是**状态相关**（依赖 atk+def）⇒ 快照里只存 `{mul:74,div:10}`，不存算好的数。
- 31/32 若将来纳入（目前不在范围），必须 `TierScaled`：50/200 × `floor_tier`，禁 `Const`。
- `floor_tier`（`effect_registry.rs:157-166`）：`((f-1)/10).max(0)`、`f>50 ⇒ 0`、再 `+1`。

---

## §5 四条护栏

1. **编译期穷尽**：`capability_of` 对 `ItemAction` 穷尽匹配。
2. **运行期显式失败**：快照里出现求解器不认识的 `Capability`（例如未来新增变体而求解器未升级）⇒ 打 `model_broken`、退出码 2、**不输出任何结论数值**（不许降级成「忽略该能力」继续算）。
3. **账单漂移即红**：`caps_snapshot.json` 与 `cap_snapshot()` 重跑结果不一致 ⇒ 门禁 FAIL（13→14 项）。
4. **注入式证明（改法）**：v1 的「注入 item 9999」覆盖不到真实硬编码。改为**行为级注入**：在测试里构造一份含 `Relocate(FloorUp)` 的假快照，喂给求解器的「符换层」路径，要求它**从快照生成跨层边**、而不是走 `[(21,1,false),(22,-1,true)]` 字面量；删掉该字面量后测试仍绿、恢复字面量后测试红。

---

## §6 消费面改造（真正的工作量）

| 位置 | 现状 | 改为 |
|---|---|---|
| `crates/game-solver/src/state.rs:40` | `QOL_ITEMS=[15,21,23]` | 从快照派生「可移动能力道具」集合，或明确保留为题面常量并注出处（二者择一，不得混） |
| `solve_local.rs:1945` | 筋斗云 `item_types.contains(&15)` | 15 的行动语义来自 `ITEM_EFFECTS` 之外的专用路径（`cloud_goals`），保持现状但**登记为已知特例** |
| `solve_local.rs:1997` | `[(21,1,false),(22,-1,true)]` | 遍历快照里 `Relocate{FloorUp|FloorDown}` 的条目生成跨层边 |

**能力边签名必须带状态**：`edges_from_caps(snap, level, pos)` 不够——能否破格取决于剩余次数与背包，签名须为 `edges_from_caps(snap, level, pos, &CanonicalBag)`；全局账本已有 `solve_global.rs:128 GKey.bag`，**不要**再造 `BTreeMap<CapKind,u32>` 并行结构（会丢 -1 无限语义）。

---

## §7 切片（S0-0..S0-5，预期数字按实测重写）

| 片 | 改什么 | 判据 | 预期 |
|---|---|---|---|
| S0-0 | `rules/caps.rs` 纯投影（11 行表）+ 单测 | 每条 id 的 `Capability` 与 `ItemAction` 一致；13/14 = NoCapability | 零行为，金样 74 不变 |
| S0-1 | `solver caps --out` + 基线 JSON + 门禁 13→14 | 漂移即红；变异：改一条 uses 必须红 | 快照行数 = 11 |
| S0-2 | 求解器删 `[(21,1,false),(22,-1,true)]` 等硬编码，改吃快照 | 行为级注入证明（§5.4） | 三查询逐值不变 |
| S0-3 | 可达性预算化（`(位置, 已花预算)` 搜索） | 前提不成立时报错，不回落 | **必经门仍 33**（32 属另一提案的 ty6 口径，不在本片） |
| S0-4 | 能力边进分层图（含 18/85/16/86/20/21/22） | 同一能力在不同层数量不同 | 当前数据仅 21 有 1 枚，18 需 f47 购买 |
| S0-5 | doors/keyflow 预算化，输出 `necessary @ budget` | 报告必须打印所用预算 | 176（172 黄 + 4 红）为今日基线 |

**不做**：给 `items.json` 加 `charges`/`category` 字段（加载器拒载）；把 25/29-32 塞进能力快照；用本 spec 解释「界内无解」（那是搜索覆盖问题）。

---

## §8 未验证项（进实施前必须测）

1. f23 上行梯所在地形的可走值（影响「楼梯豁免」是否被过度使用）。
2. f47 的 ▯4000 玄明石在全局搜索中是否经济可达——若不可达，18 的破墙能力在整局里永远是死边。
3. 若将来给 29-32 加 `ItemAction`，对 74 金样的影响（当前它们在 pickup 面，不动则零影响）。
