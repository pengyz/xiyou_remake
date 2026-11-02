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

/// 【task-15 步②】渲染簇（move-only——impl GameScene 分布多文件，蓝图 §0）。
#[path = "scene_paint.rs"]
mod scene_paint;

/// 【task-15 步③】菜单族（move-only——enter/tick/paint 三件套）。
#[path = "scene_menu.rs"]
mod scene_menu;

/// 【task-15 步④】装载/存档/换层（move-only）。
#[path = "scene_persist.rs"]
mod scene_persist;

/// 【task-15 步⑤】脚本解释器（move-only；16 op 冻结面）。
#[path = "scene_script.rs"]
mod scene_script;

/// 【task-15 V1】脚本 op 归位（enum+22 impls+parse/execute 三处汇 1，move-only）。
/// 【task-15 V2 dialog-VM】对话/浮层域（tick_dialog 归位）。
/// 【task-15 V2 item/effect】道具效果域（表驱动仿 RestoreAction）。
#[path = "item_effect.rs"]
mod item_effect;

#[path = "scene_dialog.rs"]
mod scene_dialog;

#[path = "script_ops.rs"]
mod script_ops;
/// 【task-15 V1】恢复规则归位（类型+表+执行器汇 1，move-only）。
#[path = "restore_rules.rs"]
mod restore_rules;
pub(crate) use script_ops::ScriptOp;
pub use restore_rules::{RestoreAction, RestoreRule, RESTORE_RULES};
use scene_persist::PersistState;

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
    /// frameCounter（run 循环变量，TICK n 拍 = n-1）
    pub frame_counter: i64,
    /// viewBottom：视口底（HUD 顶）——初始化后与 layout::view_bottom_y() 一致
    pub view_bottom: i32,
    /// 玩家态（paintPlayerSprite 输入）
    pub facing: i32,
    pub walk_phase: i32,
    pub player_frame: i32,
    pub player_bob_applied: bool, // player_bob_applied
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
    /// equippedWeaponType/ArmorType（resetNewGameState 重置；tryEquip 更新；HUD 槽显示）
    pub equipped_weapon: i32,
    pub equipped_armor: i32,
    /// equipTierBonuses/Types（a.java:977-991 构造常量）
    pub equip_bonuses: [i32; 12],
    pub equip_types: [i32; 12],
    /// 组开门动画数组（tryOpenChest 填充；paintTileLayer walkPhase4 消耗）
    pub open_group: Vec<usize>,

    // —— mode 11 对话（gameMode 3=世界 / 11=对话；a.java:4011-4068 相位机）——
    pub game_mode: GameMode,
    /// 转移计数（件 A' 收口观测：transition_to 调用次数）
    pub transition_count: u32,
    /// 转移日志（件 A' 收口观测：最后一次 (from, to)）——观测字段，不进金样 dump
    pub last_transition: Option<(GameMode, GameMode)>,
    /// mode20 结局卷轴状态（Java f_int_22/f_int_23/f_bool_04——dispatchModeEntry case20
    /// **不重置**，场景构造期零值即 Java 新运行初值）。
    pub ending_scroll: i32,
    pub ending_anim: i32,
    pub ending_done: bool,
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
    /// scriptWalkArmed（脚本行走武装；phase 3 门控）
    pub script_walk_armed: bool,
    /// 【task-15 V2 dialog-VM】对话/浮层域切片（22 字段→DialogState）。
    pub dialog: scene_dialog::DialogState,
    /// 【task-15 V2 menu/UI】菜单/UI 域切片（42 字段→MenuState）。
    pub menu: scene_menu::MenuState,
    /// 音频语义（crate::audio；docs/spec/audio.md）。soundEnabled 的唯一真相源
    ///（run case16 label0 翻转，a.java:4157）。独立构造默认 = 「刚进 mode 3、声音开」；
    /// 宿主从 BootMachine 交接时整体覆盖（Java 为同一对象，状态连续）。
    pub bgm: crate::audio::BgmState,
    /// shopRainbowPhase：商店条目彩虹色相位（paint 每 4 拍推进）
    pub rainbow_phase: i32,
    /// pathArrowsVisible：路径箭头可见（SEE flag=1 且 findPath 成功）
    pub path_arrows_visible: bool,
    /// walkPathBuffer（findPath 步序，目标端优先）
    pub walk_path_buffer: Vec<u8>,
    /// 【T2】操作级回撤 journal（T1 模块；只在 WalkTo/探步寻路段落笔——经典面零写入）
    pub undo: crate::undo_journal::UndoJournal,
    /// 【task-15 V2 persist 收口】持久域切片（13 字段→PersistState，唯一归属）。
    pub persist: PersistState,
    /// 【T2】WalkTo 路段追踪器（位移笔=段起终点/中断挂起=实际终点/战斗笔=终结时）
    /// 【task-15 V2 render】副作用态切片（E3 六字段→RenderState；变更点留 paint 同时点）。
    pub render: scene_paint::RenderState,
    pub route: crate::pathfind::RouteTracker,
    /// 【BUG-040 · T2】点地寻路被打断、但 dp3 仍归寻路所有（走完当前步即回 Map）。打断会收束
    /// 寻路段（route 不再开放），dp3 收尾原本只凭 `route.is_open()` 认寻路 ⇒ 打断后落入脚本机
    /// dp0 行尾收尾。置位 = interrupt_route 时处于 Dialog dp3；清位 = 新 walk_to / 剧情接管 / 回 Map。
    walkto_draining: bool,
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
    /// 【task-15 V2 tap/input 收口】触屏/行走域切片（11 字段→TapState）。
    pub tap: crate::tap_walk::TapState,
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
    scene.persist.floor = save.current_floor as i32;
    scene.persist.current_floor = save.current_floor as i32;
    scene.persist.min_floor = save.min_floor_reached as i32;
    scene.persist.max_floor = save.max_floor_reached as i32;
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
    scene.persist.journal = save.journal.iter().map(|(k, i)| (*k as i32, *i as i32)).collect();
    scene.player.item_stack_size = save.item_stack_size as i32;
    scene.alchemy_upgrade_count = save.alchemy_upgrades as i32; // a.java:9281 链（loadFloorSave 应用）
    for k in 0..save.item_stack_size as usize {
        scene.player.item_types[k] = save.item_stack_types[k];
        scene.player.item_uses[k] = save.item_stack_uses[k];
    }
    scene.persist.saved_floor_states = save.floor_states.clone();
    // scriptLineFlags 读回（a.java:9285-9291：前 scriptLineCount 项 readBoolean，余
    // skip）——须先于 step 10 loadFloorData（m_122 floor2 恢复特例读 flags[26]）
    scene.script_line_flags = vec![false; 128];
    let line_count = scene.dialog.dialog_data.scripts.len().min(save.script_flags.len());
    scene.script_line_flags[..line_count].copy_from_slice(&save.script_flags[..line_count]);
    // step 10 = loadFloorData（应用本层触发缓存）+ buildMinimap（a.java:3515-3517）
    scene.load_floor_data(maplv, px, py, records, scene.persist.floor)?;
    scene.build_minimap();
    // step 11 相机（a.java:3523-3525）+ dispatchModeEntry case 3（a.java:4312-4318）
    scene.enter_view();
    scene.menu.softkeys = (1, 3);
    scene.dialog.overlay_active = false;
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
// 【task-15 V1】RestoreAction/RestoreRule/RESTORE_RULES → restore_rules.rs（move-only 汇 1）

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
// 【task-15 V1】ScriptOp enum → script_ops.rs（move-only 汇 1）

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
        self.dialog.overlay_active = true;
        self.dialog.overlay_kind = kind;
        self.dialog.overlay_text = Some(text.to_string());
        if matches!(kind, 1 | 3) {
            let t = self.menu.overlay_kind_item;
            self.menu.overlay_icon = self.images.entity.get(t as usize).cloned().flatten();
            self.menu.overlay_item_name = self.object_type_name(t);
        }
        let font = crate::paint::paint_font();
        let line_h = font.height() + 4;
        if matches!(kind, 0 | 2 | 4 | 5) {
            self.layout_full_text(text, OVERLAY_TEXT_W, OVERLAY_TEXT_MAX_H);
            self.dialog.overlay_box_w = self.dialog.overlay_content_w + 32;
            self.dialog.overlay_box_h = 32 + (self.dialog.overlay_page_bottom - self.dialog.overlay_page_top) * line_h;
        } else {
            self.layout_full_text(text, OVERLAY_TEXT_W, OVERLAY_ICON_TEXT_MAX_H);
            self.dialog.overlay_box_w = 212;
            self.dialog.overlay_box_h = 68 + (self.dialog.overlay_page_bottom - self.dialog.overlay_page_top) * line_h;
        }
        self.dialog.overlay_left_icon = left;
        self.dialog.overlay_right_icon = right;
        if left != 0 || right != 0 {
            self.dialog.overlay_box_h += 18;
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




    /// tryRunScene（a.java:8065-8139）：脚本行门控（scriptLineFlags + 场景参数
    /// script_cursor0=fromWalk）。case 40 落 5..40 直通组（table_count 原样 = !flags[40]）。
    pub fn try_run_scene(&mut self, idx: i32, from_walk: bool) -> bool {
        if idx >= self.dialog.dialog_data.scripts.len() as i32 {
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
            16 => gate &= self.menu.difficulty_index == 0,
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
            67 => gate &= self.menu.difficulty_index > 0,
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



    /// openDialogPage（a.java:8208-8227）：页装载 + 说话者/锚 + 预排版。
    /// 布局走 layout_wrapped_text_limited（w=129、高=2·lineH+12、limit=typewriterPos）。
    fn open_dialog_page(&mut self, page: i32) {
        if page <= self.dialog.dialog_page_end {
            self.dialog.dialog_page = page;
            self.dialog.dialog_speaker = if page < 0 {
                -1
            } else {
                self.dialog.dialog_data.speakers[page as usize] as i32
            };
            self.dialog.dialog_anchor = {
                // lockCameraOn(speaker)：相机锚初始化 + 返回目标实体
                self.lock_camera_on(self.dialog.dialog_speaker);
                if self.dialog.dialog_speaker != 0 && self.dialog.dialog_speaker != 87 {
                    let mut found = -1;
                    for e in (0..self.entities.count).rev() {
                        if self.entities.items[e].type_ == self.dialog.dialog_speaker {
                            found = e as i32;
                            break;
                        }
                    }
                    if found >= 0 {
                        self.ease_target_x = self.entities.items[found as usize].px;
                        self.ease_target_y = self.entities.items[found as usize].py;
                        // a.java:8667-8668（lockCameraOn 内）：型69 目标 X += 32
                        if self.dialog.dialog_speaker == 69 {
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
            let composed = if self.dialog.dialog_speaker < 0 {
                format!("?: \\cF8F8F8{}", self.dialog.dialog_data.texts[page as usize])
            } else {
                let name = &self.object_type_names[self.dialog.dialog_speaker as usize];
                format!("{}: \\cF8F8F8{}", name, self.dialog.dialog_data.texts[page as usize])
            };
            self.dialog.dialog_char = if self.dialog.dialog_speaker < 0 {
                11
            } else {
                (self.object_type_names[self.dialog.dialog_speaker as usize].chars().count() + 10) as i32
            };
            self.dialog.overlay_text = Some(composed.clone());
            let line_h = crate::paint::paint_font().height + 4;
            self.layout_wrapped_text_limited(&composed, 129, (line_h << 1) + 12, self.dialog.dialog_char);
        } else {
            self.dialog.dialog_phase = 0;
            self.dialog.dialog_char = 0;
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
        self.dialog.overlay_page_top = 0;
        self.dialog.overlay_total_lines = r.total_lines;
        self.dialog.overlay_lines_per_page = r.lines_per_page;
        self.dialog.overlay_page_bottom = r.total_lines.min(r.lines_per_page);
        self.dialog.overlay_content_w = r.content_w;
        self.dialog.overlay_lines = r.lines.iter().map(|l| String::from_utf16_lossy(l)).collect();
    }

    /// flipOverlayPage（a.java:5129-5140）：页顶/页底钳制。
    /// handleOverlayKey（a.java:4925-4934）：-2/-1 = flipOverlayPage(top∓**1**)、
    /// 0 空、其余键关层。kind1/3/4/兜底 的非专用键统一走此口。
    /// 【2026-10-16 修正】原 kind3/4/兜底 误用 ±lines_per_page——翻页距只有
    /// kind0(15/17) 是 ±lines、kind5 与本口是 ±1（BUG-002 活码核查连带发现）。
    fn handle_overlay_key(&mut self) {
        match self.key_value {
            -2 => self.flip_overlay_page(self.dialog.overlay_page_top + 1),
            -1 => self.flip_overlay_page(self.dialog.overlay_page_top - 1),
            0 => {}
            _ => self.dialog.overlay_active = false,
        }
    }

    fn flip_overlay_page(&mut self, page: i32) {
        let mut top = page;
        if top >= 0 {
            if top > self.dialog.overlay_total_lines - self.dialog.overlay_lines_per_page {
                top = self.dialog.overlay_total_lines - self.dialog.overlay_lines_per_page;
            }
        } else {
            top = 0;
        }
        self.dialog.overlay_page_top = top;
        self.dialog.overlay_page_bottom = top + self.dialog.overlay_lines_per_page;
        if self.dialog.overlay_page_top < 0 {
            self.dialog.overlay_page_top = 0;
        }
        if self.dialog.overlay_page_bottom > self.dialog.overlay_total_lines {
            self.dialog.overlay_page_bottom = self.dialog.overlay_total_lines;
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
        self.tap.tap_bump = None;
        self.tap.tap_pending = None;
        self.tap.tap_followup_ready = false;
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
        if self.path_arrows_visible && self.tap.walkto_arrows.is_none() {
            let n = (self.walk_step_count.max(0) as usize).min(self.walk_path_buffer.len());
            self.tap.walkto_arrows = Some(self.walk_path_buffer[..n].to_vec());
        }
        self.walk_step_count = steps.len() as i32;
        self.walk_path_buffer = steps;
        self.script_walk_armed = true;
        self.dialog.dialog_phase = 3;
        self.walk_phase = 0;
        // dp3 消费机器在 Dialog 语境（op_mov 同款——脚本行走=mode 11 语境）
        self.game_mode = GameMode::Dialog;
        self.lock_camera_on(0);
    }

    /// 【STAGE-B-008】场上是否接收点地（壳据此决定点按走 [`Self::tap_cell`] 还是行命中 / 点空白）：
    /// 无浮层、非遮幅，且为地图态或点地寻路自身驱动的 dp3。手动行走 / 战斗 / 观察模式也接收（由 tap_cell 闪红或忽略）。
    pub fn accepts_field_tap(&self) -> bool {
        if self.dialog.overlay_active || self.transitioning {
            return false;
        }
        match self.game_mode {
            GameMode::Map => true,
            GameMode::Dialog => self.dialog.dialog_phase == 3 && self.walkto_owns_dp3(),
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
            if self.tap.tap_target == Some((cx, cy)) && (cx, cy) != at {
                return if self.tap.tap_pending.is_some() {
                    TapResult::Queued
                } else if self.tap.tap_bump.is_some() {
                    TapResult::Bumping
                } else {
                    TapResult::Walking
                };
            }
            // 【spec §5.2 排队】点地寻路中（含末步在途窗口，见 tap_walk_in_progress）：打断（BUG-040 排空：走完当前步回 Map wp0；
            // 末步窗口里路段已收、interrupt_route 为空操作），取消旧撞击；点自己 = 停下，点别处 = 记下新目标（只留最后一次）、
            // 目标框立即换到新目标，落格后的后续拍执行（tick_tap_state）。
            self.tap.tap_bump = None;
            self.tap.tap_selected = None;
            self.interrupt_route();
            if (cx, cy) == at {
                self.tap.tap_pending = None;
                self.tap.tap_target = None;
                return TapResult::Stopped;
            }
            self.tap.tap_pending = Some(crate::tap_walk::PendingTap::Cell(cx, cy));
            self.tap.tap_target = Some((cx, cy));
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
        self.tap.tap_bump = None;
        self.tap.tap_pending = None;
        self.tap.tap_followup_ready = false;
        let grid = self.tap_grid();
        // 【STAGE-B-008 · spec §5.2 Q8】点中怪物：自动战斗关且不是已选中的那只 ⇒ 只选中（不动、不开路线、不置目标框）
        if let Some(crate::tap_walk::TapCell { kind: crate::tap_walk::TargetKind::Monster(e), .. }) = grid.get(cx, cy) {
            // 再点判据带有效性校验：实体表重排（跨层探针 / 换层重载）后、下一拍校验前的同帧窗口里，
            // 同下标可能已是另一只怪（计划 2 T6 复核 R2）
            let again = self.tap.tap_selected.map_or(false, |s| s.entity == e && self.selection_valid(s));
            if !self.tap.auto_battle && !again {
                let ent = &self.entities.items[e];
                self.tap.tap_selected = Some(crate::tap_walk::SelectedEnemy {
                    entity: e,
                    type_: ent.type_,
                    cell: (ent.px >> 5, ent.py >> 5),
                    floor: self.persist.current_floor,
                });
                self.tap.tap_target = None;
                return TapResult::Selected;
            }
        }
        // 点别处 / 点自己 / 再点同一只 / 自动战斗：清选中，再执行这次点按
        self.tap.tap_selected = None;
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
        (self.game_mode == GameMode::Dialog && self.dialog.dialog_phase == 3 && self.walkto_owns_dp3())
            || (self.game_mode == GameMode::Map && self.walk_phase == 1 && self.tap.tap_followup_ready)
    }

    fn start_tap_plan(&mut self, plan: crate::tap_walk::TapPlan, target: (i32, i32)) -> crate::tap_walk::TapResult {
        use crate::tap_walk::{TapPlan, TapResult};
        match plan {
            TapPlan::Walk { steps, .. } => {
                self.walkto_draining = false;
                self.begin_walk_route(steps);
                self.tap.tap_target = Some(target);
                TapResult::Walking
            }
            TapPlan::Bump { steps, dock, dir } => {
                self.tap.tap_target = Some(target);
                if steps.is_empty() {
                    self.execute_bump(dir);
                } else {
                    self.walkto_draining = false;
                    self.tap.tap_bump = Some((dock, dir));
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
        self.tap.tap_reject = Some((cell, self.frame_counter + crate::tap_walk::TAP_REJECT_TICKS));
    }

    /// 【STAGE-B-008】清点地状态（撤销 / 换层 / 剧情 / 按键时）。
    pub fn clear_tap_state(&mut self) {
        self.tap.tap_target = None;
        self.tap.tap_reject = None;
        self.tap.tap_bump = None;
        self.tap.tap_followup_ready = false;
        self.tap.tap_selected = None;
        self.tap.tap_pending = None;
    }

    /// 【STAGE-B-008 · spec §5.2】选中是否仍有效（每拍校验的判据）。
    fn selection_valid(&self, s: crate::tap_walk::SelectedEnemy) -> bool {
        if self.game_mode != GameMode::Map
            || self.walk_phase != 0
            || self.dialog.overlay_active
            || self.transitioning
            || self.persist.current_floor != s.floor
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
                    self.tap.tap_bump = None;
                    self.tap.tap_selected = None;
                    self.tap.tap_pending = None;
                    self.tap.tap_target = None;
                    self.interrupt_route();
                }
                true
            }
            Action::Cancel => {
                self.tap.tap_selected = None;
                true
            }
            Action::Fight => {
                let _ = self.engage_selected();
                true
            }
            Action::StopThen(c) => {
                if self.tap_walk_in_progress() {
                    self.tap.tap_bump = None;
                    self.tap.tap_selected = None;
                    self.tap.tap_target = None; // 【复核 T2-R5】打断 ⇒ 目标标记消失（spec §6）
                    self.interrupt_route();
                    self.tap.tap_pending = Some(crate::tap_walk::PendingTap::StopThen(c));
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
        match self.tap.tap_selected {
            Some(s) if self.selection_valid(s) => self.tap_cell(s.cell.0, s.cell.1),
            _ => {
                self.tap.tap_selected = None;
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
            self.tap.tap_bump = None;
            self.tap.tap_selected = None;
            self.interrupt_route();
            self.tap.tap_pending = Some(PendingTap::Swipe(dir));
            self.tap.tap_target = None;
            return TapResult::Queued;
        }
        match self.walk_phase {
            0 if !self.script_walk_armed => {}
            2 => return TapResult::Ignored,
            _ => return TapResult::Rejected,
        }
        self.tap.tap_bump = None;
        self.tap.tap_pending = None;
        self.tap.tap_selected = None;
        self.tap.tap_target = None;
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
        if let Some(s) = self.tap.tap_selected {
            if !self.selection_valid(s) {
                self.tap.tap_selected = None;
            }
        }
        if let Some((_, until)) = self.tap.tap_reject {
            if self.frame_counter >= until {
                self.tap.tap_reject = None;
            }
        }
        // 【spec §5.2 · 计划 2 T7 复核 R1】末步在途窗口（Map wp1 ∧ 后续就绪）里按住方向键：落格那一拍 handle_field_input 按 keyHeldCode
        // 同拍续走手动步，walk_phase 回 1，窗口判据分不清末步与手动步 ⇒ 手动输入优先，就绪位、撞击、排队一并作废
        //（浮层开着时按键属浮层，不判；与 press_key 同口径）。须在下列块之前：作废后本拍起即不再算"点地寻路中"。
        if self.tap.tap_followup_ready
            && self.game_mode == GameMode::Map
            && self.walk_phase == 1
            && !self.dialog.overlay_active
            && self.key_held != 0
        {
            self.tap.tap_followup_ready = false;
            self.tap.tap_bump = None;
            self.tap.tap_pending = None;
        }
        // 【spec §5.2 · 计划 2 T7】排队 / 撞击后续只在"点地寻路中"（寻路占 dp3，或末步在途窗口）或"收尾待执行"（含下面两种暂缓）
        // 期间有效：其余时刻的残留丢弃，免得之后无关的收尾误执行、目标框挂着不消失。
        // 【计划 2 T7 复核 R6】撞击后续在途中开战挂起（route.is_suspended）期间保留：胜利后路线续走到停靠格，照常撞击；
        // 排队仍按此规则（排队后排空只走完当前步，不会再撞出挂起）。挂起被壳层投键口径的 interrupt_route 解除（win route_key
        // 抬键）⇒ 不再保留，下一拍丢弃。
        if !self.tap.tap_followup_ready && !self.tap_walk_in_progress() {
            self.tap.tap_pending = None;
            if !self.route.is_suspended() {
                self.tap.tap_bump = None;
            }
        }
        if self.tap.tap_followup_ready
            && !(self.game_mode == GameMode::Map && self.walk_phase == 1)
            && !self.dialog.overlay_active
        {
            // 末步在途（Map wp1）时暂缓：多步路线的末步由 advanceScriptWalk 在倒数第二格步末弹出，
            // finish_walkto 随即回 Map、这一步仍在走（BUG-040 收尾口径）——等它落格后再判。
            // 【spec §5.1 · 计划 2 T5 复核 R2】浮层开着（停靠格本身弹拾取框）同样暂缓：保留就绪位与撞击 / 排队，
            // 关框后下一拍再按下列就绪条件判（关框键属浮层，press_key 不清撞击 / 排队）。
            // 排队优先于撞击（排队时已清撞击，这里只是兜底次序）。
            self.tap.tap_followup_ready = false;
            let bump = self.tap.tap_bump.take();
            let pending = self.tap.tap_pending.take();
            let ready = self.game_mode == GameMode::Map
                && self.walk_phase == 0
                && !self.dialog.overlay_active
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
        if self.tap.tap_target.is_some()
            && !(self.route.is_open()
                || self.walkto_draining
                || self.tap.tap_bump.is_some()
                || self.tap.tap_followup_ready
                || self.tap.tap_pending.is_some())
        {
            self.tap.tap_target = None;
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
        self.tap.tap_bump = None;
        self.tap.tap_pending = None;
        self.tap.tap_followup_ready = false;
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
            || self.dialog.overlay_active
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
        if self.dialog.overlay_active || self.transitioning || self.walk_phase != 0 {
            return false;
        }
        match self.game_mode {
            GameMode::Map => !self.script_walk_armed,
            GameMode::Dialog => self.dialog.dialog_phase == 3 && self.walkto_owns_dp3(),
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
        if self.game_mode == GameMode::Dialog && self.dialog.dialog_phase == 3 {
            self.walkto_draining = true;
        }
    }

    /// 【STAGE-B-008 · 复核 T4-R4】按键输入口的打断：浮层开着时零操作。原版浮层分支吞键、整拍返回
    ///（a.java:3252-3345），该键属浮层（关框 / 翻页），不是场上输入；BUG-040"手动优先"只对场上输入。
    /// ⇒ 点地路线途经拾取物弹框，关框后继续走向目标。换层 / 剧情接管等须无条件收段的调用方仍用
    /// [`Self::interrupt_route`]。
    pub fn interrupt_route_for_input(&mut self) {
        if self.dialog.overlay_active {
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
        self.tap.tap_followup_ready = true;
    }

    /// 【BUG-040】dp3 当前是否归点地寻路所有（段开放，或打断后仍在走完当前步）。
    pub(crate) fn walkto_owns_dp3(&self) -> bool {
        self.route.is_open() || self.walkto_draining
    }

    /// 路径箭头的步序源（buffer, 条数）：绘制（a.java:5929-5956）自玩家格沿 `buffer[..条数]` 自末端逐格外扩。
    /// 【BUG-043】点地寻路期间取暂存的箭头步序（walkPathBuffer 此时装的是寻路路线）。
    fn path_arrow_steps(&self) -> (&[u8], usize) {
        match &self.tap.walkto_arrows {
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
        let Some(steps) = self.tap.walkto_arrows.as_mut() else { return };
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
        if self.tap.walkto_arrows.is_some() && !self.walkto_owns_dp3() && !self.route.is_suspended() {
            self.restore_walkto_arrows();
        }
    }

    /// 【BUG-043】点地寻路收尾 / 被剧情接管：余下箭头步序放回 walkPathBuffer（与方向键走到此处时同态）。
    /// 箭头途中已熄（偏离 / 浮层撞格清除）⇒ 只丢弃暂存、不动 walkPathBuffer。
    fn restore_walkto_arrows(&mut self) {
        if let Some(steps) = self.tap.walkto_arrows.take() {
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
        self.tap.walkto_arrows = None; // 【BUG-043】新路径（SEE 箭头 / MOV）取代任何暂存的旧箭头
        result.found
    }

    /// skipTextToLineEnd（a.java:5135-5148）：打字机跳到当前行尾（累计长度）。
    fn skip_text_to_line_end(&self, pos: i32) -> i32 {
        let mut cum = 0i32;
        for line in &self.dialog.overlay_lines {
            cum += line.chars().count() as i32;
            if pos >= cum - line.chars().count() as i32 && pos < cum {
                return cum;
            }
        }
        cum
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
                self.dialog.dialog_phase = 0;
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
                    self.dialog.dialog_phase = 0;
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
        self.menu.menu_line_height = font.height() + 6;
        self.menu.menu_visible_rows = 238 / self.menu.menu_line_height;
        self.menu.menu_list_height = self.menu.menu_line_height * self.menu.menu_visible_rows;
        self.menu.menu_box_width = font.char_width('回' as u16) << 1; // wideCharWidth（全宽字符宽）
        self.menu.menu_item_count = 0;
        self.menu.menu_kinds.clear();
        self.menu.menu_scroll_top = 0;
    }

    /// addMenuItem（a.java:4704-4716）：kind 登记 + 框宽按 stringWidth+60 伸长。
    fn add_menu_item(&mut self, kind: i8) {
        if self.menu.menu_item_count < 16 {
            let font = crate::paint::paint_font();
            let label = MENU_LABELS[kind as usize];
            let w: i32 = label.chars().map(|c| font.char_width(c as u16)).sum();
            let w = w + 60;
            self.menu.menu_kinds.push(kind);
            if w > self.menu.menu_box_width {
                self.menu.menu_box_width = w;
            }
            self.menu.menu_item_count += 1;
            if self.menu.menu_item_count <= self.menu.menu_visible_rows {
                self.menu.menu_visible_bottom = self.menu.menu_item_count;
            }
            // menuTotalHeight += lineHeight；listHeight 按可见区间重算
            //（a.java:4712-4713——非 resetMenuLayout 的行数积）
            self.menu.menu_list_height =
                (self.menu.menu_visible_bottom - self.menu.menu_scroll_top) * self.menu.menu_line_height;
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
        let mut g = self.persist.global.clone();
        g.option_checked[0] = self.bgm.sound_enabled;
        g.option_checked[1] = self.minimap_enabled;
        g
    }

    /// 消费脏位。
    pub fn take_global_dirty(&mut self) -> bool {
        std::mem::take(&mut self.persist.global_dirty)
    }

    /// 装载/交接灌入（不置脏）。
    pub fn apply_global(&mut self, g: crate::save::GlobalSave) {
        self.persist.global = g;
        self.persist.global_dirty = false;
    }

    /// 水位抬升（a.java:7163-7164：`floor<51 && floor>水位`）。
    pub fn note_max_floor(&mut self, floor: i32) {
        if floor < 51 && floor > self.persist.global.max_floor_watermark {
            self.persist.global.max_floor_watermark = floor;
            self.persist.global_dirty = true;
        }
    }





    /// activateMenuItem（a.java:4762-4810）可达子集：1 继续游戏→mode3；
    /// 11 商店→mode19；17 跳过教程→changeFloor(1)+hp1000/atk710/def710→mode3。
    /// 3→mode7（存档槽）/4→mode16（设置）/5→mode15（帮助）/7→mode1（退出标题）
    /// 为跨机模式移交，桌面宿主批次接线——本批对拍面不激活。
    fn activate_menu_item(&mut self) {
        match self.menu.menu_kinds[self.menu.menu_cursor as usize] {
            1 => self.transition_to(GameMode::Map),
            3 => {
                // a.java:4776-4780：gameMode=7 + menuReturnMode=4 + dispatchModeEntry
                self.menu.menu_return_mode = GameMode::InGameMenu;
                self.transition_to(GameMode::SaveSlot);
                self.enter_menu7();
            }
            4 => {
                // a.java:4781-4785：gameMode=16 + menuReturnMode=4 + dispatchModeEntry
                self.menu.menu_return_mode = GameMode::InGameMenu;
                self.transition_to(GameMode::About);
                self.enter_menu16();
            }
            5 => {
                // a.java:4786-4790：gameMode=15 + menuReturnMode=4 + dispatchModeEntry
                self.menu.menu_return_mode = GameMode::InGameMenu;
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






    /// mode 15 帮助全文（dispatchModeEntry case 15 a.java:4407-4411，showOverlayMessage
    /// kind 0 逐字原文：99FFCC 色段 + 换行转义）。
    pub const HELP_TEXT: &str = "游戏描述：\n\\c99FFCC有人的地方就有江湖，有神仙的地方何尝不是江湖；百战百胜的本事，换不回女人的真心，兄弟的真义；齐天大圣又如何，没有真情实义，做神仙跟做咸鱼有什么区别？\n\n操作方式：按左软键调出物品栏，左右选择一件道具，按确定键使用。\n游戏操作：\n上方向键/2：向上行走\n下方向键/8：向下行走\n左方向键/4：向左行走\n右方向键/6：向右行走\n确定键/5:探索地图\n左软键：打开道具列表\n右软键：打开游戏中菜单\n\n代理发行：广州易诚计算机科技有限公司\n发行商网站：www.9266.net\n客服电话：4006509913\n客服信箱：kefu@9266.net";

    /// dispatchModeEntry case 15（a.java:4402-4413）：帮助全文浮层（kind 0）+ 软键 (0,3)。
    /// loadPackedImages(8)/loadPackedImages(1) = 懒加载资源容器（Rust 构造期全量装载，等价 no-op）。
    /// 迭代 26：改走 showOverlayMessage 完整口（原内联排版漏 overlayActive=true，
    /// 浮层整体不画——preset_help 嵌套假绿掩盖至今）。
    /// 【计划 3 T7 · spec §8】帮助文本覆盖（壳按模式注入触屏/摇杆操作说明；None=原文）。
    pub fn set_help_override(&mut self, text: Option<String>) {
        self.menu.help_override = text;
    }





    /// m_091（a.java:7954-8010）：敌人图鉴数据构建——活体敌人（cat8/type41+）
    /// 按类型去重 → 预测伤害（predictBattleHpLoss noCounter=true）→ 冒泡降序
    /// → 可见行数 258/50 钳条目数 → 面板高 50×行+32 → 滚动窗 [0,行)。
    fn build_encyclopedia(&mut self) {
        self.menu.encyc_damage = vec![0; 128];
        self.menu.encyc_types = vec![0; 128];
        self.menu.encyc_panel_w = ENCYC_PANEL_W;
        self.menu.encyc_count = 0;
        'outer: for e in 0..self.entities.count {
            if self.entities.items[e].removed {
                continue;
            }
            let t = self.entities.items[e].type_;
            if self.entity_category(t) != 8 || t < 41 {
                continue;
            }
            for k in 0..self.menu.encyc_count as usize {
                if self.menu.encyc_types[k] as i32 == t {
                    continue 'outer;
                }
            }
            let dmg = self.predict_hp_loss(t, true);
            let c = self.menu.encyc_count as usize;
            self.menu.encyc_damage[c] = dmg;
            self.menu.encyc_types[c] = t as u8;
            self.menu.encyc_count += 1;
        }
        // 冒泡降序（a.java:7986-7998）
        for a in 0..self.menu.encyc_count {
            let mut b = self.menu.encyc_count;
            while b > a + 1 {
                b -= 1;
                let dm = self.menu.encyc_damage[b as usize];
                let dp = self.menu.encyc_damage[b as usize - 1];
                if dm >= 0 && dm < dp {
                    self.menu.encyc_damage.swap(b as usize, b as usize - 1);
                    self.menu.encyc_types.swap(b as usize, b as usize - 1);
                }
            }
        }
        let font = crate::paint::paint_font();
        let row_h = font.height() + 4 + 36;
        self.menu.encyc_rows_visible = 258 / row_h;
        if self.menu.encyc_rows_visible > self.menu.encyc_count {
            self.menu.encyc_rows_visible = self.menu.encyc_count;
        }
        self.menu.encyc_panel_h = row_h * self.menu.encyc_rows_visible + 32;
        self.menu.encyc_scroll_top = 0;
        self.menu.encyc_scroll_bottom = self.menu.encyc_rows_visible;
    }

    /// m_092（a.java:8016-8027）：翻页。
    fn encyclopedia_set_page(&mut self, page: i32) {
        let max_page = (self.menu.encyc_count - self.menu.encyc_rows_visible).max(0);
        if page >= 0 && page <= max_page {
            self.menu.encyc_scroll_top = page;
            self.menu.encyc_scroll_bottom = self.menu.encyc_scroll_top + self.menu.encyc_rows_visible;
            if self.menu.encyc_scroll_bottom > self.menu.encyc_count {
                self.menu.encyc_scroll_bottom = self.menu.encyc_count;
            }
        }
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
        self.menu.shell_options = rows;
        if self.game_mode == GameMode::About {
            if let Some(old) = self.menu.options.take() {
                let anchor = old.highlight_anchor(); // 【件1】随行 id 跟随
                let mut list = crate::menu_family::OptionList::with_shell_rows(
                    self.bgm.sound_enabled,
                    self.minimap_enabled,
                    &self.menu.shell_options,
                );
                list.highlight = list.resolve_highlight(anchor) as i32;
                self.menu.options = Some(list);
            }
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
        if !self.dialog.overlay_active {
            self.tap.tap_bump = None;
            self.tap.tap_pending = None;
            self.tap.tap_followup_ready = false;
        }
        // 【STAGE-B-008 · spec §5.2】任何按键清怪物选中（浮层开着时选中本就由每拍校验清掉）
        self.tap.tap_selected = None;
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
        if self.dialog.overlay_active {
            if self.dialog.overlay_kind == 4 {
                // 商人/告示牌（overlay 分派 case4，deobf 3311-3320）：-6/-5=
                // applyMerchantOffer(true)，-7=(false)，其余 handleOverlayKey
                // （±**1** 翻页——【2026-10-16 修】原误用 ±lines_per_page）
                match self.key_value {
                    -7 => self.dialog.overlay_active = self.apply_merchant_offer(false),
                    -6 | -5 => self.dialog.overlay_active = self.apply_merchant_offer(true),
                    _ => self.handle_overlay_key(),
                }
            } else if self.dialog.overlay_kind == 5 {
                // mode12 日志浏览（overlay 分派 case5，a.java:3325-3344）：
                // -3/-4 = browseOfferQueue 翻页、-2/-1 = 长文滚动、-5/-6/-7 关回 mode10。
                // （迭代 59 纠偏：本臂曾被误挂商人报价 seek——商人 quote 实为
                // kind4（a.java:7442），merchant 场景亦无 -3/-4 按键。）
                match self.key_value {
                    -7 | -6 | -5 => {
                        self.dialog.overlay_active = false;
                        self.transition_to(GameMode::ItemPanel);
                    }
                    -4 => self.open_record_page(self.menu.record_page + 1),
                    -3 => self.open_record_page(self.menu.record_page - 1),
                    -2 => self.flip_overlay_page(self.dialog.overlay_page_top + 1),
                    -1 => self.flip_overlay_page(self.dialog.overlay_page_top - 1),
                    _ => {}
                }
            } else if self.dialog.overlay_kind == 0 && (self.game_mode == GameMode::Help || self.game_mode == GameMode::IconView) {
                match self.key_value {
                    -7 => {
                        self.dialog.overlay_active = false;
                        self.transition_to(self.menu.menu_return_mode);
                    }
                    -2 => {
                        self.flip_overlay_page(self.dialog.overlay_page_top + self.dialog.overlay_lines_per_page)
                    }
                    -1 => {
                        self.flip_overlay_page(self.dialog.overlay_page_top - self.dialog.overlay_lines_per_page)
                    }
                    _ => {}
                }
            } else if self.dialog.overlay_kind == 3 {
                // 道具确认浮层（overlay 分派 case3，a.java:3296-3306）：
                // -7 回面板、-6 关层 + useItemStack、其余 handleOverlayKey
                match self.key_value {
                    -7 => {
                        self.transition_to(GameMode::ItemPanel);
                        self.dialog.overlay_active = false;
                    }
                    -6 => {
                        self.dialog.overlay_active = false;
                        self.use_item_stack(self.menu.item_cursor as usize);
                    }
                    _ => self.handle_overlay_key(),
                }
            } else if self.dialog.overlay_kind == 1 {
                // case1 图标支路（javap run() 336-441；deobf ~3245-3290）：
                // 无图标 → handleOverlayKey；right 为 2/3 → key0 吞、-2/-1
                // handleOverlayKey、其余关；right 非 2/3 → 全吞键（441 →
                // goto 654）。【2026-10-16 补】原缺臂（并入兜底）。
                if self.dialog.overlay_right_icon == 0 && self.dialog.overlay_left_icon == 0 {
                    self.handle_overlay_key();
                } else if self.dialog.overlay_right_icon == 2 || self.dialog.overlay_right_icon == 3 {
                    match self.key_value {
                        0 => {}
                        -2 | -1 => self.handle_overlay_key(),
                        _ => self.dialog.overlay_active = false,
                    }
                }
            } else if self.dialog.overlay_kind == 2 {
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
                self.menu.in_game_menu_return_mode = self.game_mode;
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
        // f_bool_13 每拍重算（a.java:5765-5805 在 label216 段后；BUG-037 移植）
        self.recompute_stair_hint();
    }

    /// easeAfterimages（a.java:5209-5256）：残影蛇缓动。目标 = 未锁定时的
    /// 视口中心偏移（bob 耦合）或锁定后的玩家位；[3] 到位且锁定 → walkPhase=0。
    fn ease_afterimages(&mut self) {
        let (mut tx, mut ty) = if !self.view_locked {
            (
                ((self.view.view_w - 41) >> 1) - self.view.cam_x,
                (self.view.view_h >> 1) - self.view.cam_y + self.render.bob_offset,
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
            if self.menu.stair_hint {
                if let Some(e) = self.find_floor_gate_entity(true) {
                    let target = self.entities.items[e].param as i32 & 255;
                    self.change_floor(target, false, true);
                }
            }
        } else if self.key_value == 55 {
            // 55=下行（a.java:5300-5305 **分立 case**）：清零键值 → f_bool_13
            // 且存在型 8 门（param>>9==0）→ changeFloor(param&255, true, true)。
            self.key_value = 0;
            if self.menu.stair_hint {
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
        } else if self.dialog.overlay_active && self.walk_phase == 1 {
            // a.java:5353-5358：overlay 内撞格中断 → 路径与相位一并清
            self.path_arrows_visible = false;
            self.walk_step_count = 0;
            self.walk_phase = 0;
        }
        ok
    }

    /// 【BUG-037 · a.java:5765-5805】f_bool_13（stair_hint）每 tick 重算：先清零；
    /// tutorialSeen 亮且玩家四邻之一为楼梯锚点格（上/下梯各判同侧两向）⇒ 置 true。
    /// 此前生产从不置位（仅测试置位）⇒ 49/55 暗道换乘键与筋斗云钮生产不可达（移植缺口）。
    /// 本块纯加法：classic 教程前 stair_hint 恒 false 与 Java 一致，教程后向 Java 收敛。
    pub fn recompute_stair_hint(&mut self) {
        let mut hint = false;
        if self.menu.tutorial_seen {
            let (px, py) = (self.player_cell_x, self.player_cell_y);
            let anchors: Vec<(i32, i32)> = [self.entities.stair_up, self.entities.stair_down]
                .into_iter()
                .flatten()
                .map(|(x, y)| (x >> 5, y >> 5))
                .collect();
            for &(ax, ay) in &anchors {
                // a.java:5775 先清零（函数头）；四向判定（上/下/左/右各两梯）
                let up = py > 0 && ax == px && ay + 1 == py; // var6-1==var43 / var6+1==var43 族
                let down = py < self.view.high - 1 && ax == px && ay == py + 1;
                let left = px > 0 && ax + 1 == px && ay == py;
                let right = px < self.view.wide - 1 && ax == px + 1 && ay == py;
                if up || down || left || right {
                    hint = true;
                    break;
                }
            }
        }
        self.menu.stair_hint = hint;
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
                                        if let Some(a) = self.tap.walkto_arrows.as_mut() {
                                            a.clear();
                                        }
                                        self.menu.overlay_kind_item = 0;
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
                                    self.dialog.overlay_active = false;
                                    self.dialog.overlay_text = None;
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
                                    self.alchemy_gain_hp = Self::scaled_by_floor_tier(100, self.persist.floor);
                                    self.alchemy_gain_atk = Self::scaled_by_floor_tier(2, self.persist.floor);
                                    self.alchemy_gain_def = Self::scaled_by_floor_tier(4, self.persist.floor);
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
                                if self.persist.floor != 23 {
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
                                        self.tap.battle_undo_snap = Some((
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
                                    if let Some(a) = self.tap.walkto_arrows.as_mut() {
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
                                if self.persist.floor <= 50 {
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
                                            if self.persist.floor == 40 {
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
                            if self.persist.floor == 49
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
                            if self.persist.floor == 35 {
                                self.spawn_boss_event(7);
                                self.load_level_script(36);
                            }
                        }
                        70 | 71 => {
                            if self.persist.floor == 49 {
                                self.load_level_script(35);
                            }
                        }
                        72 => {
                            if self.persist.floor == 50 {
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
                            self.tap.battle_undo_snap.take()
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
                        self.tap.battle_undo_snap = None;
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
                                self.dialog.overlay_active = false;
                                self.dialog.overlay_text = None;
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
        if self.path_arrows_visible && self.tap.walkto_arrows.is_some() {
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


    /// pickupItemType（a.java:7525-7621，floor1 可达子集）+ applyHpDelta
    /// （a.java:5172-5184，difficultyIndex=0 → 无 ×16）。
    /// floorTier（a.java:7642-7652）：((floor-1)/10) 钳负、floor>50 归 0，返回 +1。
    pub fn floor_tier(&self) -> i32 {
        let mut tier = (self.persist.floor - 1) / 10;
        if tier < 0 {
            tier = 0;
        }
        if self.persist.floor > 50 {
            tier = 0;
        }
        tier + 1
    }

    fn pickup_item_type(&mut self, t: i32) {
        // a.java:7528 起 pickupItemType 拾取链：itemUseTargetType = 道具类型（描述 overlay 的图标/名）
        self.menu.overlay_kind_item = t;
        // 描述表（itemDescriptions a.java:906 / equipDescriptions a.java:930——
        // 全量入库 reference；此处取用句柄）
        match t {
            13..=25 | 85 | 86 => {
                if t == 15 {
                    self.menu.tutorial_seen = true; // tutorialSeen
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
                if self.persist.floor <= 10 { self.player.atk += 1 } else { self.player.atk += self.floor_tier() }
            }
            30 => {
                if self.persist.floor <= 10 { self.player.def += 1 } else { self.player.def += self.floor_tier() }
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
        self.menu.overlay_kind_item = t;
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
            .floor_category(t, self.persist.floor)
            .unwrap_or_else(|| entity::render_category(t))
    }

    /// 型 67 片件表（a.java:682-697 f_byte_arr2_00/01 逐值）已迁
    /// `sheet_index.assembly_profiles.seg67` 表中表（9 片件 xywh + 3×5 段帧）——
    /// 查表入口 `resource_cfg::ResourceCfg::seg67`；绘制臂消费见 paint_entities
    /// 型 67 分支（tests/sheet_index_table_driven.rs::task7_seg67_follows_table 对账）。








    /// browseOfferQueue（a.java:7480-7503）：日志页钳位翻转 + kind 选表 + 页码指示。
    /// kind=0 → f_String_arr_06，其余 → f_String_arr_09；总条数 0 →
    /// "没有记录" 回 mode10。
    fn open_record_page(&mut self, page: i32) {
        let total = self.persist.journal.len() as i32;
        if total > 0 {
            self.menu.record_page = if page < 0 {
                total - 1
            } else if page >= total {
                0
            } else {
                page
            };
            let (kind, idx) = self.persist.journal[self.menu.record_page as usize];
            let text = if kind == 0 {
                self.menu.record_text_a.get(idx as usize).cloned().unwrap_or_default()
            } else {
                self.menu.record_text_b.get(idx as usize).cloned().unwrap_or_default()
            };
            self.show_overlay_message(5, &text, 0, 3);
            self.menu.page_indicator = format!("{} / {}", self.menu.record_page + 1, total);
        } else {
            self.show_overlay_message(0, "没有记录", 0, 3);
            self.transition_to(GameMode::ItemPanel);
        }
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
            self.render.open_anim_tick += 1;
            if self.render.open_anim_tick == 18 {
                let group = self.open_group.clone();
                for e in group {
                    self.entities.items[e].anim_row = 6;
                    self.entities.items[e].state = 1;
                }
            } else if self.render.open_anim_tick >= 24 {
                let group = std::mem::take(&mut self.open_group);
                for e in group {
                    let gx = self.entities.items[e].px >> 5;
                    let gy = self.entities.items[e].py >> 5;
                    entity::remove_group_at_cell_by_type(gx, gy, 4, &mut self.entities, &mut self.grid);
                }
                self.render.open_anim_tick = 0;
                self.render.tile_bob = 0;
                self.walk_phase = 0;
            }
            // tileBob ±2 交替（a.java:7016-7014）
            self.render.tile_bob = if self.render.tile_bob == -2 { 2 } else { -2 };
            // a.java:7015 `script_cursor0 = 20 + tileBob`：**重置拍（cnt24）也取 bob**
            //（wp 已清 0 但 script_cursor0 在块内计算）——迭代 64 T930 一像素差实证
            return self.render.tile_bob;
        }
        0
    }


    /// typeFrameWidth[type]（type_width_px 夹具的 i32 视图）。
    fn type_frame_width_for(&self, t: i32) -> i32 {
        self.width_table.get(t as usize).copied().unwrap_or(32)
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
#[cfg(test)]
#[path = "scene_tests/slot_equip_scene_tests.rs"]
mod slot_equip_scene_tests;

/// 场景级单测的合成夹具（本文件 #[cfg(test)] 模块共用）：13×13 网格（走廊 y=ROW、
/// x∈1..=11 地形可走，其余墙）+ 纯色合成图集 + 真实数值表夹具。
#[doc(hidden)]
#[path = "scene_tests/scene_test_support.rs"]
pub mod scene_test_support;

/// BUG-017 回归（2026-10-17 用户实机："物品栏使用火眼金睛 → 游戏已终止"）：
/// 道具面板确认型 13 = Java 双跳 mode10→3→5（a.java:3939-3945 + 7786-7788）。
/// 曾因 TRANSFER_EDGES 缺 Map→EnemyInfo 边触发 census assert panic——
/// **该 panic 属 Rust 自造断言，非原版崩溃**（"原版等价终止"声称误报）。
#[cfg(test)]
#[path = "scene_tests/item13_use_regression.rs"]
mod item13_use_regression;

/// BUG-023 回归（diff-trace 既有红，2026-10-20 根因定位）：道具确认浮层（kind3）
/// `-7`"回面板"臂在 **ItemPanel 态**调 `transition_to(ItemPanel)`——Java 原版 =
/// `gameMode = 10` **自赋值**（deobf a.java:3297 `var15.gameMode = 10;`）。TRANSFER_EDGES
/// 缺 ItemPanel→ItemPanel 自环边 ⇒ census debug_assert 误杀（BUG-012 Dialog→Dialog、
/// BUG-017 Map→EnemyInfo 同类第三例）；debug 档 `cargo test -p game-oracle` 因此
/// preset_itemstack_frames_match panic（release 档 debug_assert 不编译故 rust-tests 不现）。
#[cfg(test)]
#[path = "scene_tests/itempanel_self_edge_regression.rs"]
mod itempanel_self_edge_regression;

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
        let pad = (self.menu.menu_line_height - font.height()) >> 1;
        let by = CENTER_Y - ((self.menu.menu_list_height + 32) >> 1) + 16 + pad
            + visible * self.menu.menu_line_height;
        (by - pad, self.menu.menu_line_height)
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
            (self.layout.screen_w - self.menu.item_box_w) / 2 + 20,
            (self.layout.screen_h - self.menu.item_box_h) / 2 + 31 + font.height(),
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
        if self.dialog.overlay_active {
            // 【STAGE-B-008 · spec §5.2】赠予 / 报价（kind 4、型 77 = 文本源 0、param 高字节 &2）无脚注图标，但 −5 = 接受（扣金 / 送物），
            // 是要做决定的框 ⇒ 点空白不代按（曾代按 −5）。判据与 apply_merchant_offer 同源（买成功 ^512 后的告示子框照旧代按）。
            // type_==77 为防御性判据（复核 T9-R1：生产态空间中型 78 两子类图标恒 (1,2)、本就 None）。
            if self.dialog.overlay_kind == 4 && self.merchant_entity >= 0 {
                let ent = &self.entities.items[self.merchant_entity as usize];
                if ent.type_ == 77 && ((ent.param as i32) >> 8) & 2 != 0 {
                    return None;
                }
            }
            let yes_no = self.dialog.overlay_left_icon == 1 && self.dialog.overlay_right_icon == 2;
            return if yes_no { None } else { Some(-5) };
        }
        match self.game_mode {
            GameMode::Dialog | GameMode::Ending => Some(-5),
            _ => None,
        }
    }

    pub fn pointer_hit(&self, x: i32, y: i32) -> PointerHit {
        // 【BUG-038】浮层在最上层：其下的行 / 格不接受命中（原版浮层分支先于各模式吞键）
        if self.dialog.overlay_active {
            return PointerHit::None;
        }
        match self.game_mode {
            GameMode::InGameMenu | GameMode::Shop => {
                let vis_n = self.menu.menu_visible_bottom - self.menu.menu_scroll_top;
                for vis in 0..vis_n.max(0) {
                    let (ry, rh) = self.menu_row_band(vis);
                    if y >= ry && y < ry + rh {
                        let index = self.menu.menu_scroll_top + vis;
                        if index >= 0 && index < self.menu.menu_item_count {
                            return PointerHit::Row { index, confirm_key: -6 };
                        }
                    }
                }
                PointerHit::None
            }
            GameMode::SaveSlot | GameMode::EquipDetail => {
                // 存档槽列表（SlotSelect，7/8 共享 dispatchModeEntry case7）：行带单源；
                // confirm=-5（run case 7：-6/-5 同义确认——mode7 存档/mode8 读档）
                for vis in 0..(self.menu.slot.scroll_bottom - self.menu.slot.scroll_top).max(0) {
                    let (ry, rh) = self.menu.slot.row_band(vis, &self.layout);
                    if y >= ry && y < ry + rh {
                        let index = self.menu.slot.scroll_top + vis;
                        return PointerHit::Row { index, confirm_key: -5 };
                    }
                }
                PointerHit::None
            }
            GameMode::About => {
                // 设置面板（OptionList）：行带单源 row_band；confirm=-5（run case 16
                // 翻转键，a.java:4143-4188）
                if let Some(list) = self.menu.options.as_ref() {
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
                    if c >= 0 && c < self.menu.item_cols && r >= 0
                        && r < self.menu.item_scroll_bottom - self.menu.item_scroll_top
                    {
                        let index = (self.menu.item_scroll_top + r) * self.menu.item_cols + c;
                        return PointerHit::Row { index, confirm_key: -6 };
                    }
                }
                PointerHit::None
            }
            _ => PointerHit::None,
        }
    }

    /// 【批三】道具使用/装备替换两段式：tap1=选中（返回 false 不执行；光标+预览
    /// 随 select_row 可见）；tap2 同行=执行（true）；点别行=换选不执行。
    /// 【双归域】两段式 tap 委托（TapState 域方法持断言；select_row 光标联动=
    /// 编排面 coordination 注记归因）。
    pub fn ui_tap_two_stage(&mut self, index: usize) -> bool {
        let frame = self.tap.ui_frame;
        let exec = self.tap.tap_two_stage(index, frame);
        if !exec {
            self.select_row(index as i32); // 编排：光标+滚窗联动（跨域）
        }
        exec
    }

    /// 【双归域】打断委托（TapState::interrupt 域方法）。
    pub fn ui_interrupt(&mut self) {
        self.tap.interrupt();
    }

    /// 指针选行（hover 高亮/点击对位）：只动光标+滚窗同步，无其他状态变化（③-2/b）。
    pub fn select_row(&mut self, index: i32) {
        match self.game_mode {
            GameMode::SaveSlot | GameMode::EquipDetail => {
                self.menu.slot.set_cursor(index); // 只动光标+滚窗
            }
            GameMode::InGameMenu | GameMode::Shop => {
                if index >= 0 && index < self.menu.menu_item_count {
                    self.menu.menu_cursor = index;
                    let n = self.menu.menu_visible_bottom - self.menu.menu_scroll_top;
                    if self.menu.menu_cursor >= self.menu.menu_visible_bottom {
                        self.menu.menu_visible_bottom = self.menu.menu_cursor + 1;
                        self.menu.menu_scroll_top = self.menu.menu_visible_bottom - n;
                    } else if self.menu.menu_cursor < self.menu.menu_scroll_top {
                        self.menu.menu_scroll_top = self.menu.menu_cursor;
                        self.menu.menu_visible_bottom = self.menu.menu_scroll_top + n;
                    }
                }
            }
            GameMode::About => {
                if let Some(list) = self.menu.options.as_mut() {
                    if index >= 0 && (index as usize) < list.len() {
                        list.set_highlight(index as usize); // 只动高亮（光标语义），零其他状态
                    }
                }
            }
            GameMode::ItemPanel => {
                let total = self.menu.item_cols * self.menu.item_rows_total;
                if index >= 0 && index < total {
                    self.menu.item_cursor = index;
                    let n = self.menu.item_scroll_bottom - self.menu.item_scroll_top;
                    let row = self.menu.item_cursor / self.menu.item_cols;
                    if row >= self.menu.item_scroll_bottom {
                        self.menu.item_scroll_bottom = row + 1;
                        self.menu.item_scroll_top = self.menu.item_scroll_bottom - n;
                    } else if row < self.menu.item_scroll_top {
                        self.menu.item_scroll_top = row;
                        self.menu.item_scroll_bottom = self.menu.item_scroll_top + n;
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
#[path = "scene_tests/pointer_seam_tests.rs"]
mod pointer_seam_tests;

/// 三处静默简化保真修复 + 读档旗标（completeness §3.4，2026-10-15）的单测
/// （⑤ 存档 blob 布局见 slot_equip_scene_tests::save_blob_layout_matches_java_and_roundtrips）：
/// ① addItemToItemStack 写 itemUseTargetType（a.java:7732）；② m_119 entityCount==0
/// 存 null（a.java:9383-9384）；③ m_122 floor2 恢复特例读 scriptLineFlags[26]
/// （a.java:9643-9646）；④ loadFloorSave 读回 scriptLineFlags 前 scriptLineCount 项
/// （a.java:9285-9291）。
#[cfg(test)]
#[path = "scene_tests/fidelity_fix_tests.rs"]
mod fidelity_fix_tests;

/// openWallOnRoute / relocateSlotsAtRouteEnd（dp2 实体路径缓动，a.java:8738-8904）与 MOV 五参臂（a.java:8274-8309）
/// 的逐拍单测：合成网格见 scene_test_support，直接驱动 execute_script_instruction /
/// tick_entity_route，逐拍断言 Java 语义表的每一格。场景级全帧对拍见
/// tests/mov_route_scene.rs（floor2 script1/2 + floor40 script12 伏击）。
#[cfg(test)]
#[path = "scene_tests/entity_route_tests.rs"]
mod entity_route_tests;

/// T2 WalkTo/打断 scene 级判据（touch-native-undo-plan §3.2/3.4 + task-11 打断终版）。
#[cfg(test)]
#[path = "scene_tests/t2_walkto_route_tests.rs"]
mod t2_walkto_route_tests;


/// BUG-040：点地寻路（WalkTo）途中任意键打断 ⇒ 打断后的 dp3 失去"寻路段开放"判据，步尽时
/// 落入脚本机 dp0 的"行尾收尾"（写 script_line_flags[当前脚本] = true + dp4 等相机才回 Map）
/// ——2026-10-28 对抗复核探针 P3 实测相位轨迹 Dialog/dp3 → dp0 → dp4 → Map/dp4。
#[cfg(test)]
#[path = "scene_tests/walkto_interrupt_tests.rs"]
mod walkto_interrupt_tests;

/// BUG-040 同根第二症状（2026-10-28 本测 RED 证实：寻路后武装残留=true，按一下方向键走了 3 格
/// 仍 walkPhase 1）：单步点地寻路（点相邻格）由 dp3 wp0 弹出唯一一步，不经 advanceScriptWalk
/// 清武装 ⇒ 回 Map 后 script_walk_armed 残留 ⇒ 下一次手动步进步末走 advanceScriptWalk 臂
/// （无步可弹、不归 walkPhase 0）⇒ 停不下来。
#[cfg(test)]
#[path = "scene_tests/walkto_single_step_tests.rs"]
mod walkto_single_step_tests;

/// BUG-038：触屏点按穿透到下层 / 在场上非空闲时执行场上动作（Stage B 触屏层，2026-10-28
/// 对抗复核运行时探针 P1a/P1b/P2 实测）。原版浮层打开期间场上不吃任何输入（a.java:3252-3345
/// 浮层分支吞键后整拍返回），行走中 / 战斗中也不读场上输入（walkPhase 分派 a.java:3563-3679）。
#[cfg(test)]
#[path = "scene_tests/touch_tap_guard_tests.rs"]
mod touch_tap_guard_tests;

/// BUG-039：点空白（未命中行 / 格）恒代按确定 -5（native.rs scene_tap 行外臂、runtime.rs step_boot、
/// win click_key 非 Map 态）⇒ 存档页点空白 = 覆盖当前槽、炼丹炉 = 花钱买、游戏内菜单 = 执行高亮项、
/// 设置页 = 切换高亮行、道具栏 = 选中高亮道具。只有"继续 / 关闭"类情境代按；原版以 ○/× 脚注
/// 标明的是否框（道具使用确认 kind 3、商人报价 kind 4）点空白不代按。
#[cfg(test)]
#[path = "scene_tests/blank_tap_tests.rs"]
mod blank_tap_tests;
