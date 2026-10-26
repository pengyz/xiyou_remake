//! 【2026-10-26 进游戏全流程宿主钉】Android 壳运行时（runtime.rs，与 android_main 同一
//! 调度体）在宿主上用真资产跑：开机 → 标题（触屏轮播点"开始游戏"）→ 序章（-7 跳过）→
//! 读条 → **Boot→GameScene 交接** → 序章剧情对话 → Map 行走 → 游戏内菜单。
//! 实机根因：壳缺交接 ⇒ 读条完 mode=Map 无人接管、tick_mode0 回落标题（设备 BGM 3→2）。

use game_android::input::KeyAction;
use game_android::native::TitleGesture;
use game_android::runtime::{build_boot_machine, Runtime};
use game_core::enums::GameMode;

fn raw() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw")
}

fn tmp(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("jjxy-flow-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn runtime(tag: &str) -> (Runtime, game_shell_common::asset::DirAssetSource, std::path::PathBuf) {
    let src = game_shell_common::asset::DirAssetSource::new(&raw());
    let dir = tmp(tag);
    let (rms, stores) = game_shell_common::rms::FileRms::load(&dir);
    let machine = build_boot_machine(&src, game_core::layout::Layout::modern(18, 18));
    (Runtime::new(machine, stores, Box::new(rms)), src, dir)
}

/// 一拍 = step + paint（设备主循环每拍 tick→paint 同构：序章字幕/菜单排版在 paint 内
/// 完成，tick 读上次 paint 的布局锚——不画就永不推进）。
fn drive(
    rt: &mut Runtime,
    src: &dyn game_platform::AssetSource,
    keys: &[KeyAction],
    gestures: Vec<TitleGesture>,
    taps: Vec<(f32, f32)>,
) {
    rt.step(src, keys, gestures, taps);
    let mut canvas = game_core::render::ArgbImage::create(576, 576);
    let mut g = game_android::native::canvas_graphics(&mut canvas, 1);
    rt.paint(&mut g);
}

/// 推进至条件成立（上限 n 拍；`keys` 每拍投递同一批）。
fn run_until(
    rt: &mut Runtime,
    src: &dyn game_platform::AssetSource,
    n: usize,
    keys: &[KeyAction],
    mut done: impl FnMut(&Runtime) -> bool,
) -> bool {
    for _ in 0..n {
        if done(rt) {
            return true;
        }
        drive(rt, src, keys, Vec::new(), Vec::new());
    }
    done(rt)
}

#[test]
fn new_game_flow_reaches_scene_and_map() {
    let (mut rt, src, dir) = runtime("newgame");
    assert!(run_until(&mut rt, &src, 400, &[], |r| r.title_mode()), "开机 → 标题（Modern 跳过声音询问）");
    let geo = rt.machine.title_geom().unwrap();
    let ok = ((geo.zone_ok.0 + geo.zone_ok.2 / 2) as f32, (geo.zone_ok.1 + geo.zone_ok.3 / 2) as f32);
    drive(&mut rt, &src, &[], vec![TitleGesture::Tap(ok.0, ok.1)], Vec::new());
    assert_eq!(rt.machine.mode, GameMode::IntroCutscene, "点标签=开始游戏 → 序章");
    drive(&mut rt, &src, &[KeyAction::Press(-7), KeyAction::Release], Vec::new(), Vec::new());
    assert!(
        run_until(&mut rt, &src, 400, &[], |r| r.scene.is_some()),
        "序章跳过 → 读条 → 交接 GameScene（旧壳此处回落标题）"
    );
    assert_eq!(rt.scene.as_ref().unwrap().floor, 51, "新游戏序章层 51");
    assert!(!rt.title_mode(), "场景活动时非标题态");
    assert_eq!(rt.bgm_request(), Some(3), "游戏曲 track3（场景接管 BGM 请求）");
    // 序章剧情对话：逐拍确认推进，稳定落 Map
    let mut calm = 0;
    for i in 0..4000 {
        let mode = rt.scene.as_ref().unwrap().game_mode;
        if mode == GameMode::Map {
            calm += 1;
            if calm >= 30 {
                break;
            }
        } else {
            calm = 0;
        }
        let keys: &[KeyAction] = if mode != GameMode::Map && i % 4 == 0 {
            &[KeyAction::Press(-5), KeyAction::Release]
        } else {
            &[]
        };
        drive(&mut rt, &src, keys, Vec::new(), Vec::new());
    }
    assert_eq!(rt.scene.as_ref().unwrap().game_mode, GameMode::Map, "剧情推完 → Map 行走态");
    assert!(!rt.crashed, "全程无 game-core panic");
    // 方向键按住行走：玩家位置变化
    let before = {
        let sc = rt.scene.as_ref().unwrap();
        (sc.player_cell_x, sc.player_cell_y)
    };
    let mut moved = false;
    for dir in [-4, -3, -2, -1] {
        drive(&mut rt, &src, &[KeyAction::Press(dir)], Vec::new(), Vec::new());
        for _ in 0..12 {
            drive(&mut rt, &src, &[], Vec::new(), Vec::new());
        }
        drive(&mut rt, &src, &[KeyAction::Release], Vec::new(), Vec::new());
        for _ in 0..8 {
            drive(&mut rt, &src, &[], Vec::new(), Vec::new());
        }
        let sc = rt.scene.as_ref().unwrap();
        if (sc.player_cell_x, sc.player_cell_y) != before || sc.game_mode != GameMode::Map {
            moved = true;
            break;
        }
    }
    assert!(moved, "按住方向键 = 行走（或触发剧情）");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 点按一次（Press+Release 同批，即点按语义）。
fn tap_key(rt: &mut Runtime, src: &dyn game_platform::AssetSource, code: i32) {
    drive(rt, src, &[KeyAction::Press(code), KeyAction::Release], Vec::new(), Vec::new());
    for _ in 0..3 {
        drive(rt, src, &[], Vec::new(), Vec::new());
    }
}

/// 新游戏直达 Map（标题点"开始游戏" → -7 跳序章 → 交接 → 推完剧情）。
fn reach_map(rt: &mut Runtime, src: &dyn game_platform::AssetSource) {
    assert!(run_until(rt, src, 400, &[], |r| r.title_mode()), "到标题");
    let geo = rt.machine.title_geom().unwrap();
    while rt.machine.title_cursor() != Some(0) {
        let p = ((geo.zone_prev.0 + geo.zone_prev.2 / 2) as f32, (geo.zone_prev.1 + geo.zone_prev.3 / 2) as f32);
        drive(rt, src, &[], vec![TitleGesture::Tap(p.0, p.1)], Vec::new());
    }
    let ok = ((geo.zone_ok.0 + geo.zone_ok.2 / 2) as f32, (geo.zone_ok.1 + geo.zone_ok.3 / 2) as f32);
    drive(rt, src, &[], vec![TitleGesture::Tap(ok.0, ok.1)], Vec::new());
    drive(rt, src, &[KeyAction::Press(-7), KeyAction::Release], Vec::new(), Vec::new());
    assert!(run_until(rt, src, 400, &[], |r| r.scene.is_some()), "交接 GameScene");
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
        drive(rt, src, keys, Vec::new(), Vec::new());
    }
    panic!("剧情未推完");
}

#[test]
fn save_exit_to_title_then_continue_loads_slot() {
    let (mut rt, src, dir) = runtime("save");
    reach_map(&mut rt, &src);
    // 游戏内菜单（-7）：条目 [继续, 跳过教程(层>50), 保存, 设置, 帮助, 退出]
    tap_key(&mut rt, &src, -7);
    assert_eq!(rt.scene.as_ref().unwrap().game_mode, GameMode::InGameMenu, "-7 开游戏内菜单");
    let kinds = rt.scene.as_ref().unwrap().menu_kinds.clone();
    let idx = |k: i8| kinds.iter().position(|&x| x == k).expect("菜单条目在");
    for _ in 0..idx(3) {
        tap_key(&mut rt, &src, -2);
    }
    tap_key(&mut rt, &src, -6); // 保存 → 存档槽（mode 7）
    assert_eq!(rt.scene.as_ref().unwrap().game_mode, GameMode::SaveSlot, "进存档槽");
    tap_key(&mut rt, &src, -6); // 存入槽 0
    assert!(rt.stores.iter().any(|(k, _)| k == "MOT_L0@1"), "槽 0 存档写穿 stores");
    assert!(dir.join("MOT_L0.bin").exists(), "存档落盘（FileRms {{店}}.bin）");
    // 退回菜单 → 退出（kind 7）→ 换回标题
    while rt.scene.as_ref().unwrap().game_mode != GameMode::InGameMenu {
        tap_key(&mut rt, &src, -7);
    }
    let cur = rt.scene.as_ref().unwrap().menu_cursor as usize;
    let steps = (idx(7) + kinds.len() - cur) % kinds.len();
    for _ in 0..steps {
        tap_key(&mut rt, &src, -2);
    }
    tap_key(&mut rt, &src, -6);
    assert!(rt.scene.is_none(), "退出 → 换回 BootMachine");
    assert!(rt.title_mode(), "回到标题");
    for _ in 0..20 {
        drive(&mut rt, &src, &[], Vec::new(), Vec::new());
    }
    assert!(rt.scene.is_none(), "交接信号已消费：回标题后不自动再进新游戏（旧 win 壳 bug）");
    // 标题 → 继续游戏（kind1=读档列表）→ 确认槽 0 → 读条 → 读档交接
    let geo = rt.machine.title_geom().unwrap();
    while rt.machine.title_cursor() != Some(1) {
        let n = ((geo.zone_next.0 + geo.zone_next.2 / 2) as f32, (geo.zone_next.1 + geo.zone_next.3 / 2) as f32);
        drive(&mut rt, &src, &[], vec![TitleGesture::Tap(n.0, n.1)], Vec::new());
    }
    let ok = ((geo.zone_ok.0 + geo.zone_ok.2 / 2) as f32, (geo.zone_ok.1 + geo.zone_ok.3 / 2) as f32);
    drive(&mut rt, &src, &[], vec![TitleGesture::Tap(ok.0, ok.1)], Vec::new());
    assert_eq!(rt.machine.mode, GameMode::EquipDetail, "继续游戏 → 读档槽列表");
    tap_key(&mut rt, &src, -5);
    assert!(run_until(&mut rt, &src, 400, &[], |r| r.scene.is_some()), "读条 → 读档交接 GameScene");
    assert_eq!(rt.scene.as_ref().unwrap().floor, 51, "读回存档层号");
    assert!(!rt.crashed);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tap_anywhere_advances_dialog_and_intro() {
    // 开局剧情对话：画布内任意点按=确认 -5（PC click_key 基线同源）⇒ 只靠点屏幕即可推完
    let (mut rt, src, dir) = runtime("tapdialog");
    assert!(run_until(&mut rt, &src, 400, &[], |r| r.title_mode()), "到标题");
    let geo = rt.machine.title_geom().unwrap();
    let ok = ((geo.zone_ok.0 + geo.zone_ok.2 / 2) as f32, (geo.zone_ok.1 + geo.zone_ok.3 / 2) as f32);
    drive(&mut rt, &src, &[], vec![TitleGesture::Tap(ok.0, ok.1)], Vec::new());
    assert_eq!(rt.machine.mode, GameMode::IntroCutscene);
    // 序章：点屏幕=快进（-5 按键快进臂 a.java:4868），全程只点屏幕也能走完序章
    let mut taps = 0;
    while rt.scene.is_none() && taps < 3000 {
        let t = if taps % 3 == 0 { vec![(288.0, 300.0)] } else { Vec::new() };
        drive(&mut rt, &src, &[], Vec::new(), t);
        taps += 1;
    }
    assert!(rt.scene.is_some(), "只点屏幕：序章 → 读条 → 进游戏（{taps} 拍）");
    let mut n = 0;
    let mut calm = 0;
    while n < 4000 {
        let mode = rt.scene.as_ref().unwrap().game_mode;
        calm = if mode == GameMode::Map { calm + 1 } else { 0 };
        if calm >= 30 {
            break;
        }
        let t = if mode != GameMode::Map && n % 4 == 0 { vec![(288.0, 300.0)] } else { Vec::new() };
        drive(&mut rt, &src, &[], Vec::new(), t);
        n += 1;
    }
    assert_eq!(rt.scene.as_ref().unwrap().game_mode, GameMode::Map, "只点屏幕推完开局剧情");
    let _ = std::fs::remove_dir_all(&dir);
}
