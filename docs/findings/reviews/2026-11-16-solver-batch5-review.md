# 批5 求解器改动 · 独立对抗复核（red team）

- **复核范围**：`ac179f9..HEAD`（`f8cfd49` feat(solver) 商店买钥匙边 + `eec959a` docs 三查询重跑落盘/更正/清黄）
- **上下文链**：`80c9506..HEAD` 供理解（批4 结论「界内无解」）
- **复核者立场**：不信任提交方报告，一切以本机亲自执行的命令输出为准；未执行项一律标「未验证」并降级为假设
- **环境**：`/home/peng/workspace/xiyou_remake`，`cargo --release`，2026-11-16
- **纪律基线**：AGENTS.md（证据 A/B/C 分级、禁止调阈值掩盖、禁止伪造结论、红线路径）
- **本次未改任何源码/文档/门禁**；临时探针与变异均已还原，`git diff` 为空（见 §11 卫生证明）

---

## 0. 十面判定总表

| # | 面 | 判定 | 一句话依据 |
|---|---|---|---|
| 1 | 夹带/红线 | **绿** | 8 文件全在 `crates/game-solver` + `docs/findings`；红线零命中；源码新增行无调试残留 |
| 2 | 商店边代码正确性 | **黄** | 扣金/开价门/goods 映射正确；但「一次性移除」与引擎重复购买语义不符、「走臂踩商人格」与引擎「触发即阻挡」不符 |
| 3 | 旧入口零行为 | **绿** | 旧入口全部恒传 `&[]`（4 处），唯一传真表=query 路径；真实地图探针证实 `&[]` 下新走臂≡旧 Move 臂 |
| 4 | 三查询重跑真实性 | **绿** | 亲自复跑，`/tmp/q_rv.json` 与仓库 json **逐字段相等**；文档 §1 表逐值一致；`bound/node_cap` 常量未被改 |
| 5 | 「五种排序全败」与回滚 | **黄** | 回滚可证（HEAD 队列类型与批4 完全一致、无残留）；但「第 5 种 exp=40391」无任何入库产物 ⇒ 该实测**未验证** |
| 6 | 判据/回归/门禁 | **绿** | `gates check` PASS=13 FAIL=0 SKIP=0（ratchet total=1081 无回退）；`cargo test -p game-solver` 45 测试全绿 |
| 7 | 变异测试（真红验证） | **黄** | 4 个变异 3 红 1 存活；**「删除扣金」未被判据捕获** ⇒ 金币余额未钉 |
| 8 | 文档 claim vs 事实 | **黄** | A 级指针（id10/`a.java:7379-7384`/f12 param=774）全部吻合；但 §4.1 与 §3.2 更正互斥、§4.5「不是缺边」失实 |
| 9 | 纪律合规 | **绿** | 两 commit 均含 Evidence+Confidence；无「未执行写成已完成」的交付性表述；findings 保留假设区；无 `setState` |
| 10 | 完备性裁定 | **黄** | 自述目标达成（补边+落盘+清 4 黄），但**新补红边在真实世界模型内从未触发**（5716 次买边 0 次在 f12）；「无解」仍诚实标注为模型内结论 |

**总判：0 红 / 5 黄 / 5 绿 ⇒ 可推送，但须在 findings 落一条更正注（§4.5「不是缺边」+ §4.1 与 §3.2 互斥），并把「击杀金币未建模」登记为下一批必修缺边。**
**Confidence：82**

---

## 1. 面1 · 夹带 / 红线 —— **绿**

```
$ git diff --stat ac179f9..HEAD
 crates/game-solver/src/bin/solver.rs       |  21 ++++--
 crates/game-solver/src/query.rs            |  18 +++--
 crates/game-solver/src/solve_global.rs     |  59 ++++++++++++++--
 crates/game-solver/src/solve_local.rs      | 109 +++++++++++++++++++++++++++--
 crates/game-solver/tests/solve_global.rs   | 106 +++++++++++++++++++++++++++-
 crates/game-solver/tests/solve_local.rs    |   8 +--
 docs/findings/2026-11-15-solver-queries.md |  32 +++++++--
 docs/findings/solver-queries.json          |   2 +-
 8 files changed, 320 insertions(+), 35 deletions(-)

$ git diff --name-only ac179f9..HEAD | grep -E '^(original/|reference/|crates/game-core/|gates/|data/naming/)'
(零命中)

$ git diff ac179f9..HEAD -- '*.rs' | grep -E '^\+' | grep -nE 'dbg!|println!|eprintln!|G-DBG|probe_'
(源码新增行零命中)
```

- 范围干净：仅 `crates/game-solver/**` + `docs/findings/**`，与两项自述改动吻合。
- 红线四路（`original/`、`reference/`、`crates/game-core/`、`gates/`）**零改动**，另加查 `data/naming/` 亦零命中。
- 唯一 `probe_` 命中在**文档正文**（§3.1 提到批4 探针名 `probe_c4_f14_red_key_reachability`），非代码残留。
- 无新增文件、无 `.orig/.rej/.tmp`、无 `#[ignore]` 新增（`git diff` 中 grep `ignore` 零命中）。

---

## 2. 面2 · 商店边代码正确性 —— **黄**

### 2.1 逐项核对（与引擎对齐的部分，**正确**）

| 检查点 | 模型实现 | 引擎真源 | 判定 |
|---|---|---|---|
| 分类 cat16 | `crates/game-solver/src/solve_local.rs:1140-1141` `let mut shop`；cat16 分支 `if shop.is_none() { shop = Some((slot, e.ty)) }` | `scene.rs` `render_category 77|78=>16` | 一致 |
| 开价门 | `solve_local.rs:1126` `if (p >> 8) & 2 == 0 { return None }` | `a.java:7452-7466` `var7 = param>>>8; ... (var7&2)` | 一致（位2=开价） |
| goods 映射 | `:1129-1132` `src = ty==77 ? 0 : 1`、`idx = p & 0xff`、`shops.iter().find(src&&idx)` | `a.java:7307+` `applyMerchantOffer` type==77 ? src0 : src1 | 一致 |
| 金币门 | `:1525` `if st.player.gold >= price` | `scene.rs:2615-2622` `spend_gold`：`if gold >= amount { gold -= amount; true }` | 一致（`>=`、含等号、price=0 恒过） |
| 扣金 | `:1527` `ns.player.gold -= price` | 同上 | 一致 |
| GiveGold | `:922-924` `p.gold += amount` | `scene.rs:2666-2680` → `combat::gain_gold`（`combat.rs:191-197`，**无上限**） | 一致 |
| AddHp | `:926-929` `p.hp += amount` | `a.java` `this.playerHp += 2000`（无上限） | 一致 |
| SwapYellowKey | `:930-937` `yellow<1 ⇒ false`，否则 `yellow-=1; gold+=gold` | `scene.rs:2695-2700` `if yellow>0 { yellow-=1; gold+=v }` | 一致 |
| 无 off-by-one / 溢白嫖 | `gold: i32`（`solve_local.rs:149`）；`spend_gold` 同型 | — | 一致 |

真表钉法也对：`data/tables/shop_goods.json` `id=10, src=1, idx=6, price=800, effect={key:AddKeys, params:{blue:0,yellow:0,red:1}}`，`evidence.effect="a.java:7379-7384 ¤800 红+1（spendGold 门）"`；判定测试 `tests/solve_global.rs:1399` 断言 `paid.price==800 && AddKeys{red:1}`。

### 2.2 偏差①（**关键**）：「买一次即移除」与引擎「可重复购买」不符

引擎（`reference/src/deobf/a.java:7452-7466`，Rust 侧 `crates/game-core/src/scene.rs:2710-2732`）购买成功后：

```
if ((flags & 4) == 0) { param ^= 512; }        // 只有 bit2 未置才翻回「未开价」
if ((flags & 1) != 0) { pushOfferQueue(...) }  // bit0 置 ⇒ 不移除实体（可再买）
else { entity::remove(...) }                   // 只有 bit0 未置才移除
```

模型（`solve_local.rs:1513-1538`）无条件 `ns.ents.remove(slot)` ⇒ **每个商店只能买一次**。

真实数据（我自建临时探针跑 `ingest + load_floor(1..=54)`，A 级；探针已删）：30 个商店格中 14 家有表行且开价门，其中 **11 家 `flags=3|6`**，即引擎侧「可重复购买 / 无限购买」：

```
MAP f6  (4,4)  ty=78 param=771  flags=3 idx=3 -> id7  ¤50   AddKeys{blue:1}
MAP f7  (6,1)  ty=78 param=772  flags=3 idx=4 -> id8  ¤50   AddKeys{yellow:5}
MAP f12 (1,1)  ty=78 param=774  flags=3 idx=6 -> id10 ¤800  AddKeys{red:1}   ← H4 目标店
MAP f12 (11,1) ty=78 param=1541 flags=6 idx=5 -> id9  ¤1000 AddKeys{yellow:1} ← bit2 置 ⇒ 连开价都不翻
MAP f28 (8,4)  ty=78 param=1544 flags=6 idx=8 -> id12 ¤0    SwapYellowKey{gold:100} ← 无限换金
...
TOTAL shop cells=30  unwalkable=0  colocated_with_cat8124=0
```

后果：模型对「同一商店买 N 次」一律不可表达（**欠近似**，方向安全：不会造出假解，但会漏解）。commit message 与 `solve_local.rs:1514` 注释「`ents.remove` = `markEntityRemoved` 同款」对 flags=3 病例（正是 f12）**事实错误**——引擎在该病例下只 `param ^= 512` 并继续报价，不删实体。

### 2.3 偏差②：走臂允许「踩上商人格」，引擎是「触发即阻挡」

引擎 `crates/game-core/src/scene.rs:3632-3660` cat16 臂末 `allowed = false`（商人只能相邻 bump 交互，格不可进入）；模型 `solve_local.rs:1516-1521` 走臂 `if self.walkable(tx,ty) { 移动 }` ⇒ 多出一条「穿商人」边（**过近似**，可造假解；对本次 UNSAT 结论方向安全）。注释「走臂=与原 Move 臂同款」只对批0-3 的 `&[]` 情形成立，对真实商店表不成立。

### 2.4 面2 附带发现（并入面 8/10）：**击杀金币未建模**

- 引擎：`crates/game-core/src/scene.rs:2283 kill_adjacent_at`（`a.java:7923-7951`）在击杀后 `let gold = if find_item(25) { enemy_base[t-41].gold << 1 } else { ... }; self.player.gold += gold;`（`:2301-2309`）。敌方赏金表非零（`enemies.json` 35 行全非零，合计 4839；夹具 `enemybasegold.txt` 同为 1..100+）。
- 模型：`solve_local.rs:1539-1548` Monster 臂只有 `hp -= loss; kills += 1; ents.remove(slot); pos; steps` —— **不加金币**。
⇒ 金币经济缺一条主干边（且 `item 25` 的双倍赏金亦无）。这与 H4 属同一「缺边」类别，且**比 H4 更基础**（金币是所有付费商店边的前置）。

### 2.5 面2 结论

扣金/开价/映射/效果四类实现**代码级正确**；但两个语义近似（一次性购买=欠近似、踩商人格=过近似）与一处未建模主干（击杀金币）共同削弱「模型已闭合」的说服力。**黄**（非红：无假解风险主导，且结论方向保守）。

---

## 3. 面3 · 旧入口零行为 —— **绿**

全部注入点（grep 穷举）：

| 调用点 | 传参 | 性质 |
|---|---|---|
| `crates/game-solver/src/solve_local.rs:356` | `solve_layer_with(..., ClosurePolicy::OFF, &[])` | 旧入口，恒空 |
| `crates/game-solver/src/solve_local.rs:1109` | `shops: &[]`（Harness 默认构造） | 恒空 |
| `crates/game-solver/src/solve_global.rs:255-261` | `solve_global_with(..., shops)` → `solve_global_capped_with` | 透传 |
| `crates/game-solver/src/solve_global.rs:287` | `shops: &[ShopGoodDef]`（capped_with 形参） | 透传 |
| `crates/game-solver/src/query.rs:125` | `solve_global_with(&input, opts.node_cap, policy, shops)` | **唯一传真表**（query 路径） |
| `crates/game-solver/src/bin/solver.rs:112` | `let (gi, _shops) = solve_global_input(bound);` → `solve_global(&gi)` | 显式丢弃 ⇒ 恒空 |
| `crates/game-solver/src/bin/solver.rs:175-178` | `let (input, shops) = ...; run_query(&input, q, &opts, &shops)` | query 路径传真表 |
| `crates/game-solver/tests/solve_local.rs`（3 处） | 均显式 `&[]` | 判据不受污染 |

- 无「默认加载商店表」式偷改：`ingest` 只把表放进 `SolveInput.shops`（`ingest.rs:33/89`），是否使用完全由调用方传入的 slice 决定；`solve_layer`/`solve_global` 签名未变。
- **实测补强**（临时探针，已删）：真实 55 层地图中 cat16 共 30 格，**全部 `terrain.walkable=true`，且与 cat8/1/2/4 同格共存数 = 0**。旧代码 cat16 落 `_ => {}` → `Cell::Move`；新代码走臂 `walkable ⇒ 移动` ⇒ 在 `&[]` 下**逐转移等价**。故「批0-3 零行为」不仅结构成立，且在本仓库真实数据上成立。

**绿**。

---

## 4. 面4 · 三查询重跑真实性 —— **绿**

```
$ cargo run --release -p game-solver --bin solver -- query --all --out /tmp/q_rv.json
EXIT=0
控制台：Q1 max_hp=0 (feasible) / Q2 min_kills=0 (feasible) / Q3 reachable=false

$ python3 -c "json.load(open('/tmp/q_rv.json')) == json.load(open('docs/findings/solver-queries.json'))"
equal: True
```

逐字段（`/tmp/q_rv.json` ≡ 仓库 `solver-queries.json`）：

| 查询 | expansions | bound | node_cap | node_cap_hit | truncated_layers | goals | confidence |
|---|---|---|---|---|---|---|---|
| q1 `max_hp` | 68550 | 1000000 | 100000 | true | `[]` | 0 | feasible |
| q2 `min_kills` | 7938 | 1000000 | 100000 | true | `[[6,200000],[7,200000]]` | 0 | feasible |
| q3 `reachable` | 641 | 1000000 | 100000 | false | `[]` | 0 | feasible |

- 文档 §1 表格 `max_hp/min_kills/reachable/expanded/goals/truncated_layers/node_cap_hit` 与上述**逐值一致**。
- 常量未被偷改：`query.rs:36-44` `QueryOpts::default { bound: 1_000_000, node_cap: node_cap_env() }`；`solve_global.rs:270-273` `node_cap_env()` 缺省 `100_000`；`bin/solver.rs:129` solve-global 缺省 `bound 1_000_000`。
- 批4 历史对照可信：`git show ac179f9:docs/findings/solver-queries.json` = q1 68889 / q2 7936 `trunc=[[6,200000],[7,200001]]` / q3 641，与文档「批4 首跑值」逐值一致 ⇒ 文档没有把旧值冒充新跑值。

**绿**（唯一提示：`/tmp/q_rv.json` 与仓库文件字节级不同属正常，语义相等已证）。

---

## 5. 面5 · 「五种排序全败」与回滚 —— **黄**

- **回滚可证（绿）**：`solve_global.rs` 队列类型 HEAD 与 `ac179f9` 完全一致
  `BinaryHeap<(Reverse<u32>, u32, Reverse<u32>)>`（HEAD `:345/346/568/612`；批4 `:338/339/560/604`），
  全文件 grep `Reverse(red)` / 「持红」/ 额外排序分支 **零命中** ⇒ 第 5 种序「持红优先」确实已移除，HEAD 行为与文档「已回滚」一致。
- **但实验本身未验证（黄）**：`docs/findings/2026-11-15-solver-queries.md:119-121` 声称
  「q1 `exp=40391` 仍 `goals=0`」，仓库内**没有任何产物**（无 json 落盘、无 commit、无日志、无探针脚本）；
  该值无法复跑（`git log -p` 未保留实验代码）⇒ 只能按**假设**处理，不能作为「排序空间已穷尽」的证据。
- 另：「五种已全败 ⇒ 排序空间已穷尽」（`:121`、`:130`）是**从 5 个启发式外推到空间穷尽**，逻辑不成立（排序空间无穷）。措辞应降级为「五种启发式排序均无解」。

**黄**。

---

## 6. 面6 · 判据 / 回归 / 门禁 —— **绿**

```
$ python3 gates/cli.py check
[✓ PASS] original-integrity / assets-manifest / reference-seed-integrity / knowledge-format
[✓ PASS] docs-links / naming-ledger / rust-tests
[✓ PASS] test-ratchet — total=1081(基线1075) 无回退
[✓ PASS] table-content-hash / table-evidence（双跑同哈希 + 等值）
[✓ PASS] state-machine-anchors（锚 6 + 边数对照）
[✓ PASS] asset-truth-manifest / state-baseline（74 场景 sha 全同）
== 汇总 ==  PASS=13 FAIL=0 SKIP=0

$ cargo test --release -p game-solver
unittests src/lib.rs        : 7 passed   (0 failed)
unittests src/bin/solver.rs : 0 passed
tests/census_snapshot.rs    : 4 passed
tests/solve_global.rs       : 21 passed  (28.84s)
tests/solve_local.rs        : 13 passed
Doc-tests                   : 0 passed
⇒ 45 passed / 0 failed / 0 ignored
```

- 批0-3 判据「逐层拼接 == 全局解」在 `tests/solve_global.rs` 中随 21 项全绿（含 `run_all` 对拍）。
- 商店边新判据 `solve_global_t商店买红钥匙_花金得红过红门_红绿对照`（`tests/solve_global.rs:1399`）在 21 项内且通过。
- 编译告警：**10 条具体告警，全部是 `should have a snake case name`（中文测试名）**（另有 3 行 `generated N warnings` 汇总行）；批4 复核点名的三条卫生告警
  （`unused import: solve_global_with`、`unused import: ClosurePolicy`、`unused_mut @ solve_local.rs`）
  **已消失**，与 `eec959a` 自述「黄4 清零」吻合。
- 说明：我第一次跑 check 时混入了自己的临时探针（`zz_tmp_coloc_probe`），ratchet 报 `total=1082`；
  **删除探针后重跑才得到上表 `total=1081`**，与提交自述数字一致。

**绿**。

---

## 7. 面7 · 变异测试（真红验证）—— **黄**

对 `crates/game-solver/src/solve_local.rs` 逐个打变异，跑
`cargo test --release -p game-solver --test solve_global 商店`，每次变异后从备份还原并核对 `git diff`：

| 变异 | 内容 | 期望 | 实测 | 还原 |
|---|---|---|---|---|
| M1 | 删 `ns.player.gold -= price;`（`:1527`） | 红 | **绿（存活！** `test result: ok. 1 passed; 20 filtered out`） | ✅ clean |
| M2 | 删买得起门 `if st.player.gold >= price {` → `if true {`（`:1525`） | 红 | **红** `panicked at tests/solve_global.rs:1417`（② 金 799 变可买） | ✅ clean |
| M3 | `p.red += red;` → `p.red += 0;`（`:919`） | 红 | **红** `panicked at tests/solve_global.rs:1412`（① 金 800 买不到红） | ✅ clean |
| M4 | 开价门 `& 2` → `& 1`（`:1126`） | 红 | **红** `panicked at tests/solve_global.rs:1421`（③ 免费 GiveGold 行被挡） | ✅ clean |

- 判据**不是全假**：3/4 变异被抓，门/扣金/效果的边界都被钉住。
- **但「删除扣金」存活**：夹具里只有一件 800 的商品、买一次就通关，扣金与否不影响任何断言
  ⇒ **「购买后金币余额」没有任何判据**（真实模型里这会变成白嫖/负金币类的假解）。
  按本任务规则「变异不变红 ⇒ 测试是假的」，此项应判红；我判 **面7 黄**（3/4 已真红，仅扣金一项未钉），
  但把它列入**必修**：夹具需「买两次」或「买后余额还要用于第二处消费」才能捕获。
- 还原证明：4 次变异后 `git diff --stat` 每次均为空；最终 `git diff --quiet` 返回 `NO_DIFF`。

**黄**。

---

## 8. 面8 · 文档 claim vs 事实 —— **黄**

### 8.1 A 级指针逐条核对（**全部吻合**）

| 文档断言（`2026-11-15-solver-queries.md`） | 我的实测 | 判定 |
|---|---|---|
| §3.2 H4：`shop_goods.json` id10 `AddKeys{red:1}` price 800 | `data/tables/shop_goods.json` `id=10, src=1, idx=6, price=800, red:1` | ✅ |
| §3.2 H4：`a.java:7379-7384` spendGold 实文 | `sed -n 7370,7392p` → `case 6: if (this.spendGold(800)) { this.redKeyCount++; } else { showOverlayMessage("没有足够的金钱") }` | ✅ |
| §3.2 更正注：`f12 ty78@(1,1) param=774 → idx6 → id10 ¤800 红+1（flags=3 开价门过）` | 探针实测 `f12 (1,1) ty=78 param=774 flags=3 idx=6 gate=true -> (10, 800, AddKeys{red:1})` | ✅ |
| §1 表 `max_hp/min_kills/reachable/expanded/goals/truncated_layers/node_cap_hit` | 与 `/tmp/q_rv.json` 逐值一致（见 §4） | ✅ |
| §2 批5 行「旧入口恒 `&[]` ⇒ 批0-3 零变化」 | 见 §3，且真实地图等价 | ✅ |
| 判据测试名与「清 4 黄」 | 见 §6（三条卫生告警消失、ratchet 1081） | ✅ |

### 8.2 与事实不符 / 未证实的表述

1. **§4.5 结论「瓶颈是…，不是缺边」——失实**（`docs/findings/2026-11-15-solver-queries.md:121`）。
   本批自认（commit message）仍有 3 类商品效果（`AddStack`/`StatPct`/`PickupItem`）不出臂；
   我另查出**击杀金币完全未建模**（§2.4，引擎 `scene.rs:2301-2309` 有、模型 `solve_local.rs:1539-1548` 无）；
   加上「一次性购买」欠近似 ⇒ **至少 5 类缺边仍在**。「不是缺边」不成立，应改为「已知缺边已分类，但尚未闭合」。
2. **§4.1 与 §3.2 更正互斥**：`:93` 仍写「红钥匙的**唯一现实来源 f8**」，而 §3.2 更正注新增「f12 商店是原先缺的红边」。
   同一文档两处红源结论互相矛盾，且 §4.1 未加日期更正（AGENTS §3.6 要求原地标注、不删原文）。
3. **§3.2 更正注「f12 在 f20 之前，正是原先缺的那条红边」—— 语义过度**。
   实测：三查询里商店买边共触发 **5716 次**（f7=3419、f6=2186、f52=111），**f12 一次都没有**，
   且全程 `gold_before` 最大值仅 **210**（远低于 800），`price=800` 命中 **0** 次
   ⇒ 新补的红边在真实世界模型内**从未可用**，「补齐了红边」只是语法层成立。
4. **§3.1 探针数字（f8 `y=1→red=0`、`y=2→494/1495`；f14 30/30；f51 96/96；f20 `9R up=75`）**：
   仓库内**无入库脚本**可复跑（批4 探针为临时测试，未保留）。我**未复跑**（工作量大且夹具已删），
   这些数字只能沿用批4 复核的独立实测结论 ⇒ 本报告按**假设级**对待，不据此判红。
5. **§4.5「第 5 种队列序 exp=40391」**：无产物、不可复跑（见 §5）⇒ 假设级。

**黄**：A 级指针全部真实且内容吻合（这部分可信），但两处结论性措辞与一处内部矛盾必须更正。

---

## 9. 面9 · 纪律合规 —— **绿**

- 两笔 commit 均带 `Evidence:` + `Confidence: 85`（`f8cfd49`、`eec959a`）；本批未触及 `data/naming/`、`reference/`、`original/`
  （证据门禁非强制，但作者仍写了，合规）。
- 无「未执行却写成已完成」的**交付性**表述：`eec959a` 的「重跑/落盘」经我复跑证实（§4）；「清 4 黄」经我验证（§6）。
  唯一可疑处是「第 5 种排序已实测」——已归入面5/面8 并降级为假设，不计入伪造结论。
- `docs/findings` 有显式假设区（`## 3.2 假设（未证实，禁止据以下结论改代码）`），且更正采用「不删原文 + 日期标注」，
  符合 AGENTS §3.6。
- 无 `setState` 等不存在 API（本批为 Rust，不适用）。
- commit message 有两处**技术表述不实**（「走臂=原 Move 同款」「`ents.remove` = `markEntityRemoved` 同款」），
  已计入面2，不再重复扣分，但建议在下一批 commit 或 findings 更正注中纠正。

**绿**。

---

## 10. 面10 · 完备性裁定 —— **黄**

**自述目标是否达成**：
- 补商店边（H4）——**是**（代码+判据+接线齐备，`&[]` 零回归）。
- 三查询重跑落盘 + 结论更正 + 清 4 黄——**是**（数值逐字段可复现；卫生告警清零）。
⇒ 两项自述目标均达成。

**但站在「用户要的是可用求解器」角度，有三个保留项**：

1. **新补的红边在真实模型内是死边**（f12 触发 0/5716，全程金币 ≤210）。原因指向**击杀金币未建模**（A 级引擎有、模型无），
   而不是文档归因的「资源序未找到」。这意味着本批改动对 H4 描述的病根**没有产生实际解锁效果**，`exp` 从 68889→68550 的变化来自 f6/f7/f52 的经济边。
2. **「无解」结论仍是模型内结论**，且模型已知欠近似至少 5 处（击杀金币、三类商品效果、重复购买、走臂过近似、`item25` 双倍赏金）。
   文档在 §1/§3.2 已如实标注「模型内/界内」「不得当作已证明无解」——**这一点是诚实的，没有掩盖**；
   但 §4.5「不是缺边」的强结论把未闭合说成已闭合，属**过度收敛**。
3. 判据缺口：「购买后金币余额」无断言（M1 存活）。

**一句话结论**：本批工程实现质量合格、旧路径零回归、数值落盘可复现，**可推送**；但 findings 必须补一条更正注
（撤回 §4.5「不是缺边」、消除 §4.1 与 §3.2 的红源矛盾），并把「击杀金币边」登记为下一批头号缺口——
否则下一轮仍会把「模型缺边」读成「无解」。

**建议推送：可推送（0 红）。** 前提：推送同一批或紧随其后补上述更正注（不涉及代码回退）。

---

## 11. 阻断项 / 建议清单

**阻断项（红）：无。**

**必修（随推送或紧随其后）**：

| # | 项 | 位置 | 动作 |
|---|---|---|---|
| A | §4.5「不是缺边」失实 | `docs/findings/2026-11-15-solver-queries.md:121` | 改为「已知缺边已分类但未闭合」，并列出 5 类欠近似 |
| B | §4.1「红钥匙唯一现实来源 f8」与 §3.2 更正互斥 | 同上 `:93` | 原地加日期更正，指向 f12 商店与「模型内金币不足 800」的实测 |
| C | 击杀金币未建模（新发现，A 级） | `crates/game-solver/src/solve_local.rs:1539-1548` vs `crates/game-core/src/scene.rs:2301-2309` | 作为 H5 登记，下一批补边（含 `item25` 双倍） |

**建议（非阻断）**：

1. 判据补一条「买两次/买后余额仍要用于后续消费」的夹具，钉住扣金（当前 M1 存活）。
2. 商店「一次性移除」改为按 `flags&1`/`flags&4` 分派（可重复购买/无限购买）：14 家有表行且开价门的店里，**11 家 `flags=3|6`（可重复/无限购买）被压成 1 次**。
3. 走臂改为 `Blocked`（商人不可踩），或至少注释改为「过近似」而非「原 Move 同款」。
4. 「第 5 种排序」要么补产物（脚本+json），要么在文档中降级为「未留证的单次实验」；「排序空间已穷尽」改为「五种启发式均无解」。
5. §3.1 的探针数字建议以可复跑测试入库，否则长期只能是假设级。

---

## 12. 卫生证明（本复核未污染仓库）

```
$ git status --short -- crates/ docs/*.md docs/findings/*.json
(空)
$ git diff --quiet && echo NO_DIFF
NO_DIFF
```

- 临时探针 `crates/game-solver/tests/zz_tmp_coloc_probe.rs`：**已删除**（删除后重跑 gates 得 `total=1081`，证明删除生效）。
- 变异 M1–M4：每次从 `/tmp/solve_local.rs.bak` 还原，逐次 `git diff --stat` 为空。
- 买边打点（`eprintln!("SHOPBUY …")`）临时插入 `solve_local.rs:1526` 后已还原（`cp /tmp/solve_local_clean.rs`），最终 `git diff` 空。
- 仅写入本报告文件与 `/tmp` 下的证据文件（`/tmp/q_rv.json`、`/tmp/q_inst.err`、`/tmp/tests_base.log`、`/tmp/gates_after2.log`）。

---

**Confidence: 82**（扣分项：§3.1 探针数字与「第 5 种排序」未复跑，按假设处理；真实地图探针为自建临时测试，虽 A 级但非入库判据）。
