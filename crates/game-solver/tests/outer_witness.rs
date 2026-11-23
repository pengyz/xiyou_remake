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
        key_floor_not_consulted: None,
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
        key_floor_not_consulted: None,
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
        key_floor_not_consulted: None,
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
    Option<[i32; 3]>,
) -> (game_solver::solve_local::ExitSolve, bool) {
    |input, _entry, p, _fl, _ents, toward, _shops, _kf| {
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
                      shops: &[game_data::defs::ShopGoodDef],
                      _kf: Option<[i32; 3]>| okl(input, entry, p, fl, ents, toward, shops, _kf);
    let ow = game_solver::outer_witness::run_outer_witness_with(
        &g,
        &run,
        &[],
        &levels,
        PlayerSpec::from_combat(s0()),
        0,
        &[],
        &search,
        &OuterWitnessCfg { candidates: 1, max_floors: 8, boss_target: None },
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
                      _shops: &[game_data::defs::ShopGoodDef],
                      _kf: Option<[i32; 3]>| {
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
        &OuterWitnessCfg { candidates: 5, max_floors: 8, boss_target: None },
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
                 _shops: &[game_data::defs::ShopGoodDef],
        _kf: Option<[i32; 3]>| (fake_fail(p), false);
    let ow = game_solver::outer_witness::run_outer_witness_with(
        &g,
        &run,
        &[],
        &levels,
        PlayerSpec::from_combat(s0()),
        0,
        &[],
        &search,
        &OuterWitnessCfg { candidates: 3, max_floors: 8, boss_target: None },
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

// ---------------------------------------------------------------- S4-B：全程真引擎回放

/// 引擎一致层：从 entry 沿 y=1 走到 (7,1)（逐步邻接 token），不改玩家/旗标/实体。
/// 该路线可在 corridor 夹具上被 `replay_route_expect` 逐 token 真回放。
fn real_walk_layer() -> impl Fn(
    &SolveInput,
    (i32, i32),
    &PlayerSpec,
    u128,
    Option<&[game_solver::solve_local::EntRec]>,
    Option<i32>,
    &[game_data::defs::ShopGoodDef],
    Option<[i32; 3]>,
) -> (game_solver::solve_local::ExitSolve, bool) {
    move |input, entry, p, fl, ents, toward, _shops, _kf| {
        // 出层实体表必须**真实**：首入 ents=None ⇒ 求解侧会展开 sprite_full，
        // 终态=全表原样（corridor 楼梯不被移除）。伪造空表会让回放终态不等。
        let ents_vec: Vec<game_solver::solve_local::EntRec> = match ents {
            Some(e) => e.to_vec(),
            None => game_solver::solve_local::sprite_full(input.level.as_ref().expect("夹具必有 level")),
        };
        let mut route: Vec<String> = Vec::new();
        let mut x = entry.0;
        while x != 7 {
            x += if x < 7 { 1 } else { -1 };
            route.push(format!("{x},{}", entry.1));
        }
        let got = game_solver::solve_local::ExitSolve {
            ok: true,
            exit: Some((7, 1)),
            out_player: p.clone(),
            out_flags: fl,
            out_ents: ents_vec,
            route,
            out_hp: p.hp,
            kind: if toward.is_some() { Some(ExitKind::Stair) } else { Some(ExitKind::Ending) },
            cross_edge: None,
            truncated: false,
            pure_pickup: false,
            pure_pickup_reason: None,
            key_floor_not_consulted: None,
        };
        (got, true)
    }
}

/// 【S4-B 判据 1】witness 前缀全程真引擎逐层逐 token 回放绿。
#[test]
fn s4b_replay_end_to_end() {
    let (g, run, levels) = two_floor_setup();
    let search = fake_frontier(&["f51:a0", "f52:b0"], 7, 300);
    let layer = real_walk_layer();
    let ow = game_solver::outer_witness::run_outer_witness_with(
        &g, &run, &[], &levels, PlayerSpec::from_combat(s0()), 0, &[],
        &search, &OuterWitnessCfg { candidates: 1, max_floors: 8, boss_target: None }, &layer,
    );
    assert_eq!(ow.status, "witness", "回放测试的前置：注入层产出完整 witness");
    assert_eq!(ow.floors.len(), 2);
    let res = game_solver::outer_witness::replay_outer_witness(&levels, (1, 1), &[], &ow.floors);
    assert!(res.is_ok(), "全程回放必须绿：{res:?}");
}

/// 【S4-B 判据 2】人为破坏 route 一格 ⇒ 回放红且帧号定位（visit/floor/token_index）。
#[test]
fn s4b_replay_failure_localizes() {
    let (g, run, levels) = two_floor_setup();
    let search = fake_frontier(&["f51:a0", "f52:b0"], 7, 300);
    let layer = real_walk_layer();
    let mut ow = game_solver::outer_witness::run_outer_witness_with(
        &g, &run, &[], &levels, PlayerSpec::from_combat(s0()), 0, &[],
        &search, &OuterWitnessCfg { candidates: 1, max_floors: 8, boss_target: None }, &layer,
    );
    assert_eq!(ow.status, "witness");
    // 破坏第一层 route[2]（中间格）：改成不可达坐标 (0,0)（非邻接 + 无该格）。
    // 选非首 token：证明 token_index 真·逐帧定位，不是恒 0 的假定位。
    assert!(ow.floors[0].route.len() > 2);
    ow.floors[0].route[2] = "0,0".to_string();
    let err = game_solver::outer_witness::replay_outer_witness(&levels, (1, 1), &[], &ow.floors)
        .expect_err("被破坏的路线必须回放红");
    assert_eq!(err.stage, "route");
    assert_eq!(err.visit, 0);
    assert_eq!(err.floor, 51);
    assert_eq!(err.token_index, 2, "帧号必须定位到被破坏的那一格");
    assert_eq!(err.token, "0,0");
}

// ================================================================ S5 机制2：段链分治

use game_solver::outer_chain::{run_chain_with, ChainCfg, ChainSegSpec};

/// 四层链块表：f51 a0 stair→52 + f52 b0 boss_kill(75) + f53 c0 stair→54 + f54 d0 boss_kill(74)。
/// 两段（[51,52] 锚 75@52 / [53,54] 锚 74@54）层集 partition 覆盖全楼层。
fn chain_doc() -> String {
    r#"{
  "schema": "xiyou/block-interfaces/v2",
  "floors": [
    { "floor": 51, "entry": [1,1], "blocks": [
      { "id": "a0", "cells": [[1,1],[2,1],[3,1]], "cut_doors": [], "exports": [],
        "exits": [ { "kind": "stair", "ty": 7, "cell": [3,1], "to": 52, "same_floor": false } ] }
    ] },
    { "floor": 52, "entry": [2,1], "blocks": [
      { "id": "b0", "cells": [[2,1],[3,1]], "cut_doors": [], "exports": [],
        "exits": [ { "kind": "boss_kill", "ty": 75, "cell": [3,1], "events": [], "post_victory": false, "drops": {} } ] }
    ] },
    { "floor": 53, "entry": [1,1], "blocks": [
      { "id": "c0", "cells": [[1,1],[2,1],[3,1],[4,1]], "cut_doors": [], "exports": [],
        "exits": [ { "kind": "stair", "ty": 7, "cell": [3,1], "to": 54, "same_floor": false } ] }
    ] },
    { "floor": 54, "entry": [2,1], "blocks": [
      { "id": "d0", "cells": [[2,1],[3,1]], "cut_doors": [], "exports": [],
        "exits": [ { "kind": "boss_kill", "ty": 74, "cell": [3,1], "events": [], "post_victory": false, "drops": {} } ] }
    ] }
  ]
}"#.to_string()
}

/// 真回放走层：路线沿途**真拾取** ty26 黄钥匙（出层态 = 引擎可复算的拾取结果，
/// 回放才可能绿），拾起的实体从 ents 表移除。跨段 carry 由此可观测：f51 途经
/// (3,1) 拾 1 黄 ⇒ seg1 出场 yellow=1；段2 不重置 ⇒ in_player.yellow==1。
/// toward=Some ⇒ Stair（真路线可回放）；None（段末 boss 任务）⇒ BossKill。
fn walk_mut_layer() -> impl Fn(
    &SolveInput,
    (i32, i32),
    &PlayerSpec,
    u128,
    Option<&[game_solver::solve_local::EntRec]>,
    Option<i32>,
    &[game_data::defs::ShopGoodDef],
    Option<[i32; 3]>,
) -> (game_solver::solve_local::ExitSolve, bool) {
    move |input, entry, p, fl, ents, toward, _shops, _kf| {
        let mut ents_vec: Vec<game_solver::solve_local::EntRec> = match ents {
            Some(e) => e.to_vec(),
            None => game_solver::solve_local::sprite_full(input.level.as_ref().expect("夹具必有 level")),
        };
        let mut route: Vec<String> = Vec::new();
        let mut x = entry.0;
        while x != 7 {
            x += if x < 7 { 1 } else { -1 };
            route.push(format!("{x},{}", entry.1));
        }
        let mut out = p.clone();
        // 引擎读回把 item_types 归一为 32 槽定长（[0;32]，拾取写槽）——
        // 注入层同口径，否则 replay 逐字段全等比 item_types.len() 必红。
        if out.item_types.len() < 32 {
            out.item_types.resize(32, 0);
        }
        if out.item_uses.len() < 32 {
            out.item_uses.resize(32, 0);
        }
        for tok in &route {
            let (cx, cy) = tok.split_once(',').expect("token=x,y");
            let (cx, cy): (i32, i32) = (cx.parse().unwrap(), cy.parse().unwrap());
            if let Some(pos) = ents_vec.iter().position(|e| e.ty == 26 && e.x == cx && e.y == cy) {
                out.yellow += 1; // 拾取（引擎同口径：ty26=黄钥匙）
                ents_vec.remove(pos);
            }
        }
        let got = game_solver::solve_local::ExitSolve {
            ok: true,
            exit: Some((7, 1)),
            out_player: out,
            out_flags: fl,
            out_ents: ents_vec,
            route,
            out_hp: p.hp,
            kind: if toward.is_some() { Some(ExitKind::Stair) } else { Some(ExitKind::BossKill) },
            cross_edge: None,
            truncated: false,
            pure_pickup: false,
            pure_pickup_reason: None,
            key_floor_not_consulted: None,
        };
        (got, true)
    }
}

fn chain_levels() -> BTreeMap<i32, SolveInput> {
    let mut levels = BTreeMap::new();
    levels.insert(51, corridor(51, Some(52), 1));
    levels.insert(52, corridor(52, None, 51));
    levels.insert(53, corridor(53, Some(54), 1));
    levels.insert(54, corridor(54, None, 53));
    // f51 路线中段放一枚黄钥匙（ty26）：真拾取 ⇒ carry 可观测且回放可复算。
    levels.get_mut(&51).unwrap().level.as_mut().unwrap().entities.push(EntSpec { ty: 26, x: 3, y: 1, param: 0 });
    levels
}


/// 缩放敌表夹具：75/74 可杀（atk10 > def5）且损血 0（boss atk5 ≤ def10）⇒ 存活锚点。
fn chain_enemies() -> Vec<game_data::defs::EnemyDef> {
    use game_core::combat::trait_flags_for;
    let mk = |id: i32| game_data::defs::EnemyDef { id, hp: 100, atk: 5, def: 5, gold: 0, trait_flags: trait_flags_for(id) };
    vec![mk(75), mk(74)]
}

fn chain_segs() -> Vec<ChainSegSpec> {
    vec![
        ChainSegSpec {
            name: "seg1_f51-f52".into(),
            floors: vec![51, 52],
            start: Some((51, (1, 1))),
            anchor_floor: 52,
            anchor_boss_ty: 75,
        },
        ChainSegSpec {
            name: "seg2_f53-f54".into(),
            floors: vec![53, 54],
            start: None,
            anchor_floor: 54,
            anchor_boss_ty: 74,
        },
    ]
}

/// 【S5 判据】两段拼接：段独立收敛 + 接力逐字段一致 + 拼接回放全程绿。
#[test]
fn s5_chain_two_segments_concat_replay() {
    let doc: &'static game_solver::outer::BlockInterfaces =
        Box::leak(Box::new(load_block_interfaces(&chain_doc()).unwrap()));
    let levels = chain_levels();
    let cfg = ChainCfg { max_pops: 200_000, candidates: 5, max_floors: 16 };
    let layer = walk_mut_layer();
    let rep = run_chain_with(doc, &levels, &chain_enemies(), &[], &PlayerSpec::from_combat(s0()), 0, &chain_segs(), &cfg, &layer);
    assert_eq!(rep.schema, "xiyou/outer-chain/v1");
    assert_eq!(rep.status, "witness", "两段都必须收敛（blocked={:?}）", rep.blocked);
    assert_eq!(rep.segments.len(), 2);
    for s in &rep.segments {
        assert_eq!(s.status, "witness", "段 {} 未收敛", s.name);
        assert_eq!(s.floors_solved, 2);
    }
    assert_eq!(rep.segments[0].handoff_ok, None, "首段无上段可接力");
    assert_eq!(rep.segments[1].handoff_ok, Some(true), "跨段接力必须逐字段一致");
    // 层集 partition + 顺序拼接。
    let fl: Vec<i32> = rep.floors.iter().map(|f| f.floor).collect();
    assert_eq!(fl, vec![51, 52, 53, 54]);
    // 入口格：首段显式 start；后续段 = landing（上行落在目标层的下行梯旁 = (4,1)）。
    let ent: Vec<(i32, i32)> = rep.floors.iter().map(|f| (f.entry[0], f.entry[1])).collect();
    assert_eq!(ent, vec![(1, 1), (4, 1), (4, 1), (4, 1)], "跨段入口必须由 landing 单源推出");
    // carry 可观测：f51 真拾 1 黄 ⇒ seg1 出场 yellow=1；段2 入场=段1 出场（真 carry，
    // 不重置）。段2 无拾取 ⇒ 全程 yellow=1 到末层。
    assert_eq!(rep.floors[0].out_player.yellow, 1, "f51 真拾 1 黄钥匙");
    assert_eq!(rep.floors[1].in_player.yellow, 1);
    assert_eq!(rep.floors[2].in_player.yellow, 1, "段2 入场 = 段1 出场（跨段 carry）");
    assert_eq!(rep.floors[3].out_player.yellow, 1);
    assert_eq!(rep.floors[1].out_flags, rep.floors[2].in_flags, "flags 接力一致");
    // BossKill 段末不冒充通关。
    assert!(!rep.ending_done);
    // 拼接回放全程绿（复核黄2 首验语义）。
    assert_eq!(rep.replay.as_deref(), Some("pass"), "拼接 witness 必须全程回放绿");
    // total_steps 与 total_pops 汇总。
    let steps: i32 = rep.segments.iter().map(|s| s.witness.as_ref().unwrap().total_steps).sum();
    assert_eq!(rep.total_steps, steps);
    let pops: u64 = rep.segments.iter().map(|s| s.pops).sum();
    assert_eq!(rep.total_pops, pops);
}

/// 【S5 判据】段 blocked ⇒ 停链、拼已得段、具名断点（不许伪造全程 witness）。
#[test]
fn s5_chain_blocked_named_and_prefix_only() {
    let doc: &'static game_solver::outer::BlockInterfaces =
        Box::leak(Box::new(load_block_interfaces(&chain_doc()).unwrap()));
    let levels = chain_levels();
    let cfg = ChainCfg { max_pops: 200_000, candidates: 5, max_floors: 16 };
    // seg1 首层恒失败 ⇒ seg1 blocked；**seg2 可收敛** ⇒ 「blocked 后必须断链」
    // 才可观测（若吞 blocked 继续链，seg2 的 floors 会混进拼接 ⇒ 判据红）。
    let segs = chain_segs();
    // seg2 的层被调用次数：断链后**不许再跑搜索** ⇒ 必须 0。
    let seg2_calls = std::rc::Rc::new(std::cell::Cell::new(0usize));
    let seg2_calls_c = seg2_calls.clone();
    let layer_fail_boss = move |input: &SolveInput, entry: (i32, i32), p: &PlayerSpec, fl: u128,
                                ents: Option<&[game_solver::solve_local::EntRec]>, toward: Option<i32>,
                                _shops: &[game_data::defs::ShopGoodDef],
        kf: Option<[i32; 3]>|
        -> (game_solver::solve_local::ExitSolve, bool) {
        if input.floor == 51 {
            return (fake_fail(p), false); // seg1 首层失败 ⇒ 段 blocked
        }
        if input.floor == 53 || input.floor == 54 {
            seg2_calls_c.set(seg2_calls_c.get() + 1);
        }
        let base = walk_mut_layer();
        base(input, entry, p, fl, ents, toward, &[], kf)
    };
    let rep = run_chain_with(doc, &levels, &chain_enemies(), &[], &PlayerSpec::from_combat(s0()), 0, &segs, &cfg, &layer_fail_boss);
    assert_eq!(rep.status, "blocked");
    let b = rep.blocked.as_ref().expect("blocked 必须具名");
    assert_eq!(b.seg, "seg1_f51-f52");
    assert!(!b.reason.is_empty());
    // 只拼已得段前缀（seg1 失败 ⇒ 空前缀；seg2 绝不许混入）。
    let fl: Vec<i32> = rep.floors.iter().map(|f| f.floor).collect();
    assert_eq!(fl, Vec::<i32>::new(), "blocked 后只保留已得段（此处分段失败 ⇒ 空）");
    assert_eq!(rep.segments.len(), 2, "断链后每段仍要有具名报告");
    assert_eq!(rep.segments[0].status, "blocked");
    // 复核黄3（F3）：断链后的段必须是具名 `skipped_no_handoff`，不许冒充 blocked/witness。
    assert_eq!(
        rep.segments[1].status, "skipped_no_handoff",
        "首段 blocked ⇒ 次段必须 skipped_no_handoff（segment={:?}）", rep.segments[1]
    );
    assert_eq!(seg2_calls.get(), 0, "断链后不许再跑 seg2 的搜索（跑过就不叫 skipped）");
    assert!(rep.segments[1].witness.is_none(), "未跑搜索的段 witness 必须是 null（零值不许冒充测量）");
    assert!(
        rep.segments[1].note.as_deref().unwrap_or("").contains("未跑搜索"),
        "skipped 段必须写明「未跑搜索」：{:?}", rep.segments[1].note
    );
    assert_eq!(rep.segments[0].handoff_ok, None);
    assert!(!rep.ending_done);
    // 已得段为空 ⇒ 无拼接无回放（replay=None 如实）。
    assert_eq!(rep.replay, None);
}

/// 【S5 回归】boss_target 默认 None：BossKill 末任务 ⇒ final_not_ending（默认链路无 BossKill 混入）。
#[test]
fn s5_boss_mode_off_requires_ending() {
    let (g, run, levels) = two_floor_setup();
    let search = fake_frontier(&["f51:a0", "f52:b0"], 7, 300);
    // 注入层末任务回 BossKill（假 boss 臂）——默认模式必须拒收。
    let boss_layer = |input: &SolveInput, entry: (i32, i32), p: &PlayerSpec, fl: u128,
                      ents: Option<&[game_solver::solve_local::EntRec]>, toward: Option<i32>,
                      _shops: &[game_data::defs::ShopGoodDef],
        kf: Option<[i32; 3]>|
        -> (game_solver::solve_local::ExitSolve, bool) {
        if toward.is_none() {
            let mut e = fake_ending(p);
            e.kind = Some(ExitKind::BossKill);
            return (e, true);
        }
        let _ = (input, entry, ents);
        (fake_exit_ok(p), {
            let _ = fl;
            true
        })
    };
    let ow = game_solver::outer_witness::run_outer_witness_with(
        &g, &run, &[], &levels, PlayerSpec::from_combat(s0()), 0, &[],
        &search, &OuterWitnessCfg { candidates: 1, max_floors: 8, boss_target: None }, &boss_layer,
    );
    assert_eq!(ow.status, "blocked");
    let b = ow.blocked.expect("必须具名");
    assert_eq!(b.reason, "final_not_ending");
}

/// 【S5 判据】boss_target 注入：BossKill 末任务通过，ending_done 如实 false；同夹具真 Ending ⇒ ending_done true。
#[test]
fn s5_boss_mode_on_bosskill_accepted() {
    let (g, run, levels) = two_floor_setup();
    let search = fake_frontier(&["f51:a0", "f52:b0"], 7, 300);
    let boss_layer = |input: &SolveInput, entry: (i32, i32), p: &PlayerSpec, fl: u128,
                      ents: Option<&[game_solver::solve_local::EntRec]>, toward: Option<i32>,
                      _shops: &[game_data::defs::ShopGoodDef],
        kf: Option<[i32; 3]>|
        -> (game_solver::solve_local::ExitSolve, bool) {
        if toward.is_none() {
            let mut e = fake_ending(p);
            e.kind = Some(ExitKind::BossKill);
            return (e, true);
        }
        let _ = (input, entry, ents, fl);
        (fake_exit_ok(p), true)
    };
    let ow = game_solver::outer_witness::run_outer_witness_with(
        &g, &run, &[], &levels, PlayerSpec::from_combat(s0()), 0, &[],
        &search, &OuterWitnessCfg { candidates: 1, max_floors: 8, boss_target: Some(75) }, &boss_layer,
    );
    assert_eq!(ow.status, "witness", "BossKill 段末必须收（blocked={:?}）", ow.blocked);
    assert!(!ow.final_state.ending_done, "BossKill≠通关，ending_done 必须如实 false");
}

/// 【S5 判据】携带入场态起搜：OuterState 由 PlayerSpec 直搬（held=item_types 位图）。
#[test]
fn s5_outer_state_from_player_carries() {
    let doc: &'static game_solver::outer::BlockInterfaces =
        Box::leak(Box::new(load_block_interfaces(&mini_doc()).unwrap()));
    let g = game_solver::outer::OuterGraph::build(doc, &vec![51, 52], &[]);
    let run = RunSpec {
        floors: vec![51, 52],
        start: (51, (1, 1)),
        anchor_floor: 52,
        anchor_boss_ty: 75,
        masked_exits: Vec::new(),
        combat_cost: true,
    };
    let mut p = PlayerSpec::from_combat(s0());
    p.hp = 214;
    p.gold = 88;
    p.item_types = vec![3, 9];
    let st = game_solver::outer_chain::outer_state_from_player(&g, &run, &p).expect("起点态可构造");
    assert_eq!(st.hp, 214);
    assert_eq!(st.gold, 88);
    assert_eq!(st.atk, 10);
    assert_eq!(st.def, 10);
    assert_eq!(st.held, (1u128 << 3) | (1u128 << 9), "held = item_types 位图");
    assert!(st.picked.iter().all(|&b| b == 0), "新段拾取史归零");
    assert_eq!(st.buys, 0);
}

/// S5：LB fatal（乐观门把 hp 压负）不得取消精确层解资格——打不动（killable=false）
/// 才滤除；fatal 候选必须被尝试，由精确层解裁定存活。
#[test]
fn s5_lb_fatal_candidate_still_attempted() {
    let doc: &'static game_solver::outer::BlockInterfaces =
        Box::leak(Box::new(load_block_interfaces(&mini_doc()).unwrap()));
    let graph = game_solver::outer::OuterGraph::build(doc, &[51, 10], &[]);
    let run = RunSpec {
        floors: vec![51, 10],
        start: (51, (1, 11)),
        anchor_floor: 10,
        anchor_boss_ty: 75,
        masked_exits: Vec::new(),
        combat_cost: true,
    };
    let levels: BTreeMap<i32, SolveInput> = BTreeMap::new();
    let mut search = fake_frontier(&["f51:f51b0", "f10:f10b0"], 7, -4335);
    // 唯一候选：LB 口径 fatal（hp 下界为负）但可杀。
    search.frontier[0].fatal = true;
    let layer = real_walk_layer();
    let cfg = OuterWitnessCfg { candidates: 3, max_floors: 16, boss_target: Some(75) };
    let ow = game_solver::outer_witness::run_outer_witness_with(
        &graph, &run, &chain_enemies(), &levels, PlayerSpec::from_combat(s0()), 0, &[], &search, &cfg, &layer,
    );
    assert!(
        ow.candidates_considered >= 1,
        "fatal(=LB 口径) 候选必须被尝试，got blocked={:?} attempts={:?}",
        ow.blocked.as_ref().map(|b| b.reason.clone()),
        ow.attempts.iter().map(|a| a.outcome.clone()).collect::<Vec<_>>()
    );
}

/// S5 缺口3 判据：driver 跨层 landing 必须查**目标层**的梯（与 witness.rs replay
/// 同一真相源）。出发层只有 ty7、目标层有 ty8 ⇒ 传出发层的旧实现假 landing_missing。
#[test]
fn s5_driver_landing_uses_target_floor_stairs() {
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
        combat_cost: true,
    };
    let mut levels = BTreeMap::new();
    // 出发层 f51 剥 ty8（上行出发层无 ty8 ⇒ 旧实现（查出发层）必然 landing_missing）；
    // 目标层 f52 保留 ty8（真数据 f52 有下行梯）。
    let mut l51 = corridor(51, Some(52), 999);
    if let Some(lv) = l51.level.as_mut() {
        lv.entities.retain(|e| e.ty != 8);
        lv.exits.retain(|e| !(e.x == 5 && e.y == 1));
    }
    levels.insert(51, l51);
    levels.insert(52, corridor(52, None, 51));
    let search = fake_frontier(&["f51:a0", "f52:b0"], 7, 300);
    let layer = real_walk_layer();
    let ow = game_solver::outer_witness::run_outer_witness_with(
        &g, &run, &chain_enemies(), &levels, PlayerSpec::from_combat(s0()), 0, &[],
        &search, &OuterWitnessCfg { candidates: 1, max_floors: 8, boss_target: Some(75) }, &layer,
    );
    assert_eq!(
        ow.floors.len(), 2,
        "两层都必须解出（f51 解出后跨层落 f52 继续），got blocked={:?} attempts={:?}",
        ow.blocked.as_ref().map(|b| (b.reason.clone(), b.detail.clone())),
        ow.attempts.iter().map(|a| a.outcome.clone()).collect::<Vec<_>>()
    );
}

// ================================================================ S5 复核 F4/F7：证据进 JSON

/// 段内解出 s 层的注入层（fails on f52 anchor）——seg1 解出 f51 后在锚层断链，
/// 复现真数据形状：`segments[0].witness.floors` 有层、链级 `floors` 为空。
fn layer_fail_at_anchor() -> impl Fn(
    &SolveInput,
    (i32, i32),
    &PlayerSpec,
    u128,
    Option<&[game_solver::solve_local::EntRec]>,
    Option<i32>,
    &[game_data::defs::ShopGoodDef],
    Option<[i32; 3]>,
) -> (game_solver::solve_local::ExitSolve, bool) {
    let base = walk_mut_layer();
    move |input, entry, p, fl, ents, toward, shops, kf| {
        if input.floor == 52 {
            return (fake_fail(p), false); // 锚层失败 ⇒ 段 blocked、链断
        }
        base(input, entry, p, fl, ents, toward, shops, kf)
    }
}

/// 【S5 F4/F7 判据】blocked 链的前缀证据必须**进 JSON 顶层**：
/// `prefix_floors` / `prefix_steps` 具名承载已解前缀；`floors`/`total_steps` 仍是
/// **全链**语义（blocked ⇒ 空/0，不许把前缀冒充全链）；`prefix_replay` 是结构化
/// 对象（不是 `replay:null` 那种证据丢失），成功也必须明确 `status="pass"`。
///
/// 修复前红：JSON 无 prefix_floors/prefix_steps/prefix_replay 三键，前缀回放只是
/// `bin/solver.rs` 的 stdout（复核 F4/F7 实测）。
#[test]
fn s5_chain_prefix_evidence_lands_in_json() {
    let doc: &'static game_solver::outer::BlockInterfaces =
        Box::leak(Box::new(load_block_interfaces(&chain_doc()).unwrap()));
    let levels = chain_levels();
    let cfg = ChainCfg { max_pops: 200_000, candidates: 5, max_floors: 16 };
    let layer = layer_fail_at_anchor();
    let rep = run_chain_with(
        doc, &levels, &chain_enemies(), &[], &PlayerSpec::from_combat(s0()), 0,
        &chain_segs(), &cfg, &layer,
    );
    assert_eq!(rep.status, "blocked", "seg1 锚层失败 ⇒ 链必须 blocked");
    let v = serde_json::to_value(&rep).expect("ChainReport 必须可序列化");

    // ① 前缀层序列/步数：顶层具名字段，不许只藏在 segments[0].witness.floors。
    let pf = v
        .get("prefix_floors")
        .and_then(|x| x.as_array())
        .unwrap_or_else(|| panic!("blocked 时顶层必须有 prefix_floors：{v}"));
    assert_eq!(pf.len(), 1, "seg1 只解出 f51（锚层 f52 失败）：{v}");
    assert_eq!(pf[0]["floor"], 51);
    assert_eq!(
        v.get("prefix_steps").and_then(|x| x.as_i64()),
        Some(rep.segments[0].witness.as_ref().unwrap().total_steps as i64),
        "prefix_steps 必须与段内 witness 同步：{v}"
    );
    // floors/total_steps 保持「全链」语义：blocked ⇒ 不许把前缀冒充全链。
    assert_eq!(v["floors"].as_array().map(Vec::len), Some(0), "floors 仍是全链语义：{v}");
    assert_eq!(v["total_steps"], 0, "total_steps 仍是全链语义：{v}");

    // ② 前缀回放：结构化记录，成功也明确 pass。
    let r = v
        .get("prefix_replay")
        .unwrap_or_else(|| panic!("prefix_replay 必须存在（不许丢成 replay:null）：{v}"));
    assert_eq!(r["status"], "pass", "前缀 1 层回放必须绿：{r}");
    for k in ["visit", "floor", "token_index", "token", "stage", "reason"] {
        assert!(r.get(k).is_some(), "prefix_replay 缺字段 {k}: {r}");
    }
    assert_eq!(r["floor"], 51, "pass 时 floor=末次回放层：{r}");
    assert_eq!(r["visit"], 0, "pass 时 visit=末次访问序号（0 起）：{r}");
    // 段内 witness 仍保留（证据只提升，不删除）。
    assert_eq!(rep.segments[0].witness.as_ref().unwrap().floors.len(), 1);
}

/// 【S5 F4/F7 判据】前缀回放**失败**必须在 JSON 里可定位到 token：
/// status=fail + visit/floor/token_index/token/stage/reason 六字段。
/// 修复前红：这些字只在 stdout 出现，JSON 里 no_match/token_index 各 0 次。
#[test]
fn s5_chain_prefix_replay_failure_named_in_json() {
    let doc: &'static game_solver::outer::BlockInterfaces =
        Box::leak(Box::new(load_block_interfaces(&chain_doc()).unwrap()));
    let levels = chain_levels();
    let cfg = ChainCfg { max_pops: 200_000, candidates: 5, max_floors: 16 };
    let base = layer_fail_at_anchor();
    // f51 层解「成功」但 route 被篡改为不可达 token ⇒ witness 层落盘，回放必红。
    let layer = move |input: &SolveInput, entry: (i32, i32), p: &PlayerSpec, fl: u128,
                      ents: Option<&[game_solver::solve_local::EntRec]>, toward: Option<i32>,
                      shops: &[game_data::defs::ShopGoodDef],
        kf: Option<[i32; 3]>|
        -> (game_solver::solve_local::ExitSolve, bool) {
        let (mut got, ok) = base(input, entry, p, fl, ents, toward, shops, kf);
        if input.floor == 51 && ok {
            got.route = vec!["9,9".to_string()]; // 非邻接位移（x 从 1 直接跳 9）
        }
        (got, ok)
    };
    let rep = run_chain_with(
        doc, &levels, &chain_enemies(), &[], &PlayerSpec::from_combat(s0()), 0,
        &chain_segs(), &cfg, &layer,
    );
    assert_eq!(rep.status, "blocked");
    let v = serde_json::to_value(&rep).expect("ChainReport 必须可序列化");
    let r = v
        .get("prefix_replay")
        .unwrap_or_else(|| panic!("prefix_replay 必须存在：{v}"));
    assert_eq!(r["status"], "fail", "前缀回放必须如实红：{r}");
    assert_eq!(r["visit"], 0, "首个访问：{r}");
    assert_eq!(r["floor"], 51);
    assert_eq!(r["token_index"], 0, "失败必须定位到 token 下标：{r}");
    assert_eq!(r["token"], "9,9");
    assert_eq!(r["stage"], "route");
    let reason = r["reason"].as_str().unwrap_or("");
    assert!(reason.contains("非邻接位移"), "失败原因必须具名：{r}");
    // 前缀层仍要在顶层可读（失败不影响已解前缀的落盘）。
    assert_eq!(v["prefix_floors"].as_array().map(Vec::len), Some(1), "{v}");
}
