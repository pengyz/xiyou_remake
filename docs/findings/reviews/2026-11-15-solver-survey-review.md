# 对抗复核：外部求解算法调研报告（2026-11-15）

- 对象：`docs/findings/2026-11-15-external-solver-algorithms-survey.md`（182 行，untracked，作者 ba95f4b6）
- 复核：子 agent 0c6f8464（只读+仅写本文件），方法：sed/awk 行号锚逐条核真 + curl raw.githubusercontent.com 复抓原文 + grep 仓内反证
- 总判：**需修订（黄）**——事实面几乎全真、证据分级诚实，但 4 处论证/措辞需修后可入库并入批4 设计

## 逐面实测

### 1. 行号锚抽验（11 处 ≥8，全部核真，0 处伪锚）
| 锚 | 实测 | 判 |
|---|---|---|
| solve_global.rs:5 | `//! 传播 + K₀ 去重 + Pareto + 有界 + 确定序。判据「逐层结果拼接==全局解」` | ✅ |
| solve_global.rs:154 / :166 | `fn dominates` / `fn weakly_ge` | ✅ |
| solve_global.rs:314-315 | `BinaryHeap<(Reverse<u32>, u32, Reverse<u32>)>` + `push((Reverse(dist_root), 0, Reverse(0)))`；:566 `push((Reverse(d), depth, Reverse(new_id)))` | ✅ 建议4 所述现键属实 |
| solve_global.rs:335 / :78 / :353 | node_cap 判断 / `truncated_layers` 字段 / `push(t)` | ✅ |
| solve_global.rs:451 | `} else if hit_bound \|\| node_cap_hit \|\| !truncated_layers.is_empty() {`（bounded-best 字符串在 :453，引条件行可接受） | ✅ |
| solve_local.rs:754 / :783 / :787 | `struct Key {`（pos/ents/flags/keys/bag/atk/def）/ weakly_ge / dominates | ✅ |
| solve_local.rs:349 / :1143-1159 | `pub fn solve_layer(` / run_root doc「层间边携带上一层终态」；keys/bag 经 PlayerSpec(:142-153) 携带 | ✅ |
| solve_global.rs:4-6 | :4「入场格→本层出口的 Pareto 选项」成立；但「四类层间边」枚举实际在 :364-367（ExitKind::Stair\|Script\|Cloud\|UseItem） | ⚠️ 轻微失准 |
- 旁证：「8 层各撞 20 万截断、全程解未出」与 `docs/findings/reviews/2026-11-13-solver-batch3-review.md:353` 记录的 `truncated_layers:[[54,200000],[3..9]...]`（8 层）+ goals:[] 吻合 ✅

### 2. 五条建议攻审（方案 B 合规性 + 判定）
| 建议 | 方案B | 判定 | 复核意见 |
|---|---|---|---|
| 1 贪心闭包（需改造） | ✅ 合规（伤害/掉落直调真引擎） | 成立-黄 | 三查询参数化论证成立；缺「我们现状如何处理道具/零伤怪/商店」的对照锚，增量价值未实测 |
| 2 landmark 格点化（需改造） | ✅ 合规（只用网格+门怪清单，不重算战斗） | 成立-绿 | §3 表中唯一可在方案B内落地的文献技术，自洽 |
| 3 图缩点（需改造） | ✅ 合规（输入侧变换，route 展开回真实格序） | 成立-绿 | 判据 solve_global.rs:5 引用正确 |
| 4 tiebreak「直接可用只改扩展序」 | ✅ 合规 | **成立-黄，措辞需降级** | 现键 `(Reverse(dist_root), depth, Reverse(id))` 属实（:314-315/:566）。但：(a) 「决策数」在 GSt(:88-102) **无对应字段**（仅 kills/steps/depth，无开门/决策计数）→ 需新增计数或代理，非纯改序；(b) :303-308 教训链显示排序改动曾两次回归（b41 浅层扇出淹没、f7/f8 深徘徊），「直接可用」偏乐观，报告未点出该教训链；(c) 现 tiebreak 第 2 维 depth 为**降序**（max-heap），换「决策数升序」是方向反转 |
| 5① 粗位图+hp 支配不安全（不适用） | ✅ 结论正确 | **成立-黄，两处论据需修** | 结论方向对：State.java:91-100 hash 仅给有门/有怪节点计 bit，不含 keys/bag/flags/atk，照搬破 K₀ 安全域。但 (a) §1.2 注「put 无条件覆盖（未做 max 合并）」与所引代码**自相矛盾**——Graph.java:317 `if (map.getOrDefault(hash,-1)>another.getScore()) continue;` 恰是 max 保护（旧>新则跳过），实际=同位图保留最高 hp；(b) 「其状态只剩 hp 一个自由维度」过强——keys/atk/def/money 同样不进 hash，其支配理论上同样有损，靠塔设计兜底 |
| 5② FF/CEGAR 不适用 | ✅ 判定正确 | 成立-绿 | 需 fact/抽象转移=第二套游戏模型，违反方案 B |

### 3. 外部事实抽查（curl raw 实抓，全部 HTTP 200，防杜撰通过）
- magic-tower-AI：`STOP_COUNT=45`（State.java:15）、shouldStop（:82-89）、hash 位图（:91-100）、eatItem 不动点+shop while（:54-72）、队列比较器 cnt升序/score降序（Graph.java:275-278）、`//break;` 注释（:286-291）、should-extend 整块被注释（:299-311）、mergeNode（:196-209）、`--nomerge/--noeat`（Main.java:11-12）、shouldEat（Node.java:127-140）、getScore=hp（Hero.java:26-29）——**全部与报告引文吻合** ✅
- ⚠️ shouldMerge（Graph.java:211-229）：共享邻点是 `return false` **排除条件**（:216-218），团条件是各自邻域内部互连（:219-227）。报告 §1.6「其余要求共享邻域满足团条件」**失准**（黄-轻）
- ferroplan：api.rs:1157-1160 doc 引文逐字吻合、Decomposition 五字段（:279-289）、tresolve.rs:1-14 文件头、EPS=0.001（:25/:319）、monolithic_fallback（:130-139）、sibling protection（:223-230）、merge 递减终止（:353-354）、partition.rs:28 最细粒度句——**全部吻合** ✅
- 未核到：raw `README.md` 404（报告引的是 github HTML 页面；其三子项目/test/ 主张未独立复核，标未验）

### 4. §3 对照表（grep 仓内反证）
- 「无」项全部属实：`crates/game-solver/src/` 内无 fallback/回落/coarsen、无 landmark/下界/lower-bound、无缩点/condense/merge_node、无贪心闭包/不动点（唯一 closure 命中为 E0515 无关注释）✅
- 「有」项（dominates/weakly_ge/node_cap/bounded-best/分层）行号全中 ✅
- ⚠️ 两处措辞：「四类层间边」锚 :4-6 不支撑「四类」（实枚举在 :364-367）；`ents_digest` 沿用代码注释名，实为 `Vec<EntRec>` 4 字段精简记录（solve_local.rs:704-709）非哈希摘要

### 5. §5 未竟面（诚实性）
- 抓取失败复现：ckcz123.com 博客 http/https 均 403 ✅、raw BORDERS.md 404 ✅、web.archive.org curl 超时不可达 ✅——报告如实列，无「没读说成读了」✅
- 通篇【原文】/【摘要】标注，§3 自declare「搜索摘要未读论文全文」✅ 分级诚实

### 6. 夹带
- `git diff HEAD` 空、0 修改 0 暂存；untracked 均先于本会话（android/build mtime 11-10、既有 reviews ≤11-14），今日唯一新文件=被复核报告本身 ✅ 无码/计划改动。注：「仅该 1 个新文件」严格说不成立（存在先在 untracked），但今日新增仅此一个。

## 结论
**需修订（黄）**。4 处必修后可入库并入批4 设计：
1. §1.2 删/改「put 无条件覆盖（未做 max 合并）」——与所引 Graph.java:317 矛盾，实为 max 保护；
2. §1.6 shouldMerge 改为「共享邻点⇒不并；团条件=各自邻域内部互连」；
3. §4 建议4 降级「直接可用」→「需改造」：GSt 无决策数字段需新增/代理，且必须点名 solve_global.rs:303-308 排序教训链（tiebreak 改动两次回归史），落地须 A/B 对拍批0-3 判据；
4. §4-5① 弱化「其状态只剩 hp 一个自由维度」为「其位图不含资源维度、支配同样有损但靠塔设计兜底」。
另建议顺手修：§3「四类层间边」锚改 :364-367；ents_digest 标注为精简记录非哈希；建议1 补现状对照锚。

Confidence: 0.90（行号锚 11/11 核真、外部原文两库 curl 复抓全吻合、§3 反证 grep 全中；扣分项=README HTML 页未独立复核、magic-tower 仓库单文件级抽查未跑其测试塔）
