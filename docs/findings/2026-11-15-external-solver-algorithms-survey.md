# 外部求解算法调研（批4 前置）：magic-tower-AI / ferroplan decompose / 标准规划技术

> 日期 2026-11-15｜分支 `modern` @ 6846c85｜只读调研（未改码、未改计划）
> **证据标注**：【原文】=本轮实际 web_fetch / curl(raw) 读到源码或 README 全文；【摘要】=仅搜索结果摘要，未读原文。
> 本仓库行号均 `sed`/`grep` 实测于 2026-11-15；外部源码行号对应当日抓取的 `master`/`main` 版本（上游可能漂移）。
> 批4 背景：`solve-global` 实测 8 层各撞 20 万节点截断（`truncated_layers` 非空，`solve_global.rs:78/:353`），全程解未出。

## 0. 结论速览

1. magic-tower-AI（同域最贴脸）能跑通单塔全程，靠的**不是**更强的搜索算法，而是 4 个压缩手段：贪心零决策闭包、图缩点、粗位图去重+同键 hp 支配、决策数硬上限。它**没有**双目标（血量单目标）、**没有**分段、**没有**杀怪目标。
2. ferroplan `decompose` 与我们分层分解同构（拆→逐个解→拼接），但多了我们没有的两样：**冲突→逐级粗化→回落整解的失败回退**（完备性论证）与**兄弟目标保护**。
3. 我们的设计在标准答案轨道上（dominance + hierarchical decomposition + bounded anytime 三件套齐全）；缺的是：失败回退、landmark/资源下界剪枝、预处理缩点、贪心闭包。FF relaxation 与 CEGAR 因**方案 B** 不建议引入。
4. 批4 五条建议见 §4（4 条需改造、1 组不适用，无「直接可用」项）。

## 1. magic-tower-AI 算法拆解

来源：GitHub `ckcz123/magic-tower-AI`（README【原文】+ `AI/src` 源码 raw 全文【原文】：`Graph.java`/`State.java`/`Hero.java`/`Node.java`/`Util.java`）。博客 `http://ckcz123.com/blog/catagory/magic-tower-ai/` **一篇未读到**（见 §5）。

### 1.1 形态：单塔全程一次搜索，非分段【原文】

- README：三子项目 `AI`(Java 拆塔计算)/`program`(C++ 可视化)/`map_generator`(C#)，`test/` 有测试塔数据。算法说明指向作者博客（抓取失败）。
- `Graph.run()`（`Graph.java:261-344`）：全塔建**一个全局图**——节点=有门/怪/道具/楼梯的格子；`isLinked()`（`Graph.java:231-236`）同层 BFS 连通、跨层经楼梯递归判定 ⇒ 整塔一次搜索，无层间拼接问题（因为它把楼梯当作普通连边，不做分层分解）。

### 1.2 状态表示与去重：两层【原文】

- `State.hash()`（`State.java:91-100`）：`long` 位图，**只给「有门或有怪」的节点计 1 bit**（纯道具/走格不计）。
- 一级去重：`HashSet<Long> set`，poll 时 `if (!set.add(state.hash())) continue;`（`Graph.java:284`）。
- 二级支配：`HashMap<Long,Integer> map`，生成时 `if (map.getOrDefault(hash,-1)>another.getScore()) continue; map.put(hash, another.getScore());`（`Graph.java:316-318`）——**同位图下只留更高 hp**（`getScore()` 即 hp，见 1.4）：`Graph.java:317` 的 `>new则 continue` 正是 max 保护（仅当新分数 ≥ 已记录值才落入 `put`），故 `map` 实际保留该位图的历史最高 hp（等值也覆盖，值不变）。
- 支配维度只有 **hp 单目标**，不是 Pareto。

### 1.3 搜索策略【原文】

- 优先队列（`Graph.java:275-278`）：`cnt`（已合并的决策步数=击杀/开门数）**升序**，同 `cnt` 按 `getScore()` **降序** —— 即「先用最少决策到达，再比血量」。
- `shouldStop()`（`State.java:82-89`）：`cnt > STOP_COUNT`（`STOP_COUNT=45`，`State.java:15`）或 boss 已被访问。到达即更新 `answer`（`Graph.java:286-291`，`//break;` 被注释掉）⇒ **到达终点继续搜，全枚举取最优**，不是首次终止。
- 扩展前合法性：`Hero.isValid()` = `hp>0 && 各钥匙>=0`（`Hero.java:22-24`）。
- **被注释掉的候选剪枝**（`Graph.java:299-311` `should extend` 块）：只扩展「能解锁新节点」的邻接点——作者**未启用**，如实标注。

### 1.4 评分：血量单目标【原文】

`Hero.getScore()`（`Hero.java:26-29`）：`return hp;`（复合公式 `hp+1000*(atk+def)+…` 被注释）。⇒ 无杀怪目标、无多目标。

### 1.5 关键剪枝 A：贪心零决策闭包 `eatItem()`【原文】

`State.merge()`（`State.java:38-51`）每收一个节点后调 `eatItem()`（`State.java:54-72`）：

- 循环扫描邻接节点，凡 `shouldEat`（`Node.java:127-140`）者**自动收掉**：①有道具 ⇒ 收；②无门、有怪、且**全部怪 `Util.getDamage(hero,monster)==0`（零伤可秒）** ⇒ 收。收完 `break` 重扫，直到不动点。
- 之后自动按序用商店：`while (graph.shop.useShop(hero, shopTime)) shopTime++;`（`State.java:69-71`，钱够就买 atk；买点选择是源码里的 TODO 注释）。
- `Util.getDamage` 是确定性伤害公式（含先攻/魔攻/坚固/连击/破甲/反击/净化/模仿 special，`Util.java` 已读全文）。

**效果**：免费道具、零伤怪、可负担的商店消费**永不构成搜索分支**——「杀怪攒资源开钥匙门」的组合爆炸被压缩到只剩「有伤怪 / 门 / 商店买不买 / 岔路」这些真决策点。

### 1.6 关键剪枝 B：图缩点 `mergeNode()`【原文】

`Graph.mergeNode()`（`Graph.java:196-209`）+ `shouldMerge()`（`Graph.java:211-229`）：静态预处理（`--nomerge` 可关）把相邻节点合并成超节点——双道具相邻可并；**单侧有道具**或任一方为 boss 则不并（排除条件）；两节点**共享邻点即不并**（`Graph.java:216-218` `return false`）；其余要求**各自邻域内部两两互连**的团条件（`Graph.java:219-227`）。⇒ 纯走格/双道具分支被压缩成一步。

### 1.7 它怎么回答「三查询」？

| 我们的查询 | magic-tower-AI |
|---|---|
| ①通关剩余血量最大 | ✅ 唯一目标（`getScore=hp`） |
| ②最少杀怪 | ❌ 无此目标（`cnt` 只是队列序，不是被优化目标） |
| ③不拿终极装备能否通关 | ❌ 无此类约束机制 |

### 1.8 可偷的剪枝技术逐条清单

| # | 技术 | 出处 | 备注 |
|---|---|---|---|
| T1 | 贪心零决策闭包（道具+零伤怪+商店，不动点归一） | §1.5 | 最值得偷，但需按三查询参数化（§4-1） |
| T2 | 粗决策位图去重 + 同键单目标支配 | §1.2 | 我们已有更强的 K₀+Pareto；粗键**不可直接照搬**（§4-5） |
| T3 | 图缩点预处理（无门怪/双道具相邻合并） | §1.6 | 需保证 route 可展开回真实路径（§4-3 变体） |
| T4 | 决策数硬上限 + 到达终点继续搜（anytime 记账） | §1.3 | 我们已有有界求解+信心降级，形态不同 |
| T5 | 队列序：决策数升序、hp 降序 | §1.3 | 需新增决策计数字段且方向反转，见 §4-4（教训链 `solve_global.rs:303-308`） |
| T6 | 「只扩展能解锁新节点」 | `Graph.java:299-311` | **作者未启用**，不建议先引入 |

## 2. ferroplan `decompose` 对照

来源：GitHub `hhh42/ferroplan`（README【原文】；`crates/ferroplan/src/api.rs`、`tresolve.rs` raw 全文【原文】；早前轮次读过 `partition.rs` 关键段【原文】）。

### 2.1 API 语义【原文】

`api.rs:1157-1160`（doc 原文）：「a goal too big for the one-shot temporal search is split into ordered sub-contracts — each solved whole and verified — then stitched into one validated plan. A goal that can't be split (or whose split doesn't validate) falls back to a single monolithic contract.」返回 `Decomposition{solved, contracts, plan, monolithic, notes}`。README：Rust 数据并行 PDDL 规划器（FF 家族：bitset 状态、EHC+best-first、delete-relaxation 启发式），CLI `ff --decompose`。

### 2.2 实现机制【原文】（`tresolve.rs`）

- 文件头（`tresolve.rs:1-14`）：把世界目标拆成 contracts → 每个 contract 从**运行中的组合状态** `solve_from` 解 → **严格顺序拼接**（offset = 前一 makespan + ε 缝，`EPS=0.001`）→ **冲突就 merge 成组，最终单组回落** `temporal::solve` ⇒ 「the decomposer is solvable EXACTLY when `temporal::solve` is」——**完备性靠回落保证**。
- 兄弟目标保护：解某个 contract 时 forbidden 掉会 delete 已达成兄弟目标 fact 的操作（`tresolve.rs:223-231`）；解不出先放宽保护重试。
- 冲突检测：新 contract 若破坏已达成兄弟目标（`goal_met_with` 谓词/数值复检，`tresolve.rs:297-300`）⇒ conflict → `merge_at` / `merge_with_neighbor` 粗化（`tresolve.rs:365`）；每轮组数严格递减 ⇒ 终止于单组回落。
- 拼接后整体 `validate()` 硬校验（`tresolve.rs:334`），不过也回落。
- `monolithic_fallback` 开关（`tresolve.rs:130-139`）：直接 API 调用开；升级阶梯末级关（防递归）。
- 粒度：v1 最细=每个正目标事实一组，resolver 动态 coarsen（即 SGPlan 的 partition-and-resolve 动态粒度控制，`partition.rs`）。

### 2.3 与我们分层分解的差异

| 维度 | ferroplan `decompose` | 我们（`solve_global.rs`/`solve_local.rs`） |
|---|---|---|
| 拆分对象 | **目标事实**分组（goal-interaction graph） | **地理楼层**（单层 `solve_layer`，`solve_local.rs:349`） |
| 子问题求解 | 从运行组合状态 `solve_from` 解 | 入场格→本层出口的 Pareto 选项（`solve_global.rs:4`） |
| 拼接方式 | 时间轴严格顺序 + ε 缝 | 四类层间边静态端点拼接 |
| **失败处理** | 冲突→逐级粗化→**回落整解**（完备） | **单向拼接**，截断/触界→`bounded-best`（`solve_global.rs:451`） |
| 已达成成果保护 | sibling protection（forbidden 删除操作） | 层间边携带上一层终态 flags/keys/bag（传播式，`solve_local.rs:1143-1159`） |
| 拼接后校验 | 独立 `validate()` 硬校验 | 判据「逐层结果拼接==全局解」（`solve_global.rs:5`） |

### 2.4 可借鉴

1. **失败回退路径**（我们缺的主件）：拼接失败/层截断 → 先粗化（相邻层合并联合求解）→ 仍失败才回落有界全局解；组数递减保证终止。对照我们现状：层截断直接降级 `bounded-best`，没有「把截断层与邻居合并再解一次」的中间档。
2. **兄弟目标保护 = 我们的 flags/keys 传播**：方向一致，我们靠状态携带实现，等价。
3. **拼接后独立复检**：我们已有同款判据（`solve_global.rs:5`），无需改。

## 3. 标准技术清单 vs 我们已有设计

（本节来源均为**搜索摘要**，未读论文全文；URL 见各行。）

| 标准技术 | 我们有无（行号实测） | 差距 / 说明 | 来源 |
|---|---|---|---|
| Dominance / Pareto 剪枝（多目标搜索） | **有**：K₀ 去重键（`solve_local.rs:754` `struct Key`，hp/kills/steps 不进键）+ `dominates`（`solve_local.rs:787`、`solve_global.rs:154`）与 `weakly_ge`（`solve_local.rs:783`、`solve_global.rs:166`） | 与 BOA\* 类 bi-objective 支配检查同构；我们还多了弱支配防等价环 | [BOA\* (AIJ)](https://www.sciencedirect.com/science/article/am/pii/S0004370222001473)、[IJCAI'23 Lazy MOSF](https://ijcai.org/proceedings/2023/850) |
| Contract-based / partition-and-resolve 分解 | **有**：分层分解+四类层间边（枚举 `solve_global.rs:364-367` `ExitKind::Stair\|Script\|Cloud\|UseItem`；判据 `solve_global.rs:4-6`） | 缺失败回退粗化（§2.4-1） | [SGPlan (JAIR)](https://www.jair.org/index.php/jair/article/download/10460/25075/)、[SGPlan slides](http://wah.cse.cuhk.edu.hk/wah/Wah/papers/C152/C152pres.pdf) |
| Bounded / anytime + 信心降级 | **有**：有界求解、`node_cap`（`solve_global.rs:335`）、`bounded-best`（`solve_global.rs:451`） | 无 | FF 的 EHC+best-first 组合（见下） |
| Landmark 启发式 | **无** | 可做**格点版**：必经门/必经格 → 钥匙与杀怪下界剪枝（§4-2），不必引入 fact 模型 | [LAMA/landmark (JAIR'10)](https://gki.informatik.uni-freiburg.de/papers/richter-westphal-jair10.pdf) |
| Delete-relaxation 启发式（FF） | **无**（ferroplan 已内置） | 需把游戏状态编译成 fact ⇒ 第二套模型，**方案 B 下不建议** | [FF (JAIR 2001)](https://acm-stag.literatumonline.com/doi/10.5555/1622394.1622404) |
| State abstraction / 缩点预处理 | **部分**：K₀ 内 `ents: Vec<EntRec>` 为 4 字段精简记录（`solve_local.rs:704-709`，槽序保留、**非哈希摘要**）；无图缩点 | 缺 magic-tower-AI 式 move 分支缩点（§1.6） | [Fast Downward (Helmert'06 被引)](https://ojs.aaai.org/index.php/ICAPS/article/download/31459/33619) |
| CEGAR / abstraction refinement | **无** | 需抽象转移系统+反例精化框架，工程量大且与既有分层重复，**方案 B 下不建议** | [ICAPS CEGAR](https://ojs.aaai.org/index.php/ICAPS/article/download/3499/3367/)、[Basel 讲义](https://ai.dmi.unibas.ch/_files/teaching/hs25/po/slides/po-e14-handout4.pdf) |
| 层级/HTN 规划 | **无显式**；分层分解是地理层级而非任务网络 | 不建议上 HTN（需人工任务网络+第二套抽象） | [HTN 综述片段](https://api.drum.lib.umd.edu/server/api/core/bitstreams/dc415321-eafc-4a1e-8f1a-cbdcf69f23de/content) |
| 贪心零决策闭包 | **无** | §4-1 | 本报告 §1.5（源码原文） |

**轨道判定**：dominance + hierarchical decomposition + bounded anytime 三件套齐备，**在标准答案轨道上**；漏掉的成熟技术是 **landmark/下界剪枝**（可在方案 B 内做格点化）与 **分解失败的粗化回退**（ferroplan 已给模板）。relaxation heuristic / CEGAR 属标准答案但与方案 B 冲突，记为「不引入」。

## 4. 批4 可行动建议（5 条）

> 全部尊重方案 B：不引入第二套游戏模型，伤害/规则判定一律直调 `game-core` 真引擎；以下改动都在求解器扩展器/预处理层。

**1. 贪心零决策闭包（T1）—— 需改造**
现状对照：`classify` 把拾取/杀怪与普通走格并列成 `Cell::Pickup`/`Cell::Monster`（`solve_local.rs:1045-1046`），`expand`（`solve_local.rs:1053`）各出一条独立转移 ⇒ 道具与零伤怪**目前都是搜索分支，无闭包归一**（§3 表「贪心闭包=无」的代码依据）。
在 `solve_layer` 扩展前把「可达即捡的道具、挡路且零伤可秒的怪、买得起的商店消费」确定性收到不动点，只把有伤怪/门/岔路留作分支（直接对应 magic-tower-AI `eatItem`）。**需改造理由（三查询语义不同）**：①血量最大：零伤怪与道具可全收；②最少杀怪：**旁路**零伤怪绝不能自动收（白加 kills），只有挡路必杀的才收；③不拿终极装备：闭包必须排除被禁道具。⇒ 闭包谓词按查询参数化。合规性：闭包只决定「哪些转移是确定性的」，掉落/伤害仍由真引擎判定。

**2. 必经点/资源下界剪枝（landmark 格点化）—— 需改造**
层内对入场格→各出口算必经格/必经门（网格连通性割），得「至少开 k 门、至少杀 m 怪、至少持 t 钥匙」下界，与当前 `keys/hp` 比对，不足即剪；全局层可加跨层钥匙守恒下界。**需改造理由**：标准 landmark 是 fact 级（LAMA），我们做格点版；只需连通性分析+已有门/怪清单，不重算战斗 ⇒ 方案 B 合规。这是 §3 表中唯一能在方案 B 内落地的**标准规划文献**技术（表末的贪心闭包来自同域源码，不属规划文献）。

**3. 图缩点预处理（T3）—— 需改造**
把 `mergeNode` 式「无门无怪/双道具相邻」缩点引入层内预处理，压缩 move 分支（当前纯走格占大量节点）。**需改造理由**：缩点后 route 必须能展开回真实格序列，才能满足四类层间边拼接口径与「逐层==全局」判据（`solve_global.rs:5`）；缩点属求解器输入侧变换，不动 `game-core` ⇒ 合规。

**4. 队列序加「决策数升序、hp 降序」tiebreak（T5）—— 需改造**
当前全局队列键是 `(Reverse(dist_root), depth, Reverse(id))`（`solve_global.rs:314-315`、入队 `:566`），触界截断下**扩展序决定界内能否碰到好解**；参照 magic-tower-AI（`Graph.java:275-278`）考虑「决策数升序、hp 降序、id 兜底」。**需改造理由**：(a) `GSt`（`solve_global.rs:88-102`）**无「决策数（开门/开怪）」字段**（仅 kills/steps 等累计量）——须新增计数或代理，非纯改序；(b) 现 tiebreak 第 2 维 `depth` 在 max-heap 下为**降序**（深先），换「决策数升序」是方向反转；(c) 排序改动有两次回归教训（`solve_global.rs:303-308`：b41 浅层扇出淹没、f7/f8 深徘徊，现行「层链距升序+depth 降序」即其收敛结果）。落地须 A/B 对拍批0-3「逐层拼接==全局解」判据（K₀/去重/bound/结果集不变、同 env 同序）。

**5. 两条「不适用」—— 理由如下**
- **照搬粗位图+同键 hp 支配（T2）不适用**：它的 `hash` 位图不含 keys/bag/flags 等资源维度，跨 K₀ 状态比 hp 不安全，会破坏批0-3 已实证的去重/支配安全域（`solve_local.rs:754`、`solve_global.rs:154`）。它能成立是因为其位图已覆盖全部决策性节点，且 keys/atk/def/money 同样不进 hash、其支配**理论上同样有损**，靠单塔/测试塔形态兜底；我们 K₀ 含全部资源维、三查询目标不同，该前提不成立。
- **FF delete-relaxation / CEGAR 全局抽象不适用**：二者都要求把游戏状态编译成 fact/抽象转移系统 = 第二套游戏模型，直接违反方案 B 裁定；且 CEGAR 的精化框架与既有分层分解功能重叠，工程收益比 §4-1/§4-2 低。

## 5. 未竟面（抓取失败 / 未读到）

1. **作者博客一篇未读**：`http://ckcz123.com/blog/catagory/magic-tower-ai/`（及 http/https、www 变体）curl 返回 **403 Forbidden（nginx）**，web_fetch 返回 fetch 端点 **404**；`archive.org/wayback`、`web.archive.org`、`timetravel.mementoweb.org`、`r.jina.ai` 代理**全部网络超时/不可达**（本环境到 archive.org 路由不通）；Bing/Baidu 站内搜索未返回其文章 URL（DuckDuckGo 202 反爬）。⇒ §1 全部基于源码+README，博客中的算法自述（如设计动机、双目标讨论）**未能验证**。
2. **ferroplan `BORDERS.md` 不存在**：README 引用它描述「一次求解 vs 必须拆 contracts」的边界，但 raw 404、根目录与 `docs/` 均无此文件（README 与仓库不一致）。
3. GitHub API 匿名限流（rate limit exceeded）；文件清单靠 tree 页面与 raw 直取。
4. 未读文件：magic-tower-AI 的 `Item.java`/`Main.java`/`Shop.java`、`AI-cpp` 其余部分、`map_generator`；ferroplan 的 `resolve.rs` 全文、`docs/sgplan6-spec.md`。
5. §3 各行论文**仅搜索摘要**，未读全文；`jetd1/MogicTower`、`ThinerDAS/dasstudio2016`（后者称魔塔 NP-complete）仅 README 摘要。
6. 外部源码行号基于 2026-11-15 抓取的 `master`/`main`，上游更新后可能漂移；本仓库行号为当日 `sed` 实测，可信。

## 附录：实际抓取成功的 URL 清单

**已读原文**：
- https://github.com/ckcz123/magic-tower-AI （README）
- https://github.com/ckcz123/magic-tower-AI/tree/master/AI/src （文件清单）
- https://raw.githubusercontent.com/ckcz123/magic-tower-AI/master/AI/src/Graph.java
- https://raw.githubusercontent.com/ckcz123/magic-tower-AI/master/AI/src/State.java
- https://raw.githubusercontent.com/ckcz123/magic-tower-AI/master/AI/src/Hero.java
- https://raw.githubusercontent.com/ckcz123/magic-tower-AI/master/AI/src/Node.java
- https://raw.githubusercontent.com/ckcz123/magic-tower-AI/master/AI/src/Util.java （伤害公式，早前轮次）
- https://github.com/hhh42/ferroplan （README）
- https://raw.githubusercontent.com/hhh42/ferroplan/main/crates/ferroplan/src/api.rs
- https://raw.githubusercontent.com/hhh42/ferroplan/main/crates/ferroplan/src/tresolve.rs
- https://raw.githubusercontent.com/hhh42/ferroplan/main/crates/ferroplan/src/partition.rs （早前轮次）

**仅搜索摘要**（§3/§1.8 引用）：
- https://acm-stag.literatumonline.com/doi/10.5555/1622394.1622404 （FF, JAIR 2001）
- https://gki.informatik.uni-freiburg.de/papers/richter-westphal-jair10.pdf （LAMA/landmark）
- https://www.jair.org/index.php/jair/article/download/10460/25075/ （SGPlan partition）
- http://wah.cse.cuhk.edu.hk/wah/Wah/papers/C152/C152pres.pdf （SGPlan slides）
- https://ojs.aaai.org/index.php/ICAPS/article/download/3499/3367/ （CEGAR planning）
- https://ojs.aaai.org/index.php/ICAPS/article/download/31459/33619 （抽象启发式，引 Helmert'06）
- https://www.sciencedirect.com/science/article/am/pii/S0004370222001473 （BOA\* Pareto 支配）
- https://ijcai.org/proceedings/2023/850 （Lazy 多目标支配检查）
- https://github.com/jetd1/MogicTower 、 https://github.com/ThinerDAS/dasstudio2016 （同域参考，README 摘要）

**抓取失败**：`http://ckcz123.com/blog/catagory/magic-tower-ai/`（403/404，wayback 类代理均不可达）；`https://raw.githubusercontent.com/hhh42/ferroplan/main/BORDERS.md`（404）。
