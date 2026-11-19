//! 【P9】精确拾取 DP 的**覆盖面统计**：54 层逐层跑一遍，打印每层是否被精确解接管、
//! 拒绝理由、以及 `(picks, doors, kills)` 维数。
//!
//! 口径（写进结论时必须带上这条）：
//! * 玩家用**同一套强属性**（hp/atk/def=9999）扫描 ⇒ A1（有损血怪）几乎不响，
//!   所以这是**上界口径**的覆盖率，不是链路真实覆盖率；
//! * 入场格取层的 `start`，`flags=0`，不带背包、不带钥匙 ⇒ 门/钥匙相关的可达性
//!   与链路态不同；
//! * 结论只用来回答「这套 DP 的闸门在 54 层里能放行多少层」。
//!
//! 运行：`cargo run --release -p game-solver --example exact_coverage`

use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_solver::exact_pickup::{exact_stat, exact_stat_clear};
use game_solver::ingest::ingest;
use game_solver::solve_local::{load_floor, solve_local_exit_toward, PlayerSpec};
use std::collections::BTreeMap;

fn nums(t: &str) -> Vec<i32> {
    t.lines()
        .map(|l| l.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect()
}

fn main() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
    let si = ingest(
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
    let t = CombatTables {
        enemy_base_hp: nums(include_str!("../../game-core/tests/fixtures/enemybasehp.txt")),
        enemy_base_atk: nums(include_str!("../../game-core/tests/fixtures/enemybaseatk.txt")),
        enemy_base_def: nums(include_str!("../../game-core/tests/fixtures/enemybasedef.txt")),
        difficulty_multipliers: nums(include_str!(
            "../../game-core/tests/fixtures/difficulty_multipliers.txt"
        )),
    };
    let gold = nums(include_str!("../../game-core/tests/fixtures/enemybasegold.txt"));
    let base = t.to_records(&gold);
    let scaled = scale_enemy_stats(&base, 1);
    let tile_walk = nums(include_str!("../../game-core/tests/fixtures/tile_walkability.txt"));

    let strong = PlayerSpec::from_combat(PlayerCombat {
        hp: 9999,
        atk: 9999,
        def: 9999,
        yellow_keys: 0,
        blue_keys: 0,
        red_keys: 0,
        gold: 0,
        item_types: Vec::new(),
        item_uses: Vec::new(),
        item_stack_size: 0,
    });

    let mut hist: BTreeMap<&'static str, i32> = BTreeMap::new();
    let mut adopted = 0;
    println!("floor adopted reason picks doors kills out_hp trunc ms");
    for f in 1..=54 {
        let Ok(inp) = load_floor(&si, f as usize, (1, 11), strong.clone(), scaled.clone(), base.clone(), &tile_walk)
        else {
            println!("{f} skipped load_floor_err");
            continue;
        };
        let Some(start) = inp.level.as_ref().map(|l| l.start) else { continue };
        let t0 = std::time::Instant::now();
        let (_ex, _hit) = solve_local_exit_toward(&inp, start, &strong, 0, None, None);
        let ms = t0.elapsed().as_millis();
        let st = exact_stat();
        *hist.entry(st.reason).or_insert(0) += 1;
        if st.adopted {
            adopted += 1;
        }
        println!(
            "{f} {} {} {} {} {} {} {} {}",
            st.adopted, st.reason, st.picks, st.doors, st.kills, st.out_hp, st.steps, ms
        );
        // 每层独立：下一层不要继承统计（`solve_layer_with` 入口本就会清，这里再显式一次）
        exact_stat_clear();
    }
    println!("---- adopted={adopted}/54");
    for (k, v) in &hist {
        println!("reason {k} = {v}");
    }
}
