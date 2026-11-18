//! 【P4 判据】witness builder（`crates/game-solver/src/witness.rs`）——把 P0 层图 + P3 出层接口
//! 串成可回放见证。
//!
//! 判据（任务书 P4 + 收口面）：
//! 1. `witness_chains_real_floors`：真数据连跑 f1→f2，**第二层入场态 == 第一层出层态**
//!    （证明接力真的在接力，不是各自从初始状态重算）；上限 2 层 ⇒ 如实 `blocked/max_floors`
//!    （**不是** `witness`：2 层跑到一半绝不充作通关）。
//! 2. `witness_synthetic_two_floor_tower_reaches_witness`：两层塔（f1 楼梯 → f2 终局）⇒
//!    `status=witness`、`ending_done=true`、`floors.len()==2`。
//! 3. `witness_refuses_to_fake_success`：第二层 `ok=false` ⇒ `blocked/no_exit`、
//!    `floors` 只到第一层成功前缀、`ending_done=false`。
//! 4. `witness_truncated_floor_is_not_consumed`：第二层 `truncated=true` ⇒ `capped`、
//!    该层**不入** `floors`、`layer_truncated=true`、`ending_done=false`。
//! 5. `witness_goal_must_point_forward`：层解存在但「出层边不指向 P0 层图的上一层」
//!    （`hit=false` 的回落解）⇒ `blocked/no_exit_toward`，绝不当前进。
//! 6. `witness_is_deterministic`：两线程（各自全新 `LAYER_MEMO`，thread_local）同输入
//!    ⇒ JSON 逐字节相同。
//! 7. `witness_json_schema_stable`：字段齐全 + 类型正确（`serde_json` 反序列化回结构体，
//!    并对 `Value` 逐层核对键集）。
//!
//! 变异自证（人工，记录在 P4 报告）：把接力改成 `cur_player = player`（丢弃 `out_player`）
//! ⇒ 判据 1 红；把 `!got.ok` 分支改成继续推进 ⇒ 判据 3 红。
use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_data::defs::EnemyDef;
use game_solver::solve_local::{
    EntRec, EntSpec, ExitKind, ExitSpec, LevelSpec, PlayerSpec, SolveInput,
};
use game_solver::witness::{
    run_witness, run_witness_with, up_line_next, Witness, WitnessCfg, WitnessStatus,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

// ---------------------------------------------------------------- 夹具（与 solve_local_trunc 同式）

fn nums(text: &str) -> Vec<i32> {
    text.lines()
        .map(|l| l.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().expect("夹具整数"))
        .collect()
}

fn enemy_tables() -> (Vec<EnemyDef>, Vec<EnemyDef>) {
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
    (base, scaled)
}

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

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn load_all_ingest() -> game_solver::ingest::SolverInput {
    let root = repo();
    let mut maplvs: Vec<Vec<u8>> = Vec::new();
    let mut sprites: Vec<Vec<u8>> = Vec::new();
    for i in 0..55 {
        maplvs.push(std::fs::read(root.join(format!("assets/raw/maplv{i}"))).unwrap());
        sprites.push(std::fs::read(root.join(format!("assets/raw/sprite{i}"))).unwrap());
    }
    let maplv_refs: Vec<&[u8]> = maplvs.iter().map(|v| v.as_slice()).collect();
    let sprite_refs: Vec<&[u8]> = sprites.iter().map(|v| v.as_slice()).collect();
    game_solver::ingest::ingest(
        &maplv_refs,
        &sprite_refs,
        include_str!("../../game-core/tests/fixtures/level_scripts.txt"),
        include_str!("../../game-core/tests/fixtures/door12.txt"),
        include_str!("../../game-core/tests/fixtures/door13.txt"),
        include_str!("../../../data/tables/enemies.json"),
        include_str!("../../../data/tables/items.json"),
        include_str!("../../../data/tables/shop_goods.json"),
    )
    .expect("ingest 55 层资源")
}

/// 真数据：f1..f2（与 `tests/solve_local.rs::fixture_floor1` 同一装载路径）。
fn real_levels(floors: &[i32]) -> BTreeMap<i32, SolveInput> {
    let si = load_all_ingest();
    let (base, scaled) = enemy_tables();
    let tile_walk = nums(include_str!(
        "../../game-core/tests/fixtures/tile_walkability.txt"
    ));
    let mut out = BTreeMap::new();
    for &f in floors {
        let inp = game_solver::solve_local::load_floor(
            &si,
            f as usize,
            (6, 11),
            PlayerSpec::from_combat(s0()),
            scaled.clone(),
            base.clone(),
            &tile_walk,
        )
        .expect("真数据层装载");
        out.insert(f, inp);
    }
    out
}

/// 8×3 走廊（仅 y=1 可走）+ 可选 ty7 上行梯 @ (7,1)，`param`=目标层；
/// 另带一条 ty8 下行梯 @ (5,1) → f1，用来满足 `landing`（进层要找「回上一层的那条梯」定落点）。
///
/// `param & 0xff = 目标层` 是引擎口径（`floorgraph` 注释引 `scene.rs:3966-3978`），
/// 于是 `up_line_next` 会把它认成「上行主线」，注入的解也能被 `stair_target` 取到边上。
fn corridor(floor: i32, up_to: Option<i32>) -> SolveInput {
    let (cols, rows) = (8, 3);
    let terrain: Vec<bool> = (0..cols * rows).map(|i| (i / cols) == 1).collect();
    let mut entities = Vec::new();
    let mut exits = Vec::new();
    if let Some(to) = up_to {
        entities.push(EntSpec { ty: 7, x: 7, y: 1, param: to });
        exits.push(ExitSpec { x: 7, y: 1, kind: ExitKind::Stair, index: 1 });
    }
    // 下行梯：`landing` 靠它认出「从上一层落进来的门」，落点取它四邻首个可行走格。
    entities.push(EntSpec { ty: 8, x: 5, y: 1, param: 1 });
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

// ---------------------------------------------------------------- 注入用壳层求解器

/// 注入壳层的一层「成功出层」：`hp+7 / gold+3 / flags=0x2`，走 (7,1) 上行梯。
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
    }
}

/// 注入壳层的一层「终局」（`Ending` 不换层）。
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
    }
}

type Solver<'a> = &'a dyn Fn(
    &SolveInput,
    (i32, i32),
    &PlayerSpec,
    u128,
    Option<&[EntRec]>,
    Option<i32>,
) -> (game_solver::solve_local::ExitSolve, bool);

fn two_floor_levels() -> BTreeMap<i32, SolveInput> {
    let mut m = BTreeMap::new();
    m.insert(1, corridor(1, Some(2)));
    m.insert(2, corridor(2, None));
    m
}

// ---------------------------------------------------------------- 1. 真数据接力

/// 真数据 f1→f2：**第二层的 `in_*` 必须逐字段等于第一层的 `out_*`**。
///
/// 这是「接力真的生效」的唯一硬证据：若把 `out_player` 丢掉改用初始态，第二层 hp 会是
/// 300 而不是 f1 的出层血（变异自证见文件头）。上限 2 层 ⇒ 如实 `blocked/max_floors`，
/// 因为 2 层不是通关——`ending_done` 必须 false。
#[test]
fn witness_chains_real_floors() {
    // 连 f3 一起装：否则 f2 出层后「目标层缺规格」会先于 max_floors 护栏触发
    // （那就测不到护栏了；model_broken 本身另有判据）。
    let levels = real_levels(&[1, 2, 3]);
    let cfg = WitnessCfg { max_floors: 2 };
    let w = run_witness(&levels, 1, (6, 11), PlayerSpec::from_combat(s0()), 0, &cfg);
    println!("[接力] status={} floors={}", w.status, w.floors.len());

    assert_eq!(w.floors.len(), 2, "f1→f2 两层都必须解出：{w:?}");
    let (a, b) = (&w.floors[0], &w.floors[1]);
    assert_eq!((a.floor, b.floor), (1, 2), "层序必须是 f1→f2：{w:?}");
    // 接力硬断言：入场态来自上一层出层态（不是初始态、不是各自重算）。
    assert_eq!(b.in_player, a.out_player, "第二层入场玩家态必须 == 第一层出层态");
    assert_eq!(b.in_hp, a.out_hp, "第二层入场血量必须 == 第一层出层血");
    assert_eq!(b.in_flags, a.out_flags, "第二层入场旗标必须 == 第一层出层旗标");
    assert_ne!(b.in_player.hp, 300, "若等于初始 300 ⇒ 接力断了（用初始状态重算）");
    // 实体表按层归属：起手层没有位图（None）；新层首入带的是**该层 sprite 全表**，
    // 绝不是上一层的活体表残留（那会让下一层凭空多出上一层的怪）。
    assert!(a.ents_in.is_none(), "起手层入场活体表必须是 None：{a:?}");
    let lv2 = levels[&2].level.as_ref().unwrap();
    let want: Vec<(i32, i32, i32, i32)> = game_solver::solve_local::sprite_full(lv2)
        .iter()
        .map(|e| (e.ty, e.x, e.y, e.param))
        .collect();
    let got: Vec<(i32, i32, i32, i32)> = b
        .ents_in
        .as_ref()
        .expect("新层首入必须带该层全表（不是空/不是上一层残留）")
        .iter()
        .map(|e| (e.ty, e.x, e.y, e.param))
        .collect();
    assert_eq!(got, want, "f2 入场活体表必须是 sprite_full(f2)");
    // 成功前缀 + 如实卡住（2 层上限不是通关）。
    assert_eq!(w.status, "blocked", "2 层上限 ⇒ 不许报 witness：{w:?}");
    assert_eq!(w.blocked.as_ref().map(|x| x.reason.as_str()), Some("max_floors"));
    assert!(!w.final_state.ending_done, "未到终局 ⇒ ending_done 必须 false");
    assert_eq!(w.meta.layers, 2);
    println!("[接力] ok: hp {} -> {} -> {}", a.in_hp, a.out_hp, b.out_hp);
}

// ---------------------------------------------------------------- 2. 两层塔通关（结构面）

#[test]
fn witness_synthetic_two_floor_tower_reaches_witness() {
    let levels = two_floor_levels();
    let solver: Solver<'_> = &|input, _entry, p, _flags, _ents, _toward| {
        if input.floor == 1 {
            (fake_exit_ok(p), true)
        } else {
            (fake_ending(p), true)
        }
    };
    let cfg = WitnessCfg { max_floors: 8 };
    let w = run_witness_with(
        &levels,
        1,
        (1, 1),
        PlayerSpec::from_combat(s0()),
        0,
        &cfg,
        solver,
    );
    println!("[两层塔] status={} floors={} steps={}", w.status, w.floors.len(), w.total_steps);
    assert_eq!(w.status, "witness", "Ending ⇒ witness：{w:?}");
    assert!(w.final_state.ending_done);
    assert_eq!(w.final_state.floor, 2);
    assert_eq!(w.floors.len(), 2);
    assert_eq!(w.floors[1].kind, "Ending");
    assert_eq!(w.floors[1].in_player.hp, 307, "第二层入场血 = 第一层出层血");
    assert_eq!(w.floors[1].in_flags, "0x2");
    assert_eq!(w.total_steps, 3, "Σ route.len() = 2 + 1");
    assert!(w.blocked.is_none());
}

// ---------------------------------------------------------------- 3. 失败层不许充成功

#[test]
fn witness_refuses_to_fake_success() {
    let levels = two_floor_levels();
    let solver: Solver<'_> = &|input, _entry, p, _flags, _ents, _toward| {
        if input.floor == 1 {
            (fake_exit_ok(p), true)
        } else {
            // 第二层：无出层（ok=false 的回显形态）
            (
                game_solver::solve_local::ExitSolve {
                    ok: false,
                    exit: None,
                    out_player: p.clone(),
                    out_flags: 0,
                    out_ents: Vec::new(),
                    route: Vec::new(),
                    out_hp: p.hp,
                    kind: None,
                    cross_edge: None,
                    truncated: false,
                    pure_pickup: false,
                },
                false,
            )
        }
    };
    let cfg = WitnessCfg { max_floors: 8 };
    let w = run_witness_with(
        &levels,
        1,
        (1, 1),
        PlayerSpec::from_combat(s0()),
        0,
        &cfg,
        solver,
    );
    println!("[拒绝假通关] status={} floors={} blocked={:?}", w.status, w.floors.len(), w.blocked);
    assert_eq!(w.status, "blocked", "ok=false ⇒ 必须 blocked：{w:?}");
    let b = w.blocked.as_ref().expect("blocked 块必须有");
    assert_eq!((b.floor, b.reason.as_str()), (2, "no_exit"));
    assert_eq!(b.layer_ok, Some(false));
    assert_eq!(b.layer_truncated, Some(false));
    assert_eq!(w.floors.len(), 1, "floors 只保留成功前缀（f1）");
    assert_eq!(w.floors[0].floor, 1);
    assert!(!w.final_state.ending_done, "卡住 ⇒ ending_done=false");
    assert_eq!(w.final_state.floor, 2);
}

// ---------------------------------------------------------------- 4. 截断层不可消费

#[test]
fn witness_truncated_floor_is_not_consumed() {
    let levels = two_floor_levels();
    let solver: Solver<'_> = &|input, _entry, p, _flags, _ents, _toward| {
        if input.floor == 1 {
            (fake_exit_ok(p), true)
        } else {
            let mut got = fake_ending(p);
            got.truncated = true; // 截断 ⇒ out_* 只作参考
            got.ok = false;
            (got, false)
        }
    };
    let cfg = WitnessCfg { max_floors: 8 };
    let w = run_witness_with(
        &levels,
        1,
        (1, 1),
        PlayerSpec::from_combat(s0()),
        0,
        &cfg,
        solver,
    );
    println!("[截断层] status={} floors={} blocked={:?}", w.status, w.floors.len(), w.blocked);
    assert_eq!(w.status, "capped", "truncated ⇒ capped（不是 blocked、更不是 witness）");
    let b = w.blocked.as_ref().expect("blocked 块必须有");
    assert_eq!((b.floor, b.reason.as_str()), (2, "truncated"));
    assert_eq!(b.layer_truncated, Some(true));
    assert_eq!(w.floors.len(), 1, "截断层绝不入 floors（不可消费）");
    assert!(!w.final_state.ending_done);
}

// ---------------------------------------------------------------- 5. 回落解不许当前进

#[test]
fn witness_goal_must_point_forward() {
    let levels = two_floor_levels();
    let solver: Solver<'_> = &|input, _entry, p, _flags, _ents, _toward| {
        if input.floor == 1 {
            // 解是通往 f3 的（层图里 f1 的上一层是 f2）⇒ hit=false（= picks 的回落解）
            let mut got = fake_exit_ok(p);
            got.cross_edge = None;
            (got, false)
        } else {
            (fake_ending(p), true)
        }
    };
    let cfg = WitnessCfg { max_floors: 8 };
    let w = run_witness_with(
        &levels,
        1,
        (1, 1),
        PlayerSpec::from_combat(s0()),
        0,
        &cfg,
        solver,
    );
    println!("[回落解] status={} blocked={:?}", w.status, w.blocked);
    assert_eq!(w.status, "blocked");
    let b = w.blocked.as_ref().unwrap();
    assert_eq!((b.floor, b.reason.as_str()), (1, "no_exit_toward"));
    assert!(w.floors.is_empty(), "回落解不得落账为前进层");
    assert!(!w.final_state.ending_done);
}

// ---------------------------------------------------------------- 6. 确定性（跨线程/跨 memo）

/// 两线程各跑一遍**真数据 f1**：`LAYER_MEMO` 是 `thread_local` ⇒ 两个线程各自冷算，
/// 若输出 JSON 逐字节相同，说明结果不依赖缓存命中的偶然顺序/残留。
#[test]
fn witness_is_deterministic() {
    let run = || {
        let levels = real_levels(&[1]);
        let cfg = WitnessCfg { max_floors: 1 };
        run_witness(&levels, 1, (6, 11), PlayerSpec::from_combat(s0()), 0, &cfg).to_json()
    };
    let h1 = std::thread::spawn(run);
    let h2 = std::thread::spawn(run);
    let (j1, j2) = (h1.join().expect("线程1"), h2.join().expect("线程2"));
    println!("[确定性] len={} == {} ? {}", j1.len(), j2.len(), j1 == j2);
    assert_eq!(j1, j2, "同输入两次（双线程）JSON 必须逐字节相同");
}

// ---------------------------------------------------------------- 7. JSON 结构稳定

#[test]
fn witness_json_schema_stable() {
    let levels = real_levels(&[1, 2, 3]);
    let cfg = WitnessCfg { max_floors: 2 };
    let w = run_witness(&levels, 1, (6, 11), PlayerSpec::from_combat(s0()), 0, &cfg);
    let json = w.to_json();

    // 1) 结构体 round-trip：字段类型/必需性由 serde 判（删字段/改类型即红）。
    let back: Witness = serde_json::from_str(&json).expect("witness JSON 必须能反序列化回结构体");
    assert_eq!(back, w, "serde round-trip 必须恒等");

    // 2) 序列化字段序 = 声明序（serde 结构体按声明序写；这是「逐字节确定」的一部分）。
    //    注意：`serde_json::Value` 的 object 是 BTreeMap（按字典序），**不能**用它验声明序，
    //    必须在原始文本上按出现位置验（同 tests/solve_local.rs 的「route 是末字段」手法）。
    fn order(json: &str, fields: &[&str]) {
        let mut last = 0usize;
        for f in fields {
            let pat = format!("\"{f}\":");
            let pos = json.find(&pat).unwrap_or_else(|| panic!("缺字段 {f}"));
            assert!(pos >= last, "字段 {f} 的声明序被打乱（pos {pos} < 上一字段 {last}）");
            last = pos;
        }
    }
    order(&json, &["status", "floors", "final", "blocked", "total_steps", "meta"]);
    order(
        &json,
        &[
            "floor", "entry", "exit", "kind", "route", "in_hp", "out_hp", "in_player",
            "out_player", "in_flags", "out_flags", "ents_in", "ents_out", "ok", "truncated",
        ],
    );

    // 3) 值层面：schema 标签、状态词表、层不变式。
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["meta"]["schema"], "xiyou/witness/v1");
    assert!(
        ["witness", "blocked", "capped", "model_broken"].contains(&v["status"].as_str().unwrap()),
        "status 词表：{}",
        v["status"]
    );
    for f in v["floors"].as_array().unwrap() {
        assert!(f["floor"].as_i64().is_some(), "floor 必须是整数");
        assert_eq!(f["entry"].as_array().map(|a| a.len()), Some(2), "entry=[x,y]");
        assert!(
            f["exit"].is_null() || f["exit"].as_array().map(|a| a.len()) == Some(2),
            "exit=[x,y] 或 null"
        );
        assert!(f["route"].as_array().is_some(), "route 必须是数组");
        assert_eq!(f["truncated"].as_bool(), Some(false), "成功层 truncated=false");
        assert_eq!(f["ok"].as_bool(), Some(true), "floors 里只放 ok 层");
        assert!(f["in_player"]["hp"].is_i64() && f["out_player"]["hp"].is_i64());
        assert!(f["in_flags"].as_str().unwrap().starts_with("0x"));
    }
    assert!(v["final"]["ending_done"].is_boolean());
    assert!(v["blocked"].is_object() || v["blocked"].is_null());
    assert!(v["total_steps"].as_i64().is_some());
}

// ---------------------------------------------------------------- 8. 上行主线来自层图

/// `up_line_next` 读的是 P0 层图（`ty7 param&0xff`），不是硬编码塔序：
/// f1→f2（真数据）、f54→f1（塔序跨段，真数据）、无上行边 ⇒ None。
#[test]
fn witness_up_line_comes_from_floor_graph() {
    let levels = real_levels(&[1, 54]);
    let lays = game_solver::floorgraph::lay_targets();
    let lv1 = levels[&1].level.as_ref().unwrap();
    let lv54 = levels[&54].level.as_ref().unwrap();
    assert_eq!(up_line_next(1, lv1, &lays), Some(2), "f1 的上行主线是 f2");
    assert_eq!(up_line_next(54, lv54, &lays), Some(1), "f54 的上行主线是 f1（塔序跨段）");

    // 合成层：没有 ty7 ⇒ None（主线终点，终局应在 Ending）。
    let none = corridor(9, None);
    assert_eq!(up_line_next(9, none.level.as_ref().unwrap(), &lays), None);
    // 状态词表与 solve_global 对齐（消费者按一套词表判）。
    assert_eq!(WitnessStatus::Witness.as_str(), "witness");
    assert_eq!(WitnessStatus::Capped.as_str(), "capped");
    assert_eq!(WitnessStatus::ModelBroken.as_str(), "model_broken");
}
