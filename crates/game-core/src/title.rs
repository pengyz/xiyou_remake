//! mode 1 标题主菜单（paint case 1，a.java:2349-2376 + run case 1，a.java:3380-3429）。
//!
//! # 结构（A 级证据）
//!
//! - **paint**（a.java:2349-2376）：map 背景图 → 底部状态条 → 图标带动画
//!   （drawIconStrip，a.java:4790-4797）→ 双侧羽化箭头（bob 摆动）→ 粒子层
//!   （updateAndDrawParticles，clip 0..308）。
//! - **tick**（a.java:3380-3429）：`(frameCounter&3)==0` 时 spawn 粒子（每次
//!   吃两个 randomBelow：先 x=rb(240) 后 y=295-rb(150)）；advanceBobPhase bob 摆动；
//!   方向键移动 cursor（环绕）；-5 确认按 menuItemKinds[cursor] 切 gameMode。
//! - **粒子演化**（updateAndDrawParticles a.java:9840-9852）：槽位 **31→0 倒序**；每 paint 先
//!   画后推进：x+=velX；velX 不满（waxing=false）时 ++，触顶 2*size 翻转；
//!   满（waxing=true）时 --，触底 -2*size 翻转且 size++（>5 死亡）；y+=velY
//!   匀速。spawn 初值 velX=-3/velY=-4/size=2/waxing=false。
//! - **随机链**：java.util.Random LCG，种子 = VTime 构造时刻（=0）。实测锚：
//!   种子 0 首对 rb=(0,148)/(109,47)，T73/T77 spawn (0,147)/(109,248)，
//!   T80 帧 (0,119)/(103,236)（A-menu-sweep TICK 0080 逐字节吻合）。
//!
//! # 时序合同
//!
//! mode 切换拍（如 logic#71 消费 -6）只跑 mode 21 的分支体（break 跳出），
//! mode 1 的首个完整 tick 是**下一拍**（logic#72：bob 0→-1、T72 帧箭头 61）。

use crate::enums::GameMode;
use crate::render::{ArgbImage, SoftGraphics};
use game_platform::JavaRandom;

/// drawIconStrip 的列波动表 iconStripDx（a.java:462）。
pub const STRIP_DX: [i32; 8] = [-1, 0, 1, 1, 1, 0, -1, -1];
/// drawIconStrip 的行波动表 iconStripDy（a.java:463）。
pub const STRIP_DY: [i32; 8] = [-1, -1, -1, 0, 1, 1, 1, 0];

/// 粒子槽位容量（spawnParticle/updateAndDrawParticles 的 32 槽环形游标）。
const PARTICLE_SLOTS: usize = 32;

/// 粒子系统状态（spawnParticle 登记表 + updateAndDrawParticles 演化）。
pub struct Particles {
    x: [i32; PARTICLE_SLOTS],
    y: [i32; PARTICLE_SLOTS],
    vel_x: [i32; PARTICLE_SLOTS],
    vel_y: [i32; PARTICLE_SLOTS],
    size: [i32; PARTICLE_SLOTS],
    active: [bool; PARTICLE_SLOTS],
    waxing: [bool; PARTICLE_SLOTS],
    cursor: usize,
    /// particleColor：粒子颜色（dispatchModeEntry case 1 设 -1 = 白）。
    pub color: u32,
}

impl Particles {
    /// 诊断探针：首个活跃粒子位置。
    pub fn dump_first(&self) -> Option<(i32, i32)> {
        (0..PARTICLE_SLOTS)
            .find(|&k| self.active[k])
            .map(|k| (self.x[k], self.y[k]))
    }

    /// 诊断探针（STAGE-B-004a ①a）：活跃粒子最大 x（生成域判据用）。
    pub fn max_active_x(&self) -> i32 {
        (0..PARTICLE_SLOTS)
            .filter(|&k| self.active[k])
            .map(|k| self.x[k])
            .max()
            .unwrap_or(-1)
    }
}

impl Default for Particles {
    fn default() -> Self {
        Self::new()
    }
}

impl Particles {
    pub fn new() -> Particles {
        Particles {
            x: [0; PARTICLE_SLOTS],
            y: [0; PARTICLE_SLOTS],
            vel_x: [0; PARTICLE_SLOTS],
            vel_y: [0; PARTICLE_SLOTS],
            size: [0; PARTICLE_SLOTS],
            active: [false; PARTICLE_SLOTS],
            waxing: [false; PARTICLE_SLOTS],
            cursor: 0,
            color: crate::layout::WHITE,
        }
    }

    /// `spawnParticle`（a.java:9826-9832）：环形游标登记新粒子。
    pub fn spawn(&mut self, x: i32, y: i32) {
        let i = self.cursor;
        self.x[i] = x;
        self.y[i] = y;
        self.vel_x[i] = -3;
        self.vel_y[i] = -4;
        self.size[i] = 2;
        self.active[i] = true;
        self.waxing[i] = false;
        self.cursor = if self.cursor + 1 > 31 { 0 } else { self.cursor + 1 };
    }

    /// `updateAndDrawParticles`（a.java:9840-9852）：**paint 内调用**——先画后推进，槽位 31→0 倒序。
    pub fn paint_and_update(&mut self, g: &mut SoftGraphics<'_>) {
        g.set_color(self.color);
        for i in (0..PARTICLE_SLOTS).rev() {
            if !self.active[i] {
                continue;
            }
            let s = self.size[i];
            g.fill_rect(self.x[i], self.y[i], s, s);
            // 推进（a.java:9841-9851）
            self.x[i] += self.vel_x[i];
            let limit = s << 1;
            if !self.waxing[i] {
                self.vel_x[i] += 1;
                if self.vel_x[i] == limit {
                    self.waxing[i] = true;
                }
            } else {
                self.vel_x[i] -= 1;
                if self.vel_x[i] == -limit {
                    self.waxing[i] = false;
                    self.size[i] += 1;
                    if self.size[i] > 5 {
                        self.active[i] = false;
                    }
                }
            }
            self.y[i] += self.vel_y[i];
        }
    }
}

/// `drawIconStrip` 通用形（a.java:4793-4805）。`frame` 为 iconStripFrame
///（每 paint +1 环 8）；count 为源宽度阈值（60=title 4 列、68=菜单族 4 列）。
pub fn paint_icon_strip_at(
    g: &mut SoftGraphics<'_>,
    strip: &ArgbImage,
    x0: i32,
    y0: i32,
    sy: i32,
    count: i32,
    frame: &mut usize,
) {
    let mut f = *frame;
    let mut src_x = 0;
    let mut base_x = x0;
    while src_x < count {
        let cx = base_x + STRIP_DX[f];
        let cy = y0 + STRIP_DY[f];
        g.set_clip(cx, cy, 17, 17);
        g.draw_image(strip, cx - src_x, cy - sy, 0);
        f += 1;
        if f > 7 {
            f = 0;
        }
        src_x += 17;
        base_x += 17;
    }
    *frame += 1;
    if *frame > 7 {
        *frame = 0;
    }
    g.reset_clip();
}

/// mode 1 标题菜单状态机（run case 1 的 tick + paint case 1 的绘制输入）。
pub struct TitleMachine {
    /// menuCursorIndex：菜单光标（0..menu_item_count-1 环绕）。
    pub cursor: i32,
    /// menuItemKinds 的条目（dispatchModeEntry case 1 经 addMenuItem(0..5) 登记为 0..5）。
    pub menu_kinds: Vec<i8>,
    /// animBobOffset：bob 摆动偏移（advanceBobPhase 演化）。
    pub bob: i32,
    /// animBobRising：bob 方向（false=下降相 --，true=上升相 ++）。
    pub bob_rising: bool,
    /// iconStripFrame（drawIconStrip 每 paint +1，环绕 8）。
    strip_frame: usize,
    /// 粒子层。
    pub particles: Particles,
    /// statusBarHeight：底部条高（构造 25）。
    pub bar_height: i32,
}

impl TitleMachine {
    pub fn new() -> TitleMachine {
        TitleMachine {
            cursor: 0,
            // dispatchModeEntry case 1：resetMenuLayout 重置后 addMenuItem(0..5)（a.java:4291-4291）
            menu_kinds: vec![0, 1, 2, 3, 4, 5],
            bob: 0,
            bob_rising: false,
            strip_frame: 0,
            particles: Particles::new(),
            bar_height: 25,
        }
    }

    /// `advanceBobPhase`（a.java:6476-6479）：bob ±1 摆动（下降相 --<-1 翻上升；上升 ++>1 翻下降）。
    /// iconStripFrame 外部同步（与 BootMachine 共享同一 Java 字段 a.java:464）。
    pub fn set_strip_frame(&mut self, f: usize) {
        self.strip_frame = f;
    }

    pub fn strip_frame_value(&self) -> usize {
        self.strip_frame
    }

    fn tick_bob(&mut self) {
        if self.bob_rising {
            self.bob += 1;
            if self.bob > 1 {
                self.bob_rising = false;
            }
        } else {
            self.bob -= 1;
            if self.bob < -1 {
                self.bob_rising = true;
            }
        }
    }

    /// run case 1 的逻辑 tick（a.java:3380-3429）。
    ///
    /// `frame_counter` = 本拍的 frameCounter 值（迭代 N 拍 = N-1）。
    /// 返回 `Some(new_mode)` 当确认键切模式（调用方负责后续状态机移交）。
    pub fn tick(&mut self, key: i32, frame_counter: i32, rng: &mut JavaRandom, layout: &crate::layout::Layout) -> Option<GameMode> {
        if frame_counter & 3 == 0 {
            // a.java:3382：参数求值序 L2R——先 x=rb(240) 后 y=295-rb(150)
            // 【STAGE-B-004a ①a】粒子生成域随画布（与 intro 雨同口径；classic 同值零偏离）
            let x = rng.random_below(layout.screen_w);
            let y = layout.screen_h - self.bar_height - rng.random_below(150);
            self.particles.spawn(x, y);
        }
        self.tick_bob();
        match key {
            // 确认（a.java:3388-3415）：按 menuItemKinds[cursor] 切模式
            -5 | 53 => {
                let next = match self.menu_kinds[self.cursor as usize] {
                    0 => GameMode::IntroCutscene,
                    1 => GameMode::EquipDetail,
                    2 => GameMode::About,
                    3 => GameMode::Help,
                    4 => GameMode::IconView,
                    5 => GameMode::LogoAnim,
                    // default：kind 直接即 mode（a.java:3388-3415 quirk 保真）
                    other => GameMode::from_raw(other as i32),
                };
                Some(next)
            }
            // 下移（a.java:3414-3417）
            -4 | -2 | 54 | 56 => {
                if self.cursor < self.menu_kinds.len() as i32 - 1 {
                    self.cursor += 1;
                } else {
                    self.cursor = 0;
                }
                None
            }
            // 上移（a.java:3424-3426）
            -3 | -1 | 50 | 52 => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                } else {
                    self.cursor = self.menu_kinds.len() as i32 - 1;
                }
                None
            }
            _ => None,
        }
    }

    /// paint case 1（a.java:2349-2376）。`map_bg`=map 容器 [0]、`strip`=map[1]、
    /// `arrow`=ui[8][14]。
    pub fn paint(
        &mut self,
        g: &mut SoftGraphics<'_>,
        map_bg: &ArgbImage,
        strip: &ArgbImage,
        arrow: &ArgbImage,
        layout: &crate::layout::Layout,
    ) {
        let bar = self.bar_height;
        let (sw, sh) = (layout.screen_w, layout.screen_h);
        // 【STAGE-B-004a ②】分级适配（用户裁定：手绘素材=原尺寸居中+边缘延展
        // **不放大**；可重排 UI=锚位公式化）。classic 画布=原式原样（零偏离）。
        if sw > crate::layout::SCREEN_W || sh > crate::layout::SCREEN_H {
            // —— modern：标题图 240×320 原尺寸（**横=居中，纵=底对齐贴底栏**；禁放大）
            // + 边缘平色延展带补满（2026-10-19 措辞勘误：纵非居中=底对齐；带=平色）
            // （v6 同法：四带=边角色）——右半黑/左贴边 artifact 根因。
            let img_x = (sw - map_bg.width) / 2;
            let img_y = (sh - bar - map_bg.height).max(0);
            let bw = map_bg.width as usize;
            let bh = map_bg.height as usize;
            // 【用户验收修正 2026-10-19】左右延展带=**顶带同款纯蓝**（天空连续感，
            // 无渐变无波浪纹）——旧取图中缘像素（云纹/渐变边缘=波浪纹 artifact）；
            // 底带=图底缘色（黑带）。延展带配色与素材边缘协调（标题图边缘=天空蓝/黑带）。
            let top_px = map_bg.argb[0] & 0x00FF_FFFF | 0xFF00_0000;
            let bottom_px = map_bg.argb[(bh - 1) * bw] & 0x00FF_FFFF | 0xFF00_0000;
            g.set_color(top_px);
            g.fill_rect(0, 0, sw, img_y);
            g.set_color(bottom_px);
            g.fill_rect(0, img_y + map_bg.height, sw, sh - (img_y + map_bg.height));
            g.set_color(top_px); // 左右带=纯蓝（顶带同款）
            g.fill_rect(0, img_y, img_x, map_bg.height);
            g.fill_rect(img_x + map_bg.width, img_y, sw - (img_x + map_bg.width), map_bg.height);
            if layout.preset == crate::layout::Preset::Modern {
                // 【问题1】Modern 标题美术铺满游戏方框（用户裁定：贴图满幅、蓝底
                // 留边也行但贴图必须满幅）——cover 撑满裁切；下方蓝填充行被盖住
                g.draw_image_cover(map_bg, sw, sh, true); // 底锚：保底部 logo（截断回派）
            } else {
                g.draw_image(map_bg, img_x, img_y, 0);
            }
            // 底部条/图标带/双箭头=锚位公式化（屏心 ± 原偏置；ARROW 对原偏置
            // 不对称=原版字面，layout.rs:370-371 公式表）
            g.set_color(crate::layout::DARK_BACKDROP);
            g.fill_rect(0, sh - bar, sw, bar);
            let strip_x = layout.center_x - 34; // 经典 86 = 120-34（屏心偏置）
            let strip_y = sh - bar + ((bar - 17) >> 1);
            self.paint_icon_strip(g, strip, strip_x, strip_y, 17 * self.menu_kinds[self.cursor as usize] as i32);
            let arrow_y = sh - bar + ((bar - 13) >> 1);
            let t = crate::paint::NOKIA_TRANSFORM_TABLE[1];
            g.draw_image_transformed(arrow, layout.center_x - 60 - self.bob, arrow_y, 0, t);
            g.draw_image(arrow, layout.center_x + 40 + self.bob, arrow_y, 0);
            // 粒子层（modern clip=满画布；生成域判据见 ①a 测）
            g.set_clip(0, bar, sw, sh - (bar >> 1));
            self.particles.paint_and_update(g);
            g.reset_clip();
            return;
        }
        // —— classic 原式（a.java:2349-2376 逐字，零偏离）——
        // ① 顶部补条（a.java:2352-2355：间隙 >0 才画）
        let top_gap = sh - bar - map_bg.height;
        if top_gap > 0 {
            g.set_color(crate::layout::TITLE_TOP_GAP);
            g.fill_rect(0, 0, sw, top_gap);
        }
        // ② 背景图（a.java:2357）
        g.draw_image(map_bg, 0, top_gap, 0);
        // ③ 底部条（a.java:2358-2359）
        g.set_color(crate::layout::DARK_BACKDROP);
        g.fill_rect(0, sh - bar, sw, bar);
        // ④ 图标带（a.java:2360-2369 + drawIconStrip）
        let strip_y = sh - bar + ((bar - 17) >> 1);
        self.paint_icon_strip(g, strip, 86, strip_y, 17 * self.menu_kinds[self.cursor as usize] as i32);
        // ⑤ 双侧箭头（a.java:2370-2372）：左=镜像变换（表值 8192），右=原样
        let arrow_y = sh - bar + ((bar - 13) >> 1);
        let t = crate::paint::NOKIA_TRANSFORM_TABLE[1];
        g.draw_image_transformed(arrow, crate::layout::ARROW_PAIR_LEFT_X - self.bob, arrow_y, 0, t);
        g.draw_image(arrow, crate::layout::ARROW_PAIR_RIGHT_X + self.bob, arrow_y, 0);
        // ⑥ 粒子层（clip 0..308，a.java:2373-2375）
        g.set_clip(0, bar, sw, sh - (bar >> 1));
        self.particles.paint_and_update(g);
        g.reset_clip();
    }

    /// 【2c 裁定】6 项菜单行矩形单源（paint 与 hit-test 共调——item_cell_origin
    /// 先例）：行高 28+行隙 2、x∈[80, sw-80)、底锚于底条上方（y0=sh-bar-6*30）。
    pub fn menu_item_rects(sw: i32, sh: i32, bar: i32) -> [(i32, i32, i32, i32); 6] {
        let row_h = 28;
        let gap = 2;
        let y0 = sh - bar - 6 * (row_h + gap);
        let mut out = [(0, 0, 0, 0); 6];
        for (i, r) in out.iter_mut().enumerate() {
            *r = (80, y0 + i as i32 * (row_h + gap), sw - 160, row_h);
        }
        out
    }

    /// 【2c 裁定】tap 命中：行内=Some(index)（切换+确认一步到位），行隙/区外=None
    ///（不误触）。标题 UI=键盘语义（cursor+-5 确认），本 fn 只做命中定位。
    pub fn menu_hit(x: i32, y: i32, sw: i32, sh: i32, bar: i32) -> Option<usize> {
        for (i, (rx, ry, rw, rh)) in Self::menu_item_rects(sw, sh, bar).into_iter().enumerate() {
            if x >= rx && x < rx + rw && y >= ry && y < ry + rh {
                return Some(i);
            }
        }
        None
    }

    /// `drawIconStrip`（a.java:4790-4797）：4 列图标带（列源 x = 0/17/34/51），
    /// 每列按 STRIP_DX/DY[帧] 波动；字段帧每 paint +1（环 8）。
    fn paint_icon_strip(&mut self, g: &mut SoftGraphics<'_>, strip: &ArgbImage, x0: i32, y0: i32, sy: i32) {
        let mut frame = self.strip_frame;
        let mut src_x = 0;
        let mut base_x = x0;
        while src_x < 60 {
            let cx = base_x + STRIP_DX[frame];
            let cy = y0 + STRIP_DY[frame];
            g.set_clip(cx, cy, 17, 17);
            g.draw_image(strip, cx - src_x, cy - sy, 0);
            frame += 1;
            if frame > 7 {
                frame = 0;
            }
            src_x += 17;
            base_x += 17;
        }
        self.strip_frame += 1;
        if self.strip_frame > 7 {
            self.strip_frame = 0;
        }
        g.reset_clip();
    }
}
