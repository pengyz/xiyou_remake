    use super::scene_test_support::scene;
    use crate::enums::GameMode;

    #[test]
    fn confirm_overlay_cancel_returns_to_panel() {
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18)); // Modern 面（清屏门控域）
        sc.game_mode = GameMode::ItemPanel;
        sc.overlay_active = true;
        sc.overlay_kind = 3;
        sc.key_value = -7;
        sc.tick(0, &[]); // debug 档：缺自环边 ⇒ census assert panic（RED）
        assert_eq!(sc.game_mode, GameMode::ItemPanel, "回面板=自环（Java gameMode=10 自赋值）");
        assert!(!sc.overlay_active, "kind3 -7 关层");
    }

    /// 边表机械对照（state-machine-anchors 同源判据）：自环边在册。
    #[test]
    fn itempanel_self_edge_registered() {
        assert!(
            crate::scene::TRANSFER_EDGES
                .contains(&(GameMode::ItemPanel, Some(GameMode::ItemPanel))),
            "ItemPanel→ItemPanel 自环边须在 TRANSFER_EDGES（件 D 维护义务）"
        );
    }
