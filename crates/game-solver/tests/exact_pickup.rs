//! 【P9】精确拾取 DP：判据 1–8（TDD 红先行，实现见 `src/exact_pickup.rs`）。
//!
//! | # | 测试 | 守的是什么 |
//! |---|---|---|
//! | 1 | `exact_dp_f3_reaches_4027` | f3 实跑：`out_hp == 4027`、`ok && !truncated`、`adopted`、18/5/7 维全满、回放步数一致 |
//! | 1' | `exact_dp_adopts_simple_layer` | 正向小夹具（2 件加血 + 1 只零损血怪）⇒ `100+400`；规模缩到最小，收益推导坏了先红 |
//! | 2a | `exact_dp_off_on_monster_damage` | 有损血怪 ⇒ `Off(monster_damage)` |
//! | 2b | `exact_dp_off_on_step_arm_hazard` | 伤害型步臂（ty60）⇒ `Off(step_arm_hazard_reachable)` |
//! | 2c | `exact_dp_off_on_retriggerable_gate` | 非一次性门槽（param=32）⇒ `Off(retriggerable_gate_slot)` |
//! | 2d | `exact_dp_off_on_script_hp_spawn` | 门脚本生成 hp 物（f35 槽 62）⇒ `Off(script_hp_spawn)` |
//! | 2e | `exact_dp_off_on_post_kill_spawn` | 击杀生成通道（ty69）⇒ `Off(post_kill_hp_spawn)` |
//! | 2f | `exact_dp_off_on_too_many_pickups` | 拾取物 > `MAX_PICKUPS` ⇒ `Off(too_many_pickups)` |
//! | 3 | `exact_pickup::exact_dp_unit_tests::exact_dp_corrupted_prediction_is_off` | DP 预测被篡改 ⇒ `replay_mismatch`（防线夹具，lib 单测） |
//!
//! 判据 2a–2f 都是**反例判据**：断言拒绝理由的**具名**字面（不是「没命中」这种弱断言）。
//! 判据 1/1' 是**正向**判据：夹具里把反例去掉，精确解必须接管——这样「闸门恒 Off」
//! 这种假绿会同时让正向判据红。
//!
//! 统计口径（`ExactStat`）来自 `game_solver::exact_pickup::exact_stat()`（thread_local，
//! 每层 solve 开始时清零、memo 命中时随 memo 回放）。

use game_core::combat::{scale_enemy_stats, CombatTables};
use game_solver::exact_pickup::exact_stat;
use game_solver::ingest::{ingest, SolverInput};
use game_solver::solve_local::{
    load_floor, solve_local_exit_toward, solve_local_exit_toward_shops, EntSpec,
    ExitKind, ExitSpec, LevelSpec, PlayerSpec, SolveInput,
};
use std::path::PathBuf;

const SYNTH_ENTRY: (i32, i32) = (0, 1);
const F3_ENTRY: (i32, i32) = (2, 11);
const F3_FLAGS: u128 = 0xe337be_00000000;
const IN_HP_F3: i32 = 3227;
const F3_MAX_HP: i32 = 4027;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn nums(t: &str) -> Vec<i32> {
    t.lines()
        .map(|l| l.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect()
}

/// 与 `bin/solver.rs` / `tests/pure_pickup.rs` 同构的装载路径。
fn load_si() -> SolverInput {
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

fn tables() -> (Vec<game_data::defs::EnemyDef>, Vec<game_data::defs::EnemyDef>, Vec<i32>) {
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
    (scaled, base, tile_walk)
}

fn s0_spec(hp: i32) -> PlayerSpec {
    PlayerSpec {
        hp,
        atk: 10,
        def: 10,
        yellow: 0,
        blue: 0,
        red: 0,
        gold: 0,
        item_types: Vec::new(),
        item_uses: Vec::new(),
        item_stack_size: 0,
    }
}

/// f3 的**链路入场态**（A 级：`examples/f3_bound.rs::f3_chain_player` 同值）。
fn f3_chain_player() -> PlayerSpec {
    let mut item_types = vec![0u8; 32];
    let mut item_uses = vec![0u8; 32];
    item_types[0] = 13;
    item_uses[0] = 255;
    PlayerSpec {
        hp: IN_HP_F3,
        atk: 730,
        def: 730,
        yellow: 0,
        blue: 0,
        red: 0,
        gold: 225,
        item_types,
        item_uses,
        item_stack_size: 1,
    }
}

fn floor_input(si: &SolverInput, f: i32) -> SolveInput {
    let (scaled, base, tile_walk) = tables();
    load_floor(
        si,
        f as usize,
        (1, 11),
        PlayerSpec::from_combat(game_core::combat::PlayerCombat {
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
        }),
        scaled,
        base,
        &tile_walk,
    )
    .unwrap_or_else(|e| panic!("load_floor({f}) 失败: {e}"))
}

/// 5×3 单走廊夹具（y=1 全可走，入口 (0,1)，出口梯 (4,1)）；敌方表取 f54 的真表。
fn synth_with(si: &SolverInput, ents: Vec<EntSpec>, player: PlayerSpec) -> SolveInput {
    synth_wide(si, 5, ents, player)
}

fn synth_wide(si: &SolverInput, cols: i32, ents: Vec<EntSpec>, player: PlayerSpec) -> SolveInput {
    let f54 = floor_input(si, 54);
    let lv = f54.level.as_ref().expect("f54 必须有层");
    let rows = 3i32;
    let mut terrain = vec![false; (cols * rows) as usize];
    // 行主序：y=1 全可走，y=0/y=2 全墙。
    for x in 0..cols {
        terrain[(cols + x) as usize] = true;
    }
    let mut all = ents;
    all.push(EntSpec { ty: 7, x: cols - 1, y: 1, param: 1 });
    SolveInput {
        floor: 54,
        difficulty: f54.difficulty,
        level: Some(LevelSpec {
            cols,
            rows,
            terrain,
            start: SYNTH_ENTRY,
            entities: all,
            exits: vec![ExitSpec { x: cols - 1, y: 1, kind: ExitKind::Stair, index: 1 }],
            player: player.clone(),
            enemies: lv.enemies.clone(),
            enemy_base: lv.enemy_base.clone(),
            flags: 0,
        }),
    }
}

/// 同 `synth_wide`，但把层的 `floor` 改成指定值（脚本/掉落表按层号选语料槽）。
fn synth_floor(
    si: &SolverInput,
    floor: i32,
    cols: i32,
    ents: Vec<EntSpec>,
    player: PlayerSpec,
) -> SolveInput {
    let mut inp = synth_wide(si, cols, ents, player);
    inp.floor = floor;
    inp
}

/// 13×12 全通房间（起点 (0,0)，出口梯 (12,11)）——引擎场景夹具的固定网格。
fn synth_room(si: &SolverInput, ents: Vec<EntSpec>, player: PlayerSpec) -> SolveInput {
    let f54 = floor_input(si, 54);
    let lv = f54.level.as_ref().expect("f54 必须有层");
    let (cols, rows) = (13i32, 12i32);
    let terrain = vec![true; (cols * rows) as usize];
    let mut all = ents;
    all.push(EntSpec { ty: 7, x: 12, y: 11, param: 1 });
    SolveInput {
        floor: 54,
        difficulty: f54.difficulty,
        level: Some(LevelSpec {
            cols,
            rows,
            terrain,
            start: (0, 0),
            entities: all,
            exits: vec![ExitSpec { x: 12, y: 11, kind: ExitKind::Stair, index: 1 }],
            player: player.clone(),
            enemies: lv.enemies.clone(),
            enemy_base: lv.enemy_base.clone(),
            flags: 0,
        }),
    }
}

/// 7×5 全通矩形（入口 `SYNTH_ENTRY=(0,1)`，出口梯在 `exit`）——「顺路 vs 绕路」夹具。
fn synth_rect(
    si: &SolverInput,
    cols: i32,
    rows: i32,
    exit: (i32, i32),
    ents: Vec<EntSpec>,
    player: PlayerSpec,
) -> SolveInput {
    let f54 = floor_input(si, 54);
    let lv = f54.level.as_ref().expect("f54 必须有层");
    let terrain = vec![true; (cols * rows) as usize];
    let mut all = ents;
    all.push(EntSpec { ty: 7, x: exit.0, y: exit.1, param: 1 });
    SolveInput {
        floor: 54,
        difficulty: f54.difficulty,
        level: Some(LevelSpec {
            cols,
            rows,
            terrain,
            start: SYNTH_ENTRY,
            entities: all,
            exits: vec![ExitSpec { x: exit.0, y: exit.1, kind: ExitKind::Stair, index: 1 }],
            player: player.clone(),
            enemies: lv.enemies.clone(),
            enemy_base: lv.enemy_base.clone(),
            flags: 0,
        }),
    }
}

fn run_on(inp: &SolveInput, ps: &PlayerSpec) -> game_solver::solve_local::ExitSolve {
    let (ex, _hit) = solve_local_exit_toward(inp, SYNTH_ENTRY, ps, 0, None, None);
    ex
}

// ---------------------------------------------------------------------------
// 判据 1b（复核 F4，红先行）：**「同 hp、更少步」的态不得被反向剪掉**
//
// 夹具：7×5 全通房，入口 (0,1)，出口梯 (6,1)。ty32（+200 hp）摆在直路 (3,1)，
// ty29（+1 atk，**hp 增量为 0**）摆在绕路 (3,3)：
//   · 只捡 ty32 ⇒ 6 步到梯，hp = 500 + 200 = 700（atk 不变）
//   · 两件都捡 ⇒ 10 步到梯，hp = 700（同 hp，绕路只买到 +1 atk）
// 目标序是 (hp 最大, steps 最小) ⇒ 最优必须是 **6 步 / 1 件**。
// 旧版「反向支配」无条件让超集（2 件/10 步）杀掉子集（1 件/6 步）⇒ 报 10 步/2 件。
// ---------------------------------------------------------------------------
#[test]
fn exact_dp_same_hp_fewer_steps_survives() {
    let si = load_si();
    // 带一只**零损血** cat8（角落、不在最优路线上）：否则这是纯拾取层，闭式会先接管。
    let mut ps = s0_spec(500);
    ps.atk = 900;
    let inp = synth_rect(
        &si,
        7,
        5,
        (6, 1),
        vec![
            EntSpec { ty: 32, x: 3, y: 1, param: 0 },
            EntSpec { ty: 29, x: 3, y: 3, param: 0 },
            EntSpec { ty: 41, x: 0, y: 4, param: 0 },
        ],
        ps.clone(),
    );
    let (ex, _hit) = solve_local_exit_toward(&inp, SYNTH_ENTRY, &ps, 0, None, None);
    let st = exact_stat();
    println!(
        "[P9-1b] ok={} out_hp={} exit={:?} steps={} picks={} stat={:?}",
        ex.ok, ex.out_hp, ex.exit, st.steps, st.picks, st
    );
    assert!(st.adopted, "夹具必须由精确 DP 接管，reason={:?}", st.reason);
    assert_eq!(ex.out_hp, 700, "同 hp：500 + 200（ty29 不加血）");
    assert_eq!(st.steps, 6, "同 hp 下必须取最少步：6 步；10 步是绕路捡 ty29 的解");
    assert_eq!(st.picks, 1, "最优解只捡 ty32；ty29 对 (hp, steps) 无贡献");
}

// ---------------------------------------------------------------------------
// 判据 1（f3 打穿）：`out_hp=4027`、`truncated=false`、真引擎回放通过。
//
// 4027 = 3227（链路入场 hp）+ 800（7 件纯加血）+ 0（7 只 cat8 零损血）。
// 若某件不可达，DP 会给真实最大值——本断言就是「必须等于算术上界」的硬约束，
// 松掉它等于放弃这条判据（AGENTS §1「禁调阈值/放宽断言」）。
//
// 【P9.1 更正（对抗复核 F4）】旧版这里断言 `picks == 18`（「18 件拾取物必须全捡」），
// 并把 `steps=106` 当成结论——**两者都是反向支配剪枝杀错解的产物**，不是事实：
// 目标是 `(hp 最大, steps 最小)` 的字典序，同 hp=4027 下存在 **88 步 / 14 件** 的解
// （少捡 4 件不减血：它们对 hp 无贡献，只在 106 步那条更长路线上被顺路捡到）。
// 现在断言改成事实口径：hp 最优 + 步数/件数不超过旧版（旧版是上界不是目标）。
// 剪枝本身是否健全由判据 1b（最小夹具）钉住——只放宽这里会让 F4 回归无人拦。
// ---------------------------------------------------------------------------
#[test]
fn exact_dp_f3_reaches_4027() {
    let si = load_si();
    let inp = floor_input(&si, 3);
    let ps = f3_chain_player();
    // 【P9.2】必须传**真实 17 行商品表**：旧版这条用 `solve_local_exit_toward`（内部 `&[]`），
    // 与 `solver run` 链路口径一致但在「商品表未建模」下才成立（复核 F3/F8）。判据 2g/2h
    // 把「真能买 ⇒ Off」与「机制不可买 ⇒ 放行」分开钉住。
    let (ex, _hit) =
        solve_local_exit_toward_shops(&inp, F3_ENTRY, &ps, F3_FLAGS, None, None, &si.shops);
    let st = exact_stat();
    println!(
        "[P9-1] f3 ok={} trunc={} out_hp={} exit={:?} exact={} reason={:?} stat={:?}",
        ex.ok, ex.truncated, ex.out_hp, ex.exit, st.adopted, st.reason, st
    );
    assert!(st.adopted, "f3 必须由精确 DP 接管，实测 reason={:?}", st.reason);
    assert!(ex.ok, "f3 必须 ok");
    assert!(!ex.truncated, "精确解不允许 truncated");
    assert_eq!(ex.out_hp, F3_MAX_HP, "f3 的精确最优必须是算术上界 4027");
    assert_eq!(ex.exit, Some((11, 11)), "必须走上梯 (11,11)（下梯 (1,11) 只有 3227）");
    // 统计口径 = 终态位图势：14 拾取 + 5 门 + 7 杀（旧版 18/5/7 是更长路线的顺路产物）
    assert_eq!(
        (st.steps, st.picks),
        (88, 14),
        "实测最优元组：hp 4027 下最少 88 步 / 14 件（复核 M1 同值；旧版 106/18 系剪枝杀错解）"
    );
    assert!(
        st.steps <= 106 && st.picks <= 18,
        "步数/件数不得劣于旧版（106/18）：{} / {}",
        st.steps,
        st.picks
    );
    assert_eq!(st.doors, 5, "5 扇 ty1 门必须全开");
    assert_eq!(st.kills, 7, "7 只 cat8 必须全杀");
    assert_eq!(st.replay_steps, st.steps, "回放步数与 DP 预测必须相等");
    assert!(st.route_len > 0);
}

// ---------------------------------------------------------------------------
// 判据 2g（复核 F3/F8 ①）：**真能买的商人格 ⇒ `Off(shop_present)`**
//
// 夹具：7×5 全通房，cat16 商人格 `param=0x201`（`flags=(0x201>>8)=2` ⇒ 引擎开价、
// `idx=1` ⇒ 有货）。买臂会改 hp/钥/金（`scene.rs:2597 param^=512` + 施效），
// 而 DP 的位图无法推导这些维 ⇒ 必须 fail-closed。
// 变异自证：把 `shop_can_buy` 改成恒 false（ty 白名单式糊弄）⇒ 本判据红。
// ---------------------------------------------------------------------------
#[test]
fn exact_dp_off_on_buyable_shop() {
    let si = load_si();
    let mut ps = s0_spec(500);
    ps.atk = 900;
    let inp = synth_rect(
        &si,
        7,
        5,
        (6, 1),
        vec![
            EntSpec { ty: 77, x: 3, y: 1, param: 0x201 },
            EntSpec { ty: 41, x: 0, y: 4, param: 0 },
        ],
        ps.clone(),
    );
    let (_ex, _hit) =
        solve_local_exit_toward_shops(&inp, SYNTH_ENTRY, &ps, 0, None, None, &si.shops);
    let st = exact_stat();
    println!("[P9-2g] stat={st:?}");
    assert!(!st.adopted, "能开价买货的商人格必须 fail-closed");
    assert_eq!(st.reason, "shop_present", "必须是具名理由，不许静默截断");
    assert_eq!(st.reason_ty, 77);
}

// ---------------------------------------------------------------------------
// 判据 2h（复核 F3/F8 ②）：**机制上不可买的商人格 ⇒ 放行（不 Off）**
//
// 夹具同 1b，只是把绕路的 ty29 换成 f3 实层同款的 `ty77 param=0x116`：
// `flags=(0x116>>8)=1` ⇒ 引擎不开购买面（`scene.rs:2541` 买臂门 `flags&2`；
// `controls.rs` 只有 Close 按钮 ⇒ 玩家路径上不存在 `apply_merchant_offer(true)`
// 的入口）⇒ 它卖不出任何 hp ⇒ DP 可以接管。
//
// 变异自证：把判据改回 `if !shops.is_empty()`（旧版）⇒ 本判据红（Off(shop_present)）。
// ---------------------------------------------------------------------------
#[test]
fn exact_dp_adopts_layer_with_unbuyable_shop() {
    let si = load_si();
    let mut ps = s0_spec(500);
    ps.atk = 900;
    let inp = synth_rect(
        &si,
        7,
        5,
        (6, 1),
        vec![
            EntSpec { ty: 32, x: 3, y: 1, param: 0 },
            EntSpec { ty: 77, x: 3, y: 3, param: 0x116 },
            EntSpec { ty: 41, x: 0, y: 4, param: 0 },
        ],
        ps.clone(),
    );
    let (ex, _hit) =
        solve_local_exit_toward_shops(&inp, SYNTH_ENTRY, &ps, 0, None, None, &si.shops);
    let st = exact_stat();
    println!(
        "[P9-2h] ok={} out_hp={} exit={:?} stat={st:?}",
        ex.ok, ex.out_hp, ex.exit
    );
    assert!(st.adopted, "机制不可买的商人格不得按 shop_present 拒，reason={:?}", st.reason);
    assert_eq!(ex.out_hp, 700, "顺路 ty32：500 + 200");
    assert_eq!((st.steps, st.picks), (6, 1), "商人格不产生任何转移 ⇒ 仍是 6 步 1 件");
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// 判据 2a（A1）：有损血怪 ⇒ `Off(monster_damage)`
// ---------------------------------------------------------------------------
#[test]
fn exact_dp_off_on_monster_damage() {
    let si = load_si();
    // 弱打击（atk=50/def=10）撞 ty44 ⇒ 实测 loss=32（A 级：`exact_bump` 真引擎结果）。
    let mut ps = s0_spec(500);
    ps.atk = 50;
    ps.def = 10;
    let inp = synth_with(&si, vec![EntSpec { ty: 44, x: 2, y: 1, param: 0 }], ps.clone());
    let _ex = run_on(&inp, &ps);
    let st = exact_stat();
    println!("[P9-2a] stat={:?}", st);
    assert!(!st.adopted, "有损血怪必须 fail-closed");
    assert_eq!(st.reason, "monster_damage", "必须是 A1 具名理由");
}

// ---------------------------------------------------------------------------
// 判据 2b（A2）：伤害型步臂（陷阱 ty60）可达 ⇒ `Off(step_arm_hazard_reachable)`，
// 与 `pure_pickup` 的 G2 **同源**（共用 `hazard_gate`）。
// ---------------------------------------------------------------------------
#[test]
fn exact_dp_off_on_step_arm_hazard() {
    let si = load_si();
    let ps = s0_spec(500);
    let inp = synth_with(&si, vec![EntSpec { ty: 60, x: 2, y: 1, param: 0 }], ps.clone());
    let _ex = run_on(&inp, &ps);
    let st = exact_stat();
    println!("[P9-2b] stat={:?}", st);
    assert!(!st.adopted, "可达伤害步臂必须 fail-closed");
    assert_eq!(st.reason, "step_arm_hazard_reachable");
    assert_eq!(st.reason_ty, 60);
}

// ---------------------------------------------------------------------------
// 判据 2c（A3）：非一次性门（ty83 param=32，`LINE_END_SKIPPED_SLOTS`）⇒ 透传
// `hp_bound::plan` 的 `Off(retriggerable_gate_slot)`。
// ---------------------------------------------------------------------------
#[test]
fn exact_dp_off_on_retriggerable_gate() {
    let si = load_si();
    let ps = s0_spec(500);
    let inp = synth_with(&si, vec![EntSpec { ty: 83, x: 2, y: 1, param: 32 }], ps.clone());
    let _ex = run_on(&inp, &ps);
    let st = exact_stat();
    println!("[P9-2c] stat={:?}", st);
    assert!(!st.adopted, "可重复触发门槽必须 fail-closed");
    assert_eq!(st.reason, "retriggerable_gate_slot");
    assert_eq!(st.reason_ty, 32);
}

// ---------------------------------------------------------------------------
// 判据 2d（A4a）：门脚本**生成** hp 物（f35 的槽 62：`CES_69@35`）⇒ 透传
// `Off(script_hp_spawn)`。
// ---------------------------------------------------------------------------
#[test]
fn exact_dp_off_on_script_hp_spawn() {
    let si = load_si();
    let ps = s0_spec(500);
    let inp = synth_floor(&si, 35, 5, vec![EntSpec { ty: 83, x: 2, y: 1, param: 62 }], ps.clone());
    let _ex = run_on(&inp, &ps);
    let st = exact_stat();
    println!("[P9-2d] stat={:?}", st);
    assert!(!st.adopted, "脚本生成 hp 物必须 fail-closed");
    assert!(
        matches!(st.reason, "script_hp_spawn" | "script_writes_hp"),
        "必须是脚本侧 hp 通道的具名理由，实测={}",
        st.reason
    );
}

// ---------------------------------------------------------------------------
// 判据 2e（A4b）：击杀后生成实体（f35 杀 ty69 ⇒ post-victory 组生成 ty32×3）⇒
// 透传 `Off(post_kill_hp_spawn)`。
// ---------------------------------------------------------------------------
#[test]
fn exact_dp_off_on_post_kill_spawn() {
    let si = load_si();
    let mut ps = s0_spec(500);
    ps.atk = 900;
    let inp = synth_floor(&si, 35, 5, vec![EntSpec { ty: 69, x: 2, y: 1, param: 0 }], ps.clone());
    let _ex = run_on(&inp, &ps);
    let st = exact_stat();
    println!("[P9-2e] stat={:?}", st);
    assert!(!st.adopted, "击杀生成 hp 物必须 fail-closed");
    assert_eq!(st.reason, "post_kill_hp_spawn");
}

// ---------------------------------------------------------------------------
// 判据 2f（A5）：拾取物数 > `MAX_PICKUPS`（20）⇒ `Off(too_many_pickups)`。
// 这是**性能护栏不是正确性假设**（20 件 = 2^20×格 的状态规模）。
// ---------------------------------------------------------------------------
#[test]
fn exact_dp_off_on_too_many_pickups() {
    let si = load_si();
    let ps = s0_spec(500);
    // 引擎场景网格是 13×13（`scene_setup` 的固定夹具），故用 13×12 全通房间
    // 摆 21 件（13 + 8 两行）——超限判据在**进入 DP 之前**，与几何无关。
    let mut ents: Vec<EntSpec> = (0..13)
        .map(|x| EntSpec { ty: 31, x, y: 0, param: 0 })
        .collect();
    ents.extend((0..8).map(|x| EntSpec { ty: 31, x, y: 1, param: 0 }));
    let inp = synth_room(&si, ents, ps.clone());
    let _ex = run_on(&inp, &ps);
    let st = exact_stat();
    println!("[P9-2f] stat={:?}", st);
    assert!(!st.adopted, "超限必须 fail-closed");
    assert_eq!(st.reason, "too_many_pickups");
    assert_eq!(st.reason_ty, 21);
}

// ---------------------------------------------------------------------------
// 判据 1'（正向小夹具）：两件纯加血 + 无怪无门 ⇒ 采用精确解，hp=100+400。
// 与 f3 的差别只有规模 ⇒ 一旦 DP 的收益推导坏了，这条先红。
// ---------------------------------------------------------------------------
#[test]
fn exact_dp_adopts_simple_layer() {
    let si = load_si();
    // 带一只**零损血** cat8：否则这是纯拾取层，闭式会先接管（本判据要测的是 DP）。
    let mut ps = s0_spec(100);
    ps.atk = 900;
    let inp = synth_with(
        &si,
        vec![
            EntSpec { ty: 32, x: 1, y: 1, param: 0 },
            EntSpec { ty: 32, x: 2, y: 1, param: 0 },
            EntSpec { ty: 41, x: 3, y: 1, param: 0 },
        ],
        ps.clone(),
    );
    let ex = run_on(&inp, &ps);
    let st = exact_stat();
    println!("[P9-1'] stat={:?}", st);
    assert!(st.adopted, "两件拾取的层必须被精确解接管");
    assert_eq!(ex.out_hp, 500, "100 + 2×200");
    assert_eq!(st.picks, 2);
}

