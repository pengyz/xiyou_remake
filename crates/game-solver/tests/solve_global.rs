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
use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_solver::solve_global::{solve_global, to_solver_state, GlobalInput};
use game_solver::state::{CanonicalBag, DiscreteGold, DiscreteHp, ScriptFlags, SolverState};
use game_solver::solve_local::{
    load_floor, solve_layer, EntRec, EntSpec, ExitKind, ExitSpec, LevelSpec, PlayerSpec,
    SolveInput,
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

/// gate 夹具（批2 T9 同构，step_off 走位锚梯改 param=50→floor50）：
/// 13×13 全可走，(6,10) 入场 → (6,9) 场景门 type83 param=19；
/// 脚本19（fixture 第20行）尾部 `MOV_0_11_1 MOV_0_1_11` 把玩家走位上梯
/// (11,1) type7 → changeFloor(50, down=false, strict=false) → pending_floor=50。
/// **(11,1) 不入 exits**：只作脚本走位锚（避免与 Script 边双路径）；真实数据
/// load_floor 全量入 exits=C5 端到端再对齐。
fn gate_floor() -> LevelSpec {
    LevelSpec {
        cols: 13,
        rows: 13,
        terrain: vec![true; 13 * 13],
        start: (6, 10),
        entities: vec![
            EntSpec { ty: 83, x: 6, y: 9, param: 19 },
            EntSpec { ty: 7, x: 11, y: 1, param: 50 },
        ],
        exits: Vec::new(),
        player: player(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    }
}

/// floor50 无门玩具：(6,7)=swap_floor floor50 固定落点 → (7,7) type72；无出口。
fn floor50_nodoor() -> LevelSpec {
    let mut terrain = vec![false; 9 * 9];
    for x in 5..=8 {
        terrain[7 * 9 + x] = true;
    }
    LevelSpec {
        cols: 9,
        rows: 9,
        terrain,
        start: (6, 7),
        entities: vec![EntSpec { ty: 72, x: 7, y: 7, param: 0 }],
        exits: Vec::new(),
        player: player(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    }
}

/// C3a 主测：StepGate script 换层边（BR-1 回收 pending_floor/went_down 线程化）
/// + 跨层携态 + 分层一致性（判据 `cargo test -p game-solver solve_global`）。
#[test]
fn solve_global_toy_stepgate脚本换层边_跨层携态与通关终局() {
    let mut floors: BTreeMap<i32, LevelSpec> = BTreeMap::new();
    floors.insert(1, gate_floor());
    floors.insert(50, floor50_nodoor());
    let r = solve_global(&GlobalInput {
        floors,
        start_floor: 1,
        start_pos: (6, 10),
        start_player: player(),
        start_flags: 0,
        bound: 10_000,
    });

    assert_eq!(r.confidence, "optimal", "置信：{r:?}");
    let g = r
        .goals
        .first()
        .unwrap_or_else(|| panic!("脚本边应跨层到 floor50 通关：{r:?}"));
    assert_eq!(g.floors, vec![1, 50], "层序列：{g:?}");
    assert_eq!(g.kills, 1, "只杀 type72：{g:?}");
    assert_eq!(g.hp, 276, "300-24：{g:?}");
    assert_eq!(g.steps, 2, "floor1 走 1（入场→门格）+ floor50 走 1（落点直接 bump 邻格）：{g:?}");
    assert_eq!(
        g.route,
        vec![
            "6,10", "6,9", "gate 6,9", // floor1：入场→门格→gate 航点
            "6,7", "bump 72@7,7", // floor50：固定落点→终局
        ],
        "跨层 route：{g:?}"
    );

    // 分层一致性：逐层 solve_layer 拼接 == 全局解
    let s1 = SolveInput { floor: 1, difficulty: 0, level: Some(gate_floor()) };
    let (reach1, goals1) = solve_layer(&s1, (6, 10), None, player(), 0);
    assert!(reach1, "floor1 层可达");
    let leg1 = goals1
        .iter()
        .find(|q| q.kind == ExitKind::Script)
        .unwrap_or_else(|| panic!("floor1 应有 Script 目标：{goals1:?}"));
    // 脚本换层边的两元组：target=param&0xff=50、type7 → went_down=false
    assert_eq!(leg1.cross_edge, Some((50, false)), "script 边：{leg1:?}");
    assert_eq!(leg1.hp, 300, "floor1 无战斗：{leg1:?}");
    assert_eq!(leg1.kills, 0, "无怪：{leg1:?}");
    assert_eq!(leg1.steps, 1, "入场→门格：{leg1:?}");
    assert_eq!(leg1.route, vec!["6,10", "6,9", "gate 6,9"], "层内 route：{leg1:?}");

    let s50 = SolveInput { floor: 50, difficulty: 0, level: Some(floor50_nodoor()) };
    let (reach2, goals2) = solve_layer(&s50, (6, 7), None, leg1.post.clone(), leg1.post_flags);
    assert!(reach2, "floor50 层可达");
    let leg2 = goals2
        .iter()
        .find(|q| q.kind == ExitKind::Ending)
        .unwrap_or_else(|| panic!("floor50 应有 Ending 目标：{goals2:?}"));
    assert_eq!(leg2.hp, 276, "开怪扣血：{leg2:?}");
    assert_eq!(leg2.kills, 1, "杀 type72：{leg2:?}");
    assert_eq!(leg2.steps, 1, "落点 (6,7) 直接 bump 邻格 (7,7)：{leg2:?}");

    // 拼接 == 全局（判据原文：逐层结果拼接==全局解 hp/kills）
    assert_eq!(g.kills, leg1.kills + leg2.kills, "kills 拼接：{g:?}");
    assert_eq!(g.hp, leg2.hp, "hp 拼接（终层 post）：{g:?}");
    assert_eq!(g.steps, leg1.steps + leg2.steps, "steps 拼接：{g:?}");
    let mut joined = leg1.route.clone();
    joined.extend(leg2.route.iter().cloned());
    assert_eq!(g.route, joined, "route 拼接：{g:?}");
}

/// C3b：floor51 往返玩具——5×2；row1=(0,1) 入场、(1,1) 筋斗云 type15、
/// (2,1) 下行梯 type8 param=50（入 exits）；row0=(1,0)(2,0)(3,0) 黄门 type1、
/// (4,0) QoL type20。锚 down=(2,1)（层内唯一 type8）、无 type7 ⇒ 只有键55 云边。
fn floor51_roundtrip() -> LevelSpec {
    let mut terrain = vec![false; 5 * 2];
    for x in 1..=4 {
        terrain[x] = true; // row0 x1..4
    }
    for x in 0..=2 {
        terrain[5 + x] = true; // row1 x0..2
    }
    LevelSpec {
        cols: 5,
        rows: 2,
        terrain,
        start: (0, 1),
        entities: vec![
            EntSpec { ty: 15, x: 1, y: 1, param: 0 },
            EntSpec { ty: 8, x: 2, y: 1, param: 50 },
            EntSpec { ty: 1, x: 3, y: 0, param: 0 },
            EntSpec { ty: 20, x: 4, y: 0, param: 0 },
        ],
        exits: vec![ExitSpec { x: 2, y: 1, kind: ExitKind::Stair, index: 1 }],
        player: player(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    }
}

/// C3b：floor50 往返玩具——11×9 仅 row7 x5..10 可走；(6,7) swap_floor 固定落点、
/// (7,7) 上行梯 type7 param=51（锚点，**不入 exits**=gate_floor 同款夹具quirk，
/// 避免与普通踩梯双路径；真实数据 C5 对齐）、(8,7) 黄钥 type26、(10,7) 型72。
/// 无 type8 ⇒ floor50 上只有键49（type7）云边。
fn floor50_roundtrip() -> LevelSpec {
    let mut terrain = vec![false; 11 * 9];
    for x in 5..=10 {
        terrain[7 * 11 + x] = true;
    }
    LevelSpec {
        cols: 11,
        rows: 9,
        terrain,
        start: (6, 7),
        entities: vec![
            EntSpec { ty: 7, x: 7, y: 7, param: 51 },
            EntSpec { ty: 26, x: 8, y: 7, param: 0 },
            EntSpec { ty: 72, x: 10, y: 7, param: 0 },
        ],
        exits: Vec::new(),
        player: player(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    }
}

/// C3b：floor51 仅云通路玩具——同 roundtrip 但 **exits=[]**（唯一换层通道=筋斗云
/// 键55）、无门无 QoL20：普通踩梯路径不存在 ⇒ 全局若拒绝云边则必无解。
fn floor51_cloudonly() -> LevelSpec {
    let mut terrain = vec![false; 5 * 2];
    for x in 0..=2 {
        terrain[5 + x] = true; // row1 x0..2
    }
    LevelSpec {
        cols: 5,
        rows: 2,
        terrain,
        start: (0, 1),
        entities: vec![
            EntSpec { ty: 15, x: 1, y: 1, param: 0 },
            EntSpec { ty: 8, x: 2, y: 1, param: 50 },
        ],
        exits: Vec::new(),
        player: player(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    }
}

/// C3b 主测①：筋斗云边区间内回程——A=首访直杀通关 [51,50]；
/// B=拾钥回 floor51 开门（**黄门只在返回后可开 ⇒ route 含 `door 3,0` ⇒
/// 云回程必然发生**=最稳判据）再入 floor50 终局 [51,50,51,50]。
/// 分层一致性：solve_layer 两腿拼接 == 全局解（判据原文）。
#[test]
fn solve_global_toy筋斗云边_区间内回程开门与通关() {
    let mut floors: BTreeMap<i32, LevelSpec> = BTreeMap::new();
    floors.insert(51, floor51_roundtrip());
    floors.insert(50, floor50_roundtrip());
    let r = solve_global(&GlobalInput {
        floors,
        start_floor: 51,
        start_pos: (0, 1),
        start_player: player(),
        start_flags: 0,
        bound: 10_000,
    });

    assert_eq!(r.confidence, "optimal", "置信：{r:?}");
    let a = r
        .goals
        .iter()
        .find(|g| g.floors == vec![51, 50])
        .unwrap_or_else(|| panic!("A 首访直杀目标应存在：{r:?}"));
    assert_eq!(a.kills, 1, "A 只杀 type72：{a:?}");
    assert_eq!(a.hp, 276, "A 300-24：{a:?}");
    let b = r
        .goals
        .iter()
        .find(|g| {
            g.floors == vec![51, 50, 51, 50]
                && g.route.iter().any(|s| s == "door 3,0")
        })
        .unwrap_or_else(|| panic!("B 回程开门目标应存在：{r:?}"));
    assert_eq!(b.kills, 1, "B 只杀 type72：{b:?}");
    assert_eq!(b.hp, 276, "B 300-24：{b:?}");
    assert!(
        b.route.iter().any(|s| s == "bump 72@10,7"),
        "B 终局航点：{b:?}"
    );

    // 分层一致性腿1：floor51 云边（键55：type8 → went_down=true）
    let s1 = SolveInput { floor: 51, difficulty: 0, level: Some(floor51_roundtrip()) };
    let (reach1, goals1) = solve_layer(&s1, (0, 1), None, player(), 0);
    assert!(reach1, "floor51 层可达");
    let c1 = goals1
        .iter()
        .find(|q| q.kind == ExitKind::Cloud)
        .unwrap_or_else(|| panic!("floor1 应有云边候选：{goals1:?}"));
    assert_eq!(c1.cross_edge, Some((50, true)), "键55 边：{c1:?}");
    assert_eq!(c1.hp, 300, "无战斗：{c1:?}");
    assert_eq!(c1.kills, 0, "无怪：{c1:?}");
    assert_eq!(c1.steps, 1, "入场→拾云格：{c1:?}");
    assert_eq!(c1.route, vec!["0,1", "1,1"], "云边 route：{c1:?}");

    // 分层一致性腿2：floor50 携 15 起跳（键49：type7 → went_down=false）
    let mut stack = vec![0u8; 32];
    stack[0] = 15;
    let mut p15 = player();
    p15.item_types = stack;
    let s50 = SolveInput { floor: 50, difficulty: 0, level: Some(floor50_roundtrip()) };
    let (reach2, goals2) = solve_layer(&s50, (6, 7), None, p15, 0);
    assert!(reach2, "floor50 层可达");
    let c2 = goals2
        .iter()
        .find(|q| q.kind == ExitKind::Cloud)
        .unwrap_or_else(|| panic!("floor50 应有云边候选：{goals2:?}"));
    assert_eq!(c2.cross_edge, Some((51, false)), "键49 边：{c2:?}");
    assert_eq!(c2.hp, 300, "起跳前无战斗：{c2:?}");
    assert_eq!(c2.steps, 0, "根态起跳不耗步：{c2:?}");
    assert_eq!(c2.route, vec!["6,7"], "根态云边 route：{c2:?}");
}

/// C3b 主测②：区间门外拒绝——首访区间 [51,51] 拒 target50 ⇒ 全局无解
/// （唯一通路=云）；层内照发云边（区间门是全局史，层内不知）。
#[test]
fn solve_global_toy筋斗云边_区间门外拒绝() {
    let mut floors: BTreeMap<i32, LevelSpec> = BTreeMap::new();
    floors.insert(51, floor51_cloudonly());
    floors.insert(50, floor50_nodoor());
    let r = solve_global(&GlobalInput {
        floors,
        start_floor: 51,
        start_pos: (0, 1),
        start_player: player(),
        start_flags: 0,
        bound: 10_000,
    });
    assert!(
        r.goals.is_empty(),
        "首访 [51,51] 拒 target50 ⇒ 云边全局落闸：{r:?}"
    );
    assert_eq!(r.confidence, "feasible", "无目标置信：{r:?}");

    // 层内腿照发（证明拒绝发生在全局区间门，不是层内没生成边）
    let s1 = SolveInput { floor: 51, difficulty: 0, level: Some(floor51_cloudonly()) };
    let (_reach, goals1) = solve_layer(&s1, (0, 1), None, player(), 0);
    let c = goals1
        .iter()
        .find(|q| q.kind == ExitKind::Cloud)
        .unwrap_or_else(|| panic!("层内云边应照发：{goals1:?}"));
    assert_eq!(c.cross_edge, Some((50, true)), "层内边：{c:?}");
}


/// C4 状态抽象（计划 §3.3 字段名不许漂 + 批3 裁定①②落地 + monsters_alive 全局口径）。
#[test]
fn solve_global_t状态抽象SolverState字段与怪物存亡口径() {
    // ① 字段名锁定：字面构造 SolverState（floor/hp/atk/def/keys/gold/inventory/
    //    flags/kills/monsters_alive——计划 §3.3 十字段，编译期防漂移）。
    let s = SolverState {
        floor: 51,
        hp: DiscreteHp(300),
        atk: 10,
        def: 10,
        keys: (0, 0, 0),
        gold: DiscreteGold(0),
        inventory: CanonicalBag::from_player(&player()),
        flags: ScriptFlags(0),
        kills: 0,
        monsters_alive: 0,
    };
    assert_eq!(s.floor, 51);
    // ② 裁定①：hp 桶宽=1=精确整数（Q1 要精确值，Pareto 已压缩状态面）。
    assert_eq!(DiscreteHp::bucket(299).0, 299);
    assert_eq!(DiscreteHp::bucket(300).0, 300);
    assert_ne!(DiscreteHp::bucket(299), DiscreteHp::bucket(300));
    // ③ 背包 canonical：按 item_stack_size 前缀截断——尾槽垃圾不进去重键。
    let mut a = player();
    a.item_types = vec![0u8; 32];
    a.item_uses = vec![0u8; 32];
    a.item_types[0] = 15;
    a.item_stack_size = 1;
    let mut b = a.clone();
    b.item_types[5] = 99; // 越过 size 的尾部垃圾（不同字节、同语义）
    b.item_uses[7] = 42;
    assert_eq!(
        CanonicalBag::from_player(&a).key(),
        CanonicalBag::from_player(&b).key(),
        "canonical：size 前缀外的尾槽不进键"
    );
    assert_ne!(
        CanonicalBag::from_player(&a).key(),
        CanonicalBag::from_player(&player()).key(),
        "size 内真差异必须进键"
    );

    // ④ monsters_alive 全局口径 = 已访层位图存活(cat8) + 未访层 sprite 全活(cat8)：
    //    floor51_m41 有 type41@(3,1) 存活(1) + floor50 型72 已杀(0) + floor49 未访(2) = 3。
    let mut floors: BTreeMap<i32, LevelSpec> = BTreeMap::new();
    floors.insert(51, floor51_roundtrip_m41());
    floors.insert(50, floor50_roundtrip());
    floors.insert(49, floor49_two());
    let input = GlobalInput {
        floors,
        start_floor: 51,
        start_pos: (0, 1),
        start_player: player(),
        start_flags: 0,
        bound: 10_000,
    };
    let r = solve_global(&input);
    assert_eq!(r.confidence, "optimal", "置信：{r:?}");
    let g = r
        .goals
        .iter()
        .find(|q| q.floors == vec![51, 50] && q.kills == 1)
        .unwrap_or_else(|| panic!("A 目标应存在：{r:?}"));
    assert_eq!(g.monsters_alive, 3, "怪物存亡口径：{g:?}");

    // ⑤ to_solver_state 往返：目标→计划 §3.3 字段形状。
    let st = to_solver_state(g);
    assert_eq!(st.floor, 50);
    assert_eq!(st.hp.0, 276);
    assert_eq!(st.atk, 10);
    assert_eq!(st.def, 10);
    assert_eq!(st.keys, (1, 0, 0));
    assert_eq!(st.kills, 1);
    assert_eq!(st.monsters_alive, 3);
}

/// C4 夹具：floor51_roundtrip 变体——row1 扩到 x0..4 + type41@(3,1)
/// （路径 (0,1)→(1,1)→(2,1) 不经 (3,1) ⇒ A 直下解不杀它 ⇒ 存活计入口径）。
fn floor51_roundtrip_m41() -> LevelSpec {
    let mut lv = floor51_roundtrip();
    lv.terrain[5 + 3] = true; // (3,1) 可走
    lv.terrain[5 + 4] = true; // (4,1) 可走
    lv.entities.push(EntSpec { ty: 41, x: 3, y: 1, param: 0 });
    lv
}

/// C4 夹具：floor49 未访层——3×3 全可走、2 只 type41、无出口（全局无边可达）。
fn floor49_two() -> LevelSpec {
    LevelSpec {
        cols: 3,
        rows: 3,
        terrain: vec![true; 9],
        start: (1, 1),
        entities: vec![
            EntSpec { ty: 41, x: 0, y: 0, param: 0 },
            EntSpec { ty: 41, x: 2, y: 2, param: 0 },
        ],
        exits: vec![],
        player: player(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    }
}

/// C5：真数据全局输入（计划 §4 批3 判据③ 的 lib 腿；装载范式=census_snapshot 同构，
/// I/O 在测试面——A9 只扫 src/*.rs 排除 bin/solver.rs）。
fn real_global_input(bound: u64) -> GlobalInput {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();
    let mut maplvs: Vec<Vec<u8>> = Vec::new();
    let mut sprites: Vec<Vec<u8>> = Vec::new();
    for i in 0..55 {
        maplvs.push(std::fs::read(root.join(format!("assets/raw/maplv{i}"))).unwrap());
        sprites.push(std::fs::read(root.join(format!("assets/raw/sprite{i}"))).unwrap());
    }
    let maplv_refs: Vec<&[u8]> = maplvs.iter().map(|v| v.as_slice()).collect();
    let sprite_refs: Vec<&[u8]> = sprites.iter().map(|v| v.as_slice()).collect();
    let si = game_solver::ingest::ingest(
        &maplv_refs,
        &sprite_refs,
        include_str!("../../game-core/tests/fixtures/level_scripts.txt"),
        include_str!("../../game-core/tests/fixtures/door12.txt"),
        include_str!("../../game-core/tests/fixtures/door13.txt"),
        include_str!("../../../data/tables/enemies.json"),
        include_str!("../../../data/tables/items.json"),
        include_str!("../../../data/tables/shop_goods.json"),
    )
    .expect("ingest 失败");
    let nums = |t: &str| -> Vec<i32> {
        t.lines()
            .map(|l| l.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().expect("fixture 整数"))
            .collect()
    };
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
    let scaled = scale_enemy_stats(&base, 1); // difficulty=0 ⇒ ×1（BR-4）
    let tile_walk = nums(include_str!("../../game-core/tests/fixtures/tile_walkability.txt"));
    let mk_player = || PlayerCombat {
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
    };
    let mut floors: BTreeMap<i32, LevelSpec> = BTreeMap::new();
    for f in 1..=54 {
        if let Ok(inp) = load_floor(
            &si,
            f,
            (1, 11),
            PlayerSpec::from_combat(mk_player()),
            scaled.clone(),
            base.clone(),
            &tile_walk,
        ) {
            if let Some(lv) = inp.level {
                floors.insert(f as i32, lv);
            }
        }
    }
    GlobalInput {
        floors,
        start_floor: 51,
        start_pos: (1, 11), // handoff.rs:154 load_floor(32,352) ⇒ (1,11)
        start_player: PlayerSpec::from_combat(mk_player()),
        start_flags: 0,
        bound,
    }
}

/// C5 端到端：真数据全局求解确定性双跑（计划 §4 批3 判据③ 同输入同解）。
#[test]
fn solve_global_t真数据端到端_确定性双跑() {
    let j1 = game_solver::solve_global::solve_global(&real_global_input(500)).to_json();
    let j2 = game_solver::solve_global::solve_global(&real_global_input(500)).to_json();
    assert_eq!(j1, j2, "同输入同解（计划 §4 ④ 确定性）");
    assert!(j1.starts_with("{\"confidence\":"), "JSON 形状头：{j1:.120}");
    assert!(j1.contains("\"bound\":500"), "bound 回显：{j1:.200}");
    assert!(
        [
            "\"confidence\":\"optimal\"",
            "\"confidence\":\"bounded-best\"",
            "\"confidence\":\"feasible\""
        ]
        .iter()
        .any(|c| j1.contains(c)),
        "置信三态：{j1:.200}"
    );
}

/// 批3 C5a 正式测试（doc 误标 TEMP 已更正）：真数据首层穿门可达
#[test]
fn solve_global_t真数据首层穿门可达() {
    // 批3 C5a 红：真数据层51 从入场梯 (1,11) 出发必须能拿到出口梯 (2,2) 的
    // Stair 目标——到梯必经 9 个场景门（纯演出脚本 TAK/ROS/SEE 跑完回 Map），
    // batch2 保守口径 tickdrive None→Dropped(⊥) 卡死首层（判据③首跑空心根因）。
    let gi = real_global_input(1_000);
    let lv = gi.floors.get(&51).expect("层51 规格");
    let si = SolveInput {
        floor: 51,
        difficulty: 0,
        level: Some(lv.clone()),
    };
    let (ok, goals) = solve_layer(&si, (1, 11), None, player(), 0);
    assert!(ok, "真数据首层入场可达（穿演出门）：goals={goals:?}");
    assert!(
        goals.iter().any(|g| g.kind == ExitKind::Stair),
        "应有出口梯 (2,2) Stair 目标：{goals:?}"
    );
}

/// 批3 F3：UseItem 符21 上行换层边（a.java case21=changeFloor(currentFloor+1,
/// false, false)、var3=false 无区间门；itemUseCounts=1 a.java:944 idx8 ⇒ 成功
/// 使用即消耗、post 必须扣 21）。f49 拾 21 → use → f50 落点走 swap_floor 特例
/// (6,7)（scene_persist.rs:410）→ 型72 终局。exits=[] = 手建 LevelSpec 不复刻
/// load_floor 全量梯入出口（fixture quirk，真实数据 C5 已载）。
fn floor49_useitem() -> LevelSpec {
    let mut terrain = vec![false; 5 * 3];
    for x in 0..5 {
        terrain[1 * 5 + x] = true; // 中排全走
    }
    LevelSpec {
        cols: 5,
        rows: 3,
        terrain,
        start: (0, 1),
        entities: vec![EntSpec { ty: 21, x: 1, y: 1, param: 0 }],
        exits: Vec::new(),
        player: player(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    }
}

/// f50 夹具：落点特例 (6,7) → ty72 邻格 bump 终局（与 floor50_nodoor 同构）。
fn floor50_useitem() -> LevelSpec {
    let mut terrain = vec![false; 9 * 9];
    for x in 5..=8 {
        terrain[7 * 9 + x] = true;
    }
    LevelSpec {
        cols: 9,
        rows: 9,
        terrain,
        start: (6, 7),
        entities: vec![EntSpec { ty: 72, x: 7, y: 7, param: 0 }],
        exits: Vec::new(),
        player: player(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    }
}

#[test]
fn solve_global_tuseitem符21上行换层边_拾符跨层消耗与通关终局() {
    // 层内腿：f49 拾 21 → UseItem 边 (50,false)、post 扣 21、起跳不耗步
    let si49 = SolveInput {
        floor: 49,
        difficulty: 0,
        level: Some(floor49_useitem()),
    };
    let (ok1, g1) = solve_layer(&si49, (0, 1), None, player(), 0);
    assert!(ok1, "f49 可达");
    // C5d 归并按 (kind,cross_edge,player,flags,pos,ents) 分组、BTreeMap 按 pos 排序
    // ⇒ 首项顺序不稳（拾符后走回起点 (0,1) 的态也合法携21）——选择器钉 pos。
    let u = g1
        .iter()
        .find(|g| g.kind == ExitKind::UseItem && g.post_pos == (1, 1))
        .expect("携21 态应记录 UseItem 边");
    assert_eq!(u.cross_edge, Some((50, false)), "case21=+1 上行 went_down=false");
    assert!(!u.post.item_types.contains(&21), "首用即耗 itemUseCounts=1");
    assert_eq!(u.post_pos, (1, 1), "起跳不耗步、位置不变");
    assert_eq!(u.steps, 1, "拾符那步");
    assert_eq!(u.route, vec!["0,1".to_string(), "1,1".to_string()]);

    // 全局腿：f49 →（UseItem 消耗21）→ f50 落 (6,7) → bump 型72 → Ending
    let mut floors = BTreeMap::new();
    floors.insert(49, floor49_useitem());
    floors.insert(50, floor50_useitem());
    let gi = GlobalInput {
        floors,
        start_floor: 49,
        start_pos: (0, 1),
        start_player: player(),
        start_flags: 0,
        bound: 10_000,
    };
    let r = solve_global(&gi);
    assert_eq!(r.confidence, "optimal", "置信：{r:?}");
    let g = r
        .goals
        .iter()
        .find(|g| g.floors == vec![49, 50])
        .expect("携21 跨层到 50 的终局解");
    assert_eq!(g.kills, 1, "型72 一杀：{:?}", g);
    assert_eq!(g.hp, 276, "300-24");
    assert!(
        g.route.contains(&"bump 72@7,7".to_string()),
        "route：{:?}",
        g.route
    );
    assert!(!g.post.item_types.contains(&21), "终局背包无 21");
}

// ─── 批3 C5d：门脚本世界变保守放行（选项 D，lead 已报备） ───

/// 真数据 f52：上行梯 (11,11)→53 只能穿 gate53（line53=CES_36/40 物品+DES_11
/// 拆障+演出）与 gate50（CES_6 楼梯）。保守放行（物品/楼梯 CES、DES=开路方向、
/// 位图陈旧=更少路径=健全）后该 goal 必须存在；怪物/阻挡 CES、实体 MOV 仍拒。
#[test]
fn solve_global_t门实体变保守放行_f52穿gate53达上行梯() {
    let gi = real_global_input(1_000);
    let lv = gi.floors.get(&52).expect("层52 规格");
    let si = SolveInput { floor: 52, difficulty: 0, level: Some(lv.clone()) };
    let (ok, goals) = solve_layer(&si, (2, 11), None, player(), 0);
    assert!(ok, "f52 应可达：{goals:?}");
    assert!(
        goals.iter().any(|g| g.kind == ExitKind::Stair && g.post_pos == (11, 11)),
        "上行梯 (11,11) goal 应存在（gate53/gate50 放行）：{goals:?}"
    );
}

/// 玩家 MOV_0_*（script31 `MOV_0_6_5`）穿门后 pos 必须读回引擎真值：隔墙布局
/// （x=3 全墙），不读回 pos 则停在门格 (1,1) 永远到不了墙后出口 (6,6)。
#[test]
fn solve_global_t门玩家MOV_pos读回_穿门达墙后出口() {
    // 夹具修正（C5d 调试实证）：①墙=ty10 实体列——find_path 地形单源
    // （pathfind.rs is_cell_walkable 只查 terrain）穿实体、solver classify 挡 ty10
    // ⇒ 门内 MOV_0_6_5 是唯一跨墙途径（原地形墙两头都挡=测试前提错）；
    // ②case31 门 gate &= from_walk && find_item(16)>=0（scene.rs:1016）⇒ 须持16。
    let terrain = vec![true; 7 * 7];
    let ents = vec![
        EntSpec { ty: 83, x: 1, y: 1, param: 31 },
        EntSpec { ty: 7, x: 6, y: 6, param: 2 },
    ];
    // 无墙（C5d 实证改版）：MOV=引擎「直达走位」——find_path 规划穿实体槽、
    // try_step 撞格即中止（advance_script_walk 撞格臂），墙在路径上 walk 必死；
    // pos 读回改由路线形状证明（无读回 ⇒ BFS 从门格重走 ~10 token）。
    let mut p16 = player();
    p16.item_types = vec![16];
    p16.item_uses = vec![1];
    p16.item_stack_size = 1;
    let level = LevelSpec {
        cols: 7,
        rows: 7,
        terrain,
        start: (0, 1),
        entities: ents,
        exits: vec![ExitSpec { x: 6, y: 6, kind: ExitKind::Stair, index: 1 }],
        player: p16.clone(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    };
    let si = SolveInput { floor: 26, difficulty: 0, level: Some(level) };
    let (ok, goals) = solve_layer(&si, (0, 1), None, p16, 0);
    assert!(ok, "MOV 穿门（pos 读回 (6,5)）后应达出口：{goals:?}");
    // pos 读回形状断言：读回 (6,5) ⇒ 出口 (6,6) 一步即达（route=4 token）；
    // 读回缺失（停门格 (1,1)）⇒ BFS 重走整格 ⇒ route ≥13 token 被杀。
    // 读回实证（C5d 调试钉死的形状）：Completed 臂不加 gate 标记（非 Terminal），
    // C6a 后 route token=act 到达格：门格节点 token="1,1"（读回 pos 不进
    // route）、全程 3 token 到出口——无读回时 BFS 从 (1,1) 重走 ≥13 token 被杀；
    // 读回缺失则门格 token="1,1"+BFS 重走 ⇒ ≥13 token。
    let shaped = goals.iter().any(|g| {
        g.post_pos == (6, 6)
            && g.route.last().map(String::as_str) == Some("6,6")
            && g.route.len() == 3
            && g.route[1] == "1,1"
            && g.post_flags != 0
    });
    assert!(shaped, "MOV pos 读回应让出口一步即达：{goals:?}");
}

/// 批3 C6b（A-lite，lead 批准 2026-11-13）：怪物 CES 门不再 Dropped——
/// 穿门读回引擎实体表，CES 新实体进 post_ents。原「拒绝面」测试（选项 D
/// 口径）反向改写：line38=`CES_6_4_1 CES_61_4_1`（type6 中性楼梯+type61 怪
/// 同格 append），墙隙唯一通路穿门 ⇒ 分类器退役后 goal 必须产生。
#[test]
fn solve_global_t门怪物CES读回_分支产生() {
    let mut terrain = vec![true; 6 * 3];
    terrain[0 * 6 + 2] = false; // x=2 全墙，仅 row1 通
    terrain[2 * 6 + 2] = false;
    let level = LevelSpec {
        cols: 6,
        rows: 3,
        terrain,
        start: (0, 1),
        entities: vec![
            EntSpec { ty: 83, x: 2, y: 1, param: 38 },
            EntSpec { ty: 7, x: 5, y: 1, param: 2 },
        ],
        exits: vec![ExitSpec { x: 5, y: 1, kind: ExitKind::Stair, index: 1 }],
        player: player(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    };
    let si = SolveInput { floor: 48, difficulty: 0, level: Some(level) };
    let (ok, goals) = solve_layer(&si, (0, 1), None, player(), 0);
    assert!(ok, "穿怪物 CES 门后应可达（A-lite 读回续走）：{goals:?}");
    // CES_6_4_1 生成的中性楼梯 type6@(4,1) 永久在场（line38 无 DES）——
    // post_ents 含它=脚本世界变读回进层间态的直接证据。
    assert!(
        goals.iter().any(|g| {
            g.kind == ExitKind::Stair
                && g.post_ents.iter().any(|r| r.ty == 6 && r.x == 4 && r.y == 1)
        }),
        "CES_6_4_1 生成实体应读回进 post_ents：{goals:?}"
    );
}


/// 批3 C6a：穿门读回引擎实体表（A-lite）——脚本 CES 新实体进 `post_ents`
/// （canonical=引擎活体槽序：sprite 序幸存者+append 序新增；entity.rs:330 append-only）。
#[test]
fn solve_global_t穿门读回CES实体_进post_ents() {
    // 8×8 全可走；gate p25@(1,0)（fixture line26 idx25=`CES_22_6_6`，type22=cat2 拾取物）；
    // 出口梯 @(2,0)。entry (0,0)→踩门 Completed→CES 生成→走到出口。
    // 门条件（scene.rs case25）：`from_walk && flags[13] && 无型60@(2,2)/(10,2)`
    // ⇒ 根 flags=1<<13（首跑红=没置 flags[13]、门不 fire、脚本没跑）。
    let level = LevelSpec {
        cols: 8,
        rows: 8,
        terrain: vec![true; 64],
        start: (0, 0),
        entities: vec![
            EntSpec { ty: 83, x: 1, y: 0, param: 25 },
            EntSpec { ty: 7, x: 2, y: 0, param: 50 },
        ],
        exits: vec![ExitSpec { x: 2, y: 0, kind: ExitKind::Stair, index: 1 }],
        player: player(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    };
    let si = SolveInput { floor: 1, difficulty: 0, level: Some(level) };
    let (ok, goals) = solve_layer(&si, (0, 0), None, player(), 1 << 13);
    assert!(ok, "穿门到出口应可达");
    // 夹具 8×8 全可走⇒门格非独路：存在「绕行拾取 type22 后再出门」解（post_ents
    // 不含22）与「直穿门携 CES」解两支；取携 CES 者断言（merge 排序后 find 首项
    // 可能是拾取支——断言打脸过一次）。引擎 append 序：CES 新实体在槽尾。
    let g = goals
        .iter()
        .find(|g| {
            g.kind == ExitKind::Stair
                && g.post_ents.contains(&EntRec { ty: 22, x: 6, y: 6, param: 0 })
        })
        .expect("携 CES 实体的 Stair goal 应存在");
    assert!(
        g.post_ents.contains(&EntRec { ty: 22, x: 6, y: 6, param: 0 }),
        "CES_22_6_6 生成实体应读回进 post_ents：{:?}",
        g.post_ents
    );
    assert_eq!(g.post_ents.len(), 3, "sprite 两实体+CES 新增一（append 序）");
    assert_eq!(
        g.post_ents.last(),
        Some(&EntRec { ty: 22, x: 6, y: 6, param: 0 }),
        "CES 新实体在引擎槽尾（append-only）"
    );
}

/// 批3 C6a：穿门读回——脚本 DES 拆除的实体出 `post_ents`（alive 谓词=
/// `serialize_floor_state` 逐字 `!removed && state != 1`，entity.rs:86-88）。
#[test]
fn solve_global_t穿门读回DES拆除_出post_ents() {
    // gate p14@(1,0)（fixture file line15 idx14=`ROS_4_1 TAK_92_97 DES_73_6_8
    // TAK_98_100 DES_70_6_7`——14 号门 case14 直通（scene.rs 直通簇注释）、
    // 行内 op 全 D-放行；原 p18 被 case18 六格型1 扫描挡死=首跑红根因）；
    // 预置 type70@(6,7)。穿门后 DES_70_6_7 拆除读回。
    let level = LevelSpec {
        cols: 8,
        rows: 8,
        terrain: vec![true; 64],
        start: (0, 0),
        entities: vec![
            EntSpec { ty: 83, x: 1, y: 0, param: 14 },
            EntSpec { ty: 70, x: 6, y: 7, param: 0 },
            EntSpec { ty: 7, x: 2, y: 0, param: 50 },
        ],
        exits: vec![ExitSpec { x: 2, y: 0, kind: ExitKind::Stair, index: 1 }],
        player: player(),
        enemies: enemies(),
        enemy_base: enemies(),
        flags: 0,
    };
    let si = SolveInput { floor: 1, difficulty: 0, level: Some(level) };
    let (ok, goals) = solve_layer(&si, (0, 0), None, player(), 0);
    assert!(ok, "穿门到出口应可达");
    let g = goals
        .iter()
        .find(|g| g.kind == ExitKind::Stair)
        .expect("Stair goal");
    assert!(
        !g.post_ents.iter().any(|r| r.ty == 70 && r.x == 6 && r.y == 7),
        "DES_70_6_7 拆除应读回：{:?}",
        g.post_ents
    );
    assert!(
        g.post_ents.iter().any(|r| r.ty == 83 && r.x == 1 && r.y == 0),
        "门与出口梯保留：{:?}",
        g.post_ents
    );
}

/// C6f 红：层10 上行梯可达（隐形81 封印门穿行=引擎语义，b63/迭代65 实证）
#[test]
fn solve_global_t穿隐形81封印_层10上行梯可达() {
    let input = real_global_input(1_000);
    let lv = input.floors.get(&10).expect("f10").clone();
    let si = SolveInput { floor: 10, difficulty: 0, level: Some(lv) };
    let mut p = player();
    p.atk = 510;
    p.def = 510;
    p.yellow = 9;
    p.blue = 9;
    p.red = 9;
    let (_ok, goals) = solve_layer(&si, (1, 10), None, p, 0);
    let up = goals
        .iter()
        .any(|g| g.kind == ExitKind::Stair && g.post_pos == (6, 1));
    assert!(up, "层10 上行梯 (6,1) 应可达：stair goals={:?}",
        goals.iter().filter(|g| g.kind == ExitKind::Stair)
            .map(|g| g.post_pos).collect::<Vec<_>>());
}

/// C6g 红：根态恰在出口仍须扩展（step_off 四邻皆墙留梯格场景）
#[test]
fn solve_global_t节点上限触发_诚实降级() {
    // 批3 lead 裁定 A 同族：全局节点上限（R1 有界可配）——触界必须可观测
    // （node_cap_hit 入 JSON）且 confidence 降级 bounded-best，不得冒充 optimal。
    let mut floors = BTreeMap::new();
    floors.insert(51, floor51());
    floors.insert(50, floor50()); // 缺50 规格则子层被丢、根后即队列空，触不到 cap
    let gi = GlobalInput { floors, start_floor: 51, start_pos: (0, 1), start_player: player(), start_flags: 0, bound: 10_000 };
    let r = game_solver::solve_global::solve_global_capped(&gi, 2);
    assert!(r.node_cap_hit, "cap=2 应触界：{r:?}");
    // 无解+触界 → feasible（无解词表优先；触界降级 optimal 的路径=同
    // `hit_bound||node_cap_hit||truncated` 判据，有解时才可能 optimal→bounded-best）
    assert_eq!(r.confidence, "feasible", "触界不得冒充 optimal：{r:?}");
    assert!(r.to_json().contains("\"node_cap_hit\":true"), "{}", r.to_json());
    // 未触界对照：大 cap 不得误报
    let r2 = game_solver::solve_global::solve_global_capped(&gi, 1_000_000);
    assert!(!r2.node_cap_hit, "大 cap 不应触界：{r2:?}");
}

#[test]
fn solve_global_t根在出口仍扩展() {
    let input = real_global_input(1_000);
    let lv = input.floors.get(&10).expect("f10").clone();
    let si = SolveInput { floor: 10, difficulty: 0, level: Some(lv) };
    let mut p = player();
    p.atk = 510;
    p.def = 510;
    p.yellow = 9;
    p.blue = 9;
    p.red = 9;
    let (_ok, goals) = solve_layer(&si, (1, 11), None, p, 0);
    assert!(
        goals.len() > 1,
        "根在出口 (1,11) 应记录后仍扩展出更多目标：goals={}",
        goals.len()
    );
}
