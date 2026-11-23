//! S5 机制2：boss 锚段链分治（`docs/findings/2026-11-23-s4-inner-outer-integration.md`
//! 复核黄3：全题外层 5M pops blocked、20M pops 35min 超时——指数空间按 boss 锚分段，
//! 每段独立收敛后拼成全程 witness）。
//!
//! 分段（bossTypeOrder={75,74,68,67,69,71}，楼层普查 75→f10/74→f20/68→f25/67→f15/69→f35/71→f53）：
//! - seg1 f51..54+1..10，锚 boss75@f10（任务书原始建议 68@f25 作 seg3 锚——**更正**：
//!   段锚必须是段的收尾保证层覆盖；f25/68 是 seg3 段内小 boss，seg3 锚定为 69@f35）。
//! - seg2 f11..20，锚 boss74@f20；seg3 f21..35，锚 boss69@f35；seg4 f36..50，锚=ending@f50
//!   （floor50 击败型72 直走引擎终局边，段末判据 Ending，见 outer_witness boss_mode）。
//!
//! 跨段接力：成功段的末层 `out_player`/`out_flags` 直搬下段（单一真相源：外层起点态 =
//! [`outer::outer_state_from_player`]，内层入口态 = witness 驱动器实参，回放 = replay 直读
//! witness floors 的 in/out——三处同源不各自造转换）。下段入口格由
//! [`crate::solve_global::landing`] 推（与 replay 同口径）。
//!
//! 诚实口径：段 blocked ⇒ 停链、拼已得段前缀、`blocked` 具名（seg/reason/detail）；
//! 不许抬预算换有解；回放 [`crate::outer_witness::replay_outer_witness`] 失败即如实记录。

use crate::outer::{
    BlockInterfaces, OuterError, OuterGraph, OuterState, RunSpec, SearchBudget,
};
use crate::outer_witness::{OuterLayerSolver, OuterWitness, OuterWitnessCfg, run_outer_witness_with};
use game_data::defs::ShopGoodDef;
use crate::solve_local::{EntRec, PlayerSpec, SolveInput};
use crate::witness::WitnessFloor;
use std::collections::BTreeMap;

pub const OUTER_CHAIN_SCHEMA: &str = "xiyou/outer-chain/v1";

/// 段规格：`start=None` ⇒ 由上段末层 landing 推（seg1 必须显式给）。
#[derive(Debug, Clone)]
pub struct ChainSegSpec {
    pub name: String,
    pub floors: Vec<i32>,
    pub start: Option<(i32, (i32, i32))>,
    pub anchor_floor: i32,
    /// 锚 boss ty；f50 终局段为 72（Ending 判据由 outer_witness boss_mode 兼容）。
    pub anchor_boss_ty: i32,
}

/// 四段主线（见模块 doc 的分段更正注记）。
pub fn mainline_segments() -> Vec<ChainSegSpec> {
    let mut s1 = vec![51, 52, 53, 54];
    s1.extend(1..=10);
    let s2 = 11..=20;
    let s3 = 21..=35;
    let s4 = 36..=50;
    vec![
        ChainSegSpec {
            name: "seg1_f51-f54_f1-f10".into(),
            floors: s1,
            start: Some((51, (1, 11))),
            anchor_floor: 10,
            anchor_boss_ty: 75,
        },
        ChainSegSpec {
            name: "seg2_f11-f20".into(),
            floors: s2.collect(),
            start: None,
            anchor_floor: 20,
            anchor_boss_ty: 74,
        },
        ChainSegSpec {
            name: "seg3_f21-f35".into(),
            floors: s3.collect(),
            start: None,
            anchor_floor: 35,
            anchor_boss_ty: 69,
        },
        ChainSegSpec {
            name: "seg4_f36-f50".into(),
            floors: s4.collect(),
            start: None,
            anchor_floor: 50,
            anchor_boss_ty: 72, // floor50 击败型72 → Ending（引擎终局边）
        },
    ]
}

/// 链配置（每段同预算；禁抬 cap 换有解）。
#[derive(Debug, Clone)]
pub struct ChainCfg {
    pub max_pops: u64,
    pub candidates: usize,
    pub max_floors: usize,
}

impl Default for ChainCfg {
    fn default() -> Self {
        ChainCfg { max_pops: 20_000_000, candidates: 5, max_floors: crate::witness::DEFAULT_MAX_FLOORS }
    }
}

/// 单段结果（witness 全文内嵌——链报告即完整交接证据）。
#[derive(Debug, serde::Serialize)]
pub struct SegReport {
    pub name: String,
    /// 同 `OuterWitness.status` 词表（witness/blocked/capped）。
    pub status: String,
    pub pops: u64,
    pub truncated: bool,
    pub frontier_len: usize,
    pub killable: usize,
    pub candidates_considered: usize,
    pub floors_solved: usize,
    /// 与上段末层的逐字段接力比对（首段 `null`）。
    pub handoff_ok: Option<bool>,
    pub blocked: Option<crate::witness::WitnessBlocked>,
    pub witness: OuterWitness,
}

/// 具名断点（段 blocked 时停链）。
#[derive(Debug, serde::Serialize)]
pub struct BlockedSeg {
    pub seg: String,
    pub reason: String,
    pub detail: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ChainReport {
    pub schema: String,
    /// `witness`（全程拼接成功）| `blocked`（具名断点）。
    pub status: String,
    pub segments: Vec<SegReport>,
    /// 各段 witness floors 顺序拼接（含 in/out 玩家与 flags 接力证据）。
    pub floors: Vec<WitnessFloor>,
    pub total_steps: i32,
    /// 仅全链以真 Ending 收尾时为真（BossKill 段末不冒充通关）。
    pub ending_done: bool,
    pub blocked: Option<BlockedSeg>,
    pub total_pops: u64,
    /// 拼接 floors 的 [`crate::outer_witness::replay_outer_witness`] 自证：
    /// `Some("pass")` 或失败描述；`None`=无层可回放。
    pub replay: Option<String>,
}

/// WPlayer → PlayerSpec（与 replay_outer_witness 同一套字段直搬，禁第二份语义）。
fn wplayer_to_spec(w: &crate::witness::WPlayer) -> PlayerSpec {
    PlayerSpec {
        hp: w.hp,
        atk: w.atk,
        def: w.def,
        yellow: w.yellow,
        blue: w.blue,
        red: w.red,
        gold: w.gold,
        item_types: w.item_types.clone(),
        item_uses: w.item_uses.clone(),
        item_stack_size: w.item_stack_size,
    }
}

fn flags_parse(hex: &str) -> u128 {
    u128::from_str_radix(hex.trim_start_matches("0x"), 16).unwrap_or_else(|e| {
        panic!("flags hex 解析失败 {hex:?}: {e}（witness 落账即 hex，坏账=具名失败）")
    })
}

/// 外层起点态：携带入场态（上段 out_player 直搬）。
/// held=item_types 位图（与 `OuterState.held` 同语义）；picked 全零（新段拾取史；
/// 回拾互斥由段层集 partition 保证——段间不共层，picked 不会冲突）；buys=0（段内计数）。
pub fn outer_state_from_player(
    graph: &OuterGraph,
    run: &RunSpec,
    player: &PlayerSpec,
) -> Result<OuterState, OuterError> {
    let node = graph.start_node(run.start.0, run.start.1)?;
    let mut held: u128 = 0;
    for t in &player.item_types {
        held |= 1u128 << (*t as u128);
    }
    Ok(OuterState {
        node,
        yellow: player.yellow,
        red: player.red,
        blue: player.blue,
        hp: player.hp,
        gold: player.gold,
        atk: player.atk,
        def: player.def,
        held,
        picked: vec![0u8; (graph.floor_of.len() + 7) / 8],
        buys: 0,
    })
}

/// 逐字段接力比对：上段末层 out_* vs 下段首层 in_*。
fn handoff_matches(prev: &WitnessFloor, next: &WitnessFloor) -> bool {
    let (a, b) = (&prev.out_player, &next.in_player);
    a.hp == b.hp
        && a.atk == b.atk
        && a.def == b.def
        && a.yellow == b.yellow
        && a.blue == b.blue
        && a.red == b.red
        && a.gold == b.gold
        && a.item_types == b.item_types
        && a.item_uses == b.item_uses
        && a.item_stack_size == b.item_stack_size
        && prev.out_flags == next.in_flags
}

/// 默认层求解闭包：toward=Some → 普通出层解；None → 段锚 boss 解
/// （`solve_local_boss_toward_shops`，要求 BossKill|f50 Ending，不回退普通出层解）。
pub fn default_layer_solver<'a>(
    segs: &'a [ChainSegSpec],
) -> impl Fn(&SolveInput, (i32, i32), &PlayerSpec, u128, Option<&[EntRec]>, Option<i32>, &[ShopGoodDef]) -> (
    crate::solve_local::ExitSolve,
    bool,
) + 'a {
    move |input, entry, player, flags, ents, toward, shops| {
        match toward {
            Some(f) => crate::solve_local::solve_local_exit_toward_shops(
                input, entry, player, flags, ents, Some(f), shops,
            ),
            None => {
                let seg = segs
                    .iter()
                    .find(|s| s.floors.contains(&input.floor))
                    .unwrap_or_else(|| {
                        panic!("floor {} 不在任何段层集（链层集 partition 被破坏）", input.floor)
                    });
                crate::solve_local::solve_local_boss_toward_shops(
                    input, entry, player, flags, ents, seg.anchor_boss_ty, shops,
                )
            }
        }
    }
}

/// 段链驱动（注入层版本，测试用；生产走 [`run_chain`]）。
pub fn run_chain_with(
    doc: &BlockInterfaces,
    levels: &BTreeMap<i32, SolveInput>,
    scaled_enemies: &[game_data::defs::EnemyDef],
    shops: &[ShopGoodDef],
    player0: &PlayerSpec,
    flags0: u128,
    segs: &[ChainSegSpec],
    cfg: &ChainCfg,
    layer: OuterLayerSolver<'_>,
) -> ChainReport {
    let mut segments: Vec<SegReport> = Vec::new();
    let mut floors: Vec<WitnessFloor> = Vec::new();
    let mut total_pops: u64 = 0;
    let mut total_steps: i32 = 0;
    let mut blocked: Option<BlockedSeg> = None;
    let mut carried_player = player0.clone();
    let mut carried_flags = flags0;

    for (si, seg) in segs.iter().enumerate() {
        let chain_alive = blocked.is_none(); // 断链后：后续段仍跑（具名报告），但不并入拼接、不接力
        // 入口格：seg1 显式；后续段由上段末层 landing 推（与 replay 同口径）。
        let start_cell: (i32, i32) = if si == 0 {
            seg.start.map(|(_, c)| c).unwrap_or_else(|| panic!("首段必须显式 start"))
        } else {
            let prev_seg = &segs[si - 1];
            let prev_floor = prev_seg.floors[prev_seg.floors.len() - 1];
            let next_first = seg.floors[0];
            // went_down=下行梯(ty8)语义（stair_target：ty==8）；主线段间全上行 ⇒ false。
            // 落点梯在**目标层**（上行落在目标层的下行梯旁）——与 witness.rs:707 同源。
            let went_down = next_first < prev_floor;
            let tlevel = levels
                .get(&next_first)
                .and_then(|inp| inp.level.as_ref())
                .unwrap_or_else(|| panic!("levels 缺下段首层 f{next_first}（层集不完整）"));
            match crate::solve_global::landing(tlevel, next_first, went_down) {
                Some(c) => c,
                None => {
                    blocked = Some(BlockedSeg {
                        seg: seg.name.clone(),
                        reason: "landing_missing".into(),
                        detail: format!(
                            "上段末层 f{prev_floor} → f{next_first} 的落点格推不出（landing=None）"
                        ),
                    });
                    continue;
                }
            }
        };
        let run = RunSpec {
            floors: seg.floors.clone(),
            start: (seg.floors[0], start_cell),
            anchor_floor: seg.anchor_floor,
            anchor_boss_ty: seg.anchor_boss_ty,
            masked_exits: Vec::new(),
            combat_cost: true,
        };
        let graph = OuterGraph::build(doc, &run.floors, &run.masked_exits);
        // 携带入场态起搜（机制2 核心：跨段状态不断链）。
        let start_state = match outer_state_from_player(&graph, &run, &carried_player) {
            Ok(s) => s,
            Err(e) => {
                blocked = Some(BlockedSeg {
                    seg: seg.name.clone(),
                    reason: "start_state_missing".into(),
                    detail: format!("外层起点态构造失败: {e}"),
                });
                continue;
            }
        };
        let search = crate::outer::outer_search_with_start(
            &graph,
            &run,
            scaled_enemies,
            SearchBudget { max_pops: cfg.max_pops },
            start_state,
        );
        let killable =
            search.frontier.iter().filter(|c| !c.fatal).count();
        let wcfg = OuterWitnessCfg {
            candidates: cfg.candidates,
            max_floors: cfg.max_floors,
            boss_target: Some(seg.anchor_boss_ty),
        };
        let ow = run_outer_witness_with(
            &graph,
            &run,
            scaled_enemies,
            levels,
            carried_player.clone(),
            carried_flags,
            shops,
            &search,
            &wcfg,
            layer,
        );
        total_pops += ow.outer.pops;
        let floors_solved = ow.floors.len();
        let seg_ok = ow.status == "witness";
        if seg_ok {
            total_steps += ow.total_steps;
        }
        // 接力比对：上段末层 out_* vs 本段首层 in_*（逐字段，含 flags hex）。
        let handoff = if seg_ok && chain_alive && !floors.is_empty() && !ow.floors.is_empty() {
            let ok = handoff_matches(floors.last().unwrap(), &ow.floors[0]);
            if !ok {
                blocked = Some(BlockedSeg {
                    seg: seg.name.clone(),
                    reason: "handoff_mismatch".into(),
                    detail: "上段末层 out_player/out_flags 与本段首层 in_player/in_flags 不一致"
                        .into(),
                });
            }
            Some(ok)
        } else {
            None // 首段无上段可接力；段未收敛无从比对
        };
        let rep = SegReport {
            name: seg.name.clone(),
            status: ow.status.clone(),
            pops: ow.outer.pops,
            truncated: ow.outer.truncated,
            frontier_len: ow.outer.frontier_len,
            killable,
            candidates_considered: ow.candidates_considered,
            floors_solved,
            handoff_ok: handoff,
            blocked: ow.blocked.clone(),
            witness: ow,
        };
        if seg_ok && chain_alive {
            let last = rep.witness.floors.last().unwrap().clone();
            carried_player = wplayer_to_spec(&last.out_player);
            carried_flags = flags_parse(&last.out_flags);
            floors.extend(rep.witness.floors.iter().cloned());
        }
        segments.push(rep);
        if !seg_ok && blocked.is_none() {
            let b = segments.last().unwrap().blocked.clone().unwrap_or_else(|| {
                crate::witness::WitnessBlocked {
                    floor: seg.anchor_floor,
                    reason: segments.last().unwrap().status.clone(),
                    detail: "段未收敛且无具名 blocked（驱动器口径缺口，如实上报）".into(),
                    layer_ok: None,
                    layer_truncated: None,
                }
            });
            blocked = Some(BlockedSeg {
                seg: seg.name.clone(),
                reason: b.reason,
                detail: format!("floor={} {}", b.floor, b.detail),
            });
        }
    }

    // 拼接回放自证（复核黄2 首验：真 witness 端到端回放）。
    let replay = if floors.is_empty() {
        None
    } else {
        let first_entry = segs[0].start.map(|(_, c)| c).unwrap_or((1, 11));
        Some(
            match crate::outer_witness::replay_outer_witness(levels, first_entry, shops, &floors) {
                Ok(()) => "pass".to_string(),
                Err(e) => format!(
                    "replay_failed: visit={} floor={} token_index={} token={:?} stage={} reason={}",
                    e.visit, e.floor, e.token_index, e.token, e.stage, e.reason
                ),
            },
        )
    };
    let status = if blocked.is_none() && !segments.is_empty() { "witness" } else { "blocked" };
    let ending_done = blocked.is_none()
        && segments.last().map_or(false, |s| s.witness.final_state.ending_done);
    ChainReport {
        schema: OUTER_CHAIN_SCHEMA.to_string(),
        status: status.to_string(),
        segments,
        floors,
        total_steps,
        ending_done,
        blocked,
        total_pops,
        replay,
    }
}

/// 生产入口：默认层求解闭包。
pub fn run_chain(
    doc: &BlockInterfaces,
    levels: &BTreeMap<i32, SolveInput>,
    scaled_enemies: &[game_data::defs::EnemyDef],
    shops: &[ShopGoodDef],
    player0: &PlayerSpec,
    flags0: u128,
    segs: &[ChainSegSpec],
    cfg: &ChainCfg,
) -> ChainReport {
    let layer = default_layer_solver(segs);
    run_chain_with(doc, levels, scaled_enemies, shops, player0, flags0, segs, cfg, &layer)
}
