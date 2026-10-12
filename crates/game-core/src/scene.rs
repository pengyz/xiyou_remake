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
#[derive(Clone)]
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
    /// map[2][11]：SEE phase6 目标箭头
    pub map211: Option<ArgbImage>,
    /// map[2][10]：道具面板选中框（38×38 双帧）
    pub map210: ArgbImage,
    /// f_Image_arr2_00[10][1]：菜单图标动画条（mode16 设置条）
    pub menu_strip: ArgbImage,
    /// map[2][8]：战后数字条目标图标
    pub battle_icon: ArgbImage,
    /// map[2][7]：战后数字条（drawDigitStrip 7px 数字带）
    pub battle_strip: ArgbImage,
    /// f_Image_arr2_00[12][*]：face 包（12 张 26×22，零余字节实测）——对话
    /// 气泡标记表，索引 = f_byte_arr_22[dialogPage]（a.java:2887）。
    /// **不是** entityTypeImage——迭代 55 根因：entity[bubble] 误用致 245×39
    /// 条带上屏（T303 op 流 872 条唯一分歧 op 实证）。
    pub faces: Vec<ArgbImage>,
}

impl SceneImages {
    /// m_122 头部的类型→容器映射（a.java:9448-9470）：
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

/// bossTypeOrder（a.java:2198）：boss 型 → bossEventSpawns 事件序（击杀钩子）。
const BOSS_TYPE_ORDER: [i32; 6] = [75, 74, 68, 67, 69, 71];
/// bossEventSpawns（a.java:2041-2104）：每事件 = [型, 格x, 格y, param]×N。
const BOSS_EVENT_SPAWNS: [&[i32]; 7] = [
    &[
        29, 1, 3, 0, 29, 2, 3, 0, 29, 3, 3, 0, 26, 1, 4, 0, 26, 2, 4, 0, 26, 3, 4, 0, 30, 9, 3,
        0, 30, 10, 3, 0, 30, 11, 3, 0, 32, 9, 4, 0, 32, 10, 4, 0, 32, 11, 4, 0,
    ],
    &[
        29, 4, 4, 0, 29, 4, 5, 0, 29, 5, 4, 0, 26, 7, 4, 0, 26, 8, 4, 0, 26, 8, 5, 0, 30, 4, 7,
        0, 30, 4, 8, 0, 30, 5, 8, 0, 32, 8, 8, 0, 32, 7, 8, 0, 32, 8, 7, 0,
    ],
    &[27, 6, 5, 0, 27, 5, 6, 0, 27, 7, 6, 0, 27, 6, 7, 0],
    &[17, 6, 4, 0],
    &[16, 5, 5, 0],
    &[24, 6, 2, 0, 32, 5, 1, 0, 32, 6, 1, 0, 32, 7, 1, 0, 27, 6, 6, 0],
    &[
        32, 2, 3, 0, 32, 3, 3, 0, 32, 4, 3, 0, 26, 8, 3, 0, 26, 9, 3, 0, 26, 10, 3, 0, 29, 3, 5,
        0, 29, 4, 5, 0, 29, 5, 5, 0, 30, 7, 5, 0, 30, 8, 5, 0, 30, 9, 5, 0,
    ],
];
const SPARK_FRAME: [i32; 8] = [1, 2, 0, 3, 4, 1, 2, 0];
/// 星光四联组 (sx,sy,w,h)×4（Java sparkQuads，a.java:748-764）。
const SPARK_QUADS: [i32; 16] = [29, 31, 39, 15, 0, 31, 29, 10, 0, 0, 37, 31, 36, 0, 32, 29];

/// blocksPlayerInsert（initEntityTables 从构造字面量 f_bool_arr_03 拷贝，a.java:679/
/// 6237）：true 的类型（0/5/6/7/8/10）占据插入窗口时**不**在其前插画玩家
/// （paintEntityLayer 插入条件的 `!blocksPlayerInsert[type]` 项，a.java:6520）。
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

/// 对白数据集（构造期从夹具装载；game-core 无 I/O——include_str 编译期）。
pub struct DialogData {
    /// levelScriptLines（68 行 DSL）
    pub scripts: Vec<String>,
    /// dialogueTexts（269 条）
    pub texts: Vec<String>,
    /// dialogueSpeakerType（269 项）
    pub speakers: Vec<i8>,
    /// f_byte_arr_22（300 项气泡 face 索引）
    pub bubbles: Vec<i8>,
}

/// dialogue_texts.txt 行反转义（行安全格式：`\\` `\n` `\r` 为字面转义，
/// 由 a.java dialogueTexts 初始化器 a.java:1909-1930 一致提取；2026-10-11
/// 修复旧夹具把 trace 转义层 `\\c` 当内容的污染，19 条含 \c 转义的对白）。
pub fn unescape_text_fixture(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut it = line.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

impl DialogData {
    fn from_fixtures() -> Self {
        let scripts = include_str!("../tests/fixtures/level_scripts.txt")
            .lines()
            .map(|l| l.trim().to_string())
            .collect();
        let texts = include_str!("../tests/fixtures/dialogue_texts.txt")
            .lines()
            .map(unescape_text_fixture)
            .collect();
        let speakers = include_str!("../tests/fixtures/speaker_types.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect();
        let bubbles = include_str!("../tests/fixtures/speaker_bubbles.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect();
        Self { scripts, texts, speakers, bubbles }
    }
}

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
    /// optionChecked[1]：小地图开关（RMS 读档失败 catch 默认 true，a.java:4475-4471）
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
    /// —— mode 9 炼丹（run case 9 a.java:3885-3940 / paint case 9 a.java:2690-2740）——
    /// alchemyPrice：当前升级价（alchemyPriceFor(upgrades+1)，a.java:7256）
    pub alchemy_price: i32,
    /// f_int_77/78/79：本次升级 HP/ATK/DEF 增量（scaledByFloorTier，a.java:7250）
    pub alchemy_gain_hp: i32,
    pub alchemy_gain_atk: i32,
    pub alchemy_gain_def: i32,
    /// f_int_80：三行选择光标（0=HP 1=ATK 2=DEF）
    pub alchemy_cursor: i32,
    /// alchemyUpgradeCount：运行时升级计数（存档 alchemy_upgrades 装入）
    pub alchemy_upgrade_count: i32,
    /// —— 商人/告示牌链（interact case16 a.java:5494-5517 + overlay 分派
    ///     case4/5 a.java:3300-3334 + applyMerchantOffer a.java:7307-7370）——
    /// f_int_87：最近 interact 命中的实体下标
    pub merchant_entity: i32,
    /// 报价队列（m_075/m_076，f_byte_arr_14/15 + f_int_88/89/90）：(文本源, 报价序号)
    pub offer_queue_src: Vec<u8>,
    pub offer_queue_idx: Vec<u8>,
    pub offer_queue_cursor: i32,
    /// 软键（m_000 case 3：1/3）
    pub softkeys: (i8, i8),

    // —— 输入（keyPressed/keyReleased a.java:4663-4675）——
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

    // —— 场上战斗（walkPhase 5；tickBattle a.java:10023-10149）——
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

    // —— 换层（changeFloor a.java:7126-7164 + paint 遮幅 a.java:3177-3215）——
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
    /// f_byte_arr2_05：层状态存档（m_119 写/m_122 读；索引=层号）
    pub saved_floor_states: Vec<Option<Vec<u8>>>,
    /// equippedWeaponType/ArmorType（m_068 重置；tryEquip 更新；HUD 槽显示）
    pub equipped_weapon: i32,
    pub equipped_armor: i32,
    /// equipTierBonuses/Types（a.java:977-991 构造常量）
    pub equip_bonuses: [i32; 12],
    pub equip_types: [i32; 12],
    /// 提示浮层（showOverlayMessage a.java:4898-4916）：active/kind/text/框宽高/页
    pub overlay_active: bool,
    pub overlay_kind: i32,
    pub overlay_text: Option<String>,
    pub overlay_box_w: i32,
    pub overlay_box_h: i32,
    pub overlay_left_icon: i8,
    pub overlay_right_icon: i8,
    /// layoutWrappedText 产物（行列表 + 分页游标）
    pub overlay_lines: Vec<String>,
    pub overlay_page_top: i32,
    pub overlay_page_bottom: i32,
    pub overlay_lines_per_page: i32,
    pub overlay_total_lines: i32,
    pub overlay_content_w: i32,
    /// 组开门动画数组（tryOpenChest 填充；paintTileLayer walkPhase4 消耗）
    pub open_group: Vec<usize>,
    /// f_int_55：组开门动画计数（18 置 removed 动画、24 收尾）
    pub open_anim_tick: i32,
    /// f_int_54：遮幅纵向抖动（±2 交替）
    pub tile_bob: i32,
    /// f_bool_12：教程标记（型 15 拾取置 true——f_bool_13 楼梯指示的门控）
    pub tutorial_seen: bool,
    /// f_bool_13：楼梯指示（49/55 暗道换乘键门控；proximity 更新——
    /// lockCameraOn/行走批次——未移植，稳态 false 与 Java 稳态一致）
    pub stair_hint: bool,

    // —— mode 11 对话（gameMode 3=世界 / 11=对话；a.java:4011-4068 相位机）——
    pub game_mode: i32,
    /// dialogPhase：0 脚本步进 / 1 TAK 打字 / 2 实体路径 / 3 脚本行走 / 4 等相机 / 6 SEE 打字
    pub dialog_phase: i32,
    /// dialogPage：当前对白页（dialogueTexts 下标）
    pub dialog_page: i32,
    /// dialogPageEnd：TAK 页尾（openDialogPage 边界）
    pub dialog_page_end: i32,
    /// dialogSpeakerType：说话者类型（>=0 实体立绘 / 0 玩家头像 / <0 "?"）
    pub dialog_speaker: i32,
    /// dialogAnchorEntity：lockCameraOn 返回的锚实体（气泡定位）
    pub dialog_anchor: i32,
    /// typewriterPos：打字机游标（UTF-16 单位）
    pub dialog_char: i32,
    /// dialogMoveX/115：SEE/MOV 目标格
    pub dialog_target: (i32, i32),
    /// dialogScriptLine：当前脚本行
    pub script_line: String,
    /// scriptCursor
    pub script_cursor: usize,
    /// scriptStepDelay：phase0 步进延迟
    pub script_delay: i32,
    /// scriptLineFlags（68 位）
    pub script_line_flags: Vec<bool>,
    /// currentScriptIndex
    pub current_script_index: usize,
    /// 对白数据集
    pub dialog_data: DialogData,
    /// difficultyIndex（难度乘数索引；tryRunScene case 67 门控）
    pub difficulty_index: i32,
    /// scriptWalkArmed（脚本行走武装；phase 3 门控）
    pub script_walk_armed: bool,
    /// overlaySavedColor（paintWrappedText \c 保存/\r 恢复的色；Java 字段默认 0）
    pub overlay_saved_color: i32,
    /// mode 4/19 菜单族共享布局（menuLabels a.java:458-462 / resetMenuLayout
    /// a.java:4690-4702 / addMenuItem a.java:4704-4716）
    pub menu_kinds: Vec<i8>,
    pub menu_item_count: i32,
    pub menu_box_width: i32,
    pub menu_line_height: i32,
    pub menu_visible_rows: i32,
    pub menu_list_height: i32,
    pub menu_scroll_top: i32,
    pub menu_visible_bottom: i32,
    pub menu_cursor: i32,
    /// f_byte_01（N-0240 menuReturnMode）：mode7/8 -7 的返回目标
    pub menu_return_mode: i32,
    /// mode 7 存档槽状态（m_000 case7 布局 + loadSlotTable 冷态）
    pub slot: crate::menu_family::SlotSelect,
    /// MOT_IF 槽表 blob（preset 虚拟 RMS；loadSlotTable 每次进 mode7 重读）
    pub preset_mot_if: Option<Vec<u8>>,
    /// mode 5 敌人图鉴：f_int_arr_22（预测伤害）/f_byte_arr_20（类型）
    pub encyc_damage: Vec<i32>,
    pub encyc_types: Vec<u8>,
    pub encyc_panel_w: i32,
    pub encyc_count: i32,
    pub encyc_rows_visible: i32,
    pub encyc_panel_h: i32,
    pub encyc_scroll_top: i32,
    pub encyc_scroll_bottom: i32,
    /// mode 16 设置面板（kind4 进入；menuReturnMode=4 返回）
    pub options: Option<crate::menu_family::OptionList>,
    /// soundEnabled（run case16 label0 翻转；无像素效应，RMS 进度侧状态）
    pub sound_enabled: bool,
    /// f_byte_arr_14/15 + f_int_89：剧情物清单（m_075 维护；序章不可达恒空）
    pub special_item_types: Vec<u8>,
    pub special_item_uses: Vec<u8>,
    /// 虚拟 RMS：MOT_L{slot} 槽位 blob（saveFloorState 写侧；game-core 无 I/O）
    pub saved_slot_blobs: [Option<Vec<u8>>; 6],
    /// f_byte_25：商店条目彩虹色相位（paint 每 4 拍推进）
    pub rainbow_phase: i32,
    /// iconStripFrame（a.java:464 全局；mode16 图标条动画帧）
    pub strip_frame: usize,
    /// f_byte_02：菜单返回目标（run case3 -7 时存 gameMode）
    pub in_game_menu_return_mode: i32,
    /// mode 10 道具面板布局（m_000 case10 a.java:4370-4380：f_int_97..104）
    pub item_cols: i32,
    pub item_rows_total: i32,
    pub item_rows_visible: i32,
    pub item_scroll_top: i32,
    pub item_scroll_bottom: i32,
    pub item_box_w: i32,
    pub item_box_h: i32,
    pub item_cursor: i32,
    /// f_bool_26：路径箭头可见（SEE flag=1 且 findPath 成功）
    pub path_arrows_visible: bool,
    /// walkPathBuffer（findPath 步序，目标端优先）
    pub walk_path_buffer: Vec<u8>,
    /// f_byte_07：最近拾取/装备的类型号（浮层图标 + 商店显示）
    pub overlay_kind_item: i32,
    /// 浮层图标图（kind1/3：entityTypeImage[f_byte_07] 的克隆——paint 免借用冲突）
    pub overlay_icon: Option<ArgbImage>,
    /// 浮层标题（objectTypeNames[f_byte_07]）
    pub overlay_item_name: Option<Vec<u16>>,
    /// objectTypeNames（89 项；夹具同源 a.java:954-1041）
    pub object_type_names: Vec<String>,
    /// 宽度表/可行走表（换层重载用）
    pub width_table: Vec<i32>,
    pub tile_walk: Vec<i32>,
    /// gameRandom（java.util.Random，a.java:2324 **全局单例**——标题雪花/
    /// 菜单/粒子/战斗抖动共用一条连续消费链）。
    /// ⚠️ `new_seeded(0)` 只是构造默认值，**不是** Java 语义：Java 从 boot 起
    /// 单实例连续消费，不因换层/读档重置。任何构造 GameScene 的调用方
    /// （测试 harness/未来桌面壳）必须把 machine 阶段已消费的 LCG 状态
    /// 续接进来（`sc.rng = machine.rng.clone()`），否则场景首个消费者
    /// （state==3 战斗抖动 a.java:6559-6560 randomBelow(5)）序列错位，
    /// ±2px 抖动与 Java 分歧（迭代 35 T801 实证）。
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

/// menuLabels（a.java:458-462）：18 条菜单标签。
pub const MENU_LABELS: [&str; 18] = [
    "新游戏", "继续游戏", "载入进度", "保存游戏", "设置", "帮助", "关于", "退出",
    "返回菜单", "回放", "停止回放", "商店", "十全大补包", "攻防神油", "开门天天乐",
    "天庭消费券", "印度神血油", "跳过教程",
];
/// f_int_arr_32（a.java:2229）：商店条目逐字彩虹色环（7 色随 f_byte_25 相位）。
pub const RAINBOW_COLORS: [u32; 7] = [
    16711680, 16748544, 16776194, 1244928, 65478, 26367, 14156031,
];

/// equipmentMaterialNames（a.java:484）：装备材质名（0=无）。
pub const EQUIPMENT_MATERIAL_NAMES: [&str; 9] =
    ["无", "木", "铁", "银", "金", "布", "皮", "锁", "金"];

/// GameScene 的存档载入构造（run case8 → mode2 链 → step16 loadFloorSave +
/// step10 loadFloorData/buildMinimap + step11 相机/朝向 + m_000 case3）。
#[allow(clippy::too_many_arguments)]
pub fn load_floor_from_save(
    maplv: &[u8],
    records: &[(u8, u16, u16, i16, bool)],
    width_table: &[i32],
    tile_walkability: &[i32],
    images: SceneImages,
    floor_assets: Vec<Option<FloorAssets>>,
    save: &crate::save::FloorSave,
) -> Result<GameScene, String> {
    let px = (save.player_cell_x as i32) << 5;
    let py = (save.player_cell_y as i32) << 5;
    let mut scene = GameScene::load_floor(
        maplv,
        px,
        py,
        records,
        width_table,
        tile_walkability,
        images,
        floor_assets,
    )?;
    // step 16 = loadFloorSave 字段应用（a.java:9257-9299）
    scene.floor = save.current_floor as i32;
    scene.current_floor = save.current_floor as i32;
    scene.min_floor = save.min_floor_reached as i32;
    scene.max_floor = save.max_floor_reached as i32;
    scene.facing = 0; // step 11（a.java:3521，覆盖 blob 朝向——Java 原样）
    scene.player.hp = save.player_hp;
    scene.player.atk = save.player_atk;
    scene.player.def = save.player_def;
    scene.player.yellow_keys = save.yellow_keys as i32;
    scene.player.blue_keys = save.blue_keys as i32;
    scene.player.red_keys = save.red_keys as i32;
    scene.player.gold = save.gold;
    scene.player.item_stack_size = save.item_stack_size as i32;
    scene.alchemy_upgrade_count = save.alchemy_upgrades as i32; // a.java:9281 链（loadFloorSave 应用）
    for k in 0..save.item_stack_size as usize {
        scene.player.item_types[k] = save.item_stack_types[k];
        scene.player.item_uses[k] = save.item_stack_uses[k];
    }
    scene.saved_floor_states = save.floor_states.clone();
    scene.script_line_flags = vec![false; 128];
    // step 10 = loadFloorData（应用本层触发缓存）+ buildMinimap（a.java:3515-3517）
    scene.load_floor_data(maplv, px, py, records)?;
    scene.build_minimap();
    // step 11 相机（a.java:3523-3525）+ m_000 case 3（a.java:4312-4318）
    scene.enter_view();
    scene.softkeys = (1, 3);
    scene.overlay_active = false;
    scene.apply_step_cell_effects();
    Ok(scene)
}


impl GameScene {
    /// loadFloorData（a.java:9404-9438）：maplv 装载 + m_059 格索引重建 +
    /// 可行走重建 + m_122 实体生成 + sortEntitiesByY 排序。
    /// `floor_assets`：按层号索引的换层资产（索引缺省 = 该层不可达）。
    #[allow(clippy::too_many_arguments)]
    /// 浮层框绘制（paint 公共尾 a.java:3129-3168）：盒框 + kind 分派内容 +
    /// 左右软键图标。mode 3/4/7/10/16/19 各 paint 共用。
    fn paint_overlay_box(&mut self, g: &mut SoftGraphics<'_>) {
        // 提示浮层（paint 公共尾 a.java:3129-3168，!f_bool_16 && overlayActive）：
        // 盒框 + kind 0/2/4/5 纯文本 / kind 1/3 图标+文本 + 软键图标
        if !self.transitioning && self.overlay_active {
            let bx = (SCREEN_W - self.overlay_box_w) >> 1;
            let by = (SCREEN_H - self.overlay_box_h) >> 1;
            crate::menu_family::paint_box_frame(g, &self.images.ui[0], bx, by, self.overlay_box_w, self.overlay_box_h);
            let font = crate::paint::paint_font();
            let line_h = font.height + 4;
            let mut tx = bx + 16;
            let mut ty = by + 16;
            if matches!(self.overlay_kind, 0 | 2 | 4 | 5) {
                g.set_color(WHITE);
                self.paint_wrapped_text(g, tx, ty, 180, 240, true);
            } else {
                paint_mini_frame(g, tx, ty, 32, 32);
                // 图标 = entityTypeImage[overlayKindItem]（f_byte_07——拾取时记录）
                if let Some(img) = &self.overlay_icon {
                    g.draw_image(img, tx + ((32 - img.width) >> 1), ty + ((32 - img.height) >> 1), 0);
                }
                tx += 42;
                g.set_color(16770173);
                if let Some(name) = &self.overlay_item_name {
                    g.draw_string(name, tx, ty + ((32 - font.height) >> 1), 0);
                }
                g.set_color(WHITE);
                self.paint_wrapped_text(g, bx + 16, ty + 36, 180, self.overlay_box_h - 32 - 36, true);
            }
            let iy = by + self.overlay_box_h - 12 - 18;
            if self.overlay_left_icon != 0 {
                draw_image_clipped(g, &self.images.ui[11], bx + 13, iy + 6, (self.overlay_left_icon as i32 - 1) * 12, 0, 12, 10);
            }
            if self.overlay_right_icon != 0 {
                draw_image_clipped(g, &self.images.ui[11], bx + self.overlay_box_w - 25, iy + 6, (self.overlay_right_icon as i32 - 1) * 12, 0, 12, 10);
            }
        }
    }

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
            saved_floor_states: vec![None; 56],
            object_type_names: {
                let raw = include_str!("../tests/fixtures/object_type_names.txt");
                raw.lines().map(|l| l.trim().to_string()).collect()
            },
            equipped_weapon: 0,
            equipped_armor: 0,
            equip_bonuses: [0, 10, 30, 70, 120, 220, 0, 10, 30, 70, 120, 220],
            equip_types: [0, 33, 34, 35, 79, 36, 0, 37, 38, 39, 80, 40],
            overlay_active: false,
            overlay_kind: 0,
            overlay_text: None,
            overlay_box_w: 0,
            overlay_box_h: 0,
            overlay_left_icon: 0,
            overlay_right_icon: 0,
            overlay_lines: Vec::new(),
            overlay_page_top: 0,
            overlay_page_bottom: 0,
            overlay_lines_per_page: 0,
            overlay_total_lines: 0,
            overlay_content_w: 0,
            open_group: Vec::new(),
            open_anim_tick: 0,
            tile_bob: 0,
            tutorial_seen: false,
            stair_hint: false,
            overlay_kind_item: 0,
            overlay_icon: None,
            overlay_item_name: None,
            game_mode: 3,
            dialog_phase: 0,
            dialog_page: 0,
            dialog_page_end: 0,
            dialog_speaker: 0,
            dialog_anchor: -1,
            dialog_char: 0,
            dialog_target: (0, 0),
            script_line: String::new(),
            script_cursor: 0,
            script_delay: 0,
            script_line_flags: vec![false; 68],
            current_script_index: 0,
            dialog_data: DialogData::from_fixtures(),
            difficulty_index: 0,
            script_walk_armed: false,
            overlay_saved_color: 0,
            menu_kinds: Vec::new(),
            menu_item_count: 0,
            menu_box_width: 0,
            menu_line_height: 0,
            menu_visible_rows: 0,
            menu_list_height: 0,
            menu_scroll_top: 0,
            menu_visible_bottom: 0,
            menu_cursor: 0,
            menu_return_mode: 1,
            slot: crate::menu_family::SlotSelect::new(),
            preset_mot_if: None,
            encyc_damage: Vec::new(),
            encyc_types: Vec::new(),
            encyc_panel_w: 208,
            alchemy_price: 0,
            alchemy_gain_hp: 0,
            alchemy_gain_atk: 0,
            alchemy_gain_def: 0,
            alchemy_cursor: 0,
            alchemy_upgrade_count: 0,
            merchant_entity: -1,
            offer_queue_src: Vec::new(),
            offer_queue_idx: Vec::new(),
            offer_queue_cursor: 0,
            encyc_count: 0,
            encyc_rows_visible: 0,
            encyc_panel_h: 0,
            encyc_scroll_top: 0,
            encyc_scroll_bottom: 0,
            options: None,
            sound_enabled: true,
            special_item_types: Vec::new(),
            special_item_uses: Vec::new(),
            saved_slot_blobs: [None, None, None, None, None, None],
            rainbow_phase: 0,
            strip_frame: 0,
            in_game_menu_return_mode: 3,
            item_cols: 4,
            item_rows_total: 8,
            item_rows_visible: 6,
            item_scroll_top: 0,
            item_scroll_bottom: 6,
            item_box_w: 202,
            item_box_h: 296,
            item_cursor: 0,
            path_arrows_visible: false,
            walk_path_buffer: Vec::new(),
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

    /// loadFloorData 的原地重载（换层遮幅中段 a.java:3189 调用同方法）：
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
                // rebuildWalkability（a.java:6964-6976）的索引：**行距 wide·4、
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
        // m_122 双分支（a.java:9465+）：有层存档 → 从存档恢复（**死怪不复活**、
        // 已开门保持开）；无存档 → sprite{n} 权威数据 + 5/81/6/12/76/82/83 隐藏
        if let Some(data) = self.saved_floor_states[self.floor as usize].clone() {
            let mut pos = 0usize;
            let mut read_u16 = |d: &[u8], p: &mut usize| -> u16 {
                let v = u16::from_be_bytes([d[*p], d[*p + 1]]);
                *p += 2;
                v
            };
            let count = read_u16(&data, &mut pos) as usize;
            for _ in 0..count {
                let t = data[pos] as i32;
                pos += 1;
                let x = read_u16(&data, &mut pos) as i32;
                let y = read_u16(&data, &mut pos) as i32;
                let visible = data[pos] != 0;
                pos += 1;
                let param = read_u16(&data, &mut pos) as i16;
                let img_h = self.images.entity.get(t as usize).cloned().flatten().map(|im| im.height);
                let idx = entity::spawn(&mut entities, &mut grid, t, x, y, param, &self.width_table, img_h);
                entities.visible[idx] = visible;
                entities.removed[idx] = false; // m_122 else：恢复的实体 removed=false
            }
            // floor2 重入特例（a.java:9645-9644）：scriptLineFlags[26] 未追踪
            //（脚本批）——本窗口无脚本运行，按 false 处理
            if self.floor == 2 && entity::cell_entity(&grid, &entities, 11, 4, 84) < 0 {
                let img_h = self.images.entity[84].as_ref().map(|i| i.height);
                entity::spawn(&mut entities, &mut grid, 84, 352, 128, 0, &self.width_table, img_h);
            }
        } else {
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
        }
        entity::sort_by_y(&mut entities, &mut grid);
        // m_122 尾（a.java:9669-9667）：type 72 类别 floor50=8、其余 32——
        // render_category 静态表只覆盖非 50 层（本批范围 1/2）
        self.view = view;
        self.entities = entities;
        self.grid = grid;
        self.walkable = walkable;
        self.minimap = None;
        Ok(())
    }

    /// changeFloor（a.java:7126-7164）：层号合法性（strict=键 49/55 路径；
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
        // m_119(currentFloor)（a.java:9370-9388）：序列化**存活**实体
        // （removed/state==1 跳过）——死怪/已开门跨层持久
        self.save_floor_state();
        self.transitioning = true;
        // a.java 台账实证（迭代 46，A-preset-item21 T402）：changeFloor 起
        // 遮幅即回 mode 3（行走世界 + 遮幅点阵渐显）——面板内道具触发路径
        // 的 gameMode 不再停留 mode 10
        self.game_mode = 3;
        if target < self.min_floor {
            self.min_floor = target;
        } else if target > self.max_floor {
            self.max_floor = target;
            // f_int_154 全局进度（RMS）不追踪
        }
        self.pending_floor = target;
        self.went_down = down;
        true
    }

    /// m_119（a.java:9370-9388）：层状态序列化。格式：u16 计数（BE，m_120）+
    /// 每存活实体 [type u8, x u16 BE, y u16 BE, visible u8, param u16 BE]。
    fn save_floor_state(&mut self) {
        let mut body: Vec<u8> = Vec::new();
        let mut live = 0u16;
        for e in 0..self.entities.count {
            if !self.entities.removed[e] && self.entities.solid[e] != 1 {
                live += 1;
                body.push(self.entities.entity_type[e] as u8);
                body.extend_from_slice(&(self.entities.pixel_x[e] as u16).to_be_bytes());
                body.extend_from_slice(&(self.entities.pixel_y[e] as u16).to_be_bytes());
                body.push(self.entities.visible[e] as u8);
                body.extend_from_slice(&(self.entities.param[e] as u16).to_be_bytes());
            }
        }
        let mut data = Vec::with_capacity(body.len() + 2);
        data.extend_from_slice(&live.to_be_bytes());
        data.extend_from_slice(&body);
        self.saved_floor_states[self.floor as usize] = Some(data);
    }

    /// 遮幅中段换层（a.java:3186-3206）：loadFloorData + 落点特例
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

    /// findFloorGateEntity（a.java:7106-7118）：up=true 找型 7、false 找型 8；
/// 换层落点直传 f_bool_18（false=上楼抵达→机型 8 下楼梯），
    /// param>>9==0（低 9 位无高字节——目标层 ≤255 的常规门）。
    /// findFloorGateEntity（a.java:7106-7122）：首个 param>>9==0 的型7/8
    /// 门格实体下标（暗道换乘目标——param 低 8 位 = 目标层）。
    fn find_floor_gate_entity(&self, up: bool) -> Option<usize> {
        let want = if up { 7 } else { 8 };
        for e in 0..self.entities.count {
            if self.entities.entity_type[e] == want && (self.entities.param[e] >> 9) == 0 {
                return Some(e);
            }
        }
        None
    }

    fn find_floor_gate(&self, up: bool) -> Option<(i32, i32)> {
        let want = if up { 7 } else { 8 };
        for e in 0..self.entities.count {
            if self.entities.entity_type[e] == want && (self.entities.param[e] >> 9) == 0 {
                return Some((self.entities.pixel_x[e] >> 5, self.entities.pixel_y[e] >> 5));
            }
        }
        None
    }

    /// m_031（a.java:5807-5814）：楼梯落点 = 首个可行走邻格（上/下/左/右序）。
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

    /// m_024（a.java:5193-5193）：格坐标直置 + 步进度清零。
    fn m_024(&mut self, cx: i32, cy: i32) {
        self.player_cell_x = cx;
        self.player_cell_y = cy;
        self.view.player_px = cx << 5;
        self.view.player_py = cy << 5;
        self.step_progress = 0;
    }

    /// m_050 removeSlotAt（a.java:6435-6441）：槽位摘除 + 前移。
    pub fn remove_slot_at(&mut self, cx: i32, cy: i32, slot: i32) {
        let ct = self.grid.cell_type[cy as usize][cx as usize] as usize;
        let cap = self.grid.capacity[ct] as usize;
        if cap == 0 {
            return;
        }
        let idx = self.grid.slots[ct][slot as usize] as usize - 1;
        self.entities.removed[idx] = true;
        for k in slot as usize..cap - 1 {
            self.grid.slots[ct][k] = self.grid.slots[ct][k + 1];
        }
        self.grid.capacity[ct] -= 1;
    }

    /// m_073 tryOpenChest（a.java:7285-7299）：全图收集同 param 型 4（未移除、
    /// 非 5）；open_group 非空 = 有锁可开（walkPhase 4 动画）。
    pub fn try_open_chest(&mut self, chest_param: i16) -> bool {
        self.open_group.clear();
        for k in 0..self.entities.count {
            if !self.entities.removed[k] && chest_param == self.entities.param[k] {
                let tk = self.entities.entity_type[k];
                if tk == 5 {
                    return false; // 未开的另一宝箱挡路
                }
                if tk == 4 {
                    self.open_group.push(k);
                }
            }
        }
        !self.open_group.is_empty()
    }

    /// showOverlayMessage（a.java:4898-4916）：kind 0/2/4/5 = 纯文本框；
    /// 1/3 = 带图标框（定宽 212、高 68+页高）。
    pub fn show_overlay_message(&mut self, kind: i32, text: &str, left: i8, right: i8) {
        self.overlay_active = true;
        self.overlay_kind = kind;
        self.overlay_text = Some(text.to_string());
        if matches!(kind, 1 | 3) {
            let t = self.overlay_kind_item;
            self.overlay_icon = self.images.entity.get(t as usize).cloned().flatten();
            self.overlay_item_name = self.object_type_name(t);
        }
        let font = crate::paint::paint_font();
        let line_h = font.height + 4;
        if matches!(kind, 0 | 2 | 4 | 5) {
            self.layout_full_text(text, 180, 240);
            self.overlay_box_w = self.overlay_content_w + 32;
            self.overlay_box_h = 32 + (self.overlay_page_bottom - self.overlay_page_top) * line_h;
        } else {
            self.layout_full_text(text, 180, 234);
            self.overlay_box_w = 212;
            self.overlay_box_h = 68 + (self.overlay_page_bottom - self.overlay_page_top) * line_h;
        }
        self.overlay_left_icon = left;
        self.overlay_right_icon = right;
        if left != 0 || right != 0 {
            self.overlay_box_h += 18;
        }
    }

    /// layoutFullText → layoutWrappedText（a.java:4940-5036）：逐字换行
    ///（'\\c' 六位色 / '\\r' 转义宽度跳过），页高 = maxLines/page。
    pub fn layout_full_text(&mut self, text: &str, w: i32, max_h: i32) {
        let font = crate::paint::paint_font();
        let line_h = font.height + 4;
        let chars: Vec<char> = text.chars().collect();
        let mut lines: Vec<String> = Vec::new();
        let mut cur = String::new();
        let mut acc_w = 0i32;
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '\n' {
                lines.push(std::mem::take(&mut cur));
                acc_w = 0;
                i += 1;
                continue;
            }
            let cw;
            if c == '\\' && i + 1 < chars.len() {
                let nxt = chars[i + 1];
                if nxt == 'c' {
                    // \cXXXXXX：Java 条件 var12+6<len 且 parseHex 有效（a.java:4967）——
                    // 有效 ⇒ 整段 8 字符吞入行、宽 0（var11+=7+底部++=8，var6=0）
                    let hex: String = chars[i + 2..(i + 8).min(chars.len())].iter().collect();
                    let valid = i + 7 < chars.len()
                        && crate::menu_family::parse_hex_color_public(&hex).is_some();
                    if valid {
                        cur.push(c);
                        for k in 1..=7 {
                            cur.push(chars[i + k]);
                        }
                        i += 8;
                        continue;
                    } else {
                        // 非法色：\c 两字符按正常宽度计入（a.java:4969-4971 var6=两宽）
                        cw = font.char_width('\\' as u16) + font.char_width(nxt as u16);
                        cur.push(c);
                        cur.push(nxt);
                        acc_w += cw;
                        i += 2;
                        continue;
                    }
                } else if nxt == 'r' {
                    // \r：2 字符吞入、宽 0（a.java:4972-4975）
                    cur.push(c);
                    cur.push(nxt);
                    i += 2;
                    continue;
                }
                // 未知转义：反斜杠计宽，命令字符下轮当正文（近似；HELP_TEXT 不触发）
                cw = font.char_width('\\' as u16);
                cur.push(c);
                acc_w += cw;
                i += 1;
                continue;
            } else {
                cw = font.char_width(c as u16);
            }
            if acc_w + cw > w {
                lines.push(std::mem::take(&mut cur));
                cur.push(c);
                acc_w = cw;
            } else {
                cur.push(c);
                acc_w += cw;
            }
            i += 1;
        }
        // java：仅当末行有字符时 push（limit=0 → 0 行，a.java:5010-5014）
        if !cur.is_empty() {
            lines.push(cur);
        }
        self.overlay_page_top = 0;
        self.overlay_total_lines = lines.len() as i32;
        self.overlay_lines_per_page = (max_h / line_h).min(self.overlay_total_lines);
        self.overlay_page_bottom = if self.overlay_total_lines > self.overlay_lines_per_page {
            self.overlay_lines_per_page
        } else {
            self.overlay_total_lines
        };
        self.overlay_content_w = if self.overlay_total_lines == 1 {
            font.string_width(&text.encode_utf16().collect::<Vec<u16>>())
        } else {
            w
        };
        self.overlay_lines = lines;
    }

    /// tryEquip（a.java:7668-7680）：武器（<6）/甲（≥6）分档；降级拒绝
    ///（!force 且当前档 >= 新档 → false）；属性按 equipTierBonuses 增减。
    pub fn try_equip(&mut self, tier: i32, force: bool) -> bool {
        if tier < 6 {
            let cur = self.m_083_index(self.equipped_weapon, 0, 6);
            if cur >= tier && !force {
                return false;
            }
            self.player.atk -= self.equip_bonuses[cur as usize];
            self.player.atk += self.equip_bonuses[tier as usize];
            self.equipped_weapon = self.equip_types[tier as usize];
        } else {
            let cur = self.m_083_index(self.equipped_armor, 6, 12);
            if cur >= tier && !force {
                return false;
            }
            self.player.def -= self.equip_bonuses[cur as usize];
            self.player.def += self.equip_bonuses[tier as usize];
            self.equipped_armor = self.equip_types[tier as usize];
        }
        true
    }

    /// m_083（a.java:7688-7694）：档位区间倒序查当前装备所在档。
    fn m_083_index(&self, equipped: i32, lo: i32, hi: i32) -> i32 {
        let mut t = hi;
        while t >= lo {
            if self.equip_types[t as usize] == equipped {
                return t;
            }
            t -= 1;
        }
        0
    }

    /// objectTypeNames[type]（UTF-16 供 draw_string）。
    fn object_type_name(&self, t: i32) -> Option<Vec<u16>> {
        self.object_type_names
            .get(t as usize)
            .filter(|s| !s.is_empty())
            .map(|s| s.encode_utf16().collect())
    }

    /// paintWrappedText（a.java:5044-5121）：整行按 \\c 色段 flush（drawSubstring
    /// 语义 = 片段 drawString）+ 跨页起始色继承 + \\r 恢复 + 翻页三角。
    /// 忠实点：段起始 var12/游标 var16 的推进序、非法 hex 从反斜杠重画、
    /// 未知转义丢反斜杠、var15 每行清 0 的 `!= -1` 怪癖。
    fn paint_wrapped_text(&mut self, g: &mut SoftGraphics<'_>, x: i32, y: i32, w: i32, max_h: i32, arrows: bool) {
        let font = crate::paint::paint_font();
        let line_h = font.height + 4;
        // 向上找最近 \\c 起始色（a.java:5064-5074；解析失败 setColor(-1)=白）
        for i in (0..self.overlay_page_top as usize).rev() {
            if let Some(line) = self.overlay_lines.get(i) {
                if let Some(pos) = line.rfind('\\') {
                    if line[pos + 1..].starts_with('c') {
                        let hex = &line[(pos + 2).min(line.len())..(pos + 8).min(line.len())];
                        let c = crate::menu_family::parse_hex_color_public(hex)
                            .map(|v| v as i32)
                            .unwrap_or(-1);
                        g.set_color(c as u32);
                        break;
                    }
                }
            }
        }
        let mut ty = y + 2;
        for li in self.overlay_page_top as usize..self.overlay_page_bottom as usize {
            if let Some(line) = self.overlay_lines.get(li).cloned() {
                let codes: Vec<u16> = line.encode_utf16().collect();
                let n = codes.len() as i32;
                let mut seg_start = 0i32; // var12
                let mut cx = x; // var13
                let mut var15 = 0i32; // 每行清 0（a.java:5053）
                let mut i = 0i32;
                while i < n {
                    if codes[i as usize] == '\\' as u16 {
                        if i > 0 {
                            // flush [seg_start, i)（Java drawSubstring(var25,var12,var16-var12)）
                            let len = i - seg_start;
                            let seg: Vec<u16> =
                                codes[seg_start as usize..i as usize].to_vec();
                            g.draw_substring(&codes, seg_start, len, cx, ty, 0);
                            cx += seg.iter().map(|c| font.char_width(*c)).sum::<i32>();
                        }
                        i += 1;
                        if i < n {
                            let cmd = codes[i as usize];
                            if cmd == 'c' as u16 {
                                if i + 6 < n {
                                    let hex: String = codes[i as usize + 1..i as usize + 7]
                                        .iter()
                                        .map(|c| *c as u8 as char)
                                        .collect();
                                    var15 = crate::menu_family::parse_hex_color_public(&hex)
                                        .map(|v| v as i32)
                                        .unwrap_or(-1);
                                }
                                if var15 != -1 {
                                    self.overlay_saved_color = g.get_color() as i32;
                                    g.set_color(var15 as u32);
                                    i += 6;
                                    seg_start = i + 1;
                                } else {
                                    seg_start = i - 1; // 非法 hex：从反斜杠起当文本
                                }
                            } else if cmd == 'r' as u16 {
                                g.set_color(self.overlay_saved_color as u32);
                                seg_start = i + 1;
                            } else {
                                seg_start = i; // 未知转义：丢反斜杠画命令字符
                            }
                        }
                    }
                    i += 1;
                }
                if seg_start != 0 {
                    if seg_start < n {
                        g.draw_substring(&codes, seg_start, n - seg_start, cx, ty, 0);
                    }
                } else {
                    g.draw_string(&codes, x, ty, 0);
                }
            }
            ty += line_h;
        }
        // 翻页三角（a.java:5110-5119）
        if !arrows {
            return;
        }
        g.set_color(WHITE);
        let ax = x + (w >> 1);
        if self.overlay_page_top > 0 {
            let ay = y - 8 + ((self.frame_counter & 1) as i32);
            g.fill_triangle(ax, ay, ax - 7, ay + 7, ax + 7, ay + 7);
        }
        if self.overlay_page_bottom < self.overlay_total_lines {
            let ay = y + self.overlay_lines_per_page * line_h + 3 - ((self.frame_counter & 1) as i32);
            g.fill_triangle(ax, ay, ax - 6, ay - 6, ax + 6, ay - 6);
        }
    }

    /// tryRunScene（a.java:8065-8139）：脚本行门控（scriptLineFlags + 场景参数
    /// var2=fromWalk）。case 40 落 5..40 直通组（var3 原样 = !flags[40]）。
    pub fn try_run_scene(&mut self, idx: i32, from_walk: bool) -> bool {
        if idx >= self.dialog_data.scripts.len() as i32 {
            return false;
        }
        let mut gate = !self.script_line_flags[idx as usize];
        match idx {
            0 => {
                if gate {
                    self.script_line_flags[19] = true;
                }
            }
            1 => gate &= self.script_line_flags[0] && !from_walk,
            2 => gate &= self.script_line_flags[1] && !from_walk,
            3 => gate &= from_walk,
            4 => gate &= self.script_line_flags[3] && from_walk,
            // 5/6/10/13/14/17/20/21/22/23/28/29/35/36/37/38/39/40：直通（无门控）
            46 => gate &= from_walk && !self.script_line_flags[47],
            47 => gate &= from_walk && !self.script_line_flags[46],
            66 => gate = false,
            67 => gate &= self.difficulty_index > 0,
            _ => {}
        }
        gate
    }

    /// loadLevelScript（a.java:8035-8052）：mode 11 + 脚本行装载。
    /// case 31 层重排特例（a.java:8028-8028）未达（script 40 不涉），todo 守卫。
    pub fn load_level_script(&mut self, idx: i32) {
        self.script_walk_armed = false;
        self.game_mode = 11;
        self.dialog_page = 0;
        self.lockCameraOn(0); // ease 相机初始化到玩家
        self.dialog_phase = 0;
        self.current_script_index = idx as usize;
        self.script_line = self.dialog_data.scripts[idx as usize].clone();
        self.overlay_text = None;
        self.script_cursor = 0;
        // scriptLineFlags[idx] 在行尾收尾（AwaitCamera）或 GUT/DES 时置位——
        // loadLevelScript 本身不置（a.java:8035-8052 无 flag 写）
    }

    /// parseScriptInt（a.java:8583-8584）：from 起找 delim，返回值与终位。
    fn parse_script_int(line: &str, from: usize, delim: char) -> (i32, usize) {
        let rel = line[from..].find(delim).expect("脚本整数分隔符缺失");
        let end = from + rel;
        (line[from..end].parse().expect("脚本整数解析失败"), end)
    }

    /// openDialogPage（a.java:8208-8227）：页装载 + 说话者/锚 + 预排版。
    /// 布局走 layout_wrapped_text_limited（w=129、高=2·lineH+12、limit=typewriterPos）。
    fn open_dialog_page(&mut self, page: i32) {
        if page <= self.dialog_page_end {
            self.dialog_page = page;
            self.dialog_speaker = if page < 0 {
                -1
            } else {
                self.dialog_data.speakers[page as usize] as i32
            };
            self.dialog_anchor = {
                // lockCameraOn(speaker)：相机锚初始化 + 返回目标实体
                self.lockCameraOn(self.dialog_speaker);
                if self.dialog_speaker != 0 && self.dialog_speaker != 87 {
                    let mut found = -1;
                    for e in (0..self.entities.count).rev() {
                        if self.entities.entity_type[e] == self.dialog_speaker {
                            found = e as i32;
                            break;
                        }
                    }
                    if found >= 0 {
                        self.ease_target_x = self.entities.pixel_x[found as usize];
                        self.ease_target_y = self.entities.pixel_y[found as usize];
                    }
                    found
                } else {
                    self.ease_target_x = self.view.player_px;
                    self.ease_target_y = self.view.player_py;
                    -1
                }
            };
            let composed = if self.dialog_speaker < 0 {
                format!("?: \\cF8F8F8{}", self.dialog_data.texts[page as usize])
            } else {
                let name = &self.object_type_names[self.dialog_speaker as usize];
                format!("{}: \\cF8F8F8{}", name, self.dialog_data.texts[page as usize])
            };
            self.dialog_char = if self.dialog_speaker < 0 {
                11
            } else {
                (self.object_type_names[self.dialog_speaker as usize].chars().count() + 10) as i32
            };
            self.overlay_text = Some(composed.clone());
            let line_h = crate::paint::paint_font().height + 4;
            self.layout_wrapped_text_limited(&composed, 129, (line_h << 1) + 12, self.dialog_char);
        } else {
            self.dialog_phase = 0;
            self.dialog_char = 0;
        }
    }

    /// layoutWrappedText 带**字符上限**（Java 第 4 参 = 只排版前 limit 个字符——
    /// 打字机逐步放行的机制）。layout_full_text 是 limit=全长 的特例。
    fn layout_wrapped_text_limited(&mut self, text: &str, w: i32, max_h: i32, limit: i32) {
        let font = crate::paint::paint_font();
        let line_h = font.height + 4;
        let chars: Vec<char> = text.chars().take(limit as usize).collect();
        let mut lines: Vec<String> = Vec::new();
        let mut cur = String::new();
        let mut acc_w = 0i32;
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '\n' {
                lines.push(std::mem::take(&mut cur));
                acc_w = 0;
                i += 1;
                continue;
            }
            // 转义语义与 layout_full_text 同源（a.java:4962-4976）——迭代 26 修的是
            // overlay 全量版，本打字机限量版漏改致同一处偏差复发（迭代 30 教训：
            // 同一算法的两份拷贝必须同步修，或抽公共函数）。
            let cw;
            if c == '\\' && i + 1 < chars.len() {
                let nxt = chars[i + 1];
                if nxt == 'c' {
                    let hex: String = chars[i + 2..(i + 8).min(chars.len())].iter().collect();
                    let valid = i + 7 < chars.len()
                        && crate::menu_family::parse_hex_color_public(&hex).is_some();
                    if valid {
                        cur.push(c);
                        for k in 1..=7 {
                            cur.push(chars[i + k]);
                        }
                        i += 8;
                        continue;
                    } else {
                        cw = font.char_width('\\' as u16) + font.char_width(nxt as u16);
                        cur.push(c);
                        cur.push(nxt);
                        acc_w += cw;
                        i += 2;
                        continue;
                    }
                } else if nxt == 'r' {
                    cur.push(c);
                    cur.push(nxt);
                    i += 2;
                    continue;
                }
                cw = font.char_width('\\' as u16);
                cur.push(c);
                acc_w += cw;
                i += 1;
                continue;
            } else {
                cw = font.char_width(c as u16);
            }
            if acc_w + cw > w {
                lines.push(std::mem::take(&mut cur));
                cur.push(c);
                acc_w = cw;
            } else {
                cur.push(c);
                acc_w += cw;
            }
            i += 1;
        }
        // java：仅当末行有字符时 push（limit=0 → 0 行，a.java:5010-5014）
        if !cur.is_empty() {
            lines.push(cur);
        }
        self.overlay_page_top = 0;
        self.overlay_total_lines = lines.len() as i32;
        self.overlay_lines_per_page = (max_h / line_h).min(self.overlay_total_lines);
        self.overlay_page_bottom = self.overlay_total_lines.min(self.overlay_lines_per_page);
        self.overlay_content_w = if self.overlay_total_lines == 1 {
            font.string_width(&text.chars().take(limit as usize).map(|c| c as u16).collect::<Vec<u16>>())
        } else {
            w
        };
        self.overlay_lines = lines;
    }

    /// flipOverlayPage（a.java:5129-5140）：页顶/页底钳制。
    fn flip_overlay_page(&mut self, page: i32) {
        let mut top = page;
        if top >= 0 {
            if top >= self.overlay_total_lines - self.overlay_lines_per_page + 1 {
                top = self.overlay_total_lines - self.overlay_lines_per_page;
            }
        } else {
            top = 0;
        }
        self.overlay_page_top = top;
        self.overlay_page_bottom = top + self.overlay_lines_per_page;
        if self.overlay_page_top < 0 {
            self.overlay_page_top = 0;
        }
        if self.overlay_page_bottom > self.overlay_total_lines {
            self.overlay_page_bottom = self.overlay_total_lines;
        }
    }

    /// skipTextToLineEnd（a.java:5135-5148）：打字机跳到当前行尾（累计长度）。
    fn skip_text_to_line_end(&self, mut pos: i32) -> i32 {
        let mut cum = 0i32;
        for line in &self.overlay_lines {
            cum += line.chars().count() as i32;
            if pos >= cum - line.chars().count() as i32 && pos < cum {
                return cum;
            }
        }
        cum
    }

    /// executeScriptInstruction 的 script 40 可达子集：TAK/SEE/END/SMS。
    /// 其余 opcode（MOV/DES/CES/GUT/LAY/ROS/RES/ADD/MVS）在对应脚本对拍批实现。
    fn execute_script_instruction(&mut self) {
        let line = self.script_line.clone();
        let mut var2 = self.script_cursor;
        if var2 + 3 > line.len() {
            // 行尾收尾（a.java:8574-8580）：phase=4 等相机 + 置 flag + 相机回玩家
            self.dialog_phase = 4;
            let cur = self.current_script_index;
            if cur != 32 {
                self.script_line_flags[cur] = true;
            }
            self.lockCameraOn(0);
            return;
        }
        let op = &line[var2..var2 + 3];
        var2 += 3;
        if op == "TAK" {
            let (start, e1) = Self::parse_script_int(&line, var2 + 1, '_');
            let (end, e2) = Self::parse_script_int(&line, e1 + 1, ' ');
            self.dialog_page = start;
            self.dialog_page_end = end;
            self.script_cursor = e2 + 1; // f_int_122 + 1（a.java:8263）
            self.dialog_phase = 1;
            self.open_dialog_page(start);
        } else if op == "SEE" {
            let (tx, a) = Self::parse_script_int(&line, var2 + 1, '_');
            let (ty, b) = Self::parse_script_int(&line, a + 1, '_');
            let (pg, c) = Self::parse_script_int(&line, b + 1, '_');
            let (pg2, d) = Self::parse_script_int(&line, c + 1, '_');
            let (flag, e) = Self::parse_script_int(&line, d + 1, ' ');
            self.dialog_target = (tx, ty);
            // Java 先把 pageStart/pageEnd 解析入字段再 openDialogPage
            //（a.java:8487-8495）：pageEnd 不先更新会误判“越界”而丢页
            self.dialog_page = pg;
            self.dialog_page_end = pg2;
            self.script_cursor = e + 1; // f_int_122 + 1（a.java:8493，同 TAK 口径）
            self.open_dialog_page(pg);
            if flag == 1 {
                // findPath(player → target)：成功 → f_bool_26=true + walkPhase=0
                //（a.java:8497-8501；Java switch case1 **无 break 落穿 default**：
                //  路径成立时 m_103 与路径箭头两者都做）
                let walk = crate::pathfind::Walkability {
                    terrain: self.walkable.clone(),
                    has_entity_slot: &|x, y| {
                        let ct = self.grid.cell_type[y as usize][x as usize] as usize;
                        self.grid.capacity[ct] > 0
                    },
                    cols: self.view.wide,
                    rows: self.view.high,
                };
                let result = crate::pathfind::find_path(
                    &walk,
                    self.player_cell_x,
                    self.player_cell_y,
                    tx,
                    ty,
                );
                if result.found {
                    self.path_arrows_visible = true;
                    self.walk_phase = 0;
                    self.walk_path_buffer = result.steps;
                    self.walk_step_count = self.walk_path_buffer.len() as i32;
                }
            }
            self.dialog_phase = 6;
            // m_103：ease 相机目标 = 目标格
            self.ease_cam_x = (self.view.view_w - 32 >> 1) - self.view.cam_x;
            self.ease_cam_y = (self.view.view_h - 32 >> 1) - self.view.cam_y;
            self.ease_target_x = tx << 5;
            self.ease_target_y = ty << 5;
        } else if op == "END" {
            let (_v, e) = Self::parse_script_int(&line, var2 + 1, ' ');
            self.game_mode = 20;
            self.script_cursor = e + 1;
        } else if op == "SMS" {
            let (_v, e) = Self::parse_script_int(&line, var2 + 1, ' ');
            self.script_cursor = e + 1;
        } else if op == "ROS" {
            // ROS_sub（a.java:8359-8381）：子码直写玩家态。
            // 1=玩家格坐标(px<<5/py<<5)+锁相机回玩家；2=强制装备 tryEquip(t,true)；
            // 3/5/6=HP/ATK/DEF；4=朝向（Java byte 字段 ⇒ i8 截断保真）。
            // Java 收尾 scriptCursor = f_int_122 + 1：f_int_122 是**最后一次**
            // parseScriptInt 的终位——每臂都必须把终位写回 v2（含 case 1 的
            // y 解析与 2-6 的单解析），否则游标停在参数中间（实测 ROS_4_1
            // 后 cursor=37 落进 "1 S"）。
            let (sub, mut v2) = Self::parse_script_int(&line, var2 + 1, '_');
            match sub {
                1 => {
                    let (px, e) = Self::parse_script_int(&line, v2 + 1, '_');
                    v2 = e;
                    self.view.player_px = px << 5;
                    let (py, e) = Self::parse_script_int(&line, v2 + 1, ' ');
                    v2 = e;
                    self.view.player_py = py << 5;
                    self.lockCameraOn(0);
                }
                2 => {
                    let (tier, e) = Self::parse_script_int(&line, v2 + 1, ' ');
                    v2 = e;
                    self.try_equip(tier, true);
                }
                3 => {
                    let (v, e) = Self::parse_script_int(&line, v2 + 1, ' ');
                    v2 = e;
                    self.player.hp = v;
                }
                4 => {
                    let (v, e) = Self::parse_script_int(&line, v2 + 1, ' ');
                    v2 = e;
                    self.facing = (v as i8) as i32;
                }
                5 => {
                    let (v, e) = Self::parse_script_int(&line, v2 + 1, ' ');
                    v2 = e;
                    self.player.atk = v;
                }
                6 => {
                    let (v, e) = Self::parse_script_int(&line, v2 + 1, ' ');
                    v2 = e;
                    self.player.def = v;
                }
                _ => {}
            }
            self.script_cursor = v2 + 1;
        } else {
            todo!("executeScriptInstruction 的 {op:?} @op_start={} entry_cursor={} line={line:?}（对应脚本对拍批实现）", var2 - 3, self.script_cursor)
        }
    }

    fn walkable_at(&self, x: i32, y: i32) -> bool {
        x >= 0
            && y >= 0
            && x < self.view.wide
            && y < self.view.high
            && self.walkable[y as usize][x as usize]
    }

    /// 加载步 11（a.java:3510-3514）：进游戏视图 + 居中相机。
    pub fn enter_view(&mut self) {
        self.view.center_on_player();
    }

    /// buildMinimap（a.java:6879-6941）：小地图生成
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
                                // case 2/3 各自显式 break（a.java:6910-6917，无落穿）
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

    /// run case 11 相位机（a.java:4011-4068）。m_055 由调用方公共段跑。
    fn tick_dialog(&mut self) {
        self.step_camera_toward_target();
        match self.dialog_phase {
            0 => {
                if self.script_delay > 0 {
                    self.script_delay -= 1;
                } else {
                    self.execute_script_instruction();
                }
            }
            1 | 6 => {
                if self.dialog_phase != 6 || self.camera_at_target() {
                    let text = self.overlay_text.clone().unwrap_or_default();
                    let text_len = text.chars().count() as i32;
                    let line_h = crate::paint::paint_font().height + 4;
                    if self.dialog_char < text_len {
                        if self.key_value != 0 {
                            // 全量 layout + 跳到行尾（a.java:4029-4026）
                            self.layout_full_text(&text, 129, (line_h << 1) + 12);
                            self.dialog_char = self.skip_text_to_line_end(self.dialog_char);
                        } else if (self.frame_counter & 1) == 1 {
                            self.dialog_char += 1;
                            // 转义跳宽（charAt(typewriterPos-1)=='\\'）
                            let chars: Vec<char> = text.chars().collect();
                            let idx = self.dialog_char as usize - 1;
                            if idx < chars.len() && chars[idx] == '\\' {
                                match chars.get(idx + 1).copied().unwrap_or(' ') {
                                    'c' => self.dialog_char += 8,
                                    'r' => self.dialog_char += 2,
                                    _ => {}
                                }
                            }
                        }
                        self.layout_wrapped_text_limited(&text, 129, (line_h << 1) + 12, self.dialog_char);
                        if self.overlay_total_lines > self.overlay_page_bottom {
                            self.flip_overlay_page(self.overlay_total_lines - self.overlay_lines_per_page);
                        }
                    } else {
                        match self.key_value {
                            -2 => self.flip_overlay_page(self.overlay_page_top + self.overlay_lines_per_page),
                            -1 => self.flip_overlay_page(self.overlay_page_top - self.overlay_lines_per_page),
                            0 => {}
                            _ => {
                                self.dialog_char = 1;
                                self.open_dialog_page(self.dialog_page + 1);
                                if self.dialog_phase == 6 {
                                    // m_103：ease 目标 = SEE 目标格
                                    self.ease_cam_x = (self.view.view_w - 32 >> 1) - self.view.cam_x;
                                    self.ease_cam_y = (self.view.view_h - 32 >> 1) - self.view.cam_y;
                                    self.ease_target_x = self.dialog_target.0 << 5;
                                    self.ease_target_y = self.dialog_target.1 << 5;
                                }
                            }
                        }
                    }
                }
            }
            2 => todo!("m_106 实体路径缓动（MOV 对拍批）"),
            3 => todo!("script walk 相位（脚本行走对拍批）"),
            4 => {
                if self.camera_at_target() {
                    self.game_mode = 3;
                }
            }
            5 => {} // f_bool_16 组合（LAY 批）
            _ => {}
        }
    }

    /// resetMenuLayout（a.java:4690-4702）。
    fn reset_menu_layout(&mut self) {
        let font = crate::paint::paint_font();
        self.menu_line_height = font.height + 6;
        self.menu_visible_rows = 238 / self.menu_line_height;
        self.menu_list_height = self.menu_line_height * self.menu_visible_rows;
        self.menu_box_width = font.char_width('回' as u16) << 1; // wideCharWidth（全宽字符宽）
        self.menu_item_count = 0;
        self.menu_kinds.clear();
        self.menu_scroll_top = 0;
    }

    /// addMenuItem（a.java:4704-4716）：kind 登记 + 框宽按 stringWidth+60 伸长。
    fn add_menu_item(&mut self, kind: i8) {
        if self.menu_item_count < 16 {
            let font = crate::paint::paint_font();
            let label = MENU_LABELS[kind as usize];
            let w: i32 = label.chars().map(|c| font.char_width(c as u16)).sum();
            let w = w + 60;
            self.menu_kinds.push(kind);
            if w > self.menu_box_width {
                self.menu_box_width = w;
            }
            self.menu_item_count += 1;
            if self.menu_item_count <= self.menu_visible_rows {
                self.menu_visible_bottom = self.menu_item_count;
            }
            // menuTotalHeight += lineHeight；listHeight 按可见区间重算
            //（a.java:4712-4713——非 resetMenuLayout 的行数积）
            self.menu_list_height =
                (self.menu_visible_bottom - self.menu_scroll_top) * self.menu_line_height;
        }
    }

    /// m_000 case 4（a.java:4326-4337）：继续游戏（+floor>50 的跳过教程）/
    /// 保存/设置/帮助/退出；软键 (1,3)。
    fn enter_menu4(&mut self) {
        self.reset_menu_layout();
        self.add_menu_item(1);
        if self.floor > 50 {
            self.add_menu_item(17);
        }
        self.add_menu_item(3);
        self.add_menu_item(4);
        self.add_menu_item(5);
        self.add_menu_item(7);
        self.softkeys = (1, 3);
    }

    /// m_000 case 19（a.java:4439-4448）：商店 5 条目。
    fn enter_menu19(&mut self) {
        self.reset_menu_layout();
        for k in 12..=16 {
            self.add_menu_item(k);
        }
    }

    /// run case 4（a.java:3686-3731）：-7 返回、-6/-5 激活、-2/-4 下移、-1/-3 上移。
    fn tick_menu4(&mut self) {
        match self.key_value {
            -7 => self.game_mode = self.in_game_menu_return_mode,
            -6 | -5 => self.activate_menu_item(),
            -2 => {
                // 仅 -2 下移（a.java:3699）：-4/-3 落 default 无操作
                if self.menu_cursor < self.menu_item_count - 1 {
                    self.menu_cursor += 1;
                    if self.menu_cursor >= self.menu_visible_bottom {
                        self.menu_visible_bottom += 1;
                        self.menu_scroll_top += 1;
                    }
                } else {
                    self.menu_cursor = 0;
                    self.menu_scroll_top = 0;
                    self.menu_visible_bottom = self.menu_item_count;
                    if self.menu_item_count > self.menu_visible_rows {
                        self.menu_visible_bottom = self.menu_visible_rows;
                    }
                }
            }
            -1 => {
                // 仅 -1 上移（a.java:3713）
                if self.menu_cursor > 0 {
                    self.menu_cursor -= 1;
                    if self.menu_cursor < self.menu_scroll_top {
                        self.menu_scroll_top -= 1;
                        self.menu_visible_bottom -= 1;
                    }
                } else {
                    self.menu_cursor = self.menu_item_count - 1;
                    self.menu_visible_bottom = self.menu_item_count;
                    self.menu_scroll_top = self.menu_visible_bottom - self.menu_visible_rows;
                    if self.menu_scroll_top < 0 {
                        self.menu_scroll_top = 0;
                    }
                }
            }
            _ => {}
        }
    }

    /// run case 19（a.java:4196-4222）：-7 回 mode 4（无 break 落 default 效果相同）；
    /// -6/-5 无操作（购买走软键链路，本批对拍面不含）；-2/-4 下移环绕、-1/-3 上移环绕。
    fn tick_menu19(&mut self) {
        match self.key_value {
            -7 => {
                self.game_mode = 4;
                self.enter_menu4();
            }
            -4 | -2 | 54 | 56 => {
                if self.menu_cursor < self.menu_item_count - 1 {
                    self.menu_cursor += 1;
                } else {
                    self.menu_cursor = 0;
                }
            }
            -3 | -1 | 50 | 52 => {
                if self.menu_cursor > 0 {
                    self.menu_cursor -= 1;
                } else {
                    self.menu_cursor = self.menu_item_count - 1;
                }
            }
            _ => {}
        }
    }

    /// activateMenuItem（a.java:4762-4810）可达子集：1 继续游戏→mode3；
    /// 11 商店→mode19；17 跳过教程→changeFloor(1)+hp1000/atk710/def710→mode3。
    /// 3→mode7（存档槽）/4→mode16（设置）/5→mode15（帮助）/7→mode1（退出标题）
    /// 为跨机模式移交，桌面宿主批次接线——本批对拍面不激活。
    fn activate_menu_item(&mut self) {
        match self.menu_kinds[self.menu_cursor as usize] {
            1 => self.game_mode = 3,
            3 => {
                // a.java:4776-4780：gameMode=7 + menuReturnMode=4 + m_000
                self.menu_return_mode = 4;
                self.game_mode = 7;
                self.enter_menu7();
            }
            4 => {
                // a.java:4781-4785：gameMode=16 + menuReturnMode=4 + m_000
                self.menu_return_mode = 4;
                self.game_mode = 16;
                self.enter_menu16();
            }
            5 => {
                // a.java:4786-4790：gameMode=15 + menuReturnMode=4 + m_000
                self.menu_return_mode = 4;
                self.game_mode = 15;
                self.enter_menu15();
            }
            11 => {
                self.game_mode = 19;
                self.enter_menu19();
            }
            17 => {
                self.change_floor(1, false, false);
                self.game_mode = 3;
                self.player.hp = 1000;
                self.player.atk = 710;
                self.player.def = 710;
            }
            other => {
                let _ = other; // 3/4/5/7：待跨机模式批次
            }
        }
    }

    /// paint case 4/19 共享（a.java:2423-2486）：世界底（视差+HUD 面板+状态栏，
    /// 无瓦片/实体层）+ 框 + 条目（商店 12-16 逐字彩虹；普通 drawString 17）+
    /// 翻页箭头 + 软键栏。
    fn paint_menu_list(&mut self, g: &mut SoftGraphics<'_>) {
        self.paint_parallax(g, true);
        self.paint_hud(g);
        self.paint_status_bar(g);
        let font = crate::paint::paint_font();
        let box_w = self.menu_box_width + 22;
        let box_h = self.menu_list_height + 32;
        let mut bx = 120 - (self.menu_box_width + 22 >> 1);
        let mut by = 160 - (self.menu_list_height + 32 >> 1);
        let pad = (self.menu_line_height - font.height) >> 1;
        crate::menu_family::paint_box_frame(g, &self.images.ui[0], bx, by, box_w, box_h);
        by += 16 + pad;
        let mut i = self.menu_scroll_top;
        while i < self.menu_visible_bottom {
            let kind = self.menu_kinds[i as usize];
            if self.menu_cursor == i {
                g.set_color(0);
                g.fill_rect(120 - (self.menu_box_width >> 1), by - pad, self.menu_box_width, self.menu_line_height);
                g.set_color(16377897);
            } else if kind != 11 {
                g.set_color(7574946);
            } else {
                g.set_color(16751103);
            }
            if kind > 11 && kind < 17 {
                // 商店条目：逐字彩虹（f_int_arr_32[f_byte_25 起步环绕]），
                // 相位在 paint 尾按 fc&3 推进（a.java:2476-2479）
                let label = MENU_LABELS[kind as usize];
                let n = RAINBOW_COLORS.len();
                let label_u: Vec<u16> = label.encode_utf16().collect();
                let mut cx = 120 - (font.string_width(&label_u) >> 1);
                let mut phase = self.rainbow_phase as usize;
                for ch in label.chars() {
                    g.set_color(RAINBOW_COLORS[phase]);
                    g.draw_char(ch as u16, cx, by, 0);
                    cx += font.char_width(ch as u16);
                    phase = (phase + 1) % n;
                }
                if self.frame_counter & 3 == 0 {
                    self.rainbow_phase = (self.rainbow_phase + 1) % n as i32;
                }
            } else {
                let label: Vec<u16> = MENU_LABELS[kind as usize].encode_utf16().collect();
                g.draw_string(&label, 120, by, 17);
            }
            by += self.menu_line_height;
            i += 1;
        }
        let ax = 120 + (box_w >> 1) - 23;
        let ay = 160 + (box_h >> 1) - 38;
        if self.menu_scroll_top > 0 {
            draw_image_clipped(g, &self.images.ui[15], ax, ay, 0, 0, 7, 9);
        }
        if self.menu_visible_bottom < self.menu_item_count {
            draw_image_clipped(g, &self.images.ui[15], ax, ay + 9, 7, 0, 7, 9);
        }
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1);
    }

    /// m_000 case 10（a.java:4370-4380）：道具面板布局初始化。
    fn enter_item_panel(&mut self) {
        let font = crate::paint::paint_font();
        self.item_cols = 4;
        self.item_rows_total = 32 / self.item_cols;
        self.item_rows_visible = (268 - font.height - 10) / 40;
        if self.item_rows_visible > self.item_rows_total {
            self.item_rows_visible = self.item_rows_total;
        }
        self.item_scroll_top = 0;
        self.item_scroll_bottom = self.item_rows_visible;
        self.item_box_w = 42 + 40 * self.item_cols;
        self.item_box_h = 34 + font.height + 10 + 40 * self.item_rows_visible;
        self.softkeys = (1, 3);
    }

    /// run case 10（a.java:3931-4009）：-7 返回 mode 3；-6/-5 使用道具
    /// （13/14 直接生效回 mode 3、15/23/24/25 浮层(0,3)、其余浮层(1,2)）；
    /// -4/-3 行内环绕、-2/-1 翻行+滚动窗跟随。
    fn tick_item_panel(&mut self) {
        match self.key_value {
            -7 => self.game_mode = 3,
            -6 | -5 => {
                if self.item_cursor < self.player.item_stack_size {
                    let t = self.player.item_types[self.item_cursor as usize] as i32;
                    // itemUseTargetType（a.java:3941）：确认浮层图标/名称的数据源
                    self.overlay_kind_item = t;
                    match t {
                        13 | 14 => {
                            self.game_mode = 3;
                            self.activate_item(t);
                        }
                        15 | 23 | 24 | 25 => {
                            let text = self.item_description(self.item_desc_index(t));
                            self.show_overlay_message(3, &text, 0, 3);
                        }
                        _ => {
                            let text = self.item_description(self.item_desc_index(t));
                            self.show_overlay_message(3, &text, 1, 2);
                        }
                    }
                }
            }
            -4 => {
                if self.item_cursor % self.item_cols < self.item_cols - 1 {
                    self.item_cursor += 1;
                } else {
                    self.item_cursor -= self.item_cols - 1;
                }
            }
            -3 => {
                if self.item_cursor % self.item_cols > 0 {
                    self.item_cursor -= 1;
                } else {
                    self.item_cursor += self.item_cols - 1;
                }
            }
            -2 => {
                if self.item_cursor < self.item_cols * (self.item_rows_total - 1) {
                    self.item_cursor += self.item_cols;
                    if self.item_cursor / self.item_cols >= self.item_scroll_bottom {
                        self.item_scroll_top += 1;
                        self.item_scroll_bottom += 1;
                    }
                } else {
                    self.item_cursor -= (self.item_rows_total - 1) * self.item_cols;
                    self.item_scroll_top = 0;
                    self.item_scroll_bottom = self.item_rows_visible;
                }
            }
            -1 => {
                if self.item_cursor >= self.item_cols {
                    self.item_cursor -= self.item_cols;
                    if self.item_cursor / self.item_cols < self.item_scroll_top {
                        self.item_scroll_top -= 1;
                        self.item_scroll_bottom -= 1;
                    }
                } else {
                    self.item_cursor += (self.item_rows_total - 1) * self.item_cols;
                    self.item_scroll_bottom = self.item_rows_total;
                    self.item_scroll_top = self.item_rows_total - self.item_rows_visible;
                }
            }
            _ => {}
        }
    }

    /// activateItem（a.java:7781-7874+）的 mode10 头：13 → gameMode=5、
    /// 14 → gameMode=12（m_000 布局侧效果随对应模式批次）。16-25 的效果
    /// 分支（破墙/回血/瞬移等）为道具批次待办——序章背包恒空不可达。
    /// activateItem（a.java:7795-7860）：返回是否**消耗**（var2——true 时
    /// useItemStack 递减次数并在 0 时压实栈）。
    fn activate_item(&mut self, t: i32) -> bool {
        match t {
            13 => {
                // a.java:7786-7788：gameMode=5 + m_000 case5（m_091+软键）
                self.game_mode = 5;
                self.enter_menu5();
                false
            }
            14 => {
                // a.java:7789-7791：gameMode=12（m_000 case12 落穿 default）
                self.game_mode = 12;
                false
            }
            16 | 17 => {
                // a.java:7795-7810：四邻格 gates——16=型10 三昧真火、17=型11；
                // 未命中提示、命中 mode3 + 消耗
                let (target, msg) = if t == 16 {
                    (10, "你必须面对三昧真火再使用它。")
                } else {
                    (11, "你必须面对一堵墙使用")
                };
                if self.remove_adjacent_entity_of_type(target) {
                    self.game_mode = 3;
                    true
                } else {
                    self.show_overlay_message(0, msg, 0, 0);
                    false
                }
            }
            18 => {
                // a.java:7812-7825：移除全场型11 实体 + mode3 + 消耗
                let mut e = self.entities.count;
                while e > 0 {
                    e -= 1;
                    if self.entities.entity_type[e] == 11 {
                        entity::remove(&mut self.entities, e);
                    }
                }
                self.game_mode = 3;
                true
            }
            19 => {
                // a.java:7826-7831：(atk+def)*74/10 加血 + 提示 + 消耗
                let delta = (self.player.atk + self.player.def) * 74 / 10;
                self.apply_hp_delta(delta);
                self.show_overlay_message(0, &format!("增加了{delta}血量"), 0, 0);
                true
            }
            85 => {
                // a.java:7865-7875：移除全场型1（黄门）实体 + mode3 + 消耗
                let mut e = self.entities.count;
                while e > 0 {
                    e -= 1;
                    if self.entities.entity_type[e] == 1 {
                        entity::remove(&mut self.entities, e);
                    }
                }
                self.game_mode = 3;
                true
            }
            86 => {
                // a.java:7876-7886：四邻格 killAdjacentAt + mode3 + 消耗
                let cx = self.view.player_px >> 5;
                let cy = self.view.player_py >> 5;
                self.kill_adjacent_at(cx, cy - 1);
                self.kill_adjacent_at(cx, cy + 1);
                self.kill_adjacent_at(cx - 1, cy);
                self.kill_adjacent_at(cx + 1, cy);
                self.game_mode = 3;
                true
            }
            20 => {
                // a.java:7832-7850：对角瞬移——40 层禁用；镜像格 interact 放行
                // → placePlayerAtCell + 相机居中 + applyStepCellEffects + 消耗；
                // 拦截且无浮层 → 「无法移动到该位置」不消耗
                if self.floor == 40 {
                    self.show_overlay_message(0, "本层不能直接瞬移。", 0, 0);
                    false
                } else {
                    let tx = self.view.wide - 1 - (self.view.player_px >> 5);
                    let ty = self.view.high - 1 - (self.view.player_py >> 5);
                    let pass = self.interact_with_cell(tx, ty);
                    if pass {
                        self.m_024(tx, ty);
                        self.view.set_camera(
                            (self.view.view_w - 32 >> 1) - self.view.player_px,
                            (self.view.view_h - 32 >> 1) - self.view.player_py,
                        );
                        self.apply_step_cell_effects();
                        self.game_mode = 3;
                        true
                    } else {
                        if !self.overlay_active {
                            self.show_overlay_message(0, "无法移动到该位置", 0, 0);
                        }
                        false
                    }
                }
            }
            21 => {
                // a.java:7851-7852：changeFloor(+1)——返回值即消耗与否
                self.change_floor(self.floor + 1, false, false)
            }
            _ => false,
        }
    }

    /// killAdjacentAt（a.java:7923-7951）：格 (cx,cy) 槽位逐个——型5 宝箱类
    /// → removeSlotAt + tryOpenChest（开则 walkPhase=4 遮幅）；型<67 且
    /// renderCategory==8 且 state!=1（怪物）→ markEntityRemoved + 赏金
    /// （持 25 型金币加倍；popup 落世界原点 (0,0)——Java 字面量）。
    /// Java for 循环 slot++ 在 removeSlotAt 压实后仍前进（跳过补位项）——
    /// 缺陷保真，镜像之。
    fn kill_adjacent_at(&mut self, cx: i32, cy: i32) {
        if cx < 0 || cy < 0 || cx >= self.view.wide || cy >= self.view.high {
            return; // Java try/catch(ArrayIndexOutOfBounds) 静默等价
        }
        let ct = self.grid.cell_type[cy as usize][cx as usize] as usize;
        let cap = self.grid.capacity[ct] as i32;
        let mut slot = 0i32;
        while slot < cap {
            let e = self.grid.slots[ct][slot as usize] as usize - 1;
            let t = self.entities.entity_type[e];
            if t == 5 {
                let param = self.entities.param[e];
                self.remove_slot_at(cx, cy, slot);
                if self.try_open_chest(param) {
                    self.walk_phase = 4;
                }
            } else if t < 67
                && entity::render_category(t) == 8
                && self.entities.solid[e] != 1
            {
                entity::remove(&mut self.entities, e);
                let gold = if self.player.find_item(25) >= 0 {
                    self.enemy_base_gold[(t - 41) as usize] << 1
                } else {
                    self.enemy_base_gold[(t - 41) as usize]
                };
                self.player.gold += gold;
                if gold > 0 {
                    self.popups.spawn(
                        4,
                        gold,
                        0,
                        0,
                        self.view.cam_x,
                        self.view.cam_y,
                        self.images.popup.digit_strips[0].width / 11,
                    );
                }
            }
            slot += 1;
        }
    }

    /// m_100 语义（a.java:cellSlotTable 查找）：格 (cx,cy) 上首个型 target
    /// 且 state!=1 的实体下标，无则 -1。
    fn find_entity_of_type_at(&self, cx: i32, cy: i32, target: i32) -> i32 {
        if cx < 0 || cy < 0 || cx >= self.view.wide || cy >= self.view.high {
            return -1;
        }
        let ct = self.grid.cell_type[cy as usize][cx as usize] as usize;
        let cap = self.grid.capacity[ct] as usize;
        for s in 0..cap {
            let e = self.grid.slots[ct][s] as usize - 1;
            if self.entities.entity_type[e] == target && self.entities.solid[e] != 1 {
                return e as i32;
            }
        }
        -1
    }

    /// m_089（a.java:7892-7920）：四邻格逐格移除首个型 target 实体，
    /// 任一格命中返回 true（Java 无小地图重建，镜像之）。
    fn remove_adjacent_entity_of_type(&mut self, target: i32) -> bool {
        let cx = self.view.player_px >> 5;
        let cy = self.view.player_py >> 5;
        let mut hit = false;
        for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
            let e = self.find_entity_of_type_at(cx + dx, cy + dy, target);
            if e >= 0 {
                entity::remove(&mut self.entities, e as usize);
                hit = true;
            }
        }
        hit
    }

    /// spawnBossEvent（a.java:8903-8912）：按 bossEventSpawns[idx] 逐项
    /// spawnEntity（cell<<5 网格吸附在 entity::spawn 内）+ markEntityDesRemoved
    /// （DES 态 solid=2，登场动画帧 8）。
    fn spawn_boss_event(&mut self, idx: usize) {
        for ent in BOSS_EVENT_SPAWNS[idx].chunks_exact(4) {
            let e = entity::spawn(
                &mut self.entities,
                &mut self.grid,
                ent[0],
                ent[1] << 5,
                ent[2] << 5,
                ent[3] as i16,
                &self.width_table,
                None,
            );
            entity::remove_des(&mut self.entities, e);
        }
    }

    /// useItemStack（a.java:7761-7781）：uses>0 → activate，消耗且减至 0
    /// 时栈压实（后项前移 + size--）；uses==0 → 仅 activate 不消耗。
    fn use_item_stack(&mut self, idx: usize) {
        if self.player.item_uses[idx] > 0 {
            let consumed = self.activate_item(self.player.item_types[idx] as i32);
            if consumed {
                self.player.item_uses[idx] -= 1;
                if self.player.item_uses[idx] == 0 {
                    let n = self.player.item_stack_size as usize;
                    for k in idx..n.saturating_sub(1) {
                        self.player.item_types[k] = self.player.item_types[k + 1];
                        self.player.item_uses[k] = self.player.item_uses[k + 1];
                    }
                    if self.player.item_stack_size > 0 {
                        self.player.item_stack_size -= 1;
                    }
                }
            }
        } else {
            self.activate_item(self.player.item_types[idx] as i32);
        }
    }

    /// itemTypeToStackIndex（a.java:899-905 语义：85/86 特判，其余 -13）。
    fn item_desc_index(&self, t: i32) -> usize {
        if t == 85 {
            20
        } else if t == 86 {
            21
        } else {
            (t - 13) as usize
        }
    }

    /// paint case 10（a.java:2724-2792）：斜线背景 + paintBoxFrame + 标题 +
    /// 分隔线 + 4×N 格阵（选中框 [2][10] 双帧闪烁）+ 翻页箭头 + 软键栏。
    fn paint_item_panel(&mut self, g: &mut SoftGraphics<'_>) {
        let font = crate::paint::paint_font();
        let mut v159 = 319i32;
        while v159 >= 1 {
            g.draw_line(-1, -1 + (320 - v159), -1 + v159 - 1, 318);
            g.draw_line(239 - v159, -1, 239, -1 + v159);
            v159 -= 2;
        }
        let mut x = (240 - self.item_box_w) >> 1;
        let mut y = (320 - self.item_box_h) >> 1;
        crate::menu_family::paint_box_frame(g, &self.images.ui[0], x, y, self.item_box_w, self.item_box_h);
        x += 20;
        y += 16;
        g.set_color(13097429);
        if self.item_cursor < self.player.item_stack_size {
            let name = self.object_type_names[self.player.item_types[self.item_cursor as usize] as usize]
                .encode_utf16()
                .collect::<Vec<u16>>();
            g.draw_string(&name, x, y + 2, 0);
        }
        y += font.height + 4;
        x -= 9;
        g.set_color(6178);
        g.draw_line(x, y, x + self.item_box_w - 23, y);
        y += 1;
        g.set_color(3564144);
        g.draw_line(x, y, x + self.item_box_w - 23, y);
        y += 10;
        x += 9;
        let col_x0 = x;
        let mut grid = self.item_cols * self.item_scroll_top;
        let mut row = self.item_scroll_top;
        while row < self.item_scroll_bottom {
            for _ in 0..self.item_cols {
                crate::menu_family::paint_mini_frame(g, x, y, 32, 32);
                if grid == self.item_cursor {
                    draw_image_clipped(
                        g,
                        &self.images.map210,
                        x - 4,
                        y - 3,
                        38 * (self.frame_counter & 1) as i32,
                        0,
                        38,
                        38,
                    );
                }
                if grid < self.player.item_stack_size {
                    let t = self.player.item_types[grid as usize] as usize;
                    if let Some(Some(img)) = self.images.entity.get(t) {
                        g.draw_image(img, x + ((32 - img.width) >> 1), y + ((32 - img.height) >> 1), 0);
                    }
                    if self.player.item_uses[grid as usize] > 0 {
                        crate::menu_family::paint_number(
                            g,
                            &self.images.ui[2],
                            self.player.item_uses[grid as usize] as i32,
                            x + 32,
                            y + 26,
                        );
                    }
                }
                grid += 1;
                x += 40;
            }
            row += 1;
            x = col_x0;
            y += 40;
        }
        x = ((240 + self.item_box_w) >> 1) - 22;
        y -= (self.item_box_h - 32 - font.height - 4) >> 1;
        if self.item_scroll_top > 0 {
            draw_image_clipped(g, &self.images.ui[15], x, y - 15, 0, 0, 7, 9);
        }
        if self.item_scroll_bottom < self.item_rows_total {
            draw_image_clipped(g, &self.images.ui[15], x, y + 5, 7, 0, 7, 9);
        }
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1);
    }

    /// m_000 case 7（a.java:4342-4364，case7/8 共享）：软键 (1,3) + loadSlotTable
    /// （oracle 冷态 RMS 恒空 ⇒ 六槽全无效）+ 槽表布局（6 行/宽 148/高 196）。
    fn enter_menu7(&mut self) {
        self.slot = crate::menu_family::SlotSelect::new();
        // loadSlotTable（a.java:9313-9346）：每次进入重读 RMS 槽表
        if let Some(mot_if) = &self.preset_mot_if {
            self.slot.load_slot_table(Some(mot_if));
        }
        self.softkeys = (1, 3);
    }

    /// run case 7（a.java:3829-3878，case7/8 共享）：-7 回 menuReturnMode；
    /// -6/-5 在 mode7 = 存档（m_119 + saveFloorState + “保存成功！”浮层）；
    /// -2/-1 槽光标滚动。
    fn tick_menu7(&mut self) {
        match self.key_value {
            -7 => self.game_mode = self.menu_return_mode,
            -6 | -5 => {
                if self.game_mode == 7 {
                    self.save_floor_state();
                    let cursor = self.slot.cursor as usize;
                    let blob = self.serialize_slot_blob();
                    self.saved_slot_blobs[cursor] = Some(blob);
                    // markSlotEntry（a.java:9152-9172）
                    self.slot.valid[cursor] = true;
                    self.slot.floor_num[cursor] = self.floor as i8;
                    self.slot.hp[cursor] = self.player.hp;
                    self.slot.atk[cursor] = self.player.atk;
                    self.slot.def[cursor] = self.player.def;
                    self.slot.keys_yellow[cursor] = self.player.yellow_keys as i16;
                    self.slot.keys_blue[cursor] = self.player.blue_keys as i16;
                    self.slot.keys_red[cursor] = self.player.red_keys as i16;
                    self.slot.gold[cursor] = self.player.gold;
                    self.slot.weapon[cursor] = 0; // equippedWeaponType（装备批次）
                    self.slot.armor[cursor] = 0; // equippedArmorType（装备批次）
                    self.show_overlay_message(0, "保存成功！", 0, 0);
                }
            }
            -2 => self.slot.cursor_down(),
            -1 => self.slot.cursor_up(),
            _ => {}
        }
    }

    /// saveFloorState 写侧（a.java:9182-9244）：blob 序 = parse_floor_save 读序
    /// 的镜像（difficulty..follow 对/128 旗标/道具栈/56 层触发）。
    /// 装备位恒 0：equippedWeaponType/ArmorType 为装备批次字段（当前层恒无装备）。
    fn serialize_slot_blob(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.push(self.difficulty_index as u8);
        o.push(self.floor as u8);
        o.push(self.min_floor as u8);
        o.push(self.max_floor as u8);
        o.push(self.facing as u8);
        o.push(0u8); // equippedWeaponType
        o.push(0u8); // equippedArmorType
        o.extend_from_slice(&(self.view.player_px >> 5).to_be_bytes()); // playerCellX
        o.extend_from_slice(&(self.view.player_py >> 5).to_be_bytes()); // playerCellY
        for v in [self.player.hp, self.player.atk, self.player.def] {
            o.extend_from_slice(&v.to_be_bytes());
        }
        for v in [
            self.player.yellow_keys as i16,
            self.player.blue_keys as i16,
            self.player.red_keys as i16,
        ] {
            o.extend_from_slice(&v.to_be_bytes());
        }
        o.extend_from_slice(&self.player.gold.to_be_bytes());
        o.extend_from_slice(&0i16.to_be_bytes()); // alchemyUpgradeCount
        // f_int_89 + f_byte_arr_14/15 跟随对（剧情物清单，序章恒空）
        o.extend_from_slice(&(self.special_item_types.len() as i16).to_be_bytes());
        for k in 0..self.special_item_types.len() {
            o.push(self.special_item_types[k] as u8);
            o.push(self.special_item_uses[k] as u8);
        }
        // 128 字节脚本旗标（不足补 false）
        for k in 0..128 {
            let v = self.script_line_flags.get(k).copied().unwrap_or(false);
            o.push(v as u8);
        }
        // 道具栈
        o.push(self.player.item_stack_size as u8);
        for k in 0..self.player.item_stack_size as usize {
            o.push(self.player.item_types[k]);
            o.push(self.player.item_uses[k]);
        }
        // 56 层触发状态
        for k in 0..56usize {
            match &self.saved_floor_states[k] {
                Some(d) => {
                    o.extend_from_slice(&(d.len() as i16).to_be_bytes());
                    o.extend_from_slice(d);
                }
                None => o.extend_from_slice(&(-1i16).to_be_bytes()),
            }
        }
        o
    }

    /// m_113(true)（a.java:2679-2682 + 9046-9215）：mode 7 槽表（存档标题变体
    /// titledBox segs[2]）+ 有效槽详情面板（层数/装备/金币/三围/三钥）。
    fn paint_slot_list(&mut self, g: &mut SoftGraphics<'_>) {
        self.paint_parallax(g, false);
        let font = crate::paint::paint_font();
        let ui = &self.images.ui;
        let x = 240 - self.slot.content_w - 22 >> 1;
        let mut y = 320 - self.slot.content_h >> 1;
        // 标题变体 2（存档）：titleBarSegments[2] = {4,5}
        crate::menu_family::paint_titled_box(g, &ui[0], &ui[13], &[4, 5], x, y, self.slot.content_w + 22, self.slot.content_h);
        y += 16;
        let mut row = self.slot.scroll_top;
        while row < self.slot.scroll_bottom {
            if row != self.slot.cursor {
                g.set_color(crate::layout::SLOT_TEXT);
            } else {
                g.set_color(crate::layout::DARK_BACKDROP);
                g.fill_rect(120 - (self.slot.content_w >> 1), y, self.slot.content_w, 10 + 4);
                g.draw_image(&ui[14], 120 - (self.slot.content_w >> 1) + 10, y + (10 - 8 >> 1), 0);
                let t = crate::paint::NOKIA_TRANSFORM_TABLE[1];
                g.draw_image_transformed(&ui[14], 120 + (self.slot.content_w >> 1) - 30, y + (10 - 8 >> 1), 0, t);
                g.set_color(crate::layout::SLOT_TEXT_ACTIVE);
            }
            if !self.slot.valid[row as usize] {
                g.draw_string(&[0x2D, 0x2D, 0x2D], 120, y + 2, 17);
            } else {
                let mut label: Vec<u16> = "存档".encode_utf16().collect();
                label.extend((row as u16).to_string().chars().map(|c| c as u16));
                g.draw_string(&label, 120, y + 2, 17);
            }
            y += 14;
            row += 1;
        }
        // 翻页箭头 + 分隔线（a.java:9078-9090）
        let ax = 120 + ((self.slot.content_w >> 1) - 10);
        if self.slot.scroll_top > 0 {
            draw_image_clipped(g, &ui[15], ax, y - 20, 0, 0, 7, 9);
        }
        if self.slot.scroll_bottom < 6 {
            draw_image_clipped(g, &ui[15], ax, y - 10, 7, 0, 7, 9);
        }
        y += 2;
        let lx = ax - self.slot.content_w + 10;
        g.set_color(crate::layout::SLOT_RULE_DARK);
        g.draw_line(lx, y, lx + self.slot.content_w - 1, y);
        y += 1;
        g.set_color(crate::layout::SLOT_RULE_LIGHT);
        g.draw_line(lx, y, lx + self.slot.content_w - 1, y);
        // 详情面板（a.java:9092-9147，slotValidFlags[cursor] 才画）
        let c = self.slot.cursor as usize;
        if self.slot.valid[c] {
            let mut vx = lx + 6;
            let mut vy = y + 8;
            if self.slot.floor_num[c] < 51 {
                paint_number(g, &ui[18], self.slot.floor_num[c] as i32, vx + 48, vy);
                g.draw_image(&ui[8], vx + 50, vy + 8, 0);
            }
            vx = (240 + self.slot.content_w >> 1) - 68;
            crate::menu_family::paint_mini_frame(g, vx, vy, 32, 32);
            // 武器（f_byte_arr_25；装备批次恒 0 = “无”）
            g.set_color(0xFFFFFF);
            let m = EQUIPMENT_MATERIAL_NAMES[self.slot.weapon[c as usize].clamp(0, 8) as usize];
            let mu: Vec<u16> = m.encode_utf16().collect();
            g.draw_string(&mu, vx + ((32 - font.char_width('回' as u16)) >> 1), vy + ((32 - font.height) >> 1), 0);
            vx += 34;
            crate::menu_family::paint_mini_frame(g, vx, vy, 32, 32);
            g.set_color(0xFFFFFF);
            let a = EQUIPMENT_MATERIAL_NAMES[self.slot.armor[c as usize].clamp(0, 8) as usize];
            let au: Vec<u16> = a.encode_utf16().collect();
            g.draw_string(&au, vx + ((32 - font.char_width('回' as u16)) >> 1), vy + ((32 - font.height) >> 1), 0);
            vx = (240 - self.slot.content_w + 10 >> 1);
            vy += 20;
            g.draw_image(&ui[1], vx, vy, 0);
            vx += 60;
            paint_number(g, &ui[2], self.slot.gold[c], vx, vy + 2);
            vx -= 64;
            vy += 16;
            draw_image_clipped(g, &ui[7], vx, vy, 0, 2, 10, 10);
            vx += 15;
            crate::menu_family::paint_mini_frame(g, vx - 1, vy, 48, 12);
            paint_number(g, &ui[2], self.slot.hp[c], vx + 45, vy + 2);
            vx += 47;
            draw_image_clipped(g, &ui[7], vx, vy, 10, 0, 10, 13);
            vx += 11;
            g.set_color(512);
            crate::menu_family::paint_mini_frame(g, vx, vy, 30, 12);
            paint_number(g, &ui[2], self.slot.atk[c], vx + 28, vy + 2);
            vx += 32;
            draw_image_clipped(g, &ui[7], vx, vy, 20, 2, 10, 10);
            vx += 12;
            g.set_color(512);
            crate::menu_family::paint_mini_frame(g, vx, vy, 30, 12);
            paint_number(g, &ui[2], self.slot.def[c], vx + 28, vy + 2);
            vx = (240 - self.slot.content_w >> 1) + 6;
            vy += 16;
            self.paint_key_slot(g, 0, self.slot.keys_yellow[c] as i32, vx, vy);
            vx += 42;
            self.paint_key_slot(g, 1, self.slot.keys_blue[c] as i32, vx, vy);
            vx += 42;
            self.paint_key_slot(g, 2, self.slot.keys_red[c] as i32, vx, vy);
        }
        m_034_softkeys(g, &ui[10], &ui[11], self.softkeys.0, self.softkeys.1);
    }

    /// mode 15 帮助全文（m_000 case 15 a.java:4407-4411，showOverlayMessage
    /// kind 0 逐字原文：99FFCC 色段 + 换行转义）。
    pub const HELP_TEXT: &str = "游戏描述：\n\\c99FFCC有人的地方就有江湖，有神仙的地方何尝不是江湖；百战百胜的本事，换不回女人的真心，兄弟的真义；齐天大圣又如何，没有真情实义，做神仙跟做咸鱼有什么区别？\n\n操作方式：按左软键调出物品栏，左右选择一件道具，按确定键使用。\n游戏操作：\n上方向键/2：向上行走\n下方向键/8：向下行走\n左方向键/4：向左行走\n右方向键/6：向右行走\n确定键/5:探索地图\n左软键：打开道具列表\n右软键：打开游戏中菜单\n\n代理发行：广州易诚计算机科技有限公司\n发行商网站：www.9266.net\n客服电话：4006509913\n客服信箱：kefu@9266.net";

    /// m_000 case 15（a.java:4402-4413）：帮助全文浮层（kind 0）+ 软键 (0,3)。
    /// m_001(8)/m_001(1) = 懒加载资源容器（Rust 构造期全量装载，等价 no-op）。
    /// 迭代 26：改走 showOverlayMessage 完整口（原内联排版漏 overlayActive=true，
    /// 浮层整体不画——preset_help 嵌套假绿掩盖至今）。
    fn enter_menu15(&mut self) {
        self.show_overlay_message(0, Self::HELP_TEXT, 0, 0);
        self.softkeys = (0, 3);
    }

    /// run case 15（fallthrough no-op 组 a.java:3747-3762）+ run 头浮层支路
    /// （a.java:3262-3272，kind0 且 gameMode∈{15,17}）：仅 -7 关浮层且回
    /// menuReturnMode；-2/-1 翻页；**其余键（含 -6）一律吞掉不关**——
    /// 迭代 26 更正：原 `_ => 关浮层` 系误植，-6 关闭语义不存在于原版。
    fn tick_menu15(&mut self) {
        if self.overlay_active {
            match self.key_value {
                -7 => {
                    self.overlay_active = false;
                    self.game_mode = self.menu_return_mode;
                }
                -2 => self.flip_overlay_page(self.overlay_page_top + self.overlay_lines_per_page),
                -1 => self.flip_overlay_page(self.overlay_page_top - self.overlay_lines_per_page),
                _ => {}
            }
            self.key_value = 0;
        }
    }

    /// paint case 15（a.java:2958-2962）：parallax(false) + 帮助图标条
    /// （menu_strip sy=51）+ 软键 + 浮层尾。
    fn paint_menu15(&mut self, g: &mut SoftGraphics<'_>) {
        self.paint_parallax(g, false);
        let mut f = self.strip_frame;
        crate::title::paint_icon_strip_at(g, &self.images.menu_strip, 86, 10, 51, 68, &mut f);
        self.strip_frame = f;
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1);
        if !self.transitioning && self.overlay_active {
            self.paint_overlay_box(g);
        }
    }


    /// m_091（a.java:7954-8010）：敌人图鉴数据构建——活体敌人（cat8/type41+）
    /// 按类型去重 → 预测伤害（predictBattleHpLoss noCounter=true）→ 冒泡降序
    /// → 可见行数 258/50 钳条目数 → 面板高 50×行+32 → 滚动窗 [0,行)。
    fn build_encyclopedia(&mut self) {
        self.encyc_damage = vec![0; 128];
        self.encyc_types = vec![0; 128];
        self.encyc_panel_w = 208;
        self.encyc_count = 0;
        'outer: for e in 0..self.entities.count {
            if self.entities.removed[e] {
                continue;
            }
            let t = self.entities.entity_type[e] as i32;
            if entity::render_category(t) != 8 || t < 41 {
                continue;
            }
            for k in 0..self.encyc_count as usize {
                if self.encyc_types[k] as i32 == t {
                    continue 'outer;
                }
            }
            let dmg = self.predict_hp_loss(t, true);
            let c = self.encyc_count as usize;
            self.encyc_damage[c] = dmg;
            self.encyc_types[c] = t as u8;
            self.encyc_count += 1;
        }
        // 冒泡降序（a.java:7986-7998）
        for a in 0..self.encyc_count {
            let mut b = self.encyc_count;
            while b > a + 1 {
                b -= 1;
                let dm = self.encyc_damage[b as usize];
                let dp = self.encyc_damage[b as usize - 1];
                if dm >= 0 && dm < dp {
                    self.encyc_damage.swap(b as usize, b as usize - 1);
                    self.encyc_types.swap(b as usize, b as usize - 1);
                }
            }
        }
        let font = crate::paint::paint_font();
        let row_h = font.height + 4 + 36;
        self.encyc_rows_visible = 258 / row_h;
        if self.encyc_rows_visible > self.encyc_count {
            self.encyc_rows_visible = self.encyc_count;
        }
        self.encyc_panel_h = row_h * self.encyc_rows_visible + 32;
        self.encyc_scroll_top = 0;
        self.encyc_scroll_bottom = self.encyc_rows_visible;
    }

    /// m_092（a.java:8016-8027）：翻页。
    fn encyclopedia_set_page(&mut self, page: i32) {
        let max_page = (self.encyc_count - self.encyc_rows_visible).max(0);
        if page >= 0 && page <= max_page {
            self.encyc_scroll_top = page;
            self.encyc_scroll_bottom = self.encyc_scroll_top + self.encyc_rows_visible;
            if self.encyc_scroll_bottom > self.encyc_count {
                self.encyc_scroll_bottom = self.encyc_count;
            }
        }
    }

    /// m_000 case 5：m_091 + 软键 (0,3)。
    fn enter_menu5(&mut self) {
        self.build_encyclopedia();
        self.softkeys = (0, 3);
    }

    /// run case 5（a.java:3737-3755）：-7/-6 回 mode 3；-2 下一页、-1 上一页。
    fn tick_menu5(&mut self) {
        match self.key_value {
            -7 | -6 => self.game_mode = 3,
            -2 => {
                let p = self.encyc_scroll_top + 1;
                self.encyclopedia_set_page(p);
            }
            -1 => {
                let p = self.encyc_scroll_top - 1;
                self.encyclopedia_set_page(p);
            }
            _ => {}
        }
    }

    /// paint case 5（a.java:2497-2560）：parallax(false) + 居中 paintBoxFrame
    /// + 逐行（名称/预测/精灵/攻防/金币）+ 行分隔线 + 箭头 + 软键。
    fn paint_menu5(&mut self, g: &mut SoftGraphics<'_>) {
        self.paint_parallax(g, false);
        let font = crate::paint::paint_font();
        let bx = (SCREEN_W - self.encyc_panel_w) >> 1;
        let by = (SCREEN_H - self.encyc_panel_h) >> 1;
        if self.encyc_scroll_bottom - self.encyc_scroll_top > 0 {
            crate::menu_family::paint_box_frame(g, &self.images.ui[0], bx, by, self.encyc_panel_w, self.encyc_panel_h);
            if self.encyc_scroll_top > 0 {
                draw_image_clipped(g, &self.images.ui[19], 112, by - 17 - (self.frame_counter & 1) as i32, 16, 0, 15, 17);
            }
            let mut x = bx + 16;
            let mut y = by + 16;
            for row in self.encyc_scroll_top..self.encyc_scroll_bottom {
                let c = row as usize;
                let t = self.encyc_types[c] as usize;
                let pred = self.encyc_damage[c];
                x = bx + 16;
                g.set_color(13097429);
                let name: Vec<u16> = self.object_type_names[t].encode_utf16().collect();
                g.draw_string(&name, x, y + 2, 0);
                let dmg_x = (SCREEN_W + self.encyc_panel_w >> 1) - 75;
                let name_dy = (font.height - 19) >> 1;
                y += name_dy;
                if pred < 0 {
                    g.draw_image(&self.images.ui[17], dmg_x - 4, y + 2, 0);
                } else {
                    g.draw_image(&self.images.ui[16], dmg_x - 12, y + 2, 0);
                    x = dmg_x + 15;
                    crate::menu_family::paint_mini_frame(g, x - 1, y + 6, 48, 12);
                    paint_number(g, &self.images.ui[2], pred, x + 45, y + 8);
                }
                x = bx + 16;
                y += name_dy + 19 + 4;
                crate::menu_family::paint_mini_frame(g, x, y, 32, 32);
                if t > 40 {
                    let img = self.images.entity.get(t).cloned().flatten();
                    if let Some(img) = img {
                        let fw = self.type_frame_width_for(t as i32);
                        let off_x = (32 - fw) >> 1;
                        let draw_h = (img.height).min(31);
                        if off_x < 0 {
                            draw_image_clipped(g, &img, x, y + 31 - draw_h, -off_x, 0, fw, draw_h);
                        } else {
                            draw_image_clipped(g, &img, x + off_x, y + 31 - draw_h, 0, 0, fw, draw_h);
                        }
                    }
                } else {
                    g.set_clip(x, y, 32, 32);
                    if let Some(Some(img)) = self.images.entity.get(t) {
                        g.draw_image(img, x + ((32 - img.width) >> 1), y + ((32 - img.height) >> 1), 0);
                    }
                    g.set_clip(x, y, SCREEN_W, SCREEN_H);
                }
                y += 4;
                x += 36;
                draw_image_clipped(g, &self.images.ui[7], x, y, 10, 0, 10, 13);
                x += 15;
                crate::menu_family::paint_mini_frame(g, x, y, 48, 12);
                let atk_v = self.enemies.atk.get(t.saturating_sub(41)).copied().unwrap_or(0);
                paint_number(g, &self.images.ui[2], atk_v, x + 40, y + 2);
                x += 52;
                draw_image_clipped(g, &self.images.ui[7], x, y, 20, 2, 10, 10);
                x += 15;
                g.set_color(512);
                crate::menu_family::paint_mini_frame(g, x, y, 48, 12);
                let def_v = self.enemies.def.get(t.saturating_sub(41)).copied().unwrap_or(0);
                paint_number(g, &self.images.ui[2], def_v, x + 40, y + 2);
                x -= 82;
                y += 12;
                draw_image_clipped(g, &self.images.ui[7], x, y + 5, 0, 2, 10, 10);
                x += 15;
                crate::menu_family::paint_mini_frame(g, x, y + 5, 48, 12);
                let hp_v = self.enemies.hp.get(t.saturating_sub(41)).copied().unwrap_or(0);
                paint_number(g, &self.images.ui[2], hp_v, x + 40, y + 7);
                x += 52;
                g.draw_image(&self.images.ui[1], x, y + 5, 0);
                x += 15;
                crate::menu_family::paint_mini_frame(g, x, y + 5, 48, 12);
                let gold = self.enemy_base_gold.get(t.saturating_sub(41)).copied().unwrap_or(0);
                x += 40;
                paint_number(g, &self.images.ui[2], gold, x, y + 7);
                y += 18;
                if row + 1 < self.encyc_scroll_bottom {
                    x = bx + 16 - 5;
                    g.set_color(6435);
                    g.draw_line(x, y, x + self.encyc_panel_w - 22, y);
                    y += 1;
                    g.set_color(4803902);
                    g.draw_line(x, y, x + self.encyc_panel_w - 22, y);
                }
                y += 2;
            }
            y += 16;
            if self.encyc_scroll_bottom < self.encyc_count {
                draw_image_clipped(g, &self.images.ui[19], 112, y + (self.frame_counter & 1) as i32, 16, 17, 15, 17);
            }
        }
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1);
    }

    /// alchemyPriceFor（a.java:7256-7262）：第 k 次升级价 = 20 + Σ20i (i=1..k-2)
    fn alchemy_price_for(k: i32) -> i32 {
        let mut price = 20;
        for i in 1..k {
            // Java: for (var2 = 1; var2 < var0; var2++)（a.java:7259）——上界含 k-1
            price += 20 * i;
        }
        price
    }

    /// scaledByFloorTier（a.java:7250-7254）：tier = (floor-1)/10 + 1，n × tier
    fn scaled_by_floor_tier(n: i32, floor: i32) -> i32 {
        n * (((floor - 1) / 10) + 1)
    }

    /// run case 9（a.java:3885-3940）：-7 回 mode3；-6/-5 金币够则按光标加属性
    /// 并推进计数/价格，否则「没有足够的金钱」浮层；-2/-1 光标三行环绕。
    fn tick_menu9(&mut self) {
        match self.key_value {
            -7 => self.game_mode = 3,
            -6 | -5 => {
                if self.player.gold >= self.alchemy_price {
                    self.player.gold -= self.alchemy_price;
                    match self.alchemy_cursor {
                        0 => self.player.hp += self.alchemy_gain_hp,
                        1 => self.player.atk += self.alchemy_gain_atk,
                        _ => self.player.def += self.alchemy_gain_def,
                    }
                    self.alchemy_upgrade_count += 1;
                    self.alchemy_price = Self::alchemy_price_for(self.alchemy_upgrade_count + 1);
                } else {
                    self.show_overlay_message(0, "没有足够的金钱", 0, 0);
                }
            }
            -2 => {
                self.alchemy_cursor = if self.alchemy_cursor < 2 { self.alchemy_cursor + 1 } else { 0 };
            }
            -1 => {
                self.alchemy_cursor = if self.alchemy_cursor > 0 { self.alchemy_cursor - 1 } else { 2 };
            }
            _ => {}
        }
    }

    /// paint case 9（a.java:2690-2740）：地图/实体/HUD/状态条 + 炼丹题头框
    ///（paintTitledBox(0,·)）+ 花费行 + 三行属性（光标高亮 + 双柱图标 + 数字）。
    fn paint_menu9(&mut self, g: &mut SoftGraphics<'_>, tileset: &ArgbImage) {
        self.paint_parallax(g, true);
        self.view.paint_tiles(g, tileset, view_top_y());
        self.paint_entities(g, self.view.cam_x, self.view.cam_y + view_top_y());
        self.paint_hud(g);
        self.paint_status_bar(g);
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1);

        let font = crate::paint::paint_font();
        let bw = 115i32; // f_int_75（a.java:810）
        let bh = font.height + 90; // f_int_74（a.java:809）
        let bx = (240 - bw - 22) >> 1;
        let by = (320 - bh) >> 1;
        crate::menu_family::paint_titled_box(g, &self.images.ui[0], &self.images.ui[13], &[0, 1, 2], bx, by, bw + 22, bh);
        let wide = font.char_width('回' as u16) << 1;
        let mut x = (240 - (wide + 16 + 48)) >> 1;
        let mut y = by + 21;
        g.set_color(7575203);
        g.draw_string(&"花费".encode_utf16().collect::<Vec<u16>>(), x, y, 0);
        x += wide + 4;
        g.draw_image(&self.images.ui[1], x, y + ((font.height - 10) >> 1), 0);
        x += 60;
        paint_number(g, &self.images.ui[2], self.alchemy_price, x, y + 3 + ((font.height - 10) >> 1));
        y += font.height + 5;
        x = (240 - bw) >> 1;
        g.set_color(549016);
        g.fill_rect(x, y + (self.alchemy_cursor << 4), bw, 16);
        g.draw_image(&self.images.ui[14], x + 10, y + 2 + (self.alchemy_cursor << 4), 0);
        g.draw_image_transformed(&self.images.ui[14], x + bw - 30, y + 2 + (self.alchemy_cursor << 4), 0, 8192);
        draw_image_clipped(g, &self.images.ui[7], 97, y + 4, 0, 2, 10, 10);
        paint_number(g, &self.images.ui[2], self.alchemy_gain_hp, 142, y + 6);
        y += 16;
        draw_image_clipped(g, &self.images.ui[7], 97, y + 4, 11, 0, 8, 13);
        paint_number(g, &self.images.ui[2], self.alchemy_gain_atk, 142, y + 6);
        y += 16;
        draw_image_clipped(g, &self.images.ui[7], 97, y + 4, 20, 2, 10, 11);
        paint_number(g, &self.images.ui[2], self.alchemy_gain_def, 142, y + 6);
        // 全局浮层（paint 尾部 a.java:3125-3210 对全部模式生效）
        self.paint_overlay_box(g);
    }

    pub const OFFER_TEXTS_A: [&str; 22] = [
            "",
            "",
            "有些门不能用钥匙打开，只有当你打败它的守卫后才会自动打开。",
            "你购买了礼物后再与天宫商人对话，他会告诉你一些重要的消息。",
            "",
            "我听说在天宫中有2把隐藏的红钥匙。",
            "在这个区域不多次提升攻击力，就不能打败“杨戬副手”。切记前人教训！",
            "太上老君就在25楼。以你现在的状态去攻击他简直就是自杀。 你应当在取得更高级别的道具后再去打败他。",
            "不找到所有的暗墙29楼的暗道是不会打开的",
            "如果你到27楼时状态为：生命1500、攻击80、防御98、拥有1把蓝钥匙、5把黄钥匙。那么祝贺你，你的前期是比较成功的。",
            "六丁六甲的攻击力太高了，你最好到能对他一击必杀时再与他战斗。",
            "别匆忙，放慢速度。",
            "如果你能用好4种移动宝物，你不用与强敌作战就能上楼。",
            "",
            "你需要用“玄明石”取出37楼仓库内的所有宝物。",
            "谜题：“在3点，拥有传送功能的密宝就会出现。”",
            "“巫师”会用魔法攻击路过的人，在2个“太上老君护卫”间通过会使你的生命减少一半。",
            "44楼，被藏在异空间，你只能用密宝才能到达。",
            "41楼事实上是左右对称的。",
            "像骰子上5的形状是一种封印魔法，你最好记住它在你与49楼假魔王战斗时有用",
            "",
            "你好，我是太白金星。你最好别见敌人就杀，先往上走，拿到武器和防具再做打算。",
    ];

    pub const OFFER_TEXTS_B: [&str; 22] = [
            "感谢你救了我，这是1000金就送给你吧。",
            "试下火眼金睛吧，你能看到怪物的信息和战斗损失的血量，你可以在物品栏中使用它。",
            "",
            "",
            "很好，你居然找到了我，作为奖励我将给你一瓶千年月桂露，喝了它将按你的攻击力和防御力的综合增加的你生命点数，你越晚使用它效果越好。",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "感谢你救了我，这是1000金就送给你吧。",
            "",
            "",
            "",
            "",
            "",
            "哈喽，送你1000金作为见面礼，记得经常来找我哦。",
            "",
    ];

    pub const MERCHANT_OFFERS: [&str; 13] = [
            "大圣饶命，我可以提升3%攻击力和防御力，需要小的为你效劳吗？",
            "",
            "大圣爷，我这里有1把蓝钥匙，你给50金币吧。",
            "我有5把黄钥匙，一共50金币",
            "嘿嘿，我有很多把黄钥匙，1把1000金币",
            "我有1把红钥匙，只要800金币",
            "我有1把蓝钥匙，只要200金币",
            "我跟其他人不一样，你可以把多余的钥匙卖给我。100金币回收1把黄钥匙",
            "我有1把黄钥匙，1把蓝钥匙，一共是1000金币",
            "我有3把黄钥匙，只要200金币",
            "我有3把蓝钥匙，收你2000金币",
            "我可以恢复你2000点血，不过要1000金币",
            "我有个宝物，要收你4000金币，是个玄明石。",
    ];

    pub const SIGN_TEXTS_B: [&str; 13] = [
            "",
            "",
            "天宫一共50层，每10层为一个区域。如果不打败该区域的头目就不能到更高的地方。",
            "在商店里你最好选择提升防御力，只有在攻击力低于敌人的防御力时才提升攻击力。",
            "",
            "你是否注意到5、9、14、16、18楼有的墙与众不同？",
            "如果你持有太公杖，面对天神力士和巨灵神时你的攻击力加倍。在没有太公杖的情况下你是无法打败巨灵神的。太公杖被藏在15楼以上的墙内。",
            "",
            "天宫一共有50层，但50楼并不能直接上去。",
            "存放乌金棍的房间的门坏了，你必须用金勺子破墙而入。",
            "天宫中藏有有个“幸运金币”拥有它在打败敌人后能够获得2倍的金钱。",
            "“紫金龙鳞甲”能防御“太上老君护卫”的夹击，但它被深藏在神秘的楼层中。",
            "如果要打败杨戬你需要“乌金棍”、“银缕锁甲”、“捆仙绳”或更高等级的宝物。",
    ];

    /// m_075（a.java:7472-7477）：报价入队。
    fn offer_queue_push(&mut self, src: i32, idx: i32) {
        let cap = (Self::OFFER_TEXTS_A.len() + Self::SIGN_TEXTS_B.len()) as i32;
        if (self.offer_queue_src.len() as i32) < cap {
            self.offer_queue_src.push(src as u8);
            self.offer_queue_idx.push(idx as u8);
        }
    }

    /// m_076（a.java:7479-7495）：游标钳位导航。
    fn offer_queue_seek(&mut self, cursor: i32) {
        let n = self.offer_queue_src.len() as i32;
        if n == 0 {
            return;
        }
        self.offer_queue_cursor = if cursor < 0 {
            n - 1
        } else if cursor >= n {
            0
        } else {
            cursor
        };
    }

    /// spendGold：够则扣减返回 true（a.java:7702 邻接族，m_083）。
    fn spend_gold(&mut self, amount: i32) -> bool {
        if self.player.gold >= amount {
            self.player.gold -= amount;
            true
        } else {
            false
        }
    }

    /// applyMerchantOffer（a.java:7307-7370）：buy=true 购买/false 拒绝。
    /// 返回 overlayActive 新值（true=浮层停留，false=关闭）。
    fn apply_merchant_offer(&mut self, buy: bool) -> bool {
        let param = self.entities.param[self.merchant_entity as usize] as i32;
        let flags = (param >> 8) as i8;
        let idx = param as i8;
        let src = if self.entities.entity_type[self.merchant_entity as usize] == 77 {
            0
        } else {
            1
        };
        let mut purchase_ok = true; // var5：购买/执行是否成功
        let mut follow_up = false; // var6：后续挂起（报价队列/失败提示浮层）
        if idx <= 0 {
            return false;
        }

        if buy {
            if flags & 2 != 0 {
            match (src, idx) {
                (0, 1 | 15 | 21) => {
                    crate::combat::gain_gold(&mut self.player, 1000);
                }
                (0, 2) => self.add_item_to_stack(13),
                (0, 6) => self.add_item_to_stack(19),
                (1, 1) => {
                    self.player.atk += self.player.atk * 3 / 100;
                    self.player.def += self.player.def * 3 / 100;
                }
                (1, 3) => {
                    if !self.spend_gold(50) {
                        self.show_overlay_message(0, "没有足够的金钱", 0, 0);
                        purchase_ok = false;
                    } else {
                        self.player.blue_keys += 1;
                    }
                }
                (1, 4) => {
                    if !self.spend_gold(50) {
                        self.show_overlay_message(0, "没有足够的金钱", 0, 0);
                        purchase_ok = false;
                    } else {
                        self.player.yellow_keys += 5;
                    }
                }
                (1, 5) => {
                    if !self.spend_gold(1000) {
                        self.show_overlay_message(0, "没有足够的金钱", 0, 0);
                        purchase_ok = false;
                    } else {
                        self.player.yellow_keys += 1;
                    }
                }
                (1, 6) => {
                    if !self.spend_gold(800) {
                        self.show_overlay_message(0, "没有足够的金钱", 0, 0);
                        purchase_ok = false;
                    } else {
                        self.player.red_keys += 1;
                    }
                }
                (1, 7) => {
                    if !self.spend_gold(200) {
                        self.show_overlay_message(0, "没有足够的金钱", 0, 0);
                        purchase_ok = false;
                    } else {
                        self.player.blue_keys += 1;
                    }
                }
                (1, 8) => {
                    if self.player.yellow_keys > 0 {
                        self.player.yellow_keys -= 1;
                        self.player.gold += 100;
                    } else {
                        self.show_overlay_message(0, "没有黄钥匙", 0, 0);
                        purchase_ok = false;
                    }
                }
                (1, 9) => {
                    if !self.spend_gold(1000) {
                        self.show_overlay_message(0, "没有足够的金钱", 0, 0);
                        purchase_ok = false;
                    } else {
                        self.player.yellow_keys += 1;
                        self.player.blue_keys += 1;
                    }
                }
                (1, 10) => {
                    if !self.spend_gold(200) {
                        self.show_overlay_message(0, "没有足够的金钱", 0, 0);
                        purchase_ok = false;
                    } else {
                        self.player.yellow_keys += 3;
                    }
                }
                (1, 11) => {
                    if !self.spend_gold(2000) {
                        self.show_overlay_message(0, "没有足够的金钱", 0, 0);
                        purchase_ok = false;
                    } else {
                        self.player.blue_keys += 3;
                    }
                }
                (1, 12) => {
                    if !self.spend_gold(1000) {
                        self.show_overlay_message(0, "没有足够的金钱", 0, 0);
                        purchase_ok = false;
                    } else {
                        self.player.hp += 2000;
                    }
                }
                (1, 13) => {
                    if !self.spend_gold(4000) {
                        self.show_overlay_message(0, "没有足够的金钱", 0, 0);
                        purchase_ok = false;
                    } else {
                        self.pickup_item_type(18);
                    }
                }
                _ => {}
            }
        }

            if purchase_ok {
                if flags & 4 == 0 {
                    self.entities.param[self.merchant_entity as usize] ^= 512;
                    if flags & 1 != 0 {
                        self.offer_queue_push(src, (idx - 1) as i32);
                        let text = if src == 0 {
                            Self::OFFER_TEXTS_A[(idx - 1) as usize]
                        } else {
                            Self::SIGN_TEXTS_B[(idx - 1) as usize]
                        };
                        self.show_overlay_message(4, text, 0, 0);
                        follow_up = true;
                    } else {
                        entity::remove(&mut self.entities, self.merchant_entity as usize);
                        self.build_minimap();
                    }
                }
            } else {
                // 购买失败（「没有足够的金钱」提示浮层已开）→ 浮层保持
                follow_up = true;
            }
        } else if flags & 2 == 0 && flags & 1 != 0 {
            // 拒绝路径（a.java:7453-7460）：^256（非 ^512）+ 非一次性则移除
            self.entities.param[self.merchant_entity as usize] ^= 256;
            if flags & 4 == 0 {
                entity::remove(&mut self.entities, self.merchant_entity as usize);
                self.build_minimap();
            }
        }
        follow_up
    }

    fn enter_menu16(&mut self) {
        self.options = Some(crate::menu_family::OptionList::new(
            self.sound_enabled,
            self.minimap_enabled,
        ));
        self.softkeys = (0, 3);
    }

    /// run case 16（a.java:4143-4188）：-7/-6 回 menuReturnMode；-5/-4/-3 翻转
    /// （label0 音乐→音源态翻转 + saveGlobalProgress；label1 小地图→null/rebuild）；
    /// -2/-1 光标环绕。
    fn tick_menu16(&mut self) {
        let Some(opt) = &mut self.options else { return };
        match self.key_value {
            -7 | -6 => self.game_mode = self.menu_return_mode,
            -5 | -4 | -3 => {
                let cur = opt.highlight as usize;
                opt.checked[cur] = !opt.checked[cur];
                match opt.labels[cur] {
                    0 => {
                        self.sound_enabled = !self.sound_enabled;
                        // saveGlobalProgress（RMS 侧，无像素效应；game-core 无 I/O）
                    }
                    1 => {
                        if !opt.checked[1] {
                            self.minimap = None;
                        } else if self.menu_return_mode == 4 {
                            self.build_minimap();
                        }
                    }
                    _ => {}
                }
            }
            -2 => {
                opt.highlight += 1;
                if opt.highlight > opt.labels.len() as i32 - 1 {
                    opt.highlight = 0;
                }
            }
            -1 => {
                opt.highlight -= 1;
                if opt.highlight < 0 {
                    opt.highlight = opt.labels.len() as i32 - 1;
                }
            }
            _ => {}
        }
    }

    /// cameraAtTarget（a.java:8205-8201）。
    fn camera_at_target(&self) -> bool {
        self.ease_cam_x == self.ease_target_x && self.ease_cam_y == self.ease_target_y
    }

    /// keyPressed（a.java:4663-4670）：keyValue = keyHeldCode = code。
    pub fn press_key(&mut self, code: i32) {
        self.key_value = code;
        self.key_held = code;
    }

    /// keyReleased（a.java:4678-4674）：只清 keyHeldCode。
    pub fn release_key(&mut self) {
        self.key_held = 0;
    }

    /// run case 3（a.java:3551-3679）：keyValue 开关（-7/-6 暂停/道具——
    /// 本窗口无）→ walkPhase 状态机 → advanceEntityFrames。
    /// `frame_counter` 为本拍值（TICK n → n-1）。
    pub fn tick(&mut self, frame_counter: i64, width_table: &[i32]) {
        self.frame_counter = frame_counter;
        // overlay 显示期（run 的 label510 分支 a.java:3252）：kind0 且
        // gameMode∈{15,17} 走 -7 专用支路（仅 -7 关层+回 menuReturnMode，
        // 其余键吞掉）；其余情况 handleOverlayKey（a.java:4923-4931：任意
        // 非翻页键关层）。**walkPhase 状态机与 m_055 冻结**。
        if self.overlay_active {
            if self.overlay_kind == 4 {
                // 商人/告示牌（overlay 分派 case4，a.java:3305-3320）：-6/-5=
                // applyMerchantOffer(true)，-7=(false)，-2/-1 翻页，其余关闭
                match self.key_value {
                    -7 => self.overlay_active = self.apply_merchant_offer(false),
                    -6 | -5 => self.overlay_active = self.apply_merchant_offer(true),
                    -2 => {
                        self.flip_overlay_page(self.overlay_page_top + self.overlay_lines_per_page)
                    }
                    -1 => {
                        self.flip_overlay_page(self.overlay_page_top - self.overlay_lines_per_page)
                    }
                    0 => {}
                    _ => self.overlay_active = false,
                }
            } else if self.overlay_kind == 5 {
                // 报价浏览（overlay 分派 case5，a.java:3325-3334）：-3/-4 游标、
                // -5/-6/-7 关浮层回 mode10
                match self.key_value {
                    -3 => self.offer_queue_seek(self.offer_queue_cursor - 1),
                    -4 => self.offer_queue_seek(self.offer_queue_cursor + 1),
                    -5 | -6 | -7 => {
                        self.overlay_active = false;
                        self.game_mode = 10;
                    }
                    _ => {}
                }
            } else if self.overlay_kind == 0 && (self.game_mode == 15 || self.game_mode == 17) {
                match self.key_value {
                    -7 => {
                        self.overlay_active = false;
                        self.game_mode = self.menu_return_mode;
                    }
                    -2 => {
                        self.flip_overlay_page(self.overlay_page_top + self.overlay_lines_per_page)
                    }
                    -1 => {
                        self.flip_overlay_page(self.overlay_page_top - self.overlay_lines_per_page)
                    }
                    _ => {}
                }
            } else if self.overlay_kind == 3 {
                // 道具确认浮层（overlay 分派 case3，a.java:3296-3306）：
                // -7 回面板、-6 关层 + useItemStack、其余 handleOverlayKey
                match self.key_value {
                    -7 => {
                        self.game_mode = 10;
                        self.overlay_active = false;
                    }
                    -6 => {
                        self.overlay_active = false;
                        self.use_item_stack(self.item_cursor as usize);
                    }
                    -2 => self.flip_overlay_page(self.overlay_page_top + self.overlay_lines_per_page),
                    -1 => self.flip_overlay_page(self.overlay_page_top - self.overlay_lines_per_page),
                    0 => {}
                    _ => self.overlay_active = false,
                }
            } else {
                // handleOverlayKey（a.java:4923-4931）：±2 翻页（flipOverlayPage
                // 钳制 a.java:5129-5140）、其余键关闭
                match self.key_value {
                    -2 => self.flip_overlay_page(self.overlay_page_top + self.overlay_lines_per_page),
                    -1 => self.flip_overlay_page(self.overlay_page_top - self.overlay_lines_per_page),
                    0 => {}
                    _ => self.overlay_active = false, // 任意其他键关闭
                }
            }
            // label510 块尾双清（a.java:3345）：keyValue 与 **keyHeldCode 同清**——
            // held 残留会让关层后首拍误触发 tryStep（floor2-tour T1238 实证）
            self.key_value = 0;
            self.key_held = 0;
            return;
        }
        if self.game_mode == 15 {
            // run case 15（fallthrough no-op）；浮层支路在 run 头消费键
            self.tick_menu15();
            if self.game_mode == 15 {
                self.key_value = 0;
            }
            return;
        }
        if self.game_mode == 5 {
            // run case 5（a.java:3737-3755）
            self.tick_menu5();
            self.key_value = 0;
            return;
        }
        if self.game_mode == 16 {
            // run case 16（a.java:4143-4188）
            self.tick_menu16();
            self.key_value = 0;
            return;
        }
        if self.game_mode == 7 {
            // run case 7（a.java:3829-3878）
            self.tick_menu7();
            self.key_value = 0;
            return;
        }
        if self.game_mode == 9 {
            // run case 9（a.java:3885-3940）
            self.tick_menu9();
            self.key_value = 0;
            return;
        }
        if self.game_mode == 4 {
            // run case 4（a.java:3690-3735），尾清 keyValue（a.java:4270）
            self.tick_menu4();
            self.key_value = 0;
            return;
        }
        if self.game_mode == 19 {
            // run case 19（a.java:4200-4226）
            self.tick_menu19();
            self.key_value = 0;
            return;
        }
        if self.game_mode == 10 {
            // run case 10（a.java:3931-4009）：键位导航，尾清 keyValue（a.java:4270）
            self.tick_item_panel();
            self.key_value = 0;
            return;
        }
        if self.game_mode == 11 {
            // run case 11 顺序（a.java:4009-4011）：advanceEntityFrames →
            // stepCameraTowardTarget → switch(dialogPhase)。**无 case 12 落穿**：
            // dialogPhase 各分支 break 的是外层 label534（a.java:3340），
            // a.java:4135 的 advanceBobPhase 自 mode 11 不可达——bob 仅随
            // paintEntityLayer（a.java:6499）每 paint 推进一次
            entity::advance_frames(
                &mut self.entities,
                &mut self.grid,
                frame_counter,
                width_table,
                &self.images.entity,
            );
            self.tick_dialog();
            self.key_value = 0;
            return;
        }
        // run case 3 头（a.java:3552-3562）：keyValue 开关。-7 → gameMode=4
        // （存档菜单，mode 4 批次待办）；-6 → gameMode=10 + m_000。
        match self.key_value {
            -7 => {
                // a.java:3559-3561：f_byte_02=gameMode → gameMode=4 + m_000
                self.in_game_menu_return_mode = self.game_mode;
                self.game_mode = 4;
                self.enter_menu4();
            }
            -6 => {
                self.game_mode = 10;
                self.enter_item_panel();
            }
            _ => {}
        }
        match self.walk_phase {
            0 => self.handle_field_input(),
            1 => {
                // 步进（a.java:3576-3601）：±8px + 相机边缘跟随 + m_025 帧
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
                // m_025（a.java:5201-5201）：行走帧循环
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
                // walkPhase 2（a.java:3610-3642）：easeAfterimages 残影蛇缓动 +
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
        // 组开门动画（paintTileLayer 头段 a.java:6992-7007，每拍推进）：
        // 18 拍置 removed 动画（row 6/state 1）；24 拍 removeGroupAtCellByType(4) 收尾
        if self.walk_phase == 4 {
            self.open_anim_tick += 1;
            if self.open_anim_tick == 18 {
                let group = self.open_group.clone();
                for e in group {
                    self.entities.anim_idx[e] = 6;
                    self.entities.solid[e] = 1;
                }
            } else if self.open_anim_tick >= 24 {
                let group = std::mem::take(&mut self.open_group);
                for e in group {
                    let gx = self.entities.pixel_x[e] >> 5;
                    let gy = self.entities.pixel_y[e] >> 5;
                    entity::remove_group_at_cell_by_type(gx, gy, 4, &mut self.entities, &mut self.grid);
                }
                self.open_anim_tick = 0;
                self.tile_bob = 0;
                self.walk_phase = 0;
            }
            // f_int_54 ±2 交替（a.java:7016-7014）
            self.tile_bob = if self.tile_bob == -2 { 2 } else { -2 };
        }
        entity::advance_frames(
                &mut self.entities,
                &mut self.grid,
                frame_counter,
                width_table,
                &self.images.entity,
            );
        // run 主循环尾部（a.java:4276，else 分支）：**keyValue 每拍清零**
        // （边沿触发；keyHeldCode 由 keyReleased 清——a.java:4678）
        self.key_value = 0;
    }

    /// easeAfterimages（a.java:5209-5256）：残影蛇缓动。目标 = 未锁定时的
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

    /// lockCameraOn（a.java:8666-8696）：缓动相机初始化。0 → 目标 = 玩家位。
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

    /// stepCameraTowardTarget（a.java:8704-8727）：1/4 距离 +2 缓动，
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

    /// handleFieldInput（a.java:5265-5327）：keyValue 特例（-5 战后视点/
    /// 49/55 楼梯换乘——本窗口无）+ keyHeld 移动（Nokia 映射：
    /// -1=上 -2=下 -3=左 -4=右）。
    fn handle_field_input(&mut self) {
        if self.game_mode == 11 {
            // mode 11 委托（a.java:5267-5269）：场输入在对话模式=脚本行走推进
            self.advance_script_walk();
            return;
        }
        if self.key_value == -5 || self.key_value == 53 {
            // -5（a.java:5275-5290）：残影位填充玩家位 + 各怪预测伤害数字 +
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
        if self.key_value == 49 || self.key_value == 55 {
            // 49/55（a.java:5293-5306）：暗道换乘键——f_bool_13（楼梯指示）
            // 点亮且存在无高层位的型7/8 门格（param>>9==0）→ changeFloor
            // （目标层 = param & 255）
            self.key_value = 0;
            let up = self.key_value == 49;
            if self.stair_hint {
                if let Some(e) = self.find_floor_gate_entity(up) {
                    let target = self.entities.param[e] as i32 & 255;
                    self.change_floor(target, !up, true);
                }
            }
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

    /// tryStep（a.java:5335-5356）：目标格 + interactWithCell 放行判定。
    pub fn try_step(&mut self, facing: i32) -> bool {
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
        } else if self.overlay_active && self.walk_phase == 1 {
            // a.java:5353-5358：overlay 内撞格中断 → 路径与相位一并清
            self.path_arrows_visible = false;
            self.walk_step_count = 0;
            self.walk_phase = 0;
        }
        ok
    }

    /// interactWithCell（a.java:5363-5531）：格槽**倒序**遍历 + 类目交互 +
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
                                    // 门（a.java:5391-5411）：耗钥（consumeKeyForDoor
                                    // 内部 spawnPopup kind1 图标 a.java:7733-7749）→
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
                                        self.overlay_kind_item = 0;
                                        self.show_overlay_message(
                                            0,
                                            match t {
                                                1 => "你没有黄钥匙",
                                                2 => "你没有红钥匙",
                                                _ => "你没有蓝钥匙",
                                            },
                                            0,
                                            0,
                                        );
                                        allowed = false;
                                    }
                                }
                                5 | 81 | 4 => {
                                    if t != 4 {
                                        // 封印门/障碍（a.java:5432-5432）：阻挡
                                        allowed = false;
                                    }
                                }
                                6 | 7 | 8 => stair_count += 1,
                                83 => {
                                // 场景门（a.java:5508-5510）：tryRunScene 门控 →
                                // loadLevelScript(mode 11)
                                if self.try_run_scene(self.entities.param[e] as i32, true) {
                                    self.overlay_active = false;
                                    self.overlay_text = None;
                                    let idx = self.entities.param[e] as i32;
                                    self.load_level_script(idx);
                                }
                            }
                            9 => {
                                // 炼丹台（a.java:5419-5429）：步进必阻挡；
                                // 朝上（facingDirection==1）触发 mode 9 + 价格/增量初始化
                                allowed = false;
                                if self.facing == 1 {
                                    self.game_mode = 9;
                                    self.alchemy_price =
                                        Self::alchemy_price_for(self.alchemy_upgrade_count + 1);
                                    self.alchemy_gain_hp = Self::scaled_by_floor_tier(100, self.floor);
                                    self.alchemy_gain_atk = Self::scaled_by_floor_tier(2, self.floor);
                                    self.alchemy_gain_def = Self::scaled_by_floor_tier(4, self.floor);
                                    // m_000()：mode 9 run 臂对无键进入无动作（a.java:3885 default）
                                }
                            }
                            _ => {} // 11 / 76/82：后续批次
                            },
                            8 => {
                                // 怪物（a.java:5470-5490）：可胜 → 场上战斗；否则阻挡
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
                            16 => {
                                // 告示牌/商人（a.java:5494-5517）：param 打包交互——
                                // 高字节=标志（&1=open 入队、&2=效果表、&4=一次性），
                                // 低字节=报价序号（1 基）；type77=文本源 0，其余=源 1；
                                // 触发即阻挡（var6=false）+ kind4 报价浮层开。
                                self.merchant_entity = e as i32;
                                let var6p = self.entities.param[e] as i32;
                                let flags = (var6p >> 8) as i8;
                                let mut offer = var6p as i8;
                                let src = if t == 77 { 0 } else { 1 };
                                if offer > 0 {
                                    offer -= 1;
                                    if flags & 2 != 0 {
                                        let text = if src == 0 {
                                            Self::OFFER_TEXTS_B[offer as usize]
                                        } else {
                                            Self::MERCHANT_OFFERS[offer as usize]
                                        };
                                        let (l, r) = if src == 1 { (1, 2) } else { (0, 0) };
                                        self.show_overlay_message(4, text, l, r);
                                    } else {
                                        self.offer_queue_push(src, offer as i32);
                                        let text = if src == 0 {
                                            Self::OFFER_TEXTS_A[offer as usize]
                                        } else {
                                            Self::SIGN_TEXTS_B[offer as usize]
                                        };
                                        let (l, r) = if src == 1 { (1, 2) } else { (0, 0) };
                                        self.show_overlay_message(4, text, l, r);
                                    }
                                }
                                allowed = false;
                            }
                            32 => allowed = false, // 联动门（a.java:5520）
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

        /// predictHpLossVsType（a.java:9983-9982）：effectiveAttackVsType 桥接。
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

        /// f_byte_arr_05 特性位（initEntityTables a.java:6266-6262：49/53/74=1、69=2）。
        fn trait_flags(&self, t: i32) -> u8 {
            match t {
                49 | 53 | 74 => 1,
                69 => 2,
                _ => 0,
            }
        }

        /// tickBattle（a.java:10023-10149）：4 拍一击 → 胜利 → 死亡动画 6 拍 →
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
                                // 击杀（a.java:10036-10062）：终伤预测 popup + 赏金 +
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
                                // 击杀钩子（a.java:10053-10070）：bossTypeOrder
                                // 击杀 → spawnBossEvent；型73 → 脚本旗标 +
                                // loadLevelScript(22) + 40 层 spawnBossEvent(6)；
                                // 型74 → 脚本 21；型75 → 脚本 20（重入场景）
                                let mut rerun = false;
                                if self.floor <= 50 {
                                    if let Some(bi) =
                                        BOSS_TYPE_ORDER.iter().rposition(|&b| b == t)
                                    {
                                        self.spawn_boss_event(bi);
                                    }
                                    match t {
                                        73 => {
                                            self.script_line_flags
                                                [self.current_script_index] = true;
                                            self.load_level_script(22);
                                            if self.floor == 40 {
                                                self.spawn_boss_event(6);
                                            }
                                        }
                                        74 => self.load_level_script(21),
                                        75 => {
                                            if self.try_run_scene(20, true) {
                                                self.load_level_script(20);
                                                rerun = true;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                self.battle_rerun_scene = rerun;
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
                    // 胜利后类型特判（61-72 层绑定剧情，a.java:10118-10144）：floor1 无
                }
            }
        }

        /// applyStepCellEffects（a.java:5543-5729，floor1 可达子集）：

        /// m_000 case 3 尾（a.java:4315 applyStepCellEffects）：进图瞬间出生格
        /// 触发（type83 → try_run_scene/loadLevelScript，序章 script 40 入口）。
        /// 槽载入链在 load_floor_from_save 尾内联；新游戏冷启动链（宿主构造
        /// 场景后）经此入口。
        pub fn apply_entry_step_effects(&mut self) {
            self.apply_step_cell_effects();
        }
        /// 玩家格坐标刷新 + 格内实体效果（拾取 cat2/4；楼梯 7/8 → 换层批次）+
        /// walkPhase=0 + handleFieldInput（按住连走）。
        fn apply_step_cell_effects(&mut self) {
            self.player_cell_x = self.view.player_px >> 5;
            self.player_cell_y = self.view.player_py >> 5;
            self.player_bob_applied = false; // f_bool_07 每步清零（a.java:5550），踩喷泉再置
            let ct = self.grid.cell_type[self.player_cell_y as usize][self.player_cell_x as usize] as usize;
            let cap = self.grid.capacity[ct] as usize;
            if cap > 0 {
                let mut s = cap as i32 - 1;
                while s >= 0 {
                let e = self.grid.slots[ct][s as usize] as usize - 1;
                let t = self.entities.entity_type[e];
                match entity::render_category(t) {
                    1 => match t {
                        6 => self.player_bob_applied = true, // f_bool_07（喷泉浮沉）
                        83 => {
                            // 出生/踩格触发（a.java:5708-5709）：序章 script 40 入口
                            if self.try_run_scene(self.entities.param[e] as i32, true) {
                                self.overlay_active = false;
                                self.overlay_text = None;
                                let idx = self.entities.param[e] as i32;
                                self.load_level_script(idx);
                            }
                        }
                        7 => {
                            // 上楼梯（a.java:5660-5659）：changeFloor(param, false, false)
                            let target = (self.entities.param[e] & 0xff) as i32;
                            self.change_floor(target, false, false);
                            self.key_held = 0;
                        }
                        8 => {
                            // 下楼梯（a.java:5666-5665）：changeFloor(param, true, false)
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
            // 型 5 宝箱 / 型 76 组门（cat 1 特例，a.java:5641-5705）：
            // 槽**倒序**遍历后置 s 重扫——Java 的 continue 语义（removeSlotAt 改槽表）
            let ct2 = self.grid.cell_type[self.player_cell_y as usize][self.player_cell_x as usize] as usize;
            let cap2 = self.grid.capacity[ct2] as usize;
            let mut s2 = cap2 as i32 - 1;
            while s2 >= 0 {
                let e = self.grid.slots[ct2][s2 as usize] as usize - 1;
                let t = self.entities.entity_type[e];
                if t == 5 {
                    // 宝箱（a.java:5641-5640）：removeSlotAt + tryOpenChest(param)
                    let chest_param = self.entities.param[e];
                    self.remove_slot_at(self.player_cell_x, self.player_cell_y, s2);
                    if self.try_open_chest(chest_param) {
                        self.walk_phase = 4;
                    }
                } else if t == 76 {
                    // 76 组门（a.java:5655-5705）：param&0xff + 1 为组号；摘槽 + 全图
                    // 收集同组型 4（开锁动画）+ 清组内 76 槽
                    let group = ((self.entities.param[e] & 0xff) as i32) + 1;
                    self.remove_slot_at(self.player_cell_x, self.player_cell_y, s2);
                    self.open_group.clear();
                    let mut slots_to_clear: Vec<(usize, u8)> = Vec::new();
                    for k in 0..self.entities.count {
                        if self.entities.removed[k] {
                            continue;
                        }
                        let p = self.entities.param[k] & 0xff;
                        let tk = self.entities.entity_type[k];
                        if tk != 76 {
                            if tk == 4 && p as i32 == group {
                                self.open_group.push(k);
                            }
                        } else if (p as i32) + 1 == group {
                            let gx = self.entities.pixel_x[k] >> 5;
                            let gy = self.entities.pixel_y[k] >> 5;
                            let ctk = self.grid.cell_type[gy as usize][gx as usize] as usize;
                            slots_to_clear.push((ctk, 0));
                            let cap3 = self.grid.capacity[ctk] as usize;
                            for q in 0..cap3 {
                                let id = self.grid.slots[ctk][q] as usize - 1;
                                self.entities.removed[id] = true;
                            }
                            self.grid.capacity[ctk] = 0;
                        }
                    }
                    if !self.open_group.is_empty() {
                        self.walk_phase = 4;
                    }
                }
                s2 -= 1;
            }

            if self.walk_phase != 4 {
                self.walk_phase = 0;
                self.handle_field_input();
            }
        } else if self.script_walk_armed {
            // 分支 2（a.java:5742-5746）：脚本行走武装 → advanceScriptWalk
            self.advance_script_walk();
        } else {
            // 分支 3（a.java:5747-5751）：空格 → 复位相位 + 场输入
            //（var4 踩伤短路 a.java:5645 未移植：伤害格批次待办，当前层无此格）
            self.walk_phase = 0;
            self.handle_field_input();
        }
        // label216（a.java:5748-5761）：路径消费/清除——顶步 == 朝向则消费一步，
        // 否则整条清空 + 熄箭头；步数已尽也熄箭头
        if self.path_arrows_visible {
            if self.walk_step_count > 0 {
                let top = self.walk_path_buffer[(self.walk_step_count - 1) as usize] as i32;
                if top == self.facing {
                    self.walk_step_count -= 1;
                } else {
                    self.walk_step_count = 0;
                    self.path_arrows_visible = false;
                }
            } else {
                self.path_arrows_visible = false;
            }
        }
    }

    /// advanceScriptWalk（a.java:5822-5838）：弹顶步 → tryStep；步尽解除武装；
    /// 撞格（返回 false）解除武装并清步（walkPhase==5 战斗中除外）。
    fn advance_script_walk(&mut self) {
        if self.walk_step_count > 0 {
            self.walk_step_count -= 1;
            self.facing = self.walk_path_buffer[self.walk_step_count as usize] as i32;
            let ok = self.try_step(self.facing);
            if self.walk_step_count == 0 {
                self.script_walk_armed = false;
                return;
            }
            if !ok {
                self.script_walk_armed = false;
                if self.walk_phase != 5 {
                    self.walk_phase = 0;
                    self.walk_step_count = 0;
                }
            }
        }
    }

    /// pickupItemType（a.java:7525-7621，floor1 可达子集）+ applyHpDelta
    /// （a.java:5172-5184，difficultyIndex=0 → 无 ×16）。
    /// floorTier（a.java:7642-7652）：((floor-1)/10) 钳负、floor>50 归 0，返回 +1。
    pub fn floor_tier(&self) -> i32 {
        let mut tier = (self.floor - 1) / 10;
        if tier < 0 {
            tier = 0;
        }
        if self.floor > 50 {
            tier = 0;
        }
        tier + 1
    }

    fn pickup_item_type(&mut self, t: i32) {
        // 描述表（itemDescriptions a.java:906 / equipDescriptions a.java:930——
        // 全量入库 reference；此处取用句柄）
        match t {
            13..=25 | 85 | 86 => {
                if t == 15 {
                    self.tutorial_seen = true; // f_bool_12
                }
                self.add_item_to_stack(t);
            }
            26 => self.player.yellow_keys += 1,
            27 => self.player.red_keys += 1,
            28 => self.player.blue_keys += 1,
            // 29/30（a.java:7556-7567）：floor ≤ 10 +1，否则 +floorTier()；
            // 31/32（a.java:7568-7577）：50/200 × floorTier()——floorTier =
            // ((floor-1)/10)+1（floor>50 钳 0，a.java:7642-7652）。迭代 31：
            // 原硬编码 tier=1 仅对 ≤10 层成立，floor12 实测 +50 应为 +100。
            29 => {
                if self.floor <= 10 { self.player.atk += 1 } else { self.player.atk += self.floor_tier() }
            }
            30 => {
                if self.floor <= 10 { self.player.def += 1 } else { self.player.def += self.floor_tier() }
            }
            31 => self.apply_hp_delta(50 * self.floor_tier()),
            32 => self.apply_hp_delta(200 * self.floor_tier()),
            33..=36 | 79 => {
                // 武器档（equip_types 1..5 槽）——m_083 档位 = 表下标
                let tier = match t {
                    33 => 1,
                    34 => 2,
                    35 => 3,
                    79 => 4,
                    _ => 5, // 36
                };
                if self.try_equip(tier, false) {
                    self.overlay_kind_pickup(t);
                } else {
                    self.show_overlay_message(0, "你拥有更强力装备", 0, 3);
                }
            }
            37..=40 | 80 => {
                let tier = match t {
                    37 => 7,
                    38 => 8,
                    39 => 9,
                    80 => 10,
                    _ => 11, // 40
                };
                if self.try_equip(tier, false) {
                    self.overlay_kind_pickup(t);
                } else {
                    self.show_overlay_message(0, "你拥有更强力装备", 0, 3);
                }
            }
            _ => {} // 41+ 怪物类型不可达（a.java:7608-7620 default return）
        }
    }

    /// addItemToItemStack（a.java:7702-7714）：栈顶追加或叠加次数 +
    /// showOverlayMessage(kind1, itemDescriptions[idx])。
    fn add_item_to_stack(&mut self, t: i32) {
        let idx = if t == 85 {
            20
        } else if t == 86 {
            21
        } else {
            (t - 13) as usize
        };
        const ITEM_USES: [i32; 22] = [-1, -1, -1, -1, 1, 1, 1, 3, 1, 1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let existing = self.player.find_item(t as u8);
        if existing < 0 {
            let n = self.player.item_stack_size as usize;
            if n < self.player.item_types.len() {
                self.player.item_types[n] = t as u8;
                self.player.item_uses[n] = ITEM_USES[idx] as u8;
                self.player.item_stack_size += 1;
            }
        } else {
            let u = self.player.item_uses[existing as usize];
            self.player.item_uses[existing as usize] = (u as i32 + ITEM_USES[idx]) as u8;
        }
        let text = self.item_description(idx);
        self.show_overlay_message(1, &text, 0, 3);
    }

    /// itemDescriptions[idx]（a.java:906-929 全量；截取本批可达项的原文）。
    fn item_description(&self, idx: usize) -> String {
        const D: [&str; 22] = [
            "能看破敌人底细，显示敌人详细信息。在游戏中按快捷键5也可以查看伤害量。\n\\c00ff00[使用次数：无限]",
            "记录前尘往事。\n\\c00ff00[使用次数：无限]",
            "在楼梯边，可以瞬间上下层，留神晕机。\n\\c00ff00[使用次数：无限]",
            "熄灭\\cFFCC33三昧真火\\r的神器。\n\\c00ff00[使用次数：无限]",
            "挖洞开墙越狱的利器,挫是挫了点，但是真的很好用。\n\\c00ff00[使用次数：1次]",
            "可以震开当前层所有的墙\n\\c00ff00[使用次数：1次]",
            "喝下后，增加相当于当前\\c00FFFF攻击力\\cFFFFFF加\\c00FFFF防御力\\cFFFFFF值740%的\\cFFCC00血量\n[月宫出品，手工酿制，不含三聚氰胺，冷藏效果更佳，使用次数：1次]",
            "瞬移到以中心为对称点的位置上。\n\\c00ff00[使用次数：3次]",
            "瞬移上行一层\n\\c00ff00[使用次数：1次]",
            "瞬移下行一层\n\\c00ff00[使用次数：1次]",
            "当年姜子牙受天命封神，他的钓鱼竹竿被原始天尊附上了神力，可以役使天神力士供他差遣，此杖又名“打神鞭”，对天神力士（包括巨灵神）威力加倍。\n\\cFFCC00[放在道具栏中有效]",
            "对某些自恋的神仙伤害加倍。\n\\cFFCC00[放在道具栏中有效]",
            "打怪得到的金钱加倍。\n\\cFFCC00[放在道具栏中有效]",
            "可以开启黄门。",
            "可以开启红门。",
            "可以开启蓝门。",
            "加攻击。",
            "加防御。",
            "加血。",
            "加血。",
            "开启当前层所有黄门",
            "如来开“慈悲为怀”巡回佛经演唱会的时候，伴奏罗汉用的乐器，道行浅的敌人，会被其梵天佛音瞬间化为灰飞\n\\cFFCC00使用：杀死上下左右的敌人，对BOSS不起作用。",
        ];
        D[idx].to_string()
    }

    /// 装备拾取成功浮层（equipDescriptions[tier]；a.java:7633-7633）。
    fn overlay_kind_pickup(&mut self, t: i32) {
        const D: [&str; 12] = [
            "",
            "\\cdddddd一根相当长的木制长棍,新手必备.有了它杀人越货不慌不愁.\n\\c00ff00装备: 攻击+10.\n",
            "\\cdddddd乌黑油亮，显然经历过多人之手。\n\\c00ff00装备: 攻击+30.\n\\cFFCC00\"很粗\"",
            "\\cdddddd银棍，恩，有这个名字就足够了。\n\\c00ff00装备: 攻击+70.\n\\cFFCC00\"只是\"",
            "\\cdddddd因乘天地之灵气，集日月之精华乃“万木之灵，灵木之尊”。\n\\c00ff00装备: 攻击+120.\n\\cFFCC00",
            "\\cdddddd您的需要，它知道；您的需求，它满足。它好，你也好，龙王后宫，镇宫之宝！\n\\c00ff00装备: 攻击+220.\n\\cFFCC00",
            "",
            "\\cdddddd没有太多的装饰，一件非常朴素、轻便的布衣.\n\\c00ff00装备: 防御+10.\n\\cFFCC00",
            "\\cdddddd保暖御寒，腰不酸，腿不疼，走路也有劲了。\n\\c00ff00装备: 防御+30.\n\\cFFCC00",
            "\\cdddddd如果没有上面的那行字，它也算是个杰作。\n\\c00ff00装备: 防御+70.\n\\cFFCC00",
            "\\cdddddd华丽的装饰，就是有点旧。\n\\c00ff00装备: 防御+120.\n\\cFFCC00\"别人穿过\"",
            "\\cdddddd东海龙鳞编织而成，限量版，天上天下，只此一款。\n\\cFFCC00",
        ];
        let tier = if t < 37 {
            match t { 33 => 1, 34 => 2, 35 => 3, 79 => 4, _ => 5 }
        } else {
            match t { 37 => 7, 38 => 8, 39 => 9, 80 => 10, _ => 11 }
        };
        self.show_overlay_message(1, D[tier as usize], 0, 3);
    }

    /// applyHpDelta（a.java:5172-5184）：负 → popup2；正 → popup3。
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
        if self.game_mode == 5 {
            self.paint_menu5(g);
            return;
        }
        if self.game_mode == 16 {
            // paint case 16（a.java:2963-2990）：parallax(false) + 图标条 +
            // paintBoxFrame + 选项行（OptionList::paint）
            self.paint_parallax(g, false);
            let mut f = self.strip_frame;
            crate::title::paint_icon_strip_at(g, &self.images.menu_strip, 86, 10, 34, 68, &mut f);
            self.strip_frame = f;
            if let Some(opt) = &self.options {
                let font = crate::paint::paint_font();
                opt.paint(g, &self.images.ui, &font);
            }
            m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1);
            return;
        }
        if self.game_mode == 7 {
            // paint case 7（a.java:2679-2682）：m_113(true) 存档变体
            self.paint_slot_list(g);
            self.paint_overlay_box(g);
            return;
        }
        if self.game_mode == 15 {
            // paint case 15（a.java:2958-2962）
            self.paint_menu15(g);
            return;
        }
        if self.game_mode == 4 || self.game_mode == 19 {
            // paint case 4/19 共享（a.java:2427-2490）
            self.paint_menu_list(g);
            self.paint_overlay_box(g);
            return;
        }
        if self.game_mode == 10 {
            self.paint_item_panel(g);
            self.paint_overlay_box(g);
            // 面板内道具触发 changeFloor：遮幅公共尾同样共跑（a.java:3177-3215）
            self.paint_wipe_tail(g);
            return;
        }
        if self.game_mode == 11 {
            self.paint_dialog(g, tileset);
            return;
        }
        if self.game_mode == 9 {
            // paint case 9（a.java:2690-2740）
            self.paint_menu9(g, tileset);
            return;
        }
        self.paint_parallax(g, true);
        let tile_off = view_top_y() + if self.walk_phase == 4 { self.tile_bob } else { 0 };
        self.view.paint_tiles(g, tileset, tile_off);
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
        // paint case 3 内断言软键 (1,3)（a.java:2421-2422）——菜单族改写的
        // softkey 落回 mode 3 时由 paint 复位（Java 字段赋值语义）
        self.softkeys = (1, 3);
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1);
        // 提示浮层（paint 公共尾 a.java:3129-3168，!f_bool_16 && overlayActive）
        if !self.transitioning && self.overlay_active {
            self.paint_overlay_box(g);
        }
        self.paint_wipe_tail(g);
    }

    /// 换层遮幅（paint 公共尾 a.java:3177-3215——**所有 mode 后置共跑**，
    /// 非 mode3 专属）：状态机先行、后画黑格。闭合（wipe_closing）1..=4
    /// 全黑 → 中段换层（swap_floor）→ 开启 4..=0。mode10 面板内道具触发
    /// changeFloor 同样经过此尾（迭代 45 T401 实证缺口）。
    fn paint_wipe_tail(&mut self, g: &mut SoftGraphics<'_>) {
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

    /// drawParallaxBackdrop（a.java:6850-6871）：滚动 -1/拍（PARALLAX_WRAP 回绕），
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

    /// paintEntityLayer（a.java:6491-6717）：实体层。`cam_x/cam_y` 为调用参数
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
                        // 这些类型占据插入窗口时不插画玩家——a.java:6520）
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
                                    // 战斗目标抖动（a.java:6543-6541）：x/y 各随机 ±2
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
                                // walkPhase==2 战后数字条（a.java:6617-6638）：
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
                                    // 型 72 非追踪实体：静态（无 bob），y-(h-32)（a.java:6657-6652）
                                    draw_image_clipped(g, img, (sx + cx_off), sy - (h - CELL_PX), frame_off, 0, w, h);
                                } else {
                                    // 追踪实体：形态号帧（f_byte_19；==3 时镜像，a.java:6660-6661）
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
        // walkPhase==5 星光（a.java:6692-6704）：battleTick&7 选帧，sparkFrames
        // 帧号×4 取 sparkQuads 四联组（sx,sy,w,h）
        if self.walk_phase == 5 && self.battle_target >= 0 {
            let e = self.battle_target as usize;
            // a.java:6692-6687：星光坐标用**裸 cameraPixelX/Y**（非 m_053 入参
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
        // 楼梯浮标（paintEntityLayer 尾段 a.java:6712-6720）
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

    /// paintPlayerSprite（a.java:5841-5909）：玩家。facing 0/1/2/3 = 下/上/右/左行
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
        // walkPhase==2 残影（a.java:5860-5904）：facing 1 在主精灵**后**画，
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
        // f_bool_26 路径箭头（a.java:5929-5956）：自玩家格沿 walkPathBuffer 逐格外扩，
        // buffer 顶 = 玩家侧第一步；[8][22] 三联贴片 21×22（0 下/1 上裁剪源、2 右镜像、3 左），
        // 0/1 的 bob 加在 y、2/3 加在 x（Java 原样）；setColor(136) 每步重设。
        if self.path_arrows_visible {
            let mut ax = (self.player_cell_x << 5) + cam_x;
            let mut ay = (self.player_cell_y << 5) + cam_y;
            let bob = self.bob_offset;
            let piece = &self.images.ui[22];
            for i in (0..self.walk_step_count as usize).rev() {
                g.set_color(136);
                match self.walk_path_buffer[i] {
                    0 => {
                        ay += 32;
                        draw_image_clipped(g, piece, ax + 5, ay + 5 + bob, 21, 0, 21, 22);
                    }
                    1 => {
                        ay -= 32;
                        draw_image_clipped(g, piece, ax + 5, ay + 5 + bob, 0, 0, 21, 22);
                    }
                    2 => {
                        ax += 32;
                        crate::menu_family::draw_edge_patch(g, piece, ax + 5 + bob, ay + 5, 42, 0, 21, 22, 1);
                    }
                    3 => {
                        ax -= 32;
                        draw_image_clipped(g, piece, ax + 5 + bob, ay + 5, 42, 0, 21, 22);
                    }
                    _ => {}
                }
            }
        }
        // walkPhase==2 四角装饰（a.java:5959-5959，画后直接 return 语义）
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

    /// walkPhase==2 的四向残影（a.java:5860-5904）。
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

    /// paintHudFrame（a.java:6134-6157）：HUD 大框（ui[8][0] 边框件 64×16）。
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

    /// paintHudPanel（a.java:6052-6106）：HUD（头像/HP/攻/防/武器/甲）。
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
        // 槽 x：var11 = HUD_ICON_X+16 后 +60 → 159；甲槽 +34 → 193（a.java:6080/6087）
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

    /// paint case 11（a.java:2798-2896）：世界层 + 对话框/立绘/气泡/翻页箭头。
    /// overlayText==null 或 overlayActive 或 phase==4 时走 paintHudPanel 分支。
    fn paint_dialog(&mut self, g: &mut SoftGraphics<'_>, tileset: &ArgbImage) {
        self.paint_parallax(g, true);
        let tile_off = view_top_y() + if self.walk_phase == 4 { self.tile_bob } else { 0 };
        self.view.paint_tiles(g, tileset, tile_off);
        self.paint_entities(g, self.view.cam_x, self.view.cam_y + view_top_y());
        self.popups.draw(g, &self.images.popup);
        self.paint_status_bar(g);
        let show_box = self.overlay_text.is_some() && !self.overlay_active && self.dialog_phase != 4;
        if show_box {
            let font = crate::paint::paint_font();
            let line_h = font.height + 4;
            let var91 = (line_h << 1) + 12;
            let var120 = SCREEN_H - var91;
            self.paint_hud_frame(g, 0, var120, 80, var91);
            let mut cx = 80;
            while cx < 214 {
                g.set_clip(cx, var120, 16, 16);
                g.draw_image(&self.images.ui[0], cx - 26, var120, 0);
                cx += 16;
            }
            g.set_clip(0, 0, SCREEN_W, SCREEN_H);
            // a.java:2828：edge patch 用 var48=var120（+=16 之前）
            crate::menu_family::draw_edge_patch(g, &self.images.ui[0], 214, var120, 0, 0, 26, 16, 1);
            g.set_color(HUD_FILL);
            g.fill_rect(80, var120 + 16, 149, var91 - 16);
            // a.java:2832-2836：右条带循环 (229, var48) 画 ui[8][0]@176——
            // var48 从 var120+16 步进 16 直到 var158+var11（320）
            let mut var48 = var120 + 16;
            while var48 < var120 + var91 {
                g.set_clip(229, var48, 11, 16);
                g.draw_image(&self.images.ui[0], 176, var48, 0);
                var48 += 16;
            }
            g.set_clip(0, 0, SCREEN_W, SCREEN_H);
            if self.dialog_speaker > 0 {
                if let Some(img) = &self.images.entity[self.dialog_speaker as usize] {
                    if self.dialog_speaker != 72 {
                        let fw = self.type_frame_width_for(self.dialog_speaker);
                        draw_image_clipped(g, img, (80 - fw >> 1), SCREEN_H - img.height, 0, 0, fw, img.height);
                    } else {
                        draw_image_clipped(g, img, 24, 275, 0, 0, 32, 45);
                    }
                }
            } else if self.dialog_speaker == 0 {
                let face = &self.images.actor_face;
                g.draw_image(face, 40 - (face.width >> 1), SCREEN_H - face.height, 0);
            } else {
                g.set_color(WHITE);
                let q: Vec<u16> = "?".encode_utf16().collect();
                g.draw_string(&q, 40, var120 + (var91 - font.height >> 1), 17);
            }
            g.set_color(10473684);
            self.paint_wrapped_text(g, 85, var120 + 12, 129, var91 - 12, false);
            if self.overlay_page_top > 0 {
                draw_image_clipped(g, &self.images.ui[15], 220, SCREEN_H - line_h - 10, 0, 0, 7, 9);
            }
            if self.overlay_page_bottom < self.overlay_total_lines {
                draw_image_clipped(g, &self.images.ui[15], 220, SCREEN_H - line_h + 1, 7, 0, 7, 9);
            }
            if (self.dialog_phase == 6 || self.dialog_phase == 1)
                && self.dialog_page >= 0
                && (self.dialog_page as usize) < self.dialog_data.bubbles.len()
            {
                let bubble = self.dialog_data.bubbles[self.dialog_page as usize];
                if bubble >= 0 {
                    let (bx, by) = if self.dialog_anchor >= 0 {
                        let e = self.dialog_anchor as usize;
                        (self.entities.pixel_x[e] + 16, self.entities.pixel_y[e] - 30)
                    } else {
                        (self.view.player_px + 16, self.view.player_py - 30)
                    };
                    let by = if self.dialog_speaker == 84 { by - 20 } else { by };
                    // a.java:2887：标记图 = face 包第 bubble 张（26×22），非实体表
                    if let Some(face) = self.images.faces.get(bubble as usize) {
                        g.draw_image(face, bx + self.view.cam_x, by + self.view.cam_y + 20, 0);
                    }
                }
            }
            if self.dialog_phase == 6 {
                if let Some(arrow) = &self.images.map211 {
                    g.draw_image(
                        arrow,
                        self.view.cam_x + 5 + (self.dialog_target.0 << 5),
                        self.view.cam_y + (self.dialog_target.1 << 5) - 12 + self.bob_offset,
                        0,
                    );
                }
            }
        } else {
            self.paint_hud(g);
        }
    }

    /// typeFrameWidth[type]（type_width_px 夹具的 i32 视图）。
    fn type_frame_width_for(&self, t: i32) -> i32 {
        self.width_table.get(t as usize).copied().unwrap_or(32)
    }

    /// paintStatusBar（a.java:6018-6035）：顶部状态栏（层数/三钥/金币）+ paintKeySlot。
    pub fn paint_status_bar(&mut self, g: &mut SoftGraphics<'_>) {
        let base_y = 0;
        let font = crate::paint::paint_font();
        g.set_clip(0, 0, SCREEN_W, SCREEN_H);
        g.draw_image(&self.images.ui[3], 0, base_y, 0);
        if self.floor > 50 {
            // "引子" outline 文本（m_142 a.java:4487-4495）：anchor 17 =
            // TOP(16)|HCENTER(1) 顶部居中——8 向偏移点各画一次全串（色[0]）+
            // 中心 fill（色[1]）。shim 常量：VCENTER=2 非 16！
            let text: Vec<u16> = "引子".encode_utf16().collect();
            let x0 = 32 - 1;
            let y0 = base_y + 9 + ((19 - font.height) >> 1) - 1;
            let outline = [4202520u32, WHITE];
            let deltas = [(1i32, 0i32), (1, 0), (0, 1), (0, 1), (-1, 0), (-1, 0), (0, -1), (0, -1)];
            let mut pts = Vec::new();
            let (mut x, mut y) = (x0, y0);
            for (dx, dy) in deltas {
                x += dx;
                y += dy;
                pts.push((x, y));
            }
            pts.push((x0 + 1, y0 + 1)); // 中心 fill（var2++→32、var3 循环回 y0+1）
            for (i, (px, py)) in pts.iter().enumerate() {
                g.set_color(if i == 8 { outline[1] } else { outline[0] });
                g.draw_string(&text, *px, *py, 17);
            }
        } else {
            paint_number(g, &self.images.ui[18], self.floor, STATUS_FLOOR_NUM_X, base_y + 4 + 2);
            g.draw_image(&self.images.ui[8], STATUS_TIER_ICON_X, base_y + 4 + 10, 0);
        }
        g.set_color(STATUS_FILL);
        g.fill_rect(STATUS_KEY_X[0], base_y, SCREEN_W - STATUS_KEY_X[0], STATUS_BAR_H);
        self.paint_key_slot(g, 0, self.player.yellow_keys, STATUS_KEY_X[0], base_y);
        self.paint_key_slot(g, 1, self.player.blue_keys, STATUS_KEY_X[1], base_y);
        self.paint_key_slot(g, 2, self.player.red_keys, STATUS_KEY_X[2], base_y);
        let y = base_y + 3;
        g.draw_image(&self.images.ui[1], STATUS_GOLD_ICON_X, y, 0);
        paint_number(g, &self.images.ui[2], self.player.gold, STATUS_GOLD_NUM_X, y + 2);
    }

    /// paintKeySlot（a.java:6043-6044）：钥匙格（ui[8][6] KEY_SLOT_W 三联图 + 数量）。
    fn paint_key_slot(&self, g: &mut SoftGraphics<'_>, kind: i32, count: i32, x: i32, y: i32) {
        g.set_clip(x, y, KEY_SLOT_W, KEY_SLOT_H);
        g.draw_image(&self.images.ui[6], x - kind * KEY_SLOT_W, y, 0);
        g.set_clip(0, 0, SCREEN_W, SCREEN_H);
        paint_number(g, &self.images.ui[2], count, x + KEY_NUM_DX, y + KEY_NUM_DY);
    }
}

/// blit5Clip（a.java:4557-4556）：MINIMAP_MARKER_PX 定尺寸裁剪贴图（小地图标记）。
fn m_003_clip5(g: &mut SoftGraphics<'_>, img: &ArgbImage, x: i32, y: i32, src_x: i32) {
    g.set_clip(x, y, MINIMAP_MARKER_PX, MINIMAP_MARKER_PX);
    g.draw_image(img, x - src_x, y, 0);
    g.set_clip(0, 0, SCREEN_W, SCREEN_H);
}
