# 求解器批4 落码 · 独立对抗复核

- **被审件**：`modern` 分支 8 commit 链 `80c9506 → ac179f9`（HEAD=`ac179f9fe922b22591263faf385a1afe557db25d`，未推送），报审人 d8e4f44b
- **复核人**：独立对抗复核（subagent），只读复核，不改任何被审件文件；本报告为唯一写入物
- **复核时间**：2026-11-15
- **复核方法**：git 实核 + sed 行号实测 + 独立探针（探针 crate 位于 **仓外** `/tmp/rev_probe4`，path 依赖 game-solver/game-data/game-core，未污染被审件）+ 判据/门禁独立重跑 + 4 发变异实测
- **总判**：**可推送（0 红 / 4 黄）**，Confidence 87

---

## 面1 · commit 链与夹带 —— **绿**

- **8 笔实核存在**（`git log --format='%h %ad %s'`，时间均为 11-15 当日）：

  | commit | 时间 | 内容 | 统计 |
  |---|---|---|---|
  | `80c9506` | 08:55 | docs: 外部求解算法调研+对抗复核入库（批4 前置） | docs |
  | `8b33e6d` | 12:21 | test: C4-e① Y3 confidence 变异真红补钉 | tests/solve_global.rs +46 |
  | `65252ae` | 12:31 | docs: C4-e② cloud_goals「唯一写点」证伪更正（零行为） | solve_local.rs +4/-1 |
  | `75baf3b` | 13:30 | feat: C4-a 贪心零决策闭包（ClosurePolicy OFF/q1/q2/q3 + solve_layer_with + closure_step） | solve_local.rs +379/-5, tests +78 |
  | `5acc838` | 14:23 | feat: C4-a 全局接入 solve_global_with(policy 透传) | +21/-2 |
  | `b49aa61` | 18:41 | feat: C4-b′ 资源预留 landmark（红钥匙下界剪枝） | solve_global.rs +251, tests +113 |
  | `ef934da` | 18:51 | feat: C4-d query 子命令（三查询落盘 + A4 判据） | bin/solver.rs+65, lib.rs+1, query.rs 新增152, tests+40 |
  | `ac179f9` | 19:14 | docs: 批4 三查询结果落盘 | findings 119 行 + solver-queries.json |

  顺序与报审清单一致，无缺笔、无多笔。

- **越界=零**：`git diff --name-only 80c9506~1..ac179f9` 仅 11 文件 = `crates/game-solver/{src/bin/solver.rs, src/lib.rs, src/query.rs, src/solve_global.rs, src/solve_local.rs, tests/solve_global.rs, tests/solve_local.rs}` + `docs/findings/{2026-11-15-external-solver-algorithms-survey.md, 2026-11-15-solver-queries.md, reviews/2026-11-15-solver-survey-review.md, solver-queries.json}`。
- **红线（original/ reference/ crates/game-core/ crates/game-data/ gates/ data/）在 diff 中零出现**，`git status crates/` 干净。
- **探针已删**：`find . -name "*probe_c4*"` 空。
- **无临时插桩残留**：`grep -rn 'G-DBG|TEMP-C4|probe_c4' crates/ gates/ reference/` 代码中 0 命中（仅 `docs/findings/2026-11-15-solver-queries.md:56` 文档引用了 `[G-DBG]` 标签字样，属文档记载，非代码残留）。

## 面2 · 红档真实性（TDD 红先行）—— **绿**

5 份 `/tmp/red_c4_*.txt` 全存在，**mtime 均早于对应 commit**（`ls -la` + `git log` 对照）：

| 红档 | mtime | 对应 commit 时间 | 内容（实读） | 判定 |
|---|---|---|---|---|
| `red_c4_c4e_y3.txt` | 12:09:59 | `8b33e6d` 12:21:14 | 变异去 node_cap_hit 支路 → `assertion left:"optimal" right:"bounded-best"` FAILED @ solve_global.rs:1134 | 真断言红 ✓ |
| `red_c4_c4a.txt` | 13:02:55 | `75baf3b` 13:30:28 | E0432 unresolved `solve_layer_with`/`ClosurePolicy` + E0282，tests/solve_local.rs 编译 6 errors | 真红（测试先行、实现未落）✓ |
| `red_c4_c4b.txt` | 17:43:10 | `b49aa61` 18:41:59 | 真断言红：`expansions=1 left:1 right:0`（红耗尽必死态未被剪）@ tests/solve_global.rs:1258 | 真断言红 ✓ |
| `red_c4_c4d.txt` | 18:39:29 | `ef934da` 18:51:05 | E0433 could not find `query` in `game_solver` ×6 | 真红 ✓ |
| `red_c4_c4d_raw.txt` | 18:39:16 | 同上 | 同上（原始输出） | ✓ |

红档先于实现落地，非事后补档；C4-a/C4-b′ 均为「测试先红、实现后绿」的 TDD 顺序。

## 面3 · C4-a 判据攻审 —— **绿**

- **判据①** `c4a_闭包参数化_Q2旁路零伤怪不被自动收`（tests/solve_local.rs）：Q1 断言 `g1.iter().all(|g| g.kills >= 1 && g.route.iter().any(|w| w.starts_with("bump ")))`；**Q2 断言 `g2.iter().any(|g| g.kills == 0)`——非恒真**：夹具 `fixture_closure()` 4×2 全可走、ty41 零伤怪@(1,0) 有旁路，若 Q2 误用 `All` 则所有解 kills≥1 ⇒ 断言必红。实证：变异④（`q2()` kill_zero 由 Blocking→All）→ 该判据 **FAILED**，见面9。
- **判据②** `c4a_闭包参数化_Q3排除被禁道具`：Q1 断言 `g1.iter().any(|g| g.post.atk == 230)`（ty36 落格即穿 10→230）；Q3 断言 `g3.iter().all(|g| g.post.atk == 10 && !g.post.item_types.contains(&36))`。**ban 真排除 ty36**，实证：变异②（ban 分支 `if self.policy.ban.contains(...)` → `if false && ...`）→ 该判据 **FAILED**、另一判据通过（选择性命中），见面9。
- **classify ban→Blocked 实文**（solve_local.rs 末 match）：`if self.policy.ban.contains(&pickup_ty.unwrap_or(i32::MIN)) { Cell::Blocked } else { Cell::Pickup }`，ban 空（OFF）⇒ 恒 Pickup，批0-3 零行为变化。
- **闭包「出口格不踩」语义**：closure_step 的 BFS 中 `if dist.contains_key(&n) || self.level.exits.iter().any(|e| (e.x, e.y) == n) { continue; }`（出口设边界、不入可达域）；而 `reach()` 注释明言「出口格照常可走——本函数问的是杀哪些怪才通，故不设出口边界」。两处 BFS 出口语义不同属**有意设计**（closure 问的是"杀哪些怪通到出口前"，reach 问"是否物理可达"），非缺陷。
- **误伤疑点（H3 关联）**：ban→Blocked 只作用于 ty36/ty40 两个**具体落格**，语义即「不许踩这两格」（真实游戏踩格即拾），是模型语义而非误伤；H3 的"切断走道"经空 ban 对照探针验证不改 Q3 结论（见面6）。

## 面4 · C4-b′ 攻审 —— **绿**

- **landmark_dead 实文**（`solve_global.rs:796-831`）：`needed = floor_dist_map(...).get(&goal)`；护栏① `needed==LOCK_INF || needed==0 → false`；护栏② zero 层含 Script 出口 → false；`key_ty=[26,27,28][color]`；`held=[st.player.yellow, st.player.red, st.player.blue][color]`；avail = held + zero 层内 `st.ents` 中 ty==key_ty 且**未被墙封死**的钥匙；`avail < needed → true`。
- **调用点**（`solve_global.rs:370-377`，sed 实测）：`if input.floors.contains_key(&50) && (landmark_dead(lm, 50, st_ref, 0) || landmark_dead(lm, 50, st_ref, 1) || landmark_dead(lm, 50, st_ref, 2)) { continue; }` → **`expansions += 1;` 在剪枝之后**（377 行），剪中不计 expansions。
- **0 触发=保守正确**：avail 含 f8 未封死的 ty27（f8 真实入口可取）⇒ avail≥1；f20 红门 needed≤1 于多数态成立 ⇒ `avail < needed` 恒 false。avail 还含「将来才拿得到」的钥匙 ⇒ 下界偏松=**只可能漏剪死态、不会误剪活态**，方向保守。判据③（层内还有 ty27 ⇒ expansions>0）正钉 avail 计数不虚高。
- **数学口径**：`lock_bfs` 为 0-1 BFS 数「该色门数」（过该色门 +1，其余 0），needed=入场格到目标层 50 的**最少开门数下界**（他层用全图 min 乐观 ⇒ 低估 ⇒ 不误剪）；avail 数「手里+零代价可达层内未封死钥匙」——**每开一色门耗一色钥匙，1:1，按门数 vs 按钥匙消耗口径一致**。
- **判据实文**（tests/solve_global.rs:1167-1283）`solve_global_t资源预留landmark_红耗尽必死态被剪_红绿对照` 三段：①red=0+无 ty27 ⇒ `goals.is_empty() && expansions==0`；②red=1 ⇒ 有解且 exp>0；③有 ty27 ⇒ 有解且 exp>0。变异①（`avail<needed`→`false`）→ 该判据 **FAILED**（见面9）。

## 面5 · 锚抽验（24 个，≥12）—— **绿**

全部 sed 实测命中，引文行号=实测行号，**零漂移**：

- `solve_local.rs`：362 `pub fn solve_layer_with(` · 960 `pub enum KillMode {` · 973 `pub struct ClosurePolicy {` · 991 `pub fn q1()` · 999 q2 · 1007 q3 · 1260 `fn closure_step(&self, st: &St) -> Option<(Act, St, GateArrive)>`
- `solve_global.rs`：254 `pub fn solve_global_with(input: &GlobalInput, node_cap: u64, policy: ClosurePolicy) -> GlobalResult` · 263 `pub fn node_cap_env()` · 277 `pub fn solve_global_capped_with(` · 626 `fn lock_grids(` · 654 `fn lock_bfs(` · 701 `struct LandmarkLocks {` · 715 `fn build_landmark(` · 763 `fn floor_dist_map(` · 796 `fn landmark_dead(`
- `query.rs`：26 QueryOpts · 47 QueryValue · 57 QueryResult · 71 console_line · 98 run_query · 139 run_all · 149 to_doc
- `bin/solver.rs`：134 `if args.first().map(|s| s.as_str()) == Some("query") {`

补充实测：`solve_global.rs:487` confidence 逻辑、`solve_global.rs:370-372` landmark_dead 三色调用、`solve_global.rs:377` expansions+=1、`tests/solve_global.rs:16-17` unused import、`data/tables/shop_goods.json:157` id10、`reference/src/deobf/a.java:7379-7384` 商店 case 6——全部与报审引文一致。

## 面6 · 瓶颈分析攻审（最关键）—— **黄**（六核绿，两处措辞偏差 + H4 定级）

独立复跑（探针 crate `/tmp/rev_probe4`，仓外）：

| # | 声称 | 独立复跑结果 | 判定 |
|---|---|---|---|
| 1 | ty27 红钥匙仅 f8/f14/f34/f37/f42/f51 各 1 | 全 55 层 sprite 扫描：`{8:[(2,2)], 14:[(10,1)], 34:[(8,5)], 37:[(7,5)], 42:[(7,1)], 51:[(6,2)]}` | **绿（完全一致）** |
| 2 | f20 上行被 ty2@(6,9) 红门挡死，9R up=75 其余 0 | 真实落点 entry=(6,11)：0keys=0 / 9Y=0 / 9B=0 / **9R=75** / **all9=864**；地形 BFS 门全开可达、关死不可达 ⇒ 红门是割 | **绿（75/864 精确一致）** |
| 3 | f10 上行需黄+红 | entry=(1,10)：S/Y/R/B/y1r1 均 up=false；**y9r9 → up=true goals=4535**（=声称 S+YR 4535）；y2r1 可(5177)、y1r2 不可 ⇒ 双色成立，实测下界 2黄1红 | **绿** |
| 4 | f14 墙封死、五档×6 入口拿不到 | 6 入口 × 5 档 × S0 → key_picked 全 0（30/30）；**真实落点 (1,11)/(6,11) 各档含 god 档(99999/99999/9999) 全 0**；非真实入口 (10,1)all9→799（该入口就在钥匙格上） | **绿（结论）/ 黄（「墙」字面：地形超集 BFS 可达，实际是实体/门结构封死）** |
| 5 | f8 需≥2 黄（y=1→red=0；y=2→494/1495） | 真实落点 entry=(11,1)：y0=0(goals=1)、**y1=0(goals=7)**、**y2=494/1495**、y9=2/1377 | **绿（精确一致）** |
| 6 | 1M cap exp=791801 仍 goals=0 | `solver query q1 --node-cap 1000000` → exp=**791801**、goals=[]、node_cap_hit=true | **绿（精确一致）** |
| 7 | **f51 红门包死（起点层死锁）** | 起点 (1,11) 0keys → 96 goals **全部 route 经过 (6,2) 且 key 已从 post_ents 消失，但 96/96 post.red=0**；route tail = `6,2→…→door 8,3→…→door 4,4→…→2,2` ⇒ **钥匙可捡、随即在 door 4,4 花掉，净红恒 0** | **黄（机制描述偏差：非「红门包死」而是「捡了即花」；「起点层拿不到净红」实质结论成立）** |
| 8 | H3（Q3 ban 切断走道） | 空 ban 对照 `solver query q3 --ultimate 99` → reachable=false、exp=**68889**、node_cap_hit=true（原 ban=36,40 → exp=641 队列耗尽）⇒ ban 确实改变搜索面，**两路均无解，H3 不改 Q3 结论**；ty36 金箍棒唯一在 f13@(6,3)、ty40 紫金龙鳞甲唯一在 f44@(6,6) | **绿（假设不改结论）** |

- **H4 商店边（关键定级）**：`data/tables/shop_goods.json:157` 起 id10 实文 = `{"id":10,"src":1,"idx":6,"price":800,"effect":{"key":"AddKeys","params":{"blue":0,"yellow":0,"red":1}},...}`；`reference/src/deobf/a.java:7379-7384` 实文 = `case 6: if (this.spendGold(800)) { this.redKeyCount++; } else { showOverlayMessage("没有足够的金钱"...) }`（注意：报审引的 a.java 是 **deobf 版**，`reference/seed/a.java` 同行号是别的代码——引文内容属实）。**game-solver 无任何商店调用**：`grep -i shop` 于 solve_local.rs/solve_global.rs 仅 state.rs:37 注释提及；`AddKeys|spendGold|spend_gold` 在 `crates/game-solver/src` **0 命中**；ingest.rs:33 shops 仅 census 计数（`shops=17`）。⇒ **H4 模型缺口为真（A 级）**：商店红钥匙边不存在于求解器模型，「无解」是**模型内结论**。
  **定级：黄（非红）**——理由：① findings 已将 H4 明确列为 §3.2 假设并注明「既有模型缺口（批4 之前就存在）」，§1 写明「待批5 oracle 回放确认」「不得当作已证明无解」；② §5 建议 1 即优先补商店边；③ 结论口径是「界内无全程解（如实）」而非「游戏无解」。缺边未被虚报为结论 ⇒ 结论仍成立但须注明缺边 = 黄。

## 面7 · 「界内无解」结论定级 —— **绿**

- **A4 三行复跑逐字一致**：`cargo run --release --bin solver -- query --all --out /tmp/rev_query_all.json` → `Q1 max_hp=0(feasible)` / `Q2 min_kills=0(feasible)` / `Q3 reachable=false`；输出 json 与 `docs/findings/solver-queries.json` python 比较 **identical: True**（含 q1 exp=68889、q2 exp=7936 + truncated_layers `[[6,200000],[7,200001]]`、q3 exp=641、三者 node_cap=100000/bound=1000000/goals=[]）。**声称1 逐项复现**。
- **confidence 非虚标**（solve_global.rs:487-497 实文）：`if goals.is_empty() { "feasible" } else if hit_bound || node_cap_hit || !truncated_layers.is_empty() { "bounded-best" } else { "optimal" }` ⇒ goals 全空 ⇒ `feasible` 是正确的最弱降级词，findings §1 亦明确「max_hp=0 是空集最值，feasible 不得当已证明无解」。
- **无静默降级/调阈值**：bound=1_000_000、node_cap=100_000 全走 `QueryOpts::default`（node_cap_env=SOLVER_NODE_LIMIT 缺省 100_000）或 CLI 显式参数；对照 plan §1.3 有界求解口径（界可配+三级置信）与 R1 风险行（不承诺全局最优）——**一致**。q2 两层截断如实落盘 truncated_layers，非静默。
- 瓶颈佐证（面6 六核绿 + 四队列序全败/2M OOM 见 findings §3.1）支撑「界内无解=如实」。

## 面8 · 判据独立重跑 —— **绿**（3 条黄级卫生警告）

- `python3 gates/cli.py check` → **`PASS=13 FAIL=0 SKIP=0 EXIT=0`**（/tmp/rev_gates_check.txt）：含 test-ratchet **total=1080（基线1075）无回退**、state-baseline **[GREEN] 金样逐字节等价 74 场景 sha 全同**、original-integrity/reference-seed-integrity 等全绿。
- `cargo test -p game-solver --release` → **EXIT=0，44 测试全绿**（lib 7 / census_snapshot 4 / solve_global 20 / solve_local 13 / bin 0），含「逐层拼接==全局解」A/B 对拍（tests/solve_global.rs:180/214/318/344）。报审声称 6（闸13/13 + 金样74 + ratchet1080 + 批0-3 A/B 对拍）**复跑成立**。
- **黄级卫生（不影响功能）**：`cargo test` 输出 3 条警告——① `unused import: solve_global_with` @ tests/solve_global.rs:16:47；② `unused import: game_solver::solve_local::ClosurePolicy` @ tests/solve_global.rs:17:5（b49aa61 加的导入，测试实际用 `solve_global_capped` @ :1229/:1245/:1264）；③ `unused_mut` @ solve_local.rs:1295:13。

## 面9 · 变异 ≥2 实测 —— **绿**（4 发 4 红，全部还原 CLEAN）

改坏 → 跑判据 → 红 → `git checkout` 还原 → 确认 CLEAN、HEAD 仍 `ac179f9`：

| # | 变异 | 结果 |
|---|---|---|
| 1 | solve_global.rs `avail < needed` → `false`（禁 landmark 剪枝） | `solve_global_t资源预留landmark_红耗尽必死态被剪_红绿对照` **FAILED** ✓ |
| 2 | solve_local.rs ban 分支 → `if false && ...`（禁 ban→Blocked） | `c4a_闭包参数化_Q3排除被禁道具` **FAILED**（另一判据通过=选择性命中）✓ |
| 3 | solve_global.rs confidence goals 空 → `"optimal"` | 3 测试红：`solve_global_toy反例_无钥直入floor50无解`/`solve_global_toy筋斗云边_区间门外拒绝`/`solve_global_t节点上限触发_诚实降级` ✓ |
| 4 | solve_local.rs `q2()` kill_zero: Blocking → All | `c4a_闭包参数化_Q2旁路零伤怪不被自动收` **FAILED** ✓ |

⇒ 判别力实证：4/4 变异均红且命中预期判据，判据**非恒真、非摆设**。

## 面10 · 遗留与跳过合规 —— **绿**

- **C4-c 跳过理由已写入 findings §4.4**（sed 实测：`findings:97 "### 4.4 C4-c（图缩点）为何降级"`、`:101 "本批**跳过 C4-c**，仅保留其判据要求（route 展开回真实格序列）作为后续若实施时的硬约束"`；`:41` 亦有「C4-c（图缩点）**未实施**——理由见 §4.4」）。理由=收益已被 C4-a 吸收、瓶颈在全局资源序不在层内走格，lead 裁定可跳过——与报审一致。
- **H1/H3 未证实标注守住「禁据此改码」**：findings `:65` `### 3.2 假设（未证实，禁止据以下结论改代码）`，`:106` 商店可达性标注「**假设，未证实**」。H1/H3/H4 全部在假设区，未混入 §3.1 已证实区。
- H4 虽经本复核证实为真（模型缺口 A 级），但**被审件当时的标注纪律合规**（假设不得混入结论——当时它就是未证实的）。

---

## 总判

**可推送（0 红 / 4 黄）。**

| 面 | 判定 |
|---|---|
| 1 commit 链与夹带 | 绿 |
| 2 红档真实性 | 绿 |
| 3 C4-a 判据攻审 | 绿 |
| 4 C4-b′ 攻审 | 绿 |
| 5 锚抽验（24） | 绿 |
| 6 瓶颈分析攻审 | **黄** |
| 7 界内无解结论定级 | 绿 |
| 8 判据独立重跑 | 绿（3 条警告卫生） |
| 9 变异实测（4 发） | 绿 |
| 10 遗留与跳过合规 | 绿 |

**4 条黄（均不阻断推送，建议随批5 或后续小步处理）：**

1. **f51 机制措辞**：findings 称「f51 红门包死（起点层死锁）」，实测为**可捡但随即在 door 4,4 花掉（96/96 净红=0）**——「起点层拿不到净红」实质结论成立，机制描述应更正（按 AGENTS §3 证伪更正文化，原地标注更正）。
2. **f14「墙封死」字面**：地形超集 BFS 可达，实际封死靠实体/门结构——结论（真实入口拿不到）成立，「墙」字应改为「实体/门结构」。
3. **H4 商店模型缺口（A 级证实）**：`shop_goods.json` id10 可 800 金买红钥匙 + deobf a.java:7379-7384 spendGold 实文 + game-solver 零商店调用 ⇒ 「无解」是**模型内结论**。findings 已标假设+禁改+建议优先补边，定级黄（结论成立但须注明缺边）；**批5 若补商店边，三查询结论需重跑**。
4. **测试卫生 3 条警告**：tests/solve_global.rs:16/:17 unused import、solve_local.rs:1295 unused_mut——建议顺手清理（不改行为）。

**复核未发现任何红级问题**：无越界改动、无红线触碰、无事后补档、无静默降级/调阈值、无判据恒真、无跳过未记录、无假设混入结论。

**Confidence: 87**（扣分项：f51/f14 措辞偏差说明报审的瓶颈描述有两处未经独立复核即落笔；H4 缺口使「无解」的完备性依赖模型闭合，批5 补边前结论只能是模型内结论——这两点均已降级为黄并写入本报告，不构成红）。

---

### 附：复跑命令（均在仓内执行，探针在仓外 `/tmp/rev_probe4`）

```bash
python3 gates/cli.py check                     # PASS=13 FAIL=0 SKIP=0
cargo test -p game-solver --release            # 44 全绿（3 条 unused 警告）
cargo run --release --bin solver -- query --all --out /tmp/rev_query_all.json
cargo run --release --bin solver -- query q1 --node-cap 1000000 --out /tmp/rev_q1_1m.json
cargo run --release --bin solver -- query q3 --ultimate 99        # H3 空 ban 对照
cd /tmp/rev_probe4 && cargo run --release --bin probe{,2..9}      # 面6 瓶颈独立复跑
git status && git rev-parse HEAD               # CLEAN @ ac179f9
```
