# 求解器正确性补正：对上一轮复核的独立判定 + 可执行修复方案

> 日期 2026-11-17 · HEAD `187ab46` · 复核对象 `docs/findings/reviews/2026-11-17-solver-implementation-review.md`（Confidence 78）
> 本文只给结论与施工单，**未改任何代码**；§5 的 S0–S8 **全部未执行**（状态=待执行，验收见 §8）。凡数字均给可复现命令或 file:line；拿不到证据的条目写「未能证实」，不猜。
> 证据级别沿用 AGENTS.md §3：〔A〕硬证据（字节/引擎代码/运行时输出）／〔B〕静态推理〔C〕假设。

## 0. 独立复算设施（本文所有数字的来源）

三个一次性脚本，只读资源与源码，**不复用 game-solver 的判定代码**（否则是自我循环）：

| 脚本 | 作用 | 校验 |
|---|---|---|
| `/tmp/aud2/parse.py` | 按 `tools/route_gen.py:53-60` 规则解 `maplv`+`sprite`：头 u16>>1=每行格数、g1 行距 `wide*4` 列距 2、`tile_walkability.txt` 判可走；sprite 逐记录 `type u8 / x,y u16 / extra_len(type)`；坐标 `(x+16)>>5` | 全 54 层复算：门 309、钥匙 黄176/红6/蓝21 —— 与 `docs/findings/mandatory-doors.md`、`key-flow.md` 逐值吻合〔A〕 |
| `/tmp/aud2/bfs.py` | 复刻 `enums.rs:245-263` 类目 + `solve_local.rs:1202-1290` 分类，三模式 `nomons`/`kill`/`walls_open` | 复刻结果与 `solve_local` 分类逐格一致〔B〕 |
| `/tmp/aud2/doors_replica.py` | 复刻 `doors.rs:68-186`（`entry_and_goals`+`can_reach`+`analyze_floor`），可切三个通行变体 | 变体 A 输出与 CLI 完全一致：门 308（f49 不计）/必经 33/同色必要 172·4·0〔A〕 |

CLI 三份实测（只读）：
```
cargo run --release -q -p game-solver -- doors --out /tmp/aud2/doors.json
  == 合计 == 门 309 扇 / 必经 33 / 同色必要 176 / 判定无效层 1   # f49 打「⚠无出发型出口」
cargo run --release -q -p game-solver -- keys  --out /tmp/aud2/keys.json
  必经模式：需求 黄29 红4 蓝0 / 供给 黄176 红6 蓝21 / 三色全程不断供 / no_exit_floors=[49]
  全门模式：需求 246/18/45，黄 f1 断供、红 f25、蓝 f2
```
> 注：`doors` 打印的「同色必要 176」是**黄列 172+红列 4 的合计**（见 §6 行 2），不是黄列单值——这一行打印本身是 F7 漂移的源头。

## 1. 对上一轮复核的独立判定

| 项 | 判定 | 我的证据与修正 |
|---|---|---|
| **F1** 通行判定两套口径 | **成立，但机制归因错** | 分歧点**只有 ty6**，与 ty7/8 无关。`/tmp/aud2/doors_replica.py` 三变体：A(现行 `terrain‖goal‖entry`)=必经 **33**；B(+ty6 豁免)=**32**；C(+6\|7\|8)=**32 且与 B 逐层输出全同** ⇒ 「6\|7\|8 都豁免」与「只豁免 6」等价，33→32 的**唯一**变化是 f14 黄门 (8,11) 消失。上一轮把「6\|7\|8」整体当分歧源，掩盖了真正的机制（喷泉格 ty6 是否可过）。另：`doors.rs:114` 注释已写「楼梯格豁免地形（solve_local.rs:1183）」而 `:115` 实现 `terrain_ok \|\| goal_set \|\| entry` **根本没做**——注释与实现自相矛盾，这是 A 级自证，不需要靠「两边不一致」推。 |
| **F2** 供给不看可达性 | **成立，但结论方向夸大、附带例证错** | 不可拾取的不是「1 枚红」，而是 **f37 一层 12 枚**：ty26 9 枚 + ty27 1 枚(7,5 红) + ty28 2 枚(4,9)(5,9)，`/tmp/aud2/keyreach.py`〔B〕。而上一轮的三条例证：f3 (4,1)(6,1) **两种口径都可达**（误判）；f7 (9,10)(9,11)、f42 (1,11)(2,11) 只在「怪当墙」口径不可达，与 doors 自述「怪可穿」口径**不自洽** ⇒ 例证不可用。真正机制：f37 的 12 枚钥匙被 ty11 圈成的 pocket 封死；怪全放行(mode `kill`)时 up 梯可达但 pocket 仍不可进，破全部 ty11(mode `walls_open`)后 121 格全可达〔A/B〕。 |
| **F3** `confidence` 短路 | **成立，但「未展开 21–50 层」的归因不足** | 短路代码属实（`solve_global.rs:526-533`）。但 q1/q2 停在 f20/f10 **不是**「层间边缺失」单一原因，而是三层叠加（见 §3-C）。更关键：目标 `Ending` 只在 `self.input.floor == 50 && ety == 72` 产出（`solve_local.rs:1650-1656`），而 f49 无上行梯、f50 无任何楼梯实体（`/tmp/aud2` 层图扫描）⇒ **f50 在模型内不可达 ⇒ goals 恒空**，与钥匙/金币无关。故「界内无解」不是预算问题，是模型不通。 |
| **F4** landmark/`floor_dist` 过剪（黄） | **成立，维持黄** | `solve_global.rs:204-213` 落点写死三例 + `:806-836` 同层取 `table[f][t]` 而非 minima，属实。当前数据不触发的判据：唯一「同层同 target 两条梯」是 f24（ty7 双出口 target 50/25），二者目标不同 ⇒ 不触发。**同意维持黄**，但把它写成 S8（可选）而非免修。 |
| **F6** 未登记近似 | **成立，但清单要改一半** | ① `script_abs.rs:348-352` `find_path→false`、`:438-439` `reload_floor_entities` 空实现，**代码内都有登记注释**（且只论证「不影响 74 金样对拍」，未论证「不影响求解器模型」）⇒ 不算「未登记」，应改判为「登记范围不足」。② 「ty11 是永久墙」**错**：引擎明示可破（见 §3-A）。③ 我新增两条未登记近似：**f29 脚本换层缺边**（§3-C）、**f37 破墙能力缺失**（§3-A）。④ Q3 把禁令格编译成 `Cell::Blocked`（`solve_local.rs:1280-1283`）改拓扑，属实。 |
| **F7** 文档漂移 | **成立，9 条里 2 条要改判** | 改判 ①「同色必要 176/4/0=180」不是漂移而是**算术双计**：代码 172/4/0=176，文档把红 4 扇同时算进黄列与红列（176=172+4，再 +4 得 180）⇒ 可精确复原。②`(10,1)` 红钥匙注释（`solve_global.rs:861-865`）不是失效注释，而是**f14 层红钥匙位置由 (10,1) 表述**，需人工确认语义（C 级，未能证实是漂移）。确认的漂移：`mandatory-doors.md:14` 行号 `solve_local.rs:1183`→实际 `:1230/:1262`；商店「13 家/八家」→实际**开价 14 家**（§3-A 表）；`solve_global.rs:668` 注释色序错（ty1→0 黄、ty2→1 红、ty3→2 蓝，注释写「0黄1蓝2红」）。 |
| **F5/F8**（黄） | 同意降级 | 未发现新反证；保留为黄，不进 S1–S7 关键路径。 |

**上一轮复核的净判定**：5 红中 F1/F2/F3/F6/F7 **全部成立**，但**4 条的关键机制描述有错**（F1 归因到 6\|7\|8、F2 只数 1 枚红、F3 归因到边缺失、F6 把「已登记」算未登记）。Confidence 78 偏高，我给 **72**（结论方向对，机制层面需要重写，且漏了「目标层不可达」这个决定性事实）。

## 2. 对三步草案的攻击

**草案 1（`confidence` 三态化）：方向对，命名仍是假话 ⇒ 必须扩成二维。**
- `"infeasible-within-search"` 仍在断言「搜索空间内无解」，而 §3-C 已证模型本身不通（f50 不可达）。用这个词替换 `"feasible"` 只是把一种谎换成另一种更精细的谎。**必须拆成 `status` × `model_complete`**：只有 `status∈{Exhausted}` **且** `coverage.complete==true` 才允许输出「无解」；否则 `unknown`。
- 草案漏了「有目标」分支的强度：`optimal`/`bounded-best` 也不该无条件打印——若 `truncated_layers` 非空，`bounded-best` 也不是最优。选项集与 `truncated_layers` 必须同屏（草案这点对，但没写进选项集定义）。
- 排序错：在模型还没修（S2/S3 未做）时先改 confidence，会把「假 feasible」换成「假 unknown」，覆盖报告仍缺 21–50 层 ⇒ 读者无法判断是模型病还是预算病。

**草案 2（通行/供给统一到 `solve_local` 的单一实现）：会引入新偏差，且论证是循环的。**
- `solve_local.rs:1262` 的 `stair_like` 把 **ty7/8 也当可过**。当前数据里 ty7/8 恰好**全是** `level.exits`（`solve_local.rs:644-673`）、踩到即记终态不扩展（`:1758-1780`）⇒ 侥幸无症状。把「终点格」编码成「通道格」是把正确性押在「所有 ty7/8 都在 exits」这个**数据巧合**上；一旦某层出现不在 exits 的 ty7/8（例如 f24 的第二扇上梯若被 exits 漏收），搜索就会穿层。
- 「顺带消掉 f14 多报」不能靠「两处一致」证（循环论证）：必须证 **33 与 32 谁对**。§3-B 用引擎语义定死 32；如果反过来统一到 doors 口径，就把 `solve_local` 的 23 层上行梯判不可达（那些梯都在不可走地形上，靠 `scene.rs:3674` 的楼梯豁免才可进入）——直接毁掉层图。
- 结论：**统一的目标不是 solve_local，是引擎终审语义**（`scene.rs:3374` + `:3674-3677`），且必须保留 `enterable` / `through` 两个标志（§4.1）。

**草案 3（红钥匙归因重写）：方向对，但「降级为未验证代理目标」不够 —— 这条因果链应当整条删除。**
- 事实：f8 在 f12 之前就提供免费红钥（`mandatory-doors.md:55` 6 处免费拾取）；f10 的红门只需 1 把；f12 花 800 买红在时序上是**纯浪费**；而 f37 的开价店（`ty77 idx15`→`shop_goods id4`）是**白给 1000 金、价格 0**〔A〕。「差 33 金」既不必要也不成立。
- 同时草案把「红钥 6→5」写死为「不可得」也是假话：玄明石（item18）唯一可买处是 **f47 ¤4000**（`shop_goods id17` + `ty78 idx13` + `door13[13]=3` 开价）〔A〕⇒ f37 红钥是**条件可得**（需回溯到 f47 买石），不是不可得。正确表述是**时序能力约束**，见 §3-A 的三值分类。

**执行顺序**：草案 1→2→3 应改为 **S1 判据单源（B）→ S2 层图覆盖（C）→ S3 供给可达性（A）→ S4 结论语义（confidence）→ S5 能力边**。理由：confidence 是**结论层**，它的正确性依赖下面三层的覆盖面；先改它只会把误导搬到新字段。

## 3. 硬问题 A–E

### A. ty11 是什么；破墙道具在哪层；f37 红钥匙在游戏里可拾吗

**ty11 = 可被道具破开的墙，不是永久墙**〔A〕：
- `crates/game-core/src/scene.rs:3543-3556`（ty11 分支）提示「只有金勺子、玄明石可以凿开。或者剧情打开！」且 `allowed=false`。
- `crates/game-core/src/item_effect.rs:36-37`：`item17 金勺子 = RemoveAdjacent{11}`、`item18 玄明石 = RemoveAllType{11}`。
- f29 的 `ty72@(3,2)` 才是**永久**阻挡（`scene.rs:3664` `32 => allowed = false`）。

**道具可得性（全 54 层实扫，A 级数据）**：
| 道具 | 地面拾取 | 商店 | 脚本 | 结论 |
|---|---|---|---|---|
| item17 金勺子 | 无 | **`shop_goods.json` 无此行动**（仅 17 行，idx 1–13） | `level_scripts.txt` 的 GIN 只有 `0_13 / 0_19 / 1_1000×2`，无 17/18 | **本数据集不可获得**〔A〕 |
| item18 玄明石 | 无 | **仅 `id17 = (src1, idx13, 4000, PickupItem item18)`**；`ty78⇒src1`（`solve_local.rs:1191-1200`），`src1/idx13` 的唯一开价点在 **f47**（`door13[13]=3` ⇒ flags&2 开价，`scene.rs:2645-2745` 语义） | 同上，无 | **只能 f47 花 4000 金买到**〔A〕 |

旁证：f36 也有 `idx13` 但它是 `ty77`（`src0`），`src0` 无 idx13 行 ⇒ 无货；宝箱 ty5 是**锁**（`scene.rs:820-834` `try_open_chest` 要求同 param 的 ty4），不产出道具 ⇒ 不能作为金勺子来源。

**f37 那枚红钥匙在游戏里**：
- 正序首次经过 f37 时**不可拾取**（此时不可能持有玄明石：f47 > f37）〔A〕；
- 但 f45 (2,10) 有地面 **item21 上行符**〔A〕、f47 可买玄明石 ⇒ **理论上先下行/回溯再回来可以拾取**。是否真能回溯，取决于 item21 的使用次数与 `change_floor` 规则——**未能证实**（需查 `item_uses` 与 21/22 的 effect 分支，未在本轮取到 A 级证据）。

**因此修法二选一是个假二分**：判据层做**可达性过滤**（不做供给 = 数实体），建模层加**能力门控破墙边**（`Capabilities{has_item18}` 才允许穿 ty11）。两者角色不同、都不做才是错。keyflow 供给应分三值：
```
Supply::Always            // 起点可达，无需能力
Supply::Gated { need: vec![18], from_floor: 47, price: 4000 }  // f37 的 12 枚
Supply::Unreached { why: Blocker }                              // 其余（当前为 0）
```

### B. 33 还是 32 对（不许「不一致所以统一」）

**32 对。33 是错的**，三条独立证据：

1. **引擎 A 级**：`scene.rs:3674-3676` `if allowed && stair_count <= 0 { allowed = self.walkable[y][x]; }` ⇒ **格内有楼梯实体（ty6\|7\|8）时豁免地形终审**。这条豁免是承重的：54 层里 **23 层的上行梯座落在不可走地形上**（`/tmp/aud2/upreach2.py` 未豁免时 23 层报不可达，豁免后只剩 f29 真断）⇒ 没有这条豁免，层图当场崩掉。这与 `solve_local.rs:1262` 的 `stair_like` 同义，不是「求解器的私有近似」。
2. **数据**：f14 的 `terrain(6,9)==false` 且 `ty6@(6,9)`；该格是黄门 (8,11) 所在侧袋的唯一通路（复算：豁免 ty6 后 (8,11) 从「必经」变「不必经」）〔B〕。
3. **语义**：`scene.rs:4063-4066` `6 => self.player_bob_applied = true` —— **ty6 只做喷泉浮沉，不换层** ⇒ 它是合法走廊格；`scene.rs:4073-4085` ty7→`change_floor(param&0xff,false,false)`、ty8→`change_floor(...,true,...)` 在 `applyStepCellEffects` 内 ⇒ **踩上即换层 ⇒ 终点格**。所以正确口径是「ty6 可过；ty7/8 可进入但不可穿过」，doors 现行的「只豁免 goal/entry」把 ty6 当墙，**违背引擎**。
4. **反向自证**：若坚持 33，就必须主张「喷泉格不可通过」，而这与 ①③ 直接冲突；且 33 会让 f14 的 `(8,11)`（位于**可走地形**上的侧袋门）被记为「移除它即断绝进度」——实际效果只是绕路 15 格，不构成瓶颈。

结论：`doors.rs:114` 的注释是**正确的设计意图**，`:115` 的实现欠了 ty6 这一支；「33→32」是**修错**，不是行为变更（`solve_local` 与引擎早已是 32 的口径）。

### C. q1/q2 为什么没到 21–50 层（定位证据 + 成本）

`docs/findings/solver-queries.json` 实测（A 级产物）：
```
q1: exp 68148, bound 1e6, node_cap 100000, node_cap_hit=true, truncated_layers=[],
    floors=[1..20, 51,52,53,54], goals=[]
q2: exp  8687, node_cap_hit=true, truncated_layers=[[6,200000],[7,200000]],
    floors=[1..10, 51,52,53,54], goals=[]
q3: exp   641, node_cap_hit=false, truncated_layers=[], floors=[51,52,53], goals=[]
```
**三层原因（按决定性排序）**：

1. **目标层不可达（决定性）**：`Ending` 只在 `input.floor == 50 && ety == 72` 产出（`solve_local.rs:1650-1656`）；而 **f49 无上行梯**（只有 `ty8@(1,11) param48`）、**f50 没有任何楼梯实体**〔A：全 54 层实体扫描〕⇒ f50 只能靠脚本边进入。**即使把 cap 抬到无限，goals 也只会从「未找到」变成「耗尽」，绝不会变 feasible**。这是「界内无解」判决的真正依据，上一轮完全没提。
2. **模型层图在 f29 断裂**：f29 `entry=(3,11)`、`up=(8,11)`；`/tmp/aud2/upreach2.py` 口径=地形+楼梯豁免、破全部 ty11，仍然不可达 ⇒ 两处切口：`ty11@(8,3)` 与 **`ty72@(3,2)`**（cat32 永久阻挡 `scene.rs:3664`）；同格还有 **`ty83@(3,2) param32`**（场景门，`solve_local` 判可走）⇒ 该层连通**依赖脚本改写**，不是纯拓扑。⇒ `f29→f30` 在模型内永缺边，f30–f49 不可达（与 cap 无关）。
3. **预算**：q1 的 `node_cap_hit=true` 且 `truncated_layers=[]` ⇒ 撞的是**全局节点上限**（100000 个 GNode），停在 f20 之后；q2 另有 f6/f7 的**层内**截断（200000/层）。成本量级：q1 已用 68148 expansions/100k nodes 走完 20 层，平均 ~3400 node/层，且每向后一层状态数增加（经济/钥匙维度膨胀）⇒ 21–29 层粗估还需 3–5×10⁴ node，30–49 层另需一条**目前不存在的**边。

**因此「扩大 bound/node_cap」的性质判定**：
- **不许**把抬 cap 当修复：`bound` 抬了不解决 ① ②，只会把输出从 `capped` 变成 `exhausted`；`node_cap` 抬了只是买时间。
- **允许**改的是**覆盖面**：补 ① ② 的边（每条边必须给 A/B 级来源：脚本 op 或实体），并把 `coverage`（`floors_expected` vs `floors_expanded`、`truncated_layers`、`missing_edges`）作为**结论的一等公民**打印。
- **区分判据（可执行的硬线）**：一条层间边只允许在**引擎里有对应通路**时进入模型——`ty7/8` 实体的 `param&0xff`、`ty6 param=0` 的喷泉（不换层）、脚本 StepGate/`CHG` 的 op 号、`item21/22` 的换层效果，四者之一。**不接受**「搜索需要它才能通过」作为理由；缺来源就写 `missing_edges`，让结论降级为 `unknown`，不许静默补边（`map-vs-mechanism` 规矩 4）。

### D. `confidence` 改名会不会伤下游

**消费者盘点（穷举 grep，A 级）**：
| 消费者 | 位置 | 形态 |
|---|---|---|
| CLI/query | `query.rs:49-50`（注释「0 + confidence=feasible 表示 goals 空（无解）」）、`:72-79`（console `Q1 max_hp=0(feasible)`）、`:126`（取字符串） | 选项**只剩 3 个字符串** |
| 测试 | `tests/solve_global.rs:156,237,300,452,526,596,734-740,848,1098,1136,1148,1307`（`:734-740` 断言三值集合；`:237/:526/:1098` 断言 `"feasible"`） | 13 处硬断言 |
| 文档 | 12 个文件含 `feasible`（含 `queries-after-f2.md`、`mandatory-doors.md`） | 定性叙述 |
| 门禁 | **无**（`gates/cli.py:201,215` 的 `confidence` 是命名台账字段，同名不同物） | — |

**会不会更误解**：会，如果只换词。旧读者的行为模型已经固化成「`feasible` = 有解 / 其余 = 没找到」，而 `query.rs:49-50` 的注释把这个误读**写成了规范**。换一个 `infeasible-within-search` 只会让读者以为「确定无解」。
**迁法（必须同片完成）**：
1. 新增结构化字段：`status: {solved, exhausted, capped, model_broken}` + `coverage: {floors_expanded, floors_expected, truncated_layers, missing_edges}`；
2. `confidence` **保留为 `status` 的投影**并标 `deprecated`（`"optimal" | "bounded-best" | "unknown"` 三值，**删掉 `"feasible"`**），避免第四种歧义；
3. `query.rs:49-50` 注释与 `:72-79` console 一起改（console 打印 `status` + `coverage` 一行）；
4. 13 处测试断言改为断 `status`；12 处文档按 AGENTS §3.6 **原地加日期更正**，不删旧文。

### E. 修完之后要撤回/改判的既有结论

| # | 结论（出处） | 旧值 | 新值 | 为什么 |
|---|---|---|---|---|
| E1 | 必经门总数（`mandatory-doors.md` 表 + `doors` 输出） | **33** | **32**（f14 黄 `(8,11)` 退出清单） | ty6 喷泉格是合法走廊格，§3-B 四条证据 |
| E2 | 同色必要门数（`mandatory-doors.md:14` 表） | 黄176/红4/蓝0 **=180** | 黄**172**/红4/蓝0 **=176** | 代码与复算一致给 172/4/0；文档把红 4 扇同时计入黄列（176=172+4）再重复相加得 180 |
| E3 | 红钥匙供给（`key-flow.md`） | **6 枚，全程自给** | 5 枚 `Always` + 1 枚 `Gated{f47,¤4000}`；**结论「不断供」保留**（拾/扣序：f8 1 → f10 0 → f14 1 → f20 0 → f34 1 → f37 (gated，不计) → f40 0 → f42 1 → f51 2→1，全程 ≥0，最紧处=0） | f37 pocket 需玄明石；红需求 4、无条件可用 5 ⇒ 断供结论不变，但 f37 的「冗余 +1」消失 |
| E4 | `key-flow.md`「f37 拾 +1 → 1 ← 与 f34 同为冗余来源」 | f37 是免费冗余 | **删除该行**（模型内不可拾取；可拾时需 ¤4000 且回溯） | §3-A |
| E5 | 「黄钥匙 176 严重过剩」（`mandatory-doors.md` 结论 3） | 过剩 147（176 供给 −29 需求） | 无条件可达 **167**（−9 全在 f37）⇒ 过剩 138；买到玄明石后才恢复 176/147 | f37 的 9 枚黄钥同属 pocket |
| E6 | 「界内无解 / 差 33 金是瓶颈」（`queries-after-f2.md:24`） | 差 33 金 | **撤回**：因果链失效（f8 免费红钥在 f12 之前；f12 的 800 金买红是浪费；f37 白给 1000 金）；且「无解」的真实原因是 **f50 不可达 + f29 断边 + cap**，不是金币 | §3-A/§3-C |
| E7 | Q3「不拿终极装备不能通关」 | false ⇒ 不通关 | **撤回结论**：q3 只展开 f51–53（`gold_peak_by_floor`），f54 都没进；且 `ban` 被编译成 `Cell::Blocked`（`solve_local.rs:1280-1283`）**改了拓扑**，不是「禁装备」而是「改地图」 | A 级代码 + 产物 |
| E8 | 「f29 只是被 ty11 封住」 | 补破墙边即可 | **改判**：破全部 ty11 也不通（`ty72@(3,2)` 永久阻挡 + `ty83` 场景门） | §3-C-2 |

## 4. 实现方案

### 4.1 单一通行判定（放哪一层、签名）

**放 `game-core`**（引擎与求解器唯一可能共用的层，且 `game-solver` 已依赖 `game-core`：`crates/game-solver/Cargo.toml:8-9`）。新模块 `crates/game-core/src/cell_gate.rs`：

```rust
/// 单格通行判定：所有消费者（引擎 / doors / keyflow / solve_local）唯一入口。
pub struct Caps { pub items: &'static [i32], /* 已持有道具；破墙/换层能力来源 */ }
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    Free,                                        // 地形可走、无实体
    StairTransit,                                // ty6：可走可过（不换层，scene.rs:4063）
    StairTerminal { target: i32, down: bool },   // ty7/8：可进入、不可穿过（scene.rs:4073）
    Door { color: KeyColor },                    // ty1/2/3
    Pickup,                                      // ty2/4/13..=32/79/80/85/86/81/82/83（放行）
    Shop { src: u8, idx: u8, flags: u8 },        // ty77/78
    Blocked { by: Blocker },                     // ty9/10/12/72/84/87 + 41..=78 怪
    Breakable { need: &'static [i32] },          // ty11：item17(相邻) / item18(全体)
}
#[derive(Clone, Copy)]
pub struct Access { pub enter: bool, pub through: bool }

pub fn gate_at(terrain: bool, ents: &[EntView], caps: &Caps) -> Gate;   // 纯函数，无 I/O
pub fn access(g: Gate, caps: &Caps) -> Access;
```
- `enter` / `through` 两标志是**修正草案 2 的核心**：doors 的 goal 格与 solve_local 的 exit 格用 `enter`（可踏入=终点），走廊 BFS 用 `through`（ty6 才是 `through=true` 的楼梯）。
- **引擎侧对齐**：`scene.rs:3374 interact_with_cell` 内的终审（`:3674-3677`）改为「先算 `Gate`，再 `access().enter`」，行为保持等价（S1 用等价性测试锁死 74 金样 + 现有 scene 单测）。
- **不变量（属性测试）**：`enter ⇒ through || matches!(g, Gate::StairTerminal{..})`；`Gate::Breakable ∧ 无相关道具 ⇒ !through`；`Gate::Blocked ⇒ !enter`。

### 4.2 keyflow 可达性过滤算法

```
for f in 1..=54:
    A[f] = BFS(through=access(gate_at(..)), start = entry_of(f))        // 不破墙、不计数
    supply_always[f] = pickups(ty26/27/28) ∈ A[f]
    if 层内存在 Gate::Breakable:
        A2[f] = BFS(through | breakable with caps_from_store)           // 只做诊断
        supply_gated[f] = pickups ∈ A2[f] \ A[f]  ⇒ Gated{need, from_floor=f47, price}
    else: supply_gated[f] = {}
    supply_unreached[f] = pickups(f) - A[f] - A2[f]                     // 必须逐层打印
demand 折叠（沿层链序，同层先拾后扣——口径不变，见 map-vs-mechanism 未做项 1）：
    bal[c] += supply_always[f][c]（+ gated 仅在建模开关开时）
    if bal[c] < demand[f][c]: 记 shortfall{f,c,need,short}；**不扣 demand**（修 keyflow.rs:110-117 的幻影余额）
```
- `no_exit_floors` 拆成 `no_departure_exit`（f49/f50 型）与 `entry_unreachable`（f29 型），当前分别 = {49,50} 与 {29}〔A〕。
- 新增 `--supply=always|gated|raw` 三档，**默认 `always`**（保守），`raw` 仅供与旧文档对账。

### 4.3 `confidence` 三/四态

```rust
pub enum SearchStatus { Solved, Exhausted, Capped, ModelBroken }
pub struct Coverage { floors_expanded: Vec<i32>, floors_expected: Vec<i32>,
                      truncated_layers: Vec<(i32,u64)>, missing_edges: Vec<MissingEdge> }
```
| 条件 | `status` | `confidence`（deprecated 投影） | 允许的自然语言 |
|---|---|---|---|
| 有 goals | `Solved` | `optimal` / `bounded-best`（须 `truncated_layers==[]` 才许 `optimal`） | 「找到解」 |
| 无 goals ∧ 耗尽 ∧ `coverage.complete` | `Exhausted` | `infeasible` | 「模型内无解」 |
| 无 goals ∧ 耗尽 ∧ `!complete` | `ModelBroken` | `unknown` | 「模型不覆盖，不能判」 |
| 无 goals ∧ 撞 node_cap/bound | `Capped` | `unknown` | 「预算截断，不能判」 |
| 无 goals ∧ `truncated_layers≠[]` | `Capped` | `unknown` | 同上 |

### 4.4 破墙边取舍

**不加「到 f37 的近路」**，只把 ty11 的语义补全：
1. `Gate::Breakable{need:[17,18]}` 进 `cell_gate`；`access().through` 仅在 `Caps` 含 17/18 时为真；
2. 道具使用与 `item_effect.rs:36-37` 同源：17=相邻一格、18=全层同型；消耗记 `item_uses`（与引擎同一张表）；
3. 因为 f47 是唯一来源，模型自动得到「先到 f47、再回溯」的时序，**不需要任何硬编码楼层**；
4. f29 的 `ty72` 仍是 `Blocked`——它只能靠脚本，**不许**用破墙能力绕过（否则会得出 f29 可通的假结论）。

### 4.5 q1/q2 覆盖面方案

三步，顺序不可换：
1. **先出覆盖报告**：`coverage`（`floors_expected` = 从起点沿「实体梯边 + 脚本边」闭包可达的层）与 `missing_edges` 明细表；此时不改结论，只把「模型不通」暴露出来（预期：`missing_edges=[f29→f30, f49→f50]`）。
2. **逐条补边**：每条边必须有 A/B 级来源（脚本 op 号 / 实体 param / item21/22 effect），写进 `model_edges` 表并在报告里第二列给证据指针；拿不到来源的留 `missing_edges`（结论降 `unknown`）。
3. **再谈预算**：用**分层预算** `layer_node_limit`（已有 `solve_local.rs:1749-1757` 的 `LAYER_TRUNC`）替代全局 `node_cap` 单点，并把每层 `truncated_layers` 打印；**不抬 `bound`**（抬了不改变 ① ② 的可达性）。

## 5. 切片 S0–S8

> 每片一段：改哪些文件 / TDD 先红用例 / 预期数字 / 风险 / 回滚 / 是否改生产行为。全部门禁要求见 §8。

**S0 事实基线（只读，不改码）**：把 `doors`/`keys`/`queries` 三份输出冻结为 `docs/findings/baselines/2026-11-17-solver-pre-remediation.json`（含本文 §0 三条命令）。无红用例。风险=0。回滚=删文件。**不改生产行为**。

**S1 `cell_gate` 抽取 + 引擎对齐**：新增 `crates/game-core/src/cell_gate.rs`、改 `scene.rs:3674-3677` 调用之、`crates/game-core/src/lib.rs` 导出。红用例：`cell_gate_ty6_是通道_ty7_是终点_ty11_无道具不可过`、`cell_gate_与引擎_interact_with_cell_同判_全54层逐格等价`。预期数字：74 金样 sha 全同、`doors` 输出不变（33）。风险=中（引擎热路径）；缓解=纯函数抽取、逐格等价测试。回滚=`git revert` 单提交（新模块无外部引用）。**改生产代码但不改生产行为**。

**S2 doors 消费 `cell_gate`**：改 `doors.rs:97-133`（`can_reach` 用 `through`，`entry_and_goals` 的 goal 用 `enter`），删 `:115` 的 ad-hoc 闭包。红用例：`f14_喷泉格(6,9)是通道_黄门(8,11)不再必经`、`doors_必经门总数_32`。预期：必经 33→32（唯一变化 f14 黄 (8,11)）；门 309、同色必要 172/4/0 不变。风险=低（判据层只读）；回滚=单提交 revert，数字回到 33。**改判据输出，不改游戏行为**。

**S3 keyflow 可达性过滤 + demand 扣减**：改 `keyflow.rs:88-102`（供给按 §4.2）、`:103-118`（短供不扣 demand）、`:60/:74-76`（`no_exit_floors` 拆分）。红用例：`f37_红钥匙_无玄明石时不可拾`、`f37_十二枚钥匙_逐色计数`、`短供层不得出现幻影余额`。预期：红 6→5(+1 gated)、黄 176→167(always)/176(gated)、蓝 21→19；「不断供」结论保留但红最紧处余量 0。风险=中（会改 `key-flow.md` 全部数字）；回滚=revert + `--supply=raw` 复现旧值。**改判据输出**。

**S4 层图完整性报告**：改 `solve_global.rs:364-366/:418-422` 增 `missing_edges`、`:424-429` 的 `gold_peak_by_floor` 只统计**已展开层**（保持），新增 `coverage` 输出。红用例：`f29_ty72_切口_上行梯不可达_产生missing_edge`、`f50_无楼梯实体_目标层不可达`。预期：`missing_edges=[f29→f30, f49→f50]`、`coverage.complete=false`。风险=低。回滚=单提交 revert。**只加诊断，不改搜索**。

**S5 `status`×`coverage` 二维化**：改 `solve_global.rs:526-533`（§4.3 映射）、`query.rs:49-50,72-79,126`、`tests/solve_global.rs:237/526/1098/734-740` 断言、12 处文档原地更正。红用例：`撞node_cap时不得输出feasible`、`goals空且模型不完整时输出unknown`、`truncated_layers非空时不得输出optimal`。预期：q1/q2/q3 的 `confidence` 由 `feasible` → `unknown`，`status` = `Capped`/`ModelBroken`。风险=高（13 处测试 + 12 处文档）；回滚=单提交 revert（`confidence` 投影保留，可回滚到字符串）。**改结论语义**。

**S6 破墙能力边**：`cell_gate`（`Breakable` 已含）+ `solve_local.rs` 的 `post` 增 `caps`（或复用既有道具位）、`solve_global` 状态键纳入 caps。红用例：`无玄明石时_f37_pocket不可达`、`f47买到玄明石后_f37红钥匙可达_且路径经过f47`。预期：f37 红钥从 `Gated` 变 `可达`（仅在买了石的分支）；主路径数字不变。风险=高（状态空间膨胀，可能触发新 cap）；缓解=先只开 `--caps=on` 单测、不并入 q1/q2 默认。回滚=flag 关闭即旧行为。**改建模能力，非生产行为**。

**S7 重跑 q1/q2/q3 + 结论重述**：更新 `queries-after-f2.md`（撤回 E6）、`mandatory-doors.md`（E1/E2/E5）、`key-flow.md`（E3/E4）、`shop-flags-and-economy-ceiling.md`（补「f3 实为 flags=2」「f37/f52 白给 1000 金」）。红用例：`docs_数字与CLI一致_smoke`（比对 `doors/keys` 输出与文档表）。预期：三份文档数字与 CLI 逐值一致。风险=低。回滚=revert。**只改文档**。

**S8（可选）landmark/`floor_dist` minima 修复**（F4 黄）：改 `solve_global.rs:758-800/:806-836` 取同层最小值。红用例：`f24双target_不同target不得互相覆盖期限`。预期：当前数据 0 变化。风险=低。回滚=revert。除非先出现「同层同 target 双梯」的层，**可暂不做**。

## 6. 数字对照表

| # | 指标 | 旧 | 新 | 依据 |
|---|---|---|---|---|
| 1 | 必经门总数 | 33 | **32** | S2；f14 黄 (8,11) 退出（§3-B） |
| 2 | 同色必要门数 | 文档 176/4/0=**180**；CLI 合计 176 | **172/4/0=176** | S2 不变值，仅文档更正（红 4 被双计） |
| 3 | 全部门 | 246/18/45=309 | 不变 | `doors` 输出 |
| 4 | 开价商店 | 文档「13 家/八家」 | **14 家** | `door12/door13` 高字节 flags&2（§3-A 表） |
| 5 | 红钥匙供给 | 6（全免费） | **5 Always + 1 Gated{f47,¤4000}** | S3；`/tmp/aud2/keyreach.py` |
| 6 | 黄钥匙供给 | 176（全免费） | **167 Always**（−9 在 f37）/ 176（开 gated） | S3 |
| 7 | 蓝钥匙供给 | 21 | **19 Always**（−2 在 f37） | S3 |
| 8 | f37 单层不可拾钥匙 | 0（未计入） | **12（黄9/红1/蓝2）** | §3-A |
| 9 | f14 那扇门 (8,11) | 必经 | **非必经**（侧袋，绕行） | §3-B |
| 10 | q1 `confidence` | `feasible` | `unknown`（`status=Capped`，`floors_expanded` 无 21–50） | S5 |
| 11 | q2 `confidence` | `feasible` | `unknown`（`Capped` + `truncated_layers=[[6,·],[7,·]]`） | S5 |
| 12 | q3 结论 | 「不拿终极装备不通关」 | **撤回**（只覆盖 f51–53，且 ban 改拓扑） | E7 |
| 13 | 「差 33 金」 | 瓶颈 | **撤回**（f8 免费红钥 + f37 白给 1000 金） | E6 |
| 14 | 模型层图缺口 | 0（未报告） | **2 条**：f29→f30（`ty72@(3,2)`）、f49→f50（无梯） | S4 |
| 15 | 「176 里黄多算 4」的依据 | — | **红 4 扇被同时计入黄列**：172+4=176，再 +4=180 ⇒ 与 CLI `172/4/0=176` 冲突 | §1 F7、E2 |

## 7. 假设区 + 不适用条件

**假设（C 级，未证实，不得据此改码）**：
1. `GIN_0_13` 的语义是「给 item13」，未给 17/18 —— 只读了 `level_scripts.txt` 的 op 字面，未跑脚本解释器。
2. f29 的 `ty83@(3,2) param32` 是「场景门/路线触发」，其脚本换层目标**未证实**（可能是 f30，也可能只改 flags）。
3. item21 上行符的**使用次数**与回溯合法性未证实 ⇒ 「f47 后回溯到 f37」目前是**理论可行**，不是已验证路线。
4. f54 的 `ty7@(7,1) param1` 与 f2 的 `ty8@(1,1) param1` 构成 `f54→f1` 环 —— 环的语义（回主线 or 重开）未证实，不影响本次数字。
5. 本文的 12 枚「不可拾」是按「怪可杀可穿、门算可过（钥匙充足）、ty11 算墙」口径算的；若采用「怪当墙」口径，数字会更大（150/4/18 不可达），但那与 doors 自述口径不符，故不采用。

**不适用条件**（结论在这些前提下失效）：
- 若未来引入「金勺子 item17 可由剧情/宝箱获得」，则 A 的结论（f37 正序不可拾）失效，`Gated` 退化为 `Always`。
- 若 `node_cap` 被环境变量 `node_cap_env()` 覆盖成 ≥10⁶ 且逐层预算充足，则 §6 行 10/11 的 `status` 可能从 `Capped` 变 `Exhausted`（但**不会**变 `Solved`，因为目标层不可达）。
- 若某层出现「同层同 target 双梯」，S8 不再是可选。
- 若用户裁定「f29/f49→f50 的脚本边不建模」（即接受模型更窄），则 `coverage.complete` 恒 false，任何「模型内无解」都必须读成 `unknown` —— 这是**允许的**，但必须在文档里写明，不许静默。

## 8. 验收清单

**门禁（每条都要贴输出）**
1. `python3 gates/cli.py check` —— 13 项全绿，**SKIP/TOLERATED 必须显式打印**（AGENTS §5）。
2. `python3 tools/state_baseline.py --check` —— 74 金样逐字节等价（`[GREEN] 金样逐字节等价：74 场景 sha 全同`）；S1 之后必须仍绿（证明引擎抽取不改行为）。
3. `cargo test -q -p game-solver --lib` + `cargo test -q -p game-core --lib` 全绿（**不跑** `cargo test --release` 全量）。

**变异自证（mutation）—— 每条修复必须能「反向破坏就红」**
4. S1：把 `cell_gate` 的 ty6 分支删掉 ⇒ `cell_gate_与引擎_interact_with_cell_同判_全54层逐格等价` 必红；把 ty7 改成 `through=true` ⇒ 同测试必红。
5. S2：把 `can_reach` 的 goal 从 `enter` 改成 `through` ⇒ `f14_喷泉格(6,9)是通道_黄门(8,11)不再必经` 必红。
6. S3：把供给改回「数实体」⇒ `f37_红钥匙_无玄明石时不可拾` 必红；把短供时改成仍然扣 demand ⇒ `短供层不得出现幻影余额` 必红。
7. S4：把 `missing_edges` 恒置空 ⇒ `f29_ty72_切口_上行梯不可达` 必红。
8. S5：把 `goals.is_empty()` 短路恢复 ⇒ `撞node_cap时不得输出feasible` 必红。

**数字核对（必须与 §6 表逐值一致）**
9. `cargo run --release -q -p game-solver -- doors` ⇒ 必经 **32**、同色必要 **172/4/0**、门 309。
10. `... -- keys` ⇒ 需求 29/4/0；供给 always 167/5/19（或 gated 176/6/21，取决于 `--supply`）；`no_departure_exit=[49,50]`、`entry_unreachable=[29]`。
11. `... -- queries` ⇒ q1/q2 `status` ∈ {`Capped`,`ModelBroken`}、`confidence="unknown"`、`coverage.missing_edges` 含 f29→f30 与 f49→f50。

**不许做的事**
12. 不许改 `data/tables/shop_goods.json` 的店价（`queries-after-f2.md:§3` 禁止项，本文与之对齐）。
13. 不许为了「让 q1 出 feasible」抬 `bound`/`node_cap`（§3-C 判据）。
14. 不许把 `ban` 编译成 `Cell::Blocked` 改拓扑（`solve_local.rs:1280-1283`）；禁装备应作用于**状态过滤**，不作用于地形。
15. 不许在没有 A/B 级来源的情况下补层间边（§4.5 判据）；缺来源就 `missing_edges` + `unknown`。
16. 不许删除或改写 `docs/findings/` 旧文；只允许 AGENTS §3.6 的**原地日期更正**。
17. 不许把 `exhausted` 读成「无解」；必须同时读 `coverage.complete`。
