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
//!   `softkey_y()`），保证视口变化时推导关系不破；
//! - 与视口无关的纯数据（字符串、颜色码）保持 `const`。
//!
//! # 覆盖范围（对抗 review R-8）
//!
//! 本模块覆盖 **boot 渲染路径**（paint.rs / logo_anim.rs）+ **mode 3 游戏视图**
//! （scene.rs / game_view.rs，P3.5 第八批收编）。camera.rs 的 walk 边缘阈值
//! 106 等行为常量仍未收编（见下）。
//!
//! mode 3 常量分两类（扩视口时的处理方式不同）：
//! - **视口推导类**（写成 `fn`）：随 `SCREEN_*`/视图尺寸联动，扩视口时重推导；
//! - **设计/资源锁定类**（`const`）：格 32px、精灵帧尺寸、调色板等与视口
//!   无关（资产内尺寸），扩视口**不可**改动。

/// 设备视口宽（原版硬编码，MIDP 机型 240）。
pub const SCREEN_W: i32 = 240;
/// 设备视口高（原版硬编码，MIDP 机型 320）。
pub const SCREEN_H: i32 = 320;

// =========================================================================
// mode 3 游戏视图——几何（视口推导类）
// =========================================================================

/// 顶部状态栏高（ui[8][3] 条带 18px，paintStatusBar fillRect(65,0,175,18)）。
pub const STATUS_BAR_H: i32 = 18;

/// 游戏视图顶 y（paint case 3 的 `camY+20` 偏移 = 状态栏 18 + 2px 间隙）。
pub fn view_top_y() -> i32 {
    STATUS_BAR_H + 2
}

/// 游戏视图宽（viewWidthPx，loadFloorData；屏裁右界）。
pub const GAME_VIEW_W: i32 = 240;

/// 游戏视图高（viewHeightPx；与 camera.rs 的 252 同源——独立游戏常量
/// f_int_59，**非 SCREEN_H 派生**，扩视口需逐条考证）。
pub const GAME_VIEW_H: i32 = 252;

/// 视口底 y（f_int_48=270 = 状态栏 18 + 视图 252；HUD 大框顶）。
pub fn view_bottom_y() -> i32 {
    STATUS_BAR_H + GAME_VIEW_H
}

/// HUD 大框高（paintHudPanel 的 m_039(0, f_int_48, 240, 50)）。
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
/// 游戏视图基线 y = view_bottom - 320 + 6（f_int_48-314）。
pub fn parallax_game_base_y() -> i32 {
    view_bottom_y() - SCREEN_H + 6
}

// —— HUD（paintHudPanel/paintHudFrame/paintKeySlot）——
/// 属性图标列 x（HP/攻/防图标与条框的锚列）。
pub const HUD_ICON_X: i32 = 83;
/// 图标列 → 条框/槽的 x 间距（Java var3 += 16 / var11 += 16）。
pub const HUD_BAR_DX: i32 = 16;
/// 头像锚 x（paintHudPanel `40 - w>>1`）。
pub const HUD_FACE_AX: i32 = 40;
/// 首个属性行相对 HUD 基线的 y 偏移（Java var2 += 13）。
pub const HUD_STAT_Y0: i32 = 13;
/// 装备槽相对 HUD 基线的 y 偏移（Java var4 = var2+2，var2 已 +13）。
pub const HUD_SLOT_DY: i32 = 15;
/// 属性行距（HP→攻→防逐行 +12）。
pub const HUD_ROW_STEP: i32 = 12;
/// 属性条框尺寸（paintMiniFrame 58×11）。
pub const HUD_BAR_W: i32 = 58;
pub const HUD_BAR_H: i32 = 11;
/// 属性数值右锚 x 偏移（bar_x+52）。
pub const HUD_NUM_DX: i32 = 52;
/// 装备槽尺寸与步进（武器/甲 32×32，x 步 34）。
pub const HUD_SLOT_SIZE: i32 = 32;
pub const HUD_SLOT_STEP: i32 = 34;
/// HUD 大框边框件参数（ui[8][0]：角件 26×16、中段 16×16、侧条 11×16；
/// 侧条源偏移 x-42 / right-53 为 64px 边框条带内的取材位置）。
pub const HUD_BORDER_CORNER_W: i32 = 26;
pub const HUD_BORDER_MID_W: i32 = 16;
pub const HUD_BORDER_SIDE_W: i32 = 11;

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
/// 下/上/右/左；case 3 为镜像（drawEdgePatch transform 1）。
pub const PLAYER_ANCHORS: [(i32, i32, i32); 4] = [(-3, -18, 0), (-8, -14, 1), (-8, -16, 2), (0, -16, 3)];

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

/// mode 21 声音询问的提示文字（a.java:3052）的 UTF-16 码元序列。
/// 「是否开启声音？」= U+662F U+5426 U+5F00 U+542F U+58F0 U+97F3 U+FF1F。
pub const SOUND_PROMPT_TEXT: &[u16] = &[0x662F, 0x5426, 0x5F00, 0x542F, 0x58F0, 0x97F3, 0xFF1F];

/// mode 21 提示文字锚点 x（a.java:3052 `drawString(..., 120, 160, 17)`）。
/// 语义 = 水平居中锚：`SCREEN_W / 2`。
pub fn prompt_cx() -> i32 {
    SCREEN_W / 2
}

/// mode 21 提示文字锚点 y（a.java:3052）。语义 = 垂直居中：`SCREEN_H / 2`。
pub fn prompt_cy() -> i32 {
    SCREEN_H / 2
}

/// m_034 左软键底图 y（a.java:5985 `drawImage(f_Image_arr2_00[8][10], 0, 302, 0)`）。
/// 语义 = `SCREEN_H - 18`（302 = 320-18）。
pub fn softkey_base_y() -> i32 {
    SCREEN_H - 18
}

/// m_034 右软键底图 x（a.java:5990 `m_004(..., 222, 302, 1)`）。
/// 语义 = 右对齐 18px 边距：`SCREEN_W - 18`。
pub fn softkey_right_x() -> i32 {
    SCREEN_W - 18
}

/// m_034 软键图标 y（a.java:5986 `m_002(..., 2, 307, ...)`）。
pub fn softkey_icon_y() -> i32 {
    SCREEN_H - 13
}

/// m_034 左软键图标 x（a.java:5986）。
pub const SOFTKEY_ICON_LEFT_X: i32 = 2;

/// m_034 右软键图标 x（a.java:5991 `m_002(..., 226, 307, ...)`）。
/// 语义 = `SCREEN_W - 14`。
pub fn softkey_icon_right_x() -> i32 {
    SCREEN_W - 14
}

/// m_034 软键精灵条子图标尺寸 12×10（a.java:5986/5978 的 m_002 宽高参数）。
pub const SOFTKEY_ICON_W: i32 = 12;
pub const SOFTKEY_ICON_H: i32 = 10;

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
    }

    /// mode 3 派生几何互证（扩视口时此测试红 = 推导关系被破坏，须重考证）。
    #[test]
    fn game_view_derived_geometry_matches_java() {
        assert_eq!(view_top_y(), 20, "瓦片/实体层 y 偏移");
        assert_eq!(view_bottom_y(), 270, "f_int_48");
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
    }
}
