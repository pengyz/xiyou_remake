    use super::scene_test_support::{images, maplv, numbers, scene, SceneSetup, apply_setup};
    use super::*;

    /// ① 商人赠物路径（a.java:7333/7336 直调 addItemToItemStack，无前置写）：
    /// itemUseTargetType 由本函数自写，kind1 浮层图标/名随之取型 19。
    #[test]
    fn add_item_to_stack_writes_item_use_target_type() {
        let mut sc = scene((3, 3), &[]);
        apply_setup(&mut sc, SceneSetup { menu_overlay_kind_item: Some(0), ..Default::default() });
        sc.add_item_to_stack(19);
        assert_eq!(sc.menu.overlay_kind_item, 19);
        assert_eq!(sc.dialog.overlay_kind, 1);
        assert_eq!(sc.menu.overlay_item_name, sc.object_type_name(19), "浮层名取 itemUseTargetType");
        assert_eq!(sc.player.item_types[0], 19);
    }

    /// ② m_119：槽先置 null，仅 entityCount>0 才写——空表 = None（覆盖旧值），
    /// 有实体但全死 = Some([00 00])。
    #[test]
    fn save_floor_state_null_only_when_entity_table_empty() {
        let mut sc = scene((3, 3), &[]);
        assert_eq!(sc.entities.count, 0);
        sc.persist.saved_floor_states[2] = Some(vec![9, 9]);
        sc.save_floor_state();
        assert_eq!(sc.persist.saved_floor_states[2], None, "entityCount==0 → null（a.java:9383-9384）");

        let mut sc = scene((3, 3), &[(43, 5, 3)]);
        let e = sc.entity_at_cell(5, 3, 43) as usize;
        entity::remove(&mut sc.entities, e);
        sc.save_floor_state();
        assert_eq!(sc.persist.saved_floor_states[2], Some(vec![0, 0]), "有实体全死 → 计数 0 的 2 字节块");
    }

    /// ③ floor2 恢复特例：`!flags[26] && findBlockingEntity(11,4,84) < 0` → spawnEntity(84,352,128,0)。
    #[test]
    fn floor2_restore_respawns_84_only_while_script26_pending() {
        for (flag26, respawn) in [(false, true), (true, false)] {
            let mut sc = scene((3, 3), &[]);
            assert_eq!(sc.persist.floor, 2);
            sc.script_line_flags[26] = flag26;
            sc.persist.saved_floor_states[2] = Some(vec![0, 0]); // 恢复分支（0 实体）
            sc.load_floor_data(&maplv(), 3 << 5, 3 << 5, &[], sc.persist.floor).unwrap();
            assert_eq!(sc.entity_at_cell(11, 4, 84) >= 0, respawn, "flags[26]={flag26}");
        }
        // 非 2 层不补生
        let mut sc = scene((3, 3), &[]);
        apply_setup(&mut sc, SceneSetup { persist_floor: Some(3), ..Default::default() });
        sc.persist.saved_floor_states[3] = Some(vec![0, 0]);
        sc.load_floor_data(&maplv(), 3 << 5, 3 << 5, &[], sc.persist.floor).unwrap();
        assert!(sc.entity_at_cell(11, 4, 84) < 0);
    }

    /// ③b floor12 恢复特例（a.java:9653-9661）：型 78 两处 param 改写
    /// `lo | door13_hi[hi]<<8`（6|door13[5]<<8=1542、5|door13[4]<<8=773）。
    #[test]
    fn floor12_restore_rewrites_type78_params() {
        fn blob(recs: &[(u8, i32, i32, i16)]) -> Vec<u8> {
            let mut v = (recs.len() as u16).to_be_bytes().to_vec();
            for &(t, px, py, param) in recs {
                v.push(t);
                v.extend_from_slice(&(px as u16).to_be_bytes());
                v.extend_from_slice(&(py as u16).to_be_bytes());
                v.push(1);
                v.extend_from_slice(&(param as u16).to_be_bytes());
            }
            v
        }
        let mut sc = scene((3, 3), &[]);
        apply_setup(&mut sc, SceneSetup { persist_floor: Some(12), ..Default::default() });
        sc.persist.saved_floor_states[12] = Some(blob(&[
            (78, 1 << 5, 1 << 5, 9),
            (78, 11 << 5, 1 << 5, 9),
            (78, 5 << 5, 5 << 5, 9),
        ]));
        sc.load_floor_data(&maplv(), 3 << 5, 3 << 5, &[], 12).unwrap();
        let a = sc.entity_at_cell(1, 1, 78) as usize;
        let b = sc.entity_at_cell(11, 1, 78) as usize;
        let c = sc.entity_at_cell(5, 5, 78) as usize;
        // 值=按表索引现算（door13_hi[5]=3、[4]=6——a.java:902 表序逐字）
        assert_eq!(
            sc.entities.items[a].param,
            6 | ((DOOR13_HI[5] as i16) << 8),
            "(1,1) → 6|door13_hi[5]<<8（=774）"
        );
        assert_eq!(
            sc.entities.items[b].param,
            5 | ((DOOR13_HI[4] as i16) << 8),
            "(11,1) → 5|door13_hi[4]<<8（=1541）"
        );
        assert_eq!(sc.entities.items[c].param, 9, "无特例格的 78 参数不动");
    }

    /// ③c floor39 恢复特例（a.java:9664-9671）：型7@(11,1) → removeGroup +
    /// spawn(8,11,1,38)、型8@(11,11) → removeGroup + spawn(7,11,11,40)。
    /// **像素字面量保真**：spawn 实参=像素 ⇒ 两补生均落 cell(0,0)（javap 1446-1454/
    /// 1483-1492 常量逐字）——原版实况，D4 不"修正"。
    #[test]
    fn floor39_restore_swaps_types_with_pixel_literal_spawns() {
        fn blob(recs: &[(u8, i32, i32, i16)]) -> Vec<u8> {
            let mut v = (recs.len() as u16).to_be_bytes().to_vec();
            for &(t, px, py, param) in recs {
                v.push(t);
                v.extend_from_slice(&(px as u16).to_be_bytes());
                v.extend_from_slice(&(py as u16).to_be_bytes());
                v.push(1);
                v.extend_from_slice(&(param as u16).to_be_bytes());
            }
            v
        }
        let mut sc = scene((3, 3), &[]);
        apply_setup(&mut sc, SceneSetup { persist_floor: Some(39), ..Default::default() });
        sc.persist.saved_floor_states[39] = Some(blob(&[
            (7, 11 << 5, 1 << 5, 0),
            (8, 11 << 5, 11 << 5, 0),
            (43, 5 << 5, 5 << 5, 0),
        ]));
        sc.load_floor_data(&maplv(), 3 << 5, 3 << 5, &[], 39).unwrap();
        assert!(sc.entity_at_cell(11, 1, 7) < 0, "(11,1) 型7 已摘");
        assert!(sc.entity_at_cell(11, 11, 8) < 0, "(11,11) 型8 已摘");
        let p = sc.entity_at_cell(0, 0, 8);
        assert!(p >= 0, "补生 8 落 cell(0,0)（像素 11,1 原版实况）");
        assert_eq!(sc.entities.items[p as usize].param, 38);
        let q = sc.entity_at_cell(0, 0, 7);
        assert!(q >= 0, "补生 7 落 cell(0,0)（像素 11,11 原版实况）");
        assert_eq!(sc.entities.items[q as usize].param, 40);
        assert!(sc.entity_at_cell(5, 5, 43) >= 0, "无关实体不动");
    }

    /// ③d DOOR13_HI 与 game-data door13 表同值（两源对账——door13_hi
    /// a.java:902 `new byte[]{2,3,3,3,6,3,3,6,3,3,3,3,3}`）。
    #[test]
    fn door13_hi_table_matches_fixture() {
        let fixture: Vec<i32> = numbers(include_str!("../../tests/fixtures/door13.txt"));
        assert_eq!(DOOR13_HI.len(), fixture.len(), "表长一致");
        for (i, &v) in fixture.iter().enumerate() {
            assert_eq!(DOOR13_HI[i] as i32, v, "door13[{i}] 两源一致");
        }
    }

    /// ④ 读档旗标：前 scriptLineCount（68）项读回；之后的字节 Java skip（不读）。
    /// blob = preset-floor2-mov 的 MOT_L0（Java 侧同一预置实跑过，旗标段 [0]=1）。
    #[test]
    fn load_restores_script_flags_prefix_only() {
        let stores = crate::save::parse_preset(include_str!("../../tests/fixtures/preset-floor2-mov.preset"));
        let mut blob = stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).unwrap().1.clone();
        // 旗标段起点 = 37 + 2×日志对数（a.java:9258-9288 读序；本 blob 日志 0 对）
        assert_eq!(i16::from_be_bytes([blob[35], blob[36]]), 0);
        assert_eq!(blob[37], 1);
        blob[37 + 67] = 1; // scriptLineCount-1：读
        blob[37 + 100] = 1; // ≥ scriptLineCount：Java skip（a.java:9291-9297——2026-11-03 锚行更正）
        let save = crate::save::parse_floor_save(&blob, 0).unwrap();
        assert_eq!(save.script_flags.len(), 128);
        assert!(save.script_flags[0] && save.script_flags[67] && save.script_flags[100]);
        let width_table = numbers(include_str!("../../tests/fixtures/type_width_px.txt"));
        let walk = numbers(include_str!("../../tests/fixtures/tile_walkability.txt"));
        let sc =
            load_floor_from_save(&maplv(), &[], &width_table, &walk, images(), vec![None; 56], &save)
                .unwrap();
        assert_eq!(sc.dialog.dialog_data.scripts.len(), 68, "scriptLineCount = levelScriptLines.length");
        // 【P1-a1 黄②·断言换钉（a.java:2038-2039）+2026-11-03 更正】scriptLineFlags=
        // 新布尔[levelScriptLines.length]——levelScriptLines=全局 68 行表（a.java:1124）
        // 非"每层变长"（表述证伪更正）；68 槽形下 ≥68 的 skip 字节不在场（旧 128 缓冲的
        // [100] 反向断言=错位钉）。skip 链=save.script_flags[100] 真值仅存 save 侧
        // （a.java:9291-9297 不读入——锚行更正）。
        assert_eq!(sc.script_line_flags.len(), 68, "a.java:2038-2039 new boolean[levelScriptLines.length]");
        assert!(sc.script_line_flags[0] && sc.script_line_flags[67]);
        assert!(save.script_flags[100], "skip 字节仅存 save 侧（a.java:9291-9297 不读入）");
        assert_eq!(sc.script_line_flags.iter().filter(|f| **f).count(), 2);
    }

    /// 【P1-a1 黄五连③】道具栈 byte[32] 边界（Java byte[32] 对齐面）：可达上限
    /// **22 件**（型域 13..34=22 槽，叠加不入栈）——33 件不可经 add 达成（23+
    /// 不可达=BUG-050 潜在缺口注记）；满 32 静默吞=直接置位钉 `n < len` 守卫
    /// （Java=AIOOBE 分歧登记 BUG-050）。
    #[test]
    fn item_stack_byte32_boundary_and_overflow_silent() {
        let stores = crate::save::parse_preset(include_str!("../../tests/fixtures/preset-floor2-mov.preset"));
        let blob = stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).unwrap().1.clone();
        let save = crate::save::parse_floor_save(&blob, 0).unwrap();
        let width_table = numbers(include_str!("../../tests/fixtures/type_width_px.txt"));
        let walk = numbers(include_str!("../../tests/fixtures/tile_walkability.txt"));
        let mut sc = load_floor_from_save(&maplv(), &[], &width_table, &walk, images(), vec![None; 56], &save)
            .unwrap();
        assert_eq!(sc.player.item_types.len(), 32, "Java byte[32] 对齐（黄①收口面）");
        for i in 0..22 {
            sc.add_item_to_stack(13 + i); // 型域全域 22 件（叠加语义=同型不入栈）
        }
        assert_eq!(sc.player.item_stack_size, 22, "可达上限=型域 22 槽（33 件不可达——BUG-050）");
        sc.player.item_stack_size = 32; // 满栈直接置位（经 add 不可达）
        sc.add_item_to_stack(13); // 第 33 件：Rust `n < len` 守卫静默吞
        assert_eq!(sc.player.item_stack_size, 32, "溢出静默（Java=AIOOBE——BUG-050 分歧钉）");
    }
