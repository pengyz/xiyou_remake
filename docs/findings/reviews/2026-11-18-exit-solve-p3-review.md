# P3 独立对抗复核：`solve_local_exit`（单层「入场→出层」可续跑接口）

- 复核对象：`modern` 分支未 push 的两笔提交（`origin/modern` = `23a7eec`，HEAD = `7a8f29e`）
  - `95577e2` test(solver): 补 P2 掉落四条黄项判据（`crates/game-core/src/scene.rs` +6、`crates/game-solver/tests/solve_local.rs` +146/−6）
  - `7a8f29e` feat(solver): `solve_local_exit`（`crates/game-solver/src/solve_local.rs` +193/−19、`tests/solve_local.rs` +153/−2、`data/status/baselines/rust-test-counts.json` +2/−2）
- 复核方式：读代码 + 自建探针 + 变异（全部在临时副本 `/tmp/p3rev` 内做，原仓库零改动）+ 门禁实跑
- 复核者立场：**不采信提交信息与自述**；测试全绿不算判据，每条主张都自己造红

---

## 1. 判定摘要表

| 面 | 判定 | 一句话证据 |
|---|---|---|
| F1 注入是否真生效 | **红** | `player_in`/`ents_in` 各有判据（忽略后 2 条 / 1 条测试红）；**`flags_in` 被忽略后 21 条全绿**（§3 M2）⇒ 该注入零判据。我自建门夹具探针可一击命中（§2.1） |
| F2 出层态是否可消费且忠实 | **绿** | f1 `out_ents=30` 与 `sprite_full=35` 的 5 条差**全部追平到引擎**：1 门被开、1 钥匙被拾、3 条由门脚本 19 的 `markEntityRemoved` 清除（读回探针实测 `rm=true`）；`sprite1` 原始字节独立解码 n=35 恒等；抽 3 条交叉命中 |
| F3 确定性测试是否真测确定性 | **绿** | 测试确实双 `std::thread::spawn`（`tests/solve_local.rs:1191-1195`），`LAYER_MEMO` 是 `thread_local!`（`solve_local.rs:557`）⇒ 两份冷启；我另跑**跨进程双跑**，输出 diff 为空、md5 同为 `2c0b9a2c…` |
| F4 目标函数是否真是「出层血量最大」 | **黄** | 结构正确（`merge_layer_goals` 组键含完整 `post.player`，`solve_local.rs:671-693`；我的双路线夹具在正确代码下选 hp300/7 步而非 hp276/4 步）；但**把 `HpLoss` 的 hp 优先改成 steps 优先后 21 条全绿**（M4）⇒ tie-break 无判据 |
| F5 有没有偷偷放宽上限 | **红** | 上限/剪枝确实零改动（`layer_node_limit`/`weakly_ge`/`dominates` 无行为变更）；**但真数据 f1 `SOLVER_LAYER_LIMIT=10` ⇒ `ok=true && truncated=true`**，而字段文档写「`truncated` = `ok=false` 的成因分流」（`solve_local.rs:394-395`）⇒ `ok` 不蕴含「搜索完整」，此即 rubric 的红条件 |
| F6 四条补黄判据成色（Y1–Y4） | **黄** | Y1/Y2/Y3 成色足（M7 仅 Y1 红、M8 五红、M9 一红，红因均为精确集合/`item_types`，原因正确）；Y4 是**注释级**（无判据、无行为变更），且注释「69/70/71/72/73/74/75 各带自己的 `load_level_script` 参数」与实现不符（70\|71 共用一个臂 + 同一脚本 35） |
| F7 门禁与基线诚实性 | **绿** | `python3 gates/cli.py check` ⇒ `PASS=13 FAIL=0 SKIP=0`；末项 `state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同`；`test-ratchet total=1149(基线1149)`；`rust-test-counts.json` diff 仅 2 行且只升不降；无测试被删、无 `#[ignore]` 新增 |
| F8 红线 | **绿** | `git diff --stat a80f21c..7a8f29e -- original reference gates` 为空；`crates/game-core/src/scene.rs` 只有 1 个 hunk、6 行全 `//` 注释 |
| F9 并发写入风险 | **黄** | 两 commit 的文件集合与自述**完全一致**（无夹带）；但 reflog 显示 20:09 有一次 `commit (amend)` 把**另一条工作流的 commit message（「变形球机制窗收窄到 570 帧…」）**套到了本 P3 提交上（`abcd026`，树与 `7a8f29e` 同为 `412bf9f8`），20:12 才 amend 回正确 message ⇒ 同分支并发写入已实际发生 |
| F10 文档/自述诚实性 | **黄** | `ents_in` 空切片表述与实际一致（claim 7 属实，A 级核对）；f1 数字全部可复现（claim 6 属实）；但 (a) 「五条 `partial_eq` 断言」实为 5 条 `assert_eq!`（仓内 `partial_eq` 零命中）；(b) `truncated` 文档与 `ok=true&&truncated=true` 实测矛盾；(c) Y4 注释 70/71 表述失真 |

**计数：红 2（F1、F5）／黄 4（F4、F6、F9、F10）／绿 4（F2、F3、F7、F8）。**

---

## 2. 每面详证

### 2.1 F1 注入是否真生效 —— 红

三条注入逐条造变异（副本内改 `solve_local.rs`，跑 `cargo test -p game-solver --release --test solve_local`，21 条）：

| 变异 | 结果 | 说明 |
|---|---|---|
| M1 忽略 `player_in`（`solve_layer(..., input.level.player.clone(), flags_in)`） | 19 passed / **2 红**：`exit_solve_returns_entry_sensitive_result`（`tests:1173` `!b.ok` 失败）、`exit_solve_roundtrip_chains`（`tests:1224` left 300 / right 276） | `player_in` **有**判据 |
| M2 忽略 `flags_in`（传 0） | **21 全绿** | `flags_in` **零判据** ⇒ 按 rubric 记红 |
| M3 忽略 `ents_in`（恒 `None`） | 20 passed / **1 红**：`exit_solve_roundtrip_chains`（`tests:1224` left 252 / right 276） | `ents_in` **有**判据 |

`flags_in` 是不是「接线也错」？不是——接线是对的，只是没人测。我自建探针（`/tmp/p3rev/crates/game-solver/tests/p3probe_flag.rs`，复刻 `tests:540` 的 `fixture_gate`：13×13 全可走、`83@(6,9)` param19 门、`7@(11,1)` param2 梯、无 `ExitSpec`）：

```
[F1-flag] A flags_in=0       => ok=true  exit=Some((11, 1)) route=["6,10", "6,9", "gate 6,9"]
[F1-flag] B flags_in=bit19   => ok=false exit=None          route=[]
[F1-flag] C level=bit19,in=0 => ok=true  exit=Some((11, 1)) route=["6,10", "6,9", "gate 6,9"]
```

- A vs B：`flags_in` 的 bit19 真的把门重入挡死（`gate` 语义 `a.java:8193-8201`，见 `crates/game-core/src/rules/gate.rs:96`）⇒ 注入生效；
- C：`level.flags` 置位但注入传 0 时结果与 A 相同 ⇒ **`level.flags` 不参与注入**，注入是唯一来源（这也顺带证明 M1 那类「偷读 level」的变异会被抓到）；
- 同一 M2 变异下，我的探针 B 腿变 `ok=true` ⇒ **探针红，21 条全绿**。即：判据缺口可低成本补齐（建议直接抄这条夹具进门禁）。

结论：`player_in`/`ents_in` 绿，`flags_in` 红；整面按 rubric 记 **红**（性质=缺判据，不是接线错）。

### 2.2 F2 出层态可消费且忠实 —— 绿

f1 真数据全量 dump（探针 `p3probe.rs`，`--nocapture`）：

```
[F3] EXITF1_DEBUG=ExitSolve { ok: true, exit: Some((11, 1)), out_player: PlayerSpec { hp: 202, atk: 10, def: 10,
     yellow: 0, blue: 0, red: 0, gold: 4, ... }, out_flags: 0,
     route: ["6,11", "6,10", "5,10", "6,10", "door 6,9", "gate 6,9"], out_hp: 202,
     kind: Some(Script), cross_edge: Some((2, true)), truncated: false }
sprite_full n=35   out_ents n=30
ONLY_IN_SPRITE=["1@6,9;p0", "26@5,10;p0", "41@7,1;p0", "42@8,1;p0", "41@9,1;p0"]
ONLY_IN_OUT=[]
```

- 独立核对 `sprite1` 原始字节：直接 `game_data::sprite::SpriteTable::parse` 解 `assets/raw/sprite1` ⇒ n=35，与 `sprite_full` 逐条相等（`true`）；抽查 `15@10,11` / `26@11,6` / `31@11,3` 均 `in_raw_sprite1=true`。
- 5 条差异全部是**引擎侧状态变化**，不是求解读幻觉：
  - `1@6,9`（门）在 `gate 6,9` 被开、`26@5,10`（钥匙）在 `5,10` 被拾 —— 路线本身就能解释；
  - 余下 3 条 `41@7,1` / `42@8,1` / `41@9,1` 用副本内 `readback_ents` 临时 `eprintln` 实测两次读回：
    - 第 1 次（`5,10` 拾取后）：`count=35 live=34`，三条 `rm=false st=0`；
    - 第 2 次（`6,9` 门脚本 19 跑完后）：`count=34 live=30`，三条 `rm=true st=1`；
  - 即：**门脚本 19 的过场把 (7..9,1) 三格清了**（`markEntityRemoved`）。这是抽象层不建模、但真实引擎会做的副作用，恰好是「出层态必须来自引擎」的正面证据。
- 来源链路（A 级）：`st.player.*` 的唯一写点是 `readback_script`（`solve_local.rs:2609-2620`，全仓 grep `.player.hp =` 等只有这 10 行）；`readback_ents` 只在 `pickup_step`（`:2363`）与 `readback_script`（`:2631`）被调用；`readback_script` 只在 `gate_arrive` 的两条终局臂（`:2257` Pending / `:2271` Completed）被调用 ⇒ 求解器不自算战斗、不自造结算（与 M9 一致：凭空加钥匙会让 `tests:1105` 红）。
- f1 的 `out_player.gold=4`、`hp=202` 也都是读回值（`gold` 由引擎拾取金币改写）。

结论：绿。唯一保留见 §5 未验证项（我只核到 game-core 引擎路径，未对比 desktop 运行时线程）。

### 2.3 F3 确定性测试是否真测确定性 —— 绿

- 测试实现（`tests/solve_local.rs:1181-1207`）：`run` 闭包内 `std::thread::spawn(move || solve_local_exit(...))` 后 `join()`，两次调用各起一线程；断言 `route`/`out_hp`/`out_player`/`out_ents`/整体 `PartialEq`（`:1199-1203`）。测试注释本身写明「同线程第二次是缓存回放」——与实现一致。
- `LAYER_MEMO` 是 `thread_local!`（`solve_local.rs:557`）⇒ 双线程=两份冷启 memo，不存在缓存回放。
- 我另做**跨进程**双跑（两次独立进程，各跑同一探针 `probe_f3_determinism_print`）：

```
$ diff /tmp/f3run1.txt /tmp/f3run2.txt   # 空
$ md5sum /tmp/f3run1.txt /tmp/f3run2.txt
2c0b9a2c1b6904c3493c4380f64b6d48  /tmp/f3run1.txt
2c0b9a2c1b6904c3493c4380f64b6d48  /tmp/f3run2.txt
```

结论：绿（三态 fixture + f1 真数据的整份 `ExitSolve` Debug 输出逐字节相同）。

### 2.4 F4 目标函数是否真是「出层血量最大」 —— 黄

- 结构前提成立（A）：`merge_layer_goals`（`solve_local.rs:671-693`）组键 =
  `(g.kind, g.cross_edge, g.post.player.clone(), g.post.flags, g.post.pos, g.post.ents.clone())`，组内仅按 `(kills, steps)` 做支配剪枝 ⇒ 不同 hp 必不同组、同组 hp 必相同，「最大血那条被 Pareto 掉」在此结构下不可能（提交信息也写了这一点，我独立核对为真）。
- 行为正确（A）：我自建 `fixture_two_routes`（5×4；`y=1`/`y=2` 两行可走；`41@(1,1)` 只堵上路；出口 `7@(4,1)`）：上路 `bump 41` 得 hp276/4 步，下路绕行 hp300/7 步。正确代码下：

```
[F4] solve_local_out exit => out_hp=300 route_len=7 route=["0,1","0,2","1,2","2,2","2,1","3,1","4,1"]
```

  既取血量最大、又**不是**最短路 ⇒ 目标函数确实以 hp 为第一关键字（不是「能出层就行」也不是「步数最短」）。

- 判据缺口（黄的原因）：把 `HpLoss::better` 改成 **steps 优先**（M4），**21 条测试全绿**；只有我的夹具变红（`out_hp 276 vs 300`，路线变 `bump 41`）。即「hp 优先于 steps」这条 tie-break 目前**没有任何测试看守**。
- 关于 f1 那条断言 `got.out_hp == old.max_hp`（`tests:1264-1268`）：它**不是**纯循环论证——`solve_local` 走 `run()`，`solve_local_exit` 走 `run_root+merge_layer_goals`，是两条不同代码路径，故这条交叉校验对「merge 不丢最大血」有实质价值；但「目标函数=hp 最大」本身它证明不了（两侧同一个 `HpLoss`，M4 下两边一起变、仍相等）。

结论：黄（实现对、文档对、判据缺）。

### 2.5 F5 有没有偷偷放宽上限 —— 红

- 放宽上限：**没有**。`git diff a80f21c..7a8f29e` 中 `layer_node_limit`/`SOLVER_LAYER_LIMIT`/`bound`/`node_cap`/`MAX_` 只出现在注释（2 处），`weakly_ge`/`dominates` 零改动；`layer_node_limit()` 仍默认 200_000（`solve_local.rs:585-591`）。
- 但 rubric 的红条件 **「`ok` 在截断时为 true」直接命中**（真数据 f1，探针扫 `SOLVER_LAYER_LIMIT`）：

```
[F5-f1] limit=10 => ok=true truncated=true out_hp=202 exit=Some((11, 1)) route_len=6
[F5-f1] limit=16 => ok=true truncated=true ...
[F5-f1] limit=24/30/40/50 => ok=true truncated=true ...
（3×3 夹具：limit=1..9 ⇒ ok=false&truncated=true；limit≥12 ⇒ ok=true&truncated=false）
```

  即：**截断时 `ok` 仍为 true**，而 `ExitSolve.truncated` 的字段文档（`solve_local.rs:394-395`）写的是「`ok=false` 的成因分流」⇒ 文档与行为矛盾。后果是可消费性伤害：P4 若按文档把 `truncated` 当「ok=false 的细分原因」，就永远不会在 `ok=true` 时检查它，从而把**被截断的 202 当作出层最大血**。f1 恰好截断解与完整解同值，但那是运气，不是契约。
- 附带发现（同面，非同一次提交引入但被新接口放大）：`LAYER_MEMO` 键（`:487-495`）**不含 `layer_node_limit`**，memo 命中时回放 `hit_trunc`（`:499-506`）⇒ 同线程先跑小界、后跑大界会**回放截断结果**。实测：新线程 f1 `limit=50` 首跑 `truncated=false`；同线程先跑 `limit=10` 后再 `limit=50` 变 `truncated=true`。生产上 limit 恒定影响有限，但任何「按层调界」的 P4 尝试都会踩这个坑，且会让 `truncated` 语义不可信。
- 另：`take_layer_trunc()`（`:593-596`）文档写「取走并清零」，实现只 `get()` 不清零（`git log -S` 显示这段来自 `6de9ec1`，**非本两笔提交引入**）⇒ 记为文档失真，低危（`solve_layer_with` 入口会清零）。

结论：红（不是「偷放宽」，是 `ok`/`truncated` 不正交 + 缓存键缺维度）。

### 2.6 F6 四条补黄判据成色（Y1–Y4）—— 黄

| 判据 | 变异复现 | 结果 | 判定 |
|---|---|---|---|
| Y1 闭包臂掉落（`tests:1025-1049`） | 删掉闭包 Kill 臂的 `apply_kill_drops`（`:1737` 附近） | 20 passed / **1 红** `closure_kill_arm_spawns_drop_entity`（`tests:1037` left `[(7,2,1,2)]` / right `[(7,2,1,2),(17,6,4,0)]`） | **绿**：若 Monkey 臂也能杀 67，删闭包臂掉落不会红 ⇒ 实测证明该夹具**只经过闭包臂**（`expand_inner:1770` 早返回前提成立） |
| Y2 boss 掉落单一来源（`tests:938-1000`） | `apply_kill_drops` 改成任何怪都全组事件全发 | 16 passed / **5 红**：`boss_kill…`(`:951`)、`non_boss_kill_spawns_no_drops`(`:983`)、`closure_kill_arm…`(`:1037`)、`drop_entity_pickup…`(`:1114`)、`exit_solve_roundtrip_chains`(`:1219`) | **绿**：红因均为精确集合多出实体（不是数量差/顺序差） |
| Y3 拾取入账（`tests:1089-1121`） | 读回后丢弃引擎背包、凭空 `ns.player.yellow += 1` | 20 passed / **1 红** `drop_entity_pickup_credits_inventory`（`tests:1105` left `Some(0)` / right `Some(17)`） | **绿**：红在 `item_types[0]`，正是「入账必须是引擎」那条 |
| Y4 69 型未数据化备注（`scene.rs` +6） | 无变异可言（纯注释） | — | **黄**：注释本身把未数据化面写清楚了（`:3753-3761`，69 臂仍写死 `rules::boss::post_victory_event_of(69, …)` + `load_level_script(36)`），但「69/70/71/72/73/74/75 同构、**各带自己的** `load_level_script` 参数」与实现不符：`70 \| 71` 合并成一个臂共用 `35`（`:3774-3778`），`72 → 65`（`:3776-3779`）。细节失真，无行为影响 |

结论：Y1/Y2/Y3 成色足；Y4 注释级且有一处失真 ⇒ 整面 **黄**。

### 2.7 F7 门禁与基线诚实性 —— 绿

```
$ python3 gates/cli.py check        # exit 0
[✓ PASS] original-integrity … [✓ PASS] assets-manifest {'entries': 138, …}
[✓ PASS] reference-seed-integrity — 8457 行，sha256 对照基线通过
[✓ PASS] rust-tests — cargo test --release 全绿
[✓ PASS] test-ratchet — total=1149(基线1149) 无回退
[✓ PASS] table-content-hash … [✓ PASS] table-evidence … [✓ PASS] state-machine-anchors
[✓ PASS] asset-truth-manifest — entries=138，art 三态核通过
[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同
== 汇总 == PASS=13 FAIL=0 SKIP=0
```

- 74 金样：`state-baseline` 逐字节 sha 全同 ⇒ 与 claim 8 一致（另有 `git diff --stat … -- data/status/baselines` 只有 `rust-test-counts.json`）。
- 基线 diff：仅 `"solve_local(test)": 15 → 21`、`"total": 1143 → 1149` 两行，只升不降。逐提交数 `#[test]` 对账（A 级）：

```
git show a80f21c:crates/game-solver/tests/solve_local.rs | grep -c '#\[test\]'   # 15
git show 95577e2:crates/game-solver/tests/solve_local.rs | grep -c '#\[test\]'   # 17（P2：+2 新测试；另 2 条既有断言由 contains 加强为精确集合）
git show 7a8f29e:crates/game-solver/tests/solve_local.rs | grep -c '#\[test\]'   # 21（P3：+4）
```

  即 P2 未更新 ratchet（基线仍 15），P3 一并补到 21（15+6=21），`total` 1143+6=1149 与 `test-ratchet` 打印的实测 `total=1149` 一致 ⇒ 数字诚实（不是「只升」掩盖了漏记）。
- 无测试被删：`git diff a80f21c..7a8f29e -- tests/solve_local.rs` 的 `-` 行只有 5 类（import 改写、`assert!(`、`g.post_ents`、`.contains(&EntRec{…})`、说明串），属 `contains` → 精确集合相等的**加强**；`grep -rn "#\[ignore"` 在 `crates/game-solver/{src,tests}` 零命中。

结论：绿。（注：`gates/cli.py status` 报 `SKIP=1`（visual-diff 未就绪），`check` 报 `SKIP=0`；实现者自述的「SKIP=0」只对 `check` 成立，不算谎报，但引用时应带上口径。）

### 2.8 F8 红线 —— 绿

```
$ git diff --stat a80f21c..7a8f29e -- original reference gates
(空)
$ git show 95577e2 -- crates/game-core/src/scene.rs | grep -E "^[+-]" | grep -v "^[+-][+-]"
+                            // 【Y4 已知未数据化，2026-11-18】…（6 行全 //）
```

- `original/`、`reference/`、`gates/` 三处零改动（含 `gates/cli.py` 与 hook）。
- `game-core` 只有 `scene.rs` 一个 hunk、6 行注释，无语句变更（不改变 `69` 臂行为）。

结论：绿。

### 2.9 F9 并发写入风险 —— 黄

- 文件集合与自述一致、无夹带：

```
$ git diff --name-only 95577e2^..95577e2
crates/game-core/src/scene.rs
crates/game-solver/tests/solve_local.rs
$ git diff --name-only 7a8f29e^..7a8f29e
crates/game-solver/src/solve_local.rs
crates/game-solver/tests/solve_local.rs
data/status/baselines/rust-test-counts.json
```

- 但**同分支并发写入已实际发生**（reflog，A 级）：

```
7a8f29e HEAD@{2026-11-18 20:12:16}: commit (amend): feat(solver): 单层「入场→出层」可续跑接口…
abcd026 HEAD@{2026-11-18 20:09:34}: commit (amend): test(coverage): 变形球机制窗收窄到 570 帧，补 0px 机器指针，窗外分歧立探针
b1c5655 HEAD@{2026-11-18 19:36:51}: commit:        feat(solver): 单层「入场→出层」可续跑接口…
```

  - `abcd026` 的**message 属另一条工作流**（anim/coverage 窗），而它的**树 = `7a8f29e` 的树**（都是 `412bf9f8…`）⇒ 20:09 那次 amend 把别人的 commit message 套到了本 P3 提交上（且顺带把 `rust-test-counts.json` 纳入：`b1c5655` 的树是 `6252796…`，不含该文件），20:12 又 amend 回正确 message。最终提交内容与自述相符，但过程被并发污染。
  - 风险提示（供 lead 处置，我不动仓库）：那次 amend 可能吞掉了另一 agent 的 WIP（当前树无任何 570 帧/coverage 改动，`git fsck` 的悬空提交里也没有该 message 的独立提交）。若那位 agent 以为已提交，需要人工核对。
- 工作树状态（复核期间两次采样一致）：仅 `?? docs/findings/reviews/2026-11-18-boss-drops-p2-review.md`（前一轮复核产物）+ 本报告；HEAD 全程 `7a8f29e` 未漂移。

结论：黄（内容干净，流程有并发污染实证）。

### 2.10 F10 文档/自述诚实性 —— 黄

逐条对 claim 的核对结果见 §4。要点：

- **诚实**：claim 6（f1 七个数字）逐项复现一致；claim 7（`ents_in` 空 ⇒ `sprite_full`，无法表达「层内实体全清」）与代码 `let ents = if ents_in.is_empty() { None } …`（`:441-445`）及文档（`:426-428`）完全吻合；claim 3（读回复用）经链路核对为真；claim 5（双线程动机）为真且注释写明。
- **失真**：(a) claim 4 的「五条 `partial_eq!(...)` 断言」——全仓 `grep -rn partial_eq` 零命中，实际是 5 条 `assert_eq!`（`tests:1199-1203`）；(b) `truncated` 文档（「`ok=false` 的成因分流」）与 `ok=true&&truncated=true` 实测矛盾（§2.5）；(c) Y4 注释 70/71 表述（§2.6）；(d) `take_layer_trunc` 文档「取走并清零」而实现不清零（前序提交引入，`6de9ec1`）。
- 没有发现「把未验证写成已一致」：f1 的 `out_ents=30` 等数字我都能复现，且 `merge` 组键主张我独立读码确认为真。

结论：黄。

---

## 3. 变异实证表（全部在临时副本内做；每次 `cp` 还原并打印 md5 核对）

副本：`/tmp/p3rev`（`rsync` 剔除 `target/.git/original/reference/analysis/save/android/docs`，83M）
pristine 源：`/tmp/p3-pristine/solve_local.rs`、`/tmp/p3-pristine/tests_solve_local.rs`
pristine md5：`solve_local.rs = e205d28545a1d26d0ed2e8f58a67e0af`
实验命令：`CARGO_TARGET_DIR=/tmp/p3-target cargo test -p game-solver --release --test solve_local`

| ID | 改了什么 | 哪条测试红 | 红在哪（断言/差值） | 还原证据 |
|---|---|---|---|---|
| M1 | `solve_local_exit` 内改用 `input.level.player.clone()` | `exit_solve_returns_entry_sensitive_result`、`exit_solve_roundtrip_chains` | `tests:1173` `!b.ok` 失败；`tests:1224` left 300 / right 276 | md5 回 `e205d285…`，21 全绿 |
| M2 | `solve_local_exit` 内 `flags_in` 传 `0`（`let _ = flags_in;`） | **无（21 全绿）** | — | 同上（**判据缺口**） |
| M2′ | 同 M2 + 我的 `p3probe_flag` 探针 | 探针 `probe_f1_flags_injection` | B 腿（`flags_in=bit19`）由 `ok=false` 变 `ok=true` ⇒ 门重入未被挡 | 同上 |
| M3 | `solve_local_exit` 内 `ents` 恒 `None` | `exit_solve_roundtrip_chains` | `tests:1224` left 252 / right 276 | 同上 |
| M4 | `HpLoss::better` 改 steps 优先 | **无（21 全绿）** | 我的双路线夹具变红：`out_hp 276 vs 300`（走了 `bump 41` 的短命路） | 同上（**判据缺口**） |
| M5 | `merge_layer_goals` 组键去掉 hp（`PlayerSpec{hp:0,..}`） | 无（21 全绿） | 等价变异：同组 `post` 全等 ⇒ hp 必等，行为不变；**不足以**判「无判据」 | 同上 |
| M7 | 删闭包 Kill 臂的 `apply_kill_drops`（Y1） | `closure_kill_arm_spawns_drop_entity` | `tests:1037` left `[(7,2,1,2)]` / right `[(7,2,1,2),(17,6,4,0)]` ⇒ 证明该夹具只走闭包臂 | 同上 |
| M8 | `apply_kill_drops` 改成任意怪都全组事件全发（Y2） | 5 条：`boss_kill…`(951)、`non_boss_kill_spawns_no_drops`(983)、`closure_kill_arm…`(1037)、`drop_entity_pickup…`(1114)、`exit_solve_roundtrip_chains`(1219) | 均为精确集合多出实体（如 left 含 `29/16/24/26…`，right 只有 `[(7,2,1,2)]`） | 同上 |
| M9 | `pickup_step` 读回后丢弃引擎背包 + 凭空 `yellow += 1`（Y3） | `drop_entity_pickup_credits_inventory` | `tests:1105` left `Some(0)` / right `Some(17)` | 同上 |

搜索界扫描（M6，未改代码，只设 env）：见 §2.5 的 `[F5-f1]` 与 3×3 扫描输出。

**原仓库还原证据**（复核结束采样）：`git status --short` 只有两个未跟踪的 `docs/findings/reviews/*.md`（前一轮 P2 复核 + 本报告），HEAD=`7a8f29e`，`crates/` 下无任何修改；所有变异只发生在 `/tmp/p3rev`。

---

## 4. 实现者自述与实际不符之处（逐条）

1. **claim 4「五条 `partial_eq!(...)` 断言」** —— 仓内 `partial_eq` 零命中。实际：`ExitSolve` 派生 `PartialEq, Eq`（`solve_local.rs:370`），确定性测试里 5 条 `assert_eq!`（`tests:1199-1203`）。用词错，事实不错。
2. **`truncated` 字段文档** —— 写「`ok=false` 的成因分流」（`solve_local.rs:394-395`），实测存在 `ok=true && truncated=true`（真数据 f1，`SOLVER_LAYER_LIMIT=10`，§2.5）。文档把二维状态压成一维，P4 会照文档漏检。
3. **Y4 注释细节** —— 「69/70/71/72/73/74/75 同构，**各带自己的** `load_level_script` 参数」（`scene.rs:3757-3758`）不确切：`70|71` 共用一个臂、共用脚本 `35`（`scene.rs:3774-3778`）。
4. **`take_layer_trunc` 文档「取走并清零」** —— 实现 `L594-596` 只 `get()` 不清零。非本两笔提交引入（`6de9ec1`），但被新接口当作「成因分流」依据，值得一并修文档。
5. **claim 8「没动 `layer_node_limit()`」属实**，但「`truncated` 只是新增只读回显、不影响 `ok` 判定」这句在**语义**上成立、在**契约**上不成立：`ok` 在截断时为 true（rubric 红条件）。
6. **commit message「TDD 先红后绿，红态见下」** —— 无法从单笔提交验证先红后绿（实现与测试同笔）；只能验证「注入被判据看守」的覆盖面（§3）。列为未验证，不判失真。

其余自述（claim 1/2/3/5/6/7）**与实测一致**，包括最容易吹的那条 f1 七个数字。

---

## 5. 未验证清单（诚实列出）

1. **`out_ents` 与「desktop 运行时线程」是否一致**：我只核到 game-core 引擎路径（`build_scene → try_step/run_scene → readback`）与 `sprite1` 原始字节；未在 desktop 实机线程跑同一序列比对（授权边界内不做 UI）。故「与引擎一致」限于 **game-core 层**。
2. **P4 关心的性能面**：未测 `solve_local_exit` 在 54 层接力场景下的耗时/状态数（本复核只做语义与判据）。
3. **`total` 1149 的分母**：我核了 `solve_local` target 的 `#[test]` 逐提交计数（15/17/21）与门禁打印的实测 `total=1149`，但未把其余 ~70 个测试二进制的条数逐一对账（那是 `ratchet-update` 的职责，本复核只判「无回退+本次 +6 有据」）。
4. **另一 agent 的 WIP 是否被那次 amend 吞掉**：reflog 证明 message 串味与树变化（`6252796 → 412bf9f8`），但无法证明其源码改动是「已丢」还是「从未 staged」。需该 agent 自查。
5. **`Y4` 与 Java 侧的对应**（`a.java:10144-10149` 组 7 等）未回读反编译源核对，只核了 Rust 侧自洽。
6. **M5 的等价性**：我判定它行为等价（同组 post 全等），但未穷举证明「merge 组键去掉 hp 后任何 fixture 都不会改变选解」；结论仅用于「不能据此判无判据」。

---

## 6. 结论（两句话）

1. **「P3 能不能作为 P4 的地基」**：可以当地基，但只能当**结构地基**、不能当**契约地基**——接口形状、注入接线、出层态来自引擎读回这三件硬事都真（F2/F3/F8 绿），P4 按 `ExitSolve` 四态接力写代码不会被结构绊住；但 `flags_in` 零判据与 `ok`/`truncated` 不正交这两条会让 P4 继承「被门挡住也报 ok」「截断解当最大血」的错误前提。
2. **「有没有必须先补的红」**：有两条，都在判据/契约层、都不涉搜索重写——(1) 补 `flags_in` 判据（直接抄我 §2.1 的门夹具三腿探针：`flags=0 ⇒ ok`、`bit19 ⇒ !ok`、`level.flags` 不参与）；(2) 让 `ok` 与 `truncated` 正交（截断即 `ok=false`，或把文档改成「`ok`=存在可行出层、不保证层内最优」并在 P4 消费侧强制检查 `truncated`），顺带把 `LAYER_MEMO` 键补上 `layer_node_limit`。

---

### 附：复核实跑命令（原仓库只读，除本报告外零写入）

```bash
git status --short; git log --oneline -6; git reflog -8 --date=iso
git diff --name-status a80f21c..7a8f29e; git diff --stat a80f21c..7a8f29e -- original reference gates
git diff a80f21c..7a8f29e -- crates/game-core/src/scene.rs
git diff a80f21c..7a8f29e -- crates/game-solver/tests/solve_local.rs | grep '^-'
git show --stat 95577e2 7a8f29e
cargo test -p game-solver --release            # lib 38 / census 4 / solve_global 24 / solve_local 21 全绿
python3 gates/cli.py status                    # PASS=13 FAIL=0 SKIP=1（visual-diff）
python3 gates/cli.py check                     # PASS=13 FAIL=0 SKIP=0
grep -rn "partial_eq" crates/                  # 零命中
grep -rn "#\[ignore" crates/game-solver/{src,tests}   # 零命中
# 临时副本 / 探针（不入库）
rsync -a --exclude target --exclude .git … /tmp/p3rev/
CARGO_TARGET_DIR=/tmp/p3-target cargo test -p game-solver --release --test p3probe -- --nocapture
CARGO_TARGET_DIR=/tmp/p3-target cargo test -p game-solver --release --test p3probe_flag -- --nocapture
CARGO_TARGET_DIR=/tmp/p3-target cargo test -p game-solver --release --test solve_local   # 每轮变异后
```
