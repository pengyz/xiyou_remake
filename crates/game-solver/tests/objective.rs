//! 【P10 判据】出层目标分层契约（`crates/game-solver/src/objective.rs`）。
//!
//! 用户裁定（2026-11-20 修订版）：**单层只做软契约**——hp 大者优先 → 同 hp gold 多者
//! 优先 → steps 少者优先；demand（下一层钥匙需求）是**纯诊断字段**，绝不进入比较、
//! 过滤或可行性判定。钥匙是跨层动态资源（回早层拾取/商店购买/换钥匙），把「带够钥匙
//! 出本层」当单层硬约束会制造假不可行。
//!
//! 判据：
//! 1. `demand_fields_are_diagnostic_only`：血多钥少的解必须仍被选中（demand 若被接回
//!    比较，此判据红——变异 M1）。
//! 2. `hp_beats_gold`：血优先于金（变异 M2：gold 提到 hp 前则红）。
//! 3. `demand_same_hp_prefers_more_gold`：同 hp ⇒ 金多者优先。
//! 4. `key_shortfall_reported_not_blocking`（变异 M4：把缺口改成阻断则红）。
//! 5. `witness_floor_records_demand_fields`。
//! 6. `next_demand_*`：需求来源（逐色最大 / Undefined）。
//! 7. `f3_exit_reports_keys_out`（真数据集成）。
//! 8. `f29_status_is_blocked_by_assumption`（keydemand 具名化，唯一允许动 keydemand 的点）。

use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_data::defs::EnemyDef;
use game_solver::objective::{cmp_exit, cmp_exit_floor, next_demand, ExitScore};
use game_solver::solve_local::{EntSpec, LevelSpec, PlayerSpec, SolveInput};
use game_solver::witness::{run_witness, WitnessCfg};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::path::PathBuf;

// ---------------------------------------------------------------- 夹具（与 tests/witness.rs 同式）

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
    let read = |p: std::path::PathBuf| std::fs::read(&p).unwrap_or_else(|e| panic!("read {p:?}: {e}"));
    let mut maplvs = Vec::new();
    let mut sprites = Vec::new();
    for i in 0..55 {
        maplvs.push(read(root.join(format!("assets/raw/maplv{i}"))));
        sprites.push(read(root.join(format!("assets/raw/sprite{i}"))));
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

fn ent(ty: i32, x: i32, y: i32, param: i32) -> EntSpec {
    EntSpec { ty, x, y, param }
}

/// 钥匙效果表（game-core/src/enums.rs:185-191）：门 ty1=黄 / ty2=红 / ty3=蓝，
/// 拾取 26=黄 / 27=红 / 28=蓝；keydemand 向量口径 **(y,b,r)**。
/// `slot` 按向量序：0=黄、1=蓝、2=红。
fn door_ty(slot: usize) -> i32 {
    [1, 3, 2][slot]
}
fn key_ty(slot: usize) -> i32 {
    [26, 28, 27][slot]
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

// ---------------------------------------------------------------- 判据 1/2/3：软契约比较序

fn score(hp: i32, gold: i32, keys: [u8; 3], steps: i32) -> ExitScore {
    ExitScore { hp, gold, keys, steps }
}

/// 判据 1（变异 M1 的对象）：demand 字段不进比较——血多钥少的解必须赢。
#[test]
fn demand_fields_are_diagnostic_only() {
    // A：血多、黄钥 0；B：血少、黄钥 2（若有人把 keys 接回比较，B 会被选 ⇒ 红）
    let a = score(4100, 0, [0, 0, 0], 90);
    let b = score(4000, 0, [2, 0, 0], 88);
    assert_eq!(cmp_exit(&a, &b), Ordering::Less, "hp 大者优先，keys 不进比较");
    assert_eq!(cmp_exit(&b, &a), Ordering::Greater);
    // 同 hp 时 keys 也不许插到 gold 前（M1 变体：keys 层放在 hp 之后 gold 之前）。
    let e = score(4027, 500, [0, 0, 0], 88);
    let f = score(4027, 100, [2, 0, 0], 88);
    assert_eq!(cmp_exit(&e, &f), Ordering::Less, "同 hp ⇒ 金多者赢，keys 不翻盘");
}

/// 判据 2（变异 M2 的对象）：血优先于金——「血少金多」绝不许翻盘。
#[test]
fn hp_beats_gold() {
    let a = score(4100, 0, [0, 0, 0], 90);
    let b = score(4000, 9999, [0, 0, 0], 88);
    assert_eq!(cmp_exit(&a, &b), Ordering::Less, "血大者优先于任何 gold 优势");
}

/// 判据 3：同 hp ⇒ 金多者优先；同 hp 同 gold ⇒ 步少者优先。
#[test]
fn demand_same_hp_prefers_more_gold() {
    let a = score(4027, 500, [0, 0, 0], 90);
    let b = score(4027, 100, [0, 0, 0], 88);
    assert_eq!(cmp_exit(&a, &b), Ordering::Less, "同 hp 金多者优先");
    let c = score(4027, 500, [0, 0, 0], 88);
    assert_eq!(cmp_exit(&c, &a), Ordering::Less, "同 hp 同 gold 步少者优先");
    let d = score(4027, 500, [0, 0, 0], 90);
    assert_eq!(cmp_exit(&a, &d), Ordering::Equal);
}

/// 判据 3 的接线面（`pick_exit`/`better_exit` 用同一比较器）在
/// `crates/game-solver/src/solve_local.rs` 的 `#[cfg(test)]` 单测里钉
/// （LayerGoal 是 crate 私有构造面，引擎级等价锚由 `f3_exit_reports_keys_out`
/// 的 `out_hp == 4027` 回归锚承担）。

// ---------------------------------------------------------------- 判据 6：next_demand

/// 无门后继 ⇒ needed=(0,0,0)（Known）。
#[test]
fn next_demand_zero_for_open_successor() {
    let mut levels = BTreeMap::new();
    levels.insert(
        1,
        SolveInput { floor: 1, difficulty: 0, level: Some(corridor_with_up(2)) },
    );
    levels.insert(2, SolveInput { floor: 2, difficulty: 0, level: Some(successor_floor(2, [0, 0, 0])) });
    let d = next_demand(&levels, 1);
    assert_eq!(d.keys, [0, 0, 0]);
    assert!(
        matches!(d.state, game_solver::objective::DemandState::Known),
        "无门后继 ⇒ Known(0,0,0)"
    );
}

/// 多后继 ⇒ 逐色最大：f1→{f2(需1黄), f3(需1蓝)} ⇒ (1,1,0)。
#[test]
fn next_demand_multiple_successors_takes_componentwise_max() {
    let mut levels = BTreeMap::new();
    levels.insert(
        1,
        SolveInput { floor: 1, difficulty: 0, level: Some(two_up_corridor()) },
    );
    levels.insert(2, SolveInput { floor: 2, difficulty: 0, level: Some(successor_floor(2, [1, 0, 0])) });
    levels.insert(3, SolveInput { floor: 3, difficulty: 0, level: Some(successor_floor(3, [0, 0, 1])) });
    let d = next_demand(&levels, 1);
    assert_eq!(d.keys, [1, 0, 1], "逐色最大（保守）：max((1,0,0),(0,0,1))=(1,0,1)；蓝=ty3/28");
}

/// 后继缺规格/后继 keydemand 非 Ok ⇒ Undefined（诊断，不进比较）。
#[test]
fn next_demand_undefined_for_missing_successor_spec() {
    let mut levels = BTreeMap::new();
    levels.insert(
        1,
        SolveInput { floor: 1, difficulty: 0, level: Some(corridor_with_up(2)) },
    );
    // f2 不在表里 ⇒ Undefined
    let d = next_demand(&levels, 1);
    assert!(
        matches!(&d.state, game_solver::objective::DemandState::Undefined(r) if r.contains("model_missing")),
        "后继无规格 ⇒ Undefined(model_missing)，实际 {:?}",
        d.state
    );
}

/// 后继 keydemand 结构性无数字（no_entry）⇒ Undefined(no_entry)。
#[test]
fn next_demand_undefined_propagates_keydemand_status() {
    let mut levels = BTreeMap::new();
    // f1 有上行梯 → f2；f2 无 ty8 入场（keydemand::NoEntry）⇒ 需求数字未定义
    levels.insert(
        1,
        SolveInput { floor: 1, difficulty: 0, level: Some(corridor_with_up(2)) },
    );
    let mut f2 = successor_floor(2, [1, 0, 0]);
    f2.entities.retain(|e| e.ty != 8); // 拆掉入场
    levels.insert(2, SolveInput { floor: 2, difficulty: 0, level: Some(f2) });
    let d = next_demand(&levels, 1);
    assert!(
        matches!(&d.state, game_solver::objective::DemandState::Undefined(r) if r.contains("no_entry")),
        "后继 keydemand=no_entry ⇒ Undefined(no_entry)，实际 {:?}",
        d.state
    );
}

// ---------------------------------------------------------------- 判据 4/5：witness 诊断字段

/// f1 出层 0 黄钥、f2 需 2 黄 ⇒ f1 记 `key_shortfall{need:[2,0,0],have:[0,0,0]}`，
/// 但 **ok 仍 true、照常落账**（诊断，不阻断；变异 M4 的对象）。
#[test]
fn key_shortfall_reported_not_blocking() {
    let mut levels = BTreeMap::new();
    // f1：入口 + ty7 → f2（无黄钥可捡）
    levels.insert(1, SolveInput { floor: 1, difficulty: 0, level: Some(corridor_with_up(2)) });
    // f2：入口 + 2 黄门 + ty7 → f3 ⇒ keydemand needed=(2,0,0)
    levels.insert(2, SolveInput { floor: 2, difficulty: 0, level: Some(successor_floor(2, [2, 0, 0])) });
    let cfg = WitnessCfg { max_floors: 2 };
    let w = run_witness(&levels, 1, (1, 1), PlayerSpec::from_combat(s0()), 0, &[], &cfg);
    assert!(!w.floors.is_empty(), "f1 必须被推进并落账");
    let f1 = &w.floors[0];
    assert_eq!(f1.floor, 1);
    assert!(f1.ok, "key_shortfall 是诊断位，绝不允许改写 ok");
    assert_eq!(f1.keys_out, [0, 0, 0]);
    let sf = f1.key_shortfall.as_ref().expect("f1 必须报 key_shortfall");
    assert_eq!(sf.need, [2, 0, 0]);
    assert_eq!(sf.have, [0, 0, 0]);
    assert_eq!(f1.demand, [2, 0, 0]);
    assert!(f1.demand_undefined.is_none());
}

/// JSON 每层都有 demand / demand_undefined / keys_out（f29 口径之外的新观测面）。
#[test]
fn witness_floor_records_demand_fields() {
    let mut levels = BTreeMap::new();
    levels.insert(1, SolveInput { floor: 1, difficulty: 0, level: Some(corridor_with_up(2)) });
    levels.insert(2, SolveInput { floor: 2, difficulty: 0, level: Some(successor_floor(2, [1, 0, 0])) });
    let cfg = WitnessCfg { max_floors: 2 };
    let w = run_witness(&levels, 1, (1, 1), PlayerSpec::from_combat(s0()), 0, &[], &cfg);
    let v: serde_json::Value =
        serde_json::from_str(&w.to_json()).expect("witness JSON 解析");
    for f in v["floors"].as_array().expect("floors 数组") {
        assert!(f.get("demand").is_some(), "每层必有 demand: {f}");
        assert!(f.get("keys_out").is_some(), "每层必有 keys_out: {f}");
        assert!(f.get("demand_undefined").is_some(), "每层必有 demand_undefined（可 null）: {f}");
    }
    assert_eq!(v["floors"][0]["demand"], serde_json::json!([1, 0, 0]), "f2 需 1 黄 ⇒ f1 层 demand=(1,0,0)");
}

// ---------------------------------------------------------------- 判据 7：f3 真数据集成

/// 真数据 f3 出层必须带 `keys_out`（数值正确即可，不断言 ≥2——demand 是诊断位）；
/// 且 out_hp 仍 4027（软契约 hp 大者优先，加了 gold 软层不许把血压下去）。
/// 口径与 tests/exact_pickup.rs::exact_dp_f3_reaches_4027 一致：从 f3 链路入场态
/// （entry (2,11)、hp 3227/攻防 730、flags 0xe337be_00000000、真商品表）起 witness。
#[test]
fn f3_exit_reports_keys_out() {
    let si = load_all_ingest();
    let (base, scaled) = enemy_tables();
    let tile_walk = nums(include_str!(
        "../../game-core/tests/fixtures/tile_walkability.txt"
    ));
    let inp = game_solver::solve_local::load_floor(
        &si,
        3,
        (2, 11),
        chain_player_f3(),
        scaled,
        base,
        &tile_walk,
    )
    .expect("真数据层装载");
    let mut levels = BTreeMap::new();
    levels.insert(3, inp);
    let cfg = WitnessCfg { max_floors: 4 };
    let flags: u128 = 0xe337be_00000000;
    let w = run_witness(&levels, 3, (2, 11), chain_player_f3(), flags, &si.shops, &cfg);
    let f3 = w.floors.iter().find(|f| f.floor == 3).expect("f3 已落账");
    assert_eq!(f3.keys_out.len(), 3, "keys_out=(y,b,r)");
    assert_eq!(f3.out_hp, 4027, "f3 出层血量是回归锚（gold 软层不许压血）");
    assert!(f3.ok);
}

/// f3 链路入场态（与 tests/exact_pickup.rs::f3_chain_player 同值：IN_HP_F3=3227）。
fn chain_player_f3() -> PlayerSpec {
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

// ---------------------------------------------------------------- 判据 8：f29 具名化

/// f29 的 `unreachable` 是「硬阻挡按永久」假设的最坏产物（取证
/// docs/findings/2026-11-20-f29-reachability.md）⇒ 具名 `blocked_by_assumption`。
#[test]
fn f29_status_is_blocked_by_assumption() {
    let lv = real_level_spec(29);
    let d = game_solver::keydemand::analyze_floor(29, &lv);
    assert_eq!(
        d.status.tag(),
        "blocked_by_assumption",
        "f29 必须报具名状态（旧值 unreachable 只作别名/文档更正）"
    );
    assert_eq!(d.status, game_solver::keydemand::FloorStatus::Unreachable);
    assert_ne!(d.status.tag(), "unreachable", "JSON/CLI 不得再输出旧值");
}

// ---------------------------------------------------------------- 夹具助手

use game_solver::solve_local::{ExitKind, ExitSpec};

/// 8×3 走廊（仅 y=1 可走）+ ty8 下行梯@(1,1)（keydemand 的入场格/witness 的落点梯）
/// + ty7 上行梯@(7,1) → `to`（带 ExitSpec 注册，witness 求解需要）。
/// 与 tests/witness.rs::corridor 同式（走廊几何是 witness 求解的既有夹具面）。
fn corridor_with_up(to: i32) -> LevelSpec {
    let (cols, rows) = (8, 3);
    let terrain: Vec<bool> = (0..cols * rows).map(|i| (i / cols) == 1).collect();
    let mut lv = level(cols, rows, terrain, Vec::new());
    lv.entities.push(ent(8, 1, 1, 1));
    lv.entities.push(ent(7, 7, 1, to));
    lv.exits.push(ExitSpec { x: 7, y: 1, kind: ExitKind::Stair, index: 1 });
    lv
}

/// f1 双上行梯：@(4,1)→2、@(6,1)→3（next_demand 只读层图实体，无需 ExitSpec）。
fn two_up_corridor() -> LevelSpec {
    let (cols, rows) = (8, 3);
    let terrain: Vec<bool> = (0..cols * rows).map(|i| (i / cols) == 1).collect();
    let mut lv = level(cols, rows, terrain, Vec::new());
    lv.entities.push(ent(8, 1, 1, 1));
    lv.entities.push(ent(7, 4, 1, 2));
    lv.entities.push(ent(7, 6, 1, 3));
    lv
}

/// 后继层：ty8 入场@(1,1) + 各色门（门格堵在单行走廊上）+ 对应钥匙 + ty7@(7,1)。
/// keydemand 口径 needed = 入场钥匙（门后钥匙不减需求）⇒ needed == need。
/// `need` 即 keydemand 向量 (y,b,r)（slot0=黄、slot1=蓝、slot2=红）。状态 Ok。
fn successor_floor(floor: i32, need: [i32; 3]) -> LevelSpec {
    let mut ents = vec![ent(8, 1, 1, 1)];
    let mut x = 2;
    for slot in 0..3 {
        for _ in 0..need[slot] {
            ents.push(ent(door_ty(slot), x, 1, 0));
            x += 1;
        }
    }
    for slot in 0..3 {
        for _ in 0..need[slot] {
            ents.push(ent(key_ty(slot), x, 1, 0));
            x += 1;
        }
    }
    ents.push(ent(7, 7, 1, floor + 1));
    let mut lv = successor_base();
    lv.entities = ents;
    lv.exits.push(ExitSpec { x: 7, y: 1, kind: ExitKind::Stair, index: 1 });
    lv
}

fn successor_base() -> LevelSpec {
    let (cols, rows) = (8, 3);
    let terrain: Vec<bool> = (0..cols * rows).map(|i| (i / cols) == 1).collect();
    level(cols, rows, terrain, Vec::new())
}

/// 真数据单层 LevelSpec（与 tests/keydemand.rs::real_level 同式）。
fn real_level_spec(f: i32) -> LevelSpec {
    let si = load_all_ingest();
    let (base, scaled) = enemy_tables();
    let tile_walk = nums(include_str!(
        "../../game-core/tests/fixtures/tile_walkability.txt"
    ));
    let inp = game_solver::solve_local::load_floor(
        &si,
        f as usize,
        (6, 11),
        PlayerSpec::from_combat(s0()),
        scaled,
        base,
        &tile_walk,
    )
    .expect("真数据层装载");
    inp.level.expect("真数据层必有 LevelSpec")
}

// ===========================================================================
// 【S6a】`key_floor` 比较器（方案 §2.3 / §5）：`None` 全等 `cmp_exit`；`Some` 只做
// 二值优先级，hp/gold 差异永远压过「多带钥匙」。
// ===========================================================================

/// 判据 `s6a_cmp_none_equals_cmp_exit`（方案 M5 的靶子）：
/// `cmp_exit_floor(.., None)` 必须在整张网格上与 `cmp_exit` **逐组全等**。
#[test]
fn s6a_cmp_floor_none_equals_cmp_exit() {
    let mut grid = Vec::new();
    for hp in [4000, 4027] {
        for gold in [0, 100, 500] {
            for steps in [80, 81, 88] {
                for k in 0..8u8 {
                    // 每色只取 0 / 2（方案 §5 的采样域）
                    let keys = [(k & 1) * 2, ((k >> 1) & 1) * 2, ((k >> 2) & 1) * 2];
                    grid.push(score(hp, gold, keys, steps));
                }
            }
        }
    }
    assert_eq!(grid.len(), 2 * 3 * 3 * 8, "网格点数");
    let mut checked = 0usize;
    for (i, a) in grid.iter().enumerate() {
        for b in grid.iter().skip(i + 1) {
            assert_eq!(
                cmp_exit_floor(a, b, None),
                cmp_exit(a, b),
                "None 分支必须与 cmp_exit 全等：a={a:?} b={b:?}"
            );
            assert_eq!(
                cmp_exit_floor(b, a, None),
                cmp_exit(b, a),
                "None 分支必须与 cmp_exit 全等（反向）：a={a:?} b={b:?}"
            );
            checked += 1;
        }
    }
    assert!(checked >= 200, "至少 200 组采样，实测 {checked}");
}

/// floor 的**唯一位次**：hp → gold → 满足 floor → steps。
/// 变异（把 floor 提到 hp/gold 之前）⇒ 前两条断言红（方案 M2 / R4）。
#[test]
fn s6a_keys_never_outrank_hp_or_gold_with_floor() {
    let floor = Some([2, 0, 0]);
    // 1) 血多钥少 vs 血少钥多 ⇒ 血多者胜（floor 不许换血）
    let a = score(4100, 0, [0, 0, 0], 90);
    let b = score(4000, 0, [2, 0, 0], 88);
    assert_eq!(cmp_exit_floor(&a, &b, floor), Ordering::Less, "hp 永远优先于 key_floor");
    assert_eq!(cmp_exit_floor(&b, &a, floor), Ordering::Greater);
    // 2) 同血金多钥少 vs 金少钥多 ⇒ 金多者胜（floor 不许换金）
    let e = score(4027, 500, [0, 0, 0], 88);
    let f = score(4027, 100, [2, 0, 0], 88);
    assert_eq!(cmp_exit_floor(&e, &f, floor), Ordering::Less, "gold 优先于 key_floor");
    assert_eq!(cmp_exit_floor(&f, &e, floor), Ordering::Greater);
    // 3) 同 hp 同 gold ⇒ 满足 floor 者胜（floor 唯一生效的位次）
    let g = score(4027, 500, [0, 0, 0], 80);
    let h = score(4027, 500, [2, 0, 0], 90);
    assert_eq!(cmp_exit_floor(&h, &g, floor), Ordering::Less, "同 hp 同 gold ⇒ 带够钥匙者胜");
    assert_eq!(cmp_exit_floor(&g, &h, floor), Ordering::Greater);
    // 4) 同 hp 同 gold 都不满足 floor ⇒ 退化到 steps（floor 判断相同不颠倒序）
    let j = score(4027, 500, [1, 0, 0], 81);
    let k = score(4027, 500, [0, 0, 0], 88);
    assert_eq!(cmp_exit_floor(&j, &k, floor), Ordering::Less, "都未达 floor ⇒ 仍按 steps");
    // 5) 全部同 ⇒ steps 少者胜
    let i = score(4027, 500, [2, 0, 0], 81);
    assert_eq!(cmp_exit_floor(&i, &h, floor), Ordering::Less, "同 floor 结论 ⇒ steps 少者胜");
}
