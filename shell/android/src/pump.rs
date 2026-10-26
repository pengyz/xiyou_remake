//! 主循环调度（宿主可测纯逻辑）：事件泵 + 逻辑拍节拍器。
//!
//! android_main 主循环经 [`Pump`] 抽象 ALooper（生产实现在 native.rs cfg(android)），
//! 测试用虚拟时钟假 looper 驱动同一 [`pump_once`]。

/// looper 抽象：`poll(timeout)` 等待至多 timeout_ms；有事件源就绪=处理之并返回 true，
/// 超时=false。
pub trait Pump {
    fn now_ms(&self) -> u64;
    fn poll(&mut self, timeout_ms: i32) -> bool;
}

/// 逻辑拍节拍器（J2ME run 循环语义：拍起点+间隔=下一拍；卡顿后只出一拍并重锚，
/// **不追帧连发**——与旧 `last_logic.elapsed() >= interval` 同口径）。
#[derive(Debug, Clone)]
pub struct FramePacer {
    next_ms: u64,
}

impl FramePacer {
    pub fn new(start_ms: u64) -> FramePacer {
        FramePacer { next_ms: start_ms }
    }

    /// 到期判定：now ≥ next ⇒ 出拍并重锚 next = now + interval。
    pub fn due(&mut self, now_ms: u64, interval_ms: u64) -> bool {
        if now_ms >= self.next_ms {
            self.next_ms = now_ms + interval_ms;
            true
        } else {
            false
        }
    }

    /// 距下一拍的等待毫秒（looper 阻塞超时；事件到达提前醒）。
    pub fn wait_ms(&self, now_ms: u64) -> i32 {
        self.next_ms.saturating_sub(now_ms).min(i32::MAX as u64) as i32
    }
}

/// 一轮泵：**非阻塞**排空全部待处理事件（poll(0)）→ 到期则出拍（true=调用方执行
/// tick+paint+present）。等待交调用方 `poll(pacer.wait_ms(now))`（事件到达即醒）。
///
/// 【2026-10-26 实机根因②】旧循环 `while poll(16) >= 0` 在 120Hz Move 流下恒有事件
/// ⇒ 不退出 ⇒ 按住期间 tick/paint 整段饿死（设备 DIAG：1000ms 长按断档 1053ms）。
pub fn pump_once<P: Pump>(p: &mut P, pacer: &mut FramePacer, interval_ms: u64) -> bool {
    while p.poll(0) {}
    pacer.due(p.now_ms(), interval_ms)
}
