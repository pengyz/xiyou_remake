//! 【求解器批0】census 快照测试（TDD 红先行）——SolverInput 摘要与计划 §2 census 数字对拍。
//!
//! 判据（计划 :283-286）：floors=55 enemies=502 scripts=68 enemies_table=35 items=32 shops=17。
//! 数据源=assets/raw/maplv0..54+sprite0..54（§2.1/§2.2 实测）+fixtures+data/tables 三表。
use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn load_all() -> game_solver::ingest::SolverInput {
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
    .expect("ingest 失败")
}

#[test]
fn p2s0_census_snapshot_matches_plan() {
    let c = game_solver::ingest::census(&load_all());
    assert_eq!(
        (c.floors, c.enemies, c.scripts, c.enemies_table, c.items, c.shops),
        (55, 502, 68, 35, 32, 17),
        "census 快照 vs 计划 §2（floors/enemies/scripts/enemies_table/items/shops）"
    );
}

#[test]
fn p2s0_floor_geometry_sizes_present() {
    let inp = load_all();
    // 55 层尺寸/实体数（快照判据面）：每层解析出宽高>0 且记录计数=该层 sprite 记录数。
    for (i, f) in inp.floors.iter().enumerate() {
        assert!(f.width > 0 && f.height > 0, "maplv{i} 尺寸");
    }
    let total_records: usize = inp.per_floor_records.iter().sum();
    assert_eq!(total_records, 1703, "sprite 总记录 1703（§2.2 census）");
}

#[test]
fn p2s0_s0_and_q3_constants() {
    assert_eq!(game_solver::state::s0(), (300, 10, 10, 51));
    assert_eq!(game_solver::state::Q3_EQUIP_SET, &[36, 40], "Q3 裁定项现值=A 候选");
    // 批3 F4（用户 2026-11-12 裁 F4-A）：QoL 全收集口径=可获得子集
    assert_eq!(
        game_solver::state::QOL_ITEMS,
        &[15, 21, 23],
        "F4-A：20/22/24 世界零获取不入全收集前提"
    );
}

#[test]
fn p2s0_lib_pure_no_io() {
    // 只断言**代码面**（注释/文档可引用禁用名——剥离 `//` 注释后扫描）。
    for src in [
        include_str!("../src/lib.rs"),
        include_str!("../src/ingest.rs"),
        include_str!("../src/state.rs"),
    ] {
        for line in src.lines() {
            let code = line.split("//").next().unwrap_or("");
            for tok in ["std::fs", "std::net", "std::time", "std::env"] {
                assert!(!code.contains(tok), "game-solver lib 禁 I/O 依赖（代码面）：{tok} @ {line}");
            }
        }
    }
}
