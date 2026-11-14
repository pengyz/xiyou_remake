//! 【求解器批3 C2】全局分层求解（计划 §4 批3 ①②④；依赖批2 单层基座）。
//!
//! 形态（计划 §3.2 采纳口径）：**层间图节点=(floor, 耦合态)，边=整层穿越**——
//! 每次出边调 `solve_layer` 求「入场格 → 本层出口」的 Pareto 选项，全局层只做
//! 传播 + K₀ 去重 + Pareto + 有界 + 确定序。判据「逐层结果拼接==全局解」
//! （计划 :341）由 `solve_layer` 的单层腿与本模块的拼接口径天然对齐。
//!
//! 批3 C2 已建边：**楼梯边**（Exit#1：出口梯格→`param&0xff` 目标层，`swap_floor`
//! 落点语义）与**型72 终局边**（floor50 击败型72 → `ExitKind::Ending`=通关）。
//! C3 起补：StepGate script 边（`pending_floor`/`went_down`）、筋斗云邻梯瞬移边
//! （探索区间门 4 分量 canonical 实作：tutorialSeen 背包权威源、f_bool_13 逐 tick 派生几何、findFloorGateEntity 层数据，由 solve_local.rs:1372-1399 cloud_goals 派生；changeFloor min/max 已归 st.floors，本文件 :383-389 落闸——均不入 flags（裁定见批3 复核报告 §9.4））、battle-victory 脚本 69/70/71/73/74/75 转移边。
//!
//! 确定性（计划 §4 ④）：遍历全走 `BTreeMap`/`BinaryHeap`，无 HashMap 迭代序依赖；
//! 同输入同解由批2 同款双跑测试钉死（C4）。

use crate::solve_local::{
    solve_layer, sprite_full, take_layer_trunc, EntRec, LevelSpec, PlayerSpec, SolveInput,
};
use crate::state::{CanonicalBag, DiscreteGold, DiscreteHp, ScriptFlags, SolverState};
use game_core::subsystem::script_host::entity_category_with_floor;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, BTreeMap, VecDeque};

/// 全局求解输入。
#[derive(Debug, Clone)]
pub struct GlobalInput {
    /// 层规格（`floor 号 → 单层夹具`）；缺层的换层边=不扩展（C5 CLI 提供全 54 层）
    pub floors: BTreeMap<i32, LevelSpec>,
    /// 起始层（真数据=51，计划 R10 S₀）
    pub start_floor: i32,
    /// 起始格（真数据=(1,11)，`handoff.rs:154` `load_floor(32,352)` ⇒ (1,11)）
    pub start_pos: (i32, i32),
    /// 起始玩家状态（真数据=`state.rs s0()`）
    pub start_player: PlayerSpec,
    /// 起始 `script_line_flags`（真数据=0）
    pub start_flags: u128,
    /// 扩展界（判据 `--bound 1e6`；§4 ③ 界可配、解标注三态）
    pub bound: u64,
}

/// 全局目标（通关解；`post`=通关瞬间玩家全状态）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalGoal {
    /// 通关 hp（Q1 目标面）
    pub hp: i32,
    /// 全程击杀（Q2 目标面）
    pub kills: i32,
    /// 全程步数（tie-break 面）
    pub steps: i32,
    /// 全程路线（逐层 route_of 拼接，批2 token 语法）
    pub route: Vec<String>,
    /// 访问层序列（样例 JSON `floors` 字段，计划 §7 :469）
    pub floors: Vec<i32>,
    /// 通关瞬间玩家状态（hp/atk/def/三色钥/金币/背包）
    pub post: PlayerSpec,
    /// 各层终态活体实体列表（C6a A-lite；`monsters_alive` 全局口径取材面，C4/C5）
    pub ents: BTreeMap<i32, Vec<EntRec>>,
    /// 终态 `script_line_flags` 位图（`to_solver_state` 用；A11 checkpoint 预留）
    pub flags: u128,
    /// 存活怪物数（C4 全局口径：已访层位图∩cat8 + 未访层 sprite 全活 cat8）
    pub monsters_alive: u32,
}

/// 全局求解结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalResult {
    /// 通关目标（非支配；空=无解）
    pub goals: Vec<GlobalGoal>,
    /// 三态置信（裁定⑤）：队列耗尽且有目标=`optimal`、打界有目标=`bounded-best`、
    /// 无目标=`feasible`（与批2 `solve_local` 占位/不可达口径同源，solve_local.rs:246）
    pub confidence: &'static str,
    /// 扩展界（回显）
    pub bound: u64,
    /// 实际扩展（出队处理）节点数
    pub expansions: u64,
    /// R1 层界截断记录（lead 裁定 A 条件1）：`(floor, 截断时 nodes 实数)`——
    /// 非空 ⇒ 该结果被层界污染 ⇒ confidence 已降级（不标 optimal）。
    pub truncated_layers: Vec<(i32, usize)>,
    /// C6h：本次求解的全局节点上限（env SOLVER_NODE_LIMIT 或显式入参）。
    pub node_cap: u64,
    /// C6h：nodes.len() 达上限提前终止（可观测；confidence 已降级）。
    pub node_cap_hit: bool,
}

/// 全局态（层间耦合量，计划 §3.3 字段口径；`floor/pos/player/flags/kills/steps`
/// +逐层存活位图 + 访问层序列）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct GSt {
    floor: i32,
    pos: (i32, i32),
    player: PlayerSpec,
    flags: u128,
    /// 全程累计击杀（含已穿层）
    kills: i32,
    /// 全程累计步数（含已穿层）
    steps: i32,
    /// 已访层活体实体列表（C6a A-lite；缺席层=sprite 全活=首次进入；
    /// save_floor_state 语义：已死怪/已开门跨层持久，`scene_persist.rs:398`）
    ents: BTreeMap<i32, Vec<EntRec>>,
    /// 访问层序列（含重复访问）
    floors: Vec<i32>,
}

/// 全局去重键 K₀′（批2 K₀ 的层间延展；裁定② **gold 进键**——批2 伏笔「批3 商店层
/// 再入键」：不同金币=不同商店购买能力，不可合并）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GKey {
    floor: i32,
    pos: (i32, i32),
    flags: u128,
    keys: (i32, i32, i32),
    /// canonical 背包（size 前缀截断，`state::CanonicalBag`——裁定②同键面）
    bag: CanonicalBag,
    atk: i32,
    def: i32,
    gold: i32,
    /// 逐层实体列表展平（C6a：BTreeMap 层序 + 引擎槽序；`floor i32 + len u16 +
    /// 每条 ty/x/y/param 各 u16`；缺席层不出现——与原位图版同为确定性线形序列）
    ents_flat: Vec<u8>,
}

fn key_of(st: &GSt) -> GKey {
    let mut ents_flat = Vec::new();
    for (f, recs) in &st.ents {
        ents_flat.extend_from_slice(&f.to_le_bytes());
        ents_flat.extend_from_slice(&(recs.len() as u16).to_le_bytes());
        for r in recs {
            ents_flat.extend_from_slice(&(r.ty as u16).to_le_bytes());
            ents_flat.extend_from_slice(&(r.x as u16).to_le_bytes());
            ents_flat.extend_from_slice(&(r.y as u16).to_le_bytes());
            ents_flat.extend_from_slice(&(r.param as u16).to_le_bytes());
        }
    }
    GKey {
        floor: st.floor,
        pos: st.pos,
        flags: st.flags,
        keys: (
            st.player.yellow,
            st.player.blue,
            st.player.red,
        ),
        bag: CanonicalBag::from_player(&st.player),
        atk: st.player.atk,
        def: st.player.def,
        gold: st.player.gold,
        ents_flat,
    }
}

/// Pareto 支配（批2 同款方向：hp 越大、kills/steps 越小越优；`dominates(a,b)`=
/// a 支配 b ⇒ b 可丢）。**相等不算严格支配**——防等价环由 push 侧「已被支配或
/// 相等⇒丢」处理（落点邻格是梯会 bounce 再出，零步等价环见 `landing` 注）。
fn dominates(a: &GSt, b: &GSt) -> bool {
    let hp_ge = a.player.hp >= b.player.hp;
    let kills_le = a.kills <= b.kills;
    let steps_le = a.steps <= b.steps;
    let strict = a.player.hp > b.player.hp || a.kills < b.kills || a.steps < b.steps;
    hp_ge && kills_le && steps_le && strict
}

/// 同 K₀ **弱支配**（含指标全等）：a 全面不劣于 b ⇒ b 冗余。
/// 严格版 `dominates` 会漏掉「指标全等、仅 `floors` 史不同」的筋斗云零步等价环
/// （C3b T1 实测 `6,7↔2,0` 循环打爆 bound）；同 K₀ ⇒ 同已访层集（`ents_flat`
/// 含全部已访层位图）⇒ 区间门同域，弱支配丢弃安全。
fn weakly_ge(a: &GSt, b: &GSt) -> bool {
    a.player.hp >= b.player.hp && a.kills <= b.kills && a.steps <= b.steps
}

/// 全局节点（父指针+入边段：route 长镜像按段拼接，避免逐节点全长 route 内存爆）。
struct GNode {
    st: GSt,
    parent: u32,
    /// 根到本节点的换层深度（根=0；C5d 最深优先遍历主键）
    depth: u32,
    /// 入边段=产生本节点的 `LayerGoal.route`（根=空）
    segment: Vec<String>,
}

/// 楼梯换层落点（B 级复刻=`scene_persist.rs swap_floor :410-433` 特例链 +
/// `scene.rs step_off_stairs :772-785`；两者均 `pub(super)` 不可直调 ⇒ 复刻报审注明，
/// 非战斗/脚本/路径面——保真红线直调面不涉）。
///
/// - `target==0` → `(1,2)`；`target==50` → `(6,7)`；`target==1 && !went_down` → `(6,11)`；
/// - 否则目标层找**首个** `param>>9==0` 的梯（`want = went_down ? 7 : 8`：
///   下行落层找上梯、上行找下梯，`find_floor_gate :435`）→ 落点=首个可行走邻格
///   （上/下/左/右，四邻皆不可行留梯格自身）；
/// - 目标层无梯 ⇒ `None`=丢边（引擎=留旧坐标异常态，报审注明）。
fn landing(level: &LevelSpec, target: i32, went_down: bool) -> Option<(i32, i32)> {
    if target == 0 {
        return Some((1, 2));
    }
    if target == 50 {
        return Some((6, 7));
    }
    if target == 1 && !went_down {
        return Some((6, 11));
    }
    let want = if went_down { 7 } else { 8 };
    let gate = level
        .entities
        .iter()
        .find(|e| e.ty == want && (e.param >> 9) == 0)?;
    let (mut gx, mut gy) = (gate.x, gate.y);
    // 可行走=地形单源（批2 `walkable` 同判据；引擎 `walkable_at` 另含 grid 的
    // 差异=报审注明）
    let walk = |x: i32, y: i32| {
        x >= 0 && y >= 0 && x < level.cols && y < level.rows
            && level.terrain[(y * level.cols + x) as usize]
    };
    if walk(gx, gy - 1) {
        gy -= 1;
    } else if walk(gx, gy + 1) {
        gy += 1;
    } else if walk(gx - 1, gy) {
        gx -= 1;
    } else if walk(gx + 1, gy) {
        gx += 1;
    }
    Some((gx, gy))
}

/// 从入口格看，出口梯的目标层（`load_floor` 语义：type7|8 ⇒ `ExitSpec{Stair}`，
/// 实体带 `param`；`param&0xff`=目标层号，a.java:5638-5664）。
fn stair_target(level: &LevelSpec, exit: (i32, i32)) -> Option<(i32, bool)> {
    let e = level
        .entities
        .iter()
        .find(|e| matches!(e.ty, 7 | 8) && (e.x, e.y) == exit)?;
    Some((e.param & 0xff, e.ty == 8))
}

/// 全局 route_of（父指针镜像，批2 `route_of` :1120 同款；段=入边 LayerGoal.route）。
fn route_of_global(nodes: &[GNode], id: u32) -> Vec<String> {
    let mut segs: Vec<&[String]> = Vec::new();
    let mut cur = id;
    while cur != u32::MAX {
        segs.push(&nodes[cur as usize].segment);
        cur = nodes[cur as usize].parent;
    }
    segs.reverse();
    segs.into_iter().flat_map(|s| s.iter().cloned()).collect()
}

/// 全局分层求解入口（计划 §4 批3 判据 `solve-global --bound`）。
pub fn solve_global(input: &GlobalInput) -> GlobalResult {
    solve_global_capped(input, node_cap_env())
}

/// C6h：全局节点上限 env（SOLVER_NODE_LIMIT，默认 **100_000**）——lead 裁定 A 同族
/// （R1 有界可配）：bound1e6 双跑曾 OOM 双杀（exit137，12-15min、31GB 机），节点
/// 上限让判据③ 命令可完成且确定性可复跑。默认值实测标定：gauge 50_000→2m26s
/// （exp17532、~120 exp/s）⇒ 100_000≈4-5min/跑、双跑≈10min、内存 ~1-2GB；
/// env 可调（求解深度优先场景调大），报审记录该口径。
fn node_cap_env() -> u64 {
    std::env::var("SOLVER_NODE_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100_000)
}

/// C6h：显式节点上限求解（触界 ⇒ node_cap_hit=true、confidence 降级）。
pub fn solve_global_capped(input: &GlobalInput, node_cap: u64) -> GlobalResult {
    // 每层 SolveInput 预构造（difficulty=0=BR-4；LevelSpec 每层只 clone 一次）
    let mut per_floor: BTreeMap<i32, SolveInput> = BTreeMap::new();
    for (f, spec) in &input.floors {
        per_floor.insert(
            *f,
            SolveInput {
                floor: *f,
                difficulty: 0,
                level: Some(spec.clone()),
            },
        );
    }

    let mut start_ents = BTreeMap::new();
    start_ents.insert(
        input.start_floor,
        input
            .floors
            .get(&input.start_floor)
            .map(sprite_full)
            .unwrap_or_default(),
    );
    let root = GSt {
        floor: input.start_floor,
        pos: input.start_pos,
        player: input.start_player.clone(),
        flags: input.start_flags,
        kills: 0,
        steps: 0,
        ents: start_ents,
        floors: vec![input.start_floor],
    };
    let mut nodes: Vec<GNode> = vec![GNode {
        st: root,
        parent: u32::MAX,
        depth: 0,
        segment: Vec::new(),
    }];
    let mut pruned = vec![false];
    // 遍历序（C6c 终版）=**层链距目标升序**（goal-directed）：反向 BFS 自目标层
    // 50 沿楼梯 param 边+符21/22 ±1 边得 floor_dist，主键 dist 升序、tiebreak=
    // depth 降序、node id 升序=全序确定性。教训链：BFS FIFO 浅层扇出淹没（queue
    // 80→15.5k 恒 [51,52,53]，b41）→最深优先深徘徊（maxdepth 卡133 goals0、
    // f7/f8 内部长走态占满带宽，今日实测）→层链距直达走廊 f51→52→53→54→1→…
    // →23→24→50。**纯排序变换**：K₀/去重/bound/结果集全不变、同 env 同序。
    let floor_dist = floor_chain_dist(&input.floors);
    let dist_root = floor_dist
        .get(&input.start_floor)
        .copied()
        .unwrap_or(u32::MAX);
    let mut queue: BinaryHeap<(Reverse<u32>, u32, Reverse<u32>)> = BinaryHeap::new();
    queue.push((Reverse(dist_root), 0, Reverse(0)));
    let mut by_key: BTreeMap<GKey, Vec<u32>> = BTreeMap::new();
    by_key.entry(key_of(&nodes[0].st)).or_default().push(0);
    let mut goals: Vec<GlobalGoal> = Vec::new();
    let mut expansions: u64 = 0;
    let mut hit_bound = false;
    let mut node_cap_hit = false;
    let mut truncated_layers: Vec<(i32, usize)> = Vec::new();
    while let Some((Reverse(_), _, Reverse(id))) = queue.pop() {
        if pruned[id as usize] {
            continue;
        }
        let Some(sinput) = per_floor.get(&nodes[id as usize].st.floor) else {
            // 缺层规格 ⇒ 该态无边（C5 CLI 提供全层；玩具缺层=不可扩展）
            continue;
        };
        if expansions >= input.bound {
            hit_bound = true;
            break;
        }
        if nodes.len() as u64 >= node_cap {
            // C6h：全局节点上限（arena 内存+运行时双守卫；确定性触发）
            node_cap_hit = true;
            break;
        }
        expansions += 1;
        let st = nodes[id as usize].st.clone();
        let level = sinput.level.as_ref().expect("per_floor 已填 Some");
        let ents_here = st.ents.get(&st.floor).cloned();
        let (reachable, layer_goals) = solve_layer(
            sinput,
            st.pos,
            ents_here.as_deref(),
            st.player.clone(),
            st.flags,
        );
        if let Some(t) = take_layer_trunc() {
            if !truncated_layers.contains(&t) {
                truncated_layers.push(t);
            }
        }
        if !reachable {
            continue;
        }
        for g in layer_goals {
            // 终态位图：本层 post 覆写
            let mut ents = st.ents.clone();
            ents.insert(st.floor, g.post_ents.clone());
            match g.kind {
                crate::solve_local::ExitKind::Stair
                | crate::solve_local::ExitKind::Script
                | crate::solve_local::ExitKind::Cloud
                | crate::solve_local::ExitKind::UseItem => {
                    // 取参源分叉：Stair=出口梯实体 param（stair_target）、
                    // Script/Cloud/UseItem=换层两元组（C3a gate_arrive 线程化
                    // pending_floor、C3b 云边候选、F3 符21/22 case21/22）；
                    // 落点/子态/位图/入队同构。区间门仅 Cloud（UseItem var3=false）。
                    let edge = match g.kind {
                        crate::solve_local::ExitKind::Stair => {
                            stair_target(level, g.post_pos)
                        }
                        _ => g.cross_edge,
                    };
                    let Some((target, went_down)) = edge else {
                        // 出口梯无对应实体（夹具缺口）/Script/Cloud 缺两元组 ⇒ 丢边，报审注明
                        continue;
                    };
                    if g.kind == crate::solve_local::ExitKind::Cloud {
                        // 批3 C3b 区间门（a.java:7135-7178：strict=true 只有筋斗云
                        // 调用点带；min/maxFloorReached=已访层域 ⇒ floors 极值）
                        let lo = st.floors.iter().min().copied().unwrap_or(st.floor);
                        let hi = st.floors.iter().max().copied().unwrap_or(st.floor);
                        if target < lo || target > hi {
                            continue;
                        }
                    }
                    let Some(tspec) = per_floor.get(&target) else {
                        // 目标层缺规格 ⇒ 不扩展
                        continue;
                    };
                    let tlevel = tspec.level.as_ref().expect("per_floor 已填 Some");
                    let Some((lx, ly)) = landing(tlevel, target, went_down) else {
                        continue;
                    };
                    if !ents.contains_key(&target) {
                        // 新层首入=sprite 全活（save_floor_state count==0 语义）
                        ents.insert(target, sprite_full(tlevel));
                    }
                    let ns = GSt {
                        floor: target,
                        pos: (lx, ly),
                        player: g.post.clone(),
                        flags: g.post_flags,
                        kills: st.kills + g.kills,
                        steps: st.steps + g.steps,
                        ents,
                        floors: {
                            let mut f = st.floors.clone();
                            f.push(target);
                            f
                        },
                    };
                    push_state(
                        &mut nodes,
                        &mut pruned,
                        &mut queue,
                        &mut by_key,
                        &floor_dist,
                        id,
                        g.route,
                        ns,
                    );
                }
                crate::solve_local::ExitKind::Ending => {
                    // 通关终局边：组装全局目标（route=父链段+本层段）
                    let mut route = route_of_global(&nodes, id);
                    route.extend(g.route.iter().cloned());
                    let ma = monsters_alive_of(&input.floors, &ents);
                    goals.push(GlobalGoal {
                        hp: g.post.hp,
                        kills: st.kills + g.kills,
                        steps: st.steps + g.steps,
                        route,
                        floors: st.floors.clone(),
                        post: g.post.clone(),
                        flags: st.flags,
                        monsters_alive: ma,
                        ents,
                    });
                }
            }
        }
    }

    let confidence = if goals.is_empty() {
        "feasible"
    } else if hit_bound || node_cap_hit || !truncated_layers.is_empty() {
        // R1 层界（裁定 A）：任一层被截断/节点上限触界/打界 ⇒ 探索不完整 ⇒ 不冒充 optimal
        "bounded-best"
    } else {
        "optimal"
    };
    GlobalResult {
        goals,
        confidence,
        bound: input.bound,
        expansions,
        truncated_layers,
        node_cap,
        node_cap_hit,
    }
}

/// 目标 → 计划 §3.3 `SolverState`（字段名不许漂；C4）。
pub fn to_solver_state(goal: &GlobalGoal) -> SolverState {
    SolverState {
        floor: goal.floors.last().copied().unwrap_or(51) as u8,
        hp: DiscreteHp(goal.hp),
        atk: goal.post.atk,
        def: goal.post.def,
        keys: (
            goal.post.yellow as u8,
            goal.post.blue as u8,
            goal.post.red as u8,
        ),
        gold: DiscreteGold(goal.post.gold),
        inventory: CanonicalBag::from_player(&goal.post),
        flags: ScriptFlags(goal.flags),
        kills: goal.kills as u32,
        monsters_alive: goal.monsters_alive,
    }
}

/// monsters_alive 全局口径（C4）：已访层=活体列表∩`entity_category_with_floor==8`
/// （C6a 起列表即引擎活体槽，无位图换算）；未访层（缺席键=首入前）=sprite 全活计数。
fn monsters_alive_of(
    floors: &BTreeMap<i32, LevelSpec>,
    visited: &BTreeMap<i32, Vec<EntRec>>,
) -> u32 {
    let mut n = 0u32;
    for (f, recs) in visited {
        let _ = floors.get(f);
        for r in recs {
            if entity_category_with_floor(r.ty, *f) == 8 {
                n += 1;
            }
        }
    }
    for (f, level) in floors {
        if visited.contains_key(f) {
            continue;
        }
        n += level
            .entities
            .iter()
            .filter(|e| entity_category_with_floor(e.ty, *f) == 8)
            .count() as u32;
    }
    n
}



/// 入队+K₀ 去重+Pareto（批2 `run` 入队段镜像；**相等也丢**——防 bounce 零步等价环）。
fn push_state(
    nodes: &mut Vec<GNode>,
    pruned: &mut Vec<bool>,
    queue: &mut BinaryHeap<(Reverse<u32>, u32, Reverse<u32>)>,
    by_key: &mut BTreeMap<GKey, Vec<u32>>,
    floor_dist: &BTreeMap<i32, u32>,
    parent: u32,
    segment: Vec<String>,
    ns: GSt,
) {
    let ns_floor = ns.floor;
    let depth = if parent == u32::MAX {
        0
    } else {
        nodes[parent as usize].depth + 1
    };
    let k = key_of(&ns);
    let list = by_key.entry(k).or_default();
    let mut dominated = false;
    let mut kill: Vec<u32> = Vec::new();
    for &eid in list.iter() {
        let e = &nodes[eid as usize].st;
        if weakly_ge(e, &ns) {
            dominated = true;
            break;
        }
        if dominates(&ns, e) {
            kill.push(eid);
        }
    }
    if dominated {
        return;
    }
    list.retain(|x| !kill.contains(x));
    for &eid in &kill {
        pruned[eid as usize] = true;
    }
    let new_id = nodes.len() as u32;
    nodes.push(GNode {
        st: ns,
        parent,
        depth,
        segment,
    });
    pruned.push(false);
    list.push(new_id);
    let d = floor_dist.get(&ns_floor).copied().unwrap_or(u32::MAX);
    queue.push((Reverse(d), depth, Reverse(new_id)));
}

/// C6c 层链距目标（反向 BFS 自层50；边=楼梯 param 目标（ty7/8 的 `param&0xff`，
/// **仅楼梯**——见函数内 ±1 剔除注）；图外层=`u32::MAX` 最后弹出（仍会被探索=
/// 完备性不依赖启发式）。解环：51-54 环经 f1（f54 ty7→1）由 f24 链反向注入。
/// 确定性：BTreeMap 迭代+pred 排序+BFS 队列序全固定。
fn floor_chain_dist(floors: &BTreeMap<i32, LevelSpec>) -> BTreeMap<i32, u32> {
    let mut radj: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
    for (f, spec) in floors {
        let mut succ: Vec<i32> = Vec::new();
        for e in &spec.entities {
            if e.ty == 7 || e.ty == 8 {
                succ.push(e.param & 0xff);
            }
        }
        // 只用楼梯边：±1（符21/22 近似）会把 dist[f51] 压成 1（f51-1=50）拉平
        // 梯度、且 51-54 环靠 f24 链经 f1 反向注入（f54 ty7→1）解环——符21 分支
        // （f45..49→50）在纯楼梯图里 dist=MAX=最后探索（仍探索=完备性不依赖它）。
        for t in succ {
            if floors.contains_key(&t) {
                radj.entry(t).or_default().push(*f);
            }
        }
    }
    let mut dist: BTreeMap<i32, u32> = BTreeMap::new();
    let mut q: VecDeque<(i32, u32)> = VecDeque::new();
    dist.insert(50, 0);
    q.push_back((50, 0));
    while let Some((f, d)) = q.pop_front() {
        if let Some(preds) = radj.get(&f) {
            let mut ps = preds.clone();
            ps.sort_unstable();
            for &p in &ps {
                if !dist.contains_key(&p) {
                    dist.insert(p, d + 1);
                    q.push_back((p, d + 1));
                }
            }
        }
    }
    dist
}

// ---- C5：JSON 输出（手写，与批2 SolveResult::to_json 同范式；route token全安全字符无需转义）----

impl GlobalGoal {
    fn to_json(&self) -> String {
        let floors: Vec<String> = self.floors.iter().map(|f| f.to_string()).collect();
        let route: Vec<String> = self
            .route
            .iter()
            .map(|r| format!("\"{r}\""))
            .collect();
        format!(
            "{{\"hp\":{},\"kills\":{},\"steps\":{},\"floors\":[{}],\"monsters_alive\":{},\"route\":[{}]}}",
            self.hp,
            self.kills,
            self.steps,
            floors.join(","),
            self.monsters_alive,
            route.join(",")
        )
    }
}

impl GlobalResult {
    /// C5：确定性双跑字节同（计划 §4 批3 判据③）；形状=confidence/bound/expansions/goals[]。
    pub fn to_json(&self) -> String {
        let goals: Vec<String> = self.goals.iter().map(|g| g.to_json()).collect();
        let trunc: Vec<String> = self
            .truncated_layers
            .iter()
            .map(|(f, n)| format!("[{f},{n}]"))
            .collect();
        format!(
            "{{\"confidence\":\"{}\",\"bound\":{},\"expansions\":{},\"node_cap\":{},\"node_cap_hit\":{},\"goals\":[{}],\"truncated_layers\":[{}]}}",
            self.confidence,
            self.bound,
            self.expansions,
            self.node_cap,
            self.node_cap_hit,
            goals.join(","),
            trunc.join(",")
        )
    }
}
