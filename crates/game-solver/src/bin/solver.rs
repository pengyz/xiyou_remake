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

/// S3-B/S4 共用：缩放敌表（difficulty=0 ⇒ ×1，与 solve_global 同口径）。
/// 单一真相源（bin 内只此一份；`outer` 与 `run --outer` 两分支共用）。
fn load_scaled_enemies() -> Vec<game_data::defs::EnemyDef> {
    let nums = |t: &str| -> Vec<i32> {
        t.lines().map(|l| l.trim()).filter(|s| !s.is_empty())
            .map(|s| s.parse().expect("fixture 整数")).collect()
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
    scale_enemy_stats(&tables.to_records(&gold), 1)
}

/// S3-B/S4 共用：把 LevelSpec 实体格（ty 41..=75）按 cells 映射注入块表 monsters。
/// 怪物来源单一真相源（公式全在 game-core，bin 只搬格子）。
fn inject_block_monsters(
    doc: &mut game_solver::outer::BlockInterfaces,
    floors: &std::collections::BTreeMap<i32, game_solver::solve_local::LevelSpec>,
) {
    for (floor, lv) in floors.iter() {
        let Some(fd) = doc.floors.get_mut(floor) else { continue };
        for e in &lv.entities {
            if (41..=75).contains(&e.ty) {
                if let Some(b) = fd.blocks.iter_mut().find(|b| b.cells.contains(&(e.x, e.y))) {
                    if !b.monsters.contains(&e.ty) {
                        b.monsters.push(e.ty);
                    }
                }
            }
        }
    }
}

/// S4-A：`run --outer` —— 外层全题块图搜索 → top-K 候选实化 → 全程 witness 草案。
/// 口径声明：`--combat-cost on` 是 S4 验收口径（LB 入边权=下界，分歧逐字段具名落
/// `budget_drifts`）；无该旗时 combat_cost=false（仅诊断，不作为验收数字）。
fn run_outer_cli(args: &[String]) {
    let out = args
        .iter()
        .position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
        .cloned();
    let doc_path = args
        .iter()
        .position(|a| a == "--doc")
        .and_then(|i| args.get(i + 1))
        .map(String::clone)
        .unwrap_or_else(|| "docs/findings/block-interfaces.json".to_string());
    let raw = std::fs::read_to_string(&doc_path)
        .unwrap_or_else(|e| panic!("读 {doc_path}: {e}（块表是输入数据，缺文件=具名失败，禁回落）"));
    let mut doc = game_solver::outer::load_block_interfaces(&raw).unwrap_or_else(|e| panic!("{e}"));
    // S4 主问题空间 = 全部可达层（用户裁定：最优是全局，不逐层之和）。
    let mut floors: Vec<i32> = vec![51, 52, 53, 54];
    floors.extend(1..=50);
    let mut run = game_solver::outer::RunSpec {
        floors,
        start: (51, GLOBAL_ENTRY),
        anchor_floor: 50,
        anchor_boss_ty: 72, // floor50 击败型72 → Ending（solve_local.rs 终局边语义）
        masked_exits: Vec::new(),
        combat_cost: false,
    };
    let (gi, shops) = solve_global_input(1);
    let levels: std::collections::BTreeMap<i32, SolveInput> = gi
        .floors
        .iter()
        .map(|(f, lv)| (*f, SolveInput { floor: *f, difficulty: 0, level: Some(lv.clone()) }))
        .collect();
    if args.iter().any(|a| a == "--combat-cost") {
        run.combat_cost = true;
        inject_block_monsters(&mut doc, &gi.floors);
    }
    let scaled = load_scaled_enemies();
    let graph = game_solver::outer::OuterGraph::build(&doc, &run.floors, &run.masked_exits);
    let res = game_solver::outer::outer_search(
        &graph,
        &run,
        &scaled,
        game_solver::outer::SearchBudget {
            max_pops: args
                .iter()
                .position(|a| a == "--max-pops")
                .and_then(|i| args.get(i + 1))
                .and_then(|v| v.parse().ok())
                .unwrap_or(5_000_000),
        },
    );
    let cfg = game_solver::outer_witness::OuterWitnessCfg {
        candidates: args
            .iter()
            .position(|a| a == "--outer-candidates")
            .and_then(|i| args.get(i + 1))
            .and_then(|v| v.parse().ok())
            .unwrap_or(5),
        max_floors: args
            .iter()
            .position(|a| a == "--max-floors")
            .and_then(|i| args.get(i + 1))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1024),
        boss_target: None, // 全题通关语义（默认）；段分治走 --outer-seg
    };
    let ow = game_solver::outer_witness::run_outer_witness(
        &graph,
        &run,
        &scaled,
        &levels,
        PlayerSpec::from_combat(mk_player()),
        0,
        &shops,
        &res,
        &cfg,
    );
    eprintln!(
        "outer: pops={} truncated={} frontier={} killable={} status={} floors={} attempts={} drifts={} total_steps={} ending_done={}",
        res.pops,
        res.truncated,
        res.frontier.len(),
        res.frontier.iter().filter(|e| e.killable && !e.fatal).count(),
        ow.status,
        ow.floors.len(),
        ow.attempts.len(),
        ow.budget_drifts.len(),
        ow.total_steps,
        ow.final_state.ending_done
    );
    for s in &res.skipped_trans {
        eprintln!("  [skip] {s}");
    }
    if let Some(b) = &ow.blocked {
        eprintln!("blocked=floor {} reason={} detail={}", b.floor, b.reason, b.detail);
    }
    for d in ow.budget_drifts.iter().take(20) {
        eprintln!(
            "drift[cand{} visit{} f{} {}] outer={} inner={}",
            d.candidate, d.visit, d.floor, d.field, d.outer, d.inner
        );
    }
    if ow.budget_drifts.len() > 20 {
        eprintln!("drifts 共 {} 条（只回显前 20，JSON 全量落盘）", ow.budget_drifts.len());
    }
    match out {
        Some(path) => {
            std::fs::write(&path, ow.to_json_pretty()).unwrap_or_else(|e| panic!("写 {path}: {e}"));
            println!("已写 {path}");
        }
        None => println!("{}", ow.to_json()),
    }
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


/// S5 机制2：`run --outer-seg [--max-pops N] [--out P]` —— boss 锚段链分治。
/// 口径：combat_cost=true（同 S4 验收口径）；每段预算 = --max-pops（默认 20M，
/// 纪律：禁抬 cap 换有解）；blocked 段具名、只拼已得段前缀；链级
/// `replay_outer_witness` 自证（Some("pass")/失败串）写进 ChainReport。
fn run_outer_seg_cli(args: &[String]) {
    let doc_path = args
        .iter()
        .position(|a| a == "--doc")
        .and_then(|i| args.get(i + 1))
        .map(String::clone)
        .unwrap_or_else(|| "docs/findings/block-interfaces.json".to_string());
    let raw = std::fs::read_to_string(&doc_path)
        .unwrap_or_else(|e| panic!("读 {doc_path}: {e}（块表是输入数据，缺文件=具名失败，禁回落）"));
    let mut doc = game_solver::outer::load_block_interfaces(&raw).unwrap_or_else(|e| panic!("{e}"));
    let (gi, shops) = solve_global_input(1);
    let levels: std::collections::BTreeMap<i32, SolveInput> = gi
        .floors
        .iter()
        .map(|(f, lv)| (*f, SolveInput { floor: *f, difficulty: 0, level: Some(lv.clone()) }))
        .collect();
    inject_block_monsters(&mut doc, &gi.floors);
    let scaled = load_scaled_enemies();
    let max_pops: u64 = args
        .iter()
        .position(|a| a == "--max-pops")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(20_000_000);
    let cfg = game_solver::outer_chain::ChainCfg {
        max_pops,
        candidates: 5,
        max_floors: 1024,
    };
    let segs = game_solver::outer_chain::mainline_segments();
    let player0 = PlayerSpec::from_combat(mk_player());
    let t0 = std::time::Instant::now();
    let rep = game_solver::outer_chain::run_chain(&mut doc, &levels, &scaled, &shops, &player0, 0, &segs, &cfg);
    println!(
        "== outer-seg: status={} total_pops={} total_steps={} ending_done={} elapsed={:?} ==",
        rep.status, rep.total_pops, rep.total_steps, rep.ending_done, t0.elapsed()
    );
    for s in &rep.segments {
        println!(
            "  seg {} status={} pops={} floors_solved={} handoff={:?} blocked={:?}",
            s.name, s.status, s.pops, s.floors_solved, s.handoff_ok, s.blocked.as_ref().map(|b| (&b.reason, &b.detail))
        );
    }
    println!("  replay={:?}", rep.replay);
    // 关键数字：总损血 = 300−末层 hp；购买 = route 里 buy 臂 token 计数（`buy@`）；
    // 回拾 = 重访层对数（同层二次入场计一次回拾）。
    let last = rep.floors.last();
    let buys: usize = rep
        .floors
        .iter()
        .flat_map(|f| f.route.iter())
        .filter(|t| t.starts_with("buy@") || t.contains("buy"))
        .count();
    let mut seen = std::collections::BTreeSet::new();
    let mut reentry = 0usize;
    for f in &rep.floors {
        if !seen.insert(f.floor) {
            reentry += 1;
        }
    }
    println!(
        "  final hp={} gold={} yellow={}; 总损血={} 购买臂={} 回拾={}",
        last.map(|f| f.out_player.hp).unwrap_or(0),
        last.map(|f| f.out_player.gold).unwrap_or(0),
        last.map(|f| f.out_player.yellow).unwrap_or(0),
        300 - last.map(|f| f.out_player.hp).unwrap_or(300),
        buys,
        reentry,
    );
    // 复核黄2 首验（blocked 前缀版）：链 blocked 时 ChainReport.floors 为空，
    // 但首个有解段的段内 witness floors 就是**真数据 witness**——对它做端到端回放。
    if rep.replay.is_none() {
        if let Some(seg) = rep.segments.iter().find(|s| !s.witness.floors.is_empty()) {
            let first_entry = segs[0].start.map(|(_, c)| c).unwrap_or((1, 11));
            match game_solver::outer_witness::replay_outer_witness(&levels, first_entry, &shops, &seg.witness.floors) {
                Ok(()) => println!("  seg-prefix-replay[{}]: pass（{} 层）", seg.name, seg.witness.floors.len()),
                Err(e) => println!(
                    "  seg-prefix-replay[{}]: FAILED visit={} floor={} token_index={} token={:?} stage={} reason={}",
                    seg.name, e.visit, e.floor, e.token_index, e.token, e.stage, e.reason
                ),
            }
        }
    }
    if let Some(path) = args.iter().position(|a| a == "--out").and_then(|i| args.get(i + 1)) {
        let text = serde_json::to_string_pretty(&rep).expect("ChainReport 序列化");
        std::fs::write(path, format!("{text}\n")).unwrap_or_else(|e| panic!("写 {path}: {e}"));
        println!("== 已写 {path} ==");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(|s| s.as_str()) == Some("outer") {
        // S2 外层骨架最小闭环：第 1 段（f51→f54→f1..f10，锚=boss75@f10）
        // 产出段内块路径 + Pareto 前沿；`--out PATH` 落盘 JSON。
        let doc_path = args
            .iter()
            .position(|a| a == "--doc")
            .and_then(|i| args.get(i + 1))
            .map(String::clone)
            .unwrap_or_else(|| "docs/findings/block-interfaces.json".to_string());
        let raw = std::fs::read_to_string(&doc_path)
            .unwrap_or_else(|e| panic!("读 {doc_path}: {e}（块表是输入数据，缺文件=具名失败，禁回落）"));
        let mut doc = game_solver::outer::load_block_interfaces(&raw)
            .unwrap_or_else(|e| panic!("{e}"));
        let plans = game_solver::outer::plan_segments(&doc);
        let mut run = match plans.iter().find(|(_, p)| matches!(p, game_solver::outer::SegmentPlan::Run(_))) {
            Some((name, game_solver::outer::SegmentPlan::Run(r))) => (name.clone(), r.clone()),
            _ => panic!("无任何可执行段（计划具名 Off 全落空）"),
        };
        if args.iter().any(|a| a == "--combat-cost") {
            run.1.combat_cost = true;
            // 怪物来源：LevelSpec 实体格（ty 41..=75）按 cells 映射注入块表 monsters
            // （公式全部 game-core 导出：sensitivity_table + kill_reward，solver 零重算）。
            let (input, _) = solve_global_input(1);
            inject_block_monsters(&mut doc, &input.floors);
        }
        // 缩放敌表（difficulty=0 ⇒ ×1，与 solve_global 同口径）
        let scaled = load_scaled_enemies();
        let graph = game_solver::outer::OuterGraph::build(&doc, &run.1.floors, &run.1.masked_exits);
        let res = game_solver::outer::outer_search(
            &graph,
            &run.1,
            &scaled,
            game_solver::outer::SearchBudget {
                max_pops: args.iter().position(|a| a == "--max-pops")
                    .and_then(|i| args.get(i + 1))
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(200_000),
            },
        );
        // 摘要（stdout）
        println!("== outer 第1段 {} ==", run.0);
        println!(
            "pops={} truncated={} 前沿={} 条 unapplied_exports={} needs_sensitivity={:?} skipped_trans={}",
            res.pops,
            res.truncated,
            res.frontier.len(),
            res.unapplied_exports,
            res.needs_sensitivity,
            res.skipped_trans.len()
        );
        for s in &res.skipped_trans {
            println!("  [skip] {s}");
        }
        for (i, e) in res.frontier.iter().enumerate() {
            println!(
                "  前沿[{}] steps={} atk={} def={} hp={} gold={} y/r/b={}/{}/{} killable={} fatal={} path={}",
                i, e.steps, e.state.atk, e.state.def, e.state.hp, e.state.gold,
                e.state.yellow, e.state.red, e.state.blue, e.killable, e.fatal,
                e.path.join(" → ")
            );
            if !e.gates.is_empty() {
                println!("    gates: {}", e.gates.join("; "));
            }
        }
        // S3-C：--lb-out 落盘 LB 下界表（敏感表 @ 段起点属性 + 块级 LB 汇总）。
        if let Some(lb_path) = args
            .iter()
            .position(|a| a == "--lb-out")
            .and_then(|i| args.get(i + 1))
            .cloned()
        {
            let table = game_core::rules::sensitivity::sensitivity_table(&scaled, 10, 10, false, false);
            let mut js = String::from(
                "{\n  \"口径\": \"LB 下界表：game-core sensitivity_table（armor_gate=true，difficulty×1），属性取第 1 段起点 atk=10 def=10 held=空；counter_loss=-1=打不动（LB 计 0）；reward=combat::kill_reward 精确值\",\n",
            );
            js.push_str("  \"start_stats\": { \"atk\": 10, \"def\": 10, \"hp\": 300, \"held\": [] },\n");
            js.push_str("  \"enemies\": [");
            for (i, r) in table.iter().enumerate() {
                if i > 0 { js.push(','); }
                js.push_str(&format!(
                    "\n    {{\"ety\": {}, \"can_kill\": {}, \"counter_loss\": {}, \"one_hit_atk\": {}, \"zero_loss_def\": {}}}",
                    r.ety, r.can_kill, r.counter_loss, r.one_hit_atk, r.zero_loss_def
                ));
            }
            js.push_str("\n  ],\n  \"blocks\": [");
            let mut first_blk = true;
            for (ni, mons) in graph.monsters_of.iter().enumerate() {
                if mons.is_empty() { continue; }
                let (loss, reward) = game_solver::outer::block_combat_cost(&scaled, mons, 10, 10, 0, 0);
                let f = graph.floor_of[ni];
                let bid = &doc.floors[&f].blocks[graph.block_of[ni]].id;
                if !first_blk { js.push(','); }
                first_blk = false;
                js.push_str(&format!(
                    "\n    {{\"floor\": {}, \"block\": \"{}\", \"monsters\": [{}], \"lb_loss\": {}, \"lb_reward\": {}}}",
                    f, bid, mons.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(","), loss, reward
                ));
            }
            js.push_str("\n  ]\n}\n");
            std::fs::write(&lb_path, &js).unwrap_or_else(|e| panic!("写 {lb_path}: {e}"));
            println!("LB 表已落盘 {lb_path}");
        }
        // 落盘
        let out = args
            .iter()
            .position(|a| a == "--out")
            .and_then(|i| args.get(i + 1))
            .cloned();
        let mut js = String::from("{\n  \"segment\": \"seg1_f51-f54_f1-f10\",\n  \"pops\": ");
        js.push_str(&res.pops.to_string());
        js.push_str(",\n  \"truncated\": ");
        js.push_str(if res.truncated { "true" } else { "false" });
        js.push_str(",\n  \"unapplied_exports\": ");
        js.push_str(&res.unapplied_exports.to_string());
        js.push_str(",\n  \"needs_sensitivity\": [");
        js.push_str(&res.needs_sensitivity.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(","));
        js.push_str("],\n  \"skipped_trans\": [");
        js.push_str(&res.skipped_trans.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(","));
        js.push_str("],\n  \"frontier\": [");
        for (i, e) in res.frontier.iter().enumerate() {
            if i > 0 {
                js.push(',');
            }
            js.push_str(&format!(
                "\n    {{\"steps\": {}, \"killable\": {}, \"fatal\": {}, \"atk\": {}, \"def\": {}, \"hp\": {}, \"gold\": {}, \
                 \"yellow\": {}, \"red\": {}, \"blue\": {}, \"path\": [{}], \"gates\": [{}]}}",
                e.steps, e.killable, e.fatal, e.state.atk, e.state.def, e.state.hp, e.state.gold,
                e.state.yellow, e.state.red, e.state.blue,
                e.path.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(","),
                e.gates.iter().map(|g| format!("\"{g}\"")).collect::<Vec<_>>().join(","),
            ));
        }
        js.push_str("\n  ]\n}\n");
        match out {
            Some(path) => std::fs::write(&path, &js).unwrap_or_else(|e| panic!("写 {path}: {e}")),
            None => print!("{js}"),
        }
        return;
    }
    if args.first().map(|s| s.as_str()) == Some("probe-f13") {
        // S2 矛盾探测器（引擎实测）：f13 火环是否挡楼梯主径。
        let (input, _shops) = solve_global_input(1);
        let lv = input
            .floors
            .get(&13)
            .unwrap_or_else(|| panic!("f13 不在层表（真数据加载失败，禁回落）"));
        let ents: Vec<(i32, i32, i32)> =
            lv.entities.iter().map(|e| (e.ty, e.x, e.y)).collect();
        let stairs: Vec<(i32, i32)> = lv
            .entities
            .iter()
            .filter(|e| e.ty == 7 || e.ty == 8)
            .map(|e| (e.x, e.y))
            .collect();
        let p = game_solver::outer::probe_fire_ring(lv.cols, lv.rows, &lv.terrain, &ents, (1, 11), &stairs, (6, 3))
            .unwrap_or_else(|e| panic!("f13 probe 前提不成立: {e}"));
        println!("== probe-f13（引擎实测，坐标=当前题面常量）==");
        println!("ty10 火环 {} 处: {:?}", p.ty10_cells.len(), p.ty10_cells);
        println!("楼梯 {:?} 可达（火环=墙口径）: {}", p.stairs, p.stairs_reachable);
        println!("金箍棒格 (6,3) 不经火环可达: {}", p.cudgel_reachable_without_fire);
        let verdict = if p.stairs_reachable && !p.cudgel_reachable_without_fire {
            "无矛盾：火环只隔离 (6,3) 金箍棒，不挡楼梯主径"
        } else if p.stairs_reachable {
            "火环不挡主径但 (6,3) 可绕过（与 S1 块图不一致，待核）"
        } else {
            "矛盾成立：火环挡住楼梯主径（需改题面理解）"
        };
        println!("结论: {verdict}");
        let out = args
            .iter()
            .position(|a| a == "--out")
            .and_then(|i| args.get(i + 1))
            .cloned();
        let js = format!(
            "{{\n  \"ty10_cells\": {:?},\n  \"stairs\": {:?},\n  \"stairs_reachable\": {},\n  \"cudgel_reachable_without_fire\": {},\n  \"verdict\": \"{verdict}\"\n}}\n",
            p.ty10_cells, p.stairs, p.stairs_reachable, p.cudgel_reachable_without_fire
        );
        match out {
            Some(path) => std::fs::write(&path, &js).unwrap_or_else(|e| panic!("写 {path}: {e}")),
            None => {}
        }
        return;
    }
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
    if args.first().map(|s| s.as_str()) == Some("keydemand") {
        // 【keydemand】只读分析：每层「入场格 → 任一上行出口」最少需要多少把各色钥匙。
        // 零接线：本分支只打印/落盘数字，不改 witness / solve_local / exact_pickup 的任何调用点。
        // 假设 1-5 见 lib::keydemand 模块文档，CLI 先打印再出行。
        let (input, _shops) = solve_global_input(1);
        let mut floors: Vec<game_solver::keydemand::FloorDemand> = Vec::new();
        for (f, lv) in &input.floors {
            floors.push(game_solver::keydemand::analyze_floor(*f, lv));
        }
        println!(
            "== 假设 == 1) 只算走到任一上行梯（不算支线/宝箱/商店）；2) 条件门 ty83 与封印门 ty81/4 按可通行（gates_ignored）；3) 怪物按可清（monsters_ignored）；4) 火 ty6 按引擎豁免可站（hazards），火墙 ty10 属硬阻挡；5) 硬阻挡 ty9/10/11/12 + 商人 cat16 + 联动门 cat32 按永久不可通行（hard_blocks，需求可能偏高）"
        );
        println!("== 口径 == needed=(y,b,r) 黄/蓝/红；! = 命中网格上界 4（真需求可能更大）==");
        for d in &floors {
            println!("{}", d.cli_line());
        }
        let mut hist: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        let (mut n_ok, mut n_no_entry, mut n_no_up, mut n_unreach, mut n_grid) = (0, 0, 0, 0, 0);
        for d in &floors {
            match d.status {
                game_solver::keydemand::FloorStatus::Ok => {
                    n_ok += 1;
                    let key = match d.needed {
                        Some([y, b, r]) => format!("({y},{b},{r}){}", if d.grid_bound { "!" } else { "" }),
                        None => "-".to_string(),
                    };
                    *hist.entry(key).or_insert(0) += 1;
                }
                game_solver::keydemand::FloorStatus::NoEntry => n_no_entry += 1,
                game_solver::keydemand::FloorStatus::NoUpExit => n_no_up += 1,
                game_solver::keydemand::FloorStatus::Unreachable => n_unreach += 1,
                game_solver::keydemand::FloorStatus::NeedsMoreThanGrid => n_grid += 1,
                game_solver::keydemand::FloorStatus::TooManyDoors
                | game_solver::keydemand::FloorStatus::TooManyPickups => {}
            }
        }
        println!(
            "== 合计 == 层 {} / ok {} / no_entry {} / no_up_exit {} / blocked_by_assumption {} / needs_more_than_grid {}",
            floors.len(), n_ok, n_no_entry, n_no_up, n_unreach, n_grid
        );
        let dist: Vec<String> = hist.iter().map(|(k, v)| format!("{k}×{v}")).collect();
        println!("== needed 分布 == {}", dist.join(" "));
        let json = game_solver::keydemand::to_json(&floors);
        if let Some(path) = args
            .iter()
            .position(|a| a == "--out")
            .and_then(|i| args.get(i + 1))
            .cloned()
        {
            let text = serde_json::to_string_pretty(&json).expect("keydemand JSON 序列化");
            std::fs::write(&path, format!("{text}\n")).unwrap_or_else(|e| panic!("写 {path}: {e}"));
            println!("== 已写 {path} ==");
        }
        return;
    }
    if args.first().map(|s| s.as_str()) == Some("run") {
        // 【S4-A】`run --outer`：外层全题块图 → 内层逐层实化 → 全程 witness 草案。
        if args.iter().any(|a| a == "--outer") {
            run_outer_cli(&args);
            return;
        }
        // 【S5 机制2】`run --outer-seg`：boss 锚段链分治 → 四段 witness 拼接 →
        // 链级回放自证（复核黄2 首验在真数据上执行）。
        if args.iter().any(|a| a == "--outer-seg") {
            run_outer_seg_cli(&args);
            return;
        }
        // P4：witness builder —— 从起点按塔序逐层 `solve_local_exit` 推进，产出可回放见证。
        // lib 侧纯函数（crates/game-solver/src/witness.rs），I/O 只在此处（A9 豁免面）。
        // 诚实口径：任一层 ok=false ⇒ status=blocked（截断则 capped），floors 只留成功前缀，
        // final.ending_done=false —— 绝不把半成品包装成通关（witness.rs 模块 doc 硬约束）。
        let out = args
            .iter()
            .position(|a| a == "--out")
            .and_then(|i| args.get(i + 1))
            .cloned();
        let max_floors = args
            .iter()
            .position(|a| a == "--max-floors")
            .and_then(|i| args.get(i + 1))
            .map(|v| v.parse::<usize>().unwrap_or_else(|_| panic!("--max-floors 需正整数: {v}")))
            .unwrap_or(game_solver::witness::DEFAULT_MAX_FLOORS);
        // 【P9.1 更正（复核 F3/F8）】商品表**不许丢**：旧代码在这里写成 `_shops`，
        // witness 链于是永远在「空商品表口径」下求层解（f3 的 17 行货表根本没进去）。
        let (gi, shops) = solve_global_input(1);
        let levels: std::collections::BTreeMap<i32, SolveInput> = gi
            .floors
            .into_iter()
            .map(|(f, lv)| {
                (
                    f,
                    SolveInput { floor: f, difficulty: 0, level: Some(lv) },
                )
            })
            .collect();
        let cfg = game_solver::witness::WitnessCfg { max_floors };
        let w = game_solver::witness::run_witness(
            &levels,
            GLOBAL_START_FLOOR,
            GLOBAL_ENTRY,
            PlayerSpec::from_combat(mk_player()),
            0,
            &shops,
            &cfg,
        );
        // 人读摘要走 stderr（stdout 留给 JSON：不加 --out 时可直接管道消费）。
        match &w.blocked {
            Some(b) => eprintln!(
                "status={} floors={} total_steps={} blocked=floor {} reason={} layer_ok={:?} layer_truncated={:?}",
                w.status,
                w.floors.len(),
                w.total_steps,
                b.floor,
                b.reason,
                b.layer_ok,
                b.layer_truncated
            ),
            None => eprintln!(
                "status={} floors={} total_steps={} ending_done={}",
                w.status,
                w.floors.len(),
                w.total_steps,
                w.final_state.ending_done
            ),
        }
        match out {
            Some(path) => {
                std::fs::write(&path, w.to_json_pretty())
                    .unwrap_or_else(|e| panic!("写 {path}: {e}"));
                println!("已写 {path}");
            }
            None => println!("{}", w.to_json()),
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
        // 【批8b 复核 F8】hp 上界剪枝必须**可见**：`solve-local` 是真数据层
        // 唯一开剪枝（`solve_local` 置 `hp_prune=true`）的入口，故在 stderr 打
        // 一行统计（JSON 走 stdout/`--out`，schema 不动——74 金样 sha 与
        // witness 兼容性依赖 stdout 逐字节）。
        let r = if floor == 1 {
            let r = solve_floor1();
            print_hp_bound_stat(floor);
            r
        } else {
            let r = solve_local(&SolveInput::placeholder(floor));
            // 【批8c 复核 F8-b】占位输入（`SolveInput::placeholder`，层数据未接）
            // 根本不建 hp 上界计划 ⇒ 该行在 floor≥2 永远 `not_run/0`，当诊断看会
            // 误导。行尾注明占位，保留「不静默」的既有形状（f51..f54 走 `run`
            // 分支，不经过本行，stdout 逐字节不变）。
            print_hp_bound_stat_note(floor, Some("placeholder:层路径未启用"));
            r
        };
        println!("{}", r.to_json());
        return;
    }
    if args.first().map(|s| s.as_str()) != Some("ingest") || args.get(1).map(|s| s.as_str()) != Some("--summary") {
        eprintln!("用法：solver ingest --summary | solver solve-local --floor <N> | solver solve-global [--bound N] [--out PATH] | solver query --all [--bound N] [--node-cap N] [--ultimate 36,40] [--out PATH] | solver doors [--out PATH] | solver keys | solver run [--out PATH] [--max-floors N]");
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

/// hp 上界诊断行（stderr；与 `run` 分支 `status=… floors=…` 同风格）。
/// `enabled=false` 时 `reason` 说明卡在哪条闸门（`disabled_by_caller`/
/// `no_hp_sources`/`post_kill_hp_spawn`/`script_hp_spawn`/`shop_repeatable`…），
/// 未跑层=`not_run`——**不静默**。
fn print_hp_bound_stat(floor: i32) {
    print_hp_bound_stat_note(floor, None);
}

/// 同上，`note` 非空时在行尾加注（用于「这一行没有实义」的占位面）。
fn print_hp_bound_stat_note(floor: i32, note: Option<&str>) {
    let s = game_solver::hp_bound::hp_bound_stat();
    match note {
        Some(n) => eprintln!(
            "hp_bound floor={} enabled={} reason={} reason_ty={} pruned_entry={} pruned_pop={} slots={} pickup_types={} root_hp={} root_bound={} ({})",
            floor, s.enabled, s.reason, s.reason_ty, s.pruned_entry, s.pruned_pop, s.slots, s.pickup_types, s.root_hp, s.root_bound, n
        ),
        None => eprintln!(
            "hp_bound floor={} enabled={} reason={} reason_ty={} pruned_entry={} pruned_pop={} slots={} pickup_types={} root_hp={} root_bound={}",
            floor, s.enabled, s.reason, s.reason_ty, s.pruned_entry, s.pruned_pop, s.slots, s.pickup_types, s.root_hp, s.root_bound
        ),
    }
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
