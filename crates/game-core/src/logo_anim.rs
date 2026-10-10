//! SNOWFISH logo 动画子系统（deobf 语义名 runLogoAnimation/paintLogoAnimation/
//! registerLogoItem/modifyLogoItem/removeLogoItem——台账 N-0138..N-0142，机械名
//! runLogoAnimation..removeLogoItem，a.java:10223-10591）
//! 的 Rust 端口：注册表驱动的多元素动画层 + 绘制。
//!
//! # 结构（A 级证据：a.java 行内锚点）
//!
//! - **注册表**：`logoItemMeta[26][7]` 每项 `{x, y, kind, p3, p4, p5, slot}` +
//!   `logoItemImages[26]` 平行图列表，`logoItemCount` 为活跃数（a.java:2253、411）
//! - **registerLogoItem 登记**（a.java:10491）：kind 0=静止；1=缓动移向目标（p3 步数）；
//!   2=逐列展开（p3=总列数、p4=已展开数、p6=列宽）；3=字形槽（slot=p9 索引
//!   logoGlyphMetrics 度量表）；4=方形环绕步进
//! - **runLogoAnimation 推进**（a.java:10223）：每 tick 倒序遍历注册表按 kind 推进，
//!   然后按时间线 `var2` 登记/删除/改型（case 1..35）
//! - **paintLogoAnimation 绘制**（a.java:10461）：白底 → 遍历注册表（kind 3/4 走字形槽
//!   clip+sflogo#7；kind 2 走展开列 clip；其余直接画）
//! - **度量布局**（runLogoAnimation 首次，a.java:10226-10233）：三段高度 {87,18,9}
//!   （a.java:2248）→ y 锚表 logoAnchorY、x 居中 logoBarX；字形槽宽表
//!   logoGlyphMetrics 21 对 {x,w}（a.java:2250-2252）
//!
//! # 时序合同（差分实测）
//!
//! 每 tick **逻辑先 paint 后**：TICK n 的帧 = runLogoAnimation(0,n) 执行后
//! 的投影（paint.rs BootMachine 时序合同；硬锚 T17/T31/T32/T33 四点交叉
//! 验证，详见 game-oracle/tests/boot_replay.rs）。
//!
//! # 进度
//!
//! 时间线 case 1..10 已端口（T40 对拍覆盖）；case 11+ 待对拍扩产（`todo!` 显式挡路）。

use crate::render::{ArgbImage, SoftGraphics};

/// logoSegmentHeights（a.java:2248）：logo 三段高度 {主 logo, 间隔, 字条}。
pub const SEGMENT_HEIGHTS: [i32; 3] = [87, 18, 9];

/// logoGlyphMetrics（a.java:2250-2252）：21 对 {x偏移, 宽} 字形槽度量表。
pub const GLYPH_METRICS: [i8; 42] = [
    0, 5, 5, 5, 10, 4, 14, 7, 21, 4, 25, 3, 28, 3, 31, 4, 35, 3, 38, 4, 42, 4, 46, 4, 50, 4, 54,
    3, 57, 3, 60, 4, 64, 2, 66, 2, 68, 4, 72, 4, 76, 5,
];

/// 注册表容量（a.java:10496 `logoItemCount < 26`）。
const CAPACITY: usize = 26;

/// logo 动画层状态。
#[derive(Clone)]
pub struct LogoAnim {
    /// 注册表元数据（a.java:2253 `int[26][7]`，初始全 0）。测试探针需要快照/还原。
    registry: Vec<[i32; 7]>,
    /// 注册表平行图列表（a.java:10493，首次登记时分配；removeLogoItem(-1) 置回 None）。
    images: Vec<Option<ArgbImage>>,
    /// logoItemCount 活跃项数。
    pub count: i32,
    /// logoAnchorY：三段 y 锚（主 logo / 字条 / 底槽）。
    pub anchors_y: [i32; 3],
    /// logoBarX：底条 x 居中偏移。
    pub bar_x: i32,
    /// logoLayoutDone：布局已初始化（mode 0→21 清理时复位，a.java:3364）。
    initialized: bool,
}

impl Default for LogoAnim {
    fn default() -> Self {
        Self::new()
    }
}

impl LogoAnim {
    pub fn new() -> LogoAnim {
        LogoAnim {
            registry: vec![[0; 7]; CAPACITY],
            images: vec![None; CAPACITY],
            count: 0,
            anchors_y: [0; 3],
            bar_x: 0,
            initialized: false,
        }
    }

    /// `removeLogoItem(-1)`：全清（a.java:10587-10590）。
    pub fn clear_all(&mut self) {
        self.count = 0;
        self.images = vec![None; CAPACITY];
    }

    /// mode 0→21 清理路径的复位（a.java:3364 `logoLayoutDone = false`）：
    /// 复用同一 LogoAnim 实例时布局常量会重算（对抗 review R-5）。
    pub fn reset_layout_flag(&mut self) {
        self.initialized = false;
    }

    /// logoLayoutDone 布局初始化（a.java:10225-10234）。
    ///
    /// `logoGap = min(15, |320-Σh|>>2)`；`var4 = (320-Σh-2·logoGap)>>1`；
    /// anchors = [var4+87/2, var4+87+15, 190+18+15]；bar_x = (240-w_sflogo7)>>1。
    fn ensure_layout(&mut self, sflogo7_width: i32) {
        if self.initialized {
            return;
        }
        let sum: i32 = SEGMENT_HEIGHTS.iter().sum();
        let gap = 15.min((crate::layout::SCREEN_H - sum >> 2).abs()); // a.java:10229
        let base = crate::layout::SCREEN_H - sum - (gap << 1) >> 1; // a.java:10230
        self.anchors_y[0] = base + (SEGMENT_HEIGHTS[0] >> 1);
        self.anchors_y[1] = base + SEGMENT_HEIGHTS[0] + gap;
        self.anchors_y[2] = self.anchors_y[1] + SEGMENT_HEIGHTS[1] + gap;
        self.bar_x = crate::layout::SCREEN_W - sflogo7_width >> 1; // a.java:10233
        self.initialized = true;
    }

    /// `registerLogoItem`：登记一项（a.java:10491-10548）。
    ///
    /// 参数序与 Java 一致：`(img, x, y, kind, anchor, cols, p7, p8, p9, insert_at)`。
    /// kind 2 的 `cols` 把源图按列切分（宽/cols 入 [6]）；kind 3 的 `p9` 直入 [6]。
    #[allow(clippy::too_many_arguments)]
    pub fn register(
        &mut self,
        img: Option<ArgbImage>,
        x: i32,
        y: i32,
        kind: i32,
        anchor: i32,
        cols: i32,
        p7: i32,
        p8: i32,
        p9: i32,
        insert_at: i32,
    ) {
        if self.count >= CAPACITY as i32 {
            return;
        }
        let tail = self.count as usize;
        // var10 复用交换分支（a.java:10498-10506）：新元素占据 insert_at，
        // 原 insert_at 的（数组,图）顶到尾部；后续配置写回**新元素所在槽**（Java var11 重绑定）
        let (dst, slot_meta): (usize, [i32; 7]);
        if insert_at >= 0 && insert_at < self.count {
            let at = insert_at as usize;
            self.registry.swap(tail, at);
            self.images.swap(tail, at);
            self.images[at] = img.clone();
            dst = at;
            slot_meta = self.registry[at];
        } else {
            self.images[tail] = img.clone();
            dst = tail;
            slot_meta = self.registry[tail];
        }
        let mut meta = slot_meta;
        if kind == 3 {
            meta[6] = p9;
        } else {
            let mut w = img.as_ref().map(|i| i.width).unwrap_or(0);
            let h = img.as_ref().map(|i| i.height).unwrap_or(0);
            if kind == 2 {
                w /= cols;
                meta[6] = w;
            }
            // anchor 折算（a.java:10518-10530）：HCENTER→-全宽 / LEFT→-半宽；
            // VCENTER→-(h-1) / BOTTOM→-(h/2-1)
            let mut ax = 0;
            let mut ay = 0;
            if anchor & 1 != 0 {
                ax = -w;
            } else if anchor & 4 != 0 {
                ax = -(w >> 1);
            }
            if anchor & 2 != 0 {
                ay = -(h - 1);
            } else if anchor & 8 != 0 {
                ay = -((h >> 1) - 1);
            }
            let (x, y) = (x + ax, y + ay);
            let (p7, p8) = if kind == 1 { (p7 + ax, p8 + ay) } else { (p7, p8) };
            meta[0] = x;
            meta[1] = y;
            meta[2] = kind;
            meta[3] = cols;
            meta[4] = p7;
            meta[5] = p8;
            self.registry[dst] = meta;
            self.count += 1;
            return;
        }
        meta[0] = x;
        meta[1] = y;
        meta[2] = kind;
        meta[3] = cols;
        meta[4] = p7;
        meta[5] = p8;
        self.registry[dst] = meta;
        self.count += 1;
    }

    /// `modifyLogoItem`：改第 `idx` 项的 kind/参数（a.java:10550-10574）。
    /// `anchor != 0` 且 kind==1 时把 anchor 折算进 (p5,p6)。
    pub fn modify(&mut self, idx: i32, kind: i32, anchor: i32, p4: i32, p5: i32, p6: i32) {
        if idx >= self.count {
            return;
        }
        let meta = &mut self.registry[idx as usize];
        let (mut p5, mut p6) = (p5, p6);
        if kind == 1 {
            let img = &self.images[idx as usize];
            if let Some(im) = img {
                let (w, h) = (im.width, im.height);
                if anchor & 1 != 0 {
                    p5 -= w;
                } else if anchor & 4 != 0 {
                    p5 -= w >> 1;
                }
                if anchor & 2 != 0 {
                    p6 -= h - 1;
                } else if anchor & 8 != 0 {
                    p6 -= (h >> 1) - 1;
                }
            }
        }
        meta[2] = kind;
        meta[3] = p4;
        meta[4] = p5;
        meta[5] = p6;
    }

    /// `removeLogoItem`：删除第 `idx` 项（swap-到尾）或 `idx<0` 全清（a.java:10576-10591）。
    pub fn remove(&mut self, idx: i32) {
        if idx >= 0 {
            if idx < self.count {
                let last = (self.count - 1) as usize;
                self.registry.swap(idx as usize, last);
                let img = self.images[idx as usize].take();
                self.images[idx as usize] = self.images[last].take();
                self.images[last] = img;
                self.count -= 1;
            }
        } else {
            self.clear_all();
        }
    }

    /// `runLogoAnimation(0, var2)`：推进 + 时间线（a.java:10223-10459）。
    ///
    /// `sflogo` 为容器 8 张子图（a.java:10362 等引用 `[0][k]`）；
    /// `timeline` 为 bootPhaseCounter（1..35，a.java:3360-3361）。
    pub fn tick(&mut self, sflogo: &[ArgbImage], timeline: i32) {
        self.tick_variant(sflogo, timeline, false);
    }

    /// `runLogoAnimation(1, var2)`：帮助退出动画（mode 22，a.java:4246-4247，
    /// 时间线 1..70）。var1=1 分支（a.java:10316-10364）。
    pub fn tick_help(&mut self, sflogo: &[ArgbImage], timeline: i32) {
        self.tick_variant(sflogo, timeline, true);
    }

    /// var1 维度统一入口（对抗 review R-3 补齐：原实现只有 var1==0）。
    fn tick_variant(&mut self, sflogo: &[ArgbImage], timeline: i32, help: bool) {
        self.ensure_layout(sflogo[7].width);
        // 推进循环（倒序，a.java:10239-10308）
        for i in (0..self.count as usize).rev() {
            let m = &mut self.registry[i];
            match m[2] {
                1 => {
                    if m[3] > 0 {
                        m[0] += (m[4] - m[0]) / m[3];
                        m[1] += (m[5] - m[1]) / m[3];
                        m[3] -= 1;
                    }
                }
                2 => {
                    if m[4] > 1 {
                        m[4] -= 1;
                    }
                }
                3 => {
                    if m[3] > 0 {
                        let dx = (m[4] - m[0]) / m[3];
                        let dy = (m[5] - m[1]) / m[3];
                        m[0] += dx;
                        m[1] += dy;
                        m[3] -= 1;
                        if m[3] == 0 {
                            m[4] = if dx > 0 { 8 } else if dx < 0 { -8 } else { 0 };
                            m[5] = if dy > 0 { 8 } else if dy < 0 { -8 } else { 0 };
                        }
                    } else if m[4] == 0 {
                        match m[3] {
                            -3 | 0 => {
                                m[0] += m[4];
                                m[1] += m[5];
                            }
                            -2 => {
                                m[4] >>= 1;
                                m[5] >>= 1;
                                m[0] -= m[4];
                                m[1] -= m[5];
                            }
                            -1 => {
                                m[0] -= m[4];
                                m[1] -= m[5];
                            }
                            _ => {}
                        }
                        m[3] -= 1;
                    }
                }
                4 => {
                    if m[3] > 0 {
                        match m[5] {
                            0 => {
                                m[0] -= m[4];
                                m[1] += m[4];
                            }
                            1 => {
                                m[0] -= m[4];
                                m[1] -= m[4];
                            }
                            2 => {
                                m[0] += m[4];
                                m[1] -= m[4];
                            }
                            3 => {
                                m[0] += m[4];
                                m[1] += m[4];
                            }
                            _ => {}
                        }
                        m[5] += 1;
                        if m[5] >= 4 {
                            m[5] = 0;
                            m[3] -= 1;
                        }
                    }
                }
                _ => {}
            }
        }

        // var1!=0 且 t>=18 的帮助分支（a.java:10316-10364）；t<18 落到下方
        // 启动时间线重放（else 分支与 var1==0 共享——menu-sweep T1762-1779 实证）
        if help && timeline >= 18 {
            let ay = self.anchors_y;
            let bx = self.bar_x;
            if (18..31).contains(&timeline) {
                let idx = timeline - 10;
                let x = bx + GLYPH_METRICS[idx as usize * 2] as i32;
                self.register(None, x, crate::layout::SCREEN_H, 3, 0, 4, x, ay[2], idx, -1);
            }
            match timeline {
                50 => self.modify(2, 2, 4, 4, 4, 1),
                51 => self.modify(3, 2, 4, 4, 4, 1),
                52 => self.modify(4, 2, 4, 4, 4, 1),
                58 => {
                    self.modify(0, 1, 0, 1, 1000, 1000);
                    self.modify(1, 2, 4, 4, 4, 1);
                }
                63 => self.modify(15, 3, 0, 4, -10, ay[2]),
                _ => {}
            }
            if timeline > 52 && timeline < 63 {
                let v = timeline - 52;
                self.modify(v + 4, 3, 0, 4, -10, ay[2]);
                self.modify(26 - v, 3, 0, 4, 240, ay[2]);
            }
            return;
        }
        // 时间线 switch（a.java:10326-10402；case 11+ 部分待对拍扩产）
        let ay = self.anchors_y;
        let bx = self.bar_x;
        match timeline {
            1 => {
                self.register(Some(sflogo[1].clone()), 120, 0, 1, 12, 4, 120, ay[0], 0, -1);
            }
            5 => {
                self.remove(0);
                self.register(Some(sflogo[0].clone()), 120, ay[0], 2, 12, 4, 4, 0, 0, -1);
            }
            8 => {
                self.register(Some(sflogo[5].clone()), 120, ay[0] + 2, 0, 12, 0, 0, 0, 0, 0);
                self.register(Some(sflogo[2].clone()), 90, ay[1], 2, 4, 4, 4, 0, 0, -1);
                self.register(Some(sflogo[3].clone()), 120, ay[1], 2, 4, 4, 4, 0, 0, -1);
                self.register(Some(sflogo[4].clone()), 150, ay[1], 2, 4, 4, 4, 0, 0, -1);
                self.register(Some(sflogo[6].clone()), 15, ay[1], 1, 12, 3, bx, ay[2], 0, -1); // logoBarX（a.java:10384）
            }
            10 => {
                self.remove(5);
                self.register(None, bx, ay[2], 3, 0, 0, 0, 0, 0, -1);
                self.modify(5, 4, 0, 1, 2, 0);
            }
            14 => {
                self.modify(5, 3, 0, 3, bx - 8, ay[2]);
                self.modify(6, 3, 0, 3, bx + GLYPH_METRICS[2] as i32 - 4, ay[2]);
            }
            15 => {
                self.modify(7, 3, 0, 2, bx + GLYPH_METRICS[4] as i32 - 2, ay[2]);
            }
            17 => {
                self.modify(5, 3, 0, 3, bx, ay[2]);
                self.modify(6, 3, 0, 3, bx + GLYPH_METRICS[2] as i32, ay[2]);
                self.modify(7, 3, 0, 2, bx + GLYPH_METRICS[4] as i32, ay[2]);
            }
            // Java switch 的 default 分支（a.java:10363-10373）：无操作。
            // timeline 全域 1..=35（a.java:3360）：25..=31 由下方公共段处理
            2..=4 | 6 | 7 | 9 | 11 | 12 | 13 | 16 | 18..=24 | 25..=31 | 32..=35 => {}
            _ => unreachable!("timeline 超出 1..=35（a.java:3360 计数域）: got {timeline}"),
        }

        // switch 后的公共注册段（a.java:10404-10457，命中即 return）
        if (10..13).contains(&timeline) {
            // a.java:10396：x 参数恒 240（屏外起点），y 参数 = ay[2]；目标 x 在 [4]
            let idx = timeline - 9;
            let target = bx + GLYPH_METRICS[idx as usize * 2] as i32;
            self.register(None, crate::layout::SCREEN_W, ay[2], 3, 0, 4, target, ay[2], idx, -1); // x=240 屏外起点
            return;
        }
        if (14..=17).contains(&timeline) {
            let idx = timeline - 10;
            let x = bx + GLYPH_METRICS[idx as usize * 2] as i32;
            self.register(None, x, crate::layout::SCREEN_H, 3, 0, 4, x, ay[2], idx, -1); // y=320 屏外起点
            return;
        }
        if (25..=31).contains(&timeline) {
            let mut idx = (timeline - 25) * 2 + 8;
            let x = bx + GLYPH_METRICS[idx as usize * 2] as i32;
            self.register(None, x, ay[2], 3, 0, 0, x, ay[2], idx, -1);
            if timeline < 31 {
                idx += 1;
                let x2 = bx + GLYPH_METRICS[idx as usize * 2] as i32;
                self.register(None, x2, ay[2], 3, 0, 0, x2, ay[2], idx, -1);
            }
        }
    }

    /// `paintLogoAnimation`：绘制（a.java:10461-10489）。白底后按注册表序绘制；
    /// kind 3/4 = 字形槽 clip + sflogo#7；kind 2 = 展开列 clip；其余直接画。
    /// 每项绘制后 `setClip(0,0,240,320)` 复位。
    pub fn paint(&self, g: &mut SoftGraphics<'_>, sflogo7: &ArgbImage) {
        g.set_color(crate::layout::WHITE);
        g.fill_rect(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
        for i in 0..self.count as usize {
            let m = self.registry[i];
            let mut x = m[0];
            let y = m[1];
            if m[2] == 3 || m[2] == 4 {
                x -= GLYPH_METRICS[m[6] as usize * 2] as i32;
                g.set_clip(
                    x + GLYPH_METRICS[m[6] as usize * 2] as i32,
                    y,
                    GLYPH_METRICS[m[6] as usize * 2 + 1] as i32,
                    crate::layout::SCREEN_H,
                );
                g.draw_image(sflogo7, x, y, 0);
            } else if m[2] == 2 {
                let mut progress = m[3] - m[4];
                if m[5] == 1 {
                    progress = m[3] - progress - 2;
                }
                x -= m[6] * progress;
                g.set_clip(x + m[6] * progress, y, m[6], crate::layout::SCREEN_H);
                if let Some(img) = &self.images[i] {
                    g.draw_image(img, x, y, 0);
                }
            } else if let Some(img) = &self.images[i] {
                g.draw_image(img, x, y, 0);
            }
            g.set_clip(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::ArgbImage;
    use game_platform::hash::sha256_hex;
    use std::path::PathBuf;

    fn repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// 解码 assets/raw/sflogo 全 8 张（张数常量 evidence：a.java:465-466）。
    fn load_sflogo() -> Vec<ArgbImage> {
        let data = std::fs::read(repo().join("assets/raw/sflogo")).unwrap();
        let packed = game_data::PackedPng::parse(&data, 8).unwrap();
        packed
            .images
            .iter()
            .map(|sub| ArgbImage::from_decoded_png(game_data::decode_png(sub.bytes).unwrap()))
            .collect()
    }

    /// trace A-boot-menu TICK 40 的 20-op 序列（var2=9 的 paint 先于本拍推进）。
    /// 证据：reference/oracle/_out/A-boot-menu/trace.txt TICK 0024..0040 区段。
    #[test]
    fn paint_logo_at_timeline8_matches_trace_t40_ops() {
        let sflogo = load_sflogo();
        let mut anim = LogoAnim::new();
        for t in 1..=9 {
            anim.tick(&sflogo, t);
        }
        // var2=9 推进后绘制的是"推进后"状态，而 T40 帧是 var2=9 推进**前**——
        // 按时序合同（paint 先于逻辑），重放到 8 再画：
        let mut anim = LogoAnim::new();
        for t in 1..=8 {
            anim.tick(&sflogo, t);
        }

        let mut screen = ArgbImage::create(crate::layout::SCREEN_W, crate::layout::SCREEN_H);
        let mut g = crate::render::SoftGraphics::new(&mut screen);
        g.set_clip(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H); // serviceRepaints 入口（Canvas.java:71）
        g.set_font(Some(crate::paint::paint_font())); // paint 入口 a.java:2336
        anim.paint(&mut g, &sflogo[7]);

        let expect: Vec<String> = [
            "setClip(0,0,240,320)",
            "setFont(size=8)",
            "setColor(#ffffff)",
            "fillRect(0,0,240,320,#ffffff)",
            "drawImage(89x87,76,91,0)",
            "setClip(0,0,240,320)",
            "setClip(92,107,57,320)",
            "drawImage(228x51,-79,107,0)",
            "setClip(0,0,240,320)",
            "setClip(82,190,17,320)",
            "drawImage(68x18,82,190,0)",
            "setClip(0,0,240,320)",
            "setClip(112,190,17,320)",
            "drawImage(68x18,112,190,0)",
            "setClip(0,0,240,320)",
            "setClip(142,190,17,320)",
            "drawImage(68x18,142,190,0)",
            "setClip(0,0,240,320)",
            "drawImage(24x38,3,172,0)",
            "setClip(0,0,240,320)",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(g.ops.len(), expect.len(), "op 数不符: {:#?}", g.ops);
        for (i, (got, want)) in g.ops.iter().zip(&expect).enumerate() {
            assert_eq!(got, want, "op #{i}");
        }
    }


    /// TICK 40 帧哈希对拍：完整像素逐字节一致（TICK 0024 trace FRAME sha，截 32 hex）。
    #[test]
    fn paint_logo_at_timeline8_matches_trace_t40_frame() {
        let sflogo = load_sflogo();
        let mut anim = LogoAnim::new();
        for t in 1..=8 {
            anim.tick(&sflogo, t);
        }
        let mut screen = ArgbImage::create(crate::layout::SCREEN_W, crate::layout::SCREEN_H);
        {
            let mut g = crate::render::SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
            g.set_font(Some(crate::paint::paint_font()));
            anim.paint(&mut g, &sflogo[7]);
        }
        assert_eq!(
            &sha256_hex(&screen.hash_stream())[..32],
            "ee740c7118c115a6a546e97c67766bf5",
            "logo 帧（TICK 40）必须与 Java 逐字节一致"
        );
    }
}
