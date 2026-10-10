//! mode 1 标题主菜单（paint case 1，a.java:2349-2376 + run case 1，a.java:3374-3429）。
//!
//! # 结构（A 级证据）
//!
//! - **paint**（a.java:2349-2376）：map 背景图 → 底部状态条 → 图标带动画
//!   （drawIconStrip，a.java:4780-4793）→ 双侧羽化箭头（bob 摆动）→ 粒子层
//!   （updateAndDrawParticles，clip 0..308）。
//! - **tick**（a.java:3374-3429）：`(frameCounter&3)==0` 时 spawn 粒子（每次
//!   吃两个 randomBelow：先 x=rb(240) 后 y=295-rb(150)）；advanceBobPhase bob 摆动；
//!   方向键移动 cursor（环绕）；-5 确认按 menuItemKinds[cursor] 切 gameMode。
//! - **粒子演化**（updateAndDrawParticles a.java:9811-9829）：槽位 **31→0 倒序**；每 paint 先
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
    /// particleColor：粒子颜色（m_000 case 1 设 -1 = 白）。
    pub color: u32,
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
            color: 0xFFFFFF,
        }
    }

    /// `spawnParticle`（a.java:9797-9809）：环形游标登记新粒子。
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

    /// `updateAndDrawParticles`（a.java:9811-9829）：**paint 内调用**——先画后推进，槽位 31→0 倒序。
    pub fn paint_and_update(&mut self, g: &mut SoftGraphics<'_>) {
        g.set_color(self.color);
        for i in (0..PARTICLE_SLOTS).rev() {
            if !self.active[i] {
                continue;
            }
            let s = self.size[i];
            g.fill_rect(self.x[i], self.y[i], s, s);
            // 推进（a.java:9812-9828）
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

/// `drawIconStrip` 通用形（a.java:4783-4801）。`frame` 为 iconStripFrame
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
    g.set_clip(0, 0, 240, 320);
}

/// mode 1 标题菜单状态机（run case 1 的 tick + paint case 1 的绘制输入）。
pub struct TitleMachine {
    /// menuCursorIndex：菜单光标（0..menu_item_count-1 环绕）。
    pub cursor: i32,
    /// menuItemKinds 的条目（m_000 case 1 经 addMenuItem(0..5) 登记为 0..5）。
    pub menu_kinds: Vec<i8>,
    /// animBobOffset：bob 摆动偏移（advanceBobPhase 演化）。
    pub bob: i32,
    /// animBobRising：bob 方向（false=下降相 --，true=上升相 ++）。
    bob_rising: bool,
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
            // m_000 case 1：resetMenuLayout 重置后 addMenuItem(0..5)（a.java:4281-4287）
            menu_kinds: vec![0, 1, 2, 3, 4, 5],
            bob: 0,
            bob_rising: false,
            strip_frame: 0,
            particles: Particles::new(),
            bar_height: 25,
        }
    }

    /// `advanceBobPhase`（a.java:6453-6462）：bob ±1 摆动（下降相 --<-1 翻上升；上升 ++>1 翻下降）。
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

    /// run case 1 的逻辑 tick（a.java:3374-3429）。
    ///
    /// `frame_counter` = 本拍的 frameCounter 值（迭代 N 拍 = N-1）。
    /// 返回 `Some(new_mode)` 当确认键切模式（调用方负责后续状态机移交）。
    pub fn tick(&mut self, key: i32, frame_counter: i32, rng: &mut JavaRandom) -> Option<i32> {
        if frame_counter & 3 == 0 {
            // a.java:3376：参数求值序 L2R——先 x=rb(240) 后 y=295-rb(150)
            let x = rng.random_below(240);
            let y = 320 - self.bar_height - rng.random_below(150);
            self.particles.spawn(x, y);
        }
        self.tick_bob();
        match key {
            // 确认（a.java:3382-3415）：按 menuItemKinds[cursor] 切模式
            -5 | 53 => {
                let next = match self.menu_kinds[self.cursor as usize] {
                    0 => 14,
                    1 => 8,
                    2 => 16,
                    3 => 15,
                    4 => 17,
                    5 => 22,
                    other => other,
                };
                Some(next as i32)
            }
            // 下移（a.java:3408-3417）
            -4 | -2 | 54 | 56 => {
                if self.cursor < self.menu_kinds.len() as i32 - 1 {
                    self.cursor += 1;
                } else {
                    self.cursor = 0;
                }
                None
            }
            // 上移（a.java:3418-3426）
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
    pub fn paint(&mut self, g: &mut SoftGraphics<'_>, map_bg: &ArgbImage, strip: &ArgbImage, arrow: &ArgbImage) {
        let bar = self.bar_height;
        // ① 顶部补条（a.java:2352-2355：间隙 >0 才画）
        let top_gap = 320 - bar - map_bg.height;
        if top_gap > 0 {
            g.set_color(crate::layout::TITLE_TOP_GAP);
            g.fill_rect(0, 0, 240, top_gap);
        }
        // ② 背景图（a.java:2357）
        g.draw_image(map_bg, 0, top_gap, 0);
        // ③ 底部条（a.java:2358-2359）
        g.set_color(crate::layout::DARK_BACKDROP);
        g.fill_rect(0, 320 - bar, 240, bar);
        // ④ 图标带（a.java:2360-2369 + drawIconStrip）
        let strip_y = 320 - bar + (bar - 17 >> 1);
        self.paint_icon_strip(g, strip, 86, strip_y, 17 * self.menu_kinds[self.cursor as usize] as i32);
        // ⑤ 双侧箭头（a.java:2370-2372）：左=镜像变换（表值 8192），右=原样
        let arrow_y = 320 - bar + (bar - 13 >> 1);
        let t = crate::paint::NOKIA_TRANSFORM_TABLE[1];
        g.draw_image_transformed(arrow, 60 - self.bob, arrow_y, 0, t);
        g.draw_image(arrow, 160 + self.bob, arrow_y, 0);
        // ⑥ 粒子层（clip 0..308，a.java:2373-2375）
        g.set_clip(0, bar, 240, 320 - (bar >> 1));
        self.particles.paint_and_update(g);
        g.set_clip(0, 0, 240, 320);
    }

    /// `drawIconStrip`（a.java:4780-4793）：4 列图标带（列源 x = 0/17/34/51），
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
        g.set_clip(0, 0, 240, 320);
    }
}
