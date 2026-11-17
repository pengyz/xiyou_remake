# P2 批1 对抗复核报告 — commit c8db72e

> 被审件：`c8db72e` refactor(core)「P2 批1 EntityService 拆小首件——HostCtx 实体面 9 方法迁移（禁双轨一次性切）」（modern 分支）
> 复核方式：对抗复核（只读；唯一写=本报告 + 复核内按任务书授权的两次变异与还原）；证据 A/B 级带实测输出；纪律=AGENTS.md
> 日期：2026-11-07 · HEAD=`c8db72e9685282eee26a9321caebd4064ba07b8c` · parent=`5c23f7142344df8c6684ef30d19ec09c18ad9d76`

## 总判：可推送（七面全绿；2 条黄-轻随批登记，不阻断）

逐字迁移 9/9 零改词零丢 doc（trait 体行级差集=恰删 9 法 0 增）、HostCtx 余 12 独立数中、4 实现者行多重集仅 +2（impl 一分为二的两行）、界收窄 3 fn 精确落位且 advance/open 纯 HostCtx 零实体调用、F4 双机理变异双通过（名单断言独立判别 + sig_lock 编译钉形）、TDD 红 9×E0599 实存、闸面独立重跑 13/13+金样 74+ratchet 985≥978、夹带恰 6 文件 144+/76- 与声称逐数相符。

---

## 面1 逐字迁移（b12 契约，最关键） — 🟢 绿

**方法**：`git show 5c23f71:crates/game-core/src/script.rs` 抽 `pub trait HostCtx` 体 → 与 HEAD `crates/game-core/src/subsystem/entity_service.rs` 抽 `pub trait EntityService` 体，按「前置 `///` doc 行 + `fn` 签名行」成对解析，9 法逐项比对（脚本实测，非人眼）：

```
GREEN cell_entity: verbatim (sig+doc)      GREEN reload_floor_entities: verbatim (sig+doc)
GREEN entity_type: verbatim (sig+doc)      GREEN spawn_entity: verbatim (sig+doc)
GREEN remove_entity: verbatim (sig+doc)    GREEN set_entity_route_flag: verbatim (sig+doc)
GREEN remove_all_of_type: verbatim (sig+doc) GREEN entity_pixel: verbatim (sig+doc)   ← 原文无 doc，迁移后同样无 doc（未加词）
GREEN swd_convert_type81: verbatim (sig+doc)
extra methods in EntityService: []        missing in old: []
```

**反向行级核（HostCtx 侧）**：旧新 trait 体行多重集差集 ——

```
--- lines only in OLD (removed)：恰 17 行 = 9 法的 8 条 doc + 9 条 fn 签名（entity_pixel 原文无 doc）
--- lines only in NEW (added)：空（trait 体零增行；新增的「批1：实体面 9 方法已迁…」doc 在 `pub trait` 之前，属 trait 注记）
```

即：**除 9 法（含其 doc）整块移出外，HostCtx trait 体零改动**；EntityService 侧 9 法签名+doc 与原文逐字同（任何改词/丢 doc 都会在上面两处任一暴露）。A 级（脚本实测输出）。

**too-many-args 冻结（执行则4）确认**：`docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md:125-129` 执行则 4 = task-15 too-many-args **8 处指名清单**（load_floor_from_save/paint_overlay_box/draw_region_scaled/register/draw_edge_patch/paint_wrapped_lines/relax_path_neighbors/tick_stepping）**冻结不追溯**；计划 `docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md:20`「不做」明列「存量静态函数签名追溯改写（too-many-args 8 处冻结）」。实测：9 法签名零改写（面1 逐字核）⇒ 冻结口径未被破坏。
（附注：执行则 4 的 8 处清单是**静态函数**，不含本次 9 个 trait 法；不改签名的直接依据其实是逐字迁移契约 + 计划「不做」项，commit 引用略宽但**结果一致=零改写**。）

## 面2 HostCtx 余 12 清单 — 🟢 绿

独立复算（任务书口径命令）：

```
$ awk '/^pub trait HostCtx/,/^}/' crates/game-core/src/script.rs | grep -c "fn "
12
$ git show 5c23f71:crates/game-core/src/script.rs | awk '/^pub trait HostCtx/,/^}/' | grep -c "fn "
21
```

余 12 清单（实抽）：`find_path / give_gold / pickup_item / snap_camera_to_player / locate_speaker_entity / set_typewriter / ces_camera / set_camera_anchor_target / exit_application / mvs_exchange / m_082_set_dialog_target / snap_camera_after_ros1`。

与计划拆解账全同（`docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md:42-49`）：实体面 10（含 find_path）、道具/经济 2、相机 4、对话 2、系统 3 = 21；减去迁出的 9 实体法 ⇒ **12 = find_path(1)+2+4+2+3**，逐项对得上，无多迁/漏迁。

**find_path 留 pathfind 域依据核**：plan:44「实体面 10：… → **批1 EntityService**（find_path 留 pathfind 域视归属）」；plan:37 EntityService 行的 9 法枚举**不含 find_path**（spawn_entity/remove_entity/remove_all_of_type/entity_pixel/cell_entity/entity_type/swd_convert_type81/reload_floor_entities/set_entity_route_flag）= 与本次迁移面严格一致。commit「find_path 留 pathfind 域不入」与计划口径一致（计划作「视归属」、commit 作「不入」，语义同向：不入本批）。B 级。

## 面3 4 实现者行为不变性 — 🟢 绿

**逐法体比对（9 法 × 4 实现者 = 36 项，脚本抽取 fn 起止大括号、空白归一后比对）**：

```
GREEN Floor1Host.{cell_entity,entity_type,remove_entity,remove_all_of_type,swd_convert_type81,
      entity_pixel,reload_floor_entities,spawn_entity,set_entity_route_flag}  9/9
GREEN StubHost   同上 9/9        GREEN ReplayHost 同上 9/9
GREEN NoopHost   同上 9/9（另核其 12 个非实体法 body 亦零变化，无 RED 输出）
```

Floor1Host 真实现（walk_replay_floor1.rs）逐字实文抽样（与 5c23f71 同）：

```rust
fn cell_entity(&self, x: i32, y: i32, t: i32) -> i32 { entity::cell_entity(&self.grid, &self.table, x, y, t) }
fn remove_entity(&mut self, idx: i32) { entity::remove(&mut self.table, idx as usize); }
fn spawn_entity(&mut self, t: i32, x: i32, y: i32) -> i32 {
    entity::spawn(&mut self.table, &mut self.grid, t, x, y, 0, &type_width_table(), None) as i32 }
```

**整体法集不变性**（脚本统计，旧=5c23f71 版、新=HEAD 版）：

```
NoopHost:   methods old=21 new=21 added=[] removed=[] body_changed=[]
StubHost:   methods old=21 new=21 added=[] removed=[] body_changed=[]
ReplayHost: methods old=21 new=21 added=[] removed=[] body_changed=[]
Floor1Host: methods old=25 new=25 added=[] removed=[] body_changed=[]
```

**「除 impl 一分为二+搬移外零行为」行级核**：把旧版该类型 impl 块与新版（HostCtx 块 + EntityService 块）非空行做多重集比对，四者差集恒为：

```
+impl EntityService for X {
+}
```

——**恰好只有新增的 impl 头尾两行**（=拆分本身），其余全部行（含每个方法的签名与体）原样重排，无增删改。A 级（脚本实测）。

## 面4 界收窄 3 fn — 🟢 绿

签名实测（HEAD `crates/game-core/src/script.rs`）：

```
195: pub fn load_level_script(&mut self, host: &mut (impl HostCtx + EntityService), idx: usize)
212: pub fn advance_dialog_page(&mut self, host: &mut impl HostCtx)            ← 未动
218: pub fn open_dialog_page(&mut self, host: &mut impl HostCtx, page: i32)   ← 未动
251: pub fn step(&mut self, host: &mut (impl HostCtx + EntityService))
272: pub fn execute_script_instruction(&mut self, host: &mut (impl HostCtx + EntityService))
```

与声称的 195/251/272 **行号逐一同**；`advance_dialog_page`/`open_dialog_page` 新旧签名字符串比对逐字相同（`pub fn advance_dialog_page(&mut self, host: &mut impl HostCtx)` old==new）。

**纯 HostCtx 依据（是否漏改的实质核）**：逐读两函数体 ——

- `advance_dialog_page`：仅 `self.open_dialog_page(host, next)`；
- `open_dialog_page`：仅 `locate_speaker_entity` / `snap_camera_to_player` / `set_typewriter`（相机+对话面，均留在 HostCtx）；**零实体面调用**。

反向全库核：9 实体法调用点全集（grep，剔除 4 实现者与 subsystem 自身）只有 `script.rs:207/315/317/318/320/350/355/357/362/368/405/435-441`（全在已收窄的 load_level_script/execute_script_instruction 内）与 `scene_persist.rs:485/495`、`script_ops.rs:110/126`、`t2_walkto_route_tests.rs:741`（`self.`/`sc.` = Scene **inherent** 方法，不经 HostCtx，与本批无关）。
`grep "impl<.*> EntityService for"` = 空（**无 blanket impl**）⇒「只绑 HostCtx 却调实体法」必然编译红；HEAD 编译绿 = 无界收窄漏改。A 级（编译器+grep 实测）。

## 面5 F4 判别力变异 — 🟢 绿

**基线（独立重跑）**：

```
$ cargo test -p game-core --release        # 任务书指定基线
exit=0；lib: test result: ok. 344 passed; 0 failed; 1 ignored; ... (running 345)
22 行 "test result: ok"，零 FAILED、零 error；p2b1 ... ok
```

**变异 A：`entity_pixel`→`entity_pixel2` 全链一致改（p2b1 名单数组刻意不动）**——改 6 文件中所有 `entity_pixel(` 形出现（trait 定义/4 实现者/调用点 `script.rs:320`/sig_lock 体内调用），`mod.rs:211` 名单项 `"entity_pixel",` 无括号故未被触碰（实测 grep 确认原样）：

```
$ cargo test -p game-core --release --no-fail-fast
error 编译期：无（编译绿）
test result: FAILED. 343 passed; 1 failed; 1 ignored; ...   ← lib 恰 1 红
failures: subsystem::tests::p2b1_entity_service_method_names
  panicked at crates/game-core/src/subsystem/mod.rs:214:13:
  EntityService 方法名漂移：entity_pixel 失联（F4 名单断言）
其余 21 个 test binary 全部 "test result: ok"（含 script_golden 6/1、entity_golden、walk_route 12 等）
```

⇒ **仅 p2b1 红、名单断言独立判别力成立**（A 级）。还原：`git checkout -- <6 文件>` 后 `git diff --quiet` ⇒ exit 0（空），`grep -rn entity_pixel2 crates/` 零命中。

**变异 B：sig_lock 形（加参数）→ 编译红**——`mod.rs:175` `_sig_entity_cell_entity` 增参 `extra: i32` 并在体内透传：

```
error[E0061]: this method takes 3 arguments but 4 arguments were supplied
   --> crates/game-core/src/subsystem/mod.rs:176:15
    176 |             t.cell_entity(x, y, type_id, extra)
note: method defined here --> crates/game-core/src/subsystem/entity_service.rs:25:8
error: could not compile `game-core` (lib test) due to 1 previous error
```

⇒ **sig_lock 编译期钉形成立**（A 级）。还原后 `git diff --quiet` = 0。
（机理说明：sig_lock 9 形位于 `#[cfg(test)] mod tests` 内、`#[allow(dead_code)] mod sig_lock`，泛型 `T: EntityService` + 体内调用 ⇒ 钉的是 **trait 法名与形参表**；9 形实数核：`grep -c "pub fn _sig_entity_" = 9`。）

## 面6 TDD 红 + 闸面 — 🟢 绿

**TDD 红证据**：

```
$ ls -l /tmp/ev_b1.txt /tmp/ev_b1_red.txt
-rw-rw-r-- 5994  /tmp/ev_b1.txt        （72 行）
-rw-rw-r-- 1014  /tmp/ev_b1_red.txt    （10 行）
$ grep -c E0599 /tmp/ev_b1.txt  → 9      $ grep -c E0599 /tmp/ev_b1_red.txt → 9
error[E0599]: no method named `cell_entity|entity_type|remove_entity|remove_all_of_type|
swd_convert_type81|entity_pixel|reload_floor_entities|spawn_entity|set_entity_route_flag`
error: could not compile `game-core` (lib test) due to 9 previous errors; 10 warnings emitted
```

9 法逐一在列（E0433=0，与 commit「E0433/E0599 红」记录一致）。A 级。

**闸面（独立重跑，非抄贴件）**：

```
$ python3 gates/cli.py check ; exit=0
[✓ PASS] rust-tests — cargo test --release 全绿          ← 即 cargo test --workspace --release（gates/cli.py:244），含 game-oracle
[✓ PASS] test-ratchet — total=985(基线978) 无回退
[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同
[✓ PASS] state-machine-anchors / asset-truth-manifest / table-content-hash / …
== 汇总 ==
PASS=13 FAIL=0 SKIP=0
```

13/13 ✓、金样 74 ✓、ratchet 985 ≥ 基线 978 无回退 ✓。A 级。

## 面7 夹带 — 🟢 绿

```
$ git show --name-only --format="" c8db72e   → 恰 6 文件
crates/game-core/src/script.rs
crates/game-core/src/subsystem/entity_service.rs
crates/game-core/src/subsystem/mod.rs
crates/game-core/tests/script_golden.rs
crates/game-oracle/tests/l2_prologue.rs
crates/game-oracle/tests/walk_replay_floor1.rs
$ git show --numstat --format="" c8db72e
22 30 script.rs | 24 2 entity_service.rs | 45 0 mod.rs
22 19 script_golden.rs | 12 9 l2_prologue.rs | 19 16 walk_replay_floor1.rs
合计 144+/76-  ← 与声称「6 文件 +144/-76」逐数相符
```

无 `data/`、无 `gates/`、无 `data/status/baselines/`、无 `reference/`、无 `original/` —— **零夹带、零基线/门禁触碰**。复核结束时 `git status --porcelain`（剔除既有 untracked 与本报告）**无任何已跟踪文件改动**（两次变异均已还原）。A 级。

---

## 遗留/黄项（随批登记，不阻断）

1. **黄-轻 Y1 · 文件头 doc 陈旧**：`crates/game-core/src/subsystem/entity_service.rs:1-3` 仍为批0 骨架措辞——`//! entity_service face（P2 批0 骨架）…（spec §3.1，…形参未定）`、`//! 骨架不预设方法签名（spec …形=参形未定，防 C 级臆造；批1 终形）`，与正文已落地的 9 个具体签名并存（「形参未定」已不成立）。仅注释、零行为；建议下批顺手改（不属逐字迁移契约面，迁移面 doc 已逐字核绿）。
2. **黄-轻 Y2 · 验证链 ②③ 计数为 debug 档**：commit 贴件「343 passed; 0 failed; 2 ignored」——独立实测 **debug=`343 passed; 0 failed; 2 ignored`**、**release=`344 passed; 0 failed; 1 ignored`**（差异源：`t2_walkto_route_tests.rs:620 #[cfg_attr(debug_assertions, ignore=…)]` 在 release 会执行，`slot_equip_scene_tests.rs:732 #[ignore]` 探针两档都忽略）。两档全绿、无用例丢失（345 总数同），但贴件数值对应 debug 档而小节语境近 release；本复核以任务书指定的 `--release` 基线（344/1）+ 闸面 workspace release 全绿为准。
3. **备注（非黄）**：执行则 4 的 too-many-args 8 处清单是静态函数面，本次 9 个 trait 法不在其中；「不改签名」的直接依据=逐字迁移契约 + 计划「不做」项（plan:20），结果与冻结口径一致（面1）。

## 复核命令索引（可复现）

- 逐字比对/法集不变性：`git show 5c23f71:<path>` ↔ 工作区，Python 抽 `impl`/`trait` 块大括号配对 + 行多重集 diff（脚本见会话记录，输出已贴）。
- 计数：`awk '/^pub trait HostCtx/,/^}/' crates/game-core/src/script.rs | grep -c "fn "` → 12。
- 基线/变异：`cargo test -p game-core --release [--no-fail-fast]`（输出 `/tmp/rev_b1/baseline_gamecore.txt`、`/tmp/rev_b1/mutA_nff.txt`、`/tmp/rev_b1/mutB.txt`）。
- 闸面：`python3 gates/cli.py check` → `/tmp/rev_b1/gates_check.txt`（exit=0，PASS=13）。
- TDD 红：`/tmp/ev_b1.txt`（72 行）、`/tmp/ev_b1_red.txt`（10 行，9×E0599）。

Confidence: 93
