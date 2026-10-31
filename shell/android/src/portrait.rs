//! 竖屏动态分辨率——**全部几何由公式推导**（2026-10-26）。
//!
//! > 【2026-10-26 更正】同日首版用设备特化常量拼布局：`MIN_SCENE_COLS=12`（恰是
//! > 1200px÷96 的 Xiaomi 14 解）、`BOOT_MIN_COLS=15`、`PANEL_MIN_PCT=30`、面板单位
//! > `u=min(宽/12, 高/7)`、手势条 `80px`、字高 `14`、标签倍率 `/3`、描边 `4/5/6px`——
//! > 换分辨率/密度即失准。用户质询："你这个魔数是哪来的？…手机分辨率变了，你这些数字
//! > 还准确吗？我们需要计算公式"。本版：**设备输入 + 游戏数据 + 平台规范常量 → 公式**，
//! > 无设备特化数字；剩余设计取值全为 dp 令牌或无量纲比例（与分辨率无关），集中列于下表。
//!
//! # 画面组成（窗口物理像素）
//!
//! ```text
//! ┌──────────────────────┐ ← 内容区顶（inset_top）
//! │  游戏视口（两档画布外接）│  开场链 = 原分辨率 240×320 × s_b；游戏 = Modern 格阵 × s
//! ├──────────────────────┤
//! │ [左软]          [右软] │  控制面板 = 视口下方全部剩余
//! │  (十字键)     (确定)   │
//! │ ─ ─ 系统手势区 ─ ─ ─ ─ │  G：控件不入
//! └──────────────────────┘
//! ```
//!
//! # 输入（全部来自设备或游戏数据）
//!
//! | 符号 | 含义 | 来源 |
//! |---|---|---|
//! | W×H | 窗口物理像素 | `ANativeWindow_getWidth/Height` |
//! | dpi | 屏幕密度 | `AConfiguration_getDensity`（未知码 ⇒ 160，见 [`dpi_or_baseline`]） |
//! | inset_top/bottom | 内容区相对窗口的上/下内缩 | glue `contentRect`（onContentRectChanged） |
//! | map | 全关卡最大格数（现数据 13×13） | 全部 `maplv{n}` 头 `>>1`（[`MapExtent::from_assets`]） |
//! | 240×320、32、68 | 原分辨率 / 格边 / 状态栏+HUD 框 | `layout::{SCREEN_W,SCREEN_H,CELL_PX,STATUS_BAR_H+HUD_FRAME_H}` |
//! | 14 | 正文字高 | `paint::paint_font().height`（SMALL） |
//! | 12×10 | 软键图标 | `layout::{SOFTKEY_ICON_W,SOFTKEY_ICON_H}` |
//!
//! # 平台规范常量（1dp = dpi/160 px；sp 按系统字体缩放 1.0 计）
//!
//! | 常量 | 值 | 出处 |
//! |---|---|---|
//! | [`TOUCH_TARGET_DP`] T | 48dp | Android 无障碍 / Material：触控目标 ≥ 48×48dp |
//! | [`MARGIN_DP`] M | 16dp | Material 紧凑屏版心边距 |
//! | [`GESTURE_AREA_DP`] | 32dp | AOSP 手势导航 overlay `navigation_bar_gesture_height` |
//! | [`MIN_TEXT_SP`] | 12sp | Material 3 字阶 Body Small（正文最小字阶） |
//! | [`LABEL_SP`] | 14sp | Material 3 字阶 Label Large（按钮标签） |
//! | [`BUTTON_MIN_W_DP`] | 64dp | Material 按钮最小宽 |
//! | [`ICON_OF_TARGET`] | 1/2 | Material 图标按钮：24dp 图标 / 48dp 目标 |
//!
//! # 设计取值（dp 令牌 / 无量纲比例——换机不失准；调整即改此处）
//!
//! | 常量 | 值 | 理由 |
//! |---|---|---|
//! | [`DPAD_CELLS`] | 3 | 十字键 = 3×3 格：中格=死区、四臂各一格 ⇒ 格 ≥ T 即每向 ≥ 48dp |
//! | [`OK_CELLS`] | 2 | 确定钮直径 = 2 格（主操作键大于单臂；同格制） |
//! | [`DPAD_MAX_DP`] | 192dp | = 4T ≈ 30mm：单拇指不挪掌可覆盖；平板不随面板无限放大 |
//! | [`OUTLINE_DP`] | 2dp | 描边粗（dp 定义 ⇒ 各密度物理粗细一致） |
//! | [`ARROW_FRAC`] | 1/4 | 箭头占臂格中央一半：顶点/底边距格心 ±c/4、半底宽 c/4 |
//!
//! # 公式
//!
//! ```text
//! dp = dpi/160；T = 48dp；M = 16dp；G = max(inset_bottom, 32dp)
//! 面板下限   P = T(软键行) + 3T(十字键) + 3M(三道边距) + G
//! 可用视口高 V = H − inset_top − P
//! 开场链     s_b = max(1, min(⌊W/240⌋, ⌊V/320⌋))；画布 = 240×320（原分辨率：classic 坐标全部就位）
//! 游戏       s_read = ⌈12sp·dp / 字高⌉（正文可读下限）
//!            满图格 C = map_w/32，R = ⌈(map_h + 68)/32⌉（再大只是空背景）
//!            s_whole = min(⌊W/32C⌋, ⌊V/32R⌋)（看全层的最大倍率）
//!            s = clamp(max(s_read, s_whole), 1, s_b)
//!            列 = min(⌊W/32s⌋, C)，行 = min(⌊V/32s⌋, R)（Layout::modern 钳 ≥ 240×320）
//! 视口       = 两档物理画布外接（横居中、顶贴内容区）；面板 = 视口下方全部剩余
//! 面板       软键：[M, M+T) 行，宽 max(64dp, 图标宽 + 2M)，图标倍率 ⌊(T/2)/10⌋
//!            簇区：[2M+T, 面板高 − G − M)；D = min(簇区高, (W − 3M)/2, 192dp)；c = D/3
//!            十字键心 (M + D/2, 簇区中线)；确定钮心 = 右侧镜像位，半径 c
//!            触控：死区 = c/2（中格内切圆），捕获 2·r_switch = D/2（盘面，可见即可触）
//! ```
//!
//! 游戏倍率取"可读下限·最大视野"：魔塔类玩法看全层最有价值；s 越大像素越大、视野越小。
//! 另一端是"原版视野·最大倍率"（s = s_b，Xiaomi 14 上 = 5×、240×320、格 = 53dp ≥ 触控目标）。
//!
//! # Xiaomi 14 算例（1200×2536，480dpi；tests/portrait.rs 逐步钉）
//!
//! dp=3、T=144、M=48、G=96 ⇒ P=816、V=1720；s_b=min(5,5)=5 ⇒ 开场链 1200×1600；
//! s_read=⌈36/14⌉=3、s_whole=min(2,3)=2 ⇒ s=3、12×16 格 ⇒ 1152×1536；面板 936；
//! D=min(552, 528, 576)=528、c=176（=59dp ≥ 48dp）。

use crate::input::{DpadZone, PadLayout, SoftZone, KEY_FIRE, KEY_LEFT, KEY_LEFT_SOFT, KEY_RIGHT, KEY_RIGHT_SOFT};
use game_core::layout::{
    Layout, CELL_PX, HUD_FRAME_H, SCREEN_H, SCREEN_W, SOFTKEY_ICON_H, SOFTKEY_ICON_W, STATUS_BAR_H,
};
use game_core::paint::paint_font;
use game_core::render::{ArgbImage, SoftGraphics};

/// dp 定义基准：1dp = dpi/160 px（Android `DisplayMetrics.DENSITY_DEFAULT`）。
pub const DP_BASELINE_DPI: i32 = 160;
/// 触控目标下限 48dp（Android 无障碍指南 / Material）。
pub const TOUCH_TARGET_DP: f32 = 48.0;
/// 屏缘 / 控件间距 16dp（Material 紧凑屏版心边距）。
pub const MARGIN_DP: f32 = 16.0;
/// 底部系统手势区 32dp（AOSP 手势导航 overlay `navigation_bar_gesture_height`）；
/// contentRect 实测内缩更大时取实测。
pub const GESTURE_AREA_DP: f32 = 32.0;
/// 正文最小字号 12sp（Material 3 字阶 Body Small）——定游戏倍率下限。
pub const MIN_TEXT_SP: f32 = 12.0;
/// 按钮标签字号 14sp（Material 3 字阶 Label Large）。
pub const LABEL_SP: f32 = 14.0;
/// 按钮最小宽 64dp（Material 按钮规格）。
pub const BUTTON_MIN_W_DP: f32 = 64.0;
/// 图标高 / 触控目标高 = 1/2（Material 图标按钮：24dp 图标 / 48dp 目标）。
pub const ICON_OF_TARGET: f32 = 0.5;
/// 十字键格数（3×3：中格=死区、四臂各一格）。
pub const DPAD_CELLS: f32 = 3.0;
/// 确定钮直径（格）。
pub const OK_CELLS: f32 = 2.0;
/// 十字键直径上限 192dp（= 4T ≈ 30mm，单拇指不挪掌可覆盖）。
pub const DPAD_MAX_DP: f32 = 192.0;
/// 【计划 3 T5 · spec §4.1】触屏工具栏高（64dp）。
pub const TOOLBAR_H_DP: f32 = 64.0;
/// 描边粗 2dp。
pub const OUTLINE_DP: f32 = 2.0;
/// 箭头占臂格比例（顶点/底边距格心 ±c·ARROW_FRAC、半底宽 c·ARROW_FRAC）。
pub const ARROW_FRAC: f32 = 0.25;

/// 画布内软键栏（**core 参数 `Layout::softkey_bar_visible`，由本壳配置**）：竖屏组成的控制面板
/// 已画同款软键图标（ui[11] ×icon_scale），画布内 HUD/菜单左右下角的小图标与之重复 ⇒ 关。
/// core 默认开（横屏旧路径 / PC 壳不配置 = 原版照画）。
pub const CANVAS_SOFTKEY_BAR: bool = false;

/// 画布内标题底栏 UI 倍率（壳层配置 core `Layout::title_ui_k`）：竖屏开场链 = 原分辨率 240×320 ×
/// 最大整数倍（Xiaomi 14 = ×5）⇒ 原尺寸「开始游戏」标签物理高 17×5 = 85 px，已大于横屏 ×2 补偿
/// （18×18 画布物理 ×2：17×2×2 = 68 px）⇒ 不再放大。core 默认 ×2（横屏旧路径 / PC modern 不变）。
/// 【2026-10-27 用户："横屏时为解决'开始游戏'太小做了放大，现在改成竖屏了，不需要放大了"】
pub const CANVAS_TITLE_UI_K: i32 = 1;

/// AConfiguration 密度码 → dpi：DEFAULT=0 / ANY=0xfffe / NONE=0xffff（及越界）⇒ 160
/// （dp 基准：1dp = 1px，最保守——不放大控件也不缩小）。
pub fn dpi_or_baseline(raw: i32) -> i32 {
    match raw {
        1..=0xfffd => raw,
        _ => DP_BASELINE_DPI,
    }
}

fn ceil_div(a: i32, b: i32) -> i32 {
    (a + b - 1) / b.max(1)
}

/// 全关卡最大格数（32px 格）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct MapExtent {
    pub cells_w: i32,
    pub cells_h: i32,
}

impl MapExtent {
    /// 逐个读 `maplv0, maplv1, …`（首个缺失即止），取头宽/高（`>>1`，32px 格——
    /// game_data::maplv 与 game_view::from_maplv 同式）的最大值。解析失败的关卡跳过。
    pub fn from_assets(src: &dyn game_platform::AssetSource) -> MapExtent {
        let mut e = MapExtent::default();
        for n in 0.. {
            let Ok(bytes) = src.read(&format!("maplv{n}")) else { break };
            if let Ok((m, _)) = game_data::maplv::Maplv::parse_partial(&bytes) {
                e.cells_w = e.cells_w.max(m.width as i32);
                e.cells_h = e.cells_h.max(m.height as i32);
            }
        }
        e
    }
}

/// 规划输入（全部来自设备与数据）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanInput {
    /// 窗口物理像素 (W, H)。
    pub window: (i32, i32),
    /// 屏幕密度（已经 [`dpi_or_baseline`] 规整）。
    pub dpi: i32,
    /// 内容区上 / 下内缩（px；未知 = 0）。
    pub inset_top: i32,
    /// 【计划 3 T5】操控方式（默认摇杆；触屏=工具栏档。setting 里缺省构造处须补 Pad）。
    pub control_mode: game_shell_common::settings::ControlMode,
    pub inset_bottom: i32,
    /// 全关卡最大格数。
    pub map: MapExtent,
}

/// 矩形（物理像素）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x as f32 && y >= self.y as f32 && x < (self.x + self.w) as f32 && y < (self.y + self.h) as f32
    }
}

/// 一档逻辑画布：格数 × 整数倍。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Canvas {
    pub grid_w: i32,
    pub grid_h: i32,
    pub scale: i32,
}

impl Canvas {
    /// Modern 布局（逻辑尺寸以 layout 为准：modern 最小钳位 = 原分辨率 240×320）+ 本壳呈现
    /// 配置（画布内软键栏 = [`CANVAS_SOFTKEY_BAR`]、标题底栏倍率 = [`CANVAS_TITLE_UI_K`]）。
    pub fn layout(&self) -> Layout {
        let mut l = Layout::modern(self.grid_w, self.grid_h);
        l.softkey_bar_visible = CANVAS_SOFTKEY_BAR;
        l.title_ui_k = CANVAS_TITLE_UI_K;
        l
    }

    /// 逻辑尺寸。
    pub fn logical(&self) -> (i32, i32) {
        let l = self.layout();
        (l.screen_w, l.screen_h)
    }

    /// 物理尺寸（逻辑 × scale）。
    pub fn physical(&self) -> (i32, i32) {
        let (w, h) = self.logical();
        (w * self.scale, h * self.scale)
    }

    /// 该档画布在视口内的放置（横纵居中）。
    pub fn rect_in(&self, viewport: Rect) -> Rect {
        let (w, h) = self.physical();
        Rect { x: viewport.x + (viewport.w - w) / 2, y: viewport.y + (viewport.h - h) / 2, w, h }
    }
}

/// 面板控件几何（**面板局部物理像素**；全部由 dp 与面板尺寸推导，见模块公式）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelGeom {
    /// 软键 [左 -6, 右 -7]（触控区 = 绘制框）。
    pub soft: [SoftZone; 2],
    /// 软键图标整数倍（⌊(T/2)/SOFTKEY_ICON_H⌋）。
    pub icon_scale: i32,
    /// 十字键心与直径 D、格 c = D/3。
    pub dpad: (f32, f32),
    pub dpad_d: f32,
    pub cell: f32,
    /// 确定钮心与半径（= c）。
    pub ok: (f32, f32),
    pub ok_r: f32,
    /// 描边粗（px，2dp）。
    pub outline: i32,
    /// 标签字形整数倍（round(14sp·dp / 字高)）。
    pub label_scale: i32,
}

impl PanelGeom {
    /// 触控区（input::TouchMapper 口径）：软键序 [确定, 左软, 右软]；确定 = 圆的外接方；
    /// 十字键死区 = 中格内切圆 c/2；r_switch = D/4 ⇒ 捕获半径 2·r_switch = D/2 = 盘面。
    pub fn pad(&self) -> PadLayout {
        let (ox, oy) = self.ok;
        let r = self.ok_r;
        PadLayout {
            dpad: DpadZone { cx: self.dpad.0, cy: self.dpad.1, r_inner: self.cell / 2.0, r_switch: self.dpad_d / 4.0 },
            soft: vec![
                SoftZone { code: KEY_FIRE, x0: ox - r, y0: oy - r, x1: ox + r, y1: oy + r },
                self.soft[0],
                self.soft[1],
            ],
        }
    }
}

/// 【计划 3 T5 · spec §4.2】工具栏几何（触屏档；窗口物理 px）。
#[derive(Clone, Debug, PartialEq)]
pub struct ToolbarGeom {
    /// 工具栏条（全宽，高 64dp）。
    pub bar: Rect,
    /// 左大键 / 右大键（各约 1/3 宽）。
    pub left: Rect,
    pub right: Rect,
    /// 中间小钮（0–4 个均分剩余；命中时按 idx 均分重算）。
    pub mid: Rect,
}

/// 竖屏规划结果。
#[derive(Clone, Debug, PartialEq)]
pub struct PortraitPlan {
    pub window: (i32, i32),
    /// 1dp 的物理像素数。
    pub dp: f32,
    /// 面板下限 P = 4T + 3M + G（px，向上取整）。
    pub panel_min: i32,
    /// 游戏场景档（Modern 格阵，摇杆面板档）。
    pub scene: Canvas,
    /// 开场链档（原分辨率 240×320）。
    pub boot: Canvas,
    /// 游戏视口（两档物理画布外接；横居中、顶贴内容区）。
    pub game: Rect,
    /// 控制面板（视口下方全宽）。
    pub panel: Rect,
    /// 面板控件几何（面板局部）。
    pub geom: PanelGeom,
    /// 面板触控区（= geom.pad()）。
    pub pad: PadLayout,
    /// 【计划 3 T5 · spec §4.1】触屏档场景（工具栏替代摇杆面板：V_t 更大 ⇒ 格阵更大或同）。
    pub scene_touch: Canvas,
    /// 触屏档游戏视口（scene_touch+boot 两档外接）。
    pub game_touch: Rect,
    /// 触屏档工具栏几何（视口下方全宽条）。
    pub toolbar: ToolbarGeom,
}

/// 竖屏规划（纯函数；公式见模块文档）。
pub fn plan_portrait(inp: &PlanInput) -> PortraitPlan {
    let (w, h) = (inp.window.0.max(1), inp.window.1.max(1));
    let top = inp.inset_top.clamp(0, h - 1);
    let dp = dpi_or_baseline(inp.dpi) as f32 / DP_BASELINE_DPI as f32;
    let (t, m) = (TOUCH_TARGET_DP * dp, MARGIN_DP * dp);
    let gesture = (inp.inset_bottom.max(0) as f32).max(GESTURE_AREA_DP * dp);
    let panel_min = (t + DPAD_CELLS * t + 3.0 * m + gesture).ceil() as i32;
    let v_max = (h - top - panel_min).max(0);
    // ① 开场链：原分辨率 × 最大整数倍（Layout::modern 最小钳位即 240×320）
    let s_boot = (w / SCREEN_W).min(v_max / SCREEN_H).max(1);
    let boot = Canvas { grid_w: SCREEN_W / CELL_PX, grid_h: SCREEN_H / CELL_PX, scale: s_boot };
    // ② 游戏：可读下限倍率 vs 看全层倍率，取大者，封顶开场链倍率
    let s_read = (MIN_TEXT_SP * dp / paint_font().height.max(1) as f32).ceil() as i32;
    let map_w = inp.map.cells_w * CELL_PX; // game_view mapPixelWidth = wide<<5
    let map_h = inp.map.cells_h * CELL_PX;
    let cols_max = ceil_div(map_w, CELL_PX);
    let rows_max = ceil_div(map_h + STATUS_BAR_H + HUD_FRAME_H, CELL_PX);
    let s_whole = if cols_max > 0 { (w / (cols_max * CELL_PX)).min(v_max / (rows_max * CELL_PX)) } else { 0 };
    let s = s_read.max(s_whole).clamp(1, s_boot);
    let scene = Canvas {
        grid_w: (w / (CELL_PX * s)).min(cols_max),
        grid_h: (v_max / (CELL_PX * s)).min(rows_max),
        scale: s,
    };
    // ③ 视口 = 两档外接；面板 = 下方全部剩余
    let (bw, bh) = boot.physical();
    let (sw, sh) = scene.physical();
    let (vw, vh) = (bw.max(sw).min(w), bh.max(sh));
    let game = Rect { x: (w - vw) / 2, y: top, w: vw, h: vh };
    let panel = Rect { x: 0, y: top + vh, w, h: (h - top - vh).max(0) };
    let geom = panel_geom(panel.w, panel.h, dp, gesture);

    // 【计划 3 T5 · spec §4.1】触屏档：工具栏高 64dp 贴手势区上；V_t = 内容区高 − 64dp − G
    let toolbar_h = (TOOLBAR_H_DP * dp).ceil() as i32;
    let v_touch = (h - top - toolbar_h - gesture as i32).max(0);
    // s_t：在 [s_read, min(4, s_boot)] 中取最大 s，使 ⌊W/32s⌋ ≥ 8 且 ⌊V_t/32s⌋ ≥ 10（C_min=8，§12）
    let mut s_t = 1usize;
    for cand in (1..=4usize).rev() {
        if w / (CELL_PX * cand as i32) >= 8 && v_touch / (CELL_PX * cand as i32) >= 10 {
            s_t = cand;
            break;
        }
    }
    let s_t = s_t.max(s_read.max(1) as usize).min(s_boot.max(1) as usize) as i32;
    let scene_touch = Canvas {
        grid_w: (w / (CELL_PX * s_t)).min(cols_max),
        grid_h: (v_touch / (CELL_PX * s_t)).min(rows_max),
        scale: s_t,
    };
    let (bw2, bh2) = (boot.physical().0.max(scene_touch.physical().0), boot.physical().1.max(scene_touch.physical().1));
    let game_touch = Rect { x: (w - bw2) / 2, y: top, w: bw2, h: bh2 };
    // 工具栏条 = **贴底**（spec §4.1：位于手势区之上 ⇒ bar_y = h − G − 64dp；复核 T5-R4——
    // 首版贴视口下沿留死带，已对齐）。视口高随之钳到 bar 上沿（bh2 超出时格阵已按 v_touch 钳，无害）。
    let bar_y = (h - gesture as i32 - toolbar_h).max(top);
    let toolbar = toolbar_geom(w, bar_y, toolbar_h, dp);

    if inp.control_mode == game_shell_common::settings::ControlMode::Touch {
        // 触屏档：视口=game_touch；面板区=工具栏（pad 恒空——无摇杆控件；TouchMapper 重建由 apply_portrait 完成）
        let pad_empty = PadLayout { dpad: DpadZone { cx: -10000.0, cy: -10000.0, r_inner: 0.0, r_switch: 0.0 }, soft: vec![] };
        PortraitPlan {
            window: (w, h),
            dp,
            panel_min,
            scene,
            boot,
            game: game_touch,
            panel: Rect { x: 0, y: toolbar.bar.y, w, h: h - toolbar.bar.y },
            pad: pad_empty,
            geom: PanelGeom {
                soft: [SoftZone { code: -6, x0: -10000.0, y0: -10000.0, x1: -9999.0, y1: -9999.0 }, SoftZone { code: -7, x0: -10000.0, y0: -10000.0, x1: -9999.0, y1: -9999.0 }],
                icon_scale: 1,
                dpad: (-10000.0, -10000.0),
                dpad_d: 0.0,
                cell: 0.0,
                ok: (-10000.0, -10000.0),
                ok_r: 0.0,
                outline: 1,
                label_scale: 1,
            },
            scene_touch,
            game_touch,
            toolbar,
        }
    } else {
        PortraitPlan {
            window: (w, h),
            dp,
            panel_min,
            scene,
            boot,
            game,
            panel,
            pad: geom.pad(),
            geom,
            scene_touch,
            game_touch,
            toolbar,
        }
    }
}

/// 【计划 3 T5 · spec §4.2】工具栏几何：条=面板区全宽、高 64dp（贴手势区上）；左/右大键各约 1/3、中间均分剩余。
fn toolbar_geom(w: i32, bar_y: i32, bar_h: i32, dp: f32) -> ToolbarGeom {
    let m = (MARGIN_DP * dp) as i32;
    let bar = Rect { x: 0, y: bar_y, w, h: bar_h };
    let usable_w = w - 2 * m;
    let big_w = usable_w * 30 / 100;
    let mid_x = m + big_w + m;
    let mid_w = usable_w - 2 * big_w - 2 * m;
    ToolbarGeom {
        bar,
        left: Rect { x: m, y: bar.y + m, w: big_w, h: bar_h - 2 * m },
        right: Rect { x: w - m - big_w, y: bar.y + m, w: big_w, h: bar_h - 2 * m },
        mid: Rect { x: mid_x, y: bar.y + m, w: mid_w, h: bar_h - 2 * m },
    }
}
/// 【计划 3 T5 · spec §4.2 · 复核 T5-R2】工具栏绘制（触屏档；无按键时隐藏钮=不画）。
/// `sides` = (左, 右) 各 (查询钮, 按下反馈)；`mid` = 中钮列表。文字 = controls_ui::label（壳侧单源）。
pub fn paint_toolbar(
    buf: &mut game_core::render::ArgbImage,
    tb: &ToolbarGeom,
    sides: &[(Option<(game_core::controls::FunctionId, game_core::controls::Action)>, bool); 2],
    mid: &[(game_core::controls::FunctionId, game_core::controls::Action)],
) {
    use game_core::render::FontMetrics;
    use game_shell_common::controls_ui::label;
    fill_rect(buf, tb.bar.x, tb.bar.y, tb.bar.x + tb.bar.w, tb.bar.y + tb.bar.h, PANEL_BG);
    fill_rect(buf, tb.bar.x, tb.bar.y, tb.bar.x + tb.bar.w, tb.bar.y + 2, PANEL_LINE); // 顶分隔线
    let mut draw_btn = |r: &Rect, text: &str, pressed: bool| {
        let face = if pressed { 0xFF3A3A52 } else { 0xFF2C2C3E };
        fill_rect(buf, r.x, r.y, r.x + r.w, r.y + r.h, face);
        fill_rect(buf, r.x, r.y, r.x + r.w, r.y + 1, 0xFF4A4A66);
        fill_rect(buf, r.x, r.y + r.h - 1, r.x + r.w, r.y + r.h, 0xFF4A4A66);
        fill_rect(buf, r.x, r.y, r.x + 1, r.y + r.h, 0xFF4A4A66);
        fill_rect(buf, r.x + r.w - 1, r.y, r.x + r.w, r.y + r.h, 0xFF4A4A66);
        // 文本（游戏字体，居中；draw_string TOP|LEFT 锚）
        let font = game_core::paint::paint_font();
        let u16s: Vec<u16> = text.encode_utf16().collect();
        let (tw, th) = font.measure(&u16s);
        let cx = r.x + (r.w - tw) / 2;
        let cy = r.y + (r.h - th) / 2;
        let g = &mut game_core::render::SoftGraphics::new(buf);
        g.set_color(0xF0E8D0);
        g.draw_string(&u16s, cx, cy, 20); // TOP|LEFT
    };
    if let Some((f, _)) = sides[0].0 {
        draw_btn(&tb.left, label(f), sides[0].1);
    }
    if let Some((f, _)) = sides[1].0 {
        draw_btn(&tb.right, label(f), sides[1].1);
    }
    if !mid.is_empty() {
        let n = mid.len() as i32;
        let w = tb.mid.w / n;
        for (i, (f, _)) in mid.iter().enumerate() {
            let r = Rect { x: tb.mid.x + w * i as i32, y: tb.mid.y, w, h: tb.mid.h };
            draw_btn(&r, label(*f), false);
        }
    }
}


/// 面板控件几何（面板局部 px；公式见模块文档"面板"行）。
fn panel_geom(pw: i32, ph: i32, dp: f32, gesture: f32) -> PanelGeom {
    let (pw, ph) = (pw as f32, ph as f32);
    let (t, m) = (TOUCH_TARGET_DP * dp, MARGIN_DP * dp);
    // 软键行（顶）：高 T；图标 ≈ T/2 的整数倍；宽 = max(按钮最小宽, 图标 + 两侧边距)
    let icon_scale = ((t * ICON_OF_TARGET) / SOFTKEY_ICON_H as f32).floor().max(1.0) as i32;
    let soft_w = (BUTTON_MIN_W_DP * dp).max((SOFTKEY_ICON_W * icon_scale) as f32 + 2.0 * m);
    let soft = [
        SoftZone { code: KEY_LEFT_SOFT, x0: m, y0: m, x1: m + soft_w, y1: m + t },
        SoftZone { code: KEY_RIGHT_SOFT, x0: pw - m - soft_w, y0: m, x1: pw - m, y1: m + t },
    ];
    // 操作簇区：软键行下 M 起，至手势区上 M 止；两簇并排各占 (W−3M)/2
    let (top, bottom) = (2.0 * m + t, ph - gesture - m);
    let d = (bottom - top).min((pw - 3.0 * m) / 2.0).min(DPAD_MAX_DP * dp).max(0.0);
    let cell = d / DPAD_CELLS;
    let cy = (top + bottom) / 2.0;
    PanelGeom {
        soft,
        icon_scale,
        dpad: (m + d / 2.0, cy),
        dpad_d: d,
        cell,
        ok: (pw - m - d / 2.0, cy),
        ok_r: OK_CELLS * cell / 2.0,
        outline: (OUTLINE_DP * dp).round().max(1.0) as i32,
        label_scale: (LABEL_SP * dp / paint_font().height.max(1) as f32).round().max(1.0) as i32,
    }
}

/// 帧合成：清不透黑 → 各层 1:1 拷入（裁到窗口内）。层 = (ARGB 像素, 宽, 高, x, y)；
/// 目标 = 窗口 RGBA8888（字节 [R,G,B,A]）、行距 `stride` 像素（≥ buf_w，行尾不写）。
pub fn compose_layers(
    dst: &mut [u8],
    buf_w: i32,
    buf_h: i32,
    stride: i32,
    layers: &[(&[u32], i32, i32, i32, i32)],
) {
    let stride = stride.max(buf_w) as usize;
    for y in 0..buf_h.max(0) as usize {
        for x in 0..buf_w.max(0) as usize {
            let o = (y * stride + x) * 4;
            if o + 4 <= dst.len() {
                dst[o..o + 4].copy_from_slice(&[0, 0, 0, 0xFF]);
            }
        }
    }
    for &(px, w, h, lx, ly) in layers {
        for sy in 0..h {
            let dy = ly + sy;
            if dy < 0 || dy >= buf_h {
                continue;
            }
            for sx in 0..w {
                let dx = lx + sx;
                if dx < 0 || dx >= buf_w {
                    continue;
                }
                let Some(p) = px.get((sy * w + sx) as usize) else { continue };
                let o = (dy as usize * stride + dx as usize) * 4;
                if o + 4 <= dst.len() {
                    dst[o..o + 4].copy_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, *p as u8, (p >> 24) as u8]);
                }
            }
        }
    }
}

/// 面板绘制输入（变化才重绘：按住的键 + 活动机软键种类）。
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PanelState {
    /// 当前按住的键码（方向/确定/软键）。
    pub held: Vec<i32>,
    /// 活动机软键种类（左, 右）：0=不显示图标；k≥1 = ui[11] 第 k 个图标。
    pub softkeys: (i8, i8),
}

const PANEL_BG: u32 = 0xFF00_0000 | game_core::layout::DARK_BACKDROP; // 底栏同色
const PANEL_LINE: u32 = 0xFF5A_4F6E;
const KEY_FACE: u32 = 0xFF46_3C55;
const KEY_FACE_DIM: u32 = 0xFF3A_3246;
const KEY_FACE_HOT: u32 = 0xFF6A_5C80;
const ARROW: u32 = 0xFFC8_B98A;
const ARROW_HOT: u32 = 0xFFFF_D75A;
const OK_FACE: u32 = 0xFF9E_3B3B;
const OK_FACE_HOT: u32 = 0xFFD8_5A50;

fn put(img: &mut ArgbImage, x: i32, y: i32, c: u32) {
    if x >= 0 && y >= 0 && x < img.width && y < img.height {
        img.argb[(y * img.width + x) as usize] = c;
    }
}

fn fill_rect(img: &mut ArgbImage, x0: i32, y0: i32, x1: i32, y1: i32, c: u32) {
    for y in y0.max(0)..y1.min(img.height) {
        for x in x0.max(0)..x1.min(img.width) {
            img.argb[(y * img.width + x) as usize] = c;
        }
    }
}

fn fill_circle(img: &mut ArgbImage, cx: f32, cy: f32, r: f32, c: u32) {
    let (x0, x1) = ((cx - r).floor() as i32, (cx + r).ceil() as i32);
    let (y0, y1) = ((cy - r).floor() as i32, (cy + r).ceil() as i32);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            if dx * dx + dy * dy <= r * r {
                put(img, x, y, c);
            }
        }
    }
}

/// 实心三角（点在三边同侧判定）。
fn fill_tri(img: &mut ArgbImage, p: [(f32, f32); 3], c: u32) {
    let xs = p.iter().map(|q| q.0);
    let ys = p.iter().map(|q| q.1);
    let (x0, x1) = (xs.clone().fold(f32::MAX, f32::min) as i32, xs.fold(f32::MIN, f32::max).ceil() as i32);
    let (y0, y1) = (ys.clone().fold(f32::MAX, f32::min) as i32, ys.fold(f32::MIN, f32::max).ceil() as i32);
    let edge = |a: (f32, f32), b: (f32, f32), x: f32, y: f32| (b.0 - a.0) * (y - a.1) - (b.1 - a.1) * (x - a.0);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let e0 = edge(p[0], p[1], fx, fy);
            let e1 = edge(p[1], p[2], fx, fy);
            let e2 = edge(p[2], p[0], fx, fy);
            if (e0 >= 0.0 && e1 >= 0.0 && e2 >= 0.0) || (e0 <= 0.0 && e1 <= 0.0 && e2 <= 0.0) {
                put(img, x, y, c);
            }
        }
    }
}

/// 软键图标（ui[11] 第 kind 个 SOFTKEY_ICON_W×H，整数倍 k 放大居中；alpha=0 跳过）。
fn blit_icon(img: &mut ArgbImage, icons: &ArgbImage, kind: i8, cx: f32, cy: f32, k: i32) {
    if kind <= 0 {
        return;
    }
    let (iw, ih) = (SOFTKEY_ICON_W, SOFTKEY_ICON_H);
    let sx0 = (kind as i32 - 1) * iw;
    let (ox, oy) = ((cx - (iw * k) as f32 / 2.0) as i32, (cy - (ih * k) as f32 / 2.0) as i32);
    for v in 0..ih {
        for u in 0..iw {
            let (sx, sy) = (sx0 + u, v);
            if sx >= icons.width || sy >= icons.height {
                continue;
            }
            let p = icons.argb[(sy * icons.width + sx) as usize];
            if p >> 24 == 0 {
                continue;
            }
            fill_rect(img, ox + u * k, oy + v * k, ox + (u + 1) * k, oy + (v + 1) * k, p | 0xFF00_0000);
        }
    }
}

/// 面板绘制（物理像素直绘；全部尺寸取自 [`PanelGeom`]；"确定"字形经 SoftGraphics
/// ×label_scale 走字形钩子）。
pub fn paint_panel(img: &mut ArgbImage, g: &PanelGeom, st: &PanelState, icons: &ArgbImage) {
    let (w, h) = (img.width, img.height);
    let o = g.outline;
    let of = o as f32;
    fill_rect(img, 0, 0, w, h, PANEL_BG);
    fill_rect(img, 0, 0, w, o, PANEL_LINE); // 顶分隔线
    let held = |c: i32| st.held.contains(&c);
    // 软键（左 -6 / 右 -7）：描边框 + 面；图标 = 游戏同款 ui[11]（与画布角标同义）
    for (z, kind) in [(g.soft[0], st.softkeys.0), (g.soft[1], st.softkeys.1)] {
        let face = if held(z.code) {
            KEY_FACE_HOT
        } else if kind == 0 {
            KEY_FACE_DIM
        } else {
            KEY_FACE
        };
        let (x0, y0, x1, y1) = (z.x0 as i32, z.y0 as i32, z.x1 as i32, z.y1 as i32);
        fill_rect(img, x0, y0, x1, y1, PANEL_LINE);
        fill_rect(img, x0 + o, y0 + o, x1 - o, y1 - o, face);
        blit_icon(img, icons, kind, (z.x0 + z.x1) / 2.0, (z.y0 + z.y1) / 2.0, g.icon_scale);
    }
    // 十字键：盘面 + 描边；四臂箭头（臂格心距盘心 1 格，按住高亮）；中格 = 死区盘心
    let (cx, cy) = g.dpad;
    let (r, c) = (g.dpad_d / 2.0, g.cell);
    fill_circle(img, cx, cy, r, PANEL_LINE);
    fill_circle(img, cx, cy, r - of, KEY_FACE);
    let a = c * ARROW_FRAC;
    for (code, (ux, uy)) in [(-1, (0.0, -1.0)), (-2, (0.0, 1.0)), (KEY_LEFT, (-1.0, 0.0)), (KEY_RIGHT, (1.0, 0.0))] {
        let (tx, ty) = (cx + ux * (c + a), cy + uy * (c + a)); // 顶点
        let (bx, by) = (cx + ux * (c - a), cy + uy * (c - a)); // 底边中点
        let (px, py) = (-uy * a, ux * a);
        let col = if held(code) { ARROW_HOT } else { ARROW };
        fill_tri(img, [(tx, ty), (bx + px, by + py), (bx - px, by - py)], col);
    }
    fill_circle(img, cx, cy, c / 2.0, PANEL_LINE);
    fill_circle(img, cx, cy, c / 2.0 - of, KEY_FACE_DIM);
    // 确定钮（-5）
    let (ox, oy) = g.ok;
    fill_circle(img, ox, oy, g.ok_r, PANEL_LINE);
    fill_circle(img, ox, oy, g.ok_r - of, if held(KEY_FIRE) { OK_FACE_HOT } else { OK_FACE });
    let k = g.label_scale;
    let font = paint_font();
    let fh = font.height;
    let mut gfx = SoftGraphics::new(img);
    gfx.set_scale(k);
    gfx.set_font(Some(font));
    gfx.set_color(0xFFFFFF);
    let label: Vec<u16> = "确定".encode_utf16().collect();
    gfx.draw_string(
        &label,
        (ox / k as f32) as i32,
        (oy / k as f32) as i32 - fh / 2,
        game_core::render::anchor::TOP | game_core::render::anchor::HCENTER,
    );
}
