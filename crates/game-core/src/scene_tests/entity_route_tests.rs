    use crate::render::FontMetrics;
    use super::scene_test_support::{scene, ROW};
    use super::*;

    /// 装载单行脚本并执行一条指令（dp0 一拍的 executeScriptInstruction）。
    fn exec(sc: &mut GameScene, line: &str) {
        sc.game_mode = GameMode::Dialog;
        sc.dialog.dialog_phase = 0;
        sc.script_line = line.to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();
    }

    fn slots(sc: &GameScene, x: i32, y: i32) -> Vec<usize> {
        let ct = sc.grid.cell_type[y as usize][x as usize] as usize;
        (0..sc.grid.capacity[ct] as usize).map(|s| sc.grid.slots[ct][s] as usize - 1).collect()
    }

    /// 型72 两步右行：MOV 臂字段（a.java:8289-8299）+ openWallOnRoute 每拍
    /// (routePhase, f_byte_19, 像素, routeProgress, 相机目标, 帧) 序列 + 步尽 relocateSlotsAtRouteEnd/收尾。
    #[test]
    fn route_72_tick_sequence() {
        let mut sc = scene((9, ROW), &[(72, 3, ROW)]);
        let e = sc.entity_at_cell(3, ROW, 72);
        assert!(e >= 0);
        let eu = e as usize;
        sc.route_phase = 7; // 脏值：MOV 必须复位为 0（a.java:8291）
        exec(&mut sc, "MOV_72_3_3_5_3 \u{1e}");
        assert_eq!(sc.dialog.dialog_phase, 2);
        assert_eq!(sc.route_entity, e);
        assert_eq!(sc.route_phase, 0);
        assert_eq!(sc.route_start, (3, ROW));
        assert_eq!(sc.dialog.dialog_target, (5, ROW));
        assert_eq!(sc.entities.items[eu].anim_row, 1, "型72 置动画行 1（a.java:8293-8294）");
        assert_eq!(sc.walk_path_buffer, vec![2, 2]);
        assert_eq!(sc.walk_step_count, 2);
        assert_eq!(sc.script_cursor, "MOV_72_3_3_5_3 ".len(), "游标越过空格分隔符");

        // (routePhase, f_byte_19, px, routeProgress, cameraTargetX, frame, 余步) 逐拍
        let expect = [
            (1, 2, 96, 0, None, 0, 1),        // case0 弹第 1 步（不动）
            (1, 2, 104, 8, Some(104), 1, 1),  // case1 ×4
            (1, 2, 112, 16, Some(112), 2, 1),
            (1, 2, 120, 24, Some(120), 3, 1),
            (1, 2, 128, 0, Some(128), 0, 0),  // 满 32 归格 → 同拍递归弹第 2 步
            (1, 2, 136, 8, Some(136), 1, 0),
            (1, 2, 144, 16, Some(144), 2, 0),
            (1, 2, 152, 24, Some(152), 3, 0),
        ];
        for (i, &(phase, dir, px, prog, tgt, frame, left)) in expect.iter().enumerate() {
            sc.tick_entity_route();
            assert_eq!(
                (sc.route_phase, sc.route_dir, sc.entities.items[eu].px, sc.route_progress),
                (phase, dir, px, prog),
                "拍 {i}"
            );
            if let Some(t) = tgt {
                assert_eq!((sc.ease_target_x, sc.ease_target_y), (t, ROW << 5), "拍 {i} 相机目标=实体像素");
            }
            assert_eq!(sc.entities.items[eu].anim_frame, frame, "拍 {i} 动画行1帧（表长 4 归 0）");
            assert_eq!(sc.walk_step_count, left, "拍 {i} 余步");
            assert_eq!(sc.dialog.dialog_phase, 2);
            assert_eq!(slots(&sc, 3, ROW), vec![eu], "途中槽表不动（只在收尾 relocateSlotsAtRouteEnd）");
        }
        // 第 2 步归格 → 递归 → 步尽：relocateSlotsAtRouteEnd + dp0 + dispatchAnimFields + route_entity=-1
        sc.tick_entity_route();
        assert_eq!(sc.entities.items[eu].px, 160);
        assert_eq!((sc.dialog.dialog_phase, sc.route_phase, sc.route_entity), (0, 0, -1));
        assert_eq!((sc.entities.items[eu].anim_row, sc.entities.items[eu].anim_frame), (0, 0), "animRowByType[72]=0");
        assert!(slots(&sc, 3, ROW).is_empty(), "relocateSlotsAtRouteEnd 自起点格摘除");
        assert_eq!(slots(&sc, 5, ROW), vec![eu], "relocateSlotsAtRouteEnd 追加到当前格");
        assert_eq!(sc.entity_at_cell(5, ROW, 72), e, "findBlockingEntity 可在新格命中（DES 依赖）");
        // 行尾 → dp4（script 收尾门）
        sc.execute_script_instruction();
        assert_eq!(sc.dialog.dialog_phase, 4);
    }

    /// 途中战斗（a.java:8765-8779 + 8861-8868）：非 72 型下一格=玩家格且可胜 →
    /// 战斗态字段 + 敌先手；case5 tickBattle(false) 击杀收尾 → dp0（不 restep，
    /// walkPhase 留 5——Java 原样，待 mode3 tickBattle 复位）。
    #[test]
    /// 批3 红1②兜底断言（对称 op_mov dialog_target 断言——dialogTarget 真盲区 dump 无字段，
    /// 夹具断言兜底；script_ops.rs:136 op_see 直写，plan:69 明文盲区项）。
    #[test]
    fn see_op_seeds_dialog_target() {
        use super::scene_test_support::scene as mk_scene;
        let mut sc = mk_scene((5, ROW), &[]);
        exec(&mut sc, "SEE_4_10_168_168_0 ");
        assert_eq!(sc.dialog.dialog_target, (4, 10), "op_see 装配 dialog_target（a.java:8487-8495）");
    }

    fn route_enemy_battle_when_stepping_into_player() {
        let t = 43u8;
        let mut sc = scene((5, ROW), &[(t, 3, ROW)]);
        let e = sc.entity_at_cell(3, ROW, t as i32);
        let eu = e as usize;
        sc.player.hp = 500;
        sc.player.atk = 1000;
        sc.player.def = 0;
        let ti = (t - 41) as usize;
        let loss = sc.predict_hp_loss(t as i32, false);
        assert!(loss >= 0 && loss < 500, "夹具须可胜: {loss}");
        exec(&mut sc, "MOV_43_3_3_7_3 \u{1e}");
        assert_eq!(sc.walk_step_count, 4);
        assert_eq!(sc.entities.items[eu].anim_row, entity::ANIM_TABLE_IDX[t as usize] as i32, "非 72 不改动画行");
        for _ in 0..5 {
            sc.tick_entity_route(); // 弹第 1 步 + 4 拍步进（第 4 拍递归弹第 2 步 → 开战）
        }
        assert_eq!(sc.entities.items[eu].px, 4 << 5);
        assert_eq!(sc.route_phase, 5);
        assert_eq!(sc.walk_phase, 5);
        assert!(!sc.battle_active, "battleActiveFlag=false（a.java:8771）");
        assert_eq!(sc.battle_target, e);
        assert_eq!(sc.entities.items[eu].state, 5);
        assert!(!sc.battle_no_counter);
        assert_eq!(sc.battle_enemy_hp, sc.enemies[ti].hp, "缩放表 HP（a.java:8769，非 bump 战的基础表）");
        let strike = sc.enemies[ti].atk.max(0);
        assert_eq!(sc.player.hp, 500 - strike, "敌先手一击");
        assert_eq!(sc.walk_step_count, 2, "余步保留（战斗收尾不再消费）");
        // case5：1 拍击杀 + 6 拍死亡动画
        for k in 0..6 {
            sc.tick_entity_route();
            assert_eq!(sc.dialog.dialog_phase, 2, "收尾中不退 dp（拍 {k}）");
        }
        sc.tick_entity_route();
        assert_eq!((sc.dialog.dialog_phase, sc.route_phase, sc.route_entity), (0, 0, -1));
        assert_eq!(sc.entities.items[eu].state, 1, "markEntityRemoved");
        assert_eq!(sc.battle_target, -1);
        assert_eq!(sc.walk_phase, 5, "无 restep（battleActiveFlag=false）——walkPhase 不复位");
        assert_eq!(sc.player_cell_x, 5, "玩家未动");
    }

    /// 不可胜（a.java:8780-8783 + 8850-8854）：case0 置 walkPhase=1/朝向；实体照走进
    /// 玩家格；归格复判仍不可胜 → scriptWalkArmed=false、gameMode=3、walkPhase=1、
    /// 朝向=步向，递归继续弹下一步（dialogPhase 留 2，路径挂起）。
    #[test]
    fn route_enemy_unbeatable_hands_back_to_mode3() {
        let t = 43u8;
        let mut sc = scene((5, ROW), &[(t, 3, ROW)]);
        let eu = sc.entity_at_cell(3, ROW, t as i32) as usize;
        sc.player.atk = 1; // ≤ 敌防 ⇒ predictBattleHpLoss = -1
        assert!(sc.predict_hp_loss(t as i32, false) < 0);
        sc.facing = 0;
        exec(&mut sc, "MOV_43_3_3_7_3 \u{1e}");
        sc.script_walk_armed = true;
        for _ in 0..5 {
            sc.tick_entity_route();
        }
        // 第 2 步弹出（下一格 = 玩家格）：不可胜分支
        assert_eq!((sc.route_phase, sc.walk_phase, sc.facing), (1, 1, 2));
        assert_eq!(sc.game_mode, GameMode::Dialog);
        for _ in 0..4 {
            sc.tick_entity_route();
        }
        assert_eq!(sc.entities.items[eu].px, 5 << 5, "实体走进玩家格");
        assert!(!sc.script_walk_armed);
        assert_eq!((sc.game_mode, sc.walk_phase, sc.facing), (GameMode::Map, 1, 2));
        assert_eq!(sc.dialog.dialog_phase, 2, "dialogPhase 不变（路径挂起）");
        assert_eq!((sc.route_phase, sc.walk_step_count), (1, 1), "同拍递归已弹第 3 步");
        assert_eq!(sc.route_entity, eu as i32);
    }

    /// 下一格倒序扫 cat1 型11 → markEntityRemoved（a.java:8786-8802）；同为 cat1 的
    /// 门（型1）不受影响。
    #[test]
    fn route_opens_wall_type11_only() {
        let mut sc = scene((11, ROW), &[(72, 3, ROW), (11, 4, ROW), (1, 6, ROW)]);
        let wall = sc.entity_at_cell(4, ROW, 11) as usize;
        let door = sc.entity_at_cell(6, ROW, 1) as usize;
        exec(&mut sc, "MOV_72_3_3_7_3 \u{1e}");
        assert_eq!(sc.walk_step_count, 4, "路径穿实体格（canPathThroughEntity）");
        sc.tick_entity_route(); // 弹第 1 步 → 下一格 (4,3) 有墙
        assert_eq!(sc.entities.items[wall].state, 1);
        assert_eq!((sc.entities.items[wall].sprite_w, sc.entities.items[wall].sprite_h), (45, 57));
        assert_eq!((sc.entities.items[wall].anim_row, sc.entities.items[wall].anim_frame), (2, 0));
        for _ in 0..8 {
            sc.tick_entity_route(); // 第 1/2 步步进；第 2 步归格弹第 3 步 → 下一格 (6,3)
        }
        assert_eq!(sc.walk_step_count, 1);
        assert_eq!(sc.entities.items[door].state, 0, "门不开");
    }

    /// 实体未命中：route_entity 写 -1（a.java:8289）+ 同拍尾递归执行下一条（8301-8302）。
    #[test]
    fn mov_entity_missing_runs_next_instruction_same_tick() {
        let mut sc = scene((9, ROW), &[]);
        sc.route_entity = 0;
        sc.facing = 0;
        exec(&mut sc, "MOV_72_3_3_5_3 ROS_4_2 \u{1e}");
        assert_eq!(sc.route_entity, -1);
        assert_eq!(sc.facing, 2, "ROS_4_2 同拍已执行");
        assert_eq!(sc.dialog.dialog_phase, 0);
        assert_eq!(sc.script_cursor, "MOV_72_3_3_5_3 ROS_4_2 ".len());
    }

    /// 首参 ≤0 走三参玩家分支（a.java:8277/8288 `var55 > 0`）——负值不按五参解析。
    #[test]
    fn mov_nonpositive_first_param_is_player_walk() {
        let mut sc = scene((3, ROW), &[]);
        exec(&mut sc, "MOV_-1_5_3 \u{1e}");
        assert_eq!(sc.dialog.dialog_target, (5, ROW));
        assert_eq!(sc.dialog.dialog_phase, 3);
        assert!(sc.script_walk_armed);
        assert_eq!(sc.walk_path_buffer, vec![2, 2]);
    }
    /// 2.3（refactor-behavior-matrix §2）：**MOV-miss 尾递归**（a.java:8301-8302）——
    /// 五参 MOV 实体未命中 ⇒ 同拍尾递归执行下一条；MOV段游标**分支前推进**
    /// （a.java:8268），递归自下一条起（游标位移断言钉）。
    #[test]
    fn mov_miss_tail_recurses_into_next_instruction() {
        let line = "MOV_43_1_1_2_2 TAK_5_6 ";
        // miss 路径：(1,1) 无 type43 ⇒ 尾递归同拍执行 TAK_5_6
        let mut sc = scene((5, ROW), &[]);
        exec(&mut sc, line);
        assert_eq!(sc.dialog.dialog_page, 5, "尾递归同拍执行 TAK（page=5）");
        assert_eq!(sc.dialog.dialog_page_end, 6);
        assert_eq!(sc.script_cursor, line.len(), "游标越过 MOV段与 TAK 段（TAK 尾 +1 落行末）");
        // 命中对照：不递归——游标只到 TAK 前、相位=2（实体路线）
        let mut sc2 = scene((5, ROW), &[(43, 1, 1)]);
        exec(&mut sc2, line);
        assert_eq!(sc2.dialog.dialog_phase, 2, "命中 ⇒ 实体路线相位（不递归）");
        assert_eq!(sc2.script_cursor, "MOV_43_1_1_2_2 ".len(), "游标停在 TAK 前");
        assert_eq!(sc2.dialog.dialog_page, 0, "TAK 未执行");
    }

    /// 5.x（refactor-behavior-matrix §5）：**双面漂移防线**（DC-9 桥漏道具加成
    /// 前科同机制）——16 op 两面分派对照：harness（script.rs ScriptEngine）与
    /// 生产（execute_script_instruction）喂同一行后**游标推进必须一致**；未知
    /// 3 字母 op 两面同停摆（R-TODO-1b）。只比分派/游标（效果面由 fx 夹具+金样
    /// 钉生产面、script_golden 钉 harness 面——双面分列不混谈）。
    #[test]
    fn script_surfaces_op_dispatch_cursor_parity() {
        use super::scene_test_support::scene as mk_scene;
        use crate::script::{DialogueTable, NoopHost, ScriptEngine};
        let table = DialogueTable {
            scripts: vec!["TAK_9_9 ".into(), "TAK_9_9 ".into()],
            texts: vec!["x".into(); 256], // 对话页取 texts[page]（TAK 翻页越界防护）
            speakers: vec![0; 256],
            type_names: vec!["x".into(); 256],
            type_categories: vec![0; 96],
        };
        let cases: &[(&str, &str)] = &[
            ("TAK", "TAK_8_9 "), ("LAY", "LAY_2 "), ("GIN", "GIN_1_1000 "),
            ("RES", "RES_0 "), ("ADD", "ADD_1_2_3_4 "), ("SEE", "SEE_4_10_168_168_0 "),
            ("GUT", "GUT_1 "), ("MOV", "MOV_0_5_11 "), ("DES", "DES_70_5_8 "),
            ("SWD", "SWD "), ("CES", "CES_84_6_11 "), ("GLV", "GLV_0 "),
            ("END", "END_0 "), ("SMS", "SMS_0 "), ("ROS", "ROS_4_1 "), ("MVS", "MVS "),
        ];
        for &(op, line) in cases {
            let mut sc = mk_scene((5, ROW), &[]);
            exec(&mut sc, line);
            let prod = sc.script_cursor;
            let mut eng = ScriptEngine::new(&table, 0);
            eng.set_script_line_for_test(line);
            eng.state.script_cursor = 0;
            let mut host = NoopHost;
            eng.execute_script_instruction(&mut host);
            assert_eq!(eng.state.script_cursor, prod, "op {op} 两面游标推进一致（line={line:?}）");
        }
        // 未知 op 两面同停摆（R-TODO-1b）
        for line in ["ZZZ_0 ", "QQ "] {
            let mut sc = mk_scene((5, ROW), &[]);
            exec(&mut sc, line);
            assert_eq!(sc.script_cursor, 0, "生产面未知 op 停摆（{line:?}）");
            let mut eng = ScriptEngine::new(&table, 0);
            eng.set_script_line_for_test(line);
            eng.state.script_cursor = 0;
            let mut host = NoopHost;
            eng.execute_script_instruction(&mut host);
            assert_eq!(eng.state.script_cursor, 0, "harness 面未知 op 停摆（{line:?}）");
        }
    }

