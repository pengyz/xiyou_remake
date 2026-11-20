# P14 定向对抗性复核：`d9324a3`（代码）+ `b7dedca`（文档）

日期：2026-11-20。复核者：独立对抗性复核 agent（不参与实现）。
父提交：`d9324a3^ = d58e440`（逐层 keydemand 只读分析片）。
所有复现均在 `/tmp/z`（HEAD 解包 + `assets/raw` 符号链接）与 `/tmp/z2`（父提交解包）完成，
未触碰工作树暂存区，未 commit。

## 计数

**绿 9 / 黄 1 / 红 0。**（黄 = solve_local.rs 三处 pre-existing doc 注释漂移，见判定 2）

## 逐条判定

### 1. cmp_exit 软序与 next_demand（objective.rs）— 绿

命令/证据：
- `crates/game-solver/src/objective.rs:65-69`：`cmp_exit = b.hp.cmp(&a.hp).then(b.gold.cmp(&a.gold)).then(a.steps.cmp(&b.steps))`，纯字典序，无加权和；`keys` 字段在 `ExitScore` 中但不进比较（:57-63 注释钉死「比较的唯一输入面」不含 keys 参与）。
- `objective.rs:92-128`：`next_demand` 全部走 `floorgraph::edges_of` + `StairUp` 过滤 + `keydemand::analyze_floor`，无硬编码层号；多后继逐色 `max`（:120-123）；任一后继非 `Ok`/缺规格 ⇒ `Undefined(tag)`（:111-119，all-or-nothing，文档已声明）。
- tests/objective.rs 11 判据在 `/tmp/z` 全绿（`cargo test -p game-solver --test objective` = 11 passed）。

### 2. 接线点与 legacy 不动 — 绿（带一处黄）

命令/证据：
- `git show d9324a3 -- crates/game-solver/src/solve_local.rs`：改动仅 `solve_local_exit`（:526 `pick_exit`）、`solve_local_exit_toward_shops`（:587-590）、`best_goal`/`better_exit`/`pick_exit`（:605-638）与新增 `p10_objective_pick` 4 单测。
- `grep -n "pick::<HpLoss" solve_local.rs` → 唯一遗留调用在 `solve_local.rs:304`（legacy Q1 面），未动。
- 层内支配/剪枝（`weakly_ge`/`dominates`/`layer_node_limit`）零 diff。
- 【黄-1】**doc 注释漂移**：`solve_local.rs:509-510`（`solve_local_exit` 文档仍写「目标 = 出层血量最大…复用既有 HpLoss」）、`:534`（toward 文档「挑选是 HpLoss」）、`:543`（「仍按 HpLoss 在该子集取最优」）——三处 pre-existing 注释现在与实际比较器（cmp_exit）矛盾。纯文档问题，不影响行为，建议下片顺手更正。

### 3. witness v3 诊断面 — 绿

- `witness.rs:87` schema v2→v3；`WitnessFloor` 追加 4 字段（:213-231），追加在 `truncated` 之后不重排既有字段（tests/witness.rs 断言 ② 钉死声明序）。
- `record_floor`（witness.rs:393-447）：`key_shortfall` 由逐色 `have < need` 计算，**只落账**；`ok: got.ok` 原样（:430），无任何阻断分支。
- 实测 JSON（`/tmp/z` `solver run`）：schema=xiyou/witness/v3；f3 `keys_out=[0,1,0]`、`demand=[2,0,0]`、`key_shortfall={need:[2,0,0],have:[0,1,0]}`、ok=true；f54 `demand_undefined="no_entry"`（Undefined 携带 tag 落盘，**不会静默变 0**——`demand_undefined != null` 即消费方可见信号）。

### 4. keydemand 动面唯一 — 绿

- `git diff d58e440..d9324a3 -- crates/game-solver/src/keydemand.rs`：仅 `tag()` 一处（`Unreachable => "blocked_by_assumption"`，keydemand.rs:114），带日期更正注；算法/判定零改动。
- 实测 `solver keydemand`（/tmp/z）：`层 54 / ok 49 / no_entry 2 / no_up_exit 2 / blocked_by_assumption 1 / needs_more_than_grid 0` —— 与 findings 文档 §3 一字不差。
- tests/objective.rs `f29_status_is_blocked_by_assumption` 绿。

### 5. G1「纯软」核验 — 绿

- grep 全仓：`demand/next_demand/key_shortfall/keys_out` 只出现在 objective.rs（产出）、witness.rs（落账）、tests、keyflow.rs（**既有独立模块**，与本片无关，零 diff）。**没有任何支配、剪枝、ok/可行性分支读取这些字段**；比较面只进 `cmp_exit`（hp/gold/steps）。
- `better_exit` Equal 分支用路线字典序（决定性 tie-break），不引入 demand。

### 6. G2 多后继逐色 max 口径 — 绿

- 后继层 keydemand 是「本层入场→该层上行出口」的最少钥匙；出层方不知道会走哪个上行梯，逐色 max 是唯一不低估任何后继的口径（保守），单测 `next_demand_multiple_successors_takes_componentwise_max` 钉死（[1,0,0]∪[0,0,1]→[1,0,1]）。承认局限（回拾/复合需求看不到）——文档 §「一步前瞻的局限」已如实声明，且 demand 不进比较，保守性误差无行为后果。
- Undefined 传播：`demand_undefined=Some(tag)` + `demand=[0,0,0]` 成对落盘，f54 实测可见 tag，无静默置 0 的路径。

### 7. G3 f52 102→124 步 — 绿（软序有意变更，非支配污染）

父/子对照（同一机器、同一 assets）：

| 层 | 父 d58e440（v2） | 子 d9324a3（v3） |
|---|---|---|
| f52 steps | 102 | **124** |
| f52 out_hp / exit | 427 / (11,11) | 427 / (11,11)（不变） |
| f52 out_gold | 15 | **1021**（+1006） |
| f51/f53/f54/f1/f2/f3 | 69/25/63/6/10/88 步 | 逐一相同；f3 hp=4027/steps=88 不变 |
| floors / blocked | 7 / f4 no_exit_toward | 同 |

- 复现命令：`cd /tmp/z && cargo run -p game-solver --bin solver -- run`（及 /tmp/z2 同命令）。
- 更正注落在 `tests/pure_pickup.rs:427-433`（OLD_CHAIN 前，含日期、机制、意图指针），f52 行 102→124 且 hash 更新，f51/f53 逐字段不变。
- steps 差距量级：+22 步（21.6%），且 **有界**——gold 软层排在 hp 之后、steps 之前，gold 上界是该层金币总量（f52 全层金 ~1006），一旦 gold 达层内最大即进入 steps 比较；搜索空间仍受 `layer_node_limit` 管辖。不存在「steps 无界退化」通道。最坏情形是「同 hp 多条等 gold 路线里选步少」，仍是局部最短。

### 8. G4 schema v3 破坏面 — 绿

- 仓内 grep `witness/v2`：仅历史 findings 文档（2026-11-19 两篇 + 本片 docs，属历史记录，按 AGENTS §3.6 不改写）与 witness.rs 更正链注释。**无脚本/工具消费 schema 字符串**（gates/、tools/、crates/ 无命中）。
- tests/witness.rs 两处 v2→v3 断言均带「【2026-11-20 更正（P10）】」注，保留原主张语境（原断言文本仍在 diff 上下文可见），符合更正文化。

### 9. G6 变异可复现 — 绿（M2、M4 独立复红）

- **M2**（/tmp/z，objective.rs `cmp_exit` gold 提到 hp 前）：`cargo test --test objective hp_beats_gold` → **FAILED**（1 failed）。还原后 11 passed。
- **M4**（/tmp/z，witness.rs `record_floor` 尾部追加 `if key_shortfall.is_some() { ok = false }`）：`cargo test --test objective key_shortfall_reported_not_blocking` → **FAILED**（panic 断言「key_shortfall 是诊断位，绝不允许改写 ok」）。还原后 11 passed。
- M1/M3 未逐发复跑（时间预算），但其对象判据（`demand_fields_are_diagnostic_only` 含同 hp keys-vs-gold 用例、`next_demand_multiple_successors_...`）均在绿集内，且 M1 初发未红→补强的自述与判据现状（:175-177 同 hp 用例）一致，可信。
- 注：/tmp/z 无 `.git`，变异还原用宿主仓 `git archive HEAD <file> | tar -x` 完成；期间发现 tar 回写 mtime 早于变异编辑会导致 cargo 增量编译用过期产物——复跑前已 `touch` 全部源文件并用 `grep MUTATION`=0 验证还原干净（此坑不影响结论，仅记录给后续变异实验者）。

### 10. G7 门禁与夹带 — 黄（结论待工作树确认）/ 绿（夹带）

- **夹带**：`git show --name-status d9324a3`、`b7dedca`——10+2 个文件，无 `docs/superpowers/proposals/…`、无 `data/naming/`、无 `reference/`、无 `original/`。绿。工作树上并发子代理的提案文件处于 staged 状态，**不在两笔 commit 内**，本复核未触碰。
- **ratchet**：`data/status/baselines/rust-test-counts.json` total 1244→1259；/tmp/z `gates/cli.py check` 内 `test-ratchet — total=1259(基线1259) 无回退 PASS`；`cargo test -p game-solver` 分项实测 game_solver(lib)=68、objective(test)=11 与基线一致（+15 = 68-64+11+4？核对：lib 64→68 = +4 单测，objective 11 新增，合计 +15）。绿。
- 【已裁定，原黄-2】**gates check 总数**：真实工作树 `python3 gates/cli.py check` 复跑（只读，2026-11-20 12:28 完成）＝ **PASS=13 FAIL=0 SKIP=0**，与被告自述一致。对照：/tmp/z 与 /tmp/z2（父提交）副本均为 `PASS=10 FAIL=3`（docs-links / rust-tests 夹具 / state-baseline），且**父提交同样 3 FAIL** ⇒ 副本失败是环境性（缺未跟踪 reviews 文件与 `analysis/` 再生产物），非本两笔引入，亦非有意弱化门禁。黄-2 升绿，最终计数 **绿 9 / 黄 1 / 红 0**。

## Confidence

- 行为类结论（G1/G2/G3/G5/G6）：**A/B 级**（命令+输出可复现，父/子对照实测）。
- 门禁总数（黄-2）：副本口径已定（父=子=10/3），工作树口径见补记；Confidence 90（工作树结果落定后 95）。

## 两句话裁定

1. **这两笔能不能合并**：能——文档 commit（b7dedca）只新增两篇 findings、零代码触面，与 d9324a3 是同一逻辑变更的「实现+取证」 halves，合并不损失任何可追溯性；保持两笔也无害（AGENTS 小步提交精神），任选皆可，**无阻塞项**。
2. **f52 步数退化要不要设上限/判据**：不必设硬上限——差距 +22 步有界且根因是「同 hp 下 gold 优先于 steps」的有意裁定（血/出口零变化，下游六层逐字段不变）；但建议加一条**软判据**（如 OLD_CHAIN 式锚：f52 steps ≤ 某宽松带 + out_gold ≥ 旧值），防未来再动比较器时 gold 层被无声滥用；判据落点就是既有 pure_pickup.rs OLD_CHAIN，无需新设施。

## 补记（工作树 gates check，已落定）

真实工作树 `python3 gates/cli.py check`（只读，2026-11-20 12:28 完成）：**PASS=13 FAIL=0 SKIP=0**（13 门全绿：original-integrity / assets-manifest / reference-seed-integrity / knowledge-format / docs-links / naming-ledger / rust-tests / test-ratchet total=1259 无回退 / table-content-hash / table-evidence / state-machine-anchors / asset-truth-manifest / state-baseline 74 场景 sha 全同）。被告自述第 8 条成立。
