//! 参考版像素模型的 Rust 复刻：`javax.microedition.lcdui` shim（Graphics/Font/Image）
//! 的逐字节确定性软件光栅器。
//!
//! # 为什么逐行复刻 shim 而不是"实现 MIDP 规范"
//!
//! 差分裁判是 trace 里的 `FRAME sha=`（TickHooks.afterPaint：`getRGB` 出的 240×320
//! ARGB 流按每像素大端 4 字节喂 SHA-256）。要使 Rust 侧产出相同 sha，像素模型必须
//! 与 Java shim **逐位一致**——包括 shim 的自选自由度（确定性 hash 字形、字体度量
//! 常量、`translate` 只移动 clip、arc 判定的 double 阈值）。这些自由度两端同源即
//! 不影响等价性判定（设备级视觉保真是 L4/P4 的事）。
//!
//! # 证据（A 级）
//!
//! - 混合/图元/字形：`reference/shim/src/javax/microedition/lcdui/Graphics.java`
//! - 字体度量：`reference/shim/src/javax/microedition/lcdui/Font.java`
//! - 帧哈希：`reference/oracle/src/oracle/host/TickHooks.java:48-67`
//!
//! # double 一致性说明
//!
//! `fill_triangle`/`fill_arc` 内的乘除与 ceil/floor 是 IEEE-754 精确操作，Java/Rust
//! 逐位一致。`atan2` 有 libm ULP 风险，但其结果只喂给 `in_arc`，而游戏唯一调用
//! `fillArc(x,y,32,32,0,360)`（a.java:6635）走 `arcDeg >= 360 ⇒ true` 短路，返回值
//! 被丢弃，ULP 差异不影响任何像素。
//!
//! # Java int 位语义
//!
//! 画布像素存 `u32`（0xAARRGGBB 位模式，Java int 的负值只是同一模式的另一解释）；
//! 所有中间运算用 `i32`/`u32` 按位复制 Java 行为。混合除法的操作数恒非负，安全。

/// anchor 常量（`Graphics.java:31-37`）。
pub mod anchor {
    pub const HCENTER: i32 = 1;
    pub const VCENTER: i32 = 2;
    pub const LEFT: i32 = 4;
    pub const RIGHT: i32 = 8;
    pub const TOP: i32 = 16;
    pub const BOTTOM: i32 = 32;
    pub const BASELINE: i32 = 64;
}

/// Nokia DirectGraphics 变换码（`com/nokia/mid/ui/DirectGraphics.java:11-18`）。
pub mod transform {
    pub const NONE: i32 = 0;
    pub const MIRROR_ROT180: i32 = 1;
    pub const MIRROR: i32 = 2;
    pub const ROT180: i32 = 3;
    pub const MIRROR_ROT270: i32 = 4;
    pub const ROT90: i32 = 5;
    pub const ROT270: i32 = 6;
    pub const MIRROR_ROT90: i32 = 7;
}

/// Font 尺寸/风格常量（`Font.java:16-26`）。
pub mod font_size {
    pub const SMALL: i32 = 8;
    pub const MEDIUM: i32 = 16;
    pub const LARGE: i32 = 32;
}
pub mod font_style {
    pub const PLAIN: i32 = 0;
    pub const BOLD: i32 = 1;
}

/// 字体度量（shim 自定义常量，`Font.java:47-62`）。
///
/// 与 shim 同源：SMALL=6/10/8，MEDIUM=8/14/11，LARGE=12/20/16，BOLD 步进 +1。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoftFont {
    /// ASCII 字符步进宽（像素）。
    pub ascii_width: i32,
    /// 行高（像素）。
    pub height: i32,
    /// 基线距顶部（像素）。
    pub baseline: i32,
    pub style: i32,
}

impl SoftFont {
    /// `Font.getFont(face, style, size)`。face 在 shim 中不影响度量（系统/等宽同度量）。
    pub fn get_font(_face: i32, style: i32, size: i32) -> SoftFont {
        let (mut aw, h, bl) = match size {
            font_size::SMALL => (6, 10, 8),
            font_size::LARGE => (12, 20, 16),
            _ => (8, 14, 11),
        };
        if style & font_style::BOLD != 0 {
            aw += 1;
        }
        SoftFont { ascii_width: aw, height: h, baseline: bl, style }
    }

    /// `Font.isWide`：CJK/全角区域按 2 倍步进（`Font.java:99-106`）。
    fn is_wide(ch: u16) -> bool {
        (0x2E80..=0x9FFF).contains(&ch)
            || (0xA960..=0xA97F).contains(&ch)
            || (0xAC00..=0xD7FF).contains(&ch)
            || (0xF900..=0xFAFF).contains(&ch)
            || (0xFF00..=0xFF60).contains(&ch)
            || (0xFFE0..=0xFFE6).contains(&ch)
    }

    /// `charWidth`。
    pub fn char_width(&self, ch: u16) -> i32 {
        if Self::is_wide(ch) { self.ascii_width * 2 } else { self.ascii_width }
    }

    /// `substringWidth(str, offset, len)`：逐字符累加（`Font.java:87-96`）。
    pub fn substring_width(&self, s: &[u16], offset: usize, len: usize) -> i32 {
        let mut w = 0;
        for i in offset..offset + len {
            w += self.char_width(s[i]);
        }
        w
    }

    /// `stringWidth`。
    pub fn string_width(&self, s: &[u16]) -> i32 {
        self.substring_width(s, 0, s.len())
    }
}

/// 画布/离屏图：width×height 的非预乘 ARGB 位模式（0xAARRGGBB）。
///
/// 对应 shim `Image`（`int[] argb`）。初始全 0（Java `new int[..]` 语义）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgbImage {
    pub width: i32,
    pub height: i32,
    pub argb: Vec<u32>,
}

impl ArgbImage {
    /// `Image.createImage(w, h)`。
    pub fn create(width: i32, height: i32) -> ArgbImage {
        ArgbImage { width, height, argb: vec![0; (width * height) as usize] }
    }

    /// 资源解码产物 → 可绘制图（位模式直通，无转换）。
    pub fn from_decoded_png(png: game_data::DecodedPng) -> ArgbImage {
        ArgbImage { width: png.width, height: png.height, argb: png.argb }
    }

    /// `Image.createRGBImage(rgb, w, h, processAlpha)`（`Image.java:59-66`）。
    pub fn from_rgb(rgb: &[u32], width: i32, height: i32, process_alpha: bool) -> ArgbImage {
        let argb = rgb
            .iter()
            .map(|&v| if process_alpha { v } else { v | 0xFF000000 })
            .collect();
        ArgbImage { width, height, argb }
    }

    /// `Image.getRGB(rgbData, offset, scanlength, x, y, w, h)`（`Image.java:80-86`）。
    pub fn get_rgb(&self, out: &mut [u32], offset: usize, scanlength: usize, x: i32, y: i32, w: i32, h: i32) {
        for row in 0..h {
            let src = ((y + row) * self.width + x) as usize;
            let dst = offset + row as usize * scanlength;
            out[dst..dst + w as usize].copy_from_slice(&self.argb[src..src + w as usize]);
        }
    }

    /// TickHooks.afterPaint 的帧哈希输入流：逐像素大端 4 字节（`TickHooks.java:53-59`）。
    pub fn hash_stream(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.argb.len() * 4);
        for &p in &self.argb {
            out.extend_from_slice(&[(p >> 24) as u8, (p >> 16) as u8, (p >> 8) as u8, p as u8]);
        }
        out
    }
}

/// 像素级 `Graphics` 复刻。目标画布通过借用注入。
///
/// `record_ops` 为 `Some` 时把每个绘制调用追加为 shim `op(...)` 同格式字符串
/// （trace 里 OPS 流的对拍载体，`Graphics.java:64-73`）；`None` 时零开销。
pub struct SoftGraphics<'a> {
    target: &'a mut ArgbImage,
    color: u32,
    font: Option<SoftFont>,
    clip_x: i32,
    clip_y: i32,
    clip_w: i32,
    clip_h: i32,
    /// OPS 流（shim `Graphics.flushOps` 的对应物；不带 "OPS count=" 头）。
    pub ops: Vec<String>,
}

impl<'a> SoftGraphics<'a> {
    /// `new Graphics(target)`：clip = 整幅画布，字体默认 MEDIUM。
    pub fn new(target: &'a mut ArgbImage) -> SoftGraphics<'a> {
        let (w, h) = (target.width, target.height);
        SoftGraphics {
            target,
            color: 0,
            font: Some(SoftFont::get_font(0, font_style::PLAIN, font_size::MEDIUM)),
            clip_x: 0,
            clip_y: 0,
            clip_w: w,
            clip_h: h,
            ops: Vec::new(),
        }
    }

    fn op(&mut self, s: String) {
        self.ops.push(s);
    }

    /// `getColor`。
    pub fn get_color(&self) -> u32 {
        self.color
    }

    /// `setColor(int rgb)`：`rgb & 0xFFFFFF`。
    pub fn set_color(&mut self, rgb: u32) {
        self.color = rgb & 0xFFFFFF;
        self.op(format!("setColor(#{:06x})", self.color));
    }

    /// `setColor(r, g, b)`。
    pub fn set_color_rgb(&mut self, r: i32, g: i32, b: i32) {
        self.set_color((((r << 16) | (g << 8) | b) as u32) & 0xFFFFFF);
    }

    /// `setFont`。`None` 对应 shim 传 null（后续文字绘制会 panic，模拟 NPE）。
    pub fn set_font(&mut self, font: Option<SoftFont>) {
        self.op(format!("setFont(size={})", font.map(|f| size_code(f.height)).unwrap_or(-1)));
        self.font = font;
    }

    fn font(&self) -> SoftFont {
        self.font.expect("NullPointerException（shim：font==null 时文字度量崩溃）")
    }

    /// `setClip`：与画布取绝对交集并**替换**当前剪裁区（`Graphics.java:120-131`）。
    pub fn set_clip(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let nx = x.max(0);
        let ny = y.max(0);
        let nx2 = (x + w).min(self.target.width);
        let ny2 = (y + h).min(self.target.height);
        self.clip_x = nx;
        self.clip_y = ny;
        self.clip_w = (nx2 - nx).max(0);
        self.clip_h = (ny2 - ny).max(0);
        self.op(format!("setClip({x},{y},{w},{h})"));
    }

    /// `clipRect`：与当前剪裁区取交（`Graphics.java:133-143`）。
    pub fn clip_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let x2 = (self.clip_x + self.clip_w).min(x + w);
        let y2 = (self.clip_y + self.clip_h).min(y + h);
        let nx = self.clip_x.max(x);
        let ny = self.clip_y.max(y);
        self.clip_x = nx;
        self.clip_y = ny;
        self.clip_w = (x2 - nx).max(0);
        self.clip_h = (y2 - ny).max(0);
        self.op(format!("clipRect({x},{y},{w},{h})"));
    }

    /// `translate`：**只移动 clip**（shim 特有，`Graphics.java:145-149`；坐标原点不动）。
    pub fn translate(&mut self, x: i32, y: i32) {
        self.clip_x += x;
        self.clip_y += y;
        self.op(format!("translate({x},{y})"));
    }

    // ---------------- 基础图元 ----------------

    /// `setPixel`：clip 判断 → alpha 0 跳过 / 255 直写 / 其他 src-over（`Graphics.java:161-181`）。
    fn set_pixel(&mut self, x: i32, y: i32, argb: u32) {
        if x < self.clip_x || y < self.clip_y || x >= self.clip_x + self.clip_w || y >= self.clip_y + self.clip_h {
            return;
        }
        let a = (argb >> 24) & 0xFF;
        let idx = (y * self.target.width + x) as usize;
        if a == 0 {
            return;
        }
        if a == 255 {
            self.target.argb[idx] = argb;
            return;
        }
        let dst = self.target.argb[idx];
        let ia = 255 - a;
        let blend = |s: u32, d: u32| ((s * a + d * ia) / 255) as u32;
        let r = blend((argb >> 16) & 0xFF, (dst >> 16) & 0xFF);
        let g = blend((argb >> 8) & 0xFF, (dst >> 8) & 0xFF);
        let b = blend(argb & 0xFF, dst & 0xFF);
        let da = (((dst >> 24) & 0xFF) * ia) / 255 + a;
        self.target.argb[idx] = (da << 24) | (r << 16) | (g << 8) | b;
    }

    fn opaque_color(&self) -> u32 {
        0xFF000000 | self.color
    }

    /// `fillRect`（`Graphics.java:187-195`）。
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.op(format!("fillRect({x},{y},{w},{h},#{:06x})", self.color));
        let c = self.opaque_color();
        for yy in y..y + h {
            for xx in x..x + w {
                self.set_pixel(xx, yy, c);
            }
        }
    }

    /// `drawRect`：四条 `drawLine`（`Graphics.java:197-203`）。
    pub fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.op(format!("drawRect({x},{y},{w},{h},#{:06x})", self.color));
        self.draw_line(x, y, x + w - 1, y);
        self.draw_line(x, y + h - 1, x + w - 1, y + h - 1);
        self.draw_line(x, y, x, y + h - 1);
        self.draw_line(x + w - 1, y, x + w - 1, y + h - 1);
    }

    /// `drawLine`：Bresenham（`Graphics.java:205-227`）。
    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.op(format!("drawLine({x1},{y1},{x2},{y2},#{:06x})", self.color));
        let c = self.opaque_color();
        let (dx, dy) = ((x2 - x1).abs(), (y2 - y1).abs());
        let (sx, sy) = (if x1 < x2 { 1 } else { -1 }, if y1 < y2 { 1 } else { -1 });
        let mut err = dx - dy;
        let (mut x, mut y) = (x1, y1);
        loop {
            self.set_pixel(x, y, c);
            if x == x2 && y == y2 {
                break;
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// `fillTriangle`：顶点按 y 稳定冒泡排序 + double 边缘插值（`Graphics.java:229-259`）。
    pub fn fill_triangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32) {
        self.op(format!(
            "fillTriangle({x1},{y1},{x2},{y2},{x3},{y3},#{:06x})",
            self.color
        ));
        let c = self.opaque_color();
        let (mut x1, mut y1, mut x2, mut y2, mut x3, mut y3) = (x1, y1, x2, y2, x3, y3);
        // 顶点排序（确定性）：与 shim 相同的三次比较交换（x/y 同步换）
        if y1 > y2 {
            std::mem::swap(&mut y1, &mut y2);
            std::mem::swap(&mut x1, &mut x2);
        }
        if y2 > y3 {
            std::mem::swap(&mut y2, &mut y3);
            std::mem::swap(&mut x2, &mut x3);
        }
        if y1 > y2 {
            std::mem::swap(&mut y1, &mut y2);
            std::mem::swap(&mut x1, &mut x2);
        }
        for y in y1..=y3 {
            let (xl, xr) = if y < y2 {
                (edge(x1, y1, x3, y3, y), edge(x1, y1, x2, y2, y))
            } else {
                (edge(x1, y1, x3, y3, y), edge(x2, y2, x3, y3, y))
            };
            let lo = xl.min(xr).ceil() as i32;
            let hi = xl.max(xr).floor() as i32;
            for x in lo..=hi {
                self.set_pixel(x, y, c);
            }
        }
    }

    /// `fillArc`（`Graphics.java:261-286`）。游戏唯一调用为全圆 0..360（a.java:6635）。
    pub fn fill_arc(&mut self, x: i32, y: i32, w: i32, h: i32, start_angle: i32, arc_angle: i32) {
        self.op(format!(
            "fillArc({x},{y},{w},{h},{start_angle},{arc_angle},#{:06x})",
            self.color
        ));
        let c = self.opaque_color();
        let cx = x as f64 + w as f64 / 2.0;
        let cy = y as f64 + h as f64 / 2.0;
        let rx = w as f64 / 2.0;
        let ry = h as f64 / 2.0;
        let mut norm_start = start_angle % 360;
        if norm_start < 0 {
            norm_start += 360;
        }
        for yy in y..y + h {
            for xx in x..x + w {
                let dx = xx as f64 + 0.5 - cx;
                let dy = yy as f64 + 0.5 - cy;
                if rx <= 0.0 || ry <= 0.0 {
                    continue;
                }
                let nx = dx / rx;
                let ny = dy / ry;
                if nx * nx + ny * ny > 1.0 {
                    continue;
                }
                if in_arc(f64::atan2(ny, nx), norm_start, arc_angle) {
                    self.set_pixel(xx, yy, c);
                }
            }
        }
    }

    /// `drawArc`（`Graphics.java:288-311`）。参考版未调用，保真复刻以对齐 shim 全集。
    pub fn draw_arc(&mut self, x: i32, y: i32, w: i32, h: i32, start_angle: i32, arc_angle: i32) {
        self.op(format!(
            "drawArc({x},{y},{w},{h},{start_angle},{arc_angle},#{:06x})",
            self.color
        ));
        let c = self.opaque_color();
        let cx = x as f64 + w as f64 / 2.0;
        let cy = y as f64 + h as f64 / 2.0;
        let rx = w as f64 / 2.0;
        let ry = h as f64 / 2.0;
        let mut norm_start = start_angle % 360;
        if norm_start < 0 {
            norm_start += 360;
        }
        for yy in y..y + h {
            for xx in x..x + w {
                let dx = xx as f64 + 0.5 - cx;
                let dy = yy as f64 + 0.5 - cy;
                let nx = if rx > 0.0 { dx / rx } else { 0.0 };
                let ny = if ry > 0.0 { dy / ry } else { 0.0 };
                let r2 = nx * nx + ny * ny;
                if !(0.72..=1.28).contains(&r2) {
                    continue;
                }
                if in_arc(f64::atan2(ny, nx), norm_start, arc_angle) {
                    self.set_pixel(xx, yy, c);
                }
            }
        }
    }

    // ---------------- 图像 ----------------

    /// `drawImage(img, x, y, anchor)`（`Graphics.java:336-340`）。
    pub fn draw_image(&mut self, img: &ArgbImage, x: i32, y: i32, anchor: i32) {
        self.op(format!("drawImage({}x{},{x},{y},{anchor})", img.width, img.height));
        let adj = anchor_adjust(anchor, img.width, img.height);
        self.blit(img, x - adj[0], y - adj[1]);
    }

    /// Nokia `DirectGraphics.drawImage(img, x, y, anchor, transform)`
    /// （`Graphics.drawImageTransformed`，`Graphics.java:346-376`）。
    pub fn draw_image_transformed(&mut self, img: &ArgbImage, x: i32, y: i32, anchor: i32, transform: i32) {
        self.op(format!(
            "drawImageT({}x{},{x},{y},{anchor},{transform})",
            img.width, img.height
        ));
        let swap = (4..=7).contains(&transform);
        let bw = if swap { img.height } else { img.width };
        let bh = if swap { img.width } else { img.height };
        let adj = anchor_adjust(anchor, bw, bh);
        let ox = x - adj[0];
        let oy = y - adj[1];
        let (w, h) = (img.width, img.height);
        for sy in 0..h {
            for sx in 0..w {
                let p = img.argb[(sy * w + sx) as usize];
                if (p >> 24) & 0xFF == 0 {
                    continue;
                }
                let (dx, dy) = match transform {
                    1 => (sx, h - 1 - sy),
                    2 => (w - 1 - sx, sy),
                    3 => (w - 1 - sx, h - 1 - sy),
                    4 => (sy, sx),
                    5 => (h - 1 - sy, sx),
                    6 => (sy, w - 1 - sx),
                    7 => (h - 1 - sy, w - 1 - sx),
                    _ => (sx, sy),
                };
                self.set_pixel(ox + dx, oy + dy, p);
            }
        }
    }

    /// `Graphics.blit`（非 transpose 形态即 drawImage 内部路径，`Graphics.java:378-390`）。
    fn blit(&mut self, img: &ArgbImage, x: i32, y: i32) {
        for sy in 0..img.height {
            for sx in 0..img.width {
                let p = img.argb[(sy * img.width + sx) as usize];
                if (p >> 24) & 0xFF == 0 {
                    continue;
                }
                self.set_pixel(x + sx, y + sy, p);
            }
        }
    }

    // ---------------- 文字 ----------------

    /// `drawChar`。
    pub fn draw_char(&mut self, ch: u16, x: i32, y: i32, anchor: i32) {
        self.op(format!("drawChar({ch},{x},{y},{anchor})"));
        self.draw_glyph(ch, x, y, anchor);
    }

    /// `drawString`。
    pub fn draw_string(&mut self, s: &[u16], x: i32, y: i32, anchor: i32) {
        self.op(format!("drawString({}, {x},{y},{anchor})", quote_utf16(s)));
        self.draw_text(s, 0, s.len(), x, y, anchor);
    }

    /// `drawSubstring`。
    ///
    /// len<0 或越界时与 shim 同语义 = 空循环静默无操作
    /// （Font.substringWidth/Graphics.drawText 的 `i < offset+len` 不执行；
    /// 对抗 review D-2：调用方 a.java:5035 的 len 是差值，可能为负）。
    pub fn draw_substring(&mut self, s: &[u16], offset: i32, len: i32, x: i32, y: i32, anchor: i32) {
        self.op(format!(
            "drawSubstring({}, {offset},{len},{x},{y},{anchor})",
            quote_utf16(s)
        ));
        let from = offset.max(0) as usize;
        let to = (offset + len).clamp(0, s.len() as i32) as usize;
        if from >= to || to > s.len() {
            return;
        }
        self.draw_text(s, from, to, x, y, anchor);
    }

    /// `drawText`：先整体度量锚点，再逐字符步进（`Graphics.java:424-438`）。
    fn draw_text(&mut self, s: &[u16], from: usize, to: usize, x: i32, y: i32, anchor: i32) {
        let font = self.font();
        let w = font.substring_width(s, from, to - from);
        let h = font.height;
        let mut adj = anchor_adjust(anchor, w, h);
        if anchor & anchor::BASELINE != 0 {
            adj[1] = font.baseline;
        }
        let mut px = x - adj[0];
        let py = y - adj[1];
        for &ch in &s[from..to] {
            self.draw_glyph(ch, px, py, anchor::TOP | anchor::LEFT);
            px += font.char_width(ch);
        }
    }

    /// `drawGlyph`：确定性 hash 5 列点阵（`Graphics.java:440-462`）。
    ///
    /// hash = (int)(ch * 2654435761L)（long 乘法取低 32 位），位采样 `(hash >>> ((gx + gy*5) % 29)) & 1`。
    fn draw_glyph(&mut self, ch: u16, x: i32, y: i32, anchor: i32) {
        let font = self.font();
        let cw = font.char_width(ch);
        let chh = font.height;
        let mut adj = anchor_adjust(anchor, cw, chh);
        if anchor & anchor::BASELINE != 0 {
            adj[1] = font.baseline;
        }
        let px = x - adj[0];
        let py = y - adj[1];
        let c = self.opaque_color();
        let gw = (cw * 4 / 5).max(2);
        let gh = (chh * 3 / 5).max(3);
        let hash = ((ch as i64).wrapping_mul(2654435761)) as i32 as u32;
        for gy in 0..gh {
            for gx in 0..gw {
                let bit = (hash >> ((gx + gy * 5) % 29)) & 1;
                if bit != 0 {
                    self.set_pixel(px + (cw - gw) / 2 + gx, py + (chh - gh) / 2 + gy, c);
                }
            }
        }
    }
}

/// `edge(x1,y1,x2,y2,y)`：double 插值，Java 表达式序 `(x2-x1)*(y-y1)/(y2-y1) + x1`。
fn edge(x1: i32, y1: i32, x2: i32, y2: i32, y: i32) -> f64 {
    if y1 == y2 {
        return (x1.min(x2)) as f64;
    }
    x1 as f64 + ((x2 - x1) as f64 * (y - y1) as f64 / (y2 - y1) as f64)
}

/// `inArc`（`Graphics.java:313-332`）。
fn in_arc(rad: f64, start_deg: i32, arc_deg: i32) -> bool {
    let mut deg = rad.to_degrees();
    if deg < 0.0 {
        deg += 360.0;
    }
    if arc_deg >= 360 {
        return true;
    }
    if arc_deg < 0 {
        // shim 同位置有 % 360（Graphics.java:322）；大角度下省略会因 f64
        // 精度丢失与 while 归一化产生 ULP 级漂移（对抗 review D-1）
        return !in_arc(rad, (start_deg + arc_deg) % 360, -arc_deg);
    }
    let mut rel = deg - start_deg as f64;
    while rel < 0.0 {
        rel += 360.0;
    }
    while rel >= 360.0 {
        rel -= 360.0;
    }
    rel <= arc_deg as f64
}

/// `anchorAdjust`（`Graphics.java:392-405`）：水平 HCENTER→w/2、RIGHT→w；垂直 VCENTER→h/2、BOTTOM→h。
fn anchor_adjust(anchor: i32, w: i32, h: i32) -> [i32; 2] {
    let mut ax = 0;
    let mut ay = 0;
    if anchor & anchor::HCENTER != 0 {
        ax = w / 2;
    } else if anchor & anchor::RIGHT != 0 {
        ax = w;
    }
    if anchor & anchor::VCENTER != 0 {
        ay = h / 2;
    } else if anchor & anchor::BOTTOM != 0 {
        ay = h;
    }
    [ax, ay]
}

/// 由行高反推 shim `setFont` op 里的 size 码（只影响 op 文本，不影响像素）。
fn size_code(height: i32) -> i32 {
    match height {
        10 => 8,  // SMALL
        20 => 32, // LARGE
        14 => 16, // MEDIUM
        other => other,
    }
}

/// OPS 文本里的字符串转义（shim `Graphics.q`，`Graphics.java:75-94`）。
fn quote_utf16(s: &[u16]) -> String {
    let mut out = String::from("\"");
    for &c in s {
        match c {
            0x22 => out.push_str("\\\""), // "
            0x5C => out.push_str("\\\\"), // backslash
            0x0A => out.push_str("\\n"),
            0x0D => out.push_str("\\r"),
            0x09 => out.push_str("\\t"),
            _ if (0x20..0x7F).contains(&c) => out.push(c as u8 as char),
            _ => out.push_str(&format!("\\u{c:04x}")),
        }
    }
    out.push('"');
    out
}
