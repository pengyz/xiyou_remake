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
    probe_bump, solve_local, EntSpec, ExitKind, ExitSpec, LevelSpec, PlayerSpec, SolveInput,
};

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
