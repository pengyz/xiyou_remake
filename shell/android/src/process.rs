//! ProcessControl 实现：退出执行（游戏层只发信号，shell 执行）。
//!
//! # 契约（docs/spec/platform-contract.md §2）
//!
//! `request_exit(&mut self)`：幂等、不得 panic、**正常返回**（置退出请求，进程可随后
//! 在壳收尾中结束）。游戏层只发信号（`BootMachine::finished`：game-core paint.rs:221/585
//! = `CMidlet.dispatchModeEntry`；BUG-020 消费链），执行归壳。
//!
//! # 平台映射
//!
//! - 宿主桩 [`HostProcessControl`]：记录退出请求（幂等观测面）——宿主测试/无头验证。
//! - Android [`android::AndroidProcessControl`]：`ANativeActivity_finish`
//!   （NDK 封装 JNI `Activity.finish()`，本身**不杀进程**）；进程结束由壳在 Activity 销毁后执行
//!   （native.rs `android_main` 尾 `_exit(0)`，BUG-033）。

use game_platform::ProcessControl;
use std::sync::atomic::{AtomicUsize, Ordering};

/// 同进程 Activity 实例计数（BUG-036）。NativeActivity 允许系统把新实例放进**尚未退出的旧进程**：
/// 用户退出后约 1 s 内再点图标 ⇒ 旧实例已 finish、未销毁，新实例 `onCreate` 已在同进程跑起
/// （native_app_glue 每实例一条 `android_main` 线程）。旧实例销毁后若直接 `_exit` 会连新实例一起杀掉
/// （实机：ActivityManager `has died: fg TOP` ⇒ 系统另起进程重启 = 用户所见"又重来一遍"）。
/// 登记点 = `ANativeActivity_onCreate`（主线程，先于该实例任何生命周期回调）；注销点 = `android_main` 末。
#[derive(Debug, Default)]
pub struct InstanceCounter(AtomicUsize);

impl InstanceCounter {
    pub const fn new() -> InstanceCounter {
        InstanceCounter(AtomicUsize::new(0))
    }

    /// 登记一个实例（`ANativeActivity_onCreate`）。
    pub fn enter(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }

    /// 注销本实例，返回进程内剩余存活实例数（不下溢：未登记即注销 = 0）。
    pub fn leave(&self) -> usize {
        let prev = self
            .0
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| Some(n.saturating_sub(1)))
            .unwrap_or(0);
        prev.saturating_sub(1)
    }
}

/// 实例销毁后壳是否结束进程：本实例发起过退出（BUG-033 退出链）∧ 进程内已无其他实例（BUG-036）。
/// 非本方发起的销毁（系统回收 / 配置重建 / 最近任务划掉）不结束进程，交系统处置。
pub fn end_process_after_destroy(exit_sent: bool, remaining_instances: usize) -> bool {
    exit_sent && remaining_instances == 0
}

/// 宿主桩：记录退出请求（不杀进程——同契约：桌面=退事件循环、Android=finish Activity）。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct HostProcessControl {
    requested: bool,
    calls: u64,
}

impl HostProcessControl {
    pub fn new() -> HostProcessControl {
        HostProcessControl::default()
    }

    /// 是否已请求退出（壳循环每拍消费；`BootMachine::finished` 对偶）。
    pub fn exit_requested(&self) -> bool {
        self.requested
    }

    /// request_exit 被调用次数（幂等观测：重复调用无额外副作用，但可计数审计）。
    pub fn calls(&self) -> u64 {
        self.calls
    }
}

impl ProcessControl for HostProcessControl {
    fn request_exit(&mut self) {
        self.calls += 1;
        self.requested = true;
    }
}

/// Android 实现（`#[cfg(target_os = "android")]` 门控；宿主编译走 [`HostProcessControl`]）。
#[cfg(target_os = "android")]
pub mod android {
    //! TODO(骨架·未验证)：`ANativeActivity_finish` = NDK 对 JNI `Activity.finish()` 的
    //! 封装（android/native_activity.h，libandroid.so）。本环境无 SDK/NDK——下列
    //! extern 签名照 NDK 头文件手写，**未经编译验证**；BACK 默认行为（未消费 ⇒ 系统
    //! finish）与 finish 线程约束待实机复核升 A（README 验证状态矩阵）。
    use super::ProcessControl;
    use std::ffi::c_void;

    /// `ANativeActivity`（不透明；仅透传指针）。
    #[repr(C)]
    pub struct ANativeActivity {
        _private: [u8; 0],
    }

    extern "C" {
        /// android/native_activity.h：`void ANativeActivity_finish(ANativeActivity* activity);`
        /// 内部 = JNI `Activity.finish()`（不杀进程；可重复调用 ⇒ 天然幂等）。
        fn ANativeActivity_finish(activity: *mut ANativeActivity);
    }

    /// Android 进程控制：finish Activity（不杀进程）。
    pub struct AndroidProcessControl {
        activity: *mut ANativeActivity,
    }

    // SAFETY：activity 指针由 NativeActivity 胶水在 Activity 生命周期内持有，
    // request_exit 只在壳主线程调用（TODO(骨架)：finish 线程约束待实机核对——
    // ANativeActivity_finish 自带 JNI 投递，调用线程需已 attach）。
    unsafe impl Send for AndroidProcessControl {}

    impl AndroidProcessControl {
        /// `activity` = `(*mut ANativeActivity)`（native_app_glue 的 `android_app->activity`）。
        pub fn new(activity: *mut ANativeActivity) -> AndroidProcessControl {
            AndroidProcessControl { activity }
        }
    }

    impl ProcessControl for AndroidProcessControl {
        fn request_exit(&mut self) {
            // SAFETY：指针来自 native_app_glue 且生命周期覆盖壳运行期。
            unsafe {
                ANativeActivity_finish(self.activity);
            }
        }
    }

    /// 骨架占位（防未使用告警；c_void 引用面留 JNI 桥扩展缝）。
    pub type JniEnvPtr = *mut c_void;
}
