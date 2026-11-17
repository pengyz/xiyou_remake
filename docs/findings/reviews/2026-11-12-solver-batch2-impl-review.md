# 对抗复核：求解器批2「单层求解 MVP」落码（TDD 红→绿 + 四裁定）

- 对象：分支 `modern` 上 9 笔**本地未推** commit，`167425b..ca09101`
- 权威：`docs/superpowers/proposals/2026-11-10-solver-batch2-proposal.md`（三轮复核已过）+ `docs/superpowers/plans/2026-11-09-solver-implementation-plan.md:316-335`
- 被审声称：成员报审「C1-C7 全切片 TDD 红→绿 + 四裁定段」
- 方法：**不采信报审**，全部主张以 `git rev-parse/show/diff`、`cargo test`、`cargo run`、`python3 gates/cli.py`、python 直读 `original/囧囧西游-大闹天宫.jar`（`maplv1`/`sprite1`）+ `data/tables/*` + `crates/game-core/tests/fixtures/*`、以及 **2 次变异抽测 + 2 次证伪探针**实测复现；行号均从实测输出计数
- 复核期临时改动（变异、探针测试、证伪测试）**已全部还原**：收尾 `git status --porcelain` 仅剩既有 untracked（`android/build/`、`docs/findings/reviews/2026-11-04~11-10-*.md`），`-- crates/ data/` 干净

## 总判：**黄**（红 0 · 黄 4 · 绿 12）→ 可放行落码，**推送前须先修 Y1**

**红项清单：（空）** —— 七条 §5 判据、计划 :327-328 四命令、gates 13 项、金样 74、变异抽测 2 次「改坏→恰 1 红→还原绿」全部实测通过；无行为缺陷、无门禁/保真面违规。

---

## 面1 commit 链与夹带 —— **绿**

- 9 hash `git rev-parse --verify ${h}^{commit}` 全 OK：`167425b`（docs 提案 Y3-1 小修，父）/ `ed8391d` C1 / `32f8e01` C2 / `9f73c43` C3 / `ba3577c` C4 / `c95fb38` C5 / `19ef786` C6 / `26abf24` C7 / `ca09101` ratchet。
- 每笔 `git show --stat` 与 message 逐条一致（正文均含红档 verbatim、实测锚、门禁结果）：

| commit | subject | 文件面 |
|---|---|---|
| `ed8391d` | 批2 C1 solve_local 骨架 + solve-local 子命令壳（TDD 红先行） | `src/bin/solver.rs` `src/lib.rs` `src/solve_local.rs` `tests/solve_local.rs`（4） |
| `32f8e01` | 批2 C2 3×3 夹具可达性 + 战斗 Refused | `solve_local.rs` + `tests`（2） |
| `9f73c43` | 批2 C3 目标双查询 max_hp/min_kills + Pareto 取解 | 同上（2） |
| `ba3577c` | 批2 C4 开门转移 + floor1 fixture 装载路径 | 同上（2） |
| `c95fb38` | 批2 C5 StepGate 脚本触发转移+终止边 | 同上（2） |
| `19ef786` | 批2 C6 floor1 端到端+拾取转移+确定性双跑 | `bin/solver.rs` + `solve_local.rs` + `tests`（3） |
| `26abf24` | 批2 C7 find_path/BFS 对拍+暗墙与胜利夹具 | `solve_local.rs` + `tests`（2） |
| `ca09101` | `chore(gates): ratchet-update 批2 新增测试入册（solve_local(test): 11）` | **仅** `data/status/baselines/rust-test-counts.json`（+4/−3：`solve_local(test)` 0→11、total 1029→1057、`game_solver(lib)` 0→6、`game_core(lib)` 377→388 归位） |

- **ratchet 笔独立不混功能** ✓（AGENTS §9：门禁/基线单独成 commit，`ca09101` 不含任何生产码）。
- **全链 diff 恰范围**：`git diff --name-only 167425b..ca09101` = 5 文件（`crates/game-solver/src/{bin/solver.rs,lib.rs,solve_local.rs}`、`crates/game-solver/tests/solve_local.rs`、`data/status/baselines/rust-test-counts.json`）。**零改保真面实测**：`crates/game-data/`、`crates/game-core/src/` 生产码 0 改动（C5「门面允许面」实际未用到）、`reference/` `original/` `data/naming/` `gates/` `tools/` 全 0。
- 夹带：无。`?? android/build/` 与 `docs/findings/reviews/2026-11-04~11-10-*.md` 为既有 untracked，未入任何 commit；本次复核只新增本报告文件。

## 面2 TDD 红档 —— **绿**（无「先绿后红」补档迹象）

`/tmp/red_c1..c7.txt` 全部存在且为**真红**（编译错实文 / 断言 FAILED 实文）：

| 档 | 实文（摘） | mtime vs commit |
|---|---|---|
| c1 | `E0432 unresolved import game_solver::solve_local` + `list=0` | 11-11 10:21 < 10:31 |
| c2 | `E0432 unresolved probe_bump/LevelSpec` + `E0560 SolveInput has no field level` | 11:03 < 13:02 |
| c3 | `E0609 no field route_min_kills` ×5 | 13:27 < 14:05 |
| c4 | `E0425 cannot find function load_floor` + `E0609 keys_used` ×2 | 14:40 < 14:57 |
| c5 | `E0432 unresolved probe_gate` + `E0609 LevelSpec has no field flags` ×5（EXIT=101） | 16:04 < 16:37 |
| c6 | 断言 FAILED「floor1 必须有解 left:false right:true」+「双跑输出须非平凡」0 passed 2 failed | 20:30 < 23:41 |
| c7 | `E0432 unresolved bfs_reachable, find_cell_path` | 11-12 03:26 < 05:42 |

七档 mtime 全部**早于**对应 commit，内容形态为「缺符号/缺字段编译错」与「首跑断言红」，与各自切片的首写测试对应 ⇒ 无事后补档、无先绿后红。

## 面3 裁定① stride 修正 —— **绿**

- 修正本体：`crates/game-solver/src/solve_local.rs:322` `let idx = (y * cols * 4 + x * 2) as usize;`（原 `(y*cols+x)*2` 已消除；`terrain[y*cols+x] = tile_walk[tile]==1`）。
- **三锚实文命中**：
  - `reference/src/deobf/a.java:6973` `private void rebuildWalkability()`：内层 `var1 += 2`（逐列）、外层 `var1 += var3 << 1`（`var3=mapCellsWide`）⇒ 行距 `4·wide`、列距 2；`walkableGrid[var5][var6] = f_bool_arr_04[var2]`。
  - `tools/route_gen.py:54-55`：`def wk(y, x):` + `t = g1[y * wide * 4 + x * 2]`（`g1 = m[4:4+wide*13*4]`，`wide = u16@0 >> 1`）。
  - `crates/game-core/src/scene_persist.rs:300` `let idx = (y * cols * 4 + x * 2) as i32;`，注释自带 A 级出处「oracle FLD 134（walkableGrid）13×13 逐格复算匹配」+「与 paintTileLayer 的行距 wide·2、列距 1 是同一数组的两套读法（原版共存）」。
- **`maplv.rs:193 cell_indexing_matches_engine_stride` 的「两套读法并存」解释成立**：该测试断言 `data[4+26*3+5]=42 ⇒ tile_at(5,3)=42`，走的是 **26y+x 行主序读法**（`maplv.rs:121` `terrain.get(row*cell_cols()+col)`，服务 paint/显示）；行走判定走 `y*wide*4+x*2`。两者读**同一段 676B 数组**、原版共存（`a.java` 两处读法分别对应 `isCellWalkable` 与 `paintTileLayer`），故 `load_floor` 用行走读法**不与** `maplv.rs` 的 tile_at 冲突；金样 74 走 `scene_persist` 正确式、不经 `load_floor`。
- **零改保真面** ✓（见面1）。
- 锚小漂移见 **Y3**（route_gen 引 :52 实为 :55）。

## 面4 裁定② §8-8 `min_kills=None` 首跑改判 —— **绿**（独立复算吻合）

- 提案 :263（`167425b` 修正后 verbatim）：「若 Exit#1 不可达则 `min_kills=None` 而 `reachable=true`、`max_hp=Some`（由 Exit#2 路给出，§2.1 不收域）；T6 形状断言 `min_kills:<int>` 此情形 None 变体首跑核对」。
- 实测 T6（`tests/solve_local.rs:681` 起）：`floor=1`、`reachable=true`、`max_hp=Some(300)`、**`min_kills=="null"`**、`confidence=="bounded-best"`、route 非空且逐航点过 `waypoint_ok`；`min_kills ≥ 3` 断言已删，`:713` 注「已按 §8-8 None 变体改判」⇒ **与预置口相符**（Exit#1 不可达 + max_hp 有界）。
- **独立复算（python 直读 `original/囧囧西游-大闹天宫.jar` maplv1/sprite1，未用被审代码）**：
  - 地形 `g1[y*wide*4+x*2]` + `tile_walkability.txt`；实体 35 条（7 门 / 5 钥 / 41×4 43×3 42·44·45·46 各1 / 83×3 @`(6,9)`×2`(6,11)` / 7×1 @`(11,1)` / 15×1 @`(10,11)`）；怪按 `loss≥300 ⇒ 挡`、门耗钥。
  - `block_gate=True`（踩 type83 即视为触发 script19 换层、不可继续）：**仅 6 格（口袋），exit(11,1) 不可达**；
  - `block_gate=False`：36 格（强制钥）/ 51 格（不限钥）**均可达** ⇒ **绑定约束 = gate 而非钥经济**，与报审理由「口袋唯一出口 = `door@(6,9)`，踩格必触发 script19 换层边」**独立吻合**；提案 §3.2-5「0-1 BFS = 门全开口径」自认口径亦一致。
- 临时探针（已删除）进一步实测：`find_cell_path start→(11,1)=true`、`bfs_reachable=true`（subgraph 近似不挡）、`solve_local` 仍 `reachable=true / max_hp=Some(300) / min_kills=None / route=Path B["6,11","5,11","5,10","6,10","door 6,9","gate 6,9"]` ⇒ 即便子图过近似放行，`classify`/门/gate 面仍正确挡住，结论稳。

## 面5 裁定③ route_gen gate token 放行 —— **绿**（未越权；带 Y4(a) 遗留动作）

- 实文：`tools/route_gen.py:12-15` 只有三类语法 `x,y` / `bump 型@x,y` / `door x,y`，**无 `gate`**；计划 :322 同列三类。
- 放行点在**实现方自写测试** `waypoint_ok`（`tests/solve_local.rs:659`）：对 `gate ` 前缀 `return true`，并**在原地注明**「三类语法无此 token、提案 C6 判据歧义面、报审注明、该航点不产生移动、喂 route_gen 时剥除即可复现」。
- **A6 只约束三类语法的解释合理**：提案 :168「A6 oracle 回放…**批2 不做；但 T6 的 route 必须能喂 `route_gen.py`（A6 前置）**」、:264「A6/A11 留批5」⇒ 批2 义务 = route 可喂（gate 航点无移动、剥除后即三类语法），**不是**把 gate 写进 route_gen 语法。判据放宽发生在测试侧、非 `gates/`、且显式报审 ⇒ **不构成越权改判据**。

## 面6 裁定④ T8 type72 排除 —— **绿**（提案误标修正，证据级 A/B 足）

- `data/tables/sprite_defs.json`：`type72 → render_category=32`，且是**唯一非空 `floor_category_override`**（`72 → [{floor:50, category:8}]`）。
- A 级锚实文：`reference/src/deobf/a.java:9679-9681` `if (var1 != 50) { this.renderCategoryByType[72] = 32; } else { this.renderCategoryByType[72] = 8; }` ⇒ floor1（≠50）恒 cat32。
- 联动门证据：`crates/game-core/src/scene.rs:3663` `32 => allowed = false, // 联动门（a.java:5520）`，位于 `interact_with_cell`（`:3369` 起，`try_step` `:3315-3335` 调入）的 entity_category 分支；同分支 `8 => 怪物 resolve_bump`（`:3588`）、`1 => match t{1|2|3 门耗钥, 9|10|11|12|81 阻挡, 6|7|8 楼梯}` ⇒ cat32 连 bump 战斗都进不去，**type72 撞前不可达**成立（`tests:825` 断言 + `classify` 32 臂 `blocked` 与 `scene.rs` 同构）。
- 提案 §1.3:61「A 类换层（型69..75）」误标已核：`scene.rs:3804-3820`（73/74/75 → `load_level_script(22/21/20)`）、`:3882-3895`（69@35→script36、70|71@49→35、72@50→65）全是 **`load_level_script` 非 `change_floor`**；`level_scripts.txt` 第20/21/22 行体只含 `TAK/GUT`（无 `CHANG`/换层 op）⇒「击杀 69..75 = 载脚本对话、非换层终止边」成立，T8 的「无出口不可达」判别断言方向正确。
- **证据级够**（数据 + A 级 Java 实文 + 引擎同构臂三重）；锚漂移见 **Y3**（`:5520` 实为 `:5527`）。

## 面7 验收对照：提案 §5 C1-C7 逐条 + 计划 :327-328 四命令 —— **绿**（带 Y2）

| 切片 | 判据（提案 §5） | 实测 |
|---|---|---|
| C1 | `cargo test -p game-solver -- --list \| grep -c solve_local ≥1`；`gates check` FAIL=0 | `grep -c = 9 ≥1` ✓；gates 13 项 FAIL=0 ✓ |
| C2 | T2 绿（能杀/杀不过两例）；A9 纯度 grep 0 命中 | `p2s1_t2a/t2b` ok ✓；A9 grep = `A9_CLEAN` ✓ |
| C3 | T3 三断言 (i)(ii)(iii) 绿；并列 tie-break 有断言 | `p2s1_t3…` ok；(i) 单调性、(ii) 双查询、(iii) `route==["2,1","1,1","1,0"]`（BFS 首见上路、可判别）✓ |
| C4 | T4 绿；T5 census 快照绿（floor1 7 门 5 钥进 fixture） | T4 有钥 `keys_used==(1,0,0)` route `["0,0","door 1,0","2,0"]`、无钥 `!reachable` + `(0,0,0)` ✓；T5 census 7 门/5 钥/`type15×1`/`type12=0`/`type6=0`/`type7@(11,1)`/`type83×3 params{19,66,67}`/`exits=[(11,1) Stair idx1]` ✓ |
| C5 | T9 绿；slot19 触发→终止边；66/67 恒不触发 | T9 (a) flags0→probe 真+reachable+`max_hp=Some(300)`+`min_kills=None`+route `["6,10","6,9","gate 6,9"]`；(b) `flags[19]` 置位不可重入；(c) case66 恒不触发；(d) case67 difficulty=0 不触发 / =1 触发 ✓ |
| C6 | 计划判据 :327-328 四条全绿（含 `gates check` :331）；route 逐航点过 route_gen 语法 | 四命令见下表 ✓；`waypoint_ok` 见裁定③ |
| C7 | T7/T8 绿；A7 金样绿、A8 门禁绿、A9 纯度绿、A10 对拍不回退 | T7 全格对拍（`find_cell_path` vs `bfs_reachable`）ok；T8 暗墙/联动门/胜利夹具 ok；金样 74 `[GREEN]`、gates `PASS=13`、A9 `A9_CLEAN`、`cargo tree -i game-solver` 仅自身（无反向依赖）✓ |

**计划 :327-328 四命令实跑**（两次独立 run，输出一致）：

```text
$ cargo run --release -p game-solver --bin solver -- solve-local --floor 1
{"floor":1,"difficulty":0,"s0":{"hp":300,"atk":10,"def":10,"yellow":0,"blue":0,"red":0,"gold":0},"reachable":true,"max_hp":300,"min_kills":null,"confidence":"bounded-best","route":["6,11","5,11","5,10","6,10","door 6,9","gate 6,9"]}
$ cargo test -p game-solver -- --list | grep -c solve_local   # → 9（≥1 ✓）
$ cargo test -p game-solver solve_local                        # → 9 passed; 2 filtered out（t4/t5 被过滤，见 Y2）
$ python3 gates/cli.py check                                   # → PASS=13 FAIL=0 SKIP=0
```

## 面8 判据独立重跑 —— **绿**

- `python3 gates/cli.py check` = **PASS=13 FAIL=0 SKIP=0**（两次独立运行一致；含 `test-ratchet total=1057`、`state-baseline [GREEN]`）。
- 金样独立跑 = **`[GREEN] 74`**（sha 全同）。
- `cargo test -p game-solver p2s` = **11 passed; 0 failed**（含被计划过滤器漏掉的 t4/t5）。
- A9 纯度：`grep -rn "game_core::" crates/game-solver/src --include=*.rs`（排除 `bin/solver.rs`）= **0 命中（A9_CLEAN）**；`cargo tree -i game-solver` 仅自身。
- 输出确定性：两次 `solve-local --floor 1` **逐字节相同（DIFF_SAME）**。

## 面9 变异抽测（≥2）—— **绿**（均「改坏 → 恰 1 红 → 还原绿」）

| # | 目标 | 变异 | 结果 |
|---|---|---|---|
| M1 | C3 tie-break（实现侧） | `HpLoss::better` 并列分支 `a.route < b.route` → `a.route > b.route` | `cargo test -p game-solver solve_local` **恰 1 红**：`p2s1_t3_solve_local_目标双查询单调性与tie_break` FAILED `tests:356`「hp/kills/steps 并列 ⇒ 字典序最小路线（BFS 首见为上路 2,0，断言可判别）」，其余 8 passed；还原 → 绿 |
| M2 | C4 钥经济（实现侧） | `expand` 的 `Cell::Door` 臂 `if consumed {` → `if true {`（绕过 `consume_key_for_door`） | **恰 1 红**：`p2s1_t4_开门有钥则开无钥则挡耗钥断言` FAILED `tests:432`「无钥 ⇒ 挡」，实测 `reachable=true route=[0,0,door 1,0,2,0] keys_used=(0,0,0)`；还原 → `p2s*` 11 passed |

> 注：M2 恰 1 红是用 `p2s*` 过滤器观察到的；按计划 :330 的 `solve_local` 过滤器**跑不到 t4**（见 Y2），若只信计划过滤器会误判「变异无红」。

## 面10 证伪探针（type75）—— 见 **Y1**

`tests/solve_local.rs:826` 注「type75 缺敌表行避开」、`:828` `for ty in [69,70,71,73,74]`。复核把循环临时改为含 75 → `cargo test -p game-solver t8` = **1 passed ok**（含 `:834` `route 含 bump 75@` 断言 ⇒ `enemies_for_type(75)` 确有敌表行、非空跑）。已还原。

---

## 黄项清单（推送前/后续动作）

- **Y1（须修，推送前）｜「type75 缺敌表行」证据不实**：commit `26abf24` 正文与 `crates/game-solver/tests/solve_local.rs:826` 注均称 type75 因「缺敌表行」被排除——实测证伪：`data/tables/enemies.json` **含 id75**（hp100 atk65 def15 gold30，evidence `a.java:1044-1130`）；`crates/game-core/tests/fixtures/enemybasehp.txt` 等 4 文件各 35 词 = id 41..75；`combat.rs:24-35` `to_records` 用 `id: 41 + i` ⇒ 41..75 全有行；探针含 75 即绿。**影响**：型75 无覆盖的理由不存在，与计划 :320「型69..75 胜利触发的 A 类转移」范围不齐（行为本身无缺陷，75 是可杀的普通怪）。**动作**：更正 `:826` 注释并把 75 补进循环（一行）；`26abf24` 正文不可改，在下一笔 commit 正文按 AGENTS §3.6 注明证伪更正。
- **Y2（判据覆盖观察）｜计划过滤器只跑 9/11**：计划 :330 `cargo test -p game-solver solve_local` 的名字过滤器**不含** t4 `p2s1_t4_开门有钥则开无钥则挡耗钥断言`、t5 `p2s1_t5_floor1_fixture_census快照` ⇒ 按计划字面执行 C4 判据（T4/T5 绿）时这两条**不会被运行**。本次复核已用 `p2s*`（11 passed）补跑，二者均绿，故非违规；但后续批次应统一测试命名或改过滤器，否则「判据实跑」存在系统性漏跑面。
- **Y3（锚行号漂移 3 处，按证据纪律应正）**：注释/正文引 `route_gen.py:52` 实为 `:54-55`（`def wk` :54、`t = g1[y*wide*4+x*2]` :55）；引 `scene_persist.rs:301` 实为 `:300`；引 `a.java:5520`（case32）实为 `a.java:5527`（`:5520` 是同一 switch 内 case1 的 `showOverlayMessage` 行，**同块不跨块、语义不误**，仅行号偏 7）。均为注释级，不影响判据。
- **Y4（名实不符 + 批5 前置动作）**：(a) T6 测试/断言名仍叫「`min_kills` 下界」而断言已是 `min_kills==null`（§8-8 None 变体）——cosmetic，建议改名以免误导后续读码；(b) `waypoint_ok` 对 `gate ` 的放行是测试侧、已报审，但 `route_gen.py` 三类语法确实无 `gate`，**批5 A6 回放前**须落地「喂 route_gen 前剥除 gate 航点」的一行规则（提案 §6-10 自列为待裁定），不要拖到批5 才发现。

## 结论

- **总判：黄**——**红项 0**。commit 链、TDD 红档、四裁定（①stride ②§8-8 None ③gate token ④type72）、§5 七条判据、计划四命令、gates 13 项、金样 74、A9 纯度、2 次变异抽测、夹带检查**全部实测通过**；零改保真面（game-data/game-core 生产码、reference/original/gates/naming 全 0 改动）成立。
- **放行条件**：先修 **Y1**（更正 type75 证伪注释 + 补 75 覆盖），Y2/Y3/Y4 可随同笔或下一笔顺手处理；全部通过后按任务纪律**再推送**。
