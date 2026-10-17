//! 单元 2 契约测试：Android Activity 生命周期 → hideNotify/showNotify（TDD RED 先行）。
//!
//! 真值锚：a.java:4495-4513（showNotify/hideNotify + f_bool_01 latch）、
//! CMidlet.java:19（startApp 显式 showNotify）、shell/win/src/lifecycle.rs（同构先例）、
//! 失焦不映射（用户决策 2026-10-15）、画中画=可见非遮挡（对抗审查 S1）、
//! 音频焦点砍除（对抗审查 F-6）。

use game_android::input::{InputRouter, PadLayout, TouchEvent, TouchPhase};
use game_android::lifecycle::{Notify, Visibility};

/// startApp 后首拍即 shown（CMidlet.java:19 显式 showNotify）。
#[test]
fn starts_shown_after_startapp() {
    assert!(Visibility::new().shown());
}

/// onPause ⇒ 恰一次 Hide；重复 pause（onStop 等同态观测）不重发（f_bool_01 latch）。
#[test]
fn on_pause_emits_hide_exactly_once() {
    let mut v = Visibility::new();
    assert_eq!(v.on_pause(false), Some(Notify::Hide));
    assert!(!v.shown());
    assert_eq!(v.on_pause(false), None, "latch：重复 pause 无边沿");
}

/// onResume ⇒ Show；重复 resume 无边沿。
#[test]
fn on_resume_emits_show_from_hidden() {
    let mut v = Visibility::new();
    assert_eq!(v.on_resume(), None, "shown 态 resume 无边沿");
    v.on_pause(false);
    assert_eq!(v.on_resume(), Some(Notify::Show));
    assert!(v.shown());
    assert_eq!(v.on_resume(), None);
}

/// pause/resume 往返 ×3 幂等：每次恰一边沿。
#[test]
fn pause_resume_cycles_are_idempotent() {
    let mut v = Visibility::new();
    for _ in 0..3 {
        assert_eq!(v.on_pause(false), Some(Notify::Hide));
        assert_eq!(v.on_resume(), Some(Notify::Show));
    }
}

/// 画中画 = 可见非遮挡（「完全遮挡才 hide」锚，S1）⇒ onPause(in_pip=true) 不发 Hide。
#[test]
fn pip_pause_emits_no_hide() {
    let mut v = Visibility::new();
    assert_eq!(v.on_pause(true), None);
    assert!(v.shown(), "PiP 仍算可见");
}

/// PiP 期间的真实隐藏（如锁屏）仍发 Hide：in_pip=false 的 pause 不被吞。
#[test]
fn real_pause_during_pip_still_hides() {
    let mut v = Visibility::new();
    assert_eq!(v.on_pause(true), None);
    assert_eq!(v.on_pause(false), Some(Notify::Hide));
}

/// 可见失焦（分屏/自由窗口的 onTopResumedActivityChanged）**不**映射隐藏
/// （用户决策 2026-10-15：失焦 ≠ 隐藏；与 Windows Occluded/最小化口径同）。
#[test]
fn top_resumed_change_emits_nothing() {
    let mut v = Visibility::new();
    assert_eq!(v.on_top_resumed_changed(false), None);
    assert!(v.shown());
    assert_eq!(v.on_top_resumed_changed(true), None);
    v.on_pause(false);
    assert_eq!(v.on_top_resumed_changed(true), None, "隐藏期的 top 变化也不产边沿");
}

/// 音频焦点**无**任何观测口（F-6 砍除）：Visibility 只有 on_pause/on_resume/
/// on_top_resumed_changed 三个 Android 观测口——加焦点口须 decision 登记 + 用户确认
/// （见 lifecycle.rs 模块注释；此处以行为面锁定：任何"焦点丢/回"都不对应本状态机
/// 的输入，焦点不得经由隐藏边沿混入）。
#[test]
fn no_focus_edge_aliasing() {
    let mut v = Visibility::new();
    // 焦点事件不存在于本状态机输入面；等价检验 = 只有 pause/resume 两条边改变 shown。
    let before = v.shown();
    v.on_top_resumed_changed(false); // 焦点类观测（top 变化）不改状态
    assert_eq!(v.shown(), before);
    v.on_pause(false);
    assert!(!v.shown());
}

/// 接线契约：隐藏期输入由 InputRouter 丢弃、恢复后可投递
/// （Visibility.shown() → InputRouter::set_shown 的壳侧接线）。
#[test]
fn hidden_gates_router_until_show() {
    let mut v = Visibility::new();
    let mut r = InputRouter::new(PadLayout::default_240x320());
    let down = |x: f32, y: f32| TouchEvent { id: 1, phase: TouchPhase::Down, x, y };

    v.on_pause(false);
    r.set_shown(v.shown());
    assert!(r.touch(&down(56.0, 200.0)).is_empty(), "隐藏期不投递");

    v.on_resume();
    r.set_shown(v.shown());
    assert!(!r.touch(&down(56.0, 200.0)).is_empty(), "show 后恢复投递");
}
