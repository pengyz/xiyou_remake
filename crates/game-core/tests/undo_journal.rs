//! T1 undo_journal 判据（touch-native-undo-plan §3 TDD 红先行）。
//!
//! 用户判据原样钉死："走过去+战斗=2 个操作，按一次回撤战斗，再按一次回撤位置"；
//! **战斗终结才入栈**（战斗中途不可撤）。LIFO 逆放逐字段断言：敌复活原血/
//! 金币经验回滚/位置回起点；清栈四例（换层/存档/MOV 脚本事件格/连带战斗挂起）。

use game_core::undo_journal::{
    ClearReason, CombatStroke, DoorState, EnemyRestore, MoveStroke, PickupStroke, PlayerVitals,
    SpendKind, SpendStroke, Stroke, UndoJournal, UndoOutcome, UndoTarget,
};

// ───────────────────────────────────────── mock 目标态（逐字段断言面）

#[derive(Debug, Clone, PartialEq, Eq)]
struct MockEnemy {
    type_id: i32,
    hp: i32,
    alive: bool,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct MockState {
    player_tile: (i32, i32),
    vitals: PlayerVitals,
    enemies: Vec<MockEnemy>,
    items: Vec<(i32, i32)>,
    doors: Vec<(i32, i32, DoorState)>,
}

impl UndoTarget for MockState {
    fn set_player_tile(&mut self, tile: (i32, i32)) {
        self.player_tile = tile;
    }
    fn set_player_vitals(&mut self, vitals: PlayerVitals) {
        self.vitals = vitals;
    }
    fn revive_enemy(&mut self, enemy: EnemyRestore) {
        self.enemies[enemy.slot] =
            MockEnemy { type_id: enemy.type_id, hp: enemy.hp, alive: true };
    }
    fn set_item_count(&mut self, item_id: i32, count: i32) {
        match self.items.iter_mut().find(|(id, _)| *id == item_id) {
            Some(entry) => entry.1 = count,
            None => self.items.push((item_id, count)),
        }
    }
    fn set_door_state(&mut self, cx: i32, cy: i32, state: DoorState) {
        match self.doors.iter_mut().find(|(x, y, _)| *x == cx && *y == cy) {
            Some(entry) => entry.2 = state,
            None => self.doors.push((cx, cy, state)),
        }
    }
}

fn fresh_state() -> (UndoJournal, MockState) {
    let journal = UndoJournal::new();
    let state = MockState {
        player_tile: (2, 3),
        vitals: PlayerVitals { hp: 80, gold: 50, exp: 10 },
        enemies: vec![
            MockEnemy { type_id: 44, hp: 0, alive: false }, // 已灭
            MockEnemy { type_id: 45, hp: 30, alive: true },
        ],
        items: vec![(7, 1), (9, 0)],
        doors: vec![(4, 4, DoorState::Open)],
    };
    (journal, state)
}

// ───────────────────────────────────────── 用户判据原样

#[test]
fn undo_twice_combat_then_position_user_judge() {
    // "走过去+战斗=2 个操作，按一次回撤战斗，再按一次回撤位置"
    let (mut journal, mut st) = fresh_state();
    // 操作①：走过去（位移笔——整段起终点 (2,3)→(5,3)）
    journal.push_move(MoveStroke { from: (2, 3), to: (5, 3) });
    st.player_tile = (5, 3);
    // 操作②：战斗（敌 0 号复活前血 120；战后玩家 hp64/gold90/exp35，掉落 (7) 3 个）
    journal.begin_combat();
    assert!(journal.combat_in_progress());
    journal.finish_combat(CombatStroke {
        enemy_slot: 0,
        enemy_type_id: 44,
        enemy_hp_before: 120,
        vitals_before: PlayerVitals { hp: 80, gold: 50, exp: 10 },
        items_before: vec![(7, 1)],
    });
    assert!(!journal.combat_in_progress(), "终结后挂起清除");
    assert_eq!(journal.len(), 2, "走过去+战斗=2 笔");

    // 回撤①：战斗复原——敌复活原血、金币/经验回滚、掉落退回
    st.enemies[0] = MockEnemy { type_id: 44, hp: 0, alive: false };
    st.vitals = PlayerVitals { hp: 64, gold: 90, exp: 35 };
    st.items = vec![(7, 3), (9, 0)];
    match journal.undo(&mut st) {
        UndoOutcome::Undone(Stroke::Combat(c)) => {
            assert_eq!(c.enemy_slot, 0, "战斗笔敌位");
            assert_eq!(c.enemy_hp_before, 120, "战斗笔原血");
        }
        other => panic!("期望战斗笔，实际 {other:?}"),
    }
    assert!(st.enemies[0].alive, "敌复活");
    assert_eq!(st.enemies[0].hp, 120, "敌复活原血");
    assert_eq!(st.enemies[0].type_id, 44, "敌型不变");
    assert_eq!(st.vitals.hp, 80, "玩家血回滚");
    assert_eq!(st.vitals.gold, 50, "金币回滚");
    assert_eq!(st.vitals.exp, 10, "经验回滚");
    assert_eq!(st.items, vec![(7, 1), (9, 0)], "掉落退回");
    assert_eq!(st.player_tile, (5, 3), "位置不动（战斗笔不含位移）");

    // 回撤②：位置回起点
    assert_eq!(journal.undo(&mut st), UndoOutcome::Undone(Stroke::Move(MoveStroke { from: (2, 3), to: (5, 3) })));
    assert_eq!(st.player_tile, (2, 3), "位置回起点");
    assert_eq!(journal.undo(&mut st), UndoOutcome::Empty, "两笔撤完");
}

// ───────────────────────────────────────── LIFO 三笔组合

#[test]
fn undo_lifo_three_strokes_reverse_order() {
    let (mut journal, mut st) = fresh_state();
    // 三笔：位移→战斗→拾取开门（LIFO 逆放=先拾取、再战斗、再位移）
    journal.push_move(MoveStroke { from: (2, 3), to: (4, 4) });
    st.player_tile = (4, 4);
    journal.finish_combat(CombatStroke {
        enemy_slot: 1,
        enemy_type_id: 45,
        enemy_hp_before: 30,
        vitals_before: PlayerVitals { hp: 80, gold: 50, exp: 10 },
        items_before: vec![],
    });
    journal.push_pickup(PickupStroke {
        item: Some((9, 0, 2)),                       // 拾取道具 9：0→2
        door: Some((4, 4, DoorState::Locked, DoorState::Open)), // 开门：Locked→Open
    });
    st.enemies[1] = MockEnemy { type_id: 45, hp: 0, alive: false };
    st.vitals = PlayerVitals { hp: 70, gold: 60, exp: 20 };
    st.items = vec![(7, 1), (9, 2)];
    st.doors = vec![(4, 4, DoorState::Open)];
    assert_eq!(journal.len(), 3);

    // 逆放①：拾取开门——道具退回+门复锁
    match journal.undo(&mut st) {
        UndoOutcome::Undone(Stroke::Pickup(p)) => {
            assert_eq!(p.item, Some((9, 0, 2)), "拾取笔道具 delta");
            assert_eq!(p.door, Some((4, 4, DoorState::Locked, DoorState::Open)), "开门笔 delta");
        }
        other => panic!("期望拾取笔，实际 {other:?}"),
    }
    assert_eq!(st.items, vec![(7, 1), (9, 0)], "道具退回");
    assert_eq!(st.doors, vec![(4, 4, DoorState::Locked)], "门复锁");
    assert!(st.enemies[1].alive == false, "敌态不动");

    // 逆放②：战斗——复活原血+血金经验回滚
    assert!(matches!(journal.undo(&mut st), UndoOutcome::Undone(Stroke::Combat(_))), "逆放②=战斗笔");
    assert!(st.enemies[1].alive, "敌复活");
    assert_eq!(st.enemies[1].hp, 30, "敌复活原血");
    assert_eq!(st.vitals, PlayerVitals { hp: 80, gold: 50, exp: 10 }, "血金经验回滚");

    // 逆放③：位移——位置回起点
    assert!(matches!(journal.undo(&mut st), UndoOutcome::Undone(Stroke::Move(_))), "逆放③=位移笔");
    assert_eq!(st.player_tile, (2, 3), "位置回起点");
    assert_eq!(journal.len(), 0);
}

#[test]
fn move_stroke_interrupted_uses_actual_endpoint() {
    // 挂点契约（plan §2 入栈时机）：寻路打断=按实际终点入栈——T2 传实际终点
    let (mut journal, mut st) = fresh_state();
    journal.push_move(MoveStroke { from: (2, 3), to: (3, 3) }); // 整段本想走到 (5,3)，(3,3) 中断
    st.player_tile = (3, 3);
    assert_eq!(
        journal.undo(&mut st),
        UndoOutcome::Undone(Stroke::Move(MoveStroke { from: (2, 3), to: (3, 3) })),
        "笔内终点=中断点 (3,3)（非整段终点）"
    );
    assert_eq!(st.player_tile, (2, 3));
}

// ───────────────────────────────────────── 战斗中途不入栈

#[test]
fn combat_mid_fight_not_undoable() {
    // 战斗终结才入栈（plan §0/§2）：中途 undo=Blocked 且栈内无战斗笔
    let (mut journal, mut st) = fresh_state();
    journal.push_move(MoveStroke { from: (2, 3), to: (5, 3) });
    st.player_tile = (5, 3);
    journal.begin_combat();
    assert!(journal.combat_in_progress());
    assert_eq!(journal.undo(&mut st), UndoOutcome::Blocked, "战斗中途不可撤");
    assert_eq!(journal.len(), 1, "中途不入栈（仅位移笔）");
    assert_eq!(st.player_tile, (5, 3), "状态不动");

    journal.finish_combat(CombatStroke {
        enemy_slot: 0,
        enemy_type_id: 44,
        enemy_hp_before: 120,
        vitals_before: PlayerVitals { hp: 80, gold: 50, exp: 10 },
        items_before: vec![],
    });
    assert_eq!(journal.len(), 2, "终结入栈");
    assert!(matches!(journal.undo(&mut st), UndoOutcome::Undone(Stroke::Combat(_))), "终结后可撤");
}

// ───────────────────────────────────────── 清栈四例（边界 1：v1 不做脚本副作用反转）

#[test]
fn clear_on_floor_change() {
    let (mut journal, mut st) = fresh_state();
    journal.push_move(MoveStroke { from: (2, 3), to: (5, 3) });
    journal.push_pickup(PickupStroke { item: Some((9, 0, 1)), door: None });
    journal.clear(ClearReason::FloorChange); // 换层（changeFloor/LAY/传送）
    assert_eq!(journal.len(), 0);
    assert_eq!(journal.undo(&mut st), UndoOutcome::Empty, "清栈后无可撤");
}

#[test]
fn clear_on_save() {
    let (mut journal, mut st) = fresh_state();
    journal.push_move(MoveStroke { from: (2, 3), to: (5, 3) });
    journal.clear(ClearReason::Save); // 存档
    assert_eq!(journal.len(), 0);
    assert_eq!(journal.undo(&mut st), UndoOutcome::Empty);
}

#[test]
fn clear_on_script_event() {
    // 任何 MOV/脚本事件格触发（宝箱/陷阱/剧情/开关）=清栈——
    // 不可反转副作用不进回撤域（v1 务实边界，注释留痕见 undo_journal.rs）
    let (mut journal, mut st) = fresh_state();
    journal.push_move(MoveStroke { from: (2, 3), to: (5, 3) });
    journal.finish_combat(CombatStroke {
        enemy_slot: 0,
        enemy_type_id: 44,
        enemy_hp_before: 120,
        vitals_before: PlayerVitals { hp: 80, gold: 50, exp: 10 },
        items_before: vec![],
    });
    journal.clear(ClearReason::ScriptEvent);
    assert_eq!(journal.len(), 0, "脚本事件格触发=全清");
    assert_eq!(journal.undo(&mut st), UndoOutcome::Empty);
}

#[test]
fn clear_drops_pending_combat() {
    // 清栈连带取消进行中战斗挂起（换层/存档/脚本事件不会留"永久不可撤"残态）
    let (mut journal, mut st) = fresh_state();
    journal.begin_combat();
    journal.clear(ClearReason::FloorChange);
    assert!(!journal.combat_in_progress(), "挂起随清栈取消");
    assert_eq!(journal.undo(&mut st), UndoOutcome::Empty, "清栈后=Empty 而非 Blocked");
}

// ───────────────────────────────────────── 对话/商店/背包笔（边界 2）

#[test]
fn spend_stroke_undo_rolls_back_consumption() {
    let (mut journal, mut st) = fresh_state();
    // 商店购入道具 9×2：金 50→10、道具 9: 0→2
    journal.push_spend(SpendStroke {
        kind: SpendKind::Shop,
        vitals_before: PlayerVitals { hp: 80, gold: 50, exp: 10 },
        items_before: vec![(7, 1), (9, 0)],
    });
    st.vitals = PlayerVitals { hp: 80, gold: 10, exp: 10 };
    st.items = vec![(7, 1), (9, 2)];

    match journal.undo(&mut st) {
        UndoOutcome::Undone(Stroke::Spend(sp)) => {
            assert_eq!(sp.kind, SpendKind::Shop, "消耗笔种类");
            assert_eq!(sp.vitals_before.gold, 50);
        }
        other => panic!("期望消耗笔，实际 {other:?}"),
    }
    assert_eq!(st.vitals.gold, 50, "金币回滚（消耗型决策反悔）");
    assert_eq!(st.items, vec![(7, 1), (9, 0)], "道具退回");
}

#[test]
fn spend_kind_variants_are_distinct_strokes() {
    // 对话/商店/背包使用=各自独立操作笔
    let mut journal = UndoJournal::new();
    for kind in [SpendKind::Dialogue, SpendKind::Shop, SpendKind::InventoryUse] {
        journal.push_spend(SpendStroke {
            kind,
            vitals_before: PlayerVitals { hp: 80, gold: 50, exp: 10 },
            items_before: vec![],
        });
    }
    assert_eq!(journal.len(), 3, "三种消耗各占一笔");
}

// ───────────────────────────────────────── 可视化面（边界 3：可回撤 N 步）

#[test]
fn undo_depth_reflects_journal_length() {
    let (mut journal, _st) = fresh_state();
    assert_eq!(journal.len(), 0);
    journal.push_move(MoveStroke { from: (0, 0), to: (1, 0) });
    journal.push_pickup(PickupStroke { item: Some((7, 1, 2)), door: None });
    assert_eq!(journal.len(), 2, "可回撤 N 步可视化数据面");
}
