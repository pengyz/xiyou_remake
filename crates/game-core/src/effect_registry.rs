//! DDDSA P1-2 ItemEffect trait+内建注册（spec v3 §2.2）——行为可插拔面。
//!
//! **逐臂对照表**（67 行 match→注册行映射；a.java 锚随行）：
//!
//! | 臂 | 语义（pickup_item_type 逐字抽象） | 注册行 | 锚 |
//! |---|---|---|---|
//! | 13..=25\|85\|86 | （15=tutorialSeen=true）+addItemToItemStack | `Stack` | a.java:7537-7555（2026-11-05 sed 复核：7528=goldAmount 尾 stale 更正） |
//! | 26/27/28 | yellow/red/blue KeyCount++ | `Key` | a.java:7556-7564 |
//! | 29/30 | floor≤10 +1 否则 +floorTier | `StatUp` | a.java:7565-7580 |
//! | 31/32 | applyHpDelta(50/200×tier) | `HpUp` | a.java:7581-7587 |
//! | 33..=36\|79 | 武器 tier {1,2,3,4,5} tryEquip | `Equip` | a.java:7632-7648 |
//! | 37..=40\|80 | 防具 tier {7,8,9,10,11} tryEquip | `Equip` | a.java:7632-7648 |
//! | default | return（41+ 不可达） | `NoOp` | a.java:7630-7631（截臂：switch 终=7631） |
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
//!
//! **【硬门1】type→kind 书面约定**（32 型全映射无 miss——P1-3 switch 切换面
//! 分发真值源）：
//!
//! | 型域 | kind | 型清单 |
//! |---|---|---|
//! | 栈增族 | Stack | 13,14,16,17,18,19,20,21,85,86 |
//! | 栈增·tutorial 变体 | TutorialSeenStack | 15 |
//! | 栈增·被动族 | Passive | 22,23,24,25 |
//! | 键 | Key | 26(黄),27(红),28(蓝) |
//! | 属性 | StatUp | 29(atk),30(def) |
//! | 血量 | HpUp | 31(×50),32(×200) |
//! | 装备·武器段 | Equip | 33,34,35,79,36（tier 1..5） |
//! | 装备·防具段 | Equip | 37,38,39,80,40（tier 7..11） |
//!
//! **【硬门4】entities 字段归域**：ItemDef.icon.entity_type=**entity 图面
//! 引用**（images.entity[t] 渲染面/sprite_defs 引用面）——归 entity/sprite
//! 域外键非 item 自有字段（items.json icon 即该外键句柄；P1-6 EnemyDef 聚合
//! 时同则）。
//! **【硬门4 残留处置·归域落地】**ItemCtx.entities 死字段=**已删**（复核
//! round-2 :123 残留清——零读取零消费，P1-3 接线时按需以 schema 流程重加，
//! 不预埋死槽）。

use crate::combat::PlayerCombat;

/// 强类型效果参数（行为面——与 game-data EffectKey 载荷镜像）。
#[derive(Debug, Clone, PartialEq)]
pub enum EffectParams {
    Stack { type_id: i32, tutorial: bool },
    Key { yellow: bool, red: bool, blue: bool },
    StatUp { atk: i32, def: i32 },
    HpUp { delta: i32 },
    Equip { tier: i32, force: bool },
}

/// 动作结果（纯数据 outcome——副作用请求上交编排面执行）。
#[derive(Debug, Clone, PartialEq)]
pub enum ItemOutcome {
    /// 消耗（true 臂：栈增/键增/属性增/换装成功/加血）。hp_popup=回血浮层
    /// 请求（applyHpDelta 的 else if delta>0 → popups.spawn(3,delta)——
    /// 2026-11-05 fx-pickup-31/32 红修实录：漏读正分支丢 +N 浮层=帧差）。
    Consumed { overlay_tier: Option<i32>, hp_popup: Option<i32> },
    /// 未消耗（装备失败臂：已有更强装备——浮层提示）。
    NotConsumed { fail_msg: bool },
    /// 未知 effect/默认臂（a.java:7630-7631 return）：不消耗无副作用。
    NoOp,
}

/// Ctx 三宪：纯数据引用槽+字段冻结（结构固定不增）。
pub struct ItemCtx<'a> {
    pub player: &'a mut PlayerCombat,
    pub floor: i32,
    pub tutorial_seen: &'a mut bool,
    pub equipped_weapon: &'a mut i32,
    pub equipped_armor: &'a mut i32,
    pub equip_types: [i32; 12],
    pub equip_bonuses: [i32; 12],
}

/// type→kind 分发真值（【硬门1 代码孪生】书面表（模块头）唯一程序面——
/// a.java:7537 pickup switch 七臂语义；测试以本函数钉 rows 配对（改错配对
/// 即红）。⚠本表只管 **pickup 面**；items.json effect.key=**use 面**键
/// （13/14/16..21/85/86 十型分歧两面并存——勿互对，P1-3 接线各取所需）。
pub fn pickup_kind_for_type(t: i32) -> &'static str {
    match t {
        13 | 14 | 16..=21 | 85 | 86 => "Stack",
        15 => "TutorialSeenStack",
        22..=25 => "Passive",
        26..=28 => "Key",
        29 | 30 => "StatUp",
        31 | 32 => "HpUp",
        33..=40 | 79 | 80 => "Equip",
        _ => "NoOp",
    }
}

/// pickup 臂参数表（【P1-3 分发面】23 臂→EffectParams 逐行——a.java:7537-
/// 7648 引锚；与 pickup_kind_for_type 同表两列：kind+params）。
pub fn pickup_params_for(t: i32) -> EffectParams {
    match t {
        15 => EffectParams::Stack { type_id: 15, tutorial: true },
        13 | 14 | 16..=25 | 85 | 86 => EffectParams::Stack { type_id: t, tutorial: false },
        26 => EffectParams::Key { yellow: true, red: false, blue: false },
        27 => EffectParams::Key { yellow: false, red: true, blue: false },
        28 => EffectParams::Key { yellow: false, red: false, blue: true },
        29 => EffectParams::StatUp { atk: 1, def: 0 },
        30 => EffectParams::StatUp { atk: 0, def: 1 },
        31 => EffectParams::HpUp { delta: 50 },
        32 => EffectParams::HpUp { delta: 200 },
        33 => EffectParams::Equip { tier: 1, force: false },
        34 => EffectParams::Equip { tier: 2, force: false },
        35 => EffectParams::Equip { tier: 3, force: false },
        79 => EffectParams::Equip { tier: 4, force: false },
        36 => EffectParams::Equip { tier: 5, force: false },
        37 => EffectParams::Equip { tier: 7, force: false },
        38 => EffectParams::Equip { tier: 8, force: false },
        39 => EffectParams::Equip { tier: 9, force: false },
        80 => EffectParams::Equip { tier: 10, force: false },
        40 => EffectParams::Equip { tier: 11, force: false },
        _ => EffectParams::Stack { type_id: t, tutorial: false }, // default 臂占位（kind=NoOp 吞）
    }
}

/// floorTier 独立域函数（a.java:7651-7662 逐字：(f-1)/10 钳 0；f>50→0；+1）
/// ——【黄③】ctx 纯数据零方法（三宪：字段冻结/纯数据）。
pub fn floor_tier(floor: i32) -> i32 {
    let mut t = (floor - 1) / 10;
    if t < 0 {
        t = 0;
    }
    if floor > 50 {
        t = 0;
    }
    t + 1
}

/// ItemEffect trait（行为可插拔面——apply 签名=ctx+≤2 按三宪）。
pub trait ItemEffect {
    fn apply(&self, ctx: &mut ItemCtx<'_>, params: &EffectParams) -> ItemOutcome;
    /// 判别指纹（【红-1 修复】kind 稳定标签值比较——`std::ptr::eq` 跨
    /// codegen unit 的 &dyn 强转恒 false（两 vtable 实例），曾致 no-miss
    /// 断言永绿伪钉；本方法回静态串值比较，删/误路由注册臂必红）。
    fn kind_name(&self) -> &'static str;
}

// ── 内建效果（67 行 match 逐字抽象——逐臂等价三层：锚/夹具/金样）────────────

/// Stack 臂（a.java:7528-7555）：15=tutorialSeen+栈顶追加/叠加。
/// addItemToItemStack 语义（a.java:7727-7740）：find_item 命中→uses 叠加；
/// 否则栈顶追加（n<cap 守卫=BUG-050 静默吞面）。
pub struct StackEffect;
impl ItemEffect for StackEffect {
    fn kind_name(&self) -> &'static str { "Stack" }
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
        ItemOutcome::Consumed { overlay_tier: None, hp_popup: None }
    }
}

/// Key 臂（a.java:7556-7564）：键计数 ++。
pub struct KeyEffect;
impl ItemEffect for KeyEffect {
    fn kind_name(&self) -> &'static str { "Key" }
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
        ItemOutcome::Consumed { overlay_tier: None, hp_popup: None }
    }
}

/// StatUp 臂（a.java:7565-7580）：floor≤10 +1 否则 +floorTier。
pub struct StatUpEffect;
impl ItemEffect for StatUpEffect {
    fn kind_name(&self) -> &'static str { "StatUp" }
    fn apply(&self, ctx: &mut ItemCtx<'_>, params: &EffectParams) -> ItemOutcome {
        let EffectParams::StatUp { atk, def } = params else {
            return ItemOutcome::NoOp;
        };
        let (da, dd) = if ctx.floor <= 10 {
            (*atk, *def)
        } else {
            let f = floor_tier(ctx.floor);
            (atk * f, def * f)
        };
        ctx.player.atk += da;
        ctx.player.def += dd;
        ItemOutcome::Consumed { overlay_tier: None, hp_popup: None }
    }
}

/// HpUp 臂（a.java:7581-7587）：applyHpDelta(mul×tier)。
/// 【黄②修复=移植错】apply_hp_delta 真语义（scene.rs:4393 逐字）：hp≤-delta
/// →hp=1（伤害地板非 9999 钳——9999 前值=C 级臆造已证伪）；否则 hp+=delta。
pub struct HpUpEffect;
impl ItemEffect for HpUpEffect {
    fn kind_name(&self) -> &'static str { "HpUp" }
    fn apply(&self, ctx: &mut ItemCtx<'_>, params: &EffectParams) -> ItemOutcome {
        let EffectParams::HpUp { delta } = params else {
            return ItemOutcome::NoOp;
        };
        let mut d = delta * floor_tier(ctx.floor);
        // 归零判据逐字（父 apply_hp_delta：hp<=-delta→hp=1 且 delta=hp-1=0
        // ——**归零后不进浮层分支=不发**；2026-11-06 黄①修复：前版丢归零改写
        // 致多发浮层（战斗 strike 无 hp 地板可致负 hp 可达——scene.rs:2076/
        // 3788 面）。
        if ctx.player.hp <= -d {
            ctx.player.hp = 1;
            d = ctx.player.hp - 1;
        } else {
            ctx.player.hp += d;
        }
        let popup = if d > 0 { Some(d) } else { None };
        ItemOutcome::Consumed { overlay_tier: None, hp_popup: popup }
    }
}

/// Equip 臂（a.java:7632-7648 tryEquip→scene.rs:865 逐字）：**分段 m_083
/// 倒查+加成双动**（红-1 修复：前版丢 equip_bonuses 应用——atk/def += 新档
/// −= 旧档）。tier<6=武器段 [0,6)；≥6=防具段 [6,12)（m_083_index 先减后比
/// 倒序——a.java:7699-7709）。
pub struct EquipEffect;
impl ItemEffect for EquipEffect {
    fn kind_name(&self) -> &'static str { "Equip" }
    fn apply(&self, ctx: &mut ItemCtx<'_>, params: &EffectParams) -> ItemOutcome {
        let EffectParams::Equip { tier, force } = params else {
            return ItemOutcome::NoOp;
        };
        let tier = *tier;
        if tier < 6 {
            let cur = m_083_index(*ctx.equipped_weapon, 0, 6, &ctx.equip_types);
            if cur >= tier && !force {
                return ItemOutcome::NotConsumed { fail_msg: true };
            }
            ctx.player.atk -= ctx.equip_bonuses[cur as usize];
            ctx.player.atk += ctx.equip_bonuses[tier as usize];
            *ctx.equipped_weapon = ctx.equip_types[tier as usize];
        } else {
            let cur = m_083_index(*ctx.equipped_armor, 6, 12, &ctx.equip_types);
            if cur >= tier && !force {
                return ItemOutcome::NotConsumed { fail_msg: true };
            }
            ctx.player.def -= ctx.equip_bonuses[cur as usize];
            ctx.player.def += ctx.equip_bonuses[tier as usize];
            *ctx.equipped_armor = ctx.equip_types[tier as usize];
        }
        ItemOutcome::Consumed { overlay_tier: Some(tier), hp_popup: None }
    }
}

/// m_083_index（a.java:7699-7709 逐字：先减后比倒序——[lo,hi-1]，hi 永不触达）。
fn m_083_index(equipped: i32, lo: i32, hi: i32, types: &[i32; 12]) -> i32 {
    let mut t = hi;
    while t > lo {
        t -= 1;
        if types[t as usize] == equipped {
            return t;
        }
    }
    0
}

/// UseArm（ItemAction use 七 kind——**【硬门3 仅键可达声明】**：本行=分发键
/// 可达性钉（kind 覆盖面）；**本层零行为**（apply 返回 NoOp=不消耗不副作用）
/// ——行为面=item_effect.rs execute_item_action 编排（P1-3 分发未接线；
/// 前值 Consumed=潜伏面已消：若误接线会假报消耗）。
pub struct UseArmEffect;
impl ItemEffect for UseArmEffect {
    fn kind_name(&self) -> &'static str { "UseArm" }
    fn apply(&self, _ctx: &mut ItemCtx<'_>, _params: &EffectParams) -> ItemOutcome {
        ItemOutcome::NoOp
    }
}

/// 未知/默认臂（a.java:7630-7631 `default: return`）：NoOp 不消耗。
pub struct NoOpEffect;
impl ItemEffect for NoOpEffect {
    fn kind_name(&self) -> &'static str { "NoOp" }
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
        // 【硬门2 归一】TutorialSeenStack/Passive=栈增族（pickup 臂 13..=25 同
        // ——15=tutorial 变体；22..25=被动族（道具栏被动=find_item 活查，
        // 拾取即栈增）；params Stack{tutorial} 编码变体。
        "TutorialSeenStack" | "Passive" => &StackEffect,
        // ItemAction use 七 kind（全臂对照表覆盖——P1-3 分发键面）
        "OpenPanel" | "RemoveAdjacent" | "RemoveAllType" | "Heal"
        | "KillAdjacent4" | "MirrorTeleport" | "ChangeFloorUp" => &UseArmEffect,
        _ => &NoOpEffect,
    }
}

/// 【P1-3 红·先行】pickup_params_for 23 臂对照（match→表行参数逐行——
/// a.java:7537-7648 引锚纪律随断言）。
#[cfg(test)]
mod p1a3_tests {
    use super::*;

    /// 逐臂参数对照：23 臂→EffectParams（TDD 红：函数未实现=编译红）。
    #[test]
    fn pickup_params_for_all_arms() {
        // 栈增族（a.java:7538-7555）
        for t in [13, 14, 16, 17, 18, 19, 20, 21, 85, 86] {
            assert_eq!(
                pickup_params_for(t),
                EffectParams::Stack { type_id: t, tutorial: false },
                "型 {t} 栈增参数"
            );
        }
        assert_eq!(
            pickup_params_for(15),
            EffectParams::Stack { type_id: 15, tutorial: true },
            "15=tutorial 变体（a.java:7538-7539）"
        );
        for t in [22, 23, 24, 25] {
            assert_eq!(
                pickup_params_for(t),
                EffectParams::Stack { type_id: t, tutorial: false },
                "型 {t} 被动族栈增（a.java:7548-7554 落穿——2026-11-06 sed 核：7553=case 86: 标签）"
            );
        }
        // 键（a.java:7556-7564）
        assert_eq!(pickup_params_for(26), EffectParams::Key { yellow: true, red: false, blue: false });
        assert_eq!(pickup_params_for(27), EffectParams::Key { yellow: false, red: true, blue: false });
        assert_eq!(pickup_params_for(28), EffectParams::Key { yellow: false, red: false, blue: true });
        // 属性（a.java:7565-7580）
        assert_eq!(pickup_params_for(29), EffectParams::StatUp { atk: 1, def: 0 });
        assert_eq!(pickup_params_for(30), EffectParams::StatUp { atk: 0, def: 1 });
        // 血量（a.java:7581-7587）
        assert_eq!(pickup_params_for(31), EffectParams::HpUp { delta: 50 });
        assert_eq!(pickup_params_for(32), EffectParams::HpUp { delta: 200 });
        // 装备 tier 面（a.java:7632-7648+7651-7662 tier 映射）
        for (t, tier) in [(33, 1), (34, 2), (35, 3), (79, 4), (36, 5), (37, 7), (38, 8), (39, 9), (80, 10), (40, 11)] {
            assert_eq!(
                pickup_params_for(t),
                EffectParams::Equip { tier, force: false },
                "型 {t}→tier {tier}（force=false=拾取面）"
            );
        }
        // default 臂（a.java:7630-7631 return）：NoOp 参数
        assert_eq!(pickup_params_for(41), EffectParams::Stack { type_id: 41, tutorial: false }, "41+=default 臂参数占位");
    }

    /// 【黄②·t=79 钉】79→tier 4（Java equipTierIndex(79)=4）——父 eq_tier
    /// if t<37 else 分支对 79 落 _=>11 错帧（overlay_kind_pickup(79,11)≠Java
    /// 4）；表行=Java 真值 4（移植错修正——fb1ddc6 静默修正今披露）。
    #[test]
    fn equip_79_tier_is_4() {
        assert_eq!(
            pickup_params_for(79),
            EffectParams::Equip { tier: 4, force: false },
            "79→tier 4（Java equipTierIndex(79)=4；父 eq_tier _=>11 错帧已修）"
        );
        let mut p = PlayerCombat { hp: 0, atk: 0, def: 0, yellow_keys: 0, blue_keys: 0, red_keys: 0, gold: 0, item_types: vec![0; 32], item_uses: vec![0; 32], item_stack_size: 0 };
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ItemCtx { player: &mut p, floor: 3, tutorial_seen: &mut ts, equipped_weapon: &mut ew, equipped_armor: &mut ea, equip_types: [0, 33, 34, 35, 79, 36, 0, 37, 38, 39, 80, 40], equip_bonuses: [0, 10, 30, 70, 120, 220, 0, 10, 30, 70, 120, 220] };
        let out = EquipEffect.apply(&mut c, &EffectParams::Equip { tier: 4, force: false });
        assert_eq!(out, ItemOutcome::Consumed { overlay_tier: Some(4), hp_popup: None }, "浮层 tier=4（非 11）");
    }

    /// 【黄①·归零不发钉】hp<=-d 边界：hp=1+**不发浮层**（revert 多发必红）。
    #[test]
    fn hpup_floor_zero_no_popup() {
        let mut p = PlayerCombat { hp: -100, atk: 0, def: 0, yellow_keys: 0, blue_keys: 0, red_keys: 0, gold: 0, item_types: vec![0; 32], item_uses: vec![0; 32], item_stack_size: 0 };
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ItemCtx { player: &mut p, floor: 3, tutorial_seen: &mut ts, equipped_weapon: &mut ew, equipped_armor: &mut ea, equip_types: [0, 33, 34, 35, 79, 36, 0, 37, 38, 39, 80, 40], equip_bonuses: [0, 10, 30, 70, 120, 220, 0, 10, 30, 70, 120, 220] };
        // hp=-100<=-50→归零路径：hp=1 且 hp_popup=None（父 delta=hp-1=0 不发）
        let out = HpUpEffect.apply(&mut c, &EffectParams::HpUp { delta: 50 });
        assert_eq!(out, ItemOutcome::Consumed { overlay_tier: None, hp_popup: None }, "归零不发浮层（revert 多发必红）");
        assert_eq!(c.player.hp, 1);
    }

    /// Equip force 语义（a.java try_equip 完整体：force=true 跳 cur 检查强换）
    /// ——script_ops:281 force 路径单源面。
    #[test]
    fn equip_force_semantics() {
        let mut p = PlayerCombat { hp: 0, atk: 0, def: 0, yellow_keys: 0, blue_keys: 0, red_keys: 0, gold: 0, item_types: vec![0; 32], item_uses: vec![0; 32], item_stack_size: 0 };
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ItemCtx { player: &mut p, floor: 3, tutorial_seen: &mut ts, equipped_weapon: &mut ew, equipped_armor: &mut ea, equip_types: [0, 33, 34, 35, 79, 36, 0, 37, 38, 39, 80, 40], equip_bonuses: [0, 10, 30, 70, 120, 220, 0, 10, 30, 70, 120, 220] };
        // tier 5 换装成功
        EquipEffect.apply(&mut c, &EffectParams::Equip { tier: 5, force: false });
        assert_eq!(*c.equipped_weapon, 36);
        // tier 1 低档 force=false→NotConsumed（cur>=tier）
        let out = EquipEffect.apply(&mut c, &EffectParams::Equip { tier: 1, force: false });
        assert_eq!(out, ItemOutcome::NotConsumed { fail_msg: true });
        // tier 1 force=true→**强制换装**（跳检查；加成双动=−bonuses[5](220)+bonuses[1](10)）
        let atk0 = c.player.atk;
        let out = EquipEffect.apply(&mut c, &EffectParams::Equip { tier: 1, force: true });
        assert_eq!(out, ItemOutcome::Consumed { overlay_tier: Some(1), hp_popup: None });
        assert_eq!(*c.equipped_weapon, 33, "force 强换 type=33");
        assert_eq!(c.player.atk, atk0 - 220 + 10, "force 路径加成双动（−旧档+新档）");
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
        ts: &'a mut bool,
        ew: &'a mut i32,
        ea: &'a mut i32,
    ) -> ItemCtx<'a> {
        ItemCtx {
            player: p,
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
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut ts, &mut ew, &mut ea);
        let out = StackEffect.apply(&mut c, &EffectParams::Stack { type_id: 15, tutorial: true });
        assert_eq!(out, ItemOutcome::Consumed { overlay_tier: None, hp_popup: None });
        assert!(ts, "15=tutorialSeen（a.java:7538-7539）");
        assert_eq!(p.item_stack_size, 1, "addItemToItemStack 栈增");
    }

    /// Key 臂：键计数 ++（a.java:7556-7564）。
    #[test]
    fn key_arm_matches_match_semantics() {
        let mut p = fresh_player();
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut ts, &mut ew, &mut ea);
        KeyEffect.apply(&mut c, &EffectParams::Key { yellow: true, red: false, blue: true });
        assert_eq!((p.yellow_keys, p.red_keys, p.blue_keys), (1, 0, 1));
    }

    /// StatUp 臂：floor≤10 +1（a.java:7565-7580）。
    #[test]
    fn statup_arm_matches_match_semantics() {
        let mut p = fresh_player();
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut ts, &mut ew, &mut ea);
        StatUpEffect.apply(&mut c, &EffectParams::StatUp { atk: 1, def: 1 });
        assert_eq!((p.atk, p.def), (1, 1), "floor3≤10=+1（a.java:7565-7580）");
    }

    /// StatUp floor>10=+tier（floorTier a.java:7651-7662）。
    #[test]
    fn statup_floor_tier_arm() {
        let mut p = fresh_player();
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut ts, &mut ew, &mut ea);
        c.floor = 12; // tier=2
        StatUpEffect.apply(&mut c, &EffectParams::StatUp { atk: 1, def: 0 });
        assert_eq!(p.atk, 2, "floor12→tier2=+2（a.java:7565-7572/7651-7662）");
    }

    /// HpUp 臂：mul×tier（a.java:7581-7587）。
    #[test]
    fn hpup_arm_matches_match_semantics() {
        let mut p = fresh_player();
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut ts, &mut ew, &mut ea);
        HpUpEffect.apply(&mut c, &EffectParams::HpUp { delta: 200 });
        assert_eq!(p.hp, 200, "tier1=+200（a.java:7585-7587）");
    }

    /// Equip 臂：换装成功/失败（a.java:7632-7648 tryEquip）。
    #[test]
    fn equip_arm_matches_match_semantics() {
        let mut p = fresh_player();
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut ts, &mut ew, &mut ea);
        let atk0 = c.player.atk;
        // 空档→tier1 换装成功+**加成双动**（红-1：atk += bonuses[1] −= bonuses[0]）
        let out = EquipEffect.apply(&mut c, &EffectParams::Equip { tier: 1, force: false });
        assert_eq!(out, ItemOutcome::Consumed { overlay_tier: Some(1), hp_popup: None });
        assert_eq!(*c.equipped_weapon, 33, "换装 type=33（equip_types[1]）");
        assert_eq!(c.player.atk, atk0 + 10, "加成双动=+bonuses[1](10)−bonuses[0](0)（try_equip 逐字）");
        // 同档→失败（cur>=tier："你拥有更强力装备"=NotConsumed fail_msg）
        let out = EquipEffect.apply(&mut c, &EffectParams::Equip { tier: 1, force: false });
        assert_eq!(out, ItemOutcome::NotConsumed { fail_msg: true });
        // 低档→失败（tier1 现档=1：cur>=tier 拒）
        let out = EquipEffect.apply(&mut c, &EffectParams::Equip { tier: 1, force: false });
        assert_eq!(out, ItemOutcome::NotConsumed { fail_msg: true });
    }

    /// 【硬门1】type→kind 全映射无 miss：32 型逐型 kind 可达（书面约定执行钉）。
    #[test]
    fn type_to_kind_mapping_no_miss() {
        let rows: &[(i32, &str)] = &[
            (13, "Stack"), (14, "Stack"), (15, "TutorialSeenStack"),
            (16, "Stack"), (17, "Stack"), (18, "Stack"), (19, "Stack"),
            (20, "Stack"), (21, "Stack"), (22, "Passive"), (23, "Passive"),
            (24, "Passive"), (25, "Passive"), (26, "Key"), (27, "Key"),
            (28, "Key"), (29, "StatUp"), (30, "StatUp"), (31, "HpUp"),
            (32, "HpUp"), (33, "Equip"), (34, "Equip"), (35, "Equip"),
            (36, "Equip"), (37, "Equip"), (38, "Equip"), (39, "Equip"),
            (40, "Equip"), (79, "Equip"), (80, "Equip"), (85, "Stack"),
            (86, "Stack"),
        ];
        assert_eq!(rows.len(), 32, "32 型全量");
        // 【外锚·同源断言】rows 型集 == items.json id 集（P1-1 生成器产物独立面）
        let items_src = include_str!("../../../data/tables/items.json");
        let mut ids: Vec<i32> = items_src
            .split("\"id\": ")
            .skip(1)
            .map(|chunk| {
                chunk
                    .split([',', '\n'])
                    .next()
                    .unwrap()
                    .trim()
                    .parse::<i32>()
                    .unwrap()
            })
            .collect();
        ids.sort_unstable();
        let mut row_ids: Vec<i32> = rows.iter().map(|(t, _)| *t).collect();
        row_ids.sort_unstable();
        assert_eq!(row_ids, ids, "rows 型集与 items.json id 集同源一致（外锚防型集漂移）");
        for &(t, kind) in rows {
            // 配对钉（M4 判别：改错 (t,kind) 配对即红——pickup_kind_for_type
            // =书面表代码孪生，独立于 rows）
            assert_eq!(
                pickup_kind_for_type(t),
                kind,
                "型 {t} kind 配对漂移（rows vs pickup_kind_for_type）"
            );
            // 可达钉（M5 判误路由/删臂即红——kind_name 值比较跨 CGU 安全；
            // ptr::eq 跨 codegen unit 恒 false 伪钉教训）。
            let expected_label = match kind {
                "TutorialSeenStack" | "Passive" => "Stack",
                k => k,
            };
            assert_eq!(
                builtin_effect(kind).kind_name(),
                expected_label,
                "型 {t} kind {kind} miss/误路由"
            );
        }
    }

    /// 【黄④】kind 全覆盖：对照表全行 kind 在注册表可达（判别=kind_name）。
    #[test]
    fn kind_coverage_full_table() {
        for (kind, label) in [
            ("Stack", "Stack"), ("Key", "Key"), ("StatUp", "StatUp"),
            ("HpUp", "HpUp"), ("Equip", "Equip"),
            ("TutorialSeenStack", "Stack"), ("Passive", "Stack"),
            ("OpenPanel", "UseArm"), ("RemoveAdjacent", "UseArm"),
            ("RemoveAllType", "UseArm"), ("Heal", "UseArm"),
            ("KillAdjacent4", "UseArm"), ("MirrorTeleport", "UseArm"),
            ("ChangeFloorUp", "UseArm"),
        ] {
            assert_eq!(builtin_effect(kind).kind_name(), label, "{kind} kind 缺口/误路由");
        }
    }

    /// 【黄②·NoOp 判别钉】UseArmEffect 仅键可达=本层零行为（M2 判别：改回
    /// Consumed 即红——防'误接线假报消耗'回归）。
    #[test]
    fn use_arm_noop_discriminated() {
        let mut p = fresh_player();
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut ts, &mut ew, &mut ea);
        let out = UseArmEffect.apply(&mut c, &EffectParams::StatUp { atk: 1, def: 1 });
        assert_eq!(out, ItemOutcome::NoOp, "UseArmEffect 本层零行为（revert Consumed 必红）");
        assert_eq!(out, builtin_effect("OpenPanel").apply(&mut c, &EffectParams::StatUp { atk: 1, def: 1 }), "UseArm 路由一致性");
    }


    /// 【黄②】HpUp 判别用例：大增益超 9999 **不钳**（revert 9999 臆造语义本用
    /// 例必红——真语义=纯加+伤害地板，a.java/scene.rs:4393）。
    #[test]
    fn hpup_no_9999_clamp_discriminator() {
        let mut p = fresh_player();
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut ts, &mut ew, &mut ea);
        c.floor = 3; // tier=1
        HpUpEffect.apply(&mut c, &EffectParams::HpUp { delta: 20000 });
        assert_eq!(c.player.hp, 20000, "无 9999 钳（判别：臆造语义下=9999 应红）");
    }

    /// 【黄③】def 臂 tier≥6 断言：防具段加成双动（bonuses[7]−bonuses[6]）。
    #[test]
    fn equip_armor_tier_ge6_def_bonus() {
        let mut p = fresh_player();
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut ts, &mut ew, &mut ea);
        // 第一步：tier 7 换装（空档 cur=m_083→0，+bonuses[7]=10）
        let out = EquipEffect.apply(&mut c, &EffectParams::Equip { tier: 7, force: false });
        assert_eq!(out, ItemOutcome::Consumed { overlay_tier: Some(7), hp_popup: None });
        assert_eq!(*c.equipped_armor, 37, "防具段换装 type=37（equip_types[7]）");
        assert_eq!(c.player.def, 10, "加半=+bonuses[7](10)−bonuses[6](0)（M6 判加）");
        // 第二步：tier 11 低段→高段换装（cur=7 档 bonuses[7]=10≠0——
        // 【黄③ 减半真钉】def=10−bonuses[7](10)+bonuses[11](220)=220（M3 判减）
        let out = EquipEffect.apply(&mut c, &EffectParams::Equip { tier: 11, force: false });
        assert_eq!(out, ItemOutcome::Consumed { overlay_tier: Some(11), hp_popup: None });
        assert_eq!(*c.equipped_armor, 40, "高段换装 type=40（equip_types[11]）");
        assert_eq!(c.player.def, 220, "减半+加半双动=−bonuses[7](10)+bonuses[11](220)（M3 删减必红）");
    }

    /// 未知 effect=运行期 no-op 不消耗（a.java:7630-7631 default: return）。
    #[test]
    fn unknown_effect_is_noop() {
        let mut p = fresh_player();
        let mut ts = false;
        let (mut ew, mut ea) = (0, 0);
        let mut c = ctx(&mut p, &mut ts, &mut ew, &mut ea);
        let out = builtin_effect("NoSuchKind").apply(&mut c, &EffectParams::HpUp { delta: 50 });
        assert_eq!(out, ItemOutcome::NoOp, "registry miss=NoOp 不消耗（a.java:7630-7631）");
    }
}
