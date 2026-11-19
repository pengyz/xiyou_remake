//! 【批8】hp 可采纳上界剪枝的判据（OP 判据 1/2/3 的测试面；判据 4 见报告里的
//! `solver run` 逐字节 `cmp`）。
//!
//! 三层断言：
//! 1. **真数据、有损血**（f3、300hp 入场玩家）⇒ 剪枝真激活（`pruned_*>0`），且
//!    与关掉剪枝的结果逐字段相同（可采纳 ⇒ 不丢最优）。
//! 2. **真数据、无损血**（f3、3227hp 满装玩家）⇒ 上界恒 4027（拾取加血与实体移除
//!    严格抵消），`pruned==0`。这是对「f3 为捡药去打怪」前提的**证伪**：
//!    该玩家对 f3 全部 7 只敌人 `predict_engagement(no_counter=true)==0`（一击必杀，
//!    无反击），击杀零损 ⇒ 界永不下降。
//! 3. **无 hp 通道的层** ⇒ `Off("no_hp_sources")`，行为与不启用逐字段相同。

use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_data::defs::EnemyDef;
use game_solver::hp_bound::{hp_bound_stat, set_enabled};
use game_solver::solve_local::{
    load_floor, solve_local, EntSpec, ExitKind, ExitSpec, LevelSpec, PlayerSpec, SolveInput,
    SolveResult,
};
use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

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

fn player(hp: i32, atk: i32, def: i32) -> PlayerCombat {
    PlayerCombat {
        hp,
        atk,
        def,
        yellow_keys: 0,
        blue_keys: 0,
        red_keys: 0,
        gold: 0,
        item_types: Vec::new(),
        item_uses: Vec::new(),
        item_stack_size: 0,
    }
}

/// f3 全量资源（与 `examples/f3_bound.rs` 同源：测试侧 I/O，lib 保持纯）。
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

/// f3 层，入场格 `(2,11)`（`(2,11)` 是 ty83 门 0 的注入入场格），玩家由调用方给。
fn f3_input(pc: PlayerCombat) -> SolveInput {
    let si = load_all_ingest();
    let (base, scaled) = enemy_tables();
    let tile_walk = nums(include_str!(
        "../../game-core/tests/fixtures/tile_walkability.txt"
    ));
    let mut input = load_floor(
        &si,
        3,
        (1, 11),
        PlayerSpec::from_combat(pc),
        scaled,
        base,
        &tile_walk,
    )
    .expect("floor3 fixture 装载");
    input.level.as_mut().unwrap().start = (2, 11);
    input
}

fn with_limit(limit: usize) -> Vec<SolveResult> {
    std::env::set_var("SOLVER_LAYER_LIMIT", limit.to_string());
    Vec::new()
}

fn unpack(r: &SolveResult) -> (bool, bool, Option<i32>, Option<i32>, usize) {
    (r.reachable, r.truncated, r.max_hp, r.min_kills, r.route.len())
}

/// 【判据 1 的可实现部分】f3（300hp 默认入场）⇒ 剪枝真激活（entry+pop >0），
/// 且剪枝开/关的解逐字段相同（可采纳性：剪掉的不是最优）。
#[test]
fn hp_bound_f3_prunes_with_damage_and_keeps_optimum() {
    with_limit(20_000);
    let input = f3_input(player(300, 10, 10));

    set_enabled(true);
    let on = solve_local(&input);
    let stat = hp_bound_stat();
    set_enabled(false);
    let off = solve_local(&input);
    let stat_off = hp_bound_stat();
    set_enabled(true);

    // 【健全性主轴】先证「开/关同解」——界若低估，这里先红（M1 靶点）。
    assert_eq!(
        unpack(&on),
        unpack(&off),
        "剪枝开/关必须同解（可采纳 ⇒ 不丢最优）：on={stat:?} off={stat_off:?}"
    );
    assert!(!stat_off.enabled, "关闭后 stat 应如实标 disabled：{stat_off:?}");
    assert_eq!(stat_off.reason, "disabled_by_caller");

    assert!(stat.enabled, "f3 上界计划应启用：{stat:?}");
    assert_eq!(stat.reason, "on");
    assert_eq!(stat.slots, 2, "f3 有两个 ty83 门槽（param 0/27）");
    assert_eq!(stat.root_hp, 300);
    assert_eq!(
        stat.root_bound,
        300 + 4 * 50 + 3 * 200,
        "根态上界 = hp + 4×ty31 + 3×ty32（4027 的同构：300 入场版 1100）"
    );
    assert!(
        stat.pruned_entry > 0,
        "入队时就该剪（不能只靠 pop 端兜底）：{stat:?}"
    );
    assert!(
        stat.pruned_pop > 0,
        "pop 端也要剪（已入队态可能被后来的更优解超过）：{stat:?}"
    );
}

/// 【前提证伪】f3（3227hp/730 atk 满装入场，即 `f3_bound` 复现用的态）：
/// 全部 7 只敌人一击必杀（`predict_engagement(no_counter=true)==0`）⇒ 击杀零损
/// ⇒ `bound ≡ 4027` 恒不下降 ⇒ `pruned==0`。这里把「界为什么没剪」钉成断言，
/// 而不是悄悄留一个 enabled=true 的空剪枝。
#[test]
fn hp_bound_f3_full_player_bound_is_constant_4027() {
    with_limit(20_000);
    let mut pc = player(3227, 730, 730);
    pc.gold = 225;
    pc.item_types = vec![13];
    pc.item_uses = vec![255];
    pc.item_stack_size = 1;
    let input = f3_input(pc);

    // 引擎侧证据：f3 每只敌人的预测损血都是 0（与搜索 bump 臂同一函数）。
    let lv = input.level.as_ref().unwrap();
    for e in &lv.entities {
        if game_core::subsystem::script_host::entity_category_with_floor(e.ty, 3) == 8 {
            let loss = game_core::subsystem::battle::predict_engagement(
                &lv.player.to_combat(),
                &lv.enemies,
                &lv.enemy_base,
                e.ty,
                true,
            );
            assert_eq!(loss, 0, "f3 敌人 ty={} 对 730atk 玩家应零损", e.ty);
        }
    }

    let r = solve_local(&input);
    let stat = hp_bound_stat();
    assert!(stat.enabled, "计划仍应启用（通道齐全）：{stat:?}");
    assert_eq!(stat.root_hp, 3227);
    assert_eq!(stat.root_bound, 4027, "3227 + 800 = f3 算术上界");
    assert_eq!(
        stat.pruned_entry + stat.pruned_pop,
        0,
        "零损 ⇒ 每个可达态的 bound 恒等于 4027 ⇒ 严格 `< best` 永不成立：{stat:?}"
    );
    assert_eq!(r.truncated, true, "20k 仍截断（爆炸不是 hp 维能剪的）");
}

/// 【判据 3】无 hp 通道的层 ⇒ `Off("no_hp_sources")`，且与关剪枝同解。
#[test]
fn hp_bound_off_without_hp_sources() {
    let lv = LevelSpec {
        cols: 3,
        rows: 3,
        terrain: vec![true; 9],
        start: (0, 0),
        entities: Vec::<EntSpec>::new(),
        exits: vec![ExitSpec {
            x: 2,
            y: 2,
            kind: ExitKind::Stair,
            index: 0u8,
        }],
        player: PlayerSpec::from_combat(player(300, 10, 10)),
        enemies: Vec::new(),
        enemy_base: Vec::new(),
        flags: 0,
    };
    let input = SolveInput {
        floor: 1,
        difficulty: 0,
        level: Some(lv),
    };
    set_enabled(true);
    let on = solve_local(&input);
    let stat = hp_bound_stat();
    assert!(!stat.enabled, "无 hp 通道应 Off：{stat:?}");
    assert_eq!(stat.reason, "no_hp_sources");
    set_enabled(false);
    let off = solve_local(&input);
    set_enabled(true);
    assert_eq!(unpack(&on), unpack(&off), "Off 必须与关剪枝逐字段相同");
}
