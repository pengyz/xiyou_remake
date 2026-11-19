//! 批8：hp 可采纳上界（branch-and-bound 剪枝）——**宁可不剪，不许剪错**。
//!
//! # 问题
//!
//! `solver run` 在 f3 撞 `node_limit`（200k 截断，`out_hp=3827`）。f3 的真实
//! hp 算术上界 = 入场 3227 + 4×50（ty31）+ 3×200（ty32）= **4027**；状态爆炸
//! 来自「为捡药去打怪」的分支（每次战斗净损 hp，收益远小于代价）。用
//! **hp 上界**做 B&B：若某态「再怎么走也不可能超过已知最优 out_hp」，整枝剪掉。
//!
//! # 判据（可采纳性 = 上界不得低估）
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
                if let ShopEffect::AddHp { amount } = g.effect {
                    *out += amount.max(0);
                }
            }
        }
        _ => {}
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
                        certify_entity(a[1], 0, floor, shops, 0, &mut t.konst)?;
                    }
                }
                // CES_ty_cx_cy（`script_ops.rs:238`）：本层生成
                "CES" => {
                    let a = ints(3).ok_or(("script_bad_args", cur))?;
                    certify_entity(a[0], 0, floor, shops, 0, &mut t.konst)?;
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
            o.slots.len() as u32,
            o.pickup_hp.values().filter(|v| **v > 0).count() as u32,
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
}
