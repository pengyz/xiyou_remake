//! 【P5】纯拾取层闭式解：判据 1–7（TDD 红先行，实现见 `src/pure_pickup.rs`）。
//!
//! 判据出处：任务书 m00001「判据（TDD 先红，至少 6 条）」+「路线必须真走通」硬要求。
//!
//! | # | 测试 | 守的是什么 |
//! |---|---|---|
//! | 1 | `pure_pickup_closed_form_hits_theoretical_max` | f54 实跑：`out_hp == 827+2400 == 3227`、`ok && !truncated` |
//! | 2 | `pure_pickup_route_replays_in_engine` | 闭式产出的 `route` 逐动作喂**真引擎**，终态 hp/位置/实体表与声明一致 |
//! | 3 | `pure_pickup_fails_closed_with_monster` | 含 41 型怪 ⇒ 不采用（含同层无反证：去掉怪就采用） |
//! | 4 | `pure_pickup_fails_closed_with_hazard_cell` | 可达格上放 61（陷阱）⇒ 不采用（同上反证） |
//! | 5 | `pure_pickup_fails_closed_when_pickup_not_certified` | 掺非纯加血拾取物 ⇒ 不采用（同上反证） |
//! | 6 | `pure_pickup_does_not_change_exact_results_on_non_pure_floors` | f1/f52/f53 黑盒差分：结果逐字段等于**改动前**（`cabb1b3`）实测值 |
//! | 7 | `pure_pickup_chain_breaks_the_f54_wall` | 端到端：f51→f54 见证链不再撞界，f54 出层 hp=3227 |
//!
//! 前 5 条 + 第 6 条都直接断言「闭式是否被采用」（`Exitsolve::pure_pickup` 与
//! `pure_pickup_stat()`），**没有任何一条**靠放宽阈值或改默认层界来变绿。

use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_solver::ingest::{ingest, SolverInput};
use game_solver::pure_pickup::{pure_pickup_stat, replay_route_in_engine, PurePickupStat};
use game_solver::solve_local::{
    load_floor, solve_local_exit_toward, EntSpec, ExitKind, ExitSolve, ExitSpec,
    LevelSpec, PlayerSpec, SolveInput,
};
use game_solver::witness::{run_witness, WitnessCfg};
use std::collections::BTreeMap;
use std::path::PathBuf;

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

/// 与 `bin/solver.rs` / `examples/profile_floor.rs` 同构的装载路径。
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

const GLOBAL_ENTRY: (i32, i32) = (1, 11);
const GLOBAL_START_FLOOR: i32 = 51;

/// f54 的入场格 = `landing(f54, 54, went_down=false)` 的落点。
///
/// A 级依据（`examples/profile_floor.rs::landing_replica` 逐字复刻 `solve_global.rs:289`
/// 的 `landing`）：target=54/went_down=false ⇒ `want=8` ⇒ 该层首个 `ty8 && param>>9==0`
/// 的实体在 `(7,10)` ⇒ 落点取「上→下→左→右」首个可行走邻格 ⇒ `(7,9)`。
/// 该格同格还坐着 `ty83 param=64` 的场景门（本判据里由 `entry_hazards == 1` 反证）。
const F54_ENTRY: (i32, i32) = (7, 9);

/// f53 出层态（`docs/findings/2026-11-19-f54-state-profile.md` 实测链）：
/// `out_hp=827`、`flags=0xe337be0000000000`。
///
/// 只搬 hp/flags：本层无怪、无商店、无钥匙、无背包消耗，`atk/def/keys/gold/items`
/// **对闭式与回放断言都不参与**（回放断言是「不变」而非「等于某值」），故注入 S0 值；
/// 端到端真值由判据 7（f51→f54 见证链）覆盖。
const F53_HP: i32 = 827;
const F53_FLAGS: u128 = 0xe337be_00000000;

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
    let scaled = scale_enemy_stats(&base, 1); // difficulty=0 ⇒ ×1
    let tile_walk = nums(include_str!("../../game-core/tests/fixtures/tile_walkability.txt"));
    (scaled, base, tile_walk)
}

fn floor_input(si: &SolverInput, f: i32) -> SolveInput {
    let (scaled, base, tile_walk) = tables();
    load_floor(
        si,
        f as usize,
        GLOBAL_ENTRY,
        PlayerSpec::from_combat(mk_player()),
        scaled,
        base,
        &tile_walk,
    )
    .unwrap_or_else(|e| panic!("load_floor({f}) 失败: {e}"))
}

fn levels_map(si: &SolverInput) -> BTreeMap<i32, SolveInput> {
    let mut m = BTreeMap::new();
    for f in 1..=54 {
        let inp = floor_input(si, f);
        if inp.level.is_some() {
            m.insert(f, inp);
        }
    }
    m
}

// ---------------------------------------------------------------------------
// 合成层夹具（判据 3/4/5）：5×3，y=1 走廊 x=0..=4 可走，其余全墙。
//   (0,1) 入场 / (1,1)(2,1) ty32 长寿丹（f54 ⇒ floor_tier=1 ⇒ +200 认证）/
//   (4,1) ty7 上行梯（出口）⇒ 闭式上界 = 100 + 400 = 500。
// `enemies/enemy_base` 从真实 f54 克隆，避免 `bump_outcome` 越界。
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Poison {
    /// 反证用：无污染，闭式必须采用。
    Free,
    /// 判据 3：层内一只 41 型怪。
    Monster,
    /// 判据 4：可达格上放 61（陷阱）。
    Hazard,
    /// 判据 4 补：61 放在**入场格**（引擎进图即触发 ⇒ 闭式不得成立）。
    HazardOnEntry,
    /// 判据 5：一颗药换成「非纯加血」拾取物。
    Uncertified,
}

const SYNTH_PLAYER_HP: i32 = 100;
const SYNTH_ENTRY: (i32, i32) = (0, 1);

fn synth(poison: Poison) -> SolveInput {
    let si = load_si();
    let f54 = floor_input(&si, 54);
    let lv = f54.level.as_ref().expect("f54 必须有层");
    let (cols, rows) = (5i32, 3i32);
    let mut terrain = vec![false; (cols * rows) as usize];
    for x in 0..cols {
        terrain[(1 * cols + x) as usize] = true;
    }
    let mut ents = vec![
        EntSpec { ty: 32, x: 1, y: 1, param: 1 },
        EntSpec { ty: 32, x: 2, y: 1, param: 1 },
        EntSpec { ty: 7, x: 4, y: 1, param: 1 },
    ];
    match poison {
        Poison::Free => {}
        // 41 型怪放在不可走地形上：只为触发闸门 G1，绝不让通用搜索真去打它。
        Poison::Monster => ents.push(EntSpec { ty: 41, x: 0, y: 0, param: 0 }),
        Poison::Hazard => ents.push(EntSpec { ty: 61, x: 3, y: 1, param: 0 }),
        // 陷阱放在**入场格**：引擎进图链（`scene.rs:3878-3887` dispatchModeEntry case 3）
        // 在出生格也跑 `apply_step_cell_effects` ⇒ 真实玩家一进层就踩雷 ⇒ 闭式不得成立。
        Poison::HazardOnEntry => ents.push(EntSpec { ty: 61, x: SYNTH_ENTRY.0, y: SYNTH_ENTRY.1, param: 0 }),
        // 26 型=钥匙拾取（cat 2 拾取、`pickup_kind_for_type`≠HpUp）⇒ 认证必 None。
        Poison::Uncertified => ents[0] = EntSpec { ty: 26, x: 1, y: 1, param: 0 },
    }
    SolveInput {
        floor: 54,
        difficulty: f54.difficulty,
        level: Some(LevelSpec {
            cols,
            rows,
            terrain,
            start: SYNTH_ENTRY,
            entities: ents,
            exits: vec![ExitSpec { x: 4, y: 1, kind: ExitKind::Stair, index: 1 }],
            player: s0_spec(SYNTH_PLAYER_HP),
            enemies: lv.enemies.clone(),
            enemy_base: lv.enemy_base.clone(),
            flags: 0,
        }),
    }
}

fn run_synth(poison: Poison) -> (ExitSolve, PurePickupStat) {
    let inp = synth(poison);
    let ps = s0_spec(SYNTH_PLAYER_HP);
    let (ex, _hit) = solve_local_exit_toward(&inp, SYNTH_ENTRY, &ps, 0, None, None);
    (ex, pure_pickup_stat())
}

// ---------------------------------------------------------------------------
// 判据 1
// ---------------------------------------------------------------------------
#[test]
fn pure_pickup_closed_form_hits_theoretical_max() {
    let si = load_si();
    let inp = floor_input(&si, 54);
    let ps = s0_spec(F53_HP);
    let t0 = std::time::Instant::now();
    let (ex, _hit) = solve_local_exit_toward(&inp, F54_ENTRY, &ps, F53_FLAGS, None, Some(1));
    let elapsed = t0.elapsed().as_secs_f64();
    let st = pure_pickup_stat();
    println!(
        "f54: out_hp={} ok={} truncated={} exit={:?} route={} pure_pickup={} \
         stat{{reason={}, pickups={}, sum_delta={}, entry_hazards={}, orders_tried={}, \
         replay_steps={}, hp_expected={}, hp_replayed={}}} elapsed={elapsed:.3}s",
        ex.out_hp,
        ex.ok,
        ex.truncated,
        ex.exit,
        ex.route.len(),
        ex.pure_pickup,
        st.reason,
        st.pickups,
        st.sum_delta,
        st.entry_hazards,
        st.orders_tried,
        st.replay_steps,
        st.hp_expected,
        st.hp_replayed,
    );

    assert_eq!(st.reason, "adopted", "闭式必须被采用，实际 reason={}", st.reason);
    assert!(st.adopted && ex.pure_pickup, "纯拾取标记必须为真");
    assert_eq!(st.pickups, 18, "f54 的 18 颗药必须全收（8×ty31 + 10×ty32）");
    assert_eq!(st.sum_delta, 2400, "Σdelta = 8×50 + 10×200");
    assert_eq!(st.entry_hazards, 1, "f54 的 ty83 场景门与入场格同格");
    assert_eq!(st.hp_expected, 3227);
    assert_eq!(st.hp_replayed, 3227, "真引擎回放出的 hp 必须等于上界");
    assert_eq!(ex.out_hp, 3227, "827 + 2400 = 3227（理论最优）");
    assert!(ex.ok, "闭式短路必须给 ok");
    assert!(!ex.truncated, "闭式不走层界，不可能 truncated");
    assert_eq!(ex.exit, Some((7, 1)), "纯路线唯一可用出口=(7,1) 上行梯");
    assert_eq!(
        ex.route.len() as i32,
        st.replay_steps + 1,
        "route 长度 = 回放步数 + 1（首 token 是入场格）"
    );
}

// ---------------------------------------------------------------------------
// 判据 2：路线必须真走通（独立回放，不信任闭式自己的断言）
// ---------------------------------------------------------------------------
#[test]
fn pure_pickup_route_replays_in_engine() {
    let si = load_si();
    let inp = floor_input(&si, 54);
    let ps = s0_spec(F53_HP);
    let (ex, _hit) = solve_local_exit_toward(&inp, F54_ENTRY, &ps, F53_FLAGS, None, Some(1));
    assert!(ex.pure_pickup, "本判据前提：闭式被采用（否则 route 是通用搜索的，另算）");
    let exit = ex.exit.expect("闭式必须有出口");

    // ① 完整回放：逐 token 喂真引擎（`try_step` + 读回），终态必须与声明一致。
    let rp = replay_route_in_engine(&inp, F54_ENTRY, &ps, F53_FLAGS, None, &ex.route)
        .expect("闭式路线必须在真引擎里跑通");
    println!(
        "replay: hp={} pos={:?} steps={} ents={} (sprite_full={})",
        rp.player.hp,
        rp.pos,
        rp.steps,
        rp.ents.len(),
        inp.level.as_ref().unwrap().entities.len()
    );
    assert_eq!(rp.player.hp, ex.out_hp, "回放 hp 必须等于声明 out_hp");
    assert_eq!(rp.player.hp, 3227);
    assert_eq!(rp.pos, exit, "回放终态位置必须等于声明出口");
    assert_eq!(rp.steps, ex.route.len() as i32 - 1);
    assert_eq!(
        rp.ents.len(),
        inp.level.as_ref().unwrap().entities.len() - 18,
        "18 颗药必须真的从引擎实体表消失（37−18=19）"
    );
    // 非拾取实体（楼梯/门/墙/场景门）一个都不能少：ty83 与两条梯子仍在。
    for ty in [6i32, 7, 8, 83] {
        assert!(
            rp.ents.iter().any(|e| e.ty == ty),
            "回放后 ty{ty} 实体必须仍在（拾取不应改动其它实体）"
        );
    }

    // ② 反向对照：同一回放对它自己的 route 是敏感的——只走前 20 步（路上还有药）就该掉血，
    //    且不该宣称到达出口。若回放是假的（直接抄闭式的 hp），这条必红。
    let short = &ex.route[..20];
    let rp_short = replay_route_in_engine(&inp, F54_ENTRY, &ps, F53_FLAGS, None, short)
        .expect("短路线本身也必须是合法前缀");
    println!("short route: hp={} pos={:?}", rp_short.player.hp, rp_short.pos);
    assert!(
        rp_short.player.hp < 3227,
        "只走 20 步必须掉血（否则回放根本没在跑真引擎）：hp={}",
        rp_short.player.hp
    );
    assert_ne!(rp_short.pos, exit, "只走 20 步不得宣称到达出口");
    // ③ 伪造 route：把某个中间 token 换成入场格自己 ⇒ 位置断言必须抓到最后没到出口。
    let mut bogus = ex.route.clone();
    let n = bogus.len();
    bogus[n - 1] = format!("{},{}", F54_ENTRY.0, F54_ENTRY.1);
    match replay_route_in_engine(&inp, F54_ENTRY, &ps, F53_FLAGS, None, &bogus) {
        None => {}
        Some(r) => assert_ne!(r.pos, exit, "改掉末 token 后终态不得仍宣称到达出口"),
    }
}

// ---------------------------------------------------------------------------
// 判据 3/4/5：fail-closed（每条都带「去掉污染源就采用」的反证）
// ---------------------------------------------------------------------------
#[test]
fn pure_pickup_fails_closed_with_monster() {
    let (ex_free, st_free) = run_synth(Poison::Free);
    assert_eq!(st_free.reason, "adopted", "反证：无污染时必须采用");
    assert!(ex_free.pure_pickup);
    assert_eq!(ex_free.out_hp, 500, "100 + 2×200");

    let (ex, st) = run_synth(Poison::Monster);
    println!("monster poison: reason={} reason_ty={} pure_pickup={}", st.reason, st.reason_ty, ex.pure_pickup);
    assert!(!ex.pure_pickup, "含 41 型怪 ⇒ 必须回退通用搜索");
    assert!(!st.adopted);
    assert_eq!(st.reason, "monster_ents");
    assert_eq!(st.reason_ty, 41);
}

#[test]
fn pure_pickup_fails_closed_with_hazard_cell() {
    let (ex_free, st_free) = run_synth(Poison::Free);
    assert_eq!(st_free.reason, "adopted", "反证：无陷阱时必须采用");
    assert_eq!(ex_free.out_hp, 500);

    let (ex, st) = run_synth(Poison::Hazard);
    println!("hazard poison: reason={} reason_ty={} pure_pickup={}", st.reason, st.reason_ty, ex.pure_pickup);
    assert!(!ex.pure_pickup, "可达格上的 61 陷阱 ⇒ 必须回退通用搜索");
    assert!(!st.adopted);
    assert_eq!(st.reason, "step_arm_hazard_reachable");
    assert_eq!(st.reason_ty, 61);

    // 陷阱挪到入场格：G2 的「入场格例外」只放行**门已开的场景门**(ty83)，其余仍失败。
    let (ex2, st2) = run_synth(Poison::HazardOnEntry);
    println!("entry hazard: reason={} reason_ty={} pure_pickup={}", st2.reason, st2.reason_ty, ex2.pure_pickup);
    assert!(!ex2.pure_pickup, "入场格上的 61 陷阱 ⇒ 进层即触发 ⇒ 必须回退");
    assert_eq!(st2.reason, "step_arm_hazard_on_entry");
    assert_eq!(st2.reason_ty, 61);
}

#[test]
fn pure_pickup_fails_closed_when_pickup_not_certified() {
    let (ex_free, st_free) = run_synth(Poison::Free);
    assert_eq!(st_free.reason, "adopted", "反证：两颗药全是纯加血时必须采用");
    assert_eq!(st_free.pickups, 2);
    assert_eq!(ex_free.out_hp, 500);

    let (ex, st) = run_synth(Poison::Uncertified);
    println!(
        "uncertified poison: reason={} reason_ty={} pure_pickup={}",
        st.reason, st.reason_ty, ex.pure_pickup
    );
    assert!(!ex.pure_pickup, "非纯加血拾取物 ⇒ 必须回退通用搜索");
    assert!(!st.adopted);
    assert_eq!(st.reason, "pickup_not_certified");
    assert_eq!(st.reason_ty, 26);
    // 闸门必须发生在「算上界」之前：不许出现「采用了但坏血」的中间态。
    assert_eq!(st.hp_expected, 0, "未采用时不许留下上界残值");
    assert_eq!(st.hp_replayed, 0);
}

// ---------------------------------------------------------------------------
// 判据 6：非纯层黑盒差分（结果必须逐字段等于改动前实测值）
// ---------------------------------------------------------------------------
/// 改动前（HEAD=`cabb1b3`，即本片改动**之前**）在同一装载路径下的实测值：
/// `(floor, ok, truncated, in_hp, out_hp, exit, waypoints, route_fnv)`
///
/// 来源=旧快照 worktree（`git worktree add /tmp/dsp-pure-old cabb1b3`）跑
/// `examples/dump_chain.rs`（同一 `run_witness(51, max_floors=3)`），输出见
/// `docs/findings/2026-11-19-pure-pickup-closed-form.md` §「差分」。
/// 三层都**不是**纯拾取层（f52/f53 有怪、f1 有怪）⇒ 闭式 fail-closed ⇒ 结果必须一字不差。
const OLD_CHAIN: [(i32, bool, bool, i32, i32, Option<[i32; 2]>, usize, u64); 3] = [
    (51, true, false, 300, 247, Some([2, 2]), 69, 0x8e93_f289_6fa6_4309),
    (52, true, false, 247, 427, Some([11, 11]), 102, 0x59a3_c230_a9dc_d5c6),
    (53, true, false, 427, 827, Some([6, 1]), 25, 0xfacb_03ba_5dae_df4e),
];

fn route_fnv(route: &[String]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for t in route {
        for b in t.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h ^= b'|' as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[test]
fn pure_pickup_does_not_change_exact_results_on_non_pure_floors() {
    let si = load_si();
    let levels = levels_map(&si);
    let w = run_witness(
        &levels,
        GLOBAL_START_FLOOR,
        GLOBAL_ENTRY,
        PlayerSpec::from_combat(mk_player()),
        0,
        &WitnessCfg { max_floors: 3 },
    );
    assert_eq!(w.floors.len(), 3, "f51..f53 必须全在链上");
    for (fl, want) in w.floors.iter().zip(OLD_CHAIN.iter()) {
        let got = (
            fl.floor,
            fl.ok,
            fl.truncated,
            fl.in_hp,
            fl.out_hp,
            fl.exit,
            fl.route.len(),
            route_fnv(&fl.route),
        );
        println!("diff f{}: got={:?} want={:?}", fl.floor, got, want);
        assert_eq!(got, *want, "f{} 非纯层的结果必须与改动前逐字段相同", fl.floor);
    }
    // 并且：三层闭式一律 fail-closed（理由具名，不是靠"结果碰巧相同"）。
    for f in [1i32, 52, 53] {
        let inp = floor_input(&si, f);
        let lv = inp.level.as_ref().unwrap();
        let ps = PlayerSpec::from_combat(mk_player());
        let (ex, _hit) = solve_local_exit_toward(&inp, lv.start, &ps, 0, None, None);
        let st = pure_pickup_stat();
        println!("gate f{f}: reason={} ty={} pure_pickup={}", st.reason, st.reason_ty, ex.pure_pickup);
        assert!(!ex.pure_pickup, "f{f} 不是纯拾取层，闭式不得采用");
        // 拒绝理由具名且与实测一致（不是"碰巧结果相同"）：
        // f1 层内有怪(41)、f52 场景门(83) 落在几何可达格、f53 层内有怪(51)。
        let want = match f {
            1 => ("monster_ents", 41),
            52 => ("step_arm_hazard_reachable", 83),
            _ => ("monster_ents", 51),
        };
        assert_eq!((st.reason, st.reason_ty), want, "f{f} 的拒绝理由");

    }
}

// ---------------------------------------------------------------------------
// 判据 7：端到端——f51→f54 见证链不再撞界
// ---------------------------------------------------------------------------
#[test]
fn pure_pickup_chain_breaks_the_f54_wall() {
    let si = load_si();
    let levels = levels_map(&si);
    let t0 = std::time::Instant::now();
    let w = run_witness(
        &levels,
        GLOBAL_START_FLOOR,
        GLOBAL_ENTRY,
        PlayerSpec::from_combat(mk_player()),
        0,
        &WitnessCfg { max_floors: 4 },
    );
    let elapsed = t0.elapsed().as_secs_f64();
    println!("chain: status={} floors={} elapsed={elapsed:.3}s", w.status, w.floors.len());
    for fl in &w.floors {
        println!(
            "  f{} ok={} truncated={} in_hp={} out_hp={} waypoints={} exit={:?} kind={}",
            fl.floor, fl.ok, fl.truncated, fl.in_hp, fl.out_hp, fl.route.len(), fl.exit, fl.kind
        );
    }
    assert_eq!(w.floors.len(), 4, "f51..f54 四层都要在链上（旧行为在第 3 层就 blocked）");
    assert_eq!(w.floors[3].floor, 54);
    assert_eq!(w.floors[3].out_hp, 3227, "f54 出层血 = 827 + 2400");
    assert!(w.floors[3].ok, "f54 必须 ok");
    assert!(!w.floors[3].truncated, "f54 必须不再撞界");
    assert_eq!(w.floors[3].exit, Some([7, 1]));
}

// ---------------------------------------------------------------------------
// 判据 A1（F7 修复）：`witness` 的 floor JSON 必须能区分「闭式命中」与「通用搜索」。
//
// 红（修前实测）：`WitnessFloor` 根本没有 `pure_pickup` 字段 ⇒ 编译期 E0609；
// 即便只看 `ExitSolve`，下游也只能从 `pure_pickup_stat()` 的**线程局部**猜，而
// `solver run` 消费的是 JSON ⇒ 不可区分（复核 F7 报告原文）。
// ---------------------------------------------------------------------------
#[test]
fn witness_floor_records_pure_pickup_flag() {
    let si = load_si();
    let levels = levels_map(&si);

    // ① f54 段：闭式命中 ⇒ flag=true、reason 是命中标签。
    let w = run_witness(
        &levels,
        GLOBAL_START_FLOOR,
        GLOBAL_ENTRY,
        PlayerSpec::from_combat(mk_player()),
        0,
        &WitnessCfg { max_floors: 4 },
    );
    assert_eq!(w.floors.len(), 4, "f51..f54 四层都要在链上");
    let f54 = w.floors.iter().find(|f| f.floor == 54).expect("f54 在链上");
    println!(
        "[A1] f54 pure_pickup={} reason={:?}",
        f54.pure_pickup, f54.pure_pickup_reason
    );
    assert!(f54.pure_pickup, "f54 出层解来自闭式 ⇒ 标记必须为 true（F7 核心）");
    assert_eq!(f54.pure_pickup_reason.as_deref(), Some("adopted"));

    // ② 同一条链上的 f51：通用搜索 ⇒ flag=false + 具名拒绝原因（带触发实体型）。
    //    为什么不用 f1：入口 (1,11) 下 f1 的闭式判定发生在 G2a（入场格 ty83 场景门），
    //    闸门执行序 G2a 在 G1 之前 ⇒ 理由是 `step_arm_hazard_on_entry/83`。f51 才是
    //    干净落在 G1 的样本（`monster_ents/41`），且它与 f54 出自**同一次**链跑。
    let f51 = w.floors.iter().find(|f| f.floor == 51).expect("f51 在链上");
    println!(
        "[A1] f51 pure_pickup={} reason={:?}",
        f51.pure_pickup, f51.pure_pickup_reason
    );
    assert!(!f51.pure_pickup, "f51 ⇒ 闭式不得命中");
    // 实测（隔离进程 `examples/probe_f3` 与链上**两条路径同值**）：
    // f51/f52/f53 的拒绝原因都是 `step_arm_hazard_reachable/83`——三层都有**可达的
    // ty83 场景门**，而闸门执行序是 G2a→G2→G1 ⇒ 只要 G2 先响，G1 的 `monster_ents`
    // 就轮不到。任务书预期的 `monster_ents/41` 在本代码上不成立（见本片报告的
    // 「具名理由与预期不符」一条）：这是**既有行为**，本片不改闸门语义。
    assert_eq!(
        f51.pure_pickup_reason.as_deref(),
        Some("step_arm_hazard_reachable/83"),
        "回退必须带具名原因+触发实体型，禁静默"
    );
}
