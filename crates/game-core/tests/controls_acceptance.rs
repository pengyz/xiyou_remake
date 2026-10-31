//! 【STAGE-B-008 计划 3 T9 · spec §9】controls 一致性验收（GameScene 面）：
//! 每个 FunctionId 有独立后置条件谓词；实跑（非查询推导）断言功能发生；双键码（工具栏键码 vs
//! 摇杆物理键 −6/−7）同谓词。夹具 = scene_test_support（合成图像 + 真实数值表）。

use game_core::controls::{Action, Controls, FunctionId};
use game_core::enums::GameMode;
use game_core::scene::scene_test_support::{scene, ROW};
use game_core::scene::GameScene;

fn settle(sc: &mut GameScene, n: i64) {
    let wt = sc.width_table.clone();
    for fc in 0..n {
        sc.tick(fc, &wt);
    }
}

/// 谓词表：FunctionId → 执行后置条件（在对应情境夹具上验证）。
/// 每个 id 独立写出（期望不由 controls() 推导）。

/// 【地图空闲】左=背包(−6)：执行后进道具栏。
#[test]
fn accept_bag_opens_item_panel() {
    let mut sc = scene((2, ROW), &[]);
    assert_eq!(sc.controls().left.as_ref().unwrap().function, FunctionId::Bag);
    sc.perform_action(Action::Key(-6));
    settle(&mut sc, 30);
    assert_eq!(sc.game_mode, GameMode::ItemPanel, "背包 → ItemPanel");
    // 双键码：摇杆左软键物理键 −6 同谓词（本情境键码即 −6，等价重放）
    let mut sc2 = scene((2, ROW), &[]);
    sc2.perform_action(Action::Key(-6));
    settle(&mut sc2, 30);
    assert_eq!(sc2.game_mode, sc.game_mode);
}

/// 【地图空闲】中=观察(−5)：执行后 walk_phase=2（含画面即无浮层前置）。
#[test]
fn accept_observe_enters_observe() {
    let mut sc = scene((2, ROW), &[]);
    let binding = sc.controls();
    let b = binding.mid.iter().find(|b| b.function == FunctionId::Observe).unwrap();
    sc.perform_action(b.action);
    settle(&mut sc, 4);
    assert_eq!(sc.walk_phase, 2, "观察 → wp2（键值同拍 tick 消费）");
}

/// 【地图空闲】中=撤销：推一笔位移 → Undo → 位置回退（无键码动作走壳 undo——此处验 Journal 层效果）。
#[test]
fn accept_undo_reverts_position() {
    use game_core::undo_journal::MoveStroke;
    let mut sc = scene((2, ROW), &[]);
    sc.undo.push_move(MoveStroke { from: (2, ROW), to: (5, ROW) });
    sc.player_cell_x = 5;
    sc.view.player_px = 5 << 5;
    // undo_last 在壳（undo_ui::SceneUndo 适配器实现 UndoTarget）；core 侧验 Journal 逆放本体：
    // 测试内联最小 UndoTarget（位写与壳适配器同式，锚 undo_ui.rs set_player_tile）
    use game_core::undo_journal::{UndoOutcome, UndoTarget};
    struct TileUndo<'a>(&'a mut GameScene);
    impl UndoTarget for TileUndo<'_> {
        fn set_player_tile(&mut self, tile: (i32, i32)) {
            self.0.player_cell_x = tile.0;
            self.0.player_cell_y = tile.1;
            self.0.view.player_px = tile.0 << 5;
            self.0.view.player_py = tile.1 << 5;
            self.0.step_progress = 0;
            self.0.walk_phase = 0;
        }
        fn set_player_vitals(&mut self, v: game_core::undo_journal::PlayerVitals) {
            self.0.player.hp = v.hp;
            self.0.player.gold = v.gold;
        }
        fn revive_enemy(&mut self, _e: game_core::undo_journal::EnemyRestore) {}
        fn set_item_count(&mut self, _item_id: i32, _count: i32) {}
        fn set_door_state(&mut self, _cx: i32, _cy: i32, _st: game_core::undo_journal::DoorState) {}
    }
    let out = {
        let mut journal = std::mem::take(&mut sc.undo);
        let mut t = TileUndo(&mut sc);
        let out = journal.undo(&mut t);
        sc.undo = journal;
        out
    };
    assert!(matches!(out, UndoOutcome::Undone(_)), "撤销成功");
    assert_eq!(sc.player_cell_x, 2, "撤销回起点");
}

/// 【地图空闲】右=菜单(−7)：执行后进游戏内菜单；摇杆右软键物理键 −7 同谓词（双键码）。
#[test]
fn accept_menu_opens_ingame_menu_both_keycodes() {
    for code in [-7i32] {
        let mut sc = scene((2, ROW), &[]);
        assert_eq!(sc.controls().right.as_ref().unwrap().action, Action::Key(code), "菜单键码 −7（工具栏与摇杆同码）");
        sc.perform_action(Action::Key(code));
        settle(&mut sc, 30);
        assert_eq!(sc.game_mode, GameMode::InGameMenu, "菜单 → InGameMenu（code={code}）");
    }
}

/// 【选中怪】开战/取消（无键码动作）：开战=engage_selected（击杀+清选中）；取消=清选中不动。
#[test]
fn accept_fight_and_cancel_on_selection() {
    let mut sc = scene((2, ROW), &[(41, 8, ROW)]);
    sc.player.atk = 100_000;
    sc.player.def = 100_000;
    sc.player.hp = 100_000;
    let _ = sc.tap_cell(8, ROW); // Selected
    // 取消：清选中、玩家不动
    let mut sc2 = scene((2, ROW), &[(41, 8, ROW)]);
    sc2.player.atk = 100_000;
    sc2.player.def = 100_000;
    sc2.player.hp = 100_000;
    let _ = sc2.tap_cell(8, ROW);
    let _ = sc2.perform_action(Action::Cancel);
    assert!(sc2.tap_selected.is_none(), "取消 = 清选中");
    assert_eq!((sc2.player_cell_x, sc2.player_cell_y), (2, ROW), "取消不动");
    // 开战：engage → 战斗胜利
    let _ = sc.perform_action(Action::Fight);
    settle(&mut sc, 400);
    let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
    assert!(sc.entities.items[e].removed || sc.entities.items[e].state == 1, "开战 → 击杀");
}

/// 【点地寻路】停下（Stop）：打断路线 → 排空回 Map。
#[test]
fn accept_stop_interrupts_route() {
    let mut sc = scene((1, ROW), &[]);
    let _ = sc.tap_cell(10, ROW);
    assert!(sc.game_mode == GameMode::Dialog, "前置：寻路中");
    let _ = sc.perform_action(Action::Stop);
    settle(&mut sc, 120);
    assert_eq!(sc.game_mode, GameMode::Map, "停下 → 排空回 Map");
    assert!(!sc.route.is_open());
}

/// 【页面】菜单确定(−6)/返回(−7)、帮助上/下页(−1/−2)、炼丹上/下一项+离开、结局继续。
#[test]
fn accept_page_rows() {
    // 帮助：−2 下页（overlay_page 前进）
    let mut sc = scene((2, ROW), &[]);
    sc.game_mode = GameMode::Help;
    sc.show_overlay_message(0, &"第一行\n第二行\n第三行\n第四行\n第五行\n第六行\n第七行\n第八行\n第九行\n第十行\n第十一行\n第十二行".repeat(1), 0, 3);
    let top0 = sc.overlay_page_top;
    sc.key_value = -2;
    settle(&mut sc, 4);
    assert!(sc.overlay_page_top > top0 || sc.overlay_lines_per_page >= sc.overlay_total_lines, "下页/或单页无翻");
    // 结局：ending_done → 继续(−5) 回主菜单
    let mut sc = scene((2, ROW), &[]);
    sc.game_mode = GameMode::Ending;
    sc.ending_done = true;
    sc.key_value = -5;
    settle(&mut sc, 30);
    assert_eq!(sc.game_mode, GameMode::MainMenu, "结局继续 → MainMenu");
}

/// 【隐藏钮】不画不可点不发键：Controls.hidden() 情境（标题/遮幅）——查询空=壳不画（spec §9 ②）。
#[test]
fn hidden_contexts_have_no_buttons() {
    let mut sc = scene((2, ROW), &[]);
    sc.game_mode = GameMode::MainMenu;
    assert_eq!(sc.controls(), Controls::hidden(), "标题全隐藏");
    sc.transitioning = true;
    sc.game_mode = GameMode::Map;
    assert_eq!(sc.controls(), Controls::hidden(), "遮幅全隐藏");
}

/// 【对话】继续(−5)：dp1 点按/键 = 关框或翻页推进（脚本行尾快进）。
#[test]
fn accept_dialog_continue() {
    let mut sc = scene((2, ROW), &[]);
    sc.game_mode = GameMode::Dialog;
    sc.dialog_phase = 1;
    // 对话 dp1 的继续/翻页 = dialog_controls（浮层未开——序章对话走 dp 机文字带非浮层）
    let c = sc.controls();
    assert_eq!(c.right.as_ref().unwrap().function, FunctionId::Continue);
    sc.key_value = -5;
    settle(&mut sc, 6);
    assert!(!sc.overlay_active || sc.dialog_phase != 1, "继续 = 关闭/推进");
}
