    use super::scene_test_support::{scene, ROW};
    use super::*;

    #[test]
    fn single_step_walkto_then_manual_step_stops_at_next_cell() {
        let mut sc = scene((1, ROW), &[]);
        sc.walk_to(2, ROW);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while fc < 40 && !(sc.game_mode == GameMode::Map && sc.walk_phase == 0) {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!((sc.game_mode, sc.player_cell_x), (GameMode::Map, 2), "单步寻路到达");
        let armed_after_walkto = sc.script_walk_armed;
        sc.press_key(-4); // 手动向右一步
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        for _ in 0..12 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!(
            (sc.walk_phase, sc.player_cell_x, sc.view.player_px & 31),
            (0, 3, 0),
            "手动一步后停在下一格（寻路后武装残留={armed_after_walkto}）"
        );
    }
