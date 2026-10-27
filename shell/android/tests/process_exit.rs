//! 进程结束判定宿主钉（BUG-036；native.rs：`ANativeActivity_onCreate` 登记实例、`android_main` 末注销并判定）。
//!
//! 【2026-10-27 用户实机】pid 32119：16:39:44.367 主菜单返回 → finish；用户 1 s 后再点图标 ⇒ 16:39:45.384 START，
//! 系统把新 Activity 实例放进**尚未退出的旧进程**（16:39:45.407 第二个 `android_main entered`，同 pid）；
//! 16:39:45.829 旧实例销毁后 `_exit(0)` ⇒ 连同新实例一起杀掉 ⇒ ActivityManager `has died: fg TOP` ⇒ 系统另起
//! 进程重启（16:39:46.826 pid 32638）——用户所见"又重来一遍"。16:38:43 / 16:38:48 / 16:38:58 三次同型。
//! 判定：仅当本实例发起过退出 ∧ 进程内已无其他实例，才结束进程。

use game_android::process::{end_process_after_destroy, InstanceCounter};

/// 单实例：退出 → 销毁 ⇒ 结束进程（BUG-033 原口径不变）。
#[test]
fn single_instance_exit_ends_process() {
    let c = InstanceCounter::new();
    c.enter();
    assert!(end_process_after_destroy(true, c.leave()));
}

/// 退出后立即再点图标：新实例进了将退出的旧进程 ⇒ 旧实例销毁时**不得**结束进程；新实例日后退出才结束。
#[test]
fn relaunch_into_exiting_process_keeps_it_alive() {
    let c = InstanceCounter::new();
    c.enter(); // A：启动
    c.enter(); // B：A finish 后、A 销毁前，用户再点图标（同进程新实例）
    assert!(!end_process_after_destroy(true, c.leave()), "A 销毁：B 仍存活 ⇒ 保留进程");
    assert!(end_process_after_destroy(true, c.leave()), "B 退出销毁：已无实例 ⇒ 结束进程");
}

/// 非本方发起的销毁（系统回收 / 配置重建 / 最近任务划掉）⇒ 不由壳结束进程（交系统处置）。
#[test]
fn destroy_without_exit_request_keeps_process() {
    let c = InstanceCounter::new();
    c.enter();
    assert!(!end_process_after_destroy(false, c.leave()));
}

/// 计数不下溢：未登记即注销返回 0，之后登记/注销照常。
#[test]
fn leave_without_enter_does_not_underflow() {
    let c = InstanceCounter::new();
    assert_eq!(c.leave(), 0);
    c.enter();
    assert_eq!(c.leave(), 0);
    c.enter();
    c.enter();
    assert_eq!(c.leave(), 1);
}
