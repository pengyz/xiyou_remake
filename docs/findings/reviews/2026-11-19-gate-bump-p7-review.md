# P7 撞门片 · 独立对抗复核（2026-11-19）

- 被审对象：`modern` HEAD `4be401e`（B 片撞门）+ `d963ec0`（A 片守卫/ignore），父 `fb5c15b`
- 复核立场：只读代码 + 自跑实测；实现者自述与「测试全绿」均不作为证据
- 环境：`/home/peng/workspace/xiyou_remake`，分支 `modern`；变异在 `/tmp/p7-mut`（HEAD 的 `git archive` 副本，`assets/raw` 软链，`CARGO_TARGET_DIR=/tmp/p7-target`），未改仓库任何 tracked 文件

## 1. 计数表

**红 1 / 黄 3 / 绿 6。**

| 面 | 判定 | 结论（一句话） | 证据 |
|---|---|---|---|
| F1 撞门语义同源 | **黄** | 判门/脚本/tick 确实走引擎（`try_run_scene(param,false)`→`load_level_script`→`tickdrive`→仅 `Completed` 回读），但 `Pending`/`Stalled`/`pos` 变化三处**静默丢弃且无计数**；commit message 说「断言 pos 未变」，代码实为 `continue` | `crates/game-solver/src/solve_local.rs:3054-3106`（判门 `:3078`、脚本 `:3081`、回读 `:3085`）、`:3092`（pos 守卫）、`:3105`（Pending/Stalled→None）；`crates/game-solver/tests/gate_bump.rs:126-135` |
| F2 `StGate` 安全边界 | **绿（附黄注）** | `1/2/24` 三臂确实不碰两常量；`32` **碰** `cross_floor_snapshot(23,12)`（`gate.rs:97`），但 `build_scene` 也不注入跨层快照 ⇒ 引擎侧同为 `false`，两侧无分歧；注释把 `predict_hp_loss_vs` 归给 `param=15` 是事实错误（真消费者是 `param=34`，且 34 是 walk-only，撞面永不读它） | `crates/game-core/src/rules/gate.rs:97-103`（32）、`:105-110`（15）、`:130-135`（24）、`:152-158`（34）；`crates/game-solver/src/solve_local.rs:3591-3637`（注释 `:3585-3590`、`cross_floor_snapshot :3630`、`predict_hp_loss_vs :3633`） |
| F3 预筛健全性 | **黄** | 「走能开 ⇒ 走臂支配」在**非可走格**上为假：全 52 门扫描出 `f41 param=13@(10,2)`、`f48 param=38@(4,1)` 是 `bump=1 ∧ walk=1` 且格不可走 ⇒ 被预筛排除、走臂也进不去（该门**两条臂都没有**）。两脚本是 `CES_` 陷阱（`scripts[13]=CES_6_10_2 CES_60_10_2`、`scripts[38]=CES_6_4_1 CES_61_4_1`）⇒ 今日无收益损失，但所述不变量与测试都不承重 | 自写 `examples/scan_gates.rs`（`/tmp/p7-mut`）输出 `TOTAL_GATES=52 BUMP_ONLY=3 ON_NONWALKABLE=3`；`solve_local.rs:3008-3025`（预筛只比 `evaluate` 两臂，不看地形）；`gate_bump.rs:150-170`（判据 2 只测可走格） |
| F4 回归诚实性 / memo | **黄** | `f51..f54` **逐字节不变**（HEAD 与 `fb5c15b` 独立 worktree 的 `solver run` JSON `cmp` 无输出、字段级 diff 0 条）；但 `BUMP_MEMO` **从不清理**、键不含 `pos` 与门格（脚本坐标全绝对 ⇒ 今日不可观测，属 P3 同类潜伏）；`GATE_BUMPS` 两处清零、thread_local 不跨测试 | `/tmp/p7_head_run.json` vs `/tmp/p7_fb5_run.json`；`solve_local.rs:3562-3567`（BUMP_MEMO）、`:3071/:3095`（仅读写，无 clear）、`:296`/`:668`（GATE_BUMPS 清零） |
| F5 三条守卫承重 | **绿** | M1/M2/M3 各只打红**对应那一条**；M4 三条同删 ⇒ 3 条全红，无相互遮蔽。扣分项：M5 删 `try_closed_form` 的调用点后**全绿** ⇒ 只测了函数、没测接线 | 变异表 M1–M4；`crates/game-solver/src/pure_pickup.rs:427-470`（函数）、`:802-805`（调用点）、`:868-980`（三条判据） |
| F6 `#[ignore]` 退出 ratchet | **绿** | `-- --ignored --list` = 0 条；`pure_pickup` 14 条全为会跑；基线 `total=1201` 与实测一致；基线 diff 只增不减、无 ignore 名额 | `cargo test -p game-solver --test pure_pickup -- --ignored --list`；`data/status/baselines/rust-test-counts.json` diff（`game_core 412→414`、`game_solver lib 46→49`、`gate_bump(test)=5`、`pure_pickup 7→14`、`witness 8→9`、`total 1183→1201`） |
| F7 f3 数值链 | **绿（附黄注）** | 20k/200k/1M = **3427/3827/3877**、三档全 `truncated`、`flags_out` 恒 `0xe337be0000000000`，全部复现；200k 路线含 **3 枚 ty32**（`(4,2)(5,1)(6,2)`）⇒ `3227+600=3827`；1M 多捡 1 枚 ty31（`(11,7)`，+50）；两场战斗零损（`def=730`，敌 atk 远小于 def ⇒ `predict_battle_hp_loss` 恒 0）。`4027 = 3227 + 3×200 + 4×50` 是**算术上界**（全层 ty32 仅 3 枚、ty31 仅 4 枚），1M 只捡到 1 枚 ty31 ⇒ 3827 确非最优。黄注：文档/测试引用的 `examples/probe_f3` **在仓库中不存在** | 自写 `examples/probe_f3_chain.rs`、`examples/ent_f3.rs`（`/tmp/p7-mut`）；`crates/game-core/src/combat.rs:106-129`；`docs/findings/2026-11-19-f3-gate27-walk-only.md:94-110`；`crates/game-solver/tests/pure_pickup.rs:564,589`（坏指针） |
| F8 阳性结论夸大 | **红** | 全游戏唯一「撞-only」是 `f35 param=24` 三格，撞后**只置 flag24**（`flags 0x0→0x1000000`，实体 28→28 不变），而没有任何门臂读 `flag(24)`；f35 从 `(1,11)` **不用撞门 23 步直接出层**（`ok=true, hit=false`）⇒ 本片**零进度收益**。「真能力缺口」只在「动作覆盖」意义上成立，在「关卡需要」意义上不成立 | 自写 `examples/probe_f35.rs`、`examples/f35_solve.rs`；`docs/findings/2026-11-19-f3-gate27-walk-only.md:122`（「真能力缺口」）；`solve_local.rs:3004`（同口径注释） |
| F9 门禁与夹带 | **绿** | `gates check`：**PASS=13 FAIL=0 SKIP=0**，`test-ratchet total=1201(基线1201)`；两笔 commit 文件与声明一致（A 4 文件 / B 3 文件）；reflog 线性、两笔均非 amend（历史 amend `7a8f29e/abcd026` 在父之前）；6 份他人 review 文档仍 untracked、未入库 | `python3 gates/cli.py check`；`git reflog --date=iso`；`git show --stat d963ec0 4be401e` |
| F10 断言卫生 | **绿（附黄注）** | 既有断言**零放宽**；`terminal_mismatch` 抽取后判定顺序（atk/def→三钥→gold→背包）、四条理由串、`REASON_RANK_REPLAY` 与 `note_reason(...,0,rank)` 逐字不变；`solve_global`/`witness` 未动；`StGate` 只筛候选，**效果仍由真引擎重判** ⇒ 未绕过 `GameScene` 判定。黄注：M5/M6/M7 全绿说明「接线」与两个硬编常量无测试；`gate_bump.rs:94,262` 把槽 24 写成 `TAK_40_40`（实为 `TAK_101_105`） | `git diff fb5c15b..HEAD -- crates/game-solver/src/pure_pickup.rs`；`pure_pickup.rs:427-470`；`scene.rs:4824-4870`（真 GateView）；`gate_bump.rs:94,262` |

## 2. 变异表（我自己跑的，不是抄的）

目标：`guard`=`cargo test --release -q -p game-solver --lib p7_guard -- --test-threads=1`；`gate_bump`/`pure_pickup`/`witness` 为同名 `--test`。基线：3 / 5 / 14 / 9 全绿。

| # | 变异 | guard | gate_bump | pure_pickup | witness | 判读 |
|---|---|---|---|---|---|---|
| M1 | 删 `atk/def` 臂（`if false && …`） | **1 红**（仅 `replay_atk_def_guard_is_load_bearing`） | 绿 5 | 绿 14 | 绿 9 | 独立承重 |
| M2 | 删三钥臂 | **1 红**（仅 `replay_keys_guard_is_load_bearing`） | 绿 5 | 绿 14 | 绿 9 | 独立承重 |
| M3 | 删背包臂 | **1 红**（仅 `replay_bag_guard_is_load_bearing`） | 绿 5 | 绿 14 | 绿 9 | 独立承重 |
| M4 | 三条同删 | **3 红**（0 passed / 3 failed） | 绿 5 | 绿 14 | 绿 9 | 无遮蔽 |
| M5 | 删 `try_closed_form` 里的 `terminal_mismatch` 调用点 | 绿 3 | 绿 5 | 绿 14 | 绿 9 | **接线零覆盖**（黄） |
| M6 | `StGate::cross_floor_snapshot` → `true` | 绿 3 | 绿 5 | 绿 14 | 绿 9 | 常量未被测（黄） |
| M7 | `StGate::predict_hp_loss_vs` → `1` | 绿 3 | 绿 5 | 绿 14 | 绿 9 | 常量未被测（黄） |

M5–M7 是「变异存活」：不是代码错，而是**测试无法区分正确实现与被改坏的实现**。M5 意味着「守卫被接线」这件事只由 diff 保证，不由测试保证；M6/M7 意味着 F2 的安全边界是**注释承诺**，不是判据。

## 3. Confidence

**86 / 100。**

扣分来源：
- 未亲自跑全量 `cargo test --workspace`（交给 `gates check` 的 `rust-tests`，PASS；但那是门禁口径，不是我逐 crate 复核）。
- `BUMP_MEMO` 跨调用碰撞只有代码推理，未构造出可观测的错值（脚本坐标全绝对，构造不出）。
- f35 真实入场（来自 f34 出口）未取证，我用 `(1,11)`；结论「不用撞门可出层」对该入场成立，对真实入场是推断。
- 52 扇门只做了扫描表 + 抽验（f2 p1/p2、f29 p32、f35 p24、f41 p13/p25、f48 p38），未逐门手工核对。
- `4027` 是算术上界（3×ty32 + 4×ty31），未找到实际路线。

## 4. 两句话

**「这片能不能留在主干」**：能留——A 片是净改善（守卫独立承重、ignore 名额退出、文档更正），B 片不改变任何链上输出、不造假状态、效果全由真引擎产生；但必须把 B 片的「真能力缺口」降级为「动作覆盖 + 零进度」（唯一实例 f35 p24 只置无人读的 flag24，且 f35 不用撞门也能出层），并补 `BUMP_MEMO` 清理与「非可走格两路都能开」的反例测试。

**「f3 下一步该做支配还是抛弃层间接力改做别的」**：做**支配/剪枝**，不要抛弃层间接力——接力已把 f51..f54 打穿，f3 的瓶颈是状态爆炸（20k/200k/1M 仍 `truncated`，`out_hp` 随界单调升），不是缺动作；下一步应用 `4027` 作可采纳上界做 branch-and-bound（或对称/支配剪枝），而不是继续往 `expand_inner` 加层间接力动作。

## 5. 未验证清单

- 全量 `cargo test --workspace`（只跑了 `gates check` 的 rust-tests 汇总）。
- `BUMP_MEMO` 跨 `solve_local` 调用/跨同层号合成层的实际错值（推理为「脚本坐标绝对 ⇒ 不可观测」，未实证）。
- f35 的真实入场格与真实出层路径（用 `(1,11)` 代替）。
- `f41 param=13` / `f48 param=38` 是否在任何候选链/必经路径上（只知不在当前 6 层链）。
- 52 扇门的逐门 `bump/walk` 手工核对（用扫描程序一次性统计）。
- `4027` 是否真能走到（只验证了它是算术上界）。
- `f41 p13`/`f48 p38` 的 `CES_` 脚本是否在引擎里确实只生成陷阱、不改变连通性（只按脚本字面判断）。

## 6. 必跑命令清单（复现用）

```bash
cd /home/peng/workspace/xiyou_remake

# 基线：两片测试全绿
cargo test --release -p game-solver --test gate_bump
cargo test --release -p game-solver --test pure_pickup
cargo test --release -p game-solver --lib --release p7_guard

# F6：ignore 已退出
cargo test --release -p game-solver --test pure_pickup -- --list
cargo test --release -p game-solver --test pure_pickup -- --ignored --list

# F4：链上逐字节不变（HEAD vs fb5c15b worktree）
cargo run --release -p game-solver --bin solver -- run --max-floors 30 --out /tmp/p7_head_run.json
git worktree add /tmp/p7-fb5 fb5c15b && ln -s "$PWD/assets/raw" /tmp/p7-fb5/assets/raw
(cd /tmp/p7-fb5 && cargo run --release -p game-solver --bin solver -- run --max-floors 30 --out /tmp/p7_fb5_run.json)
cmp /tmp/p7_head_run.json /tmp/p7_fb5_run.json

# F7：f3 三档（用仓库副本加自写 example，勿改仓库）
#   /tmp/p7-mut = git archive HEAD | tar -x; assets/raw 软链; CARGO_TARGET_DIR=/tmp/p7-target
SOLVER_LAYER_LIMIT=20000   cargo run --release -q -p game-solver --example probe_f3_chain
SOLVER_LAYER_LIMIT=200000  cargo run --release -q -p game-solver --example probe_f3_chain
SOLVER_LAYER_LIMIT=1000000 cargo run --release -q -p game-solver --example probe_f3_chain

# F8：f35 唯一撞-only 门 + 不撞也能出层
cargo run --release -q -p game-solver --example probe_f35
cargo run --release -q -p game-solver --example f35_solve

# F3：52 门 bump/walk 扫描
cargo run --release -q -p game-solver --example scan_gates

# F9：总闸
python3 gates/cli.py check
```
