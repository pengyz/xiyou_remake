//! DDDSA P1-6 enemies.json 对账+五类拒载负例+trait 键抽查。

use game_data::defs::{parse_enemies, Reject};

const ENEMIES: &str = include_str!("../../../data/tables/enemies.json");

/// 行数对账：**35 行**=源表长（a.java enemyBaseHp/Atk/Def/Gold 各 35——
/// 2026-11-06 实测校正：lead 89 数=TYPE 域宽误，非敌表长）；id=41..75。
#[test]
fn enemies_row_count_matches_source() {
    let (rows, mult) = parse_enemies(ENEMIES).expect("enemies 解载");
    assert_eq!(rows.len(), 35, "行数对账=源表长 35");
    assert_eq!(mult, vec![1, 30, 60, 80, 100], "difficultyMultipliers 5 项（a.java:2236）");
    for (i, r) in rows.iter().enumerate() {
        assert_eq!(r.id, 41 + i as i32, "id=41..75 横索引形");
    }
}

/// trait 行为键抽查（a.java:6266-6267 稀疏逐字）。
#[test]
fn enemies_trait_keys_spot() {
    let (rows, _) = parse_enemies(ENEMIES).expect("enemies 解载");
    let g = |id: i32| rows.iter().find(|r| r.id == id).unwrap().trait_flags;
    assert_eq!(g(49), 1, "49=1（a.java:6266）");
    assert_eq!(g(53), 1);
    assert_eq!(g(74), 1);
    assert_eq!(g(69), 2, "69=2（a.java:6267）");
    assert_eq!(g(41), 0, "无特性=0");
}

/// 拒载负例 **3 个**（2026-11-06 amend 改口：前 doc 列 4 项含"非整数"
/// ——实测无此测=超宣；实存=未知字段/域溢出/params 溢出三负例）。
#[test]
fn enemies_reject_unknown_field() {
    let bad = ENEMIES.replace("\"trait_flags\"", "\"trait_flag\"");
    assert!(matches!(parse_enemies(&bad).unwrap_err(), Reject::UnknownField { .. }));
}
#[test]
fn enemies_reject_domain_overflow() {
    let bad = ENEMIES.replace("\"id\": 41,", "\"id\": 999,");
    assert!(matches!(parse_enemies(&bad).unwrap_err(), Reject::DomainOverflow { .. }));
}
#[test]
fn enemies_reject_param_overflow() {
    let bad = ENEMIES.replace("\"trait_flags\": 0", "\"trait_flags\": 999");
    assert!(matches!(parse_enemies(&bad).unwrap_err(), Reject::FrameIndexOutOfBounds { .. }));
}

/// C1 演练（新增敌人=表行零改码一例）：内存追加行→解载过=数据面扩展性钉
/// （加行零改码的机器面：解析面无 per-id 特判——drill 脚本段③同款判据）。
#[test]
fn c1_drill_add_enemy_row_zero_code() {
    let row = "{\"id\": 76, \"hp\": 1, \"atk\": 1, \"def\": 1, \"gold\": 1, \"behavior\": {\"trait_flags\": 0}, \"evidence\": {\"hp\": \"drill\", \"atk\": \"drill\", \"def\": \"drill\", \"gold\": \"drill\", \"behavior\": \"drill\"}}";
    // 末位追加（rows 数组收尾——replace 全命中曾破坏 difficulty 数组实录）
    let (head, tail) = ENEMIES.rsplit_once("\n ]").expect("rows 收尾锚");
    let extra = format!("{head},\n  {row}\n ]{tail}");
    let (rows, _) = parse_enemies(&extra).expect("追加行解载过（数据面零改码）");
    assert_eq!(rows.len(), 36, "演练行入表");
}
