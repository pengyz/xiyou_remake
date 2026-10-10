//! 战斗模块 —— tickBattle 数值链的忠实移植（零随机，D4/规格 R-battle-*）。
//!
//! 证据：reference/src/deobf/a.java
//! - effectiveAttackVsType :9940-9950（双武器/防具道具 ×2）
//! - predictBattleHpLoss :9952-9970（静态纯函数）
//! - predictHpLossVsType :9937-9939（参数桥接）
//! - tickBattle 伤害段 :9971-9998（f_int_151&3 节拍、敌先攻、反击预测）
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

/// 玩家侧战斗相关字段（RES_0 重置集与 tickBattle 读写面）。
pub struct PlayerCombat {
    pub hp: i32,
    pub atk: i32,
    pub def: i32,
    pub yellow_keys: i32,
    pub blue_keys: i32,
    pub red_keys: i32,
    pub gold: i32,
    /// itemStackTypes（物品栈类型，容量 = f_int_88）
    pub item_types: Vec<u8>,
    pub item_uses: Vec<u8>,
    pub item_stack_size: i32,
}

impl PlayerCombat {
    /// m_081 忠实移植（a.java:8462 区域）：按类型找物品栈下标，无则 -1。
    pub fn find_item(&self, item_type: u8) -> i32 {
        for i in 0..self.item_stack_size as usize {
            if self.item_types[i] == item_type {
                return i as i32;
            }
        }
        -1
    }
}

/// 敌方缩放三表（scaleEnemyStats a.java:7180-7210：scaled = base × multiplier）。
#[derive(Default)]
pub struct ScaledEnemies {
    pub atk: Vec<i32>,
    pub def: Vec<i32>,
    pub hp: Vec<i32>,
}

pub fn scale_enemy_stats(base: &CombatTables, multiplier: i32) -> ScaledEnemies {
    ScaledEnemies {
        atk: base.enemy_base_atk.iter().map(|v| v * multiplier).collect(),
        def: base.enemy_base_def.iter().map(|v| v * multiplier).collect(),
        hp: base.enemy_base_hp.iter().map(|v| v * multiplier).collect(),
    }
}

/// effectiveAttackVsType 忠实移植（a.java:9940-9950）：
/// 特性位 &1 + 背包有 23 号 → ×2；特性位 &2 + 背包有 24 号 → ×2；返回 atk × 倍。
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

/// predictBattleHpLoss 纯函数忠实移植（a.java:9952-9970）。
/// 参数序 = Java 调用点（a.java:9938）：(atk, def, enemy_hp, enemy_atk, enemy_def, has_armor_gate)
pub fn predict_battle_hp_loss(
    player_atk: i32,
    player_def: i32,
    enemy_hp: i32,
    enemy_atk: i32,
    enemy_def: i32,
    armor_gate: bool,
) -> i32 {
    let mut result = -1;
    let mut hits = 0;
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

/// predictHpLossVsType 参数桥接（a.java:9937-9939）。
pub fn predict_hp_loss_vs_type(
    player: &PlayerCombat,
    scaled: &ScaledEnemies,
    type_idx_base: usize,
    armor_gate: bool,
) -> i32 {
    predict_battle_hp_loss(
        player.atk,
        player.def,
        scaled.hp[type_idx_base],
        scaled.atk[type_idx_base],
        scaled.def[type_idx_base],
        armor_gate,
    )
}

/// spawnPopup kind 常量（a.java:9650-9663 分支语义 + tickBattle 写点）。
pub mod popup_kind {
    /// 1：左移小字（钥匙消耗）
    pub const LEFT_SHIFT: i32 = 1;
    /// 2：玩家侧预测伤害数字（kind2 走数字宽度居中分支）
    pub const PLAYER_DAMAGE: i32 = 2;
    /// 4：特殊（y/宽常量 240/30）
    pub const SPECIAL: i32 = 4;
    /// 5..=7：彩色右移（tickBattle 按 f_int_151%3 选色）
    pub const ENEMY_DAMAGE_BASE: i32 = 5;
}

/// 门码（consumeKeyForDoor 参数；a.java:8081-8106 + interactWithCell 对账）。
pub mod door_code {
    pub const YELLOW: u8 = 26;
    pub const RED: u8 = 27;
    pub const BLUE: u8 = 28;
}

/// 战斗特性道具（effectiveAttackVsType 背包检索；a.java:9944-9948）。
pub mod item_kind {
    /// 特性位 &1 配对：攻击 ×2
    pub const TRAIT1_AMULET: u8 = 23;
    /// 特性位 &2 配对：攻击 ×2
    pub const TRAIT2_AMULET: u8 = 24;
}

/// consumeKeyForDoor 忠实移植（a.java:8081-8106）：
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

/// gainGold 忠实移植（a.java:7480-7486）：gold += n；n>0 时 popup kind4。
/// 返回 popup 值（n>0 时 Some(n)）。
pub fn gain_gold(player: &mut PlayerCombat, amount: i32) -> Option<i32> {
    player.gold += amount;
    if amount > 0 {
        Some(amount)
    } else {
        None
    }
}
