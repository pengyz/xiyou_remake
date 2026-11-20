# e0322c9 复核：P3 红1/红2 修复 + 三黄补口（独立对抗复核）

- **复核对象**：`e0322c9`（父 `7a8f29e`，`modern` 分支，未 push）
- **上一轮结论**：`docs/findings/reviews/2026-11-18-exit-solve-p3-review.md`（2 红 4 黄）
- **本次规矩**：实现者自述不可信；测试全绿不算判据；每条主张自行复现（变异只在临时副本）
- **副本**：`/tmp/fixrev-A7K`（`rsync` 排除 `target/.git/analysis/android/save`），
  `CARGO_TARGET_DIR=/tmp/fixrev-target`；pristine 备份 `/tmp/fixrev-pristine/`
  （`solve_local.rs` md5 `d6d2dd9c9578a1fe7d5c9ecd397e8ac1`、`tests/solve_local.rs` md5
  `6cdb7c2792cf018f02a62fab24c49038`）；父提交同名文件 `/tmp/fixrev-old/`
- **原仓库零改动**：全部变异/探针只在副本；收尾 `git status --short` 仅两条既有未跟踪复核报告
  （见 §7）

## 1. 判定摘要

| 面 | 判定 | 一句话证据 |
|---|---|---|
| R1 红1（`flags_in` 零判据） | **绿** | 变异 M1（`flags_in`→`u128::default()`）只红 `exit_solve_honors_injected_flags_only` 且红在腿2（`solve_local.rs:1304`）；腿3 另有独立有效性：把实现改成「注入 0 时回退读 `level.flags`」→ 红在腿3（`:1311`） |
| R2 红2（`ok`×`truncated`） | **黄** | `ExitSolve` 面自洽且不可绕过（`SOLVER_LAYER_LIMIT=10` ⇒ `truncated=true && ok=false`；M4b 两测试红）；但**同语义旧入口 `solve_local()` 仍无任何截断信号**，`limit=10` 与 `limit=1e6` 输出逐字段相同（`reachable=true / max_hp=202 / bounded-best`）⇒ 红2 是「面级」修复，不是「不变量级」修复 |
| R3 `LAYER_MEMO` 键 | **黄** | `floor` 与 `node_limit` 确在键内、同线程 10→默认可复现；但键**不含 `shops`**，A 级串味实测：第二次传 `shops=&[]` 仍返回买臂解（`gold=50/yellow=5`，`identical_goals=true`），而提交自述「也保证键=全部输入」（`solve_local.rs:512`）不成立 |
| R4 env 注入污染 | **黄（低）** | `SOLVER_LAYER_LIMIT=10 cargo test --test solve_local` = 19 passed / **5 failed**；父提交同环境 18 passed / 3 failed ⇒ 脆弱性非本次新引入，但新增的两条判据同样未隔离；`solve_local_trunc` 独立二进制的隔离设计**正确** |
| R5 三黄补口 | **黄** | F4 绿（M4 变异必红）、`ents_in` 绿（M3 变异必红）；**Y4 注释 4 条子claim里 `69` 一条是错的**：脚本 36 实际被 `post_victory_event_of(69,35)` 隐式钉在 `floor==35`，注释写「无楼层条件」 |
| R6 有无放宽既有断言 | **绿** | 删除行仅 7 行，全部是签名迁移 `&[]`→`None` / `&a.out_ents`→`Some(&a.out_ents)`，无一条断言被删或放松 |
| R7 门禁与基线 | **绿** | 实跑 `PASS=13 FAIL=0 SKIP=0`；`test-ratchet — total=1154(基线1154) 无回退`（无「新增二进制」告警 ⇒ 实跑二进制集 == 基线 76 个）；`state-baseline — 74 场景 sha 全同` |
| R8 并发/夹带/reflog | **绿** | `git diff --name-status 7a8f29e..e0322c9` 恰 5 个文件（含 1 新增）；`e0322c9` 非 amend；`HEAD@{1}` 的 amend **只改 message**（tree 与 `abcd026` 相同）⇒ 上轮 F9「message 被覆盖」已修复；无「变形球/570 帧」内容夹带 |

**计数：0 红 / 4 黄 / 4 绿。**

## 2. 逐面详证

### R1 红1 是否真被守住 — 绿

判据位置：`crates/game-solver/tests/solve_local.rs:1292`（腿1）/`:1303`（腿2）/`:1310`（腿3）。

变异 M1（副本内，仅改 `crates/game-solver/src/solve_local.rs:456` 的 `flags_in` → `u128::default()`）：

```
failures:
thread 'exit_solve_honors_injected_flags_only' panicked at crates/game-solver/tests/solve_local.rs:1304:5:
flags_in=bit19 ⇒ 门被挡、且无出口 ⇒ 必须 !ok：ExitSolve { ok: true, exit: Some((11, 1)), ... route: ["6,10", "6,9", "gate 6,9"], out_hp: 300, ... }
failures:
    exit_solve_honors_injected_flags_only
test result: FAILED. 23 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

⇒ 红1 的原始症状（忽略 `flags_in` 全绿）已消失：忽略即红，且**恰好一条**、**红在腿2**。

**腿3 的独立有效性**（这是本次唯一需要额外攻击的点，因为「三腿」里腿3 最像装饰）：
把实现改成「注入 0 时回退读 `input.level.flags`」（模拟「优先读 level.flags」的实现错误）：

```
thread 'exit_solve_honors_injected_flags_only' panicked at crates/game-solver/tests/solve_local.rs:1311:5:
level.flags 不参与判定（契约=只认 flags_in）：ExitSolve { ok: false, exit: None, ..., route: [], out_flags: 0, ... }
test result: FAILED. 23 passed; 1 failed
```

⇒ 腿3 **会红**，不是无效腿。三条腿各自钉一条性质：腿1=默认能出层（夹具正向控制）、腿2=注入生效、
腿3=只认注入不认 `level.flags`。若只有腿1/腿2，`flags_in | level.flags` 这类实现可以全绿——腿3 是必要的。

### R2 红2 契约是否自洽且不可绕过 — 黄

**(a) 契约面实测（绿的部分）**：`crates/game-solver/tests/solve_local_trunc.rs` 在真数据 f1 上：

```
[红2] f1 limit=10 => ExitSolve { ok: false, exit: Some((11, 1)), out_hp: 202, truncated: true,
  route: ["6,11","6,10","5,10","6,10","door 6,9","gate 6,9"], kind: Some(Script), cross_edge: Some((2,false)) }
```

实现 `solve_local.rs:462-464`：`Some(g) if reachable => { ok: !truncated, ... }`，失败臂
`ExitSolve::failed` 恒 `ok=false` ⇒ 有效契约 `ok = 有目标 && reachable && !truncated`。
变异 M4b（`ok: !truncated` → `ok: true`）⇒ 2/2 红（`:159`「ok 必须 false」、`:188`「截断 ⇒ !ok」）。

**(b) 绕过面（黄的原因）**：
1. **旧入口 `solve_local()`（`solve_local.rs:257-310`）与 `SolveResult`（`:189-210`）没有 `truncated` 字段**，
   也不读 `take_layer_trunc()`。副本内探针（`limit` 从 1e6 递减）实测：

```
[probe legacy] limit=1000000 => reachable=true max_hp=Some(202) min_kills=None confidence=bounded-best route_len=6
[probe legacy] limit=200     => reachable=true max_hp=Some(202) min_kills=None confidence=bounded-best route_len=6
[probe legacy] limit=10      => reachable=true max_hp=Some(202) min_kills=None confidence=bounded-best route_len=6
[probe legacy] limit=1       => reachable=false max_hp=None     min_kills=None confidence=bounded-best route_len=0
```

`limit=10` 与 `limit=1e6` **逐字段相同**。即「截断但报可行」在旧入口**原封不动**存在，
且这正是上一轮红2 描述的危害形态（把截断的 202 当出层最大血）。
`grep -rn solve_local_exit` 显示当前消费者只有测试（`ExitSolve` 尚无仓内下游），
所以这是**潜伏**而非**在发生的**误用——按「P4 只接 `ExitSolve`」的口径可以接受，但它意味着
红2 的修复**没有变成不变量**（`Search::run()`/`solve_layer` 返回值里也没有截断位，
只有 thread-local `LAYER_TRUNC` 可观测）。

2. **半成品回填**：`truncated=true` 时 `out_player/out_flags/out_ents/out_hp/route/kind` 仍全量回填
   （`solve_local.rs:466-474`），只靠文档（`:372-374` 与 `:461-463` 注释「`out_*` 只作参考」）
   约束消费方；类型层面没有强制（`out_hp: i32` 而非 `Option`）。现存下游
   （`solve_global`、CLI）**不消费 `ExitSolve`**，故当前无误用；风险在 P4。

3. **全局路径**：`solve_global.rs:502-511` 直接调 `solve_layer_with`，`take_layer_trunc()` 收
   `truncated_layers`（`:503-507`），但**截断层的 goals 仍被 push 进前沿**（`:508` 之后没有
   `continue`）。它靠 `coverage.truncated`/`blocked.reason="truncated"`（`:613-650`）+ 置信降级
   `bounded-best`（`:623-629`）暴露，CLI 控制台行本身不带截断标记
   （`crates/game-solver/src/query.rs:74-82`），只有 `--out` JSON 里才看得到
   （`solve_global.rs:1126` 的 `coverage.truncated`）。这是**既有面、本次未声明修**，记为观察。

### R3 `LAYER_MEMO` 键 — 黄

键的完整构成（`crates/game-solver/src/solve_local.rs:515-523` + 类型 `:584-587`）：

```
(input.floor, entry, player.clone(), flags, root_ents.clone(),
 level_digest(input), policy, node_limit)
```

- `floor` **在**键内 ⇒ 不同层不会串（这一条上一轮担心的「新引入 bug」不成立）；
- `node_limit` 确已加入，`Search::run_root` 改为接收参数（`:1936`），memo 键与搜索界同源；
- **跨线程/跨进程隔离**：`thread_local!`（`:584`）由类型保证，另加 `level_digest`
  （`:643-693`，覆盖 terrain/start/entities/exits/player/enemies/flags）防「同 floor 不同夹具」。

**同线程 10→默认**复现（A 级，`solve_local_trunc.rs:191-193`）：

```
[红2-memo] second(default) => ExitSolve { ok: true, ... truncated: false }
```
变异 M5（键里的 `node_limit` → `0usize`）⇒ 只红 `layer_memo_key_includes_node_limit`
（`:193`「默认界下必须重算且不截断」，红态 `second.truncated=true`）。⇒ 该判据有效。

**但是键缺 `shops`**，而 `solve_layer_with` 的 `shops` 是实参（`:489`、注入 `search.shops`
`:535`），且 `solve_layer`= `shops=&[]`、`solve_global`= 真商品表 ⇒ 同一线程两种调用可撞同一键。
副本内探针（8×3 夹具，`ty78 param=(2<<8)|4` 商店 + 上行梯；先用真表、再用空表，**同线程**）：

```
[probe shops] with_goods  reachable=true ngoals=2
   with: hp=300 gold=100 yellow=0 steps=7
   with: hp=300 gold=50  yellow=5 steps=7      ← 买臂解（黄钥匙+5、金币-50）
[probe shops] no_goods   reachable=true ngoals=2
   w/o : hp=300 gold=100 yellow=0 steps=7
   w/o : hp=300 gold=50  yellow=5 steps=7      ← 空表却仍出买臂解 = 命中缓存回放
[probe shops] identical_goals=true
```

⇒ `LAYER_MEMO` 键**不等于全部输入**，`solve_local.rs:512` 的
「…也保证键=全部输入」是错的自述。该缺口**是既有的**（父提交键里同样没有 `shops`，
`/tmp/fixrev-old/solve_local.rs` 的 `memo_key` 元组为 7 元），本次只是**新增了一句错注释**，
故记黄而非红；但 P4 若在同线程混用「出层接口（空表）」与「全局/查询（真表）」，会静默串味。

### R4 环境变量注入是否污染并行测试 — 黄（低）

```
$ SOLVER_LAYER_LIMIT=10 cargo test -p game-solver --release --test solve_local
test result: FAILED. 19 passed; 5 failed
   drop_entity_pickup_credits_inventory        (:1094)
   exit_solve_honors_injected_flags_only       (:1293 腿1 assert!(a.ok))
   exit_solve_maximizes_hp_on_real_data        (:1264 !truncated)
   exit_solve_prefers_hp_over_steps            (:1356 assert!(got.ok))
   p2s1_t3_solve_local_目标双查询单调性与tie_break (:324)
```

父提交 `7a8f29e` 同环境（21 条）对照：`18 passed; 3 failed`
（`p2s1_t3 :324`、`drop_entity_pickup_credits_inventory :1094`、
`exit_solve_maximizes_hp_on_real_data :1273`）。

⇒ 结论分两层：(a) `solve_local` 这个二进制**确实**对环境敏感，f1 一族的「满界」断言（含新增两条）
会被 `SOLVER_LAYER_LIMIT` 影响——按任务口径记**黄**；(b) 但这不是本次引入的（3→5，新增的 5 条红里
2 条是新测试），且新的 `solve_local_trunc.rs` **独立二进制 + `ENV_LOCK` + `LayerLimitEnv::Drop`
恢复**的隔离设计是对的：默认 `cargo test --workspace`（不设 env）下 24+2 全绿，互不污染。

### R5 三条黄（F4 / ents_in / Y4 注释） — 黄

- **F4（判据绿）**：`HpLoss::better` 改成 steps 优先 ⇒ 只红 `exit_solve_prefers_hp_over_steps`
  （`tests/solve_local.rs:1357`「必须选血多的下路」，红态 `out_hp=276`
  `route=["0,1","bump 41@1,1","2,1","3,1","4,1"]`）。⇒ 双路线夹具确实能证伪「步数优先」。
- **ents_in（判据绿）**：把 `Some(&[])` 重新并入 sprite ⇒ 只红
  `exit_solve_entities_none_is_sprite_some_empty_is_cleared`（`:1395`「无怪 ⇒ 出层血=入场血」，
  红态 `out_hp=276`、route 含 `bump 41@2,1`）。⇒ `None`/`Some(&[])` 语义确实分开。
- **Y4 注释（黄）**：逐条对代码核（`crates/game-core/src/scene.rs`）：
  | 注释（`scene.rs:3758-3771`） | 代码 | 结论 |
  |---|---|---|
  | `70｜71` 共臂、共用 35、仅 `floor==49` | `:3780-3784` `70 ｜ 71 => if floor == 49 { load_level_script(35) }` | ✅ |
  | `72` 仅 `floor==50` ⇒ 65 | `:3785-3789` | ✅ |
  | `73/74/75` 不在本 switch，在击杀钩子（22/21/20，仅 `floor<=50`） | `:3683-3698` `if floor <= 50 { match t { 73 => {script_line_flags[cur]=true; 22} 74 => 21, 75 => try_run_scene(20,true)·20 } }` | ✅（精确行号 `:3683`，注释写 `3650+`，属宽松下界） |
  | `69`：本臂，`load_level_script(36)` **无楼层条件** | `:3755-3774` 脚本 36 在 `if let Some(ev) = post_victory_event_of(69, floor)` **之内**；`rules::boss::post_victory_event_of`（`crates/game-core/src/rules/boss.rs:155-161`）仅 `69 && floor==35` 返回 `Some(7)`，其单测 `:230-235` 明确 `post_victory_event_of(69,36) == None` | ❌ **与事实不符** |

  同一段注释的末尾自己写着「`69@35` → 组7」，与「无楼层条件」自相矛盾。材料性在于注释的目的正是
  「未来建『型 → (post-anim 组, 是否 post-anim 臂, 脚本号, **楼层条件**)』表」——
  按现注释建表会把脚本 36 的 `floor==35` 条件丢掉，把 0 层条件写成 0 个。

### R6 有无顺手放宽别的断言 — 绿

`git diff 7a8f29e..e0322c9 -- crates/game-solver/tests/solve_local.rs` 的**全部**删除行（7 行）：

```
-    let a = solve_local_exit(&input, (0, 1), &hi, 0, &[]);
-    let b = solve_local_exit(&input, (0, 1), &lo, 0, &[]);
-        std::thread::spawn(move || solve_local_exit(&input, (0, 1), &p, 0, &[]))
-    let a = solve_local_exit(&a_in, (0, 1), &p, 0, &[]);
-    let b = solve_local_exit(&b_in, (0, 1), &a.out_player, a.out_flags, &a.out_ents);
-    let naive = solve_local_exit(&b_in, (0, 1), &a.out_player, a.out_flags, &[]);
-    let got = solve_local_exit(&input, entry, &level.player, level.flags, &[]);
```

对应新增行只把最后一个实参换成 `None` / `Some(&a.out_ents)`，参数其余部分（含 `level.flags`）逐字相同；
**没有任何 `assert`/阈值/夹具被删除或放松**。`+139` 行里除 7 行迁移外是三个新测试与它们的文档。

### R7 门禁与基线 — 绿

```
$ python3 gates/cli.py check
[✓ PASS] rust-tests — cargo test --release 全绿
[✓ PASS] test-ratchet — total=1154(基线1154) 无回退
[✓ PASS] state-machine-anchors — 锚 6 + 边数对照；全命中
[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同
== 汇总 ==
PASS=13 FAIL=0 SKIP=0
```

- ratchet：基线 JSON 自身算术核对 `7a8f29e` binaries=75/total=1149 ⇒ `e0322c9` binaries=76/total=1154，
  `solve_local(test) 21→24`、新增 `solve_local_trunc(test)=2`（+5 与 +1154-1149 一致，无删测试）；
  门禁实跑 `total=1154` 与基线相等，且**未打印「新增二进制待 ratchet-update」** ⇒ 实跑二进制集
  恰为 76 个，与基线一致。
- 我另在副本实跑 `solve_local` 24 / `solve_local_trunc` 2 全绿，与自述数字相符。
- 74 金样 sha 由 `gate_state_baseline` 自报 GREEN；commit 只动 5 个文件、无一 golden/baseline
  除 `rust-test-counts.json`，与「sha 全同」一致（**未**逐文件手算 74 个 sha，见 §5）。

### R8 并发污染/夹带/reflog — 绿

```
$ git diff 7a8f29e..e0322c9 --name-status
M crates/game-core/src/scene.rs
M crates/game-solver/src/solve_local.rs
M crates/game-solver/tests/solve_local.rs
A crates/game-solver/tests/solve_local_trunc.rs
M data/status/baselines/rust-test-counts.json
```

恰为自述的 5 个文件；`scene.rs` 的 12+/3- 全是注释（Y4 段），无逻辑夹带。

reflog 近 12 条：

```
e0322c9 HEAD@{0}: commit: test(solver): P3 复核红1/红2 判据 + 出层成败契约改判 + 文档失真更正
7a8f29e HEAD@{1}: commit (amend): feat(solver): 单层「入场→出层」可续跑接口 solve_local_exit（导出出层态）
abcd026 HEAD@{2}: commit (amend): test(coverage): 变形球机制窗收窄到 570 帧，补 0px 机器指针，窗外分歧立探针
b1c5655 HEAD@{3}: commit: feat(solver): 单层「入场→出层」可续跑接口 solve_local_exit（导出出层态）
```

- `e0322c9` 不是 amend；
- `HEAD@{1}` 的 amend **只改 message**：`git rev-parse 7a8f29e^{tree}` == `abcd026^{tree}`
  （`412bf9f8…`），且 `7a8f29e` 的 message 已回到与内容相符的 `feat(solver): 单层…`；
  `git diff --stat b1c5655 7a8f29e` 只差 `data/status/baselines/rust-test-counts.json` 2 行。
  ⇒ 上轮 F9「amend 把 message 覆盖成『变形球/570 帧』」**已修复**，且**没有**把「变形球」内容
  夹带进来（`git diff --stat 7a8f29e^ 7a8f29e` 只含 solve_local 源/测与 ratchet，3 文件）。
- 工作树：`git status --short` 仅两条既有未跟踪复核报告；近 30 分钟 `crates/**/*.rs` 无新 mtime
  ⇒ 并行 agent 本时段未触碰求解器 crate。

## 3. 变异实证表（全部在 `/tmp/fixrev-A7K`，改后用 pristine 覆盖还原）

| # | 改了什么（文件:行） | 预期 | 实测哪条红 | 红在哪（断言/输出） | 还原证据 |
|---|---|---|---|---|---|
| M1 | `solve_local.rs:456` `flags_in` → `u128::default()` | 红1 腿2 | `exit_solve_honors_injected_flags_only`（1 红/23 绿） | `tests/solve_local.rs:1304`「门被挡…必须 !ok」，红态 `ok=true route=["6,10","6,9","gate 6,9"]` | md5 回 `d6d2dd9c…` |
| M1b | 同处改为「`flags_in==0` 时回退 `input.level.flags`」 | 红1 腿3 | 同上（1 红） | `:1311`「level.flags 不参与判定」，红态 `ok=false route=[]` | md5 回 `d6d2dd9c…` |
| M2 | `HpLoss::better` 改 steps 优先 | F4 | `exit_solve_prefers_hp_over_steps`（1 红） | `:1357`，红态 `out_hp=276`、route 含 `bump 41@1,1` | md5 回 `d6d2dd9c…` |
| M3 | `solve_local.rs:456` 把 `Some(&[])` 并回 `None`（sprite） | ents_in | `exit_solve_entities_none_is_sprite_some_empty_is_cleared`（1 红） | `:1395`「无怪 ⇒ 出层血=入场血」，红态 `out_hp=276` | md5 回 `d6d2dd9c…` |
| M4 | `solve_local.rs:464` `ok: !truncated` → `ok: true` | 红2-A | `exit_solve_truncation_forces_ok_false` + `layer_memo_key_includes_node_limit`（2 红） | `trunc.rs:159`、`trunc.rs:188` | md5 回 `d6d2dd9c…` |
| M5 | `solve_local.rs:523` 键里 `node_limit` → `0usize` | R3 键 | `layer_memo_key_includes_node_limit`（1 红） | `trunc.rs:193`，红态 `second.truncated=true` | md5 回 `d6d2dd9c…` |
| P1 | （探针）旧 `solve_local()` 在 `limit=10/200/1e6` | 观察 | — | 三次输出逐字段相同（`reachable=true/max_hp=202`） | 探针文件删除 |
| P2 | （探针）`solve_layer_with` 先真商品表、再空表 | 观察 | — | `identical_goals=true`（空表仍出买臂解） | 探针文件删除 |

**副本还原终态**（证明实验没有伪造出的「绿」）：

```
$ md5sum crates/game-solver/src/solve_local.rs crates/game-solver/tests/solve_local.rs
d6d2dd9c9578a1fe7d5c9ecd397e8ac1  …/src/solve_local.rs
6cdb7c2792cf018f02a62fab24c49038  …/tests/solve_local.rs
$ cmp -s 副本/源仓（三个文件）→ same ×3
$ cargo test --release --test solve_local --test solve_local_trunc
test result: ok. 24 passed; 0 failed
test result: ok. 2 passed; 0 failed
```

## 4. 实现者自述与实际不符处

| 自述 | 实际 | 级 |
|---|---|---|
| 6.「`scene.rs` Y4 注释按事实改」 | `69` 一条仍错：注释说脚本 36「无楼层条件」，实际被 `post_victory_event_of(69, 35)` 的 `Option` 隐式钉在 `floor==35`（`rules/boss.rs:155-161`，单测 `:230-235`）；且与同段末尾「69@35 → 组7」自相矛盾 | 黄 |
| 3.「键加 `layer_node_limit()`…**也保证键=全部输入**」（`solve_local.rs:512`） | 键缺 `shops`，A 级串味实测（§3 P2：空表仍出买臂解）。该缺口是既有的，但**这句注释是本次新写的错话** | 黄 |
| 2.「`ok = reachable && !truncated`」（commit 正文） | 实现是 `Some(g) if reachable => ok: !truncated`，语义等价；`solve_local.rs:448` 的 doc 写「`ok = 有目标 && !truncated`」漏了 `reachable`，属措辞不精确（无行为影响） | 观察 |
| 7.「gates PASS=13、ratchet 1149→1154、binaries 75→76、74 金样 sha 全同」 | **全部核实为真**（§2 R7） | — |
| 5.「`ents_in` 改 `Option`，None=sprite / `Some(&[])`=清空」 | 核实为真（M3 变异必红） | — |
| 2/3.「M1–M5 五组变异自证」 | 我**独立重做了 6 组**（M1/M1b/M2/M3/M4/M5），全部按自述预期红；其中 M2 与 M4 的断言消息与自述一致 | — |

## 5. 未验证清单

1. **只跑 release 档**（与门禁同档）；debug 档未复跑（`overflow-checks` 由 release profile 保证）。
2. **74 金样 sha 取自门禁自报**（`state-baseline [GREEN]`），未逐文件手算 74 个 sha 与基线比。
3. **跨进程/跨线程 memo 串味未做实验**：`thread_local!` 是类型级保证，我按代码接受（B 级）。
4. **`gates/cli.py all` 未跑**（只跑 `check`），`visual-diff`/`diff-trace` 等 gate 本次未触发。
5. **P4 侧消费行为无法验证**：P4 尚未实现，`ExitSolve` 当前仓内消费者只有测试。
6. **并行 agent 的写入**只查了工作树与 `git status`/mtime，未做更早时间点的对照。
7. **mutant 覆盖面**：只针对 `solve_local_exit`/`HpLoss`/memo 键/`ents_in`/`ok` 五点；
   `Search` 内部剪枝（`weakly_ge`/`dominates`）的等价性未做变异（超出本轮 R1–R8）。

## 6. 结论

**「P3 现在能不能作为 P4 的地基」**：能——P4 唯一被授权读的 `ExitSolve.ok` 现在真的等于
「搜索完整且确有出层解」，红1/红2 都已有可复现的变异判据支撑（忽略注入必红、去掉 `!truncated` 必红、
memo 键丢 `node_limit` 必红），满界路径逐字节未变（f1 `out_hp=202/exit=(11,1)/route_len=6`）；
但 P4 必须自己守两条：**只信 `ok`**（`truncated=true` 时 `out_*` 是参考值），**别走旧的 `solve_local()`**
（那条路至今没有截断信号）。

**「还有没有必须再补的红」**：没有新的红；但开工 P4 之前建议补两条黄级硬约束——把旧
`solve_local()` 标注 `deprecated`/或在 `SolveResult` 里也加 `truncated`（否则「截断但报可行」还有出口），
以及给 `LAYER_MEMO` 键补 `shops`（或把 `solve_local.rs:512` 的「键=全部输入」改成实话）。
