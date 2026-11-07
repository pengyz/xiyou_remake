//! DDDSA P2 批9 shop_goods.json 对账+五类拒载负例+ItemRegistry 外键抽查。
//! 照 items.json/enemies.json 先例（id 升序+五类拒载）。

use game_data::defs::{parse_items, parse_shop_goods, Reject, ShopEffect};

const GOODS: &str = include_str!("../../../data/tables/shop_goods.json");
const ITEMS: &str = include_str!("../../../data/tables/items.json");

fn item_ids() -> Vec<i32> {
    parse_items(ITEMS).expect("ItemRegistry 加载").iter().map(|it| it.id).collect()
}

/// 正例：17 行解析+id 升序+src 双商+臂面属性束逐行钉。
#[test]
fn shop_goods_parses_seventeen_rows() {
    let rows = parse_shop_goods(GOODS, &item_ids()).expect("shop_goods 加载");
    assert_eq!(rows.len(), 17, "17 行（(0,*)5+(1,*)12）");
    let ids: Vec<i32> = rows.iter().map(|r| r.id).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "id 升序（先例）");
    assert!(rows.iter().any(|r| r.src == 0), "src0 商在册");
    assert!(rows.iter().any(|r| r.src == 1), "src1 商在册");
    // 臂面抽查（价/效果/提示逐臂）
    let gift = rows.iter().find(|r| r.src == 0 && r.idx == 1).unwrap();
    assert_eq!(gift.effect, ShopEffect::GiveGold { amount: 1000 }, "(0,1) 赠金 1000（a.java:7336）");
    assert_eq!(gift.price, 0, "免费行无价");
    let priced = rows.iter().find(|r| r.src == 1 && r.idx == 3).unwrap();
    assert_eq!(priced.price, 50, "(1,3) 价 50");
    assert_eq!(priced.effect, ShopEffect::AddKeys { blue: 1, yellow: 0, red: 0 });
    assert_eq!(priced.fail_hint, "没有足够的金钱");
    let swap = rows.iter().find(|r| r.src == 1 && r.idx == 8).unwrap();
    assert_eq!(swap.effect, ShopEffect::SwapYellowKey { gold: 100 }, "(1,8) 黄-1→¤+100");
    assert_eq!(swap.fail_hint, "没有黄钥匙", "键数门提示");
}

/// ⑤ UnknownField：行键外拒载。
#[test]
fn shop_goods_rejects_unknown_field() {
    let bad = GOODS.replace("\"price\": 50", "\"pirce\": 50");
    match parse_shop_goods(&bad, &item_ids()) {
        Err(Reject::UnknownField { .. }) => {}
        other => panic!("应 UnknownField 拒载：{other:?}"),
    }
}

/// id 稳定性先例：非升序拒载。
#[test]
fn shop_goods_rejects_non_ascending_id() {
    let bad = GOODS.replacen("\"id\": 2,", "\"id\": 0,", 1);
    match parse_shop_goods(&bad, &item_ids()) {
        Err(Reject::Schema { reason, .. }) => assert!(reason.contains("升序"), "{reason}"),
        other => panic!("应升序拒载：{other:?}"),
    }
}

/// ③ DomainOverflow：src 域外拒载。
#[test]
fn shop_goods_rejects_domain_overflow() {
    let bad = GOODS.replacen("\"src\": 0", "\"src\": 2", 1);
    match parse_shop_goods(&bad, &item_ids()) {
        Err(Reject::DomainOverflow { .. }) => {}
        other => panic!("应 DomainOverflow 拒载：{other:?}"),
    }
}

/// ② BadForeignKey：ItemRegistry 外键拒载（首个消费面——AddStack item 不在 items.json）。
#[test]
fn shop_goods_rejects_item_registry_bad_fk() {
    let bad = GOODS.replace("\"item\": 13", "\"item\": 999");
    match parse_shop_goods(&bad, &item_ids()) {
        Err(Reject::BadForeignKey { detail, .. }) => {
            assert!(detail.contains("ItemRegistry"), "{detail}");
        }
        other => panic!("应 BadForeignKey 拒载：{other:?}"),
    }
}

/// ① BadJson：坏 JSON 拒载。
#[test]
fn shop_goods_rejects_bad_json() {
    match parse_shop_goods("{ 坏", &item_ids()) {
        Err(Reject::BadJson { .. }) => {}
        other => panic!("应 BadJson 拒载：{other:?}"),
    }
}
