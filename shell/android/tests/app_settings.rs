//! 【STAGE-B-006】Android 设置文件接入宿主钉（设计件 docs/superpowers/specs/2026-10-28-settings-splash-key-feedback-design.md §4）：
//! 启动装载并应用（开屏默认关 ⇒ 首拍前即主菜单；声音先于跳开屏写入）、两处设置页 5 行、切换同拍落盘、回标题 / 读档交接不回滚。
//! 运行时与 android_main 同一调度体（runtime.rs）；真资产 + 临时目录。

use game_android::input::KeyAction;
use game_android::runtime::{build_boot_machine, Runtime, PREF_CLICK, PREF_HAPTIC, PREF_SPLASH};
use game_core::enums::GameMode;
use game_shell_common::settings::AppSettings;

fn raw() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw")
}

fn tmp(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("jjxy-appset-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// 运行时（android_main 同序：Runtime::new 后、首拍前 open_app_settings）。`stores` = 预置存档（读档用例）或空。
fn runtime_with(dir: &std::path::Path, stores: Vec<(String, Vec<u8>)>) -> (Runtime, game_shell_common::asset::DirAssetSource) {
    let src = game_shell_common::asset::DirAssetSource::new(&raw());
    let (rms, _) = game_shell_common::rms::FileRms::load(&dir.to_path_buf());
    let machine = build_boot_machine(&src, game_core::layout::Layout::modern(18, 18));
    let mut rt = Runtime::new(machine, stores, Box::new(rms));
    rt.open_app_settings(dir.join("app_settings.cfg"));
    (rt, src)
}

/// 一拍 = step + paint（设备主循环同构）。
fn drive(rt: &mut Runtime, src: &dyn game_platform::AssetSource, keys: &[KeyAction]) {
    rt.step(src, keys, Vec::new(), Vec::new());
    let mut canvas = game_core::render::ArgbImage::create(576, 576);
    let mut g = game_android::native::canvas_graphics(&mut canvas, 1);
    rt.paint(&mut g);
}

/// 点按一次（Press+Release 同批）+ 3 拍空闲。
fn tap_key(rt: &mut Runtime, src: &dyn game_platform::AssetSource, code: i32) {
    drive(rt, src, &[KeyAction::Press(code), KeyAction::Release]);
    for _ in 0..3 {
        drive(rt, src, &[]);
    }
}

fn run_until(rt: &mut Runtime, src: &dyn game_platform::AssetSource, n: usize, mut done: impl FnMut(&Runtime) -> bool) -> bool {
    for _ in 0..n {
        if done(rt) {
            return true;
        }
        drive(rt, src, &[]);
    }
    done(rt)
}

fn rows_of(opts: &[game_core::menu_family::ShellOption]) -> Vec<(u8, String, bool)> {
    opts.iter().map(|o| (o.id, String::from_utf16_lossy(&o.label), o.checked)).collect()
}

fn expected_rows(splash: bool, haptic: bool, click: bool) -> Vec<(u8, String, bool)> {
    vec![
        (PREF_SPLASH, "开屏画面".into(), splash),
        (PREF_HAPTIC, "按键震动".into(), haptic),
        (PREF_CLICK, "按键音".into(), click),
    ]
}

/// 标题光标移到 kind `k` 并确认（标题轮播：-2 下一项、-5 确认）。
fn title_open(rt: &mut Runtime, src: &dyn game_platform::AssetSource, k: i8) {
    let kinds = rt.machine.title.as_ref().unwrap().menu_kinds.clone();
    let want = kinds.iter().position(|&x| x == k).unwrap() as i32;
    while rt.machine.title_cursor() != Some(want) {
        tap_key(rt, src, -2);
    }
    tap_key(rt, src, -5);
}

/// 地图行走态开游戏内菜单并进入 kind `k`（菜单：-2 下移、-6 确认）。
fn in_game_open(rt: &mut Runtime, src: &dyn game_platform::AssetSource, k: i8) {
    tap_key(rt, src, -7);
    let sc = rt.scene.as_ref().unwrap();
    assert_eq!(sc.game_mode, GameMode::InGameMenu, "游戏内菜单");
    let kinds = sc.menu_kinds.clone();
    let want = kinds.iter().position(|&x| x == k).expect("菜单条目在") as i32;
    while rt.scene.as_ref().unwrap().menu_cursor != want {
        tap_key(rt, src, -2);
    }
    tap_key(rt, src, -6);
}

/// 剧情推完、稳定落 Map（非 Map 态每 4 拍按一次确认）。
fn settle_map(rt: &mut Runtime, src: &dyn game_platform::AssetSource) {
    let mut calm = 0;
    for i in 0..4000 {
        let mode = rt.scene.as_ref().unwrap().game_mode;
        calm = if mode == GameMode::Map { calm + 1 } else { 0 };
        if calm >= 30 {
            return;
        }
        let keys: &[KeyAction] = if mode != GameMode::Map && i % 4 == 0 {
            &[KeyAction::Press(-5), KeyAction::Release]
        } else {
            &[]
        };
        drive(rt, src, keys);
    }
    panic!("未稳定落 Map");
}

/// 新游戏直达 Map（标题"开始游戏" → -7 跳序章 → 交接 → 推完剧情）。
fn new_game_to_map(rt: &mut Runtime, src: &dyn game_platform::AssetSource) {
    assert!(run_until(rt, src, 200, |r| r.title_mode()), "到标题");
    title_open(rt, src, 0);
    assert_eq!(rt.machine.mode, GameMode::IntroCutscene, "序章");
    drive(rt, src, &[KeyAction::Press(-7), KeyAction::Release]);
    assert!(run_until(rt, src, 400, |r| r.scene.is_some()), "交接 GameScene");
    settle_map(rt, src);
}

/// oracle 预置存档（`键=hex`；reference/ 只读，win 壳 / handoff 测试同口径）。
fn preset_stores(name: &str) -> Vec<(String, Vec<u8>)> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../reference/oracle/scenarios").join(name);
    std::fs::read_to_string(p)
        .unwrap()
        .lines()
        .filter_map(|l| l.split_once('='))
        .map(|(k, hex)| {
            let h = hex.trim();
            let bytes = (0..h.len() / 2).map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).unwrap()).collect();
            (k.trim().to_string(), bytes)
        })
        .collect()
}

#[test]
fn no_file_starts_at_main_menu_with_default_rows() {
    let dir = tmp("nofile");
    let (mut rt, src) = runtime_with(&dir, Vec::new());
    let cfg = dir.join("app_settings.cfg");
    assert_eq!(rt.machine.mode, GameMode::MainMenu, "开屏默认关：首拍前即主菜单");
    assert!(rt.title_mode(), "首拍前即标题态（壳据此路由首批触点，复核 T3-F2）");
    assert_eq!(rt.frame_interval_ms(), 75, "不走 mode 0 的 100 ms 节拍");
    assert_eq!(rows_of(&rt.machine.shell_options), expected_rows(false, true, true), "三行默认值");
    drive(&mut rt, &src, &[]);
    assert_eq!(rt.machine.mode, GameMode::MainMenu);
    assert_eq!(rt.bgm_request(), Some(2), "标题曲直接开始");
    assert_eq!(rt.app_settings(), AppSettings::default());
    assert!(!cfg.exists(), "未改动 ⇒ 不写文件");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn splash_on_plays_boot_screens() {
    let dir = tmp("splash");
    std::fs::write(dir.join("app_settings.cfg"), "show_splash=true\n").unwrap();
    let (mut rt, src) = runtime_with(&dir, Vec::new());
    assert_eq!(rt.machine.mode, GameMode::Boot, "开屏开：仍走 mode 0");
    let mut steps = 0;
    while rt.machine.mode == GameMode::Boot {
        drive(&mut rt, &src, &[]);
        steps += 1;
        assert!(steps <= 100, "开屏未结束");
        if steps == 1 {
            assert_eq!(rt.frame_interval_ms(), 100, "mode 0 节拍 100 ms（a.java:3350）");
        }
    }
    assert_eq!(steps, 68, "三屏 = l0 16 + l1 16 + logo 36 拍");
    assert!(rt.title_mode(), "开屏后进主菜单");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn saved_sound_off_restored_before_title() {
    let dir = tmp("soundoff");
    let cfg = dir.join("app_settings.cfg");
    std::fs::write(&cfg, "sound_enabled=false\nvolume_level=0\n").unwrap();
    let (mut rt, src) = runtime_with(&dir, Vec::new());
    assert!(!rt.sound_enabled(), "声音关跨启动保留");
    assert_eq!(
        rt.app_settings(),
        AppSettings { sound_enabled: false, volume_level: 0, ..AppSettings::default() },
        "首拍前：声音对（含音量）原样恢复（复核 T6-F2 / T6b-N2）"
    );
    drive(&mut rt, &src, &[]);
    assert_eq!(rt.bgm_request(), None, "跳过开屏直入标题：标题曲不响（声音先于跳开屏写入）");
    assert_eq!(std::fs::read_to_string(&cfg).unwrap(), "sound_enabled=false\nvolume_level=0\n", "首拍不改写");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 未接设置文件（宿主流程测试同款）：三行缺席 ⇒ 快照取默认（复核 T6-F2）；开屏照常播放。
#[test]
fn without_settings_file_snapshot_is_default() {
    let src = game_shell_common::asset::DirAssetSource::new(&raw());
    let dir = tmp("nowire");
    let (rms, _) = game_shell_common::rms::FileRms::load(&dir);
    let rt = Runtime::new(build_boot_machine(&src, game_core::layout::Layout::modern(18, 18)), Vec::new(), Box::new(rms));
    assert!(rt.machine.shell_options.is_empty(), "未接线：无附加行（设置页仍是原版两行）");
    assert_eq!(rt.app_settings(), AppSettings::default());
    assert_eq!(rt.machine.mode, GameMode::Boot, "未接线：不跳开屏");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn title_settings_toggle_persists_same_step() {
    let dir = tmp("title");
    let cfg = dir.join("app_settings.cfg");
    let (mut rt, src) = runtime_with(&dir, Vec::new());
    drive(&mut rt, &src, &[]);
    title_open(&mut rt, &src, 2); // 设置
    assert_eq!(rt.machine.mode, GameMode::About, "标题侧设置页");
    let opts = rt.machine.options.as_ref().unwrap();
    assert_eq!(opts.len(), 5, "声音 / 小地图 + 三行");
    assert_eq!(rows_of(opts.shell_rows()), expected_rows(false, true, true));
    tap_key(&mut rt, &src, -2);
    tap_key(&mut rt, &src, -2);
    assert_eq!(rt.machine.options.as_ref().unwrap().highlight_index(), 2, "高亮 = 开屏画面");
    rt.step(&src, &[KeyAction::Press(-5), KeyAction::Release], Vec::new(), Vec::new());
    assert!(AppSettings::load(&cfg).show_splash, "切换那一拍即落盘");
    assert!(rt.app_settings().show_splash);
    let (rt2, _) = runtime_with(&dir, Vec::new());
    assert_eq!(rt2.machine.mode, GameMode::Boot, "下次启动播放开屏");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn in_game_toggle_persists_and_survives_return_to_title() {
    let dir = tmp("ingame");
    let cfg = dir.join("app_settings.cfg");
    let (mut rt, src) = runtime_with(&dir, Vec::new());
    new_game_to_map(&mut rt, &src);
    in_game_open(&mut rt, &src, 4); // 设置
    let sc = rt.scene.as_ref().unwrap();
    assert_eq!(sc.game_mode, GameMode::About, "游戏内设置页");
    assert_eq!(rows_of(sc.options.as_ref().unwrap().shell_rows()), expected_rows(false, true, true), "交接带入三行");
    for _ in 0..3 {
        tap_key(&mut rt, &src, -2);
    }
    assert_eq!(rt.scene.as_ref().unwrap().options.as_ref().unwrap().highlight_index(), 3, "高亮 = 按键震动");
    rt.step(&src, &[KeyAction::Press(-5), KeyAction::Release], Vec::new(), Vec::new());
    assert!(!AppSettings::load(&cfg).key_haptic, "切换那一拍即落盘");
    let saved = std::fs::read(&cfg).unwrap();
    // 换成同值的非规范写法：之后任何重写（哪怕内容相同）都会改变字节（复核 T6-F1）
    let same = format!("# 手写\n{}", String::from_utf8(saved).unwrap().lines().rev().collect::<Vec<_>>().join("\n"));
    std::fs::write(&cfg, &same).unwrap();
    for _ in 0..5 {
        drive(&mut rt, &src, &[]);
    }
    assert_eq!(std::fs::read_to_string(&cfg).unwrap(), same, "无变化 ⇒ 不再写");
    tap_key(&mut rt, &src, -7); // 回游戏内菜单
    let kinds = rt.scene.as_ref().unwrap().menu_kinds.clone();
    let exit = kinds.iter().position(|&x| x == 7).unwrap() as i32;
    while rt.scene.as_ref().unwrap().menu_cursor != exit {
        tap_key(&mut rt, &src, -2);
    }
    tap_key(&mut rt, &src, -6); // 退出 → 回标题
    assert!(rt.scene.is_none() && rt.title_mode(), "回到标题");
    assert!(!rt.app_settings().key_haptic, "回标题不回滚");
    assert_eq!(rows_of(&rt.machine.shell_options), expected_rows(false, false, true));
    assert_eq!(std::fs::read_to_string(&cfg).unwrap(), same, "回标题无变化 ⇒ 不再写");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 复核 T6-F3：首帧呈现前的输入（用户看见的是黑屏）一律丢弃——开屏关时首拍前即主菜单，看不见的标题确认区 /
/// 面板确定键若接住这批点按，会直接开始新游戏。首帧呈现之后照常交出。
#[test]
fn inputs_before_first_frame_are_dropped() {
    let mut w = game_android::native::ShellWiring::new();
    let mut keys = vec![KeyAction::Press(-5)];
    w.pending_taps.push((10.0, 10.0));
    w.title_gestures.push(game_android::native::TitleGesture::Tap(10.0, 10.0));
    let (k, g, t) = w.take_step_inputs(&mut keys);
    assert!(k.is_empty() && g.is_empty() && t.is_empty(), "首帧前：键 / 手势 / 点按全丢");
    assert!(keys.is_empty() && w.pending_taps.is_empty() && w.title_gestures.is_empty(), "队列已清空（不留到首帧后）");
    w.mark_presented();
    let mut keys = vec![KeyAction::Press(-5), KeyAction::Release];
    w.pending_taps.push((1.0, 2.0));
    w.title_gestures.push(game_android::native::TitleGesture::Tap(3.0, 4.0));
    let (k, g, t) = w.take_step_inputs(&mut keys);
    assert_eq!(k, vec![KeyAction::Press(-5), KeyAction::Release], "首帧后照常交出");
    assert_eq!((g.len(), t), (1, vec![(1.0, 2.0)]));
    assert!(keys.is_empty(), "取走即清");
}

/// 复核 T6b-N1：标题触点首帧前按下、首帧后抬起——按下那半被丢弃，抬起不得补出 Tap（否则仍会开始新游戏）。
#[test]
fn title_touch_spanning_first_frame_yields_no_gesture() {
    use game_android::input::{TouchEvent, TouchPhase};
    let mut w = game_android::native::ShellWiring::new();
    w.title_mode = true;
    let ev = |phase, x, y| TouchEvent { id: 1, phase, x, y };
    w.on_touch_event(&ev(TouchPhase::Down, 120.0, 300.0));
    let mut keys = Vec::new();
    let _ = w.take_step_inputs(&mut keys); // 首帧前：丢弃
    w.mark_presented();
    w.on_touch_event(&ev(TouchPhase::Up, 120.0, 300.0));
    let (_, g, _) = w.take_step_inputs(&mut keys);
    assert!(g.is_empty(), "跨首帧的点按不补出 Tap（实得 {g:?}）");
    // 首帧后的完整点按照常
    w.on_touch_event(&ev(TouchPhase::Down, 120.0, 300.0));
    w.on_touch_event(&ev(TouchPhase::Up, 120.0, 300.0));
    let (_, g, _) = w.take_step_inputs(&mut keys);
    assert_eq!(g, vec![game_android::native::TitleGesture::Tap(120.0, 300.0)]);
}

/// 复核 T6b-N3：呈现成功即放行输入（宿主可测：非 Android 分支不解引用窗口）；空窗口不算呈现。
#[test]
fn presenting_marks_first_frame() {
    let mut w = game_android::native::ShellWiring::new();
    let mut keys = vec![KeyAction::Press(-6)];
    assert!(!w.present(std::ptr::null_mut(), &[]), "空窗口不算呈现");
    assert!(w.take_step_inputs(&mut keys).0.is_empty(), "未呈现：仍丢弃");
    let window = std::ptr::NonNull::<game_android::native::ANativeWindow>::dangling().as_ptr();
    assert!(w.present(window, &[]), "宿主桩：非空窗口即视为呈现");
    let mut keys = vec![KeyAction::Press(-6)];
    assert_eq!(w.take_step_inputs(&mut keys).0, vec![KeyAction::Press(-6)], "呈现后放行");
    let mut w2 = game_android::native::ShellWiring::new();
    assert!(w2.present_layers(window, &[]), "多层呈现同口径");
    let mut keys = vec![KeyAction::Press(-6)];
    assert_eq!(w2.take_step_inputs(&mut keys).0.len(), 1);
}

/// 复核 T4-F2：非默认设置经读档交接不被写回默认（场景缺行时 app_settings 会取默认 ⇒ 读档漏拷会静默回滚）。
#[test]
fn slot_load_keeps_non_default_settings() {
    let dir = tmp("slot");
    let cfg = dir.join("app_settings.cfg");
    let original = "show_splash=true\nkey_haptic=false\n";
    std::fs::write(&cfg, original).unwrap();
    let (mut rt, src) = runtime_with(&dir, preset_stores("preset-floor1.preset"));
    let loaded = rt.app_settings();
    assert!(loaded.show_splash && !loaded.key_haptic, "前置：非默认值已装载");
    assert!(run_until(&mut rt, &src, 200, |r| r.title_mode()), "开屏后到标题");
    title_open(&mut rt, &src, 1); // 继续游戏（读档列表）
    assert_eq!(rt.machine.mode, GameMode::EquipDetail, "读档列表");
    tap_key(&mut rt, &src, -5); // 读槽 0
    assert!(run_until(&mut rt, &src, 400, |r| r.scene.is_some()), "读档交接 GameScene");
    assert_eq!(rt.app_settings(), loaded, "交接拍后设置不变");
    assert_eq!(std::fs::read_to_string(&cfg).unwrap(), original, "文件未被改写");
    settle_map(&mut rt, &src);
    in_game_open(&mut rt, &src, 4);
    let sc = rt.scene.as_ref().unwrap();
    assert_eq!(sc.game_mode, GameMode::About);
    assert_eq!(rows_of(sc.options.as_ref().unwrap().shell_rows()), expected_rows(true, false, true), "游戏内设置页 5 行、值同文件");
    let _ = std::fs::remove_dir_all(&dir);
}
