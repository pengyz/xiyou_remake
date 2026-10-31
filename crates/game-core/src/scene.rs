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

use crate::enums::GameMode;
use crate::entity::{self, CellGrid, EntityTable, ANIM_OFFSET_TABLE};
use crate::game_view::GameView;
use crate::layout::*;
use crate::menu_family::{paint_mini_frame, paint_number};
use crate::paint::{draw_image_clipped, m_034_softkeys};
use crate::render::FontMetrics;
use crate::render::{ArgbImage, SoftGraphics};

/// f_Image_arr2_00 的 16 容器名序已迁 `sheet_index.containers`（a.java:451 名序值行；
/// 张数表锚勘误=a.java:453 非 451）——查表入口 `resource_cfg::ResourceCfg::container_names`。
/// 本场景常驻引用见 [`SceneImages`]。
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
    /// f_Image_arr2_00[14][0]：end 包结局卷轴（240×145，loadPackedImages(14) 迭代 57）
    pub ending: ArgbImage,
    /// f_Image_arr2_00[9][*]：xtq 包（6 张；[5] = 型69 拼装效果帧条 26×13）
    pub xtq: Vec<ArgbImage>,
}

impl SceneImages {
    /// m_122 头部的类型→容器映射（a.java:9448-9470）。
    /// 【V2-P2 消费替换】图集数字（容器/槽下标）改查 `sheet_index.slots.claims`
    /// （82 条 claim 覆盖 85 型；0/82/83/88 无臂=无图）——旧体 5 区间+9 特例
    /// +2 别名（12=11、71=70）逐值等价，冻结副本对拍见
    /// tests/sheet_index_table_driven.rs::task7_map_entity_images_matches_legacy_oracle。
    pub fn map_entity_images(
        sptmap: &[ArgbImage],
        sptprop: &[ArgbImage],
        sptarm: &[ArgbImage],
        sptenemy1: &[ArgbImage],
        sptenemy2: &[ArgbImage],
    ) -> Vec<Option<ArgbImage>> {
        let cfg = crate::resource_cfg::global();
        let mut out: Vec<Option<ArgbImage>> = vec![None; 89];
        for (t, slot) in out.iter_mut().enumerate() {
            let Some((container, si)) = cfg.entity_sprite_slot(t as i32) else {
                continue;
            };
            let src: &[ArgbImage] = match container {
                "sptmap" => sptmap,
                "sptprop" => sptprop,
                "sptarm" => sptarm,
                "sptenemy1" => sptenemy1,
                "sptenemy2" => sptenemy2,
                other => panic!("m_122 实体容器域外：{other:?}"),
            };
            *slot = src.get(si).cloned();
        }
        out
    }
}

/// 玩家帧表 playerAnimTables[0]（a.java:477：{0,1,0,2}）。
const PLAYER_FRAME_TABLE: [i32; 4] = [0, 1, 0, 2];

/// bossTypeOrder（a.java:2198）：boss 型 → bossEventSpawns 事件序（击杀钩子）。
const BOSS_TYPE_ORDER: [i32; 6] = [75, 74, 68, 67, 69, 71];
/// bossEventSpawns（a.java:2041-2104）：每事件 = [型, 格x, 格y, param]×N。
const BOSS_EVENT_SPAWNS: [&[i32]; 8] = [
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
    &[
        32, 4, 4, 0, 32, 5, 4, 0, 32, 6, 4, 0,
    ],
];
/// 战斗星光帧号表（Java sparkFrames，a.java:766-773：8 拍循环，0 = 不画）。
const SPARK_FRAME: [i32; 8] = [1, 2, 0, 3, 4, 1, 2, 0];
/// 星光四联组 (sx,sy,w,h)×4（Java sparkQuads，a.java:748-764）。
const SPARK_QUADS: [i32; 16] = [29, 31, 39, 15, 0, 31, 29, 10, 0, 0, 37, 31, 36, 0, 32, 29];

/// blocksPlayerInsert（initEntityTables 从构造字面量 noInsertTypeFlags 拷贝，a.java:679/
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

/// 【STAGE-B-008 · spec §6】点地反馈色（modern 专用）：目标 / 选中框金、闪红红、选中闪烁的另一相位橙。
const TAP_TARGET_RGB: u32 = 0xFFE070;
const TAP_REJECT_RGB: u32 = 0xFF3030;
const TAP_SELECT_ALT_RGB: u32 = 0xFF8040;

/// 对白数据集（构造期从夹具装载；game-core 无 I/O——include_str 编译期）。
/// miscTexts（a.java:438-445 初始化器逐字）：[0-3]=changeFloor 提示浮层，
/// [4-7]=序章卷轴文本（[5]=装备丢弃提示）。
pub const MISC_TEXTS: [&str; 8] = [
    "无法再下一层了，这是你达到的最底层",
    "无法再上一层了，这是你达到的最高层",
    "无法再下去了。",
    "无法再上去了。",
    "俺，当世神界第一斗者，孙!悟!空! 自从受封为齐天大圣，掌管蟠桃园以来，一直逍遥快活，无拘束……",
    "你拥有更强力的装备，因此将捡到的丢弃了。",
    "直到那一天，遇到了她，在筋斗云上的我，竟然第一次心潮起伏，有了晕机的感觉……",
    "神仙动了感情，往往会万劫不复，\n这一次，让我付出了五百年的时间去忘记她……\n五指山脚下的沙子，掠过我的脸庞。\n沙子，跟时间一样，同样随风流逝；同样掩埋过去；\n多少次伸手想抓住，却从指隙溜走……\n看夜空，半梦半醒间，往事历历上心头……",
];

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
    /// 布局面（STAGE-B-002）：classic=原版恒等（默认）；modern=公式派生能力面。
    /// 消费方（shell 缩放适配）在交接时按预设改写。
    pub layout: crate::layout::Layout,
    /// SKY_WAR 全局存档（saveGlobalProgress a.java:8917-8942；两机同构拷贝）。
    pub global: crate::save::GlobalSave,
    /// 全局态脏位（壳轮询持久化）。
    pub global_dirty: bool,
    /// frameCounter（run 循环变量，TICK n 拍 = n-1）
    pub frame_counter: i64,
    /// advanceBobPhase 浮沉相位（paint 内推进）
    pub bob_offset: i32,
    pub bob_rising: bool,
    /// drawParallaxBackdrop 的滚动相位（每 paint -1，PARALLAX_WRAP 回绕）
    pub backdrop_scroll: i32,
    /// viewBottom：视口底（HUD 顶）——初始化后与 layout::view_bottom_y() 一致
    pub view_bottom: i32,
    /// 玩家态（paintPlayerSprite 输入）
    pub facing: i32,
    pub walk_phase: i32,
    pub player_frame: i32,
    pub player_bob_applied: bool, // player_bob_applied
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
    /// alchemyGainHp/78/79：本次升级 HP/ATK/DEF 增量（scaledByFloorTier，a.java:7250）
    pub alchemy_gain_hp: i32,
    pub alchemy_gain_atk: i32,
    pub alchemy_gain_def: i32,
    /// alchemyCursor：三行选择光标（0=HP 1=ATK 2=DEF）
    pub alchemy_cursor: i32,
    /// alchemyUpgradeCount：运行时升级计数（存档 alchemy_upgrades 装入）
    pub alchemy_upgrade_count: i32,
    /// —— 商人/告示牌链（interact case16 a.java:5494-5517 + overlay 分派
    ///     case4/5 a.java:3300-3334 + applyMerchantOffer a.java:7307-7370）——
    /// merchantEntity：最近 interact 命中的实体下标
    pub merchant_entity: i32,
    /// 报价队列（pushOfferQueue/browseOfferQueue，f_byte_arr_14/15 + offerQueueSrc/89/90）：(文本源, 报价序号)
    pub offer_queue_src: Vec<u8>,
    pub offer_queue_idx: Vec<u8>,
    pub offer_queue_cursor: i32,
    /// 软键（dispatchModeEntry case 3：1/3）
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
    /// walkStepCount（pathArrowsVisible 路径步数，行走扩展批次）
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
    /// floorTransitioning：遮幅进行中
    pub transitioning: bool,
    /// wipeClosing：闭合相位（构造 true；全黑后翻 false 开启）
    pub wipe_closing: bool,
    /// wipePhase：遮幅计数 0..=4
    pub wipe: i32,
    /// pendingFloor：目标层
    pub pending_floor: i32,
    /// wentDown：下楼标记（型7 上楼传 false）
    pub went_down: bool,
    /// minFloorReached/maxFloorReached
    pub min_floor: i32,
    pub max_floor: i32,
    /// route_entity：dp2 路径缓动实体索引（MOV 五参 a.java:8289 写 findBlockingEntity（原 findBlockingEntity）结果——
    /// 未命中亦写 -1；openWallOnRoute 收尾/战斗收尾写 -1）。paintEntityLayer case 32 型 72
    /// "追踪中"分支（a.java:6660）读之。Java int 字段默认 0（首个 MOV 前恒 0）。
    pub route_entity: i32,
    /// f_byte_19：dp2 当前步方向（openWallOnRoute case0 弹自 walkPathBuffer，a.java:8742：
    /// 0 下/1 上/2 右/3 左）——兼作型 72 追踪帧行号（a.java:6663：==3 镜像）。
    pub route_dir: i32,
    /// routePhase（原 routePhase，台账 N-0303）：openWallOnRoute 子相位（0 弹步 / 1 8px 步进 /
    /// 5 途中战斗；MOV 五参置 0）
    pub route_phase: i32,
    /// routeProgress（原 routeProgress，台账 N-0306）：openWallOnRoute 步内像素累计（8/拍，==32 归格；
    /// 仅归格清零，MOV 不重置）
    pub route_progress: i32,
    /// f_int_129/f_int_130：路径起点格（MOV 五参 a.java:8297-8298 写；relocateSlotsAtRouteEnd 迁槽的
    /// "旧格"——全程不随实体移动更新）
    pub route_start: (i32, i32),
    pub current_floor: i32,
    /// 层资产表（按层号索引）
    pub floor_assets: Vec<Option<FloorAssets>>,
    /// f_byte_arr2_05：层状态存档（m_119 写/m_122 读；索引=层号）
    pub saved_floor_states: Vec<Option<Vec<u8>>>,
    /// equippedWeaponType/ArmorType（resetNewGameState 重置；tryEquip 更新；HUD 槽显示）
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
    /// openAnimTick：组开门动画计数（18 置 removed 动画、24 收尾）
    pub open_anim_tick: i32,
    /// tileBob：遮幅纵向抖动（±2 交替）
    pub tile_bob: i32,
    /// tutorialSeen：教程标记（型 15 拾取置 true——f_bool_13 楼梯指示的门控）
    pub tutorial_seen: bool,
    /// f_bool_13：楼梯指示（49/55 暗道换乘键门控；proximity 更新——
    /// lockCameraOn/行走批次——未移植，稳态 false 与 Java 稳态一致）
    pub stair_hint: bool,

    // —— mode 11 对话（gameMode 3=世界 / 11=对话；a.java:4011-4068 相位机）——
    pub game_mode: GameMode,
    /// 转移计数（件 A' 收口观测：transition_to 调用次数）
    pub transition_count: u32,
    /// 转移日志（件 A' 收口观测：最后一次 (from, to)）——观测字段，不进金样 dump
    pub last_transition: Option<(GameMode, GameMode)>,
    /// dialogPhase：0 脚本步进 / 1 TAK 打字 / 2 实体路径 / 3 脚本行走 / 4 等相机 / 6 SEE 打字
    pub dialog_phase: i32,
    /// mode12 日志浏览（recordPage 页 + f_String_04 页码指示 + 日志对）
    pub record_page: i32,
    /// f_String_arr_06/09（a.java:823/887 init 常量；fixtures 迭代 59）
    pub record_text_a: Vec<String>,
    pub record_text_b: Vec<String>,
    pub page_indicator: String,
    pub journal: Vec<(i32, i32)>,
    /// mode20 结局卷轴状态（Java f_int_22/f_int_23/f_bool_04——dispatchModeEntry case20
    /// **不重置**，场景构造期零值即 Java 新运行初值）。
    pub ending_scroll: i32,
    pub ending_anim: i32,
    pub ending_done: bool,
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
    /// menuReturnMode（N-0240 menuReturnMode）：mode7/8 -7 的返回目标
    pub menu_return_mode: GameMode,
    /// mode 7 存档槽状态（dispatchModeEntry case7 布局 + loadSlotTable 冷态）
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
    /// 【计划 3 T7】帮助文本覆盖（壳注入；交接不清除=随档持久，读档/换层不影响）。
    pub help_override: Option<String>,
    /// 【STAGE-B-006】壳层附加设置行（交接自 BootMachine，回标题时交回；空 = 原版两行）：设置页入口据此生成第 3 行起的
    /// 附加行，切换回写此处。core 不解释其含义。
    pub shell_options: Vec<crate::menu_family::ShellOption>,
    /// 音频语义（crate::audio；docs/spec/audio.md）。soundEnabled 的唯一真相源
    ///（run case16 label0 翻转，a.java:4157）。独立构造默认 = 「刚进 mode 3、声音开」；
    /// 宿主从 BootMachine 交接时整体覆盖（Java 为同一对象，状态连续）。
    pub bgm: crate::audio::BgmState,
    /// 虚拟 RMS：MOT_L{slot} 槽位 blob（saveFloorState 写侧；game-core 无 I/O）
    pub saved_slot_blobs: [Option<Vec<u8>>; 6],
    /// shopRainbowPhase：商店条目彩虹色相位（paint 每 4 拍推进）
    pub rainbow_phase: i32,
    /// iconStripFrame（a.java:464 全局；mode16 图标条动画帧）
    pub strip_frame: usize,
    /// inGameMenuReturnMode：菜单返回目标（run case3 -7 时存 gameMode）
    pub in_game_menu_return_mode: GameMode,
    /// mode 10 道具面板布局（dispatchModeEntry case10 a.java:4370-4380：itemPanelCols..104）
    pub item_cols: i32,
    pub item_rows_total: i32,
    pub item_rows_visible: i32,
    pub item_scroll_top: i32,
    pub item_scroll_bottom: i32,
    pub item_box_w: i32,
    pub item_box_h: i32,
    pub item_cursor: i32,
    /// pathArrowsVisible：路径箭头可见（SEE flag=1 且 findPath 成功）
    pub path_arrows_visible: bool,
    /// walkPathBuffer（findPath 步序，目标端优先）
    pub walk_path_buffer: Vec<u8>,
    /// 【T2】操作级回撤 journal（T1 模块；只在 WalkTo/探步寻路段落笔——经典面零写入）
    pub undo: crate::undo_journal::UndoJournal,
    /// 【T2】WalkTo 路段追踪器（位移笔=段起终点/中断挂起=实际终点/战斗笔=终结时）
    pub route: crate::pathfind::RouteTracker,
    /// 【BUG-040 · T2】点地寻路被打断、但 dp3 仍归寻路所有（走完当前步即回 Map）。打断会收束
    /// 寻路段（route 不再开放），dp3 收尾原本只凭 `route.is_open()` 认寻路 ⇒ 打断后落入脚本机
    /// dp0 行尾收尾。置位 = interrupt_route 时处于 Dialog dp3；清位 = 新 walk_to / 剧情接管 / 回 Map。
    walkto_draining: bool,
    /// 【BUG-043】点地寻路期间暂存的教程路径箭头步序（SEE flag=1 写入 walkPathBuffer 的那条；末元素 = 下一步）。
    /// walk_to 会占用 walkPathBuffer 装寻路路线，箭头改从这里画、每走完一格按 label216 规则消费，收尾放回 walkPathBuffer。
    /// None = 未暂存（经典面恒 None）。
    walkto_arrows: Option<Vec<u8>>,
    /// 【T2】战斗段快照（撞敌挂起时记账，战斗终结入 CombatStroke）
    /// 撤销快照（实体槽, 型, 复活血, 战前血金, 战前整格实体, 开战时实体数——战斗期间实体表增长 = 不可逆生成，见 BUG-045）
    battle_undo_snap: Option<(usize, i32, i32, crate::undo_journal::PlayerVitals, entity::Entity, usize)>,
    /// 【STAGE-B-008】自动战斗（触屏模式设置项，壳写入；默认 false = 点怪先选中、再点开战）
    pub auto_battle: bool,
    /// 【STAGE-B-008】点地目标标记（走向 / 撞向的格；路线与撞击结束即清）
    pub tap_target: Option<(i32, i32)>,
    /// 【STAGE-B-008】闪红（格, 到期拍）
    pub tap_reject: Option<((i32, i32), i64)>,
    /// 【STAGE-B-008】路线走完后的撞击（停靠格, 方向）——在 Map wp0 执行
    tap_bump: Option<((i32, i32), u8)>,
    /// 【STAGE-B-008】点地寻路收尾后下一拍执行后续（撞击 / 排队）
    tap_followup_ready: bool,
    /// 【STAGE-B-008】已选中的怪物（首点选中、再点开战；每拍校验）
    pub tap_selected: Option<crate::tap_walk::SelectedEnemy>,
    /// 【STAGE-B-008】点地寻路中排队的输入（排空回 Map wp0 后下一拍执行；只留最后一次）
    pub(crate) tap_pending: Option<crate::tap_walk::PendingTap>,
    /// itemUseTargetType：最近拾取/装备的类型号（浮层图标 + 商店显示）
    pub overlay_kind_item: i32,
    /// 浮层图标图（kind1/3：entityTypeImage[itemUseTargetType] 的克隆——paint 免借用冲突）
    pub overlay_icon: Option<ArgbImage>,
    /// 浮层标题（objectTypeNames[itemUseTargetType]）
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
/// shopRainbowColors（a.java:2229）：商店条目逐字彩虹色环（7 色随 shopRainbowPhase 相位）。
pub const RAINBOW_COLORS: [u32; 7] = [
    16711680, 16748544, 16776194, 1244928, 65478, 26367, 14156031,
];

/// equipmentMaterialNames（a.java:484）：装备材质名（0=无）。
pub const EQUIPMENT_MATERIAL_NAMES: [&str; 9] =
    ["无", "木", "铁", "银", "金", "布", "皮", "锁", "金"];

/// GameScene 的存档载入构造（run case8 → mode2 链 → step16 loadFloorSave +
/// step10 loadFloorData/buildMinimap + step11 相机/朝向 + dispatchModeEntry case3）。
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
    scene.equipped_weapon = save.equipped_weapon_type as i32; // 读档恢复装备（a.java:9260）
    scene.equipped_armor = save.equipped_armor_type as i32; // （a.java:9261）
    scene.player.hp = save.player_hp;
    scene.player.atk = save.player_atk;
    scene.player.def = save.player_def;
    scene.player.yellow_keys = save.yellow_keys as i32;
    scene.player.blue_keys = save.blue_keys as i32;
    scene.player.red_keys = save.red_keys as i32;
    scene.player.gold = save.gold;
    scene.journal = save.journal.iter().map(|(k, i)| (*k as i32, *i as i32)).collect();
    scene.player.item_stack_size = save.item_stack_size as i32;
    scene.alchemy_upgrade_count = save.alchemy_upgrades as i32; // a.java:9281 链（loadFloorSave 应用）
    for k in 0..save.item_stack_size as usize {
        scene.player.item_types[k] = save.item_stack_types[k];
        scene.player.item_uses[k] = save.item_stack_uses[k];
    }
    scene.saved_floor_states = save.floor_states.clone();
    // scriptLineFlags 读回（a.java:9285-9291：前 scriptLineCount 项 readBoolean，余
    // skip）——须先于 step 10 loadFloorData（m_122 floor2 恢复特例读 flags[26]）
    scene.script_line_flags = vec![false; 128];
    let line_count = scene.dialog_data.scripts.len().min(save.script_flags.len());
    scene.script_line_flags[..line_count].copy_from_slice(&save.script_flags[..line_count]);
    // step 10 = loadFloorData（应用本层触发缓存）+ buildMinimap（a.java:3515-3517）
    scene.load_floor_data(maplv, px, py, records, scene.floor)?;
    scene.build_minimap();
    // step 11 相机（a.java:3523-3525）+ dispatchModeEntry case 3（a.java:4312-4318）
    scene.enter_view();
    scene.softkeys = (1, 3);
    scene.overlay_active = false;
    scene.apply_step_cell_effects();
    Ok(scene)
}


// ===== restore_rules（refactor-plan §4 特例批）：m_122 恢复分支尾的层特例 =====
// switch(currentFloor)（a.java:9648-9673）配置化。**恢复臂 index quirk 显式保留**
// （entity::restore_floor_state 核内，a.java:9638）；**flags[26] 双重角色**（boss-
// evidence 复核点）= script 26 完成旗标兼 floor2 补生守卫——表的 SpawnIfAbsent.flag
// 原样引用，禁顺手"修正"。

/// door13_hi 高字节表（a.java:902 `new byte[]{2,3,3,3,6,3,3,6,3,3,3,3,3}`）——
/// 型 78 参数高字节同源（game-data door13 表同值，表驱动单测对账两源）。
pub const DOOR13_HI: [u8; 13] = [2, 3, 3, 3, 6, 3, 3, 6, 3, 3, 3, 3, 3];

/// 恢复特例动作（m_122 层特例 switch 的三个 case）。
pub enum RestoreAction {
    /// case 2（a.java:9650-9652）：`!flags[flag] && find(find) < 0` → spawn（像素坐标）。
    SpawnIfAbsent {
        flag: usize,
        find: (i32, i32, i32),
        spawn: (i32, i32, i32, i16),
    },
    /// case 12（a.java:9653-9661）：find 命中 → param = lo | DOOR13_HI[hi_idx] << 8
    ///（原式 6|door13_hi[5]<<8 与 5|door13_hi[4]<<8）。
    ParamRewrite {
        find: (i32, i32, i32),
        lo: i16,
        hi_idx: usize,
    },
    /// case 39（a.java:9664-9671）：find 命中 → removeGroup(find) + spawn。
    /// **像素字面量保真**：Java `spawnEntity(8, 11, 1, 38)` 字节码常量即像素 (11,1)
    /// ⇒ 落 cell(0,0)（javap 1446-1454/1483-1492 逐字）——原版实况，D4 不"修正"。
    SwapSpawn {
        find: (i32, i32, i32),
        spawn: (i32, i32, i32, i16),
    },
}

/// 层恢复特例规则（floor → 动作表）。
pub struct RestoreRule {
    pub floor: i32,
    pub actions: &'static [RestoreAction],
}

/// m_122 恢复特例表（a.java:9648-9673 switch(currentFloor) 逐 case）。
pub static RESTORE_RULES: &[RestoreRule] = &[
    RestoreRule {
        floor: 2,
        actions: &[RestoreAction::SpawnIfAbsent {
            flag: 26,
            find: (11, 4, 84),
            spawn: (84, 352, 128, 0),
        }],
    },
    RestoreRule {
        floor: 12,
        actions: &[
            RestoreAction::ParamRewrite { find: (1, 1, 78), lo: 6, hi_idx: 5 },
            RestoreAction::ParamRewrite { find: (11, 1, 78), lo: 5, hi_idx: 4 },
        ],
    },
    RestoreRule {
        floor: 39,
        actions: &[
            RestoreAction::SwapSpawn { find: (11, 1, 7), spawn: (8, 11, 1, 38) },
            RestoreAction::SwapSpawn { find: (11, 11, 8), spawn: (7, 11, 11, 40) },
        ],
    },
];

/// 模式转移边集（state-machine-survey §2 35 边逐条；件 A' 校验挂点读本表）。
/// `to: None` = **动态目标**（menu_return_mode/in_game_menu_return_mode/target_mode
/// 变量直写——Java 原样，边表记录"允许动态"）。from/to 值=映射表 case 号（GameMode doc）。
pub static TRANSFER_EDGES: &[(GameMode, Option<GameMode>)] = &[
    // 1-3：boot 链（进程→0→21→1）
    (GameMode::Unknown(-1), Some(GameMode::Boot)),
    (GameMode::Boot, Some(GameMode::SoundAsk)),
    (GameMode::SoundAsk, Some(GameMode::MainMenu)),
    // 4-7：主菜单四项
    (GameMode::MainMenu, Some(GameMode::Loading)),
    (GameMode::MainMenu, Some(GameMode::Map)),
    (GameMode::MainMenu, Some(GameMode::InGameMenu)),
    (GameMode::MainMenu, Some(GameMode::Ending)),
    // 8-10：载入完成/过场链（8 target_mode=动态；10 自环）
    (GameMode::Loading, None),
    (GameMode::IntroCutscene, Some(GameMode::Loading)),
    (GameMode::IntroCutscene, Some(GameMode::IntroCutscene)),
    // 11-15：hub（Map）出口
    (GameMode::Map, Some(GameMode::Map)),
    (GameMode::Map, Some(GameMode::InGameMenu)),
    (GameMode::Map, Some(GameMode::ItemPanel)),
    (GameMode::Map, Some(GameMode::Dialog)),
    (GameMode::Map, Some(GameMode::Alchemy)),
    (GameMode::Map, Some(GameMode::Ending)),
    // 16-22：游戏内菜单出口（16 return_mode=动态）
    (GameMode::InGameMenu, None),
    (GameMode::InGameMenu, Some(GameMode::SaveSlot)),
    (GameMode::InGameMenu, Some(GameMode::About)),
    (GameMode::InGameMenu, Some(GameMode::Help)),
    (GameMode::InGameMenu, Some(GameMode::Shop)),
    (GameMode::InGameMenu, Some(GameMode::Map)),
    (GameMode::InGameMenu, Some(GameMode::EnemyInfo)),
            // 退出→回标题（kind7→gameMode=1，a.java:4776-4790 区；2026-10-18 补登）
            (GameMode::InGameMenu, Some(GameMode::MainMenu)),
    // 23：EnemyInfo 返回
    (GameMode::EnemyInfo, Some(GameMode::Map)),
    // 24：7/8 存档槽（return_mode=动态）
    (GameMode::SaveSlot, None),
    (GameMode::EquipDetail, None),
    // 25-29：炼丹/道具面板出口
    (GameMode::Alchemy, Some(GameMode::Map)),
    (GameMode::ItemPanel, Some(GameMode::Map)),
    (GameMode::ItemPanel, Some(GameMode::EnemyInfo)),
    (GameMode::ItemPanel, Some(GameMode::RecordBrowser)),
    // 29e：ItemPanel→ItemPanel 自环——kind3 确认浮层 -7"回面板"臂（Java 自赋值
    // `gameMode = 10`，deobf a.java:3297）；缺此边时 census debug_assert 于合法
    // 自赋值处误杀（2026-10-20 P0-3 同类第三例，BUG-023）。
    (GameMode::ItemPanel, Some(GameMode::ItemPanel)),
    // 29c：RecordBrowser→ItemPanel——mode12 -5/-6/-7 关回 mode10（a.java run case 12 头注）。
    (GameMode::RecordBrowser, Some(GameMode::ItemPanel)),
    // 29b：Map→RecordBrowser——道具 14（生死簿）使用臂直开记录浏览器（mode 12）。
    // 巡查 35 边表漏登（2026-10-17 P0-1 同类第二缺口，preset_floor5 实测暴露）。
    (GameMode::Map, Some(GameMode::RecordBrowser)),
    // 29d：Map→EnemyInfo——道具 13（火眼金睛）使用臂：Java mode10 确认先置
    // gameMode=3、activateItem 内置 gameMode=5（a.java:3939-3945 + 7786-7788
    // 双跳原样）；巡查表漏登此边 ⇒ census assert panic 误报"原版崩溃等价终止"
    // （用户实机 BUG-017，2026-10-17 P0-1 同类第三缺口）。
    (GameMode::Map, Some(GameMode::EnemyInfo)),
    // 30-31：对话收尾（行尾门→相位 4 非模式转移；相机收敛→Map）
    (GameMode::Dialog, Some(GameMode::Map)),
    // 30c：Dialog→Ending——END 脚本指令（a.java:8510-8514 gameMode=20）。
    (GameMode::Dialog, Some(GameMode::Ending)),
    // 30b：Dialog 自环——脚本重入（loadLevelScript 在 mode 11 内重跑剧本、scriptCursor
    // 归 0；BUG-012 三拍循环实证，a.java:8053/8519 同构）；与边 10（IntroCutscene 自环）
    // 同款。缺此边时 debug_assert 于合法重入处误杀（2026-10-17 P0-1）。
    (GameMode::Dialog, Some(GameMode::Dialog)),
    // 32-35：Help/About/Shop/Ending 出口（32/33 return_mode=动态）
    (GameMode::Help, None),
    (GameMode::About, None),
    (GameMode::Shop, Some(GameMode::InGameMenu)),
    (GameMode::Ending, Some(GameMode::MainMenu)),
];

/// 命令模式（refactor-plan §3 第 2 步）：16 op 枚举。变体载荷=各 op 条件取参
/// 结果 + **推进后游标**（游标契约，条件 4）；解析与执行分离但游标语义逐字节
/// 等价（scriptCursor/lastDelimPos 口径 gotcha 级，两面对照测钉）。
pub(crate) enum ScriptOp {
    /// TAK_a_b（a.java:8257-8263）
    Tak { start: i32, end: i32, cursor: usize },
    /// LAY_x（a.java:8355-8362）
    Lay { floor: i32, cursor: usize },
    /// GIN_a_b（a.java:8424-8437）
    Gin { kind: i32, value: i32, cursor: usize },
    /// RES_x（a.java:8470-8490）
    Res { cursor: usize },
    /// ADD_a_b_c_d（a.java:8438-8468）
    Add { floor: i32, ty: i32, cx: i32, cy: i32, cursor: usize },
    /// SEE_a_b_c_d_e（a.java:8487-8510）
    See { tx: i32, ty: i32, page: i32, page_end: i32, flag: i32, cursor: usize },
    /// GUT_n（a.java:8308-8311）——cursor 由 loadLevelScript 覆写为 0
    Gut { target: i32, cursor: usize },
    /// MOV（a.java:8274-8309）：var55>0 ⇒ cell=Some(五参实体路线)；否则三参玩家行
    Mov { target_type: i32, cell: Option<(i32, i32)>, dst: (i32, i32), cursor: usize },
    /// DES 负参（a.java:8311-8320）：全表批量移除 -type
    DesAll { want: i32, cursor: usize },
    /// DES 正参（a.java:8321-8334）：格上定点移除
    DesAt { ty: i32, cx: i32, cy: i32, cursor: usize },
    /// SWD（a.java:8336-8352）
    Swd { cursor: usize },
    /// CES_t_cx_cy（a.java:8386-8420）
    Ces { ty: i32, cx: i32, cy: i32, cursor: usize },
    /// GLV_n（a.java:8464-8466）
    Glv { cursor: usize },
    /// END_n（a.java:8445-8463 区）
    End { cursor: usize },
    /// SMS_n（a.java 纯推进）
    Sms { cursor: usize },
    /// ROS_1（a.java:8361-8366）：玩家格坐标
    RosMove { px: i32, py: i32, cursor: usize },
    /// ROS_2（a.java:8367-8369）：强制装备
    RosEquip { tier: i32, cursor: usize },
    /// ROS_3（a.java:8370-8372）：HP
    RosHp { v: i32, cursor: usize },
    /// ROS_4（a.java:8373-8375）：朝向（Java byte 字段 ⇒ i8 截断保真）
    RosFace { v: i32, cursor: usize },
    /// ROS_5（a.java:8376-8378）：ATK
    RosAtk { v: i32, cursor: usize },
    /// ROS_6（a.java:8379-8381）：DEF
    RosDef { v: i32, cursor: usize },
    /// ROS 其余子码（a.java:8381 default）：不取参不推进（v2 不动）
    RosNoop { cursor: usize },
    /// MVS（a.java:8519-8592）
    Mvs { cursor: usize },
}

impl ScriptOp {
    /// 游标契约（条件 4）：各 op 的推进后游标。
    fn cursor(&self) -> usize {
        match self {
            ScriptOp::Tak { cursor, .. }
            | ScriptOp::Lay { cursor, .. }
            | ScriptOp::Gin { cursor, .. }
            | ScriptOp::Res { cursor }
            | ScriptOp::Add { cursor, .. }
            | ScriptOp::See { cursor, .. }
            | ScriptOp::Gut { cursor, .. }
            | ScriptOp::Mov { cursor, .. }
            | ScriptOp::DesAll { cursor, .. }
            | ScriptOp::DesAt { cursor, .. }
            | ScriptOp::Swd { cursor }
            | ScriptOp::Ces { cursor, .. }
            | ScriptOp::Glv { cursor }
            | ScriptOp::End { cursor }
            | ScriptOp::Sms { cursor }
            | ScriptOp::RosMove { cursor, .. }
            | ScriptOp::RosEquip { cursor, .. }
            | ScriptOp::RosHp { cursor, .. }
            | ScriptOp::RosFace { cursor, .. }
            | ScriptOp::RosAtk { cursor, .. }
            | ScriptOp::RosDef { cursor, .. }
            | ScriptOp::RosNoop { cursor }
            | ScriptOp::Mvs { cursor } => *cursor,
        }
    }
}


impl GameScene {
    /// loadFloorData（a.java:9404-9438）：maplv 装载 + initCellGrid 格索引重建 +
    /// 可行走重建 + m_122 实体生成 + sortEntitiesByY 排序。
    /// `floor_assets`：按层号索引的换层资产（索引缺省 = 该层不可达）。
    #[allow(clippy::too_many_arguments)]
    /// 浮层框绘制（paint 公共尾 a.java:3129-3168）：盒框 + kind 分派内容 +
    /// 左右软键图标。mode 3/4/7/10/16/19 各 paint 共用。
    fn paint_overlay_box(&mut self, g: &mut SoftGraphics<'_>) {
        // 提示浮层（paint 公共尾 a.java:3129-3168，!floorTransitioning && overlayActive）：
        // 盒框 + kind 0/2/4/5 纯文本 / kind 1/3 图标+文本 + 软键图标
        if !self.transitioning && self.overlay_active {
            let bx = (self.layout.screen_w - self.overlay_box_w) >> 1;
            let by = (self.layout.screen_h - self.overlay_box_h) >> 1;
            crate::menu_family::paint_box_frame(g, &self.images.ui[0], bx, by, self.overlay_box_w, self.overlay_box_h);
            let font = crate::paint::paint_font();
            let mut tx = bx + 16;
            let ty = by + 16;
            if matches!(self.overlay_kind, 0 | 2 | 4 | 5) {
                g.set_color(WHITE);
                self.paint_wrapped_text(g, tx, ty, OVERLAY_TEXT_W, OVERLAY_TEXT_MAX_H, true);
            } else {
                paint_mini_frame(g, tx, ty, 32, 32);
                // 图标 = entityTypeImage[overlayKindItem]（itemUseTargetType——拾取时记录）
                // 【R-2 换算收敛】居中式 (32-img.width)>>1：宽高用**逻辑**尺寸（=w/d）。
                if let Some(img) = &self.overlay_icon {
                    g.draw_image(
                        img,
                        tx + ((32 - img.logical_width()) >> 1),
                        ty + ((32 - img.logical_height()) >> 1),
                        0,
                    );
                }
                tx += 42;
                g.set_color(16770173);
                if let Some(name) = &self.overlay_item_name {
                    g.draw_string(name, tx, ty + ((32 - font.height()) >> 1), 0);
                }
                g.set_color(WHITE);
                self.paint_wrapped_text(g, bx + 16, ty + 36, OVERLAY_TEXT_W, self.overlay_box_h - 32 - 36, true);
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
            global: crate::save::GlobalSave::default_7(),
            global_dirty: false,
            view: GameView::from_maplv(maplv, player_px, player_py, crate::game_view::ViewDims::classic())?,
            entities: EntityTable::new(100),
            grid: CellGrid::new(1, 1),
            images,
            walkable: Vec::new(),
            minimap: None,
            minimap_enabled: true,
            layout: crate::layout::Layout::classic(),
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
            route_dir: 0,
            route_phase: 0,
            route_progress: 0,
            route_start: (0, 0),
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
            game_mode: GameMode::Map,
            transition_count: 0,
            last_transition: None,
            dialog_phase: 0,
            record_page: 0,
            record_text_a: include_str!("../tests/fixtures/record_texts_a.txt")
                .lines()
                .map(|l| l.replace("\\n", "\n"))
                .collect(),
            record_text_b: include_str!("../tests/fixtures/record_texts_b.txt")
                .lines()
                .map(|l| l.replace("\\n", "\n"))
                .collect(),
            page_indicator: String::new(),
            journal: Vec::new(),
            ending_scroll: 0,
            ending_anim: 0,
            ending_done: false,
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
            menu_return_mode: GameMode::MainMenu,
            slot: crate::menu_family::SlotSelect::new(),
            preset_mot_if: None,
            encyc_damage: Vec::new(),
            encyc_types: Vec::new(),
            encyc_panel_w: ENCYC_PANEL_W,
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
            help_override: None,
            shell_options: Vec::new(),
            bgm: crate::audio::BgmState::in_game_default(),
            saved_slot_blobs: [None, None, None, None, None, None],
            rainbow_phase: 0,
            strip_frame: 0,
            in_game_menu_return_mode: GameMode::Boot, // BUG-008：Java 无初值（默认 0）保真对齐；原值 3 属顺手修 bug 偏差
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
            undo: crate::undo_journal::UndoJournal::new(),
            route: crate::pathfind::RouteTracker::new(),
            walkto_draining: false,
            walkto_arrows: None,
            battle_undo_snap: None,
            auto_battle: false,
            tap_target: None,
            tap_reject: None,
            tap_bump: None,
            tap_followup_ready: false,
            tap_selected: None,
            tap_pending: None,
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
        scene.load_floor_data(maplv, player_px, player_py, records, scene.floor)?;
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
        floor: i32,
    ) -> Result<(), String> {
        // 【BUG-045④】换层清栈（旧层坐标的位移笔在新层无意义——复核探针 D：撤销把玩家传回上一层坐标）
        self.undo.clear(crate::undo_journal::ClearReason::FloorChange);
        self.clear_tap_state();
        // 【P0 回归修复 2026-10-20】换层重建视图**布局换轨收口**：from_maplv 显式
        // 带 ViewDims（编译强制=漏网-proof）——旧 classic 常量初始致换层残留
        // 240-era col/row 范围=modern 地形半绘根因（floor1 正常=初始已换轨、
        // 切层后半绘实证）。
        let view = GameView::from_maplv(
            maplv,
            player_px,
            player_py,
            crate::game_view::ViewDims::from_layout(&self.layout),
        )?;
        let cols = view.wide as usize;
        let rows = view.high as usize;
        let mut walkable = vec![vec![false; cols]; rows];
        for (y, row) in walkable.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                // rebuildWalkability（a.java:6964-6976）的索引：**行距 wide·4、
                // 列距 2**（py_key 行尾 +=wide<<1 叠加列循环的 ×2）——与 paintTileLayer
                // 的行距 wide·2、列距 1 是同一数组的两套读法（原版共存）。
                // A 级：oracle FLD 134（walkableGrid）13×13 逐格复算匹配。
                let idx = (y * cols * 4 + x * 2) as i32;
                let tile = view.terrain_at(0, idx) as usize;
                *cell = tile < self.tile_walk.len() && self.tile_walk[tile] == 1;
            }
        }
        let mut entities = EntityTable::new(100);
        let mut grid = CellGrid::new(view.wide, view.high);
        // m_122 双分支（a.java:9465+）：有层存档 → 从存档恢复（**死怪不复活**、
        // 已开门保持开）；无存档 → sprite{n} 权威数据 + 5/81/6/12/76/82/83 隐藏
        // 存档态按**目标层**取键（a.java:9465+ f_byte_arr2_05[py_key]）——跨层装载
        // （probe/ADD）时不得取当前层（潜伏缺陷 2026-10-15 修）
        if let Some(data) = self.saved_floor_states[floor as usize].clone() {
            // m_122 存档分支核=entity::restore_floor_state（第 1 步文件合流）；
            // 恢复臂 index quirk（a.java:9638）随核保留。
            let img_h_of = |t: i32| self.images.entity.get(t as usize).cloned().flatten().map(|im| im.height);
            entity::restore_floor_state(&mut entities, &mut grid, &data, &self.width_table, &img_h_of);
            // 层特例 switch(currentFloor)（a.java:9648-9673）= RESTORE_RULES 表驱动
            //（第 4 步特例批配置化；case 12/39 一并入表——原"未移植"欠账清偿）。
            self.apply_restore_rules(&mut entities, &mut grid);
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
                    entities.items[idx].visible = false; // m_122 特例分支（5/81/6/12/76/82/83）
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
            // 已达层界提示（a.java:7133-7144）：下界 miscTexts[0]、上界 [1]
            if target < self.min_floor {
                self.show_overlay_message(0, MISC_TEXTS[0], 0, 0);
                return false;
            }
            if target > self.max_floor {
                self.show_overlay_message(0, MISC_TEXTS[1], 0, 0);
                return false;
            }
        }
        // 层界提示（a.java:7150-7156）：负层 [2]、超 floorCountMax=55 [3]（a.java:808）
        if target < 0 {
            self.show_overlay_message(0, MISC_TEXTS[2], 0, 0);
            return false;
        }
        if target > 55 {
            self.show_overlay_message(0, MISC_TEXTS[3], 0, 0);
            return false;
        }
        // m_119(currentFloor)（a.java:9370-9388）：序列化**存活**实体
        // （removed/state==1 跳过）——死怪/已开门跨层持久
        self.save_floor_state();
        self.transitioning = true;
        // > 【2026-10-26 更正·BUG-026】原此处 `transition_to(Map)`（注"changeFloor 起遮幅即回 mode 3，
        // > A-preset-item21 T402 实证"）系**调用方语义误挂**：Java changeFloor（a.java:7135-7180）
        // > 不写 gameMode；mode 3 来自道具 21 调用方 `changeFloor; keyHeldCode=0; gameMode=3`
        // > （a.java:7863-7866）。误挂使**脚本行走踩楼梯**（mode 11 中 MOV 走上梯格，如一层嫦娥
        // > 走位戏 script19 尾 `MOV_0_11_1 MOV_0_1_11`）切层即回 mode 3、丢弃剩余脚本——Java 在
        // > mode 11 续跑（fx-chang-e-floor1-taps T786：Java gameMode=11，2 层续走 MOV_0_1_11）。
        if target < self.min_floor {
            self.min_floor = target;
        } else if target > self.max_floor {
            self.max_floor = target;
            self.note_max_floor(target); // f_int_154 水位（a.java:7163-7164）
        }
        self.pending_floor = target;
        self.went_down = down;
        true
    }

    /// m_119（a.java:9373-9400）：层状态序列化。格式：u16 计数（BE，m_120）+
    /// 每存活实体 [type u8, x u16 BE, y u16 BE, visible u8, param u16 BE]。
    ///
    /// 槽先置 null（a.java:9383），**仅 entityCount > 0** 才分配写入（a.java:9384）
    /// ——实体表全空的层存 None（重入时走 sprite{n} 权威数据），而"有实体但全
    /// 死/已开"的层存 `[00 00]`（重入恢复 0 个）。注意判据是 entityCount（含已
    /// 移除槽位），非存活数：故存档恢复出 0 实体的层再次离开时 entityCount==0
    /// → None → 第三次进入从 sprite 重新生成全部实体（原版"清层复活"行为，保真）。
    fn save_floor_state(&mut self) {
        // m_119：表操作核=entity::serialize_floor_state（第 1 步文件合流）；
        // count==0 ⇒ 存 null（a.java:9383-9384）留本层。
        if self.entities.count == 0 {
            self.saved_floor_states[self.floor as usize] = None;
            return;
        }
        let data = entity::serialize_floor_state(&self.entities);
        self.saved_floor_states[self.floor as usize] = Some(data);
    }

    /// 遮幅中段换层（a.java:3186-3206）：loadFloorData + 落点特例
    /// （0/50/1-上行 place_player_at_cell；其余 findFloorGateEntity → step_off_stairs）+ 小地图 + 步效果。
    fn swap_floor(&mut self) {
        self.floor = self.pending_floor;
        self.current_floor = self.pending_floor;
        let fa = self.floor_assets[self.pending_floor as usize]
            .clone()
            .expect("层资产未提供");
        let px = self.view.player_px;
        let py = self.view.player_py;
        self.load_floor_data(&fa.maplv, px, py, &fa.records, self.floor)
            .expect("层资产重载失败");
        if self.floor == 0 {
            self.place_player_at_cell(1, 2);
            self.view.center_on_player();
        } else if self.floor == 50 {
            self.place_player_at_cell(6, 7);
            self.view.center_on_player();
        } else if self.floor == 1 && !self.went_down {
            self.place_player_at_cell(6, 11);
            self.view.center_on_player();
        } else if let Some((gx, gy)) = self.find_floor_gate(self.went_down) {
            self.step_off_stairs(gx, gy);
        }
        self.build_minimap();
        self.apply_step_cell_effects();
    }

    /// findFloorGateEntity（a.java:7106-7118）：up=true 找型 7、false 找型 8；
/// 换层落点直传 wentDown（false=上楼抵达→机型 8 下楼梯），
    /// param>>9==0（低 9 位无高字节——目标层 ≤255 的常规门）。
    /// findFloorGateEntity（a.java:7106-7122）：首个 param>>9==0 的型7/8
    /// 门格实体下标（暗道换乘目标——param 低 8 位 = 目标层）。
    pub(crate) fn find_floor_gate_entity(&self, up: bool) -> Option<usize> {
        let want = if up { 7 } else { 8 };
        for e in 0..self.entities.count {
            if self.entities.items[e].type_ == want && (self.entities.items[e].param >> 9) == 0 {
                return Some(e);
            }
        }
        None
    }

    fn find_floor_gate(&self, up: bool) -> Option<(i32, i32)> {
        let want = if up { 7 } else { 8 };
        for e in 0..self.entities.count {
            if self.entities.items[e].type_ == want && (self.entities.items[e].param >> 9) == 0 {
                return Some((self.entities.items[e].px >> 5, self.entities.items[e].py >> 5));
            }
        }
        None
    }

    /// step_off_stairs（a.java:5807-5814）：楼梯落点 = 首个可行走邻格（上/下/左/右序）。
    /// Java 反混淆原名 `stepOffStairs`（对照性锚——P0 命名批 2026-10-19 机械改名，语义不变）。
    fn step_off_stairs(&mut self, mut gx: i32, mut gy: i32) {
        if self.walkable_at(gx, gy - 1) {
            gy -= 1;
        } else if self.walkable_at(gx, gy + 1) {
            gy += 1;
        } else if self.walkable_at(gx - 1, gy) {
            gx -= 1;
        } else if self.walkable_at(gx + 1, gy) {
            gx += 1;
        }
        self.place_player_at_cell(gx, gy);
        self.view.center_on_player();
    }

    /// place_player_at_cell（a.java:5193-5193）：格坐标直置 + 步进度清零。
    /// Java 反混淆原名 `placePlayerAtCell`（对照性锚——P0 命名批 2026-10-19 机械改名，语义不变）。
    fn place_player_at_cell(&mut self, cx: i32, cy: i32) {
        self.player_cell_x = cx;
        self.player_cell_y = cy;
        self.view.player_px = cx << 5;
        self.view.player_py = cy << 5;
        self.step_progress = 0;
    }

    /// removeSlotAt removeSlotAt（a.java:6435-6441）：槽位摘除 + 前移。
    pub fn remove_slot_at(&mut self, cx: i32, cy: i32, slot: i32) {
        let ct = self.grid.cell_type[cy as usize][cx as usize] as usize;
        let cap = self.grid.capacity[ct] as usize;
        if cap == 0 {
            return;
        }
        let idx = self.grid.slots[ct][slot as usize] as usize - 1;
        self.entities.items[idx].removed = true;
        // 【六写点收口】按下标摘除面 slot_remove_at（压缩语义同原手写环）
        self.grid.slot_remove_at(ct, slot as usize);
    }

    /// tryOpenChest tryOpenChest（a.java:7285-7299）：全图收集同 param 型 4（未移除、
    /// 非 5）；open_group 非空 = 有锁可开（walkPhase 4 动画）。
    pub fn try_open_chest(&mut self, chest_param: i16) -> bool {
        self.open_group.clear();
        for k in 0..self.entities.count {
            if !self.entities.items[k].removed && chest_param == self.entities.items[k].param {
                let tk = self.entities.items[k].type_;
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
        let line_h = font.height() + 4;
        if matches!(kind, 0 | 2 | 4 | 5) {
            self.layout_full_text(text, OVERLAY_TEXT_W, OVERLAY_TEXT_MAX_H);
            self.overlay_box_w = self.overlay_content_w + 32;
            self.overlay_box_h = 32 + (self.overlay_page_bottom - self.overlay_page_top) * line_h;
        } else {
            self.layout_full_text(text, OVERLAY_TEXT_W, OVERLAY_ICON_TEXT_MAX_H);
            self.overlay_box_w = 212;
            self.overlay_box_h = 68 + (self.overlay_page_bottom - self.overlay_page_top) * line_h;
        }
        self.overlay_left_icon = left;
        self.overlay_right_icon = right;
        if left != 0 || right != 0 {
            self.overlay_box_h += 18;
        }
    }

    /// layoutFullText（a.java:4940-4942）：Java 原文即
    /// `layoutWrappedText(py_key, script_cursor0, table_count, py_key.length())`——limit=全长 的特例，
    /// 同构委托给单一实现 [`Self::layout_wrapped_text_limited`]（曾为两份拷贝，
    /// 迭代 26→30 修一份漏一份；见 pattern_dual-copy-algorithms-sync-fix）。
    /// 全长取 chars 计数：Java `length()` 是 UTF-16 码元数，game-core 全部文本源
    /// （字面量 + include_str 夹具）均为 BMP 字符，二者逐值相等。
    pub fn layout_full_text(&mut self, text: &str, w: i32, max_h: i32) {
        let full_len = text.chars().count() as i32;
        self.layout_wrapped_text_limited(text, w, max_h, full_len);
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

    /// equipTierIndex（a.java:7688-7694）：档位区间倒序查当前装备所在档。
    fn m_083_index(&self, equipped: i32, lo: i32, hi: i32) -> i32 {
        // a.java:7693 `while (--table_count >= script_cursor0)`：**先减后比**——扫描区间 =
        // [lo, hi-1]，索引 hi 永不触达（equip_types 12 项、hi=12 不越界）。
        // 迭代 70 floor43 80 甲拾取越界实为此处误译（后减结构触达 hi）。
        let mut t = hi;
        while t > lo {
            t -= 1;
            if self.equip_types[t as usize] == equipped {
                return t;
            }
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
    /// paintWrappedText（a.java:5021-5119）：2026-10-17 单源适配——绘制本体 =
    /// `menu_family::paint_wrapped_lines`（边缘语义以 Java 为准：overlaySavedColor
    /// 开场置 `getColor()`、每次 `\c` 刷新、跨行持久；旧拷贝行内/跨 paint 两向偏离）。
    fn paint_wrapped_text(&mut self, g: &mut SoftGraphics<'_>, x: i32, y: i32, w: i32, _max_h: i32, arrows: bool) {
        let font = crate::paint::paint_font();
        let line_h = font.height() + 4;
        let lines: Vec<Vec<u16>> = self
            .overlay_lines
            .iter()
            .map(|l| l.encode_utf16().collect())
            .collect();
        self.overlay_saved_color = crate::menu_family::paint_wrapped_lines(
            g,
            &lines,
            self.overlay_page_top,
            self.overlay_page_bottom,
            self.overlay_total_lines,
            line_h * self.overlay_lines_per_page,
            line_h,
            x,
            y,
            w,
            arrows,
            self.frame_counter as i32,
            &font,
        ) as i32;
    }

    /// m_122（a.java:9465-9680）：**纯实体表装载**（≠loadFloorData——后者含视图/
    /// 行走表/小地图）。存档态优先（替换语义 + removed=false），否则 sprite 记录
    /// （5/81/6/12/76/82/83 生成置不可见）；尾部层特例 + type72 renderCategory 随层。
    /// probe/ADD 的 m_119/m_122 往返用本函数（用 load_floor_data 会清小地图/重建
    /// 视图——实测 T281 帧差元凶）。格恒 13×13 ⇒ 网格重建无涉视图。
    /// 模式转移收口（件 A'）：**game_mode 唯一写点**。散点归零——加新转移=改
    /// 本函数调用点 + TRANSFER_EDGES 边表（件 D 维护文档）。
    /// **边集机械校验挂点**：debug_assert 对边表（release 不触发——金样零漂移）；
    /// 转移计数/日志=观测字段（不进 dump，无行为影响）。
    pub fn transition_to(&mut self, mode: GameMode) {
        let from = self.game_mode;
        debug_assert!(
            TRANSFER_EDGES.iter().any(|(f, t)| *f == from && (*t == Some(mode) || t.is_none())),
            "未登记转移边：{:?} -> {:?}（TRANSFER_EDGES 表，件 D 维护）",
            from,
            mode
        );
        self.transition_count += 1;
        self.last_transition = Some((from, mode));
        self.game_mode = mode;
    }

    /// 应用层恢复特例（m_122 尾 switch(currentFloor)，a.java:9648-9673）——
    /// RESTORE_RULES 表驱动；flags[26] 双重角色与 case 39 像素字面量 quirk 随表保留。
    fn apply_restore_rules(
        &self,
        entities: &mut EntityTable,
        grid: &mut CellGrid,
    ) {
        let Some(rule) = RESTORE_RULES.iter().find(|r| r.floor == self.floor) else {
            return;
        };
        for action in rule.actions {
            match *action {
                RestoreAction::SpawnIfAbsent { flag, find, spawn } => {
                    // flags[26] 双重角色：守卫=旗标关 + 目标格无活同型（a.java:9650-9652）
                    if self.script_line_flags[flag] {
                        continue;
                    }
                    if entity::cell_entity(grid, entities, find.0, find.1, find.2) >= 0 {
                        continue;
                    }
                    let img_h = self.images.entity[spawn.0 as usize].as_ref().map(|i| i.height);
                    entity::spawn(
                        entities,
                        grid,
                        spawn.0,
                        spawn.1,
                        spawn.2,
                        spawn.3,
                        &self.width_table,
                        img_h,
                    );
                }
                RestoreAction::ParamRewrite { find, lo, hi_idx } => {
                    // a.java:9653-9661：find 命中 → param = lo | door13_hi[hi_idx] << 8
                    let idx = entity::cell_entity(grid, entities, find.0, find.1, find.2);
                    if idx >= 0 {
                        entities.items[idx as usize].param =
                            lo | ((DOOR13_HI[hi_idx] as i16) << 8);
                    }
                }
                RestoreAction::SwapSpawn { find, spawn } => {
                    // a.java:9664-9671：find 命中 → removeGroup + spawn（像素字面量保真）
                    if entity::cell_entity(grid, entities, find.0, find.1, find.2) < 0 {
                        continue;
                    }
                    entity::remove_group_at_cell_by_type(find.0, find.1, find.2, entities, grid);
                    let img_h = self.images.entity[spawn.0 as usize].as_ref().map(|i| i.height);
                    entity::spawn(
                        entities,
                        grid,
                        spawn.0,
                        spawn.1,
                        spawn.2,
                        spawn.3,
                        &self.width_table,
                        img_h,
                    );
                }
            }
        }
    }

    pub fn reload_floor_entities(&mut self, floor: i32) {
        let mut entities = entity::EntityTable::new(100);
        let mut grid = entity::CellGrid::new(self.view.wide, self.view.high);
        if let Some(data) = self.saved_floor_states[floor as usize].clone() {
            // m_122 存档分支核=entity::restore_floor_state（第 1 步文件合流）；
            // 恢复臂 index quirk（按循环下标回填 visible/removed，a.java:9638）随核保留。
            let img_h_of = |t: i32| self.images.entity.get(t as usize).cloned().flatten().map(|im| im.height);
            entity::restore_floor_state(&mut entities, &mut grid, &data, &self.width_table, &img_h_of);
            self.apply_restore_rules(&mut entities, &mut grid);
        } else if let Some(fa) = self.floor_assets.get(floor as usize).cloned().flatten() {
            for &(type_code, x, y, param, visible) in &fa.records {
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
                    entities.items[idx].visible = false;
                }
            }
        }
        entity::sort_by_y(&mut entities, &mut grid);
        self.entities = entities;
        self.grid = grid;
        // 【BUG-045】实体表按活体重建并重排下标（跨层探针 m_119/m_122 往返）⇒ 撤销笔里的实体槽号失效（复核 T3-R3）⇒ 清栈
        self.undo.clear(crate::undo_journal::ClearReason::Irreversible);
    }

    /// probeCrossFloorSnapshot（a.java:8640-8659）：跨层探存活实体——忠实往返：
    /// m_119 存当前 → m_122（reload_floor_entities）载目标 → 扫 → 还原（不存目标层）。
    /// 结果以真装载后的实体表为准（floor23 存档态无型12 ⇒ false，与参照跑
    /// T281 mode=3 吻合）。
    pub fn probe_cross_floor_snapshot(&mut self, floor: i32, type_id: i32) -> bool {
        let foreign = self.floor != floor;
        if foreign {
            self.save_floor_state();
            self.reload_floor_entities(floor);
        }
        let mut found = false;
        for e in 0..self.entities.count {
            if !self.entities.items[e].removed && self.entities.items[e].type_ == type_id {
                found = true;
                break;
            }
        }
        if foreign {
            self.reload_floor_entities(self.floor);
        }
        found
    }

    /// tryRunScene（a.java:8065-8139）：脚本行门控（scriptLineFlags + 场景参数
    /// script_cursor0=fromWalk）。case 40 落 5..40 直通组（table_count 原样 = !flags[40]）。
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
            // 7/32 需 probeCrossFloorSnapshot(23,12) 跨层快照探针（f_byte_arr2_05 层序列化语义）——
            // 未移植，显式挡路（覆盖窗不可达）。
            // tryRunScene case 7（a.java:8123-8127）：gate = from_walk ∧ 跨层无存活
            // 型12（floor23）；过门即置 flags[32]（32 号脚本视为已跑）
            7 => {
                gate &= from_walk && !self.probe_cross_floor_snapshot(23, 12);
                if gate {
                    self.script_line_flags[32] = true;
                }
            }
            // tryRunScene 到达臂（a.java:8128-8146）：case 8/9/11/12/19/30/41-45/
            // 48-50/52/53/55/56 同体 `table_count &= script_cursor0`——**from_walk=true（到达）才过**。
            // 【2026-10-16 更正】此前误并入 case 32 探针臂（"共臂"注释错），导致
            // bump 即触发 script12，早于 Java 真身 7 拍（mov_route floor40 T281 实证）。
            // cover"进 (6,7) 型83(param12) 到达触发"= applyStepCellEffects 路径。
            8 | 9 | 11 | 12 | 19 | 30 | 41 | 42 | 43 | 44 | 45 | 48 | 49 | 50 | 52
            | 53 | 55 | 56 => {
                gate &= from_walk;
            }
            // tryRunScene 探针臂（a.java:8193-8201）：case 32 独占——
            // flags[7] 已跑 ⇒ 挡；否则 gate &= !from_walk ∧ floor23 有存活型12
            32 => {
                if self.script_line_flags[7] {
                    gate = false;
                } else {
                    gate &= !from_walk && self.probe_cross_floor_snapshot(23, 12);
                }
            }
            // 15（a.java:8119-8126）：当前层有活型5（门卫/宝箱守卫）→ false；
            // 否则 &= from_walk（bump 不触发、到达才触发——floor49 伏击门）。
            15 => {
                if self.m_102_current_floor(5) {
                    gate = false;
                } else {
                    gate &= from_walk;
                }
            }
            16 => gate &= self.difficulty_index == 0,
            // 18（a.java:8127-8144）：from_walk 且八 Cell 扫描任一命中 → false
            // + flags[18] 置位（六格缺型1 或两格有型1）。
            18 => {
                gate &= from_walk;
                if gate
                    && (self.entity_at_cell(2, 2, 1) < 0
                        || self.entity_at_cell(2, 4, 1) < 0
                        || self.entity_at_cell(2, 6, 1) < 0
                        || self.entity_at_cell(6, 2, 1) < 0
                        || self.entity_at_cell(4, 6, 1) < 0
                        || self.entity_at_cell(6, 6, 1) < 0
                        || self.entity_at_cell(4, 2, 1) >= 0
                        || self.entity_at_cell(6, 4, 1) >= 0)
                {
                    gate = false;
                    self.script_line_flags[18] = true;
                }
            }
            24 => {
                gate &= !from_walk;
                if gate {
                    gate = self.player.hp > 0;
                }
            }
            25 => {
                gate &= from_walk
                    && self.script_line_flags[13]
                    && self.entity_at_cell(2, 2, 60) < 0
                    && self.entity_at_cell(10, 2, 60) < 0;
            }
            26 => {
                if self.entity_at_cell(11, 4, 84) < 0 {
                    gate = false;
                }
            }
            27 => gate &= from_walk && self.script_line_flags[0],
            31 => {
                gate &= from_walk && self.player.find_item(16) >= 0;
            }
            33 => gate &= self.script_line_flags[7],
            34 => {
                gate &= from_walk && !self.script_line_flags[10];
                if gate {
                    let loss = self.predict_hp_loss(69, true);
                    gate &= loss < 0 || loss >= self.player.hp;
                }
            }
            // 5/6/10/13/14/17/20/21/22/23/28/29/35/36/37/38/39/40/51/54/57-65：
            // 直通（无门控，a.java:8098-8108 簇 default: break）
            46 => gate &= from_walk && !self.script_line_flags[47],
            47 => gate &= from_walk && !self.script_line_flags[46],
            66 => gate = false,
            67 => gate &= self.difficulty_index > 0,
            _ => {}
        }
        gate
    }

    /// probeCrossFloorSnapshot(currentFloor, type) 的同层路径（a.java:8637-8657 无换层分支）：
    /// 扫当前实体表是否存在活的目标类型。跨层探针（case 7/32 的
    /// probeCrossFloorSnapshot(23,12)）需层序列化快照，未移植（见上 todo）。
    fn m_102_current_floor(&self, type_id: i32) -> bool {
        (0..self.entities.count)
            .any(|i| !self.entities.items[i].removed && self.entities.items[i].type_ == type_id)
    }

    /// loadLevelScript（a.java:8035-8052）：mode 11 + 脚本行装载。
    /// case 31 层重排特例（a.java:8028-8028）未达（script 40 不涉），todo 守卫。
    pub fn load_level_script(&mut self, idx: i32) {
        // 【2026-10-26·BUG-025】剧情接管 = 收束触屏寻路段（WalkTo 走进剧情点）：段收于实际终点、清余步。
        // 否则剧情中投键（press_key 见段开放即打断）截断脚本 MOV 行走，dp3 步尽臂还会把
        // 剧情中途切回 Map。经典面（无寻路段）interrupt_route 零操作。
        self.interrupt_route();
        // 【BUG-045④】剧情推进不可撤（undo_journal 清栈边界：MOV / 脚本事件触发 = 清栈）
        self.undo.clear(crate::undo_journal::ClearReason::ScriptEvent);
        self.clear_tap_state();
        // 【BUG-043】寻路被剧情接管：箭头暂存**不在此放回**——撞格触发剧情时本函数在 advanceScriptWalk 撞格臂之前执行，
        // 立即放回会被随后的 `walk_step_count = 0` 清掉（复核 R2）；改由 settle_walkto_arrows 在寻路释放 dp 后放回
        self.walkto_draining = false; // 【BUG-040】dp 归剧情（打断后的寻路收尾作废）
        self.script_walk_armed = false;
        self.transition_to(GameMode::Dialog);
        self.dialog_page = 0;
        self.lock_camera_on(0); // ease 相机初始化到玩家
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
                self.lock_camera_on(self.dialog_speaker);
                if self.dialog_speaker != 0 && self.dialog_speaker != 87 {
                    let mut found = -1;
                    for e in (0..self.entities.count).rev() {
                        if self.entities.items[e].type_ == self.dialog_speaker {
                            found = e as i32;
                            break;
                        }
                    }
                    if found >= 0 {
                        self.ease_target_x = self.entities.items[found as usize].px;
                        self.ease_target_y = self.entities.items[found as usize].py;
                        // a.java:8667-8668（lockCameraOn 内）：型69 目标 X += 32
                        if self.dialog_speaker == 69 {
                            self.ease_target_x += 32;
                        }
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

    /// layoutWrappedText（a.java:4944-5019）带**字符上限**（Java 第 4 参 = 只排版前
    /// limit 个字符——打字机逐步放行的机制）：逐字换行（'\\c' 六位色 / '\\r' 转义
    /// 宽度跳过），页高 = maxLines/page。**本函数是该算法唯一实现**；
    /// [`Self::layout_full_text`] 即 limit=全长 的特例（与 Java 同构）。
    /// layoutWrappedText 打字机限量版（a.java:4944，limit=已打字符数）：
    /// 2026-10-17 单源适配——布局核 = `menu_family::layout_wrapped_text`
    ///（非法 `\c` 记账/单行内容宽等边缘语义以 Java 为准，见核心文档）。
    fn layout_wrapped_text_limited(&mut self, text: &str, w: i32, max_h: i32, limit: i32) {
        let t: Vec<u16> = text.encode_utf16().collect();
        self.apply_wrapped_layout(&t, w, max_h, limit.max(0) as usize);
    }

    /// 布局核适配（GameScene overlay 状态落盘口；行存 String 以保既有字段形状）。
    fn apply_wrapped_layout(&mut self, text: &[u16], w: i32, max_h: i32, limit: usize) {
        let font = crate::paint::paint_font();
        let r = crate::menu_family::layout_wrapped_text(text, w, max_h, limit, &font);
        self.overlay_page_top = 0;
        self.overlay_total_lines = r.total_lines;
        self.overlay_lines_per_page = r.lines_per_page;
        self.overlay_page_bottom = r.total_lines.min(r.lines_per_page);
        self.overlay_content_w = r.content_w;
        self.overlay_lines = r.lines.iter().map(|l| String::from_utf16_lossy(l)).collect();
    }

    /// flipOverlayPage（a.java:5129-5140）：页顶/页底钳制。
    /// handleOverlayKey（a.java:4925-4934）：-2/-1 = flipOverlayPage(top∓**1**)、
    /// 0 空、其余键关层。kind1/3/4/兜底 的非专用键统一走此口。
    /// 【2026-10-16 修正】原 kind3/4/兜底 误用 ±lines_per_page——翻页距只有
    /// kind0(15/17) 是 ±lines、kind5 与本口是 ±1（BUG-002 活码核查连带发现）。
    fn handle_overlay_key(&mut self) {
        match self.key_value {
            -2 => self.flip_overlay_page(self.overlay_page_top + 1),
            -1 => self.flip_overlay_page(self.overlay_page_top - 1),
            0 => {}
            _ => self.overlay_active = false,
        }
    }

    fn flip_overlay_page(&mut self, page: i32) {
        let mut top = page;
        if top >= 0 {
            if top > self.overlay_total_lines - self.overlay_lines_per_page {
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

    /// findPath（a.java:9892-9957）的场景外壳：Walkability = 地形 walkable 克隆 +
    /// 【T2】WalkTo(tile) 触屏寻路入口（touch-native-undo §0 操作模型）：
    /// 策略层 [`crate::pathfind::plan_route`]（直达/朝目标最远可达停靠/墙格探步入）
    /// → 喂 walkPathBuffer（dp3 消费机器=MOV 五参同款，零发明保真）。
    /// **新入口即打断**旧路线（手动优先）。返回计划（含探步入=T3 暗墙双通道接面）。
    /// 【BUG-038】场上非空闲（[`Self::field_tap_allowed`] 为假）⇒ 拒绝：零状态变化，返回空计划。
    pub fn walk_to(&mut self, tx: i32, ty: i32) -> crate::pathfind::RoutePlan {
        if !self.field_tap_allowed() {
            let at = (self.player_cell_x, self.player_cell_y);
            return crate::pathfind::RoutePlan { steps: Vec::new(), dock: at, reached: false, probe: None };
        }
        // 【STAGE-B-008 · 计划 2 T5 复核 R5】旧入口不继承点地后续（撞击 / 排队 / 就绪位），免得之后无关的收尾执行旧撞击
        self.tap_bump = None;
        self.tap_pending = None;
        self.tap_followup_ready = false;
        self.interrupt_route();
        self.walkto_draining = false; // 新路线 = 段重新开放（改道）
        let (cx, cy) = (self.player_cell_x, self.player_cell_y);
        let plan = {
            let walk = crate::pathfind::Walkability {
                terrain: self.walkable.clone(),
                has_entity_slot: &|x, y| {
                    let ct = self.grid.cell_type[y as usize][x as usize] as usize;
                    self.grid.capacity[ct] > 0
                },
                cols: self.view.wide,
                rows: self.view.high,
            };
            crate::pathfind::plan_route(&walk, (cx, cy), (tx, ty))
        };
        self.begin_walk_route(plan.steps.clone());
        plan
    }

    /// 【T2 / STAGE-B-008】按给定步序（目标端优先）开点地寻路：开路段、暂存教程箭头（BUG-043）、喂 dp3 消费机（Dialog 语境）。
    fn begin_walk_route(&mut self, steps: Vec<u8>) {
        let (cx, cy) = (self.player_cell_x, self.player_cell_y);
        self.route.begin((cx, cy));
        // 【BUG-043】箭头可见 ⇒ 先把箭头步序移出 walkPathBuffer（改道时已暂存，保留原暂存）
        if self.path_arrows_visible && self.walkto_arrows.is_none() {
            let n = (self.walk_step_count.max(0) as usize).min(self.walk_path_buffer.len());
            self.walkto_arrows = Some(self.walk_path_buffer[..n].to_vec());
        }
        self.walk_step_count = steps.len() as i32;
        self.walk_path_buffer = steps;
        self.script_walk_armed = true;
        self.dialog_phase = 3;
        self.walk_phase = 0;
        // dp3 消费机器在 Dialog 语境（op_mov 同款——脚本行走=mode 11 语境）
        self.game_mode = GameMode::Dialog;
        self.lock_camera_on(0);
    }

    /// 【STAGE-B-008】场上是否接收点地（壳据此决定点按走 [`Self::tap_cell`] 还是行命中 / 点空白）：
    /// 无浮层、非遮幅，且为地图态或点地寻路自身驱动的 dp3。手动行走 / 战斗 / 观察模式也接收（由 tap_cell 闪红或忽略）。
    pub fn accepts_field_tap(&self) -> bool {
        if self.overlay_active || self.transitioning {
            return false;
        }
        match self.game_mode {
            GameMode::Map => true,
            GameMode::Dialog => self.dialog_phase == 3 && self.walkto_owns_dp3(),
            _ => false,
        }
    }

    /// 【STAGE-B-008 · spec §5.2】点中格 (cx, cy)。
    pub fn tap_cell(&mut self, cx: i32, cy: i32) -> crate::tap_walk::TapResult {
        use crate::tap_walk::TapResult;
        if !self.accepts_field_tap() {
            return TapResult::Ignored;
        }
        let at = (self.player_cell_x, self.player_cell_y);
        if self.tap_walk_in_progress() {
            // 【spec §5.2 · 计划 2 T7 复核 R2】再点当前目标格（不是点自己）= 不打断、不排队、不动撞击 / 选中，原路线继续
            //（走向怪途中再点这只怪：照常开战，不降级为选中）。排队后再点排队目标同此（tap_target 已换到排队目标）。
            if self.tap_target == Some((cx, cy)) && (cx, cy) != at {
                return if self.tap_pending.is_some() {
                    TapResult::Queued
                } else if self.tap_bump.is_some() {
                    TapResult::Bumping
                } else {
                    TapResult::Walking
                };
            }
            // 【spec §5.2 排队】点地寻路中（含末步在途窗口，见 tap_walk_in_progress）：打断（BUG-040 排空：走完当前步回 Map wp0；
            // 末步窗口里路段已收、interrupt_route 为空操作），取消旧撞击；点自己 = 停下，点别处 = 记下新目标（只留最后一次）、
            // 目标框立即换到新目标，落格后的后续拍执行（tick_tap_state）。
            self.tap_bump = None;
            self.tap_selected = None;
            self.interrupt_route();
            if (cx, cy) == at {
                self.tap_pending = None;
                self.tap_target = None;
                return TapResult::Stopped;
            }
            self.tap_pending = Some(crate::tap_walk::PendingTap::Cell(cx, cy));
            self.tap_target = Some((cx, cy));
            return TapResult::Queued;
        }
        match self.walk_phase {
            0 if !self.script_walk_armed => {}
            2 => return TapResult::Ignored, // 观察模式：点按无效（拖动平移由壳接，计划 3）
            _ => {
                // 手动行走 / 战斗 / Map 语境的脚本行走（wp0 ∧ 武装，field_tap_allowed 同口径拒绝）：闪红、状态不变
                self.flash_reject((cx, cy));
                return TapResult::Rejected;
            }
        }
        // 新点按取代上一次点按尚未执行的撞击 / 排队后续（路线已收尾、后续排在下一拍开头的那一拍间隙里点按）
        self.tap_bump = None;
        self.tap_pending = None;
        self.tap_followup_ready = false;
        let grid = self.tap_grid();
        // 【STAGE-B-008 · spec §5.2 Q8】点中怪物：自动战斗关且不是已选中的那只 ⇒ 只选中（不动、不开路线、不置目标框）
        if let Some(crate::tap_walk::TapCell { kind: crate::tap_walk::TargetKind::Monster(e), .. }) = grid.get(cx, cy) {
            // 再点判据带有效性校验：实体表重排（跨层探针 / 换层重载）后、下一拍校验前的同帧窗口里，
            // 同下标可能已是另一只怪（计划 2 T6 复核 R2）
            let again = self.tap_selected.map_or(false, |s| s.entity == e && self.selection_valid(s));
            if !self.auto_battle && !again {
                let ent = &self.entities.items[e];
                self.tap_selected = Some(crate::tap_walk::SelectedEnemy {
                    entity: e,
                    type_: ent.type_,
                    cell: (ent.px >> 5, ent.py >> 5),
                    floor: self.current_floor,
                });
                self.tap_target = None;
                return TapResult::Selected;
            }
        }
        // 点别处 / 点自己 / 再点同一只 / 自动战斗：清选中，再执行这次点按
        self.tap_selected = None;
        if (cx, cy) == at {
            return TapResult::Ignored;
        }
        let plan = crate::tap_walk::plan_tap(&grid, at, (cx, cy));
        self.start_tap_plan(plan, (cx, cy))
    }

    /// 【STAGE-B-008 · spec §5.2 · 计划 2 T5 复核 R3】是否"点地寻路中"：dp3 归点地寻路（Dialog 语境），或多步路线的
    /// 末步在途窗口——末步由 advanceScriptWalk 在倒数第二格步末弹出，finish_walkto 随即回 Map，这一步在 Map wp1 走完
    ///（约 4 拍）。该窗口 [`Self::accepts_field_tap`] 为真（Map ⇒ true）。
    fn tap_walk_in_progress(&self) -> bool {
        (self.game_mode == GameMode::Dialog && self.dialog_phase == 3 && self.walkto_owns_dp3())
            || (self.game_mode == GameMode::Map && self.walk_phase == 1 && self.tap_followup_ready)
    }

    fn start_tap_plan(&mut self, plan: crate::tap_walk::TapPlan, target: (i32, i32)) -> crate::tap_walk::TapResult {
        use crate::tap_walk::{TapPlan, TapResult};
        match plan {
            TapPlan::Walk { steps, .. } => {
                self.walkto_draining = false;
                self.begin_walk_route(steps);
                self.tap_target = Some(target);
                TapResult::Walking
            }
            TapPlan::Bump { steps, dock, dir } => {
                self.tap_target = Some(target);
                if steps.is_empty() {
                    self.execute_bump(dir);
                } else {
                    self.walkto_draining = false;
                    self.tap_bump = Some((dock, dir));
                    self.begin_walk_route(steps);
                }
                TapResult::Bumping
            }
            TapPlan::Dock { steps, .. } => {
                self.flash_reject(target);
                if !steps.is_empty() {
                    self.walkto_draining = false;
                    self.begin_walk_route(steps);
                }
                TapResult::Docking
            }
            TapPlan::Ignore => TapResult::Ignored,
        }
    }

    /// 【STAGE-B-008 · R2-2】Map wp0 撞一步：置朝向 → 开路段（撞怪开战时记撤销快照）→ try_step。
    /// 未开战且被挡 ⇒ 收段（零长段不入栈）；走进去了 ⇒ Map wp1 步末收段。
    fn execute_bump(&mut self, dir: u8) {
        let at = (self.player_cell_x, self.player_cell_y);
        self.facing = dir as i32;
        self.route.begin(at);
        let ok = self.try_step(dir as i32);
        if self.walk_phase == 5 || ok {
            return;
        }
        self.route_arrived();
    }

    fn flash_reject(&mut self, cell: (i32, i32)) {
        self.tap_reject = Some((cell, self.frame_counter + crate::tap_walk::TAP_REJECT_TICKS));
    }

    /// 【STAGE-B-008】清点地状态（撤销 / 换层 / 剧情 / 按键时）。
    pub fn clear_tap_state(&mut self) {
        self.tap_target = None;
        self.tap_reject = None;
        self.tap_bump = None;
        self.tap_followup_ready = false;
        self.tap_selected = None;
        self.tap_pending = None;
    }

    /// 【STAGE-B-008 · spec §5.2】选中是否仍有效（每拍校验的判据）。
    fn selection_valid(&self, s: crate::tap_walk::SelectedEnemy) -> bool {
        if self.game_mode != GameMode::Map
            || self.walk_phase != 0
            || self.overlay_active
            || self.transitioning
            || self.current_floor != s.floor
            || s.entity >= self.entities.count
        {
            return false;
        }
        let ent = &self.entities.items[s.entity];
        !ent.removed && ent.state != 1 && ent.visible && ent.type_ == s.type_ && (ent.px >> 5, ent.py >> 5) == s.cell
    }

    /// 【STAGE-B-008 计划 3 T2 · spec §3.1】执行 [`crate::controls::Action`]（壳工具栏 / 摇杆标注消费，计划 3 T5/T6 接线）。
    /// 返回 true = 动作被受理（`Undo` 恒 false：撤销执行在壳 undo_ui——core 无撤销入口）。
    pub fn perform_action(&mut self, a: crate::controls::Action) -> bool {
        use crate::controls::Action;
        match a {
            // 键码动作 = 边沿键值（按钮无抬起事件，不走 press_key 的 keyHeld 锁存；菜单/确定类键
            // 在出现的情境里无 held 语义）。工具栏情境不含寻路中 ⇒ 无须打断。
            Action::Key(c) => {
                self.key_value = c;
                true
            }
            Action::Stop => {
                if self.tap_walk_in_progress() {
                    self.tap_bump = None;
                    self.tap_selected = None;
                    self.tap_pending = None;
                    self.tap_target = None;
                    self.interrupt_route();
                }
                true
            }
            Action::Cancel => {
                self.tap_selected = None;
                true
            }
            Action::Fight => {
                let _ = self.engage_selected();
                true
            }
            Action::StopThen(c) => {
                if self.tap_walk_in_progress() {
                    self.tap_bump = None;
                    self.tap_selected = None;
                    self.tap_target = None; // 【复核 T2-R5】打断 ⇒ 目标标记消失（spec §6）
                    self.interrupt_route();
                    self.tap_pending = Some(crate::tap_walk::PendingTap::StopThen(c));
                    true
                } else {
                    self.key_value = c;
                    true
                }
            }
            Action::Undo => false,
        }
    }

    /// 【STAGE-B-008 · spec §3.2「开战」】撞已选中的怪（工具栏按钮由计划 3 接入）。无有效选中 = Ignored。
    pub fn engage_selected(&mut self) -> crate::tap_walk::TapResult {
        match self.tap_selected {
            Some(s) if self.selection_valid(s) => self.tap_cell(s.cell.0, s.cell.1),
            _ => {
                self.tap_selected = None;
                crate::tap_walk::TapResult::Ignored
            }
        }
    }

    /// 【STAGE-B-008 · spec §5.3】滑动一格：地图空闲 = 朝 `dir` 探步（`probe_step`，先置朝向）；点地寻路中（含末步在途窗口）= 排队；
    /// 观察模式 / 浮层 / 非法方向忽略；手动行走 / 战斗 / Map 语境脚本行走 = Rejected（无目标格，不闪红）。
    pub fn swipe_step(&mut self, dir: u8) -> crate::tap_walk::TapResult {
        use crate::tap_walk::{PendingTap, TapResult};
        if !self.accepts_field_tap() || dir > 3 {
            return TapResult::Ignored;
        }
        if self.tap_walk_in_progress() {
            self.tap_bump = None;
            self.tap_selected = None;
            self.interrupt_route();
            self.tap_pending = Some(PendingTap::Swipe(dir));
            self.tap_target = None;
            return TapResult::Queued;
        }
        match self.walk_phase {
            0 if !self.script_walk_armed => {}
            2 => return TapResult::Ignored,
            _ => return TapResult::Rejected,
        }
        self.tap_bump = None;
        self.tap_pending = None;
        self.tap_selected = None;
        self.tap_target = None;
        if self.probe_step(dir) {
            TapResult::Walking
        } else {
            TapResult::Bumping
        }
    }

    /// 【STAGE-B-008】每拍开头：闪红到期；点地寻路收尾、玩家落格后的第一拍执行撞击（回到 Map wp0、无浮层 / 遮幅 / 新按键，
    /// 且玩家正在停靠格），否则作废；末步在途（Map wp1）或浮层开着时暂缓到下一拍再判（spec §5.1 T5 复核 R2）；
    /// 路线与撞击都结束即清目标标记。classic 下各字段恒空，零操作。
    fn tick_tap_state(&mut self) {
        // 【STAGE-B-008 · spec §5.2】选中每拍校验：任一条件不满足即清
        if let Some(s) = self.tap_selected {
            if !self.selection_valid(s) {
                self.tap_selected = None;
            }
        }
        if let Some((_, until)) = self.tap_reject {
            if self.frame_counter >= until {
                self.tap_reject = None;
            }
        }
        // 【spec §5.2 · 计划 2 T7 复核 R1】末步在途窗口（Map wp1 ∧ 后续就绪）里按住方向键：落格那一拍 handle_field_input 按 keyHeldCode
        // 同拍续走手动步，walk_phase 回 1，窗口判据分不清末步与手动步 ⇒ 手动输入优先，就绪位、撞击、排队一并作废
        //（浮层开着时按键属浮层，不判；与 press_key 同口径）。须在下列块之前：作废后本拍起即不再算"点地寻路中"。
        if self.tap_followup_ready
            && self.game_mode == GameMode::Map
            && self.walk_phase == 1
            && !self.overlay_active
            && self.key_held != 0
        {
            self.tap_followup_ready = false;
            self.tap_bump = None;
            self.tap_pending = None;
        }
        // 【spec §5.2 · 计划 2 T7】排队 / 撞击后续只在"点地寻路中"（寻路占 dp3，或末步在途窗口）或"收尾待执行"（含下面两种暂缓）
        // 期间有效：其余时刻的残留丢弃，免得之后无关的收尾误执行、目标框挂着不消失。
        // 【计划 2 T7 复核 R6】撞击后续在途中开战挂起（route.is_suspended）期间保留：胜利后路线续走到停靠格，照常撞击；
        // 排队仍按此规则（排队后排空只走完当前步，不会再撞出挂起）。挂起被壳层投键口径的 interrupt_route 解除（win route_key
        // 抬键）⇒ 不再保留，下一拍丢弃。
        if !self.tap_followup_ready && !self.tap_walk_in_progress() {
            self.tap_pending = None;
            if !self.route.is_suspended() {
                self.tap_bump = None;
            }
        }
        if self.tap_followup_ready
            && !(self.game_mode == GameMode::Map && self.walk_phase == 1)
            && !self.overlay_active
        {
            // 末步在途（Map wp1）时暂缓：多步路线的末步由 advanceScriptWalk 在倒数第二格步末弹出，
            // finish_walkto 随即回 Map、这一步仍在走（BUG-040 收尾口径）——等它落格后再判。
            // 【spec §5.1 · 计划 2 T5 复核 R2】浮层开着（停靠格本身弹拾取框）同样暂缓：保留就绪位与撞击 / 排队，
            // 关框后下一拍再按下列就绪条件判（关框键属浮层，press_key 不清撞击 / 排队）。
            // 排队优先于撞击（排队时已清撞击，这里只是兜底次序）。
            self.tap_followup_ready = false;
            let bump = self.tap_bump.take();
            let pending = self.tap_pending.take();
            let ready = self.game_mode == GameMode::Map
                && self.walk_phase == 0
                && !self.overlay_active
                && !self.transitioning
                && self.key_value == 0
                && self.key_held == 0;
            if ready {
                match pending {
                    Some(crate::tap_walk::PendingTap::Cell(x, y)) => {
                        let _ = self.tap_cell(x, y);
                    }
                    Some(crate::tap_walk::PendingTap::Swipe(d)) => {
                        let _ = self.swipe_step(d);
                    }
                    // 【计划 3 T2 · spec §3.2"停下后打开"】排空落 Map wp0：置键值待 Map 分派消费
                    //（直写 key_value 而非 press_key——按钮无抬起事件，press_key 的 keyHeld 锁存会
                    // 卡住点地后续的就绪判据 key_held==0）
                    Some(crate::tap_walk::PendingTap::StopThen(c)) => {
                        self.key_value = c;
                    }
                    None => {
                        if let Some((dock, dir)) = bump {
                            if (self.player_cell_x, self.player_cell_y) == dock {
                                self.execute_bump(dir);
                            }
                        }
                    }
                }
            }
        }
        if self.tap_target.is_some()
            && !(self.route.is_open()
                || self.walkto_draining
                || self.tap_bump.is_some()
                || self.tap_followup_ready
                || self.tap_pending.is_some())
        {
            self.tap_target = None;
        }
    }

    /// 【T2】探步入（一步撞击尝试）：暗墙命中=进入/揭示（挂撞击臂型 10/11/12/
    /// openWallOnRoute），实墙=撞停。**探墙双通道**（方向键踏步/触屏点墙滑动）
    /// 由 T3 发起——本入口为 core 端探步接口。新入口即打断旧路线。
    /// 【BUG-038】场上非空闲 ⇒ 拒绝（零状态变化，返回 false）。
    pub fn probe_step(&mut self, dir: u8) -> bool {
        if !self.field_tap_allowed() {
            return false;
        }
        // 【STAGE-B-008 · 计划 2 T5 复核 R5】同 walk_to：不继承点地后续
        self.tap_bump = None;
        self.tap_pending = None;
        self.tap_followup_ready = false;
        self.interrupt_route();
        // 【BUG-047】先置朝向再撞（与 handle_field_input 同式，scene.rs:5154-5155）：
        // 放行后 Map wp1 臂按 self.facing 位移；不写则按旧朝向走出一整格、且该方向不做碰撞（可穿墙）。
        self.facing = dir as i32;
        self.try_step(dir as i32)
    }

    /// 【STAGE-B-008 计划 3 T4 · spec §5.3】观察模式拖动：按拖动增量平移相机 + 主方向改 facing
    ///（与 wp2 keyHeld 平移臂同式：±16px/次、set_camera 原版钳位；a.java:3610-3642）。
    /// 仅 modern 布局（触屏模式专属）；其余态 = 零变化。返回实际平移量（钳位后）。
    pub fn pan_viewport(&mut self, dx: i32, dy: i32) -> (i32, i32) {
        if self.layout.preset != crate::layout::Preset::Modern
            || self.game_mode != GameMode::Map
            || self.walk_phase != 2
            || self.view_locked
            || self.overlay_active
            // 【复核 T4-R5】遮幅期防御门（生产 (wp2∧transitioning) 不可达；与 field_tap_allowed 同口径）
            || self.transitioning
        {
            return (0, 0);
        }
        if dx == 0 && dy == 0 {
            return (0, 0);
        }
        // 【复核 T4-R2 · 方向裁定（操作便利性）】渲染约定 screen=world+cam（a.java:7034）：
        // cam += 增量 ⇒ 内容屏幕坐标同向移动 =「内容跟手」（地图应用惯例；原版 wp2 键臂 cam−16
        // =「镜头跟手」是按键机取景框语义，触屏拖动不用）。facing 恒表镜头朝向：拖右（内容右移）
        // = 镜头左移 = facing 左(3)，与键臂（按右=镜头右=facing 2）恰好镜像、各自自洽。
        let (cx, cy) = (self.view.cam_x, self.view.cam_y);
        let (nx, ny) = (cx + dx, cy + dy);
        self.view.set_camera(nx, ny);
        let f = if dx.abs() >= dy.abs() {
            if dx > 0 { 3 } else { 2 }
        } else if dy > 0 {
            1
        } else if dy < 0 {
            0
        } else {
            self.facing
        };
        self.facing = f;
        (self.view.cam_x - cx, self.view.cam_y - cy)
    }

    /// 【BUG-038 · Stage B 触屏】场上是否接受点地动作（点格寻路 / 探步 / 撤销钮）：原版只在
    /// 地图行走态 walkPhase 0、无浮层时读场上输入（浮层分支吞键整拍返回 a.java:3252-3345；
    /// walkPhase 分派 a.java:3563-3679），换层遮幅期亦不行走。另一许可情形 = 点地寻路自身
    /// 驱动的 dp3 正处于格边界（段开放或打断后收尾中，见 BUG-040；新点地 = 改道，T2"手动优先"）。
    /// 壳层据此决定点按走场上通道。
    pub fn field_tap_allowed(&self) -> bool {
        if self.overlay_active || self.transitioning || self.walk_phase != 0 {
            return false;
        }
        match self.game_mode {
            GameMode::Map => !self.script_walk_armed,
            GameMode::Dialog => self.dialog_phase == 3 && self.walkto_owns_dp3(),
            _ => false,
        }
    }

    /// 【T2】打断终版（task-11）：寻路中任意输入皆打断——清 walkPathBuffer 停步
    /// （手动优先）。位移段收于**实际终点**入栈（中断=实际终点，T1 挂点契约）。
    /// 仅对 WalkTo/探步寻路段生效：经典脚本行走（MOV/dp2）key 链一行不动
    /// （plan §0 保真红线"key 行为链一行不动"）。
    /// > 【2026-10-26 更正·BUG-025】旧实现**无条件**清 walkPathBuffer/walkStepCount（与上句承诺相反）；
    /// > 壳层每次投键都先调本函数（android replay_scene_keys / win route_key / 触屏 Dispatcher）⇒
    /// > 剧情中连点"确定"截断脚本 MOV 的实体路径（dp2）与玩家脚本行走（dp3）——用户实机
    /// > "跳过教程后，嫦娥脚本行为异常"（一层 script19 嫦娥停在半路、后续 MOV/DES 落空）。
    /// > 现：仅寻路段开放或挂起（战斗/拾取后待续走）时清；否则零操作。
    /// > 【2026-10-30 更正·STAGE-B-008 复核 R1/R2】"实际终点"= 排空后的落格：步在途时打断，
    /// > 这一步照样走完（BUG-040 排空 / 胜利尾 restep），逻辑格到步末才更新——收段点取
    /// > [`Self::route_landing_cell`]，否则首步内 / restep 内打断被判成零长丢弃，走出的一格不可撤。
    pub fn interrupt_route(&mut self) {
        if !self.route.is_open() && !self.route.is_suspended() {
            return;
        }
        let at = self.route_landing_cell();
        self.route.interrupt(at, &mut self.undo);
        self.walk_path_buffer.clear();
        self.walk_step_count = 0;
        // 【BUG-040】寻路驱动的 dp3 被打断：dp3 仍归寻路，走完当前步即回 Map（见 finish_walkto）
        if self.game_mode == GameMode::Dialog && self.dialog_phase == 3 {
            self.walkto_draining = true;
        }
    }

    /// 【STAGE-B-008 · 复核 T4-R4】按键输入口的打断：浮层开着时零操作。原版浮层分支吞键、整拍返回
    ///（a.java:3252-3345），该键属浮层（关框 / 翻页），不是场上输入；BUG-040"手动优先"只对场上输入。
    /// ⇒ 点地路线途经拾取物弹框，关框后继续走向目标。换层 / 剧情接管等须无条件收段的调用方仍用
    /// [`Self::interrupt_route`]。
    pub fn interrupt_route_for_input(&mut self) {
        if self.overlay_active {
            return;
        }
        self.interrupt_route();
    }

    /// 【STAGE-B-008 · 复核 R1/R2】打断收段点 = 玩家最终落格。`walk_phase == 1`（步在途）⇒ 这一步必然走完：
    /// dp3 wp1 臂与 Map wp1 臂都按 facing 每拍 8px、满 32px 归格（逻辑格 = 像素 >> 5，步中 facing 不变），
    /// 落格 = 玩家格 + facing 增量（0 下 y+1、1 上 y−1、2 右 x+1、3 左 x−1）；否则 = 玩家格。
    fn route_landing_cell(&self) -> (i32, i32) {
        let (x, y) = (self.player_cell_x, self.player_cell_y);
        if self.walk_phase != 1 {
            return (x, y);
        }
        match self.facing {
            0 => (x, y + 1),
            1 => (x, y - 1),
            2 => (x + 1, y),
            3 => (x - 1, y),
            _ => (x, y),
        }
    }

    /// 【BUG-040 · T2】点地寻路收尾（dp3 步尽：到达或打断后走完当前步）：段终位入栈 + 回 Map，
    /// 并解除脚本行走武装——寻路首步由 dp3 wp0 弹出、不经 advanceScriptWalk 清武装（单步寻路
    /// 即全程如此），武装残留会让 Map 态下一次手动步进步末走 advanceScriptWalk 臂、不归
    /// walkPhase 0（实测：按一下方向键连走 3 格不停）。
    fn finish_walkto(&mut self) {
        self.route_arrived();
        self.restore_walkto_arrows();
        self.walkto_draining = false;
        self.script_walk_armed = false;
        self.game_mode = GameMode::Map;
        // 【STAGE-B-008】下一拍开头执行点地后续（撞击 / 排队）
        self.tap_followup_ready = true;
    }

    /// 【BUG-040】dp3 当前是否归点地寻路所有（段开放，或打断后仍在走完当前步）。
    pub(crate) fn walkto_owns_dp3(&self) -> bool {
        self.route.is_open() || self.walkto_draining
    }

    /// 路径箭头的步序源（buffer, 条数）：绘制（a.java:5929-5956）自玩家格沿 `buffer[..条数]` 自末端逐格外扩。
    /// 【BUG-043】点地寻路期间取暂存的箭头步序（walkPathBuffer 此时装的是寻路路线）。
    fn path_arrow_steps(&self) -> (&[u8], usize) {
        match &self.walkto_arrows {
            Some(steps) => (steps, steps.len()),
            None => (&self.walk_path_buffer, self.walk_step_count.max(0) as usize),
        }
    }

    /// 【BUG-043】点地寻路期间的 label216（a.java:5748-5761，applyStepCellEffects 步末）作用于暂存箭头：
    /// 顶步 == 刚走完那一步的朝向 ⇒ 消费一步；否则整条清空并熄箭头；箭头已尽亦熄——与方向键走同一路线时同规则同时机。
    /// 用 `walked`（步首朝向）而非当前 facing：寻路的下一步已在本步末弹出并转向，转弯处 facing 已是下一步方向
    ///（方向键一格一按时步末 facing 即刚走那一步，二者一致）。
    fn consume_walkto_arrow_step(&mut self, walked: i32) {
        if !self.path_arrows_visible {
            return;
        }
        let Some(steps) = self.walkto_arrows.as_mut() else { return };
        match steps.last() {
            Some(&top) if top as i32 == walked => {
                steps.pop();
            }
            Some(_) => {
                steps.clear();
                self.path_arrows_visible = false;
            }
            None => self.path_arrows_visible = false,
        }
    }

    /// 【BUG-043】寻路已释放 dp（段关闭、非收尾中、非事件挂起）而箭头仍暂存 ⇒ 放回（剧情接管后的延迟放回）。
    fn settle_walkto_arrows(&mut self) {
        if self.walkto_arrows.is_some() && !self.walkto_owns_dp3() && !self.route.is_suspended() {
            self.restore_walkto_arrows();
        }
    }

    /// 【BUG-043】点地寻路收尾 / 被剧情接管：余下箭头步序放回 walkPathBuffer（与方向键走到此处时同态）。
    /// 箭头途中已熄（偏离 / 浮层撞格清除）⇒ 只丢弃暂存、不动 walkPathBuffer。
    fn restore_walkto_arrows(&mut self) {
        if let Some(steps) = self.walkto_arrows.take() {
            if self.path_arrows_visible {
                self.walk_step_count = steps.len() as i32;
                self.walk_path_buffer = steps;
            }
        }
    }

    /// 路径箭头当前绘制在哪些格（自玩家格起，与绘制同源）；箭头不可见 ⇒ 空。
    pub fn path_arrow_cells(&self) -> Vec<(i32, i32)> {
        if !self.path_arrows_visible {
            return Vec::new();
        }
        let (buf, count) = self.path_arrow_steps();
        let (mut x, mut y) = (self.player_cell_x, self.player_cell_y);
        (0..count)
            .rev()
            .filter_map(|i| {
                match buf[i] {
                    0 => y += 1,
                    1 => y -= 1,
                    2 => x += 1,
                    3 => x -= 1,
                    _ => return None,
                }
                Some((x, y))
            })
            .collect()
    }

    /// 【T2】事件挂起（撞敌/拾取/开门）：段收于实际终点+挂起（T1 挂点契约）。
    /// 战斗笔由 [`Self::route_combat_finished`] 终结时入栈；拾取开门笔由 T3
    /// 交互面经 [`Self::route_pickup`] 落笔。
    pub fn route_event_suspend(&mut self) {
        if self.route.is_open() {
            let at = (self.player_cell_x, self.player_cell_y);
            self.route.suspend(at, &mut self.undo);
        }
    }

    /// 【T2】战斗终结入栈（T1 挂点契约：战斗终结才入栈）+挂起解除。
    pub fn route_combat_finished(&mut self, stroke: crate::undo_journal::CombatStroke) {
        self.route.finish_combat(stroke, &mut self.undo);
    }

    /// 【T2】拾取/开门笔挂点（T3 交互面调用：立即入栈）。
    pub fn route_pickup(&mut self, stroke: crate::undo_journal::PickupStroke) {
        self.undo.push_pickup(stroke);
    }

    /// 【T2】段终位（dp3 步序耗尽）：整段起终点入栈。
    pub fn route_arrived(&mut self) {
        if self.route.is_open() {
            let at = (self.player_cell_x, self.player_cell_y);
            self.route.arrive(at, &mut self.undo);
        }
    }

    /// 实体槽判定 canPathThroughEntity（格类型容量 > 0）。**恒**重写 walkPathBuffer/walkStepCount
    /// ——Java 头部 `walkPathBuffer = new byte[100]; walkStepCount = 0`（a.java:9894-9895），
    /// 回溯逐步填充（回溯失败 `return false` 时保留已填前缀）——返回 found。
    /// 调用方：SEE flag=1 / MOV 三参（玩家格起）/ MOV 五参（实体格起，返回值不判）。
    fn find_path(&mut self, fx: i32, fy: i32, tx: i32, ty: i32) -> bool {
        let result = {
            let walk = crate::pathfind::Walkability {
                terrain: self.walkable.clone(),
                has_entity_slot: &|x, y| {
                    let ct = self.grid.cell_type[y as usize][x as usize] as usize;
                    self.grid.capacity[ct] > 0
                },
                cols: self.view.wide,
                rows: self.view.high,
            };
            crate::pathfind::find_path(&walk, fx, fy, tx, ty)
        };
        self.walk_path_buffer = result.steps;
        self.walk_step_count = self.walk_path_buffer.len() as i32;
        self.walkto_arrows = None; // 【BUG-043】新路径（SEE 箭头 / MOV）取代任何暂存的旧箭头
        result.found
    }

    /// skipTextToLineEnd（a.java:5135-5148）：打字机跳到当前行尾（累计长度）。
    fn skip_text_to_line_end(&self, pos: i32) -> i32 {
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
        let script_cursor0 = self.script_cursor;
        if script_cursor0 + 3 > line.len() {
            // 行尾收尾（a.java:8597-8604）：phase=4 等相机 + 置 flag + 相机回玩家
            self.dialog_phase = 4;
            let cur = self.current_script_index;
            if cur != 32 {
                self.script_line_flags[cur] = true;
            }
            self.lock_camera_on(0);
            return;
        }
        // 命令模式（refactor-plan §3 第 2 步）：parse_op 纯解析（条件取参逐字节
        // 等价）→ execute_op 执行（逐 op 独立函数）。UNKNOWN ⇒ None ⇒ 停摆。
        let Some(op) = Self::parse_op(&line, script_cursor0) else {
            // 真未知 op（16 标签外）：Java 静默停摆（a.java:8594-8595 原地 return：
            // 无臂写 scriptCursor ⇒ 下一拍重解析同一 token）。禁止 3+N 跳字节续行。
            // 语义见 docs/spec/evidence-todos.md T-1 R-TODO-1b。
            return;
        };
        self.execute_op(op);
    }

    /// 纯解析层（游标契约，refactor-plan 条件 4）：条件取参清单逐字节等价——
    /// MOV var55>0 续取 3 参（五参）/ DES 负参分支 / ROS 子码条件取参 /
    /// GUT 无游标（load 重置）。每变体自带**推进后游标**（各 op 的 +1 口径
    /// 逐 op 锚注释）；16 标签外 ⇒ None（R-TODO-1b 停摆）。
    fn parse_op(line: &str, script_cursor0: usize) -> Option<ScriptOp> {
        let op = &line[script_cursor0..script_cursor0 + 3];
        // 与旧臂同口径：substring 后 script_cursor0 无条件 +3（a.java:8244-8239）——
        // 首参基址 = script_cursor0+1；SWD/MVS 游标 = script_cursor0+1
        let script_cursor0 = script_cursor0 + 3;
        let base = script_cursor0 + 1;
        Some(match op {
            // TAK_a_b（a.java:8257-8263）：游标 = 尾分隔位 + 1
            "TAK" => {
                let (start, e1) = Self::parse_script_int(line, base, '_');
                let (end, e2) = Self::parse_script_int(line, e1 + 1, ' ');
                ScriptOp::Tak { start, end, cursor: e2 + 1 }
            }
            // LAY_x（a.java:8355-8362）
            "LAY" => {
                let (floor, e) = Self::parse_script_int(line, base, ' ');
                ScriptOp::Lay { floor, cursor: e + 1 }
            }
            // GIN_a_b（a.java:8424-8437）：0=pickupItemType(b)/1=gainGold(b)；尾递归由执行层做
            "GIN" => {
                let (kind, e1) = Self::parse_script_int(line, base, '_');
                let (value, e2) = Self::parse_script_int(line, e1 + 1, ' ');
                ScriptOp::Gin { kind, value, cursor: e2 + 1 }
            }
            // RES_x（a.java:8470-8490）
            "RES" => {
                let (_v, e) = Self::parse_script_int(line, base, ' ');
                ScriptOp::Res { cursor: e + 1 }
            }
            // ADD_a_b_c_d（a.java:8438-8468）
            "ADD" => {
                let (a, e1) = Self::parse_script_int(line, base, '_');
                let (b, e2) = Self::parse_script_int(line, e1 + 1, '_');
                let (c, e3) = Self::parse_script_int(line, e2 + 1, '_');
                let (d, e4) = Self::parse_script_int(line, e3 + 1, ' ');
                ScriptOp::Add { floor: a, ty: b, cx: c, cy: d, cursor: e4 + 1 }
            }
            // SEE_a_b_c_d_e（a.java:8487-8510）：五参条件取参
            "SEE" => {
                let (tx, a) = Self::parse_script_int(line, base, '_');
                let (ty, b) = Self::parse_script_int(line, a + 1, '_');
                let (pg, c) = Self::parse_script_int(line, b + 1, '_');
                let (pg2, d) = Self::parse_script_int(line, c + 1, '_');
                let (flag, e) = Self::parse_script_int(line, d + 1, ' ');
                ScriptOp::See { tx, ty, page: pg, page_end: pg2, flag, cursor: e + 1 }
            }
            // GUT_n（a.java:8308-8311）：游标由 loadLevelScript 重置——解析层
            // 仍算尾位（执行层统一写后被 load 覆写，等价）
            "GUT" => {
                let (target, e) = Self::parse_script_int(line, base, ' ');
                ScriptOp::Gut { target, cursor: e + 1 }
            }
            // MOV（a.java:8274-8309）：**var55>0 续取 3 参**（条件取参清单①）
            "MOV" => {
                let (target_type, mut v2) = Self::parse_script_int(line, base, '_');
                let mut cell = None;
                if target_type > 0 {
                    let (cx, e) = Self::parse_script_int(line, v2 + 1, '_');
                    v2 = e;
                    let (cy, e) = Self::parse_script_int(line, v2 + 1, '_');
                    v2 = e;
                    cell = Some((cx, cy));
                }
                let (mx, e) = Self::parse_script_int(line, v2 + 1, '_');
                v2 = e;
                let (my, e) = Self::parse_script_int(line, v2 + 1, ' ');
                ScriptOp::Mov { target_type, cell, dst: (mx, my), cursor: e + 1 }
            }
            // DES（a.java:8311-8334）：**负参分支**（条件取参清单②）
            "DES" => {
                let (t0, v2) = Self::parse_script_int(line, base, '_');
                if t0 < 0 {
                    let (_, e) = Self::parse_script_int(line, v2 + 1, ' ');
                    ScriptOp::DesAll { want: -t0, cursor: e + 1 }
                } else {
                    let (cx, e1) = Self::parse_script_int(line, v2 + 1, '_');
                    let (cy, e2) = Self::parse_script_int(line, e1 + 1, ' ');
                    ScriptOp::DesAt { ty: t0, cx, cy, cursor: e2 + 1 }
                }
            }
            // SWD（a.java:8336-8352）：无参，游标 = script_cursor0+1
            "SWD" => ScriptOp::Swd { cursor: script_cursor0 + 1 },
            // CES_t_cx_cy（a.java:8386-8420）
            "CES" => {
                let (ty, v2) = Self::parse_script_int(line, base, '_');
                let (cx, e1) = Self::parse_script_int(line, v2 + 1, '_');
                let (cy, e2) = Self::parse_script_int(line, e1 + 1, ' ');
                ScriptOp::Ces { ty, cx, cy, cursor: e2 + 1 }
            }
            // GLV_n（a.java:8464-8466）
            "GLV" => {
                let (_v, e) = Self::parse_script_int(line, base, ' ');
                ScriptOp::Glv { cursor: e + 1 }
            }
            // END_n（a.java:8445-8463 区）
            "END" => {
                let (_v, e) = Self::parse_script_int(line, base, ' ');
                ScriptOp::End { cursor: e + 1 }
            }
            // SMS_n（a.java:8500s 区）：纯推进
            "SMS" => {
                let (_v, e) = Self::parse_script_int(line, base, ' ');
                ScriptOp::Sms { cursor: e + 1 }
            }
            // ROS_sub（a.java:8359-8381）：**子码条件取参**（条件取参清单④）——
            // 1=格坐标两参；2-6=单参；其余不取（v2 不动）；游标 = v2+1
            "ROS" => {
                let (sub, mut v2) = Self::parse_script_int(line, base, '_');
                match sub {
                    1 => {
                        let (px, e) = Self::parse_script_int(line, v2 + 1, '_');
                        v2 = e;
                        let (py, e) = Self::parse_script_int(line, v2 + 1, ' ');
                        v2 = e;
                        ScriptOp::RosMove { px, py, cursor: v2 + 1 }
                    }
                    2 => {
                        let (tier, e) = Self::parse_script_int(line, v2 + 1, ' ');
                        v2 = e;
                        ScriptOp::RosEquip { tier, cursor: v2 + 1 }
                    }
                    3 => {
                        let (v, e) = Self::parse_script_int(line, v2 + 1, ' ');
                        v2 = e;
                        ScriptOp::RosHp { v, cursor: v2 + 1 }
                    }
                    4 => {
                        let (v, e) = Self::parse_script_int(line, v2 + 1, ' ');
                        v2 = e;
                        ScriptOp::RosFace { v, cursor: v2 + 1 }
                    }
                    5 => {
                        let (v, e) = Self::parse_script_int(line, v2 + 1, ' ');
                        v2 = e;
                        ScriptOp::RosAtk { v, cursor: v2 + 1 }
                    }
                    6 => {
                        let (v, e) = Self::parse_script_int(line, v2 + 1, ' ');
                        v2 = e;
                        ScriptOp::RosDef { v, cursor: v2 + 1 }
                    }
                    _ => ScriptOp::RosNoop { cursor: v2 + 1 },
                }
            }
            // MVS（a.java:8519-8592）：无参，游标 = script_cursor0+1
            "MVS" => ScriptOp::Mvs { cursor: script_cursor0 + 1 },
            _ => return None,
        })
    }

    /// 执行层：统一写游标（解析层逐 op 计算）→ 逐 op 独立函数。
    /// GUT 的游标随后被 loadLevelScript 重置为 0（a.java:8308-8311 原样）。
    fn execute_op(&mut self, op: ScriptOp) {
        self.script_cursor = op.cursor();
        match op {
            ScriptOp::Tak { start, end, .. } => self.op_tak(start, end),
            ScriptOp::Lay { floor, .. } => self.op_lay(floor),
            ScriptOp::Gin { kind, value, .. } => self.op_gin(kind, value),
            ScriptOp::Res { .. } => self.op_res(),
            ScriptOp::Add { floor, ty, cx, cy, .. } => self.op_add(floor, ty, cx, cy),
            ScriptOp::See { tx, ty, page, page_end, flag, .. } => {
                self.op_see(tx, ty, page, page_end, flag)
            }
            ScriptOp::Gut { target, .. } => self.op_gut(target),
            ScriptOp::Mov { target_type, cell, dst, .. } => self.op_mov(target_type, cell, dst),
            ScriptOp::DesAll { want, .. } => self.op_des_all(want),
            ScriptOp::DesAt { ty, cx, cy, .. } => self.op_des_at(ty, cx, cy),
            ScriptOp::Swd { .. } => self.op_swd(),
            ScriptOp::Ces { ty, cx, cy, .. } => self.op_ces(ty, cx, cy),
            ScriptOp::Glv { .. } => self.op_glv(),
            ScriptOp::End { .. } => self.op_end(),
            ScriptOp::Sms { .. } => self.op_sms(),
            ScriptOp::RosMove { px, py, .. } => self.op_ros_move(px, py),
            ScriptOp::RosEquip { tier, .. } => self.op_ros_equip(tier),
            ScriptOp::RosHp { v, .. } => self.op_ros_hp(v),
            ScriptOp::RosFace { v, .. } => self.op_ros_face(v),
            ScriptOp::RosAtk { v, .. } => self.op_ros_atk(v),
            ScriptOp::RosDef { v, .. } => self.op_ros_def(v),
            ScriptOp::RosNoop { .. } => {}
            ScriptOp::Mvs { .. } => self.op_mvs(),
        }
    }

    /// TAK_a_b（a.java:8257-8263）：页区间 + 对话相位 1 + 开页（pageEnd 先写——
    /// 不先更新会误判"越界"而丢页）。
    fn op_tak(&mut self, start: i32, end: i32) {
        self.dialog_page = start;
        self.dialog_page_end = end;
        self.dialog_phase = 1;
        self.open_dialog_page(start);
    }

    /// LAY_x（a.java:8355-8362）：m_119 存态 + 目标层 + 遮幅（phase=5）+ 清 overlay。
    fn op_lay(&mut self, floor: i32) {
        self.save_floor_state();
        self.pending_floor = floor;
        self.transitioning = true;
        self.overlay_text = None;
        self.dialog_phase = 5;
    }

    /// GIN_a_b（a.java:8424-8437）：0=pickupItemType(b)/1=gainGold(b)；
    /// 完后**尾递归**执行下一条（8435-8436）。
    fn op_gin(&mut self, kind: i32, value: i32) {
        match kind {
            0 => self.pickup_item_type(value),
            1 => {
                crate::combat::gain_gold(&mut self.player, value);
            }
            _ => {}
        }
        self.execute_script_instruction();
    }

    /// RES_x（a.java:8470-8490）：easy（difficultyIndex==0）重置 run 统计；非 easy 空转。
    fn op_res(&mut self) {
        if self.difficulty_index == 0 {
            self.player.yellow_keys = 0;
            self.player.red_keys = 0;
            self.player.blue_keys = 0;
            self.player.item_stack_size = 0;
            self.player.gold = 4;
            self.alchemy_upgrade_count = 0;
            self.min_floor = 1;
            self.max_floor = 3;
            self.player.atk = 10;
            self.player.def = 10;
            self.player.hp = 400;
            self.journal.clear();
        }
    }

    /// ADD_a_b_c_d（a.java:8438-8468）：跨层生成实体——目标层非当前时
    /// m_119 存当前→m_122 载目标→spawn→m_119 存目标→还原（8462-8464）。
    fn op_add(&mut self, floor: i32, ty: i32, cx: i32, cy: i32) {
        if self.floor != floor {
            self.save_floor_state();
            if self.floor_assets.get(floor as usize).cloned().flatten().is_some() {
                self.reload_floor_entities(floor);
            }
        }
        entity::spawn(
            &mut self.entities,
            &mut self.grid,
            ty,
            cx << 5,
            cy << 5,
            0,
            &self.width_table,
            None,
        );
        self.save_floor_state();
        if self.floor != floor {
            if self.floor_assets.get(self.floor as usize).cloned().flatten().is_some() {
                self.reload_floor_entities(self.floor);
            }
        }
    }

    /// SEE_a_b_c_d_e（a.java:8487-8510）：pageStart/End 先解析入字段再开页
    ///（8487-8495）；flag==1 ⇒ findPath 成功 → 路径箭头 + walkPhase=0（8497-8501，
    /// case1 **无 break 落穿 default**：easeCameraToTarget 与箭头两者都做）；
    /// 收尾 phase=6 + easeCameraToTarget ease 相机目标 = 目标格。
    fn op_see(&mut self, tx: i32, ty: i32, page: i32, page_end: i32, flag: i32) {
        self.dialog_target = (tx, ty);
        self.dialog_page = page;
        self.dialog_page_end = page_end;
        self.open_dialog_page(page);
        if flag == 1 && self.find_path(self.player_cell_x, self.player_cell_y, tx, ty) {
            self.path_arrows_visible = true;
            self.walk_phase = 0;
        }
        self.dialog_phase = 6;
        self.ease_cam_x = ((self.view.view_w - 32) >> 1) - self.view.cam_x;
        self.ease_cam_y = ((self.view.view_h - 32) >> 1) - self.view.cam_y;
        self.ease_target_x = tx << 5;
        self.ease_target_y = ty << 5;
    }

    /// GUT_n（a.java:8308-8311）：旗标置位 + loadLevelScript(n)（游标由 load
    /// 重置为 0——Java 原样不前进）。
    fn op_gut(&mut self, target: i32) {
        let cur = self.current_script_index;
        self.script_line_flags[cur] = true;
        self.load_level_script(target);
    }

    /// MOV（a.java:8274-8309）：var55>0 ⇒ 五参实体路线（dp2/openWallOnRoute）；否则三参
    /// 玩家脚本行走（findPath 成功 → scriptWalkArmed+dp3+walkPhase=0+lockCameraOn(0)；
    /// 失败无相位副作用）。**miss 尾递归**（8301-8302）：同拍执行下一条。
    fn op_mov(&mut self, target_type: i32, cell: Option<(i32, i32)>, dst: (i32, i32)) {
        let (mx, my) = dst;
        self.dialog_target = (mx, my);
        if target_type > 0 {
            let (ent_x, ent_y) = cell.expect("五参已解析");
            // a.java:8289：route_entity = findBlockingEntity(...)（原 findBlockingEntity）——未命中也写入（-1）
            self.route_entity = self.entity_at_cell(ent_x, ent_y, target_type);
            if self.route_entity >= 0 {
                let ei = self.route_entity as usize;
                self.route_phase = 0; // routePhase = 0（a.java:8291）
                if self.entities.items[ei].type_ == 72 {
                    self.entities.items[ei].anim_row = 1; // a.java:8293-8294
                }
                // f_int_129/130 = 实体起点格（a.java:8297-8298）
                self.route_start = (self.entities.items[ei].px >> 5, self.entities.items[ei].py >> 5);
                // findPath(起点格 → 目标格)：返回值不判（a.java:8299）——openWallOnRoute
                // 只认 walkPathBuffer/walkStepCount（find_path 恒重写，含回溯
                // 失败的部分前缀）
                self.find_path(self.route_start.0, self.route_start.1, mx, my);
                self.dialog_phase = 2;
            } else {
                // 实体未找到 → 同拍尾递归执行下一条（a.java:8301-8302）
                self.execute_script_instruction();
            }
        } else if self.find_path(self.player_cell_x, self.player_cell_y, mx, my) {
            self.script_walk_armed = true;
            self.dialog_phase = 3;
            self.walk_phase = 0;
            self.lock_camera_on(0);
        }
    }

    /// DES 负参（a.java:8311-8320）：全表批量移除 -type。
    fn op_des_all(&mut self, want: i32) {
        for i in (0..self.entities.count).rev() {
            if self.entities.items[i].type_ == want && entity::remove(&mut self.entities, i) {
                self.build_minimap();
            }
        }
    }

    /// DES 正参（a.java:8321-8334）：格上首只该类型移除（findBlockingEntity + markEntityRemoved）。
    fn op_des_at(&mut self, ty: i32, cx: i32, cy: i32) {
        let idx = self.entity_at_cell(cx, cy, ty);
        if idx >= 0 && entity::remove(&mut self.entities, idx as usize) {
            self.build_minimap();
        }
    }

    /// SWD（a.java:8336-8352）：全表扫 81 → 显形 + 改型 4 + DES 状态动画
    ///（state=2 行 8——advance_frames 帧尽自解为常态）。
    fn op_swd(&mut self) {
        for i in (0..self.entities.count).rev() {
            if !self.entities.items[i].removed && self.entities.items[i].type_ == 81 {
                self.entities.items[i].visible = true;
                self.entities.items[i].type_ = 4;
                let img_h = self.images.entity.get(4).cloned().flatten().map(|im| im.height);
                entity::dispatch_sprite_size(4, i, &mut self.entities, &self.width_table, img_h);
                self.entities.items[i].state = 2;
                self.entities.items[i].anim_row = 8;
            }
        }
    }

    /// CES（a.java:8386-8420）：spawnEntity(type, cx<<5, cy<<5, 0) +
    /// markEntityDesRemoved + 相机回中后缓向新实体 + scriptStepDelay
    ///（cat 1/8/16=5、2/4=8、其余 0）。
    fn op_ces(&mut self, ty: i32, cx: i32, cy: i32) {
        let img_h = self.images.entity.get(ty as usize).cloned().flatten().map(|im| im.height);
        let idx = entity::spawn(
            &mut self.entities,
            &mut self.grid,
            ty,
            cx << 5,
            cy << 5,
            0,
            &self.width_table,
            img_h,
        );
        entity::remove_des(&mut self.entities, idx);
        self.ease_cam_x = ((self.view.view_w - 32) >> 1) - self.view.cam_x;
        self.ease_cam_y = ((self.view.view_h - 32) >> 1) - self.view.cam_y;
        // Java `if (sort_i >= 0)` 守卫（a.java:8398）：spawn 返回 usize 恒非负 ⇒ 恒真
        self.ease_target_x = self.entities.items[idx].px;
        self.ease_target_y = self.entities.items[idx].py;
        self.script_delay = match self.entity_category(ty) {
            1 | 8 | 16 => 5,
            2 | 4 => 8,
            _ => 0,
        };
    }

    /// GLV_n（a.java:8464-8466）：纯解析推进（无副作用）。
    fn op_glv(&mut self) {}

    /// END_n（a.java:8445-8463 区）：进结局态——dispatchModeEntry case 20（a.java:4443-4450）
    /// 可观测部分：typewriterPos=1 + 初次 layout（对象 = **当时 overlayText 残页**，
    /// Java 原样）；f_int_22/23/f_bool_04 不重置。
    fn op_end(&mut self) {
        self.transition_to(GameMode::Ending);
        self.dialog_char = 1;
        if let Some(t) = self.overlay_text.clone() {
            let line_h = crate::paint::paint_font().height + 4;
            self.layout_wrapped_text_limited(&t, 209, (line_h << 1) + 12, 1);
        }
    }

    /// SMS_n（a.java 纯推进）：无副作用。
    fn op_sms(&mut self) {}

    /// ROS_1（a.java:8361-8366）：玩家格坐标（px<<5/py<<5）+ 锁相机回玩家。
    fn op_ros_move(&mut self, px: i32, py: i32) {
        self.view.player_px = px << 5;
        self.view.player_py = py << 5;
        self.lock_camera_on(0);
    }

    /// ROS_2（a.java:8367-8369）：强制装备 tryEquip(tier,true)。
    fn op_ros_equip(&mut self, tier: i32) {
        self.try_equip(tier, true);
    }

    /// ROS_3（a.java:8370-8372）：HP。
    fn op_ros_hp(&mut self, v: i32) {
        self.player.hp = v;
    }

    /// ROS_4（a.java:8373-8375）：朝向（Java byte 字段 ⇒ i8 截断保真）。
    fn op_ros_face(&mut self, v: i32) {
        self.facing = (v as i8) as i32;
    }

    /// ROS_5（a.java:8376-8378）：ATK。
    fn op_ros_atk(&mut self, v: i32) {
        self.player.atk = v;
    }

    /// ROS_6（a.java:8379-8381）：DEF。
    fn op_ros_def(&mut self, v: i32) {
        self.player.def = v;
    }

    /// MVS（a.java:8519-8592）：型 82 传怪点成对互传。**双重死码**
    ///（语料 297 条指令 0 条 MVS + 全 55 层 sprite 型 82=0，
    /// evidence-todos.md T-1）——语料不可达，为忠实而保。体随迁移原样。
    fn op_mvs(&mut self) {

        // MVS（a.java:8519-8592）：型 82 传怪点成对互传。**双重死码**
        // （语料 297 条指令 0 条 MVS + 全 55 层 sprite 型 82=0，
        // evidence-todos.md T-1）——语料不可达，为忠实而保。
        // 倒序扫 type==82 按 param&255 两两配对进 16 对短缓存
        // （f_short_arr_04 等价，Java 每次 MVS 重建，a.java:8521-8522）；
        // 每对两向（8571-8573 交换重复）：扫描格槽组内 renderCategory==8
        // 怪物 → 双向 removeGroupAtCellByType(两格, 82)（82 点位消耗）
        // → carry_monsters_to 搬到对端。槽组循环 var8 为**循环前**容量快照、槽表
        // 就地压缩不清尾（a.java:6440-6443）⇒ stale 尾重触发 quirk 保真。
        let mut buf = [0i16; 32];
        for v6 in (0..self.entities.count as i32).rev() {
            if self.entities.items[v6 as usize].type_ == 82 {
                let key = self.entities.items[v6 as usize].param as i32 & 255;
                let mut v7 = 0;
                while v7 < 32 {
                    let v70 = buf[v7] as i32 - 1;
                    if v70 <= 0 {
                        buf[v7] = (v6 + 1) as i16;
                        break;
                    }
                    if key == (self.entities.items[v70 as usize].param as i32 & 255) {
                        buf[v7 + 1] = (v6 + 1) as i16;
                        break;
                    }
                    v7 += 2;
                }
            }
        }
        let mut v11 = 0;
        while v11 < 32 {
            let mut v70 = buf[v11] as i32;
            let mut v9 = buf[v11 + 1] as i32;
            if v70 <= 0 {
                break;
            }
            if v9 > 0 {
                v70 -= 1;
                v9 -= 1;
                let mut cx86 = self.entities.items[v70 as usize].px >> 5;
                let mut cy6 = self.entities.items[v70 as usize].py >> 5;
                let ct44 = self.grid.cell_type[cy6 as usize][cx86 as usize] as usize;
                let n8 = self.grid.capacity[ct44] as usize;
                let mut v12 = 0;
                while v12 < n8 {
                    let v10 = self.grid.slots[ct44][v12] as usize - 1;
                    let t56 = self.entities.items[v10].type_;
                    if entity::render_category(t56) == 8 {
                        let tx = self.entities.items[v9 as usize].px >> 5;
                        let ty = self.entities.items[v9 as usize].py >> 5;
                        entity::remove_group_at_cell_by_type(tx, ty, 82, &mut self.entities, &mut self.grid);
                        entity::remove_group_at_cell_by_type(cx86, cy6, 82, &mut self.entities, &mut self.grid);
                        self.carry_monsters_to(v10, tx, ty);
                    }
                    v12 += 1;
                }
                // A/B 交换（a.java:8571-8573）后重复反向
                core::mem::swap(&mut v70, &mut v9);
                cx86 = self.entities.items[v70 as usize].px >> 5;
                cy6 = self.entities.items[v70 as usize].py >> 5;
                let ct46 = self.grid.cell_type[cy6 as usize][cx86 as usize] as usize;
                let n8b = self.grid.capacity[ct46] as usize;
                let mut v93 = 0;
                while v93 < n8b {
                    let v92 = self.grid.slots[ct46][v93] as usize - 1;
                    let t56 = self.entities.items[v92].type_;
                    if entity::render_category(t56) == 8 {
                        let tx = self.entities.items[v9 as usize].px >> 5;
                        let ty = self.entities.items[v9 as usize].py >> 5;
                        entity::remove_group_at_cell_by_type(tx, ty, 82, &mut self.entities, &mut self.grid);
                        entity::remove_group_at_cell_by_type(cx86, cy6, 82, &mut self.entities, &mut self.grid);
                        self.carry_monsters_to(v92, tx, ty);
                    }
                    v93 += 1;
                }
            }
            v11 += 2;
        }
    }

    /// carry_monsters_to（a.java:8872-8878）：怪物搬运原语——同型同 param 在目标格
    /// spawnEntity（入参格坐标，Java 内部 <<5 转像素）+ 对**旧格全部同型**
    /// 实体 removeGroupAtCellByType 摘槽（"复制-删除" quirk：旧格同型多只
    /// 一并消失，Stage A 保真）。
    /// Java 反混淆原名 `carryMonstersTo`（对照性锚——P0 命名批 2026-10-19 机械改名，语义不变）。
    fn carry_monsters_to(&mut self, idx: usize, cx: i32, cy: i32) {
        let ox = self.entities.items[idx].px;
        let oy = self.entities.items[idx].py;
        let t = self.entities.items[idx].type_;
        let p = self.entities.items[idx].param;
        let img_h = self.images.entity.get(t as usize).cloned().flatten().map(|im| im.height);
        entity::spawn(&mut self.entities, &mut self.grid, t, cx << 5, cy << 5, p, &self.width_table, img_h);
        entity::remove_group_at_cell_by_type(ox >> 5, oy >> 5, t, &mut self.entities, &mut self.grid);
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

    /// 布局面换轨（STAGE-B-002 返修）：写入 layout 并同步**绘制坐标系**消费点
    /// （视口/画布尺寸+相机重钳+视口底线）——002 首版只连公式值未连绘制落点，
    /// 用户实机三破绽（地图只画左上块/属性浮屏中/元素各自为政）的根因面。
    pub fn apply_layout(&mut self, layout: crate::layout::Layout) {
        self.view.set_layout_dims(
            layout.game_view_w,
            layout.game_view_h,
            layout.screen_w,
            layout.screen_h,
        );
        self.view_bottom = layout.view_bottom_y;
        self.popups.set_gold_anchor(layout.gold_popup_x);
        self.layout = layout;
    }

    /// 底部面板水平偏移（lead 设计口径 2026-10-18：**经典尺寸固定+整体居中**）：
    /// 面板宽=经典全幅 240（左框 0..80/顶条 80..214/右缘 214..240 实测全幅），
    /// `dx=(screen_w−240)/2`——classic dx=0 回代原值（零偏离），modern 居中。
    fn bottom_panel_dx(&self) -> i32 {
        (self.layout.screen_w - crate::layout::SCREEN_W) / 2
    }

    /// buildMinimap（a.java:6879-6941）：小地图生成
    /// （wide×MINIMAP_PX_PER_CELL 见方，dim MINIMAP_DIM_ALPHA）。
    pub fn build_minimap(&mut self) {
        self.minimap = None;
        if !self.minimap_enabled {
            return;
        }
        let ppc = self.layout.minimap_px_per_cell;
        let w = self.view.wide * ppc;
        let h = self.view.high * ppc;
        let mut img = ArgbImage::create(w, h);
        {
            let mut g = SoftGraphics::new(&mut img);
            g.set_color(MINIMAP_BG);
            g.fill_rect(0, 0, w, h);
            for y in 0..self.view.high as usize {
                for x in 0..self.view.wide as usize {
                    let px = (x as i32) * ppc;
                    let py = (y as i32) * ppc;
                    if self.walkable[y][x] {
                        g.set_color(MINIMAP_WALKABLE);
                        g.fill_rect(px, py, ppc, ppc);
                    }
                    let ct = self.grid.cell_type[y][x] as usize;
                    if ct > 0 && self.grid.capacity[ct] > 0 {
                        let first = self.grid.slots[ct][0] as usize - 1;
                        if !self.entities.items[first].removed && self.entities.items[first].state != 1 {
                            match self.entities.items[first].type_ {
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
                                7 => m_003_clip5(&mut g, &self.images.ui[12], px - 1, py - 1, 5, w, h),
                                8 => m_003_clip5(&mut g, &self.images.ui[12], px - 1, py - 1, 0, w, h),
                                _ => {}
                            }
                        }
                    }
                }
            }
            // ④ R6 实体色点层（**modern 能力**，classic 零偏离不出图）：
            // 类型→色点映射（layout::minimap_dot_color）+ 同格高优先级胜
            //（玩家点在绘制面覆盖=最高层）。粒度 = 格内居中 minimap_dot_px 点。
            if self.layout.preset == crate::layout::Preset::Modern {
                let dot = self.layout.minimap_dot_px;
                let off = (ppc - dot) / 2;
                let cells = (self.view.wide * self.view.high) as usize;
                let mut best = vec![(0u8, 0u32); cells]; // (优先级, 色)
                for e in self.entities.items.iter() {
                    if e.removed || e.state == 1 || !e.visible {
                        continue;
                    }
                    if crate::layout::minimap_dot_suppressed(e.type_) {
                        continue; // 门/楼梯：基底层已可视化，不重复标注（用户裁定 2026-10-18）
                    }
                    let (cx, cy) = (e.px >> 5, e.py >> 5);
                    if cx < 0 || cy < 0 || cx >= self.view.wide || cy >= self.view.high {
                        continue;
                    }
                    let cat = crate::entity::render_category(e.type_);
                    let color = crate::layout::minimap_dot_color(cat);
                    let pri = crate::layout::minimap_dot_priority(cat);
                    if pri == 0 {
                        continue;
                    }
                    let cell = &mut best[(cy * self.view.wide + cx) as usize];
                    if pri > cell.0 {
                        *cell = (pri, color);
                    }
                }
                for (i, &(_, color)) in best.iter().enumerate() {
                    if color != 0 {
                        let (cx, cy) = ((i as i32) % self.view.wide, (i as i32) / self.view.wide);
                        g.set_color(color);
                        g.fill_rect(cx * ppc + off, cy * ppc + off, dot, dot);
                    }
                }
            }
        }
        self.minimap = Some(crate::intro::dim_image(&img, MINIMAP_DIM_ALPHA));
    }

    /// run case 11 相位机（a.java:4011-4068）。advanceEntityFrames 由调用方公共段跑。
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
                                    // easeCameraToTarget：ease 目标 = SEE 目标格
                                    self.ease_cam_x = ((self.view.view_w - 32) >> 1) - self.view.cam_x;
                                    self.ease_cam_y = ((self.view.view_h - 32) >> 1) - self.view.cam_y;
                                    self.ease_target_x = self.dialog_target.0 << 5;
                                    self.ease_target_y = self.dialog_target.1 << 5;
                                }
                            }
                        }
                    }
                }
            }
            2 => self.tick_entity_route(),
            3 => {
                // dp3 脚本行走（a.java:4077-4114）：wp0 弹路径步 tryStep /
                // 耗尽 → scriptWalkArmed=false + dp0；wp1 8px 步进（无边缘
                // 跟随，cameraTarget 随玩家）+ m_025 帧 + 32px 归格
                // applyStepCellEffects；wp5 战斗拍。
                if !self.transitioning {
                    match self.walk_phase {
                        0 => {
                            if self.walk_step_count > 0 {
                                // 【BUG-044】弹步前记"本步归点地寻路"：被挡步可能经剧情接管（型 83 → load_level_script 先收段）
                                // 让 walkto_owns_dp3() 在 try_step 之后变假，门控须看弹步时的归属（复核 T2-2）。
                                let owned = self.walkto_owns_dp3();
                                self.walk_step_count -= 1;
                                self.facing =
                                    self.walk_path_buffer[self.walk_step_count as usize] as i32;
                                self.walk_phase = 1;
                                let ok = self.try_step(self.facing);
                                // 【BUG-044】点地寻路专用：被挡且未开战（wp 仍为 1）⇒ 清步、归 wp0，下拍走 finish_walkto。
                                // 原版 MOV 消费机不判返回值（a.java:4080-4084，脚本路线由剧本保证可走）；
                                // 经典脚本行走不占用 dp3（owned 恒假），此分支不触达 classic 按键链。
                                if !ok && self.walk_phase == 1 && owned {
                                    self.walk_step_count = 0;
                                    self.walk_phase = 0;
                                    // 被挡步把模式切走（朝上撞炉 ⇒ Alchemy）：dp3 不再 tick、finish_walkto 等不到，就地收尾
                                    //（不写回 Map，保留炉页；复核 T2-1：否则武装残留 ⇒ 出页后点地全被拒、方向键连走穿墙）
                                    if self.game_mode != GameMode::Dialog && self.walkto_owns_dp3() {
                                        self.route_arrived();
                                        self.restore_walkto_arrows();
                                        self.walkto_draining = false;
                                        self.script_walk_armed = false;
                                        self.dialog_phase = 0;
                                    }
                                }
                            } else {
                                // 【T2】WalkTo 段终位（整段起终点入栈）+回 Map 模式；
                                // 经典脚本行走（route 未开放）零改动——mode 回归留脚本机器
                                // 【BUG-040】打断后的收尾（walkto_draining）同走寻路收尾
                                if self.walkto_owns_dp3() {
                                    self.finish_walkto();
                                } else {
                                    self.route_arrived();
                                }
                                self.script_walk_armed = false;
                                self.dialog_phase = 0;
                            }
                        }
                        1 => {
                            match self.facing {
                                0 => self.view.player_py += 8,
                                1 => self.view.player_py -= 8,
                                2 => self.view.player_px += 8,
                                _ => self.view.player_px -= 8,
                            }
                            self.step_progress += 8;
                            self.ease_target_x = self.view.player_px;
                            self.ease_target_y = self.view.player_py;
                            if self.player_frame < PLAYER_FRAME_TABLE.len() as i32 - 1 {
                                self.player_frame += 1;
                            } else {
                                self.player_frame = 0;
                            }
                            if self.step_progress >= 32 {
                                self.step_progress = 0;
                                self.player_frame = 0;
                                self.walk_phase = 0;
                                self.player_cell_x = self.view.player_px >> 5;
                                self.player_cell_y = self.view.player_py >> 5;
                                self.apply_step_cell_effects();
                                // 【T2】WalkTo 段终位：步尽归格=实际终点（整段起终点
                                // 入栈）+回 Map；经典脚本行走 route 未开放=零改动
                                // 【BUG-040】打断后走完当前步同此收尾；收尾解除武装
                                if self.walk_step_count == 0 && self.walkto_owns_dp3() {
                                    self.finish_walkto();
                                }
                            }
                        }
                        5 => self.tick_battle(self.battle_active),
                        _ => {}
                    }
                }
            }
            4 => {
                if self.camera_at_target() {
                    self.transition_to(GameMode::Map);
                }
            }
            5 => {
                // LAY 批续跑缝（a.java:4123-4127）：floorTransitioning ∧ !wipeClosing
                // （换层已过、遮幅开启中）⇒ 相位归 0 续跑脚本。op_lay 停表于此；
                // 2026-10-17 前为空臂桩（"floorTransitioning 组合（LAY 批）"未译）⇒ 剧情链
                // 永停（用户实机 BUG-016：3→2 天牢切层后牛魔王对话不触发 + 输入假死）。
                if self.transitioning && !self.wipe_closing {
                    self.dialog_phase = 0;
                }
            }
            _ => {}
        }
    }

    /// openWallOnRoute（a.java:8738-8870）：dp2 实体路径缓动（MOV 五参后每拍一次，run case 11
    /// dialogPhase 2 臂 a.java:4072-4073）。子相位 routePhase（[`Self::route_phase`]）：
    ///
    /// - **0 弹步**（8740-8810）：walkStepCount>0 → 弹顶步为 f_byte_19，预算下一格；
    ///   routePhase=1；型≠72 且下一格 = 玩家格 → 可胜（predictHpLossVsType(t,false)
    ///   ∈[0,hp)）开**途中战斗**（缩放表 HP、walkPhase=5、battleActiveFlag=false、
    ///   battleTarget、state=5、routePhase=5、battlePredictGate=false、敌先手一击
    ///   atk−def>0 扣血），不可胜 → walkPhase=1 + 朝向=步向（玩家此刻不动，mode 3 才
    ///   消费）；随后下一格槽表**倒序**扫 cat1 型11（墙）→ markEntityRemoved（开墙）。
    ///   步尽 → relocateSlotsAtRouteEnd 迁槽 + dialogPhase=0 + dispatchAnimFields（复位动画行）+ route_entity=-1。
    /// - **1 步进**（8811-8859）：实体 ±8px、相机目标 = 实体像素、routeProgress+=8、动画行
    ///   >0 时帧 +1（表尾归 0）；满 32 → 归格（routeProgress=0、routePhase=0），实体落到
    ///   > 玩家格且型≠72：可胜 → 玩家直扣预测损血 + relocateSlotsAtRouteEnd + removeGroupAtCellByType
    ///   > （无死亡动画），不可胜 → scriptWalkArmed=false、**gameMode=3**、walkPhase=1、
    ///   > 朝向=步向（dialogPhase 仍为 2——路径挂起）；最后**同拍递归** openWallOnRoute()（弹下一步
    ///   > 或收尾）。
    /// - **5 战斗**（8861-8868）：tickBattle(battleActiveFlag)；非收尾中且实体已移除/
    ///   state==1 → dialogPhase=0、routePhase=0、route_entity=-1（路径余步丢弃，无 relocateSlotsAtRouteEnd）。
    fn tick_entity_route(&mut self) {
        match self.route_phase {
            0 => {
                let e = self.route_entity as usize;
                if self.walk_step_count > 0 {
                    self.walk_step_count -= 1;
                    self.route_dir = self.walk_path_buffer[self.walk_step_count as usize] as i32;
                    let mut nx = self.entities.items[e].px >> 5;
                    let mut ny = self.entities.items[e].py >> 5;
                    match self.route_dir {
                        0 => ny += 1,
                        1 => ny -= 1,
                        2 => nx += 1,
                        3 => nx -= 1,
                        _ => {}
                    }
                    self.route_phase = 1;
                    let t = self.entities.items[e].type_;
                    if t != 72 && nx == self.player_cell_x && ny == self.player_cell_y {
                        let loss = self.predict_hp_loss(t, false);
                        if loss >= 0 && loss < self.player.hp {
                            let ti = (t - 41) as usize;
                            self.battle_enemy_hp = self.enemies.hp[ti]; // enemyHpScaled（a.java:8769）
                            self.walk_phase = 5;
                            self.battle_active = false;
                            self.battle_target = self.route_entity;
                            self.entities.items[e].state = 5;
                            self.route_phase = 5;
                            self.battle_no_counter = false;
                            let strike = self.enemies.atk[ti] - self.player.def;
                            if strike > 0 {
                                self.player.hp -= strike;
                            }
                        } else {
                            self.walk_phase = 1;
                            self.facing = self.route_dir;
                        }
                    }
                    // 下一格槽表倒序扫：cat1 型11 → markEntityRemoved（a.java:8786-8802；
                    // 计数先取后扫，markEntityRemoved 不动槽表）
                    let ct = self.grid.cell_type[ny as usize][nx as usize] as usize;
                    let mut k = self.grid.capacity[ct] as i32;
                    while k > 0 {
                        k -= 1;
                        let id = self.grid.slots[ct][k as usize] as usize - 1;
                        let tt = self.entities.items[id].type_;
                        if self.entity_category(tt) == 1 && tt == 11 && entity::remove(&mut self.entities, id) {
                            self.build_minimap();
                        }
                    }
                    return;
                }
                let (sx, sy) = self.route_start;
                entity::relocate_slot(&mut self.grid, &self.entities, e, sx, sy);
                self.dialog_phase = 0;
                let t = self.entities.items[e].type_;
                entity::dispatch_anim(t, e, &mut self.entities);
                self.route_entity = -1;
            }
            1 => {
                let e = self.route_entity as usize;
                match self.route_dir {
                    0 => self.entities.items[e].py += 8,
                    1 => self.entities.items[e].py -= 8,
                    2 => self.entities.items[e].px += 8,
                    3 => self.entities.items[e].px -= 8,
                    _ => {}
                }
                self.ease_target_x = self.entities.items[e].px;
                self.ease_target_y = self.entities.items[e].py;
                self.route_progress += 8;
                let row = self.entities.items[e].anim_row;
                if row > 0 {
                    if self.entities.items[e].anim_frame < ANIM_OFFSET_TABLE[row as usize].len() as i32 - 1 {
                        self.entities.items[e].anim_frame += 1;
                    } else {
                        self.entities.items[e].anim_frame = 0;
                    }
                }
                if self.route_progress == 32 {
                    self.route_progress = 0;
                    self.route_phase = 0;
                    let cx = self.entities.items[e].px >> 5;
                    let cy = self.entities.items[e].py >> 5;
                    let t = self.entities.items[e].type_;
                    if cx == self.player_cell_x && cy == self.player_cell_y && t != 72 {
                        let loss = self.predict_hp_loss(t, false);
                        if loss >= 0 && loss < self.player.hp {
                            self.player.hp -= loss;
                            let (sx, sy) = self.route_start;
                            entity::relocate_slot(&mut self.grid, &self.entities, e, sx, sy);
                            let t = self.entities.items[e].type_;
                            entity::remove_group_at_cell_by_type(
                                cx,
                                cy,
                                t,
                                &mut self.entities,
                                &mut self.grid,
                            );
                        } else {
                            self.script_walk_armed = false;
                            self.transition_to(GameMode::Map);
                            self.walk_phase = 1;
                            self.facing = self.route_dir;
                        }
                    }
                    self.tick_entity_route(); // a.java:8858 同拍递归
                }
            }
            5 => {
                self.tick_battle(self.battle_active);
                let e = self.route_entity as usize;
                if !self.battle_finishing && (self.entities.items[e].removed || self.entities.items[e].state == 1) {
                    self.dialog_phase = 0;
                    self.route_phase = 0;
                    self.route_entity = -1;
                }
            }
            _ => {}
        }
    }

    /// resetMenuLayout（a.java:4690-4702）。
    fn reset_menu_layout(&mut self) {
        let font = crate::paint::paint_font();
        self.menu_line_height = font.height() + 6;
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

    /// hideNotify 游戏态半（a.java:4502-4513；f_bool_01 latch 与触发判断由壳承担）：
    /// 4504-4507 `if (gameMode==3){gameMode=4; dispatchModeEntry();}` = 切 mode 4 + 进游戏内菜单
    /// （dispatchModeEntry case 4 = enter_menu4）；4509-4510 closeAudioPlayer + anyKeyPlayed=true
    /// 由 BgmState::on_hide_notify 承担。Java 顺序 = 菜单先、音频后。
    pub fn hide_notify(&mut self) {
        if self.game_mode == GameMode::Map {
            self.transition_to(GameMode::InGameMenu);
            self.enter_menu4();
        }
        self.bgm.on_hide_notify();
    }

    /// SKY_WAR 快照（[0]=soundEnabled 镜像 a.java:7284；[1]=minimap_enabled）。
    pub fn global_snapshot(&self) -> crate::save::GlobalSave {
        let mut g = self.global.clone();
        g.option_checked[0] = self.bgm.sound_enabled;
        g.option_checked[1] = self.minimap_enabled;
        g
    }

    /// 消费脏位。
    pub fn take_global_dirty(&mut self) -> bool {
        std::mem::take(&mut self.global_dirty)
    }

    /// 装载/交接灌入（不置脏）。
    pub fn apply_global(&mut self, g: crate::save::GlobalSave) {
        self.global = g;
        self.global_dirty = false;
    }

    /// 水位抬升（a.java:7163-7164：`floor<51 && floor>水位`）。
    pub fn note_max_floor(&mut self, floor: i32) {
        if floor < 51 && floor > self.global.max_floor_watermark {
            self.global.max_floor_watermark = floor;
            self.global_dirty = true;
        }
    }

    /// dispatchModeEntry case 4（a.java:4326-4337）：继续游戏（+floor>50 的跳过教程）/
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

    /// dispatchModeEntry case 19（a.java:4439-4448）：商店 5 条目。
    fn enter_menu19(&mut self) {
        self.reset_menu_layout();
        for k in 12..=16 {
            self.add_menu_item(k);
        }
    }

    /// run case 4（a.java:3686-3731）：-7 返回、-6/-5 激活、-2/-4 下移、-1/-3 上移。
    fn tick_menu4(&mut self) {
        match self.key_value {
            -7 => self.transition_to(self.in_game_menu_return_mode),
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
                self.transition_to(GameMode::InGameMenu);
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
            1 => self.transition_to(GameMode::Map),
            3 => {
                // a.java:4776-4780：gameMode=7 + menuReturnMode=4 + dispatchModeEntry
                self.menu_return_mode = GameMode::InGameMenu;
                self.transition_to(GameMode::SaveSlot);
                self.enter_menu7();
            }
            4 => {
                // a.java:4781-4785：gameMode=16 + menuReturnMode=4 + dispatchModeEntry
                self.menu_return_mode = GameMode::InGameMenu;
                self.transition_to(GameMode::About);
                self.enter_menu16();
            }
            5 => {
                // a.java:4786-4790：gameMode=15 + menuReturnMode=4 + dispatchModeEntry
                self.menu_return_mode = GameMode::InGameMenu;
                self.transition_to(GameMode::Help);
                self.enter_menu15();
            }
            11 => {
                self.transition_to(GameMode::Shop);
                self.enter_menu19();
            }
            17 => {
                self.change_floor(1, false, false);
                self.transition_to(GameMode::Map);
                self.player.hp = 1000;
                self.player.atk = 710;
                self.player.def = 710;
            }
            7 => {
                // 退出→回标题（Java gameMode=1，run case 4 家族 a.java:4776-4790 区
                // "7→mode1（退出标题）"）：跨机信号——game_mode=MainMenu 即换机请求，
                // shell 换回 BootMachine（stash 保标题态，CMidlet 同对象语义）。
                // 2026-10-18 用户实报：原落 other 臂空转（"待跨机模式批次"）＝退出无反应。
                self.transition_to(GameMode::MainMenu);
            }
            other => {
                let _ = other; // 其余 kind：待跨机模式批次
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
        // STAGE-B-004b ①③：菜单框锚位公式化（menu_family::in_game_menu_box）——
        // classic 代入（center=SCREEN 系 120/160）逐值化回 a.java:2431-2444 原值；
        // modern 随 layout.center 重排，高亮/箭头随框派生。
        let mb = crate::menu_family::in_game_menu_box(
            self.layout.center_x,
            self.layout.center_y,
            self.menu_box_width,
            self.menu_list_height,
            self.menu_line_height,
            font.height(),
        );
        let (box_w, box_h) = (mb.box_w, mb.box_h);
        crate::menu_family::paint_box_frame(g, &self.images.ui[0], mb.bx, mb.by, box_w, box_h);
        let pad = mb.pad;
        let mut by = mb.first_row_y;
        let mut i = self.menu_scroll_top;
        while i < self.menu_visible_bottom {
            let kind = self.menu_kinds[i as usize];
            if self.menu_cursor == i {
                g.set_color(0);
                g.fill_rect(mb.hl_x, by - pad, mb.hl_w, self.menu_line_height);
                g.set_color(16377897);
            } else if kind != 11 {
                g.set_color(7574946);
            } else {
                g.set_color(16751103);
            }
            if kind > 11 && kind < 17 {
                // 商店条目：逐字彩虹（shopRainbowColors[shopRainbowPhase 起步环绕]），
                // 相位在 paint 尾按 fc&3 推进（a.java:2476-2479）
                let label = MENU_LABELS[kind as usize];
                let n = RAINBOW_COLORS.len();
                let label_u: Vec<u16> = label.encode_utf16().collect();
                let mut cx = mb.item_cx - (font.string_width(&label_u) >> 1);
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
                g.draw_string(&label, mb.item_cx, by, 17);
            }
            by += self.menu_line_height;
            i += 1;
        }
        let (ax, ay) = (mb.arrow_x, mb.arrow_y);
        if self.menu_scroll_top > 0 {
            draw_image_clipped(g, &self.images.ui[15], ax, ay, 0, 0, 7, 9);
        }
        if self.menu_visible_bottom < self.menu_item_count {
            draw_image_clipped(g, &self.images.ui[15], ax, ay + 9, 7, 0, 7, 9);
        }
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
    }

    /// dispatchModeEntry case 10（a.java:4370-4380）：道具面板布局初始化。
    fn enter_item_panel(&mut self) {
        let font = crate::paint::paint_font();
        self.item_cols = 4;
        self.item_rows_total = 32 / self.item_cols;
        self.item_rows_visible = (268 - font.height() - 10) / 40;
        if self.item_rows_visible > self.item_rows_total {
            self.item_rows_visible = self.item_rows_total;
        }
        self.item_scroll_top = 0;
        self.item_scroll_bottom = self.item_rows_visible;
        self.item_box_w = 42 + 40 * self.item_cols;
        self.item_box_h = 34 + font.height() + 10 + 40 * self.item_rows_visible;
        self.softkeys = (1, 3);
    }

    /// run case 10（a.java:3931-4009）：-7 返回 mode 3；-6/-5 使用道具
    /// （13/14 直接生效回 mode 3、15/23/24/25 浮层(0,3)、其余浮层(1,2)）；
    /// -4/-3 行内环绕、-2/-1 翻行+滚动窗跟随。
    fn tick_item_panel(&mut self) {
        match self.key_value {
            -7 => self.transition_to(GameMode::Map),
            -6 | -5 => {
                if self.item_cursor < self.player.item_stack_size {
                    let t = self.player.item_types[self.item_cursor as usize] as i32;
                    // itemUseTargetType（a.java:3941）：确认浮层图标/名称的数据源
                    self.overlay_kind_item = t;
                    match t {
                        13 | 14 => {
                            self.transition_to(GameMode::Map);
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
    /// 14 → gameMode=12（dispatchModeEntry 布局侧效果随对应模式批次）。16-25 的效果
    /// 分支（破墙/回血/瞬移等）为道具批次待办——序章背包恒空不可达。
    /// activateItem（a.java:7795-7860）：返回是否**消耗**（script_cursor0——true 时
    /// useItemStack 递减次数并在 0 时压实栈）。
    fn activate_item(&mut self, t: i32) -> bool {
        match t {
            13 => {
                // a.java:7786-7788：gameMode=5 + dispatchModeEntry case5（m_091+软键）
                self.transition_to(GameMode::EnemyInfo);
                self.enter_menu5();
                false
            }
            14 => {
                // a.java:7789-7791：gameMode=12 + dispatchModeEntry case12（a.java:4386-4388：
                // browseOfferQueue(0) + softkeyRightKind=3）
                self.transition_to(GameMode::RecordBrowser);
                self.open_record_page(0);
                self.softkeys.1 = 3;
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
                    self.transition_to(GameMode::Map);
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
                    if self.entities.items[e].type_ == 11
                        && entity::remove(&mut self.entities, e)
                    {
                        self.build_minimap();
                    }
                }
                self.transition_to(GameMode::Map);
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
                    if self.entities.items[e].type_ == 1 && entity::remove(&mut self.entities, e) {
                        self.build_minimap();
                    }
                }
                self.transition_to(GameMode::Map);
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
                self.transition_to(GameMode::Map);
                true
            }
            20 => {
                // a.java:7832-7850：对角瞬移——40 层禁用；镜像格 interact 放行
                // → place_player_at_cell + 相机居中 + applyStepCellEffects + 消耗；
                // 拦截且无浮层 → 「无法移动到该位置」不消耗
                if self.floor == 40 {
                    self.show_overlay_message(0, "本层不能直接瞬移。", 0, 0);
                    false
                } else {
                    let tx = self.view.wide - 1 - (self.view.player_px >> 5);
                    let ty = self.view.high - 1 - (self.view.player_py >> 5);
                    let pass = self.interact_with_cell(tx, ty);
                    if pass {
                        self.place_player_at_cell(tx, ty);
                        self.view.set_camera(
                            ((self.view.view_w - 32) >> 1) - self.view.player_px,
                            ((self.view.view_h - 32) >> 1) - self.view.player_py,
                        );
                        self.apply_step_cell_effects();
                        self.transition_to(GameMode::Map);
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
                // a.java:7863-7866：changeFloor(+1)（返回值即消耗与否）+ keyHeldCode=0 + gameMode=3
                //（面板内道具触发路径不停留 mode 10——A-preset-item21 T402 实证；mode 3 归本调用方，BUG-026）
                let consumed = self.change_floor(self.floor + 1, false, false);
                self.key_held = 0;
                self.transition_to(GameMode::Map);
                consumed
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
            let t = self.entities.items[e].type_;
            if t == 5 {
                let param = self.entities.items[e].param;
                self.remove_slot_at(cx, cy, slot);
                if self.try_open_chest(param) {
                    self.walk_phase = 4;
                }
            } else if t < 67
                && self.entity_category(t) == 8
                && self.entities.items[e].state != 1
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

    /// findBlockingEntity 语义（a.java:cellSlotTable 查找）：格 (cx,cy) 上首个型 target
    /// 且 state!=1 的实体下标，无则 -1。
    fn find_entity_of_type_at(&self, cx: i32, cy: i32, target: i32) -> i32 {
        if cx < 0 || cy < 0 || cx >= self.view.wide || cy >= self.view.high {
            return -1;
        }
        let ct = self.grid.cell_type[cy as usize][cx as usize] as usize;
        let cap = self.grid.capacity[ct] as usize;
        for s in 0..cap {
            let e = self.grid.slots[ct][s] as usize - 1;
            if self.entities.items[e].type_ == target && self.entities.items[e].state != 1 {
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
                if entity::remove(&mut self.entities, e as usize) {
                    self.build_minimap();
                }
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
        // 【BUG-045】道具使用（消耗 / 瞬移 / 换层 / 回血）不记账 ⇒ 清栈（复核 T3-R4）
        self.undo.clear(crate::undo_journal::ClearReason::Irreversible);
        // itemStackUses = **有符号 byte**（a.java:7762）：-1 = 无限次（走
        // else 无条件 activate 不计数）——u8 直读会把 -1 读成 255 误计数。
        if (self.player.item_uses[idx] as i8) > 0 {
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

    /// itemTypeToStackIndex（a.java:7705-7719 语义：85/86 特判，其余 -13）。
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
        // 斜线底纹（a.java:2725-2734）：Java 方法内留有未用局部 `short var141 = 320;
        // short var122 = 240;`——源码以 H/W 局部常量书写、javac 常量折叠的残迹
        //（B 级），故 319/318/239 = H-1/H-2/W-1 一并按屏幕派生书写。
        let mut v159 = self.layout.screen_h - 1;
        while v159 >= 1 {
            g.draw_line(-1, -1 + (self.layout.screen_h - v159), -1 + v159 - 1, self.layout.screen_h - 2);
            g.draw_line(self.layout.screen_w - 1 - v159, -1, self.layout.screen_w - 1, -1 + v159);
            v159 -= 2;
        }
        let mut x = (self.layout.screen_w - self.item_box_w) >> 1;
        let mut y = (self.layout.screen_h - self.item_box_h) >> 1;
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
        y += font.height() + 4;
        x -= 9;
        g.set_color(6178);
        g.draw_line(x, y, x + self.item_box_w - 23, y);
        y += 1;
        g.set_color(3564144);
        g.draw_line(x, y, x + self.item_box_w - 23, y);
        // 格阵原点 = 共享 fn（③-1 单源：与 pointer_hit 共调 item_cell_origin）
        let (col_x0, grid_y0) = self.item_cell_origin();
        let mut x = col_x0;
        let mut y = grid_y0;
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
                        // 【R-2 换算收敛】居中式宽高用**逻辑**尺寸（=w/d）。
                        g.draw_image(
                            img,
                            x + ((32 - img.logical_width()) >> 1),
                            y + ((32 - img.logical_height()) >> 1),
                            0,
                        );
                    }
                    if (self.player.item_uses[grid as usize] as i8) > 0 {
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
        // 翻页箭头=盒右-22（盒框同源 layout；【2026-10-26 更正】旧 SCREEN_W 同病）
        x = ((self.layout.screen_w + self.item_box_w) >> 1) - 22;
        y -= (self.item_box_h - 32 - font.height() - 4) >> 1;
        if self.item_scroll_top > 0 {
            draw_image_clipped(g, &self.images.ui[15], x, y - 15, 0, 0, 7, 9);
        }
        if self.item_scroll_bottom < self.item_rows_total {
            draw_image_clipped(g, &self.images.ui[15], x, y + 5, 7, 0, 7, 9);
        }
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
    }

    /// dispatchModeEntry case 7（a.java:4342-4364，case7/8 共享）：软键 (1,3) + loadSlotTable
    /// （每次进入重读 RMS 槽表）+ 槽表布局（6 行/宽 148/高 196）。
    /// 【2026-10-18 BUG-021】入口只复位布局/光标（a.java:4354-4366），**不动数据
    /// 数组**（Java 数组跨进入复用）；读失败（缺店/空/截断）留半解析旧值 +
    /// parse_dirty（markSlotEntry(-1) 落盘触发器，壳消费回写 MOT_IF）。
    fn enter_menu7(&mut self) {
        self.slot.reset_entry_layout();
        self.slot.load_slot_table(self.preset_mot_if.as_deref());
        self.softkeys = (1, 3);
    }

    /// run case 7（a.java:3829-3878，case7/8 共享）：-7 回 menuReturnMode；
    /// -6/-5 在 mode7 = 存档（m_119 + saveFloorState + “保存成功！”浮层）；
    /// -2/-1 槽光标滚动。
    fn tick_menu7(&mut self) {
        match self.key_value {
            -7 => self.transition_to(self.menu_return_mode),
            -6 | -5 => {
                if self.game_mode == GameMode::SaveSlot {
                    // 【BUG-045④】存档清栈（存档后不可撤回存档前的操作）
                    self.undo.clear(crate::undo_journal::ClearReason::Save);
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
                    // a.java:9013-9014：f_byte_arr_25/26 = equippedWeaponType/ArmorType
                    //（byte 字段；值域 equipTierTypes ⊂ i8）——装备框图标分支的数据源
                    self.slot.weapon[cursor] = self.equipped_weapon as i8;
                    self.slot.armor[cursor] = self.equipped_armor as i8;
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
        o.push(self.equipped_weapon as u8); // equippedWeaponType（a.java:9192 写真实值）
        o.push(self.equipped_armor as u8); // equippedArmorType（a.java:9193）
        // writeShort(playerCellX/Y)（a.java:9197-9198）：**2 字节**——旧写法
        // `(px>>5).to_be_bytes()` 是 i32 四字节，令其后全部字段错位 4 字节
        o.extend_from_slice(&(self.player_cell_x as i16).to_be_bytes());
        o.extend_from_slice(&(self.player_cell_y as i16).to_be_bytes());
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
        // writeShort(alchemyUpgradeCount)（a.java:9206；旧写法恒 0）
        o.extend_from_slice(&(self.alchemy_upgrade_count as i16).to_be_bytes());
        // writeShort(inventoryEntryCount，原 inventoryEntryCount) + f_byte_arr_14/15 对（a.java:9207-9212）= mode12 日志
        //（读侧 a.java:9278-9283 → journal；旧写法取恒空的 special_item_types）
        o.extend_from_slice(&(self.journal.len() as i16).to_be_bytes());
        for &(kind, idx) in &self.journal {
            o.push(kind as u8);
            o.push(idx as u8);
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

    /// paintSlotTable(true)（a.java:2679-2682 + 9046-9215）：mode 7 槽表（存档标题变体
    /// titledBox segs[2]）+ 有效槽详情面板（层数/装备/金币/三围/三钥）。
    fn paint_slot_list(&mut self, g: &mut SoftGraphics<'_>) {
        self.paint_parallax(g, false);
        let ui = &self.images.ui;
        // 标题变体 2（存档）：titleBarSegments[2] = {4,5}；列表段/装备框段/三围段与
        // boot 读档路径共用 SlotSelect 方法（原为双拷贝）
        let (lx, ly) = self.slot.paint_list(g, ui, &[4, 5], &self.layout);
        // 详情面板（a.java:9092-9147，slotValidFlags[cursor] 才画）
        let c = self.slot.cursor as usize;
        if self.slot.valid[c] {
            let vx = lx + 6;
            let vy = ly + 8;
            if self.slot.floor_num[c] < 51 {
                paint_number(g, &ui[18], self.slot.floor_num[c] as i32, vx + 48, vy);
                g.draw_image(&ui[8], vx + 50, vy + 8, 0);
            }
            // 装备框段（a.java:9104-9143）：f_Image_arr2_00[6][i] 经 m_122 映射表
            // 反查 entityTypeImage（同一位图，见 SPTARM_ENTITY_TYPES）
            let entity = &self.images.entity;
            self.slot.paint_equipment_frames(g, c, vy, &self.layout, |i| {
                crate::menu_family::SPTARM_ENTITY_TYPES
                    .get(i)
                    .and_then(|&t| entity.get(t))
                    .and_then(|o| o.as_ref())
            });
            let (mut vx, vy) = self.slot.paint_detail_stats(g, ui, c, vy, &self.layout);
            self.paint_key_slot(g, 0, self.slot.keys_yellow[c] as i32, vx, vy);
            vx += 42;
            self.paint_key_slot(g, 1, self.slot.keys_blue[c] as i32, vx, vy);
            vx += 42;
            self.paint_key_slot(g, 2, self.slot.keys_red[c] as i32, vx, vy);
        }
        m_034_softkeys(g, &ui[10], &ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
    }

    /// mode 15 帮助全文（dispatchModeEntry case 15 a.java:4407-4411，showOverlayMessage
    /// kind 0 逐字原文：99FFCC 色段 + 换行转义）。
    pub const HELP_TEXT: &str = "游戏描述：\n\\c99FFCC有人的地方就有江湖，有神仙的地方何尝不是江湖；百战百胜的本事，换不回女人的真心，兄弟的真义；齐天大圣又如何，没有真情实义，做神仙跟做咸鱼有什么区别？\n\n操作方式：按左软键调出物品栏，左右选择一件道具，按确定键使用。\n游戏操作：\n上方向键/2：向上行走\n下方向键/8：向下行走\n左方向键/4：向左行走\n右方向键/6：向右行走\n确定键/5:探索地图\n左软键：打开道具列表\n右软键：打开游戏中菜单\n\n代理发行：广州易诚计算机科技有限公司\n发行商网站：www.9266.net\n客服电话：4006509913\n客服信箱：kefu@9266.net";

    /// dispatchModeEntry case 15（a.java:4402-4413）：帮助全文浮层（kind 0）+ 软键 (0,3)。
    /// loadPackedImages(8)/loadPackedImages(1) = 懒加载资源容器（Rust 构造期全量装载，等价 no-op）。
    /// 迭代 26：改走 showOverlayMessage 完整口（原内联排版漏 overlayActive=true，
    /// 浮层整体不画——preset_help 嵌套假绿掩盖至今）。
    /// 【计划 3 T7 · spec §8】帮助文本覆盖（壳按模式注入触屏/摇杆操作说明；None=原文）。
    pub fn set_help_override(&mut self, text: Option<String>) {
        self.help_override = text;
    }

    fn enter_menu15(&mut self) {
        let owned = self.help_override.clone().unwrap_or_else(|| Self::HELP_TEXT.to_string());
        self.show_overlay_message(0, &owned, 0, 0);
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
                    self.transition_to(self.menu_return_mode);
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
        let sx = crate::title::menu_strip_x(self.layout.screen_w);
        crate::title::paint_icon_strip_at(g, &self.images.menu_strip, sx, 10, 51, 68, &mut f);
        self.strip_frame = f;
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
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
        self.encyc_panel_w = ENCYC_PANEL_W;
        self.encyc_count = 0;
        'outer: for e in 0..self.entities.count {
            if self.entities.items[e].removed {
                continue;
            }
            let t = self.entities.items[e].type_;
            if self.entity_category(t) != 8 || t < 41 {
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
        let row_h = font.height() + 4 + 36;
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

    /// dispatchModeEntry case 5：m_091 + 软键 (0,3)。
    fn enter_menu5(&mut self) {
        self.build_encyclopedia();
        self.softkeys = (0, 3);
    }

    /// run case 5（a.java:3737-3755）：-7/-6 回 mode 3；-2 下一页、-1 上一页。
    fn tick_menu5(&mut self) {
        match self.key_value {
            -7 | -6 => self.transition_to(GameMode::Map),
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
        let bx = (self.layout.screen_w - self.encyc_panel_w) >> 1;
        let by = (self.layout.screen_h - self.encyc_panel_h) >> 1;
        if self.encyc_scroll_bottom - self.encyc_scroll_top > 0 {
            crate::menu_family::paint_box_frame(g, &self.images.ui[0], bx, by, self.encyc_panel_w, self.encyc_panel_h);
            // 【BUG-031③】a.java:2511/2595 翻页箭头 x 112 = 屏心 120 − 8（面板居中 ⇒ 随屏心）
            let arrow_x = self.layout.center_x - 8;
            if self.encyc_scroll_top > 0 {
                draw_image_clipped(g, &self.images.ui[19], arrow_x, by - 17 - (self.frame_counter & 1) as i32, 16, 0, 15, 17);
            }
            let mut y = by + 16;
            for row in self.encyc_scroll_top..self.encyc_scroll_bottom {
                let c = row as usize;
                let t = self.encyc_types[c] as usize;
                let pred = self.encyc_damage[c];
                let mut x = bx + 16;
                g.set_color(13097429);
                let name: Vec<u16> = self.object_type_names[t].encode_utf16().collect();
                g.draw_string(&name, x, y + 2, 0);
                // 【BUG-031③】a.java:2524 `(240 + 面板宽 >> 1) − 75`：屏宽 240 随 layout（伤害列贴面板右缘）
                let dmg_x = ((self.layout.screen_w + self.encyc_panel_w) >> 1) - 75;
                let name_dy = (font.height() - 19) >> 1;
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
                        // 【R-2 换算收敛】居中式宽高用**逻辑**尺寸（=w/d）。
                        g.draw_image(
                            img,
                            x + ((32 - img.logical_width()) >> 1),
                            y + ((32 - img.logical_height()) >> 1),
                            0,
                        );
                    }
                    g.set_clip(x, y, self.layout.screen_w, self.layout.screen_h); // a.java:2553 `240, 320`
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
                draw_image_clipped(g, &self.images.ui[19], arrow_x, y + (self.frame_counter & 1) as i32, 16, 17, 15, 17);
            }
        }
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
    }

    /// alchemyPriceFor（a.java:7256-7262）：第 k 次升级价 = 20 + Σ20i (i=1..k-2)
    fn alchemy_price_for(k: i32) -> i32 {
        let mut price = 20;
        for i in 1..k {
            // Java: for (script_cursor0 = 1; script_cursor0 < var0; script_cursor0++)（a.java:7259）——上界含 k-1
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
            -7 => self.transition_to(GameMode::Map),
            -6 | -5 => {
                if self.player.gold >= self.alchemy_price {
                    // 【BUG-045】未记账的消费：撤销按战前绝对值还原血金会白送回花掉的钱（复核 T3-R4）⇒ 清栈
                    self.undo.clear(crate::undo_journal::ClearReason::Irreversible);
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
        let tile_off = self.layout.view_top_y + self.advance_open_group_paint();
        self.view.paint_tiles(g, tileset, tile_off);
        self.paint_entities(g, self.view.cam_x, self.view.cam_y + self.layout.view_top_y);
        self.paint_hud(g);
        self.paint_status_bar(g);
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);

        let font = crate::paint::paint_font();
        let bw = 115i32; // f_int_75（a.java:810）
        let bh = font.height() + 90; // f_int_74（a.java:809）
        let bx = (self.layout.screen_w - bw - 22) >> 1;
        let by = (self.layout.screen_h - bh) >> 1;
        crate::menu_family::paint_titled_box(g, &self.images.ui[0], &self.images.ui[13], &[0, 1, 2], bx, by, bw + 22, bh);
        let wide = font.char_width('回' as u16) << 1;
        let mut x = (self.layout.screen_w - (wide + 16 + 48)) >> 1;
        let mut y = by + 21;
        g.set_color(7575203);
        g.draw_string(&"花费".encode_utf16().collect::<Vec<u16>>(), x, y, 0);
        x += wide + 4;
        g.draw_image(&self.images.ui[1], x, y + ((font.height() - 10) >> 1), 0);
        x += 60;
        paint_number(g, &self.images.ui[2], self.alchemy_price, x, y + 3 + ((font.height() - 10) >> 1));
        y += font.height() + 5;
        x = (self.layout.screen_w - bw) >> 1;
        g.set_color(549016);
        g.fill_rect(x, y + (self.alchemy_cursor << 4), bw, 16);
        g.draw_image(&self.images.ui[14], x + 10, y + 2 + (self.alchemy_cursor << 4), 0);
        g.draw_image_transformed(&self.images.ui[14], x + bw - 30, y + 2 + (self.alchemy_cursor << 4), 0, 8192);
        // 【BUG-031②】a.java:2715-2722 图标 x 97 / 加值 x 142 = 高亮行 x 62（`240 − 115 >> 1`）+ 35 / + 80
        // ——行内相对位，随居中的高亮行（classic 回代原值）。
        draw_image_clipped(g, &self.images.ui[7], x + 35, y + 4, 0, 2, 10, 10);
        paint_number(g, &self.images.ui[2], self.alchemy_gain_hp, x + 80, y + 6);
        y += 16;
        draw_image_clipped(g, &self.images.ui[7], x + 35, y + 4, 11, 0, 8, 13);
        paint_number(g, &self.images.ui[2], self.alchemy_gain_atk, x + 80, y + 6);
        y += 16;
        draw_image_clipped(g, &self.images.ui[7], x + 35, y + 4, 20, 2, 10, 11);
        paint_number(g, &self.images.ui[2], self.alchemy_gain_def, x + 80, y + 6);
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

    /// pushOfferQueue（a.java:7472-7477）：报价入队。
    fn offer_queue_push(&mut self, src: i32, idx: i32) {
        let cap = (Self::OFFER_TEXTS_A.len() + Self::SIGN_TEXTS_B.len()) as i32;
        if (self.offer_queue_src.len() as i32) < cap {
            self.offer_queue_src.push(src as u8);
            self.offer_queue_idx.push(idx as u8);
        }
    }

    /// spendGold：够则扣减返回 true（a.java:7702 邻接族，equipTierIndex）。
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
        if buy {
            // 【BUG-045】商人交易（扣金 / 送物）不记账 ⇒ 清栈（复核 T3-R4）
            self.undo.clear(crate::undo_journal::ClearReason::Irreversible);
        }
        let param = self.entities.items[self.merchant_entity as usize].param as i32;
        let flags = (param >> 8) as i8;
        let idx = param as i8;
        let src = if self.entities.items[self.merchant_entity as usize].type_ == 77 {
            0
        } else {
            1
        };
        let mut purchase_ok = true; // open_n：购买/执行是否成功
        let mut follow_up = false; // cur_x：后续挂起（报价队列/失败提示浮层）
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
                // a.java:7422-7436：**收尾块在 flags&2 门内**——flags&2=0
                // 时整块跳过（无效果、浮层关、follow_up=false）。迭代 76：
                // 此块曾误置门外致 flags=1 受领后 follow_up=true 浮层永开
                // （floor46 T451 7868px 实证）。
                if purchase_ok {
                    if flags & 4 == 0 {
                        self.entities.items[self.merchant_entity as usize].param ^= 512;
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
                            // a.java:7450：flags&1==0 → markEntityRemoved（cat16
                            // 不重绘小地图——buildMinimap 仅在 markEntityRemoved
                            // 的 cat1 分支内，a.java:6367）
                            entity::remove(&mut self.entities, self.merchant_entity as usize);
                        }
                    }
                } else {
                    // 购买失败（「没有足够的金钱」提示浮层已开）→ 浮层保持
                    follow_up = true;
                }
            } else if flags & 1 != 0 {
                // a.java:7456-7460：**购买路径 flags&2==0 且 flags&1!=0 臂**——
                // 一次性商人（flags&1）任意处置（买/拒）皆消耗：^256 + 非
                // flags&4 防护则 markEntityRemoved（cat16 → 27×29 row6 白烟）。
                // floor46 T450 实证：-5 购买后 Java 商人消失（白云残置），
                // Rust 曾缺此臂致老者滞留（413px 单点根因，迭代 77）。
                self.entities.items[self.merchant_entity as usize].param ^= 256;
                if flags & 4 == 0 {
                    entity::remove(&mut self.entities, self.merchant_entity as usize);
                }
            }
        } else if flags & 2 == 0 && flags & 1 != 0 {
            // 拒绝路径（a.java:7462-7466）：^256（非 ^512）+ 非一次性则移除
            self.entities.items[self.merchant_entity as usize].param ^= 256;
            if flags & 4 == 0 {
                entity::remove(&mut self.entities, self.merchant_entity as usize);
            }
        }
        follow_up
    }

    /// 【STAGE-B-008 计划 3 T3.2 · spec §8】整体替换壳层附加行（如操控方式切换 ⇒ 自动战斗行增减）。
    /// 设置页开着 ⇒ 同步重建 options（盒高随之变），高亮钳在新行数内（spec：保持在「操控方式」行）。
    pub fn replace_shell_rows(&mut self, rows: Vec<crate::menu_family::ShellOption>) {
        self.shell_options = rows;
        if self.game_mode == GameMode::About {
            if let Some(old) = self.options.take() {
                let h = old.highlight;
                let mut list = crate::menu_family::OptionList::with_shell_rows(
                    self.bgm.sound_enabled,
                    self.minimap_enabled,
                    &self.shell_options,
                );
                list.highlight = h.clamp(0, list.len() as i32 - 1);
                self.options = Some(list);
            }
        }
    }

    fn enter_menu16(&mut self) {
        // 【STAGE-B-006】原两行之后接壳层附加行（空 = 原版两行）
        self.options = Some(crate::menu_family::OptionList::with_shell_rows(
            self.bgm.sound_enabled,
            self.minimap_enabled,
            &self.shell_options,
        ));
        self.softkeys = (0, 3);
    }

    /// run case 16（a.java:4143-4188）：-7/-6 回 menuReturnMode；-5/-4/-3 翻转
    /// （label0 音乐→音源态翻转 + saveGlobalProgress；label1 小地图→null/rebuild）；
    /// -2/-1 光标环绕。
    fn tick_menu16(&mut self) {
        let Some(opt) = &mut self.options else { return };
        match self.key_value {
            -7 | -6 => self.transition_to(self.menu_return_mode),
            -5 | -4 | -3 => {
                let cur = opt.highlight as usize;
                if let Some((id, v)) = opt.toggle_shell_row(cur) {
                    // 【STAGE-B-006】壳层附加行：只翻该行并回写 shell_options——不碰声音 / 小地图 / SKY_WAR。
                    // 早退：本 match 是函数末语句（原臂文本保持原样）；若日后在 match 后加逻辑，须改回 if/else。
                    crate::menu_family::set_shell_option(&mut self.shell_options, id, v);
                    return;
                }
                opt.checked[cur] = !opt.checked[cur];
                match opt.labels[cur] {
                    0 => {
                        // 开 ⇒ volumeLevel=60 + playBgm(f_bool_28?3:2)，关 ⇒ closeAudioPlayer
                        //（a.java:4157-4167；EV-toggle-exit T521/T561）。
                        self.bgm.toggle_sound();
                        // saveGlobalProgress（RMS 侧，无像素效应；game-core 无 I/O）
                    }
                    1 => {
                        if !opt.checked[1] {
                            self.minimap = None;
                        } else if self.menu_return_mode == GameMode::InGameMenu {
                            self.build_minimap();
                        }
                    }
                    _ => {}
                }
            }
            -2 => {
                opt.highlight += 1;
                if opt.highlight > opt.len() as i32 - 1 {
                    opt.highlight = 0;
                }
            }
            -1 => {
                opt.highlight -= 1;
                if opt.highlight < 0 {
                    opt.highlight = opt.len() as i32 - 1;
                }
            }
            _ => {}
        }
    }

    /// cameraAtTarget（a.java:8205-8201）。
    fn camera_at_target(&self) -> bool {
        self.ease_cam_x == self.ease_target_x && self.ease_cam_y == self.ease_target_y
    }

    /// keyPressed（a.java:4663-4676）：keyValue = keyHeldCode = code（覆盖式：同拍
    /// 多次按下取末个，R-2）；anyKeyPlayed 时按 gameMode 恢复播放（音频半）。
    pub fn press_key(&mut self, code: i32) {
        // 【STAGE-B-008】场上按键取消点地后续撞击（BUG-040"手动优先"同口径）。
        // 【spec §5.1 · 计划 2 T5 复核 R1/R2】浮层开着时按键属浮层（关框 / 翻页），不取消撞击——与
        // interrupt_route_for_input 同判据：关框后继续走向目标并照常撞击。
        // 【计划 2 T7】点地排队同此判据：排空期间的场上按键作废排队；浮层开着时的关框键不作废（关框后照常执行）。
        // 【计划 2 T7 复核 R1】同时清后续就绪位：末步在途窗口内按下又在下一拍前松开，tick_tap_state 看不到 keyHeldCode，
        // 就绪位留着会让本步与之后的手动步仍算"点地寻路中"。
        if !self.overlay_active {
            self.tap_bump = None;
            self.tap_pending = None;
            self.tap_followup_ready = false;
        }
        // 【STAGE-B-008 · spec §5.2】任何按键清怪物选中（浮层开着时选中本就由每拍校验清掉）
        self.tap_selected = None;
        // 【T2 打断终版】寻路（WalkTo/探步段）中任意输入皆打断——全通道（触屏/
        // 方向键/软键）即刻清 walkPathBuffer 停步（手动优先）。经典脚本行走
        // （MOV/dp2）不受影响（route 段仅 WalkTo 开放）——保真红线 key 链不动。
        // 【STAGE-B-008 复核 T4-R4】浮层开着时按键属浮层（关框 / 翻页），不打断——见 interrupt_route_for_input。
        if self.route.is_open() {
            self.interrupt_route_for_input();
        }
        self.key_value = code;
        self.key_held = code;
        self.bgm.on_key_pressed(self.game_mode);
    }

    /// keyReleased（a.java:4678-4674）：只清 keyHeldCode。
    pub fn release_key(&mut self) {
        self.key_held = 0;
    }

    /// BGM 宿主契约：应当在放的轨道（`audio::TRACKS` 下标），None = 停止/无声。
    /// 语义 = Java `bgm != null ? track : 无`（docs/spec/audio.md）。
    pub fn bgm_request(&self) -> Option<usize> {
        self.bgm.request()
    }

    /// 当前播放器序号：同轨 serial 变化 = Java 关闭重建（从头播放）。
    pub fn bgm_serial(&self) -> u32 {
        self.bgm.serial()
    }

    /// 当前播放器 VolumeControl 级别（0..=100）。
    pub fn bgm_volume(&self) -> Option<i32> {
        self.bgm.volume()
    }

    /// run case 3（a.java:3551-3679）：keyValue 开关（-7/-6 暂停/道具——
    /// 本窗口无）→ walkPhase 状态机 → advanceEntityFrames。
    /// `frame_counter` 为本拍值（TICK n → n-1）。
    pub fn tick(&mut self, frame_counter: i64, width_table: &[i32]) {
        self.frame_counter = frame_counter;
        self.tick_tap_state(); // 【STAGE-B-008】classic 下点地状态恒空 ⇒ 零操作
        self.settle_walkto_arrows(); // 【BUG-043】经典面暂存恒 None ⇒ 零操作
        // overlay 显示期（run 的 label510 分支 a.java:3252）：kind0 且
        // gameMode∈{15,17} 走 -7 专用支路（仅 -7 关层+回 menuReturnMode，
        // 其余键吞掉）；其余情况 handleOverlayKey（a.java:4923-4931：任意
        // 非翻页键关层）。**walkPhase 状态机与 advanceEntityFrames 冻结**。
        if self.overlay_active {
            if self.overlay_kind == 4 {
                // 商人/告示牌（overlay 分派 case4，deobf 3311-3320）：-6/-5=
                // applyMerchantOffer(true)，-7=(false)，其余 handleOverlayKey
                // （±**1** 翻页——【2026-10-16 修】原误用 ±lines_per_page）
                match self.key_value {
                    -7 => self.overlay_active = self.apply_merchant_offer(false),
                    -6 | -5 => self.overlay_active = self.apply_merchant_offer(true),
                    _ => self.handle_overlay_key(),
                }
            } else if self.overlay_kind == 5 {
                // mode12 日志浏览（overlay 分派 case5，a.java:3325-3344）：
                // -3/-4 = browseOfferQueue 翻页、-2/-1 = 长文滚动、-5/-6/-7 关回 mode10。
                // （迭代 59 纠偏：本臂曾被误挂商人报价 seek——商人 quote 实为
                // kind4（a.java:7442），merchant 场景亦无 -3/-4 按键。）
                match self.key_value {
                    -7 | -6 | -5 => {
                        self.overlay_active = false;
                        self.transition_to(GameMode::ItemPanel);
                    }
                    -4 => self.open_record_page(self.record_page + 1),
                    -3 => self.open_record_page(self.record_page - 1),
                    -2 => self.flip_overlay_page(self.overlay_page_top + 1),
                    -1 => self.flip_overlay_page(self.overlay_page_top - 1),
                    _ => {}
                }
            } else if self.overlay_kind == 0 && (self.game_mode == GameMode::Help || self.game_mode == GameMode::IconView) {
                match self.key_value {
                    -7 => {
                        self.overlay_active = false;
                        self.transition_to(self.menu_return_mode);
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
                        self.transition_to(GameMode::ItemPanel);
                        self.overlay_active = false;
                    }
                    -6 => {
                        self.overlay_active = false;
                        self.use_item_stack(self.item_cursor as usize);
                    }
                    _ => self.handle_overlay_key(),
                }
            } else if self.overlay_kind == 1 {
                // case1 图标支路（javap run() 336-441；deobf ~3245-3290）：
                // 无图标 → handleOverlayKey；right 为 2/3 → key0 吞、-2/-1
                // handleOverlayKey、其余关；right 非 2/3 → 全吞键（441 →
                // goto 654）。【2026-10-16 补】原缺臂（并入兜底）。
                if self.overlay_right_icon == 0 && self.overlay_left_icon == 0 {
                    self.handle_overlay_key();
                } else if self.overlay_right_icon == 2 || self.overlay_right_icon == 3 {
                    match self.key_value {
                        0 => {}
                        -2 | -1 => self.handle_overlay_key(),
                        _ => self.overlay_active = false,
                    }
                }
            } else if self.overlay_kind == 2 {
                // case2（deobf 3290-3291 `case 2: default: break;`）：全吞键。
                // 调用面 0 命中（Java 亦无生产者）——死码臂为忠实而保
                // （MVS 先例），不得并入兜底关层。
            } else {
                // 兜底 = handleOverlayKey（含 kind0 非 15/17 的 else 路，
                // deobf 3288-3289 `case 2: default: break` 之外的收口）
                self.handle_overlay_key();
            }
            // label510 块尾双清（a.java:3345）：keyValue 与 **keyHeldCode 同清**——
            // held 残留会让关层后首拍误触发 tryStep（floor2-tour T1238 实证）
            self.key_value = 0;
            self.key_held = 0;
            return;
        }
        if self.game_mode == GameMode::Help {
            // run case 15（fallthrough no-op）；浮层支路在 run 头消费键
            self.tick_menu15();
            if self.game_mode == GameMode::Help {
                self.key_value = 0;
            }
            return;
        }
        if self.game_mode == GameMode::EnemyInfo {
            // run case 5（a.java:3737-3755）
            self.tick_menu5();
            self.key_value = 0;
            return;
        }
        if self.game_mode == GameMode::About {
            // run case 16（a.java:4143-4188）
            self.tick_menu16();
            self.key_value = 0;
            return;
        }
        if self.game_mode == GameMode::SaveSlot {
            // run case 7（a.java:3829-3878）
            self.tick_menu7();
            self.key_value = 0;
            return;
        }
        if self.game_mode == GameMode::Alchemy {
            // run case 9（a.java:3885-3940）
            self.tick_menu9();
            self.key_value = 0;
            return;
        }
        if self.game_mode == GameMode::InGameMenu {
            // run case 4（a.java:3690-3735），尾清 keyValue（a.java:4270）
            self.tick_menu4();
            self.key_value = 0;
            return;
        }
        if self.game_mode == GameMode::Shop {
            // run case 19（a.java:4200-4226）
            self.tick_menu19();
            self.key_value = 0;
            return;
        }
        if self.game_mode == GameMode::ItemPanel {
            // run case 10（a.java:3931-4009）：键位导航，尾清 keyValue（a.java:4270）
            self.tick_item_panel();
            self.key_value = 0;
            return;
        }
        if self.game_mode == GameMode::Dialog {
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
        if self.game_mode == GameMode::Ending {
            // run case 20（a.java:4223-4228）：f_bool_04 后任意键回主菜单
            // （gameMode=1 + dispatchModeEntry——场景→BootMachine 反向移交为已知缝，
            // 对拍窗口不触键）。f_bool_04 未到 → 任意键仍清零。
            if self.ending_done && self.key_value != 0 {
                self.transition_to(GameMode::MainMenu);
                // dispatchModeEntry case 1 音频半：f_bool_28=false + playBgm(2)：game→menu 重建
                //（a.java:4295/4311；EV-ending-title T1301）。非音频半（标题菜单
                // 重建/反向移交 BootMachine）仍为已知缝。
                self.bgm.enter_title();
            }
            self.key_value = 0;
            return;
        }
        // run case 3 头（a.java:3552-3562）：keyValue 开关。-7 → gameMode=4
        // （存档菜单，mode 4 批次待办）；-6 → gameMode=10 + dispatchModeEntry。
        match self.key_value {
            -7 => {
                // a.java:3559-3561：inGameMenuReturnMode=gameMode → gameMode=4 + dispatchModeEntry
                self.in_game_menu_return_mode = self.game_mode;
                self.transition_to(GameMode::InGameMenu);
                self.enter_menu4();
            }
            -6 => {
                self.transition_to(GameMode::ItemPanel);
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
                        if self.view.player_py + self.view.cam_y + 16
                            > self.view.view_h - crate::layout::EDGE_MARGIN
                        {
                            let cy = self.view.cam_y - 8;
                            self.view.set_camera(self.view.cam_x, cy);
                        }
                    }
                    1 => {
                        self.view.player_py -= 8;
                        if self.view.player_py + self.view.cam_y + 16 < crate::layout::EDGE_MARGIN {
                            let cy = self.view.cam_y + 8;
                            self.view.set_camera(self.view.cam_x, cy);
                        }
                    }
                    2 => {
                        self.view.player_px += 8;
                        if self.view.player_px + self.view.cam_x + 16
                            > self.view.view_w - crate::layout::EDGE_MARGIN
                        {
                            let cx = self.view.cam_x - 8;
                            self.view.set_camera(cx, self.view.cam_y);
                        }
                    }
                    3 => {
                        self.view.player_px -= 8;
                        if self.view.player_px + self.view.cam_x + 16 < crate::layout::EDGE_MARGIN {
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
                    // 【T2】WalkTo 段终位：步尽归格=实际终点（整段起终点入栈）+回 Map；
                    // 经典脚本行走 route 未开放=零改动（保真红线）
                    // 【STAGE-B-008 · 计划 2 T5 复核问题 B】Map 语境下开着的路段只可能是单步段：点地撞击（execute_bump 走进门格 /
                    // 开战）或 Map 语境战斗的胜利 restep（route.resume）——点地路线的步序只在 Dialog dp3 下装进 walkPathBuffer。
                    // 此时 walkPathBuffer / walk_step_count 装的可能是 finish_walkto 放回的教程箭头（BUG-043），不是路线步，
                    // 故不再以 `walk_step_count == 0` 为条件：否则箭头开着时段一直不收（目标标记不清、下次按键清掉箭头、
                    // 下次点地重设段起点丢掉这一格的位移笔）。
                    if self.route.is_open() {
                        self.route_arrived();
                        self.game_mode = GameMode::Map;
                    }
                }
            }
            2 => {
                // walkPhase 2（a.java:3610-3642）：easeAfterimages 残影蛇缓动 +
                // 未锁定时 keyHeld 相机平移 / keyValue -5 → 锁定 + lockCameraOn；
                // 已锁定时 stepCameraTowardTarget 回到玩家
                self.ease_afterimages();
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
                        self.lock_camera_on(0);
                    }
                } else {
                    self.step_camera_toward_target();
                }
            }
            5 => self.tick_battle(true),
            _ => {} // 3 道具菜单 / 4 楼层切换：本窗口无
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
    fn ease_afterimages(&mut self) {
        let (mut tx, mut ty) = if !self.view_locked {
            (
                ((self.view.view_w - 41) >> 1) - self.view.cam_x,
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
                let mut v3 = 4 - i as i32; // Java sort_i==0 的 +5 分支不可达（循环从 1 起）
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
    fn lock_camera_on(&mut self, target: i32) {
        self.ease_cam_x = ((self.view.view_w - 32) >> 1) - self.view.cam_x;
        self.ease_cam_y = ((self.view.view_h - 32) >> 1) - self.view.cam_y;
        if target != 0 && target != 87 {
            // 目标类型查实体（倒序首个）——战后视点本窗口只用 0（玩家）
            let mut found = -1;
            for e in (0..self.entities.count).rev() {
                if self.entities.items[e].type_ == target {
                    found = e as i32;
                    break;
                }
            }
            if found >= 0 {
                self.ease_target_x = self.entities.items[found as usize].px;
                self.ease_target_y = self.entities.items[found as usize].py;
                // a.java:8667-8668：型69 目标 X += 32（取景右偏，全调用方共用）
                if target == 69 {
                    self.ease_target_x += 32;
                }
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
            self.ease_cam_x += ((self.ease_target_x - self.ease_cam_x) >> 2) + 2;
            if self.ease_cam_x > self.ease_target_x {
                self.ease_cam_x = self.ease_target_x;
            }
        } else if self.ease_cam_x > self.ease_target_x {
            self.ease_cam_x += ((self.ease_target_x - self.ease_cam_x) >> 2) - 2;
            if self.ease_cam_x < self.ease_target_x {
                self.ease_cam_x = self.ease_target_x;
            }
        }
        if self.ease_cam_y < self.ease_target_y {
            self.ease_cam_y += ((self.ease_target_y - self.ease_cam_y) >> 2) + 2;
            if self.ease_cam_y > self.ease_target_y {
                self.ease_cam_y = self.ease_target_y;
            }
        } else if self.ease_cam_y > self.ease_target_y {
            self.ease_cam_y += ((self.ease_target_y - self.ease_cam_y) >> 2) - 2;
            if self.ease_cam_y < self.ease_target_y {
                self.ease_cam_y = self.ease_target_y;
            }
        }
        self.view.set_camera(
            ((self.view.view_w - 32) >> 1) - self.ease_cam_x,
            ((self.view.view_h - 32) >> 1) - self.ease_cam_y,
        );
    }

    /// handleFieldInput（a.java:5265-5327）：keyValue 特例（-5 战后视点/
    /// 49/55 楼梯换乘——本窗口无）+ keyHeld 移动（Nokia 映射：
    /// -1=上 -2=下 -3=左 -4=右）。
    fn handle_field_input(&mut self) {
        if self.game_mode == GameMode::Dialog {
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
                let t = self.entities.items[e].type_;
                if !self.entities.items[e].removed && self.entity_category(t) == 8 {
                    self.battle_digit_by_type[t as usize] = self.predict_hp_loss(t, true);
                }
            }
            self.walk_phase = 2;
            self.show_battle_digits = self.player.find_item(13) >= 0;
            // Java case -5/53 尾 break（a.java:5294）⇒ 出 keyValue switch 后
            // **续走** keyHeldCode switch（a.java:5308 起）——不 return，
            // 同拍方向键照常 facing+tryStep。【2026-10-16 修复】原臂末
            // return 吞同拍步进（与型 15 break 语义 bug 同型，lead 裁定修）。
        }
        if self.key_value == 49 {
            // 49=上行（a.java:5295-5299 **分立 case**）：清零键值 → f_bool_13
            // 且存在型 7 门（param>>9==0）→ changeFloor(param&255, false, true)。
            // 【2026-10-16 修复】Rust 移植 bug（非原版 bug，不属 D4）：原实现
            // 先清零 key_value 再判 up ⇒ 恒 false、49 臂被并入 55；且臂末
            // return 吞掉 Java break 后续走的 keyHeldCode switch（5308 起）。
            self.key_value = 0;
            if self.stair_hint {
                if let Some(e) = self.find_floor_gate_entity(true) {
                    let target = self.entities.items[e].param as i32 & 255;
                    self.change_floor(target, false, true);
                }
            }
        } else if self.key_value == 55 {
            // 55=下行（a.java:5300-5305 **分立 case**）：清零键值 → f_bool_13
            // 且存在型 8 门（param>>9==0）→ changeFloor(param&255, true, true)。
            self.key_value = 0;
            if self.stair_hint {
                if let Some(e) = self.find_floor_gate_entity(false) {
                    let target = self.entities.items[e].param as i32 & 255;
                    self.change_floor(target, true, true);
                }
            }
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
            // a.java:5341-5342：interact 通过即 wp=1，**无 mode11 特例**（迭代 65：
            // 旧权宜复位会杀掉 dp3 脚本行走——floor10 MOV_0_6_5 T504 实证）。
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
    /// 地形终审（cur_y 楼梯/喷泉计数 >0 时豁免地形）。
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
                    let t = self.entities.items[e].type_;
                    if self.entities.items[e].state != 1 {
                        match self.entity_category(t) {
                            1 => match t {
                                1 | 2 | 3 => {
                                    // 门（a.java:5391-5411）：耗钥（consumeKeyForDoor
                                    // 内部 spawnPopup kind1 图标 a.java:7746-7752）→
                                    // markEntityRemoved + 重建小地图；无钥 → 阻挡 +
                                    // walkStepCount=0（提示浮层属交互批次）。
                                    // 【2026-10-16 裁定】popup 时机差异：Java 在
                                    // consumeKeyForDoor 内部先 popup 后减数，Rust 在
                                    // consumed 后外层发 popup——tick 内变更序不可观测
                                    // （oracle trace 无 tick 内序观测面；popup 只读玩家
                                    // 像素坐标不读 keyCount，B 级），登记良性差异不改。
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
                                        // 【BUG-043】原版此处清的是与箭头共用的 walkStepCount（方向键撞门 ⇒ 箭头即不画）；
                                        // 点地寻路期间箭头在暂存里，同步清空（经典面暂存恒 None）
                                        if let Some(a) = self.walkto_arrows.as_mut() {
                                            a.clear();
                                        }
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
                                4 => {
                                    // 封印门（a.java:5432-5437）：恒拦截 + 文案
                                    // + 停扫（var8=0）
                                    self.show_overlay_message(
                                        0,
                                        "障碍物：封印门\n需要消灭指定的怪物才能打开的门！",
                                        0,
                                        0,
                                    );
                                    allowed = false;
                                    s = -1;
                                }
                                81 => {
                                    // 封印门81（a.java:5458-5465）：**仅可见时**拦截
                                    // + 同文案 + 停扫；隐形期 continue 放行（floor10
                                    // 南北隐形封印门可穿，迭代 65 差分实证）
                                    if self.entities.items[e].visible {
                                        self.show_overlay_message(
                                            0,
                                            "障碍物：封印门\n需要消灭指定的怪物才能打开的门！",
                                            0,
                                            0,
                                        );
                                        allowed = false;
                                        s = -1;
                                    }
                                }
                                5 => {} // 门卫：cat1 交互臂无 case（a.java:5391-5530
                                        // switch 无 5）→ bump 放行；开启臂在
                                        // applyStepCellEffects case5（宝箱）。
                                        // **不可写 `_`**——会吞掉其后的 83/9/77 臂。
                                6 => {
                                    // 型 6 隐形实体 bump 显形（a.java:5410-5417）：
                                    // cur_y++ **恒执行**于显形判定之前（a.java:5411）；
                                    // 显形三元组一次性，写序固定 visible=true →
                                    // entityState=2 → entityAnimRow=9（a.java:5413/
                                    // 5414/5415 逐字）；continue 不改 allowed、
                                    // 不置 var8=0（续扫同格后继槽）。
                                    // 【V2-P2 消费替换】写点改走 transition_overrides
                                    // `bump_reveal` 型 6 规则（键序=写序，顺序契约）。
                                    stair_count += 1;
                                    if !self.entities.items[e].visible {
                                        crate::resource_cfg::global().apply_bump_reveal(
                                            &mut self.entities,
                                            e,
                                            &self.width_table,
                                            None,
                                        );
                                    }
                                }
                                7 | 8 => stair_count += 1,
                                83 => {
                                // 场景门（a.java:5508-5510）：tryRunScene 门控 →
                                // loadLevelScript(mode 11)。命中即 var8=0 停扫
                                // （a.java:5512）——同格后继实体（如 floor50 的
                                // 型72）不再处理，迭代 57：缺停扫曾同时触发
                                // 对话+战斗。**bump 侧第二参 = false**
                                // （a.java:5508 原文）——script3 类 `gate &=
                                // from_walk` 门控使 bump 不触发、到达才触发
                                // （floor10 T401 实证：Java -1 撞 (6,6) 先行走、
                                // applyStep 到达才开 script3 对话）。
                                if self.try_run_scene(self.entities.items[e].param as i32, false) {
                                    self.overlay_active = false;
                                    self.overlay_text = None;
                                    let idx = self.entities.items[e].param as i32;
                                    self.load_level_script(idx);
                                    s = -1;
                                    // a.java:5509 `cur_x = loadLevelScript(...)`：声明
                                    // boolean 但 **default 恒 return false**（a.java:8075）
                                    // ⇒ cur_x=false，tryStep 不置 wp=1。**仅在命中分支**：
                                    // seen 跳过时 cur_x 保持 true——击杀后重步
                                    // （tick_battle → try_step，T768）必须放行置 wp=1，
                                    // 驱动对话关闭后的 (4,6) 行走与 wp4 组开门链
                                    // （迭代 65 T904 实证）。
                                    allowed = false;
                                }
                            }
                            9 => {
                                // 炼丹台（a.java:5419-5429）：步进必阻挡；
                                // 朝上（facingDirection==1）触发 mode 9 + 价格/增量初始化
                                allowed = false;
                                if self.facing == 1 {
                                    self.transition_to(GameMode::Alchemy);
                                    self.alchemy_price =
                                        Self::alchemy_price_for(self.alchemy_upgrade_count + 1);
                                    self.alchemy_gain_hp = Self::scaled_by_floor_tier(100, self.floor);
                                    self.alchemy_gain_atk = Self::scaled_by_floor_tier(2, self.floor);
                                    self.alchemy_gain_def = Self::scaled_by_floor_tier(4, self.floor);
                                    // dispatchModeEntry()：mode 9 run 臂对无键进入无动作（a.java:3885 default）
                                }
                            }
                            10 => {
                                // 三昧真火（a.java:5436-5440）：提示 + 阻挡 + 停扫
                                self.show_overlay_message(
                                    0,
                                    "障碍物：三昧真火\n必须用芭蕉扇才能熄灭它. ",
                                    0,
                                    0,
                                );
                                allowed = false;
                                s = -1;
                            }
                            11 => {
                                // 墙（a.java:5441-5448）：23 层静默；其余提示。阻挡 + 停扫
                                if self.floor != 23 {
                                    self.show_overlay_message(
                                        0,
                                        "障碍物：墙\n只有金勺子、玄明石可以凿开。或者剧情打开！",
                                        0,
                                        0,
                                    );
                                }
                                allowed = false;
                                s = -1;
                            }
                            12 => {
                                // 撞击墙（a.java:5449-5456）：变形为型 11（墙）+ 精灵/动画
                                // 重派 + 置可见（state=2 行 9）；阻挡但**续扫**（无 var8=0）
                                // 【V2-P2 消费替换】字段写点改走 transition_overrides
                                // `bump_reveal` 型 12 规则（type_id→redispatch→覆写，
                                // 顺序契约钉死）。**Q-001 留痕**：重派尺寸需按变形后型的
                                // **运行时图高**（图高是运行时事实，规则形装不下）——
                                // img_h 查取留在消费侧代码。**Q-004 留痕**：block_step
                                // 承载 var6=false（阻挡），续扫 continue 语义留本代码。
                                let cfg = crate::resource_cfg::global();
                                let target = cfg
                                    .bump_reveal_target(12)
                                    .expect("bump_reveal 型 12 规则缺 type_id");
                                let img_h = self
                                    .images
                                    .entity
                                    .get(target as usize)
                                    .cloned()
                                    .flatten()
                                    .map(|im| im.height);
                                let out = cfg.apply_bump_reveal(
                                    &mut self.entities,
                                    e,
                                    &self.width_table,
                                    img_h,
                                );
                                if out.block_step {
                                    allowed = false;
                                }
                            }
                            _ => {} // 76/82：Java bump switch（a.java:5391-5530）无此
                                    // 二 case，default=continue 放行续扫——空臂等价
                                    // （2026-10-17 排查核销；原"后续批次"注记过时）
                            },
                            8 => {
                                // 怪物（a.java:5470-5490）：可胜 → 场上战斗；否则阻挡
                                let loss = self.predict_hp_loss(t, true);
                                allowed = false;
                                if loss >= 0 && loss < self.player.hp {
                                    // 【BUG-045】撤销快照须在改 state 之前取（复活 = 还原到开战前的整格实体）
                                    let snapshot = self.entities.items[e];
                                    self.walk_phase = 5;
                                    self.entities.items[e].state = 3;
                                    self.battle_target = e as i32;
                                    // battleEnemyHp = enemyBaseHp[type-41]（**基础表**非缩放表）
                                    self.battle_enemy_hp = self.enemy_base_hp[(t - 41) as usize];
                                    // 【T2】撞敌=事件挂起（段收于实际终点=敌格，T1 挂点）
                                    // ——仅 WalkTo/探步寻路段记账；经典面 route 未开放=零写入
                                    if self.route.is_open() {
                                        self.route_event_suspend();
                                        // 【BUG-045③】战斗进行中不可撤（T1 契约的生产接线）
                                        self.undo.begin_combat();
                                        self.battle_undo_snap = Some((
                                            e,
                                            snapshot.type_,
                                            self.battle_enemy_hp,
                                            crate::undo_journal::PlayerVitals {
                                                hp: self.player.hp,
                                                gold: self.player.gold,
                                                exp: 0, // 本作无经验字段（PlayerCombat 面）——T1 通用字段恒 0
                                            },
                                            snapshot,
                                            self.entities.count,
                                        ));
                                    }
                                    self.battle_active = true;
                                    self.battle_no_counter = true;
                                } else {
                                    self.walk_step_count = 0;
                                    // 【BUG-043】同上：箭头暂存随共用 walkStepCount 一并清空
                                    if let Some(a) = self.walkto_arrows.as_mut() {
                                        a.clear();
                                    }
                                    // 【BUG-046】a.java:5487-5490：拒战提示框（kind 0，无脚注图标）
                                    self.show_overlay_message(0, "你无法战胜它", 0, 0);
                                }
                            }
                            16 => {
                                // 告示牌/商人（a.java:5494-5517）：param 打包交互——
                                // 高字节=标志（&1=交互入队且两键都消耗实体、&2=报价（效果表）、&4=不消耗 / 买后不翻转（可重复）），
                                // > 【2026-10-30 更正 · STAGE-B-008】原注"&4=一次性"与 apply_merchant_offer（:4821 起，flags&4 防护不摘实体）相反。
                                // 低字节=报价序号（1 基）；type77=文本源 0，其余=源 1；
                                // 触发即阻挡（cur_x=false）+ kind4 报价浮层开。
                                self.merchant_entity = e as i32;
                                let var6p = self.entities.items[e].param as i32;
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

        /// traitFlagsByType 特性位（initEntityTables a.java:6266-6262：49/53/74=1、69=2）。
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
                    let t = self.entities.items[e].type_;
                    let idx = (t - 41) as usize;
                    let dmg = crate::combat::effective_attack_vs_type(&self.player, self.trait_flags(t))
                        - self.enemies.def[idx];
                    if dmg > 0 {
                        if (self.battle_tick & 3) == 0 {
                            self.battle_enemy_hp -= dmg;
                            let (ex, ey) = (self.entities.items[e].px, self.entities.items[e].py);
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
                                let (gx, gy) = (self.entities.items[e].px, self.entities.items[e].py);
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
                    // 胜利后类型特判（a.java:10110-10145）：类型在 battle_target
                    // 清空**前**读取（restep 可能开新战斗改写 target，Java 同序）。
                    // 69/70/71/72 = 层绑定剧情钩子；61 = 49 层封印门 puzzle。
                    let vt = self.entities.items[self.battle_target as usize].type_;
                    match vt {
                        61 => {
                            if self.floor == 49
                                && self.entity_at_cell(6, 2, 61) < 0
                                && self.entity_at_cell(5, 3, 61) < 0
                                && self.entity_at_cell(7, 3, 61) < 0
                                && self.entity_at_cell(6, 4, 61) < 0
                                && self.entity_at_cell(5, 2, 61) >= 0
                                && self.entity_at_cell(7, 2, 61) >= 0
                                && self.entity_at_cell(5, 4, 61) >= 0
                                && self.entity_at_cell(7, 4, 61) >= 0
                            {
                                entity::remove_group_at_cell_by_type(
                                    6,
                                    3,
                                    70,
                                    &mut self.entities,
                                    &mut self.grid,
                                );
                                let img_h = self.images.entity[71].as_ref().map(|i| i.height);
                                entity::spawn(
                                    &mut self.entities,
                                    &mut self.grid,
                                    71,
                                    192,
                                    96,
                                    0,
                                    &self.width_table,
                                    img_h,
                                );
                            }
                        }
                        69 => {
                            if self.floor == 35 {
                                self.spawn_boss_event(7);
                                self.load_level_script(36);
                            }
                        }
                        70 | 71 => {
                            if self.floor == 49 {
                                self.load_level_script(35);
                            }
                        }
                        72 => {
                            if self.floor == 50 {
                                self.load_level_script(65);
                            }
                        }
                        _ => {}
                    }
                    // 【T2】战斗终结才入栈（T1 挂点契约）+挂起解除+续走新段
                    // 【BUG-048】剧情接管（胜利特判里 load_level_script 已收段并清栈 ⇒ 路段与战斗挂起都已解除）：丢快照、不入栈、不续走——
                    // 补入战斗笔会让撤销在剧情后复活怪物，重开路段会截断剧情行走。
                    // 其余（路段仍挂起 = 正常；或战斗中经壳层按键链 interrupt_route 打断 = 路段挂起解除但战斗仍挂起、未清栈）：入栈（不可逆则清栈）
                    // 并续走——按键打断时步序已清空，续走只开一个零步段给胜利 restep 记位移笔（否则撤销战斗后玩家与复活的怪重叠，复核 T4-R1）。
                    if self.route.is_suspended() || self.undo.combat_in_progress() {
                        if let Some((slot, ety, hp_before, vitals_before, snapshot, count_at_start)) =
                            self.battle_undo_snap.take()
                        {
                            if self.entities.count != count_at_start {
                                // 【BUG-045】击杀掉落（spawn_boss_event）/ 61 机关生成实体等不可逆副作用：实体表在战斗期间增长（append-only）⇒
                                // 撤销只能复活怪、撤不掉生成物（复核 T3-R1/R2：循环复制掉落，16 次后槽表越界 panic）⇒ 不入战斗笔并清栈；
                                // 续走照旧（restep 的位移笔记在清栈之后）。
                                self.undo.clear(crate::undo_journal::ClearReason::Irreversible);
                            } else {
                                let items_before = self
                                    .player
                                    .item_types
                                    .iter()
                                    .zip(self.player.item_uses.iter())
                                    .take(self.player.item_stack_size as usize)
                                    .map(|(t, u)| (*t as i32, *u as i32))
                                    .collect();
                                self.route_combat_finished(crate::undo_journal::CombatStroke {
                                    enemy_slot: slot,
                                    enemy_type_id: ety,
                                    enemy_hp_before: hp_before,
                                    vitals_before,
                                    items_before,
                                    enemy_snapshot: snapshot,
                                });
                            }
                            let at = (self.player_cell_x, self.player_cell_y);
                            self.route.resume(at);
                        }
                    } else {
                        self.battle_undo_snap = None;
                    }
                    self.battle_target = -1;
                }
            }
        }

        /// fire_trap_climb_up（a.java:8628-8636）：火焰陷阱上爬——上格可走/无槽/非玩家格
        /// 时拉起旧 60（removed+脱槽）并在上格重生 DES 态新 60。
    /// Java 反混淆原名 `fireTrapClimbUp`（对照性锚——P0 命名批 2026-10-19 机械改名，语义不变）。
        fn fire_trap_climb_up(&mut self, x: i32, y: i32, idx: i32) {
            let ty = y - 1;
            let walkable = self.walkable_at(x, ty);
            let has_slot = if x >= 0 && ty >= 0 && x < self.view.wide && ty < self.view.high {
                let ct = self.grid.cell_type[ty as usize][x as usize] as usize;
                self.grid.capacity[ct] > 0
            } else {
                false
            };
            if walkable && !has_slot && (x != self.player_cell_x || ty != self.player_cell_y) {
                entity::detach_from_cell(&mut self.grid, &mut self.entities, idx as usize);
                let img_h = self
                    .images
                    .entity
                    .get(60)
                    .cloned()
                    .flatten()
                    .map(|im| im.height);
                let ni = entity::spawn(
                    &mut self.entities,
                    &mut self.grid,
                    60,
                    x << 5,
                    ty << 5,
                    0,
                    &self.width_table,
                    img_h,
                );
                entity::remove_des(&mut self.entities, ni);
            }
        }

        /// dispatchModeEntry case 3 尾（a.java:4315 applyStepCellEffects）：进图瞬间出生格
        /// 触发（type83 → try_run_scene/loadLevelScript，序章 script 40 入口）。
        /// 槽载入链在 load_floor_from_save 尾内联；新游戏冷启动链（宿主构造
        /// 场景后）经此入口。
        pub fn apply_entry_step_effects(&mut self) {
            self.apply_step_cell_effects();
        }
        /// applyStepCellEffects（a.java:5543-5729，floor1 可达子集）：玩家格坐标
        /// 刷新 + 格内实体效果（拾取 cat2/4；楼梯 7/8 → 换层批次）+
        /// walkPhase=0 + handleFieldInput（按住连走）。
        fn apply_step_cell_effects(&mut self) {
            // 【BUG-043】刚走完这一步的朝向（下文 handleFieldInput 在 Dialog 语境会弹出点地寻路的下一步并转向）
            let walked_facing = self.facing;
            self.player_cell_x = self.view.player_px >> 5;
            self.player_cell_y = self.view.player_py >> 5;
            self.player_bob_applied = false; // player_bob_applied 每步清零（a.java:5550），踩喷泉再置
            // 陷阱扣血臂（a.java:5549-5645）：61 夹击 → -HP/2；62 邻接 → -100；
            // 60 邻接 → -200 + sort_i 短路 handleFieldInput；60 上邻 → fire_trap_climb_up
            // 火焰上爬（拉旧格 + 上格重生 DES 态）。armor 40（火浣衣）免疫
            // 61/62 与 60 计数，但仍执行上爬。（floor49 陷阱层 T305 实证缺口）
            let (pcx, pcy) = (self.player_cell_x, self.player_cell_y);
            let armor40 = self.equipped_armor == 40;
            let mut step_triggered = false;
            if !armor40 {
                let hit_lr = pcx > 0
                    && self.entity_at_cell(pcx - 1, pcy, 61) >= 0
                    && pcx < self.view.wide - 1
                    && self.entity_at_cell(pcx + 1, pcy, 61) >= 0;
                let hit_ud = pcy > 0
                    && self.entity_at_cell(pcx, pcy - 1, 61) >= 0
                    && pcy < self.view.high - 1
                    && self.entity_at_cell(pcx, pcy + 1, 61) >= 0;
                if hit_lr || hit_ud {
                    self.apply_hp_delta(-(self.player.hp >> 1));
                }
                let mut n62 = 0;
                if self.entity_at_cell(pcx - 1, pcy, 62) >= 0 {
                    n62 += 1;
                }
                if pcx < self.view.high - 1 && self.entity_at_cell(pcx + 1, pcy, 62) >= 0 {
                    n62 += 1;
                }
                if pcy > 0 && self.entity_at_cell(pcx, pcy - 1, 62) >= 0 {
                    n62 += 1;
                }
                if pcy < self.view.high - 1 && self.entity_at_cell(pcx, pcy + 1, 62) >= 0 {
                    n62 += 1;
                }
                if n62 > 0 {
                    self.apply_hp_delta(-100);
                }
            }
            let mut n60 = 0;
            if !armor40 {
                if pcx > 0 && self.entity_at_cell(pcx - 1, pcy, 60) >= 0 {
                    n60 += 1;
                }
                if pcx < self.view.high - 1 && self.entity_at_cell(pcx + 1, pcy, 60) >= 0 {
                    n60 += 1;
                }
                if pcy > 0 {
                    let idx = self.entity_at_cell(pcx, pcy - 1, 60);
                    if idx >= 0 {
                        n60 += 1;
                        self.fire_trap_climb_up(pcx, pcy - 1, idx);
                    }
                }
                if pcy < self.view.high - 1 && self.entity_at_cell(pcx, pcy + 1, 60) >= 0 {
                    n60 += 1;
                }
            } else if pcy > 0 {
                let idx = self.entity_at_cell(pcx, pcy - 1, 60);
                if idx >= 0 {
                    self.fire_trap_climb_up(pcx, pcy - 1, idx);
                }
            }
            if n60 > 0 {
                self.apply_hp_delta(-200);
                step_triggered = true;
            }
            let ct = self.grid.cell_type[self.player_cell_y as usize][self.player_cell_x as usize] as usize;
            let cap = self.grid.capacity[ct] as usize;
            if cap > 0 {
                let mut s = cap as i32 - 1;
                while s >= 0 {
                let e = self.grid.slots[ct][s as usize] as usize - 1;
                let t = self.entities.items[e].type_;
                match self.entity_category(t) {
                    1 => match t {
                        6 => self.player_bob_applied = true, // player_bob_applied（喷泉浮沉）
                        83 => {
                            // 出生/踩格触发（a.java:5708-5709）：序章 script 40 入口
                            if self.try_run_scene(self.entities.items[e].param as i32, true) {
                                self.overlay_active = false;
                                self.overlay_text = None;
                                let idx = self.entities.items[e].param as i32;
                                self.load_level_script(idx);
                                step_triggered = true; // a.java:5712 sort_i=true
                            }
                        }
                        7 => {
                            // 上楼梯（a.java:5660-5659）：changeFloor(param, false, false)
                            let target = (self.entities.items[e].param & 0xff) as i32;
                            self.change_floor(target, false, false);
                            self.key_held = 0;
                        }
                        8 => {
                            // 下楼梯（a.java:5666-5665）：changeFloor(param, true, false)
                            let target = (self.entities.items[e].param & 0xff) as i32;
                            self.change_floor(target, true, false);
                            self.key_held = 0;
                        }
                        _ => {} // 5 宝箱/76 组门在下方 cat1 特例块（a.java:5641-5705）完整
                        // 实现；83 在上方臂——本兜底为其余型（2026-10-17 排查更正
                        // "后续批次"过时注记）
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
                let t = self.entities.items[e].type_;
                if t == 5 {
                    // 宝箱（a.java:5641-5640）：removeSlotAt + tryOpenChest(param)
                    let chest_param = self.entities.items[e].param;
                    self.remove_slot_at(self.player_cell_x, self.player_cell_y, s2);
                    if self.try_open_chest(chest_param) {
                        self.walk_phase = 4;
                    }
                } else if t == 76 {
                    // 76 组门（a.java:5655-5705）：param&0xff + 1 为组号；摘槽 + 全图
                    // 收集同组型 4（开锁动画）+ 清组内 76 槽
                    let group = ((self.entities.items[e].param & 0xff) as i32) + 1;
                    self.remove_slot_at(self.player_cell_x, self.player_cell_y, s2);
                    self.open_group.clear();
                    let mut slots_to_clear: Vec<(usize, u8)> = Vec::new();
                    for k in 0..self.entities.count {
                        if self.entities.items[k].removed {
                            continue;
                        }
                        let p = self.entities.items[k].param & 0xff;
                        let tk = self.entities.items[k].type_;
                        if tk != 76 {
                            if tk == 4 && p as i32 == group {
                                self.open_group.push(k);
                            }
                        } else if (p as i32) + 1 == group {
                            let gx = self.entities.items[k].px >> 5;
                            let gy = self.entities.items[k].py >> 5;
                            let ctk = self.grid.cell_type[gy as usize][gx as usize] as usize;
                            slots_to_clear.push((ctk, 0));
                            let cap3 = self.grid.capacity[ctk] as usize;
                            for q in 0..cap3 {
                                let id = self.grid.slots[ctk][q] as usize - 1;
                                self.entities.items[id].removed = true;
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
                if !step_triggered {
                    // a.java:5645 sort_i：83 触发/陷阱踩伤短路场输入
                    self.handle_field_input();
                }
            }
        } else if self.script_walk_armed {
            // 分支 2（a.java:5742-5746）：脚本行走武装 → advanceScriptWalk
            //（sort_i 踩伤 → 短路行走置 wp=0）
            if !step_triggered {
                self.advance_script_walk();
            } else {
                self.walk_phase = 0;
            }
        } else {
            // 分支 3（a.java:5747-5751）：空格 → 复位相位 + 场输入
            //（sort_i 踩伤短路 a.java:5645——陷阱臂迭代 66 已移植）
            self.walk_phase = 0;
            if !step_triggered {
                self.handle_field_input();
            }
        }
        // label216（a.java:5748-5761）：路径消费/清除——顶步 == 朝向则消费一步，
        // 否则整条清空 + 熄箭头；步数已尽也熄箭头
        // 【BUG-043】点地寻路期间 walkPathBuffer 装的是寻路路线：规则改作用于暂存的箭头步序
        //（否则会吃掉寻路步、并在寻路步尽时熄箭头）；经典面 walkto_arrows 恒 None，原逻辑一字不动
        if self.path_arrows_visible && self.walkto_arrows.is_some() {
            self.consume_walkto_arrow_step(walked_facing);
            self.settle_walkto_arrows();
        } else if self.path_arrows_visible {
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
        // a.java:7528 起 pickupItemType 拾取链：itemUseTargetType = 道具类型（描述 overlay 的图标/名）
        self.overlay_kind_item = t;
        // 描述表（itemDescriptions a.java:906 / equipDescriptions a.java:930——
        // 全量入库 reference；此处取用句柄）
        match t {
            13..=25 | 85 | 86 => {
                if t == 15 {
                    self.tutorial_seen = true; // tutorialSeen
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
                // 武器档（equip_types 1..5 槽）——equipTierIndex 档位 = 表下标
                let tier = match t {
                    33 => 1,
                    34 => 2,
                    35 => 3,
                    79 => 4,
                    _ => 5, // 36
                };
                if self.try_equip(tier, false) {
                    let eq_tier = if t < 37 {
                        match t { 33 => 1, 34 => 2, 35 => 3, 79 => 4, _ => 5 }
                    } else {
                        match t { 37 => 7, 38 => 8, 39 => 9, 80 => 10, _ => 11 }
                    };
                    self.overlay_kind_pickup(t, eq_tier);
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
                    // a.java:7595 `equipDescriptions[script_cursor0]`：script_cursor0 = equipTierIndex 的
                    // tier（10）——**非物品类型**，D 表按 tier 索引（迭代 70
                    // floor43 80 甲实证：传类型 80 = D 表越界错帧）。
                    self.overlay_kind_pickup(t, tier);
                } else {
                    self.show_overlay_message(0, "你拥有更强力装备", 0, 3);
                }
            }
            _ => {} // 41+ 怪物类型不可达（a.java:7608-7620 default return）
        }
    }

    /// addItemToItemStack（a.java:7721-7734）：栈顶追加或叠加次数 +
    /// itemUseTargetType 写 + showOverlayMessage(kind1, itemDescriptions[idx])。
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
        // itemUseTargetType = (byte)py_key（a.java:7732）：kind1 浮层图标/名的数据源
        // ——拾取路径调用方已先写同值，商人赠物路径（a.java:7333/7336）唯此一写
        self.overlay_kind_item = t;
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
    fn overlay_kind_pickup(&mut self, _t: i32, tier: i32) {
        const D: [&str; 12] = [
            "",
            "\\cdddddd一根相当长的木制长棍,新手必备.有了它杀人越货不慌不愁.\n\\c00ff00装备: 攻击+10.\n\\cFFCC00\"看上去似乎会断掉。\".",
            "\\cdddddd乌黑油亮，显然经历过多人之手。\n\\c00ff00装备: 攻击+30.\n\\cFFCC00\"很粗很结实！\".",
            "\\cdddddd银棍，恩，有这个名字就足够了。\n\\c00ff00装备: 攻击+70.\n\\cFFCC00\"只是根银棍\".",
            "\\cdddddd因乘天地之灵气，集日月之精华乃“万木之灵，灵木之尊”。\n\\c00ff00装备: 攻击+120.\n\\cFFCC00\"木之精华，削铁断金\".",
            "\\cdddddd您的需要，它知道；您的需求，它满足。它好，你也好，龙王后宫，镇宫之宝！\n\\c00ff00装备: 攻击+220.\n\\cFFCC00\"不要迷恋它，它只是一根传说。\".",
            "",
            "\\cdddddd没有太多的装饰，一件非常朴素、轻便的布衣.\n\\c00ff00装备: 防御+10.\n\\cFFCC00\"看上去有不少人用过了。\".",
            "\\cdddddd保暖御寒，腰不酸，腿不疼，走路也有劲了。\n\\c00ff00装备: 防御+30.\n\\cFFCC00\"豹纹，性感又野性，今年天宫最流行的皮草款式\".",
            "\\cdddddd如果没有上面的那行字，它也算是个杰作。\n\\c00ff00装备: 防御+70.\n\\cFFCC00\"上面写着'办四级神仙证书，回收二手莲花宝座'\".",
            "\\cdddddd华丽的装饰，就是有点旧。\n\\c00ff00装备: 防御+120.\n\\cFFCC00\"别人穿过的极品。\".",
            "\\cdddddd东海龙鳞编织而成，限量版，天上天下，只此一款。\n\\c00ff00装备: 防御+220.\n\\cFFCC00\"更轻薄，更透气，更多防护，更多安心\"."
        ];
        // a.java:7595-7597：图标 itemUseTargetType=py_key（物品类型）、文本
        // equipDescriptions[script_cursor0]（equipTierIndex 的 tier）——双参分列。tier 由调用
        // 点传入，此处**不可按 t 重算**（80 的 tier=10 经 t<37 分支会误落
        // _=>5 武器文案——迭代 70 floor43 T451 白烟盒文本错误实证）。
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
        // 【A 黄2·2026-10-23】Modern 对称清屏（照 BootMachine 先例）——
        // 升格钉当场红实证残留（局部绘制态留底，旧靠 paint_parallax 连带覆盖
        // ="暂不残留"非保障）；classic 路径零动（零漂移）。
        if self.layout.preset == crate::layout::Preset::Modern {
            g.set_color(crate::layout::WHITE);
            g.fill_rect(0, 0, self.layout.screen_w, self.layout.screen_h);
        }
        if self.game_mode == GameMode::EnemyInfo {
            self.paint_menu5(g);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::About {
            // paint case 16（a.java:2963-2990）：parallax(false) + 图标条 +
            // paintBoxFrame + 选项行（OptionList::paint）【task-14：内容居中；
            // 视差=全幅层自豁免】
            self.paint_parallax(g, false);
            let mut f = self.strip_frame;
            let sx = crate::title::menu_strip_x(self.layout.screen_w);
            crate::title::paint_icon_strip_at(g, &self.images.menu_strip, sx, 10, 34, 68, &mut f);
            self.strip_frame = f;
            if let Some(opt) = &self.options {
                let font = crate::paint::paint_font();
                opt.paint(g, &self.images.ui, &font, &self.layout);
            }
            m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::SaveSlot {
            // paint case 7（a.java:2679-2682）：paintSlotTable(true) 存档变体
            self.paint_slot_list(g);
            self.paint_overlay_box(g);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::Help {
            // paint case 15（a.java:2958-2962）
            self.paint_menu15(g);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::InGameMenu || self.game_mode == GameMode::Shop {
            // paint case 4/19 共享（a.java:2427-2490）
            // 【task-14 实测更正 2026-10-20】paint_menu_list=公式锚（center_x）
            // ⇒ 不加臂平移（双移=右偏 88px，ops 差分实证）；公式化=STAGE-B-004a 既定机制
            g.set_origin(0);
            self.paint_menu_list(g);
            self.paint_overlay_box(g);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::ItemPanel {
            self.paint_item_panel(g);
            self.paint_overlay_box(g);
            // 面板内道具触发 changeFloor：遮幅公共尾同样共跑（a.java:3177-3215）
            self.paint_wipe_tail(g);
            return;
        }
        if self.game_mode == GameMode::Ending {
            self.paint_ending(g);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::RecordBrowser {
            self.paint_mode12(g);
            return;
        }
        if self.game_mode == GameMode::Dialog {
            self.paint_dialog(g, tileset);
            // 【BUG-049】提示浮层在 paint 公共尾对所有模式共跑（a.java:3129-3168；switch 各 case 皆 break、无提前 return）：
            // mode 11 里弹出的提示（脚本 / 点地寻路行走撞上打不过的怪、无钥匙的门等）原先不画——画面无提示却吞键。
            self.paint_overlay_box(g);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::Alchemy {
            // paint case 9（a.java:2690-2740）
            self.paint_menu9(g, tileset);
            return self.paint_wipe_tail(g);
        }
        self.paint_parallax(g, true);
        let tile_off = self.layout.view_top_y + self.advance_open_group_paint();
        self.view.paint_tiles(g, tileset, tile_off);
        self.paint_entities(g, self.view.cam_x, self.view.cam_y + self.layout.view_top_y);
        self.paint_tap_markers(g, self.view.cam_x, self.view.cam_y + self.layout.view_top_y); // 【STAGE-B-008】
        // 小地图（optionChecked[1] && minimapImage）：右上角，玩家点
        if self.minimap_enabled {
            if let Some(mm) = &self.minimap {
                let ppc = self.layout.minimap_px_per_cell;
                let dot = self.layout.minimap_dot_px;
                let off = (ppc - dot) / 2; // 点居格中（classic 4/4 ⇒ 0，零偏离）
                let x = self.layout.screen_w - ((self.view.wide + 1) * ppc);
                g.draw_image(mm, x, self.layout.view_top_y, 0);
                g.set_color(MINIMAP_PLAYER_DOT);
                // a.java:2397-2398：点 = playerCellX/Y（**格坐标**——步进中
                // 与像素 >>5 不同，上/左行时像素已进目标格而格坐标未更新）
                g.fill_rect(
                    x + self.player_cell_x * ppc + off,
                    self.layout.view_top_y + self.player_cell_y * ppc + off,
                    dot,
                    dot,
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
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
        // 提示浮层（paint 公共尾 a.java:3129-3168，!floorTransitioning && overlayActive）
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
        // 【task-14】遮幅尾=全画布层——归零平移（各臂内容居中后统一复位点）
        g.set_origin(0);
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
            // 【BUG-022 修复·STAGE-B-002 铺满】网格界随 Layout 铺满画布——
            // Java 写死 40 行×30 列（=传统 240×320 全覆盖，deobf a.java:3217-3222
            // `var176<40`/`var181<30`），扩屏后只盖左上 240×320（切屏动画不覆盖
            // 全屏）。格数=ceil(屏/格距)：classic 240/8=30 列、320/8=40 行 代入
            // =原值零变化（格距 WIPE_CELL_PX=8 见 layout）。
            let cell = WIPE_CELL_PX;
            let cols = (self.layout.screen_w + cell - 1) / cell;
            let rows = (self.layout.screen_h + cell - 1) / cell;
            for row in 0..rows {
                let y = off + row * cell;
                for col in 0..cols {
                    g.fill_rect(off + col * cell, y, size, size);
                }
            }
        }
    }

    /// drawParallaxBackdrop（a.java:6850-6871）：滚动 -1/拍（PARALLAX_WRAP 回绕），
    /// PARALLAX_COL_STEP 列步进、屏高行步进、列内交替镜像。
    /// 【STAGE-B-002 返修②】列/行界与基线随 Layout 铺满画布（classic 240×320
    /// 代入=原值零变化：base=270-320+6=-44、列界 240、行界 -320）。
    fn paint_parallax(&mut self, g: &mut SoftGraphics<'_>, game_view: bool) {
        // 【task-14 豁免】背景=全幅层（铺满画布）——不随 origin 平移（否则左缘
        // 露空）；绘后恢复调用方平移（ops 面不受影响=origin 隐式）。
        let saved_origin = g.origin();
        g.set_origin(0);
        self.backdrop_scroll -= 1;
        if self.backdrop_scroll < PARALLAX_WRAP {
            self.backdrop_scroll = 0;
        }
        let bh = self.images.backdrop.height;
        let base_y = if game_view { self.layout.view_bottom_y - bh + 6 } else { 0 };
        let bw = self.images.backdrop.width as usize;
        let mut col_x = self.backdrop_scroll;
        while col_x < self.layout.screen_w {
            if self.layout.screen_h > bh {
                // 【v6 返修】画布高于背景图 ⇒ 边缘**钳制延展**（顶带=图顶行色、
                // 底带=图底行色）+ 仅画基行——镜像铺贴会把渐变反射（沙地翻上屏顶/
                // 天空翻下底带）=用户"颜色奇怪/过渡不自然"根因（v5 镜像补铺同病）；
                // classic 画布=图高 ⇒ 走下方原式原样。
                let top = self.images.backdrop.argb[0];
                let bottom = self.images.backdrop.argb[(bh as usize - 1) * bw];
                g.set_color(top & 0x00FF_FFFF | 0xFF00_0000);
                g.fill_rect(col_x, 0, bw as i32, base_y);
                g.set_color(bottom & 0x00FF_FFFF | 0xFF00_0000);
                g.fill_rect(col_x, base_y + bh, bw as i32, self.layout.screen_h - (base_y + bh));
                g.draw_image(&self.images.backdrop, col_x, base_y, 0);
            } else {
                // classic 原式（a.java:6850-6871）：行内交替镜像，一行可见=原样零变化
                let mut mirrored = false;
                let mut y = base_y;
                while y > -self.layout.screen_h {
                    if mirrored {
                        g.draw_image_transformed(&self.images.backdrop, col_x, y, 0, 2);
                    } else {
                        g.draw_image(&self.images.backdrop, col_x, y, 0);
                    }
                    mirrored = !mirrored;
                    y -= bh;
                }
            }
            col_x += PARALLAX_COL_STEP;
        }
        g.set_origin(saved_origin);
    }

    /// paintEntityLayer（a.java:6491-6717）：实体层。`cam_x/cam_y` 为调用参数
    /// （paint case 3：cameraPixelX / cameraPixelY+view_top）。
    /// floor1 类别覆盖 1/2/8 + 空图文字标签；16/32/67/69 拼装 todo（floor1
    /// 实体清单无这些类型，spec p3-render §3.5）。
    /// findBlockingEntity（a.java:8610-8625）：格上首个指定类型的未移除实体索引（-1 = 无）。
    fn entity_at_cell(&self, x: i32, y: i32, type_id: i32) -> i32 {
        entity::cell_entity(&self.grid, &self.entities, x, y, type_id)
    }

    /// 【STAGE-B-008】站上 (x, y) 是否会被陷阱伤到（点地寻路途中避让用；纯判定、无副作用）。
    /// 与 `apply_step_cell_effects` 的陷阱块逐式同判（a.java:5555-5621）：装备 40 全免；61 左右或上下夹击（两侧都须在界内）；
    /// 62 / 60 四邻（右邻判界用 view.high——原式怪癖照搬）。等价性由 `trap_predicate_matches_step_effects_everywhere` 对拍钉住。
    /// 返回的是"陷阱触发条件成立"，与当前 hp 无关（hp 很小时 apply_hp_delta 钳 1、实际可能不掉血；避让取保守）。
    /// 边框格上 Java 原式会越界抛异常（findBlockingEntity 不判界），实机边框不可走、不可达；此处返回 false。
    fn trap_damage_at(&self, x: i32, y: i32) -> bool {
        if self.equipped_armor == 40 {
            return false;
        }
        let hit_lr = x > 0
            && self.entity_at_cell(x - 1, y, 61) >= 0
            && x < self.view.wide - 1
            && self.entity_at_cell(x + 1, y, 61) >= 0;
        let hit_ud = y > 0
            && self.entity_at_cell(x, y - 1, 61) >= 0
            && y < self.view.high - 1
            && self.entity_at_cell(x, y + 1, 61) >= 0;
        if hit_lr || hit_ud {
            return true;
        }
        let adjacent = |t: i32| {
            (x > 0 && self.entity_at_cell(x - 1, y, t) >= 0)
                || (x < self.view.high - 1 && self.entity_at_cell(x + 1, y, t) >= 0)
                || (y > 0 && self.entity_at_cell(x, y - 1, t) >= 0)
                || (y < self.view.high - 1 && self.entity_at_cell(x, y + 1, t) >= 0)
        };
        adjacent(62) || adjacent(60)
    }

    /// 【STAGE-B-008 · spec §5.1】点地格分类（见 [`crate::tap_walk::TapGrid`]）。
    pub fn tap_grid(&self) -> crate::tap_walk::TapGrid {
        use crate::tap_walk::{TapCell, TapGrid, TargetKind};
        let (cols, rows) = (self.view.wide, self.view.high);
        let mut g = TapGrid::new(cols, rows);
        let rank = |k: &Option<TargetKind>| match k {
            Some(TargetKind::Monster(_)) => 3,
            Some(TargetKind::Alchemy) => 2,
            Some(TargetKind::Interact) => 1,
            _ => 0,
        };
        for y in 0..rows {
            for x in 0..cols {
                let ct = self.grid.cell_type[y as usize][x as usize] as usize;
                let cap = self.grid.capacity[ct] as usize;
                let mut blocker: Option<TargetKind> = None;
                let mut revealed_path = false;
                for s in 0..cap {
                    let e = self.grid.slots[ct][s] as usize - 1; // 槽存 1 基实体号（CellGrid::slot_push）
                    let ent = &self.entities.items[e];
                    if !ent.visible || ent.state == 1 || ent.removed {
                        continue; // 不可见 / 开门或死亡动画中 / 已移除：对寻路不存在
                    }
                    let t = ent.type_;
                    let cat = self.entity_category(t);
                    let k = if t == 9 {
                        Some(TargetKind::Alchemy)
                    } else if cat == 8 {
                        Some(TargetKind::Monster(e))
                    } else if cat == 2 || cat == 4 {
                        None
                    } else if t == 6 {
                        revealed_path = true;
                        None
                    } else {
                        Some(TargetKind::Interact)
                    };
                    if rank(&k) > rank(&blocker) {
                        blocker = k;
                    }
                }
                let kind = match blocker {
                    Some(k) => k,
                    None if self.walkable[y as usize][x as usize] || revealed_path => TargetKind::Floor,
                    None => TargetKind::Wall,
                };
                let danger = kind == TargetKind::Floor && self.trap_damage_at(x, y);
                g.set(x, y, TapCell { kind, danger });
            }
        }
        g
    }

    /// renderCategoryByType 的场景视角（a.java:9669-9667 m_122 尾）：type 72
    /// 类别**层相关**——floor50 = 8（可战斗），其余层 = 32。静态
    /// entity::render_category 只覆盖非 50 层语义（迭代 56：floor50 boss
    /// 走 cat32 臂缺 4px lift，T237 op 唯一分歧实证）。
    fn entity_category(&self, t: i32) -> u8 {
        // 【V2-P2 消费替换】floor50 类目覆写改查 sprite_defs.floor_category_override
        // （表驱动；旧特例 `t==72 && floor==50 → 8` 与表逐值等价）。
        crate::resource_cfg::global()
            .floor_category(t, self.floor)
            .unwrap_or_else(|| entity::render_category(t))
    }

    /// 型 67 片件表（a.java:682-697 f_byte_arr2_00/01 逐值）已迁
    /// `sheet_index.assembly_profiles.seg67` 表中表（9 片件 xywh + 3×5 段帧）——
    /// 查表入口 `resource_cfg::ResourceCfg::seg67`；绘制臂消费见 paint_entities
    /// 型 67 分支（tests/sheet_index_table_driven.rs::task7_seg67_follows_table 对账）。

    /// 【STAGE-B-008 · spec §6】点地反馈层：目标框（金）、闪红（红框 + 叉）、选中怪物（闪烁框；持火眼金睛（13）旁标预计掉血，
    /// 与观察模式战后数字条同源同位（a.java:6617-6638：通用怪先 −ENTITY_LIFT、69/67 分支不抬；标记为静态位、无波动相位）。
    /// 层级 = 实体层之上、HUD / 浮层之下。classic 零绘制 op。
    fn paint_tap_markers(&self, g: &mut SoftGraphics<'_>, cam_x: i32, cam_y: i32) {
        if self.layout.preset != crate::layout::Preset::Modern {
            return;
        }
        fn frame(g: &mut SoftGraphics<'_>, x: i32, y: i32, w: i32) {
            g.draw_rect(x, y, w, 32);
            g.draw_rect(x + 1, y + 1, w - 2, 30);
        }
        if let Some((x, y)) = self.tap_target {
            g.set_color(TAP_TARGET_RGB);
            frame(g, cam_x + (x << 5), cam_y + (y << 5), 32);
        }
        if let Some(((x, y), until)) = self.tap_reject {
            if self.frame_counter < until {
                let (sx, sy) = (cam_x + (x << 5), cam_y + (y << 5));
                g.set_color(TAP_REJECT_RGB);
                frame(g, sx, sy, 32);
                g.draw_line(sx + 8, sy + 8, sx + 23, sy + 23);
                g.draw_line(sx + 23, sy + 8, sx + 8, sy + 23);
            }
        }
        if let Some(s) = self.tap_selected {
            let w = entity::width_cells(s.type_, &self.width_table) << 5;
            let (sx, sy) = (cam_x + (s.cell.0 << 5), cam_y + (s.cell.1 << 5));
            g.set_color(if (self.frame_counter >> 1) & 1 == 0 { TAP_TARGET_RGB } else { TAP_SELECT_ALT_RGB });
            frame(g, sx, sy, w);
            if self.player.find_item(13) >= 0 {
                // 与观察模式数字条同基位：通用怪先 −ENTITY_LIFT（a.java:6613 var10 −= 4），69/67 不抬（复核 T8-R1）
                let lift = if s.type_ != 69 && s.type_ != 67 { ENTITY_LIFT } else { 0 };
                let digit = self.predict_hp_loss(s.type_, true);
                if digit >= 0 {
                    crate::popup::draw_digit_strip(g, &self.images.battle_strip, digit, sx + 30, sy - lift - 17);
                } else {
                    draw_image_clipped(g, &self.images.battle_strip, sx + 14, sy - lift - 17, 70, 0, 7, 9);
                }
            }
        }
    }

    pub fn paint_entities(&mut self, g: &mut SoftGraphics<'_>, cam_x: i32, cam_y: i32) {
        entity::advance_bob(&mut self.bob_offset, &mut self.bob_rising);
        let bob = self.bob_offset;
        let mut player_drawn = false;
        let cull_bottom = self.layout.view_top_y + self.view.view_h;

        for e in 0..self.entities.count {
            if !self.entities.items[e].removed && self.entities.items[e].visible {
                let ex = self.entities.items[e].px;
                let ey = self.entities.items[e].py;
                let w = self.entities.items[e].sprite_w;
                let h = self.entities.items[e].sprite_h;
                let state = self.entities.items[e].state;
                let t = self.entities.items[e].type_;
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
                        let anim = self.entities.items[e].anim_row as usize;
                        let frame = self.entities.items[e].anim_frame;
                        // 帧偏移 = 帧宽 × animOffsetTable[行][帧]（Java open_n 算式）
                        let frame_off = w * ANIM_OFFSET_TABLE[anim][frame as usize];
                        // 注：横向居中 (32-w)>>1 在**各臂内**取 sx 现值计算——
                        // case 8 抖动 / case 1 的 6/9 特例先改 sx 再用（Java cur_idx 语义）
                        let cx_off = (CELL_PX - w) >> 1;
                        match self.entity_category(t) {
                            1 => {
                                if t == 6 {
                                    sy += 5 - bob;
                                } else if t == 9 {
                                    sx += CELL_PX;
                                }
                                let img = if state == 1 { &self.images.opened_door } else { img };
                                draw_image_clipped(g, img, sx + cx_off, sy - (h - CELL_PX), frame_off, 0, w, h);
                            }
                            2 => {
                                if state == 1 {
                                    draw_image_clipped(g, &self.images.opened_door, sx + cx_off, sy - (h - CELL_PX), frame_off, 0, w, h);
                                } else if state == 2 {
                                    draw_image_clipped(g, &self.images.open_anim, sx + OPEN_ANIM_DX, sy + OPEN_ANIM_DX - (frame * OPEN_ANIM_STEP), frame_off, 0, OPEN_ANIM_W, OPEN_ANIM_H);
                                } else if t < SMALL_ITEM_MAX_TYPE {
                                    g.draw_image(&self.images.shadow, sx + SHADOW_DX, sy + SHADOW_DY, 0);
                                    g.draw_image(img, sx + cx_off, sy - (h - ANCHOR_MARGIN_24) + bob, 0);
                                } else {
                                    g.draw_image(img, sx + cx_off, sy - (h - ANCHOR_MARGIN_30), 0);
                                }
                            }
                            4 => {
                                g.draw_image(&self.images.shadow, sx + SHADOW_DX, sy + SHADOW_DY, 0);
                                draw_image_clipped(g, img, sx + cx_off, sy - (h - ANCHOR_MARGIN_24) + bob, frame_off, 0, w, h);
                            }
                            8 => {
                                if state == 3 {
                                    // 战斗目标抖动（a.java:6543-6541）：x/y 各随机 ±2
                                    sx += self.rng.random_below(5) - 2;
                                    sy += self.rng.random_below(5) - 2;
                                }
                                if state == 1 || state == 2 {
                                    draw_image_clipped(g, &self.images.open_anim, sx + OPEN_ANIM_DX, sy + OPEN_ANIM_DX - (frame * OPEN_ANIM_STEP), frame_off, 0, OPEN_ANIM_W, OPEN_ANIM_H);
                                } else if t == 69 {
                                    // a.java:6590-6600：型69 拼装——阴影 +40、
                                    // 全身图整图直画 (x+3, y−49+bob)（无裁剪无
                                    // 帧偏移）、帧偏移 >0 时 xtq[5] 26×13 效果帧
                                    //（src = 26×(偏移−1)）。
                                    g.draw_image(&self.images.shadow, sx + 40, sy + 22, 0);
                                    let raw_off = ANIM_OFFSET_TABLE[anim][frame as usize];
                                    let sx69 = sx + 3;
                                    let sy69 = sy - 49 + self.bob_offset;
                                    g.draw_image(img, sx69, sy69, 0);
                                    if raw_off > 0 {
                                        draw_image_clipped(
                                            g,
                                            &self.images.xtq[5],
                                            sx69 + 32,
                                            sy69 + 27,
                                            (raw_off - 1) * 26,
                                            0,
                                            26,
                                            13,
                                        );
                                    }
                                } else if t != 67 {
                                    g.draw_image(&self.images.shadow, sx + SHADOW_DX, sy + SHADOW_DY, 0);
                                    sy -= ENTITY_LIFT;
                                    draw_image_clipped(g, img, sx + cx_off, sy - (h - CELL_PX), frame_off, 0, w, h);
                                } else {
                                    // 型 67 多段拼装（a.java:6580-6603 逐字）：
                                    // 影子 @ (x+8, y+22)（6581）→ 基点 (x+16,
                                    // y+16)（6582-6586 倒手）→ SEG67_FRAMES[
                                    // 裸帧偏移] 5 段，每段片件 SEG67_PIECES[
                                    // seg[0]] 画 xtq[seg.sheet] @ (基点+dx,
                                    // 基点+dy) 源 (srcX, srcY) w×h（6591-6601）。
                                    // 裸帧偏移 = ANIM_OFFSET_TABLE[anim][frame]
                                    // **不乘精灵宽**（a.java:6578，对照通用臂
                                    // 6528 乘宽）。型 67 精灵盒 32×32（6323-6324）。
                                    // 【2026-10-16 注】deobf:6592 将片件表元素
                                    // 强转 (Object[]) 与 :682 的 byte[][] 声明
                                    // 矛盾——C 变体（deobf 重编译）画 67 即
                                    // ClassCastException，系投影伪影非原字节码
                                    // 语义；Rust 侧以 (sheet,srcX,srcY,w,h) 元组
                                    // 建模规避，与 A 变体 fx-boss67a 逐帧像素
                                    // 等价。类型语义合同移交
                                    // reference/oracle/scenarios/
                                    // bug-draft-paint67-deobf-cast.md（lead 入库）。
                                    g.draw_image(&self.images.shadow, sx + 8, sy + 22, 0);
                                    let base_x = sx + 16;
                                    let base_y = sy + 16;
                                    let seg_frame = ANIM_OFFSET_TABLE[anim][frame as usize] as usize;
                                    let seg67 = crate::resource_cfg::global().seg67();
                                    for seg in seg67.frames[seg_frame].clone() {
                                        let (sheet, src_x, src_y, w, h) =
                                            seg67.pieces[seg.0 as usize];
                                        draw_image_clipped(
                                            g,
                                            &self.images.xtq[sheet as usize],
                                            base_x + seg.1,
                                            base_y + seg.2,
                                            src_x,
                                            src_y,
                                            w,
                                            h,
                                        );
                                    }
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
                                    draw_image_clipped(g, img, sx + cx_off, sy - (h - ANCHOR_MARGIN_24) + bob, frame_off, 0, w, h);
                                } else {
                                    draw_image_clipped(g, &self.images.open_anim, sx + OPEN_ANIM_DX, sy + OPEN_ANIM_DX - (frame * OPEN_ANIM_STEP), frame_off, 0, OPEN_ANIM_W, OPEN_ANIM_H);
                                }
                            }
                            32 => {
                                g.draw_image(&self.images.shadow, sx + SHADOW_DX, sy + SHADOW_DY, 0);
                                if state == 1 || state == 2 {
                                    draw_image_clipped(g, &self.images.open_anim, sx + OPEN_ANIM_DX, sy + OPEN_ANIM_DX - (frame * OPEN_ANIM_STEP), frame_off, 0, OPEN_ANIM_W, OPEN_ANIM_H);
                                } else if t != 72 {
                                    draw_image_clipped(g, img, sx + cx_off, sy - (h - ANCHOR_MARGIN_16) + bob, frame_off, 0, w, h);
                                } else if self.route_entity != e as i32 {
                                    // 型 72 非追踪实体：静态（无 bob），y-(h-32)（a.java:6657-6652）
                                    draw_image_clipped(g, img, sx + cx_off, sy - (h - CELL_PX), frame_off, 0, w, h);
                                } else {
                                    // 追踪实体：步方向帧行（f_byte_19；==3 时镜像，a.java:6663-6664）
                                    let mut src_y = h * self.route_dir;
                                    if self.route_dir == 3 {
                                        src_y -= h;
                                        crate::menu_family::draw_edge_patch(g, img, sx + cx_off, sy - (h - CELL_PX), frame_off, src_y, w, h, 1);
                                    } else {
                                        draw_image_clipped(g, img, sx + cx_off, sy - (h - CELL_PX), frame_off, src_y, w, h);
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        // 空图文字标签分支（a.java:6672-6683）：**全语料不可达
                        // 死码**——A 级证明链：空图类型仅 82/83（m_122 图表
                        // 9456-9485 二者恒 null）；82/83 出层即 hidden
                        // （a.java:9605/9618）；entityVisible 置 true 写点清单
                        // （6407/5413/5455/8345/9638）无一覆盖 82/83；语料
                        // CES/ADD 类型参数不含 82/83；全 55 层 sprite 型 82=0、
                        // 83=52 全隐藏 ⇒ 循环门 entityVisible 恒 false ⇒ 分支
                        // 不可达（evidence-todos.md T-3 裁定 B 升 A 且更强=全
                        // 语料域）。保真期**不删分支**：todo!() 作死码哨兵；
                        // 分支几何公式已取证（6673-6682），若未来语料以
                        // CES/ADD 生成 82/83 或 SWD 改型 83 则按公式实现。
                        todo!(
                            "paintEntityLayer 空图标签分支：全语料不可达死码（evidence-todos.md T-3）"
                        )
                    }
                }
            }
        }

        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
        if !player_drawn {
            self.paint_player(g, cam_x, cam_y);
        }
        // walkPhase==5 星光（a.java:6692-6704）：battleTick&7 选帧，sparkFrames
        // 帧号×4 取 sparkQuads 四联组（sx,sy,w,h）
        if self.walk_phase == 5 && self.battle_target >= 0 {
            let e = self.battle_target as usize;
            // a.java:6692-6687：星光坐标用**裸 cameraPixelX/Y**（非 paintEntityLayer 入参
            // 的 cameraPixelY+view_top——尾段直接读字段）
            let x = self.view.cam_x + self.entities.items[e].px + 16;
            let y = self.view.cam_y + self.entities.items[e].py + 32;
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
        // 【STAGE-B-002 偏离点·观察形态(a)（lead 裁决 2026-10-19，用户批准）】
        // 空格观察形态（walkPhase==2 未锁定）modern：**角色恒居中**（与残影蛇
        // 收敛点同锚=视口中心 easeAfterimages 目标），地图随相机滑动=以角色
        // 为中心环视；classic 保持原机制（cam+player）零偏离。
        let soul_view = self.walk_phase == 2
            && !self.view_locked
            && self.layout.preset == crate::layout::Preset::Modern;
        let (px, mut py) = if soul_view {
            // 与残影蛇收敛点严格同点（easeAfterimages 目标换屏坐标=视口中心）：
            // cam_y 含 view_top 分量（paint_entities 传参），故回补之
            (
                (self.view.view_w - 41) >> 1,
                (cam_y - self.view.cam_y) + (self.view.view_h >> 1) + self.bob_offset,
            )
        } else {
            (cam_x + self.view.player_px, cam_y + self.view.player_py)
        };
        if !soul_view && self.player_bob_applied {
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
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
        // pathArrowsVisible 路径箭头（a.java:5929-5956）：自玩家格沿 walkPathBuffer 逐格外扩，
        // buffer 顶 = 玩家侧第一步；[8][22] 三联贴片 21×22（0 下/1 上裁剪源、2 右镜像、3 左），
        // 0/1 的 bob 加在 y、2/3 加在 x（Java 原样）；setColor(136) 每步重设。
        if self.path_arrows_visible {
            let mut ax = (self.player_cell_x << 5) + cam_x;
            let mut ay = (self.player_cell_y << 5) + cam_y;
            let bob = self.bob_offset;
            let piece = &self.images.ui[22];
            let (buf, count) = self.path_arrow_steps();
            for i in (0..count).rev() {
                g.set_color(136);
                match buf[i] {
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
            // 【BUG-031①】a.java:5964-5967 字面量 148 / 109 / 215 = 屏心 y − 12 / 屏心 x − 11 / 屏宽 − 25
            // （classic 回代原值）；下臂 viewBottom 原本即变量——同块另三臂随之换轨。
            let (cx, cy, w) = (self.layout.center_x, self.layout.center_y, self.layout.screen_w);
            draw_image_clipped(g, &self.images.corner_sprites, 5 - bob, cy - 12, 44, 0, 22, 24);
            draw_image_clipped(g, &self.images.corner_sprites, cx - 11, 25 - bob, 0, 0, 22, 24);
            draw_image_clipped(g, &self.images.corner_sprites, cx - 11, vb - 29 + bob, 22, 0, 22, 24);
            crate::menu_family::draw_edge_patch(g, &self.images.corner_sprites, w - 25 + bob, cy - 12, 44, 0, 22, 24, 1);
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
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
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
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
    }

    /// paintHudPanel（a.java:6052-6106）：HUD（头像/HP/攻/防/武器/甲）。
    pub fn paint_hud(&mut self, g: &mut SoftGraphics<'_>) {
        let font = crate::paint::paint_font();
        // STAGE-B-004b ②：底部 HUD 信息栏锚位公式化（menu_family::hud_panel_anchor
        // 贴底随宽，v5 底部带背景同款拉伸判据；classic 代入化回 bottom_panel_dx()+240
        // 原值）。立绘/属性条内容锚仍走 Layout（hud_face_ax/hud_icon_x 居中不放大）。
        let anchor = crate::menu_family::hud_panel_anchor(self.layout.screen_w, self.view_bottom, self.layout.hud_frame_h);
        let base_y = anchor.y;
        self.paint_hud_frame(g, anchor.x, base_y, anchor.w, anchor.h);
        let face = &self.images.actor_face;
        g.draw_image(face, self.layout.hud_face_ax - (face.width >> 1), base_y + HUD_FRAME_H - face.height, 0);
        let icon_x = self.layout.hud_icon_x;
        let bar_x = icon_x + HUD_BAR_DX;
        let mut y = base_y + HUD_STAT_Y0;
        // HP 行：图标 HUD_ICON_CLIP 见方 + 条框 + 数值
        g.set_clip(icon_x, y, HUD_ICON_CLIP, HUD_ICON_CLIP);
        g.draw_image(&self.images.ui[7], icon_x, y + HUD_ICON_LIFT, 0);
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
        paint_mini_frame(g, bar_x, y + 1, HUD_BAR_W, HUD_BAR_H);
        paint_number(g, &self.images.ui[2], self.player.hp, bar_x + HUD_NUM_DX, y + 2);
        // 攻行：图标条带片 (10,0,10,13)
        y += HUD_ROW_STEP;
        draw_image_clipped(g, &self.images.ui[7], icon_x, y, 10, 0, HUD_ICON_CLIP, HUD_ICON_STRIP_H);
        g.set_color(HUD_ROW_ACCENT);
        paint_mini_frame(g, bar_x, y + 1, HUD_BAR_W, HUD_BAR_H);
        paint_number(g, &self.images.ui[2], self.player.atk, bar_x + HUD_NUM_DX, y + 2);
        // 防行：图标条带片 (20,0,10,13)
        y += HUD_ROW_STEP;
        draw_image_clipped(g, &self.images.ui[7], icon_x, y, 20, 0, HUD_ICON_CLIP, HUD_ICON_STRIP_H);
        g.set_color(HUD_ROW_ACCENT);
        paint_mini_frame(g, bar_x, y + 1, HUD_BAR_W, HUD_BAR_H);
        paint_number(g, &self.images.ui[2], self.player.def, bar_x + HUD_NUM_DX, y + 2);
        // 装备槽（a.java:6079-6110）：equipped==0 → 白字“无”
        // （equipmentMaterialNames[0]）；否则画 entityTypeImage[equipped]
        // 居中（槽 32×32，(32-图宽>>1)/(32-图高>>1)）。
        // 槽 x：var11 = HUD_ICON_X+16 后 +60 → 159；甲槽 +34 → 193（a.java:6080/6087）
        let slot_y = base_y + HUD_SLOT_DY;
        let mut slot_x = bar_x + 60;
        for equipped in [self.equipped_weapon, self.equipped_armor] {
            paint_mini_frame(g, slot_x, slot_y, HUD_SLOT_SIZE, HUD_SLOT_SIZE);
            if equipped == 0 {
                let none: Vec<u16> = "无".encode_utf16().collect();
                let cw = font.char_width(none[0]);
                g.set_color(WHITE);
                g.draw_string(
                    &none,
                    slot_x + ((HUD_SLOT_SIZE - cw) >> 1),
                    slot_y + ((HUD_SLOT_SIZE - font.height()) >> 1),
                    0,
                );
            } else if let Some(img) = self
                .images
                .entity
                .get(equipped as usize)
                .and_then(|o| o.as_ref())
            {
                g.draw_image(
                    img,
                    slot_x + ((HUD_SLOT_SIZE - img.width) >> 1),
                    slot_y + ((HUD_SLOT_SIZE - img.height) >> 1),
                    0,
                );
            }
            slot_x += HUD_SLOT_STEP;
        }
    }

    /// paint case 11（a.java:2798-2896）：世界层 + 对话框/立绘/气泡/翻页箭头。
    /// overlayText==null 或 overlayActive 或 phase==4 时走 paintHudPanel 分支。
    /// paint case 20（a.java:3039-3067）：结局卷轴。状态推进在 paint 内
    /// （Java 原样）：滚动 f_int_22 奇数拍 ++（>w-240 → f_bool_04），
    /// 打字机 (fc&3)>>1 门控推进并重排 209 宽限版；信箱 3156024 上下
    /// 各 img_y0 高；卷轴画 (−f_int_22, img_y0)；底栏 paintWrappedText
    /// (texts[260], 15, 320−fontHeight−10, 210, fontHeight+10)。
    fn paint_ending(&mut self, g: &mut SoftGraphics<'_>) {
        let img = self.images.ending.clone();
        let text = self.dialog_data.texts[260].clone();
        let font = crate::paint::paint_font();
        let img_y0 = (self.layout.screen_h - img.height) >> 1;
        self.ending_anim += 2;
        if !self.ending_done && (self.frame_counter & 1) != 0 {
            self.ending_scroll += 1;
            if self.ending_scroll > img.width - SCREEN_W {
                self.ending_done = true;
            }
        }
        let text_len = text.chars().count() as i32;
        if (self.frame_counter & 3) >> 1 != 0 && self.dialog_char < text_len {
            self.dialog_char += 1;
            let line_h = font.height() + 4;
            self.layout_wrapped_text_limited(&text, 209, (line_h << 1) + 12, self.dialog_char);
            self.overlay_page_top = (self.overlay_total_lines - 1).max(0);
            self.overlay_page_bottom = self.overlay_total_lines;
        }
        g.set_color(3156024);
        // 【A 黄2·2026-10-23】顶带几何源改 layout（旧 classic 常量 SCREEN_W ⇒
        // Modern 576 下右 336px 不铺+横滚图右缘露底——与下带相邻行对照实锤；
        // 根因=classic 常量泄漏非缺清屏，三问选根修不动帧面清屏语义）
        g.fill_rect(0, 0, self.layout.screen_w, img_y0);
        g.fill_rect(0, self.layout.screen_h - img_y0, self.layout.screen_w, img_y0);
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
        g.draw_image(&img, -self.ending_scroll, img_y0, 0);
        g.set_color(0xFFFF_FFFF);
        self.paint_wrapped_text(g, 15, self.layout.screen_h - font.height() - 10, 210, font.height() + 10, false);
    }

    /// browseOfferQueue（a.java:7480-7503）：日志页钳位翻转 + kind 选表 + 页码指示。
    /// kind=0 → f_String_arr_06，其余 → f_String_arr_09；总条数 0 →
    /// "没有记录" 回 mode10。
    fn open_record_page(&mut self, page: i32) {
        let total = self.journal.len() as i32;
        if total > 0 {
            self.record_page = if page < 0 {
                total - 1
            } else if page >= total {
                0
            } else {
                page
            };
            let (kind, idx) = self.journal[self.record_page as usize];
            let text = if kind == 0 {
                self.record_text_a.get(idx as usize).cloned().unwrap_or_default()
            } else {
                self.record_text_b.get(idx as usize).cloned().unwrap_or_default()
            };
            self.show_overlay_message(5, &text, 0, 3);
            self.page_indicator = format!("{} / {}", self.record_page + 1, total);
        } else {
            self.show_overlay_message(0, "没有记录", 0, 3);
            self.transition_to(GameMode::ItemPanel);
        }
    }

    /// paint case 12（a.java:2866-2880）：视差 + 页码描边指示 + 双箭头
    /// （左 transform 镜像 60−bob / 右 160+bob）+ 软键条。
    fn paint_mode12(&mut self, g: &mut SoftGraphics<'_>) {
        self.paint_parallax(g, false);
        let text_bottom_y = self.layout.screen_h - crate::paint::paint_font().height - 4;
        let indicator = self.page_indicator.clone();
        // 【BUG-031④】a.java:2906-2914 页码 x 120 / 箭头 60−bob、160+bob = 屏心派生 ⇒ Layout 字段
        //（center_x / arrow_pair_left_x / arrow_pair_right_x，classic 回代原值）
        let cx = self.layout.center_x;
        self.draw_shadow_text(g, &indicator, cx, text_bottom_y + 2, 4202520, 0xFFFF_FFFFu32 as i32);
        // 【BUG-032】a.java:2908 `var46 += fontHeight + 4 - 13 >> 1`：复合赋值右侧整体 = (fh+4−13)>>1
        // （javap 5242-5255：iload_2 … iconst_1 ishr iadd）——旧译 `(var46 + …) >> 1` 把行底也减半（classic
        // 画到 y 153，Java 304）。
        let arrow_y = text_bottom_y + ((crate::paint::paint_font().height + 4 - 13) >> 1);
        let arrow = self.images.ui[14].clone();
        let bob = self.bob_offset;
        if self.record_page > 0 {
            // a.java:2910 drawImageWithNokiaTransform(…, 1) ⇒ 变换码 = NOKIA_TRANSFORM_TABLE[1] = 8192（水平翻转；
            // Java trace `drawImageT(20x13,60,304,0,8192)`）。【BUG-032】旧直传 1（shim 表 1 = 垂直翻转）。
            g.draw_image_transformed(&arrow, self.layout.arrow_pair_left_x - bob, arrow_y, 0, crate::paint::NOKIA_TRANSFORM_TABLE[1]);
        }
        if self.record_page < self.journal.len() as i32 - 1 {
            g.draw_image(&arrow, self.layout.arrow_pair_right_x + bob, arrow_y, 0);
        }
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
        // 公共尾 overlay 框（paintOverlayBox）：日志文本 kind5 框在指示/箭头之上
        self.paint_overlay_box(g);
    }

    /// drawOutlinedText（a.java:10221-10236）：8 向 ±1 暗色描边 + 主色居中
    /// （anchor 17 = HCENTER|BASELINE；坐标按反编译的绕圈次序）。
    fn draw_shadow_text(&mut self, g: &mut SoftGraphics<'_>, text: &str, cx: i32, cy: i32, shadow: i32, main: i32) {
        let codes: Vec<u16> = text.encode_utf16().collect();
        let mut x = cx - 1;
        let mut y = cy - 1;
        g.set_color(shadow as u32);
        // 初始点 (cx-1, cy-1) 先画（Java script_cursor0--/table_count-- 后立即首画），
        // 再按反编译绕圈次序走完 8 邻域
        g.draw_string(&codes, x, y, 17);
        // Java 序：右移画、右移画、**下移画**（(cx+1,cy-1) 与 (cx+1,cy)
        // 各一画）——非斜跳；8 邻域全覆盖后主色居中
        for (dx, dy) in [(1, 0), (1, 0), (0, 1), (0, 1), (-1, 0), (-1, 0), (0, -1)] {
            x += dx;
            y += dy;
            g.draw_string(&codes, x, y, 17);
        }
        // 末次 script_cursor0++（a.java:10233）：主色落位 = 居中点 (cx, cy)
        x += 1;
        g.set_color(main as u32);
        g.draw_string(&codes, x, y, 17);
    }

    /// paintTileLayer 头段（a.java:6990-7014）：wp=4 组开门动画 **每 paint 推进**——
    /// 计数 18 置 row6/state1（开门画切换）、24 removeGroupAtCellByType(4) 收尾
    /// wp=0；tileBob ±2 交替供 tile 层 Y 偏移（20±2）。必须在 **paint 时点**推进
    /// （Java 本就在 paintTileLayer 内）：若放 tick 内先推进，同拍 advance_frames
    /// 会立刻多推一帧——迭代 64 T924 实证（Rust src=45 vs Java src=0）。
    /// paint case 3/9/11 三处 paintTileLayer(0,20) 调用点均须先跑本函数。
    fn advance_open_group_paint(&mut self) -> i32 {
        if self.walk_phase == 4 {
            self.open_anim_tick += 1;
            if self.open_anim_tick == 18 {
                let group = self.open_group.clone();
                for e in group {
                    self.entities.items[e].anim_row = 6;
                    self.entities.items[e].state = 1;
                }
            } else if self.open_anim_tick >= 24 {
                let group = std::mem::take(&mut self.open_group);
                for e in group {
                    let gx = self.entities.items[e].px >> 5;
                    let gy = self.entities.items[e].py >> 5;
                    entity::remove_group_at_cell_by_type(gx, gy, 4, &mut self.entities, &mut self.grid);
                }
                self.open_anim_tick = 0;
                self.tile_bob = 0;
                self.walk_phase = 0;
            }
            // tileBob ±2 交替（a.java:7016-7014）
            self.tile_bob = if self.tile_bob == -2 { 2 } else { -2 };
            // a.java:7015 `script_cursor0 = 20 + tileBob`：**重置拍（cnt24）也取 bob**
            //（wp 已清 0 但 script_cursor0 在块内计算）——迭代 64 T930 一像素差实证
            return self.tile_bob;
        }
        0
    }

    fn paint_dialog(&mut self, g: &mut SoftGraphics<'_>, tileset: &ArgbImage) {
        self.paint_parallax(g, true);
        let tile_off = self.layout.view_top_y + self.advance_open_group_paint();
        self.view.paint_tiles(g, tileset, tile_off);
        self.paint_entities(g, self.view.cam_x, self.view.cam_y + self.layout.view_top_y);
        self.paint_tap_markers(g, self.view.cam_x, self.view.cam_y + self.layout.view_top_y); // 【STAGE-B-008】
        self.popups.draw(g, &self.images.popup);
        self.paint_status_bar(g);
        let show_box = self.overlay_text.is_some() && !self.overlay_active && self.dialog_phase != 4;
        if show_box {
            let font = crate::paint::paint_font();
            let line_h = font.height() + 4;
            let dialog_block_h = (line_h << 1) + 12;
            let dialog_block_y0 = self.layout.screen_h - dialog_block_h;
            // 【STAGE-B-002 口径修正 2026-18-18】底部面板=**经典尺寸固定+整体居中**
            //（非随屏拉伸——二轮返修走偏）：面板内相对布局 classic 同构（常量回代），
            // 面板整体平移 dx；classic dx=0 零偏离。
            let dx = self.bottom_panel_dx();
            self.paint_hud_frame(g, dx, dialog_block_y0, 80, dialog_block_h);
            let mut cx = 80 + dx;
            while cx < 214 + dx {
                g.set_clip(cx, dialog_block_y0, 16, 16);
                g.draw_image(&self.images.ui[0], cx - 26, dialog_block_y0, 0);
                cx += 16;
            }
            g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
            // a.java:2828：edge patch 用 draw_y=dialog_block_y0（+=16 之前）
            crate::menu_family::draw_edge_patch(g, &self.images.ui[0], 214 + dx, dialog_block_y0, 0, 0, 26, 16, 1);
            g.set_color(HUD_FILL);
            g.fill_rect(80 + dx, dialog_block_y0 + 16, 149, dialog_block_h - 16);
            // a.java:2832-2836：右条带循环 (229, draw_y) 画 ui[8][0]@176——
            // draw_y 从 dialog_block_y0+16 步进 16 直到 var158+var11（320）
            let mut draw_y = dialog_block_y0 + 16;
            while draw_y < dialog_block_y0 + dialog_block_h {
                g.set_clip(229 + dx, draw_y, 11, 16);
                g.draw_image(&self.images.ui[0], 176 + dx, draw_y, 0);
                draw_y += 16;
            }
            g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
            if self.dialog_speaker > 0 {
                if let Some(img) = &self.images.entity[self.dialog_speaker as usize] {
                    if self.dialog_speaker != 72 {
                        let fw = self.type_frame_width_for(self.dialog_speaker);
                        draw_image_clipped(g, img, ((80 - fw) >> 1) + dx, self.layout.screen_h - img.height, 0, 0, fw, img.height);
                    } else {
                        // a.java:2852 `drawImageClipped(var9, 24, 275, 0, 0, 32, 45)`：275 = 320 − 45
                        // （定尺 32×45 剪裁贴屏底，同段两臂 `320 − h` 同义）⇒ 公式 screen_h − 45，
                        // classic 回代 275 零偏离。【BUG-029】旧照抄字面量 275 ⇒ Modern 下牛魔王
                        // 立绘画到地图中左、对话框头像格空。
                        draw_image_clipped(g, img, 24 + dx, self.layout.screen_h - 45, 0, 0, 32, 45);
                    }
                }
            } else if self.dialog_speaker == 0 {
                let face = &self.images.actor_face;
                g.draw_image(face, 40 - (face.width >> 1) + dx, self.layout.screen_h - face.height, 0);
            } else {
                g.set_color(WHITE);
                let q: Vec<u16> = "?".encode_utf16().collect();
                g.draw_string(&q, 40 + dx, dialog_block_y0 + ((dialog_block_h - font.height()) >> 1), 17);
            }
            g.set_color(10473684);
            self.paint_wrapped_text(g, 85 + dx, dialog_block_y0 + 12, 129, dialog_block_h - 12, false);
            if self.overlay_page_top > 0 {
                draw_image_clipped(g, &self.images.ui[15], 220 + dx, self.layout.screen_h - line_h - 10, 0, 0, 7, 9);
            }
            if self.overlay_page_bottom < self.overlay_total_lines {
                draw_image_clipped(g, &self.images.ui[15], 220 + dx, self.layout.screen_h - line_h + 1, 7, 0, 7, 9);
            }
            if (self.dialog_phase == 6 || self.dialog_phase == 1)
                && self.dialog_page >= 0
                && (self.dialog_page as usize) < self.dialog_data.bubbles.len()
            {
                let bubble = self.dialog_data.bubbles[self.dialog_page as usize];
                if bubble >= 0 {
                    let (bx, by) = if self.dialog_anchor >= 0 {
                        let e = self.dialog_anchor as usize;
                        (self.entities.items[e].px + 16, self.entities.items[e].py - 30)
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
    /// 锚位随 `self.layout`（STAGE-B-002 ②：楼层贴左/钥匙金币贴右/中间填充界）。
    pub fn paint_status_bar(&mut self, g: &mut SoftGraphics<'_>) {
        let base_y = 0;
        let font = crate::paint::paint_font();
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
        g.draw_image(&self.images.ui[3], 0, base_y, 0);
        if self.floor > 50 {
            // "引子" outline 文本（drawOutlinedText a.java:4487-4495）：anchor 17 =
            // TOP(16)|HCENTER(1) 顶部居中——8 向偏移点各画一次全串（色[0]）+
            // 中心 fill（色[1]）。shim 常量：VCENTER=2 非 16！
            let text: Vec<u16> = "引子".encode_utf16().collect();
            let x0 = 32 - 1;
            let y0 = base_y + 9 + ((19 - font.height()) >> 1) - 1;
            let outline = [4202520u32, WHITE];
            let deltas = [(1i32, 0i32), (1, 0), (0, 1), (0, 1), (-1, 0), (-1, 0), (0, -1), (0, -1)];
            let mut pts = Vec::new();
            let (mut x, mut y) = (x0, y0);
            for (dx, dy) in deltas {
                x += dx;
                y += dy;
                pts.push((x, y));
            }
            pts.push((x0 + 1, y0 + 1)); // 中心 fill（script_cursor0++→32、table_count 循环回 y0+1）
            for (i, (px, py)) in pts.iter().enumerate() {
                g.set_color(if i == 8 { outline[1] } else { outline[0] });
                g.draw_string(&text, *px, *py, 17);
            }
        } else {
            paint_number(g, &self.images.ui[18], self.floor, STATUS_FLOOR_NUM_X, base_y + 4 + 2);
            g.draw_image(&self.images.ui[8], STATUS_TIER_ICON_X, base_y + 4 + 10, 0);
        }
        g.set_color(STATUS_FILL);
        let fill_x = self.layout.status_fill_x;
        g.fill_rect(fill_x, base_y, self.layout.screen_w - fill_x, STATUS_BAR_H);
        self.paint_key_slot(g, 0, self.player.yellow_keys, self.layout.status_key_x[0], base_y);
        self.paint_key_slot(g, 1, self.player.blue_keys, self.layout.status_key_x[1], base_y);
        self.paint_key_slot(g, 2, self.player.red_keys, self.layout.status_key_x[2], base_y);
        let y = base_y + 3;
        g.draw_image(&self.images.ui[1], self.layout.status_gold_icon_x, y, 0);
        paint_number(g, &self.images.ui[2], self.player.gold, self.layout.status_gold_num_x, y + 2);
    }

    /// paintKeySlot（a.java:6043-6044）：钥匙格（ui[8][6] KEY_SLOT_W 三联图 + 数量）。
    fn paint_key_slot(&self, g: &mut SoftGraphics<'_>, kind: i32, count: i32, x: i32, y: i32) {
        g.set_clip(x, y, KEY_SLOT_W, KEY_SLOT_H);
        g.draw_image(&self.images.ui[6], x - kind * KEY_SLOT_W, y, 0);
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
        paint_number(g, &self.images.ui[2], count, x + KEY_NUM_DX, y + KEY_NUM_DY);
    }
}

/// blit5Clip（a.java:4557-4556）：MINIMAP_MARKER_PX 定尺寸裁剪贴图（小地图标记）。
fn m_003_clip5(g: &mut SoftGraphics<'_>, img: &ArgbImage, x: i32, y: i32, src_x: i32, reset_w: i32, reset_h: i32) {
    g.set_clip(x, y, MINIMAP_MARKER_PX, MINIMAP_MARKER_PX);
    g.draw_image(img, x - src_x, y, 0);
    g.set_clip(0, 0, reset_w, reset_h);
}

/// 场景存档槽表（mode 7）装备框接线：paint_slot_list 图标分支（entityTypeImage
/// 反查 sptarm）+ markSlotEntry 装备拷贝（a.java:9013-9014）。像素判据见
/// [`crate::menu_family::slot_equip_test_support`]。
mod slot_equip_scene_tests {
    use crate::render::FontMetrics;
    use super::*;
    use crate::menu_family::slot_equip_test_support::*;

    fn solid(w: i32, h: i32) -> ArgbImage {
        ArgbImage::from_rgb(&vec![0xFF202020; (w * h) as usize], w, h, true)
    }

    fn many(n: usize) -> Vec<ArgbImage> {
        (0..n).map(|_| solid(24, 24)).collect()
    }

    /// maplv1 夹具 + 合成图集（sptarm = synthetic_sptarm，经 m_122 映射进 entity）。
    fn fixtures() -> (Vec<i32>, Vec<i32>) {
        let width_table: Vec<i32> = include_str!("../tests/fixtures/type_width_px.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect();
        let walk: Vec<i32> = include_str!("../tests/fixtures/tile_walkability.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect();
        (width_table, walk)
    }

    fn synth_images(icons: &[ArgbImage]) -> SceneImages {
        SceneImages {
            backdrop: solid(77, SCREEN_H),
            shadow: solid(24, 24),
            actor: solid(24, 24),
            actor_face: solid(24, 24),
            ui: many(25),
            stair_icons: (solid(24, 24), solid(24, 24)),
            open_anim: solid(24, 24),
            opened_door: solid(24, 24),
            entity: SceneImages::map_entity_images(&many(13), &many(23), icons, &many(20), &many(20)),
            popup: crate::popup::PopupImages {
                digit_strips: [solid(24, 24), solid(24, 24), solid(24, 24)],
                icon_strip: solid(24, 24),
                battle_digits: solid(24, 24),
            },
            spark_sheet: solid(24, 24),
            corner_sprites: solid(24, 24),
            map211: None,
            map210: solid(24, 24),
            menu_strip: solid(24, 24),
            battle_icon: solid(24, 24),
            battle_strip: solid(24, 24),
            faces: many(12),
            ending: solid(24, 24),
            xtq: many(6),
        }
    }

    fn scene_with(icons: &[ArgbImage]) -> GameScene {
        let (width_table, walk) = fixtures();
        GameScene::load_floor(
            include_bytes!("../tests/fixtures/maplv1.bin"),
            3 << 5,
            10 << 5,
            &[],
            &width_table,
            &walk,
            synth_images(icons),
            vec![None; 56],
        )
        .unwrap()
    }

    /// 【BUG-030】金币弹字（spawnPopup kind 4，a.java:9706 `popupX = 240`）右缘锚**跟随金币列**：
    /// classic 240 = 金币数值右锚 237 + 3；modern 金币列贴右（`screen_w − 3`）⇒ 弹字须同步贴右。
    /// 判据 = 弹字首位裁剪 x − 金币数值右锚，两预设相等（旧字面量 240 ⇒ modern 偏左 screen_w − 240）。
    #[test]
    fn gold_popup_tracks_gold_column() {
        let icons = synthetic_sptarm();
        let offset = |layout: crate::layout::Layout| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout.clone());
            let dw = sc.images.popup.digit_strips[0].width / 11;
            sc.popups.spawn(4, 1, 0, 0, sc.view.cam_x, sc.view.cam_y, dw);
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let mut g = SoftGraphics::new(&mut img);
            sc.popups.draw(&mut g, &sc.images.popup);
            let first = g.ops.iter().find(|o| o.starts_with("setClip(")).expect("金币弹字裁剪");
            op_xy(first).unwrap().0 - sc.layout.status_gold_num_x
        };
        let classic = offset(crate::layout::Layout::classic());
        for (gw, gh) in [(12, 16), (13, 13), (18, 18)] {
            assert_eq!(
                offset(crate::layout::Layout::modern(gw, gh)),
                classic,
                "modern {gw}×{gh}：金币弹字须对齐金币列（与 classic 同相对位）"
            );
        }
    }

    /// `setClip(x,y,w,h)` 四元解析（reset 臂 `setClip(0,0,画布)` 同格式）。
    fn clip4(o: &str) -> Option<(i32, i32, i32, i32)> {
        let a = o.strip_prefix("setClip(")?.strip_suffix(')')?;
        let v: Vec<i32> = a.split(',').filter_map(|t| t.trim().parse().ok()).collect();
        (v.len() == 4).then(|| (v[0], v[1], v[2], v[3]))
    }

    /// 【BUG-031①】walkPhase==2 观察形态四角箭头（a.java:5964-5967：`(5−bob,148)` `(109,25−bob)`
    /// `(109,viewBottom−29+bob)` `(215+bob,148)`）：148 = 160−12、109 = 120−11、215 = 240−25 是屏心/屏宽
    /// 折叠字面量，同块下臂早随 view_bottom（BUG-029 同型）⇒ modern 须随 center_y / center_x / screen_w。
    #[test]
    fn soul_view_corner_arrows_frame_the_screen() {
        let icons = synthetic_sptarm();
        for layout in [
            crate::layout::Layout::classic(),
            crate::layout::Layout::modern(12, 16),
            crate::layout::Layout::modern(13, 13),
            crate::layout::Layout::modern(18, 18),
        ] {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout.clone());
            sc.walk_phase = 2;
            sc.view_locked = false;
            let bob = sc.bob_offset;
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let ops = {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint_player(&mut g, sc.view.cam_x, sc.view.cam_y + sc.layout.view_top_y);
                g.ops.clone()
            };
            let arrows: Vec<(i32, i32)> =
                ops.iter().filter_map(|o| clip4(o)).filter(|c| (c.2, c.3) == (22, 24)).map(|c| (c.0, c.1)).collect();
            let (cx, cy, vb, w) = (layout.center_x, layout.center_y, sc.view_bottom, layout.screen_w);
            assert_eq!(
                arrows,
                vec![(5 - bob, cy - 12), (cx - 11, 25 - bob), (cx - 11, vb - 29 + bob), (w - 25 + bob, cy - 12)],
                "{}×{}：四角箭头须框住画面（左/右屏心 y、上/下屏心 x、右贴右缘）",
                layout.screen_w,
                layout.screen_h
            );
        }
    }

    /// 【BUG-031②】炼丹（mode 9，a.java:2696-2722）：框 / 花费行 / 高亮行已随屏居中，三行属性图标 x 97、
    /// 加值 x 142（= 高亮行 x 62 + 35 / + 80）仍是 classic 字面量 ⇒ modern 落到框外。判据：高亮行之后的
    /// 全部绘制 op 相对高亮行左上的坐标序列，modern 与 classic 逐项相等（纯平移）。
    #[test]
    fn alchemy_rows_translate_with_highlight_row() {
        let icons = synthetic_sptarm();
        let rel = |layout: crate::layout::Layout| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout.clone());
            sc.game_mode = GameMode::Alchemy;
            sc.alchemy_cursor = 0;
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let ops = {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                g.ops.clone()
            };
            let hi = ops
                .iter()
                .position(|o| o.starts_with("fillRect(") && o.contains(",115,16,"))
                .expect("高亮行 fillRect(x,y,115,16)");
            let (rx, ry) = op_xy(&ops[hi]).unwrap();
            ops[hi + 1..]
                .iter()
                .filter(|o| !o.starts_with("setClip(0,0,")) // 整画布 reset 臂不属行内容
                .filter_map(|o| {
                    let (x, y) = op_xy(o)?;
                    Some((o.split('(').next().unwrap_or("").to_string(), x - rx, y - ry))
                })
                .collect::<Vec<_>>()
        };
        let classic = rel(crate::layout::Layout::classic());
        assert!(classic.len() >= 6, "前置：三行图标+加值均有绘制 op：{classic:?}");
        for (gw, gh) in [(12, 16), (13, 13), (18, 18)] {
            assert_eq!(rel(crate::layout::Layout::modern(gw, gh)), classic, "modern {gw}×{gh}：炼丹行内容须随高亮行平移");
        }
    }

    /// 【BUG-031③】敌人图鉴（mode 5，a.java:2497-2595）：面板已随屏居中（modern 面板宽 = 屏宽 − 32），
    /// 翻页箭头 x 112（= 120 − 8 屏心折叠）与预测伤害列 `(240 + 面板宽 >> 1) − 75`（SCREEN_W 常量）未随动。
    /// 判据：箭头相对面板中线、伤害图标相对面板右缘的 x，modern 与 classic 相等。
    #[test]
    fn encyclopedia_arrows_and_damage_column_follow_panel() {
        let icons = synthetic_sptarm();
        let probe = |layout: crate::layout::Layout| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout.clone());
            sc.game_mode = GameMode::EnemyInfo;
            sc.images.ui[16] = solid(17, 9); // 预测伤害图标：独有尺寸便于定位
            sc.encyc_panel_w = sc.layout.encyc_panel_w;
            sc.encyc_types = vec![41, 42, 43];
            sc.encyc_damage = vec![10, 20, -1];
            sc.encyc_count = 3;
            sc.encyc_rows_visible = 1;
            sc.encyc_panel_h = 72;
            sc.encyc_scroll_top = 1; // 中间页：上下翻页箭头都画
            sc.encyc_scroll_bottom = 2;
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let ops = {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                g.ops.clone()
            };
            let bx = (layout.screen_w - sc.encyc_panel_w) >> 1;
            let (mid, right) = (bx + (sc.encyc_panel_w >> 1), bx + sc.encyc_panel_w);
            let arrows: Vec<i32> =
                ops.iter().filter_map(|o| clip4(o)).filter(|c| (c.2, c.3) == (15, 17)).map(|c| c.0 - mid).collect();
            let dmg: Vec<i32> =
                ops.iter().filter(|o| o.starts_with("drawImage(17x9,")).filter_map(|o| op_xy(o)).map(|p| p.0 - right).collect();
            (arrows, dmg)
        };
        let classic = probe(crate::layout::Layout::classic());
        assert_eq!(classic, (vec![-8, -8], vec![-87]), "classic：箭头 112 = 中线 120 − 8；伤害图标 149 − 12 = 右缘 224 − 87");
        for (gw, gh) in [(12, 16), (13, 13), (18, 18)] {
            assert_eq!(probe(crate::layout::Layout::modern(gw, gh)), classic, "modern {gw}×{gh}：箭头随面板中线、伤害列随面板右缘");
        }
    }

    /// 【BUG-031④】记录浏览（mode 12，a.java:2905-2914）：页码 x 120、双箭头 x `60 − bob` / `160 + bob` 为屏心
    /// 派生（Layout 早有 center_x / arrow_pair_left_x / arrow_pair_right_x，标题页已用）⇒ modern 须随屏心。
    /// 判据：页码主色串与两箭头的 x 相对 center_x，modern 与 classic 相等（y 见 BUG-032）。
    #[test]
    fn record_browser_indicator_and_arrows_follow_center() {
        let icons = synthetic_sptarm();
        let probe = |layout: crate::layout::Layout| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout.clone());
            sc.game_mode = GameMode::RecordBrowser;
            sc.images.ui[14] = solid(20, 13); // 箭头图：独有尺寸便于定位
            sc.journal = vec![(0, 0), (0, 1), (0, 2)];
            sc.record_page = 1; // 中间页：左右箭头都画
            sc.page_indicator = "2 / 3".into();
            sc.overlay_active = false;
            let bob = sc.bob_offset;
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let ops = {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                g.ops.clone()
            };
            let cx = layout.center_x;
            let text = ops.iter().filter(|o| o.starts_with("drawString(")).filter_map(|o| op_xy_after_text(o)).last().map(|p| p.0 - cx);
            let arrows: Vec<i32> = ops
                .iter()
                .filter(|o| o.starts_with("drawImage(20x13,") || o.starts_with("drawImageT(20x13,"))
                .filter_map(|o| op_xy(o))
                .map(|p| p.0 - cx)
                .collect();
            (text, arrows, bob)
        };
        let classic = probe(crate::layout::Layout::classic());
        let bob = classic.2;
        assert_eq!((classic.0, classic.1.clone()), (Some(0), vec![-60 - bob, 40 + bob]), "classic：页码 120、箭头 60−bob / 160+bob");
        for (gw, gh) in [(12, 16), (13, 13), (18, 18)] {
            assert_eq!(probe(crate::layout::Layout::modern(gw, gh)), classic, "modern {gw}×{gh}：页码与箭头须随屏心");
        }
    }

    /// 【BUG-032】记录浏览双箭头 y（a.java:2908 `var46 += fontHeight + 4 - 13 >> 1`；javap 5242-5255
    /// `iload_2 … iconst_1 ishr iadd`）= 页码行底 + ((fh + 4 − 13) >> 1)，非 (行底 + …) >> 1。两预设同式。
    #[test]
    fn record_browser_arrow_y_sits_beside_indicator() {
        let icons = synthetic_sptarm();
        for layout in [crate::layout::Layout::classic(), crate::layout::Layout::modern(12, 16)] {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout.clone());
            sc.game_mode = GameMode::RecordBrowser;
            sc.images.ui[14] = solid(20, 13);
            sc.journal = vec![(0, 0), (0, 1), (0, 2)];
            sc.record_page = 1;
            sc.overlay_active = false;
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let ops = {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                g.ops.clone()
            };
            let fh = crate::paint::paint_font().height;
            let text_bottom = layout.screen_h - fh - 4;
            let ys: Vec<i32> = ops
                .iter()
                .filter(|o| o.starts_with("drawImage(20x13,") || o.starts_with("drawImageT(20x13,"))
                .filter_map(|o| op_xy(o))
                .map(|p| p.1)
                .collect();
            let y = text_bottom + ((fh + 4 - 13) >> 1);
            assert_eq!(ys, vec![y, y], "{}×{}：箭头 y = 页码行底 + ((fh+4−13)>>1)", layout.screen_w, layout.screen_h);
        }
    }

    /// `drawString(text,x,y,anchor)` 的 (x,y)（跳过首个文本参数）。
    fn op_xy_after_text(o: &str) -> Option<(i32, i32)> {
        let args = o.strip_prefix("drawString(")?.strip_suffix(')')?;
        let nums: Vec<i32> = args.rsplitn(4, ',').filter_map(|t| t.trim().parse().ok()).collect();
        // rsplitn 逆序：anchor, y, x
        (nums.len() == 3).then(|| (nums[2], nums[1]))
    }

    /// STAGE-B-002 ② 消费面：paint_status_bar 锚位随 `self.layout`（modern 贴右/
    /// 黑填充界），classic 零偏离（既有像素/ops 测全绿护航）。
    #[test]
    fn status_bar_anchors_follow_layout() {
        let icons = synthetic_sptarm();
        let norm = |o: &String| o.replace(", ", ",");
        // classic：填充 (65,0,175,18)——240-65=175，原值
        let mut sc = scene_with(&icons);
        let mut img = ArgbImage::create(crate::layout::SCREEN_W, crate::layout::SCREEN_H);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint_status_bar(&mut g);
            let ops: Vec<String> = g.ops.iter().map(norm).collect();
            assert!(
                ops.iter().any(|o| o.contains("fillRect(65,0,175,18,")),
                "classic 原值填充：{ops:?}"
            );
        }
        // modern(13,13)=416：填充 (65,0,351,18)——416-65=351；钥匙首槽 clip x=241（贴右）
        let mut sc = scene_with(&icons);
        sc.layout = crate::layout::Layout::modern(13, 13);
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint_status_bar(&mut g);
            let ops: Vec<String> = g.ops.iter().map(norm).collect();
            assert!(
                ops.iter().any(|o| o.contains("fillRect(65,0,351,18,")),
                "modern 贴右填充宽随屏：{ops:?}"
            );
            assert!(
                ops.iter().any(|o| o.contains("setClip(241,0,18,16)")),
                "modern 钥匙首槽贴右 x=241：{ops:?}"
            );
        }
    }

    /// 返修判据（用户实机三破绽 2026-10-18）：modern 渲染**分布**——瓦片绘制占
    /// [view_top, view_bottom) 全宽、HUD/软键落底带。钉"绘制坐标系整体换轨"
    /// （002 首版只连了 Layout 字段值、draw 落点仍是旧坐标系 ⇒ 地图只画左上角块、
    /// 属性浮屏中、钥/对话框各自为政）。
    #[test]
    fn modern_render_distribution_spans_full_canvas() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let mut img = ArgbImage::create(416, 416);
        let ops;
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
            ops = g.ops.clone();
        }
        // op 坐标抽取（"name(x,y,…)" → (x,y)）
        let coords: Vec<(i32, i32)> = ops
            .iter()
            .filter_map(|o| {
                let args = o.split('(').nth(1)?.split(')').next()?;
                let mut it = args.split(',');
                let x: i32 = it.next()?.trim().parse().ok()?;
                let y: i32 = it.next()?.trim().parse().ok()?;
                Some((x, y))
            })
            .collect();
        // ① 地图瓦片（clip 16×16 专属模式）横跨全宽且填满可视窗：
        //    破绽现状=旧视口 15 列×~16 行 ≈240 片、max_x≈224；换轨后 26 列 ≥380 片、max_x≥400
        let tiles: Vec<(i32, i32)> = ops
            .iter()
            .filter(|o| o.starts_with("setClip(") && o.ends_with(",16,16)"))
            .filter_map(|o| {
                let args = o.split('(').nth(1)?.split(')').next()?;
                let mut it = args.split(',');
                let x: i32 = it.next()?.trim().parse().ok()?;
                let y: i32 = it.next()?.trim().parse().ok()?;
                Some((x, y))
            })
            .collect();
        let tile_max_x = tiles.iter().map(|t| t.0).max().unwrap_or(0);
        assert!(tile_max_x >= 380, "modern 瓦片须横跨画布全宽（现 tile_max_x={tile_max_x}）");
        assert!(
            tiles.len() >= 380,
            "modern 瓦片须填满可视窗（现 {} 片，旧视口窗≈240）",
            tiles.len()
        );
        // ② HUD 落底带（≥366）：现破绽=属性浮屏中（旧 hud_base=270）
        let bottom_max_y = coords.iter().map(|c| c.1).max().unwrap_or(0);
        assert!(bottom_max_y >= 366, "modern HUD/软键须落底带（现 bottom_max_y={bottom_max_y}）");
    }

    /// 返修判据补：modern 画布绘制零大幅越界（坐标系整体换轨后无旧系残留出框）。
    #[test]
    fn modern_render_no_draw_outside_canvas() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
            for o in &g.ops {
                let Some(args) = o.split('(').nth(1).and_then(|s| s.split(')').next()) else {
                    continue;
                };
                let nums: Vec<i32> = args
                    .split(',')
                    .filter_map(|t| t.trim().trim_start_matches('#').parse().ok())
                    .collect();
                if let (Some(&x), Some(&y)) = (nums.first(), nums.get(1)) {
                    assert!(x < 416 + 64 && y < 416 + 64, "越界绘制 {o}（坐标系残留）");
                }
            }
        }
    }

    /// 残项清账判据（lead 派单 2026-10-18，探针三选一结论入册）：
    /// - mapbg 77×320 **全不透明**（探针实证）⇒ 排除"透明透出画布黑"；
    /// - 黑区像素 **alpha=0（0x00000000 未绘制面）**（自验图解码实证）⇒ 排除"黑瓦片/画出来的黑"；
    /// - 故来源=**未绘制面**（mode 3 链在地图下缘悬垂带没垫任何层）。
    ///   本判据钉"无未解释黑区"：modern 全链后视口带内未绘制像素 = 0。
    #[test]
    fn modern_render_no_unexplained_black() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let packed = std::fs::read(repo.join("assets/raw/mapbg")).unwrap();
        let bg = game_data::PackedPng::parse(&packed, 1)
            .unwrap()
            .images
            .iter()
            .map(|s| ArgbImage::from_decoded_png(game_data::decode_png(s.bytes).unwrap()))
            .next()
            .unwrap();
        // mapbg 无透明（来源分类断言①：排除透出候选）
        assert!(
            bg.argb.iter().all(|p| p >> 24 == 0xFF),
            "mapbg 须全不透明（若有透明则黑区另有透出路径）"
        );
        let mut sc = scene_with(&synthetic_sptarm());
        sc.images.backdrop = bg;
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
        }
        // 无未解释黑区（来源分类断言②：alpha=0 未绘制面须为 0）——视口带 [20,366)
        let mut undrawn = 0usize;
        for y in 20..366 {
            for x in 0..416 {
                if img.argb[y * 416 + x] == 0 {
                    undrawn += 1;
                }
            }
        }
        assert_eq!(undrawn, 0, "modern 视口带内未绘制像素数（黑区根因面）");
    }

    /// v6 判据（用户"背景颜色很奇怪/过渡不自然"）：真背景图**边缘钳制延展**——
    /// 顶带像素=图顶行色、底带=图底行色；镜像反转（沙地翻上屏顶/天空翻下底带）即红。
    #[test]
    fn modern_backdrop_edge_clamp_no_mirror_flip() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let packed = std::fs::read(repo.join("assets/raw/mapbg")).unwrap();
        let bg = game_data::PackedPng::parse(&packed, 1)
            .unwrap()
            .images
            .iter()
            .map(|s| ArgbImage::from_decoded_png(game_data::decode_png(s.bytes).unwrap()))
            .next()
            .unwrap();
        let bw = bg.width as usize;
        let bh = bg.height as usize;
        let (top_row, bottom_row) = (bg.argb[0], bg.argb[(bh - 1) * bw]);
        let mut sc = scene_with(&synthetic_sptarm());
        sc.images.backdrop = bg;
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        sc.backdrop_scroll = -1;
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint_parallax(&mut g, true);
        }
        let top_px = img.argb[4 * 416 + 40]; // 顶带采样
        let bottom_px = img.argb[411 * 416 + 40]; // 底带采样
        assert_eq!(top_px, top_row & 0x00FF_FFFF | 0xFF00_0000, "顶带=图顶行色（钳制延展）");
        assert_eq!(bottom_px, bottom_row & 0x00FF_FFFF | 0xFF00_0000, "底带=图底行色（钳制延展）");
    }

    /// modern 双缺陷判据（lead classic 对照定案 2026-10-18）：
    /// ① 顶栏金币图标必须绘出（像素级：图标位≠状态栏底色——裁剪复位常量化会把
    ///    绘制裁掉但 ops 仍在，故必须像素判）；② 对话文本必须绘出（drawString 在场）。
    #[test]
    fn modern_topbar_gold_icon_and_dialog_text_drawn() {
        let icons = synthetic_sptarm();
        // ① 金币图标像素判（modern 状态栏：图标位 (gold_icon_x+8, 8) ≠ STATUS_FILL）
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
        }
        let gx = sc.layout.status_gold_icon_x as usize;
        let icon_px = img.argb[8 * 416 + gx + 8];
        assert_ne!(
            icon_px, STATUS_FILL,
            "金币图标须绘出（现像素={icon_px:#x} = 底色 ⇒ 被裁剪复位裁掉）"
        );
        // ② 对话文本在场（对话态绘制含 drawString）
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        sc.game_mode = GameMode::Dialog;
        sc.open_dialog_page(0);
        let mut img = ArgbImage::create(416, 416);
        let ops;
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
            ops = g.ops.clone();
        }
        assert!(
            ops.iter().any(|o| o.starts_with("drawString") || o.starts_with("drawSubstring")),
            "对话文本须绘出（drawString/drawSubstring 在场）：{:?}",
            ops.iter().filter(|o| o.contains("tring")).collect::<Vec<_>>()
        );
        // 【v7 判据】对话框体底色=HUD_FILL（用户实报"框底色被背景覆盖"——
        // 裁剪复位常量漏网把 body 填充裁掉、露背景）
        let body = img.argb[404 * 416 + 260];
        assert_eq!(body, HUD_FILL | 0xFF00_0000, "对话框体底色须=HUD_FILL（现={body:#010x}=背景色透出）");
    }

    /// P0 补修判据（2026-10-20 存档入口漏网）：loadFloorSave 直入 26 格楼层——
    /// tile 专色全绘（半绘=视图重建残留 240-era 范围）。真修点=load_floor_from_save
    /// 路径（loadSlotTable→loadFloorSave→视图重建）。
    #[test]
    fn modern_save_load_tiles_cover_full_view() {
        let (wide, high) = (26i32, 26i32);
        let mut maplv = Vec::new();
        maplv.extend_from_slice(&((wide * 2) as u16).to_le_bytes());
        maplv.extend_from_slice(&((high * 2) as u16).to_le_bytes());
        let gl = (wide * high * 4) as usize;
        maplv.extend_from_slice(&vec![1u8; gl]);
        maplv.extend_from_slice(&vec![0u8; gl]);
        // FloorSave：preset-floor2-mov 的 MOT_L0 真 blob（save 形状=Java loadFloorSave 布局）
        let stores = crate::save::parse_preset(include_str!("../tests/fixtures/preset-floor2-mov.preset"));
        let blob = stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).unwrap().1.clone();
        let save = crate::save::parse_floor_save(&blob, 0).unwrap();
        let icons = synthetic_sptarm();
        let (wt, walk) = fixtures();
        let mut sc = load_floor_from_save(
            &maplv,
            &[],
            &wt,
            &walk,
            synth_images(&icons),
            vec![None; 56],
            &save,
        )
        .unwrap();
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let mut tileset = ArgbImage::create(64, 64);
        tileset.argb.fill(0xFF00_FF00);
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &tileset);
        }
        let w = 416usize;
        let mut right_tiles = 0;
        for x in 300..410 {
            if img.argb[200 * w + x] == 0xFF00_FF00 {
                right_tiles += 1;
            }
        }
        assert!(
            right_tiles > 20,
            "存档直入后右半须绘出 tile（现 {right_tiles}/110 专色）——半绘 artifact"
        );
    }

    /// P0 回归判据（2026-10-20 modern 地形半绘·真修点路径）：modern 换层加载
    /// 26 格宽楼层后 tile 层须绘满视口（右半空白=半绘 artifact）——走
    /// load_floor_data 重建视图路径（from_maplv classic 初始+布局换轨修复点）。
    #[test]
    fn modern_floor_switch_tiles_cover_full_view() {
        // 26 格宽合成 maplv（头 u16LE=52；tile 全 1=可见地形）
        let (wide, high) = (26i32, 26i32);
        let mut maplv = Vec::new();
        maplv.extend_from_slice(&((wide * 2) as u16).to_le_bytes());
        maplv.extend_from_slice(&((high * 2) as u16).to_le_bytes());
        let gl = (wide * high * 4) as usize;
        // 地形段全置 tile 1（绘制索引=行距 wide·2、列距 1——全段覆盖免索引形错）
        maplv.extend_from_slice(&vec![1u8; gl]);
        maplv.extend_from_slice(&vec![0u8; gl]);
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        sc.load_floor_data(&maplv, 10 << 5, 10 << 5, &[], 3).unwrap();
        // tileset=专色绿（tile 专色判据——背板/其他层误计防）
        let mut tileset = ArgbImage::create(64, 64);
        tileset.argb.fill(0xFF00_FF00);
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &tileset);
        }
        // 右半视口（x=300..410）中带行（y=200）须有 tile 专色像素
        let w = 416usize;
        let mut right_tiles = 0;
        for x in 300..410 {
            if img.argb[200 * w + x] == 0xFF00_FF00 {
                right_tiles += 1;
            }
        }
        assert!(
            right_tiles > 20,
            "modern 换层后右半须绘出 tile（现 {right_tiles}/110 专色）——半绘 artifact"
        );
    }

    /// BUG-022 判据（用户实机 2026-10-20"扩大分辨率后，切屏时，切换动画是按
    /// 传统分辨率绘制的，没有覆盖全屏"）：换层遮幅**全黑帧**（wipe=4，格 8px
    /// 无缝）须覆盖整个画布——旧硬编码 40 行×30 列（=240×320 传统分辨率）只盖
    /// 左上角。双臂：modern 416×416=主判据；classic 240×320=零漂移锚（同判据
    /// 恒过=网格仍为 30×40 原值）。
    #[test]
    fn wipe_full_black_covers_whole_canvas() {
        for (tag, layout, w, h) in [
            ("classic", crate::layout::Layout::classic(), 240i32, 320i32),
            ("modern", crate::layout::Layout::modern(13, 13), 416, 416),
        ] {
            let icons = synthetic_sptarm();
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout);
            // 全黑帧：closing 相位推进 3→4（size=8/off=0 无缝格）；
            // wipe=4 不触 swap_floor（仅 wipe>4 才换层）。
            sc.transitioning = true;
            sc.wipe_closing = true;
            sc.wipe = 3;
            let mut img = ArgbImage::create(w, h);
            {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
            }
            for (i, p) in img.argb.iter().enumerate() {
                assert_eq!(
                    *p, 0xFF00_0000,
                    "[{tag}] 遮幅全黑帧像素 {i} 未覆盖（旧网格止步 240×320）"
                );
            }
        }
    }

    /// BUG-022 连带判据：遮幅点阵**中段相位**（wipe=2，格 4px/off=2）同样须铺满
    /// 画布右下角——旧网格点阵末格止步 (234,314)，右下角格永不落点。
    #[test]
    fn modern_wipe_dither_spans_whole_canvas() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        sc.transitioning = true;
        sc.wipe_closing = true;
        sc.wipe = 1; // closing 相位 1→2（size=4/off=2）
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint_wipe_tail(&mut g);
        }
        // 末格 (51,51) 起点 off+51*8=410，覆盖 410..414（中段相位右下角格）
        assert_eq!(
            img.argb[412 * 416 + 412], 0xFF00_0000,
            "modern 中段点阵右下角格未落点（旧网格止步 240×320）"
        );
    }

    /// STAGE-B-004a ①c 判据（对话框底边距=游戏视图底黑边）：modern 无对话态下
    /// 底行面板两侧不得留黑——背景带须补铺到底（黑边=未补铺 artifact）。
    #[test]
    fn modern_bottom_band_sides_never_black() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
        }
        let w = 416usize;
        for (x, y) in [(5, 415), (410, 415), (5, 400), (410, 400)] {
            let p = img.argb[y * w + x];
            assert_ne!(
                p & 0x00FF_FFFF,
                0,
                "({x},{y}) 不得黑（游戏视图底黑边 artifact，现={p:#010x}）"
            );
        }
    }

    /// 底部区口径修正判据（lead 设计口径级修正 2026-10-18：**经典尺寸固定+整体居中**，
    /// 非随屏拉伸）：面板宽 == 经典常量、居中偏移 = (screen_w−经典面板宽)/2、
    /// classic 下偏移回代原值（0）。
    #[test]
    fn bottom_panel_classic_size_centered() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let dx = sc.bottom_panel_dx();
        assert_eq!(dx, (416 - 240) / 2, "modern 面板居中偏移 =（屏宽−经典面板宽 240）/2");
        sc.game_mode = GameMode::Dialog;
        sc.open_dialog_page(0);
        let mut img = ArgbImage::create(416, 416);
        let ops;
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
            ops = g.ops.clone();
        }
        // 对话框填充宽 == 经典常量 149、起点 == 80+dx（面板内相对布局=classic 同构）
        assert!(
            ops.iter()
                .any(|o| o.contains(&format!("fillRect({},", 80 + dx)) && o.contains(",149,")),
            "对话框填充宽须为经典常量 149（拉伸=违口径）：{:?}",
            ops.iter().filter(|o| o.contains("149") || o.contains("fillRect(80")).collect::<Vec<_>>()
        );
        // classic 回代：偏移 0（240 宽面板在 240 屏=原值）
        let mut sc2 = scene_with(&icons);
        assert_eq!(sc2.bottom_panel_dx(), 0, "classic 面板偏移回代原值 0（零偏离）");
        let _ = &mut sc2;
    }

    /// 底部区像素体检（用户复验 2026-10-18"渲染还是不对"）：量对话文本包围盒的
    /// **实测数**，classic vs modern 对照——文本字号/位置/框体关系拿数说话。
    #[test]
    // 【2026-10-19】依赖 analysis/b002-*-v7.png（可再生工作区不入库）——
    // clean 树/CI 必红（STAGE-B-002 v7 实验探针污染正式测试）。标 ignore
    // 待自包含化（生成 PNG 再探）或移实验区。lead 处置。
    #[ignore = "实验探针：依赖 analysis/ 探针 PNG（可再生工作区），自包含化后转正"]
    fn probe_bottom_region_pixel_metrics() {
        let icons = synthetic_sptarm();
        for (label, layout) in [
            ("classic", crate::layout::Layout::classic()),
            ("modern", crate::layout::Layout::modern(13, 13)),
        ] {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout);
            sc.game_mode = GameMode::Dialog;
            sc.open_dialog_page(0);
            let (w, h) = (sc.layout.screen_w as usize, sc.layout.screen_h as usize);
            let mut img = ArgbImage::create(w as i32, h as i32);
            {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
            }
            let dx = sc.bottom_panel_dx();
            // 文本行带（block_h=40 ⇒ y0=h-40+12 起）内非底色像素包围盒
            let band_y0 = h as i32 - 40 + 12;
            let (mut x_min, mut x_max, mut y_min, mut y_max) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
            for y in band_y0..(h as i32 - 2) {
                for x in 0..w as i32 {
                    let p = img.argb[(y as usize) * w + x as usize];
                    if p != 0 && p != HUD_FILL && p != STATUS_FILL {
                        x_min = x_min.min(x);
                        x_max = x_max.max(x);
                        y_min = y_min.min(y);
                        y_max = y_max.max(y);
                    }
                }
            }
            eprintln!(
                "PROBE {label} dx={dx} 文本带包围盒 x[{x_min},{x_max}] y[{y_min},{y_max}] 宽={} 高={}",
                x_max - x_min + 1,
                y_max - y_min + 1
            );
        }
        // 真图实测：解码两预设自验图，量底部带逐行内容列段（非底色像素游程）
        // +【v7】对话框体打点：body/顶条/左面板三点采样——框内应=框底色，
        // 若=沙色即"对话框底色被背景覆盖/透印"实证（用户 2026-10-18 质询）
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for (label, file) in [
            ("classic", "analysis/b002-classic-v7.png"),
            ("modern", "analysis/b002-modern-v7.png"),
        ] {
            let bytes = std::fs::read(repo.join(file)).unwrap();
            let dec = game_data::decode_png(&bytes).unwrap();
            let (w, h) = (dec.width as usize, dec.height as usize);
            let px = |x: usize, y: usize| dec.argb[y * w + x];
            eprintln!(
                "PROBE shot-{label} {w}x{h} 框体打点 body({},{})={:#010x} topstrip({},{})={:#010x} leftpanel({},{})={:#010x}（HUD_FILL={:#010x}）",
                w * 5 / 8, h - 12, px(w * 5 / 8, h - 12),
                w / 2, h - 34, px(w / 2, h - 34),
                w / 6, h - 12, px(w / 6, h - 12),
                HUD_FILL | 0xFF00_0000
            );
            for y in (h - 50)..h {
                let mut runs: Vec<(usize, usize)> = Vec::new();
                let mut start = usize::MAX;
                for x in 0..w {
                    let p = dec.argb[y * w + x];
                    let bg = p == 0 || p == HUD_FILL || p == STATUS_FILL;
                    if !bg && start == usize::MAX {
                        start = x;
                    }
                    if bg && start != usize::MAX {
                        runs.push((start, x - 1));
                        start = usize::MAX;
                    }
                }
                if start != usize::MAX {
                    runs.push((start, w - 1));
                }
                if !runs.is_empty() && y % 2 == 0 {
                    eprintln!("PROBE shot-{label} y={y} runs={runs:?}");
                }
            }
        }
        // 标题"大闹天宫"屏对齐实测（用户 2026-10-19"还是没有对齐"）：
        // classic vs modern 内容块边界对照
        for (label, file) in [
            ("title-classic", "analysis/loading-280.png"),
            ("title-modern", "analysis/title-modern.png"),
        ] {
            let bytes = std::fs::read(repo.join(file)).unwrap();
            let dec = game_data::decode_png(&bytes).unwrap();
            let (w, h) = (dec.width as usize, dec.height as usize);
            for y in (0..h).step_by(16) {
                let mut runs: Vec<(usize, usize)> = Vec::new();
                let mut start = usize::MAX;
                for x in 0..w {
                    let p = dec.argb[y * w + x];
                    let (r, g, b) = ((p >> 16) & 0xFF, (p >> 8) & 0xFF, p & 0xFF);
                    let bg = p == 0 || (b > 150 && r < 120 && g < 130); // 蓝底/透明=背景
                    if !bg && start == usize::MAX {
                        start = x;
                    }
                    if bg && start != usize::MAX {
                        runs.push((start, x - 1));
                        start = usize::MAX;
                    }
                }
                if start != usize::MAX {
                    runs.push((start, w - 1));
                }
                if !runs.is_empty() {
                    eprintln!("PROBE {label} y={y} runs={runs:?}");
                }
            }
        }
    }

    /// 底部区组合关系判据（lead"组合关系整体钉死"，v5 扩展：**层序也是组合关系**）：
    /// 同状态 classic vs modern 的底部带绘制**序列**（op 名+去平移坐标，保序）
    /// 逐项相等——坐标多重集之外再钉绘制顺序（谁盖谁）。
    #[test]
    fn bottom_composition_is_translate_only() {
        let icons = synthetic_sptarm();
        let coords = |layout: crate::layout::Layout, speaker: i32| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout);
            sc.game_mode = GameMode::Dialog;
            sc.open_dialog_page(0);
            sc.dialog_speaker = speaker; // open_dialog_page 会置 speaker——须在其后覆盖
            if speaker > 0 {
                sc.images.entity[speaker as usize] = Some(solid(36, 40)); // 立绘播种（分支必画）
            }
            let (w, h) = (sc.layout.screen_w as usize, sc.layout.screen_h as usize);
            let mut img = ArgbImage::create(w as i32, h as i32);
            let ops;
            {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                ops = g.ops.clone();
            }
            let dx = sc.bottom_panel_dx();
            let dy = h as i32 - 320;
            // 保序序列：(op 名, x−dx, y−dy)——层序也入判据（v5）；
            // 背景层（drawImageT 镜像行 / 列宽 77 边缘填充 / 画布自适应列）不计
            // ——组合关系指面板构图
            ops
                .iter()
                .filter(|o| !o.starts_with("drawImageT"))
                .filter(|o| !(o.starts_with("fillRect(") && o.contains(",77,")))
                .filter_map(|o| {
                    let (x, y) = op_xy(o)?;
                    (y >= h as i32 - 45).then(|| {
                        let name = o.split('(').next().unwrap_or("").to_string();
                        (name, x - dx, y - dy)
                    })
                })
                .collect::<Vec<_>>()
        };
        // speaker>0（立绘分支）、speaker==72（牛魔王定尺剪裁臂 a.java:2852）与 speaker==0
        // （面孔分支）三路都钉（坐标+层序）。72 臂漏钉 ⇒ 字面量 y=275 漏网（BUG-029）。
        for speaker in [0, 1, 72] {
            let classic = coords(crate::layout::Layout::classic(), speaker);
            let modern = coords(crate::layout::Layout::modern(13, 13), speaker);
            assert_eq!(
                modern, classic,
                "底部区组合关系须纯平移且层序不变（speaker={speaker}）"
            );
        }
    }

    /// 观察形态判据（lead 裁决 (a) 2026-10-19 用户批准，STAGE-B-002 偏离点）：
    /// 空格观察形态（walkPhase==2 未锁定）modern **角色恒居中**（视口中心成像
    /// 出现角色=与基帧不同）；classic 保持原机制（角色留 cam+player 位、
    /// 视口中心与基帧同）——classic 零偏离。
    #[test]
    fn soul_view_character_pinned_center_modern_only() {
        let icons = synthetic_sptarm();
        let shot = |layout: crate::layout::Layout, walk: i32| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout);
            sc.view.player_px = 32;
            sc.view.player_py = 32;
            for i in 0..4 {
                sc.afterimage_x[i] = 32;
                sc.afterimage_y[i] = 32;
            }
            sc.walk_phase = walk;
            sc.view_locked = false;
            let (cx, cy) = (
                ((sc.layout.game_view_w - 41) >> 1) as usize,
                (sc.layout.view_top_y + sc.layout.game_view_h / 2) as usize,
            );
            let (w, h) = (sc.layout.screen_w as usize, sc.layout.screen_h as usize);
            let mut img = ArgbImage::create(w as i32, h as i32);
            let ops;
            {
                // 直驱 paint_player（cam 传参=真实调用方口径：view.cam+view_top）
                let mut g = SoftGraphics::new(&mut img);
                sc.paint_player(&mut g, sc.view.cam_x, sc.view.cam_y + sc.layout.view_top_y);
                ops = g.ops.clone();
            }
            if std::env::var("PROBE_SOUL").is_ok() {
                eprintln!("PROBE walk={walk} ops: {:?}", ops);
            }
            (img, cx, cy)
        };
        for (label, layout, expect_pin) in [
            ("classic", crate::layout::Layout::classic(), false),
            ("modern", crate::layout::Layout::modern(13, 13), true),
        ] {
            let (base, cx, cy) = shot(layout.clone(), 0);
            let (soul, _, _) = shot(layout, 2);
            let bw = base.width as usize;
            let mut diff = 0usize;
            for y in cy..cy + 30 {
                for x in cx..cx + 30 {
                    if base.argb[y * bw + x] != soul.argb[y * bw + x] {
                        diff += 1;
                    }
                }
            }
            if expect_pin {
                assert!(diff > 40, "{label}：观察态角色须钉视口中心（中心窗 diff={diff}）");
            } else {
                assert_eq!(diff, 0, "{label}：观察态原机制（角色留 cam+player 位，中心窗须与基帧同）");
            }
        }
    }

    /// 【task-14 场景级·布局纪律】菜单臂二属性分类判据：逐臂须满足其一——
    /// (A) 纯平移（内容 ops classic/modern 逐项恒等=origin 隐式平移）或
    /// (B) 相对布局保持（屏心偏移双预设同值，公式锚语义）。
    /// 双破=错位元素（违布局纪律）——37b53a4 双移回归即此判据揪出。
    /// 【分类表 2026-10-20】InGameMenu=公式锚(B)；EnemyInfo/ItemPanel 随测落定。
    #[test]
    fn menu_arms_layout_discipline() {
        let icons = synthetic_sptarm();
        /// 【分类表 2026-10-20】InGameMenu=公式锚(B) 已定；
        /// **EnemyInfo(paint_menu5)=混装**（c=94 m=-8）、**ItemPanel=混装**
        /// （c=-41 m=-2）——modern 侧近居中（-8/-2）但相对布局漂移=双属性皆破，
        /// 分类未定前不接线，列入收口余项（逐元素锚对账后定 A/B）。
        type Setup = fn(&mut GameScene);
        let arms: &[(&str, Setup)] = &[("InGameMenu", |sc| {
            sc.game_mode = GameMode::InGameMenu;
            sc.enter_menu4();
        })];
        let ops_of = |layout: crate::layout::Layout, setup: Setup| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout);
            setup(&mut sc);
            let (w, h) = (sc.layout.screen_w as usize, sc.layout.screen_h as usize);
            let mut img = ArgbImage::create(w as i32, h as i32);
            let ops;
            {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                ops = g.ops.clone();
            }
            (ops, w)
        };
        let content = |ops: Vec<String>| -> Vec<String> {
            ops.into_iter()
                .filter(|o| {
                    if o.starts_with("drawImageT") || o.contains("77x320") {
                        return false;
                    }
                    if o.starts_with("fillRect(") || o.starts_with("setClip(") {
                        let nums: Vec<i32> = o
                            .split('(')
                            .nth(1)
                            .and_then(|s| s.split(')').next())
                            .map(|a| {
                                a.split(',')
                                    .filter_map(|t| t.trim().trim_start_matches('#').parse().ok())
                                    .collect()
                            })
                            .unwrap_or_default();
                        if nums.len() >= 4 && (nums[2] >= 240 || nums[3] >= 320) {
                            return false;
                        }
                    }
                    true
                })
                .collect()
        };
        let center_of = |ops: &Vec<String>, w: i32| -> i32 {
            let (mut lo, mut hi) = (i32::MAX, i32::MIN);
            for o in ops {
                if let Some((x, _)) = op_xy(o) {
                    lo = lo.min(x);
                    hi = hi.max(x);
                }
            }
            (lo + hi) / 2 - w / 2
        };
        for (name, setup) in arms {
            let (c_ops, c_w) = ops_of(crate::layout::Layout::classic(), *setup);
            let (m_ops, m_w) = ops_of(crate::layout::Layout::modern(13, 13), *setup);
            let (c, m) = (content(c_ops), content(m_ops));
            let pure_translate = m == c;
            let (co, mo) = (center_of(&c, c_w as i32), center_of(&m, m_w as i32));
            assert!(
                pure_translate || co == mo,
                "{name} 须过布局纪律：纯平移（ops 恒等）或相对布局保持（屏心偏移同值 c={co} m={mo}）"
            );
        }
    }

    /// 【lead 数据vs渲染二面 2026-10-20·③】tile 层**逐层全扫**（55 maplv）：
    /// 画布内的数据格（v>0）必有对应绘制（setClip(x,y,16,16)）——数据有格未画
    /// =渲染面缺格（同族漏网）；数据真无格=该层设计如此（判据自动静默=合法）。
    /// 破缺层报 (层号, 缺格数) ⇒ ①数据面/②渲染面二选一定位。
    #[test]
    fn tiles_draw_all_data_cells_in_canvas_all_layers() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut bad: Vec<(u32, usize)> = Vec::new();
        for n in 0..55u32 {
            let Ok(bytes) = std::fs::read(repo.join(format!("assets/raw/maplv{n}"))) else {
                continue;
            };
            let Ok(mut v) = crate::game_view::GameView::from_maplv(&bytes, 0, 0, crate::game_view::ViewDims::classic()) else {
                continue;
            };
            v.set_layout_dims(416, 348, 416, 416);
            let mut img = ArgbImage::create(416, 416);
            let ops;
            {
                let mut g = SoftGraphics::new(&mut img);
                v.paint_tiles(&mut g, &solid(64, 64), 20);
                ops = g.ops.clone();
            }
            let drawn: std::collections::HashSet<(i32, i32)> = ops
                .iter()
                .filter(|o| o.starts_with("setClip(") && o.ends_with(",16,16)"))
                .filter_map(|o| op_xy(o))
                .collect();
            let mut miss = 0usize;
            for r in 0..(v.high as i32) * 2 {
                for c in 0..(v.wide as i32) * 2 {
                    if v.terrain_at(r, c) > 0 {
                        let (x, y) = (v.cam_x + c * 16, 20 + v.cam_y + r * 16);
                        if (0..416).contains(&x) && (20..366).contains(&y) && !drawn.contains(&(x, y)) {
                            miss += 1;
                        }
                    }
                }
            }
            if miss > 0 {
                bad.push((n, miss));
            }
            if std::env::var("PROBE_DATA_RIGHT").is_ok() {
                let (mut right, mut map_w) = (0i32, (v.wide as i32) * 32);
                for r in 0..(v.high as i32) * 2 {
                    for c in 0..(v.wide as i32) * 2 {
                        if v.terrain_at(r, c) > 0 {
                            right = right.max(c * 16 + 16);
                        }
                    }
                }
                eprintln!("PROBE maplv{n} 数据右缘={right}px 地图宽={map_w}px（右缘<地图宽=①数据边界）");
            }
        }
        assert!(
            bad.is_empty(),
            "②渲染面缺格层（数据有格未画=同族漏网）: {bad:?}；空表=①数据边界合法"
        );
    }

    /// 跳过 "77x320" 尺寸记号与 '#rrggbb' 色参（drawImage/drawImageTransformed 可用）。
    fn op_xy(o: &str) -> Option<(i32, i32)> {
        let args = o.split('(').nth(1)?.split(')').next()?;
        let mut nums = args
            .split(',')
            .filter_map(|t| t.trim().trim_start_matches('#').parse::<i32>().ok());
        Some((nums.next()?, nums.next()?))
    }

    /// 【2026-10-26 壳层呈现开关】mode 3 游戏画面：`softkey_bar_visible=false` 时绘制序列 =
    /// 默认序列**恰去掉一段 paintSoftkeyBar**（HUD 左右下角的软键底图 + 图标），其余 op
    /// （瓦片/实体/HUD/状态栏…）逐条不变；软键种类状态照常（壳据此画外置软键）。
    #[test]
    fn softkey_bar_switch_removes_exactly_the_softkey_block() {
        let icons = synthetic_sptarm();
        let layout = crate::layout::Layout::modern(12, 16);
        let render = |visible: bool| {
            let mut sc = scene_with(&icons);
            let mut l = layout.clone();
            l.softkey_bar_visible = visible;
            sc.apply_layout(l);
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let ops = {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                g.ops.clone()
            };
            (ops, sc.softkeys)
        };
        let (shown, keys) = render(true);
        let (hidden, keys_hidden) = render(false);
        assert_eq!(keys, (1, 3), "mode 3 软键种类");
        assert_eq!(keys_hidden, keys, "开关不动软键状态");
        // 期望删去的块 = 同尺寸画布上单独调 paintSoftkeyBar 的 op 串
        let block = {
            let sc = scene_with(&icons);
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let mut g = SoftGraphics::new(&mut img);
            crate::paint::m_034_softkeys(&mut g, &sc.images.ui[10], &sc.images.ui[11], 1, 3, &layout);
            g.ops.clone()
        };
        assert!(!block.is_empty());
        let at: Vec<usize> = (0..=shown.len().saturating_sub(block.len()))
            .filter(|&i| shown[i..i + block.len()] == block[..])
            .collect();
        assert_eq!(at.len(), 1, "默认帧含且仅含一段软键栏");
        let mut expect = shown.clone();
        expect.drain(at[0]..at[0] + block.len());
        assert_eq!(hidden, expect, "关闭 = 恰去掉软键栏，其余 op 逐条不变");
    }

    /// 第二轮返修判据（用户复验三破绽 2026-10-18）：
    /// ②背景铺满：视差列/行须盖满画布（现状=只盖左上固定区，右侧/下方大片黑）；
    /// ③HUD 内容组水平居中偏移 ≤ 12px；
    /// ④瓦片"右缘止"判定：绘制数 == 窗口内 v>0 格数（=数据边界则合法、
    ///   相机窗口缺格则断言破——区分二选一）。
    #[test]
    fn modern_render_background_fill_and_hud_center() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let dx = sc.bottom_panel_dx();
        let mut img = ArgbImage::create(416, 416);
        let ops;
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
            ops = g.ops.clone();
        }
        if std::env::var("PROBE_FILLS").is_ok() {
            for o in &ops {
                if o.starts_with("fillRect") || o.contains("Transformed") {
                    eprintln!("PROBE {o}");
                }
            }
        }
        // ② 背景覆盖：视差绘制（backdrop 77×320）x 跨度盖满 [0,416)、上下两行堆叠
        let bg: Vec<(i32, i32)> = ops
            .iter()
            .filter(|o| o.contains("77x320") || o.contains("drawImageTransformed"))
            .filter_map(|o| op_xy(o))
            .collect();
        // 【v5 判据扩展①】全画布未绘制像素=0（含底部带——面板两侧露背景非黑边）
        let mut undrawn_all = 0usize;
        for y in 0..416 {
            for x in 0..416 {
                if img.argb[y * 416 + x] == 0 {
                    undrawn_all += 1;
                }
            }
        }
        assert_eq!(undrawn_all, 0, "modern 全画布未绘制像素（底部带黑边=视差未铺满高）");
        let bg_max_x = bg.iter().map(|c| c.0).max().unwrap_or(-999);
        let bg_min_x = bg.iter().map(|c| c.0).min().unwrap_or(999);
        assert!(
            bg_max_x + 77 >= 416 && bg_min_x <= 0,
            "背景须铺满画布宽（现列起点 [{bg_min_x},{bg_max_x}]，末列+77 须 ≥416）"
        );
        assert!(bg.len() >= 5, "背景列须覆盖画布宽（每列一条基行；现 {} 条）", bg.len());
        // ③ HUD 内容组居中（底带**内容**x 跨度中心 vs 屏心，偏移 ≤12；
        //    框体=全宽设计（fill 宽≥200 / 边条 x<20 或 ≥400）不计入内容组）
        let hud: Vec<(i32, i32)> = ops
            .iter()
            .filter(|o| !o.starts_with("drawImageT")) // 背景层（画布自适应）不计
            .filter(|o| !(o.starts_with("fillRect(") && o.contains(",77,"))) // 背景边缘填充不计
            .filter(|o| op_xy(o).map(|c| c.1 > 368 && c.1 < 412).unwrap_or(false))
            .filter(|o| {
                let w = o
                    .split('(')
                    .nth(1)
                    .and_then(|s| s.split(')').next())
                    .map(|a| {
                        a.split(',')
                            .filter_map(|t| t.trim().trim_start_matches('#').parse::<i32>().ok())
                            .nth(2)
                            .unwrap_or(0)
                    })
                    .unwrap_or(0);
                w < 200
            })
            .filter_map(|o| op_xy(o))
            // 内容组域：排除框体四柱列（dx-42/dx/dx+176/dx+229 = 框角/框缘/右缘/右条
            // ——框体=面板构件非内容）与域外 x
            .filter(|c| {
                (26..330).contains(&c.0) && ![dx - 42, dx, dx + 176, dx + 229].contains(&c.0)
            })
            .collect();
        let (hmin, hmax) = (
            hud.iter().map(|c| c.0).min().unwrap_or(0),
            hud.iter().map(|c| c.0).max().unwrap_or(0),
        );
        let center = (hmin + hmax) / 2;
        assert!(
            (center - 208).abs() <= 12,
            "HUD 内容组水平居中（现 center={center} vs 屏心 208，跨度 [{hmin},{hmax}]，明细 {hud:?}）"
        );
        // ④ 瓦片右缘判定（二选一定位）：绘制数 ≤ 数据 v>0 格数 且
        //    tile_max_x == 数据右缘（=数据边界则两者相等=合法；窗口缺格则绘制右缘<数据右缘=bug）
        let (v0_total, data_right) = {
            let v = &sc.view;
            let (mut n, mut right) = (0usize, 0i32);
            for r in 0..(v.high as usize) * 2 {
                for c in 0..(v.wide as usize) * 2 {
                    if v.terrain_at(r as i32, c as i32) > 0 {
                        n += 1;
                        right = right.max((c as i32) * 16);
                    }
                }
            }
            (n, right)
        };
        let tiles_x: Vec<i32> = ops
            .iter()
            .filter(|o| o.starts_with("setClip(") && o.ends_with(",16,16)"))
            .filter_map(|o| op_xy(o))
            .map(|c| c.0)
            .collect();
        let tile_max_x = tiles_x.iter().copied().max().unwrap_or(0);
        assert!(
            tiles_x.len() <= v0_total,
            "绘制瓦片数 {} 不得超数据 v>0 格数 {v0_total}（右缘止=数据边界即合法）",
            tiles_x.len()
        );
        assert_eq!(
            tile_max_x, data_right,
            "瓦片右缘判定：tile_max_x={tile_max_x} vs 数据右缘={data_right}（相等=数据边界合法；小于=相机窗口缺格 bug）"
        );
    }

    fn loaded_with(icons: &[ArgbImage], save: &crate::save::FloorSave) -> GameScene {
        let (width_table, walk) = fixtures();
        load_floor_from_save(
            include_bytes!("../tests/fixtures/maplv1.bin"),
            &[],
            &width_table,
            &walk,
            synth_images(icons),
            vec![None; 56],
            save,
        )
        .unwrap()
    }

    /// 换层水位（a.java:7163-7164）：50 抬升+脏；51 结局层不计；单调不回退。
    #[test]
    fn scene_watermark_bump_marks_dirty() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.note_max_floor(50);
        assert!(sc.take_global_dirty(), "水位抬升 ⇒ dirty");
        assert_eq!(sc.global_snapshot().max_floor_watermark, 50);
        sc.note_max_floor(51);
        assert!(!sc.take_global_dirty(), "51 不计入");
        assert_eq!(sc.global_snapshot().max_floor_watermark, 50);
        sc.note_max_floor(30);
        assert_eq!(sc.global_snapshot().max_floor_watermark, 50, "单调不回退");
    }

    /// probeCrossFloorSnapshot 同层路径（a.java:8640-8655）：存活目标型 ⇒ true；removed 跳过。
    #[test]
    fn probe_finds_live_entities_same_floor() {
        let mut sc = scene_with(&synthetic_sptarm());
        let f = sc.floor;
        entity::spawn(&mut sc.entities, &mut sc.grid, 12, 3 << 5, 3 << 5, 0, &sc.width_table, None);
        assert!(sc.probe_cross_floor_snapshot(f, 12), "存活型12 ⇒ true");
        assert!(!sc.probe_cross_floor_snapshot(f, 99), "无此型 ⇒ false");
        let e = sc.entities.count - 1;
        sc.entities.items[e].removed = true;
        assert!(!sc.probe_cross_floor_snapshot(f, 12), "removed 不计");
    }

    /// tryRunScene 共臂门控（a.java:8193-8201）：flags[7] 挡；否则需 floor23 有存活型12。
    #[test]
    fn try_run_scene_shared_probe_gate() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.floor = 23; // 同层化：probe(23,12) 免跨层资产
        assert!(!sc.try_run_scene(32, false), "无型12 ⇒ 挡");
        entity::spawn(&mut sc.entities, &mut sc.grid, 12, 3 << 5, 3 << 5, 0, &sc.width_table, None);
        assert!(sc.try_run_scene(32, false), "有型12 ⇒ 过");
        assert!(!sc.try_run_scene(32, true), "from_walk ⇒ 挡（!script_cursor0）");
        sc.script_line_flags[7] = true;
        assert!(!sc.try_run_scene(32, false), "flags[7] ⇒ 挡");
    }

    /// tryRunScene case 7（a.java:8123-8127）：from_walk ∧ 无型12 ⇒ 过 + 置 flags[32]。
    #[test]
    fn try_run_scene_case7_sets_flag32() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.floor = 23;
        assert!(!sc.try_run_scene(7, false), "非 from_walk ⇒ 挡");
        assert!(sc.try_run_scene(7, true), "from_walk ∧ 无型12 ⇒ 过");
        assert!(sc.script_line_flags[32], "过门置 flags[32]");
    }

    /// changeFloor 四提示浮层（a.java:7135/7142/7153/7156）：strict 下/上界 + 负层/超 55。
    #[test]
    fn change_floor_prompt_texts() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.min_floor = 2;
        sc.max_floor = 10;
        assert!(!sc.change_floor(1, false, true), "strict 低于 min ⇒ false");
        assert_eq!(sc.overlay_text.as_deref(), Some(MISC_TEXTS[0]));
        assert!(!sc.change_floor(11, false, true), "strict 高于 max ⇒ false");
        assert_eq!(sc.overlay_text.as_deref(), Some(MISC_TEXTS[1]));
        assert!(!sc.change_floor(-1, false, false), "负层 ⇒ false");
        assert_eq!(sc.overlay_text.as_deref(), Some(MISC_TEXTS[2]));
        assert!(!sc.change_floor(56, false, false), "超 55 ⇒ false");
        assert_eq!(sc.overlay_text.as_deref(), Some(MISC_TEXTS[3]));
    }

    /// LAY_x（a.java:8355-8362）：脚本层跳转——m_119 存态 + 目标层 + 遮幅（dialogPhase=5）。
    #[test]
    fn script_lay_jumps_floor_with_wipe() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.overlay_text = Some("x".into());
        sc.script_line = "LAY_7 ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();
        assert_eq!(sc.pending_floor, 7);
        assert!(sc.transitioning);
        assert_eq!(sc.dialog_phase, 5);
        assert!(sc.overlay_text.is_none());
    }

    /// GIN_a_b（a.java:8424-8437）：1=得金 b；尾递归执行下一条（END→行尾收尾）。
    #[test]
    fn script_gin_gives_gold_and_recurses() {
        let mut sc = scene_with(&synthetic_sptarm());
        let g0 = sc.player.gold;
        sc.script_line = "GIN_1_1000 ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();
        assert_eq!(sc.player.gold, g0 + 1000, "gainGold(1000)");
        // 尾递归自证：GIN 后递归执行到行尾收尾分支（dialogPhase=4 + 置 flag）
        assert_eq!(sc.dialog_phase, 4, "尾递归走到行尾收尾（a.java:8435-8436）");
        assert!(sc.script_line_flags[sc.current_script_index]);
    }

    /// RES_x（a.java:8470-8490）：easy（difficulty==0）重置 run 统计；非 easy 空转。
    #[test]
    fn script_res_resets_run_stats_on_easy() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.difficulty_index = 0;
        sc.player.hp = 999;
        sc.player.gold = 777;
        sc.player.yellow_keys = 5;
        sc.player.atk = 50;
        sc.min_floor = 9;
        sc.max_floor = 40;
        sc.journal.push((3, 1));
        sc.script_line = "RES_0 ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();
        assert_eq!((sc.player.hp, sc.player.atk, sc.player.def), (400, 10, 10));
        assert_eq!(sc.player.gold, 4);
        assert_eq!((sc.min_floor, sc.max_floor), (1, 3));
        assert_eq!((sc.player.yellow_keys, sc.player.red_keys, sc.player.blue_keys), (0, 0, 0));
        assert_eq!(sc.alchemy_upgrade_count, 0);
        assert!(sc.journal.is_empty(), "inventoryEntryCount 清零（inventoryEntryCount+跟随对）");
        // 非 easy 不动
        let mut sc2 = scene_with(&synthetic_sptarm());
        sc2.difficulty_index = 1;
        sc2.player.hp = 999;
        sc2.script_line = "RES_0 ".to_string();
        sc2.script_cursor = 0;
        sc2.execute_script_instruction();
        assert_eq!(sc2.player.hp, 999, "difficulty!=0 空转");
    }

    /// ADD_a_b_c_d（a.java:8438-8468）：同层生成路径（a==currentFloor 时免存/载切换）。
    #[test]
    fn script_add_spawns_entity_same_floor() {
        let mut sc = scene_with(&synthetic_sptarm());
        let f = sc.floor;
        let n0 = sc.entities.count;
        sc.script_line = format!("ADD_{}_72_11_10 ", f);
        sc.script_cursor = 0;
        sc.execute_script_instruction();
        assert_eq!(sc.entities.count, n0 + 1);
        assert_eq!(sc.entities.items[n0].type_, 72);
        assert_eq!((sc.entities.items[n0].px, sc.entities.items[n0].py), (11 << 5, 10 << 5));
    }
    /// ①②装备往返：serialize 写真实值（a.java:9192-9193）、load 恢复（a.java:9260-9261）。
    #[test]
    fn save_roundtrip_restores_equipment() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.equipped_weapon = 79;
        sc.equipped_armor = 80;
        let blob = sc.serialize_slot_blob();
        assert_eq!(blob[5], 79, "blob[5]=equippedWeaponType（a.java:9192）");
        assert_eq!(blob[6], 80, "blob[6]=equippedArmorType（a.java:9193）");
        let save = crate::save::parse_floor_save(&blob, 128).expect("blob 可解析");
        assert_eq!((save.equipped_weapon_type, save.equipped_armor_type), (79, 80));
        let sc2 = loaded_with(&synthetic_sptarm(), &save);
        assert_eq!((sc2.equipped_weapon, sc2.equipped_armor), (79, 80), "load 恢复装备（a.java:9260-9261）");
    }

    /// 写侧 blob 布局 = Java saveFloorState（a.java:9190-9236）：playerCellX/Y 为 short
    /// （a.java:9197-9198）、alchemyUpgradeCount 实值（9206）、inventoryEntryCount（原 inventoryEntryCount）+
    /// f_byte_arr_14/15 日志对（9207-9212，读侧 9278-9283 → journal）——parse → load 全字段往返。
    #[test]
    fn save_blob_layout_matches_java_and_roundtrips() {
        let mut sc = scene_with(&synthetic_sptarm()); // floor1 (3,10)
        sc.player.hp = 777;
        sc.player.atk = 41;
        sc.player.def = 42;
        sc.player.yellow_keys = 3;
        sc.player.blue_keys = 2;
        sc.player.red_keys = 1;
        sc.player.gold = 1234;
        sc.alchemy_upgrade_count = 5;
        sc.journal = vec![(0, 3), (1, 7)];
        sc.script_line_flags[0] = true;
        let blob = sc.serialize_slot_blob();
        assert_eq!(&blob[7..11], &[0, 3, 0, 10], "writeShort(playerCellX/Y)");
        assert_eq!(&blob[11..15], &777i32.to_be_bytes(), "hp 紧随两 short");
        assert_eq!(&blob[29..33], &1234i32.to_be_bytes(), "gold");
        assert_eq!(&blob[33..35], &5i16.to_be_bytes(), "writeShort(alchemyUpgradeCount)");
        assert_eq!(&blob[35..41], &[0, 2, 0, 3, 1, 7], "inventoryEntryCount=2 + 日志对");
        assert_eq!(blob[41], 1, "旗标段紧随日志对（flags[0]）");
        assert_eq!(blob.len(), 37 + 2 * 2 + 128 + 1 + 56 * 2, "道具栈 0 + 56 层全 null(-1)");
        let save = crate::save::parse_floor_save(&blob, 0).expect("blob 可解析");
        assert_eq!((save.player_cell_x, save.player_cell_y), (3, 10));
        assert_eq!((save.player_hp, save.player_atk, save.player_def), (777, 41, 42));
        assert_eq!((save.yellow_keys, save.blue_keys, save.red_keys), (3, 2, 1));
        assert_eq!((save.gold, save.alchemy_upgrades), (1234, 5));
        assert_eq!(save.journal, vec![(0, 3), (1, 7)]);
        assert!(save.floor_states.iter().all(|s| s.is_none()));
        let sc2 = loaded_with(&synthetic_sptarm(), &save);
        assert_eq!((sc2.player_cell_x, sc2.player_cell_y), (3, 10));
        assert_eq!((sc2.player.hp, sc2.player.atk, sc2.player.def), (777, 41, 42));
        assert_eq!((sc2.player.gold, sc2.alchemy_upgrade_count), (1234, 5));
        assert_eq!(sc2.journal, vec![(0, 3), (1, 7)]);
        assert!(sc2.script_line_flags[0]);
    }

    /// MOT_IF 6 槽表（loadSlotTable 读序 a.java:8974-8988）：`(槽, 武器, 甲)`。
    fn mot_if(slots: &[(usize, i8, i8)]) -> Vec<u8> {
        let mut o = Vec::new();
        for k in 0..6 {
            match slots.iter().find(|s| s.0 == k) {
                Some(&(_, w, a)) => {
                    o.extend_from_slice(&[1, 1, w as u8, a as u8]);
                    for v in [500i32, 30, 30, 1, 0, 0, 100] {
                        o.extend_from_slice(&v.to_be_bytes());
                    }
                }
                None => o.push(0),
            }
        }
        o
    }

    fn paint7(sc: &mut GameScene) -> ArgbImage {
        let mut screen = ArgbImage::create(SCREEN_W, SCREEN_H);
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_font(Some(crate::paint::paint_font()));
            sc.paint_slot_list(&mut g);
        }
        screen
    }

    fn enter7(sc: &mut GameScene) {
        sc.menu_return_mode = GameMode::InGameMenu;
        sc.game_mode = GameMode::SaveSlot;
        sc.enter_menu7();
    }

    fn key7(sc: &mut GameScene, k: i32) {
        sc.key_value = k;
        sc.tick_menu7();
        sc.key_value = 0;
    }

    /// 读入槽表的装备 ≠0 → 图标（经 entityTypeImage 反查同一位图）；=0 → “无”。
    #[test]
    fn scene_slot_list_draws_icon_or_material_name() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.preset_mot_if = Some(mot_if(&[(0, 34, 80), (1, 0, 0), (2, 79, 41)]));
        enter7(&mut sc);
        let s = paint7(&mut sc);
        assert_frame_icon(&s, WEAPON_X, FRAME_Y, &icons, 1, "槽0 武器=34");
        assert_frame_icon(&s, ARMOR_X, FRAME_Y, &icons, 9, "槽0 甲=80");
        key7(&mut sc, -2);
        let s = paint7(&mut sc);
        assert_frame_material_name(&s, WEAPON_X, FRAME_Y, "槽1 武器=0");
        assert_frame_material_name(&s, ARMOR_X, FRAME_Y, "槽1 甲=0");
        key7(&mut sc, -2);
        let s = paint7(&mut sc);
        assert_frame_icon(&s, WEAPON_X, FRAME_Y, &icons, 8, "槽2 武器=79（>7 钳 8）");
        assert_frame_icon(&s, ARMOR_X, FRAME_Y, &icons, 9, "槽2 甲=41（>7 钳 9）");
    }

    /// markSlotEntry（a.java:9013-9014）：存档写回当前 equippedWeaponType/ArmorType，
    /// 详情面板随即按新值画（“保存成功！”浮层下的槽表帧）。
    #[test]
    fn scene_save_copies_equipment_into_slot_table() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.preset_mot_if = Some(mot_if(&[(0, 0, 0)]));
        sc.equipped_weapon = 33;
        sc.equipped_armor = 0;
        enter7(&mut sc);
        for _ in 0..4 {
            key7(&mut sc, -2);
        }
        assert_eq!(sc.slot.cursor, 4);
        assert!(!sc.slot.valid[4]);
        key7(&mut sc, -5);
        assert!(sc.slot.valid[4]);
        assert_eq!((sc.slot.weapon[4], sc.slot.armor[4]), (33, 0));
        let s = paint7(&mut sc);
        assert_frame_icon(&s, WEAPON_X, FRAME_Y, &icons, 0, "存档后 武器=33");
        assert_frame_material_name(&s, ARMOR_X, FRAME_Y, "存档后 甲=0");
        // 再存一次（同槽覆写）：拾甲 80 后的装备态
        sc.overlay_active = false;
        sc.equipped_weapon = 0;
        sc.equipped_armor = 80;
        key7(&mut sc, -5);
        assert_eq!((sc.slot.weapon[4], sc.slot.armor[4]), (0, 80));
        let s = paint7(&mut sc);
        assert_frame_material_name(&s, WEAPON_X, FRAME_Y, "覆写后 武器=0");
        assert_frame_icon(&s, ARMOR_X, FRAME_Y, &icons, 9, "覆写后 甲=80");
    }

    /// 型 6 隐形实体 bump 显形（a.java:5410-5417）：`cur_y++` **恒执行**于显形判定前；
    /// 显形三元组写序钉死 visible→entityState(solid)=2→entityAnimRow(anim_idx)=9
    /// （a.java:5413/5414/5415 逐字，本测试以行号注释锚定写序契约）；已显形再 bump
    /// 仅 cur_y++、三字段**不重写**（a.java:5412 条件）；`continue` 续扫同格后继槽
    /// （同格双型 6 一次 bump 全显形）；cur_y>0 豁免地形终审（a.java:5533-5534）。
    #[test]
    fn type6_bump_reveal_triple_write_order() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 6, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let e1 = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 6, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let e2 = sc.entities.count - 1;
        // m_122 载入语义：型 6 生成后覆盖 visible=false（a.java:9514-9517）
        sc.entities.items[e1].visible = false;
        sc.entities.items[e2].visible = false;
        // 不可走地形：bump 放行只能靠 cur_y++ 豁免（a.java:5533）
        sc.walkable[2][2] = false;

        assert!(
            sc.interact_with_cell(2, 2),
            "型 6 bump 放行（cur_y++ 豁免地形终审，a.java:5533）"
        );
        for e in [e1, e2] {
            assert!(sc.entities.items[e].visible, "显形 1/3 visible=true（a.java:5413）");
            assert_eq!(sc.entities.items[e].state, 2, "显形 2/3 entityState=2（a.java:5414）");
            assert_eq!(sc.entities.items[e].anim_row, 9, "显形 3/3 entityAnimRow=9（a.java:5415）");
        }

        // 二次 bump：已显形仅 cur_y++，三元组不再重写（a.java:5412 `if (!entityVisible)`）
        sc.entities.items[e1].anim_row = 42;
        assert!(sc.interact_with_cell(2, 2), "已显形再 bump 仍放行（cur_y++ 恒执行）");
        assert_eq!(sc.entities.items[e1].anim_row, 42, "一次性显形：anim 不重写");
        assert!(sc.entities.items[e1].visible, "一次性显形：visible 不重写");
        assert_eq!(sc.entities.items[e1].state, 2, "一次性显形：solid 不重写");
    }

    /// MVS 型 82 配对传送（a.java:8519-8592）三断言之【配对+互传+消耗】：
    /// 倒序扫 type==82 按 `param&255` 两两配对（本例 3 与 0x0103=259 低字节
    /// 同键 ⇒ 配对成立，钉 &255 掩码）；cat8 怪物经 carry_monsters_to（a.java:8872-8878）
    /// 搬到对端格（同型同 param spawn + 旧格全同型摘槽）；两 82 点位被
    /// removeGroupAtCellByType(…, 82)（a.java:6434-6449）消耗（removed=true）；
    /// 无参指令游标 = 标签后 1 字符位（a.java:8591 `scriptCursor = script_cursor0 + 1`）。
    #[test]
    fn mvs_pairs_by_param_low_byte_transfers_and_consumes() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let e82a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 5 << 5, 5 << 5, 0x0103, &sc.width_table, None);
        let e82b = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 41, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let ema = sc.entities.count - 1;
        // m_122 载入语义：型 82 生成后覆盖 visible=false（a.java:9601-9605）
        sc.entities.items[e82a].visible = false;
        sc.entities.items[e82b].visible = false;

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        assert!(sc.entities.items[e82a].removed && sc.entities.items[e82b].removed, "消耗：两 82 点位摘槽");
        assert!(sc.entities.items[ema].removed, "原地怪物被 carry_monsters_to 旧格全同型摘除");
        assert_eq!(sc.entities.count, ema + 2, "carry_monsters_to 复制-删除：目标格新增 1 只");
        let moved = ema + 1;
        assert_eq!(sc.entities.items[moved].type_, 41, "互传：同型复制");
        assert!(!sc.entities.items[moved].removed, "互传：副本存活");
        assert_eq!(
            (sc.entities.items[moved].px, sc.entities.items[moved].py),
            (5 << 5, 5 << 5),
            "互传：落到对端格（82B 格）"
        );
        assert_eq!(sc.script_cursor, 4, "无参 op 游标=标签后 1 位（a.java:8591）");
    }

    /// MVS 配对负例（a.java:8532-8542）：`param&255` 不同键 ⇒ 半对（buf 只存
    /// 首元素）⇒ 配对循环 `cur_idx > 0` 不成立 ⇒ 全程无事发生——82 不消耗、
    /// 怪物不搬、实体数不变。
    #[test]
    fn mvs_mismatched_pair_key_leaves_world_untouched() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let e82a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 5 << 5, 5 << 5, 4, &sc.width_table, None);
        let e82b = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 41, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let ema = sc.entities.count - 1;
        sc.entities.items[e82a].visible = false;
        sc.entities.items[e82b].visible = false;
        let count_before = sc.entities.count;

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        assert!(!sc.entities.items[e82a].removed && !sc.entities.items[e82b].removed, "错配 ⇒ 82 不消耗");
        assert!(!sc.entities.items[ema].removed, "错配 ⇒ 怪物不搬");
        assert_eq!(sc.entities.count, count_before, "错配 ⇒ 无 spawn");
    }

    /// MVS 双向互传（a.java:8561-8590 两向 + 8571-8573 交换）：B 格怪物→A 格、
    /// A 格怪物→B 格；方向 2 会重搬方向 1 落进扫描格的**到达怪物**（含槽压缩后
    /// stale 尾重触发 quirk，a.java:6440-6443 就地压缩不清尾）⇒ 合成双怪场景
    /// 终态 = 双怪堆聚于 buf[0]（逆序首扫 82）实体格。Stage A 保真钉住。
    #[test]
    fn mvs_two_way_transfer_piles_monsters_at_first_slot_cell() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let e82a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 5 << 5, 5 << 5, 3, &sc.width_table, None);
        let e82b = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 41, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let ema = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 42, 5 << 5, 5 << 5, 0, &sc.width_table, None);
        let emb = sc.entities.count - 1;
        sc.entities.items[e82a].visible = false;
        sc.entities.items[e82b].visible = false;

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        // 原始 4 只全部摘除（41/42 旧格全同型删除 quirk 一并带走过路副本）
        for e in [e82a, e82b, ema, emb] {
            assert!(sc.entities.items[e].removed, "原地实体 {e} 摘除");
        }
        // 存活终态：双怪副本堆聚 (5,5)（buf[0]=82B 逆序首扫实体格）
        let mut alive: Vec<i32> = (0..sc.entities.count)
            .filter(|&i| !sc.entities.items[i].removed && sc.entities.items[i].type_ >= 41)
            .map(|i| sc.entities.items[i].type_)
            .collect();
        alive.sort_unstable();
        assert_eq!(alive, vec![41, 42], "双怪副本均存活且各型一只");
        for i in 0..sc.entities.count {
            if !sc.entities.items[i].removed && sc.entities.items[i].type_ >= 41 {
                assert_eq!(
                    (sc.entities.items[i].px, sc.entities.items[i].py),
                    (5 << 5, 5 << 5),
                    "副本堆聚 (5,5)"
                );
            }
        }
    }

    /// 真未知 op（16 标签外）忠实行为 = 静默停摆（a.java:8594-8595 原地
    /// return：无臂写 scriptCursor ⇒ 下一拍重解析同一 token）；不推游标、
    /// 不改 dialogPhase。禁止 3+N 跳字节续行（Java 无此行为）。
    /// 语义见 docs/spec/evidence-todos.md T-1 R-TODO-1b。
    #[test]
    fn unknown_op_silently_stalls_cursor() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.script_line = "XYZ ".to_string();
        sc.script_cursor = 0;
        sc.dialog_phase = 3;

        sc.execute_script_instruction();
        assert_eq!(sc.script_cursor, 0, "未知 op 不推游标（原地 return）");
        assert_eq!(sc.dialog_phase, 3, "未知 op 不改 dialogPhase");

        sc.execute_script_instruction(); // 下一拍重解析同一 token
        assert_eq!(sc.script_cursor, 0, "下一拍原地重解析（停摆持续）");
        assert_eq!(sc.dialog_phase, 3, "停摆期间 dialogPhase 不变");
    }

    /// 型 67 多段拼装元数据（a.java:682-697 f_byte_arr2_00/01 逐值）：9 片件
    /// (sheet, srcX, srcY, w, h) + 3 帧 × 5 段 (pieceId, dx, dy, 第4字节)。
    /// 第 4 字节三行恒 0 且绘制循环不读（保真占位，语义未知，C 级不入结论）。
    #[test]
    fn seg67_tables_match_java_metadata() {
        let seg = crate::resource_cfg::global().seg67();
        assert_eq!(
            seg.pieces.as_slice(),
            [
                (0, 0, 0, 33, 56),
                (0, 33, 0, 33, 56),
                (0, 66, 0, 33, 56),
                (2, 0, 0, 18, 26),
                (3, 0, 0, 18, 34),
                (1, 0, 0, 55, 61),
                (4, 0, 0, 14, 26),
                (4, 14, 0, 14, 26),
                (4, 28, 0, 24, 26),
            ],
            "片件表（a.java:682-692）"
        );
        assert_eq!(
            seg.frames,
            [
                [(6, -1, -79, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -53, 0), (0, -16, -27, 0)],
                [(7, -1, -80, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -54, 0), (2, -16, -26, 0)],
                [(8, -1, -81, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -55, 0), (1, -16, -25, 0)],
            ]
            .map(|row: [(i32, i32, i32, i32); 5]| row.to_vec())
            .to_vec(),
            "帧拼装表（a.java:693-697）"
        );
    }

    /// 型 67 多段拼装渲染（a.java:6580-6603）：影子 (x+8, y+22)（6581）→
    /// 基点 (x+16, y+16)（6582-6586）→ SEG67_FRAMES[裸帧偏移] 5 段，每段
    /// 片件 SEG67_PIECES[seg[0]] 画 xtq[seg.sheet] @ (基点+dx, 基点+dy)，
    /// 源 (srcX, srcY) 尺寸 w×h（6591-6601）。**裸帧偏移不乘精灵宽**
    /// （a.java:6578，对照通用臂 6528 乘宽）。逐像素对拍独立转写的参考帧；
    /// 帧行 3 偏移序列 {0,1,2,1} 来自 animOffsetTable[3]（a.java:485-497）。
    #[test]
    fn paint_type67_assembles_five_segments_like_java() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 67, 64, 96, 0, &sc.width_table, None);
        let e = sc.entities.count - 1;
        sc.entities.items[e].anim_row = 3; // animRowByType[67]=3（a.java:498 数组）
        const PIECES: [(i32, i32, i32, i32, i32); 9] = [
            (0, 0, 0, 33, 56),
            (0, 33, 0, 33, 56),
            (0, 66, 0, 33, 56),
            (2, 0, 0, 18, 26),
            (3, 0, 0, 18, 34),
            (1, 0, 0, 55, 61),
            (4, 0, 0, 14, 26),
            (4, 14, 0, 14, 26),
            (4, 28, 0, 24, 26),
        ];
        const FRAMES: [[(i32, i32, i32, i32); 5]; 3] = [
            [(6, -1, -79, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -53, 0), (0, -16, -27, 0)],
            [(7, -1, -80, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -54, 0), (2, -16, -26, 0)],
            [(8, -1, -81, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -55, 0), (1, -16, -25, 0)],
        ];
        const ROW3: [usize; 4] = [0, 1, 2, 1]; // animOffsetTable[3]（a.java:485-497）

        for f in 0..4i32 {
            sc.entities.items[e].anim_frame = f;
            let mut actual = ArgbImage::create(SCREEN_W, SCREEN_H);
            {
                let mut g = SoftGraphics::new(&mut actual);
                sc.paint_entities(&mut g, 0, 0);
            }
            let mut expected = ArgbImage::create(SCREEN_W, SCREEN_H);
            {
                let mut g2 = SoftGraphics::new(&mut expected);
                g2.draw_image(&sc.images.shadow, 64 + 8, 96 + 22, 0); // 影子（6581）
                for &(piece, dx, dy, _pad) in &FRAMES[ROW3[f as usize]] {
                    let (sheet, src_x, src_y, w, h) = PIECES[piece as usize];
                    draw_image_clipped(
                        &mut g2,
                        &sc.images.xtq[sheet as usize],
                        64 + 16 + dx,
                        96 + 16 + dy,
                        src_x,
                        src_y,
                        w,
                        h,
                    );
                }
                // 玩家尾绘（paint_entities 尾段同源，非本测试标的）
                sc.paint_player(&mut g2, 0, 0);
            }
            assert_eq!(
                actual.argb, expected.argb,
                "帧 {f}（裸帧偏移 {}）拼装逐像素一致",
                ROW3[f as usize]
            );
        }
    }

    /// MVS 多对配对扫描序（a.java:8528-8542 + 8546-8558）：逆序首扫的 82 进
    /// buf[偶]，同键后扫的进 buf[奇]；配对循环按 buf 序处理。本例两对
    /// （键 3 / 键 7）各自互传——键 3 对的怪物落 buf[0]=82p1b 格 (5,5)、
    /// 键 7 对的落 buf[0]=82p2b 格 (11,11)。错配/串对都会改写目的地。
    #[test]
    fn mvs_multi_pair_pinning_scans_and_targets_each_pair_cell() {
        let mut sc = scene_with(&synthetic_sptarm());
        // 对 1（键 3）：82p1a (2,2) / 82p1b (5,5)；对 2（键 7）：82p2a (8,8) / 82p2b (11,11)
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let ep1a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 5 << 5, 5 << 5, 3, &sc.width_table, None);
        let ep1b = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 8 << 5, 8 << 5, 7, &sc.width_table, None);
        let ep2a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 11 << 5, 11 << 5, 7, &sc.width_table, None);
        let ep2b = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 41, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let m1 = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 42, 8 << 5, 8 << 5, 0, &sc.width_table, None);
        let m2 = sc.entities.count - 1;
        for e in [ep1a, ep1b, ep2a, ep2b] {
            sc.entities.items[e].visible = false;
        }

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        // 四点位全消耗、两原地怪物摘除
        for e in [ep1a, ep1b, ep2a, ep2b, m1, m2] {
            assert!(sc.entities.items[e].removed, "原地实体 {e} 摘除");
        }
        // 互传目的地按各对 buf[0] 实体格：键 3 对 → (5,5)，键 7 对 → (11,11)
        let mut dest: Vec<(i32, i32, i32)> = (0..sc.entities.count)
            .filter(|&i| !sc.entities.items[i].removed && sc.entities.items[i].type_ >= 41)
            .map(|i| (sc.entities.items[i].type_, sc.entities.items[i].px, sc.entities.items[i].py))
            .collect();
        dest.sort_unstable();
        assert_eq!(
            dest,
            vec![(41, 5 << 5, 5 << 5), (42, 11 << 5, 11 << 5)],
            "两对各归各的 buf[0] 82 格（扫描序/配对键钉死）"
        );
    }

    /// MVS 无怪纯配对（a.java:8564-8568）：82 消耗（removeGroupAtCellByType(…,82)）
    /// 是 cat8 搬运分支**内部的副作用**——扫描格无怪物时该分支不触发 ⇒ 配对成立
    /// 也**不消耗**点位、无 spawn。补遗 #4"一次性消耗两点位"的精确条件由此钉死。
    #[test]
    fn mvs_pair_without_monsters_leaves_82_points() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let e82a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 5 << 5, 5 << 5, 3, &sc.width_table, None);
        let e82b = sc.entities.count - 1;
        sc.entities.items[e82a].visible = false;
        sc.entities.items[e82b].visible = false;
        let count_before = sc.entities.count;

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        assert!(!sc.entities.items[e82a].removed && !sc.entities.items[e82b].removed, "无怪 ⇒ 82 不消耗");
        assert_eq!(sc.entities.count, count_before, "无怪 ⇒ 无 spawn");
        assert_eq!(sc.script_cursor, 4, "无参 op 游标照常 = 标签后 1 位（a.java:8591）");
    }

    /// carry_monsters_to 旧格同型一并消失 + stale 尾复活（a.java:8872-8878 + 6434-6449 +
    /// 8561-8568 循环体）：同格双 41 时，M1 的 carry_monsters_to 清旧格把 M2 **一并带走**
    /// （removeGroupAtCellByType 按型全清，非搬移）；随后槽组循环的容量快照
    /// 不变、压缩不清尾 ⇒ M2 经 stale 尾重触发再被复制到目标格。净终态 =
    /// 双 41 副本堆聚 (5,5)、原地全灭、实体数 +2（两次 carry_monsters_to spawn）。
    #[test]
    fn m_107_old_cell_same_type_sweep_with_stale_revival() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let e82a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 5 << 5, 5 << 5, 3, &sc.width_table, None);
        let e82b = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 41, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let m1 = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 41, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let m2 = sc.entities.count - 1;
        sc.entities.items[e82a].visible = false;
        sc.entities.items[e82b].visible = false;

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        for e in [e82a, e82b, m1, m2] {
            assert!(sc.entities.items[e].removed, "原地实体 {e} 摘除（含同型一并带走）");
        }
        assert_eq!(sc.entities.count, m2 + 3, "两次 carry_monsters_to spawn（stale 尾复活）");
        let alive: Vec<usize> = (0..sc.entities.count).filter(|&i| !sc.entities.items[i].removed).collect();
        assert_eq!(alive.len(), 2, "双 41 副本存活");
        for i in alive {
            assert_eq!(sc.entities.items[i].type_, 41);
            assert_eq!(
                (sc.entities.items[i].px, sc.entities.items[i].py),
                (5 << 5, 5 << 5),
                "副本堆聚目标格 (5,5)"
            );
        }
    }

    /// MVS 不成对单只 82 空转（a.java:8528-8542 配对槽判空 buf[v]-1<=0 存首元素、
    /// 8546-8558 配对循环 `cur_idx > 0` 才进搬运分支）：孤只 82 只占半对（buf[偶]）
    /// ⇒ 搬运分支不触发 ⇒ 全表无变化（不消耗、无 spawn）；无参指令游标仍照常
    /// 推进 = 标签后 1 字符位（a.java:8591）。fixture-plan §2 FX-MVS-A 断言点补口。
    #[test]
    fn mvs_unpaired_single_82_is_noop_but_advances_cursor() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let e82 = sc.entities.count - 1;
        sc.entities.items[e82].visible = false;
        let count_before = sc.entities.count;

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        // 全表无变化：不消耗、无 spawn、字段不动
        assert!(!sc.entities.items[e82].removed, "孤只 82 不消耗");
        assert_eq!(sc.entities.count, count_before, "孤只 82 无 spawn");
        assert_eq!(sc.entities.items[e82].type_, 82);
        assert_eq!(sc.entities.items[e82].param, 3);
        assert!(!sc.entities.items[e82].visible, "hidden 状态不被改写");
        assert_eq!(sc.entities.items[e82].state, 0);
        // 游标照常推进（空转 ≠ 停摆；停摆语义属未知 op，见 unknown_op 测试）
        assert_eq!(sc.script_cursor, 4, "无参 op 游标 = 标签后 1 位（a.java:8591）");

        // 幂等：游标拨回重解析同一 token，MVS 臂本身仍空转
        // （注意不拨游标的第二次调用会走行尾收尾 a.java:8597-8604，非重解析）
        sc.script_cursor = 0;
        sc.execute_script_instruction();
        assert_eq!(sc.entities.count, count_before);
        assert!(!sc.entities.items[e82].removed);
    }

    /// 暗道换乘键 49/55 **分立臂**（a.java:5295-5306）：49=上行 →
    /// findFloorGateEntity(true)（型 7 门）+ changeFloor(param&255, false, true)；
    /// 55=下行 → findFloorGateEntity(false)（型 8 门）+ changeFloor(…, true, true)。
    /// 回归钉（Rust 移植 bug 修复，非原版 bug、不属 D4 保真）：①曾先清零
    /// key_value 再判 up ⇒ up 恒 false、49 臂被并入 55；②Java case break 后
    /// 续走 keyHeldCode switch（a.java:5308 起），臂末 return 曾吞同拍方向步进。
    #[test]
    fn field_input_keys_49_55_split_up_down_gates() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.stair_hint = true; // f_bool_13（a.java:5297/5303 门控）
        sc.min_floor = 1;
        sc.max_floor = 55;
        // 双门并存：型 7（上行）目标层 10、型 8（下行）目标层 20（param>>9==0）
        entity::spawn(&mut sc.entities, &mut sc.grid, 7, 2 << 5, 2 << 5, 10, &sc.width_table, None);
        entity::spawn(&mut sc.entities, &mut sc.grid, 8, 5 << 5, 5 << 5, 20, &sc.width_table, None);

        // 49 = 上行：只认型 7 门；changeFloor 第二参 false
        sc.key_value = 49;
        sc.handle_field_input();
        assert_eq!(sc.key_value, 0, "键值清零（a.java:5296）");
        assert_eq!(
            sc.pending_floor, 10,
            "49 走型 7 门 findFloorGateEntity(true)（a.java:5297）"
        );
        assert!(!sc.went_down, "49 → changeFloor(…, false, true)（a.java:5297）");

        // 55 = 下行：只认型 8 门；changeFloor 第二参 true
        sc.key_value = 55;
        sc.handle_field_input();
        assert_eq!(sc.key_value, 0, "键值清零（a.java:5302）");
        assert_eq!(
            sc.pending_floor, 20,
            "55 走型 8 门 findFloorGateEntity(false)（a.java:5303）"
        );
        assert!(sc.went_down, "55 → changeFloor(…, true, true)（a.java:5303）");

        // break 语义：臂末不 return，同拍 keyHeldCode switch 照跑（a.java:5308 起）
        sc.key_value = 49;
        sc.key_held = -1;
        sc.handle_field_input();
        assert_eq!(sc.facing, 1, "49 臂后 keyHeld switch 照跑（-1=上 → facing=1）");
    }

    /// 49/55 门控：f_bool_13（stair_hint）未亮则双键均不动（a.java:5297/5303）。
    #[test]
    fn field_input_keys_49_55_gated_by_stair_hint() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.stair_hint = false;
        entity::spawn(&mut sc.entities, &mut sc.grid, 7, 2 << 5, 2 << 5, 10, &sc.width_table, None);
        entity::spawn(&mut sc.entities, &mut sc.grid, 8, 5 << 5, 5 << 5, 20, &sc.width_table, None);
        sc.pending_floor = -1;
        for k in [49, 55] {
            sc.key_value = k;
            sc.handle_field_input();
        }
        assert_eq!(sc.pending_floor, -1, "stair_hint 未亮 ⇒ 不换层");
    }

    /// -5/53 战况屏臂 break 语义（a.java:5271-5294 case -5/53 共臂尾 break）：
    /// Java break 出 keyValue switch 后**续走** keyHeldCode switch（5308 起）
    /// ——与方向键同拍时同拍 facing+tryStep。原实现臂末 return 吞掉（与型 15
    /// 同型移植缺陷；2026-10-16 lead 裁定修，非 D4 保真对象）。
    #[test]
    fn field_input_key_minus5_falls_through_to_key_held() {
        let mut sc = scene_with(&synthetic_sptarm());
        // 臂效（无方向键，keyHeld switch 落 default）：战况屏态 + 相机解锁
        sc.key_value = -5;
        sc.handle_field_input();
        assert_eq!(sc.walk_phase, 2, "战况屏态照设（a.java:5292）");
        assert!(!sc.view_locked, "cameraLockedOnPlayer=false（a.java:5280）");
        // break 语义：同拍方向键照跑 keyHeldCode switch（a.java:5308 起）
        sc.key_value = -5;
        sc.key_held = -1;
        sc.handle_field_input();
        assert_eq!(sc.facing, 1, "-5 臂后 keyHeld switch 照跑（-1=上 → facing=1）");
    }

    /// 53 同臂（a.java:5272 case 53 与 -5 共体）：break 语义同上。
    #[test]
    fn field_input_key_53_falls_through_to_key_held() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.key_value = 53;
        sc.handle_field_input();
        assert_eq!(sc.walk_phase, 2, "战况屏态照设（a.java:5292）");
        sc.key_value = 53;
        sc.key_held = -3;
        sc.handle_field_input();
        assert_eq!(sc.facing, 3, "53 臂后 keyHeld switch 照跑（-3=左 → facing=3）");
    }

    /// overlayKind 六臂分派（run() tableswitch 0-5：javap renamed dump 95-663 +
    /// deobf 3252-3345）BUG-002 活码核查补移植钉①：kind4/3/兜底的 -2/-1 翻页
    /// = handleOverlayKey 的 top∓**1**（a.java:4925-4934）——原实现误用
    /// ±lines_per_page（kind0(15/17) 的 ±lines 与 kind5 的 ±1 是对的，勿动）。
    #[test]
    fn overlay_paging_keys_are_single_line_for_kind4_and_default() {
        let mut sc = scene_with(&synthetic_sptarm());
        let long = "一二三四五六七八九十甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午未申酉戌亥"
            .repeat(12);
        sc.show_overlay_message(4, &long, 0, 0);
        assert!(
            sc.overlay_total_lines > sc.overlay_lines_per_page + 1,
            "总行数须超出单页才可翻页 total={} lpp={}",
            sc.overlay_total_lines,
            sc.overlay_lines_per_page
        );
        sc.overlay_page_top = 0;
        sc.key_value = -2;
        sc.tick(0, &[]);
        assert_eq!(sc.overlay_page_top, 1, "kind4 -2 = top+1（handleOverlayKey ±1）");

        sc.show_overlay_message(3, &long, 0, 3);
        sc.overlay_page_top = 0;
        sc.key_value = -2;
        sc.tick(0, &[]);
        assert_eq!(sc.overlay_page_top, 1, "kind3 default → handleOverlayKey ±1");
    }

    /// 钉②：kind2 全吞键（Java `case 2: default: break;`，deobf 3290-3291）——
    /// 调用面 0 命中（Java 亦无生产者），死码臂为忠实而保（同 MVS 先例）。
    #[test]
    fn overlay_kind2_swallows_all_keys() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.show_overlay_message(2, "占位", 0, 0);
        for k in [-7, -6, -1, 1] {
            sc.key_value = k;
            sc.tick(0, &[]);
            assert!(sc.overlay_active, "kind2 全吞键（key={k}）");
        }
    }

    /// 钉③：kind1 图标支路（javap run() 336-441）：right∉{2,3} 全吞键；
    /// right∈{2,3}：0 吞、-2/-1 走 handleOverlayKey、其余关。
    #[test]
    fn overlay_kind1_icon_dispatch() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.show_overlay_message(1, "占位", 0, 1); // right=1（钥匙图标类）
        sc.key_value = -7;
        sc.tick(0, &[]);
        assert!(sc.overlay_active, "right 非 2/3：全吞键");

        sc.show_overlay_message(1, "占位", 0, 3); // right=3（拾取浮层实况）
        sc.key_value = 0;
        sc.tick(0, &[]);
        assert!(sc.overlay_active, "right 为 2/3 + key0 吞");
        sc.key_value = -7;
        sc.tick(0, &[]);
        assert!(!sc.overlay_active, "right 为 2/3 + 非翻页键关闭");
    }
}

/// 场景级单测的合成夹具（本文件 #[cfg(test)] 模块共用）：13×13 网格（走廊 y=ROW、
/// x∈1..=11 地形可走，其余墙）+ 纯色合成图集 + 真实数值表夹具。
#[doc(hidden)]
pub mod scene_test_support {
    use crate::render::FontMetrics;
    use super::*;

    const W: i32 = 13;
    pub const ROW: i32 = 3;

    fn solid(w: i32, h: i32) -> ArgbImage {
        ArgbImage::from_rgb(&vec![0xFF202020; (w * h) as usize], w, h, true)
    }

    fn many(n: usize) -> Vec<ArgbImage> {
        (0..n).map(|_| solid(24, 24)).collect()
    }

    pub(crate) fn images() -> SceneImages {
        SceneImages {
            backdrop: solid(77, SCREEN_H),
            shadow: solid(24, 24),
            actor: solid(24, 24),
            actor_face: solid(24, 24),
            ui: many(25),
            stair_icons: (solid(24, 24), solid(24, 24)),
            open_anim: solid(24, 24),
            opened_door: solid(24, 24),
            entity: SceneImages::map_entity_images(&many(13), &many(23), &many(10), &many(20), &many(20)),
            popup: crate::popup::PopupImages {
                digit_strips: [solid(24, 24), solid(24, 24), solid(24, 24)],
                icon_strip: solid(24, 24),
                battle_digits: solid(24, 24),
            },
            spark_sheet: solid(24, 24),
            corner_sprites: solid(24, 24),
            map211: None,
            map210: solid(24, 24),
            menu_strip: solid(24, 24),
            battle_icon: solid(24, 24),
            battle_strip: solid(24, 24),
            faces: many(12),
            ending: solid(24, 24),
            xtq: many(6),
        }
    }

    pub(super) fn numbers(text: &str) -> Vec<i32> {
        text.lines().filter_map(|l| l.trim().parse().ok()).collect()
    }

    /// 合成 maplv：头 u16 LE（格数×2），地形段在 rebuildWalkability 索引
    /// （行距 wide·4、列距 2）置 tile 1（可走），其余 tile 0（墙）。
    pub(super) fn maplv() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&((W * 2) as u16).to_le_bytes());
        v.extend_from_slice(&((W * 2) as u16).to_le_bytes());
        let n = (W * W * 4) as usize;
        let mut terrain = vec![0u8; n];
        for x in 1..=11 {
            terrain[(ROW * W * 4 + x * 2) as usize] = 1;
        }
        v.extend_from_slice(&terrain);
        v.extend_from_slice(&vec![0u8; n]);
        v
    }

    /// 玩家格 + 实体 (type, cx, cy) 的场景；敌表 ×1 缩放 + 金币表。
    #[test]
    fn modern_paint_no_residual_across_states() {
        // 【A 黄2·DEADBEEF 钉升格】迁移链 dirty-canvas 零残留
        // （title→存档→游戏→游戏；原钉仅罩 BootMachine 单点=破坏态零旁证）。
        use crate::layout::Layout;
        use crate::paint::BootMachine;
        use crate::render::{ArgbImage, SoftGraphics};
        let img = || ArgbImage::create(2, 2);
        let vec = |n: usize| (0..n).map(|_| img()).collect::<Vec<_>>();
        let mut m = BootMachine::new(img(), img(), vec(8), vec(25), vec(2), img(), vec(2), vec(1), vec(2));
        m.apply_layout(Layout::modern(18, 18));
        for _ in 0..200 {
            m.tick(0); // → MainMenu（title/存档族态）
        }
        let mut canvas = ArgbImage::create(576, 576);
        for px in canvas.argb.iter_mut() {
            *px = 0xDEADBEEF;
        }
        {
            let mut g = SoftGraphics::new(&mut canvas);
            m.paint(&mut g);
        }
        assert_eq!(
            canvas.argb.iter().filter(|p| **p == 0xDEADBEEF).count(),
            0,
            "title/menu 态零残留"
        );
        // 游戏态（GameScene::paint 入口面——A 黄2 点名 scene.rs:5981）
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18)); // Modern 面（清屏门控域）
        let mut canvas2 = ArgbImage::create(240, 320);
        for px in canvas2.argb.iter_mut() {
            *px = 0xDEADBEEF;
        }
        {
            let mut g = SoftGraphics::new(&mut canvas2);
            sc.paint(&mut g, &img());
        }
        assert_eq!(
            canvas2.argb.iter().filter(|p| **p == 0xDEADBEEF).count(),
            0,
            "游戏态零残留（paint 入口全幅自铺）"
        );
        // 再迁移（对话/再游戏帧序）：同画布复绘零残留
        {
            let mut g = SoftGraphics::new(&mut canvas2);
            sc.paint(&mut g, &img());
        }
        assert_eq!(
            canvas2.argb.iter().filter(|p| **p == 0xDEADBEEF).count(),
            0,
            "迁移后复绘零残留"
        );
    }

    pub fn scene(player: (i32, i32), ents: &[(u8, i32, i32)]) -> GameScene {
        let width_table = numbers(include_str!("../tests/fixtures/type_width_px.txt"));
        let walk = numbers(include_str!("../tests/fixtures/tile_walkability.txt"));
        assert_eq!((walk[0], walk[1]), (0, 1), "合成 tile 0=墙 1=可走");
        let recs: Vec<(u8, u16, u16, i16, bool)> = ents
            .iter()
            .map(|&(t, x, y)| (t, (x << 5) as u16, (y << 5) as u16, 0, true))
            .collect();
        let mut sc = GameScene::load_floor(
            &maplv(),
            player.0 << 5,
            player.1 << 5,
            &recs,
            &width_table,
            &walk,
            images(),
            vec![None; 56],
        )
        .unwrap();
        sc.floor = 2;
        sc.current_floor = 2;
        let tables = crate::combat::CombatTables {
            enemy_base_hp: numbers(include_str!("../tests/fixtures/enemybasehp.txt")),
            enemy_base_atk: numbers(include_str!("../tests/fixtures/enemybaseatk.txt")),
            enemy_base_def: numbers(include_str!("../tests/fixtures/enemybasedef.txt")),
            difficulty_multipliers: numbers(include_str!("../tests/fixtures/difficulty_multipliers.txt")),
        };
        sc.enemies = crate::combat::scale_enemy_stats(&tables, 1);
        sc.enemy_base_hp = tables.enemy_base_hp.clone();
        sc.enemy_base_gold = numbers(include_str!("../tests/fixtures/enemybasegold.txt"));
        sc
    }
}

/// BUG-017 回归（2026-10-17 用户实机："物品栏使用火眼金睛 → 游戏已终止"）：
/// 道具面板确认型 13 = Java 双跳 mode10→3→5（a.java:3939-3945 + 7786-7788）。
/// 曾因 TRANSFER_EDGES 缺 Map→EnemyInfo 边触发 census assert panic——
/// **该 panic 属 Rust 自造断言，非原版崩溃**（"原版等价终止"声称误报）。
#[cfg(test)]
mod item13_use_regression {
    use super::scene_test_support::scene;
    use crate::enums::GameMode;

    #[test]
    fn item13_use_from_panel_reaches_enemyinfo() {
        let mut sc = scene((2, 2), &[(43, 5, 5), (78, 6, 6)]);
        sc.player.item_types[0] = 13;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        sc.item_cursor = 0;
        sc.game_mode = GameMode::ItemPanel;
        sc.key_value = -5;
        sc.tick_item_panel(); // 型13：transition_to(Map) + activate_item(13)→EnemyInfo
        assert_eq!(sc.game_mode, GameMode::EnemyInfo, "型13 使用后应进图鉴（mode5）");
    }

    #[test]
    fn item14_use_from_panel_reaches_recordbrowser() {
        let mut sc = scene((2, 2), &[(43, 5, 5)]);
        sc.player.item_types[0] = 14;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        sc.item_cursor = 0;
        sc.journal.push((0, 0)); // 非空日志（空日志回退=“没有记录”回面板，另一分支）
        sc.game_mode = GameMode::ItemPanel;
        sc.key_value = -5;
        sc.tick_item_panel();
        assert_eq!(sc.game_mode, GameMode::RecordBrowser, "型14 使用后应进记录浏览器（mode12）");
    }
}

/// BUG-023 回归（diff-trace 既有红，2026-10-20 根因定位）：道具确认浮层（kind3）
/// `-7`"回面板"臂在 **ItemPanel 态**调 `transition_to(ItemPanel)`——Java 原版 =
/// `gameMode = 10` **自赋值**（deobf a.java:3297 `var15.gameMode = 10;`）。TRANSFER_EDGES
/// 缺 ItemPanel→ItemPanel 自环边 ⇒ census debug_assert 误杀（BUG-012 Dialog→Dialog、
/// BUG-017 Map→EnemyInfo 同类第三例）；debug 档 `cargo test -p game-oracle` 因此
/// preset_itemstack_frames_match panic（release 档 debug_assert 不编译故 rust-tests 不现）。
#[cfg(test)]
mod itempanel_self_edge_regression {
    use super::scene_test_support::scene;
    use crate::enums::GameMode;

    #[test]
    fn confirm_overlay_cancel_returns_to_panel() {
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18)); // Modern 面（清屏门控域）
        sc.game_mode = GameMode::ItemPanel;
        sc.overlay_active = true;
        sc.overlay_kind = 3;
        sc.key_value = -7;
        sc.tick(0, &[]); // debug 档：缺自环边 ⇒ census assert panic（RED）
        assert_eq!(sc.game_mode, GameMode::ItemPanel, "回面板=自环（Java gameMode=10 自赋值）");
        assert!(!sc.overlay_active, "kind3 -7 关层");
    }

    /// 边表机械对照（state-machine-anchors 同源判据）：自环边在册。
    #[test]
    fn itempanel_self_edge_registered() {
        assert!(
            crate::scene::TRANSFER_EDGES
                .contains(&(GameMode::ItemPanel, Some(GameMode::ItemPanel))),
            "ItemPanel→ItemPanel 自环边须在 TRANSFER_EDGES（件 D 维护义务）"
        );
    }
}

/// 指针命中（WIN-OP③，Stage B 加法——decision_pointer-menu-stage-b；
/// 实现微调经 adversary 批准 2026-10-18：命中→Row{index, confirm_key}+select_row
/// 只动光标，确认走 shell route_key 合成键=唯一键路，零新游戏语义）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerHit {
    None,
    /// index = 绝对行/格索引（③-2 定义）；confirm_key = 该态确认键（③-3 逐态）。
    Row { index: i32, confirm_key: i32 },
}

impl GameScene {
    /// 菜单行带几何（③-1 单源：与 paint_menu_list 逐项同式——居中盒、行首 16+pad、
    /// 行距 menu_line_height；pad=(line_height-font.height())>>1）。返回 (行y, 行高)。
    fn menu_row_band(&self, visible: i32) -> (i32, i32) {
        let font = crate::paint::paint_font();
        let pad = (self.menu_line_height - font.height()) >> 1;
        let by = CENTER_Y - ((self.menu_list_height + 32) >> 1) + 16 + pad
            + visible * self.menu_line_height;
        (by - pad, self.menu_line_height)
    }

    /// 背包栅格原点（③-1 单源）：**paint_item_panel 与 pointer_hit 共调本 fn**。
    /// y = 盒顶+31+font.height()——paint 累积式（:3412 +16、:3420 +fh+4、:3424 +1、
    /// :3427 +10）逐项同式；2026-10-18 F-1 整改：原写 32 差 1px + 测试自证，已修。
    /// > 【2026-10-26 更正】盒顶/盒左须取 **layout** 画布（paint 盒框已是 layout 居中）——
    /// > 旧 SCREEN_W/H 常量使 Modern 下格阵留在经典 240×320 左上区、与盒框脱节（竖屏实机
    /// > 截图实证）。classic 代入 layout=240×320 原值零偏离。
    pub fn item_cell_origin(&self) -> (i32, i32) {
        let font = crate::paint::paint_font();
        (
            (self.layout.screen_w - self.item_box_w) / 2 + 20,
            (self.layout.screen_h - self.item_box_h) / 2 + 31 + font.height(),
        )
    }

    /// 指针命中：游戏内菜单/商店（行带）与背包（栅格）。对话/玩法态恒 None
    /// （不点走剧情、不在行走态误触）。confirm_key 逐态（③-3）：菜单族激活=-6、
    /// 道具使用=-6（键码契约 §4 同码不同动作——标题确认另有 -5/53）。
    /// 【BUG-039 · Stage B 触屏】点空白（未命中任何行 / 格）时代按的键。只有"继续 / 关闭"类
    /// 情境代按确定 -5：对话、结局、普通提示框；要做决定的界面（菜单、存档、炼丹、道具栏、
    /// 设置等列表）、原版以 ○/× 脚注标明的是否框（道具使用确认 kind 3、商人报价 kind 4）与
    /// 赠予 / 报价（kind 4 型 77 flags&2，STAGE-B-008）点空白
    /// 不代按——曾：存档页点空白 = 覆盖当前槽、炼丹炉 = 花钱、菜单 = 执行高亮项。换层遮幅期
    /// 浮层不画却照吞键（paint 公共尾 `!transitioning` 门），点空白亦不代按。
    pub fn blank_tap_key(&self) -> Option<i32> {
        if self.transitioning {
            return None;
        }
        if self.overlay_active {
            // 【STAGE-B-008 · spec §5.2】赠予 / 报价（kind 4、型 77 = 文本源 0、param 高字节 &2）无脚注图标，但 −5 = 接受（扣金 / 送物），
            // 是要做决定的框 ⇒ 点空白不代按（曾代按 −5）。判据与 apply_merchant_offer 同源（买成功 ^512 后的告示子框照旧代按）。
            // type_==77 为防御性判据（复核 T9-R1：生产态空间中型 78 两子类图标恒 (1,2)、本就 None）。
            if self.overlay_kind == 4 && self.merchant_entity >= 0 {
                let ent = &self.entities.items[self.merchant_entity as usize];
                if ent.type_ == 77 && ((ent.param as i32) >> 8) & 2 != 0 {
                    return None;
                }
            }
            let yes_no = self.overlay_left_icon == 1 && self.overlay_right_icon == 2;
            return if yes_no { None } else { Some(-5) };
        }
        match self.game_mode {
            GameMode::Dialog | GameMode::Ending => Some(-5),
            _ => None,
        }
    }

    pub fn pointer_hit(&self, x: i32, y: i32) -> PointerHit {
        // 【BUG-038】浮层在最上层：其下的行 / 格不接受命中（原版浮层分支先于各模式吞键）
        if self.overlay_active {
            return PointerHit::None;
        }
        match self.game_mode {
            GameMode::InGameMenu | GameMode::Shop => {
                let vis_n = self.menu_visible_bottom - self.menu_scroll_top;
                for vis in 0..vis_n.max(0) {
                    let (ry, rh) = self.menu_row_band(vis);
                    if y >= ry && y < ry + rh {
                        let index = self.menu_scroll_top + vis;
                        if index >= 0 && index < self.menu_item_count {
                            return PointerHit::Row { index, confirm_key: -6 };
                        }
                    }
                }
                PointerHit::None
            }
            GameMode::SaveSlot | GameMode::EquipDetail => {
                // 存档槽列表（SlotSelect，7/8 共享 dispatchModeEntry case7）：行带单源；
                // confirm=-5（run case 7：-6/-5 同义确认——mode7 存档/mode8 读档）
                for vis in 0..(self.slot.scroll_bottom - self.slot.scroll_top).max(0) {
                    let (ry, rh) = self.slot.row_band(vis, &self.layout);
                    if y >= ry && y < ry + rh {
                        let index = self.slot.scroll_top + vis;
                        return PointerHit::Row { index, confirm_key: -5 };
                    }
                }
                PointerHit::None
            }
            GameMode::About => {
                // 设置面板（OptionList）：行带单源 row_band；confirm=-5（run case 16
                // 翻转键，a.java:4143-4188）
                if let Some(list) = self.options.as_ref() {
                    for i in 0..list.len() {
                        let (ry, rh) = list.row_band(i, &self.layout);
                        if y >= ry && y < ry + rh {
                            return PointerHit::Row { index: i as i32, confirm_key: -5 };
                        }
                    }
                }
                PointerHit::None
            }
            GameMode::ItemPanel => {
                let (ox, oy) = self.item_cell_origin();
                if x >= ox && y >= oy {
                    let c = (x - ox) / 40;
                    let r = (y - oy) / 40;
                    if c >= 0 && c < self.item_cols && r >= 0
                        && r < self.item_scroll_bottom - self.item_scroll_top
                    {
                        let index = (self.item_scroll_top + r) * self.item_cols + c;
                        return PointerHit::Row { index, confirm_key: -6 };
                    }
                }
                PointerHit::None
            }
            _ => PointerHit::None,
        }
    }

    /// 指针选行（hover 高亮/点击对位）：只动光标+滚窗同步，无其他状态变化（③-2/b）。
    pub fn select_row(&mut self, index: i32) {
        match self.game_mode {
            GameMode::SaveSlot | GameMode::EquipDetail => {
                self.slot.set_cursor(index); // 只动光标+滚窗
            }
            GameMode::InGameMenu | GameMode::Shop => {
                if index >= 0 && index < self.menu_item_count {
                    self.menu_cursor = index;
                    let n = self.menu_visible_bottom - self.menu_scroll_top;
                    if self.menu_cursor >= self.menu_visible_bottom {
                        self.menu_visible_bottom = self.menu_cursor + 1;
                        self.menu_scroll_top = self.menu_visible_bottom - n;
                    } else if self.menu_cursor < self.menu_scroll_top {
                        self.menu_scroll_top = self.menu_cursor;
                        self.menu_visible_bottom = self.menu_scroll_top + n;
                    }
                }
            }
            GameMode::About => {
                if let Some(list) = self.options.as_mut() {
                    if index >= 0 && (index as usize) < list.len() {
                        list.set_highlight(index as usize); // 只动高亮（光标语义），零其他状态
                    }
                }
            }
            GameMode::ItemPanel => {
                let total = self.item_cols * self.item_rows_total;
                if index >= 0 && index < total {
                    self.item_cursor = index;
                    let n = self.item_scroll_bottom - self.item_scroll_top;
                    let row = self.item_cursor / self.item_cols;
                    if row >= self.item_scroll_bottom {
                        self.item_scroll_bottom = row + 1;
                        self.item_scroll_top = self.item_scroll_bottom - n;
                    } else if row < self.item_scroll_top {
                        self.item_scroll_top = row;
                        self.item_scroll_bottom = self.item_scroll_top + n;
                    }
                }
            }
            _ => {}
        }
    }
}

/// WIN-OP③ 指针缝测试（adversary 4 条件：(a) confirm_key 逐态 (b) select_row 只动
/// 光标+滚窗 (c) 合成=press+release 同拍（壳侧同款点按，见 shell 测试）(d) 几何单源）
#[cfg(test)]
mod pointer_seam_tests {
    use crate::render::FontMetrics;
    use super::scene_test_support::scene;
    use super::{GameMode, PointerHit};
    use crate::layout::{SCREEN_H, SCREEN_W};

    fn menu_scene() -> super::GameScene {
        let mut sc = scene((2, 2), &[(43, 5, 5)]);
        sc.game_mode = GameMode::InGameMenu;
        sc.menu_kinds = vec![0, 1, 2, 3, 4, 5];
        sc.menu_item_count = 6;
        sc.menu_scroll_top = 0;
        sc.menu_visible_bottom = 6;
        sc.menu_line_height = 18;
        sc.menu_box_width = 148;
        sc.menu_list_height = 112;
        sc.menu_cursor = 0;
        sc
    }

    /// (d)+(a)：行带几何与 paint_menu_list 同式逐行锚 + 命中带定死 + confirm_key=-6
    #[test]
    fn menu_pointer_hit_geometry_boundaries_and_confirm_key() {
        let sc = menu_scene();
        let (y0, h) = sc.menu_row_band(0);
        assert_eq!(sc.pointer_hit(120, y0 + h / 2), PointerHit::Row { index: 0, confirm_key: -6 });
        let (y2, h2) = sc.menu_row_band(2);
        assert_eq!(sc.pointer_hit(120, y2 + h2 / 2), PointerHit::Row { index: 2, confirm_key: -6 });
        // 命中带边界（③-2/c）：行尾-1 归本行、下一像素归下行
        assert_eq!(sc.pointer_hit(120, y0 + h - 1), PointerHit::Row { index: 0, confirm_key: -6 });
        assert_eq!(sc.pointer_hit(120, y0 + h), PointerHit::Row { index: 1, confirm_key: -6 });
        // 盒外/空洞 = None
        assert_eq!(sc.pointer_hit(120, y0 - 3), PointerHit::None);
        // 行几何 = paint 同式（居中盒 16+pad、行距 18）
        assert_eq!(y0, 160 - ((112 + 32) >> 1) + 16, "行带 y 与 paint_menu_list 同式锚（pad 相消）");
        // 玩法/对话态恒 None（不误触）
        let mut s2 = menu_scene();
        s2.game_mode = GameMode::Map;
        assert_eq!(s2.pointer_hit(120, y0 + 5), PointerHit::None);
    }

    /// (b)：select_row 只动光标+滚窗同步，其他状态零变化
    #[test]
    fn select_row_only_cursor_and_scroll() {
        let mut sc = menu_scene();
        let hp = sc.player.hp;
        let gold = sc.player.gold;
        sc.select_row(2);
        assert_eq!(sc.menu_cursor, 2);
        assert_eq!(sc.menu_scroll_top, 0, "可视窗内不滚");
        assert_eq!(sc.player.hp, hp);
        assert_eq!(sc.player.gold, gold);
        sc.menu_scroll_top = 0;
        sc.menu_visible_bottom = 4;
        sc.select_row(5);
        assert_eq!(sc.menu_cursor, 5);
        assert_eq!(sc.menu_visible_bottom, 6, "滚窗跟随下缘");
        assert_eq!(sc.menu_scroll_top, 2);
        sc.select_row(0);
        assert_eq!(sc.menu_scroll_top, 0, "滚窗跟随上缘");
        assert_eq!(sc.menu_visible_bottom, 4);
    }

    /// 设置面板（OptionList，2026-10-18 用户反馈面）：行带单源 + 命中 ±1px +
    /// confirm_key=-5（run case 16 翻转键）+ select_row 只动高亮
    #[test]
    fn options_panel_pointer_hit_and_select() {
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18)); // Modern 面（清屏门控域）
        sc.game_mode = GameMode::About;
        let list = crate::menu_family::OptionList::new(true, false);
        sc.options = Some(list);
        let font = crate::paint::paint_font();
        let row_h = font.height() + 8;
        // 期望=字面常量锚（F-1 口径：禁被测自身作期望）——OptionList::add 累加式：
        // box_h=32+2*row_h（a.java:9911-9926）；行首=盒顶+16（paint case 16）
        // > 【2026-10-26 更正】盒顶取 **layout 画布**高（本测 Modern 18×18）——旧期望取
        // > SCREEN_H/W 把"Modern 面板留在经典左上区"钉成了正确（竖屏实机截图实证，用户报
        // > "存档UI没有居中"；menu_family::modern_center_tests）。
        let (lw, lh) = (sc.layout.screen_w, sc.layout.screen_h);
        assert_eq!((lw, lh), (18 * crate::layout::CELL_PX, 18 * crate::layout::CELL_PX), "Modern 18×18 格");
        let box_h = 32 + 2 * row_h;
        let y0 = (lh - box_h) / 2 + 16;
        let x = lw / 2; // 盒内任一 x（行带命中与 x 无关，盒外另测）
        assert_eq!(
            sc.pointer_hit(x, y0 + 1),
            PointerHit::Row { index: 0, confirm_key: -5 },
            "行0 顶像素归本行"
        );
        assert_eq!(sc.pointer_hit(x, y0 - 1), PointerHit::None, "行0 顶-1 不命中");
        assert_eq!(
            sc.pointer_hit(x, y0 + row_h),
            PointerHit::Row { index: 1, confirm_key: -5 },
            "行界像素归下行"
        );
        // select_row 只动高亮
        let hp = sc.player.hp;
        sc.select_row(1);
        if let Some(l) = sc.options.as_ref() {
            assert_eq!(l.highlight_index(), 1);
        } else {
            panic!();
        }
        assert_eq!(sc.player.hp, hp, "他态零变化");
    }

    /// 【STAGE-B-006】游戏内设置页：壳层附加行排在第 3-5 行；切换只改 `shell_options`（声音 / 小地图 / SKY_WAR 快照 /
    /// 脏位不变）；高亮在 5 行内环绕；-5 / -4 / -3 与"点行 → 选中 → 确认"都只翻附加行（复核 T1-F2）。
    #[test]
    fn in_game_settings_shell_rows_toggle_only_shell_options() {
        let rows: Vec<crate::menu_family::ShellOption> = [(1u8, "开屏画面", false), (2, "按键震动", true), (3, "按键音", true)]
            .into_iter()
            .map(|(id, l, checked)| crate::menu_family::ShellOption { id, label: l.encode_utf16().collect(), checked })
            .collect();
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18));
        sc.build_minimap();
        sc.shell_options = rows.clone();
        sc.menu_return_mode = GameMode::InGameMenu;
        sc.enter_menu16();
        sc.game_mode = GameMode::About;
        let hl = |sc: &super::GameScene| sc.options.as_ref().unwrap().highlight_index();
        assert_eq!(sc.options.as_ref().unwrap().len(), 5, "原两行 + 三行附加行");
        for want in [1, 2, 3, 4, 0] {
            sc.key_value = -2;
            sc.tick_menu16();
            assert_eq!(hl(&sc), want, "下移逐行走完一圈（第 5 行后回第 1 行）");
        }
        for want in [4, 3, 2, 1, 0] {
            sc.key_value = -1;
            sc.tick_menu16();
            assert_eq!(hl(&sc), want, "上移逐行走完一圈（第 1 行后到第 5 行）");
        }
        // 不变量逐键核对（复核 T2-F1：偶数次翻转会抵消布尔旁写，只在末尾断言抓不到）
        let inv = |sc: &super::GameScene| {
            let o = sc.options.as_ref().unwrap();
            (
                sc.bgm.clone(),
                sc.minimap_enabled,
                sc.minimap.is_some(),
                sc.global_snapshot(),
                sc.global_dirty,
                o.labels.clone(),
                o.checked.clone(),
            )
        };
        let before = inv(&sc);
        assert!(before.2, "前置：小地图已建");
        sc.options.as_mut().unwrap().set_highlight(3); // 按键震动
        for (k, expect) in [(-5, false), (-4, true), (-3, false)] {
            sc.key_value = k;
            sc.tick_menu16();
            assert_eq!(sc.shell_options[1].checked, expect, "键 {k}：按键震动行翻转");
            assert_eq!(sc.options.as_ref().unwrap().shell_rows()[1].checked, expect, "键 {k}：面板同步");
            assert_eq!(inv(&sc), before, "键 {k}：声音 / 小地图 / SKY_WAR / 脏位 / 原两行不变");
        }
        let (ry, rh) = sc.options.as_ref().unwrap().row_band(3, &sc.layout);
        assert_eq!(sc.pointer_hit(sc.layout.screen_w / 2, ry + rh / 2), PointerHit::Row { index: 3, confirm_key: -5 });
        sc.select_row(3);
        sc.key_value = -5;
        sc.tick_menu16();
        assert!(sc.shell_options[1].checked, "点行确认：按键震动行翻回开");
        assert_eq!((&sc.shell_options[0], &sc.shell_options[2]), (&rows[0], &rows[2]), "其余附加行不动");
        assert_eq!(inv(&sc), before, "点行确认：声音 / 小地图 / SKY_WAR / 脏位 / 原两行不变");
        sc.select_row(1);
        sc.key_value = -5;
        sc.tick_menu16();
        assert!(!sc.options.as_ref().unwrap().checked[1] && sc.minimap.is_none(), "原行为：小地图行照旧关小地图");
    }

    /// 存档槽列表（SlotSelect 面，2026-10-18 用户反馈）：行带单源 + 命中 ±1px +
    /// confirm=-5（run case 7：-6/-5 同义）+ set_cursor 只动光标+滚窗
    #[test]
    fn slot_list_pointer_hit_and_select() {
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18)); // Modern 面（清屏门控域）
        sc.game_mode = GameMode::SaveSlot;
        let font = crate::paint::paint_font();
        let lh = font.height() + 4;
        // > 【2026-10-26 更正】行首=盒顶+16 的盒顶取 **layout 画布**高（同上 options 测更正）
        let (cw, ch) = (sc.layout.screen_w, sc.layout.screen_h);
        assert_eq!((cw, ch), (18 * crate::layout::CELL_PX, 18 * crate::layout::CELL_PX), "Modern 18×18 格");
        let y0 = (ch - sc.slot.content_h) / 2 + 16; // 字面锚：行首=盒顶+16
        let x = cw / 2;
        assert_eq!(
            sc.pointer_hit(x, y0 + 1),
            PointerHit::Row { index: 0, confirm_key: -5 },
            "槽0 顶像素归本行"
        );
        assert_eq!(sc.pointer_hit(x, y0 - 1), PointerHit::None, "槽0 顶-1 不命中");
        assert_eq!(
            sc.pointer_hit(x, y0 + lh),
            PointerHit::Row { index: 1, confirm_key: -5 },
            "行界像素归下槽"
        );
        let hp = sc.player.hp;
        sc.select_row(1);
        assert_eq!(sc.slot.cursor, 1);
        assert_eq!(sc.player.hp, hp, "他态零变化");
    }

    /// STAGE-B-001（2026-10-18 用户裁定）**modern 预设面**：boot 不再进 SoundAsk
    /// （mode 21），直入 MainMenu；默认开声；标题 BGM 请求=track2（防无声标题屏）。
    /// 2026-10-19 门控化（lead 裁定）：001 行为只在 modern 生效，classic 走原版时间轴
    /// （对测见 stage_b001_classic_keeps_sound_ask）。
    #[test]
    fn stage_b001_boot_skips_sound_ask() {
        use crate::paint::BootMachine;
        let mut m = BootMachine::new(
            crate::render::ArgbImage::create(1, 1),
            crate::render::ArgbImage::create(1, 1),
            (0..8).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..25).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..2).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            crate::render::ArgbImage::create(1, 1),
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
        );
        // 001 门控面 = modern 预设（classic 预设保留开机询问）
        m.layout = crate::layout::Layout::modern(13, 13);
        // logo 相位推进（paint 侧计数——直接驱动 logo 尾转出）
        for _ in 0..400 {
            m.tick(0);
            if m.mode != GameMode::Boot {
                break;
            }
            m.paints = m.paints.wrapping_add(1); // paint 计数推进（boot-replay 同构）
        }
        assert_eq!(
            m.mode,
            GameMode::MainMenu,
            "STAGE-B-001（modern）：boot 直入 MainMenu（不得停留 SoundAsk）"
        );
        // 默认开声（BgmState::new=true）
        assert!(m.bgm.sound_enabled, "STAGE-B-001（modern）：默认开声");
        // 标题 BGM 请求（硬要求 3：防无声标题屏）
        assert_eq!(m.bgm_request(), Some(2), "STAGE-B-001（modern）：标题 BGM=track2");
    }

    /// STAGE-B-001 门控化对测（2026-10-19）：classic 预设保留开机声音询问 = 原版
    /// 时间轴（a.java:3371-3373 → SoundAsk + case21 入场 soundEnabled=false a.java:4484）
    /// ——64 枚帧夹具 Java 参照面零偏离的命门回归。
    #[test]
    fn stage_b001_classic_keeps_sound_ask() {
        use crate::paint::BootMachine;
        let mut m = BootMachine::new(
            crate::render::ArgbImage::create(1, 1),
            crate::render::ArgbImage::create(1, 1),
            (0..8).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..25).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..2).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            crate::render::ArgbImage::create(1, 1),
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
        );
        // 默认布局 = classic（BootMachine::new 内建）——不再覆写
        for _ in 0..400 {
            m.tick(0);
            if m.mode != GameMode::Boot {
                break;
            }
            m.paints = m.paints.wrapping_add(1);
        }
        assert_eq!(
            m.mode,
            GameMode::SoundAsk,
            "classic 预设：boot 进 SoundAsk（原版时间轴，a.java:3371-3373）"
        );
        assert_eq!(m.softkeys, (1, 2), "case21 入场软键 (1,2)（a.java:4482-4478）");
        assert!(!m.bgm.sound_enabled, "case21 入场 soundEnabled=false（a.java:4484）");
    }

    /// 游戏内菜单"退出"（kind7）→ 回标题（Java gameMode=1 跨机语义）——
    /// 2026-10-18 用户实报修复：原 other 臂空转＝退出无反应。
    #[test]
    fn ingame_menu_exit_returns_to_title() {
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18)); // Modern 面（清屏门控域）
        sc.game_mode = GameMode::InGameMenu;
        sc.menu_kinds = vec![7]; // 退出项
        sc.menu_item_count = 1;
        sc.menu_cursor = 0;
        sc.key_value = -6; // 确认（菜单族激活键）
        sc.tick(0, &[]);
        assert_eq!(
            sc.game_mode,
            GameMode::MainMenu,
            "kind7 退出须置 MainMenu（跨机信号→shell 换回 BootMachine）"
        );
        // 信号持久性：后续拍不得洗掉换机信号（2026-10-18 用户实报追查）
        sc.tick(1, &[]);
        assert_eq!(sc.game_mode, GameMode::MainMenu, "换机信号须持久");
    }

    /// 退出链无头复现（BUG-020 复发追查 2026-10-18）：标题→退出项→确认→
    /// LogoAnim 退场动画→counter>70→finished。断在哪段一测即知。
    #[test]
    fn boot_exit_chain_reaches_finished() {
        use crate::paint::BootMachine;
        let mut m = BootMachine::new(
            crate::render::ArgbImage::create(1, 1),
            crate::render::ArgbImage::create(1, 1),
            (0..8).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..25).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..2).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            crate::render::ArgbImage::create(1, 1),
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
        );
        // 开机相位推进在 paint 侧（boot-replay 口径）——探针直置 MainMenu 态验退出链本体
        m.mode = GameMode::MainMenu;
        m.title = Some(crate::title::TitleMachine::new());
        // 退出项 = cursor 末位（menu_kinds 尾 kind5）；下移键=-2
        for _ in 0..10 {
            m.tick(-2);
        }
        // 光标推到末位（kind5=LogoAnim=退出）
        let n = if let Some(t2) = m.title.as_ref() { t2.menu_kinds.len() } else { 0 };
        if let Some(t2) = m.title.as_mut() {
            t2.cursor = n as i32 - 1;
        }
        m.tick(-5);
        println!("确认后 mode={:?}", m.mode);
        for i in 0..80 {
            m.tick(0);
            if m.finished {
                println!("第 {i} 拍 finished=true");
                break;
            }
        }
        assert!(m.finished, "退出链断裂：80 拍后 finished 仍为 false（mode={:?}）", m.mode);
    }

    /// 标题侧设置面板（BootMachine::About，C-2 补面）：命中 + confirm=-5 + 只动高亮
    #[test]
    fn boot_options_pointer_hit_and_select() {
        use crate::paint::BootMachine;
        let mut m = BootMachine::new(
            crate::render::ArgbImage::create(1, 1),
            crate::render::ArgbImage::create(1, 1),
            (0..8).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..25).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..2).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            crate::render::ArgbImage::create(1, 1),
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
        );
        m.mode = GameMode::About;
        let list = crate::menu_family::OptionList::new(true, false);
        m.options = Some(list);
        let font = crate::paint::paint_font();
        let row_h = font.height() + 8;
        let y0 = (SCREEN_H - (32 + 2 * row_h)) / 2 + 16; // 字面锚（禁自证）
        assert_eq!(
            m.pointer_hit(SCREEN_W / 2, y0 + 2),
            PointerHit::Row { index: 0, confirm_key: -5 }
        );
        assert_eq!(
            m.pointer_hit(SCREEN_W / 2, y0 + row_h + 2),
            PointerHit::Row { index: 1, confirm_key: -5 }
        );
        assert_eq!(m.pointer_hit(SCREEN_W / 2, y0 - 1), PointerHit::None);
        m.select_row(1);
        if let Some(l) = m.options.as_ref() {
            assert_eq!(l.highlight_index(), 1, "标题侧选行=只动高亮");
        } else {
            panic!();
        }
    }

    /// 【2026-10-26 竖屏实机】Modern 背包：格阵须在盒框内（盒=layout 居中，格原点=盒左+20、
    /// 盒顶+31+fh——paint 累积式）。实机 12×16 格竖屏：格阵画在经典 240×320 左上区、
    /// 盒框却居中 ⇒ 格阵与盒框脱节（截图实证）。翻页箭头 x 同源（盒右-22）。
    #[test]
    fn modern_item_grid_sits_inside_box_frame() {
        for layout in [crate::layout::Layout::modern(12, 16), crate::layout::Layout::modern(18, 18)] {
            let mut sc = scene((2, 2), &[]);
            sc.apply_layout(layout.clone());
            let bx = (layout.screen_w - sc.item_box_w) >> 1;
            let by = (layout.screen_h - sc.item_box_h) >> 1;
            let fh = crate::paint::paint_font().height;
            assert_eq!(
                sc.item_cell_origin(),
                (bx + 20, by + 31 + fh),
                "{}×{}：格原点=盒左+20 / 盒顶+31+fh（与 paint 盒框同源）",
                layout.screen_w,
                layout.screen_h
            );
        }
        // classic：盒=(240-w)/2 ⇒ 原值零偏离
        let sc = scene((2, 2), &[]);
        assert_eq!(
            sc.item_cell_origin(),
            ((SCREEN_W - sc.item_box_w) / 2 + 20, (SCREEN_H - sc.item_box_h) / 2 + 31 + crate::paint::paint_font().height),
            "classic 原值"
        );
    }

    /// 背包栅格：几何单源 + 命中 + (a) confirm_key=-6 + select_row
    #[test]
    fn item_grid_pointer_hit_and_select() {
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18)); // Modern 面（清屏门控域）
        sc.game_mode = GameMode::ItemPanel;
        sc.item_cols = 4;
        sc.item_rows_total = 3;
        sc.item_scroll_top = 0;
        sc.item_scroll_bottom = 3;
        sc.item_cursor = 0;
        // 期望取字面常量（F-1：禁被测自身作期望）——锚 paint_item_panel 累积式
        //（盒顶+16+fh+4+1+10 = 盒顶+31+fh；(320-盒高)/2=…+31+14）
        // > 【2026-10-26 更正】盒顶/盒左取 **layout 画布**（本测 Modern 18×18；paint 盒框早已
        // > layout 居中）——旧期望字面 SCREEN_W/H 把"格阵与盒框脱节"的缺陷钉成了正确
        // >（竖屏实机截图实证，modern_item_grid_sits_inside_box_frame）。
        let (lw, lh) = (sc.layout.screen_w, sc.layout.screen_h);
        assert_eq!((lw, lh), (18 * crate::layout::CELL_PX, 18 * crate::layout::CELL_PX), "Modern 18×18 格");
        let ox = (lw - sc.item_box_w) / 2 + 20;
        let oy = (lh - sc.item_box_h) / 2 + 31 + crate::paint::paint_font().height;
        assert_eq!(sc.item_cell_origin(), (ox, oy), "格原点与 paint 累积式同式");
        // 顶像素 ±1px 归属锚（栅格版，照菜单侧规格）
        assert_eq!(sc.pointer_hit(ox + 16, oy), PointerHit::Row { index: 0, confirm_key: -6 }, "格顶像素归本格");
        assert_eq!(sc.pointer_hit(ox + 16, oy - 1), PointerHit::None, "格顶-1 不命中");
        assert_eq!(sc.pointer_hit(ox + 16, oy + 16), PointerHit::Row { index: 0, confirm_key: -6 });
        assert_eq!(
            sc.pointer_hit(ox + 40 + 16, oy + 40 + 16),
            PointerHit::Row { index: 5, confirm_key: -6 }
        );
        assert_eq!(sc.pointer_hit(ox - 5, oy + 16), PointerHit::None);
        sc.select_row(5);
        assert_eq!(sc.item_cursor, 5);
        assert_eq!(sc.item_scroll_top, 0);
        // 越窗行 → 滚窗同步
        sc.select_row(11);
        assert_eq!(sc.item_cursor, 11);
        assert_eq!(sc.item_scroll_bottom, 3, "末行仍全窗");
    }
}

/// 三处静默简化保真修复 + 读档旗标（completeness §3.4，2026-10-15）的单测
/// （⑤ 存档 blob 布局见 slot_equip_scene_tests::save_blob_layout_matches_java_and_roundtrips）：
/// ① addItemToItemStack 写 itemUseTargetType（a.java:7732）；② m_119 entityCount==0
/// 存 null（a.java:9383-9384）；③ m_122 floor2 恢复特例读 scriptLineFlags[26]
/// （a.java:9643-9646）；④ loadFloorSave 读回 scriptLineFlags 前 scriptLineCount 项
/// （a.java:9285-9291）。
#[cfg(test)]
mod fidelity_fix_tests {
    use crate::render::FontMetrics;
    use super::scene_test_support::{images, maplv, numbers, scene};
    use super::*;

    /// ① 商人赠物路径（a.java:7333/7336 直调 addItemToItemStack，无前置写）：
    /// itemUseTargetType 由本函数自写，kind1 浮层图标/名随之取型 19。
    #[test]
    fn add_item_to_stack_writes_item_use_target_type() {
        let mut sc = scene((3, 3), &[]);
        sc.overlay_kind_item = 0;
        sc.add_item_to_stack(19);
        assert_eq!(sc.overlay_kind_item, 19);
        assert_eq!(sc.overlay_kind, 1);
        assert_eq!(sc.overlay_item_name, sc.object_type_name(19), "浮层名取 itemUseTargetType");
        assert_eq!(sc.player.item_types[0], 19);
    }

    /// ② m_119：槽先置 null，仅 entityCount>0 才写——空表 = None（覆盖旧值），
    /// 有实体但全死 = Some([00 00])。
    #[test]
    fn save_floor_state_null_only_when_entity_table_empty() {
        let mut sc = scene((3, 3), &[]);
        assert_eq!(sc.entities.count, 0);
        sc.saved_floor_states[2] = Some(vec![9, 9]);
        sc.save_floor_state();
        assert_eq!(sc.saved_floor_states[2], None, "entityCount==0 → null（a.java:9383-9384）");

        let mut sc = scene((3, 3), &[(43, 5, 3)]);
        let e = sc.entity_at_cell(5, 3, 43) as usize;
        entity::remove(&mut sc.entities, e);
        sc.save_floor_state();
        assert_eq!(sc.saved_floor_states[2], Some(vec![0, 0]), "有实体全死 → 计数 0 的 2 字节块");
    }

    /// ③ floor2 恢复特例：`!flags[26] && findBlockingEntity(11,4,84) < 0` → spawnEntity(84,352,128,0)。
    #[test]
    fn floor2_restore_respawns_84_only_while_script26_pending() {
        for (flag26, respawn) in [(false, true), (true, false)] {
            let mut sc = scene((3, 3), &[]);
            assert_eq!(sc.floor, 2);
            sc.script_line_flags[26] = flag26;
            sc.saved_floor_states[2] = Some(vec![0, 0]); // 恢复分支（0 实体）
            sc.load_floor_data(&maplv(), 3 << 5, 3 << 5, &[], sc.floor).unwrap();
            assert_eq!(sc.entity_at_cell(11, 4, 84) >= 0, respawn, "flags[26]={flag26}");
        }
        // 非 2 层不补生
        let mut sc = scene((3, 3), &[]);
        sc.floor = 3;
        sc.saved_floor_states[3] = Some(vec![0, 0]);
        sc.load_floor_data(&maplv(), 3 << 5, 3 << 5, &[], sc.floor).unwrap();
        assert!(sc.entity_at_cell(11, 4, 84) < 0);
    }

    /// ③b floor12 恢复特例（a.java:9653-9661）：型 78 两处 param 改写
    /// `lo | door13_hi[hi]<<8`（6|door13[5]<<8=1542、5|door13[4]<<8=773）。
    #[test]
    fn floor12_restore_rewrites_type78_params() {
        fn blob(recs: &[(u8, i32, i32, i16)]) -> Vec<u8> {
            let mut v = (recs.len() as u16).to_be_bytes().to_vec();
            for &(t, px, py, param) in recs {
                v.push(t);
                v.extend_from_slice(&(px as u16).to_be_bytes());
                v.extend_from_slice(&(py as u16).to_be_bytes());
                v.push(1);
                v.extend_from_slice(&(param as u16).to_be_bytes());
            }
            v
        }
        let mut sc = scene((3, 3), &[]);
        sc.floor = 12;
        sc.saved_floor_states[12] = Some(blob(&[
            (78, 1 << 5, 1 << 5, 9),
            (78, 11 << 5, 1 << 5, 9),
            (78, 5 << 5, 5 << 5, 9),
        ]));
        sc.load_floor_data(&maplv(), 3 << 5, 3 << 5, &[], 12).unwrap();
        let a = sc.entity_at_cell(1, 1, 78) as usize;
        let b = sc.entity_at_cell(11, 1, 78) as usize;
        let c = sc.entity_at_cell(5, 5, 78) as usize;
        // 值=按表索引现算（door13_hi[5]=3、[4]=6——a.java:902 表序逐字）
        assert_eq!(
            sc.entities.items[a].param,
            6 | ((DOOR13_HI[5] as i16) << 8),
            "(1,1) → 6|door13_hi[5]<<8（=774）"
        );
        assert_eq!(
            sc.entities.items[b].param,
            5 | ((DOOR13_HI[4] as i16) << 8),
            "(11,1) → 5|door13_hi[4]<<8（=1541）"
        );
        assert_eq!(sc.entities.items[c].param, 9, "无特例格的 78 参数不动");
    }

    /// ③c floor39 恢复特例（a.java:9664-9671）：型7@(11,1) → removeGroup +
    /// spawn(8,11,1,38)、型8@(11,11) → removeGroup + spawn(7,11,11,40)。
    /// **像素字面量保真**：spawn 实参=像素 ⇒ 两补生均落 cell(0,0)（javap 1446-1454/
    /// 1483-1492 常量逐字）——原版实况，D4 不"修正"。
    #[test]
    fn floor39_restore_swaps_types_with_pixel_literal_spawns() {
        fn blob(recs: &[(u8, i32, i32, i16)]) -> Vec<u8> {
            let mut v = (recs.len() as u16).to_be_bytes().to_vec();
            for &(t, px, py, param) in recs {
                v.push(t);
                v.extend_from_slice(&(px as u16).to_be_bytes());
                v.extend_from_slice(&(py as u16).to_be_bytes());
                v.push(1);
                v.extend_from_slice(&(param as u16).to_be_bytes());
            }
            v
        }
        let mut sc = scene((3, 3), &[]);
        sc.floor = 39;
        sc.saved_floor_states[39] = Some(blob(&[
            (7, 11 << 5, 1 << 5, 0),
            (8, 11 << 5, 11 << 5, 0),
            (43, 5 << 5, 5 << 5, 0),
        ]));
        sc.load_floor_data(&maplv(), 3 << 5, 3 << 5, &[], 39).unwrap();
        assert!(sc.entity_at_cell(11, 1, 7) < 0, "(11,1) 型7 已摘");
        assert!(sc.entity_at_cell(11, 11, 8) < 0, "(11,11) 型8 已摘");
        let p = sc.entity_at_cell(0, 0, 8);
        assert!(p >= 0, "补生 8 落 cell(0,0)（像素 11,1 原版实况）");
        assert_eq!(sc.entities.items[p as usize].param, 38);
        let q = sc.entity_at_cell(0, 0, 7);
        assert!(q >= 0, "补生 7 落 cell(0,0)（像素 11,11 原版实况）");
        assert_eq!(sc.entities.items[q as usize].param, 40);
        assert!(sc.entity_at_cell(5, 5, 43) >= 0, "无关实体不动");
    }

    /// ③d DOOR13_HI 与 game-data door13 表同值（两源对账——door13_hi
    /// a.java:902 `new byte[]{2,3,3,3,6,3,3,6,3,3,3,3,3}`）。
    #[test]
    fn door13_hi_table_matches_fixture() {
        let fixture: Vec<i32> = numbers(include_str!("../tests/fixtures/door13.txt"));
        assert_eq!(DOOR13_HI.len(), fixture.len(), "表长一致");
        for (i, &v) in fixture.iter().enumerate() {
            assert_eq!(DOOR13_HI[i] as i32, v, "door13[{i}] 两源一致");
        }
    }

    /// ④ 读档旗标：前 scriptLineCount（68）项读回；之后的字节 Java skip（不读）。
    /// blob = preset-floor2-mov 的 MOT_L0（Java 侧同一预置实跑过，旗标段 [0]=1）。
    #[test]
    fn load_restores_script_flags_prefix_only() {
        let stores = crate::save::parse_preset(include_str!("../tests/fixtures/preset-floor2-mov.preset"));
        let mut blob = stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).unwrap().1.clone();
        // 旗标段起点 = 37 + 2×日志对数（a.java:9258-9288 读序；本 blob 日志 0 对）
        assert_eq!(i16::from_be_bytes([blob[35], blob[36]]), 0);
        assert_eq!(blob[37], 1);
        blob[37 + 67] = 1; // scriptLineCount-1：读
        blob[37 + 100] = 1; // ≥ scriptLineCount：Java skip（a.java:9289-9291）
        let save = crate::save::parse_floor_save(&blob, 0).unwrap();
        assert_eq!(save.script_flags.len(), 128);
        assert!(save.script_flags[0] && save.script_flags[67] && save.script_flags[100]);
        let width_table = numbers(include_str!("../tests/fixtures/type_width_px.txt"));
        let walk = numbers(include_str!("../tests/fixtures/tile_walkability.txt"));
        let sc =
            load_floor_from_save(&maplv(), &[], &width_table, &walk, images(), vec![None; 56], &save)
                .unwrap();
        assert_eq!(sc.dialog_data.scripts.len(), 68, "scriptLineCount = levelScriptLines.length");
        assert!(sc.script_line_flags[0] && sc.script_line_flags[67]);
        assert!(!sc.script_line_flags[100], "scriptLineCount 之后被 skip");
        assert_eq!(sc.script_line_flags.iter().filter(|f| **f).count(), 2);
    }
}

/// openWallOnRoute / relocateSlotsAtRouteEnd（dp2 实体路径缓动，a.java:8738-8904）与 MOV 五参臂（a.java:8274-8309）
/// 的逐拍单测：合成网格见 scene_test_support，直接驱动 execute_script_instruction /
/// tick_entity_route，逐拍断言 Java 语义表的每一格。场景级全帧对拍见
/// tests/mov_route_scene.rs（floor2 script1/2 + floor40 script12 伏击）。
#[cfg(test)]
mod entity_route_tests {
    use crate::render::FontMetrics;
    use super::scene_test_support::{scene, ROW};
    use super::*;

    /// 装载单行脚本并执行一条指令（dp0 一拍的 executeScriptInstruction）。
    fn exec(sc: &mut GameScene, line: &str) {
        sc.game_mode = GameMode::Dialog;
        sc.dialog_phase = 0;
        sc.script_line = line.to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();
    }

    fn slots(sc: &GameScene, x: i32, y: i32) -> Vec<usize> {
        let ct = sc.grid.cell_type[y as usize][x as usize] as usize;
        (0..sc.grid.capacity[ct] as usize).map(|s| sc.grid.slots[ct][s] as usize - 1).collect()
    }

    /// 型72 两步右行：MOV 臂字段（a.java:8289-8299）+ openWallOnRoute 每拍
    /// (routePhase, f_byte_19, 像素, routeProgress, 相机目标, 帧) 序列 + 步尽 relocateSlotsAtRouteEnd/收尾。
    #[test]
    fn route_72_tick_sequence() {
        let mut sc = scene((9, ROW), &[(72, 3, ROW)]);
        let e = sc.entity_at_cell(3, ROW, 72);
        assert!(e >= 0);
        let eu = e as usize;
        sc.route_phase = 7; // 脏值：MOV 必须复位为 0（a.java:8291）
        exec(&mut sc, "MOV_72_3_3_5_3 \u{1e}");
        assert_eq!(sc.dialog_phase, 2);
        assert_eq!(sc.route_entity, e);
        assert_eq!(sc.route_phase, 0);
        assert_eq!(sc.route_start, (3, ROW));
        assert_eq!(sc.dialog_target, (5, ROW));
        assert_eq!(sc.entities.items[eu].anim_row, 1, "型72 置动画行 1（a.java:8293-8294）");
        assert_eq!(sc.walk_path_buffer, vec![2, 2]);
        assert_eq!(sc.walk_step_count, 2);
        assert_eq!(sc.script_cursor, "MOV_72_3_3_5_3 ".len(), "游标越过空格分隔符");

        // (routePhase, f_byte_19, px, routeProgress, cameraTargetX, frame, 余步) 逐拍
        let expect = [
            (1, 2, 96, 0, None, 0, 1),        // case0 弹第 1 步（不动）
            (1, 2, 104, 8, Some(104), 1, 1),  // case1 ×4
            (1, 2, 112, 16, Some(112), 2, 1),
            (1, 2, 120, 24, Some(120), 3, 1),
            (1, 2, 128, 0, Some(128), 0, 0),  // 满 32 归格 → 同拍递归弹第 2 步
            (1, 2, 136, 8, Some(136), 1, 0),
            (1, 2, 144, 16, Some(144), 2, 0),
            (1, 2, 152, 24, Some(152), 3, 0),
        ];
        for (i, &(phase, dir, px, prog, tgt, frame, left)) in expect.iter().enumerate() {
            sc.tick_entity_route();
            assert_eq!(
                (sc.route_phase, sc.route_dir, sc.entities.items[eu].px, sc.route_progress),
                (phase, dir, px, prog),
                "拍 {i}"
            );
            if let Some(t) = tgt {
                assert_eq!((sc.ease_target_x, sc.ease_target_y), (t, ROW << 5), "拍 {i} 相机目标=实体像素");
            }
            assert_eq!(sc.entities.items[eu].anim_frame, frame, "拍 {i} 动画行1帧（表长 4 归 0）");
            assert_eq!(sc.walk_step_count, left, "拍 {i} 余步");
            assert_eq!(sc.dialog_phase, 2);
            assert_eq!(slots(&sc, 3, ROW), vec![eu], "途中槽表不动（只在收尾 relocateSlotsAtRouteEnd）");
        }
        // 第 2 步归格 → 递归 → 步尽：relocateSlotsAtRouteEnd + dp0 + dispatchAnimFields + route_entity=-1
        sc.tick_entity_route();
        assert_eq!(sc.entities.items[eu].px, 160);
        assert_eq!((sc.dialog_phase, sc.route_phase, sc.route_entity), (0, 0, -1));
        assert_eq!((sc.entities.items[eu].anim_row, sc.entities.items[eu].anim_frame), (0, 0), "animRowByType[72]=0");
        assert!(slots(&sc, 3, ROW).is_empty(), "relocateSlotsAtRouteEnd 自起点格摘除");
        assert_eq!(slots(&sc, 5, ROW), vec![eu], "relocateSlotsAtRouteEnd 追加到当前格");
        assert_eq!(sc.entity_at_cell(5, ROW, 72), e, "findBlockingEntity 可在新格命中（DES 依赖）");
        // 行尾 → dp4（script 收尾门）
        sc.execute_script_instruction();
        assert_eq!(sc.dialog_phase, 4);
    }

    /// 途中战斗（a.java:8765-8779 + 8861-8868）：非 72 型下一格=玩家格且可胜 →
    /// 战斗态字段 + 敌先手；case5 tickBattle(false) 击杀收尾 → dp0（不 restep，
    /// walkPhase 留 5——Java 原样，待 mode3 tickBattle 复位）。
    #[test]
    fn route_enemy_battle_when_stepping_into_player() {
        let t = 43u8;
        let mut sc = scene((5, ROW), &[(t, 3, ROW)]);
        let e = sc.entity_at_cell(3, ROW, t as i32);
        let eu = e as usize;
        sc.player.hp = 500;
        sc.player.atk = 1000;
        sc.player.def = 0;
        let ti = (t - 41) as usize;
        let loss = sc.predict_hp_loss(t as i32, false);
        assert!(loss >= 0 && loss < 500, "夹具须可胜: {loss}");
        exec(&mut sc, "MOV_43_3_3_7_3 \u{1e}");
        assert_eq!(sc.walk_step_count, 4);
        assert_eq!(sc.entities.items[eu].anim_row, entity::ANIM_TABLE_IDX[t as usize] as i32, "非 72 不改动画行");
        for _ in 0..5 {
            sc.tick_entity_route(); // 弹第 1 步 + 4 拍步进（第 4 拍递归弹第 2 步 → 开战）
        }
        assert_eq!(sc.entities.items[eu].px, 4 << 5);
        assert_eq!(sc.route_phase, 5);
        assert_eq!(sc.walk_phase, 5);
        assert!(!sc.battle_active, "battleActiveFlag=false（a.java:8771）");
        assert_eq!(sc.battle_target, e);
        assert_eq!(sc.entities.items[eu].state, 5);
        assert!(!sc.battle_no_counter);
        assert_eq!(sc.battle_enemy_hp, sc.enemies.hp[ti], "缩放表 HP（a.java:8769，非 bump 战的基础表）");
        let strike = sc.enemies.atk[ti].max(0);
        assert_eq!(sc.player.hp, 500 - strike, "敌先手一击");
        assert_eq!(sc.walk_step_count, 2, "余步保留（战斗收尾不再消费）");
        // case5：1 拍击杀 + 6 拍死亡动画
        for k in 0..6 {
            sc.tick_entity_route();
            assert_eq!(sc.dialog_phase, 2, "收尾中不退 dp（拍 {k}）");
        }
        sc.tick_entity_route();
        assert_eq!((sc.dialog_phase, sc.route_phase, sc.route_entity), (0, 0, -1));
        assert_eq!(sc.entities.items[eu].state, 1, "markEntityRemoved");
        assert_eq!(sc.battle_target, -1);
        assert_eq!(sc.walk_phase, 5, "无 restep（battleActiveFlag=false）——walkPhase 不复位");
        assert_eq!(sc.player_cell_x, 5, "玩家未动");
    }

    /// 不可胜（a.java:8780-8783 + 8850-8854）：case0 置 walkPhase=1/朝向；实体照走进
    /// 玩家格；归格复判仍不可胜 → scriptWalkArmed=false、gameMode=3、walkPhase=1、
    /// 朝向=步向，递归继续弹下一步（dialogPhase 留 2，路径挂起）。
    #[test]
    fn route_enemy_unbeatable_hands_back_to_mode3() {
        let t = 43u8;
        let mut sc = scene((5, ROW), &[(t, 3, ROW)]);
        let eu = sc.entity_at_cell(3, ROW, t as i32) as usize;
        sc.player.atk = 1; // ≤ 敌防 ⇒ predictBattleHpLoss = -1
        assert!(sc.predict_hp_loss(t as i32, false) < 0);
        sc.facing = 0;
        exec(&mut sc, "MOV_43_3_3_7_3 \u{1e}");
        sc.script_walk_armed = true;
        for _ in 0..5 {
            sc.tick_entity_route();
        }
        // 第 2 步弹出（下一格 = 玩家格）：不可胜分支
        assert_eq!((sc.route_phase, sc.walk_phase, sc.facing), (1, 1, 2));
        assert_eq!(sc.game_mode, GameMode::Dialog);
        for _ in 0..4 {
            sc.tick_entity_route();
        }
        assert_eq!(sc.entities.items[eu].px, 5 << 5, "实体走进玩家格");
        assert!(!sc.script_walk_armed);
        assert_eq!((sc.game_mode, sc.walk_phase, sc.facing), (GameMode::Map, 1, 2));
        assert_eq!(sc.dialog_phase, 2, "dialogPhase 不变（路径挂起）");
        assert_eq!((sc.route_phase, sc.walk_step_count), (1, 1), "同拍递归已弹第 3 步");
        assert_eq!(sc.route_entity, eu as i32);
    }

    /// 下一格倒序扫 cat1 型11 → markEntityRemoved（a.java:8786-8802）；同为 cat1 的
    /// 门（型1）不受影响。
    #[test]
    fn route_opens_wall_type11_only() {
        let mut sc = scene((11, ROW), &[(72, 3, ROW), (11, 4, ROW), (1, 6, ROW)]);
        let wall = sc.entity_at_cell(4, ROW, 11) as usize;
        let door = sc.entity_at_cell(6, ROW, 1) as usize;
        exec(&mut sc, "MOV_72_3_3_7_3 \u{1e}");
        assert_eq!(sc.walk_step_count, 4, "路径穿实体格（canPathThroughEntity）");
        sc.tick_entity_route(); // 弹第 1 步 → 下一格 (4,3) 有墙
        assert_eq!(sc.entities.items[wall].state, 1);
        assert_eq!((sc.entities.items[wall].sprite_w, sc.entities.items[wall].sprite_h), (45, 57));
        assert_eq!((sc.entities.items[wall].anim_row, sc.entities.items[wall].anim_frame), (2, 0));
        for _ in 0..8 {
            sc.tick_entity_route(); // 第 1/2 步步进；第 2 步归格弹第 3 步 → 下一格 (6,3)
        }
        assert_eq!(sc.walk_step_count, 1);
        assert_eq!(sc.entities.items[door].state, 0, "门不开");
    }

    /// 实体未命中：route_entity 写 -1（a.java:8289）+ 同拍尾递归执行下一条（8301-8302）。
    #[test]
    fn mov_entity_missing_runs_next_instruction_same_tick() {
        let mut sc = scene((9, ROW), &[]);
        sc.route_entity = 0;
        sc.facing = 0;
        exec(&mut sc, "MOV_72_3_3_5_3 ROS_4_2 \u{1e}");
        assert_eq!(sc.route_entity, -1);
        assert_eq!(sc.facing, 2, "ROS_4_2 同拍已执行");
        assert_eq!(sc.dialog_phase, 0);
        assert_eq!(sc.script_cursor, "MOV_72_3_3_5_3 ROS_4_2 ".len());
    }

    /// 首参 ≤0 走三参玩家分支（a.java:8277/8288 `var55 > 0`）——负值不按五参解析。
    #[test]
    fn mov_nonpositive_first_param_is_player_walk() {
        let mut sc = scene((3, ROW), &[]);
        exec(&mut sc, "MOV_-1_5_3 \u{1e}");
        assert_eq!(sc.dialog_target, (5, ROW));
        assert_eq!(sc.dialog_phase, 3);
        assert!(sc.script_walk_armed);
        assert_eq!(sc.walk_path_buffer, vec![2, 2]);
    }
    /// 2.3（refactor-behavior-matrix §2）：**MOV-miss 尾递归**（a.java:8301-8302）——
    /// 五参 MOV 实体未命中 ⇒ 同拍尾递归执行下一条；MOV段游标**分支前推进**
    /// （a.java:8268），递归自下一条起（游标位移断言钉）。
    #[test]
    fn mov_miss_tail_recurses_into_next_instruction() {
        let line = "MOV_43_1_1_2_2 TAK_5_6 ";
        // miss 路径：(1,1) 无 type43 ⇒ 尾递归同拍执行 TAK_5_6
        let mut sc = scene((5, ROW), &[]);
        exec(&mut sc, line);
        assert_eq!(sc.dialog_page, 5, "尾递归同拍执行 TAK（page=5）");
        assert_eq!(sc.dialog_page_end, 6);
        assert_eq!(sc.script_cursor, line.len(), "游标越过 MOV段与 TAK 段（TAK 尾 +1 落行末）");
        // 命中对照：不递归——游标只到 TAK 前、相位=2（实体路线）
        let mut sc2 = scene((5, ROW), &[(43, 1, 1)]);
        exec(&mut sc2, line);
        assert_eq!(sc2.dialog_phase, 2, "命中 ⇒ 实体路线相位（不递归）");
        assert_eq!(sc2.script_cursor, "MOV_43_1_1_2_2 ".len(), "游标停在 TAK 前");
        assert_eq!(sc2.dialog_page, 0, "TAK 未执行");
    }

    /// 5.x（refactor-behavior-matrix §5）：**双面漂移防线**（DC-9 桥漏道具加成
    /// 前科同机制）——16 op 两面分派对照：harness（script.rs ScriptEngine）与
    /// 生产（execute_script_instruction）喂同一行后**游标推进必须一致**；未知
    /// 3 字母 op 两面同停摆（R-TODO-1b）。只比分派/游标（效果面由 fx 夹具+金样
    /// 钉生产面、script_golden 钉 harness 面——双面分列不混谈）。
    #[test]
    fn script_surfaces_op_dispatch_cursor_parity() {
        use super::scene_test_support::scene as mk_scene;
        use crate::script::{DialogueTable, NoopHost, ScriptEngine};
        let table = DialogueTable {
            scripts: vec!["TAK_9_9 ".into(), "TAK_9_9 ".into()],
            texts: vec!["x".into(); 256], // 对话页取 texts[page]（TAK 翻页越界防护）
            speakers: vec![0; 256],
            type_names: vec!["x".into(); 256],
            type_categories: vec![0; 96],
        };
        let cases: &[(&str, &str)] = &[
            ("TAK", "TAK_8_9 "), ("LAY", "LAY_2 "), ("GIN", "GIN_1_1000 "),
            ("RES", "RES_0 "), ("ADD", "ADD_1_2_3_4 "), ("SEE", "SEE_4_10_168_168_0 "),
            ("GUT", "GUT_1 "), ("MOV", "MOV_0_5_11 "), ("DES", "DES_70_5_8 "),
            ("SWD", "SWD "), ("CES", "CES_84_6_11 "), ("GLV", "GLV_0 "),
            ("END", "END_0 "), ("SMS", "SMS_0 "), ("ROS", "ROS_4_1 "), ("MVS", "MVS "),
        ];
        for &(op, line) in cases {
            let mut sc = mk_scene((5, ROW), &[]);
            exec(&mut sc, line);
            let prod = sc.script_cursor;
            let mut eng = ScriptEngine::new(&table, 0);
            eng.set_script_line_for_test(line);
            eng.state.script_cursor = 0;
            let mut host = NoopHost;
            eng.execute_script_instruction(&mut host);
            assert_eq!(eng.state.script_cursor, prod, "op {op} 两面游标推进一致（line={line:?}）");
        }
        // 未知 op 两面同停摆（R-TODO-1b）
        for line in ["ZZZ_0 ", "QQ "] {
            let mut sc = mk_scene((5, ROW), &[]);
            exec(&mut sc, line);
            assert_eq!(sc.script_cursor, 0, "生产面未知 op 停摆（{line:?}）");
            let mut eng = ScriptEngine::new(&table, 0);
            eng.set_script_line_for_test(line);
            eng.state.script_cursor = 0;
            let mut host = NoopHost;
            eng.execute_script_instruction(&mut host);
            assert_eq!(eng.state.script_cursor, 0, "harness 面未知 op 停摆（{line:?}）");
        }
    }

}

/// T2 WalkTo/打断 scene 级判据（touch-native-undo-plan §3.2/3.4 + task-11 打断终版）。
#[cfg(test)]
mod t2_walkto_route_tests {
    use super::scene_test_support::{scene, ROW};
    use super::*;

    fn run_until_settled(sc: &mut GameScene, limit: i64) {
        let wt = sc.width_table.clone();
        for fc in 0..limit {
            sc.tick(fc, &wt);
            if !sc.script_walk_armed && sc.walk_phase == 0 {
                return;
            }
        }
    }

    #[test]
    fn t2_walk_to_feeds_buffer_and_arrives() {
        // WalkTo 链：tap 格→walkPathBuffer 推进→终位断言+位移笔（整段起终点）
        let mut sc = scene((1, ROW), &[]);
        let plan = sc.walk_to(5, ROW);
        assert!(plan.reached, "直线路直达");
        assert_eq!(sc.walk_step_count, 4, "喂 walkPathBuffer 4 步");
        run_until_settled(&mut sc, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "终位=tap 格");
        assert_eq!(sc.undo.len(), 1, "段终位=位移笔一笔");
    }

    /// 【BUG-043】教程路径箭头（SEE flag=1：findPath 写 walkPathBuffer + pathArrowsVisible）下点地寻路：箭头须与用方向键
    /// 走同一条路线时完全一致（沿箭头走 ⇒ 逐格消费、余下箭头不动；偏离箭头 ⇒ 箭头熄灭，a.java:5748-5761 label216）。
    /// 旧行为：walk_to 覆写同一个 walkPathBuffer，箭头改画寻路路线（以玩家为起点 ⇒ "错位"），且 dp3 弹步不走 label216。
    fn arrows_scene(from: i32, to: i32) -> GameScene {
        let mut sc = scene((from, ROW), &[]);
        assert!(sc.find_path(from, ROW, to, ROW), "前置：箭头路径可达");
        sc.path_arrows_visible = true; // op_see flag=1 同款
        sc
    }

    /// 方向键逐格走到 `dest`（参照行为：每格按一下、松开、等走完这一格再按下一下）。
    fn key_walk(sc: &mut GameScene, code: i32, dest: (i32, i32)) {
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while (sc.player_cell_x, sc.player_cell_y) != dest && fc < 400 {
            let at = (sc.player_cell_x, sc.player_cell_y);
            sc.press_key(code);
            sc.tick(fc, &wt);
            fc += 1;
            sc.release_key();
            while ((sc.player_cell_x, sc.player_cell_y) == at || sc.walk_phase != 0) && fc < 400 {
                sc.tick(fc, &wt);
                fc += 1;
            }
        }
        for _ in 0..40 {
            sc.tick(fc, &wt);
            fc += 1;
        }
    }

    /// 点地走到 `dest`，逐拍记录箭头格。
    fn tap_walk(sc: &mut GameScene, dest: (i32, i32)) -> Vec<Vec<(i32, i32)>> {
        let wt = sc.width_table.clone();
        assert!(sc.walk_to(dest.0, dest.1).reached);
        let mut frames = vec![sc.path_arrow_cells()];
        for fc in 0..400 {
            sc.tick(fc, &wt);
            frames.push(sc.path_arrow_cells());
            if !sc.script_walk_armed && sc.walk_phase == 0 && !sc.route.is_open() {
                break;
            }
        }
        for fc in 400..440 {
            sc.tick(fc, &wt);
        }
        frames
    }

    #[test]
    fn bug043_tap_walk_along_arrows_consumes_like_keys() {
        let original = arrows_scene(1, 6).path_arrow_cells();
        assert_eq!(original, vec![(2, ROW), (3, ROW), (4, ROW), (5, ROW), (6, ROW)], "前置：箭头 (1)→(6)");
        let mut keys = arrows_scene(1, 6);
        key_walk(&mut keys, -4, (3, ROW));
        let mut touch = arrows_scene(1, 6);
        let frames = tap_walk(&mut touch, (3, ROW));
        assert_eq!((touch.player_cell_x, touch.player_cell_y), (3, ROW), "走到点中格");
        assert_eq!(touch.game_mode, GameMode::Map);
        assert_eq!(keys.path_arrow_cells(), vec![(4, ROW), (5, ROW), (6, ROW)], "参照：方向键沿箭头走两格 ⇒ 余下三格");
        assert_eq!(touch.path_arrow_cells(), keys.path_arrow_cells(), "点地 = 方向键");
        for (t, cells) in frames.iter().enumerate() {
            assert!(
                original.ends_with(cells),
                "第 {t} 拍箭头 {cells:?} 须是原箭头路径的尾段（不得改画寻路路线 / 错位）"
            );
        }
    }

    #[test]
    fn bug043_tap_walk_off_arrows_clears_them_like_keys() {
        let mut keys = arrows_scene(3, 6);
        key_walk(&mut keys, -3, (2, ROW));
        assert!(!keys.path_arrows_visible, "参照：方向键走离箭头 ⇒ 箭头熄灭（label216）");
        let mut touch = arrows_scene(3, 6);
        tap_walk(&mut touch, (2, ROW));
        assert_eq!((touch.player_cell_x, touch.player_cell_y), (2, ROW));
        assert_eq!(touch.path_arrows_visible, keys.path_arrows_visible, "点地 = 方向键");
        assert!(touch.path_arrow_cells().is_empty());
    }

    #[test]
    fn bug043_interrupted_tap_walk_keeps_remaining_arrows() {
        // 沿箭头点地走、途中按键打断（BUG-040 收尾）：已走的格消费，余下箭头仍按原路线画
        let original = arrows_scene(1, 6).path_arrow_cells();
        let mut sc = arrows_scene(1, 6);
        let wt = sc.width_table.clone();
        assert!(sc.walk_to(6, ROW).reached);
        let mut fc = 0;
        while sc.player_cell_x < 3 && fc < 400 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        sc.press_key(-1); // 上 = 撞墙，打断寻路
        sc.release_key();
        for _ in 0..60 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(!sc.route.is_open());
        let cells = sc.path_arrow_cells();
        assert!(sc.path_arrows_visible, "沿箭头走被打断：箭头仍在（方向键沿箭头走同样保留）");
        assert!(original.ends_with(&cells), "余下箭头 {cells:?} 须是原路线尾段");
        assert_eq!(cells.first().copied(), Some((sc.player_cell_x + 1, ROW)), "余下箭头紧接玩家格");
    }

    /// 箭头相关全部状态（对照用）：可见、绘制格、暂存是否已清、walkPathBuffer 有效段（label216 / 绘制的数据源）。
    fn arrow_state(sc: &GameScene) -> (bool, Vec<(i32, i32)>, bool, Vec<u8>) {
        let n = sc.walk_step_count.max(0) as usize;
        (sc.path_arrows_visible, sc.path_arrow_cells(), sc.walkto_arrows.is_none(), sc.walk_path_buffer[..n].to_vec())
    }

    /// 复核 bug043-R1：到达 / 打断两个出口收尾后，暂存已清、walkPathBuffer 有效段与方向键参照相同（不只是"画出来一样"）。
    #[test]
    fn bug043_exits_restore_buffer_like_keys() {
        let mut keys = arrows_scene(1, 6);
        key_walk(&mut keys, -4, (3, ROW));
        let mut arrived = arrows_scene(1, 6);
        tap_walk(&mut arrived, (3, ROW));
        assert_eq!(arrow_state(&arrived), arrow_state(&keys), "到达");
        assert!(arrow_state(&arrived).2, "到达：暂存已清");
        // 打断：寻路走到 3 时按"上"（撞墙），收尾停在 3
        let mut cut = arrows_scene(1, 6);
        let wt = cut.width_table.clone();
        assert!(cut.walk_to(6, ROW).reached);
        let mut fc = 0;
        while cut.player_cell_x < 3 && fc < 400 {
            cut.tick(fc, &wt);
            fc += 1;
        }
        cut.press_key(-1);
        cut.release_key();
        for _ in 0..60 {
            cut.tick(fc, &wt);
            fc += 1;
        }
        let at = (cut.player_cell_x, cut.player_cell_y);
        let mut keys2 = arrows_scene(1, 6);
        key_walk(&mut keys2, -4, at);
        assert_eq!(arrow_state(&cut), arrow_state(&keys2), "打断于 {at:?}");
    }

    /// 复核 bug043-R1：新路径（SEE / MOV 的 findPath）取代暂存——寻路途中剧情重算路径后不得再画旧箭头。
    #[test]
    fn bug043_find_path_drops_stash() {
        let mut sc = arrows_scene(1, 6);
        assert!(sc.walk_to(4, ROW).reached);
        assert!(sc.walkto_arrows.is_some(), "前置：箭头已暂存");
        assert!(sc.find_path(1, ROW, 2, ROW));
        assert!(sc.walkto_arrows.is_none(), "findPath 后暂存作废");
        assert_eq!(sc.path_arrow_cells(), vec![(2, ROW)], "箭头 = 新路径");
    }

    /// 复核 bug043-R2：点地走到剧情触发格（型 83，撞格触发）前一格并撞进去 = 方向键走过去再撞进去。
    #[test]
    fn bug043_bump_into_scene_trigger_like_keys() {
        let make = || {
            let mut sc = scene((1, ROW), &[(83, 5, ROW)]);
            sc.dialog_data.scripts = vec!["TAK_0_0 ".to_string()];
            assert!(sc.find_path(1, ROW, 6, ROW) || sc.find_path(1, ROW, 4, ROW), "前置：箭头路径");
            sc.path_arrows_visible = true;
            sc
        };
        let mut keys = make();
        key_walk(&mut keys, -4, (4, ROW));
        let wt = keys.width_table.clone();
        keys.press_key(-4); // 撞 83 ⇒ 剧情
        for fc in 0..20 {
            keys.tick(fc, &wt);
        }
        keys.release_key();
        let mut touch = make();
        let wt2 = touch.width_table.clone();
        let _ = touch.walk_to(5, ROW);
        for fc in 0..200 {
            touch.tick(fc, &wt2);
            if touch.game_mode == GameMode::Dialog && !touch.route.is_open() && touch.dialog_phase != 3 {
                break;
            }
        }
        for fc in 200..220 {
            touch.tick(fc, &wt2);
        }
        assert_eq!(keys.game_mode, GameMode::Dialog, "参照：撞 83 进剧情");
        assert_eq!(touch.game_mode, GameMode::Dialog, "点地：撞 83 进剧情");
        assert_eq!((touch.player_cell_x, touch.player_cell_y), (keys.player_cell_x, keys.player_cell_y));
        assert_eq!(arrow_state(&touch), arrow_state(&keys), "撞格触发剧情后的箭头状态");
    }

    /// 复核 bug043-R3：箭头一路指到锁门（无钥匙），点地沿箭头走到门前并撞门 = 方向键沿箭头走过去再撞门：
    /// 原版撞门清共用 walkStepCount ⇒ 余下箭头不再画。多步寻路的末步撞门走 advanceScriptWalk（步末连走）臂，
    /// 不经"浮层撞格熄箭头"臂——暂存须在撞门处同清。
    #[test]
    fn bug043_bump_locked_door_like_keys() {
        let make = || {
            let mut sc = scene((1, ROW), &[(1, 4, ROW)]); // 型 1 = 黄门（无钥匙）
            assert!(sc.find_path(1, ROW, 4, ROW), "前置：箭头路径 (1)→门(4)");
            sc.path_arrows_visible = true;
            assert_eq!(sc.path_arrow_cells(), vec![(2, ROW), (3, ROW), (4, ROW)]);
            sc
        };
        let mut keys = make();
        key_walk(&mut keys, -4, (3, ROW));
        assert_eq!(keys.path_arrow_cells(), vec![(4, ROW)], "参照前置：走到门前余下门格箭头");
        let wt = keys.width_table.clone();
        keys.press_key(-4); // 撞门
        keys.tick(500, &wt);
        keys.release_key();
        let mut touch = make();
        let wt2 = touch.width_table.clone();
        let _ = touch.walk_to(4, ROW);
        for fc in 0..200 {
            touch.tick(fc, &wt2);
            if touch.overlay_active {
                break;
            }
        }
        assert!(keys.overlay_active && touch.overlay_active, "两边都弹出\"没有钥匙\"提示");
        assert_eq!((touch.player_cell_x, touch.player_cell_y), (3, ROW), "停在门前");
        assert!(keys.path_arrow_cells().is_empty(), "参照：方向键撞门后余下箭头不再画（原版清 walkStepCount）");
        // 关掉提示后再走几拍：两边一致、箭头不复现
        for sc in [&mut keys, &mut touch] {
            sc.overlay_active = false;
            let wt = sc.width_table.clone();
            for fc in 600..640 {
                sc.tick(fc, &wt);
            }
        }
        assert!(touch.walkto_arrows.is_none(), "暂存已清");
        assert_eq!(touch.path_arrow_cells(), keys.path_arrow_cells(), "点地撞门 = 方向键撞门");
    }

    #[test]
    fn t2_interrupt_all_channels_clear_buffer() {
        // 打断全通道：press_key（方向键/软键同口）/新 WalkTo/探步入 皆清 walkPathBuffer
        let mut sc = scene((1, ROW), &[]);
        sc.walk_to(5, ROW);
        assert!(sc.walk_step_count > 0);
        sc.press_key(-4); // 任意输入（方向键）
        assert_eq!(sc.walk_step_count, 0, "press_key 打断=清 walkPathBuffer 停步");
        assert_eq!(sc.undo.len(), 0, "中断段于起点（未动）= 零长段，不入栈（STAGE-B-008）");

        sc.walk_to(5, ROW);
        assert!(sc.walk_step_count > 0);
        let plan2 = sc.walk_to(2, ROW); // 新触点=打断旧路线（手动优先）
        assert_eq!(sc.walk_step_count, 1, "新 WalkTo 重设路线");
        assert_eq!(plan2.dock, (2, ROW));
        sc.probe_step(0); // 探步入=打断
        assert_eq!(sc.walk_step_count, 0, "探步入打断=清 walkPathBuffer");
    }

    #[test]
    fn t2_classic_script_walk_unaffected_by_press_key() {
        // 保真红线：经典脚本行走（op_mov 路）press_key 不清缓冲（route 段未开放）
        let mut sc = scene((1, ROW), &[]);
        sc.find_path(1, ROW, 5, ROW); // MOV 同口（findPath 喂缓冲）
        let before = sc.walk_step_count;
        assert!(before > 0);
        sc.press_key(-4);
        assert_eq!(sc.walk_step_count, before, "经典面 key 链一行不动（缓冲不清）");
    }

    /// 【2026-10-26 用户实机"跳过教程后，嫦娥脚本行为异常"】壳层投键口径 = 每次按键/抬键前先
    /// `interrupt_route()`（android replay_scene_keys / win route_key / 触屏 Dispatcher）。
    /// 无 WalkTo 段时它必须零操作——旧实现无条件清 walkPathBuffer/walkStepCount ⇒ 剧情里
    /// 连点"确定"截断 MOV 实体路径（dp2）与玩家脚本行走（dp3）：嫦娥停在半路、后续
    /// `MOV_84_x_y_…` 找不到她而跳过、`DES_84_10_1` 落空（Java keyPressed 只写
    /// keyValue/keyHeldCode，a.java:4663-4676）。
    #[test]
    fn t2_interrupt_route_without_open_route_keeps_script_walk() {
        let mut sc = scene((1, ROW), &[]);
        sc.find_path(1, ROW, 5, ROW); // 脚本 MOV 同口
        let (steps, buf) = (sc.walk_step_count, sc.walk_path_buffer.clone());
        assert!(steps > 0 && !sc.route.is_open());
        sc.interrupt_route();
        assert_eq!((sc.walk_step_count, &sc.walk_path_buffer), (steps, &buf), "无寻路段：打断=零操作");
        // 寻路段被战斗/拾取挂起时（段已收、待续走），任意输入仍须打断续走
        sc.walk_to(5, ROW);
        sc.route_event_suspend();
        assert!(!sc.route.is_open() && sc.route.is_suspended() && sc.walk_step_count > 0);
        sc.interrupt_route();
        assert_eq!(sc.walk_step_count, 0, "挂起中的寻路余步：打断清除");
        assert!(!sc.route.is_suspended(), "打断=挂起解除");
    }

    /// 同一 bug 的第二入口：触屏点格寻路（WalkTo）走进剧情点 ⇒ 剧情接管时寻路段仍开放 ⇒
    /// 剧情中点"确定"（press_key 见段开放即打断）照样截断脚本行走，且 dp3 步尽臂
    /// `walk_step_count==0 && route.is_open()` 会把剧情中途切回 Map。剧情接管必须收束寻路段。
    #[test]
    fn t2_script_takeover_closes_walkto_route() {
        let mut sc = scene((1, ROW), &[]);
        sc.dialog_data.scripts = vec!["TAK_0_0 ".to_string()];
        sc.walk_to(5, ROW);
        assert!(sc.route.is_open());
        sc.load_level_script(0);
        assert!(!sc.route.is_open() && !sc.route.is_suspended(), "剧情接管 = 寻路段收束");
        sc.find_path(1, ROW, 5, ROW); // 剧情 MOV
        let steps = sc.walk_step_count;
        sc.press_key(-5);
        sc.interrupt_route();
        assert_eq!(sc.walk_step_count, steps, "剧情行走不受投键打断");
    }

    /// 【BUG-047】探步按探步方向走，不按旧朝向走：{旧朝向 × 4} × {探步方向 × 4}。
    /// 走廊 y=ROW 横向可走、上下是墙 ⇒ 右探到 (6,ROW)、左探到 (4,ROW)、上下探原地不动；朝向 = 探步方向。
    #[test]
    fn bug047_probe_step_moves_in_probe_direction_for_every_old_facing() {
        for old in 0..4 {
            for dir in 0u8..4 {
                let mut sc = scene((5, ROW), &[]);
                let wt = sc.width_table.clone();
                sc.facing = old;
                let accepted = sc.probe_step(dir);
                assert_eq!(accepted, matches!(dir, 2 | 3), "旧朝向 {old}、探步方向 {dir}：返回值 = 是否放行（壳层滑动撞墙即停的依据）");
                for fc in 0..20 {
                    sc.tick(fc, &wt);
                }
                let want = match dir {
                    2 => (6, ROW),
                    3 => (4, ROW),
                    _ => (5, ROW),
                };
                assert_eq!(
                    (sc.player_cell_x, sc.player_cell_y),
                    want,
                    "旧朝向 {old}、探步方向 {dir}：终点须是探步方向邻格或原地"
                );
                assert_eq!(
                    (sc.view.player_px, sc.view.player_py),
                    (want.0 << 5, want.1 << 5),
                    "旧朝向 {old}、探步方向 {dir}：像素位置与格一致（不得走进墙格半途）"
                );
                assert_eq!(sc.facing, dir as i32, "旧朝向 {old}、探步方向 {dir}：朝向 = 探步方向");
            }
        }
    }

    /// 【BUG-047】炼丹炉只在朝上走入时打开（scene.rs:5329-5341，5333 判 facing == 1）：
    /// 旧朝向为上、向右横向探步撞炉，不得打开炼丹炉页，玩家不动。
    #[test]
    fn bug047_sideways_probe_into_alchemy_with_old_facing_up_does_not_open_it() {
        let mut sc = scene((5, ROW), &[(9, 6, ROW)]);
        let wt = sc.width_table.clone();
        sc.facing = 1;
        let _ = sc.probe_step(2);
        for fc in 0..20 {
            sc.tick(fc, &wt);
        }
        assert_ne!(sc.game_mode, GameMode::Alchemy, "横向撞炉不进炼丹炉");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "被炉挡住，原地");
    }

    /// 【BUG-047】正向：旧朝向为下、向上探步撞炉（炉在正上方）⇒ 打开炼丹炉页（判的是探步方向，不是旧朝向）。
    #[test]
    fn bug047_upward_probe_into_alchemy_with_old_facing_down_opens_it() {
        let mut sc = scene((5, ROW), &[(9, 5, ROW - 1)]);
        sc.facing = 0;
        let _ = sc.probe_step(1);
        assert_eq!(sc.game_mode, GameMode::Alchemy, "朝上撞炉进炼丹炉");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "进炉不移动玩家");
    }

    /// 【BUG-047】探步 ≡ 方向键（复核 R2）：撞带钥匙的黄门、撞可胜的怪，探步与按方向键走同一格的结果逐字段一致。
    /// 期望值取方向键路径（handle_field_input，a.java:5308-5329 同式），不依赖 probe_step。
    #[test]
    fn bug047_probe_step_matches_direction_key_on_doors_and_enemies() {
        fn outcome(sc: &GameScene) -> ((i32, i32), (i32, i32), i32, i32, i32, Vec<(i32, u8, bool)>) {
            let ents = (0..sc.entities.count)
                .map(|i| (sc.entities.items[i].type_, sc.entities.items[i].state, sc.entities.items[i].removed))
                .collect();
            (
                (sc.player_cell_x, sc.player_cell_y),
                (sc.view.player_px, sc.view.player_py),
                sc.facing,
                sc.player.yellow_keys,
                sc.player.hp,
                ents,
            )
        }
        let make = |t: u8, old: i32| {
            let mut sc = scene((5, ROW), &[(t, 6, ROW)]);
            sc.player.yellow_keys = 1;
            sc.player.atk = 100_000;
            sc.player.def = 100_000;
            sc.facing = old;
            sc
        };
        for (t, label) in [(1u8, "黄门（带钥匙）"), (43, "可胜的怪")] {
            for old in 0..4 {
                let mut keys = make(t, old);
                let wt = keys.width_table.clone();
                keys.press_key(-4);
                keys.tick(0, &wt);
                keys.release_key();
                for fc in 1..300 {
                    keys.tick(fc, &wt);
                }
                let mut probe = make(t, old);
                let _ = probe.probe_step(2);
                for fc in 0..300 {
                    probe.tick(fc, &wt);
                }
                assert_eq!(outcome(&probe), outcome(&keys), "{label}、旧朝向 {old}：探步 = 方向键");
            }
        }
    }

    /// 【BUG-049】Dialog 模式（mode 11）下弹出的提示框必须画出来（Java paint 公共尾对所有模式共跑，a.java:3129-3168）。
    /// 实机发现：点地寻路首步撞打不过的怪 ⇒ 拒战提示已激活（吞键），但画面上看不到。
    #[test]
    fn bug049_overlay_is_painted_in_dialog_mode() {
        let paint_once = |overlay: bool| {
            let mut sc = scene((5, ROW), &[]);
            sc.game_mode = GameMode::Dialog;
            sc.dialog_phase = 3;
            if overlay {
                sc.show_overlay_message(0, "你无法战胜它", 0, 0);
            }
            let mut canvas = crate::render::ArgbImage::create(240, 320);
            {
                let mut g = crate::render::SoftGraphics::new(&mut canvas);
                sc.paint(&mut g, &crate::render::ArgbImage::create(2, 2));
            }
            (canvas.argb, sc.overlay_box_w, sc.overlay_box_h)
        };
        let (plain, _, _) = paint_once(false);
        let (with_box, bw, bh) = paint_once(true);
        assert!(bw > 0 && bh > 0, "前置：提示框有尺寸");
        // 盒框居中（paint_overlay_box：bx = (screen_w - w) >> 1）；框内必须有像素与无框画面不同
        let (bx, by) = ((240 - bw) >> 1, (320 - bh) >> 1);
        let changed = (by..by + bh)
            .flat_map(|y| (bx..bx + bw).map(move |x| (x, y)))
            .filter(|&(x, y)| plain[(y * 240 + x) as usize] != with_box[(y * 240 + x) as usize])
            .count();
        assert!(changed > 0, "Dialog 模式下提示框区域须被绘制");
    }

    /// 【BUG-044】点地寻路首步撞上不弹浮层的阻挡：玩家不得进入该格，寻路收尾回 Map。
    /// find_path 经实体槽规划路线（walk_to(7,ROW) 自 (5,ROW) 起两步，首步即 (6,ROW)）。
    #[test]
    fn bug044_tap_walk_first_step_into_blocker_stays_put() {
        let cases: &[(u8, &str)] = &[(9, "炼丹炉（横向撞）"), (12, "隐形墙"), (70, "打不过的怪")];
        for &(t, label) in cases {
            let mut sc = scene((5, ROW), &[(t, 6, ROW)]);
            if t == 70 {
                sc.player.hp = 1;
            }
            let plan = sc.walk_to(7, ROW);
            assert!(!plan.steps.is_empty(), "{label}：前置——find_path 经实体槽规划出路线");
            run_until_settled(&mut sc, 300);
            if t == 70 {
                // 【BUG-046】打不过 ⇒ 原版拒战提示框（a.java:5487-5490）；关框后寻路收尾
                assert!(sc.overlay_active, "{label}：弹拒战提示框");
                assert_eq!(sc.overlay_text.as_deref(), Some("你无法战胜它"), "{label}：提示文案");
                assert_eq!((sc.game_mode, sc.dialog_phase), (GameMode::Dialog, 3), "{label}：弹框期间仍归点地寻路的 dp3");
                assert!(sc.route.is_open(), "{label}：弹框期间路段开着（关框键属浮层不打断；步序已清，关框后 finish_walkto 收尾）");
                let wt = sc.width_table.clone();
                sc.press_key(-5);
                sc.tick(1000, &wt);
                sc.release_key();
                run_until_settled(&mut sc, 300);
                for fc in 1001..1010 {
                    sc.tick(fc, &wt);
                }
            }
            assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "{label}：首步被挡，玩家留在原格");
            assert_eq!((sc.view.player_px, sc.view.player_py), (5 << 5, ROW << 5), "{label}：像素位置未动");
            assert_eq!(sc.game_mode, GameMode::Map, "{label}：寻路收尾回 Map");
            assert!(!sc.route.is_open(), "{label}：路段已收");
            assert!(!sc.script_walk_armed && !sc.walkto_draining, "{label}：无寻路武装 / 排空残留");
            assert_ne!(sc.walk_phase, 5, "{label}：未开战");
        }
    }

    /// 【BUG-044】可胜的怪仍照常开战（返回 false 但 wp=5 的分支不得被当成"被挡"清掉）。
    #[test]
    fn bug044_tap_walk_first_step_into_winnable_enemy_still_fights() {
        let mut sc = scene((5, ROW), &[(41, 6, ROW)]);
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 500; // 夹具默认 hp=0 ⇒ 预测 loss(0) < hp 不成立，会被判"打不过"
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut saw_battle = false;
        for fc in 0..40 {
            sc.tick(fc, &wt);
            saw_battle |= sc.walk_phase == 5;
        }
        assert!(saw_battle, "首步撞可胜的怪 ⇒ 开战（walk_phase 5）");
    }

    /// 【BUG-044】保真钉：经典脚本行走（MOV 驱动的 dp3，路段未开）首步撞阻挡仍按原版不判返回值
    /// （a.java:4080-4084）——修复分支只对点地寻路生效（walkto_owns_dp3）。去掉该门控的变异须在此变红。
    #[test]
    fn bug044_classic_script_walk_keeps_original_unchecked_first_step() {
        let mut sc = scene((5, ROW), &[(9, 6, ROW)]);
        let wt = sc.width_table.clone();
        // dp3 消费机的经典入口状态（op_mov 同款：路径缓冲 + 武装 + Dialog dp3 wp0），不开路段
        sc.walk_path_buffer = vec![2];
        sc.walk_step_count = 1;
        sc.script_walk_armed = true;
        sc.dialog_phase = 3;
        sc.walk_phase = 0;
        sc.game_mode = GameMode::Dialog;
        assert!(!sc.route.is_open(), "前置：经典脚本行走不开路段");
        for fc in 0..12 {
            sc.tick(fc, &wt);
        }
        assert_eq!(
            (sc.player_cell_x, sc.player_cell_y),
            (6, ROW),
            "经典 MOV 首步不判碰撞（原版行为，Stage A 保真）"
        );
    }

    /// 一拍 + 绘制（wp4 开门动画由 paint 推进，advance_open_group_paint）。
    fn tick_paint(sc: &mut GameScene, fc: i64) {
        let wt = sc.width_table.clone();
        sc.tick(fc, &wt);
        let mut canvas = crate::render::ArgbImage::create(240, 320);
        let mut g = crate::render::SoftGraphics::new(&mut canvas);
        sc.paint(&mut g, &crate::render::ArgbImage::create(2, 2));
    }

    /// 【BUG-044】复核 T2-2：首步撞"撞格即触发"的剧情点（型 83，空脚本立即收尾）——剧情接管先收段，
    /// 门控须按弹步时的归属判定；终格 = 方向键撞同一格的终格（停在原格）。
    #[test]
    fn bug044_first_step_into_scene_trigger_matches_direction_key() {
        let make = || {
            let mut sc = scene((4, ROW), &[(83, 5, ROW)]);
            sc.dialog_data.scripts = vec![String::new()];
            sc
        };
        let mut keys = make();
        let wt = keys.width_table.clone();
        keys.press_key(-4);
        keys.tick(0, &wt);
        keys.release_key();
        for fc in 1..120 {
            keys.tick(fc, &wt);
        }
        let mut touch = make();
        let _ = touch.walk_to(6, ROW);
        for fc in 0..120 {
            touch.tick(fc, &wt);
        }
        assert_eq!(
            (touch.player_cell_x, touch.player_cell_y, touch.view.player_px),
            (keys.player_cell_x, keys.player_cell_y, keys.view.player_px),
            "首步撞剧情点：点地终格 = 方向键终格"
        );
        assert_eq!((touch.player_cell_x, touch.player_cell_y), (4, ROW), "参照：方向键撞剧情点停在原格");
    }

    /// 【BUG-044】踩 60 邻格扣血（step_triggered 短路 advance_script_walk）后的下一步经 dp3 wp0：撞炉须被挡。
    #[test]
    fn bug044_step_after_trap_damage_into_blocker_stays_put() {
        let mut sc = scene((1, ROW), &[(60, 2, ROW + 1), (9, 3, ROW)]);
        sc.player.hp = 1000;
        let _ = sc.walk_to(5, ROW);
        let mut hp_dropped = false;
        for fc in 0..200 {
            tick_paint(&mut sc, fc);
            hp_dropped |= sc.player.hp < 1000;
        }
        assert!(hp_dropped, "前置：踩 60 邻格扣血");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "扣血后的下一步撞炉被挡，停在 (2,ROW)");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(!sc.route.is_open());
    }

    /// 【BUG-044】踏板 76 开组门（wp4 开门动画，paint 推进）后的下一步经 dp3 wp0：撞炉须被挡。
    #[test]
    fn bug044_step_after_open_group_animation_into_blocker_stays_put() {
        let mut sc = scene((1, ROW), &[(76, 2, ROW), (4, 10, ROW), (9, 3, ROW)]);
        let door = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 4).expect("型 4");
        sc.entities.items[door].param = 1; // 组号 = 76 的 param&0xff + 1 = 1
        let _ = sc.walk_to(5, ROW);
        let mut saw_wp4 = false;
        for fc in 0..200 {
            tick_paint(&mut sc, fc);
            saw_wp4 |= sc.walk_phase == 4;
        }
        assert!(saw_wp4, "前置：踩 76 进 wp4 开门动画");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "wp4 后的下一步撞炉被挡，停在 (2,ROW)");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(!sc.route.is_open());
    }

    /// 【BUG-044】复核 T2-1：炉在正上方、点炉格（首步朝上撞炉 ⇒ 炉页）：不得残留寻路武装——出页后点地可用、方向键一步一格。
    /// debug 构建下 Dialog → Alchemy 未登记转移边会触发 transition_to 的 debug_assert（点地路径借 dp3 进炉，计划 2 改为 Map wp0 撞击后消除），
    /// 故本钉只在 release（gates rust-tests 口径）运行。
    #[test]
    #[cfg_attr(debug_assertions, ignore = "Dialog→Alchemy 未登记转移边（debug_assert）；计划 2 改 Map wp0 撞击后取消")]
    fn bug044_tap_alchemy_above_leaves_no_armed_walk() {
        let mut sc = scene((5, ROW), &[(9, 5, ROW - 1)]);
        let wt = sc.width_table.clone();
        sc.walkable[(ROW - 1) as usize][5] = true; // 夹具走廊只有一行：把炉格地形置可走，让 walk_to 直达炉格（真资产 1 层 (8,1) 同形）
        let plan = sc.walk_to(5, ROW - 1);
        assert_eq!(plan.steps, vec![1u8], "前置：一步朝上进炉格");
        for fc in 0..10 {
            sc.tick(fc, &wt);
        }
        assert_eq!(sc.game_mode, GameMode::Alchemy, "前置：进炉页");
        assert!(!sc.script_walk_armed, "炉页中无寻路武装残留");
        assert!(!sc.route.is_open(), "路段已收");
        sc.press_key(-7);
        sc.tick(10, &wt);
        sc.release_key();
        for fc in 11..20 {
            sc.tick(fc, &wt);
        }
        assert_eq!(sc.game_mode, GameMode::Map, "前置：出炉页");
        assert!(sc.field_tap_allowed(), "出页后点地可用");
        sc.press_key(-4);
        sc.tick(20, &wt);
        sc.release_key();
        for fc in 21..80 {
            sc.tick(fc, &wt);
        }
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "方向键一步只走一格");
    }

    /// 【BUG-045④】换层 / 存档 / 剧情接管后撤销栈清空（undo_journal.rs 清栈边界契约的生产接线）。
    #[test]
    fn bug045_journal_clears_on_floor_change_save_and_script() {
        use crate::undo_journal::MoveStroke;
        let stroke = MoveStroke { from: (1, ROW), to: (2, ROW) };
        // 换层（load_floor_data = changeFloor / LAY 的原地重载）
        let mut sc = scene((5, ROW), &[]);
        sc.undo.push_move(stroke);
        sc.load_floor_data(&super::scene_test_support::maplv(), 5 << 5, ROW << 5, &[], 2).unwrap();
        assert!(sc.undo.is_empty(), "换层清栈");
        // 剧情接管
        let mut sc = scene((5, ROW), &[]);
        sc.dialog_data.scripts = vec!["TAK_0_0 ".to_string(); 2];
        sc.undo.push_move(stroke);
        sc.load_level_script(0);
        assert!(sc.undo.is_empty(), "剧情清栈");
        // 存档（mode 7 确认键）
        let mut sc = scene((5, ROW), &[]);
        let wt = sc.width_table.clone();
        sc.undo.push_move(stroke);
        sc.menu_return_mode = GameMode::InGameMenu;
        sc.game_mode = GameMode::SaveSlot;
        sc.enter_menu7();
        sc.press_key(-5);
        sc.tick(0, &wt);
        sc.release_key();
        assert!(sc.slot.valid[sc.slot.cursor as usize], "前置：存档写入");
        assert!(sc.undo.is_empty(), "存档清栈");
    }

    /// 撤销栈里是否有战斗笔（逐笔弹出检查；只读状态副本的空目标）。
    fn journal_has_combat(sc: &mut GameScene) -> bool {
        struct Nop;
        impl crate::undo_journal::UndoTarget for Nop {
            fn set_player_tile(&mut self, _: (i32, i32)) {}
            fn set_player_vitals(&mut self, _: crate::undo_journal::PlayerVitals) {}
            fn revive_enemy(&mut self, _: crate::undo_journal::EnemyRestore) {}
            fn set_item_count(&mut self, _: i32, _: i32) {}
            fn set_door_state(&mut self, _: i32, _: i32, _: crate::undo_journal::DoorState) {}
        }
        let mut j = std::mem::take(&mut sc.undo);
        let mut found = false;
        while let crate::undo_journal::UndoOutcome::Undone(s) = j.undo(&mut Nop) {
            found |= matches!(s, crate::undo_journal::Stroke::Combat(_));
        }
        found
    }

    /// 【BUG-045 · 复核 T3-R1】击杀带不可逆生成（boss 序 68 ⇒ spawn_boss_event 生成掉落）：不入战斗笔、清栈——
    /// 否则"击杀 → 撤销"循环复制掉落（16 次后槽表越界 panic）。对照：普通怪 41 照常入战斗笔。
    #[test]
    fn bug045_kill_with_spawned_drops_pushes_no_combat_stroke() {
        let fight = |t: u8| {
            let mut sc = scene((5, ROW), &[(t, 6, ROW)]);
            sc.player.atk = 1_000_000;
            sc.player.def = 1_000_000;
            sc.player.hp = 1000;
            let count0 = sc.entities.count;
            let wt = sc.width_table.clone();
            let _ = sc.walk_to(6, ROW);
            let mut saw_battle = false;
            let mut fc = 0;
            while fc < 600 {
                sc.tick(fc, &wt);
                fc += 1;
                saw_battle |= sc.walk_phase == 5;
                if saw_battle && sc.battle_target < 0 {
                    break;
                }
            }
            assert!(saw_battle && sc.battle_target < 0, "型 {t}：前置——开战并结束");
            let spawned = sc.entities.count > count0;
            (spawned, journal_has_combat(&mut sc))
        };
        let (spawned68, combat68) = fight(68);
        assert!(spawned68, "前置：型 68 击杀生成掉落");
        assert!(!combat68, "有不可逆生成 ⇒ 栈里没有战斗笔");
        let (spawned41, combat41) = fight(41);
        assert!(!spawned41, "前置：型 41 击杀不生成实体");
        assert!(combat41, "对照：普通怪照常入战斗笔");
    }

    /// 【BUG-045 · 复核 T3-R3/R4】实体表重载（跨层探针）与未记账的消费（炼丹 / 商人 / 道具使用）清栈。
    #[test]
    fn bug045_journal_clears_on_reload_and_unrecorded_spending() {
        use crate::undo_journal::MoveStroke;
        let stroke = MoveStroke { from: (1, ROW), to: (2, ROW) };
        // 跨层探针的实体表重载（下标重排）
        let mut sc = scene((5, ROW), &[(41, 8, ROW)]);
        sc.undo.push_move(stroke);
        let f = sc.floor;
        sc.reload_floor_entities(f);
        assert!(sc.undo.is_empty(), "实体表重载清栈");
        // 炼丹购买
        let mut sc = scene((5, ROW), &[]);
        let wt = sc.width_table.clone();
        sc.undo.push_move(stroke);
        sc.player.gold = 1000;
        sc.alchemy_price = 10;
        sc.game_mode = GameMode::Alchemy;
        sc.press_key(-6);
        sc.tick(0, &wt);
        sc.release_key();
        assert_eq!(sc.player.gold, 990, "前置：炼丹扣金");
        assert!(sc.undo.is_empty(), "炼丹清栈");
        // 商人交易（型 78，src 1，flags 2，报价 3 = 50 金买蓝钥匙）
        let mut sc = scene((5, ROW), &[(78, 6, ROW)]);
        sc.undo.push_move(stroke);
        sc.player.gold = 100;
        let m = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 78).expect("型 78");
        sc.entities.items[m].param = (2 << 8) | 3;
        sc.merchant_entity = m as i32;
        let _ = sc.apply_merchant_offer(true);
        assert_eq!((sc.player.gold, sc.player.blue_keys), (50, 1), "前置：商人扣金给钥匙");
        assert!(sc.undo.is_empty(), "商人交易清栈");
        // 道具使用（19 回血）
        let mut sc = scene((5, ROW), &[]);
        sc.undo.push_move(stroke);
        sc.player.item_types[0] = 19;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        sc.use_item_stack(0);
        assert!(sc.undo.is_empty(), "道具使用清栈");
    }

    /// 【BUG-045 · 复核 T3-R6】reattach_to_cell 幂等：未摘槽（死亡动画中）即重登，不重复登记。
    #[test]
    fn bug045_reattach_is_idempotent_when_still_in_cell() {
        let mut sc = scene((5, ROW), &[(41, 8, ROW)]);
        let e = entity::cell_entity(&sc.grid, &sc.entities, 8, ROW, 41);
        assert!(e >= 0);
        let ct = sc.grid.cell_type[ROW as usize][8] as usize;
        let cap = sc.grid.capacity[ct];
        entity::reattach_to_cell(&mut sc.grid, &mut sc.entities, e as usize);
        assert_eq!(sc.grid.capacity[ct], cap, "已在槽表中 ⇒ 不重复登记");
        entity::detach_from_cell(&mut sc.grid, &mut sc.entities, e as usize);
        assert_eq!(sc.grid.capacity[ct], cap - 1, "前置：摘槽");
        entity::reattach_to_cell(&mut sc.grid, &mut sc.entities, e as usize);
        assert_eq!(sc.grid.capacity[ct], cap, "摘槽后重登一次");
        assert!(!sc.entities.items[e as usize].removed);
    }

    /// 【BUG-048】点地撞怪开战、击杀触发剧情（49 层型 70 → load_level_script(35)）：
    /// 剧情接管后寻路段不得被重开，撤销栈保持清空（不补入战斗笔）。
    #[test]
    fn bug048_victory_script_takeover_keeps_route_closed_and_journal_empty() {
        let mut sc = scene((5, ROW), &[(70, 6, ROW)]);
        sc.floor = 49;
        sc.current_floor = 49;
        sc.dialog_data.scripts = vec!["TAK_0_0 ".to_string(); 40];
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut saw_battle = false;
        let mut fc = 0;
        while fc < 600 {
            sc.tick(fc, &wt);
            fc += 1;
            saw_battle |= sc.walk_phase == 5;
            if saw_battle && sc.battle_target < 0 {
                break;
            }
        }
        assert!(saw_battle, "前置：开战");
        assert!(sc.battle_target < 0, "前置：战斗结束");
        assert!(!sc.route.is_open(), "剧情接管后寻路段不得被重开");
        assert!(sc.undo.is_empty(), "剧情接管 ⇒ 撤销栈已清，不再补入战斗笔");
    }

    /// 【BUG-048】战斗中经壳层按键链打断（壳先 interrupt_route 再 press ⇒ 路段挂起被解除，但未清栈）：战斗照常入栈、撤销不被永久挡住，路段不续走。
    #[test]
    fn bug048_key_interrupt_mid_tap_battle_still_records_combat() {
        let mut sc = scene((5, ROW), &[(41, 6, ROW)]);
        sc.player.atk = 2; // 刚好打得动：多回合战斗，留出按键窗口
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut fc = 0;
        while sc.walk_phase != 5 && fc < 40 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!(sc.walk_phase, 5, "前置：战斗中");
        // 壳层按键链：先 interrupt_route 再 press（native.rs replay / Dispatcher::key——不看路段是否开放，战斗挂起中同样打断）
        sc.interrupt_route();
        sc.press_key(-1); // 上 = 墙：只起打断作用
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        while sc.battle_target >= 0 && fc < 3000 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.battle_target < 0, "前置：战斗结束");
        assert!(!sc.undo.combat_in_progress(), "战斗挂起已解除（撤销不被永久挡住）");
        for _ in 0..60 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!(sc.game_mode, GameMode::Map, "收尾回 Map");
        assert!(!sc.route.is_open() && !sc.script_walk_armed, "路段已收、无武装残留");
        // 栈顶两笔：Move(邻格→敌格)（胜利 restep）在 Combat 之上——撤销先退回邻格，再复活怪物，人怪不重叠（复核 T4-R1）
        struct Rec(Vec<String>);
        impl crate::undo_journal::UndoTarget for Rec {
            fn set_player_tile(&mut self, t: (i32, i32)) {
                self.0.push(format!("tile{t:?}"));
            }
            fn set_player_vitals(&mut self, _: crate::undo_journal::PlayerVitals) {}
            fn revive_enemy(&mut self, _: crate::undo_journal::EnemyRestore) {
                self.0.push("revive".into());
            }
            fn set_item_count(&mut self, _: i32, _: i32) {}
            fn set_door_state(&mut self, _: i32, _: i32, _: crate::undo_journal::DoorState) {}
        }
        let mut rec = Rec(Vec::new());
        let mut j = std::mem::take(&mut sc.undo);
        let _ = j.undo(&mut rec);
        let _ = j.undo(&mut rec);
        assert_eq!(rec.0, vec![format!("tile{:?}", (5, ROW)), "revive".to_string()], "先退回邻格 (5,ROW)，再复活怪");
    }

    /// 弹出撤销栈全部笔，按逆放顺序记录（tile = 玩家落格，revive = 复活怪）。
    fn undo_seq(sc: &mut GameScene) -> Vec<String> {
        struct Rec(Vec<String>);
        impl crate::undo_journal::UndoTarget for Rec {
            fn set_player_tile(&mut self, t: (i32, i32)) {
                self.0.push(format!("tile{t:?}"));
            }
            fn set_player_vitals(&mut self, _: crate::undo_journal::PlayerVitals) {}
            fn revive_enemy(&mut self, _: crate::undo_journal::EnemyRestore) {
                self.0.push("revive".into());
            }
            fn set_item_count(&mut self, _: i32, _: i32) {}
            fn set_door_state(&mut self, _: i32, _: i32, _: crate::undo_journal::DoorState) {}
        }
        let mut rec = Rec(Vec::new());
        let mut j = std::mem::take(&mut sc.undo);
        while let crate::undo_journal::UndoOutcome::Undone(_) = j.undo(&mut rec) {}
        rec.0
    }

    /// 壳层投键口径打断（先 interrupt_route 再 press / release），随后跑到收尾。
    fn shell_key_interrupt_then_settle(sc: &mut GameScene, mut fc: i64) {
        let wt = sc.width_table.clone();
        sc.interrupt_route();
        sc.press_key(-1); // 上 = 墙：只起打断作用
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        for _ in 0..60 {
            sc.tick(fc, &wt);
            fc += 1;
        }
    }

    /// 【STAGE-B-008 · 复核 R2】首步在途（wp1，逻辑格仍是起点）时打断：排空走完这一步，这一格必须可撤。
    /// 收段点 = 在途步的落格（`route_landing_cell`），不是打断瞬间的逻辑格——否则判成零长丢弃，栈空。
    #[test]
    fn stage_b008_interrupt_mid_first_step_keeps_drained_step_undoable() {
        let mut sc = scene((1, ROW), &[]);
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(5, ROW);
        let mut fc = 0;
        while sc.walk_phase != 1 && fc < 10 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!(sc.walk_phase, 1, "前置：首步在途");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (1, ROW), "前置：逻辑格仍在起点");
        shell_key_interrupt_then_settle(&mut sc, fc);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "排空：在途步走完");
        assert_eq!(sc.undo.len(), 1, "走出的一格入栈");
        assert_eq!(undo_seq(&mut sc), vec![format!("tile{:?}", (1, ROW))], "撤销回起点");
    }

    /// 【STAGE-B-008 · 复核 R1】紧邻开战胜利后 restep 在途时打断：撤销序列 = [退回邻格, 复活怪]。
    /// 先复活会把怪压在玩家身上（复核 T4-R1 的重叠）。
    #[test]
    fn stage_b008_interrupt_mid_victory_restep_undoes_step_before_revive() {
        let mut sc = scene((5, ROW), &[(41, 6, ROW)]);
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 500; // 夹具默认 hp=0 ⇒ 判"打不过"
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut saw_battle = false;
        let mut fc = 0;
        while fc < 600 {
            sc.tick(fc, &wt);
            fc += 1;
            saw_battle |= sc.walk_phase == 5;
            if saw_battle && sc.battle_target < 0 && sc.walk_phase == 1 {
                break;
            }
        }
        assert!(saw_battle && sc.battle_target < 0, "前置：开战并胜利");
        assert_eq!(sc.walk_phase, 1, "前置：胜利 restep 在途");
        assert!(sc.route.is_open(), "前置：续走段已开");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "前置：逻辑格仍在邻格");
        shell_key_interrupt_then_settle(&mut sc, fc);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "排空：restep 走进敌格");
        assert_eq!(
            undo_seq(&mut sc),
            vec![format!("tile{:?}", (5, ROW)), "revive".to_string()],
            "先退回邻格 (5,ROW)，再复活怪"
        );
    }

    /// 【BUG-048 · 复核 T4-R2】剧情接管且击杀有生成（49 层型 71 ⇒ spawn_boss_event + load_level_script(35)）：同样不入栈、不续走、战斗挂起已解除。
    #[test]
    fn bug048_script_takeover_with_spawn_keeps_route_closed_and_journal_empty() {
        let mut sc = scene((5, ROW), &[(71, 6, ROW)]);
        sc.floor = 49;
        sc.current_floor = 49;
        sc.dialog_data.scripts = vec!["TAK_0_0 ".to_string(); 40];
        sc.player.atk = 1_000_000;
        sc.player.def = 1_000_000;
        sc.player.hp = 100_000;
        let count0 = sc.entities.count;
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut saw_battle = false;
        let mut fc = 0;
        while fc < 600 {
            sc.tick(fc, &wt);
            fc += 1;
            saw_battle |= sc.walk_phase == 5;
            if saw_battle && sc.battle_target < 0 {
                break;
            }
        }
        assert!(saw_battle && sc.battle_target < 0, "前置：开战并结束");
        assert!(sc.entities.count > count0, "前置：击杀生成实体");
        assert!(!sc.route.is_open() && !sc.route.is_suspended(), "剧情接管：路段不重开");
        assert!(!sc.undo.combat_in_progress(), "战斗挂起已解除");
        assert!(sc.undo.is_empty(), "撤销栈为空");
    }

    /// 【BUG-048】回归钉：无剧情钩子的击杀照常入战斗笔并续走（路段在胜利尾重开）。
    #[test]
    fn bug048_plain_victory_still_pushes_combat_and_resumes() {
        let mut sc = scene((5, ROW), &[(41, 6, ROW)]);
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut saw_battle = false;
        let mut fc = 0;
        while fc < 600 {
            sc.tick(fc, &wt);
            fc += 1;
            saw_battle |= sc.walk_phase == 5;
            if saw_battle && sc.battle_target < 0 {
                break;
            }
        }
        assert!(saw_battle && sc.battle_target < 0, "前置：开战并结束");
        assert!(sc.route.is_open(), "无剧情：胜利尾续走（路段重开，restep 进敌格）");
        assert!(journal_has_combat(&mut sc), "无剧情：栈里有战斗笔");
    }

    /// 【STAGE-B-008 · spec §2.2 保真红线】陷阱谓词与 classic 原块（apply_step_cell_effects）逐格对拍：
    /// 合成摆放 × 视口判界变体 × 全格 × 装备 {0, 40}，"站上该格是否扣血"须一致。原块是裁判（实跑扣血），谓词不得由原块推导。
    /// 视口变体：格表恒为 13×13，覆写 view.wide / view.high 把"视口判界"与"格表越界（entity_at_cell 返 −1）"解耦——
    /// 实机两者同源（CellGrid::new(view.wide, view.high)），不解耦时 61 的 `x < wide - 1` 等判界项被越界兜底吞掉、
    /// 变异杀不死；(12, 13) = 唯一非方形层 floor 51 的真实尺寸（maplv51 头 `1800 1a00`），(13, 12) 是"右邻用 view.high"
    /// 怪癖实际收窄判界的方向。
    #[test]
    fn trap_predicate_matches_step_effects_everywhere() {
        // 摆放：61 夹击（横 / 竖）、62 邻接、60 邻接（含 60 上邻的上爬分支）、贴边（判界怪癖）
        let layouts: &[&[(u8, i32, i32)]] = &[
            &[(61, 4, ROW), (61, 6, ROW)],
            &[(61, 5, ROW - 1), (61, 5, ROW + 1)],
            &[(62, 7, ROW)],
            &[(60, 3, ROW)],
            &[(60, 8, ROW - 1)],
            &[(62, 0, ROW), (60, 12, ROW)],
            &[(61, 0, ROW), (61, 2, ROW), (62, 11, ROW + 1)],
            // 贴右 / 贴下（配合视口变体）：61 横对跨 x = 11、竖对跨 y = 11；62 / 60 在 x = 12 列
            &[(61, 10, ROW), (61, 12, ROW), (61, 5, 10), (61, 5, 12)],
            &[(62, 12, ROW), (60, 12, ROW + 2), (62, 7, 12)],
            // 贴顶（复核 T3-R1）：上邻 / 61 竖夹的 `y > 0` 判界
            &[(62, 7, 0), (60, 9, 0), (61, 3, 0), (61, 3, 2)],
        ];
        for (li, ents) in layouts.iter().enumerate() {
            for view_dims in [None, Some((12, 13)), Some((13, 12))] {
                for armor in [0, 40] {
                    for y in 0..13 {
                        for x in 0..13 {
                            if ents.iter().any(|&(_, ex, ey)| (ex, ey) == (x, y)) {
                                continue;
                            }
                            let mut sc = scene((1, ROW), ents);
                            if let Some((w, h)) = view_dims {
                                sc.view.wide = w;
                                sc.view.high = h;
                            }
                            sc.equipped_armor = armor;
                            let predicted = sc.trap_damage_at(x, y);
                            sc.player.hp = 10_000;
                            sc.view.player_px = x << 5;
                            sc.view.player_py = y << 5;
                            sc.apply_step_cell_effects();
                            let damaged = sc.player.hp < 10_000;
                            assert_eq!(
                                predicted, damaged,
                                "摆放 {li} 视口 {view_dims:?} 装备 {armor} 格 ({x},{y})：谓词 = 原块"
                            );
                        }
                    }
                }
            }
        }
    }

    /// 【STAGE-B-008】陷阱谓词真实数据对拍：assets/raw 全部 maplv{n}+sprite{n}（0..=54）中含型 60/61/62 的层，
    /// 全图每格 × 装备 {0, 40}，谓词 vs 原块实跑扣血。实测覆盖 floor 41–43、45–49（均 13×13；唯一非方形层 51 无陷阱实体）。
    /// 隔离：站立格本身有实体（拾取 / 楼梯 / 宝箱 / 83 踩格 /
    /// 门组等格内效果会改 hp 或换层）的格跳过并计数——陷阱块只读四邻，站立格实体不参与陷阱判定。
    #[test]
    fn trap_predicate_matches_step_effects_on_real_trap_floors() {
        use super::scene_test_support::{images, numbers};
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let raw = |name: &str| {
            std::fs::read(repo.join("assets/raw").join(name)).unwrap_or_else(|e| panic!("缺资产 {name}：{e}"))
        };
        let width_table = numbers(include_str!("../tests/fixtures/type_width_px.txt"));
        let walk = numbers(include_str!("../tests/fixtures/tile_walkability.txt"));
        let doors = crate::sprite_spawn::DoorTables {
            door12: numbers(include_str!("../tests/fixtures/door12.txt")),
            door13: numbers(include_str!("../tests/fixtures/door13.txt")),
        };
        let mut covered: Vec<(i32, i32, i32, usize, usize, usize)> = Vec::new(); // (层, wide, high, 比较格, 跳过格, 扣血格)
        for n in 0..55i32 {
            let maplv = raw(&format!("maplv{n}"));
            let sp = game_data::SpriteTable::parse(&raw(&format!("sprite{n}"))).unwrap();
            let recs: Vec<(u8, u16, u16, i16, bool)> = sp
                .records
                .iter()
                .map(|r| {
                    let (param, hide) = crate::sprite_spawn::spawn_param_and_visibility(r.type_code, &r.extra, &doors);
                    (r.type_code, r.x, r.y, param, !hide)
                })
                .collect();
            if !recs.iter().any(|r| matches!(r.0, 60..=62)) {
                continue;
            }
            let load = || {
                let mut sc =
                    GameScene::load_floor(&maplv, 0, 0, &recs, &width_table, &walk, images(), vec![None; 56]).unwrap();
                sc.floor = n;
                sc.current_floor = n;
                sc
            };
            let probe = load();
            let (wide, high) = (probe.view.wide, probe.view.high);
            let (mut cmp, mut skip, mut hurt) = (0usize, 0usize, 0usize);
            for y in 0..high {
                for x in 0..wide {
                    let ct = probe.grid.cell_type[y as usize][x as usize] as usize;
                    if probe.grid.capacity[ct] > 0 {
                        skip += 1;
                        continue;
                    }
                    for armor in [0, 40] {
                        let mut sc = load();
                        sc.equipped_armor = armor;
                        let predicted = sc.trap_damage_at(x, y);
                        sc.player.hp = 10_000;
                        sc.view.player_px = x << 5;
                        sc.view.player_py = y << 5;
                        sc.apply_step_cell_effects();
                        let damaged = sc.player.hp < 10_000;
                        assert_eq!(predicted, damaged, "floor {n}（{wide}×{high}）装备 {armor} 格 ({x},{y})：谓词 = 原块");
                        cmp += 1;
                        hurt += damaged as usize;
                    }
                }
            }
            covered.push((n, wide, high, cmp, skip, hurt));
        }
        eprintln!("PROBE 陷阱层对拍（层, wide, high, 比较格次, 跳过实体格, 扣血格次）: {covered:?}");
        assert_eq!(
            covered.iter().map(|c| c.0).collect::<Vec<_>>(),
            vec![41, 42, 43, 45, 46, 47, 48, 49],
            "含型 60/61/62 的层须全部覆盖（复核 T3-R2：44、50–54 无陷阱）"
        );
        assert!(covered.iter().all(|c| c.5 > 0), "每个陷阱层至少有扣血格（防判定全假的空对拍）");
    }

    /// 【STAGE-B-008 · spec §5.1】格分类：不可见实体只看地形（隐形路径 = 墙、隐形墙 / 剧情点 = 地面），
    /// 可见的门 / 商人 / 怪 / 炉按类别，道具格是地面，陷阱邻格标危险，已死 / 开门动画中的实体不挡。
    #[test]
    fn tap_grid_classifies_by_visible_entities_and_terrain() {
        use crate::tap_walk::TargetKind;
        // 走廊 y=ROW、x∈1..=11 可走；其余墙
        let mut sc = scene((1, ROW), &[(6, 2, ROW + 1), (12, 3, ROW), (83, 4, ROW), (1, 5, ROW), (41, 6, ROW), (9, 8, ROW), (13, 11, ROW), (62, 10, ROW - 1)]);
        // 夹具 scene() 的记录一律 visible=true（隐藏位由调用方经 sprite_spawn::spawn_param_and_visibility 给出）；
        // 此处按 m_122 无存档分支语义手动隐藏 6 / 12 / 83（load_floor_data `if !visible` 臂）。
        for i in 0..sc.entities.count {
            if matches!(sc.entities.items[i].type_, 6 | 12 | 83) {
                sc.entities.items[i].visible = false;
            }
        }
        let g = sc.tap_grid();
        let kind = |x: i32, y: i32| g.get(x, y).unwrap().kind;
        assert_eq!(kind(2, ROW + 1), TargetKind::Wall, "未显形隐形路径 = 墙（地形不可走）");
        assert_eq!(kind(3, ROW), TargetKind::Floor, "未显形隐形墙 = 地面（地形可走）");
        assert_eq!(kind(4, ROW), TargetKind::Floor, "剧情点（不可见）= 地面");
        assert_eq!(kind(5, ROW), TargetKind::Interact, "门 = 交互物");
        assert!(matches!(kind(6, ROW), TargetKind::Monster(_)), "怪");
        assert_eq!(kind(8, ROW), TargetKind::Alchemy, "炉（3 格宽：8..=10）");
        assert_eq!(kind(10, ROW), TargetKind::Alchemy, "炉第三格");
        assert_eq!(kind(11, ROW), TargetKind::Floor, "道具格 = 地面");
        assert_eq!(kind(0, ROW), TargetKind::Wall, "地形墙");
        assert!(g.get(10, ROW - 2).map_or(true, |c| !c.danger), "非地面格不标危险（即使紧邻 62）");
        // (3, ROW) 的四邻 (2,ROW)(4,ROW)(3,ROW±1) 都不是 62（62 在 (10, ROW-1)）
        assert!(!g.get(3, ROW).unwrap().danger, "不邻 62 的地面格不危险");
        // 显形后的隐形路径 = 地面；死亡 / 移除的怪不挡
        let i6 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 6).unwrap();
        sc.entities.items[i6].visible = true;
        let i41 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        sc.entities.items[i41].state = 1;
        let g = sc.tap_grid();
        assert_eq!(g.get(2, ROW + 1).unwrap().kind, TargetKind::Floor, "已显形隐形路径 = 地面");
        assert_eq!(g.get(6, ROW).unwrap().kind, TargetKind::Floor, "死亡动画中的怪不挡");
    }

    /// 危险标记：62 上方紧邻的走廊格危险；装备 40 时不危险。
    #[test]
    fn tap_grid_marks_trap_danger() {
        let mut sc = scene((1, ROW), &[(62, 7, ROW - 1)]);
        assert!(sc.tap_grid().get(7, ROW).unwrap().danger, "62 正下方危险");
        assert!(!sc.tap_grid().get(5, ROW).unwrap().danger, "远处不危险");
        sc.equipped_armor = 40;
        assert!(!sc.tap_grid().get(7, ROW).unwrap().danger, "装备 40 全免");
    }

    /// 类目覆盖：装备（类目 4）= 地面；77/78（类目 16）、84 / 非 50 层 72（类目 32）= 交互物；
    /// 50 层 72 = 怪（层覆写，entity_category 读 `self.floor`）；型 69 三格同一 Monster 下标；Monster(e) 指向本格怪。
    #[test]
    fn tap_grid_categories_and_monster_index() {
        use crate::tap_walk::TargetKind;
        let mut sc = scene(
            (1, ROW),
            &[(33, 2, ROW), (77, 3, ROW), (78, 4, ROW), (84, 5, ROW), (72, 6, ROW), (69, 7, ROW), (41, 11, ROW)],
        );
        let g = sc.tap_grid();
        let kind = |x: i32| g.get(x, ROW).unwrap().kind;
        assert_eq!(kind(2), TargetKind::Floor, "装备 33（类目 4）= 地面");
        assert_eq!(kind(3), TargetKind::Interact, "77（类目 16）= 交互物");
        assert_eq!(kind(4), TargetKind::Interact, "78（类目 16）= 交互物");
        assert_eq!(kind(5), TargetKind::Interact, "84（类目 32）= 交互物");
        assert_eq!(kind(6), TargetKind::Interact, "非 50 层 72（类目 32）= 交互物");
        let TargetKind::Monster(e69) = kind(7) else { panic!("69 首格应为怪: {:?}", kind(7)) };
        assert_eq!(sc.entities.items[e69].type_, 69, "Monster 下标指向 69");
        assert_eq!(kind(8), TargetKind::Monster(e69), "69 第二格同一下标（96px = 3 格）");
        assert_eq!(kind(9), TargetKind::Monster(e69), "69 第三格同一下标");
        assert_eq!(kind(10), TargetKind::Floor, "69 只占 3 格");
        let TargetKind::Monster(e41) = kind(11) else { panic!("41 应为怪: {:?}", kind(11)) };
        assert_eq!(sc.entities.items[e41].type_, 41, "Monster 下标指向 41");

        sc.floor = 50;
        sc.current_floor = 50;
        let g = sc.tap_grid();
        let TargetKind::Monster(e72) = g.get(6, ROW).unwrap().kind else {
            panic!("50 层 72 应为怪: {:?}", g.get(6, ROW).unwrap().kind)
        };
        assert_eq!(sc.entities.items[e72].type_, 72, "Monster 下标指向 72");
    }

    /// 同格多实体的优先级 Monster > Alchemy > Interact（两种登记顺序都试：防"后者覆盖"）；道具 + 门 = 交互物。
    #[test]
    fn tap_grid_same_cell_priority() {
        use crate::tap_walk::TargetKind;
        let sc = scene(
            (1, ROW),
            &[
                (41, 2, ROW), (1, 2, ROW), // 怪先登记
                (1, 3, ROW), (41, 3, ROW), // 门先登记
                (9, 4, ROW), (1, 5, ROW), // 炉 4..=6，门叠在炉第二格（炉先）
                (1, 7, ROW), (9, 7, ROW), // 门先，炉 7..=9
                (13, 10, ROW), (1, 10, ROW),
                (1, 11, ROW), (13, 11, ROW),
            ],
        );
        // 前置：同格确实登记了两个槽
        for x in [2, 3, 5, 7, 10, 11] {
            let ct = sc.grid.cell_type[ROW as usize][x] as usize;
            assert_eq!(sc.grid.capacity[ct], 2, "({x}, ROW) 应有 2 个槽");
        }
        let g = sc.tap_grid();
        let kind = |x: i32| g.get(x, ROW).unwrap().kind;
        for x in [2, 3] {
            let TargetKind::Monster(e) = kind(x) else { panic!("({x}) 怪 + 门应为怪: {:?}", kind(x)) };
            assert_eq!(sc.entities.items[e].type_, 41, "({x}) Monster 下标指向 41");
        }
        assert_eq!(kind(5), TargetKind::Alchemy, "炉 + 门（炉先）= 炉");
        assert_eq!(kind(7), TargetKind::Alchemy, "门 + 炉（门先）= 炉");
        assert_eq!(kind(10), TargetKind::Interact, "道具 + 门 = 交互物");
        assert_eq!(kind(11), TargetKind::Interact, "门 + 道具 = 交互物");
    }

    fn settle(sc: &mut GameScene, from: i64, n: i64) {
        let wt = sc.width_table.clone();
        for fc in from..from + n {
            sc.tick(fc, &wt);
        }
    }

    /// 夹具 `scene()` 的记录一律 visible=true：按 m_122 无存档分支语义手动隐藏指定型（同 tap_grid 测试写法）。
    fn hide_types(sc: &mut GameScene, types: &[i32]) {
        for i in 0..sc.entities.count {
            if types.contains(&sc.entities.items[i].type_) {
                sc.entities.items[i].visible = false;
            }
        }
    }

    /// 【STAGE-B-008 · spec §5.2】点可达地面：走过去（BFS），到达后回 Map、目标标记清除。
    #[test]
    fn tap_cell_walks_to_reachable_floor() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Walking);
        assert_eq!(sc.tap_target, Some((7, ROW)), "目标标记");
        settle(&mut sc, 0, 120);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (7, ROW));
        assert_eq!(sc.game_mode, GameMode::Map);
        assert_eq!(sc.tap_target, None, "到达后标记清除");
    }

    /// 点远处的墙：走到墙边可达邻格，再朝墙撞一步（不移动、朝向 = 撞击方向、不留零长撤销笔）。
    #[test]
    fn tap_cell_far_wall_walks_adjacent_then_bumps() {
        let mut sc = scene((2, ROW), &[]);
        let _ = sc.tap_cell(8, ROW - 1);
        settle(&mut sc, 0, 160);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (8, ROW), "停在墙下方");
        assert_eq!(sc.facing, 1, "朝上撞过墙");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert_eq!(sc.undo.len(), 1, "只有一笔 (2,ROW)→(8,ROW) 的位移笔（撞墙零长段不入栈）");
    }

    /// 点门（有钥匙）：走到门旁、撞门开门并走进门格。
    #[test]
    fn tap_cell_door_opens_with_key() {
        let mut sc = scene((2, ROW), &[(1, 6, ROW)]);
        sc.player.yellow_keys = 1;
        let _ = sc.tap_cell(6, ROW);
        settle(&mut sc, 0, 200);
        assert_eq!(sc.player.yellow_keys, 0, "耗钥开门");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "走进门格");
    }

    /// 点正上方的炼丹炉：停在炉正下方朝上撞 ⇒ 打开炼丹炉页（Map → Alchemy，不经 Dialog——debug 构建不 panic）。
    #[test]
    fn tap_cell_alchemy_from_below_opens_page() {
        let mut sc = scene((2, ROW), &[(9, 5, ROW - 1)]);
        let _ = sc.tap_cell(6, ROW - 1);
        settle(&mut sc, 0, 160);
        assert_eq!(sc.game_mode, GameMode::Alchemy, "炉页打开");
        assert!(!sc.script_walk_armed && !sc.route.is_open(), "无寻路残留");
    }

    /// 到不了的地面：停在最近可达格，目标格闪红。
    #[test]
    fn tap_cell_unreachable_floor_docks_and_flashes() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        sc.walkable[ROW as usize][7] = false; // 走廊中间砌墙
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Docking);
        assert!(matches!(sc.tap_reject, Some(((9, r), _)) if r == ROW), "目标格闪红");
        settle(&mut sc, 0, 120);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "停在墙前最近格");
    }

    /// 【spec §0 隐藏通路泄露】只能经隐形路径到达的地面：不得走过去、隐形路径不显形（停靠 + 闪红）。
    #[test]
    fn tap_cell_does_not_leak_hidden_path() {
        let mut sc = scene((2, ROW), &[(6, 6, ROW)]);
        hide_types(&mut sc, &[6]); // 未显形的隐形路径
        sc.walkable[ROW as usize][6] = false; // 隐形路径格的地形 = 墙（真实关卡同形）
        let _ = sc.tap_cell(9, ROW);
        settle(&mut sc, 0, 160);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "停在隐形路径前");
        let i6 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 6).unwrap();
        assert!(!sc.entities.items[i6].visible, "隐形路径未显形");
    }

    /// 途中撞上隐形墙（看起来是地面）：被挡停下，撞击后续作废（玩家不在停靠格）。
    #[test]
    fn tap_cell_bump_cancelled_when_route_stops_early() {
        let mut sc = scene((2, ROW), &[(12, 5, ROW)]);
        hide_types(&mut sc, &[12]); // 未显形的隐形墙（tap_grid 视为地面）
        let _ = sc.tap_cell(9, ROW - 1); // 墙：停靠 (9,ROW)，路线途经 (5,ROW) 隐形墙
        settle(&mut sc, 0, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (4, ROW), "撞隐形墙停下");
        assert_ne!(sc.facing, 1, "朝上的撞击已作废");
        assert_eq!(sc.game_mode, GameMode::Map);
    }

    /// 观察模式点按无效；手动行走中点按闪红、状态不变。
    #[test]
    fn tap_cell_ignored_in_observe_and_rejected_while_walking() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        sc.walk_phase = 2;
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Ignored);
        sc.walk_phase = 1;
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Rejected);
        assert!(sc.tap_reject.is_some() && !sc.route.is_open());
        sc.walk_phase = 5;
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Rejected, "战斗中闪红");
        assert_eq!(sc.walk_phase, 5, "战斗状态不变");
        sc.walk_phase = 0;
        sc.script_walk_armed = true; // Map 语境脚本行走的步间
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Rejected, "脚本行走中不得插入点地路线");
        assert!(!sc.route.is_open());
    }

    /// 【STAGE-B-008 · 复核 T4-R4】点地路线途经特殊拾取物（型 25：道具栈 + kind 1 带图标提示框，right=3）：
    /// 拾取格步末 advanceScriptWalk 已弹出下一步（wp1），提示框冻结 walkPhase 机——框开时人在拾取格、路段开着、目标标记仍在。
    /// 裁定（spec §5.1【2026-10-30 更正 · 计划 2 T5 复核 R1/R2】）：关框键属浮层（a.java:3252-3345 浮层分支吞键），不是手动行走输入，不打断点地路线
    ///（[`GameScene::interrupt_route_for_input`]）；关框后继续走到目标（"点哪里就往哪里走"）。
    /// 夹具：起点 (2,ROW) → 目标 (9,ROW)，拾取物在 (5,ROW)。`close_key` = 关框所按的键。
    fn pickup_overlay_route(close_key: i32) {
        use crate::tap_walk::{TapResult, TargetKind};
        let mut sc = scene((2, ROW), &[(25, 5, ROW)]);
        assert_eq!(sc.tap_grid().get(5, ROW).unwrap().kind, TargetKind::Floor, "前置：道具格 = 地面（可途经）");
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Walking);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !sc.overlay_active && fc < 200 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.overlay_active, "前置：拾取型 25 弹提示框");
        assert_eq!((sc.overlay_kind, sc.overlay_right_icon), (1, 3), "前置：kind 1、right=3 拾取框");
        settle(&mut sc, fc, 10); // 框开期间多跑几拍：浮层冻结 walkPhase 机
        fc += 10;
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "框开时人在拾取格");
        assert!(sc.route.is_open(), "框开时路段开着");
        assert_eq!(sc.tap_target, Some((9, ROW)), "框开时目标标记仍在");
        // 真实关框路径：按键 → tick（浮层分支处理该键）→ 松键
        sc.press_key(close_key);
        assert!(sc.route.is_open(), "key={close_key}：按键属浮层，不打断路段");
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        assert!(!sc.overlay_active, "key={close_key}：提示框已关");
        settle(&mut sc, fc, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (9, ROW), "key={close_key}：关框后续走到终点");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(!sc.route.is_open() && !sc.script_walk_armed && !sc.walkto_draining, "无寻路残留");
        assert_eq!(sc.tap_target, None, "收尾后目标标记清除");
        assert_eq!(undo_seq(&mut sc), vec![format!("tile{:?}", (2, ROW))], "撤销栈 = 整段位移一笔（撤回起点）");
    }

    /// 【STAGE-B-008 · 计划 2 T5 复核问题 B】教程箭头开着时点门（有钥匙）/ 点紧邻可胜怪：路线收尾 `finish_walkto`
    /// 把余下箭头放回 walkPathBuffer（BUG-043，walk_step_count ≠ 0），随后 `execute_bump` 在 Map 语境开撞击段、走进门格 /
    /// 胜利 restep 进敌格。Map wp1 步末收段原条件含 `walk_step_count == 0` ⇒ 段一直开着：目标标记不清；下一次按键
    /// `interrupt_route` 清掉 walkPathBuffer（= 箭头），与方向键走同一路线不一致；下一次点地 `begin_walk_route` 重设段起点，
    /// 撞进去那一格的位移笔丢失。期望：撞进去那一步步末即收段，箭头 / 撤销与方向键参照一致。
    fn bump_with_tutorial_arrows(target_type: u8) {
        let make = || {
            let mut sc = scene((1, ROW), &[(target_type, 6, ROW)]);
            sc.player.yellow_keys = 1;
            sc.player.atk = 100_000;
            sc.player.def = 100_000;
            sc.player.hp = 500;
            assert!(sc.find_path(1, ROW, 8, ROW), "前置：箭头 (1)→(8) 经 (6,ROW)");
            sc.path_arrows_visible = true;
            sc
        };
        let label = if target_type == 1 { "黄门" } else { "可胜怪" };
        // 参照：方向键逐格走进 (6,ROW)（门：撞开后走进；怪：开战、胜利 restep 进敌格）
        let mut keys = make();
        key_walk(&mut keys, -4, (6, ROW));
        assert_eq!((keys.player_cell_x, keys.player_cell_y), (6, ROW), "{label} 参照前置：走进 (6,ROW)");
        let mut sc = make();
        sc.auto_battle = true; // 【计划 2 T6】本测试测撞击收尾、不测首点选中：自动战斗开 ⇒ 点怪一点即撞（门不受影响）
        let wt = sc.width_table.clone();
        assert_eq!(sc.tap_cell(6, ROW), crate::tap_walk::TapResult::Bumping, "{label}");
        let mut fc = 0;
        while fc < 3000 && !((sc.player_cell_x, sc.player_cell_y) == (6, ROW) && sc.walk_phase == 0 && sc.game_mode == GameMode::Map) {
            sc.tick(fc, &wt);
            fc += 1;
        }
        settle(&mut sc, fc, 20);
        fc += 20;
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "{label}：撞进去了");
        assert!(!sc.route.is_open() && !sc.route.is_suspended(), "{label}：撞进去那一步步末收段");
        assert_eq!(sc.tap_target, None, "{label}：收尾后目标标记清除");
        assert!(!sc.script_walk_armed, "{label}：无脚本行走武装（下次方向键不走 advanceScriptWalk 臂）");
        assert_eq!(arrow_state(&sc), arrow_state(&keys), "{label}：撞进去后箭头 = 方向键参照");
        // 再按一下方向键（右，沿箭头）：与方向键参照同一结果
        for k in [&mut keys, &mut sc] {
            k.press_key(-4);
            k.tick(fc, &wt);
            k.release_key();
            settle(k, fc + 1, 40);
        }
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (7, ROW), "{label}：方向键走一格");
        assert_eq!(arrow_state(&sc), arrow_state(&keys), "{label}：再走一格后箭头 = 方向键参照");
        // 撤销栈：点地段 (1→5) 与撞进去那一格 (5→6) 各一笔（怪另有战斗笔），方向键那格不入栈
        let seq = undo_seq(&mut sc);
        let expect: Vec<String> = if target_type == 1 {
            vec![format!("tile{:?}", (5, ROW)), format!("tile{:?}", (1, ROW))]
        } else {
            vec![format!("tile{:?}", (5, ROW)), "revive".to_string(), format!("tile{:?}", (1, ROW))]
        };
        assert_eq!(seq, expect, "{label}：撤销序列");
    }

    #[test]
    fn tap_bump_door_with_tutorial_arrows_closes_route() {
        bump_with_tutorial_arrows(1);
    }

    #[test]
    fn tap_bump_enemy_with_tutorial_arrows_closes_route_after_restep() {
        bump_with_tutorial_arrows(41);
    }

    #[test]
    fn tap_cell_route_through_pickup_overlay_resumes_after_close() {
        pickup_overlay_route(-5);
    }

    /// 对照：框开着时按方向键（-3 左）。kind 1 / right=3 分支（scene.rs tick 浮层块）对 0 吞、-2/-1 翻页、
    /// 其余键关层，块尾 keyValue / keyHeldCode 双清（a.java:3345）⇒ -3 只起关框作用、不走一步、不打断路线；
    /// 关框后同样续走到终点。
    #[test]
    fn tap_cell_route_through_pickup_overlay_direction_key_also_resumes() {
        pickup_overlay_route(-3);
    }

    /// (x, ROW) 上的门（型 1）是否未开：实体仍在、常态、未移除。
    fn door_intact(sc: &GameScene, x: i32) -> bool {
        (0..sc.entities.count).any(|i| {
            let e = &sc.entities.items[i];
            e.type_ == 1 && e.px == x << 5 && e.py == ROW << 5 && e.state == 0 && !e.removed
        })
    }

    /// 点门（有钥匙）多步路线：起点 (2,ROW)、门 (7,ROW)、停靠格 (6,ROW)。跑到末步在途窗口
    ///（`Map ∧ walk_phase == 1 ∧ tap_followup_ready`：advanceScriptWalk 在 (5,ROW) 步末弹出末步，finish_walkto 随即回 Map）。
    fn door_route_in_last_step_window() -> (GameScene, i64) {
        let mut sc = scene((2, ROW), &[(1, 7, ROW)]);
        sc.player.yellow_keys = 1;
        assert_eq!(sc.tap_cell(7, ROW), crate::tap_walk::TapResult::Bumping);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !(sc.game_mode == GameMode::Map && sc.walk_phase == 1 && sc.tap_followup_ready) && fc < 400 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.game_mode == GameMode::Map && sc.walk_phase == 1 && sc.tap_followup_ready, "前置：到达末步在途窗口");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "前置：末步 (5,ROW)→(6,ROW) 在途");
        assert_eq!(sc.tap_bump, Some(((6, ROW), 2)), "前置：撞击后续在");
        (sc, fc)
    }

    /// 【STAGE-B-008 · spec §5.2 · 计划 2 T5 复核 R3（探针 P1）】末步在途窗口算"点地寻路中"：点别处 ⇒ `Queued`
    ///（【计划 2 T7 改写】T5 原期望 `Stopped`，T7 起按"末步窗口点别处 = 排队改道"），不闪红、旧撞击取消；
    /// 落格后改道去新目标 (3,ROW)，旧撞击不执行（钥匙仍 1、门未开）。
    #[test]
    fn tap_cell_in_last_step_window_queues_reroute_and_cancels_bump() {
        let (mut sc, fc) = door_route_in_last_step_window();
        assert!(sc.accepts_field_tap(), "末步在途窗口场上接收点地（Map）");
        assert_eq!(sc.tap_cell(3, ROW), crate::tap_walk::TapResult::Queued);
        assert_eq!(sc.tap_reject, None, "不闪红");
        assert_eq!(sc.tap_bump, None, "撞击已取消");
        assert_eq!(sc.tap_target, Some((3, ROW)), "目标标记换到新目标");
        settle(&mut sc, fc, 120);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (3, ROW), "末步走完后改道到新目标");
        assert_eq!(sc.player.yellow_keys, 1, "未耗钥");
        assert!(door_intact(&sc, 7), "门未开");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert_eq!(sc.tap_target, None, "目标标记清除");
    }

    /// 【计划 2 T5 复核 R4（探针 P3）】末步在途窗口按方向键（上，撞墙方向）⇒ 撞击取消（钥匙仍 1、门未开）。
    #[test]
    fn key_press_in_last_step_window_cancels_bump() {
        let (mut sc, fc) = door_route_in_last_step_window();
        sc.press_key(-1);
        sc.release_key();
        assert_eq!(sc.tap_bump, None, "场上按键取消撞击");
        settle(&mut sc, fc, 80);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "停在停靠格");
        assert_eq!(sc.player.yellow_keys, 1, "未耗钥");
        assert!(door_intact(&sc, 7), "门未开");
        assert_eq!(sc.tap_target, None, "目标标记清除");
    }

    /// 【STAGE-B-008 · spec §5.1 · 计划 2 T5 复核 R1/R2（探针 M）】点门路线途经特殊拾取物（型 25 @ (4,ROW)）：
    /// 框开时按 -5 关框 ⇒ 关框键属浮层，不打断路线、不取消撞击；关框后续走到停靠格 (7,ROW) 并撞开门 (8,ROW)、走进门格。
    #[test]
    fn tap_bump_route_through_pickup_overlay_still_bumps_after_close() {
        let mut sc = scene((2, ROW), &[(25, 4, ROW), (1, 8, ROW)]);
        sc.player.yellow_keys = 1;
        assert_eq!(sc.tap_cell(8, ROW), crate::tap_walk::TapResult::Bumping);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !sc.overlay_active && fc < 200 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.overlay_active, "前置：拾取型 25 弹提示框");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (4, ROW), "前置：框开时人在拾取格");
        assert_eq!(sc.tap_bump, Some(((7, ROW), 2)), "前置：撞击后续在");
        sc.press_key(-5);
        assert_eq!(sc.tap_bump, Some(((7, ROW), 2)), "关框键不取消撞击");
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        assert!(!sc.overlay_active, "提示框已关");
        settle(&mut sc, fc, 300);
        assert_eq!(sc.player.yellow_keys, 0, "撞门耗钥");
        assert!(!door_intact(&sc, 8), "门已开");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (8, ROW), "走进门格");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(!sc.route.is_open() && !sc.script_walk_armed && !sc.walkto_draining, "无寻路残留");
        assert_eq!(sc.tap_target, None, "收尾后目标标记清除");
    }

    /// 【STAGE-B-008 · spec §5.1 · 计划 2 T5 复核 R2（探针 N2）】停靠格 (6,ROW) 本身有型 25：末步落格弹框，
    /// 撞击暂缓到关框后（框开期间保留撞击后续），关框后下一拍撞开门 (7,ROW)。
    #[test]
    fn tap_bump_dock_pickup_overlay_defers_bump_until_close() {
        let mut sc = scene((2, ROW), &[(25, 6, ROW), (1, 7, ROW)]);
        sc.player.yellow_keys = 1;
        assert_eq!(sc.tap_cell(7, ROW), crate::tap_walk::TapResult::Bumping);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !sc.overlay_active && fc < 200 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.overlay_active, "前置：停靠格拾取弹框");
        settle(&mut sc, fc, 10); // 框开期间多跑几拍：撞击须暂缓、不作废
        fc += 10;
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "框开时人在停靠格");
        assert_eq!(sc.tap_bump, Some(((6, ROW), 2)), "框开期间撞击后续保留");
        assert_eq!(sc.player.yellow_keys, 1, "框开期间未撞");
        assert_eq!(sc.tap_target, Some((7, ROW)), "框开期间目标标记仍在");
        sc.press_key(-5);
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        assert!(!sc.overlay_active, "提示框已关");
        settle(&mut sc, fc, 200);
        assert_eq!(sc.player.yellow_keys, 0, "关框后撞门耗钥");
        assert!(!door_intact(&sc, 7), "门已开");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (7, ROW), "走进门格");
        assert_eq!(sc.tap_target, None, "收尾后目标标记清除");
    }

    /// 【计划 2 T5 复核 R4（变异 M11）】1 步路线：站 (5,ROW) 点门 (7,ROW)，停靠格 (6,ROW)。首步在途（Dialog dp3 wp1）时点自己
    /// ⇒ `Stopped`；排空落到停靠格后不撞（钥匙仍 1、门未开）。
    #[test]
    fn tap_self_during_one_step_route_stops_without_bump() {
        let mut sc = scene((5, ROW), &[(1, 7, ROW)]);
        sc.player.yellow_keys = 1;
        assert_eq!(sc.tap_cell(7, ROW), crate::tap_walk::TapResult::Bumping);
        assert_eq!(sc.walk_step_count, 1, "前置：1 步路线");
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !(sc.game_mode == GameMode::Dialog && sc.walk_phase == 1) && fc < 50 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.game_mode == GameMode::Dialog && sc.walk_phase == 1, "前置：首步在途");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW));
        assert_eq!(sc.tap_cell(5, ROW), crate::tap_walk::TapResult::Stopped);
        settle(&mut sc, fc, 80);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "排空落到停靠格");
        assert_eq!(sc.player.yellow_keys, 1, "未耗钥");
        assert!(door_intact(&sc, 7), "门未开");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert_eq!(sc.tap_target, None, "目标标记清除");
    }

    /// 【计划 2 T5 复核 R4（变异 M9）】按住方向键（上，顶墙不动：keyHeldCode 每拍 tryStep 撞墙，停在 wp0）时点门：
    /// 撞击后续是在按键之后设的，press_key 清不到；落到停靠格时 keyHeldCode 仍非 0 ⇒ 场上手动输入优先，撞击作废
    ///（钥匙仍 1、门未开、人留在停靠格）。
    #[test]
    fn held_key_at_dock_cancels_bump() {
        let mut sc = scene((2, ROW), &[(1, 7, ROW)]);
        sc.player.yellow_keys = 1;
        let wt = sc.width_table.clone();
        sc.press_key(-1); // 按住不放
        sc.tick(0, &wt);
        assert_eq!((sc.player_cell_x, sc.player_cell_y, sc.walk_phase), (2, ROW, 0), "前置：顶墙不动");
        assert_eq!(sc.tap_cell(7, ROW), crate::tap_walk::TapResult::Bumping);
        assert_eq!(sc.tap_bump, Some(((6, ROW), 2)), "前置：撞击后续在");
        settle(&mut sc, 1, 200);
        assert_eq!(sc.key_held, -1, "前置：全程按住");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "停在停靠格");
        assert_eq!(sc.player.yellow_keys, 1, "未耗钥");
        assert!(door_intact(&sc, 7), "门未开");
        assert_eq!(sc.tap_target, None, "目标标记清除");
    }

    /// 【计划 2 T5 复核 R4（变异 M8 / M8b）】换层（load_floor_data）与剧情接管（load_level_script）清点地状态：
    /// 目标标记、闪红、撞击后续、后续就绪位全空。
    #[test]
    fn tap_state_cleared_on_floor_change_and_script_takeover() {
        for entry in ["换层", "剧情"] {
            let mut sc = scene((2, ROW), &[(1, 7, ROW)]);
            sc.player.yellow_keys = 1;
            sc.dialog_data.scripts = vec!["TAK_0_0 ".to_string()];
            assert_eq!(sc.tap_cell(7, ROW), crate::tap_walk::TapResult::Bumping);
            sc.flash_reject((9, ROW));
            assert_eq!(sc.tap_target, Some((7, ROW)), "{entry} 前置：目标标记");
            assert_eq!(sc.tap_bump, Some(((6, ROW), 2)), "{entry} 前置：撞击后续");
            assert!(sc.tap_reject.is_some(), "{entry} 前置：闪红");
            if entry == "换层" {
                sc.load_floor_data(&super::scene_test_support::maplv(), 2 << 5, ROW << 5, &[], 2).unwrap();
            } else {
                sc.load_level_script(0);
            }
            assert_eq!(sc.tap_target, None, "{entry}：目标标记清除");
            assert_eq!(sc.tap_reject, None, "{entry}：闪红清除");
            assert_eq!(sc.tap_bump, None, "{entry}：撞击后续清除");
            assert!(!sc.tap_followup_ready, "{entry}：后续就绪位清除");
        }
    }

    /// 路线已收尾（Map wp0）、撞击排在下一拍开头时再点按（此处点自己：不开新路线、仍在 Map）：旧撞击作废。
    #[test]
    fn t6_new_tap_in_followup_gap_cancels_pending_bump() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(1, 8, ROW)]);
        sc.player.yellow_keys = 1;
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !(sc.tap_followup_ready && sc.game_mode == GameMode::Map && sc.walk_phase == 0) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：收尾间隙");
        }
        assert_eq!(sc.player_cell_x, 7, "前置：停在门前");
        // 点自己（不开新路线、仍在 Map）：间隙里的旧撞击必须作废，否则下一拍照样开门
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Ignored);
        settle(&mut sc, fc, 60);
        assert_eq!(sc.player.yellow_keys, 1, "旧撞击作废：门未开");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (7, ROW), "原地不动");
        // 【复核 R4】只去间隙里的 tap_bump = None 时旧撞击滞留 ⇒ 目标框挂在门上不消失
        assert!(sc.tap_bump.is_none(), "旧撞击后续已清");
        assert_eq!(sc.tap_target, None, "目标框不残留");
    }

    fn strong(sc: &mut GameScene) {
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
    }

    /// 【STAGE-B-008 · spec §5.2 Q8】点怪：首点只选中（不动、不开路线）；再点同一只 = 走到邻格撞上开战（战斗笔可撤销）。
    #[test]
    fn t6_first_tap_selects_second_tap_fights() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 6, ROW)]);
        strong(&mut sc);
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Selected);
        assert_eq!(sc.tap_selected.map(|s| s.entity), Some(e));
        assert_eq!(sc.tap_target, None, "选中不画目标框（有自己的闪烁框）");
        settle(&mut sc, 0, 20);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "选中不移动");
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map);
        assert!(sc.tap_selected.is_some(), "空闲时选中保持");
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Bumping);
        assert!(sc.tap_selected.is_none(), "开战即清选中");
        settle(&mut sc, 20, 400);
        let ent = &sc.entities.items[e];
        assert!(ent.removed || ent.state == 1, "怪被击杀");
        assert!(journal_has_combat(&mut sc), "战斗笔入栈（误触靠撤销）");
    }

    /// 自动战斗开：一点即战。
    #[test]
    fn t6_auto_battle_fights_on_first_tap() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 6, ROW)]);
        strong(&mut sc);
        sc.auto_battle = true;
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Bumping);
        assert!(sc.tap_selected.is_none());
    }

    /// 点另一只 = 改选；点别处 = 清选中并执行。
    #[test]
    fn t6_tap_elsewhere_or_other_monster_replaces_selection() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 6, ROW), (42, 9, ROW)]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Selected);
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Selected, "点另一只 = 改选");
        assert_eq!(sc.tap_selected.map(|s| s.type_), Some(42));
        assert_eq!(sc.tap_cell(4, ROW), TapResult::Walking, "点别处 = 清选中并执行");
        assert!(sc.tap_selected.is_none());
    }

    /// 多格怪（型 69 宽 3 格）：点任一格选中同一只，再点另一格 = 开战（停靠在整只怪的外侧邻格）。
    #[test]
    fn t6_multi_cell_monster_any_cell_is_same_selection() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(69, 6, ROW)]);
        strong(&mut sc);
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Selected, "点 69 的第三格");
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Bumping, "再点同一只的中间格 = 开战");
    }

    /// 选中有效性每拍校验：移除 / 死亡动画 / 被搬走 / 变型 / 换层 / 浮层 / 按键 / 清状态，任一即清。
    #[test]
    fn t6_selection_invalidated_every_tick() {
        let mk = || {
            let mut sc = scene((2, ROW), &[(41, 6, ROW)]);
            let _ = sc.tap_cell(6, ROW);
            assert!(sc.tap_selected.is_some(), "前置：已选中");
            sc
        };
        let cases: Vec<(&str, Box<dyn Fn(&mut GameScene)>)> = vec![
            ("实体移除（他处击杀）", Box::new(|sc| { let e = sc.tap_selected.unwrap().entity; sc.entities.items[e].removed = true; })),
            ("死亡动画 state = 1", Box::new(|sc| { let e = sc.tap_selected.unwrap().entity; sc.entities.items[e].state = 1; })),
            ("被搬走（MOV）", Box::new(|sc| { let e = sc.tap_selected.unwrap().entity; sc.entities.items[e].px += 32; })),
            ("变型（道具改写）", Box::new(|sc| { let e = sc.tap_selected.unwrap().entity; sc.entities.items[e].type_ = 42; })),
            ("楼层号变化", Box::new(|sc| sc.current_floor += 1)),
            ("浮层", Box::new(|sc| sc.overlay_active = true)),
            // 【复核 R1】以下各钉一个 selection_valid 判据（去掉该判据即红）
            // hide_notify 真实入口：Map → InGameMenu（walk_phase / 浮层 / 楼层 / 实体都不变，只有 game_mode 判据拦得住）
            ("hide_notify（切后台 ⇒ 游戏内菜单）", Box::new(|sc| {
                sc.hide_notify();
                assert_eq!(sc.game_mode, GameMode::InGameMenu, "前置：hide_notify 进游戏内菜单");
            })),
            ("观察模式 walk_phase = 2", Box::new(|sc| sc.walk_phase = 2)),
            ("换层遮幅 transitioning", Box::new(|sc| sc.transitioning = true)),
            ("实体隐藏 visible = false", Box::new(|sc| { let e = sc.tap_selected.unwrap().entity; sc.entities.items[e].visible = false; })),
            // 防御判据：层内实体表只追加不压实、重排只在 reload_floor_entities（同时有型 / 格判据兜底），
            // 没有真实路径能让选中下标单独越界 ⇒ 直接缩小 count 构造
            ("实体下标越界（防御判据）", Box::new(|sc| sc.entities.count = 0)),
            // 「切模式」（classic ↔ modern）由计划 3 接入后补测
        ];
        for (what, f) in cases {
            let mut sc = mk();
            f(&mut sc);
            settle(&mut sc, 0, 1);
            assert!(sc.tap_selected.is_none(), "{what} ⇒ 清选中");
        }
        let mut sc = mk();
        sc.press_key(-2);
        assert!(sc.tap_selected.is_none(), "任何按键 ⇒ 清选中");
        let mut sc = mk();
        sc.clear_tap_state();
        assert!(sc.tap_selected.is_none(), "撤销 / 换层 / 剧情 ⇒ 清选中");
        let mut sc = mk();
        assert_eq!(sc.engage_selected(), crate::tap_walk::TapResult::Bumping, "「开战」= 撞已选中的怪");
    }

    /// 【复核 R1】换层 / 读档真实入口 load_floor_data：立即清选中（不等下一拍）。
    #[test]
    fn t6_selection_cleared_by_load_floor_data() {
        let mut sc = scene((2, ROW), &[(41, 6, ROW)]);
        let _ = sc.tap_cell(6, ROW);
        assert!(sc.tap_selected.is_some(), "前置：已选中");
        sc.load_floor_data(&super::scene_test_support::maplv(), 2 << 5, ROW << 5, &[], 2).unwrap();
        assert!(sc.tap_selected.is_none(), "换层 / 读档 ⇒ 立即清选中");
    }

    /// 【复核 R1】他处击杀真实击杀代码：选中邻格怪后用道具 86（use_item_stack → activate_item(86) → 四邻格 killAdjacentAt），下一拍清选中。
    /// 直接调确认浮层 -6 臂的 use_item_stack：UI 全路径上开面板的按键 / ItemPanel 模式 / 确认浮层本身都会先清选中（复核探针 P05），
    /// 这里要钉的是"选中仍在时实体被他处击杀"的 removed / state 判据。
    #[test]
    fn t6_selection_cleared_after_item86_kills_selected() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((5, ROW), &[(41, 6, ROW)]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Selected, "前置：选中邻格怪");
        let e = sc.tap_selected.unwrap().entity;
        sc.player.item_types[0] = 86;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        sc.use_item_stack(0);
        let ent = &sc.entities.items[e];
        assert!(ent.removed || ent.state == 1, "前置：86 击杀选中的怪");
        assert!(sc.tap_selected.is_some(), "前置：选中要等每拍校验清");
        settle(&mut sc, 0, 1);
        assert!(sc.tap_selected.is_none(), "道具 86 击杀 ⇒ 清选中");
    }

    /// 【复核 R2】实体表重排（跨层探针 m_119/m_122 往返压实）后、下一拍校验前的同帧里点同下标的另一只：只选中、不开战。
    #[test]
    fn t6_reorder_then_tap_other_monster_same_frame_selects() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((6, ROW), &[(41, 2, ROW), (42, 9, ROW), (43, 4, ROW)]);
        let i41 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        sc.entities.items[i41].removed = true; // 已死：往返后压掉 ⇒ 后面的下标前移
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Selected, "前置：选中 42");
        let sel = sc.tap_selected.unwrap();
        let _ = sc.probe_cross_floor_snapshot(23, 12); // 结果无关：只要它的 save → reload(23) → reload(本层) 往返
        let at = &sc.entities.items[sel.entity];
        assert_eq!((at.type_, at.px >> 5, at.py >> 5), (43, 4, ROW), "前置：重排后同下标 = 43@4");
        assert!(sc.tap_selected.is_some(), "前置：未 tick，选中仍在");
        assert_eq!(sc.tap_cell(4, ROW), TapResult::Selected, "同下标的另一只 = 首点选中，不开战");
        assert_eq!(sc.tap_selected.map(|s| s.type_), Some(43));
        assert!(!sc.route.is_open(), "未开路线");
    }

    /// 【复核 R3】选中后实体被移除、未 tick 就按「开战」：Ignored、清选中、玩家不动。
    #[test]
    fn t6_engage_selected_invalid_is_ignored() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 6, ROW)]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Selected);
        let e = sc.tap_selected.unwrap().entity;
        sc.entities.items[e].removed = true;
        assert_eq!(sc.engage_selected(), TapResult::Ignored);
        assert!(sc.tap_selected.is_none(), "清选中");
        assert!(!sc.route.is_open(), "未开路线");
        settle(&mut sc, 0, 30);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "玩家不动");
    }

    /// 自起步跑到玩家格 x ≥ `x` 且仍在点地寻路中（Dialog），返回下一拍帧号。
    fn walk_until_x(sc: &mut GameScene, x: i32) -> i64 {
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while sc.player_cell_x < x {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：走到 x={x}");
        }
        assert_eq!(sc.game_mode, GameMode::Dialog, "前置：仍在寻路中");
        fc
    }

    /// 【STAGE-B-008 · spec §5.2 排队】寻路中点别处：走完当前格后改道去新目标，标记立即换到新目标。
    #[test]
    fn t7_tap_while_walking_queues_reroute() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        assert_eq!(sc.tap_cell(10, ROW), TapResult::Walking);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        assert_eq!(sc.tap_target, Some((2, ROW)), "标记立即换到新目标");
        settle(&mut sc, fc, 300);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "改道到新目标");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert_eq!(sc.tap_target, None);
    }

    /// 寻路中点自己：停下（走完当前格），无排队、无标记。
    #[test]
    fn t7_tap_self_while_walking_stops() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        let me = (sc.player_cell_x, sc.player_cell_y);
        assert_eq!(sc.tap_cell(me.0, me.1), TapResult::Stopped);
        settle(&mut sc, fc, 200);
        assert!((4..=5).contains(&sc.player_cell_x), "走完当前格即停（实得 {}）", sc.player_cell_x);
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map);
        assert_eq!(sc.tap_target, None);

        // 【计划 2 T7 复核 R3（变异 Mb）】先排队再点自己：排队作废 ⇒ Stopped、排空后停在原处附近，不改道去排队目标
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued, "前置：排队");
        let me = (sc.player_cell_x, sc.player_cell_y);
        assert_eq!(sc.tap_cell(me.0, me.1), TapResult::Stopped);
        assert!(sc.tap_pending.is_none(), "点自己清排队");
        assert_eq!(sc.tap_target, None, "点自己清目标框");
        settle(&mut sc, fc, 200);
        assert!((4..=5).contains(&sc.player_cell_x), "排空后停在原处附近、不改道去 x=2（实得 {}）", sc.player_cell_x);
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map);
        assert!(sc.tap_pending.is_none() && sc.tap_target.is_none());
    }

    /// 排队失效：排空期间按键 ⇒ 丢弃；落格那一拍有手动按键（按住）⇒ 丢弃。
    /// 【执行期更正 · 以代码为准】落格时浮层开着 ⇒ 与撞击后续同口径**暂缓**到关框后执行（计划原文写"丢弃"，见 Task 7 更正②）。
    #[test]
    fn t7_queued_tap_discarded_on_key_or_blocked_landing() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        sc.press_key(-2); // 朝下（墙）
        sc.release_key();
        settle(&mut sc, fc, 200);
        assert!(sc.player_cell_x >= 4, "按键作废排队：不改道回 x=2（实得 {}）", sc.player_cell_x);

        // 浮层：落格（收尾）后、执行前出现浮层 ⇒ 暂缓，标记仍在；关框后下一拍执行
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let mut fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        let wt = sc.width_table.clone();
        while sc.game_mode != GameMode::Map {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 300);
        }
        let landed = sc.player_cell_x;
        sc.overlay_active = true; // 收尾后、执行前出现浮层（如拾取提示）
        sc.overlay_text = Some("提示".into());
        sc.tick(fc, &wt);
        sc.tick(fc + 1, &wt);
        assert_eq!(sc.player_cell_x, landed, "浮层期间不执行排队");
        assert_eq!(sc.tap_target, Some((2, ROW)), "浮层期间排队暂缓、标记仍在");
        sc.overlay_active = false;
        settle(&mut sc, fc + 2, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "关框后执行排队");
        assert!(!sc.route.is_open() && sc.tap_target.is_none());

        // 落格时按住方向键（朝下顶墙）：手动输入优先 ⇒ 排队作废
        let mut sc = scene((1, ROW), &[]);
        let wt = sc.width_table.clone();
        sc.press_key(-2); // 按住不放（press_key 发生在排队之前，清不到）
        sc.tick(0, &wt);
        assert_eq!((sc.player_cell_x, sc.walk_phase), (1, 0), "前置：顶墙不动");
        assert_eq!(sc.tap_cell(10, ROW), TapResult::Walking);
        let mut fc = 1;
        while sc.player_cell_x < 4 {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200);
        }
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        settle(&mut sc, fc, 200);
        assert_eq!(sc.key_held, -2, "前置：全程按住");
        assert!(sc.player_cell_x >= 4, "按住键作废排队（实得 {}）", sc.player_cell_x);
        assert!(!sc.route.is_open() && sc.tap_target.is_none() && sc.tap_pending.is_none());
    }

    /// 滑动：空闲 = 一格（朝墙 = 撞停并转向）；寻路中 = 排队，排空回 Map 后朝该方向踏一步。
    #[test]
    fn t7_swipe_steps_when_idle_and_queues_while_walking() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((3, ROW), &[]);
        assert_eq!(sc.swipe_step(2), TapResult::Walking);
        settle(&mut sc, 0, 40);
        assert_eq!(sc.player_cell_x, 4, "空闲滑动 = 一格");
        assert_eq!(sc.swipe_step(1), TapResult::Bumping, "朝墙滑 = 撞停");
        assert_eq!(sc.facing, 1, "撞墙后朝向 = 滑动方向（BUG-047）");

        let _ = sc.tap_cell(10, ROW);
        let wt = sc.width_table.clone();
        let mut fc = 40;
        while sc.player_cell_x < 6 {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 300);
        }
        assert_eq!(sc.swipe_step(3), TapResult::Queued);
        while sc.game_mode != GameMode::Map {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 400);
        }
        let landed = sc.player_cell_x;
        settle(&mut sc, fc, 60);
        assert_eq!(sc.player_cell_x, landed - 1, "排空落地后朝左踏一步");

        // 【计划 2 T7 复核 R5（变异 Ml）】排队 Cell 后再滑动：滑动覆盖排队、目标框立即清除
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued, "前置：排队 Cell");
        assert_eq!(sc.tap_target, Some((2, ROW)), "前置：目标框在排队目标");
        assert_eq!(sc.swipe_step(2), TapResult::Queued);
        assert_eq!(sc.tap_target, None, "滑动排队 ⇒ 目标框清除");
        assert_eq!(sc.tap_pending, Some(crate::tap_walk::PendingTap::Swipe(2)), "滑动覆盖排队 Cell");
        let wt = sc.width_table.clone();
        sc.tick(fc, &wt);
        assert_eq!(sc.tap_target, None, "下一拍目标框仍不在");
    }

    /// 滑动的非空闲态：观察模式忽略；手动行走 / 战斗 / Map 语境脚本行走 = Rejected（无格可闪红）；浮层开着 = Ignored。
    #[test]
    fn t7_swipe_ignored_or_rejected_when_not_idle() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((3, ROW), &[]);
        sc.walk_phase = 2;
        assert_eq!(sc.swipe_step(2), TapResult::Ignored, "观察模式");
        sc.walk_phase = 1;
        assert_eq!(sc.swipe_step(2), TapResult::Rejected, "手动行走中");
        sc.walk_phase = 5;
        assert_eq!(sc.swipe_step(2), TapResult::Rejected, "战斗中");
        sc.walk_phase = 0;
        sc.script_walk_armed = true;
        assert_eq!(sc.swipe_step(2), TapResult::Rejected, "Map 语境脚本行走");
        sc.script_walk_armed = false;
        assert!(sc.tap_reject.is_none(), "滑动无目标格、不闪红");
        sc.overlay_active = true;
        assert_eq!(sc.swipe_step(2), TapResult::Ignored, "浮层开着");
        sc.overlay_active = false;
        assert_eq!(sc.swipe_step(4), TapResult::Ignored, "非法方向码");
        assert_eq!((sc.player_cell_x, sc.walk_phase), (3, 0), "全程未动");
    }

    /// 【spec §5.2 · T5 复核 R3】末步在途窗口（Map wp1 ∧ 后续就绪）点别处 ⇒ 排队：`interrupt_route` 为空操作，
    /// 末步照常落格，落格后的后续拍改道去新目标。
    #[test]
    fn t7_tap_in_last_step_window_queues_reroute() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Walking);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !(sc.game_mode == GameMode::Map && sc.walk_phase == 1 && sc.tap_followup_ready) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：末步在途窗口");
        }
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "前置：末步 (5,ROW)→(6,ROW) 在途");
        assert_eq!(sc.tap_cell(3, ROW), TapResult::Queued);
        assert_eq!(sc.tap_target, Some((3, ROW)), "标记立即换到新目标");
        assert_eq!(sc.tap_reject, None, "不闪红");
        settle(&mut sc, fc, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (3, ROW), "落格后改道到新目标");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(!sc.route.is_open() && sc.tap_target.is_none() && sc.tap_pending.is_none());
    }

    /// 寻路途经拾取物弹框时点别处 / 滑动：浮层开着 `accepts_field_tap` 为假 ⇒ Ignored、不排队；关框后续走原目标。
    #[test]
    fn t7_tap_while_pickup_overlay_mid_route_is_ignored() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(25, 5, ROW)]);
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Walking);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !sc.overlay_active && fc < 200 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.overlay_active, "前置：拾取型 25 弹提示框");
        assert!(!sc.accepts_field_tap(), "浮层开着不收点地");
        assert_eq!(sc.tap_cell(3, ROW), TapResult::Ignored);
        assert_eq!(sc.swipe_step(3), TapResult::Ignored);
        assert!(sc.tap_pending.is_none(), "不排队");
        assert_eq!(sc.tap_target, Some((9, ROW)), "原目标标记不变");
        assert!(sc.route.is_open(), "路段未打断");
        sc.press_key(-5);
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        assert!(!sc.overlay_active, "提示框已关");
        settle(&mut sc, fc, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (9, ROW), "关框后续走原目标");
    }

    /// 排队点怪：落格后按 `tap_cell` 语义 = 首点选中（不走过去、不开战）。
    #[test]
    fn t7_queued_tap_on_monster_selects_after_landing() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((5, ROW), &[(41, 2, ROW)]);
        strong(&mut sc);
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        assert_eq!(sc.tap_cell(10, ROW), TapResult::Walking);
        let fc = walk_until_x(&mut sc, 7);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        settle(&mut sc, fc, 200);
        assert_eq!(sc.tap_selected.map(|s| s.entity), Some(e), "落格后选中");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (8, ROW), "停在排空落格，不走向怪");
        let ent = &sc.entities.items[e];
        assert!(!ent.removed && ent.state == 0, "未开战");
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map);
        assert_eq!(sc.tap_target, None, "选中不画目标框");
    }

    /// 末步在途窗口滑动 ⇒ 排队（与点按同一"寻路中"判据），落格后朝该方向踏一步。
    #[test]
    fn t7_swipe_in_last_step_window_queues() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Walking);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !(sc.game_mode == GameMode::Map && sc.walk_phase == 1 && sc.tap_followup_ready) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：末步在途窗口");
        }
        assert_eq!(sc.swipe_step(2), TapResult::Queued);
        settle(&mut sc, fc, 60);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (7, ROW), "末步落 (6,ROW) 后朝右再踏一步");
    }

    /// 撤销 / 换层（clear_tap_state）清排队：之后排空落格不改道。
    #[test]
    fn t7_queued_tap_cleared_by_clear_tap_state() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        sc.clear_tap_state();
        assert!(sc.tap_pending.is_none() && sc.tap_target.is_none());
        settle(&mut sc, fc, 200);
        assert_eq!(sc.player_cell_x, 5, "排空落格后不改道");

        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let _ = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        sc.load_floor_data(&super::scene_test_support::maplv(), 5 << 5, ROW << 5, &[], 2).unwrap();
        assert!(sc.tap_pending.is_none(), "换层清排队");
    }

    /// 排队已收尾、执行排在下一拍开头的间隙里再点按（Map wp0 新路线）：新点按取代旧排队——新路线走完后不再执行旧排队。
    #[test]
    fn t7_new_tap_in_followup_gap_supersedes_queue() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let mut fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        let wt = sc.width_table.clone();
        while !(sc.tap_followup_ready && sc.game_mode == GameMode::Map && sc.walk_phase == 0) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：收尾间隙");
        }
        assert_eq!(sc.player_cell_x, 5, "前置：排空落在 (5,ROW)");
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Walking, "间隙里点按 = 新路线");
        settle(&mut sc, fc, 300);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (8, ROW), "新路线走完即停，不再执行旧排队（回 x=2）");
        assert!(sc.tap_pending.is_none() && sc.tap_target.is_none());
    }

    /// 【计划 2 T7 复核 R6 · spec §5.2 更正②】撞击后续所在路线途中被隐形怪截住开战（路段挂起）：撞击后续与目标框在挂起期间保留，
    /// 胜利后路线续走到停靠格 (7,ROW)，照常撞开门 (8,ROW) 并走进门格（"点哪里就往哪里走"）。
    #[test]
    fn t7_bump_survives_battle_suspension_mid_route() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 5, ROW), (1, 8, ROW)]);
        hide_types(&mut sc, &[41]); // 未显形的怪：tap_grid 视为地面，try_step 照样开战
        strong(&mut sc);
        sc.player.yellow_keys = 1;
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping);
        assert_eq!(sc.tap_bump, Some(((7, ROW), 2)), "前置：撞击后续在");
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while sc.walk_phase != 5 {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：途中开战");
        }
        assert!(!sc.route.is_open() && sc.route.is_suspended(), "前置：路段挂起");
        sc.tick(fc, &wt);
        fc += 1;
        assert_eq!(sc.tap_bump, Some(((7, ROW), 2)), "开战挂起期间撞击后续保留");
        assert_eq!(sc.tap_target, Some((8, ROW)), "挂起期间目标框保留");
        settle(&mut sc, fc, 400);
        let i41 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        assert!(sc.entities.items[i41].removed || sc.entities.items[i41].state == 1, "前置：怪被击杀");
        assert_eq!(sc.player.yellow_keys, 0, "胜利后续走到停靠格并撞门耗钥");
        assert!(!door_intact(&sc, 8), "门已开");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (8, ROW), "走进门格");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(sc.tap_bump.is_none() && sc.tap_target.is_none(), "收尾后无残留");
    }

    /// 【计划 2 T7 复核 R6】路段开战挂起期间，壳层投键口径（win `route_key` 抬键前先 `interrupt_route`）解除挂起：Dialog dp3 下
    /// interrupt_route 置排空位 ⇒ 仍算"点地寻路中"，撞击后续留到排空收尾；胜利 restep 落在怪格 (5,ROW) ≠ 停靠格 ⇒ 撞击作废，不撞门。
    /// 按键在点地之前按下（按住顶墙），故 press_key 清不到撞击；抬键只走 release_key（不清撞击）。
    #[test]
    fn t7_shell_interrupt_during_battle_suspension_stops_without_bump() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 5, ROW), (1, 8, ROW)]);
        hide_types(&mut sc, &[41]);
        strong(&mut sc);
        sc.player.yellow_keys = 1;
        let wt = sc.width_table.clone();
        sc.press_key(-1); // 按住顶墙
        sc.tick(0, &wt);
        assert_eq!((sc.player_cell_x, sc.walk_phase), (2, 0), "前置：顶墙不动");
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping);
        let mut fc = 1;
        while sc.walk_phase != 5 {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：途中开战");
        }
        assert!(sc.route.is_suspended() && sc.tap_bump.is_some(), "前置：挂起中、撞击后续在");
        sc.interrupt_route(); // win route_key：抬键前先打断
        sc.release_key();
        assert!(!sc.route.is_suspended() && sc.walkto_draining, "前置：挂起解除、转排空");
        settle(&mut sc, fc, 400);
        let i41 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        assert!(sc.entities.items[i41].removed || sc.entities.items[i41].state == 1, "前置：怪被击杀");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "胜利 restep 落怪格即停");
        assert_eq!(sc.player.yellow_keys, 1, "未耗钥");
        assert!(door_intact(&sc, 8), "门未开");
        assert!(sc.tap_bump.is_none() && sc.tap_target.is_none() && !sc.route.is_open());
    }

    /// 跑到多步路线的末步在途窗口（`Map ∧ walk_phase == 1 ∧ tap_followup_ready`），返回下一拍帧号。
    fn run_to_last_step_window(sc: &mut GameScene, mut fc: i64) -> i64 {
        let wt = sc.width_table.clone();
        while !(sc.game_mode == GameMode::Map && sc.walk_phase == 1 && sc.tap_followup_ready) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 300, "前置：末步在途窗口");
        }
        fc
    }

    /// 手动行走中（Map wp1、非点地寻路）的点地 / 滑动断言：点地 Rejected 且闪红、滑动 Rejected、无排队 / 撞击 / 就绪位 / 目标框。
    fn assert_manual_walk_rejects(sc: &mut GameScene, tap: (i32, i32), label: &str) {
        use crate::tap_walk::TapResult;
        assert!(sc.game_mode == GameMode::Map && sc.walk_phase == 1, "{label} 前置：手动行走中");
        assert!(!sc.tap_followup_ready, "{label}：就绪位已作废");
        assert_eq!(sc.tap_cell(tap.0, tap.1), TapResult::Rejected, "{label}：手动行走中点地 = Rejected");
        assert!(matches!(sc.tap_reject, Some((c, _)) if c == tap), "{label}：闪红");
        assert_eq!(sc.swipe_step(2), TapResult::Rejected, "{label}：手动行走中滑动 = Rejected");
        assert!(sc.tap_pending.is_none() && sc.tap_bump.is_none(), "{label}：无排队 / 撞击");
        assert_eq!(sc.tap_target, None, "{label}：目标框清除");
    }

    /// 【计划 2 T7 复核 R1（探针 P1 / P11）· spec §5.2 更正③】末步在途窗口里按下并按住可走方向（左）：
    /// press_key 作废排队与就绪位；落格那一拍按 keyHeldCode 接上手动步 ⇒ 手动行走中点地闪红、滑动拒绝，松键后不执行旧排队。
    #[test]
    fn t7_held_key_in_last_step_window_takes_over() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Walking);
        let mut fc = run_to_last_step_window(&mut sc, 0);
        assert_eq!(sc.player_cell_x, 5, "前置：末步 (5,ROW)→(6,ROW) 在途");
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Queued, "前置：窗口里排队");
        sc.press_key(-3); // 左，按住
        let wt = sc.width_table.clone();
        while !(sc.player_cell_x == 6 && sc.walk_phase == 1 && sc.facing == 3) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 300, "前置：落格后接上手动步");
        }
        assert_manual_walk_rejects(&mut sc, (9, ROW), "窗口内按住");
        sc.release_key();
        settle(&mut sc, fc, 200);
        assert!(sc.player_cell_x < 6, "松键后停在手动步落格，不执行旧排队去 x=9（实得 {}）", sc.player_cell_x);
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map && sc.walk_phase == 0);
        assert!(sc.tap_pending.is_none() && sc.tap_target.is_none());
    }

    /// 【计划 2 T7 复核 R1（探针 P11b）】点地前已按住方向键（左，在 x=1 顶墙不动）再点门：路线照走；末步窗口里 keyHeldCode ≠ 0
    /// ⇒ tick_tap_state 作废就绪位与撞击；落格那一拍接上手动步（左）⇒ 手动行走中点地闪红、滑动拒绝，松键后不撞门。
    #[test]
    fn t7_key_held_before_tap_takes_over_after_last_step() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[(1, 8, ROW)]);
        sc.player.yellow_keys = 1;
        let wt = sc.width_table.clone();
        sc.press_key(-3); // 按住不放（press_key 发生在点地之前，清不到之后的撞击）
        sc.tick(0, &wt);
        assert_eq!((sc.player_cell_x, sc.walk_phase), (1, 0), "前置：顶墙不动");
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping);
        let mut fc = run_to_last_step_window(&mut sc, 1);
        assert_eq!(sc.player_cell_x, 6, "前置：末步 (6,ROW)→(7,ROW) 在途");
        sc.tick(fc, &wt);
        fc += 1;
        assert!(!sc.tap_followup_ready && sc.tap_bump.is_none(), "窗口内按住 ⇒ 就绪位与撞击作废");
        while !(sc.player_cell_x == 7 && sc.walk_phase == 1 && sc.facing == 3) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 300, "前置：落格后接上手动步");
        }
        assert_manual_walk_rejects(&mut sc, (3, ROW), "点地前按住");
        sc.release_key();
        settle(&mut sc, fc, 200);
        assert!(sc.player_cell_x < 7, "松键后停在手动步落格（实得 {}）", sc.player_cell_x);
        assert_eq!(sc.player.yellow_keys, 1, "未耗钥");
        assert!(door_intact(&sc, 8), "门未开");
        assert!(sc.tap_bump.is_none() && sc.tap_pending.is_none() && sc.tap_target.is_none());
    }

    /// 【计划 2 T7 复核 R1】末步窗口内按下又在下一拍前松开（tick_tap_state 看不到 keyHeldCode）：press_key 本身作废就绪位 ⇒
    /// 这一步余下的拍里点地不再算"点地寻路中"（Rejected 闪红、不排队），落格后不改道。
    #[test]
    fn t7_key_tapped_in_last_step_window_clears_followup() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Walking);
        let fc = run_to_last_step_window(&mut sc, 0);
        sc.press_key(-2); // 下（墙）
        sc.release_key();
        assert!(!sc.tap_followup_ready, "场上按键作废就绪位");
        assert_eq!(sc.tap_cell(3, ROW), TapResult::Rejected, "末步余下拍 = 手动输入后的行走：点地 Rejected");
        assert!(matches!(sc.tap_reject, Some(((3, r), _)) if r == ROW), "闪红");
        assert!(sc.tap_pending.is_none(), "不排队");
        settle(&mut sc, fc, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "落在原末步终点，不改道");
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map && sc.tap_target.is_none());
    }

    /// 【计划 2 T7 复核 R2（探针 P10）· spec §5.2 更正④】走向怪途中再点同一只（同格 = 当前 tap_target）：不打断、不排队、
    /// 撞击保留 ⇒ 照常开战（不降级为选中）。
    /// 已知限制（保持现状）：多格怪点其**另一格**（≠ tap_target）按"点别处"排队；首点 / 再点时选中已清 ⇒ 落格执行为首点 = 选中，不开战。
    #[test]
    fn t7_retap_current_monster_while_approaching_still_fights() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 8, ROW)]);
        strong(&mut sc);
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Selected);
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping, "再点 = 开战");
        let bump = sc.tap_bump;
        assert!(bump.is_some(), "前置：撞击后续在");
        let fc = walk_until_x(&mut sc, 4);
        let steps = sc.walk_step_count;
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping, "途中再点同一只 = 原路线继续");
        assert_eq!(sc.tap_bump, bump, "撞击保留");
        assert!(sc.route.is_open() && !sc.walkto_draining, "未打断");
        assert_eq!(sc.walk_step_count, steps, "步序未清");
        assert!(sc.tap_pending.is_none(), "不排队");
        assert_eq!(sc.tap_target, Some((8, ROW)), "目标框不变");
        settle(&mut sc, fc, 400);
        let ent = &sc.entities.items[e];
        assert!(ent.removed || ent.state == 1, "照常开战并击杀");
        assert!(journal_has_combat(&mut sc), "战斗笔入栈");

        // 已知限制：多格怪 69（宽 3：(7..=9,ROW)）再点 (7,ROW) 开战后，途中点其另一格 (8,ROW) ⇒ 排队 ⇒ 落格后 = 选中、不开战
        let mut sc = scene((2, ROW), &[(69, 7, ROW)]);
        strong(&mut sc);
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 69).unwrap();
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Selected);
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Bumping, "再点同一只另一格 = 开战");
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Queued, "已知限制：另一格 ≠ tap_target ⇒ 按点别处排队");
        settle(&mut sc, fc, 200);
        assert_eq!(sc.tap_selected.map(|s| s.entity), Some(e), "已知限制：落格执行 = 首点选中");
        let ent = &sc.entities.items[e];
        assert!(!ent.removed && ent.state == 0, "已知限制：未开战");
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map);
    }

    /// 【计划 2 T7 复核 R2】走向门途中再点门：不停顿（与不再点的对照路线同一拍开门），照常耗钥开门、走进门格。
    #[test]
    fn t7_retap_current_door_while_walking_keeps_route() {
        use crate::tap_walk::TapResult;
        let open_frame = |retap: bool| -> (i64, (i32, i32)) {
            let mut sc = scene((2, ROW), &[(1, 8, ROW)]);
            sc.player.yellow_keys = 1;
            assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping);
            let wt = sc.width_table.clone();
            let mut fc = 0;
            while sc.player.yellow_keys != 0 {
                if retap && sc.player_cell_x == 4 && sc.game_mode == GameMode::Dialog && sc.tap_pending.is_none() {
                    let steps = sc.walk_step_count;
                    assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping, "途中再点门 = 原路线继续");
                    assert!(sc.route.is_open() && !sc.walkto_draining && sc.walk_step_count == steps, "未打断");
                    assert!(sc.tap_bump.is_some() && sc.tap_pending.is_none(), "撞击保留、不排队");
                }
                sc.tick(fc, &wt);
                fc += 1;
                assert!(fc < 400, "开门");
            }
            let open = fc;
            settle(&mut sc, fc, 100);
            assert!(!door_intact(&sc, 8), "门已开");
            assert!(sc.tap_target.is_none() && sc.tap_bump.is_none());
            (open, (sc.player_cell_x, sc.player_cell_y))
        };
        let control = open_frame(false);
        assert_eq!(control.1, (8, ROW), "对照：走进门格");
        assert_eq!(open_frame(true), control, "再点门不停顿：同一拍开门、同一落格");
    }

    /// 【计划 2 T7 复核 R2】排队后再点排队目标：仍 Queued，排队与目标框不变，落格后照常改道。
    #[test]
    fn t7_retap_queued_target_keeps_queue() {
        use crate::tap_walk::{PendingTap, TapResult};
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued, "再点排队目标 = 仍 Queued");
        assert_eq!(sc.tap_pending, Some(PendingTap::Cell(2, ROW)), "排队不变");
        assert_eq!(sc.tap_target, Some((2, ROW)), "目标框不变");
        settle(&mut sc, fc, 300);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "落格后照常改道");
        assert!(sc.tap_pending.is_none() && sc.tap_target.is_none());
    }

    /// 跑到排队收尾间隙（排空已落 (5,ROW)、Map wp0、就绪位在、排队 (2,ROW) 在），返回下一拍帧号。
    fn queued_followup_gap() -> (GameScene, i64) {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let mut fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        let wt = sc.width_table.clone();
        while !(sc.tap_followup_ready && sc.game_mode == GameMode::Map && sc.walk_phase == 0) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：收尾间隙");
        }
        assert_eq!(sc.player_cell_x, 5, "前置：排空落在 (5,ROW)");
        assert!(sc.tap_pending.is_some(), "前置：排队在");
        (sc, fc)
    }

    /// 【计划 2 T7 复核 R4（变异 Mi）】收尾间隙里壳直接调 `walk_to`：新路线取代旧排队——走完后不再执行旧排队（回 x=2）。
    #[test]
    fn t7_walk_to_in_followup_gap_supersedes_queue() {
        let (mut sc, fc) = queued_followup_gap();
        assert!(sc.walk_to(8, ROW).reached, "间隙里 walk_to = 新路线");
        assert!(sc.tap_pending.is_none(), "walk_to 清排队");
        settle(&mut sc, fc, 300);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (8, ROW), "新路线走完即停，不再执行旧排队");
        assert!(sc.tap_pending.is_none() && sc.tap_target.is_none());
    }

    /// 【计划 2 T7 复核 R4（变异 Mj）】收尾间隙里壳直接调 `probe_step`：探步取代旧排队——探步落格后不再执行旧排队（回 x=2）。
    #[test]
    fn t7_probe_step_in_followup_gap_supersedes_queue() {
        let (mut sc, fc) = queued_followup_gap();
        assert!(sc.probe_step(2), "间隙里探步（右）放行");
        assert!(sc.tap_pending.is_none(), "probe_step 清排队");
        settle(&mut sc, fc, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "探步落 (6,ROW) 即停，不再执行旧排队");
        assert!(sc.tap_pending.is_none() && sc.tap_target.is_none());
    }

    // ------- Task 8：点地反馈绘制（spec §6，modern 专用、classic 零 op） -------

    fn marker_ops(sc: &GameScene) -> Vec<String> {
        let mut img = crate::render::ArgbImage::create(sc.layout.screen_w, sc.layout.screen_h);
        let mut g = crate::render::SoftGraphics::new(&mut img);
        sc.paint_tap_markers(&mut g, sc.view.cam_x, sc.view.cam_y + sc.layout.view_top_y);
        g.ops.clone()
    }

    fn full_frame_ops(sc: &mut GameScene) -> Vec<String> {
        let mut img = crate::render::ArgbImage::create(sc.layout.screen_w, sc.layout.screen_h);
        let mut g = crate::render::SoftGraphics::new(&mut img);
        sc.paint(&mut g, &crate::render::ArgbImage::create(2, 2));
        g.ops.clone()
    }

    fn put_markers(sc: &mut GameScene) {
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        sc.tap_target = Some((5, ROW));
        sc.tap_reject = Some(((4, ROW), sc.frame_counter + crate::tap_walk::TAP_REJECT_TICKS));
        sc.tap_selected = Some(crate::tap_walk::SelectedEnemy { entity: e, type_: 41, cell: (8, ROW), floor: sc.current_floor });
    }

    /// 【STAGE-B-008 · spec §6】classic：反馈层零 op，整帧 op 流与无点地状态时逐条相同。
    #[test]
    fn t8_markers_classic_zero_ops() {
        let mut a = scene((2, ROW), &[(41, 8, ROW)]);
        let mut b = scene((2, ROW), &[(41, 8, ROW)]);
        put_markers(&mut a);
        assert!(marker_ops(&a).is_empty(), "classic 反馈层零 op");
        assert_eq!(full_frame_ops(&mut a), full_frame_ops(&mut b), "classic 整帧 op 流不变");
    }

    /// modern：目标金框 / 闪红红框加叉（到期即不画）/ 选中闪烁框；火眼金睛才画预计掉血（弱 = "无法战胜"字形、
    /// 强 = 数字条，与观察模式同位：通用怪先 −ENTITY_LIFT(4) ⇒ y = 格顶 − 21）；多格怪 69 框宽 96；整帧里确有反馈层。
    #[test]
    fn t8_markers_modern_draw_target_reject_and_selection() {
        let mut sc = scene((2, ROW), &[(41, 8, ROW)]);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        put_markers(&mut sc);
        let (ox, oy) = (sc.view.cam_x, sc.view.cam_y + sc.layout.view_top_y);
        let ops = marker_ops(&sc);
        assert!(ops.contains(&format!("drawRect({},{},32,32,#ffe070)", ox + 5 * 32, oy + ROW * 32)), "目标金框：{ops:?}");
        assert!(ops.iter().any(|o| o.contains("#ff3030")), "闪红");
        assert!(ops.iter().any(|o| o.starts_with(&format!("drawRect({},{},32,32,", ox + 8 * 32, oy + ROW * 32))), "选中框");
        sc.frame_counter += crate::tap_walk::TAP_REJECT_TICKS;
        assert!(!marker_ops(&sc).iter().any(|o| o.contains("#ff3030")), "闪红到期不画");
        let without = marker_ops(&sc).len();
        sc.player.item_types[0] = 13;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        let ops = marker_ops(&sc);
        assert!(ops.len() > without, "持火眼金睛：多画预计掉血");
        let (m41x, m41y) = (ox + 8 * 32, oy + ROW * 32);
        assert!(
            ops.contains(&format!("setClip({},{},7,9)", m41x + 14, m41y - 21)),
            "弱方 = 无法战胜字形（观察模式同位：−ENTITY_LIFT−17）：{ops:?}"
        );
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
        let ops = marker_ops(&sc);
        assert!(!ops.iter().any(|o| o.ends_with(",7,9)")), "强方无字形：{ops:?}");
        assert!(
            ops.iter().any(|o| o.starts_with("setClip(") && o.ends_with(",2,24)")),
            "强方 = 数字条（夹具条 24×24 ⇒ 数位宽 2、高 24）：{ops:?}"
        );
        assert!(full_frame_ops(&mut sc).iter().any(|o| o.contains("#ffe070")), "整帧含反馈层");
    }

    /// 【T8 复核 R2】多格怪 69（宽 3 格）：选中框按实体宽 96 画；69 分支不抬 ⇒ 数字条 y = 格顶 − 17。
    #[test]
    fn t8_markers_multi_cell_monster_frame_and_no_lift() {
        let mut sc = scene((2, ROW), &[(69, 7, ROW)]);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 69).unwrap();
        sc.player.item_types[0] = 13;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
        sc.tap_selected = Some(crate::tap_walk::SelectedEnemy { entity: e, type_: 69, cell: (7, ROW), floor: sc.current_floor });
        let (ox, oy) = (sc.view.cam_x, sc.view.cam_y + sc.layout.view_top_y);
        let ops = marker_ops(&sc);
        assert!(
            ops.iter().any(|o| o.starts_with(&format!("drawRect({},{},96,32,", ox + 7 * 32, oy + ROW * 32))),
            "69 三格宽选中框：{ops:?}"
        );
        assert!(
            ops.iter().any(|o| o.starts_with("setClip(") && o.contains(&format!(",{},2,24)", oy + ROW * 32 - 17))),
            "69 不抬 ⇒ 数字条 y = 格顶 − 17：{ops:?}"
        );
    }

    // ------- T4：观察模式拖动 pan_viewport（spec §5.3） -------

        /// 【计划 3 T4】拖动平移：观察模式（wp2 未锁）按增量平移相机 + 主方向 facing；钳位同原版；
    /// 锁定期 / 非 wp2 / classic / 浮层 = 零变化。
    #[test]
    fn t4_pan_viewport_observe_drag() {
        let mut sc = scene((4, ROW), &[]);
        // 窄视口（宽 320 < 地图 416 ⇒ fits_w=false）：横向可平移（13×13 整层可见时 x 臂归零=原版语义）
        sc.apply_layout(crate::layout::Layout::modern(10, 13));
        // 进观察：-5（Map wp0）→ wp2 未锁定
        sc.key_value = -5;
        sc.handle_field_input();
        assert_eq!(sc.walk_phase, 2, "前置：观察模式");
        assert!(!sc.view_locked);
        let (cx0, cy0) = (sc.view.cam_x, sc.view.cam_y);
        // 向右拖 3 拍（每拍 16px）：内容跟手（cam+dx，复核 T4-R2 裁定）
        let mut moved = 0;
        for _ in 0..3 {
            let (dx, _) = sc.pan_viewport(16, 0);
            moved += dx;
        }
        assert_eq!(moved, 48, "三拍共移 48px（内容跟手）");
        assert_eq!(sc.view.cam_x, cx0 + moved, "拖右 = 内容右移（cam 增）");
        assert_eq!(sc.facing, 3, "镜头朝向 = 左（内容右移 ⇒ 看向更左）");
        // 上拖（dy<0 = 内容上移 = 镜头下）⇒ facing 下
        let _ = sc.pan_viewport(0, -16);
        assert_eq!(sc.facing, 0, "上拖 = 镜头下（facing 0）");
        // 对角（dx 优先；拖右 = 镜头左）
        let _ = sc.pan_viewport(16, -1);
        assert_eq!(sc.facing, 3, "对角 dx 优先（镜头左）");
        // 零增量
        let before = (sc.view.cam_x, sc.view.cam_y);
        assert_eq!(sc.pan_viewport(0, 0), (0, 0));
        assert_eq!((sc.view.cam_x, sc.view.cam_y), before);
        // 锁定期（回锁期）零变化（锁定态由 tick wp2 臂消费 -5 进入；纯查询直置）
        sc.view_locked = true;
        let (cx, cy) = (sc.view.cam_x, sc.view.cam_y);
        assert_eq!(sc.pan_viewport(32, 32), (0, 0), "回锁期拖动无效");
        assert_eq!((sc.view.cam_x, sc.view.cam_y), (cx, cy));
        // classic 零变化
        let mut cl = scene((4, ROW), &[]);
        cl.key_value = -5;
        cl.handle_field_input();
        assert_eq!(cl.walk_phase, 2);
        let c0 = (cl.view.cam_x, cl.view.cam_y);
        assert_eq!(cl.pan_viewport(64, 64), (0, 0), "classic 拖动无效");
        assert_eq!((cl.view.cam_x, cl.view.cam_y), c0);
    }
}


/// BUG-040：点地寻路（WalkTo）途中任意键打断 ⇒ 打断后的 dp3 失去"寻路段开放"判据，步尽时
/// 落入脚本机 dp0 的"行尾收尾"（写 script_line_flags[当前脚本] = true + dp4 等相机才回 Map）
/// ——2026-10-28 对抗复核探针 P3 实测相位轨迹 Dialog/dp3 → dp0 → dp4 → Map/dp4。
#[cfg(test)]
mod walkto_interrupt_tests {
    use super::scene_test_support::{scene, ROW};
    use super::*;

    /// 打断后走完当前这一步即回 Map；全程不进 dp0、不写当前脚本旗标、解除脚本行走武装
    /// （武装残留会让 Map 态下一次手动步进走 advanceScriptWalk 臂而不归 walkPhase 0）。
    #[test]
    fn walkto_interrupt_drains_to_map_without_script_tail() {
        let mut sc = scene((1, ROW), &[]);
        let cur = sc.current_script_index;
        sc.script_line_flags[cur] = false;
        sc.walk_to(5, ROW);
        let wt = sc.width_table.clone();
        sc.tick(0, &wt); // dp3 弹第一步、起步
        sc.press_key(-1); // 任意键打断（壳层投键口径）
        sc.release_key();
        let mut trail = Vec::new();
        for fc in 1..60 {
            sc.tick(fc, &wt);
            trail.push((sc.game_mode, sc.dialog_phase));
            if sc.game_mode == GameMode::Map && sc.walk_phase == 0 {
                break;
            }
        }
        assert_eq!(sc.game_mode, GameMode::Map, "打断后回 Map：{trail:?}");
        assert!(
            !trail.iter().any(|&(m, dp)| m == GameMode::Dialog && dp == 0),
            "不得落入脚本机 dp0：{trail:?}"
        );
        assert!(!sc.script_line_flags[cur], "当前脚本完成旗标不得被写");
        assert!(!sc.script_walk_armed, "脚本行走武装已解除");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "走完打断时的那一步后停下");
    }

    /// 剧情接管（踩剧情点 ⇒ loadLevelScript）后的 dp 归剧情所有：先前的打断不得让剧情中途切回 Map。
    #[test]
    fn script_takeover_after_interrupt_keeps_dialog() {
        let mut sc = scene((1, ROW), &[]);
        sc.dialog_data.scripts = vec!["TAK_0_0 ".to_string()];
        sc.walk_to(5, ROW);
        sc.press_key(-1); // 打断（未出拍）
        sc.load_level_script(0); // 剧情接管
        sc.find_path(1, ROW, 3, ROW); // 剧情 MOV 驱动的脚本行走
        sc.script_walk_armed = true;
        sc.dialog_phase = 3;
        let wt = sc.width_table.clone();
        for fc in 0..12 {
            sc.tick(fc, &wt);
            assert_eq!(sc.game_mode, GameMode::Dialog, "剧情行走期仍是 Dialog（拍 {fc}）");
        }
    }
}

/// BUG-040 同根第二症状（2026-10-28 本测 RED 证实：寻路后武装残留=true，按一下方向键走了 3 格
/// 仍 walkPhase 1）：单步点地寻路（点相邻格）由 dp3 wp0 弹出唯一一步，不经 advanceScriptWalk
/// 清武装 ⇒ 回 Map 后 script_walk_armed 残留 ⇒ 下一次手动步进步末走 advanceScriptWalk 臂
/// （无步可弹、不归 walkPhase 0）⇒ 停不下来。
#[cfg(test)]
mod walkto_single_step_tests {
    use super::scene_test_support::{scene, ROW};
    use super::*;

    #[test]
    fn single_step_walkto_then_manual_step_stops_at_next_cell() {
        let mut sc = scene((1, ROW), &[]);
        sc.walk_to(2, ROW);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while fc < 40 && !(sc.game_mode == GameMode::Map && sc.walk_phase == 0) {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!((sc.game_mode, sc.player_cell_x), (GameMode::Map, 2), "单步寻路到达");
        let armed_after_walkto = sc.script_walk_armed;
        sc.press_key(-4); // 手动向右一步
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        for _ in 0..12 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!(
            (sc.walk_phase, sc.player_cell_x, sc.view.player_px & 31),
            (0, 3, 0),
            "手动一步后停在下一格（寻路后武装残留={armed_after_walkto}）"
        );
    }
}

/// BUG-038：触屏点按穿透到下层 / 在场上非空闲时执行场上动作（Stage B 触屏层，2026-10-28
/// 对抗复核运行时探针 P1a/P1b/P2 实测）。原版浮层打开期间场上不吃任何输入（a.java:3252-3345
/// 浮层分支吞键后整拍返回），行走中 / 战斗中也不读场上输入（walkPhase 分派 a.java:3563-3679）。
#[cfg(test)]
mod touch_tap_guard_tests {
    use super::scene_test_support::{scene, ROW};
    use super::*;

    /// 地图上弹着提示框：点地寻路 / 探步一律拒绝，且拒绝 = 零状态变化（探针 P1a/P1b：
    /// 曾在框后挂上寻路并切到 Dialog、或在框开着时起步撞格）。
    #[test]
    fn field_taps_refused_while_overlay_open() {
        let mut sc = scene((1, ROW), &[]);
        sc.show_overlay_message(0, "提示", 0, 0);
        assert!(!sc.field_tap_allowed(), "弹框期不接受点地");
        let plan = sc.walk_to(5, ROW);
        assert!(!plan.reached && plan.steps.is_empty(), "拒绝寻路：{plan:?}");
        assert_eq!((sc.game_mode, sc.walk_phase, sc.walk_step_count), (GameMode::Map, 0, 0), "模式/相位/步数不变");
        assert!(!sc.route.is_open(), "未开寻路段");
        assert!(!sc.probe_step(2), "拒绝探步");
        assert_eq!(sc.walk_phase, 0, "探步未起步");
        assert!(sc.overlay_active, "弹框仍开着");
    }

    /// 行走中（walkPhase 1）、探索（2）、换层（4）、战斗（5）、换层遮幅中同样拒绝；空闲地图允许。
    #[test]
    fn field_taps_refused_unless_field_idle() {
        let mut sc = scene((1, ROW), &[]);
        assert!(sc.field_tap_allowed(), "空闲地图允许点地");
        for wp in [1, 2, 4, 5] {
            sc.walk_phase = wp;
            assert!(!sc.field_tap_allowed(), "walkPhase {wp} 不接受点地");
            let plan = sc.walk_to(5, ROW);
            assert!(plan.steps.is_empty() && sc.walk_phase == wp && !sc.route.is_open(), "walkPhase {wp}：walk_to 零变化");
            sc.facing = 0;
            assert!(!sc.probe_step(2) && sc.walk_phase == wp, "walkPhase {wp}：probe_step 零变化");
            assert_eq!(sc.facing, 0, "walkPhase {wp}：拒绝的探步不改朝向（BUG-047 写朝向须在守卫之后）");
        }
        sc.walk_phase = 0;
        sc.transitioning = true;
        assert!(!sc.field_tap_allowed(), "换层遮幅中不接受点地");
    }

    /// 弹框盖住的下层行 / 格不接受指针命中（探针 P2：道具使用确认框开着时点另一格 ⇒ 光标改到
    /// 被点那格并"使用"了它——框里显示的却是原先那件）。
    #[test]
    fn pointer_hit_ignores_rows_beneath_overlay() {
        let mut sc = scene((1, ROW), &[]);
        sc.player.item_types[0] = 16;
        sc.player.item_uses[0] = 1;
        sc.player.item_types[1] = 19;
        sc.player.item_uses[1] = 1;
        sc.player.item_stack_size = 2;
        sc.transition_to(GameMode::ItemPanel);
        sc.enter_item_panel();
        let (ox, oy) = sc.item_cell_origin();
        let cell1 = (ox + 40 + 20, oy + 20);
        assert!(
            matches!(sc.pointer_hit(cell1.0, cell1.1), PointerHit::Row { index: 1, .. }),
            "无弹框：命中 1 号格"
        );
        sc.show_overlay_message(3, "芭蕉扇", 1, 2);
        assert!(
            matches!(sc.pointer_hit(cell1.0, cell1.1), PointerHit::None),
            "确认框开着：下层道具格不可点"
        );
    }
}

/// BUG-039：点空白（未命中行 / 格）恒代按确定 -5（native.rs scene_tap 行外臂、runtime.rs step_boot、
/// win click_key 非 Map 态）⇒ 存档页点空白 = 覆盖当前槽、炼丹炉 = 花钱买、游戏内菜单 = 执行高亮项、
/// 设置页 = 切换高亮行、道具栏 = 选中高亮道具。只有"继续 / 关闭"类情境代按；原版以 ○/× 脚注
/// 标明的是否框（道具使用确认 kind 3、商人报价 kind 4）点空白不代按。
#[cfg(test)]
mod blank_tap_tests {
    use super::scene_test_support::{scene, ROW};
    use super::*;

    #[test]
    fn blank_tap_continues_only_in_continue_contexts() {
        let mut sc = scene((1, ROW), &[]);
        for (mode, want) in [
            (GameMode::Dialog, Some(-5)),
            (GameMode::Ending, Some(-5)),
            (GameMode::Map, None),
            (GameMode::InGameMenu, None),
            (GameMode::SaveSlot, None),
            (GameMode::Alchemy, None),
            (GameMode::ItemPanel, None),
            (GameMode::About, None),
            (GameMode::EnemyInfo, None),
            (GameMode::Help, None),
        ] {
            sc.game_mode = mode;
            assert_eq!(sc.blank_tap_key(), want, "{mode:?} 无浮层点空白");
        }
        sc.game_mode = GameMode::Map;
        for (kind, l, r, want) in [
            (0, 0, 0, Some(-5)), // 提示框：关闭
            (1, 0, 3, Some(-5)), // 拾取说明：关闭
            (3, 0, 3, Some(-5)), // 说明类道具：关闭
            (4, 0, 0, Some(-5)), // 赠礼 / 告示：关闭
            (3, 1, 2, None),     // 道具使用确认（○/×）：不代按
            (4, 1, 2, None),     // 商人报价（○/×）：不代按
        ] {
            sc.show_overlay_message(kind, "x", l, r);
            assert_eq!(sc.blank_tap_key(), want, "浮层 kind {kind} 脚注 ({l},{r}) 点空白");
        }
        sc.show_overlay_message(0, "x", 0, 0);
        sc.transitioning = true;
        assert_eq!(sc.blank_tap_key(), None, "换层遮幅期（浮层不画却照吞键）点空白不代按");
    }

    /// 【STAGE-B-008 · spec §5.2】赠予 / 报价（kind 4、型 77、flags&2）点空白不代按；同实体 flags&2 = 0（告示子类）照旧代按 −5；
    /// 型 78 报价（○/× 图标）本就不代按。
    #[test]
    fn blank_tap_does_not_accept_type77_offer() {
        let mut sc = scene((2, ROW), &[(77, 5, ROW), (78, 7, ROW)]);
        let i77 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 77).unwrap();
        let i78 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 78).unwrap();
        sc.overlay_active = true;
        sc.overlay_kind = 4;
        sc.merchant_entity = i77 as i32;
        sc.entities.items[i77].param = (2 << 8) | 1;
        sc.overlay_left_icon = 0;
        sc.overlay_right_icon = 0;
        assert_eq!(sc.blank_tap_key(), None, "型 77 报价：点空白不接受");
        sc.entities.items[i77].param = 1;
        assert_eq!(sc.blank_tap_key(), Some(-5), "型 77 告示子类（flags&2 = 0）：点空白关闭");
        sc.merchant_entity = i78 as i32;
        sc.entities.items[i78].param = (2 << 8) | 1;
        sc.overlay_left_icon = 1;
        sc.overlay_right_icon = 2;
        assert_eq!(sc.blank_tap_key(), None, "型 78 报价（○/×）不代按（原规则）");
        sc.merchant_entity = -1;
        sc.overlay_left_icon = 0;
        sc.overlay_right_icon = 0;
        assert_eq!(sc.blank_tap_key(), Some(-5), "merchant_entity=-1 的 kind 4（测试构造态；生产长文浏览必经 case16 设 merchant 且靠 &2=0 放行，复核 T9-R2）照旧代按");
    }
}
