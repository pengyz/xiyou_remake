# game-solver 实现对抗性复核（红队）

日期：2026-11-17　复核对象：`crates/game-solver`（doors / keyflow / solve_local / solve_global / script_abs / bin）
纪律前提（AGENTS.md D5/D6）：**Rust 实现是唯一行为真相源**，Java 只作历史参照——本篇不把"与原版不同"当缺陷；
只判四类问题：结论会错、会骗用户、换随机地图会失效、Evidence 纪律不成立。
所有判定以代码为准（文档与代码冲突一律记漂移，见 §3）。

复核方式：只读代码 + 独立 python 复算（`/tmp/aud`，解析规则抄 `tools/route_gen.py:53-60`）+ 实跑现成 release 二进制
（`./target/release/solver`，2026-11-17 16:50 构建）+ `cargo test -q -p game-solver --lib`（19 passed）、`cargo check -q -p game-solver`（绿）。
**未使用"测试全绿"作为任何结论的依据**；判据全部按充分性论证。

---

## 1. 结论表

| 面 | 判定 | 一句话要点 | 证据 |
|---|---|---|---|
| F1 doors 必经判据 | **红** | 通行谓词与 `solve_local` 不同源，会**虚报必经门**；且忽略实体阻挡/怪/脚本，三条"入口"定义并存 | `doors.rs:115`（passable=`terrain‖goal‖entry`）vs `solve_local.rs:1230,1262`（`6\|7\|8` 楼梯豁免）；`doors.rs:68-94`（entry=首个 ty8）vs `solve_local.rs:667-673`（`LevelSpec.start`=传参） |
| F2 keyflow 供需 | **红** | 供给=全层 ty26/27/28 **实体计数**，不看可达性/层状态——模型内拿不到的钥匙照样算供给；断供层不扣 demand | `keyflow.rs:88-104`（无条件数拾取）、`keyflow.rs:110-117`（`else` 才扣 demand）、`keyflow.rs:60`（`no_exit_floors` 注释只写了"无出发型出口"） |
| F3 三查询语义 | **红** | `confidence` 在"无目标"分支**短路**，撞 cap/截断层的运行仍报 `feasible`；Q1/Q2 从未展开 21–50 层；文档把"红钥匙唯一来源"写成 f12 商店 | `solve_global.rs:526-533`；实测 `solver-queries.json` q1 `feasible`+`node_cap_hit:true`、q2 `feasible`+`truncated_layers:[[6,200000],[7,200000]]`；`docs/findings/2026-11-17-queries-after-f2.md:24` |
| F4 剪枝健全性 | **黄** | 支配维度本身**健全**（Key/GKey 已覆盖 atk/def/keys/bag/ents/flags/gold，绿）；landmark 的 `needed` 只对当前层用"最后一扇同目标梯"的表，同层同目标多梯时可能**高估止损门数 ⇒ 过剪**（现数据不触发） | `solve_local.rs:801-810,839,846`；`solve_global.rs:131,169,181`；`solve_global.rs:816-828`（`f==st.floor` 走 `table[f][t]` 而非 `minima`）；`solve_global.rs:795-796`（`table.insert(t, per_tile)` 同 target 覆盖） |
| F5 模型/引擎一致性 | **黄** | 赏金**单源**成立；但金落账有第二处实现、买臂无条件删实体（ignore `flags&4/&1`，方向=漏解）、买臂落位**不检查 walkable**（方向=假解，现数据 14 家全 walkable 未触发） | `solve_local.rs:976-979`→`combat.rs:217`；`apply_shop_effect:990,1002` 直写 `p.gold +=` vs `combat.rs:192 gain_gold_to`；`solve_local.rs:1625`（remove 无条件）、`:1626`（`ns.pos=(tx,ty)` 无 walkable 检查）vs 走臂 `:1595` |
| F6 未建模边清单 | **红** | 至少 4 条**未登记**近似：墙 ty11 永久阻挡（f29 因此是模型内硬死路）、keyflow 供给含不可拾取钥匙、`script_abs` 的 `find_path`/`reload_floor_entities` 空实现、Q3 把"禁令格"编译成 Blocked（改拓扑而非只改策略） | `solve_local.rs:1228,1262`；`item_effect.rs:37`（item17 可凿 ty11）；`scene.rs:3543-3556`；`script_abs.rs:351,439`；`solve_local.rs:1280-1283` |
| F7 文档/代码漂移 | **红** | 9 条（含 1 条硬事实错误、3 条计数差、2 条失效引用） | §3 表 |
| F8 证据纪律 | **黄** | 两处"实测"当前**不可复现**（`probe_c4_f14_red_key_reachability` 已不存在；`[G-DBG]` 插桩已删除）；一处证据指针错行；未发现"原版如此"作唯一理由 | `solve_global.rs:861-865`；`grep -rn "G-DBG" crates/` 空；`mandatory-doors.md:14`（引 `solve_local.rs:1183`）vs 实际 `:1262` |

---

## 2. 红/黄项复现命令与最小推理链

### F1-a 必经判据随通行语义漂移（33 → 32，可复现）

`doors.rs:115`：`passable = terrain_ok || goal_set.contains || (x,y)==entry`——**只豁免 goal(ty7) 与 entry(ty8)**。
`solve_local.rs:1230,1262`：`6 | 7 | 8 => stair_like`，且 `if !(walkable || stair_like) → Blocked`——**所有楼梯格都豁免地形**。
同一个 f14，两套语义给出不同的"必经"结论：

```bash
./target/release/solver doors --out /tmp/doors.json      # 合计：门 309 / 必经 33 / 同色必要 176 / 无效层 1(f49)
python3 - <<'EOF'   # 变体 A=doors.rs 原样；C=再豁免所有 ty6/7/8
import json
from collections import deque
F={int(k):v for k,v in json.load(open('/tmp/aud/floors.json')).items()}   # 我按 route_gen.py 规则复算的 54 层
def nec(f,stairs):
    v=F[f];ter=v['terrain'];ents=v['ents']
    doors=[(t,x,y) for(t,x,y,p) in ents if t in(1,2,3)]
    down=[(x,y) for(t,x,y,p) in ents if t==8];up=[(x,y) for(t,x,y,p) in ents if t==7]
    entry=down[0];goals=set(up)-{entry}
    sc={(x,y) for(t,x,y,p) in ents if stairs and t in(6,7,8)}
    def reach(bl):
        seen={entry};q=deque([entry])
        while q:
            c=q.popleft()
            for d in((0,1),(0,-1),(1,0),(-1,0)):
                n=(c[0]+d[0],c[1]+d[1])
                if n in seen or not(0<=n[0]<13 and 0<=n[1]<13):continue
                if any((bx,by)==n and (bt,bx,by) in bl for(bt,bx,by) in doors):continue
                if ter[n[1]][n[0]] or n in goals or n==entry or n in sc:seen.add(n);q.append(n)
        return seen
    return [d for d in doors if not(reach({d})&goals)]
print('A', sum(len(nec(f,False)) for f in F), 'C', sum(len(nec(f,True)) for f in F))   # A 33 / C 32
print('f14 A', nec(14,False), ' C', nec(14,True))   # A 多出 (1,8,11)，因 ty6@(6,9) 在 C 里可穿
EOF
```

结论：**f14 的 (8,11) 黄门是 doors.rs 语义下的虚报必经门**（f14 需求因此是 2 而非 1），
keyflow 的累计需求直接继承这个 2。方向：doors.rs 比 solve_local **更严** ⇒ 多报必经，不是少报。
（少报方向的证据见 F1-b：blockers 当墙时 f29 变 baseline_unreachable，而 doors.rs 报 ok。）

### F1-b 实体阻挡完全不在 doors.rs 里（f29）

`doors.rs` 只把"地形 + 门槽"当约束；`solve_local.rs:1228` 把 `9|10|11|12` 与 `:1251` 的 `32` 判永久 Blocked。
把 blockers 当墙后复算：**f29 从 ok 变 baseline_unreachable**（其余 53 层必经集不变）：

```bash
python3 -c "
# 同 §F1-a 的 reach()，block_cells 加上 ty in (9,10,11,12,32)
"   # 结果：只有 f29 变化（我实测：33 门不变，f29 由 ok 变 baseline_unreachable）
```

f29 结构（打印地形+实体）：

```
#############
###......####
###S.##..####      S=ty83 场景门, X=ty11 墙(8,3)
###.####X####
###.####.####
...
###S.##..####
###D.##.U####      D=ty8(3,11) 目标28, U=ty7(8,11) 目标30
#############
```

x=8 的竖廊 rows4-11 被 (8,3) 的 ty11 封死，**模型内 f29 没有上行边**（f29→f30 不可达），而
`doors.rs` 报 `ok`、`keyflow` 也不报任何警示。引擎侧 ty11 的语义是**可凿开**：
`scene.rs:3543-3556`（"只有金勺子、玄明石可以凿开。或者剧情打开！"，`allowed=false`）、
`item_effect.rs:37`（`(17, &[ItemAction::RemoveAdjacent{target:11,…}])`）。而求解器的 `UseItem` 边**只有**符 21/22 换层
（`solve_local.rs:1746,1990`）⇒ **模型内无法破墙**。这不是"少一条边"的记账问题，是"某些层在模型内必然死路"却未登记（见 F6）。

### F1-c 三条"入口"定义并存

1. `doors.rs:68-94`：entry = sprite 序第一个 ty8；
2. `solve_local.rs:667-673`：`LevelSpec.start` = `load_floor(..., start, ...)` 的入参，而 `bin/solver.rs` 对**全部 54 层**传
   `GLOBAL_ENTRY=(1,11)` ⇒ `Search::run()`（`solve_local.rs:1702-1708`）永远从题面入口起搜；
   实测 `./target/release/solver solve-local --floor 14` 的 S0 与 f14 无关（从 (1,11) 起），`reachable:false` 无诊断意义；
3. `solve_global.rs landing(:204)`：上一层出场梯的落点。

三者中 ②③ 用于搜索、① 用于判据；②的入参在 `solve-local` 路径上形同虚设（`from_entry` 的 `_entry` 也被忽略，
`solve_local.rs:1158`——只因 `run_root` 用 `pos: entry` 才无害）。

### F2-a 供给口径与求解器模型不一致（红钥匙 6 → 5）

`keyflow.rs:88-104` 无条件把该层所有 `ty26/27/28` 计为供给；`doors` 的 `status` 只进 `no_exit_floors`，不影响统计。
f37 的实体图（打印）：

```
#W.??W??.W?#     W=ty11 墙, R=红钥(7,5), y=黄钥, b=蓝钥
#.Wyy?WRyyW.#    R 的上下左右是 W/y/W/W ⇒ 被 ty11 围成口袋
#?.WWW.WWW.?#
```

`solve_local.rs:1228` 把这些 ty11 判 Blocked 且模型无破墙边 ⇒ **f37 的红钥匙、2 枚蓝钥匙、多枚黄钥匙在模型内不可拾取**。
复算：模型语义下 f37 共 12 枚钥匙不可达（含 `(7,5)` 红、`(4,9)(5,9)` 蓝）。同理 f3 `(4,1)(6,1)`、f7 `(9,10)(9,11)`、
f42 `(1,11)(2,11)` 的黄钥匙也不可达。
⇒ `./target/release/solver keys` 打的 `拾取供给 黄176 红6 蓝21` 是**实体计数**，不是"模型内可拾取数"；
红在模型内只有 5 枚。文档 `mandatory-doors.md:55` 的"供给=6 把（f8/f14/f34/f37/f42/f51 免费拾取）"没有标注这一点。

（本项**不改变**"不买钥匙不断供"的定性结论：红需求 4 ≤ 模型内供给 5，且 f37 在 f40 之前不是必需；
但它改变了"余量从不低于 0""按刚好够设计"这类定量叙事的可信度。）

### F2-b 断供层不扣 demand（幻影余额）

`keyflow.rs:110-117`：

```rust
if balance[i] < demand[i] { broke[i] = true; ... } else { balance[i] -= demand[i]; }
```

断供那一层**不扣 demand**，余额被留在高位；后续层因此更难报断供。现数据 Necessary 模式无断供 ⇒ 结论暂不受影响，
但这是"结论依赖一个不触发的 bug 才能成立"的典型（随机地图上会失效）。同时 `no_exit_floors` 把
`NoExit` 与 `BaselineUnreachable` 混在一个列表，而注释（`keyflow.rs:60`）与 CLI 文案只写了"无出发型出口（换层靠脚本）"——
一旦某层真的 `baseline_unreachable`，输出会撒谎。

### F3-a 置信级短路（红，且最便宜可修）

```rust
// solve_global.rs:526-533
let confidence = if goals.is_empty() { "feasible" }
    else if hit_bound || node_cap_hit || !truncated_layers.is_empty() { "bounded-best" }
    else { "optimal" };
```

`goals.is_empty()` 优先于"撞界"分支 ⇒ **被截断的运行永远报 `feasible`**，恰恰在最需要降级时降不了级。
实测两条独立证据（`docs/findings/solver-queries.json`）：

```bash
python3 -c "
import json;d=json.load(open('docs/findings/solver-queries.json'))
for q in ('q1','q2','q3'):
    g=d[q]['global'];print(q,'conf',g['confidence'],'cap',g['node_cap_hit'],'trunc',g['truncated_layers'],'exp',g['expansions'],'floors',[f for f,_ in g['gold_peak_by_floor']])
"
# q1 conf feasible cap True  trunc []                     exp 68148  floors 1..20,51..54
# q2 conf feasible cap True  trunc [[6,200000],[7,200000]] exp 8687   floors 1..10,51..54
# q3 conf feasible cap False trunc []                     exp 641    floors 51,52,53
```

`gold_peak_by_floor` 是**按"该层是否被展开"插入**的（`solve_global.rs:424-426` 的 `entry(...).or_insert`），
⇒ **Q1/Q2 从未展开任何 21–50 层的状态**。所以"界内无解"其实主要是"节点池在 f20 前灌满、21+ 层一次都没展开"，
不是"f12 差 33 金"（后者见 F7-#5）。而 CLI 打出的是 `Q1 max_hp=0(feasible)`（`query.rs:72-79`），
`query.rs:49-50` 又把 `0 + feasible` 注释成"表示 goals 空（**无解**）"——"没找到"被写成了"无解"，且标签字面义（feasible=可行）与内部义相反。

同一模式在局部求解器也出现，用户界面级证据：

```bash
./target/release/solver solve-local --floor 14
# {"floor":14,...,"reachable":false,"max_hp":null,"min_kills":null,"confidence":"feasible","route":[]}
```

`reachable:false` 与 `confidence:"feasible"` 同现。

### F3-b Q3 的 false 是"止于 f53"，不是"全局探索后不行"

Q3 只展开 51/52/53 三层、641 次扩展、`node_cap_hit=false`、`truncated_layers=[]` ⇒ 队列耗尽。
结合 `solve_local.rs:1280-1283`（`policy.ban` 命中的拾取格直接 `Cell::Blocked`），Q3 的"不拿终极装备"
被编译成"该格不可通行"——若 f53 通往 f54 的走廊上有 item36/40 的拾取格，Q3 就是在 f53 被一堵**人造墙**挡住，
与"终极装备值不值得拿"不是同一个问题。文档只写 `Q3 reachable=false`（表 1 第 3 行），没有说明它止于 f53 这一事实。

### F4 landmark 的 `needed` 只在"同层同目标多梯"时不可靠（潜在）

`build_landmark(:758-800)` 对每扇梯建 `per_tile` 并 `lk.table[f].insert(t, per_tile)`——**同层同 target 后者覆盖前者**；
`floor_dist_map(:806-836)` 对 `f == st.floor` 取 `table[f][t]`，对其它层取 `minima`（真 min，注释"乐观 ⇒ 不会误剪"成立）。
覆盖问题只影响**当前层**：此时 `needed` 可能取到别的梯的表 ⇒ 可高估 ⇒ `avail < needed` 误判 ⇒ **过剪**。
实测现数据**不触发**：f24 有两扇 ty7 但 target 分别是 50/25，其余 53 层每层各一扇 ⇒ 无同目标多梯。
故这是"随机/改图后失效"类，不是现行缺陷；但代码注释（`solve_global.rs:805`"乐观 ⇒ 低估 ⇒ 不会误剪"）与
这一分支的实际语义不符，应注明"当前层例外"。

---

## 3. 文档漂移清单（引用句 → 实际代码行为）

| # | 文档引用句 | 实际代码/数据 | 级别 |
|---|---|---|---|
| 1 | `mandatory-doors.md:25`「同色必要（门数） 176 \| 4 \| 0 \| 180」 | `doors --out` JSON 统计：**黄 172 / 红 4 / 蓝 0 = 176**（`summary.color_needed=176`）。黄多算 4、合计多算 4 | **硬错** |
| 2 | `queries-after-f2.md:24`「红钥匙**只在** f12 商店（`shop_goods.json` id10…price=800）」 | 数据有 **6 枚 ty27 地面拾取**：f8(2,2) f14(10,1) f34(8,5) f37(7,5) f42(7,1) f51(6,2)，且 f8 在 f12 **之前**；同日的 `mandatory-doors.md:55` 也写着"6 把免费拾取" | **硬错（同批文档互相矛盾）** |
| 3 | `mandatory-doors.md:14`「楼梯格豁免地形（`solve_local.rs:1183`…）」 | 豁免在 `solve_local.rs:1230`（`6\|7\|8 => stair_like`）+`:1262`（终审）；`:1183` 是 `walkable()` 前的空行/注释区，且 `doors.rs` **根本没有实现**这个豁免（见 F1-a） | 指针错 + 语义不符 |
| 4 | `shop-flags-and-economy-ceiling.md` §2「逐层商店 **13** 家」/§3「flags=3 **八家**」 | 扫描 30 个 cat16 格：开价店 **14 家** = flags2 三家(513/527/533) + flags3 **九**家(771/772/774/775/777/778/779/780/781) + flags6 两家(1541/1544)；与 2026-11-15 文档"11 家 flags=3\|6"（9+2）自洽，与本篇"八家"冲突 | 计数错 |
| 5 | `queries-after-f2.md:26-30`「真正的瓶颈：f12 金峰值 767 < 800…这 33 金必须来自 f1..f12 的额外击杀」 | 同一份 `solver-queries.json` 里 Q1 的金峰值 f18=**1861**、f14=925，且 21–50 层**从未展开**（F3-a）；"33 金"不是可观测的拦截点。红钥匙另有 5 枚可拾取。归因缺乏支撑 | 结论级漂移（**未能证实**） |
| 6 | `solve_global.rs:861-865` 注释「实测 f14 的 ty27@(10,1) 连 9+9+9 全钥匙都拿不到（`probe_c4_f14_red_key_reachability`）…计进来会让 avail 虚高」 | (a) 该 probe 在代码里已不存在（`grep -rn probe_c4_f14 crates/` 空）；(b) 实际代码 `:902-912` 对**任何** lock≠INF 的钥匙格都 `avail += 1`，f14 那枚会被计入——注释描述的"排除"并未实现；(c) 独立拓扑复算（doors 语义）给出 f14 (10,1) 的**具体 20 步路径**（经 (3,9)黄门、(6,6)黄门、(10,4)蓝门），只需 1 黄+1 蓝 | 失效"实测" + 注释与代码相反 |
| 7 | `2026-11-15-solver-queries.md:73` 引 `[G-DBG] red>0 pushes by floor` 作为证据 | `grep -rn "G-DBG" crates/game-solver/src/` 为空 ⇒ 该插桩已删除，表格无法复现 | 失效证据 |
| 8 | `solve_global.rs:668` 注释「0黄1蓝2红」 | `:684` 实际 `ty1→0, ty2→1, ty3→2`（黄红蓝），`landmark_dead:860` 顺序 `[yellow,red,blue]` 也印证黄红蓝；只有注释错 | 注释漂移 |
| 9 | `mandatory-doors.md:13`「怪不是固定墙」+ `:71-72`「需与 C4-a blocking_kill_slots 交叉核对」 | 交叉核对**未做**（仓库内无该结果）；`doors.rs` 完全不看 cat8 | 自认缺口未闭环 |

（口径一致性抽验：我的独立复算给出 门 **309**（黄246/红18/蓝45）、必经 **33**、钥匙 **176/6/21**，与文档逐值吻合 ⇒
`doors`/`keys` 的主数字不是编的；问题集中在"这些数字被赋予了什么语义"。）

---

## 4. 两句总结

**① 当前求解器实现是否合理：有前提可用（判据层可用，结论层不可直接引用）。**
`doors`/`keys`/三查询的**机制单源**（门色/钥匙码/赏金/开门）确实做到了，支配维度也健全；但
"必经 33 门""红钥匙 6 供给""全程不断供""界内无解""Q3 不拿终极装备过不去"这五句话，各自都能被本报告里的
一条代码事实或数据事实推翻/削弱——它们现在只能读作"在某个未写明的通行与供给口径下的模型内观测"，不能读作游戏事实。

**② 最该先修的 1–3 项（按收益排序）**

1. **置信级短路（`solve_global.rs:526-533`）**——一行改动，直接决定所有对外输出是"最优/截断/没找到"。
   现状是撞 cap 或层截断也报 `feasible`，且 CLI 把它印成 `Q1 max_hp=0(feasible)`：这是最容易被用户当成"可行"的一句话。
   顺带修 `query.rs:49-50` 的"0 ⇒ 无解"注释为"0 ⇒ 未找到（可能被截断）"。
2. **把 doors/keyflow 的通行与供给口径单源化到 `solve_local` 的 `classify`**——统一"楼梯豁免（含 ty6）"和
   "blockers 是否算墙"，并让 keyflow 只统计**模型内可拾取**的钥匙（或输出两列：实体数/可拾取数）。
   这一项同时消掉 F1-a 的虚报（f14 需求 2→1）、F2-a 的供给虚高（红 6→5）、F1-b 的 f29 静默死路。
3. **补/禁"破墙边"并重写红钥匙归因**——要么给 `UseItem` 加 item16/17 的 `RemoveAdjacent`（这是引擎已有的确定性语义，
   `item_effect.rs:36-37`），要么在 findings 里显式登记"模型内 f29 不可上行、f37 钥匙不可拾取、红钥匙实际只有 5 枚"，
   并把"瓶颈=f12 金差 33"改成"未定位（Q1/Q2 均未展开 21–50 层）"。

---

## 5. Confidence

**78 / 100**

- 高置信（90+）：必经判据的语义漂移（F1-a，两套谓词逐行可查、两套口径的复算都跑出了可对比的数字）；置信级短路（F3-a，代码 + JSON 双证）；
  供给口径（F2-a，代码逐行 + f37 实体图）；漂移表 #1/#2/#4/#6/#8（可 `grep`/`jq` 复现）。
- 中置信（60–75）：F5 买臂两条差异的影响面（数据未被触发，方向已定但"会不会真出错"要随机/改图才验证）；
  F6 的 `script_abs` 空实现影响（未逐脚本核对哪些脚本会走 `find_path`/`reload_floor_entities`）。
- 低置信/未能证实："33 金是瓶颈"的**证伪**——我只能证明它缺证据（21–50 层未展开、红钥匙另有 5 处），
  不能证明"若展开 21–50 层就一定有解"；f14 红钥匙在**战斗可行性**（怪能不能杀、血够不够）上是否可达，无插桩无法判定。
- 未覆盖：`crates/game-core/src/scene_script.rs` 与 `script_exec.rs` 的逐 op 语义未逐条精读（`script_abs` 是"驱动真引擎对拍"，
  重点已覆盖在 F6）；4 枚红钥匙之外的掉率/事件供给未核（只按实体与商店两源统计）。
