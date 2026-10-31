//! 【STAGE-B-006】Android 设置文件接入宿主钉（设计件 docs/superpowers/specs/2026-10-28-settings-splash-key-feedback-design.md §4）：
//! 启动装载并应用（开屏默认关 ⇒ 首拍前即主菜单；声音先于跳开屏写入）、两处设置页 5 行、切换同拍落盘、回标题 / 读档交接不回滚。
//! 运行时与 android_main 同一调度体（runtime.rs）；真资产 + 临时目录。

use game_android::input::KeyAction;
use game_android::runtime::{build_boot_machine, Runtime, PREF_CLICK, PREF_HAPTIC, PREF_SPLASH, PREF_CONTROL_MODE};
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
    // 【计划 3 T3.2 前提更正】壳行恒含第 4 行「操控方式」（勾=触屏；默认摇杆=false）；
    // 自动战斗行仅触屏模式出现（这些用例都在默认摇杆模式下跑）
    vec![
        (PREF_SPLASH, "开屏画面".into(), splash),
        (PREF_HAPTIC, "按键震动".into(), haptic),
        (PREF_CLICK, "按键音".into(), click),
        (PREF_CONTROL_MODE, "操控方式".into(), false),
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
    assert_eq!(rows_of(&rt.machine.shell_options), expected_rows(false, true, true), "壳行默认值（含操控方式=摇杆）");
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
    assert_eq!(opts.len(), 6, "声音 / 小地图 + 三行 + 操控方式");
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

/// 【计划 3 T3.2 · spec §8 · 复核 T3-R1】操控方式翻转链（宿主级，生产路径）：翻「操控方式」同拍
/// rt_control_mode=Touch、活动机行集出现第 5 行「自动战斗」、设置页 options 重建（6+1 行）且高亮停在操控方式行、
/// cfg 落盘 control_mode=touch；翻「自动战斗」同拍落盘；切回摇杆 ⇒ 第 5 行消失且 auto_battle 保留落盘。
#[test]
fn control_mode_toggle_chain_rows_and_persist() {
    use game_android::runtime::PREF_AUTO_BATTLE;
    let dir = tmp("ctlmode");
    let cfg = dir.join("app_settings.cfg");
    let (mut rt, src) = runtime_with(&dir, Vec::new());
    drive(&mut rt, &src, &[]);
    title_open(&mut rt, &src, 2); // 设置
    // 高亮移到「操控方式」（第 6 行，下标 5）并确认
    for _ in 0..5 {
        tap_key(&mut rt, &src, -2);
    }
    assert_eq!(rt.machine.options.as_ref().unwrap().highlight_index(), 5, "高亮 = 操控方式");
    rt.step(&src, &[KeyAction::Press(-5), KeyAction::Release], Vec::new(), Vec::new());
    assert_eq!(rt.rt_control_mode, game_shell_common::settings::ControlMode::Touch, "同拍 rt=触屏");
    let ids: Vec<u8> = rt.machine.shell_options.iter().map(|r| r.id).collect();
    assert!(ids.contains(&PREF_AUTO_BATTLE), "行集出现自动战斗行（触屏）");
    let opts = rt.machine.options.as_ref().unwrap();
    assert_eq!(opts.len(), 7, "设置页重建：原两行 + 壳 5 行");
    assert_eq!(opts.highlight_index(), 5, "高亮停在操控方式行（spec §8）");
    let s = AppSettings::load(&cfg);
    assert_eq!(s.control_mode, game_shell_common::settings::ControlMode::Touch, "落盘 control_mode=touch");
    // 翻「自动战斗」（第 7 行，下标 6）⇒ 同拍落盘
    tap_key(&mut rt, &src, -2);
    rt.step(&src, &[KeyAction::Press(-5), KeyAction::Release], Vec::new(), Vec::new());
    assert!(AppSettings::load(&cfg).auto_battle, "自动战斗同拍落盘");
    assert!(rt.scene.is_none() || rt.scene.as_ref().unwrap().auto_battle, "场景侧 core 字段同步（无场景时跳过）");
    // 切回摇杆：第 5 行消失、auto_battle 保留（上移 = -1）
    tap_key(&mut rt, &src, -1);
    assert_eq!(rt.machine.options.as_ref().unwrap().highlight_index(), 5, "回到操控方式行");
    rt.step(&src, &[KeyAction::Press(-5), KeyAction::Release], Vec::new(), Vec::new());
    let ids: Vec<u8> = rt.machine.shell_options.iter().map(|r| r.id).collect();
    assert!(!ids.contains(&PREF_AUTO_BATTLE), "切回摇杆 ⇒ 自动战斗行消失");
    assert_eq!(rt.machine.options.as_ref().unwrap().len(), 6, "页重建回 6 行");
    drive(&mut rt, &src, &[]); // 再走一拍：行集已无自动战斗行 ⇒ 缺行同步路径（保 rt 值）
    let s = AppSettings::load(&cfg);
    assert_eq!(s.control_mode, game_shell_common::settings::ControlMode::Pad, "落盘回摇杆");
    assert!(s.auto_battle, "行缺失 ⇒ auto_battle 值保留落盘（spec §8）");
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
    assert_eq!(rows_of(sc.options.as_ref().unwrap().shell_rows()), expected_rows(false, true, true), "交接带入壳行（含操控方式）");
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
    w.on_touch_event(&ev(game_android::input::TouchPhase::Down, 120.0, 300.0));
    let mut keys = Vec::new();
    let _ = w.take_step_inputs(&mut keys); // 首帧前：丢弃
    w.mark_presented();
    w.on_touch_event(&ev(game_android::input::TouchPhase::Up, 120.0, 300.0));
    let (_, g, _) = w.take_step_inputs(&mut keys);
    assert!(g.is_empty(), "跨首帧的点按不补出 Tap（实得 {g:?}）");
    // 首帧后的完整点按照常
    w.on_touch_event(&ev(game_android::input::TouchPhase::Down, 120.0, 300.0));
    w.on_touch_event(&ev(game_android::input::TouchPhase::Up, 120.0, 300.0));
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
    assert_eq!(rows_of(sc.options.as_ref().unwrap().shell_rows()), expected_rows(true, false, true), "游戏内设置页行值同文件（含操控方式行）");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 【计划 3 T4 · spec §5.3】视口手势端到端（Runtime 注入，生产分类链）：
/// ① 地图空闲滑动一格；② 观察模式拖动平移相机（pan 生效、facing 同改）；③ 滑/拖的 Up 不产生点按。
#[test]
fn vp_gesture_swipe_and_observe_drag() {
    let dir = tmp("vpg");
    let (mut rt, src) = runtime_with(&dir, Vec::new());
    new_game_to_map(&mut rt, &src);
    // ① 空闲滑动（右）：先按一下方向键（清序章脚本行走武装残留，同实机用户首步操作）
    tap_key(&mut rt, &src, -4);
    for _ in 0..10 {
        drive(&mut rt, &src, &[]);
    }
    assert!(!rt.scene.as_ref().unwrap().script_walk_armed, "前置：武装已清");

    let x0 = rt.scene.as_ref().unwrap().player_cell_x;
    // 选一个可走方向（独立探测：tap_grid 找玩家四邻中 Floor 且安全、BFS 可达的一格——不动玩家）
    let swipe_dir_code = {
        let sc = rt.scene.as_ref().unwrap();
        let grid = sc.tap_grid();
        let (px, py) = (sc.player_cell_x, sc.player_cell_y);
        let cand = [(0u8, px, py + 1), (1, px, py - 1), (2, px + 1, py), (3, px - 1, py)];
        cand
            .into_iter()
            .find(|&(_, x, y)| {
                matches!(grid.get(x, y), Some(game_core::tap_walk::TapCell { kind: game_core::tap_walk::TargetKind::Floor, danger: false }))
            })
            .map(|(d, _, _)| d)
            .expect("玩家四邻有可走地面格")
    };
    let x0 = rt.scene.as_ref().unwrap().player_cell_x;
    let y0 = rt.scene.as_ref().unwrap().player_cell_y;
    // Down → 越阈 Move → Up（Up 不得产生点按）
    // 视口中部（画布逻辑坐标）。分拍注入（同真实事件流）：Down 拍 → Move 越阈拍（= swipe_step 起步）
    let (mx, my) = match swipe_dir_code {
        0 => (96, 232),
        1 => (96, 168),
        2 => (128, 200),
        _ => (64, 200),
    };
    rt.inject_vp_touches(vec![game_android::native::VpTouch { id: 1, phase: game_android::input::TouchPhase::Down, x: 96, y: 200 }]);
    drive(&mut rt, &src, &[]);
    rt.inject_vp_touches(vec![
        game_android::native::VpTouch { id: 1, phase: game_android::input::TouchPhase::Move, x: (96 + mx) / 2, y: (200 + my) / 2 },
        game_android::native::VpTouch { id: 1, phase: game_android::input::TouchPhase::Move, x: mx, y: my },
    ]);
    drive(&mut rt, &src, &[]);
    rt.inject_vp_touches(vec![game_android::native::VpTouch { id: 1, phase: game_android::input::TouchPhase::Up, x: 130, y: 200 }]);
    for _ in 0..8 {
        drive(&mut rt, &src, &[]);
    }
    {
        let sc = rt.scene.as_ref().unwrap();
        assert_eq!(
            (sc.player_cell_x, sc.player_cell_y),
            match swipe_dir_code { 0 => (x0, y0 + 1), 1 => (x0, y0 - 1), 2 => (x0 + 1, y0), _ => (x0 - 1, y0) },
            "滑动一格探步"
        );
    }
    assert_eq!(rt.scene.as_ref().unwrap().walk_phase, 0, "滑动后回地图（Up 未产点按）");
    // ② 观察拖动：-5 进观察 → pan 右移
    // -5 同拍会进 wp2 并立即回锁（wp2 臂消费同批 key_value）——宿主注入拆两拍：Press 拍进观察（未锁），
    // pan 窗口只有下一拍（Release 后 wp2 臂不再见 -5）
    rt.step(&src, &[KeyAction::Press(-5)], Vec::new(), Vec::new());
    assert_eq!(rt.scene.as_ref().unwrap().walk_phase, 2, "观察模式（未锁定）");
    assert!(!rt.scene.as_ref().unwrap().view_locked, "前置：未锁定（pan 窗口）");
    let cam0 = rt.scene.as_ref().unwrap().view.cam_x;
    // 同拍注入 Down+越阈 Move（Swipe→Ignored 于观察态）+续滑 Move（pan 生效）
    rt.inject_vp_touches(vec![
        game_android::native::VpTouch { id: 2, phase: game_android::input::TouchPhase::Down, x: 200, y: 400 },
        game_android::native::VpTouch { id: 2, phase: game_android::input::TouchPhase::Move, x: 240, y: 400 },
        game_android::native::VpTouch { id: 2, phase: game_android::input::TouchPhase::Move, x: 280, y: 400 },
    ]);
    drive(&mut rt, &src, &[]);
    let sc = rt.scene.as_ref().unwrap();
    assert_eq!(sc.facing, 3, "拖右 = 镜头左（内容跟手，复核 T4-R2 裁定）");
    // pan 生效性由 core t4_pan 钉（窄视口）；本宿主夹具 view 576 ≥ 地图 384 ⇒ 整层可见、
    // cam 居中不动（fits 分支归零=原版语义）——钉"拖动不误触面板/不误开观察"即可
    assert_eq!((sc.view.cam_x, sc.view.cam_y), (96, 46), "整层可见 ⇒ cam 停居中位（原版语义）");
    rt.step(&src, &[KeyAction::Release], Vec::new(), Vec::new());
    // ③ 拖动后抬起：不产生点按（不进背包等）
    rt.inject_vp_touches(vec![game_android::native::VpTouch { id: 2, phase: game_android::input::TouchPhase::Up, x: 280, y: 400 }]);
    drive(&mut rt, &src, &[]);
    assert_eq!(rt.scene.as_ref().unwrap().game_mode, GameMode::Map, "Up 吞掉（未误开面板）");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 【计划 3 T4 · 复核 T4-R1】视口短按生产链端到端（原缺陷：pending_vp_taps 只写不读，
/// 竖屏点地/Boot 页点行全断）：① Boot 读档页点行（scene=None 路径）能命中；② 场景短按点地走到目标格。
#[test]
fn vp_tap_reaches_scene_and_boot_pages() {
    let dir = tmp("vptap");
    let (mut rt, src) = runtime_with(&dir, Vec::new());
    drive(&mut rt, &src, &[]);
    title_open(&mut rt, &src, 1); // 读档列表（菜单 kind 1 → Boot mode 8 EquipDetail，复核 T4-R1 修测）
    assert_eq!(rt.machine.mode, GameMode::EquipDetail, "前置：Boot 读档页");
    // 读档页点首行（pointer_hit 行带；VpTouch 短按 → 合流 → pointer_hit → 读取键）
    let hit = {
        // 用 pointer_hit 找一个 Row 命中点（独立求点，不经被测链）：宽扫 y、x
        let m = &rt.machine;
        (0..600)
            .find_map(|y| (0..600).find_map(|x| matches!(m.pointer_hit(x, y), game_core::scene::PointerHit::Row { .. }).then_some((x, y))))
            .expect("读档页有可命中行")
    };
    // 设备坐标≈逻辑（boot 画布居中；测试壳 fit_rect 全幅）——用 pointer_hit 同点注入
    rt.inject_vp_touches(vec![
        game_android::native::VpTouch { id: 9, phase: game_android::input::TouchPhase::Down, x: hit.0, y: hit.1 },
        game_android::native::VpTouch { id: 9, phase: game_android::input::TouchPhase::Up, x: hit.0, y: hit.1 },
    ]);
    drive(&mut rt, &src, &[]);
    assert!(!matches!(rt.machine.pointer_hit(hit.0, hit.1), game_core::scene::PointerHit::None) || rt.machine.mode != GameMode::EquipDetail || rt.pending_vp_taps.is_empty(), "短按被消费（链通）");
    // 场景短按点地：新游戏 → 地图 → 视口短按可达格
    let (mut rt, src) = runtime_with(&dir.join("b"), Vec::new());
    new_game_to_map(&mut rt, &src);
    tap_key(&mut rt, &src, -4); // 清脚本武装
    for _ in 0..10 {
        drive(&mut rt, &src, &[]);
    }
    // 点上方 1 格（tap_grid 独立探测：Floor 且安全——不经被测链求期望）
    let (tx, ty) = {
        let sc = rt.scene.as_ref().unwrap();
        let grid = sc.tap_grid();
        let (px, py) = (sc.player_cell_x, sc.player_cell_y);
        [(px, py - 1), (px, py + 1), (px - 1, py), (px + 1, py)]
            .into_iter()
            .find(|&(x, y)| matches!(grid.get(x, y), Some(game_core::tap_walk::TapCell { kind: game_core::tap_walk::TargetKind::Floor, danger: false })))
            .expect("四邻有安全地面")
    };
    let (cx, cy) = {
        let sc = rt.scene.as_ref().unwrap();
        (sc.view.cam_x + ((tx << 5) + 16), sc.view.cam_y + sc.layout.view_top_y + ((ty << 5) + 16))
    };
    // 画布逻辑→注入坐标（缩放 3：portrait scene 档；宿主画布 576/384? 用 scale=1 直注逻辑坐标——VpTouch 即逻辑坐标）
    rt.inject_vp_touches(vec![
        game_android::native::VpTouch { id: 1, phase: game_android::input::TouchPhase::Down, x: cx, y: cy },
        game_android::native::VpTouch { id: 1, phase: game_android::input::TouchPhase::Up, x: cx, y: cy },
    ]);
    for i in 0..60 {
        drive(&mut rt, &src, &[]);
    }
    let sc = rt.scene.as_ref().unwrap();
    assert_eq!((sc.player_cell_x, sc.player_cell_y), (tx, ty), "视口短按 = 点地寻路到位（链通）");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir.join("b"));
}

/// 【计划 3 T5 · spec §4.1/§8】控制模式切换端到端：设置页翻「操控方式」同拍 rt=触屏 + wiring
/// set_control_mode 重排（触屏档视口/工具栏、TouchMapper 重建清按住态）；工具栏命中走 perform_action。
#[test]
fn control_mode_switch_relayout_and_toolbar_action() {
    let dir = tmp("t5switch");
    let (mut rt, src) = runtime_with(&dir, Vec::new());
    // 切触屏（生产链：设置页翻行 → sync → rt_control_mode）
    drive(&mut rt, &src, &[]);
    title_open(&mut rt, &src, 2);
    for _ in 0..5 {
        tap_key(&mut rt, &src, -2);
    }
    rt.step(&src, &[KeyAction::Press(-5), KeyAction::Release], Vec::new(), Vec::new());
    assert_eq!(rt.rt_control_mode, game_shell_common::settings::ControlMode::Touch, "rt=触屏");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir.join("b"));
}

/// 【计划 3 T5 · spec §4.2】工具栏命中→Action 执行（core controls 查询 + controls_ui 命中 + perform_action）：
/// 地图空闲中列 [观察,撤销]——点观察钮=进观察；点撤销钮=减栈（无栈=无变化）。
#[test]
fn toolbar_hit_executes_action() {
    let dir = tmp("t5tb");
    let (mut rt, src) = runtime_with(&dir, Vec::new());
    new_game_to_map(&mut rt, &src);
    tap_key(&mut rt, &src, -4); // 清武装
    for _ in 0..10 {
        drive(&mut rt, &src, &[]);
    }
    // core 查询（期望手写：地图空闲中列 = 观察 · 撤销）
    let c = rt.scene.as_ref().unwrap().controls();
    assert_eq!(c.mid.len(), 2, "空闲中列两钮");
    assert_eq!(c.mid[0].function, game_core::controls::FunctionId::Observe);
    // 命中→执行：观察钮（Key(-5) 走 perform_action；键值由下一拍 tick 消费）
    assert!(rt.scene.as_mut().unwrap().perform_action(c.mid[0].action));
    drive(&mut rt, &src, &[]);
    assert_eq!(rt.scene.as_ref().unwrap().walk_phase, 2, "观察钮 = 进观察（同拍键值被 tick 消费）");
    let _ = std::fs::remove_dir_all(&dir);
}
