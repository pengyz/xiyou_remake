//! 【P2 批9 红测试】BUG-051 给金浮层两站（bug-ledger:1001-1009 保真缺口·可达）：
//! ① op_gin kind1（a.java:8439 `gainGold(v, playerPixelX, playerPixelY)` 内弹 spawnPopup(4,v,px,py)）
//! ② 商人赠金 (0,1|15|21)（a.java:7336 同参）——修前双站丢 Option 不弹。

use super::scene_test_support;

/// BUG-051① op_gin 给金缺 kind4 浮层（Java gainGold 内弹——a.java:7527-7532 体）。
#[test]
fn p2b9_op_gin_gold_pops_kind4() {
    let mut s = scene_test_support::scene((128, 128), &[]);
    s.op_gin(1, 1000);
    assert_eq!(s.player.gold, 1000, "gold 入账（Java goldAmount+= 同值）");
    let act = s.popups.test_active();
    assert!(act.contains(&(4, 1000)), "BUG-051① op_gin 给金缺 kind4 浮层：{act:?}");
}

/// BUG-051② 商人赠金 (0,1|15|21) 缺 kind4 浮层（a.java:7336 同参）。
#[test]
fn p2b9_merchant_gift_gold_pops_kind4() {
    let mut s = scene_test_support::scene((128, 128), &[(77, 8, 8)]);
    // param=(flags<<8)|idx：flags=2（flags&2 门内）+idx=1 ⇒ 臂 (0,1) 赠金 1000
    s.entities.items[0].param = ((2i32 << 8) | 1) as i16;
    s.merchant_entity = 0;
    let follow_up = s.apply_merchant_offer(true);
    assert!(!follow_up, "flags=2 无后续挂起（follow_up 语义=返回值）");
    assert_eq!(s.player.gold, 1000, "赠金入账");
    let act = s.popups.test_active();
    assert!(act.contains(&(4, 1000)), "BUG-051② 商人赠金缺 kind4 浮层：{act:?}");
}
