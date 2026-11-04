//! DDDSA P1-2 ItemEffect trait+内建注册（spec v3 §2.2）——行为可插拔面。
//!
//! **逐臂对照表**（67 行 match→注册行映射；a.java 锚随行）：
//!
//! | 臂 | 语义（pickup_item_type 逐字抽象） | 注册行 | 锚 |
//! |---|---|---|---|
//! | 13..=25\|85\|86 | （15=tutorialSeen=true）+addItemToItemStack | `Stack` | a.java:7528-7555 |
//! | 26/27/28 | yellow/red/blue KeyCount++ | `Key` | a.java:7556-7564 |
//! | 29/30 | floor≤10 +1 否则 +floorTier | `StatUp` | a.java:7565-7580 |
//! | 31/32 | applyHpDelta(50/200×tier) | `HpUp` | a.java:7581-7587 |
//! | 33..=36\|79 | 武器 tier {1,2,3,4,5} tryEquip | `Equip` | a.java:7632-7648 |
//! | 37..=40\|80 | 防具 tier {7,8,9,10,11} tryEquip | `Equip` | a.java:7632-7648 |
//! | default | return（41+ 不可达） | `NoOp` | a.java:7630-7631 |
//!
//! **两面并存**（P1-2 红线）：pickup_item_type 23 臂 switch **本件不动**（P1-3
//! 一次性切）——注册表侧行为面建好+等价实证；等价三层=锚+夹具 21 臂（全绿
//! 在册）+金样 74 全同每步。
//!
//! **Ctx 三宪**（spec §3.2）：ItemCtx=纯数据（引用槽位）/字段冻结（结构固定）/
//! apply 签名=ctx+≤2（params、无第三参）。
//!
//! **未知 effect 两层语义**（spec 终裁）：加载期拒载（defs.rs 未知键=P1-1 已
//! 钉）+运行期 no-op 不消耗（a.java:7630-7631 `default: return`——registry
//! miss→[`ItemOutcome::NoOp`]）。

use crate::combat::PlayerCombat;
use crate::entity::EntityTable;

/// 强类型效果参数（行为面——与 game-data EffectKey 载荷镜像）。
#[derive(Debug, Clone, PartialEq)]
pub enum EffectParams {
    Stack { type_id: i32, tutorial: bool },
    Key { yellow: bool, red: bool, blue: bool },
    StatUp { atk: i32, def: i32 },
    HpUp { delta: i32 },
    Equip { tier: i32 },
}

/// 动作结果（纯数据 outcome——副作用请求上交编排面执行）。
#[derive(Debug, Clone, PartialEq)]
pub enum ItemOutcome {
    /// 消耗（true 臂：栈增/键增/属性增/换装成功/加血）。
    Consumed { overlay_tier: Option<i32> },
    /// 未消耗（装备失败臂：已有更强装备——浮层提示）。
    NotConsumed { fail_msg: bool },
    /// 未知 effect/默认臂（a.java:7630-7631 return）：不消耗无副作用。
    NoOp,
}

/// Ctx 三宪：纯数据引用槽+字段冻结（结构固定不增）。
pub struct ItemCtx<'a> {
    pub player: &'a mut PlayerCombat,
    pub entities: &'a mut EntityTable,
    pub floor: i32,
    pub tutorial_seen: &'a mut bool,
    pub equipped_weapon: &'a mut i32,
    pub equipped_armor: &'a mut i32,
    pub equip_types: [i32; 12],
    pub equip_bonuses: [i32; 12],
}

impl ItemCtx<'_> {
    /// floorTier（a.java:7651-7662 逐字：(f-1)/10 钳 0；f>50→0；+1）。
    pub fn floor_tier(&self) -> i32 {
        let mut t = (self.floor - 1) / 10;
        if t < 0 {
            t = 0;
        }
        if self.floor > 50 {
            t = 0;
        }
        t + 1
    }
}

/// ItemEffect trait（行为可插拔面——apply 签名=ctx+≤2 按三宪）。
pub trait ItemEffect {
    fn apply(&self, ctx: &mut ItemCtx<'_>, params: &EffectParams) -> ItemOutcome;
}

// ── 内建效果（67 行 match 逐字抽象——逐臂等价三层：锚/夹具/金样）────────────

/// Stack 臂（a.java:7528-7555）：15=tutorialSeen+栈顶追加/叠加。
/// addItemToItemStack 语义（a.java:7727-7740）：find_item 命中→uses 叠加；
/// 否则栈顶追加（n<cap 守卫=BUG-050 静默吞面）。
pub struct StackEffect;
impl ItemEffect for StackEffect {
    fn apply(&self, ctx: &mut ItemCtx<'_>, params: &EffectParams) -> ItemOutcome {
        let EffectParams::Stack { type_id, tutorial } = params else {
            return ItemOutcome::NoOp;
        };
        if *tutorial {
            *ctx.tutorial_seen = true;
        }
        let p = &mut *ctx.player;
        let t = *type_id as u8;
        let idx = if *type_id == 85 {
            20
        } else if *type_id == 86 {
            21
        } else {
            (*type_id - 13) as usize
        };
        const ITEM_USES: [i32; 22] = [-1, -1, -1, -1, 1, 1, 1, 3, 1, 1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let existing = p.find_item(t);
        if existing < 0 {
            let n = p.item_stack_size as usize;
            if n < p.item_types.len() {
                p.item_types[n] = t;
                p.item_uses[n] = ITEM_USES[idx] as u8;
                p.item_stack_size += 1;
            }
        } else {
            let u = p.item_uses[existing as usize];
            p.item_uses[existing as usize] = (u as i32 + ITEM_USES[idx]) as u8;
        }
        ItemOutcome::Consumed { overlay_tier: None }
    }
}

/// Key 臂（a.java:7556-7564）：键计数 ++。
pub struct KeyEffect;
impl ItemEffect for KeyEffect {
    fn apply(&self, ctx: &mut ItemCtx<'_>, params: &EffectParams) -> ItemOutcome {
        let EffectParams::Key { yellow, red, blue } = params else {
            return ItemOutcome::NoOp;
        };
        if *yellow {
            ctx.player.yellow_keys += 1;
        }
        if *red {
            ctx.player.red_keys += 1;
        }
        if *blue {
            ctx.player.blue_keys += 1;
        }
        ItemOutcome::Consumed { overlay_tier: None }
    }
}

/// StatUp 臂（a.java:7565-7580）：floor≤10 +1 否则 +floorTier。
pub struct StatUpEffect;
impl ItemEffect for StatUpEffect {
    fn apply(&self, ctx: &mut ItemCtx<'_>, params: &EffectParams) -> ItemOutcome {
        let EffectParams::StatUp { atk, def } = params else {
            return ItemOutcome::NoOp;
        };
        let (da, dd) = if ctx.floor <= 10 {
            (*atk, *def)
        } else {
            let f = ctx.floor_tier();
            (atk * f, def * f)
        };
        ctx.player.atk += da;
        ctx.player.def += dd;
        ItemOutcome::Consumed { overlay_tier: None }
    }
}

/// HpUp 臂（a.java:7581-7587）：applyHpDelta(mul×tier)。
/// apply_hp_delta 语义（clamp 面）按 player.hp 钳形逐字（编排面同则）。
pub struct HpUpEffect;
impl ItemEffect for HpUpEffect {
    fn apply(&self, ctx: &mut ItemCtx<'_>, params: &EffectParams) -> ItemOutcome {
        let EffectParams::HpUp { delta } = params else {
            return ItemOutcome::NoOp;
        };
        let d = delta * ctx.floor_tier();
        let max_hp = 9999; // apply_hp_delta 钳上界=编排面同值（hp clamp 逐字待 P1-3 对齐）
        let hp = (ctx.player.hp + d).clamp(0, max_hp);
        ctx.player.hp = hp;
        ItemOutcome::Consumed { overlay_tier: None }
    }
}

/// Equip 臂（a.java:7632-7648 tryEquip）：tier 档比较换装。
/// try_equip 语义（a.java:7688-7694 equipTierIndex 倒序查档）：目标 tier 档
/// 现档为空或更低→换装成功；否则失败（"你拥有更强力装备"）。
pub struct EquipEffect;
impl ItemEffect for EquipEffect {
    fn apply(&self, ctx: &mut ItemCtx<'_>, params: &EffectParams) -> ItemOutcome {
        let EffectParams::Equip { tier } = params else {
            return ItemOutcome::NoOp;
        };
        let want = *tier;
        let weapon_side = (1..=5).contains(tier);
        let cur_type = if weapon_side {
            *ctx.equipped_weapon
        } else {
            *ctx.equipped_armor
        };
        let cur_tier = if cur_type == 0 {
            0 // 空档=0 档（防 equip_types[6]=0 空臂误配——实测红修）
        } else {
            let mut t = 0i32;
            for i in (0..12).rev() {
                if ctx.equip_types[i] == cur_type {
                    t = i as i32;
                    break;
                }
            }
            t
        };
        // tryEquip 判定（a.java 语义）：目标档装备类型非 0 且当前档<目标档→换
        if ctx.equip_types[want as usize] != 0 && cur_tier < want {
            let new_type = ctx.equip_types[want as usize];
            if weapon_side {
                *ctx.equipped_weapon = new_type;
            } else {
                *ctx.equipped_armor = new_type;
            }
            ItemOutcome::Consumed { overlay_tier: Some(want) }
        } else {
            ItemOutcome::NotConsumed { fail_msg: true }
        }
    }
}

/// 未知/默认臂（a.java:7630-7631 `default: return`）：NoOp 不消耗。
pub struct NoOpEffect;
impl ItemEffect for NoOpEffect {
    fn apply(&self, _ctx: &mut ItemCtx<'_>, _params: &EffectParams) -> ItemOutcome {
        ItemOutcome::NoOp
    }
}

/// 内建注册（编译期注册——spec 裁定：禁运行期 so 插件；加载期数据解析=P1-1
/// items.json/defs.rs）。miss→NoOpEffect（运行期 no-op 不消耗两层语义）。
pub fn builtin_effect(kind: &str) -> &'static dyn ItemEffect {
    match kind {
        "Stack" => &StackEffect,
        "Key" => &KeyEffect,
        "StatUp" => &StatUpEffect,
        "HpUp" => &HpUpEffect,
        "Equip" => &EquipEffect,
        _ => &NoOpEffect,
    }
}

/// 【P1-2 等价测】逐臂语义钉（67 行 match 逐字抽象对拍——锚随断言）。
#[cfg(test)]
mod p1a2_tests {
    use super::*;

    fn fresh_player() -> PlayerCombat {
        PlayerCombat {
            hp: 0,
            atk: 0,
            def: 0,
            yellow_keys: 0,
            blue_keys: 0,
            red_keys: 0,
            gold: 0,
            item_types: vec![0; 32],
            item_uses: vec![0; 32],
            item_stack_size: 0,
        }
    }

    fn ctx<'a>(
        p: &'a mut PlayerCombat,
        e: &'a mut EntityTable,
        ts: &'a mut bool,
        ew: &'a mut i32,
        ea: &'a mut i32,
    ) -> ItemCtx<'a> {
        ItemCtx {
            player: p,
            entities: e,
            floor: 3,
            tutorial_seen: ts,
            equipped_weapon: ew,
            equipped_armor: ea,
            equip_types: [0, 33, 34, 35, 79, 36, 0, 37, 38, 39, 80, 40],
            equip_bonuses: [0, 10, 30, 70, 120, 220, 0, 10, 30, 70, 120, 220],
        }
    }

    /// Stack 臂：栈增+tutorial（a.java:7528-7555/7727-7740）。
    #[test]
    fn stack_arm_matches_match_semantics() {
        let mut p = fresh_player();
        let mut e = EntityTable::new(4);
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut e, &mut ts, &mut ew, &mut ea);
        let out = StackEffect.apply(&mut c, &EffectParams::Stack { type_id: 15, tutorial: true });
        assert_eq!(out, ItemOutcome::Consumed { overlay_tier: None });
        assert!(ts, "15=tutorialSeen（a.java:7538-7539）");
        assert_eq!(p.item_stack_size, 1, "addItemToItemStack 栈增");
    }

    /// Key 臂：键计数 ++（a.java:7556-7564）。
    #[test]
    fn key_arm_matches_match_semantics() {
        let mut p = fresh_player();
        let mut e = EntityTable::new(4);
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut e, &mut ts, &mut ew, &mut ea);
        KeyEffect.apply(&mut c, &EffectParams::Key { yellow: true, red: false, blue: true });
        assert_eq!((p.yellow_keys, p.red_keys, p.blue_keys), (1, 0, 1));
    }

    /// StatUp 臂：floor≤10 +1（a.java:7565-7580）。
    #[test]
    fn statup_arm_matches_match_semantics() {
        let mut p = fresh_player();
        let mut e = EntityTable::new(4);
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut e, &mut ts, &mut ew, &mut ea);
        StatUpEffect.apply(&mut c, &EffectParams::StatUp { atk: 1, def: 1 });
        assert_eq!((p.atk, p.def), (1, 1), "floor3≤10=+1（a.java:7565-7580）");
    }

    /// StatUp floor>10=+tier（floorTier a.java:7651-7662）。
    #[test]
    fn statup_floor_tier_arm() {
        let mut p = fresh_player();
        let mut e = EntityTable::new(4);
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut e, &mut ts, &mut ew, &mut ea);
        c.floor = 12; // tier=2
        StatUpEffect.apply(&mut c, &EffectParams::StatUp { atk: 1, def: 0 });
        assert_eq!(p.atk, 2, "floor12→tier2=+2（a.java:7565-7572/7651-7662）");
    }

    /// HpUp 臂：mul×tier（a.java:7581-7587）。
    #[test]
    fn hpup_arm_matches_match_semantics() {
        let mut p = fresh_player();
        let mut e = EntityTable::new(4);
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut e, &mut ts, &mut ew, &mut ea);
        HpUpEffect.apply(&mut c, &EffectParams::HpUp { delta: 200 });
        assert_eq!(p.hp, 200, "tier1=+200（a.java:7585-7587）");
    }

    /// Equip 臂：换装成功/失败（a.java:7632-7648 tryEquip）。
    #[test]
    fn equip_arm_matches_match_semantics() {
        let mut p = fresh_player();
        let mut e = EntityTable::new(4);
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut e, &mut ts, &mut ew, &mut ea);
        // 空档→tier1 换装成功
        let out = EquipEffect.apply(&mut c, &EffectParams::Equip { tier: 1 });
        assert_eq!(out, ItemOutcome::Consumed { overlay_tier: Some(1) });
        assert_eq!(*c.equipped_weapon, 33, "换装 type=33（equip_types[1]）");
        // 低档→失败（"你拥有更强力装备"=NotConsumed fail_msg）
        let out = EquipEffect.apply(&mut c, &EffectParams::Equip { tier: 1 });
        assert_eq!(out, ItemOutcome::NotConsumed { fail_msg: true });
    }

    /// 未知 effect=运行期 no-op 不消耗（a.java:7630-7631 default: return）。
    #[test]
    fn unknown_effect_is_noop() {
        let mut p = fresh_player();
        let mut e = EntityTable::new(4);
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut e, &mut ts, &mut ew, &mut ea);
        let out = builtin_effect("NoSuchKind").apply(&mut c, &EffectParams::HpUp { delta: 50 });
        assert_eq!(out, ItemOutcome::NoOp, "registry miss=NoOp 不消耗（a.java:7630-7631）");
    }
}
