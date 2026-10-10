//! mode 3 游戏画面场景（a.java paint case 3 :2385-2422 + run case 3 :3545-3676）。
//!
//! 绘制链（paint 顺序，T544 实证 ops 同序）：
//! `drawParallaxBackdrop(true)` → `paintTileLayer(0,20)` → `m_053` 实体层
//! （含玩家 m_033 插入 + 楼梯浮标）→ 小地图 → `m_037` HUD → `m_035` 状态栏
//! → 楼梯指示（f_bool_13）→ `drawPopupLayer` → `paintSoftkeyBar`。
//!
//! 逻辑链（每 tick）：keyValue 开关 → walkPhase 状态机 → `m_055` 帧推进。
//! bob（m_052）在 **paint** 内推进（m_053 首行调用，一拍一次）。
//!
//! P3.5 对拍窗口：gameplay-floor1 T544-599 稳态（无输入）——popup 环形队列、
//! walkPhase 1 步进、m_104 交互为后续批次（spec p3-render §3.5 待办）。

use crate::entity::{self, CellGrid, EntityTable, ANIM_OFFSET_TABLE};
use crate::game_view::GameView;
use crate::menu_family::{paint_mini_frame, paint_number};
use crate::paint::{draw_image_clipped, m_034_softkeys};
use crate::render::{ArgbImage, SoftGraphics};

/// f_Image_arr2_00 的 16 容器名序（a.java:451）。本场景常驻引用见 [`SceneImages`]。
pub const CONTAINER_NAMES: [&str; 16] = [
    "sflogo", "mapbg", "map", "actor", "sptmap", "sptprop", "sptarm", "sptenemy1", "ui", "xtq",
    "menu", "intro", "face", "sptenemy2", "end", "load",
];

/// 场景图像集（按需装载的容器子集 + m_122 实体类型→精灵条带映射产物）。
pub struct SceneImages {
    /// f_Image_arr2_00[1][0]：视差背景瓦（77x320）
    pub backdrop: ArgbImage,
    /// f_Image_arr2_00[5][0]：通用阴影
    pub shadow: ArgbImage,
    /// f_Image_arr2_00[3][0]：玩家三向精灵条带（123x138 = 3 行 41x46）
    pub actor: ArgbImage,
    /// f_Image_arr2_00[3][1]：HUD 头像
    pub actor_face: ArgbImage,
    /// f_Image_arr2_00[8][*]：ui 容器（25 张）
    pub ui: Vec<ArgbImage>,
    /// f_Image_arr2_00[2][1]/[2][2]：上/下楼浮标
    pub stair_icons: (ArgbImage, ArgbImage),
    /// f_Image_arr2_00[2][9]：开格动画贴图（27x29）
    pub open_anim: ArgbImage,
    /// f_Image_arr2_00[4][12]：已开门贴图
    pub opened_door: ArgbImage,
    /// f_Image_arr_00[type]：实体类型→精灵条带（null → None；m_122 映射）
    pub entity: Vec<Option<ArgbImage>>,
}

impl SceneImages {
    /// m_122 头部的类型→容器映射（a.java:9425-9453）：
    /// 1..12→[4][t-1]，13..32→[5][t-12]，33..40→[6][t-33]，41..60→[7][t-41]，
    /// 61..78→[13][t-61]，84→[13][18]，87→[13][19]，85→[5][21]，86→[5][22]，
    /// 79→[6][8]，80→[6][9]，12→=11，81→[4][3]，71→=70。
    /// 容器计数来自 oracle FLD 018（T500 实测）：sptmap=13、sptprop=23、
    /// sptarm=10、sptenemy1=20、sptenemy2=20。
    pub fn map_entity_images(
        sptmap: &[ArgbImage],
        sptprop: &[ArgbImage],
        sptarm: &[ArgbImage],
        sptenemy1: &[ArgbImage],
        sptenemy2: &[ArgbImage],
    ) -> Vec<Option<ArgbImage>> {
        let mut out: Vec<Option<ArgbImage>> = vec![None; 89];
        let at = |c: &[ArgbImage], i: usize| -> Option<ArgbImage> { c.get(i).cloned() };
        for t in 1..=12 {
            out[t] = at(sptmap, t - 1);
        }
        for t in 13..=32 {
            out[t] = at(sptprop, t - 12);
        }
        for t in 33..=40 {
            out[t] = at(sptarm, t - 33);
        }
        for t in 41..=60 {
            out[t] = at(sptenemy1, t - 41);
        }
        for t in 61..=78 {
            out[t] = at(sptenemy2, t - 61);
        }
        out[84] = at(sptenemy2, 18);
        out[87] = at(sptenemy2, 19);
        out[85] = at(sptprop, 21);
        out[86] = at(sptprop, 22);
        out[79] = at(sptarm, 8);
        out[80] = at(sptarm, 9);
        out[12] = out[11].clone();
        out[81] = at(sptmap, 3);
        out[71] = out[70].clone();
        out
    }
}

/// 玩家帧表 f_int_arr2_01[0]（a.java:477：{0,1,0,2}）。
const PLAYER_FRAME_TABLE: [i32; 4] = [0, 1, 0, 2];

/// mode 3 场景状态。
pub struct GameScene {
    pub view: GameView,
    pub entities: EntityTable,
    pub grid: CellGrid,
    pub images: SceneImages,
    /// f_bool_arr2_00：可行走网格（rebuildWalkability，m_057 小地图用）
    pub walkable: Vec<Vec<bool>>,
    /// f_Image_03：小地图（m_057 生成；None = optionChecked[1]=false）
    pub minimap: Option<ArgbImage>,
    /// optionChecked[1]：小地图开关（RMS 读档失败 catch 默认 true，a.java:4451）
    pub minimap_enabled: bool,
    /// frameCounter（run 循环变量，TICK n 拍 = n-1）
    pub frame_counter: i64,
    /// m_052 浮沉相位（paint 内推进）
    pub bob_offset: i32,
    pub bob_rising: bool,
    /// drawParallaxBackdrop 的滚动相位（每 paint -1，<-154 → 0）
    pub backdrop_scroll: i32,
    /// f_int_48：视口底（HUD 顶）
    pub view_bottom: i32,
    /// 玩家态（m_033 输入）
    pub facing: i32,
    pub walk_phase: i32,
    pub player_frame: i32,
    pub player_bob_applied: bool, // f_bool_07
    /// HUD 数值（m_037/m_035 输入）
    pub floor: i32,
    pub hp: i32,
    pub atk: i32,
    pub def: i32,
    pub keys: (i32, i32, i32), // 黄/蓝/红
    pub gold: i32,
    /// 软键（m_000 case 3：1/3）
    pub softkeys: (i8, i8),
}

impl GameScene {
    /// loadFloorData（a.java:9375-9415）：maplv 装载 + m_059 格索引重建 +
    /// 可行走重建 + m_122 实体生成 + m_054 排序。
    /// `sprite_records`：sprite{n} 权威解析产物（type, x, y, param, visible）。
    #[allow(clippy::too_many_arguments)]
    pub fn load_floor(
        maplv: &[u8],
        player_px: i32,
        player_py: i32,
        records: &[(u8, u16, u16, i16, bool)],
        width_table: &[i32],
        tile_walkability: &[i32],
        images: SceneImages,
    ) -> Result<GameScene, String> {
        let view = GameView::from_maplv(maplv, player_px, player_py)?;
        let cols = view.wide as usize;
        let rows = view.high as usize;
        let mut walkable = vec![vec![false; cols]; rows];
        for y in 0..rows {
            for x in 0..cols {
                // rebuildWalkability（a.java:6941-6959）的索引：**行距 wide·4、
                // 列距 2**（var1 行尾 +=wide<<1 叠加列循环的 ×2）——与 paintTileLayer
                // 的行距 wide·2、列距 1 是同一数组的两套读法（原版共存）。
                // A 级：oracle FLD 134（f_bool_arr2_00）13×13 逐格复算匹配。
                let idx = (y * cols * 4 + x * 2) as i32;
                let tile = view.terrain_at(0, idx) as usize;
                walkable[y][x] = tile < tile_walkability.len() && tile_walkability[tile] == 1;
            }
        }
        let mut entities = EntityTable::new(100);
        let mut grid = CellGrid::new(view.wide, view.high);
        for &(type_code, x, y, param, visible) in records {
            let img_h = images.entity[type_code as usize].as_ref().map(|i| i.height);
            let idx = entity::spawn(
                &mut entities,
                &mut grid,
                type_code as i32,
                x as i32,
                y as i32,
                param,
                width_table,
                img_h,
            );
            if !visible {
                entities.visible[idx] = false; // m_122 特例分支（5/81/6/12/76/82/83）
            }
        }
        entity::sort_by_y(&mut entities, &mut grid);
        Ok(GameScene {
            view,
            entities,
            grid,
            images,
            walkable,
            minimap: None,
            minimap_enabled: true,
            frame_counter: 0,
            bob_offset: 0,
            bob_rising: false,
            backdrop_scroll: 0,
            view_bottom: 270,
            facing: 0,
            walk_phase: 0,
            player_frame: 0,
            player_bob_applied: false,
            floor: 1,
            hp: 0,
            atk: 0,
            def: 0,
            keys: (0, 0, 0),
            gold: 0,
            softkeys: (1, 3),
        })
    }

    /// 加载步 11（a.java:3501-3504）：进游戏视图 + 居中相机。
    pub fn enter_view(&mut self) {
        self.view.center_on_player();
    }

    /// m_057（a.java:6856-6924）：小地图生成（52x52 = 13 格 × 4px，dim 170）。
    pub fn build_minimap(&mut self) {
        self.minimap = None;
        if !self.minimap_enabled {
            return;
        }
        let w = self.view.wide << 2;
        let h = self.view.high << 2;
        let mut img = ArgbImage::create(w, h);
        {
            let mut g = SoftGraphics::new(&mut img);
            g.set_color(13097429);
            g.fill_rect(0, 0, w, h);
            for y in 0..self.view.high as usize {
                for x in 0..self.view.wide as usize {
                    let px = (x as i32) << 2;
                    let py = (y as i32) << 2;
                    if self.walkable[y][x] {
                        g.set_color(7509153);
                        g.fill_rect(px, py, 4, 4);
                    }
                    let ct = self.grid.cell_type[y][x] as usize;
                    if ct > 0 && self.grid.capacity[ct] > 0 {
                        let first = self.grid.slots[ct][0] as usize - 1;
                        if !self.entities.removed[first] && self.entities.solid[first] != 1 {
                            match self.entities.entity_type[first] {
                                1 => {
                                    g.set_color(15461198);
                                    g.fill_rect(px, py, 3, 3);
                                    g.set_color(9794048);
                                    g.draw_line(px + 3, py, px + 3, py + 3);
                                    g.draw_line(px, py + 3, px + 3, py + 3);
                                }
                                // case 2 无 break 落穿 case 3（原版行为，floor1 无 2/3 型）
                                2 | 3 => {
                                    if self.entities.entity_type[first] == 2 {
                                        g.set_color(16273480);
                                        g.fill_rect(px, py, 4, 4);
                                        g.set_color(8388608);
                                        g.draw_line(px + 3, py, px + 3, py + 3);
                                        g.draw_line(px, py + 3, px + 3, py + 3);
                                    }
                                    g.set_color(4767984);
                                    g.fill_rect(px, py, 3, 3);
                                    g.set_color(549016);
                                    g.draw_line(px + 3, py, px + 3, py + 3);
                                    g.draw_line(px, py + 3, px + 3, py + 3);
                                }
                                7 => m_003_clip5(&mut g, &self.images.ui[12], px - 1, py - 1, 5),
                                8 => m_003_clip5(&mut g, &self.images.ui[12], px - 1, py - 1, 0),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        self.minimap = Some(crate::intro::dim_image(&img, 170));
    }

    /// run case 3 的稳态子集（a.java:3545-3676）：无输入时仅 m_055。
    /// `frame_counter` 为本拍值（TICK n → n-1）。
    pub fn tick(&mut self, frame_counter: i64, width_table: &[i32]) {
        self.frame_counter = frame_counter;
        entity::advance_frames(&mut self.entities, &mut self.grid, frame_counter, width_table);
        // walkPhase 状态机（步进/交互）与 popup 环队列属移动批次（spec 待办）
    }

    /// paint case 3（a.java:2385-2422）。
    pub fn paint(&mut self, g: &mut SoftGraphics<'_>, tileset: &ArgbImage) {
        self.paint_parallax(g, true);
        self.view.paint_tiles(g, tileset, 20);
        self.paint_entities(g, self.view.cam_x, self.view.cam_y + 20);
        // 小地图（optionChecked[1] && f_Image_03）
        if self.minimap_enabled {
            if let Some(mm) = &self.minimap {
                let x = 240 - ((self.view.wide + 1) << 2);
                g.draw_image(mm, x, 20, 0);
                g.set_color(1112072);
                g.fill_rect(
                    x + ((self.view.player_px >> 5) << 2),
                    20 + ((self.view.player_py >> 5) << 2),
                    4,
                    4,
                );
            }
        }
        self.paint_hud(g);
        self.paint_status_bar(g);
        // 楼梯指示（f_bool_13；稳态 false—— proximity 更新在 m_104/行走批次）
        // drawPopupLayer：popup 空（retire == write）时不绘制
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1);
    }

    /// drawParallaxBackdrop（a.java:6827-6854）：滚动 -1/拍（<-154 回 0），
    /// 77px 列步进、320px 行步进、列内交替镜像。
    fn paint_parallax(&mut self, g: &mut SoftGraphics<'_>, game_view: bool) {
        self.backdrop_scroll -= 1;
        if self.backdrop_scroll < -154 {
            self.backdrop_scroll = 0;
        }
        let base_y = if game_view { self.view_bottom - 320 + 6 } else { 0 };
        let mut col_x = self.backdrop_scroll;
        while col_x < 240 {
            let mut mirrored = false;
            let mut y = base_y;
            while y > -320 {
                if mirrored {
                    g.draw_image_transformed(&self.images.backdrop, col_x, y, 0, 2);
                } else {
                    g.draw_image(&self.images.backdrop, col_x, y, 0);
                }
                mirrored = !mirrored;
                y -= 320;
            }
            col_x += 77;
        }
    }

    /// m_053（a.java:6468-6704）：实体层。`cam_x/cam_y` 为调用参数
    /// （paint case 3：cameraPixelX / cameraPixelY+20）。
    /// floor1 类别覆盖 1/2/8 + 空图文字标签；16/32/67/69 拼装 todo（floor1
    /// 实体清单无这些类型，spec p3-render §3.5）。
    pub fn paint_entities(&mut self, g: &mut SoftGraphics<'_>, cam_x: i32, cam_y: i32) {
        entity::advance_bob(&mut self.bob_offset, &mut self.bob_rising);
        let bob = self.bob_offset;
        let mut player_drawn = false;

        for e in 0..self.entities.count {
            if !self.entities.removed[e] && self.entities.visible[e] {
                let ex = self.entities.pixel_x[e];
                let ey = self.entities.pixel_y[e];
                let w = self.entities.sprite_w[e];
                let h = self.entities.sprite_h[e];
                let state = self.entities.solid[e];
                let t = self.entities.entity_type[e];
                let mut sx = cam_x + ex;
                let mut sy = cam_y + ey;
                if sx >= -w && sx <= self.view.view_w && sy >= -12 && sy <= 20 + self.view.view_h {
                    if let Some(img) = &self.images.entity[t as usize] {
                        // 玩家插入（单次；f_bool_arr_02 全 false——m_043 对新数组拷贝）
                        if !player_drawn
                            && self.view.player_px > ex - 32
                            && self.view.player_px < ex + 32
                            && self.view.player_py >= ey - 32
                            && ey > self.view.player_py
                        {
                            self.paint_player(g, cam_x, cam_y);
                            player_drawn = true;
                        }
                        let anim = self.entities.anim_idx[e] as usize;
                        let frame = self.entities.frame[e];
                        let frame_off = w * ANIM_OFFSET_TABLE[anim][frame as usize];
                        match entity::render_category(t) {
                            1 => {
                                if t == 6 {
                                    sy += 5 - bob;
                                } else if t == 9 {
                                    sx += 32;
                                }
                                if state != 1 {
                                    draw_image_clipped(g, img, sx + ((32 - w) >> 1), sy - (h - 32), frame_off, 0, w, h);
                                } else {
                                    draw_image_clipped(g, &self.images.opened_door, sx + ((32 - w) >> 1), sy - (h - 32), frame_off, 0, w, h);
                                }
                            }
                            2 => {
                                if state == 1 {
                                    draw_image_clipped(g, &self.images.opened_door, sx + ((32 - w) >> 1), sy - (h - 32), frame_off, 0, w, h);
                                } else if state == 2 {
                                    draw_image_clipped(g, &self.images.open_anim, sx + 2, sy + 2 - (self.entities.frame[e] << 3), frame_off, 0, 27, 29);
                                } else if t < 26 {
                                    g.draw_image(&self.images.shadow, sx + 8, sy + 22, 0);
                                    g.draw_image(img, sx + ((32 - w) >> 1), sy - (h - 24) + bob, 0);
                                } else {
                                    g.draw_image(img, sx + ((32 - w) >> 1), sy - (h - 30), 0);
                                }
                            }
                            4 => {
                                g.draw_image(&self.images.shadow, sx + 8, sy + 22, 0);
                                draw_image_clipped(g, img, sx + ((32 - w) >> 1), sy - (h - 24) + bob, frame_off, 0, w, h);
                            }
                            8 => {
                                if state == 3 {
                                    // var13==3 抖动（randomBelow(5)-2）——floor1 无此态
                                    todo!("m_053 case 8 var13==3 抖动（floor1 实体清单无此态）")
                                }
                                if state == 1 || state == 2 {
                                    draw_image_clipped(g, &self.images.open_anim, sx + 2, sy + 2 - (self.entities.frame[e] << 3), frame_off, 0, 27, 29);
                                } else if t != 67 && t != 69 {
                                    g.draw_image(&self.images.shadow, sx + 8, sy + 22, 0);
                                    sy -= 4;
                                    draw_image_clipped(g, img, sx + ((32 - w) >> 1), sy - (h - 32), frame_off, 0, w, h);
                                } else {
                                    // 67/69 拼装表组合（f_byte_arr2_00/01 元数据）——floor1 无
                                    todo!("m_053 case 8 的 67/69 拼装组合（floor1 实体清单无）")
                                }
                                if self.walk_phase == 2 {
                                    todo!("m_053 case 8 walkPhase==2 交互浮标（战斗后批次）")
                                }
                            }
                            16 => {
                                if state != 1 && state != 2 {
                                    if let Some(img15) = &self.images.entity[15] {
                                        g.draw_image(img15, sx + 1, sy + 8 + bob, 0);
                                    }
                                    draw_image_clipped(g, img, sx + ((32 - w) >> 1), sy - (h - 24) + bob, frame_off, 0, w, h);
                                } else {
                                    draw_image_clipped(g, &self.images.open_anim, sx + 2, sy + 2 - (self.entities.frame[e] << 3), frame_off, 0, 27, 29);
                                }
                            }
                            32 => {
                                g.draw_image(&self.images.shadow, sx + 8, sy + 22, 0);
                                if state == 1 || state == 2 {
                                    draw_image_clipped(g, &self.images.open_anim, sx + 2, sy + 2 - (self.entities.frame[e] << 3), frame_off, 0, 27, 29);
                                } else if t != 72 {
                                    draw_image_clipped(g, img, sx + ((32 - w) >> 1), sy - (h - 16) + bob, frame_off, 0, w, h);
                                } else {
                                    todo!("m_053 case 32 的 72 型多形态（f_byte_19 倍率 + m_127）")
                                }
                            }
                            _ => {}
                        }
                    } else {
                        // 空图文字标签（objectTypeNames 圆底标签；type 83 走此径但 m_122 置 visible=false）
                        todo!("m_053 空图标签分支（objectTypeNames 文本，floor1 不可达）")
                    }
                }
            }
        }

        g.set_clip(0, 0, 240, 320);
        if !player_drawn {
            self.paint_player(g, cam_x, cam_y);
        }
        // walkPhase==5 星光（f_int_44 >= 0）：战斗批次
        // 楼梯浮标（m_053 尾段 a.java:6689-6703）
        if let Some((ux, uy)) = self.entities.stair_up {
            let x = cam_x + ux;
            let y = cam_y + uy;
            if x >= -32 && x <= self.view.view_w && y >= -12 && y <= 20 + self.view.view_h {
                g.draw_image(&self.images.stair_icons.0, x + 5, y - 30 + bob, 0);
            }
        }
        if let Some((dx, dy)) = self.entities.stair_down {
            let x = cam_x + dx;
            let y = cam_y + dy;
            if x >= -32 && x <= self.view.view_w && y >= -12 && y <= 20 + self.view.view_h {
                g.draw_image(&self.images.stair_icons.1, x, y - 30 + bob, 0);
            }
        }
    }

    /// m_033（a.java:5829-5903）：玩家。facing 0/1/2/3 = 下/上/右/左行；
    /// 行 y 偏移 0/46/92（case 3 为镜像——drawEdgePatch transform 1）；
    /// walkPhase==2 残影（f_int_arr_04/05）战斗批次。
    pub fn paint_player(&self, g: &mut SoftGraphics<'_>, cam_x: i32, cam_y: i32) {
        let px = cam_x + self.view.player_px;
        let mut py = cam_y + self.view.player_py;
        if self.player_bob_applied {
            py -= self.bob_offset;
        }
        g.draw_image(&self.images.shadow, px + 8, py + 22, 0);
        let frame_x = PLAYER_FRAME_TABLE[self.player_frame as usize] * 41;
        match self.facing {
            0 => draw_image_clipped(g, &self.images.actor, px - 3, py - 18, frame_x, 0, 41, 46),
            1 => draw_image_clipped(g, &self.images.actor, px - 8, py - 14, frame_x, 46, 41, 46),
            2 => draw_image_clipped(g, &self.images.actor, px - 8, py - 16, frame_x, 92, 41, 46),
            3 => crate::menu_family::draw_edge_patch(g, &self.images.actor, px, py - 16, frame_x, 92, 41, 46, 1),
            _ => {}
        }
        g.set_clip(0, 0, 240, 320);
        // f_bool_26 路径残影 + walkPhase 2/3 特效：行走批次（spec 待办）
    }

    /// m_039（a.java:6111-6140）：HUD 大框（ui[8][0] 边框件 64x16）。
    fn paint_hud_frame(&mut self, g: &mut SoftGraphics<'_>, x: i32, y: i32, w: i32, h: i32) {
        let img = &self.images.ui[0];
        g.set_clip(x, y, 26, 16);
        g.draw_image(img, x, y, 0);
        let mut cx = x + 26;
        while cx < x + w - 26 {
            g.set_clip(cx, y, 16, 16);
            g.draw_image(img, cx - 26, y, 0);
            cx += 16;
        }
        g.set_clip(0, 0, 240, 320);
        crate::menu_family::draw_edge_patch(g, img, x + w - 26, y, 0, 0, 26, 16, 1);
        g.set_color(2699825);
        g.fill_rect(x + 11, y + 16, w - 22, h - 16);
        let right = x + w - 11;
        let mut cy = y + 16;
        while cy < y + h {
            g.set_clip(x, cy, 11, 16);
            g.draw_image(img, x - 42, cy, 0);
            g.set_clip(right, cy, 11, 16);
            g.draw_image(img, right - 53, cy, 0);
            cy += 16;
        }
        g.set_clip(0, 0, 240, 320);
    }

    /// m_037（a.java:6029-6089）：HUD（头像/HP/攻/防/武器/甲）。
    pub fn paint_hud(&mut self, g: &mut SoftGraphics<'_>) {
        let font = crate::paint::paint_font();
        let base_y = self.view_bottom;
        self.paint_hud_frame(g, 0, base_y, 240, 50);
        let face = &self.images.actor_face;
        g.draw_image(face, 40 - (face.width >> 1), base_y + 50 - face.height, 0);
        let mut y = base_y + 13;
        let icon_x = 83;
        g.set_clip(icon_x, y, 10, 10);
        g.draw_image(&self.images.ui[7], icon_x, y - 2, 0);
        g.set_clip(0, 0, 240, 320);
        paint_mini_frame(g, icon_x + 16, y + 1, 58, 11);
        paint_number(g, &self.images.ui[2], self.hp, icon_x + 16 + 52, y + 2);
        y += 12;
        draw_image_clipped(g, &self.images.ui[7], icon_x, y, 10, 0, 10, 13);
        g.set_color(512);
        paint_mini_frame(g, icon_x + 16, y + 1, 58, 11);
        paint_number(g, &self.images.ui[2], self.atk, icon_x + 16 + 52, y + 2);
        y += 12;
        draw_image_clipped(g, &self.images.ui[7], icon_x, y, 20, 0, 10, 13);
        g.set_color(512);
        paint_mini_frame(g, icon_x + 16, y + 1, 58, 11);
        paint_number(g, &self.images.ui[2], self.def, icon_x + 16 + 52, y + 2);
        // 武器/甲框（f_String_arr_04 = 无/木/铁/银/金/布/皮/锁/金，a.java:484）
        // 槽 x：var11 = 83+16(=99) 后 +60 → 159；甲槽 +34 → 193（a.java:6057/6074）
        let slot_y = base_y + 13 + 2;
        let mut slot_x = icon_x + 16 + 60;
        paint_mini_frame(g, slot_x, slot_y, 32, 32);
        let none: Vec<u16> = "无".encode_utf16().collect();
        let cw = font.char_width(none[0]);
        g.set_color(0xFFFFFF);
        g.draw_string(&none, slot_x + ((32 - cw) >> 1), slot_y + ((32 - font.height) >> 1), 0);
        slot_x += 34;
        paint_mini_frame(g, slot_x, slot_y, 32, 32);
        g.set_color(0xFFFFFF);
        g.draw_string(&none, slot_x + ((32 - cw) >> 1), slot_y + ((32 - font.height) >> 1), 0);
    }

    /// m_035（a.java:5995-6018）：顶部状态栏（层数/三钥/金币）+ m_036 钥匙格。
    pub fn paint_status_bar(&mut self, g: &mut SoftGraphics<'_>) {
        let base_y = 0;
        g.set_clip(0, 0, 240, 320);
        g.draw_image(&self.images.ui[3], 0, base_y, 0);
        // currentFloor ≤ 50 分支（>50 的"引子"文字属尾声楼层）
        paint_number(g, &self.images.ui[18], self.floor, 40, base_y + 4 + 2);
        g.draw_image(&self.images.ui[8], 41, base_y + 4 + 10, 0);
        g.set_color(2435368);
        g.fill_rect(65, base_y, 175, 18);
        self.paint_key_slot(g, 0, self.keys.0, 65, base_y);
        self.paint_key_slot(g, 1, self.keys.1, 107, base_y);
        self.paint_key_slot(g, 2, self.keys.2, 149, base_y);
        let y = base_y + 3;
        g.draw_image(&self.images.ui[1], 191, y, 0);
        paint_number(g, &self.images.ui[2], self.gold, 237, y + 2);
    }

    /// m_036（a.java:6020-6027）：钥匙格（ui[8][6] 18px 三联图 + 数量）。
    fn paint_key_slot(&mut self, g: &mut SoftGraphics<'_>, kind: i32, count: i32, x: i32, y: i32) {
        g.set_clip(x, y, 18, 16);
        g.draw_image(&self.images.ui[6], x - kind * 18, y, 0);
        g.set_clip(0, 0, 240, 320);
        paint_number(g, &self.images.ui[2], count, x + 40, y + 5);
    }
}

/// m_003（a.java:4547-4552）：5x5 定尺寸裁剪贴图（小地图标记）。
fn m_003_clip5(g: &mut SoftGraphics<'_>, img: &ArgbImage, x: i32, y: i32, src_x: i32) {
    g.set_clip(x, y, 5, 5);
    g.draw_image(img, x - src_x, y, 0);
    g.set_clip(0, 0, 240, 320);
}
