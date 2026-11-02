    use super::scene_test_support::scene;
    use crate::enums::GameMode;

    #[test]
    fn item13_use_from_panel_reaches_enemyinfo() {
        let mut sc = scene((2, 2), &[(43, 5, 5), (78, 6, 6)]);
        sc.player.item_types[0] = 13;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        sc.item_cursor = 0;
        sc.game_mode = GameMode::ItemPanel;
        sc.key_value = -5;
        sc.tick_item_panel(); // 型13：transition_to(Map) + activate_item(13)→EnemyInfo
        assert_eq!(sc.game_mode, GameMode::EnemyInfo, "型13 使用后应进图鉴（mode5）");
    }

    #[test]
    fn item14_use_from_panel_reaches_recordbrowser() {
        let mut sc = scene((2, 2), &[(43, 5, 5)]);
        sc.player.item_types[0] = 14;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        sc.item_cursor = 0;
        sc.journal.push((0, 0)); // 非空日志（空日志回退=“没有记录”回面板，另一分支）
        sc.game_mode = GameMode::ItemPanel;
        sc.key_value = -5;
        sc.tick_item_panel();
        assert_eq!(sc.game_mode, GameMode::RecordBrowser, "型14 使用后应进记录浏览器（mode12）");
    }
