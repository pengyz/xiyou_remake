//! battle face（P2 批0 骨架）——BattleSubsystem：resolve_bump 战斗解析面（spec §3.1）。
//!
//! 形照 spec §3.1：`fn resolve_bump(&mut self, ctx: &mut Ctx, t: EnemyTypeId) -> BattleOutcome`。
//! 实现迁移=批2（scene.rs 战斗簇→impl BattleSubsystem）。Ctx=BattleCtx（combat 域写面 10 槽）。

use crate::combat::{EnemyDef, PlayerCombat};

/// 【批2 占位】战斗结果（spec §3.1 BattleOutcome——批2 终形随实现迁移定）。
pub struct BattleOutcome;

/// 敌型 id（i32 newtype——a.java 型号 t 语义；newtype 防裸混用）。
pub struct EnemyTypeId(pub i32);

/// BattleCtx——BattleSubsystem 专属 Ctx（ItemCtx 先例形=借用槽+值拷贝纯数据，无方法）。
///
/// 写槽=field-ownership combat 域 10 字段（写面归因登记 face=BattleCtx）；
/// 读槽=player/enemies/enemy_base（只读借——读面不计写集）。
pub struct BattleCtx<'a> {
    // —— 写槽（combat 域 10）——
    pub battle_active: &'a mut bool,
    pub battle_anim_tick: &'a mut i32,
    pub battle_digit_by_type: &'a mut Vec<i32>,
    pub battle_enemy_hp: &'a mut i32,
    pub battle_finishing: &'a mut bool,
    pub battle_no_counter: &'a mut bool,
    pub battle_rerun_scene: &'a mut bool,
    pub battle_target: &'a mut i32,
    pub battle_tick: &'a mut i32,
    pub show_battle_digits: &'a mut bool,
    // —— 读槽（只读）——
    pub player: &'a PlayerCombat,
    pub enemies: &'a [EnemyDef],
    pub enemy_base: &'a [EnemyDef],
}

/// spec §3.1：战斗子系统。
pub trait BattleSubsystem {
    fn resolve_bump(&mut self, ctx: &mut BattleCtx, t: EnemyTypeId) -> BattleOutcome;
}
