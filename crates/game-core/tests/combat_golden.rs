//! P3.4 战斗模块 golden vectors（TDD）。
//!
//! 证据：deobf a.java:9940-9998（战斗数值链）/8081-8106（consumeKeyForDoor）/
//! 7480-7486（gainGold）/7180-7210（scaleEnemyStats）/1044-1118（基础表）；
//! gameplay-floor1 trace（tick1557 HP -8 / popup 值 27,27,8,3）；
//! 规格合同：docs/spec/gameplay.md R-battle-*（零随机、HP 下限 1）。

use game_core::combat::*;

fn player() -> PlayerCombat {
    PlayerCombat {
        hp: 500,
        atk: 30,
        def: 10,
        yellow_keys: 1,
        blue_keys: 0,
        red_keys: 0,
        gold: 100,
        item_types: vec![0; 16],
        item_uses: vec![0; 16],
        item_stack_size: 0,
    }
}

fn tables() -> CombatTables {
    CombatTables {
        enemy_base_hp: include_str!("fixtures/enemybasehp.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
        enemy_base_atk: include_str!("fixtures/enemybaseatk.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
        enemy_base_def: include_str!("fixtures/enemybasedef.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
        difficulty_multipliers: include_str!("fixtures/difficulty_multipliers.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
    }
}

#[test]
fn predict_battle_hp_loss_matches_deobf_arithmetic() {
    // 手算锚点（a.java:9952-9970）：atk=30 > edef=10；
    // hits = hp/(20) = 40/20 = 2（2*20=40 不<40）；hits>1 && eatk 20 > def 10
    // → (20-10)*(2-1)=10；!gate && eatk>def → +10 ⇒ 20
    assert_eq!(predict_battle_hp_loss(30, 10, 40, 20, 10, false), 20);
    // atk ≤ edef ⇒ -1（打不动）
    assert_eq!(predict_battle_hp_loss(10, 10, 40, 20, 10, false), -1);
    // gate=true ⇒ 无末段 +10：10*(1)=10
    assert_eq!(predict_battle_hp_loss(30, 10, 40, 20, 10, true), 10);
}

#[test]
fn scale_enemy_stats_multiplies_base() {
    let base = tables();
    let scaled = scale_enemy_stats(&base, 1);
    // trace 实证（floor1 敌 popup 27 = 敌 HP 基值域内；baseHp[0]=35 为首敌）
    assert_eq!(scaled.hp[0], base.enemy_base_hp[0]);
    assert_eq!(scaled.atk[0], base.enemy_base_atk[0]);
    let scaled30 = scale_enemy_stats(&base, 30);
    assert_eq!(scaled30.hp[0], base.enemy_base_hp[0] * 30);
}

#[test]
fn effective_attack_doubles_with_matching_item() {
    let mut p = player();
    p.atk = 30;
    // 特性位 &1 需要背包 23 号
    assert_eq!(effective_attack_vs_type(&p, 0b01), 30, "无 23 号不翻倍");
    p.item_types[0] = 23;
    p.item_stack_size = 1;
    assert_eq!(effective_attack_vs_type(&p, 0b01), 60, "23 号 + &1 ⇒ ×2");
    assert_eq!(effective_attack_vs_type(&p, 0b10), 30, "&2 需要 24 号");
    p.item_types[1] = 24;
    p.item_stack_size = 2;
    assert_eq!(effective_attack_vs_type(&p, 0b10), 60);
}

#[test]
fn consume_key_matches_door_color() {
    let mut p = player(); // 黄 1 蓝 0 红 0
    let (ok, popup) = consume_key_for_door(&mut p, 26);
    assert!(ok && popup == 2 && p.yellow_keys == 0, "黄门耗黄钥匙");
    assert!(!consume_key_for_door(&mut p, 26).0, "耗尽后失败");
    let (ok2, popup2) = consume_key_for_door(&mut p, 28);
    assert!(!ok2 && popup2 == 0, "蓝门无蓝钥匙");
    p.red_keys = 1;
    let (ok3, popup3) = consume_key_for_door(&mut p, 27);
    assert!(ok3 && popup3 == 1 && p.red_keys == 0, "红门耗红钥匙");
}

#[test]
fn gain_gold_adds_and_pops() {
    let mut p = player();
    assert_eq!(gain_gold(&mut p, 1), Some(1));
    assert_eq!(p.gold, 101, "trace tick1506：金 100→101");
    assert_eq!(gain_gold(&mut p, 0), None, "零增无弹窗");
}

#[test]
fn difficulty_scales_from_table() {
    // 难度倍率表（a.java:2236，夹具同值）：{1,30,60,80,100}——R-battle-14 口径：
    // 正常流程 mult = 表[difficultyIndex]，difficultyIndex 由存档 byte0 读入
    let base = tables();
    assert_eq!(base.difficulty_multipliers, vec![1, 30, 60, 80, 100]);
}
