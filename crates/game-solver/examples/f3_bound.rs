//! f3 单层三档探针（证据工具，**不参与门禁**）。
//!
//! 复刻 `solver run`（默认 200k）到达 f3 时的**真实链态**——不是冷启 S0：
//! 链 51→52→53→54→1→2 的出层态 = `hp=3227, atk=def=730, gold=225,
//! flags=0xe337be0000000000, bag=[(13,uses=255)], entry=(2,11)`。
//! 依据：`solver run --out` 的 floor 2 条目（`in_player`/`out_flags`），
//! 见 docs/findings/2026-11-19-f3-gate27-walk-only.md §3。
//!
//! 用法：
//! ```text
//! cargo run --release -p game-solver --example f3_bound -- 20000 200000 1000000
//! F3_GRID=1 cargo run --release -p game-solver --example f3_bound -- 200000
//! ```
//!
//! 输出每档：`ok/truncated/out_hp/route_len/elapsed` + E 面界统计（若已接入）。

use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_solver::ingest::{ingest, SolverInput};
use game_solver::solve_local::{
    load_floor, solve_local_exit_toward, LevelSpec, PlayerSpec, SolveInput,
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn nums(t: &str) -> Vec<i32> {
    t.lines()
        .map(|l| l.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect()
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
        include_str!("../../game-core/tests/fixtures/level_scripts.txt"),
        include_str!("../../game-core/tests/fixtures/door12.txt"),
        include_str!("../../game-core/tests/fixtures/door13.txt"),
        include_str!("../../../data/tables/enemies.json"),
        include_str!("../../../data/tables/items.json"),
        include_str!("../../../data/tables/shop_goods.json"),
    )
    .expect("ingest")
}

const GLOBAL_ENTRY: (i32, i32) = (1, 11);

/// f3 链态（f2 出层态，逐字段来自 `solver run` 的 JSON）。
fn f3_chain_player() -> PlayerSpec {
    let mut item_types = vec![0u8; 32];
    let mut item_uses = vec![0u8; 32];
    item_types[0] = 13;
    item_uses[0] = 255;
    PlayerSpec {
        hp: 3227,
        atk: 730,
        def: 730,
        yellow: 0,
        blue: 0,
        red: 0,
        gold: 225,
        item_types,
        item_uses,
        item_stack_size: 1,
    }
}

const F3_ENTRY: (i32, i32) = (2, 11);
const F3_FLAGS: u128 = 0xe337be_00000000;

fn main() {
    let limits: Vec<usize> = std::env::args()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let limits = if limits.is_empty() { vec![20_000, 200_000, 1_000_000] } else { limits };

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

    let inp: SolveInput = load_floor(
        &si,
        3,
        GLOBAL_ENTRY,
        PlayerSpec::from_combat(PlayerCombat {
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
        }),
        scaled.clone(),
        base.clone(),
        &tile_walk,
    )
    .expect("f3 input");
    let level: &LevelSpec = inp.level.as_ref().expect("f3 level");

    if std::env::var("F3_GRID").is_ok() {
        dump_grid(level);
    } else {
        dump_ents(level);
    }

    let lays = game_solver::floorgraph::lay_targets();
    let toward = game_solver::witness::up_line_next(3, level, &lays);
    let player = f3_chain_player();
    println!(
        "== f3 probe: entry={:?} in_hp={} atk/def={}/{} gold={} flags={:#x} toward={:?} ==",
        F3_ENTRY, player.hp, player.atk, player.def, player.gold, F3_FLAGS, toward
    );

    if std::env::var("F3_FIGHT").is_ok() {
        // A 级证据：本层每个可见敌人的「撞一次」预测损血（真调引擎 predict_engagement，
        // 与搜索 bump 臂同一函数、同 no_counter=true）。
        for e in &level.entities {
            if game_core::subsystem::script_host::entity_category_with_floor(e.ty, 3) == 8 {
                let loss = game_core::subsystem::battle::predict_engagement(
                    &player.to_combat(),
                    &level.enemies,
                    &level.enemy_base,
                    e.ty,
                    true,
                );
                println!("fight ty={} at ({},{}) predict_loss={}", e.ty, e.x, e.y, loss);
            }
        }
    }

    if std::env::var("F3_LOCAL").is_ok() {
        // 【批8】层路径（`solve_layer`/`exit_toward`）的消费者是「朝目标出口」查询，
        // hp-max 剪枝在那里不成立（实测会把通向目标出口的路径剪掉，见 solve_layer_with
        // 注释）⇒ 剪枝只在 `solve_local`（全层 hp-max + 最少击杀）启用。此处用
        // **同一 f3 态**（level.start 置为 F3_ENTRY）走 solve_local 路径取证。
        let mut lv = level.clone();
        lv.start = F3_ENTRY;
        let inp2 = game_solver::solve_local::SolveInput {
            floor: 3,
            difficulty: 0,
            level: Some(lv),
        };
        std::env::set_var("SOLVER_LAYER_LIMIT", "200000");
        let t = Instant::now();
        let ex = game_solver::solve_local::solve_local(&inp2);
        let hb = game_solver::hp_bound::hp_bound_stat();
        println!(
            "== f3 solve_local: reachable={} truncated={} max_hp={:?} min_kills={:?} confidence={} route_len={} elapsed={:.2}s",
            ex.reachable,
            ex.truncated,
            ex.max_hp,
            ex.min_kills,
            ex.confidence,
            ex.route.len(),
            t.elapsed().as_secs_f64()
        );
        println!(
            "  hp_bound enabled={} reason={} ty={} slots={} pickup_types={} root_hp={} \
             root_bound={} pruned_entry={} pruned_pop={}",
            hb.enabled,
            hb.reason,
            hb.reason_ty,
            hb.slots,
            hb.pickup_types,
            hb.root_hp,
            hb.root_bound,
            hb.pruned_entry,
            hb.pruned_pop,
        );
    }

    for limit in limits {
        std::env::set_var("SOLVER_LAYER_LIMIT", limit.to_string());
        let t = Instant::now();
        let (ex, hit) =
            solve_local_exit_toward(&inp, F3_ENTRY, &player, F3_FLAGS, None, toward);
        let el = t.elapsed().as_secs_f64();
        println!(
            "limit={limit:>8} ok={} truncated={} out_hp={} waypoints={} exit={:?} kind={:?} \
             hit={} elapsed={el:.2}s",
            ex.ok,
            ex.truncated,
            ex.out_hp,
            ex.route.len(),
            ex.exit,
            ex.kind,
            hit,
        );
        if std::env::var("F3_PILL").is_ok() {
            let pd = game_solver::solve_local::pill_dom_stat();
            println!(
                "  pill enabled={} reason={} ty={} pills={} states={} cmp={} pruned_new={} killed_old={} budget_exhausted={}",
                pd.enabled, pd.reason, pd.reason_ty, pd.pills, pd.states, pd.cmp, pd.pruned_new, pd.killed_old, pd.budget_exhausted
            );
        }
        // 【批8】hp 可采纳上界剪枝的可观测面（OP 判据 1 的 stat 要求）。
        let hb = game_solver::hp_bound::hp_bound_stat();
        println!(
            "  hp_bound enabled={} reason={} ty={} slots={} pickup_types={} root_hp={} \
             root_bound={} pruned_entry={} pruned_pop={}",
            hb.enabled,
            hb.reason,
            hb.reason_ty,
            hb.slots,
            hb.pickup_types,
            hb.root_hp,
            hb.root_bound,
            hb.pruned_entry,
            hb.pruned_pop,
        );
    }
}

fn dump_ents(level: &LevelSpec) {
    println!("-- f3 entity table (static) --");
    let mut hist: BTreeMap<i32, (i32, Vec<(i32, i32)>)> = BTreeMap::new();
    for e in &level.entities {
        let slot = hist.entry(e.ty).or_insert((0, Vec::new()));
        slot.0 += 1;
        slot.1.push((e.x, e.y));
    }
    for (ty, (n, cells)) in &hist {
        let cat = game_core::subsystem::script_host::entity_category_with_floor(*ty, 3);
        let gain = game_core::effect_registry::certified_monotone_pickup(*ty, 3);
        println!(
            "ty={ty:>3} n={n:>3} cat={cat} monotone_gain={:?} cells={:?}",
            gain, cells
        );
    }
    println!("total ents={} cols={} rows={}", level.entities.len(), level.cols, level.rows);
}

fn dump_grid(level: &LevelSpec) {
    let tile_walk = nums(include_str!("../../game-core/tests/fixtures/tile_walkability.txt"));
    println!("-- f3 walkable grid ('#'=wall, '.'=floor, 'D'=door, 'S'=stair, 'G'=ty83, digits=ty%10) --");
    for y in 0..level.rows {
        let mut line = String::new();
        for x in 0..level.cols {
            let _ = &tile_walk;
            let walk = level.terrain[(y * level.cols + x) as usize];
            let ch = if let Some(e) = level.entities.iter().find(|e| e.x == x && e.y == y) {
                if e.ty == 83 {
                    'G'
                } else if e.ty == 7 || e.ty == 8 {
                    'S'
                } else if (1..=3).contains(&e.ty) {
                    'D'
                } else {
                    char::from_digit((e.ty % 10) as u32, 10).unwrap_or('?')
                }
            } else if walk {
                '.'
            } else {
                '#'
            };
            line.push(ch);
        }
        println!("{y:>2} {line}");
    }
}
