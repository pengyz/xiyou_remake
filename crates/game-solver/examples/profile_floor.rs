//! 单层状态空间剖析（证据工具，**不参与门禁**）：在指定层界下重放 f51→f54 见证链，
//! 再单独冷启目标层，逐项打印「撞界与否 / 出层血 / 航点数 / 耗时」与目标层的**状态空间
//! 清单**（可走格、实体类型直方图、独立拾取物清单及其效果、以及**从落点出发的有效可走集**）。
//!
//! 用法：
//! ```text
//! cargo run --release -p game-solver --example profile_floor -- 54 200000
//! cargo run --release -p game-solver --example profile_floor -- 54 1000000
//! PROFILE_GRID=1 cargo run --release -p game-solver --example profile_floor -- 54 200000
//! PROFILE_ENTRY=7,9 cargo run --release -p game-solver --example profile_floor -- 54 200000
//! ```
//! 参数：`<floor>`（缺省 54）、`<layer_limit>`（缺省 200000，写进 `SOLVER_LAYER_LIMIT`）。
//!
//! **入场格必须复刻 `landing`**：换层落点由 `solve_global.rs:289` 的 `landing(目标层, floor,
//! went_down)` 决定（`pub(crate)`，本文件里逐字复刻成 `landing_replica`）。第一版曾直接把
//! 「上一层出口格」当成本层入场格，血量整体偏一颗药（f54 差 +200）——教训写在这里，别重犯。
//!
//! **能力边界（诚实声明）**：本 example 只用 lib 的 **public API**（`run_witness` /
//! `solve_local_exit_toward` / `load_floor` / `SolveInput`）——因此**拿不到** `unique_keys`、
//! `Key` 各维度取值数、同键 Pareto 合并数：那些量在 `solve_local.rs` 的私有
//! `Search::run_root` 内（`by_key: BTreeMap<Key, Vec<u32>>`）。要测它们必须临时插桩
//! **探针副本**（不改仓库 src），补丁与复现命令见
//! `docs/findings/2026-11-19-f54-state-profile.md` §「方法」。
//!
//! 本 example 能独立复现的 A 级事实：撞界点（`truncated`）、出层血、层界抬升的线性代价、
//! **目标集过滤（`toward`）的价差**、以及「该层有几个独立 +hp 拾取物、它们是否全部可达」——
//! 最后一条足以算出该层出层血的**闭式上界**（f54：827 + 2400 = 3227，实测 200k/1M = 2027/2627）。

use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_data::defs::EffectKey;
use game_solver::ingest::{ingest, SolverInput};
use game_solver::solve_local::{
    load_floor, solve_local_exit_toward, LevelSpec, PlayerSpec, SolveInput,
};
use game_solver::witness::{run_witness, up_line_next, WPlayer, WitnessCfg};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

/// S0 玩家构造（与 `bin/solver.rs::mk_player` 同值：300/10/10、三色 0、空背包）。
fn mk_player() -> PlayerCombat {
    PlayerCombat {
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
    }
}

fn nums(t: &str) -> Vec<i32> {
    t.lines()
        .map(|l| l.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect()
}

/// 与 `bin/solver.rs::solve_global_input` 同构：读 55 槽 maplv/sprite + fixtures + 三表。
fn load_solver_input() -> SolverInput {
    let root = repo();
    let mut maplvs = Vec::new();
    let mut sprites = Vec::new();
    for i in 0..55 {
        maplvs.push(std::fs::read(root.join(format!("assets/raw/maplv{i}"))).unwrap());
        sprites.push(std::fs::read(root.join(format!("assets/raw/sprite{i}"))).unwrap());
    }
    let maplv_refs: Vec<&[u8]> = maplvs.iter().map(|v| v.as_slice()).collect();
    let sprite_refs: Vec<&[u8]> = sprites.iter().map(|v| v.as_slice()).collect();
    ingest(
        &maplv_refs,
        &sprite_refs,
        include_str!("../../game-core/tests/fixtures/level_scripts.txt"),
        include_str!("../../game-core/tests/fixtures/door12.txt"),
        include_str!("../../game-core/tests/fixtures/door13.txt"),
        include_str!("../../../data/tables/enemies.json"),
        include_str!("../../../data/tables/items.json"),
        include_str!("../../../data/tables/shop_goods.json"),
    )
    .expect("ingest")
}

const GLOBAL_ENTRY: (i32, i32) = (1, 11);
const GLOBAL_START_FLOOR: i32 = 51;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let floor: i32 = args.first().and_then(|s| s.parse().ok()).unwrap_or(54);
    let limit: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(200_000);
    // 与 `bin/solver.rs` 读同一个 env（`layer_node_limit()` 的唯一闸口）。
    std::env::set_var("SOLVER_LAYER_LIMIT", limit.to_string());

    let si = load_solver_input();
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
    let scaled = scale_enemy_stats(&base, 1); // difficulty=0 ⇒ ×1
    let tile_walk = nums(include_str!("../../game-core/tests/fixtures/tile_walkability.txt"));

    // 层图（1..=54），入场格与 bin 同值——`run_witness` 内部按塔序替换每层 entry。
    let mut levels: BTreeMap<i32, SolveInput> = BTreeMap::new();
    for f in 1..=54usize {
        if let Ok(inp) = load_floor(
            &si,
            f,
            GLOBAL_ENTRY,
            PlayerSpec::from_combat(mk_player()),
            scaled.clone(),
            base.clone(),
            &tile_walk,
        ) {
            if inp.level.is_some() {
                levels.insert(f as i32, inp);
            }
        }
    }

    println!("== profile_floor: target=f{floor} layer_limit={limit} ==");
    inspect_level(&si, levels.get(&floor).and_then(|i| i.level.as_ref()), floor);

    // ---------- ① 前置链 f51..f{floor-1}（拿真实出层态；不碰目标层）----------
    let span = (floor - GLOBAL_START_FLOOR).max(0) as usize;
    let t0 = Instant::now();
    let warm = run_witness(
        &levels,
        GLOBAL_START_FLOOR,
        GLOBAL_ENTRY,
        PlayerSpec::from_combat(mk_player()),
        0,
        &WitnessCfg { max_floors: span },
    );
    println!("-- prefix chain f{GLOBAL_START_FLOOR}..f{} (limit={limit}) --", floor - 1);
    for fl in &warm.floors {
        println!(
            "f{} ok={} truncated={} in_hp={} out_hp={} waypoints={} exit={:?} kind={}",
            fl.floor,
            fl.ok,
            fl.truncated,
            fl.in_hp,
            fl.out_hp,
            fl.route.len(),
            fl.exit,
            fl.kind
        );
    }
    println!(
        "prefix: status={} floors={} elapsed={:.2}s",
        warm.status,
        warm.floors.len(),
        t0.elapsed().as_secs_f64()
    );

    // ---------- ② 目标层单层隔离重放（冷启：ents_in=None ⇒ 该层 sprite_full）----------
    // `ents_in` 语义：None = 该层从未被推进过（求解侧展开成 sprite_full）；
    // Some = 位图里记的**该层**全表（绝不是上一层的残留表）⇒ 目标层首入必须是 None。
    let prev = warm.floors.last().filter(|fl| fl.floor == floor - 1);
    match (levels.get(&floor), prev) {
        (Some(inp), Some(prev)) => {
            // 入场格 = **目标层的落点**（`landing(目标层, floor, went_down)`），
            // 绝不是上一层出口格在上一层坐标里的位置——这是本 example 第一版的错，
            // 错法会把目标层当成「从爬梯格开始的房间」，少走一段梯口位移。
            let lv = inp.level.as_ref().expect("level");
            // 换层方向由**上一层出口梯的类型**决定（`stair_target`：ty8 ⇒ 下行），
            // 再喂给 `landing(目标层, floor, went_down)`——`want = went_down ? 7 : 8`。
            let went_down = levels
                .get(&(floor - 1))
                .and_then(|p| p.level.as_ref())
                .and_then(|pl| {
                    let (ex, ey) = {
                        let [a, b] = prev.exit?;
                        (a, b)
                    };
                    pl.entities
                        .iter()
                        .find(|e| matches!(e.ty, 7 | 8) && (e.x, e.y) == (ex, ey))
                        .map(|e| e.ty == 8)
                })
                .unwrap_or(false);
            let landing = landing_replica(lv, floor, went_down);
            let entry = match std::env::var("PROFILE_ENTRY")
                .ok()
                .and_then(|v| {
                    let p: Vec<i32> = v.split(',').filter_map(|s| s.trim().parse().ok()).collect();
                    if p.len() == 2 {
                        Some((p[0], p[1]))
                    } else {
                        None
                    }
                }) {
                Some(e) => e,
                None => landing.unwrap_or(GLOBAL_ENTRY),
            };
            println!(
                "-- entry: landing_replica(went_down={went_down})={landing:?} \
                 prev_exit(f{} 坐标)={:?} 采用={entry:?} --",
                floor - 1,
                prev.exit
            );
            let player = player_spec(&prev.out_player);
            let flags = parse_hex_u128(&prev.out_flags);
            let lays = game_solver::floorgraph::lay_targets();
            let toward = up_line_next(floor, inp.level.as_ref().expect("level"), &lays);
            let t1 = Instant::now();
            let (ex, hit) =
                solve_local_exit_toward(inp, entry, &player, flags, None, toward);
            let el = t1.elapsed();
            println!(
                "-- target floor isolated replay (entry={entry:?} in_hp={} flags={} toward={toward:?} hit={hit}) --",
                player.hp, flags
            );
            println!(
                "f{floor} ok={} truncated={} out_hp={} waypoints={} exit={:?} kind={:?} elapsed={:.2}s",
                ex.ok,
                ex.truncated,
                ex.out_hp,
                ex.route.len(),
                ex.exit,
                ex.kind,
                el.as_secs_f64()
            );
            println!(
                "note: 截断 ⇒ `materialize_exit` 契约要求 ok=false（截断结果不可信），\
                 即便搜索里已记录到可走通的出口目标也只回填参考值。"
            );
            // 【D 面】子集支配（纯加血拾取）的可观测统计：`enabled=false` 时
            // `reason/reason_ty` 指出卡在哪条闸门（禁静默）。
            {
                let st = game_solver::solve_local::pill_dom_stat();
                println!(
                    "pill_dom: enabled={} reason={} reason_ty={} pill_types={} pills={} states={} cmp={} pruned_new={} killed_old={} budget_exhausted={}",
                    st.enabled,
                    st.reason,
                    st.reason_ty,
                    st.pill_types,
                    st.pills,
                    st.states,
                    st.cmp,
                    st.pruned_new,
                    st.killed_old,
                    st.budget_exhausted
                );
            }

            // ---------- ②b 目标集过滤的代价：toward=None / 上行 / 下行 各挑一条 ----------
            // `solve_local_exit_toward` 只在**搜索结束后**按 `goal_edge(...).to` 过滤 goal，
            // 搜索本身与 toward 无关 ⇒ 两次调用的 nodes/keys 完全相同，只是挑哪个解不同。
            // 这一组数据用来量化「链只消费上行目标」损失了多少血量（=攻法 B 的收益上限）。
            println!(
                "-- target-set filter scan (identical search, only goal pick differs) --"
            );
            for t in [None, toward] {
                let t1 = Instant::now();
                let (e2, h2) = solve_local_exit_toward(inp, entry, &player, flags, None, t);
                println!(
                    "  toward={t:?} hit={h2} truncated={} out_hp={} exit={:?} kind={:?} elapsed={:.2}s",
                    e2.truncated,
                    e2.out_hp,
                    e2.exit,
                    e2.kind,
                    t1.elapsed().as_secs_f64()
                );
            }
        }
        _ => println!(
            "-- target floor isolated replay: 前置链没推进到 f{}，跳过 --",
            floor - 1
        ),
    }

    // ---------- ③ 与 `solver run` 同入口的整链（目标层这次是 memo 命中，仅验状态词）----------
    let t2 = Instant::now();
    let w = run_witness(
        &levels,
        GLOBAL_START_FLOOR,
        GLOBAL_ENTRY,
        PlayerSpec::from_combat(mk_player()),
        0,
        &WitnessCfg { max_floors: span as usize + 1 },
    );
    println!("-- witness chain f{GLOBAL_START_FLOOR}..f{floor} (=`solver run` 入口) --");
    for fl in &w.floors {
        println!(
            "f{} ok={} truncated={} out_hp={} waypoints={}",
            fl.floor,
            fl.ok,
            fl.truncated,
            fl.out_hp,
            fl.route.len()
        );
    }
    match &w.blocked {
        Some(b) => println!(
            "chain: status={} blocked=floor {} reason={} detail={} layer_ok={:?} layer_truncated={:?} elapsed={:.2}s",
            w.status,
            b.floor,
            b.reason,
            b.detail,
            b.layer_ok,
            b.layer_truncated,
            t2.elapsed().as_secs_f64()
        ),
        None => println!(
            "chain: status={} floors={} elapsed={:.2}s",
            w.status,
            w.floors.len(),
            t2.elapsed().as_secs_f64()
        ),
    }
}

fn player_spec(p: &WPlayer) -> PlayerSpec {
    PlayerSpec {
        hp: p.hp,
        atk: p.atk,
        def: p.def,
        yellow: p.yellow,
        blue: p.blue,
        red: p.red,
        gold: p.gold,
        item_types: p.item_types.clone(),
        item_uses: p.item_uses.clone(),
        item_stack_size: p.item_stack_size,
    }
}

fn parse_hex_u128(s: &str) -> u128 {
    let t = s.trim().trim_start_matches("0x");
    u128::from_str_radix(t, 16).unwrap_or(0)
}

/// `solve_global::landing` 的**逐字复刻**（那份是 `pub(crate)`，example 拿不到）。
/// 规则见 `crates/game-solver/src/solve_global.rs:289`：
/// target∈{0,50} 与 `target==1 && !went_down` 是特例；其余=目标层找**首个**
/// `param>>9==0` 的梯（`want = went_down ? 7 : 8`），落点=首个可行走邻格
/// （上→下→左→右，四邻皆不可走则留梯格）。
fn landing_replica(lv: &LevelSpec, target: i32, went_down: bool) -> Option<(i32, i32)> {
    if target == 0 {
        return Some((1, 2));
    }
    if target == 50 {
        return Some((6, 7));
    }
    if target == 1 && !went_down {
        return Some((6, 11));
    }
    let want = if went_down { 7 } else { 8 };
    let gate = lv.entities.iter().find(|e| e.ty == want && (e.param >> 9) == 0)?;
    let (mut gx, mut gy) = (gate.x, gate.y);
    let walk = |x: i32, y: i32| {
        x >= 0
            && y >= 0
            && x < lv.cols
            && y < lv.rows
            && lv.terrain[(y * lv.cols + x) as usize]
    };
    if walk(gx, gy - 1) {
        gy -= 1;
    } else if walk(gx, gy + 1) {
        gy += 1;
    } else if walk(gx - 1, gy) {
        gx -= 1;
    } else if walk(gx + 1, gy) {
        gx += 1;
    }
    Some((gx, gy))
}

/// 目标层状态空间清单：可走格、实体类型直方图、独立拾取物（尤其 +hp 药）明细。
///
/// 这是「层界被谁吃光」的直接线索：**每个独立拾取物都是一个可拾/可不拾的二元决策**，
/// 其集合幂集在 `Key.ents` 里两两不同。
fn inspect_level(si: &SolverInput, lv: Option<&LevelSpec>, floor: i32) {
    let Some(lv) = lv else {
        println!("level: floor {floor} 无规格（load_floor 失败）");
        return;
    };
    let cells = (lv.cols * lv.rows) as usize;
    let walkable = lv.terrain.iter().filter(|w| **w).count();
    let monster_ents = lv
        .entities
        .iter()
        .filter(|e| (41..=75).contains(&e.ty))
        .count();
    println!(
        "level: {}x{} cells={} walkable={} ents={} exits={} monster_ents={} enemy_table_rows={}",
        lv.cols,
        lv.rows,
        cells,
        walkable,
        lv.entities.len(),
        lv.exits.len(),
        monster_ents,
        lv.enemies.len()
    );
    let mut hist: BTreeMap<i32, usize> = BTreeMap::new();
    for e in &lv.entities {
        *hist.entry(e.ty).or_default() += 1;
    }
    let hist_s: Vec<String> = hist.iter().map(|(k, v)| format!("ty{k}x{v}")).collect();
    println!("level: ents_ty_hist {}", hist_s.join(" "));
    let stairs: Vec<String> = lv
        .entities
        .iter()
        .filter(|e| matches!(e.ty, 7 | 8))
        .map(|e| {
            format!(
                "ty{}@({},{}) param=0x{:x}(target={} dir_bit={})",
                e.ty,
                e.x,
                e.y,
                e.param,
                e.param & 0xff,
                e.param >> 9
            )
        })
        .collect();
    if !stairs.is_empty() {
        println!("level: stairs {}", stairs.join(" | "));
    }
    println!("level: start={:?} exits={:?}", lv.start, lv.exits);

    // 独立拾取物：cat 2|4 的「道物」里，效果只影响 hp/gold 的 = 可自由取舍的二元决策。
    let mut picks: Vec<(i32, i32, i32, String)> = Vec::new();
    let mut non_item: Vec<i32> = Vec::new();
    for e in &lv.entities {
        if !(13..=40).contains(&e.ty) && !(79..=86).contains(&e.ty) {
            continue;
        }
        let def = si.items.iter().find(|it| it.id == e.ty);
        if def.is_none() {
            // 同区间里还有「非道具」实体（脚本门/机关）——不是可拾物，单独列出。
            if !non_item.contains(&e.ty) {
                non_item.push(e.ty);
            }
            continue;
        }
        let desc = match def {
            Some(d) => {
                let eff = match &d.effect {
                    EffectKey::HpUp { mul } => format!("HpUp(+{mul}hp)"),
                    EffectKey::Heal { mul, div, .. } => format!("Heal(mul={mul},div={div})"),
                    EffectKey::Key { color } => format!("Key({color:?})"),
                    EffectKey::StatUp { stat } => format!("StatUp({stat:?})"),
                    EffectKey::Equip { tier, bonus } => format!("Equip(t{tier},+{bonus})"),
                    other => format!("{other:?}"),
                };
                format!("{} {}", d.name, eff)
            }
            None => "?无表行".to_string(),
        };
        picks.push((e.ty, e.x, e.y, desc));
    }
    println!("level: independent_item_pickups={}", picks.len());
    if !non_item.is_empty() {
        println!("level: non_item_entities_in_same_id_range={non_item:?}（脚本门等，不参与拾取组合）");
    }
    for (ty, x, y, desc) in &picks {
        println!("  pickup ty{ty} @({x},{y}) {desc}");
    }
    let n_hp = picks
        .iter()
        .filter(|(_, _, _, d)| d.contains("HpUp"))
        .count();
    if n_hp > 0 {
        let subsets = 2f64.powi(n_hp as i32);
        println!(
            "level: hp_only_pickups={n_hp} ⇒ 这些格子的「拾/不拾」组合上界 = 2^{n_hp} = {subsets:.0} 个不同实体表"
        );
        println!(
            "level: 每个组合在 `Key.ents` 里都不同 ⇒ 每个组合 × 每个到达格 × flags 都是独立去重键（见 findings §归因）"
        );
    }
    // ---- 纯地形可达性：从落点 BFS（4 邻、只走 `terrain`）----
    // 拾取物不阻挡通行（`classify` 把 cat 2|4 记为 pickup，踩上去即捡），所以
    // 「可达」⇒「能捡」。这条决定了「把所有 +hp 药一次走遍」在几何上是否成立。
    if let Some(start) = landing_replica(lv, floor, false) {
        if std::env::var("PROFILE_GRID").is_ok() {
            let mut occ: BTreeMap<(i32, i32), Vec<i32>> = BTreeMap::new();
            for e in &lv.entities {
                occ.entry((e.x, e.y)).or_default().push(e.ty);
            }
            println!("grid('#=unwalkable . =walkable, 数字=该格实体 ty):");
            for y in 0..lv.rows {
                let mut row = String::new();
                for x in 0..lv.cols {
                    if !lv.terrain[(y * lv.cols + x) as usize] {
                        row.push('#');
                    } else if let Some(ts) = occ.get(&(x, y)) {
                        row.push(char::from_digit((ts[0] % 10) as u32, 10).unwrap_or('?'));
                    } else {
                        row.push('.');
                    }
                }
                println!("  y={y:2} {row}");
            }
            for (k, v) in &occ {
                println!("  ent @{:?} tys={:?}", k, v);
            }
        }
        let idx = |x: i32, y: i32| (y * lv.cols + x) as usize;
        // 可走判据 = `solve_local.rs:1645` 的同款：地形 **或** 楼梯族豁免。
        // `stair_like` 的定义在 `:1613`（该格有 ty∈{6,7,8} 实体）；f54 的 16 个 ty6
        // 楼梯格正好是四条竖廊的竖向连接体，只用地形会把可达集错判成 12 格小袋。
        let has_stair = |x: i32, y: i32| {
            lv.entities
                .iter()
                .any(|e| e.x == x && e.y == y && matches!(e.ty, 6 | 7 | 8))
        };
        let walk = |x: i32, y: i32| {
            x >= 0
                && y >= 0
                && x < lv.cols
                && y < lv.rows
                && (lv.terrain[idx(x, y)] || has_stair(x, y))
        };
        let mut seen = vec![false; (lv.cols * lv.rows) as usize];
        let mut stack = vec![start];
        let mut reach = 0usize;
        if walk(start.0, start.1) {
            seen[idx(start.0, start.1)] = true;
            reach = 1;
        }
        while let Some((x, y)) = stack.pop() {
            for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
                let (nx, ny) = (x + dx, y + dy);
                if walk(nx, ny) && !seen[idx(nx, ny)] {
                    seen[idx(nx, ny)] = true;
                    reach += 1;
                    stack.push((nx, ny));
                }
            }
        }
        let mut on_pill = 0usize;
        let mut sum_hp = 0i64;
        let mut unreach: Vec<(i32, i32, i32)> = Vec::new();
        for (ty, x, y, d) in &picks {
            let r = *x >= 0
                && *y >= 0
                && *x < lv.cols
                && *y < lv.rows
                && seen[idx(*x, *y)];
            if r {
                on_pill += 1;
                if let Some(hi) = d.find("+").and_then(|i| {
                    d[i + 1..].split(|c: char| !c.is_ascii_digit()).next().and_then(|s| s.parse::<i64>().ok())
                }) {
                    if d.contains("HpUp") {
                        sum_hp += hi;
                    }
                }
            } else {
                unreach.push((*ty, *x, *y));
            }
        }
        println!(
            "level: reachable_cells={reach} (from landing {start:?}) pickups_reachable={on_pill}/{} hp_gain_if_all={sum_hp} unreachable={unreach:?}",
            picks.len()
        );
        println!(
            "level: 若该层无步数/HP 上限且药不阻挡，则真最优 out_hp = in_hp + {sum_hp}（见 findings §上界）"
        );
    }
}
