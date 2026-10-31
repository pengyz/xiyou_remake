//! STAGE-B-005（系统对话框版）游戏内退出确认·壳宿主钉（runtime.rs = android_main 同一调度体）。
//!
//! 用户 2026-10-27："游戏内右滑弹出退出确认菜单"；实机评审游戏内样式后改裁："不用游戏内UI，直接用Android标准关闭UI，
//! 作为shell层实现，提醒效果更明确"（文案 标题"退出游戏"/按钮"取消·退出"；框开时音乐继续）。
//! 对话框本体 = Android `AlertDialog`（Java 辅助类 `com.jjxy.game.ExitConfirm`，JNI 工作线程 `exit_dialog.rs`，
//! 只能实机验证）；本测钉 Runtime 状态机：地图态返回 ⇒ 请求对话框（未决期游戏暂停、输入不进游戏）⇒
//! 结果：退出 = 进程退出请求 / 取消 = 原样继续。游戏核心零改动。

use game_android::input::KeyAction;
use game_android::native::{OverlayState, TitleGesture};
use game_android::runtime::{build_boot_machine, Runtime};
use game_core::enums::GameMode;
use game_core::layout::Layout;

/// 设备同构双档布局（portrait.rs：开场链 7×10、游戏 12×16）。
fn runtime(tag: &str) -> (Runtime, game_shell_common::asset::DirAssetSource, std::path::PathBuf) {
    let raw = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw");
    let src = game_shell_common::asset::DirAssetSource::new(&raw);
    let dir = std::env::temp_dir().join(format!("jjxy-exitdlg-{tag}-{}", std::process::id()));
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
            // 【计划 3 T7】首次到地图弹提示卡（浮层吞 -7）——统一关框后再交用例
            if rt.scene.as_ref().unwrap().overlay_active {
                drive(rt, src, &[KeyAction::Press(-5), KeyAction::Release], Vec::new());
                drive(rt, src, &[], Vec::new());
            }
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

/// 地图态返回 = 请求系统退出对话框（不进游戏内菜单）；未决期触屏按 UI 态。
#[test]
fn back_on_map_requests_dialog_instead_of_menu() {
    let (mut rt, src, dir) = runtime("open");
    to_map(&mut rt, &src);
    assert!(!rt.exit_dialog_pending());
    assert!(rt.back_requests_exit_dialog(), "地图态返回 = 请求退出对话框");
    assert!(rt.exit_dialog_pending());
    drive(&mut rt, &src, &[], Vec::new());
    assert_eq!(mode(&rt), GameMode::Map, "未进游戏内菜单");
    assert!(!matches!(rt.overlay_state(), OverlayState::Game), "未决期触屏按 UI 态");
    assert!(!rt.exit_requested(), "仅请求对话框，未退出");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 未决期游戏暂停：逻辑拍不出、键与点按不进游戏。
#[test]
fn game_paused_while_dialog_pending() {
    let (mut rt, src, dir) = runtime("pause");
    to_map(&mut rt, &src);
    assert!(rt.back_requests_exit_dialog());
    let fc = rt.frame_counter;
    let px = rt.scene.as_ref().unwrap().view.player_px;
    for _ in 0..20 {
        drive(&mut rt, &src, &[KeyAction::Press(-7), KeyAction::Release], vec![(100.0, 200.0)]);
    }
    assert_eq!(rt.frame_counter, fc, "未决期不出逻辑拍");
    assert_eq!(mode(&rt), GameMode::Map, "键不进游戏（-7 未开游戏内菜单）");
    assert_eq!(rt.scene.as_ref().unwrap().view.player_px, px, "点按不寻路");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 取消 = 关框原样继续：逻辑拍恢复、输入恢复（面板右软键 -7 照旧开游戏内菜单）。
#[test]
fn cancel_resumes_play() {
    let (mut rt, src, dir) = runtime("cancel");
    to_map(&mut rt, &src);
    assert!(rt.back_requests_exit_dialog());
    rt.on_exit_dialog_result(false);
    assert!(!rt.exit_dialog_pending(), "取消即结束未决");
    assert!(!rt.exit_requested(), "取消不退出");
    let fc = rt.frame_counter;
    drive(&mut rt, &src, &[], Vec::new());
    assert_eq!(rt.frame_counter, fc + 1, "逻辑拍恢复");
    press(&mut rt, &src, -7);
    assert_eq!(mode(&rt), GameMode::InGameMenu, "输入恢复：-7 开游戏内菜单");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 选"退出" = 进程退出请求（走既有退出链：不再出拍 → finish → 销毁后 _exit）。
#[test]
fn exit_choice_requests_process_exit() {
    let (mut rt, src, dir) = runtime("exit");
    to_map(&mut rt, &src);
    assert!(rt.back_requests_exit_dialog());
    rt.on_exit_dialog_result(true);
    assert!(!rt.exit_dialog_pending());
    assert!(rt.exit_requested(), "选退出 = 进程退出请求");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 面板右软键（键批 -7，不经 BACK 分支）在地图上照旧开游戏内菜单；菜单里返回不请求对话框（照旧 -7 返回）。
#[test]
fn panel_right_softkey_still_opens_menu() {
    let (mut rt, src, dir) = runtime("panel");
    to_map(&mut rt, &src);
    press(&mut rt, &src, -7);
    assert_eq!(mode(&rt), GameMode::InGameMenu, "右软键照旧开游戏内菜单");
    assert!(!rt.back_requests_exit_dialog(), "游戏内菜单里返回不请求对话框");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 地图上已开原版浮层（告示/拾取提示）时返回 = 原版 -7（关浮层/拒绝），不请求对话框。
#[test]
fn no_dialog_over_original_overlay() {
    let (mut rt, src, dir) = runtime("overlay");
    to_map(&mut rt, &src);
    rt.scene.as_mut().unwrap().show_overlay_message(0, "测试提示", 0, 0);
    assert!(!rt.back_requests_exit_dialog(), "原版浮层开着：返回不请求对话框");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 未决期再次请求无效（系统对话框自行处理返回 = 取消）。
#[test]
fn second_request_while_pending_is_noop() {
    let (mut rt, src, dir) = runtime("again");
    to_map(&mut rt, &src);
    assert!(rt.back_requests_exit_dialog());
    assert!(!rt.back_requests_exit_dialog(), "未决期不重复请求");
    assert!(rt.exit_dialog_pending());
    let _ = std::fs::remove_dir_all(&dir);
}
