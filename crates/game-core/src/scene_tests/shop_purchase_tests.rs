//! 【P2 批9】商人购买夹具（plan:97 判据：src0/1 双商）+grep 双源=0 断言。
//! ShopGoods 表一次性切（15 match 臂→表查+dispatcher——价格/效果/提示 JSON 单源）。

use super::scene_test_support;

/// src0 商人（type77）：(0,2) AddStack{item:13} 免费行——表查+执行。
#[test]
fn p2b9_shop_src0_add_stack_from_table() {
    let mut s = scene_test_support::scene((128, 128), &[(77, 8, 8)]);
    // param=(flags<<8)|idx：flags=2（flags&2 门内）+idx=2 ⇒ 表行 (0,2) AddStack{item:13}
    s.entities.items[0].param = ((2i32 << 8) | 2) as i16;
    s.merchant_entity = 0;
    let before = s.player.gold;
    let follow_up = s.apply_merchant_offer(true);
    assert!(!follow_up, "flags=2 无挂起");
    assert_eq!(s.player.gold, before, "免费行不扣金（price=0 表行）");
    assert!(s.player.find_item(13) >= 0, "AddStack item 13 入栈（表行 effect）");
}

/// src1 商人（非 77 型）：(1,3) 花 50 蓝+1——价格/效果/提示三源全表。
#[test]
fn p2b9_shop_src1_purchase_priced_row() {
    let mut s = scene_test_support::scene((128, 128), &[(78, 8, 8)]);
    s.entities.items[0].param = ((2i32 << 8) | 3) as i16;
    s.merchant_entity = 0;
    s.player.gold = 100;
    let follow_up = s.apply_merchant_offer(true);
    assert!(!follow_up);
    assert_eq!(s.player.gold, 50, "扣 50（表行 price 单源）");
    assert_eq!(s.player.blue_keys, 1, "蓝+1（表行 effect）");
    // 穷购：fail_hint 单源（表行"没有足够的金钱"）+purchase_ok=false
    s.entities.items[0].param = ((2i32 << 8) | 3) as i16;
    s.player.gold = 0;
    s.player.blue_keys = 0;
    let follow_up = s.apply_merchant_offer(true);
    assert!(follow_up, "失败提示浮层挂起（follow_up=true 原语义）");
    assert_eq!(s.player.gold, 0, "未扣金（spend_gold 门）");
    assert_eq!(s.player.blue_keys, 0, "未加蓝（门失败不执行 effect）");
}

/// (1,8) SwapYellowKey 键数门（fail_hint=表行"没有黄钥匙"）。
#[test]
fn p2b9_shop_swap_yellow_key_gate() {
    let mut s = scene_test_support::scene((128, 128), &[(78, 8, 8)]);
    s.entities.items[0].param = ((2i32 << 8) | 8) as i16;
    s.merchant_entity = 0;
    s.player.yellow_keys = 1;
    s.player.gold = 0;
    let _ = s.apply_merchant_offer(true);
    assert_eq!(s.player.yellow_keys, 0, "黄-1（表行 effect）");
    assert_eq!(s.player.gold, 100, "¤+100 直加（Java 无浮层同形——BUG-051 审计恰2 点不含此臂）");
    s.entities.items[0].param = ((2i32 << 8) | 8) as i16;
    s.player.gold = 0;
    let follow_up = s.apply_merchant_offer(true);
    assert!(follow_up, "无黄钥匙提示挂起");
    assert_eq!(s.player.gold, 0);
}

/// 【grep 双源=0（plan:97 判据）】apply_merchant_offer 体无内联价/提示字面量。
#[test]
fn p2b9_shop_goods_single_source_no_inline_values() {
    let scene_src = include_str!("../scene.rs");
    let start = scene_src.find("fn apply_merchant_offer").expect("fn 在档");
    // 界定=apply_merchant_offer 函数体（到下一 fn replace_shell_rows 前止——
    // 超出体的后续 helper 不入判定面；体内注释亦须零字面=过扫更严）。
    // 【2026-11-08 更正】原注称 OFFER_TEXTS 报价单文案含「没有足够的金钱」子串
    // ——实测 0 处（复核建议②），失实注释删除：本界真正的意义=单源判定聚焦
    // 买臂效果面（收尾块/报价文案面均为消费方非价格源）。
    let end = scene_src[start..]
        .find("\n    fn ")
        .map(|i| start + i)
        .unwrap_or(scene_src.len());
    let body = &scene_src[start..end];
    for lit in ["spend_gold(50)", "spend_gold(200)", "spend_gold(800)", "spend_gold(1000)", "spend_gold(2000)", "spend_gold(4000)", "没有足够的金钱", "没有黄钥匙"] {
        assert!(!body.contains(lit), "双源残留：{lit}（价格/提示=shop_goods.json 单源）");
    }
}
