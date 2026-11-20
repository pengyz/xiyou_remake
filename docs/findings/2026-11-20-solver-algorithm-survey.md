# 全局求解器算法外部调研

> 日期：2026-11-20 ｜ 作者：技术调研员（agent-16）｜ Confidence 说明见各节
> 本文件为**考证记录（findings）**：外部结论按"已找到来源（A/B 级 = 有公开原文链接）/ 未找到（C 级推断）"区分。所有链接访问日期均为 2026-11-20。
> 只做调研，不改代码、不 commit、不 push。

---

## 1. 魔塔（Magic Tower / Mota）现有求解器

### 1.1 ckcz123/magic-tower-AI（Java，2017）—— 同类项目中最完整的公开实现

- 链接：https://github.com/ckcz123/magic-tower-AI （访问 2026-11-20）
- 算法说明博客：http://ckcz123.com/blog/catagory/magic-tower-ai/ （链接在 README 中给出；**博客域名 2026-11-20 已无法解析**，视为失效——未找到正文）
- 星数/活跃：README 未显示 star 数（页面快照未含）；最后 commit 约 2017-10（搜索快照时间戳）。**不活跃**。
- 核心思路（据 README + 可见源码结构）：把整塔地图一次性输入 AI，Java 求解"最佳路线"。分块思想：把地图按"怪物/门"划分连通块，块与块构成图，在图上做搜索。自定义地图格式，含商店等元素的数据说明（README 提及"地图数据说明"章节）。
- 商店消费/道具时点/回溯：README 未明示；从"找最佳路线"的表述与自定义地图含商店推断其模型含商店，但**道具使用时点是否作为决策变量未见证据**。Confidence：中。

### 1.2 jetd1/MogicTower（C++，2016）

- 链接：https://github.com/jetd1/MogicTower （访问 2026-11-20）
- 课程项目级实现。关键设计（README 自述）：**每层用图（GraphNode）、层与层之间用树组织**；节点只对**门和怪物**建模（它们决定连通分支数），普通地板不进图。先用 BFS 找当前封闭区域内的 `next` 节点。
- README 记录了一个与我们直接相关的教训：**朴素 BFS 会把增益物品"归属"给最近的怪物节点，导致对高损怪物+高增益组合的错误评估**——即"拾取物归属/执行顺序"不能在单层静态图上贪心决定。这从外部印证了我们"单层硬约束/贪心会制造假不可行"的架构结论。
- 商店消费/回溯：README 描述含"一层一层向下搜索与回溯"，即支持**层间回溯**；商店与道具时点未见专门处理。Confidence：中。

### 1.3 ThinerDAS/dasstudio2016（C++，2017）

- 链接：https://github.com/ThinerDAS/dasstudio2016 （访问 2026-11-20）
- README 明确主张：**魔塔是 NP-complete 问题的一个实例**（"magic tower is an instance of NP-complete problem. We will put the proof in the end"——**证明最终未随 README 提供，未见正式出处**，按 C 级假设对待；不过该主张与 §2 的 key-door 复杂度结果方向一致）。作者自称 AI 代码注释不足、"连自己都难读"，计划除 AI 外转 Python。可复用价值低，但**"NP-complete + 多数实例非病态"的经验判断**与我们逐层接力的工程选择一致。Confidence：低（证明缺失）。

### 1.4 Fancc666 魔塔网页版（JS，2023）

- 链接：https://github.com/Fancc666/Fancc666.github.io/tree/master/magictower ；文档：https://www.565455.xyz/docs/19-%E9%AD%94%E5%A1%94.html （访问 2026-11-20）
- 2023-08 新增"无限模式和求解器"。算法细节文档中未见说明（只到"求解器"功能层面）。Confidence：低。

### 1.5 人工攻略语料（对照用，非算法）

- TapTap 魔塔合集版论坛的"10093 完美路线"系列（2025-12）：https://www.taptap.cn/moment/753721457585949732 、https://www.taptap.cn/moment/753722682637288477 （访问 2026-11-20）
- 7001 攻击极限路线帖：http://www-v2.7723.cn:8000/hk/strategy/409444.html （访问 2026-11-20）
- 价值：人工最优化路线通篇都是"**先杀谁→得金→立刻消费→属性变化改变后续损血→留钥匙/留血瓶到特定时点**"的时序经济链（如"15楼剑盾在最后拿十字架找仙子的时候再买"、"留血瓶越多越好"、"杀一只买一次属性"）。**这些正是"最优性对时序敏感"的活例证**，可直接作为我们 witness 链路层的测试期望样例。

**小结（问题1）**：没有找到支持"商店消费 + 道具使用时点 + 层间回溯 + 全局最优"四要素齐备的开源求解器。最接近的是 ckcz123/magic-tower-AI（整塔建模+路线搜索），其**连通块建图 + 块图搜索**思想可复用；MogicTower 的"门/怪物为图节点"抽象与我们的单层 BFS/Pareto 同构。适配度：**中**——思想可借，工程上我们的分层接力+逐帧回放架构在确定性验证上已超出这些项目。

---

## 2. 键门图（key-door graph）复杂度与算法

### 2.1 复杂度结果（已证实，有正式论文）

- **Helmert 2001（ Diplomarbeit，Basel）**：https://ai.dmi.unibas.ch/papers/helmert-diplom-2001.pdf （访问 2026-11-20）。用"clausal barrier / variable barrier"构造把 **3SAT 归约到 key-door 网格规划问题**，证明最短计划存在性 NP-hard；同时该文给出把这些 key/door 状态纳入启发式搜索（state = pos × key inventory）的实用算法框架。这是与我们"state = (格, 已捡, 已开门, 已杀)"分层 BFS 最直接的学术对应。
- **Zelda 机制复杂度**：Bosboom, Brunner, Coulombe, Demaine et al., *The Legend of Zelda: the complexity of mechanics*, Thai J. Math. 21(4):687–716 (2023)；PDF：https://erikdemaine.org/papers/Zelda_TJM/paper.pdf （访问 2026-11-20）。对钥匙/锁、按钮、坑洞等机制逐一给出 P / NP-complete / PSPACE-complete 分类；钥匙-锁机制归约属 NP-complete 一档。**对魔塔的意义：钥匙（不可再生）+ 门（单向消耗）机制本身不引入 PSPACE，仍落在 NP 域**——支持我们用逐层确定性搜索而非对抗/无限时域方法。
- **Baba Is You（FUN 2024）**：https://drops.dagstuhl.de/storage/00lipics/lipics-vol291-fun2024/LIPIcs.FUN.2024.1/LIPIcs.FUN.2024.1.pdf （访问 2026-11-20）。有限关卡 PSPACE-hard，无限关卡 RE-hard。远超魔塔（魔塔规则固定、不可改写），仅作上限参照。
- **推箱子（Sokoban）**：PSPACE-complete 是经典结果（KIT 课程讲义引用：https://formal.kastel.kit.edu/teaching/FormSysWS0910/praxis1.pdf ；推块类 NP-hard 汇总：https://erikdemaine.org/pushingblocks/ 、https://www.inf.ethz.ch/personal/hoffmann/pub/dh-pbnns-01.pdf ，均访问 2026-11-20）。

### 2.2 实用算法

- Helmert 2001（同上）即 **pos × keyset 显式状态 + 启发式搜索**的代表；复杂度随 key 种类指数、随网格规模多项式——与我们 20 件以内拾取 DP 的容量判断一致（拾取物枚举 2^20 ≈ 1e6 是可承受上限，钥匙种类是同一枚举维度的另一个实例）。
- **Symbolic search / BDD**：规划领域的 BDD 符号搜索综述（含 Sokoban 编码练习、relation product、symbolic PDB）：Edelkamp 等启发式搜索教材节选 https://github.com/manjunath5496/Heuristics-Books/blob/master/heu(14)..pdf （访问 2026-11-20）；Torralba "Large-Scale Symbolic Search" 幻灯：https://pdb18.zib.de/09_Alvaro_Torralba.pdf （访问 2026-11-20）。后者明确指出 BDD 表示的**适用边界**："对置换谜题（N-puzzle）与网格相邻关系（Connect-4、Snake）通常不紧凑"——即**对魔塔这类以网格移动为主的状态，BDD 收益不确定**，不应列为优先方向。Confidence：中（有原文，但未对魔塔型状态实测）。

**适配度：高**（Helmert 式显式 keyset 状态）/ **低**（BDD 路线）。可直接借鉴：keyset 维度与拾取集合并入同一支配比较框架；保留"NP 域"认知以拒绝过重的求解器选型。

---

## 3. 资源约束最短路 / 带背包的路线规划（RCSP / Orienteering）

### 3.1 RCSP 标准解法

- **Feillet et al. 2004 式 label-setting + dominance**（elementary RCSP）：教材/论文中的要素已在我们可见来源反复出现：label = (path, cost, resources…)，**支配条件 = 全资源分量不劣 + 目标成本不劣**；多资源时支配效率骤降、label 爆炸（法文综述明确讨论"资源数大时 dominance 消除子路径的机会变少、状态空间迅速膨胀"：https://core.ac.uk/download/213622937.pdf 、https://theses.hal.science/file/index/docid/603891/filename/these_sauvanet.pdf ，均访问 2026-11-20）。这正是我们 hp/kills/steps/gold 四维 Pareto 支配的标准形态；**学术界的缓解手段 DSSR（declement-state relax-and-reinsert 迭代放松元素性约束）**在新综述中被用于 elementary RCSP（Springer 4OR 论文：https://link.springer.com/content/pdf/10.1007/s10479-026-07457-7.pdf ，访问 2026-11-20）——若我们逐层搜索 label 爆炸，DSSR 是第一个可试的升级。
- **拉格朗日松弛**：Carlyle, Royset, Wood, *Lagrangian relaxation and enumeration for solving constrained shortest-path problems*, Networks (2008)：https://dlnext.acm.org/doi/10.5555/1464273.1464284 （访问 2026-11-20）。思路：松弛资源约束求拉格朗日下界，再枚举"近最短路"闭 gap。MIT 课程讲义给出标准形式：https://dspace.mit.edu/bitstream/handle/1721.1/74617/15-082j-spring-2003/contents/lecture-notes/21lagrangian_relaxation_2.pdf （访问 2026-11-20）。
  对我们的适配：**低-中**。魔塔的"资源"（血/金/钥匙）不是路径边的侧约束而是**状态分量**，且约束多为硬性（血≥伤害）而非预算型；拉格朗日更适合"血预算软限"近似上界估计，可作 hp 上界剪枝的加强，不宜作主求解器。

### 3.2 与我们结构对应的变体判定

- Orienteering Problem（OP）综述：Vansteenwegen et al., EJOR (2011)：https://www.sciencedirect.com/science/article/abs/pii/S0377221710002973 ；2025 最新综述（含 exact B&C / 元启发对比表）：https://arxiv.org/html/2512.16865v1 （均访问 2026-11-20）。
- **Routing problems with profits** 分类：https://core.ac.uk/download/pdf/32241014.pdf 、Springer 章节 *Other Orienteering Problem Variants*（时变收益/随机收益/库存-路线联合：https://link.springer.com/chapter/10.1007/978-3-030-29746-6_8 ，均访问 2026-11-20）。
- **判定**：我们的问题结构 = **Selective routing with profits（OP 家族）+ 库存/状态反馈**：
  - 杀怪损血 = 访问成本；金/经验 = 收益 → **Orienteering / Prize-collecting**；
  - 商店 = **收益的延迟转化点**（时变 profit + 库存管理）→ 文献中最近的命名是 **Inventory Orienteering Problem**（Springer 章节，见上）——**没有完全对口的经典变体**，"多商店、转化率随进度变化"尤其罕见；
  - 钥匙/门 = 状态反馈，OP 文献不覆盖，须借 §2 的 key-door 建模。
  - 结论：**我们是"OP × key-door × 时变收益"的复合体**，任何单一标准变体都只能提供组件算法（label-setting 支配、B&C、收益时序），不能整体套用。

**适配度**：label-setting + dominance **高**（我们已在用，DSSR 是现成升级）；拉格朗日松弛 **低**；OP 精确算法（B&C/ILS）**中**（若做全局"杀怪集合选择"上界可用）。

---

## 4. 时序/阶段价值建模（时变收益、回头路、延迟执行）

- **时变收益 + 库存**：见 §3.2 的 Inventory OP 与 "Time-Dependent profits" 变体（https://link.springer.com/chapter/10.1007/978-3-030-29746-6_8 ，访问 2026-11-20）——学术上存在此方向，但**均为路线规划语境（车辆、时间窗），没有网格逐格移动 + tick 经济链的案例**。Confidence：找到的是方向存在性（B 级），未找到可直接移植的实现（如实说）。
- **可回溯网格世界中的搜索实践**：
  - Sokoban 求解器生态（CCCG 2021 综述含历史与变体）：https://projects.cs.dal.ca/cccg2021/wordpress/wp-content/uploads/2021/08/CCCG2021.pdf#page=353 （访问 2026-11-20）。
  - **MCTS_Sokoban**（Python, 2020）：https://github.com/csdankim/MCTS_Sokoban （访问 2026-11-20）——IDA*/RBFS/MCTS 三种算法 + 启发式 + **deadlock detector（死锁检测剪枝）**。经验教训：回溯型网格世界的搜索效率主要靠**死锁/不可行态剪枝**而非搜索算法本身。
  - GitHub topics/sokoban 下还有 AlphaGo-Zero 式 RL+MCTS（如 `DeepL2/Sokoban` 类项目，见 https://github.com/topics/sokoban?l=python ，访问 2026-11-20）——对确定性求解需求过重，不适合我们（无随机、需可证明最优）。
  - "回头路"处理：Sokoban 文献的标准答案是**抽象成高层"推动"动作（macro move）+ 像素级行走用最短路补全**（同一 CCCG 综述与 sokobano.de 惯例）。对魔塔的直接类比：**决策图不必逐格——以"事件节点"（怪/门/道具/商店）为决策图顶点，行走代价用曼哈顿/可达性最短路折叠**。这与我们"门/怪物决定连通分支"的观测（§1.2 MogicTower 亦然）一致，可作为 witness 链路层的事件图化依据。

**适配度：中**。可借鉴点：①事件图折叠（决策不在地板上）；②死锁/支配剪枝优先于换算法；③ OP 文献的"时变收益需把转化时点并入状态"与我们"使用时点是决策变量"的结论相互印证。

---

## 5. 通用求解器适配性（CP-SAT / PDDL / SMT）

- **SAT/BMC 解 Sokoban**（AECE 2016）：https://aece.ro/displaypdf.php?year=2016&number=3&article=13 （访问 2026-11-20）。结论原文要义：即使现代 SAT 求解器，**标准 BMC（逐步 unroll）公式只能解极浅关卡**；plan-as-SAT 对"移动步数大"的网格问题不经济。
- **CP/SAT 对网格谜题的新编码**：Bofill et al., *On Grid Graph Reachability and Puzzle Games*（arXiv 2310.01378，后发表于_constraints_）：https://arxiv.org/abs/2310.01378 （访问 2026-11-20）。提出**可达性（reachability）子句编码**并把"行走"与"对象操作"分层——经验上这种分层编码明显优于逐步编码。**对我们**：把"从事件 A 走到事件 B"编译成可达性约束而非逐步动作，正是该文核心。适配度：中（若未来想用 CP-SAT 做全局上界/交叉验证器，此编码是起点）。
- **PDDL classical planning**：Fast Downward（Helmert 2006）等规划器对 Sokoban 域可直接建模，实测均值 ~14s/题、最长 ~40s（学位论文实测表：https://d-nb.info/1366372271/34 ，访问 2026-11-20）；ICAPS 工具论文（SymK 符号规划、ENHSP 数值规划，含 OneShot/Anytime/Optimal 三模式）：https://ojs.aaai.org/index.php/ICAPS/article/download/42813/50373/46914 （访问 2026-11-20）。**但**：PDDL 的 cost 模型表达"杀怪→得金→商店转化率随进度变化→幸运金币时点重定价"这类**乘性、时点敏感的经济链**需要大量条件效果与数值 fluent，建模成本高且最优性保证弱（`numeric` planner 的 optimality 不如我们的确定性逐层搜索）。适配度：低-中（适合做**独立交叉验证器**，不适合做主求解器）。
- **有没有人用 SAT/CP 解魔塔**：**未找到**（GitHub、论文检索均无）。如实记录：截至 2026-11-20 无公开先例。Confidence：检索覆盖 GitHub/Google/ArXiv，未覆盖中文非公开资源。

---

## 6. 回放验证范式（求解器出路线 + 确定性引擎重放验证）

- **未找到与我们同构的公开学术案例**（"solver proposes, deterministic engine disposes" 两段式架构的直接文献）。最接近的旁证：
  - **GAOS SDK**（devpost 项目页，2026-07）：https://devpost.com/software/zonoid-labs-agi-arena （访问 2026-11-20）——自述确定性回合制网格游戏提供"…solving, and replay verification"。工程级旁证（非论文），Confidence：低-中。
  - **hayao-js VERIFICATION.md**（2026-07）：https://github.com/hellojanpacan/hayao-js/blob/main/docs/VERIFICATION.md （访问 2026-11-20）——明确规则：**winnability 求解器用纯 `Puzzle<State,Move>` 模块（零引擎依赖），再由独立引擎层验证**，与"确定性对手 ⇒ 单智能体 BFS"的论证同我们 game-core 无 I/O 约束一致。
  - Sokoban 社区的长期惯例：求解器输出 replay 文件，由独立 checker（如 ysokoban/sok solution checker）逐 move 校验——存在但散落各工具 README，无统一文献。Confidence：中。
- 我们自己的架构（逐帧回放 + strict 上行 witness 链）在确定性验证强度上**强于**上述旁证；外部经验教训可归纳为一条：**求解器与验证器必须共享同一规则层定义，且规则层以引擎为唯一真相源**（与我们 D5 一致）。

---

## 7. 建议吸收清单（按 收益/成本 排序）

| # | 建议 | 来源 | 落点 | 收益/成本 |
|---|---|---|---|---|
| 1 | **事件图折叠**：决策图以怪/门/道具/商店为顶点，行走代价用可达性最短路折叠；地板不进搜索状态 | Sokoban macro-move 惯例（CCCG 2021 综述）；MogicTower 的门/怪物建图（§1.2）；Bofill et al. reachability 编码（§5） | witness 链路层（事件图化后接力） | 高 / 低 |
| 2 | **人工极限攻略语料 → witness 金样**：把 TapTap/7723 的 7001/10093 路线中"留血瓶、延迟买剑盾、杀一只买一次"的时序决策转成链路层测试期望 | §1.5 攻略帖 | witness 链路层 + 回归金样 | 高 / 低 |
| 3 | **DSSR 式支配放松**：若单层 Pareto label 爆炸（资源维 >4 或拾取 >20），先试 DSSR 迭代放松而非换框架 | Feillet 系 RCSP 文献 + 4OR 2026 综述（§3.1） | 单层搜索（BFS/Pareto 模块） | 中 / 中 |
| 4 | **MogicTower 的"归属错误"教训**：拾取物不得静态归属最近怪物；增益评估必须随拾取顺序重算——与我们"单层硬约束制造假不可行"结论互相印证，可写成架构文档引证 | https://github.com/jetd1/MogicTower README（§1.2） | 架构文档 / 规则层注释 | 中 / 低 |
| 5 | **OP 上界估计器（可选）**：用 orienteering 的 B&C/贪心比率给"全局杀怪集合收益"做上界，供 hp/收益剪枝 | OP 综述（Vansteenwegen 2011；arXiv 2512.16865）（§3.2） | witness 链路层（上界函数） | 中 / 中 |
| 6 | **CP-SAT 交叉验证器（远期）**：若需要独立实现验证最优性，用 Bofill 式 reachability 编码在 OR-Tools CP-SAT 里复算单层，不改主求解器 | §5 Bofill et al.；SAT/BMC 不经济的教训（AECE 2016） | 规则层 + 新验证工具（不动 game-core） | 低-中 / 高 |

**不做清单**（同样重要）：BDD/symbolic search（网格相邻关系 BDD 不紧凑，§2.2）；PDDL 主求解器（经济链建模成本高、最优性弱，§5）；RL/MCTS（需求是可证明最优，§4）；拉格朗日松弛主算法（资源是状态分量非边约束，§3.1）。

---

## 8. 没查到的方向（如实列出）

1. ckcz123 的算法博客正文（域名失效），magic-tower-AI 的具体搜索策略细节只能从 README 与仓库结构推断。
2. dasstudio2016 声称的魔塔 NP-complete **正式证明**（README 承诺但未随附，论文库未见）。
3. "用 SAT/CP 直接解魔塔"的公开先例。
4. 与我们完全同构的"solver + deterministic replay 验证"学术文献（只有工程级旁证）。
5. Fancc666 求解器的算法说明。
6. 中文非公开资源（贴吧、QQ 群共享等）未覆盖。

## 9. Confidence 汇总

- **A 级**（有公开原文可复核）：§2 复杂度论文（Helmert 2001 / Zelda 2023 / FUN 2024 / 推块 NP-hard）、§3 RCSP/OP 文献、§5 SAT/CP/PDDL 文献。
- **B 级**（仓库 README/页面快照推断）：§1 各魔塔求解器的算法描述与活跃度。
- **C 级**（缺失或推断）：dasstudio2016 的 NP-complete 证明、"魔塔求解器支持商店+道具时点+回溯的完整度"的细节判断、§6 的工程旁证强度。
