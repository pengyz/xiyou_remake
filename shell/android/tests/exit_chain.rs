//! Android 壳退出链宿主钉（runtime.rs = android_main 同一调度体）。
//!
//! 【2026-10-27 用户实机 · BUG-033】"主菜单右滑无法退出，主菜单选择退出也无法结束进程"——
//! ① 主菜单选"退出"（menuItemKinds 5 → mode 22 退场动画 70 拍 → `BootMachine::finished`
//!   = CMidlet.m_000 进程退出信号）后 Android 壳**无人消费** finished ⇒ 停在退场动画末帧（白屏），进程不退；
//! ② 返回（BACK）恒吞并译为右软键 -7，而标题 tick 不认 -7 ⇒ 主菜单返回 = 死键。
//! 修：壳每拍查 `exit_requested` ⇒ finish Activity，Activity 销毁后结束进程。
//!
//! 【2026-10-27 用户实机 · BUG-034】BUG-033 版"标题返回 = 选中退出项 + 确认"走 mode 22 logo 退场动画——
//! 形似"进程内重置、重新播放片头"，5.3 s 内再返回无效；开机 logo 期返回亦无效——"不符合用户习惯"。
//! 改：**根界面**（开机 logo mode 0 / 主菜单 mode 1 / 退场动画 mode 22）返回 = 立即退出（Nokia 红键语义：
//! AMS 随时销毁 MIDlet，CMidlet.destroyApp 为空 ⇒ 跳过动画不丢状态）；其余态保持 -7（子页返回 / 序章跳过 /
//! 游戏内菜单）。菜单"退出"项仍走原版退场动画。

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

fn press(rt: &mut Runtime, src: &dyn game_platform::AssetSource, code: i32) {
    drive(rt, src, &[KeyAction::Press(code), KeyAction::Release], Vec::new());
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

/// 主菜单选"退出"（轮播左移环绕到末项 = kind 5）→ 确认 → 原版退场动画（mode 22）播完 → 退出请求；
/// 动画期间不提前请求。
#[test]
fn title_menu_exit_item_requests_process_exit() {
    let (mut rt, src, dir) = runtime("menu");
    to_title(&mut rt, &src);
    assert!(!rt.exit_requested(), "标题态无退出请求");
    press(&mut rt, &src, -3);
    press(&mut rt, &src, -5);
    assert_eq!(rt.machine.mode, GameMode::LogoAnim, "进入退场动画（mode 22）");
    let n = (1..=120)
        .find(|_| {
            drive(&mut rt, &src, &[], Vec::new());
            rt.exit_requested()
        })
        .expect("退场动画 120 拍后仍无退出请求");
    assert!(n > 60, "退场动画须播完再退（第 {n} 拍即请求）");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 开机 logo 期（mode 0，约 7 s）返回 = 立即退出——不等 logo 走完、不进主菜单。
#[test]
fn back_during_boot_logo_exits_immediately() {
    let (mut rt, src, dir) = runtime("logo");
    drive(&mut rt, &src, &[], Vec::new());
    assert_eq!(rt.machine.mode, GameMode::Boot, "开机 logo 期");
    assert!(!rt.exit_requested());
    assert!(rt.back_requests_exit(), "logo 期返回 = 立即退出");
    assert!(rt.exit_requested(), "退出请求即刻成立");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 主菜单返回 = 立即退出，**不进** mode 22 logo 退场动画（用户实机：动画形似重播片头）。
#[test]
fn back_at_title_exits_immediately_without_exit_animation() {
    let (mut rt, src, dir) = runtime("title");
    to_title(&mut rt, &src);
    assert!(rt.back_requests_exit(), "主菜单返回 = 立即退出");
    assert!(rt.exit_requested());
    assert_eq!(rt.machine.mode, GameMode::MainMenu, "未进退场动画");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 菜单"退出"后的退场动画期返回 = 立即退出（不再等剩余动画）。
#[test]
fn back_during_exit_animation_exits_immediately() {
    let (mut rt, src, dir) = runtime("anim");
    to_title(&mut rt, &src);
    press(&mut rt, &src, -3);
    press(&mut rt, &src, -5);
    assert_eq!(rt.machine.mode, GameMode::LogoAnim);
    for _ in 0..10 {
        drive(&mut rt, &src, &[], Vec::new());
    }
    assert!(!rt.exit_requested(), "动画未播完");
    assert!(rt.back_requests_exit(), "退场动画期返回 = 立即退出");
    assert!(rt.exit_requested());
    let _ = std::fs::remove_dir_all(&dir);
}

/// 非根界面返回 = 原映射右软键 -7，不劫持：读档子页（-7 回主菜单）、序章（-7 跳过）、游戏内（场景活动）。
#[test]
fn back_outside_root_keeps_right_softkey() {
    let (mut rt, src, dir) = runtime("other");
    to_title(&mut rt, &src);
    press(&mut rt, &src, -4);
    press(&mut rt, &src, -5);
    assert_eq!(rt.machine.mode, GameMode::EquipDetail, "继续游戏 → 读档槽");
    assert!(!rt.back_requests_exit(), "读档子页：不劫持");
    press(&mut rt, &src, -7);
    assert_eq!(rt.machine.mode, GameMode::MainMenu, "子页 -7 = 回主菜单");
    press(&mut rt, &src, -3);
    press(&mut rt, &src, -5);
    assert_eq!(rt.machine.mode, GameMode::IntroCutscene, "开始游戏 → 序章");
    assert!(!rt.back_requests_exit(), "序章：不劫持（-7 = 跳过）");
    press(&mut rt, &src, -7);
    let entered = (0..400).any(|_| {
        drive(&mut rt, &src, &[], Vec::new());
        rt.scene.is_some()
    });
    assert!(entered, "序章跳过 → 读条 → 交接 GameScene");
    assert!(!rt.back_requests_exit(), "游戏内：不劫持（-7 = 游戏内菜单）");
    assert!(!rt.exit_requested(), "全程无退出请求");
    let _ = std::fs::remove_dir_all(&dir);
}
