//! 单元 1 契约测试：触控/硬件键 → 游戏键码映射（TDD RED 先行）。
//!
//! 真值锚：
//! - 键码面 docs/spec/key-code-contract.md §4：-1 上/-2 下/-3 左/-4 右/-5 探索确认/
//!   -6 确认使用（左软）/-7 返回退出（右软）；与 shell/win `key_code()` 同表
//!   （shell/win/src/main.rs:222-235，Enter|Space→-5）。
//! - **折叠单键状态机**（实现硬门，对抗审查 v2 复审口径修正）：多点触控折叠进
//!   单一键状态机，**禁止 per-role/per-pointer 独立 held**——权威锚 scene.rs:4270-4274
//!   （press_key = key_value = key_held = code 覆盖式取末键）+ :4276-4279
//!   （release_key 只清 key_held=0，不认键）。按住方向盘(-1)再按软键(-5)⇒折叠为
//!   key_value=key_held=-5（行走即停）；松任一**持键**触点 ⇒ key_held=0 且
//!   **行走不自动恢复**（"松开软键恢复行走"= 发明行为 = 禁）。
//! - BACK 键吞默认（BUG-020 退出链：退出只经 m_000 信号 → shell 消费；
//!   BACK 不消费 ⇒ 系统 finish Activity ⇒ 绕过退出链）。
//! - 隐藏期输入不投递（MIDP Canvas「Event Delivery」；shell/win lifecycle.rs:26-28）。
//! - OS 键重复丢弃（docs/knowledge/gotcha_os-key-repeat-is-not-keypressed.md）。
//!
//! 按区域定角色只管"触点→键码翻译"，不决定状态共存：本模块的 per-pointer 记录
//! 仅是"该触点翻译出过哪个键码"的翻译簿（Move/Up 需要知道），不是独立 held 态。

use game_android::input::{
    map_hw_key, DpadZone, HwKey, InputRouter, KeyAction, KeyMapResult, PadLayout, SoftZone,
    TouchEvent, TouchMapper, TouchPhase, KEY_DOWN, KEY_FIRE, KEY_LEFT, KEY_LEFT_SOFT, KEY_RIGHT,
    KEY_RIGHT_SOFT, KEY_UP,
};

/// scene.rs:4270-4279 语义的测试镜像（真消费在 game-core `press_key`/`release_key`）。
#[derive(Default)]
struct FakeGame {
    key_value: i32,
    key_held: i32,
}

impl FakeGame {
    /// scene.rs:4270-4274：按下 = key_value = key_held = code（覆盖式取末键）。
    fn press(&mut self, code: i32) {
        self.key_value = code;
        self.key_held = code;
    }
    /// scene.rs:4276-4279：抬起 = 只清 key_held = 0（不认键、不动 key_value）。
    fn release(&mut self) {
        self.key_held = 0;
    }
    fn apply(&mut self, acts: &[KeyAction]) {
        for a in acts {
            match a {
                KeyAction::Press(c) => self.press(*c),
                KeyAction::Release => self.release(),
            }
        }
    }
}

/// 测试布局：盘心 (0,0)、死区 r=10、迟滞切换 r=30；三软键横排 y∈[100,140)。
fn layout() -> PadLayout {
    PadLayout {
        dpad: DpadZone { cx: 0.0, cy: 0.0, r_inner: 10.0, r_switch: 30.0 },
        soft: vec![
            SoftZone { code: KEY_LEFT_SOFT, x0: 100.0, y0: 100.0, x1: 140.0, y1: 140.0 },
            SoftZone { code: KEY_FIRE, x0: 150.0, y0: 100.0, x1: 190.0, y1: 140.0 },
            SoftZone { code: KEY_RIGHT_SOFT, x0: 200.0, y0: 100.0, x1: 240.0, y1: 140.0 },
        ],
    }
}

fn touch(id: i32, phase: TouchPhase, x: f32, y: f32) -> TouchEvent {
    TouchEvent { id, phase, x, y }
}

// ---------------------------------------------------------------- 硬件键

/// BACK → -7 且**吞默认**（consumed=true：系统不得 finish Activity）。
#[test]
fn back_maps_to_minus7_and_consumed() {
    let r = map_hw_key(HwKey::Back, false);
    assert_eq!(r, KeyMapResult { code: Some(KEY_RIGHT_SOFT), consumed: true });
}

/// BACK 长按产生的 OS repeat 事件**丢弃**（不投递键）但**仍吞默认**。
#[test]
fn back_repeat_dropped_but_still_consumed() {
    let r = map_hw_key(HwKey::Back, true);
    assert_eq!(r, KeyMapResult { code: None, consumed: true });
}

/// 硬件键全表：DPAD→-1..-4、CENTER/ENTER/SPACE→-5、MENU→-6（与 win 同表）。
#[test]
fn hardware_key_table_matches_contract() {
    for (k, code) in [
        (HwKey::DpadUp, KEY_UP),
        (HwKey::DpadDown, KEY_DOWN),
        (HwKey::DpadLeft, KEY_LEFT),
        (HwKey::DpadRight, KEY_RIGHT),
        (HwKey::DpadCenter, KEY_FIRE),
        (HwKey::Enter, KEY_FIRE),
        (HwKey::Space, KEY_FIRE),
        (HwKey::Menu, KEY_LEFT_SOFT),
    ] {
        assert_eq!(
            map_hw_key(k, false),
            KeyMapResult { code: Some(code), consumed: true },
            "{k:?} → {code}"
        );
        assert_eq!(map_hw_key(k, true).code, None, "{k:?} repeat 丢弃");
    }
}

/// 音量键交系统（consumed=false）；未知键不投递不吞。
#[test]
fn volume_and_unknown_keys_not_consumed() {
    for k in [HwKey::VolumeUp, HwKey::VolumeDown, HwKey::Other] {
        assert_eq!(map_hw_key(k, false), KeyMapResult { code: None, consumed: false }, "{k:?}");
    }
}

// ---------------------------------------------------------------- 触控翻译

/// 盘心死区内按下/抬起 ⇒ 无键事件（按下不发、抬起也不发——没按下过就没有 keyReleased）。
#[test]
fn dead_zone_down_and_up_emit_nothing() {
    let mut m = TouchMapper::new(layout());
    assert_eq!(m.feed(&touch(1, TouchPhase::Down, 3.0, 4.0)), vec![]);
    assert_eq!(m.feed(&touch(1, TouchPhase::Up, 3.0, 4.0)), vec![]);
}

/// 四方向：主导轴判定（|dx| vs |dy|）。
#[test]
fn four_directions_by_dominant_axis() {
    for (x, y, code) in [
        (0.0, -40.0, KEY_UP),
        (0.0, 40.0, KEY_DOWN),
        (-40.0, 0.0, KEY_LEFT),
        (40.0, 0.0, KEY_RIGHT),
    ] {
        let mut m = TouchMapper::new(layout());
        assert_eq!(m.feed(&touch(1, TouchPhase::Down, x, y)), vec![KeyAction::Press(code)]);
    }
}

/// 迟滞防抖：已持方向在 [r_inner, r_switch] 带内不换向（主导轴已变也不换）；
/// 出带（> r_switch）才换向（Release 旧 + Press 新，保序）。
#[test]
fn direction_hysteresis_prevents_flicker() {
    let mut m = TouchMapper::new(layout());
    assert_eq!(m.feed(&touch(1, TouchPhase::Down, 35.0, 0.0)), vec![KeyAction::Press(KEY_RIGHT)]);
    // 带内（r≈29.7 < 30）且主导轴已翻成 DOWN ⇒ 仍保持 RIGHT，无事件
    assert_eq!(m.feed(&touch(1, TouchPhase::Move, 20.0, 22.0)), vec![]);
    // 出带（r≈40.4 > 30）⇒ 换向 DOWN：Release 旧 + Press 新
    assert_eq!(
        m.feed(&touch(1, TouchPhase::Move, 20.0, 35.0)),
        vec![KeyAction::Release, KeyAction::Press(KEY_DOWN)]
    );
}

/// 拖动换向事件流 = 抬起旧键 + 按下新键（keyHeldCode 覆盖式，无双 held 并存）。
#[test]
fn drag_up_to_right_emits_release_then_press() {
    let mut m = TouchMapper::new(layout());
    let mut g = FakeGame::default();
    g.apply(&m.feed(&touch(1, TouchPhase::Down, 0.0, -40.0)));
    assert_eq!((g.key_value, g.key_held), (KEY_UP, KEY_UP));
    let acts = m.feed(&touch(1, TouchPhase::Move, 40.0, -1.0));
    assert_eq!(acts, vec![KeyAction::Release, KeyAction::Press(KEY_RIGHT)], "换向 = 旧键抬 + 新键按");
    g.apply(&acts);
    assert_eq!((g.key_value, g.key_held), (KEY_RIGHT, KEY_RIGHT), "折叠后 held 只有新键");
}

/// Cancel = Up 语义（Android ACTION_CANCEL ⇒ 视同抬起）。
#[test]
fn cancel_acts_as_release() {
    let mut m = TouchMapper::new(layout());
    assert_eq!(m.feed(&touch(1, TouchPhase::Down, 120.0, 120.0)), vec![KeyAction::Press(KEY_LEFT_SOFT)]);
    assert_eq!(m.feed(&touch(1, TouchPhase::Cancel, 120.0, 120.0)), vec![KeyAction::Release]);
}

// ---------------------------------------------------------------- 折叠单键状态机（实现硬门）

/// 折叠终态：按住方向盘(-1)再按软键(-5) ⇒ key_value = key_held = -5（覆盖式取末键，行走即停）。
#[test]
fn fold_hold_dpad_then_soft_key_overwrites_held() {
    let mut m = TouchMapper::new(layout());
    let mut g = FakeGame::default();
    g.apply(&m.feed(&touch(1, TouchPhase::Down, 0.0, -40.0))); // -1
    assert_eq!((g.key_value, g.key_held), (KEY_UP, KEY_UP));
    g.apply(&m.feed(&touch(2, TouchPhase::Down, 170.0, 120.0))); // -5
    assert_eq!((g.key_value, g.key_held), (KEY_FIRE, KEY_FIRE), "折叠：无双 held 并存");
}

/// 任一持键触点 Up ⇒ key_held=0 且**不自动恢复**（松开软键不恢复行走——原版实机同款）。
#[test]
fn fold_release_any_clears_held_without_auto_resume() {
    let mut m = TouchMapper::new(layout());
    let mut g = FakeGame::default();
    g.apply(&m.feed(&touch(1, TouchPhase::Down, 0.0, -40.0))); // -1 held
    g.apply(&m.feed(&touch(2, TouchPhase::Down, 170.0, 120.0))); // -5 覆盖 held
    g.apply(&m.feed(&touch(2, TouchPhase::Up, 170.0, 120.0))); // 松软键
    assert_eq!(g.key_held, 0, "任一持键触点 Up 清 held");
    assert_eq!(g.key_value, KEY_FIRE, "key_value 不被 release 改写");
    // 方向指仍按着 ⇒ held 仍 0（无恢复）；再等一拍 tick 尾清 key_value=0 后行走自然停
    assert_eq!(m.feed(&touch(1, TouchPhase::Up, 0.0, -40.0)), vec![KeyAction::Release]);
    assert_eq!(g.key_held, 0);
}

/// 事件保序：翻译输出的事件序列顺序与源事件顺序一致（Press/Release 交错不重排）。
#[test]
fn event_order_preserved() {
    let mut m = TouchMapper::new(layout());
    let seq = [
        touch(1, TouchPhase::Down, 0.0, -40.0),   // Press(-1)
        touch(2, TouchPhase::Down, 120.0, 120.0), // Press(-6)
        touch(1, TouchPhase::Move, 40.0, -1.0),   // Release + Press(-4)
        touch(2, TouchPhase::Up, 120.0, 120.0),   // Release
        touch(1, TouchPhase::Up, 40.0, -1.0),     // Release
    ];
    let mut out = Vec::new();
    for ev in &seq {
        out.extend(m.feed(ev));
    }
    assert_eq!(
        out,
        vec![
            KeyAction::Press(KEY_UP),
            KeyAction::Press(KEY_LEFT_SOFT),
            KeyAction::Release,
            KeyAction::Press(KEY_RIGHT),
            KeyAction::Release,
            KeyAction::Release,
        ]
    );
}

/// 按区域定角色：软键+方向盘可同时按（两指独立翻译），角色由落点区域决定、互不抢占。
#[test]
fn region_roles_allow_simultaneous_soft_and_dpad() {
    let mut m = TouchMapper::new(layout());
    assert_eq!(m.feed(&touch(1, TouchPhase::Down, 0.0, -40.0)), vec![KeyAction::Press(KEY_UP)]);
    assert_eq!(m.feed(&touch(2, TouchPhase::Down, 220.0, 120.0)), vec![KeyAction::Press(KEY_RIGHT_SOFT)]);
    assert_eq!(m.feed(&touch(3, TouchPhase::Down, 170.0, 120.0)), vec![KeyAction::Press(KEY_FIRE)]);
}

/// Java 语义精确口径：没按下过的触点（死区指）抬起**不发** Release（keyReleased 只为
/// 曾按下的键发生）；曾按下过的触点无论指针漂移到哪，Up ⇒ Release。
#[test]
fn up_emits_release_only_for_pointers_that_pressed() {
    let mut m = TouchMapper::new(layout());
    assert_eq!(m.feed(&touch(1, TouchPhase::Down, 0.0, -40.0)), vec![KeyAction::Press(KEY_UP)]);
    assert_eq!(m.feed(&touch(2, TouchPhase::Down, 3.0, 3.0)), vec![], "死区指无键");
    assert_eq!(m.feed(&touch(2, TouchPhase::Up, 3.0, 3.0)), vec![], "死区指 Up 无 Release");
    assert_eq!(m.feed(&touch(1, TouchPhase::Up, 300.0, 300.0)), vec![KeyAction::Release]);
}

/// 隐藏期输入不投递（Visibility 门控）；BACK 仍吞默认（防系统 finish）。
#[test]
fn hidden_drops_input_but_back_still_consumed() {
    let mut r = InputRouter::new(layout());
    r.set_shown(false);
    assert_eq!(r.touch(&touch(1, TouchPhase::Down, 0.0, -40.0)), vec![]);
    assert_eq!(r.touch(&touch(1, TouchPhase::Up, 0.0, -40.0)), vec![]);
    assert_eq!(r.hw(HwKey::DpadUp, false), KeyMapResult { code: None, consumed: true });
    assert_eq!(r.hw(HwKey::Back, false), KeyMapResult { code: None, consumed: true });
    r.set_shown(true);
    assert_eq!(r.touch(&touch(1, TouchPhase::Down, 0.0, -40.0)), vec![KeyAction::Press(KEY_UP)]);
}

/// 默认 240×320 布局自洽：软键区在画布内、键码齐全。
#[test]
fn default_layout_fits_canvas() {
    let l = PadLayout::default_240x320();
    for s in &l.soft {
        assert!(s.x0 >= 0.0 && s.x1 <= 240.0 && s.y0 >= 0.0 && s.y1 <= 320.0, "{s:?} 越界");
    }
    let mut codes: Vec<i32> = l.soft.iter().map(|s| s.code).collect();
    codes.sort_unstable();
    assert_eq!(codes, [KEY_RIGHT_SOFT, KEY_LEFT_SOFT, KEY_FIRE], "升序 -7,-6,-5");
}
