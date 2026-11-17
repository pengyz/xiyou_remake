//! 战斗模块 —— tickBattle 数值链的忠实移植（零随机，D4/规格 R-battle-*）。
//!
//! 证据：reference/src/deobf/a.java
//! - effectiveAttackVsType :9940-9950（双武器/防具道具 ×2）
//! - predictBattleHpLoss :9952-9970（静态纯函数）
//! - predictHpLossVsType :9937-9939（参数桥接）
//! - tickBattle 伤害段 :9971-9998（battleTick&3 节拍、敌先攻、反击预测）
//! - scaleEnemyStats :7180-7210（scaled = base × multiplier）
//! - consumeKeyForDoor :8081-8106（26 黄/27 红/28 蓝，spawnPopup kind1）
//! - gainGold :7480-7486（popup kind4）

/// 敌方基础数值表 ×3 + 难度倍率（a.java:1044-1118/2236；夹具同值）。
pub struct CombatTables {
    pub enemy_base_hp: Vec<i32>,
    pub enemy_base_atk: Vec<i32>,
    pub enemy_base_def: Vec<i32>,
    /// difficultyMultipliers = {1, 30, 60, 80, 100}（a.java:2236）
    pub difficulty_multipliers: Vec<i32>,
}

impl CombatTables {
    /// 【P1-6 一次性切】四表横列→EnemyDef 逐行聚合（gold=独立表传参——
    /// 消费面=记录（SoA 零残留）；装载入形保留=fixture/生成器数据形）。
    pub fn to_records(&self, gold: &[i32]) -> Vec<game_data::defs::EnemyDef> {
        (0..self.enemy_base_hp.len())
            .map(|i| game_data::defs::EnemyDef {
                id: 41 + i as i32,
                hp: self.enemy_base_hp[i],
                atk: self.enemy_base_atk[i],
                def: self.enemy_base_def[i],
                gold: gold[i],
                trait_flags: trait_flags_for(41 + i as i32),
            })
            .collect()
    }
}

/// traitFlagsByType 行为键（a.java:6266-6267 稀疏逐字：49/53/74=1、69=2）
/// ——唯一源（scene 侧 trait_flags 表查已并入记录）。
pub use game_data::defs::EnemyDef;

pub fn trait_flags_for(t: i32) -> u8 {
    match t {
        49 | 53 | 74 => 1,
        69 => 2,
        _ => 0,
    }
}

/// 玩家侧战斗相关字段（RES_0 重置集与 tickBattle 读写面）。
pub struct PlayerCombat {
    pub hp: i32,
    pub atk: i32,
    pub def: i32,
    pub yellow_keys: i32,
    pub blue_keys: i32,
    pub red_keys: i32,
    pub gold: i32,
    /// itemStackTypes（物品栈类型，容量 = offerQueueSrc）
    pub item_types: Vec<u8>,
    pub item_uses: Vec<u8>,
    pub item_stack_size: i32,
}

impl PlayerCombat {
    /// findItemStackIndex 忠实移植（a.java:8485 区域）：按类型找物品栈下标，无则 -1。
    pub fn find_item(&self, item_type: u8) -> i32 {
        for i in 0..self.item_stack_size as usize {
            if self.item_types[i] == item_type {
                return i as i32;
            }
        }
        -1
    }
}

/// 敌方缩放三表（scaleEnemyStats a.java:7203-7227：scaled = base × multiplier）。
/// 敌方缩放（scaleEnemyStats a.java:7203-7227：scaled=base×multiplier——
/// hp/atk/def 乘算，gold/trait_flags 透传）。
pub fn scale_enemy_stats(base: &[game_data::defs::EnemyDef], multiplier: i32) -> Vec<game_data::defs::EnemyDef> {
    base.iter()
        .map(|r| game_data::defs::EnemyDef {
            id: r.id,
            hp: r.hp * multiplier,
            atk: r.atk * multiplier,
            def: r.def * multiplier,
            gold: r.gold,
            trait_flags: r.trait_flags,
        })
        .collect()
}

pub fn effective_attack_vs_type(player: &PlayerCombat, trait_flags: u8) -> i32 {
    use item_kind::{TRAIT1_AMULET, TRAIT2_AMULET};
    let mut mult = 1;
    if (trait_flags & 1) != 0 && player.find_item(TRAIT1_AMULET) >= 0 {
        mult = 2;
    } else if (trait_flags & 2) != 0 && player.find_item(TRAIT2_AMULET) >= 0 {
        mult = 2;
    }
    player.atk * mult
}

/// predictBattleHpLoss 纯函数忠实移植（a.java:9981-9993）。
/// 参数序 = Java 调用点（a.java:9967）：(atk, def, enemy_hp, enemy_atk, enemy_def, has_armor_gate)
pub fn predict_battle_hp_loss(
    player_atk: i32,
    player_def: i32,
    enemy_hp: i32,
    enemy_atk: i32,
    enemy_def: i32,
    armor_gate: bool,
) -> i32 {
    let mut result = -1;
    let mut hits;
    if player_atk > enemy_def {
        hits = enemy_hp / (player_atk - enemy_def);
        if hits * (player_atk - enemy_def) < enemy_hp {
            hits += 1;
        }
        if hits > 1 && enemy_atk > player_def {
            result = (enemy_atk - player_def) * (hits - 1);
        } else {
            result = 0;
        }
        if !armor_gate && enemy_atk > player_def {
            result += enemy_atk - player_def;
        }
    }
    result
}

// 【2026-10-17 删除·DC-9 ☠️】predictHpLossVsType 参数桥（原 combat.rs:107-122）
// 已按 docs/spec/dead-code-rulings.md DC-9 裁定移除：零调用 + **桥公式缺道具
// 23/24 二倍攻击加成**（用 player.atk 而非 effectiveAttackVsType，a.java:9997-10004）
// ——带缺陷的冗余桥，幸缺陷未传播。忠实语义由 scene.rs GameScene::predict_hp_loss
// 持有（与 a.java:9992-9995 逐项对上）。**不得"顺手修桥再接线"**：若未来遇敌预演
// UI（a.java:8774/8852 臂）需要此桥，按 DC-9 对照表改用 effective_attack 公式。

/// spawnPopup kind 常量（a.java:9679-9686 分支语义 + tickBattle 写点）。
pub mod popup_kind {
    /// 1：左移小字（钥匙消耗）
    pub const LEFT_SHIFT: i32 = 1;
    /// 2：玩家侧预测伤害数字（kind2 走数字宽度居中分支）
    pub const PLAYER_DAMAGE: i32 = 2;
    /// 4：特殊（y/宽常量 240/30）
    pub const SPECIAL: i32 = 4;
    /// 5..=7：彩色右移（tickBattle 按 battleTick%3 选色）
    pub const ENEMY_DAMAGE_BASE: i32 = 5;
}

/// 门码（consumeKeyForDoor 参数；a.java:8104-8123 + interactWithCell 对账）。
pub mod door_code {
    pub const YELLOW: u8 = 26;
    pub const RED: u8 = 27;
    pub const BLUE: u8 = 28;
}

/// 战斗特性道具（effectiveAttackVsType 背包检索；a.java:9973-9971）。
pub mod item_kind {
    /// 特性位 &1 配对：攻击 ×2
    pub const TRAIT1_AMULET: u8 = 23;
    /// 特性位 &2 配对：攻击 ×2
    pub const TRAIT2_AMULET: u8 = 24;
}

/// consumeKeyForDoor 忠实移植（a.java:8104-8123）：
/// 26=黄门（popup 1,2）/ 27=红门（popup 1,1）/ 28=蓝门（popup 1,0）。
/// 返回 (是否消耗, popup_value)。
pub fn consume_key_for_door(player: &mut PlayerCombat, door: u8) -> (bool, i32) {
    use door_code::{BLUE, RED, YELLOW};
    match door {
        YELLOW if player.yellow_keys > 0 => {
            player.yellow_keys -= 1;
            (true, 2)
        }
        RED if player.red_keys > 0 => {
            player.red_keys -= 1;
            (true, 1)
        }
        BLUE if player.blue_keys > 0 => {
            player.blue_keys -= 1;
            (true, 0)
        }
        _ => (false, 0),
    }
}

/// gainGold 的**唯一实现**：gold += n；n>0 时返回 popup 值。
/// 拆出 `&mut i32` 形是为了让求解器（只有 `PlayerSpec.gold` 字段，没有 `PlayerCombat`）
/// 也能走同一段代码，而不是各自 `gold += n` 另立一份。
pub fn gain_gold_to(gold: &mut i32, amount: i32) -> Option<i32> {
    *gold += amount;
    if amount > 0 {
        Some(amount)
    } else {
        None
    }
}

/// gainGold 忠实移植（a.java:7503-7503）：gold += n；n>0 时 popup kind4。
/// 返回 popup 值（n>0 时 Some(n)）。
pub fn gain_gold(player: &mut PlayerCombat, amount: i32) -> Option<i32> {
    gain_gold_to(&mut player.gold, amount)
}

/// 击杀赏金的**单一源**（C6d）：引擎 `GameScene::kill_adjacent_at` 与求解器同调。
///
/// `gold = enemy_base[ety - 41].gold`；持 item25（幸运币）时双倍。
/// 证据：原内联处 `crates/game-core/src/scene.rs:2301-2309`；索引约定由
/// `CombatTables::to_records` 保证（`enemy_base[i].id == 41 + i`，combat.rs:24-35）。
/// 表缺行/越界 ⇒ 0（引擎面该分支受 `t < 67 && category == 8` 守护，越界不可达）。
///
/// 为什么必须在这里而不是求解器里：求解器此前自写 `kill_bounty`，与引擎内联公式
/// 各存一份 ⇒ 任何一侧改动（新增饰品倍率、赏金曲线调整）都会静默分叉，而这类分叉
/// 不会让任何测试失败，只会让「界内无解」多一个无法归因的理由。
pub fn kill_reward(enemy_base: &[EnemyDef], ety: i32, player: &PlayerCombat) -> i32 {
    let base = enemy_base
        .get((ety - 41) as usize)
        .map(|e| e.gold)
        .unwrap_or(0);
    if player.find_item(25) >= 0 {
        base << 1
    } else {
        base
    }
}

#[cfg(test)]
mod kill_reward_tests {
    use super::*;

    fn player(items: &[u8]) -> PlayerCombat {
        PlayerCombat {
            hp: 100,
            atk: 10,
            def: 5,
            yellow_keys: 0,
            blue_keys: 0,
            red_keys: 0,
            gold: 0,
            item_types: items.to_vec(),
            item_uses: vec![0; items.len()],
            item_stack_size: items.len() as i32,
        }
    }

    fn base() -> Vec<EnemyDef> {
        vec![
            EnemyDef { id: 41, hp: 10, atk: 1, def: 0, gold: 7, trait_flags: 0 },
            EnemyDef { id: 42, hp: 20, atk: 2, def: 0, gold: 13, trait_flags: 0 },
        ]
    }

    /// C6d 判据 1：赏金按 `id = 41 + i` 取行（不是按数组下标猜）。
    #[test]
    fn c6d_kill_reward_reads_by_id_offset() {
        let b = base();
        assert_eq!(kill_reward(&b, 41, &player(&[])), 7);
        assert_eq!(kill_reward(&b, 42, &player(&[])), 13);
    }

    /// C6d 判据 2：持 item25（幸运币）双倍——变异点，删掉 `<< 1` 立刻红。
    #[test]
    fn c6d_kill_reward_doubles_with_item25() {
        let b = base();
        assert_eq!(kill_reward(&b, 42, &player(&[25])), 26);
        assert_eq!(kill_reward(&b, 42, &player(&[25, 13])), 26);
        assert_eq!(kill_reward(&b, 42, &player(&[13])), 13);
    }

    /// C6d 判据 3：越界/缺行 ⇒ 0（不 panic、不虚构赏金）。
    #[test]
    fn c6d_kill_reward_out_of_range_is_zero() {
        let b = base();
        assert_eq!(kill_reward(&b, 99, &player(&[25])), 0);
        assert_eq!(kill_reward(&[], 41, &player(&[25])), 0);
    }

    /// C6d 判据 4：落账唯一实现——正增量返 popup 值，零/负增量不返但仍落账。
    #[test]
    fn c6d_gain_gold_to_is_single_impl() {
        let mut g = 5;
        assert_eq!(gain_gold_to(&mut g, 7), Some(7));
        assert_eq!(g, 12);
        assert_eq!(gain_gold_to(&mut g, 0), None);
        assert_eq!(gain_gold_to(&mut g, -3), None);
        assert_eq!(g, 9);
    }
}
