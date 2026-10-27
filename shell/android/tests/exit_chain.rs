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
//! 游戏内菜单）。
//!
//! 【2026-10-27 用户实机 · BUG-035】"主菜单选择退出没有结束进程"——菜单"退出"仍走 mode 22 logo 退场动画
//! （5.3 s，与开机同款 logo，用户此前称"重播片头"）；用户确认后 4.25 s 上滑离开、0.28 s 后在最近任务划掉
//! （SwipeUpClean 强停）——动画未走完，进程是被系统强停而非经退出链结束。
//! 改：菜单"退出"确认（游戏进 mode 22 = 原版退出链入口）即立即退出，与返回同口径。

use game_android::input::KeyAction;
use game_android::native::TitleGesture;
use game_android::runtime::{build_boot_machine, Runtime};
use game_core::enums::GameMode;
use game_core::layout::Layout;

/// 设备同构双档布局（portrait.rs：开场链 7×10、游戏 12×16）。
fn runtime(tag: &str) -> (Runtime, game_shell_common::asset::DirAssetSource, std::path::PathBuf) {
    let raw = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw");
    let src = game_shell_common::asset::DirAssetSource::new(&raw);
    let dir = std::env::temp_dir().join(format!("jjxy-exit-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (rms, stores) = game_shell_common::rms::FileRms::load(&dir);
    let machine = build_boot_machine(&src, Layout::modern(7, 10));
    let mut rt = Runtime::new(machine, stores, Box::new(rms));
    rt.apply_layouts(Layout::modern(7, 10), Layout::modern(12, 16));
    (rt, src, dir)
}

/// 一拍 = step + paint（设备主循环同构：序章/菜单排版在 paint 内完成，不画就不推进）。
fn drive(rt: &mut Runtime, src: &dyn game_platform::AssetSource, keys: &[KeyAction], g: Vec<TitleGesture>) {
    rt.step(src, keys, g, Vec::new());
    let mut canvas = game_core::render::ArgbImage::create(576, 576);
    let mut gfx = game_android::native::canvas_graphics(&mut canvas, 1);
    rt.paint(&mut gfx);
}

fn press(rt: &mut Runtime, src: &dyn game_platform::AssetSource, code: i32) {
    drive(rt, src, &[KeyAction::Press(code), KeyAction::Release], Vec::new());
}

/// 点按 + 3 拍空转（场景菜单切换留拍）。
fn tap(rt: &mut Runtime, src: &dyn game_platform::AssetSource, code: i32) {
    press(rt, src, code);
    for _ in 0..3 {
        drive(rt, src, &[], Vec::new());
    }
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

/// 主菜单选"退出"（轮播左移环绕到末项 = kind 5）→ 确认 ⇒ **同拍**即退出请求（不等 70 拍退场动画）；
/// 游戏自身仍走原版退出链入口（mode 22 + closeAudioPlayer ⇒ BGM 已停）。
#[test]
fn title_menu_exit_item_exits_immediately() {
    let (mut rt, src, dir) = runtime("menu");
    to_title(&mut rt, &src);
    assert!(!rt.exit_requested(), "标题态无退出请求");
    press(&mut rt, &src, -3);
    assert!(!rt.exit_requested(), "仅移光标不退出");
    press(&mut rt, &src, -5);
    assert_eq!(rt.machine.mode, GameMode::LogoAnim, "游戏进原版退出链入口（mode 22）");
    assert!(rt.exit_requested(), "确认即退出请求（不等退场动画）");
    assert_eq!(rt.bgm_request(), None, "BGM 已停（closeAudioPlayer）");
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

/// 游戏中经游戏内菜单"退出"（kind 7）回主菜单后，菜单"退出"仍立即退出（场景换回 BootMachine 后
/// 退出请求面不受残留场景影响；用户实机 16:40 会话即此路径）。
#[test]
fn menu_exit_after_returning_from_game_exits_immediately() {
    let (mut rt, src, dir) = runtime("game");
    to_title(&mut rt, &src);
    press(&mut rt, &src, -5);
    assert_eq!(rt.machine.mode, GameMode::IntroCutscene, "开始游戏 → 序章");
    press(&mut rt, &src, -7);
    let entered = (0..400).any(|_| {
        drive(&mut rt, &src, &[], Vec::new());
        rt.scene.is_some()
    });
    assert!(entered, "序章跳过 → 读条 → 交接 GameScene");
    // 序章剧情对话：逐拍确认推进，稳定落 Map
    let mut calm = 0;
    for i in 0..4000 {
        let mode = rt.scene.as_ref().unwrap().game_mode;
        calm = if mode == GameMode::Map { calm + 1 } else { 0 };
        if calm >= 30 {
            break;
        }
        let keys: &[KeyAction] =
            if mode != GameMode::Map && i % 4 == 0 { &[KeyAction::Press(-5), KeyAction::Release] } else { &[] };
        drive(&mut rt, &src, keys, Vec::new());
    }
    assert_eq!(rt.scene.as_ref().unwrap().game_mode, GameMode::Map, "剧情推完 → Map");
    assert!(!rt.back_requests_exit(), "游戏内返回不劫持（-7 = 游戏内菜单）");
    tap(&mut rt, &src, -7);
    assert_eq!(rt.scene.as_ref().unwrap().game_mode, GameMode::InGameMenu, "-7 开游戏内菜单");
    let kinds = rt.scene.as_ref().unwrap().menu_kinds.clone();
    let quit = kinds.iter().position(|&k| k == 7).expect("游戏内菜单有退出项（kind 7）");
    let cur = rt.scene.as_ref().unwrap().menu_cursor as usize;
    for _ in 0..(quit + kinds.len() - cur) % kinds.len() {
        tap(&mut rt, &src, -2);
    }
    tap(&mut rt, &src, -6);
    assert!(rt.scene.is_none() && rt.title_mode(), "游戏内菜单退出 → 回主菜单");
    assert!(!rt.exit_requested(), "回主菜单不等于退出进程");
    press(&mut rt, &src, -3);
    press(&mut rt, &src, -5);
    assert!(rt.exit_requested(), "回主菜单后菜单退出 = 立即退出");
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
