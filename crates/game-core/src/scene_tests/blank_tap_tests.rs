    use super::scene_test_support::{scene, ROW};
    use super::*;

    #[test]
    fn blank_tap_continues_only_in_continue_contexts() {
        let mut sc = scene((1, ROW), &[]);
        for (mode, want) in [
            (GameMode::Dialog, Some(-5)),
            (GameMode::Ending, Some(-5)),
            (GameMode::Map, None),
            (GameMode::InGameMenu, None),
            (GameMode::SaveSlot, None),
            (GameMode::Alchemy, None),
            (GameMode::ItemPanel, None),
            (GameMode::About, None),
            (GameMode::EnemyInfo, None),
            (GameMode::Help, None),
        ] {
            sc.game_mode = mode;
            assert_eq!(sc.blank_tap_key(), want, "{mode:?} 无浮层点空白");
        }
        sc.game_mode = GameMode::Map;
        for (kind, l, r, want) in [
            (0, 0, 0, Some(-5)), // 提示框：关闭
            (1, 0, 3, Some(-5)), // 拾取说明：关闭
            (3, 0, 3, Some(-5)), // 说明类道具：关闭
            (4, 0, 0, Some(-5)), // 赠礼 / 告示：关闭
            (3, 1, 2, None),     // 道具使用确认（○/×）：不代按
            (4, 1, 2, None),     // 商人报价（○/×）：不代按
        ] {
            sc.show_overlay_message(kind, "x", l, r);
            assert_eq!(sc.blank_tap_key(), want, "浮层 kind {kind} 脚注 ({l},{r}) 点空白");
        }
        sc.show_overlay_message(0, "x", 0, 0);
        sc.transitioning = true;
        assert_eq!(sc.blank_tap_key(), None, "换层遮幅期（浮层不画却照吞键）点空白不代按");
    }

    /// 【STAGE-B-008 · spec §5.2】赠予 / 报价（kind 4、型 77、flags&2）点空白不代按；同实体 flags&2 = 0（告示子类）照旧代按 −5；
    /// 型 78 报价（○/× 图标）本就不代按。
    #[test]
    fn blank_tap_does_not_accept_type77_offer() {
        let mut sc = scene((2, ROW), &[(77, 5, ROW), (78, 7, ROW)]);
        let i77 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 77).unwrap();
        let i78 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 78).unwrap();
        sc.dialog.overlay_active = true;
        sc.dialog.overlay_kind = 4;
        sc.merchant_entity = i77 as i32;
        sc.entities.items[i77].param = (2 << 8) | 1;
        sc.dialog.overlay_left_icon = 0;
        sc.dialog.overlay_right_icon = 0;
        assert_eq!(sc.blank_tap_key(), None, "型 77 报价：点空白不接受");
        sc.entities.items[i77].param = 1;
        assert_eq!(sc.blank_tap_key(), Some(-5), "型 77 告示子类（flags&2 = 0）：点空白关闭");
        sc.merchant_entity = i78 as i32;
        sc.entities.items[i78].param = (2 << 8) | 1;
        sc.dialog.overlay_left_icon = 1;
        sc.dialog.overlay_right_icon = 2;
        assert_eq!(sc.blank_tap_key(), None, "型 78 报价（○/×）不代按（原规则）");
        sc.merchant_entity = -1;
        sc.dialog.overlay_left_icon = 0;
        sc.dialog.overlay_right_icon = 0;
        assert_eq!(sc.blank_tap_key(), Some(-5), "merchant_entity=-1 的 kind 4（测试构造态；生产长文浏览必经 case16 设 merchant 且靠 &2=0 放行，复核 T9-R2）照旧代按");
    }
