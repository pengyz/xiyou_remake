//! 【求解器批0/批2】solver CLI：`ingest --summary`（批0 census）+ `solve-local --floor N`（批2 单层求解，C1 起）。
//!
//! I/O 只在此 bin（lib 纯度边界见 src/lib.rs）。
use game_solver::ingest::{census, ingest};
use game_solver::solve_local::{solve_local, SolveInput};
use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(|s| s.as_str()) == Some("solve-local") {
        // 批2 C1：单层求解子命令（逻辑在 lib::solve_local，壳只在此处解析参数+打印）
        let floor = args
            .iter()
            .position(|a| a == "--floor")
            .and_then(|i| args.get(i + 1))
            .map(|v| v.parse::<i32>().expect("--floor 需整数"))
            .unwrap_or_else(|| {
                eprintln!("用法：solver solve-local --floor <N>");
                std::process::exit(2);
            });
        println!("{}", solve_local(&SolveInput::placeholder(floor)).to_json());
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
