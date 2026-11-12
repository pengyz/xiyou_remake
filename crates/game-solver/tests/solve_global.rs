//! 【求解器批3 C2】两层玩具图（钥匙门跨层）——计划 §4 批3 TDD 起手式。
//!
//! 断言口径（计划 :339-341）：
//! ① **跨层钥匙传播**：floor51 拾黄钥（type26）→ 下行梯（type8 param=50）→ floor50
//!   黄门（type1）开锁 → 终局——若全局态不把 post.player 传入下层，门打不开、无目标；
//! ② **型72 终局边**：floor50 击败 type72（floor50 cat8 覆写可 bump，sprite_defs
//!   floor50→cat8；钩子 scene.rs:3891-3894 → load_level_script(65)=TAK_127_128
//!   END_0 → GameMode::Ending = 通关）；
//! ③ **分层一致性**（判据 `cargo test -p game-solver solve_global`）：逐层
//!   `solve_layer` 结果拼接 == 全局解 hp/kills/steps/route/floors。
//!
//! 夹具全手工（13×13 真图归 C5 CLI 端到端）；敌表 dense 41..=72——
//! `resolve_bump`/`predict_engagement` 按 `idx=(t-41)` 稠密下标（battle.rs:105/130）。
use game_data::defs::EnemyDef;
use game_solver::solve_global::{solve_global, GlobalInput};
use game_solver::solve_local::{
    solve_layer, EntSpec, ExitKind, ExitSpec, LevelSpec, PlayerSpec, SolveInput,
};
use std::collections::BTreeMap;

/// 敌表：32 行 id41..=72，默认 hp10/atk1/def0/gold0/trait_flags0；
/// `v[31]`=type72 {hp35,atk18,def1}（地板伤害 24=300-276 的来源）。
fn enemies() -> Vec<EnemyDef> {
    let mut v: Vec<EnemyDef> = (0..32)
        .map(|i| EnemyDef {
            id: 41 + i,
            hp: 10,
            atk: 1,
            def: 0,
            gold: 0,
            trait_flags: 0,
        })
        .collect();
    v[31] = EnemyDef {
        id: 72,
        hp: 35,
        atk: 18,
        def: 1,
        gold: 0,
        trait_flags: 0,
    };
    v
}

/// S₀ 玩家（计划 §2.5：hp300/atk10/def10、三色钥 0、金币 0、∅背包）。
fn player() -> PlayerSpec {
    PlayerSpec {
        hp: 300,
        atk: 10,
        def: 10,
        yellow: 0,
        blue: 0,
        red: 0,
        gold: 0,
        item_types: Vec::new(),
        item_uses: Vec::new(),
        item_stack_size: 0,
    }
}

/// floor51 玩具：4×3，仅中排 y=1 可走；(0,1) 入场 → (1,1) 黄钥 type26 →
/// (2,1) 下行梯 type8 param=50（→ target=50，went_down=true）。
fn floor51() -> LevelSpec {
    LevelSpec {
        cols: 4,
        rows: 3,
        terrain: vec![
            false, false, false, false, //
            true, true, true, true, //
            false, false, false, false,
        ],
        start: (0, 1),
        entities: vec![
            EntSpec {
                ty: 26,
                x: 1,
                y: 1,
                param: 0,
            },
            EntSpec {
                ty: 8,
                x: 2,
                y: 1,
                param: 50,
            },
        ],
        exits: vec![ExitSpec {
            x: 2,
            y: 1,
            kind: ExitKind::Stair,
            index: 1,
        }],
        player: player(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    }
}

/// floor50 玩具：9×9，仅底排 y=7 x=5..8 可走；
/// (6,7)=swap_floor floor50 固定落点（scene_persist.rs:421）→
/// (7,7) 黄门 type1 → (8,7) type72（floor50 cat8 可 bump）；无出口。
fn floor50() -> LevelSpec {
    let mut terrain = vec![false; 9 * 9];
    for x in 5..=8 {
        terrain[7 * 9 + x] = true;
    }
    LevelSpec {
        cols: 9,
        rows: 9,
        terrain,
        start: (6, 7),
        entities: vec![
            EntSpec {
                ty: 1,
                x: 7,
                y: 7,
                param: 0,
            },
            EntSpec {
                ty: 72,
                x: 8,
                y: 7,
                param: 0,
            },
        ],
        exits: Vec::new(),
        player: player(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    }
}

/// C2 主测：钥匙门跨层 + 型72 Ending + 分层一致性（计划 :339-341 TDD 起手式）。
#[test]
fn solve_global_toy两层钥匙门跨层_分层一致性与通关终局() {
    let mut floors: BTreeMap<i32, LevelSpec> = BTreeMap::new();
    floors.insert(51, floor51());
    floors.insert(50, floor50());
    let input = GlobalInput {
        floors,
        start_floor: 51,
        start_pos: (0, 1),
        start_player: player(),
        start_flags: 0,
        bound: 10_000,
    };
    let r = solve_global(&input);

    // 全队列耗尽（玩具全边已建）⇒ optimal（裁定⑤）
    assert_eq!(r.confidence, "optimal", "置信：{r:?}");
    let g = r
        .goals
        .first()
        .unwrap_or_else(|| panic!("应有且仅有通关目标：{r:?}"));

    // ② 终局：型72 击败 → Ending；跨层序列 51→50
    assert_eq!(g.floors, vec![51, 50], "层序列：{g:?}");
    assert_eq!(g.kills, 1, "只杀 type72（黄钥是拾取非击杀）：{g:?}");
    assert_eq!(g.hp, 276, "300-24（type72 atk18-def10=8×3 击杀轮）：{g:?}");
    assert_eq!(g.steps, 4, "floor51 走 2 + floor50 走 2：{g:?}");

    // route=逐层 route_of 拼接：Start|Move→x,y；Door→door x,y；Bump→bump ty@x,y
    assert_eq!(
        g.route,
        vec![
            "0,1", "1,1", "2,1", // floor51：入场→拾钥→梯
            "6,7", "door 7,7", "bump 72@8,7", // floor50：落点→开门→终局
        ],
        "跨层 route：{g:?}"
    );
    // 跨层传播证明：floor51 post.yellow=1（拾钥），开门耗钥后终态=0
    assert_eq!(g.post.yellow, 0, "黄钥被 floor50 门消耗：{g:?}");

    // ③ 分层一致性：逐层 solve_layer 拼接 == 全局解
    let s51 = SolveInput {
        floor: 51,
        difficulty: 0,
        level: Some(floor51()),
    };
    let (reach1, goals1) = solve_layer(&s51, (0, 1), None, player(), 0);
    assert!(reach1, "floor51 层可达");
    let leg1 = goals1
        .iter()
        .find(|q| q.kind == ExitKind::Stair)
        .unwrap_or_else(|| panic!("floor51 应有楼梯目标：{goals1:?}"));
    assert_eq!(leg1.post.yellow, 1, "拾钥进 post.player：{leg1:?}");
    assert_eq!(leg1.hp, 300, "floor51 无战斗：{leg1:?}");
    assert_eq!(leg1.kills, 0, "黄钥非击杀：{leg1:?}");
    assert_eq!(leg1.steps, 2, "入场→拾钥→梯：{leg1:?}");
    assert_eq!(leg1.route, vec!["0,1", "1,1", "2,1"], "层内 route：{leg1:?}");

    let s50 = SolveInput {
        floor: 50,
        difficulty: 0,
        level: Some(floor50()),
    };
    // 入场格=swap_floor floor50 固定落点 (6,7)；携带 floor51 的 post.player
    let (reach2, goals2) = solve_layer(&s50, (6, 7), None, leg1.post.clone(), 0);
    assert!(reach2, "floor50 层可达（携钥开门）");
    let leg2 = goals2
        .iter()
        .find(|q| q.kind == ExitKind::Ending)
        .unwrap_or_else(|| panic!("floor50 应有 Ending 目标：{goals2:?}"));
    assert_eq!(leg2.hp, 276, "开怪扣血：{leg2:?}");
    assert_eq!(leg2.kills, 1, "杀 type72：{leg2:?}");
    assert_eq!(leg2.steps, 2, "落点→开门→bump：{leg2:?}");

    // 拼接 == 全局（判据原文：逐层结果拼接==全局解 hp/kills）
    assert_eq!(g.kills, leg1.kills + leg2.kills, "kills 拼接：{g:?}");
    assert_eq!(g.hp, leg2.hp, "hp 拼接（终层 post）：{g:?}");
    assert_eq!(g.steps, leg1.steps + leg2.steps, "steps 拼接：{g:?}");
    let mut joined = leg1.route.clone();
    joined.extend(leg2.route.iter().cloned());
    assert_eq!(g.route, joined, "route 拼接：{g:?}");
}

/// 反例：无钥直入 floor50 ⇒ 黄门挡死、无终局目标（跨层携钥是通关必要条件）。
#[test]
fn solve_global_toy反例_无钥直入floor50无解() {
    let mut floors: BTreeMap<i32, LevelSpec> = BTreeMap::new();
    floors.insert(50, floor50());
    let r = solve_global(&GlobalInput {
        floors,
        start_floor: 50,
        start_pos: (6, 7),
        start_player: player(),
        start_flags: 0,
        bound: 10_000,
    });
    assert!(r.goals.is_empty(), "无钥不应有通关目标：{r:?}");
    assert_eq!(r.confidence, "feasible", "无解=feasible（裁定⑤）：{r:?}");
}
