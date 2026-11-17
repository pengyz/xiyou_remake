# 求解器批1 对抗复核报告（2026-11-10）

**复核对象**：`modern` 分支 origin/modern=5f48fe4 → HEAD=c2f2d84（3 commit 未推送）
**链**：`1ad29e6`（门面 2 files +63）→ `c679ae3`（G5a/G5b 锁+冒烟 1 file +521）→ `c2f2d84`（script_abs 直调+对拍 2 files +704）
**参照**：提案 `docs/superpowers/proposals/2026-11-09-game-core-solver-facade-proposal.md`（§3 五闸 / §4 施工图 / §5 R1）、计划 `docs/superpowers/plans/2026-11-09-solver-implementation-plan.md` :291-314（判据 :307-311）
**角色**：独立对抗复核者（任务=设法证明它不该推）；只写本文件，未改代码、未动测试、未推送；所有变异均已还原（`git diff` 空）。

---

## 总判：**可推送**（附 2 个非阻断不符点，见文末红点清单）

报审 7 条声称逐面实测，**核心声称全部成立**；未发现阻断性缺陷。发现 2 处 message 文字与实测不符（均不触及代码正确性）与 1 处既有基线问题（非本批引入）。

**置信度：高（约 90%）**。变异与门禁均本机实跑复做，非读报审文字。

---

## 面1 门面最小性与零改动 —— 通过

**判据**：恰 3 纯转发 fn、无夹带实现；commit1 diff 恰 2 files；底层零改动。

**实测**：
- `git show 1ad29e6 --stat` = 恰 2 files：`crates/game-core/src/scene.rs` (+1)、`crates/game-core/src/scene_solver_api.rs` (+62)。
- `scene_solver_api.rs` 实读：文件前 61 行为 `#[cfg(test)]` 外的全部生产代码——恰 3 个 `#[inline] pub fn`（`parse_op` / `execute_op` / `execute_script_instruction`），每个都是单行转发到 `GameScene::…`，无任何实现逻辑夹带；其余 521 行为 `#[cfg(test)] mod tests`（11 个测试）。
- `scene.rs:57` 挂载单行：`#[doc(hidden)] #[path = "scene_solver_api.rs"] pub mod solver_api;`（+1 漂移，如实申报）。
- `git diff 5f48fe4..c2f2d84 -- crates/game-core/src/script_ops.rs crates/game-core/src/scene_script.rs` = **0 行**（底层 pub(super) 零改动实证）。

**结论**：成立。

## 面2 G5a/G5b 判别力独立复做 —— 通过（真锁，非假钉）

**判据**：自选真变异（不看贴件），任一突变不红=锁假钉=红。基线 11 passed，变异后逐一还原。

| 变异 | 结果 | 判 |
|---|---|---|
| ① 门面 `parse_op` 签名加 `_extra: bool` | 编译红：E0308 + E0061×5（G5a 签名锁抓到） | ✅ |
| ② fn#1 体 `GameScene::parse_op(line, script_cursor0 + 1)` | `g5b_parse_parity_fixture_all_lines` **红**，`g5a` **仍绿**（Y4 边界：G5a 只锁签名不锁实现，被证实）；另 2 个依赖 parse 的对拍也红（11 中 3 FAILED） | ✅ |
| ③ fn#2 体加 `scene.player.gold += 1;` | 仅 `g5b_execute_op_parity_direct_per_instruction` 红（10 passed 1 failed，判别力精确） | ✅ |
| ④ fn#2 空实现 | 同上测试红 | ✅ |
| ⑤ fn#3 体加 `gold+=1` | `g5b_execute_parity_fixture_all_lines` + 2 smoke 红 | ✅ |

还原后 `cargo test -p game-core --lib solver_api` = **11 passed**，`git diff` 空。

**结论**：G5a/G5b 是真锁。**判别力边界清晰**：G5a 签名锁不挡实现突变（设计如此，由 G5b 承担），G5b 按 fn 精确归因。

## 面3 R1 红线（side_b 零 solver_api / 零 GameScene） —— 通过

**判据**：提案 §5 条款 2（:115-123）违例点=0：(b) 侧不 import `solver_api`、不驱动 `GameScene` 解释器。

**实测（grep 全文 + 实读）**：
- `script_abs.rs` 中 `solver_api` 仅出现在 :187/:198（**side_a 模块内**，`use game_core::scene::{solver_api, GameScene}`）及 doc 注释行。
- side_b 模块（:227 起）：**零** `solver_api`、**零** `GameScene` 类型 import/调用（仅注释提及类名）。`side_b::drive(init: &Snapshot, table: &DialogueTable, idx: usize)` 只收 Snapshot/DialogueTable。
- `Snapshot::from_scene`（script_abs.rs:118）**只读 pub 字段**（`sc.player.*`、`sc.view.*`、`sc.persist.floor`、`sc.menu.*`、`sc.grid.*`、`sc.images.entity`），不调方法、不驱动解释器。
- `script.rs`（ScriptEngine 底层）无 `GameScene` use，仅 :172 注释提及（对照口径说明）。

**结论**：违例点=0，R1 成立。

## 面4 槽67 裁定 —— 通过（裁定真实，排除只在测试）

**实测**：
- `docs/spec/dead-code-rulings.md:170-200` 实读：DC-5 终裁 ☠️ A 级——`tryRunScene case67` 门控 `var3 &= this.difficultyIndex > 0`（a.java:8226），`difficultyIndex` 值域={0}（由存档往返引理 L1-L5 证）⇒ 恒 false ⇒ `loadLevelScript(67)` 不可达。
- **根因链独立验证**：fixture 原文末行 python 实测 `b'TAK_264_268 '`（带行尾空格、无 `\x1e`），共 68 行；`scene.rs:253-254` `from_fixtures` 的 `.map(|l| l.trim().to_string())` 吃掉尾空格 ⇒ `script_ops.rs:410` `parse_script_int(line, e1+1, ' ')` 分隔符缺失 ⇒ `scene_script.rs:8` `.expect("脚本整数分隔符缺失")` panic，**列号实测=44，与红档 scene_script.rs:8:44 吻合**。
- Java 侧对照：a.java:8613 `parseScriptInt` `indexOf`=-1 ⇒ `substring(var2,-1)` 同抛（panic parity 成立）；TAK 分隔符 a.java:8274-8276（起用 `_`、止用 ` `）。
- **排除范围**：`take(SLOT67)` / `0..SLOT67` 仅存在于测试（`scene_solver_api.rs:111/405/438`、`script_abs.rs:591`），生产代码零排除。panic-parity 断言两侧真同消息（`mf==md` 且 `contains("脚本整数分隔符缺失")`）。

**结论**：DC-5 排除合规，根因链 A 级闭环。

## 面5 对拍判据 :307-311 —— 通过（断言非恒真）

**逐条实跑**：
| 判据 | 实测 | 结果 |
|---|---|---|
| script_abs 测试数 ≥1（期望 6） | `--list \| grep -c script_abs` = **6** | ✅ |
| `cargo test -p game-solver --lib script_abs` | **6 passed** | ✅ |
| `grep -c 对拍 script_abs.rs` = 10 | **10** | ✅ |
| `cargo test --lib 对拍` | **1 passed**（0..SLOT67，排除 67） | ✅ |
| `gates check` FAIL=0 | **PASS 13 / FAIL 0 / SKIP 0** | ✅ |

**防恒真抽检**：对拍比较的是 `Obs` 全字段 derive PartialEq——`{cursor, index, flags[68], stats[8], facing, px, py, entities{count, stair_up, stair_down, items: Vec<Entity 含 11 字段>}}`，非只比 cursor。抽槽断言实读：RES 断言两侧 `stats==[400,10,10,0,0,0,4,0]`（与红档 verbatim 一致）；GIN 断言净增 1000 且两侧相等；DES 断言实体集已变且两侧相等。面2 变异③⑤红档证明这些断言真会失败——**非恒真**。

**结论**：判据全过，对拍有真实判别力。

## 面6 五闸独立重跑 —— 通过

- `python3 gates/cli.py check` → **PASS=13 FAIL=0 SKIP=0**（含 state-baseline GREEN、test-ratchet `total=1046(基线1029)`）。
- `python3 tools/state_baseline.py --check` → `[GREEN] 74 场景 sha 全同`（后台 job bash-5358f41fed764024，exit 0）。
- `python3 gates/cli.py trace` → exit 0，`diff-trace PASS`（74 场景 sha 全同）。
- **金样 74 零重冻**：本批 4 files 无一触及 baselines/金样。

## 面7 锚漂清单 —— 通过（漏报=0）

- python 比对 5f48fe4 scene.rs（4908 行）vs HEAD（4909 行）：挂载行 new:57；抽 20 个锚，`old:N == new:N+1` 全 match（60/252/253/282/411/419/3191/4624/4626/612/926/978/2045/3592/3739/3804/3820/3891/3894/4079）；N∈{43,44,55,56}（挂载点之前）行号不变。
- 提案 scene.rs 锚={43,55,60,252,282,3191,4624}；计划锚={252,612,926,978,2045,3592,3739,3804,3891,4079}。**与 1ad29e6 message 漂移清单差集=空，漏报=0**。
- Y3 +1 漂移已记入 1ad29e6 message（含逐锚清单）。

## 面8 夹带 —— 通过

`git diff 5f48fe4..c2f2d84 --stat` 全量过目 = **恰 4 files +1288**：
- `crates/game-core/src/scene.rs` +1
- `crates/game-core/src/scene_solver_api.rs` +583
- `crates/game-solver/src/lib.rs` +1
- `crates/game-solver/src/script_abs.rs` +703

无 game-data / reference / 计划 / 提案 / 台账 / 门禁改动。工作树净（仅既有 untracked，0 个 tracked 修改）。

## 面9 红档真实性 —— 通过

- `/tmp/red_final.txt`（4317B，2026-11-10 14:43:09）：6 tests **4 FAILED**（res/des/gin/对拍），panic 文本 `脚本整数分隔符缺失` @ `scene_script.rs:8:44`、RES `left[0..] right[400,10,10,0,0,0,4,0]`——与报审 verbatim 相符。
- `/tmp/green_final.txt`（795B，14:43:16）：**6 passed**。时间序红→绿（7 秒内）→ commit（14:53:54）合理。
- 红①（commit2 首跑槽67 panic）机理独立复现链成立：fixture 尾空格（python 实测）→ trim（scene.rs:254 实读）→ expect @8:44（列号 python 实测=44）→ Java 同抛（a.java:8613）。非构造性贴件。
- 面2 的变异红档为本次复核本机实跑（非贴件）。

## 面10 commit message 质量 —— 基本通过，2 处不符

- AGENTS §4 三问（问题在哪一层/消费者读哪份/证据级别）：三 commit 各 1 处 ✅。
- **Evidence 字段：三 commit 均无**（grep=0）——但本批不触及 `data/naming/`、`reference/`、`original/`，commit-msg 门禁不要求，**不构成违规**；报审问"含 Evidence"的字面答案=不含。
- Confidence: 95 在 `1ad29e6`/`c679ae3`；**`c2f2d84` 无 Confidence 行**（弱项，见红点 #2）。
- Y3 漂移量 +1 已记入 `1ad29e6` message ✅。
- `git rev-parse` 1ad29e6 / c679ae3 / c2f2d84 / origin/modern 全 OK（origin/modern=5f48fe4）✅。

---

## 红点清单（非阻断）

| # | 级别 | 不符点 | 影响 |
|---|---|---|---|
| 1 | 低（文字不符） | `c679ae3` message 写 `tools/dead_code_ruling.py --check exit 0`，实测 HEAD 与基线 5f48fe4 **均 exit=1**（根因=DC-8 `intro.rs:363 new_game_stats` 已被接线，输出"须复核 ['DC-8']"）。**系既有状态，非本批引入**；DC-1..DC-5 本体全成立 | 无代码影响；建议下次 message 措辞改为"DC-1..DC-5 成立（DC-8 既有恒红）" |
| 2 | 低（字段缺失） | `c2f2d84` message 缺 `Confidence:` 行（另两 commit 有 95） | 门禁不要求（不触及证据敏感路径）；建议补 |
| 3 | 信息（既有） | `dead_code_ruling --check` 恒 exit=1（DC-8）——基线即如此 | 与本批无关，不阻断 |

## 盲区

1. **未验证**报审"三 commit 各过五闸"的历史运行（只验证终态 c2f2d84 五闸全绿 + 三 commit 树各自可编译的间接证据）；commit1/commit2 时点的 gates 输出无法复现（工作树已前进）。
2. **红①**（commit2 首跑 WIP 红）为机理复现 + 时间戳佐证，非逐字重跑当时工作树状态。
3. Java 侧 panic parity 为静态对照（a.java:8613 代码路径），未在 JVM 实跑。
4. 对拍抽检覆盖 RES/GIN/DES 三槽 + Obs 全字段结构，未逐槽人工核对全部 67 槽断言内容（但变异③⑤证明测试网有真实判别力）。

## 复核方法声明

- 全程只读 + 临时变异（5 次），每次变异后还原并跑基线确认 11 passed；最终 `git diff` 空、`git status` 0 个 tracked 修改。
- 过程中曾误用 `git stash -u`，已立即 `git stash pop` 完整恢复（35 行 untracked 全在，stash list 空），无损。
- 本文件为本次复核唯一产出。
