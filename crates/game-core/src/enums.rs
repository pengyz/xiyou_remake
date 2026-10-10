//! 游戏语义枚举 —— 消灭魔数（2026-10-10 用户要求落地）。
//!
//! 每个枚举的取值域与语义都有 A 级证据（deobf 行号 / oracle trace）：
//! 未证语义的值保留机械命名变体（`Phase4` 等），证伪更正时改名不改值。

/// 对白相位（Java `dialogPhase`，f_byte_16）。
/// 证据：docs/findings/runtime-semantics.md §5 相位表。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogPhase {
    /// 0：脚本步进（executeScriptInstruction 消费）
    ScriptStep,
    /// 1：打字机（文本未完）
    Typewriter,
    /// 2：选项/分支（m_106）
    Choice,
    /// 3：行走执行（消费 walkPathBuffer）
    Walk,
    /// 4：等待相机到位（cameraAtTarget → 回地图；行尾收尾门亦落此相位）
    AwaitCamera,
    /// 5：LAY 过场（换层清屏）
    LayCutscene,
    /// 6：打字机（带相机门控变体，SEE 使用）
    TypewriterCamera,
    /// 未知值（Java 字节可容纳任意值，保真保留）
    Unknown(i32),
}

impl DialogPhase {
    pub fn from_raw(v: i32) -> Self {
        match v {
            0 => Self::ScriptStep,
            1 => Self::Typewriter,
            2 => Self::Choice,
            3 => Self::Walk,
            4 => Self::AwaitCamera,
            5 => Self::LayCutscene,
            6 => Self::TypewriterCamera,
            other => Self::Unknown(other),
        }
    }

    pub fn raw(self) -> i32 {
        match self {
            Self::ScriptStep => 0,
            Self::Typewriter => 1,
            Self::Choice => 2,
            Self::Walk => 3,
            Self::AwaitCamera => 4,
            Self::LayCutscene => 5,
            Self::TypewriterCamera => 6,
            Self::Unknown(v) => v,
        }
    }
}

/// 行走相位（Java `walkPhase`，f_byte_11）。
/// 证据：runtime-semantics §5。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalkPhase {
    /// 0：待机/探索（-5 探索与方向处理）
    Idle,
    /// 1：步进动画（8px 步进，32px 归格）
    Stepping,
    /// 2：探索确认弹窗（f_bool_06）
    ExploreConfirm,
    /// 3：脚本寻路行走（消费 walkPathBuffer）
    ScriptWalk,
    /// 4：m_063 使用点（语义待考证，机械名保留）
    Phase4,
    /// 5：战斗（tickBattle）
    Battle,
    Unknown(i32),
}

impl WalkPhase {
    pub fn from_raw(v: i32) -> Self {
        match v {
            0 => Self::Idle,
            1 => Self::Stepping,
            2 => Self::ExploreConfirm,
            3 => Self::ScriptWalk,
            4 => Self::Phase4,
            5 => Self::Battle,
            other => Self::Unknown(other),
        }
    }

    pub fn raw(self) -> i32 {
        match self {
            Self::Idle => 0,
            Self::Stepping => 1,
            Self::ExploreConfirm => 2,
            Self::ScriptWalk => 3,
            Self::Phase4 => 4,
            Self::Battle => 5,
            Self::Unknown(v) => v,
        }
    }
}

/// 行走方向码（walkPathBuffer / tryStep 语义，m_133 回溯映射）。
/// 证据：deobf a.java:9887-9899（差分 → 方向码映射）+ walkPhase 1 步进 switch。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    /// 0：屏幕向下（pixelY += 8）
    Down,
    /// 1：屏幕向上
    Up,
    /// 2：屏幕向右
    Right,
    /// 3：屏幕向左
    Left,
}

impl Dir {
    pub fn raw(self) -> u8 {
        self as u8
    }
}

/// 实体类目（Java `f_byte_arr_03[type]`，a.java:6204-6252 初始化）。
/// CES 延迟与 markEntityRemoved 移除分派都查此表。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Cat1,
    Cat2,
    Cat4,
    Cat8,
    Cat16,
    Cat32,
    /// 未在初始化中赋值（默认 0）
    Zero,
    Unknown(u8),
}

impl Category {
    pub fn from_raw(v: u8) -> Self {
        match v {
            0 => Self::Zero,
            1 => Self::Cat1,
            2 => Self::Cat2,
            4 => Self::Cat4,
            8 => Self::Cat8,
            16 => Self::Cat16,
            32 => Self::Cat32,
            other => Self::Unknown(other),
        }
    }

    /// CES 延迟映射（a.java:8384-8391）：Cat1/Cat8/Cat16 → 5；Cat2/Cat4 → 8；默认 0。
    pub fn scene_delay(self) -> i32 {
        match self {
            Self::Cat1 | Self::Cat8 | Self::Cat16 => 5,
            Self::Cat2 | Self::Cat4 => 8,
            _ => 0,
        }
    }
}

/// 实体类型码常量（开放域：此处仅列已被 trace/deobf 证实语义者；
/// 证据：object_type_names 夹具中文名 + trace 行为）。
pub mod entity_kind {
    /// 上楼梯（spawnEntity 楼梯锚点 f_int_70/72）
    pub const STAIR_UP: i32 = 7;
    /// 下楼梯（f_int_71/73）
    pub const STAIR_DOWN: i32 = 8;
    /// 路线触发型（MOV 5 参分支写 f_int_arr_12=1）
    pub const ROUTE: i32 = 72;
    /// 菩提老祖（序章 NPC；m_104 说话人特例 0/87 免查实体）
    pub const NPC_BODHI: i32 = 87;
    /// SWD 改写源/目标（封印门 → 可通行物件）
    pub const SEAL_GATE: i32 = 81;
    pub const SEAL_CONVERTED: i32 = 4;
    /// MVS 配对交换型
    pub const MVS_PAIR: i32 = 82;
}

/// 游戏模式（Java `gameMode`，FLD010；state-machine.md 全表）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// 0：启动画面
    Boot,
    /// 1：主菜单
    MainMenu,
    /// 2：资源载入进度
    Loading,
    /// 3：地图主玩法
    Map,
    /// 11：对白/剧情模式（loadLevelScript 进入）
    Dialog,
    /// 14：读条过场（新游戏）
    IntroCutscene,
    /// 20：END 结束过场（70 帧后销毁）
    Ending,
    /// 21：音效询问
    SoundAsk,
    /// 99：联网等待
    NetWait,
    Unknown(i32),
}

impl GameMode {
    pub fn from_raw(v: i32) -> Self {
        match v {
            0 => Self::Boot,
            1 => Self::MainMenu,
            2 => Self::Loading,
            3 => Self::Map,
            11 => Self::Dialog,
            14 => Self::IntroCutscene,
            20 => Self::Ending,
            21 => Self::SoundAsk,
            99 => Self::NetWait,
            other => Self::Unknown(other),
        }
    }

    pub fn raw(self) -> i32 {
        match self {
            Self::Boot => 0,
            Self::MainMenu => 1,
            Self::Loading => 2,
            Self::Map => 3,
            Self::Dialog => 11,
            Self::IntroCutscene => 14,
            Self::Ending => 20,
            Self::SoundAsk => 21,
            Self::NetWait => 99,
            Self::Unknown(v) => v,
        }
    }
}
