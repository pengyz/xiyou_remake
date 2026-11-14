//! 求解器初始状态与常量（批0）。
//!
//! **S0 考证（2026-11-09 重新取证，行号现场实测）**：
//! - **真源 = 原版新游戏加载链 step8**：`reference/src/deobf/a.java:3482-3491`
//!   （`case 8:` → `currentFloor = minFloorReached = 51` / `maxFloorReached = 51` /
//!   `playerHp=300` / `playerAtk=10` / `playerDef=10` / 三色钥匙=0）。
//! - 加载链定义：`crates/game-core/src/intro.rs:336-338` `LoadProgress::new_game()`
//!   `steps = &[6, 8, 5, 13, 9, 10, 2, 3, 12, 11]`（原版 `startNewGameLoad` a.java:7183-7194：
//!   :7183 方法声明、:7185-7194 十步同序；旧注 a.java:7172-7179 实测落在
//!   `changeFloor`(:7135-7181) 尾段属错锚——已由 fac2c65 注释级更正裁决【2026-11-10】）。
//! - Rust 侧接线（**内联等值落实**，非经 new_game_stats）：`shell/common/src/handoff.rs:158-163`
//!   `floor/min_floor/max_floor = 51; hp=300; atk=10; def=10`。
//! - `crates/game-core/src/intro.rs:363-365 new_game_stats() = (300,10,10,51)` = **未接线死码**
//!   （全仓 grep 零调用方；裁定 `docs/spec/dead-code-rulings.md:316` DC-8 ☠️——其内联锚
//!   `game-desktop/main.rs:766-771` / `preset_cases.rs:2600/2810` 已漂移，现锚=handoff.rs:158-163 /
//!   preset_cases.rs:2722/2941，四元组值一致无行为缺口）。
//! - 求解器**直接取常量不等接线**（计划 §2.5 批0 二选一之"取常量"支）。

/// S0 = (hp=300, atk=10, def=10, floor=51)（真源 a.java:3482-3491 step8；keys/gold 由
/// `ingest` 侧另行初始化为 0，本函数只承载四元组）。
pub fn s0() -> (i32, i32, i32, i32) {
    (300, 10, 10, 51)
}

/// Q3「终极装备集」（计划 §2.7 批0 裁定项）。
///
/// **裁定（用户 2026-11-09，A 案）**：`{36 金箍棒, 40 紫金龙鳞甲}`（tier 最高 bonus220 二件）；
/// 次级档 `{79 乌金棍, 80 银缕锁甲}`（bonus120）**不入禁**。裁定后本常量即现值，无待改项。
pub const Q3_EQUIP_SET: &[i32] = &[36, 40];

/// QoL 必收集集（解合法性=通关+QoL 全收集，计划 §1.2 口径细则 m03276）。
///
/// **裁定（用户 2026-11-12，F4-A 案，lead 传令）**：全收集口径 =
/// 「计划 §2.7 QoL 清单 ∩ 世界可获得」= `{15, 21, 23}`；**20/22/24 世界零获取**
/// ⇒ 收窄至可获得子集，不制造不可满足的全收集前提。A 级穷尽对照（命令与输出：
/// docs/findings/reviews/2026-11-13-solver-batch3-review.md §5.2）：sprite 55 层 1703 记录普查仅 15→f1(10,11)/21→f45(2,10)/23→f19(6,3)
/// 且 ty22 全世 0 枚；shop_goods 17 行无符（src0 idx21=GiveGold 非 item21）；
/// GIN×4 只发 13/19/金；type5 开箱只动画+removeGroupAtCellByType 无授予；
/// 战斗掉落只掉金。计划 :62/:165 已原地日期更正（保留原主张不删，AGENTS §3.6）。
pub const QOL_ITEMS: &[i32] = &[15, 21, 23];

// ================= 批3 C4：状态抽象（计划 §3.3 字段名不许漂） =================

/// 批3 裁定①（报审定死）：hp 离散桶宽=**1=精确整数**——Q1 要精确通关值、
/// 双目标 Pareto 已在同键内做 hp 支配压缩，再分桶只损精度不省键。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiscreteHp(pub i32);

impl DiscreteHp {
    /// 桶化（桶宽1=恒等；若改桶宽只动此函数并同步报审口径）。
    pub fn bucket(v: i32) -> Self {
        Self(v)
    }
}

/// 批3 裁定②（报审定死）：gold 离散桶宽=**1=精确整数**，且**进去重键 K₀′**
/// ——批2 伏笔「批3 商店层再入键」（批2 floor1 无商店未入键；批3 全局起有
/// 商店层，不同金币=不同购买能力不可合并）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiscreteGold(pub i32);

/// `script_line_flags` 位图（68 槽 ≤ u128；字段名 `ScriptFlags` 计划 §3.3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ScriptFlags(pub u128);

/// 背包 canonical（计划 §3.3 `inventory CanonicalBag`）：按 `item_stack_size`
/// **前缀截断**——越过 size 的尾槽垃圾字节不进去重键；size 内真差异保留
/// （类型/次数有序、引擎栈序=拾取序确定性：不排序不聚合=保真）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalBag {
    types: Vec<u8>,
    uses: Vec<u8>,
}

impl CanonicalBag {
    pub fn from_player(p: &crate::solve_local::PlayerSpec) -> Self {
        let n = (p.item_stack_size.max(0) as usize).min(p.item_types.len());
        let m = (p.item_stack_size.max(0) as usize).min(p.item_uses.len());
        Self {
            types: p.item_types[..n].to_vec(),
            uses: p.item_uses[..m].to_vec(),
        }
    }

    /// 去重键面（size 前缀对）。
    pub fn key(&self) -> (Vec<u8>, Vec<u8>) {
        (self.types.clone(), self.uses.clone())
    }
}

/// 计划 §3.3 层间传播状态（**字段名不许漂**：floor/hp/atk/def/keys/gold/
/// inventory/flags/kills/monsters_alive 十字段逐一对位）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolverState {
    pub floor: u8,
    pub hp: DiscreteHp,
    pub atk: i32,
    pub def: i32,
    /// (黄, 蓝, 红)（PlayerSpec 字段序）
    pub keys: (u8, u8, u8),
    pub gold: DiscreteGold,
    pub inventory: CanonicalBag,
    pub flags: ScriptFlags,
    pub kills: u32,
    pub monsters_alive: u32,
}
