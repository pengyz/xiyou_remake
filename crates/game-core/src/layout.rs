//! 屏幕布局常量——**视口扩展的唯一修改点**。
//!
//! # 为什么集中于此（D2 扩展缝 + 视口扩大预案）
//!
//! 原版把 240×320 内联在几百个绘制调用里（A 级证据见各常量的 Java 行号）。
//! Stage A 像素一致要求这些数值**逐处与 Java 一致**，因此这里收编为具名常量、
//! 绘制代码禁止内联数字；Stage B 扩大视口时，把这些 `const` 升级为运行时
//! `LayoutConfig` 结构（或按分支参数化），消费方已全部经由此模块。
//!
//! # 分类纪律
//!
//! - `SCREEN_*`：设备视口（240×320），未来可变；
//! - 语义常量（软键栏 y、菜单柱 x 等）：**相对视口推导**的写成 `fn`（如
//!   `softkey_y()`）或派生 `const`（`CENTER_X = SCREEN_W / 2`、`GAME_VIEW_W =
//!   SCREEN_W`），保证视口变化时推导关系不破；
//! - **设计常量**（240×320 设计稿内的布局魔数，如浮层文本 180×240、图鉴面板宽
//!   208）：与 `SCREEN_*` 无维度推导关系，保留字面量但具名集中（文末专段）；
//! - 与视口无关的纯数据（字符串、颜色码）保持 `const`。
//!
//! # 覆盖范围（对抗 review R-8）
//!
//! 本模块覆盖 **boot 渲染路径**（paint.rs / logo_anim.rs）+ **mode 3 游戏视图**
//! （scene.rs / game_view.rs，P3.5 第八批收编）。camera.rs 的 walk 边缘阈值
//! 106 等行为常量仍未收编（见下）。
//!
//! 2026-10-15 卫生批：game-core 全部 240/320/120/160/208 字面量（143 行）逐处
//! 分类——屏幕派生类改写为 `SCREEN_*`/`CENTER_*` 公式；设计常量入文末专段；
//! 语义不明的 2 处（intro 快进哨兵 `scroll_x = 240`、popup 金币弹字 x=240）
//! 保留字面量并就地注明。
//!
//! mode 3 常量分两类（扩视口时的处理方式不同）：
//! - **视口推导类**（写成 `fn`）：随 `SCREEN_*`/视图尺寸联动，扩视口时重推导；
//! - **设计/资源锁定类**（`const`）：格 32px、精灵帧尺寸、调色板等与视口
//!   无关（资产内尺寸），扩视口**不可**改动。

/// 设备视口宽（原版硬编码，MIDP 机型 240）。
pub const SCREEN_W: i32 = 240;
/// 设备视口高（原版硬编码，MIDP 机型 320）。
pub const SCREEN_H: i32 = 320;

/// 屏幕中心 x（原版内联 `120`：HCENTER 锚点、`120 - (w >> 1)` 居中式，如
/// a.java:2433 菜单盒 / 2932 引子图）。派生 const：随 SCREEN_W 联动。
pub const CENTER_X: i32 = SCREEN_W / 2;
/// 屏幕中心 y（原版内联 `160`：`160 - (h >> 1)` 居中式，如 a.java:2434）。
pub const CENTER_Y: i32 = SCREEN_H / 2;

// =========================================================================
// mode 3 游戏视图——几何（视口推导类）
// =========================================================================

/// 顶部状态栏高（ui[8][3] 条带 18px，paintStatusBar fillRect(65,0,175,18)）。
pub const STATUS_BAR_H: i32 = 18;

/// 游戏视图顶 y（paint case 3 的 `camY+20` 偏移 = 状态栏 18 + 2px 间隙）。
pub fn view_top_y() -> i32 {
    STATUS_BAR_H + 2
}

/// 游戏视图宽（viewWidthPx，loadFloorData `= 240`；屏裁右界）。视图横跨全屏宽
///（HUD 大框 paintHudFrame(0, viewBottom, 240, 50)、状态栏 fillRect(65,0,175,18)
/// 右缘 240 同宽互证）⇒ SCREEN_W 派生。对照：GAME_VIEW_H 是独立常量（下）。
pub const GAME_VIEW_W: i32 = SCREEN_W;

/// 游戏视图高（viewHeightPx；与 camera.rs 的 252 同源——独立游戏常量
/// viewHeightPx，**非 SCREEN_H 派生**，扩视口需逐条考证）。
pub const GAME_VIEW_H: i32 = 252;

/// 视口底 y（viewBottom=270 = 状态栏 18 + 视图 252；HUD 大框顶）。
pub fn view_bottom_y() -> i32 {
    STATUS_BAR_H + GAME_VIEW_H
}

/// HUD 大框高（paintHudPanel 的 paintHudFrame(0, viewBottom, 240, 50)）。
pub const HUD_FRAME_H: i32 = 50;

/// HUD 大框顶 y = 视口底（SCREEN_H - 50 = 270，与 view_bottom_y 数值互证）。
pub fn hud_base_y() -> i32 {
    SCREEN_H - HUD_FRAME_H
}

// =========================================================================
// mode 3 游戏视图——几何（设计/资源锁定类，与视口无关）
// =========================================================================

/// 逻辑格边长（实体坐标对齐/格槽换算的单位，>>5）。
pub const CELL_PX: i32 = 32;

/// 瓦片绘制边长（paintTileLayer 的 16px 切片，<<4）。
pub const TILE_PX: i32 = 16;

/// 玩家精灵帧尺寸（actor 条带 3 行 41×46，paintPlayerSprite）。
pub const PLAYER_FRAME_W: i32 = 41;
pub const PLAYER_FRAME_H: i32 = 46;

/// 开格动画贴图尺寸（map[2][9]，entityState 1/2 的 27×29 帧片）。
pub const OPEN_ANIM_W: i32 = 27;
pub const OPEN_ANIM_H: i32 = 29;

/// 通用阴影偏移（sptprop[5][0] 椭圆，绘制于 (sx+8, sy+22)）。
pub const SHADOW_DX: i32 = 8;
pub const SHADOW_DY: i32 = 22;

/// 实体层屏裁 y 上界（paintEntityLayer：`sy >= -12`）。
pub const ENTITY_CULL_TOP: i32 = -12;

/// 楼梯浮标屏裁 x 余量（尾段 `x >= -32`）。
pub const MARKER_CULL_MARGIN: i32 = -32;

/// 玩家插入邻域（±32 的 x 窗口 / y 高差判定）。
pub const PLAYER_INSERT_WINDOW: i32 = 32;

// —— 视差背景（drawParallaxBackdrop）——
/// 列步进 77px（mapbg 瓦宽）。
pub const PARALLAX_COL_STEP: i32 = 77;
/// 滚动回绕：`--scroll < -154 → 0`（周期 155）。
pub const PARALLAX_WRAP: i32 = -154;
/// 游戏视图基线 y = view_bottom - 320 + 6（viewBottom-314）。
pub fn parallax_game_base_y() -> i32 {
    view_bottom_y() - SCREEN_H + 6
}

// —— HUD（paintHudPanel/paintHudFrame/paintKeySlot）——
/// 属性图标列 x（HP/攻/防图标与条框的锚列）。
pub const HUD_ICON_X: i32 = 83;
/// 图标列 → 条框/槽的 x 间距（Java table_count += 16 / var11 += 16）。
pub const HUD_BAR_DX: i32 = 16;
/// 头像锚 x（paintHudPanel `40 - w>>1`）。
pub const HUD_FACE_AX: i32 = 40;
/// 首个属性行相对 HUD 基线的 y 偏移（Java script_cursor0 += 13）。
pub const HUD_STAT_Y0: i32 = 13;
/// 装备槽相对 HUD 基线的 y 偏移（Java sort_i = script_cursor0+2，script_cursor0 已 +13）。
pub const HUD_SLOT_DY: i32 = 15;
/// 属性行距（HP→攻→防逐行 +12）。
pub const HUD_ROW_STEP: i32 = 12;
/// 属性条框尺寸（paintMiniFrame 58×11）。
pub const HUD_BAR_W: i32 = 58;
pub const HUD_BAR_H: i32 = 11;
/// 属性数值右锚 x 偏移（bar_x+52）。
pub const HUD_NUM_DX: i32 = 52;
/// HP 图标 10×10 裁剪及其 y 抬升；攻/防图标取 ui[8][7] 条带 (10,0)/(20,0) 10×13 片。
pub const HUD_ICON_CLIP: i32 = 10;
pub const HUD_ICON_LIFT: i32 = -2;
pub const HUD_ICON_STRIP_H: i32 = 13;
/// 装备槽尺寸与步进（武器/甲 32×32；首槽 bar_x+60，x 步 34）。
pub const HUD_SLOT_SIZE: i32 = 32;
pub const HUD_SLOT_DX: i32 = 60;
pub const HUD_SLOT_STEP: i32 = 34;
/// 盒框/HUD 框边框件参数（ui[8][0] 64×16 条带：角件 26、中段 16、侧条 11、
/// 条带高 16；左右侧条源取材偏移 -42 / -53——paintBoxFrame 与 paintHudFrame
/// 共用同一条带几何）。
pub const BORDER_CORNER_W: i32 = 26;
pub const BORDER_MID_W: i32 = 16;
pub const BORDER_SIDE_W: i32 = 11;
pub const BORDER_STRIP_H: i32 = 16;
pub const BORDER_SIDE_SRC_L: i32 = 42;
pub const BORDER_SIDE_SRC_R: i32 = 53;

// —— 实体层锚点（paintEntityLayer 各类别的"底对齐"边距）——
/// 阴影浮沉类的底边距（cat 2 小件/4/16）。
pub const ANCHOR_MARGIN_24: i32 = 24;
/// cat 2 高件的底边距。
pub const ANCHOR_MARGIN_30: i32 = 30;
/// cat 32 的底边距。
pub const ANCHOR_MARGIN_16: i32 = 16;
/// cat 8 直立怪的整体抬升（阴影后 var10 -= 4）。
pub const ENTITY_LIFT: i32 = 4;
/// 开格动画贴图的放置偏移与逐帧下移步（`sy+2-(frame<<3)`）。
pub const OPEN_ANIM_DX: i32 = 2;
pub const OPEN_ANIM_STEP: i32 = 8;
/// cat 2 的"小件"类型上界（<26 画阴影+浮沉，≥26 高件直贴）。
pub const SMALL_ITEM_MAX_TYPE: i32 = 26;

// —— 玩家精灵（paintPlayerSprite 各朝向的 (dx, dy, 行) 锚）——
/// 下/上/右/左；**左向（facing 3）是右向行（行 2）的镜像**（a.java:5925
/// drawEdgePatch(..., frame*41, 92, 41, 46, 1)——条带仅 3 行，无独立左行）。
pub const PLAYER_ANCHORS: [(i32, i32, i32); 4] = [(-3, -18, 0), (-8, -14, 1), (-8, -16, 2), (0, -16, 2)];

// —— 楼梯浮标（paintEntityLayer 尾段）——
/// 浮标相对楼梯格的 y 抬升（y-30+bob）。
pub const MARKER_DY: i32 = -30;
/// 上楼浮标的额外 x 偏移（x+5+bob）。
pub const MARKER_UP_DX: i32 = 5;

// —— 状态栏（paintStatusBar）——
/// 层数右锚 x（paintNumber ui[8][18]）。
pub const STATUS_FLOOR_NUM_X: i32 = 40;
/// 楼层图标 x。
pub const STATUS_TIER_ICON_X: i32 = 41;
/// 三钥格 x（黄/蓝/红，步 42）。
pub const STATUS_KEY_X: [i32; 3] = [65, 107, 149];
/// 金币图标 x 与数值右锚 x。
pub const STATUS_GOLD_ICON_X: i32 = 191;
pub const STATUS_GOLD_NUM_X: i32 = 237;
/// 钥匙格尺寸（ui[8][6] 18px 三联片）与数量偏移。
pub const KEY_SLOT_W: i32 = 18;
pub const KEY_SLOT_H: i32 = 16;
pub const KEY_NUM_DX: i32 = 40;
pub const KEY_NUM_DY: i32 = 5;

// —— 小地图（buildMinimap）——
/// 每格像素数（<<2 = 4px/格）。
pub const MINIMAP_PX_PER_CELL: i32 = 4;
/// 压暗 alpha（dimImage(img, 170)）。
pub const MINIMAP_DIM_ALPHA: i32 = 170;
/// 格标记尺寸（blit5Clip 5×5）。
pub const MINIMAP_MARKER_PX: i32 = 5;
/// 玩家点 4×4。
pub const MINIMAP_DOT_PX: i32 = 4;

// =========================================================================
// 调色板（与视口无关的纯数据；0xRRGGBB = Java setColor 十进制字面量的十六进制）
// =========================================================================

/// 小地图底色（Java 13097429）。
pub const MINIMAP_BG: u32 = 0xC7D9D5;
/// 小地图可行走格（Java 7509153）。
pub const MINIMAP_WALKABLE: u32 = 0x7294A1;
/// 小地图黄门标记/描边（Java 15461198 / 9794048）。
pub const MINIMAP_DOOR: u32 = 0xEBEB4E;
pub const MINIMAP_DOOR_EDGE: u32 = 0x957200;
/// 小地图红门标记/描边（Java 16273480 / 8388608）。
pub const MINIMAP_DOOR2: u32 = 0xF85048;
pub const MINIMAP_DOOR2_EDGE: u32 = 0x800000;
/// 小地图蓝门标记/描边（Java 4767984 / 549016）。
pub const MINIMAP_DOOR3: u32 = 0x48C0F0;
pub const MINIMAP_DOOR3_EDGE: u32 = 0x086098;
/// 小地图玩家点（Java 1112072）。
pub const MINIMAP_PLAYER_DOT: u32 = 0x10F808;

/// HUD 大框内衬（Java 2699825）。
pub const HUD_FILL: u32 = 0x293231;
/// 状态栏分隔底（Java 2435368）。
pub const STATUS_FILL: u32 = 0x252928;
/// miniFrame 填充/阴影边（Java 4803902 / 1645850，paintMiniFrame 固定色）。
pub const MINIFRAME_FILL: u32 = 0x494D3E;
pub const MINIFRAME_SHADE: u32 = 0x191D1A;
/// paintNumber 负数横线（Java 15027533）。
pub const NUMBER_NEGATIVE: u32 = 0xE54D4D;
/// Java 在攻/防行 paintMiniFrame 前的 setColor(512)——miniFrame 自用固定色，
/// 此设置无视觉效果，忠实保留。
pub const HUD_ROW_ACCENT: u32 = 0x000200;
/// 白（Java setColor(-1)）。
pub const WHITE: u32 = 0xFFFFFF;
/// 黑（Java setColor(0)）。
pub const BLACK: u32 = 0x000000;

// —— 通用 UI 底/文字色（跨模块复用；Java 十进制见互证测试）——
/// 深底（title 底条 / mode 2 加载屏 / 存档槽选中底，Java 3156024）。
pub const DARK_BACKDROP: u32 = 0x302838;
/// mode 8 顶部补条天蓝（Java 6662375）。
pub const TITLE_TOP_GAP: u32 = 0x65A8E7;
/// 存档槽未选中文字（Java 7574946）。
pub const SLOT_TEXT: u32 = 0x7395A2;
/// 存档槽选中文字亮黄（Java 16377897）。
pub const SLOT_TEXT_ACTIVE: u32 = 0xF9E829;
/// 存档列表分隔线暗/亮（Java 6178 / 3564144）。
pub const SLOT_RULE_DARK: u32 = 0x001822;
pub const SLOT_RULE_LIGHT: u32 = 0x366270;
/// 选项列表高亮条（Java 549016；与 MINIMAP_DOOR3_EDGE 同值不同用途）。
pub const OPTION_HIGHLIGHT: u32 = 0x086098;
/// 主题色解析失败默认（Java 2829099，parseHexColor）。
pub const PARSE_DEFAULT: u32 = 0x2B2B2B;
/// 引子字幕调色循环（Java introColors a.java:467：灰阶 5 色循环）。
pub const SUBTITLE_COLORS: [u32; 5] = [0xF8F8F8, 0xAAAAAA, 0x888888, 0x444444, 0x111111];

/// mode 21 声音询问的提示文字（a.java:3071）的 UTF-16 码元序列。
/// 「是否开启声音？」= U+662F U+5426 U+5F00 U+542F U+58F0 U+97F3 U+FF1F。
pub const SOUND_PROMPT_TEXT: &[u16] = &[0x662F, 0x5426, 0x5F00, 0x542F, 0x58F0, 0x97F3, 0xFF1F];

/// mode 21 提示文字锚点 x（a.java:3071 `drawString(..., 120, 160, 17)`）。
/// 语义 = 水平居中锚：`SCREEN_W / 2`。
pub fn prompt_cx() -> i32 {
    SCREEN_W / 2
}

/// mode 21 提示文字锚点 y（a.java:3071）。语义 = 垂直居中：`SCREEN_H / 2`。
pub fn prompt_cy() -> i32 {
    SCREEN_H / 2
}

/// paintSoftkeyBar 左软键底图 y（a.java:6008 `drawImage(f_Image_arr2_00[8][10], 0, 302, 0)`）。
/// 语义 = `SCREEN_H - 18`（302 = 320-18）。
pub fn softkey_base_y() -> i32 {
    SCREEN_H - 18
}

/// paintSoftkeyBar 右软键底图 x（a.java:6012 drawImageWithNokiaTransform(..., 222, 302, 1)）。
/// 语义 = 右对齐 18px 边距：`SCREEN_W - 18`。
pub fn softkey_right_x() -> i32 {
    SCREEN_W - 18
}

/// paintSoftkeyBar 软键图标 y（a.java:6009 drawImageClipped(..., 2, 307, ...)）。
pub fn softkey_icon_y() -> i32 {
    SCREEN_H - 13
}

/// paintSoftkeyBar 左软键图标 x（a.java:6009）。
pub const SOFTKEY_ICON_LEFT_X: i32 = 2;

/// paintSoftkeyBar 右软键图标 x（a.java:6013 drawImageClipped(..., 226, 307, ...)）。
/// 语义 = `SCREEN_W - 14`。
pub fn softkey_icon_right_x() -> i32 {
    SCREEN_W - 14
}

/// paintSoftkeyBar 软键图标尺寸 12×10（a.java:6009/6003 的 drawImageClipped 宽高）。
pub const SOFTKEY_ICON_W: i32 = 12;
pub const SOFTKEY_ICON_H: i32 = 10;

// =========================================================================
// 设计常量（240×320 设计稿；与 SCREEN_* **无**维度推导关系——扩视口时须逐条
// 重新考证，**不可**随 SCREEN_* 机械缩放）
// =========================================================================

/// 提示浮层文本排版宽（showOverlayMessage → layoutFullText(_, 180, _)，
/// a.java:4907/4913；paint 公共尾 paintWrappedText(_, _, _, 180, 240, true) a.java:3146）。
pub const OVERLAY_TEXT_W: i32 = 180;
/// 提示浮层文本页高上限（kind 0/2/4/5，a.java:4907/3146）。数值同 SCREEN_W
/// 但语义是**高度预算**（maxLines = 240/textLineHeight），非屏宽派生。
pub const OVERLAY_TEXT_MAX_H: i32 = 240;
/// 带图标浮层（kind 1/3）文本页高上限（a.java:4913）。
pub const OVERLAY_ICON_TEXT_MAX_H: i32 = 234;
/// 存档槽列表高度预算：slotsPerPage = 208 / textLineHeight（dispatchModeEntry case 8，a.java:4354）。
pub const SLOT_LIST_MAX_H: i32 = 208;
/// 图鉴面板宽 f_int_108（m_091 a.java:7959；paint case 5 居中于 a.java:2501）。
pub const ENCYC_PANEL_W: i32 = 208;
/// 左右翻页箭头对的 x（ui[8][14]；左 = 镜像 `60 - bob`、右 = `160 + bob`）：
/// mode 1 标题条（a.java:2371-2372）与 mode 12 日志浏览（a.java:2910/2914）同用。
pub const ARROW_PAIR_LEFT_X: i32 = 60;
pub const ARROW_PAIR_RIGHT_X: i32 = 160;

// =========================================================================
// STAGE-B-002：动态逻辑分辨率双预设——Layout 运行时形态（classic 恒等 / modern 派生）
// =========================================================================

/// 屏幕边缘跟随阈值 106（a.java:3562 面：下 `view_h-106` / 上 `106` / 右
/// `view_w-106` / 左 `106`）。与 `walk.rs:16 EDGE_MARGIN` 同值同义（双源登记，
/// walk.rs 收编待其写域开放）。
pub const EDGE_MARGIN: i32 = 106;

/// 相机偏移 clamp 上界 64（`game_view.rs` setCameraClamped `x > 64` / camera.rs 同源）。
pub const CAMERA_CLAMP_MAX: i32 = 64;

/// 布局预设。`Classic` = 原版 240×320（Legacy 等价命门：全部字段逐值 = 现行常量，
/// walk_camera_golden / game_view (8,-210) / ease 序列一字不改仍绿）；
/// `Modern` = 公式派生（逻辑分辨率 = 任意格数、UI 自适配、最小 = 原分辨率）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Classic,
    Modern,
}

/// 布局面（视口推导类 + 设计常量的运行时形态）。消费方经此取几何；
/// 常量面原样保留 = Legacy 锚（classic 下两者逐值互证，见 tests）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub preset: Preset,
    // —— 视口推导类 ——
    pub screen_w: i32,
    pub screen_h: i32,
    pub center_x: i32,
    pub center_y: i32,
    pub status_bar_h: i32,
    pub view_top_y: i32,
    pub game_view_w: i32,
    pub game_view_h: i32,
    pub view_bottom_y: i32,
    pub hud_frame_h: i32,
    pub hud_base_y: i32,
    pub prompt_cx: i32,
    pub prompt_cy: i32,
    pub softkey_base_y: i32,
    pub softkey_right_x: i32,
    pub softkey_icon_y: i32,
    pub softkey_icon_right_x: i32,
    // —— 相机参数（③：行为常量的 Layout 形态，classic 代入化回原值）——
    /// 屏幕边缘跟随阈值（a.java:3562 面；scene.rs 步进四处 / walk.rs EDGE_MARGIN 同值）。
    pub edge_margin: i32,
    /// setCameraClamped 偏移上界（game_view.rs `x > 64` / camera.rs 同源）。
    pub camera_clamp_max: i32,
    // —— UI 公式锚（②：顶栏楼层贴左/钥匙金币贴右/中间填充；HUD 贴底居中）——
    /// 三钥格 x（贴右：`screen_w - 175 + 42i`，右块 = 三钥槽 126 + 金币区 49）。
    pub status_key_x: [i32; 3],
    /// 金币图标/数值右锚 x（贴右：`screen_w - 49` / `screen_w - 3`）。
    pub status_gold_icon_x: i32,
    pub status_gold_num_x: i32,
    /// 顶栏中间填充左界 = 左块宽（楼层贴左，固定 65）。
    pub status_fill_x: i32,
    /// HUD 面孔锚 x / 属性图标列 x（贴底居中：`center_x - 80` / `center_x - 37`）。
    pub hud_face_ax: i32,
    pub hud_icon_x: i32,
    // —— 设计常量（文末专段的运行时形态；classic = 原值）——
    pub overlay_text_w: i32,
    pub overlay_text_max_h: i32,
    pub overlay_icon_text_max_h: i32,
    pub slot_list_max_h: i32,
    pub encyc_panel_w: i32,
    pub arrow_pair_left_x: i32,
    pub arrow_pair_right_x: i32,
}

impl Layout {
    /// classic 预设：逐值 = 现行常量面（Legacy 等价命门判据本体）。
    pub fn classic() -> Layout {
        Layout {
            preset: Preset::Classic,
            screen_w: SCREEN_W,
            screen_h: SCREEN_H,
            center_x: CENTER_X,
            center_y: CENTER_Y,
            status_bar_h: STATUS_BAR_H,
            view_top_y: view_top_y(),
            game_view_w: GAME_VIEW_W,
            game_view_h: GAME_VIEW_H,
            view_bottom_y: view_bottom_y(),
            hud_frame_h: HUD_FRAME_H,
            hud_base_y: hud_base_y(),
            prompt_cx: prompt_cx(),
            prompt_cy: prompt_cy(),
            softkey_base_y: softkey_base_y(),
            softkey_right_x: softkey_right_x(),
            softkey_icon_y: softkey_icon_y(),
            softkey_icon_right_x: softkey_icon_right_x(),
            edge_margin: EDGE_MARGIN,
            camera_clamp_max: CAMERA_CLAMP_MAX,
            status_key_x: STATUS_KEY_X,
            status_gold_icon_x: STATUS_GOLD_ICON_X,
            status_gold_num_x: STATUS_GOLD_NUM_X,
            status_fill_x: STATUS_KEY_X[0],
            hud_face_ax: HUD_FACE_AX,
            hud_icon_x: HUD_ICON_X,
            overlay_text_w: OVERLAY_TEXT_W,
            overlay_text_max_h: OVERLAY_TEXT_MAX_H,
            overlay_icon_text_max_h: OVERLAY_ICON_TEXT_MAX_H,
            slot_list_max_h: SLOT_LIST_MAX_H,
            encyc_panel_w: ENCYC_PANEL_W,
            arrow_pair_left_x: ARROW_PAIR_LEFT_X,
            arrow_pair_right_x: ARROW_PAIR_RIGHT_X,
        }
    }

    /// modern 预设：逻辑分辨率 = **任意格数**（`grid_w × grid_h` 格 × [`CELL_PX`]），
    /// 最小 = 原分辨率（240×320，即 classic）；视口推导类全字段公式自适配：
    /// - 视图宽 = 屏宽；视图高 = 屏高 − 状态栏 − HUD 框（classic 代入 320-18-50=252）；
    /// - 视口底 = 状态栏 + 视图高 = 屏高 − HUD 框（HUD 贴底，两式互证）；
    /// - 提示居中式 = 屏心；软键贴底/贴右 = `屏 − 边距`。
    ///
    /// 设计常量段（overlay/slot_list/encyc/arrow_pair）暂随 classic 值——
    /// modern 新值的逐条考证归 STAGE-B-002 ⑤（见登记册）。
    pub fn modern(grid_w: i32, grid_h: i32) -> Layout {
        let screen_w = (grid_w * CELL_PX).max(SCREEN_W);
        let screen_h = (grid_h * CELL_PX).max(SCREEN_H);
        let mut l = Layout::classic();
        l.preset = Preset::Modern;
        l.screen_w = screen_w;
        l.screen_h = screen_h;
        l.center_x = screen_w / 2;
        l.center_y = screen_h / 2;
        l.game_view_w = screen_w;
        l.game_view_h = screen_h - STATUS_BAR_H - HUD_FRAME_H;
        l.view_bottom_y = STATUS_BAR_H + l.game_view_h;
        l.hud_base_y = screen_h - HUD_FRAME_H;
        l.prompt_cx = screen_w / 2;
        l.prompt_cy = screen_h / 2;
        l.softkey_base_y = screen_h - 18;
        l.softkey_right_x = screen_w - 18;
        l.softkey_icon_y = screen_h - 13;
        l.softkey_icon_right_x = screen_w - 14;
        // ② UI 公式锚：楼层贴左（左块宽固定）、钥匙金币**贴右**（右块 175px =
        // 三钥槽 126 + 金币区 49，从右缘起）、HUD **贴底居中**（屏心 ± 偏置）
        l.status_key_x = [screen_w - 175, screen_w - 133, screen_w - 91];
        l.status_gold_icon_x = screen_w - 49;
        l.status_gold_num_x = screen_w - 3;
        l.hud_face_ax = screen_w / 2 - 80;
        l.hud_icon_x = screen_w / 2 - 37;
        l
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_prompt_text_is_utf16_exact() {
        // "是否开启声音？" = U+662F U+5426 U+5F00 U+542F U+58F0 U+97F3 U+FF1F
        let expect: [u16; 7] = [0x662F, 0x5426, 0x5F00, 0x542F, 0x58F0, 0x97F3, 0xFF1F];
        assert_eq!(SOUND_PROMPT_TEXT, &expect);
    }

    #[test]
    fn derived_layout_matches_java_literals() {
        assert_eq!(prompt_cx(), 120);
        assert_eq!(prompt_cy(), 160);
        assert_eq!(softkey_base_y(), 302);
        assert_eq!(softkey_right_x(), 222);
        assert_eq!(softkey_icon_y(), 307);
        // 屏幕派生（A 类）：240×320 设计稿下必须逐值等于 Java 内联字面量
        assert_eq!(CENTER_X, 120);
        assert_eq!(CENTER_Y, 160);
        assert_eq!(GAME_VIEW_W, 240, "viewWidthPx（a.java:9431）");
    }

    /// 设计常量（B 类）与 Java 字面量互证（防抄写错位）。
    #[test]
    fn design_constants_match_java_literals() {
        assert_eq!((OVERLAY_TEXT_W, OVERLAY_TEXT_MAX_H, OVERLAY_ICON_TEXT_MAX_H), (180, 240, 234));
        assert_eq!((SLOT_LIST_MAX_H, ENCYC_PANEL_W), (208, 208));
        assert_eq!((ARROW_PAIR_LEFT_X, ARROW_PAIR_RIGHT_X), (60, 160));
    }

    /// mode 3 派生几何互证（扩视口时此测试红 = 推导关系被破坏，须重考证）。
    #[test]
    fn game_view_derived_geometry_matches_java() {
        assert_eq!(view_top_y(), 20, "瓦片/实体层 y 偏移");
        assert_eq!(view_bottom_y(), 270, "viewBottom");
        assert_eq!(hud_base_y(), 270, "SCREEN_H-50 与 view_bottom 互证");
        assert_eq!(parallax_game_base_y(), -44, "T544 首列 y 实证");
    }

    /// 调色板十六进制 ↔ Java 十进制字面量互证（防抄写错位）。
    #[test]
    fn palette_hex_matches_java_literals() {
        assert_eq!(MINIMAP_BG, 13097429);
        assert_eq!(MINIMAP_WALKABLE, 7509153);
        assert_eq!(MINIMAP_DOOR, 15461198);
        assert_eq!(MINIMAP_DOOR_EDGE, 9794048);
        assert_eq!(MINIMAP_DOOR2, 16273480);
        assert_eq!(MINIMAP_DOOR2_EDGE, 8388608);
        assert_eq!(MINIMAP_DOOR3, 4767984);
        assert_eq!(MINIMAP_DOOR3_EDGE, 549016);
        assert_eq!(MINIMAP_PLAYER_DOT, 1112072);
        assert_eq!(HUD_FILL, 2699825);
        assert_eq!(STATUS_FILL, 2435368);
        assert_eq!(MINIFRAME_FILL, 4803902);
        assert_eq!(MINIFRAME_SHADE, 1645850);
        assert_eq!(NUMBER_NEGATIVE, 15027533);
        assert_eq!(HUD_ROW_ACCENT, 512);
        assert_eq!(BLACK, 0);
        assert_eq!(DARK_BACKDROP, 3156024);
        assert_eq!(TITLE_TOP_GAP, 6662375);
        assert_eq!(SLOT_TEXT, 7574946);
        assert_eq!(SLOT_TEXT_ACTIVE, 16377897);
        assert_eq!(SLOT_RULE_DARK, 6178);
        assert_eq!(SLOT_RULE_LIGHT, 3564144);
        assert_eq!(OPTION_HIGHLIGHT, 549016);
        assert_eq!(PARSE_DEFAULT, 2829099);
        assert_eq!(SUBTITLE_COLORS, [16316664, 11184810, 8947848, 4473924, 1118481]);
    }

    /// STAGE-B-002 **Legacy 等价命门（判据本体）**：classic 预设逐值 = 现行常量面
    /// （视口推导类 + 设计常量段），且 Java 字面量互证不变。本套件与
    /// walk_camera_golden 4 测 / game_view (8,-210) 钳制 / ease 收敛序列共同构成
    /// "一字不改仍绿"的 Legacy 判据面。
    #[test]
    fn layout_classic_equals_constant_surface() {
        let l = Layout::classic();
        assert_eq!(l.preset, Preset::Classic);
        // 视口推导类逐值恒等
        assert_eq!((l.screen_w, l.screen_h), (SCREEN_W, SCREEN_H));
        assert_eq!((l.center_x, l.center_y), (CENTER_X, CENTER_Y));
        assert_eq!((l.status_bar_h, l.view_top_y), (STATUS_BAR_H, view_top_y()));
        assert_eq!((l.game_view_w, l.game_view_h), (GAME_VIEW_W, GAME_VIEW_H));
        assert_eq!(
            (l.view_bottom_y, l.hud_frame_h, l.hud_base_y),
            (view_bottom_y(), HUD_FRAME_H, hud_base_y())
        );
        assert_eq!((l.prompt_cx, l.prompt_cy), (prompt_cx(), prompt_cy()));
        assert_eq!((l.softkey_base_y, l.softkey_right_x), (softkey_base_y(), softkey_right_x()));
        assert_eq!((l.softkey_icon_y, l.softkey_icon_right_x), (softkey_icon_y(), softkey_icon_right_x()));
        // 设计常量段（layout.rs 文末专段）逐值恒等（⑤）
        assert_eq!(
            (l.overlay_text_w, l.overlay_text_max_h, l.overlay_icon_text_max_h),
            (OVERLAY_TEXT_W, OVERLAY_TEXT_MAX_H, OVERLAY_ICON_TEXT_MAX_H)
        );
        assert_eq!((l.slot_list_max_h, l.encyc_panel_w), (SLOT_LIST_MAX_H, ENCYC_PANEL_W));
        assert_eq!((l.arrow_pair_left_x, l.arrow_pair_right_x), (ARROW_PAIR_LEFT_X, ARROW_PAIR_RIGHT_X));
        // Java 字面量互证（240×320 设计稿锚，防常量面自身漂移）
        assert_eq!((l.screen_w, l.screen_h), (240, 320));
        assert_eq!((l.view_top_y, l.game_view_h, l.view_bottom_y), (20, 252, 270));
        assert_eq!((l.overlay_text_w, l.overlay_text_max_h, l.slot_list_max_h), (180, 240, 208));
    }

    /// modern 预设公式（②UI 自适配的几何面）：
    /// - 逻辑分辨率 = 任意格数（grid×CELL_PX），**最小 = 原分辨率**（240×320）；
    /// - 视口推导类全字段随公式派生，且格数最小值时**逐值 = classic**（最小=原分辨率锚）；
    /// - 派生一致性不变量：view_bottom = status_bar_h + game_view_h = hud_base_y。
    #[test]
    fn layout_modern_formula_derivation() {
        // 最小格数（≤7×10）⇒ 钳到原分辨率 ⇒ 视口推导类逐值 = classic
        let min = Layout::modern(4, 4);
        let classic = Layout::classic();
        assert_eq!((min.screen_w, min.screen_h), (240, 320), "最小 = 原分辨率");
        assert_eq!(
            (
                min.center_x, min.center_y, min.view_top_y, min.game_view_w, min.game_view_h,
                min.view_bottom_y, min.hud_base_y, min.prompt_cx, min.prompt_cy
            ),
            (
                classic.center_x, classic.center_y, classic.view_top_y, classic.game_view_w,
                classic.game_view_h, classic.view_bottom_y, classic.hud_base_y, classic.prompt_cx,
                classic.prompt_cy
            ),
            "最小分辨率下视口推导类逐值 = classic"
        );
        // 任意格数（13×13 = 416×416）
        let big = Layout::modern(13, 13);
        assert_eq!((big.screen_w, big.screen_h), (416, 416), "格数×32");
        assert_eq!(big.game_view_w, 416);
        assert_eq!(big.game_view_h, 416 - big.status_bar_h - big.hud_frame_h);
        assert_eq!(big.view_bottom_y, big.status_bar_h + big.game_view_h);
        assert_eq!(big.hud_base_y, big.screen_h - big.hud_frame_h);
        assert_eq!(big.view_bottom_y, big.hud_base_y, "视口底 = HUD 基线（不变量）");
        assert_eq!((big.prompt_cx, big.prompt_cy), (208, 208), "居中式随屏派生");
        assert_eq!(big.softkey_base_y, 416 - 18, "软键贴底公式");
        assert_eq!(big.softkey_right_x, 416 - 18, "软键贴右公式");
        // 宽扁/高窄任意格数同样满足不变量（UI 自适配面）
        for (w, h) in [(20, 8), (8, 20), (7, 10), (9, 12)] {
            let l = Layout::modern(w, h);
            assert_eq!((l.screen_w, l.screen_h), ((w * 32).max(240), (h * 32).max(320)));
            assert_eq!(l.view_bottom_y, l.status_bar_h + l.game_view_h);
            assert_eq!(l.view_bottom_y, l.hud_base_y);
            assert_eq!(l.game_view_w, l.screen_w);
        }
    }

    /// ③ 相机参数入 Layout（classic 下代入**化回原值断言**）：边缘跟随阈值 106
    /// （a.java:3562 面：scene.rs 步进四处内联 + walk.rs EDGE_MARGIN 同值）与
    /// 相机 clamp 64（setCameraClamped 上界）。两参数=行为常量（无维度推导关系，
    /// 与 240×320 设计稿无关）⇒ modern 同值（考证结论：阈值语义是玩家跟随
    /// 舒适带宽，不随分辨率缩放）。
    #[test]
    fn camera_params_classic_substitution_is_identity() {
        let l = Layout::classic();
        assert_eq!((l.edge_margin, l.camera_clamp_max), (106, 64), "classic 化回原值");
        assert_eq!(Layout::modern(13, 13).edge_margin, 106, "行为常量两预设同值");
        assert_eq!(Layout::modern(13, 13).camera_clamp_max, 64);
    }

    /// ② UI 公式锚：顶栏**楼层贴左 / 钥匙金币贴右 / 中间填充**、HUD **贴底居中**。
    /// classic 代入化回原值（240 下 [65,107,149]/191/237/65、40/83）；modern 右钉/中钉
    /// 公式（右块 = 三钥槽 + 金币区 = 175px 从右缘起；HUD 面孔/图标列 = 屏心 ± 偏置）。
    #[test]
    fn ui_anchor_formulas_classic_identity_and_modern_pin() {
        let l = Layout::classic();
        assert_eq!(l.status_key_x, [65, 107, 149], "classic 化回原值");
        assert_eq!((l.status_gold_icon_x, l.status_gold_num_x), (191, 237));
        assert_eq!(l.status_fill_x, 65, "中间填充左界 = 左块宽");
        assert_eq!((l.hud_face_ax, l.hud_icon_x), (40, 83));
        // modern：贴右/贴中公式（416 宽：右块 175 起 = 241，金币 367/413）
        let big = Layout::modern(13, 13);
        assert_eq!(big.status_key_x, [241, 283, 325], "钥匙贴右 = 屏宽-175+42i");
        assert_eq!(
            (big.status_gold_icon_x, big.status_gold_num_x),
            (367, 413),
            "金币贴右 = 屏宽-49 / 屏宽-3"
        );
        assert_eq!(big.status_fill_x, 65, "左块宽固定（楼层贴左）");
        assert_eq!((big.hud_face_ax, big.hud_icon_x), (128, 171), "HUD 中钉 = 屏心-80 / 屏心-37");
    }
}
