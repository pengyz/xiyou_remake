//! 【A1 证据】全游戏 `ty83` 场景门审计表（证据工具，**不参与门禁**）。
//!
//! 回答 P7 复核红 F8 留下的问题：**撞门臂还有没有存在价值**。
//!
//! 用法：
//! ```text
//! cargo run --release -p game-solver --example audit_gates
//! ```
//!
//! 每扇门输出一行：
//! - `bump/walk`：`probe_gate_verdict`（与搜索同一个 `StGate`）在两组上下文下的判门结果
//!   —— `0` = `flags=0, hp=1, 空包`；`A` = `flags=bit0..67 全 1, hp=1, 背包含 16`。
//!   两组取并（`ever`）得到该臂**在该模型下能否打开**的保守上界。
//! - `cls`：门格分类（`probe_cell_class`，与 `Search::classify` 同实现）。
//! - `adj`：门的 4 邻格是否有「从 start 可达」的格 ⇒ 这扇门**能否被撞**（撞门需要站在邻格）。
//! - `ops`：脚本 op 前缀清单（逐字取自 `game-core/tests/fixtures/level_scripts.txt`）。
//! - `setf`：`evaluate` 的 `set_flags` ∪ 脚本完成旗标 `flags[param]`。
//! - `obs`：`setf` 里**有门臂读**的成员（机械判定：置该位会让某扇门的某臂判决翻转）。
//!
//! **诚实边界**：`ever` 是「两组上下文取并」的探针，不是「这扇门必然开」的证明；
//! 必经性用 `adj` + `ops` 回答（撞门不移动玩家 ⇒ 门格永远不在步行路径上，
//! 「必经」只能落在「脚本效果是否必需」上）。
use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_solver::ingest::{ingest, SolverInput};
use game_solver::solve_local::{
    load_floor, probe_cell_class, probe_gate_verdict, LevelSpec, PlayerSpec, SolveInput,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn nums(t: &str) -> Vec<i32> {
    t.lines().map(|l| l.trim()).filter(|s| !s.is_empty()).map(|s| s.parse().unwrap()).collect()
}

fn load_solver_input() -> SolverInput {
    let root = repo();
    let mut maplvs = Vec::new();
    let mut sprites = Vec::new();
    for i in 0..55 {
        maplvs.push(std::fs::read(root.join(format!("assets/raw/maplv{i}"))).unwrap());
        sprites.push(std::fs::read(root.join(format!("assets/raw/sprite{i}"))).unwrap());
    }
    let maplv_refs: Vec<&[u8]> = maplvs.iter().map(|v| v.as_slice()).collect();
    let sprite_refs: Vec<&[u8]> = sprites.iter().map(|v| v.as_slice()).collect();
    ingest(
        &maplv_refs,
        &sprite_refs,
        LEVEL_SCRIPTS,
        include_str!("../../game-core/tests/fixtures/door12.txt"),
        include_str!("../../game-core/tests/fixtures/door13.txt"),
        include_str!("../../../data/tables/enemies.json"),
        include_str!("../../../data/tables/items.json"),
        include_str!("../../../data/tables/shop_goods.json"),
    )
    .expect("ingest")
}

const LEVEL_SCRIPTS: &str = include_str!("../../game-core/tests/fixtures/level_scripts.txt");
const GLOBAL_ENTRY: (i32, i32) = (1, 11);

fn mk_player(hp: i32, items: Vec<u8>) -> PlayerSpec {
    let n = items.len();
    let mut types = items;
    types.resize(32, 0);
    let mut uses = vec![0u8; 32];
    for i in 0..n {
        uses[i] = 1;
    }
    PlayerSpec {
        hp,
        atk: 10,
        def: 10,
        yellow: 0,
        blue: 0,
        red: 0,
        gold: 0,
        item_types: types,
        item_uses: uses,
        item_stack_size: n as i32,
    }
}

const ALL_FLAGS: u128 = (1u128 << 68) - 1;

/// 脚本 op 前缀集合（`TAK_..` ⇒ `TAK`）。
fn script_ops(param: i32) -> Vec<String> {
    let line = LEVEL_SCRIPTS.lines().nth(param as usize).unwrap_or("");
    let mut v: Vec<String> = line
        .trim_end_matches('\x1e')
        .split_whitespace()
        .map(|t| t.split('_').next().unwrap_or(t).to_string())
        .collect();
    v.dedup();
    v
}

struct Gate {
    floor: i32,
    param: i32,
    cell: (i32, i32),
    cls: &'static str,
    bump: [bool; 2],
    walk: [bool; 2],
    setf: BTreeSet<i32>,
    adj: bool,
    ops: Vec<String>,
}

fn main() {
    let si = load_solver_input();
    let tables = CombatTables {
        enemy_base_hp: nums(include_str!("../../game-core/tests/fixtures/enemybasehp.txt")),
        enemy_base_atk: nums(include_str!("../../game-core/tests/fixtures/enemybaseatk.txt")),
        enemy_base_def: nums(include_str!("../../game-core/tests/fixtures/enemybasedef.txt")),
        difficulty_multipliers: nums(include_str!(
            "../../game-core/tests/fixtures/difficulty_multipliers.txt"
        )),
    };
    let gold = nums(include_str!("../../game-core/tests/fixtures/enemybasegold.txt"));
    let base = tables.to_records(&gold);
    let scaled = scale_enemy_stats(&base, 1);
    let tile_walk = nums(include_str!("../../game-core/tests/fixtures/tile_walkability.txt"));

    let mut inputs: BTreeMap<i32, SolveInput> = BTreeMap::new();
    for f in 1..=54i32 {
        if let Ok(inp) = load_floor(
            &si,
            f as usize,
            GLOBAL_ENTRY,
            mk_player(300, vec![]),
            scaled.clone(),
            base.clone(),
            &tile_walk,
        ) {
            if inp.level.is_some() {
                inputs.insert(f, inp);
            }
        }
    }

    let ctx = [mk_player(1, vec![]), mk_player(1, vec![16])];
    let flags_ctx = [0u128, ALL_FLAGS];

    let mut gates: Vec<Gate> = Vec::new();
    for (floor, inp) in inputs.iter() {
        let level = inp.level.as_ref().expect("level");
        let reach = reach_from(level, inp, level.start);
        for e in level.entities.iter().filter(|e| e.ty == 83) {
            let cell = (e.x, e.y);
            let mut bump = [false; 2];
            let mut walk = [false; 2];
            let mut setf: BTreeSet<i32> = BTreeSet::new();
            for k in 0..2 {
                bump[k] = probe_gate_verdict(inp, &ctx[k], flags_ctx[k], None, e.param, false);
                walk[k] = probe_gate_verdict(inp, &ctx[k], flags_ctx[k], None, e.param, true);
            }
            // set_flags 与「脚本完成旗标 flags[param]」都要算（引擎 scene.rs:4834-4835）
            for k in 0..2 {
                let level = inp.level.as_ref().unwrap();
                let ents: Vec<_> = {
                    let mut v = Vec::new();
                    for spec in &level.entities {
                        v.push(game_solver::solve_local::EntRec {
                            ty: spec.ty,
                            x: spec.x,
                            y: spec.y,
                            param: spec.param,
                        });
                    }
                    v
                };
                let mut view = AuditView {
                    ents: &ents,
                    difficulty: inp.difficulty,
                    hp: ctx[k].hp,
                    items: &ctx[k].item_types,
                    flags: flags_ctx[k],
                };
                let v = game_core::rules::gate::evaluate(e.param, &mut view, false);
                for f in v.set_flags {
                    setf.insert(f);
                }
            }
            setf.insert(e.param);
            let adj = neighbours(cell)
                .into_iter()
                .any(|c| c.0 >= 0 && c.1 >= 0 && reach.contains(&c));
            gates.push(Gate {
                floor: *floor,
                param: e.param,
                cell,
                cls: probe_cell_class(inp, None, cell),
                bump,
                walk,
                setf,
                adj,
                ops: script_ops(e.param),
            });
        }
    }

    // 旗标读取面：置位某旗标会让哪些 **param 的门** 判决翻转（机械判定，逐 (param, arm)）。
    //
    // 关键语义（gate.rs:73）：`let mut gate = !v.flag(param);` —— **每扇门都读自己的旗标位**
    // 作为「是否已开过」的自守卫。所以「旗标可观测」必须分成两类：
    //   - self：只有「自己的 param == 该旗标位」时读 ⇒ 只是自重入保护，**不能解锁任何东西**；
    //   - cross：存在 `param != 旗标位` 的门臂读它 ⇒ 这才是「撞这扇门能改变别的门」。
    let mut readers: BTreeMap<i32, BTreeSet<i32>> = BTreeMap::new();
    for f in 0..68i32 {
        let bit = 1u128 << f;
        for (_, inp) in inputs.iter() {
            for e in inp.level.as_ref().unwrap().entities.iter().filter(|e| e.ty == 83) {
                for k in 0..2 {
                    for from_walk in [false, true] {
                        let a = probe_gate_verdict(inp, &ctx[k], 0, None, e.param, from_walk);
                        let b = probe_gate_verdict(inp, &ctx[k], bit, None, e.param, from_walk);
                        if a != b {
                            readers.entry(f).or_default().insert(e.param);
                        }
                    }
                }
            }
        }
    }
    let gate_observable: BTreeSet<i32> = readers.keys().copied().collect();
    let cross_observable: BTreeSet<i32> = readers
        .iter()
        .filter(|(f, ps)| ps.iter().any(|p| p != *f))
        .map(|(f, _)| *f)
        .collect();

    // ---------------------------------------------------------------- 表
    println!("== ty83 场景门审计（floor/param/cell/cls/adj/bump(0,A)/walk(0,A)/class/ops/setf/obs）==");
    println!("GATES={} SELF_READ_FLAGS={:?}", gates.len(), gate_observable);
    println!("CROSS_READ_FLAGS={:?}", cross_observable);
    for (f, ps) in readers.iter() {
        if ps.iter().any(|p| p != f) {
            println!("  flag {f} 被这些 param 的门臂读: {:?}", ps);
        }
    }
    let mut tally: BTreeMap<&'static str, i32> = BTreeMap::new();
    for g in &gates {
        let ever_b = g.bump[0] || g.bump[1];
        let ever_w = g.walk[0] || g.walk[1];
        let class = match (ever_b, ever_w) {
            (true, false) => "bump_only",
            (false, true) => "walk_only",
            (true, true) => "both",
            (false, false) => "neither",
        };
        *tally.entry(class).or_insert(0) += 1;
        let obs: Vec<i32> = g.setf.iter().copied().filter(|f| gate_observable.contains(f)).collect();
        let xobs: Vec<i32> = g.setf.iter().copied().filter(|f| cross_observable.contains(f)).collect();
        println!(
            "f{:<2} p{:<2} ({:>2},{:>2}) cls={:<7} adj={:<5} bump={}{} walk={}{} {:>9} ops={:<44} setf={:?} obs={:?} xobs={:?}",
            g.floor,
            g.param,
            g.cell.0,
            g.cell.1,
            g.cls,
            g.adj,
            g.bump[0] as u8,
            g.bump[1] as u8,
            g.walk[0] as u8,
            g.walk[1] as u8,
            class,
            g.ops.join("+"),
            g.setf,
            obs,
            xobs
        );
    }
    println!("TALLY={tally:?}");

    // 撞门臂的价值判定：bump_only 门逐扇给出「能否被撞 / 脚本是否有非演出 op / 旗标是否可观测」
    println!("-- bump_only 门的价值 --");
    for g in gates.iter().filter(|g| (g.bump[0] || g.bump[1]) && !(g.walk[0] || g.walk[1])) {
        let non_dialog: Vec<&str> = g
            .ops
            .iter()
            .map(|s| s.as_str())
            .filter(|o| !matches!(*o, "TAK" | "SEE" | "SMS"))
            .collect();
        let obs: Vec<i32> = g.setf.iter().copied().filter(|f| cross_observable.contains(f)).collect();
        println!(
            "f{} p{} ({:?}) adj={} ops={:?} non_dialog={:?} cross_observable_flags={:?} ⇒ 价值={}",
            g.floor,
            g.param,
            g.cell,
            g.adj,
            g.ops,
            non_dialog,
            obs,
            if obs.is_empty() && non_dialog.is_empty() {
                "仅动作覆盖（零进度收益）"
            } else {
                "潜在进度相关（需逐条取证）"
            }
        );
    }

    // f35 是否需要撞门：从 start 到任一出口的可行走步数（门格本来就 Blocked，故不依赖撞门）
    if let Some(f35) = inputs.get(&35) {
        let level = f35.level.as_ref().unwrap();
        let reach = reach_from(level, f35, level.start);
        let exits: Vec<(i32, i32)> = level.entities.iter().filter(|e| e.ty == 7 || e.ty == 8).map(|e| (e.x, e.y)).collect();
        let dist = bfs(level, f35, level.start);
        println!("-- f35 绕门可达性 --");
        println!("f35 start={:?} exits={:?}", level.start, exits);
        for ex in &exits {
            println!("  exit {:?} dist={:?}", ex, dist.get(ex));
        }
        println!("f35 reachable cells={}", reach.len());
    }
}

fn neighbours(c: (i32, i32)) -> Vec<(i32, i32)> {
    vec![(c.0 + 1, c.1), (c.0 - 1, c.1), (c.0, c.1 + 1), (c.0, c.1 - 1)]
}

/// 从 `start` 出发的 4 邻 BFS（可走 = `probe_cell_class != "blocked"`，门/怪/拾取/商人
/// 都算「可进」——这是几何可达性的上界）。
fn bfs(level: &LevelSpec, inp: &SolveInput, start: (i32, i32)) -> BTreeMap<(i32, i32), i32> {
    let _ = level;
    let mut dist: BTreeMap<(i32, i32), i32> = BTreeMap::new();
    let mut q: VecDeque<(i32, i32)> = VecDeque::new();
    dist.insert(start, 0);
    q.push_back(start);
    while let Some(c) = q.pop_front() {
        let d = dist[&c];
        for n in neighbours(c) {
            if n.0 < 0 || n.1 < 0 || n.0 >= level.cols || n.1 >= level.rows || dist.contains_key(&n) {
                continue;
            }
            if probe_cell_class(inp, None, n) == "blocked" {
                continue;
            }
            dist.insert(n, d + 1);
            q.push_back(n);
        }
    }
    dist
}

fn reach_from(level: &LevelSpec, inp: &SolveInput, start: (i32, i32)) -> BTreeSet<(i32, i32)> {
    bfs(level, inp, start).into_keys().collect()
}

/// 与 `StGate` 逐字段同义的审计视图（只为取 `set_flags`；判门本身走 `probe_gate_verdict`）。
struct AuditView<'a> {
    ents: &'a [game_solver::solve_local::EntRec],
    difficulty: i32,
    hp: i32,
    items: &'a [u8],
    flags: u128,
}

impl game_core::rules::gate::GateView for AuditView<'_> {
    fn script_count(&self) -> i32 {
        68
    }
    fn flag(&self, idx: i32) -> bool {
        idx >= 0 && idx < 128 && (self.flags >> idx) & 1 == 1
    }
    fn set_flag(&mut self, idx: i32) {
        if idx >= 0 && idx < 128 {
            self.flags |= 1u128 << idx;
        }
    }
    fn entity_at(&self, x: i32, y: i32, type_id: i32) -> i32 {
        self.ents
            .iter()
            .position(|e| e.x == x && e.y == y && e.ty == type_id)
            .map(|i| i as i32)
            .unwrap_or(-1)
    }
    fn alive_type_on_floor(&self, type_id: i32) -> bool {
        self.ents.iter().any(|e| e.ty == type_id)
    }
    fn difficulty_index(&self) -> i32 {
        self.difficulty
    }
    fn player_hp(&self) -> i32 {
        self.hp
    }
    fn has_item(&self, item: i32) -> bool {
        item >= 0 && self.items.contains(&(item as u8))
    }
    fn cross_floor_snapshot(&mut self, _floor: i32, _type_id: i32) -> bool {
        false
    }
    fn predict_hp_loss_vs(&self, _ety: i32, _no_counter: bool) -> i32 {
        0
    }
}

#[allow(dead_code)]
fn player_of(_p: &PlayerCombat) {}
