//! mode 3 游戏画面场景（a.java paint case 3 :2385-2422 + run case 3 :3545-3679）。
//!
//! 绘制链（paint 顺序，T544 实证 ops 同序）：
//! `drawParallaxBackdrop(true)` → `paintTileLayer(0, view_top)` → `paintEntityLayer`
//! 实体层（含玩家 paintPlayerSprite 插入 + 楼梯浮标）→ 小地图 → `paintHudPanel`
//! HUD → `paintStatusBar` 状态栏 → 楼梯指示（f_bool_13）→ `drawPopupLayer` →
//! `paintSoftkeyBar`。
//!
//! 逻辑链（每 tick）：keyValue 开关 → walkPhase 状态机 → `advanceEntityFrames`
//! 帧推进。bob（advanceBobPhase）在 **paint** 内推进（paintEntityLayer 首行调用，
//! 一拍一次）。
//!
//! **布局/颜色常量一律取自 [`crate::layout`]（视口扩展唯一修改点）**；本模块
//! 只保留与实体/帧数据耦合的内联算式（Java 公式原样）。
//!
//! P3.5 对拍窗口：gameplay-floor1 T537-599 稳态（无输入）——popup 环形队列、
//! walkPhase 1 步进、lockCameraOn 交互为后续批次（spec p3-render §3.6 待办）。

use crate::entity::{self, CellGrid, EntityTable, ANIM_OFFSET_TABLE};
use crate::game_view::GameView;
use crate::layout::*;
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
    /// f_Image_arr2_00[1][0]：视差背景瓦（77×SCREEN_H）
    pub backdrop: ArgbImage,
    /// f_Image_arr2_00[5][0]：通用阴影
    pub shadow: ArgbImage,
    /// f_Image_arr2_00[3][0]：玩家三向精灵条带（3 行 PLAYER_FRAME_W×PLAYER_FRAME_H）
    pub actor: ArgbImage,
    /// f_Image_arr2_00[3][1]：HUD 头像
    pub actor_face: ArgbImage,
    /// f_Image_arr2_00[8][*]：ui 容器（25 张）
    pub ui: Vec<ArgbImage>,
    /// f_Image_arr2_00[2][1]/[2][2]：上/下楼浮标
    pub stair_icons: (ArgbImage, ArgbImage),
    /// f_Image_arr2_00[2][9]：开格动画贴图（OPEN_ANIM_W×OPEN_ANIM_H）
    pub open_anim: ArgbImage,
    /// f_Image_arr2_00[4][12]：已开门贴图
    pub opened_door: ArgbImage,
    /// entityTypeImage[type]：实体类型→精灵条带（null → None；m_122 映射）
    pub entity: Vec<Option<ArgbImage>>,
    /// popup 层图集（map[2][3/4/5] 数字带 + map[2][6] 图标条 + ui[8][9] 战斗数字）
    pub popup: crate::popup::PopupImages,
    /// f_Image_arr2_00[3][2]：战斗星光表（walkPhase==5 的 4 帧星芒）
    pub spark_sheet: ArgbImage,
    /// ui[8][23]：walkPhase 2 四角装饰（22×24 帧 ×3）
    pub corner_sprites: ArgbImage,
    /// map[2][8]：战后数字条目标图标
    pub battle_icon: ArgbImage,
    /// map[2][7]：战后数字条（drawDigitStrip 7px 数字带）
    pub battle_strip: ArgbImage,
}

impl SceneImages {
    /// m_122 头部的类型→容器映射（a.java:9438-9466）：
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

/// 玩家帧表 playerAnimTables[0]（a.java:477：{0,1,0,2}）。
const PLAYER_FRAME_TABLE: [i32; 4] = [0, 1, 0, 2];

/// 战斗星光帧号表（Java sparkFrames，a.java:766-773：8 拍循环，0 = 不画）。
const SPARK_FRAME: [i32; 8] = [1, 2, 0, 3, 4, 1, 2, 0];
/// 星光四联组 (sx,sy,w,h)×4（Java sparkQuads，a.java:748-764）。
const SPARK_QUADS: [i32; 16] = [29, 31, 39, 15, 0, 31, 29, 10, 0, 0, 37, 31, 36, 0, 32, 29];

/// blocksPlayerInsert（initEntityTables 从构造字面量 f_bool_arr_03 拷贝，a.java:679/
/// 6237）：true 的类型（0/5/6/7/8/10）占据插入窗口时**不**在其前插画玩家
/// （paintEntityLayer 插入条件的 `!blocksPlayerInsert[type]` 项，a.java:6510）。
const BLOCKS_PLAYER_INSERT: [bool; 13] = {
    let mut t = [false; 13];
    let true_at = [0usize, 5, 6, 7, 8, 10];
    let mut i = 0;
    while i < true_at.len() {
        t[true_at[i]] = true;
        i += 1;
    }
    t
};

/// 单层资产（换层时由场景内自带——game-core 无 I/O，字节由宿主预置）。
#[derive(Clone)]
pub struct FloorAssets {
    pub maplv: Vec<u8>,
    /// (type, x, y, param, visible)（sprite{n} 解析 + m_122 可见性）
    pub records: Vec<(u8, u16, u16, i16, bool)>,
}

/// mode 3 场景状态。
pub struct GameScene {
    pub view: GameView,
    pub entities: EntityTable,
    pub grid: CellGrid,
    pub images: SceneImages,
    /// walkableGrid：可行走网格（rebuildWalkability，buildMinimap 小地图用）
    pub walkable: Vec<Vec<bool>>,
    /// minimapImage：小地图（buildMinimap 生成；None = optionChecked[1]=false）
    pub minimap: Option<ArgbImage>,
    /// optionChecked[1]：小地图开关（RMS 读档失败 catch 默认 true，a.java:4465-4467）
    pub minimap_enabled: bool,
    /// frameCounter（run 循环变量，TICK n 拍 = n-1）
    pub frame_counter: i64,
    /// advanceBobPhase 浮沉相位（paint 内推进）
    pub bob_offset: i32,
    pub bob_rising: bool,
    /// drawParallaxBackdrop 的滚动相位（每 paint -1，PARALLAX_WRAP 回绕）
    pub backdrop_scroll: i32,
    /// f_int_48：视口底（HUD 顶）——初始化后与 layout::view_bottom_y() 一致
    pub view_bottom: i32,
    /// 玩家态（paintPlayerSprite 输入）
    pub facing: i32,
    pub walk_phase: i32,
    pub player_frame: i32,
    pub player_bob_applied: bool, // f_bool_07
    /// HUD 数值源（paintHudPanel/paintStatusBar 读；战斗/拾取写）
    pub floor: i32,
    /// 玩家战斗面（HP/攻/防/三钥/金/物品栈——tickBattle 与拾取的读写集）
    pub player: crate::combat::PlayerCombat,
    /// 敌方缩放表（scaleEnemyStats：difficultyIndex=0 → ×1）
    pub enemies: crate::combat::ScaledEnemies,
    /// 敌方基础表（enemyBaseHp/enemyBaseGold，a.java:1044/1121；夹具）
    pub enemy_base_hp: Vec<i32>,
    pub enemy_base_gold: Vec<i32>,
    /// 软键（m_000 case 3：1/3）
    pub softkeys: (i8, i8),

    // —— 输入（keyPressed/keyReleased a.java:4653-4671）——
    /// keyValue：粘性"最近按键"（keyReleased 不清）
    pub key_value: i32,
    /// keyHeldCode：按住电平（release 清 0；移动由它驱动）
    pub key_held: i32,

    // —— 步进（run case 3 walkPhase 1）——
    pub player_cell_x: i32,
    pub player_cell_y: i32,
    /// stepProgressPx：步内像素累计（32 = 一步）
    pub step_progress: i32,
    /// walkStepCount（f_bool_26 路径步数，行走扩展批次）
    pub walk_step_count: i32,

    // —— 场上战斗（walkPhase 5；tickBattle a.java:10013-10145）——
    /// battleTargetEntity（-1 = 无）
    pub battle_target: i32,
    /// battleEnemyHp：当前敌人剩余 HP
    pub battle_enemy_hp: i32,
    /// battleTick：战斗节拍（&3==0 交换；%3 选伤害 popup 色）
    pub battle_tick: i32,
    /// battleAnimTick：胜利死亡动画计数
    pub battle_anim_tick: i32,
    /// battleFinishing：胜利动画阶段
    pub battle_finishing: bool,
    /// battleSceneRerun：胜利后重入场景（boss 剧情用；floor1 恒 false）
    pub battle_rerun_scene: bool,
    /// battlePredictGate：预测含反击标记（cat8 置 true）
    pub battle_no_counter: bool,
    /// battleActiveFlag：战斗中标记（语义未完全考证）
    pub battle_active: bool,

    // —— popup 环形队列 ——
    pub popups: crate::popup::PopupRing,

    // —— 换层（changeFloor a.java:7116-7160 + paint 遮幅 a.java:3171-3215）——
    /// f_bool_16：遮幅进行中
    pub transitioning: bool,
    /// f_bool_17：闭合相位（构造 true；全黑后翻 false 开启）
    pub wipe_closing: bool,
    /// f_int_142：遮幅计数 0..=4
    pub wipe: i32,
    /// f_byte_23：目标层
    pub pending_floor: i32,
    /// f_bool_18：下楼标记（型7 上楼传 false）
    pub went_down: bool,
    /// minFloorReached/maxFloorReached
    pub min_floor: i32,
    pub max_floor: i32,
    /// f_int_127：脚本路线实体（case 32 型 72 的"追踪中"分支；默认 0）
    pub route_entity: i32,
    /// f_byte_19：型 72 形态号（脚本写；默认 0）
    pub form_19: i32,
    pub current_floor: i32,
    /// 层资产表（按层号索引）
    pub floor_assets: Vec<Option<FloorAssets>>,
    /// 宽度表/可行走表（换层重载用）
    pub width_table: Vec<i32>,
    pub tile_walk: Vec<i32>,
    /// gameRandom（java.util.Random，seed=0 起的全局单例——粒子/战斗抖动共用
    /// 消费链；装载历史由测试侧预推进，paint 内 state==3 抖动消费）
    pub rng: game_platform::JavaRandom,

    // —— walkPhase 2 交互视点（-5 键；easeAfterimages/lockCameraOn 家族）——
    /// afterimageX/Y[4]：残影蛇（-5 时填充玩家位，easeAfterimages 逐拍缓动）
    pub afterimage_x: [i32; 4],
    pub afterimage_y: [i32; 4],
    /// cameraLockedOnPlayer：视点锁定（第二次 -5 置位 → stepCamera 回玩家）
    pub view_locked: bool,
    /// showBattleDigits：战后数字条开关（findItemStackIndex(13) 有 13 号道具）
    pub show_battle_digits: bool,
    /// battleDigitByType：-5 时对各怪的预测伤害数字
    pub battle_digit_by_type: Vec<i32>,
    /// lockCameraOn 的缓动相机坐标（注意与 cameraPixelX/Y 是**反演关系**）
    pub ease_cam_x: i32,
    pub ease_cam_y: i32,
    pub ease_target_x: i32,
    pub ease_target_y: i32,
}

impl GameScene {
    /// loadFloorData（a.java:9394-9434）：maplv 装载 + m_059 格索引重建 +
    /// 可行走重建 + m_122 实体生成 + sortEntitiesByY 排序。
    /// `floor_assets`：按层号索引的换层资产（索引缺省 = 该层不可达）。
    #[allow(clippy::too_many_arguments)]
    pub fn load_floor(
        maplv: &[u8],
        player_px: i32,
        player_py: i32,
        records: &[(u8, u16, u16, i16, bool)],
        width_table: &[i32],
        tile_walkability: &[i32],
        images: SceneImages,
        floor_assets: Vec<Option<FloorAssets>>,
    ) -> Result<GameScene, String> {
        let mut scene = GameScene {
            view: GameView::from_maplv(maplv, player_px, player_py)?,
            entities: EntityTable::new(100),
            grid: CellGrid::new(1, 1),
            images,
            walkable: Vec::new(),
            minimap: None,
            minimap_enabled: true,
            frame_counter: 0,
            bob_offset: 0,
            bob_rising: false,
            backdrop_scroll: 0,
            view_bottom: view_bottom_y(),
            facing: 0,
            walk_phase: 0,
            player_frame: 0,
            player_bob_applied: false,
            floor: 1,
            player: crate::combat::PlayerCombat {
                hp: 0,
                atk: 0,
                def: 0,
                yellow_keys: 0,
                blue_keys: 0,
                red_keys: 0,
                gold: 0,
                item_types: vec![0; 16],
                item_uses: vec![0; 16],
                item_stack_size: 0,
            },
            enemies: crate::combat::ScaledEnemies::default(),
            enemy_base_hp: Vec::new(),
            enemy_base_gold: Vec::new(),
            softkeys: (1, 3),
            key_value: 0,
            key_held: 0,
            player_cell_x: player_px >> 5,
            player_cell_y: player_py >> 5,
            step_progress: 0,
            walk_step_count: 0,
            battle_target: -1,
            battle_enemy_hp: 0,
            battle_tick: 0,
            battle_anim_tick: 0,
            battle_finishing: false,
            battle_rerun_scene: false,
            battle_no_counter: false,
            battle_active: false,
            popups: crate::popup::PopupRing::default(),
            rng: game_platform::JavaRandom::new_seeded(0),
            transitioning: false,
            wipe_closing: true,
            wipe: 0,
            pending_floor: 0,
            went_down: false,
            min_floor: 1,
            max_floor: 1,
            route_entity: 0,
            form_19: 0,
            current_floor: 1,
            floor_assets,
            width_table: width_table.to_vec(),
            tile_walk: tile_walkability.to_vec(),
            afterimage_x: [0; 4],
            afterimage_y: [0; 4],
            view_locked: false,
            show_battle_digits: false,
            battle_digit_by_type: vec![0; 89],
            ease_cam_x: 0,
            ease_cam_y: 0,
            ease_target_x: 0,
            ease_target_y: 0,
        };
        scene.load_floor_data(maplv, player_px, player_py, records)?;
        Ok(scene)
    }

    /// loadFloorData 的原地重载（换层遮幅中段 a.java:3183 调用同方法）：
    /// 重建 view/entities/grid/walkable，其余场景状态（玩家/HUD/RNG/popup）保留。
    pub fn load_floor_data(
        &mut self,
        maplv: &[u8],
        player_px: i32,
        player_py: i32,
        records: &[(u8, u16, u16, i16, bool)],
    ) -> Result<(), String> {
        let view = GameView::from_maplv(maplv, player_px, player_py)?;
        let cols = view.wide as usize;
        let rows = view.high as usize;
        let mut walkable = vec![vec![false; cols]; rows];
        for y in 0..rows {
            for x in 0..cols {
                // rebuildWalkability（a.java:6954-6972）的索引：**行距 wide·4、
                // 列距 2**（var1 行尾 +=wide<<1 叠加列循环的 ×2）——与 paintTileLayer
                // 的行距 wide·2、列距 1 是同一数组的两套读法（原版共存）。
                // A 级：oracle FLD 134（walkableGrid）13×13 逐格复算匹配。
                let idx = (y * cols * 4 + x * 2) as i32;
                let tile = view.terrain_at(0, idx) as usize;
                walkable[y][x] = tile < self.tile_walk.len() && self.tile_walk[tile] == 1;
            }
        }
        let mut entities = EntityTable::new(100);
        let mut grid = CellGrid::new(view.wide, view.high);
        for &(type_code, x, y, param, visible) in records {
            let img_h = self.images.entity[type_code as usize].as_ref().map(|i| i.height);
            let idx = entity::spawn(
                &mut entities,
                &mut grid,
                type_code as i32,
                x as i32,
                y as i32,
                param,
                &self.width_table,
                img_h,
            );
            if !visible {
                entities.visible[idx] = false; // m_122 特例分支（5/81/6/12/76/82/83）
            }
        }
        entity::sort_by_y(&mut entities, &mut grid);
        self.view = view;
        self.entities = entities;
        self.grid = grid;
        self.walkable = walkable;
        self.minimap = None;
        Ok(())
    }

    /// changeFloor（a.java:7116-7160）：层号合法性（strict=键 49/55 路径；
    /// 楼梯路径 strict=false 跳过已达层检查）→ 置遮幅态 + 记录目标层。
    /// 越界提示浮层（miscTexts 0-3）属交互批次——todo 守卫。
    pub fn change_floor(&mut self, target: i32, down: bool, strict: bool) -> bool {
        if strict {
            if target < self.min_floor || target > self.max_floor {
                todo!("changeFloor 已达层提示浮层（miscTexts[0/1]，交互批次）")
            }
        }
        if target < 0 || target > 55 {
            // f_int_68=55（a.java:808）；miscTexts[2/3]
            todo!("changeFloor 层界提示浮层（miscTexts[2/3]，交互批次）")
        }
        self.transitioning = true;
        if target < self.min_floor {
            self.min_floor = target;
        } else if target > self.max_floor {
            self.max_floor = target;
            // f_int_154 全局进度（RMS）不追踪
        }
        // m_119(currentFloor)（楼层状态序列化 → RMS，Rust 不追踪）
        self.pending_floor = target;
        self.went_down = down;
        true
    }

    /// 遮幅中段换层（a.java:3180-3206）：loadFloorData + 落点特例
    /// （0/50/1-上行 m_024；其余 findFloorGateEntity → m_031）+ 小地图 + 步效果。
    fn swap_floor(&mut self) {
        self.floor = self.pending_floor;
        self.current_floor = self.pending_floor;
        let fa = self.floor_assets[self.pending_floor as usize]
            .clone()
            .expect("层资产未提供");
        let px = self.view.player_px;
        let py = self.view.player_py;
        self.load_floor_data(&fa.maplv, px, py, &fa.records)
            .expect("层资产重载失败");
        if self.floor == 0 {
            self.m_024(1, 2);
            self.view.center_on_player();
        } else if self.floor == 50 {
            self.m_024(6, 7);
            self.view.center_on_player();
        } else if self.floor == 1 && !self.went_down {
            self.m_024(6, 11);
            self.view.center_on_player();
        } else if let Some((gx, gy)) = self.find_floor_gate(self.went_down) {
            self.m_031(gx, gy);
        }
        self.build_minimap();
        self.apply_step_cell_effects();
    }

    /// findFloorGateEntity（a.java:7096-7114）：up=true 找型 7、false 找型 8；
/// 换层落点直传 f_bool_18（false=上楼抵达→机型 8 下楼梯），
    /// param>>9==0（低 9 位无高字节——目标层 ≤255 的常规门）。
    fn find_floor_gate(&self, up: bool) -> Option<(i32, i32)> {
        let want = if up { 7 } else { 8 };
        for e in 0..self.entities.count {
            if self.entities.entity_type[e] == want && (self.entities.param[e] >> 9) == 0 {
                return Some((self.entities.pixel_x[e] >> 5, self.entities.pixel_y[e] >> 5));
            }
        }
        None
    }

    /// m_031（a.java:5797-5810）：楼梯落点 = 首个可行走邻格（上/下/左/右序）。
    fn m_031(&mut self, mut gx: i32, mut gy: i32) {
        if self.walkable_at(gx, gy - 1) {
            gy -= 1;
        } else if self.walkable_at(gx, gy + 1) {
            gy += 1;
        } else if self.walkable_at(gx - 1, gy) {
            gx -= 1;
        } else if self.walkable_at(gx + 1, gy) {
            gx += 1;
        }
        self.m_024(gx, gy);
        self.view.center_on_player();
    }

    /// m_024（a.java:5183-5189）：格坐标直置 + 步进度清零。
    fn m_024(&mut self, cx: i32, cy: i32) {
        self.player_cell_x = cx;
        self.player_cell_y = cy;
        self.view.player_px = cx << 5;
        self.view.player_py = cy << 5;
        self.step_progress = 0;
    }

    fn walkable_at(&self, x: i32, y: i32) -> bool {
        x >= 0
            && y >= 0
            && x < self.view.wide
            && y < self.view.high
            && self.walkable[y as usize][x as usize]
    }

    /// 加载步 11（a.java:3504-3514）：进游戏视图 + 居中相机。
    pub fn enter_view(&mut self) {
        self.view.center_on_player();
    }

    /// buildMinimap（a.java:6869-6937）：小地图生成
    /// （wide×MINIMAP_PX_PER_CELL 见方，dim MINIMAP_DIM_ALPHA）。
    pub fn build_minimap(&mut self) {
        self.minimap = None;
        if !self.minimap_enabled {
            return;
        }
        let w = self.view.wide * MINIMAP_PX_PER_CELL;
        let h = self.view.high * MINIMAP_PX_PER_CELL;
        let mut img = ArgbImage::create(w, h);
        {
            let mut g = SoftGraphics::new(&mut img);
            g.set_color(MINIMAP_BG);
            g.fill_rect(0, 0, w, h);
            for y in 0..self.view.high as usize {
                for x in 0..self.view.wide as usize {
                    let px = (x as i32) * MINIMAP_PX_PER_CELL;
                    let py = (y as i32) * MINIMAP_PX_PER_CELL;
                    if self.walkable[y][x] {
                        g.set_color(MINIMAP_WALKABLE);
                        g.fill_rect(px, py, MINIMAP_PX_PER_CELL, MINIMAP_PX_PER_CELL);
                    }
                    let ct = self.grid.cell_type[y][x] as usize;
                    if ct > 0 && self.grid.capacity[ct] > 0 {
                        let first = self.grid.slots[ct][0] as usize - 1;
                        if !self.entities.removed[first] && self.entities.solid[first] != 1 {
                            match self.entities.entity_type[first] {
                                1 => {
                                    g.set_color(MINIMAP_DOOR);
                                    g.fill_rect(px, py, 3, 3);
                                    g.set_color(MINIMAP_DOOR_EDGE);
                                    g.draw_line(px + 3, py, px + 3, py + 3);
                                    g.draw_line(px, py + 3, px + 3, py + 3);
                                }
                                // case 2/3 各自显式 break（a.java:6900-6913，无落穿）
                                2 => {
                                    g.set_color(MINIMAP_DOOR2);
                                    g.fill_rect(px, py, 4, 4);
                                    g.set_color(MINIMAP_DOOR2_EDGE);
                                    g.draw_line(px + 3, py, px + 3, py + 3);
                                    g.draw_line(px, py + 3, px + 3, py + 3);
                                }
                                3 => {
                                    g.set_color(MINIMAP_DOOR3);
                                    g.fill_rect(px, py, 3, 3);
                                    g.set_color(MINIMAP_DOOR3_EDGE);
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
        self.minimap = Some(crate::intro::dim_image(&img, MINIMAP_DIM_ALPHA));
    }

    /// keyPressed（a.java:4653-4666）：keyValue = keyHeldCode = code。
    pub fn press_key(&mut self, code: i32) {
        self.key_value = code;
        self.key_held = code;
    }

    /// keyReleased（a.java:4668-4670）：只清 keyHeldCode。
    pub fn release_key(&mut self) {
        self.key_held = 0;
    }

    /// run case 3（a.java:3545-3679）：keyValue 开关（-7/-6 暂停/道具——
    /// 本窗口无）→ walkPhase 状态机 → advanceEntityFrames。
    /// `frame_counter` 为本拍值（TICK n → n-1）。
    pub fn tick(&mut self, frame_counter: i64, width_table: &[i32]) {
        self.frame_counter = frame_counter;
        match self.walk_phase {
            0 => self.handle_field_input(),
            1 => {
                // 步进（a.java:3570-3601）：±8px + 相机边缘跟随 + m_025 帧
                match self.facing {
                    0 => {
                        self.view.player_py += 8;
                        if self.view.player_py + self.view.cam_y + 16 > self.view.view_h - 106 {
                            let cy = self.view.cam_y - 8;
                            self.view.set_camera(self.view.cam_x, cy);
                        }
                    }
                    1 => {
                        self.view.player_py -= 8;
                        if self.view.player_py + self.view.cam_y + 16 < 106 {
                            let cy = self.view.cam_y + 8;
                            self.view.set_camera(self.view.cam_x, cy);
                        }
                    }
                    2 => {
                        self.view.player_px += 8;
                        if self.view.player_px + self.view.cam_x + 16 > self.view.view_w - 106 {
                            let cx = self.view.cam_x - 8;
                            self.view.set_camera(cx, self.view.cam_y);
                        }
                    }
                    3 => {
                        self.view.player_px -= 8;
                        if self.view.player_px + self.view.cam_x + 16 < 106 {
                            let cx = self.view.cam_x + 8;
                            self.view.set_camera(cx, self.view.cam_y);
                        }
                    }
                    _ => {}
                }
                self.step_progress += 8;
                // m_025（a.java:5191-5197）：行走帧循环
                if self.player_frame < PLAYER_FRAME_TABLE.len() as i32 - 1 {
                    self.player_frame += 1;
                } else {
                    self.player_frame = 0;
                }
                if self.step_progress >= 32 {
                    self.step_progress = 0;
                    self.player_frame = 0;
                    self.apply_step_cell_effects();
                }
            }
            2 => {
                // walkPhase 2（a.java:3604-3642）：easeAfterimages 残影蛇缓动 +
                // 未锁定时 keyHeld 相机平移 / keyValue -5 → 锁定 + lockCameraOn；
                // 已锁定时 stepCameraTowardTarget 回到玩家
                self.easeAfterimages();
                if !self.view_locked {
                    match self.key_held {
                        -4 | 54 => {
                            self.facing = 2;
                            let cx = self.view.cam_x - 16;
                            self.view.set_camera(cx, self.view.cam_y);
                        }
                        -3 | 52 => {
                            self.facing = 3;
                            let cx = self.view.cam_x + 16;
                            self.view.set_camera(cx, self.view.cam_y);
                        }
                        -2 | 56 => {
                            self.facing = 0;
                            let cy = self.view.cam_y - 16;
                            self.view.set_camera(self.view.cam_x, cy);
                        }
                        -1 | 50 => {
                            self.facing = 1;
                            let cy = self.view.cam_y + 16;
                            self.view.set_camera(self.view.cam_x, cy);
                        }
                        _ => {}
                    }
                    if self.key_value == -5 || self.key_value == 53 {
                        self.view_locked = true;
                        self.lockCameraOn(0);
                    }
                } else {
                    self.step_camera_toward_target();
                }
            }
            5 => self.tick_battle(true),
            _ => {} // 3 道具菜单 / 4 楼层切换：本窗口无
        }
        entity::advance_frames(&mut self.entities, &mut self.grid, frame_counter, width_table);
        // run 主循环尾部（a.java:4266，else 分支）：**keyValue 每拍清零**
        // （边沿触发；keyHeldCode 由 keyReleased 清——a.java:4668）
        self.key_value = 0;
    }

    /// easeAfterimages（a.java:5199-5252）：残影蛇缓动。目标 = 未锁定时的
    /// 视口中心偏移（bob 耦合）或锁定后的玩家位；[3] 到位且锁定 → walkPhase=0。
    fn easeAfterimages(&mut self) {
        let (mut tx, mut ty) = if !self.view_locked {
            (
                (self.view.view_w - 41 >> 1) - self.view.cam_x,
                (self.view.view_h >> 1) - self.view.cam_y + self.bob_offset,
            )
        } else {
            (self.view.player_px, self.view.player_py)
        };
        if self.afterimage_x[3] == tx && self.afterimage_y[3] == ty {
            if self.view_locked {
                self.walk_phase = 0;
            }
        } else {
            self.afterimage_x[0] = tx;
            self.afterimage_y[0] = ty;
            for i in 1..4 {
                let mut v3 = 4 - i as i32; // Java var4==0 的 +5 分支不可达（循环从 1 起）
                if i == 0 {
                    v3 += 5;
                }
                let ax = self.afterimage_x[i];
                if ax < tx {
                    self.afterimage_x[i] = ax + ((tx - ax) >> 1) + v3;
                    if self.afterimage_x[i] > tx {
                        self.afterimage_x[i] = tx;
                    }
                } else if ax > tx {
                    self.afterimage_x[i] = ax + (((tx - ax) >> 1) - v3);
                    if self.afterimage_x[i] < tx {
                        self.afterimage_x[i] = tx;
                    }
                }
                let ay = self.afterimage_y[i];
                if ay < ty {
                    self.afterimage_y[i] = ay + ((ty - ay) >> 1) + v3;
                    if self.afterimage_y[i] > ty {
                        self.afterimage_y[i] = ty;
                    }
                } else if ay > ty {
                    self.afterimage_y[i] = ay + (((ty - ay) >> 1) - v3);
                    if self.afterimage_y[i] < ty {
                        self.afterimage_y[i] = ty;
                    }
                }
                tx = self.afterimage_x[i];
                ty = self.afterimage_y[i];
            }
        }
    }

    /// lockCameraOn（a.java:8656-8692）：缓动相机初始化。0 → 目标 = 玩家位。
    fn lockCameraOn(&mut self, target: i32) {
        self.ease_cam_x = (self.view.view_w - 32 >> 1) - self.view.cam_x;
        self.ease_cam_y = (self.view.view_h - 32 >> 1) - self.view.cam_y;
        if target != 0 && target != 87 {
            // 目标类型查实体（倒序首个）——战后视点本窗口只用 0（玩家）
            let mut found = -1;
            for e in (0..self.entities.count).rev() {
                if self.entities.entity_type[e] == target {
                    found = e as i32;
                    break;
                }
            }
            if found >= 0 {
                self.ease_target_x = self.entities.pixel_x[found as usize];
                self.ease_target_y = self.entities.pixel_y[found as usize];
            }
        } else {
            self.ease_target_x = self.view.player_px;
            self.ease_target_y = self.view.player_py;
        }
    }

    /// stepCameraTowardTarget（a.java:8694-8723）：1/4 距离 +2 缓动，
    /// 反演回 cameraPixel（(中心) - ease 坐标）。
    fn step_camera_toward_target(&mut self) {
        if self.ease_cam_x < self.ease_target_x {
            self.ease_cam_x += (self.ease_target_x - self.ease_cam_x >> 2) + 2;
            if self.ease_cam_x > self.ease_target_x {
                self.ease_cam_x = self.ease_target_x;
            }
        } else if self.ease_cam_x > self.ease_target_x {
            self.ease_cam_x += ((self.ease_target_x - self.ease_cam_x >> 2) - 2);
            if self.ease_cam_x < self.ease_target_x {
                self.ease_cam_x = self.ease_target_x;
            }
        }
        if self.ease_cam_y < self.ease_target_y {
            self.ease_cam_y += (self.ease_target_y - self.ease_cam_y >> 2) + 2;
            if self.ease_cam_y > self.ease_target_y {
                self.ease_cam_y = self.ease_target_y;
            }
        } else if self.ease_cam_y > self.ease_target_y {
            self.ease_cam_y += ((self.ease_target_y - self.ease_cam_y >> 2) - 2);
            if self.ease_cam_y < self.ease_target_y {
                self.ease_cam_y = self.ease_target_y;
            }
        }
        self.view.set_camera(
            (self.view.view_w - 32 >> 1) - self.ease_cam_x,
            (self.view.view_h - 32 >> 1) - self.ease_cam_y,
        );
    }

    /// handleFieldInput（a.java:5255-5323）：keyValue 特例（-5 战后视点/
    /// 49/55 楼梯换乘——本窗口无）+ keyHeld 移动（Nokia 映射：
    /// -1=上 -2=下 -3=左 -4=右）。
    fn handle_field_input(&mut self) {
        if self.key_value == -5 || self.key_value == 53 {
            // -5（a.java:5265-5286）：残影位填充玩家位 + 各怪预测伤害数字 +
            // walkPhase=2 + showBattleDigits（有 13 号道具才画数字条）
            for i in 0..4 {
                self.afterimage_x[i] = self.view.player_px;
                self.afterimage_y[i] = self.view.player_py;
            }
            self.view_locked = false;
            for e in 0..self.entities.count {
                let t = self.entities.entity_type[e];
                if !self.entities.removed[e] && entity::render_category(t) == 8 {
                    self.battle_digit_by_type[t as usize] = self.predict_hp_loss(t, true);
                }
            }
            self.walk_phase = 2;
            self.show_battle_digits = self.player.find_item(13) >= 0;
            return;
        }
        let facing = match self.key_held {
            -4 | 54 => 2,
            -3 | 52 => 3,
            -2 | 56 => 0,
            -1 | 50 => 1,
            _ => return,
        };
        self.facing = facing;
        self.try_step(facing);
    }

    /// tryStep（a.java:5325-5352）：目标格 + interactWithCell 放行判定。
    fn try_step(&mut self, facing: i32) -> bool {
        let mut tx = self.view.player_px >> 5;
        let mut ty = self.view.player_py >> 5;
        match facing {
            0 => ty += 1,
            1 => ty -= 1,
            2 => tx += 1,
            3 => tx -= 1,
            _ => {}
        }
        let ok = self.interact_with_cell(tx, ty);
        if ok {
            self.walk_phase = 1;
        } else if self.walk_phase == 1 {
            self.walk_step_count = 0;
            self.walk_phase = 0;
        }
        ok
    }

    /// interactWithCell（a.java:5353-5527）：格槽**倒序**遍历 + 类目交互 +
    /// 地形终审（var7 楼梯/喷泉计数 >0 时豁免地形）。
    fn interact_with_cell(&mut self, x: i32, y: i32) -> bool {
        let mut allowed = true;
        let mut stair_count = 0;
        let ct = if x >= 0 && y >= 0 && x < self.view.wide && y < self.view.high {
            self.grid.cell_type[y as usize][x as usize] as usize
        } else {
            return false;
        };
        let cap = self.grid.capacity[ct] as usize;
        if cap > 0 {
            let mut s = cap as i32 - 1;
            while s >= 0 {
                let e = self.grid.slots[ct][s as usize] as usize - 1;
                let t = self.entities.entity_type[e];
                if self.entities.solid[e] != 1 {
                    match entity::render_category(t) {
                        1 => match t {
                            1 | 2 | 3 => {
                                // 门（a.java:5381-5407）：耗钥（consumeKeyForDoor
                                // 内部 spawnPopup kind1 图标 a.java:7723-7745）→
                                // markEntityRemoved + 重建小地图；无钥 → 阻挡 +
                                // walkStepCount=0（提示浮层属交互批次）
                                let door = match t {
                                    1 => crate::combat::door_code::YELLOW,
                                    2 => crate::combat::door_code::RED,
                                    _ => crate::combat::door_code::BLUE,
                                };
                                let (consumed, icon) =
                                    crate::combat::consume_key_for_door(&mut self.player, door);
                                if consumed {
                                    entity::remove(&mut self.entities, e);
                                    self.popups.spawn(
                                        1,
                                        icon,
                                        self.view.player_px,
                                        self.view.player_py,
                                        self.view.cam_x,
                                        self.view.cam_y,
                                        self.images.popup.digit_strips[0].width / 11,
                                    );
                                    self.build_minimap();
                                } else {
                                    self.walk_step_count = 0;
                                    allowed = false;
                                }
                            }
                            5 | 81 | 4 => {
                                if t != 4 {
                                    // 封印门/障碍（a.java:5422-5428）：阻挡
                                    allowed = false;
                                }
                            }
                            6 | 7 | 8 => stair_count += 1,
                            _ => {} // 9 炼丹 / 11 / 76/82 / 83 场景门：后续批次
                        },
                        8 => {
                            // 怪物（a.java:5460-5486）：可胜 → 场上战斗；否则阻挡
                            let loss = self.predict_hp_loss(t, true);
                            allowed = false;
                            if loss >= 0 && loss < self.player.hp {
                                self.walk_phase = 5;
                                self.entities.solid[e] = 3;
                                self.battle_target = e as i32;
                                // battleEnemyHp = enemyBaseHp[type-41]（**基础表**非缩放表）
                                self.battle_enemy_hp = self.enemy_base_hp[(t - 41) as usize];
                                self.battle_active = true;
                                self.battle_no_counter = true;
                            } else {
                                self.walk_step_count = 0;
                                // "你无法战胜它" 提示浮层：交互批次
                            }
                        }
                        16 | 32 => allowed = false, // 联动门/多形态：阻挡（a.java:5490/5520）
                        _ => {}                    // 2/4 拾取：放行（步末 applyStepCellEffects）
                    }
                }
                s -= 1;
            }
            if allowed && stair_count <= 0 {
                allowed = self.walkable[y as usize][x as usize];
            }
        } else if !self.walkable[y as usize][x as usize] {
            allowed = false;
        }
        allowed
    }

    /// predictHpLossVsType（a.java:9973-9978）：effectiveAttackVsType 桥接。
    fn predict_hp_loss(&self, t: i32, no_counter: bool) -> i32 {
        let idx = (t - 41) as usize;
        let atk = crate::combat::effective_attack_vs_type(&self.player, self.trait_flags(t));
        crate::combat::predict_battle_hp_loss(
            atk,
            self.player.def,
            self.enemies.hp[idx],
            self.enemies.atk[idx],
            self.enemies.def[idx],
            no_counter,
        )
    }

    /// f_byte_arr_05 特性位（initEntityTables a.java:6256-6258：49/53/74=1、69=2）。
    fn trait_flags(&self, t: i32) -> u8 {
        match t {
            49 | 53 | 74 => 1,
            69 => 2,
            _ => 0,
        }
    }

    /// tickBattle（a.java:10013-10145）：4 拍一击 → 胜利 → 死亡动画 6 拍 →
    /// tryStep(facing) 重试（胜利跳格）→ battleTargetEntity=-1。
    fn tick_battle(&mut self, allow_restep: bool) {
        if !self.battle_finishing {
            if self.battle_target < 0 {
                self.walk_phase = 0;
            } else {
                let e = self.battle_target as usize;
                let t = self.entities.entity_type[e];
                let idx = (t - 41) as usize;
                let dmg = crate::combat::effective_attack_vs_type(&self.player, self.trait_flags(t))
                    - self.enemies.def[idx];
                if dmg > 0 {
                    if (self.battle_tick & 3) == 0 {
                        self.battle_enemy_hp -= dmg;
                        let (ex, ey) = (self.entities.pixel_x[e], self.entities.pixel_y[e]);
                        self.popups.spawn(
                            5 + self.battle_tick % 3,
                            dmg,
                            ex,
                            ey,
                            self.view.cam_x,
                            self.view.cam_y,
                            self.images.popup.digit_strips[0].width / 11,
                        );
                        if self.battle_enemy_hp > 0 {
                            let counter = self.enemies.atk[idx] - self.player.def;
                            if counter > 0 {
                                self.player.hp -= counter;
                            }
                        } else {
                            // 击杀（a.java:10026-10058）：终伤预测 popup + 赏金 +
                            // boss 分支（floor1 无 73-75 型）→ 胜利动画阶段
                            let loss = self.predict_hp_loss(t, self.battle_no_counter);
                            if loss > 0 {
                                self.popups.spawn(
                                    2,
                                    loss,
                                    self.view.player_px,
                                    self.view.player_py,
                                    self.view.cam_x,
                                    self.view.cam_y,
                                    self.images.popup.digit_strips[0].width / 11,
                                );
                            }
                            let gold = if self.player.find_item(25) >= 0 {
                                self.enemy_base_gold[idx] << 1
                            } else {
                                self.enemy_base_gold[idx]
                            };
                            let (gx, gy) = (self.entities.pixel_x[e], self.entities.pixel_y[e]);
                            self.player.gold += gold;
                            if gold > 0 {
                                self.popups.spawn(
                                    4,
                                    gold,
                                    gx,
                                    gy,
                                    self.view.cam_x,
                                    self.view.cam_y,
                                    self.images.popup.digit_strips[0].width / 11,
                                );
                            }
                            self.battle_rerun_scene = false;
                            self.battle_finishing = true;
                        }
                    }
                } else {
                    self.walk_phase = 0;
                }
                self.battle_tick += 1;
            }
        } else {
            if self.battle_anim_tick == 0 {
                entity::remove(&mut self.entities, self.battle_target as usize);
            }
            self.battle_anim_tick += 1;
            if self.battle_anim_tick > 5 {
                self.battle_anim_tick = 0;
                if !self.battle_rerun_scene && allow_restep {
                    self.try_step(self.facing);
                }
                self.battle_finishing = false;
                self.battle_target = -1;
                // 胜利后类型特判（61-72 层绑定剧情，a.java:10108-10140）：floor1 无
            }
        }
    }

    /// applyStepCellEffects（a.java:5533-5725，floor1 可达子集）：
    /// 玩家格坐标刷新 + 格内实体效果（拾取 cat2/4；楼梯 7/8 → 换层批次）+
    /// walkPhase=0 + handleFieldInput（按住连走）。
    fn apply_step_cell_effects(&mut self) {
        self.player_cell_x = self.view.player_px >> 5;
        self.player_cell_y = self.view.player_py >> 5;
        let ct = self.grid.cell_type[self.player_cell_y as usize][self.player_cell_x as usize] as usize;
        let cap = self.grid.capacity[ct] as usize;
        let mut s = cap as i32 - 1;
        while s >= 0 {
            let e = self.grid.slots[ct][s as usize] as usize - 1;
            let t = self.entities.entity_type[e];
            match entity::render_category(t) {
                1 => match t {
                    6 => self.player_bob_applied = true, // f_bool_07（喷泉浮沉）
                    7 => {
                        // 上楼梯（a.java:5650-5655）：changeFloor(param, false, false)
                        let target = (self.entities.param[e] & 0xff) as i32;
                        self.change_floor(target, false, false);
                        self.key_held = 0;
                    }
                    8 => {
                        // 下楼梯（a.java:5656-5661）：changeFloor(param, true, false)
                        let target = (self.entities.param[e] & 0xff) as i32;
                        self.change_floor(target, true, false);
                        self.key_held = 0;
                    }
                    _ => {} // 5 宝箱/76/83 场景门：后续批次
                },
                2 | 4 => {
                    self.pickup_item_type(t);
                    entity::remove(&mut self.entities, e);
                }
                _ => {}
            }
            s -= 1;
        }
        if self.walk_phase != 4 {
            self.walk_phase = 0;
            self.handle_field_input();
        }
    }

    /// pickupItemType（a.java:7515-7617，floor1 可达子集）+ applyHpDelta
    /// （a.java:5162-5180，difficultyIndex=0 → 无 ×16）。
    fn pickup_item_type(&mut self, t: i32) {
        match t {
            13..=25 | 85 | 86 => {
                // 物品入栈（addItemToItemStack）：type 15 另置 f_bool_12
                if t == 15 {
                    // f_bool_12（教程标记）：教程批次
                }
                let n = self.player.item_stack_size as usize;
                if n < self.player.item_types.len() {
                    self.player.item_types[n] = t as u8;
                    self.player.item_stack_size += 1;
                }
            }
            26 => self.player.yellow_keys += 1,
            27 => self.player.red_keys += 1,
            28 => self.player.blue_keys += 1,
            29 => self.player.atk += 1, // floor ≤ 10 分支
            30 => self.player.def += 1,
            31 => {
                let heal = 50;
                self.apply_hp_delta(heal);
            }
            32 => {
                let heal = 200;
                self.apply_hp_delta(heal);
            }
            _ => {} // 33-40/79/80 装备：装备批次；41+ 怪物类型不可达
        }
    }

    /// applyHpDelta（a.java:5162-5180）：负 → popup2；正 → popup3。
    fn apply_hp_delta(&mut self, mut delta: i32) {
        if self.player.hp <= -delta {
            self.player.hp = 1;
            delta = self.player.hp - 1;
        } else {
            self.player.hp += delta;
        }
        if delta < 0 {
            self.popups.spawn(
                2,
                delta,
                self.view.player_px,
                self.view.player_py,
                self.view.cam_x,
                self.view.cam_y,
                self.images.popup.digit_strips[0].width / 11,
            );
        } else if delta > 0 {
            self.popups.spawn(
                3,
                delta,
                self.view.player_px,
                self.view.player_py,
                self.view.cam_x,
                self.view.cam_y,
                self.images.popup.digit_strips[0].width / 11,
            );
        }
    }

    /// paint case 3（a.java:2385-2422）。
    pub fn paint(&mut self, g: &mut SoftGraphics<'_>, tileset: &ArgbImage) {
        self.paint_parallax(g, true);
        self.view.paint_tiles(g, tileset, view_top_y());
        self.paint_entities(g, self.view.cam_x, self.view.cam_y + view_top_y());
        // 小地图（optionChecked[1] && minimapImage）：右上角，玩家点
        if self.minimap_enabled {
            if let Some(mm) = &self.minimap {
                let x = SCREEN_W - ((self.view.wide + 1) * MINIMAP_PX_PER_CELL);
                g.draw_image(mm, x, view_top_y(), 0);
                g.set_color(MINIMAP_PLAYER_DOT);
                // a.java:2397-2398：点 = playerCellX/Y（**格坐标**——步进中
                // 与像素 >>5 不同，上/左行时像素已进目标格而格坐标未更新）
                g.fill_rect(
                    x + self.player_cell_x * MINIMAP_PX_PER_CELL,
                    view_top_y() + self.player_cell_y * MINIMAP_PX_PER_CELL,
                    MINIMAP_DOT_PX,
                    MINIMAP_DOT_PX,
                );
            }
        }
        self.paint_hud(g);
        self.paint_status_bar(g);
        // 楼梯指示（f_bool_13；稳态 false—— proximity 更新在 lockCameraOn/行走批次）
        self.popups.draw(g, &self.images.popup);
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1);
        // 换层遮幅（paint 公共尾 a.java:3171-3215）：状态机先行、后画黑格。
        // 闭合（wipe_closing）1..=4 全黑 → 中段换层 → 开启 4..=0。
        if self.transitioning {
            if self.wipe_closing {
                self.wipe += 1;
                if self.wipe > 4 {
                    self.wipe = 4;
                    self.wipe_closing = false;
                    self.swap_floor();
                }
            } else {
                self.wipe -= 1;
                if self.wipe <= 0 {
                    self.wipe_closing = true;
                    self.transitioning = false;
                }
            }
            let size = self.wipe * 2;
            let off = 4 - self.wipe;
            g.set_color(BLACK);
            for row in 0..40 {
                let y = off + row * 8;
                for col in 0..30 {
                    g.fill_rect(off + col * 8, y, size, size);
                }
            }
        }
    }

    /// drawParallaxBackdrop（a.java:6840-6867）：滚动 -1/拍（PARALLAX_WRAP 回绕），
    /// PARALLAX_COL_STEP 列步进、SCREEN_H 行步进、列内交替镜像。
    fn paint_parallax(&mut self, g: &mut SoftGraphics<'_>, game_view: bool) {
        self.backdrop_scroll -= 1;
        if self.backdrop_scroll < PARALLAX_WRAP {
            self.backdrop_scroll = 0;
        }
        let base_y = if game_view { parallax_game_base_y() } else { 0 };
        let mut col_x = self.backdrop_scroll;
        while col_x < SCREEN_W {
            let mut mirrored = false;
            let mut y = base_y;
            while y > -SCREEN_H {
                if mirrored {
                    g.draw_image_transformed(&self.images.backdrop, col_x, y, 0, 2);
                } else {
                    g.draw_image(&self.images.backdrop, col_x, y, 0);
                }
                mirrored = !mirrored;
                y -= SCREEN_H;
            }
            col_x += PARALLAX_COL_STEP;
        }
    }

    /// paintEntityLayer（a.java:6481-6713）：实体层。`cam_x/cam_y` 为调用参数
    /// （paint case 3：cameraPixelX / cameraPixelY+view_top）。
    /// floor1 类别覆盖 1/2/8 + 空图文字标签；16/32/67/69 拼装 todo（floor1
    /// 实体清单无这些类型，spec p3-render §3.5）。
    pub fn paint_entities(&mut self, g: &mut SoftGraphics<'_>, cam_x: i32, cam_y: i32) {
        entity::advance_bob(&mut self.bob_offset, &mut self.bob_rising);
        let bob = self.bob_offset;
        let mut player_drawn = false;
        let cull_bottom = view_top_y() + self.view.view_h;

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
                if sx >= -w && sx <= self.view.view_w && sy >= ENTITY_CULL_TOP && sy <= cull_bottom {
                    if let Some(img) = &self.images.entity[t as usize] {
                        // 玩家插入（单次；blocksPlayerInsert 在 0/5/6/7/8/10 型为 true，
                        // 这些类型占据插入窗口时不插画玩家——a.java:6510）
                        if !player_drawn
                            && self.view.player_px > ex - PLAYER_INSERT_WINDOW
                            && self.view.player_px < ex + PLAYER_INSERT_WINDOW
                            && self.view.player_py >= ey - PLAYER_INSERT_WINDOW
                            && ey > self.view.player_py
                            && !BLOCKS_PLAYER_INSERT.get(t as usize).copied().unwrap_or(false)
                        {
                            self.paint_player(g, cam_x, cam_y);
                            player_drawn = true;
                        }
                        let anim = self.entities.anim_idx[e] as usize;
                        let frame = self.entities.frame[e];
                        // 帧偏移 = 帧宽 × animOffsetTable[行][帧]（Java var5 算式）
                        let frame_off = w * ANIM_OFFSET_TABLE[anim][frame as usize];
                        // 注：横向居中 (32-w)>>1 在**各臂内**取 sx 现值计算——
                        // case 8 抖动 / case 1 的 6/9 特例先改 sx 再用（Java var9 语义）
                        let cx_off = (CELL_PX - w) >> 1;
                        match entity::render_category(t) {
                            1 => {
                                if t == 6 {
                                    sy += 5 - bob;
                                } else if t == 9 {
                                    sx += CELL_PX;
                                }
                                let img = if state == 1 { &self.images.opened_door } else { img };
                                draw_image_clipped(g, img, (sx + cx_off), sy - (h - CELL_PX), frame_off, 0, w, h);
                            }
                            2 => {
                                if state == 1 {
                                    draw_image_clipped(g, &self.images.opened_door, (sx + cx_off), sy - (h - CELL_PX), frame_off, 0, w, h);
                                } else if state == 2 {
                                    draw_image_clipped(g, &self.images.open_anim, sx + OPEN_ANIM_DX, sy + OPEN_ANIM_DX - (frame * OPEN_ANIM_STEP), frame_off, 0, OPEN_ANIM_W, OPEN_ANIM_H);
                                } else if t < SMALL_ITEM_MAX_TYPE {
                                    g.draw_image(&self.images.shadow, sx + SHADOW_DX, sy + SHADOW_DY, 0);
                                    g.draw_image(img, (sx + cx_off), sy - (h - ANCHOR_MARGIN_24) + bob, 0);
                                } else {
                                    g.draw_image(img, (sx + cx_off), sy - (h - ANCHOR_MARGIN_30), 0);
                                }
                            }
                            4 => {
                                g.draw_image(&self.images.shadow, sx + SHADOW_DX, sy + SHADOW_DY, 0);
                                draw_image_clipped(g, img, (sx + cx_off), sy - (h - ANCHOR_MARGIN_24) + bob, frame_off, 0, w, h);
                            }
                            8 => {
                                if state == 3 {
                                    // 战斗目标抖动（a.java:6533-6537）：x/y 各随机 ±2
                                    sx += self.rng.random_below(5) - 2;
                                    sy += self.rng.random_below(5) - 2;
                                }
                                if state == 1 || state == 2 {
                                    draw_image_clipped(g, &self.images.open_anim, sx + OPEN_ANIM_DX, sy + OPEN_ANIM_DX - (frame * OPEN_ANIM_STEP), frame_off, 0, OPEN_ANIM_W, OPEN_ANIM_H);
                                } else if t != 67 && t != 69 {
                                    g.draw_image(&self.images.shadow, sx + SHADOW_DX, sy + SHADOW_DY, 0);
                                    sy -= ENTITY_LIFT;
                                    draw_image_clipped(g, img, (sx + cx_off), sy - (h - CELL_PX), frame_off, 0, w, h);
                                } else {
                                    // 67/69 拼装表组合（f_byte_arr2_00/01 元数据）——floor1 无
                                    todo!("paintEntityLayer case 8 的 67/69 拼装组合（floor1 实体清单无）")
                                }
                                // walkPhase==2 战后数字条（a.java:6607-6634）：
                                // showBattleDigits 开关 + iconStrip 波动 + 预测伤害 drawDigitStrip
                                if self.walk_phase == 2 && self.show_battle_digits {
                                    let f8 = (self.frame_counter & 7) as usize;
                                    let d = crate::title::STRIP_DX[f8];
                                    let e = crate::title::STRIP_DY[f8];
                                    g.draw_image(&self.images.battle_icon, sx - 3 + d, sy - 24 + e, 0);
                                    let digit = self.battle_digit_by_type[t as usize];
                                    if digit >= 0 {
                                        crate::popup::draw_digit_strip(
                                            g,
                                            &self.images.battle_strip,
                                            digit,
                                            sx + 30 + d,
                                            sy - 17 + e,
                                        );
                                    } else {
                                        draw_image_clipped(g, &self.images.battle_strip, sx + 14 + d, sy - 17 + e, 70, 0, 7, 9);
                                    }
                                }
                            }
                            16 => {
                                if state != 1 && state != 2 {
                                    if let Some(img15) = &self.images.entity[15] {
                                        g.draw_image(img15, sx + 1, sy + 8 + bob, 0);
                                    }
                                    draw_image_clipped(g, img, (sx + cx_off), sy - (h - ANCHOR_MARGIN_24) + bob, frame_off, 0, w, h);
                                } else {
                                    draw_image_clipped(g, &self.images.open_anim, sx + OPEN_ANIM_DX, sy + OPEN_ANIM_DX - (frame * OPEN_ANIM_STEP), frame_off, 0, OPEN_ANIM_W, OPEN_ANIM_H);
                                }
                            }
                            32 => {
                                g.draw_image(&self.images.shadow, sx + SHADOW_DX, sy + SHADOW_DY, 0);
                                if state == 1 || state == 2 {
                                    draw_image_clipped(g, &self.images.open_anim, sx + OPEN_ANIM_DX, sy + OPEN_ANIM_DX - (frame * OPEN_ANIM_STEP), frame_off, 0, OPEN_ANIM_W, OPEN_ANIM_H);
                                } else if t != 72 {
                                    draw_image_clipped(g, img, (sx + cx_off), sy - (h - ANCHOR_MARGIN_16) + bob, frame_off, 0, w, h);
                                } else if self.route_entity != e as i32 {
                                    // 型 72 非追踪实体：静态（无 bob），y-(h-32)（a.java:6647-6648）
                                    draw_image_clipped(g, img, (sx + cx_off), sy - (h - CELL_PX), frame_off, 0, w, h);
                                } else {
                                    // 追踪实体：形态号帧（f_byte_19；==3 时镜像，a.java:6650-6657）
                                    let mut src_y = h * self.form_19;
                                    if self.form_19 == 3 {
                                        src_y -= h;
                                        crate::menu_family::draw_edge_patch(g, img, sx + cx_off, sy - (h - CELL_PX), frame_off, src_y, w, h, 1);
                                    } else {
                                        draw_image_clipped(g, img, (sx + cx_off), sy - (h - CELL_PX), frame_off, src_y, w, h);
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        // 空图文字标签（objectTypeNames 圆底标签；type 83 走此径但 m_122 置 entityVisible=false）
                        todo!("paintEntityLayer 空图标签分支（objectTypeNames 文本，floor1 不可达）")
                    }
                }
            }
        }

        g.set_clip(0, 0, SCREEN_W, SCREEN_H);
        if !player_drawn {
            self.paint_player(g, cam_x, cam_y);
        }
        // walkPhase==5 星光（a.java:6682-6700）：battleTick&7 选帧，sparkFrames
        // 帧号×4 取 sparkQuads 四联组（sx,sy,w,h）
        if self.walk_phase == 5 && self.battle_target >= 0 {
            let e = self.battle_target as usize;
            // a.java:6682-6683：星光坐标用**裸 cameraPixelX/Y**（非 m_053 入参
            // 的 cameraPixelY+view_top——尾段直接读字段）
            let x = self.view.cam_x + self.entities.pixel_x[e] + 16;
            let y = self.view.cam_y + self.entities.pixel_y[e] + 32;
            let phase = (self.battle_tick & 7) as usize;
            let frame = SPARK_FRAME[phase];
            if frame > 0 {
                let q = ((frame - 1) << 2) as usize;
                draw_image_clipped(
                    g,
                    &self.images.spark_sheet,
                    x - (SPARK_QUADS[q + 2] >> 1),
                    y - (SPARK_QUADS[q + 3] >> 1),
                    SPARK_QUADS[q],
                    SPARK_QUADS[q + 1],
                    SPARK_QUADS[q + 2],
                    SPARK_QUADS[q + 3],
                );
            }
        }
        // 楼梯浮标（paintEntityLayer 尾段 a.java:6702-6716）
        if let Some((ux, uy)) = self.entities.stair_up {
            let x = cam_x + ux;
            let y = cam_y + uy;
            if x >= MARKER_CULL_MARGIN && x <= self.view.view_w && y >= ENTITY_CULL_TOP && y <= cull_bottom {
                g.draw_image(&self.images.stair_icons.0, x + MARKER_UP_DX, y + MARKER_DY + bob, 0);
            }
        }
        if let Some((dx, dy)) = self.entities.stair_down {
            let x = cam_x + dx;
            let y = cam_y + dy;
            if x >= MARKER_CULL_MARGIN && x <= self.view.view_w && y >= ENTITY_CULL_TOP && y <= cull_bottom {
                g.draw_image(&self.images.stair_icons.1, x, y + MARKER_DY + bob, 0);
            }
        }
    }

    /// paintPlayerSprite（a.java:5831-5905）：玩家。facing 0/1/2/3 = 下/上/右/左行
    /// （PLAYER_ANCHORS 的 (dx,dy,行)；case 3 为镜像——drawEdgePatch transform 1）；
    /// walkPhase==2 残影（afterimageX/Y）战斗批次。
    pub fn paint_player(&self, g: &mut SoftGraphics<'_>, cam_x: i32, cam_y: i32) {
        let px = cam_x + self.view.player_px;
        let mut py = cam_y + self.view.player_py;
        if self.player_bob_applied {
            py -= self.bob_offset;
        }
        g.draw_image(&self.images.shadow, px + SHADOW_DX, py + SHADOW_DY, 0);
        let frame_x = PLAYER_FRAME_TABLE[self.player_frame as usize] * PLAYER_FRAME_W;
        let (fw, fh) = (PLAYER_FRAME_W, PLAYER_FRAME_H);
        let idx = self.facing.clamp(0, 3) as usize;
        let (dx, dy, row) = PLAYER_ANCHORS[idx];
        let sy = fh * row;
        // walkPhase==2 残影（a.java:5850-5900）：facing 1 在主精灵**后**画，
        // 其余在前；i=3..0 逆序。残影源 = playerGhostSheet（actor 条带别名）
        let ghost = self.walk_phase == 2 && self.facing != 1;
        if ghost {
            self.paint_afterimages(g, cam_x, cam_y, frame_x);
        }
        if self.facing == 3 {
            crate::menu_family::draw_edge_patch(g, &self.images.actor, px + dx, py + dy, frame_x, sy, fw, fh, 1);
        } else {
            draw_image_clipped(g, &self.images.actor, px + dx, py + dy, frame_x, sy, fw, fh);
        }
        if self.walk_phase == 2 && self.facing == 1 {
            self.paint_afterimages(g, cam_x, cam_y, frame_x);
        }
        g.set_clip(0, 0, SCREEN_W, SCREEN_H);
        // f_bool_26 路径走格精灵（[8][22]）：脚本行走批次（spec 待办）
        // walkPhase==2 四角装饰（a.java:5949-5955，画后直接 return 语义）
        if self.walk_phase == 2 {
            let bob = self.bob_offset;
            let vb = self.view_bottom;
            draw_image_clipped(g, &self.images.corner_sprites, 5 - bob, 148, 44, 0, 22, 24);
            draw_image_clipped(g, &self.images.corner_sprites, 109, 25 - bob, 0, 0, 22, 24);
            draw_image_clipped(g, &self.images.corner_sprites, 109, vb - 29 + bob, 22, 0, 22, 24);
            crate::menu_family::draw_edge_patch(g, &self.images.corner_sprites, 215 + bob, 148, 44, 0, 22, 24, 1);
        }
        // walkPhase==3 道具面板：道具批次（spec 待办）
    }

    /// walkPhase==2 的四向残影（a.java:5850-5900）。
    fn paint_afterimages(&self, g: &mut SoftGraphics<'_>, cam_x: i32, cam_y: i32, frame_x: i32) {
        let (fw, fh) = (PLAYER_FRAME_W, PLAYER_FRAME_H);
        for i in (0..4).rev() {
            let ax = cam_x + self.afterimage_x[i];
            let ay = cam_y + self.afterimage_y[i];
            match self.facing {
                0 => draw_image_clipped(g, &self.images.actor, ax - 4, ay - 14, frame_x, 0, fw, fh),
                1 => draw_image_clipped(g, &self.images.actor, ax - 8, ay - 14, frame_x, fh, fw, fh),
                2 => draw_image_clipped(g, &self.images.actor, ax - 6, ay - 14, frame_x, fh * 2, fw, fh),
                _ => crate::menu_family::draw_edge_patch(g, &self.images.actor, ax, ay - 14, frame_x, fh * 2, fw, fh, 1),
            }
        }
    }

    /// paintHudFrame（a.java:6124-6153）：HUD 大框（ui[8][0] 边框件 64×16）。
    fn paint_hud_frame(&mut self, g: &mut SoftGraphics<'_>, x: i32, y: i32, w: i32, h: i32) {
        let img = &self.images.ui[0];
        let (corner, mid, side) = (BORDER_CORNER_W, BORDER_MID_W, BORDER_SIDE_W);
        let strip = BORDER_STRIP_H;
        g.set_clip(x, y, corner, strip);
        g.draw_image(img, x, y, 0);
        let mut cx = x + corner;
        while cx < x + w - corner {
            g.set_clip(cx, y, mid, strip);
            g.draw_image(img, cx - corner, y, 0);
            cx += mid;
        }
        g.set_clip(0, 0, SCREEN_W, SCREEN_H);
        crate::menu_family::draw_edge_patch(g, img, x + w - corner, y, 0, 0, corner, strip, 1);
        g.set_color(HUD_FILL);
        g.fill_rect(x + side, y + strip, w - side * 2, h - strip);
        let right = x + w - side;
        let mut cy = y + strip;
        while cy < y + h {
            g.set_clip(x, cy, side, strip);
            g.draw_image(img, x - BORDER_SIDE_SRC_L, cy, 0);
            g.set_clip(right, cy, side, strip);
            g.draw_image(img, right - BORDER_SIDE_SRC_R, cy, 0);
            cy += strip;
        }
        g.set_clip(0, 0, SCREEN_W, SCREEN_H);
    }

    /// paintHudPanel（a.java:6042-6102）：HUD（头像/HP/攻/防/武器/甲）。
    pub fn paint_hud(&mut self, g: &mut SoftGraphics<'_>) {
        let font = crate::paint::paint_font();
        let base_y = self.view_bottom;
        self.paint_hud_frame(g, 0, base_y, SCREEN_W, HUD_FRAME_H);
        let face = &self.images.actor_face;
        g.draw_image(face, HUD_FACE_AX - (face.width >> 1), base_y + HUD_FRAME_H - face.height, 0);
        let bar_x = HUD_ICON_X + HUD_BAR_DX;
        let mut y = base_y + HUD_STAT_Y0;
        // HP 行：图标 HUD_ICON_CLIP 见方 + 条框 + 数值
        g.set_clip(HUD_ICON_X, y, HUD_ICON_CLIP, HUD_ICON_CLIP);
        g.draw_image(&self.images.ui[7], HUD_ICON_X, y + HUD_ICON_LIFT, 0);
        g.set_clip(0, 0, SCREEN_W, SCREEN_H);
        paint_mini_frame(g, bar_x, y + 1, HUD_BAR_W, HUD_BAR_H);
        paint_number(g, &self.images.ui[2], self.player.hp, bar_x + HUD_NUM_DX, y + 2);
        // 攻行：图标条带片 (10,0,10,13)
        y += HUD_ROW_STEP;
        draw_image_clipped(g, &self.images.ui[7], HUD_ICON_X, y, 10, 0, HUD_ICON_CLIP, HUD_ICON_STRIP_H);
        g.set_color(HUD_ROW_ACCENT);
        paint_mini_frame(g, bar_x, y + 1, HUD_BAR_W, HUD_BAR_H);
        paint_number(g, &self.images.ui[2], self.player.atk, bar_x + HUD_NUM_DX, y + 2);
        // 防行：图标条带片 (20,0,10,13)
        y += HUD_ROW_STEP;
        draw_image_clipped(g, &self.images.ui[7], HUD_ICON_X, y, 20, 0, HUD_ICON_CLIP, HUD_ICON_STRIP_H);
        g.set_color(HUD_ROW_ACCENT);
        paint_mini_frame(g, bar_x, y + 1, HUD_BAR_W, HUD_BAR_H);
        paint_number(g, &self.images.ui[2], self.player.def, bar_x + HUD_NUM_DX, y + 2);
        // 装备槽（equipmentMaterialNames = 无/木/铁/银/金/布/皮/锁/金，a.java:484）
        // 槽 x：var11 = HUD_ICON_X+16 后 +60 → 159；甲槽 +34 → 193（a.java:6070/6087）
        let slot_y = base_y + HUD_SLOT_DY;
        let mut slot_x = bar_x + 60;
        let none: Vec<u16> = "无".encode_utf16().collect();
        let cw = font.char_width(none[0]);
        for _ in 0..2 {
            paint_mini_frame(g, slot_x, slot_y, HUD_SLOT_SIZE, HUD_SLOT_SIZE);
            g.set_color(WHITE);
            g.draw_string(
                &none,
                slot_x + ((HUD_SLOT_SIZE - cw) >> 1),
                slot_y + ((HUD_SLOT_SIZE - font.height) >> 1),
                0,
            );
            slot_x += HUD_SLOT_STEP;
        }
    }

    /// paintStatusBar（a.java:6008-6031）：顶部状态栏（层数/三钥/金币）+ paintKeySlot。
    pub fn paint_status_bar(&mut self, g: &mut SoftGraphics<'_>) {
        let base_y = 0;
        g.set_clip(0, 0, SCREEN_W, SCREEN_H);
        g.draw_image(&self.images.ui[3], 0, base_y, 0);
        // currentFloor ≤ 50 分支（>50 的"引子"文字属尾声楼层）
        paint_number(g, &self.images.ui[18], self.floor, STATUS_FLOOR_NUM_X, base_y + 4 + 2);
        g.draw_image(&self.images.ui[8], STATUS_TIER_ICON_X, base_y + 4 + 10, 0);
        g.set_color(STATUS_FILL);
        g.fill_rect(STATUS_KEY_X[0], base_y, SCREEN_W - STATUS_KEY_X[0], STATUS_BAR_H);
        self.paint_key_slot(g, 0, self.player.yellow_keys, STATUS_KEY_X[0], base_y);
        self.paint_key_slot(g, 1, self.player.blue_keys, STATUS_KEY_X[1], base_y);
        self.paint_key_slot(g, 2, self.player.red_keys, STATUS_KEY_X[2], base_y);
        let y = base_y + 3;
        g.draw_image(&self.images.ui[1], STATUS_GOLD_ICON_X, y, 0);
        paint_number(g, &self.images.ui[2], self.player.gold, STATUS_GOLD_NUM_X, y + 2);
    }

    /// paintKeySlot（a.java:6033-6040）：钥匙格（ui[8][6] KEY_SLOT_W 三联图 + 数量）。
    fn paint_key_slot(&mut self, g: &mut SoftGraphics<'_>, kind: i32, count: i32, x: i32, y: i32) {
        g.set_clip(x, y, KEY_SLOT_W, KEY_SLOT_H);
        g.draw_image(&self.images.ui[6], x - kind * KEY_SLOT_W, y, 0);
        g.set_clip(0, 0, SCREEN_W, SCREEN_H);
        paint_number(g, &self.images.ui[2], count, x + KEY_NUM_DX, y + KEY_NUM_DY);
    }
}

/// blit5Clip（a.java:4547-4552）：MINIMAP_MARKER_PX 定尺寸裁剪贴图（小地图标记）。
fn m_003_clip5(g: &mut SoftGraphics<'_>, img: &ArgbImage, x: i32, y: i32, src_x: i32) {
    g.set_clip(x, y, MINIMAP_MARKER_PX, MINIMAP_MARKER_PX);
    g.draw_image(img, x - src_x, y, 0);
    g.set_clip(0, 0, SCREEN_W, SCREEN_H);
}
