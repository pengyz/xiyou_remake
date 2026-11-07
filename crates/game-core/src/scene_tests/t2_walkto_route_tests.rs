    use super::scene_test_support::{scene, ROW, SceneSetup, apply_setup};
    use super::*;

    fn run_until_settled(sc: &mut GameScene, limit: i64) {
        let wt = sc.width_table.clone();
        for fc in 0..limit {
            sc.tick(fc, &wt);
            if !sc.script_walk_armed && sc.walk_phase == 0 {
                return;
            }
        }
    }

    #[test]
    fn t2_walk_to_feeds_buffer_and_arrives() {
        // WalkTo 链：tap 格→walkPathBuffer 推进→终位断言+位移笔（整段起终点）
        let mut sc = scene((1, ROW), &[]);
        let plan = sc.walk_to(5, ROW);
        assert!(plan.reached, "直线路直达");
        assert_eq!(sc.walk_step_count, 4, "喂 walkPathBuffer 4 步");
        run_until_settled(&mut sc, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "终位=tap 格");
        assert_eq!(sc.undo.len(), 1, "段终位=位移笔一笔");
    }

    /// 【BUG-043】教程路径箭头（SEE flag=1：findPath 写 walkPathBuffer + pathArrowsVisible）下点地寻路：箭头须与用方向键
    /// 走同一条路线时完全一致（沿箭头走 ⇒ 逐格消费、余下箭头不动；偏离箭头 ⇒ 箭头熄灭，a.java:5748-5761 label216）。
    /// 旧行为：walk_to 覆写同一个 walkPathBuffer，箭头改画寻路路线（以玩家为起点 ⇒ "错位"），且 dp3 弹步不走 label216。
    fn arrows_scene(from: i32, to: i32) -> GameScene {
        let mut sc = scene((from, ROW), &[]);
        assert!(sc.find_path(from, ROW, to, ROW), "前置：箭头路径可达");
        sc.path_arrows_visible = true; // op_see flag=1 同款
        sc
    }

    /// 方向键逐格走到 `dest`（参照行为：每格按一下、松开、等走完这一格再按下一下）。
    fn key_walk(sc: &mut GameScene, code: i32, dest: (i32, i32)) {
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while (sc.player_cell_x, sc.player_cell_y) != dest && fc < 400 {
            let at = (sc.player_cell_x, sc.player_cell_y);
            sc.press_key(code);
            sc.tick(fc, &wt);
            fc += 1;
            sc.release_key();
            while ((sc.player_cell_x, sc.player_cell_y) == at || sc.walk_phase != 0) && fc < 400 {
                sc.tick(fc, &wt);
                fc += 1;
            }
        }
        for _ in 0..40 {
            sc.tick(fc, &wt);
            fc += 1;
        }
    }

    /// 点地走到 `dest`，逐拍记录箭头格。
    fn tap_walk(sc: &mut GameScene, dest: (i32, i32)) -> Vec<Vec<(i32, i32)>> {
        let wt = sc.width_table.clone();
        assert!(sc.walk_to(dest.0, dest.1).reached);
        let mut frames = vec![sc.path_arrow_cells()];
        for fc in 0..400 {
            sc.tick(fc, &wt);
            frames.push(sc.path_arrow_cells());
            if !sc.script_walk_armed && sc.walk_phase == 0 && !sc.route.is_open() {
                break;
            }
        }
        for fc in 400..440 {
            sc.tick(fc, &wt);
        }
        frames
    }

    #[test]
    fn bug043_tap_walk_along_arrows_consumes_like_keys() {
        let original = arrows_scene(1, 6).path_arrow_cells();
        assert_eq!(original, vec![(2, ROW), (3, ROW), (4, ROW), (5, ROW), (6, ROW)], "前置：箭头 (1)→(6)");
        let mut keys = arrows_scene(1, 6);
        key_walk(&mut keys, -4, (3, ROW));
        let mut touch = arrows_scene(1, 6);
        let frames = tap_walk(&mut touch, (3, ROW));
        assert_eq!((touch.player_cell_x, touch.player_cell_y), (3, ROW), "走到点中格");
        assert_eq!(touch.game_mode, GameMode::Map);
        assert_eq!(keys.path_arrow_cells(), vec![(4, ROW), (5, ROW), (6, ROW)], "参照：方向键沿箭头走两格 ⇒ 余下三格");
        assert_eq!(touch.path_arrow_cells(), keys.path_arrow_cells(), "点地 = 方向键");
        for (t, cells) in frames.iter().enumerate() {
            assert!(
                original.ends_with(cells),
                "第 {t} 拍箭头 {cells:?} 须是原箭头路径的尾段（不得改画寻路路线 / 错位）"
            );
        }
    }

    #[test]
    fn bug043_tap_walk_off_arrows_clears_them_like_keys() {
        let mut keys = arrows_scene(3, 6);
        key_walk(&mut keys, -3, (2, ROW));
        assert!(!keys.path_arrows_visible, "参照：方向键走离箭头 ⇒ 箭头熄灭（label216）");
        let mut touch = arrows_scene(3, 6);
        tap_walk(&mut touch, (2, ROW));
        assert_eq!((touch.player_cell_x, touch.player_cell_y), (2, ROW));
        assert_eq!(touch.path_arrows_visible, keys.path_arrows_visible, "点地 = 方向键");
        assert!(touch.path_arrow_cells().is_empty());
    }

    #[test]
    fn bug043_interrupted_tap_walk_keeps_remaining_arrows() {
        // 沿箭头点地走、途中按键打断（BUG-040 收尾）：已走的格消费，余下箭头仍按原路线画
        let original = arrows_scene(1, 6).path_arrow_cells();
        let mut sc = arrows_scene(1, 6);
        let wt = sc.width_table.clone();
        assert!(sc.walk_to(6, ROW).reached);
        let mut fc = 0;
        while sc.player_cell_x < 3 && fc < 400 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        sc.press_key(-1); // 上 = 撞墙，打断寻路
        sc.release_key();
        for _ in 0..60 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(!sc.route.is_open());
        let cells = sc.path_arrow_cells();
        assert!(sc.path_arrows_visible, "沿箭头走被打断：箭头仍在（方向键沿箭头走同样保留）");
        assert!(original.ends_with(&cells), "余下箭头 {cells:?} 须是原路线尾段");
        assert_eq!(cells.first().copied(), Some((sc.player_cell_x + 1, ROW)), "余下箭头紧接玩家格");
    }

    /// 箭头相关全部状态（对照用）：可见、绘制格、暂存是否已清、walkPathBuffer 有效段（label216 / 绘制的数据源）。
    fn arrow_state(sc: &GameScene) -> (bool, Vec<(i32, i32)>, bool, Vec<u8>) {
        let n = sc.walk_step_count.max(0) as usize;
        (sc.path_arrows_visible, sc.path_arrow_cells(), sc.tap.walkto_arrows.is_none(), sc.walk_path_buffer[..n].to_vec())
    }

    /// 复核 bug043-R1：到达 / 打断两个出口收尾后，暂存已清、walkPathBuffer 有效段与方向键参照相同（不只是"画出来一样"）。
    #[test]
    fn bug043_exits_restore_buffer_like_keys() {
        let mut keys = arrows_scene(1, 6);
        key_walk(&mut keys, -4, (3, ROW));
        let mut arrived = arrows_scene(1, 6);
        tap_walk(&mut arrived, (3, ROW));
        assert_eq!(arrow_state(&arrived), arrow_state(&keys), "到达");
        assert!(arrow_state(&arrived).2, "到达：暂存已清");
        // 打断：寻路走到 3 时按"上"（撞墙），收尾停在 3
        let mut cut = arrows_scene(1, 6);
        let wt = cut.width_table.clone();
        assert!(cut.walk_to(6, ROW).reached);
        let mut fc = 0;
        while cut.player_cell_x < 3 && fc < 400 {
            cut.tick(fc, &wt);
            fc += 1;
        }
        cut.press_key(-1);
        cut.release_key();
        for _ in 0..60 {
            cut.tick(fc, &wt);
            fc += 1;
        }
        let at = (cut.player_cell_x, cut.player_cell_y);
        let mut keys2 = arrows_scene(1, 6);
        key_walk(&mut keys2, -4, at);
        assert_eq!(arrow_state(&cut), arrow_state(&keys2), "打断于 {at:?}");
    }

    /// 复核 bug043-R1：新路径（SEE / MOV 的 findPath）取代暂存——寻路途中剧情重算路径后不得再画旧箭头。
    #[test]
    fn bug043_find_path_drops_stash() {
        let mut sc = arrows_scene(1, 6);
        assert!(sc.walk_to(4, ROW).reached);
        assert!(sc.tap.walkto_arrows.is_some(), "前置：箭头已暂存");
        assert!(sc.find_path(1, ROW, 2, ROW));
        assert!(sc.tap.walkto_arrows.is_none(), "findPath 后暂存作废");
        assert_eq!(sc.path_arrow_cells(), vec![(2, ROW)], "箭头 = 新路径");
    }

    /// 复核 bug043-R2：点地走到剧情触发格（型 83，撞格触发）前一格并撞进去 = 方向键走过去再撞进去。
    #[test]
    fn bug043_bump_into_scene_trigger_like_keys() {
        let make = || {
            let mut sc = scene((1, ROW), &[(83, 5, ROW)]);
            apply_setup(&mut sc, SceneSetup { dialog_dialog_data_scripts: Some(vec!["TAK_0_0 ".to_string()]), ..Default::default() });
            assert!(sc.find_path(1, ROW, 6, ROW) || sc.find_path(1, ROW, 4, ROW), "前置：箭头路径");
            sc.path_arrows_visible = true;
            sc
        };
        let mut keys = make();
        key_walk(&mut keys, -4, (4, ROW));
        let wt = keys.width_table.clone();
        keys.press_key(-4); // 撞 83 ⇒ 剧情
        for fc in 0..20 {
            keys.tick(fc, &wt);
        }
        keys.release_key();
        let mut touch = make();
        let wt2 = touch.width_table.clone();
        let _ = touch.walk_to(5, ROW);
        for fc in 0..200 {
            touch.tick(fc, &wt2);
            if touch.game_mode == GameMode::Dialog && !touch.route.is_open() && touch.dialog.dialog_phase != 3 {
                break;
            }
        }
        for fc in 200..220 {
            touch.tick(fc, &wt2);
        }
        assert_eq!(keys.game_mode, GameMode::Dialog, "参照：撞 83 进剧情");
        assert_eq!(touch.game_mode, GameMode::Dialog, "点地：撞 83 进剧情");
        assert_eq!((touch.player_cell_x, touch.player_cell_y), (keys.player_cell_x, keys.player_cell_y));
        assert_eq!(arrow_state(&touch), arrow_state(&keys), "撞格触发剧情后的箭头状态");
    }

    /// 复核 bug043-R3：箭头一路指到锁门（无钥匙），点地沿箭头走到门前并撞门 = 方向键沿箭头走过去再撞门：
    /// 原版撞门清共用 walkStepCount ⇒ 余下箭头不再画。多步寻路的末步撞门走 advanceScriptWalk（步末连走）臂，
    /// 不经"浮层撞格熄箭头"臂——暂存须在撞门处同清。
    #[test]
    fn bug043_bump_locked_door_like_keys() {
        let make = || {
            let mut sc = scene((1, ROW), &[(1, 4, ROW)]); // 型 1 = 黄门（无钥匙）
            assert!(sc.find_path(1, ROW, 4, ROW), "前置：箭头路径 (1)→门(4)");
            sc.path_arrows_visible = true;
            assert_eq!(sc.path_arrow_cells(), vec![(2, ROW), (3, ROW), (4, ROW)]);
            sc
        };
        let mut keys = make();
        key_walk(&mut keys, -4, (3, ROW));
        assert_eq!(keys.path_arrow_cells(), vec![(4, ROW)], "参照前置：走到门前余下门格箭头");
        let wt = keys.width_table.clone();
        keys.press_key(-4); // 撞门
        keys.tick(500, &wt);
        keys.release_key();
        let mut touch = make();
        let wt2 = touch.width_table.clone();
        let _ = touch.walk_to(4, ROW);
        for fc in 0..200 {
            touch.tick(fc, &wt2);
            if touch.dialog.overlay_active {
                break;
            }
        }
        assert!(keys.dialog.overlay_active && touch.dialog.overlay_active, "两边都弹出\"没有钥匙\"提示");
        assert_eq!((touch.player_cell_x, touch.player_cell_y), (3, ROW), "停在门前");
        assert!(keys.path_arrow_cells().is_empty(), "参照：方向键撞门后余下箭头不再画（原版清 walkStepCount）");
        // 关掉提示后再走几拍：两边一致、箭头不复现
        for sc in [&mut keys, &mut touch] {
            sc.dialog.overlay_active = false;
            let wt = sc.width_table.clone();
            for fc in 600..640 {
                sc.tick(fc, &wt);
            }
        }
        assert!(touch.tap.walkto_arrows.is_none(), "暂存已清");
        assert_eq!(touch.path_arrow_cells(), keys.path_arrow_cells(), "点地撞门 = 方向键撞门");
    }

    #[test]
    fn t2_interrupt_all_channels_clear_buffer() {
        // 打断全通道：press_key（方向键/软键同口）/新 WalkTo/探步入 皆清 walkPathBuffer
        let mut sc = scene((1, ROW), &[]);
        sc.walk_to(5, ROW);
        assert!(sc.walk_step_count > 0);
        sc.press_key(-4); // 任意输入（方向键）
        assert_eq!(sc.walk_step_count, 0, "press_key 打断=清 walkPathBuffer 停步");
        assert_eq!(sc.undo.len(), 0, "中断段于起点（未动）= 零长段，不入栈（STAGE-B-008）");

        sc.walk_to(5, ROW);
        assert!(sc.walk_step_count > 0);
        let plan2 = sc.walk_to(2, ROW); // 新触点=打断旧路线（手动优先）
        assert_eq!(sc.walk_step_count, 1, "新 WalkTo 重设路线");
        assert_eq!(plan2.dock, (2, ROW));
        sc.probe_step(0); // 探步入=打断
        assert_eq!(sc.walk_step_count, 0, "探步入打断=清 walkPathBuffer");
    }

    #[test]
    fn t2_classic_script_walk_unaffected_by_press_key() {
        // 保真红线：经典脚本行走（op_mov 路）press_key 不清缓冲（route 段未开放）
        let mut sc = scene((1, ROW), &[]);
        sc.find_path(1, ROW, 5, ROW); // MOV 同口（findPath 喂缓冲）
        let before = sc.walk_step_count;
        assert!(before > 0);
        sc.press_key(-4);
        assert_eq!(sc.walk_step_count, before, "经典面 key 链一行不动（缓冲不清）");
    }

    /// 【2026-10-26 用户实机"跳过教程后，嫦娥脚本行为异常"】壳层投键口径 = 每次按键/抬键前先
    /// `interrupt_route()`（android replay_scene_keys / win route_key / 触屏 Dispatcher）。
    /// 无 WalkTo 段时它必须零操作——旧实现无条件清 walkPathBuffer/walkStepCount ⇒ 剧情里
    /// 连点"确定"截断 MOV 实体路径（dp2）与玩家脚本行走（dp3）：嫦娥停在半路、后续
    /// `MOV_84_x_y_…` 找不到她而跳过、`DES_84_10_1` 落空（Java keyPressed 只写
    /// keyValue/keyHeldCode，a.java:4663-4676）。
    #[test]
    fn t2_interrupt_route_without_open_route_keeps_script_walk() {
        let mut sc = scene((1, ROW), &[]);
        sc.find_path(1, ROW, 5, ROW); // 脚本 MOV 同口
        let (steps, buf) = (sc.walk_step_count, sc.walk_path_buffer.clone());
        assert!(steps > 0 && !sc.route.is_open());
        sc.interrupt_route();
        assert_eq!((sc.walk_step_count, &sc.walk_path_buffer), (steps, &buf), "无寻路段：打断=零操作");
        // 寻路段被战斗/拾取挂起时（段已收、待续走），任意输入仍须打断续走
        sc.walk_to(5, ROW);
        sc.route_event_suspend();
        assert!(!sc.route.is_open() && sc.route.is_suspended() && sc.walk_step_count > 0);
        sc.interrupt_route();
        assert_eq!(sc.walk_step_count, 0, "挂起中的寻路余步：打断清除");
        assert!(!sc.route.is_suspended(), "打断=挂起解除");
    }

    /// 同一 bug 的第二入口：触屏点格寻路（WalkTo）走进剧情点 ⇒ 剧情接管时寻路段仍开放 ⇒
    /// 剧情中点"确定"（press_key 见段开放即打断）照样截断脚本行走，且 dp3 步尽臂
    /// `walk_step_count==0 && route.is_open()` 会把剧情中途切回 Map。剧情接管必须收束寻路段。
    #[test]
    fn t2_script_takeover_closes_walkto_route() {
        let mut sc = scene((1, ROW), &[]);
        apply_setup(&mut sc, SceneSetup { dialog_dialog_data_scripts: Some(vec!["TAK_0_0 ".to_string()]), ..Default::default() });
        sc.walk_to(5, ROW);
        assert!(sc.route.is_open());
        sc.load_level_script(0);
        assert!(!sc.route.is_open() && !sc.route.is_suspended(), "剧情接管 = 寻路段收束");
        sc.find_path(1, ROW, 5, ROW); // 剧情 MOV
        let steps = sc.walk_step_count;
        sc.press_key(-5);
        sc.interrupt_route();
        assert_eq!(sc.walk_step_count, steps, "剧情行走不受投键打断");
    }

    /// 【BUG-047】探步按探步方向走，不按旧朝向走：{旧朝向 × 4} × {探步方向 × 4}。
    /// 走廊 y=ROW 横向可走、上下是墙 ⇒ 右探到 (6,ROW)、左探到 (4,ROW)、上下探原地不动；朝向 = 探步方向。
    #[test]
    fn bug047_probe_step_moves_in_probe_direction_for_every_old_facing() {
        for old in 0..4 {
            for dir in 0u8..4 {
                let mut sc = scene((5, ROW), &[]);
                let wt = sc.width_table.clone();
                sc.facing = old;
                let accepted = sc.probe_step(dir);
                assert_eq!(accepted, matches!(dir, 2 | 3), "旧朝向 {old}、探步方向 {dir}：返回值 = 是否放行（壳层滑动撞墙即停的依据）");
                for fc in 0..20 {
                    sc.tick(fc, &wt);
                }
                let want = match dir {
                    2 => (6, ROW),
                    3 => (4, ROW),
                    _ => (5, ROW),
                };
                assert_eq!(
                    (sc.player_cell_x, sc.player_cell_y),
                    want,
                    "旧朝向 {old}、探步方向 {dir}：终点须是探步方向邻格或原地"
                );
                assert_eq!(
                    (sc.view.player_px, sc.view.player_py),
                    (want.0 << 5, want.1 << 5),
                    "旧朝向 {old}、探步方向 {dir}：像素位置与格一致（不得走进墙格半途）"
                );
                assert_eq!(sc.facing, dir as i32, "旧朝向 {old}、探步方向 {dir}：朝向 = 探步方向");
            }
        }
    }

    /// 【BUG-047】炼丹炉只在朝上走入时打开（scene.rs:5329-5341，5333 判 facing == 1）：
    /// 旧朝向为上、向右横向探步撞炉，不得打开炼丹炉页，玩家不动。
    #[test]
    fn bug047_sideways_probe_into_alchemy_with_old_facing_up_does_not_open_it() {
        let mut sc = scene((5, ROW), &[(9, 6, ROW)]);
        let wt = sc.width_table.clone();
        sc.facing = 1;
        let _ = sc.probe_step(2);
        for fc in 0..20 {
            sc.tick(fc, &wt);
        }
        assert_ne!(sc.game_mode, GameMode::Alchemy, "横向撞炉不进炼丹炉");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "被炉挡住，原地");
    }

    /// 【BUG-047】正向：旧朝向为下、向上探步撞炉（炉在正上方）⇒ 打开炼丹炉页（判的是探步方向，不是旧朝向）。
    #[test]
    fn bug047_upward_probe_into_alchemy_with_old_facing_down_opens_it() {
        let mut sc = scene((5, ROW), &[(9, 5, ROW - 1)]);
        apply_setup(&mut sc, SceneSetup { facing: Some(0), ..Default::default() });
        let _ = sc.probe_step(1);
        assert_eq!(sc.game_mode, GameMode::Alchemy, "朝上撞炉进炼丹炉");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "进炉不移动玩家");
    }

    /// 【BUG-047】探步 ≡ 方向键（复核 R2）：撞带钥匙的黄门、撞可胜的怪，探步与按方向键走同一格的结果逐字段一致。
    /// 期望值取方向键路径（handle_field_input，a.java:5308-5329 同式），不依赖 probe_step。
    #[test]
    fn bug047_probe_step_matches_direction_key_on_doors_and_enemies() {
        fn outcome(sc: &GameScene) -> ((i32, i32), (i32, i32), i32, i32, i32, Vec<(i32, u8, bool)>) {
            let ents = (0..sc.entities.count)
                .map(|i| (sc.entities.items[i].type_, sc.entities.items[i].state, sc.entities.items[i].removed))
                .collect();
            (
                (sc.player_cell_x, sc.player_cell_y),
                (sc.view.player_px, sc.view.player_py),
                sc.facing,
                sc.player.yellow_keys,
                sc.player.hp,
                ents,
            )
        }
        let make = |t: u8, old: i32| {
            let mut sc = scene((5, ROW), &[(t, 6, ROW)]);
            apply_setup(&mut sc, SceneSetup { player_yellow_keys: Some(1), player_atk: Some(100_000), player_def: Some(100_000), facing: Some(old), ..Default::default() });
            sc
        };
        for (t, label) in [(1u8, "黄门（带钥匙）"), (43, "可胜的怪")] {
            for old in 0..4 {
                let mut keys = make(t, old);
                let wt = keys.width_table.clone();
                keys.press_key(-4);
                keys.tick(0, &wt);
                keys.release_key();
                for fc in 1..300 {
                    keys.tick(fc, &wt);
                }
                let mut probe = make(t, old);
                let _ = probe.probe_step(2);
                for fc in 0..300 {
                    probe.tick(fc, &wt);
                }
                assert_eq!(outcome(&probe), outcome(&keys), "{label}、旧朝向 {old}：探步 = 方向键");
            }
        }
    }

    /// 【BUG-049】Dialog 模式（mode 11）下弹出的提示框必须画出来（Java paint 公共尾对所有模式共跑，a.java:3129-3168）。
    /// 实机发现：点地寻路首步撞打不过的怪 ⇒ 拒战提示已激活（吞键），但画面上看不到。
    #[test]
    fn bug049_overlay_is_painted_in_dialog_mode() {
        let paint_once = |overlay: bool| {
            let mut sc = scene((5, ROW), &[]);
            apply_setup(&mut sc, SceneSetup { game_mode: Some(GameMode::Dialog), dialog_dialog_phase: Some(3), ..Default::default() });
            if overlay {
                sc.show_overlay_message(0, "你无法战胜它", 0, 0);
            }
            let mut canvas = crate::render::ArgbImage::create(240, 320);
            {
                let mut g = crate::render::SoftGraphics::new(&mut canvas);
                sc.paint(&mut g, &crate::render::ArgbImage::create(2, 2));
            }
            (canvas.argb, sc.dialog.overlay_box_w, sc.dialog.overlay_box_h)
        };
        let (plain, _, _) = paint_once(false);
        let (with_box, bw, bh) = paint_once(true);
        assert!(bw > 0 && bh > 0, "前置：提示框有尺寸");
        // 盒框居中（paint_overlay_box：bx = (screen_w - w) >> 1）；框内必须有像素与无框画面不同
        let (bx, by) = ((240 - bw) >> 1, (320 - bh) >> 1);
        let changed = (by..by + bh)
            .flat_map(|y| (bx..bx + bw).map(move |x| (x, y)))
            .filter(|&(x, y)| plain[(y * 240 + x) as usize] != with_box[(y * 240 + x) as usize])
            .count();
        assert!(changed > 0, "Dialog 模式下提示框区域须被绘制");
    }

    /// 【BUG-044】点地寻路首步撞上不弹浮层的阻挡：玩家不得进入该格，寻路收尾回 Map。
    /// find_path 经实体槽规划路线（walk_to(7,ROW) 自 (5,ROW) 起两步，首步即 (6,ROW)）。
    #[test]
    fn bug044_tap_walk_first_step_into_blocker_stays_put() {
        let cases: &[(u8, &str)] = &[(9, "炼丹炉（横向撞）"), (12, "隐形墙"), (70, "打不过的怪")];
        for &(t, label) in cases {
            let mut sc = scene((5, ROW), &[(t, 6, ROW)]);
            if t == 70 {
                sc.player.hp = 1;
            }
            let plan = sc.walk_to(7, ROW);
            assert!(!plan.steps.is_empty(), "{label}：前置——find_path 经实体槽规划出路线");
            run_until_settled(&mut sc, 300);
            if t == 70 {
                // 【BUG-046】打不过 ⇒ 原版拒战提示框（a.java:5487-5490）；关框后寻路收尾
                assert!(sc.dialog.overlay_active, "{label}：弹拒战提示框");
                assert_eq!(sc.dialog.overlay_text.as_deref(), Some("你无法战胜它"), "{label}：提示文案");
                assert_eq!((sc.game_mode, sc.dialog.dialog_phase), (GameMode::Dialog, 3), "{label}：弹框期间仍归点地寻路的 dp3");
                assert!(sc.route.is_open(), "{label}：弹框期间路段开着（关框键属浮层不打断；步序已清，关框后 finish_walkto 收尾）");
                let wt = sc.width_table.clone();
                sc.press_key(-5);
                sc.tick(1000, &wt);
                sc.release_key();
                run_until_settled(&mut sc, 300);
                for fc in 1001..1010 {
                    sc.tick(fc, &wt);
                }
            }
            assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "{label}：首步被挡，玩家留在原格");
            assert_eq!((sc.view.player_px, sc.view.player_py), (5 << 5, ROW << 5), "{label}：像素位置未动");
            assert_eq!(sc.game_mode, GameMode::Map, "{label}：寻路收尾回 Map");
            assert!(!sc.route.is_open(), "{label}：路段已收");
            assert!(!sc.script_walk_armed && !sc.walkto_draining, "{label}：无寻路武装 / 排空残留");
            assert_ne!(sc.walk_phase, 5, "{label}：未开战");
        }
    }

    /// 【BUG-044】可胜的怪仍照常开战（返回 false 但 wp=5 的分支不得被当成"被挡"清掉）。
    #[test]
    fn bug044_tap_walk_first_step_into_winnable_enemy_still_fights() {
        let mut sc = scene((5, ROW), &[(41, 6, ROW)]);
        apply_setup(&mut sc, SceneSetup { player_atk: Some(100_000), player_def: Some(100_000), ..Default::default() });
        sc.player.hp = 500; // 夹具默认 hp=0 ⇒ 预测 loss(0) < hp 不成立，会被判"打不过"
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut saw_battle = false;
        for fc in 0..40 {
            sc.tick(fc, &wt);
            saw_battle |= sc.walk_phase == 5;
        }
        assert!(saw_battle, "首步撞可胜的怪 ⇒ 开战（walk_phase 5）");
    }

    /// 【BUG-044】保真钉：经典脚本行走（MOV 驱动的 dp3，路段未开）首步撞阻挡仍按原版不判返回值
    /// （a.java:4080-4084）——修复分支只对点地寻路生效（walkto_owns_dp3）。去掉该门控的变异须在此变红。
    #[test]
    fn bug044_classic_script_walk_keeps_original_unchecked_first_step() {
        let mut sc = scene((5, ROW), &[(9, 6, ROW)]);
        let wt = sc.width_table.clone();
        // dp3 消费机的经典入口状态（op_mov 同款：路径缓冲 + 武装 + Dialog dp3 wp0），不开路段
        sc.walk_path_buffer = vec![2];
        sc.walk_step_count = 1;
        sc.script_walk_armed = true;
        sc.dialog.dialog_phase = 3;
        sc.walk_phase = 0;
        sc.game_mode = GameMode::Dialog;
        assert!(!sc.route.is_open(), "前置：经典脚本行走不开路段");
        for fc in 0..12 {
            sc.tick(fc, &wt);
        }
        assert_eq!(
            (sc.player_cell_x, sc.player_cell_y),
            (6, ROW),
            "经典 MOV 首步不判碰撞（原版行为，Stage A 保真）"
        );
    }

    /// 一拍 + 绘制（wp4 开门动画由 paint 推进，advance_open_group_paint）。
    fn tick_paint(sc: &mut GameScene, fc: i64) {
        let wt = sc.width_table.clone();
        sc.tick(fc, &wt);
        let mut canvas = crate::render::ArgbImage::create(240, 320);
        let mut g = crate::render::SoftGraphics::new(&mut canvas);
        sc.paint(&mut g, &crate::render::ArgbImage::create(2, 2));
    }

    /// 【BUG-044】复核 T2-2：首步撞"撞格即触发"的剧情点（型 83，空脚本立即收尾）——剧情接管先收段，
    /// 门控须按弹步时的归属判定；终格 = 方向键撞同一格的终格（停在原格）。
    #[test]
    fn bug044_first_step_into_scene_trigger_matches_direction_key() {
        let make = || {
            let mut sc = scene((4, ROW), &[(83, 5, ROW)]);
            apply_setup(&mut sc, SceneSetup { dialog_dialog_data_scripts: Some(vec![String::new()]), ..Default::default() });
            sc
        };
        let mut keys = make();
        let wt = keys.width_table.clone();
        keys.press_key(-4);
        keys.tick(0, &wt);
        keys.release_key();
        for fc in 1..120 {
            keys.tick(fc, &wt);
        }
        let mut touch = make();
        let _ = touch.walk_to(6, ROW);
        for fc in 0..120 {
            touch.tick(fc, &wt);
        }
        assert_eq!(
            (touch.player_cell_x, touch.player_cell_y, touch.view.player_px),
            (keys.player_cell_x, keys.player_cell_y, keys.view.player_px),
            "首步撞剧情点：点地终格 = 方向键终格"
        );
        assert_eq!((touch.player_cell_x, touch.player_cell_y), (4, ROW), "参照：方向键撞剧情点停在原格");
    }

    /// 【BUG-044】踩 60 邻格扣血（step_triggered 短路 advance_script_walk）后的下一步经 dp3 wp0：撞炉须被挡。
    #[test]
    fn bug044_step_after_trap_damage_into_blocker_stays_put() {
        let mut sc = scene((1, ROW), &[(60, 2, ROW + 1), (9, 3, ROW)]);
        apply_setup(&mut sc, SceneSetup { player_hp: Some(1000), ..Default::default() });
        let _ = sc.walk_to(5, ROW);
        let mut hp_dropped = false;
        for fc in 0..200 {
            tick_paint(&mut sc, fc);
            hp_dropped |= sc.player.hp < 1000;
        }
        assert!(hp_dropped, "前置：踩 60 邻格扣血");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "扣血后的下一步撞炉被挡，停在 (2,ROW)");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(!sc.route.is_open());
    }

    /// 【BUG-044】踏板 76 开组门（wp4 开门动画，paint 推进）后的下一步经 dp3 wp0：撞炉须被挡。
    #[test]
    fn bug044_step_after_open_group_animation_into_blocker_stays_put() {
        let mut sc = scene((1, ROW), &[(76, 2, ROW), (4, 10, ROW), (9, 3, ROW)]);
        let door = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 4).expect("型 4");
        sc.entities.items[door].param = 1; // 组号 = 76 的 param&0xff + 1 = 1
        let _ = sc.walk_to(5, ROW);
        let mut saw_wp4 = false;
        for fc in 0..200 {
            tick_paint(&mut sc, fc);
            saw_wp4 |= sc.walk_phase == 4;
        }
        assert!(saw_wp4, "前置：踩 76 进 wp4 开门动画");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "wp4 后的下一步撞炉被挡，停在 (2,ROW)");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(!sc.route.is_open());
    }

    /// 【BUG-044】复核 T2-1：炉在正上方、点炉格（首步朝上撞炉 ⇒ 炉页）：不得残留寻路武装——出页后点地可用、方向键一步一格。
    /// debug 构建下 Dialog → Alchemy 未登记转移边会触发 transition_to 的 debug_assert（点地路径借 dp3 进炉，计划 2 改为 Map wp0 撞击后消除），
    /// 故本钉只在 release（gates rust-tests 口径）运行。
    #[test]
    #[cfg_attr(debug_assertions, ignore = "Dialog→Alchemy 未登记转移边（debug_assert）；计划 2 改 Map wp0 撞击后取消")]
    fn bug044_tap_alchemy_above_leaves_no_armed_walk() {
        let mut sc = scene((5, ROW), &[(9, 5, ROW - 1)]);
        let wt = sc.width_table.clone();
        sc.walkable[(ROW - 1) as usize][5] = true; // 夹具走廊只有一行：把炉格地形置可走，让 walk_to 直达炉格（真资产 1 层 (8,1) 同形）
        let plan = sc.walk_to(5, ROW - 1);
        assert_eq!(plan.steps, vec![1u8], "前置：一步朝上进炉格");
        for fc in 0..10 {
            sc.tick(fc, &wt);
        }
        assert_eq!(sc.game_mode, GameMode::Alchemy, "前置：进炉页");
        assert!(!sc.script_walk_armed, "炉页中无寻路武装残留");
        assert!(!sc.route.is_open(), "路段已收");
        sc.press_key(-7);
        sc.tick(10, &wt);
        sc.release_key();
        for fc in 11..20 {
            sc.tick(fc, &wt);
        }
        assert_eq!(sc.game_mode, GameMode::Map, "前置：出炉页");
        assert!(sc.field_tap_allowed(), "出页后点地可用");
        sc.press_key(-4);
        sc.tick(20, &wt);
        sc.release_key();
        for fc in 21..80 {
            sc.tick(fc, &wt);
        }
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "方向键一步只走一格");
    }

    /// 【BUG-045④】换层 / 存档 / 剧情接管后撤销栈清空（undo_journal.rs 清栈边界契约的生产接线）。
    #[test]
    fn bug045_journal_clears_on_floor_change_save_and_script() {
        use crate::undo_journal::MoveStroke;
        let stroke = MoveStroke { from: (1, ROW), to: (2, ROW) };
        // 换层（load_floor_data = changeFloor / LAY 的原地重载）
        let mut sc = scene((5, ROW), &[]);
        sc.undo.push_move(stroke);
        sc.load_floor_data(&super::scene_test_support::maplv(), 5 << 5, ROW << 5, &[], 2).unwrap();
        assert!(sc.undo.is_empty(), "换层清栈");
        // 剧情接管
        let mut sc = scene((5, ROW), &[]);
        apply_setup(&mut sc, SceneSetup { dialog_dialog_data_scripts: Some(vec!["TAK_0_0 ".to_string(); 2]), ..Default::default() });
        sc.undo.push_move(stroke);
        sc.load_level_script(0);
        assert!(sc.undo.is_empty(), "剧情清栈");
        // 存档（mode 7 确认键）
        let mut sc = scene((5, ROW), &[]);
        let wt = sc.width_table.clone();
        sc.undo.push_move(stroke);
        sc.menu.menu_return_mode = GameMode::InGameMenu;
        sc.game_mode = GameMode::SaveSlot;
        sc.enter_menu7();
        sc.press_key(-5);
        sc.tick(0, &wt);
        sc.release_key();
        assert!(sc.menu.slot.valid[sc.menu.slot.cursor as usize], "前置：存档写入");
        assert!(sc.undo.is_empty(), "存档清栈");
    }

    /// 撤销栈里是否有战斗笔（逐笔弹出检查；只读状态副本的空目标）。
    fn journal_has_combat(sc: &mut GameScene) -> bool {
        struct Nop;
        impl crate::undo_journal::UndoTarget for Nop {
            fn set_player_tile(&mut self, _: (i32, i32)) {}
            fn set_player_vitals(&mut self, _: crate::undo_journal::PlayerVitals) {}
            fn revive_enemy(&mut self, _: crate::undo_journal::EnemyRestore) {}
            fn set_item_count(&mut self, _: i32, _: i32) {}
            fn set_door_state(&mut self, _: i32, _: i32, _: crate::undo_journal::DoorState) {}
        }
        let mut j = std::mem::take(&mut sc.undo);
        let mut found = false;
        while let crate::undo_journal::UndoOutcome::Undone(s) = j.undo(&mut Nop) {
            found |= matches!(s, crate::undo_journal::Stroke::Combat(_));
        }
        found
    }

    /// 【BUG-045 · 复核 T3-R1】击杀带不可逆生成（boss 序 68 ⇒ spawn_boss_event 生成掉落）：不入战斗笔、清栈——
    /// 否则"击杀 → 撤销"循环复制掉落（16 次后槽表越界 panic）。对照：普通怪 41 照常入战斗笔。
    #[test]
    fn bug045_kill_with_spawned_drops_pushes_no_combat_stroke() {
        let fight = |t: u8| {
            let mut sc = scene((5, ROW), &[(t, 6, ROW)]);
            apply_setup(&mut sc, SceneSetup { player_atk: Some(1_000_000), player_def: Some(1_000_000), player_hp: Some(1000), ..Default::default() });
            let count0 = sc.entities.count;
            let wt = sc.width_table.clone();
            let _ = sc.walk_to(6, ROW);
            let mut saw_battle = false;
            let mut fc = 0;
            while fc < 600 {
                sc.tick(fc, &wt);
                fc += 1;
                saw_battle |= sc.walk_phase == 5;
                if saw_battle && sc.battle_target < 0 {
                    break;
                }
            }
            assert!(saw_battle && sc.battle_target < 0, "型 {t}：前置——开战并结束");
            let spawned = sc.entities.count > count0;
            (spawned, journal_has_combat(&mut sc))
        };
        let (spawned68, combat68) = fight(68);
        assert!(spawned68, "前置：型 68 击杀生成掉落");
        assert!(!combat68, "有不可逆生成 ⇒ 栈里没有战斗笔");
        let (spawned41, combat41) = fight(41);
        assert!(!spawned41, "前置：型 41 击杀不生成实体");
        assert!(combat41, "对照：普通怪照常入战斗笔");
    }

    /// 【BUG-045 · 复核 T3-R3/R4】实体表重载（跨层探针）与未记账的消费（炼丹 / 商人 / 道具使用）清栈。
    #[test]
    fn bug045_journal_clears_on_reload_and_unrecorded_spending() {
        use crate::undo_journal::MoveStroke;
        let stroke = MoveStroke { from: (1, ROW), to: (2, ROW) };
        // 跨层探针的实体表重载（下标重排）
        let mut sc = scene((5, ROW), &[(41, 8, ROW)]);
        sc.undo.push_move(stroke);
        let f = sc.persist.floor;
        sc.reload_floor_entities(f);
        assert!(sc.undo.is_empty(), "实体表重载清栈");
        // 炼丹购买
        let mut sc = scene((5, ROW), &[]);
        let wt = sc.width_table.clone();
        sc.undo.push_move(stroke);
        sc.player.gold = 1000;
        sc.alchemy_price = 10;
        sc.game_mode = GameMode::Alchemy;
        sc.press_key(-6);
        sc.tick(0, &wt);
        sc.release_key();
        assert_eq!(sc.player.gold, 990, "前置：炼丹扣金");
        assert!(sc.undo.is_empty(), "炼丹清栈");
        // 商人交易（型 78，src 1，flags 2，报价 3 = 50 金买蓝钥匙）
        let mut sc = scene((5, ROW), &[(78, 6, ROW)]);
        sc.undo.push_move(stroke);
        sc.player.gold = 100;
        let m = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 78).expect("型 78");
        sc.entities.items[m].param = (2 << 8) | 3;
        sc.merchant_entity = m as i32;
        let _ = sc.apply_merchant_offer(true);
        assert_eq!((sc.player.gold, sc.player.blue_keys), (50, 1), "前置：商人扣金给钥匙");
        assert!(sc.undo.is_empty(), "商人交易清栈");
        // 道具使用（19 回血）
        let mut sc = scene((5, ROW), &[]);
        sc.undo.push_move(stroke);
        sc.player.item_types[0] = 19;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        sc.use_item_stack(0);
        assert!(sc.undo.is_empty(), "道具使用清栈");
    }

    /// 【BUG-045 · 复核 T3-R6】reattach_to_cell 幂等：未摘槽（死亡动画中）即重登，不重复登记。
    #[test]
    fn bug045_reattach_is_idempotent_when_still_in_cell() {
        let mut sc = scene((5, ROW), &[(41, 8, ROW)]);
        let e = entity::cell_entity(&sc.grid, &sc.entities, 8, ROW, 41);
        assert!(e >= 0);
        let ct = sc.grid.cell_type[ROW as usize][8] as usize;
        let cap = sc.grid.capacity[ct];
        entity::reattach_to_cell(&mut sc.grid, &mut sc.entities, e as usize);
        assert_eq!(sc.grid.capacity[ct], cap, "已在槽表中 ⇒ 不重复登记");
        entity::detach_from_cell(&mut sc.grid, &mut sc.entities, e as usize);
        assert_eq!(sc.grid.capacity[ct], cap - 1, "前置：摘槽");
        entity::reattach_to_cell(&mut sc.grid, &mut sc.entities, e as usize);
        assert_eq!(sc.grid.capacity[ct], cap, "摘槽后重登一次");
        assert!(!sc.entities.items[e as usize].removed);
    }

    /// 【BUG-048】点地撞怪开战、击杀触发剧情（49 层型 70 → load_level_script(35)）：
    /// 剧情接管后寻路段不得被重开，撤销栈保持清空（不补入战斗笔）。
    #[test]
    fn bug048_victory_script_takeover_keeps_route_closed_and_journal_empty() {
        let mut sc = scene((5, ROW), &[(70, 6, ROW)]);
        apply_setup(&mut sc, SceneSetup { persist_floor: Some(49), persist_current_floor: Some(49), dialog_dialog_data_scripts: Some(vec!["TAK_0_0 ".to_string(); 40]), player_atk: Some(100_000), player_def: Some(100_000), player_hp: Some(100_000), ..Default::default() });
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut saw_battle = false;
        let mut fc = 0;
        while fc < 600 {
            sc.tick(fc, &wt);
            fc += 1;
            saw_battle |= sc.walk_phase == 5;
            if saw_battle && sc.battle_target < 0 {
                break;
            }
        }
        assert!(saw_battle, "前置：开战");
        assert!(sc.battle_target < 0, "前置：战斗结束");
        assert!(!sc.route.is_open(), "剧情接管后寻路段不得被重开");
        assert!(sc.undo.is_empty(), "剧情接管 ⇒ 撤销栈已清，不再补入战斗笔");
    }

    /// 【BUG-048】战斗中经壳层按键链打断（壳先 interrupt_route 再 press ⇒ 路段挂起被解除，但未清栈）：战斗照常入栈、撤销不被永久挡住，路段不续走。
    #[test]
    fn bug048_key_interrupt_mid_tap_battle_still_records_combat() {
        let mut sc = scene((5, ROW), &[(41, 6, ROW)]);
        sc.player.atk = 2; // 刚好打得动：多回合战斗，留出按键窗口
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut fc = 0;
        while sc.walk_phase != 5 && fc < 40 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!(sc.walk_phase, 5, "前置：战斗中");
        // 壳层按键链：先 interrupt_route 再 press（native.rs replay / Dispatcher::key——不看路段是否开放，战斗挂起中同样打断）
        sc.interrupt_route();
        sc.press_key(-1); // 上 = 墙：只起打断作用
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        while sc.battle_target >= 0 && fc < 3000 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.battle_target < 0, "前置：战斗结束");
        assert!(!sc.undo.combat_in_progress(), "战斗挂起已解除（撤销不被永久挡住）");
        for _ in 0..60 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!(sc.game_mode, GameMode::Map, "收尾回 Map");
        assert!(!sc.route.is_open() && !sc.script_walk_armed, "路段已收、无武装残留");
        // 栈顶两笔：Move(邻格→敌格)（胜利 restep）在 Combat 之上——撤销先退回邻格，再复活怪物，人怪不重叠（复核 T4-R1）
        struct Rec(Vec<String>);
        impl crate::undo_journal::UndoTarget for Rec {
            fn set_player_tile(&mut self, t: (i32, i32)) {
                self.0.push(format!("tile{t:?}"));
            }
            fn set_player_vitals(&mut self, _: crate::undo_journal::PlayerVitals) {}
            fn revive_enemy(&mut self, _: crate::undo_journal::EnemyRestore) {
                self.0.push("revive".into());
            }
            fn set_item_count(&mut self, _: i32, _: i32) {}
            fn set_door_state(&mut self, _: i32, _: i32, _: crate::undo_journal::DoorState) {}
        }
        let mut rec = Rec(Vec::new());
        let mut j = std::mem::take(&mut sc.undo);
        let _ = j.undo(&mut rec);
        let _ = j.undo(&mut rec);
        assert_eq!(rec.0, vec![format!("tile{:?}", (5, ROW)), "revive".to_string()], "先退回邻格 (5,ROW)，再复活怪");
    }

    /// 弹出撤销栈全部笔，按逆放顺序记录（tile = 玩家落格，revive = 复活怪）。
    fn undo_seq(sc: &mut GameScene) -> Vec<String> {
        struct Rec(Vec<String>);
        impl crate::undo_journal::UndoTarget for Rec {
            fn set_player_tile(&mut self, t: (i32, i32)) {
                self.0.push(format!("tile{t:?}"));
            }
            fn set_player_vitals(&mut self, _: crate::undo_journal::PlayerVitals) {}
            fn revive_enemy(&mut self, _: crate::undo_journal::EnemyRestore) {
                self.0.push("revive".into());
            }
            fn set_item_count(&mut self, _: i32, _: i32) {}
            fn set_door_state(&mut self, _: i32, _: i32, _: crate::undo_journal::DoorState) {}
        }
        let mut rec = Rec(Vec::new());
        let mut j = std::mem::take(&mut sc.undo);
        while let crate::undo_journal::UndoOutcome::Undone(_) = j.undo(&mut rec) {}
        rec.0
    }

    /// 壳层投键口径打断（先 interrupt_route 再 press / release），随后跑到收尾。
    fn shell_key_interrupt_then_settle(sc: &mut GameScene, mut fc: i64) {
        let wt = sc.width_table.clone();
        sc.interrupt_route();
        sc.press_key(-1); // 上 = 墙：只起打断作用
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        for _ in 0..60 {
            sc.tick(fc, &wt);
            fc += 1;
        }
    }

    /// 【STAGE-B-008 · 复核 R2】首步在途（wp1，逻辑格仍是起点）时打断：排空走完这一步，这一格必须可撤。
    /// 收段点 = 在途步的落格（`route_landing_cell`），不是打断瞬间的逻辑格——否则判成零长丢弃，栈空。
    #[test]
    fn stage_b008_interrupt_mid_first_step_keeps_drained_step_undoable() {
        let mut sc = scene((1, ROW), &[]);
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(5, ROW);
        let mut fc = 0;
        while sc.walk_phase != 1 && fc < 10 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!(sc.walk_phase, 1, "前置：首步在途");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (1, ROW), "前置：逻辑格仍在起点");
        shell_key_interrupt_then_settle(&mut sc, fc);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "排空：在途步走完");
        assert_eq!(sc.undo.len(), 1, "走出的一格入栈");
        assert_eq!(undo_seq(&mut sc), vec![format!("tile{:?}", (1, ROW))], "撤销回起点");
    }

    /// 【STAGE-B-008 · 复核 R1】紧邻开战胜利后 restep 在途时打断：撤销序列 = [退回邻格, 复活怪]。
    /// 先复活会把怪压在玩家身上（复核 T4-R1 的重叠）。
    #[test]
    fn stage_b008_interrupt_mid_victory_restep_undoes_step_before_revive() {
        let mut sc = scene((5, ROW), &[(41, 6, ROW)]);
        apply_setup(&mut sc, SceneSetup { player_atk: Some(100_000), player_def: Some(100_000), ..Default::default() });
        sc.player.hp = 500; // 夹具默认 hp=0 ⇒ 判"打不过"
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut saw_battle = false;
        let mut fc = 0;
        while fc < 600 {
            sc.tick(fc, &wt);
            fc += 1;
            saw_battle |= sc.walk_phase == 5;
            if saw_battle && sc.battle_target < 0 && sc.walk_phase == 1 {
                break;
            }
        }
        assert!(saw_battle && sc.battle_target < 0, "前置：开战并胜利");
        assert_eq!(sc.walk_phase, 1, "前置：胜利 restep 在途");
        assert!(sc.route.is_open(), "前置：续走段已开");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "前置：逻辑格仍在邻格");
        shell_key_interrupt_then_settle(&mut sc, fc);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "排空：restep 走进敌格");
        assert_eq!(
            undo_seq(&mut sc),
            vec![format!("tile{:?}", (5, ROW)), "revive".to_string()],
            "先退回邻格 (5,ROW)，再复活怪"
        );
    }

    /// 【BUG-048 · 复核 T4-R2】剧情接管且击杀有生成（49 层型 71 ⇒ spawn_boss_event + load_level_script(35)）：同样不入栈、不续走、战斗挂起已解除。
    #[test]
    fn bug048_script_takeover_with_spawn_keeps_route_closed_and_journal_empty() {
        let mut sc = scene((5, ROW), &[(71, 6, ROW)]);
        apply_setup(&mut sc, SceneSetup { persist_floor: Some(49), persist_current_floor: Some(49), dialog_dialog_data_scripts: Some(vec!["TAK_0_0 ".to_string(); 40]), player_atk: Some(1_000_000), player_def: Some(1_000_000), player_hp: Some(100_000), ..Default::default() });
        let count0 = sc.entities.count;
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut saw_battle = false;
        let mut fc = 0;
        while fc < 600 {
            sc.tick(fc, &wt);
            fc += 1;
            saw_battle |= sc.walk_phase == 5;
            if saw_battle && sc.battle_target < 0 {
                break;
            }
        }
        assert!(saw_battle && sc.battle_target < 0, "前置：开战并结束");
        assert!(sc.entities.count > count0, "前置：击杀生成实体");
        assert!(!sc.route.is_open() && !sc.route.is_suspended(), "剧情接管：路段不重开");
        assert!(!sc.undo.combat_in_progress(), "战斗挂起已解除");
        assert!(sc.undo.is_empty(), "撤销栈为空");
    }

    /// 【BUG-048】回归钉：无剧情钩子的击杀照常入战斗笔并续走（路段在胜利尾重开）。
    #[test]
    fn bug048_plain_victory_still_pushes_combat_and_resumes() {
        let mut sc = scene((5, ROW), &[(41, 6, ROW)]);
        apply_setup(&mut sc, SceneSetup { player_atk: Some(100_000), player_def: Some(100_000), player_hp: Some(100_000), ..Default::default() });
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut saw_battle = false;
        let mut fc = 0;
        while fc < 600 {
            sc.tick(fc, &wt);
            fc += 1;
            saw_battle |= sc.walk_phase == 5;
            if saw_battle && sc.battle_target < 0 {
                break;
            }
        }
        assert!(saw_battle && sc.battle_target < 0, "前置：开战并结束");
        assert!(sc.route.is_open(), "无剧情：胜利尾续走（路段重开，restep 进敌格）");
        assert!(journal_has_combat(&mut sc), "无剧情：栈里有战斗笔");
    }

    /// 【STAGE-B-008 · spec §2.2 保真红线】陷阱谓词与 classic 原块（apply_step_cell_effects）逐格对拍：
    /// 合成摆放 × 视口判界变体 × 全格 × 装备 {0, 40}，"站上该格是否扣血"须一致。原块是裁判（实跑扣血），谓词不得由原块推导。
    /// 视口变体：格表恒为 13×13，覆写 view.wide / view.high 把"视口判界"与"格表越界（entity_at_cell 返 −1）"解耦——
    /// 实机两者同源（CellGrid::new(view.wide, view.high)），不解耦时 61 的 `x < wide - 1` 等判界项被越界兜底吞掉、
    /// 变异杀不死；(12, 13) = 唯一非方形层 floor 51 的真实尺寸（maplv51 头 `1800 1a00`），(13, 12) 是"右邻用 view.high"
    /// 怪癖实际收窄判界的方向。
    #[test]
    fn trap_predicate_matches_step_effects_everywhere() {
        // 摆放：61 夹击（横 / 竖）、62 邻接、60 邻接（含 60 上邻的上爬分支）、贴边（判界怪癖）
        let layouts: &[&[(u8, i32, i32)]] = &[
            &[(61, 4, ROW), (61, 6, ROW)],
            &[(61, 5, ROW - 1), (61, 5, ROW + 1)],
            &[(62, 7, ROW)],
            &[(60, 3, ROW)],
            &[(60, 8, ROW - 1)],
            &[(62, 0, ROW), (60, 12, ROW)],
            &[(61, 0, ROW), (61, 2, ROW), (62, 11, ROW + 1)],
            // 贴右 / 贴下（配合视口变体）：61 横对跨 x = 11、竖对跨 y = 11；62 / 60 在 x = 12 列
            &[(61, 10, ROW), (61, 12, ROW), (61, 5, 10), (61, 5, 12)],
            &[(62, 12, ROW), (60, 12, ROW + 2), (62, 7, 12)],
            // 贴顶（复核 T3-R1）：上邻 / 61 竖夹的 `y > 0` 判界
            &[(62, 7, 0), (60, 9, 0), (61, 3, 0), (61, 3, 2)],
        ];
        for (li, ents) in layouts.iter().enumerate() {
            for view_dims in [None, Some((12, 13)), Some((13, 12))] {
                for armor in [0, 40] {
                    for y in 0..13 {
                        for x in 0..13 {
                            if ents.iter().any(|&(_, ex, ey)| (ex, ey) == (x, y)) {
                                continue;
                            }
                            let mut sc = scene((1, ROW), ents);
                            if let Some((w, h)) = view_dims {
                                sc.view.wide = w;
                                sc.view.high = h;
                            }
                            sc.equipped_armor = armor;
                            let predicted = sc.trap_damage_at(x, y);
                            sc.player.hp = 10_000;
                            sc.view.player_px = x << 5;
                            sc.view.player_py = y << 5;
                            sc.apply_step_cell_effects();
                            let damaged = sc.player.hp < 10_000;
                            assert_eq!(
                                predicted, damaged,
                                "摆放 {li} 视口 {view_dims:?} 装备 {armor} 格 ({x},{y})：谓词 = 原块"
                            );
                        }
                    }
                }
            }
        }
    }

    /// 【STAGE-B-008】陷阱谓词真实数据对拍：assets/raw 全部 maplv{n}+sprite{n}（0..=54）中含型 60/61/62 的层，
    /// 全图每格 × 装备 {0, 40}，谓词 vs 原块实跑扣血。实测覆盖 floor 41–43、45–49（均 13×13；唯一非方形层 51 无陷阱实体）。
    /// 隔离：站立格本身有实体（拾取 / 楼梯 / 宝箱 / 83 踩格 /
    /// 门组等格内效果会改 hp 或换层）的格跳过并计数——陷阱块只读四邻，站立格实体不参与陷阱判定。
    #[test]
    fn trap_predicate_matches_step_effects_on_real_trap_floors() {
        use super::scene_test_support::{images, numbers};
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let raw = |name: &str| {
            std::fs::read(repo.join("assets/raw").join(name)).unwrap_or_else(|e| panic!("缺资产 {name}：{e}"))
        };
        let width_table = numbers(include_str!("../../tests/fixtures/type_width_px.txt"));
        let walk = numbers(include_str!("../../tests/fixtures/tile_walkability.txt"));
        let doors = crate::sprite_spawn::DoorTables {
            door12: numbers(include_str!("../../tests/fixtures/door12.txt")),
            door13: numbers(include_str!("../../tests/fixtures/door13.txt")),
        };
        let mut covered: Vec<(i32, i32, i32, usize, usize, usize)> = Vec::new(); // (层, wide, high, 比较格, 跳过格, 扣血格)
        for n in 0..55i32 {
            let maplv = raw(&format!("maplv{n}"));
            let sp = game_data::SpriteTable::parse(&raw(&format!("sprite{n}"))).unwrap();
            let recs: Vec<(u8, u16, u16, i16, bool)> = sp
                .records
                .iter()
                .map(|r| {
                    let (param, hide) = crate::sprite_spawn::spawn_param_and_visibility(r.type_code, &r.extra, &doors);
                    (r.type_code, r.x, r.y, param, !hide)
                })
                .collect();
            if !recs.iter().any(|r| matches!(r.0, 60..=62)) {
                continue;
            }
            let load = || {
                let mut sc =
                    GameScene::load_floor(&maplv, 0, 0, &recs, &width_table, &walk, images(), vec![None; 56]).unwrap();
                sc.persist.floor = n;
                sc.persist.current_floor = n;
                sc
            };
            let probe = load();
            let (wide, high) = (probe.view.wide, probe.view.high);
            let (mut cmp, mut skip, mut hurt) = (0usize, 0usize, 0usize);
            for y in 0..high {
                for x in 0..wide {
                    let ct = probe.grid.cell_type[y as usize][x as usize] as usize;
                    if probe.grid.capacity[ct] > 0 {
                        skip += 1;
                        continue;
                    }
                    for armor in [0, 40] {
                        let mut sc = load();
                        sc.equipped_armor = armor;
                        let predicted = sc.trap_damage_at(x, y);
                        sc.player.hp = 10_000;
                        sc.view.player_px = x << 5;
                        sc.view.player_py = y << 5;
                        sc.apply_step_cell_effects();
                        let damaged = sc.player.hp < 10_000;
                        assert_eq!(predicted, damaged, "floor {n}（{wide}×{high}）装备 {armor} 格 ({x},{y})：谓词 = 原块");
                        cmp += 1;
                        hurt += damaged as usize;
                    }
                }
            }
            covered.push((n, wide, high, cmp, skip, hurt));
        }
        eprintln!("PROBE 陷阱层对拍（层, wide, high, 比较格次, 跳过实体格, 扣血格次）: {covered:?}");
        assert_eq!(
            covered.iter().map(|c| c.0).collect::<Vec<_>>(),
            vec![41, 42, 43, 45, 46, 47, 48, 49],
            "含型 60/61/62 的层须全部覆盖（复核 T3-R2：44、50–54 无陷阱）"
        );
        assert!(covered.iter().all(|c| c.5 > 0), "每个陷阱层至少有扣血格（防判定全假的空对拍）");
    }

    /// 【STAGE-B-008 · spec §5.1】格分类：不可见实体只看地形（隐形路径 = 墙、隐形墙 / 剧情点 = 地面），
    /// 可见的门 / 商人 / 怪 / 炉按类别，道具格是地面，陷阱邻格标危险，已死 / 开门动画中的实体不挡。
    #[test]
    fn tap_grid_classifies_by_visible_entities_and_terrain() {
        use crate::tap_walk::TargetKind;
        // 走廊 y=ROW、x∈1..=11 可走；其余墙
        let mut sc = scene((1, ROW), &[(6, 2, ROW + 1), (12, 3, ROW), (83, 4, ROW), (1, 5, ROW), (41, 6, ROW), (9, 8, ROW), (13, 11, ROW), (62, 10, ROW - 1)]);
        // 夹具 scene() 的记录一律 visible=true（隐藏位由调用方经 sprite_spawn::spawn_param_and_visibility 给出）；
        // 此处按 m_122 无存档分支语义手动隐藏 6 / 12 / 83（load_floor_data `if !visible` 臂）。
        for i in 0..sc.entities.count {
            if matches!(sc.entities.items[i].type_, 6 | 12 | 83) {
                sc.entities.items[i].visible = false;
            }
        }
        let g = sc.tap_grid();
        let kind = |x: i32, y: i32| g.get(x, y).unwrap().kind;
        assert_eq!(kind(2, ROW + 1), TargetKind::Wall, "未显形隐形路径 = 墙（地形不可走）");
        assert_eq!(kind(3, ROW), TargetKind::Floor, "未显形隐形墙 = 地面（地形可走）");
        assert_eq!(kind(4, ROW), TargetKind::Floor, "剧情点（不可见）= 地面");
        assert_eq!(kind(5, ROW), TargetKind::Interact, "门 = 交互物");
        assert!(matches!(kind(6, ROW), TargetKind::Monster(_)), "怪");
        assert_eq!(kind(8, ROW), TargetKind::Alchemy, "炉（3 格宽：8..=10）");
        assert_eq!(kind(10, ROW), TargetKind::Alchemy, "炉第三格");
        assert_eq!(kind(11, ROW), TargetKind::Floor, "道具格 = 地面");
        assert_eq!(kind(0, ROW), TargetKind::Wall, "地形墙");
        assert!(g.get(10, ROW - 2).map_or(true, |c| !c.danger), "非地面格不标危险（即使紧邻 62）");
        // (3, ROW) 的四邻 (2,ROW)(4,ROW)(3,ROW±1) 都不是 62（62 在 (10, ROW-1)）
        assert!(!g.get(3, ROW).unwrap().danger, "不邻 62 的地面格不危险");
        // 显形后的隐形路径 = 地面；死亡 / 移除的怪不挡
        let i6 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 6).unwrap();
        sc.entities.items[i6].visible = true;
        let i41 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        sc.entities.items[i41].state = 1;
        let g = sc.tap_grid();
        assert_eq!(g.get(2, ROW + 1).unwrap().kind, TargetKind::Floor, "已显形隐形路径 = 地面");
        assert_eq!(g.get(6, ROW).unwrap().kind, TargetKind::Floor, "死亡动画中的怪不挡");
    }

    /// 危险标记：62 上方紧邻的走廊格危险；装备 40 时不危险。
    #[test]
    fn tap_grid_marks_trap_danger() {
        let mut sc = scene((1, ROW), &[(62, 7, ROW - 1)]);
        assert!(sc.tap_grid().get(7, ROW).unwrap().danger, "62 正下方危险");
        assert!(!sc.tap_grid().get(5, ROW).unwrap().danger, "远处不危险");
        sc.equipped_armor = 40;
        assert!(!sc.tap_grid().get(7, ROW).unwrap().danger, "装备 40 全免");
    }

    /// 类目覆盖：装备（类目 4）= 地面；77/78（类目 16）、84 / 非 50 层 72（类目 32）= 交互物；
    /// 50 层 72 = 怪（层覆写，entity_category 读 `self.persist.floor`）；型 69 三格同一 Monster 下标；Monster(e) 指向本格怪。
    #[test]
    fn tap_grid_categories_and_monster_index() {
        use crate::tap_walk::TargetKind;
        let mut sc = scene(
            (1, ROW),
            &[(33, 2, ROW), (77, 3, ROW), (78, 4, ROW), (84, 5, ROW), (72, 6, ROW), (69, 7, ROW), (41, 11, ROW)],
        );
        let g = sc.tap_grid();
        let kind = |x: i32| g.get(x, ROW).unwrap().kind;
        assert_eq!(kind(2), TargetKind::Floor, "装备 33（类目 4）= 地面");
        assert_eq!(kind(3), TargetKind::Interact, "77（类目 16）= 交互物");
        assert_eq!(kind(4), TargetKind::Interact, "78（类目 16）= 交互物");
        assert_eq!(kind(5), TargetKind::Interact, "84（类目 32）= 交互物");
        assert_eq!(kind(6), TargetKind::Interact, "非 50 层 72（类目 32）= 交互物");
        let TargetKind::Monster(e69) = kind(7) else { panic!("69 首格应为怪: {:?}", kind(7)) };
        assert_eq!(sc.entities.items[e69].type_, 69, "Monster 下标指向 69");
        assert_eq!(kind(8), TargetKind::Monster(e69), "69 第二格同一下标（96px = 3 格）");
        assert_eq!(kind(9), TargetKind::Monster(e69), "69 第三格同一下标");
        assert_eq!(kind(10), TargetKind::Floor, "69 只占 3 格");
        let TargetKind::Monster(e41) = kind(11) else { panic!("41 应为怪: {:?}", kind(11)) };
        assert_eq!(sc.entities.items[e41].type_, 41, "Monster 下标指向 41");

        sc.persist.floor = 50;
        sc.persist.current_floor = 50;
        let g = sc.tap_grid();
        let TargetKind::Monster(e72) = g.get(6, ROW).unwrap().kind else {
            panic!("50 层 72 应为怪: {:?}", g.get(6, ROW).unwrap().kind)
        };
        assert_eq!(sc.entities.items[e72].type_, 72, "Monster 下标指向 72");
    }

    /// 同格多实体的优先级 Monster > Alchemy > Interact（两种登记顺序都试：防"后者覆盖"）；道具 + 门 = 交互物。
    #[test]
    fn tap_grid_same_cell_priority() {
        use crate::tap_walk::TargetKind;
        let sc = scene(
            (1, ROW),
            &[
                (41, 2, ROW), (1, 2, ROW), // 怪先登记
                (1, 3, ROW), (41, 3, ROW), // 门先登记
                (9, 4, ROW), (1, 5, ROW), // 炉 4..=6，门叠在炉第二格（炉先）
                (1, 7, ROW), (9, 7, ROW), // 门先，炉 7..=9
                (13, 10, ROW), (1, 10, ROW),
                (1, 11, ROW), (13, 11, ROW),
            ],
        );
        // 前置：同格确实登记了两个槽
        for x in [2, 3, 5, 7, 10, 11] {
            let ct = sc.grid.cell_type[ROW as usize][x] as usize;
            assert_eq!(sc.grid.capacity[ct], 2, "({x}, ROW) 应有 2 个槽");
        }
        let g = sc.tap_grid();
        let kind = |x: i32| g.get(x, ROW).unwrap().kind;
        for x in [2, 3] {
            let TargetKind::Monster(e) = kind(x) else { panic!("({x}) 怪 + 门应为怪: {:?}", kind(x)) };
            assert_eq!(sc.entities.items[e].type_, 41, "({x}) Monster 下标指向 41");
        }
        assert_eq!(kind(5), TargetKind::Alchemy, "炉 + 门（炉先）= 炉");
        assert_eq!(kind(7), TargetKind::Alchemy, "门 + 炉（门先）= 炉");
        assert_eq!(kind(10), TargetKind::Interact, "道具 + 门 = 交互物");
        assert_eq!(kind(11), TargetKind::Interact, "门 + 道具 = 交互物");
    }

    fn settle(sc: &mut GameScene, from: i64, n: i64) {
        let wt = sc.width_table.clone();
        for fc in from..from + n {
            sc.tick(fc, &wt);
        }
    }

    /// 夹具 `scene()` 的记录一律 visible=true：按 m_122 无存档分支语义手动隐藏指定型（同 tap_grid 测试写法）。
    fn hide_types(sc: &mut GameScene, types: &[i32]) {
        for i in 0..sc.entities.count {
            if types.contains(&sc.entities.items[i].type_) {
                sc.entities.items[i].visible = false;
            }
        }
    }

    /// 【STAGE-B-008 · spec §5.2】点可达地面：走过去（BFS），到达后回 Map、目标标记清除。
    #[test]
    fn tap_cell_walks_to_reachable_floor() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Walking);
        assert_eq!(sc.tap.tap_target, Some((7, ROW)), "目标标记");
        settle(&mut sc, 0, 120);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (7, ROW));
        assert_eq!(sc.game_mode, GameMode::Map);
        assert_eq!(sc.tap.tap_target, None, "到达后标记清除");
    }

    /// 点远处的墙：走到墙边可达邻格，再朝墙撞一步（不移动、朝向 = 撞击方向、不留零长撤销笔）。
    #[test]
    fn tap_cell_far_wall_walks_adjacent_then_bumps() {
        let mut sc = scene((2, ROW), &[]);
        let _ = sc.tap_cell(8, ROW - 1);
        settle(&mut sc, 0, 160);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (8, ROW), "停在墙下方");
        assert_eq!(sc.facing, 1, "朝上撞过墙");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert_eq!(sc.undo.len(), 1, "只有一笔 (2,ROW)→(8,ROW) 的位移笔（撞墙零长段不入栈）");
    }

    /// 点门（有钥匙）：走到门旁、撞门开门并走进门格。
    #[test]
    fn tap_cell_door_opens_with_key() {
        let mut sc = scene((2, ROW), &[(1, 6, ROW)]);
        apply_setup(&mut sc, SceneSetup { player_yellow_keys: Some(1), ..Default::default() });
        let _ = sc.tap_cell(6, ROW);
        settle(&mut sc, 0, 200);
        assert_eq!(sc.player.yellow_keys, 0, "耗钥开门");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "走进门格");
    }

    /// 点正上方的炼丹炉：停在炉正下方朝上撞 ⇒ 打开炼丹炉页（Map → Alchemy，不经 Dialog——debug 构建不 panic）。
    #[test]
    fn tap_cell_alchemy_from_below_opens_page() {
        let mut sc = scene((2, ROW), &[(9, 5, ROW - 1)]);
        let _ = sc.tap_cell(6, ROW - 1);
        settle(&mut sc, 0, 160);
        assert_eq!(sc.game_mode, GameMode::Alchemy, "炉页打开");
        assert!(!sc.script_walk_armed && !sc.route.is_open(), "无寻路残留");
    }

    /// 到不了的地面：停在最近可达格，目标格闪红。
    #[test]
    fn tap_cell_unreachable_floor_docks_and_flashes() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        sc.walkable[ROW as usize][7] = false; // 走廊中间砌墙
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Docking);
        assert!(matches!(sc.tap.tap_reject, Some(((9, r), _)) if r == ROW), "目标格闪红");
        settle(&mut sc, 0, 120);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "停在墙前最近格");
    }

    /// 【spec §0 隐藏通路泄露】只能经隐形路径到达的地面：不得走过去、隐形路径不显形（停靠 + 闪红）。
    #[test]
    fn tap_cell_does_not_leak_hidden_path() {
        let mut sc = scene((2, ROW), &[(6, 6, ROW)]);
        hide_types(&mut sc, &[6]); // 未显形的隐形路径
        sc.walkable[ROW as usize][6] = false; // 隐形路径格的地形 = 墙（真实关卡同形）
        let _ = sc.tap_cell(9, ROW);
        settle(&mut sc, 0, 160);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "停在隐形路径前");
        let i6 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 6).unwrap();
        assert!(!sc.entities.items[i6].visible, "隐形路径未显形");
    }

    /// 途中撞上隐形墙（看起来是地面）：被挡停下，撞击后续作废（玩家不在停靠格）。
    #[test]
    fn tap_cell_bump_cancelled_when_route_stops_early() {
        let mut sc = scene((2, ROW), &[(12, 5, ROW)]);
        hide_types(&mut sc, &[12]); // 未显形的隐形墙（tap_grid 视为地面）
        let _ = sc.tap_cell(9, ROW - 1); // 墙：停靠 (9,ROW)，路线途经 (5,ROW) 隐形墙
        settle(&mut sc, 0, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (4, ROW), "撞隐形墙停下");
        assert_ne!(sc.facing, 1, "朝上的撞击已作废");
        assert_eq!(sc.game_mode, GameMode::Map);
    }

    /// 观察模式点按无效；手动行走中点按闪红、状态不变。
    #[test]
    fn tap_cell_ignored_in_observe_and_rejected_while_walking() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        apply_setup(&mut sc, SceneSetup { walk_phase: Some(2), ..Default::default() });
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Ignored);
        sc.walk_phase = 1;
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Rejected);
        assert!(sc.tap.tap_reject.is_some() && !sc.route.is_open());
        sc.walk_phase = 5;
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Rejected, "战斗中闪红");
        assert_eq!(sc.walk_phase, 5, "战斗状态不变");
        sc.walk_phase = 0;
        sc.script_walk_armed = true; // Map 语境脚本行走的步间
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Rejected, "脚本行走中不得插入点地路线");
        assert!(!sc.route.is_open());
    }

    /// 【STAGE-B-008 · 复核 T4-R4】点地路线途经特殊拾取物（型 25：道具栈 + kind 1 带图标提示框，right=3）：
    /// 拾取格步末 advanceScriptWalk 已弹出下一步（wp1），提示框冻结 walkPhase 机——框开时人在拾取格、路段开着、目标标记仍在。
    /// 裁定（spec §5.1【2026-10-30 更正 · 计划 2 T5 复核 R1/R2】）：关框键属浮层（a.java:3252-3345 浮层分支吞键），不是手动行走输入，不打断点地路线
    ///（[`GameScene::interrupt_route_for_input`]）；关框后继续走到目标（"点哪里就往哪里走"）。
    /// 夹具：起点 (2,ROW) → 目标 (9,ROW)，拾取物在 (5,ROW)。`close_key` = 关框所按的键。
    fn pickup_overlay_route(close_key: i32) {
        use crate::tap_walk::{TapResult, TargetKind};
        let mut sc = scene((2, ROW), &[(25, 5, ROW)]);
        assert_eq!(sc.tap_grid().get(5, ROW).unwrap().kind, TargetKind::Floor, "前置：道具格 = 地面（可途经）");
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Walking);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !sc.dialog.overlay_active && fc < 200 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.dialog.overlay_active, "前置：拾取型 25 弹提示框");
        assert_eq!((sc.dialog.overlay_kind, sc.dialog.overlay_right_icon), (1, 3), "前置：kind 1、right=3 拾取框");
        settle(&mut sc, fc, 10); // 框开期间多跑几拍：浮层冻结 walkPhase 机
        fc += 10;
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "框开时人在拾取格");
        assert!(sc.route.is_open(), "框开时路段开着");
        assert_eq!(sc.tap.tap_target, Some((9, ROW)), "框开时目标标记仍在");
        // 真实关框路径：按键 → tick（浮层分支处理该键）→ 松键
        sc.press_key(close_key);
        assert!(sc.route.is_open(), "key={close_key}：按键属浮层，不打断路段");
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        assert!(!sc.dialog.overlay_active, "key={close_key}：提示框已关");
        settle(&mut sc, fc, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (9, ROW), "key={close_key}：关框后续走到终点");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(!sc.route.is_open() && !sc.script_walk_armed && !sc.walkto_draining, "无寻路残留");
        assert_eq!(sc.tap.tap_target, None, "收尾后目标标记清除");
        assert_eq!(undo_seq(&mut sc), vec![format!("tile{:?}", (2, ROW))], "撤销栈 = 整段位移一笔（撤回起点）");
    }

    /// 【STAGE-B-008 · 计划 2 T5 复核问题 B】教程箭头开着时点门（有钥匙）/ 点紧邻可胜怪：路线收尾 `finish_walkto`
    /// 把余下箭头放回 walkPathBuffer（BUG-043，walk_step_count ≠ 0），随后 `execute_bump` 在 Map 语境开撞击段、走进门格 /
    /// 胜利 restep 进敌格。Map wp1 步末收段原条件含 `walk_step_count == 0` ⇒ 段一直开着：目标标记不清；下一次按键
    /// `interrupt_route` 清掉 walkPathBuffer（= 箭头），与方向键走同一路线不一致；下一次点地 `begin_walk_route` 重设段起点，
    /// 撞进去那一格的位移笔丢失。期望：撞进去那一步步末即收段，箭头 / 撤销与方向键参照一致。
    fn bump_with_tutorial_arrows(target_type: u8) {
        let make = || {
            let mut sc = scene((1, ROW), &[(target_type, 6, ROW)]);
            apply_setup(&mut sc, SceneSetup { player_yellow_keys: Some(1), player_atk: Some(100_000), player_def: Some(100_000), player_hp: Some(500), ..Default::default() });
            assert!(sc.find_path(1, ROW, 8, ROW), "前置：箭头 (1)→(8) 经 (6,ROW)");
            sc.path_arrows_visible = true;
            sc
        };
        let label = if target_type == 1 { "黄门" } else { "可胜怪" };
        // 参照：方向键逐格走进 (6,ROW)（门：撞开后走进；怪：开战、胜利 restep 进敌格）
        let mut keys = make();
        key_walk(&mut keys, -4, (6, ROW));
        assert_eq!((keys.player_cell_x, keys.player_cell_y), (6, ROW), "{label} 参照前置：走进 (6,ROW)");
        let mut sc = make();
        sc.tap.auto_battle = true; // 【计划 2 T6】本测试测撞击收尾、不测首点选中：自动战斗开 ⇒ 点怪一点即撞（门不受影响）
        let wt = sc.width_table.clone();
        assert_eq!(sc.tap_cell(6, ROW), crate::tap_walk::TapResult::Bumping, "{label}");
        let mut fc = 0;
        while fc < 3000 && !((sc.player_cell_x, sc.player_cell_y) == (6, ROW) && sc.walk_phase == 0 && sc.game_mode == GameMode::Map) {
            sc.tick(fc, &wt);
            fc += 1;
        }
        settle(&mut sc, fc, 20);
        fc += 20;
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "{label}：撞进去了");
        assert!(!sc.route.is_open() && !sc.route.is_suspended(), "{label}：撞进去那一步步末收段");
        assert_eq!(sc.tap.tap_target, None, "{label}：收尾后目标标记清除");
        assert!(!sc.script_walk_armed, "{label}：无脚本行走武装（下次方向键不走 advanceScriptWalk 臂）");
        assert_eq!(arrow_state(&sc), arrow_state(&keys), "{label}：撞进去后箭头 = 方向键参照");
        // 再按一下方向键（右，沿箭头）：与方向键参照同一结果
        for k in [&mut keys, &mut sc] {
            k.press_key(-4);
            k.tick(fc, &wt);
            k.release_key();
            settle(k, fc + 1, 40);
        }
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (7, ROW), "{label}：方向键走一格");
        assert_eq!(arrow_state(&sc), arrow_state(&keys), "{label}：再走一格后箭头 = 方向键参照");
        // 撤销栈：点地段 (1→5) 与撞进去那一格 (5→6) 各一笔（怪另有战斗笔），方向键那格不入栈
        let seq = undo_seq(&mut sc);
        let expect: Vec<String> = if target_type == 1 {
            vec![format!("tile{:?}", (5, ROW)), format!("tile{:?}", (1, ROW))]
        } else {
            vec![format!("tile{:?}", (5, ROW)), "revive".to_string(), format!("tile{:?}", (1, ROW))]
        };
        assert_eq!(seq, expect, "{label}：撤销序列");
    }

    #[test]
    fn tap_bump_door_with_tutorial_arrows_closes_route() {
        bump_with_tutorial_arrows(1);
    }

    #[test]
    fn tap_bump_enemy_with_tutorial_arrows_closes_route_after_restep() {
        bump_with_tutorial_arrows(41);
    }

    #[test]
    fn tap_cell_route_through_pickup_overlay_resumes_after_close() {
        pickup_overlay_route(-5);
    }

    /// 对照：框开着时按方向键（-3 左）。kind 1 / right=3 分支（scene.rs tick 浮层块）对 0 吞、-2/-1 翻页、
    /// 其余键关层，块尾 keyValue / keyHeldCode 双清（a.java:3345）⇒ -3 只起关框作用、不走一步、不打断路线；
    /// 关框后同样续走到终点。
    #[test]
    fn tap_cell_route_through_pickup_overlay_direction_key_also_resumes() {
        pickup_overlay_route(-3);
    }

    /// (x, ROW) 上的门（型 1）是否未开：实体仍在、常态、未移除。
    fn door_intact(sc: &GameScene, x: i32) -> bool {
        (0..sc.entities.count).any(|i| {
            let e = &sc.entities.items[i];
            e.type_ == 1 && e.px == x << 5 && e.py == ROW << 5 && e.state == 0 && !e.removed
        })
    }

    /// 点门（有钥匙）多步路线：起点 (2,ROW)、门 (7,ROW)、停靠格 (6,ROW)。跑到末步在途窗口
    ///（`Map ∧ walk_phase == 1 ∧ tap_followup_ready`：advanceScriptWalk 在 (5,ROW) 步末弹出末步，finish_walkto 随即回 Map）。
    fn door_route_in_last_step_window() -> (GameScene, i64) {
        let mut sc = scene((2, ROW), &[(1, 7, ROW)]);
        apply_setup(&mut sc, SceneSetup { player_yellow_keys: Some(1), ..Default::default() });
        assert_eq!(sc.tap_cell(7, ROW), crate::tap_walk::TapResult::Bumping);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !(sc.game_mode == GameMode::Map && sc.walk_phase == 1 && sc.tap.tap_followup_ready) && fc < 400 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.game_mode == GameMode::Map && sc.walk_phase == 1 && sc.tap.tap_followup_ready, "前置：到达末步在途窗口");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "前置：末步 (5,ROW)→(6,ROW) 在途");
        assert_eq!(sc.tap.tap_bump, Some(((6, ROW), 2)), "前置：撞击后续在");
        (sc, fc)
    }

    /// 【STAGE-B-008 · spec §5.2 · 计划 2 T5 复核 R3（探针 P1）】末步在途窗口算"点地寻路中"：点别处 ⇒ `Queued`
    ///（【计划 2 T7 改写】T5 原期望 `Stopped`，T7 起按"末步窗口点别处 = 排队改道"），不闪红、旧撞击取消；
    /// 落格后改道去新目标 (3,ROW)，旧撞击不执行（钥匙仍 1、门未开）。
    #[test]
    fn tap_cell_in_last_step_window_queues_reroute_and_cancels_bump() {
        let (mut sc, fc) = door_route_in_last_step_window();
        assert!(sc.accepts_field_tap(), "末步在途窗口场上接收点地（Map）");
        assert_eq!(sc.tap_cell(3, ROW), crate::tap_walk::TapResult::Queued);
        assert_eq!(sc.tap.tap_reject, None, "不闪红");
        assert_eq!(sc.tap.tap_bump, None, "撞击已取消");
        assert_eq!(sc.tap.tap_target, Some((3, ROW)), "目标标记换到新目标");
        settle(&mut sc, fc, 120);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (3, ROW), "末步走完后改道到新目标");
        assert_eq!(sc.player.yellow_keys, 1, "未耗钥");
        assert!(door_intact(&sc, 7), "门未开");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert_eq!(sc.tap.tap_target, None, "目标标记清除");
    }

    /// 【计划 2 T5 复核 R4（探针 P3）】末步在途窗口按方向键（上，撞墙方向）⇒ 撞击取消（钥匙仍 1、门未开）。
    #[test]
    fn key_press_in_last_step_window_cancels_bump() {
        let (mut sc, fc) = door_route_in_last_step_window();
        sc.press_key(-1);
        sc.release_key();
        assert_eq!(sc.tap.tap_bump, None, "场上按键取消撞击");
        settle(&mut sc, fc, 80);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "停在停靠格");
        assert_eq!(sc.player.yellow_keys, 1, "未耗钥");
        assert!(door_intact(&sc, 7), "门未开");
        assert_eq!(sc.tap.tap_target, None, "目标标记清除");
    }

    /// 【STAGE-B-008 · spec §5.1 · 计划 2 T5 复核 R1/R2（探针 M）】点门路线途经特殊拾取物（型 25 @ (4,ROW)）：
    /// 框开时按 -5 关框 ⇒ 关框键属浮层，不打断路线、不取消撞击；关框后续走到停靠格 (7,ROW) 并撞开门 (8,ROW)、走进门格。
    #[test]
    fn tap_bump_route_through_pickup_overlay_still_bumps_after_close() {
        let mut sc = scene((2, ROW), &[(25, 4, ROW), (1, 8, ROW)]);
        apply_setup(&mut sc, SceneSetup { player_yellow_keys: Some(1), ..Default::default() });
        assert_eq!(sc.tap_cell(8, ROW), crate::tap_walk::TapResult::Bumping);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !sc.dialog.overlay_active && fc < 200 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.dialog.overlay_active, "前置：拾取型 25 弹提示框");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (4, ROW), "前置：框开时人在拾取格");
        assert_eq!(sc.tap.tap_bump, Some(((7, ROW), 2)), "前置：撞击后续在");
        sc.press_key(-5);
        assert_eq!(sc.tap.tap_bump, Some(((7, ROW), 2)), "关框键不取消撞击");
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        assert!(!sc.dialog.overlay_active, "提示框已关");
        settle(&mut sc, fc, 300);
        assert_eq!(sc.player.yellow_keys, 0, "撞门耗钥");
        assert!(!door_intact(&sc, 8), "门已开");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (8, ROW), "走进门格");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(!sc.route.is_open() && !sc.script_walk_armed && !sc.walkto_draining, "无寻路残留");
        assert_eq!(sc.tap.tap_target, None, "收尾后目标标记清除");
    }

    /// 【STAGE-B-008 · spec §5.1 · 计划 2 T5 复核 R2（探针 N2）】停靠格 (6,ROW) 本身有型 25：末步落格弹框，
    /// 撞击暂缓到关框后（框开期间保留撞击后续），关框后下一拍撞开门 (7,ROW)。
    #[test]
    fn tap_bump_dock_pickup_overlay_defers_bump_until_close() {
        let mut sc = scene((2, ROW), &[(25, 6, ROW), (1, 7, ROW)]);
        apply_setup(&mut sc, SceneSetup { player_yellow_keys: Some(1), ..Default::default() });
        assert_eq!(sc.tap_cell(7, ROW), crate::tap_walk::TapResult::Bumping);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !sc.dialog.overlay_active && fc < 200 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.dialog.overlay_active, "前置：停靠格拾取弹框");
        settle(&mut sc, fc, 10); // 框开期间多跑几拍：撞击须暂缓、不作废
        fc += 10;
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "框开时人在停靠格");
        assert_eq!(sc.tap.tap_bump, Some(((6, ROW), 2)), "框开期间撞击后续保留");
        assert_eq!(sc.player.yellow_keys, 1, "框开期间未撞");
        assert_eq!(sc.tap.tap_target, Some((7, ROW)), "框开期间目标标记仍在");
        sc.press_key(-5);
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        assert!(!sc.dialog.overlay_active, "提示框已关");
        settle(&mut sc, fc, 200);
        assert_eq!(sc.player.yellow_keys, 0, "关框后撞门耗钥");
        assert!(!door_intact(&sc, 7), "门已开");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (7, ROW), "走进门格");
        assert_eq!(sc.tap.tap_target, None, "收尾后目标标记清除");
    }

    /// 【计划 2 T5 复核 R4（变异 M11）】1 步路线：站 (5,ROW) 点门 (7,ROW)，停靠格 (6,ROW)。首步在途（Dialog dp3 wp1）时点自己
    /// ⇒ `Stopped`；排空落到停靠格后不撞（钥匙仍 1、门未开）。
    #[test]
    fn tap_self_during_one_step_route_stops_without_bump() {
        let mut sc = scene((5, ROW), &[(1, 7, ROW)]);
        apply_setup(&mut sc, SceneSetup { player_yellow_keys: Some(1), ..Default::default() });
        assert_eq!(sc.tap_cell(7, ROW), crate::tap_walk::TapResult::Bumping);
        assert_eq!(sc.walk_step_count, 1, "前置：1 步路线");
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !(sc.game_mode == GameMode::Dialog && sc.walk_phase == 1) && fc < 50 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.game_mode == GameMode::Dialog && sc.walk_phase == 1, "前置：首步在途");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW));
        assert_eq!(sc.tap_cell(5, ROW), crate::tap_walk::TapResult::Stopped);
        settle(&mut sc, fc, 80);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "排空落到停靠格");
        assert_eq!(sc.player.yellow_keys, 1, "未耗钥");
        assert!(door_intact(&sc, 7), "门未开");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert_eq!(sc.tap.tap_target, None, "目标标记清除");
    }

    /// 【计划 2 T5 复核 R4（变异 M9）】按住方向键（上，顶墙不动：keyHeldCode 每拍 tryStep 撞墙，停在 wp0）时点门：
    /// 撞击后续是在按键之后设的，press_key 清不到；落到停靠格时 keyHeldCode 仍非 0 ⇒ 场上手动输入优先，撞击作废
    ///（钥匙仍 1、门未开、人留在停靠格）。
    #[test]
    fn held_key_at_dock_cancels_bump() {
        let mut sc = scene((2, ROW), &[(1, 7, ROW)]);
        apply_setup(&mut sc, SceneSetup { player_yellow_keys: Some(1), ..Default::default() });
        let wt = sc.width_table.clone();
        sc.press_key(-1); // 按住不放
        sc.tick(0, &wt);
        assert_eq!((sc.player_cell_x, sc.player_cell_y, sc.walk_phase), (2, ROW, 0), "前置：顶墙不动");
        assert_eq!(sc.tap_cell(7, ROW), crate::tap_walk::TapResult::Bumping);
        assert_eq!(sc.tap.tap_bump, Some(((6, ROW), 2)), "前置：撞击后续在");
        settle(&mut sc, 1, 200);
        assert_eq!(sc.key_held, -1, "前置：全程按住");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "停在停靠格");
        assert_eq!(sc.player.yellow_keys, 1, "未耗钥");
        assert!(door_intact(&sc, 7), "门未开");
        assert_eq!(sc.tap.tap_target, None, "目标标记清除");
    }

    /// 【计划 2 T5 复核 R4（变异 M8 / M8b）】换层（load_floor_data）与剧情接管（load_level_script）清点地状态：
    /// 目标标记、闪红、撞击后续、后续就绪位全空。
    #[test]
    fn tap_state_cleared_on_floor_change_and_script_takeover() {
        for entry in ["换层", "剧情"] {
            let mut sc = scene((2, ROW), &[(1, 7, ROW)]);
            apply_setup(&mut sc, SceneSetup { player_yellow_keys: Some(1), dialog_dialog_data_scripts: Some(vec!["TAK_0_0 ".to_string()]), ..Default::default() });
            assert_eq!(sc.tap_cell(7, ROW), crate::tap_walk::TapResult::Bumping);
            sc.flash_reject((9, ROW));
            assert_eq!(sc.tap.tap_target, Some((7, ROW)), "{entry} 前置：目标标记");
            assert_eq!(sc.tap.tap_bump, Some(((6, ROW), 2)), "{entry} 前置：撞击后续");
            assert!(sc.tap.tap_reject.is_some(), "{entry} 前置：闪红");
            if entry == "换层" {
                sc.load_floor_data(&super::scene_test_support::maplv(), 2 << 5, ROW << 5, &[], 2).unwrap();
            } else {
                sc.load_level_script(0);
            }
            assert_eq!(sc.tap.tap_target, None, "{entry}：目标标记清除");
            assert_eq!(sc.tap.tap_reject, None, "{entry}：闪红清除");
            assert_eq!(sc.tap.tap_bump, None, "{entry}：撞击后续清除");
            assert!(!sc.tap.tap_followup_ready, "{entry}：后续就绪位清除");
        }
    }

    /// 路线已收尾（Map wp0）、撞击排在下一拍开头时再点按（此处点自己：不开新路线、仍在 Map）：旧撞击作废。
    #[test]
    fn t6_new_tap_in_followup_gap_cancels_pending_bump() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(1, 8, ROW)]);
        apply_setup(&mut sc, SceneSetup { player_yellow_keys: Some(1), ..Default::default() });
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !(sc.tap.tap_followup_ready && sc.game_mode == GameMode::Map && sc.walk_phase == 0) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：收尾间隙");
        }
        assert_eq!(sc.player_cell_x, 7, "前置：停在门前");
        // 点自己（不开新路线、仍在 Map）：间隙里的旧撞击必须作废，否则下一拍照样开门
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Ignored);
        settle(&mut sc, fc, 60);
        assert_eq!(sc.player.yellow_keys, 1, "旧撞击作废：门未开");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (7, ROW), "原地不动");
        // 【复核 R4】只去间隙里的 tap_bump = None 时旧撞击滞留 ⇒ 目标框挂在门上不消失
        assert!(sc.tap.tap_bump.is_none(), "旧撞击后续已清");
        assert_eq!(sc.tap.tap_target, None, "目标框不残留");
    }

    fn strong(sc: &mut GameScene) {
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
    }

    /// 【STAGE-B-008 · spec §5.2 Q8】点怪：首点只选中（不动、不开路线）；再点同一只 = 走到邻格撞上开战（战斗笔可撤销）。
    #[test]
    fn t6_first_tap_selects_second_tap_fights() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 6, ROW)]);
        strong(&mut sc);
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Selected);
        assert_eq!(sc.tap.tap_selected.map(|s| s.entity), Some(e));
        assert_eq!(sc.tap.tap_target, None, "选中不画目标框（有自己的闪烁框）");
        settle(&mut sc, 0, 20);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "选中不移动");
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map);
        assert!(sc.tap.tap_selected.is_some(), "空闲时选中保持");
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Bumping);
        assert!(sc.tap.tap_selected.is_none(), "开战即清选中");
        settle(&mut sc, 20, 400);
        let ent = &sc.entities.items[e];
        assert!(ent.removed || ent.state == 1, "怪被击杀");
        assert!(journal_has_combat(&mut sc), "战斗笔入栈（误触靠撤销）");
    }

    /// 自动战斗开：一点即战。
    #[test]
    fn t6_auto_battle_fights_on_first_tap() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 6, ROW)]);
        strong(&mut sc);
        sc.tap.auto_battle = true;
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Bumping);
        assert!(sc.tap.tap_selected.is_none());
    }

    /// 点另一只 = 改选；点别处 = 清选中并执行。
    #[test]
    fn t6_tap_elsewhere_or_other_monster_replaces_selection() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 6, ROW), (42, 9, ROW)]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Selected);
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Selected, "点另一只 = 改选");
        assert_eq!(sc.tap.tap_selected.map(|s| s.type_), Some(42));
        assert_eq!(sc.tap_cell(4, ROW), TapResult::Walking, "点别处 = 清选中并执行");
        assert!(sc.tap.tap_selected.is_none());
    }

    /// 多格怪（型 69 宽 3 格）：点任一格选中同一只，再点另一格 = 开战（停靠在整只怪的外侧邻格）。
    #[test]
    fn t6_multi_cell_monster_any_cell_is_same_selection() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(69, 6, ROW)]);
        strong(&mut sc);
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Selected, "点 69 的第三格");
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Bumping, "再点同一只的中间格 = 开战");
    }

    /// 选中有效性每拍校验：移除 / 死亡动画 / 被搬走 / 变型 / 换层 / 浮层 / 按键 / 清状态，任一即清。
    #[test]
    fn t6_selection_invalidated_every_tick() {
        let mk = || {
            let mut sc = scene((2, ROW), &[(41, 6, ROW)]);
            let _ = sc.tap_cell(6, ROW);
            assert!(sc.tap.tap_selected.is_some(), "前置：已选中");
            sc
        };
        let cases: Vec<(&str, Box<dyn Fn(&mut GameScene)>)> = vec![
            ("实体移除（他处击杀）", Box::new(|sc| { let e = sc.tap.tap_selected.unwrap().entity; sc.entities.items[e].removed = true; })),
            ("死亡动画 state = 1", Box::new(|sc| { let e = sc.tap.tap_selected.unwrap().entity; sc.entities.items[e].state = 1; })),
            ("被搬走（MOV）", Box::new(|sc| { let e = sc.tap.tap_selected.unwrap().entity; sc.entities.items[e].px += 32; })),
            ("变型（道具改写）", Box::new(|sc| { let e = sc.tap.tap_selected.unwrap().entity; sc.entities.items[e].type_ = 42; })),
            ("楼层号变化", Box::new(|sc| sc.persist.current_floor += 1)),
            ("浮层", Box::new(|sc| sc.dialog.overlay_active = true)),
            // 【复核 R1】以下各钉一个 selection_valid 判据（去掉该判据即红）
            // hide_notify 真实入口：Map → InGameMenu（walk_phase / 浮层 / 楼层 / 实体都不变，只有 game_mode 判据拦得住）
            ("hide_notify（切后台 ⇒ 游戏内菜单）", Box::new(|sc| {
                sc.hide_notify();
                assert_eq!(sc.game_mode, GameMode::InGameMenu, "前置：hide_notify 进游戏内菜单");
            })),
            ("观察模式 walk_phase = 2", Box::new(|sc| sc.walk_phase = 2)),
            ("换层遮幅 transitioning", Box::new(|sc| sc.transitioning = true)),
            ("实体隐藏 visible = false", Box::new(|sc| { let e = sc.tap.tap_selected.unwrap().entity; sc.entities.items[e].visible = false; })),
            // 防御判据：层内实体表只追加不压实、重排只在 reload_floor_entities（同时有型 / 格判据兜底），
            // 没有真实路径能让选中下标单独越界 ⇒ 直接缩小 count 构造
            ("实体下标越界（防御判据）", Box::new(|sc| sc.entities.count = 0)),
            // 「切模式」（classic ↔ modern）由计划 3 接入后补测
        ];
        for (what, f) in cases {
            let mut sc = mk();
            f(&mut sc);
            settle(&mut sc, 0, 1);
            assert!(sc.tap.tap_selected.is_none(), "{what} ⇒ 清选中");
        }
        let mut sc = mk();
        sc.press_key(-2);
        assert!(sc.tap.tap_selected.is_none(), "任何按键 ⇒ 清选中");
        let mut sc = mk();
        sc.clear_tap_state();
        assert!(sc.tap.tap_selected.is_none(), "撤销 / 换层 / 剧情 ⇒ 清选中");
        let mut sc = mk();
        assert_eq!(sc.engage_selected(), crate::tap_walk::TapResult::Bumping, "「开战」= 撞已选中的怪");
    }

    /// 【复核 R1】换层 / 读档真实入口 load_floor_data：立即清选中（不等下一拍）。
    #[test]
    fn t6_selection_cleared_by_load_floor_data() {
        let mut sc = scene((2, ROW), &[(41, 6, ROW)]);
        let _ = sc.tap_cell(6, ROW);
        assert!(sc.tap.tap_selected.is_some(), "前置：已选中");
        sc.load_floor_data(&super::scene_test_support::maplv(), 2 << 5, ROW << 5, &[], 2).unwrap();
        assert!(sc.tap.tap_selected.is_none(), "换层 / 读档 ⇒ 立即清选中");
    }

    /// 【复核 R1】他处击杀真实击杀代码：选中邻格怪后用道具 86（use_item_stack → activate_item(86) → 四邻格 killAdjacentAt），下一拍清选中。
    /// 直接调确认浮层 -6 臂的 use_item_stack：UI 全路径上开面板的按键 / ItemPanel 模式 / 确认浮层本身都会先清选中（复核探针 P05），
    /// 这里要钉的是"选中仍在时实体被他处击杀"的 removed / state 判据。
    #[test]
    fn t6_selection_cleared_after_item86_kills_selected() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((5, ROW), &[(41, 6, ROW)]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Selected, "前置：选中邻格怪");
        let e = sc.tap.tap_selected.unwrap().entity;
        sc.player.item_types[0] = 86;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        sc.use_item_stack(0);
        let ent = &sc.entities.items[e];
        assert!(ent.removed || ent.state == 1, "前置：86 击杀选中的怪");
        assert!(sc.tap.tap_selected.is_some(), "前置：选中要等每拍校验清");
        settle(&mut sc, 0, 1);
        assert!(sc.tap.tap_selected.is_none(), "道具 86 击杀 ⇒ 清选中");
    }

    /// 【复核 R2】实体表重排（跨层探针 m_119/m_122 往返压实）后、下一拍校验前的同帧里点同下标的另一只：只选中、不开战。
    #[test]
    fn t6_reorder_then_tap_other_monster_same_frame_selects() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((6, ROW), &[(41, 2, ROW), (42, 9, ROW), (43, 4, ROW)]);
        let i41 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        sc.entities.items[i41].removed = true; // 已死：往返后压掉 ⇒ 后面的下标前移
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Selected, "前置：选中 42");
        let sel = sc.tap.tap_selected.unwrap();
        let _ = sc.probe_cross_floor_snapshot(23, 12); // 结果无关：只要它的 save → reload(23) → reload(本层) 往返
        let at = &sc.entities.items[sel.entity];
        assert_eq!((at.type_, at.px >> 5, at.py >> 5), (43, 4, ROW), "前置：重排后同下标 = 43@4");
        assert!(sc.tap.tap_selected.is_some(), "前置：未 tick，选中仍在");
        assert_eq!(sc.tap_cell(4, ROW), TapResult::Selected, "同下标的另一只 = 首点选中，不开战");
        assert_eq!(sc.tap.tap_selected.map(|s| s.type_), Some(43));
        assert!(!sc.route.is_open(), "未开路线");
    }

    /// 【复核 R3】选中后实体被移除、未 tick 就按「开战」：Ignored、清选中、玩家不动。
    #[test]
    fn t6_engage_selected_invalid_is_ignored() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 6, ROW)]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Selected);
        let e = sc.tap.tap_selected.unwrap().entity;
        sc.entities.items[e].removed = true;
        assert_eq!(sc.engage_selected(), TapResult::Ignored);
        assert!(sc.tap.tap_selected.is_none(), "清选中");
        assert!(!sc.route.is_open(), "未开路线");
        settle(&mut sc, 0, 30);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "玩家不动");
    }

    /// 自起步跑到玩家格 x ≥ `x` 且仍在点地寻路中（Dialog），返回下一拍帧号。
    fn walk_until_x(sc: &mut GameScene, x: i32) -> i64 {
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while sc.player_cell_x < x {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：走到 x={x}");
        }
        assert_eq!(sc.game_mode, GameMode::Dialog, "前置：仍在寻路中");
        fc
    }

    /// 【STAGE-B-008 · spec §5.2 排队】寻路中点别处：走完当前格后改道去新目标，标记立即换到新目标。
    #[test]
    fn t7_tap_while_walking_queues_reroute() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        assert_eq!(sc.tap_cell(10, ROW), TapResult::Walking);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        assert_eq!(sc.tap.tap_target, Some((2, ROW)), "标记立即换到新目标");
        settle(&mut sc, fc, 300);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "改道到新目标");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert_eq!(sc.tap.tap_target, None);
    }

    /// 寻路中点自己：停下（走完当前格），无排队、无标记。
    #[test]
    fn t7_tap_self_while_walking_stops() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        let me = (sc.player_cell_x, sc.player_cell_y);
        assert_eq!(sc.tap_cell(me.0, me.1), TapResult::Stopped);
        settle(&mut sc, fc, 200);
        assert!((4..=5).contains(&sc.player_cell_x), "走完当前格即停（实得 {}）", sc.player_cell_x);
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map);
        assert_eq!(sc.tap.tap_target, None);

        // 【计划 2 T7 复核 R3（变异 Mb）】先排队再点自己：排队作废 ⇒ Stopped、排空后停在原处附近，不改道去排队目标
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued, "前置：排队");
        let me = (sc.player_cell_x, sc.player_cell_y);
        assert_eq!(sc.tap_cell(me.0, me.1), TapResult::Stopped);
        assert!(sc.tap.tap_pending.is_none(), "点自己清排队");
        assert_eq!(sc.tap.tap_target, None, "点自己清目标框");
        settle(&mut sc, fc, 200);
        assert!((4..=5).contains(&sc.player_cell_x), "排空后停在原处附近、不改道去 x=2（实得 {}）", sc.player_cell_x);
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map);
        assert!(sc.tap.tap_pending.is_none() && sc.tap.tap_target.is_none());
    }

    /// 排队失效：排空期间按键 ⇒ 丢弃；落格那一拍有手动按键（按住）⇒ 丢弃。
    /// 【执行期更正 · 以代码为准】落格时浮层开着 ⇒ 与撞击后续同口径**暂缓**到关框后执行（计划原文写"丢弃"，见 Task 7 更正②）。
    #[test]
    fn t7_queued_tap_discarded_on_key_or_blocked_landing() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        sc.press_key(-2); // 朝下（墙）
        sc.release_key();
        settle(&mut sc, fc, 200);
        assert!(sc.player_cell_x >= 4, "按键作废排队：不改道回 x=2（实得 {}）", sc.player_cell_x);

        // 浮层：落格（收尾）后、执行前出现浮层 ⇒ 暂缓，标记仍在；关框后下一拍执行
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let mut fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        let wt = sc.width_table.clone();
        while sc.game_mode != GameMode::Map {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 300);
        }
        let landed = sc.player_cell_x;
        sc.dialog.overlay_active = true; // 收尾后、执行前出现浮层（如拾取提示）
        sc.dialog.overlay_text = Some("提示".into());
        sc.tick(fc, &wt);
        sc.tick(fc + 1, &wt);
        assert_eq!(sc.player_cell_x, landed, "浮层期间不执行排队");
        assert_eq!(sc.tap.tap_target, Some((2, ROW)), "浮层期间排队暂缓、标记仍在");
        sc.dialog.overlay_active = false;
        settle(&mut sc, fc + 2, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "关框后执行排队");
        assert!(!sc.route.is_open() && sc.tap.tap_target.is_none());

        // 落格时按住方向键（朝下顶墙）：手动输入优先 ⇒ 排队作废
        let mut sc = scene((1, ROW), &[]);
        let wt = sc.width_table.clone();
        sc.press_key(-2); // 按住不放（press_key 发生在排队之前，清不到）
        sc.tick(0, &wt);
        assert_eq!((sc.player_cell_x, sc.walk_phase), (1, 0), "前置：顶墙不动");
        assert_eq!(sc.tap_cell(10, ROW), TapResult::Walking);
        let mut fc = 1;
        while sc.player_cell_x < 4 {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200);
        }
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        settle(&mut sc, fc, 200);
        assert_eq!(sc.key_held, -2, "前置：全程按住");
        assert!(sc.player_cell_x >= 4, "按住键作废排队（实得 {}）", sc.player_cell_x);
        assert!(!sc.route.is_open() && sc.tap.tap_target.is_none() && sc.tap.tap_pending.is_none());
    }

    /// 滑动：空闲 = 一格（朝墙 = 撞停并转向）；寻路中 = 排队，排空回 Map 后朝该方向踏一步。
    #[test]
    fn t7_swipe_steps_when_idle_and_queues_while_walking() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((3, ROW), &[]);
        assert_eq!(sc.swipe_step(2), TapResult::Walking);
        settle(&mut sc, 0, 40);
        assert_eq!(sc.player_cell_x, 4, "空闲滑动 = 一格");
        assert_eq!(sc.swipe_step(1), TapResult::Bumping, "朝墙滑 = 撞停");
        assert_eq!(sc.facing, 1, "撞墙后朝向 = 滑动方向（BUG-047）");

        let _ = sc.tap_cell(10, ROW);
        let wt = sc.width_table.clone();
        let mut fc = 40;
        while sc.player_cell_x < 6 {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 300);
        }
        assert_eq!(sc.swipe_step(3), TapResult::Queued);
        while sc.game_mode != GameMode::Map {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 400);
        }
        let landed = sc.player_cell_x;
        settle(&mut sc, fc, 60);
        assert_eq!(sc.player_cell_x, landed - 1, "排空落地后朝左踏一步");

        // 【计划 2 T7 复核 R5（变异 Ml）】排队 Cell 后再滑动：滑动覆盖排队、目标框立即清除
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued, "前置：排队 Cell");
        assert_eq!(sc.tap.tap_target, Some((2, ROW)), "前置：目标框在排队目标");
        assert_eq!(sc.swipe_step(2), TapResult::Queued);
        assert_eq!(sc.tap.tap_target, None, "滑动排队 ⇒ 目标框清除");
        assert_eq!(sc.tap.tap_pending, Some(crate::tap_walk::PendingTap::Swipe(2)), "滑动覆盖排队 Cell");
        let wt = sc.width_table.clone();
        sc.tick(fc, &wt);
        assert_eq!(sc.tap.tap_target, None, "下一拍目标框仍不在");
    }

    /// 滑动的非空闲态：观察模式忽略；手动行走 / 战斗 / Map 语境脚本行走 = Rejected（无格可闪红）；浮层开着 = Ignored。
    #[test]
    fn t7_swipe_ignored_or_rejected_when_not_idle() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((3, ROW), &[]);
        apply_setup(&mut sc, SceneSetup { walk_phase: Some(2), ..Default::default() });
        assert_eq!(sc.swipe_step(2), TapResult::Ignored, "观察模式");
        sc.walk_phase = 1;
        assert_eq!(sc.swipe_step(2), TapResult::Rejected, "手动行走中");
        sc.walk_phase = 5;
        assert_eq!(sc.swipe_step(2), TapResult::Rejected, "战斗中");
        sc.walk_phase = 0;
        sc.script_walk_armed = true;
        assert_eq!(sc.swipe_step(2), TapResult::Rejected, "Map 语境脚本行走");
        sc.script_walk_armed = false;
        assert!(sc.tap.tap_reject.is_none(), "滑动无目标格、不闪红");
        sc.dialog.overlay_active = true;
        assert_eq!(sc.swipe_step(2), TapResult::Ignored, "浮层开着");
        sc.dialog.overlay_active = false;
        assert_eq!(sc.swipe_step(4), TapResult::Ignored, "非法方向码");
        assert_eq!((sc.player_cell_x, sc.walk_phase), (3, 0), "全程未动");
    }

    /// 【spec §5.2 · T5 复核 R3】末步在途窗口（Map wp1 ∧ 后续就绪）点别处 ⇒ 排队：`interrupt_route` 为空操作，
    /// 末步照常落格，落格后的后续拍改道去新目标。
    #[test]
    fn t7_tap_in_last_step_window_queues_reroute() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Walking);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !(sc.game_mode == GameMode::Map && sc.walk_phase == 1 && sc.tap.tap_followup_ready) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：末步在途窗口");
        }
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "前置：末步 (5,ROW)→(6,ROW) 在途");
        assert_eq!(sc.tap_cell(3, ROW), TapResult::Queued);
        assert_eq!(sc.tap.tap_target, Some((3, ROW)), "标记立即换到新目标");
        assert_eq!(sc.tap.tap_reject, None, "不闪红");
        settle(&mut sc, fc, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (3, ROW), "落格后改道到新目标");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(!sc.route.is_open() && sc.tap.tap_target.is_none() && sc.tap.tap_pending.is_none());
    }

    /// 寻路途经拾取物弹框时点别处 / 滑动：浮层开着 `accepts_field_tap` 为假 ⇒ Ignored、不排队；关框后续走原目标。
    #[test]
    fn t7_tap_while_pickup_overlay_mid_route_is_ignored() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(25, 5, ROW)]);
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Walking);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !sc.dialog.overlay_active && fc < 200 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert!(sc.dialog.overlay_active, "前置：拾取型 25 弹提示框");
        assert!(!sc.accepts_field_tap(), "浮层开着不收点地");
        assert_eq!(sc.tap_cell(3, ROW), TapResult::Ignored);
        assert_eq!(sc.swipe_step(3), TapResult::Ignored);
        assert!(sc.tap.tap_pending.is_none(), "不排队");
        assert_eq!(sc.tap.tap_target, Some((9, ROW)), "原目标标记不变");
        assert!(sc.route.is_open(), "路段未打断");
        sc.press_key(-5);
        sc.tick(fc, &wt);
        fc += 1;
        sc.release_key();
        assert!(!sc.dialog.overlay_active, "提示框已关");
        settle(&mut sc, fc, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (9, ROW), "关框后续走原目标");
    }

    /// 排队点怪：落格后按 `tap_cell` 语义 = 首点选中（不走过去、不开战）。
    #[test]
    fn t7_queued_tap_on_monster_selects_after_landing() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((5, ROW), &[(41, 2, ROW)]);
        strong(&mut sc);
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        assert_eq!(sc.tap_cell(10, ROW), TapResult::Walking);
        let fc = walk_until_x(&mut sc, 7);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        settle(&mut sc, fc, 200);
        assert_eq!(sc.tap.tap_selected.map(|s| s.entity), Some(e), "落格后选中");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (8, ROW), "停在排空落格，不走向怪");
        let ent = &sc.entities.items[e];
        assert!(!ent.removed && ent.state == 0, "未开战");
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map);
        assert_eq!(sc.tap.tap_target, None, "选中不画目标框");
    }

    /// 末步在途窗口滑动 ⇒ 排队（与点按同一"寻路中"判据），落格后朝该方向踏一步。
    #[test]
    fn t7_swipe_in_last_step_window_queues() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Walking);
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while !(sc.game_mode == GameMode::Map && sc.walk_phase == 1 && sc.tap.tap_followup_ready) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：末步在途窗口");
        }
        assert_eq!(sc.swipe_step(2), TapResult::Queued);
        settle(&mut sc, fc, 60);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (7, ROW), "末步落 (6,ROW) 后朝右再踏一步");
    }

    /// 撤销 / 换层（clear_tap_state）清排队：之后排空落格不改道。
    #[test]
    fn t7_queued_tap_cleared_by_clear_tap_state() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        sc.clear_tap_state();
        assert!(sc.tap.tap_pending.is_none() && sc.tap.tap_target.is_none());
        settle(&mut sc, fc, 200);
        assert_eq!(sc.player_cell_x, 5, "排空落格后不改道");

        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let _ = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        sc.load_floor_data(&super::scene_test_support::maplv(), 5 << 5, ROW << 5, &[], 2).unwrap();
        assert!(sc.tap.tap_pending.is_none(), "换层清排队");
    }

    /// 排队已收尾、执行排在下一拍开头的间隙里再点按（Map wp0 新路线）：新点按取代旧排队——新路线走完后不再执行旧排队。
    #[test]
    fn t7_new_tap_in_followup_gap_supersedes_queue() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let mut fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        let wt = sc.width_table.clone();
        while !(sc.tap.tap_followup_ready && sc.game_mode == GameMode::Map && sc.walk_phase == 0) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：收尾间隙");
        }
        assert_eq!(sc.player_cell_x, 5, "前置：排空落在 (5,ROW)");
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Walking, "间隙里点按 = 新路线");
        settle(&mut sc, fc, 300);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (8, ROW), "新路线走完即停，不再执行旧排队（回 x=2）");
        assert!(sc.tap.tap_pending.is_none() && sc.tap.tap_target.is_none());
    }

    /// 【计划 2 T7 复核 R6 · spec §5.2 更正②】撞击后续所在路线途中被隐形怪截住开战（路段挂起）：撞击后续与目标框在挂起期间保留，
    /// 胜利后路线续走到停靠格 (7,ROW)，照常撞开门 (8,ROW) 并走进门格（"点哪里就往哪里走"）。
    #[test]
    fn t7_bump_survives_battle_suspension_mid_route() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 5, ROW), (1, 8, ROW)]);
        hide_types(&mut sc, &[41]); // 未显形的怪：tap_grid 视为地面，try_step 照样开战
        strong(&mut sc);
        sc.player.yellow_keys = 1;
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping);
        assert_eq!(sc.tap.tap_bump, Some(((7, ROW), 2)), "前置：撞击后续在");
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while sc.walk_phase != 5 {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：途中开战");
        }
        assert!(!sc.route.is_open() && sc.route.is_suspended(), "前置：路段挂起");
        sc.tick(fc, &wt);
        fc += 1;
        assert_eq!(sc.tap.tap_bump, Some(((7, ROW), 2)), "开战挂起期间撞击后续保留");
        assert_eq!(sc.tap.tap_target, Some((8, ROW)), "挂起期间目标框保留");
        settle(&mut sc, fc, 400);
        let i41 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        assert!(sc.entities.items[i41].removed || sc.entities.items[i41].state == 1, "前置：怪被击杀");
        assert_eq!(sc.player.yellow_keys, 0, "胜利后续走到停靠格并撞门耗钥");
        assert!(!door_intact(&sc, 8), "门已开");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (8, ROW), "走进门格");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert!(sc.tap.tap_bump.is_none() && sc.tap.tap_target.is_none(), "收尾后无残留");
    }

    /// 【计划 2 T7 复核 R6】路段开战挂起期间，壳层投键口径（win `route_key` 抬键前先 `interrupt_route`）解除挂起：Dialog dp3 下
    /// interrupt_route 置排空位 ⇒ 仍算"点地寻路中"，撞击后续留到排空收尾；胜利 restep 落在怪格 (5,ROW) ≠ 停靠格 ⇒ 撞击作废，不撞门。
    /// 按键在点地之前按下（按住顶墙），故 press_key 清不到撞击；抬键只走 release_key（不清撞击）。
    #[test]
    fn t7_shell_interrupt_during_battle_suspension_stops_without_bump() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 5, ROW), (1, 8, ROW)]);
        hide_types(&mut sc, &[41]);
        strong(&mut sc);
        sc.player.yellow_keys = 1;
        let wt = sc.width_table.clone();
        sc.press_key(-1); // 按住顶墙
        sc.tick(0, &wt);
        assert_eq!((sc.player_cell_x, sc.walk_phase), (2, 0), "前置：顶墙不动");
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping);
        let mut fc = 1;
        while sc.walk_phase != 5 {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：途中开战");
        }
        assert!(sc.route.is_suspended() && sc.tap.tap_bump.is_some(), "前置：挂起中、撞击后续在");
        sc.interrupt_route(); // win route_key：抬键前先打断
        sc.release_key();
        assert!(!sc.route.is_suspended() && sc.walkto_draining, "前置：挂起解除、转排空");
        settle(&mut sc, fc, 400);
        let i41 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        assert!(sc.entities.items[i41].removed || sc.entities.items[i41].state == 1, "前置：怪被击杀");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "胜利 restep 落怪格即停");
        assert_eq!(sc.player.yellow_keys, 1, "未耗钥");
        assert!(door_intact(&sc, 8), "门未开");
        assert!(sc.tap.tap_bump.is_none() && sc.tap.tap_target.is_none() && !sc.route.is_open());
    }

    /// 跑到多步路线的末步在途窗口（`Map ∧ walk_phase == 1 ∧ tap_followup_ready`），返回下一拍帧号。
    fn run_to_last_step_window(sc: &mut GameScene, mut fc: i64) -> i64 {
        let wt = sc.width_table.clone();
        while !(sc.game_mode == GameMode::Map && sc.walk_phase == 1 && sc.tap.tap_followup_ready) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 300, "前置：末步在途窗口");
        }
        fc
    }

    /// 手动行走中（Map wp1、非点地寻路）的点地 / 滑动断言：点地 Rejected 且闪红、滑动 Rejected、无排队 / 撞击 / 就绪位 / 目标框。
    fn assert_manual_walk_rejects(sc: &mut GameScene, tap: (i32, i32), label: &str) {
        use crate::tap_walk::TapResult;
        assert!(sc.game_mode == GameMode::Map && sc.walk_phase == 1, "{label} 前置：手动行走中");
        assert!(!sc.tap.tap_followup_ready, "{label}：就绪位已作废");
        assert_eq!(sc.tap_cell(tap.0, tap.1), TapResult::Rejected, "{label}：手动行走中点地 = Rejected");
        assert!(matches!(sc.tap.tap_reject, Some((c, _)) if c == tap), "{label}：闪红");
        assert_eq!(sc.swipe_step(2), TapResult::Rejected, "{label}：手动行走中滑动 = Rejected");
        assert!(sc.tap.tap_pending.is_none() && sc.tap.tap_bump.is_none(), "{label}：无排队 / 撞击");
        assert_eq!(sc.tap.tap_target, None, "{label}：目标框清除");
    }

    /// 【计划 2 T7 复核 R1（探针 P1 / P11）· spec §5.2 更正③】末步在途窗口里按下并按住可走方向（左）：
    /// press_key 作废排队与就绪位；落格那一拍按 keyHeldCode 接上手动步 ⇒ 手动行走中点地闪红、滑动拒绝，松键后不执行旧排队。
    #[test]
    fn t7_held_key_in_last_step_window_takes_over() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Walking);
        let mut fc = run_to_last_step_window(&mut sc, 0);
        assert_eq!(sc.player_cell_x, 5, "前置：末步 (5,ROW)→(6,ROW) 在途");
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Queued, "前置：窗口里排队");
        sc.press_key(-3); // 左，按住
        let wt = sc.width_table.clone();
        while !(sc.player_cell_x == 6 && sc.walk_phase == 1 && sc.facing == 3) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 300, "前置：落格后接上手动步");
        }
        assert_manual_walk_rejects(&mut sc, (9, ROW), "窗口内按住");
        sc.release_key();
        settle(&mut sc, fc, 200);
        assert!(sc.player_cell_x < 6, "松键后停在手动步落格，不执行旧排队去 x=9（实得 {}）", sc.player_cell_x);
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map && sc.walk_phase == 0);
        assert!(sc.tap.tap_pending.is_none() && sc.tap.tap_target.is_none());
    }

    /// 【计划 2 T7 复核 R1（探针 P11b）】点地前已按住方向键（左，在 x=1 顶墙不动）再点门：路线照走；末步窗口里 keyHeldCode ≠ 0
    /// ⇒ tick_tap_state 作废就绪位与撞击；落格那一拍接上手动步（左）⇒ 手动行走中点地闪红、滑动拒绝，松键后不撞门。
    #[test]
    fn t7_key_held_before_tap_takes_over_after_last_step() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[(1, 8, ROW)]);
        apply_setup(&mut sc, SceneSetup { player_yellow_keys: Some(1), ..Default::default() });
        let wt = sc.width_table.clone();
        sc.press_key(-3); // 按住不放（press_key 发生在点地之前，清不到之后的撞击）
        sc.tick(0, &wt);
        assert_eq!((sc.player_cell_x, sc.walk_phase), (1, 0), "前置：顶墙不动");
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping);
        let mut fc = run_to_last_step_window(&mut sc, 1);
        assert_eq!(sc.player_cell_x, 6, "前置：末步 (6,ROW)→(7,ROW) 在途");
        sc.tick(fc, &wt);
        fc += 1;
        assert!(!sc.tap.tap_followup_ready && sc.tap.tap_bump.is_none(), "窗口内按住 ⇒ 就绪位与撞击作废");
        while !(sc.player_cell_x == 7 && sc.walk_phase == 1 && sc.facing == 3) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 300, "前置：落格后接上手动步");
        }
        assert_manual_walk_rejects(&mut sc, (3, ROW), "点地前按住");
        sc.release_key();
        settle(&mut sc, fc, 200);
        assert!(sc.player_cell_x < 7, "松键后停在手动步落格（实得 {}）", sc.player_cell_x);
        assert_eq!(sc.player.yellow_keys, 1, "未耗钥");
        assert!(door_intact(&sc, 8), "门未开");
        assert!(sc.tap.tap_bump.is_none() && sc.tap.tap_pending.is_none() && sc.tap.tap_target.is_none());
    }

    /// 【计划 2 T7 复核 R1】末步窗口内按下又在下一拍前松开（tick_tap_state 看不到 keyHeldCode）：press_key 本身作废就绪位 ⇒
    /// 这一步余下的拍里点地不再算"点地寻路中"（Rejected 闪红、不排队），落格后不改道。
    #[test]
    fn t7_key_tapped_in_last_step_window_clears_followup() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Walking);
        let fc = run_to_last_step_window(&mut sc, 0);
        sc.press_key(-2); // 下（墙）
        sc.release_key();
        assert!(!sc.tap.tap_followup_ready, "场上按键作废就绪位");
        assert_eq!(sc.tap_cell(3, ROW), TapResult::Rejected, "末步余下拍 = 手动输入后的行走：点地 Rejected");
        assert!(matches!(sc.tap.tap_reject, Some(((3, r), _)) if r == ROW), "闪红");
        assert!(sc.tap.tap_pending.is_none(), "不排队");
        settle(&mut sc, fc, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "落在原末步终点，不改道");
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map && sc.tap.tap_target.is_none());
    }

    /// 【计划 2 T7 复核 R2（探针 P10）· spec §5.2 更正④】走向怪途中再点同一只（同格 = 当前 tap_target）：不打断、不排队、
    /// 撞击保留 ⇒ 照常开战（不降级为选中）。
    /// 已知限制（保持现状）：多格怪点其**另一格**（≠ tap_target）按"点别处"排队；首点 / 再点时选中已清 ⇒ 落格执行为首点 = 选中，不开战。
    #[test]
    fn t7_retap_current_monster_while_approaching_still_fights() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 8, ROW)]);
        strong(&mut sc);
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Selected);
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping, "再点 = 开战");
        let bump = sc.tap.tap_bump;
        assert!(bump.is_some(), "前置：撞击后续在");
        let fc = walk_until_x(&mut sc, 4);
        let steps = sc.walk_step_count;
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping, "途中再点同一只 = 原路线继续");
        assert_eq!(sc.tap.tap_bump, bump, "撞击保留");
        assert!(sc.route.is_open() && !sc.walkto_draining, "未打断");
        assert_eq!(sc.walk_step_count, steps, "步序未清");
        assert!(sc.tap.tap_pending.is_none(), "不排队");
        assert_eq!(sc.tap.tap_target, Some((8, ROW)), "目标框不变");
        settle(&mut sc, fc, 400);
        let ent = &sc.entities.items[e];
        assert!(ent.removed || ent.state == 1, "照常开战并击杀");
        assert!(journal_has_combat(&mut sc), "战斗笔入栈");

        // 已知限制：多格怪 69（宽 3：(7..=9,ROW)）再点 (7,ROW) 开战后，途中点其另一格 (8,ROW) ⇒ 排队 ⇒ 落格后 = 选中、不开战
        let mut sc = scene((2, ROW), &[(69, 7, ROW)]);
        strong(&mut sc);
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 69).unwrap();
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Selected);
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Bumping, "再点同一只另一格 = 开战");
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Queued, "已知限制：另一格 ≠ tap_target ⇒ 按点别处排队");
        settle(&mut sc, fc, 200);
        assert_eq!(sc.tap.tap_selected.map(|s| s.entity), Some(e), "已知限制：落格执行 = 首点选中");
        let ent = &sc.entities.items[e];
        assert!(!ent.removed && ent.state == 0, "已知限制：未开战");
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map);
    }

    /// 【计划 2 T7 复核 R2】走向门途中再点门：不停顿（与不再点的对照路线同一拍开门），照常耗钥开门、走进门格。
    #[test]
    fn t7_retap_current_door_while_walking_keeps_route() {
        use crate::tap_walk::TapResult;
        let open_frame = |retap: bool| -> (i64, (i32, i32)) {
            let mut sc = scene((2, ROW), &[(1, 8, ROW)]);
            apply_setup(&mut sc, SceneSetup { player_yellow_keys: Some(1), ..Default::default() });
            assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping);
            let wt = sc.width_table.clone();
            let mut fc = 0;
            while sc.player.yellow_keys != 0 {
                if retap && sc.player_cell_x == 4 && sc.game_mode == GameMode::Dialog && sc.tap.tap_pending.is_none() {
                    let steps = sc.walk_step_count;
                    assert_eq!(sc.tap_cell(8, ROW), TapResult::Bumping, "途中再点门 = 原路线继续");
                    assert!(sc.route.is_open() && !sc.walkto_draining && sc.walk_step_count == steps, "未打断");
                    assert!(sc.tap.tap_bump.is_some() && sc.tap.tap_pending.is_none(), "撞击保留、不排队");
                }
                sc.tick(fc, &wt);
                fc += 1;
                assert!(fc < 400, "开门");
            }
            let open = fc;
            settle(&mut sc, fc, 100);
            assert!(!door_intact(&sc, 8), "门已开");
            assert!(sc.tap.tap_target.is_none() && sc.tap.tap_bump.is_none());
            (open, (sc.player_cell_x, sc.player_cell_y))
        };
        let control = open_frame(false);
        assert_eq!(control.1, (8, ROW), "对照：走进门格");
        assert_eq!(open_frame(true), control, "再点门不停顿：同一拍开门、同一落格");
    }

    /// 【计划 2 T7 复核 R2】排队后再点排队目标：仍 Queued，排队与目标框不变，落格后照常改道。
    #[test]
    fn t7_retap_queued_target_keeps_queue() {
        use crate::tap_walk::{PendingTap, TapResult};
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued, "再点排队目标 = 仍 Queued");
        assert_eq!(sc.tap.tap_pending, Some(PendingTap::Cell(2, ROW)), "排队不变");
        assert_eq!(sc.tap.tap_target, Some((2, ROW)), "目标框不变");
        settle(&mut sc, fc, 300);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "落格后照常改道");
        assert!(sc.tap.tap_pending.is_none() && sc.tap.tap_target.is_none());
    }

    /// 跑到排队收尾间隙（排空已落 (5,ROW)、Map wp0、就绪位在、排队 (2,ROW) 在），返回下一拍帧号。
    fn queued_followup_gap() -> (GameScene, i64) {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let mut fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        let wt = sc.width_table.clone();
        while !(sc.tap.tap_followup_ready && sc.game_mode == GameMode::Map && sc.walk_phase == 0) {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：收尾间隙");
        }
        assert_eq!(sc.player_cell_x, 5, "前置：排空落在 (5,ROW)");
        assert!(sc.tap.tap_pending.is_some(), "前置：排队在");
        (sc, fc)
    }

    /// 【计划 2 T7 复核 R4（变异 Mi）】收尾间隙里壳直接调 `walk_to`：新路线取代旧排队——走完后不再执行旧排队（回 x=2）。
    #[test]
    fn t7_walk_to_in_followup_gap_supersedes_queue() {
        let (mut sc, fc) = queued_followup_gap();
        assert!(sc.walk_to(8, ROW).reached, "间隙里 walk_to = 新路线");
        assert!(sc.tap.tap_pending.is_none(), "walk_to 清排队");
        settle(&mut sc, fc, 300);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (8, ROW), "新路线走完即停，不再执行旧排队");
        assert!(sc.tap.tap_pending.is_none() && sc.tap.tap_target.is_none());
    }

    /// 【计划 2 T7 复核 R4（变异 Mj）】收尾间隙里壳直接调 `probe_step`：探步取代旧排队——探步落格后不再执行旧排队（回 x=2）。
    #[test]
    fn t7_probe_step_in_followup_gap_supersedes_queue() {
        let (mut sc, fc) = queued_followup_gap();
        assert!(sc.probe_step(2), "间隙里探步（右）放行");
        assert!(sc.tap.tap_pending.is_none(), "probe_step 清排队");
        settle(&mut sc, fc, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "探步落 (6,ROW) 即停，不再执行旧排队");
        assert!(sc.tap.tap_pending.is_none() && sc.tap.tap_target.is_none());
    }

    // ------- Task 8：点地反馈绘制（spec §6，modern 专用、classic 零 op） -------

    fn marker_ops(sc: &GameScene) -> Vec<String> {
        let mut img = crate::render::ArgbImage::create(sc.layout.screen_w, sc.layout.screen_h);
        let mut g = crate::render::SoftGraphics::new(&mut img);
        sc.paint_tap_markers(&mut g, sc.view.cam_x, sc.view.cam_y + sc.layout.view_top_y);
        g.ops.clone()
    }

    fn full_frame_ops(sc: &mut GameScene) -> Vec<String> {
        let mut img = crate::render::ArgbImage::create(sc.layout.screen_w, sc.layout.screen_h);
        let mut g = crate::render::SoftGraphics::new(&mut img);
        sc.paint(&mut g, &crate::render::ArgbImage::create(2, 2));
        g.ops.clone()
    }

    fn put_markers(sc: &mut GameScene) {
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        sc.tap.tap_target = Some((5, ROW));
        sc.tap.tap_reject = Some(((4, ROW), sc.frame_counter + crate::tap_walk::TAP_REJECT_TICKS));
        sc.tap.tap_selected = Some(crate::tap_walk::SelectedEnemy { entity: e, type_: 41, cell: (8, ROW), floor: sc.persist.current_floor });
    }

    /// 【STAGE-B-008 · spec §6】classic：反馈层零 op，整帧 op 流与无点地状态时逐条相同。
    #[test]
    fn t8_markers_classic_zero_ops() {
        let mut a = scene((2, ROW), &[(41, 8, ROW)]);
        let mut b = scene((2, ROW), &[(41, 8, ROW)]);
        put_markers(&mut a);
        assert!(marker_ops(&a).is_empty(), "classic 反馈层零 op");
        assert_eq!(full_frame_ops(&mut a), full_frame_ops(&mut b), "classic 整帧 op 流不变");
    }

    /// modern：目标金框 / 闪红红框加叉（到期即不画）/ 选中闪烁框；火眼金睛才画预计掉血（弱 = "无法战胜"字形、
    /// 强 = 数字条，与观察模式同位：通用怪先 −ENTITY_LIFT(4) ⇒ y = 格顶 − 21）；多格怪 69 框宽 96；整帧里确有反馈层。
    #[test]
    fn t8_markers_modern_draw_target_reject_and_selection() {
        let mut sc = scene((2, ROW), &[(41, 8, ROW)]);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        put_markers(&mut sc);
        let (ox, oy) = (sc.view.cam_x, sc.view.cam_y + sc.layout.view_top_y);
        let ops = marker_ops(&sc);
        assert!(ops.contains(&format!("drawRect({},{},32,32,#ffe070)", ox + 5 * 32, oy + ROW * 32)), "目标金框：{ops:?}");
        assert!(ops.iter().any(|o| o.contains("#ff3030")), "闪红");
        assert!(ops.iter().any(|o| o.starts_with(&format!("drawRect({},{},32,32,", ox + 8 * 32, oy + ROW * 32))), "选中框");
        sc.frame_counter += crate::tap_walk::TAP_REJECT_TICKS;
        assert!(!marker_ops(&sc).iter().any(|o| o.contains("#ff3030")), "闪红到期不画");
        let without = marker_ops(&sc).len();
        sc.player.item_types[0] = 13;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        let ops = marker_ops(&sc);
        assert!(ops.len() > without, "持火眼金睛：多画预计掉血");
        let (m41x, m41y) = (ox + 8 * 32, oy + ROW * 32);
        assert!(
            ops.contains(&format!("setClip({},{},7,9)", m41x + 14, m41y - 21)),
            "弱方 = 无法战胜字形（观察模式同位：−ENTITY_LIFT−17）：{ops:?}"
        );
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
        let ops = marker_ops(&sc);
        assert!(!ops.iter().any(|o| o.ends_with(",7,9)")), "强方无字形：{ops:?}");
        assert!(
            ops.iter().any(|o| o.starts_with("setClip(") && o.ends_with(",2,24)")),
            "强方 = 数字条（夹具条 24×24 ⇒ 数位宽 2、高 24）：{ops:?}"
        );
        assert!(full_frame_ops(&mut sc).iter().any(|o| o.contains("#ffe070")), "整帧含反馈层");
    }

    /// 【T8 复核 R2】多格怪 69（宽 3 格）：选中框按实体宽 96 画；69 分支不抬 ⇒ 数字条 y = 格顶 − 17。
    #[test]
    fn t8_markers_multi_cell_monster_frame_and_no_lift() {
        let mut sc = scene((2, ROW), &[(69, 7, ROW)]);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 69).unwrap();
        sc.player.item_types[0] = 13;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
        sc.tap.tap_selected = Some(crate::tap_walk::SelectedEnemy { entity: e, type_: 69, cell: (7, ROW), floor: sc.persist.current_floor });
        let (ox, oy) = (sc.view.cam_x, sc.view.cam_y + sc.layout.view_top_y);
        let ops = marker_ops(&sc);
        assert!(
            ops.iter().any(|o| o.starts_with(&format!("drawRect({},{},96,32,", ox + 7 * 32, oy + ROW * 32))),
            "69 三格宽选中框：{ops:?}"
        );
        assert!(
            ops.iter().any(|o| o.starts_with("setClip(") && o.contains(&format!(",{},2,24)", oy + ROW * 32 - 17))),
            "69 不抬 ⇒ 数字条 y = 格顶 − 17：{ops:?}"
        );
    }

    // ------- T4：观察模式拖动 pan_viewport（spec §5.3） -------

        /// 【计划 3 T4】拖动平移：观察模式（wp2 未锁）按增量平移相机 + 主方向 facing；钳位同原版；
    /// 锁定期 / 非 wp2 / classic / 浮层 = 零变化。
    #[test]
    fn t4_pan_viewport_observe_drag() {
        let mut sc = scene((4, ROW), &[]);
        // 窄视口（宽 320 < 地图 416 ⇒ fits_w=false）：横向可平移（13×13 整层可见时 x 臂归零=原版语义）
        sc.apply_layout(crate::layout::Layout::modern(10, 13));
        // 进观察：-5（Map wp0）→ wp2 未锁定
        sc.key_value = -5;
        sc.handle_field_input();
        assert_eq!(sc.walk_phase, 2, "前置：观察模式");
        assert!(!sc.view_locked);
        let (cx0, cy0) = (sc.view.cam_x, sc.view.cam_y);
        // 向右拖 3 拍（每拍 16px）：内容跟手（cam+dx，复核 T4-R2 裁定）
        let mut moved = 0;
        for _ in 0..3 {
            let (dx, _) = sc.pan_viewport(16, 0);
            moved += dx;
        }
        assert_eq!(moved, 48, "三拍共移 48px（内容跟手）");
        assert_eq!(sc.view.cam_x, cx0 + moved, "拖右 = 内容右移（cam 增）");
        assert_eq!(sc.facing, 3, "镜头朝向 = 左（内容右移 ⇒ 看向更左）");
        // 上拖（dy<0 = 内容上移 = 镜头下）⇒ facing 下
        let _ = sc.pan_viewport(0, -16);
        assert_eq!(sc.facing, 0, "上拖 = 镜头下（facing 0）");
        // 对角（dx 优先；拖右 = 镜头左）
        let _ = sc.pan_viewport(16, -1);
        assert_eq!(sc.facing, 3, "对角 dx 优先（镜头左）");
        // 零增量
        let before = (sc.view.cam_x, sc.view.cam_y);
        assert_eq!(sc.pan_viewport(0, 0), (0, 0));
        assert_eq!((sc.view.cam_x, sc.view.cam_y), before);
        // 锁定期（回锁期）零变化（锁定态由 tick wp2 臂消费 -5 进入；纯查询直置）
        sc.view_locked = true;
        let (cx, cy) = (sc.view.cam_x, sc.view.cam_y);
        assert_eq!(sc.pan_viewport(32, 32), (0, 0), "回锁期拖动无效");
        assert_eq!((sc.view.cam_x, sc.view.cam_y), (cx, cy));
        // classic 零变化
        let mut cl = scene((4, ROW), &[]);
        apply_setup(&mut cl, SceneSetup { key_value: Some(-5), ..Default::default() });
        cl.handle_field_input();
        assert_eq!(cl.walk_phase, 2);
        let c0 = (cl.view.cam_x, cl.view.cam_y);
        assert_eq!(cl.pan_viewport(64, 64), (0, 0), "classic 拖动无效");
        assert_eq!((cl.view.cam_x, cl.view.cam_y), c0);
    }

    // ------- BUG-037：stair_hint（f_bool_13）生产置位（a.java:5765-5805 忠实移植）-------

    /// 【BUG-037】教程后玩家走到楼梯锚点四邻 ⇒ tick 置 stair_hint（此前生产恒 false=移植缺口）；
    /// 走离 ⇒ 清除；期间 49/55 换层键可用（门控链完整）。
    #[test]
    fn bug037_stair_hint_set_when_adjacent_and_gates_keys() {
        let mut sc = scene((2, ROW), &[]);
        apply_setup(&mut sc, SceneSetup { menu_tutorial_seen: Some(true), ..Default::default() });
        entity::spawn(&mut sc.entities, &mut sc.grid, 7, 4 << 5, ROW << 5, 10, &sc.width_table, None);
        // 非邻格：tick 不置位
        sc.tick(0, &sc.width_table.clone());
        assert!(!sc.menu.stair_hint, "非邻格不置位");
        // 走到左邻格 (3, ROW)：tick 置位（a.java:5790-5795）
        sc.player_cell_x = 3;
        sc.view.player_px = 3 << 5;
        sc.tick(1, &sc.width_table.clone());
        assert!(sc.menu.stair_hint, "邻格置位（左邻）");
        // 门控生效：49（筋斗云上）换层（change_floor strict 需层界含目标 10）
        sc.persist.min_floor = 1;
        sc.persist.max_floor = 55;
        sc.key_value = 49;
        sc.handle_field_input();
        assert_eq!(sc.persist.pending_floor, 10, "49 → 型 7 门换层");
        // 走离 → 清除
        sc.player_cell_x = 1;
        sc.view.player_px = 1 << 5;
        sc.tick(2, &sc.width_table.clone());
        assert!(!sc.menu.stair_hint, "走离清除");
        // 教程未亮：邻格也不置位
        sc.menu.tutorial_seen = false;
        sc.player_cell_x = 3;
        sc.view.player_px = 3 << 5;
        sc.tick(3, &sc.width_table.clone());
        assert!(!sc.menu.stair_hint, "tutorialSeen 未亮 ⇒ 不置位");
    }

    /// 【BUG-037】下梯锚（型 8）的 up 方向（a.java:5776-5779）与右邻方向（5797-5802）。
    #[test]
    fn bug037_stair_down_directions() {
        let mut sc = scene((2, ROW), &[]);
        apply_setup(&mut sc, SceneSetup { menu_tutorial_seen: Some(true), ..Default::default() });
        entity::spawn(&mut sc.entities, &mut sc.grid, 8, 2 << 5, (ROW - 1) << 5, 20, &sc.width_table, None);
        sc.tick(0, &sc.width_table.clone());
        assert!(sc.menu.stair_hint, "上邻下梯锚置位");
        let mut sc2 = scene((2, ROW), &[]);
        apply_setup(&mut sc2, SceneSetup { menu_tutorial_seen: Some(true), ..Default::default() });
        entity::spawn(&mut sc2.entities, &mut sc2.grid, 8, 3 << 5, ROW << 5, 20, &sc2.width_table, None);
        sc2.tick(0, &sc2.width_table.clone());
        assert!(sc2.menu.stair_hint, "右邻下梯锚置位");
    }
