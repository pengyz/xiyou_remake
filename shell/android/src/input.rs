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
//! - BACK 键吞默认（BUG-020 退出链：退出只经 `CMidlet.m_000` 信号 → shell 消费；
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
#[derive(Clone, Copy, Debug)]
pub struct DpadZone {
    pub cx: f32,
    pub cy: f32,
    /// 死区半径：盘心内按下/漂移不产生键。
    pub r_inner: f32,
    /// 换向迟滞半径：带内保持既有方向（防抖），出带才换向。
    pub r_switch: f32,
}

/// 软键区（矩形，半开 [x0,x1)×[y0,y1)）。
#[derive(Clone, Copy, Debug)]
pub struct SoftZone {
    pub code: i32,
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

/// 虚拟手柄布局（overlay 几何）。
#[derive(Clone, Debug)]
pub struct PadLayout {
    pub dpad: DpadZone,
    pub soft: Vec<SoftZone>,
}

impl PadLayout {
    /// 默认 240×320 布局：左下方向盘 + 右下三软键（-6 左软/-5 确认/-7 右软）。
    /// 对照 shell/win 鼠标映射（main.rs:13：底部 30% 左 1/3=-6、右 1/3=-7、其余=-5），
    /// 触屏用常显 overlay 几何（对抗审查 §3：坐标→键码解耦为纯函数）。
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
fn in_dpad_capture(d: &DpadZone, x: f32, y: f32) -> bool {
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
