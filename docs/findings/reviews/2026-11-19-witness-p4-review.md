# P4 witness 复核（独立对抗复核 · 2026-11-19）

- 复核对象：`501f02a`（P3 收口三条黄）、`c6a9a00`（P4 witness builder + `solver run`），HEAD=`c6a9a00`，分支 `modern`（未 push）。
- 授权边界：**只读 + 可跑测试/门禁/CLI**；仓库内只写本文件；所有变异在 `/tmp/p4rev-base`（`cp -a` 含 `target`，只重编 `game-solver`，10–15s/次）进行，`git checkout -- .` 逐次回滚。
- 环境：`nproc=20`、`cargo/rustc 1.94.0`、`target/release/solver` 与 HEAD 一致（增量构建 0.17s 无重编）。
- 纪律：实现者自述逐条复现，**测试全绿不作判据**；证据分级按 `AGENTS.md` §3（A=字节/实跑 trace，B=代码语境，C=猜测）。

## 1. 判定摘要表

| 面 | 判定 | 一句话证据 |
|---|---|---|
| F1 witness 接力 | **绿** | 真数据 JSON 逐层 `out_player==in_player` / `out_flags==in_flags` / `out_hp==in_hp` 全 True；接力点唯一 `witness.rs:636-637`；实体表按层归属（`ents_out≠ents_in` 属设计，`witness.rs:633` 首入层取 `sprite_full`，变异 M9 打红对应判据） |
| F2 拒绝造通关 | **绿** | `status="witness"` 全仓唯一赋值点 `witness.rs:493`（在 `Ending` 分支内、`truncated`/`!ok` 两闸之后）；`record_floor` 仅 `:492`/`:589` 两处，均在 `!ok` 闸后 ⇒ 不存在 `witness` 携 `ok=false` 层；M1（删 `!ok` 闸）、M2（`truncated` 闸移到 Ending 后）各打红一条判据 |
| F3 f54 层界 | **绿** | 插桩 `[TRUNC]`：200k ⇒ `nodes=200000 limit=200000 goals=4826 queue=50899`；1M ⇒ `nodes=1000000 limit=1000000 goals=30391 queue=181257`。节点数**恰等于上限且队列非空** ⇒ 真撞界；三档参考 `out_hp` 单调 2027<2627<3127（A 级三份 JSON） |
| F4 hp/atk 跳变 | **绿** | `XIYOU_TRACE` 拾取 trace：atk 510→730 = f52(11,8) 拾 **ty=36 金箍棒**（Equip tier5 +220），def 510→730 = f52(11,9) 拾 **ty=40 紫金龙鳞甲**（tier11 +220）；hp 为 HpUp 净额（回春丸 +50 / 长寿丹 +200 的离散块）；`pickup_step` 拾后 `ns.ents=readback_ents(&sc)` ⇒ 无双计，`apply_kill_drops` 只 spawn 实体不入账 |
| F5 shops_key | **绿** | `ShopGoodDef` 六字段中仅 `fail_hint` 未入键（`solve_local.rs:600-604` 实测编码七臂 `(tag,a,b,c)`）；键被 `solve_global.rs:494` 真实消费（真实 `shops`），M5b（键塞空表）打红 `layer_memo_key_includes_shops:389`。注记两条：`level_digest` 是 64-bit FNV 摘要（与「拒绝摘要」的理由不完全自洽）；witness 路径传 `&[]`（`solve_local.rs:380`） |
| F6 LAYER_TRUNC 竞态 | **绿** | `LAYER_TRUNC` 为 `static thread_local Cell<Option<..>>`（`solve_local.rs:757`）⇒ 跨线程不可能串；自造探针（旧入口↔层入口双向往返 + 同入口连跑）**5 passed**；删旧入口清零 ⇒ 探针红于 `:424`，泄漏值 `truncated:true,confidence:"capped-search",max_hp:Some(202)`（202 是满界真值 ⇒ 假降级）⇒ 501f02a 的清零是**载荷性**修复 |
| F7 确定性/信息完整 | **黄** | 同 budget 跨进程 sha 完全相同（200k 两次均 `e8aee40c…`）⇒ 确定性成立；但（a）自述「200k 与 1M sha 相同」**为假**（实测 `e8aee40c…`/`1e1b55f4…`/`90e70989…` 三档互异，仅 `blocked.detail` 字符串里的 `out_hp` 不同）；（b）f54 参考 hp **只存在于人类可读的 `detail` 字符串**，无结构化字段，budget 也不进 JSON ⇒ 下游只能正则挖 |
| F8 门禁与基线 | **绿** | `python3 gates/cli.py check` ⇒ `PASS=13 FAIL=0 SKIP=0`，末项 `state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同`；独立复跑 `tools/state_baseline.py --check` 亦 `[GREEN] 74 场景 sha 全同`；`test-ratchet total=1164(基线1164)`；基线文件实测 `binaries=77 / sum=1164 / witness(test)=8 / solve_local_trunc(test)=4`；两 commit 未新增 `#[ignore]` |
| F9 夹带/并发 | **绿** | `--name-status` 与自述一致（501f02a 3 文件、c6a9a00 8 文件）；`reflog -12` 两处 amend（`7a8f29e`/`abcd026`）**均早于** 501f02a；工作树仅 3 个**他人**未跟踪 review 文档，无被改的跟踪文件 |
| F10 判据成色 | **绿** | 8 条判据中 **5 条**被至少一次变异打红（`refuses_to_fake_success` / `truncated_floor_is_not_consumed` / `goal_must_point_forward` / `up_line_comes_from_floor_graph` / `chains_real_floors`）；余 3 条中 `witness_is_deterministic` 功效低（同进程双线程只证可重复）、`witness_json_schema_stable` 只验结构、合成塔用例**抓不到**丢 `out_player`（M8 时它仍绿） |
| **X1（附加）** 通关可达性 | **黄** | 真数据下 `status="witness"` **不可达**：`up_line_next` 只认 `StairUp`（`witness.rs:313`），而 f49 **无 StairUp**（`floors.json` 只有 `StairDown→48`），f50 只能靠**上行符**（`ItemUp`）进入；`Ending` 又只在 **floor 50** 产出（`solve_local.rs:1933`/`:2041` 的 `floor == 50 && ety == 72`）⇒ 主线走到 f49 必 `blocked/no_up_line`，永远到不了 f50 |

计数：**绿 9（F1–F6、F8–F10）/ 黄 2（F7、X1）/ 红 0**。

## 2. 每面详证

### F1 witness 接力是否真接力 — 绿

代码路径（B 级）：`witness.rs:452` 传入 `&cur_player, cur_flags`，`:636-637` 更新 `cur_player = next_player; cur_flags = next_flags;`；`next_player` 来自 `witness.rs:593` `let next_player = got.out_player.clone();`（出层求解结果），**不是**初始态。接力点在全仓唯一（`grep cur_player = next_player` 仅 `:636`）。

A 级实证（自写脚本，不依赖测试）：

```bash
./target/release/solver run --out /tmp/p4rev/r200k.json
python3 - <<'PY'   # 逐层断言
a,b = fl[i], fl[i+1]
a['out_player']==b['in_player'], a['out_flags']==b['in_flags'], a['out_hp']==b['in_hp']
PY
```

输出：

```
total_steps 196 sum route 196
f51->f52 out_player==in_player:True out_flags==in_flags:True out_hp==in_hp:True
f52->f53 out_player==in_player:True out_flags==in_flags:True out_hp==in_hp:True
kinds ['Stair', 'Stair', 'Stair'] ok flags [True, True, True] trunc [False, False, False]
first floor ents_in is None: True
f52 ents_in len 23
```

**实体表不等是设计而非缺陷**：`ents_out` 是 f51 离场残余位图，`ents_in` 是目标层自己的位图；`witness.rs:633` 在新层首入时 `insert(target, sprite_full(tlevel))`，故 f52 的 `ents_in` 长度 = 该层 sprite 全表（23）。变异 **M9**（把 `sprite_full(tlevel)` 换成 `Vec::new()`）⇒ `witness_chains_real_floors` **FAILED**，证明该断言是真断言、不是空转。

注记（非缺陷）：`in_hp`/`out_hp` 与 `in_player.hp`/`out_player.hp` 是同一事实的两份投影，JSON 里可互相矛盾；P5 消费方应择一。

### F2 拒绝造通关是否真拦得住 — 绿

闸序实测（B 级，`witness.rs`）：

```
:455  if got.truncated   → status="capped", reason="truncated"   // 硬约束 A：截断层不可消费
:470  if !got.ok         → status="blocked", reason="no_exit"
:491  if kind == Ending  → record_floor(...); status="witness"; ending_done=true; break
:498  next_up None       → blocked "no_up_line"
:515  Cloud              → blocked "cloud_not_supported"
:530  if !hit            → blocked "no_exit_toward"
:548/:559/:575           exit/edge/target 核验 → "no_exit"/"stair_edge_missing"/"edge_mismatch"
:589  record_floor(...)  // 出层边核验通过后才落账
```

旁路排查（逐条否定）：

1. `status="witness"` 全仓唯一赋值点 `:493`，位于 `Ending` 分支内部，其前有 `truncated`（:455）与 `!ok`（:470）两道闸 ⇒ 不可能出现「capped/blocked 却 ending_done=true」。
2. `record_floor` 只有两个调用点（`:492`、`:589`），**都在 `!ok` 闸之后** ⇒ `floors` 里不可能出现 `ok=false` 的层。实测 JSON 佐证：`ok flags [True, True, True]`、`trunc [False, False, False]`。
3. `:643-646` 还有防御性不变式：`blocked.is_some() && ending_done` ⇒ 强制 `ending_done=false`（capped/model_broken 同路径）。
4. `Ending` 分支**先于** `next_up`/`hit` 判定 ⇒ 语义上「有终局就以终局收工，即使层图还指了上行边」。真数据下这不构成漏洞（f50 无 StairUp），但作为 P5 前的模型假设应显式记录。

变异实证：M1（`if !got.ok` → `if false && !got.ok`）⇒ `witness_refuses_to_fake_success` **FAILED**；M2（把整块「硬约束 A」移到 Ending 分支之后）⇒ `witness_truncated_floor_is_not_consumed` **FAILED**。两条闸都是载荷性的。

**但见 X1**：拦得住「假通关」不等于「真通关可达」。

### F3 f54 到底是撞层界还是搜索停滞 — 绿（4M 节点数未插桩）

A 级插桩（副本里在 `LAYER_TRUNC.set(Some(...))` 处加 `eprintln!`，唯一触发点 `solve_local.rs:2146`）：

```
[TRUNC] f54 nodes=200000  limit=200000  goals=4826  queue=50899
[TRUNC] f54 nodes=1000000 limit=1000000 goals=30391 queue=181257
```

节点计数**精确等于各自的界**、且 `queue` 非空（还有待扩展前沿）⇒ 不是「提前收敛/空耗」，是真撞界。若三档都撞在同一个固定数字（例如都恰好 200k）才是「跳过而非撞界」——不是。

三档实跑（原仓库 release，非副本）：

| budget | wall | blocked.reason | 参考 out_hp | sha256 |
|---|---|---|---|---|
| 200 000 | 9.5s | truncated | 2027 | `e8aee40c…`（重跑同值） |
| 1 000 000 | 97.4s | truncated | 2627 | `1e1b55f4…` |
| 4 000 000 | **24m56s**（user 5m39s，与门禁并发争核） | truncated | 3127 | `90e70989…` |

单调性成立且增量均匀（+600、+600）⇒ 收益随预算近似线性、前沿仍宽，属「状态空间宽度」问题而非「收敛边缘」。

未插桩项：4M 档的 `nodes/limit/goals/queue` 未测（单次 25 分钟，未再排）；4M 的节点数按同一代码路径推断为 `=4000000`。

注记：4M 需 **25 分钟墙钟**（且 stderr 仅一行摘要），不适合当常设门禁/日常判据。

### F4 血量/攻防跳变是否合法 — 绿

来源定位（A 级 trace：副本里给 `Cell::Pickup`/`Cell::Monster`/`Cell::Shop` 三臂加 `XIYOU_TRACE` gated `eprintln!`，`solver run --max-floors 2`）：

```
[PICKUP] 9299 条   [BUMP] 2816 条   [SHOP] 0 条
f52 (11,8)  ty=36  atk 510 -> 730
f52 (11,9)  ty=40  def 510 -> 730
hp 147->197 (+50 回春丸 ty31)；139->339；339->539 (+200 长寿丹 ty32)
```

数值对账：`data/tables/items.json` 36 = 金箍棒 `Equip tier5 bonus 220`、40 = 紫金龙鳞甲 `tier11 bonus 220`；`effect_registry.rs` `EquipEffect` 的增量 = 新档 − 旧档 ⇒ 510+220=730 自洽。510 本身由同层若干武器/防具先后装备累加（trace 逐条可见），29/30 属性丹只 +1（`floor>50 ⇒ tier=1`，`effect_registry.rs:157-166`）。

**双计排查（两条都否定）**：

- 同一颗丹被拾两次：`pickup_step` 直调引擎（`build_scene→try_step→tick→apply_step_cell_effects`），结束后 `ns.ents = readback_ents(&sc)`（`solve_local.rs:2559`）⇒ 拾取即从位图移除，不存在二次入账。
- `apply_kill_drops` 与拾取臂都入账：`apply_kill_drops` **只 spawn 实体**（`solve_local.rs:1098`），金币由 `kill_bounty` 单独入账（注释同源 `scene.rs:2301-2309`）⇒ 两臂互斥。

因此 f51 300→247（净掉血）、f52 247→427（战斗掉血 + 回春/长寿净额）、f53 427→827 全部是引擎效果的合成，不是求解器自算、不是双计。f54 的 827→2027 是 `blocked.detail` 里的**参考值**（同一层搜索在更大预算下的最优出层 hp），未消费。

### F5 shops_key 是否真覆盖语义 — 绿（两条注记）

`ShopGoodDef` 全字段（`crates/game-data/src/defs.rs:621-635`）：`id, src, idx, price, effect, fail_hint`；`ShopEffect` 七臂（`:639-647`）。

- `shops_key`（`solve_local.rs:598-619`）投影 `(id, src, idx, price, (tag,a,b,c))`，七臂各自定长编码，**没有丢参数**。
- `fail_hint` 的消费面实测只有文案：`crates/game-core/src/scene.rs:2549/2582/2615` 的 `show_overlay_message`，以及 `crates/game-data/tests/shop_goods_defs.rs` 的文案断言 ⇒ 不进键安全。
- 行为输入还包括：`st.ents[slot].param`（开价位，读 `root_ents`）、玩家金币（读 `player`）、商店表顺序（读 `shops_key`，顺序敏感）——**全部在键里**。
- 键的**真实消费者**不是 witness，而是 `solve_global.rs:494`（传真实 `shops`）：`LAYER_MEMO` 是 thread_local，同线程内多次全局求解复用缓存 ⇒ 缺 `shops` 会串味。变异 M5b（键里塞空表）⇒ `layer_memo_key_includes_shops` 红于 `:389`（空表回放真表 7 条 goal 签名）。

注记 1（低危、设计自洽性）：`level_digest` 用 **64-bit FNV-1a 摘要**（`solve_local.rs:800-830`）代表整层内容，而同文件为「摘要可能碰撞」明确拒绝给 `shops` 用摘要。概率上可忽略，但论证不能两套标准。
注记 2（覆盖面）：witness/`solve_local` 链投的是 `&[]`（`solve_local.rs:380` 的 `solve_layer`），所以**买臂在 witness 里根本不参与**——这解释了 f52 为何能在「不买任何东西」的前提下推进；`shops_key` 的价值全在 `solve_global`。这不是缺陷，但自述把「memo 键补 shops」说成 witness 的前提会误导。

### F6 LAYER_TRUNC 清理是否引入竞态 — 绿

结构（A 级代码事实）：`static LAYER_TRUNC: std::cell::Cell<Option<(i32, usize)>>`（`solve_local.rs:757`）⇒ 线程私有，跨线程无数据竞争。`take_layer_trunc()`（`:786`）是 **`get()` 只读不清零**，doc 明写清零归 `solve_layer_with` 入口（`:634`）+ 旧入口 `solve_local`（`:293`）。因此不存在「take 偷走对方标记」的语义问题，真正的风险是**遗留 Some 泄漏到下一次调用**。

自造探针（副本追加 `p4rev_f6_trunc_mark_does_not_leak_between_entries`，复用既有 `env_lock`/`LayerLimitEnv`/`fixture_floor1`）：

- A）旧入口 `limit=10` 截断 → 立刻层入口满界 ⇒ `truncated=false && ok=true`
- B）层入口 `limit=10` 截断 → 立刻旧入口满界 ⇒ `truncated=false && confidence="bounded-best"`
- C）同入口连跑两次 ⇒ 第二次不截断

```
test p4rev_f6_trunc_mark_does_not_leak_between_entries ... ok
test result: ok. 5 passed
```

敏感性自证（变异：只删旧入口的 `LAYER_TRUNC.with(|f| f.set(None));`）：

```
F6-B：层入口的 Some 不许漏给旧入口: SolveResult { ... truncated: true,
confidence: "capped-search", max_hp: Some(202), route: [...] }
panicked at crates/game-solver/tests/solve_local_trunc.rs:424
```

`max_hp:Some(202)` 恰是满界真值 ⇒ 这是**假降级**（把完整解标成 capped）⇒ 501f02a 的入口清零不是装饰，是修掉一个可观测的错误置信度。

### F7 确定性 — 黄（确定性绿，信息完整度黄）

同 budget 跨**进程**两次跑 200k：两次 sha256 均 `e8aee40c58d85597a21f9694317256daa8f68a4984bd8a4501004e8ce99bac4d` ⇒ 确定性成立。

自述「200k 与 1M 两次 sha 相同」**不成立**：

```
e8aee40c…  r200k.json  ref 2027
1e1b55f4…  r1m.json    ref 2627
90e70989…  r4m.json    ref 3127
```

三份 JSON 除 `blocked.detail` 内的 `out_hp` 数字外逐字节相同（layers=3、total_steps=196、floors 全同）——这恰恰暴露两个信息缺口：

1. **f54 参考 hp 只在自由文本里**：`"层 54 搜索撞层界（truncated=true）；out_hp=2027 只作参考，不可消费"`。结构化字段只有 `blocked.{floor,reason,layer_ok,layer_truncated}`，没有 `reference_out_hp`、也没有 `node_limit`/`nodes`。P5 回放要做「同预算可复现」比对时只能挖字符串。
2. **budget 不进 JSON** ⇒ 同一份 witness 无法自证是哪个界跑出来的（`meta` 只有 `schema/start_floor/start_entry/layers/steps_metric`）。

### F8 门禁与基线 — 绿

```
$ python3 gates/cli.py check
[✓ PASS] rust-tests — cargo test --release 全绿
[✓ PASS] test-ratchet — total=1164(基线1164) 无回退
[✓ PASS] state-machine-anchors — 锚 6 + 边数对照；全命中
[✓ PASS] asset-truth-manifest — 真相源清单校验 PASS（entries=138，art 三态核通过）
[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同
== 汇总 ==
PASS=13 FAIL=0 SKIP=0
```

- 独立复跑 `python3 tools/state_baseline.py --check` ⇒ 同样 `[GREEN] 74 场景 sha 全同`（≈11 分钟）。
- `data/status/baselines/rust-test-counts.json` 实测：`binaries` 键 77 个、`sum=1164`、`witness(test)=8`、`solve_local_trunc(test)=4`；c6a9a00 的基线 diff 只有 `solve_local_trunc 2→4`、新增 `witness 8`、`total 1154→1164`，**无任何减少**（ratchet 只拦减少项，此处无需担心基线被压低）。
- 两 commit 均未新增 `#[ignore]`（`git show 501f02a c6a9a00 | grep -c '#\[ignore\]'` = 0）；现存 6 处 ignore 均在 `game-oracle`/`game-core` 既有文件，与本次无关。

### F9 夹带/并发 — 绿

- `git show --name-status`：501f02a = `scene.rs` / `solve_local.rs` / `solve_local_trunc.rs`（3 文件）；c6a9a00 = `Cargo.lock` / `Cargo.toml` / `bin/solver.rs` / `lib.rs` / `solve_global.rs` / `witness.rs` / `tests/witness.rs` / `rust-test-counts.json`（8 文件）。与自述清单一致，无第四个逻辑变更。
- `git reflog -12`：`HEAD@{3}=7a8f29e`（amend `b1c5655 solve_local_exit`）、`HEAD@{4}=abcd026`（amend 变形球测试覆盖）——**均早于** 501f02a，未触碰本次两 commit。
- 工作树 `git status --short` 只有 3 个他人未跟踪 review 文档（`2026-11-18-*`），无跟踪文件被改；并行 agent 未混入本仓。

### F10 判据成色 — 绿

| 判据（`tests/witness.rs`） | 断言内容 | 变异能否打红 |
|---|---|---|
| `witness_chains_real_floors:225` | 真数据 f1/f2/f3 接力、`in_hp!=300`、`ents_in==sprite_full` | ✅ M8（丢 `out_player`）、M9（实体表取空表） |
| `witness_synthetic_two_floor_tower_reaches_witness:268` | 注入壳层到 `witness`、hp/gold/flags/total_steps | ❌ 抓不到 M8（它只断言 `in_player.hp`，而 `in_player` 就是 `cur_player`，写回旧值仍 307 一致） |
| `witness_refuses_to_fake_success:302` | 第二层 `ok=false` ⇒ blocked/`no_exit`/floors=1 | ✅ M1 |
| `witness_truncated_floor_is_not_consumed:351` | 截断层不可消费 | ✅ M2 |
| `witness_goal_must_point_forward:385` | `hit=false` ⇒ blocked/`no_exit_toward`/floors 空 | ✅ M3 |
| `witness_is_deterministic:420` | 双线程同 JSON | ⚠️ 功效低：同进程可重复 ≠ 跨进程确定性 |
| `witness_json_schema_stable:436` | serde round-trip + 字段出现顺序 + 词表 | ⚠️ 只验结构，不验语义 |
| `witness_up_line_comes_from_floor_graph:498` | f1→f2、f54→f1、无 ty7 ⇒ None | ✅ M4 |

P3 两条判据也各自有变异自证：M6b（旧入口恒 false）⇒ `legacy_solve_local_reports_truncation` 红于 `:244`；M5b（键塞空表）⇒ `layer_memo_key_includes_shops` 红于 `:389`。

补充观察（判据与注释不符，非缺陷但会误导读者）：`witness_goal_must_point_forward` 的夹具 `fake_exit_ok` 出口是 `(7,1)` 且走廊 `ty7 param=2` ⇒ 该边实际指向 f2 = `expect`，注释却说「解是通往 f3」；测试之所以红，靠的是壳层第二返回值 `false`，不是「方向真的错」。「截断层」夹具同时 `truncated=true && ok=false`，未覆盖「`truncated=true` 且 `ok=true`」这一交界（若将来 `materialize_exit` 契约改动，该用例可能空转）。

### X1（附加）真数据下「通关」不可达 — 黄

层图实测（`./target/release/solver floors --out /tmp/p4rev/floors.json`）：

```
49  [('StairDown',[1,11],48), ('ItemUp',[-1,-1],50), ('ItemDown',[-1,-1],48)]   ← 无 StairUp
50  [('ItemUp',[-1,-1],51), ('ItemDown',[-1,-1],49)]                            ← 无 StairUp
54  [('StairUp',[7,1],1), ('StairDown',[7,10],53), ('ItemUp',...55), ...]
```

- `Ending` 只在 **floor 50** 产出：`solve_local.rs:1933` 与 `:2041` 的 `if self.input.floor == 50 && ety == 72 { GateArrive::Ending }`。
- `up_line_next` 只认 `StairUp`（`witness.rs:313` `.filter(|e| e.kind == EdgeKind::StairUp)`）⇒ 从 f51 出发的主线是 `51→52→53→54→1→2→…→49`，到 f49 得 `blocked/"no_up_line"`，**永远进不了 f50**，因此 `status="witness"` 在真数据上不可达；成功分支目前只有合成壳层用例覆盖。

这不影响「拒绝造假」的结论（F2 绿），但它意味着：P4 交付的是一个**接力骨架 + 拒绝语义**，而不是「已能跑通关的回放」。

## 3. 变异实证表

全部在 `/tmp/p4rev-base`（`cp -a` 含 target），每次 `git checkout -- .` 回滚；基线 `cargo test -p game-solver --test witness` = 8 passed / 0.18s。

| # | 变异（文件:位置） | 期望 | 实测 |
|---|---|---|---|
| M1 | `witness.rs:470` `if !got.ok` → `if false && !got.ok` | 造通关被拦的判据红 | `witness_refuses_to_fake_success` FAILED（7 passed） |
| M2 | 「硬约束 A：截断层不可消费」整块移到 `Ending` 分支之后 | 截断判据红 | `witness_truncated_floor_is_not_consumed` FAILED |
| M3 | `witness.rs:530` `if !hit` → `if false && !hit` | 方向判据红 | `witness_goal_must_point_forward` FAILED |
| M4 | `witness.rs:313` 删 `.filter(StairUp)` | 层图判据红 | `witness_up_line_comes_from_floor_graph` FAILED |
| M5 | 9 元组删 `shops_key(shops)` | 判据红 | **编译失败**（`expected tuple with 9 elements, found 8`） |
| M5b | `shops_key(shops)` → `Vec::new()`（空表） | memo 串味判据红 | `layer_memo_key_includes_shops` FAILED @ `solve_local_trunc.rs:389` |
| M6 | 全局把 `take_layer_trunc().is_some()` 改 false | 判据红 | 首次匹配到 3 处（`:297/:481/:512`）→ 改用 M6b |
| M6b | 只改旧入口（删清零 + 恒 false） | 旧入口截断判据红 | `legacy_solve_local_reports_truncation` FAILED @ `:244` |
| M8 | `witness.rs:352` `out_player: WPlayer::from(&got.out_player)` → `from(cur_player)` | 接力判据红 | `witness_chains_real_floors` FAILED @ `tests/witness.rs:237` |
| M9 | `witness.rs:633` 首入层 `sprite_full(tlevel)` → `Vec::new()` | 实体表判据红 | `witness_chains_real_floors` FAILED |
| F6-A | 只删旧入口 `LAYER_TRUNC.set(None)` | 自造探针红 | `p4rev_f6…` FAILED @ `solve_local_trunc.rs:424`（泄漏 `capped-search` + 真值 202） |

结论：**8 条 witness 判据里 5 条被变异打红**；`witness_synthetic_two_floor_tower_reaches_witness` 抓不到 M8 是唯一实质覆盖空洞（不影响现有结论，但说明「合成塔」只验 happy path）。

## 4. 自述与实际不符处（逐条）

| 自述 | 实测 | 严重度 |
|---|---|---|
| 「`200k` 与 `1M` 两次 `sha256` 相同」 | 三档 sha 互异：`e8aee40c…` / `1e1b55f4…` / `90e70989…`（只差 `detail` 里的 `out_hp`） | 低（属自述表述错误，实现本身正确且更好——预算不同结果必须不同） |
| 「witness.rs 549 行」（任务书转述） | 实际 **651 行**（commit message 里写的是 651，正确） | 低（转述误差） |
| 「闸顺序 truncated→capped、!ok→blocked、Ending→witness、!hit→blocked」 | 中间还有两闸：`:498 no_up_line`、`:515 cloud_not_supported`；`Ending` 分支先于 `hit` 判定 | 低（省略不影响结论，但「先 Ending 后 hit」是语义决策，应写明） |
| 「`shops_key` 修复」的语境 | 键**确实**被 `solve_global` 消费；但 witness/`solve_local` 路径传 `&[]`（`:380`），买臂不参与 witness | 中（P5 若指望 witness 自己「买丹补血」会落空） |
| 「8 条 witness 判据 + M1/M2 变异必红」 | 成立；额外补 M3/M4/M8/M9 也红；但合成塔用例抓不到 M8 | 低（判据成色好于自述，除 happy path 空洞） |
| 「真数据实跑 floors=3 / total_steps=196 / blocked f54 truncated / hp 链 / 参考 2027」 | 全部逐项复现一致 | — |
| 「层界三档单调 ⇒ 真撞界」 | 成立，另有节点数=界 + queue 非空两条更强证据；4M 档节点数未插桩 | 中（4M 的「=4000000」是推断） |

## 5. 未验证清单

1. **4M 档的 `nodes/limit/goals/queue`** 未插桩（单次 25 分钟）；`nodes=limit` 对 4M 属推断（同一代码路径、同 set 点）。
2. 4M 档未再跑第二次（跨进程确定性只在 200k 上验过）。
3. `status="witness"` 的**真数据**路径未跑通（F3/X1 阻断）；该分支只有合成壳层覆盖。
4. `level_digest` 的哈希碰撞风险未量化（只作定性注记）。
5. f54 的「参考 out_hp」是否等于该层理论最优未知（层界未破 ⇒ 无上界证据）。
6. 门禁 `check` 的 `rust-tests`（`cargo test --release` 全绿）我只读其 PASS 行，未逐二进制核用例数（ratchet 已覆盖「减少」类风险）。
7. 4M 的 24m56s 墙钟受同机门禁并发影响（user 5m39s），不代表干净机器上的耗时。

## 6. 结论

**「P4 能不能作为 P5 回放的地基」**：能作**逐层接力 + 拒绝造假**的地基（接力面 A 级实证、5/8 判据可被变异打红、`capped` 与 `blocked` 语义分层干净），但**不能作「通关回放」的地基**——真数据下 `status="witness"` 不可达（f49 无 StairUp、f50 只能靠上行符、Ending 只钉在 f50），且 f54 参考 hp/budget 只活在 `detail` 自由文本里；P5 开工前应把「参考 hp + node_limit + nodes」升为结构化字段，并补 `ItemUp` 支线。

**「f54 卡点的下一步该往哪查」**：不是再抬层界（三档已证真撞界，且 4M 要 25 分钟换 +600 参考 hp，收益线性），而是**压状态/剪枝**：f54 在 1M 节点时 `goals=30391`、`queue=181257`，去重键是「pos×ents 全表×flags×三色钥×背包(item_types/item_uses/stack)×atk×def」的精确匹配，Pareto 只在同键内合并 `(hp,kills,steps,gold)` ⇒ 前沿宽度被**实体位图 × 背包 × 攻防**撑爆。下一步先给 f54 做「节点按 Key 维度分桶」的 profile（哪一维贡献最大：剩余实体差异？背包/符差异？atk/def 档位差异？），再决定是加强支配（例如按 atk/def 档位做区间支配）、还是对已拾取装备类实体做等价类合并；单纯加界是错的投入方向。
