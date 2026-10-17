//! ProcessControl 契约测试（docs/spec/platform-contract.md §2）。
//!
//! 契约面 = `game_platform::ProcessControl`（退出执行的 shell 半边）。真实实现
//! 在壳层（shell/win winit `el.exit()`/exit-flag 信箱、shell/android Activity.finish），
//! 本 crate 只放 trait 与类型（AGENTS §2 红线）⇒ 此处以测试替身承载契约语义的
//! 可执行文档 + API 形状锁（含"无 'static / 无 Send"负形状锁）。
//!
//! TDD RED 复现（A 强化协议 §7）：本文件单独应用到父树（`game_platform` 尚无
//! ProcessControl）⇒ `cargo test -p game-platform --test process_control_contract`
//! 编译失败（E0432）即红。

use game_platform::ProcessControl;

/// 测试替身：记录退出请求（不触进程/事件循环）。
#[derive(Default)]
struct ExitRecorder {
    requests: u32,
}

impl ProcessControl for ExitRecorder {
    fn request_exit(&mut self) {
        self.requests += 1;
    }
}

/// 退出执行归 shell：trait 只承接"执行退出"这一半，游戏层信号
/// （BootMachine::finished / CMidlet.m_000）不入 trait（spec §2.2）。
#[test]
fn request_exit_runs_the_shell_side_effect() {
    let mut p = ExitRecorder::default();
    p.request_exit();
    assert_eq!(p.requests, 1);
}

/// 幂等契约：重复调用不得 panic；平台效果由实现去重（winit `el.exit()` 天然幂等）。
#[test]
fn request_exit_allows_repeated_calls() {
    let mut p = ExitRecorder::default();
    for _ in 0..3 {
        p.request_exit();
    }
    assert_eq!(p.requests, 3, "调用可见次数由实现决定；契约只保证重复调用合法");
}

/// 形状锁①：对象可用（壳层以 `&mut dyn ProcessControl` 接线）。
#[test]
fn process_control_is_object_safe() {
    fn via_dyn(p: &mut dyn ProcessControl) {
        p.request_exit();
    }
    let mut p = ExitRecorder::default();
    via_dyn(&mut p);
    assert_eq!(p.requests, 1);
}

/// 形状锁②：**无 `'static` 约束**——带生命周期的实现必须能编译（winit 侧允许
/// 作用域内薄适配器借用 `ActiveEventLoop`，spec §2.2）。
#[test]
fn process_control_has_no_static_bound() {
    struct ScopedExit<'a> {
        flag: &'a std::cell::Cell<bool>,
    }
    impl ProcessControl for ScopedExit<'_> {
        fn request_exit(&mut self) {
            self.flag.set(true);
        }
    }
    let flag = std::cell::Cell::new(false);
    let mut p = ScopedExit { flag: &flag };
    p.request_exit();
    assert!(flag.get(), "作用域适配器的退出请求落到借用的信箱");
}

/// 形状锁③：**无 Send 约束**——线程本地实现（如事件循环句柄）必须能编译。
#[test]
fn process_control_has_no_send_bound() {
    use std::rc::Rc;
    // 字段是负形状锁的载体（Rc 即非 Send 证明），不读。
    #[allow(dead_code)]
    struct LocalExit(Rc<()>);
    impl ProcessControl for LocalExit {
        fn request_exit(&mut self) {}
    }
    let mut p = LocalExit(Rc::new(()));
    p.request_exit();
}
