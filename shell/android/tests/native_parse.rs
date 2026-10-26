//! T5 宿主侧解析单测（task-14 TDD 红先行）：AInputEvent 解析面（MotionEvent→
//! TouchPhase/触点、Key→-1..-7）、呈现 blit（ARGB→RGBA8888 信箱）、生命周期
//! APP_CMD_*→Notify。全部纯函数（cfg(android) FFI 之上的可测层）。

use game_android::native::{
    blit_argb_rgba, hw_key_from_android, key_actions_from_android, lifecycle_notify,
    motion_phase, motion_pointer_index, APP_CMD_PAUSE, APP_CMD_RESUME, APP_CMD_START,
    AKEYCODE_BACK, AKEYCODE_DPAD_CENTER, AKEYCODE_DPAD_DOWN, AKEYCODE_DPAD_LEFT,
    AKEYCODE_DPAD_RIGHT, AKEYCODE_DPAD_UP, AKEYCODE_ENTER, AKEYCODE_MENU, AKEYCODE_SPACE,
    AKEYCODE_VOLUME_DOWN, AKEYCODE_VOLUME_UP, AMOTION_EVENT_ACTION_CANCEL,
    AMOTION_EVENT_ACTION_DOWN, AMOTION_EVENT_ACTION_MOVE, AMOTION_EVENT_ACTION_POINTER_DOWN,
    AMOTION_EVENT_ACTION_POINTER_UP, AMOTION_EVENT_ACTION_UP,
};
use game_android::input::{HwKey, KeyAction, TouchPhase};
use game_android::native::{CANVAS_H, CANVAS_W};
use game_android::lifecycle::{Notify, Visibility};

// ───────────────────────────────────────── MotionEvent 解析

#[test]
fn motion_phase_maps_android_actions() {
    assert_eq!(motion_phase(AMOTION_EVENT_ACTION_DOWN), Some(TouchPhase::Down));
    assert_eq!(motion_phase(AMOTION_EVENT_ACTION_UP), Some(TouchPhase::Up));
    assert_eq!(motion_phase(AMOTION_EVENT_ACTION_MOVE), Some(TouchPhase::Move));
    assert_eq!(motion_phase(AMOTION_EVENT_ACTION_CANCEL), Some(TouchPhase::Cancel));
    assert_eq!(motion_phase(AMOTION_EVENT_ACTION_POINTER_DOWN), Some(TouchPhase::Down));
    assert_eq!(motion_phase(AMOTION_EVENT_ACTION_POINTER_UP), Some(TouchPhase::Up));
    assert_eq!(motion_phase(4), None, "ACTION_OUTSIDE 不入触点管线");
}

#[test]
fn motion_pointer_index_from_action_high_bits() {
    assert_eq!(motion_pointer_index(0), 0);
    // POINTER_DOWN/UP：pointer index 在 (action>>8)&0xff
    assert_eq!(motion_pointer_index((2 << 8) | AMOTION_EVENT_ACTION_POINTER_DOWN), 2);
    assert_eq!(motion_pointer_index((1 << 8) | AMOTION_EVENT_ACTION_POINTER_UP), 1);
    assert_eq!(motion_pointer_index(AMOTION_EVENT_ACTION_MOVE), 0);
}

// ───────────────────────────────────────── KeyEvent 解析（-1..-7 键码契约）

#[test]
fn hw_key_from_android_codes() {
    assert_eq!(hw_key_from_android(AKEYCODE_BACK), HwKey::Back);
    assert_eq!(hw_key_from_android(AKEYCODE_DPAD_UP), HwKey::DpadUp);
    assert_eq!(hw_key_from_android(AKEYCODE_DPAD_DOWN), HwKey::DpadDown);
    assert_eq!(hw_key_from_android(AKEYCODE_DPAD_LEFT), HwKey::DpadLeft);
    assert_eq!(hw_key_from_android(AKEYCODE_DPAD_RIGHT), HwKey::DpadRight);
    assert_eq!(hw_key_from_android(AKEYCODE_DPAD_CENTER), HwKey::DpadCenter);
    assert_eq!(hw_key_from_android(AKEYCODE_ENTER), HwKey::Enter);
    assert_eq!(hw_key_from_android(AKEYCODE_SPACE), HwKey::Space);
    assert_eq!(hw_key_from_android(AKEYCODE_MENU), HwKey::Menu);
    assert_eq!(hw_key_from_android(AKEYCODE_VOLUME_UP), HwKey::VolumeUp);
    assert_eq!(hw_key_from_android(AKEYCODE_VOLUME_DOWN), HwKey::VolumeDown);
    assert_eq!(hw_key_from_android(1000), HwKey::Other);
}

#[test]
fn key_press_maps_to_game_codes() {
    // Key→-1..-7（key-code-contract §4）
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_UP, 0, 0), (vec![KeyAction::Press(-1)], true));
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_DOWN, 0, 0), (vec![KeyAction::Press(-2)], true));
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_LEFT, 0, 0), (vec![KeyAction::Press(-3)], true));
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_RIGHT, 0, 0), (vec![KeyAction::Press(-4)], true));
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_CENTER, 0, 0), (vec![KeyAction::Press(-5)], true));
    assert_eq!(key_actions_from_android(AKEYCODE_MENU, 0, 0), (vec![KeyAction::Press(-6)], true));
    assert_eq!(key_actions_from_android(AKEYCODE_BACK, 0, 0), (vec![KeyAction::Press(-7)], true));
}

#[test]
fn key_release_and_repeat_semantics() {
    // UP → Release（不带键）；repeat 丢弃（不投递）但消费性不变
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_UP, 1, 0), (vec![KeyAction::Release], true));
    assert_eq!(key_actions_from_android(AKEYCODE_DPAD_UP, 0, 3), (vec![], true), "OS 重复丢弃");
    // BACK 恒吞（BUG-020：含 repeat 与 UP 都不得走系统 finish）
    assert_eq!(key_actions_from_android(AKEYCODE_BACK, 0, 2), (vec![], true), "BACK repeat 仍吞");
    assert_eq!(key_actions_from_android(AKEYCODE_BACK, 1, 0), (vec![KeyAction::Release], true), "BACK UP 仍吞");
    // 音量/未知：不投递不吞（交系统）
    assert_eq!(key_actions_from_android(AKEYCODE_VOLUME_UP, 0, 0), (vec![], false));
    assert_eq!(key_actions_from_android(1000, 0, 0), (vec![], false));
}

// ───────────────────────────────────────── 生命周期 cmd → Notify

#[test]
fn lifecycle_cmd_dispatch() {
    let mut vis = Visibility::new();
    assert_eq!(lifecycle_notify(&mut vis, APP_CMD_PAUSE, false), Some(Notify::Hide));
    assert_eq!(lifecycle_notify(&mut vis, APP_CMD_PAUSE, false), None, "latch 不重发");
    assert_eq!(lifecycle_notify(&mut vis, APP_CMD_RESUME, false), Some(Notify::Show));
    assert_eq!(lifecycle_notify(&mut vis, APP_CMD_RESUME, false), None);
    // PiP=可见非遮挡不发 Hide（lifecycle.rs S1）
    let mut v2 = Visibility::new();
    assert_eq!(lifecycle_notify(&mut v2, APP_CMD_PAUSE, true), None);
    assert!(v2.shown(), "PiP pause 不动 latch");
    // 其余 cmd 无边沿
    assert_eq!(lifecycle_notify(&mut v2, APP_CMD_START, false), None);
    assert_eq!(lifecycle_notify(&mut v2, 1, false), None, "INIT_WINDOW 无 Notify");
}

// ───────────────────────────────────────── 呈现 blit（ARGB→RGBA8888 信箱）

#[test]
fn blit_argb_to_rgba8888_byte_order() {
    // 240×320 画布直拷进等大窗口：逐像素 [R,G,B,A]（WINDOW_FORMAT_RGBA_8888）
    let argb = [0xFF112233u32, 0x80445566]; // 0xAARRGGBB
    let mut dst = vec![0u8; 2 * 4];
    let r = blit_argb_rgba(&mut dst, 2, 1, 2, &argb, 2, 1, 1);
    assert_eq!(r, (0, 0, 2, 1), "1:1 直拷无信箱");
    assert_eq!(&dst[0..4], &[0x11, 0x22, 0x33, 0xFF], "R,G,B,A 序");
    assert_eq!(&dst[4..8], &[0x44, 0x55, 0x66, 0x80], "alpha 保真");
}

#[test]
fn blit_letterbox_on_larger_surface() {
    // 4×2 窗口装 2×2 画布：居中信箱（宽 2 高 2 ⇒ 左右各补 1 列 0）
    let argb = [0xFFFFFFFFu32, 0xFF000000, 0xFF00FF00, 0xFFFF0000];
    let mut dst = vec![0xAAu8; 4 * 2 * 4];
    let r = blit_argb_rgba(&mut dst, 4, 2, 4, &argb, 2, 2, 1);
    assert_eq!(r, (1, 0, 2, 2), "居中信箱 rect（x=1,w=2）");
    // 左补列=0（dst 先清）
    assert_eq!(&dst[0..4], &[0, 0, 0, 0xFF], "信箱区=不透黑（复核A 黄2：防截图透桌面）");
    assert_eq!(&dst[4..8], &[0xFF, 0xFF, 0xFF, 0xFF], "画布首像素落 (1,0)");
}

#[test]
fn blit_scales_up_by_nearest_when_window_larger_in_both_axes() {
    // 4×4 窗口装 2×2 画布：等比放大 ×2（信箱消隐）——像素块复制
    let argb = [0xFF112233u32, 0xFF445566, 0xFF778899, 0xFFAABBCC];
    let mut dst = vec![0u8; 4 * 4 * 4];
    let r = blit_argb_rgba(&mut dst, 4, 4, 4, &argb, 2, 2, 2);
    assert_eq!(r, (0, 0, 4, 4), "放大满窗");
    // (0,0) 与 (1,0) 同源 (0,0)
    assert_eq!(&dst[0..4], &[0x11, 0x22, 0x33, 0xFF]);
    assert_eq!(&dst[4..8], &[0x11, 0x22, 0x33, 0xFF], "最近邻 ×2 块复制");
    // (0,2) 源 (0,1)
    assert_eq!(&dst[2 * 4 * 4..2 * 4 * 4 + 4], &[0x77, 0x88, 0x99, 0xFF], "第二行源 (0,1)");
}

#[test]
fn blit_rect_reported_for_painter() {
    // 返回矩形=呈现脏区（unlockAndPost 前的 lock 缓冲换算面）
    let mut dst = vec![0u8; 3 * 3 * 4];
    let argb = vec![0xFF000000u32; 9];
    let r = blit_argb_rgba(&mut dst, 3, 3, 3, &argb, 3, 3, 1);
    assert_eq!(r, (0, 0, 3, 3));
}

// ───────────────────────────────────────── 窗口未就绪 No-op 钉（实机 SIGSEGV 回派）

#[test]
fn present_null_window_noop() {
    // 实机崩因钉（SIGSEGV setBuffersGeometry 空窗口）：window=NULL ⇒ present 必须
    // No-op（不触 FFI），返回未呈现。
    let mut w = game_android::native::ShellWiring::new();
    let argb = vec![0u32; CANVAS_W as usize * CANVAS_H as usize];
    let null_win: *mut game_android::native::ANativeWindow = std::ptr::null_mut();
    assert!(!w.present(null_win, &argb), "window 未就绪=No-op 且报告未呈现");
}

#[test]
fn surface_negotiate_null_window_noop() {
    // 几何协商同判：APP_CMD_INIT_WINDOW 处理里也只在 window 非空时协商
    let mut w = game_android::native::ShellWiring::new();
    let null_win: *mut game_android::native::ANativeWindow = std::ptr::null_mut();
    assert!(!w.ensure_surface(null_win), "NULL window 不协商（不调 setBuffersGeometry）");
}

// ───────────────────────────────────────── Bug1 stride 行距钉（实机碎裂回派）

#[test]
fn blit_row_pitch_uses_stride_not_width() {
    // 实机 Bug1（字形碎裂每行左移累积）：lock 缓冲 stride(256)≠width(240)——
    // 逐行按 stride 跳行，禁按 width 连写。
    let (w, h, stride) = (240u32, 2u32, 256u32);
    let mut dst = vec![0u8; (stride * h * 4) as usize];
    let argb = vec![0xFF112233u32; (w * h) as usize];
    let r = blit_argb_rgba(&mut dst, w, h, stride, &argb, w, h, 1);
    assert_eq!(r, (0, 0, w, h), "1:1 直拷（宽度域）");
    // 行 0 首像素在 0；行 1 首像素在 stride*4（不是 width*4）
    assert_eq!(&dst[0..4], &[0x11, 0x22, 0x33, 0xFF], "行 0");
    assert_eq!(&dst[(stride * 4) as usize..(stride * 4) as usize + 4], &[0x11, 0x22, 0x33, 0xFF], "行 1 落 stride 行距");
    assert_eq!(
        &dst[(w * 4) as usize..(w * 4) as usize + 4],
        &[0, 0, 0, 0xFF],
        "width*4 处=行 0 尾隙（不透黑），不得错位连写"
    );
}

// ───────────────────────────────────────── Bug2 信箱几何钉（实机压扁回派·#2 规格）

#[test]
fn landscape_matrix_game_rects() {
    // 【2026-10-23 更正】Bug2 信箱钉数字随 #2 显式规格修正：诊断括注"0.75 画幅"
    // 系宽高比对比非尺寸规则；lead #2 给出显式几何（18 格=576@2x=1152、侧余 759px
    // 放触控）——固定 SCALE 倍数居中方画幅+侧余触控区。四档分辨率矩阵矩形：
    let buf = (2670u32, 1200u32); // Xiaomi 14 横屏
    // A) 13 格=416@2x=832
    assert_eq!(game_android::native::fit_rect(buf.0, buf.1, 416, 416, 2), (919, 184, 832, 832));
    // B) 18 格=576@2x=1152（贴高，侧余 759px——lead 候选 B）
    assert_eq!(game_android::native::fit_rect(buf.0, buf.1, 576, 576, 2), (759, 24, 1152, 1152));
    // C) 15 格=480@2x=960
    assert_eq!(game_android::native::fit_rect(buf.0, buf.1, 480, 480, 2), (855, 120, 960, 960));
    // D) 37 格=1184@1x=1184（最大视野）
    assert_eq!(game_android::native::fit_rect(buf.0, buf.1, 1184, 1184, 1), (743, 8, 1184, 1184));
}

#[test]
fn fit_rect_overflow_shrinks_proportionally() {
    // 游戏区越界=等比缩进（分数倍最近邻）：2670×1200 @ 37 格@2x=2368 超高
    let r = game_android::native::fit_rect(2670, 1200, 1184, 1184, 2);
    assert_eq!(r, (735, 0, 1200, 1200), "高限 1200 ⇒ 等比缩进 1200×1200 居中");
}

#[test]
fn touch_inverse_maps_through_game_rect() {
    // 触点反变换随格数/缩放自适配（#2）：buf 坐标→游戏区→逻辑画布坐标
    // 2670×1200 横屏 B 档（576@2x=1152，x0=759,y0=24）：点 (759,24)→逻辑 (0,0)；
    // 点 (759+1151, 24+1151)→逻辑 (575,575)
    let (lx, ly) = game_android::native::map_touch_xy(759.0, 24.0, 2670, 1200, 18, 2);
    assert_eq!((lx.round() as i32, ly.round() as i32), (0, 0), "游戏区左上=逻辑原点");
    let (hx, hy) = game_android::native::map_touch_xy(759.0 + 1151.0, 24.0 + 1151.0, 2670, 1200, 18, 2);
    assert_eq!((hx.round() as i32, hy.round() as i32), (575, 575), "游戏区右下=逻辑 (575,575)");
    // 触控侧余区（759px 内）：逻辑 x 为负（区外=触控键区，不入格）
    let (ox, _) = game_android::native::map_touch_xy(100.0, 600.0, 2670, 1200, 18, 2);
    assert!(ox < 0.0, "侧余触控区落在画布外（负坐标）");
}

// ───────────────────────────────────────── #2 动态分辨率选择策略（假分辨率表）

#[test]
fn pick_resolution_matrix() {
    // 用户批准的三步策略：①scale=最大整数倍(1x/2x/3x)使方画幅≤min(屏宽,屏高)
    // ②grid=floor(边长/32/scale) clamp [13,24] ③触控区=横屏侧区自适应
    use game_android::native::pick_resolution;
    assert_eq!(pick_resolution(2670, 1200), (18, 2), "Xiaomi 14 横屏：scale2/grid18（1200/32/2=18.75）");
    assert_eq!(pick_resolution(240, 320), (13, 1), "小屏 clamp 下限 13（原视野起手）");
    assert_eq!(pick_resolution(3840, 2400), (24, 3), "巨屏 clamp 上限 24（防全图失衡）");
    assert_eq!(pick_resolution(1280, 720), (22, 1), "720p：3x/2x 都不足 13 ⇒ 1x/grid22");
}

#[test]
fn pick_resolution_prefers_largest_scale() {
    // ①最大整数倍优先：3x 可用则 3x（如 2560×1440：1440/96=15 ≥13 ⇒ (15,3)）
    use game_android::native::pick_resolution;
    assert_eq!(pick_resolution(2560, 1440), (15, 3));
    // 2x/1x 均可时选 2x（2670×1200 已证）；1x 仅在 3x/2x 都 <13 时
    assert_eq!(pick_resolution(900, 600), (18, 1), "600/32/1=18.75→18；600/64=9<13");
}

// ───────────────────────────────────────── 复核B 三红钉（侧余对称/出口钳制/Modern 布局）

#[test]
fn touch_margins_negative_all_four_sides() {
    // 【R1】侧余负坐标四向对称（旧右/下返正越界 870）——UNDO/触控件按符号判区
    use game_android::native::map_touch_xy;
    let (lx, _) = map_touch_xy(2500.0, 600.0, 2670, 1200, 18, 2);
    assert!(lx < 0.0, "右侧余=负（旧 870 正越界）");
    let (_, ty) = map_touch_xy(1000.0, 1190.0, 2670, 1200, 18, 2);
    assert!(ty < 0.0, "下侧余=负");
    let (a, b) = map_touch_xy(0.0, 0.0, 2670, 1200, 18, 2);
    assert!(a < 0.0 && b < 0.0, "左上角=双负");
    let (c, d) = map_touch_xy(2669.0, 1199.0, 2670, 1200, 18, 2);
    assert!(c < 0.0 && d < 0.0, "右下角=双负");
    // 四角+24 格（黄：断言扩 24 格——24格@1x=768 画幅四边有信箱，取角=双负；
    // 24格@2x=缩进档上下贴边属区内，另在缩进档测断言）
    let (e, f) = map_touch_xy(2669.0, 1199.0, 2670, 1200, 24, 1);
    assert!(e < 0.0 && f < 0.0, "24 格右下角=双负");
    let (e2, f2) = map_touch_xy(0.0, 0.0, 2670, 1200, 24, 1);
    assert!(e2 < 0.0 && f2 < 0.0, "24 格左上角=双负");
    // 游戏区内=非负有界
    let (okx, oky) = map_touch_xy(759.0 + 576.0, 24.0 + 576.0, 2670, 1200, 18, 2);
    assert!((0.0..576.0).contains(&okx) && (0.0..576.0).contains(&oky), "区内=[0,575]");
}

#[test]
fn touch_edge_clamped_in_shrink_case() {
    // 【R2】缩进档出口钳制：g13s1（buf 240×320、画幅 416）——旧 f64 截断 w=239
    // ⇒ 点 (239,279) 映到 (416,416) 越一格；整数化+钳制后 ≤ 415。
    use game_android::native::{fit_rect, map_touch_xy};
    assert_eq!(fit_rect(240, 320, 416, 416, 1), (0, 40, 240, 240), "整数化 w=240（非 239）");
    let (x, y) = map_touch_xy(239.0, 279.0, 240, 320, 13, 1);
    assert!((0.0..416.0).contains(&x) && (0.0..416.0).contains(&y), "边缘触点钳制<grid*32（得 {x},{y}）");
    // 缩进档四角（黄）：x0=0（宽贴齐）⇒左缘=区内 0；上侧余（y<40）=负
    let (a, b) = map_touch_xy(0.0, 0.0, 240, 320, 13, 1);
    assert_eq!(a, 0.0, "左缘贴齐=逻辑 x=0");
    assert!(b < 0.0, "上侧余=负 y");
    let (_, by) = map_touch_xy(120.0, 39.0, 240, 320, 13, 1);
    assert!(by < 0.0, "上侧余（y<40）=负");
}

#[test]
fn pad_layout_modern_tracks_grid() {
    // 【R3】Modern 触控件随格数贴底/贴右（对齐 layout.rs modern softkey_base_y=边长-18）
    use game_android::input::PadLayout;
    let p18 = PadLayout::for_modern(18); // 576
    assert_eq!((p18.soft[2].x0, p18.soft[2].x1), (512.0, 568.0), "右键贴右 [w-64,w-8)");
    assert_eq!((p18.soft[2].y0, p18.soft[2].y1), (476.0, 540.0), "右键贴底 [h-100,h-36)");
    assert_eq!(p18.dpad.cy, 508.0, "方向盘贴底 h-68");
    let p24 = PadLayout::for_modern(24); // 768（黄：扩 24 格）
    assert_eq!((p24.soft[2].x0, p24.soft[2].y0), (704.0, 668.0), "24 格贴右/贴底");
    assert_eq!(p24.dpad.cy, 700.0);
    // layout.rs modern 贴底锚互证（24 格钉）
    let l24 = game_core::layout::Layout::modern(24, 24);
    assert_eq!(l24.softkey_base_y, 750, "softkey_base_y=768-18（24 格）");
    let l18 = game_core::layout::Layout::modern(18, 18);
    assert_eq!(l18.softkey_base_y, 558, "576-18（18 格）");
}

// ───────────────────────────────────────── 实机双问题钉（问题1 铺满/问题2 全链）

#[test]
fn title_fill_rect_covers_canvas() {
    // 【问题1】576²/832² 画布下标题图绘制尺寸 ≥ 画布边长（cover 撑满裁切）
    use game_core::paint::title_fill_rect;
    for side in [576i32, 832] {
        let (x, y, w, h) = title_fill_rect(side, side, 240, 320);
        assert!(w >= side && h >= side, "{side}² 画布：标题图 {w}×{h} 须满幅（≥边长）");
        assert!(x <= 0 && y <= 0, "cover 居中裁切（溢出为负偏移）");
    }
    // classic 240×320 画布 240×320 图：1:1 满幅
    let (_, _, w, h) = title_fill_rect(240, 320, 240, 320);
    assert_eq!((w, h), (240, 320));
}

#[test]
fn tap_chain_buf_to_walk_to() {
    // 【问题2d】全链：buf 坐标→map_touch_xy→classify_tap→WalkTo(tile)
    use game_android::native::map_touch_xy;
    use game_shell_common::touch::{classify_tap, TapAction, TouchGeom};
    let geom = TouchGeom { cam_x: 0, cam_y: 0, tile_off: 0, cols: 20, rows: 20 };
    // 2670×1200 B 档：点游戏方框内 (1335,600)（原实机"开始游戏"带）→ 逻辑格
    let (lx, ly) = map_touch_xy(1335.0, 600.0, 2670, 1200, 18, 2);
    assert!((0.0..576.0).contains(&lx) && (0.0..576.0).contains(&ly), "落画布内（2b 修后）");
    let act = classify_tap(lx as i32, ly as i32, &geom, 576, 576, (0, 0), |_, _| true);
    assert_eq!(act, TapAction::WalkTo(((lx as i32) / 16, (ly as i32) / 16)), "点格→WalkTo 全链");
}

#[test]
fn title_touch_routes_tap_and_swipe_through_wiring() {
    // 【C 方案 2026-10-26】标题态触点只走轮播手势（点按/水平滑动），**不喂**虚拟键区
    // （方向盘捕获圆 (56,508) r=60 与"上一项"热区重叠——旧路由先喂 pad 会把左下点按
    // 译成方向键=语义错位）。
    // > 【2026-10-26 更正】本钉取代 `menu_hit_rows_and_gaps`（2c 六行隐形横条几何钉：
    // > 行不可见、实机 (350,542) 落第 6 行=退出误触，已随 C 方案撤除）。
    use game_android::input::TouchEvent;
    use game_android::native::{ShellWiring, TitleGesture};
    use game_core::title::TitleTouch;
    let mut w = ShellWiring::new();
    w.pad = game_android::input::PadLayout::for_modern(18);
    w.touch = game_android::input::TouchMapper::new(w.pad.clone());
    w.title_mode = true;
    let ev = |id, phase, x, y| TouchEvent { id, phase, x, y };
    // 点按（方向盘区内）：零键、记 Tap（落点=按下点）
    assert!(w.on_touch_event(&ev(1, TouchPhase::Down, 60.0, 520.0)).is_empty(), "标题态不喂方向盘");
    assert!(w.on_touch_event(&ev(1, TouchPhase::Move, 64.0, 522.0)).is_empty(), "阈值内抖动≠滑动");
    assert!(w.on_touch_event(&ev(1, TouchPhase::Up, 64.0, 522.0)).is_empty());
    assert_eq!(std::mem::take(&mut w.title_gestures), vec![TitleGesture::Tap(60.0, 520.0)]);
    // 左滑=下一项（一手势一次），抬手不再记 Tap
    w.on_touch_event(&ev(2, TouchPhase::Down, 300.0, 300.0));
    w.on_touch_event(&ev(2, TouchPhase::Move, 250.0, 305.0));
    w.on_touch_event(&ev(2, TouchPhase::Move, 150.0, 305.0));
    w.on_touch_event(&ev(2, TouchPhase::Up, 150.0, 305.0));
    assert_eq!(std::mem::take(&mut w.title_gestures), vec![TitleGesture::Swipe(TitleTouch::Next)]);
    // 非标题态：照旧喂虚拟键区（软键 -5 中心）
    w.title_mode = false;
    let keys = w.on_touch_event(&ev(3, TouchPhase::Down, 476.0, 526.0));
    assert_eq!(keys, vec![KeyAction::Press(-5)], "UI/游戏态虚拟键区不受影响");
    assert!(w.title_gestures.is_empty());
}

// ───────────────────────────────────────── 双件钉（件1 清屏/件2 同源框）

#[test]
fn modern_paint_no_residual_pixels() {
    // 【件1】modern 画幅清屏重绘：脏画布（DEADBEEF 全铺）paint 后零残留
    //（对拍 title→menu 切换帧——继续游戏三景错叠根治）
    use game_core::paint::BootMachine;
    use game_core::render::{ArgbImage, SoftGraphics};
    let img = || ArgbImage::create(2, 2);
    let vec = |n: usize| (0..n).map(|_| img()).collect::<Vec<_>>();
    let mut m = BootMachine::new(img(), img(), vec(8), vec(25), vec(2), img(), vec(2), vec(1), vec(2));
    m.apply_layout(game_core::layout::Layout::modern(18, 18));
    for _ in 0..200 {
        m.tick(0); // 走 boot 时间线（含状态迁移）
    }
    let mut canvas = ArgbImage::create(576, 576);
    for px in canvas.argb.iter_mut() {
        *px = 0xDEADBEEF; // 脏画布=标题美术残留面
    }
    {
        let mut g = SoftGraphics::new(&mut canvas);
        m.paint(&mut g);
    }
    let residue = canvas.argb.iter().filter(|p| **p == 0xDEADBEEF).count();
    assert_eq!(residue, 0, "modern 画幅零残留（清屏重绘）：残留 {residue} px");
}

#[test]
fn overlay_rects_match_hit_tables() {
    // 【件2】框坐标==命中表坐标同源断言（防框和感应区脱节=二次盲区）
    // > 【2026-10-26 更正】按态分：标题态=轮播三热区（TitleGeom 单源，pad 不接管）；
    // > 原"6 菜单行框"随 2c 六行横条撤除（C 方案）。
    use game_android::native::{overlay_rects, OverlayState};
    let pad = game_android::input::PadLayout::for_modern(18);
    let geo = game_core::title::TitleGeom::modern(&game_core::layout::Layout::modern(18, 18), 240, 295, 20, 13);
    let title_rects = overlay_rects(18, &pad, OverlayState::Title(geo));
    let zones: Vec<_> = title_rects.iter().map(|r| (r.0, r.1, r.2, r.3)).collect();
    assert_eq!(zones, vec![geo.zone_prev, geo.zone_ok, geo.zone_next], "标题态框==热区（单源逐元）");
    let rects = overlay_rects(18, &pad, OverlayState::Game);
    // UNDO 框中心=undo_zone_hit 命中（同源）
    let undo = rects.iter().find(|r| r.4 == "undo").expect("UNDO 框在");
    let (cx, cy) = (undo.0 + undo.2 / 2, undo.1 + undo.3 / 2);
    assert!(
        game_shell_common::touch::undo_zone_hit(cx, cy, 576, 576),
        "UNDO 框中心=感应区命中（同源）"
    );
    // 虚拟键区==wiring.pad 同源（黄②：消费而非重建）
    let rects2 = overlay_rects(18, &pad, OverlayState::Game);
    for z in &pad.soft {
        assert!(
            rects2.iter().any(|r| r.4 == "soft"
                && (r.0, r.1, r.2, r.3) == (z.x0 as i32, z.y0 as i32, (z.x1 - z.x0) as i32, (z.y1 - z.y0) as i32)),
            "软键框==PadLayout 单源"
        );
    }
    // 【红-1】dpad 边界探针：45px 出键且框内=同源（旧 60×60 框=出键在框外）
    let dpad = rects.iter().find(|r| r.4 == "dpad").expect("dpad 框在");
    let (cx, cy) = (pad.dpad.cx as i32, pad.dpad.cy as i32);
    for (dx, dy) in [(45, 0), (-45, 0), (0, 45), (0, -45)] {
        assert!(game_android::input::in_dpad_capture(&pad.dpad, (cx + dx) as f32, (cy + dy) as f32), "45px 探针=捕获圆内出键（绝对坐标域）");
        let (px, py) = (cx + dx, cy + dy);
        assert!(
            px >= dpad.0 && px < dpad.0 + dpad.2 && py >= dpad.1 && py < dpad.1 + dpad.3,
            "45px 探针=框内（同源）：({px},{py}) vs {dpad:?}"
        );
    }
    // 地图区（黄③）：游戏态覆盖 tap→walk 响应面
    assert!(rects.iter().any(|r| r.4 == "map"), "游戏态地图区框在");
    // UI 态不画地图区/UNDO（UNDO 仅游戏态生效）
    let ui_rects = overlay_rects(18, &pad, OverlayState::Ui);
    assert!(!ui_rects.iter().any(|r| r.4 == "map" || r.4 == "undo"), "UI 态无地图区/UNDO 框");
    assert!(ui_rects.iter().any(|r| r.4 == "dpad"), "UI 态虚拟键区照显");
}

// ─────────────────────────── 【2026-10-23 复原】复核C 硬发现①误删两测 ──
// d80d96e 以"去重"名义误删（splice 锚序倒置的截断误伤，非重复断言合并）：
// 按 lead 口径 `git show 50081d8` 原文复原，**内容逐字不变**；menu_hit 钉
// （2c 裁定）并存=覆盖只增不减（删一补一零净减成立面）。

#[test]
fn title_button_hit_diff_smoke() {
    // 【问题2c】标题按钮命中面：BootMachine::pointer_hit 底部中带 vs 远角=不同命中
    use game_core::paint::BootMachine;
    use game_core::render::ArgbImage;
    let img = || ArgbImage::create(2, 2);
    let vec = |n: usize| (0..n).map(|_| img()).collect::<Vec<_>>();
    let mut m = BootMachine::new(img(), img(), vec(8), vec(25), vec(2), img(), vec(2), vec(1), vec(2));
    m.apply_layout(game_core::layout::Layout::modern(18, 18));
    let band = m.pointer_hit(288, 540); // 底部中带（开始游戏带）
    let corner = m.pointer_hit(2, 2);
    let _ = (band, corner); // 命中面可达（Debug 面随设备日志 jjxy-input 观测）
}

#[test]
fn title_cover_bottom_keeps_logo_visible() {
    // 【截断回派】576² 画布底锚 cover：底部 logo 区非透明（裁上部天空保 logo）
    use game_core::paint::title_fill_rect_bottom;
    use game_core::render::{ArgbImage, SoftGraphics};
    let mut img = ArgbImage::create(240, 320);
    for y in 0..320i32 {
        for x in 0..240i32 {
            let c = if y >= 300 { 0xFFFF0000u32 } else { 0xFF0000FF };
            img.argb[(y * 240 + x) as usize] = c;
        }
    }
    let mut canvas = ArgbImage::create(576, 576);
    {
        let mut g = SoftGraphics::new(&mut canvas);
        g.draw_image_cover(&img, 576, 576, true);
    }
    assert_eq!(canvas.argb[575 * 576], 0xFFFF0000, "画布底行=源底 logo 色（非透明未截）");
    let r = title_fill_rect_bottom(576, 576, 240, 320);
    assert_eq!(r.1 + r.3, 576, "底锚 y+h=画布底（裁上部天空）");
}

#[test]
fn title_gesture_chain_drives_cursor_and_confirm() {
    // 【C 方案链路钉】手势→键→title tick 既有臂→光标/模式全链（壳解析面
    // resolve_title_gestures + BootMachine 单源几何）。
    // > 【2026-10-26 更正】本钉取代 `tap_row_chain_cursor_confirm_mode`（2c 六行
    // > 横条链路钉，随六行撤除）。
    use game_android::native::{resolve_title_gestures, TitleGesture};
    use game_core::enums::GameMode;
    use game_core::paint::BootMachine;
    use game_core::render::ArgbImage;
    use game_core::title::TitleTouch;
    let img = |w, h| ArgbImage::create(w, h);
    let mut ui: Vec<_> = (0..25).map(|_| img(2, 2)).collect();
    ui[14] = img(20, 13);
    let v = |n: usize| (0..n).map(|_| img(2, 2)).collect::<Vec<_>>();
    let mut m = BootMachine::new(img(2, 2), img(2, 2), v(8), ui, vec![img(240, 295), img(68, 102)], img(2, 2), v(2), v(1), v(2));
    m.apply_layout(game_core::layout::Layout::modern(18, 18));
    for _ in 0..200 {
        m.tick(0);
        if m.mode == GameMode::MainMenu {
            break;
        }
    }
    let g = m.title_geom().expect("标题态");
    let mid = |r: (i32, i32, i32, i32)| ((r.0 + r.2 / 2) as f32, (r.1 + r.3 / 2) as f32);
    let press = |m: &mut BootMachine, gs: Vec<TitleGesture>| {
        let keys = resolve_title_gestures(m, gs);
        m.tick(game_android::native::boot_key_from_batch(&keys));
    };
    let (nx, ny) = mid(g.zone_next);
    press(&mut m, vec![TitleGesture::Tap(nx, ny)]);
    assert_eq!(m.title_cursor(), Some(1), "点下一项热区");
    press(&mut m, vec![TitleGesture::Swipe(TitleTouch::Prev)]);
    assert_eq!(m.title_cursor(), Some(0), "右滑=上一项");
    press(&mut m, vec![TitleGesture::Tap(10.0, 10.0)]);
    assert_eq!(m.title_cursor(), Some(0), "美术区点按=无键");
    let (ox, oy) = mid(g.zone_ok);
    press(&mut m, vec![TitleGesture::Tap(ox, oy)]);
    assert_eq!(m.mode, GameMode::IntroCutscene, "点标签=确认（kind0 开始游戏→序章）");
    // 离开标题态后残留手势一律丢弃（滑动不得在序章合成键）
    assert!(resolve_title_gestures(&m, vec![TitleGesture::Swipe(TitleTouch::Next), TitleGesture::Tap(ox, oy)]).is_empty());
}

#[test]
fn boot_key_latch_keeps_tap_press_across_release() {
    // 【2026-10-26 实机根因】同批 [Press(-5), Release]（点按/长按抬手后首拍）旧 fold
    // "取末动作"=0 ⇒ 按键被吞（设备 DIAG：batch=[Press(-5), Release] 后标题不动）。
    // 对齐 win 壳 route_key：BootMachine 只读 keyValue ⇒ Press 覆盖式锁存（末个按下胜），
    // Release 对 BootMachine 无效果（shell/win/src/main.rs:654-658 pending_machine_key）。
    use game_android::native::boot_key_from_batch;
    assert_eq!(boot_key_from_batch(&[KeyAction::Press(-5), KeyAction::Release]), -5, "点按=按下不被抬起抹掉");
    assert_eq!(
        boot_key_from_batch(&[KeyAction::Press(-3), KeyAction::Release, KeyAction::Press(-4), KeyAction::Release]),
        -4,
        "同拍多次按下=覆盖式取末个按下"
    );
    assert_eq!(boot_key_from_batch(&[KeyAction::Release]), 0, "纯抬起=无键");
    assert_eq!(boot_key_from_batch(&[]), 0, "空批=无键");
}

// ─────────────────────────── 【2026-10-26 实机根因②】主循环饿死 ──
// 设备 DIAG 实录：按住软键 1000ms（119 个 Move 事件）期间逻辑拍断档 1053ms——
// 内层 `while ALooper_pollAll(16) >= 0` 在 120Hz Move 流下恒返回 ≥0 不退出 ⇒
// tick/paint 整段饿死，抬手后 [Press, Release] 同批（叠加旧 fold ⇒ 全吞）。
// 钉：虚拟时钟假 looper（事件按时刻到达），驱动生产同一 pump_once。

use game_android::pump::{pump_once, FramePacer, Pump};

/// 虚拟时钟 looper：poll(timeout) = 截止 now+timeout 内有事件则推进到事件时刻并消费。
struct FakeLooper {
    now: u64,
    events: std::collections::VecDeque<u64>,
}

impl Pump for FakeLooper {
    fn now_ms(&self) -> u64 {
        self.now
    }
    fn poll(&mut self, timeout_ms: i32) -> bool {
        let deadline = self.now + timeout_ms.max(0) as u64;
        match self.events.front().copied() {
            Some(t) if t <= deadline => {
                self.now = self.now.max(t);
                self.events.pop_front();
                true
            }
            _ => {
                self.now = deadline;
                false
            }
        }
    }
}

#[test]
fn pump_keeps_ticking_under_continuous_touch_moves() {
    // 100ms 起按住 1000ms：每 8ms 一个 Move（≈120Hz 触控采样）
    let events = (0..125u64).map(|i| 100 + i * 8).collect();
    let mut lp = FakeLooper { now: 0, events };
    let mut pacer = FramePacer::new(0);
    let mut ticks = Vec::new();
    while lp.now < 1500 {
        if pump_once(&mut lp, &mut pacer, 75) {
            ticks.push(lp.now);
            lp.now += 5; // 拍内工作（tick+paint+present）耗时
        }
        let wait = pacer.wait_ms(lp.now);
        lp.poll(wait);
    }
    let max_gap = ticks.windows(2).map(|w| w[1] - w[0]).max().unwrap_or(0);
    assert!(ticks.len() >= 18, "1500ms 内约 20 拍（75ms 节拍）：实得 {}", ticks.len());
    assert!(max_gap <= 90, "触摸流期间逻辑拍不得断档：最大间隔 {max_gap}ms（旧 poll(16) 循环≈1000ms）");
}

#[test]
fn pacer_sleeps_remainder_without_catch_up_burst() {
    // J2ME run 循环语义：拍起点+间隔=下一拍（睡余量，不追帧）
    let mut p = FramePacer::new(0);
    assert!(p.due(0, 75), "首拍立即到期");
    assert!(!p.due(74, 75), "未到间隔不出拍");
    assert_eq!(p.wait_ms(70), 5, "等待=距下一拍余量");
    assert!(p.due(75, 75));
    // 卡顿 300ms 后：只出一拍并重锚（不连发 4 拍追帧）
    assert!(p.due(375, 75), "卡顿后出一拍");
    assert!(!p.due(376, 75), "不追帧连发");
    assert_eq!(p.wait_ms(376), 74);
}

// ─────────────────────────── 【2026-10-26 字体修复】物理背板（字形清晰度） ──
// 旧：画布=逻辑 576²、SoftGraphics scale=1、呈现 NN×2 ⇒ 字形按 1× 光栅化再放大=
// 马赛克。新：背板=逻辑×scale 物理像素（win 壳 canvas_graphics 同口径，task-13 ②），
// 字形钩子按 scale×格高直落物理像素；呈现 1:1。

#[test]
fn physical_canvas_graphics_rasterizes_at_scale() {
    use game_android::native::canvas_graphics;
    use game_core::render::ArgbImage;
    let mut c = ArgbImage::create(2 * 576, 2 * 576);
    {
        let mut g = canvas_graphics(&mut c, 2);
        assert_eq!(g.canvas_logical(), (576, 576), "逻辑坐标面不变（game-core 只见布局坐标）");
        g.set_color(0xFF0000);
        g.fill_rect(575, 575, 1, 1); // 逻辑右下角 1px
    }
    let w = 1152usize;
    for (x, y) in [(1150, 1150), (1151, 1150), (1150, 1151), (1151, 1151)] {
        assert_eq!(c.argb[y * w + x], 0xFFFF0000, "逻辑 1px = 物理 2×2 块（{x},{y}）");
    }
    assert_eq!(c.argb[1149 * w + 1149] & 0xFF000000, 0, "块外未写");
}

#[test]
fn physical_canvas_present_rect_matches_logical_scale() {
    use game_android::native::fit_rect;
    // 物理背板 1152² 以 1:1 呈现 == 逻辑 576² 以 2× 呈现（信箱/触点反变换不变）
    assert_eq!(fit_rect(2536, 1200, 1152, 1152, 1), fit_rect(2536, 1200, 576, 576, 2));
    assert_eq!(fit_rect(2536, 1200, 1152, 1152, 1), (692, 24, 1152, 1152), "Xiaomi 14 实测窗 2536×1200");
}
