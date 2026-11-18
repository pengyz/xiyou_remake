//! 【求解器批0/批2】solver CLI：`ingest --summary`（批0 census）+ `solve-local --floor N`（批2 单层求解，C1 起；C6 起 floor1 接真数据）。
//!
//! I/O 只在此 bin（lib 纯度边界见 src/lib.rs；A9 纯度闸只查 src/*.rs 排除本文件）。
use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_solver::ingest::{census, ingest};
use game_solver::solve_local::{load_floor, solve_local, PlayerSpec, SolveInput};
use game_solver::solve_global::solve_global;
use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

/// S0 玩家构造（a.java:3482-3491 step8 / handoff new_game_scene：300/10/10、三色0、金币0、空背包）。
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

/// C5：真数据全局求解（层间图 1..=54，入场=(1,11) 来自 handoff load_floor(32,352)；I/O 全在本 bin）。
/// 全局入口（当前题面：51 层 (1,11)，来自 `state.rs s0()` / `handoff.rs:154 load_floor(32,352)`）。
/// **这是当前地图的常量，不是机制**——随机地图入口不同，必须由 `--entry/--start-floor`
/// 或上游配置覆盖；逐层门分析一律用该层自己的到场梯（`doors::entry_and_goals`）。
const GLOBAL_ENTRY: (i32, i32) = (1, 11);
const GLOBAL_START_FLOOR: i32 = 51;

/// `--entry X,Y`（缺省=当前题面入口）。
fn parse_entry(args: &[String]) -> (i32, i32) {
    let Some(i) = args.iter().position(|a| a == "--entry") else {
        return GLOBAL_ENTRY;
    };
    let v = args.get(i + 1).unwrap_or_else(|| panic!("--entry 需 X,Y"));
    let mut it = v.split(',');
    let x = it.next().and_then(|t| t.trim().parse().ok()).unwrap_or_else(|| panic!("--entry X,Y: {v}"));
    let y = it.next().and_then(|t| t.trim().parse().ok()).unwrap_or_else(|| panic!("--entry X,Y: {v}"));
    (x, y)
}

/// `--start-floor N`（缺省=当前题面起点层）。
fn parse_start_floor(args: &[String]) -> i32 {
    args.iter()
        .position(|a| a == "--start-floor")
        .and_then(|i| args.get(i + 1))
        .map(|v| v.parse().unwrap_or_else(|_| panic!("--start-floor 需整数: {v}")))
        .unwrap_or(GLOBAL_START_FLOOR)
}

fn solve_global_input(
    bound: u64,
) -> (
    game_solver::solve_global::GlobalInput,
    Vec<game_data::defs::ShopGoodDef>,
) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();
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
    let nums = |t: &str| -> Vec<i32> {
        t.lines()
            .map(|l| l.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().expect("fixture 整数"))
            .collect()
    };
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
    let scaled = scale_enemy_stats(&base, 1); // difficulty=0 ⇒ ×1（BR-4 同 solve_floor1）
    let tile_walk = nums(include_str!("../../../game-core/tests/fixtures/tile_walkability.txt"));
    let mut floors: std::collections::BTreeMap<i32, game_solver::solve_local::LevelSpec> =
        std::collections::BTreeMap::new();
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
            if let Some(lv) = inp.level {
                floors.insert(f as i32, lv);
            }
        }
    }
    let gi = game_solver::solve_global::GlobalInput {
        floors,
        start_floor: GLOBAL_START_FLOOR,
        start_pos: GLOBAL_ENTRY,
        start_player: PlayerSpec::from_combat(mk_player()),
        start_flags: 0,
        bound,
    };
    // 批5 H4：商品表随输入一起交出（`shop_goods.json` 已在 ingest 解析，17 行）。
    // 只有 `query` 路径用它；`solve-global` 忽略（旧入口恒 `&[]` ⇒ 批0-3 零变化）。
    (gi, si.shops)
}

/// 批4：三查询与 `solve-global` 共用同一份真数据输入（只构造一次 `GlobalInput`）。
fn solve_global_real(bound: u64) -> game_solver::solve_global::GlobalResult {
    let (gi, _shops) = solve_global_input(bound);
    solve_global(&gi)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(|s| s.as_str()) == Some("solve-global") {
            // C5（计划 §4 批3 判据③）：--bound 层间节点扩展数上限（支持 1e6 科学计数）、--out 落盘（缺省 stdout）。
            let bound = args
                .iter()
                .position(|a| a == "--bound")
                .and_then(|i| args.get(i + 1))
                .map(|v| {
                    v.parse::<f64>()
                        .map(|f| f as u64)
                        .unwrap_or_else(|_| panic!("--bound 需数值（可 1e6）: {v}"))
                })
                .unwrap_or(1_000_000);
            let out = args
                .iter()
                .position(|a| a == "--out")
                .and_then(|i| args.get(i + 1))
                .cloned();
            let g = solve_global_real(bound);
            let json = g.to_json();
            match out {
                Some(path) => std::fs::write(&path, &json).unwrap_or_else(|e| panic!("写 {path}: {e}")),
                None => println!("{json}"),
            }
            return;
        }
    if args.first().map(|s| s.as_str()) == Some("query") {
        // 批4 A4（plan §4 批4 判据命令）：`query --all [--out PATH]` 三查询。
        // 预期控制台：Q1 max_hp=<N>(<置信级>) / Q2 min_kills=<M>(<置信级>) / Q3 reachable=<true|false>
        let all = args.iter().any(|a| a == "--all");
        let picked: Vec<u8> = [1u8, 2, 3]
            .iter()
            .copied()
            .filter(|q| args.iter().any(|a| a == &format!("q{q}")))
            .collect();
        let which: Vec<u8> = if all || picked.is_empty() { vec![1, 2, 3] } else { picked };
        let num = |flag: &str, dflt: u64| -> u64 {
            args.iter()
                .position(|a| a == flag)
                .and_then(|i| args.get(i + 1))
                .map(|v| {
                    v.parse::<f64>()
                        .map(|f| f as u64)
                        .unwrap_or_else(|_| panic!("{flag} 需数值（可 1e6）: {v}"))
                })
                .unwrap_or(dflt)
        };
        let mut opts = game_solver::query::QueryOpts::default();
        let (d_bound, d_cap) = (opts.bound, opts.node_cap);
        opts.bound = num("--bound", d_bound);
        opts.node_cap = num("--node-cap", d_cap);
        if let Some(i) = args.iter().position(|a| a == "--ultimate") {
            let v = args.get(i + 1).unwrap_or_else(|| panic!("--ultimate 需整数列表"));
            opts.ultimate = v
                .split(',')
                .map(|s| s.trim().parse::<i32>().unwrap_or_else(|_| panic!("--ultimate 整数列表: {v}")))
                .collect();
        }
        let (input, shops) = solve_global_input(opts.bound);
        let mut results = std::collections::BTreeMap::new();
        for q in which {
            let r = game_solver::query::run_query(&input, q, &opts, &shops);
            println!("{}", r.console_line());
            results.insert(r.key.to_string(), r);
        }
        if let Some(path) = args
            .iter()
            .position(|a| a == "--out")
            .and_then(|i| args.get(i + 1))
            .cloned()
        {
            let doc = game_solver::query::to_doc(&results);
            std::fs::write(&path, &doc).unwrap_or_else(|e| panic!("写 {path}: {e}"));
        }
        return;
    }
    if args.first().map(|s| s.as_str()) == Some("floors") {
        // 层间转移图（P0）：楼梯 / 脚本 LAY / 道具三类通道，回答「主线怎么走、卡在哪」。
        let (input, _shops) = solve_global_input(1);
        let g = game_solver::floorgraph::graph(&input.floors);
        let reach = game_solver::floorgraph::reachable_by_stairs(&input.floors, input.start_floor);
        let mut n = 0usize;
        for (f, edges) in &g {
            let main: Vec<String> = edges
                .iter()
                .filter(|e| e.kind != game_solver::floorgraph::EdgeKind::ItemUp
                    && e.kind != game_solver::floorgraph::EdgeKind::ItemDown)
                .map(|e| format!("{:?}@({},{})→f{}", e.kind, e.cell.0, e.cell.1, e.to))
                .collect();
            n += main.len();
            if !main.is_empty() {
                println!("f{f} 边 {}: {}", main.len(), main.join("  "));
            }
        }
        let reach_list: Vec<String> = reach.iter().map(|f| f.to_string()).collect();
        println!("== 楼梯/脚本口径：{} 条边 / 从 f{} 可达 {} 层 ==", n, input.start_floor, reach.len());
        println!("可达层: {}", reach_list.join(","));
        let missing: Vec<String> = (0..=54).filter(|f| !reach.contains(f)).map(|f| f.to_string()).collect();
        println!("不可达层（仅楼梯/脚本）: {}", missing.join(","));
        // 门（ty83）的槽号：回答「这层的门触发哪个脚本、脚本里有没有换层」。
        for (f, lv) in &input.floors {
            let gates: Vec<String> = lv
                .entities
                .iter()
                .filter(|e| e.ty == 83)
                .map(|e| format!("({},{})slot={}", e.x, e.y, e.param))
                .collect();
            if !gates.is_empty() {
                println!("f{f} 门 {}: {}", gates.len(), gates.join("  "));
            }
        }
        // 落盘：data/tables/floor_transitions.json（机器可读，供 witness builder 与文档引用）。
        if let Some(pos) = args.iter().position(|a| a == "--out") {
            if let Some(path) = args.get(pos + 1) {
                let mut js = String::from("{\n  \"start_floor\": ");
                js.push_str(&input.start_floor.to_string());
                js.push_str(",\n  \"floors\": [");
                let mut first_floor = true;
                for (f, lv) in &input.floors {
                    if !first_floor {
                        js.push(',');
                    }
                    first_floor = false;
                    js.push_str(&format!("\n    {{\"floor\": {},\"edges\": [", f));
                    let edges = g.get(f).cloned().unwrap_or_default();
                    let mut first_edge = true;
                    for e in &edges {
                        if !first_edge {
                            js.push(',');
                        }
                        first_edge = false;
                        js.push_str(&format!(
                            "{{\"kind\":\"{:?}\",\"cell\":[{},{}],\"to\":{},\"detail\":\"{}\"}}",
                            e.kind, e.cell.0, e.cell.1, e.to, e.detail
                        ));
                    }
                    let gates: Vec<String> = lv
                        .entities
                        .iter()
                        .filter(|e| e.ty == 83)
                        .map(|e| format!("{{\"cell\":[{},{}],\"slot\":{}}}", e.x, e.y, e.param))
                        .collect();
                    js.push_str(&format!("],\"gates\":[{}]}}", gates.join(",")));
                }
                js.push_str("\n  ]\n}\n");
                std::fs::write(path, js).expect("写 floor_transitions.json");
                println!("已写 {path}");
            }
        }
    }
    if args.first().map(|s| s.as_str()) == Some("doors") {
        // 必经门清单：把 C4-a「充分+必要」判据推广到门（逻辑在 lib::doors）。
        // 口径：单独封住这扇门 ⇒ 出口不可达 = 必经；封住同色全部门 ⇒ 不可达 = 该色钥匙至少一把必须。
        let (mut input, _shops) = solve_global_input(1);
        input.start_pos = parse_entry(&args);
        input.start_floor = parse_start_floor(&args);
        let mut all: Vec<(i32, Vec<game_solver::doors::DoorFact>, game_solver::doors::FloorStatus)> = Vec::new();
        for (f, lv) in &input.floors {
            let scripts = game_solver::dynreach::fixture_scripts();
            let (facts, status) = game_solver::doors::analyze_floor_with(lv, &scripts);
            for (dx, dy, slot, reach) in game_solver::doors::dynamic_probe(lv, &scripts) {
                println!(
                    "   f{} 动态探针 脚本门@({dx},{dy}) 槽{slot} ⇒ {}",
                    f,
                    if reach { "可达（靠运行期召唤）" } else { "仍不可达" }
                );
            }
            all.push((*f, facts, status));
        }
        let mut json = String::from("{\"floors\":[");
        let mut first_floor = true;
        let (mut n_doors, mut n_nec, mut n_color, mut n_unreach) = (0usize, 0usize, 0usize, 0usize);
        for (f, facts, status) in &all {
            n_doors += facts.len();
            let nec: Vec<String> = facts
                .iter()
                .filter(|d| d.necessary)
                .map(|d| format!("{}({},{})", game_solver::doors::color_name(d.ty), d.x, d.y))
                .collect();
            let colors: Vec<&str> = {
                let mut cs: Vec<&str> = Vec::new();
                for d in facts.iter().filter(|d| d.color_needed) {
                    let n = game_solver::doors::color_name(d.ty);
                    if !cs.contains(&n) {
                        cs.push(n);
                    }
                }
                cs
            };
            n_nec += facts.iter().filter(|d| d.necessary).count();
            n_color += facts.iter().filter(|d| d.color_needed).count();
            if *status != game_solver::doors::FloorStatus::Ok {
                n_unreach += 1;
            }
            if !facts.is_empty() {
                println!(
                    "f{} 门{} 必经[{}] 同色必要[{}]{}",
                    f,
                    facts.len(),
                    if nec.is_empty() { "—".to_string() } else { nec.join(" ") },
                    if colors.is_empty() { "—".to_string() } else { colors.join("") },
                    match status {
                        game_solver::doors::FloorStatus::Ok => "",
                        game_solver::doors::FloorStatus::NoExit => "  ⚠无出发型出口（换层靠脚本，未建模）",
                        game_solver::doors::FloorStatus::BaselineUnreachable => "  ⚠门全开也走不到（非钥匙问题）",
                        game_solver::doors::FloorStatus::DynamicGate => {
                            "  ⚠脚本门在不可走格：已按动态模型判定（见逐格探针）"
                        }
                    }
                );
            }
            if !first_floor {
                json.push(',');
            }
            first_floor = false;
            let status_tag = match status {
                game_solver::doors::FloorStatus::Ok => "ok",
                game_solver::doors::FloorStatus::NoExit => "no_exit",
                game_solver::doors::FloorStatus::BaselineUnreachable => "baseline_unreachable",
                game_solver::doors::FloorStatus::DynamicGate => "dynamic_gate",
            };
            json.push_str(&format!("{{\"floor\":{},\"status\":\"{}\",\"doors\":[", f, status_tag));
            for (i, d) in facts.iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                json.push_str(&format!(
                    "{{\"ty\":{},\"color\":\"{}\",\"x\":{},\"y\":{},\"necessary\":{},\"color_needed\":{}}}",
                    d.ty,
                    game_solver::doors::color_name(d.ty),
                    d.x,
                    d.y,
                    d.necessary,
                    d.color_needed
                ));
            }
            json.push_str("]}");
        }
        println!(
            "== 合计 == 门 {} 扇 / 必经 {} / 同色必要 {} / 判定无效层 {}",
            n_doors, n_nec, n_color, n_unreach
        );
        json.push_str(&format!(
            "],\"summary\":{{\"doors\":{},\"necessary\":{},\"color_needed\":{},\"no_verdict_floors\":{}}}}}",
            n_doors, n_nec, n_color, n_unreach
        ));
        if let Some(path) = args
            .iter()
            .position(|a| a == "--out")
            .and_then(|i| args.get(i + 1))
            .cloned()
        {
            std::fs::write(&path, &json).unwrap_or_else(|e| panic!("写 {path}: {e}"));
        }
        return;
    }
    if args.first().map(|s| s.as_str()) == Some("keys") {
        // 逐层累计供需对账：必经门需求 vs 地面拾取供给（逻辑在 lib::keyflow）。
        let (input, _shops) = solve_global_input(1);
        let modes = [(game_solver::keyflow::DemandMode::Necessary, "必经"), (game_solver::keyflow::DemandMode::AllDoors, "全门")];
        for (mode, name) in modes {
            let r = game_solver::keyflow::analyze_with(&input.floors, mode, &game_solver::dynreach::fixture_scripts());
            println!(
                "== 模式={} == 累计需求 黄{} 红{} 蓝{} / 拾取供给 黄{} 红{} 蓝{}",
                name,
                r.total_demand[0], r.total_demand[1], r.total_demand[2],
                r.total_pickup[0], r.total_pickup[1], r.total_pickup[2]
            );
            for color in 0..3 {
                match r.first_break[color] {
                    Some(f) => println!("   {} 首次断供：f{}", game_solver::keyflow::COLORS[color], f),
                    None => println!("   {} 全程不断供", game_solver::keyflow::COLORS[color]),
                }
            }
            if !r.no_exit_floors.is_empty() {
                println!(
                    "   ⚠无出发型出口的层（换层靠脚本，未计入门需求）：{:?}",
                    r.no_exit_floors
                );
            }
            if !r.dynamic_floors.is_empty() {
                println!(
                    "   ⚠门判定依赖运行期召唤物的层（脚本门摆在不可走格，靠召唤 ty6 火触发地形豁免）：{:?}",
                    r.dynamic_floors
                );
            }
            for row in &r.rows {
                if row.demand == [0, 0, 0] && !row.broke.iter().any(|b| *b) {
                    continue;
                }
                println!(
                    "   f{:>2} 需求 {}/{}/{} 拾取 {}/{}/{} 余额 {}/{}/{}{}",
                    row.floor,
                    row.demand[0], row.demand[1], row.demand[2],
                    row.pickup[0], row.pickup[1], row.pickup[2],
                    row.balance[0], row.balance[1], row.balance[2],
                    if row.broke.iter().any(|b| *b) { "  ⚠断供" } else { "" }
                );
            }
        }
        return;
    }
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
        eprintln!("用法：solver ingest --summary | solver solve-local --floor <N> | solver solve-global [--bound N] [--out PATH] | solver query --all [--bound N] [--node-cap N] [--ultimate 36,40] [--out PATH] | solver doors [--out PATH] | solver keys");
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
    let player = mk_player();
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
