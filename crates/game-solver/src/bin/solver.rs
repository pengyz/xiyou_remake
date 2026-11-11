//! 【求解器批0/批2】solver CLI：`ingest --summary`（批0 census）+ `solve-local --floor N`（批2 单层求解，C1 起；C6 起 floor1 接真数据）。
//!
//! I/O 只在此 bin（lib 纯度边界见 src/lib.rs；A9 纯度闸只查 src/*.rs 排除本文件）。
use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_solver::ingest::{census, ingest};
use game_solver::solve_local::{load_floor, solve_local, PlayerSpec, SolveInput};
use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(|s| s.as_str()) == Some("solve-local") {
        // 批2 C1：单层求解子命令（逻辑在 lib::solve_local，壳只在此处解析参数+打印）。
        // C6：floor1 接真数据（I/O 归此 bin=A9 豁免面）；其余层保持占位（T1 恒绿面）。
        let floor = args
            .iter()
            .position(|a| a == "--floor")
            .and_then(|i| args.get(i + 1))
            .map(|v| v.parse::<i32>().expect("--floor 需整数"))
            .unwrap_or_else(|| {
                eprintln!("用法：solver solve-local --floor <N>");
                std::process::exit(2);
            });
        if floor == 1 {
            println!("{}", solve_floor1().to_json());
        } else {
            println!("{}", solve_local(&SolveInput::placeholder(floor)).to_json());
        }
        return;
    }
    if args.first().map(|s| s.as_str()) != Some("ingest") || args.get(1).map(|s| s.as_str()) != Some("--summary") {
        eprintln!("用法：solver ingest --summary | solver solve-local --floor <N>");
        std::process::exit(2);
    }
    let root = repo();
    let mut maplvs: Vec<Vec<u8>> = Vec::new();
    let mut sprites: Vec<Vec<u8>> = Vec::new();
    for i in 0..55 {
        maplvs.push(std::fs::read(root.join(format!("assets/raw/maplv{i}"))).expect("maplv"));
        sprites.push(std::fs::read(root.join(format!("assets/raw/sprite{i}"))).expect("sprite"));
    }
    let maplv_refs: Vec<&[u8]> = maplvs.iter().map(|v| v.as_slice()).collect();
    let sprite_refs: Vec<&[u8]> = sprites.iter().map(|v| v.as_slice()).collect();
    let inp = ingest(
        &maplv_refs,
        &sprite_refs,
        include_str!("../../../game-core/tests/fixtures/level_scripts.txt"),
        include_str!("../../../game-core/tests/fixtures/door12.txt"),
        include_str!("../../../game-core/tests/fixtures/door13.txt"),
        include_str!("../../../../data/tables/enemies.json"),
        include_str!("../../../../data/tables/items.json"),
        include_str!("../../../../data/tables/shop_goods.json"),
    )
    .expect("ingest 失败");
    let c = census(&inp);
    println!(
        "floors={} enemies={} scripts={} enemies_table={} items={} shops={}",
        c.floors, c.enemies, c.scripts, c.enemies_table, c.items, c.shops
    );
    let sizes: Vec<String> = inp
        .floors
        .iter()
        .zip(&inp.per_floor_records)
        .enumerate()
        .map(|(i, (f, n))| format!("f{i}:{}x{}/{}", f.width, f.height, n))
        .collect();
    println!("floorsizes {}", sizes.join(" "));
}

/// 行整数解析（tile_walkability / 敌表 fixture；tests 助手 `nums` 同构）。
fn nums(text: &str) -> Vec<i32> {
    text.lines()
        .map(|l| l.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().expect("fixture 整数"))
        .collect()
}

/// C6：floor1 端到端（计划 :327 判据命令 `solve-local --floor 1`）。
/// 装载路径与测试 `load_floor` 同构（ingest→load_floor），入口 (6,11)、difficulty=0、
/// 玩家 s0=(300,10,10,0,0,0,0)（提案 §4.1 s₀ 注证据；新局三色钥匙=0）。
fn solve_floor1() -> game_solver::solve_local::SolveResult {
    let root = repo();
    let mut maplvs: Vec<Vec<u8>> = Vec::new();
    let mut sprites: Vec<Vec<u8>> = Vec::new();
    for i in 0..55 {
        maplvs.push(std::fs::read(root.join(format!("assets/raw/maplv{i}"))).expect("maplv"));
        sprites.push(std::fs::read(root.join(format!("assets/raw/sprite{i}"))).expect("sprite"));
    }
    let maplv_refs: Vec<&[u8]> = maplvs.iter().map(|v| v.as_slice()).collect();
    let sprite_refs: Vec<&[u8]> = sprites.iter().map(|v| v.as_slice()).collect();
    let si = ingest(
        &maplv_refs,
        &sprite_refs,
        include_str!("../../../game-core/tests/fixtures/level_scripts.txt"),
        include_str!("../../../game-core/tests/fixtures/door12.txt"),
        include_str!("../../../game-core/tests/fixtures/door13.txt"),
        include_str!("../../../../data/tables/enemies.json"),
        include_str!("../../../../data/tables/items.json"),
        include_str!("../../../../data/tables/shop_goods.json"),
    )
    .expect("ingest 失败");
    // 敌表=game-core fixture 同源（tests `enemy_tables` 同构；BR-2 双侧输入逐位相同）
    let tables = CombatTables {
        enemy_base_hp: nums(include_str!("../../../game-core/tests/fixtures/enemybasehp.txt")),
        enemy_base_atk: nums(include_str!("../../../game-core/tests/fixtures/enemybaseatk.txt")),
        enemy_base_def: nums(include_str!("../../../game-core/tests/fixtures/enemybasedef.txt")),
        difficulty_multipliers: nums(include_str!(
            "../../../game-core/tests/fixtures/difficulty_multipliers.txt"
        )),
    };
    let gold = nums(include_str!("../../../game-core/tests/fixtures/enemybasegold.txt"));
    let base = tables.to_records(&gold);
    let scaled = scale_enemy_stats(&base, 1); // difficulty=0 ⇒ ×1（BR-4）
    let tile_walk = nums(include_str!(
        "../../../game-core/tests/fixtures/tile_walkability.txt"
    ));
    let player = PlayerCombat {
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
    let input = load_floor(
        &si,
        1,
        (6, 11),
        PlayerSpec::from_combat(player),
        scaled,
        base,
        &tile_walk,
    )
    .expect("floor1 fixture 装载");
    solve_local(&input)
}
