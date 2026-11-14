# 批3 随批黄项处置方案（2026-11-14）

> **状态**：草案，待第二子 agent 对抗复核；**未过复核不推送**。
> **依据**：`docs/findings/reviews/2026-11-13-solver-batch3-review.md`（445 行 12 节，红0黄7；随批修5 = Y1/Y2/Y4/Y6/Y7，挂批4 = Y3/Y5）。
> **范围**：本文件只是一份处置方案（供成员执行）；起草阶段**不改代码、不改台账、不改 plan**。
> **纪律**：下文所有行号/原文均为 2026-11-14 对工作树 **sed/grep 实测**（分支 `modern`，HEAD=`71908d6`，工作树除未跟踪报告外干净），不凭记忆。实测中已抓到**复核报告自身 5 处行号偏差**（见 §8.10），故本方案对报告引用一律以实测覆核。

---

## 0. 总览与 6 个报审行号

| 项 | 位置（实测） | 现状问题 | 处置 | 性质 |
|---|---|---|---|---|
| **Y1** | `3e7d235` message L1/L4/L6-7 vs `data/status/baselines/rust-test-counts.json:63` | message 写 `solve_global(test)=18`，实值 **17**（18 = 存在性闸 `--list` 计数） | message 侧更正（§1 决策链） | 零行为 |
| **Y2** | `crates/game-solver/src/state.rs:39` | doc 引「计划 :60/:163」已漂（F4 更正注现位 plan:62/:165） | 行锚更新 | 零行为 |
| **Y4** | `crates/game-solver/src/solve_global.rs:11` | doc 写「探索区间门 4 分量入 flags」与实作不符（canonical 裁定见复核 §9.4） | doc 改为如实描述 | 零行为 |
| **Y6** | `docs/superpowers/plans/2026-11-09-solver-implementation-plan.md:62`、`:165`、`crates/game-solver/src/state.rs:35-36` | 3 处指「批3 报审件」= 对话交付物**不入库** | 改指入库物（复核报告 §5.2，**前置=报告先入库**，§4） | 零行为 |
| **Y7** | `docs/superpowers/plans/2026-11-09-solver-implementation-plan.md:345`（**注意：报告写 :344，实测 :345**）+ `71908d6` message | 裸锚 `a.java:6261` 是 **seed 行号**；仓内裸 `a.java:NNNN` 惯例 = deobf → sed 扑空 | 锚加限定词（推荐）+ message 侧更正（§5/§1） | 零行为 |
| Y3 | `crates/game-solver/src/solve_global.rs:449-456`（`|| node_cap_hit` 在 **:451**） | 变异去该支路 0 红 = 覆盖缺口 | **挂批4，不修**（§6.1 记账） | — |
| Y5 | 全仓 + 全历史 | A-lite④ 风险注记未落库 | **挂批4，不修**（§6.2 记账） | — |

**6 个报审行号（全部本会话 sed 复测通过）**：

1. `data/status/baselines/rust-test-counts.json:63` = `    "solve_global(test)": 17,`
2. `crates/game-solver/src/state.rs:39` = `/// 战斗掉落只掉金。计划 :60/:163 已原地日期更正（保留原主张不删，AGENTS §3.6）。`
3. `crates/game-solver/src/solve_global.rs:11` = `//! （探索区间门 4 分量入 flags）、battle-victory 脚本 69/70/71/73/74/75 转移边。`
4. `docs/superpowers/plans/2026-11-09-solver-implementation-plan.md:62` 与 `:165`（两行均含「见批3 报审件」，grep -o 实测命中）
5. `docs/superpowers/plans/2026-11-09-solver-implementation-plan.md:345`（71908d6 插入的 F1 更正注整行，内含裸锚 `a.java:6261`；hunk `@@ -341,6 +341,8 @@` 证实插入即在 344-345，344 为空行）
6. `reference/seed/a.java:6261` = `            } else if (string.equals("LAY")) {`；对照 `reference/src/deobf/a.java:8361` = `            if (var3.equals("LAY")) {`、`:8367` = `               this.dialogPhase = 5;`、`:8368` = `            } else if (var3.equals("ROS")) {`（→ deobf handler = **8361-8368**，复核 §11.4 写 8361-8369 多算 1 行：8369 已是 ROS 体 `var4 = this.parseScriptInt(…)`）

---

## 1. Y1（+Y7 的 message 侧）：commit message 更正的决策链

### 1.1 现状实文（实测）

`git log -1 --format=%B 3e7d235` 关键行：

- L1：`chore(gates): 批3 C6 ratchet 入册（solve_global(test) 17→18 等）`
- L4：`solve_global(test)=18（C6h 节点上限测试 +1）；total=1075（基线1057→1075，`
- L6-7：`前置判据：cargo test --release -p game-solver 全绿（lib7/census4/` / `solve_global17→18/solve_local11）；本笔仅基线 JSON，无功能变更。`

同笔 JSON 实文：`git show 3e7d235 -- data/status/baselines/rust-test-counts.json` 显示 `+ "solve_global(test)": 17,`（**新键，值=17**，非 17→18）、`game_solver(lib)` 6→7、`total` 1057→1075；现状 `rust-test-counts.json:63` = `    "solve_global(test)": 17,`。

我本会话两条独立实测：

- `grep -c '#\[test\]' crates/game-solver/tests/solve_global.rs` = **17**
- `cargo test --release -p game-solver --lib --bins --tests -- --list | grep -c solve_global` = **18**（= 17 + lib 内 1 个 `crates/game-solver/src/solve_local.rs:1980 fn solve_global_c1_run_from自定义入场_初始位图_终态St`；复核 §7.2 同口径）

即：**message 的「=18」是存在性闸计数口径，ratchet 键值口径是 17，同一笔 message 内 L6-7 又写 `solve_global17→18`——三处口径混用，L4 与同笔 JSON 自相矛盾。**

### 1.2 三个 message 侧方案的利弊与推荐

| 方案 | 做法 | 利 | 弊 |
|---|---|---|---|
| **A. reword 直接改对**（改数字为 17 或改写口径） | `git rebase -i` reword `3e7d235`，原文替换 | 读者在原笔看到正确口径 | **删历史主张，违 AGENTS §3.6「不删除历史主张」**；hash churn（见下）；且 L6-7 的 `17→18` 语义要连带重写，改写面大 |
| **B. reword 原地追加证伪更正注** | 在 `3e7d235` message 尾部追加 `> 【2026-11-14 更正】…` 块，**原句保留** | 满足 §3.6「原地 + 日期更正 + 不删史」 | **hash churn**：`3e7d235` 变 → 其后 `da60d84`、`71908d6` 重放变 hash → 复核报告 pin 的 4 组引用（§1 `de80ad1..da60d84`、§0.2/§7.2 `3e7d235`、§0.1 `da60d84`、§0.1/§11 `71908d6`）与本方案 §0 引用**全失效**，需事后编辑审计报告（审计件被事后改写）；reword 后仍需手动 `python3 gates/cli.py check` 1 次（>300s，见 §7） |
| **C. 后续笔（黄项修复笔）message 记日期更正注**（**推荐**） | 笔1 message 内追加证伪更正注，引用原笔原文 + 实测口径；原笔不动 | **零 hash churn** → 复核报告与本方案全部引用不失效，审计链完整；零额外闸；`git log --grep 更正` 可查；§3.6 实质满足（不删史、日期更正随批入仓史）；Y1 危害≈0（同笔 `json:63=17` 已自相矛盾，读者有同笔反证） | 更正与原句不在同一 message（跨笔查证，需 `git log --grep`）；严格意义的「原地」不满足 |

**关于「rebase 触 pre-commit 4 分钟全量闸 ×N」的前提——实测不成立（须向 lead 更正）**：

- 本仓 `.git/hooks/pre-commit` = `exec "$PY" gates/cli.py check`（全量闸）；`.git/hooks/commit-msg` = 仅 staged 含 `data/naming/|reference/|original/` 时强制 Evidence+Confidence；`git config -l --local` 无 hook/rebase 覆盖、无 pre-rebase hook。
- 临时仓探针（git 2.34.1，/tmp/hp3、hp4、hp5 三仓）：`git commit` → pre-commit+commit-msg 都跑；`git commit --amend` → pre-commit 跑；**`git rebase` 干净重放 → 0 钩子；`git rebase -i` reword → 0 钩子；冲突后 `git rebase --continue` → 0 钩子**。
- 结论：reword 的真实成本 = **3 个 hash 变更 + 报告引用同步 + 手动闸 1 次**，不是「4 分钟 × N」。即便如此，方案 B 的 hash churn 对**即将入库的审计报告**仍是净损失，而收益只是一处有同笔反证的口径笔误。

**推荐：方案 C（对 Y1 与 Y7 的 message 侧统一适用）**；若 lead 坚持「更正必须原地」，退 **方案 B**；**任何情况下不用方案 A**（删历史主张违 §3.6）。

### 1.3 修法（方案 C 的更正注模板，原文照贴进笔1 message）

```
证伪更正（AGENTS §3.6，2026-11-14；原笔不删，以本注为准）：
- 3e7d235 message「solve_global(test) 17→18 / =18」口径更正：ratchet 键值实测 = 17
  （data/status/baselines/rust-test-counts.json:63 = "solve_global(test)": 17,；
   grep -c '#[test]' crates/game-solver/tests/solve_global.rs = 17）。
  「18」是存在性闸计数（cargo test --release -p game-solver --lib --bins --tests
  -- --list | grep -c solve_global = 18 = 17 + lib 内 solve_local.rs:1980
  solve_global_c1_run_from… 1 个）——非 ratchet 键值。
- 71908d6 message 内联锚 a.java:6261-6268 为 seed 行号（reference/seed/a.java，
  受 gates/cli.py:175-190 reference-seed-integrity 保护）；仓内裸 a.java:NNNN
  惯例 = deobf（reference/src/deobf/a.java，同 LAY handler 实测 :8361-8368）。
  后续引用必须带文件限定词；plan:345 内联锚已同步改标（见本笔 diff）。
```

（方案 B 时：把上述块以 `> 【2026-11-14 更正】` 形式**追加**进原 message 尾部，原句逐字保留。）

### 1.4 判据

```bash
# 方案 C：更正注入库史，原笔不动
git log -1 --format=%H 3e7d235   # 应与复核报告 pin 的 hash 完全一致（未 reword）
git log --format=%B -1 <笔1> | grep -c '3e7d235.*口径更正\|口径更正'   # ≥1
git log --format=%B -1 <笔1> | grep -c 'a.java:6261-6268 为 seed 行号'  # ≥1
# 若执行方案 B（例外）：hash 变更必须与报告引用同步
git log --format=%H -1 3e7d235   # 变了 → 必须出示报告 pin 同步注 + 手动 gates check 通过记录
```

### 1.5 风险

- 纯 message，**零行为、零文件 diff**；不触发 commit-msg 证据闸（不动 data/naming/reference/original）。
- 方案 C 的唯一残余风险：读者只看 `3e7d235` 原笔看不到更正 → 用「同笔 JSON:63=17 已自相矛盾 + `git log --grep 更正`」缓解；本方案 §0 表与复核报告 §7.2 都留了口径记录。
- 方案 B 的风险：hash churn 使复核报告 4 组 pin 失效 + 本方案 §0 引用失效（若走 B，**必须在笔0 入库报告之前完成 reword 并同步改报告**，否则入库件即刻过期——顺序约束见 §7）。

---

## 2. Y2：`state.rs:39` 行锚更新

### 2.1 现状实文（sed 实测）

- `crates/game-solver/src/state.rs:39` = `/// 战斗掉落只掉金。计划 :60/:163 已原地日期更正（保留原主张不删，AGENTS §3.6）。`
- F4 更正注现位（实测）：`docs/superpowers/plans/2026-11-09-solver-implementation-plan.md:62` 与 `:165`（两行均含 `【2026-11-12 更正·F4】` 与「见批3 报审件」）。
- 复核报告 §5.1 亦 pin `plan.md:62` ✅（本会话实测一致）。
- 附：`:60/:163` 已不在（原位被 F4 插行顶走）——`grep -rn ':60/:163' crates/` 应仅此 1 处（修后 0 处）。

### 2.2 修法（单行原地替换，行数守恒）

`crates/game-solver/src/state.rs:39`：

- 改前：`/// 战斗掉落只掉金。计划 :60/:163 已原地日期更正（保留原主张不删，AGENTS §3.6）。`
- 改后：`/// 战斗掉落只掉金。计划 :62/:165 已原地日期更正（保留原主张不删，AGENTS §3.6）。`

> 注意：**只改锚数字，不改「已原地日期更正…不删」的表述**——这是 §3.6 的历史主张，不得删。行数必须守恒（`state.rs` 现 106 行；`:40 = pub const QOL_ITEMS…` 被复核报告引用，行数一变即全漂）。

### 2.3 判据

```bash
sed -n '39p' crates/game-solver/src/state.rs | grep -q '计划 :62/:165'          # 命中
grep -rn ':60/:163' crates/ | wc -l                                             # 0
sed -n '62p;165p' docs/superpowers/plans/2026-11-09-solver-implementation-plan.md \
  | grep -c '【2026-11-12 更正·F4】'                                             # 2（锚目标在位）
wc -l crates/game-solver/src/state.rs                                           # 106（不变）
```

### 2.4 风险

- 纯 doc 行，零行为；单行替换不漂任何下游行号。不碰 ratchet/门禁。

---

## 3. Y4：`solve_global.rs:11` doc 如实描述

### 3.1 现状实文（sed 实测）

- `crates/game-solver/src/solve_global.rs:11` = `//! （探索区间门 4 分量入 flags）、battle-victory 脚本 69/70/71/73/74/75 转移边。`（`:10` = `//! C3 起补：StepGate script 边（pending_floor/went_down）、筋斗云邻梯瞬移边`；`:12` = `//!` 空行）
- 实作锚（sed 实测）：
  - `solve_global.rs:383-389` 区间门：383-384 注 `// 批3 C3b 区间门（a.java:7135-7178：strict=true 只有筋斗云` / `// 调用点带；min/maxFloorReached=已访层域 ⇒ floors 极值）`；385 `let lo = st.floors.iter().min()…`；386 `let hi = …max()…`；387 `if target < lo || target > hi {`；388 `continue;`；389 `}`
  - `crates/game-solver/src/solve_local.rs:1372-1399` `fn cloud_goals(&self, st: &St) -> Vec<(i32, bool)>`（**复核 §9.4 写 1374-1397 = 偏差，实测 1372-1399**）：`:1374` `if !self.cloud || !st.player.item_types.contains(&15) { return out; }`（tutorialSeen=背包权威源）；`:1377-1386` 邻梯几何 `adj(self.stair_up) || adj(self.stair_down)`（f_bool_13 派生几何）；`:1388-1397` 按 `ty==7/8` 找实体（findFloorGateEntity=层数据）
- 需求原文（实测）：`plan:178` 4 分量表行（tutorialSeen / f_bool_13 / findFloorGateEntity / changeFloor strict + 锚 `a.java:7538-7539, 5767-5806, 7115-7133, 7139-7151, 5295-5306`）；**`plan:423` = R11 行**（`…区间门 4 分量入 SolverState.flags…`；**复核 §9.4 写 plan:421 = 偏差，:421 是 R9 行**）
- canonical 裁定（复核 §9.4 原文，实测行 341）：tutorialSeen 权威源=背包 `item_types.contains(&15)`（唯一写点 pickup case15），抄进 flags 即双源；f_bool_13 逐 tick 派生几何，存值会与重算分叉；findFloorGateEntity 是层数据非状态；changeFloor min/maxFloorReached 是全局史已归 `st.floors`；塞 flags 只污染 K₀ ScriptFlags 键域而无行为增益 → **实作 canonical、doc 措辞错（Y4 随批修）**。

### 3.2 修法（单行原地替换，行数守恒）

`solve_global.rs:11`：

- 改前：`//! （探索区间门 4 分量入 flags）、battle-victory 脚本 69/70/71/73/74/75 转移边。`
- 改后（推荐文案，仍为 1 行）：
  `//! （探索区间门 4 分量 canonical 实作：tutorialSeen 背包权威源、f_bool_13 逐 tick 派生几何、findFloorGateEntity 层数据，由 solve_local.rs:1372-1399 cloud_goals 派生；changeFloor min/max 已归 st.floors，本文件 :383-389 落闸——均不入 flags（裁定见批3 复核报告 §9.4））、battle-victory 脚本 69/70/71/73/74/75 转移边。`

> **硬约束：行数守恒**。复核报告大量 pin 本文件行号（`:250-260`、`:311-316`、`:383-389`、`:449-456`、`:571-600`、`:642`…），改 1 行为 2 行会让其全部失效。若 lead 觉得单行过长，唯一可接受的等长改写是压缩措辞，不是拆行。

### 3.3 判据

```bash
grep -n '4 分量入 flags' crates/game-solver/src/solve_global.rs | wc -l        # 0
grep -n 'cloud_goals 派生' crates/game-solver/src/solve_global.rs | wc -l      # 1
sed -n '385p;386p' crates/game-solver/src/solve_global.rs | grep -c 'st.floors' # 2（锚目标在位）
wc -l crates/game-solver/src/solve_global.rs                                  # 652（不变）
git diff <笔1> --numstat -- crates/game-solver/src/solve_global.rs            # ins==del（单行替换）
```

### 3.4 风险

- 纯 `//!` doc 行，零行为、零测试影响；行数守恒则不漂任何引用。**不改 `plan:423` R11 需求原文**（那是历史需求，偏离裁定已记录在复核 §9.4；改需求原文超出本批授权，见 §8.6）。

---

## 4. Y6：3 处证据指针改指入库物

### 4.1 现状实文（sed/grep 实测）

指向不入库报审件的 3 处（`grep -rn '批3 报审件'` 全仓命中，除报告自身外恰为这 3 处）：

1. `plan:62`：`…（sprite 1703 记录+shop_goods 17 行+GIN×4+宝箱穷尽对照见批3 报审件）；原主张保留不删（AGENTS §3.6 证伪更正）…`
2. `plan:165`：`…20/22/24 世界零获取——普查对照见批3 报审件，原清单保留不删；另 16/17/25 亦无获取来源…`
3. `crates/game-solver/src/state.rs:35-36`（两行跨行指针）：
   - L35 `/// ⇒ 收窄至可获得子集，不制造不可满足的全收集前提。A 级穷尽对照（批3 报审件`
   - L36 `/// 出处）：sprite 55 层 1703 记录普查仅 15→f1(10,11)/21→f45(2,10)/23→f19(6,3)`

目标物实测：

- 复核报告 `docs/findings/reviews/2026-11-13-solver-batch3-review.md` 当前 **UNTRACKED**（`git ls-files --error-unmatch` 报「未匹配任何 git 已知文件」）；`git ls-files docs/findings/reviews/` = 63 tracked（该目录部分报告已入库，本笔需显式 add）。
- 报告 **§5.2 = 行 204-215**：含可复现命令 `$ python3 tools/type_census.py`（`tools/type_census.py` tracked）与输出 `sprite0..54 共 1703 条记录`、`type15 → floor1 (320,352)`、`type20/22/24 全世 0 枚`、`shop_goods.json rows = 17`。
- **无任何 tracked 文档含该普查**：`grep 零获取` 仅命中 plan + 未入库报告；`20/22/24` 仅命中 plan:62/165；`docs/findings/2026-11-09-qol-passive-item-triggers.md`（tracked）无普查 → 只能指向报告 §5.2。
- 闸的盲点（实测）：`gate_docs_links`（gates/cli.py）只查 **tracked .md 内 md 链接的磁盘存在性**（剥代码围栏/行内代码），**不查目标是否 tracked** → 指向未入库文件照样过闸。**因此判据必须用 `git ls-files`，不能拿「闸绿」当入库证据。**

> 与复核报告处置列的差异（如实记录）：报告 Y6 行写「**随批4** 把 1703 记录普查命令/结果落 `docs/findings/`」。本方案按本任务口径执行「**随批修：指针改指入库物（报告 §5.2）**」——两者不冲突：指针修随批解「库内无对照命令可复现」；普查**独立**落 `docs/findings/` 仍可挂批4（届时直接引 §5.2 命令），本方案不替代它。

### 4.2 修法（前置：笔0 先把报告入库）

**前置笔0**：`git add docs/findings/reviews/2026-11-13-solver-batch3-review.md`（入库理由：本项指针的目标物 + 批3 审计件；报告无代码围栏外 md 链接，docs-links 安全）。

**3 处原地替换（行数守恒）**：

1. `plan:62`：`宝箱穷尽对照见批3 报审件）` → `宝箱穷尽对照命令与输出见 [批3 复核报告 §5.2](../../findings/reviews/2026-11-13-solver-batch3-review.md)）`
   （相对路径自 `docs/superpowers/plans/` 解析 = `docs/findings/reviews/…`，docs-links 闸会验存在性）
2. `plan:165`：`普查对照见批3 报审件，原清单保留不删` → `普查对照命令与输出见 [批3 复核报告 §5.2](../../findings/reviews/2026-11-13-solver-batch3-review.md)，原清单保留不删`
3. `state.rs:35-36`（**保持恰好 2 行**，否则 `:39/:40` 漂移）：
   - L35 改后：`/// ⇒ 收窄至可获得子集，不制造不可满足的全收集前提。A 级穷尽对照（命令与输出：`
   - L36 改后：`/// docs/findings/reviews/2026-11-13-solver-batch3-review.md §5.2）：sprite 55 层 1703 记录普查仅 15→f1(10,11)/21→f45(2,10)/23→f19(6,3)`
   （`///` 注释内纯文本路径，不涉 docs-links 闸；语义从「见报审件」改为「命令与输出见入库件 §5.2」，**不改普查数字**）

**不改**：`plan:345` 行尾的 `（F1 偏离见批3 报审）`——它是另一个报审指针，**不在本 5 项范围**（见 §8.6 夹带约束）。

### 4.3 判据

```bash
# 先决：目标已入库
git ls-files --error-unmatch docs/findings/reviews/2026-11-13-solver-batch3-review.md  # 通过
# 指针已换
grep -rn '批3 报审件' crates/ docs/superpowers/ | wc -l                                # 0
grep -c 'reviews/2026-11-13-solver-batch3-review.md' \
  docs/superpowers/plans/2026-11-09-solver-implementation-plan.md                       # 2（plan 两处）
grep -c 'reviews/2026-11-13-solver-batch3-review.md' \
  crates/game-solver/src/state.rs                                                       # 1（state.rs 一处）
# 行数守恒 + 链接可解析
wc -l docs/superpowers/plans/2026-11-09-solver-implementation-plan.md crates/game-solver/src/state.rs
                                                                                        # 506 / 106
python3 gates/cli.py docs-links   # 或并入 check；链接目标须存在（笔0 后必然存在）
```

### 4.4 风险

- 零行为（doc/注释）；顺序风险唯一：**笔1 不得先于笔0**，否则 `git ls-files` 判据失败（docs-links 闸对此盲，会假绿）。
- 若改选行内代码路径（反引号）规避链接：可过闸但不可点读——不推荐，判据里已同时要求 `grep` 命中路径字符串，两种写法都覆盖。

---

## 5. Y7：`a.java:6261` 锚歧义（仓内修法 + message 侧）

### 5.1 现状实文（sed 实测）

- **仓内唯一问题锚**：`plan:345`（**复核报告 §11.4 两处写 `plan:344`，实测 344 是空行、345 才是该行——报告 off-by-one**）：
  `…（op_lay=save_floor_state+set_pending_floor+set_transitioning，script_ops.rs:68；a.java:6261 q=5 为其渲染相位）但经既有 pending_floor 通道…`
- `71908d6` message：L8 `  （script_ops.rs:68-73 实读；a.java:6261-6268 q=5 是其渲染相位非反证）`；Evidence 段 `  reference/seed/a.java:6261-6268(LAY handler q=5);`（**已限定 seed，可查**）+ `Confidence: 95`。
- 两套行号实测：
  - seed：`reference/seed/a.java:6261` = `            } else if (string.equals("LAY")) {`、`:6267` = `                this.q = (byte)5;`、`:6268` = `            } else if (string.equals("ROS")) {` → handler = **6261-6268**（message Evidence 正确）
  - deobf：`reference/src/deobf/a.java:8361` = `            if (var3.equals("LAY")) {`、`:8367` = `               this.dialogPhase = 5;`、`:8368` = `            } else if (var3.equals("ROS")) {`、`:8369` = ROS 体 → handler = **8361-8368**（复核 §11.4/§0.4/Y7 行三处写 `8361-8369` 多算 1 行）
  - 陷阱实证：`reference/src/deobf/a.java:6261` = `      this.entityAnimRow = new int[100];` → 裸锚按 deobf 惯例 sed 必扑空
- 仓内惯例（抽验实测）：裸 `a.java:NNNN` = deobf（deobf:7864 = `var2 = this.changeFloor(this.currentFloor + 1, false, false);` 对 F3 case21；seed:7864 = `catch (Exception exception) {}`；deobf 10889 行 vs seed 8457 行；findings 引 `a.java:9413` 超 seed 上限）。
- 保护与漂移风险：seed 受 `gates/cli.py:175-190 gate_reference_seed_integrity`（sha256+行数对照 baseline）保护 → **seed 行号永不失效**；deobf 按 AGENTS §2 允许语义等价改名/格式化 → **deobf 行号可再漂**。
- 相邻既有锚漂（**不属本 5 项**，只记观察）：`crates/game-core/src/script_ops.rs:67` doc 写 `LAY_x（a.java:8355-8362）`，实测 deobf 8355-8362 **不覆盖** handler（8355 = `var17.entityAnimRow[var51] = 8;`、8360 = `} else if (!var3.equals("MVS")) {`、8361 = LAY if、8362 = m_119）；该 doc 随文件创建笔 047e434（move-only）带入 = 既有漂移，随批4/另行裁定（**本批不改代码文件**）。

### 5.2 修法：二选一，**推荐 A（加限定词指 seed）**

| 选项 | plan:345 改法 | 利 | 弊 |
|---|---|---|---|
| **A（推荐）** | `script_ops.rs:68；a.java:6261 q=5 为其渲染相位` → `script_ops.rs:68；reference/seed/a.java:6261-6268 q=5 为其渲染相位` | ① 与该笔 `Evidence:` 字段一致（`reference/seed/a.java:6261-6268`）② seed 受 reference-seed-integrity 闸保护，锚**永不失效** ③ 裸锚约定只约束裸锚，加限定词即消歧，不与既有 `a.java:NNNN`=deobf 惯例打架 ④ 避免再造一套 deobf 数字（`script_ops.rs:67` 的 8355-8362 已漂，仓内 deobf 数字已有前科） | 读者需知道 seed/deobf 是同文件两投影（可在句内括注） |
| **B（备选）** | → `script_ops.rs:68；reference/src/deobf/a.java:8361-8368 q=5 为其渲染相位` | 与「裸锚=deobf」主流约定一致，转 deobf 读者无脑可读 | **必须写全路径**（裸 `a.java:`会再入歧义）；**行号可再漂**（deobf 允许语义等价改动）；且**必须写 8361-8368，禁写 8361-8369**（复核三处 8369 实测多算 1 行） |

**推荐 A**，理由如上（闸保护 + 与本笔 Evidence 自洽 + 不制造第三套数字）。

**message 侧**：不改 `71908d6`（Evidence 已限定 seed、L8 的 `a.java:6261-6268` 在 Evidence 语境下可查），更正注按 §1.3 模板第 2 条记入笔1 message（方案 C）。

### 5.3 判据

```bash
grep -n '；a\.java:6261' docs/superpowers/plans/2026-11-09-solver-implementation-plan.md | wc -l  # 0（裸锚消除）
grep -n 'reference/seed/a\.java:6261-6268' docs/superpowers/plans/2026-11-09-solver-implementation-plan.md | wc -l  # 1
sed -n '6261p' reference/seed/a.java | grep -q 'string.equals("LAY")'                            # 锚目标在位
wc -l docs/superpowers/plans/2026-11-09-solver-implementation-plan.md                             # 506（不变）
git diff <笔1> --numstat -- docs/superpowers/plans/2026-11-09-solver-implementation-plan.md      # ins==del
# 若走 B：额外
sed -n '8361p;8368p' reference/src/deobf/a.java    # LAY if / ROS else-if；确认 8369 不被写进锚
```

### 5.4 风险

- 零行为；**不碰 `reference/` 任何文件**（seed 只读红线，改 seed = 严重违规）；单行替换行数守恒 → `plan:346` 起不漂。
- 若误选 B 且写成裸 `a.java:8361-8368`：仍违反仓内裸锚约定（扑空），判据第 1 条抓不到，需人工看 diff——列入 §8.1/§8.6。

---

## 6. 挂批4 记账（**本批不修**，只记录）

### 6.1 Y3：confidence 去 `|| node_cap_hit` 变异 0 红

- 现状实文（sed 实测）：`crates/game-solver/src/solve_global.rs:449-456` 为 confidence 三元块，`:451` = `    } else if hit_bound || node_cap_hit || !truncated_layers.is_empty() {`（复核 §3.1 写 450-456，实测块起点 449）；`:452` 注 `// R1 层界（裁定 A）：任一层被截断/节点上限触界/打界 ⇒ 探索不完整 ⇒ 不冒充 optimal`。
- 覆盖缺口：变异去掉 `|| node_cap_hit` → 0 红（复核 §8.3）；`crates/game-solver/tests/solve_global.rs:1096` 注自认：`// 无解+触界 → feasible（无解词表优先；触界降级 optimal 的路径=同` / `// hit_bound||node_cap_hit||truncated 判据，有解时才可能 optimal→bounded-best）`——现有用例只钉「无解+触界→feasible」，**「有解+node_cap_hit→bounded-best」无用例**。
- **批4 动作**：补该断言用例（属代码变更，不在本 5 项）。本批**不改** `:451`、不改测试。

### 6.2 Y5：A-lite④ 风险注记未落库

- 本会话实测：`grep -rn '上确界|支配吸收'`（md/rs/json）工作树 = **3 命中，全部在复核报告自身**（`:38`（Y5 行）、`:133`（引报审原句「须子集上确界代表或限定效果被 hp/gold 支配吸收的槽位，任意先到者保留=不健全」）、`:134`（「全仓 + 全 git 历史 grep = 0」））；`git log --all -S"上确界"` = 0、`git log --all -S"支配吸收"` = 0。
- 结论维持：**除报告自身外全仓 + 全历史无落库** → 挂批4（B 提案补录时落 `docs/findings/` 或 plan）。
- 判据备注（笔0 入库报告后）：`grep -rn '上确界|支配吸收' --include='*.md' . | grep -v 2026-11-13-solver-batch3-review.md | wc -l` 应 = 0（**须排除报告本身**，否则报告入库后恒非 0）。

---

## 7. 落笔顺序、最少笔数、每笔闸

**前置（AGENTS §5）**：`python3 gates/cli.py status` 只读预检，看清 SKIP/恒红项。

| 步骤 | 内容 | 笔数 | 闸 |
|---|---|---|---|
| **笔0** | `git add docs/findings/reviews/2026-11-13-solver-batch3-review.md` → `chore(docs): 入库批3 对抗复核报告（红0黄7，随批修5/挂批4）`；message 带 `Evidence:`（指向报告 §5.2 可复现命令 + 本方案 6 行号）+ `Confidence:` | 1 | pre-commit 自动 `python3 gates/cli.py check`（**实测 >300s**，复核 §7.3 曾转后台 job；基线 PASS=13 FAIL=0 SKIP=0、ratchet total=1075）；报告无代码围栏外 md 链接 → docs-links 安全；不动 data/naming/reference/original → commit-msg 证据闸不强制（仍建议带） |
| **笔1** | 黄项修复（3 文件、6 处原地单行/双行替换）：Y2 `state.rs:39`、Y4 `solve_global.rs:11`、Y6 `plan:62`+`plan:165`+`state.rs:35-36`、Y7 `plan:345`；message 内按 §1.3 追加 Y1+Y7 证伪更正注（方案 C）。建议 message：`docs(solver): 批3 随批黄 Y1/Y2/Y4/Y6/Y7 对齐（零行为）` | 1 | 同上自动全量闸；**行数守恒硬判据**（`wc -l` 三文件 = 106/652/506 不变，否则复核报告 pin 的 `:383/:449/:62/:165/:345` 全漂）；§2.3/§3.3/§4.3/§5.3 判据全跑；改后复跑 `cargo test --release -p game-solver`（计数应仍 17/18、ratchet total=1075 不动） |

**最少 2 笔**（笔0+笔1）。若报告在他处已入库，则 1 笔。

**若 lead 选 message 方案 B（reword）**：额外 0 笔但有硬顺序——**必须在笔0 之前**完成 reword + 同步改复核报告 4 组 hash pin（`de80ad1..da60d84`/`3e7d235`/`da60d84`/`71908d6`）+ 本方案 §0/§1 引用，再笔0 入库；reword 期间实测 **0 自动钩子**，须**手动** `python3 gates/cli.py check` 至少 1 次（>300s）。顺序颠倒 = 入库件即刻过期（§8.4）。

**不做的事（本批红线）**：不改 `reference/`（seed 只读）、不改 `gates/`、不改 ratchet JSON、不改 `solve_global.rs:451`/测试（Y3 挂批4）、不改 `script_ops.rs:67`（既有锚漂，另行裁定）、不改 plan 需求原文（`:178/:343/:423`）、不顺手改 `plan:345` 的 `（F1 偏离见批3 报审）`。

---

## 8. 待复核攻击面（第二子 agent 逐条验，10 点）

1. **行号锚全量复测**：本方案引用的每个行号逐个 `sed -n` 复跑——`json:63`、`state.rs:39`（及 `:35/:36/:40`）、`solve_global.rs:11`（及 `:383-389/:449-456/:451`）、`plan:62/:165/:345`（及 `:178/:423`）、`solve_local.rs:1372-1399`、`seed:6261/6267/6268`、`deobf:8361/8367/8368/8369`、报告 `§5.2=行204-215`。**引锚漂移是惯犯，本方案起草中就抓到自己一处**：初稿与复核报告都写 `plan:344`，实测为 **345**（344 空行；`@@ -341,6 +341,8 @@` 证实）——凡报告与实测冲突，以实测为准。
2. **零行为证明**：`git diff 笔1` 只动 `//!`/`///`/plan 文本行；`git diff 笔1 -- crates/` 内不得出现非注释行；`solve_global.rs:451`、ratchet JSON、`reference/`、`gates/` 必须零 diff。
3. **行数守恒**：`wc -l` 改前后 `state.rs=106`、`solve_global.rs=652`、`plan=506` 不变；各文件 `--numstat` ins==del。任何行数变化 → 复核报告全部行号 pin 失效 → 打回。
4. **message 决策合理性**：推荐=方案 C（不 reword）。若执行了 reword（方案 B）：核对 4 组 hash pin 是否同步更正、是否**手动**跑过 `gates check`（实测 rebase **不触发任何钩子**——成员若称「rebase 时闸跑过了」，要求出示钩子执行证据，这是本方案与 brief 前提的已知分歧点）；若走方案 A（直接改数字）→ 违 §3.6，打回。
5. **判据可执行性**：§2.3/§3.3/§4.3/§5.3/§6.2/§7 每条命令原样跑通、输出与预期一致；发现「已验证/已生效」类未执行断言 → 打回。
6. **夹带检查**：笔0/笔1 diff 不得含 Y3/Y5 修复、不得含 `script_ops.rs:67` 锚修、不得改 `plan:178/343/423` 需求原文、不得改 `plan:345` 尾部 `（F1 偏离见批3 报审）`（不在本 5 项；若改了须出示 lead 授权）、不得动测试与 ratchet。
7. **Y6 目标必须 tracked**：`git ls-files --error-unmatch docs/findings/reviews/2026-11-13-solver-batch3-review.md` 必须在笔1 前通过；**docs-links 闸不查 trackedness（实测）**，闸绿 ≠ 入库，拿闸绿当入库证据 → 打回。
8. **Y4 新措辞如实性**：对照实文核——min/max 来自 `st.floors`（`solve_global.rs:385-386`）、三分量由 `cloud_goals`（`solve_local.rs:1372-1399`：`:1374` item15 背包、`:1377-1386` 邻梯几何、`:1388-1397` ty7/8 实体）派生。新 doc 若与这些实文不符（把「入 flags」换成另一个不实描述）→ 打回。
9. **Y1 更正注数字自洽**：17（`json:63` + `grep -c '#\[test\]'` 实测）/ 18（存在性闸 `--list | grep -c` 实测）/ lib 1 个（`solve_local.rs:1980`）三数在更正注内互洽，且与 `3e7d235` message 原文逐字对应——更正注本身写错数 → 打回。
10. **复核报告自身 5 处行号偏差不得被照抄**（逐条实测覆核）：§9.4 `plan:421`→实为 **423**（R11）；§9.4 `cloud_goals 1374-1397`→实为 **1372-1399**；§3.1 `solve_global.rs:450-456`→实为 **449-456**；§11.4/§0.4/Y7 行 deobf `8361-8369`→实为 **8361-8368**（8369=ROS 体）；§11.4 两处 `plan:344`→实为 **345**。本方案已按实测修正；若执行件或报告修订沿用报告原数 → 打回。

---

*起草：批3 黄项处置方案（2026-11-14）；所有行号/原文为 HEAD=`71908d6` 工作树实测。未过对抗复核不推送。*
