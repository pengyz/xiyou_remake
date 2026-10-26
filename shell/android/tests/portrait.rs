//! 【2026-10-26 竖屏动态分辨率】宿主钉：竖屏组成规划（游戏视口在上、控制面板在下）、
//! 触点分区路由、帧合成、面板绘制、运行时双档布局。用户："动态分辨率，默认竖屏，
//! 先做 Xiaomi 14 上合理的布局看一下"。
//!
//! > 【2026-10-26 更正】首版钉的是设备特化魔数（12 列 / 30% 面板 / 132px 按钮…）。用户质询
//! > "手机分辨率变了，你这些数字还准确吗？我们需要计算公式" ⇒ 本版改钉**公式**：
//! > ① Xiaomi 14 逐步推导算例（每一步的期望值为手算字面量，F-1：不以被测输出作期望）；
//! > ② 设备矩阵不变量（dpi/尺寸任意组合下：48dp 触控目标、面板下限、原分辨率下限、
//! >    满图上限、整数倍极大性、可读字号）；③ 输入（dpi/内容区/地图头）全来自设备与数据。

use game_android::input::{KeyAction, TouchPhase};
use game_android::portrait::{
    compose_layers, dpi_or_baseline, paint_panel, plan_portrait, MapExtent, PanelState, PlanInput, PortraitPlan, Rect,
    BUTTON_MIN_W_DP, DPAD_MAX_DP, GESTURE_AREA_DP, MARGIN_DP, MIN_TEXT_SP, TOUCH_TARGET_DP,
};
use game_core::layout::{CELL_PX, HUD_FRAME_H, SCREEN_H, SCREEN_W, STATUS_BAR_H};

/// 全关卡最大格数（maplv 头 >>1：54 张 13×13 + maplv51 12×13，见 map_extent 测试）。
const MAP: MapExtent = MapExtent { cells_w: 13, cells_h: 13 };

/// Xiaomi 14 竖屏全屏窗口实测（logcat portrait plan：1200×2536，设备 y=134 起——状态栏/挖孔
/// 让出；dumpsys：密度 480、底部 navigationBars=mandatorySystemGestures=48px）。
fn xm14() -> PlanInput {
    PlanInput { window: (1200, 2536), dpi: 480, inset_top: 0, inset_bottom: 0, map: MAP }
}

fn assets() -> game_shell_common::asset::DirAssetSource {
    game_shell_common::asset::DirAssetSource::new(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw"))
}

#[test]
fn map_extent_is_max_over_all_level_headers() {
    // 数据源 = 全部 maplv{n} 头（u16 LE ×2，>>1 = 32px 格）：54 张 (26,26)>>1=13×13、
    // maplv51 (24,26)>>1=12×13（python 头统计 2026-10-26）⇒ 最大 13×13
    assert_eq!(MapExtent::from_assets(&assets()), MapExtent { cells_w: 13, cells_h: 13 });
}

#[test]
fn dpi_unknown_codes_fall_back_to_dp_baseline() {
    // AConfiguration_getDensity：DEFAULT=0 / ANY=0xfffe / NONE=0xffff ⇒ 160（1dp=1px 基准）
    assert_eq!(dpi_or_baseline(480), 480);
    assert_eq!(dpi_or_baseline(0), 160);
    assert_eq!(dpi_or_baseline(0xfffe), 160);
    assert_eq!(dpi_or_baseline(0xffff), 160);
}

/// Xiaomi 14 逐步推导（手算字面量）：
/// dp=480/160=3；T=48dp=144；M=16dp=48；G=max(0,32dp)=96；面板下限 P=4T+3M+G=816；
/// 可用视口高 V=2536−816=1720。
/// 开场链：s_b=min(⌊1200/240⌋,⌊1720/320⌋)=min(5,5)=5 ⇒ 240×320@5=1200×1600。
/// 游戏：s_read=⌈12·3/14⌉=⌈2.57⌉=3；满图 C=13、R=⌈(416+68)/32⌉=16；
/// s_whole=min(⌊1200/416⌋,⌊1720/512⌋)=min(2,3)=2；s=clamp(max(3,2),1,5)=3；
/// 列=min(⌊1200/96⌋,13)=12；行=min(⌊1720/96⌋,16)=16 ⇒ 384×512@3=1152×1536。
/// 视口=外接 1200×1600；面板=2536−1600=936。
#[test]
fn xiaomi14_worked_derivation() {
    let p = plan_portrait(&xm14());
    assert_eq!(p.dp, 3.0);
    assert_eq!(p.panel_min, 816, "4T+3M+G = 576+144+96");
    assert_eq!((p.boot.logical(), p.boot.scale), ((240, 320), 5), "开场链 = 原分辨率 × 最大整数倍");
    assert_eq!(p.boot.physical(), (1200, 1600));
    assert_eq!((p.scene.grid_w, p.scene.grid_h, p.scene.scale), (12, 16, 3), "游戏 = 可读下限倍率下最大视野");
    assert_eq!(p.scene.physical(), (1152, 1536));
    assert_eq!(p.game, Rect { x: 0, y: 0, w: 1200, h: 1600 }, "视口 = 两档画布外接");
    assert_eq!(p.scene.rect_in(p.game), Rect { x: 24, y: 32, w: 1152, h: 1536 }, "游戏画布在视口内居中");
    assert_eq!(p.panel, Rect { x: 0, y: 1600, w: 1200, h: 936 }, "面板 = 视口下方全部剩余");
    // 面板（局部 px）：软键行 [M,M+T)=[48,192)；图标倍率 ⌊(T/2)/10⌋=7 ⇒ 84×70；
    // 软键宽=max(64dp=192, 84+2M=180)=192；簇区 [2M+T, 面板高−G−M)=[240,792) 高 552；
    // D=min(552,(1200−3M)/2=528,192dp=576)=528；c=176；簇心 y=(240+792)/2=516
    let g = &p.geom;
    assert_eq!((g.soft[0].x0, g.soft[0].y0, g.soft[0].x1, g.soft[0].y1), (48.0, 48.0, 240.0, 192.0), "左软键");
    assert_eq!((g.soft[1].x0, g.soft[1].x1), (960.0, 1152.0), "右软键贴右边距");
    assert_eq!(g.icon_scale, 7);
    assert_eq!((g.dpad_d, g.cell), (528.0, 176.0));
    assert_eq!(g.dpad, (312.0, 516.0), "十字键心 = (M+D/2, 簇心)");
    assert_eq!((g.ok, g.ok_r), ((888.0, 516.0), 176.0), "确定钮 = 右侧镜像位、直径 2 格");
    assert_eq!(g.outline, 6, "描边 2dp");
    assert_eq!(g.label_scale, 3, "标签 round(14sp·3/14)");
    // 触控区由几何派生：死区=中格内切圆 c/2、捕获半径 2·r_switch = D/2（盘面）
    assert_eq!((p.pad.dpad.cx, p.pad.dpad.cy, p.pad.dpad.r_inner, p.pad.dpad.r_switch), (312.0, 516.0, 88.0, 132.0));
    let codes: Vec<i32> = p.pad.soft.iter().map(|z| z.code).collect();
    assert_eq!(codes, vec![-5, -6, -7], "确定 / 左软键 / 右软键");
}

/// 独立复算的面板下限（测试侧自推，不取被测输出）。
fn panel_min_of(inp: &PlanInput) -> f32 {
    let dp = dpi_or_baseline(inp.dpi) as f32 / 160.0;
    let (t, m) = (TOUCH_TARGET_DP * dp, MARGIN_DP * dp);
    4.0 * t + 3.0 * m + (inp.inset_bottom as f32).max(GESTURE_AREA_DP * dp)
}

fn check_invariants(inp: &PlanInput) -> PortraitPlan {
    let p = plan_portrait(inp);
    let tag = format!("{:?}@{}dpi", inp.window, inp.dpi);
    let (w, h) = inp.window;
    let dp = inp.dpi as f32 / 160.0;
    let (t, m) = (TOUCH_TARGET_DP * dp, MARGIN_DP * dp);
    let gesture = (inp.inset_bottom as f32).max(GESTURE_AREA_DP * dp);
    let v = h - inp.inset_top - panel_min_of(inp).ceil() as i32;
    // —— 画布 ——
    assert_eq!(p.boot.logical(), (SCREEN_W, SCREEN_H), "{tag}：开场链 = 原分辨率");
    let sb = p.boot.scale;
    assert!(SCREEN_W * sb <= w && SCREEN_H * sb <= v, "{tag}：开场链放得下");
    assert!(SCREEN_W * (sb + 1) > w || SCREEN_H * (sb + 1) > v, "{tag}：开场链倍率极大");
    let (sl, s) = (p.scene.logical(), p.scene.scale);
    assert!(sl.0 >= SCREEN_W && sl.1 >= SCREEN_H, "{tag}：游戏画布 ≥ 原分辨率");
    let rows_max = (MAP.cells_h * CELL_PX + STATUS_BAR_H + HUD_FRAME_H + CELL_PX - 1) / CELL_PX;
    assert!(p.scene.grid_w <= MAP.cells_w && p.scene.grid_h <= rows_max, "{tag}：不超满图（再大只是空背景）");
    assert!((1..=sb).contains(&s), "{tag}：游戏倍率 ∈ [1, 开场链倍率]");
    let font_h = game_core::paint::paint_font().height as f32;
    assert!(font_h * s as f32 >= MIN_TEXT_SP * dp || s == sb, "{tag}：正文 ≥ 12sp（仅原分辨率放不下时让步）");
    let (sw, sh) = p.scene.physical();
    assert!(sw <= p.game.w && sh <= p.game.h, "{tag}：游戏画布 ⊆ 视口");
    let (bw, bh) = p.boot.physical();
    assert!(bw <= p.game.w && bh <= p.game.h, "{tag}：开场链画布 ⊆ 视口");
    // —— 视口 / 面板 ——
    assert!(p.game.x >= 0 && p.game.x + p.game.w <= w && p.game.y == inp.inset_top, "{tag}：视口在窗内、顶贴内容区");
    assert_eq!((p.panel.y, p.panel.y + p.panel.h), (p.game.y + p.game.h, h), "{tag}：面板 = 视口下方全部剩余");
    assert!(p.panel.h as f32 >= panel_min_of(inp) - 0.5, "{tag}：面板 ≥ 下限 4T+3M+G（实得 {}）", p.panel.h);
    // —— 控件 ——
    let g = &p.geom;
    let (pw, ph) = (p.panel.w as f32, p.panel.h as f32);
    let floor = ph - gesture; // 手势区上沿
    let (dx, dy) = g.dpad;
    let r = g.dpad_d / 2.0;
    assert!(dx - r >= m - 0.5 && dx + r <= pw && dy - r >= 0.0 && dy + r <= floor - m + 0.5, "{tag}：十字键在面板内、手势区之上");
    assert!(g.dpad_d <= DPAD_MAX_DP * dp + 0.5, "{tag}：十字键 ≤ 192dp");
    let (ox, oy) = g.ok;
    assert!(ox + g.ok_r <= pw - m + 0.5 && oy + g.ok_r <= floor, "{tag}：确定钮在面板内、手势区之上");
    assert!((ox - dx).hypot(oy - dy) >= r + g.ok_r + m - 0.5, "{tag}：确定钮与十字键至少隔 M");
    for z in &g.soft {
        assert!(z.x0 >= 0.0 && z.x1 <= pw && z.y0 >= 0.0 && z.y1 <= floor, "{tag}：软键在面板内");
        assert!(z.y1 - z.y0 >= t - 0.5 && z.x1 - z.x0 >= BUTTON_MIN_W_DP * dp - 0.5, "{tag}：软键 ≥ 64×48dp");
        let (nx, ny) = (dx.clamp(z.x0, z.x1), dy.clamp(z.y0, z.y1));
        assert!((nx - dx).hypot(ny - dy) >= r, "{tag}：软键不压十字键");
    }
    assert!(g.soft[0].x1 <= g.soft[1].x0, "{tag}：左右软键不重叠");
    // 48dp 触控目标：两簇并排所需宽 6T+3M（=336dp）以上的设备必须满足
    if w as f32 >= 6.0 * t + 3.0 * m {
        assert!(g.cell >= t - 0.5, "{tag}：十字键每臂格 ≥ 48dp（实得 {}）", g.cell);
        assert!(2.0 * g.ok_r >= t, "{tag}：确定钮 ≥ 48dp");
    }
    // 触控区 = 几何派生（可见即可触）
    assert_eq!(p.pad.dpad.r_switch * 2.0, r, "{tag}：捕获半径 = 盘面半径");
    assert!(p.pad.dpad.r_inner < p.pad.dpad.r_switch, "{tag}：死区 ⊂ 迟滞带");
    p
}

#[test]
fn plan_invariants_hold_across_devices() {
    let devices = [
        (1200, 2536, 480), // Xiaomi 14 实测窗口
        (1200, 2670, 480), // 同机全屏面板
        (1200, 2536, 440), // 同机"显示大小"调小
        (1200, 2536, 520), // 同机"显示大小"调大
        (1080, 2400, 420),
        (1080, 2340, 440),
        (1440, 3200, 560),
        (720, 1600, 320),
        (1600, 2560, 320),  // 平板竖持
        (1812, 2176, 420),  // 折叠屏内屏
        (480, 800, 240),    // 窄屏（320dp < 336dp：48dp 臂格不保证，其余不变量照钉）
    ];
    for (w, h, dpi) in devices {
        check_invariants(&PlanInput { window: (w, h), dpi, inset_top: 0, inset_bottom: 0, map: MAP });
    }
}

#[test]
fn density_drives_control_size_and_scene_scale() {
    // 同像素窗口、不同密度：控件物理尺寸恒（48dp），游戏倍率随"可读下限"变
    let hi = check_invariants(&xm14());
    let lo = check_invariants(&PlanInput { dpi: 320, ..xm14() });
    assert_eq!(hi.geom.soft[0].y1 - hi.geom.soft[0].y0, 144.0, "480dpi：T=48dp=144px");
    assert_eq!(lo.geom.soft[0].y1 - lo.geom.soft[0].y0, 96.0, "320dpi：T=48dp=96px");
    // 320dpi：s_read=⌈12·2/14⌉=2；满图 416×512 在 2× 放得下（832×1024）⇒ 看全层
    assert_eq!((lo.scene.grid_w, lo.scene.grid_h, lo.scene.scale), (13, 16, 2), "低密度屏同像素 = 物理更大 ⇒ 看全层");
}

#[test]
fn map_extent_bounds_scene_grid() {
    // 小地图（8×8 格 = 256²）：满图即上限，倍率升到满图放得下的最大值
    let p = check_invariants(&PlanInput { map: MapExtent { cells_w: 8, cells_h: 8 }, ..xm14() });
    // C=8、R=⌈(256+68)/32⌉=11；s_whole=min(⌊1200/256⌋,⌊1720/352⌋)=min(4,4)=4；
    // 列=min(⌊1200/128⌋,8)=8、行=min(⌊1720/128⌋,11)=11 ⇒ 256×352@4
    assert_eq!((p.scene.grid_w, p.scene.grid_h, p.scene.scale), (8, 11, 4));
}

#[test]
fn content_insets_shift_viewport_and_lift_controls() {
    // 内容区上缩 100（状态栏未隐藏等）：视口顶贴内容区；下缩 300 > 32dp：控件让到其上
    let p = check_invariants(&PlanInput { inset_top: 100, inset_bottom: 300, ..xm14() });
    assert_eq!(p.game.y, 100);
    let lowest = p.geom.dpad.1 + p.geom.dpad_d / 2.0;
    assert!(lowest <= p.panel.h as f32 - 300.0, "控件底 {lowest} 在实测内缩之上");
    // 下缩 48（Xiaomi 14 实测 16dp）< 32dp：取规范下限，与无内缩同规划
    assert_eq!(plan_portrait(&PlanInput { inset_bottom: 48, ..xm14() }), plan_portrait(&xm14()));
}

#[test]
fn wiring_derives_insets_from_content_rect_and_replans_on_change() {
    use game_android::native::ShellWiring;
    let mut w = ShellWiring::new();
    w.dpi = 480;
    w.map_extent = MAP;
    // 内容区未知 ⇒ 内缩 0
    assert_eq!(w.portrait_input(1200, 2536), xm14());
    // glue contentRect（窗口坐标 l,t,r,b）⇒ 上缩=top、下缩=窗高−bottom
    w.content_rect = Some((0, 0, 1200, 2488));
    let inp = w.portrait_input(1200, 2536);
    assert_eq!((inp.inset_top, inp.inset_bottom), (0, 48));
    w.apply_portrait(plan_portrait(&inp));
    assert_eq!(w.replan_portrait(), None, "内缩 48 < 32dp：规划不变 ⇒ 不重排");
    w.content_rect = Some((0, 0, 1200, 2236));
    let re = w.replan_portrait().expect("内缩 300 > 32dp ⇒ 重排");
    assert_eq!(Some(&re), w.portrait.as_ref(), "重排结果已采用");
    assert!(re.geom.dpad.1 + re.geom.dpad_d / 2.0 <= re.panel.h as f32 - 300.0);
}

#[test]
fn portrait_touch_routes_panel_to_pad_and_viewport_to_game() {
    use game_android::native::{ShellWiring, TitleGesture};
    let mut w = ShellWiring::new();
    w.apply_portrait(plan_portrait(&xm14()));
    let p = w.portrait.clone().unwrap();
    let (px, py) = (p.panel.x as f32, p.panel.y as f32);
    let g = &p.geom;
    // 面板：十字键右臂格心（距盘心 1 格）→ -4；拖进视口仍归面板；抬起 → Release
    let (rx, ry) = (px + g.dpad.0 + g.cell, py + g.dpad.1);
    assert_eq!(w.on_touch_device(1, TouchPhase::Down, rx, ry), vec![KeyAction::Press(-4)]);
    assert!(w.on_touch_device(1, TouchPhase::Move, rx, p.game.y as f32 + 1.0).len() <= 2);
    assert_eq!(w.on_touch_device(1, TouchPhase::Up, rx, p.game.y as f32 + 1.0), vec![KeyAction::Release]);
    // 面板：盘心（死区）无键
    assert!(w.on_touch_device(5, TouchPhase::Down, px + g.dpad.0, py + g.dpad.1).is_empty(), "盘心死区");
    w.on_touch_device(5, TouchPhase::Up, px + g.dpad.0, py + g.dpad.1);
    // 面板：确定钮心 → -5
    let (ox, oy) = (px + g.ok.0, py + g.ok.1);
    assert_eq!(w.on_touch_device(2, TouchPhase::Down, ox, oy), vec![KeyAction::Press(-5)]);
    w.on_touch_device(2, TouchPhase::Up, ox, oy);
    // 视口（标题态，开场链档）：设备 → 逻辑 = (设备 − 画布原点) / s_b
    w.title_mode = true;
    w.set_active_canvas(p.boot);
    let b = p.boot.rect_in(p.game);
    let sb = p.boot.scale as f32;
    let (tx, ty) = (b.x as f32 + 120.0 * sb, b.y as f32 + 160.0 * sb);
    assert!(w.on_touch_device(3, TouchPhase::Down, tx, ty).is_empty(), "视口触点不出键");
    w.on_touch_device(3, TouchPhase::Up, tx, ty);
    assert_eq!(std::mem::take(&mut w.title_gestures), vec![TitleGesture::Tap(120.0, 160.0)]);
    // 视口（游戏态，scene 档）：Up → 待决点按
    w.title_mode = false;
    w.set_active_canvas(p.scene);
    let c = p.scene.rect_in(p.game);
    let s = p.scene.scale as f32;
    let (gx, gy) = (c.x as f32 + 100.0 * s, c.y as f32 + 200.0 * s);
    w.on_touch_device(4, TouchPhase::Down, gx, gy);
    w.on_touch_device(4, TouchPhase::Up, gx, gy);
    assert_eq!(std::mem::take(&mut w.pending_taps), vec![(100.0, 200.0)]);
}

#[test]
fn compose_layers_places_canvas_and_panel_rgba_with_stride() {
    // 窗 6×5、行距 8：画布 2×2 @ (1,0)、面板 6×2 @ (0,3)；其余=不透黑
    let canvas = vec![0xFF112233u32; 4];
    let panel = vec![0xFF445566u32; 12];
    let mut dst = vec![0xAAu8; 8 * 5 * 4];
    compose_layers(&mut dst, 6, 5, 8, &[(&canvas, 2, 2, 1, 0), (&panel, 6, 2, 0, 3)]);
    let px = |x: usize, y: usize| &dst[(y * 8 + x) * 4..(y * 8 + x) * 4 + 4];
    assert_eq!(px(1, 0), &[0x11, 0x22, 0x33, 0xFF], "画布像素 RGBA 字节序");
    assert_eq!(px(2, 1), &[0x11, 0x22, 0x33, 0xFF]);
    assert_eq!(px(0, 0), &[0, 0, 0, 0xFF], "信箱=不透黑");
    assert_eq!(px(3, 2), &[0, 0, 0, 0xFF]);
    assert_eq!(px(5, 4), &[0x44, 0x55, 0x66, 0xFF], "面板像素");
    assert_eq!(dst[(0 * 8 + 6) * 4], 0xAA, "行距尾部（stride 外）不写");
}

#[test]
fn panel_paint_shows_dpad_ok_and_pressed_feedback() {
    let p = plan_portrait(&xm14());
    let g = &p.geom;
    let icons = game_core::render::ArgbImage::create(36, 10);
    let mut a = game_core::render::ArgbImage::create(p.panel.w, p.panel.h);
    paint_panel(&mut a, g, &PanelState { held: vec![], softkeys: (1, 3) }, &icons);
    let at = |img: &game_core::render::ArgbImage, x: f32, y: f32| img.argb[(y as i32 * img.width + x as i32) as usize];
    assert!(a.argb.iter().all(|px| px >> 24 == 0xFF), "面板全不透明");
    let bg = at(&a, 2.0, p.panel.h as f32 - 2.0);
    let (dx, dy) = g.dpad;
    assert_ne!(at(&a, dx, dy - g.cell), bg, "十字键上臂箭头（臂格心）");
    let ok_idle = at(&a, g.ok.0, g.ok.1 + g.ok_r * 0.6);
    assert_ne!(ok_idle, bg, "确定钮有图");
    assert_eq!(at(&a, dx, dy + g.dpad_d / 2.0 + 2.0 * g.outline as f32), bg, "盘下沿外 = 底色");
    // 按下反馈：确定 / 方向右
    let mut b = game_core::render::ArgbImage::create(p.panel.w, p.panel.h);
    paint_panel(&mut b, g, &PanelState { held: vec![-5, -4], softkeys: (1, 3) }, &icons);
    assert_ne!(at(&b, g.ok.0, g.ok.1 + g.ok_r * 0.6), ok_idle, "按下确定=变色");
    assert_ne!(at(&b, dx + g.cell, dy), at(&a, dx + g.cell, dy), "按下右=右箭头高亮");
}

#[test]
fn runtime_dual_layouts_handoff_uses_scene_layout() {
    use game_android::runtime::{build_boot_machine, Runtime};
    let src = assets();
    let p = plan_portrait(&xm14());
    let dir = std::env::temp_dir().join(format!("jjxy-portrait-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (rms, stores) = game_shell_common::rms::FileRms::load(&dir);
    let machine = build_boot_machine(&src, p.scene.layout());
    let mut rt = Runtime::new(machine, stores, Box::new(rms));
    rt.apply_layouts(p.boot.layout(), p.scene.layout());
    assert_eq!((rt.machine.layout.screen_w, rt.machine.layout.screen_h), (SCREEN_W, SCREEN_H), "开场链 = 原分辨率");
    assert_eq!(rt.active_canvas(&p), p.boot, "开场期活动画布=boot 档");
    rt.machine.new_game_ready = true;
    rt.machine.mode = game_core::enums::GameMode::Map;
    rt.step(&src, &[], Vec::new(), Vec::new());
    let sc = rt.scene.as_ref().expect("交接");
    assert_eq!((sc.layout.screen_w, sc.layout.screen_h), p.scene.logical(), "游戏场景=scene 档");
    assert_eq!(rt.active_canvas(&p), p.scene, "场景期活动画布=scene 档");
    let _ = std::fs::remove_dir_all(&dir);
}
