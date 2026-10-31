//! 菜单族（modes 8/15/16/17/22）——读档槽、帮助/关于 overlay、选项列表、
//! 帮助退出动画（第二个 logo 时间线）。
//!
//! # 结构（A 级证据）
//!
//! - **mode 8**（paint a.java:2684-2688 / run a.java:3827-3876 / dispatchModeEntry case 8
//!   a.java:4358-4377）：parallax 背景 + paintSlotList 槽位列表 + 软键 (1,3)。
//!   -2/-1 移动槽光标（slotCursor，窗口 slotScrollTop/slotScrollBottom）；-7 → menuReturnMode；
//!   槽无效（oracle RMS 恒空）时 -5 无操作。
//! - **mode 15/17**（BUG-007：标签「保存游戏/设置」实为帮助/关于）：
//!   paint case 15/17 = parallax + iconStrip(y0=10, sy=51/68) + 软键 (0,3)；
//!   dispatchModeEntry 经 showOverlayMessage(0, 大文本) 置 overlayActive——**overlay 层**
//!   （run a.java:3252-3266 case 0 的 15/17 分支）处理 -7（退出→menuReturnMode）
//!   与 -2/-1（翻页）。
//! - **mode 16**（选项列表）：paint case 16（paintBoxFrame 盒 + 高亮行 + 勾选框）；
//!   run case 16（a.java:4141-4183）：-2/-1 移动 optionCursor、-5/-4/-3 翻转
//!   optionChecked[高亮]；-7/-6 → menuReturnMode。
//! - **mode 22**（帮助→退出）：run case 22（a.java:4247-4250）++bootPhaseCounter≤70
//!   ⇒ runLogoAnimation(1, t)（py_key=1 分支 a.java:10322-10364：t=18..30 逐字
//!   上滑注册、t=50/51/52/58/63 修改项）；paint case 22 = paintLogoAnimation。
//!   70 拍后 f_bool_00=false（进程退出）。
//! - **overlay 绘制**（paint 公共尾 a.java:3129-3171）：paintBoxFrame 居中盒 +
//!   paintWrappedText 排版文本（\cRRGGBB 着色、\r 复原、翻页箭头闪烁 frameCounter&1）。
//!
//! # paint 内状态推进
//!
//! `drawParallaxBackdrop` 的 backdropScroll 在 **paint 内** --1（<-154 重置 0，
//! 周期 155）——与 Java 相同的副作用位置。

use crate::render::FontMetrics;
use crate::render::{ArgbImage, SoftGraphics};

/// parseHexColor（a.java:5159-5163）：hex 颜色解析，失败 → PARSE_DEFAULT（永不 -1）。
/// parseHexColor 的 &str 薄壳（scene 的 paintWrappedText 用）。
pub fn parse_hex_color_public(hex: &str) -> Option<u32> {
    let v = i32::from_str_radix(hex, 16).unwrap_or(crate::layout::PARSE_DEFAULT as i32);
    Some(v as u32)
}

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

/// 视差背景（drawParallaxBackdrop a.java:6848-6869）。
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
    /// backdropScroll 只读（boot→场景交接用）。
    pub fn scroll(&self) -> i32 {
        self.scroll
    }

    /// backdropScroll 写入（场景接管）。
    pub fn set_scroll(&mut self, v: i32) {
        self.scroll = v;
    }
}

impl ParallaxBackdrop {
    pub fn new() -> ParallaxBackdrop {
        ParallaxBackdrop { scroll: 0 }
    }

    /// `drawParallaxBackdrop(game_view)`（a.java:6850-6871，与 scene.rs
    /// `paint_parallax` 同一 Java 方法——列步进/回绕/行步进共用 layout 常量）。
    /// `game_view=true` 时纵向平铺带翻转（mode 3/4/19 用），菜单族恒 false。
    pub fn paint(&mut self, g: &mut SoftGraphics<'_>, tile: &ArgbImage, game_view: bool, view_bottom: i32) {
        // 【task-14 豁免+界随画布】背景=全幅层（不随 origin；铺界=画布逻辑尺寸
        // 禁 SCREEN 常量——modern 菜单背景只盖经典区=黑边根因同族）。
        let saved_origin = g.origin();
        g.set_origin(0);
        let (canvas_w, canvas_h) = g.canvas_logical();
        self.scroll -= 1;
        if self.scroll < crate::layout::PARALLAX_WRAP {
            self.scroll = 0;
        }
        let base_y = if game_view { view_bottom - crate::layout::SCREEN_H + 6 } else { 0 };
        let mut x = self.scroll;
        while x < canvas_w {
            let mut y = base_y;
            let mut flipped = false;
            while y > -canvas_h {
                if flipped {
                    let t = crate::paint::NOKIA_TRANSFORM_TABLE[2];
                    g.draw_image_transformed(tile, x, y, 0, t);
                } else {
                    g.draw_image(tile, x, y, 0);
                }
                flipped = !flipped;
                y -= tile.height;
            }
            x += crate::layout::PARALLAX_COL_STEP;
        }
        g.set_origin(saved_origin);
    }
}

// ============================== 盒框绘制 ==============================

/// `drawEdgePatch`（a.java:4577-4620）：角/边片绘制（kind 1-7 = 变换码表索引，
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
        // 【R-2 换算收敛】iw/ih = 图集**逻辑**宽高（=w/d）：镜像偏移在逻辑域推导；
        // 像素边界 ×d 由 draw_image 内部密度采样器承担（R-4 只对像素边界 ×d）。
        let (iw, ih) = (img.logical_width(), img.logical_height());
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
    g.reset_clip();
}

/// `paintBoxFrame`（a.java:6119-6153）：标准盒框（ui[8][0] 角片 26×16 / 边片 16×16）。
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
    g.reset_clip();
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
    g.reset_clip();
}

/// `paintTitledBox`（a.java:6112-6125）：盒框 + 顶部标题条（ui[8][13] 按 `segs`
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
    g.reset_clip();
}

/// `paintMiniFrame`（a.java:6196-6199）：小框（32×32 图标框底）。
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

/// `paintNumber`（a.java:6207-6230）：数字条（右对齐逐位，负数画横线）。
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
    g.reset_clip();
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
    /// overlayContentWidth：内容宽（单行 = 整串宽，多行 = 布局宽 script_cursor0）。
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

/// `layoutWrappedText` 单源核心（a.java:4940-5020；`layoutFullText` = limit=全长的
/// 委托，a.java:4940-4942）。2026-10-17 双拷贝单源化：消费方 =
/// `OverlayEngine::layout`（u16 行）与 `scene::layout_wrapped_text_limited`
/// （打字机限量）。语义锚 Java 原样（缺陷保真）：① 非法 `\c` 两字符仅记 1 行位
/// （var11++ 原样）；② 单行 contentWidth 测**全文**宽（a.java:5013-5014；scene
/// 旧拷贝测截断宽——经考证不可观测，见其原注，切换安全）；③ 尾行仅 line_len>0
/// 时 push。
pub struct WrappedLayout {
    pub lines: Vec<Vec<u16>>,
    pub total_lines: i32,
    pub lines_per_page: i32,
    pub page_height: i32,
    pub content_w: i32,
    pub line_height: i32,
}

pub fn layout_wrapped_text(
    text: &[u16],
    max_w: i32,
    max_h: i32,
    limit: usize,
    font: &dyn crate::render::FontMetrics,
) -> WrappedLayout {
    let line_height = font.height() + 4;
    let mut lines: Vec<Vec<u16>> = Vec::new();
    let mut cur_w = 0i32; // var8
    let mut line_start = 0usize; // var10
    let mut line_len = 0usize; // var11
    let n = limit.min(text.len());
    let mut i = 0usize; // var12
    while i < n {
        let ch = text[i];
        let mut cw = font.char_width(ch);
        if ch == b'\n' as u16 {
            lines.push(text[line_start..line_start + line_len].to_vec());
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
                lines.push(text[line_start..line_start + line_len].to_vec());
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
        lines.push(text[line_start..line_start + line_len].to_vec());
    }
    let total_lines = lines.len() as i32;
    let mut lines_per_page = max_h / line_height;
    if total_lines < lines_per_page {
        lines_per_page = total_lines;
    }
    let page_height = line_height * lines_per_page;
    let content_w = if total_lines == 1 { font.string_width(text) } else { max_w };
    WrappedLayout {
        lines,
        total_lines,
        lines_per_page,
        page_height,
        content_w,
        line_height,
    }
}

/// `paintWrappedText` 单源核心（a.java:5021-5119）：页首前 `\c` 色回溯 + 行内转义
/// 绘制（\c 着色 / \r 复原）+ 翻页三角。2026-10-17 双拷贝单源化：消费方 =
/// `OverlayEngine::paint` / `scene::paint_wrapped_text`。语义锚 Java 原样：
/// ① overlaySavedColor 开场置 `getColor()`、每次 `\c` 刷新、**跨行持久**
/// （engine 旧拷贝行内局部、scene 旧拷贝跨 paint 持久，皆偏离）；
/// ② 尾段 `\c` 长度不足时 parsed 保持行内初值 0（a.java:5063，缺陷保真）。
/// 返回末态 overlaySavedColor。
#[allow(clippy::too_many_arguments)]
pub fn paint_wrapped_lines(
    g: &mut SoftGraphics<'_>,
    lines: &[Vec<u16>],
    page_top: i32,
    page_bottom: i32,
    total_lines: i32,
    page_height: i32,
    line_height: i32,
    x: i32,
    y: i32,
    w: i32,
    arrows: bool,
    frame_counter: i32,
    font: &dyn crate::render::FontMetrics,
) -> u32 {
    let mut saved_color = g.get_color();
    let mut dy = y + 2;
    // 页首行之前最近的颜色标记（a.java:5043-5047，倒序扫 --var8 >= 0）
    let mut probe = page_top;
    loop {
        probe -= 1;
        if probe < 0 {
            break;
        }
        let line = &lines[probe as usize];
        if let Some(pos) = line.iter().rposition(|&c| c == b'\\' as u16) {
            if line.get(pos + 1) == Some(&(b'c' as u16)) && pos + 8 <= line.len() {
                let color = parse_hex_color(&line[pos + 2..pos + 8]);
                g.set_color(color as u32);
                break;
            }
        }
    }
    // 行绘制（a.java:5055-5076）
    let mut row = page_top;
    while row < page_bottom {
        let line = &lines[row as usize];
        let len = line.len();
        let mut seg_start = 0usize;
        let mut px = x;
        let mut parsed: i32 = 0;
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
                        if j + 6 < len {
                            parsed = parse_hex_color(&line[j + 1..j + 7]);
                        }
                        if parsed != -1 {
                            saved_color = g.get_color();
                            g.set_color(parsed as u32);
                            j += 6;
                            seg_start = j + 1;
                        } else {
                            seg_start = j - 1;
                        }
                    } else if c2 == b'r' as u16 {
                        g.set_color(saved_color);
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
        dy += line_height;
        row += 1;
    }
    // 翻页箭头（a.java:5083-5092）
    let cx = x + (w >> 1);
    if arrows {
        g.set_color(crate::layout::WHITE);
        if page_top > 0 {
            let ay = y - 8 + (frame_counter & 1);
            g.fill_triangle(cx, ay, cx - 7, ay + 7, cx + 7, ay + 7);
        }
        if page_bottom < total_lines {
            let ay = y + page_height + 3 - (frame_counter & 1);
            g.fill_triangle(cx, ay, cx - 6, ay - 6, cx + 6, ay - 6);
        }
    }
    saved_color
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
            line_height: crate::paint::paint_font().height + 4, // textLineHeight（a.java:471，fontHeight 派生）
        }
    }

    /// `layoutWrappedText`（a.java:4998-5013 区）：切行布局。`(max_w, max_h)` =
    /// showOverlayMessage case 0 的 (180, 240)。
    fn layout(&mut self, text: &[u16], max_w: i32, max_h: i32, font: &dyn crate::render::FontMetrics) {
        let r = layout_wrapped_text(text, max_w, max_h, text.len(), font);
        self.lines = r.lines;
        self.total_lines = r.total_lines;
        self.lines_per_page = r.lines_per_page;
        self.page_height = r.page_height;
        self.content_width = r.content_w;
        self.line_height = r.line_height;
        self.page_top = 0;
        self.page_bottom = r.total_lines.min(r.lines_per_page);
    }

    /// `showOverlayMessage` case 0/2/4/5（a.java:5005-5006）：布局 + 盒尺寸。
    pub fn show_kind0(&mut self, text: &[u16], font: &dyn crate::render::FontMetrics, footer_icons: (i8, i8)) {
        self.layout(text, crate::layout::OVERLAY_TEXT_W, crate::layout::OVERLAY_TEXT_MAX_H, font);
        self.box_w = self.content_width + 32;
        self.box_h = 32 + (self.page_bottom - self.page_top) * self.line_height;
        if footer_icons.0 != 0 || footer_icons.1 != 0 {
            self.box_h += 18;
        }
        self.cached_text = Some(text.to_vec());
    }

    /// 引子字幕布局（runIntroViewer 依赖行宽/总行数；布局参数与 paint 调用一致 220/28）。
    pub fn layout_for_intro(&mut self, text: &[u16], font: &dyn crate::render::FontMetrics) {
        self.layout(text, 220, (font.height() + 4) << 1, font); // max_h = textLineHeight<<1（a.java:2941）
        self.cached_text = Some(text.to_vec());
    }

    /// overlayLayoutCache != null（上次 paint 布局锚存在——runIntroViewer 滚动机门槛）。
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

    /// 【计划 3 T2 · 复核 T2-R8】页尾行（= min(top+lpp, total)，排版不变量单源）。
    pub fn page_bottom(&self) -> i32 {
        (self.page_top + self.lines_per_page).min(self.total_lines)
    }

    pub fn lines_per_page(&self) -> i32 {
        self.lines_per_page
    }

    pub fn page_top(&self) -> i32 {
        self.page_top
    }

    /// `flipOverlayPage`（a.java:5053-5066）：翻页钳制。
    pub fn flip_page(&mut self, page: i32) {
        let mut p = page;
        if p >= 0 {
            if p > self.total_lines - self.lines_per_page {
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

    /// `paintWrappedText`（a.java:5028-5076）：排版绘制（\c 着色 / \r 复原 / 翻页箭头）。
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
        font: &dyn crate::render::FontMetrics,
    ) {
        if self.cached_text.as_deref() != Some(text) {
            // paintWrappedText 缓存失效 ⇒ layoutFullText(text, w, h)（a.java:5024-5026）
            self.layout(text, w, h, font);
            self.cached_text = Some(text.to_vec());
        }
        self.saved_color = paint_wrapped_lines(
            g,
            &self.lines,
            self.page_top,
            self.page_bottom,
            self.total_lines,
            self.page_height,
            self.line_height,
            x,
            y,
            w,
            arrows,
            frame_counter,
            font,
        );
    }
}

// ============================== 槽位列表 ==============================

/// paintSlotList（a.java:9050-9155）的槽位表（oracle RMS 恒空 ⇒ 全无效）。
#[derive(Clone)]
pub struct SlotSelect {
    /// slotCursor：槽光标（0..5 环绕）。
    pub cursor: i32,
    /// slotScrollTop/slotScrollBottom：可见窗口。
    pub scroll_top: i32,
    pub scroll_bottom: i32,
    /// slotsPerPage：每页行数（dispatchModeEntry case 8 = min(6, 208/14) = 6）。
    pub rows_per_page: i32,
    /// slotListWidth：内容宽（max(148, wideCharWidth*3+60) = 148）。
    pub content_w: i32,
    /// slotListHeight：内容高（112+14*6 = 196）。
    pub content_h: i32,
    /// 槽有效性（RMS 读档；oracle 冷态恒 [false;6]）。
    pub valid: [bool; 6],
    /// markSlotEntry 详情（a.java:9157-9172）：层数/三围/三钥/金币/装备。
    pub floor_num: [i8; 6],
    pub hp: [i32; 6],
    pub atk: [i32; 6],
    pub def: [i32; 6],
    pub keys_yellow: [i16; 6],
    pub keys_blue: [i16; 6],
    pub keys_red: [i16; 6],
    pub gold: [i32; 6],
    pub weapon: [i8; 6],
    pub armor: [i8; 6],
    /// 读失败落盘触发器（BUG-021）：`load_slot_table` 失败（缺店/空/截断）置位 =
    /// Java markSlotEntry(-1) 的 delete+addRecord 整表落盘（a.java:9000-9001→9026-9052）；
    /// 壳消费 [`SlotSelect::take_parse_dirty`] 回写 MOT_IF。
    pub parse_dirty: bool,
}

impl SlotSelect {
    /// case 7/8 入口布局（a.java:4354-4366）：度量随入口重算 + 光标/滚动复位。
    /// **不动数据数组**——Java 数组仅 `!f_bool_15` 首次分配（8961-8974）、跨进入
    /// 复用 ⇒ 半截解析的"旧值"即上次表值（BUG-021 保真面；旧实现 `SlotSelect::new()`
    /// 全量重建把旧值清零 = 保真偏差，2026-10-18 修复）。
    pub fn reset_entry_layout(&mut self) {
        // 度量全部由 fontHeight 派生（a.java:4354-4364；(10+4)/12 字面量在字高 10
        // 时同值、字高 14 后与 Java 分叉——BUG-008 根因修复面）
        let font = crate::paint::paint_font();
        let lh = font.height() + 4;
        let per = crate::layout::SLOT_LIST_MAX_H / lh;
        self.rows_per_page = if per > 6 { 6 } else { per };
        self.content_w = (font.char_width('国' as u16) * 3 + 60).max(148); // wideCharWidth*3+60，钳 148
        self.content_h = 112 + lh * self.rows_per_page;
        self.cursor = 0; // slotScrollTop = slotCursor = 0（a.java:4365）
        self.scroll_top = 0;
        self.scroll_bottom = self.rows_per_page; // slotScrollBottom = slotsPerPage（4366）
    }

    pub fn new() -> SlotSelect {
        let mut s = SlotSelect {
            cursor: 0,
            scroll_top: 0,
            scroll_bottom: 0,
            rows_per_page: 0,
            content_w: 0,
            content_h: 0,
            valid: [false; 6],
            floor_num: [0; 6],
            hp: [0; 6],
            atk: [0; 6],
            def: [0; 6],
            keys_yellow: [0; 6],
            keys_blue: [0; 6],
            keys_red: [0; 6],
            gold: [0; 6],
            weapon: [0; 6],
            armor: [0; 6],
            parse_dirty: false,
        };
        s.reset_entry_layout();
        s
    }

    /// 消费 parse_dirty（markSlotEntry(-1) 落盘触发器）：壳据此把当前槽表整表
    /// 回写 MOT_IF（BUG-021 ②c 对偶）。
    pub fn take_parse_dirty(&mut self) -> bool {
        std::mem::take(&mut self.parse_dirty)
    }

    /// loadSlotTable（a.java:8959-9005；失败臂 = markSlotEntry(-1) 9000-9001→9007-9059）：
    /// MOT_IF 槽表解析（6 槽；有效槽 10 字段：floor/weapon/armor/hp/atk/def/钥×3/gold）。
    ///
    /// 【2026-10-18 更正】原注释"解析失败 → markSlotEntry(-1)（全无效冷态）"**证伪**：
    /// Java 渐进写数组、EOFException 的 RHS 先求值不赋值（该字段留**旧值**）、catch
    /// 只调 markSlotEntry(-1)——**不清数组**（9007-9024）且**整表落盘**（9026-9052）。
    /// 仅"空店/首字节即失败"才等价冷态（BUG-021，考证
    /// docs/findings/reviews/2026-10-17-mot-if-truncation-fidelity.md）。
    ///
    /// 字段级粒度（Java 同字节停）：每字段**整段在界内才提交**（readInt 读到 1-3 字节
    /// 抛 ⇒ 整字段不写），否则该字段及其后全部留旧值。返回 true=完整解析；
    /// false=读失败（缺店/空/截断）⇒ 置 [`Self::parse_dirty`]。
    pub fn load_slot_table(&mut self, mot_if: Option<&[u8]>) -> bool {
        // 缺店/空记录：openRecordStore/getRecord 抛 ⇒ markSlotEntry(-1)（不动数组）
        let Some(data) = mot_if.filter(|d| !d.is_empty()) else {
            self.parse_dirty = true;
            return false;
        };
        let mut pos = 0usize;
        let mut ok = true;
        'parse: for k in 0..6 {
            // valid 字节（a.java:8984）：RHS 先求值，读失败 ⇒ 保留旧值、停在同一字节
            if pos >= data.len() {
                ok = false;
                break;
            }
            let valid = data[pos] != 0;
            pos += 1;
            self.valid[k] = valid;
            if !valid {
                continue;
            }
            // 有效槽 10 字段逐赋值（a.java:8986-8995）
            macro_rules! read_u8_to {
                ($arr:ident) => {
                    if pos >= data.len() {
                        ok = false;
                        break 'parse;
                    }
                    self.$arr[k] = data[pos] as i8;
                    pos += 1;
                };
            }
            macro_rules! read_i32_to {
                ($arr:ident, $ty:ty) => {
                    if pos + 4 > data.len() {
                        ok = false;
                        break 'parse;
                    }
                    self.$arr[k] = i32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as $ty;
                    pos += 4;
                };
            }
            read_u8_to!(floor_num);
            read_u8_to!(weapon);
            read_u8_to!(armor);
            read_i32_to!(hp, i32);
            read_i32_to!(atk, i32);
            read_i32_to!(def, i32);
            read_i32_to!(keys_yellow, i16);
            read_i32_to!(keys_blue, i16);
            read_i32_to!(keys_red, i16);
            read_i32_to!(gold, i32);
        }
        if !ok {
            self.parse_dirty = true;
        }
        ok
    }

    /// run case 8 的 -2（a.java:3859-3862）。
    /// 行带几何（③-1 单源：paint_list 与 pointer 共调）：行首 = 盒顶+16、
    /// 行高 = textLineHeight = font.height()+4（a.java:471 字高派生）。返回 (行y, 行高)。
    /// > 【2026-10-26 更正】盒顶取 **layout 画布**高（旧 SCREEN_H ⇒ Modern 槽表留在经典
    /// > 左上区，用户报"存档UI没有居中"）；classic 代入 240×320 原值零偏离。
    pub fn row_band(&self, visible: i32, layout: &crate::layout::Layout) -> (i32, i32) {
        let font = crate::paint::paint_font();
        let y = (layout.screen_h - self.content_h) / 2 + 16;
        (y + visible * (font.height() + 4), font.height() + 4)
    }

    /// 指针选行（只动光标+滚窗，镜像 cursor_down/up 内蕴；零其他状态）。
    pub fn set_cursor(&mut self, index: i32) {
        if index < 0 || index > 5 {
            return;
        }
        self.cursor = index;
        if self.cursor >= self.scroll_bottom {
            self.scroll_bottom = self.cursor + 1;
            self.scroll_top = self.scroll_bottom - self.rows_per_page;
        } else if self.cursor < self.scroll_top {
            self.scroll_top = self.cursor;
            self.scroll_bottom = self.scroll_top + self.rows_per_page;
        }
    }

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

    /// run case 8 的 -1（a.java:3870-3873）。
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

    /// `paintSlotList(false)`（a.java:9050-9087）：列表 + 有效槽详情面板
    ///（paintSlotTable script_cursor0+=6 起全段：层数/装备框/金币/三围/三钥）。
    /// `sptarm` = f_Image_arr2_00[6]（武器/防具位图）；paintKeySlot 语义内联。
    pub fn paint(
        &self,
        g: &mut SoftGraphics<'_>,
        ui: &[ArgbImage],
        segs: &[i32],
        sptarm: &[ArgbImage],
        arrow_y: fn() -> i32,
        layout: &crate::layout::Layout,
    ) {
        let (lx, ly) = self.paint_list(g, ui, segs, layout);
        let _ = arrow_y;
        // 详情面板（a.java:9130-9150，slotValidFlags[cursor] 才画）
        if self.valid[self.cursor as usize] {
            let c = self.cursor as usize;
            let vx = lx + 6;
            let vy = ly + 8;
            if self.floor_num[c] < 51 {
                paint_number(g, &ui[18], self.floor_num[c] as i32, vx + 48, vy);
                g.draw_image(&ui[8], vx + 50, vy + 8, 0);
            }
            // 装备框段（a.java:9104-9143）：sptarm = f_Image_arr2_00[6] 原容器
            self.paint_equipment_frames(g, c, vy, layout, |i| sptarm.get(i));
            let (mut vx, vy) = self.paint_detail_stats(g, ui, c, vy, layout);
            // paintKeySlot（a.java:6046-6052）：ui[6] 槽位图 + ui[2] 数字（+40/+5）
            for (kind, count) in [(0, self.keys_yellow[c]), (1, self.keys_blue[c]), (2, self.keys_red[c])] {
                g.set_clip(vx, vy, 18, 16);
                g.draw_image(&ui[6], vx - kind * 18, vy, 0);
                g.reset_clip();
                paint_number(g, &ui[2], count as i32, vx + 40, vy + 5);
                vx += 42;
            }
        }
    }

    /// paintSlotTable 列表段（a.java:9052-9095）：标题盒（`segs` = titleBarSegments 变体）+
    /// 可见行（选中高亮条/双箭头 + "---" 或 "存档N"）+ 翻页箭头 + 双色分隔线。
    /// 返回第二条分隔线的 `(lx, ly)`（详情面板锚点）。
    ///
    /// paintSlotTable 有两条移植——boot 读档 [`Self::paint`] 与场景存档
    /// `GameScene::paint_slot_list`——本段、[`Self::paint_equipment_frames`] 与
    /// [`Self::paint_detail_stats`] 原为逐字双拷贝，现共用。
    /// 【2026-10-15 更正】装备框段曾两份**不一致**（场景侧恒画材质名、未移植
    /// a.java:9116-9123 / 9136-9143 图标分支；boot 侧甲胄钳位误作 `>9`），
    /// 已合并为 [`Self::paint_equipment_frames`] 单源。
    ///
    /// > 【2026-10-26 更正】盒/行/箭头/分隔线几何取 **layout 画布**（`screen_w/screen_h/center_x`）：
    /// > 旧 SCREEN_W/H/CENTER_X 使 Modern 槽表留在经典 240×320 左上区（Xiaomi 14 竖屏实机截图，
    /// > 用户报"存档UI没有居中"）。Modern 构图 = classic 整体平移到画布中心；classic 零偏离。
    pub fn paint_list(
        &self,
        g: &mut SoftGraphics<'_>,
        ui: &[ArgbImage],
        segs: &[i32],
        layout: &crate::layout::Layout,
    ) -> (i32, i32) {
        let font = crate::paint::paint_font();
        let _lh = font.height() + 4; // textLineHeight（a.java:471；行高现由 row_band 提供）
        let cx = layout.center_x;
        let x = (layout.screen_w - self.content_w - 22) >> 1;
        let y = (layout.screen_h - self.content_h) >> 1;
        paint_titled_box(g, &ui[0], &ui[13], segs, x, y, self.content_w + 22, self.content_h);
        for row in self.scroll_top..self.scroll_bottom {
            // 行几何 = row_band 单源（C-1 口径：paint 真调，与 pointer 共调）
            let (y, lh) = self.row_band(row - self.scroll_top, layout);
            if row != self.cursor {
                g.set_color(crate::layout::SLOT_TEXT);
            } else {
                g.set_color(crate::layout::DARK_BACKDROP);
                g.fill_rect(cx - (self.content_w >> 1), y, self.content_w, lh);
                g.draw_image(&ui[14], cx - (self.content_w >> 1) + 10, y + ((font.height() - 8) >> 1), 0);
                let t = crate::paint::NOKIA_TRANSFORM_TABLE[1];
                g.draw_image_transformed(&ui[14], cx + (self.content_w >> 1) - 30, y + ((font.height() - 8) >> 1), 0, t);
                g.set_color(crate::layout::SLOT_TEXT_ACTIVE);
            }
            if !self.valid[row as usize] {
                g.draw_string(DASH_LABEL, cx, y + 2, 17);
            } else {
                let mut label: Vec<u16> = SAVE_LABEL.to_vec();
                label.extend((row as u16).to_string().chars().map(|c| c as u16));
                g.draw_string(&label, cx, y + 2, 17);
            }
            // 行 y 由 row_band 逐行定位（原 y += lh 累加已删）
        }
        // 行后锚 = row_band(行数).0（原 y 累计终点——箭头/分隔线/详情段锚定于此；
        // 2026-10-18 整改：单源派生，恢复下游链）
        let y = self.row_band(self.scroll_bottom - self.scroll_top, layout).0;
        // 翻页箭头（a.java:9080-9087）
        let ax = cx + ((self.content_w >> 1) - 10);
        if self.scroll_top > 0 {
            crate::paint::draw_image_clipped(g, &ui[15], ax, y - 20, 0, 0, 7, 9);
        }
        if self.scroll_bottom < 6 {
            crate::paint::draw_image_clipped(g, &ui[15], ax, y - 10, 7, 0, 7, 9);
        }
        // 分隔线（a.java:9089-9095）
        let mut ly = y + 2;
        let lx = ax - self.content_w + 10;
        g.set_color(crate::layout::SLOT_RULE_DARK);
        g.draw_line(lx, ly, lx + self.content_w - 1, ly);
        ly += 1;
        g.set_color(crate::layout::SLOT_RULE_LIGHT);
        g.draw_line(lx, ly, lx + self.content_w - 1, ly);
        (lx, ly)
    }

    /// paintSlotTable 装备框段（a.java:9104-9143）：槽 `c` 的武器框 + 甲框（`vy` = 装备框行 y）。
    /// 两条移植共用（见 [`Self::paint_list`]）。
    ///
    /// `sptarm(i)` = `f_Image_arr2_00[6][i]`（boot 侧传原容器；场景侧经
    /// [`SPTARM_ENTITY_TYPES`] 从 entityTypeImage 取同一位图）。
    pub fn paint_equipment_frames<'a>(
        &self,
        g: &mut SoftGraphics<'_>,
        c: usize,
        vy: i32,
        layout: &crate::layout::Layout,
        sptarm: impl Fn(usize) -> Option<&'a ArgbImage>,
    ) {
        // a.java:9104-9105：script_cursor0 = 240 + slotListWidth >> 1; script_cursor0 -= 68
        //（240 = 屏宽 ⇒ layout.screen_w；【2026-10-26 更正】同 paint_list）
        let vx = ((layout.screen_w + self.content_w) >> 1) - 68;
        // 武器 f_byte_arr_25（a.java:9106-9123）：v-33 > 7 ⇒ 钳 8
        paint_equipment_frame(g, vx, vy, self.weapon[c], 8, &sptarm);
        // 甲 f_byte_arr_26（a.java:9125-9143）：script_cursor0 += 34；v-33 > 7 ⇒ 钳 9
        paint_equipment_frame(g, vx + 34, vy, self.armor[c], 9, &sptarm);
    }

    /// paintSlotTable 详情面板中段（a.java:9145-9169）：金币行 + HP/攻/防 三围框（槽 `c`）。
    /// `vy` = 装备框行 y；返回三钥行起点 `(vx, vy)`。两条移植共用（见 [`Self::paint_list`]）。
    pub fn paint_detail_stats(
        &self,
        g: &mut SoftGraphics<'_>,
        ui: &[ArgbImage],
        c: usize,
        vy: i32,
        layout: &crate::layout::Layout,
    ) -> (i32, i32) {
        let mut vx = (layout.screen_w - self.content_w + 10) >> 1;
        let mut vy = vy + 20;
        g.draw_image(&ui[1], vx, vy, 0);
        vx += 60;
        paint_number(g, &ui[2], self.gold[c], vx, vy + 2);
        vx -= 64;
        vy += 16;
        crate::paint::draw_image_clipped(g, &ui[7], vx, vy, 0, 2, 10, 10);
        vx += 15;
        paint_mini_frame(g, vx - 1, vy, 48, 12);
        paint_number(g, &ui[2], self.hp[c], vx + 45, vy + 2);
        vx += 47;
        crate::paint::draw_image_clipped(g, &ui[7], vx, vy, 10, 0, 10, 13);
        vx += 11;
        g.set_color(512);
        paint_mini_frame(g, vx, vy, 30, 12);
        paint_number(g, &ui[2], self.atk[c], vx + 28, vy + 2);
        vx += 32;
        crate::paint::draw_image_clipped(g, &ui[7], vx, vy, 20, 2, 10, 10);
        vx += 12;
        g.set_color(512);
        paint_mini_frame(g, vx, vy, 30, 12);
        paint_number(g, &ui[2], self.def[c], vx + 28, vy + 2);
        (((layout.screen_w - self.content_w) >> 1) + 6, vy + 16)
    }
}

/// "---"（无效槽标签，a.java:9070）。
const DASH_LABEL: &[u16] = &[0x2D, 0x2D, 0x2D];
/// "存档"（有效槽标签前缀，a.java:9072）。
const SAVE_LABEL: &[u16] = &[0x5B58, 0x6863];

/// `f_Image_arr2_00[6][i]`（sptarm）↔ entityTypeImage 下标：m_122 是两者唯一
/// 赋值点——`[33..=40] = [6][t-33]`（a.java:9462-9464）、`[79] = [6][8]`、
/// `[80] = [6][9]`（a.java:9478-9479）。场景侧 [`crate::scene::SceneImages::entity`]
/// 即该映射产物（`map_entity_images`），按本表反查即得 sptarm 原图。
pub const SPTARM_ENTITY_TYPES: [usize; 10] = [33, 34, 35, 36, 37, 38, 39, 40, 79, 80];

/// paintSlotTable 单个装备框（a.java:9106-9123 武器 / 9126-9143 甲）：`paintMiniFrame(32×32)`
/// 后按槽值 `v` 二分——
/// - `v == 0`：`setColor(-1)` + `drawString(equipmentMaterialNames[v])`（本支恒 [0]
///   =“无”），偏移 `(32-wideCharWidth>>1, 32-fontHeight>>1)`（a.java:9107-9115）；
/// - 否则：`i = v-33`（byte→int 提升，无溢出）；`i > 7 ⇒ i = clamp_to`（武器 8 /
///   甲 9）；`drawImage([6][i])` 居中 `(32-w>>1, 32-h>>1)`（a.java:9116-9123）。
///
/// `i < 0`（v ∈ 1..=32 或负值）/ 容器缺图：Java 抛 AIOOBE（paint 中断）——仅损坏
/// 存档可达（玩法写入值 ∈ {0,33..=40,79,80}，equipTierTypes）；此处不画图标、不 panic
/// （与 HUD 装备槽 `.get()` 兜底同口径）。
fn paint_equipment_frame<'a>(
    g: &mut SoftGraphics<'_>,
    vx: i32,
    vy: i32,
    v: i8,
    clamp_to: i32,
    sptarm: &impl Fn(usize) -> Option<&'a ArgbImage>,
) {
    paint_mini_frame(g, vx, vy, 32, 32);
    if v == 0 {
        let font = crate::paint::paint_font();
        let wide = font.char_width('国' as u16); // wideCharWidth（a.java:25）
        g.set_color(-1i32 as u32);
        let name: Vec<u16> = crate::scene::EQUIPMENT_MATERIAL_NAMES[v as usize].encode_utf16().collect();
        g.draw_string(&name, vx + ((32 - wide) >> 1), vy + ((32 - font.height()) >> 1), 0);
    } else {
        let mut i = v as i32 - 33;
        if i > 7 {
            i = clamp_to;
        }
        if let Some(img) = usize::try_from(i).ok().and_then(sptarm) {
            g.draw_image(img, vx + ((32 - img.width) >> 1), vy + ((32 - img.height) >> 1), 0);
        }
    }
}

// ============================== 菜单族状态机 ==============================

/// 装备框像素断言工具（单测共用：本模块 + scene 的场景侧接线测试）。
///
/// 判据全部由 Java 公式直接推出（a.java:9104-9143），不经被测函数：
/// 框 = paintMiniFrame(32×32) 的填充区 `[fx+1, fx+31) × [fy, fy+31)`；
/// 图标 = sptarm[k] 左上 `(fx + (32-w>>1), fy + (32-h>>1))`；
/// 材质名 = 白字“无”@`(fx + (32-wideCharWidth>>1), fy + (32-fontHeight>>1))`
/// （wideCharWidth=12、fontHeight=14——2026-10-15 字高更正后 32-14>>1=9）。

pub(crate) mod slot_equip_test_support {
    use crate::layout::{MINIFRAME_FILL, SCREEN_H, SCREEN_W};
    use crate::render::{ArgbImage, SoftGraphics};

    /// 合成 sptarm 的逐张纯色（互异，且异于框填充色/白字/黑底）。
    pub const ICON_COLORS: [u32; 10] = [
        0xFFFF0000, 0xFF00FF00, 0xFF0000FF, 0xFFFFFF00, 0xFFFF00FF, 0xFF00FFFF, 0xFF800000,
        0xFF008000, 0xFF000080, 0xFF808000,
    ];
    pub const FILL: u32 = 0xFF000000 | MINIFRAME_FILL;
    pub const WHITE: u32 = 0xFFFFFFFF;
    /// 装备框行 y（SlotSelect::new 布局：box_y=(320-content_h)>>1、+16+per*lh、
    /// 分隔线 +3、+8；content_h=112+lh*per，lh=fontHeight+4=18、per=6 ⇒ 185）。
    pub const FRAME_Y: i32 = 185;
    /// 武器框 x：(240 + slotListWidth(148) >> 1) - 68 = 126；甲框 +34 = 160。
    pub const WEAPON_X: i32 = 126;
    pub const ARMOR_X: i32 = 160;

    /// 合成 sptarm 容器：第 i 张 `(11+2i) × (25-i)` 纯色——宽恒奇数（验 `>>1` 向下取整），
    /// 尺寸逐张不同（居中偏移逐张可辨）。
    pub fn synthetic_sptarm() -> Vec<ArgbImage> {
        (0..10)
            .map(|i| {
                let (w, h) = (11 + 2 * i as i32, 25 - i as i32);
                ArgbImage::from_rgb(&vec![ICON_COLORS[i]; (w * h) as usize], w, h, true)
            })
            .collect()
    }

    pub fn px(img: &ArgbImage, x: i32, y: i32) -> u32 {
        img.argb[(y * img.width + x) as usize]
    }

    /// 框填充区内：图标矩形逐像素 = ICON_COLORS[k]，其余 = 框填充色。
    pub fn assert_frame_icon(screen: &ArgbImage, fx: i32, fy: i32, icons: &[ArgbImage], k: usize, what: &str) {
        let (w, h) = (icons[k].width, icons[k].height);
        let (ox, oy) = (fx + ((32 - w) >> 1), fy + ((32 - h) >> 1));
        for y in fy..fy + 31 {
            for x in fx + 1..fx + 31 {
                let inside = x >= ox && x < ox + w && y >= oy && y < oy + h;
                let expect = if inside { ICON_COLORS[k] } else { FILL };
                assert_eq!(px(screen, x, y), expect, "{what}: sptarm[{k}] 框({fx},{fy}) 像素({x},{y})");
            }
        }
    }

    /// Java 参照（a.java:9106-9115 逐句）：paintMiniFrame + setColor(-1) +
    /// drawString(“无”, fx+(32-12>>1), fy+(32-10>>1), 0)——与被测画面的框区逐像素相等；
    /// 另断言白字确实落笔且框内无任何图标色。
    pub fn assert_frame_material_name(screen: &ArgbImage, fx: i32, fy: i32, what: &str) {
        use crate::render::FontMetrics;
        let mut reference = ArgbImage::create(SCREEN_W, SCREEN_H);
        {
            let font = crate::paint::paint_font();
            let mut g = SoftGraphics::new(&mut reference);
            g.set_font(Some(font));
            crate::menu_family::paint_mini_frame(&mut g, fx, fy, 32, 32);
            g.set_color(0xFFFFFF);
            let wide = font.char_width('国' as u16);
            g.draw_string(&[0x65E0], fx + ((32 - wide) >> 1), fy + ((32 - font.height()) >> 1), 0); // “无”
        }
        let mut white = 0;
        for y in fy..fy + 31 {
            for x in fx + 1..fx + 31 {
                let p = px(screen, x, y);
                assert_eq!(p, px(&reference, x, y), "{what}: 材质名框({fx},{fy}) 像素({x},{y})");
                assert!(!ICON_COLORS.contains(&p), "{what}: 材质名框内出现图标色 ({x},{y})");
                if p == WHITE {
                    white += 1;
                }
            }
        }
        assert!(white > 0, "{what}: 材质名白字未落笔");
    }

    /// 框填充区全为填充色（无字无图）。
    pub fn assert_frame_empty(screen: &ArgbImage, fx: i32, fy: i32, what: &str) {
        for y in fy..fy + 31 {
            for x in fx + 1..fx + 31 {
                assert_eq!(px(screen, x, y), FILL, "{what}: 空框({fx},{fy}) 像素({x},{y})");
            }
        }
    }
}

#[cfg(test)]
mod slot_equip_tests {
    use crate::layout::{SCREEN_H, SCREEN_W};
    use crate::render::FontMetrics;
    use super::slot_equip_test_support::*;
    use super::*;

    /// 只画装备框段（槽 0），y 取 FRAME_Y。
    fn render_frames(weapon: i8, armor: i8, icons: &[ArgbImage]) -> ArgbImage {
        let mut slot = SlotSelect::new();
        slot.valid[0] = true;
        slot.weapon[0] = weapon;
        slot.armor[0] = armor;
        let mut screen = ArgbImage::create(SCREEN_W, SCREEN_H);
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_font(Some(crate::paint::paint_font()));
            slot.paint_equipment_frames(&mut g, 0, FRAME_Y, &crate::layout::Layout::classic(), |i| icons.get(i));
        }
        screen
    }

    /// 框 x 公式（a.java:9104-9105 + 9125）：SlotSelect::new 宽 148 ⇒ 126 / 160。
    #[test]
    fn equip_frame_x_follows_java_formula() {
        let s = SlotSelect::new();
        assert_eq!(((SCREEN_W + s.content_w) >> 1) - 68, WEAPON_X);
        assert_eq!(WEAPON_X + 34, ARMOR_X);
    }

    /// 装备 = 0（a.java:9107-9115 / 9127-9135）：两框均白字“无”，无图标。
    #[test]
    fn equip_zero_draws_material_name() {
        let icons = synthetic_sptarm();
        let screen = render_frames(0, 0, &icons);
        assert_frame_material_name(&screen, WEAPON_X, FRAME_Y, "武器=0");
        assert_frame_material_name(&screen, ARMOR_X, FRAME_Y, "甲=0");
    }

    /// 装备 ≠ 0（a.java:9116-9123 / 9136-9143）：sptarm[v-33] 居中；框内无白字。
    #[test]
    fn equip_nonzero_draws_centered_sptarm_icon() {
        let icons = synthetic_sptarm();
        let screen = render_frames(34, 38, &icons);
        assert_frame_icon(&screen, WEAPON_X, FRAME_Y, &icons, 1, "武器=34");
        assert_frame_icon(&screen, ARMOR_X, FRAME_Y, &icons, 5, "甲=38");
    }

    /// 一框 0、一框非 0 互不串扰（场景存档后的典型态：无武器 + 80 甲）。
    #[test]
    fn equip_mixed_zero_and_icon() {
        let icons = synthetic_sptarm();
        let screen = render_frames(0, 80, &icons);
        assert_frame_material_name(&screen, WEAPON_X, FRAME_Y, "武器=0");
        assert_frame_icon(&screen, ARMOR_X, FRAME_Y, &icons, 9, "甲=80");
        let screen = render_frames(79, 0, &icons);
        assert_frame_icon(&screen, WEAPON_X, FRAME_Y, &icons, 8, "武器=79");
        assert_frame_material_name(&screen, ARMOR_X, FRAME_Y, "甲=0");
    }

    /// 下标钳位（a.java:9117-9118 武器 `>7 ⇒ 8` / 9137-9138 甲 `>7 ⇒ 9`）。
    /// 甲 41 是 `>7` 与旧 boot 误植 `>9` 唯一分歧值（i=8：`>7` ⇒ 9，`>9` ⇒ 8）；
    /// 40（i=7）为边界不钳。
    #[test]
    fn equip_index_clamp_matches_java() {
        let icons = synthetic_sptarm();
        for (v, k) in [(33i8, 0usize), (36, 3), (40, 7), (41, 8), (42, 8), (79, 8), (80, 8), (127, 8)] {
            let screen = render_frames(v, 0, &icons);
            assert_frame_icon(&screen, WEAPON_X, FRAME_Y, &icons, k, &format!("武器={v}"));
        }
        for (v, k) in [(37i8, 4usize), (40, 7), (41, 9), (42, 9), (43, 9), (79, 9), (80, 9), (127, 9)] {
            let screen = render_frames(0, v, &icons);
            assert_frame_icon(&screen, ARMOR_X, FRAME_Y, &icons, k, &format!("甲={v}"));
        }
    }

    /// v-33 < 0（v ∈ 1..=32 / 负值；Java AIOOBE，仅损坏存档可达）：只画空框、不 panic
    /// ——含 -128（旧 boot 版 i8 减法在 overflow-checks 下 panic）。
    #[test]
    fn equip_negative_index_draws_frame_only() {
        let icons = synthetic_sptarm();
        for (w, a) in [(1i8, 32i8), (5, -1), (-128, -34)] {
            let screen = render_frames(w, a, &icons);
            assert_frame_empty(&screen, WEAPON_X, FRAME_Y, &format!("武器={w}"));
            assert_frame_empty(&screen, ARMOR_X, FRAME_Y, &format!("甲={a}"));
        }
    }

    /// 场景侧取图口径：SPTARM_ENTITY_TYPES 是 m_122 映射（map_entity_images）的逆——
    /// entityTypeImage[SPTARM_ENTITY_TYPES[i]] 与 sptarm[i] 同图（a.java:9462-9464/9478-9479）。
    #[test]
    fn sptarm_entity_types_invert_m122_mapping() {
        let icons = synthetic_sptarm();
        let filler = |n: usize| -> Vec<ArgbImage> {
            (0..n).map(|_| ArgbImage::from_rgb(&[0xFF101010; 4], 2, 2, true)).collect()
        };
        let entity =
            crate::scene::SceneImages::map_entity_images(&filler(13), &filler(23), &icons, &filler(20), &filler(20));
        for (i, &t) in SPTARM_ENTITY_TYPES.iter().enumerate() {
            assert_eq!(entity[t].as_ref(), Some(&icons[i]), "entity[{t}] ≠ sptarm[{i}]");
        }
    }

    /// boot 侧接线（SlotSelect::paint 全段，mode 8）：详情面板 vy 锚点 + 原容器取图。
    #[test]
    fn boot_slot_paint_wires_equipment_frames() {
        let icons = synthetic_sptarm();
        let ui: Vec<ArgbImage> =
            (0..25).map(|_| ArgbImage::from_rgb(&vec![0xFF202020; 24 * 24], 24, 24, true)).collect();
        let mut slot = SlotSelect::new();
        slot.valid[0] = true;
        slot.weapon[0] = 35;
        slot.armor[0] = 41;
        slot.valid[1] = true;
        let paint = |slot: &SlotSelect| {
            let mut screen = ArgbImage::create(SCREEN_W, SCREEN_H);
            {
                let mut g = SoftGraphics::new(&mut screen);
                g.set_font(Some(crate::paint::paint_font()));
                slot.paint(&mut g, &ui, &[3, 5], &icons, || 0, &crate::layout::Layout::classic());
            }
            screen
        };
        let screen = paint(&slot);
        assert_frame_icon(&screen, WEAPON_X, FRAME_Y, &icons, 2, "boot 武器=35");
        assert_frame_icon(&screen, ARMOR_X, FRAME_Y, &icons, 9, "boot 甲=41（>7 钳 9）");
        slot.cursor_down(); // 槽 1：装备 0/0
        let screen = paint(&slot);
        assert_frame_material_name(&screen, WEAPON_X, FRAME_Y, "boot 武器=0");
        assert_frame_material_name(&screen, ARMOR_X, FRAME_Y, "boot 甲=0");
    }
}

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
    /// 【STAGE-B-006】壳层附加行：排在原两行之后（空 = 原版两行）；原两行的 `labels` / `checked` 不含它们。
    shell: Vec<ShellOption>,
}

/// 【STAGE-B-006】壳层附加设置行：core 只负责显示、切换、交接，不解释含义（`id` 由壳定义）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellOption {
    pub id: u8,
    /// 行文字（UTF-16，字体绘制，不需美术）。
    pub label: Vec<u16>,
    pub checked: bool,
}

/// 【STAGE-B-006】按 id 回写机器侧附加行的勾选（两处设置页入口共用；未知 id 不动）。
pub fn set_shell_option(rows: &mut [ShellOption], id: u8, checked: bool) {
    for r in rows.iter_mut().filter(|r| r.id == id) {
        r.checked = checked;
    }
}

#[cfg(test)]
mod option_tests {
    use crate::render::FontMetrics;
    use super::*;

    #[test]
    fn checkbox_x_math() {
        let font = crate::paint::paint_font();
        for (idx, expect_lx, expect_cx) in [(0usize, 100i32, 126i32), (1, 94, 132)] {
            let label = OPTION_LABELS[idx];
            let w = font.string_width(label);
            let lx = (240 - w - 16) >> 1;
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
    /// dispatchModeEntry case 16（a.java:4412-4414）：optionCount=0 后 addOption(0)/addOption(1)。
    ///
    /// `minimap_checked`：optionChecked[1]——oracle RMS 恒空 ⇒ mode 21 的
    /// dispatchModeEntry case 21 catch 分支设 true（a.java:4475-4471）。
    pub fn new(sound_enabled: bool, minimap_checked: bool) -> OptionList {
        Self::with_shell_rows(sound_enabled, minimap_checked, &[])
    }

    /// 【STAGE-B-006】原两行 + 壳层附加行（`shell` 空 ⇒ 与 [`Self::new`] 相同）。附加行盒尺寸按 addOption
    /// 非首行同式累加（[`Self::grow_box`]）。
    pub fn with_shell_rows(sound_enabled: bool, minimap_checked: bool, shell: &[ShellOption]) -> OptionList {
        let mut list = OptionList {
            labels: Vec::new(),
            checked: Vec::new(),
            highlight: 0,
            box_w: 0,
            box_h: 0,
            shell: Vec::new(),
        };
        list.add(0, sound_enabled); // addOption(0)：checked=soundEnabled
        list.add(1, minimap_checked);
        for row in shell {
            list.grow_box(&row.label);
            list.shell.push(row.clone());
        }
        list
    }

    /// `addOption`（a.java:9911-9926）：登记选项 + 盒尺寸累加。
    fn add(&mut self, label_idx: usize, checked: bool) {
        let font = crate::paint::paint_font();
        self.labels.push(label_idx);
        self.checked.push(checked);
        if self.labels.len() == 1 {
            self.box_h = 32 + (font.height() + 8); // optionRowHeight = fontHeight+8（a.java:812）
            self.box_w = font.string_width(OPTION_LABELS[label_idx]) + 80;
        } else {
            self.grow_box(OPTION_LABELS[label_idx]);
        }
    }

    /// addOption 非首行的盒尺寸累加（a.java:9911-9926）：盒高 += 行高；盒宽 = max(盒宽, 标签宽 + 80)。
    fn grow_box(&mut self, label: &[u16]) {
        let font = crate::paint::paint_font();
        let w = font.string_width(label) + 80;
        self.box_h += font.height() + 8;
        if self.box_w < w {
            self.box_w = w;
        }
    }

    /// 总行数（原行 + 壳层附加行；指针命中遍历 / 高亮环绕用）。
    pub fn len(&self) -> usize {
        self.labels.len() + self.shell.len()
    }

    /// 当前高亮（指针选行断言用）。
    pub fn highlight_index(&self) -> usize {
        self.highlight as usize
    }

    /// 设高亮（指针选行；只动光标语义）。
    pub fn set_highlight(&mut self, i: usize) {
        if i < self.len() {
            self.highlight = i as _;
        }
    }

    /// 【STAGE-B-006】第 i 行若是壳层附加行 ⇒ 翻转并返回 `(id, 新勾选)`；原行 / 越界 ⇒ None（不动任何状态）。
    pub fn toggle_shell_row(&mut self, i: usize) -> Option<(u8, bool)> {
        let row = self.shell.get_mut(i.checked_sub(self.labels.len())?)?;
        row.checked = !row.checked;
        Some((row.id, row.checked))
    }

    /// 【STAGE-B-006】壳层附加行（只读）。
    pub fn shell_rows(&self) -> &[ShellOption] {
        &self.shell
    }

    /// 第 i 行的（标签, 勾选）：原行取 optionLabels / optionChecked，其后为壳层附加行。
    fn row(&self, i: usize) -> (&[u16], bool) {
        match self.labels.get(i) {
            Some(&idx) => (OPTION_LABELS[idx], self.checked[i]),
            None => {
                let r = &self.shell[i - self.labels.len()];
                (&r.label, r.checked)
            }
        }
    }

    /// 行带几何（③-1 单源：paint 与 pointer_hit 共调）：行高 = optionRowHeight
    /// = font.height()+8（a.java:812）；行首 = 盒顶+16。返回 (行y, 行高)。
    /// > 【2026-10-26 更正】盒顶取 **layout 画布**高（同 SlotSelect::row_band）。
    pub fn row_band(&self, i: usize, layout: &crate::layout::Layout) -> (i32, i32) {
        let font = crate::paint::paint_font();
        let y = (layout.screen_h - self.box_h) / 2 + 16;
        (y + (font.height() + 8) * i as i32, font.height() + 8)
    }

    /// paint case 16（a.java:2959-2990）。
    /// > 【2026-10-26 更正】盒/标签几何取 **layout 画布**（240 = 屏宽 ⇒ layout.screen_w；
    /// > 320 = 屏高 ⇒ layout.screen_h）：Modern 构图 = classic 平移到画布中心；classic 零偏离。
    pub fn paint(
        &self,
        g: &mut SoftGraphics<'_>,
        ui: &[ArgbImage],
        font: &dyn crate::render::FontMetrics,
        layout: &crate::layout::Layout,
    ) {
        let x = (layout.screen_w - self.box_w - 22) >> 1;
        let y = (layout.screen_h - self.box_h) >> 1;
        paint_box_frame(g, &ui[0], x, y, self.box_w + 22, self.box_h);
        let content_x = x + 11;
        // 行几何 = row_band 单源（C-1 整改：paint 真调，根除公式双份）
        let (y0, row_h) = self.row_band(0, layout);
        let hl = row_h * self.highlight;
        g.set_color(crate::layout::OPTION_HIGHLIGHT);
        g.fill_rect(content_x, y0 + hl, self.box_w, row_h);
        g.draw_image(&ui[14], content_x + 10, y0 + 2 + hl + ((font.height() - 5) >> 1), 0);
        let t = crate::paint::NOKIA_TRANSFORM_TABLE[1];
        g.draw_image_transformed(
            &ui[14],
            content_x + self.box_w - 30,
            y0 + 2 + hl + ((font.height() - 5) >> 1),
            0,
            t,
        );
        for i in 0..self.len() {
            let (label, checked) = self.row(i); // 【STAGE-B-006】原两行之后接壳层附加行，同式绘制
            let mut lx = (layout.screen_w - font.string_width(label) - 16) >> 1;
            let (ry, _) = self.row_band(i, layout); // C-1：行 y 单源
            g.set_color(crate::layout::WHITE);
            g.draw_string(label, lx, ry + 4, 0);
            lx += 2 + font.string_width(label);
            // 勾选框：未选中 sy=12、选中 sy=0（a.java:2987-2987 分支方向）
            let sy = if !checked { 12 } else { 0 };
            crate::paint::draw_image_clipped(g, &ui[11], lx, ry + ((font.height() - 2) >> 1), sy, 0, 12, 10);
        }
    }
}

#[cfg(test)]
mod shell_row_tests {
    //! 【STAGE-B-006】设置页壳层附加行（设计件 docs/superpowers/specs/2026-10-28-settings-splash-key-feedback-design.md §2.1）。
    use super::*;
    use crate::layout::Layout;

    fn ui() -> Vec<ArgbImage> {
        (0..25).map(|_| ArgbImage::from_rgb(&vec![0xFF202020; 24 * 24], 24, 24, true)).collect()
    }

    fn ops(list: &OptionList, layout: &Layout) -> Vec<String> {
        let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
        let mut g = SoftGraphics::new(&mut img);
        g.set_font(Some(crate::paint::paint_font()));
        list.paint(&mut g, &ui(), &crate::paint::paint_font(), layout);
        g.ops.clone()
    }

    fn digest(v: &[String]) -> String {
        game_platform::hash::sha256_hex(v.join("\n").as_bytes())[..32].to_string()
    }

    /// 零附加行 ⇒ op 流与改前逐字相同。期望 = 改前（HEAD ba7e92e）实现实测摘要：重构不变量，非被测自证。
    #[test]
    fn zero_shell_rows_paint_matches_pre_change_ops() {
        let cases = [
            (true, false, 0, Layout::classic(), "cccf05f1a160709ba357d9787d21e497"),
            (false, true, 1, Layout::classic(), "2b0580dca5c873bc8463a65d1961d05d"),
            (true, true, 1, Layout::modern(12, 16), "d17849443b103d788e5d2ee7004b1b29"),
        ];
        let got: Vec<(String, String)> = cases
            .iter()
            .map(|(sound, minimap, hl, layout, _)| {
                let mut a = OptionList::new(*sound, *minimap);
                a.highlight = *hl;
                let mut b = OptionList::with_shell_rows(*sound, *minimap, &[]);
                b.highlight = *hl;
                (digest(&ops(&a, layout)), digest(&ops(&b, layout)))
            })
            .collect();
        for ((sound, minimap, hl, _, expect), (via_new, via_empty)) in cases.iter().zip(got) {
            assert_eq!(via_new, *expect, "new() sound={sound} minimap={minimap} hl={hl}");
            assert_eq!(via_empty, *expect, "with_shell_rows(&[]) sound={sound} minimap={minimap} hl={hl}");
        }
    }

    fn rows() -> Vec<ShellOption> {
        [(1u8, "开屏画面", false), (2, "按键震动", true), (3, "按键音", true)]
            .into_iter()
            .map(|(id, l, checked)| ShellOption { id, label: l.encode_utf16().collect(), checked })
            .collect()
    }

    fn utf16(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    /// 附加行几何 = 原行同式（addOption a.java:9911-9926 / paint case 16 a.java:2959-2990）：盒高 = 32 + 5×行高、
    /// 盒宽 = 最宽标签 + 80、行 i 顶 = 盒顶 + 16 + 行高×i；标签与勾选框逐行同式。
    #[test]
    fn shell_rows_extend_box_rows_and_paint() {
        use crate::render::FontMetrics;
        let font = crate::paint::paint_font();
        let layout = Layout::modern(12, 16);
        let mut list = OptionList::with_shell_rows(true, false, &rows());
        assert_eq!(list.len(), 5);
        let row_h = font.height() + 8;
        let box_h = 32 + 5 * row_h;
        let top = (layout.screen_h - box_h) / 2 + 16;
        for i in 0..5 {
            assert_eq!(list.row_band(i, &layout), (top + row_h * i as i32, row_h), "行 {i}");
        }
        let widest = ["声音", "小地图", "开屏画面", "按键震动", "按键音"]
            .iter()
            .map(|s| font.string_width(&utf16(s)))
            .max()
            .unwrap();
        let box_w = widest + 80;
        list.highlight = 4;
        let got = ops(&list, &layout);
        let cx = ((layout.screen_w - box_w - 22) >> 1) + 11;
        assert!(
            got.iter().any(|o| o.starts_with(&format!("fillRect({cx},{},{box_w},{row_h},", top + row_h * 4))),
            "高亮条在第 5 行、宽 = 盒宽"
        );
        for (i, (label, checked)) in [("开屏画面", false), ("按键震动", true), ("按键音", true)].into_iter().enumerate() {
            let w = font.string_width(&utf16(label));
            let lx = (layout.screen_w - w - 16) >> 1;
            let ry = top + row_h * (i as i32 + 2);
            // op 文本里非 ASCII 记作 \uXXXX（render.rs quote_utf16）
            let quoted: String = utf16(label).iter().map(|c| format!("\\u{c:04x}")).collect();
            let at = got
                .iter()
                .position(|o| *o == format!("drawString(\"{quoted}\",{lx},{},0)", ry + 4))
                .unwrap_or_else(|| panic!("{label} 标签位置"));
            let (bx, by) = (lx + 2 + w, ry + ((font.height() - 2) >> 1));
            assert_eq!(got[at + 1], format!("setClip({bx},{by},12,10)"), "{label} 勾选框位置");
            let sel = if checked { 0 } else { 12 };
            assert_eq!(got[at + 2], format!("drawImage(24x24,{},{by},0)", bx - sel), "{label} 勾选态");
        }
    }

    #[test]
    fn toggle_shell_row_flips_only_shell_rows() {
        let mut list = OptionList::with_shell_rows(true, false, &rows());
        assert_eq!(list.toggle_shell_row(0), None, "声音行不归附加行");
        assert_eq!(list.toggle_shell_row(1), None, "小地图行不归附加行");
        assert_eq!(list.checked, vec![true, false], "原两行勾选不动");
        assert_eq!(list.toggle_shell_row(2), Some((1, true)));
        assert_eq!(list.toggle_shell_row(4), Some((3, false)));
        assert_eq!(list.toggle_shell_row(5), None, "越界");
        let r = list.shell_rows();
        assert_eq!((r[0].checked, r[1].checked, r[2].checked), (true, true, false));
        list.set_highlight(4);
        assert_eq!(list.highlight_index(), 4, "附加行可选中");
        list.set_highlight(5);
        assert_eq!(list.highlight_index(), 4, "越界不选");
    }

    /// 翻附加行不碰原两行（`labels` / `checked` 是 SKY_WAR [0][1] 位的来源：BootMachine::global_snapshot）与高亮——
    /// 每行连翻两次，true / false 各写一遍，任何旁写都会暴露（复核 T1-F1）。
    #[test]
    fn toggling_shell_rows_never_touches_original_rows_or_highlight() {
        for (sound, minimap) in [(true, false), (false, true)] {
            let mut list = OptionList::with_shell_rows(sound, minimap, &rows());
            list.highlight = 1;
            let before = (list.labels.clone(), list.checked.clone(), list.highlight);
            assert_eq!(before, (vec![0, 1], vec![sound, minimap], 1), "附加行不进原两行");
            for i in 2..5 {
                for _ in 0..2 {
                    assert!(list.toggle_shell_row(i).is_some());
                    assert_eq!((list.labels.clone(), list.checked.clone(), list.highlight), before, "行 {i}");
                }
            }
            assert_eq!(list.shell_rows(), &rows()[..], "连翻两次回原值");
        }
    }

    #[test]
    fn set_shell_option_writes_by_id() {
        let mut r = rows();
        set_shell_option(&mut r, 2, false);
        set_shell_option(&mut r, 9, true); // 未知 id：不动
        assert_eq!(r.iter().map(|o| o.checked).collect::<Vec<_>>(), vec![false, false, true]);
    }
}

// ==================== STAGE-B-004b：游戏内菜单分级适配（menu 线）====================
// 分级策略（用户认可）：框体=可重排 UI 锚位公式化；悟空立绘=素材居中不放大。
// classic 代入逐值化回 a.java:2423-2497（paint case 4/19）原公式 = 零偏离命门。

/// 游戏内菜单框布局锚（①框体重排 + ③高亮/排布随框）。
///
/// - ① 框体：内容尺寸**居中**于 (center_x, center_y)（modern 随 layout.center 重排）；
///   框体不拉伸（拉伸语义如需按 STAGE-B-002 v5 判据改本函数一处）。
/// - ③ 选中高亮矩形（hl_x/hl_w）与滚动箭头随框派生——高亮宽 = 条目宽
///   `menu_box_w`（非框宽，Java a.java:2444 原样）。
///
/// 公式逐字对照 a.java:2431-2444/2486-2487：`var128 = menuBoxWidth+22`、
/// `var163 = 120-(var128>>1)`、`var172 = 160-(var146>>1)`、
/// `var180 = menuLineHeight-fontHeight>>1`、高亮 `120-(menuBoxWidth>>1)`、
/// 箭头 `(120+(var128>>1)-23, 160+(var146>>1)-38)`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InGameMenuBox {
    /// 框左上。
    pub bx: i32,
    pub by: i32,
    pub box_w: i32,
    pub box_h: i32,
    /// 首条目文字 y（框顶 +16 边带 + 行距余量 pad）。
    pub first_row_y: i32,
    pub line_h: i32,
    /// 行距余量（line_h - font_h）>>1。
    pub pad: i32,
    /// 条目文本居中锚 x（draw_string anchor 17 = TOP|HCENTER）。
    pub item_cx: i32,
    /// ③ 选中高亮矩形（x/宽 = 条目宽）。
    pub hl_x: i32,
    pub hl_w: i32,
    /// ③ 滚动箭头锚（框右下内侧）。
    pub arrow_x: i32,
    pub arrow_y: i32,
}

/// ①③ 锚位公式化：classic 代入（center 120/160）化回原值；modern 随 center 重排。
pub fn in_game_menu_box(
    center_x: i32,
    center_y: i32,
    menu_box_w: i32,
    list_h: i32,
    line_h: i32,
    font_h: i32,
) -> InGameMenuBox {
    let box_w = menu_box_w + 22;
    let box_h = list_h + 32;
    let bx = center_x - (box_w >> 1);
    let by = center_y - (box_h >> 1);
    let pad = (line_h - font_h) >> 1;
    InGameMenuBox {
        bx,
        by,
        box_w,
        box_h,
        first_row_y: by + 16 + pad,
        line_h,
        pad,
        item_cx: center_x,
        hl_x: center_x - (menu_box_w >> 1),
        hl_w: menu_box_w,
        arrow_x: center_x + (box_w >> 1) - 23,
        arrow_y: center_y + (box_h >> 1) - 38,
    }
}

/// 底部 HUD 信息栏（悟空立绘+属性条）锚（②）——**贴底随宽**。
///
/// 拉伸语义 = STAGE-B-002 v5 底部带背景同款判据（画布宽 > 经典 240 ⇒ 铺满）；
/// 与顶栏状态栏（status_fill 全宽）同 HUD 系两套锚的一致性 = 都随屏宽铺满。
/// classic 代入 = (0, view_bottom, 240, frame_h)——与现行
/// `bottom_panel_dx()=0 + SCREEN_W` 形态逐值同（零偏离）。立绘/属性条内容锚
/// 仍走 Layout（hud_face_ax/hud_icon_x 贴底居中），立绘素材居中不放大。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudPanelAnchor {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// ② 贴底随宽锚位公式。
pub fn hud_panel_anchor(screen_w: i32, view_bottom: i32, frame_h: i32) -> HudPanelAnchor {
    HudPanelAnchor {
        x: 0,
        y: view_bottom,
        w: screen_w,
        h: frame_h,
    }
}

#[cfg(test)]
mod stage_b_004b_menu_anchor_tests {
    use crate::render::FontMetrics;
    use super::{hud_panel_anchor, in_game_menu_box};

    /// classic 代入化回 a.java:2431-2444 原值（零偏离命门）。
    #[test]
    fn in_game_menu_box_classic_substitutes_to_java_literals() {
        // center 120/160 = classic；menuBoxW=80/listH=238/lineH=22/fontH=16 取样值
        let b = in_game_menu_box(120, 160, 80, 238, 22, 16);
        assert_eq!(b.box_w, 80 + 22);
        assert_eq!(b.box_h, 238 + 32);
        assert_eq!(b.bx, 120 - ((80 + 22) >> 1), "a.java:2433 var163");
        assert_eq!(b.by, 160 - ((238 + 32) >> 1), "a.java:2434 var172");
        assert_eq!(b.pad, (22 - 16) >> 1, "a.java:2435 var180");
        assert_eq!(b.first_row_y, b.by + 16 + b.pad, "a.java:2438 首行");
        assert_eq!(b.hl_x, 120 - (80 >> 1), "a.java:2444 高亮 x");
        assert_eq!(b.hl_w, 80, "高亮宽 = 条目宽（非框宽）");
        assert_eq!(b.arrow_x, 120 + ((80 + 22) >> 1) - 23, "a.java:2486");
        assert_eq!(b.arrow_y, 160 + ((238 + 32) >> 1) - 38, "a.java:2487");
        assert_eq!(b.item_cx, 120);
        assert_eq!(b.line_h, 22);
    }

    /// ①③ modern 重排：框心 = layout center，高亮随框居中。
    #[test]
    fn in_game_menu_box_centers_on_layout_modern() {
        let (cx, cy) = (224, 208); // modern(14,13) 448×416 的 center
        let b = in_game_menu_box(cx, cy, 80, 238, 22, 16);
        assert_eq!(b.bx + (b.box_w >> 1), cx, "框水平居中于 layout center");
        assert_eq!(b.by + (b.box_h >> 1), cy, "框垂直居中于 layout center");
        assert_eq!(b.hl_x + (b.hl_w >> 1), cx, "③ 高亮随框居中");
        assert_eq!(b.item_cx, cx, "③ 文本排布随框");
        assert_eq!(b.arrow_x, cx + (b.box_w >> 1) - 23, "③ 箭头随框");
    }

    /// ② classic 零偏离 + modern 贴底随宽（v5 同款拉伸判据）。
    #[test]
    fn hud_panel_anchor_classic_zero_deviation_modern_full_width() {
        let c = hud_panel_anchor(240, 252, 50);
        assert_eq!((c.x, c.y, c.w, c.h), (0, 252, 240, 50), "classic = 现行 bottom_panel_dx()+240 形态");
        let m = hud_panel_anchor(448, 276, 50);
        assert_eq!((m.x, m.y, m.w, m.h), (0, 276, 448, 50), "modern 贴底随宽");
        assert_eq!(m.x + m.w, 448, "随宽铺满（顶栏 status_fill 同款全宽锚）");
    }
}

/// 【2026-10-26 竖屏实机·用户报"存档UI没有居中"】Modern 画布下槽表（mode 7 存档 / mode 8 读档）、
/// 选项面板（mode 16）与菜单族图标条 = **classic 构图整体平移到画布中心**：每个绘制 op 的
/// (x, y) = classic + (dx, dy)，dx = (W−240)/2、dy = (H−320)/2；行带（指针命中单源）同平移。
/// 旧实现几何取 SCREEN_W/H/CENTER_X ⇒ 留在经典 240×320 左上区（Xiaomi 14 竖屏 384×512
/// 实机截图实证）。classic 代入 layout = 240×320 ⇒ 原值零偏离（既有 classic 钉不动）。
#[cfg(test)]
mod modern_center_tests {
    use super::slot_equip_test_support::synthetic_sptarm;
    use super::*;
    use crate::layout::{Layout, SCREEN_H, SCREEN_W};

    fn ui() -> Vec<ArgbImage> {
        (0..25).map(|_| ArgbImage::from_rgb(&vec![0xFF202020; 24 * 24], 24, 24, true)).collect()
    }

    /// 绘制 op 的 (名, x, y) 序列；reset_clip 的 `setClip(0,0,画布)` 为画布级、不计。
    fn ops_xy(layout: &Layout, f: &dyn Fn(&mut SoftGraphics<'_>)) -> Vec<(String, i32, i32)> {
        let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
        let ops = {
            let mut g = SoftGraphics::new(&mut img);
            g.set_font(Some(crate::paint::paint_font()));
            f(&mut g);
            g.ops.clone()
        };
        ops.iter()
            .filter(|o| !o.starts_with("setClip(0,0,"))
            .filter_map(|o| {
                let name = o.split('(').next()?.to_string();
                let args = o.split_once('(')?.1.rsplit_once(')')?.0;
                let mut nums = args.split(',').filter_map(|t| t.trim().parse::<i32>().ok());
                Some((name, nums.next()?, nums.next()?))
            })
            .collect()
    }

    fn shifted(v: &[(String, i32, i32)], dx: i32, dy: i32) -> Vec<(String, i32, i32)> {
        v.iter().map(|(n, x, y)| (n.clone(), x + dx, y + dy)).collect()
    }

    fn modern_layouts() -> [Layout; 3] {
        [Layout::modern(12, 16), Layout::modern(13, 13), Layout::modern(18, 18)]
    }

    #[test]
    fn slot_table_is_classic_composition_centered() {
        let (ui, icons) = (ui(), synthetic_sptarm());
        let mut slot = SlotSelect::new();
        slot.valid[0] = true; // 详情面板（层数/装备框/金币/三围/三钥）全段入判据
        slot.floor_num[0] = 7;
        slot.weapon[0] = 35;
        slot.armor[0] = 41;
        slot.gold[0] = 120;
        slot.valid[2] = true;
        let classic = Layout::classic();
        let base = ops_xy(&classic, &|g| slot.paint(g, &ui, &[3, 5], &icons, || 0, &classic));
        assert!(base.len() > 20, "判据非空（{} op）", base.len());
        for l in modern_layouts() {
            let (dx, dy) = ((l.screen_w - SCREEN_W) / 2, (l.screen_h - SCREEN_H) / 2);
            let got = ops_xy(&l, &|g| slot.paint(g, &ui, &[3, 5], &icons, || 0, &l));
            assert_eq!(got, shifted(&base, dx, dy), "{}×{}：槽表 = classic 构图平移 ({dx},{dy})", l.screen_w, l.screen_h);
            for vis in 0..=slot.rows_per_page {
                let (cy, ch) = slot.row_band(vis, &classic);
                assert_eq!(slot.row_band(vis, &l), (cy + dy, ch), "{}×{}：行带（命中单源）同平移", l.screen_w, l.screen_h);
            }
        }
    }

    #[test]
    fn option_panel_is_classic_composition_centered() {
        let ui = ui();
        let list = OptionList::new(true, false);
        let font = crate::paint::paint_font();
        let classic = Layout::classic();
        let base = ops_xy(&classic, &|g| list.paint(g, &ui, &font, &classic));
        assert!(base.len() > 8, "判据非空（{} op）", base.len());
        for l in modern_layouts() {
            let (dx, dy) = ((l.screen_w - SCREEN_W) / 2, (l.screen_h - SCREEN_H) / 2);
            let got = ops_xy(&l, &|g| list.paint(g, &ui, &font, &l));
            assert_eq!(got, shifted(&base, dx, dy), "{}×{}：选项面板 = classic 构图平移", l.screen_w, l.screen_h);
            for i in 0..list.len() {
                let (cy, ch) = list.row_band(i, &classic);
                assert_eq!(list.row_band(i, &l), (cy + dy, ch));
            }
        }
    }

    #[test]
    fn menu_icon_strip_x_centers_on_canvas() {
        // a.java:2960/2969/3000 drawIconStrip(…, 86, 10, 0, sy, 68, 17, 17)：86 = (240−68)/2
        assert_eq!(crate::title::menu_strip_x(SCREEN_W), 86, "classic = Java 字面");
        for l in modern_layouts() {
            assert_eq!(crate::title::menu_strip_x(l.screen_w), 86 + (l.screen_w - SCREEN_W) / 2);
        }
    }
}
