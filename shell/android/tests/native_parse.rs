//! T5 宿主侧解析单测（task-14 TDD 红先行）：AInputEvent 解析面（MotionEvent→
//! TouchPhase/触点、Key→-1..-7）、呈现 blit（ARGB→RGBA8888 信箱）、生命周期
//! APP_CMD_*→Notify。全部纯函数（cfg(android) FFI 之上的可测层）。

use game_android::native::{
    blit_argb_rgba, hw_key_from_android, key_actions_from_android, lifecycle_notify,
    motion_phase, motion_pointer_index, APP_CMD_PAUSE, APP_CMD_RESUME, APP_CMD_START,
    AKEYCODE_BACK, AKEYCODE_DPAD_CENTER, AKEYCODE_DPAD_DOWN, AKEYCODE_DPAD_LEFT,
    AKEYCODE_DPAD_RIGHT, AKEYCODE_DPAD_UP, AKEYCODE_ENTER, AKEYCODE_MENU, AKEYCODE_SPACE,
    AKEYCODE_VOLUME_DOWN, AKEYCODE_VOLUME_UP, AMOTION_EVENT_ACTION_CANCEL,
    AMOTION_EVENT_ACTION_DOWN, AMOTION_EVENT_ACTION_MOVE, AMOTION_EVENT_ACTION_POINTER_DOWN,
    AMOTION_EVENT_ACTION_POINTER_UP, AMOTION_EVENT_ACTION_UP,
};
use game_android::input::{HwKey, KeyAction, TouchPhase};
use game_android::native::{CANVAS_H, CANVAS_W};
use game_android::lifecycle::{Notify, Visibility};

// ───────────────────────────────────────── MotionEvent 解析

#[test]
fn motion_phase_maps_android_actions() {
    assert_eq!(motion_phase(AMOTION_EVENT_ACTION_DOWN), Some(TouchPhase::Down));
    assert_eq!(motion_phase(AMOTION_EVENT_ACTION_UP), Some(TouchPhase::Up));
    assert_eq!(motion_phase(AMOTION_EVENT_ACTION_MOVE), Some(TouchPhase::Move));
    assert_eq!(motion_phase(AMOTION_EVENT_ACTION_CANCEL), Some(TouchPhase::Cancel));
    assert_eq!(motion_phase(AMOTION_EVENT_ACTION_POINTER_DOWN), Some(TouchPhase::Down));
    assert_eq!(motion_phase(AMOTION_EVENT_ACTION_POINTER_UP), Some(TouchPhase::Up));
    assert_eq!(motion_phase(4), None, "ACTION_OUTSIDE 不入触点管线");
}

#[test]
fn motion_pointer_index_from_action_high_bits() {
    assert_eq!(motion_pointer_index(0), 0);
    // POINTER_DOWN/UP：pointer index 在 (action>>8)&0xff
    assert_eq!(motion_pointer_index((2 << 8) | AMOTION_EVENT_ACTION_POINTER_DOWN), 2);
    assert_eq!(motion_pointer_index((1 << 8) | AMOTION_EVENT_ACTION_POINTER_UP), 1);
    assert_eq!(motion_pointer_index(AMOTION_EVENT_ACTION_MOVE), 0);
}

// ───────────────────────────────────────── KeyEvent 解析（-1..-7 键码契约）

#[test]
fn hw_key_from_android_codes() {
    assert_eq!(hw_key_from_android(AKEYCODE_BACK), HwKey::Back);
    assert_eq!(hw_key_from_android(AKEYCODE_DPAD_UP), HwKey::DpadUp);
    assert_eq!(hw_key_from_android(AKEYCODE_DPAD_DOWN), HwKey::DpadDown);
    assert_eq!(hw_key_from_android(AKEYCODE_DPAD_LEFT), HwKey::DpadLeft);
    assert_eq!(hw_key_from_android(AKEYCODE_DPAD_RIGHT), HwKey::DpadRight);
    assert_eq!(hw_key_from_android(AKEYCODE_DPAD_CENTER), HwKey::DpadCenter);
    assert_eq!(hw_key_from_android(AKEYCODE_ENTER), HwKey::Enter);
    assert_eq!(hw_key_from_android(AKEYCODE_SPACE), HwKey::Space);
    assert_eq!(hw_key_from_android(AKEYCODE_MENU), HwKey::Menu);
    assert_eq!(hw_key_from_android(AKEYCODE_VOLUME_UP), HwKey::VolumeUp);
    assert_eq!(hw_key_from_android(AKEYCODE_VOLUME_DOWN), HwKey::VolumeDown);
    assert_eq!(hw_key_from_android(1000), HwKey::Other);
}

#[test]
fn key_press_maps_to_game_codes() {
    // Key→-1..-7（key-code-contract §4）
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_UP, 0, 0), (vec![KeyAction::Press(-1)], true));
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_DOWN, 0, 0), (vec![KeyAction::Press(-2)], true));
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_LEFT, 0, 0), (vec![KeyAction::Press(-3)], true));
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_RIGHT, 0, 0), (vec![KeyAction::Press(-4)], true));
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_CENTER, 0, 0), (vec![KeyAction::Press(-5)], true));
    assert_eq!(key_actions_from_android(AKEYCODE_MENU, 0, 0), (vec![KeyAction::Press(-6)], true));
    assert_eq!(key_actions_from_android(AKEYCODE_BACK, 0, 0), (vec![KeyAction::Press(-7)], true));
}

#[test]
fn key_release_and_repeat_semantics() {
    // UP → Release（不带键）；repeat 丢弃（不投递）但消费性不变
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_UP, 1, 0), (vec![KeyAction::Release], true));
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_UP, 0, 3), (vec![], true), "OS 重复丢弃");
    // BACK 恒吞（BUG-020：含 repeat 与 UP 都不得走系统 finish）
    assert_eq!(key_actions_from_android(AKEYCODE_BACK, 0, 2), (vec![], true), "BACK repeat 仍吞");
    assert_eq!(key_actions_from_android(AKEYCODE_BACK, 1, 0), (vec![KeyAction::Release], true), "BACK UP 仍吞");
    // 音量/未知：不投递不吞（交系统）
    assert_eq!(key_actions_from_android(AKEYCODE_VOLUME_UP, 0, 0), (vec![], false));
    assert_eq!(key_actions_from_android(1000, 0, 0), (vec![], false));
}

// ───────────────────────────────────────── 生命周期 cmd → Notify

#[test]
fn lifecycle_cmd_dispatch() {
    let mut vis = Visibility::new();
    assert_eq!(lifecycle_notify(&mut vis, APP_CMD_PAUSE, false), Some(Notify::Hide));
    assert_eq!(lifecycle_notify(&mut vis, APP_CMD_PAUSE, false), None, "latch 不重发");
    assert_eq!(lifecycle_notify(&mut vis, APP_CMD_RESUME, false), Some(Notify::Show));
    assert_eq!(lifecycle_notify(&mut vis, APP_CMD_RESUME, false), None);
    // PiP=可见非遮挡不发 Hide（lifecycle.rs S1）
    let mut v2 = Visibility::new();
    assert_eq!(lifecycle_notify(&mut v2, APP_CMD_PAUSE, true), None);
    assert!(v2.shown(), "PiP pause 不动 latch");
    // 其余 cmd 无边沿
    assert_eq!(lifecycle_notify(&mut v2, APP_CMD_START, false), None);
    assert_eq!(lifecycle_notify(&mut v2, 1, false), None, "INIT_WINDOW 无 Notify");
}

// ───────────────────────────────────────── 呈现 blit（ARGB→RGBA8888 信箱）

#[test]
fn blit_argb_to_rgba8888_byte_order() {
    // 240×320 画布直拷进等大窗口：逐像素 [R,G,B,A]（WINDOW_FORMAT_RGBA_8888）
    let argb = [0xFF112233u32, 0x80445566]; // 0xAARRGGBB
    let mut dst = vec![0u8; 2 * 4];
    let r = blit_argb_rgba(&mut dst, 2, 1, &argb, 2, 1);
    assert_eq!(r, (0, 0, 2, 1), "1:1 直拷无信箱");
    assert_eq!(&dst[0..4], &[0x11, 0x22, 0x33, 0xFF], "R,G,B,A 序");
    assert_eq!(&dst[4..8], &[0x44, 0x55, 0x66, 0x80], "alpha 保真");
}

#[test]
fn blit_letterbox_on_larger_surface() {
    // 4×2 窗口装 2×2 画布：居中信箱（宽 2 高 2 ⇒ 左右各补 1 列 0）
    let argb = [0xFFFFFFFFu32, 0xFF000000, 0xFF00FF00, 0xFFFF0000];
    let mut dst = vec![0xAAu8; 4 * 2 * 4];
    let r = blit_argb_rgba(&mut dst, 4, 2, &argb, 2, 2);
    assert_eq!(r, (1, 0, 2, 2), "居中信箱 rect（x=1,w=2）");
    // 左补列=0（dst 先清）
    assert_eq!(&dst[0..4], &[0, 0, 0, 0], "信箱区清零");
    assert_eq!(&dst[4..8], &[0xFF, 0xFF, 0xFF, 0xFF], "画布首像素落 (1,0)");
}

#[test]
fn blit_scales_up_by_nearest_when_window_larger_in_both_axes() {
    // 4×4 窗口装 2×2 画布：等比放大 ×2（信箱消隐）——像素块复制
    let argb = [0xFF112233u32, 0xFF445566, 0xFF778899, 0xFFAABBCC];
    let mut dst = vec![0u8; 4 * 4 * 4];
    let r = blit_argb_rgba(&mut dst, 4, 4, &argb, 2, 2);
    assert_eq!(r, (0, 0, 4, 4), "放大满窗");
    // (0,0) 与 (1,0) 同源 (0,0)
    assert_eq!(&dst[0..4], &[0x11, 0x22, 0x33, 0xFF]);
    assert_eq!(&dst[4..8], &[0x11, 0x22, 0x33, 0xFF], "最近邻 ×2 块复制");
    // (0,2) 源 (0,1)
    assert_eq!(&dst[2 * 4 * 4..2 * 4 * 4 + 4], &[0x77, 0x88, 0x99, 0xFF], "第二行源 (0,1)");
}

#[test]
fn blit_rect_reported_for_painter() {
    // 返回矩形=呈现脏区（unlockAndPost 前的 lock 缓冲换算面）
    let mut dst = vec![0u8; 3 * 3 * 4];
    let argb = vec![0xFF000000u32; 9];
    let r = blit_argb_rgba(&mut dst, 3, 3, &argb, 3, 3);
    assert_eq!(r, (0, 0, 3, 3));
}

// ───────────────────────────────────────── 窗口未就绪 No-op 钉（实机 SIGSEGV 回派）

#[test]
fn present_null_window_noop() {
    // 实机崩因钉（SIGSEGV setBuffersGeometry 空窗口）：window=NULL ⇒ present 必须
    // No-op（不触 FFI），返回未呈现。
    let mut w = game_android::native::ShellWiring::new();
    let argb = vec![0u32; CANVAS_W as usize * CANVAS_H as usize];
    let null_win: *mut game_android::native::ANativeWindow = std::ptr::null_mut();
    assert!(!w.present(null_win, &argb), "window 未就绪=No-op 且报告未呈现");
}

#[test]
fn surface_negotiate_null_window_noop() {
    // 几何协商同判：APP_CMD_INIT_WINDOW 处理里也只在 window 非空时协商
    let mut w = game_android::native::ShellWiring::new();
    let null_win: *mut game_android::native::ANativeWindow = std::ptr::null_mut();
    assert!(!w.ensure_surface(null_win), "NULL window 不协商（不调 setBuffersGeometry）");
}
