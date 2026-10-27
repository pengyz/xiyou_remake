# 游戏内右滑退出确认框 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Android 竖屏游戏内、地图行走态下系统返回（右滑/BACK）弹出原版 kind 4 样式的退出确认框，确定即退出进程。

**Architecture:** 游戏核心 `GameScene` 新增独立模态状态（STAGE-B-005，不碰原版 `overlay_*`）：壳调用 `open_exit_confirm()` 打开，tick 头拦截框内键并冻结逻辑，Map 臂末绘框，确认发 `take_exit_confirmed()` 信号；Android 壳在 BACK 分支打开框、框开屏蔽点按、取信号置退出请求走既有退出链。

**Tech Stack:** Rust（game-core / game-android / game-shell-common），cargo test，cargo-ndk + gradle，adb 实机。

设计件：`docs/superpowers/specs/2026-10-27-in-game-exit-confirm-design.md`。

---

## File Structure

| 文件 | 职责 | 动作 |
|---|---|---|
| `crates/game-core/src/scene.rs` | 模态状态/接口/tick 拦截/绘制 + 单源抽取（盒尺寸、底角图标）+ 核心单测 `exit_confirm_tests` | Modify |
| `shell/android/src/runtime.rs` | `back_opens_exit_confirm`、框开屏蔽点按、取确认信号置 `shell_exit`、`overlay_state` | Modify |
| `shell/android/src/native.rs` | `handle_input` BACK 分支：根界面立即退出 / 地图开框 | Modify |
| `shell/android/tests/exit_confirm.rs` | 壳宿主钉（5 测） | Create |
| `docs/spec/stage-b-deviations.md` | STAGE-B-005 登记 | Modify |
| `shell/android/README.md` | 输入表 KEYCODE_BACK 行 | Modify |
| `data/status/baselines/rust-test-counts.json` | ratchet | Modify（工具生成） |

---

### Task 1: 核心模态状态 + 接口 + tick 拦截

**Files:**
- Modify: `crates/game-core/src/scene.rs`（struct 尾 ~550、初始化尾 ~1104、`show_overlay_message` ~1362、`hide_notify` ~3011、`tick` 首 ~4534、文件尾新增测试模块）

- [ ] **Step 1: 写失败测试**（scene.rs 文件尾追加）

```rust
/// 【STAGE-B-005】游戏内退出确认框（壳层返回手势打开的核心模态）——设计件
/// docs/superpowers/specs/2026-10-27-in-game-exit-confirm-design.md。
#[cfg(test)]
mod exit_confirm_tests {
    use super::scene_test_support::scene;
    use crate::enums::GameMode;

    fn map_scene() -> super::GameScene {
        let mut sc = scene((2, 3), &[]);
        sc.game_mode = GameMode::Map;
        sc
    }

    #[test]
    fn opens_only_in_map_without_overlay_or_wipe() {
        let mut sc = map_scene();
        assert!(sc.open_exit_confirm(), "地图行走态可开");
        assert!(sc.exit_confirm_open());
        assert!(!sc.open_exit_confirm(), "已开不重开");
        for mode in [GameMode::InGameMenu, GameMode::ItemPanel, GameMode::Dialog, GameMode::SaveSlot] {
            let mut s = map_scene();
            s.game_mode = mode;
            assert!(!s.open_exit_confirm(), "{mode:?} 不开");
        }
        let mut s = map_scene();
        s.show_overlay_message(0, "保存成功！", 0, 0);
        assert!(!s.open_exit_confirm(), "原版浮层开着不开");
        assert!(s.overlay_active && s.overlay_text.as_deref() == Some("保存成功！"), "原版浮层不被冲掉");
        let mut s = map_scene();
        s.transitioning = true;
        assert!(!s.open_exit_confirm(), "换层遮幅中不开");
    }

    #[test]
    fn confirm_keys_emit_signal_once() {
        for key in [-5, -6] {
            let mut sc = map_scene();
            sc.open_exit_confirm();
            sc.press_key(key);
            sc.tick(1, &[]);
            assert!(!sc.exit_confirm_open(), "确认即关框（key {key}）");
            assert!(sc.take_exit_confirmed(), "确认信号（key {key}）");
            assert!(!sc.take_exit_confirmed(), "信号一次性");
            assert_eq!(sc.game_mode, GameMode::Map, "游戏态不变（退出由壳执行；-6 未开背包）");
        }
    }

    #[test]
    fn cancel_closes_and_right_softkey_resumes_original_menu() {
        let mut sc = map_scene();
        sc.open_exit_confirm();
        sc.press_key(-7);
        sc.tick(1, &[]);
        assert!(!sc.exit_confirm_open(), "-7 取消关框");
        assert!(!sc.take_exit_confirmed(), "取消无退出信号");
        assert_eq!(sc.game_mode, GameMode::Map, "该 -7 被框消费，不开游戏内菜单");
        sc.press_key(-7);
        sc.tick(2, &[]);
        assert_eq!(sc.game_mode, GameMode::InGameMenu, "框关后 -7 照旧开游戏内菜单（a.java:3553-3558）");
    }

    #[test]
    fn open_dialog_freezes_game_logic() {
        let mut sc = map_scene();
        let start = sc.view.player_px;
        sc.open_exit_confirm();
        for fc in 1..20 {
            sc.press_key(-4);
            sc.tick(fc, &[]);
        }
        assert!(sc.exit_confirm_open(), "方向键不关框");
        assert_eq!(sc.view.player_px, start, "框开期间玩家不动");
        assert_eq!(sc.key_held, 0, "框内键双清（关框后首拍不误触 tryStep）");
        // 对照：关框后同一方向键可走（证上面不动是框冻结所致）
        sc.press_key(-7);
        sc.tick(20, &[]);
        sc.press_key(-4);
        for fc in 21..30 {
            sc.tick(fc, &[]);
        }
        assert_ne!(sc.view.player_px, start, "关框后右行");
    }

    #[test]
    fn hide_notify_cancels_dialog_then_pauses_to_menu() {
        let mut sc = map_scene();
        sc.open_exit_confirm();
        sc.hide_notify();
        assert!(!sc.exit_confirm_open(), "切后台 = 取消");
        assert!(!sc.take_exit_confirmed());
        assert_eq!(sc.game_mode, GameMode::InGameMenu, "原版暂停：Map → 游戏内菜单");
    }
}
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cargo test --release -p game-core --lib exit_confirm_tests 2>&1 | grep -E "^error|test result"`
Expected: `error[E0599]: no method named `open_exit_confirm`…`（接口不存在）

- [ ] **Step 3: 最小实现**

① 常量与布局结构（`MENU_LABELS` 定义之前、struct 定义之后）：

```rust
/// 【STAGE-B-005】游戏内退出确认框文案：首行 = 原版未用字面量（`f_String_arr_16`，a.java:2298
/// "是否确定退出游戏？"，全文件无消费点）；次行 = 未保存提醒（用户裁定 2026-10-27 Q3=A）。
pub const EXIT_CONFIRM_TEXT: &str = "是否确定退出游戏？\n未保存的进度将会丢失";

/// 浮层底角图标行高（showOverlayMessage a.java:4917-4919 `overlayBoxHeight += 18`）。
const OVERLAY_ICON_ROW_H: i32 = 18;

/// 文本浮层盒尺寸（showOverlayMessage 文本臂 a.java:4904-4909：宽 = 内容宽 + 32、高 = 32 + 可见行 × 行高）。
/// 原版 kind 0/2/4/5 与 STAGE-B-005 确认框共用（单源）。
fn text_overlay_box(content_w: i32, visible_lines: i32, line_h: i32) -> (i32, i32) {
    (content_w + 32, 32 + visible_lines * line_h)
}

/// 【STAGE-B-005】退出确认框布局（独立于原版 overlay_* 状态；kind 4 文本浮层同式）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExitConfirmBox {
    pub lines: Vec<Vec<u16>>,
    pub total_lines: i32,
    pub lines_per_page: i32,
    pub line_height: i32,
    pub box_w: i32,
    pub box_h: i32,
}
```

② `GameScene` struct 尾（`ease_target_y` 之后）：

```rust
    /// 【STAGE-B-005】游戏内退出确认框（壳层返回手势打开；None = 未开）。
    pub exit_confirm: Option<ExitConfirmBox>,
    /// 【STAGE-B-005】退出确认信号（框内确认即置；壳 `take_exit_confirmed` 取走）。
    exit_confirmed: bool,
```

③ `load_floor` 初始化尾（`ease_target_y: 0,` 之后）：

```rust
            exit_confirm: None,
            exit_confirmed: false,
```

④ `show_overlay_message` 文本臂改用单源（行为等价）：

```rust
        if matches!(kind, 0 | 2 | 4 | 5) {
            self.layout_full_text(text, OVERLAY_TEXT_W, OVERLAY_TEXT_MAX_H);
            let (w, h) = text_overlay_box(self.overlay_content_w, self.overlay_page_bottom - self.overlay_page_top, line_h);
            self.overlay_box_w = w;
            self.overlay_box_h = h;
        } else {
```

且尾部 `self.overlay_box_h += 18;` 改为 `self.overlay_box_h += OVERLAY_ICON_ROW_H;`。

⑤ 接口（`impl GameScene`，`show_overlay_message` 之后）：

```rust
    /// 【STAGE-B-005】打开游戏内退出确认框：地图行走态 ∧ 原版浮层未开 ∧ 非换层遮幅 ∧ 未开 ⇒ 布局并打开，
    /// 返回是否打开。仅壳调用（Android 返回手势）；classic 回放/夹具/差分不调用 ⇒ 零偏离。
    pub fn open_exit_confirm(&mut self) -> bool {
        if self.game_mode != GameMode::Map || self.overlay_active || self.transitioning || self.exit_confirm.is_some() {
            return false;
        }
        let font = crate::paint::paint_font();
        let text: Vec<u16> = EXIT_CONFIRM_TEXT.encode_utf16().collect();
        let r = crate::menu_family::layout_wrapped_text(&text, OVERLAY_TEXT_W, OVERLAY_TEXT_MAX_H, text.len(), &font);
        let visible = r.total_lines.min(r.lines_per_page);
        let (box_w, h) = text_overlay_box(r.content_w, visible, r.line_height);
        self.exit_confirm = Some(ExitConfirmBox {
            lines: r.lines,
            total_lines: r.total_lines,
            lines_per_page: r.lines_per_page,
            line_height: r.line_height,
            box_w,
            box_h: h + OVERLAY_ICON_ROW_H,
        });
        true
    }

    /// 【STAGE-B-005】退出确认框是否开着。
    pub fn exit_confirm_open(&self) -> bool {
        self.exit_confirm.is_some()
    }

    /// 【STAGE-B-005】取走退出确认信号（取后清零）。
    pub fn take_exit_confirmed(&mut self) -> bool {
        std::mem::take(&mut self.exit_confirmed)
    }

    /// 【STAGE-B-005】框内键（tick 头拦截）：-5 确定键 / -6 左软键 = 确认；-7 右软键（含再次返回）= 取消；其余吞。
    /// 键值双清同原版浮层 label510 块尾（a.java:3345）——held 残留会让关框后首拍误触发 tryStep。
    fn tick_exit_confirm(&mut self) {
        match self.key_value {
            -5 | -6 => {
                self.exit_confirm = None;
                self.exit_confirmed = true;
            }
            -7 => self.exit_confirm = None,
            _ => {}
        }
        self.key_value = 0;
        self.key_held = 0;
    }
```

⑥ `tick` 首行 `self.frame_counter = frame_counter;` 之后插入：

```rust
        // 【STAGE-B-005】退出确认框（壳层打开的模态）：先于原版浮层拦截——框开即冻结全部逻辑
        if self.exit_confirm.is_some() {
            self.tick_exit_confirm();
            return;
        }
```

⑦ `hide_notify` 首行插入：

```rust
        // 【STAGE-B-005】切后台 = 取消退出确认框，再走原版暂停（Map → 游戏内菜单 + 关音）
        self.exit_confirm = None;
```

- [ ] **Step 4: 跑测试确认通过**

Run: `cargo test --release -p game-core --lib exit_confirm_tests 2>&1 | grep -E "^test |test result"`
Expected: 5 passed

- [ ] **Step 5: 全量回归 + 提交**

Run: `cargo test --release -p game-core 2>&1 | grep -cE "FAILED|panicked"` → `0`
Commit（pathspec）：`feat(core): STAGE-B-005 游戏内退出确认框——核心模态状态/接口/tick 冻结拦截`（正文附 RED 逐字 + GREEN）。

---

### Task 2: 核心绘制（kind 4 样式盒 + 底角 ○/× + 软键 (1,2)）

**Files:**
- Modify: `crates/game-core/src/scene.rs`（`paint_overlay_box` ~861、Map paint 臂 ~6104-6112、`exit_confirm_tests`）

- [ ] **Step 1: 写失败测试**（追加进 `exit_confirm_tests`）

```rust
    fn paint_ops(sc: &mut super::GameScene) -> Vec<String> {
        use crate::render::{ArgbImage, SoftGraphics};
        let mut canvas = ArgbImage::create(sc.layout.screen_w, sc.layout.screen_h);
        let mut g = SoftGraphics::new(&mut canvas);
        sc.paint(&mut g, &ArgbImage::create(64, 64));
        g.ops.clone()
    }

    #[test]
    fn paint_draws_kind4_style_box_and_switches_softkeys() {
        let mut sc = map_scene();
        sc.open_exit_confirm();
        let b = sc.exit_confirm.clone().unwrap();
        let lh = crate::paint::paint_font().height() + 4;
        assert_eq!(b.total_lines, 2, "两行文案");
        assert_eq!(b.box_w, crate::layout::OVERLAY_TEXT_W + 32, "多行 ⇒ 盒宽 = 列宽 + 32（kind 4 同式）");
        assert_eq!(b.box_h, 32 + 2 * lh + 18, "盒高 = 32 + 行 × 行高 + 图标行 18");
        let (bx, by) = ((sc.layout.screen_w - b.box_w) >> 1, (sc.layout.screen_h - b.box_h) >> 1);
        let iy = by + b.box_h - 12 - 18 + 6;
        let left = format!("setClip({},{iy},12,10)", bx + 13);
        let right = format!("setClip({},{iy},12,10)", bx + b.box_w - 25);
        let ops = paint_ops(&mut sc);
        assert!(ops.contains(&left), "左下 ○ 图标位");
        assert!(ops.contains(&right), "右下 × 图标位");
        assert_eq!(sc.softkeys, (1, 2), "框开软键 (1,2)");
        sc.press_key(-7);
        sc.tick(1, &[]);
        let ops2 = paint_ops(&mut sc);
        assert!(!ops2.contains(&left) && !ops2.contains(&right), "关框后不画");
        assert_eq!(sc.softkeys, (1, 3), "关框软键复原 (1,3)（原版 a.java:2421-2422）");
    }
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cargo test --release -p game-core --lib paint_draws_kind4_style_box 2>&1 | grep -E "panicked|test result" -A2`
Expected: FAIL `左下 ○ 图标位`（尚未绘制）

- [ ] **Step 3: 实现**

① 底角图标单源（自由函数，`text_overlay_box` 旁）：

```rust
/// 浮层底角软键图标（paint 公共尾 a.java:3160-3168）：左 x = bx+13、右 x = bx+w−25，y = by+h−12−18+6；
/// ui[11] 精灵条第 n 枚 12×10（实物：1 ○ 确定 / 2 × 取消 / 3 ↩ 返回）。原版浮层与 STAGE-B-005 确认框共用。
#[allow(clippy::too_many_arguments)]
fn paint_overlay_footer_icons(g: &mut SoftGraphics<'_>, ui11: &ArgbImage, bx: i32, by: i32, box_w: i32, box_h: i32, left: i8, right: i8) {
    let iy = by + box_h - 12 - 18;
    if left != 0 {
        draw_image_clipped(g, ui11, bx + 13, iy + 6, (left as i32 - 1) * 12, 0, 12, 10);
    }
    if right != 0 {
        draw_image_clipped(g, ui11, bx + box_w - 25, iy + 6, (right as i32 - 1) * 12, 0, 12, 10);
    }
}
```

`paint_overlay_box` 尾部两段 `if self.overlay_left_icon != 0 {…}` / `if self.overlay_right_icon != 0 {…}`（含 `let iy = …`）替换为：

```rust
            paint_overlay_footer_icons(
                g,
                &self.images.ui[11],
                bx,
                by,
                self.overlay_box_w,
                self.overlay_box_h,
                self.overlay_left_icon,
                self.overlay_right_icon,
            );
```

② 确认框绘制（`impl GameScene`，`paint_overlay_box` 之后）：

```rust
    /// 【STAGE-B-005】退出确认框绘制（kind 4 文本浮层同式：ui[0] 盒画布居中 + 白字换行 + 底角 ○/×）。
    fn paint_exit_confirm(&self, g: &mut SoftGraphics<'_>) {
        let Some(b) = self.exit_confirm.as_ref() else { return };
        let bx = (self.layout.screen_w - b.box_w) >> 1;
        let by = (self.layout.screen_h - b.box_h) >> 1;
        crate::menu_family::paint_box_frame(g, &self.images.ui[0], bx, by, b.box_w, b.box_h);
        g.set_color(WHITE);
        let font = crate::paint::paint_font();
        let bottom = b.total_lines.min(b.lines_per_page);
        crate::menu_family::paint_wrapped_lines(
            g,
            &b.lines,
            0,
            bottom,
            b.total_lines,
            b.line_height * b.lines_per_page,
            b.line_height,
            bx + 16,
            by + 16,
            OVERLAY_TEXT_W,
            true,
            self.frame_counter as i32,
            &font,
        );
        paint_overlay_footer_icons(g, &self.images.ui[11], bx, by, b.box_w, b.box_h, 1, 2);
    }
```

③ Map paint 臂：`self.softkeys = (1, 3);` 改为

```rust
        // 【STAGE-B-005】退出确认框开 ⇒ (1,2)（○ 确定 / × 取消）；框关 = 原值 (1,3) 零偏离
        self.softkeys = if self.exit_confirm.is_some() { (1, 2) } else { (1, 3) };
```

并在 `if !self.transitioning && self.overlay_active { self.paint_overlay_box(g); }` 之后、`self.paint_wipe_tail(g);` 之前插入：

```rust
        // 【STAGE-B-005】退出确认框（原版浮层之后、遮幅尾之前）
        self.paint_exit_confirm(g);
```

- [ ] **Step 4: 跑测试确认通过 + 等价重构回归**

Run: `cargo test --release -p game-core --lib exit_confirm_tests 2>&1 | grep "test result"` → 6 passed
Run: `cargo test --release -p game-core -p game-oracle 2>&1 | grep -cE "FAILED|panicked"` → `0`（classic 帧夹具 = 浮层等价重构兜底）
Run: `python3 tools/modern_frame_baseline.py --check` → 5 状态 sha 不变

- [ ] **Step 5: 提交**

`feat(core): STAGE-B-005 退出确认框绘制——kind 4 样式盒 + 底角 ○/× + 软键 (1,2)；浮层盒尺寸/底角图标单源`

---

### Task 3: Android 壳接线

**Files:**
- Modify: `shell/android/src/runtime.rs`（`back_requests_exit` 之后、`overlay_state`、`step_scene`）
- Modify: `shell/android/src/native.rs`（`handle_input` BACK 分支）
- Create: `shell/android/tests/exit_confirm.rs`

- [ ] **Step 1: 写失败测试** `shell/android/tests/exit_confirm.rs`

```rust
//! STAGE-B-005 游戏内退出确认框·壳宿主钉（runtime.rs = android_main 同一调度体）。
//! 用户 2026-10-27："游戏内右滑弹出退出确认菜单，样式就用游戏内弹框样式"（Q1 地图态 / Q2 确定即退出）。

use game_android::input::KeyAction;
use game_android::native::{OverlayState, TitleGesture};
use game_android::runtime::{build_boot_machine, Runtime};
use game_core::enums::GameMode;
use game_core::layout::Layout;
use game_shell_common::touch::TouchGeom;

fn runtime(tag: &str) -> (Runtime, game_shell_common::asset::DirAssetSource, std::path::PathBuf) {
    let raw = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw");
    let src = game_shell_common::asset::DirAssetSource::new(&raw);
    let dir = std::env::temp_dir().join(format!("jjxy-confirm-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (rms, stores) = game_shell_common::rms::FileRms::load(&dir);
    let machine = build_boot_machine(&src, Layout::modern(7, 10));
    let mut rt = Runtime::new(machine, stores, Box::new(rms));
    rt.apply_layouts(Layout::modern(7, 10), Layout::modern(12, 16));
    (rt, src, dir)
}

fn drive(rt: &mut Runtime, src: &dyn game_platform::AssetSource, keys: &[KeyAction], taps: Vec<(f32, f32)>) {
    rt.step(src, keys, Vec::<TitleGesture>::new(), taps);
    let mut canvas = game_core::render::ArgbImage::create(576, 576);
    let mut g = game_android::native::canvas_graphics(&mut canvas, 1);
    rt.paint(&mut g);
}

fn press(rt: &mut Runtime, src: &dyn game_platform::AssetSource, code: i32) {
    drive(rt, src, &[KeyAction::Press(code), KeyAction::Release], Vec::new());
}

/// 新游戏 → 序章跳过 → 交接 → 剧情推完 → Map 稳定 30 拍。
fn to_map(rt: &mut Runtime, src: &dyn game_platform::AssetSource) {
    assert!((0..400).any(|_| {
        drive(rt, src, &[], Vec::new());
        rt.title_mode()
    }));
    press(rt, src, -5);
    press(rt, src, -7);
    assert!((0..400).any(|_| {
        drive(rt, src, &[], Vec::new());
        rt.scene.is_some()
    }));
    let mut calm = 0;
    for i in 0..4000 {
        let mode = rt.scene.as_ref().unwrap().game_mode;
        calm = if mode == GameMode::Map { calm + 1 } else { 0 };
        if calm >= 30 {
            return;
        }
        let keys: &[KeyAction] =
            if mode != GameMode::Map && i % 4 == 0 { &[KeyAction::Press(-5), KeyAction::Release] } else { &[] };
        drive(rt, src, keys, Vec::new());
    }
    panic!("剧情未推完");
}

fn mode(rt: &Runtime) -> GameMode {
    rt.scene.as_ref().unwrap().game_mode
}

fn open(rt: &Runtime) -> bool {
    rt.scene.as_ref().unwrap().exit_confirm_open()
}

#[test]
fn back_on_map_opens_confirm_instead_of_menu() {
    let (mut rt, src, dir) = runtime("open");
    to_map(&mut rt, &src);
    assert!(rt.back_opens_exit_confirm(), "地图态返回 = 开退出确认框");
    for _ in 0..3 {
        drive(&mut rt, &src, &[], Vec::new());
    }
    assert!(open(&rt) && mode(&rt) == GameMode::Map, "框开、未进游戏内菜单");
    assert_eq!(rt.softkeys(), (1, 2), "面板软键 ○ / ×");
    assert!(!matches!(rt.overlay_state(), OverlayState::Game), "框开 ⇒ 触屏按 UI 态");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn panel_right_softkey_still_opens_in_game_menu() {
    let (mut rt, src, dir) = runtime("panel");
    to_map(&mut rt, &src);
    press(&mut rt, &src, -7); // 面板右软键 = 键批 -7（不经 BACK 分支）
    assert_eq!(mode(&rt), GameMode::InGameMenu, "右软键照旧开游戏内菜单");
    assert!(!rt.back_opens_exit_confirm(), "游戏内菜单里返回不开框（照旧 -7 返回）");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn taps_ignored_while_confirm_open_then_work_after_cancel() {
    let (mut rt, src, dir) = runtime("taps");
    to_map(&mut rt, &src);
    let tap = {
        let sc = rt.scene.as_ref().unwrap();
        let geom = TouchGeom {
            cam_x: sc.view.cam_x,
            cam_y: sc.view.cam_y,
            tile_off: sc.layout.view_top_y,
            cols: sc.view.wide,
            rows: sc.view.high,
        };
        let me = (sc.player_cell_x, sc.player_cell_y);
        let half = TouchGeom::TILE / 2;
        (0..geom.rows)
            .flat_map(|cy| (0..geom.cols).map(move |cx| (cx, cy)))
            .filter(|&c| c != me && sc.walkable.get(c.1 as usize).and_then(|r| r.get(c.0 as usize)) == Some(&true))
            .map(|(cx, cy)| {
                let (ox, oy) = geom.cell_screen_xy(cx, cy);
                (ox + half, oy + half)
            })
            .find(|&(x, y)| {
                x >= 0 && y >= 20 && x < sc.layout.screen_w - 60 && y < sc.layout.screen_h
            })
            .map(|(x, y)| (x as f32, y as f32))
            .expect("视口内有可走格")
    };
    assert!(rt.back_opens_exit_confirm());
    drive(&mut rt, &src, &[], vec![tap]);
    assert!(open(&rt) && mode(&rt) == GameMode::Map, "框开：点地不寻路");
    press(&mut rt, &src, -7);
    assert!(!open(&rt), "-7 取消");
    drive(&mut rt, &src, &[], vec![tap]);
    assert_eq!(mode(&rt), GameMode::Dialog, "对照：关框后同一点按 = 寻路（walk_to 进 Dialog 语境）");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn confirm_requests_process_exit() {
    for key in [-5, -6] {
        let (mut rt, src, dir) = runtime(&format!("ok{}", -key));
        to_map(&mut rt, &src);
        assert!(rt.back_opens_exit_confirm());
        assert!(!rt.exit_requested());
        press(&mut rt, &src, key);
        assert!(rt.exit_requested(), "框内确认（key {key}）= 退出进程");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn back_again_cancels() {
    let (mut rt, src, dir) = runtime("again");
    to_map(&mut rt, &src);
    assert!(rt.back_opens_exit_confirm());
    assert!(!rt.back_opens_exit_confirm(), "框已开：返回不再开框（照旧投 -7）");
    press(&mut rt, &src, -7);
    assert!(!open(&rt) && !rt.exit_requested(), "再次返回 = 取消");
    assert_eq!(mode(&rt), GameMode::Map);
    let _ = std::fs::remove_dir_all(&dir);
}
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cargo test --release -p game-android --test exit_confirm 2>&1 | grep -E "^error" | head -3`
Expected: `error[E0599]: no method named `back_opens_exit_confirm` found for struct `Runtime``

- [ ] **Step 3: 实现**

① `runtime.rs`（`back_requests_exit` 之后）：

```rust
    /// 【STAGE-B-005 · 壳层输入映射】游戏内返回（BACK 键 / 边缘返回手势）：场景活动 ∧ 确认框未开 ⇒ 交核心
    /// `open_exit_confirm`（仅地图行走态 ∧ 无原版浮层 ∧ 非遮幅才开），开了返回 true（调用方丢弃该 BACK 的 -7）。
    /// 框已开 / 其余游戏内界面 ⇒ false（BACK 照旧 -7：框内 = 取消；菜单返回 / 对话推进…）。
    /// 面板右软键不经此口（地图上照旧开游戏内菜单）。
    pub fn back_opens_exit_confirm(&mut self) -> bool {
        match self.scene.as_mut() {
            Some(sc) if !sc.exit_confirm_open() => sc.open_exit_confirm(),
            _ => false,
        }
    }
```

② `overlay_state()` 首臂改为：

```rust
            Some(sc) if sc.game_mode == GameMode::Map && !sc.exit_confirm_open() => OverlayState::Game,
```

③ `step_scene`：`let mut batch = keys.to_vec();` 之后插入

```rust
        // 【STAGE-B-005】退出确认框开 ⇒ 画面点按不生效（不寻路/探步/撤销；键照常交核心框内处理）
        let taps = if sc.exit_confirm_open() {
            if !taps.is_empty() {
                log_input("taps ignored: exit confirm open");
            }
            Vec::new()
        } else {
            taps
        };
```

并在 tick（`catch_unwind` 块）之后、`take_global_dirty` 之前插入：

```rust
        // 【STAGE-B-005】框内确认 ⇒ 退出进程（同根界面返回：不再出拍 → finish → 销毁后 _exit）
        if sc.take_exit_confirmed() {
            self.shell_exit = true;
            log_input("exit confirm → 立即退出");
        }
```

④ `native.rs handle_input`：原 BACK 单条件块替换为

```rust
        const AINPUT_EVENT_TYPE_KEY: i32 = 1;
        let back_down = ffi::AInputEvent_getType(event) == AINPUT_EVENT_TYPE_KEY
            && ffi::AKeyEvent_getKeyCode(event) == AKEYCODE_BACK
            && ffi::AKeyEvent_getAction(event) == AKEY_EVENT_ACTION_DOWN
            && ffi::AKeyEvent_getRepeatCount(event) == 0;
        if back_down {
            if st.rt.back_requests_exit() {
                log_input(&format!("back at root mode={:?} → 立即退出", st.rt.machine.mode));
                keys.clear();
            } else if st.rt.back_opens_exit_confirm() {
                // 【STAGE-B-005】地图行走态返回 = 退出确认框（丢弃该 BACK 的 -7）
                log_input("back in game → 退出确认框");
                keys.clear();
            }
        }
```

（块前注释同步：根界面 BUG-034 / 地图态 STAGE-B-005 / 其余 -7。）

- [ ] **Step 4: 跑测试确认通过**

Run: `cargo test --release -p game-android --test exit_confirm 2>&1 | grep -E "^test |test result"` → 5 passed
Run: `cargo test --release -p game-android 2>&1 | grep -cE "FAILED|panicked"` → `0`
Run: `cargo ndk -t arm64-v8a -P 26 -o android/app/src/main/jniLibs build --offline --release -p game-android 2>&1 | grep -E "^error|Finished"` → Finished

- [ ] **Step 5: 提交**

`feat(android): STAGE-B-005 地图态右滑 ⇒ 退出确认框；框开屏蔽点按、确认即退出进程`

---

### Task 4: 登记与文档

**Files:**
- Modify: `docs/spec/stage-b-deviations.md`（文件尾追加 STAGE-B-005）
- Modify: `shell/android/README.md`（KEYCODE_BACK 行）
- Modify: `docs/knowledge/kairos/2026/10/2026-10-27.md`（记事一行）

- [ ] **Step 1: STAGE-B-005 登记**（追加）

```markdown
## STAGE-B-005 游戏内退出确认框（Android 返回手势）

- **状态**：已实施（2026-10-27）
- **用户裁定原话**（2026-10-27）："游戏内右滑弹出推出确认菜单，样式就用游戏内弹框样式"；逐项：触发范围 A
  （只在地图行走态）、确定后 A（直接退出进程）、文案 A（加未保存提醒）、实现层方案 2（放进游戏核心）。
- **偏离点**：原版**无**退出确认框（`f_String_arr_16` 的 "是否确定退出游戏？" a.java:2298 全文件无消费点）；
  Rust 新增核心模态 `GameScene::exit_confirm`（独立于原版 `overlay_*`）：样式 = 原版 kind 4 文本浮层
  （a.java:3129-3168 盒/白字/底角 ui[11] 图标 1 ○、2 ×），键 = kind 4 同义（-6/-5 确认、-7 取消，a.java:3308-3318），
  软键 (1,2)；框开期间 tick 头拦截冻结逻辑。文案次行"未保存的进度将会丢失"为新增文本。
- **门控**：仅壳调用 `open_exit_confirm` 可达（Android `handle_input` BACK 分支，地图行走态）；classic 回放/夹具/差分
  不调用 ⇒ **classic 零偏离、无新增豁免**。附带等价重构：浮层盒尺寸（`text_overlay_box`）与底角图标
  （`paint_overlay_footer_icons`）单源共用——op 序列不变（classic 帧夹具全绿兜底）。
- **金样影响**：无（Modern 帧基线 5 状态 sha 不变；新增宿主钉 scene.rs `exit_confirm_tests` 6 测 +
  shell/android/tests/exit_confirm.rs 5 测）。
- **代码注释**：`// STAGE-B-005`（scene.rs / runtime.rs / native.rs）。
- **设计件**：docs/superpowers/specs/2026-10-27-in-game-exit-confirm-design.md。
```

- [ ] **Step 2: README KEYCODE_BACK 行**改为：

```markdown
| KEYCODE_BACK | **根界面（开机 logo / 主菜单 / 退场动画）= 立即退出**；**地图行走态 = 退出确认框**（STAGE-B-005：○ 确定即退出 / × 取消）；其余 **-7**；恒吞默认 | 根界面 = Nokia 红键语义（`CMidlet.destroyApp` 为空，跳过 logo 退场动画不丢状态；BUG-034）。【2026-10-27 更正】BUG-033 版"标题态 = 选中「退出」项 + 确认 -5"走 mode 22 logo 动画，用户实机形似重播片头、动画期返回无效，已弃。地图态面板右软键仍 = -7 开游戏内菜单（只系统返回开框）；框开时返回 = -7 = 取消。其余态 -7 = 子页返回主菜单 / 序章跳过 / 游戏内菜单返回；不吞 ⇒ 系统 finish Activity ⇒ 绕过 m_000 退出链（BUG-020）；repeat 丢键仍吞 |
```

- [ ] **Step 3: kairos 一行**（文件尾追加）：

```markdown
- HH:MM — STAGE-B-005 游戏内退出确认框（设计 4debeb6 / 核心 + 壳 + 登记）：地图态右滑 ⇒ 原版 kind 4 样式框（"是否确定退出游戏？"原版未用字面量 + 未保存提醒），○ 确定即退出进程
```

- [ ] **Step 4: 提交**：`docs(spec): STAGE-B-005 登记 + README 输入表 + kairos`

---

### Task 5: ratchet

- [ ] **Step 1:** Run `python3 gates/cli.py ratchet-update`，核对 diff 仅 `game_core(lib)` +6、`exit_confirm(test)` +5、total +11。
- [ ] **Step 2:** 提交 `gate(baselines): ratchet 入账（exit_confirm_tests 6 + exit_confirm 5，STAGE-B-005）`。

---

### Task 6: 实机验证（本方运行，Xiaomi 14 `c362a15b`）

- [ ] **Step 1:** 构建安装：`cargo ndk … build` → `gradle assembleDebug` → `adb -s c362a15b install -r …`；核装机包 .so sha256 = 本地构建。
- [ ] **Step 2:** 前台守卫（桌面/本游戏、近 1 min 无交互）后 `am start`；继续游戏/新游戏进地图（`/tmp/jjxy_tap.sh` 注入，首触防变暗吞）。
- [ ] **Step 3:** 注入左缘右滑（`input swipe 3 1300 650 1300 250`）⇒ logcat `back in game → 退出确认框`；截屏核：框居中、两行白字、左下 ○ 右下 ×、面板软键 ○ / ×。
- [ ] **Step 4:** 点面板右软键（× / -7）⇒ 框关、继续行走（方向键一步，位置变化）。
- [ ] **Step 5:** 再右滑开框 → 点确定（888,2250）⇒ `exit confirm → 立即退出` → `ANativeActivity_finish` → `process exit`、AM `has died: cch`、crash 缓冲空。
- [ ] **Step 6:** 再进游戏开框 → `input keyevent 3`（HOME）⇒ 回前台见游戏内菜单（原版暂停）、框已关。
- [ ] **Step 7:** 实机结果入 STAGE-B-005 登记"实机复核"注 + 提交。
