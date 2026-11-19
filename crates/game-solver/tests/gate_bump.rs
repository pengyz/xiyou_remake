//! 【P7】撞门臂（`interact_with_cell` 的 bump 面，`from_walk=false`）：判据 1–4。
//!
//! | # | 测试 | 守的是什么 |
//! |---|---|---|
//! | 1 | `gate_bump_runs_bump_only_gate_without_moving` | 撞-only 门（`!from_walk` 臂）能跑脚本、玩家**不移动**、脚本效果真跑进实体表 |
//! | 2 | `gate_bump_not_generated_for_walk_triggerable_gate` | 走能触发的门（`from_walk` 臂）**不建**撞门臂（防状态爆炸；f51/f54 不变量之源） |
//! | 3 | `no_bump_without_gate` | 邻格无 `ty83` ⇒ 零撞门后继、零计数（无门层输出不变） |
//! | 4 | `f3_gate27_is_walk_only_no_bump_arm` | 真层 f3：`(4,11)→(5,11)` 撞不出后继（门 27 只认 `from_walk`） |
//!
//! 为什么撞门只对「撞才触发」的门建臂：走能触发的门走上目标格本身就跑同一个脚本并
//! 停在原地（`interact_with_cell` 命中即 `allowed=false`，scene.rs:3379-3393）
//! ⇒ 撞门后继被走步臂支配。反向隐患正是本片硬约束「f51..f54 逐字段不变」。
use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_solver::ingest::{ingest, SolverInput};
use game_solver::solve_local::{
    bump_memo_clear, bump_memo_len, gate_bump_stat, gate_bump_stats, load_floor, probe_cell_class,
    probe_gate, probe_gate_bump, probe_gate_verdict, solve_local, EntSpec, ExitKind, ExitSpec,
    LevelSpec, PlayerSpec, SolveInput,
};
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

/// 与 `tests/pure_pickup.rs::load_si` 同构的装载路径（同一份夹具/表）。
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

/// 合成层：13×13 全可走；`(0,1)` 入场、`(2,1)` ty7 上行梯（出口）。
/// `gate` = 放在 `(1,1)` 的 ty83 槽（`None` ⇒ 无门）；`ent72_at` = 给脚本当靶子的 ty72。
///
/// 脚本槽内容（`game-core/tests/fixtures/level_scripts.txt`，行号 = 槽号+1）：
/// - 槽 1 = `TAK_22_22 ROS_4_3 TAK_23_32 MOV_72_3_7_1_8`（`flag(0) && !from_walk`）
/// - 槽 2 = `TAK_33_36 MOV_72_1_8_1_10 DES_72_1_10`（`flag(1) && !from_walk`）
/// - 槽 24 = `TAK_40_40`（`!from_walk && hp>0`）
/// - 槽 27 = `TAK_113_117 TAK_133_135 DES_84_6_11 GIN_0_13`（`from_walk && flag(0)`，走-only）
fn synth(gate: Option<i32>, ent72_at: Option<(i32, i32)>, flags: u128) -> SolveInput {
    let (scaled, base, _tile_walk) = tables();
    let n = 13i32;
    let mut entities = vec![
        EntSpec { ty: 7, x: 2, y: 1, param: 4 },
    ];
    if let Some(p) = gate {
        entities.push(EntSpec { ty: 83, x: 1, y: 1, param: p });
    }
    if let Some((x, y)) = ent72_at {
        entities.push(EntSpec { ty: 72, x, y, param: 0 });
    }
    let level = LevelSpec {
        cols: n,
        rows: n,
        terrain: vec![true; (n * n) as usize],
        start: (0, 1),
        entities,
        exits: vec![ExitSpec { x: 2, y: 1, kind: ExitKind::Stair, index: 1 }],
        player: PlayerSpec::from_combat(mk_player()),
        enemies: scaled,
        enemy_base: base,
        flags,
    };
    SolveInput { floor: 1, difficulty: 0, level: Some(level) }
}

fn floor_input(si: &SolverInput, f: i32) -> SolveInput {
    let (scaled, base, tile_walk) = tables();
    load_floor(
        si,
        f as usize,
        (1, 11),
        PlayerSpec::from_combat(mk_player()),
        scaled,
        base,
        &tile_walk,
    )
    .unwrap_or_else(|e| panic!("load_floor({f}) 失败: {e}"))
}

// ---------------------------------------------------------------- 判据

/// 判据 1：撞-only 门（槽 2 = `flag(1) && !from_walk`）能触发 ⇒ 脚本真跑
/// （`MOV_72_1_8_1_10` + `DES_72_1_10` 把 (1,8) 的实体 72 送走并移除），
/// **玩家留在入场格**、不计步。
#[test]
fn gate_bump_runs_bump_only_gate_without_moving() {
    // flags bit1 ⇒ 槽 2 的 `v.flag(1)` 满足；bump 面 `!from_walk` 满足 ⇒ 可触发
    let input = synth(Some(2), Some((1, 8)), 0b10);
    let ps = input.level.as_ref().unwrap().player.clone();

    // 反证腿：同一门**走上去不触发**（`from_walk` 不满足）——这正是「撞才有意义」
    let before_walk = probe_gate_bump(&input, (0, 1), &ps, 0b10, None, (1, 1));
    assert!(before_walk.is_some(), "bump 面必须能触发槽 2");

    let obs = before_walk.unwrap();
    assert_eq!(obs.pos, (0, 1), "撞门不移动玩家（bump 面 allowed=false）");
    assert!(
        !obs.ents.iter().any(|e| e.ty == 72),
        "脚本必须真跑：MOV_72_1_8_1_10 + DES_72_1_10 ⇒ 实体 72 应被送走并移除，实得 {:?}",
        obs.ents
    );

    // 臂确实进了**搜索**（不是只有探针能调）：同层跑一遍，计数必须 ≥1。
    let r = solve_local(&input);
    assert!(r.reachable, "同层照常可解：{r:?}");
    let (gen, hit) = gate_bump_stats();
    assert!(
        hit >= 1,
        "撞门臂必须真进搜索（生成 {gen} / 命中 {hit}）——否则 expand 没接上"
    );
}

/// 判据 2：走能触发的门（槽 27 = `from_walk && flag(0)`）**不建**撞门臂。
/// 这是 f51/f54「逐字段不变」的机制来源（那两层的 ty83 全是 walk 可触发）。
#[test]
fn gate_bump_not_generated_for_walk_triggerable_gate() {
    let input = synth(Some(27), None, 0b1);
    let ps = input.level.as_ref().unwrap().player.clone();

    // 走面确实能开（否则本判据退化成「门坏了」）
    assert!(probe_gate(&input, 27), "槽 27 在 flag(0) 置位时走上格必须触发");

    assert!(
        probe_gate_bump(&input, (0, 1), &ps, 0b1, None, (1, 1)).is_none(),
        "走能触发的门不得生成撞门后继（撞门被走步臂支配，只会状态爆炸）"
    );

    // 更严的一刀：**连尝试都不做**。`bump_only_gate_params` 预筛在先，否则每个
    // 「邻格有走-only 门」的节点都要白跑一次 `build_scene`（C5b 实测的主要开销）。
    let r = solve_local(&input);
    assert!(r.reachable, "同层照常可解：{r:?}");
    assert_eq!(
        gate_bump_stats(),
        (0, 0),
        "走-only 门连撞门候选都不许进（预筛失效 ⇒ 逐节点白建场景）"
    );
}

/// 判据 3：邻格无 `ty83` ⇒ 零撞门后继、零计数。用真实搜索跑一遍整层取证
/// （不是只看探针）：无门层的输出必须与撞门臂无关。
#[test]
fn no_bump_without_gate() {
    let input = synth(None, Some((1, 8)), 0b10);
    let ps = input.level.as_ref().unwrap().player.clone();
    assert!(
        probe_gate_bump(&input, (0, 1), &ps, 0b10, None, (1, 1)).is_none(),
        "无门格不得产生撞门后继"
    );

    let r = solve_local(&input);
    assert!(r.reachable, "无门层照常可解：{r:?}");
    assert_eq!(gate_bump_stats(), (0, 0), "无门层撞门臂计数必须为 0");
    assert!(
        !r.route.iter().any(|s| s.starts_with("gatebump")),
        "无门层路线里不得出现撞门航点：{:?}",
        r.route
    );
}

/// 判据 4：真层 f3 的 `(5,11)` 门 27 只认 `from_walk`（gate.rs:147）⇒ 从 `(4,11)`
/// 撞它**不产生**后继。这同时是「口袋没被封」的机制前提：走上去（`from_walk=true`）
/// 才是它的唯一合法触发方式，而 flags[0]=0 时它连走都不开 ⇒ 格可穿。
#[test]
fn f3_gate27_is_walk_only_no_bump_arm() {
    let si = load_si();
    let f3 = floor_input(&si, 3);
    let lv = f3.level.as_ref().expect("f3 必须有层");
    let ps = PlayerSpec::from_combat(mk_player());

    assert!(!probe_gate(&f3, 27), "flags[0]=0 ⇒ 门 27 走也不开（gate.rs:147）");
    assert!(
        probe_gate_bump(&f3, (4, 11), &ps, 0, None, (5, 11)).is_none(),
        "门 27 是走-only 门，撞门面恒 false ⇒ 不得生成撞门后继"
    );
    // 入口门 param=0：恒过（gate.rs:76），走面/撞面都可，但它在入场格、不属于撞门目标
    assert!(probe_gate(&f3, 0), "param=0 恒过（G2a 通过的依据）");
    assert!(lv.entities.iter().any(|e| e.ty == 83 && e.x == 5 && e.y == 11 && e.param == 27));
}

/// 判据 5（真层正例）：f35 有 `ty83 param=24` 的门（`!from_walk && hp>0`——
/// 撞-only 臂）。撞门臂在**真实层数据**上必须真触发并读回（非纯合成夹具）。
#[test]
fn gate_bump_fires_on_real_floor35() {
    let si = load_si();
    let f35 = floor_input(&si, 35);
    let lv = f35.level.as_ref().expect("f35 必须有层");
    let g = lv
        .entities
        .iter()
        .find(|e| e.ty == 83 && e.param == 24)
        .expect("f35 应有 param=24 的 ty83 门（P6 扫描表）");
    // 门格 (g.x,g.y)；从相邻可走格撞它
    let ps = PlayerSpec::from_combat(mk_player());
    let adj = [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .map(|(dx, dy)| (g.x + dx, g.y + dy))
        .find(|&(x, y)| {
            probe_gate_bump(&f35, (x, y), &ps, 0, None, (g.x, g.y)).is_some()
        });
    let entry = adj.unwrap_or_else(|| {
        panic!("f35 的 param=24 门 @({},{}) 相邻格撞不动（bump 臂在真层失效）", g.x, g.y)
    });
    let obs = probe_gate_bump(&f35, entry, &ps, 0, None, (g.x, g.y)).unwrap();
    assert_eq!(obs.pos, entry, "撞门不移动玩家");
    assert_eq!(obs.hp, 300, "槽 24 = TAK_40_40 纯演出，不掉血（实得 {}）", obs.hp);
}

// ---------------------------------------------------------------- P7 复核 A 片判据

/// 判据 6（A2/F1）：三处「静默丢弃」必须**可观测**。
///
/// P7 复核黄 F1：`gate_bump` 对 `Pending`/`Stalled`/`pos 变化`/`判门未开` 四处
/// 直接 `continue`/`return None`，全仓无计数 ⇒ 「断言 pos 未变」实为**守卫跳过**，
/// 既不可测也不可观测。
///
/// 本判据钉死：① 统计面能看见候选数与真命中；② 今天的真层脚本**确实没有**走位
/// （`rejected_pos_moved=0`——如果将来某层 bump-only 脚本含 `MOV_0_*` 传送，这条会红，
/// 逼我们回来重新裁定「撞门不动」这个前提）。
#[test]
fn bump_stat_counts_every_rejection() {
    let input = synth(Some(2), Some((1, 8)), 0b10);
    let r = solve_local(&input);
    assert!(r.reachable, "同层照常可解：{r:?}");

    let st = gate_bump_stat();
    assert!(st.candidates >= 1, "撞门候选必须可观测（实得 {st:?}）");
    assert!(st.script_hits >= 1, "真跑引擎命中必须可观测（实得 {st:?}）");
    assert_eq!(st.rejected_pos_moved, 0, "槽 2 脚本不得移动玩家，实得 {st:?}");
    assert_eq!(st.rejected_stalled, 0, "槽 2 不得停摆，实得 {st:?}");
    assert_eq!(st.rejected_pending, 0, "槽 2 不换层，实得 {st:?}");
    assert_eq!(st.rejected_not_fired, 0, "预筛说能开 ⇒ 引擎也必须开，实得 {st:?}");
    // 旧二元投影必须与新统计同源（禁两套真相）
    assert_eq!(gate_bump_stats(), (st.candidates, st.script_hits));

    // 真层 f35（唯一 bump-only 层）同样不得出现「脚本移动玩家」
    let si = load_si();
    let f35 = floor_input(&si, 35);
    let lv = f35.level.as_ref().expect("f35 层");
    let g = lv.entities.iter().find(|e| e.ty == 83 && e.param == 24).expect("f35 p24");
    let ps = PlayerSpec::from_combat(mk_player());
    let _ = probe_gate_bump(&f35, (g.x + 1, g.y), &ps, 0, None, (g.x, g.y));
    assert_eq!(
        gate_bump_stat().rejected_pos_moved,
        0,
        "f35 槽 24 = `TAK_101_105` 纯演出，撞完不得移动玩家"
    );
}

/// 判据 7（A3/F3）：「走能开 ⇒ 走臂支配」这条预筛不变量的**反例**与**定义域**。
///
/// P7 复核黄 F3：不变量在**非可走格**上为假 —— f41 `p13@(10,2)`、f48 `p38@(4,1)`
/// 都是 `bump=1 ∧ walk=1`（直通臂，gate.rs:159）且格 `Blocked`：
/// 预筛按「walk 能开」排除撞门臂，而走臂根本进不去 ⇒ **两臂都没有**。
///
/// 今天零进度损失（两条脚本都是 `CES_` 陷阱：13=`CES_6_10_2 CES_60_10_2`、
/// 38=`CES_6_4_1 CES_61_4_1`，只在原地生成怪/火，且门格本身不可达），
/// 但「不变量」因此**只在可走格上成立**。本判据把反例钉成事实，防止后人把
/// 不变量当成全域真命题去推更宽的剪枝。
#[test]
fn both_arm_gate_on_nonwalkable_cell_is_outside_the_dominance_invariant() {
    let si = load_si();
    let ps = PlayerSpec::from_combat(mk_player());
    for (floor, param, cell) in [(41i32, 13i32, (10i32, 2i32)), (48, 38, (4, 1))] {
        let inp = floor_input(&si, floor);
        let lv = inp.level.as_ref().expect("层");
        assert!(
            lv.entities.iter().any(|e| e.ty == 83 && (e.x, e.y) == cell && e.param == param),
            "f{floor} 必须在 {cell:?} 有 ty83 param={param}"
        );
        // ① 两臂都真（直通臂 ⇒ 撞面/走面同真）
        assert!(
            probe_gate_verdict(&inp, &ps, 0, None, param, false),
            "f{floor} p{param} 撞面必须能开（直通臂）"
        );
        assert!(
            probe_gate_verdict(&inp, &ps, 0, None, param, true),
            "f{floor} p{param} 走面必须能开（直通臂）"
        );
        // ② 门格不可走
        assert_eq!(
            probe_cell_class(&inp, None, cell),
            "blocked",
            "f{floor} 的 p{param} 门格必须 Blocked（ty83 = cat1 阻挡）"
        );
        // ③ 现状（特性判据）：预筛排掉撞门臂，走臂也进不去 ⇒ 撞门探针 None。
        //    这是 uninterrupted 的「两臂都没有」——不变量定义域之外，不是待修 bug。
        assert!(
            probe_gate_bump(&inp, (cell.0 + 1, cell.1), &ps, 0, None, cell).is_none(),
            "f{floor} p{param} 被预筛排除（walk=1），当前不生成撞门臂"
        );
    }
    // ④ 反例的「无进度损失」证据：两条脚本只有原地生成类 op（`CES_`），无 hp/钥匙/
    //    层间效果 ⇒ 少这两臂不改变可达集。（逐字取自 level_scripts.txt 槽 13/38。）
    let scripts = include_str!("../../game-core/tests/fixtures/level_scripts.txt");
    let line = |i: usize| scripts.lines().nth(i).unwrap_or("").trim_end_matches('\x1e').trim();
    assert_eq!(line(13), "CES_6_10_2 CES_60_10_2", "槽 13 必须仍是纯 CES 陷阱");
    assert_eq!(line(38), "CES_6_4_1 CES_61_4_1", "槽 38 必须仍是纯 CES 陷阱");
}

/// 判据 8（A4/F4）：`BUMP_MEMO` 键必须含 `pos`（与门格），且每层开搜前清空。
///
/// P7 复核黄 F4：原键 `(floor,param,player,flags,ents)` 不含 `pos`；脚本坐标全绝对
/// ⇒ 今日不可观测，但这是「键不完备」的潜伏（P3 同类）。判据用**条目数**取证：
/// 同一 `(floor,param,player,flags,ents)` 从两个不同相邻格撞同一扇门，
/// 修好前只留 1 条（第二次错误复用），修好后 2 条。
#[test]
fn bump_memo_key_includes_pos_and_is_cleared_per_layer() {
    let si = load_si();
    let f35 = floor_input(&si, 35);
    let lv = f35.level.as_ref().expect("f35 层");
    let g = lv.entities.iter().find(|e| e.ty == 83 && e.param == 24).expect("f35 p24");
    let ps = PlayerSpec::from_combat(mk_player());
    bump_memo_clear();
    let a = (g.x + 1, g.y);
    let b = (g.x - 1, g.y);
    assert!(probe_gate_bump(&f35, a, &ps, 0, None, (g.x, g.y)).is_some(), "从 {a:?} 撞得动");
    assert!(probe_gate_bump(&f35, b, &ps, 0, None, (g.x, g.y)).is_some(), "从 {b:?} 撞得动");
    assert_eq!(
        bump_memo_len(),
        2,
        "不同入口 pos 必须各占一条 memo（键缺 pos ⇒ 错误复用，条数掉到 1）"
    );

    // 清理策略：每次 `run_root_with`（=每层搜索）入口清空 ⇒ 层间不串味、内存有界。
    // 先用手工撞门把 memo 弄脏，再跑一层**无门**搜索：入口清空 ⇒ 结束时 0 条。
    bump_memo_clear();
    assert!(probe_gate_bump(&f35, a, &ps, 0, None, (g.x, g.y)).is_some());
    assert_eq!(bump_memo_len(), 1, "脏场：1 条");
    let clean = synth(None, None, 0);
    let r = solve_local(&clean);
    assert!(r.reachable, "无门层照常可解：{r:?}");
    assert_eq!(bump_memo_len(), 0, "每层开搜前必须清空 BUMP_MEMO（否则跨层无限增长）");
}

/// 判据 9（A5 / 复核 M6）：`StGate::cross_floor_snapshot` **恒 false** 是撞门臂的
/// 健全性契约。变异成 `true` 会让槽 32（`flag(7)=0 ⇒ gate &= !from_walk &&
/// cross_floor_snapshot(23,12)`）凭空变成 bump-only，撞门臂随即伪造后继。
#[test]
fn stgate_cross_floor_snapshot_is_false_by_contract() {
    let input = synth(Some(32), None, 0);
    let ps = input.level.as_ref().expect("层").player.clone();
    assert!(
        !probe_gate_verdict(&input, &ps, 0, None, 32, false),
        "槽 32 撞面必须 false（夹具无跨层快照 ⇒ cross_floor_snapshot=false）"
    );
    assert!(
        !probe_gate_verdict(&input, &ps, 0, None, 32, true),
        "槽 32 走面必须 false（`!from_walk` 不满足）"
    );
    assert!(
        probe_gate_bump(&input, (0, 1), &ps, 0, None, (1, 1)).is_none(),
        "槽 32 不得生成撞门后继（M6 变异：快照→true 会让它变成 bump-only）"
    );
}

/// 判据 10（A5 / 复核 M7）：`StGate::predict_hp_loss_vs` **恒 0**。槽 34 的
/// `gate &= loss < 0 || loss >= player_hp()` 在 `hp=1` 时只有 `loss>=1` 才开；
/// `loss=0` ⇒ 走面 false。变异成 `1` ⇒ 走面翻 true（本判据打红）。
/// 对照腿 `hp=0` 证明判据不是恒假。
#[test]
fn stgate_predict_hp_loss_is_zero_by_contract() {
    let mut input = synth(Some(34), None, 0);
    let ps1 = {
        let lv = input.level.as_mut().expect("层");
        lv.player.hp = 1;
        lv.player.clone()
    };
    assert!(
        !probe_gate_verdict(&input, &ps1, 0, None, 34, true),
        "hp=1 且 StGate.loss=0 ⇒ `0>=1` 不成立 ⇒ 槽 34 走面必须 false"
    );
    let ps0 = {
        let lv = input.level.as_mut().expect("层");
        lv.player.hp = 0;
        lv.player.clone()
    };
    assert!(
        probe_gate_verdict(&input, &ps0, 0, None, 34, true),
        "对照腿：hp=0 ⇒ `0>=0` 成立 ⇒ 走面 true（判据非恒假）"
    );
}
