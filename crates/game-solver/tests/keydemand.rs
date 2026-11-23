//! 【keydemand 判据】`crates/game-solver/src/keydemand.rs` 只读分析模块——
//! 「每层从入场格走到任一上行出口，最少需要多少把各色钥匙」。
//!
//! 判据（任务书 6 条，先红后绿）：
//! 1. `keydemand_f4_requires_two_yellow`：f4 **2** 把黄钥（0/1 不可达、2 可达）——
//!    独立 python BFS 复核数；若自验不是 2，报告写差异，**不改判据**。
//! 2. `keydemand_floor_without_up_exit_is_no_up_exit`：f49 / f50 ⇒ `NoUpExit`。
//! 3. `keydemand_floor_without_entry_is_no_entry`：f1（54 层中唯一的「有 ty7 无 ty8」层）
//!    ⇒ `NoEntry`，且 `entry == None`（**不许静默回落 `level.start`**）。
//! 4. `keydemand_no_doors_zero`：无门层 ⇒ `needed = (0,0,0)`。
//! 5. `keydemand_door_budget_is_exact`：走廊 + 两黄门 + 一黄钥 ⇒ 需求 1（层内捡 1 把）；
//!    删掉拾取 ⇒ 需求 2（反证「层内拾取」真被建模）。
//! 6. `keydemand_rejects_too_many_doors`：>24 扇门 ⇒ `TooManyDoors` 具名拒绝。
//!
//! 钥匙向量口径 = 任务书 `(y, b, r)`（黄/蓝/红）；入场格=该层 ty8；出口=该层全部 ty7。
use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_data::defs::EnemyDef;
use game_solver::keydemand::{self, FloorDemand, FloorStatus};
use game_solver::solve_local::{EntSpec, LevelSpec, PlayerSpec};
use std::path::PathBuf;

// ---------------------------------------------------------------- 真数据装载（与 tests/witness.rs 同式）

fn nums(text: &str) -> Vec<i32> {
    text.lines()
        .map(|l| l.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().expect("夹具整数"))
        .collect()
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

/// 真数据单层（floor=1..=54；装载路径 = `tests/witness.rs::real_levels`）。
fn real_level(f: i32) -> LevelSpec {
    let si = load_all_ingest();
    let (base, scaled) = enemy_tables();
    let tile_walk = nums(include_str!(
        "../../game-core/tests/fixtures/tile_walkability.txt"
    ));
    game_solver::solve_local::load_floor(
        &si,
        f as usize,
        (6, 11),
        PlayerSpec::from_combat(s0()),
        scaled,
        base,
        &tile_walk,
    )
    .expect("真数据层装载")
    .level
    .expect("真数据层图")
}

fn analyze_real(f: i32) -> FloorDemand {
    keydemand::analyze_floor(f, &real_level(f))
}

// ---------------------------------------------------------------- 手工夹具

fn ent(ty: i32, x: i32, y: i32, param: i32) -> EntSpec {
    EntSpec { ty, x, y, param }
}

fn level(cols: i32, rows: i32, terrain: Vec<bool>, entities: Vec<EntSpec>) -> LevelSpec {
    LevelSpec {
        cols,
        rows,
        terrain,
        start: (0, 0),
        entities,
        exits: Vec::new(),
        player: PlayerSpec::from_combat(s0()),
        enemies: Vec::new(),
        enemy_base: Vec::new(),
        flags: 0,
    }
}

/// `n`×1 全可走走廊（y=0 单行）。
fn corridor(n: i32) -> LevelSpec {
    level(n, 1, vec![true; n as usize], Vec::new())
}

// ---------------------------------------------------------------- 判据 1

#[test]
fn keydemand_f4_requires_two_yellow() {
    let lv = real_level(4);
    let (entry, ups) = keydemand::entry_and_up_exits(&lv);
    let entry = entry.expect("f4 有 ty8（入场格不许回落 start）");
    assert!(!ups.is_empty(), "f4 有 ty7");
    assert!(
        !keydemand::reachable_with_entry(&lv, entry, &ups, [0, 0, 0]),
        "f4 0 把黄钥应不可达"
    );
    assert!(
        !keydemand::reachable_with_entry(&lv, entry, &ups, [1, 0, 0]),
        "f4 1 把黄钥应不可达"
    );
    assert!(
        keydemand::reachable_with_entry(&lv, entry, &ups, [2, 0, 0]),
        "f4 2 把黄钥应可达"
    );
    let d = keydemand::analyze_floor(4, &lv);
    assert_eq!(d.status, FloorStatus::Ok, "f4 状态");
    assert_eq!(d.needed, Some([2, 0, 0]), "f4 最小入场钥匙向量（y,b,r）");
    assert_eq!(d.min_needed_y(), Some(2));
    assert_eq!(d.min_needed_b(), Some(0));
    assert_eq!(d.min_needed_r(), Some(0));
}

// ---------------------------------------------------------------- 判据 2

#[test]
fn keydemand_floor_without_up_exit_is_no_up_exit() {
    for f in [49, 50] {
        let d = analyze_real(f);
        assert_eq!(d.status, FloorStatus::NoUpExit, "f{f} 无 ty7");
        assert!(d.up_exits.is_empty(), "f{f} 出口集必须为空");
        assert_eq!(d.needed, None, "f{f} 无出口就没有 needed");
        assert!(d.pareto.is_empty(), "f{f} 无出口就没有可行向量");
    }
    // f49 有 ty8、f50 两者皆无：入口是否取到不影响 NoUpExit 判定
    assert!(analyze_real(49).entry.is_some(), "f49 有 ty8");
    assert_eq!(analyze_real(50).entry, None, "f50 无 ty8");
}

// ---------------------------------------------------------------- 判据 3

#[test]
fn keydemand_floor_without_entry_is_no_entry() {
    let d = analyze_real(1);
    assert_eq!(d.status, FloorStatus::NoEntry, "f1 有 ty7 无 ty8");
    assert_eq!(d.entry, None, "无 ty8 ⇒ entry=None，不许回落到 level.start");
    assert_eq!(d.needed, None);
    assert!(!d.up_exits.is_empty(), "f1 的 ty7 仍应被列出");

    // 夹具复核：start 可走但无 ty8 ⇒ 依然 NoEntry（回落就会被这条抓住）
    let mut lv = corridor(3);
    lv.entities.push(ent(7, 2, 0, 2));
    let d = keydemand::analyze_floor(1, &lv);
    assert_eq!(d.status, FloorStatus::NoEntry);
    assert_eq!(d.entry, None);
}

// ---------------------------------------------------------------- 判据 4

#[test]
fn keydemand_no_doors_zero() {
    let mut lv = corridor(3);
    lv.entities.push(ent(8, 0, 0, 1)); // 入场
    lv.entities.push(ent(7, 2, 0, 2)); // 上行出口
    let d = keydemand::analyze_floor(3, &lv);
    assert_eq!(d.status, FloorStatus::Ok);
    assert_eq!(d.needed, Some([0, 0, 0]));
    assert_eq!(d.pareto, vec![[0, 0, 0]]);
    assert_eq!(d.doors, 0);
    assert_eq!(d.key_pickups, 0);
    assert!(!d.grid_bound);
}

// ---------------------------------------------------------------- 判据 5

#[test]
fn keydemand_door_budget_is_exact() {
    // 5×1：0=ty8 入场、1=黄钥、2=黄门、3=黄门、4=ty7 出口
    let mk = |with_pick: bool| {
        let mut lv = corridor(5);
        lv.entities.push(ent(8, 0, 0, 0));
        if with_pick {
            lv.entities.push(ent(26, 1, 0, 0));
        }
        lv.entities.push(ent(1, 2, 0, 0));
        lv.entities.push(ent(1, 3, 0, 0));
        lv.entities.push(ent(7, 4, 0, 0));
        lv
    };
    let with_pick = keydemand::analyze_floor(5, &mk(true));
    assert_eq!(with_pick.status, FloorStatus::Ok);
    assert_eq!(with_pick.doors, 2);
    assert_eq!(with_pick.key_pickups, 1);
    assert_eq!(
        with_pick.needed,
        Some([1, 0, 0]),
        "层内能捡 1 把 ⇒ 入场只要 1 把"
    );

    let no_pick = keydemand::analyze_floor(5, &mk(false));
    assert_eq!(no_pick.status, FloorStatus::Ok);
    assert_eq!(no_pick.key_pickups, 0);
    assert_eq!(
        no_pick.needed,
        Some([2, 0, 0]),
        "删掉拾取 ⇒ 入场要 2 把（反证层内拾取真被建模）"
    );
    assert_ne!(with_pick.needed, no_pick.needed);
}

// ---------------------------------------------------------------- 判据 6

#[test]
fn keydemand_rejects_too_many_doors() {
    // 27×1：0=ty8、1..=25=黄门（25 扇 > 24）、26=ty7
    let mut lv = corridor(27);
    lv.entities.push(ent(8, 0, 0, 0));
    for x in 1..=25 {
        lv.entities.push(ent(1, x, 0, 0));
    }
    lv.entities.push(ent(7, 26, 0, 0));
    let d = keydemand::analyze_floor(6, &lv);
    assert_eq!(d.status, FloorStatus::TooManyDoors, "25 扇门 > 24 上限");
    assert_eq!(d.doors, 25);
    assert_eq!(d.needed, None, "拒绝时不给数字（不许静默近似）");
    assert!(d.pareto.is_empty());
}

// ---------------------------------------------------------------- 诚实计数（假设 2/3/4 的可见性）

#[test]
fn keydemand_assumption_counters_are_visible() {
    let mut lv = corridor(6);
    lv.entities.push(ent(8, 0, 0, 0));
    lv.entities.push(ent(83, 1, 0, 0)); // script gate：按可通行处理并计数
    lv.entities.push(ent(41, 2, 0, 0)); // 怪（cat8）：按可清处理并计数
    lv.entities.push(ent(6, 3, 0, 0)); // 火：豁免并计数
    lv.entities.push(ent(7, 5, 0, 0));
    let d = keydemand::analyze_floor(9, &lv);
    assert_eq!(d.status, FloorStatus::Ok);
    assert_eq!(d.needed, Some([0, 0, 0]));
    assert_eq!(d.gates_ignored, 1);
    assert_eq!(d.monsters_ignored, 1, "类型 41 在默认表里是 cat8 怪");
    assert_eq!(d.hazards, 1);
}

// ---------------------------------------------------------------- S6b：key_floor 口径（单源）

/// 【S6b 判据 1】`key_floor_for_up_edge` 是 key_floor 的**唯一口径**：
/// 只在上行边（`toward == floor+1`）给需求；回拾/无需求/未定义一律 None。
///
/// 真数据：f3→f4 的入场最低钥匙需求是 (2,0,0)（`keydemand_f4_requires_two_yellow` 已钉），
/// 而 f4→f3 是下行 ⇒ 必须 None —— 这正是用户裁定 m04638（钥匙是跨层动态资源、
/// 单层硬门槛会造成假不可行）的代码化。
#[test]
fn s6b_key_floor_only_on_up_edges() {
    use game_solver::objective::key_floor_for_up_edge;
    use game_solver::solve_local::SolveInput;

    let mk = |f: i32| SolveInput {
        floor: f,
        difficulty: 0,
        level: Some(real_level(f)),
    };
    let mut levels = std::collections::BTreeMap::new();
    for f in [3, 4] {
        levels.insert(f, mk(f));
    }

    // 上行 f3→f4：f3 的出层需求 = f4 的入场最低钥匙需求 = (2,0,0)。
    assert_eq!(
        key_floor_for_up_edge(&levels, 3, Some(4)),
        Some([2, 0, 0]),
        "上行边必须给出目标层需求（f4 入场 2 黄）"
    );
    // 下行回拾 f4→f3：不许强加需求。
    assert_eq!(
        key_floor_for_up_edge(&levels, 4, Some(3)),
        None,
        "回拾（下行）不许强加 key_floor —— m04638：钥匙跨层动态，单层硬门槛造不出假不可行"
    );
    // 同层/无边界：不传。
    assert_eq!(key_floor_for_up_edge(&levels, 3, Some(3)), None);
    assert_eq!(key_floor_for_up_edge(&levels, 3, None), None);
    // 需求未定义（该层不在 levels 里）⇒ None，且原因由调用方 demand_undefined 具名落盘。
    assert_eq!(
        key_floor_for_up_edge(&levels, 77, Some(78)),
        None,
        "未定义需求不许当 0 传下去"
    );
}
