//! 【P7】撞门臂（`interact_with_cell` 的 bump 面，`from_walk=false`）：判据 1–4。
//!
//! | # | 测试 | 守的是什么 |
//! |---|---|---|
//! | 1 | `gate_bump_runs_bump_only_gate_without_moving` | 撞-only 门（`!from_walk` 臂）能跑脚本、玩家**不移动**、脚本效果真跑进实体表 |
//! | 2 | `gate_bump_not_generated_for_walk_triggerable_gate` | 走能触发的门（`from_walk` 臂）**不建**撞门臂（防状态爆炸；f51/f54 不变量之源） |
//! | 3 | `no_bump_without_gate` | 邻格无 `ty83` ⇒ 零撞门后继、零计数（无门层输出不变） |
//! | 4 | `f3_gate27_is_walk_only_no_bump_arm` | 真层 f3：`(4,11)→(5,11)` 撞不出后继（门 27 只认 `from_walk`） |
//!
//! 为什么撞门只对「撞才触发」的门建臂：走能触发的门走上目标格本身就跑同一个脚本并
//! 停在原地（`interact_with_cell` 命中即 `allowed=false`，scene.rs:3379-3393）
//! ⇒ 撞门后继被走步臂支配。反向隐患正是本片硬约束「f51..f54 逐字段不变」。
use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_solver::ingest::{ingest, SolverInput};
use game_solver::solve_local::{
    gate_bump_stats, load_floor, probe_gate, probe_gate_bump, solve_local, EntSpec, ExitKind,
    ExitSpec, LevelSpec, PlayerSpec, SolveInput,
};
use std::path::PathBuf;

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

/// 与 `tests/pure_pickup.rs::load_si` 同构的装载路径（同一份夹具/表）。
fn load_si() -> SolverInput {
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

fn tables() -> (Vec<game_data::defs::EnemyDef>, Vec<game_data::defs::EnemyDef>, Vec<i32>) {
    let t = CombatTables {
        enemy_base_hp: nums(include_str!("../../game-core/tests/fixtures/enemybasehp.txt")),
        enemy_base_atk: nums(include_str!("../../game-core/tests/fixtures/enemybaseatk.txt")),
        enemy_base_def: nums(include_str!("../../game-core/tests/fixtures/enemybasedef.txt")),
        difficulty_multipliers: nums(include_str!(
            "../../game-core/tests/fixtures/difficulty_multipliers.txt"
        )),
    };
    let gold = nums(include_str!("../../game-core/tests/fixtures/enemybasegold.txt"));
    let base = t.to_records(&gold);
    let scaled = scale_enemy_stats(&base, 1);
    let tile_walk = nums(include_str!("../../game-core/tests/fixtures/tile_walkability.txt"));
    (scaled, base, tile_walk)
}

fn mk_player() -> PlayerCombat {
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

/// 合成层：13×13 全可走；`(0,1)` 入场、`(2,1)` ty7 上行梯（出口）。
/// `gate` = 放在 `(1,1)` 的 ty83 槽（`None` ⇒ 无门）；`ent72_at` = 给脚本当靶子的 ty72。
///
/// 脚本槽内容（`game-core/tests/fixtures/level_scripts.txt`，行号 = 槽号+1）：
/// - 槽 1 = `TAK_22_22 ROS_4_3 TAK_23_32 MOV_72_3_7_1_8`（`flag(0) && !from_walk`）
/// - 槽 2 = `TAK_33_36 MOV_72_1_8_1_10 DES_72_1_10`（`flag(1) && !from_walk`）
/// - 槽 24 = `TAK_40_40`（`!from_walk && hp>0`）
/// - 槽 27 = `TAK_113_117 TAK_133_135 DES_84_6_11 GIN_0_13`（`from_walk && flag(0)`，走-only）
fn synth(gate: Option<i32>, ent72_at: Option<(i32, i32)>, flags: u128) -> SolveInput {
    let (scaled, base, _tile_walk) = tables();
    let n = 13i32;
    let mut entities = vec![
        EntSpec { ty: 7, x: 2, y: 1, param: 4 },
    ];
    if let Some(p) = gate {
        entities.push(EntSpec { ty: 83, x: 1, y: 1, param: p });
    }
    if let Some((x, y)) = ent72_at {
        entities.push(EntSpec { ty: 72, x, y, param: 0 });
    }
    let level = LevelSpec {
        cols: n,
        rows: n,
        terrain: vec![true; (n * n) as usize],
        start: (0, 1),
        entities,
        exits: vec![ExitSpec { x: 2, y: 1, kind: ExitKind::Stair, index: 1 }],
        player: PlayerSpec::from_combat(mk_player()),
        enemies: scaled,
        enemy_base: base,
        flags,
    };
    SolveInput { floor: 1, difficulty: 0, level: Some(level) }
}

fn floor_input(si: &SolverInput, f: i32) -> SolveInput {
    let (scaled, base, tile_walk) = tables();
    load_floor(
        si,
        f as usize,
        (1, 11),
        PlayerSpec::from_combat(mk_player()),
        scaled,
        base,
        &tile_walk,
    )
    .unwrap_or_else(|e| panic!("load_floor({f}) 失败: {e}"))
}

// ---------------------------------------------------------------- 判据

/// 判据 1：撞-only 门（槽 2 = `flag(1) && !from_walk`）能触发 ⇒ 脚本真跑
/// （`MOV_72_1_8_1_10` + `DES_72_1_10` 把 (1,8) 的实体 72 送走并移除），
/// **玩家留在入场格**、不计步。
#[test]
fn gate_bump_runs_bump_only_gate_without_moving() {
    // flags bit1 ⇒ 槽 2 的 `v.flag(1)` 满足；bump 面 `!from_walk` 满足 ⇒ 可触发
    let input = synth(Some(2), Some((1, 8)), 0b10);
    let ps = input.level.as_ref().unwrap().player.clone();

    // 反证腿：同一门**走上去不触发**（`from_walk` 不满足）——这正是「撞才有意义」
    let before_walk = probe_gate_bump(&input, (0, 1), &ps, 0b10, None, (1, 1));
    assert!(before_walk.is_some(), "bump 面必须能触发槽 2");

    let obs = before_walk.unwrap();
    assert_eq!(obs.pos, (0, 1), "撞门不移动玩家（bump 面 allowed=false）");
    assert!(
        !obs.ents.iter().any(|e| e.ty == 72),
        "脚本必须真跑：MOV_72_1_8_1_10 + DES_72_1_10 ⇒ 实体 72 应被送走并移除，实得 {:?}",
        obs.ents
    );

    // 臂确实进了**搜索**（不是只有探针能调）：同层跑一遍，计数必须 ≥1。
    let r = solve_local(&input);
    assert!(r.reachable, "同层照常可解：{r:?}");
    let (gen, hit) = gate_bump_stats();
    assert!(
        hit >= 1,
        "撞门臂必须真进搜索（生成 {gen} / 命中 {hit}）——否则 expand 没接上"
    );
}

/// 判据 2：走能触发的门（槽 27 = `from_walk && flag(0)`）**不建**撞门臂。
/// 这是 f51/f54「逐字段不变」的机制来源（那两层的 ty83 全是 walk 可触发）。
#[test]
fn gate_bump_not_generated_for_walk_triggerable_gate() {
    let input = synth(Some(27), None, 0b1);
    let ps = input.level.as_ref().unwrap().player.clone();

    // 走面确实能开（否则本判据退化成「门坏了」）
    assert!(probe_gate(&input, 27), "槽 27 在 flag(0) 置位时走上格必须触发");

    assert!(
        probe_gate_bump(&input, (0, 1), &ps, 0b1, None, (1, 1)).is_none(),
        "走能触发的门不得生成撞门后继（撞门被走步臂支配，只会状态爆炸）"
    );

    // 更严的一刀：**连尝试都不做**。`bump_only_gate_params` 预筛在先，否则每个
    // 「邻格有走-only 门」的节点都要白跑一次 `build_scene`（C5b 实测的主要开销）。
    let r = solve_local(&input);
    assert!(r.reachable, "同层照常可解：{r:?}");
    assert_eq!(
        gate_bump_stats(),
        (0, 0),
        "走-only 门连撞门候选都不许进（预筛失效 ⇒ 逐节点白建场景）"
    );
}

/// 判据 3：邻格无 `ty83` ⇒ 零撞门后继、零计数。用真实搜索跑一遍整层取证
/// （不是只看探针）：无门层的输出必须与撞门臂无关。
#[test]
fn no_bump_without_gate() {
    let input = synth(None, Some((1, 8)), 0b10);
    let ps = input.level.as_ref().unwrap().player.clone();
    assert!(
        probe_gate_bump(&input, (0, 1), &ps, 0b10, None, (1, 1)).is_none(),
        "无门格不得产生撞门后继"
    );

    let r = solve_local(&input);
    assert!(r.reachable, "无门层照常可解：{r:?}");
    assert_eq!(gate_bump_stats(), (0, 0), "无门层撞门臂计数必须为 0");
    assert!(
        !r.route.iter().any(|s| s.starts_with("gatebump")),
        "无门层路线里不得出现撞门航点：{:?}",
        r.route
    );
}

/// 判据 4：真层 f3 的 `(5,11)` 门 27 只认 `from_walk`（gate.rs:147）⇒ 从 `(4,11)`
/// 撞它**不产生**后继。这同时是「口袋没被封」的机制前提：走上去（`from_walk=true`）
/// 才是它的唯一合法触发方式，而 flags[0]=0 时它连走都不开 ⇒ 格可穿。
#[test]
fn f3_gate27_is_walk_only_no_bump_arm() {
    let si = load_si();
    let f3 = floor_input(&si, 3);
    let lv = f3.level.as_ref().expect("f3 必须有层");
    let ps = PlayerSpec::from_combat(mk_player());

    assert!(!probe_gate(&f3, 27), "flags[0]=0 ⇒ 门 27 走也不开（gate.rs:147）");
    assert!(
        probe_gate_bump(&f3, (4, 11), &ps, 0, None, (5, 11)).is_none(),
        "门 27 是走-only 门，撞门面恒 false ⇒ 不得生成撞门后继"
    );
    // 入口门 param=0：恒过（gate.rs:76），走面/撞面都可，但它在入场格、不属于撞门目标
    assert!(probe_gate(&f3, 0), "param=0 恒过（G2a 通过的依据）");
    assert!(lv.entities.iter().any(|e| e.ty == 83 && e.x == 5 && e.y == 11 && e.param == 27));
}

/// 判据 5（真层正例）：f35 有 `ty83 param=24` 的门（`!from_walk && hp>0`——
/// 撞-only 臂）。撞门臂在**真实层数据**上必须真触发并读回（非纯合成夹具）。
#[test]
fn gate_bump_fires_on_real_floor35() {
    let si = load_si();
    let f35 = floor_input(&si, 35);
    let lv = f35.level.as_ref().expect("f35 必须有层");
    let g = lv
        .entities
        .iter()
        .find(|e| e.ty == 83 && e.param == 24)
        .expect("f35 应有 param=24 的 ty83 门（P6 扫描表）");
    // 门格 (g.x,g.y)；从相邻可走格撞它
    let ps = PlayerSpec::from_combat(mk_player());
    let adj = [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .map(|(dx, dy)| (g.x + dx, g.y + dy))
        .find(|&(x, y)| {
            probe_gate_bump(&f35, (x, y), &ps, 0, None, (g.x, g.y)).is_some()
        });
    let entry = adj.unwrap_or_else(|| {
        panic!("f35 的 param=24 门 @({},{}) 相邻格撞不动（bump 臂在真层失效）", g.x, g.y)
    });
    let obs = probe_gate_bump(&f35, entry, &ps, 0, None, (g.x, g.y)).unwrap();
    assert_eq!(obs.pos, entry, "撞门不移动玩家");
    assert_eq!(obs.hp, 300, "槽 24 = TAK_40_40 纯演出，不掉血（实得 {}）", obs.hp);
}
