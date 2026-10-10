//! mode 14 新游戏引子（runIntroViewer a.java:4816-4887 + paint case 14 a.java:2917-2950）
//! 与 mode 2 加载链（run case 2 a.java:3441-3542 + startNewGameLoad/m_131/m_132）。
//!
//! # mode 14 引子（三页图文 + 字幕滚动机）
//!
//! - 页 0/1：intro 容器图 + `dimImage(img, 34)`（alpha=34 压暗）；文本
//!   miscTexts[4]/[6]；页 2：end 容器图 + 文本[7]；页 3：无图占位
//!   （立即转加载）。
//! - **字幕滚动机**（复用 overlay 排版状态 wrappedLines/overlayPageTop）：
//!   滚动相 `introScrollX += 4`（按键快进 =240），达 `stringWidth(行[页首])+10`
//!   翻调色相；调色相 `++introColorIdx >= 5` 翻回且 2 行窗口下滚
//!   （flipOverlayPage(+2)）；滚到尾（overlayTotalLines-2）→ 页++。
//! - paint：黑遮幅上下 + HCENTER 图 + 字幕（滚动相白 / 调色相 5 色循环
//!   introColors）+「跳过/请按任意键」+ 粒子雨（case 14 tick 4 分频 spawn）。
//! - -7 或页 3 → `startNewGameLoad()` 启动加载链。
//!
//! # mode 2 加载链（进度追赶模型）
//!
//! `m_131(target, call_m000)`：steps 表 loadSteps[16]、loadStepCount=步数、
//! m_001(15)（load 容器进度条图）。tick：`loadProgress < loadProgressTarget` 时进度
//! +4/tick；否则执行 steps[loadStepIndex] 并 `loadProgressTarget = 步idx*100/步数`（≤
//! 进度则 = 进度+1）。进度 ≥100 → 重置 + gameMode=loadTargetMode（loadCallInit 则
//! m_000）。**42 tick/次**（enter-game T382-423 实证）。
//!
//! Rust 侧资源已全预载——步骤只落实**状态效果**（步骤 8 的新游戏初始值），
//! 其余步骤的地图/实体装载属 mode 3 范围（本模块只推进度与目标模式）。

use crate::render::{ArgbImage, SoftGraphics};

/// `dimImage`（a.java:4604-4626）：压暗变换——白色像素(-1)跳过，其余非透明
/// 像素 alpha 置 `alpha`。
pub fn dim_image(img: &ArgbImage, alpha: i32) -> ArgbImage {
    let a = (alpha as u32) << 24;
    let mut out = img.clone();
    for px in out.argb.iter_mut() {
        if *px != 0xFFFF_FFFF && (*px & 0xFF00_0000) != 0 {
            *px = (*px & 0x00FF_FFFF) | a;
        }
    }
    out
}

/// 引子文本表 miscTexts[4]/[6]/[7]（a.java:438-447，UTF-16）。
pub fn intro_text(page: i32) -> Vec<u16> {
    const T4: &str = "俺，当世神界第一斗者，孙!悟!空! 自从受封为齐天大圣，掌管蟠桃园以来，一直逍遥快活，无拘束……";
    const T6: &str = "直到那一天，遇到了她，在筋斗云上的我，竟然第一次心潮起伏，有了晕机的感觉……";
    const T7: &str = "神仙动了感情，往往会万劫不复，\n这一次，让我付出了五百年的时间去忘记她……\n五指山脚下的沙子，掠过我的脸庞。\n沙子，跟时间一样，同样随风流逝；同样掩埋过去；\n多少次伸手想抓住，却从指隙溜走……\n看夜空，半梦半醒间，往事历历上心头……";
    match page {
        0 => T4.encode_utf16().collect(),
        1 => T6.encode_utf16().collect(),
        _ => T7.encode_utf16().collect(),
    }
}

/// 「跳过」/「请按任意键」（a.java:2938/2940）。
pub const SKIP_LABEL: &[u16] = &[0x8DF3, 0x8FC7]; // 跳过
pub const PRESS_ANY_KEY: &[u16] = &[0x8BF7, 0x6309, 0x4EFB, 0x610F, 0x952E]; // 请按任意键

/// mode 14 引子状态机。
pub struct IntroSequence {
    /// introPage：页号（0/1/2 图文，3=空页转加载）。
    pub page: i32,
    /// introPageTick：页内 tick 计数（0=本页首拍装载）。
    page_tick: i32,
    /// introScrollX：字幕滚动偏移。
    scroll_x: i32,
    /// introColorPhase：调色相标志（false=滚动相）。
    color_phase: bool,
    /// introColorIdx：调色索引。
    color_idx: i32,
    /// introImage：当前引子图（压暗后）。
    image: Option<ArgbImage>,
    /// introText：当前字幕文本。
    text: Option<Vec<u16>>,
}

impl Default for IntroSequence {
    fn default() -> Self {
        Self::new()
    }
}

impl IntroSequence {
    pub fn new() -> IntroSequence {
        IntroSequence { page: 0, page_tick: 0, scroll_x: 0, color_phase: false, color_idx: 0, image: None, text: None }
    }

    /// 页首拍装载（runIntroViewer 的 introPageTick==0 分支，a.java:4826-4850）。
    /// `intro`/`end_container` 为容器图列表。
    fn load_page(&mut self, intro: &[ArgbImage], end: &[ArgbImage]) {
        self.image = None;
        match self.page {
            0 => {
                self.image = Some(dim_image(&intro[0], 34));
                self.text = Some(intro_text(0));
            }
            1 => {
                self.image = Some(dim_image(&intro[1], 34));
                self.text = Some(intro_text(1));
            }
            2 => {
                self.image = Some(dim_image(&end[0], 34));
                self.text = Some(intro_text(2));
            }
            _ => {}
        }
    }

    /// runIntroViewer 一次 tick（a.java:4816-4887）。返回 `true` 当引子结束
    /// （-7 或页 3 滚完 → 调用方启动 startNewGameLoad 加载链）。
    pub fn tick(
        &mut self,
        key: i32,
        intro: &[ArgbImage],
        end: &[ArgbImage],
        overlay: &mut crate::menu_family::OverlayEngine,
        font: &crate::render::SoftFont,
    ) -> bool {
        if key == -7 {
            self.reset();
            return true;
        }
        if self.page_tick == 0 {
            self.load_page(intro, end);
        }
        self.page_tick += 1;
        if self.page < 3 {
            if let Some(text) = self.text.clone() {
                // 滚动机（a.java:4852-4885）：门槛 overlayLayoutCache != null——
                // **上次 paint 的布局锚**。首拍锚为 null ⇒ 整段跳过（enter-game
                // T102 实证：scroll 自 T103 起 +4）；tick 不主动布局，读 overlay
                // 现有行（paint 首拍已按 (220, 28) 排好）
                if !overlay.layout_done() {
                    return false;
                }
                let _ = &text;
                if !self.color_phase {
                    let top_line = overlay.line_at_top();
                    let line_w = font.string_width(&overlay.line(top_line));
                    if self.scroll_x >= line_w + 10 {
                        self.color_phase = true;
                        self.color_idx = 0;
                        return false;
                    }
                    self.scroll_x += 4;
                    if key != 0 {
                        self.scroll_x = 240;
                    }
                } else {
                    self.color_idx += 1;
                    if self.color_idx >= crate::layout::SUBTITLE_COLORS.len() as i32 {
                        self.scroll_x = 0;
                        self.color_idx = 0;
                        self.color_phase = false;
                        if overlay.page_top() < overlay.total_lines() - 2 {
                            let top = overlay.page_top();
                            overlay.flip_page(top + 2);
                            return false;
                        }
                        // 本页滚完 → 页++（Java 递归 runIntroViewer：装载下一页并**同拍
                        // 继续滚动机**——a.java:4881-4884，enter-game T285 实证）
                        self.page += 1;
                        self.page_tick = 0;
                        if self.page < 3 {
                            self.load_page(intro, end);
                            self.page_tick = 1;
                            // 递归拍继续滚动机（scroll 0 → 4）
                            if overlay.layout_done() {
                                self.scroll_x += 4;
                            }
                            return false;
                        }
                        return true; // 页 3 → 清理 + startNewGameLoad
                    }
                }
            } else {
                // 无文本页（不该发生——页 0..2 恒有文本）
                return true;
            }
        } else {
            // 页 >= 3（a.java:4871-4879）：清理 + startNewGameLoad
            self.reset();
            return true;
        }
        false
    }

    fn reset(&mut self) {
        self.image = None;
        self.text = None;
        self.page = 0;
        self.page_tick = 0;
        self.scroll_x = 0;
        self.color_phase = false;
        self.color_idx = 0;
    }

    /// paint case 14（a.java:2917-2950）。`particles` 由调用方持有（case 14
    /// 尾部的粒子层）；返回值无。
    pub fn paint(
        &self,
        g: &mut SoftGraphics<'_>,
        overlay: &mut crate::menu_family::OverlayEngine,
        particles: &mut crate::title::Particles,
        font: &crate::render::SoftFont,
    ) {
        if let Some(img) = &self.image {
            let ih = img.height;
            let band_y = 320 - ih - (font.height + 4) * 3 >> 1;
            g.set_color(crate::layout::BLACK);
            g.fill_rect(0, 0, 240, band_y);
            let below = band_y + ih;
            g.fill_rect(0, below, 240, 320 - below);
            g.draw_image(img, 120, band_y, 17);
            if !self.color_phase {
                g.set_color(crate::layout::WHITE);
            } else {
                g.set_color(crate::layout::SUBTITLE_COLORS[self.color_idx as usize]);
            }
            if self.page < 3 {
                if let Some(text) = &self.text {
                    overlay.paint(
                        g,
                        text,
                        10,
                        band_y + ih + font.height,
                        220,
                        (font.height + 4) << 1,
                        false,
                        0,
                        font,
                    );
                }
            }
        }
        if self.page > 2 {
            g.set_color(crate::layout::WHITE);
            g.draw_string(PRESS_ANY_KEY, 120, 320 - font.height - 2, 17);
        } else {
            g.set_color(crate::layout::WHITE);
            g.draw_string(SKIP_LABEL, 240 - font.string_width(SKIP_LABEL) - 5, 320 - font.height - 2, 0);
        }
        g.set_clip(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
        particles.paint_and_update(g);
        g.set_clip(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
    }
}

/// mode 2 加载链状态（m_131/m_132 + run case 2 的进度追赶）。
pub struct LoadProgress {
    /// steps：loadSteps 有效前缀（m_132 登记序）。
    steps: Vec<i8>,
    /// loadStepIndex：步指针。
    step_idx: i32,
    /// loadProgress：进度（0..100）。
    progress: i32,
    /// loadProgressTarget：进度追赶目标。
    bar_max: i32,
    /// loadTargetMode：目标模式。
    pub target_mode: i32,
    /// loadCallInit：完成时是否调 m_000。
    call_m000: bool,
}

impl LoadProgress {
    /// `m_131(target, call_m000)` + `m_132(...)`（a.java:9875-9889 区）。
    pub fn new(target: i32, call_m000: bool, steps: &[i8]) -> LoadProgress {
        LoadProgress { steps: steps.to_vec(), step_idx: 0, progress: 0, bar_max: 0, target_mode: target, call_m000 }
    }

    /// `startNewGameLoad`（a.java:7162-7175）：新游戏加载链。
    pub fn new_game() -> LoadProgress {
        LoadProgress::new(3, true, &[6, 8, 5, 13, 9, 10, 2, 3, 12, 11])
    }

    /// run case 2 一次 tick（a.java:3443-3541）。返回 `Some(target_mode)`
    /// 当进度 ≥100（加载完成；`call_m000` 供调用方决定初始化）。
    pub fn tick(&mut self) -> Option<(i32, bool)> {
        if self.progress < 100 {
            if self.progress < self.bar_max {
                self.progress += 4;
            } else {
                // 步执行（资源已预载；步骤 8 的新游戏初始状态由调用方落实）
                self.step_idx += 1;
                let total = self.steps.len() as i32;
                self.bar_max = self.step_idx * 100 / total;
                if self.bar_max <= self.progress {
                    self.bar_max = self.progress + 1;
                }
            }
            None
        } else {
            Some((self.target_mode, self.call_m000))
        }
    }

    /// 步骤 8 语义（a.java:3476-3489）：新游戏初始状态（HP/楼层/钥匙）。
    /// 返回 (hp, atk, def, floor)。
    pub fn new_game_stats() -> (i32, i32, i32, i32) {
        (300, 10, 10, 51)
    }

    /// paint case 2（a.java:2377-2384）：全屏底色 + 双段进度条（load 容器图）。
    pub fn paint(&self, g: &mut SoftGraphics<'_>, load: &[ArgbImage]) {
        g.set_color(crate::layout::DARK_BACKDROP);
        g.fill_rect(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
        let shown = 148 * self.progress / 100;
        crate::paint::draw_image_clipped(g, &load[0], 108, 86, 0, 0, 24, shown);
        crate::paint::draw_image_clipped(g, &load[1], 108, 86 + shown, 0, shown, 24, 148 - shown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intro_layout_two_lines_per_page() {
        let mut ov = crate::menu_family::OverlayEngine::new();
        let font = crate::paint::paint_font();
        let text = intro_text(0);
        ov.layout_for_intro(&text, &font);
        assert_eq!(ov.lines_per_page(), 2, "28/14=2 行/页");
        assert_eq!(ov.total_lines(), 3, "引子页 0 文本切 3 行（216+216+尾）");
        let l0 = ov.line(0);
        let l0s: String = l0.iter().map(|&c| char::from_u32(c as u32).unwrap()).collect();
        eprintln!("line0 = {l0s:?} width={}", font.string_width(&l0));
    }
}
