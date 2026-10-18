//! MIDP Canvas 可见性生命周期（宿主侧，纯计算可单测）：窗口观测 → showNotify / hideNotify 边沿。
//!
//! Java（行号指 reference/src/deobf/a.java；字节码偏移指 renamed a.class `javap -c -p`，A 级）：
//!
//! ```text
//! showNotify()  a.java:4495-4500 / @0-17   if (!f_bool_01) { f_bool_01 = true; }
//! hideNotify()  a.java:4502-4513 / @0-43   if (f_bool_01) {
//!                                             if (gameMode == 3) { gameMode = 4; dispatchModeEntry(); }
//!                                             closeAudioPlayer(); anyKeyPlayed = true;
//!                                             f_bool_01 = false; }
//! ```
//!
//! `f_bool_01`（a.java:32）无初值（false）；`CMidlet.startApp` 显式 `showNotify()`
//! （CMidlet.java:19）⇒ 运行期首拍起恒 true（oracle trace `FLD 008` 全程 true，A 级）。
//! 它的全部读写点就是上面 4 处（a.java:4496/4497/4503/4511）——只是 show/hide 的去重 latch，
//! 无其他读者，故由宿主持有与由 game-core 持有不可区分。本模块的 [`Visibility::shown`] 即它。
//!
//! 窗口 → 可见性（桌面映射，壳的解释）：**hidden = 完全遮挡 ∨ 最小化**。
//! - `WindowEvent::Occluded(bool)`：X11 来自 VisibilityNotify `FullyObscured`
//!   （winit 0.30.13 `platform_impl/linux/x11/event_processor.rs:871`）；macOS/iOS/Web 另有来源。
//! - 最小化：X11 的 iconify = unmap，X 协议对「变为不可视」**不发** VisibilityNotify，winit 也不
//!   转发 UnmapNotify / `_NET_WM_STATE` 的 PropertyNotify ⇒ 没有事件，只能轮询
//!   `Window::is_minimized()`（读 `_NET_WM_STATE_HIDDEN`，`x11/window.rs:864`）。
//! - 失焦**不**映射（用户决策 2026-10-15：失焦 ≠ 隐藏）。
//!
//! 隐藏期的事件投递（MIDP Canvas javadoc「Event Delivery」）：key/pointer/paint 回调只在
//! showNotify 之后、hideNotify 之前调用；hideNotify 之后直到下一次 showNotify 返回，一律不调用。
//! 壳据 [`Visibility::shown`] 门控键/鼠标投递（main.rs `Shell::route_key`）。

/// 可见性边沿：宿主应调用的 Canvas 回调。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notify {
    Show,
    Hide,
}

/// 两路窗口观测（遮挡事件 / 最小化轮询）合成的可见性 + `f_bool_01` latch。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Visibility {
    /// `f_bool_01`：已 showNotify、未 hideNotify。
    shown: bool,
    occluded: bool,
    minimized: bool,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::new()
    }
}

impl Visibility {
    /// startApp → showNotify 之后的状态（CMidlet.java:19）。
    pub fn new() -> Visibility {
        Visibility { shown: true, occluded: false, minimized: false }
    }

    /// Canvas 当前可见（= `f_bool_01`）：键/鼠标投递门控。
    pub fn shown(&self) -> bool {
        self.shown
    }

    /// `WindowEvent::Occluded` 的新值。
    pub fn set_occluded(&mut self, occluded: bool) -> Option<Notify> {
        self.occluded = occluded;
        self.settle()
    }

    /// `Window::is_minimized()` 轮询的新值。
    pub fn set_minimized(&mut self, minimized: bool) -> Option<Notify> {
        self.minimized = minimized;
        self.settle()
    }

    /// 合成可见性并过 latch：只在 shown 翻转时给出边沿（Java 两回调的 f_bool_01 门控）。
    fn settle(&mut self) -> Option<Notify> {
        let hidden = self.occluded || self.minimized;
        match (hidden, self.shown) {
            (true, true) => {
                self.shown = false;
                Some(Notify::Hide)
            }
            (false, false) => {
                self.shown = true;
                Some(Notify::Show)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 初值 = startApp 后（shown）；遮挡 → 一次 Hide，重复/叠加来源不再触发（latch）。
    #[test]
    fn starts_shown_and_hides_once() {
        let mut v = Visibility::new();
        assert!(v.shown());
        assert_eq!(v.set_occluded(false), None, "启动时的 Occluded(false) 无边沿");
        assert_eq!(v.set_occluded(true), Some(Notify::Hide));
        assert!(!v.shown());
        assert_eq!(v.set_occluded(true), None, "重复事件无边沿");
        assert_eq!(v.set_minimized(true), None, "已隐藏时叠加最小化无边沿（f_bool_01 门控）");
    }

    /// 两路都解除才 Show：遮挡解除但仍最小化 ⇒ 仍隐藏。
    #[test]
    fn shows_only_when_both_sources_clear() {
        let mut v = Visibility::new();
        assert_eq!(v.set_minimized(true), Some(Notify::Hide));
        assert_eq!(v.set_occluded(true), None);
        assert_eq!(v.set_occluded(false), None, "仍最小化");
        assert!(!v.shown());
        assert_eq!(v.set_minimized(false), Some(Notify::Show));
        assert!(v.shown());
    }

    /// latch 往复：Hide/Show 交替可重复；可见期的「解除」观测无边沿。
    #[test]
    fn latch_rearms_after_show() {
        let mut v = Visibility::new();
        assert_eq!(v.set_minimized(false), None);
        for _ in 0..3 {
            assert_eq!(v.set_minimized(true), Some(Notify::Hide));
            assert_eq!(v.set_minimized(false), Some(Notify::Show));
        }
    }
}
