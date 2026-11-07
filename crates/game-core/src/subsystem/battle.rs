//! battle face（P2 批2 终形）——BattleSubsystem：resolve_bump 战斗解析面（spec §3.1）。
//!
//! 形照 spec §3.1 +【2026-11-07 更正·批2】（spec 注记③）：实现为
//! `fn resolve_bump(&mut self, ctx: &mut BattleCtx, t: EnemyTypeId, ev: BattleEvent) -> BattleOutcome`
//! （+ev 参——plan 批2「scene 战斗路径编排面→BattleOutcome」三事件面[途中开战/撞击开战/一击交换]
//! 需事件分辨；ctx+≤2 恰满 t+ev）。Ctx=BattleCtx（combat 域写面 10 槽）。
//! 写面=battle_enemy_hp（三事件）+battle_target/active/no_counter（Engage 装配）——经 ctx 槽
//! 唯一写（battle_enemy_hp 写面归因核=批2 交付）；预测/交换数学=crate::combat 纯函数层
//! （P1-6 trait_flags_for 单源不动，消费迁入本面）。

use crate::combat::{EnemyDef, PlayerCombat};

/// 事件面（spec 单 (ctx,t) 形无法表达三事件——plan 批2 授权扩展；F2 更正注记③）。
pub enum BattleEvent {
    /// 途中遭遇开战（a.java:8771-8787）：hp 缩放源（enemyHpScaled a.java:8775）+敌先手一击
    /// （a.java:8783-8784 strike 行）。【2026-11-08 批10 勘误】原 8760-8780/8769 等距推错且漏先手行。
    RouteEngage {
        /// 实体槽位（battle_target 写值——resolve_bump 无 slot 知识，经事件带入）。
        entity_slot: i32,
    },
    /// 撞击开战（a.java:5470-5490）：hp 基础源（enemyBaseHp）+无先手。
    BumpEngage {
        entity_slot: i32,
    },
    /// 一击交换（tickBattle a.java:10023-10149 击打拍数学）。
    Exchange,
}

/// 战斗解析结果（批2 终形随实现迁移定）。
pub enum BattleOutcome {
    /// predict 拒绝（loss<0 或 loss>=当前 hp）——零装配；Coordinator 处置
    /// （bump=阻挡+拒战提示 BUG-046 / route=走开改向）。
    Refused,
    /// 开战（battle_target/active/no_counter/enemy_hp 已经 ctx 装配）。
    /// first_strike=途中面敌先手 raw（撞击面 0；>0 门控=Coordinator 原行）。
    Engage {
        first_strike: i32,
    },
    /// 一击交换（blow=true 时 battle_enemy_hp 已经 ctx 扣减）。
    /// penetrates=false→Coordinator walk_phase=0（未击穿）；blow=false→空拍。
    Exchange {
        penetrates: bool,
        blow: bool,
        enemy_dmg: i32,
        player_dmg: i32,
        kill: bool,
        predicted_loss: i32,
        gold: i32,
    },
}

/// 敌型 id（i32 newtype——a.java 型号 t 语义；newtype 防裸混用）。
pub struct EnemyTypeId(pub i32);

/// BattleCtx——BattleSubsystem 专属 Ctx（ItemCtx 先例形=借用槽+值拷贝纯数据，无方法）。
///
/// 写槽=field-ownership combat 域 10 字段（写面归因登记 face=BattleCtx）；
/// 读槽=player/enemies/enemy_base（只读借——读面不计写集）。
pub struct BattleCtx<'a> {
    // —— 写槽（combat 域 10）——
    pub battle_active: &'a mut bool,
    pub battle_anim_tick: &'a mut i32,
    pub battle_digit_by_type: &'a mut Vec<i32>,
    pub battle_enemy_hp: &'a mut i32,
    pub battle_finishing: &'a mut bool,
    pub battle_no_counter: &'a mut bool,
    pub battle_rerun_scene: &'a mut bool,
    pub battle_target: &'a mut i32,
    pub battle_tick: &'a mut i32,
    pub show_battle_digits: &'a mut bool,
    // —— 读槽（只读）——
    pub player: &'a PlayerCombat,
    pub enemies: &'a [EnemyDef],
    pub enemy_base: &'a [EnemyDef],
}

/// spec §3.1：战斗子系统。
pub trait BattleSubsystem {
    fn resolve_bump(&mut self, ctx: &mut BattleCtx, t: EnemyTypeId, ev: BattleEvent) -> BattleOutcome;
}

/// 开战预测纯查询（predictHpLossVsType a.java:9992-9996 桥真语义——预测读**缩放表**，
    /// 【2026-11-08 批10 勘误】原 9983-9982 行锚倒置，sed 实核修。
/// 与 hp 装配分源无关）；query 面消费=digit 显示/撞格预判/开战门。math 单源=crate::combat。
pub fn predict_engagement(
    player: &PlayerCombat,
    enemies: &[EnemyDef],
    enemy_base: &[EnemyDef],
    t: i32,
    no_counter: bool,
) -> i32 {
    let idx = (t - 41) as usize;
    let trait_flags = enemy_base.get(idx).map(|r| r.trait_flags).unwrap_or(0);
    let atk = crate::combat::effective_attack_vs_type(player, trait_flags);
    crate::combat::predict_battle_hp_loss(
        atk,
        player.def,
        enemies[idx].hp,
        enemies[idx].atk,
        enemies[idx].def,
        no_counter,
    )
}

/// 战斗解析器（无状态 ZST——状态全经 BattleCtx 借用槽；调用点内联构造
/// `let mut r = BattleResolver;`，禁 GameScene 加字段[382 测试构造器面]）。
pub struct BattleResolver;

impl BattleSubsystem for BattleResolver {
    fn resolve_bump(&mut self, ctx: &mut BattleCtx, t: EnemyTypeId, ev: BattleEvent) -> BattleOutcome {
        let idx = (t.0 - 41) as usize;
        let trait_flags = ctx.enemy_base.get(idx).map(|r| r.trait_flags).unwrap_or(0);
        let atk = crate::combat::effective_attack_vs_type(ctx.player, trait_flags);
        // 开战面（途中/撞击）
        if !matches!(ev, BattleEvent::Exchange) {
            // armor_gate=撞击面无先手位（bump:true/route:false——a.java 两臂捆绑）
            let no_first_strike = matches!(ev, BattleEvent::BumpEngage { .. });
            // 预测=缩放表（predictHpLossVsType 桥真语义 a.java:9992-9996——
            // bump 面预测亦读缩放表，仅 hp 装配分源：保真不改）。
            let loss = predict_engagement(ctx.player, ctx.enemies, ctx.enemy_base, t.0, no_first_strike);
            if !(loss >= 0 && loss < ctx.player.hp) {
                return BattleOutcome::Refused;
            }
            return if no_first_strike {
                // 撞击面（a.java:5470-5490）：基础表源+无先手
                *ctx.battle_enemy_hp = ctx.enemy_base[idx].hp;
                *ctx.battle_active = true;
                *ctx.battle_no_counter = true;
                *ctx.battle_target = match ev {
                    BattleEvent::BumpEngage { entity_slot } => entity_slot,
                    _ => unreachable!(),
                };
                BattleOutcome::Engage { first_strike: 0 }
            } else {
                // 途中面（a.java:8771-8787）：缩放表源（enemyHpScaled a.java:8775）+敌先手 raw
                // （a.java:8783-8784）
                *ctx.battle_enemy_hp = ctx.enemies[idx].hp;
                *ctx.battle_active = false;
                *ctx.battle_no_counter = false;
                *ctx.battle_target = match ev {
                    BattleEvent::RouteEngage { entity_slot } => entity_slot,
                    _ => unreachable!(),
                };
                BattleOutcome::Engage {
                    first_strike: ctx.enemies[idx].atk - ctx.player.def,
                }
            };
        }
        // 一击交换（tickBattle a.java:10023-10149 击打拍数学）
        let dmg = atk - ctx.enemies[idx].def;
        if dmg <= 0 {
            return BattleOutcome::Exchange {
                penetrates: false,
                blow: false,
                enemy_dmg: dmg,
                player_dmg: 0,
                kill: false,
                predicted_loss: 0,
                gold: 0,
            };
        }
        if (*ctx.battle_tick & 3) != 0 {
            return BattleOutcome::Exchange {
                penetrates: true,
                blow: false,
                enemy_dmg: dmg,
                player_dmg: 0,
                kill: false,
                predicted_loss: 0,
                gold: 0,
            };
        }
        *ctx.battle_enemy_hp -= dmg;
        if *ctx.battle_enemy_hp > 0 {
            let counter = ctx.enemies[idx].atk - ctx.player.def;
            return BattleOutcome::Exchange {
                penetrates: true,
                blow: true,
                enemy_dmg: dmg,
                player_dmg: counter,
                kill: false,
                predicted_loss: 0,
                gold: 0,
            };
        }
        // 击杀（a.java:10036-10062）：终伤预测（armor_gate=init 位回声）+
        // 赏金（base 源+道具 25 二倍）。
        let loss = predict_engagement(ctx.player, ctx.enemies, ctx.enemy_base, t.0, *ctx.battle_no_counter);
        let gold = if ctx.player.find_item(25) >= 0 {
            ctx.enemy_base[idx].gold << 1
        } else {
            ctx.enemy_base[idx].gold
        };
        BattleOutcome::Exchange {
            penetrates: true,
            blow: true,
            enemy_dmg: dmg,
            player_dmg: 0,
            kill: true,
            predicted_loss: loss,
            gold,
        }
    }
}

#[cfg(test)]
mod tests {
    //! 【P2 批2 红测试】resolve_bump 三事件解析（设计定案 b19：事件形/写面/保真数值钉）。
    use super::*;
    use crate::combat::EnemyDef;

    struct Fixture {
        p: PlayerCombat,
        enemies: Vec<EnemyDef>,
        base: Vec<EnemyDef>,
        ba: bool,
        bt: i32,
        bd: Vec<i32>,
        bh: i32,
        bf: bool,
        bn: bool,
        br: bool,
        bta: i32,
        btk: i32,
        sbd: bool,
    }

    impl Fixture {
        fn new() -> Fixture {
            Fixture {
                p: PlayerCombat {
                    hp: 100,
                    atk: 10,
                    def: 4,
                    yellow_keys: 0,
                    blue_keys: 0,
                    red_keys: 0,
                    gold: 0,
                    item_types: Vec::new(),
                    item_uses: Vec::new(),
                    item_stack_size: 0,
                },
                // 缩放表 hp=50（≠base 40——源分辨钉）
                enemies: vec![EnemyDef { id: 41, hp: 50, atk: 8, def: 3, gold: 10, trait_flags: 0 }],
                base: vec![EnemyDef { id: 41, hp: 40, atk: 8, def: 3, gold: 20, trait_flags: 0 }],
                ba: false,
                bt: 0,
                bd: Vec::new(),
                bh: 0,
                bf: false,
                bn: false,
                br: false,
                bta: -1,
                btk: 0,
                sbd: false,
            }
        }
        fn ctx(&mut self) -> BattleCtx<'_> {
            BattleCtx {
                battle_active: &mut self.ba,
                battle_anim_tick: &mut self.bt,
                battle_digit_by_type: &mut self.bd,
                battle_enemy_hp: &mut self.bh,
                battle_finishing: &mut self.bf,
                battle_no_counter: &mut self.bn,
                battle_rerun_scene: &mut self.br,
                battle_target: &mut self.bta,
                battle_tick: &mut self.btk,
                show_battle_digits: &mut self.sbd,
                player: &self.p,
                enemies: &self.enemies,
                enemy_base: &self.base,
            }
        }
    }

    /// 开战双模式：撞击面=base 源+无先手+active；途中面=缩放源+敌先手 raw。
    #[test]
    fn p2b2_resolve_engage_modes() {
        let mut f = Fixture::new();
        let mut r = BattleResolver;
        // 撞击面（a.java:5470-5490）：battle_enemy_hp=base.hp(40)
        let out = {
            let mut ctx = f.ctx();
            r.resolve_bump(&mut ctx, EnemyTypeId(41), BattleEvent::BumpEngage { entity_slot: 5 })
        };
        assert!(matches!(out, BattleOutcome::Engage { first_strike: 0 }), "bump 无先手");
        assert_eq!(f.bh, 40, "bump 面 battle_enemy_hp=基础表源");
        assert!(f.bn, "bump 面 battle_no_counter=true");
        assert!(f.ba, "bump 面 battle_active=true");
        assert_eq!(f.bta, 5, "battle_target=事件槽位");
        // 途中面（a.java:8760-8780）：battle_enemy_hp=enemies.hp(50)+first_strike=8-4
        let out = {
            let mut ctx = f.ctx();
            r.resolve_bump(&mut ctx, EnemyTypeId(41), BattleEvent::RouteEngage { entity_slot: 7 })
        };
        assert!(matches!(out, BattleOutcome::Engage { first_strike: 4 }), "route 先手=atk-def raw");
        assert_eq!(f.bh, 50, "route 面 battle_enemy_hp=缩放表源");
        assert!(!f.bn, "route 面 battle_no_counter=false");
        assert!(!f.ba, "route 面 battle_active=false");
        assert_eq!(f.bta, 7);
    }

    /// 拒战（predict 拒绝：loss>=hp）：零装配（写面不触）。
    #[test]
    fn p2b2_resolve_refused() {
        let mut f = Fixture::new();
        f.p.hp = 10; // loss=28 >= hp=10
        let mut r = BattleResolver;
        let out = {
            let mut ctx = f.ctx();
            r.resolve_bump(&mut ctx, EnemyTypeId(41), BattleEvent::BumpEngage { entity_slot: 5 })
        };
        assert!(matches!(out, BattleOutcome::Refused));
        assert_eq!(f.bh, 0, "拒战零装配");
        assert_eq!(f.bta, -1, "拒战不动 battle_target");
    }

    /// 一击交换：dmg/counter/空拍/未击穿/击杀+赏金+终伤预测（确定性数值钉）。
    #[test]
    fn p2b2_resolve_exchange_math() {
        let mut r = BattleResolver;
        // 击打拍（tick 0）：dmg=10-3=7；hp 50→43；counter=8-4=4
        let mut f = Fixture::new();
        f.bh = 50; // Exchange 不装配 hp——预置（Engage 面装配语义见 p2b2_resolve_engage_modes）
        let out = {
            let mut ctx = f.ctx();
            r.resolve_bump(&mut ctx, EnemyTypeId(41), BattleEvent::Exchange)
        };
        let BattleOutcome::Exchange { penetrates, blow, enemy_dmg, player_dmg, kill, predicted_loss, gold } = out
        else {
            panic!("非 Exchange");
        };
        assert_eq!((penetrates, blow, enemy_dmg, player_dmg, kill, predicted_loss, gold), (true, true, 7, 4, false, 0, 0));
        assert_eq!(f.bh, 43, "击打拍扣血");
        // 空拍（tick 1）：不扣血不反击
        let mut f = Fixture::new();
        f.bh = 50;
        f.btk = 1;
        let out = {
            let mut ctx = f.ctx();
            r.resolve_bump(&mut ctx, EnemyTypeId(41), BattleEvent::Exchange)
        };
        let BattleOutcome::Exchange { penetrates, blow, enemy_dmg, player_dmg, kill, predicted_loss, gold } = out
        else {
            panic!("非 Exchange");
        };
        assert_eq!((penetrates, blow, enemy_dmg, player_dmg, kill, predicted_loss, gold), (true, false, 7, 0, false, 0, 0));
        assert_eq!(f.bh, 50, "空拍不扣血");
        // 未击穿（atk 2-def 3）：penetrates=false
        let mut f = Fixture::new();
        f.bh = 50;
        f.p.atk = 2;
        let out = {
            let mut ctx = f.ctx();
            r.resolve_bump(&mut ctx, EnemyTypeId(41), BattleEvent::Exchange)
        };
        let BattleOutcome::Exchange { penetrates, blow, enemy_dmg, .. } = out else {
            panic!("非 Exchange");
        };
        assert!(!penetrates && !blow, "未击穿");
        assert_eq!(enemy_dmg, -1);
        assert_eq!(f.bh, 50, "未击穿不动 battle_enemy_hp");
        // 击杀（hp 5-7→-2）：predicted_loss=28（armor_gate=bn=true）+gold=base.gold(20 无道具25)
        let mut f = Fixture::new();
        f.bh = 5;
        f.bn = true;
        let out = {
            let mut ctx = f.ctx();
            r.resolve_bump(&mut ctx, EnemyTypeId(41), BattleEvent::Exchange)
        };
        let BattleOutcome::Exchange { penetrates, blow, enemy_dmg, player_dmg, kill, predicted_loss, gold } = out
        else {
            panic!("非 Exchange");
        };
        assert_eq!((penetrates, blow, enemy_dmg, player_dmg, kill), (true, true, 7, 0, true));
        assert_eq!((predicted_loss, gold), (28, 20), "终伤 28+赏金 base 源无倍率");
        assert_eq!(f.bh, -2);
        // 击杀+道具25（幸运币二倍）：gold=20<<1=40
        let mut f = Fixture::new();
        f.bh = 5;
        f.p.item_types = vec![25];
        f.p.item_uses = vec![0];
        f.p.item_stack_size = 1;
        let out = {
            let mut ctx = f.ctx();
            r.resolve_bump(&mut ctx, EnemyTypeId(41), BattleEvent::Exchange)
        };
        let BattleOutcome::Exchange { gold, .. } = out else {
            panic!("非 Exchange");
        };
        assert_eq!(gold, 40, "道具 25 赏金二倍");
    }
}
