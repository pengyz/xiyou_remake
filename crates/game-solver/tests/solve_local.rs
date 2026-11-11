//! 【求解器批2】T1 存在性闸（提案 §3.1 T1；计划 :328 `-- --list | grep -c solve_local ≥1`）。
//!
//! 断言口径：只钉 **JSON 形状/字段类型**（防空过），不钉具体数值——数值断言归 T3/T6/T10，
//! 本测试跨切片恒绿。
//!
//! C2 起追加 T2（提案 §3.1 T2 / 计划 :334 TDD 起手式）：3×3 手工夹具可达性 + 战斗 Refused。
use game_data::defs::EnemyDef;
use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_core::scene_test_support::scene;
use game_solver::solve_local::{
    probe_bump, probe_gate, solve_local, EntSpec, ExitKind, ExitSpec, LevelSpec, PlayerSpec,
    SolveInput,
};
use std::path::PathBuf;

/// 取 `"key":` 之后的标量值 token（到下一个顶层 `,` 为止）；route 数组除外（取到末尾 `}` 前）。
fn scalar<'a>(json: &'a str, key: &str) -> &'a str {
    let pat = format!("\"{key}\":");
    let i = json.find(&pat).unwrap_or_else(|| panic!("缺字段 {key}：{json}"));
    let rest = &json[i + pat.len()..];
    let end = rest.find(',').unwrap_or(rest.rfind('}').expect("JSON 无右括号"));
    &rest[..end]
}

#[test]
fn p2s1_t1_solve_local存在性闸与json形状() {
    // T1 存在性闸：本测试名含 `solve_local` ⇒ `cargo test -p game-solver -- --list | grep -c solve_local ≥1`
    let r = solve_local(&SolveInput::placeholder(1));
    let j = r.to_json();

    // 逐字段类型断言（T6 形状的前身；字段序固定=确定性要求，§3.3 :257）
    assert_eq!(scalar(&j, "floor"), "1", "floor 为 int：{j}");
    assert_eq!(scalar(&j, "difficulty"), "0", "difficulty=int（BR-4 占位字段）：{j}");
    let reach = scalar(&j, "reachable");
    assert!(
        reach == "true" || reach == "false",
        "reachable 为 bool：{j}"
    );
    for key in ["max_hp", "min_kills"] {
        let v = scalar(&j, key);
        let ok = v == "null" || v.parse::<i32>().is_ok();
        assert!(ok, "{key} 为 int|null：{j}");
    }
    let conf = scalar(&j, "confidence");
    assert!(
        matches!(conf, "\"optimal\"" | "\"bounded-best\"" | "\"feasible\""),
        "confidence 三态枚举：{j}"
    );
    // route=[...]（末字段）：元素须为字符串、逗号分隔
    let ri = j.find("\"route\":[").unwrap_or_else(|| panic!("缺 route：{j}"));
    let rest = &j[ri + "\"route\":".len()..];
    assert!(rest.ends_with('}'), "route 是末字段、其后为右括号：{j}");
    let arr = &rest[..rest.len() - 1]; // 剥掉最外层 }
    assert!(arr.starts_with('[') && arr.ends_with(']'), "route 为数组：{j}");
    let inner = &arr[1..arr.len() - 1];
    if !inner.is_empty() {
        assert!(inner.starts_with('"') && inner.ends_with('"'), "route 元素为字符串：{j}");
        for part in inner.split("\",\"") {
            assert!(!part.is_empty(), "route 元素非空：{j}");
        }
    }
}

// ================= C2（提案 §3.1 T2；计划 :334 TDD 起手式）=================

/// 夹具整数表按行解析（与 game-core `scene_test_support::numbers` 同式）。
fn nums(text: &str) -> Vec<i32> {
    text.lines()
        .map(|l| l.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().expect("夹具整数"))
        .collect()
}

/// 敌表：**与引擎侧同源**（game-core 四 fixture → `CombatTables::to_records` → `scale ×1`，
/// 即 `scene_test_support::scene()` 内部同一装配）——BR-2 双侧断言要求两侧输入逐位相同。
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
    let scaled = scale_enemy_stats(&base, 1); // difficulty=0 ⇒ ×1（BR-4）
    (base, scaled)
}

/// 玩家（无道具/无装备 ⇒ `effective_attack_vs_type` 恒等于 atk，DC-9 分叉面不存在）。
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

/// T2 手工 3×3 夹具：单走廊（中行 y=1 可走，余墙）——入口 (0,1)、type41 强怪挡路 (1,1)、
/// Exit#1 type7 楼梯 (2,1)。怪是唯一通路 ⇒ 杀不过即封死。
fn fixture_3x3(p: PlayerCombat) -> SolveInput {
    let (base, scaled) = enemy_tables();
    let level = LevelSpec {
        cols: 3,
        rows: 3,
        // 行主序：y=0 全墙、y=1 全可走、y=2 全墙
        terrain: vec![false, false, false, true, true, true, false, false, false],
        start: (0, 1),
        entities: vec![
            EntSpec {
                ty: 41,
                x: 1,
                y: 1,
                param: 0,
            },
            EntSpec {
                ty: 7,
                x: 2,
                y: 1,
                param: 2,
            },
        ],
        exits: vec![ExitSpec {
            x: 2,
            y: 1,
            kind: ExitKind::Stair,
            index: 1,
        }],
        player: PlayerSpec::from_combat(p),
        enemies: scaled,
        enemy_base: base,
        flags: 0,
    };
    SolveInput {
        floor: 1,
        difficulty: 0,
        level: Some(level),
    }
}

/// 引擎侧同输入场景（BR-2 双侧断言的 harness 腿）：13×13 合成层行 y=3 可走，
/// 玩家 (1,3)、type41 (2,3) —— 与 3×3 夹具的局部三元组（入口-怪-出口）同构。
fn engine_side(p: PlayerCombat) -> game_core::scene::GameScene {
    let mut sc = scene((1, 3), &[(41u8, 2, 3)]);
    sc.persist.floor = 1; // 与夹具楼层同号 ⇒ entity_category 单源同判
    sc.player = p;
    sc
}

#[test]
fn p2s1_t2a_solve_local_3x3强怪挡路_能杀则reachable() {
    // type41 数值（enemybasehp/atk/def.txt 首行）：hp35 atk18 def1。
    // 玩家 atk10 > def1 ⇒ hits=ceil(35/9)=4；bump 面 no_counter=true（`battle.rs:117`）
    // ⇒ loss=(18-10)×(4-1)=24 < hp300 ⇒ `resolve_bump` `:121` 门放行 Engage。
    let input = fixture_3x3(player(300, 10, 10));

    let r = solve_local(&input);
    assert!(r.reachable, "能杀 ⇒ reachable=true：{r:?}");
    // BR-6：路过怪格必须建模 BumpEnemy，不许白走；route[0]=入口（对齐 A1 样例 route 起点=spawn）
    assert_eq!(
        r.route,
        vec!["0,1", "bump 41@1,1", "2,1"],
        "路线=入口→bump→出口，逐动作航点"
    );
    // 探针与搜索同一实现（非自证：值=直调 predict_engagement 的闭式结果 24）
    assert_eq!(probe_bump(&input, 41), Some(24), "Engage ⇒ 直调取损 24");

    // —— BR-2 双侧断言（引擎腿）：同输入 ⇒ 同结论（开战）——
    let mut sc = engine_side(player(300, 10, 10));
    let ok = sc.try_step(2); // 右=怪格 (2,3)
    assert!(!ok, "怪格 allowed=false ⇒ 不放行");
    assert_eq!(sc.walk_phase, 5, "Engage ⇒ 引擎侧开战（scene.rs:3598）");
    assert!(sc.battle_active, "Engage ⇒ battle_active=true");
}

#[test]
fn p2s1_t2b_solve_local_3x3强怪挡路_杀不过refused不计杀() {
    // 同 atk/def，hp10 < loss24 ⇒ `battle.rs:121` `!(loss>=0 && loss<hp)` ⇒ Refused
    let input = fixture_3x3(player(10, 10, 10));

    let r = solve_local(&input);
    assert!(!r.reachable, "杀不过 ⇒ 被封死（T(s,a)=⊥ 不产生状态）：{r:?}");
    assert!(r.route.is_empty(), "Refused ⇒ 无解 ⇒ 路线为空（不计杀）");
    assert_eq!(probe_bump(&input, 41), None, "Refused ⇒ 探针 None（不计杀）");

    // —— BR-2 双侧断言（引擎腿）：同输入 ⇒ 同结论（拒战、不开战）——
    let mut sc = engine_side(player(10, 10, 10));
    let ok = sc.try_step(2);
    assert!(!ok, "怪格 allowed=false");
    assert_ne!(sc.walk_phase, 5, "Refused ⇒ 不开战");
    assert!(!sc.battle_active, "Refused ⇒ battle_active=false");
    assert_eq!(
        sc.dialog.overlay_text.as_deref(),
        Some("你无法战胜它"),
        "Refused ⇒ 拒战浮层（scene.rs:3626）"
    );
}

/// 航点里 `bump ` 前缀数 = 该路杀怪数（Refused ⇒ 不产生航点，§1.1「⊥ 不产生状态」）。
fn bumps(route: &[String]) -> i32 {
    route.iter().filter(|w| w.starts_with("bump ")).count() as i32
}

/// T3 双查询分叉夹具（5×3，下排整墙）：
/// - 中排直穿：type41 三连挡 (1,1)(2,1)(3,1)，4 步 3 杀；
/// - 上排绕行：(1,0)(3,0) 两怪，6 步 2 杀——全图无 1 杀路（x∈{1,2,3} 两排必穿其二，下排墙）。
/// 玩家 def18 ≥ 怪 atk18 ⇒ 每战 loss=0（combat.rs:106-131）⇒ 两路 hp 同为 300
/// ⇒ max_hp 并列走 tie-break（步数少 ⇒ 直穿 3 杀）、min_kills 取 2 杀路
/// ⇒ 双查询独立出解（§1.4「双查询独立出解」实证）。
fn fixture_split() -> SolveInput {
    let (base, scaled) = enemy_tables();
    let level = LevelSpec {
        cols: 5,
        rows: 3,
        // 行主序：y=0 全可走、y=1 全可走、y=2 全墙
        terrain: vec![
            true, true, true, true, true, true, true, true, true, true, false, false, false,
            false, false,
        ],
        start: (0, 1),
        entities: vec![
            EntSpec { ty: 41, x: 1, y: 1, param: 0 },
            EntSpec { ty: 41, x: 2, y: 1, param: 0 },
            EntSpec { ty: 41, x: 3, y: 1, param: 0 },
            EntSpec { ty: 41, x: 1, y: 0, param: 0 },
            EntSpec { ty: 41, x: 3, y: 0, param: 0 },
            EntSpec { ty: 7, x: 4, y: 1, param: 2 },
        ],
        exits: vec![ExitSpec {
            x: 4,
            y: 1,
            kind: ExitKind::Stair,
            index: 1,
        }],
        player: PlayerSpec::from_combat(player(300, 10, 18)),
        enemies: scaled,
        enemy_base: base,
        flags: 0,
    };
    SolveInput {
        floor: 1,
        difficulty: 0,
        level: Some(level),
    }
}

/// T3 tie-break 夹具（3×2 全可走）：S=(2,1)、出口 (1,0)。两条等长 2 步路：
/// - 上路（BFS 首见，DIRS 序「上」优先）：2,1 → 2,0 → 1,0
/// - 左路（字典序更小）：2,1 → 1,1 → 1,0
/// hp/kills/steps 全并列 ⇒ §3.3 取字典序最小路线；只取 BFS 首见会得上路 ⇒ 断言可判别。
fn fixture_tie() -> SolveInput {
    let (base, scaled) = enemy_tables();
    let level = LevelSpec {
        cols: 3,
        rows: 2,
        terrain: vec![true; 6],
        start: (2, 1),
        entities: vec![EntSpec {
            ty: 7,
            x: 1,
            y: 0,
            param: 2,
        }],
        exits: vec![ExitSpec {
            x: 1,
            y: 0,
            kind: ExitKind::Stair,
            index: 1,
        }],
        player: PlayerSpec::from_combat(player(300, 10, 10)),
        enemies: scaled,
        enemy_base: base,
        flags: 0,
    };
    SolveInput {
        floor: 1,
        difficulty: 0,
        level: Some(level),
    }
}

/// T3（提案 §3.1 T3 / 计划 :330「hp/kills 单调性断言」）：三断言 (i)-(iii) + 并列 tie-break。
#[test]
fn p2s1_t3_solve_local_目标双查询单调性与tie_break() {
    // (i) reachable=false ⇒ max_hp=min_kills=None（T2b 同款杀不过封死）
    let r = solve_local(&fixture_3x3(player(10, 10, 10)));
    assert!(!r.reachable, "(i) 杀不过 ⇒ 封死：{r:?}");
    assert_eq!(r.max_hp, None, "(i) reachable=false ⇒ max_hp=None");
    assert_eq!(r.min_kills, None, "(i) reachable=false ⇒ min_kills=None");

    // (ii)+(iii) 走廊夹具：唯一 Exit#1 路=必杀 1 怪（loss=24）
    let r = solve_local(&fixture_3x3(player(300, 10, 10)));
    assert!(r.reachable, "(ii) 可达");
    let max_hp = r.max_hp.expect("(ii) reachable=true ⇒ Some");
    assert!(max_hp <= 300, "(ii) max_hp ≤ hp(s0)=300：{max_hp}");
    assert_eq!(max_hp, 276, "唯一路 300-24");
    let min_kills = r.min_kills.expect("(iii) Exit#1 域非空 ⇒ Some");
    assert_eq!(
        bumps(&r.route_min_kills),
        min_kills,
        "(iii) route_min_kills 取等号"
    );
    assert!(
        min_kills <= bumps(&r.route),
        "(iii) route_max_hp 终态∈Exit#1 ⇒ 域内下界"
    );

    // (iii) 分叉夹具：min_kills 走 2 杀路、max_hp 走 3 杀直穿 ⇒ 双查询独立出解
    let r = solve_local(&fixture_split());
    assert!(r.reachable, "(iii) 分叉夹具可达");
    assert_eq!(r.max_hp, Some(300), "两路 loss=0 ⇒ hp 同 300");
    let min_kills = r.min_kills.expect("(iii)");
    assert_eq!(min_kills, 2, "全图最少杀数=2（下排墙，x∈{{1,2,3}} 必穿其二）");
    assert_ne!(
        r.route, r.route_min_kills,
        "双查询独立出解：max_hp 走 3 杀直穿、min_kills 走 2 杀绕行（§1.4）"
    );
    assert_eq!(
        bumps(&r.route_min_kills),
        min_kills,
        "(iii) route_min_kills 取等号"
    );
    assert_eq!(
        bumps(&r.route),
        3,
        "route_max_hp=直穿 3 杀（hp 并列 ⇒ 步数少者胜）"
    );
    assert!(
        min_kills <= bumps(&r.route),
        "(iii) route_max_hp 终态∈Exit#1 ⇒ 亦须满足域内下界"
    );
    // 夹具内可枚举的全部 Exit#1 路逐一断言
    for route in [
        vec!["0,1", "bump 41@1,1", "bump 41@2,1", "bump 41@3,1", "4,1"],        // 直穿 3 杀
        vec!["0,1", "0,0", "bump 41@1,0", "2,0", "bump 41@3,0", "4,0", "4,1"],   // 上绕 2 杀
    ] {
        let k = route.iter().filter(|w| w.starts_with("bump ")).count() as i32;
        assert!(
            min_kills <= k,
            "π={route:?} kills={k} ≥ min_kills={min_kills}（(iii) 逐路断言）"
        );
    }

    // 并列 tie-break（§3.3）：字典序最小，而非 BFS 首见
    let r = solve_local(&fixture_tie());
    assert_eq!(r.max_hp, Some(300), "无伤 ⇒ hp(s0)");
    assert_eq!(
        r.route,
        vec!["2,1", "1,1", "1,0"],
        "hp/kills/steps 并列 ⇒ 字典序最小路线（BFS 首见为上路 2,0，断言可判别）"
    );
    assert_eq!(r.min_kills, Some(0), "无怪 ⇒ 0 杀");
    assert_eq!(
        r.route_min_kills, r.route,
        "同并列口径：min_kills 路线亦为字典序最小"
    );
}

// ================= C4（提案 §3.1 T4/T5；§5 C4）=================

/// C4 开门夹具：3×1 走廊，start (0,0) → 黄门 type1@(1,0) → 出口 type7@(2,0)。
/// `yellow` = 入口黄钥数（有钥开 / 无钥挡双例，提案 §3.1 T4）。
fn fixture_door(yellow: i32) -> SolveInput {
    let (base, scaled) = enemy_tables();
    let level = LevelSpec {
        cols: 3,
        rows: 1,
        terrain: vec![true; 3],
        start: (0, 0),
        entities: vec![
            EntSpec {
                ty: 1,
                x: 1,
                y: 0,
                param: 0,
            },
            EntSpec {
                ty: 7,
                x: 2,
                y: 0,
                param: 2,
            },
        ],
        exits: vec![ExitSpec {
            x: 2,
            y: 0,
            kind: ExitKind::Stair,
            index: 1,
        }],
        player: PlayerSpec {
            yellow,
            ..PlayerSpec::from_combat(player(300, 10, 10))
        },
        enemies: scaled,
        enemy_base: base,
        flags: 0,
    };
    SolveInput {
        floor: 1,
        difficulty: 0,
        level: Some(level),
    }
}

#[test]
fn p2s1_t4_开门有钥则开无钥则挡耗钥断言() {
    // 有钥：开门走入、耗一把黄钥（consume_key_for_door 直调进转移，§1.3 开门行）
    let r = solve_local(&fixture_door(1));
    assert!(r.reachable, "有钥 ⇒ 出口可达：{r:?}");
    assert_eq!(
        r.keys_used,
        (1, 0, 0),
        "开黄门耗一把黄钥（T4 钥匙数变化断言；序=(yellow,blue,red)）：{r:?}"
    );
    assert_eq!(
        r.route,
        vec!["0,0", "door 1,0", "2,0"],
        "route_gen door 航点（route_gen.py:14 `door x,y`）：{r:?}"
    );

    // 无钥：阻挡 ⇒ 不可达、零消耗（§1.1 ⊥ 不产生状态）
    let r = solve_local(&fixture_door(0));
    assert!(!r.reachable, "无钥 ⇒ 挡：{r:?}");
    assert!(r.route.is_empty(), "无解 ⇒ 空路线：{r:?}");
    assert_eq!(r.keys_used, (0, 0, 0), "未开门 ⇒ 零消耗：{r:?}");
}

/// C4 fixture 装载路径数据源（census_snapshot.rs 同法：测试侧 I/O，lib 保持纯红线）。
fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

/// 与 census_snapshot.rs 的 `load_all` 同构：assets/raw 55 层 + 七参夹具（测试侧 I/O）。
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

#[test]
fn p2s1_t5_floor1_fixture_census快照() {
    // T5：断言挂 solve_local 的 fixture 装载路径（提案 §3.1 T5：未实现⇒红）。
    // census.rs 既有直调断言属计划 :301/:314 B 类（写时即绿），不得当红。
    let si = load_all_ingest();
    let (base, scaled) = enemy_tables();
    let tile_walk = nums(include_str!(
        "../../game-core/tests/fixtures/tile_walkability.txt"
    ));
    let input = game_solver::solve_local::load_floor(
        &si,
        1,
        (6, 11),
        PlayerSpec::from_combat(player(300, 10, 10)),
        scaled,
        base,
        &tile_walk,
    )
    .expect("floor1 fixture 装载");
    assert_eq!(input.floor, 1, "maplv1 ⇒ floor 字段=1（preset 同号先例）");
    let lv = input.level.as_ref().expect("level 已装载");

    // census 快照（提案 §3.1 T5）
    assert_eq!((lv.cols, lv.rows), (13, 13), "floor1 13×13 真格");
    let cnt = |ty: i32| lv.entities.iter().filter(|e| e.ty == ty).count();
    assert_eq!(cnt(1), 7, "type1 黄门×7");
    assert_eq!(cnt(26), 5, "type26 黄钥×5");
    assert_eq!(cnt(15), 1, "type15 筋斗云×1");
    assert_eq!(cnt(12), 0, "type12=0");
    assert_eq!(cnt(6), 0, "type6=0");
    let e7: Vec<&EntSpec> = lv.entities.iter().filter(|e| e.ty == 7).collect();
    assert_eq!(e7.len(), 1, "type7×1");
    assert_eq!(
        (e7[0].x, e7[0].y, e7[0].param),
        (11, 1, 2),
        "type7@(11,1)→2（提案 §2 出口）"
    );
    assert_eq!(cnt(83), 3, "type83×3");
    let mut p83: Vec<i32> = lv
        .entities
        .iter()
        .filter(|e| e.ty == 83)
        .map(|e| e.param)
        .collect();
    p83.sort_unstable();
    assert_eq!(p83, vec![19, 66, 67], "type83 三分支参数 {{19,66,67}}");

    // 出口派生（type7 ⇒ Exit#1，index=出口序号 1-based）
    assert_eq!(
        lv.exits,
        vec![ExitSpec {
            x: 11,
            y: 1,
            kind: ExitKind::Stair,
            index: 1,
        }],
        "唯一出口 (11,1) Stair"
    );
}

/// T9 专用夹具（提案 §3.1 T9）：13×13 全可走，start=(6,10)，门格 83@param@(6,9) +
/// 楼梯 7@(11,1) param=2（script19 尾 `MOV_0_11_1 MOV_0_1_11` 走上梯格触发
/// `change_floor(2)` ⇒ harness 读 `persist.pending_floor` 作终止边，§2.2 ④）；
/// **无 ExitSpec**——脚本换层边是唯一出口，用于断言终止边入 reachable/max_hp、
/// 不入 min_kills 量化域。`flags`=script_line_flags 位图初值（bit19 重入断言用）。
fn fixture_gate(param: i32, flags: u128, difficulty: i32) -> SolveInput {
    let (base, scaled) = enemy_tables();
    let level = LevelSpec {
        cols: 13,
        rows: 13,
        terrain: vec![true; 13 * 13],
        start: (6, 10),
        entities: vec![
            EntSpec {
                ty: 83,
                x: 6,
                y: 9,
                param,
            },
            EntSpec {
                ty: 7,
                x: 11,
                y: 1,
                param: 2,
            },
        ],
        exits: vec![],
        player: PlayerSpec::from_combat(player(300, 10, 10)),
        enemies: scaled,
        enemy_base: base,
        flags,
    };
    SolveInput {
        floor: 1,
        difficulty,
        level: Some(level),
    }
}

/// T9（提案 §3.1 T9 / 计划 :314）：type83 三分支断言经 solve_local StepGate 转移层
/// + 终止边入 reachable/max_hp、不入 min_kills（§1.4 ②）。
#[test]
fn p2s1_t9_solve_local_StepGate三分支与脚本终止边() {
    // (a) case19：踩格触发（from_walk=true 才跑）+ flags 未置位 ⇒ probe 真；
    //     终止边=唯一出口 ⇒ reachable=true、max_hp=Some、min_kills=None。
    let f19 = fixture_gate(19, 0, 0);
    assert!(
        probe_gate(&f19, 19),
        "case19 踩格必须触发（flags 未置位、from_walk=true）"
    );
    let r = solve_local(&f19);
    assert!(r.reachable, "脚本换层终止边必须入 reachable");
    assert_eq!(r.max_hp, Some(300), "终止边必须入 max_hp");
    assert_eq!(
        r.min_kills, None,
        "终止边不入 min_kills 量化域（§1.4 ② 域=Exit#1）"
    );
    assert_eq!(
        r.route,
        vec!["6,10", "6,9", "gate 6,9"],
        "route=入口→门格→gate 航点收尾"
    );

    // (b) flags 置位后不可重入（try_run_scene:931 gate=!flags[idx]）
    let f19r = fixture_gate(19, 1u128 << 19, 0);
    assert!(!probe_gate(&f19r, 19), "flags[19] 置位后必须不可重入");
    let rr = solve_local(&f19r);
    assert!(!rr.reachable, "门不触发且无出口 ⇒ 无解");
    assert!(rr.route.is_empty(), "无解 ⇒ route 空");

    // (c) case66：恒不触发（try_run_scene:1031 gate=false）
    let f66 = fixture_gate(66, 0, 0);
    assert!(!probe_gate(&f66, 66), "case66 必须恒不触发");
    assert!(!solve_local(&f66).reachable, "66 不触发且无出口 ⇒ 无解");

    // (d) case67：difficulty=0 不触发（try_run_scene:1032 gate &= difficulty_index>0）；
    //     附 difficulty>0 放行（证分支读 difficulty_index 而非恒假）
    let f67 = fixture_gate(67, 0, 0);
    assert!(!probe_gate(&f67, 67), "case67 在 difficulty=0 必须不触发");
    assert!(!solve_local(&f67).reachable, "67@d0 不触发且无出口 ⇒ 无解");
    let f67h = fixture_gate(67, 0, 1);
    assert!(probe_gate(&f67h, 67), "case67 在 difficulty>0 必须触发");
}

// ================= C6（提案 §3.1 T6/T10；计划 :327-328 判据）=================

/// C6 输入：floor1 真层端到端（load_all_ingest → load_floor，与 T5 同装载路径）。
fn fixture_floor1() -> SolveInput {
    let si = load_all_ingest();
    let (base, scaled) = enemy_tables();
    let tile_walk = nums(include_str!(
        "../../game-core/tests/fixtures/tile_walkability.txt"
    ));
    game_solver::solve_local::load_floor(
        &si,
        1,
        (6, 11),
        PlayerSpec::from_combat(player(300, 10, 10)),
        scaled,
        base,
        &tile_walk,
    )
    .expect("floor1 fixture 装载")
}

/// route 元素提取（T1 同式：route=末字段字符串数组）。
fn route_elems(json: &str) -> Vec<String> {
    let ri = json
        .find("\"route\":[")
        .unwrap_or_else(|| panic!("缺 route：{json}"));
    let rest = &json[ri + "\"route\":".len()..];
    assert!(rest.ends_with('}'), "route 是末字段、其后为右括号：{json}");
    let arr = &rest[..rest.len() - 1];
    assert!(arr.starts_with('[') && arr.ends_with(']'), "route 为数组：{json}");
    let inner = &arr[1..arr.len() - 1];
    if inner.is_empty() {
        return Vec::new();
    }
    assert!(inner.starts_with('"') && inner.ends_with('"'), "route 元素为字符串：{json}");
    inner
        .split("\",\"")
        .map(|s| s.trim_matches('"').to_string())
        .collect()
}

/// 航点过 `route_gen.py:12-15` 语法：`x,y` 普通格 / `bump 型@x,y` / `door x,y`。
/// 批2 追加 `gate x,y`=脚本换层终止边标记——**route_gen 三类语法无此 token**
/// （提案 C6 判据「route 逐航点过 route_gen 语法」的歧义面，报审注明；该航点
/// 不产生移动、位置已由前一航点覆盖，喂 route_gen 时剥除即可复现）。
fn waypoint_ok(wp: &str) -> bool {
    let cell_ok = |s: &str| {
        let parts: Vec<&str> = s.split(',').collect();
        parts.len() == 2 && parts.iter().all(|p| p.parse::<i32>().is_ok())
    };
    if wp.starts_with("gate ") {
        return true;
    }
    if let Some(r) = wp.strip_prefix("bump ") {
        let Some((ty, cell)) = r.split_once('@') else {
            return false;
        };
        return !ty.trim().is_empty() && cell_ok(cell);
    }
    if let Some(r) = wp.strip_prefix("door ") {
        return cell_ok(r);
    }
    cell_ok(wp)
}

/// T6（提案 §3.1 T6 / 计划 :327-328）：floor1 端到端 JSON 形状 + min_kills≥3 下界。
#[test]
fn p2s1_t6_solve_local_floor1端到端json形状与min_kills下界() {
    let r = solve_local(&fixture_floor1());
    let j = r.to_json();

    // 逐字段类型断言（提案 :151 原文形状，防空过）
    assert_eq!(scalar(&j, "floor"), "1", "floor=1：{j}");
    assert_eq!(scalar(&j, "difficulty"), "0", "difficulty=int：{j}");
    assert_eq!(scalar(&j, "reachable"), "true", "floor1 必须有解：{j}");
    let mh = scalar(&j, "max_hp");
    assert!(mh.parse::<i32>().is_ok(), "max_hp 必须为 int：{j}");
    let mk = scalar(&j, "min_kills");
    // §8-8 首跑核对（2026-11-11 实测裁定）：Exit#1 步行不可达 ⇒ min_kills=None——
    // 口袋唯一出口=door@(6,9)，踩格必触发 script19 换层边（flags[19] 新局 false、
    // case0 在 floor1 无触发源），Exit#1 无合法路；reachable/max_hp 由 Exit#2
    // （gate19 终止边）给出，route=Path B（kills=0 不入 min_kills 域，§1.4 ②）。
    // 提案 §8-8 verbatim：「若 Exit#1 不可达则 min_kills=None 而 reachable=true、
    // max_hp=Some（由 Exit#2 路给出）；T6 形状断言 min_kills:<int> 此情形 None
    // 变体首跑核对（T6 真层输出核对）」⇒ 本断言按 None 变体钉。
    assert_eq!(mk, "null", "min_kills=None（§8-8 None 变体）：{j}");
    assert_eq!(
        scalar(&j, "confidence"),
        "\"bounded-best\"",
        "confidence=bounded-best（BR-7 / §64 单层版不冒充 optimal）：{j}"
    );

    // route 非空字符串数组 + 逐航点过 route_gen.py 语法（:12-15，C6 判据）
    let elems = route_elems(&j);
    assert!(!elems.is_empty(), "floor1 route 非空：{j}");
    for wp in &elems {
        assert!(waypoint_ok(wp), "航点过 route_gen 语法：{wp}（{j}）");
    }

    // min_kills ≥ 3 断言已按 §8-8 None 变体改判（见上方 :691 注释块，2026-11-11）
}

/// T10（提案 §3.1 T10 / A5 形状）：同输入双跑 route/max_hp/min_kills 逐字节相等。
/// 附非平凡闸：双跑须为有解输出（空 route 双跑恒等=确定性空过，不算证据）。
#[test]
fn p2s1_t10_solve_local_同输入双跑逐字节相等() {
    let input = fixture_floor1();
    let a = solve_local(&input).to_json();
    let b = solve_local(&input).to_json();
    assert_eq!(
        a, b,
        "同输入双跑 route/max_hp/min_kills 逐字节相等（§3.3 / T10）"
    );
    assert!(
        !route_elems(&a).is_empty(),
        "双跑输出须非平凡（route 非空）：{a}"
    );
}
