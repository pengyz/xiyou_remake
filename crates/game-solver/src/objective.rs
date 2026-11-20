//! 【P10】出层目标分层契约（修订版，2026-11-20 用户裁定）。
//!
//! # 为什么是「分层」而不是加权和
//!
//! 旧出层比较（[`crate::solve_local::HpLoss`]）只有 `(hp, steps)`：gold 不进比较，
//! f3 那样的层解「捡多少金币」完全随机。本模块把出层比较改为**字典序**：
//!
//! 1. `hp` 大者优先（软 1）；
//! 2. 同 hp ⇒ `gold` 多者优先（软 2）；
//! 3. 同 hp 同 gold ⇒ `steps` 少者优先。
//!
//! **禁止把 gold 折算进 hp 做加和**：λ=2（AddHp2000 价 1000 金，
//! `data/tables/shop_goods.json` id12）做加和会让求解器用 1999 血换 1000 金——
//! 与用户裁定「不为攒钱牺牲血」冲突；且金币可花费上界未建模（f7 商店黄×5¤50 等
//! 可重复消费面没有全局预算），加和会系统性高估金币价值。
//!
//! # demand 为什么**不在**比较里（2026-11-20 用户裁定，覆盖早先「硬约束层」方案）
//!
//! 钥匙是**跨层动态资源**：不够时可以回早前楼层拾取剩余钥匙、用商店买（f7 黄×5¤50）、
//! 或 SwapYellowKey。把「出本层时带够下一层钥匙」当单层硬约束会制造假不可行——钥匙
//! 的来源根本不在本层。正确的归属是**链路层（witness）持有钥匙状态**；单层的
//! [`Demand`]（`next_demand` 产出）只是**诊断字段**：witness 落账 `demand/keys_out/
//! key_shortfall`，供人工审计与下一片「链路层回拾/补钥」使用，**绝不**进入比较、
//! 过滤或可行性判定（判据 `demand_fields_are_diagnostic_only` 钉死）。
//!
//! `next_demand` 是**一步前瞻**：只看本层 `StairUp` 后继层的 keydemand。它的已知局限
//! （写进 `docs/findings/2026-11-20-layer-objective.md`）：不建模「回早层拾取」，
//! needed 偏保守（逐色最大）；这些局限正是把约束放在链路层的理由。

use crate::keydemand::{analyze_floor, FloorStatus};
use crate::solve_local::{LayerGoal, SolveInput};
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// 下一层钥匙需求（诊断面；**不参与任何比较**——见模块 doc）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemandState {
    /// 后继 keydemand 给出了确定数字。
    Known,
    /// 需求数字未定义（后继缺规格 / keydemand 结构性无解如 no_entry）。
    /// 携带原因 tag（与 `FloorStatus::tag` 同词表 + `model_missing`）。
    Undefined(String),
}

/// 一层的后继钥匙需求（逐色最大，保守）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Demand {
    /// (yellow, blue, red)。
    pub keys: [u8; 3],
    pub state: DemandState,
}

/// 出层解的观测投影（比较的唯一输入面）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitScore {
    pub hp: i32,
    pub gold: i32,
    /// 出层携带钥匙 (y,b,r)。**诊断字段**：留在结构体里供审计，比较不用它。
    pub keys: [u8; 3],
    pub steps: i32,
}

/// 出层比较（本片核心契约，2026-11-20 用户裁定修订：**纯软序**）：
/// hp 大者优先 → 同 hp gold 多者优先 → 同 hp 同 gold steps 少者优先。
/// Equal ⇒ 调用方可用路线字典序做决定性 tie-break（契约外，不改序）。
pub fn cmp_exit(a: &ExitScore, b: &ExitScore) -> Ordering {
    b.hp.cmp(&a.hp)
        .then(b.gold.cmp(&a.gold))
        .then(a.steps.cmp(&b.steps))
}

/// 层目标 → 观测投影。gold 取出层玩家态；keys 取目标集记录的出层钥匙。
pub fn score_of_goal(g: &LayerGoal) -> ExitScore {
    let clamp = |v: i32| v.clamp(0, u8::MAX as i32) as u8;
    ExitScore {
        hp: g.hp,
        gold: g.post.gold,
        keys: [clamp(g.keys.0), clamp(g.keys.1), clamp(g.keys.2)],
        steps: g.steps,
    }
}

/// 一层的后继钥匙需求（诊断面；一步前瞻）。
///
/// - 后继层 = [`crate::floorgraph`] 的全部 `StairUp` 边目标（不硬编码层号）；
/// - 多后继 ⇒ 逐色最大 needed（保守）；
/// - 任一后继缺规格 ⇒ `Undefined("model_missing")`；
/// - 后继 keydemand 状态非 [`FloorStatus::Ok`] ⇒ `Undefined(状态 tag)`（不复制
///   keydemand 逻辑，只调用 [`analyze_floor`]）。
pub fn next_demand(levels: &BTreeMap<i32, SolveInput>, floor: i32) -> Demand {
    let lays = crate::floorgraph::lay_targets();
    let Some(input) = levels.get(&floor) else {
        return Demand { keys: [0, 0, 0], state: DemandState::Undefined("model_missing".into()) };
    };
    let Some(level) = input.level.as_ref() else {
        return Demand { keys: [0, 0, 0], state: DemandState::Undefined("model_missing".into()) };
    };
    let mut succ: Vec<i32> = crate::floorgraph::edges_of(floor, level, &lays)
        .into_iter()
        .filter(|e| e.kind == crate::floorgraph::EdgeKind::StairUp)
        .map(|e| e.to)
        .collect();
    succ.sort();
    succ.dedup();
    if succ.is_empty() {
        return Demand { keys: [0, 0, 0], state: DemandState::Undefined("no_up_exit".into()) };
    }
    let mut need = [0u8; 3];
    for to in succ {
        let Some(tinput) = levels.get(&to) else {
            return Demand { keys: [0, 0, 0], state: DemandState::Undefined("model_missing".into()) };
        };
        let Some(tlevel) = tinput.level.as_ref() else {
            return Demand { keys: [0, 0, 0], state: DemandState::Undefined("model_missing".into()) };
        };
        let d = analyze_floor(to, tlevel);
        if d.status != FloorStatus::Ok {
            return Demand { keys: [0, 0, 0], state: DemandState::Undefined(d.status.tag().into()) };
        }
        let Some(n) = d.needed else {
            return Demand { keys: [0, 0, 0], state: DemandState::Undefined(d.status.tag().into()) };
        };
        for i in 0..3 {
            need[i] = need[i].max(n[i].clamp(0, u8::MAX as i32) as u8);
        }
    }
    Demand { keys: need, state: DemandState::Known }
}
