//! 菜单族（modes 8/15/16/17/22）——读档槽、帮助/关于 overlay、选项列表、
//! 帮助退出动画（第二个 logo 时间线）。
//!
//! # 结构（A 级证据）
//!
//! - **mode 8**（paint a.java:2684-2688 / run a.java:3821-3876 / m_000 case 8
//!   a.java:4348-4373）：parallax 背景 + paintSlotList 槽位列表 + 软键 (1,3)。
//!   -2/-1 移动槽光标（slotCursor，窗口 slotScrollTop/slotScrollBottom）；-7 → menuReturnMode；
//!   槽无效（oracle RMS 恒空）时 -5 无操作。
//! - **mode 15/17**（BUG-007：标签「保存游戏/设置」实为帮助/关于）：
//!   paint case 15/17 = parallax + iconStrip(y0=10, sy=51/68) + 软键 (0,3)；
//!   m_000 经 showOverlayMessage(0, 大文本) 置 overlayActive——**overlay 层**
//!   （run a.java:3246-3266 case 0 的 15/17 分支）处理 -7（退出→menuReturnMode）
//!   与 -2/-1（翻页）。
//! - **mode 16**（选项列表）：paint case 16（paintBoxFrame 盒 + 高亮行 + 勾选框）；
//!   run case 16（a.java:4131-4179）：-2/-1 移动 optionCursor、-5/-4/-3 翻转
//!   optionChecked[高亮]；-7/-6 → menuReturnMode。
//! - **mode 22**（帮助→退出）：run case 22（a.java:4237-4246）++bootPhaseCounter≤70
//!   ⇒ runLogoAnimation(1, t)（var1=1 分支 a.java:10312-10360：t=18..30 逐字
//!   上滑注册、t=50/51/52/58/63 修改项）；paint case 22 = paintLogoAnimation。
//!   70 拍后 f_bool_00=false（进程退出）。
//! - **overlay 绘制**（paint 公共尾 a.java:3123-3171）：paintBoxFrame 居中盒 +
//!   paintWrappedText 排版文本（\cRRGGBB 着色、\r 复原、翻页箭头闪烁 frameCounter&1）。
//!
//! # paint 内状态推进
//!
//! `drawParallaxBackdrop` 的 backdropScroll 在 **paint 内** --1（<-154 重置 0，
//! 周期 155）——与 Java 相同的副作用位置。

use crate::render::{ArgbImage, SoftGraphics};

/// parseHexColor（a.java:5149-5159）：hex 颜色解析，失败 → PARSE_DEFAULT（永不 -1）。
fn parse_hex_color(s: &[u16]) -> i32 {
    let txt: String = s.iter().map(|&c| c as u8 as char).collect();
    i32::from_str_radix(&txt, 16).unwrap_or(crate::layout::PARSE_DEFAULT as i32)
}

/// parseHexColor 恒不为 -1 的判定（6 位 hex 解析结果域内不为 -1；NaN 路径返回默认色）。
fn hex_color_valid(s: &[u16]) -> bool {
    let txt: String = s.iter().map(|&c| c as u8 as char).collect();
    i32::from_str_radix(&txt, 16).map(|v| v != -1).unwrap_or(false)
}

// ================================ parallax ================================

/// 视差背景（drawParallaxBackdrop a.java:6838-6865）。
pub struct ParallaxBackdrop {
    /// f_int_149 之外独立字段 backdropScroll（paint 内每拍 -1，<-154 重置）。
    scroll: i32,
}

impl Default for ParallaxBackdrop {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallaxBackdrop {
    pub fn new() -> ParallaxBackdrop {
        ParallaxBackdrop { scroll: 0 }
    }

    /// `drawParallaxBackdrop(game_view)`（a.java:6840-6867，与 scene.rs
    /// `paint_parallax` 同一 Java 方法——列步进/回绕/行步进共用 layout 常量）。
    /// `game_view=true` 时纵向平铺带翻转（mode 3/4/19 用），菜单族恒 false。
    pub fn paint(&mut self, g: &mut SoftGraphics<'_>, tile: &ArgbImage, game_view: bool, view_bottom: i32) {
        self.scroll -= 1;
        if self.scroll < crate::layout::PARALLAX_WRAP {
            self.scroll = 0;
        }
        let base_y = if game_view { view_bottom - crate::layout::SCREEN_H + 6 } else { 0 };
        let mut x = self.scroll;
        while x < crate::layout::SCREEN_W {
            let mut y = base_y;
            let mut flipped = false;
            while y > -crate::layout::SCREEN_H {
                if flipped {
                    let t = crate::paint::NOKIA_TRANSFORM_TABLE[2];
                    g.draw_image_transformed(tile, x, y, 0, t);
                } else {
                    g.draw_image(tile, x, y, 0);
                }
                flipped = !flipped;
                y -= crate::layout::SCREEN_H;
            }
            x += crate::layout::PARALLAX_COL_STEP;
        }
    }
}

// ============================== 盒框绘制 ==============================

/// `drawEdgePatch`（a.java:4567-4616）：角/边片绘制（kind 1-7 = 变换码表索引，
/// 全 >7 ⇒ shim 原样绘制，仅坐标公式生效）。
#[allow(clippy::too_many_arguments)]
pub fn draw_edge_patch(
    g: &mut SoftGraphics<'_>,
    img: &ArgbImage,
    x: i32,
    y: i32,
    sx: i32,
    sy: i32,
    w: i32,
    h: i32,
    kind: i32,
) {
    g.set_clip(x, y, w, h);
    if kind == 0 {
        g.draw_image(img, x - sx, y - sy, 0);
    } else {
        let (iw, ih) = (img.width, img.height);
        let (dx, dy) = match kind {
            1 => (x - (iw - sx - w), y - sy),
            2 => (x - sx, y - (ih - sy - h)),
            3 => (x - (iw - sx - w), y - (ih - sy - h)),
            4 => {
                g.set_clip(x, y, h, w);
                (x - sy, y - sx)
            }
            5 => {
                g.set_clip(x, y, h, w);
                (x - (ih - sy - h), y - sx)
            }
            6 => {
                g.set_clip(x, y, h, w);
                (x - sy, y - (iw - sx - w))
            }
            7 => {
                g.set_clip(x, y, h, w);
                (x - (ih - sy - h), y - (iw - sx - w))
            }
            _ => (x - sx, y - sy),
        };
        let t = crate::paint::NOKIA_TRANSFORM_TABLE[kind as usize];
        g.draw_image_transformed(img, dx, dy, 0, t);
    }
    g.set_clip(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
}

/// `paintBoxFrame`（a.java:6109-6149）：标准盒框（ui[8][0] 角片 26×16 / 边片 16×16）。
pub fn paint_box_frame(g: &mut SoftGraphics<'_>, ui0: &ArgbImage, x: i32, y: i32, w: i32, h: i32) {
    // 顶边（含左上角）
    g.set_clip(x, y, 26, 16);
    g.draw_image(ui0, x, y, 0);
    let mut vx = x + 26;
    use crate::layout::{
        BORDER_CORNER_W, BORDER_MID_W, BORDER_SIDE_SRC_L, BORDER_SIDE_SRC_R, BORDER_SIDE_W,
        BORDER_STRIP_H,
    };
    while vx < x + w - BORDER_CORNER_W {
        g.set_clip(vx, y, BORDER_MID_W, BORDER_STRIP_H);
        g.draw_image(ui0, vx - BORDER_CORNER_W, y, 0);
        draw_edge_patch(g, ui0, vx, y + h - BORDER_STRIP_H, BORDER_CORNER_W, 0, BORDER_MID_W, BORDER_STRIP_H, 2);
        vx += BORDER_MID_W;
    }
    g.set_clip(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
    draw_edge_patch(g, ui0, x + w - BORDER_CORNER_W, y, 0, 0, BORDER_CORNER_W, BORDER_STRIP_H, 1);
    // 内衬
    g.set_color(crate::layout::HUD_FILL);
    g.fill_rect(x + BORDER_SIDE_W, y + BORDER_STRIP_H, w - BORDER_SIDE_W * 2, h - BORDER_STRIP_H * 2);
    // 侧边
    let right_x = x + w - BORDER_SIDE_W;
    let mut vy = y + BORDER_STRIP_H;
    while vy < y + h - BORDER_STRIP_H {
        g.set_clip(x, vy, BORDER_SIDE_W, BORDER_STRIP_H);
        g.draw_image(ui0, x - BORDER_SIDE_SRC_L, vy, 0);
        g.set_clip(right_x, vy, BORDER_SIDE_W, BORDER_STRIP_H);
        g.draw_image(ui0, right_x - BORDER_SIDE_SRC_R, vy, 0);
        vy += BORDER_STRIP_H;
    }
    draw_edge_patch(g, ui0, x, y + h - BORDER_STRIP_H, 0, 0, BORDER_CORNER_W, BORDER_STRIP_H, 2);
    draw_edge_patch(g, ui0, x + w - BORDER_CORNER_W, y + h - BORDER_STRIP_H, 0, 0, BORDER_CORNER_W, BORDER_STRIP_H, 3);
    g.set_clip(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
}

/// `paintTitledBox`（a.java:6102-6121）：盒框 + 顶部标题条（ui[8][13] 按 `segs`
/// 段表切 14px 片）。
pub fn paint_titled_box(
    g: &mut SoftGraphics<'_>,
    ui0: &ArgbImage,
    ui13: &ArgbImage,
    segs: &[i32],
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) {
    paint_box_frame(g, ui0, x, y, w, h);
    let mut tx = x + 2;
    let ty = y - 2;
    for &seg in segs {
        g.set_clip(tx, ty, 14, 16);
        g.draw_image(ui13, tx - seg * 14, ty, 0);
        tx += 14;
    }
    g.set_clip(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
}

/// `paintMiniFrame`（a.java:6186-6195）：小框（32×32 图标框底）。
pub fn paint_mini_frame(g: &mut SoftGraphics<'_>, x: i32, y: i32, w: i32, h: i32) {
    let bottom = y + h - 2;
    let right = x + w - 1;
    g.set_color(crate::layout::MINIFRAME_FILL);
    g.fill_rect(x + 1, y, w - 2, h - 1);
    g.draw_line(x, y + 1, x, bottom);
    g.set_color(crate::layout::MINIFRAME_SHADE);
    g.draw_line(right, y + 1, right, bottom);
    g.draw_line(x + 1, bottom + 1, right - 1, bottom + 1);
}

/// `paintNumber`（a.java:6197-6226）：数字条（右对齐逐位，负数画横线）。
/// 返回绘制位数（调用方用于宽度推进）。
pub fn paint_number(g: &mut SoftGraphics<'_>, digits: &ArgbImage, value: i32, x: i32, y: i32) -> i32 {
    let negative = value < 0;
    let mut v = if negative { -value } else { value };
    let dw = digits.width / 10;
    let dh = digits.height;
    let mut cx = x;
    let mut count = 0;
    loop {
        cx -= dw + 1;
        let d = v % 10;
        g.set_clip(cx, y, dw, dh);
        g.draw_image(digits, cx - d * dw, y, 0);
        v /= 10;
        count += 1;
        if v <= 0 {
            break;
        }
    }
    g.set_clip(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
    if negative {
        g.set_color(crate::layout::NUMBER_NEGATIVE);
        g.draw_line(cx - dw + 1, y + 3, cx - 1, y + 3);
        count += 1;
    }
    count
}

// ============================== overlay 引擎 ==============================

/// overlay 文本状态（showOverlayMessage + layoutFullText/layoutWrappedText 布局 + paintWrappedText 绘制 +
/// flipOverlayPage 翻页；行缓冲 wrappedLines）。
pub struct OverlayEngine {
    /// 行缓冲（wrappedLines，layoutWrappedText 切行产物；UTF-16 码元）。
    lines: Vec<Vec<u16>>,
    /// 缓存锚（overlayLayoutCache != text 才重排）。
    cached_text: Option<Vec<u16>>,
    /// overlayPageTop：页首行。
    page_top: i32,
    /// overlayPageBottom：页尾行（独占）。
    page_bottom: i32,
    /// overlayTotalLines：总行数。
    total_lines: i32,
    /// overlayLinesPerPage：每页行数。
    lines_per_page: i32,
    /// overlayPageHeight：页高（行高×每页行数）。
    page_height: i32,
    /// overlayContentWidth：内容宽（单行 = 整串宽，多行 = 布局宽 var2）。
    content_width: i32,
    /// overlayBoxWidth/overlayBoxHeight：overlay 盒尺寸。
    pub box_w: i32,
    pub box_h: i32,
    /// overlaySavedColor：\r 恢复色。
    saved_color: u32,
    /// 行高 textLineHeight（fontHeight+4=14）。
    line_height: i32,
}

impl Default for OverlayEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayEngine {
    pub fn new() -> OverlayEngine {
        OverlayEngine {
            lines: Vec::new(),
            cached_text: None,
            page_top: 0,
            page_bottom: 0,
            total_lines: 0,
            lines_per_page: 0,
            page_height: 0,
            content_width: 0,
            box_w: 0,
            box_h: 0,
            saved_color: 0,
            line_height: 14, // textLineHeight = fontHeight+4（a.java:471）
        }
    }

    /// `layoutWrappedText`（a.java:4988-5009 区）：切行布局。`(max_w, max_h)` =
    /// showOverlayMessage case 0 的 (180, 240)。
    fn layout(&mut self, text: &[u16], max_w: i32, max_h: i32, font: &crate::render::SoftFont) {
        self.lines.clear();
        let mut cur_w = 0i32; // var8
        let mut line_start = 0usize; // var10
        let mut line_len = 0usize; // var11
        let n = text.len();
        let mut i = 0usize; // var12
        while i < n {
            let ch = text[i];
            let mut cw = font.char_width(ch);
            if ch == b'\n' as u16 {
                self.lines.push(text[line_start..line_start + line_len].to_vec());
                line_start = i + 1;
                cur_w = 0;
                line_len = 0;
            } else {
                if ch == b'\\' as u16 {
                    i += 1;
                    if i < n {
                        let c2 = text[i];
                        if c2 == b'c' as u16 {
                            if i + 6 < n && hex_color_valid(&text[i + 1..i + 7]) {
                                i += 6;
                                line_len += 7;
                                cw = 0;
                            } else {
                                cw = font.char_width(b'\\' as u16) + font.char_width(c2);
                            }
                        } else if c2 == b'r' as u16 {
                            line_len += 1;
                            cw = 0;
                        }
                    } else {
                        cw = font.char_width(b'\\' as u16);
                        i -= 1; // 溢出保护：var12--
                    }
                }
                cur_w += cw;
                if cur_w > max_w {
                    self.lines.push(text[line_start..line_start + line_len].to_vec());
                    cur_w = cw;
                    line_start = i;
                    line_len = 1;
                } else {
                    line_len += 1;
                }
            }
            i += 1;
        }
        if line_len > 0 {
            self.lines.push(text[line_start..line_start + line_len].to_vec());
        }
        self.page_top = 0;
        self.total_lines = self.lines.len() as i32;
        let mut per = max_h / self.line_height;
        if self.total_lines < per {
            per = self.total_lines;
        }
        self.lines_per_page = per;
        self.page_height = self.line_height * per;
        self.page_bottom = if self.total_lines > per { per } else { self.total_lines };
        self.content_width = if self.total_lines == 1 {
            font.string_width(text)
        } else {
            max_w
        };
    }

    /// `showOverlayMessage` case 0/2/4/5（a.java:4995-5002）：布局 + 盒尺寸。
    pub fn show_kind0(&mut self, text: &[u16], font: &crate::render::SoftFont, footer_icons: (i8, i8)) {
        self.layout(text, 180, 240, font);
        self.box_w = self.content_width + 32;
        self.box_h = 32 + (self.page_bottom - self.page_top) * self.line_height;
        if footer_icons.0 != 0 || footer_icons.1 != 0 {
            self.box_h += 18;
        }
        self.cached_text = Some(text.to_vec());
    }

    /// 引子字幕布局（m_014 依赖行宽/总行数；布局参数与 paint 调用一致 220/28）。
    pub fn layout_for_intro(&mut self, text: &[u16], font: &crate::render::SoftFont) {
        self.layout(text, 220, 28, font);
        self.cached_text = Some(text.to_vec());
    }

    /// f_String_03 != null（上次 paint 布局锚存在——m_014 滚动机门槛）。
    pub fn layout_done(&self) -> bool {
        self.cached_text.is_some()
    }

    pub fn needs_layout(&self, text: &[u16]) -> bool {
        self.cached_text.as_deref() != Some(text)
    }

    pub fn line_at_top(&self) -> usize {
        self.page_top as usize
    }

    pub fn line(&self, idx: usize) -> Vec<u16> {
        self.lines.get(idx).cloned().unwrap_or_default()
    }

    pub fn total_lines(&self) -> i32 {
        self.total_lines
    }

    pub fn lines_per_page(&self) -> i32 {
        self.lines_per_page
    }

    pub fn page_top(&self) -> i32 {
        self.page_top
    }

    /// `flipOverlayPage`（a.java:5043-5062）：翻页钳制。
    pub fn flip_page(&mut self, page: i32) {
        let mut p = page;
        if p >= 0 {
            if p >= self.total_lines - self.lines_per_page + 1 {
                p = self.total_lines - self.lines_per_page;
            }
        } else {
            p = 0;
        }
        self.page_top = p;
        self.page_bottom = self.page_top + self.lines_per_page;
        if self.page_top < 0 {
            self.page_top = 0;
        }
        if self.page_bottom > self.total_lines {
            self.page_bottom = self.total_lines;
        }
    }

    /// `paintWrappedText`（a.java:5018-5072）：排版绘制（\c 着色 / \r 复原 / 翻页箭头）。
    /// `(x, y, w, _h, arrows)`；`frame_counter` = paint 时 frameCounter 值。
    pub fn paint(
        &mut self,
        g: &mut SoftGraphics<'_>,
        text: &[u16],
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        arrows: bool,
        frame_counter: i32,
        font: &crate::render::SoftFont,
    ) {
        if self.cached_text.as_deref() != Some(text) {
            // paintWrappedText 缓存失效 ⇒ layoutFullText(text, w, h)（a.java:5019-5022）——
            // 布局参数来自调用方（overlay=180/240 预排、引子字幕=220/28）
            self.layout(text, w, h, font);
            self.cached_text = Some(text.to_vec());
        }
        let mut dy = y + 2;
        self.saved_color = g.get_color();
        // 页首行之前最近的颜色标记（a.java:5033-5043，倒序扫 --var8 >= 0）
        let mut probe = self.page_top;
        loop {
            probe -= 1;
            if probe < 0 {
                break;
            }
            let line = &self.lines[probe as usize];
            if let Some(pos) = line.iter().rposition(|&c| c == b'\\' as u16) {
                if line.get(pos + 1) == Some(&(b'c' as u16)) && pos + 8 <= line.len() {
                    let color = parse_hex_color(&line[pos + 2..pos + 8]);
                    g.set_color(color as u32);
                    break;
                }
            }
        }
        // 行绘制（a.java:5045-5072）
        let mut row = self.page_top;
        while row < self.page_bottom {
            let line = &self.lines[row as usize].clone();
            let len = line.len();
            let mut seg_start = 0usize;
            let mut px = x;
            let mut color_tmp: i32 = 0;
            let mut j = 0usize;
            while j < len {
                if line[j] == b'\\' as u16 {
                    if j > 0 {
                        g.draw_substring(line, seg_start as i32, (j - seg_start) as i32, px, dy, 0);
                        px += font.substring_width(line, seg_start, j - seg_start);
                    }
                    j += 1;
                    if j < len {
                        let c2 = line[j];
                        if c2 == b'c' as u16 {
                            let mut parsed: i32 = 0;
                            if j + 6 < len {
                                parsed = parse_hex_color(&line[j + 1..j + 7]);
                            }
                            if parsed != -1 {
                                color_tmp = g.get_color() as i32;
                                g.set_color(parsed as u32);
                                j += 6;
                                seg_start = j + 1;
                            } else {
                                seg_start = j - 1;
                            }
                        } else if c2 == b'r' as u16 {
                            g.set_color(color_tmp as u32);
                            seg_start = j + 1;
                        } else {
                            seg_start = j;
                        }
                    }
                }
                j += 1;
            }
            if seg_start != 0 {
                if seg_start < len {
                    g.draw_substring(line, seg_start as i32, (len - seg_start) as i32, px, dy, 0);
                }
            } else {
                g.draw_string(line, x, dy, 0);
            }
            dy += self.line_height;
            row += 1;
        }
        // 翻页箭头（a.java:5073-5088）
        let cx = x + (w >> 1);
        if arrows {
            g.set_color(crate::layout::WHITE);
            if self.page_top > 0 {
                let ay = y - 8 + (frame_counter & 1);
                g.fill_triangle(cx, ay, cx - 7, ay + 7, cx + 7, ay + 7);
            }
            if self.page_bottom < self.total_lines {
                let ay = y + self.page_height + 3 - (frame_counter & 1);
                g.fill_triangle(cx, ay, cx - 6, ay - 6, cx + 6, ay - 6);
            }
        }
    }
}

// ============================== 槽位列表 ==============================

/// paintSlotList（a.java:9040-9151）的槽位表（oracle RMS 恒空 ⇒ 全无效）。
pub struct SlotSelect {
    /// slotCursor：槽光标（0..5 环绕）。
    pub cursor: i32,
    /// slotScrollTop/slotScrollBottom：可见窗口。
    scroll_top: i32,
    scroll_bottom: i32,
    /// slotsPerPage：每页行数（m_000 case 8 = min(6, 208/14) = 6）。
    rows_per_page: i32,
    /// slotListWidth：内容宽（max(148, wideCharWidth*3+60) = 148）。
    content_w: i32,
    /// slotListHeight：内容高（112+14*6 = 196）。
    content_h: i32,
    /// 槽有效性（RMS 读档；oracle 恒 [false;6]）。
    pub valid: [bool; 6],
}

impl SlotSelect {
    pub fn new() -> SlotSelect {
        let per = 208 / (10 + 4); // fontHeight+4 = 14
        let per = if per > 6 { 6 } else { per };
        let w = (12 * 3 + 60).max(148); // wideCharWidth*3+60，钳 148
        let h = 112 + (10 + 4) * per;
        SlotSelect {
            cursor: 0,
            scroll_top: 0,
            scroll_bottom: per,
            rows_per_page: per,
            content_w: w,
            content_h: h,
            valid: [false; 6],
        }
    }

    /// run case 8 的 -2（a.java:3853-3862）。
    pub fn cursor_down(&mut self) {
        if self.cursor < 5 {
            self.cursor += 1;
            if self.cursor >= self.scroll_bottom {
                self.scroll_bottom += 1;
                self.scroll_top += 1;
            }
        } else {
            self.cursor = 0;
            self.scroll_top = 0;
            self.scroll_bottom = self.rows_per_page;
        }
    }

    /// run case 8 的 -1（a.java:3864-3873）。
    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            if self.cursor < self.scroll_top {
                self.scroll_top -= 1;
                self.scroll_bottom -= 1;
            }
        } else {
            self.cursor = 5;
            self.scroll_bottom = 6;
            self.scroll_top = 6 - self.rows_per_page;
        }
    }

    /// `paintSlotList(false)`（a.java:9040-9083 无效槽路径：详情面板跳过）。
    pub fn paint(
        &self,
        g: &mut SoftGraphics<'_>,
        ui: &[ArgbImage],
        segs: &[i32],
        arrow_y: fn() -> i32,
    ) {
        let x = 240 - self.content_w - 22 >> 1;
        let mut y = 320 - self.content_h >> 1;
        paint_titled_box(g, &ui[0], &ui[13], segs, x, y, self.content_w + 22, self.content_h);
        y += 16;
        for row in self.scroll_top..self.scroll_bottom {
            if row != self.cursor {
                g.set_color(crate::layout::SLOT_TEXT);
            } else {
                g.set_color(crate::layout::DARK_BACKDROP);
                g.fill_rect(120 - (self.content_w >> 1), y, self.content_w, 10 + 4);
                g.draw_image(&ui[14], 120 - (self.content_w >> 1) + 10, y + (10 - 8 >> 1), 0);
                let t = crate::paint::NOKIA_TRANSFORM_TABLE[1];
                g.draw_image_transformed(&ui[14], 120 + (self.content_w >> 1) - 30, y + (10 - 8 >> 1), 0, t);
                g.set_color(crate::layout::SLOT_TEXT_ACTIVE);
            }
            if !self.valid[row as usize] {
                g.draw_string(&DASH_LABEL, 120, y + 2, 17);
            } else {
                let mut label: Vec<u16> = SAVE_LABEL.to_vec();
                label.extend((row as u16).to_string().chars().map(|c| c as u16));
                g.draw_string(&label, 120, y + 2, 17);
            }
            y += 14;
        }
        // 翻页箭头（a.java:9068-9075）
        let ax = 120 + ((self.content_w >> 1) - 10);
        if self.scroll_top > 0 {
            crate::paint::draw_image_clipped(g, &ui[15], ax, y - 20, 0, 0, 7, 9);
        }
        if self.scroll_bottom < 6 {
            crate::paint::draw_image_clipped(g, &ui[15], ax, y - 10, 7, 0, 7, 9);
        }
        // 分隔线（a.java:9077-9083）
        let mut ly = y + 2;
        let lx = ax - self.content_w + 10;
        g.set_color(crate::layout::SLOT_RULE_DARK);
        g.draw_line(lx, ly, lx + self.content_w - 1, ly);
        ly += 1;
        g.set_color(crate::layout::SLOT_RULE_LIGHT);
        g.draw_line(lx, ly, lx + self.content_w - 1, ly);
        let _ = arrow_y; // 详情面板（有效槽）未启用——oracle RMS 恒空
    }
}

/// "---"（无效槽标签，a.java:9060）。
const DASH_LABEL: &[u16] = &[0x2D, 0x2D, 0x2D];
/// "存档"（有效槽标签前缀，a.java:9062）。
const SAVE_LABEL: &[u16] = &[0x5B58, 0x6863];

// ============================== 菜单族状态机 ==============================

/// mode 16 选项列表（addOption 登记产物）。
pub struct OptionList {
    /// optionLabelIdx：标签索引表（optionLabels 下标）。
    pub labels: Vec<usize>,
    /// optionChecked：勾选状态。
    pub checked: Vec<bool>,
    /// optionCursor：高亮行。
    pub highlight: i32,
    /// optionBoxWidth/optionBoxHeight：盒宽/高。
    box_w: i32,
    box_h: i32,
}

#[cfg(test)]
mod option_tests {
    use super::*;

    #[test]
    fn checkbox_x_math() {
        let font = crate::paint::paint_font();
        for (idx, expect_lx, expect_cx) in [(0usize, 100i32, 126i32), (1, 94, 132)] {
            let label = OPTION_LABELS[idx];
            let w = font.string_width(label);
            let lx = 240 - w - 16 >> 1;
            let cx = lx + 2 + w;
            assert_eq!((lx, cx), (expect_lx, expect_cx), "label[{idx}] w={w}");
        }
    }
}

/// optionLabels（a.java 构造）：{"声音","小地图",...} 取前两项的 UTF-16。
pub const OPTION_LABELS: [&[u16]; 2] = [
    &[0x58F0, 0x97F3],       // 声音
    &[0x5C0F, 0x5730, 0x56FE], // 小地图
];

impl OptionList {
    /// m_000 case 16（a.java:4402-4410）：optionCount=0 后 addOption(0)/addOption(1)。
    ///
    /// `minimap_checked`：optionChecked[1]——oracle RMS 恒空 ⇒ mode 21 的
    /// m_000 case 21 catch 分支设 true（a.java:4465-4467）。
    pub fn new(sound_enabled: bool, minimap_checked: bool) -> OptionList {
        let mut list = OptionList {
            labels: Vec::new(),
            checked: Vec::new(),
            highlight: 0,
            box_w: 0,
            box_h: 0,
        };
        list.add(0, sound_enabled); // addOption(0)：checked=soundEnabled
        list.add(1, minimap_checked);
        list
    }

    /// `addOption`（a.java:9901-9922）：登记选项 + 盒尺寸累加。
    fn add(&mut self, label_idx: usize, checked: bool) {
        let font = crate::paint::paint_font();
        let w = font.string_width(OPTION_LABELS[label_idx]) + 80;
        self.labels.push(label_idx);
        self.checked.push(checked);
        if self.labels.len() == 1 {
            self.box_h = 32 + 18; // 32 + optionRowHeight(fontHeight+8)
            self.box_w = w;
        } else {
            self.box_h += 18;
            if self.box_w < w {
                self.box_w = w;
            }
        }
    }

    /// paint case 16（a.java:2953-2990）。
    pub fn paint(&self, g: &mut SoftGraphics<'_>, ui: &[ArgbImage], font: &crate::render::SoftFont) {
        let x = 240 - self.box_w - 22 >> 1;
        let mut y = 320 - self.box_h >> 1;
        paint_box_frame(g, &ui[0], x, y, self.box_w + 22, self.box_h);
        let content_x = x + 11;
        y += 16;
        let hl = (font.height + 8) * self.highlight;
        g.set_color(crate::layout::OPTION_HIGHLIGHT);
        g.fill_rect(content_x, y + hl, self.box_w, font.height + 8);
        g.draw_image(&ui[14], content_x + 10, y + 2 + hl + (font.height - 5 >> 1), 0);
        let t = crate::paint::NOKIA_TRANSFORM_TABLE[1];
        g.draw_image_transformed(
            &ui[14],
            content_x + self.box_w - 30,
            y + 2 + hl + (font.height - 5 >> 1),
            0,
            t,
        );
        for i in 0..self.labels.len() {
            let label = OPTION_LABELS[self.labels[i]];
            let mut lx = 240 - font.string_width(label) - 16 >> 1;
            g.set_color(crate::layout::WHITE);
            g.draw_string(label, lx, y + 4, 0);
            lx += 2 + font.string_width(label);
            // 勾选框：未选中 sy=12、选中 sy=0（a.java:2981-2987 分支方向）
            let sy = if !self.checked[i] { 12 } else { 0 };
            crate::paint::draw_image_clipped(g, &ui[11], lx, y + (font.height - 2 >> 1), sy, 0, 12, 10);
            y += font.height + 8;
        }
    }
}
