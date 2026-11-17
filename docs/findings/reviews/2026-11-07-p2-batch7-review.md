# P2 批7 对抗复核报告 — commit 6ce6f85

> 被审件：`6ce6f85` feat(core)「P2 批7 InputMapper trait 化（RawInput→Action+apply 域写收拢）+tap 盲区销账」（6 文件 +273/-48，`git show --stat` 实测），modern 分支
> 复核方式：对抗复核（只读；唯一写=本报告 + 任务授权的变异与还原）；证据带实测输出；纪律=AGENTS.md（保真=金样裁判）
> 日期：2026-11-07 · 被审 HEAD=`6ce6f85` · 判据全部独立重跑（金样/闸/单测/变异复做）

## 总判：**可推送**（9 面 **0 红**；1 条黄-中 + 5 条黄-轻，全部零行为、不违 plan 批7 交付口径；挂账 6 条见下，①为下批必办口径更正）

| 面 | 裁定 | 一句话 |
|---|---|---|
| 1 RawInput 三变体 vs 三漏斗 | 🟡 **黄-中** | 枚举终形/三臂映射/Key 同币全绿（测试钉），但「shell 三漏斗实证[产出]」实测**不成立**：全库生产构造点=0、map 仅测试面消费、main.rs:669 与 touch Undo 两漏斗旁路不经 Action |
| 2 map 绑定单源裁定 | 🟢 绿（附挂账②） | map 三臂纯 match 无任何 key_value/controls 查询（ZST 无状态，结构性防双轨）；绑定全在 ControlButton 构造侧 controls.rs 两表；附：注释「BootMachine 测试 :362-363」标签/行锚偏 |
| 3 apply 逐臂+序等价 | 🟢 绿（附挂账③） | 七案逐臂 diff 对拍一致；**interrupt_route 实测 body scene.rs:1631-1643 `grep -c tap`=0**（序等价 A 级）；「interrupt_route:1646」三处行锚错位 |
| 4 InputStep 5 臂桥接完备 | 🟢 绿 | match 无通配=编译器强制穷尽；5 臂语义逐一对拍；Undo=Ignored→false ≡ 旧 `Action::Undo => false` |
| 5 盲区对账 | 🟢 绿（附挂账④） | `PLAN_NAMED_PENDING = &[]` 真身清空；plan:69-71 点名 **14 token（dialog5/menu4/persist4/tap1）全登记**；commit 称「13 项 dialog4」少算 overlay；t6 行锚 :1740 对（真路径 `src/scene_tests/`） |
| 6 处置账表同步 | 🟢 绿（附挂账⑤） | 余 8 / 已迁标注 / 批7 N/A 行三要素齐（批6 黄-轻②主体）；批6 点名的另两行 `:19`/`:104` 仍 stale |
| 7 判据独立重跑 | 🟢 绿 | p2b7 **4/4**、game-core --lib **363/0/2i**、fixture_frames **45/45**、`gates/cli.py check` **PASS=13 FAIL=0 SKIP=0**、state-baseline **[GREEN] 74 场景 sha 全同**、ratchet total=1005(基线978) |
| 8 变异独立复做 | 🟢 绿 | `TapUndo=>Stop //MUT` → 全库 --lib **恰 1 红=p2b7_map_raw_input_to_action**（362 过）→ 还原 md5 复原 → p2b7 **4/4 绿**、`git diff` 空 |
| 9 夹带+spec 注记⑥ | 🟢 绿（附挂账⑥） | `--name-status` 恰 6 文件全 M、+273/-48；spec 恰 1 行=注记⑥**如实**（trait 实证 2 方法）；红档 `/tmp/ev_b7_red.txt` 29 错逐码吻合且 mtime 早于 commit；附：注记⑥跳⑤、commit「F2」字面不存在 |

### 挂账（6 条，均零行为）

1. **① 黄-中·面1（下批必办）**：`input_mapper.rs:10-12` 注释与 commit「三动作产出边界事件（shell 三漏斗**实证**）」口径与实测不符——全库 `RawInput::` 生产构造点=**0**，`main.rs:669`（tap_cell 旁路）/`touch.rs:145`（Undo→undo_ui 旁路）两漏斗根本不经 `Action`；map 仅测试面消费（未接线）。**下批随 doc 触碰改口径**（「抽象定义+消费面；shell 接线未入本批」），或批10 收口接线闭合。计划交付（抽象/消费面/夹具/金样零重冻）无缺口，行为零影响。
2. **② 黄-轻·面2**：`input_mapper.rs:20` 注「BootMachine 测试 :362-363=Key」——`controls.rs:359` 实为 **GameScene** `t1_map_idle_buttons`（idle=Key(-6)/Key(-7)）；BootMachine 侧 Key 断言真身=`paint.rs:1116` 等。语义（host 差异随 ControlButton 携带）成立，标签/行锚偏。
3. **③ 黄-轻·面3**：「interrupt_route:**1646**」在 commit message、`scene.rs:1406` 注、`input_mapper.rs:38` 注三处同偏——`:1646` 是 `interrupt_route_for_input` 的 doc 行；`interrupt_route` body=**1631-1643**（结论「不触 tap_pending」实测成立）。
4. **④ 黄-轻·面5**：点名计数「13 项 dialog4」少算 **overlay**（plan:69 dialog 行实为 5 名 ⇒ 全集 14 token；登记已全覆盖，仅计数与 `p2b5` 测试 dialog 断言表=4 名无 overlay——overlay 无测试钉）；t6 夹具真路径=`crates/game-core/src/scene_tests/t2_walkto_route_tests.rs:1740`（`tests/` 目录不存在该文件；行号对）。
5. **⑤ 黄-轻·面6**：批6 黄-轻②另两行未随本批同步——`script_host.rs:19`「menu 4+tap 1=显式 pending 批7/批9 销账」（menu 已批6 清、tap 已批7 清=双 stale）、`:104` PLAN_NAMED_PENDING doc「批7 InputMapper/tap+批9 …menu 登记时销账」（const 已空）。
6. **⑥ 黄-轻·面9**：spec 注记序号 **⑥跳⑤**（系列=①②批0/③批2/④批3/⑥批7，批4/5/6 未触 spec 无⑤）；commit「spec **F2** 注记⑥」中 `F2` 字面在 spec/plan 中 grep=0（注记本体在 spec:113 如实存在）。

---

## 面1 RawInput 三变体 vs 三漏斗边界事件 — 🟡 黄-中（挂账①）

### 实测

```text
$ sed -n '10,22p' crates/game-core/src/subsystem/input_mapper.rs
/// 原始输入事件（批7 终形）：三动作产出边界事件——shell 三漏斗实证
/// （shell/win/src/main.rs:669 地图格/shell/common/src/controls_ui.rs 工具栏命中/shell/common/src/touch.rs:145
/// TapTarget=Undo 区>地图格>图外/scene.rs:488 键盘）。
pub enum RawInput {
    Key(i32),
    TapButton(crate::controls::ControlButton),
    TapUndo,
}
```

- **构造点普查**：`grep -rn "RawInput::" --include="*.rs" crates/ shell/` → **7 命中全在 input_mapper.rs**（map 3 臂 + 测试 4 断言）；排除 input_mapper 后 `grep -rn "RawInput"` → **仅 `subsystem/mod.rs:105/:174`（测试 sig `_sig_input_map`）**；`shell/` 目录 **0 命中**。⇒ **生产构造点=0，map 消费者=仅 p2b7_map+sig_lock（测试面-only，未接线）**。
- **三锚逐点核**：
  - `main.rs:669` = `let _ = sc.tap_cell(x, y);`（`ShellAction::TapCell` 臂）→ 直进 `tap_cell`，**不产 RawInput 也不产 Action**（地图格旁路）。
  - `controls_ui.rs` 的 `Action` 表（:107-109 `Action::Key(-6)/Key(-5)/Undo/Key(-7)`）位于 **`#[cfg(test)] mod tests`（:84 起）**；生产命中路径=`hit_toolbar→ToolbarHit`，android `native.rs:1637` 直接 `sc.perform_action(act)`——不经 RawInput。
  - `touch.rs:188` = `TapTarget::Undo => vec![ShellAction::Undo]` → main 的 `A::Undo` 臂 → `undo_ui::undo_last`（**不调 perform_action**）；`:145` 是 `tap_target` 的 doc 行。**触点 Undo 也是旁路**。
  - `scene.rs:488` = `pub(super) fn set_key_value(&mut self, v: i32) { self.key_value = v; }`——key_value **尾写接口**，非键盘入口（键盘真入口=`main.rs:455 route_key→sc.press_key`，亦不经 RawInput）。
- **键码同币 ✓**：`input_mapper.rs:64` `RawInput::Key(c) => Action::Key(c)`；p2b7 断言 `map(RawInput::Key(5)) == Action::Key(5)`（:154）。
- **Action 生产点全集**（对照「第四入口」问题）：生产代码仅 `controls.rs`——`Controls::key` helper（:77-79）+ `GameScene::controls`（:88）/`BootMachine::controls`（:315）两表字面量；shell 侧 `ShellAction`/`KeyAction` 为独立枚举，不经 `Action`。

### 裁定

- **交付面**：plan 批7 要求「RawInput→Action 抽象（controls+tap_walk 消费面）」——枚举终形、三臂纯 map、controls 消费面（perform_action→apply）、tap_walk 消费面（tap_two_stage 夹具）**均交付**，Key 同币有钉 ⇒ 不判红。
- **证据面**：commit/doc 声称三漏斗「实证产出」三变体，实测 0 构造点且两漏斗旁路——违反 AGENTS §3「结论须证据支撑」口径，且比批5/6 行锚偏差更重（不是行号偏，是**声明的接线不存在**）⇒ **黄-中**，挂账①下批改口径（或批10 接线闭合）。零行为影响（金样 74 绿）。

---

## 面2 map 绑定单源裁定 — 🟢 绿（附挂账②）

### 实测

```text
$ sed -n '61,68p' crates/game-core/src/subsystem/input_mapper.rs
    fn map(&self, ev: RawInput) -> Action {
        match ev {
            RawInput::Key(c) => Action::Key(c),
            RawInput::TapButton(cb) => cb.action,
            RawInput::TapUndo => Action::Undo,
        }
    }
```

- **双轨红判据**：map 臂内**无 `key_value` 读写、无 `controls()` 调用、无表查询**——`InputResolver` 为 ZST（`let mut r = InputResolver;`），struct 无字段，**结构性不可能自设绑定**。⇒ 单源裁定成立。
- **绑定构造侧（唯一产地）实文**：
  - `controls.rs:77-79` `fn key(function, code) -> ControlButton { … action: Action::Key(code) }`（单 helper）；
  - `controls.rs:203/205`（GameScene `dialog_controls` 寻路中臂）`ControlButton { function: Bag, action: Action::StopThen(-6) }` / `{ Menu, action: Action::StopThen(-7) }`；
  - `controls.rs:362-363`（`t1_map_idle_buttons`）`btn(Bag, Action::Key(-6))` / `btn(Menu, Action::Key(-7))`——idle 态=Key（同钮不同态不同绑定，**随 ControlButton 携带** ✓）；
  - BootMachine 侧 Key 断言真身=`paint.rs:1116` `Some(Action::Key(-7))`（挂账②：input_mapper.rs:20 注把它标成「:362-363」）。
- `map` 的 TapButton 臂=`cb.action` 直通（:66），p2b7 断言 `TapButton(cb)==Fight`、`TapButton(cb2)==StopThen(-6)`（host 差异携带）✓。

### 裁定

绑定只在 ControlButton 构造侧一处（controls 两表 + 单 helper），map 纯转发无第二轨 ⇒ **绿**；注释锚标签偏=挂账②（零行为）。

---

## 面3 apply 逐臂 vs perform_action 原文等价 + 序等价独立验证 — 🟢 绿（附挂账③）

### 逐臂对拍（`scene.rs:1384` 起 diff 现场；apply 实文 `input_mapper.rs:69-105`）

| Action 臂 | 旧 perform_action | 新（resolver 域写 + InputStep 桥接） | 等价 |
|---|---|---|---|
| Key(c) | `key_value=c; true` | `*ctx.key_value=c; Accepted→true` | ✓ 字面同 |
| Stop 寻路中 | 清 bump/selected/pending/target → `interrupt_route()` → true | 同序四清（:74-77）→ `Interrupt` → 桥接 `interrupt_route()` → true | ✓ 同序同集 |
| Cancel | `selected=None; true` | `tap_selected=None; Accepted→true` | ✓ |
| Fight | `engage_selected(); true` | `Engage` → 桥接 `engage_selected()` → true | ✓ |
| StopThen 寻路中 | 清 bump/selected/target → `interrupt_route()` → **后置** `pending=Some(StopThen(c))` → true | 清同3（:97-99 保文「【复核 T2-R5】打断 ⇒ 目标标记消失」）→ **先置** `pending=Some(StopThen(c))`（:100）→ `InterruptThen(c)` → 桥接 `interrupt_route()` → true | ✓ **序差=装配时机前移；由序等价独立验证闭合（下）** |
| StopThen 非寻路 | `key_value=c; true` | `*ctx.key_value=c; Accepted→true` | ✓ |
| Undo | `false` | `Ignored→false`（桥接注「Undo：core 无撤销入口」） | ✓ |

- **walk 先算后借**：`scene.rs:1388` `let walk = self.tap_walk_in_progress();`（纯读，无副作用）→ 其后才构造 `InputCtx { walk_in_progress: walk, key_value: &mut self.key_value, tap: &mut self.tap }` ⇒ 无 E0502 借用冲突（批2 教训落实）。

### 序等价独立验证（读 interrupt_route 实现——任务要求的独立核）

```text
$ sed -n '1631,1643p' crates/game-core/src/scene.rs
    pub fn interrupt_route(&mut self) {
        if !self.route.is_open() && !self.route.is_suspended() { return; }
        let at = self.route_landing_cell();
        self.route.interrupt(at, &mut self.undo);
        self.walk_path_buffer.clear();
        self.walk_step_count = 0;
        if self.game_mode == GameMode::Dialog && self.dialog.dialog_phase == 3 {
            self.walkto_draining = true;
        }
    }
$ sed -n '1631,1643p' … | grep -c tap
0
```

- body 只碰 `route/undo/walk_path_buffer/walk_step_count/game_mode/dialog.dialog_phase/walkto_draining`，**零 tap 字段访问（grep=0）** ⇒ `interrupt_route` 不读不写 `tap_pending` ⇒ StopThen 臂 pending 装配前移**不可观察** ⇒ 序等价 **A 级** ✓。
- 行锚：commit/两处注释写 `interrupt_route:1646`——`:1645-1646` 实为 `interrupt_route_for_input` 的 doc 行，body=1631-1643 ⇒ 挂账③（结论对、锚偏）。

### 裁定

逐臂语义全等 + 序等价 A 级独立证实 ⇒ **绿**（附挂账③）。

---

## 面4 InputStep 5 臂桥接完备性 — 🟢 绿

### 实测

- `input_mapper.rs:27-39`：`pub enum InputStep { Accepted, Ignored, Interrupt, Engage, InterruptThen(i32) }`（5 变体，doc 注逐臂语义）。
- `Action` 枚举恰 **6** 变体（`controls.rs`：Key/Stop/Cancel/Fight/StopThen/Undo）→ `apply` match **无通配**覆盖 6 臂（编译器强制穷尽）。
- 桥接现场 `scene.rs:1393-1411`：

```text
        match step {
            InputStep::Ignored => false, // Undo：core 无撤销入口（撤销执行在壳 undo_ui）
            InputStep::Accepted => true,
            InputStep::Interrupt => { self.interrupt_route(); true }
            InputStep::Engage => { let _ = self.engage_selected(); true }
            InputStep::InterruptThen(_) => {
                // pending=StopThen(c) 装配已在 resolver（interrupt_route:1646 不触 tap_pending，序等价 A 级）
                self.interrupt_route(); true
            }
        }
```

- **逐臂消费语义**：Ignored→false / Accepted→true / Interrupt→`interrupt_route`+true / Engage→`engage_selected`+true / `InterruptThen(_)`→pending 已在 resolver 装配、Coordinator 只执行 `interrupt_route`+true。**无通配=漏臂即编译红**，5 臂全消费 ✓。
- **Undo=Ignored→false 语义对否**：≡ 旧 `Action::Undo => false`（core 无撤销入口，撤销执行在壳 `undo_ui`）✓；p2b7_apply 断言 `apply(Undo)==Ignored`（:178）。
- `InterruptThen(i32)` 的 i32 在桥接臂显式丢弃（`InterruptThen(_)`）——键码用途已在 resolver 装入 `tap_pending`，无泄漏 ✓。

### 裁定

**绿**（5/5 臂、穷尽由编译器钉、Undo 语义与旧行为逐字等价）。

---

## 面5 盲区 13 项对账 — 🟢 绿（附挂账④）

### 实测

```text
$ sed -n '104,107p' crates/game-core/src/subsystem/script_host.rs
/// plan:69 点名项显式 pending（批7 InputMapper/tap+批9 ShopGoods/menu 登记时销账——批3 红1 教训横切）。
pub const PLAN_NAMED_PENDING: &[&str] = &[];
// plan:69-71 点名 13 项已全部登记销账：dialog4 批3（DIALOG_BLIND_ZONES）+persist4 批4（PERSIST_BLIND_ZONES）
// +menu4 批6（MENU_BLIND_ZONES）+tap1 批7（INPUT_BLIND_ZONES tap.auto_battle）。本表清空，挂账机制保留。
```

- **真身清空 ✓**：`PLAN_NAMED_PENDING: &[&str] = &[]`（:105）；p2b7 断言 `!all.contains("autoBattle")`（:244，all=INPUT+PLAN 拼接）+ p2b5 断言 `PLAN_NAMED_PENDING.is_empty()`（script_host.rs:195）双钉。
- **plan 点名全集实文**（`docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md:68-72`）：

```text
  dialog（typewriterPos/dialogSpeaker/dialogAnchor/dialogTarget/overlay 布局面）、
  menu（menuCursor/itemCursor/recordPage/difficultyIndex）、
  persist（min/maxFloor/pendingFloor/wentDown）、tap（autoBattle）
```

  ⇒ **dialog=5、menu=4、persist=4、tap=1 ⇒ 共 14 token**；commit/注释「13 项 dialog4」**少算 overlay** ⇒ 挂账④（仅计数口径）。
- **覆盖核（机械提取四表）**：
  - `DIALOG_BLIND_ZONES`（dialog.rs:71）**10 条**，5 点名全含：`typewriterPos=dialog_char` 条 ✓、`dialogTarget 直写（script_ops.rs:136/164…）` 条 ✓、`open_dialog_page…内写 dialogSpeaker :1050/dialogAnchor :1055` 条（一钉二）✓、**`overlay 布局面（…plan:69 明文项）` 条 ✓**；
  - `PERSIST_BLIND_ZONES`（persist_codec.rs:103）**5 条=4 点名+1 评估行**（minFloor/maxFloor/pendingFloor/wentDown 逐条 ✓）；
  - `MENU_BLIND_ZONES`（item_service.rs:41）**恰 4** ✓；
  - `INPUT_BLIND_ZONES`（input_mapper.rs:110）**2 条=1 点名（tap.auto_battle）+1 状态族评估行** ✓。
- **锚实文（写点/读门）**：
  - `scene_persist.rs:57` = `auto_battle: false,`（`load_floor` 内 TapState init 字面 :52-63）✓；
  - `scene.rs:1279` = `if !self.tap.auto_battle && !again {`（首点选中读门）✓；
  - 夹具钉：`crates/game-core/src/scene_tests/t2_walkto_route_tests.rs:1740` = `fn t6_auto_battle_fights_on_first_tap()` ✓（**行号对**；commit 注只写文件名、brief 写 `tests/` 前缀——真身在 `src/scene_tests/`，`crates/game-core/tests/` 无此文件）。
- **测试钉缺口（挂账④内）**：`p2b5_blind_zones_cover_plan_named`（script_host.rs:186-189）dialog 断言表=4 名（typewriterPos/dialogSpeaker/dialogAnchor/dialogTarget），**overlay 不在断言表**（登记在册但删掉不红）。

### 裁定

清空=点名项**全部在册**（14/14 覆盖，非 commit 所称 13/13——计数少 1 但无漏登记）、写点/读门锚全对、夹具钉存在 ⇒ **绿**；计数口径+overlay 无测试钉+t6 路径前缀=挂账④。

---

## 面6 处置账表同步充分性 — 🟢 绿（附挂账⑤）

### 实测

```text
$ sed -n '6,20p' crates/game-core/src/subsystem/script_host.rs（关键行）
//! 【HostCtx 余方法处置账】（plan 批5 明文交付；**批6 摘 2 后 HostCtx 现 8 方法**[give_gold/pickup_item 已迁批6 ItemService]；
//! 批7 InputMapper 无 HostCtx 面=N/A（输入面三漏斗不涉 host 方法）——逐条处置归宿如下）：
//! | give_gold / pickup_item | GIN op 面 | ~~批6 ItemService~~ **已迁（批6 落定）** |
…
//! （dialog/persist 域已登记=p2b5 跨域断言；menu 4+tap 1=显式 pending 批7/批9 销账）。     ← :19 stale
```

- **批6 黄-轻②三要素全齐**：①「HostCtx 现 **8**」（:7）✓ ② give_gold/pickup_item **已迁（批6 落定）**（:12）✓ ③ 批7 InputMapper **N/A 行**（:8）✓。
- **HostCtx 实测恰 8 方法**：`script.rs:135-156` `find_path / snap_camera_to_player / locate_speaker_entity / ces_camera / set_camera_anchor_target / exit_application / mvs_exchange / snap_camera_after_ros1`（doc 注「批6 摘 2…HostCtx 余 8」）⇒ 对账链 10→8 成立 ✓。
- **附挂账⑤**：批6 黄-轻②点名的另两行本批未同步——`:19`「menu 4+tap 1=显式 pending 批7/批9 销账」（menu 已批6 清、tap 已批7 清=文已双 stale）、`:104` PLAN doc「批7 InputMapper/tap+批9 ShopGoods/menu 登记时销账」（const 已空）。零行为、纯 doc 陈旧（与批6 同类，下批顺手同步）。

### 裁定

任务要求的三要素（余 8/已迁标注/N/A 行）**全部交付** ⇒ **绿**；残余两行=挂账⑤。

---

## 面7 判据独立重跑 — 🟢 绿

| 判据 | 独立重跑结果 | 命令 |
|---|---|---|
| p2b7 四测 | **4 passed / 0 failed / 361 filtered** | `cargo test -p game-core --lib p2b7` |
| game-core --lib | **363 passed / 0 failed / 2 ignored**（11.80s） | `cargo test -p game-core --lib` |
| fixture_frames 夹具面 | **45 passed / 0 failed**（95.35s） | `cargo test -p game-oracle --test fixture_frames` |
| 金样 | **[GREEN] 金样逐字节等价：74 场景 sha 全同** | `python3 gates/cli.py check` → state-baseline |
| 总闸 | **PASS=13 FAIL=0 SKIP=0**（original/assets/reference-seed/knowledge/docs-links/naming/rust-tests/test-ratchet=total 1005≥978/table-content-hash/table-evidence/state-machine-anchors/asset-truth/state-baseline 全绿，exit=0） | 同上 |

- 口径注：filter `fixture_frames` 只中 3 测（错口径）；正确=**test target** `--test fixture_frames`（`crates/game-oracle/tests/fixture_frames.rs`）45/45 ✓。
- **未独立复跑项（如实披露）**：commit ⑦ Java 三方 47 场景（`A==B==C: PASS sha=3ad6ef6a…`、`47/47 场景 PASS`）不在任务判据清单内、本次未复跑（不在 13 闸内）；金样 74 已独立复跑覆盖保真主裁。

### 裁定

**绿**（5/5 判据全独立复现，数字与 commit 逐项吻合）。

---

## 面8 变异独立复做 — 🟢 绿

### 协议与实测

```text
$ cp crates/game-core/src/subsystem/input_mapper.rs /tmp/im_bak_6ce6f85.rs
$ md5sum … → 83dd71159add1c202881191ce394fb49（备份=原盘）
$ edit: RawInput::TapUndo => Action::Undo,  →  RawInput::TapUndo => Action::Stop, //MUT   （:66，git diff 恰 1 行）
$ cargo test -p game-core --lib
test subsystem::input_mapper::tests::p2b7_map_raw_input_to_action ... FAILED
test result: FAILED. 362 passed; 1 failed; 2 ignored …                        ← 恰 1 红
$ cp /tmp/im_bak_6ce6f85.rs … && md5sum → 83dd71159add1c202881191ce394fb49（复原一致）
$ git status --porcelain -- crates/ shell/ reference/ gates/ data/ original/ → 空（树净）
$ cargo test -p game-core --lib p2b7
test result: ok. 4 passed; 0 failed …                                                  ← 还原绿
```

- **判别力**：全 --lib 下扰动只红 `p2b7_map_raw_input_to_action`（其断言 `TapUndo→Undo` 正是扰动面），其余 362 全绿 ⇒ 测试**恰覆盖该语义、无误伤无漏判**；与 commit「变异自证：仅 p2b7_map 红→回原 4/4 绿（MUT 计数 0）」独立吻合。
- 还原后 `grep -c MUT crates/game-core/src/subsystem/input_mapper.rs` = 0（md5 与原盘一致）。

### 裁定

**绿**（扰动→恰 1 红→还原→复绿，全链独立复做，工作树净）。

---

## 面9 夹带 + spec 注记⑥ — 🟢 绿（附挂账⑥）

### 实测

```text
$ git show 6ce6f85 --name-status
M crates/game-core/src/scene.rs
M crates/game-core/src/subsystem/input_mapper.rs
M crates/game-core/src/subsystem/item_service.rs
M crates/game-core/src/subsystem/mod.rs
M crates/game-core/src/subsystem/script_host.rs
M docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md
$ git show 6ce6f85 --stat | tail -1
 6 files changed, 273 insertions(+), 48 deletions(-)
```

- **恰 6 文件**，全 M；**基线/reference/gates/original 零触碰** ✓（金样零重冻由面7 闸证）。
- **spec 恰 1 行改动=注记⑥，如实**：`spec:113` 新增「【2026-11-07 更正·批7】⑥ InputMapper 实现为 2 方法（map+apply——`apply(&mut self, ctx:&mut InputCtx, a:Action)->InputStep` …原文单 map 形不删备查）」；`spec:103` 原文 `trait InputMapper { fn map(&self, ev: RawInput) -> Action; }` **未删** ✓；**内容与实现对拍属实**——trait 现恰 2 方法（map :53 + apply :55，impl :61/:69），签名逐字一致（`&mut InputCtx`/`InputStep`/`Action`）✓。批2/批3 注记同款先例格式。
- **附挂账⑥**：注记序号 ⑥ 跳 ⑤（系列：①②批0→③批2(spec:111)→④批3(:112)→⑥批7(:113)；批4/5/6 未触 spec）；commit「spec F2 注记⑥」中 `F2` 字面 spec/plan grep=0（注记本体在 :113 存在，编号自洽于正文「⑥」）。
- **修复实录三则核**：
  - ① `fresh_tap()`：`input_mapper.rs:130-143` **11 字段**字面 helper，与 `scene_persist.rs:52-63`（load_floor 内 TapState 字面）**逐字段同序同值** ✓（`TapState` 11 pub 字段+编译穷尽=字段全）；commit「scene_persist.rs:52 照录」属实。
  - ② sig_lock：`mod.rs` use 行=`use super::input_mapper::{InputCtx, InputMapper, InputStep};` + `_sig_input_apply` ✓。错误码：commit 写 **E0425**（use 行缺型→cannot find type，与「补 use」修法自洽）；**任务 brief 写 E0509**——两口径不一，红档实文见下（红档为 E0425 家族，brief 口径未获证据支持）。
  - ③ `item_service.rs:119` = `assert!(!pend.contains("autoBattle"), "tap:autoBattle 已批7 销账…")`（contains→!contains 翻向）✓。
- **TDD 红档独立核**（artifact，非重跑）：`/tmp/ev_b7_red.txt`——`stat` mtime=**20:48:18** < commit 时间 **21:24:58** ✓；内容 `error: could not compile game-core (lib test) due to 29 previous errors; 12 warnings`，逐码统计 **29 个 error[E…]=E0282×1/E0422×7/E0425×10/E0433×7/E0599×4**，与 commit Evidence① 所列码集（E0282/E0422/E0425/E0433/E0599）**逐字吻合**，warning 汇总行=`generated 12 warnings` 同 ✓。首行现场=`input_mapper.rs:47 cannot find value InputResolver`（缺位面纯净）。

### 裁定

无夹带、spec 改动=如实注记、修复实录可验项全对、红档实证吻合 ⇒ **绿**（附挂账⑥编号/字面小疵）。

---

## 结论

- **9 面 0 红**：判据 5/5 独立复现（含金样 74 逐字节、13 闸全绿）、变异扰动→恰 1 红→还原复绿、逐臂/序等价 A 级独立证实、盲区点名 14/14 全登记、夹带恰 6 文件。
- **1 黄-中（挂账①）**：RawInput「三漏斗实证」口径与实测不符（生产构造点=0、map 测试面-only、两漏斗旁路）——**下批必办**（doc 口径更正或批10 接线），不阻断本批（计划交付与行为面零缺口）。
- **5 黄-轻**：行锚/标签/计数/编号类（挂账②-⑥），全部零行为，随下批 doc 触碰顺手同步。
- **总判：可推送**（附挂账 6 条；①为下批必办）。

Confidence: 93/100
