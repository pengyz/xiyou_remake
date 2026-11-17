# 批3 黄7 处置执行件 终验复核（2026-11-14）
| 项 | 值 |
|---|---|
| 对象 | `modern` 链 `261702d`→`1c6d738`（笔0 报告入库）→`1cc5cd2`（笔1 4 文件 11+/11−），两笔均**未推送** |
| 方案/方式 | 权威=`docs/superpowers/proposals/2026-11-14-batch3-yellow-fix-plan.md` @`261702d`（快审红0黄2绿16）；只读 `git/sed/grep/python` 逐条实测，本文件=唯一写入，未改其他任何文件、未推送 |
| 闸 | `python3 gates/cli.py check` 实跑 **PASS=13 FAIL=0 SKIP=0**（`state-baseline [GREEN] 74 场景 sha 全同` / `test-ratchet total=1075` / `docs-links` / `rust-tests` 全绿） |

## 总判：**需修复** —— 红 1 / 黄 3 / 绿 10 面；Confidence 88

**红1（阻断；message-only、零行为、修法极省）**：**Y1 证伪更正注未落任何 message**。方案 §7 笔1 明文「message 内按 §1.3 追加 Y1+Y7 证伪更正注（方案 C）」，§1.4 判据实跑：
```bash
$ git log --format=%B -1 1cc5cd2 | grep -c '3e7d235.*口径更正\|口径更正'   # 期望≥1 → 实测 0
$ git log --format=%B -1 1cc5cd2 | grep -c 'a.java:6261-6268 为 seed 行号'  # 期望≥1 → 实测 0
$ git log --oneline --all --grep 3e7d235                                    # 空（全链无 Y1 更正记录）
```
Y1 属报告 §12「随批修5」之一 → 随批修5 只落 4（Y2/Y4/Y6/Y7）+NY1/NY2。范围注：本 brief 笔1 枚举未含 Y1；若 lead 已裁定 Y1 不随本批，须落一处记账，否则 Y1 悬空。修法：笔1 未推送 → `git commit --amend` 补 §1.3 模板块（Y1+Y7 两条，**同时解黄1**），代价 = pre-commit 全量闸 ≈11 min；链上无 pin 引用 `1cc5cd2`。

**黄1**：Y7 message 侧更正**语义在位、非 §1.3 模板逐字**——有「seed 锚补全 `reference/seed/a.java:6261-6268`（仓内约定 seed 层非 deobf，a.java:6261 单点系 seed 行号）」，但 §1.4 判据3 字面串=0，且未提 `71908d6` 与 `reference-seed-integrity` 闸保护；与红1 同一处修。
**黄2**：NY1/NY2 写「仓内**两份**复核报告」实为**三份**——快审 `2026-11-14-batch3-yellow-fix-plan-rereview.md`（untracked，含 `20/22/24` 于 :33/:63）与对抗复核 `…-plan-review.md`（untracked，:27）不在排除清单（§4.1:192 / §8.11）；两份入库后 `git grep '20/22/24'` 由 7→10。NY1 本体（tracked-only 口径 + 排除式警示）已落地且数值实测全对。
**黄3**：笔1 message 写「Y2 `state.rs:35` 锚漂移修正」——Y2 实改 **`:39`**（`:35/:36` 是 Y6 的两行）；message 行号误标，判据不受影响（§2.3 用 `sed -n '39p'`）。
**绿**：必验面 1-10 除上述外逐条实测通过（下附原样输出）。

## 逐面实测
### 1. §7 两笔顺序与内容 — ✅（文件面全落齐；message 面缺 Y1 → 红1）
```bash
$ git log --format='%h %p %s' -3
1cc5cd2 1c6d738 docs(solver): 批3 随批黄 Y2/Y4/Y6/Y7+NY1/NY2 处置执行（零行为）
1c6d738 261702d chore(docs): 入库批3 对抗复核报告（红0黄7，随批修5/挂批4）
$ git show --stat --format= 1c6d738   →  …/2026-11-13-solver-batch3-review.md | 445 +++++（恰报告 1 文件）
$ git ls-files docs/findings/reviews/2026-11-13-solver-batch3-review.md   # 命中 ✅
```
- 顺序 ✅：笔0 父=`261702d`、笔1 父=`1c6d738` → 先入库后改指针（§4.4 唯一顺序风险解除）。
- **逐字比对**（脚本抽方案 §2.2/§3.2/§4.2/§5.2 backticked 期望串 vs 现文）全 PASS：`Y2 state.rs:39`、`Y4 solve_global.rs:11`、`Y6 state.rs:35`、`Y6 state.rs:36`、`Y6 plan:62 子串`、`Y6 plan:165 子串`、`Y7 新锚在位+裸锚消失`；NY1（`:192`）、NY2（`:241/:327/:330`）在位。
- **方案外多改行 = 0**：4 文件改动行恰 11 = Y2 1 + Y4 1 + Y6 4（plan:62/165 + state:35/36）+ Y7 1 + NY1/NY2 4，无第 12 行；Y3/Y5 按 §6 挂批4 未动 ✅。

### 2. 行数守恒 — ✅（`git show <rev>:<file> | wc -l` 前/后/工作树三点一致）
| 文件 | 笔1前 `261702d` | 笔1后 `1cc5cd2` | 工作树 |
|---|---|---|---|
| `crates/game-solver/src/state.rs` | 106 | 106 | 106 |
| `crates/game-solver/src/solve_global.rs` | 652 | 652 | 652 |
| `docs/superpowers/plans/2026-11-09-solver-implementation-plan.md` | 506 | 506 | 506 |
| `docs/superpowers/proposals/2026-11-14-batch3-yellow-fix-plan.md` | 335 | 335 | 335 |
`--numstat` = 1/1、3/3、3/3、4/4 → 各 ins==del ✅（`state.rs:40` `pub const QOL_ITEMS: &[i32] = &[15, 21, 23];` 未漂 ✅）。
### 3. Y2 / Y7 锚实文 — ✅
```bash
$ sed -n '39p' crates/game-solver/src/state.rs
/// 战斗掉落只掉金。计划 :62/:165 已原地日期更正（保留原主张不删，AGENTS §3.6）。
$ sed -n '62p;165p' …solver-implementation-plan.md | grep -c '【2026-11-12 更正·F4】'   # 2 ✅
$ grep -rn ':60/:163' crates/ | wc -l                                                 # 0 ✅
$ sed -n '345p' …plan.md | grep -o 'reference/seed/a\.java:6261-6268 q=5 为其渲染相位'  # 1 ✅
$ grep -n '；a\.java:6261' …plan.md | wc -l                                            # 0 ✅
$ sed -n '6261,6268p' reference/seed/a.java
6261 } else if (string.equals("LAY")) { … 6267 this.q = (byte)5; … 6268 } else if (string.equals("ROS")) {
```
seed 层实文 = LAY handler 6261-6268（`q=5` 在 6267）✅；deobf 对照 `8361` LAY if / `8367` `dialogPhase = 5;` / `8368` ROS else-if / `8369` 已是 ROS 体 → 方案自纠 8361-8368 正确，未照抄报告的 8361-8369 ✅。
### 4. Y4 头注与实作一致 — ✅
```bash
$ sed -n '11p' crates/game-solver/src/solve_global.rs
//! （探索区间门 4 分量 canonical 实作：tutorialSeen 背包权威源、f_bool_13 逐 tick 派生几何、findFloorGateEntity 层数据，由 solve_local.rs:1372-1399 cloud_goals 派生；changeFloor min/max 已归 st.floors，本文件 :383-389 落闸——均不入 flags（裁定见批3 复核报告 §9.4））、battle-victory 脚本 …转移边。
$ sed -n '1372p;1374p;1384p;1397p;1399p' crates/game-solver/src/solve_local.rs
    fn cloud_goals(&self, st: &St) -> Vec<(i32, bool)> {       # 1372 起
        if !self.cloud || !st.player.item_types.contains(&15) { # 1374 背包权威源
        if !(adj(self.stair_up) || adj(self.stair_down)) {      # 1384 邻梯几何（1377-1386）
            }                                                   # 1397 ty7/8 实体循环尾（1388-1397）
    }                                                           # 1399 函数尾
$ sed -n '385p;386p' crates/game-solver/src/solve_global.rs | grep -c 'st.floors'   # 2
```
四分量相符：tutorialSeen=`item_types.contains(&15)`（1374 背包权威源，无「唯一写点」绝对表述）✅；f_bool_13=1377-1386 `adj()` 派生几何不存值 ✅；findFloorGateEntity=1388-1397 `self.level.entities` 层实体 ✅；changeFloor min/max=`st.floors`（385/386）+387-389 落闸 ✅；`solve_local.rs:1366-1371` 既有 doc 同口径（「区间门不在层内——由 solve_global 落闸」）✅；`均不入 flags` 与实作一致（cloud 路径无 flags 写点）✅。§3.3 判据 =0/1/2 ✅。
### 5. Y6 出处指针 — ✅
```bash
$ grep -rn '批3 报审件' crates/ docs/superpowers/ | grep -v 2026-11-14-batch3-yellow-fix-plan.md | wc -l   # 0 ✅（笔1 前=3）
$ grep -c 'reviews/2026-11-13-solver-batch3-review.md' …plan.md                       # 2 ✅
$ grep -c 'reviews/2026-11-13-solver-batch3-review.md' crates/game-solver/src/state.rs # 1 ✅
$ git ls-files --error-unmatch docs/findings/reviews/2026-11-13-solver-batch3-review.md                    # 通过 ✅
$ grep -n '^### 5\.2' docs/findings/reviews/2026-11-13-solver-batch3-review.md
204:### 5.2 零获取穷尽对照抽验 … sprite1703+GIN×4+shop17 ✅      # §5.2 = 204-215
$ sed -n '210p;212p' …2026-11-13-solver-batch3-review.md
sprite0..54 共 1703 条记录（floor1..54 = 1701）/ type20/22/24 全世 0 枚      # 世界零获取 ✓
```
目标文件 tracked、§5.2 真含 sprite 1703 普查命令与输出 ✅；`（F1 偏离见批3 报审）`（plan:345 尾）按 §4.2「不改」保留=1 ✅。
### 6. NY1 / NY2 — ✅（NY1 数值全对，残留口径见黄2）
```bash
$ sed -n '192p' …yellow-fix-plan.md | grep -o 'NY1 警示.*'
NY1 警示（笔0 1c6d738 后实测）：本行判据现=7 行**（批3 复核报告 2 :200/:212 + plan 2 + 本方案 3；入库前 5=报告 untracked 未计）——独立出处计数须逐份排除两份复核报告与本方案自身（:180 同款排除式）。
$ git grep -n '20/22/24' -- '*.md' | wc -l                            # 7 ✅（报告2 + plan2 + 方案3）
$ sed -n '241p;327p;330p' …yellow-fix-plan.md | grep -o '2026-11-13-solver-batch3-review.md' | wc -l   # 3 ✅（§5.1/§8.10/§8.13）
```
NY2 三处全名齐 ✅；NY1 的 tracked-only 口径（「入库前 5=报告 untracked 未计」）+ 排除式警示在位 ✅；排除清单仍写「两份」= 黄2。
### 7. 零行为 — ✅
```bash
$ git show 1cc5cd2 --format= -- crates/ | grep '^[+-]' | grep -v '^[+-][+-]' | grep -vE '^[+-]\s*(//!|///)'
                                                          # 空：crates 内 8 行改动全为 //! / /// 注释
$ git diff --name-only 261702d..1cc5cd2 -- reference/ gates/ data/status/ crates/game-solver/tests/ | wc -l   # 0
$ git diff 261702d..1cc5cd2 -- crates/game-core/src/script_ops.rs | wc -l                                     # 0
```
`state.rs`/`solve_global.rs` 改动行全在 `///`/`//!` 内（非可执行行）✅；`reference/`、`gates/`、ratchet JSON、测试、`script_ops.rs:67`、`solve_global.rs:451`、`plan:178/:343/:423` 需求原文零 diff ✅（plan 只动 `:62/:165/:345` 三个 hunk）✅。

### 8. 判据独立重跑 — ✅（全量闸实跑）
- §2.3/§3.3/§4.3/§5.3 判据**原样跑通、输出=预期**：`计划 :62/:165` 命中 / `:60/:163`=0 / F4=2 / 裸锚=0 / seed 锚=1 / Y6 残留=0 / plan=2 / state=1 / `wc -l` 106·652·506·335 不变 / `numstat` ins==del / `ls-files --error-unmatch` 通过。
- §4.3 判据2：`python3 gates/cli.py check`（全量 ≈10 min，本会话实跑）→ **PASS=13 FAIL=0 SKIP=0**，其中 `state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同`、`test-ratchet total=1075(基线1075) 无回退`、`docs-links 全部相对链接有效`、`rust-tests cargo test --release 全绿`、`reference-seed-integrity 8457 行 sha256 通过` ✅。

### 9. 夹带 — ✅
```bash
$ git diff 261702d..1cc5cd2 --name-only
crates/game-solver/src/solve_global.rs
crates/game-solver/src/state.rs
docs/findings/reviews/2026-11-13-solver-batch3-review.md
docs/superpowers/plans/2026-11-09-solver-implementation-plan.md
docs/superpowers/proposals/2026-11-14-batch3-yellow-fix-plan.md     # 恰 5，无第 6 个 ✅
$ git status --porcelain | grep -v '^??' | wc -l                      # 0（tracked 与 HEAD 一致）
```
笔0 = 报告 1 文件（untracked→tracked，445 行）+ 笔1 = 4 文件 ✅；方案自身 4 行改动 = 快审 NY1/NY2 授权项，非夹带 ✅。

### 10. 链自证 — ✅
```bash
$ git rev-parse 1c6d738 1cc5cd2 261702d e4b19a6 71908d6
1c6d738fb5ce93140b42c74060c0933d15ec81be / 1cc5cd2c8837e327dff32b7dfdc795454191b639 / 261702d09cc1276340395ab3c40db05c74021a8b
e4b19a6bcedb6c3de63687654cd32279bb7d0a07 / 71908d643797b6197c93f87d8e8e560e97e02c93   # 5/5 全过；3e7d235 未被 reword ✅
```
**Confidence: 88**（判据与行号全部本会话原样实测，A 级可复现；扣分点：红1 定级取决于「方案 §7 要求」与「本 brief 笔1 枚举未含 Y1」之间的范围裁量，需 lead 裁定——但「全链无任何 Y1 更正记录」是实测事实，非推断）。
