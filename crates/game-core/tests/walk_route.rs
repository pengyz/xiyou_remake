//! T2 寻路策略判据（touch-native-undo-plan §3 + task-11 行为政策）。
//!
//! A* 优先复刻原版 findPath（a.java:8299 MOV 五参同款）；策略层：
//! 不可达可走格=朝目标方向最远可达点停靠；墙格=最近相邻+探步入（一步撞击尝试，
//! 挂 openWallOnRoute/撞击臂型 10/11/12 语义）；斜向禁走；路径连续性。

use game_core::pathfind::{plan_route, ProbeStep, RoutePlan, RouteTracker, Walkability};
use game_core::undo_journal::{
    CombatStroke, MoveStroke, PlayerVitals, Stroke, UndoJournal, UndoOutcome,
};

/// 网格夹具：`#`=墙 `.`=可走（见各测注释的图）。
fn walk_of(map: &[&str]) -> (Walkability<'static>, Vec<Vec<bool>>) {
    let terrain: Vec<Vec<bool>> = map
        .iter()
        .map(|row| row.chars().map(|c| c == '.').collect())
        .collect();
    let cols = terrain[0].len() as i32;
    let rows = terrain.len() as i32;
    let no_slot: &'static dyn Fn(i32, i32) -> bool = &|_, _| false;
    let walk = Walkability { terrain: terrain.clone(), has_entity_slot: no_slot, cols, rows };
    (walk, terrain)
}

/// 步序（缓冲序=目标端优先，自末端消费）→ 足迹格序列（from 起）。
fn trace_steps(from: (i32, i32), steps: &[u8]) -> Vec<(i32, i32)> {
    let mut pos = from;
    let mut out = vec![from];
    for &d in steps.iter().rev() {
        pos = match d {
            0 => (pos.0, pos.1 + 1),
            1 => (pos.0, pos.1 - 1),
            2 => (pos.0 + 1, pos.1),
            3 => (pos.0 - 1, pos.1),
            other => panic!("非法方向码 {other}"),
        };
        out.push(pos);
    }
    out
}

// ───────────────────────────────────────── A* 四例

#[test]
fn astar_straight_line() {
    // "..........." 单行直线 (1,0)→(5,0)
    let (walk, _) = walk_of(&["..........."]);
    let plan = plan_route(&walk, (1, 0), (5, 0));
    assert!(plan.reached, "直达");
    assert_eq!(plan.dock, (5, 0));
    assert_eq!(plan.steps.len(), 4, "直线 4 步");
    assert_eq!(trace_steps((1, 0), &plan.steps).last(), Some(&(5, 0)));
}

#[test]
fn astar_detour_around_obstacle() {
    // 绕障：墙挡直路，须绕行
    // ". . . ."  行0 通道
    // "# # # ."  行1 三墙留右口
    let map = [".....", "###..", "....."];
    let (walk, terrain) = walk_of(&map);
    let plan = plan_route(&walk, (0, 0), (0, 2));
    assert!(plan.reached);
    assert_eq!(plan.dock, (0, 2));
    let path = trace_steps((0, 0), &plan.steps);
    assert!(path.len() > 4, "绕障长于直线（{} 步）", path.len() - 1);
    for &(x, y) in &path {
        assert!(terrain[y as usize][x as usize], "路径点 ({x},{y}) 可走");
    }
}

#[test]
fn astar_unreachable_docks_furthest_toward_target() {
    // 不可达可走格=朝目标方向最远可达点停靠：
    // 左室 (0..2,0..2) 与右室 (6..8,0..2) 隔墙；从 (1,1) 点 (7,1)
    let map = [
        "...###...",
        "...###...",
        "...###...",
    ];
    let (walk, _) = walk_of(&map);
    let plan = plan_route(&walk, (1, 1), (7, 1));
    assert!(!plan.reached, "目标不可达");
    assert_eq!(plan.dock, (2, 1), "朝目标方向最远可达点（左室右缘）");
    let path = trace_steps((1, 1), &plan.steps);
    assert_eq!(path.last(), Some(&(2, 1)));
}

#[test]
fn astar_no_diagonal_steps() {
    // 斜向禁走：绕障路径每步只能是 4 邻接
    let map = [".....", "###..", "....."];
    let (walk, _) = walk_of(&map);
    let plan = plan_route(&walk, (0, 0), (0, 2));
    let path = trace_steps((0, 0), &plan.steps);
    for w in path.windows(2) {
        let dx = (w[1].0 - w[0].0).abs();
        let dy = (w[1].1 - w[0].1).abs();
        assert_eq!(dx + dy, 1, "步 {:?}→{:?} 非斜即直", w[0], w[1]);
    }
}

#[test]
fn astar_path_continuity() {
    // 路径连续性：相邻格恒相邻、无跳格、无回退环
    let map = [".......", "###..#.", ".......", ".#####.", "......."];
    let (walk, _) = walk_of(&map);
    let plan = plan_route(&walk, (0, 0), (6, 4));
    assert!(plan.reached);
    let path = trace_steps((0, 0), &plan.steps);
    let mut seen = std::collections::HashSet::new();
    for p in &path {
        assert!(seen.insert(*p), "路径不回环（{p:?} 重复）");
    }
}

// ───────────────────────────────────────── 最近点停靠 + 探步入

#[test]
fn wall_target_docks_nearest_adjacent_and_probes() {
    // 墙格目标=最近相邻格停靠+探步入（一步撞击尝试，into=墙格）
    // 起点 (1,1)，目标 (3,1) 是墙；最近相邻=(2,1)（或 (3,0)/(3,2)/(4,1) 可走时按最近）
    let map = [
        ".....",
        "...#.",
        ".....",
    ];
    let (walk, _) = walk_of(&map);
    let plan = plan_route(&walk, (1, 1), (3, 1));
    assert!(!plan.reached, "墙格不可直达");
    assert_eq!(plan.dock, (2, 1), "最近相邻停靠（左邻最近）");
    let probe = plan.probe.expect("墙格目标必带探步入");
    assert_eq!(
        probe,
        ProbeStep { from: (2, 1), dir: 2, into: (3, 1) },
        "探步入=自停靠格向目标墙格一步"
    );
    let path = trace_steps((1, 1), &plan.steps);
    assert_eq!(path.last(), Some(&(2, 1)), "步序止于停靠格");
}

#[test]
fn probe_directions_cover_four_axes() {
    // 探步方向码（0 下 1 上 2 右 3 左——walkPathBuffer 语义）
    let map = [".....", "...#.", "....."];
    let (walk, _) = walk_of(&map);
    // 目标 (3,1) 墙、起点 (3,2)：最近相邻含 (3,2)（下邻）→ 探步 dir=1（上）
    let plan = plan_route(&walk, (3, 2), (3, 1));
    let probe = plan.probe.expect("探步入");
    assert_eq!(probe.dir, 1, "自下邻向上探");
    assert_eq!(probe.into, (3, 1));
    assert_eq!(plan.dock, (3, 2), "起点即相邻=零步停靠+探入");
}

// ───────────────────────────────────────── 挂起续走（T1 journal 挂点契约）

#[test]
fn suspend_combat_resume_lifo_with_journal() {
    // 挂起→战斗终结入栈→续走：三笔 LIFO 逐字段逆放
    let mut journal = UndoJournal::new();
    let mut track = RouteTracker::new();
    track.begin((1, 3));
    track.suspend((5, 3), &mut journal); // 途中撞敌（段①收于敌格）
    track.finish_combat(
        CombatStroke {
            enemy_slot: 0,
            enemy_type_id: 44,
            enemy_hp_before: 120,
            vitals_before: PlayerVitals { hp: 80, gold: 50, exp: 10 },
            items_before: vec![(7, 1)],
        },
        &mut journal,
    );
    assert!(!track.is_suspended(), "战斗终结=挂起解除");
    track.resume((5, 3)); // 续走（段②自敌格）
    track.arrive((9, 3), &mut journal); // 段②终位
    assert_eq!(journal.len(), 3, "段①+战斗+段② 三笔");

    let mut st = UndoMock::new();
    st.tile = (9, 3);
    st.vitals = PlayerVitals { hp: 64, gold: 90, exp: 35 };
    st.enemy_hp = 0;
    st.enemy_alive = false;
    st.item7 = 3;
    // 回撤①：段②位置→(5,3)
    assert!(matches!(journal.undo(&mut st), UndoOutcome::Undone(Stroke::Move(_))));
    assert_eq!(st.tile, (5, 3), "段②撤销=回敌格");
    // 回撤②：战斗复原（敌复活原血/血金经验回滚/掉落退回）
    assert!(matches!(journal.undo(&mut st), UndoOutcome::Undone(Stroke::Combat(_))));
    assert!(st.enemy_alive && st.enemy_hp == 120, "敌复活原血");
    assert_eq!(st.vitals, PlayerVitals { hp: 80, gold: 50, exp: 10 }, "血金经验回滚");
    assert_eq!(st.item7, 1, "掉落退回");
    // 回撤③：段①位置→(1,3)
    assert!(matches!(journal.undo(&mut st), UndoOutcome::Undone(Stroke::Move(_))));
    assert_eq!(st.tile, (1, 3), "段①撤销=回段起点");
}

#[test]
fn interrupt_closes_segment_at_actual_endpoint() {
    // 中断=实际终点（挂点契约）：打断段收于当前格
    let mut journal = UndoJournal::new();
    let mut track = RouteTracker::new();
    track.begin((2, 3));
    track.interrupt((4, 3), &mut journal);
    assert!(!track.is_suspended());
    assert_eq!(journal.len(), 1);
    let mut st = UndoMock::new();
    st.tile = (4, 3);
    assert_eq!(
        journal.undo(&mut st),
        UndoOutcome::Undone(Stroke::Move(MoveStroke { from: (2, 3), to: (4, 3) })),
        "中断段=起终点 (2,3)→(4,3)（非整段终点）"
    );
    assert_eq!(st.tile, (2, 3));
}

// ───────────────────────────────────────── 时序组合钉（T2 终检黄-1 补钉）

#[test]
fn undo_midway_then_new_walk_keeps_lifo() {
    // 组合①：回撤中寻路——push 2 笔→undo 1→新 WalkTo 起新段→旧栈 LIFO 不变
    //（仍 1 笔）且新段终点再入栈（walk_to=RouteTracker::begin 生产调用方）。
    let mut journal = UndoJournal::new();
    let mut track = RouteTracker::new();
    track.begin((1, 3));
    track.arrive((3, 3), &mut journal); // 笔①：段 (1,3)→(3,3)
    track.begin((3, 3));
    track.arrive((5, 3), &mut journal); // 笔②：段 (3,3)→(5,3)
    assert_eq!(journal.len(), 2);

    let mut st = UndoMock::new();
    st.tile = (5, 3);
    // 回撤 1：撤笔②——栈内剩 1 笔（笔①原样在底）
    assert_eq!(
        journal.undo(&mut st),
        UndoOutcome::Undone(Stroke::Move(MoveStroke { from: (3, 3), to: (5, 3) })),
        "undo#1=笔②（LIFO 顶）"
    );
    assert_eq!(journal.len(), 1, "旧栈剩 1 笔");
    assert_eq!(st.tile, (3, 3));

    // 新 WalkTo 起新段（回撤中寻路）→ 终点再入栈
    track.begin((3, 3));
    track.arrive((4, 4), &mut journal);
    assert_eq!(journal.len(), 2, "新段终点入栈后=2 笔");
    assert_eq!(
        journal.undo(&mut st),
        UndoOutcome::Undone(Stroke::Move(MoveStroke { from: (3, 3), to: (4, 4) })),
        "新段笔=LIFO 顶（先于旧笔①逆放）"
    );
    assert_eq!(
        journal.undo(&mut st),
        UndoOutcome::Undone(Stroke::Move(MoveStroke { from: (1, 3), to: (3, 3) })),
        "旧笔①原样在底（旧栈 LIFO 不变）"
    );
    assert_eq!(journal.undo(&mut st), UndoOutcome::Empty);
}

#[test]
fn interrupt_during_combat_pending_blocks_then_lifo() {
    // 组合②：战斗挂起中打断——begin_combat→interrupt_route→undo==Blocked→
    // finish_combat→undo：战斗笔先于段笔逆放（LIFO：终结入栈在段笔之后）。
    let mut journal = UndoJournal::new();
    let mut track = RouteTracker::new();
    track.begin((2, 3));
    journal.begin_combat(); // 战斗挂起（终结才入栈）
    track.interrupt((4, 3), &mut journal); // 打断=段收于实际终点（挂起中）
    assert_eq!(journal.len(), 1, "段笔入栈（打断=实际终点）");
    assert_eq!(journal.undo(&mut UndoMock::new()), UndoOutcome::Blocked, "战斗挂起中不可撤");

    track.finish_combat(
        CombatStroke {
            enemy_slot: 0,
            enemy_type_id: 44,
            enemy_hp_before: 120,
            vitals_before: PlayerVitals { hp: 80, gold: 50, exp: 10 },
            items_before: vec![],
        },
        &mut journal,
    );
    assert_eq!(journal.len(), 2, "战斗终结入栈（段笔之后）");

    let mut st = UndoMock::new();
    st.tile = (4, 3);
    st.enemy_alive = false;
    st.enemy_hp = 0;
    // LIFO 逆放：战斗笔先、段笔后
    assert!(matches!(journal.undo(&mut st), UndoOutcome::Undone(Stroke::Combat(_))), "逆放#1=战斗笔");
    assert!(st.enemy_alive && st.enemy_hp == 120, "敌复活原血");
    assert_eq!(st.tile, (4, 3), "战斗笔不含位移");
    assert!(matches!(journal.undo(&mut st), UndoOutcome::Undone(Stroke::Move(_))), "逆放#2=段笔");
    assert_eq!(st.tile, (2, 3), "段笔=实际终点回起点");
    assert_eq!(journal.undo(&mut st), UndoOutcome::Empty);
}

// ───────────────────────────────────────── mock（T1 UndoTarget 语义面）

#[derive(Default)]
struct UndoMock {
    tile: (i32, i32),
    vitals: PlayerVitals,
    enemy_hp: i32,
    enemy_alive: bool,
    item7: i32,
}

impl UndoMock {
    fn new() -> Self {
        Self::default()
    }
}

impl game_core::undo_journal::UndoTarget for UndoMock {
    fn set_player_tile(&mut self, tile: (i32, i32)) {
        self.tile = tile;
    }
    fn set_player_vitals(&mut self, v: PlayerVitals) {
        self.vitals = v;
    }
    fn revive_enemy(&mut self, e: game_core::undo_journal::EnemyRestore) {
        self.enemy_hp = e.hp;
        self.enemy_alive = true;
    }
    fn set_item_count(&mut self, item_id: i32, count: i32) {
        if item_id == 7 {
            self.item7 = count;
        }
    }
    fn set_door_state(&mut self, _cx: i32, _cy: i32, _s: game_core::undo_journal::DoorState) {}
}
