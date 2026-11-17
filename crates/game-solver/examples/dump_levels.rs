//! 关卡数据速览（调试工具，**不参与门禁**）：逐层打印商店（77/78，含 param 拆解）、
//! 关键拾取（13..=40 / 79..=86）与怪物赏金合计。
//!
//! 用法：`cargo run -q -p game-solver --example dump_levels`
//!
//! 用途：回答「某条资源边到底存不存在、在几层、开价位是什么」这类问题，避免把
//! 「模型没建这条边」误读成「世界里没有这条边」（H4 的教训）。

use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_solver::ingest::{ingest, SolverInput};
use game_solver::solve_local::{load_floor, PlayerSpec};
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
    let mk_pc = || PlayerCombat {
        hp: 300,
        atk: 5,
        def: 5,
        yellow_keys: 0,
        blue_keys: 0,
        red_keys: 0,
        gold: 0,
        item_types: vec![],
        item_uses: vec![],
        item_stack_size: 0,
    };
    let mut total_bounty = 0;
    for f in 1..=54usize {
        let inp = match load_floor(
            &si,
            f,
            (1, 11),
            PlayerSpec::from_combat(mk_pc()),
            scaled.clone(),
            base.clone(),
            &tile_walk,
        ) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let lv = match inp.level.as_ref() {
            Some(l) => l,
            None => continue,
        };
        for e in lv.entities.iter().filter(|e| e.ty == 77 || e.ty == 78) {
            let flags = (e.param >> 8) as i32;
            let idx = e.param & 0xff;
            println!(
                "floor={} SHOP ty={} pos=({},{}) param={} flags={} (open={} once={} repeat={}) idx={}",
                f,
                e.ty,
                e.x,
                e.y,
                e.param,
                flags,
                flags & 2 != 0,
                flags & 1 != 0,
                flags & 4 != 0,
                idx
            );
        }
        for e in lv.entities.iter() {
            if (13..=40).contains(&e.ty) || (79..=86).contains(&e.ty) {
                println!("floor={} PICKUP ty={} pos=({},{})", f, e.ty, e.x, e.y);
            }
        }
        if f == 16 {
            for e in lv.entities.iter().filter(|e| matches!(e.ty, 9..=12)) {
                println!("floor=16 WALL-ish ty={} pos=({},{}) param={}", e.ty, e.x, e.y, e.param);
            }
            for e in lv.entities.iter().filter(|e| matches!(e.ty, 7 | 8)) {
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let (nx, ny) = (e.x + dx, e.y + dy);
                    if nx < 0 || ny < 0 || nx >= lv.cols || ny >= lv.rows { continue; }
                    let w = lv.terrain[(ny * lv.cols + nx) as usize];
                    println!("floor=16 NBR exit=({},{}) nbr=({},{}) walkable={}", e.x, e.y, nx, ny, w);
                }
            }
        }
        {
            let (sx, sy) = lv.start;
            let w = lv.terrain[(sy * lv.cols + sx) as usize];
            println!("floor={} START pos=({},{}) terrain_walkable={}", f, sx, sy, w);
        }
        for e in lv.entities.iter().filter(|e| matches!(e.ty, 7 | 8)) {
            let w = lv.terrain[(e.y * lv.cols + e.x) as usize];
            println!("floor={} EXIT ty={} pos=({},{}) terrain_walkable={}", f, e.ty, e.x, e.y, w);
        }
        for e in lv.entities.iter().filter(|e| (1..=3).contains(&e.ty)) {
            let color = match e.ty { 1 => "黄", 2 => "红", 3 => "蓝", _ => "?" };
            println!("floor={} DOOR {} ty={} pos=({},{}) param={}", f, color, e.ty, e.x, e.y, e.param);
        }
        let mut n = 0;
        let mut g = 0;
        let mut ids: Vec<i32> = vec![];
        for e in lv.entities.iter() {
            if (41..67).contains(&e.ty) {
                n += 1;
                ids.push(e.ty);
                g += base.get((e.ty - 41) as usize).map(|d| d.gold).unwrap_or(0);
            }
        }
        total_bounty += g;
        println!(
            "floor={} MONSTERS n={} bounty_sum={} cumulative={} ids={:?}",
            f,
            n,
            g,
            total_bounty,
            &ids[..ids.len().min(12)]
        );
    }
}
