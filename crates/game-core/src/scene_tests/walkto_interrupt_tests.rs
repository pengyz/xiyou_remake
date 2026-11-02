    use super::scene_test_support::{scene, ROW};
    use super::*;

    /// 打断后走完当前这一步即回 Map；全程不进 dp0、不写当前脚本旗标、解除脚本行走武装
    /// （武装残留会让 Map 态下一次手动步进走 advanceScriptWalk 臂而不归 walkPhase 0）。
    #[test]
    fn walkto_interrupt_drains_to_map_without_script_tail() {
        let mut sc = scene((1, ROW), &[]);
        let cur = sc.current_script_index;
        sc.script_line_flags[cur] = false;
        sc.walk_to(5, ROW);
        let wt = sc.width_table.clone();
        sc.tick(0, &wt); // dp3 弹第一步、起步
        sc.press_key(-1); // 任意键打断（壳层投键口径）
        sc.release_key();
        let mut trail = Vec::new();
        for fc in 1..60 {
            sc.tick(fc, &wt);
            trail.push((sc.game_mode, sc.dialog_phase));
            if sc.game_mode == GameMode::Map && sc.walk_phase == 0 {
                break;
            }
        }
        assert_eq!(sc.game_mode, GameMode::Map, "打断后回 Map：{trail:?}");
        assert!(
            !trail.iter().any(|&(m, dp)| m == GameMode::Dialog && dp == 0),
            "不得落入脚本机 dp0：{trail:?}"
        );
        assert!(!sc.script_line_flags[cur], "当前脚本完成旗标不得被写");
        assert!(!sc.script_walk_armed, "脚本行走武装已解除");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "走完打断时的那一步后停下");
    }

    /// 剧情接管（踩剧情点 ⇒ loadLevelScript）后的 dp 归剧情所有：先前的打断不得让剧情中途切回 Map。
    #[test]
    fn script_takeover_after_interrupt_keeps_dialog() {
        let mut sc = scene((1, ROW), &[]);
        sc.dialog_data.scripts = vec!["TAK_0_0 ".to_string()];
        sc.walk_to(5, ROW);
        sc.press_key(-1); // 打断（未出拍）
        sc.load_level_script(0); // 剧情接管
        sc.find_path(1, ROW, 3, ROW); // 剧情 MOV 驱动的脚本行走
        sc.script_walk_armed = true;
        sc.dialog_phase = 3;
        let wt = sc.width_table.clone();
        for fc in 0..12 {
            sc.tick(fc, &wt);
            assert_eq!(sc.game_mode, GameMode::Dialog, "剧情行走期仍是 Dialog（拍 {fc}）");
        }
    }
