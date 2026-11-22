//! 怪物特化敏感表（提案 §1.4-d / §7 S2，单一真相源在 game-core rules/）。
//!
//! 规矩（提案 §4 归属裁定 + N7）：本模块**只导出、零复制公式**——损血数值一律
//! 委托 [`crate::combat::predict_battle_hp_loss`]，有效攻一律委托
//! [`crate::combat::effective_attack_vs_type`]（trait 23/24 ×2 语义由它单源持有）。
//! game-solver 只消费本模块导出面，禁止在 solver 侧重算公式。
//!
//! 阶跃点语义（与公式逐点对齐，combat.rs:106-131）：
//! - 打不动谓词：`eff_atk > e.def` 不成立 ⇒ loss = -1；
//! - 零伤阈值（防侧）：`def >= e.atk` ⇒ 反击分支不成立 ⇒ counter_loss = 0；
//! - 秒杀阈值（攻侧）：`eff_atk >= e.hp + e.def` ⇒ hits=1 ⇒ loss = 0；
//! - counter_loss 口径 = **撞面开战**（armor_gate=true，无先手位——battle.rs
//!   BumpEngage 分支），即求解器收割怪的真实损血口径。
//!
//! 难度倍率 [`crate::combat::scale_enemy_stats`] 整体平移阶跃点：调用方传**缩放后**
//! 的敌表，表内阶跃点随缩放自动平移（不复制缩放公式）。

use crate::combat::{effective_attack_vs_type, predict_battle_hp_loss, EnemyDef, PlayerCombat};

/// 一只怪的敏感表行（全部从 combat 公式直算，无独立常量）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitivityRow {
    pub ety: i32,
    pub trait_flags: u8,
    /// 当前口径下对该怪的有效攻（trait 23/24 ×2 已含）。
    pub eff_atk: i32,
    /// 打不动谓词的补：eff_atk > e.def。
    pub can_kill: bool,
    /// 秒杀阈值：eff_atk ≥ 此值 ⇒ counter_loss 必为 0（hits=1）。
    pub one_hit_atk: i32,
    /// 零伤阈值（防侧）：def ≥ 此值 ⇒ 反击损血为 0（前提 can_kill）。
    pub zero_loss_def: i32,
    /// 撞面口径当前损血（-1 = 打不动）。
    pub counter_loss: i32,
}

/// 敏感表查询面用的最小 PlayerCombat（只填 atk/def/背包 trait 位）。
fn probe_player(atk: i32, def: i32, hold23: bool, hold24: bool) -> PlayerCombat {
    let mut items: Vec<u8> = Vec::new();
    if hold23 {
        items.push(crate::combat::item_kind::TRAIT1_AMULET);
    }
    if hold24 {
        items.push(crate::combat::item_kind::TRAIT2_AMULET);
    }
    PlayerCombat {
        hp: 1,
        atk,
        def,
        yellow_keys: 0,
        blue_keys: 0,
        red_keys: 0,
        gold: 0,
        item_types: items.clone(),
        item_uses: vec![0; items.len()],
        item_stack_size: items.len() as i32,
    }
}

/// 对**缩放后**敌表逐怪导出敏感行（倍率维由调用方换表实现，见模块注）。
pub fn sensitivity_table(
    scaled: &[EnemyDef],
    atk: i32,
    def: i32,
    hold23: bool,
    hold24: bool,
) -> Vec<SensitivityRow> {
    let p = probe_player(atk, def, hold23, hold24);
    scaled
        .iter()
        .map(|e| {
            let eff = effective_attack_vs_type(&p, e.trait_flags);
            let can_kill = eff > e.def;
            // 撞面口径：armor_gate=true（无先手位，battle.rs BumpEngage 同参）。
            let loss = predict_battle_hp_loss(eff, def, e.hp, e.atk, e.def, true);
            SensitivityRow {
                ety: e.id,
                trait_flags: e.trait_flags,
                eff_atk: eff,
                can_kill,
                one_hit_atk: e.hp + e.def,
                zero_loss_def: e.atk,
                counter_loss: loss,
            }
        })
        .collect()
}

/// 攻侧断点集合（升序去重）：每怪「可击杀」阶跃 `def+1` 与「秒杀」阶跃 `hp+def`。
pub fn atk_breakpoints(scaled: &[EnemyDef]) -> Vec<i32> {
    let mut v: Vec<i32> = scaled
        .iter()
        .flat_map(|e| [e.def + 1, e.hp + e.def])
        .filter(|x| *x > 0)
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// 防侧断点集合（升序去重）：每怪「反击归零」阶跃 `atk`。
pub fn def_breakpoints(scaled: &[EnemyDef]) -> Vec<i32> {
    let mut v: Vec<i32> = scaled.iter().map(|e| e.atk).filter(|x| *x > 0).collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// 「+1 攻能掉到零伤档」的边际清单：当前 counter_loss > 0 且 atk+1 后 == 0
/// （两态都必须 can_kill；打不动→可杀的掉档不算零伤档）。
pub fn atk_tier_drops(
    scaled: &[EnemyDef],
    atk: i32,
    def: i32,
    hold23: bool,
    hold24: bool,
) -> Vec<i32> {
    let now = sensitivity_table(scaled, atk, def, hold23, hold24);
    let next = sensitivity_table(scaled, atk + 1, def, hold23, hold24);
    now.iter()
        .zip(next.iter())
        .filter(|(a, b)| {
            a.can_kill && b.can_kill && a.counter_loss > 0 && b.counter_loss == 0
        })
        .map(|(a, _)| a.ety)
        .collect()
}

/// 「+1 攻能掉到秒杀档」的边际清单。
pub fn one_hit_drops(
    scaled: &[EnemyDef],
    atk: i32,
    def: i32,
    hold23: bool,
    hold24: bool,
) -> Vec<i32> {
    let now = sensitivity_table(scaled, atk, def, hold23, hold24);
    let next = sensitivity_table(scaled, atk + 1, def, hold23, hold24);
    now.iter()
        .zip(next.iter())
        .filter(|(a, b)| a.counter_loss > 0 && b.counter_loss == 0 && b.eff_atk >= b.one_hit_atk)
        .map(|(a, _)| a.ety)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{predict_battle_hp_loss, scale_enemy_stats, trait_flags_for};

    /// 覆盖两 trait 位 + 普通怪的小表（id 对齐 enemy_base 语义：41+i）。
    fn base() -> Vec<EnemyDef> {
        vec![
            EnemyDef { id: 41, hp: 35, atk: 18, def: 1, gold: 1, trait_flags: trait_flags_for(41) },
            EnemyDef { id: 49, hp: 50, atk: 40, def: 10, gold: 9, trait_flags: trait_flags_for(49) }, // bit1(=1)→配 23
            EnemyDef { id: 69, hp: 1500, atk: 600, def: 250, gold: 800, trait_flags: trait_flags_for(69) }, // bit2(=2)→配 24
            EnemyDef { id: 75, hp: 100, atk: 65, def: 15, gold: 30, trait_flags: trait_flags_for(75) },
        ]
    }

    /// 判据 7（单一真相源）：表行损血与 combat 公式**独立重算**逐点一致，
    /// 含 9999 属性、trait 两种、倍率两档（×1 / ×30）全网格。
    #[test]
    fn counter_loss_matches_combat_formula_on_grid() {
        for mult in [1i32, 30] {
            let scaled = scale_enemy_stats(&base(), mult);
            for hold23 in [false, true] {
                for hold24 in [false, true] {
                    for atk in [0, 5, 10, 16, 100, 9999] {
                        for def in [0, 5, 10, 17, 18, 30, 600, 9999] {
                            let p = probe_player(atk, def, hold23, hold24);
                            for e in &scaled {
                                let row = sensitivity_table(&scaled, atk, def, hold23, hold24)
                                    .into_iter()
                                    .find(|r| r.ety == e.id)
                                    .unwrap();
                                let eff = effective_attack_vs_type(&p, e.trait_flags);
                                let expect =
                                    predict_battle_hp_loss(eff, def, e.hp, e.atk, e.def, true);
                                assert_eq!(row.counter_loss, expect, "ety={} atk={atk} def={def} mult={mult} h23={hold23} h24={hold24}", e.id);
                                assert_eq!(row.eff_atk, eff);
                                assert_eq!(row.can_kill, eff > e.def);
                            }
                        }
                    }
                }
            }
        }
    }

    /// 阶跃点精确性：id41（hp35 atk18 def1）——秒杀 36、零伤防 18，
    /// 阈值两侧行为与公式一致（先红判据：±1 必变色）。
    #[test]
    fn thresholds_are_exact_for_id41() {
        let scaled = scale_enemy_stats(&base(), 1);
        let row = |atk: i32, def: i32| {
            sensitivity_table(&scaled, atk, def, false, false)
                .into_iter()
                .find(|r| r.ety == 41)
                .unwrap()
        };
        assert_eq!(row(36, 5).counter_loss, 0, "秒杀阈值 35+1");
        assert!(row(35, 5).counter_loss > 0, "阈值下侧仍有损血");
        assert_eq!(row(20, 18).counter_loss, 0, "防侧零伤阈值 18");
        assert!(row(20, 17).counter_loss > 0, "防侧阈值下侧仍有损血");
        assert_eq!(row(1, 5).counter_loss, -1, "eff_atk<=def ⇒ 打不动");
    }

    /// 倍率整体平移阶跃点（F1 前提）：×30 后阶跃 = 基表阶跃 ×30。
    #[test]
    fn multiplier_shifts_breakpoints_linearly() {
        let base_bp: Vec<i32> = atk_breakpoints(&base());
        let scaled = scale_enemy_stats(&base(), 30);
        let shifted = atk_breakpoints(&scaled);
        // 缩放表上的阶跃 = 缩放后行值直算（def+1、hp+def 都在缩放域内），
        // 期望值从基表行独立构造。
        let expect: Vec<i32> = {
            let mut v: Vec<i32> = base()
                .iter()
                .flat_map(|e| [e.def * 30 + 1, (e.hp + e.def) * 30])
                .collect();
            v.sort_unstable();
            v.dedup();
            v
        };
        assert_eq!(shifted, expect);
        assert_eq!(
            def_breakpoints(&scaled),
            def_breakpoints(&base()).iter().map(|x| x * 30).collect::<Vec<i32>>()
        );
    }

    /// trait 两维：49（bit0）只配 23 翻倍、69（bit1）只配 24 翻倍，错配不翻倍。
    #[test]
    fn trait_amulets_double_only_matched_bits() {
        let scaled = scale_enemy_stats(&base(), 1);
        let row = |ety: i32, atk: i32, h23: bool, h24: bool| {
            sensitivity_table(&scaled, atk, 10, h23, h24)
                .into_iter()
                .find(|r| r.ety == ety)
                .unwrap()
        };
        // 49（bit0，def10）：不持/错配 eff=6 ≤10 打不动；持 23 eff=12>10 可杀。
        assert!(!row(49, 6, false, false).can_kill);
        assert!(row(49, 6, true, false).can_kill);
        assert!(!row(49, 6, false, true).can_kill, "24 对 bit0 怪不翻倍");
        assert!(!row(69, 126, true, false).can_kill, "23 对 bit1 怪不翻倍");
        // 69（bit1，def250）：持 24 时 eff=252>250 可杀；不持 eff=126 打不动。
        assert!(!row(69, 126, false, false).can_kill);
        assert!(row(69, 126, false, true).can_kill);
    }

    /// 边际清单：atk_tier_drops / one_hit_drops 在阈值±1 处变色。
    #[test]
    fn tier_drops_flip_at_breakpoints() {
        let scaled = scale_enemy_stats(&base(), 1);
        // def=5（低于 41/75 的 atk）：损血只在秒杀阶跃归零——两清单在阈值±1 变色。
        assert!(one_hit_drops(&scaled, 35, 5, false, false).contains(&41), "atk35→36 秒杀掉档");
        assert!(!one_hit_drops(&scaled, 36, 5, false, false).contains(&41));
        assert!(atk_tier_drops(&scaled, 35, 5, false, false).contains(&41));
        assert!(one_hit_drops(&scaled, 114, 5, false, false).contains(&75), "atk114→115 秒杀 id75");
        assert!(!one_hit_drops(&scaled, 115, 5, false, false).contains(&75));
        // def=65 ≥ 全部怪 atk（除 69）：41/49/75 恒零伤，不在零伤掉档清单里。
        let drops = atk_tier_drops(&scaled, 20, 65, false, false);
        assert!(!drops.contains(&41) && !drops.contains(&49) && !drops.contains(&75));
        assert!(!drops.contains(&69), "69 一击点在 1750，atk20→21 不掉档");
        assert!(atk_tier_drops(&scaled, 1749, 65, false, false).contains(&69));
    }

    /// 9999 属性：全表可杀且全零伤（含 69 这类高防怪——秒杀阶跃 1750 < 9999）。
    #[test]
    fn saturation_grid_zeroes_everything() {
        let scaled = scale_enemy_stats(&base(), 1);
        for r in sensitivity_table(&scaled, 9999, 9999, false, false) {
            assert!(r.can_kill && r.counter_loss == 0, "ety={}", r.ety);
        }
    }
}
