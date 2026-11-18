//! 【P3 复核红2 判据】`ok` 与 `truncated` 正交性 + `LAYER_MEMO` 键含层界。
//!
//! 为什么单独一个测试二进制：`SOLVER_LAYER_LIMIT` 是**进程全局**环境变量，而
//! `cargo test` 默认在同一二进制内多线程跑测试 ⇒ 若把它塞进 `solve_local.rs`，
//! `SOLVER_LAYER_LIMIT=10` 会污染并行跑的 f1 满界测试（`!truncated` 断言随机红）。
//! 独立二进制=独立进程 ⇒ 环境变量不外泄；本文件内两个测试再用互斥锁串行。
//!
//! 判据（P3 红2 两条 + P3 收口两条）：
//! 1. `exit_solve_truncation_forces_ok_false`：真数据 f1 + `SOLVER_LAYER_LIMIT=10`
//!    ⇒ `truncated=true` 且 `ok=false`（红2-A 契约：截断⇒不可信⇒ok=false）；
//! 2. `layer_memo_key_includes_node_limit`：**同线程**先 `limit=10`（截断）再默认界
//!    ⇒ 第二次必须 `truncated=false && ok=true`（旧键缺 node_limit 时会回放截断结果）；
//! 3. `legacy_solve_local_reports_truncation`（复核 R2 黄收口）：**旧入口**
//!    `solve_local()` 也必须有截断信号——`SolveResult.truncated`，且截断时
//!    `confidence` 降级为 `capped-search`（旧行为=恒 `bounded-best`，截断被伪装成完整解）；
//! 4. `layer_memo_key_includes_shops`（复核 R3 黄收口）：`LAYER_MEMO` 键必须含**商品表内容**
//!    ——同线程先真表（有买臂）后空表，解集必须不同（旧键缺 shops ⇒ 空表回放带买臂的解）。
use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_data::defs::EnemyDef;
use game_data::defs::{ShopEffect, ShopGoodDef};
use game_solver::solve_local::{
    solve_layer_with, solve_local, solve_local_exit, ClosurePolicy, EntSpec, ExitKind, ExitSpec,
    LayerGoal, LevelSpec, PlayerSpec, SolveInput,
};
use std::path::PathBuf;

/// 进程内串行锁：两个测试都要改进程全局 env，必须互斥（Cargo 默认并行跑本文件内测试）。
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    // 前一个测试若在持锁时 panic，锁会中毒；这里取回内部值，避免把「上一条红」
    // 伪装成「这条也红」（判据要看到真正的断言差异）。
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// 作用域式 `SOLVER_LAYER_LIMIT` 设置：Drop 恢复原值（不污染同进程后续测试）。
struct LayerLimitEnv(Option<String>);

impl LayerLimitEnv {
    fn set(v: usize) -> Self {
        let prev = std::env::var("SOLVER_LAYER_LIMIT").ok();
        std::env::set_var("SOLVER_LAYER_LIMIT", v.to_string());
        LayerLimitEnv(prev)
    }
}

impl Drop for LayerLimitEnv {
    fn drop(&mut self) {
        match &self.0 {
            Some(v) => std::env::set_var("SOLVER_LAYER_LIMIT", v),
            None => std::env::remove_var("SOLVER_LAYER_LIMIT"),
        }
    }
}

fn nums(text: &str) -> Vec<i32> {
    text.lines()
        .map(|l| l.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().expect("夹具整数"))
        .collect()
}

/// 敌表与 `tests/solve_local.rs::enemy_tables` 同式（与引擎侧同源，BR-2）。
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

/// 真数据 f1（与 `tests/solve_local.rs::fixture_floor1` 同一装载路径）。
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

/// 【红2-A 判据】真数据 f1 在 `SOLVER_LAYER_LIMIT=10` 下**必被截断**，且截断 ⇒ `ok=false`。
///
/// 旧契约（复核者实测）：`ok=true && truncated=true && out_hp=202` ⇒ `ok` 不蕴含「搜索完整」，
/// P4 会把被截断的 202 当出层最大血。新契约（方向 A）：`truncated=true ⇒ ok=false`，
/// `out_*` 降级为「部分搜索里最优的一条，仅参考」。
///
/// 变异自证：把 `ok` 改回 `reached && !truncated` 的老式 `ok: true`（或删掉 `!truncated`）
/// ⇒ `got.ok` 为 true ⇒ 本测试红在「必须 !ok」。
#[test]
fn exit_solve_truncation_forces_ok_false() {
    let _guard = env_lock();
    let input = fixture_floor1();
    let level = input.level.clone().expect("floor1 已装载");
    let entry = level.start;

    let _env = LayerLimitEnv::set(10); // 只本次作用域生效；默认值 200_000 不动
    let got = solve_local_exit(&input, entry, &level.player, level.flags, None);

    println!("[红2] f1 limit=10 => {got:?}");
    assert!(
        got.truncated,
        "limit=10 必须撞层界（f1 单层搜索需要远多于 10 个节点）：{got:?}"
    );
    assert!(
        !got.ok,
        "截断 ⇒ 本次结果不可信 ⇒ ok 必须 false（红2-A 契约）：{got:?}"
    );
    // out_* 仍是「部分搜索里最优的一条」——保留可诊断性，但不许当出层态消费。
    assert_eq!(
        got.out_hp, 202,
        "参考值：截断解的最优血量（与满界解同为 202 是巧合，不是契约）：{got:?}"
    );
    assert_eq!(got.exit, Some((11, 1)), "参考：截断解也找到出层落点：{got:?}");
}

/// 【红2 附带判据】`LAYER_MEMO` 键必须含 `layer_node_limit()`：
/// **同线程**先小界（10，截断）再默认界 ⇒ 第二次必须重新求解、`truncated=false`。
///
/// 变异自证：把 memo 键里的 `node_limit` 去掉（恢复旧键）⇒ 第二次命中第一次的缓存、
/// 回放 `hit_trunc=Some(..)` ⇒ `truncated=true`（且新契约下 `ok=false`）⇒ 本测试红。
#[test]
fn layer_memo_key_includes_node_limit() {
    let _guard = env_lock();
    let input = fixture_floor1();
    let level = input.level.clone().expect("floor1 已装载");
    let entry = level.start;

    let first = {
        let _env = LayerLimitEnv::set(10);
        solve_local_exit(&input, entry, &level.player, level.flags, None)
    };
    assert!(first.truncated, "第一次（limit=10）必须截断：{first:?}");
    assert!(!first.ok, "截断 ⇒ !ok：{first:?}");

    // 同一线程、同一层/入场态，只有层界回默认——必须冷算，不得回放截断结果。
    let second = solve_local_exit(&input, entry, &level.player, level.flags, None);
    println!("[红2-memo] second(default) => {second:?}");
    assert!(
        !second.truncated,
        "默认界下必须重算且不截断（键含 node_limit ⇒ 不命中 limit=10 的缓存）：{second:?}"
    );
    assert!(second.ok, "满界解可信 ⇒ ok=true：{second:?}");
    assert_eq!(second.out_hp, 202, "满界出层最大血：{second:?}");
    assert_eq!(second.exit, Some((11, 1)), "出层落点：{second:?}");
}

// ================================================================ P3 收口（复核 R2/R3 黄）

/// 收口1 判据（复核 R2 黄）：旧入口 `solve_local()` 必须自带截断信号。
///
/// 旧行为（复核者实测 `SOLVER_LAYER_LIMIT=10/200/1e6` 三种输出逐字段相同）：
/// 截断解与完整解在 `SolveResult` 上不可区分 ⇒ 「截断被当完整解」的洞只在
/// `ExitSolve` 层被堵。新契约（方向 A）：`truncated` 与 `ExitSolve.truncated`
/// 同源（同一个 `take_layer_trunc()`），且 `confidence` 在截断时降级为
/// `capped-search`（`solve_global::RunStatus::Capped` 同词）。
///
/// 变异自证：把 `solve_local` 里的 `truncated` 写死 `false`（或删掉入口清
/// `LAYER_TRUNC` 那一行、让它读到上一条测试残留的截断标记）⇒ 本测试红。
#[test]
fn legacy_solve_local_reports_truncation() {
    let _guard = env_lock();
    let input = fixture_floor1();

    // (a) 满界：不截断 ⇒ truncated=false 且置信仍是 bounded-best
    let full = solve_local(&input);
    let jfull = full.to_json();
    println!("[收口1 full] {jfull}");
    assert!(full.reachable, "f1 满界有解：{jfull}");
    assert!(!full.truncated, "满界（默认 200_000 节点）不应截断：{jfull}");
    assert!(jfull.contains("\"truncated\":false"), "JSON 必须带 truncated=false：{jfull}");
    assert_eq!(full.confidence, "bounded-best", "满界单层置信：{jfull}");

    // (b) 小界：必截断 ⇒ truncated=true 且置信降级
    let small = {
        let _env = LayerLimitEnv::set(10);
        solve_local(&input)
    };
    let jsmall = small.to_json();
    println!("[收口1 limit=10] {jsmall}");
    assert!(
        small.truncated,
        "limit=10 必撞层界 ⇒ truncated=true（旧入口必须能报截断）：{jsmall}"
    );
    assert!(
        jsmall.contains("\"truncated\":true"),
        "JSON 必须带 truncated=true：{jsmall}"
    );
    assert_eq!(
        small.confidence, "capped-search",
        "截断 ⇒ 置信降级 capped-search（不许冒充 bounded-best）：{jsmall}"
    );
    // route 仍是 JSON 末字段（T1 形状判据不许被本次字段新增破坏）
    let ri = jsmall.find("\"route\":[").expect("缺 route");
    assert!(jsmall[ri..].ends_with('}'), "route 必须仍是末字段：{jsmall}");
}

/// 收口2 夹具：8×3 单走廊（仅 y=1 可走）+ 三格 ty78 商人 + 上行梯 ty7(x=7,param=50)。
/// 与 `tests/solve_global.rs::gold_world` 同款（复核 R3 探针同夹具）。
fn shop_world() -> SolveInput {
    const W: i32 = 8;
    let mut terrain = vec![false; (W * 3) as usize];
    for x in 0..W {
        terrain[(W + x) as usize] = true; // y=1
    }
    let mut entities: Vec<EntSpec> = Vec::new();
    for x in 4..=6 {
        // ty78 ⇒ src=1；开价门 (param>>8)&2 ⇒ 3；idx=4 ⇒ 真表「¤50 黄+5」那一行
        entities.push(EntSpec { ty: 78, x, y: 1, param: (3 << 8) | 4 });
    }
    entities.push(EntSpec { ty: 7, x: 7, y: 1, param: 50 });
    SolveInput {
        floor: 1,
        difficulty: 0,
        level: Some(LevelSpec {
            cols: W,
            rows: 3,
            terrain,
            start: (0, 1),
            entities,
            exits: vec![ExitSpec { x: 7, y: 1, kind: ExitKind::Stair, index: 1 }],
            player: PlayerSpec {
                hp: 300,
                atk: 10,
                def: 10,
                yellow: 0,
                blue: 0,
                red: 0,
                gold: 100,
                item_types: Vec::new(),
                item_uses: Vec::new(),
                item_stack_size: 0,
            },
            enemies: Vec::new(),
            enemy_base: Vec::new(),
            flags: 0,
        }),
    }
}

/// 真表那一行（`data/tables/shop_goods.json` id8 语义：¤50 黄+5），手工构造以免本
/// 测试二进制再拖 JSON 解析；内容必须与 memokey 的语义字段同面。
fn one_good() -> Vec<ShopGoodDef> {
    vec![ShopGoodDef {
        id: 1,
        src: 1,
        idx: 4,
        price: 50,
        effect: ShopEffect::AddKeys { blue: 0, yellow: 5, red: 0 },
        fail_hint: String::new(),
    }]
}

/// 解集指纹（跨两次调用可比；`LayerGoal` 无 PartialEq）。
fn goals_sig(goals: &[LayerGoal]) -> Vec<(i32, i32, i32, (i32, i32, i32), String, Vec<String>, i32, i32, i32)> {
    goals
        .iter()
        .map(|g| {
            (
                g.hp,
                g.kills,
                g.steps,
                g.keys,
                format!("{:?}", g.kind),
                g.route.clone(),
                g.post.gold,
                g.post.yellow,
                g.post.hp,
            )
        })
        .collect()
}

/// 收口2 判据（复核 R3 黄）：`LAYER_MEMO` 键必须含**商品表内容**。
///
/// 复核 A 级探针（旧键缺 shops）：同线程先真表后空表，空表仍回放带买臂的解
/// （`identical_goals=true`，hp=300 / gold=50 / yellow=5 / steps=7）。
///
/// 变异自证：把 `shops`（商品表规范编码）从 memo 键里拿掉 ⇒ 第二次命中第一次的
/// 缓存、解集逐字段相同 ⇒ `assert_ne!` 红。
#[test]
fn layer_memo_key_includes_shops() {
    let _guard = env_lock();
    let input = shop_world();
    let level = input.level.clone().expect("夹具已装载");
    let entry = level.start;
    let player = level.player.clone();

    // 第一次：真商品表（买得起 ⇒ 解集里应有「已买」终态）
    let (ok1, g1) = solve_layer_with(
        &input,
        entry,
        None,
        player.clone(),
        0,
        ClosurePolicy::OFF,
        &one_good(),
    );
    assert!(ok1, "真表：走廊必达上行梯");
    let sig1 = goals_sig(&g1);
    println!("[收口2 real] {} goals", sig1.len());
    for s in &sig1 {
        println!("  {s:?}");
    }
    assert!(
        g1.iter().any(|g| g.post.yellow == 5 && g.post.gold == 50),
        "真表必须给出买臂终态（黄+5 / 金 100-50）：{sig1:?}"
    );

    // 第二次：同一层/同一入场态，仅商品表为空 ⇒ 必须冷算（空表买不到东西）
    let (ok2, g2) = solve_layer_with(
        &input,
        entry,
        None,
        player.clone(),
        0,
        ClosurePolicy::OFF,
        &[],
    );
    assert!(ok2, "空表：走廊仍必达上行梯");
    let sig2 = goals_sig(&g2);
    println!("[收口2 empty] {} goals", sig2.len());
    for s in &sig2 {
        println!("  {s:?}");
    }
    assert_ne!(
        sig1, sig2,
        "空商品表必须与真表给出不同解集（旧键缺 shops ⇒ 回放真表解）：\nreal={sig1:?}\nempty={sig2:?}"
    );
    assert!(
        !g2.iter().any(|g| g.post.yellow != 0 || g.post.gold != 100),
        "空表不许出现任何购买痕迹（黄钥=0 / 金=100）：{sig2:?}"
    );
}
