//! 【P9】**精确拾取 DP**：把闭式解从「纯拾取层」推广到「**拾取集精确 DP**」。
//!
//! ## 为什么需要它（f3 卡点的本质）
//!
//! `pure_pickup` 的定理要求「全捡支配任意子集」，代价是 G1（无 cat8 怪）与 G2（无危险
//! 步臂可达）——而 f3 既有 7 只零伤怪、又有 2 扇 `ty83` 场景门（其中 `param=0` 是换层门、
//! `param=27` 是通出口的 walk-only 门），于是一路回退到通用搜索，在 `node_limit` 上撞墙
//! （20k/200k/1M 三档都给 truncated）。但 f3 的难点**不是「哪些状态更优」而是「18 件拾取物
//! 的子集组合又叠加 5 扇门 / 7 只怪」**——这类层可以**精确求解**而不是搜索。
//!
//! ## 定理（本模块的全部正确性都挂在它上面）
//!
//! 若某层满足闸门 A1–A5（见 [`run`] 的注释表），则
//! 1. 每只 cat8 怪 `predict_engagement(no_counter=true)` 损血 0、无掉落实体（A1）
//!    ⇒ 击杀是**零代价**且**纯单调**的（只缩小可通行格集合的补集）；
//! 2. 无伤害型步臂（A2，复用 `pure_pickup::hazard_gate`）、无非一次性门且门脚本不写 hp
//!    （A3，复用 `hp_bound::plan`）、无击杀生成通道（A4，同源）、拾取物 ≤ [`MAX_PICKUPS`]
//!    （A5，**性能护栏不是正确性假设**）；
//! 3. 拾取物经 `certified_monotone_pickup` 认证为**单调且不加背包**
//!    （`exact_layer_view` 已强制）与「何时捡、之前捡了什么」无关 ⇒ 五个计数器
//!    （hp/atk/def/三钥/gold）都是**已捡位图的线性函数**；
//! 4. 门钥**单调消耗**、怪**零损血**、gate **恒等**（A3+真引擎采样验证）⇒ 状态可压成
//!    `(pos, 拾取位图, 开门位图, 击杀位图)`，player 由位图**推导**而不是存进状态。
//!
//! 于是 `(cell, mask)` 上的**分层 BFS**（每步代价 1，升血是收益不是代价，故不用 Dijkstra）
//! 恰好枚举了全部可达状态 ⇒ 取「到达出口格」的状态里 hp 最大者即**层内精确最优**。
//!
//! ## 与既有模块的关系（禁复制规则表，P9 教训）
//!
//! - 格分类：`Search::classify`（`Cell` 单源）——见 `solve_local::exact_layer_view`；
//! - 危险步臂闸门：`pure_pickup::hazard_gate`（G2′/G2 原样抽出，数值表仍 `STEP_ARM_HAZARD_TYPES`）；
//! - hp 写 / 生成通道 / 一次性门槽：`hp_bound::plan`（1b/2/2b/2c 全部判据原样复用）；
//! - 撞怪 / 开门 / 踩 gate / 拾取：`exact_bump` / `exact_open_door` / `Search::gate_arrive` /
//!   `Search::pickup_step`——全部是 `expand` 的**同一份**实现，DP 里没有第二份引擎语义。

use crate::hp_bound;
use crate::pure_pickup::hazard_gate;
use crate::solve_local::{
    exact_bump, exact_gate_effect, exact_layer_view, exact_replay_route, ClosurePolicy, EntRec,
    ExactLayerView, LayerGoal, PlayerSpec, SolveInput, XC_BLOCKED, XC_DOOR, XC_FREE,
    XC_GATE, XC_MONSTER, XC_PICKUP,
};
use game_core::effect_registry::{certified_monotone_pickup, PickupGain};
#[cfg(test)]
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

/// 拾取物件数上限。**这是性能护栏，不是正确性假设**：
/// 位图规模 2^n，n=20 时 1M 位图 × 每格状态仍在预算内；超过它本层 `Off(too_many_pickups)`
/// 回退通用搜索（仍然正确，只是不再精确）。FA 层实际最大拾取物数是 f3 的 18。
pub const MAX_PICKUPS: usize = 20;
/// 状态数预算。超限 ⇒ `Off(state_cap)`（**不是截断**：截断会给出「非最优但自称精确」的答案）。
///
/// **P9.1 更正**：删掉不健全的反向支配（见 `Dp::push` 的 P9.1 段）后，f3 层的存活状态
/// 由 13 万升到 **4,741,000**（复核 M1 实测，release 3.08s）。预算必须覆盖真实规模，
/// 否则 f3 会退化成 `Off(state_cap)`——那等于用护栏太紧冒充「不可解」。6M 留 ~1.26× 余量。
pub const MAX_STATES: usize = 6_000_000;
/// gate 引擎调用预算（每次 `gate_arrive` 都是一次真脚本执行）。
pub const MAX_GATE_CALLS: usize = 64;
/// 位图总位宽上限（`pos` 之外；`u64` 打包：pos 16 + pick 20 + door 12 + kill 12 = 60）。
const MAX_DIM_BITS: u32 = 56;
/// 门/杀位宽上限（`pack` 的碰撞安全边界，不是性能取舍：16 位放得下 65535 扇）。
const MAX_DOORS: usize = 16;
const MAX_KILLS: usize = 16;

/// 采纳标记（`reason` 的「成功」取值，与 `pure_pickup` 的 `adopted()` 同义）。
pub const ADOPTED: &str = "adopted";

/// 精确 DP 的统计（thread_local 快照；体例同 `PurePickupStat`）。
/// `reason` 是 **fail-closed 的唯一凭据**：只有 `ADOPTED` 表示「本层被精确解接管」。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactStat {
    /// 是否尝试过（`solve_layer_with` 每层都会试，故恒 true 除非路径未达）。
    pub attempted: bool,
    /// 是否接管本层（`true` ⇒ `solve_layer_with` 直接返回本模块给的 goals）。
    pub adopted: bool,
    /// `adopted` 或 `not_run`；否则是具名拒绝理由。
    pub reason: &'static str,
    /// 拒绝理由附带的类型/编号（0 = 无）。
    pub reason_ty: i32,
    pub out_hp: i32,
    pub steps: i32,
    pub picks: usize,
    pub doors: usize,
    pub kills: usize,
    /// 访问过的状态数（`index` 大小）。
    pub states: usize,
    /// 真引擎回放的步数（应与 `steps` 相等；不等即 mismatch）。
    pub replay_steps: i32,
    pub route_len: usize,
    /// 【S6a 复核 F3】`key_floor` 的**层内可达性诊断**（不改 `ok`/`adopted` 语义）：
    /// - `None` = 没给 floor，或层内上界（入场钥匙 + 全层拾取钥匙，忽略开门消耗）已够；
    /// - `Some("key_floor_unreachable_in_layer")` = 上界都 < floor ⇒ 本层**不可能**满足。
    ///
    /// 为什么不直接改 `ok`：用户裁定 m04638——单层 floor 只是软目标，缺口只作诊断，
    /// S6b 接线后由外层 `key_shortfall`（非阻断）兜底。本字段只把「够不到」与
    /// 「路线没走到」分开，不引入第二套成败语义。
    pub key_floor_reason: Option<&'static str>,
}

impl ExactStat {
    pub const CLEARED: Self = Self {
        attempted: false,
        adopted: false,
        reason: "not_run",
        reason_ty: 0,
        out_hp: 0,
        steps: 0,
        picks: 0,
        doors: 0,
        kills: 0,
        states: 0,
        replay_steps: 0,
        route_len: 0,
        key_floor_reason: None,
    };

    pub fn reason_label(&self) -> Option<String> {
        if self.reason == ADOPTED || self.reason == "not_run" {
            None
        } else {
            Some(format!("{}/{}", self.reason, self.reason_ty))
        }
    }
}

thread_local! {
    static STAT: RefCell<ExactStat> = const { RefCell::new(ExactStat::CLEARED) };
}

pub fn exact_stat() -> ExactStat {
    STAT.with(|s| s.borrow().clone())
}

pub fn exact_stat_set(s: ExactStat) {
    STAT.with(|c| *c.borrow_mut() = s);
}

pub fn exact_stat_clear() {
    exact_stat_set(ExactStat::CLEARED);
}

fn give_up(st: &mut ExactStat, reason: &'static str, ty: i32) -> Option<Vec<LayerGoal>> {
    st.reason = reason;
    st.reason_ty = ty;
    None
}

/// 入口：`solve_layer_with` 在 `pure_pickup` 之后、通用 `Search` 之前调用。
///
/// 返回 `Some(goals)` = 已接管（goals 已逐字段经真引擎回放核对）；
/// `None` = 本层不适用/失败，**理由在 `exact_stat()`**（绝不静默降级）。
pub(crate) fn try_exact_dp(
    input: &SolveInput,
    entry: (i32, i32),
    root_ents: &[EntRec],
    player_in: &PlayerSpec,
    flags_in: u128,
    policy: ClosurePolicy,
    shops: &[game_data::defs::ShopGoodDef],
) -> Option<Vec<LayerGoal>> {
    try_exact_dp_floor(input, entry, root_ents, player_in, flags_in, policy, shops, None)
}

/// 带钥匙下限的精确 DP（S6a）：`key_floor` 只影响**终态挑选**，不参与状态支配。
#[allow(clippy::too_many_arguments)]
pub(crate) fn try_exact_dp_floor(
    input: &SolveInput,
    entry: (i32, i32),
    root_ents: &[EntRec],
    player_in: &PlayerSpec,
    flags_in: u128,
    policy: ClosurePolicy,
    shops: &[game_data::defs::ShopGoodDef],
    key_floor: Option<[i32; 3]>,
) -> Option<Vec<LayerGoal>> {
    let mut st = ExactStat { attempted: true, ..ExactStat::CLEARED };
    let out = run(input, entry, root_ents, player_in, flags_in, policy, shops, key_floor, &mut st);
    exact_stat_set(st);
    out
}

/// 状态数预算的**唯一取用点**（判据 4：护栏必须 fail-closed）。
///
/// 生产档 = [`MAX_STATES`]；lib 单测可线程局部覆盖成极小值，用来证明「预算耗尽 ⇒
/// `state_cap` ⇒ `Off`」而不是静默截断或自称最优。故意不用环境变量：测试不许依赖
/// 进程级副作用（并发测试会互相污染）。
#[cfg(test)]
fn state_budget() -> usize {
    STATE_BUDGET.with(|b| b.get().unwrap_or(MAX_STATES))
}

#[cfg(test)]
thread_local! {
    static STATE_BUDGET: Cell<Option<usize>> = const { Cell::new(None) };
}

#[cfg(not(test))]
fn state_budget() -> usize {
    MAX_STATES
}

/// 门采样预算的**唯一取用点**（判据 4：护栏必须 fail-closed）。
///
/// 生产档 = [`MAX_GATE_CALLS`]；lib 单测可线程局部覆盖，用来证明「撞预算 ⇒
/// `gate_budget` ⇒ `Off`」。与 [`state_budget`] 同一理由不用环境变量。
#[cfg(test)]
fn gate_budget() -> usize {
    GATE_BUDGET.with(|b| b.get().unwrap_or(MAX_GATE_CALLS))
}

#[cfg(test)]
thread_local! {
    static GATE_BUDGET: Cell<Option<usize>> = const { Cell::new(None) };
}

#[cfg(not(test))]
fn gate_budget() -> usize {
    MAX_GATE_CALLS
}

const DIRS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

#[allow(clippy::too_many_arguments)]
fn run(
    input: &SolveInput,
    entry: (i32, i32),
    root_ents: &[EntRec],
    player_in: &PlayerSpec,
    flags_in: u128,
    policy: ClosurePolicy,
    shops: &[game_data::defs::ShopGoodDef],
    key_floor: Option<[i32; 3]>,
    st: &mut ExactStat,
) -> Option<Vec<LayerGoal>> {
    // ---- G6' 策略：Q1/Q2 的「零伤怪自动收」语义与「精确最优」不同口径 ⇒ 只接默认面 ----
    if policy != ClosurePolicy::OFF {
        return give_up(st, "policy_not_off", 0);
    }
    let Some(level) = input.level.as_ref() else {
        return give_up(st, "no_level", 0);
    };
    if entry.0 < 0 || entry.1 < 0 || entry.0 >= level.cols || entry.1 >= level.rows {
        return give_up(st, "entry_out_of_bounds", 0);
    }
    if player_in.hp <= 0 {
        return give_up(st, "entry_hp_nonpositive", player_in.hp);
    }

    // ---- A2：危险步臂 —— 与闭式解**同一个**闸门，只把「场景门」划给本模块建模 ----
    // 为什么允许 `ty83` 可达：闭式解把任何步臂都当「不可建模的代价」；本模块**显式**
    // 建了 gate 维（踩上去零代价、且下面逐扇用真引擎验证恒等，非恒等即 Off）
    // ⇒ 可达不再是理由。陷阱/宝箱/组门（`5/60/61/62/76`）仍然不可建模 ⇒ 照旧 Off。
    if let Err(((r, t), _arms)) = hazard_gate(input, entry, root_ents, &[83]) {
        return give_up(st, r, t);
    }

    // ---- A3/A4：非一次性门槽 + 门脚本写 hp + 击杀/脚本生成 hp 通道 ----
    // `hp_bound::plan` 是这三条的单一真相源（1b `post_kill_hp_spawn` / 2b `retriggerable_gate_slot`
    // / 2c `script_hp_spawn`·`script_op_unknown`）。它 Off 的**任何**理由都让本层不精确。
    match hp_bound::plan(input.floor, input.difficulty, level, root_ents, shops) {
        hp_bound::HpBoundPlan::On(_) => {}
        hp_bound::HpBoundPlan::Off(r, t) => return give_up(st, r, t),
    }

    // ---- 层视图（格分类/拾取认证/出口表；A5 的拾取计数在视图里）----
    // 【P9.2】层视图不再吃商品表：商店格的判定读实体 `param` 的机制位
    // （`shop_can_buy`），表只喂下面的 `hp_bound::plan` 与 `Dp` 的 `Search` 面。
    let mut view = match exact_layer_view(input, entry, root_ents) {
        Ok(v) => v,
        Err((r, t)) => return give_up(st, r, t),
    };
    if view.pick_ty.len() > MAX_PICKUPS {
        return give_up(st, "too_many_pickups", view.pick_ty.len() as i32);
    }
    let bits = (view.pick_ty.len() + view.door_ty.len() + view.kill_ty.len()) as u32;
    if bits > MAX_DIM_BITS {
        return give_up(st, "too_many_dims", bits as i32);
    }
    if view.door_ty.len() > MAX_DOORS || view.kill_ty.len() > MAX_KILLS {
        return give_up(st, "too_many_dims", bits as i32);
    }

    // ---- A3 另一半：每扇场景门都必须**恒等**（踩上去不改任何状态），否则 DP 少一维 ----
    // 只对 `GateArrive::Pass` 采样；`Terminal`/`Dropped`（换层门、停摆门）⇒ 该格视作墙
    // （层内路线不使用它——f3 的 (2,11) `param=0` 就是这种），不是拒绝理由。
    let mut gate_calls = 0usize;
    for i in 0..view.cells.len() {
        if view.cells[i].kind != XC_GATE {
            continue;
        }
        let pos = ((i as i32) % view.cols, (i as i32) / view.cols);
        gate_calls += 1;
        if gate_calls > gate_budget() {
            return give_up(st, "gate_budget", gate_calls as i32);
        }
        match exact_gate_effect(input, entry, root_ents, player_in, flags_in, pos) {
            None => view.cells[i].kind = XC_BLOCKED,
            Some((p, f, e)) => {
                if p != *player_in || f != flags_in || e != *root_ents {
                    return give_up(st, "gate_mutates_state", view.cells[i].param);
                }
            }
        }
    }

    // ---- DP ----
    let mut dp = Dp::new(input, view, entry, player_in, root_ents, flags_in);
    // ---- 【S6a 复核 F3】floor 的层内可达性诊断（独立于终态挑选，不碰 ok/adopted）----
    // 上界 = 入场三钥 + **全层**拾取钥增益（`key_sums` 已在 `Dp::new` 按 2^n 建好）。
    // 开门只会减钥、不会加钥 ⇒ 上界不够即层内不可能；够也不保证某条路线真走到
    // （那是终态挑选的事，`key_floor_reason` 不对它下结论）。
    if let Some(f) = key_floor {
        let s = dp.key_sums[dp.key_sums.len() - 1];
        let upper = [
            player_in.yellow + s[0],
            player_in.blue + s[1],
            player_in.red + s[2],
        ];
        if (0..3).any(|i| upper[i] < f[i]) {
            st.key_floor_reason = Some("key_floor_unreachable_in_layer");
        }
    }
    if let Err((r, t)) = dp.search() {
        return give_up(st, r, t);
    }

    // ---- 逐出口取最优：hp 最大 → 满足 key_floor 优先 → steps 最小 → 状态序最小 ----
    //
    // 【S6a】`key_floor` 只做**终态二值优先级**，不参与状态支配（不影响 `states` 规模）：
    // 同 hp 的终态里先挑「出层至少带这些钥匙」的，仍同才比步数。`key_floor = None` 时
    // `(nm, bm)` 恒为 `(true, true)` ⇒ 与旧判据（hp 大、同 hp 步数小）逐字节同序。
    //
    // 终态钥匙直接取现成 `keys_at`（入场钥匙 + 本态拾取钥匙和 − 已开门消耗），
    // 与 `LayerGoal.keys` / 真引擎回放的出层玩家态同一口子，不另写一套算术。
    // DP 不建模 gold（赏金只在 `player_at` 里回填），故序里没有 gold 维——
    // 这里不调 `objective::cmp_exit_floor`（它含 gold），只共用 floor 判定。
    let mut goals: Vec<LayerGoal> = Vec::new();
    // 与 `goals` 一一对应的终态位图 `(pick, door, kill)`——统计口径取自它。
    let mut masks: Vec<(u32, u32, u32)> = Vec::new();
    for &(epos, ekind) in &dp.view.exits {
        let mut best: Option<u32> = None;
        for (i, n) in dp.states.iter().enumerate() {
            if dp.dead[i] || n.pos != epos {
                continue;
            }
            let better = match best {
                None => true,
                Some(b) => {
                    let bs = dp.states[b as usize];
                    let (hb, sb) = (dp.hp_at(bs.pm), bs.steps);
                    let hn = dp.hp_at(n.pm);
                    let (nm, bm) = match key_floor {
                        None => (true, true),
                        Some(f) => {
                            let kb = dp.keys_at(bs.pm, bs.dm);
                            let kn = dp.keys_at(n.pm, n.dm);
                            (
                                crate::objective::meets_key_floor([kn.0, kn.1, kn.2], f),
                                crate::objective::meets_key_floor([kb.0, kb.1, kb.2], f),
                            )
                        }
                    };
                    hn.cmp(&hb)
                        .then(nm.cmp(&bm))
                        .then(sb.cmp(&n.steps))
                        == std::cmp::Ordering::Greater
                }
            };
            if better {
                best = Some(i as u32);
            }
        }
        let Some(idx) = best else { continue };
        let route = dp.route_to(idx);
        let (pm, dm, km) = (dp.states[idx as usize].pm, dp.states[idx as usize].dm, dp.states[idx as usize].km);
        masks.push((pm, dm, km));
        goals.push(LayerGoal {
            hp: dp.hp_at(pm),
            kills: km.count_ones() as i32,
            steps: dp.states[idx as usize].steps,
            keys: dp.keys_at(pm, dm),
            kind: ekind,
            route,
            post: dp.player_at(pm, dm, km),
            post_pos: epos,
            post_ents: dp.ents_at(pm, dm, km),
            post_flags: flags_in,
            cross_edge: None,
        });
    }
    if goals.is_empty() {
        return give_up(st, "exit_unreachable", 0);
    }

    // ---- 真引擎回放（唯一防线）：逐字段核对，任何不一致 ⇒ Off(replay_mismatch) ----
    if let Err((r, t)) = validate_outcomes(input, entry, root_ents, player_in, flags_in, &goals) {
        eprintln!(
            "exact_pickup: floor={} entry={:?} {r}/{t} —— DP 预测与真引擎回放不一致，本层 Off",
            input.floor, entry
        );
        return give_up(st, r, t);
    }

    let (mk, main) = goals
        .iter()
        .enumerate()
        .max_by_key(|(_, g)| (g.hp, -g.steps))
        .expect("non-empty");
    let (pm, dm, km) = masks[mk];
    st.adopted = true;
    st.reason = ADOPTED;
    st.reason_ty = 0;
    st.out_hp = main.hp;
    st.steps = main.steps;
    // 统计口径 = **终态位图**的势（不是 route 里经过拾取格的次数，后者会因重复穿行多计）。
    st.picks = pm.count_ones() as usize;
    st.doors = dm.count_ones() as usize;
    st.kills = km.count_ones() as usize;
    st.states = dp.states.len();
    // `validate_outcomes` 已断言回放步数 == `main.steps`，故这里两者恒等。
    st.replay_steps = main.steps;
    st.route_len = main.route.len();
    Some(goals)
}


/// 玩家态逐轴比较：**背包按 `pure_pickup::trim_slots` 的规范形**比（引擎 `readback`
/// 给定长 32 槽、空槽填 0；入参常用 `[]` 表空）。非零槽仍逐槽比对，不是放宽断言。
fn player_eq(a: &PlayerSpec, b: &PlayerSpec) -> bool {
    a.hp == b.hp
        && a.atk == b.atk
        && a.def == b.def
        && a.yellow == b.yellow
        && a.blue == b.blue
        && a.red == b.red
        && a.gold == b.gold
        && a.item_stack_size == b.item_stack_size
        && crate::pure_pickup::trim_slots(&a.item_types) == crate::pure_pickup::trim_slots(&b.item_types)
        && crate::pure_pickup::trim_slots(&a.item_uses) == crate::pure_pickup::trim_slots(&b.item_uses)
}


/// 回放核对：DP 的**每一条** goal 都要与真引擎逐字段相同（hp/atk/def/三钥/gold/背包/flags/
/// 实体表/位置/步数/击杀数）。任一条不一致 ⇒ `Err`，调用方 `Off`。
///
/// 为什么「任何一条」而不是「只查最好的那条」：非最优出口的不一致同样说明 DP 的状态推导
/// 与引擎语义有分歧，此时**任何**一条结果都不再可信（fail-closed）。
pub(crate) fn validate_outcomes(
    input: &SolveInput,
    entry: (i32, i32),
    root_ents: &[EntRec],
    player_in: &PlayerSpec,
    flags_in: u128,
    goals: &[LayerGoal],
) -> Result<(), (&'static str, i32)> {
    for g in goals {
        let Some(r) = exact_replay_route(input, entry, root_ents, player_in, flags_in, &g.route)
        else {
            return Err(("replay_failed", 0));
        };
        if !player_eq(&r.player, &g.post) {
            return Err(("replay_mismatch", 1));
        }
        if r.pos != g.post_pos {
            return Err(("replay_mismatch", 2));
        }
        // 实体集合按**多重集**比对（顺序不参与判定）：`pickup_step` 会走一次真引擎
        // `try_step` + 回读，引擎侧的实体表顺序与 `root_ents`（关卡装载序）不同，
        // 但两者是同一组实体。顺序差异是引擎内部序，不是语义差异（P9 记录）。
        let mut re = r.ents.clone();
        let mut de = g.post_ents.clone();
        let key = |e: &EntRec| (e.ty, e.x, e.y, e.param);
        re.sort_by_key(key);
        de.sort_by_key(key);
        if re != de {
            return Err(("replay_mismatch", 3));
        }
        if r.flags != flags_in || g.post_flags != flags_in {
            return Err(("replay_mismatch", 4));
        }
        if r.steps != g.steps {
            return Err(("replay_mismatch", 5));
        }
        if r.kills != g.kills {
            return Err(("replay_mismatch", 6));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct SNode {
    pos: (i32, i32),
    pm: u32,
    dm: u32,
    km: u32,
    steps: i32,
    prev: u32,
    /// 低 2 位 = 方向；bit2 = 该转移是撞怪；bit3 = 该转移**开**了一扇门
    /// （都置位时是「走到已开的门格」，回放发普通移动 token）。
    act: u8,
}

struct Dp<'a> {
    input: &'a SolveInput,
    view: ExactLayerView,
    entry: (i32, i32),
    player0: PlayerSpec,
    root_ents: Vec<EntRec>,
    gains: Vec<PickupGain>,
    /// pm → (黄, 蓝, 红) 拾取钥增益（低位递推，供开门热路径 O(1)）。
    key_sums: Vec<[i32; 3]>,
    /// 门位 → (黄=0, 蓝=1, 红=2)。
    door_color: Vec<usize>,
    /// 杀位 → 赏金。
    bounty: Vec<i32>,
    /// 入场 flags（常量；ty75 撞杀是否 restep 读 flag20，见
    /// [`crate::solve_local::ty75_no_restep`]）。
    flags0: u128,
    /// (ty, atk, def) → (损血, 实体表净变化)。损血必须为 0、净变化必须为 0（A1/A4）。
    bump_cache: HashMap<(i32, i32, i32), (i32, i32)>,
    states: Vec<SNode>,
    index: HashMap<u128, u32>,
    /// `(pos, dm, km)`（**不含 pm**）→ 已入队的 `(pm, state)` 列表。
    /// 只用于正向支配查询（见 `push` 的支配引理 + P9.1 更正：反向剪枝已删除）。
    front: HashMap<u128, Vec<(u32, u32)>>,
    /// 历史上用于标记「被后续超集按目标序支配」的状态；P9.1 删掉反向剪枝后没有任何
    /// 代码再把它置 true（挑选侧仍读它，保留字段以免改动面外扩）。
    dead: Vec<bool>,
    /// 状态数预算（默认 `MAX_STATES`；lib 单测把它调小以证明「预算耗尽 ⇒ `state_cap` ⇒
    /// `Off`」——预算护栏必须是具名拒绝，不是静默截断）。
    cap: usize,
}

impl<'a> Dp<'a> {
    fn new(
        input: &'a SolveInput,
        view: ExactLayerView,
        entry: (i32, i32),
        player0: &PlayerSpec,
        root_ents: &[EntRec],
        flags0: u128,
    ) -> Self {
        let gains: Vec<PickupGain> = view
            .pick_ty
            .iter()
            .map(|&ty| certified_monotone_pickup(ty, input.floor).expect("视图已认证"))
            .collect();
        let n = gains.len();
        let mut key_sums: Vec<[i32; 3]> = Vec::with_capacity(1 << n);
        key_sums.push([0, 0, 0]);
        for m in 1..(1usize << n) {
            let low = m & m.wrapping_neg();
            let i = low.trailing_zeros() as usize;
            let mut v = key_sums[m ^ low];
            v[0] += gains[i].yellow;
            v[1] += gains[i].blue;
            v[2] += gains[i].red;
            key_sums.push(v);
        }
        let door_color = view
            .door_ty
            .iter()
            .map(|&ty| match ty {
                1 => 0,
                2 => 2,
                _ => 1,
            })
            .collect();
        Dp {
            input,
            view,
            entry,
            player0: player0.clone(),
            root_ents: root_ents.to_vec(),
            flags0,
            gains,
            key_sums,
            door_color,
            bounty: vec![0; 0],
            bump_cache: HashMap::new(),
            states: Vec::new(),
            index: HashMap::new(),
            front: HashMap::new(),
            dead: Vec::new(),
            cap: state_budget(),
        }
    }

    /// 预算是否已耗尽（单一判定点，`push` 与单测共用）。
    fn cap_reached(&self) -> bool {
        self.states.len() >= self.cap
    }

    /// 状态键：`pos(16) | pm(24) | dm(16) | km(16)` = 72 位（`u128`）。
    /// 位宽是**碰撞安全**要求（不是性能）：`pm ≤ MAX_PICKUPS=20 < 24`，
    /// 门/杀各 ≤ 16 位——超过即 `too_many_dims`，绝不静默撞键。
    fn pack(&self, n: &SNode) -> u128 {
        let pos = ((n.pos.0 as u128) & 0xff) | (((n.pos.1 as u128) & 0xff) << 8);
        pos | ((n.pm as u128) << 16) | ((n.dm as u128) << 40) | ((n.km as u128) << 56)
    }

    /// 同 [`Dp::pack`] 但 **抹掉 pm**：这是支配引理的作用域键。
    fn pack_no_pm(&self, n: &SNode) -> u128 {
        let pos = ((n.pos.0 as u128) & 0xff) | (((n.pos.1 as u128) & 0xff) << 8);
        pos | ((n.dm as u128) << 16) | ((n.km as u128) << 32)
    }

    fn hp_at(&self, pm: u32) -> i32 {
        let mut hp = self.player0.hp;
        for i in 0..self.gains.len() {
            if pm >> i & 1 == 1 {
                hp += self.gains[i].hp;
            }
        }
        hp
    }

    fn atk_def_at(&self, pm: u32) -> (i32, i32) {
        let (mut atk, mut def) = (self.player0.atk, self.player0.def);
        for i in 0..self.gains.len() {
            if pm >> i & 1 == 1 {
                atk += self.gains[i].atk;
                def += self.gains[i].def;
            }
        }
        (atk, def)
    }

    fn gold_at(&self, km: u32) -> i32 {
        let mut gold = self.player0.gold;
        for i in 0..self.bounty.len() {
            if km >> i & 1 == 1 {
                gold += self.bounty[i];
            }
        }
        gold
    }

    fn keys_at(&self, pm: u32, dm: u32) -> (i32, i32, i32) {
        let s = self.key_sums[pm as usize];
        let (mut y, mut b, mut r) = (
            self.player0.yellow + s[0],
            self.player0.blue + s[1],
            self.player0.red + s[2],
        );
        for i in 0..self.door_color.len() {
            if dm >> i & 1 == 1 {
                match self.door_color[i] {
                    0 => y -= 1,
                    1 => b -= 1,
                    _ => r -= 1,
                }
            }
        }
        (y, b, r)
    }

    fn player_at(&self, pm: u32, dm: u32, km: u32) -> PlayerSpec {
        let mut p = self.player0.clone();
        p.hp = self.hp_at(pm);
        let (atk, def) = self.atk_def_at(pm);
        p.atk = atk;
        p.def = def;
        let (y, b, r) = self.keys_at(pm, dm);
        p.yellow = y;
        p.blue = b;
        p.red = r;
        p.gold = self.gold_at(km);
        p
    }

    /// 终态实体表 = 入场实体表减去「已捡 / 已开 / 已杀」的实体（按槽位，**不按格子**——
    /// 同格多实体时按格子推断会串味）。
    fn ents_at(&self, pm: u32, dm: u32, km: u32) -> Vec<EntRec> {
        let mut removed = vec![false; self.root_ents.len()];
        for i in 0..self.view.pick_slot.len() {
            if pm >> i & 1 == 1 {
                removed[self.view.pick_slot[i]] = true;
            }
        }
        for i in 0..self.view.door_slot.len() {
            if dm >> i & 1 == 1 {
                removed[self.view.door_slot[i]] = true;
            }
        }
        for i in 0..self.view.kill_slot.len() {
            if km >> i & 1 == 1 {
                removed[self.view.kill_slot[i]] = true;
            }
        }
        self.root_ents
            .iter()
            .enumerate()
            .filter(|(i, _)| !removed[*i])
            .map(|(_, e)| e.clone())
            .collect()
    }

    /// 状态入队 + **超集支配**剪枝。
    ///
    /// 引理（本模块正确性的第二根柱子）：同一 `(pos, dm, km)` 下，若 `pm₁ ⊇ pm₂` **且**
    /// `steps₁ ≤ steps₂`，则 `pm₂` 被 `pm₁` 支配——两态可走格集合相同（拾取物不挡路、门/怪
    /// 位图相同）、三钥与 hp/atk/def 逐轴 ≥（G3 单调认证），且从 `pm₁` 照抄 `pm₂` 的后续
    /// 路线只会**再多捡**不亏，步数还不更多 ⇒ 任何以 `pm₂` 结尾的解都被 `pm₁` 的同形解不劣地
    /// 覆盖。故 `pm₂` 可以丢弃，且丢弃**不损失精确性**（与 `pure_pickup` 里那条「无独立
    /// hp 源时子集支配恒 0」的负结果是两件事：那里的支配条件被联立成「已捡集合相同」，
    /// 这里是**跨已捡集合**的真支配，且不依赖 hp 上界）。
    ///
    /// # P9.1 更正（对抗复核 F4）：才只保留**正向**剪枝，反向 dead 段删除
    ///
    /// 旧版还有一条反向剪枝：本态 `pm ⊃ 已有 pm₂` ⇒ 标 `pm₂` 为 dead。它**不健全**：
    /// 目标是 `(hp 最大, steps 最小)` 的**字典序**，`pm ⊃ pm₂` 只保证「此刻 hp ≥」，
    /// 不保证「出口 hp 严格更大」，也不保证步数更少。两种坑都实测到了：
    /// 1. `pm ⊃ pm₂` 但步数更多（FIFO 后入队者步数只多不少）⇒ 杀掉步数更少的解：
    ///    f3 层由 88 步/14 件退化为 106 步/18 件（复核报告 F4 的实证）。
    /// 2. 试图用「hp 严格更大才杀」补丁也不行：多出来的 hp 物品可能**正躺在本态后续
    ///    必经路线上**，两边到出口 hp 打平，而被杀的那个步数更少 ⇒ 实测 f3 得 98 步/16 件，
    ///    仍不是 88 步/14 件。即「此刻 hp 严格更大」不蕴含「出口 hp 严格更大」。
    /// 唯一健全的反向谓词是 `steps₁ ≤ steps₂`（见引理）；BFS 是 FIFO，后入队的超集步数只多
    /// 不少 ⇒ 该条件下反向剪枝恒为空。故反向段整体删除，只保留正向剪枝：超集若**先**入队，
    /// 其步数必 ≤ 本态（FIFO），支配引理完整成立。
    ///
    /// 代价：状态数上升（f3 由剪枝后的 13 万升到 474 万）。这是精确性的必要成本，
    /// 用 `MAX_STATES` 预算护栏（耗尽 ⇒ `state_cap` ⇒ `Off`，不静默截断）兜住。
    ///
    /// 返回 `Some(idx)` = 真正入队（调用方据此继续 BFS）；`None` = 被支配/已存在 ⇒ 不展开。
    fn push(&mut self, n: SNode) -> Result<Option<u32>, (&'static str, i32)> {
        let k = self.pack(&n);
        if self.index.contains_key(&k) {
            return Ok(None);
        }
        let kk = self.pack_no_pm(&n);
        if let Some(front) = self.front.get(&kk) {
            // 已有 pm₂ ⊇ pm ⇒ 本态被支配（`n.pm & !pm2 == 0` 即 pm ⊆ pm₂）。
            // 正向剪枝的健全性依赖 FIFO：`pm₂` 先入队 ⇒ `steps₂ ≤ n.steps`。
            if front.iter().any(|&(pm2, _)| n.pm & !pm2 == 0) {
                return Ok(None);
            }
        }
        if self.cap_reached() {
            return Err(("state_cap", self.states.len() as i32));
        }
        let i = self.states.len() as u32;
        self.front.entry(kk).or_default().push((n.pm, i));
        self.states.push(n);
        self.dead.push(false);
        self.index.insert(k, i);
        Ok(Some(i))
    }

    /// 撞怪真值（A1/A4）：`loss` 必须 0，实体表净变化必须 0，否则本层不精确。
    fn bump(&mut self, n: &SNode, ty: i32, bit: u32, slot: usize) -> Result<SNode, (&'static str, i32)> {
        let (atk, def) = self.atk_def_at(n.pm);
        let key = (ty, atk, def);
        let (loss, delta) = match self.bump_cache.get(&key) {
            Some(v) => *v,
            None => {
                let p = self.player_at(n.pm, n.dm, n.km);
                // `slot` 是 `root_ents` 下标（`ExactLayerView::kill_slot`），
                // 因此这里必须传**入场实体表**而不是 `ents_at`（已按 mask 过滤，下标会错位）。
                let scale = self.root_ents.len() as i32;
                let Some(b) = exact_bump(self.input, &p, &self.root_ents, ty, slot) else {
                    return Err(("bump_refused", ty));
                };
                let delta = b.ents.len() as i32 - (scale - 1);
                self.bump_cache.insert(key, (b.loss, delta));
                (b.loss, delta)
            }
        };
        if loss != 0 {
            return Err(("monster_damage", ty));
        }
        if delta != 0 {
            return Err(("kill_spawns_ents", ty));
        }
        Ok(SNode { km: n.km | bit, ..*n })
    }

    fn search(&mut self) -> Result<(), (&'static str, i32)> {
        // 赏金维：先采样一次（零伤 ⇒ 各处一致，只依赖 ty），再入 `bounty`。
        let mut bounties = Vec::with_capacity(self.view.kill_ty.len());
        for i in 0..self.view.kill_ty.len() {
            let ty = self.view.kill_ty[i];
            let p = self.player0.clone();
            let Some(b) = exact_bump(self.input, &p, &self.root_ents, ty, self.view.kill_slot[i])
            else {
                return Err(("bump_refused", ty));
            };
            // A1 的损血检查**不在这里做**：不可达的怪不该拖垮整层（fail-closed 但过严）。
            // 它在 `bump()` 里对**真正要撞的那只**按当时的 atk/def 判（缓存复用）。
            if b.ents.len() as i32 - (self.root_ents.len() as i32 - 1) != 0 {
                return Err(("kill_spawns_ents", ty));
            }
            bounties.push(b.gold);
        }
        self.bounty = bounties;

        let start = SNode {
            pos: self.entry,
            pm: 0,
            dm: 0,
            km: 0,
            steps: 0,
            prev: u32::MAX,
            act: 0,
        };
        let mut q: VecDeque<u32> = VecDeque::new();
        if let Some(s0) = self.push(start)? {
            q.push_back(s0);
        }
        while let Some(i) = q.pop_front() {
            if self.dead[i as usize] {
                continue;
            }
            let n = self.states[i as usize];
            for (d, (dx, dy)) in DIRS.iter().enumerate() {
                let np = (n.pos.0 + dx, n.pos.1 + dy);
                let Some(c) = self.view.cell(np) else { continue };
                let mut nn = SNode {
                    pos: np,
                    pm: n.pm,
                    dm: n.dm,
                    km: n.km,
                    steps: n.steps + 1,
                    prev: i,
                    act: d as u8,
                };
                match c.kind {
                    XC_BLOCKED => continue,
                    XC_MONSTER => {
                        let bit = 1u32 << c.idx;
                        if n.km & bit == 0 {
                            // 撞怪：移入被杀格（与 Search::expand Monster 臂
                            // `ns.pos=(tx,ty)` 同口径——bump() 只改杀位，pos 修正在此）。
                            // 【2026-11-23 更正】旧版注释「不移动」与 expand/引擎回放
                            // 口径相反，DP 路线与 witness 回放互斥（S5 真数据实锤）。
                            // 【S5 复核 F1 子项】ty75@floor<=50 引擎不 restep ⇒ 不移位、不计步。
                            nn = self.bump(&n, c.ty, bit, self.view.kill_slot[c.idx])?;
                            if !crate::solve_local::ty75_no_restep(self.input.floor, c.ty, self.flags0)
                            {
                                nn.pos = np;
                                nn.steps = n.steps + 1;
                            }
                            nn.prev = i;
                            nn.act = (d as u8) | 4;
                        }
                    }
                    XC_DOOR => {
                        let bit = 1u32 << c.idx;
                        if n.dm & bit == 0 {
                            let (y, b, r) = self.keys_at(n.pm, n.dm);
                            let have = match self.door_color[c.idx] {
                                0 => y,
                                1 => b,
                                _ => r,
                            };
                            if have <= 0 {
                                continue;
                            }
                            nn.dm |= bit;
                            nn.act |= 8; // 本转移**开**这扇门（回放要 `door` token）
                        }
                    }
                    XC_PICKUP => {
                        nn.pm |= 1u32 << c.idx;
                    }
                    XC_FREE | XC_GATE => {}
                    _ => continue,
                }
                if let Some(j) = self.push(nn)? {
                    q.push_back(j);
                }
            }
        }
        Ok(())
    }

    fn route_to(&self, idx: u32) -> Vec<String> {
        let mut toks: Vec<String> = Vec::new();
        let mut i = idx;
        while self.states[i as usize].prev != u32::MAX {
            let n = self.states[i as usize];
            let p = self.states[n.prev as usize];
            let (dx, dy) = DIRS[(n.act & 3) as usize];
            let cellpos = (p.pos.0 + dx, p.pos.1 + dy);
            if n.act & 4 != 0 {
                let new_bits = n.km & !p.km;
                let k = new_bits.trailing_zeros() as usize;
                toks.push(format!("bump {}@{},{}", self.view.kill_ty[k], cellpos.0, cellpos.1));
            } else if n.act & 8 != 0 {
                toks.push(format!("door {},{}", cellpos.0, cellpos.1));
            } else {
                toks.push(format!("{},{}", cellpos.0, cellpos.1));
            }
            i = n.prev;
        }
        toks.reverse();
        toks
    }
}

#[cfg(test)]
mod exact_dp_unit_tests {
    use super::*;

    /// 5×3 走廊夹具：y=1 全可走；入口 (0,1)，出口 (4,1)；两枚 ty32(+200hp)。
    /// 两态起步 ⇒ 预算调到 1 时必然在第二次 `push` 处撞护栏（判据 4 的夹具）。
    fn corridor() -> (
        crate::solve_local::SolveInput,
        Vec<EntRec>,
        crate::solve_local::PlayerSpec,
    ) {
        use crate::solve_local::{EntSpec, ExitKind, ExitSpec, LevelSpec, PlayerSpec, SolveInput};
        let level = LevelSpec {
            cols: 5,
            rows: 3,
            terrain: vec![
                false, false, false, false, false, //
                true, true, true, true, true, //
                false, false, false, false, false,
            ],
            start: (0, 1),
            entities: vec![
                EntSpec { ty: 32, x: 1, y: 1, param: 0 },
                EntSpec { ty: 32, x: 2, y: 1, param: 0 },
            ],
            exits: vec![ExitSpec { x: 4, y: 1, kind: ExitKind::Stair, index: 1 }],
            player: PlayerSpec {
                hp: 100,
                atk: 10,
                def: 10,
                yellow: 0,
                blue: 0,
                red: 0,
                gold: 0,
                item_types: vec![0; 32],
                item_uses: vec![0; 32],
                item_stack_size: 1,
            },
            enemies: vec![],
            enemy_base: vec![],
            flags: 0,
        };
        let input = SolveInput { floor: 1, difficulty: 0, level: Some(level) };
        let lv = input.level.as_ref().unwrap();
        let ents: Vec<EntRec> = lv
            .entities
            .iter()
            .map(|e| EntRec { ty: e.ty, x: e.x, y: e.y, param: e.param })
            .collect();
        let player: PlayerSpec = lv.player.clone();
        (input, ents, player)
    }

    /// 判据（验收 #3）：DP 预测与真引擎回放不一致时**必须 Off**，不许返回结果。
    ///
    /// 构造方式：先让 DP 正常解出一层，再**篡改预测的 hp**（模拟「DP 状态推导与引擎
    /// 语义分歧」），断言 `validate_outcomes` 返回 `replay_mismatch`。这是对**防线本身**
    /// 的夹具——真实分歧我们复现不出来（derive 与引擎同源），所以直接对校验器下毒。
    #[test]
    fn exact_dp_corrupted_prediction_is_off() {
        let (input, ents, player) = corridor();
        let mut goals = try_exact_dp(&input, (0, 1), &ents, &player, 0, ClosurePolicy::OFF, &[])
            .expect("合成层应被精确解接管");
        assert_eq!(goals.len(), 1);
        assert_eq!(goals[0].hp, 500, "100 + 2×200");
        // 真实预测 → 通过
        assert!(validate_outcomes(&input, (0, 1), &ents, &player, 0, &goals).is_ok());
        // 篡改 hp → 必须被回放抓住
        goals[0].hp += 1;
        goals[0].post.hp += 1;
        assert_eq!(
            validate_outcomes(&input, (0, 1), &ents, &player, 0, &goals).unwrap_err().0,
            "replay_mismatch"
        );
    }

    /// 判据 4（护栏 fail-closed）：状态预算耗尽 ⇒ **`Off(state_cap)`**，不是静默截断、
    /// 也不是拿一个「非最优但自称精确」的解返回。
    ///
    /// `MAX_PICKUPS` 那条护栏的对应判据在集成测试里
    /// （`tests/exact_pickup.rs::exact_dp_off_on_too_many_pickups`，断言
    /// `reason=="too_many_pickups"`、`reason_ty==21`）；本条补的是状态预算这条——
    /// 它到 P9.1 为止**没有任何判据**，只在源码里存在（复核 F6 指出「失败判据空洞」）。
    #[test]
    fn exact_dp_state_cap_is_off_not_truncation() {
        let (input, ents, player) = corridor();
        STATE_BUDGET.with(|b| b.set(Some(1)));
        let got = try_exact_dp(&input, (0, 1), &ents, &player, 0, ClosurePolicy::OFF, &[]);
        let st = exact_stat();
        STATE_BUDGET.with(|b| b.set(None));
        assert!(got.is_none(), "预算耗尽必须返回 None（Off），实测给出解");
        assert!(!st.adopted, "预算耗尽不得 adopted");
        assert_eq!(st.reason, "state_cap", "必须是具名拒绝，实测={:?}", st.reason);
        assert_eq!(st.reason_ty, 1, "reason_ty 必须带上撞到的预算值");
        assert_eq!((st.steps, st.picks, st.out_hp), (0, 0, 0), "Off 不得携带半成品统计");
    }

    /// 判据 4（护栏 fail-closed，第二道）：门采样预算撞界 ⇒ **`Off(gate_budget)`**。
    ///
    /// 与 [`Self`] 的 state_cap 判据同源：护栏触发时必须给出**具名理由**并让本层不精确，
    /// 不许「采一半就按恒等处理」——那会把没验证过的门当安全门，是静默截断的另一种形态。
    /// 夹具复用 corridor，把两枚 ty32 换成两扇 ty83 `param=0` 场景门（引擎语义：游标无脚本 ⇒ 恒等），
    /// 预算压到 1 ⇒ 采到第 2 扇时撞界。
    ///
    /// 变异自证：把 `if gate_calls > gate_budget() { return give_up(...) }` 的 `return` 改成
    /// `break`（半途而废但继续跑 DP）⇒ 本条红（理由退化成 `exit_unreachable`/`adopted`）。
    #[test]
    fn exact_dp_gate_budget_is_off_not_truncation() {
        use crate::solve_local::EntSpec;
        let (mut input, _ents, player) = corridor();
        input.level.as_mut().unwrap().entities = vec![
            EntSpec { ty: 83, x: 1, y: 1, param: 0 },
            EntSpec { ty: 83, x: 2, y: 1, param: 0 },
        ];
        let lv = input.level.as_ref().unwrap();
        let ents: Vec<EntRec> =
            lv.entities.iter().map(|e| EntRec { ty: e.ty, x: e.x, y: e.y, param: e.param }).collect();
        GATE_BUDGET.with(|b| b.set(Some(1)));
        let got = try_exact_dp(&input, (0, 1), &ents, &player, 0, ClosurePolicy::OFF, &[]);
        let st = exact_stat();
        GATE_BUDGET.with(|b| b.set(None));
        assert!(got.is_none(), "撞门预算必须 Off，实测给出解");
        assert!(!st.adopted, "撞门预算不得 adopted");
        assert_eq!(st.reason, "gate_budget", "必须具名，实测={:?}", st.reason);
        assert_eq!(st.reason_ty, 2, "reason_ty = 实际采样次数");
    }
}
