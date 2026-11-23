# S5 外层全局搜索收敛性改造 —— findings

> 【2026-11-23】S5 任务产出。基点 fcbc97c，三笔 commit：机制 1 / 机制 2+真 witness / 遗留清理。
> 状态区分：**已证实**（有命令可复现）vs **假设**（待后续验证）。

## 1. 机制 1：支配剪枝加步数维（commit1）

**已证实**：
- `outer.rs` `dominated_seen(seen, state, steps)`：归档 tuple 第 10 位由 0 → steps，
  支配谓词加 `*sp <= steps`——同态不同步数的 Pareto 点不再互吞。
- TDD：`s5_dominance_explicit_steps_keeps_cheaper_state`（同态两代价夹具，旧逻辑只留一个=红）。
- 变异对照（3 发全红，已还原）：M1 删 steps 比较；M2 反向 `*sp > steps`；M3 删 `if !dominated` 门（无条件 push）。
- **回归零漂移**：combat-off seg1 700k pops 与 HEAD 基线 `cmp` **BYTE-IDENTICAL**；
  combat-on seg1 5M pops pre/post 均 `pops=4,553,792 truncated=false frontier=3630`，JSON BYTE-IDENTICAL。

**预期收益的如实结论（重要更正）**：任务书预期「pop 数显著下降」——**实测未下降**。
seg1（f51..f54,f1..f10）在加 steps 维前后 pop 数、前沿逐字节一致。解释（假设，B 级）：
该段的支配冲突本来就少（同态异代价几乎不出现），steps 维是**正确性补丁**（防同态异代价误剪）
而非性能补丁；在更长的段（全题 f51..f50）上收益未测（全题外层在锚不可达处收敛，见 §2）。

## 2. 机制 2：boss 锚段链分治 + 真 witness（commit2）

### 2.1 段式末任务语义（BossKill）

- `solve_local.rs`：`ExitKind::BossKill` + `Search.boss_target` + `GateArrive::BossKill`
  （bump 杀命中 boss_target ⇒ 记候选不入队，route 尾=bump token 可回放）；
  `solve_local_boss_toward_shops(...)`：prefer=BossKill，无候选⇒failed，不回退普通出层解。
- `outer_witness.rs`：`OuterWitnessCfg.boss_target`；boss_mode 末任务判据=BossKill|Ending
  （f50 段锚 bump ty72 本就走引擎 Ending）；`ending_done` 只在真 Ending 时 true。
- 分段定案（比任务书更正一处）：seg1=[51,52,53,54,1..10] 锚 75@f10；seg2=11..20 锚 74@f20；
  **seg3=21..35 锚 69@f35**（任务书原议 68@f25——68@f25 是段内小 boss，锚必须收尾保证层覆盖）；
  seg4=36..50 锚 72@f50(Ending)。

### 2.2 真数据链跑暴露并修复的四个真缺口

| # | 缺口 | 根因 | 修复 | 判据 |
|---|---|---|---|---|
| 1 | LB fatal 取消精确层解资格 | `fatal` 是外层 LB 口径（块内 Σmax counter_loss 下界，atk19 候选可压到 hp=-4335），不证明真路径必死。frontier 3630 条 killable 全 fatal ⇒ 旧过滤全拒 | 先取 killable&&!fatal，不足 K 再补 killable&&fatal | `s5_lb_fatal_candidate_still_attempted` |
| 2 | 外层 path 回放不支持穿已拾取块多跳 | path 只记首访块（pop 时才 push），相邻 token 间可穿已拾取块（真数据 b7→b5 实际 b7→b8→b11→b5，平行门需求不同） | `route_through_picked`：DFS+回溯，边必须 `apply_outer_edge` 当前可付（最短≠可行），中间块须已拾取 | `s5_path_replay_routes_through_picked_blocks`（先红 NotAdjacent 后绿） |
| 3 | driver 跨层 landing 传错层 | `landing()` 查**目标层**的梯（witness.rs replay 同款）；driver 传出发层 ⇒ f51（只有 ty7）找不到 ty8 ⇒ 假 landing_missing | driver 先取目标层 LevelSpec 再 landing | `s5_driver_landing_uses_target_floor_stairs`（变异=退回出发层⇒红） |
| 4 | 回放永远没有买臂 | `replay_route_expect` 建的 `Search::from_entry` shops=&[] | `replay_route_expect/replay_route_inner` 加 `shops` 参数 | `shop_buy_arm_replay_roundtrip`（吞买臂变异⇒no_match 红） |

### 2.3 真数据链跑结果（`run --outer-seg --max-pops 20000000`，每段上限 20M/10min）

| run | 结果 |
|---|---|
| run1 | 4 段全 blocked no_killable_candidate；seg1 pops=4,553,792 frontier=3630 |
| run5 | seg1 走通 11 层（f51..f54 + f1..f3 含 f1↔f2↔f3 回拾链），blocked 于 **f4 no_exit_toward（无指向 f5 的出层解）**；seg2/3/4 报 `skipped_no_handoff`（复核黄3 修正后口径；旧版报 no_killable_candidate 是「未跑搜索却给地图结论」） |

- 段基准 pops（机制 1 前后不变）：seg1=4,553,792 / seg2=14,177 / seg3=314,229 / seg4=105,254，
  total=4,987,452，全程 ~8.5min release。**四段 pops 总量远低于预算——外层不再是大档瓶颈**（已证实）。
- witness 关键数字（seg1 前缀 11 层，run5）：末层 hp=4027（**总损血按初始 300 口径为负增益，
  即净回复 -3727**——f52..f54 大量回血道具）；gold=1256；购买臂=0；回拾=3（f2×2、f3×2、f1×2 的重访）。
  keys 全程 0/0/0——见 §2.4 缺口。

### 2.2b 第五、六个真缺口（run6→run9 前缀回放逐层推进中暴露）

| # | 缺口 | 根因 | 处置 | 判据 |
|---|---|---|---|---|
| 5 | **撞杀移位语义分裂**：`Search::expand` Monster 臂撞杀后移入被杀格（1 步），而 `replay_route_inner`/`exact_replay_route`/exact DP 三处都不移位 ⇒ 真 witness route 回放必红（run6 f51 tok16 实锤） | DP 注释「不移动」与 expand 相反 | 全对齐 expand：两处回放加 `pos=(x,y)`、DP XC_MONSTER 臂 `nn.pos=np`；金样有意变更 `exact_dp_f3_reaches_4027` (88,14)→**(81,14)**（撞 7 只每撞省 1 步折返）。**A 级依据**：引擎 `scene.rs:3711-3713` 胜利后 `try_step(facing)` 自动补步、`scene.rs:3003` 相位 5 派发 `tick_battle(true)`——88→81 是**纠错不是放宽**。判据 `s5_bump_kill_moves_player_replay`（变异=删移位⇒「非邻接位移」红） |
| 6 | replay entry 重推 landing 传出发层（driver 缺口3 的回放侧漏网）+ Script 换层门（f1 6,9=门+83）在回放侧被判死 | 同缺口3；`gate_arrive` 返回 `Terminal{..}` 是正常出层语义 | replay landing 改目标层；回放 gate 容忍集 Pass|Terminal（Dropped 仍死） | run8 真数据红（visit4 f1 tok4）→ 修复后 run9 推进到 visit10 |

**run9（终态）**：seg-prefix-replay 推进到 **visit10 floor3 token80 no_match（无分支与 witness 出层态全等）**——f51/f52/f53/f54/f1/f2… 前十访全绿，f3 尾段仍有搜索/回放态分歧，未走通全程。

### 2.4 当前头号缺口（**已证实**，按任务纪律具名不抬 cap）

**外层块拾取积分 vs 内层实际拾取的分歧（outer/inner pickup divergence）**：
外层把「块被访问」记为该块 exports 全部入账（keys 进 OuterState 决定边可付性），
但内层 layer solve 只解「指向下一层的最短出层 route」——route 外的钥匙实体不会真被拾起。
真数据实测：seg1 前缀 11 层跑完后玩家 y/b/r=0/0/0，f4 从右侧入口 (11,10) 无钥匙打不开
`door 11,8` 等门 ⇒ 5 个候选全部 `no_exit_toward`（f4 toward5 用富钥匙玩家可解，
用 0 钥匙不可解——`dbg_landing` 探针复现）。
**修复方向（假设，未实施）**：中间层任务目标从「最短出层」改为「出层且拾取外层入账的 exports 集合」
（多目标层解，代价≈每层多一次求解），或外层边需求改为对内层可见的软约束。
在此修复前，seg1 无法产出全程 witness；任务纪律下不抬预算换有解，如实具名于此。

### 2.5 真 witness 首验（复核遗留 2，详见 §6）

`run --outer-seg` 已内置两路自证：
- 全链 witness：`replay_outer_witness(levels, (1,11), shops, &rep.floors)` ⇒ ChainReport.replay 字段；
- blocked 前缀：首个有解段的段内 floors 也做端到端回放（seg-prefix-replay 行）。
回放自证推进史：run6（f51 tok16 撞杀移位）→ run8（f1 tok4 Script 门）→ run9（visit10 f3 tok80 no_match，终态）；修复明细 §2.2b。

## 3. 遗留清理（并入 commit2）

- **黄1 商店双臂回溯判据**：根因是回放侧 shops 恒空（§2.2 缺口4）——修复后
  `shop_buy_arm_replay_roundtrip`：买臂 goal 的 route 可逐 token 回放；
  变异（回放侧吞买臂分支）⇒ no_match 红。**已证实**。
- **黄3 cost-model**（本文件 §2.3 表）：四段外层合计 4.99M pops / 8.5min，
  单段上限 20M pops/10min；后续跑档一律先 `outer`（seg1 单段）再 `run --outer-seg`，禁裸跑全题单跑大档。
- **黄4 测试计数**：本任务新增测试均以 TDD 先红记录；ratchet 如实（见 commit message）。
- 复核黄2（真 witness 未产出过）：§2.5 两路自证 + run5 前缀 witness 已产出（status=blocked，
  floors_solved=11，具名缺口 §2.4）——「真数据 witness」首次落盘：`docs/findings/outer-chain-real.json`。

## 5. 口径记录（lead 已核实，S6/复核后处置，本任务不下结论）

1. **seg2..4 的 `no_killable_candidate` 不是地图结论**：`outer_chain.rs` 的 `carried_player`
   只在 `seg_ok && chain_alive` 时更新——seg1 断链后 seg2 拿的是全新开局属性
   （hp300/atk10/def10），判锚不可杀必然成立。**已修（S5 复核 F3）**：断链后后续段不再跑搜索，
   以具名 `status=skipped_no_handoff` + `witness=null` + `note`「未跑搜索」上报；
   seg 级计数改名 `frontier_alive`（存活 frontier 数），`blocked.detail` 里改用「可杀候选」措辞——
   两者不再是同一个 `killable` 词。旧 run1 的 `4 段全 blocked no_killable_candidate` 属修复前口径。
2. **f1 (6,9) 搜索/回放语义不一致待裁决**：`solve_local.rs` 门臂（expand `Cell::Door` 臂）
   容忍 `gate_arrive` 非 Pass（`out.push((Act::Door, ns, g))`，非 Pass 由 goal/edge 层消化），
   回放侧原断言必须 Pass。这不是「回放太严」或「搜索太松」任何一边能单独断定的——
   本任务回放侧已放宽到 Pass|Terminal 使前缀回放可推进，**裁决权留待复核**。
3. **seg1 卡 f4 的根因 = outer/inner pickup divergence**（§2.4），与 P12
   「f3 88 步捡 14 件、f4 缺 2 黄钥」同一根因。

## 6. 真 witness 落盘（复核遗留 2 首验载体）

`docs/findings/outer-chain-real.json`（= `/tmp/s5_chain_real9.json`，ChainReport schema
`xiyou/outer-chain/v1`）：status=blocked，seg1 前缀 11 层 witness + 4 段具名报告 +
prefix-replay 失败定位（visit10 f3 tok80 no_match）。跑档纪律：先 `outer`（seg1 单段）
再 `run --outer-seg`，禁裸跑全题大档。

## 7. 段基准 pop 对比（机制 1 前后）

| 段 | pre | post | Δ |
|---|---|---|---|
| seg1(combat-on,5M) | 4,553,792 | 4,553,792 | 0（BYTE-IDENTICAL） |
| seg1(combat-off,700k) | 同 | 同 | BYTE-IDENTICAL |

结论：机制 1 无性能收益但有正确性收益（同态异代价不再误剪）；性能瓶颈已转移到
内层/外层拾取分歧与锚不可达（S4 黄3 的真实形态）。
