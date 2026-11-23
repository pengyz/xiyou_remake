# S5 四项修复定向复核（2cda988 / f4de582 / 9949801 / 0fd170a）

> 结论一句话：**有前提可合并**——四笔把上一轮 2 红 3 黄（F4/F7/F2/F3/F1 子项）落到了可复算、
> 可变异打红的判据上；唯一未收口的是 ty75 的 `flag20` 写回（黄，已披露、当前数据不可达、该分支零覆盖）。
> 计数：**红 0 / 黄 1 / 绿 5**。Confidence：**85**（R1 产物的「重跑」属性只有 B 级——未留 run10 日志；
> 其余为 A 级代码/变异/门禁证据）。

## 0. 范围与方法

- 只读复核 HEAD `0fd170a`；主工作树未改（`git status` 仅剩两份既有未跟踪文档）。
- 变异实验在 `git worktree` 副本 `/tmp/s5fixmut`（HEAD=`0fd170a`，复用主树 target 缓存）做，
  全部还原后 `git status --porcelain` 为空；**未在主树做任何变异**。
- 未跑 `solver run --outer-seg`（8–13min）；真数据用 `docs/findings/outer-chain-real.json` 与 md。
- 门禁只跑一次：`python3 gates/cli.py check` ⇒ **PASS=13 FAIL=0 SKIP=0**（6m39.6s）；
  `python3 tools/state_baseline.py --check` ⇒ `[GREEN] 金样逐字节等价：74 场景 sha 全同`。

## 1. R1 ← 红 F4（前缀证据 JSON）：**绿**（附 B 级溯源说明）

**措辞已降级**：`docs/findings/2026-11-23-s5-convergence.md:4-6` 明确写「曾把前缀 witness 落盘写成
真 witness 首验/端到端自证，措辞已按复核要求降级」；`:146` 再次写「不许表述为『真 witness 首验』
『已自证』或『端到端』」；`:131` 的现状表述是「前缀 witness 落盘 + 十访回放绿 + 第 11 访红 +
全链 replay 未运行」。

**JSON 字段齐全且失败点可只读定位**（`docs/findings/outer-chain-real.json`）：

- `prefix_floors` 11 层、`prefix_steps`=401、`replay=null`、`floors=[]`、`total_steps=0`；
- `prefix_replay` = `{status:"fail", visit:10, floor:3, token_index:80, token:"11,11",
  stage:"route", reason:"no_match（无分支与 witness 出层态全等）", layers:11}`；
- 自定位链：`prefix_floors[10].floor==3`，其 `route.len()==81`，`route[80]=="11,11"` ——
  失败层/下标/token 在同一 JSON 内闭合；
- `reason` 与生产侧字面量逐字一致（含全角括号）：`crates/game-solver/src/solve_local.rs:4908`
  `if expect.is_some() { "no_match（无分支与 witness 出层态全等）" } else { ... }`；
- 上一轮红点「JSON 里 no_match/token_index 各 0 次」已消失：两词各出现一次且互相印证。

**是重跑还是手改**：判为**重跑产物（B 级）**，不是手改：

- 时间序：JSON mtime `2026-11-23 17:58:04` → md `17:58:43` → commit `0fd170a` `17:58:53`，
  先落盘再提交；
- `git show 0fd170a -- docs/findings/outer-chain-real.json` 的 diff 形状是「seg2/3/4
  `blocked→skipped_no_handoff`、pops `14177/314229/105254→0`、witness 对象删除」
  +「`+5695` 行插入 `prefix_floors/prefix_steps/prefix_replay`」，seg0（含 11 层 route）
  与 `/tmp/s5_chain_real9.json` 的 `witness.floors` 逐值相同——确定性重跑本就会复现；
- 手改假设要同时伪造 code 生成的字段、把三段 pops 精确归零、逐字复刻 reason——不成立；
- 但 `/tmp` 里只有 `s5_chain_real1..9.json`，**没有 run10 的 log/JSON**，
  「release 7m48s 只跑一次」不可复验 ⇒ 产物性质只能给 B 级（内容一致性充分，执行事实未留档）。

**口径核对**：`prefix_steps=401` 与 11 层 `Σ route.len()=401` 一致；
`crates/game-solver/src/witness.rs:278-283` 已声明 `total_steps`=Σ`route.len()`（航点数，
不是 tick 步数），md 没有把 401 冒充 tick 步数。

## 2. R2 ← 红 F7（md 数字）：**绿**

我从 JSON 独立数了一遍（不采信 md）：

- 层序：`[51,52,53,54,1,2,3,2,1,2,3]`；
- 回拾=**4**：f2 三访（+2）、f1 两访（+1）、f3 两访（+1）；
- `keys_out`：前 10 层均 `[0,0,0]`，末层 f3 `[0,1,0]`（1 蓝）；末层 `demand=[2,0,0]`、
  `key_shortfall.have=[0,1,0]`。

md `:58-61` 已改成同样的数字，并保留了旧错标注（「旧文『回拾=3』是重访层种数」「旧文 keys
全程 0/0/0 错」）。一致。

## 3. R3 ← 黄 F2（Terminal 末位 + 零位移）：**绿**（有一条覆盖残余提示）

**约束确实被强制**（`crates/game-solver/src/solve_local.rs`）：

- `:4640-4644 terminal_at_route_end(route,i)`：`i` 之后只允许 `gate x,y` 注记；
- `:4721-4730 gate_ok`：`Pass` 放行、`Terminal` 仅在 `terminal_at_route_end` 为真时放行，
  否则 dead 且 reason 点名「末位」；
- 四处使用：`:4787 door 臂`、`:4830 Move 臂`、`:4847 商店走臂`、`:4862 商店买臂`；
- 零位移 `:4815-4825` 只放行 `i==0`，mid-route 同位置 dead「零位移只允许 route 首位
  （入场格 token）；mid-route 同位置无效」。

**变异打红（/tmp/s5fixmut，已还原）**：

| 变异 | 结果 |
|---|---|
| MUT-A：`terminal_at_route_end` 恒 `true` | `s5_replay_terminal_only_at_route_end` **FAILED**（1 passed;1 failed） |
| MUT-B：零位移 `if i == 0` → `if true` | `s5_replay_zero_displacement_only_first_token` **FAILED**（1 passed;1 failed） |

两条测试都同时断言正例（末位 Terminal + 仅 gate 注记 ⇒ Ok；首位入场 token ⇒ Ok），
不是只测死；也就是说放宽与收紧两侧都钉住了。

**残余（提示，不升级）**：两处商店臂共用同一个 `gate_ok`，但没有 per-site 判据——
若将来只从商店臂删掉 `gate_ok`，现有测试不会红（单一 helper，风险低）。
另核对：商店臂对非 Terminal 的 `Some(_)` 仍是「丢弃该臂」，与旧 `matches!` 行为一致，
没有借修复放宽；old→new 的 `matches!(Pass|Terminal)` 全部换成了更严格的分支。

## 4. R4 ← 黄 F1 子项（ty75 撞杀不 restep）：**黄**

**引擎真值（A 级，自己读的）**：

- 击杀钩子整体在 `if self.persist.floor <= 50 { ... }` 内：`crates/game-core/src/scene.rs:3668-3699`；
- ty75 臂：`if self.try_run_scene(20, true) { self.load_level_script(20); rerun = true; }`，
  随后 `self.battle_rerun_scene = rerun;`（`scene.rs:3691-3699`）；
- restep 在 `crates/game-core/src/scene.rs:3715`：
  `if !self.battle_rerun_scene && allow_restep { self.try_step(self.facing); }`；
- `try_run_scene`（`scene.rs:915-921`）→ `rules::gate::evaluate`；`crates/game-core/src/rules/gate.rs:96`
  `let mut gate = !v.flag(param);`，param=20 无专臂 ⇒ 落到 `gate.rs:166 "_ => {}"`；
- `script_count` 恒 68：`scene.rs:4825-4827`（`scripts.len()`）+ `scene.rs:594-596`
  （「levelScriptLines=全局 68 行表，非逐层」；测试亦断言 68）⇒ 20 不越界，run=!flag20；
- 脚本 20 的内容 = `crates/game-core/tests/fixtures/level_scripts.txt` 第 21 行 **`TAK_40_40`**
  （对白，无位移）；行尾 `crates/game-core/src/scene_script.rs:64` 置
  `script_line_flags[cur]=true`。

**三问结论**：

- (a) ty75 在 floor≤50 撞杀后**不**移入被杀格：`battle_rerun_scene=true` ⇒ `:3715` 不
  `try_step(facing)`；且脚本 20 是 TAK，不改玩家位置。实现「不移位、不计步」正确。
- (b) `flag20` 之后**会翻转到 restep**：脚本 20 行尾置位后再杀 ty75，`try_run_scene` 返回
  false ⇒ `rerun=false` ⇒ 引擎执行 `try_step`。实现的谓词读数正确，但**三处 state 都不写回
  bit20**（`Search::expand` 杀怪不改 `ns.flags`；`Dp` 用入场常量 `flags0`），所以同局第二只
  及以后 ty75 会被永久建模成「不 restep」，与引擎相反。
- (c) floor>50 钩子整体不执行 ⇒ 引擎总是 restep；实现用 `floor<=50` 把它排除，正确
  （不是漏收窄，而是引擎钩子作用域本来如此）。

**判据与变异**：`s5_ty75_kill_no_restep_on_floor_le50`（f10，下一 token `"1,1"`、回放 Ok
`pos=(2,1)`）、`s5_ty75_kill_resteps_on_floor_gt50`（f53，下一 token `"2,1"`）。
MUT-E（谓词改恒 `false`）⇒ 前件红（1 passed;1 failed），主分叉有覆盖。
但 **MUT-D（删掉 `(flags >> 20) & 1 == 0` 合取项）⇒ 5 个 S5 测试全绿**
⇒ `flag20` 分支**零覆盖**。

**判黄而不是绿**：规则在「首杀」上与引擎一致（这是真修，此前是无条件 restep），
但 (i) `flag20` 分支无任何判据；(ii) 缺写回让边界不是「未观测」而是「系统性错误建模」，
只因本次数据不可达才无影响——11 层前缀 route 里**没有任何 ty75 bump**（bump 只有 41/42/43/44/45），
ty75 在 f10（链在 f4 已断、未解到）与 f53（>50），所以 run10 数字确实不受影响。
md `:111` 与 `solve_local.rs:4635` 注释都披露了「异步置旗未读回」，属已披露边界；
但 commit message 声称「§2.2c 记录边界」，§2.2c 正文其实没有这句（只在 `:111`），建议补进去。

**最小修法（不阻塞本次合并）**：命中后把 `ns.flags/cur.flags` 置 bit20（DP 改为可变 flags），
或删掉谓词里的 flag20、在文档声明「同局多只 ty75 未建模」；并补一条「第二只 ty75 必须 restep」判据。

## 5. R5 ← 黄 F3（seg2..4 口径）：**绿**

- 代码：`crates/game-solver/src/outer_chain.rs` 在主循环里
  `let chain_alive = blocked.is_none();`，随后 `if !chain_alive { push(SegReport{ status:
  "skipped_no_handoff", witness: None, pops: 0, frontier_alive: 0,
  note: "上一段断链（chain_alive=false）⇒ 本段未跑搜索；占位零值不是测量结果，不可作独立结论" }); continue; }`
  —— 后续段**确实不再调用层求解**。
- 命名：`SegReport` 已无 `killable`，改 `frontier_alive`（doc：frontier 中未判死 `!fatal`
  状态数）；残留 `killable` 仅是 `FrontierEntry.killable`（`outer.rs:615`、`outer_witness.rs:112`）
  与 `no_killable_candidate` 措辞，语义不再混用。
- JSON 实测：`seg1_f51-f54_f1-f10` blocked/pops=4553792/frontier_alive=2645/witness 实体；
  `seg2_f11-f20`、`seg3_f21-f35`、`seg4_f36-f50` 全部 `skipped_no_handoff`、pops=0、
  frontier_alive=0、`witness=null`、note 同上；段对象里没有 `killable` 键。
- 判据被加强而非替换：`s5_chain_blocked_named_and_prefix_only` 除断言 status/witness/note 外，
  还用 `Rc<Cell>` 计 seg2 层求解调用数并断言 **==0**——这是「真的没跑搜索」的可观测证据。
- 文档瑕疵（不升级）：md `:51` 把 run5 的 seg2..4 也写成 `skipped_no_handoff`
  （已标「复核修正后口径」，但同行又说旧版报 `no_killable_candidate`，历史/现状混写）；
  `:54` 仍列 pre 值 pops=14177/314229/105254，`:192` 说明现状 0——均带标注，不构成错误论断。

## 6. R6（静默放宽 / 门禁）：**绿**

- 四笔中所有以 `-` 开头的 assert/matches/expect 逐条看：只有
  （i）`assert_eq!(segments[1].status, "witness")` → 换成更强的 skipped_no_handoff +
  `witness.is_none()` + note 含「未跑搜索」+ calls==0；
  （ii）`segments.len()==2` 保留（仅改文案）；
  （iii）`.unwrap()` → `Option` 适配（语义变化所致）；
  （iv）4 处 `matches!(Pass|Terminal)` 回放宽松门 → 更严格的 `gate_ok`。
  **没有**「删断言换绿」。
- `#[ignore]`：`88902af`/`906df56`/`2cda988`/`f4de582`/`9949801`/`0fd170a` 六点均为 **8** 条，
  未增未减。
- ratchet：`data/status/baselines/rust-test-counts.json` 只增不减——
  `game_solver(lib) 87→89`、`outer_witness(test) 6→15`、`solve_local(test) 25→30`、
  `solve_local_trunc(test) 4→5`、`total 1293→1310`；`gates check` 实测
  `total=1310(基线1310) 无回退`。
- 金样：`state_baseline.py --check` 74 场景 sha 全同（不是放宽阈值换绿）。
- 无 `gates/`、`original/` 改动；新增的 `ChainCfg{max_pops:200_000, candidates:5,
  max_floors:16}` 只出现在测试夹具，不是生产阈值。

## 7. 合并结论

**有前提可合并**。F4（前缀证据入 JSON 且可只读定位）、F7（两处数字与 JSON 自洽）、
F2（回放两条约束被强制且变异打红）、F3（断链段具名 skipped_no_handoff、witness=null、
零调用）、F1 子项（ty75 首杀不 restep 与引擎一致）均成立；门禁/金样/ratchet 全绿。

前提两条：

1. 把 R4 的 `flag20` 写回缺口与「零覆盖」标成明确 TODO（§2.2c 正文补上，别只留在
   代码注释 `solve_local.rs:4635` 与 md `:111`），不要以 run10「逐项相同」声称 ty75 修正
   被真数据验证过——本次链根本没有跑到 ty75。
2. 保留「R1 产物是 B 级」的口径：下次真跑请把 stdout/日志落到 `/tmp` 或 docs 并指向它，
   否则「7m48s 重跑」永远不可复验。

---

**能作什么用**：可据此判定这四笔在 F4/F7/F2/F3/F1 子项上完成了「判据化」——前缀 JSON 可只读
定位、两条回放约束变异可打红、断链段具名且层求解零调用、md 数字与 JSON 自洽、ty75 首杀与引擎
一致；`gates check` 13 PASS/0 FAIL/0 SKIP、74 金样 sha 全同，可作合并与后续 S6 的基线。

**不能作什么用**：不能证明 run10 真是 7m48s 重跑产物（无日志，仅 B 级）；不能证明 ty75 在
「同局多杀/跨层再杀」下与引擎对齐（`flag20` 不写回、该分支零覆盖）；不能证明回放侧四处
`gate_ok` 各有独立覆盖（仅共享 helper 被变异打红）；也不能用测试全绿反推分段链已收敛——
seg1 仍在 f4 断（S6 拾取对齐面），全链 `replay=null`。
