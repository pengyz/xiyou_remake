//! 窗口呈现与节拍（纯计算，可单测）：整数倍缩放 + 信箱、鼠标坐标反映射、
//! 固定步长节拍器（失焦冻结、无追帧）。

use std::time::{Duration, Instant};

/// 240×320 画面在窗口里的放置：整数倍 `scale`、左上偏移 `(ox, oy)`（信箱居中）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Viewport {
    pub win_w: u32,
    pub win_h: u32,
    pub scale: u32,
    pub ox: u32,
    pub oy: u32,
}

impl Viewport {
    /// 取能完整放下画面的最大整数倍（至少 1；窗口比画面还小时画面左上对齐裁切）。
    pub fn fit(win_w: u32, win_h: u32, src_w: u32, src_h: u32) -> Viewport {
        let scale = (win_w / src_w).min(win_h / src_h).max(1);
        let ox = win_w.saturating_sub(src_w * scale) / 2;
        let oy = win_h.saturating_sub(src_h * scale) / 2;
        Viewport { win_w, win_h, scale, ox, oy }
    }

    /// 窗口物理像素 → 画面坐标；落在信箱/画面外 ⇒ None。
    pub fn to_screen(&self, x: f64, y: f64, src_w: u32, src_h: u32) -> Option<(i32, i32)> {
        let (fx, fy) = (x - self.ox as f64, y - self.oy as f64);
        if fx < 0.0 || fy < 0.0 {
            return None;
        }
        let (sx, sy) = ((fx / self.scale as f64) as u32, (fy / self.scale as f64) as u32);
        (sx < src_w && sy < src_h).then_some((sx as i32, sy as i32))
    }
}

/// 把 `src`（ARGB，`src_w`×`src_h`）按 `vp` 写入窗口缓冲 `dst`（0RGB，`vp.win_w` 行宽）；
/// 信箱填黑。
pub fn blit(dst: &mut [u32], vp: &Viewport, src: &[u32], src_w: u32, src_h: u32) {
    let (ww, wh) = (vp.win_w as usize, vp.win_h as usize);
    debug_assert_eq!(dst.len(), ww * wh);
    let s = vp.scale as usize;
    let (ox, oy) = (vp.ox as usize, vp.oy as usize);
    let (sw, sh) = (src_w as usize, src_h as usize);
    for (wy, row) in dst.chunks_exact_mut(ww).enumerate() {
        if wy < oy || wy >= oy + sh * s {
            row.fill(0);
            continue;
        }
        let sy = (wy - oy) / s;
        let src_row = &src[sy * sw..sy * sw + sw];
        row[..ox.min(ww)].fill(0);
        let mut x = ox;
        for &p in src_row {
            if x >= ww {
                break;
            }
            let end = (x + s).min(ww);
            row[x..end].fill(0xFF00_0000 | (p & 0x00FF_FFFF));
            x = end;
        }
        if x < ww {
            row[x..].fill(0);
        }
    }
}

/// 固定步长节拍器。语义取 Java run 循环（a.java:3249-3250 / 4279-4284）：每帧起点
/// `t0`、截止 `t0 + f_int_02`，逻辑+重绘后忙等到截止——**落后不追帧**（一次唤醒只跑一拍）。
/// 失焦冻结：不出拍；回前台从「现在 + 一拍」重新起算。
pub struct Pacer {
    next: Instant,
    paused: bool,
}

impl Pacer {
    pub fn new(now: Instant, interval: Duration) -> Pacer {
        Pacer { next: now + interval, paused: false }
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    /// 是否该跑一拍。
    pub fn due(&self, now: Instant) -> bool {
        !self.paused && now >= self.next
    }

    /// 跑完一拍后排下一拍（`interval` 取逻辑后的帧间隔 f_int_02）。落后超过一拍 ⇒
    /// 从现在重新起算（丢弃积压，Java 忙等循环同样不补帧）。
    pub fn after_tick(&mut self, now: Instant, interval: Duration) {
        self.next += interval;
        if self.next <= now {
            self.next = now + interval;
        }
    }

    /// 失焦/回前台。返回状态是否改变。
    pub fn set_paused(&mut self, paused: bool, now: Instant, interval: Duration) -> bool {
        if paused == self.paused {
            return false;
        }
        self.paused = paused;
        if !paused {
            self.next = now + interval;
        }
        true
    }

    /// 事件循环下一次唤醒时刻（冻结中 ⇒ None = 一直睡到事件）。
    pub fn wake_at(&self) -> Option<Instant> {
        (!self.paused).then_some(self.next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_integer_scale_and_letterbox() {
        assert_eq!(Viewport::fit(480, 640, 240, 320), Viewport { win_w: 480, win_h: 640, scale: 2, ox: 0, oy: 0 });
        // 宽屏全屏 1920×1080：3 倍（720×960），左右各 600、上下各 60
        assert_eq!(Viewport::fit(1920, 1080, 240, 320), Viewport { win_w: 1920, win_h: 1080, scale: 3, ox: 600, oy: 60 });
        // 非整数比：取能放下的最大整数倍，余量居中
        assert_eq!(Viewport::fit(500, 700, 240, 320).scale, 2);
        assert_eq!(Viewport::fit(500, 700, 240, 320).ox, 10);
        // 比画面还小：倍数下限 1、偏移 0
        let v = Viewport::fit(100, 100, 240, 320);
        assert_eq!((v.scale, v.ox, v.oy), (1, 0, 0));
    }

    #[test]
    fn mouse_maps_through_scale_and_letterbox() {
        let v = Viewport::fit(1920, 1080, 240, 320);
        assert_eq!(v.to_screen(600.0, 60.0, 240, 320), Some((0, 0)));
        assert_eq!(v.to_screen(600.0 + 719.0, 60.0 + 959.0, 240, 320), Some((239, 319)));
        assert_eq!(v.to_screen(599.0, 500.0, 240, 320), None, "左信箱");
        assert_eq!(v.to_screen(1320.0, 500.0, 240, 320), None, "右信箱");
        assert_eq!(v.to_screen(900.0, 1020.0, 240, 320), None, "下信箱");
    }

    #[test]
    fn blit_scales_pixels_and_blacks_out_bars() {
        // 2×1 源、窗口 7×4：scale = min(7/2, 4/1) = 3 → 画面 6×3，ox=0（余 1 列右侧黑）、oy=0（余 1 行黑）
        let src = [0xAA11_2233u32, 0x0044_5566];
        let vp = Viewport::fit(7, 4, 2, 1);
        assert_eq!((vp.scale, vp.ox, vp.oy), (3, 0, 0));
        let mut dst = vec![0xDEAD_BEEFu32; 7 * 4];
        blit(&mut dst, &vp, &src, 2, 1);
        let a = 0xFF11_2233;
        let b = 0xFF44_5566;
        for y in 0..3 {
            assert_eq!(&dst[y * 7..y * 7 + 7], &[a, a, a, b, b, b, 0], "行 {y}");
        }
        assert!(dst[21..].iter().all(|&p| p == 0), "下信箱");
    }

    #[test]
    fn pacer_fixed_step_no_catch_up_and_focus_freeze() {
        let t0 = Instant::now();
        let iv = Duration::from_millis(75);
        let mut p = Pacer::new(t0, iv);
        assert!(!p.due(t0 + Duration::from_millis(74)));
        assert!(p.due(t0 + iv));
        // 准点：下一拍 = 上一拍 + 75（不随唤醒抖动漂移）
        p.after_tick(t0 + Duration::from_millis(76), iv);
        assert_eq!(p.wake_at(), Some(t0 + 2 * iv));
        // 卡顿 1 s：只跑一拍，下一拍从现在 +75 起算（不追 13 帧）
        let late = t0 + Duration::from_millis(1150);
        assert!(p.due(late));
        p.after_tick(late, iv);
        assert_eq!(p.wake_at(), Some(late + iv));
        assert!(!p.due(late + Duration::from_millis(10)));
        // 失焦冻结：永不出拍、不定时唤醒
        assert!(p.set_paused(true, late, iv));
        assert!(!p.set_paused(true, late, iv));
        assert!(!p.due(late + Duration::from_secs(60)));
        assert_eq!(p.wake_at(), None);
        // 回前台：从现在 +一拍，无积压
        let back = late + Duration::from_secs(60);
        assert!(p.set_paused(false, back, iv));
        assert_eq!(p.wake_at(), Some(back + iv));
        assert!(!p.due(back));
    }
}
