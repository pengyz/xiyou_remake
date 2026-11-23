# S4 findings：外层→内层贯通（inner–outer integration）

- 日期：2026-11-23
- 口径：**combat-cost on**（S4 验收口径，见 2026-11-22-s3-cross-floor-reentry.md；off 仅作 S2 回归逐字节对照）
- 提交：c7766aa（S4-A 外层→内层实化）、927a156（S4-B 真引擎回放）
- 产物：`solver run --outer --combat-cost --out run-witness.json`
  （status ∈ {witness, blocked, capped}；blocked 一律具名 `{floor, reason, detail}`）

## 1. 交付面

| 片 | 内容 | 判据（tests/game-solver outer_witness） |
|---|---|---|
| A | 外层 top-K 候选路径 → `path_to_tasks` 逐层任务队列（连续同层块归组、预算=组首块 `state_pre`、末组 toward=None）；驱动器逐访问 `solve_local_exit_toward_shops` 精解（`out_player/out_flags/ents` 接力，复访层用上次 `out_ents`） | `s4a_path_to_tasks` / `s4a_budget_divergence_named` / `s4a_no_silent_override` / `s4a_all_candidates_fail_is_named_blocked` |
| B | witness 前缀全程真引擎回放：`replay_route_expect`（5 种 route token DFS，商店格双臂）逐层逐 token，失败回 `ReplayFailure{visit,floor,token_index,token,stage,reason}` 帧号定位 | `s4b_replay_end_to_end` / `s4b_replay_failure_localizes`（破坏 route[2] ⇒ 红 + token_index==2） |

变异对照（每笔 ≥3 发，临时改源→判据红→还原）：
- A（随 c7766aa）：①预算改取组尾块 `state_pre` ⇒ `s4a_path_to_tasks` 红；②候选失败不记 attempts ⇒ `s4a_no_silent_override` 红；③drift 丢 hp 字段 ⇒ `s4a_budget_divergence_named` 红。
- B（随 927a156）：①吞掉回放 route 阶段 Err ⇒ `s4b_replay_failure_localizes` 红；②入场实体伪造空表 ⇒ `s4b_replay_end_to_end` 红；③`token_index` 恒 0 ⇒ `s4b_replay_failure_localizes` 红。

## 2. 全题实跑（combat-on）

外层模型补的两处缺口（commit c7766aa 内）：
1. f50 无 `boss_kill` 出口（块表仅 `script_gate` slot16@(6,5)；终局边是引擎语义
   `solve_local.rs` floor50 击败型72→Ending）⇒ `anchor_block` 加回落臂
   （本层无 boss_kill 出口时取 monsters 含 boss_ty 的块；combat-off 时 monsters
   空 ⇒ 仍具名 `NoAnchorBlock`，S2 口径不变）。
2. f24→50 stair 到场格：v1 ex.cell 代理 (6,1) 与 f50b0 cells 不相邻 ⇒ 具名 skip；
   按 `solve_global.rs:289 landing` 单一真相源特判 `to_floor==50 ⇒ (6,7)`。

### 预算耗尽实测（诚实记录）

- 5M pops（≈8min release）：`truncated=true frontier=0` ⇒ 具名 blocked
  `anchor_not_reached_within_pops`（pops=5000001, frontier_len=0）。
  对照：S3 seg1（f51..f10）4.55M pops 即收敛（前沿 3,630）；全题（51,52,53,54,1..50）
  状态空间显著更大，5M 远不够。
- 20M pops（35min 硬超时止损，`timeout 2100`）：**未能在 35min 内完成**（exit=124，
  无 JSON）——全题 pop 成本超线性，8min/5M 的 seg1 外推失效。
- 5M pops（现行二进制复跑，exit=0，/tmp/s4_outer_5m.json）：

| 项 | 值 |
|---|---|
| status | `blocked`（具名，非 feasible） |
| blocked | floor=50, reason=`anchor_not_reached_within_pops`, detail=`pops=5000001 truncated=true frontier_len=0（搜索预算内未触及锚层 f50）` |
| candidates_considered / attempts / floors | 0 / 0 / 0 |
| drifts / total_steps / ending_done | 0 / 0 / false |

**边界声明（最终档位数字待补）**：全题 witness 尚未产出——外层在 5M pops 内
未触及 f50 锚、20M pops 35min 未跑完；内层逐层数字（in/out hp、总损血、购买、
回拾）与 drift 表**因此暂无全题实跑样本**，合成判据面（§3）与 6 个判据测试
（§1）已绿。补跑命令：`cargo run --release -p game-solver --bin solver -- run
--outer --combat-cost --max-pops <N> --out run-witness.json`（N 需 ≥ 未测上界，
20M 档 >35min）。

## 3. 外层 LB 预测 vs 内层真值 drift 表

外层入边权 = 1 + 战斗 LB（可杀怪 max counter_loss）；它是**下界**：内层真损血
只会 ≥ 预测（hp 方向语义 inner ≤ outer）。其余字段（atk/def/gold/keys）外层
不建模磨属性购买（StatPct/AddKeys/Swap 不随 path 重放），方向不承诺，只如实
记两侧数值。漂移逐访问逐字段落 `OuterWitness.budget_drifts`
（`{candidate,visit,floor,block,field,outer,inner}`）。

- 合成判据面：fake 层 hp+7/gold+3 ⇒ drift `{visit:1,floor:52,field:hp,outer:300,inner:307}`
  与 gold 同记（`s4a_budget_divergence_named`）。
- 全题 drift 表：**暂无样本**（外层未达锚 ⇒ 内层未启动，见 §2.1 边界声明）。

## 4. 已知缺口（下一片清单）

1. **外层全局预算**：全题 pops 需求未知上界（≥80M 待证）。若 80M 仍不达锚，
   候选方向：支配剪枝加步数维 / 锚分层分治（seg 链）+ 全局 LB 融合 /
   frontier 采样求解（不求全局最优，出 witness 草案 + 最优性留档待证）。
2. **购买不随外层 path 重放**：`replay_outer_path` 不含 StatPct/AddKeys/Swap 臂，
   外层 atk/def/gold 预测系统性偏低（LB），drift 表只记不修。B 片真回放
   能兜住正确性（witness 路线按真实状态走），但「外层预测可信度」要靠 drift 表说话。
3. **Script/Cloud/UseItem 跨层落点**：这三类出层的 `cross_edge` 未进 witness JSON
   （serde 面只存 entry/exit），`replay_outer_witness` 对它们如实跳过落点回推
   （已知缺口，不静默充绿）。主线全程只有 Stair 换层，故不阻断全题 witness。
4. **复访层实体表**：驱动器允许复访层（外层语义），回放用上次 `ents_out` 接力；
   若未来出现「同层多次进、中途实体再生」语义（引擎有刷新怪？），需在
   solve_local 出层契约里显式建模。
5. **内层搜索界**：逐层精解撞界时如实 blocked（floor+reason），本片不抬 cap；
   全题跑若在某层 truncated，findings 记录该层与截断预算。

## 5. 回归与门禁

- cargo test -p game-solver：217（A 前）→ 219（B 后）判据全绿。
- gates check：PASS=13 FAIL=0 SKIP=0（两笔 commit 的 pre-commit 均全绿）。
- S2 口径 combat-off：`solver outer --max-pops 700000` 与
  `outer_seg1_baseline_pre_s3.json` **BYTE-IDENTICAL**（转移单源化零漂移）。
- ratchet：total=1259→1293，无回退（新增 outer_witness 判据计入基线）。

## 结论（两句话）

- witness 出没出来：**没出来**——外层全题搜索 5M pops 未达 f50 锚（具名
  `anchor_not_reached_within_pops`，不冒充无解），20M pops 35min 未跑完；
  机制面（A/B 两片、6 判据、真引擎回放）已全部落地并绿。
- 最大剩余缺口：外层全局搜索预算（pop 成本超线性，上界未知）与购买臂不入
  path 重放（LB 预测系统性偏低）。
