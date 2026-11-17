# P2 批5 对抗复核报告 — commit 058204d

> 被审件：`058204d` refactor(core)「P2 批5 ScriptHost execute_op trait 化（脚本域写面收拢）+ 盲区登记」（4 文件 +217/-24），modern 分支
> 复核方式：对抗复核（只读；唯一写=本报告 + 任务授权的变异与还原）；证据 A/B 级带实测输出；纪律=AGENTS.md（保真=金样裁判；指针问题按批2 先例定色）
> 日期：2026-11-07 · 被审 HEAD=`058204d` · parent=`058204d^` · 复核执行=独立重跑（变异独立复做）

## 总判：可推送（核心 A 级面全绿；随批登记 3 条黄-轻 + 1 条黄挂账，不阻断）

- **面1 写面收拢保真（核心 A）=绿**：4 处写面与旧体逐行等价；a.java 真身常量/分支逐值同（`case 1/8/16→5、case 2/4→8、default→0` ↔ `1|8|16=>5, 2|4=>8, _=>0`）。附黄-轻①：引文行锚在 HEAD 不落真身（+7..+9 行漂移=批2 已披露的仓库级行锚过期），语义无一错，按批2 先例不判红。
- **面2 Recurse 执行位=绿**：GIN 同拍尾递归（script_ops.rs:576 直调，无延迟门），编排体先于递归=与旧次序等价。
- **面3 floor 读槽=绿**：`pub floor: i32` 值拷贝；3 处构造全同步。附黄-轻②：`subsystem/mod.rs:267` 注释「6 写槽+2 读槽」未随批更（现 3 读槽）。
- **面4 单源=绿**：`entity_category_with_floor` 单实现，scene.rs 薄委托 0 实现体，行为逐字等价。
- **面5 处置账表=绿**：HostCtx 余 10 fn 与 plan §1.2（21 项）逐条归宿一致，无漏项无错派。
- **面6 TDD+变异=绿**：红档 mtime 早于 commit；变异独立复做恰 1 红、还原 diff 空、复绿。
- **面7 判据重跑=绿**：p2b5 3/3、game-core --lib 355/0/2i、fixture_frames 45/45、`gates cli.py check` PASS=13 FAIL=0 SKIP=0（含 state-baseline `[GREEN] 74 场景 sha 全同`）、Java 三方 scenario-suite **47/47 PASS**（独立第二裁判，sha 与贴件一致）。
- **面8 竞态披露=黄**：披露与三跑定谳充分、金样零重冻可独立复证（绿侧）；但机制归因「同写产物目录」在 HEAD 不可复现（C 级未证实），harness 隔离缺陷按任务挂账（黄，非本笔红）。
- **面9 夹带=绿**：`git show 058204d --name-status` 恰 4 文件；修复实录五则可验项全对。

---

## 面1 写面收拢保真（核心 A 级） — 🟢 绿（附黄-轻① a.java 行锚过期/区间不全）

### ① 新体 vs 旧体逐行对（4 处写面）

新体 = `crates/game-core/src/subsystem/script_host.rs` `ScriptResolver::execute_op`（:64 起）：

```rust
*ctx.script_cursor = op.cursor();                     // 普适 cursor 写（原 execute_op 头）
ScriptOp::Gut { target, .. } => {
    let cur = *ctx.current_script_index;              // 越界守卫后写旗标
    if (cur as usize) < ctx.script_line_flags.len() { ctx.script_line_flags[cur] = true; }
    ...
ScriptOp::Ces { ty, .. } => {
    *ctx.script_delay = match entity_category_with_floor(ty, ctx.floor) {
        1 | 8 | 16 => 5, 2 | 4 => 8, _ => 0, };       // delay 三档常量
ScriptOp::Gin { .. } => ScriptStep::Recurse,          // 尾递归信号化
_ => ScriptStep::Advance,
```

旧体（`git show 058204d^:crates/game-core/src/script_ops.rs`，实测行号）：

```
$ git show 058204d^:crates/game-core/src/script_ops.rs | grep -n "set_script_delay\|script_line_flags\[cur\]\|self.execute_script_instruction()"
83:        self.execute_script_instruction();          ← op_gin 尾递归
155:        self.script_line_flags[cur] = true;         ← op_gut 旗标（无界）
247:        self.set_script_delay(match self.entity_category(ty) {
```

| 写面 | 旧体 | 新体 | 裁定 |
|---|---|---|---|
| cursor | `execute_op` 头 `set_script_cursor(op.cursor())`（旧:545） | `*ctx.script_cursor = op.cursor()`（头） | 同位同值 ✓ |
| GUT 旗标 | `script_line_flags[cur] = true`（旧:155-156，无界） | 同写 + 越界守卫 | **差异点 (a)**：守卫为新增；`flags.len()==scripts.len()==68` 恒等且 `load_level_script` 越界先 panic ⇒ 分支不可达，值等价 ✓ |
| CES delay | `set_script_delay(match entity_category(ty) {1|8|16=>5,2|4=>8,_=>0})`（旧:247-251） | `match entity_category_with_floor(ty, ctx.floor)` 同三档 | 三档常量逐字同 ✓；**差异点 (b)**：delay/cursor 写由 op 体尾移到 `execute_op` 头——同拍内无交错读（op_ces 不改 `persist.floor`），终值等价 ✓ |
| GIN 递归 | `op_gin` 尾 `self.execute_script_instruction()`（旧:83） | `ScriptStep::Recurse` → Coordinator :576 直调 | 见面2 ✓ |

`scene.rs` setter 均纯赋值（`scene.rs:495-497`），无隐式副作用。

### ② a.java 三锚 sed 实文（canonical=`reference/src/deobf/a.java`，10889 行；seed/analysis 各副本均混淆无标记）

**真身（A 级）**：

```
$ sed -n '8409,8430p' reference/src/deobf/a.java
               byte var10001;
               switch (this.renderCategoryByType[var4]) {
                  case 1: case 8: case 16:  var10001 = 5; break;
                  case 2: case 4:           var10001 = 8; break;
                  default:                  var10001 = 0;
               }
               this.scriptStepDelay = var10001;
               this.scriptCursor = this.lastDelimPos + 1;
            } else if (var3.equals("GIN")) {
```

- CES 层覆写真身=**8414（switch）→8428（scriptStepDelay 写）**；Rust `1|8|16=>5, 2|4=>8, _=>0` ↔ Java `case1/8/16→5、case2/4→8、default→0` **逐值相同** ✓
- GIN 尾递归真身=**8442 `scriptCursor=lastDelimPos+1` + 8443 `executeScriptInstruction(...)`** ✓（grep 实测；8308 为 MOV miss 同型递归）
- GUT 旗标写真身=**8318** `this.scriptLineFlags[this.currentScriptIndex] = true;`（8316 `else if GUT` → 8319 `loadLevelScript`）✓

**引文行号在 HEAD 不落真身（黄-轻①）**：

| 引文（commit/script_host.rs:46/:60、批前既有注释） | HEAD sed 实文 | 真身 | 差 |
|---|---|---|---|
| `a.java:8398-8406`（ces 层覆写，含 cat 5/8/0） | 8396-8408 = CES 参数解析+spawnEntity+相机写+`if(var4>=0)`，**无 5/8/0 常量** | 常量在 8414-8428 | +7..+16；即使按 pre-drift(+9) 折算=8407-8415，仍止于 switch 头、**不覆盖 8406-8418 的 case 常量行** |
| `a.java:8435-8436`（gin 尾递归） | 8433-8445：8435-8436=`case 0: pickupItemType`，**无递归** | 8442-8443 | +7/+8 |
| `a.java:8308-8311`（GUT，批前既有） | 真身 8316-8319 | 8316-8319 | +8 |

- **根因（A 级，批2 已披露）**：`docs/findings/reviews/2026-11-07-p2-batch2-review.md:12` 记「deobf a.java 于 `dc470b2`(10-15,+3) 与 `d389ddf`(10-16,+6) 两次前移行号(+9)，指针未随更」=仓库级行锚过期；GUT 8308-8311、守卫 8398、GIN 8435-8436 均为**批前既有注释/测试**（如 `slot_equip_scene_tests.rs:1369`），本批沿用；本批**新增**的 ces 区间（`script_host.rs:60` + commit message「a.java:8398-8406 cat 1|8|16=5/2|4=8/其余 0」）即使折算 pre-drift 也未盖住常量行。
- **定色**：任务规则「常量/分支对不上=红」——在**真身**处常量/分支完全对上（上表逐值），红条件未触发；引文失效=指针问题 ⇒ 按批2 先例（`…batch2-review.md:117-121`「数值/分支无一错…仅指针失效 → 黄-轻，不判红」）定 **黄-轻①**。建议随批/批10 一次 `a.java` 行锚勘误（8398-8406→8414-8428、8435-8436→8442-8443、8308-8311→8316-8319）。

**HEAD Rust 行锚实测全对**（commit「【HEAD 实测行锚】」逐个 grep）：`script_host.rs:27/41/42/48/54/64/89/96/104`、`script_ops.rs:542/544/576/86/156/231`、`scene.rs:714/3764/4538` ✓（`script_ops.rs:84` 注引 op_gin=84 vs 实测 :83 差 1；`op_ces` 注引 251-254 vs 实测旧体 247-251 差 4——同属指针陈旧，随①一并勘误）。

---

## 面2 ScriptStep::Recurse 执行位 — 🟢 绿

```
$ sed -n '542,578p' crates/game-core/src/script_ops.rs
    pub(super) fn execute_op(&mut self, op: ScriptOp) {
        // …ctx 块调 ScriptResolver::execute_op → 写面收拢…
        let step = … match op 编排体（op_gin kind0/1 效果先执行）…;
        if matches!(step, crate::subsystem::script_host::ScriptStep::Recurse) {
            self.execute_script_instruction();   // :576-577 GIN 同拍尾递归
        }
    }
```

- 递归为**同拍直调**，无 delay/tick 门；a.java 真身 8442-8443=同拍 `cursor 更新→executeScriptInstruction`，语义一致 ✓。
- 次序等价：新体 cursor 头写 → op_gin 编排体（kind0 pickup / kind1 gainGold）→ 递归；旧体=头写 cursor → 编排体 → 体尾递归。同序 ✓。
- MOV miss 尾递归仍内联 `script_ops.rs:186`（未信号化）——本批只声称 GIN 信号化，非缺口。
- 测试：`p2b5_gin_recurse_signal` ✓（面7 3/3）。

---

## 面3 ScriptCtx floor 读槽 — 🟢 绿（附黄-轻② 计数注释未随批更）

- 字段：`pub floor: i32` **值拷贝**（非引用，无逃逸面）。
- 构造恰 3 处、全带 floor（grep 全仓 `ScriptCtx {`）：
  - `scene.rs:3764` `floor: self.persist.floor`（生产路径）
  - `subsystem/mod.rs:271` `floor: 1`（p2b0 测试构造同步）
  - `subsystem/script_host.rs:128` 测试 `floor: …`
- **黄-轻②**：`crates/game-core/src/subsystem/mod.rs:267` 注释仍写 `// ScriptCtx：6 写槽+2 读槽`——本批在紧邻 :271 插入 `floor: 1` 后读槽实为 **3**（player/entities/floor），注释未随更（同批内漏更，批2 黄3「码内注记」同类）。写槽仍 6 ⇒ `WRITE_FACE_REGISTRY` 43 断言不受影响。

---

## 面4 entity_category_with_floor 单源 — 🟢 绿

```
$ grep -n "fn entity_category" crates/game-core/src/scene.rs
4538:    fn entity_category(&self, t: i32) -> u8 {
        crate::subsystem::script_host::entity_category_with_floor(t, self.persist.floor)   // 薄委托，0 实现体
```

- 自由函数单实现（`script_host.rs:89`）；`floor_category(` 全仓仅 `resource_cfg.rs:319` 定义 + `script_host.rs:91` 调用（单点）。
- 行为等价：`floor_category(t,floor).unwrap_or_else(|| render_category(t))` 与旧 `entity_category` 逐字同；保真冻结镜像 `legacy_render_category_mirror`（`entity.rs:247`，只读）值域抽验：`1..=12=>1`（ty=1→cat1→5）、`85|86=>2`（→8）、`72|84|87=>32`（→0）与 p2b5 夹具断言一致 ✓。

---

## 面5 处置账表 vs plan §1 — 🟢 绿

- plan=`docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md` §1.2：21 项=实体 10+道具 2+相机 4+对话 2+系统 3。
- 批后 HostCtx 恰 10 fn（`awk '/trait HostCtx/,/^}/' … grep fn` 实测）：`find_path / give_gold / pickup_item / snap_camera_to_player / locate_speaker_entity / ces_camera / set_camera_anchor_target / exit_application / mvs_exchange / snap_camera_after_ros1`。
- `script_host.rs` 模块处置账表 10/10 全覆盖且归宿与 plan 一致：find_path→pathfind 域；give_gold/pickup_item→批6；相机 4→Coordinator；exit_application/mvs_exchange→平台边界 stub 留 HostCtx；locate_speaker→批3 dialog 面。**无漏项、无错派**。
- 锚实测：`script.rs:224 locate_speaker`、`script.rs:496 exit_application`、`script.rs:507 mvs_exchange` ✓。
- 盲区显式账：`SCRIPT_BLIND_ZONES`（`script_host.rs:96`，scriptLine/scriptCursor/scriptDelay + FLD 覆盖评估行）、`PLAN_NAMED_PENDING`（`:104`，=5 条 menu4+tap1）；`DIALOG_BLIND_ZONES`（`dialog.rs:71-82`）、`PERSIST_BLIND_ZONES`（`persist_codec.rs:103-108`）已登记项被 `p2b5_blind_zones_cover_plan_named` 跨域断言钉住 ✓；plan:69 点名项无 script 域项（核对属实）。

---

## 面6 TDD 红 + 变异独立复做 — 🟢 绿

- **红档**：`/tmp/ev_b5_red.txt` mtime `2026-11-07 16:29` < commit `17:26` ✓；内容 7 行（`E0432 unresolved import ScriptResolver` + 5×`E0425 PLAYER_FIXTURE/ENTITIES_FIXTURE` + 计数 16）。
- **变异独立复做**（本复核执行，非采信 commit 贴件）：

```
# crates/game-core/src/subsystem/script_host.rs:76
1 | 8 | 16 => 5,   →   1 | 8 | 16 => 6,
$ cargo test -p game-core --lib
test result: ok. 354 passed; 1 failed   ← failures 恰 subsystem::script_host::tests::p2b5_execute_op_script_writes
$ git checkout -- …/script_host.rs
md5 e828f74eea3e0a5e08cd72b9792bded6 复同 · git diff --stat 空 · p2b5 复绿 3/3
```

- 变异 → **恰 1 红**（该测独占写面钉）✓；还原 diff 空 ✓。

---

## 面7 判据独立重跑 — 🟢 绿

| 判据 | 独立重跑结果 |
|---|---|
| `cargo test -p game-core --lib p2b5` | **3/3** ✓（p2b5_blind_zones_cover_plan_named / p2b5_gin_recurse_signal / p2b5_execute_op_script_writes） |
| `cargo test -p game-core --lib` | **355 passed / 0 failed / 2 ignored** ✓ |
| `cargo test -p game-oracle --test fixture_frames` | **45 passed / 45** ✓ |
| `python3 gates/cli.py check`（权威，变异还原后，exit=0） | **PASS=13 FAIL=0 SKIP=0** ✓，含 `state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同`、`rust-tests cargo test --release 全绿`、`test-ratchet total=997(基线978) 无回退` |
| Java 三方 scenario-suite `reference/oracle/run.py --scenarios` | 见「面7 补记」 |

> 口径注记：先跑的 `gates/cli.py status`（17:36-17:51）与本复核 17:50 变异窗口重叠、且其 SKIP=1（visual-diff，P4 启用）为 status 预检口径，均**不作数**；上表 check=commit 声称口径（PASS=13 FAIL=0 SKIP=0）实测相符。

### 面7 补记（Java 三方 scenario-suite） — 🟢 PASS

本复核独立执行（`python3 reference/oracle/run.py --scenarios`，exit=0，18:08-18:22）：

```
[oracle]   A==B==C: PASS  sha=d273e5d9b34b…
[oracle] 4/4 报告 → reference/oracle/_diff/scenarios.md
[oracle] == --scenarios 结果：PASS（47/47 场景）==
```

- **47/47 PASS**、末批 `sha=3ad6ef6ad65b…` 与 commit 贴件所记 `3ad6ef6ad65b` 前缀一致 ⇒ 独立于金样（state-baseline 74）的**另一裁判**同绿；A==B==C=原版 jar / shim / T-变换三方逐字节等价。

---

## 面8 竞态披露 — 🟡 黄（披露定谳充分=绿侧；机制归因不可复现 + harness 隔离缺陷=挂账）

**披露本体（实测 commit message 原文）**：

> 【竞态假象披露】验证链并发跑批时 state_baseline 曾报「~字节漂移 preset-floor14.state」=scenario suite 与 state_baseline 同写产物目录竞态（中写快照）；solo 复跑+suite 结束后复跑双 GREEN=金样零重冻定谳。

- **三跑定谳 ✓**：事故跑（RED `~字节漂移`）+ solo GREEN + post-suite GREEN；**本复核第 3 跑独立 GREEN**（gates check 内 `state-baseline [GREEN] 74 场景 sha 全同`，变异还原后执行）⇒「金样零重冻」可独立复证：`data/status/baselines/state-dump-golden.json` 不在本 commit 4 文件内、工作树 tracked 干净。
- **机制归因在 HEAD 不可复现（C 级未证实）**：
  - `tools/state_baseline.py:102-103` 每次跑用 `tempfile.TemporaryDirectory(prefix="state-golden-")` **唯一临时目录**作 STATE_OUT，不落共享产物目录；`MANIFEST`（`data/status/baselines/state-dump-golden.json`）在 `--check` 中只读。
  - `reference/oracle/run.py:35 OUT = HERE / "_out"`——scenario suite 只写 `reference/oracle/_out/`（gitignore），且 `grep STATE_OUT` 全仓仅 `state_baseline.py:77`（设置方）与三测试文件（读取方），run.py **不设** STATE_OUT。
  - ⇒ 仓库内不存在「state_baseline 与 scenario-suite 同写产物目录」；「中写快照」亦无第二写者（`preset-floor14.state.txt` 全仓唯一写点=`preset_cases.rs:6014`，且在子进程 join 后才 `collect()`）。归因属未证实假设（可能真因=并发跑批共享 target/源码树或 manifest 写读互斥缺失）。
- **harness 隔离缺陷=黄挂账建议（按任务规定非本笔红）**：① STATE_OUT 为全局 env，任何并发 `cargo test -p game-oracle` 带同一 STATE_OUT 即互写；② 测试逐 tick `fs::write` 整文件覆盖（非原子），读方可见半截；③ 验证链并发跑批无互斥/无目录隔离约定。建议：STATE_OUT 目录加运行级唯一后缀或跑批互斥锁，并在 knowledge 登记。

---

## 面9 夹带 — 🟢 绿

```
$ git show 058204d --name-status
M  crates/game-core/src/scene.rs
M  crates/game-core/src/script_ops.rs
M  crates/game-core/src/subsystem/mod.rs
M  crates/game-core/src/subsystem/script_host.rs
```

- 恰 4 文件、全在声称域 ✓（注：任务描述称 `script_host.rs`「新建」，实测为 **M**——批0 骨架已存在，非新增文件；不构成夹带）。
- **修复实录五则可验项**：
  - ③ `#[derive(Clone, Copy)]`（`script_ops.rs:4`，commit 记 `Clone,Copy`）✓ 纯数据零行为；
  - ⑤ `cursor` 改 pub（`scene.rs:714`）+ `use crate::subsystem::script_host::ScriptHost`（`script_ops.rs:56`）✓（E0624/E0599 对应）；
  - ④ 夹具 ty 映射：测试实测 `ty:1→delay 5`（cat1，镜像 `1..=12=>1`）、`ty:85→8`（cat2，`85|86=>2`）、`ty:72→0`（cat32，`72|84|87=>32`）✓；commit 文字写「ty=2→cat1→5」而测试用 `ty=1`——两者同属 `1..=12=>1`，语义无差（文字口径小疵，随①勘误）；
  - ①② 括号归位/去重：以「现树 `cargo test` 编译+全绿」间接佐证（历史中间态不可回放）。
- 零夹带：diff 中无第 5 文件、无门禁/基线/参考版改动。

---

## 黄项清单（随批登记，不阻断）

| # | 面 | 内容 | 建议 |
|---|---|---|---|
| 黄-轻① | 面1 | a.java 行锚过期：`8398-8406`（真身常量 8414-8428，引文不覆盖）、`8435-8436`（真身 8442-8443）、`8308-8311`（真身 8316-8319）、`op_ces:251-254`（真身旧体 247-251）；根因=批2 已披露 dc470b2+d389ddf +9 未随更 | 随批/批10 一次行锚勘误；数值/分支已逐值验对，不判红 |
| 黄-轻② | 面3 | `subsystem/mod.rs:267`「6 写槽+2 读槽」→ 现 3 读槽未随批更 | 下批改注 |
| 黄-轻③ | 面9 | commit 文字 `ty=2→cat1→5` vs 测试 `ty:1`（同 cat1，语义无差） | 勘误时一并 |
| 黄挂账 | 面8 | ①竞态机制归因「同写产物目录」仓库内不可复现（C 级）；②harness 隔离：STATE_OUT 全局 env + 逐 tick 非原子写 + 并发跑批无互斥 | knowledge 登记 + STATE_OUT 目录唯一化/互斥；非本笔红 |

## 判据命令备忘

```bash
cargo test -p game-core --lib p2b5        # 3/3
cargo test -p game-core --lib             # 355/0/2i
cargo test -p game-oracle --test fixture_frames   # 45/45
python3 gates/cli.py check                # PASS=13 FAIL=0 SKIP=0（含 state-baseline GREEN 74）
python3 reference/oracle/run.py --scenarios       # Java 三方 47 场景（独立裁判）
```

Confidence: 92/100
