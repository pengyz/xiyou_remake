# P2 批8 对抗复核报告 — commit 7f9e7ff

> 被审件：`7f9e7ff` feat(core)「P2 批8 UIRender trait 化（UiFrame 纯读快照+E3 六字段值拷贝）+批7 挂账①-⑥清」（5 文件 +108/-16，`git show --numstat` 实测：scene.rs 2/1、input_mapper.rs 7/5、script_host.rs 5/5、ui_render.rs 93/4、spec 1/1），modern 分支
> 复核方式：对抗复核（只读；唯一写=本报告 + 任务授权的变异与还原）；证据带实测输出；纪律=AGENTS.md（保真=金样裁判）
> 日期：2026-11-07 · 被审 HEAD=`7f9e7ffd4fcf355d57db2f270e70f376f570f23d`（`git status` 实测，tracked 干净）· 复核执行=独立重跑（判据全复跑、变异独立复做）

## 总判：可推送（9 面 **0 红**；4 条黄-轻挂账，均零行为、不阻断本批）

| 面 | 裁定 | 一句话 |
|---|---|---|
| 1 UiFrame 六字段 vs RenderState E3 | 🟢 绿 | 恰 6 字段名称/类型/语义逐字段同，snapshot=结构体字面量逐字段值拷贝（derive Copy，无 &T 泄漏）；附黄-轻①`p2b8_ui_frame_field_set_matches_render_state` 恒真=判别力缺口 |
| 2 纯读裁定 | 🟢 绿 | sig_lock `_sig_ui_snapshot`（mod.rs:180-182）+ `via_ref<T:&T>`（ui_render.rs:81-89）双钉在位；grep `mut`/`&mut` ui_render.rs 仅命中注释行，impl 体零写 |
| 3 facade 复出口 | 🟢 绿 | scene.rs:62 `pub use scene_paint::RenderState;` 与 :61 批3 `pub use scene_dialog::DialogState;` 同形先例 |
| 4 ease 簇处置登记闭环 | 🟢 绿 | ui_render.rs 头注↔dialog.rs:87 注记双向在位；4 setter 真身 scene_paint.rs:27-30（26-33 覆盖 impl 块），scene_paint/dialog 均不在 diff=仅登记不迁 |
| 5 批7 挂账①-⑥销账核 | 🟢 绿 | 6 条逐条实改在位（①必办=头注新文+更正注在位）；附黄-轻②③④（③勘误注 body 跨度 off-by-one、commit scene.rs:1406→实 1407、doc 续行顶格） |
| 6 消费面披露=0 | 🟢 绿 | `grep -rn "snapshot("` 生产调用点=0（仅 ui_render 自身定义+测试），commit 口径与实测一致 |
| 7 判据独立重跑 | 🟢 绿 | p2b8 **3/3**、game-core --lib **366/0/2i**、全量 game-core **22 桶全绿**、fixture_frames **45/45**、金样 **[GREEN] 74**、闸 **PASS=13 FAIL=0 SKIP=0** |
| 8 变异独立复做 | 🟢 绿 | backdrop_scroll←open_anim_tick 同型互换 → 恰 1 红（value_copy，(3,11,5)≠(3,-7,5)，另 2 绿）→ 还原后 3/3 绿、`git diff` 空、MUT=0 |
| 9 夹带+spec⑥+零行为 | 🟢 绿 | `--name-status` 恰 5 文件全 M（基线/reference/gates/original 零触碰）；spec diff=恰 1 行=⑥编号空缺说明（历史主张未删）；ui_render 全增行无副作用、HEAD 锚逐一对上 |

### 挂账（不阻断本批，须入档）

1. **黄-轻①·面1 判别力缺口**：`p2b8_ui_frame_field_set_matches_render_state`（ui_render.rs:93-99）恒真——`names_f`/`names_r` 两个相同字面量数组自比（:95/:96 逐字同），`let _ = f;` 不消费快照，与真实结构零耦合。**RenderState 单侧加第 7 字段不红**（UiFrame 侧改/增字段有 impl 结构体字面量编译耦合兜底，反方向无钉）。commit 称「六字段同集防单侧加字段」判别力名不副实。零行为（纯测试）；建议批9 用 `std::mem::discriminant`/字段数断言或对 RenderState 反射比对收口。
2. **黄-轻②·面5 挂账③勘误注自身行锚 off-by-one**：input_mapper.rs:38-39 与 scene.rs:1407 新注写「interrupt_route body=**1631-1643**」——该数沿用批7 复核原文，但本批在 scene.rs:62 插入 1 行（+1 平移）后 HEAD 实测：`pub fn interrupt_route` 在 **:1632**、语句体 **1633-1643**、闭 `}` **1644**（:1631 为 doc 行）。语义结论「不触 tap_pending」实测成立（`sed -n 1631,1643p | grep -c tap_pending`=0）。零行为。
3. **黄-轻③·面5 commit message 行锚**：commit 称③修复点「scene.rs:**1406**」，HEAD 实测该注在 **scene.rs:1407**（同因 +1 平移；input_mapper.rs:38 实对）。零行为。
4. **黄-轻④·面5 doc 续行顶格**：input_mapper.rs 新增勘误注的续行 `/// :1646 为 interrupt_route_for_input doc 行`（:39）与挂账②续行（:20）均从列 0 起（前一行 4 空格缩进），缩进断裂——合法编译、rustfmt 违形。零行为。

（登记类观察，不计挂账：面4 ease 登记锚「scene_paint.rs:26-33」含 4 setter（:27-30）之外还覆盖 3 个 add_*（:31-33），略松不漏；dialog 侧按名「批8 UIRender 面」、ui_render 侧按文件「dialog.rs」互指，语义闭环但字面非对称 file:line。）

---

## 面1 UiFrame 六字段 vs RenderState E3 面逐字段 — 🟢 绿（附黄-轻①）

**定义对照**（`sed -n` 实文）：

- `crates/game-core/src/scene_paint.rs:6-18`：
  ```
  pub struct RenderState {
      pub strip_frame: usize,     // :8
      pub backdrop_scroll: i32,   // :10
      pub bob_offset: i32,        // :12
      pub bob_rising: bool,       // :13
      pub open_anim_tick: i32,    // :15
      pub tile_bob: i32,          // :17
  }
  ```
- `crates/game-core/src/subsystem/ui_render.rs:19-31`：`pub struct UiFrame` 恰同 6 字段、同名、同型（usize/i32/i32/bool/i32/i32），derive `Debug, Clone, Copy, PartialEq, Eq`。**多/缺字段=无**（两侧 grep 字段数均=6，字段名集合逐字同）。

**值拷贝 vs 引用**（ui_render.rs:41-50 impl 实文）：

```rust
impl UIRender for UIRenderResolver<'_> {
    fn snapshot(&self) -> UiFrame {
        UiFrame {
            strip_frame: self.0.strip_frame,
            backdrop_scroll: self.0.backdrop_scroll,
            bob_offset: self.0.bob_offset,
            bob_rising: self.0.bob_rising,
            open_anim_tick: self.0.open_anim_tick,
            tile_bob: self.0.tile_bob,
        }
    }
}
```

六字段逐字面量拷贝、结构体按值返回（derive Copy），**无 `&T`/`&self.0.x` 泄漏**。doc 逐字同源（a.java:464/PARALLAX_WRAP/浮沉/开门 18·24/遮幅 ±2 锚）。

**附黄-轻①**：`p2b8_ui_frame_field_set_matches_render_state` 恒真（见挂账 1）。

## 面2 纯读裁定 — 🟢 绿（双钉在位+实现体零写）

**钉1 sig_lock**（`crates/game-core/src/subsystem/mod.rs:180-182`，在 `#[cfg(test)] mod tests`(:98)/`#[allow(dead_code)] mod sig_lock`(:149) 内）：

```rust
pub fn _sig_ui_snapshot<T: UIRender>(t: &T) -> UiFrame {
    t.snapshot()
}
```

签名收 `&T`——trait 若改 `&mut self` 此处**不可编译**（编译期钉）。

**钉2 via_ref**（ui_render.rs:81-89）：

```rust
fn via_ref<T: UIRender>(t: &T) -> UiFrame { t.snapshot() }
```

测试 `p2b8_ui_render_pure_read` 经 `&T` 调 snapshot 并断言 strip_frame=3/bob_rising=true。

**实现体写扫描**：`grep -n "&mut\|mut " crates/game-core/src/subsystem/ui_render.rs` 仅命中 :4/:78/:79 三处**注释**行（判据说明文字），代码零 `mut`；impl 体仅结构体字面量读（见面1）。

## 面3 facade 复出口 vs 批3 DialogState 先例 — 🟢 绿

- `crates/game-core/src/scene.rs:62`：`pub use scene_paint::RenderState; // 【P2 批8】UIRender 消费面（E3 六字段切片）跨模块可达（facade 复出口先例）`
- `crates/game-core/src/scene.rs:61`（批3 先例）：`pub use scene_dialog::DialogState;`

两行同形（均 `pub use <私有 mod>::<Type>;`，mod 经 `#[path]` 私有挂载、由 pub use 复出），**形同**。

## 面4 ease 簇处置登记闭环 — 🟢 绿

**互指实文**：

- `ui_render.rs:6-8` 头注：登记 ease 簇（`set_ease_cam_x/y`+`set_ease_target_x/y`）= render 域直写接口家，「dialog.rs 盲区注记指回本行」。
- `dialog.rs:87`（批3 既有，不在 diff）：「ease_cam_x/y/ease_target_x/y），非 dialog 域；**归批8 UIRender 面处置**，不入本清单。」

两侧语义闭环：dialog 认领归批8、批8 头注登记归 render 域。

**4 setter 真身**（`scene_paint.rs:26-33`）：:26 `impl GameScene {`、:27-30 = `set_ease_cam_x/set_ease_cam_y/set_ease_target_x/set_ease_target_y` 4 个 setter、:31-33 = add_ease_cam_x/cam_y/target_x（登记锚 26-33 覆盖 4 setter 且略含 add_*，略松不漏）。

**归属声明 vs diff 一致**：`git show 7f9e7ff --numstat` 不含 `scene_paint.rs`、不含 `dialog.rs` ⇒ **仅登记不迁，声明与 diff 零动一致**（commit「scene_paint.rs 925 行 paint 编排面不动」成立）。

## 面5 批7 挂账①-⑥逐条销账核 — 🟢 绿（附黄-轻黄-轻②③④）

挂账原文=`docs/findings/reviews/2026-11-07-p2-batch7-review.md:21-28`。逐条实测：

| 挂账 | 要求 | 实测（HEAD 实文） | 判 |
|---|---|---|---|
| ① 必办 | input_mapper 头注改口径+更正注 | `input_mapper.rs:10-11`「原始输入事件（批7 终形）：抽象定义+消费面，shell 接线未入本批（生产构造点=0，map 现仅测试消费）——三漏斗边界事件口径…」+ `:12`「【2026-11-07 更正·挂账①】原注「三漏斗实证」口径不实（列了 shell 行锚但无生产构造点）。」✔ 全库 grep「三漏斗实证」仅命中此更正注=旧口径零残留 | 🟢 |
| ② | BootMachine 标签/行锚更正 | `input_mapper.rs:18-20`「controls.rs:362-363 t1_map_idle_buttons=Key(-6/-7)（GameScene 侧）；BootMachine 侧 Key 断言真身=paint.rs:1116——挂账②行锚勘误」；实测 controls.rs:359 fn t1_map_idle_buttons/:362=Key(-6)/:363=Key(-7)、paint.rs:1116=`assert_eq!(…Some(Action::Key(-7)), "设置返回 −7")` ✔ | 🟢（附黄-轻④缩进） |
| ③ | interrupt_route:1646→body 1631-1643 ×2 注 | `input_mapper.rs:38-39`+`scene.rs:1407` 两处「interrupt_route body=1631-1643 不触 tap_pending（挂账③行锚勘误：:1646 为 interrupt_route_for_input doc 行）」；实测 :1646=`/// 【STAGE-B-008…`（interrupt_route_for_input 首 doc 行，fn :1650）✔「:1646 doc 行」精确；body 跨度见黄-轻② | 🟢（附黄-轻②③） |
| ④ | 计数 13→14+p2b5 加 overlay | `script_host.rs:106`「14 token（挂账④计数勘误 13→14）：dialog5 批3（DIALOG_BLIND_ZONES 含 overlay）+persist4 批4+menu4 批6+tap1 批7」；`script_host.rs:187` p2b5 断言表=`["typewriterPos","dialogSpeaker","dialogAnchor","dialogTarget","overlay"]`（5 名含 overlay）；dialog.rs:71 `DIALOG_BLIND_ZONES` 含 overlay ✔ | 🟢 |
| ⑤ | script_host.rs :19/:104 双 stale 改已销账 | `:19`「（dialog/persist 域已登记=p2b5 跨域断言；menu4 批6/tap1 批7 已销账——pending 全清）」；`:104`「plan:69-71 点名项显式 pending（已全清——批3 红1 教训横切机制保留）」（const 已空 `&[]`）✔ | 🟢 |
| ⑥ | spec 注记补编号空缺说明 | spec:110 行尾增「（序号⑤空缺：批4/5/6 未触本节，编号如实不补号）」，1 行改、历史主张未删 ✔ | 🟢 |

注：t6 夹具 `tests/` 前缀残留于批7 commit message（不可改），本批代码注 input_mapper.rs:115 仅写文件名「（t2_walkto_route_tests.rs:1740）」无 `tests/` 前缀=无需改。

## 面6 消费面披露=0 充分性 — 🟢 绿

- `grep -rn "snapshot("`（crates/，排除定义自身与测试）：生产调用点 **恰 0**——`UiFrame`/`UIRenderResolver` 全库仅 ui_render.rs 自身 + mod.rs:108 sig_lock（测试）。
- ui_render.rs:11 头注「【消费面披露】snapshot 生产消费点=0（shell 接线未入本批——批7 挂账①同口径）：本批只立面+终形。」commit 第 7 条同口径 ⇒ **注释/commit/实测三者一致**，批7 挂账①「口径病」未复发。

## 面7 判据独立重跑 — 🟢 绿

| 判据 | 实测输出 |
|---|---|
| p2b8 3/3 | `cargo test -p game-core --lib p2b8` → `ok. 3 passed; 0 failed; 365 filtered out` |
| game-core --lib | `366 passed; 0 failed; 2 ignored; 0 measured`（15.18s） |
| 22 桶 | 全量 `cargo test -p game-core` → **22 个 test result 套件全 ok**（366,6,1,9,16,12,6,6,5,6,4,16,14,6,5,2,1,10,11,4,12,0；0 failed） |
| fixture_frames 45/45 | `cargo test -p game-oracle --test fixture_frames` → `ok. 45 passed; 0 failed`（171.67s） |
| 金样 [GREEN] 74 | `python3 gates/cli.py check` → `[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同` |
| 闸 13 | 同跑 → `exit=0`、`== 汇总 == PASS=13 FAIL=0 SKIP=0`（含 rust-tests release 全绿、test-ratchet total=1008≥978、reference-regression api_surface refs=707 shim=69 missing=0、diff-trace ok） |
| Java 47/47 | `python3 gates/cli.py scenarios` → 见「面7 补记」 |
| 帧夹具判据锚 | `fixture_frames.rs:310-313`（:313 `if let Ok(d) = std::env::var("FRAME_OUT")`）像素取证缝在位 |
| 红档（TDD 先红） | `/tmp/ev_b8_red.txt` mtime **22:24:23** 早于 commit **22:59:00**，内容=7 错（E0425×4/E0609×2 + could not compile…7 previous errors） |

**面7 补记（Java 三方 47 场景）**：`python3 gates/cli.py scenarios` → `[✓ PASS] scenario-suite — [oracle] == --scenarios 结果：PASS（47/47 场景）==`（与 commit ⑦ 贴件一致）。附观察：该子命令 shell exit=1——`COMMANDS["scenarios"]=gate_scenarios` 返回 report() 的状态字符串，`sys.exit("PASS")` 在 Python 语义下=打印 PASS 并以码 1 退出（cli.py:48-53/:636）；**既有 gates CLI 行为，本 commit 未触 gates/，非本批引入**，PASS/FAIL 以状态行判读。

## 面8 变异独立复做 — 🟢 绿

1. `cp ui_render.rs /tmp/ur_bak.rs` 备份 → python 将 impl 内 `backdrop_scroll: self.0.backdrop_scroll,` 改 `backdrop_scroll: self.0.open_anim_tick, //MUT`、`open_anim_tick: self.0.open_anim_tick,` 改 `open_anim_tick: self.0.backdrop_scroll, //MUT`（i32 同型互换）。
2. `cargo test -p game-core --lib p2b8` → **恰 1 红**：
   ```
   assertion `left == right` failed: 值拷贝前三字段
     left: (3, 11, 5)
    right: (3, -7, 5)
   failures: subsystem::ui_render::tests::p2b8_ui_frame_snapshot_value_copy
   test result: FAILED. 2 passed; 1 failed; 365 filtered out
   ```
   （value_copy 红；field_set 恒真仍绿、pure_read 只钉 strip_frame/bob_rising 仍绿——与面1 黄-轻①预测一致。）
3. `cp /tmp/ur_bak.rs` 回原 → `git diff --stat`=空（0 行）、`grep -c MUT ui_render.rs`=0、p2b8 → `ok. 3 passed; 0 failed` **还原绿**。

## 面9 夹带+spec⑥+零行为核 — 🟢 绿

- **恰 5 文件**：`git show 7f9e7ff --name-status` = M scene.rs / M subsystem/input_mapper.rs / M subsystem/script_host.rs / M subsystem/ui_render.rs / M docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md。`data/status/baselines`、`reference/`、`gates/`、`original/` **零触碰**。
- **spec 改=⑥**：spec diff 恰 1 行（1+/1-），唯一改动=行尾追加「（序号⑤空缺：批4/5/6 未触本节，编号如实不补号）」，与⑥要求如实、历史主张未删 ✔。
- **零行为核**：ui_render.rs 93 增行=模块头注+`use`+`UiFrame`/`UIRender`/`UIRenderResolver` 定义+impl+3 测试（grep 扫描：无赋值副作用、无 `push/insert/mut`、无 control-flow）；scene.rs 仅 `pub use`+1 注释；input_mapper/script_host 仅注释与测试断言表；spec 仅注释 ⇒ 金样零重冻（gates check `[GREEN] 74` 佐证）。
- **HEAD 锚 sed 实测**：ui_render.rs:19 `pub struct UiFrame` /:34 `pub trait UIRender` /:35 `fn snapshot(&self) -> UiFrame` /:39 `pub struct UIRenderResolver<'a>(pub &'a RenderState)` /:41 `impl` /:42 `fn snapshot`；scene.rs:62 facade ——与 commit「HEAD 锚」段逐一对上 ✔。

---

Confidence: 90/100
