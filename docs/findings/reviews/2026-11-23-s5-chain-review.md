# S5 机制2 boss 锚段链分治 · 独立对抗复核（906df56）

**一句话结论：有前提可推。** 六处「回放缺口」与「撞杀移位 1 步」经我独立复算成立（引擎级 A 证），seg1 的 11 层前缀 witness 是真实产物；但「真 witness 首验 / 自证」不成立——全链 replay 从未运行（`replay=null`），前缀 replay 在 visit10 失败，且 findings §6 对 JSON 内容的描述与产物不符。

**红 2 / 黄 3 / 绿 3**（F1..F8）。**Confidence: 80。**

复核方式：只读复核，仓库零改动（唯一写入本文件）。cargo 全部前台串行；未跑 13min 真数据链，改用 `/tmp` 的 run5/6/8/9 日志与已落盘 JSON（`cmp` 与 run9 逐字节相同）。6 组变异实验在 `/tmp/s5mut`（rsync 副本 + `assets/`）内做、逐条复原。`python3 gates/cli.py check` 跑一次：PASS=13 FAIL=0 SKIP=0；`python3 tools/state_baseline.py --check`：74 场景 sha 全同。

---

## F1 撞杀移位语义 —— **绿**

判定：实现者结论正确。一次方向输入撞怪后玩家**最终落在被杀格**，按 route token 计为 **1 步**；金样 (88,14)→(81,14) 是**纠错不是放宽**。

证据（A，我亲自跑的引擎测试 + 代码）：
- 撞怪本身不移位：`crates/game-core/src/scene.rs:3197 try_step` 只做 `interact_with_cell` 并置 `walk_phase=1`；Monster 臂在 `scene.rs:3480` 才 `self.walk_phase = 5`（进战斗），玩家坐标未动。
- 胜利后引擎**自动补一步**：`scene.rs:3712-3716`——`battle_anim_tick>5` 且 `!self.battle_rerun_scene && allow_restep` ⇒ `self.try_step(self.facing)`（实为 3715 行；3712-3714 是 remove/自增）；`allow_restep` 来自相位 5 派发 `scene.rs:3003 5 => self.tick_battle(true)`。
- 引擎**自己的位移记账就是一步**：`cargo test --release -p game-core --lib stage_b008_interrupt_mid_victory_restep` → 1 passed；该测试 `crates/game-core/src/scene_tests/t2_walkto_route_tests.rs:919-940` 断言 restep 后玩家在 `(6,ROW)`（被杀格）且 `undo_seq == [tile(5,ROW), "revive"]` 只有一笔位移。旁证 `t2_walkto_route_tests.rs:2228`、`crates/game-core/tests/mov_route_scene.rs:460`。
- 求解侧对齐：`Search::expand` Monster 臂进格 1 步；`crates/game-solver/src/exact_pickup.rs:764-775` `XC_MONSTER` 现为 `nn.pos = np; nn.steps = n.steps + 1;`（旧注释「不移动」已删）。
- 88 步是错的、81 是**唯一可回放**值：变异 M-B（删 `nn.pos = np;`）→ `cargo test --release -p game-solver --test exact_pickup exact_dp_f3_reaches_4027` 红：`crates/game-solver/tests/exact_pickup.rs:328` `f3 必须由精确 DP 接管，实测 reason="replay_failed"`。88−81=7=撞怪 7 只（`kills==7`、`doors==5` 断言仍绿），数目自洽。

残留（黄级子项，B 级静态阅读，不改本判定；但实现者「全对齐引擎」的措辞过宽）：
- ty75 重跑剧情战斗时引擎**不 restep**（`scene.rs:3715` 的 `!battle_rerun_scene` 条件），求解侧仍无条件进格；seg1 锚正是 ty75@f10。
- restep 要求被杀格可进入；求解侧不检查落点（若被挡，引擎停在原地而求解器已进格）。

最小修法：把「撞杀必进格」注释改为分情况（普通战 restep / ty75 重跑不 restep），witness 侧对 ty75 的 bump token 去掉位移期待。

---

## F2 f1(6,9) 语义裁决 + 放宽边界 —— **黄**

(a) 语义判定：`GateArrive::Terminal` 是**正常出层**，不是失败。`crates/game-solver/src/solve_local.rs:1884-1905` 枚举注释 + `3284 gate_arrive`：`tickdrive`（`:3850`）读到 `persist.pending_floor != 0` 时返回 `Terminal{pending,went_down}` ⇒ 门命中且脚本换层；`Dropped` 才是失败。因此把 Terminal 当出层接受**不冤**。

(b) 但「Terminal 只允许是该层 route 最后一个 token」这条约束**没有任何代码或测试强制**：
- 搜索产出的 route 天然满足（`solve_local.rs:2948` 附近 `GateArrive::Terminal{..}` 记 goal 后 `route.push("gate gx,gy")` 且**不入队**）；
- 回放侧 `solve_local.rs:4727 / 4765 / 4779 / 4792` 四处是**无条件**接受 `Pass | Terminal{..}`，而 `gate x,y` token 本身是 no-op（`:4741-4743`），回放**不做换层** ⇒ mid-route 的 Terminal 会被静默续走。
- 真数据里约束**恰好成立**（我扫了 JSON）：11 层 witness route 中仅一条含 `gate `，即 visit4 floor1，`route=['6,11','6,10','5,10','6,10','door 6,9','gate 6,9']`，gate 在 idx5=末位（len=6）。所以本次放宽对这条 route **合法**，但只是碰巧。

(c) 变异验证：M-A 把 4 处恢复成 `GateArrive::Pass` → `outer_witness 13 / solve_local 26 / solve_local_trunc 5` **全绿** ⇒ 放宽**零单测覆盖**，只有真数据能触发（run8 日志 A 级：`visit=4 floor=1 token_index=4 token="door 6,9" reason=开门后 gate_arrive 非 Pass`）。

另发现一处**未在 md / commit 交代**的放宽：`solve_local.rs:4745-4750` 新增「`(x,y)==cur.pos ⇒ i+=1; continue`」的零位移容忍。注释只说「前导入场格首 token」，**实际接受任意重复位置 token**。它承载了「route 首 token=入场格」约定（11 层里 9 层 `route[0]==entry`）；M-G 变异（删该块）→ `s5_bump_kill_moves_player_replay` 在 `token_index:0, token:"0,1"` 红，说明该约定确实在用。

最小修法：4 处接受前加断言「Terminal ⇒ `i == route.len()-1`」（或接受后立即 `break` 并要求无后续 token）；零位移容忍收紧为 `i==0`；补一条「mid-route Terminal 必须红」的单测。

---

## F3 seg2..4 口径 —— **黄**

解释**成立**（A，代码+产物）：`crates/game-solver/src/outer_chain.rs:249 let mut carried_player = player0.clone();`，`:365` 仅在 `if seg_ok && chain_alive` 分支内更新，`:292` 用它构造 `start_state` ⇒ seg1 断链后 seg2 起拿到全新 hp300/atk10/def10。产物同样如此：seg2/3/4 的 `witness.floors=0 / attempts=0 / total_steps=0 / candidates_considered=0`，`blocked.floor` 直接落在锚层（20/35/50），从未进入 witness 驱动。所以 `no_killable_candidate` 是**属性口径**结论而非地图结论——md:97-100 的说法属实。

报告结构会误导（黄的原因）：
- `skipped_no_handoff` 在 `outer_chain.rs` **不存在任何字符串**；seg2..4 的 `status="blocked"` + `blocked.reason="no_killable_candidate"` 与 seg1 的 `no_exit_toward` 外观同权，只有 `handoff_ok:null` / `floors_solved:0` 能间接看出是断链。
- 同一份 JSON 里 `killable` 一词两个定义：seg 级 `killable` = `frontier.filter(|c| !c.fatal).count()`（`outer_chain.rs:310`）⇒ seg2=4 / seg3=215；而 `blocked.detail` 的 `killable=0` 是 `outer_witness.rs:240-275` 的**敏感表**口径，且串里的 `0` 是**硬编码**（`format!("frontier_len={} killable=0…")`）。读者只能看到「killable=4」与「killable=0」并列。

最小修法：断链段输出 `status="skipped_no_handoff"`（或显式 `handoff:None` 布尔）；seg 级字段改名 `non_fatal_frontier`，detail 打印真实敏感表计数。

---

## F4 「真 witness 首验 / 自证」 —— **红**

两件事**不矛盾，但被混写成了自证通过**：
- 落盘是真的（A）：`docs/findings/outer-chain-real.json` 存在，`cmp` 与 `/tmp/s5_chain_real9.json` **逐字节相同**；seg1 前缀 11 层 witness（末层 out `hp=4027 / gold=1256 / atk=731`）。
- 自证**没通过**（A）：JSON 顶层 `replay=null`、`floors=[]`、`total_steps=0`（`outer_chain.rs` 仅在 `floors` 非空时回放，seg1 `status!=witness` ⇒ 全链 replay 永不触发）；前缀回放只到 visit10 就红（run9 日志：`token_index=80 token="11,11" reason=no_match`）。
- 更硬的问题：md:110-113 **声称**该 JSON 内含「prefix-replay 失败定位（visit10 f3 tok80 no_match）」。实测 JSON 中 `"no_match"` 出现 **0** 次、`"prefix"` **0** 次、`"token_index"` **0** 次、`replay=None`——定位只存在于 stdout（`bin/solver.rs` 打印，不落盘）。这是**产物与描述不符**。

必须降级到的措辞：**「seg1 前缀 11 层 witness 已落盘；前缀回放十访绿、第 11 访 f3 tok80 红；全链 replay 未运行」**。凡「两路自证」「首验」这类可被读成已自证的表述都应删除。

最小修法：`ChainReport` 增 `prefix_replay:{status,visit,floor,token_index,token,reason}` 并落盘；md §6 改写为上述一句，删掉「prefix-replay 失败定位」这句无载体的描述。

---

## F5 六处「回放真缺口」逐条 —— **绿**

不看自述，逐条自造反例（变异均在 `/tmp/s5mut`，仓库零改动）：

| # | 缺口 | 我的变异 | 结果 | 真伪 |
|---|---|---|---|---|
| 1 | LB fatal 候选补位 | `if cands.len() < cfg.candidates {`→`if false {` | `tests/outer_witness.rs:824` 红：`fatal(=LB 口径) 候选必须被尝试，got blocked=Some("no_killable_candidate") attempts=[]` | 真缺口；但真数据侧 seg1 `killable=2645 ≥ K=5` 补位不触发、seg2 触发后仍空 ⇒ 对 run9 结果无影响（潜伏修复） |
| 2 | 外层 path 穿已拾取块多跳 | `route_through_picked` 首行 `return Some(vec![state.node, to]);` | `outer.rs:1732` 红：`跨已拾取块的 path 回放必须成功: Some(NotAdjacent{from:"f10:b0",to:"f51:a1"})` | 真缺口（单测证明；全链 replay 未跑 ⇒ 无真数据证据） |
| 3 | driver 跨层 landing 传错层 | `landing(tlevel_t,..)`→`landing(tlevel,..)`（出发层） | `tests/outer_witness.rs:864` 红：`blocked=Some(("landing_missing","f52 无落点梯")) left:1 right:2` | 真缺口 |
| 4 | 回放侧 shops 恒空吞买臂 | `replay_route_expect` 传 `&[]` 代替 `shops` | `tests/solve_local_trunc.rs:439` 红：`Err(RouteReplayError{token_index:7,token:"7,1",reason:"no_match"})` | 真缺口（真数据购买臂=0 ⇒ 无真数据证据） |
| 5 | 撞杀移位语义分裂 | 删 `nn.pos = np;` | `tests/exact_pickup.rs:328` 红 `reason="replay_failed"`（同 F1） | 真缺口（引擎 A 级旁证） |
| 6 | replay entry landing 传出发层 + Script 换层门判死 | 已由 run8 真数据红证明（非我变异） | run8 日志：`visit=4 floor=1 token_index=4 token="door 6,9" reason=开门后 gate_arrive 非 Pass`；放宽后 run9 推进到 visit10 | 真缺口，但「谁对谁错」未裁决（md:101-104 自己写「留待复核」，如实） |

结论：**6/6 都是真缺口**，不是「为让回放通过而改语义」。#1/#2/#4 仅有单测/合成证据，真数据链路未触达；#6 的语义方向仍开放。

---

## F6 有没有静默放宽 —— **黄**

- 被删/改弱的断言（全 diff 逐行核）只有两处：4 处 `gate_arrive` 的 `Pass`→`Pass|Terminal{..}`（md/commit 文档化了）+ DP 金样 `(88,14)`→`(81,14)`（文档化了，F1 证明是纠错）。`tests/outer_witness.rs` 的 `-` 行全是 `OuterWitnessCfg` / `replay_outer_witness` 签名带新人参，**没有删断言**；`tests/solve_local.rs` / `tests/solve_local_trunc.rs` 的 `-` 行全是 `solve_layer_with(..., None, ...)` 加 `boss_target` 参数，断言原样保留。
- `#[ignore]`：`grep -rn "#\[ignore" crates/game-solver` 仍为 2 条，diff 无新增/删除 ⇒ 没有把测试藏进 ignore 骗 ratchet。
- ratchet：gates `test-ratchet total=1304(基线1293) 无回退`，与 commit message 一致（+11 为新增测试）。
- 74 金样：`tools/state_baseline.py --check` → `[GREEN] 金样逐字节等价：74 场景 sha 全同`；diff 未碰 `data/status/baselines/`。
- **唯一静默的是** `solve_local.rs:4745-4750` 的零位移容忍：md 内 grep「零位移|cur.pos|入场格」**零命中**，commit message 也没提，只在代码注释写「先容忍该注记 token」；而它的接受面是「任意重复位置 token」。
- 覆盖缺口：M-A 证明 `Pass|Terminal` 放宽**零单测覆盖**（见 F2）；未来若搜索侧产出 mid-route Terminal 或 route 生成器变化，回放会静默吞错。

最小修法：把零位移容忍写进 commit/findings 并把条件收紧到 `i==0`；补「mid-route Terminal 必须红」的单测。

---

## F7 数字自洽 —— **红**

对得上（A）：四段 pops `4,553,792+14,177+314,229+105,254=4,987,452`（md:50-51 ↔ JSON 顶层 `total_pops=4987452` ↔ run9 日志，三处一致）；`floors_solved=11`（JSON+日志）；md:52 的 hp4027/gold1256/atk731 与 run5、run9 JSON 的 `out_player` 逐值相同；visit10 f3 tok80 的标红位置与日志一致；log 与 JSON 同源（`cmp` 相同）。

对不上（A，均为「md vs 产物」）：
1. **md:54「keys 全程 0/0/0」不成立**——末层 f3 的 `keys_out=[0,1,0]`、`key_shortfall.have=[0,1,0]`（run5 与 run9 皆然）。终态带 1 把蓝钥。
2. **md:53「回拾=3（f2×2、f3×2、f1×2）」不成立**——11 层序列 `[51,52,53,54,1,2,3,2,1,2,3]` 的重访应为 **4**（f2 多 2、f1 多 1、f3 多 1）；且「f1×2」与 f1 只出现 2 次（多 1 次）自相矛盾。run5 JSON 同为 4，不是 run5/run9 口径差。
3. **md:110-113 声称 JSON 内含 prefix-replay 定位**——实测 JSON 无任何 `no_match/prefix/token_index` 字段，`replay=null`（同 F4）。
4. 同一名词两定义：seg2/3 的 `killable=4/215` 与 `blocked.detail` 的「killable=0」并列（同 F3）。
5. 引用漂移（不影响结论）：md:60 引 `scene.rs:3711-3713`，实际自动补步在 `scene.rs:3715`。

最小修法：md:54 改为「末层 keys=0/1/0（1 蓝）」；md:53 改为「回拾=4（f2×2、f1×1、f3×1）」；md:110-113 删除「JSON 内含定位」（或在 F4 的 `prefix_replay` 落盘后再写）。

---

## F8 夹带与红线 —— **绿**

- 只动 `crates/game-solver/**` 与 `docs/findings/**`：`git show 906df56 --name-only` 无 `original/`、`reference/`、`data/naming/`、`gates/`、`data/status/baselines/` 任何条目。
- 74 金样未改、sha 全同（F6）；总闸 `PASS=13 FAIL=0 SKIP=0`（我跑一次，与 commit message 数字一致）。
- 措辞：「首验/真实/已证实」的问题集中在 F4/F7 两处（md:92-93 / :54 / :110-113）；**commit message 本身对「witness 没出来」是如实的**（明写 seg1 卡 f4、seg2..4 无 handoff、未走通全程），没有把未验证写成已证实。md 里「§2.4 已证实」「黄1 已证实」属代码/单测级结论，可接受。
- 最小修法：随 F4/F7 一并改写 md 那两处。

---

## 汇总表

| 面 | 判定 | 一句话 |
|---|---|---|
| F1 撞杀 1 步 | 绿 | 引擎 A 证：撞不移位、胜利后自动补 1 步、undo 仅一笔；88→81 是纠错 |
| F2 Terminal 放宽 | 黄 | 语义正确但「末 token」约束无强制、4 处无条件接受、零单测覆盖；另加未交代的零位移容忍 |
| F3 seg2..4 口径 | 黄 | 解释属实；`skipped_no_handoff` 缺位 + `killable` 两口径同词 |
| F4 真 witness 自证 | 红 | 落盘真、自证未过；md 声称 JSON 内含回放定位，实测无此字段 |
| F5 六处缺口 | 绿 | 6/6 真缺口，变异逐条红；#1/#2/#4 无真数据证据 |
| F6 静默放宽 | 黄 | 无删断言、无 ignore、金样全同；零位移容忍未交代且接受面过宽 |
| F7 数字自洽 | 红 | keys 0/0/0 与回拾=3 与产物冲突；JSON 无 md 声称的定位字段 |
| F8 夹带红线 | 绿 | 未碰 original/reference/naming/gates/金样 |

**能作什么用**：可作为「机制2 框架 + 六处回放缺口修复 + seg1 11 层前缀 witness」的可用基线——该 commit 上 `exact_pickup / outer_witness / solve_local*` 单测与 gates 13 项确实绿，撞杀 1 步语义有引擎级 A 证，金样变更与回放口径已一致。

**不能作什么用**：不能当作「链已收敛」或「真 witness 已自证」的证据——全链 replay 从未运行（`replay=null`）、前缀 replay 在 visit10 失败、seg2..4 的 blocked 是属性口径假象；此后任何「首验 / 自证通过」的表述都缺载体。
