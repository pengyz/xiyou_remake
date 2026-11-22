//! S4 外层→内层贯通驱动器：把 [`crate::outer::outer_search`] 的 top-K 候选块路径
//! 逐候选「实化」成内层逐层精解任务队列，产出**全程 witness 草案**
//! （逐层 entry/exit/route/in_player/out_player/in_flags/keys）。
//!
//! # 契约（S4 用户裁定，逐条落进代码）
//!
//! - **外层是候选生产者，内层是裁决者**：外层 [`crate::outer::outer_search`] 的
//!   LB 预算（combat-cost 入边权 = `1 + Σ 可杀怪 counter_loss`）是**下界**——
//!   内层真引擎损血只会 ≥ 预测。分歧必须**逐字段具名**落进
//!   [`BudgetDrift`]（`hp` 方向：inner ≤ outer 预测值），绝不静默吞掉。
//! - **禁静默换路**：某候选在内层失败 ⇒ 记 [`CandidateAttempt`]、**取下一候选**
//!   并计数（`candidates_considered`）；全部候选失败 ⇒ `blocked` 带末次
//!   `{floor, reason}`。绝不放宽断言换「有解」。
//! - **内层精解驱动真引擎**：逐层走 [`crate::witness`] 的同一口径
//!   （`solve_local_exit_toward_shops`，其内部已用 P5 闭式加速：纯拾取层
//!   `pure_pickup=true`、单调层走 `exact_pickup`），复用
//!   [`crate::witness::record_floor`] 落账——**不做第二份 witness 记账**。
//! - 跨层实体表**按层归属**（与 `witness.rs` 同模型）：首入 `sprite_full`，
//!   复访用该层上次 `ents_out`；`flags` 全程携带。
//!
//! # 为什么逐层而不是逐块
//!
//! 外层块路径给出的是「块序候选」，内层 [`solve_local_exit_toward_shops`]
//! 的解算粒度是**整层**（入口格 → 出层梯/终局）。因此
//! [`path_to_tasks`] 把连续同层块**归组**为一次「层访问」（复访允许，每次
//! 访问独立一任务），预算对照取组首块的 `state_pre`——组内中间块的
//! 逐格内层状态由 B 片回放（`route` 逐步）提供，不在本模块口径内。
//!
//! # 与 `witness.rs` 的关系
//!
//! `witness.rs` 的 `run_witness_with` 是「塔序单向推进」的驱动器（每层只进
//! 一次、有 cycle 护栏）。外层路径允许复访层（跨层回拾），故本模块不套用
//! 它的循环，而是**镜像它的每步硬约束**（truncated/ok/edge 一致性）并复用
//! 它的落账原语，让两条驱动器的 witness 记账**逐字段同构**。

use crate::outer::{OuterGraph, OuterResult, PathSnapshot, PathReplayError, RunSpec};
use crate::objective::next_demand;
use crate::solve_local::{solve_local_exit_toward_shops, ExitKind, PlayerSpec, SolveInput};
use crate::witness::{
    exit_edge, kind_str, record_floor, WitnessBlocked, WitnessFinal, WitnessFloor,
};
use game_data::defs::{EnemyDef, ShopGoodDef};
use std::collections::BTreeMap;

pub const OUTER_WITNESS_SCHEMA: &str = "xiyou/outer-witness/v1";

/// 一次内层精解任务：连续同层块归组（见模块 doc「为什么逐层而不是逐块」）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathTask {
    /// 第几次层访问（0 起，跨候选从 0 重新计）。
    pub visit: usize,
    pub floor: i32,
    /// 本次访问覆盖的块 id 序（外层路径原序）。
    pub blocks: Vec<String>,
    /// 内层入口预算 = 组首块 `state_pre`（外层 LB 口径）。
    pub entry_budget: crate::outer::OuterState,
    /// 出层目标（下一访问的层）；末任务 `None`（要求 `kind=Ending`）。
    pub toward: Option<i32>,
    pub is_final: bool,
}

/// 把外层块路径快照归组成层访问任务队列。连续同层块合一任务；
/// 预算取组首块 `state_pre`（判据 `s4a_path_to_tasks`）。
pub fn path_to_tasks(snaps: &[PathSnapshot]) -> Vec<PathTask> {
    let mut tasks: Vec<PathTask> = Vec::new();
    for s in snaps {
        match tasks.last_mut() {
            Some(t) if t.floor == s.floor => t.blocks.push(s.block.clone()),
            _ => tasks.push(PathTask {
                visit: tasks.len(),
                floor: s.floor,
                blocks: vec![s.block.clone()],
                entry_budget: s.state_pre.clone(),
                toward: None, // 归组完成后回填
                is_final: false,
            }),
        }
    }
    let n = tasks.len();
    let next_floors: Vec<i32> = tasks.iter().skip(1).map(|t| t.floor).collect();
    for (i, t) in tasks.iter_mut().enumerate() {
        t.is_final = i + 1 == n;
        if !t.is_final {
            t.toward = Some(next_floors[i]);
        }
    }
    tasks
}

/// 外层 LB 预算 vs 内层真值的逐字段分歧（判据 `s4a_budget_divergence_named`）。
///
/// 方向语义：外层入边权是损血**下界**（LB），故 `hp` 字段分歧时
/// `inner ≤ outer`（真损血 ≥ 预测）；其余字段（atk/def/gold/keys）外层
/// 未建模购买/拾取全量，方向不承诺，只如实记两侧数值。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BudgetDrift {
    pub candidate: usize,
    pub visit: usize,
    pub floor: i32,
    pub block: String,
    /// `hp|atk|def|gold|yellow|blue|red`。
    pub field: String,
    pub outer: i32,
    pub inner: i32,
}

/// 一个候选路径的尝试记录（判据 `s4a_no_silent_override` 的计数面）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CandidateAttempt {
    pub index: usize,
    pub steps: u64,
    pub killable: bool,
    pub fatal: bool,
    /// `witness` / `replay_failed:{kind}` / `no_exit` / `no_exit_toward` /
    /// `truncated` / `edge_mismatch` / `ending_before_final` / `model_missing` /
    /// `final_not_ending` / `max_floors`。
    pub outcome: String,
    pub floors_solved: usize,
    pub failed_floor: Option<i32>,
    pub failed_reason: Option<String>,
}

/// 外层搜索侧元信息（自解释口径声明）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OuterRunMeta {
    pub combat_cost: bool,
    pub pops: u64,
    pub truncated: bool,
    pub frontier_len: usize,
    /// 实际消费的候选数上限（CLI `--outer-candidates`）。
    pub candidates: usize,
}

/// 全程 witness 草案（`--outer` 模式的 run-witness.json 根）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OuterWitness {
    pub schema: String,
    /// `witness|blocked|capped`（词表同 `WitnessStatus`）。
    pub status: String,
    /// 被实化消费的候选数（含失败尝试）。
    pub candidates_considered: usize,
    pub attempts: Vec<CandidateAttempt>,
    /// **报告采纳的**那次尝试的成功层前缀（最深者；其余尝试的层不混入）。
    pub floors: Vec<WitnessFloor>,
    pub budget_drifts: Vec<BudgetDrift>,
    pub total_steps: i32,
    #[serde(rename = "final")]
    pub final_state: WitnessFinal,
    pub blocked: Option<WitnessBlocked>,
    pub outer: OuterRunMeta,
}

/// 注入式层求解器（与 [`crate::witness::LayerSolver`] 同签名，测试注入用）。
pub type OuterLayerSolver<'a> =
    &'a dyn Fn(&SolveInput, (i32, i32), &PlayerSpec, u128, Option<&[EntRec]>, Option<i32>, &[ShopGoodDef]) -> (
        crate::solve_local::ExitSolve,
        bool,
    );
use crate::solve_local::EntRec;

/// 驱动器配置。
#[derive(Debug, Clone)]
pub struct OuterWitnessCfg {
    /// 最多实化的候选数（frontier 序 = 外层步数升序）。
    pub candidates: usize,
    /// 成功层前缀上限（防失控；与 `witness.rs` 的护栏同语义）。
    pub max_floors: usize,
}

impl Default for OuterWitnessCfg {
    fn default() -> Self {
        OuterWitnessCfg { candidates: 5, max_floors: crate::witness::DEFAULT_MAX_FLOORS }
    }
}

/// 驱动器（真引擎口径）：逐候选「外层路径 → 内层任务队列 → 逐层精解」。
pub fn run_outer_witness(
    graph: &OuterGraph,
    run: &RunSpec,
    scaled_enemies: &[EnemyDef],
    levels: &BTreeMap<i32, SolveInput>,
    player: PlayerSpec,
    flags: u128,
    shops: &[ShopGoodDef],
    search: &OuterResult,
    cfg: &OuterWitnessCfg,
) -> OuterWitness {
    run_outer_witness_with(
        graph, run, scaled_enemies, levels, player, flags, shops, search, cfg,
        &|input, entry, p, fl, ents, toward, sh| {
            solve_local_exit_toward_shops(input, entry, p, fl, ents, toward, sh)
        },
    )
}

/// 注入式驱动器（测试用；生产走 [`run_outer_witness`]）。
#[allow(clippy::too_many_arguments)]
pub fn run_outer_witness_with(
    graph: &OuterGraph,
    run: &RunSpec,
    scaled_enemies: &[EnemyDef],
    levels: &BTreeMap<i32, SolveInput>,
    player: PlayerSpec,
    flags: u128,
    shops: &[ShopGoodDef],
    search: &OuterResult,
    cfg: &OuterWitnessCfg,
    layer: OuterLayerSolver,
) -> OuterWitness {
    let mut ow = OuterWitness {
        schema: OUTER_WITNESS_SCHEMA.to_string(),
        status: "blocked".to_string(),
        candidates_considered: 0,
        attempts: Vec::new(),
        floors: Vec::new(),
        budget_drifts: Vec::new(),
        total_steps: 0,
        final_state: WitnessFinal { ending_done: false, floor: run.start.0 },
        blocked: None,
        outer: OuterRunMeta {
            combat_cost: run.combat_cost,
            pops: search.pops,
            truncated: search.truncated,
            frontier_len: search.frontier.len(),
            candidates: cfg.candidates,
        },
    };

    // 候选序 = frontier 序（外层步数升序），只取 killable && !fatal 的前 K 个。
    let cands: Vec<&crate::outer::FrontierEntry> = search
        .frontier
        .iter()
        .filter(|e| e.killable && !e.fatal)
        .take(cfg.candidates)
        .collect();
    if cands.is_empty() {
        // 诚实分型：搜索截断且前沿为空 = **预算内没摸到锚**（不是「到了杀不动」），
        // 两者必须具名区分，不许把预算耗尽冒充无解。
        let (reason, detail) = if search.truncated && search.frontier.is_empty() {
            (
                "anchor_not_reached_within_pops",
                format!(
                    "pops={} truncated=true frontier_len=0（搜索预算内未触及锚层 f{}：加预算或如实 blocked，不冒充无解）",
                    search.pops, run.anchor_floor
                ),
            )
        } else {
            (
                "no_killable_candidate",
                format!(
                    "frontier_len={} killable=0（外层敏感表判定无活着杀完锚的候选）",
                    search.frontier.len()
                ),
            )
        };
        ow.blocked = Some(WitnessBlocked {
            floor: run.anchor_floor,
            reason: reason.to_string(),
            detail,
            layer_ok: None,
            layer_truncated: None,
        });
        return ow;
    }

    // 报告面：保留「最深成功前缀」的那次尝试。
    let mut best: Option<(usize, Vec<WitnessFloor>, i32, WitnessBlocked, bool)> = None;

    for (ci, cand) in cands.iter().enumerate() {
        ow.candidates_considered += 1;
        let snaps = match crate::outer::replay_outer_path(graph, run, scaled_enemies, &cand.path) {
            Ok(s) => s,
            Err(e) => {
                ow.attempts.push(CandidateAttempt {
                    index: ci,
                    steps: cand.steps,
                    killable: cand.killable,
                    fatal: cand.fatal,
                    outcome: format!("replay_failed:{}", replay_err_tag(&e)),
                    floors_solved: 0,
                    failed_floor: None,
                    failed_reason: Some(format!("{e:?}")),
                });
                continue;
            }
        };
        let tasks = path_to_tasks(&snaps);

        // —— 逐任务精解（镜像 witness.rs 的硬约束，复用其落账原语）——
        let mut w = crate::witness::Witness {
            status: crate::witness::WitnessStatus::Blocked.as_str().to_string(),
            floors: Vec::new(),
            final_state: WitnessFinal { ending_done: false, floor: run.start.0 },
            blocked: None,
            total_steps: 0,
            meta: crate::witness::WitnessMeta {
                schema: crate::witness::WITNESS_SCHEMA.to_string(),
                start_floor: run.start.0,
                start_entry: [run.start.1.0, run.start.1.1],
                layers: 0,
                steps_metric: "route_waypoints".to_string(),
            },
        };
        let mut cur_player = player.clone();
        let mut cur_flags = flags;
        let mut ents_by_floor: BTreeMap<i32, Vec<EntRec>> = BTreeMap::new();
        let mut entry: (i32, i32) = run.start.1;
        let mut outcome = String::new();
        let mut fail_floor: Option<i32> = None;
        let mut fail_reason: Option<String> = None;
        let mut end_ok = false;

        for task in &tasks {
            if w.floors.len() >= cfg.max_floors {
                outcome = "max_floors".to_string();
                fail_reason = Some(format!("floors={} 上限", cfg.max_floors));
                fail_floor = Some(task.floor);
                break;
            }
            let input = match levels.get(&task.floor) {
                Some(i) => i,
                None => {
                    outcome = "model_missing".to_string();
                    fail_floor = Some(task.floor);
                    fail_reason = Some(format!("f{} 无 LevelSpec", task.floor));
                    break;
                }
            };
            let tlevel = match &input.level {
                Some(l) => l,
                None => {
                    outcome = "model_missing".to_string();
                    fail_floor = Some(task.floor);
                    fail_reason = Some(format!("f{} level=None", task.floor));
                    break;
                }
            };
            // 预算 drift 对照：组首块 state_pre vs 内层入场真值（逐字段具名）。
            let eb = &task.entry_budget;
            let p = &cur_player;
            for (field, o, i) in [
                ("hp", eb.hp, p.hp),
                ("atk", eb.atk, p.atk),
                ("def", eb.def, p.def),
                ("gold", eb.gold, p.gold),
                ("yellow", eb.yellow, p.yellow),
                ("blue", eb.blue, p.blue),
                ("red", eb.red, p.red),
            ] {
                if o != i {
                    ow.budget_drifts.push(BudgetDrift {
                        candidate: ci,
                        visit: task.visit,
                        floor: task.floor,
                        block: task.blocks[0].clone(),
                        field: field.to_string(),
                        outer: o,
                        inner: i,
                    });
                }
            }

            let ents_here = ents_by_floor.get(&task.floor).map(|v| v.clone());
            let demand = next_demand(levels, task.floor);
            let (got, hit) = layer(
                input,
                entry,
                &cur_player,
                cur_flags,
                ents_here.as_deref(),
                task.toward,
                shops,
            );
            // —— 硬约束（逐条对齐 run_witness_with）——
            if got.truncated {
                outcome = "truncated".to_string();
                fail_floor = Some(task.floor);
                fail_reason = Some("层解截断，不可消费".to_string());
                break;
            }
            if !got.ok {
                outcome = "no_exit".to_string();
                fail_floor = Some(task.floor);
                fail_reason = Some(format!("{} 层解 ok=false", task.floor));
                break;
            }
            if task.is_final {
                // 末任务：不换层，要求 Ending（终局边）。
                if got.kind != Some(ExitKind::Ending) {
                    outcome = "final_not_ending".to_string();
                    fail_floor = Some(task.floor);
                    fail_reason = Some(format!(
                        "末任务 kind={:?} ≠ Ending",
                        got.kind.map(kind_str)
                    ));
                    break;
                }
                record_floor(&mut w, task.floor, entry, &got, &cur_player, cur_flags, ents_here.as_deref(), &demand);
                ents_by_floor.insert(task.floor, got.out_ents.clone());
                cur_player = got.out_player.clone();
                w.final_state = WitnessFinal { ending_done: true, floor: task.floor };
                end_ok = true;
                outcome = "witness".to_string();
                break;
            }
            let toward = task.toward.expect("非末任务必有 toward");
            if got.kind == Some(ExitKind::Ending) {
                outcome = "ending_before_final".to_string();
                fail_floor = Some(task.floor);
                fail_reason = Some("非末任务解出 Ending（与外层块序矛盾）".to_string());
                break;
            }
            if !hit {
                outcome = "no_exit_toward".to_string();
                fail_floor = Some(task.floor);
                fail_reason = Some(format!("{} 无指向 f{} 的出层解", task.floor, toward));
                break;
            }
            if got.exit.is_none() {
                outcome = "no_exit".to_string();
                fail_floor = Some(task.floor);
                fail_reason = Some("ok=true 但 exit=None（契约违例）".to_string());
                break;
            }
            let edge = exit_edge(tlevel, &got);
            let Some((target, went_down)) = edge else {
                outcome = "stair_edge_missing".to_string();
                fail_floor = Some(task.floor);
                fail_reason = Some(format!("kind={:?} 无换层边", got.kind.map(kind_str)));
                break;
            };
            if target != toward {
                outcome = "edge_mismatch".to_string();
                fail_floor = Some(task.floor);
                fail_reason = Some(format!("出层边指 f{}，外层块序要求 f{}", target, toward));
                break;
            }
            record_floor(&mut w, task.floor, entry, &got, &cur_player, cur_flags, ents_here.as_deref(), &demand);
            ents_by_floor.insert(task.floor, got.out_ents.clone());
            cur_player = got.out_player.clone();
            cur_flags = got.out_flags;
            let Some(landing) = crate::solve_global::landing(tlevel, target, went_down) else {
                outcome = "landing_missing".to_string();
                fail_floor = Some(target);
                fail_reason = Some(format!("f{} 无落点梯", target));
                break;
            };
            entry = landing;
            if !ents_by_floor.contains_key(&target) {
                let ti = match levels.get(&target) {
                    Some(i) => i,
                    None => {
                        outcome = "model_missing".to_string();
                        fail_floor = Some(target);
                        fail_reason = Some(format!("f{} 无 LevelSpec", target));
                        break;
                    }
                };
                let tl = match &ti.level {
                    Some(l) => l,
                    None => {
                        outcome = "model_missing".to_string();
                        fail_floor = Some(target);
                        fail_reason = Some(format!("f{} level=None", target));
                        break;
                    }
                };
                ents_by_floor.insert(target, crate::solve_local::sprite_full(tl));
            }
            outcome = "in_progress".to_string();
        }

        ow.attempts.push(CandidateAttempt {
            index: ci,
            steps: cand.steps,
            killable: cand.killable,
            fatal: cand.fatal,
            outcome: outcome.clone(),
            floors_solved: w.floors.len(),
            failed_floor: fail_floor,
            failed_reason: fail_reason.clone(),
        });
        let deeper = best.as_ref().map_or(true, |b| w.floors.len() > b.1.len());
        if deeper {
            best = Some((ci, w.floors.clone(), w.total_steps, w.blocked.clone().unwrap_or(WitnessBlocked {
                floor: fail_floor.unwrap_or(run.anchor_floor),
                reason: outcome.clone(),
                detail: fail_reason.clone().unwrap_or_default(),
                layer_ok: None,
                layer_truncated: None,
            }), end_ok));
        }
        if end_ok {
            break; // 第一个到终局的候选即采纳（候选序确定性；计数已留档）
        }
    }

    // —— 汇总报告面 ——
    match best {
        Some((_ci, floors, steps, blocked, end_ok)) => {
            ow.floors = floors;
            ow.total_steps = steps;
            ow.final_state.floor = ow.floors.last().map_or(run.start.0, |f| f.floor);
            ow.final_state.ending_done = end_ok;
            if end_ok {
                ow.status = "witness".to_string();
            } else if blocked.reason == "truncated" {
                ow.status = "capped".to_string();
                ow.blocked = Some(blocked);
            } else {
                ow.blocked = Some(blocked);
            }
        }
        None => {
            // 全部候选失败（或路径回放失败）：blocked 带末次具名。
            let last = ow.attempts.last().cloned();
            ow.blocked = Some(WitnessBlocked {
                floor: last.as_ref().and_then(|a| a.failed_floor).unwrap_or(run.anchor_floor),
                reason: last.as_ref().map_or("no_killable_candidate", |a| a.outcome.as_str()).to_string(),
                detail: last
                    .and_then(|a| a.failed_reason)
                    .unwrap_or_else(|| "全部候选实化失败".to_string()),
                layer_ok: None,
                layer_truncated: None,
            });
        }
    }
    ow
}

fn replay_err_tag(e: &PathReplayError) -> &'static str {
    match e {
        PathReplayError::BadPathToken(_) => "bad_token",
        PathReplayError::UnknownBlock { .. } => "unknown_block",
        PathReplayError::StartMismatch { .. } => "start_mismatch",
        PathReplayError::NotAdjacent { .. } => "not_adjacent",
        PathReplayError::KeyGateShortfall { .. } => "key_shortfall",
    }
}

impl OuterWitness {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("OuterWitness 序列化")
    }
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("OuterWitness 序列化")
    }
}
