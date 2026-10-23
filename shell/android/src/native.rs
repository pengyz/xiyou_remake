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

/// 呈现 blit：画布 ARGB（0xAARRGGBB）→ 窗口 RGBA8888（字节序 [R,G,B,A]）等比信箱。
/// 整数倍最近邻放大（contain）+ 居中；信箱区清零。返回 blit 矩形 (x,y,w,h)。
pub fn blit_argb_rgba(
    dst: &mut [u8],
    dst_w: u32,
    dst_h: u32,
    argb: &[u32],
    canvas_w: u32,
    canvas_h: u32,
) -> (u32, u32, u32, u32) {
    let (cw, ch) = (canvas_w.max(1), canvas_h.max(1));
    let (dw, dh) = (dst_w, dst_h);
    for b in dst.iter_mut() {
        *b = 0; // 信箱清零
    }
    let scale = ((dw / cw).min(dh / ch)).max(1);
    let (w, h) = (cw * scale, ch * scale);
    let (x0, y0) = ((dw.saturating_sub(w)) / 2, (dh.saturating_sub(h)) / 2);
    for y in 0..h.min(dh.saturating_sub(y0)) {
        for x in 0..w.min(dw.saturating_sub(x0)) {
            let src = ((y / scale) * cw + (x / scale)) as usize;
            let Some(px) = argb.get(src) else { continue };
            let a = (*px >> 24) & 0xff;
            let r = (*px >> 16) & 0xff;
            let g = (*px >> 8) & 0xff;
            let b = *px & 0xff;
            let off = (((y0 + y) as u32 * dw + (x0 + x) as u32) * 4) as usize;
            if off + 4 <= dst.len() {
                dst[off] = r as u8;
                dst[off + 1] = g as u8;
                dst[off + 2] = b as u8;
                dst[off + 3] = a as u8;
            }
        }
    }
    (x0, y0, w.min(dw), h.min(dh))
}

/// 触点窗口坐标→逻辑画布 240×320（等比信箱反变换：减信箱偏移、除缩放）。
pub fn map_touch_xy(x: f32, y: f32, surf_w: u32, surf_h: u32) -> (f32, f32) {
    let (cw, ch) = (CANVAS_W.max(1), CANVAS_H.max(1));
    let scale = ((surf_w / cw).min(surf_h / ch)).max(1) as f32;
    let (w, h) = (cw as f32 * scale, ch as f32 * scale);
    let (x0, y0) = ((surf_w as f32 - w) / 2.0, (surf_h as f32 - h) / 2.0);
    (((x - x0) / scale), ((y - y0) / scale))
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
    /// 呈现表面尺寸（INIT_WINDOW 更新；触点信箱反变换用）
    pub surface: (u32, u32),
}

impl ShellWiring {
    pub fn new() -> ShellWiring {
        let pad = PadLayout::default_240x320();
        ShellWiring {
            visibility: Visibility::new(),
            touch: TouchMapper::new(pad.clone()),
            pad,
            gesture: game_shell_common::touch::GestureTracker::new(24),
            surface: (CANVAS_W, CANVAS_H),
        }
    }

    /// 逻辑拍节奏（毫秒）：mode 0 = 100（a.java:3350）、其余 = 75（a.java:3377）——
    /// shell/win `frame_interval()` 同构（main.rs:645-650）。
    pub fn frame_interval_ms(&self) -> u64 {
        75
    }

    /// 触点事件翻译（宿主可测面）：pad 区→键动作流 + 手势簿登记。
    pub fn on_touch(&mut self, ev: &TouchEvent) -> Vec<KeyAction> {
        self.gesture.down(ev.x as i32, ev.y as i32);
        self.touch.feed(ev)
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
                    let (x, y) = map_touch_xy(
                        ffi::AMotionEvent_getX(event, idx),
                        ffi::AMotionEvent_getY(event, idx),
                        self.surface.0,
                        self.surface.1,
                    );
                    let ev = TouchEvent { id, phase, x, y };
                    (self.on_touch(&ev), true) // 触点恒吞（触摸屏无系统默认）
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

    /// 呈现：ArgbImage → ANativeWindow 缓冲（RGBA8888 格式协商 + 等比信箱 blit，
    /// SoftGraphics ARGB 直拷——setBuffersGeometry(240×320) 后 1:1 直拷路径）。
    /// 20 拍/秒呈现节拍由 android_main 主循环控制（逻辑拍 frame_interval_ms）。
    pub fn present(&mut self, window: *mut ANativeWindow, argb: &[u32]) {
        #[cfg(target_os = "android")]
        unsafe {
            const WINDOW_FORMAT_RGBA_8888: i32 = 1;
            // 格式协商：画布 240×320 RGBA8888（协商失败=按窗口原尺寸信箱 blit）
            ffi::ANativeWindow_setBuffersGeometry(
                window,
                CANVAS_W as i32,
                CANVAS_H as i32,
                WINDOW_FORMAT_RGBA_8888,
            );
            let mut buf = ANativeWindowBuffer {
                width: 0,
                height: 0,
                stride: 0,
                format: 0,
                bits: std::ptr::null_mut(),
            };
            if ffi::ANativeWindow_lock(window, &mut buf, std::ptr::null_mut()) != 0 {
                return;
            }
            let (bw, bh) = (buf.width.max(0) as u32, buf.height.max(0) as u32);
            if !buf.bits.is_null() && bw > 0 && bh > 0 {
                let stride = buf.stride.max(0) as u32;
                let total = (stride as usize) * (bh as usize) * 4;
                let dst = std::slice::from_raw_parts_mut(buf.bits as *mut u8, total);
                let (_, _, _, _) = blit_argb_rgba(dst, stride, bh, argb, CANVAS_W, CANVAS_H);
            }
            ffi::ANativeWindow_unlockAndPost(window);
        }
        #[cfg(not(target_os = "android"))]
        {
            let _ = (window, argb);
        }
    }
}

impl Default for ShellWiring {
    fn default() -> Self {
        Self::new()
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
#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn ANativeActivity_onCreate(
    activity: *mut std::ffi::c_void,
    saved_state: *const u8,
    saved_state_size: usize,
) {
    glue_ANativeActivity_onCreate(activity, saved_state, saved_state_size)
}

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
        pub fn ALooper_pollAll(
            timeout_millis: i32,
            out_fd: *mut i32,
            out_events: *mut i32,
            out_data: *mut *mut std::ffi::c_void,
        ) -> i32;
    }
}

/// 主机装配态（android_main 主循环持有：四 trait 实现 + 游戏机 + 画布）。
#[cfg(target_os = "android")]
pub struct HostState {
    pub wiring: ShellWiring,
    pub machine: game_core::paint::BootMachine,
    pub canvas: game_core::render::ArgbImage,
    pub assets: crate::asset::apk::ApkAssetSource,
    pub _storage: crate::storage::AndroidStorage,
    pub surface: (u32, u32),
    pub pending_keys: Vec<KeyAction>,
}

#[cfg(target_os = "android")]
extern "C" fn handle_cmd(app: *mut AndroidApp, cmd: i32) {
    unsafe {
        let st = &mut *((*app).user_data as *mut HostState);
        match cmd {
            APP_CMD_INIT_WINDOW => {
                if !(*app).window.is_null() {
                    // 窗口就绪：格式协商在 present() 内（setBuffersGeometry 240×320）
                    st.surface = (CANVAS_W, CANVAS_H);
                    st.wiring.surface = st.surface;
                }
            }
            APP_CMD_TERM_WINDOW => {}
            _ => {}
        }
        let in_pip = false; // PiP 判定（isInPictureInPictureMode）走 JNI；默认非 PiP
        if let Some(_n) = st.wiring.on_lifecycle(cmd, in_pip) {
            // 边沿交游戏机：hide_notify/showNotify（BootMachine::tick 入口同构消费）
        }
    }
}

#[cfg(target_os = "android")]
extern "C" fn handle_input(app: *mut AndroidApp, event: *mut AInputEvent) -> i32 {
    unsafe {
        let st = &mut *((*app).user_data as *mut HostState);
        let (keys, consumed) = st.wiring.on_input_ex(event);
        st.pending_keys.extend(keys);
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
        use crate::asset::apk::ApkAssetSource;
        use crate::storage::AndroidStorage;
        use game_core::paint::BootMachine;
        use game_platform::{AssetSource, AudioOut};

        /// 无声降级设备（AAudio 开失败/API<26）：吞写不阻塞（loop 自带节拍）。
        struct SilentOut;
        impl AudioOut for SilentOut {
            fn write(&mut self, _frames: &[i16]) -> Result<(), String> {
                Ok(())
            }
            fn idle(&mut self) {}
        }
        use game_core::render::{ArgbImage, SoftGraphics};
        use game_data::{decode_png, PackedPng};
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
        let _ = &stores; // 持久记录（SKY_WAR 等）；BootMachine.apply_global 接线随实机核对

        // BootMachine 装配（win main.rs:560-580 同构：APK assets 容器→游戏机）
        let png = |name: &str| {
            ArgbImage::from_decoded_png(
                decode_png(&assets.read(name).expect("缺 assets 资产")).expect("png 解码"),
            )
        };
        let container = |name: &str, count: usize| {
            PackedPng::parse(&assets.read(name).expect("缺容器资产"), count)
                .expect("容器解析")
                .images
                .iter()
                .map(|s| ArgbImage::from_decoded_png(decode_png(s.bytes).unwrap()))
                .collect::<Vec<_>>()
        };
        let mapbg = container("mapbg", 1);
        let mut machine = BootMachine::new(
            png("l0.png"),
            png("l1.png"),
            container("sflogo", 8),
            container("ui", 25),
            container("menu", 2),
            mapbg[0].clone(),
            container("intro", 2),
            container("end", 1),
            container("load", 2),
        );
        machine.sptarm = container("sptarm", 10);
        machine.apply_layout(game_core::layout::Layout::classic());

        let mut st = Box::new(HostState {
            wiring: ShellWiring::new(),
            machine,
            canvas: ArgbImage::create(CANVAS_W as i32, CANVAS_H as i32),
            assets,
            _storage: storage,
            surface: (CANVAS_W, CANVAS_H),
            pending_keys: Vec::new(),
        });
        (*app).user_data = &mut *st as *mut HostState as *mut std::ffi::c_void;
        (*app).on_app_cmd = Some(handle_cmd);
        (*app).on_input_event = Some(handle_input);

        // AAudio 音频环（阻塞 write=节拍；开失败=NullSink 无声降级，API<26 同）
        let (tx, rx) = std::sync::mpsc::channel::<game_shell_common::audio::Msg>();
        std::thread::spawn(move || {
            let dev: Box<dyn AudioOut> = match crate::audio::aaudio::AAudioPcm::open() {
                Ok(pcm) => Box::new(pcm),
                Err(_) => Box::new(SilentOut),
            };
            let store: game_shell_common::audio::TrackStore =
                std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            game_shell_common::audio::audio_loop(rx, dev, game_shell_common::audio::Mixer::new(store));
        });
        let mut last_bgm: Option<usize> = None;

        // 20 拍/秒呈现节拍（50ms）；逻辑拍=frame_interval_ms（75/100）
        let present_budget = std::time::Duration::from_millis(50);
        let mut last_logic = std::time::Instant::now();
        while (*app).destroy_requested == 0 {
            let mut events = 0i32;
            let mut data: *mut std::ffi::c_void = std::ptr::null_mut();
            while ffi::ALooper_pollAll(0, std::ptr::null_mut(), &mut events, &mut data) >= 0 {
                if !data.is_null() {
                    let src = data as *mut AndroidPollSource;
                    if let Some(process) = (*src).process {
                        process(app, src);
                    }
                }
                if (*app).destroy_requested != 0 {
                    break;
                }
            }
            // 逻辑拍（frame_interval_ms 节奏）：键流折叠单键（覆盖式取末键）
            let interval = std::time::Duration::from_millis(st.wiring.frame_interval_ms());
            if last_logic.elapsed() >= interval {
                last_logic = std::time::Instant::now();
                let key = st.pending_keys.drain(..).fold(0i32, |_, a| match a {
                    KeyAction::Press(c) => c,
                    KeyAction::Release => 0,
                });
                st.machine.tick(key);
                // BGM：轨道变更 → 音频线程（bgm_request/bgm_serial 同构 shell/win）
                if let Some(track) = st.machine.bgm_request() {
                    let serial = st.machine.bgm_serial();
                    if last_bgm != Some(track) {
                        last_bgm = Some(track);
                        let _ = tx.send(game_shell_common::audio::Msg::Cmd(
                            game_shell_common::audio::Cmd::Play { track, gain: 1.0 },
                        ));
                        let _ = serial;
                    }
                }
            }
            // 呈现（20 拍/秒）
            {
                let mut g = SoftGraphics::new(&mut st.canvas);
                st.machine.paint(&mut g);
            }
            let argb = st.canvas.argb.clone();
            st.wiring.present((*app).window, &argb);
            std::thread::sleep(present_budget / 4);
        }
        (*app).user_data = std::ptr::null_mut();
        drop(st);
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
        assert_eq!(w.surface, (CANVAS_W, CANVAS_H), "逻辑画布 240×320（Stage A）");
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
