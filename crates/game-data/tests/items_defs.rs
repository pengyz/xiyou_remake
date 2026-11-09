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

/// BUG-052 红钉：装备 desc/pickup_msg 正源=equipDescriptions（a.java:930-945），
/// 不得是 miscTexts 系统提示语（BUG-052 错源标志串）。
#[test]
fn bug052_equip_descs_from_equip_descriptions() {
    let rows = parse_items(ITEMS).expect("items.json 解载");
    let expect = [
        (33, "木制长棍", "攻击+10"), (34, "乌黑油亮", "攻击+30"),
        (35, "银棍", "攻击+70"), (79, "木之精华", "攻击+120"),
        (36, "龙王后宫", "攻击+220"), (37, "布衣", "防御+10"),
        (38, "豹纹", "防御+30"), (39, "四级神仙证书", "防御+70"),
        (80, "华丽的装饰", "防御+120"), (40, "东海龙鳞", "防御+220"),
    ];
    for (id, word, stat) in expect {
        let r = rows.iter().find(|r| r.id == id).unwrap();
        assert!(r.desc.contains(word) && r.desc.contains(stat),
            "id{id} desc 应含 equipDescriptions 锚词「{word}·{stat}」，实={:.30}", r.desc);
        assert_eq!(r.desc, r.pickup_msg, "id{id} desc==pickup_msg（生成器同源）");
    }
    for r in &rows {
        assert!(!r.desc.contains("无法再下一层了"),
            "id{} desc 不得含系统提示语（BUG-052 错源标志）", r.id);
    }
}

/// BUG-053 红钉：id22 下行符=主动 ChangeFloorDown（与 id21 ChangeFloorUp 对称）。
#[test]
fn bug053_id22_effect_is_active_change_floor_down() {
    let rows = parse_items(ITEMS).expect("items.json 解载");
    let t21 = rows.iter().find(|r| r.id == 21).unwrap();
    assert!(matches!(t21.effect, EffectKey::ChangeFloorUp), "21=ChangeFloorUp 对称锚");
    let t22 = rows.iter().find(|r| r.id == 22).unwrap();
    assert!(matches!(t22.effect, EffectKey::ChangeFloorDown),
        "22=ChangeFloorDown（BUG-053 修 id22 部分），实={:?}", t22.effect);
}
