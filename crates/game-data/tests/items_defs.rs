//! DDDSA P1-1 items.json 对账+五类拒载负例（defs.rs 真校验器）。

use game_data::defs::{parse_items, EffectKey, Reject};

const ITEMS: &str = include_str!("../../../data/tables/items.json");

/// 行数对账：items.json 行数=原表行数（可入栈 22（13..32+85/86）+装备 10=32）。
#[test]
fn items_row_count_matches_source_tables() {
    let rows = parse_items(ITEMS).expect("items.json 解载");
    assert_eq!(rows.len(), 32, "行数对账：22 可入栈+10 装备=32");
    // id 序唯一升序（保真 id 冻结不重排）
    for w in rows.windows(2) {
        assert!(w[0].id < w[1].id, "id 升序唯一");
    }
}

/// 强类型 EffectKey 抽查（禁 JSON map 弱类型——值在枚举里）。
#[test]
fn items_effect_keys_are_strong() {
    let rows = parse_items(ITEMS).expect("items.json 解载");
    let t13 = rows.iter().find(|r| r.id == 13).unwrap();
    assert!(matches!(t13.effect, EffectKey::OpenPanel { .. }), "13=OpenPanel");
    let t31 = rows.iter().find(|r| r.id == 31).unwrap();
    assert!(matches!(t31.effect, EffectKey::HpUp { mul: 50 }), "31=HpUp50");
}

/// ⑤ UnknownField：行键外拒载。
#[test]
fn items_reject_unknown_field() {
    let bad = ITEMS.replace("\"pickup_msg\"", "\"picku_msg\"");
    let err = parse_items(&bad).unwrap_err();
    assert!(matches!(err, Reject::UnknownField { .. }), "{err:?}");
}

/// ⑤ UnknownField：effect.key 枚举值外拒载。
#[test]
fn items_reject_unknown_effect_key() {
    let bad = ITEMS.replace("\"OpenPanel\"", "\"OpenPane1\"");
    let err = parse_items(&bad).unwrap_err();
    assert!(matches!(err, Reject::UnknownField { .. }), "{err:?}");
}

/// ③ DomainOverflow：id 域外拒载。
#[test]
fn items_reject_domain_overflow() {
    let bad = ITEMS.replace("\"id\": 13,", "\"id\": 999,");
    let err = parse_items(&bad).unwrap_err();
    assert!(matches!(err, Reject::DomainOverflow { .. }), "{err:?}");
}

/// ② BadForeignKey：icon.entity_type 域外拒载。
#[test]
fn items_reject_bad_fk() {
    let bad = ITEMS.replace("\"entity_type\": 13", "\"entity_type\": 999");
    let err = parse_items(&bad).unwrap_err();
    assert!(matches!(err, Reject::BadForeignKey { .. }), "{err:?}");
}

/// ④ ParamsRange：Heal div=0 拒载（FrameIndex 同族）。
#[test]
fn items_reject_bad_param_range() {
    let bad = ITEMS.replace("\"div\": 10", "\"div\": 0");
    let err = parse_items(&bad).unwrap_err();
    assert!(matches!(err, Reject::FrameIndexOutOfBounds { .. }), "{err:?}");
}
