//! 求解器初始状态与常量（批0）。
//!
//! **S0 考证（2026-11-09 重新取证，行号现场实测）**：
//! - **真源 = 原版新游戏加载链 step8**：`reference/src/deobf/a.java:3482-3491`
//!   （`case 8:` → `currentFloor = minFloorReached = 51` / `maxFloorReached = 51` /
//!   `playerHp=300` / `playerAtk=10` / `playerDef=10` / 三色钥匙=0）。
//! - 加载链定义：`crates/game-core/src/intro.rs:334-336` `LoadProgress::new_game()`
//!   `steps = &[6, 8, 5, 13, 9, 10, 2, 3, 12, 11]`（原版 `startNewGameLoad` a.java:7183-7194：
//!   :7183 方法声明、:7185-7194 十步同序；`intro.rs:333` 旧注标 a.java:7172-7179 实测落在
//!   `changeFloor`(:7135-7181) 尾段，属错锚——game-core 非批0 授权改动面，列账待裁）。
//! - Rust 侧接线（**内联等值落实**，非经 new_game_stats）：`shell/common/src/handoff.rs:158-163`
//!   `floor/min_floor/max_floor = 51; hp=300; atk=10; def=10`。
//! - `crates/game-core/src/intro.rs:361-363 new_game_stats() = (300,10,10,51)` = **未接线死码**
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
