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

// ---------------- 【STAGE-B 触屏标题 C 方案（2026-10-26 用户裁定）】 ----------------
//
// 决策记录：docs/knowledge/decision_touch-title-carousel-stage-b.md。
// - 保留原版**轮播**视觉（底栏「◆ 当前项 ◆」，左右键切项、确认进入）；
// - 触屏 = 点左/右热区切上/下一项、点中间标签确认、水平滑动切项（键码走既有
//   title tick 臂 -3/-4/-5，零新游戏状态）；
// - Modern 底栏/标签/箭头 ×k（`Layout::title_ui_k`：modern 默认 ×2 = 横屏大画布补偿；壳可配，
//   Android 竖屏开场链 = 原分辨率 × 整数倍 ⇒ ×1）+ 美术**整数倍**放大（像素均匀）；
// - 几何单源：paint 与命中共用 [`TitleGeom::modern`]（pointer-menu 决策硬条件①）。
// classic 路径零改动（原版 240×320 逐字）。

/// Modern 底栏 UI 倍率默认值（`Layout::modern` 的 `title_ui_k`；标签/箭头放大）。
pub const MODERN_UI_K: i32 = 2;
/// Modern 底栏标签上下留白。
pub const MODERN_BAR_PAD: i32 = 3;
/// 触屏热区 = 底栏 + 向上延伸 `TOUCH_ZONE_EXT × ui_k`（自画布底向上；×2 档 = 40 + 32 = 72 逻辑 px，
/// 横屏 2× 呈现@480dpi ≈ 7.6mm 手指可点；×1 档 = 23 + 16 = 39，竖屏 ×5 呈现 ≈ 10mm）。延伸随 ui_k：
/// 壳按物理倍率选 ui_k，延伸同比 ⇒ 热区物理尺寸两档相当、不大面积覆盖美术（隐形热区 = 2c 误触教训）。
pub const TOUCH_ZONE_EXT: i32 = 16;
/// 美术顶部最大裁切 = 美术高的 1/10（超出则降一档整数倍）。
const ART_MAX_CROP_DIV: i32 = 10;
/// 图标带格边长（drawIconStrip 17×17 格，4 格=一个菜单项标签）。
const STRIP_CELL: i32 = 17;
const STRIP_CELLS: i32 = 4;

/// Modern 标题几何（paint 与触屏命中的**唯一**几何源）。矩形均为 (x, y, w, h) 逻辑 px。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TitleGeom {
    /// 美术整数倍率（≥1）。
    pub art_k: i32,
    /// 美术左上角（横居中；y 负=顶部裁切，底边恒贴底栏上沿）。
    pub art_x: i32,
    pub art_y: i32,
    /// 底栏高（2× 标签 + 上下留白）。
    pub bar_h: i32,
    /// 底栏 UI 倍率。
    pub ui_k: i32,
    /// 当前项标签（图标带 4 格）矩形。
    pub label: (i32, i32, i32, i32),
    /// 左/右箭头基准 x（bob 摆动另加 ±ui_k×bob）与 y。
    pub arrow_left_x: i32,
    pub arrow_right_x: i32,
    pub arrow_y: i32,
    /// 触屏热区：上一项 / 确认 / 下一项（横向无缝铺满画布，贴底）。
    pub zone_prev: (i32, i32, i32, i32),
    pub zone_ok: (i32, i32, i32, i32),
    pub zone_next: (i32, i32, i32, i32),
}

impl TitleGeom {
    /// Modern 几何推导。`art_*`=标题美术逻辑尺寸（menu[0] 240×295）、`arrow_*`=箭头
    /// （ui[14] 20×13）。箭头偏置取 layout 的 arrow_pair（屏心 −60/+40 原偏置）×ui_k。
    pub fn modern(
        layout: &crate::layout::Layout,
        art_w: i32,
        art_h: i32,
        arrow_w: i32,
        arrow_h: i32,
    ) -> TitleGeom {
        let (sw, sh, cx) = (layout.screen_w, layout.screen_h, layout.center_x);
        let ui_k = layout.title_ui_k.max(1);
        let label_w = STRIP_CELL * STRIP_CELLS * ui_k;
        let label_h = STRIP_CELL * ui_k;
        let bar_h = label_h + 2 * MODERN_BAR_PAD;
        let avail_h = sh - bar_h;
        let (aw, ah) = (art_w.max(1), art_h.max(1));
        // 最大整数倍：不超宽、顶裁 ≤ 1/10（两约束随 k 单调 ⇒ 首个失败即停）
        let mut art_k = 1;
        let mut k = 2;
        while aw * k <= sw && ah * k - avail_h <= ah * k / ART_MAX_CROP_DIV {
            art_k = k;
            k += 1;
        }
        let label_x = cx - label_w / 2;
        let zone_h = bar_h + TOUCH_ZONE_EXT * ui_k;
        let zy = sh - zone_h;
        let _ = arrow_w;
        TitleGeom {
            art_k,
            art_x: (sw - aw * art_k) / 2,
            art_y: avail_h - ah * art_k,
            bar_h,
            ui_k,
            label: (label_x, sh - bar_h + MODERN_BAR_PAD, label_w, label_h),
            arrow_left_x: cx + (layout.arrow_pair_left_x - cx) * ui_k,
            arrow_right_x: cx + (layout.arrow_pair_right_x - cx) * ui_k,
            arrow_y: sh - bar_h + ((bar_h - arrow_h * ui_k) >> 1),
            zone_prev: (0, zy, label_x, zone_h),
            zone_ok: (label_x, zy, label_w, zone_h),
            zone_next: (label_x + label_w, zy, sw - (label_x + label_w), zone_h),
        }
    }

    /// 触点命中（逻辑坐标；热区外/画布外=None——美术区不误触）。
    pub fn hit(&self, x: i32, y: i32) -> Option<TitleTouch> {
        let inside = |r: (i32, i32, i32, i32)| x >= r.0 && x < r.0 + r.2 && y >= r.1 && y < r.1 + r.3;
        if inside(self.zone_prev) {
            Some(TitleTouch::Prev)
        } else if inside(self.zone_ok) {
            Some(TitleTouch::Confirm)
        } else if inside(self.zone_next) {
            Some(TitleTouch::Next)
        } else {
            None
        }
    }
}

/// 标题轮播触屏动作。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleTouch {
    Prev,
    Confirm,
    Next,
}

impl TitleTouch {
    /// 投递键码=既有 title tick 臂（a.java:3388-3426）：-3 上移环绕 / -4 下移环绕 /
    /// -5 确认——触屏只合成键，零新游戏状态。
    pub fn key(self) -> i32 {
        match self {
            TitleTouch::Prev => -3,
            TitleTouch::Confirm => -5,
            TitleTouch::Next => -4,
        }
    }
}

/// 水平滑动=切项（左滑=下一项：轮播内容左移；右滑=上一项）；纵滑/阈值内=None。
pub fn title_swipe(dx: i32, dy: i32, min_len: i32) -> Option<TitleTouch> {
    if dx.abs() < min_len || dx.abs() < dy.abs() {
        return None;
    }
    Some(if dx < 0 { TitleTouch::Next } else { TitleTouch::Prev })
}

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

/// 菜单族图标条源宽（4 列 × 17px；drawIconStrip 第 6 参 68，deobf a.java:2960/2969/3000）。
pub const MENU_STRIP_W: i32 = 68;

/// 菜单族图标条左 x = 水平居中于画布：`(screen_w − 68) / 2`。classic 代入 (240−68)/2 = 86 =
/// Java 字面（deobf a.java:2960/2969/3000 `drawIconStrip(…, 86, 10, 0, sy, 68, 17, 17)`，
/// 帮助/设置/图鉴三处同值 ⇒ 居中语义，B 级）。【2026-10-26】Modern 画布随宽居中。
pub fn menu_strip_x(screen_w: i32) -> i32 {
    (screen_w - MENU_STRIP_W) / 2
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
        // 【C 方案 2026-10-26】Modern=触屏轮播版（美术整数倍+底栏 ×2，几何单源）
        if layout.preset == crate::layout::Preset::Modern {
            self.paint_modern_touch(g, map_bg, strip, arrow, layout);
            return;
        }
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
            // > 【2026-10-26 更正】原 Modern 分支 `draw_image_cover` 铺满（2026-10-23
            // 问题1"贴图满幅"）被用户实机反馈"放太大"推翻：240×295 放 2.4×（非整数倍
            // ⇒ 像素 2/3 格交替不均）、顶裁 19%（人物头被切）、底部 logo 被底栏盖住。
            // Modern 已改走 `paint_modern_touch`（2× 整数倍，见上方早返回）；本分支只剩
            // 非 Modern 大画幅（公开构造不可达，保留原尺寸居中原式）。
            g.draw_image(map_bg, img_x, img_y, 0);
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

    /// 【C 方案 2026-10-26】Modern 标题绘制：美术**整数倍**（像素均匀，底贴底栏、
    /// 顶裁 ≤1/10）+ 底栏/当前项标签/双箭头 ×ui_k。几何=[`TitleGeom::modern`] 单源
    /// （触屏命中同源）。延展带=顶带同款纯蓝（2026-10-19 用户验收口径不变）。
    fn paint_modern_touch(
        &mut self,
        g: &mut SoftGraphics<'_>,
        map_bg: &ArgbImage,
        strip: &ArgbImage,
        arrow: &ArgbImage,
        layout: &crate::layout::Layout,
    ) {
        let (sw, sh) = (layout.screen_w, layout.screen_h);
        let (aw, ah) = (arrow.logical_width(), arrow.logical_height());
        let geo = TitleGeom::modern(layout, map_bg.logical_width(), map_bg.logical_height(), aw, ah);
        let top_px = map_bg.argb[0] & 0x00FF_FFFF | 0xFF00_0000;
        g.set_color(top_px);
        g.fill_rect(0, 0, sw, sh);
        g.draw_image_scaled(map_bg, geo.art_x, geo.art_y, geo.art_k);
        g.set_color(crate::layout::DARK_BACKDROP);
        g.fill_rect(0, sh - geo.bar_h, sw, geo.bar_h);
        let sy = STRIP_CELL * self.menu_kinds[self.cursor as usize] as i32;
        self.paint_icon_strip_scaled(g, strip, geo.label.0, geo.label.1, sy, geo.ui_k);
        // 双箭头（左=水平翻转，诺基亚 8192 同义；bob 摆幅同倍）
        let k = geo.ui_k;
        g.draw_region_scaled(arrow, 0, 0, aw, ah, geo.arrow_left_x - k * self.bob, geo.arrow_y, k, true);
        g.draw_region_scaled(arrow, 0, 0, aw, ah, geo.arrow_right_x + k * self.bob, geo.arrow_y, k, false);
        // 粒子层（原式 clip；生成域随画布 ①a）
        let bar = self.bar_height;
        g.set_clip(0, bar, sw, sh - (bar >> 1));
        self.particles.paint_and_update(g);
        g.reset_clip();
    }

    /// drawIconStrip 的 ×k 形（列宽/波动幅 ×k；帧推进与 [`Self::paint_icon_strip`] 逐字同）。
    fn paint_icon_strip_scaled(
        &mut self,
        g: &mut SoftGraphics<'_>,
        strip: &ArgbImage,
        x0: i32,
        y0: i32,
        sy: i32,
        k: i32,
    ) {
        let mut frame = self.strip_frame;
        let mut src_x = 0;
        let mut base_x = x0;
        while src_x < 60 {
            let cx = base_x + STRIP_DX[frame] * k;
            let cy = y0 + STRIP_DY[frame] * k;
            g.draw_region_scaled(strip, src_x, sy, STRIP_CELL, STRIP_CELL, cx, cy, k, false);
            frame += 1;
            if frame > 7 {
                frame = 0;
            }
            src_x += STRIP_CELL;
            base_x += STRIP_CELL * k;
        }
        self.strip_frame += 1;
        if self.strip_frame > 7 {
            self.strip_frame = 0;
        }
    }

    /// > 【2026-10-26 更正】原 `menu_item_rects`/`menu_hit`（2c 裁定：6 条行矩形叠在
    /// > 标题美术上，tap 行 i=选中+确认）已撤除：行**不可见**（paint 从未画行，可见
    /// > 菜单=底栏轮播单项），实机点"开始游戏"文字落行外、点到第 6 行=退出
    /// > （LogoAnim）误触。由用户裁定 C 方案取代：轮播热区 [`TitleGeom::hit`]
    /// > （与 paint 同源）+ [`title_swipe`]。
    ///
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

#[cfg(test)]
mod touch_carousel_tests {
    //! 【STAGE-B 触屏标题 C 方案（2026-10-26 用户裁定）】Modern 标题几何单源
    //! （paint 与触屏命中共用）+ 轮播触屏语义 + 2× 整数倍美术。
    use super::*;
    use crate::layout::Layout;

    /// 真素材尺寸：menu[0] 标题美术 240×295、ui[14] 箭头 20×13（assets/raw 实测）。
    fn geom(grid: i32) -> TitleGeom {
        TitleGeom::modern(&Layout::modern(grid, grid), 240, 295, 20, 13)
    }

    #[test]
    fn geom_576_integer_2x_art_bottom_on_bar_and_finger_zones() {
        let g = geom(18);
        assert_eq!((g.art_k, g.art_x, g.art_y), (2, 48, -54), "2× 整数倍、横居中、底贴底栏（顶裁 54）");
        assert_eq!(g.bar_h, 40, "底栏=2× 标签 34 + 上下留白 3");
        assert_eq!(g.art_y + 295 * g.art_k, 576 - g.bar_h, "美术底边=底栏上沿（logo 不被盖）");
        assert_eq!(g.label, (220, 539, 136, 34), "标签 2×（4 格×17×2）屏心居中");
        assert_eq!((g.arrow_left_x, g.arrow_right_x, g.arrow_y), (168, 368, 543), "箭头=原偏置×2");
        assert_eq!(g.zone_prev, (0, 504, 220, 72));
        assert_eq!(g.zone_ok, (220, 504, 136, 72));
        assert_eq!(g.zone_next, (356, 504, 220, 72));
    }

    #[test]
    fn geom_art_scale_is_integer_and_crop_bounded_across_grids() {
        for grid in 13..=24 {
            let g = geom(grid);
            let (sw, sh) = (grid * 32, grid * 32);
            assert!(g.art_k >= 1 && 240 * g.art_k <= sw, "grid {grid}：美术不超宽");
            let crop = (295 * g.art_k - (sh - g.bar_h)).max(0);
            assert!(crop * 10 <= 295 * g.art_k, "grid {grid}：顶裁 ≤10%（实裁 {crop}）");
            assert_eq!(g.art_y + 295 * g.art_k, sh - g.bar_h, "grid {grid}：底贴底栏");
            // 热区横向无缝铺满、互不重叠
            assert_eq!(g.zone_prev.0, 0);
            assert_eq!(g.zone_prev.0 + g.zone_prev.2, g.zone_ok.0);
            assert_eq!(g.zone_ok.0 + g.zone_ok.2, g.zone_next.0);
            assert_eq!(g.zone_next.0 + g.zone_next.2, sw);
            assert_eq!(g.zone_ok.1 + g.zone_ok.3, sh, "热区贴底");
            assert!(g.zone_ok.3 >= g.bar_h, "热区覆盖底栏");
        }
        assert_eq!(geom(13).art_k, 1, "416 画幅放不下 2×（480>416）⇒ 1×");
        assert_eq!(geom(24).art_k, 2, "768 画幅 3× 顶裁 157>88 ⇒ 2×");
    }

    #[test]
    fn touch_zones_map_to_existing_key_arms() {
        let g = geom(18);
        let mid = |r: (i32, i32, i32, i32)| (r.0 + r.2 / 2, r.1 + r.3 / 2);
        let (px, py) = mid(g.zone_prev);
        let (ox, oy) = mid(g.zone_ok);
        let (nx, ny) = mid(g.zone_next);
        assert_eq!(g.hit(px, py), Some(TitleTouch::Prev));
        assert_eq!(g.hit(ox, oy), Some(TitleTouch::Confirm));
        assert_eq!(g.hit(nx, ny), Some(TitleTouch::Next));
        assert_eq!(g.hit(ox, g.zone_ok.1 - 1), None, "热区上沿外=美术区不误触");
        assert_eq!(g.hit(-1, oy), None, "画布外（侧余负坐标）=None");
        // 键码=既有 title tick 臂（-3 上移环绕 / -4 下移环绕 / -5 确认）
        assert_eq!(TitleTouch::Prev.key(), -3);
        assert_eq!(TitleTouch::Next.key(), -4);
        assert_eq!(TitleTouch::Confirm.key(), -5);
    }

    #[test]
    fn horizontal_swipe_switches_item_vertical_ignored() {
        assert_eq!(title_swipe(-40, 5, 24), Some(TitleTouch::Next), "左滑=下一项（轮播内容左移）");
        assert_eq!(title_swipe(40, -5, 24), Some(TitleTouch::Prev), "右滑=上一项");
        assert_eq!(title_swipe(5, 60, 24), None, "纵滑不切项");
        assert_eq!(title_swipe(10, 0, 24), None, "阈值内=非滑动");
    }

    #[test]
    fn modern_paint_uses_geom_single_source() {
        let layout = Layout::modern(18, 18);
        let g0 = TitleGeom::modern(&layout, 240, 295, 20, 13);
        // 美术：逐像素可辨（x/y 编进颜色），顶行纯蓝
        let mut art = ArgbImage::create(240, 295);
        for y in 0..295 {
            for x in 0..240 {
                art.argb[(y * 240 + x) as usize] =
                    if y == 0 { 0xFF3388CC } else { 0xFF000000 | ((x as u32) << 16) | ((y as u32) << 8) | 0x55 };
            }
        }
        let mut strip = ArgbImage::create(68, 102);
        strip.argb.fill(0xFFEE1111);
        let mut arrow = ArgbImage::create(20, 13);
        arrow.argb.fill(0xFF11EE11);
        let mut canvas = ArgbImage::create(576, 576);
        let mut t = TitleMachine::new();
        {
            let mut g = SoftGraphics::new(&mut canvas);
            t.paint(&mut g, &art, &strip, &arrow, &layout);
        }
        let at = |x: i32, y: i32| canvas.argb[(y * 576 + x) as usize];
        // 2× 整数倍：源 (u,v) → 2×2 逻辑块（均匀），左上块角=art_x+2u, art_y+2v
        for (u, v) in [(10, 40), (100, 150), (239, 294)] {
            let src = art.argb[(v * 240 + u) as usize];
            let (x, y) = (g0.art_x + 2 * u, g0.art_y + 2 * v);
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                assert_eq!(at(x + dx, y + dy), src, "源({u},{v}) 2×2 块均匀");
            }
        }
        assert_eq!(at(5, 200), 0xFF3388CC, "左延展带=顶带同款纯蓝（10-19 验收口径不变）");
        assert_eq!(at(570, 200), 0xFF3388CC, "右延展带同");
        assert_eq!(at(5, 575), 0xFF000000 | crate::layout::DARK_BACKDROP, "底栏=暗底色 40 高");
        assert_eq!(at(5, 576 - g0.bar_h), 0xFF000000 | crate::layout::DARK_BACKDROP, "底栏上沿行");
        // 标签 2×：标签首格内（列波动 ±2 不出格）=图标带色
        let (lx, ly, _lw, lh) = g0.label;
        assert_eq!(at(lx + 17, ly + lh / 2), 0xFFEE1111, "标签 2× 落在 geom.label");
        // 箭头 2×：左/右箭头基准 x 起 40×26（bob=0 首帧）
        assert_eq!(at(g0.arrow_left_x + 20, g0.arrow_y + 13), 0xFF11EE11, "左箭头 2×");
        assert_eq!(at(g0.arrow_right_x + 20, g0.arrow_y + 13), 0xFF11EE11, "右箭头 2×");
    }
}
