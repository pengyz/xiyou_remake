//! NativeActivity 胶水（骨架：纯装配态宿主可编译；NDK extern 全在 cfg(android) 子模块）。
//!
//! # TODO(骨架·未验证) 边界
//!
//! 本模块是 **Android 装配骨架**：事件环/窗口呈现/四 trait 接线的组织形状，
//! **未经 SDK/NDK 编译与真机验证**（本环境无 Android 工具链）。模块本体（不透明
//! 类型 + 纯装配态）宿主可编译可测；真正的 NDK 调用在 `process::android`/
//! `audio::aaudio`/`asset::apk`（`#[cfg(target_os = "android")]`，签名照头文件手写
//! 未验证）与本模块 TODO 函数体。类型/符号正确性待 `cargo ndk` 交叉编译升 A。
//!
//! # 接线表（契约 §5 映射表 + 对抗审查整改后口径）
//!
//! | 接缝 | 装配 |
//! |---|---|
//! | Storage | `AndroidStorage::open(activity->internalDataPath)`（应用私有目录注入） |
//! | AssetSource | `apk::ApkAssetSource::new(activity->assetManager)`（APK assets，非 std::fs） |
//! | AudioOut | `aaudio::AAudioPcm::open()` + `game_shell_common::audio::audio_loop`（阻塞 write=节拍）；轨道 `load_tracks_from_assets`（AssetSource，B2） |
//! | ProcessControl | `AndroidProcessControl::new(activity)`（ANativeActivity_finish，不杀进程） |
//! | 输入 | `AInputEvent` → `input::InputRouter`（触控翻译 + 折叠单键状态机事件流；BACK 恒吞默认） |
//! | 生命周期 | `APP_CMD_RESUME/PAUSE` → `lifecycle::Visibility`（onPause⇒hideNotify；PiP/失焦口径见 lifecycle.rs） |
//! | 呈现 | `ANativeWindow` lock/blit 240×320 等比信箱（Viewport 语义同 shell/win view.rs，暂不上移） |
//! | 节拍 | 75ms 逻辑拍（mode 0 的 100ms 走 BootMachine::frame_interval_ms），阻塞 write 为执行侧节拍 |

#![allow(dead_code, unused_variables)]

use crate::input::{map_hw_key, HwKey, KeyAction, PadLayout, TouchEvent, TouchMapper, TouchPhase};
use crate::lifecycle::{Notify, Visibility};

/// 逻辑画布（Stage A 240×320；game_core::layout 同值）。
pub const CANVAS_W: u32 = 240;
pub const CANVAS_H: u32 = 320;

/// 【#2 横屏 Modern】格数（任意格数×32=逻辑边长，layout.rs modern 公式）——
/// 真机分辨率矩阵可调（lead 候选 A13/B18/C15/D37，默认 B=18）。
pub const GRID_N: i32 = 18;
/// 【#2】呈现缩放（整数倍；游戏区像素边长=GRID_N*32*SCALE——矩阵可调，default 2x）。
pub const SCALE: i32 = 2;

/// 逻辑画布（Modern 方画幅 = GRID_N×32 见方）。
pub fn logical_canvas() -> (u32, u32) {
    ((GRID_N * 32) as u32, (GRID_N * 32) as u32)
}

/// 【#2 动态分辨率选择】（用户批准策略）三步算 (grid, scale)：
/// ① scale=最大整数倍（3x/2x/1x）使方画幅 ≤ min(屏宽,屏高)（横屏=屏高）；
/// ② grid=floor(边长/32/scale)，clamp [13,24]（13=原视野起手，24=防全图失衡）；
/// ③ 触控区=横屏侧区自适应（画幅居中信箱，侧区放触控件/UNDO——fit_rect 承载）。
/// GRID_N/SCALE 常量=测试注入点（覆盖选策略用），默认=runtime 选择。
pub fn pick_resolution(buf_w: u32, buf_h: u32) -> (u32, u32) {
    let side = buf_w.min(buf_h);
    for scale in [3u32, 2, 1] {
        let raw = side / (32 * scale);
        if raw >= 13 {
            return (raw.clamp(13, 24), scale);
        }
    }
    (13, 1) // 小屏：clamp 下限（越界由 fit_rect 等比缩进兜底）
}

/// 游戏区像素（呈现面 = 逻辑×SCALE）。
pub fn game_pixels() -> (u32, u32) {
    let (w, h) = logical_canvas();
    (w * SCALE as u32, h * SCALE as u32)
}

/// 【2026-10-26 字体修复】画布绘制上下文（win 壳 `canvas_graphics` 同口径，task-13 ②）：
/// 背板=逻辑×scale **物理**像素、全幅裁剪、游戏字体、光栅倍率——字形钩子按 scale×格高
/// 直落物理像素（旧 scale=1 逻辑背板+呈现 NN×2 ⇒ 1× 字形放大=马赛克）。
pub fn canvas_graphics(canvas: &mut game_core::render::ArgbImage, scale: u32) -> game_core::render::SoftGraphics<'_> {
    let (w, h) = (canvas.width, canvas.height);
    let mut g = game_core::render::SoftGraphics::new(canvas);
    g.set_clip(0, 0, w, h);
    g.set_scale(scale.max(1) as i32);
    g.set_font(Some(game_core::paint::paint_font()));
    g
}

// ───────────────────────────────────────── NDK 常量（android/input.h + native_app_glue.h）

/// MotionEvent ACTION_*（android/input.h；低 8 位=动作、(action>>8)&0xff=pointer index）。
pub const AMOTION_EVENT_ACTION_DOWN: i32 = 0;
pub const AMOTION_EVENT_ACTION_UP: i32 = 1;
pub const AMOTION_EVENT_ACTION_MOVE: i32 = 2;
pub const AMOTION_EVENT_ACTION_CANCEL: i32 = 3;
pub const AMOTION_EVENT_ACTION_POINTER_DOWN: i32 = 5;
pub const AMOTION_EVENT_ACTION_POINTER_UP: i32 = 6;

/// KeyEvent KEYCODE_*（android/keycodes.h）。
pub const AKEYCODE_BACK: i32 = 4;
pub const AKEYCODE_DPAD_UP: i32 = 19;
pub const AKEYCODE_DPAD_DOWN: i32 = 20;
pub const AKEYCODE_DPAD_LEFT: i32 = 21;
pub const AKEYCODE_DPAD_RIGHT: i32 = 22;
pub const AKEYCODE_DPAD_CENTER: i32 = 23;
pub const AKEYCODE_VOLUME_UP: i32 = 24;
pub const AKEYCODE_VOLUME_DOWN: i32 = 25;
pub const AKEYCODE_SPACE: i32 = 62;
pub const AKEYCODE_ENTER: i32 = 66;
pub const AKEYCODE_MENU: i32 = 82;

/// APP_CMD_*（native_app_glue.h `android_app_cmd`）。
pub const APP_CMD_INIT_WINDOW: i32 = 1;
pub const APP_CMD_TERM_WINDOW: i32 = 2;
pub const APP_CMD_CONTENT_RECT_CHANGED: i32 = 5;
pub const APP_CMD_GAINED_FOCUS: i32 = 6;
pub const APP_CMD_LOST_FOCUS: i32 = 7;
pub const APP_CMD_START: i32 = 10;
pub const APP_CMD_RESUME: i32 = 11;
pub const APP_CMD_PAUSE: i32 = 13;
pub const APP_CMD_STOP: i32 = 14;
pub const APP_CMD_DESTROY: i32 = 15;

/// AKeyEvent action（android/input.h）。
pub const AKEY_EVENT_ACTION_DOWN: i32 = 0;
pub const AKEY_EVENT_ACTION_UP: i32 = 1;

// ───────────────────────────────────────── 纯解析层（宿主可测；FFI 之上的语义面）

/// MotionEvent action → 触控阶段（POINTER_DOWN/UP=多指按/抬；OUTSIDE 等=None 不入管线）。
pub fn motion_phase(action: i32) -> Option<TouchPhase> {
    match action & 0xff {
        AMOTION_EVENT_ACTION_DOWN | AMOTION_EVENT_ACTION_POINTER_DOWN => Some(TouchPhase::Down),
        AMOTION_EVENT_ACTION_UP | AMOTION_EVENT_ACTION_POINTER_UP => Some(TouchPhase::Up),
        AMOTION_EVENT_ACTION_MOVE => Some(TouchPhase::Move),
        AMOTION_EVENT_ACTION_CANCEL => Some(TouchPhase::Cancel),
        _ => None,
    }
}

/// pointer index（action 高位 (action>>8)&0xff——POINTER_DOWN/UP 的触点号）。
pub fn motion_pointer_index(action: i32) -> i32 {
    (action >> 8) & 0xff
}

/// AKEYCODE_* → 硬件键归纳（input::HwKey）。
pub fn hw_key_from_android(keycode: i32) -> HwKey {
    match keycode {
        AKEYCODE_BACK => HwKey::Back,
        AKEYCODE_DPAD_UP => HwKey::DpadUp,
        AKEYCODE_DPAD_DOWN => HwKey::DpadDown,
        AKEYCODE_DPAD_LEFT => HwKey::DpadLeft,
        AKEYCODE_DPAD_RIGHT => HwKey::DpadRight,
        AKEYCODE_DPAD_CENTER => HwKey::DpadCenter,
        AKEYCODE_ENTER => HwKey::Enter,
        AKEYCODE_SPACE => HwKey::Space,
        AKEYCODE_MENU => HwKey::Menu,
        AKEYCODE_VOLUME_UP => HwKey::VolumeUp,
        AKEYCODE_VOLUME_DOWN => HwKey::VolumeDown,
        _ => HwKey::Other,
    }
}

/// KeyEvent → 投递键动作流 + 是否消费（consumed=false ⇒ 系统默认行为）。
/// 语义经 [`crate::input::map_hw_key`]（BACK 恒吞——BUG-020 退出链；repeat 丢弃）。
pub fn key_actions_from_android(keycode: i32, action: i32, repeat: i32) -> (Vec<KeyAction>, bool) {
    let mapped = map_hw_key(hw_key_from_android(keycode), repeat > 0);
    let actions = match action {
        AKEY_EVENT_ACTION_DOWN => mapped.code.map(|c| vec![KeyAction::Press(c)]).unwrap_or_default(),
        AKEY_EVENT_ACTION_UP => {
            if mapped.consumed {
                vec![KeyAction::Release]
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };
    (actions, mapped.consumed)
}

/// APP_CMD_* → 可见性边沿（RESUME⇒on_resume、PAUSE⇒on_pause(in_pip)；其余无边沿）。
pub fn lifecycle_notify(vis: &mut Visibility, cmd: i32, in_pip: bool) -> Option<Notify> {
    match cmd {
        APP_CMD_RESUME => vis.on_resume(),
        APP_CMD_PAUSE => vis.on_pause(in_pip),
        _ => None,
    }
}

/// 游戏区矩形（【Bug2 信箱+#2】固定 SCALE 倍数居中方画幅+侧余触控区）：
/// game=(canvas_w×scale, canvas_h×scale) 居中于 buf；越界=等比缩进（分数倍最近邻）。
/// 钉：2670×1200 + 576@2x=1152 ⇒ (759,24,1152,1152)（侧余 759px 放触控）。
pub fn fit_rect(
    buf_w: u32,
    buf_h: u32,
    canvas_w: u32,
    canvas_h: u32,
    scale: u32,
) -> (u32, u32, u32, u32) {
    let sc = scale.max(1);
    let (mut w, mut h) = (canvas_w * sc, canvas_h * sc);
    if w > buf_w || h > buf_h {
        // 【复核B R2】整数化（f64 截断曾致 w=239≠240⇒出口越一格）：
        // contain 宽=min(buf_w, buf_h*w/ch)、高=min(buf_h, buf_w*h/cw)——纯整数。
        let wi = ((buf_h as u64 * w as u64) / (h as u64).max(1)) as u32;
        let hi = ((buf_w as u64 * h as u64) / (w as u64).max(1)) as u32;
        w = buf_w.min(wi).max(1);
        h = buf_h.min(hi).max(1);
    }
    ((buf_w.saturating_sub(w)) / 2, (buf_h.saturating_sub(h)) / 2, w, h)
}

/// 呈现 blit：画布 ARGB（0xAARRGGBB）→ 窗口 RGBA8888（字节序 [R,G,B,A]）。
/// **逐行按 stride 跳行**（lock 缓冲行距≠width——实机碎裂 Bug1 根因：禁按 width
/// 连写）；游戏区=fit_rect 居中信箱，最近邻整数倍缩放（分数倍 NN 同式）。
/// 返回 blit 矩形 (x,y,w,h)。
pub fn blit_argb_rgba(
    dst: &mut [u8],
    buf_w: u32,
    buf_h: u32,
    stride_px: u32,
    argb: &[u32],
    canvas_w: u32,
    canvas_h: u32,
    scale: u32,
) -> (u32, u32, u32, u32) {
    let (cw, ch) = (canvas_w.max(1), canvas_h.max(1));
    let stride = stride_px.max(buf_w);
    // 【复核A 黄2】信箱=不透黑（0xFF000000）——alpha=0 透明条会在截图工具下透桌面
    for (i, b) in dst.iter_mut().enumerate() {
        *b = if i % 4 == 3 { 0xFF } else { 0 };
    }
    let (x0, y0, w, h) = fit_rect(buf_w, buf_h, cw, ch, scale);
    for y in 0..h.min(buf_h.saturating_sub(y0)) {
        for x in 0..w.min(buf_w.saturating_sub(x0)) {
            // 最近邻源坐标（与 map_touch_xy 同式：floor((x-x0)*cw/w)）
            let sx = ((x as u64 * cw as u64) / w as u64) as u32;
            let sy = ((y as u64 * ch as u64) / h as u64) as u32;
            let Some(px) = argb.get((sy * cw + sx) as usize) else { continue };
            let a = (*px >> 24) & 0xff;
            let r = (*px >> 16) & 0xff;
            let g = (*px >> 8) & 0xff;
            let b = *px & 0xff;
            // 【Bug1】行距=stride（y*stride），列=buf 列（x0+x）
            let off = (((y0 + y) as usize) * (stride as usize) + (x0 + x) as usize) * 4;
            if off + 4 <= dst.len() {
                dst[off] = r as u8;
                dst[off + 1] = g as u8;
                dst[off + 2] = b as u8;
                dst[off + 3] = a as u8;
            }
        }
    }
    (x0, y0, w, h)
}

/// 触点窗口坐标→逻辑画布（【#2】经 fit_rect 反变换：格数/缩放自适配）：
/// logical = floor((x-x0)*canvas_w/w)（与 blit 最近邻同式）；侧余触控区=负坐标
/// （画布外=触控键区，不入格）。
pub fn map_touch_xy(x: f32, y: f32, buf_w: u32, buf_h: u32, grid: u32, scale: u32) -> (f32, f32) {
    // 【复核A 黄3】grid=0 ⇒ 画布 0 ⇒ 除零 NaN 不可达：debug_assert 拦 + max(1) 防御
    // （契约：pick_resolution 恒返 [13,24]，GRID_N 注入点亦应 ≥1）
    debug_assert!(grid >= 1, "grid 须 ≥1（pick_resolution 契约域 [13,24]）");
    let (cw, ch) = ((grid.max(1)) * 32, (grid.max(1)) * 32);
    let (x0, y0, w, h) = fit_rect(buf_w, buf_h, cw, ch, scale);
    // 【复核B R1】侧余**负坐标**四向对称（左/上线性负；右/下=溢出逻辑距取负）——
    // UNDO/触控件按符号判区；【复核B R2】出口钳制=逻辑域 [0, grid*32-1]（越一格拒）。
    let map_axis = |v: f32, o: u32, len: u32, span: u32| -> f32 {
        let raw = (v - o as f32) * len as f32 / span as f32;
        if raw < 0.0 {
            raw.floor() // 左/上侧余：线性负
        } else if raw >= len as f32 {
            -((raw - len as f32).floor() + 1.0) // 右/下侧余：负（溢出逻辑距+1）
        } else {
            raw.floor().min(len as f32 - 1.0) // 出口钳制（R2）
        }
    };
    (map_axis(x, x0, cw, w), map_axis(y, y0, ch, h))
}

/// 【2026-10-26 实机根因】一拍内键动作批 → BootMachine 本拍键值（keyValue 语义）。
/// 旧 fold"取末动作"把 [Press, Release] 折成 0 ⇒ 点按/抬手首拍全吞（设备 DIAG 实录）。
/// 对齐 win 壳 `route_key`（shell/win/src/main.rs:654-658）：BootMachine 只读 keyValue，
/// Press = 覆盖式锁存（同拍末个按下胜，a.java:4663-4680 keyPressed 语义），
/// Release 对 BootMachine 无效果（无 keyHeldCode 消费者）。
pub fn boot_key_from_batch(batch: &[KeyAction]) -> i32 {
    batch.iter().fold(0i32, |k, a| match a {
        KeyAction::Press(c) => *c,
        KeyAction::Release => k,
    })
}

/// 【C 方案 2026-10-26】标题态手势（壳记录、拍前经 [`resolve_title_gestures`] 解析为键）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TitleGesture {
    /// 点按（落点=按下点，逻辑坐标；同 GestureTracker Tap 口径）
    Tap(f32, f32),
    /// 水平滑动（一手势至多一次）
    Swipe(game_core::title::TitleTouch),
}

/// 视口滑动阈值（逻辑 px；GestureTracker::new(24) 同值——一格/次的发步口径）。
pub const VP_SWIPE_MIN: i32 = 24;

/// 【计划 3 T4】视口触点（逻辑坐标；Game 区、非标题态——分类在 Runtime 手势簿）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VpTouch {
    pub id: i32,
    pub phase: TouchPhase,
    pub x: i32,
    pub y: i32,
}

/// 标题滑动阈值（逻辑 px；与 GestureTracker::new(24) 同值）。
pub const TITLE_SWIPE_MIN: i32 = 24;

/// 标题手势 → 键动作（拍前解析）。**仅标题态**生效：几何=BootMachine::title_geom
/// 单源；离开标题态的残留手势一律丢弃（防滑动在序章/读档面合成键）。
pub fn resolve_title_gestures(
    machine: &game_core::paint::BootMachine,
    gestures: Vec<TitleGesture>,
) -> Vec<KeyAction> {
    let Some(geo) = machine.title_geom() else { return Vec::new() };
    gestures
        .into_iter()
        .filter_map(|gst| match gst {
            TitleGesture::Tap(x, y) => geo.hit(x as i32, y as i32),
            TitleGesture::Swipe(t) => Some(t),
        })
        .map(|t| KeyAction::Press(t.key()))
        .collect()
}

/// 【进游戏】场景键重放（win 壳 route_key 同口径）：Press = 打断寻路 + press_key
/// （keyValue=keyHeld=code 覆盖式，a.java:4663-4676）；Release = release_key（只清
/// keyHeld，a.java:4678）。按事件到达序逐个重放（同批点按：keyValue 留给下一拍）。
/// 【STAGE-B-008 · 复核 T5-R1】打断用 interrupt_route_for_input：浮层开着时按键属浮层
///（关框 / 翻页），不打断点地路线——路线途经拾取物弹框，关框后继续走向目标。
pub fn replay_scene_keys(sc: &mut game_core::scene::GameScene, batch: &[KeyAction]) {
    for a in batch {
        match *a {
            KeyAction::Press(c) => {
                sc.interrupt_route_for_input();
                sc.press_key(c);
            }
            KeyAction::Release => sc.release_key(),
        }
    }
}

/// 【进游戏】场景点按（逻辑坐标；win 壳 pointer_tap/pointer_click 同口径）：
/// - 场上收点地（[`GameScene::accepts_field_tap`]：地图态或点地寻路中，无浮层 / 遮幅）：
///   触点 → 撤销钮 / 格 → core `tap_cell`（走 / 停靠 / 撞 / 选中 / 排队 / 闪红，STAGE-B-008），
///   返回 None；
/// - 其余：pointer_hit 行命中 → 选行并返回该态确认键（调用方按普通点按投递）；未命中 = 点空白，
///   返回 [`GameScene::blank_tap_key`]（BUG-039：只有继续 / 关闭类情境代按 -5）。
pub fn scene_tap(sc: &mut game_core::scene::GameScene, x: i32, y: i32) -> Option<i32> {
    use game_shell_common::touch::{Dispatcher, ShellAction, TapTarget, TouchGeom};
    if sc.accepts_field_tap() {
        let geom = TouchGeom {
            cam_x: sc.view.cam_x,
            cam_y: sc.view.cam_y,
            tile_off: sc.layout.view_top_y,
            cols: sc.view.wide,
            rows: sc.view.high,
        };
        // 【计划 3 T8 · spec §7】Android 不再调 undo_zone_hit（右上隐形撤销热区去除；撤销=工具栏/摇杆
        // 由计划 3 呈现承担；Windows 保留可见钮走 tap_target）。android 分类 = 图内即格、图外 Ignore。
        let t = match geom.cell_at(x, y) {
            Some((cx, cy)) => TapTarget::Cell(cx, cy),
            None => TapTarget::Outside,
        };
        for a in Dispatcher::tap(t) {
            match a {
                ShellAction::TapCell((cx, cy)) => {
                    let r = sc.tap_cell(cx, cy);
                    log_input(&format!(
                        "field tap ({x},{y}) cam=({},{}) cell=({cx},{cy}) player=({},{}) → {r:?}",
                        geom.cam_x,
                        geom.cam_y,
                        sc.player_cell_x,
                        sc.player_cell_y
                    ));
                }
                ShellAction::Undo => {
                    // BUG-038：只在 field_tap_allowed 时撤销。dp3 步进期恒拒（不打断寻路）；
                    // 被挡步边界拍（Dialog∧wp0∧route 开）会放行撤销——既有行为，后果良性（复核 T10-12 R2）。
                    if sc.field_tap_allowed() {
                        let _ = game_shell_common::undo_ui::undo_last(sc);
                    }
                }
                ShellAction::Interrupt | ShellAction::Swipe(_) | ShellAction::Key(_) | ShellAction::KeyRelease => {}
            }
        }
        return None;
    }
    if let game_core::scene::PointerHit::Row { index, confirm_key } = sc.pointer_hit(x, y) {
        sc.select_row(index);
        return Some(confirm_key);
    }
    // 点空白：对话"点屏幕继续"、提示框关闭代按 -5；要做决定的界面不代按（BUG-039）
    sc.blank_tap_key()
}

/// `ANativeActivity`（不透明；字段面见 android/native_activity.h——internalDataPath
/// / assetManager / vm / clazz 顺序布局，TODO：实机核对后升 A）。
#[repr(C)]
pub struct ANativeActivity {
    _private: [u8; 0],
}

/// `AInputEvent` / `ANativeWindow`（不透明）。
#[repr(C)]
pub struct AInputEvent {
    _private: [u8; 0],
}
#[repr(C)]
pub struct ANativeWindow {
    _private: [u8; 0],
}

/// 装配态（骨架：字段随实现期补全——BootMachine/GameScene/BgmController 等）。
pub struct ShellWiring {
    pub visibility: Visibility,
    pub pad: PadLayout,
    /// 触控翻译（pad 区→键码，折叠单键口径 input.rs）
    pub touch: TouchMapper,
    /// T3 手势（滑动=方向手动踏步；shell/common/touch.rs）
    pub gesture: game_shell_common::touch::GestureTracker,
    /// 呈现表面尺寸（逻辑画布 g*32）
    pub surface: (u32, u32),
    /// 【2c】tap 待决表（画布内 Up，无 pad 键产出）——UI 态 pointer_hit 消费
    pub pending_taps: Vec<(f32, f32)>,
    /// 【计划 3 T4】视口触点队列（Game 区逻辑坐标；Runtime 拍前取走做手势分类）。
    pub pending_vp_touches: Vec<VpTouch>,
    /// 【计划 3 T5 · 复核 T5-R2】工具栏命中队列（Touch 模式；Runtime 拍前取走 perform_action）。
    pub pending_toolbar_actions: Vec<game_core::controls::Action>,
    /// 左右大键当前 Action（Runtime 每拍自 controls() 同步；None=隐藏）。
    pub toolbar_side: Option<(game_core::controls::FunctionId, game_core::controls::Action)>,
    /// 【黄④】触控区描边开关（debug 常驻样式可关——量产/帧哈希前收审美）
    pub overlay_debug: bool,
    /// 【问题2b】窗口/缓冲真实像素尺寸（INIT_WINDOW 实测——触点反变换分母；
    /// 旧误传逻辑尺寸 ⇒ 点击全落侧余负区=触屏全链不通根因）
    pub window_size: (u32, u32),
    /// 【#2】分辨率选择结果 (grid, scale)：None=INIT_WINDOW 时按策略选；
    /// Some=测试注入点（覆盖选策略——GRID_N/SCALE 常量默认值同此用途）
    pub resolution: Option<(u32, u32)>,
    /// 【C 方案】标题态（主循环每拍据 BootMachine::title_geom 置位）：触点只走
    /// 轮播手势，不喂虚拟键区。
    pub title_mode: bool,
    /// 【C 方案】待解析标题手势（拍前 resolve_title_gestures）。
    pub title_gestures: Vec<TitleGesture>,
    /// 标题手势簿：(触点 id, 按下点, 本手势已发滑动)
    title_track: Option<(i32, f32, f32, bool)>,
    /// 【竖屏动态分辨率】竖屏规划（Some=竖屏组成：游戏视口+控制面板；None=旧横屏方画幅）。
    pub portrait: Option<crate::portrait::PortraitPlan>,
    /// 【竖屏】活动画布档（触点→逻辑坐标的缩放/放置依据；主循环每拍按活动机更新）。
    pub active_canvas: Option<crate::portrait::Canvas>,
    /// 【竖屏】触点分区簿（按下时定区：面板/视口；拖出区仍归原区——按钮捕获语义）。
    route: Vec<(i32, TouchRegion)>,
    /// 【竖屏公式化】屏幕密度（AConfiguration_getDensity 经 dpi_or_baseline；dp 换算源）。
    pub dpi: i32,
    /// 【计划 3 T5】操控方式（规划输入；切换 ⇒ replan 生效，apply_portrait 重建 TouchMapper 清按住态）
    pub control_mode: game_shell_common::settings::ControlMode,
    /// 【竖屏公式化】全关卡最大格数（maplv 头；游戏格阵上限）。
    pub map_extent: crate::portrait::MapExtent,
    /// 【竖屏公式化】glue contentRect（窗口坐标 l,t,r,b；None=未收到）——内容区内缩源。
    pub content_rect: Option<(i32, i32, i32, i32)>,
    /// 【按键反馈】待派发的虚拟键按下数（TouchMapper 产出的 Press；输入回调后立即取走）。
    key_presses: u32,
    /// 【STAGE-B-006 · 复核 T6-F3】首帧已呈现（主循环首次呈现成功后置位）；之前的输入一律丢弃。
    presented: bool,
}

/// 【竖屏】触点区。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TouchRegion {
    Panel,
    Game,
}

impl ShellWiring {
    pub fn new() -> ShellWiring {
        let pad = PadLayout::default_240x320();
        ShellWiring {
            visibility: Visibility::new(),
            touch: TouchMapper::new(pad.clone()),
            pad,
            gesture: game_shell_common::touch::GestureTracker::new(24),
            surface: logical_canvas(),
            pending_taps: Vec::new(),
            overlay_debug: true,
            window_size: (0, 0),
            resolution: None,
            title_mode: false,
            title_gestures: Vec::new(),
            title_track: None,
            pending_vp_touches: Vec::new(),
            pending_toolbar_actions: Vec::new(),
            toolbar_side: None,
            portrait: None,
            active_canvas: None,
            route: Vec::new(),
            dpi: crate::portrait::DP_BASELINE_DPI,
            control_mode: game_shell_common::settings::ControlMode::Pad,
            map_extent: crate::portrait::MapExtent::default(),
            content_rect: None,
            key_presses: 0,
            presented: false,
        }
    }

    /// 【按键反馈】取走待派发的虚拟键按下数（feedback::emit 的输入；取走即清）。
    pub fn take_key_presses(&mut self) -> u32 {
        std::mem::take(&mut self.key_presses)
    }

    /// 【STAGE-B-006 · 复核 T6-F3】首帧已呈现（present / present_layers 成功时自动置位；宿主测试可直接调用）。
    pub fn mark_presented(&mut self) {
        self.presented = true;
    }

    /// 【STAGE-B-006 · 复核 T6-F3】取走本拍输入（键批 `keys` / 标题手势 / 画布点按，取走即清）。首帧呈现前一律丢弃：
    /// 用户看到的是黑屏，而开屏关时首拍前已在主菜单——看不见的标题确认区 / 面板确定键接住的点按会直接开始新游戏。
    /// 开屏开时 mode 0 本就不读键，行为不变。
    pub fn take_step_inputs(
        &mut self,
        keys: &mut Vec<KeyAction>,
    ) -> (Vec<KeyAction>, Vec<TitleGesture>, Vec<(f32, f32)>) {
        let k = std::mem::take(keys);
        let g = std::mem::take(&mut self.title_gestures);
        let t = std::mem::take(&mut self.pending_taps);
        if self.presented {
            return (k, g, t);
        }
        // 按下早于首帧、抬起晚于首帧的标题触点：按下半已作废，抬起不得补出 Tap（复核 T6b-N1）
        self.title_track = None;
        if !(k.is_empty() && g.is_empty() && t.is_empty()) {
            log_input(&format!("pre-first-frame input dropped keys={k:?} gestures={g:?} taps={t:?}"));
        }
        (Vec::new(), Vec::new(), Vec::new())
    }

    /// 【竖屏公式化】规划输入（窗口 w×h + 本机密度/内容区/地图——全部设备与数据来源）：
    /// 上缩 = contentRect.top、下缩 = 窗高 − contentRect.bottom（越界/未知 = 0）。
    pub fn portrait_input(&self, w: i32, h: i32) -> crate::portrait::PlanInput {
        let (inset_top, inset_bottom) = match self.content_rect {
            Some((_, t, _, b)) if b > t && b <= h => (t.clamp(0, h), h - b),
            _ => (0, 0),
        };
        crate::portrait::PlanInput {
            window: (w, h),
            dpi: self.dpi,
            inset_top,
            inset_bottom,
            map: self.map_extent,
            control_mode: self.control_mode,
        }
    }

    /// 【竖屏公式化】输入变化（contentRect 到达）后重规划：结果与现行不同才采用并返回
    /// （Some = 宿主须重建画布/面板）；非竖屏或未变 = None。
    /// 【计划 3 T5 · spec §8】切换操控方式：重算规划（触屏档/摇杆档），变化才采用（返回 Some）。
    /// apply_portrait 重建 TouchMapper ⇒ 按住状态清零（key_held 归零，spec 切换即生效条款）。
    pub fn set_control_mode(&mut self, mode: game_shell_common::settings::ControlMode) -> Option<crate::portrait::PortraitPlan> {
        if mode == self.control_mode {
            return None;
        }
        self.control_mode = mode;
        let (w, h) = self.portrait.as_ref()?.window;
        let mut inp = self.portrait_input(w, h);
        inp.control_mode = mode;
        let plan = crate::portrait::plan_portrait(&inp);
        if plan == self.portrait.as_ref()?.clone() {
            return None;
        }
        self.apply_portrait(plan.clone());
        Some(plan)
    }

    pub fn replan_portrait(&mut self) -> Option<crate::portrait::PortraitPlan> {
        let (w, h) = self.portrait.as_ref()?.window;
        let plan = crate::portrait::plan_portrait(&self.portrait_input(w, h));
        if self.portrait.as_ref() == Some(&plan) {
            return None;
        }
        self.apply_portrait(plan.clone());
        Some(plan)
    }

    /// 【竖屏】采用竖屏规划：面板控件=PadLayout（面板局部物理像素），触控翻译换轨；
    /// 画布内调试描边关闭（控件已是面板上的可见 UI）；活动画布初始=开场链档。
    pub fn apply_portrait(&mut self, plan: crate::portrait::PortraitPlan) {
        self.pad = plan.pad.clone();
        self.touch = TouchMapper::new(plan.pad.clone());
        self.window_size = (plan.window.0 as u32, plan.window.1 as u32);
        // 【复核 T5-R1】resolution/surface 按操控方式选档（触屏=scene_touch）
        let (res_g, res_s, surf) = if self.control_mode == game_shell_common::settings::ControlMode::Touch {
            (plan.scene_touch.grid_w as u32, plan.scene_touch.scale as u32, plan.scene_touch.logical())
        } else {
            (plan.scene.grid_w as u32, plan.scene.scale as u32, plan.scene.logical())
        };
        self.resolution = Some((res_g, res_s));
        self.surface = (surf.0 as u32, surf.1 as u32);
        self.overlay_debug = false;
        self.active_canvas = Some(plan.boot);
        self.route.clear();
        self.portrait = Some(plan);
    }

    /// 【竖屏】活动画布档（主循环按活动机每拍更新）。
    pub fn set_active_canvas(&mut self, c: crate::portrait::Canvas) {
        self.active_canvas = Some(c);
    }

    /// 【竖屏】设备坐标触点路由（物理像素，窗口坐标）：按下定区——
    /// 面板 → 面板局部坐标喂 TouchMapper（方向盘/确定/软键出键）；
    /// 视口 → 活动画布逻辑坐标：标题态记轮播手势、其余态 Up 入待决点按（不出键）。
    /// 拖出原区仍归原区（按住方向拖进视口不被截走）。
    pub fn on_touch_device(&mut self, id: i32, phase: TouchPhase, x: f32, y: f32) -> Vec<KeyAction> {
        let Some(plan) = self.portrait.as_ref() else { return Vec::new() };
        let (panel, game) = (plan.panel, plan.game);
        let canvas = self.active_canvas.unwrap_or(plan.boot);
        let region = match phase {
            TouchPhase::Down => {
                let r = if panel.contains(x, y) {
                    Some(TouchRegion::Panel)
                } else if game.contains(x, y) {
                    Some(TouchRegion::Game)
                } else {
                    None
                };
                self.route.retain(|(i, _)| *i != id);
                if let Some(r) = r {
                    self.route.push((id, r));
                }
                r
            }
            _ => self.route.iter().find(|(i, _)| *i == id).map(|(_, r)| *r),
        };
        if matches!(phase, TouchPhase::Up | TouchPhase::Cancel) {
            self.route.retain(|(i, _)| *i != id);
        }
        match region {
            Some(TouchRegion::Panel) => {
                // 【计划 3 T5 · 复核 T5-R2】Touch 模式面板=工具栏：Up 命中 → Action 队列（Runtime
                // perform_action 执行）；Pad 模式照旧摇杆 mapper。
                if self.control_mode == game_shell_common::settings::ControlMode::Touch {
                    if matches!(phase, TouchPhase::Up) {
                        if let Some(plan) = self.portrait.as_ref() {
                            let tb = &plan.toolbar;
                            let hit = game_shell_common::controls_ui::hit_toolbar(
                                (x - tb.bar.x as f32) as i32,
                                (y - tb.bar.y as f32) as i32,
                                (tb.left.x - tb.bar.x, tb.left.y - tb.bar.y, tb.left.w, tb.left.h),
                                (tb.mid.x - tb.bar.x, tb.mid.y - tb.bar.y, tb.mid.w, tb.mid.h),
                                (tb.right.x - tb.bar.x, tb.right.y - tb.bar.y, tb.right.w, tb.right.h),
                                4, // mid 上限 4 均分；idx 越界由 Runtime 消费侧按查询校验
                            );
                            if let Some(h) = hit {
                                use game_shell_common::controls_ui::ToolbarHit;
                                let a = match h {
                                    ToolbarHit::Left => self.toolbar_side.map(|(_, a)| a).unwrap_or(game_core::controls::Action::Key(-6)),
                                    ToolbarHit::Right => self.toolbar_side.map(|(_, a)| a).unwrap_or(game_core::controls::Action::Key(-7)),
                                    ToolbarHit::Mid(i) => game_core::controls::Action::Key(-(i as i32) - 100), // 占位码，Runtime 校验
                                };
                                self.pending_toolbar_actions.push(a);
                            }
                        }
                    }
                    return Vec::new();
                }
                let ev = TouchEvent { id, phase, x: x - panel.x as f32, y: y - panel.y as f32 };
                let keys = self.touch.feed(&ev);
                self.key_presses += crate::feedback::presses_in(&keys);
                keys
            }
            Some(TouchRegion::Game) => {
                let c = canvas.rect_in(game);
                let s = canvas.scale.max(1) as f32;
                let (lx, ly) = (((x - c.x as f32) / s).floor(), ((y - c.y as f32) / s).floor());
                let ev = TouchEvent { id, phase, x: lx, y: ly };
                if self.title_mode {
                    self.on_title_touch(&ev);
                } else {
                    // 【计划 3 T4 · spec §5.3】视口触点原样上报（逻辑坐标）：点按/滑/拖分类由
                    // Runtime 持手势簿判定（本函数纯翻译零状态），消费见 Runtime::step_scene。
                    self.pending_vp_touches.push(VpTouch { id, phase, x: ev.x as i32, y: ev.y as i32 });
                }
                Vec::new()
            }
            None => Vec::new(),
        }
    }

    /// 逻辑拍节奏（毫秒）：mode 0 = 100（a.java:3350）、其余 = 75（a.java:3377）——
    /// shell/win `frame_interval()` 同构（main.rs:645-650）。
    pub fn frame_interval_ms(&self) -> u64 {
        75
    }

    /// 分辨率（grid, scale）：测试注入 resolution 覆盖，否则常量默认（runtime
    /// 选择在 INIT_WINDOW 由 pick_resolution 写入 resolution）。
    pub fn resolved(&self) -> (u32, u32) {
        self.resolution.unwrap_or((GRID_N as u32, SCALE as u32))
    }

    /// 触点事件翻译（宿主可测面）：pad 区→键动作流 + 手势簿登记。
    pub fn on_touch(&mut self, ev: &TouchEvent) -> Vec<KeyAction> {
        self.gesture.down(ev.x as i32, ev.y as i32);
        let keys = self.touch.feed(ev);
        self.key_presses += crate::feedback::presses_in(&keys);
        keys
    }

    /// 触点路由（宿主可测全面；on_input_ex 坐标反变换后调用）：
    /// - 标题态（title_mode）：只记轮播手势（点按/水平滑动，单指），**不喂** pad；
    /// - 其余态：虚拟键区（TouchMapper）出键；画布内 Up 且无键 ⇒ pending_taps
    ///   （UI 行命中=BootMachine::pointer_hit，主循环拍前解析）。
    pub fn on_touch_event(&mut self, ev: &TouchEvent) -> Vec<KeyAction> {
        if self.title_mode {
            self.on_title_touch(ev);
            return Vec::new();
        }
        let keys = self.on_touch(ev);
        if matches!(ev.phase, TouchPhase::Up) && ev.x >= 0.0 && ev.y >= 0.0 && keys.is_empty() {
            self.pending_taps.push((ev.x, ev.y));
        }
        keys
    }

    /// 标题态单指手势：越阈水平滑动=一次 Swipe；未滑抬起=Tap(按下点)。
    fn on_title_touch(&mut self, ev: &TouchEvent) {
        match ev.phase {
            TouchPhase::Down => {
                if self.title_track.is_none() {
                    self.title_track = Some((ev.id, ev.x, ev.y, false));
                }
            }
            TouchPhase::Move => {
                if let Some((id, x0, y0, fired)) = self.title_track.as_mut() {
                    if *id == ev.id && !*fired {
                        let (dx, dy) = ((ev.x - *x0) as i32, (ev.y - *y0) as i32);
                        if let Some(t) = game_core::title::title_swipe(dx, dy, TITLE_SWIPE_MIN) {
                            *fired = true;
                            self.title_gestures.push(TitleGesture::Swipe(t));
                        }
                    }
                }
            }
            TouchPhase::Up | TouchPhase::Cancel => {
                if let Some((id, x0, y0, fired)) = self.title_track {
                    if id == ev.id {
                        self.title_track = None;
                        if !fired && matches!(ev.phase, TouchPhase::Up) {
                            self.title_gestures.push(TitleGesture::Tap(x0, y0));
                        }
                    }
                }
            }
        }
    }

    /// 硬件键翻译（宿主可测面）：AKEYCODE 语义→-1..-7（BACK 恒 consumed——BUG-020）。
    pub fn on_hw_key(&mut self, keycode: i32, action: i32, repeat: i32) -> (Vec<KeyAction>, bool) {
        key_actions_from_android(keycode, action, repeat)
    }

    /// 输入事件翻译（NDK 解析→on_touch/on_hw_key 纯面）。
    /// **BACK 恒返回 consumed**（BUG-020 退出链：不得走系统默认 finish）。
    pub fn on_input(&mut self, event: *mut AInputEvent) -> Vec<KeyAction> {
        self.on_input_ex(event).0
    }

    /// 解析+翻译并回消费位（glue onInputEvent 返回值用）。
    pub fn on_input_ex(&mut self, event: *mut AInputEvent) -> (Vec<KeyAction>, bool) {
        #[cfg(target_os = "android")]
        unsafe {
            const AINPUT_EVENT_TYPE_KEY: i32 = 1;
            const AINPUT_EVENT_TYPE_MOTION: i32 = 2;
            match ffi::AInputEvent_getType(event) {
                AINPUT_EVENT_TYPE_MOTION => {
                    let action = ffi::AMotionEvent_getAction(event);
                    let Some(phase) = motion_phase(action) else { return (Vec::new(), true) };
                    let idx = motion_pointer_index(action) as usize;
                    let id = ffi::AMotionEvent_getPointerId(event, idx);
                    let (g, sc) = self.resolved();
                    let (bx, by) = (ffi::AMotionEvent_getX(event, idx), ffi::AMotionEvent_getY(event, idx));
                    // 【竖屏】设备坐标分区路由（面板=控件，视口=游戏）
                    if self.portrait.is_some() {
                        let keys = self.on_touch_device(id, phase, bx, by);
                        log_input(&format!("motion phase={phase:?} raw=({bx:.0},{by:.0}) portrait keys={keys:?}"));
                        return (keys, true);
                    }
                    // 【问题2b】反变换分母=真实窗口尺寸（旧传 self.surface=逻辑尺寸 ⇒ 全落侧余）
                    let (ww, wh) = if self.window_size.0 > 0 {
                        self.window_size
                    } else {
                        self.surface
                    };
                    let (x, y) = map_touch_xy(bx, by, ww, wh, g, sc);
                    log_input(&format!(
                        "motion phase={phase:?} raw=({bx:.0},{by:.0}) buf=({ww}x{wh}) logical=({x:.0},{y:.0})"
                    ));
                    let ev = TouchEvent { id, phase, x, y };
                    let keys = self.on_touch_event(&ev);
                    log_input(&format!(
                        "touch keys={keys:?} in_canvas={} title_mode={}",
                        x >= 0.0 && y >= 0.0,
                        self.title_mode
                    ));
                    (keys, true) // 触点恒吞（触摸屏无系统默认）
                }
                AINPUT_EVENT_TYPE_KEY => {
                    let code = ffi::AKeyEvent_getKeyCode(event);
                    let act = ffi::AKeyEvent_getAction(event);
                    let rep = ffi::AKeyEvent_getRepeatCount(event);
                    self.on_hw_key(code, act, rep)
                }
                _ => (Vec::new(), false),
            }
        }
        #[cfg(not(target_os = "android"))]
        {
            let _ = event;
            (Vec::new(), false)
        }
    }

    /// 生命周期回调（骨架）：APP_CMD_RESUME ⇒ on_resume；APP_CMD_PAUSE ⇒ on_pause
    /// （in_pip = isInPictureInPictureMode——PiP=可见非遮挡不发 Hide，lifecycle.rs S1）。
    /// 边沿消费 = game-core `hide_notify()`（mode3→4 + closeAudioPlayer + anyKeyPlayed）
    /// 与 showNotify（只清 latch）——壳只决定何时调（shell/win main.rs:578-622 同构）。
    pub fn on_lifecycle(&mut self, cmd: i32, in_pip: bool) -> Option<Notify> {
        lifecycle_notify(&mut self.visibility, cmd, in_pip)
    }

    /// 窗口就绪判定+格式协商（APP_CMD_INIT_WINDOW 处理内调用）：
    /// **window=NULL ⇒ No-op**（实机 SIGSEGV 回派根因：android_main 进入时
    /// app->window 未就绪——glue 在 INIT_WINDOW 后才非空、TERM_WINDOW 即置 NULL
    /// （native_app_glue.c:105/145），未就绪/竞态一律不得触 FFI）。
    /// 返回 true=已协商（格式 RGBA8888、几何 (0,0)=buffer 留显示原生比例）。
    pub fn ensure_surface(&mut self, window: *mut ANativeWindow) -> bool {
        if window.is_null() {
            return false;
        }
        #[cfg(target_os = "android")]
        unsafe {
            const WINDOW_FORMAT_RGBA_8888: i32 = 1;
            // 【Bug2 根治】只协商**格式**（几何 (0,0)=buffer 留显示原生尺寸比例）——
            // 几何设 240×320 会让 SurfaceFlinger 全屏硬拉压扁。
            ffi::ANativeWindow_setBuffersGeometry(window, 0, 0, WINDOW_FORMAT_RGBA_8888);
        }
        // 【#2】runtime 选择：按 buffer/显示尺寸三步算（测试注入 resolution 不覆盖）
        if self.resolution.is_none() {
            #[cfg(target_os = "android")]
            unsafe {
                let bw = ffi::ANativeWindow_getWidth(window).max(0) as u32;
                let bh = ffi::ANativeWindow_getHeight(window).max(0) as u32;
                if bw > 0 && bh >= bw {
                    // 【竖屏动态分辨率】竖窗=游戏视口+控制面板组成（portrait.rs 公式规划）
                    let plan = crate::portrait::plan_portrait(&self.portrait_input(bw as i32, bh as i32));
                    self.apply_portrait(plan);
                } else if bw > 0 && bh > 0 {
                    self.window_size = (bw, bh); // 【问题2b】真实缓冲尺寸
                    let (g, sc) = pick_resolution(bw, bh);
                    self.resolution = Some((g, sc));
                    self.surface = (g * 32, g * 32);
                    // 【复核B R3】触控件随选策略生成（贴底/贴右 Modern 布局）
                    self.pad = crate::input::PadLayout::for_modern(g as i32);
                    self.touch = TouchMapper::new(self.pad.clone());
                }
            }
        }
        if self.resolution.is_none() {
            self.surface = logical_canvas();
        }
        true
    }

    /// 【竖屏】多层呈现：锁窗缓冲 → compose_layers（清不透黑 + 各层 1:1 拷入）→ post。
    /// 层 = (ARGB, 宽, 高, x, y)（窗口物理像素坐标）。window=NULL ⇒ No-op 返回 false。
    pub fn present_layers(&mut self, window: *mut ANativeWindow, layers: &[(&[u32], i32, i32, i32, i32)]) -> bool {
        if window.is_null() {
            return false;
        }
        #[cfg(target_os = "android")]
        unsafe {
            let mut buf = ANativeWindowBuffer {
                width: 0,
                height: 0,
                stride: 0,
                format: 0,
                bits: std::ptr::null_mut(),
            };
            if ffi::ANativeWindow_lock(window, &mut buf, std::ptr::null_mut()) != 0 {
                return false;
            }
            if !buf.bits.is_null() && buf.width > 0 && buf.height > 0 {
                let total = (buf.stride.max(0) as usize) * (buf.height as usize) * 4;
                let dst = std::slice::from_raw_parts_mut(buf.bits as *mut u8, total);
                crate::portrait::compose_layers(dst, buf.width, buf.height, buf.stride, layers);
            }
            ffi::ANativeWindow_unlockAndPost(window);
        }
        #[cfg(not(target_os = "android"))]
        {
            let _ = layers;
        }
        self.presented = true; // 【STAGE-B-006 · 复核 T6b-N3】首帧已呈现 ⇒ 输入放行
        true
    }

    /// 呈现：ArgbImage → ANativeWindow 缓冲（等比信箱 blit——buffer=显示原生
    /// 尺寸比例，fit_rect 居中信箱+NN 缩放；SoftGraphics ARGB 直拷换算）。
    /// 20 拍/秒呈现节拍由 android_main 主循环控制（逻辑拍 frame_interval_ms）。
    /// **window=NULL ⇒ No-op 返回 false**（TERM_WINDOW 竞态挡——每帧前判非空）。
    pub fn present(&mut self, window: *mut ANativeWindow, argb: &[u32]) -> bool {
        if window.is_null() {
            return false;
        }
        #[cfg(target_os = "android")]
        unsafe {
            // 【复核A 红-1】此处**不得**再调几何协商——每帧 (240,320) 会覆盖
            // ensure_surface 的 (0,0) 原生比例协商 ⇒ 信箱不可达=Bug2 回归。
            // 协商唯一写点=ensure_surface（APP_CMD_INIT_WINDOW，(0,0)=只定格式）。
            let mut buf = ANativeWindowBuffer {
                width: 0,
                height: 0,
                stride: 0,
                format: 0,
                bits: std::ptr::null_mut(),
            };
            if ffi::ANativeWindow_lock(window, &mut buf, std::ptr::null_mut()) != 0 {
                return false;
            }
            let (bw, bh) = (buf.width.max(0) as u32, buf.height.max(0) as u32);
            if !buf.bits.is_null() && bw > 0 && bh > 0 {
                let stride = buf.stride.max(0) as u32;
                let total = (stride as usize) * (bh as usize) * 4;
                let dst = std::slice::from_raw_parts_mut(buf.bits as *mut u8, total);
                let (g, sc) = self.resolved();
                // 【字体修复】背板=物理像素（逻辑×sc）⇒ 呈现 1:1（矩形同逻辑×sc，钉见 native_parse）
                blit_argb_rgba(dst, bw, bh, stride, argb, g * 32 * sc, g * 32 * sc, 1);
            }
            ffi::ANativeWindow_unlockAndPost(window);
        }
        #[cfg(not(target_os = "android"))]
        {
            let _ = argb;
        }
        self.presented = true; // 【STAGE-B-006 · 复核 T6b-N3】首帧已呈现 ⇒ 输入放行
        true
    }
}

impl Default for ShellWiring {
    fn default() -> Self {
        Self::new()
    }
}

/// 【件2 触控区可视化】叠加框的态（框=当前态**生效**的命中面，单源）。
#[derive(Clone, Copy, Debug)]
pub enum OverlayState {
    /// 标题态：轮播三热区（TitleGeom 单源；pad 不接管）
    Title(game_core::title::TitleGeom),
    /// 其余 UI 态：虚拟键区
    Ui,
    /// 游戏态：虚拟键区 + UNDO + 地图点格区
    Game,
}

/// 【件2 触控区可视化】叠加框（debug 描边常驻，量产收审美）——**单源**：
/// 标题热区=TitleGeom（与 BootMachine::title_touch_key 同源）、虚拟键区=
/// wiring.pad（TouchMapper 活布局）、UNDO=undo_zone_hit 同源矩形。
/// > 【2026-10-26 更正】原"6 菜单行框"（2c 六行横条 + SOFTKEY_BAR_H 双写常量）
/// > 随 C 方案撤除；框按态只画该态生效的命中面。
pub fn overlay_rects(
    grid: u32,
    pad: &crate::input::PadLayout,
    state: OverlayState,
) -> Vec<(i32, i32, i32, i32, &'static str)> {
    let (sw, sh) = ((grid * 32) as i32, (grid * 32) as i32);
    let mut out = Vec::new();
    if let OverlayState::Title(geo) = state {
        let (a, b, c) = (geo.zone_prev, geo.zone_ok, geo.zone_next);
        out.push((a.0, a.1, a.2, a.3, "prev"));
        out.push((b.0, b.1, b.2, b.3, "ok"));
        out.push((c.0, c.1, c.2, c.3, "next"));
        return out;
    }
    if matches!(state, OverlayState::Game) {
        // UNDO 钮（shell/common/touch.rs undo_zone_hit 同源：**右上角** UNDO_ZONE_W×H）
        out.push((
            sw - game_shell_common::touch::UNDO_ZONE_W,
            0,
            game_shell_common::touch::UNDO_ZONE_W,
            game_shell_common::touch::UNDO_ZONE_H,
            "undo",
        ));
    }
    // 【红-1】dpad 框由 r_switch 派生：捕获圆 r=2*r_switch（in_dpad_capture）——
    // 外切方=2×捕获半径（**覆盖捕获圆全域**）；旧 60×60 硬编码=45px 出键在框外
    // =画的框骗人（复核B 红-1 实锤）。
    let half = 2 * pad.dpad.r_switch as i32; // =捕获半径 60
    out.push((
        pad.dpad.cx as i32 - half,
        pad.dpad.cy as i32 - half,
        half * 2,
        half * 2,
        "dpad",
    ));
    for z in &pad.soft {
        out.push((
            z.x0 as i32,
            z.y0 as i32,
            (z.x1 - z.x0) as i32,
            (z.y1 - z.y0) as i32,
            "soft",
        ));
    }
    // 【黄③】覆盖补齐：游戏态=地图点格区（tap→walk_to 响应面，全画布格域）+滑动区
    if matches!(state, OverlayState::Game) {
        out.push((0, 0, sw, sh, "map"));
    }
    out
}

/// 【问题2】debug 日志（logcat TAG=jjxy-input，debug 级保留——实机复核用）。
pub fn log_input(msg: &str) {
    #[cfg(target_os = "android")]
    unsafe {
        // NUL 结尾（Rust &str 非 NUL 串——%s 直读会拖尾垃圾）
        let c = std::ffi::CString::new(msg.replace('\0', " ")).unwrap_or_default();
        ffi::__android_log_print(
            3,
            b"jjxy-input\0".as_ptr() as *const _,
            b"%s\0".as_ptr() as *const _,
            c.as_ptr(),
        );
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = msg;
    }
}

// ───────────────────────────────────────── NDK FFI + android_main（仅 android 目标）

/// `ANativeWindow_Buffer`（android/native_window.h）。
#[repr(C)]
#[cfg(target_os = "android")]
pub struct ANativeWindowBuffer {
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub format: i32,
    pub bits: *mut std::ffi::c_void,
}

/// `ANativeActivity` 字段面前缀（android/native_activity.h，NDK 27 字段序：
/// callbacks/vm/env/clazz/internalDataPath/externalDataPath/sdkVersion/instance/
/// assetManager/obbPath——只声明到 assetManager，指针读前缀）。
#[repr(C)]
pub struct ANativeActivityFields {
    pub callbacks: *mut std::ffi::c_void,
    pub vm: *mut std::ffi::c_void,
    pub env: *mut std::ffi::c_void,
    pub clazz: *mut std::ffi::c_void,
    pub internal_data_path: *const std::os::raw::c_char,
    pub external_data_path: *const std::os::raw::c_char,
    pub sdk_version: i32,
    pub instance: *mut std::ffi::c_void,
    pub asset_manager: *mut std::ffi::c_void,
}

/// `ARect`（android/native_window.h）。
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ARect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// `android_app`（native_app_glue.h，NDK 27 字段序）——只声明到 destroyRequested
/// （glue 私有段不透明尾随：仅经指针读前缀字段，不分配本结构）。
#[repr(C)]
pub struct AndroidApp {
    pub user_data: *mut std::ffi::c_void,
    pub on_app_cmd: Option<extern "C" fn(*mut AndroidApp, i32)>,
    pub on_input_event: Option<extern "C" fn(*mut AndroidApp, *mut AInputEvent) -> i32>,
    pub activity: *mut std::ffi::c_void,
    pub config: *mut std::ffi::c_void,
    pub saved_state: *mut std::ffi::c_void,
    pub saved_state_size: usize,
    pub looper: *mut std::ffi::c_void,
    pub input_queue: *mut std::ffi::c_void,
    pub window: *mut ANativeWindow,
    pub content_rect: ARect,
    pub activity_state: i32,
    pub destroy_requested: i32,
}

/// `LOOPER_ID_INPUT`（native_app_glue.h enum：MAIN=1/INPUT=2/USER=3）。
#[cfg(target_os = "android")]
const LOOPER_ID_INPUT: i32 = 2;

/// `android_poll_source`（native_app_glue.h）。
#[repr(C)]
#[cfg(target_os = "android")]
pub struct AndroidPollSource {
    pub id: i32,
    pub app: *mut AndroidApp,
    pub process: Option<extern "C" fn(*mut AndroidApp, *mut AndroidPollSource)>,
}

/// 胶水入口（build.rs 编译期重命名自 native_app_glue.c 的
/// `ANativeActivity_onCreate`——静态库符号局部化规避）。
#[cfg(target_os = "android")]
extern "C" {
    fn glue_ANativeActivity_onCreate(
        activity: *mut std::ffi::c_void,
        saved_state: *const u8,
        saved_state_size: usize,
    );
}

/// NativeActivity 系统 dlsym 入口（真名导出：Rust `#[no_mangle]` 必进 dynsym）。
/// 与 [`android_main`] 构成标准入口对——**实机崩因
/// `undefined symbol: ANativeActivity_onCreate` 的根治点**：
/// ① build.rs 编译胶水并重命名其入口；② 本 trampoline 提供真名导出转发。
/// 【BUG-036】先登记实例（主线程，早于该实例的 android_main 线程与旧实例的销毁判定）。
#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn ANativeActivity_onCreate(
    activity: *mut std::ffi::c_void,
    saved_state: *const u8,
    saved_state_size: usize,
) {
    LIVE_INSTANCES.enter();
    glue_ANativeActivity_onCreate(activity, saved_state, saved_state_size)
}

/// 本进程存活的 Activity 实例数（BUG-036：旧实例销毁时仍有新实例 ⇒ 不结束进程）。
#[cfg(target_os = "android")]
static LIVE_INSTANCES: crate::process::InstanceCounter = crate::process::InstanceCounter::new();

#[cfg(target_os = "android")]
mod ffi {
    use super::{ANativeWindow, ANativeWindowBuffer, ARect, AInputEvent};

    extern "C" {
        pub fn AInputEvent_getType(event: *mut AInputEvent) -> i32;
        pub fn AMotionEvent_getAction(event: *mut AInputEvent) -> i32;
        pub fn AMotionEvent_getPointerId(event: *mut AInputEvent, pointer_index: usize) -> i32;
        pub fn AMotionEvent_getX(event: *mut AInputEvent, pointer_index: usize) -> f32;
        pub fn AMotionEvent_getY(event: *mut AInputEvent, pointer_index: usize) -> f32;
        pub fn AKeyEvent_getKeyCode(event: *mut AInputEvent) -> i32;
        pub fn AKeyEvent_getAction(event: *mut AInputEvent) -> i32;
        pub fn AKeyEvent_getRepeatCount(event: *mut AInputEvent) -> i32;
        pub fn ANativeWindow_lock(
            window: *mut ANativeWindow,
            out_buffer: *mut ANativeWindowBuffer,
            in_out_dirty_bounds: *mut ARect,
        ) -> i32;
        pub fn ANativeWindow_unlockAndPost(window: *mut ANativeWindow) -> i32;
        pub fn ANativeWindow_setBuffersGeometry(
            window: *mut ANativeWindow,
            width: i32,
            height: i32,
            format: i32,
        ) -> i32;
        pub fn __android_log_print(prio: i32, tag: *const std::os::raw::c_char, fmt: *const std::os::raw::c_char, ...) -> i32;
        pub fn AInputQueue_getEvent(queue: *mut std::ffi::c_void, out_event: *mut *mut AInputEvent) -> i32;
        pub fn AInputQueue_preDispatchEvent(queue: *mut std::ffi::c_void, event: *mut AInputEvent) -> i32;
        pub fn AInputQueue_finishEvent(queue: *mut std::ffi::c_void, event: *mut AInputEvent, handled: i32) -> i32;
        pub fn ANativeWindow_getWidth(window: *mut ANativeWindow) -> i32;
        pub fn ANativeWindow_getHeight(window: *mut ANativeWindow) -> i32;
        /// android/configuration.h：`int32_t AConfiguration_getDensity(AConfiguration*)`
        /// （ACONFIGURATION_DENSITY_*：dpi 值；0/0xfffe/0xffff = 未知码）。
        pub fn AConfiguration_getDensity(config: *mut std::ffi::c_void) -> i32;
        pub fn ALooper_pollAll(
            timeout_millis: i32,
            out_fd: *mut i32,
            out_events: *mut i32,
            out_data: *mut *mut std::ffi::c_void,
        ) -> i32;
    }
}

/// 主机装配态（android_main 主循环持有：平台件 + 运行时 + 画布）。
#[cfg(target_os = "android")]
pub struct HostState {
    pub wiring: ShellWiring,
    /// 【进游戏】双机运行时（Boot/GameScene 调度 + 交接 + 存档写穿；宿主流程钉同一体）
    pub rt: crate::runtime::Runtime,
    pub canvas: game_core::render::ArgbImage,
    pub assets: crate::asset::apk::ApkAssetSource,
    pub surface: (u32, u32),
    pub pending_keys: Vec<KeyAction>,
    /// 【竖屏】控制面板画布（面板物理像素；状态变化才重绘）+ 上次绘制状态。
    pub panel: game_core::render::ArgbImage,
    pub panel_state: Option<crate::portrait::PanelState>,
    /// 【竖屏】软键图标条（ui[11]，面板软键图标=游戏同款）。
    pub softkey_icons: game_core::render::ArgbImage,
    /// 【按键反馈】触感 + 按键音出口（JNI 工作线程；None = 未起）。
    pub feedback: Option<crate::feedback::android::JniFeedback>,
    /// 【STAGE-B-005】游戏内退出确认 = Android 标准对话框出口（JNI 工作线程；None = 未起）。
    pub exit_dialog: Option<crate::exit_dialog::android::ExitDialog>,
}

#[cfg(target_os = "android")]
impl HostState {
    /// 【竖屏】采用规划（INIT_WINDOW / contentRect 变化重排共用）：双档布局换轨 +
    /// 活动档画布与面板按物理尺寸重建（面板强制重绘）。
    fn adopt_portrait(&mut self, plan: &crate::portrait::PortraitPlan) {
        // 【复核 T5-R1】场景布局按操控方式选档（触屏=scene_touch 的 Layout；交接 apply_layouts 同源）
        let scene_layout = if self.rt.rt_control_mode == game_shell_common::settings::ControlMode::Touch {
            plan.scene_touch.layout()
        } else {
            plan.scene.layout()
        };
        self.rt.apply_layouts(plan.boot.layout(), scene_layout);
        let (cw, ch) = self.rt.active_canvas(plan).physical();
        self.canvas = game_core::render::ArgbImage::create(cw, ch);
        // 面板画布：Touch=工具栏条尺寸（绘制走 toolbar 臂）；Pad=摇杆面板
        if self.rt.rt_control_mode == game_shell_common::settings::ControlMode::Touch {
            self.panel = game_core::render::ArgbImage::create(plan.toolbar.bar.w, plan.toolbar.bar.h);
        } else {
            self.panel = game_core::render::ArgbImage::create(plan.panel.w, plan.panel.h);
        }
        self.panel_state = None;
        log_input(&format!(
            "portrait plan window={:?} dp={} panel_min={} scene={:?} boot={:?} game={:?} panel={:?} geom={:?}",
            plan.window, plan.dp, plan.panel_min, plan.scene, plan.boot, plan.game, plan.panel, plan.geom
        ));
    }
}

/// 生产 looper（[`crate::pump::Pump`] 的 ALooper 实现）：poll(timeout) 就绪源经
/// glue 的 `process`（process_input 内部循环 getEvent→preDispatch→onInputEvent→
/// finishEvent 排空整队，native_app_glue.c:181-192；process_cmd 处理 APP_CMD_*）。
#[cfg(target_os = "android")]
struct AndroidLooper {
    app: *mut AndroidApp,
    t0: std::time::Instant,
}

#[cfg(target_os = "android")]
impl crate::pump::Pump for AndroidLooper {
    fn now_ms(&self) -> u64 {
        self.t0.elapsed().as_millis() as u64
    }

    fn poll(&mut self, timeout_ms: i32) -> bool {
        unsafe {
            let mut events = 0i32;
            let mut data: *mut std::ffi::c_void = std::ptr::null_mut();
            let ident = ffi::ALooper_pollAll(timeout_ms, std::ptr::null_mut(), &mut events, &mut data);
            if ident < 0 {
                return false; // TIMEOUT/WAKE/ERROR：无就绪源
            }
            if !data.is_null() {
                let src = data as *mut AndroidPollSource;
                if let Some(process) = (*src).process {
                    process(self.app, src);
                }
            }
            (*self.app).destroy_requested == 0
        }
    }
}

#[cfg(target_os = "android")]
extern "C" fn handle_cmd(app: *mut AndroidApp, cmd: i32) {
    unsafe {
        let st = &mut *((*app).user_data as *mut HostState);
        match cmd {
            APP_CMD_INIT_WINDOW => {
                // 【实机 SIGSEGV 回派修复①】窗口就绪才协商（几何协商唯一写点）
                let ok = st.wiring.ensure_surface((*app).window);
                st.surface = st.wiring.surface;
                if let Some(plan) = st.wiring.portrait.clone() {
                    // 【竖屏动态分辨率】双档布局：开场链 boot 档 / 游戏 scene 档（同一视口）
                    st.adopt_portrait(&plan);
                } else {
                    // 【#2】策略选定分辨率 → 画布/Modern 布局重解析（任意格数×32 公式）
                    let (g, sc) = st.wiring.resolved();
                    // 【字体修复】物理背板（逻辑×sc；字形按物理光栅化）
                    st.canvas = game_core::render::ArgbImage::create((g * 32 * sc) as i32, (g * 32 * sc) as i32);
                    st.rt.apply_layout(game_core::layout::Layout::modern(g as i32, g as i32));
                }
                let _ = ok;
            }
            APP_CMD_TERM_WINDOW => {
                // glue 随后置 app->window=NULL（native_app_glue.c:145）——呈现侧每帧判空
            }
            APP_CMD_CONTENT_RECT_CHANGED => {
                // 【竖屏公式化】内容区（系统栏/手势条让位）到达：内缩入规划输入，结果变才重排
                let r = (*app).content_rect;
                st.wiring.content_rect = Some((r.left, r.top, r.right, r.bottom));
                log_input(&format!("content rect l={} t={} r={} b={}", r.left, r.top, r.right, r.bottom));
                if let Some(plan) = st.wiring.replan_portrait() {
                    st.adopt_portrait(&plan);
                }
            }
            _ => {}
        }
        let in_pip = false; // PiP 判定（isInPictureInPictureMode）走 JNI；默认非 PiP
        if let Some(n) = st.wiring.on_lifecycle(cmd, in_pip) {
            // 【进游戏】hideNotify 边沿交游戏机（win 壳 hide_notify 同构）：游戏态=
            // GameScene::hide_notify（mode3→4 弹菜单 + closeAudioPlayer + anyKeyPlayed）；
            // Boot 态=bgm.on_hide_notify。showNotify 只清 latch（无游戏态副作用）。
            if n == Notify::Hide {
                st.rt.hide_notify();
                log_input("lifecycle hideNotify");
            }
        }
    }
}

#[cfg(target_os = "android")]
extern "C" fn handle_input(app: *mut AndroidApp, event: *mut AInputEvent) -> i32 {
    unsafe {
        let st = &mut *((*app).user_data as *mut HostState);
        log_input(&format!("input-arrival type={}", ffi::AInputEvent_getType(event)));
        let (mut keys, consumed) = st.wiring.on_input_ex(event);
        // 【计划 3 T4】视口触点（Game 区逻辑坐标）搬运进 Runtime（step_scene 分类：tap/swipe/drag）
        st.rt.inject_vp_touches(std::mem::take(&mut st.wiring.pending_vp_touches));
        // 【BUG-034 · 壳层输入映射】根界面（开机 logo / 主菜单 / 退场动画）BACK（返回键/边缘返回手势）=
        // 立即退出（runtime.rs back_requests_exit：置退出请求，主循环下一轮 finish）；丢弃该 BACK 的 -7。
        // 【STAGE-B-005】地图行走态 BACK = Android 标准退出确认框（runtime.rs back_requests_exit_dialog 置未决 ⇒
        // 暂停；exit_dialog.rs 弹框）；丢弃该 BACK 的 -7。辅助类未就绪 ⇒ 照旧 -7（开游戏内菜单）。
        // 其余态保持右软键 -7 原映射（子页返回 / 序章跳过 / 游戏内菜单 / 浮层关闭）。
        const AINPUT_EVENT_TYPE_KEY: i32 = 1;
        let back_down = ffi::AInputEvent_getType(event) == AINPUT_EVENT_TYPE_KEY
            && ffi::AKeyEvent_getKeyCode(event) == AKEYCODE_BACK
            && ffi::AKeyEvent_getAction(event) == AKEY_EVENT_ACTION_DOWN
            && ffi::AKeyEvent_getRepeatCount(event) == 0;
        if back_down {
            let dialog_ready = st.exit_dialog.as_ref().is_some_and(|d| d.available());
            if st.rt.back_requests_exit() {
                log_input(&format!("back at root mode={:?} → 立即退出", st.rt.machine.mode));
                keys.clear();
            } else if dialog_ready && st.rt.back_requests_exit_dialog() {
                if let Some(d) = st.exit_dialog.as_ref() {
                    d.show();
                }
                log_input("back in game → 系统退出确认框");
                keys.clear();
            }
        }
        st.pending_keys.extend(keys);
        // 【按键反馈】虚拟键按下 ⇒ 输入回调内立即派发（不等 75ms 逻辑拍）；触感 / 按键音各看设置页"按键震动 / 按键音"
        //（STAGE-B-006：与"声音"互不影响；读活动机快照）
        let presses = st.wiring.take_key_presses();
        if let Some(fb) = st.feedback.as_mut() {
            let (haptic, click) = st.rt.key_feedback();
            crate::feedback::emit(fb, presses, haptic, click);
        }
        if consumed {
            1
        } else {
            0
        }
    }
}

/// native_app_glue 入口（`ANativeActivity_onCreate` 起线程调用）：
/// 装配四 trait（ApkAssetSource/AAudioPcm/AndroidStorage/触屏 T3 件）+ 事件环
/// （poll→APP_CMD_*/AInputEvent→ShellWiring）+ 20 拍/秒呈现（逻辑拍
/// frame_interval_ms 驱动 BootMachine::tick/paint）。
#[no_mangle]
pub extern "C" fn android_main(app: *mut AndroidApp) {
    #[cfg(target_os = "android")]
    unsafe {
        // 【哨兵1】入口哨兵（裸格式串零可变参数——兼验日志机制本身）
        ffi::__android_log_print(
            3,
            b"jjxy-input\0".as_ptr() as *const _,
            b"boot android_main entered\0".as_ptr() as *const _,
        );
        use crate::asset::apk::ApkAssetSource;
        use crate::storage::AndroidStorage;
        use game_platform::{AssetSource, AudioOut};

        // 【2026-10-26 字体修复】D2 字形缝：系统 CJK 字体（NotoSansCJK SC 面）注入
        // game-core 字形钩子——旧未注入 ⇒ 全部文字=差分占位哈希块（读档"???"/序章乱块）。
        // 共享实现 game_shell_common::font（win 壳同源）；无字体=占位真回退。
        let t_font = std::time::Instant::now();
        let chosen = game_shell_common::font::install(
            &game_shell_common::font::android_candidates(),
            &mut |m| log_input(m),
        );
        log_input(&format!("font install {:?} in {}ms", chosen, t_font.elapsed().as_millis()));

        /// 无声降级设备（AAudio 开失败/API<26）：吞写不阻塞（loop 自带节拍）。
        struct SilentOut;
        impl AudioOut for SilentOut {
            fn write(&mut self, _frames: &[i16]) -> Result<(), String> {
                Ok(())
            }
            fn idle(&mut self) {}
        }
        use game_core::render::ArgbImage;
        use std::ffi::CStr;

        let activity = (*app).activity as *mut ANativeActivityFields;
        let asset_mgr = (*activity).asset_manager as *mut crate::asset::apk::AAssetManager;
        let data_path = if (*activity).internal_data_path.is_null() {
            std::path::PathBuf::from(".")
        } else {
            std::path::PathBuf::from(
                CStr::from_ptr((*activity).internal_data_path).to_string_lossy().into_owned(),
            )
        };
        let assets = ApkAssetSource::new(asset_mgr);
        let (storage, stores) = AndroidStorage::open(&data_path);
        log_input(&format!("storage {} stores={:?}", data_path.display(), stores.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>()));

        // BootMachine 装配 + 双机运行时（runtime.rs：宿主流程钉同一调度体）
        let machine = crate::runtime::build_boot_machine(
            &assets,
            game_core::layout::Layout::modern(GRID_N, GRID_N),
        );
        let mut rt = crate::runtime::Runtime::new(machine, stores, Box::new(storage));
        // 【STAGE-B-006】设置文件（声音 / 开屏画面 / 按键震动 / 按键音）：首拍前装载并应用——开屏关 ⇒ 首拍前即主菜单；
        // 之后设置页切换由 Runtime 每拍末同拍落盘（与存档同目录，FileRms 只认 *.bin 记录名、不误读）
        rt.open_app_settings(data_path.join("app_settings.cfg"));
        // 【竖屏】面板软键图标=游戏同款 ui[11]（与画布角标同义，随态切换）
        let softkey_icons = game_shell_common::handoff::load_container(&assets, "ui", 25)[11].clone();

        // 【音频断层修】4 轨先载入 TrackStore（旧 store=空 Vec=Mixer 无轨可混=静音）
        // ——Android 轨道 AssetSource::read("track{i}.ogg")（shell/common audio.rs 口径）
        let mut tracks: Vec<Option<std::sync::Arc<game_shell_common::audio::Track>>> =
            (0..4).map(|_| None).collect();
        for i in 0..4usize {
            match assets.read(&format!("track{i}.ogg")) {
                Ok(bytes) => match game_shell_common::audio::decode_ogg(&bytes) {
                    Ok(t) => {
                        log_input(&format!("track{i} loaded ok"));
                        tracks[i] = Some(std::sync::Arc::new(t));
                    }
                    Err(e) => log_input(&format!("track{i} decode err {e}")),
                },
                Err(e) => log_input(&format!("track{i} read err {e}")),
            }
        }
        let mut st = Box::new(HostState {
            wiring: ShellWiring::new(),
            rt,
            canvas: ArgbImage::create(GRID_N * 32 * SCALE, GRID_N * 32 * SCALE),
            assets,
            surface: (CANVAS_W, CANVAS_H),
            pending_keys: Vec::new(),
            panel: ArgbImage::create(1, 1),
            panel_state: None,
            softkey_icons,
            feedback: None,
            exit_dialog: None,
        });
        // 【STAGE-B-006 · 复核 T3-F2】开屏关时首拍前已在主菜单：标题态路由须先于首批触点生效（否则初始化期间的点按按
        // 非标题态走点空白 = 代按确认，直接开始新游戏）；之后照旧每拍刷新
        st.wiring.title_mode = st.rt.title_mode();
        // 【按键反馈】JNI 工作线程（触感 + 按键音；attach 一次、缓存方法 ID，输入回调零阻塞）
        st.feedback = Some(crate::feedback::android::JniFeedback::spawn((*activity).vm, (*activity).clazz, log_input));
        // 【STAGE-B-005】退出确认对话框 JNI 工作线程（Java 辅助类经 activity 类加载器解析；主循环零 JNI）
        st.exit_dialog = Some(crate::exit_dialog::android::ExitDialog::spawn((*activity).vm, (*activity).clazz, log_input));
        // 【竖屏公式化】规划输入的设备/数据源：密度（glue 已自 assetManager 填 config）+
        // 全关卡最大格数（maplv 头）——INIT_WINDOW 时与窗口尺寸一并入公式
        if !(*app).config.is_null() {
            st.wiring.dpi = crate::portrait::dpi_or_baseline(ffi::AConfiguration_getDensity((*app).config));
        }
        st.wiring.map_extent = crate::portrait::MapExtent::from_assets(&st.assets);
        log_input(&format!("portrait inputs dpi={} map={:?}", st.wiring.dpi, st.wiring.map_extent));
        (*app).user_data = &mut *st as *mut HostState as *mut std::ffi::c_void;
        (*app).on_app_cmd = Some(handle_cmd);
        (*app).on_input_event = Some(handle_input);

        // AAudio 音频环（阻塞 write=节拍；开失败=NullSink 无声降级，API<26 同）
        let (tx, rx) = std::sync::mpsc::channel::<game_shell_common::audio::Msg>();
        std::thread::spawn(move || {
            log_input("aaudio thread spawn");
            /// 设备写错误进 logcat（audio_loop 的 eprintln 在 NativeActivity 下无处可去）。
            struct LoggedOut(Box<dyn AudioOut>);
            impl AudioOut for LoggedOut {
                fn write(&mut self, frames: &[i16]) -> Result<(), String> {
                    self.0.write(frames).map_err(|e| {
                        log_input(&format!("aaudio write err {e}"));
                        e
                    })
                }
                fn idle(&mut self) {
                    self.0.idle()
                }
            }
            let dev: Box<dyn AudioOut> = match crate::audio::aaudio::AAudioPcm::open() {
                Ok(pcm) => {
                    log_input(&format!("aaudio open ret=ok {}", pcm.describe()));
                    Box::new(LoggedOut(Box::new(pcm)))
                }
                Err(e) => {
                    log_input(&format!("aaudio open ret=err {e}"));
                    Box::new(SilentOut)
                }
            };
            let store: game_shell_common::audio::TrackStore =
                std::sync::Arc::new(std::sync::Mutex::new(tracks));
            game_shell_common::audio::audio_loop(rx, dev, game_shell_common::audio::Mixer::new(store));
        });
        // BGM 命令出口：Runtime 每拍末 + hideNotify 后经 shell-common BgmController 翻译
        // （runtime.rs sync_bgm，win 壳同源；宿主钉 tests/bgm_sync.rs）——只在变化时发、日志随
        // 命令。【2026-10-26】旧手写翻译只认轨道：None 不发 Stop、serial 变不重发、gain 恒 1.0、
        // last_bgm 永不复位 ⇒ `bgm req=None` 每拍刷屏。
        struct AudioThreadSink(std::sync::mpsc::Sender<game_shell_common::audio::Msg>);
        impl game_shell_common::audio::AudioSink for AudioThreadSink {
            fn send(&mut self, cmd: game_shell_common::audio::Cmd) {
                log_input(&format!("bgm {cmd:?}"));
                let _ = self.0.send(game_shell_common::audio::Msg::Cmd(cmd));
            }
        }
        st.rt.set_bgm_sink(Box::new(AudioThreadSink(tx)));

        // 【2026-10-26 实机根因②】主循环=pump 调度（shell/android/src/pump.rs 宿主钉）：
        // 非阻塞排空事件 → 到期出拍（tick→paint→present **每拍一次**：BootMachine::paint
        // 推进粒子/图标带动画，旧"每轮都 paint"使动画速度随事件频率漂移）→ 阻塞等到
        // 下一拍（触点到达提前醒）。旧 `while pollAll(16)>=0` 在 Move 流下饿死 tick。
        let mut looper = AndroidLooper { app, t0: std::time::Instant::now() };
        let mut pacer = crate::pump::FramePacer::new(0);
        // 【BUG-033】退出链执行半（win 壳 consume_exit 同口径）：BootMachine::finished ⇒
        // ProcessControl::request_exit = ANativeActivity_finish（任意线程可调，NDK 投递主线程 finish）；
        // Activity 销毁（APP_CMD_DESTROY ⇒ destroy_requested）后循环退出，再结束进程
        // （= MIDlet notifyDestroyed；亦免 NativeActivity 同进程二次进入时静态状态残留）。
        let mut process = crate::process::android::AndroidProcessControl::new(
            (*app).activity as *mut crate::process::android::ANativeActivity,
        );
        let mut exit_sent = false;
        while (*app).destroy_requested == 0 {
            // 逻辑拍间隔：Boot=frame_interval_ms（mode 0=100 a.java:3350，其余 75
            // a.java:3377）、场景恒 75——win 壳 frame_interval() 同源
            let interval = st.rt.frame_interval_ms();
            if !crate::pump::pump_once(&mut looper, &mut pacer, interval) {
                let wait = pacer.wait_ms(crate::pump::Pump::now_ms(&looper));
                crate::pump::Pump::poll(&mut looper, wait);
                continue;
            }
            if (*app).destroy_requested != 0 {
                break;
            }
            {
                // 【进游戏】双机运行时一拍（runtime.rs；宿主流程钉 tests/flow.rs 同一体）：
                // GameScene 接管后走场景拍（键重放/点按/存档写穿/kind7 换回标题），否则
                // 走 Boot 拍（手势/点按解析 → 锁存键 → tick → 读条完交接 GameScene）。
                // 【BUG-034】退出请求已成立（返回键立即退出 / 退场动画走完）⇒ 不再出拍：免 logo 期
                // 返回后再走拍进主菜单起 BGM；finish 到 Activity 销毁之间画面停在末帧。
                // 【STAGE-B-005】系统退出确认框结果回灌（退出 ⇒ 进程退出请求；取消 ⇒ 结束暂停）
                if let Some(r) = st.exit_dialog.as_ref().and_then(|d| d.try_result()) {
                    log_input(&format!("exit dialog result {r:?}"));
                    st.rt.on_exit_dialog_result(r == crate::exit_dialog::DialogResult::Exit);
                }
                if !exit_sent && !st.rt.exit_requested() {
                    // 【计划 3 T5 · 复核 T5-R1】操控方式轮询：rt 侧翻转（设置页翻行→sync）→ wiring 重排+adopt
                    // （触屏档画布/Layout/工具栏生效；apply_portrait 重建 TouchMapper=按住态清零）
                    st.wiring.toolbar_side = st.rt.toolbar_side_left.or(st.rt.toolbar_side_right);
                    // 【计划 3 T5 · 复核 T5-R2】工具栏命中消费：perform_action（Mid 占位码 → 查询校验；
                    // Boot 面 = machine.key_value 直置）
                    for a in std::mem::take(&mut st.wiring.pending_toolbar_actions) {
                        let mid_n = st.rt.toolbar_mid.len();
                        let resolved: Option<game_core::controls::Action> = match a {
                            game_core::controls::Action::Key(code) if (-code) >= 101 => {
                                st.rt.toolbar_mid.get((-code - 101) as usize).map(|b| b.action)
                            }
                            other => Some(other),
                        };
                        let _ = mid_n;
                        if let Some(act) = resolved {
                            match st.rt.scene.as_mut() {
                                Some(sc) => {
                                    let _ = sc.perform_action(act);
                                }
                                None => {
                                    if let game_core::controls::Action::Key(k) = act {
                                        st.rt.machine.key_value = k;
                                    }
                                }
                            }
                        }
                    }
                    if st.rt.rt_control_mode != st.wiring.control_mode {
                        if let Some(plan) = st.wiring.set_control_mode(st.rt.rt_control_mode) {
                            st.adopt_portrait(&plan);
                        }
                    }
                    // 【STAGE-B-006 · 复核 T6-F3】首帧呈现前的输入丢弃（take_step_inputs）
                    let (keys, gestures, taps) = st.wiring.take_step_inputs(&mut st.pending_keys);
                    st.rt.step(&st.assets, &keys, gestures, taps);
                    // 标题态随拍更新（下一批触点的路由依据；场景活动时恒非标题）
                    st.wiring.title_mode = st.rt.title_mode();
                    // BGM：step 末已同步（Runtime::sync_bgm → AudioThreadSink）
                }
                if !exit_sent && st.rt.exit_requested() {
                    exit_sent = true;
                    log_input("exit chain: exit requested → ANativeActivity_finish");
                    game_platform::ProcessControl::request_exit(&mut process);
                }
            }
            // 呈现（每拍一次，tick 后）——【回派修复②】每帧前判 window 非空
            // （TERM_WINDOW→window=NULL 竞态挡；未就绪期跳过渲染呈现）
            // 【BUG-035】退出已发（finish 至销毁约 1 s）⇒ 不再呈现：窗口保留最后一帧（主菜单）随关闭动画收起，
            // 免画出 mode 22 首帧（白底 logo 层）闪一下。
            if !(*app).window.is_null() && !exit_sent {
                if let Some(plan) = st.wiring.portrait.clone() {
                    // 【竖屏动态分辨率】活动机画布档（开场链 2× / 游戏 3×，同一物理视口）
                    let canvas = st.rt.active_canvas(&plan);
                    st.wiring.set_active_canvas(canvas);
                    let (cw, ch) = canvas.physical();
                    if (st.canvas.width, st.canvas.height) != (cw, ch) {
                        st.canvas = ArgbImage::create(cw, ch);
                    }
                    {
                        let mut g = canvas_graphics(&mut st.canvas, canvas.scale as u32);
                        st.rt.paint(&mut g);
                    }
                    // 面板：按住键 + 软键种类变化才重绘；Touch 模式=工具栏臂（controls 查询+controls_ui 绘制）
                    if st.rt.rt_control_mode == game_shell_common::settings::ControlMode::Touch {
                        if let Some(sc) = st.rt.scene.as_ref() {
                            let c = sc.controls();
                            crate::portrait::paint_toolbar(
                                &mut st.panel,
                                &plan.toolbar,
                                &[
                                    (c.left.map(|b| (b.function, b.action)), st.wiring.touch.held_codes().contains(&-6)),
                                    (c.right.map(|b| (b.function, b.action)), st.wiring.touch.held_codes().contains(&-7)),
                                ],
                                &c.mid.iter().map(|b| (b.function, b.action)).collect::<Vec<_>>(),
                            );
                        }
                    } else {
                    let ps = crate::portrait::PanelState {
                        held: st.wiring.touch.held_codes(),
                        softkeys: st.rt.softkeys(),
                        // 【计划 3 T6】软键标注 = controls() 左右钮功能名（两呈现同源；None=隐藏）
                        labels: (
                            st.rt.toolbar_side_left.map(|(f, _)| game_shell_common::controls_ui::label(f).to_string()),
                            st.rt.toolbar_side_right.map(|(f, _)| game_shell_common::controls_ui::label(f).to_string()),
                        ),
                    };
                    if st.panel_state.as_ref() != Some(&ps) {
                        crate::portrait::paint_panel(&mut st.panel, &plan.geom, &ps, &st.softkey_icons);
                        st.panel_state = Some(ps);
                    }
                    }
                    let cr = canvas.rect_in(plan.game);
                    let layers: [(&[u32], i32, i32, i32, i32); 2] = [
                        (&st.canvas.argb, st.canvas.width, st.canvas.height, cr.x, cr.y),
                        (&st.panel.argb, st.panel.width, st.panel.height, plan.panel.x, plan.panel.y),
                    ];
                    st.wiring.present_layers((*app).window, &layers);
                } else {
                    let (g_, sc_) = st.wiring.resolved();
                    {
                        let mut g = canvas_graphics(&mut st.canvas, sc_);
                        st.rt.paint(&mut g);
                        // 【件2+黄④】触控区叠加（按态=该态生效命中面，同源；debug 开关可关）
                        if st.wiring.overlay_debug {
                            let state = st.rt.overlay_state();
                            let rects = overlay_rects(g_, &st.wiring.pad, state);
                            g.set_color(0xFFFF00FFu32);
                            for (x, y, w, h, _tag) in rects {
                                g.draw_rect(x, y, w, h);
                            }
                        }
                    }
                    let argb = st.canvas.argb.clone();
                    st.wiring.present((*app).window, &argb);
                }
            }
        }
        (*app).user_data = std::ptr::null_mut();
        drop(st);
        // 【BUG-036】注销本实例；仅当本实例发起过退出 ∧ 进程内已无其他实例才结束进程——用户退出后立即再点
        // 图标时，新实例已进本进程（同 pid 第二个 android_main），此时 `_exit` 会连新实例一起杀掉。
        let remaining = LIVE_INSTANCES.leave();
        if crate::process::end_process_after_destroy(exit_sent, remaining) {
            log_input("exit chain: activity destroyed → process exit");
            // `_exit` 而非 libc `exit`：此刻 Java 主线程阻塞在 onDestroy（glue android_app_free 等本线程
            // 置 destroyed）、binder/AAudio 线程仍活——`exit` 的 __cxa_finalize 会析构各共享库静态对象，
            // 与这些线程并发竞态；`_exit` 不跑 atexit/析构。无待刷数据：存档同步写穿
            // （FileRms::persist = std::fs::write，无用户态缓冲），logcat 写为同步。
            extern "C" {
                fn _exit(status: i32) -> !;
            }
            _exit(0);
        } else if exit_sent {
            log_input(&format!(
                "exit chain: activity destroyed, {remaining} other instance(s) alive → keep process"
            ));
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
    }
}

/// rel 校验再导出（装配面统一入口）。
pub use crate::asset::validate_rel as rel_contract;

/// 骨架自检（宿主可跑：纯装配态无原生调用）。
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiring_defaults_match_shell_contract() {
        let w = ShellWiring::new();
        assert!(w.visibility.shown());
        assert_eq!(w.frame_interval_ms(), 75);
        assert!(rel_contract("mapbg").is_ok());
    }

    #[test]
    fn wiring_assembly_touch_surface_ready() {
        // T5 装配自检：触屏件（TouchMapper/GestureTracker）+ 呈现面 240×320 就位
        let mut w = ShellWiring::new();
        assert_eq!(w.surface, logical_canvas(), "Modern 方画布（GRID_N×32）");
        assert_eq!(logical_canvas(), (576, 576), "默认 18 格=576 逻辑边长");
        assert_eq!(game_pixels(), (1152, 1152), "默认 @2x=1152 呈现边长");
        assert_eq!(w.pad.dpad.cx, 56.0, "虚拟方向盘几何（T3 PadLayout）");
        // 硬件键翻译直通（-1..-7 契约面）：BACK 恒吞（BUG-020）
        let (keys, consumed) = w.on_hw_key(AKEYCODE_BACK, 0, 0);
        assert_eq!(keys, vec![KeyAction::Press(-7)]);
        assert!(consumed);
        // 触点→pad 键（软键区 -6）：TouchMapper 接线生效
        let ev = TouchEvent { id: 0, phase: crate::input::TouchPhase::Down, x: 130.0, y: 230.0 };
        assert_eq!(w.on_touch(&ev), vec![KeyAction::Press(crate::input::KEY_LEFT_SOFT)]);
    }

    #[test]
    fn lifecycle_dispatch_through_wiring() {
        let mut w = ShellWiring::new();
        assert_eq!(w.on_lifecycle(APP_CMD_PAUSE, false), Some(Notify::Hide));
        assert_eq!(w.on_lifecycle(APP_CMD_RESUME, false), Some(Notify::Show));
    }
}
