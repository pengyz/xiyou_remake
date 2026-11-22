//! 【S4-A 判据】外层→内层路径实化（`crates/game-solver/src/outer_witness.rs`）：
//!
//! 1. `s4a_path_to_tasks`：外层块路径快照 → 内层逐层任务队列——连续同层块归组、
//!    任务入口预算 = 组首块 `state_pre` **逐字段一致**（keys/gold/hp/atk/def）。
//! 2. `s4a_budget_divergence_named`：外层 LB 预算 vs 内层真值分歧**逐字段具名**
//!    落 `budget_drifts`（hp 方向：LB 下界 ⇒ inner ≤ outer）。
//! 3. `s4a_no_silent_override`：候选 0 内层失败 ⇒ 记 attempt、取候选 1 并计数，
//!    绝不静默吞失败；全部失败 ⇒ blocked 带末次 {floor, reason}。
//!
//! 注入式层求解器（不依赖真引擎数据）+ 最小 v2 块表（同 outer.rs tests 口径）。
use game_core::combat::PlayerCombat;
use game_solver::outer::{
    load_block_interfaces, replay_outer_path, FrontierEntry, OuterResult, OuterState, PathSnapshot,
    RunSpec,
};
use game_solver::outer_witness::{path_to_tasks, run_outer_witness, OuterWitnessCfg};
use game_solver::solve_local::{EntSpec, ExitKind, ExitSpec, LevelSpec, PlayerSpec, SolveInput};
use std::cell::Cell;
use std::collections::BTreeMap;

// ---------------------------------------------------------------- 夹具

fn s0() -> PlayerCombat {
    PlayerCombat {
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
    }
}

/// 两层塔合成层：8×3 只 y=1 可走；上行梯 ty7@(7,1) param=目标层 + 下行梯 ty8@(5,1)。
fn corridor(floor: i32, up_to: Option<i32>, down_to: i32) -> SolveInput {
    let (cols, rows) = (8, 3);
    let terrain: Vec<bool> = (0..cols * rows).map(|i| (i / cols) == 1).collect();
    let mut entities = Vec::new();
    let mut exits = Vec::new();
    if let Some(to) = up_to {
        entities.push(EntSpec { ty: 7, x: 7, y: 1, param: to });
        exits.push(ExitSpec { x: 7, y: 1, kind: ExitKind::Stair, index: 1 });
    }
    entities.push(EntSpec { ty: 8, x: 5, y: 1, param: down_to });
    exits.push(ExitSpec { x: 5, y: 1, kind: ExitKind::Stair, index: 2 });
    SolveInput {
        floor,
        difficulty: 0,
        level: Some(LevelSpec {
            cols,
            rows,
            terrain,
            start: (1, 1),
            entities,
            exits,
            player: PlayerSpec::from_combat(s0()),
            enemies: Vec::new(),
            enemy_base: Vec::new(),
            flags: 0,
        }),
    }
}

/// 最小 v2 块表：f51 a0（stair→52）+ f52 b0（boss_kill 锚）。
fn mini_doc() -> String {
    r#"{
  "schema": "xiyou/block-interfaces/v2",
  "floors": [
    { "floor": 51, "entry": [1,1], "blocks": [
      { "id": "a0", "cells": [[1,1],[2,1],[3,1]],
        "cut_doors": [],
        "exports": [],
        "exits": [ { "kind": "stair", "ty": 7, "cell": [3,1], "to": 52, "same_floor": false } ] }
    ] },
    { "floor": 52, "entry": [2,1], "blocks": [
      { "id": "b0", "cells": [[2,1],[3,1]],
        "cut_doors": [],
        "exports": [],
        "exits": [ { "kind": "boss_kill", "ty": 75, "cell": [3,1], "events": [], "post_victory": false, "drops": {} } ] }
    ] }
  ]
}"#
    .to_string()
}

fn fake_exit_ok(player: &PlayerSpec) -> game_solver::solve_local::ExitSolve {
    let mut out = player.clone();
    out.hp += 7;
    out.gold += 3;
    game_solver::solve_local::ExitSolve {
        ok: true,
        exit: Some((7, 1)),
        out_player: out,
        out_flags: 0x2,
        out_ents: Vec::new(),
        route: vec!["1,1".to_string(), "7,1".to_string()],
        out_hp: player.hp + 7,
        kind: Some(ExitKind::Stair),
        cross_edge: None,
        truncated: false,
        pure_pickup: false,
        pure_pickup_reason: None,
    }
}

fn fake_ending(player: &PlayerSpec) -> game_solver::solve_local::ExitSolve {
    game_solver::solve_local::ExitSolve {
        ok: true,
        exit: Some((7, 1)),
        out_player: player.clone(),
        out_flags: 0x2,
        out_ents: Vec::new(),
        route: vec!["7,1".to_string()],
        out_hp: player.hp,
        kind: Some(ExitKind::Ending),
        cross_edge: None,
        truncated: false,
        pure_pickup: false,
        pure_pickup_reason: None,
    }
}

fn fake_fail(player: &PlayerSpec) -> game_solver::solve_local::ExitSolve {
    game_solver::solve_local::ExitSolve {
        ok: false,
        exit: None,
        out_player: player.clone(),
        out_flags: 0,
        out_ents: Vec::new(),
        route: Vec::new(),
        out_hp: player.hp,
        kind: None,
        cross_edge: None,
        truncated: false,
        pure_pickup: false,
        pure_pickup_reason: None,
    }
}

/// 候选注入层：toward=Some ⇒ fake_exit_ok(hit=true)；None ⇒ fake_ending。
fn ok_layer() -> impl Fn(
    &SolveInput,
    (i32, i32),
    &PlayerSpec,
    u128,
    Option<&[game_solver::solve_local::EntRec]>,
    Option<i32>,
    &[game_data::defs::ShopGoodDef],
) -> (game_solver::solve_local::ExitSolve, bool) {
    |input, _entry, p, _fl, _ents, toward, _shops| {
        let _ = input;
        match toward {
            Some(_) => (fake_exit_ok(p), true),
            None => (fake_ending(p), true),
        }
    }
}

fn two_floor_setup() -> (
    game_solver::outer::OuterGraph<'static>,
    RunSpec,
    BTreeMap<i32, SolveInput>,
) {
    // 泄漏 doc（测试生命周期 = 进程，静态借用最省事）。
    let doc: &'static game_solver::outer::BlockInterfaces =
        Box::leak(Box::new(load_block_interfaces(&mini_doc()).unwrap()));
    let floors = vec![51, 52];
    let g = game_solver::outer::OuterGraph::build(doc, &floors, &[]);
    let run = RunSpec {
        floors: floors.clone(),
        start: (51, (1, 1)),
        anchor_floor: 52,
        anchor_boss_ty: 75,
        masked_exits: Vec::new(),
        combat_cost: false,
    };
    let mut levels = BTreeMap::new();
    levels.insert(51, corridor(51, Some(52), 1));
    levels.insert(52, corridor(52, None, 51));
    (g, run, levels)
}

fn fake_frontier(path: &[&str], steps: u64, hp: i32) -> OuterResult {
    let n = 10;
    let state = OuterState {
        node: 1,
        yellow: 0,
        red: 0,
        blue: 0,
        hp,
        gold: 0,
        atk: 10,
        def: 10,
        held: 0,
        picked: vec![0u8; (n + 7) / 8],
        buys: 0,
    };
    OuterResult {
        frontier: vec![FrontierEntry {
            steps,
            killable: true,
            fatal: false,
            state: state.clone(),
            path: path.iter().map(|s| s.to_string()).collect(),
            gates: Vec::new(),
        }],
        pops: 1,
        truncated: false,
        unapplied_exports: 0,
        needs_sensitivity: Vec::new(),
        skipped_trans: Vec::new(),
        reentry_edges: 0,
        masked_exits: Vec::new(),
    }
}

// ---------------------------------------------------------------- 判据 1

#[test]
fn s4a_path_to_tasks() {
    // 三块路径：f51 两块（复访同层归组）→ f52 终局。
    let (g, run, levels) = two_floor_setup();
    // 真回放：外层路径预算必须与内层任务入口逐字段一致（keys/gold/hp/atk/def）。
    let snaps = replay_outer_path(&g, &run, &[], &["f51:a0".to_string(), "f52:b0".to_string()]).unwrap();
    let tasks = path_to_tasks(&snaps);
    assert_eq!(tasks.len(), 2, "f51 一访问 + f52 一访问");
    assert_eq!(tasks[0].floor, 51);
    assert_eq!(tasks[0].blocks, vec!["a0"]);
    assert_eq!(tasks[0].toward, Some(52));
    assert!(!tasks[0].is_final);
    assert_eq!(tasks[1].floor, 52);
    assert!(tasks[1].is_final);
    assert_eq!(tasks[1].toward, None);
    // 预算逐字段一致（组首块 state_pre vs 任务入口预算本体）。
    let eb = &tasks[0].entry_budget;
    let sp = &snaps[0].state_pre;
    assert_eq!(eb.yellow, sp.yellow);
    assert_eq!(eb.blue, sp.blue);
    assert_eq!(eb.red, sp.red);
    assert_eq!(eb.gold, sp.gold);
    assert_eq!(eb.hp, sp.hp);
    assert_eq!(eb.atk, sp.atk);
    assert_eq!(eb.def, sp.def);
    // 连续同层块归组（合成快照面）。
    let mk = |node: usize, floor: i32, block: &str, hp: i32, steps: u64| PathSnapshot {
        node,
        floor,
        block: block.to_string(),
        state_pre: OuterState {
            node,
            yellow: 2,
            red: 0,
            blue: 0,
            hp,
            gold: 5,
            atk: 10,
            def: 10,
            held: 0,
            picked: vec![0u8; 2],
            buys: 0,
        },
        state: OuterState { node, yellow: 2, red: 0, blue: 0, hp, gold: 5, atk: 10, def: 10, held: 0, picked: vec![0u8; 2], buys: 0 },
        steps,
    };
    let s3 = vec![mk(0, 51, "a0", 300, 1), mk(1, 51, "a1", 299, 2), mk(2, 52, "b0", 299, 3)];
    let t3 = path_to_tasks(&s3);
    assert_eq!(t3.len(), 2);
    assert_eq!(t3[0].blocks, vec!["a0", "a1"], "连续同层块归组为一次访问");
    assert_eq!(t3[0].entry_budget.yellow, 2, "预算取组首块 state_pre");
    assert_eq!(t3[0].entry_budget.gold, 5);
    assert_eq!(t3[0].entry_budget.hp, 300, "预算 hp 取组首块（300），非组尾（299）");
    assert!(t3[1].is_final);
    let _ = levels;
}

// ---------------------------------------------------------------- 判据 2

#[test]
fn s4a_budget_divergence_named() {
    let (g, run, levels) = two_floor_setup();
    let search = fake_frontier(&["f51:a0", "f52:b0"], 7, 300);
    // 注入层给 hp+7/gold+3 ⇒ 与外层预算（300/0）必分歧，且逐字段具名。
    let okl = ok_layer();
    let layer = move |input: &SolveInput,
                      entry: (i32, i32),
                      p: &PlayerSpec,
                      fl: u128,
                      ents: Option<&[game_solver::solve_local::EntRec]>,
                      toward: Option<i32>,
                      shops: &[game_data::defs::ShopGoodDef]| okl(input, entry, p, fl, ents, toward, shops);
    let ow = game_solver::outer_witness::run_outer_witness_with(
        &g,
        &run,
        &[],
        &levels,
        PlayerSpec::from_combat(s0()),
        0,
        &[],
        &search,
        &OuterWitnessCfg { candidates: 1, max_floors: 8 },
        &layer,
    );
    assert_eq!(ow.status, "witness");
    assert!(!ow.budget_drifts.is_empty(), "分歧必须具名落账");
    let hp_drift = ow
        .budget_drifts
        .iter()
        .find(|d| d.field == "hp" && d.floor == 52 && d.visit == 1)
        .expect("hp 分歧必须具名");
    assert_eq!(hp_drift.outer, 300);
    assert_eq!(hp_drift.inner, 307);
    assert!(ow.budget_drifts.iter().any(|d| d.field == "gold"), "gold 分歧也要具名");
    assert_eq!(ow.attempts.len(), 1);
    assert_eq!(ow.attempts[0].outcome, "witness");
}

// ---------------------------------------------------------------- 判据 3

#[test]
fn s4a_no_silent_override() {
    let (g, run, levels) = two_floor_setup();
    // 两个相同候选；注入层第一次调用失败、后续成功 ⇒ 候选 0 败、候选 1 成。
    let search = OuterResult {
        frontier: vec![
            FrontierEntry {
                steps: 7,
                killable: true,
                fatal: false,
                state: OuterState { node: 1, yellow: 0, red: 0, blue: 0, hp: 300, gold: 0, atk: 10, def: 10, held: 0, picked: vec![0u8; 2], buys: 0 },
                path: vec!["f51:a0".into(), "f52:b0".into()],
                gates: Vec::new(),
            },
            FrontierEntry {
                steps: 9,
                killable: true,
                fatal: false,
                state: OuterState { node: 1, yellow: 0, red: 0, blue: 0, hp: 300, gold: 0, atk: 10, def: 10, held: 0, picked: vec![0u8; 2], buys: 0 },
                path: vec!["f51:a0".into(), "f52:b0".into()],
                gates: Vec::new(),
            },
        ],
        pops: 2,
        truncated: false,
        unapplied_exports: 0,
        needs_sensitivity: Vec::new(),
        skipped_trans: Vec::new(),
        reentry_edges: 0,
        masked_exits: Vec::new(),
    };
    let calls = Cell::new(0u32);
    let layer = move |_input: &SolveInput,
                      _entry: (i32, i32),
                      p: &PlayerSpec,
                      _fl: u128,
                      _ents: Option<&[game_solver::solve_local::EntRec]>,
                      toward: Option<i32>,
                      _shops: &[game_data::defs::ShopGoodDef]| {
        let c = calls.get();
        calls.set(c + 1);
        if c == 0 {
            (fake_fail(p), false)
        } else {
            match toward {
                Some(_) => (fake_exit_ok(p), true),
                None => (fake_ending(p), true),
            }
        }
    };
    let ow = game_solver::outer_witness::run_outer_witness_with(
        &g,
        &run,
        &[],
        &levels,
        PlayerSpec::from_combat(s0()),
        0,
        &[],
        &search,
        &OuterWitnessCfg { candidates: 5, max_floors: 8 },
        &layer,
    );
    assert_eq!(ow.candidates_considered, 2, "两个候选都被实化计数");
    assert_eq!(ow.attempts[0].outcome, "no_exit", "候选 0 的失败必须留痕");
    assert_eq!(ow.attempts[0].failed_floor, Some(51));
    assert_eq!(ow.attempts[1].outcome, "witness", "候选 1 成功");
    assert_eq!(ow.status, "witness");
    assert_eq!(ow.floors.len(), 2);
    assert!(ow.final_state.ending_done);
}

/// 全部候选失败 ⇒ blocked 带末次 {floor, reason}，绝不包装成 witness。
#[test]
fn s4a_all_candidates_fail_is_named_blocked() {
    let (g, run, levels) = two_floor_setup();
    let search = fake_frontier(&["f51:a0", "f52:b0"], 7, 300);
    let layer = |_input: &SolveInput,
                 _entry: (i32, i32),
                 p: &PlayerSpec,
                 _fl: u128,
                 _ents: Option<&[game_solver::solve_local::EntRec]>,
                 _toward: Option<i32>,
                 _shops: &[game_data::defs::ShopGoodDef]| (fake_fail(p), false);
    let ow = game_solver::outer_witness::run_outer_witness_with(
        &g,
        &run,
        &[],
        &levels,
        PlayerSpec::from_combat(s0()),
        0,
        &[],
        &search,
        &OuterWitnessCfg { candidates: 3, max_floors: 8 },
        &layer,
    );
    assert_eq!(ow.status, "blocked");
    assert!(!ow.final_state.ending_done);
    assert!(ow.floors.is_empty());
    let b = ow.blocked.expect("blocked 具名");
    assert_eq!(b.floor, 51);
    assert_eq!(b.reason, "no_exit");
    assert_eq!(ow.attempts.len(), 1);
}
