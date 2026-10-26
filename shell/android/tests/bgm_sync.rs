//! Android 壳 BGM 命令翻译宿主钉：`Runtime` 每拍 + hideNotify 后把活动机 BGM 快照
//! （track/serial/volume）经 shell-common `BgmController`（win 壳同源）译成 Play/Stop——
//! 与 android_main 同一调度体（runtime.rs）。
//!
//! 实机根因（2026-10-26）：native.rs 主循环手写翻译只认轨道下标——变 None 不发 Stop（切后台/
//! 设置关声后音乐照放）、`last_bgm` 永不复位（`bgm req=None` 每拍刷屏、logcat 轮转）、同轨
//! serial 变（Java 关闭重建 = 从头播放，a.java:10178-10199）不重发、音量恒 1.0（应为
//! VolumeControl 级别/100，默认 60）。

use game_android::input::KeyAction;
use game_android::native::TitleGesture;
use game_android::runtime::{build_boot_machine, Runtime};
use game_shell_common::audio::{AudioSink, Cmd};
use std::sync::{Arc, Mutex};

fn raw() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw")
}

struct Recorder(Arc<Mutex<Vec<Cmd>>>);

impl AudioSink for Recorder {
    fn send(&mut self, cmd: Cmd) {
        self.0.lock().unwrap().push(cmd);
    }
}

/// 一拍 = step + paint（设备主循环每拍 tick→paint 同构）。
fn drive(rt: &mut Runtime, src: &dyn game_platform::AssetSource, keys: &[KeyAction], g: Vec<TitleGesture>) {
    rt.step(src, keys, g, Vec::new());
    let mut canvas = game_core::render::ArgbImage::create(576, 576);
    let mut gfx = game_android::native::canvas_graphics(&mut canvas, 1);
    rt.paint(&mut gfx);
}

fn idle_ticks(rt: &mut Runtime, src: &dyn game_platform::AssetSource, n: usize) {
    for _ in 0..n {
        drive(rt, src, &[], Vec::new());
    }
}

#[test]
fn bgm_commands_follow_core_requests_across_lifecycle() {
    let src = game_shell_common::asset::DirAssetSource::new(&raw());
    let dir = std::env::temp_dir().join(format!("jjxy-bgm-sync-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (rms, stores) = game_shell_common::rms::FileRms::load(&dir);
    let machine = build_boot_machine(&src, game_core::layout::Layout::modern(18, 18));
    let mut rt = Runtime::new(machine, stores, Box::new(rms));
    let log = Arc::new(Mutex::new(Vec::new()));
    rt.set_bgm_sink(Box::new(Recorder(log.clone())));
    let cmds = || log.lock().unwrap().clone();
    let mut seen: Vec<(&str, Vec<Cmd>)> = Vec::new();

    // 开机 → 标题：菜单曲 track2；VolumeControl 级别 60 ⇒ gain 0.6。空转多拍不重发。
    for _ in 0..400 {
        if rt.title_mode() {
            break;
        }
        drive(&mut rt, &src, &[], Vec::new());
    }
    assert!(rt.title_mode(), "开机 → 标题");
    idle_ticks(&mut rt, &src, 20);
    seen.push(("标题", cmds()));

    // 切后台（hideNotify = closeAudioPlayer + anyKeyPlayed）：**即时** Stop（不等下一拍）
    rt.hide_notify();
    seen.push(("hideNotify 即时", cmds()));
    idle_ticks(&mut rt, &src, 10);

    // 回前台后首个按键 = anyKeyPlayed 续播：同轨新播放器 ⇒ 从头播（Java 关闭重建）
    drive(&mut rt, &src, &[KeyAction::Press(-1), KeyAction::Release], Vec::new());
    idle_ticks(&mut rt, &src, 5);
    seen.push(("按键续播", cmds()));

    // 设置里关声 ⇒ Stop；再开 ⇒ 音量复位 60 + 从头播（a.java:4157-4167）
    rt.machine.bgm.toggle_sound();
    idle_ticks(&mut rt, &src, 3);
    rt.machine.bgm.toggle_sound();
    idle_ticks(&mut rt, &src, 3);
    seen.push(("设置关声/开声", cmds()));

    // 开始游戏 → 序章 -7 跳过 → 读条完 enter_game ⇒ 游戏曲 track3
    let geo = rt.machine.title_geom().unwrap();
    while rt.machine.title_cursor() != Some(0) {
        let p = ((geo.zone_prev.0 + geo.zone_prev.2 / 2) as f32, (geo.zone_prev.1 + geo.zone_prev.3 / 2) as f32);
        drive(&mut rt, &src, &[], vec![TitleGesture::Tap(p.0, p.1)]);
    }
    let ok = ((geo.zone_ok.0 + geo.zone_ok.2 / 2) as f32, (geo.zone_ok.1 + geo.zone_ok.3 / 2) as f32);
    drive(&mut rt, &src, &[], vec![TitleGesture::Tap(ok.0, ok.1)]);
    drive(&mut rt, &src, &[KeyAction::Press(-7), KeyAction::Release], Vec::new());
    for _ in 0..400 {
        if rt.scene.is_some() {
            break;
        }
        drive(&mut rt, &src, &[], Vec::new());
    }
    assert!(rt.scene.is_some(), "交接 GameScene");
    idle_ticks(&mut rt, &src, 10);
    seen.push(("进游戏", cmds()));

    // 游戏中切后台：GameScene::hide_notify（mode3→4 + closeAudioPlayer）⇒ 即时 Stop
    rt.hide_notify();
    seen.push(("游戏中 hideNotify 即时", cmds()));

    let p2 = Cmd::Play { track: 2, gain: 0.6 };
    let p3 = Cmd::Play { track: 3, gain: 0.6 };
    let s = Cmd::Stop;
    assert_eq!(
        seen,
        vec![
            ("标题", vec![p2]),
            ("hideNotify 即时", vec![p2, s]),
            ("按键续播", vec![p2, s, p2]),
            ("设置关声/开声", vec![p2, s, p2, s, p2]),
            ("进游戏", vec![p2, s, p2, s, p2, p3]),
            ("游戏中 hideNotify 即时", vec![p2, s, p2, s, p2, p3, s]),
        ],
        "BGM 命令序列（每检查点累计）"
    );
    assert!(!rt.crashed);
    let _ = std::fs::remove_dir_all(&dir);
}
