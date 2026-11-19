//! hp 上界计划 68 槽扫描（证据工具，**不参与门禁**）。
//!
//! 逐层建 `hp_bound::plan`（层入口=整层静态实体全活，S0 300/10/10），打印
//! `reason`/`reason_ty`/槽数/拾取类型数/根态上界。用途：
//! 1. 「闸门到底咬不咬当前语料」——`post_kill_hp_spawn`/`script_hp_spawn`
//!    各咬几层（复核 F2 的修复面）；
//! 2. 回归对照：修复前后 `reason=on` 的层集合（剪枝收益面）。
//!
//! 用法：`cargo run --release -p game-solver --example hp_bound_sweep`
//! 输出每层一行，末尾一行汇总。

use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_solver::hp_bound::{plan, HpBoundPlan};
use game_solver::ingest::{ingest, SolverInput};
use game_solver::solve_local::{load_floor, EntRec, PlayerSpec};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn nums(t: &str) -> Vec<i32> {
    t.lines()
        .map(|l| l.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect()
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();
    let mut maplvs = Vec::new();
    let mut sprites = Vec::new();
    for i in 0..55 {
        maplvs.push(std::fs::read(root.join(format!("assets/raw/maplv{i}"))).unwrap());
        sprites.push(std::fs::read(root.join(format!("assets/raw/sprite{i}"))).unwrap());
    }
    let maplv_refs: Vec<&[u8]> = maplvs.iter().map(|v| v.as_slice()).collect();
    let sprite_refs: Vec<&[u8]> = sprites.iter().map(|v| v.as_slice()).collect();
    let si: SolverInput = ingest(
        &maplv_refs,
        &sprite_refs,
        include_str!("../../game-core/tests/fixtures/level_scripts.txt"),
        include_str!("../../game-core/tests/fixtures/door12.txt"),
        include_str!("../../game-core/tests/fixtures/door13.txt"),
        include_str!("../../../data/tables/enemies.json"),
        include_str!("../../../data/tables/items.json"),
        include_str!("../../../data/tables/shop_goods.json"),
    )
    .expect("ingest");
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
    let tile_walk = nums(include_str!("../../game-core/tests/fixtures/tile_walkability.txt"));
    game_solver::hp_bound::set_enabled(true);
    let mut by_reason: BTreeMap<&'static str, usize> = BTreeMap::new();
    for f in 1..=54usize {
        let pc = PlayerCombat {
            hp: 300,
            atk: 10,
            def: 10,
            yellow_keys: 0,
            blue_keys: 0,
            red_keys: 0,
            gold: 0,
            item_types: vec![],
            item_uses: vec![],
            item_stack_size: 0,
        };
        let inp = match load_floor(
            &si,
            f,
            (1, 11),
            PlayerSpec::from_combat(pc),
            scaled.clone(),
            base.clone(),
            &tile_walk,
        ) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let Some(lv) = inp.level.as_ref() else { continue };
        let pc = PlayerCombat {
            hp: 300,
            atk: 10,
            def: 10,
            yellow_keys: 0,
            blue_keys: 0,
            red_keys: 0,
            gold: 0,
            item_types: vec![],
            item_uses: vec![],
            item_stack_size: 0,
        };
        // 层入口面：整层静态实体全活（不模拟已拾取/已击杀）。
        let ents: Vec<EntRec> = lv
            .entities
            .iter()
            .map(|e| EntRec {
                ty: e.ty,
                x: e.x,
                y: e.y,
                param: e.param,
            })
            .collect();
        let p = plan(f as i32, 0, lv, &ents, &si.shops);
        let (reason, ty) = p.reason();
        *by_reason.entry(reason).or_default() += 1;
        match &p {
            HpBoundPlan::On(o) => {
                let (slots, pickups) = (
                    o.slot_count(),
                    o.pickup_type_count(),
                );
                println!(
                    "floor={} reason={} ty={} slots={} pickup_types={} root_bound={}",
                    f,
                    reason,
                    ty,
                    slots,
                    pickups,
                    o.bound(300, &ents, 0)
                );
            }
            HpBoundPlan::Off(_, _) => {
                println!("floor={} reason={} ty={}", f, reason, ty);
            }
        }
    }
    println!("summary {:?}", by_reason);
}
