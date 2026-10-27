//! STAGE-B-005 游戏内退出确认框·壳宿主钉（runtime.rs = android_main 同一调度体）。
//! 用户 2026-10-27："游戏内右滑弹出退出确认菜单，样式就用游戏内弹框样式"（Q1 地图态 / Q2 确定即退出）。
//! 设计件 docs/superpowers/specs/2026-10-27-in-game-exit-confirm-design.md。

use game_android::input::KeyAction;
use game_android::native::{OverlayState, TitleGesture};
use game_android::runtime::{build_boot_machine, Runtime};
use game_core::enums::GameMode;
use game_core::layout::Layout;
use game_shell_common::touch::TouchGeom;

/// 设备同构双档布局（portrait.rs：开场链 7×10、游戏 12×16）。
fn runtime(tag: &str) -> (Runtime, game_shell_common::asset::DirAssetSource, std::path::PathBuf) {
    let raw = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw");
    let src = game_shell_common::asset::DirAssetSource::new(&raw);
    let dir = std::env::temp_dir().join(format!("jjxy-confirm-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (rms, stores) = game_shell_common::rms::FileRms::load(&dir);
    let machine = build_boot_machine(&src, Layout::modern(7, 10));
    let mut rt = Runtime::new(machine, stores, Box::new(rms));
    rt.apply_layouts(Layout::modern(7, 10), Layout::modern(12, 16));
    (rt, src, dir)
}

/// 一拍 = step + paint（设备主循环同构）。
fn drive(rt: &mut Runtime, src: &dyn game_platform::AssetSource, keys: &[KeyAction], taps: Vec<(f32, f32)>) {
    rt.step(src, keys, Vec::<TitleGesture>::new(), taps);
    let mut canvas = game_core::render::ArgbImage::create(576, 576);
    let mut g = game_android::native::canvas_graphics(&mut canvas, 1);
    rt.paint(&mut g);
}

fn press(rt: &mut Runtime, src: &dyn game_platform::AssetSource, code: i32) {
    drive(rt, src, &[KeyAction::Press(code), KeyAction::Release], Vec::new());
}

/// 新游戏 → 序章跳过 → 交接 → 剧情推完 → Map 稳定 30 拍。
fn to_map(rt: &mut Runtime, src: &dyn game_platform::AssetSource) {
    assert!(
        (0..400).any(|_| {
            drive(rt, src, &[], Vec::new());
            rt.title_mode()
        }),
        "开机到标题"
    );
    press(rt, src, -5);
    press(rt, src, -7);
    assert!(
        (0..400).any(|_| {
            drive(rt, src, &[], Vec::new());
            rt.scene.is_some()
        }),
        "序章跳过 → 读条 → 交接 GameScene"
    );
    let mut calm = 0;
    for i in 0..4000 {
        let mode = rt.scene.as_ref().unwrap().game_mode;
        calm = if mode == GameMode::Map { calm + 1 } else { 0 };
        if calm >= 30 {
            return;
        }
        let keys: &[KeyAction] =
            if mode != GameMode::Map && i % 4 == 0 { &[KeyAction::Press(-5), KeyAction::Release] } else { &[] };
        drive(rt, src, keys, Vec::new());
    }
    panic!("剧情未推完");
}

fn mode(rt: &Runtime) -> GameMode {
    rt.scene.as_ref().unwrap().game_mode
}

fn open(rt: &Runtime) -> bool {
    rt.scene.as_ref().unwrap().exit_confirm_open()
}

/// 地图态返回 = 开退出确认框（不进游戏内菜单）；框开期间面板软键 ○ / ×、触屏按 UI 态。
#[test]
fn back_on_map_opens_confirm_instead_of_menu() {
    let (mut rt, src, dir) = runtime("open");
    to_map(&mut rt, &src);
    assert!(rt.back_opens_exit_confirm(), "地图态返回 = 开退出确认框");
    for _ in 0..3 {
        drive(&mut rt, &src, &[], Vec::new());
    }
    assert!(open(&rt) && mode(&rt) == GameMode::Map, "框开、未进游戏内菜单");
    assert_eq!(rt.softkeys(), (1, 2), "面板软键 ○ / ×");
    assert!(!matches!(rt.overlay_state(), OverlayState::Game), "框开 ⇒ 触屏按 UI 态");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 面板右软键（键批 -7，不经 BACK 分支）在地图上照旧开游戏内菜单；菜单里返回不开框。
#[test]
fn panel_right_softkey_still_opens_in_game_menu() {
    let (mut rt, src, dir) = runtime("panel");
    to_map(&mut rt, &src);
    press(&mut rt, &src, -7);
    assert_eq!(mode(&rt), GameMode::InGameMenu, "右软键照旧开游戏内菜单");
    assert!(!rt.back_opens_exit_confirm(), "游戏内菜单里返回不开框（照旧 -7 返回）");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 框开：画面点按不寻路；对照：取消后同一点按 = 寻路（walk_to 进 Dialog 语境）。
#[test]
fn taps_ignored_while_confirm_open_then_work_after_cancel() {
    let (mut rt, src, dir) = runtime("taps");
    to_map(&mut rt, &src);
    let tap = {
        let sc = rt.scene.as_ref().unwrap();
        let geom = TouchGeom {
            cam_x: sc.view.cam_x,
            cam_y: sc.view.cam_y,
            tile_off: sc.layout.view_top_y,
            cols: sc.view.wide,
            rows: sc.view.high,
        };
        let me = (sc.player_cell_x, sc.player_cell_y);
        let half = TouchGeom::TILE / 2;
        let (w, h) = (sc.layout.screen_w, sc.layout.screen_h);
        (0..geom.rows)
            .flat_map(|cy| (0..geom.cols).map(move |cx| (cx, cy)))
            .filter(|&c| c != me && sc.walkable.get(c.1 as usize).and_then(|r| r.get(c.0 as usize)) == Some(&true))
            .map(|(cx, cy)| {
                let (ox, oy) = geom.cell_screen_xy(cx, cy);
                (ox + half, oy + half)
            })
            .find(|&(x, y)| x >= 0 && y >= 20 && x < w - 60 && y < h)
            .map(|(x, y)| (x as f32, y as f32))
            .expect("视口内有可走格")
    };
    assert!(rt.back_opens_exit_confirm());
    drive(&mut rt, &src, &[], vec![tap]);
    assert!(open(&rt) && mode(&rt) == GameMode::Map, "框开：点地不寻路");
    press(&mut rt, &src, -7);
    assert!(!open(&rt), "-7 取消");
    drive(&mut rt, &src, &[], vec![tap]);
    assert_eq!(mode(&rt), GameMode::Dialog, "对照：关框后同一点按 = 寻路（walk_to 进 Dialog 语境）");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 框内确认（确定键 -5 / 左软键 -6）= 退出进程。
#[test]
fn confirm_requests_process_exit() {
    for key in [-5, -6] {
        let (mut rt, src, dir) = runtime(&format!("ok{}", -key));
        to_map(&mut rt, &src);
        assert!(rt.back_opens_exit_confirm());
        assert!(!rt.exit_requested());
        press(&mut rt, &src, key);
        assert!(rt.exit_requested(), "框内确认（key {key}）= 退出进程");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// 框已开：再次返回不重开（照旧投 -7）⇒ 取消、游戏继续。
#[test]
fn back_again_cancels() {
    let (mut rt, src, dir) = runtime("again");
    to_map(&mut rt, &src);
    assert!(rt.back_opens_exit_confirm());
    assert!(!rt.back_opens_exit_confirm(), "框已开：返回不再开框（照旧投 -7）");
    press(&mut rt, &src, -7);
    assert!(!open(&rt) && !rt.exit_requested(), "再次返回 = 取消");
    assert_eq!(mode(&rt), GameMode::Map);
    let _ = std::fs::remove_dir_all(&dir);
}
