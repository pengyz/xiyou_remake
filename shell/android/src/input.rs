//! 触控/硬件键 → 游戏键码映射（纯逻辑，宿主可测）。
//!
//! # 真值锚
//!
//! - 键码面 docs/spec/key-code-contract.md §4（与 shell/win `key_code()`
//!   `shell/win/src/main.rs:222-235` 同表）：-1 上/-2 下/-3 左/-4 右/-5 探索确认/
//!   -6 确认使用（左软）/-7 返回退出（右软）。
//! - **折叠单键状态机**（实现硬门，对抗审查 v2 复审口径修正 2026-10-17）：多点触控
//!   折叠进**单一**键状态机，禁止 per-role/per-pointer 独立 held。权威锚
//!   `scene.rs:4270-4274`（press_key = key_value = key_held = code 覆盖式取末键）与
//!   `:4276-4279`（release_key 只清 key_held=0，不认键）。按住方向盘(-1)再按软键(-5)
//!   ⇒ 折叠为 key_value = key_held = -5（行走即停）；松任一**持键**触点 ⇒ key_held=0
//!   且**行走不自动恢复**（"松开软键恢复行走" = 发明行为 = 禁）。
//! - BACK 键吞默认（BUG-020 退出链：退出只经 `CMidlet.dispatchModeEntry` 信号 → shell 消费；
//!   BACK 不消费 ⇒ 系统 finish Activity ⇒ 绕过退出链、时机全变）。
//! - OS 键重复丢弃（`docs/knowledge/gotcha_os-key-repeat-is-not-keypressed.md`）。
//! - 隐藏期输入不投递（MIDP Canvas「Event Delivery」；`shell/win/src/lifecycle.rs:26-28`）。
//!
//! # 折叠的实现口径
//!
//! 本模块的 per-pointer 记录只是**翻译簿**（Move/Up 需知道该触点翻译出过哪个键码），
//! 不是独立 held 态：输出是扁平事件流 `Press(code)` / `Release`（Release 不带键——
//! `release_key()` 不认键），状态共存由 game-core 的 press_key/release_key 折叠。
//! 译法：没按下过的触点（死区指）Up **不发** Release（keyReleased 只为曾按下的键发生；
//! scene.rs:4276 的"不认键"指 release 不挑键，不指任何 Up 都产生 release）。

/// 游戏键码（key-code-contract §4）。
pub const KEY_UP: i32 = -1;
pub const KEY_DOWN: i32 = -2;
pub const KEY_LEFT: i32 = -3;
pub const KEY_RIGHT: i32 = -4;
pub const KEY_FIRE: i32 = -5;
pub const KEY_LEFT_SOFT: i32 = -6;
pub const KEY_RIGHT_SOFT: i32 = -7;

/// 触控阶段（Android MotionEvent ACTION_* 的壳侧归纳）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchPhase {
    Down,
    Move,
    Up,
    /// ACTION_CANCEL：视同抬起（触点被系统截走）。
    Cancel,
}

/// 一个触点事件（坐标 = 逻辑画布 240×320 空间，Viewport 反映射后的值）。
#[derive(Clone, Copy, Debug)]
pub struct TouchEvent {
    pub id: i32,
    pub phase: TouchPhase,
    pub x: f32,
    pub y: f32,
}

/// 投递给游戏的键动作（折叠单键状态机的事件面）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    Press(i32),
    /// 抬起（不带键：`release_key()` 不认键，scene.rs:4276-4279）。
    Release,
}

/// 硬件键（Android KeyEvent KEYCODE_* 的壳侧归纳）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HwKey {
    Back,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    DpadCenter,
    Enter,
    Space,
    Menu,
    VolumeUp,
    VolumeDown,
    Other,
}

/// 硬件键映射结果：投递的键码 + 是否消费（consumed=false ⇒ 交系统默认行为）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyMapResult {
    pub code: Option<i32>,
    pub consumed: bool,
}

/// 硬件键 → 键码。repeat = OS 自动重复：**丢弃**（不投递键），但消费性不变
/// （BACK 的 repeat 仍须吞默认，否则长按 BACK 触发系统 finish）。
pub fn map_hw_key(key: HwKey, repeat: bool) -> KeyMapResult {
    let code = match key {
        HwKey::Back => KEY_RIGHT_SOFT,
        HwKey::DpadUp => KEY_UP,
        HwKey::DpadDown => KEY_DOWN,
        HwKey::DpadLeft => KEY_LEFT,
        HwKey::DpadRight => KEY_RIGHT,
        HwKey::DpadCenter | HwKey::Enter | HwKey::Space => KEY_FIRE,
        HwKey::Menu => KEY_LEFT_SOFT,
        HwKey::VolumeUp | HwKey::VolumeDown | HwKey::Other => {
            return KeyMapResult { code: None, consumed: false }
        }
    };
    KeyMapResult { code: if repeat { None } else { Some(code) }, consumed: true }
}

/// 虚拟方向盘区（逻辑画布坐标）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DpadZone {
    pub cx: f32,
    pub cy: f32,
    /// 死区半径：盘心内按下/漂移不产生键。
    pub r_inner: f32,
    /// 换向迟滞半径：带内保持既有方向（防抖），出带才换向。
    pub r_switch: f32,
}

/// 软键区（矩形，半开 [x0,x1)×[y0,y1)）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoftZone {
    pub code: i32,
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

/// 虚拟手柄布局（overlay 几何）。
#[derive(Clone, Debug, PartialEq)]
pub struct PadLayout {
    pub dpad: DpadZone,
    pub soft: Vec<SoftZone>,
}

impl PadLayout {
    /// 默认 240×320 布局：左下方向盘 + 右下三软键（-6 左软/-5 确认/-7 右软）。
    /// 对照 shell/win 鼠标映射（main.rs:13：底部 30% 左 1/3=-6、右 1/3=-7、其余=-5），
    /// 触屏用常显 overlay 几何（对抗审查 §3：坐标→键码解耦为纯函数）。
    /// 【复核B R3】Modern 方画幅布局（贴底/贴右，对齐 layout.rs modern 软键锚：
    /// softkey_base_y=grid*32-18——13/18/24 格=398/558/750）：触控件随格数生成，
    /// 区=classic 布局相对 (右,底) 偏移同式（左块 [w-128,w-72)、右键 [w-64,w-8)、
    /// 底带 [h-100,h-36)、方向盘 (56, h-68)）。
    pub fn for_modern(grid_n: i32) -> PadLayout {
        let w = (grid_n * 32) as f32;
        let h = (grid_n * 32) as f32;
        PadLayout {
            dpad: DpadZone { cx: 56.0, cy: h - 68.0, r_inner: 12.0, r_switch: 30.0 },
            soft: vec![
                SoftZone { code: KEY_LEFT_SOFT, x0: w - 128.0, y0: h - 100.0, x1: w - 72.0, y1: h - 72.0 },
                SoftZone { code: KEY_FIRE, x0: w - 128.0, y0: h - 64.0, x1: w - 72.0, y1: h - 36.0 },
                SoftZone { code: KEY_RIGHT_SOFT, x0: w - 64.0, y0: h - 100.0, x1: w - 8.0, y1: h - 36.0 },
            ],
        }
    }

    pub fn default_240x320() -> PadLayout {
        PadLayout {
            dpad: DpadZone { cx: 56.0, cy: 252.0, r_inner: 12.0, r_switch: 30.0 },
            soft: vec![
                SoftZone { code: KEY_LEFT_SOFT, x0: 112.0, y0: 220.0, x1: 168.0, y1: 248.0 },
                SoftZone { code: KEY_FIRE, x0: 112.0, y0: 256.0, x1: 168.0, y1: 284.0 },
                SoftZone { code: KEY_RIGHT_SOFT, x0: 176.0, y0: 220.0, x1: 232.0, y1: 284.0 },
            ],
        }
    }
}

/// 触点在方向盘区（捕获半径 = 2×r_switch：迟滞半径外仍捕获方向指——拖远 = 强方向）。
pub fn in_dpad_capture(d: &DpadZone, x: f32, y: f32) -> bool {
    let (dx, dy) = (x - d.cx, y - d.cy);
    let r = d.r_switch * 2.0;
    dx * dx + dy * dy <= r * r
}

/// 方向判定（主导轴 + 死区 + 迟滞）。
fn dpad_dir(d: &DpadZone, x: f32, y: f32, prev: Option<i32>) -> Option<i32> {
    let (dx, dy) = (x - d.cx, y - d.cy);
    let d2 = dx * dx + dy * dy;
    if d2 <= d.r_inner * d.r_inner {
        return None; // 死区
    }
    let cand = if dx.abs() >= dy.abs() {
        if dx < 0.0 { KEY_LEFT } else { KEY_RIGHT }
    } else if dy < 0.0 {
        KEY_UP
    } else {
        KEY_DOWN
    };
    if d2 <= d.r_switch * d.r_switch {
        prev.or(Some(cand)) // 迟滞带：保持既有方向
    } else {
        Some(cand)
    }
}

/// 触点角色（按区域定角色：落点决定翻译，不决定状态共存）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Dpad,
    Soft,
}

/// 翻译簿条目（该触点翻译出过的键码——Move/Up 用；非独立 held 态）。
#[derive(Clone, Copy, Debug)]
struct Pointer {
    role: Role,
    code: Option<i32>,
}

/// 触控 → 键动作流翻译器（多指；折叠交给消费侧单一键状态机）。
#[derive(Debug)]
pub struct TouchMapper {
    layout: PadLayout,
    pointers: Vec<(i32, Pointer)>,
}

impl TouchMapper {
    pub fn new(layout: PadLayout) -> TouchMapper {
        TouchMapper { layout, pointers: Vec::new() }
    }

    fn soft_at(&self, x: f32, y: f32) -> Option<i32> {
        self.layout
            .soft
            .iter()
            .find(|s| x >= s.x0 && x < s.x1 && y >= s.y0 && y < s.y1)
            .map(|s| s.code)
    }

    /// 当前按住的键码（各触点翻译簿里的现行键；面板按下反馈用，零状态副作用）。
    pub fn held_codes(&self) -> Vec<i32> {
        let mut v: Vec<i32> = self.pointers.iter().filter_map(|(_, p)| p.code).collect();
        v.sort();
        v.dedup();
        v
    }

    /// 一个触点事件 → 键动作流（保序）。
    pub fn feed(&mut self, ev: &TouchEvent) -> Vec<KeyAction> {
        match ev.phase {
            TouchPhase::Down => {
                // 同 id 重复 Down（异常流）：以新 Down 为准，旧簿丢弃（无键事件）。
                self.pointers.retain(|(id, _)| *id != ev.id);
                if let Some(code) = self.soft_at(ev.x, ev.y) {
                    self.pointers.push((ev.id, Pointer { role: Role::Soft, code: Some(code) }));
                    vec![KeyAction::Press(code)]
                } else if in_dpad_capture(&self.layout.dpad, ev.x, ev.y) {
                    let code = dpad_dir(&self.layout.dpad, ev.x, ev.y, None);
                    self.pointers.push((ev.id, Pointer { role: Role::Dpad, code }));
                    code.map(|c| vec![KeyAction::Press(c)]).unwrap_or_default()
                } else {
                    Vec::new() // overlay 区域外触点：忽略（不入翻译簿）
                }
            }
            TouchPhase::Move => {
                let Some(i) = self.pointers.iter().position(|(id, _)| *id == ev.id) else {
                    return Vec::new();
                };
                let p = self.pointers[i].1;
                match p.role {
                    // 按钮捕获语义：按下后漂移不换键、不出区不释放，Up/Cancel 才释放。
                    Role::Soft => Vec::new(),
                    Role::Dpad => {
                        let new = dpad_dir(&self.layout.dpad, ev.x, ev.y, p.code);
                        if new == p.code {
                            return Vec::new();
                        }
                        self.pointers[i].1.code = new;
                        let mut acts = Vec::new();
                        if p.code.is_some() {
                            acts.push(KeyAction::Release);
                        }
                        if let Some(c) = new {
                            acts.push(KeyAction::Press(c));
                        }
                        acts
                    }
                }
            }
            TouchPhase::Up | TouchPhase::Cancel => {
                let Some(i) = self.pointers.iter().position(|(id, _)| *id == ev.id) else {
                    return Vec::new();
                };
                let p = self.pointers.remove(i).1;
                // 只为**曾按下**的键发 Release（keyReleased 语义）；死区指 Up 无事件。
                if p.code.is_some() {
                    vec![KeyAction::Release]
                } else {
                    Vec::new()
                }
            }
        }
    }
}

/// 输入路由（Visibility 门控）：隐藏期输入不投递，但 BACK 仍吞默认
/// （防隐藏期系统 finish Activity 绕过退出链）。
#[derive(Debug)]
pub struct InputRouter {
    mapper: TouchMapper,
    shown: bool,
}

impl InputRouter {
    pub fn new(layout: PadLayout) -> InputRouter {
        InputRouter { mapper: TouchMapper::new(layout), shown: true }
    }

    /// 由生命周期状态机驱动（`lifecycle::Visibility::shown()`）。
    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn touch(&mut self, ev: &TouchEvent) -> Vec<KeyAction> {
        if self.shown {
            self.mapper.feed(ev)
        } else {
            Vec::new() // 隐藏期事件整体不投递（含簿记：隐藏期无键可按）
        }
    }

    pub fn hw(&self, key: HwKey, repeat: bool) -> KeyMapResult {
        let r = map_hw_key(key, repeat);
        if self.shown {
            r
        } else {
            KeyMapResult { code: None, consumed: r.consumed }
        }
    }

}

// ---------------- T3 触屏映射集成（touch-native-undo-plan §1，2026-10-23） ----------------

use game_shell_common::touch::{self as tch, Gesture, ShellAction, TouchGeom};

/// 触点双模集成（用户裁定：双模**并存**非替代）：虚拟键区（DpadZone/SoftZone
/// 一等公民，既有 TouchMapper 语义一行不改）+ 自由区触点（点按移动/滑动踏步/
/// 撤销钮，game-shell-common::touch 单源）。win 鼠标=触点等价（shell/win
/// main.rs 同 consume 共享层——双壳同构）。
#[derive(Debug)]
pub struct TouchInput {
    pub mapper: TouchMapper,
    /// 虚拟键区触点 id 簿（区分键通道/手势通道）
    pad_ids: Vec<i32>,
    gestures: tch::GestureTracker,
}

impl TouchInput {
    pub fn new(layout: PadLayout) -> TouchInput {
        TouchInput {
            mapper: TouchMapper::new(layout),
            pad_ids: Vec::new(),
            gestures: tch::GestureTracker::new(24),
        }
    }

    fn in_pad(&self, x: f32, y: f32) -> bool {
        self.mapper.soft_at(x, y).is_some() || in_dpad_capture(&self.mapper.layout.dpad, x, y)
    }

    fn map_keys(keys: Vec<KeyAction>) -> Vec<ShellAction> {
        keys.iter()
            .flat_map(|k| match k {
                KeyAction::Press(c) => tch::Dispatcher::key(*c),
                KeyAction::Release => tch::Dispatcher::release(),
            })
            .collect()
    }

    /// 一个触点事件 → ShellAction 流（保序；键/触两通道同流=双模并存无互斥；
    /// 打断全通道=各通道首动作 Interrupt，由 Dispatcher 组装）。
    pub fn feed(
        &mut self,
        ev: &TouchEvent,
        geom: &TouchGeom,
        screen_w: i32,
        screen_h: i32,
        player: (i32, i32),
        walkable: impl Fn(i32, i32) -> bool,
    ) -> Vec<ShellAction> {
        match ev.phase {
            TouchPhase::Down => {
                if self.in_pad(ev.x, ev.y) {
                    self.pad_ids.push(ev.id);
                    let keys = self.mapper.feed(ev);
                    Self::map_keys(keys)
                } else {
                    self.gestures.down(ev.x as i32, ev.y as i32);
                    Vec::new()
                }
            }
            TouchPhase::Move => {
                if self.pad_ids.contains(&ev.id) {
                    let keys = self.mapper.feed(ev);
                    Self::map_keys(keys)
                } else {
                    // 滑动=方向手动踏步（一格/次、撞墙即停）
                    self.gestures
                        .move_to(ev.x as i32, ev.y as i32)
                        .map(tch::Dispatcher::swipe)
                        .unwrap_or_default()
                }
            }
            TouchPhase::Up | TouchPhase::Cancel => {
                if let Some(pos) = self.pad_ids.iter().position(|i| *i == ev.id) {
                    self.pad_ids.remove(pos);
                    let keys = self.mapper.feed(ev);
                    Self::map_keys(keys)
                } else {
                    match self.gestures.up(ev.x as i32, ev.y as i32) {
                        Gesture::Tap(x, y) => tch::Dispatcher::tap(tch::classify_tap(
                            x,
                            y,
                            geom,
                            screen_w,
                            screen_h,
                            player,
                            walkable,
                        )),
                        _ => Vec::new(),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod t3_touch_tests {
    use super::*;

    fn geom() -> TouchGeom {
        TouchGeom { cam_x: -144, cam_y: -16, tile_off: 20, cols: 26, rows: 26 }
    }
    fn ev(id: i32, phase: TouchPhase, x: f32, y: f32) -> TouchEvent {
        TouchEvent { id, phase, x, y }
    }

    /// 双模并存：虚拟键区按键流与自由区 tap 动作流互不压制（用户裁定）。
    #[test]
    fn dual_mode_pad_keys_and_free_taps_coexist() {
        let mut ti = TouchInput::new(PadLayout::default_240x320());
        let g = geom();
        // 自由区 tap → WalkTo（点空地）
        let a1 = ti.feed(&ev(1, TouchPhase::Down, 32.0, 48.0), &g, 240, 320, (9, 1), |_, _| true);
        assert!(a1.is_empty(), "按下不发（手势待抬起分类）");
        let a2 = ti.feed(&ev(1, TouchPhase::Up, 32.0, 48.0), &g, 240, 320, (9, 1), |_, _| true);
        assert_eq!(a2, vec![ShellAction::Interrupt, ShellAction::WalkTo((11, 2))], "tap=打断+WalkTo");
        // 虚拟键区（方向盘 cy=252）按键流照常（不被 tap 压制）
        let a3 = ti.feed(&ev(2, TouchPhase::Down, 86.0, 252.0), &g, 240, 320, (9, 1), |_, _| true);
        assert_eq!(a3, vec![ShellAction::Interrupt, ShellAction::Key(KEY_RIGHT)], "tap 后方向盘不被压制");
        let a4 = ti.feed(&ev(2, TouchPhase::Up, 86.0, 252.0), &g, 240, 320, (9, 1), |_, _| true);
        assert_eq!(a4, vec![ShellAction::KeyRelease], "抬起=Release");
        // 自由区 tap 照常（不被键流压制）
        let a5 = ti.feed(&ev(3, TouchPhase::Down, 32.0, 48.0), &g, 240, 320, (9, 1), |_, _| true);
        let a6 = ti.feed(&ev(3, TouchPhase::Up, 32.0, 48.0), &g, 240, 320, (9, 1), |_, _| true);
        assert_eq!(a6[1], ShellAction::WalkTo((11, 2)), "按键后 tap 不被压制");
        assert_eq!(a5, Vec::new());
    }

    /// 点墙=探步入、撤销区=Undo、滑动=一格/次踏步（撞墙即停）。
    #[test]
    fn free_tap_wall_probe_undo_and_swipe_step() {
        let mut ti = TouchInput::new(PadLayout::default_240x320());
        let g = geom();
        let walkable = |x: i32, y: i32| !(x == 10 && y == 1); // (10,1)=墙
        // 点墙格 → 探步入
        ti.feed(&ev(1, TouchPhase::Down, 16.0, 20.0), &g, 240, 320, (9, 1), walkable);
        let a = ti.feed(&ev(1, TouchPhase::Up, 16.0, 20.0), &g, 240, 320, (9, 1), walkable);
        assert_eq!(a, vec![ShellAction::Interrupt, ShellAction::ProbeStep(2)], "点墙格=探步入 Right");
        // 撤销区 → Undo
        ti.feed(&ev(2, TouchPhase::Down, 200.0, 5.0), &g, 240, 320, (9, 1), walkable);
        let a = ti.feed(&ev(2, TouchPhase::Up, 200.0, 5.0), &g, 240, 320, (9, 1), walkable);
        assert_eq!(a, vec![ShellAction::Interrupt, ShellAction::Undo], "撤销区=Undo");
        // 滑动：一格/次（越阈一次；续滑无后续=撞墙即停语义）
        ti.feed(&ev(3, TouchPhase::Down, 100.0, 100.0), &g, 240, 320, (9, 1), walkable);
        let a = ti.feed(&ev(3, TouchPhase::Move, 140.0, 100.0), &g, 240, 320, (9, 1), walkable);
        assert_eq!(a, vec![ShellAction::Interrupt, ShellAction::ProbeStep(2)], "滑动=打断+一步踏步");
        let a = ti.feed(&ev(3, TouchPhase::Move, 300.0, 100.0), &g, 240, 320, (9, 1), walkable);
        assert!(a.is_empty(), "同手势续滑无后续步（一格/次、撞墙即停）");
    }
}
