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

use crate::input::{KeyAction, PadLayout};
use crate::lifecycle::{Notify, Visibility};

/// 逻辑画布（Stage A 240×320；game_core::layout 同值）。
pub const CANVAS_W: u32 = 240;
pub const CANVAS_H: u32 = 320;

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

/// `android_app`（native_app_glue；不透明——骨架只透传）。
#[repr(C)]
pub struct AndroidApp {
    _private: [u8; 0],
}

/// 装配态（骨架：字段随实现期补全——BootMachine/GameScene/BgmController 等）。
pub struct ShellWiring {
    pub visibility: Visibility,
    pub pad: PadLayout,
}

impl ShellWiring {
    pub fn new() -> ShellWiring {
        ShellWiring { visibility: Visibility::new(), pad: PadLayout::default_240x320() }
    }

    /// 逻辑拍节奏（毫秒）：mode 0 = 100（a.java:3350）、其余 = 75（a.java:3377）——
    /// shell/win `frame_interval()` 同构（main.rs:645-650）。
    pub fn frame_interval_ms(&self) -> u64 {
        75
    }

    /// 输入事件翻译（骨架：坐标经 Viewport 反映射后进 InputRouter）。
    /// TODO(骨架)：AInputEvent → TouchEvent/HwKey 的 NDK 解析；
    /// **BACK 恒返回 consumed**（BUG-020 退出链：不得走系统默认 finish）。
    pub fn on_input(&mut self, _event: *mut AInputEvent) -> Vec<KeyAction> {
        Vec::new()
    }

    /// 生命周期回调（骨架）：APP_CMD_RESUME ⇒ on_resume；APP_CMD_PAUSE ⇒ on_pause
    /// （in_pip = isInPictureInPictureMode——PiP=可见非遮挡不发 Hide，lifecycle.rs S1）。
    /// 边沿消费 = game-core `hide_notify()`（mode3→4 + closeAudioPlayer + anyKeyPlayed）
    /// 与 showNotify（只清 latch）——壳只决定何时调（shell/win main.rs:578-622 同构）。
    pub fn on_lifecycle(&mut self, _cmd: i32, in_pip: bool) -> Option<Notify> {
        None
    }

    /// 呈现（骨架）：ArgbImage → ANativeWindow 缓冲（等比信箱 blit）。
    /// TODO(骨架)：ANativeWindow_lock/unlockAndPost + RGBA8888 格式协商。
    pub fn present(&mut self, _window: *mut ANativeWindow, _argb: &[u32]) {}
}

impl Default for ShellWiring {
    fn default() -> Self {
        Self::new()
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
}
