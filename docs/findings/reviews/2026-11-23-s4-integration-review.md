# S4 独立对抗性复核：外层→内层贯通（c7766aa / 927a156 / ce6f979）

- 日期：2026-11-23
- 复核者：独立对抗性复核 agent（不参与实现）
- 对象：父 d4f1da0 之上三笔本地 commit；被告自述 `docs/findings/2026-11-23-s4-inner-outer-integration.md`
- 方法：原仓回归复跑 + /tmp 副本独立变异与探针实验（变异不在原仓）
- 纪律：本文为唯一产出文件；未 commit、未改任何代码

## 计数：绿 5 / 黄 4 / 红 0

| # | 面 | 判定 |
|---|---|---|
| F1 | 回放真实性 | **绿**（M1/M2 变异红，双判据互补有牙；商店双臂见 F1-Y） |
| F2 | blocked 判定诚实性 | **绿**（探针证实纯预算/爆炸问题，非结构性过滤 bug；量化缺口见 F2-N） |
| F3 | 单源转移行为等价 | **绿**（combat-off 700k BYTE-IDENTICAL 亲自复跑） |
| F4 | budget_drifts 七字段 | **绿**（合成判据红验 + 「零样本」如实标注） |
| F5 | 判据与变异重放 | **绿**（M1/M2/M3 亲验红 + cargo test 全绿 + ratchet 1293 无回退） |
| F6 | 门禁/夹带 | **绿**（gates PASS=13 FAIL=0 复跑；三笔 --stat 无越界；witness.rs 放宽纯可见性） |
| F7 | findings 诚实性 | **绿**（blocked/超时如实记录、可操作） |
| F1-Y | 商店双臂回溯的判据保护 | **黄**（M4 变异：吞买臂 ⇒ 全部判据仍绿） |
| F1-N2 | 端到端回放仅合成夹具 | **黄**（真实 solve 产出的 witness 从未被回放过；且 `run_outer_cli` 不调 `replay_outer_witness`） |
| F2-N | 预算外推严重低估 | **黄**（见 F2 证据） |
| F5-N | cargo test 计数 | **黄**（复核只确认 0 failed / exit 0，未逐套数齐 219；ratchet 1293 由门禁独立确认） |

## F1 回放真实性（头号面）——绿

**代码核实**：`replay_route_inner`（solve_local.rs:4551 起）逐 token 驱动与 `expand` 同一条转移实现（`pickup_step`/`gate_arrive`/`gate_bump`/`consume_key_for_door`/`exact_bump`+`apply_kill_drops`/`shop_offer`+`apply_shop_effect`），非路径自洽循环；商店格双臂（走臂/买臂）DFS 压栈回溯（solve_local.rs:4655-4690），`replay_route_expect` 以终态全等定臂、零匹配/多匹配均具名。

**变异独立验证（/tmp/s4rev 副本，均已还原并回绿）**：
- **M1 伪造回放**（`replay_route_inner` 收到 expect 时直接返回期望终态、不执行任何 token）⇒ `s4b_replay_failure_localizes` **红**（实测 5 passed 1 failed）。判据有牙：能抓住「假回放」。
- **M2 入场实体伪造空表**（`outer_witness.rs` `None => Vec::new()`）⇒ `s4b_replay_end_to_end` **红**。实现者自述的该变异复跑成立。
- 附带观察：M1 下 `s4b_replay_end_to_end` 单独仍绿（它只断言成功）——**两个判据必须成对存在才有牙**，现设计成立。
- 商店双臂回溯：代码面人工核对实现正确，但见 F1-Y。

## F2 blocked 判定的诚实性——绿（表述准确），附黄 F2-N

**「frontier_len=0 是否过滤 bug」**：核实 `outer_search`（outer.rs:975-1140）——frontier 只在锚节点被 pop 时按 `anchor_killable` 判定入列，**不存在任何把候选提前丢弃的过滤器**；frontier_len=0 ⟺ 锚节点在预算内从未被 pop。措辞「搜索预算内未触及锚层」准确。

**小实验（/tmp 副本 + pop 进度探针，全题图 51,52,53,54,1..50 combat-on）**：
- 锚改 f20/ty74（boss_kill 真实锚 f20b4，块表定向查询确认），1M pops：锚节点 pops=0，被 pop 的状态只覆盖 13 个层，max_floor=54，尾部分布 `f2=265352 / f3=183873 / f4=125952 / f5=90608 / f7=94397 / f8=69865 / f9=18014 / f51..54≈3200`——**链路通、无过滤异常，是 Dijkstra 全展在前几层被状态数淹没**。
- 锚改 f10/ty75（S3 已证锚，boss_kill f10b0）：全题图 6M pops（30min 内跑完）⇒ **frontier=3065 但 killable=0**，驱动器如实具名 `no_killable_candidate`（candidates_considered=0）——真实数据下 frontier 生成/锚判定/候选过滤链路全部工作；「killable」门槛在全题图 6M pops 内尚未被任何候选达到（战斗 LB 下 f10 型75 对 6M 步内的攻防全不可杀），故 `replay_outer_path`→内层精解的实跑样本仍为空。combat-off 同锚 700k pops 前沿 1095（BYTE-IDENTICAL 基线）对照：combat-on 的 killable 判定让全题图锚候选目前全灭。
- 结论：**纯预算/组合爆炸问题，非结构性 bug**；F2 的「能不能出候选并精解」链路被 F5 合成判据与 S3 seg1 真实收敛（4.55M pops 前沿 3630）双重覆盖。

**F2-N（黄）**：findings 的「≥80M 待证」很可能严重低估。1M pops 尚未穷尽 f2-f9（f2 一个层就有 265k pops 且未收敛），按此外推触达 f20/f50 需要 pops 数量级远超 80M（数十亿级不无可能）。findings 未给出任何 pop 进度量化数据（本次复核的 13 层分布是第一条），下一位工程师若直接裸跑 80M 会重复烧机时。**建议先做支配剪枝加步数维或锚分层分治，再谈预算**。

## F3 单源转移重构行为等价——绿

- combat-off 口径 `cargo run --release -p game-solver --bin solver -- outer --max-pops 700000 --out` 与 `/tmp/outer_seg1_baseline_pre_s3.json` **BYTE-IDENTICAL**（cmp 无输出，亲自复跑；注意必须走 `--out`，stdout 是人类可读报告不是 JSON）。
- anchor 回落臂证据核对：块表 boss_kill 出口仅 f10/f15/f20/f25/f35/f40（python 定向查询），f50 只有 `script_gate slot16@(6,5)`、monsters 空——回落臂只在 combat-on 注入 monsters 后对 f50b0 生效，combat-off 仍 `NoAnchorBlock`，与 seg1 基线不变自洽（cmp 已证）。
- `to_floor==50 ⇒ (6,7)` 特判与 `solve_global.rs` landing 表逐字一致（landing: `target==50 → (6,7)`，solve_global.rs:293）。

## F4 budget_drifts 七字段——绿

- 合成判据 `s4a_budget_divergence_named`：fake 层 hp+7 ⇒ drift `{visit:1,floor:52,field:hp,outer:300,inner:307}` + gold 同记，绿。
- 真数据 drift 全 0 的原因（witness 未出 ⇒ 内层未启动）与 findings §3「**暂无样本**（外层未达锚 ⇒ 内层未启动）」的零样本标注一致，无冒充。

## F5 判据与变异重放——绿（附黄 F5-N）

- 亲验变异：M1（伪造回放）红 / M2（空实体表）红 / M3（预算改组尾）红——三条与 findings §1 自述一致，逐条复现。
- `cargo test -p game-solver` 亲自复跑：**exit 0、全部套件 0 failed**（含 556s 的 solve_global 套件与 189s 的 pure_pickup 套件）。未逐套加总到 219（F5-N 黄）：总数由 `test-ratchet` 门禁独立背书（PASS，total=1293(基线1293) 无回退）。
- 门禁复跑：`python3 gates/cli.py check` **PASS=13 FAIL=0 SKIP=0**（`status` 预检显式打印 visual-diff SKIP——该门禁在 P4 前未启用，非静默）。

## F6 门禁/夹带——绿

- 三笔 commit `--stat` 核查：c7766aa（outer.rs/outer_witness.rs/witness.rs/lib.rs/bin/solver.rs/tests）、927a156（solve_local.rs/outer_witness.rs/tests/ratchet 基线）、ce6f979（findings 单文件）——**无越界文件**；工作区干净。
- witness.rs 放宽 diff 核实：仅 `pub` → `pub(crate)` 可见性，无逻辑行变化。

## F7 findings 诚实性——绿

- 20M 档 35min 超时（exit=124，无 JSON）如实记录（§2），未掩盖；5M blocked 的 floor/reason/detail 三元组可直接操作；补跑命令留档清晰（§2 边界声明）。
- 唯一操作化缺口：F2-N 的量化缺失（见上）。

## F1-Y（黄）：商店双臂回溯无判据保护

**M4 变异**（/tmp 副本吞掉 `replay_route_inner` 的买臂，只留走臂）⇒ outer_witness 全部 6 判据**仍绿**。即：回放器商店双臂 DFS 的正确性目前只靠人工代码核对，无变异守护。真实 witness 一旦产出且途经商店，此面才第一次被真数据检验。建议补一条带商店臂的夹具判据（变异必须红）。

## F1-N2（黄）：端到端回放仅合成夹具

`s4b_replay_end_to_end` 的层解来自注入的 `real_walk_layer`（手工 route），非真求解器产出；且 `run_outer_cli` 全题跑**不调用** `replay_outer_witness` 自证。witness 产出路径（真 solve → record_floor → 回放全等）在真实数据上从未走通。非造假（findings 已如实声明机制面绿、实跑面空），但「S4 端到端真回放」仍是未验证命题。

## Confidence

- F1/F3/F5/F6 变异与复跑：A 级（亲跑命令+cmp+exit code）。
- F2 结论：A 级（探针实测 pop 分布）；外推部分（数十亿级）为 B 级推断。
- 总体 Confidence: 88

## 结论两句话

1. **S4 能不能推**：能推——三笔 commit 机制面真实、判据有牙、回归零漂移、findings 诚实；但「全题 witness」在可预见预算内出不来，下一步价值最高的不是继续烧 pops，而是外层搜索本身的收敛性改造。
2. **修法方向**：F1 回放与 F2 链路均查无造假/无结构性 bug，属纯预算爆炸——收敛建议按序：①支配剪枝加步数维（现剪枝明确不含步数，outer.rs dominated_seen 注释已自认）；②锚分层分治（seg 链 + 全局 LB 融合，S3 已证 f51..f10 段 4.55M 可收敛，逐段收敛后全局 witness 由段 witness 拼接）；③或 frontier 采样求草案 witness（放弃最优性声明）。80M 档先别裸跑，用本次 13 层 pop 分布做一次 cost-model 估算再决定。
