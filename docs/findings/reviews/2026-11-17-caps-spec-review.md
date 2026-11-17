# 红队复核：道具能力规则层 spec（`2026-11-17-item-capability-rules-design.md`）

- 复核对象：`docs/superpowers/specs/2026-11-17-item-capability-rules-design.md`（267 行，commit `13f1d7c`）
- 复核日期：2026-11-17
- 角色：对抗性设计复核（红队）。只找「会让实施阶段走偏」的具体缺陷，不评价文风/排版。
- 纪律：文中每条断言都带 `file:line` 或命令输出；无法用现有仓库事实裁决的标 **未验证**。
  不以「测试全绿」当设计正确的证据。
- 边界：本报告是本次复核唯一写入的文件；未改代码、未改 spec、未 push。

---

## 结论

**判定：不可实施（as written）。** 方向（规则层为唯一真值、求解器只吃快照、未知即报错）正确且与
`docs/findings/2026-11-17-solver-map-vs-mechanism.md` 的机制/判据分层同向；但 spec 的**四个承重事实**
与仓库现状不符，照它实施会在 S0-0 就撞墙，且会被"就地修补"成它自己声称要消灭的形态（第二份手写语义）。

| 项 | 值 |
|---|---|
| 面级判定 | **红 8 / 黄 2 / 绿 0**（F1/F2/F3/F4/F5/F6/F7/F9 红，F8/F10 黄） |
| Confidence | **88**（承重反驳均为 A 级代码/数据事实；余 12 分留给三处「未验证」：f23 梯地形、18 的 4000 金全局可达性、给 29-32 加 `ItemAction` 的回归影响） |
| 最主要的一条 | **规则层有「两张互相矛盾的执行表」**（`item_effect.rs:33-45` vs `effect_registry.rs:104-140`），spec 只承认其中一张，而求解器实际走的是另一张 |

四个承重事实错误（详见逐面）：

1. `ITEM_EFFECTS` 里**没有** 25/29/30/31/32（`crates/game-core/src/item_effect.rs:33-45`，11 行），
   spec §8 S0-0 的预期 dump 集合**推不出来**；反过来它有 13/14/16/85/86，spec 的集合**漏了 5 个**。
2. spec §5.1 说"参数由 items.json 提供"，但 `ItemAction` 变体**自带参数**（`RemoveAdjacent{target:11}`、
   `Heal{mul:74,div:10}`），items.json 只是**从代码抄过去的副本**（`tools/item-defs/gen_items.py:57-80`）；
   且 `crates/game-solver/src/ingest.rs:3-5` **已立红线**：items.json 的 effect 字段禁作求解器效果源（BUG-053）。
3. 使用次数**已经**在规则层（`crates/game-core/src/effect_registry.rs:201` `ITEM_USES`），
   spec §5.2「次数只存在于 desc 中文文本」为假；且 `charges: u32` 无法表达其中的 **-1=无限**。
4. §6 的装备豁免依据 `items.json` 行的 `category`（或等价字段）——**该字段不存在**
   （`crates/game-data/src/defs.rs:568` 行键白名单只有 7 个）。

---

## 逐面发现

### F1 单源是否真的成立 —— **红**

**缺陷 1（致命）：参数已经在代码里，items.json 是它的副本，"两路输入"是把副本当真值。**
- `crates/game-core/src/item_effect.rs:13-19`：`RemoveAdjacent { target: i32, msg: &'static str }`、
  `RemoveAllType { target: i32 }`、`Heal { mul: i32, div: i32, ... }` —— **参数是 `ItemAction` 的载荷**。
  实表 `:36-40` 直接写 `target: 11`、`target: 10`、`mul: 74, div: 10`。
- `data/tables/items.json` 的 `effect.params` 由 `tools/item-defs/gen_items.py:57-80` 逐 id 写死生成
  （该文件 note 明写「禁手抄逐值迁」），即**数据表是代码的投影**。
- 因此 spec §5.1 表格（`spec:118-122`）「执行面给种类 / 数据表给参数」是**反的**。照此实施只有两条路：
  (a) `capability_of(action)` 忽略 `action.target` 去查表 —— 代码里的 `target` 变成死字段，
      改了不会有任何编译错误，**漂移无警报**；(b) 忽略表 —— 那 §5.1 就是假话，S0-1 的"改表"理由随之消失。

**缺陷 2（致命）：仓库已有红线禁止把 items.json 的 effect 当效果源。**
- `crates/game-solver/src/ingest.rs:3-5`：「**红线**（计划 §0/§2.7，BUG-053）：items.json 22-25 的 effect
  字段**禁作求解器效果源**……效果建模一律以**原版 activateItem** 与实考证为准」。
- `docs/bug-ledger.md:1039-1046`（BUG-053）：id22 曾标 `Passive AntiNarcissusDamage` 而实际是
  `changeFloor(currentFloor−1)`；"求解器若按 items.json effect 建模=按错效果算"。
- spec §5.1 恰恰提议按 items.json effect 建模 ⇒ **复辟 BUG-053 的触发条件**。

**缺陷 3（致命）：game-core 内部有两张互相矛盾的执行表，spec 只认一张。**
```
$ grep -n '"Passive"' crates/game-core/src/effect_registry.rs
118:    (22, "Passive", EffectParams::Stack { type_id: 22, tutorial: false }),
```
- `effect_registry.rs:117-120`：22/23/24/25 全部标 `Passive`；而 `item_effect.rs:44` 把 **22 标为
  `ChangeFloorDown`**（主动换层）。BUG-053 的更正记录也承认这处「标注同病未随改」
  （`docs/bug-ledger.md:1053-1055`）。
- 引擎**拾取**走 `effect_registry` 的 `PICKUP_ARM_TABLE`（`:104-140`），**使用**走 `item_effect` 的
  `ITEM_EFFECTS`。求解器的 `pickup_step` 又是走引擎（`crates/game-solver/src/solve_local.rs:2113-2195`）
  ⇒ 快照若只由 `ITEM_EFFECTS` 推导，就**描述不了求解器实际执行的那一半**。
- 结论：这不是"两路输入"问题，是"**规则层自己就有两份真值**"问题。spec 必须先裁决 22-25 谁是权威，
  否则 §3 的铁律 1（唯一真值）在实施第一天就不成立。

**修法（可直接照改）**
1. 删掉 §5.1 表格第二行：参数**只**取 `ItemAction` 载荷，`capability_of` 直接读 `action.target/mul/div`。
   items.json 的 `effect.params` 从此**不得**被语义面读取（与 `ingest.rs:3-5` 红线一致）。
2. 次数：把 `effect_registry.rs:201` 的 `ITEM_USES` 提升为 `pub const ITEM_CHARGES: &[(i32, Charges)]`
   放在 `item_effect.rs` 旁（**零新数据、零 schema 变更、不进 items.json**），并加变体：
   ```rust
   pub enum Charges { Infinite, N(u32) }
   ```
   85/86 的 `idx` 重映射逻辑（`effect_registry.rs:199`：85→20、86→21）必须一起搬进具名表，
   否则快照里 85/86 的次数会取到 33/34 槽位。
3. 22-25 的权威裁决写进 spec（并同步改 `effect_registry.rs:117-120` 的标注或写明为何保留）：
   22=主动 `ChangeFloorDown`（依据 `item_effect.rs:44` + BUG-053 已修事实）。
4. 若坚持"快照包含拾取面"（29-32 等），则必须显式声明**两张表都是执行面**：
   `CapsSnapshot` 由 `ITEM_EFFECTS ∪ PICKUP_ARM_TABLE` 推导，并给出两表冲突时的仲裁规则。
   **不要**再从 items.json 取任何语义。

---

### F2 注入式证明是否真能证明 —— **红**

**缺陷 1：`s0b2_破障边与旧硬编码等价` 不可执行——不存在"旧硬编码"。**
- spec 自己 §1 的证据行就是 `grep -rn "RemoveAdjacent\|RemoveAllType" crates/game-solver/` **零命中**
  （复跑：`0`）。既然没有旧实现，就没有"等价"这个判据对象。该用例名会把实现者逼到"自己写一版旧逻辑再比"
  的假等价上（循环论证）。

**缺陷 2：一条注入用例只覆盖一族原语，且被绕过的 ID 硬编码正好在它覆盖不到的地方。**
```
$ for id in 15 17 18 20 21 22 23 25; do ... done   # 去掉注释行后统计字面量
15=2  17=0  18=1  20=0  21=2  22=1  23=3  25=0
$ grep -nE "(15|21|22)" crates/game-solver/src/state.rs crates/game-solver/src/solve_local.rs | grep -v "//"
crates/game-solver/src/state.rs:40:pub const QOL_ITEMS: &[i32] = &[15, 21, 23];
crates/game-solver/src/solve_local.rs:1945:        if !self.cloud || !st.player.item_types.contains(&15) {
crates/game-solver/src/solve_local.rs:1997:        for (item, delta, went_down) in [(21u8, 1i32, false), (22u8, -1, true)] {
```
- 注入的 `item 9999` 与 15（筋斗云换层）/21/22（符换层）**无关**：这三处仍按 ID 判，
  用例全绿也证明不了"零改动"。而 spec §7 又要求"源码不得出现道具 ID 字面量，但**不做成强门禁**"
  （`spec:174`）⇒ 唯一能抓住这三处的机制被主动放弃。
- 附带：spec:19 的表格行「17/18/20/21/22/25 均以字面量出现」**对 4/6 个 id 是假的**
  （17/18/20/25 各 0 次）。真实清单是 15/21/22/23。

**缺陷 3：注入证明掩盖了"边是状态依赖"这一层（见 F6）。**
一条 `Obstacle{Adjacent}` 注入用例只测"能不能生成一条破墙边"，不测"没有该道具时不生成边"
（负例），也不测"跨层之后仍持有"（预算不丢）。

**需要几条用例才能覆盖"零改动"承诺（最小集，6 条）**
1. `注入_ObstacleAdjacent_产边且预算2`：断言 `Budget::from_inventory(&inv9999x2,&fake)[&CapKey::Obstacle(Adjacent,11)]==2`
   且合成 3×3 层上 `reachable` 由 false→true。
2. `注入_ObstacleWholeFloor_与_Adjacent_不同桶`：`target=11` 与 `target=1` 必须是两个不同的预算键（见 F5 缺陷 3）。
3. `注入_RelocateMirror_状态依赖`：未持有不产边、持有产边、f40 上不产边（三条断言同一用例）。
4. `注入_CrossFloor_双向`：`Relocate{FloorUp}` 必须产出 `(target_floor, went_down)` 跨层边，
   并断言 `solve_global` 侧把它当层间边消费（对齐 `solve_global.rs:445-475`）。
5. `注入_GainHp_随层缩放`：`Heal` 在 f45 的收益必须按 `floor_tier(45)=5` 计（见 F5 缺陷 4）。
6. `静态门禁_求解器源码道具ID字面量=0`：**做成强门禁**（正则白名单 `9999`/测试夹具），
   否则 15/21/22 那三处永远逃出证明。

---

### F3 失败语义的爆炸半径 —— **红**

**缺陷：第一次在合法数据上跑就会 `model_broken`，且报的是**可拾取、有真实效果**的道具。**
按 spec 的 dump 集合（17/18/19/20/21/22/25/29/30/31/32）对世界普查逐 id 对账：

```
$ python3 tools/type_census.py  # 经其 census() API 汇总 sprite0..54，共 1703 记录
13 → f52；14 → f5；15 → f1；21 → f45；23 → f19；25 → f0；85 → f46；86 → f37
17 → 0 处；18 → 0 处（唯一来源=商店）；16/19/20/22/24 → 0 处
```

| item | 世界位置 | 快照内？ | 首跑后果 |
|---|---|---|---|
| 13 火眼金睛 | f52 ×1 | ❌ | **model_broken**（它有 `ItemAction::OpenPanel`，`item_effect.rs:34`） |
| 14 生死簿 | f5 ×1 | ❌ | **model_broken**（`OpenPanel`，`:35`） |
| 15 筋斗云 | f1 ×1 | ❌ | **model_broken**（`TutorialSeenStack`，是换层机制本体，`solve_local.rs:1945`） |
| 16 芭蕉扇 | 0 处 | ❌ | 暂不触发（世界无），但数据合法、`RemoveAdjacent{target:10}` 无原语可装 → 见 F5 |
| 23 太公杖 | f19 ×1 | ❌ | **model_broken** 或被迫豁免（但它有真实战斗被动，豁免=说谎） |
| 85 黄金钥匙 | f46 ×1 | ❌ | **model_broken**（`RemoveAllType{target:1}`，是"开全层黄门"，见 F5 缺陷 3） |
| 86 日月无极钹 | f37 ×1 | ❌ | **model_broken**（`KillAdjacent4` 无原语，见 F5 缺陷 1） |
| 26/27/28 钥匙 | 全图大量 | ❌ | 必须豁免；豁免依据见下一条 |
| 33-40/79/80 装备 | 各 1 处 | ❌ | 必须豁免 |
| 17/18/19/20/22/24 | 17: 0 处无商店；18: 仅 f47 ¤4000 | ✅ | 进快照但**当前数据几乎不可达**（见 F9） |

**缺陷 2：豁免依据的字段不存在。**
- `spec:158` 写「`items.json` 中该行的 `category`（或等价字段）为装备即豁免」，但
  `crates/game-data/src/defs.rs:568` 行键白名单 = `id/name/icon/desc/pickup_msg/effect/evidence`，
  **无 `category`**；`items.json` 32 行同。同段还称装备属性来自「items.json 的 tier/bonus 字段」，
  实际在 `effect.params` 内（33 号：`{"tier":1,"bonus":10}`）——**也不存在独立字段**。
- 且 `parse_item` 对行键外字段是**硬拒载**（`defs.rs:571-580` `Reject::UnknownField`；
  用例 `crates/game-data/tests/items_defs.rs:32-36 items_reject_unknown_field`）⇒ 想加 `category` 就是 F4 的全部破坏面再来一遍。

**修法**
1. 豁免表写成 `rules/caps.rs` 的**显式常量**（spec 自己已要求"每项带理由"），例如
   `EQUIPMENT_ITEMS: &[i32] = &[33,34,35,36,37,38,39,40,79,80]`（依据 `items.json` effect.key=Equip，
   32 行实测），`KEY_ITEMS: &[i32] = &[26,27,28]`；不要依赖任何"category"。
2. 补 13/14/15/85/86 到快照（它们有 `ItemAction`），或明确列入白名单并写清"为何不构成求解器能力"
   （13/14 打开的是 UI 面板，可以白名单；15/85/86 不能白名单——它们改通行/战斗）。
3. 把 §6 的 `UnknownItem` 语义改成"**未知 ItemAction** 才报错，未知 id 且无 effect 才报错"，
   避免把"UI 类道具"和"未建模能力"混成一个错误码。

---

### F4 charges 数据改动的破坏面 —— **红**

**缺陷 1：加字段不是"只动表哈希基线"，是加载期硬拒载。**
- `crates/game-data/src/defs.rs:568-580`：行键白名单 + `UnknownField` ⇒ 给 items.json 行加 `charges`
  **立刻**让 `parse_items` 失败；所有消费者同时红：`crates/game-solver/src/bin/solver.rs:79`
  （`parse_items(include_str!(...items.json))`）、`crates/game-core/src/effect_registry.rs:436-446`/`:634-652`
  （断言 rows 型集 == items.json id 集）、`crates/game-oracle/tests/p15_zero_code_drill.rs:11`（`include_str!` items.json）。
- **S0-1 的文件清单（`spec:191`）里没有 `crates/game-data/src/defs.rs`**，也没有生成器
  → 按切片清单实施，第一刀就红，然后实施者会"顺手"改 game-data 解析器（未计划、无证据链、跨契约面）。

**缺陷 2：手改表会被生成器抹掉。**
`tools/item-defs/gen_items.py` 是 items.json 的生成器（note「禁手抄逐值迁」），且 `grep gen_items gates/ tools/*.py crates/*/tests/`
**零命中** —— 没有门禁校验"表 == 生成器输出"。手工加 `charges` 后任何人重跑生成器即静默丢失，
且没有任何门禁会发现（只有 `table-content-hash` 会红，然后被"更新基线"掩盖）。

**缺陷 3（数据正确性）：spec 的取值规则会**改坏** 4 个道具。**
- `spec:130-131`：`20→3；17/18/21/22→1；无"使用次数"标注→1`。**没有「无限」这一档。**
- 实测 desc：13/14/15/16 都写「使用次数：无限」；`charges: u32` 表达不了无限。
- 规则层真值 `effect_registry.rs:201`：`13/14/15/16/23/24/25 = -1`（无限），消费判定
  `crates/game-core/src/scene.rs:2392` `if (self.player.item_uses[idx] as i8) > 0` ⇒ `-1` 走"不消耗"臂。
  照 spec 填 `1`，就把 4 件无限道具降为一次性 —— **这是行为级错误数据**，不是勘误。
- 另外 `spec:249` 假设 2 说"若文本与实际消耗不符，以执行面实际消耗为准" —— 那么 13/14/15/16
  必须写"无限"，与 `charges: u32` 直接冲突（必须加变体或哨兵，见 F1 修法 2）。

**准确的破坏面清单与顺序（照此执行，不让任一步静默）**

| # | 文件 | 动作 | 为什么必须在这一步 |
|---|---|---|---|
| 1 | `crates/game-data/src/defs.rs`（`ITEM_ROW_KEYS` + `ItemDef` + `parse_item`） | 先支持**可选** `charges`（`Option`，缺省不报错） | 否则 items.json 一加字段就拒载（`defs.rs:571-580`） |
| 2 | `tools/item-defs/gen_items.py` | 增加 `charges` 产出（含 `Infinite`） | 否则下次重生成即抹掉（无门禁可查） |
| 3 | `data/tables/items.json`（32 行）+ `note` | 重新生成，不手改 | 与 1/2 同 commit 才自洽 |
| 4 | `crates/game-data/tests/items_defs.rs` | 加"缺 `charges` 时行为"与"非法 `charges` 拒载"用例 | 防 UnknownField 语义被放松 |
| 5 | `crates/game-core/src/rules/caps.rs` + `crates/game-solver/src/bin/solver.rs` | `caps --out` 消费该字段 | 语义面落点 |
| 6 | `data/status/baselines/rust-test-counts.json` | `python3 gates/cli.py ratchet-update` | 新用例计入 ratchet（只许增） |
| 7 | `gates/table_content_hash.json` | `python3 gates/cli.py table-content-hash-update`，**单独 commit** 写明"加字段非改值" | `gates/cli.py:415-455 gate_table_content_hash`（items.json 是 FIDELITY_TABLES 成员） |

**不受影响的（spec 未点名，但提了会误导做法）**
- `table-evidence`：`tools/table-evidence/comparator.py:20-22` 只核 `sprite_defs/anim_defs/sheet_index`，
  **不覆盖 items.json**（`grep -n "items" tools/table-evidence/comparator.py` 零命中）。
  所以 `gates/cli.py:419` 文档串里"逐值锚证据更新（tools/table-evidence 生成式）"对 items.json **没有落点**；
  照它去"重锚证据"会找不到入口。
- `asset-truth-manifest`：`grep -n "data/tables" tools/asset_manifest.py` 零命中，不涉及。
- **74 金样**：`tools/state_baseline.py:19` 的 `TARGETS = ["fixture_frames","preset_cases","gameplay_floor1"]`
  只比 `state_dump`，items.json 不参与 dump ⇒ 加字段 sha **必然全同**。因此
  「74 金样 sha 全同」**不能**充当"加字段无副作用"的证据（它连 parse 是否成功都不看）。

**证明"加字段而非改值"的正确命令（写进 commit）**
```bash
# 旧表 vs 新表：按键对齐后逐行 diff，必须只有新增键
git show HEAD:data/tables/items.json > /tmp/old.json
python3 - <<'EOF'
import json
o={r['id']:r for r in json.load(open('/tmp/old.json'))['rows']}
n={r['id']:r for r in json.load(open('data/tables/items.json'))['rows']}
assert o.keys()==n.keys(), "行集变了"
for i in o:
    diff={k for k in n[i] if k not in o[i]}
    changed={k for k in o[i] if o[i][k]!=n[i].get(k)}
    assert diff=={'charges'} and not changed, (i,diff,changed)
print("OK：仅新增 charges，既有值零变更")
EOF
```

---

### F5 能力语义的漏项 —— **红**

逐 `ItemAction` 对账（`crates/game-core/src/item_effect.rs:8-24` 共 8 变体）：

| ItemAction（行） | 用它的 id | spec 原语能装？ | 缺陷 |
|---|---|---|---|
| `OpenPanel{panel,page}`（:10） | 13, 14 | ❌ | 无原语。13 在 f52、14 在 f5 都可拾取 ⇒ 首跑必撞（F3）。建议 `Ui{panel}` 或白名单+理由 |
| `RemoveAdjacent{target,msg}`（:12） | 16, 17 | ✅ `Obstacle{Adjacent}` | 但**未命中不消耗**（`:77-84` 返回 false）⇒ 预算语义缺 `consume_on_miss` |
| `RemoveAllType{target}`（:14） | 18, 85 | ⚠️ 半装 | **语义合并**：18 是"拆墙"（target=11），85 是"开本层全部黄门"（target=1，`item_effect.rs:39`）。二者都落到 `ObstacleWholeFloor`，而 `CapKind`（`spec:100-107`）**不含 target** ⇒ 预算把"玄明石"和"黄金钥匙"计成同一桶 |
| `Heal{mul,div,msg_fmt}`（:16） | 19 | ✅ `StatScaled{(Atk,Def),74,10}` | — |
| `KillAdjacent4`（:18） | 86 | ❌ | **装不进去**（不是 Obstacle/Relocate/Gain/Stat/Passive）。86 在 f37 可拾取 ⇒ 首跑必撞。建议 `Clear{scope:Adjacent4, target:Enemy}` |
| `MirrorTeleport`（:20） | 20 | ⚠️ 半装 | 落点走 `interact_with_cell` 终审（`:119-124`）⇒ 边的前置条件是"落点格可进入（含门/梯）"，`Relocate{Mirror}` 表达不了。S0-4 的用例 `相形变位_可落在门后并触发交互` 正是在测这个，但类型里没有 `pass` 谓词 |
| `ChangeFloorUp/Down`（:22-23） | 21, 22 | ⚠️ 半装 | 见 F7：需要带目标层/went_down 的跨层边，`Vec<MoveEdge>` 装不下 |

**passive / 装备 / 消耗品三类边界**
- 23/24（`Passive` 与 Stack 拾取，`effect_registry.rs:118-119`）是**可拾取且有真实效果**
  （23 在 f19；BUG-053 记录 23/24 别名在战斗侧已实现），spec 完全没提 23/24 ⇒ 要么建模要么显式豁免，二选一必须写。
- 25 只有 f0（标题图）1 枚、1..54 层 0 枚（普查；同结论见
  `docs/findings/2026-11-17-shop-flags-and-economy-ceiling.md:38-44`）⇒ `Passive{LuckyCoinGoldDouble}`
  在当前数据是**死代码**，给 25 配 `charges` 更无意义（see F10）。

**缺陷 4（数值形态）：`Amount` 表达不了 `×floor_tier`，也会把 HpUp 误标为 MaxHp。**
```
crates/game-core/src/effect_registry.rs:157-166  floor_tier(f) = (f-1)/10 钳 0，f>50→0，+1
crates/game-core/src/effect_registry.rs:238-251  StatUp: f<=10 ? +1 : +1*tier
crates/game-core/src/effect_registry.rs:262-281  HpUp:  hp += 50*tier（当前血量，且 hp<=-d 时落到 1）
```
- spec `spec:76/80` 的 `StatKind{Atk,Def,MaxHp}` + `Amount{Const,StatScaled}`：
  31/32 是 **当前 HP** 增量且**随层缩放**，不是 MaxHp、也不是常量 ⇒ 用 `Const(50)` 建模在 f>10 的层**数值就错**（f45 应为 250）。
  同理 29/30 的 `+1` 在 f>10 应变为 `+tier`。
- 修法：`Amount` 加 `TierScaled { base: i32 }`（乘 `floor_tier(floor)`），`StatKind` 去掉 `MaxHp`，
  改 `Gain{resource:Hp, amount:TierScaled{50}}`。

**缺陷 5：多动作道具的预算语义未定义。**
`ITEM_EFFECTS` 类型是 `&[ItemAction]`（多动作），`CapDef.caps: Vec<Capability>` 也允许多个，
但"用一次扣 1 次却产出 N 个能力"在 `Budget` 里没有规则（当前表里恰好每 id 单动作，属**数据巧合**）。
要么在 spec 写明"多动作道具按整单计 1 次，预算逐 cap 各 +1"，要么把 `caps` 限定为 1 并加编译期断言。

---

### F6 预算与边的时间性 —— **红**

**缺陷：`edges_from_caps(snap, level, pos)` 没有状态参数，无法表达"捡到才能用"，而现状代码是状态依赖的。**
```
crates/game-solver/src/solve_local.rs:1945   if !self.cloud || !st.player.item_types.contains(&15) { return out; }
crates/game-solver/src/solve_local.rs:1997   for (item, delta, went_down) in [(21u8,1i32,false), (22u8,-1,true)]
crates/game-solver/src/solve_local.rs:1998       if !st.player.item_types.contains(&item) { continue; }
crates/game-solver/src/solve_local.rs:337    pub cross_edge: Option<(i32,bool)>
```
- 现实现：边由**搜索态**（`St.player.item_types`）决定，且 `record_useitem` 会 `consume_item`
  （`:1049-1060`）扣掉一枚。spec 的签名把 `level,pos` 之外全部丢掉 ⇒ 实施时必然退化成
  "静态边表"，于是"没捡到符也能换层"——**比现在更错**（现在是欠近似，静态边是过近似并产生假可行解）。
- 更严重的是**预算的记账位置**没说（见 F7），静态边下会出现"同一枚 18 在 f37 用一次、f2 再用一次"。

**修法（签名 + 语义两处都改）**
```rust
// 能力边：随状态产生，不是一次性静态表
fn capability_edges(snap:&CapsSnapshot, bag:&CanonicalBag, level:&LevelSpec, pos:(i32,i32)) -> Vec<MoveEdge>;
// bag 现成：crates/game-solver/src/state.rs（CanonicalBag::from_player，含 types+uses）
```
并在 spec §7 明确一句：「能力边在**每个搜索态**上生成；`CapsSnapshot` 只提供"能做/花几次"的静态事实，
不提供"现在能不能"」。§8 S0-2 的判据随之改成状态依赖的负例（F2 用例 3）。

---

### F7 cross-floor 与分层假设 —— **红**

**缺陷 1：跨层边没有类型，`Vec<MoveEdge>` 装不下。**
`cross_edge: Option<(target_floor, went_down)>`（`solve_local.rs:337`）在全局侧被
`ExitKind::{Stair,Script,Cloud,UseItem}` + `landing()` 消费（`crates/game-solver/src/solve_global.rs:445-475`），
`Cloud` 还有区间门 `lo/hi`（`:1943-1970` 取 `st.floors` 极值）与"必须邻梯"前置。spec 的 `MoveEdge`
（未定义内部结构）如果不带 `CrossFloor{target, went_down, cond}`，S0-4 的 `上行符_f45到f46` 无法落地。

**缺陷 2（致命）：全局预算没有账本，spec 的 API 会被实现成"每层各自结算"。**
- 现状全局态已经带背包：`solve_global.rs:116-145` 的 `GKey.bag = CanonicalBag::from_player(&st.player)`，
  `state.rs` 的 `CanonicalBag{types,uses}` ⇒ **道具持有与剩余次数已经是全局键的一部分**。
- spec 却引入 `Budget::from_inventory() -> BTreeMap<CapKind,u32>`（`spec:171`）作为新的并行结构，
  而**没有任何一处说这个预算在哪一层扣减**。若它只在 `edges_from_caps`（逐层）里读、在 `doors.rs`
  里判，就会出现"每层都从满预算开始"⇒ 一枚 18 拆遍全图所有层的墙。
- 这是本 spec 最深的结构漏洞：它把"**判据口径**（有/无忧什么能力）"和"**搜索状态**（此刻持有什么）"
  混成了一个类型。

**修法**
1. 拆成两个概念并写进 spec：
   - `CapKey`（判据用，含 target）：`doors`/`keyflow` 的 `--caps` 口径参数，形如 `BTreeSet<CapKey>`；
   - 状态侧：不加新结构，直接用 `CanonicalBag`（已在键里），能力边按 bag 现算（F6 签名）。
2. `CapKey` 必须含 `target`（否则 18 与 85 同桶，见 F5 缺陷 3），例如
   `enum CapKey { Obstacle{scope,target}, Mirror, Floor(i32), HealAttr, Passive(kind) }`。
3. 明确「预算扣减发生在搜索转移上（`consume_item` 语义），不在判据层」；并加一条**跨层负例**：
   `同一枚18_只用一次_跨两层时第二层不再生成拆墙边`。

---

### F8 可测试性 —— **黄**

S0-0..S0-5 的判据名存在，但 4 条断言对象不明确、2 条当前**不可能执行**：

| 判据（spec 行） | 问题 | 改写建议（可直接照写） |
|---|---|---|
| `caps_every_action_classified`（:185） | 括号里要求断言"覆盖 20/21/22/17/18/25/19/29-32"，其中 25/29-32 **不是 `ItemAction`**，断言写不出来 | 拆两条：`caps_snapshot_item_set_eq(&[13,14,16,17,18,19,20,21,22,85,86])`（若采纳 F1 修法 1）+ `caps_all_item_actions_classified`（遍历 `ITEM_EFFECTS` 断言每个 action 有 cap） |
| `caps_snapshot_has_no_duplicate_item`（:186） | `id` 唯一是最弱断言（BTreeMap 化后恒真） | 改为 `assert snapshot.items == expect`（整表快照，含 caps/charges），并断言 `evidence` 非空、id 升序 |
| `新增道具_用已有原语_求解器零改动`（:141） | "断言求解器将其当作破墙预算 2 使用"——断言对象未定义 | `let b=Budget::from_inventory(&inv,&fake); assert_eq!(b[&CapKey::Obstacle{scope:Adjacent,target:11}],2);` + 合成 3×3 层 `assert!(reachable_after == true)` + 负例 `inv` 为空时 `assert!(edge 不存在)` |
| `s0b2_破障边与旧硬编码等价`（:201） | **无对象**（旧硬编码不存在，§1 自证 grep=0） | 删；改 `s0b2_f37_持有18时_36面墙可拆_由不可达转可达`（f37 墙数 36，普查实测） |
| `s0b2_上行符边存在`（:202） | 只写"f45 (2,10)"，未给期望边类型/目标层 | `assert_eq!(edges, vec![MoveEdge::CrossFloor{to:46, went_down:false}])`，并断言未持有 21 时为空 |
| `单格破墙预算1可通`/`两格破墙预算1不可通`（:210） | 无层/坐标/预算键 | 给定合成层（如 5×3，墙在 (1,1)(2,1)）、`CapKey::Obstacle{Adjacent,11}`、预算 1，断言 `reachable` 布尔 |
| `钥匙计数`（:211） | 断言对象模糊（需求？供给？余额？） | `assert_eq!((demand_yellow, supply_yellow), (29, 176))`（数值随 F9 校正后定） |
| `楼梯终审豁免地形`（:211） | 未给坐标；"f23 上行梯坐不可走地形"未验证 | 给 `(floor, x, y)` 与 `terrain=false` 前提，断言 `passable==true`，并加同格去梯负例 |
| `相形变位_可落在门后并触发交互`（:216） | "门后"未定义 | 给定 f/坐标与落点门的 locked 状态，断言 `pass` 由 `interact_with_cell` 终审决定（`item_effect.rs:119-124`） |
| `f40_无镜像边`（:216） | 只有正例 | 加 f39 上**有**镜像边（`item_effect.rs:116-118` 是唯一特例，两向都要钉） |
| `下行符_无来源仍建模`（:217） | 可执行，但没写断言 | `assert!(snap.def(22).is_some() && snap.def(22).charges == Charges::N(1))`（世界 0 处，普查实测） |

---

### F9 与既有 findings 的一致性 —— **红**

**缺陷 1：`33→32` 挂错了机制，且 S0-3 的文件表里没有产生它的改动。**
- 现状 33 = 29 黄 + 4 红：复跑 `./target/release/solver doors --out /tmp/rev_doors.json`
  → `summary {"doors":309,"necessary":33,"color_needed":176}`，逐色 `necessary {黄:29,红:4}=33` ✅ 与
  `docs/findings/2026-11-17-mandatory-doors.md:20-27` 一致。
- 但 32 的来源是**另一份**提案：`docs/superpowers/proposals/2026-11-17-solver-correctness-remediation.md:235`
  「S2 doors 消费 `cell_gate`…预期：必经 33→32（**唯一变化 f14 黄 (8,11)**）」——机制是 **ty6 喷泉格=通道**，
  与"破墙预算"无关。
- spec `:212` 把 32 写成 S0-3 的预期，而 S0-3 的文件是 `reach.rs`（新）+`doors.rs`，
  **不含 `cell_gate`/ty6 语义** ⇒ 照切片做完，doors 仍是 33。"预算=B0（不可破障）"更与"数字要变"自相矛盾。
- 且该提案**未入库**：`git status --porcelain` → `?? docs/superpowers/proposals/2026-11-17-solver-correctness-remediation.md`
  ⇒ spec 的数字在库内**没有证据链**（违 AGENTS §7「产出归宿是仓库」）。

**缺陷 2：`180→176（172 黄/4 红）` 是把文档笔误当成"现状"。**
```
$ python3 - <<'EOF'  # 解析 /tmp/rev_doors.json
necessary {黄:29,红:4}=33
doors-in-needed-color {黄:172, 红:4} = 176
EOF
```
- 工具实际是 **172 黄 + 4 红 = 176**；`mandatory-doors.md:27` 却记成「176 黄 / 4 红 / 0 蓝 = 180」
  （把 summary 的 `color_needed=176` 总数当成了黄门数）。⇒ "现状 180" 是**文档错误**，
  "172 黄"是**今天就有**的值，spec 把它写成"将来会变成"。
- `solver doors` 的人读输出同样是 `同色必要 176`（无逐色分解）——这正是笔误的源头，建议工具侧直接打 172/4/0。

**缺陷 3：`10 个道具定义` 与括号里的 11 个 id 自相矛盾**（`spec:187`）；且这 11 个里 25/29/30/31/32
根本不在 `ITEM_EFFECTS`（F1 缺陷 1）。

**缺陷 4：`17/18/20/21/22/25 均以字面量出现`（`spec:19`）4/6 为假**（见 F2 的命令输出）。

**缺陷 5：可得性披露不全，且把 17 当成"可用动机"。**
- `spec:246` 只写「20 0 处、21 仅 f45、22 0 处」。实测：**17 也是 0 处、且无商店供给**；
  **18 也是 0 处，但商店有 1 处**：`data/tables/shop_goods.json` 第 17 行
  `{"src":1,"idx":13,"price":4000,"effect":{"key":"PickupItem","params":{"item":18}}}`
  （= f47 的 ty78 idx13 开价店：`docs/findings/2026-11-17-shop-flags-and-economy-ceiling.md:26`）。
- 后果：spec §1 的动机（"17/18 破障没建模 ⇒ 结论方向错"）在当前数据上**没有实例**：
  17 无法入手（地图 0 + 商店无），18 需 ¤4000 且只在 f47（是否凑得齐 **未验证**；
  已知 f12 处上限 767 见 `docs/findings/2026-11-17-queries-after-f2.md:32`，
  且同提案 `:243` S6 的预期只是「f37 红钥 Gated→可达（仅买石分支）」。主路径数字不变」）。
  ⇒ 破障建模的正确预期是"**几乎不改变任何现结论**"，不是"结论方向错"。
- `红钥 6→5+1 条件可得`（`spec:223`）：`key-flow.md` 现状 = 需求 4 / 拾取 6（f8/f14/f34/f37/f42/f51），
  spec 未指明哪一个变"条件可得"，也未给 6→5+1 的推导。按提案 `:50` 应写"f37 的那一枚需回溯 f47 买石"。

**修法**：所有数字改为"命令 + 输出"引用（`./target/release/solver doors` /
`python3 tools/type_census.py`），并把 33→32 的功劳归还 `cell_gate` 切片（或从本 spec 删除该预期）。

---

### F10 过度设计 / 欠设计 —— **黄**

**YAGNI（建议删）**
1. `Passive{kind}` + 25（`spec:80/107`）：25 全世仅 f0 一枚（普查），`Passive` 分支无消费者。
   删到 §10"明确不做"，或保留类型但不在 S0 落地。
2. `CapDef.evidence: String`（`spec:91`）**进入 dump**：它让基线对"文档行号变更"敏感
   （改一行 findings 就红门禁），而它不承载语义。改成 `#[serde(skip)]` 或放旁表。
3. `Amount::StatScaled{from:(StatKind,StatKind)}`（`spec:84`）：全库只有一个用户（19），
   二元素元组是空泛化；且真正缺的是 `TierScaled`（F5 缺陷 5）。
4. `Resource::Item(i32)` / `CapKind::GainItem`：`ITEM_EFFECTS` 里没有任何动作产出道具（加栈是拾取面）。
5. §11 假设 3「`RemoveAllType{11}` 是否只作用于当前层」——**现在就能定论**：
   `item_effect.rs:86-98` 遍历 `self.entities`（当前层实体表）并 `build_minimap()` ⇒ 作用域=当前层。
   不必留待证。

**致命缺失（必须补）**
1. 两张执行表（`ITEM_EFFECTS` vs `PICKUP_ARM_TABLE`）的**权威裁决规则**（F1）。
2. `KillAdjacent4`(86) / `OpenPanel`(13,14) 的原语或白名单（F5）。
3. `CapKey` 带 `target`，否则预算错误合并 18 与 85（F5/F7）。
4. `Amount::TierScaled` + `HpUp` 是当前 HP 而非 MaxHp（F5）。
5. 能力边签名带状态 + 全局预算记账位置（F6/F7）。
6. S0-1 的 `crates/game-data/src/defs.rs` + `tools/item-defs/gen_items.py`（F4）。
7. 14/15/23/85/86 的快照覆盖或白名单 + 理由（F3）。
8. `caps_every_action_classified` 的断言对象与 `s0b2_破障边与旧硬编码等价` 的重写（F8）。

---

## 必须改的 N 处（按严重度排序，可直接照着改 spec）

| # | 改哪里 | 改成什么 | 依据 |
|---|---|---|---|
| 1 | §5.1 表格第二行 + §4 `charges` 来源 | 删除"参数/次数来自 items.json"。参数取 `ItemAction` 载荷；次数用 `ITEM_CHARGES`（由 `effect_registry.rs:201` 提升，含 `Infinite`）。items.json **不得**被语义面读取 | `item_effect.rs:13-19`；`ingest.rs:3-5` 红线；BUG-053 |
| 2 | §4 `CapsSnapshot` 的推导输入 | 明确 `ITEM_EFFECTS`（使用面）与 `PICKUP_ARM_TABLE`（拾取面）的**关系与仲裁**；对 22 定论 `ChangeFloorDown` | `effect_registry.rs:117-120` vs `item_effect.rs:44` |
| 3 | §8 S0-0 预期集合（`:187`） | 改为 `[13,14,16,17,18,19,20,21,22,85,86]`（+ 若要拾取面则另列 29/30/31/32 并给 `TierScaled`） | `item_effect.rs:33-45` |
| 4 | §4 `CapKind` → 判据键 | 判据键必须含 `target`：`CapKey{Obstacle{scope,target},...}`；删"不带参分类"的说法 | `item_effect.rs:38-39`（18 vs 85） |
| 5 | §7 `edges_from_caps` 签名 + §6/§7 记账 | 加 `bag:&CanonicalBag`；写明"边随状态生成"；写明预算在**搜索转移**上扣减（全局唯一账本） | `solve_local.rs:1945-1999`；`state.rs` `CanonicalBag`；`solve_global.rs:128` |
| 6 | §4 `Amount` / `StatKind` | 加 `TierScaled{base}`；`HpUp`→`Gain{Hp, TierScaled}`；删 `MaxHp` | `effect_registry.rs:157-166,238-281` |
| 7 | §5 原语清单 | 加 `Clear{Adjacent4}`（86）、`Ui{panel}`（13/14）或把 13/14 列白名单 | `item_effect.rs:10,18` |
| 8 | §6 豁免判据（`:158`） | 删 `category` 字段说法；改 `rules/caps.rs` 显式常量（装备 33-40/79/80、钥匙 26-28）并写理由 | `defs.rs:568`；`items.json` 实测 effect.key |
| 9 | §8 S0-1 文件表（`:191`） | 加 `crates/game-data/src/defs.rs`、`tools/item-defs/gen_items.py`、`crates/game-data/tests/items_defs.rs`；风险等级由"中"升"高" | `defs.rs:571-580`；`items_defs.rs:32-36` |
| 10 | §8 S0-3 预期（`:212`）与 S0-5（`:223`） | 删 `33→32`（归 `cell_gate` 切片）；`180→176` 改为"现状实测 172 黄/4 红=176，本片不变"；红钥改"f37 那枚需 f47 ¤4000 买石（条件可得）" | `solver doors` 实测；`mandatory-doors.md:27`；提案 `:235` |
| 11 | §1 证据表第 3 行（`:19`） | 改为"15/21/22/23 以字面量出现（`state.rs:40`、`solve_local.rs:1945,1997`）" | grep 实测 |
| 12 | §5.3 护栏 4 + §7 末条 | 注入用例从 1 条扩到 6 条（F2 清单）；"源码无道具 ID"升级为**强门禁** | F2 |
| 13 | §8 各片判据 | 按 F8 表逐条补"断言对象/期望值"；删 `s0b2_破障边与旧硬编码等价` | F8 |
| 14 | §11 假设 3 | 删除（已可定论：当前层，`item_effect.rs:86-98`） | 同左 |

---

## 不必改但要知道的

1. **判据方向是对的，别把它一起推翻**：`solve_local.rs:1262`（`walkable || stair_like`）与
   `doors.rs:115`（`terrain_ok || goal || entry`）确实并存两套"能否进入"口径，且 spec 引的两处行号
   **准确**（我逐行核对过）。只是"后果"被夸大（见 F9 缺陷 5）：真正的 33→32 来自 ty6 通道语义。
2. **`scene.rs:3674-3676` 楼梯豁免承重属实**（`if allowed && stair_count <= 0`），S0-3 的
   `楼梯终审豁免地形` 用例可写；f23 那格的地形值我**未验证**（未逐层 dump 地形）。
3. **74 金样与 items.json 无耦合**：`tools/state_baseline.py:19` 只跑三个 dump 场景，
   所以"加 charges ⇒ 74 sha 全同"是**必然**，不构成证据（见 F4）。
   spec 若想用金样证明"零行为"，应改引 `cargo test -p game-core -p game-data` 的用例面。
4. **`charges` 不是唯一路径**：`ITEM_USES` 已是规则层常量，提升它比改 items.json 少碰 4 个文件、
   0 个门禁、0 个 schema 变更。若目标只是"次数进快照"，这是更短的桥（spec 未考虑，值得写进 §10 备选）。
5. **求解器已把道具持有放进全局去重键**（`solve_global.rs:128` `bag: CanonicalBag`），
   所以"零改动"承诺的一半**已达成**：新增道具实例不需要改键。真正要改的只是**边生成**与**判据口径**。
6. **`input.rs:79` 直接 `parse_items(include_str!(items.json))`**：任何表字段改动都会第一时间在
   `ingest` 炸，不会静默——这是好事，但会让 S0-1 的失败看起来很"突然"。
7. **[未验证] 的三个面**：f23 上行梯地形值；¤4000 在全局搜索中是否可达（需跑 `query --all`，
   spec 自己说约 4 分钟）；给 29-32 加 `ItemAction` 变体会不会动 74 金样（需实跑）。
8. **两张既有 findings 的 176 冲突**（`mandatory-doors.md:27` vs 提案 `:235`）建议**就地更正**，
   并在 `docs/findings/` 留一行"2026-11-17 更正：同色必要逐色为 172/4/0"，否则后续 spec 会继续引用错值。

---

## 两句话回答

**(a) 这份 spec 现在能不能进实施计划？**
不能：它的四条承重事实（`ITEM_EFFECTS` 覆盖 25/29-32、参数来自 items.json、次数只在 desc、装备有 category 字段）
与仓库现状逐条相反，按它写实施计划会在 S0-0/S0-1 直接红，并被"就地修补"成它声称要消灭的第二份手写语义。

**(b) 最小必改清单：**
① 语义来源改为单一（参数取 `ItemAction`，次数提升 `ITEM_USES`，**禁止**再读 items.json 的 effect）；
② 快照集合与豁免改成实测事实（+13/14/16/85/86/-25/29-32 或另立拾取面，装备/钥匙用显式常量豁免）；
③ 判据键加 `target`（18 与 85 不同桶）+ `Amount` 加 `TierScaled`（HpUp 是当前 HP×tier）；
④ 能力边签名带 `CanonicalBag` 并在全局账本扣减（不得逐层各算）；
⑤ S0-1 文件表补 `defs.rs`/`gen_items.py`/`items_defs.rs`，S0-3/S0-5 的数字（33→32、180→176、红钥 6→5+1）
按实测与 `cell_gate` 归属重写。改完这 5 条，方向可以原样保留进 writing-plans。
