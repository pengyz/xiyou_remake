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
    /// S5 段分治：段末任务成功判据。`None`（默认）= 要求 `ExitKind::Ending`
    /// （全题通关语义，现状）；`Some(ty)` = 要求 `ExitKind::BossKill`（撞杀锚
    /// boss ty 即达段末——段锚 f10/f20/f35 非通关层，Ending 判据在段式 witness
    /// 恒假）。注入层闭包据此选 `solve_local_boss_toward_shops`。
    pub boss_target: Option<i32>,
}

impl Default for OuterWitnessCfg {
    fn default() -> Self {
        OuterWitnessCfg {
            candidates: 5,
            max_floors: crate::witness::DEFAULT_MAX_FLOORS,
            boss_target: None,
        }
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

    // 候选序 = frontier 序（外层步数升序）：先取 killable && !fatal，再补
    // killable && fatal——fatal 是 LB 口径（frontier.hp=Σ块内 max counter_loss 的
    // 下界，乐观门可把 hp 压负）⇒ LB fatal 不证明真路径必死；精确层解才是真裁判。
    // 只滤掉 killable=false（打不动锚）。
    let mut cands: Vec<&crate::outer::FrontierEntry> = search
        .frontier
        .iter()
        .filter(|e| e.killable && !e.fatal)
        .collect();
    if cands.len() < cfg.candidates {
        for e in search.frontier.iter().filter(|e| e.killable && e.fatal) {
            if cands.len() >= cfg.candidates {
                break;
            }
            cands.push(e);
        }
    }
    cands.truncate(cfg.candidates);
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
                    "frontier_len={} 可杀候选=0（外层敏感表判定：无「活着且杀得完锚」的候选）",
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
    let mut best: Option<(usize, Vec<WitnessFloor>, i32, WitnessBlocked, bool, bool)> = None;

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
        let mut ending_done = false;

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
                // 末任务：默认=不换层，要求 Ending（终局边）；段分治
                // （boss_target 注入）=要求 BossKill（撞杀锚 boss）——f50 锚除外：
                // floor50 击败型72 直接走引擎终局边（Ending），同为合法段末。
                let boss_mode = cfg.boss_target.is_some();
                let ok_final = if boss_mode {
                    matches!(got.kind, Some(ExitKind::BossKill) | Some(ExitKind::Ending))
                } else {
                    got.kind == Some(ExitKind::Ending)
                };
                if !ok_final {
                    outcome = if boss_mode {
                        "final_not_bosskill".to_string()
                    } else {
                        "final_not_ending".to_string()
                    };
                    fail_floor = Some(task.floor);
                    fail_reason = Some(format!(
                        "末任务 kind={:?} ≠ 段末判据({})",
                        got.kind.map(kind_str),
                        if boss_mode { "BossKill|Ending" } else { "Ending" }
                    ));
                    break;
                }
                record_floor(&mut w, task.floor, entry, &got, &cur_player, cur_flags, ents_here.as_deref(), &demand, None);
                ents_by_floor.insert(task.floor, got.out_ents.clone());
                cur_player = got.out_player.clone();
                // BossKill≠通关：ending_done 只在真 Ending 时为真（不许冒充通关）。
                let is_ending = got.kind == Some(ExitKind::Ending);
                w.final_state = WitnessFinal { ending_done: is_ending, floor: task.floor };
                end_ok = true;
                ending_done = is_ending;
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
            record_floor(&mut w, task.floor, entry, &got, &cur_player, cur_flags, ents_here.as_deref(), &demand, None);
            ents_by_floor.insert(task.floor, got.out_ents.clone());
            cur_player = got.out_player.clone();
            cur_flags = got.out_flags;
            // landing 单一真相源：查**目标层**的梯（witness.rs replay 同款）——
            // 上行落在目标层的下行梯（ty8）旁；传出发层会在出发层找不到 ty8 ⇒
            // 假 landing_missing（S5 真数据链跑实锤：f51→f52 卡死于此）。
            let tin = match levels.get(&target) {
                Some(i) => i,
                None => {
                    outcome = "model_missing".to_string();
                    fail_floor = Some(target);
                    fail_reason = Some(format!("f{} 无 LevelSpec", target));
                    break;
                }
            };
            let tlevel_t = match &tin.level {
                Some(l) => l,
                None => {
                    outcome = "model_missing".to_string();
                    fail_floor = Some(target);
                    fail_reason = Some(format!("f{} level=None", target));
                    break;
                }
            };
            let Some(landing) = crate::solve_global::landing(tlevel_t, target, went_down) else {
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
            }), end_ok, ending_done));
        }
        if end_ok {
            break; // 第一个到终局的候选即采纳（候选序确定性；计数已留档）
        }
    }

    // —— 汇总报告面 ——
    match best {
        Some((_ci, floors, steps, blocked, end_ok, ending_ok)) => {
            ow.floors = floors;
            ow.total_steps = steps;
            ow.final_state.floor = ow.floors.last().map_or(run.start.0, |f| f.floor);
            // 段分治：BossKill≠通关；ending_done 只在真 Ending（含 f50 段锚）时为真。
            ow.final_state.ending_done = end_ok && ending_ok;
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

// ---------------------------------------------------------------- S4-B：全程真引擎回放

/// 跨层回放的具名失败（帧定位 = (visit, floor, token_index)）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReplayFailure {
    pub visit: usize,
    pub floor: i32,
    pub token_index: usize,
    pub token: String,
    pub stage: String,
    pub reason: String,
}

fn kind_of(s: &str) -> Option<ExitKind> {
    match s {
        "Stair" => Some(ExitKind::Stair),
        "Script" => Some(ExitKind::Script),
        "Ending" => Some(ExitKind::Ending),
        "Cloud" => Some(ExitKind::Cloud),
        "UseItem" => Some(ExitKind::UseItem),
        "BossKill" => Some(ExitKind::BossKill),
        _ => None,
    }
}

/// 从 route 尾 token 推终态 pos（`x,y`/`bump ty@x,y`/`gate x,y` 的坐标语义都
/// 是「走完这一步之后的 pos」；`gatebump` 不移位 ⇒ 找最后一个非 gatebump token）。
fn route_end_pos(route: &[String]) -> Option<(i32, i32)> {
    for tok in route.iter().rev() {
        if let Some(rest) = tok.strip_prefix("bump ") {
            let (_, xy) = rest.split_once('@')?;
            return crate::solve_local::parse_xy_pub(xy);
        }
        if tok.starts_with("gatebump ") || tok == "gate" {
            continue;
        }
        if let Some(rest) = tok.strip_prefix("gate ") {
            return crate::solve_local::parse_xy_pub(rest);
        }
        if let Some(rest) = tok.strip_prefix("door ") {
            return crate::solve_local::parse_xy_pub(rest);
        }
        return crate::solve_local::parse_xy_pub(tok);
    }
    None
}

/// 【S4-B】把一段 witness 前缀在真引擎上逐层逐 token 回放（自证禁令的对面：
/// 每步走 `solve_local` 与 expand 同一条转移实现）。
///
/// - 入场实体：`ents_in = Some` ⇒ 原样（WEnt→EntRec）；`None` ⇒ 该层 `sprite_full`
///   全表（与求解侧展开同口径）；复访层用的是求解时记下的 `ents_out`（经 `ents_in`
///   回传——record_floor 复访时把上次 `ents_out` 写进新访问的 `ents_in`）。
/// - 换层 entry 复核：上一层 `kind=Stair` ⇒ 用 `witness::exit_edge`（stair_target
///   单一真相源）重推 `(target, went_down)` 并断言 `landing == 本层 entry`；
///   `Script/Cloud/UseItem` 的 `cross_edge` 不在 witness JSON 里（v3 schema 无此
///   字段）⇒ **如实跳过** entry 重推（记为已知缺口，findings 列明），路线本身
///   照常回放。
/// - 出层态全等断言：`out_player/out_flags/out_ents/pos` 逐字段 == 引擎回放终态。
pub fn replay_outer_witness(
    levels: &BTreeMap<i32, SolveInput>,
    start_entry: (i32, i32),
    shops: &[game_data::defs::ShopGoodDef],
    floors: &[crate::witness::WitnessFloor],
) -> Result<(), ReplayFailure> {
    let mk = |visit: usize, f: &crate::witness::WitnessFloor, stage: &str, reason: String, e: Option<&crate::solve_local::RouteReplayError>| ReplayFailure {
        visit,
        floor: f.floor,
        token_index: e.map_or(0, |x| x.token_index),
        token: e.map(|x| x.token.clone()).unwrap_or_default(),
        stage: stage.to_string(),
        reason: e.map(|x| x.reason.clone()).unwrap_or(reason),
    };
    for (i, f) in floors.iter().enumerate() {
        let Some(input) = levels.get(&f.floor) else {
            return Err(mk(i, f, "model_missing", format!("层 {} 无规格", f.floor), None));
        };
        let Some(level) = input.level.as_ref() else {
            return Err(mk(i, f, "level_missing", format!("层 {} level=None", f.floor), None));
        };
        // 入场格：首访问=run 起点；否则=上一访问换层 landing 复核结果。
        let entry = if i == 0 {
            start_entry
        } else {
            let prev = &floors[i - 1];
            let Some(pinput) = levels.get(&prev.floor) else {
                return Err(mk(i, f, "model_missing", format!("层 {} 无规格", prev.floor), None));
            };
            let Some(plevel) = pinput.level.as_ref() else {
                return Err(mk(i, f, "level_missing", format!("层 {} level=None", prev.floor), None));
            };
            let Some(pk) = kind_of(&prev.kind) else {
                return Err(mk(i, f, "kind_unknown", format!("未知 kind {}", prev.kind), None));
            };
            // Fabricate minimal ExitSolve 让 witness::exit_edge 单一真相源重推换层边。
            let fake = crate::solve_local::ExitSolve {
                ok: true,
                exit: prev.exit.map(|e| (e[0], e[1])),
                out_player: crate::solve_local::PlayerSpec {
                    hp: 0, atk: 0, def: 0, yellow: 0, blue: 0, red: 0, gold: 0,
                    item_types: Vec::new(), item_uses: Vec::new(), item_stack_size: 0,
                },
                out_flags: 0,
                out_ents: Vec::new(),
                route: Vec::new(),
                out_hp: 0,
                kind: Some(pk),
                cross_edge: None,
                truncated: false,
                pure_pickup: false,
                pure_pickup_reason: None,
                key_floor_not_consulted: None,
            };
            if let Some((target, went_down)) = crate::witness::exit_edge(plevel, &fake) {
                // landing 查的是**目标层**的落点梯（与 driver 缺口3 同口径；
                // 旧版传出发层 plevel ⇒ 目标层无相应梯时假 entry_mismatch）。
                let tlevel = levels
                    .get(&target)
                    .and_then(|t| t.level.as_ref())
                    .unwrap_or(plevel);
                let land = crate::solve_global::landing(tlevel, target, went_down);
                if land != Some((f.entry[0], f.entry[1])) {
                    return Err(mk(
                        i,
                        f,
                        "entry_mismatch",
                        format!(
                            "换层落点不符：landing(target={target}, went_down={went_down})={land:?} != 记录 entry {:?}",
                            f.entry,
                            target = target,
                            went_down = went_down,
                            land = land
                        ),
                        None,
                    ));
                }
            }
            // Stair 以外的 kind：cross_edge 未序列化 ⇒ 跳过 entry 重推（已知缺口）。
            (f.entry[0], f.entry[1])
        };
        // 入场实体表：Some=原样；None=sprite_full（求解侧同口径）。
        let ents: Vec<EntRec> = match &f.ents_in {
            Some(ws) => ws.iter().map(|w| EntRec { ty: w.ty, x: w.x, y: w.y, param: w.param }).collect(),
            None => crate::solve_local::sprite_full(level),
        };
        // 出层 pos：exit 优先，缺（Ending）则取 route 尾 token 坐标。
        let end_pos: Option<(i32, i32)> = f.exit.map(|e| (e[0], e[1])).or_else(|| route_end_pos(&f.route));
        let Some(end_pos) = end_pos else {
            return Err(mk(i, f, "pos_unknown", "exit=None 且 route 尾无坐标 token".to_string(), None));
        };
        let w2p = |w: &crate::witness::WPlayer| crate::solve_local::PlayerSpec {
            hp: w.hp, atk: w.atk, def: w.def, yellow: w.yellow, blue: w.blue, red: w.red, gold: w.gold,
            item_types: w.item_types.clone(), item_uses: w.item_uses.clone(), item_stack_size: w.item_stack_size,
        };
        let h2f = |s: &str| -> Result<u128, ReplayFailure> {
            u128::from_str_radix(s.trim_start_matches("0x"), 16).map_err(|e| ReplayFailure {
                visit: i, floor: f.floor, token_index: 0, token: String::new(),
                stage: "flags_hex".to_string(), reason: format!("flags 非法 hex {s:?}: {e}"),
            })
        };
        let in_flags = match h2f(&f.in_flags) { Ok(v) => v, Err(e) => return Err(e) };
        let out_flags = match h2f(&f.out_flags) { Ok(v) => v, Err(e) => return Err(e) };
        let out_ents: Vec<EntRec> = f.ents_out.iter().map(|w| EntRec { ty: w.ty, x: w.x, y: w.y, param: w.param }).collect();
        let got = crate::solve_local::replay_route_expect(
            input, entry, &ents, &w2p(&f.in_player), in_flags, &f.route, shops, &w2p(&f.out_player), out_flags, &out_ents, end_pos,
        );
        if let Err(e) = got {
            return Err(mk(i, f, "route", String::new(), Some(&e)));
        }
    }
    Ok(())
}
