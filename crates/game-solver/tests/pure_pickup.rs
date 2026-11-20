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
//! 【P6 追加】判据 8–14（单调拾取层）：8 `monotone_floor_collects_keys_and_stats`、
//! 9 `non_monotone_pickup_still_fails_closed`、10 `monotone_floor_collects_bag_item`、
//! 11 `pure_pickup_flag_reflects_monotone_hit`、12 `f3_is_excluded_by_g1_g2`、
//! 13 `f3_is_not_closed_form_g2_first`（会跑的反向判据：G2a 通过、第一道响的闸是 G2）、
//! 14 witness 标志复用。
//!
//! 【2026-11-19 P7】判据 13 原来是一条 `#[ignore]` 的「任务书要求式」断言
//! （`f3 out_hp == 4027 && !truncated`）。它把一条**永不运行**的证伪记录算进了
//! test-ratchet 基线，且前提本身是假的。现已移出测试文件（历史记录见
//! `docs/findings/2026-11-19-f3-gate27-walk-only.md`），替换为会跑的反向判据。
//!
//! 前 5 条 + 第 6 条都直接断言「闭式是否被采用」（`Exitsolve::pure_pickup` 与
//! `pure_pickup_stat()`），**没有任何一条**靠放宽阈值或改默认层界来变绿。

use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_solver::ingest::{ingest, SolverInput};
use game_solver::pure_pickup::{
    closed_form_terminal_checks, pure_pickup_stat, replay_route_in_engine, PurePickupStat,
};
use game_solver::solve_local::{
    load_floor, probe_gate, solve_local_exit_toward, solve_local_exit_toward_shops, EntRec, EntSpec,
    ExitKind, ExitSolve, ExitSpec, LevelSpec, PlayerSpec, SolveInput,
};
use game_solver::witness::{run_witness, run_witness_with, WitnessCfg};
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
        // 【P6 改载荷】原用 26 型（黄钥匙）：P6 前它是「非纯加血」⇒ 认证 None；P6 起
        // 钥匙是**单调增益**（`certified_monotone_pickup` 采样出 `yellow:+1`）⇒ 会采用。
        // 「非单调」得换成真有字段**下降**的道具：13 型（Stack，探针背包已有 13 且
        // `ITEM_USES[0] = -1` ⇒ 捡起来 uses 2→1 下降）。断言本身一条未改。
        Poison::Uncertified => ents[0] = EntSpec { ty: 13, x: 1, y: 1, param: 0 },
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
    assert!(!ex.pure_pickup, "会让字段下降的拾取物 ⇒ 必须回退通用搜索");
    assert!(!st.adopted);
    assert_eq!(st.reason, "pickup_not_certified");
    assert_eq!(st.reason_ty, 13);
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
/// 【2026-11-20 更正（P10 出层分层软契约）】f52 行不再是旧值：
/// cmp_exit 在 hp 大者优先之后新增 **gold 多者优先**（软约束层 2，位于 steps 之前——
/// 修订裁定：不做加权和、不把 gold 折算进 hp，但同 hp 下攒钱优先于省步）。
/// f52 同 hp=427 下存在金更多的 124 步路线 ⇒ 从旧 102 步路线切换（血/出口不变）。
/// f51/f53 无同 hp 金差异 ⇒ 仍与旧值一字不差。变更意图见
/// docs/findings/2026-11-20-layer-objective.md。
const OLD_CHAIN: [(i32, bool, bool, i32, i32, Option<[i32; 2]>, usize, u64); 3] = [
    (51, true, false, 300, 247, Some([2, 2]), 69, 0x8e93_f289_6fa6_4309),
    (52, true, false, 247, 427, Some([11, 11]), 124, 0xdd98_dd23_6c41_7cfb),
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
        &si.shops,
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
        &si.shops,
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
        &si.shops,
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

// ---------------------------------------------------------------------------
// 【P6】单调拾取层（B 片）：判据 8–14
//
// 定理（`src/pure_pickup.rs` 模块文档同步）：
//   G1 无怪 ∧ G2 无危险步臂可达 ∧ G2a 入场格安全 ∧
//   G3′ 每个可达拾取物的效果经**真引擎采样**认证为**单调**
//   （hp/atk/def/三钥/背包项 全部不下降，且其余字段零变化）
//   ⇒「把可达拾取物全捡一遍再出层」是 HP 最优，闭式 `out_hp = in_hp + Σ hp_delta`。
// ---------------------------------------------------------------------------

/// f3 的**真实入场 hp**（来源：`solver run --max-floors 20` 链上 f3 的 `in_hp`，
/// 由 f54 闭式 × 通用搜索的路径逐层推出 = 3227；不是本片发明的常量）。
const IN_HP_F3: i32 = 3227;
/// f3 的真实入场格（`landing(f3, 3, went_down=false)`，A 级：`examples/probe_f3` 实测）。
const F3_ENTRY: (i32, i32) = (2, 11);

/// 合成层（5×3，y=1 走廊，入口 (0,1)，出口 (4,1)）= `synth` 的同构推广：
/// 只换拾取物清单与入场玩家，别的（地形/层号/敌表/出口）逐字相同。
fn synth_with(ents: Vec<EntSpec>, player: PlayerSpec) -> SolveInput {
    let si = load_si();
    let f54 = floor_input(&si, 54);
    let lv = f54.level.as_ref().expect("f54 必须有层");
    let (cols, rows) = (5i32, 3i32);
    let mut terrain = vec![false; (cols * rows) as usize];
    for x in 0..cols {
        terrain[(1 * cols + x) as usize] = true;
    }
    let mut all = ents;
    all.push(EntSpec { ty: 7, x: 4, y: 1, param: 1 });
    SolveInput {
        floor: 54,
        difficulty: f54.difficulty,
        level: Some(LevelSpec {
            cols,
            rows,
            terrain,
            start: SYNTH_ENTRY,
            entities: all,
            exits: vec![ExitSpec { x: 4, y: 1, kind: ExitKind::Stair, index: 1 }],
            player: player.clone(),
            enemies: lv.enemies.clone(),
            enemy_base: lv.enemy_base.clone(),
            flags: 0,
        }),
    }
}

fn run_on(inp: &SolveInput, ps: &PlayerSpec) -> (ExitSolve, PurePickupStat) {
    let (ex, _hit) = solve_local_exit_toward(inp, SYNTH_ENTRY, ps, 0, None, None);
    (ex, pure_pickup_stat())
}

// ---------------------------------------------------------------------------
// 判据 8（B）：单调层（钥匙 + 攻 + 血）走闭式，且逐字段等于「全捡」期望
// ---------------------------------------------------------------------------
#[test]
fn monotone_floor_collects_keys_and_stats() {
    let ps = s0_spec(SYNTH_PLAYER_HP);
    // 26=黄钥匙+1、29=攻+1（f54 ⇒ tier1）、32=血+200（f54 ⇒ tier1）
    let inp = synth_with(
        vec![
            EntSpec { ty: 26, x: 1, y: 1, param: 0 },
            EntSpec { ty: 29, x: 2, y: 1, param: 0 },
            EntSpec { ty: 32, x: 3, y: 1, param: 1 },
        ],
        ps.clone(),
    );
    let (ex, st) = run_on(&inp, &ps);
    println!(
        "[B2] ok={} trunc={} out_hp={} atk={} def={} y/b/r={}/{}/{} pure={} reason={} sum_atk={} sum_keys={:?}",
        ex.ok, ex.truncated, ex.out_hp, ex.out_player.atk, ex.out_player.def,
        ex.out_player.yellow, ex.out_player.blue, ex.out_player.red,
        ex.pure_pickup, st.reason, st.sum_atk, st.sum_keys
    );
    assert!(ex.ok && !ex.truncated, "三枚单调拾取 + 可达出口 ⇒ 必须出解");
    // 全捡期望：血 +200、攻 +1、黄钥 +1，其余不动。
    assert_eq!(ex.out_hp, SYNTH_PLAYER_HP + 200, "hp 必须等于 in_hp+Σhp_delta");
    assert_eq!(ex.out_player.atk, 11, "29 型必须真加攻");
    assert_eq!(ex.out_player.def, 10, "def 不得被动过");
    assert_eq!(
        (ex.out_player.yellow, ex.out_player.blue, ex.out_player.red),
        (1, 0, 0),
        "三钥必须逐值等于全捡期望（只有 26 型产黄钥）"
    );
    assert!(ex.pure_pickup, "单调层必须走闭式（B 核心）");
    assert_eq!(st.reason, "adopted");
    assert_eq!((st.sum_atk, st.sum_def, st.sum_keys), (1, 0, (1, 0, 0)));
    assert_eq!(st.sum_delta, 200, "sum_delta 仍是 hp 增量之和");
}

// ---------------------------------------------------------------------------
// 判据 9（B）：非单调拾取物 ⇒ fail-closed（含同层正对照）
// ---------------------------------------------------------------------------
#[test]
fn non_monotone_pickup_still_fails_closed() {
    let ps = s0_spec(SYNTH_PLAYER_HP);
    // 13 型 = Stack，探针背包里**已有** 13（uses=2），而 ITEM_USES[0]=-1
    // ⇒ 真引擎把这枚捡起来会让 uses 2→1（**下降**）⇒ 不是单调增益 ⇒ 必须拒。
    let inp = synth_with(vec![EntSpec { ty: 13, x: 1, y: 1, param: 0 }], ps.clone());
    let (ex, st) = run_on(&inp, &ps);
    println!(
        "[B9] 13 型 ok={} pure={} reason={} ty={} out_hp={}",
        ex.ok, ex.pure_pickup, st.reason, st.reason_ty, ex.out_hp
    );
    assert!(!ex.pure_pickup, "会让字段下降的拾取物 ⇒ 闭式必须 fail-closed");
    assert_eq!(st.reason, "pickup_not_certified", "拒绝理由必须具名");
    assert_eq!(st.reason_ty, 13, "理由必须指出是哪一型");

    // 正对照：同一个合成层，只把 13 换成 26（黄钥，单调）⇒ 必须采用。
    let inp2 = synth_with(vec![EntSpec { ty: 26, x: 1, y: 1, param: 0 }], ps.clone());
    let (ex2, st2) = run_on(&inp2, &ps);
    println!("[B9] 对照 26 型 pure={} reason={} yellow={}", ex2.pure_pickup, st2.reason, ex2.out_player.yellow);
    assert!(ex2.pure_pickup && st2.reason == "adopted", "同层换成单调拾取物必须采用（排除「整层不可用」）");
    assert_eq!(ex2.out_player.yellow, 1);
}

// ---------------------------------------------------------------------------
// 判据 10（B）：背包项纯追加（32 槽真实玩家），且回放逐字段相等
// ---------------------------------------------------------------------------
#[test]
fn monotone_floor_collects_bag_item() {
    // 背包探针：19 型 = Stack（`ITEM_USES[19-13] = 1` ⇒ 捡起来 uses +1），空背包 ⇒ 纯追加。
    let mut ps = s0_spec(SYNTH_PLAYER_HP);
    ps.item_types = vec![0u8; 32];
    ps.item_uses = vec![0u8; 32];
    ps.item_stack_size = 0;
    let inp = synth_with(vec![EntSpec { ty: 19, x: 1, y: 1, param: 0 }], ps.clone());
    let (ex, st) = run_on(&inp, &ps);
    println!(
        "[B10] ok={} pure={} reason={} stack={} t0={} u0={} sum_bag={}",
        ex.ok, ex.pure_pickup, st.reason,
        ex.out_player.item_stack_size, ex.out_player.item_types[0],
        ex.out_player.item_uses[0], st.sum_bag
    );
    assert!(ex.pure_pickup && st.reason == "adopted", "空背包 + Stack 拾取 ⇒ 纯追加 ⇒ 必须采用");
    assert_eq!(ex.out_player.item_stack_size, 1, "栈位必须 +1");
    assert_eq!(ex.out_player.item_types[0], 19, "新槽必须是 19 型");
    assert_eq!(ex.out_player.item_uses[0], 1, "uses 必须等于 ITEM_USES[6]=1（采样值，非硬编码）");
    assert_eq!(st.sum_bag, 1);
}

// ---------------------------------------------------------------------------
// 判据 11（B）：witness 的 pure_pickup 标志反映「单调闭式命中」
// ---------------------------------------------------------------------------
#[test]
fn pure_pickup_flag_reflects_monotone_hit() {
    let ps = s0_spec(SYNTH_PLAYER_HP);
    let inp = synth_with(
        vec![
            EntSpec { ty: 26, x: 1, y: 1, param: 0 },
            EntSpec { ty: 32, x: 2, y: 1, param: 1 },
        ],
        ps.clone(),
    );
    let mut levels = BTreeMap::new();
    levels.insert(54i32, inp);
    let cfg = WitnessCfg { max_floors: 1 };
    // 合成层无商人格 ⇒ 空商品表（口径与生产无关，显式传）
    let w = run_witness(&levels, 54, SYNTH_ENTRY, ps, 0, &[], &cfg);
    let f = &w.floors[0];
    println!(
        "[B11] floors={} floor={} out_hp={} pure={} reason={:?}",
        w.floors.len(), f.floor, f.out_hp, f.pure_pickup, f.pure_pickup_reason
    );
    assert!(f.pure_pickup, "单调层命中闭式 ⇒ witness 标志必须为 true");
    assert_eq!(f.pure_pickup_reason.as_deref(), Some("adopted"));
    assert_eq!(f.out_hp, SYNTH_PLAYER_HP + 200);
    assert_eq!(f.out_player.yellow, 1);
}

// ---------------------------------------------------------------------------
// 判据 12（B）：f3 的真实处境——被 G1/G2/G2a 结构性排除（不是「单调层」）
// ---------------------------------------------------------------------------
#[test]
fn f3_is_excluded_by_g1_g2() {
    let si = load_si();
    let inp = floor_input(&si, 3);
    let ps = s0_spec(IN_HP_F3);
    let (ex, _hit) = solve_local_exit_toward(&inp, F3_ENTRY, &ps, 0, None, None);
    let st = pure_pickup_stat();
    println!(
        "[B12] f3 ok={} trunc={} out_hp={} pure={} reason={} ty={} entry_hazards={}",
        ex.ok, ex.truncated, ex.out_hp, ex.pure_pickup, st.reason, st.reason_ty, st.entry_hazards
    );
    assert!(!ex.pure_pickup, "f3 含 7 个 cat8 怪 + 可达/入场 ty83 ⇒ 闭式必须 fail-closed");
    assert!(
        matches!(
            st.reason,
            "monster_ents" | "step_arm_hazard_reachable" | "step_arm_hazard_on_entry"
        ),
        "拒绝理由必须是闸门具名原因，实测={}",
        st.reason
    );
}

// ---------------------------------------------------------------------------
// 判据 13（B）：f3 **不是**闭式层，且第一道响的闸是 **G2**（不是 G1，也不是 G2a）
// ---------------------------------------------------------------------------
/// 这条是 P6 复核「黄 F8」的修复：原来这里是一条 `#[ignore]` 的
/// `monotone_floor_f3_hits_closed_form`（断言 `f3 out_hp == 4027 && !truncated`）。
/// 它有两个毛病：①**永不运行**却被 test-ratchet 计进基线（占名额、制造虚假覆盖）；
/// ②它的前提是假的（f3 有 7 个 cat8 怪 + 可达 ty83 + 入场格 ty83）。
///
/// 历史记录已移入 `docs/findings/2026-11-19-f3-gate27-walk-only.md`（AGENTS.md §3.6
/// 证伪更正文化：不删历史主张，只是不再让它冒充一条测试）。
///
/// 替换后的判据断言三件**实测事实**（A 级：本仓引擎 + 真实夹具）：
/// 1. **G2a 通过**：入场格 `(2,11)` 上的 `ty83 param=0` 是「门已开」的场景门
///    ⇒ `probe_gate(input, 0) == true` ⇒ 它**不是**第一道闸；
/// 2. **第一道响的闸是 G2**：`reason == "step_arm_hazard_reachable"`、`reason_ty == 83`
///    （`(5,11)` 的 `ty83 param=27` 在纯图内几何可达）；
/// 3. 结论：`pure_pickup == false`（闭式必须 fail-closed）。
///
/// 不在这里断言 `truncated`：那是**层界**的现况（200k 节点），会随剪枝演化，
/// 不是闸门语义的一部分；把它写进判据等于给搜索规模上锁。
#[test]
fn f3_is_not_closed_form_g2_first() {
    let si = load_si();
    let inp = floor_input(&si, 3);
    let ps = s0_spec(IN_HP_F3);
    // ① G2a 通过：入场格的 ty83 是「已开门」场景门（`evaluate(param=0)` 恒过且置 flags[19]）
    assert!(
        probe_gate(&inp, 0),
        "G2a 必须通过：f3 入场格 (2,11) 的 ty83 param=0 probe_gate=true（P6 复核更正项）"
    );
    let (ex, _hit) = solve_local_exit_toward(&inp, F3_ENTRY, &ps, 0, None, None);
    let st = pure_pickup_stat();
    println!(
        "[B13] f3 ok={} trunc={} out_hp={} pure={} reason={} ty={} entry_hazards={}",
        ex.ok, ex.truncated, ex.out_hp, ex.pure_pickup, st.reason, st.reason_ty, st.entry_hazards
    );
    // ② 第一道响的闸是 G2（可达步臂 hazard），不是 G1（怪）
    assert_eq!(
        st.reason, "step_arm_hazard_reachable",
        "f3 的第一道闸是 G2（可达 ty83），不是 G1；实测 reason={} ty={}",
        st.reason, st.reason_ty
    );
    assert_eq!(st.reason_ty, 83, "G2 命中的步臂型必须是 ty83（(5,11) 的 param=27 场景门）");
    assert_eq!(st.entry_hazards, 1, "入场格 hazard 恰好 1 条（(2,11) 的 ty83 param=0）");
    // ③ 结论：不是闭式层
    assert!(!ex.pure_pickup, "f3 有可达步臂 hazard ⇒ 闭式必须 fail-closed");
}

// ---------------------------------------------------------------------------
// 判据 15（A5 / P7 复核 M5）：`terminal_mismatch` 的**调用点**必须真被执行
//
// 复核者把 `try_closed_form` 里 `if let Some(..) = terminal_mismatch(..)` 整条
// 调用点删掉，**全仓测试仍全绿** ⇒ 闭式的「终态三源核对」只有函数级单测、
// 接线零覆盖。本判据用计数器钉死接线：闭式成功路径必须走到该调用点，
// 删调用点 ⇒ 计数不涨 ⇒ 红。
// ---------------------------------------------------------------------------
#[test]
fn closed_form_terminal_mismatch_call_site_is_wired() {
    let ps = s0_spec(SYNTH_PLAYER_HP);
    // 26=黄钥+1、32=血+200（f54 tier1）——单调 ⇒ 闭式必须走到回放终态核对
    let inp = synth_with(
        vec![
            EntSpec { ty: 26, x: 1, y: 1, param: 0 },
            EntSpec { ty: 32, x: 2, y: 1, param: 1 },
        ],
        ps.clone(),
    );
    let before = closed_form_terminal_checks();
    let (ex, st) = run_on(&inp, &ps);
    let after = closed_form_terminal_checks();
    println!("[A5/M5] pure={} reason={} terminal_checks {before} -> {after}", ex.pure_pickup, st.reason);
    assert!(ex.pure_pickup && st.reason == "adopted", "正例必须走闭式：{st:?}");
    assert!(
        after > before,
        "闭式成功路径必须走到 `terminal_mismatch` 调用点（删掉调用点 ⇒ 计数不涨 ⇒ 本判据红）"
    );
}

// ---------------------------------------------------------------------------
// 判据 15（复核 F3/F8 修法）：**商品表必须一路传进层解器**，不许在接力里被丢掉
// ---------------------------------------------------------------------------
/// 旧 bug 链：`bin/solver.rs:462 let (gi, _shops)` 丢弃 ⇒ `run_witness` ⇒
/// `solve_local_exit_toward` ⇒ `solve_layer(..., &[])` 恒传空表 ⇒「打穿 f3」只在
/// 「商品表未建模」口径成立（复核 F3/F8）。
///
/// 本判据用**记录式层解器**看接力实际交下来的切片：长度与首地址都必须等于调用方传入的
/// 真实 17 行表（不是复制、不是空表）。
///
/// 变异自证：把 `witness.rs` 的委托闭包改成 `..., _shops) => solve_local_exit_toward_shops(..., &[])`
/// （旧 bug 同款）⇒ 本判据红。
#[test]
fn witness_forwards_real_shop_table_to_layer_solver() {
    let si = load_si();
    let levels = levels_map(&si);
    assert_eq!(si.shops.len(), 17, "真实商品表必须是 17 行");
    let got: std::cell::RefCell<Vec<(usize, usize)>> = std::cell::RefCell::new(Vec::new());
    let layer = &|input: &SolveInput,
                  entry: (i32, i32),
                  p: &PlayerSpec,
                  f: u128,
                  ents: Option<&[EntRec]>,
                  toward: Option<i32>,
                  shops: &[game_data::defs::ShopGoodDef]| {
        got.borrow_mut().push((shops.len(), shops.as_ptr() as usize));
        solve_local_exit_toward_shops(input, entry, p, f, ents, toward, shops)
    };
    let _ = run_witness_with(
        &levels,
        GLOBAL_START_FLOOR,
        GLOBAL_ENTRY,
        PlayerSpec::from_combat(mk_player()),
        0,
        &si.shops,
        &WitnessCfg { max_floors: 1 },
        layer,
    );
    let got = got.into_inner();
    println!("[shops-forward] got={got:?} want=({}, {:p})", si.shops.len(), si.shops.as_ptr());
    assert_eq!(got.len(), 1, "max_floors=1 ⇒ 层解器恰好被调用一次");
    assert_eq!(got[0].0, si.shops.len(), "层解器收到的商品表长度必须等于真实表");
    assert_eq!(
        got[0].1,
        si.shops.as_ptr() as usize,
        "层解器必须拿到同一份切片（空表/复制都算丢）"
    );
}
