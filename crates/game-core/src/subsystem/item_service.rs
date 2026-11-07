//! item_service face（P2 批6 终形）——ItemService：pickup/use 生命周期编排面（spec §3.1 §7-Q1 终案）。
//!
//! 【分发链唯一】Service→Registry::lookup→ItemEffect 序——现 execute_item_action 形保持
//! （item_effect.rs:59 ItemAction 语义冻结面），**禁第二分发链**。本面 pickup=分发核心
//! （pickup_kind_for_type+pickup_params_for+builtin_effect.apply——P1-3 一次性切的中间段单源）。
//!
//! 【HostCtx 摘 2】give_gold/pickup_item **逐字迁**（HostCtx 原签——ScriptEngine GIN 边界语义
//! script.rs GIN 臂 0/1；批5 处置账表 give_gold+pickup_item→批6 销账）。HostCtx 余 8。
//!
//! 【MENU_BLIND_ZONES】menu 域 dump 盲区（plan 批6 明文「menu 盲区登记」=plan:69 点名 4 项销账位）。

use crate::effect_registry::{ItemCtx, ItemOutcome};

/// spec §3.1：道具服务面（Registry=分发、Service=pickup 编排——§7-Q1）。
pub trait ItemService {
    /// pickup 生命周期核心（分发链唯一入口）：Registry::lookup→ItemEffect 序。
    fn pickup(&mut self, ctx: &mut ItemCtx, t: i32) -> ItemOutcome;
    /// give_gold 逐字迁（HostCtx script.rs:136 原签）——Host 边界方法（ScriptEngine GIN 面）。
    fn give_gold(&mut self, amount: i32);
    /// pickup_item 逐字迁（HostCtx script.rs:137 原签）——Host 边界方法（ScriptEngine GIN 面）。
    fn pickup_item(&mut self, item_id: i32);
}

/// 道具服务解析器（无状态 ZST——分发经 effect_registry 单源，写面经 ItemCtx）。
pub struct ItemResolver;

impl ItemService for ItemResolver {
    /// 分发链唯一（禁第二分发链）：kind/params 表查+builtin_effect(kind).apply。
    fn pickup(&mut self, ctx: &mut ItemCtx, t: i32) -> ItemOutcome {
        let kind = crate::effect_registry::pickup_kind_for_type(t);
        let params = crate::effect_registry::pickup_params_for(t);
        crate::effect_registry::builtin_effect(kind).apply(ctx, &params)
    }
    /// Host 边界方法（ScriptEngine 消费面）——resolver 不服务（批3 同款）。
    fn give_gold(&mut self, _: i32) {}
    /// Host 边界方法（ScriptEngine 消费面）——resolver 不服务（批3 同款）。
    fn pickup_item(&mut self, _: i32) {}
}

/// menu 域 dump 盲区清单（plan 批6 明文登记位；plan:69 点名 4 项销账）。
pub const MENU_BLIND_ZONES: &[&str] = &[
    "menuCursor（MenuState 菜单游标）：菜单导航写面——dump 盲区（state_dump 无 menuCursor）",
    "itemCursor（MenuState 道具面板游标）：item 面板选择写面——dump 盲区",
    "recordPage（MenuState 记录页）：open_record_page 翻页写面——dump 盲区",
    "difficultyIndex（MenuState 难度档）：op_res easy 判据+存档头写面——dump 盲区（state_dump 无 difficultyIndex）",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::PlayerCombat;
    use crate::effect_registry::{ItemCtx, ItemOutcome};

    fn pc() -> PlayerCombat {
        PlayerCombat {
            hp: 100, atk: 10, def: 5,
            yellow_keys: 0, blue_keys: 0, red_keys: 0, gold: 0,
            item_types: Vec::new(), item_uses: Vec::new(), item_stack_size: 0,
        }
    }

    fn ctx<'a>(
        player: &'a mut PlayerCombat,
        tut: &'a mut bool,
        ew: &'a mut i32,
        ea: &'a mut i32,
    ) -> ItemCtx<'a> {
        ItemCtx {
            player,
            floor: 1,
            tutorial_seen: tut,
            equipped_weapon: ew,
            equipped_armor: ea,
            equip_types: [0, 33, 34, 35, 79, 36, 0, 37, 38, 39, 80, 40],
            equip_bonuses: [0, 10, 30, 70, 120, 220, 220, 0, 0, 0, 0, 0],
        }
    }

    /// 【P2 批6 红测试】pickup 分发链单源钉（Service→Registry::lookup→ItemEffect 序——禁第二分发链）。
    #[test]
    fn p2b6_pickup_dispatch_single_source() {
        let mut p = pc();
        let (mut t, mut ew, mut ea) = (false, 0i32, 0i32);
        let mut c = ctx(&mut p, &mut t, &mut ew, &mut ea);
        let out = ItemResolver.pickup(&mut c, 25);
        drop(c);
        let kind = crate::effect_registry::pickup_kind_for_type(25);
        let params = crate::effect_registry::pickup_params_for(25);
        let mut p2 = pc();
        let (mut t2, mut ew2, mut ea2) = (false, 0i32, 0i32);
        let mut c2 = ctx(&mut p2, &mut t2, &mut ew2, &mut ea2);
        let expect = crate::effect_registry::builtin_effect(kind).apply(&mut c2, &params);
        drop(c2);
        assert_eq!(format!("{out:?}"), format!("{expect:?}"), "Service 分发=Registry 链唯一源");
    }

    /// 【P2 批6 红测试】Registry 单测无副作用断言（plan 批6 判据）。
    #[test]
    fn p2b6_registry_lookup_side_effect_free() {
        for t in [13, 25, 41, 79, 85] {
            let k1 = crate::effect_registry::pickup_kind_for_type(t);
            let p1 = format!("{:?}", crate::effect_registry::pickup_params_for(t));
            let k2 = crate::effect_registry::pickup_kind_for_type(t);
            let p2 = format!("{:?}", crate::effect_registry::pickup_params_for(t));
            assert_eq!(k1, k2, "kind 查表幂等（t={t}）");
            assert_eq!(p1, p2, "params 查表幂等（t={t}）");
        }
    }

    /// 【P2 批6 红测试】menu 盲区登记 plan:69 点名项销账（批3 红1 教训——本批登记位=plan 批6「menu 盲区登记」）。
    #[test]
    fn p2b6_menu_blind_zones_cover_plan_named() {
        let m = MENU_BLIND_ZONES.join("\n");
        for name in ["menuCursor", "itemCursor", "recordPage", "difficultyIndex"] {
            assert!(m.contains(name), "menu 盲区漏登记 plan:69 点名项：{name}");
        }
        let pend = crate::subsystem::script_host::PLAN_NAMED_PENDING.join("\n");
        assert!(!pend.contains("menu:"), "menu 4 项应销账（plan 批6 登记位）");
        assert!(!pend.contains("autoBattle"), "tap:autoBattle 已批7 销账（INPUT_BLIND_ZONES 归档）");
    }

    /// 【P2 批6 红测试】give_gold 逐字迁边界形（HostCtx 原签——sig_lock 孪生钉）。
    #[test]
    fn p2b6_boundary_method_shapes() {
        fn _g<T: ItemService>(t: &mut T, amount: i32) {
            t.give_gold(amount);
        }
        fn _p<T: ItemService>(t: &mut T, item_id: i32) {
            t.pickup_item(item_id);
        }
    }
}
