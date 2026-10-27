//! Android 壳退出链宿主钉（runtime.rs = android_main 同一调度体）。
//!
//! 【2026-10-27 用户实机】"主菜单右滑无法退出，主菜单选择退出也无法结束进程"——
//! ① 主菜单选"退出"（menuItemKinds 5 → mode 22 退场动画 70 拍 → `BootMachine::finished`
//!   = CMidlet.dispatchModeEntry 进程退出信号）后 Android 壳**无人消费** finished（BUG-020 win 壳同型；
//!   Android 从未接 ProcessControl）⇒ 停在退场动画末帧（白屏），进程不退；
//! ② 返回手势（BACK）恒吞并译为右软键 -7（input.rs，退出须走游戏退出链），而标题 tick 不认 -7
//!   ⇒ 主菜单返回 = 死键。
//! 修：标题态 BACK = 选中"退出"项 + 确认（走同一条退出链）；壳每拍查 `exit_requested` ⇒ finish Activity，
//! Activity 销毁后结束进程。

use game_android::input::KeyAction;
use game_android::native::TitleGesture;
use game_android::runtime::{build_boot_machine, Runtime};
use game_core::enums::GameMode;

fn runtime(tag: &str) -> (Runtime, game_shell_common::asset::DirAssetSource, std::path::PathBuf) {
    let raw = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw");
    let src = game_shell_common::asset::DirAssetSource::new(&raw);
    let dir = std::env::temp_dir().join(format!("jjxy-exit-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (rms, stores) = game_shell_common::rms::FileRms::load(&dir);
    let machine = build_boot_machine(&src, game_core::layout::Layout::modern(7, 10));
    (Runtime::new(machine, stores, Box::new(rms)), src, dir)
}

/// 一拍 = step + paint（设备主循环同构）。
fn drive(rt: &mut Runtime, src: &dyn game_platform::AssetSource, keys: &[KeyAction], g: Vec<TitleGesture>) {
    rt.step(src, keys, g, Vec::new());
    let mut canvas = game_core::render::ArgbImage::create(240, 320);
    let mut gfx = game_android::native::canvas_graphics(&mut canvas, 1);
    rt.paint(&mut gfx);
}

fn to_title(rt: &mut Runtime, src: &dyn game_platform::AssetSource) {
    for _ in 0..400 {
        if rt.title_mode() {
            return;
        }
        drive(rt, src, &[], Vec::new());
    }
    panic!("开机 400 拍内未到标题");
}

/// 退场动画（mode 22，70 拍）走完 ⇒ 退出请求；动画期间不提前请求。返回请求出现的拍数。
fn run_exit_animation(rt: &mut Runtime, src: &dyn game_platform::AssetSource) -> usize {
    assert_eq!(rt.machine.mode, GameMode::LogoAnim, "进入退场动画（mode 22）");
    for n in 1..=120 {
        drive(rt, src, &[], Vec::new());
        if rt.exit_requested() {
            assert!(n > 60, "退场动画须播完再退（第 {n} 拍即请求）");
            return n;
        }
    }
    panic!("退场动画 120 拍后仍无退出请求");
}

/// 主菜单选"退出"（轮播左移环绕到末项 = kind 5）→ 确认 → 退场动画 → 退出请求。
#[test]
fn title_menu_exit_item_requests_process_exit() {
    let (mut rt, src, dir) = runtime("menu");
    to_title(&mut rt, &src);
    assert!(!rt.exit_requested(), "标题态无退出请求");
    drive(&mut rt, &src, &[KeyAction::Press(-3), KeyAction::Release], Vec::new());
    drive(&mut rt, &src, &[KeyAction::Press(-5), KeyAction::Release], Vec::new());
    run_exit_animation(&mut rt, &src);
    let _ = std::fs::remove_dir_all(&dir);
}

/// 标题态返回手势 = 选中"退出"项 + 确认（同一条退出链：动画 → 退出请求）。
#[test]
fn title_back_runs_the_same_exit_chain() {
    let (mut rt, src, dir) = runtime("back");
    to_title(&mut rt, &src);
    let keys = rt.title_back_keys().expect("标题态：返回 = 退出项确认");
    assert_eq!(keys, vec![KeyAction::Press(-5), KeyAction::Release]);
    let exit_idx = rt.machine.title.as_ref().unwrap().menu_kinds.iter().position(|&k| k == 5).unwrap() as i32;
    assert_eq!(rt.machine.title_cursor(), Some(exit_idx), "光标已落「退出」项");
    drive(&mut rt, &src, &keys, Vec::new());
    run_exit_animation(&mut rt, &src);
    let _ = std::fs::remove_dir_all(&dir);
}

/// 非标题态返回 = 原映射（右软键 -7：子页返回 / 游戏内菜单），不劫持。
#[test]
fn back_outside_title_keeps_right_softkey() {
    let (mut rt, src, dir) = runtime("other");
    assert!(rt.title_back_keys().is_none(), "开机 logo 期：非标题态");
    to_title(&mut rt, &src);
    // 进"继续游戏"读档槽列表（标题子页）：返回仍 = -7 原语义
    drive(&mut rt, &src, &[KeyAction::Press(-4), KeyAction::Release], Vec::new());
    drive(&mut rt, &src, &[KeyAction::Press(-5), KeyAction::Release], Vec::new());
    assert_eq!(rt.machine.mode, GameMode::EquipDetail, "继续游戏 → 读档槽");
    assert!(rt.title_back_keys().is_none(), "子页：非标题态");
    assert!(!rt.exit_requested());
    let _ = std::fs::remove_dir_all(&dir);
}
