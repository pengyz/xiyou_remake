# 对抗复核：批3 黄项处置方案（2026-11-14）

| 项 | 值 |
|---|---|
| 复核对象 | `docs/superpowers/proposals/2026-11-14-batch3-yellow-fix-plan.md`（330 行） |
| 对象 commit | `e4b19a6`（分支 `modern`；起草件 `88bc9f60`） |
| 上游件 | `docs/findings/reviews/2026-11-13-solver-batch3-review.md`（445 行，红0黄7） |
| 复核方式 | **只读实测**（`sed`/`grep`/`git`/`python3`），行数守恒用 `/tmp/simfix` 三文件副本模拟；**仓库 tracked 文件零改动**（`git status --porcelain \| grep -v '^??'` → 空），仅新增本报告 |
| 复核者 | 第二子 agent（对抗复核）；日期 2026-11-14 |

---

## 一、总判

### **需修订**（红 3 / 黄 4 / 绿 12）；Confidence = **92**

一句话结论：**5 项随批修的「修法本体」全部成立**——行数守恒可行（已逐处模拟）、Y4 新文案与实码逐句相符、Y7 seed/deobf 取舍有据、Y1 三数自洽、message 方案 C 所依赖的「rebase 0 钩子」断言在本机 git 2.34.1 实测为真、夹带干净、全量闸实测 PASS=13 FAIL=0 SKIP=0（≈11 分钟 >300s）。**但方案对自身所处仓库状态的 3 组「实测/判据」声明为假**：`grep '批3 报审件'` 与 `grep '上确界|支配吸收'` 的判据**没有把方案文档自己排除在检索面外**，`python3 gates/cli.py docs-links` 是**不存在的子命令**——这 3 组按方案 §8.5 自定标准（「每条命令原样跑通、输出与预期一致，否则打回」）应判红，须先修订方案再执行。

根因只有一个，三处红都是它的表现：**方案把自己写进了自己的检索面，且没有预演判据命令的 exit code**。

### 红项（必修，3）

| # | 位置 | 症状 | 实测 |
|---|---|---|---|
| **R1** | §4.3 判据 2 条 | ① `grep -rn '批3 报审件' crates/ docs/superpowers/ \| wc -l  # 0` 必然 ≠0；② `python3 gates/cli.py docs-links` 无此子命令 | ① 现值 **13**（方案自身 10 + plan 2 + state.rs 1）；笔1 正确执行后仍 = **10**（方案自身 10）≠ 0。② `exit=2`（`COMMANDS` 键表 gates/cli.py:611-629 无 `docs-links`），`# 或并入 check` 的备选有效 |
| **R2** | §6.2 Y5 记账段 | 三条声明/判据均失真：判据备注、`git log -S`、「工作树 3 命中」 | ① 判据 `grep -rn '上确界\|支配吸收' --include='*.md' . \| grep -v 2026-11-13-solver-batch3-review.md \| wc -l` 现 = **1**（命中方案自身 :292；:294 因含报告文件名被 `grep -v` 连带滤掉）≠ 0；② `git log --all -S'上确界'` / `-S'支配吸收'` 现各 = **1（e4b19a6，即方案自己的 commit）**，非 0；③ 工作树 `-E` 口径命中 **5**（报告 3 + 方案 :292/:294），非「3 全在报告」。**结论本身仍成立**（排除两份文档后 = 0，见 §7 面），但记账段必须改写 |
| **R3** | §4.1 现状实测 3 句 | 「除报告自身外恰为这 3 处」「无任何 tracked 文档含该普查」「`20/22/24` 仅命中 plan:62/165」三句「实测」为假（同一自指根因） | `grep -rn '批3 报审件'` 全仓 = **14**（方案 10 + plan 2 + state.rs 1 + 报告 1）；`git grep -ln '零获取' -- '*.md'` = plan **+ 本方案**；`git grep -n '20/22/24' -- '*.md'` = **5**（plan:62/:165 + 本方案 :183/:191/:192）；本方案 :191 自身就含 `$ python3 tools/type_census.py` 与 3 行输出 |

**统一修法建议（一次改完三红）**：所有判据与「实测」句显式排除方案自身——
`grep -rn '批3 报审件' crates/ docs/superpowers/ | grep -v 2026-11-14-batch3-yellow-fix-plan.md | wc -l  # 笔1 后 = 0`；
§4.1 三句改为「除报告自身与**本方案自身**外恰为这 3 处」；§4.3 第 5 条改 `python3 gates/cli.py check`（或直接删，因 `check` 已含 `gate_docs_links`）；§6.2 判据补 `grep -v 2026-11-14-batch3-yellow-fix-plan.md`，`git log -S` 两行改为「e4b19a6 起恒 ≥1（方案自身引入）」。

### 黄项（建议，4）

| # | 位置 | 内容 |
|---|---|---|
| **Y1** | §5.1（行 246）、§8.10（行 326） | 引报告章节号错：写「复核报告 **§0.4**」——报告**无 §0.4**（`grep -c '^### 0\.4'` = 0，只有 §0.1/§0.2）；`8361-8369` 实际出现在报告 :40（§0.2 Y7 行）、:419/:420（§11.4）。另 §5.1 写「§11.4 **两处** `plan:344`」——实为 §11.1（:382）+ §11.4（:420）**各一处**（总数 2 对，归属错） |
| **Y2** | §8 攻击面 10 点 | 漏 4 项：(a) 判据必须排除方案文档自身（§4.3/§6.2 已实际踩中）；(b) `gates/cli.py` 子命令存在性预演（`docs-links` exit 2）；(c) 方案对**报告章节号**的引用准确性（§0.4/§11.4）；(d) §6.2 `git log -S` 判据会被方案自身 commit 推翻。#5 的「每条命令原样跑通」泛覆盖了 (b)，但 (a)(c)(d) 完全未覆盖 |
| **Y3** | §3.1 行 148 | 转述报告「`tutorialSeen` **唯一写点** pickup case15」不严谨：deobf 实有 **3 处赋值**（`:7539` pickup case15；`:9299` 存档反序列化先置 false；`:9305-9306` 按 `itemStackTypes==15` 从背包容器回推 true）。功能上「背包权威源」**成立**（load 路径本身就由背包推导），但「唯一写点」作为绝对表述为假。**所幸 §3.2 的改写文案本身不含该措辞**（见绿 G5） |
| **Y4** | §3.1 行 160 | 行数守恒理由把「复核报告 pin 的行号」列成 `:449-456`——报告实际 pin 的是 **450-456**（报告 :105），449-456 是本方案的自纠值；自纠值混进「报告 pin」列表易误导（其余 `:250-260`/`:311-316`/`:383-389`/`:571-600`/`:642` 报告确有 pin ✅） |

### 绿项（已验，12）

- **G1** 报审 6 行号全真（json:63 / state.rs:39 / solve_global.rs:11 / plan:62 / plan:165 / plan:345+seed:6261）
- **G2** 方案自称的 5 处报告偏差纠正**全部为真**，且方案未引入新错锚
- **G3** 行数守恒**可行**：6 处替换逐处模拟 → 106/652/506 不变，numstat 3/3、1/1、3/3
- **G4** message 方案 C 的 hooks 前提**实测为真**（干净 rebase / rebase -i / --continue = 0 钩子）
- **G5** Y4 新文案**逐句与实码相符**，无超实主张（含「均不入 flags」）
- **G6** Y6 「闸只查磁盘不查 trackedness」**断言为真**（读 `gate_docs_links` 实现）
- **G7** Y7 推荐 option A 成立（seed 受闸保护 / deobf 可漂 / 裸锚=deobf 惯例 / 与 `71908d6` Evidence 自洽）
- **G8** Y1 更正注三数自洽（17 / 18 / lib 1 个），与 `3e7d235` message 原文逐字对应
- **G9** Y3 记账锚全真（`solve_global.rs:449-456`、`:451`、`tests/solve_global.rs:1095-1096`）
- **G10** 夹带干净：`e4b19a6` 恰 1 文件 +330 行；工作树无 tracked 改动
- **G11** 全量闸实测 `PASS=13 FAIL=0 SKIP=0`、`ratchet total=1075`、耗时 ≈11 分钟（>300s 属实）
- **G12** 笔0/笔1 顺序、红线清单、其余判据（§1.4/§2.3/§3.3/§5.3）逐条可跑

---

## 二、逐面实测（命令 + 输出摘录）

### 面 1：行号锚全量复测 —— ✅ 绿（附 Y1 黄）

**报审件引的 6 组行号（方案 §0）**

```bash
$ sed -n '63p' data/status/baselines/rust-test-counts.json
    "solve_global(test)": 17,
$ sed -n '39p' crates/game-solver/src/state.rs
/// 战斗掉落只掉金。计划 :60/:163 已原地日期更正（保留原主张不删，AGENTS §3.6）。
$ sed -n '11p' crates/game-solver/src/solve_global.rs
//! （探索区间门 4 分量入 flags）、battle-victory 脚本 69/70/71/73/74/75 转移边。   # :10=StepGate 行、:12=//! 空行 ✅
$ grep -n '见批3 报审件' docs/superpowers/plans/2026-11-09-solver-implementation-plan.md
62:…宝箱穷尽对照见批3 报审件…【2026-11-12 更正·F4】
165:…普查对照见批3 报审件…【2026-11-12 更正·F4】
$ sed -n '344p;345p' docs/superpowers/plans/2026-11-09-solver-implementation-plan.md
（344 = 空行；345 = F1 更正注整行，含裸锚 `a.java:6261`）
$ sed -n '6261p;6267p;6268p' reference/seed/a.java
            } else if (string.equals("LAY")) {
                this.q = (byte)5;
            } else if (string.equals("ROS")) {
```

**方案声称的 5 处报告偏差纠正 —— 逐条覆核，全部为真**

| 报告原写 | 方案纠为 | 实测 |
|---|---|---|
| §9.4 `plan:421` | `plan:423` | `sed -n '421p;423p' plan.md` → 421=R9 行，**423=R11 行**（含「区间门 4 分量入 `SolverState.flags`」）✅ |
| §9.4 `cloud_goals 1374-1397` | `1372-1399` | `grep -n 'fn cloud_goals' solve_local.rs` → **1372**；闭合 **1399**；:1374 item15 背包 / :1377-1386 邻梯 / :1388-1397 ty7/8 ✅ |
| §3.1 `450-456` | `449-456` | 449=`let confidence = if goals.is_empty() {`、**451**=`    } else if hit_bound \|\| node_cap_hit \|\| !truncated_layers.is_empty() {`、456=`    };` ✅ |
| §11.4/§0.2 deobf `8361-8369` | `8361-8368` | 8361=LAY if、8367=`this.dialogPhase = 5;`、8368=ROS else-if、**8369=ROS 体**（多算 1 行）✅ |
| §11.1+§11.4 `plan:344`（2 处） | `345` | `grep -n 'plan:344' 报告` = **:382（§11.1）+ :420（§11.4）**；344 空行 ✅（**归属见黄 Y1**） |

**其余锚抽验**

```bash
$ sed -n '204p;215p;217p' docs/findings/reviews/2026-11-13-solver-batch3-review.md
### 5.2 零获取穷尽对照抽验（…）✅   /   ```   /   ### 5.3 …        # §5.2 = 行 204-215 ✅
$ sed -n '1095,1096p' crates/game-solver/tests/solve_global.rs        # 「无解+触界 → feasible…」注 ✅ 与方案引文一致
$ sed -n '67p;68p;167p' crates/game-core/src/script_ops.rs             # :67 doc / :68 op_lay / :167 op_gut ✅
$ sed -n '7864p' reference/src/deobf/a.java  ->  var2 = this.changeFloor(this.currentFloor + 1, false, false);
$ sed -n '7864p' reference/seed/a.java       ->                  catch (Exception exception) {}
$ grep -n 'a\.java:9413' docs/coverage-matrix.md                       # :382 裸锚，> seed 8457 行 → 裸锚=deobf ✅
$ sed -n '174,195p' gates/cli.py                                       # gate_reference_seed_integrity（sha256+行数对基线）✅
```

**报告 pin 的 4 组 hash**（`de80ad1..da60d84` / `3e7d235` / `da60d84` / `71908d6`）`git cat-file -e` 全部存在 ✅。
**报告 pin 的 solve_global 行号集合**：`:11/:22/:78-82/:250-260/:255-259/:311-316/:383-389/:450-456/:571-600/:642`（`:571-600` = `floor_chain_dist` @ solve_global.rs:573 区，归属正确；`:642` = `to_json` 格式串，实测在位）——**这些 pin 全部依赖行数守恒**，故方案把守恒列为硬判据是必要的。

> **黄 Y1**：方案 §5.1/§8.10 引「复核报告 §0.4」——报告无此节（`grep -c '^### 0\.4'` = 0）；`8361-8369` 在报告的 3 处是 :40（§0.2 的 Y7 行）、:419/:420（§11.4）。「§11.4 两处 plan:344」应为「§11.1 一处 + §11.4 一处」。
> **黄 Y4**：§3.1 把报告 pin 的 `450-456` 写成 `449-456`（自纠值混入引用）。

### 面 2：零行为 + 行数守恒 —— ✅ 绿

方案主张：笔1 = 三文件**行数守恒原地替换**（106/652/506 不变）。**实测可行**（`/tmp/simfix` 副本逐处替换，仓库未动）：

```bash
$ python3 <按方案 §2.2/§3.2/§4.2/§5.2 原文做 6 处替换>
$ wc -l /tmp/simfix/state.rs /tmp/simfix/solve_global.rs /tmp/simfix/plan.md
  106 state.rs   652 solve_global.rs   506 plan.md        # 全部不变 ✅
$ diff 改前 改后 | 统计
state.rs        del=3  add=3   （L35、L36、L39 各 1 行换 1 行）
solve_global.rs del=1  add=1   （L11 单行）
plan.md         del=3  add=3   （L62、L165、L345 行内子串）
```

判据逐条预演（改后）：

```bash
2.3  sed -n '39p' state.rs | grep -q '计划 :62/:165'          → 命中 ✅
     grep -rn ':60/:163' crates/ | wc -l                       → 修后 0（现 1）✅
     sed -n '62p;165p' plan.md | grep -c '【2026-11-12 更正·F4】' → 2 ✅
3.3  grep -n '4 分量入 flags' solve_global.rs | wc -l          → 0 ✅
     grep -n 'cloud_goals 派生' solve_global.rs | wc -l        → 1 ✅
     sed -n '385p;386p' solve_global.rs | grep -c 'st.floors'  → 2 ✅
     wc -l solve_global.rs                                     → 652 ✅
4.3  grep -c 'reviews/2026-11-13-...-review.md' plan.md        → 2 ✅
     grep -c 'reviews/2026-11-13-...-review.md' state.rs       → 1 ✅
5.3  grep -n '；a\.java:6261' plan.md | wc -l                  → 0 ✅
     grep -n 'reference/seed/a\.java:6261-6268' plan.md | wc -l→ 1 ✅
     wc -l plan.md                                             → 506 ✅
docs-links 逻辑演练（剥围栏/行内代码后取链接目标）             → missing=[] ✅
```

**零行为**：6 处全部落在 `///`、`//!`、blockquote/列表文本行；`solve_global.rs:451`、ratchet JSON、`reference/`、`gates/`、测试零 diff（方案 §8.2 要求，本次模拟 diff 亦证实只有上述 7 行）。

### 面 3：message 决策（C vs B）—— ✅ 绿

**「rebase 0 钩子」断言抽验**（不破坏本仓，只读 + /tmp 探针，git 2.34.1）：

```bash
$ git --version                                    # git version 2.34.1
$ ls -l .git/hooks/pre-commit -> ../../gates/hooks/pre-commit   （exec "$PY" gates/cli.py check）
$ ls .git/hooks/ | grep -E 'pre-rebase|post-rewrite'            # 空（无 pre-rebase）
$ git config -l --local | grep -i 'hook\|rebase'                # 空（无覆盖）
探针（/tmp/hp2、/tmp/hookprobe）：
  git commit              → pre-commit + commit-msg  都跑
  git commit --amend      → pre-commit + commit-msg  都跑
  git rebase（干净重放）  → 0 钩子
  git rebase -i  reword   → EDITOR_RAN=1 但 0 钩子
  git rebase --continue（冲突后）→ 0 钩子
```

**判定**：方案 §1.2 对 brief 前提（「rebase 会跑 4 分钟全量闸 ×N」）的**更正成立**——reword 的真实成本 = 3 个 hash 变更 + 报告 4 组 pin 同步 + 手动 `gates check` 1 次。

**C vs B 论证是否实质满足 AGENTS §3.6**：§3.6 三要件 = 原地标注 + 日期 + 不删主张。
- C：不删史 ✅、日期更正随批入仓 ✅、`git log --grep 更正` 可查 ✅、**原地 ✗**（更正与原句跨笔）。
- 方案**如实写出**「严格意义的『原地』不满足」并给退路（lead 坚持原地 → 方案 B），且**禁用**方案 A（A 真删历史主张，违 §3.6）。
- 附带事实（本次复核新增支持）：更正注所纠正的口径错误**同笔 `rust-test-counts.json:63=17` 已自相矛盾**，读者有同笔反证 → C 的实际危害 ≈0。
- **结论**：C 是纪律内的合理选择，论证成立；「原地」一项属**留给 lead 的裁定点**，方案已显式声明，不算隐瞒。→ 绿。

### 面 4：Y7 修法（option A）—— ✅ 绿

- **seed 受保护**：`gates/cli.py:174-195 gate_reference_seed_integrity` 以 sha256 + 行数对 `data/status/baselines` 基线 → seed 行号永不失效 ✅（本次全量闸实测 `[✓ PASS] reference-seed-integrity — 8457 行，sha256 对照基线通过`）。
- **deobf 允许语义等价改动**：AGENTS §2 `reference/` 红线 = 只允许语义等价改名/拆分/格式化 → deobf 行号可再漂 ✅。
- **裸锚惯例 = deobf**：全仓裸 `a.java:NNNN` ≈3544 处；`docs/coverage-matrix.md:382` 裸 `a.java:9413-9453` **超出 seed 8457 行上限**；抽验 `deobf:7864` = F3 `case 21:` 的 changeFloor（与 `plan:178` 锚表对得上），`seed:7864` = `catch` ✅；`docs/spec/p3-render.md:111` 裸 `a.java:6261-6288` 指 entity 数组初始化 = deobf:6261 语义 ✅。
- **与该笔自洽**：`71908d6` message L8 `a.java:6261-6268` 且 `Evidence:` 段已限定 `reference/seed/a.java:6261-6268(LAY handler q=5)` + `Confidence: 95` ✅ → option A（加限定词指 seed）**不制造第三套数字、与 Evidence 同源**；方案同时列出 B 的全部代价（必须写全路径、必须写 8361-8368、行号可再漂），**取舍论证成立**。
- **陷阱实证**：`deobf:6261` = `this.entityAnimRow = new int[100];` → 裸锚按 deobf 惯例 sed 必扑空 ✅（方案已写明）。
- 附注：方案说「仓内唯一问题锚 = plan:345」——按 `a.java:6261` 这一具体数字计为真（`p3-render.md:111` 的裸锚按惯例是**正确**的）；`script_ops.rs:67` 的 `8355-8362` 漂移方案已自我标注「不属本 5 项」✅。

### 面 5：Y4 新措辞如实性 —— ✅ 绿（附 Y3 黄）

方案 §3.2 改写文案（**逐句对实码**）：

| 文案片段 | 实码 | 判 |
|---|---|---|
| `solve_local.rs:1372-1399 cloud_goals 派生` | `fn cloud_goals(&self, st: &St) -> Vec<(i32, bool)>` @1372，闭合 1399 | ✅ |
| `tutorialSeen 背包权威源` | :1374 `if !self.cloud \|\| !st.player.item_types.contains(&15) { return out; }` | ✅ |
| `f_bool_13 逐 tick 派生几何` | :1377-1386 `adj(self.stair_up) \|\| adj(self.stair_down)` 邻梯几何 | ✅ |
| `findFloorGateEntity 层数据` | :1388-1397 按 `ty==7/8` 扫 `self.level.entities` | ✅ |
| `changeFloor min/max 已归 st.floors` | solve_global.rs **:385/:386** `st.floors.iter().min()/max()` | ✅ |
| `本文件 :383-389 落闸` | :383-384 注、:387 `if target < lo \|\| target > hi {`、:388 `continue;`、:389 `}` | ✅ |
| `均不入 flags` | `state.rs:103 pub flags: ScriptFlags`（:62 注 `script_line_flags` 位图 68 槽）——四分量均非脚本位，四者由 `cloud_goals`/`st.floors` 派生，无一处写 flags | ✅ |
| 引「裁定见批3 复核报告 §9.4」 | 报告 §9.4 canonical 段在 :339-343 | ✅ |

**未超实**：文案只陈述「派生/落闸/不入 flags」，没有复述报告的「唯一写点」这类更强主张（见黄 Y3——该措辞只出现在 §3.1 的转述里）。单行长度可接受，且方案给了「压缩措辞、不拆行」的约束。

### 面 6：Y6 目标 tracked 性 —— ✅ 绿（断言）＋ 🔴 红（判据，见 R1/R3）

```bash
$ grep -n 'def gate_docs_links' gates/cli.py        # :149-171
# 源面：_tracked_md_files()  → 只遍历 tracked .md
# 目标：if re.match(r"^(https?://|mailto:|#)", t): continue
#       full = (p.parent / t.split('#')[0]).resolve(); if not full.exists() → FAIL
# 全程无 git ls-files / trackedness 判定
$ python3 gates/cli.py docs-links
（打印 __doc__ 用法）  exit=2        # COMMANDS 无此键（gates/cli.py:611-629）
```

- 断言「闸只查磁盘不查 trackedness、判据必须用 `git ls-files`」→ **真** ✅。
- 但 §4.3 的两条相关命令本身有问题 → R1；§4.1 的三句「实测」为假 → R3。

### 面 7：Y1 更正注数字自洽 + Y3/Y5 记账段 —— Y1 ✅ 绿 / Y3 ✅ 绿 / Y5 🔴 红

**Y1（绿）**

```bash
$ grep -c '#\[test\]' crates/game-solver/tests/solve_global.rs        # 17
$ cargo test --release -p game-solver --lib --bins --tests -- --list | grep -c solve_global
18
$ grep -rn 'fn solve_global_c1_run' crates/game-solver/src/
crates/game-solver/src/solve_local.rs:1980:    fn solve_global_c1_run_from自定义入场_初始位图_终态St() {
$ git log -1 --format=%B 3e7d235 | sed -n '1p;4p;6,7p'
chore(gates): 批3 C6 ratchet 入册（solve_global(test) 17→18 等）
solve_global(test)=18（C6h 节点上限测试 +1）；total=1075（基线1057→1075，
前置判据：cargo test --release -p game-solver 全绿（lib7/census4/
solve_global17→18/solve_local11）；本笔仅基线 JSON，无功能变更。
$ sed -n '63p;79p' data/status/baselines/rust-test-counts.json        # 17 / "total": 1075
```

→ 17（ratchet 键值）/18（存在性闸）/lib 内 1 个 三数互洽，与方案 §1.3 更正注模板逐字对应 ✅；§1.4 判据命令（`git log --format=%B -1 <笔1> | grep -c '…口径更正\|口径更正'`，BRE `\|` 转义正确）可跑 ✅。

**Y3（绿）**：`sed -n '449p;451p;456p' solve_global.rs` = 三元块起点/`else if hit_bound || node_cap_hit || !truncated_layers…`/闭合；`sed -n '1095,1096p' tests/solve_global.rs` 注自认「无解+触界→feasible…」与方案引文一致 ✅；记账动作（批4 补用例、本批不动 `:451`）如实 ✅。

**Y5（🔴 R2）**：

```bash
$ grep -rnE '上确界|支配吸收' --include='*.md' .            # 5
./docs/superpowers/proposals/2026-11-14-batch3-yellow-fix-plan.md:292:…
./docs/superpowers/proposals/2026-11-14-batch3-yellow-fix-plan.md:294:…
./docs/findings/reviews/2026-11-13-solver-batch3-review.md:38 / :133 / :134
$ # 方案 §6.2 原样判据（BRE 中 | 是字面量）：
$ grep -rn '上确界|支配吸收' --include='*.md' . | grep -v 2026-11-13-solver-batch3-review.md | wc -l
1                                     # ≠ 0（命中方案自身 :292）
$ git log --all -S'上确界' --oneline | wc -l
1                                     # e4b19a6（方案自己的 commit）≠ 0
$ git log --all -S'支配吸收' --oneline | wc -l
1                                     # 同上 ≠ 0
$ # 排除两份文档后：
$ grep -rnE '上确界|支配吸收' --include='*.md' . | grep -v 2026-11-13-solver-batch3-review.md | grep -v 2026-11-14-batch3-yellow-fix-plan.md | wc -l
0                                     # 结论「无落库」仍成立 ✅
```

→ 记账段三条声明失真（R2），但**「除两份文档外全仓无落库、挂批4」的结论本身仍然成立**。

### 面 8：夹带 —— ✅ 绿

```bash
$ git show --stat e4b19a6
 docs/superpowers/proposals/2026-11-14-batch3-yellow-fix-plan.md | 330 +++++++ (1 file, 330 insertions(+), 0 deletions)
$ git status --porcelain | grep -v '^??'      # 空：无 tracked 修改
```

恰 1 文件 +330 行；无代码 / plan / 台账 / 门禁 / ratchet 改动 ✅。

### 面 9：可执行性 —— ✅ 绿（除 R1 的两条命令）

```bash
$ python3 gates/cli.py check        # 本机实跑
[✓ PASS] original-integrity / assets-manifest / reference-seed-integrity(8457 行)
[✓ PASS] knowledge-format / docs-links / naming-ledger
[✓ PASS] rust-tests / test-ratchet(total=1075) / table-content-hash / table-evidence
[✓ PASS] state-machine-anchors / asset-truth-manifest / state-baseline(74 场景)
== 汇总 == PASS=13 FAIL=0 SKIP=0     EXIT=0     # 起跑 ~13:14，回收 13:25:41 ≈ 11 分钟 >300s ✅
$ timeout 240 python3 gates/cli.py status ; echo $?
124        # status 也 >240s（内部跑 gate_rust_tests 等），前置预检可行但同样耗时（提示级）
```

- **顺序**：笔0（入库报告）→ 笔1（3 文件 6 处）正确；方案已写明「笔1 不得先于笔0，否则 `git ls-files` 判据失败且 docs-links 假绿」✅。
- **红线清单**（§7 不做的事）与 §8.6 夹带面一致 ✅。
- **可跑判据**：§1.4 / §2.3 / §3.3 / §5.3 逐条实跑通过；**§4.3 两条失败（R1）**、**§6.2 判据失败（R2）**。
- message 惯例：`docs(solver): …` 与仓内近期 `docs(solver)`/`docs(plans)`/`chore(gates)` 一致 ✅。

### 面 10：攻击面自检 —— ⚠️ 黄（覆盖良好，漏 4 项，见 Y2）

方案 §8 十点已覆盖：行号锚全量、零行为、行数守恒、message 决策（含「rebase 不触发钩子」的取证要求）、判据可执行性、夹带、Y6 tracked、Y4 措辞、Y1 数字、报告 5 处偏差不照抄——**主线无遗漏**。本次复核补 4 点（并入黄 Y2）：

1. **判据必须排除方案文档自身**（§4.3 判据2、§6.2 判据已实际踩中，两处红）；
2. **子命令存在性预演**（`gates/cli.py docs-links` exit 2）——#5「每条命令原样跑通」泛覆盖但未点名，实测已踩中；
3. **方案对报告章节号的引用准确性**（`§0.4` 不存在、`§11.4 两处`归属错）；
4. **`git log -S` 类判据会被方案自身 commit 推翻**（`e4b19a6` 后恒 ≥1）。

另：§8.1 自称「起草中就抓到自己一处（初稿与报告都写 plan:344）」属起草史陈述，仓内无初稿可验，**不计入判定**。

---

## 三、执行前须完成的修订清单

1. **R1**：§4.3 判据2 加 `grep -v 2026-11-14-batch3-yellow-fix-plan.md`（或改在 `git grep` 面上跑并排除本文件），预期值改「笔1 后 = 0」；判据5 `python3 gates/cli.py docs-links` 改为 `python3 gates/cli.py check`（`check` 内含 `gate_docs_links`）或标注「该子命令不存在，只能并入 check」。
2. **R2**：§6.2 判据备注补排除本方案；`git log --all -S` 两行改为如实描述（`e4b19a6` 起 ≥1）；「工作树 3 命中全在报告」改为「5 命中（报告 3 + 本方案 2）」，并注明「排除两份文档后 = 0，结论不变」。
3. **R3**：§4.1 三句加「（除本方案自身外）」限定并更新计数（14 / 5 / plan 两处）。
4. **Y1–Y4**（建议级）：修正报告章节号引用、补 §8 攻击面 4 点、`唯一写点` 加「（存读档路径另有 2 处由背包回推的赋值）」或直接删该括注、`449-456` 改回「报告 pin 450-456（本方案纠为 449-456）」。

修订后：**5 项随批修可直接执行**（笔0 → 笔1），message 走方案 C。

---

## 四、Confidence

**92 / 100**。

- 扣分项：① 三处红的「现值」依赖方案文档已在仓内（`e4b19a6` 已入库，属既成事实，非假设）——判据在**修订前后**的期望值需 lead 确认口径；② 方案 §8.1 的起草史陈述（「初稿也写 plan:344」）仓内无初稿可验；③ 「唯一写点」的判定基于 deobf 单文件，seed 侧无 `tutorialSeen` 标识符（seed 命名系不同），未做双投影对照。
- 加分项：全部行号、判据、hooks 断言均为**本机 sed/grep/git 实跑**（非转述），行数守恒为**逐处替换模拟**（非推断），全量闸为**本机真实跑通**（PASS=13、≈11 分钟）。

*复核产出：本文件；仓库 tracked 文件零改动（`git status` 仅 untracked）。*
