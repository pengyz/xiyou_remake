# P2 批3 对抗复核报告 — commit 80fd9d0

> 被审件：`80fd9d0` feat(core)「P2 批3 DialogSubsystem」（8 文件 +209/-36），modern 分支
> 复核方式：对抗复核（只读；唯一写=本报告 + 任务书授权的变异与还原）；证据 A/B 级带实测输出；纪律=AGENTS.md（保真=金样裁判；证伪更正文化=日期更正不删原文）
> 日期：2026-11-07 · 被审 HEAD=`80fd9d0` · parent=`760a1bb`

## 总判：需修复（七面绿、面4 红）

六臂等价、Confirm 域写保真、界收窄+4 实现者、spec 注记④、TDD+变异、判据独立重跑、夹带核**全绿**——`p2b3` 2/2、`--lib` 348、`fixture_frames` 45/45、金样 `[GREEN] 74 场景 sha 全同`、`gates PASS=13 FAIL=0 SKIP=0` 全部独立复跑取得；变异独立复做 Confirm 域写剥除 → `left: 2 / right: 1` 红与贴件⑤逐字同、还原 `git diff` 空；锚 `a.java:4029-4026` 在 HEAD 实文对上（case 11 域决策入口，非批2 式过期锚）。

**面4 红 = 盲区登记漏 `dialogTarget` 且零披露**：plan:69 明文将 `dialogTarget` 列入 dialog 域 P2 盲区清单（批3 plan:91 判据=「对话盲区清单登记（typewriterPos 等）…盲区登记入档」），`DIALOG_BLIND_ZONES` 8 条零点名、commit message 全文零提及、docs 无任何批3 对该缺口的披露；其直写恰 2 处（`script_ops.rs:136` op_see / `script_ops.rs:164` op_mov）均为真盲区（state_dump 无此字段，WriteFaceRow `flds=&[]`），且 plan:69 规定的兜底「不能进=夹具内断言」仅 op_mov 侧有（`entity_route_tests.rs:33`）、**op_see 侧无夹具断言**。按任务判据「漏=黄，漏且未披露=红」→ **红**。

修复面=纯清单/注记增补（不动行为，预期金样/gates 零影响）：`DIALOG_BLIND_ZONES` 增补 `dialog_target` 条目点名两写点 + 披露/补 op_see 断言；可顺带登记两条次漏点（见面4）。修复后复跑 `gates check` 即可推送。

---

## 面1 六臂等价 — 🟢 绿

**旧六臂（760a1bb `scene_dialog.rs:81-93`）vs 新 `DialogStep`（`subsystem/dialog.rs:13-26`）逐一对**：名同（Typing / SkipToLineEnd / PageNext / PagePrev / Wait / Confirm）、序同、derive 同（`Debug, Clone, Copy, PartialEq, Eq`）——`git show 760a1bb:crates/game-core/src/scene_dialog.rs` 与 HEAD 实读逐行比对一致。

**type alias 零改动实证**：

```
$ sed -n '83p' crates/game-core/src/scene_dialog.rs
pub type TypewriterStep = crate::subsystem::dialog::DialogStep;
```

- `git diff 760a1bb..80fd9d0 -- crates/game-core/src/scene_dialog.rs`：**match 臂行（scene_dialog.rs:132-155，`scene_dialog::TypewriterStep::X` 路径）零行变化**；唯一臂体变化 = Confirm 臂删 `self.dialog.set_dialog_char(1);`→改注释（该行移入 resolver，见面2）。`advance_typewriter` 本体（50-76）零改，唯一新调用方=`subsystem/dialog.rs:55`。
- 测试引用零变化：`git diff` 三个测试文件（script_golden/l2_prologue/walk_replay）仅 import + HostCtx impl 删 2 stub 方法 + 新增 `impl DialogSubsystem` 块，**断言行零变化**。
- 外部消费者=零：`git grep TypewriterStep 760a1bb` 只命中 scene_dialog.rs 内部与 subsystem/dialog.rs 注释，无第三方引用。
- 语义等价性核：Rust type alias 对 enum 变体路径透明（`TypewriterStep::Confirm` ≡ `DialogStep::Confirm`），编译绿（348 过）即证路径解析等价；alias 未使任何 match 臂路径改变（臂行 diff 为零，根本未重写）。

## 面2 Confirm 域写保真 — 🟢 绿

**同值同序**：原 Confirm 臂序 = `set_dialog_char(1)` → `open_dialog_page(page+1)` → phase6 ease 4 setter；新序 = resolver tick 内 advance 后 `set_dialog_char(1)` → 返回 → 编排面 `open_dialog_page` + ease——**值同（1）序同（域写先于开页）**。Java 对拍：

```
$ sed -n '4063,4066p' reference/src/deobf/a.java
var20.typewriterPos = 1;            ← 4063 域写=1（同值）
var20.openDialogPage(var20.dialogPage + 1);   ← 4064 开页（同序）
var20.dialogPhase = 6;
var20.easeCameraToTarget(...);
```

**锚 a.java:4029-4026 实核（HEAD）**：

```
$ sed -n '4026p;4029p' reference/src/deobf/a.java
if (var20.dialogPhase != 6 || var20.cameraAtTarget()) {    ← 4026 域步进守卫
if (var20.keyValue != 0) {                                  ← 4029 键判定入口
$ sed -n '4008,4026p' reference/src/deobf/a.java   # 外层
case 11: ... switch (var20.dialogPhase) { case 0/1/6...    ← 4026/4029 均在 run case 11 内
```

- **实文对上**：两行均存在且语义=「case 11 域步进决策」入口，与 dialog.rs 行4/11/41/129 四处注记逐字所述一致——**非批2 式过期锚**。
- 观察（不计本批判定）：锚为**入口锚非全集锚**——六臂键分派全段实为 a.java:4054-4068（case -2 翻页 / case -1 落穿 case 0 / default `typewriterPos=1`→开页→phase6→ease），不在 4026-4029 范围内；锚系既有锚（引入于 fff7805/bca52bb），本 commit 新复制 4 处。
- Java case -1 无 break 落穿与 Rust PagePrev 臂语义等价（Wait 臂空）；Java `flipOverlayPage(top±1)` 与 Rust `top±lines_per_page` 的形参形式差异属**本 commit 未触碰的既有面**（PageNext/PagePrev 臂 diff 零改），金样 74 绿证当前行为等价，不属本批。

## 面3 界收窄+4 实现者 — 🟢 绿

**4 fn 新签名**：`advance_dialog_page`/`open_dialog_page` → `impl HostCtx + DialogSubsystem`；`step`/`execute_script_instruction` → `impl HostCtx + EntityService + DialogSubsystem`。调用面 grep 全量 = script.rs 内部（:212/:254/:294）+ 三个测试文件，无其他 crate 消费——编译绿（348 + 45）即证所有调用点满足新 bound，函数体零改（diff 只动签名行）。

**4 实现者 stub 完整性**（`grep "impl HostCtx for"` 恰 4 + `impl DialogSubsystem for` 恰 5）：

| 实现者 | 位置 | 新增 impl 内容 |
|---|---|---|
| NoopHost | script.rs:526 | `tick→DialogStep::Wait; set_typewriter{}; m_082_set_dialog_target{}` |
| StubHost | script_golden.rs:51 | 同上 |
| ReplayHost | l2_prologue.rs:41 | 同上 |
| Floor1Host | walk_replay_floor1.rs:107 | 同上 |

- **原行为零变**：各 stub 体 `{}` 与迁出前 HostCtx 方法体逐字同（m_082 stub 三处 `fn m_082_set_dialog_target(&mut self, _: i32) {}` 实读确认）；diff 除 impl 块与 import 外零改动。
- `tick` 返回 `Wait` 无任何 host 侧调用方（tick 仅由 scene 的 `DialogResolver.tick` 消费）→ 零行为。
- HostCtx 12→10：trait 内 `fn` 计数=**10** ✓。

## 面4 盲区 8 条完备性 — 🔴 红（漏登记且未披露）

**行锚全对**：8 条在 `subsystem/dialog.rs:71-80`，4 个行锚实测命中——`scene.rs:859`=layout_full_text ✓、`scene.rs:1109`=layout_wrapped_text_limited ✓、`scene.rs:1828`=skip_text_to_line_end ✓、`scene.rs:1047`=open_dialog_page ✓。scene_dialog.rs 域内直写全覆盖：`dialog_phase=0`（201/214/262→条5/7）、SkipToLineEnd 臂 `set_dialog_char(135)`（条2）、ease（160-163→条6）、flip（149/152→条3）、`open_dialog_page`（157→条4）。controls.rs 的 `dialog_phase` 直写实读全在 `#[cfg(test)]` 构造内（非生产直写）。

**漏登记主证 = `dialogTarget`**：

```
$ grep -n "dialog_target = " crates/game-core/src/*.rs crates/game-core/src/**/*.rs
script_ops.rs:136:  self.dialog.dialog_target = (tx, ty);   ← op_see（SEE 指令）
script_ops.rs:164:  self.dialog.dialog_target = (mx, my);   ← op_mov（MOV 指令）
（仅此 2 处生产直写；无第三处）
```

- **plan 明文要求**：`plan:69` P2 盲区清单 dialog 域 = `typewriterPos/dialogSpeaker/dialogAnchor/dialogTarget/overlay 布局面`——五项中 `dialogTarget` 被点名须「各域迁移批随批登记」；批3 plan:91 判据=「对话盲区清单登记（typewriterPos 等）…盲区登记入档」。
- **8 条零点名**：`DIALOG_BLIND_ZONES` 8 条文本无 `dialog_target`/`dialogTarget`；条7 只点 `dialog_phase/dialog_page`，条8 只点 `typewriterPos`。
- **零披露**：`git show 80fd9d0 -s --format=%B` 全文无 `dialogTarget`/`dialog_target`；`grep -rn dialogTarget docs/` 只命中 plan:69/spec:171（需求原文）/plan:48/plan-review:88——**无任何批3 对该缺口的披露**；`docs/coverage-matrix.md` 无 dialogTarget 盲区行。
- **真盲区确认**：state_dump.rs 对话域仅 push `overlayActive/overlayKind/dialogPage/dialogPhase/overlayText`，无 dialog_target；WriteFaceRow `subsystem/mod.rs:61` `dialog_target → flds=&[]`（=不进 dump，写面归因非盲区登记）。
- **兜底亦缺**：plan:69「不能进=夹具内断言兜底」——op_mov 侧有（`entity_route_tests.rs:33/191` `assert_eq!(sc.dialog.dialog_target, (5, ROW))`），**op_see 侧无**（l2_prologue SEE 夹具只断言打开页 166 文本 `:130`，不断言 dialog_target）。

**次漏点（同面佐证，随主证一并修）**：
1. op_see phase6 ease 4 setter 簇 `script_ops.rs:145-148`——条6 只点「Confirm 臂」（scene_dialog.rs:160-163），未覆盖该簇；
2. `show_overlay_message` 的 overlay 布局面直写 `scene.rs:841-856`（`overlay_box_w/h`、`overlay_left/right_icon=`）——plan:69「overlay 布局面」项，条1 只点 layout 两 fn。

**裁定：漏 + 未披露 = 红**（任务判据）。修复=清单增补 + 披露（或补 op_see 断言），纯注记面，不动行为。

## 面5 spec 注记④合规 — 🟢 绿

- **原文保留**：spec §3.1 行101 `trait DialogSubsystem { fn tick(&mut self, ctx: &mut Ctx) -> DialogStep; }` 实读仍在、未删未改。
- **日期头格式**：spec:112 `> 【2026-11-07 更正·批3】④ DialogSubsystem 实现为 3 方法（tick+set_typewriter/m_082_set_dialog_target——Host 边界 2 方法自 HostCtx 逐字迁入，lead 批3 派工「HostCtx 再收窄的面逐字迁+界收窄」授权；tick 形不变，ctx+≤2 不涉）。原文单 tick 形不删备查。`——日期头 ✓、原地标注 ✓、显式声明原文保留 ✓，与批2 注记（spec:111）同格式同序列。
- **授权链成立**：plan:48「对话面 2：set_typewriter/m_082_set_dialog_target → 批3 DialogSubsystem」明文授权该 2 方法迁入。

## 面6 TDD+变异 — 🟢 绿

- **红档实存且早于 commit**：`/tmp/ev_b3_red.txt` mtime=2026-11-07 11:33（commit=12:05:39，早 33 分钟），15 行=14 编译错+总结行，内容与 commit 贴件①逐字同。
- **变异独立复做**：基线净（HEAD=80fd9d0、tracked 全净）→ python 精确剥除 `subsystem/dialog.rs` resolver tick 内 `if step == DialogStep::Confirm { ctx.dialog.set_dialog_char(1); }`（1 file -3 行）→ `cargo test -p game-core --lib p2b3`：

```
test p2b3_resolve_typewriter_domain ... FAILED
panicked at crates/game-core/src/subsystem/dialog.rs:168:9:
assertion `left == right` failed: Confirm 域写 dialog_char=1
  left: 2
 right: 1
test result: FAILED. 1 passed; 1 failed
```

  与贴件⑤ `left:2/right:1` **逐字同**（p2b3_dialog_step_shape 仍 pass，恰 1 红）✓。
- **还原核**：`git checkout -- crates/game-core/src/subsystem/dialog.rs` → `git diff` 空、`git status --porcelain -- crates/` 空 ✓。

## 面7 判据独立重跑 — 🟢 绿

| 判据 | 独立重跑结果 |
|---|---|
| p2b3 | `cargo test -p game-core --lib p2b3` → **2 passed**（p2b3_dialog_step_shape / p2b3_resolve_typewriter_domain） |
| game-core --lib | **348 passed; 0 failed; 2 ignored** |
| fixture_frames | `cargo test -p game-oracle --test fixture_frames` → **45 passed; 0 failed**（95.48s，exit 0） |
| 金样 74 | `python3 gates/cli.py check` 内 state-baseline 门 → **[GREEN] 金样逐字节等价：74 场景 sha 全同** |
| gates 13 | **PASS=13 FAIL=0 SKIP=0**（original-integrity / assets-manifest 138项 / reference-seed 8457行 / knowledge-format 64 / docs-links / naming-ledger / rust-tests / test-ratchet total=990 基线978 / table-content-hash / table-evidence / state-machine-anchors 锚6 / asset-truth-manifest / state-baseline） |

（旁注：另起 `cargo run -p game-oracle --release --bin state-baseline -- check` 报 `no bin target named state-baseline`——金样证据以 gates 内该门为准，不受影响。）

## 面8 夹带 — 🟢 绿

- `git show 80fd9d0 --name-only` **恰 8 文件**，全在声称域：`crates/game-core/src/scene_dialog.rs`、`crates/game-core/src/script.rs`、`crates/game-core/src/subsystem/dialog.rs`、`crates/game-core/src/subsystem/mod.rs`、`crates/game-core/tests/script_golden.rs`、`crates/game-oracle/tests/l2_prologue.rs`、`crates/game-oracle/tests/walk_replay_floor1.rs`、`docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md`（spec=面5 注记④本体）。
- **Coordinator 遗留披露 vs 实际**：commit message 披露 DialogResolver ZST 由 scene 内联构造（Coordinator 编排面仍留 scene）+ 条7 登记「dialog_phase/dialog_page 编排直写=批10 清零面」；实测 `scene_dialog.rs:201/214/262` 仍有 `dialog_phase=0` 直写（tick_dialog 编排保留）——**披露与实际相符，非夹带、非隐匿**。

---

## 修复清单（推送前）

1. **【必须·面4】** `DIALOG_BLIND_ZONES` 增补 `dialog_target` 条目，点名 `script_ops.rs:136`（op_see）与 `script_ops.rs:164`（op_mov）两直写点；并披露 op_see 侧无夹具断言（或补一条 `entity_route_tests` 式断言闭环 plan:69 兜底）。
2. **【建议·面4】** 同步登记次漏点：op_see phase6 ease 簇（`script_ops.rs:145-148`）、`show_overlay_message` box/icon 布局面（`scene.rs:841-856`）。
3. 修复后复跑 `python3 gates/cli.py check`（预期 13 全绿、金样零重冻）即可推送。

Confidence: 90/100
