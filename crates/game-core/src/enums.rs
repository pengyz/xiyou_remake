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
    /// 2：实体路径缓动（MOV 五参 → m_106 每拍推进，a.java:4072-4073 / 8738-8870）。
    /// 【2026-10-15 更正】原名 `Choice`（"选项/分支"）证伪：m_106 字节码 = 实体沿
    /// walkPathBuffer 8px/拍步进 + m_108 迁槽，无任何选项语义。
    EntityRoute,
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
            2 => Self::EntityRoute,
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
            Self::EntityRoute => 2,
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
    /// 2：探索确认弹窗（cameraLockedOnPlayer）
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
/// 证据：deobf a.java:9916-9922（差分 → 方向码映射）+ walkPhase 1 步进 switch。
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

/// 实体类目（Java `renderCategoryByType[type]`，a.java:6227-6269 初始化）。
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

    /// CES 延迟映射（a.java:8407-8408）：Cat1/Cat8/Cat16 → 5；Cat2/Cat4 → 8；默认 0。
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
    /// 上楼梯（spawnEntity 楼梯锚点 stairUpMarkerX/72）
    pub const STAIR_UP: i32 = 7;
    /// 下楼梯（stairDownMarkerX/73）
    pub const STAIR_DOWN: i32 = 8;
    /// 路线触发型（MOV 5 参分支写 entityAnimRow=1）
    pub const ROUTE: i32 = 72;
    /// 菩提老祖（序章 NPC；lockCameraOn 说话人特例 0/87 免查实体）
    pub const NPC_BODHI: i32 = 87;
    /// SWD 改写源/目标（封印门 → 可通行物件）
    pub const SEAL_GATE: i32 = 81;
    pub const SEAL_CONVERTED: i32 = 4;
    /// MVS 配对交换型
    pub const MVS_PAIR: i32 = 82;
}

/// 游戏模式（Java `gameMode`，FLD010；state-machine-survey §1.1 全表）。
///
/// **变体↔a.java case 号映射表**（件 A 验证三件之②——近形变体防张冠李戴，
/// BUG-007 同型反例；a.java 锚=run/m_000/paint 三 case 树）：
///
/// | 变体 | case | 语义 | a.java 锚 |
/// |---|---|---|---|
/// | Boot | 0 | 启动画面 | paint/run case 0 |
/// | MainMenu | 1 | 主菜单 | m_000 case 1 |
/// | Loading | 2 | 资源载入进度 | run case 2（:3449-3541） |
/// | Map | 3 | 地图主玩法 hub | run case 3（:3551-3679）/paint case 3（:2385-2422） |
/// | InGameMenu | 4 | 游戏内菜单 | m_000 case 4（:4326-4337）/run case 4（:3686-3735） |
/// | **EnemyInfo** | **5** | 敌人信息/帮助页 | run case 5（:3737-3755）/paint case 5（:2497-2560） |
/// | SaveSlot | 7 | 存档槽（7/8 共享） | m_000 case 7（:4342-4364）/run case 7（:3829-3878） |
/// | EquipDetail | 8 | 装备详情（7/8 共享） | 同 7 |
/// | Alchemy | 9 | 炼丹 | run case 9（:3885-3940）/paint case 9（:2690-2740） |
/// | ItemPanel | 10 | 道具面板 | m_000 case 10（:4370-4380）/run case 10（:3931-4009） |
/// | Dialog | 11 | 对白/剧情（脚本模式） | paint case 11（:2798-2896） |
/// | RecordBrowser | 12 | 记录浏览器 | overlay case5（:3325-3344）/paint_mode12 |
/// | IntroCutscene | 14 | 读条过场（新游戏） | run case 14（:4137） |
/// | **Help** | **15** | 帮助全文浮层 | m_000 case 15（:4402-4413）/paint case 15（:2958-2962） |
/// | About | 16 | 关于（图标条） | run case 16（:4143-4188）/paint case 16（:2963-2990） |
/// | IconView | 17 | 图标查看屏（floor>50 菜单项；视差背+68×68 图标条+kind0 浮层） | paint case 17（:2998-3002）/run 激活（:3415） |
/// | Shop | 19 | 商店（5 条目） | m_000 case 19（:4439-4448）/run case 19（:4196-4226） |
/// | Ending | 20 | END 结局过场 | m_000 case 20（:4443-4450）/run case 20（:4223-4228） |
/// | LogoAnim | 22 | logo 动画收尾（70 帧后 CMidlet.m_000 软复位） | paint case 22（:3074）/run case 22（:4249） |
/// | SoundAsk | 21 | 音效询问 | boot 序列 |
/// | NetWait | 99 | 联网等待（**DC-3 裁死**悬空态，枚举项保留作对照） | — |
/// | Unknown(i32) | — | 未观测值（D4 不发明） | — |
///
/// ⚠️ **近形防混淆**：EnemyInfo=**5**、Help=**15**（帮助全文）、About=**16**——
/// 三者 case 号相邻/语义近形，张冠李戴=BUG-007 同型（菜单文字-动作错位）。
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
    /// 4：游戏内菜单
    InGameMenu,
    /// 5：敌人信息/帮助页（⚠️≠15 Help——近形防混淆）
    EnemyInfo,
    /// 7：存档槽（7/8 共享）
    SaveSlot,
    /// 8：装备详情（7/8 共享）
    EquipDetail,
    /// 9：炼丹
    Alchemy,
    /// 10：道具面板
    ItemPanel,
    /// 11：对白/剧情模式（loadLevelScript 进入）
    Dialog,
    /// 12：记录浏览器
    RecordBrowser,
    /// 14：读条过场（新游戏）
    IntroCutscene,
    /// 15：帮助全文浮层（⚠️≠5 EnemyInfo——近形防混淆）
    Help,
    /// 16：关于（图标条）
    About,
    /// 17：图标查看屏（floor>50 菜单项；paint 视差背+68×68 图标条+kind0 浮层）——
    /// 语义名按观测渲染行为取，条目语义待证
    IconView,
    /// 19：商店（5 条目）
    Shop,
    /// 20：END 结束过场（70 帧后销毁）
    Ending,
    /// 22：logo 动画收尾（70 帧后 CMidlet.m_000 软复位）
    LogoAnim,
    /// 21：音效询问
    SoundAsk,
    /// 99：联网等待
    NetWait,
    Unknown(i32),
}

impl std::fmt::Display for GameMode {
    /// 显示=raw 号（诊断输出与旧 i32 口径逐字一致——零漂移）。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw())
    }
}

impl GameMode {
    pub fn from_raw(v: i32) -> Self {
        match v {
            0 => Self::Boot,
            1 => Self::MainMenu,
            2 => Self::Loading,
            3 => Self::Map,
            4 => Self::InGameMenu,
            5 => Self::EnemyInfo,
            7 => Self::SaveSlot,
            8 => Self::EquipDetail,
            9 => Self::Alchemy,
            10 => Self::ItemPanel,
            11 => Self::Dialog,
            12 => Self::RecordBrowser,
            14 => Self::IntroCutscene,
            15 => Self::Help,
            16 => Self::About,
            17 => Self::IconView,
            19 => Self::Shop,
            20 => Self::Ending,
            22 => Self::LogoAnim,
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
            Self::InGameMenu => 4,
            Self::EnemyInfo => 5,
            Self::SaveSlot => 7,
            Self::EquipDetail => 8,
            Self::Alchemy => 9,
            Self::ItemPanel => 10,
            Self::Dialog => 11,
            Self::RecordBrowser => 12,
            Self::IntroCutscene => 14,
            Self::Help => 15,
            Self::About => 16,
            Self::IconView => 17,
            Self::Shop => 19,
            Self::Ending => 20,
            Self::LogoAnim => 22,
            Self::SoundAsk => 21,
            Self::NetWait => 99,
            Self::Unknown(v) => v,
        }
    }
}

#[cfg(test)]
mod game_mode_wiring_tests {
    use super::GameMode;

    /// 件 A 验证三件之①：**Unknown(i32) 路径**（现状 0 测试 → 本测钉）。
    /// 未观测值 ⇒ Unknown 兜底（D4 不发明变体）；raw() 往返逐值保真；
    /// 21 观测值全量不落 Unknown（变体↔case 号映射表见 GameMode doc）。
    #[test]
    fn game_mode_unknown_roundtrip_preserves_raw() {
        for v in [-3, 6, 13, 18, 23, 42, 100, 255] {
            let m = GameMode::from_raw(v);
            assert!(matches!(m, GameMode::Unknown(x) if x == v), "from_raw({v}) ⇒ Unknown({v})");
            assert_eq!(m.raw(), v, "Unknown raw() 往返保真");
        }
        for v in [0, 1, 2, 3, 4, 5, 7, 8, 9, 10, 11, 12, 14, 15, 16, 17, 19, 20, 21, 22, 99] {
            assert_eq!(GameMode::from_raw(v).raw(), v, "观测值 {v} raw() 往返");
            assert!(!matches!(GameMode::from_raw(v), GameMode::Unknown(_)), "观测值 {v} 不落 Unknown");
        }
    }

    /// 近形变体防张冠李戴（BUG-007 同型）：EnemyInfo=5、Help=15、About=16 互不串。
    #[test]
    fn game_mode_near_shape_variants_do_not_mix() {
        assert_eq!(GameMode::EnemyInfo.raw(), 5);
        assert_eq!(GameMode::Help.raw(), 15);
        assert_eq!(GameMode::About.raw(), 16);
        assert_eq!(GameMode::IconView.raw(), 17);
        assert_eq!(GameMode::LogoAnim.raw(), 22);
    }
}
