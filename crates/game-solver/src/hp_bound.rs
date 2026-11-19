//! 批8：hp 上界剪枝（branch-and-bound）——**宁可不剪，不许剪错**。
//!
//! ~~hp 上界可采纳~~（2026-11-19 证伪，见下「可采纳性口径」）：现行主张只到
//! **当前 68 槽语料实测开/关同解 + 已知反例 f35（低估 2400）已闸门化 Off**。
//!
//! # 问题
//!
//! `solver run` 在 f3 撞 `node_limit`（200k 截断，`out_hp=3827`）。f3 的真实
//! hp 算术上界 = 入场 3227 + 4×50（ty31）+ 3×200（ty32）= **4027**；状态爆炸
//! 来自「为捡药去打怪」的分支（每次战斗净损 hp，收益远小于代价）。用
//! **hp 上界**做 B&B：若某态「再怎么走也不可能超过已知最优 out_hp」，整枝剪掉。
//!
//! # 判据（历史名：可采纳性 = 上界不得低估；成效见「可采纳性口径」）
//!
//! ```text
//! bound(st) = st.hp
//!           + Σ_{e∈st.ents}（该实体的可认证 hp 增量：拾取/掉落/商店）
//!           + Σ_{槽 p 在本层、flags[p]==0}（该槽脚本的可认证 hp 上限）
//! 仅当 bound(st) < best_out_hp（**严格**小于）时剪。
//! ```
//!
//! 严格小于保留「平局最优」：`pick::<HpLoss>` 的次序是 hp 降序 → steps 升序 →
//! route 字典序（`solve_local.rs` 的 `HpLoss::better`），hp 相等的候选必须全部
//! 留下，否则 route/steps 的 tie-break 会漂。
//!
//! # 为什么这是上界（逐通道审计，A 级）
//!
//! 层内**唯一**抬高 hp 的通道（`grep -rn "\.hp = \|hp +=" crates/game-core/src`）：
//!
//! | # | 通道 | 证据 | 本模块处理 |
//! |---|---|---|---|
//! | 1 | 拾取（ItemResolver） | `effect_registry.rs:275-278` `hp=1` 钳位 + `hp += d` | `certified_monotone_pickup(ty,floor).hp`（精确增量） |
//! | 2 | 脚本 `GIN_0_<ty>` → `pickup_item_type` | `script_ops.rs:77-80` | 同 1（`certified_monotone_pickup`），未认证 ⇒ Off |
//! | 3 | 脚本 `RES_0` → `hp=400` | `script_ops.rs:100-113`（仅 easy `difficulty_index==0`） | 绝对写点，取 `max(0,400-hp)` |
//! | 4 | 脚本 `ROS_3_v` → `hp=v` | `script_ops.rs:294` | 绝对写点，取 `max(0,max(v)-hp)` |
//! | 5 | 脚本 `ADD`/`CES` 生成实体 | `script_ops.rs:119-146` / `:238+` | 递归认证生成物（拾取/掉落/商店），他层 `ADD` 记 0 |
//! | 6 | 商店 `AddHp` 商品 | `scene.rs:2586` `player.hp += amount` | 每个商店实体至多买一次 ⇒ 记一次（忽略价格=高估） |
//! | 7 | boss 掉落 | `rules/boss.rs` `event_of`/`post_victory_event_of` + `event_spawns`；求解器 `apply_kill_drops` 只 append 实体 | 递归认证掉落物 hp |
//!
//! **只降不升**的通道不入上界（不影响可采纳性）：战斗损血
//! （`scene.rs:1937/1996/3640`）、陷阱 `apply_hp_delta`（`scene.rs:4271-4274`
//! 负 delta）、`RES` 在 hp>400 时、`ROS_3` 写小值。
//!
//! 明确**不在模型内**因而无需上界的通道：菜单炼丹回血
//! （`scene_menu.rs:690`，搜索动作集无菜单臂）、读档恢复
//! （`scene.rs:577`）、菜单 17 新手重置（`scene.rs:2139`，层外）。
//!
//! # 槽位一次性（为什么每个槽只计一次）
//!
//! `gate::evaluate`（`rules/gate.rs:68`）的**基谓词恒为** `!v.flag(param)`，
//! 任何门触发后其 flag 置位（脚本行尾 `on_line_end` 置 flag / `GUT` 置当前槽
//! flag / 槽内 `DES`·`GUT` 亦置位），因此同一层内每个 param 的门最多触发一次
//! ⇒ 其 hp 奖励最多拿一次。`flags` 里已置位的槽不再计入。
//!
//! # 生成通道：为什么「击杀后才出现的实体」必须整层 Off（2026-11-19 复核 F2）
//!
//! 本模块的三张实体表（`pickup_hp`/`drop_hp`/`shop_hp`）**只在层入口从根态
//! `ents` 播种**（`plan` 步骤 1）；`entity_gain` 只按 `ty`/`param` 查表 ⇒ 任何
//! 「层入口不在 `ents`、之后才被生成」的实体，生成后查表得 0。
//!
//! 活反例（复核 F2，f35，A 级可复现）：击杀 ty69 触发 post-anim 组 7 生成
//! ty32×3（各 +800，合 2400）。杀前根态有 ty69 ⇒ `drop_hp[69]=2400`（掉落被
//! 计入），但杀后 `ents=[ty32×3]` 而表里没有 32（f35 根态无 hp 拾取）⇒
//! `bound = hp`，真实可达 `hp+2400`。实测
//! `should_prune(ord, best=301, hp=200, ents=[ty32×3], flags=MAX) == true`
//! 而该态真实可达 2600 ⇒ **低估 2400** ⇒ 剪错。
//!
//! 因此：本层只要有「击杀后生成 / 脚本生成」的实体含 hp 增益，就整层 `Off`：
//! - 击杀掉落（`rules/boss.rs` 的 `event_of` + `post_victory_event_of`，含
//!   post-anim 收尾组）⇒ `Off("post_kill_hp_spawn", 怪物 ty)`；
//! - 门槽脚本 `CES_ty_cx_cy` / 同层 `ADD_floor_ty_cx_cy` ⇒
//!   `Off("script_hp_spawn", 槽号)`。
//!
//! **为什么不走「把生成物计入界」**：那需要静态证明生成物集合可完整枚举
//! （脚本每条 CES/ADD 链、跨层 ADD 回跳、boss 掉落里再掉 boss 的递归、未知 op）
//! 且生成物类型在表里已播种——`entity_gain` 是「表里有该 ty ⇒ 每颗都算」，
//! 一旦某个生成物类型没播种，`bound` 就少算。静态证明不了 ⇒ 按纪律只能 Off。
//! 注意 Off 的判据**不看**「表里是否已有同型」：f10/f20/f40 那种「根态恰好有
//! ty32」的层同样 Off——放过它们等于用「当前语料不咬」当理由（复核点名禁止）。
//!
//! # 可采纳性口径（2026-11-19 降级，AGENTS §3.6 证伪更正）
//!
//! ~~hp 上界可采纳（任何可达态都不会被剪错）~~ **已证伪**：f35 反例（低估 2400，
//! 见上节与判据 `unit_plan_off_for_post_kill_hp_spawn_f35`）。现行主张只到：
//! 「**当前 68 槽语料实测**启用前后最优解相同（f3 200k 档 on/off 同解），且已知
//! 反例 f35 已被 `post_kill_hp_spawn` 闸门挡在 `Off`」。这是**回归实测**，不是
//! 可采纳性证明：其他生成通道（未建模的脚本/跨层路径）仍可能低估。
//!
//! # 已知不健全边界（登记，不掩盖）
//!
//! 1. **门槽「未执行却已置位」**（`rules/gate.rs` `evaluate` 的 case 0/18/7
//!    `set_flags.push`）：槽 18/19/32 可在脚本未执行时被置位。**不低估**：
//!    基谓词恒 `!v.flag(param)` ⇒ 置位后该槽永不再触发，真实玩到也拿不到它的
//!    奖励 ⇒ 从界里抹掉与真实一致（不是「因为当前无 hp op 才安全」）。当前这
//!    3 槽无 hp op（槽 18=`DES_1_4_4 CES_20_4_4`、19=`CES_84_7_7…`、
//!    32=`TAK_79_79`）；将来若加 hp op，本论证仍成立，不需改代码。
//! 2. **商店可重复购买**（`scene.rs` 购买成功收尾块 `if flags & 4 == 0` 才
//!    `param ^= 512`/移除；`flags&4`=param bit10 时**什么都不做** ⇒ 可反复买）。
//!    本模块只建模一次购买（`shop_hp` 每实体一条）。当前表唯一 hp 商品是
//!    `src=1 idx=12`（`AddHp 2000`），带 repeat 位的两家店是 `idx=5`/`idx=8`
//!    （黄钥匙/黄钥匙换钱）⇒ 今天不低估。**已加闸门**（不靠语料）：
//!    `certify_entity` 见 cat16 且 `param & SHOP_REPEAT != 0` 且效果是 `AddHp`
//!    ⇒ `Off("shop_repeatable", ty)`。
//! 3. **商店 `PickupItem`**：效果是 `pickup_item_type(item)`（`scene.rs:2585`），
//!    与拾取臂同源 ⇒ 按 `hp_gain_of_type(item)` 计入（HpUp 精确、其余 0）。
//!    当前表唯一 `PickupItem` 行是 `src=1 idx=13 → item 18`（`RemoveAllType`，
//!    非 hp）；若将来改成 hp 型，本条自动计入（不需要再改判据）。
//!
//! # 失败即关（Off）
//!
//! 任何**不可认证**的 hp 通道 ⇒ 整层 `Off(reason, ty)`，退回原搜索（行为与
//! 父提交逐字节相同）。理由：漏算 = 低估 = 剪错；多算只是少剪。
//!
//! 与既有 `PillDom` 同风格：诊断面 `hp_bound_stat()` 可观测，禁静默。

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use game_core::effect_registry::{certified_pure_hp_gain, pickup_kind_for_type};
use game_core::rules::boss::{event_of, event_spawns, post_victory_event_of};
use game_core::subsystem::script_host::entity_category_with_floor;
use game_data::defs::{ShopEffect, ShopGoodDef};

use crate::solve_local::{EntRec, LevelSpec};

/// 脚本语料（与 `dynreach::fixture_scripts`（`dynreach.rs:353`）**同一文件**：
/// 生产侧早已 `include_str!` 该夹具作为 68 槽真值；本模块不新增 `LevelSpec`
/// 字段，避免 57 处 `LevelSpec` 字面量与 61 处 `SolveInput` 字面量的结构性改动）。
const CORPUS_RAW: &str = include_str!("../../game-core/tests/fixtures/level_scripts.txt");
/// `script_line_flags` 是定长表（`scene_persist.rs:233` 硬编码 68 槽；flags 位宽
/// 128）——param 越出 [0,128) 无法用 flags 判定「是否还是一次性」，直接 Off。
const MAX_SLOT: i32 = 128;
/// 掉落递归深度上限（boss 掉落里再掉 boss 是病态构造；超限即 Off）。
const DROP_DEPTH_MAX: u32 = 4;
/// 商店 `flags & 4`（`scene.rs` `apply_merchant_offer`）= `param` bit10：置位时
/// 购买成功**不清报价位、不移除实体** ⇒ 可反复购买。`AddHp` 商品因此没有
/// 有限上界 ⇒ 只要见到「repeat 位 + AddHp」就 `Off`（不靠语料）。
const SHOP_REPEAT: i32 = 1 << 10;

/// 本层 hp 上界剪枝的可观测统计（`hp_bound_stat()`）。**禁静默**：
/// 没启用时 `enabled=false` + `reason`/`reason_ty` 说明卡在哪条闸门。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HpBoundStat {
    /// 本层是否启用（`false` ⇒ 退回既有搜索，行为与原版逐字节相同）
    pub enabled: bool,
    /// `enabled=false` 的原因；启用时为 `"on"`；未跑过层=`"not_run"`
    pub reason: &'static str,
    /// 触发 fail-closed 的实体/槽；0 = 与具体类型无关
    pub reason_ty: i32,
    /// 入队侧（dedup 前）被剪的后继态数
    pub pruned_entry: u64,
    /// pop 侧（已入队态被后来更优解超过）被剪的态数
    pub pruned_pop: u64,
    /// 计入上界的门槽数
    pub slots: u32,
    /// 计入上界的 hp 型拾取数（去重后）
    pub pickup_types: u32,
    /// 根态 hp（随本层入场态）
    pub root_hp: i32,
    /// 根态上界 `bound(root)`（诊断：与真实 opt 的差距一眼可见）
    pub root_bound: i32,
}

impl HpBoundStat {
    const CLEARED: HpBoundStat = HpBoundStat {
        enabled: false,
        reason: "not_run",
        reason_ty: 0,
        pruned_entry: 0,
        pruned_pop: 0,
        slots: 0,
        pickup_types: 0,
        root_hp: 0,
        root_bound: 0,
    };
}

thread_local! {
    /// thread_local = 多线程测试隔离；层入口清零、`run_root_with` 末尾写回。
    static HP_BOUND: Cell<HpBoundStat> = Cell::new(HpBoundStat::CLEARED);
    /// 线程级急停（测试用：同一进程内可对照「剪/不剪」两次求解）。
    static HP_BOUND_ENABLED: Cell<bool> = Cell::new(true);
}

/// 最近一次层求解的 hp 上界统计（诊断面；不参与搜索决策）。
pub fn hp_bound_stat() -> HpBoundStat {
    HP_BOUND.with(|c| c.get())
}

/// 层入口清零（`solve_layer_with` 调用；与 `PILL_DOM` 同口径）。
pub fn hp_bound_stat_clear() {
    HP_BOUND.with(|c| c.set(HpBoundStat::CLEARED));
}

fn hp_bound_stat_set(s: HpBoundStat) {
    HP_BOUND.with(|c| c.set(s));
}

/// 层缓存命中：本层搜索**未真正执行**。统计不假装「跑过」（若要跨调用回放
/// 剪枝计数，需把统计写进 `LAYER_MEMO` 值——当前不写，故显式标注 memo_hit）。
pub fn stat_mark_memo_hit() {
    hp_bound_stat_set(HpBoundStat {
        enabled: false,
        reason: "memo_hit",
        ..HpBoundStat::CLEARED
    });
}

/// 线程级开关（默认开）。测试用：同进程内可关掉剪枝跑对照解。
pub fn set_enabled(on: bool) {
    HP_BOUND_ENABLED.with(|c| c.set(on));
}

fn enabled() -> bool {
    HP_BOUND_ENABLED.with(|c| c.get())
}

/// 上界计划：`On` 才剪；`Off` 零行为差异。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HpBoundPlan {
    On(HpBoundOrd),
    Off(&'static str, i32),
}

impl HpBoundPlan {
    pub fn is_on(&self) -> bool {
        matches!(self, HpBoundPlan::On(_))
    }

    pub fn reason(&self) -> (&'static str, i32) {
        match self {
            HpBoundPlan::On(_) => ("on", 0),
            HpBoundPlan::Off(r, t) => (r, *t),
        }
    }
}

/// 一个门槽脚本的 hp 上限：常量奖励 + 绝对写点上界。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SlotTerm {
    /// `GIN_0` hp 型拾取 + 同层 `ADD`/`CES` 认证生成物的 hp 之和
    pub konst: i32,
    /// `ROS_3_v` 的最大绝对写值（无条件生效）
    pub abs_write: i32,
    /// `RES_0` 的绝对写值（仅 easy 生效）
    pub res_write: i32,
}

impl SlotTerm {
    /// 该槽在**当前 hp** 下的乐观增益（i64 防溢出）。
    fn gain(&self, hp: i32, easy: bool) -> i64 {
        let mut g = self.konst as i64;
        let mut m = self.abs_write;
        if easy && self.res_write > m {
            m = self.res_write;
        }
        if m > hp {
            g += (m - hp) as i64;
        }
        g
    }
}

/// 已认证的 hp 上界表（只含通过认证的通道）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HpBoundOrd {
    /// 拾取型 ty → 单颗精确 hp 增量（`certified_monotone_pickup`）
    pickup_hp: BTreeMap<i32, i32>,
    /// 怪物型 ty → 一次击杀的掉落 hp 上界（`event_of` + `post_victory_event_of`）
    drop_hp: BTreeMap<i32, i32>,
    /// 商店 (ty,param) → 一次购买的 `AddHp` 增量
    shop_hp: BTreeMap<(i32, i32), i32>,
    /// 门槽 param → 脚本 hp 上限
    slots: BTreeMap<i32, SlotTerm>,
    /// easy 难度（`difficulty_index==0`）才启用 `RES_0`
    difficulty_easy: bool,
}

impl HpBoundOrd {
    /// 根态上界（诊断 + `no_hp_sources` 闸门）。
    pub fn bound(&self, hp: i32, ents: &[EntRec], flags: u128) -> i64 {
        let mut b = hp as i64;
        for e in ents {
            b += self.entity_gain(e.ty, e.param) as i64;
        }
        for (&p, t) in &self.slots {
            if p >= 0 && p < MAX_SLOT && (flags >> p) & 1 == 0 {
                b += t.gain(hp, self.difficulty_easy);
            }
        }
        b
    }

    /// 计入上界的门槽数（诊断面；`HpBoundStat::slots` 同源）。
    pub fn slot_count(&self) -> u32 {
        self.slots.len() as u32
    }

    /// 计入上界的 hp 型拾取类型数（去重后；`HpBoundStat::pickup_types` 同源）。
    pub fn pickup_type_count(&self) -> u32 {
        self.pickup_hp.values().filter(|v| **v > 0).count() as u32
    }

    fn entity_gain(&self, ty: i32, param: i32) -> i32 {
        if let Some(g) = self.shop_hp.get(&(ty, param)) {
            return *g;
        }
        if let Some(g) = self.pickup_hp.get(&ty) {
            return *g;
        }
        self.drop_hp.get(&ty).copied().unwrap_or(0)
    }
}

/// 剪枝判据的唯一实现点（`BestHp::prunable` 调它——**严格小于**的唯一来源，
/// 变异体只能改这一行）。
///
/// **不是可采纳性证明**：`bound` 只覆盖「根态已播种的实体通道 + 门槽 + 已
/// 闸门化的生成通道」。一般主张已证伪（f35 低估 2400），现行口径见模块文档
/// 「可采纳性口径（2026-11-19 降级）」：仅在 68 槽语料实测同解。
pub fn should_prune(ord: &HpBoundOrd, best_hp: i32, hp: i32, ents: &[EntRec], flags: u128) -> bool {
    ord.bound(hp, ents, flags) < best_hp as i64
}

/// 语料分槽（`\u{1e}` 分隔；与 `ScriptSpawns::from_lines` 同切法）。
fn corpus() -> &'static [&'static str] {
    static SLOTS: OnceLock<Vec<&'static str>> = OnceLock::new();
    SLOTS.get_or_init(|| {
        CORPUS_RAW
            .split('\u{1e}')
            .map(|s| s.trim_matches('\n'))
            .collect()
    })
}

/// 商店报价：与 `Search::shop_offer`（`solve_local.rs:1850`）同判据——
/// `(param>>8)&2 != 0` 才开价；`src = ty==77?0:1`；`idx = param&0xff`。
fn shop_offer<'a>(shops: &'a [ShopGoodDef], ty: i32, param: i32) -> Option<&'a ShopGoodDef> {
    if (param >> 8) & 2 == 0 {
        return None;
    }
    let src = if ty == 77 { 0 } else { 1 };
    let idx = param & 0xff;
    shops.iter().find(|g| g.src == src && g.idx == idx)
}

/// 单个道具 id 的拾取 hp 增益（脚本 `GIN_0_<ty>` 与拾取臂同源）。
///
/// A 级证据（`effect_registry.rs:183-289` 七个 `apply` 实现）：**只有
/// `HpUpEffect` 写 `player.hp`**；Stack/Key/StatUp/Equip/TutorialSeenStack/
/// Passive/UseArm/NoOp 只动背包/键/atk/def/装备/tutorial 标志。因此
/// `kind != "HpUp"` ⇒ hp 增益恒 0；`kind == "HpUp"` 用
/// `certified_pure_hp_gain` 真跑采样取值（未通过认证 ⇒ Err，禁估算）。
fn hp_gain_of_type(ty: i32, floor: i32) -> Result<i32, (&'static str, i32)> {
    if pickup_kind_for_type(ty) == "HpUp" {
        certified_pure_hp_gain(ty, floor)
            .map(|g| g.max(0))
            .ok_or(("pickup_not_certified", ty))
    } else {
        Ok(0)
    }
}

/// 认证一个「会出现在层内」的实体类型的 hp 上界，累加进 `out`。
///
/// 拾取 ⇒ 必须过 `certified_monotone_pickup`（未认证 Off）；怪物 ⇒ 递归掉落；
/// 商店 ⇒ 只认 `AddHp`（其余效果求解器 `apply_shop_effect` 返回 false，不构成
/// 可达购买面）；其余类目（门/装饰/楼梯）不产 hp。
fn certify_entity(
    ty: i32,
    param: i32,
    floor: i32,
    shops: &[ShopGoodDef],
    depth: u32,
    out: &mut i32,
) -> Result<(), (&'static str, i32)> {
    match entity_category_with_floor(ty, floor) {
        2 | 4 => {
            *out += hp_gain_of_type(ty, floor)?;
        }
        8 => {
            let d = drop_gain(ty, floor, shops, depth + 1)?;
            *out += d;
        }
        16 => {
            if let Some(g) = shop_offer(shops, ty, param) {
                if param & SHOP_REPEAT != 0 {
                    // repeat 位 + hp 效果 ⇒ 可反复买 ⇒ 无有限上界（见模块文档边界 2）。
                    if matches!(g.effect, ShopEffect::AddHp { .. }) {
                        return Err(("shop_repeatable", ty));
                    }
                }
                match &g.effect {
                    ShopEffect::AddHp { amount } => *out += (*amount).max(0),
                    // `scene.rs` 购买臂：`pickup_item_type(item)`——与拾取臂同源
                    // ⇒ HpUp 精确计入，其余 kind（Stack/UseArm/NoOp…）恒 0。
                    ShopEffect::PickupItem { item } => *out += hp_gain_of_type(*item, floor)?,
                    _ => {}
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// 生成物（击杀掉落 / 脚本 CES·ADD）可能带来的 hp 增益。
///
/// 与 `certify_entity` 同一套表与认证（不复制数据）；返回 0 表示该生成物
/// 不产 hp。用于「生成通道」闸门：>0 ⇒ 该层上界无法静态保证。
fn spawn_gain(
    ty: i32,
    param: i32,
    floor: i32,
    shops: &[ShopGoodDef],
) -> Result<i32, (&'static str, i32)> {
    let mut g = 0i32;
    certify_entity(ty, param, floor, shops, 0, &mut g)?;
    Ok(g)
}

/// 门槽脚本在**本层**生成实体：生成物含 hp 增益 ⇒ 整层 `Off`。
///
/// 生成物在脚本执行后才进 `ents`，而 `plan` 的表只在层入口播种 ⇒ 生成后
/// `entity_gain` 记 0（f35 同族低估）。`slot` = 触发该脚本的槽号（诊断用）。
fn certify_script_spawn(
    ty: i32,
    floor: i32,
    shops: &[ShopGoodDef],
    slot: i32,
) -> Result<(), (&'static str, i32)> {
    if spawn_gain(ty, 0, floor, shops)? > 0 {
        return Err(("script_hp_spawn", slot));
    }
    Ok(())
}

/// 一次击杀（含 post-victory 收尾组）的掉落 hp 上界。
fn drop_gain(ty: i32, floor: i32, shops: &[ShopGoodDef], depth: u32) -> Result<i32, (&'static str, i32)> {
    if depth > DROP_DEPTH_MAX {
        return Err(("drop_depth_exceeded", ty));
    }
    let mut out = 0i32;
    for ev in [event_of(ty, floor), post_victory_event_of(ty, floor)]
        .into_iter()
        .flatten()
    {
        for s in event_spawns(ev) {
            certify_entity(s.ty, s.extra, floor, shops, depth, &mut out)?;
        }
    }
    Ok(out)
}

/// 解析一个门槽（含 `GUT` 尾链，`GUT` 之外的行尾不继续）。
///
/// `LAY`=本层终止（`script_ops.rs` `op_lay` 置 pending_floor ⇒ 求解器在本层
/// 得到 Terminal 边）：此后脚本在**别层**继续，不影响本层 out_hp ⇒ 停止累计
/// （不是低估：本层已经结束）。
fn parse_slot(
    idx: i32,
    floor: i32,
    shops: &[ShopGoodDef],
) -> Result<SlotTerm, (&'static str, i32)> {
    let mut t = SlotTerm::default();
    let mut cur = idx;
    let mut seen: BTreeSet<i32> = BTreeSet::new();
    loop {
        if cur < 0 || cur as usize >= corpus().len() {
            return Err(("script_slot_out_of_range", cur));
        }
        if !seen.insert(cur) {
            return Err(("script_cycle", cur));
        }
        let line = corpus()[cur as usize];
        let mut jump: Option<i32> = None;
        let mut ended = false;
        for tok in line.split_whitespace() {
            let mut it = tok.split('_');
            let op = it.next().unwrap_or("");
            let rest: Vec<&str> = it.collect();
            let ints = |n: usize| -> Option<Vec<i32>> {
                if rest.len() != n {
                    return None;
                }
                rest.iter().map(|s| s.parse::<i32>().ok()).collect()
            };
            match op {
                // GIN_kind_value（`script_ops.rs:77`）：kind0 = 真拾取（hp 型会加血）
                "GIN" => {
                    let a = ints(2).ok_or(("script_bad_args", cur))?;
                    if a[0] == 0 {
                        match hp_gain_of_type(a[1], floor) {
                            Ok(g) => t.konst += g,
                            Err((r, ty)) => return Err((r, ty)),
                        }
                    }
                }
                // ROS_sub_arg（`script_ops.rs:281-313`）：只 sub==3 写 hp
                "ROS" => {
                    if rest.is_empty() {
                        return Err(("script_bad_args", cur));
                    }
                    let sub = rest[0].parse::<i32>().map_err(|_| ("script_bad_args", cur))?;
                    if sub == 3 {
                        let a = ints(2).ok_or(("script_bad_args", cur))?;
                        t.abs_write = t.abs_write.max(a[1]);
                    } else if sub == 1 {
                        if ints(3).is_none() {
                            return Err(("script_bad_args", cur));
                        }
                    } else if (2..=6).contains(&sub) && ints(2).is_none() {
                        return Err(("script_bad_args", cur));
                    }
                }
                // RES_x（`script_ops.rs:100`）：仅 easy 把 hp 置 400
                "RES" => {
                    t.res_write = t.res_write.max(400);
                }
                // ADD_floor_ty_cx_cy（`script_ops.rs:119`）：他层生成不影响本层 out_hp
                "ADD" => {
                    let a = ints(4).ok_or(("script_bad_args", cur))?;
                    if a[0] == floor {
                        certify_script_spawn(a[1], floor, shops, cur)?;
                    }
                }
                // CES_ty_cx_cy（`script_ops.rs:238`）：本层生成
                "CES" => {
                    let a = ints(3).ok_or(("script_bad_args", cur))?;
                    certify_script_spawn(a[0], floor, shops, cur)?;
                }
                // GUT_n（`script_ops.rs:166`）：跳转（尾链互斥，不是循环）
                "GUT" => {
                    let a = ints(1).ok_or(("script_bad_args", cur))?;
                    jump = Some(a[0]);
                }
                // LAY_n：本层终止
                "LAY" => {
                    ended = true;
                    break;
                }
                // 无 hp 面：对话/演示/音乐/显隐/走位/移除/装备/结束
                "TAK" | "SEE" | "SMS" | "GLV" | "MVS" | "SWD" | "MOV" | "DES" | "END" => {}
                // 未知 op：解析器 `_ => return None`（`script_ops.rs:546`）⇒ 语义未定义
                _ => return Err(("script_op_unknown", cur)),
            }
        }
        if ended {
            break;
        }
        match jump {
            Some(n) => cur = n,
            None => break,
        }
    }
    Ok(t)
}

/// 建本层上界计划。`ents` = 本层**剩余实体表**（根态），`shops` = 商品表。
pub fn plan(
    floor: i32,
    difficulty: i32,
    level: &LevelSpec,
    ents: &[EntRec],
    shops: &[ShopGoodDef],
) -> HpBoundPlan {
    if !enabled() {
        return HpBoundPlan::Off("disabled_by_caller", 0);
    }
    let easy = difficulty == 0;
    let mut ord = HpBoundOrd {
        pickup_hp: BTreeMap::new(),
        drop_hp: BTreeMap::new(),
        shop_hp: BTreeMap::new(),
        slots: BTreeMap::new(),
        difficulty_easy: easy,
    };
    // 1) 实体面：只认证「当前层内可能出现」的类目（根态剩余 + 静态全量）。
    let mut seen_types: BTreeSet<(i32, i32)> = BTreeSet::new();
    for e in ents {
        seen_types.insert((e.ty, e.param & 0xff));
        match entity_category_with_floor(e.ty, floor) {
            2 | 4 => {
                let g = match hp_gain_of_type(e.ty, floor) {
                    Ok(g) => g,
                    Err((r, t)) => return HpBoundPlan::Off(r, t),
                };
                ord.pickup_hp.insert(e.ty, g);
            }
            8 => {
                let d = match drop_gain(e.ty, floor, shops, 0) {
                    Ok(d) => d,
                    Err((r, t)) => return HpBoundPlan::Off(r, t),
                };
                ord.drop_hp.insert(e.ty, d);
            }
            16 => {
                let mut g = 0i32;
                if let Err((r, t)) = certify_entity(e.ty, e.param, floor, shops, 0, &mut g) {
                    return HpBoundPlan::Off(r, t);
                }
                ord.shop_hp.insert((e.ty, e.param), g);
            }
            _ => {}
        }
    }
    // 静态实体里可能有「根态已捡走」的 hp 型：不影响上界（多算安全），
    // 但类型若未认证就说明本层存在未知拾取面 ⇒ Off。
    for e in &level.entities {
        if seen_types.contains(&(e.ty, e.param & 0xff)) {
            continue;
        }
        let mut g = 0i32;
        if let Err((r, t)) = certify_entity(e.ty, e.param, floor, shops, 0, &mut g) {
            return HpBoundPlan::Off(r, t);
        }
    }
    // 1b) 击杀生成通道（复核 F2 / f35）：击杀会把掉落实体 append 进 `ents`，
    //     而掉落在杀前不在表里 ⇒ 生成后 `entity_gain` 记 0 ⇒ 低估。只要本层
    //     **活着的**怪里有「击杀后会生成 hp 型实体」的，整层就不剪（Off）。
    //     数据源=引擎 `rules/boss.rs`（`event_of`/`post_victory_event_of`/
    //     `event_spawns`），不复制表、不硬编码 ty。
    for e in ents {
        if entity_category_with_floor(e.ty, floor) != 8 {
            continue;
        }
        let d = match drop_gain(e.ty, floor, shops, 0) {
            Ok(d) => d,
            Err((r, t)) => return HpBoundPlan::Off(r, t),
        };
        if d > 0 {
            return HpBoundPlan::Off("post_kill_hp_spawn", e.ty);
        }
    }
    // 2) 门槽面：本层所有 ty83 的 param（根态 ∪ 静态）。
    let mut params: BTreeSet<i32> = BTreeSet::new();
    for e in ents {
        if e.ty == 83 {
            params.insert(e.param);
        }
    }
    for e in &level.entities {
        if e.ty == 83 {
            params.insert(e.param);
        }
    }
    for p in params {
        if p < 0 || p >= MAX_SLOT {
            return HpBoundPlan::Off("slot_out_of_range", p);
        }
        let t = match parse_slot(p, floor, shops) {
            Ok(t) => t,
            Err((r, ty)) => return HpBoundPlan::Off(r, ty),
        };
        ord.slots.insert(p, t);
    }
    HpBoundPlan::On(ord)
}

/// 建计划并把诊断写回 `HP_BOUND`（`run_root_with` 唯一入口）。
pub fn plan_and_stat(
    floor: i32,
    difficulty: i32,
    level: &LevelSpec,
    root_hp: i32,
    ents: &[EntRec],
    flags: u128,
    shops: &[ShopGoodDef],
) -> HpBoundPlan {
    let plan = plan(floor, difficulty, level, ents, shops);
    let (reason, reason_ty) = plan.reason();
    let (slots, pickup_types, root_bound) = match &plan {
        HpBoundPlan::Off(_, _) => (0, 0, 0),
        HpBoundPlan::On(o) => (
            o.slot_count(),
            o.pickup_type_count(),
            o.bound(root_hp, ents, flags),
        ),
    };
    // `no_hp_sources`：连乐观上界都等于当前 hp ⇒ 本层没有任何加血通道，
    // 剪枝不可能有收益，直接 Off（与「不启用」逐字节相同）。
    let plan = match plan {
        HpBoundPlan::On(_) if root_bound <= root_hp as i64 => {
            HpBoundPlan::Off("no_hp_sources", 0)
        }
        other => other,
    };
    let (reason, reason_ty) = if matches!(plan, HpBoundPlan::Off("no_hp_sources", _)) {
        ("no_hp_sources", 0)
    } else {
        (reason, reason_ty)
    };
    let (enabled, root_bound) = match &plan {
        HpBoundPlan::On(o) => (true, o.bound(root_hp, ents, flags) as i32),
        HpBoundPlan::Off(_, _) => (false, 0),
    };
    hp_bound_stat_set(HpBoundStat {
        enabled,
        reason,
        reason_ty,
        pruned_entry: 0,
        pruned_pop: 0,
        slots: if enabled { slots } else { 0 },
        pickup_types: if enabled { pickup_types } else { 0 },
        root_hp,
        root_bound,
    });
    plan
}

/// 累加剪枝计数并写回（`run_root_with` 末尾一次；避免逐剪枝 thread_local 写）。
pub fn stat_add_commit(pop: u64, entry: u64) {
    let b = hp_bound_stat();
    hp_bound_stat_set(HpBoundStat {
        pruned_pop: b.pruned_pop + pop,
        pruned_entry: b.pruned_entry + entry,
        ..b
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solve_local::{EntSpec, ExitSpec, LevelSpec};

    fn ord_hp() -> HpBoundOrd {
        HpBoundOrd {
            pickup_hp: BTreeMap::from([(31, 50), (32, 200)]),
            drop_hp: BTreeMap::new(),
            shop_hp: BTreeMap::from([((77, 0x202), 2000)]),
            slots: BTreeMap::from([(
                0,
                SlotTerm {
                    konst: 0,
                    abs_write: 0,
                    res_write: 400,
                },
            )]),
            difficulty_easy: true,
        }
    }

    /// 上界=hp+剩余拾取+商店(一次/实体)+未触发槽的绝对写增量。
    #[test]
    fn unit_bound_counts_pickups_slots_and_shops() {
        let o = ord_hp();
        let ents = [
            EntRec { ty: 31, x: 0, y: 0, param: 0 },
            EntRec { ty: 32, x: 1, y: 0, param: 0 },
            EntRec { ty: 77, x: 2, y: 0, param: 0x202 },
        ];
        // 300 + 50 + 200 + 2000 + (RES 400-300)
        assert_eq!(o.bound(300, &ents, 0), 300 + 50 + 200 + 2000 + 100);
        // 槽 0 已触发（flag0=1）⇒ 不再计
        assert_eq!(o.bound(300, &ents, 1u128 << 0), 300 + 50 + 200 + 2000);
        // hp 已高于 RES 写点 ⇒ 槽增益为 0（不低估：RES 只降到 400）
        assert_eq!(o.bound(500, &ents[..2], 0), 500 + 250);
    }

    /// 【判据 2 的严格性】`bound == best` 必须**不剪**（保平局/最优），只有
    /// `bound < best` 才剪。这是变异体 M2（`<`→`<=`）的靶点。
    #[test]
    fn unit_strict_less_than_is_the_only_prune_point() {
        let o = ord_hp();
        let ents = [EntRec { ty: 31, x: 0, y: 0, param: 0 }];
        // slot0 已置位 ⇒ 不计槽；bound(1000) = 1000+50 = 1050
        let flags = 1u128 << 0;
        assert!(!should_prune(&o, 1050, 1000, &ents, flags), "平局（bound==best）不得剪");
        assert!(should_prune(&o, 1051, 1000, &ents, flags), "严格小于才剪");
        // 槽未触发但 hp(1000) 已高于 RES 写点 400 ⇒ 槽增益 0，bound 仍是 1050
        assert!(!should_prune(&o, 1050, 1000, &ents, 0));
        // hp=300 时槽增益 100 ⇒ bound=450；best=451 才剪
        assert!(!should_prune(&o, 450, 300, &ents, 0));
        assert!(should_prune(&o, 451, 300, &ents, 0));
    }

    fn level_with_entities(entities: Vec<EntSpec>) -> LevelSpec {
        LevelSpec {
            cols: 3,
            rows: 3,
            terrain: vec![true; 9],
            start: (0, 0),
            entities,
            exits: vec![ExitSpec {
                x: 2,
                y: 2,
                kind: crate::solve_local::ExitKind::Stair,
                index: 0u8,
            }],
            player: crate::solve_local::PlayerSpec::from_combat(
                game_core::combat::PlayerCombat {
                    hp: 300,
                    atk: 10,
                    def: 10,
                    yellow_keys: 0,
                    blue_keys: 0,
                    red_keys: 0,
                    gold: 0,
                    item_types: Vec::new(),
                    item_uses: Vec::new(),
                    item_stack_size: 0,
                },
            ),
            enemies: Vec::new(),
            enemy_base: Vec::new(),
            flags: 0,
        }
    }

    /// 【判据 3 的闸门面】槽号越界 ⇒ 必须 Off（不猜、不忽略）——变异体 M3
    /// （解析失败时静默用 default 继续）的靶点。
    #[test]
    fn unit_plan_off_for_out_of_range_slot() {
        set_enabled(true);
        let lv = level_with_entities(vec![EntSpec {
            ty: 83,
            x: 1,
            y: 1,
            param: 5000,
        }]);
        let p = plan(1, 0, &lv, &[], &[]);
        set_enabled(false);
        assert_eq!(p.reason(), ("slot_out_of_range", 5000), "越界槽必须 fail-closed");
    }

    /// 真实槽 0：`RES_0` 在 `LAY_2` **之后** ⇒ 本层脚本在 LAY 处终止，对**本层**
    /// out_hp 的贡献为 0（RES 作用于换层后的下一层）。这钉住「不凭空加分」：
    /// 若把 LAY 之后的 op 计入，根态上界会被抬高 100，剪枝就失去可采纳性依据
    /// （允许剪 → 可能剪错）。两颗认证 hp 拾取仍须精确计入。
    #[test]
    fn unit_plan_slot_zero_after_lay_contributes_zero() {
        set_enabled(true);
        let lv = level_with_entities(vec![
            EntSpec { ty: 83, x: 1, y: 1, param: 0 },
            EntSpec { ty: 31, x: 0, y: 0, param: 0 },
            EntSpec { ty: 32, x: 2, y: 0, param: 0 },
        ]);
        let ents = [
            EntRec { ty: 31, x: 0, y: 0, param: 0 },
            EntRec { ty: 32, x: 2, y: 0, param: 0 },
        ];
        let p = plan(1, 0, &lv, &ents, &[]);
        set_enabled(false);
        match p {
            HpBoundPlan::On(o) => {
                assert_eq!(o.bound(300, &ents, 0), 300 + 50 + 200);
            }
            other => panic!("应 On：{:?}", other.reason()),
        }
    }

    /// 【判据 A1（复核 F2 红→绿）】f35 的活反例：击杀 ty69 触发 post-anim 组 7
    /// 生成 ty32×3（各 +800）。杀前界把 2400 记在 `drop_hp[69]`，杀后这 3 颗
    /// 不在根态播种表里 ⇒ `entity_gain(32)=0` ⇒ 旧实现 `bound=200` 而真实可达
    /// 2600 ⇒ `should_prune(best=301)` 会剪掉真最优的支路。
    /// 修复：本层含「击杀后生成 hp」通道 ⇒ `plan()` 必须 `Off`（整层不剪）。
    #[test]
    fn unit_plan_off_for_post_kill_hp_spawn_f35() {
        set_enabled(true);
        let lv = level_with_entities(vec![]);
        let ents = [EntRec { ty: 69, x: 1, y: 1, param: 0 }];
        let p = plan(35, 0, &lv, &ents, &[]);
        set_enabled(false);
        assert_eq!(
            p.reason(),
            ("post_kill_hp_spawn", 69),
            "f35 杀 69 会生成 hp 型（组 7）⇒ 必须 Off，不许剪"
        );

        // 反例算术（旧口径）：根态 ty69 ⇒ drop_hp[69]=2400；杀后态没有 ty69、
        // 也没有 ty32 的播种项 ⇒ bound=200。真实可达 200+2400=2600。
        let old = HpBoundOrd {
            pickup_hp: BTreeMap::new(),
            drop_hp: BTreeMap::from([(69, drop_gain(69, 35, &[], 0).expect("组4+组7"))]),
            shop_hp: BTreeMap::new(),
            slots: BTreeMap::new(),
            difficulty_easy: true,
        };
        assert_eq!(old.drop_hp.get(&69).copied(), Some(2400), "组 7 = ty32×3×800");
        let after = [
            EntRec { ty: 32, x: 0, y: 0, param: 0 },
            EntRec { ty: 32, x: 1, y: 0, param: 0 },
            EntRec { ty: 32, x: 2, y: 0, param: 0 },
        ];
        assert_eq!(old.bound(200, &after, u128::MAX), 200, "杀后态：3 颗 hp 药按 0 计");
        assert!(
            should_prune(&old, 301, 200, &after, u128::MAX),
            "旧实现确实会剪掉真实可达 2600 的态（低估 2400）"
        );
        // 修复后该层 plan() 是 Off ⇒ `hplan` 不是 On ⇒ 剪点整段不执行。
        assert!(!p.is_on());
    }

    /// 【判据 A2 · 严格 fail-closed 口径】即使根态**已有**同型 hp 拾取
    /// （f10/f20/f40 的情形：根态有 ty32），只要本层有「击杀后生成 hp」的通道
    /// 就必须 Off。变异体 M2（放宽成「表里已有同型 ⇒ 不 Off」）会让本判据变红。
    #[test]
    fn unit_plan_off_for_kill_spawn_even_when_hp_type_registered() {
        set_enabled(true);
        let lv = level_with_entities(vec![]);
        let ents = [
            EntRec { ty: 75, x: 1, y: 1, param: 0 }, // 组 0：ty32×3 + 钥匙/药
            EntRec { ty: 32, x: 0, y: 0, param: 0 },  // 根态已有 hp 拾取
        ];
        let p = plan(10, 0, &lv, &ents, &[]);
        set_enabled(false);
        assert_eq!(p.reason(), ("post_kill_hp_spawn", 75));
    }

    /// 【判据 A3 · 脚本生成通道】槽 62 = `CES_69_5_5 …`（在**本层**生成 ty69）。
    /// floor 35 的 ty69 带 post-anim 组 7（ty32×3）⇒ 生成物含 hp ⇒ Off；
    /// floor 3 的 ty69 只触发组 4（ty16，非 hp）⇒ 不得过度 Off。
    #[test]
    fn unit_plan_off_for_script_ces_hp_spawn() {
        set_enabled(true);
        let lv = level_with_entities(vec![EntSpec { ty: 83, x: 1, y: 1, param: 62 }]);
        let p35 = plan(35, 0, &lv, &[], &[]);
        let p3 = plan(3, 0, &lv, &[], &[]);
        set_enabled(false);
        assert_eq!(p35.reason(), ("script_hp_spawn", 62), "CES_69@35 生成物含 hp");
        assert!(p3.is_on(), "CES_69@3 只生成 ty16（非 hp）⇒ 不该 Off");
    }

    fn addhp_good(src: i32, idx: i32, amount: i32) -> ShopGoodDef {
        ShopGoodDef {
            id: 16,
            src,
            idx,
            price: 1000,
            effect: ShopEffect::AddHp { amount },
            fail_hint: "没有足够的金钱".to_string(),
        }
    }

    /// 【判据 A4 · 商店可重复购买】`flags&4`（param bit10）时引擎购买成功
    /// 不清报价位、不移除实体（`scene.rs` 收尾块）⇒ `AddHp` 可反复买 ⇒ 无有限
    /// 上界。真实表里唯一 hp 商品 = `src=1 idx=12`（+2000）；取 param=(6<<8)|12
    /// （flags=6 ⇒ repeat 位开）必须 Off。
    #[test]
    fn unit_plan_off_for_repeatable_addhp_shop() {
        set_enabled(true);
        let lv = level_with_entities(vec![]);
        let ents = [EntRec { ty: 78, x: 1, y: 1, param: (6 << 8) | 12 }];
        let shops = [addhp_good(1, 12, 2000)];
        let p = plan(45, 0, &lv, &ents, &shops);
        // 同一家店去掉 repeat 位（flags=3，f45 的真实 param）⇒ 一次购买可精确计入。
        let ents_once = [EntRec { ty: 78, x: 1, y: 1, param: (3 << 8) | 12 }];
        let p_once = plan(45, 0, &lv, &ents_once, &shops);
        set_enabled(false);
        assert_eq!(p.reason(), ("shop_repeatable", 78), "repeat 位 + AddHp ⇒ 无有限上界");
        match p_once {
            HpBoundPlan::On(o) => assert_eq!(o.bound(300, &ents_once, 0), 300 + 2000),
            other => panic!("一次性 AddHp 店应 On：{:?}", other.reason()),
        }
    }

    /// 【判据 A5 · 商店 PickupItem 面】`PickupItem{item}` 引擎走
    /// `pickup_item_type(item)`（与拾取臂同源）⇒ hp 型按认证增量计入，不当作 0。
    #[test]
    fn unit_plan_counts_shop_pickup_item_hp() {
        set_enabled(true);
        let lv = level_with_entities(vec![]);
        let ents = [EntRec { ty: 78, x: 1, y: 1, param: (2 << 8) | 20 }];
        let shops = [ShopGoodDef {
            id: 18,
            src: 1,
            idx: 20,
            price: 0,
            effect: ShopEffect::PickupItem { item: 32 },
            fail_hint: String::new(),
        }];
        let p = plan(3, 0, &lv, &ents, &shops);
        set_enabled(false);
        match p {
            HpBoundPlan::On(o) => assert_eq!(o.bound(300, &ents, 0), 300 + 200),
            other => panic!("应 On：{:?}", other.reason()),
        }
    }
}
