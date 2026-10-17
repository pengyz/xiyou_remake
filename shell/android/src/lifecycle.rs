//! Android Activity 生命周期 → MIDP Canvas hideNotify/showNotify 映射（纯逻辑，宿主可测）。
//!
//! # 真值锚
//!
//! ```text
//! showNotify()  a.java:4495-4500 / @0-17   if (!f_bool_01) { f_bool_01 = true; }
//! hideNotify()  a.java:4502-4513 / @0-43   if (f_bool_01) {
//!                                             if (gameMode == 3) { gameMode = 4; m_000(); }
//!                                             closeAudioPlayer(); anyKeyPlayed = true;
//!                                             f_bool_01 = false; }
//! ```
//!
//! `f_bool_01`（a.java:32）是 show/hide 去重 latch（读写点仅 a.java:4496/4497/4503/4511），
//! 由宿主持有不可区分（shell/win `src/lifecycle.rs` 模块注释同论证）。本模块
//! [`Visibility`] 即它：`new()` = startApp 后（`CMidlet.startApp` 显式 showNotify，
//! CMidlet.java:19 ⇒ 运行期首拍起恒 true）。
//!
//! # Android 回调 → MIDP 语义映射（近似边界如实登记，对抗审查 F-7）
//!
//! | Android 场景 | 回调 | 映射 | 近似性 |
//! |---|---|---|---|
//! | 退后台/锁屏/来电/分屏隐藏 | onPause | ⇒ Hide（hideNotify 全链） | ≈完全遮挡，吻合 |
//! | 返回前台/解锁/通话结束 | onResume | ⇒ Show（只清 latch，BGM 等后续首键续播） | 吻合 |
//! | 分屏/自由窗口「可见但失焦」 | onTopResumedActivityChanged(false) | **不发** Notify | 同 Windows 失焦不映射（用户决策 2026-10-15） |
//! | 画中画（可见非遮挡） | onPause(in_pip=true) | **不发** Hide | 按「完全遮挡才 hide」锚（S1 裁定） |
//!
//! **音频焦点不处理**（对抗审查 F-6 砍除）：原版 J2ME 无音频焦点概念（a.java 音频入口
//! 仅 m_139/closeAudioPlayer，:10158-10215）；焦点被抢期间的发声属平台近似边界，
//! 不得引入原版没有的可听行为（AGENTS §1）。如日后需要 opt-in，须 decision 登记 +
//! 用户确认（另立任务）。
//!
//! hide 的**消费**（mode3→4 弹菜单 + closeAudioPlayer + anyKeyPlayed=true）全在
//! game-core（`GameScene::hide_notify`），壳只决定何时调用——本模块只产边沿。

/// 宿主应执行的 Canvas 回调（f_bool_01 latch 只在 shown 翻转时给边沿）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notify {
    Show,
    Hide,
}

/// 可见性 latch（= `f_bool_01`）+ Android 观测源合成。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Visibility {
    /// `f_bool_01`：已 showNotify、未 hideNotify。
    shown: bool,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::new()
    }
}

impl Visibility {
    /// startApp → showNotify 之后（CMidlet.java:19）。
    pub fn new() -> Visibility {
        Visibility { shown: true }
    }

    /// Canvas 当前可见（= `f_bool_01`）：键/触控投递门控（`input::InputRouter`）。
    pub fn shown(&self) -> bool {
        self.shown
    }

    /// onPause（in_pip = `isInPictureInPictureMode()`）。
    /// 画中画 = 可见非遮挡（「完全遮挡才 hide」锚，S1）⇒ 不发 Hide、状态不动。
    pub fn on_pause(&mut self, in_pip: bool) -> Option<Notify> {
        if in_pip {
            return None;
        }
        if self.shown {
            self.shown = false;
            Some(Notify::Hide)
        } else {
            None // latch：重复 pause（onStop 等）不重发
        }
    }

    /// onResume：只清 latch（showNotify 无游戏态副作用）；BGM 不在此恢复——
    /// 等其后首个键（keyPressed 续播边沿，a.java:4665-4674，game-core 内执行）。
    pub fn on_resume(&mut self) -> Option<Notify> {
        if self.shown {
            return None;
        }
        self.shown = true;
        Some(Notify::Show)
    }

    /// onTopResumedActivityChanged：可见失焦/回到顶 ⇒ 恒不发 Notify
    /// （失焦 ≠ 隐藏，用户决策 2026-10-15；与 Windows「失焦不映射」同口径）。
    pub fn on_top_resumed_changed(&mut self, _top: bool) -> Option<Notify> {
        None
    }
}
