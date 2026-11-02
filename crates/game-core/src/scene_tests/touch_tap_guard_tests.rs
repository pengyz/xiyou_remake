    use super::scene_test_support::{scene, ROW};
    use super::*;

    /// 地图上弹着提示框：点地寻路 / 探步一律拒绝，且拒绝 = 零状态变化（探针 P1a/P1b：
    /// 曾在框后挂上寻路并切到 Dialog、或在框开着时起步撞格）。
    #[test]
    fn field_taps_refused_while_overlay_open() {
        let mut sc = scene((1, ROW), &[]);
        sc.show_overlay_message(0, "提示", 0, 0);
        assert!(!sc.field_tap_allowed(), "弹框期不接受点地");
        let plan = sc.walk_to(5, ROW);
        assert!(!plan.reached && plan.steps.is_empty(), "拒绝寻路：{plan:?}");
        assert_eq!((sc.game_mode, sc.walk_phase, sc.walk_step_count), (GameMode::Map, 0, 0), "模式/相位/步数不变");
        assert!(!sc.route.is_open(), "未开寻路段");
        assert!(!sc.probe_step(2), "拒绝探步");
        assert_eq!(sc.walk_phase, 0, "探步未起步");
        assert!(sc.overlay_active, "弹框仍开着");
    }

    /// 行走中（walkPhase 1）、探索（2）、换层（4）、战斗（5）、换层遮幅中同样拒绝；空闲地图允许。
    #[test]
    fn field_taps_refused_unless_field_idle() {
        let mut sc = scene((1, ROW), &[]);
        assert!(sc.field_tap_allowed(), "空闲地图允许点地");
        for wp in [1, 2, 4, 5] {
            sc.walk_phase = wp;
            assert!(!sc.field_tap_allowed(), "walkPhase {wp} 不接受点地");
            let plan = sc.walk_to(5, ROW);
            assert!(plan.steps.is_empty() && sc.walk_phase == wp && !sc.route.is_open(), "walkPhase {wp}：walk_to 零变化");
            sc.facing = 0;
            assert!(!sc.probe_step(2) && sc.walk_phase == wp, "walkPhase {wp}：probe_step 零变化");
            assert_eq!(sc.facing, 0, "walkPhase {wp}：拒绝的探步不改朝向（BUG-047 写朝向须在守卫之后）");
        }
        sc.walk_phase = 0;
        sc.transitioning = true;
        assert!(!sc.field_tap_allowed(), "换层遮幅中不接受点地");
    }

    /// 弹框盖住的下层行 / 格不接受指针命中（探针 P2：道具使用确认框开着时点另一格 ⇒ 光标改到
    /// 被点那格并"使用"了它——框里显示的却是原先那件）。
    #[test]
    fn pointer_hit_ignores_rows_beneath_overlay() {
        let mut sc = scene((1, ROW), &[]);
        sc.player.item_types[0] = 16;
        sc.player.item_uses[0] = 1;
        sc.player.item_types[1] = 19;
        sc.player.item_uses[1] = 1;
        sc.player.item_stack_size = 2;
        sc.transition_to(GameMode::ItemPanel);
        sc.enter_item_panel();
        let (ox, oy) = sc.item_cell_origin();
        let cell1 = (ox + 40 + 20, oy + 20);
        assert!(
            matches!(sc.pointer_hit(cell1.0, cell1.1), PointerHit::Row { index: 1, .. }),
            "无弹框：命中 1 号格"
        );
        sc.show_overlay_message(3, "芭蕉扇", 1, 2);
        assert!(
            matches!(sc.pointer_hit(cell1.0, cell1.1), PointerHit::None),
            "确认框开着：下层道具格不可点"
        );
    }
