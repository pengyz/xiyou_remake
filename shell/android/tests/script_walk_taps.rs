//! 【2026-10-26 用户实机"跳过教程后，嫦娥脚本行为异常"】Android 运行时全链（runtime.rs 双机调度 +
//! native::replay_scene_keys 壳层投键）跑一层嫦娥走位戏 script19：新游戏 → 菜单"跳过教程" →
//! 上(6,10) → 左拾黄钥匙 → 右 → 上开黄门进 (6,9) ⇒ 到达触发 script19。状态驱动输入（与拍时序解耦）。
//!
//! 根因（已修，core `GameScene::interrupt_route`）：壳层每次投键先 `interrupt_route()`，旧实现无条件
//! 清 walkPathBuffer ⇒ 剧情里连点"确定"截断嫦娥 MOV 路径/玩家脚本行走，嫦娥停在半路、后续
//! MOV/DES 落空。Java 锚：crates/game-oracle fx-chang-e-floor1-taps（壳层投键口径逐帧 = Java）。

use game_android::input::KeyAction;
use game_android::native::{canvas_graphics, TitleGesture};
use game_android::runtime::{build_boot_machine, Runtime};
use game_core::enums::GameMode;
use game_core::layout::Layout;
use game_core::render::ArgbImage;

fn tap(code: i32) -> Vec<KeyAction> {
    vec![KeyAction::Press(code), KeyAction::Release]
}

/// 逻辑事件行：层/模式/脚本游标/页/对话相位/玩家格+像素余/嫦娥格+像素余+态（与相机、视口无关）。
fn event_line(rt: &Runtime) -> String {
    let Some(sc) = rt.scene.as_ref() else { return "boot".into() };
    let e84: Vec<String> = (0..sc.entities.count)
        .map(|i| &sc.entities.items[i])
        .filter(|e| e.type_ == 84)
        .map(|e| format!("({},{})+({},{})s{}", e.px >> 5, e.py >> 5, e.px & 31, e.py & 31, e.state))
        .collect();
    format!(
        "f{} {:?} s{}@{} pg{} ph{} pl({},{})+({},{}) e84{:?}",
        sc.floor,
        sc.game_mode,
        sc.current_script_index,
        sc.script_cursor,
        sc.dialog_page,
        sc.dialog_phase,
        sc.view.player_px >> 5,
        sc.view.player_py >> 5,
        sc.view.player_px & 31,
        sc.view.player_py & 31,
        e84
    )
}

/// 跑到 script19 结束（或 max_ticks），返回去重事件序列。
/// `tap_while_walking`：剧情行走期（dialogPhase 2 嫦娥路径 / 3 玩家脚本行走）也每 4 拍点"确定"
/// （= 用户在剧情里习惯性连点）；文字页（phase 1）两种口径都点。
fn run_script19(scene_layout: Layout, tap_while_walking: bool) -> Vec<String> {
    let raw = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw");
    let src = game_shell_common::asset::DirAssetSource::new(&raw);
    let dir = std::env::temp_dir().join(format!(
        "jjxy-s19-{}-{}-{}",
        std::process::id(),
        scene_layout.screen_w,
        tap_while_walking
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let (rms, stores) = game_shell_common::rms::FileRms::load(&dir);
    let boot = Layout::modern(7, 10);
    let mut rt = Runtime::new(build_boot_machine(&src, boot.clone()), stores, Box::new(rms));
    rt.apply_layouts(boot, scene_layout.clone());
    let mut boot_canvas = ArgbImage::create(240, 320);
    let mut scene_canvas = ArgbImage::create(scene_layout.screen_w, scene_layout.screen_h);
    let mut step = |rt: &mut Runtime, keys: Vec<KeyAction>, g: Vec<TitleGesture>, t: Vec<(f32, f32)>| {
        rt.step(&src, &keys, g, t);
        let c = if rt.scene.is_some() { &mut scene_canvas } else { &mut boot_canvas };
        let mut gfx = canvas_graphics(c, 1);
        rt.paint(&mut gfx); // 绘制不可省：换层遮幅推进挂 paint 公共尾
    };
    // 开场链：标题 → 开始 → 序章快进 → 交接 → 序章对话推进到首个 Map
    let mut n = 0;
    while !rt.title_mode() && n < 400 {
        step(&mut rt, vec![], vec![], vec![]);
        n += 1;
    }
    let geo = rt.machine.title_geom().expect("标题");
    let ok = ((geo.zone_ok.0 + geo.zone_ok.2 / 2) as f32, (geo.zone_ok.1 + geo.zone_ok.3 / 2) as f32);
    step(&mut rt, vec![], vec![TitleGesture::Tap(ok.0, ok.1)], vec![]);
    n = 0;
    while rt.scene.is_none() && n < 4000 {
        step(&mut rt, vec![], vec![], if n % 3 == 0 { vec![(120.0, 160.0)] } else { vec![] });
        n += 1;
    }
    n = 0;
    while rt.scene.as_ref().expect("交接").game_mode != GameMode::Map && n < 4000 {
        step(&mut rt, if n % 4 == 0 { tap(-5) } else { vec![] }, vec![], vec![]);
        n += 1;
    }
    for _ in 0..4 {
        step(&mut rt, vec![], vec![], vec![]);
    }
    // 【计划 3 T7】首次到地图提示卡（浮层吞键）——统一关框后再交后续操作
    if rt.scene.as_ref().unwrap().overlay_active {
        step(&mut rt, tap(-5), vec![], vec![]);
        step(&mut rt, vec![], vec![], vec![]);
    }
    // 菜单 → 跳过教程（floor>50 菜单序 [1,17,3,4,5,7]，索引 1）
    step(&mut rt, tap(-7), vec![], vec![]);
    step(&mut rt, vec![], vec![], vec![]);
    step(&mut rt, tap(-2), vec![], vec![]);
    step(&mut rt, vec![], vec![], vec![]);
    assert_eq!(rt.scene.as_ref().unwrap().menu_cursor, 1, "菜单光标在'跳过教程'");
    step(&mut rt, tap(-5), vec![], vec![]);
    // 一格一按：按下 → 起步即抬起
    let route = [-1, -3, -4, -1];
    let (mut leg, mut holding) = (0usize, false);
    let mut seq: Vec<String> = Vec::new();
    for k in 0..4000 {
        let sc = rt.scene.as_ref().unwrap();
        if sc.script_line_flags[19] && sc.game_mode == GameMode::Map {
            break; // script19 收尾回 Map
        }
        let keys = if holding {
            if sc.walk_phase != 0 || sc.game_mode != GameMode::Map {
                holding = false;
                leg += 1;
                vec![KeyAction::Release]
            } else {
                vec![]
            }
        } else if leg < route.len()
            && sc.game_mode == GameMode::Map
            && !sc.transitioning
            && sc.walk_phase == 0
            && !sc.overlay_active
            && k % 3 == 0
        {
            holding = true;
            vec![KeyAction::Press(route[leg])]
        } else if sc.game_mode == GameMode::Map && sc.overlay_active && k % 4 == 0 {
            tap(-5)
        } else if sc.game_mode == GameMode::Dialog && k % 4 == 0 && (sc.dialog_phase == 1 || (tap_while_walking && matches!(sc.dialog_phase, 2 | 3))) {
            tap(-5)
        } else {
            vec![]
        };
        step(&mut rt, keys, vec![], vec![]);
        let l = event_line(&rt);
        if seq.last() != Some(&l) {
            seq.push(l);
        }
        assert!(!rt.crashed, "未崩");
    }
    let _ = std::fs::remove_dir_all(&dir);
    seq
}

fn portrait() -> Layout {
    let mut l = Layout::modern(12, 16); // Xiaomi 14 竖屏场景档
    l.softkey_bar_visible = false;
    l
}

/// 原版剧情的关键里程碑（Java 锚 fx-chang-e-floor1-taps 同链）。
fn assert_story_milestones(seq: &[String], tag: &str) {
    let has = |pat: &str| seq.iter().any(|l| l.contains(pat));
    assert!(has("s19@") && has(r#"e84["(6,8)+(0,0)s"#), "{tag}：嫦娥走到 (6,8)");
    assert!(has(r#"e84["(1,8)+(0,0)s0"]"#), "{tag}：嫦娥走到 (1,8)");
    assert!(has(r#"e84["(1,1)+(0,0)s0"]"#), "{tag}：嫦娥走到 (1,1)");
    assert!(has(r#"e84["(10,1)+(0,0)s1"]"#), "{tag}：嫦娥到 (10,1) 后被 DES 移除");
    assert!(has("f2 "), "{tag}：玩家脚本行走上 (11,1) 楼梯 → 2 层续跑");
}

#[test]
fn script19_taps_during_scripted_walks_do_not_derail() {
    let (calm, tapped) = std::thread::scope(|s| {
        let a = s.spawn(|| run_script19(portrait(), false));
        let b = s.spawn(|| run_script19(portrait(), true));
        (a.join().unwrap(), b.join().unwrap())
    });
    assert_story_milestones(&calm, "竖屏·不点");
    assert_story_milestones(&tapped, "竖屏·行走中连点");
    assert_eq!(tapped, calm, "剧情行走中连点'确定'不得改变剧情走向（事件序列逐条一致）");
}

#[test]
fn script19_event_sequence_independent_of_logical_resolution() {
    // 用户问"脚本和逻辑分辨率有关系吗？"——剧情顺序/路径/位置与逻辑分辨率无关（状态驱动输入）
    let (classic, modern) = std::thread::scope(|s| {
        let a = s.spawn(|| run_script19(Layout::classic(), true));
        let b = s.spawn(|| run_script19(portrait(), true));
        (a.join().unwrap(), b.join().unwrap())
    });
    assert_story_milestones(&classic, "classic 240×320");
    assert_eq!(modern, classic, "竖屏 12×16 与 classic 事件序列逐条一致");
}
