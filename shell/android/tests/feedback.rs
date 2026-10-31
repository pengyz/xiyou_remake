//! 【2026-10-26 虚拟键按下反馈】宿主钉：哪些触点算"虚拟键按下"、反馈策略、运行时开关读数。用户："虚拟按键按的时候
//! 触发震动和按键声音，现在操作无反馈"。纯壳层（core 不知情）；JNI 出口（View.performHapticFeedback /
//! AudioManager.playSoundEffect）在 feedback::android，实机 logcat 验证。
//! > 【2026-10-28 更正 · STAGE-B-006】策略由"触感恒发、按键音随游戏内'声音'"改为"触感看'按键震动'、按键音看'按键音'"
//! > （用户裁定"声音和按键音是分开的"）；对应测试改写，见 `cue_policy_haptic_and_click_follow_their_own_switches`。

use game_android::feedback::{emit, presses_in, Cue, RecordingFeedback};
use game_shell_common::settings::ControlMode;
use game_android::input::{KeyAction, TouchEvent, TouchPhase};
use game_android::native::ShellWiring;
use game_android::portrait::{plan_portrait, MapExtent, PlanInput};

fn portrait_wiring() -> ShellWiring {
    let mut w = ShellWiring::new();
    w.apply_portrait(plan_portrait(&PlanInput {
        window: (1200, 2536),
        dpi: 480,
        inset_top: 0,
        inset_bottom: 0,
        map: MapExtent { cells_w: 13, cells_h: 13 },
        control_mode: game_shell_common::settings::ControlMode::Pad,
    }));
    w
}

#[test]
fn presses_in_counts_press_actions_only() {
    assert_eq!(presses_in(&[]), 0);
    assert_eq!(presses_in(&[KeyAction::Release]), 0, "抬起不计");
    assert_eq!(presses_in(&[KeyAction::Release, KeyAction::Press(-1)]), 1, "十字键拖动换向 = 一次按下");
    assert_eq!(presses_in(&[KeyAction::Press(-5), KeyAction::Press(-4)]), 2);
}

#[test]
fn panel_virtual_key_downs_raise_presses_hold_and_viewport_do_not() {
    let mut w = portrait_wiring();
    let p = w.portrait.clone().unwrap();
    let (px, py) = (p.panel.x as f32, p.panel.y as f32);
    let g = &p.geom;
    let (cx, cy) = (px + g.dpad.0, py + g.dpad.1);
    w.on_touch_device(1, TouchPhase::Down, cx + g.cell, cy);
    assert_eq!(w.take_key_presses(), 1, "按下右臂 = 一次反馈");
    w.on_touch_device(1, TouchPhase::Move, cx + g.cell * 1.2, cy + 2.0);
    assert_eq!(w.take_key_presses(), 0, "按住同向漂移不重复");
    w.on_touch_device(1, TouchPhase::Move, cx, cy - g.cell);
    assert_eq!(w.take_key_presses(), 1, "拖到上臂换向 = 一次反馈");
    w.on_touch_device(1, TouchPhase::Up, cx, cy - g.cell);
    assert_eq!(w.take_key_presses(), 0, "抬起无反馈");
    w.on_touch_device(2, TouchPhase::Down, cx, cy);
    w.on_touch_device(2, TouchPhase::Up, cx, cy);
    assert_eq!(w.take_key_presses(), 0, "盘心死区无键 = 无反馈");
    w.on_touch_device(3, TouchPhase::Down, px + g.ok.0, py + g.ok.1);
    let s = g.soft[0];
    w.on_touch_device(4, TouchPhase::Down, px + (s.x0 + s.x1) / 2.0, py + (s.y0 + s.y1) / 2.0);
    assert_eq!(w.take_key_presses(), 2, "确定 + 左软键（多指）各一次");
    assert_eq!(w.take_key_presses(), 0, "取走即清");
    // 视口点按（游戏态 / 标题态）不是虚拟键
    w.set_active_canvas(p.scene);
    let c = p.scene.rect_in(p.game);
    let (vx, vy) = (c.x as f32 + 50.0, c.y as f32 + 50.0);
    for title in [false, true] {
        w.title_mode = title;
        w.on_touch_device(5, TouchPhase::Down, vx, vy);
        w.on_touch_device(5, TouchPhase::Up, vx, vy);
        assert_eq!(w.take_key_presses(), 0, "视口点按无反馈（title={title}）");
    }
}

#[test]
fn landscape_canvas_pad_presses_count_too() {
    // 横屏旧路径（画布内键区 PadLayout）同一口径：键区按下计、画布 UI 点按不计
    let mut w = ShellWiring::new();
    let s = w.pad.soft[0];
    let (sx, sy) = ((s.x0 + s.x1) / 2.0, (s.y0 + s.y1) / 2.0);
    let ev = |id, phase, x, y| TouchEvent { id, phase, x, y };
    w.on_touch_event(&ev(1, TouchPhase::Down, sx, sy));
    assert_eq!(w.take_key_presses(), 1, "画布内软键按下");
    w.on_touch_event(&ev(1, TouchPhase::Up, sx, sy));
    w.on_touch_event(&ev(2, TouchPhase::Down, 5.0, 5.0));
    w.on_touch_event(&ev(2, TouchPhase::Up, 5.0, 5.0));
    assert_eq!(w.take_key_presses(), 0, "键区外点按 = UI 点按，非虚拟键");
}

/// 【STAGE-B-006】策略改写（原 `cue_policy_haptic_always_click_follows_game_sound_switch`：触感恒发、按键音随游戏内"声音"）：
/// 触感看设置页"按键震动"、按键音看"按键音"，与"声音"互不影响（用户裁定"声音和按键音是分开的"）；两项全关 ⇒ 无反馈。
#[test]
fn cue_policy_haptic_and_click_follow_their_own_switches() {
    let mut out = RecordingFeedback::default();
    emit(&mut out, 0, true, true);
    assert!(out.cues.is_empty(), "无按下不反馈");
    for presses in [1, 2] {
        for (haptic, click) in [(true, true), (true, false), (false, true), (false, false)] {
            emit(&mut out, presses, haptic, click);
        }
    }
    let three = vec![
        Cue { haptic: true, click: true },
        Cue { haptic: true, click: false },
        Cue { haptic: false, click: true },
    ];
    assert_eq!(
        out.cues,
        [three.clone(), three].concat(),
        "单次 / 多次按下各一次反馈（同一事件合并）；两开关各自独立；两项全关 ⇒ 无反馈"
    );
}

/// 【STAGE-B-006】反馈开关读活动机的"按键震动 / 按键音"两行：开场链读 BootMachine、交接后读 GameScene（设置页切换即生效）；
/// 无附加行（未接设置文件）⇒ 两项默认开；游戏内"声音"不参与。
#[test]
fn runtime_key_feedback_reads_active_machine() {
    use game_android::runtime::{build_boot_machine, Runtime, PREF_CLICK, PREF_HAPTIC};
    use game_core::menu_family::set_shell_option;
    let raw = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw");
    let src = game_shell_common::asset::DirAssetSource::new(&raw);
    let dir = std::env::temp_dir().join(format!("jjxy-feedback-keys-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (rms, stores) = game_shell_common::rms::FileRms::load(&dir);
    let mut rt = Runtime::new(build_boot_machine(&src, game_core::layout::Layout::modern(12, 16)), stores, Box::new(rms));
    assert_eq!(rt.key_feedback(), (true, true), "无附加行：两项默认开");
    rt.apply_app_settings(&game_shell_common::settings::AppSettings { key_haptic: false, ..Default::default() });
    assert_eq!(rt.key_feedback(), (false, true), "开场链：读 BootMachine 两行");
    rt.machine.bgm.sound_enabled = false;
    assert_eq!(rt.key_feedback(), (false, true), "声音关不影响按键音");
    rt.machine.new_game_ready = true;
    rt.machine.mode = game_core::enums::GameMode::Map;
    rt.step(&src, &[], Vec::new(), Vec::new());
    let sc = rt.scene.as_mut().expect("交接");
    set_shell_option(&mut sc.shell_options, PREF_HAPTIC, true);
    set_shell_option(&mut sc.shell_options, PREF_CLICK, false);
    assert_eq!(rt.key_feedback(), (true, false), "游戏中：读 GameScene");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn runtime_sound_enabled_reads_active_machine() {
    use game_android::runtime::{build_boot_machine, Runtime};
    let raw = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw");
    let src = game_shell_common::asset::DirAssetSource::new(&raw);
    let dir = std::env::temp_dir().join(format!("jjxy-feedback-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (rms, stores) = game_shell_common::rms::FileRms::load(&dir);
    let mut rt = Runtime::new(build_boot_machine(&src, game_core::layout::Layout::modern(12, 16)), stores, Box::new(rms));
    rt.machine.bgm.sound_enabled = false;
    assert!(!rt.sound_enabled(), "开场链：读 BootMachine");
    rt.machine.bgm.sound_enabled = true;
    assert!(rt.sound_enabled());
    rt.machine.new_game_ready = true;
    rt.machine.mode = game_core::enums::GameMode::Map;
    rt.step(&src, &[], Vec::new(), Vec::new());
    rt.scene.as_mut().expect("交接").bgm.sound_enabled = false;
    assert!(!rt.sound_enabled(), "游戏中：读 GameScene（设置里关声音即生效）");
    let _ = std::fs::remove_dir_all(&dir);
}
