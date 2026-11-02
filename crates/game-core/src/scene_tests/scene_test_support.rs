        use super::*;

    const W: i32 = 13;
    pub const ROW: i32 = 3;

    fn solid(w: i32, h: i32) -> ArgbImage {
        ArgbImage::from_rgb(&vec![0xFF202020; (w * h) as usize], w, h, true)
    }

    fn many(n: usize) -> Vec<ArgbImage> {
        (0..n).map(|_| solid(24, 24)).collect()
    }

    pub(crate) fn images() -> SceneImages {
        SceneImages {
            backdrop: solid(77, SCREEN_H),
            shadow: solid(24, 24),
            actor: solid(24, 24),
            actor_face: solid(24, 24),
            ui: many(25),
            stair_icons: (solid(24, 24), solid(24, 24)),
            open_anim: solid(24, 24),
            opened_door: solid(24, 24),
            entity: SceneImages::map_entity_images(&many(13), &many(23), &many(10), &many(20), &many(20)),
            popup: crate::popup::PopupImages {
                digit_strips: [solid(24, 24), solid(24, 24), solid(24, 24)],
                icon_strip: solid(24, 24),
                battle_digits: solid(24, 24),
            },
            spark_sheet: solid(24, 24),
            corner_sprites: solid(24, 24),
            map211: None,
            map210: solid(24, 24),
            menu_strip: solid(24, 24),
            battle_icon: solid(24, 24),
            battle_strip: solid(24, 24),
            faces: many(12),
            ending: solid(24, 24),
            xtq: many(6),
        }
    }

    pub(super) fn numbers(text: &str) -> Vec<i32> {
        text.lines().filter_map(|l| l.trim().parse().ok()).collect()
    }

    /// 合成 maplv：头 u16 LE（格数×2），地形段在 rebuildWalkability 索引
    /// （行距 wide·4、列距 2）置 tile 1（可走），其余 tile 0（墙）。
    pub(super) fn maplv() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&((W * 2) as u16).to_le_bytes());
        v.extend_from_slice(&((W * 2) as u16).to_le_bytes());
        let n = (W * W * 4) as usize;
        let mut terrain = vec![0u8; n];
        for x in 1..=11 {
            terrain[(ROW * W * 4 + x * 2) as usize] = 1;
        }
        v.extend_from_slice(&terrain);
        v.extend_from_slice(&vec![0u8; n]);
        v
    }

    /// 玩家格 + 实体 (type, cx, cy) 的场景；敌表 ×1 缩放 + 金币表。
    #[test]
    fn modern_paint_no_residual_across_states() {
        // 【A 黄2·DEADBEEF 钉升格】迁移链 dirty-canvas 零残留
        // （title→存档→游戏→游戏；原钉仅罩 BootMachine 单点=破坏态零旁证）。
        use crate::layout::Layout;
        use crate::paint::BootMachine;
        use crate::render::{ArgbImage, SoftGraphics};
        let img = || ArgbImage::create(2, 2);
        let vec = |n: usize| (0..n).map(|_| img()).collect::<Vec<_>>();
        let mut m = BootMachine::new(img(), img(), vec(8), vec(25), vec(2), img(), vec(2), vec(1), vec(2));
        m.apply_layout(Layout::modern(18, 18));
        for _ in 0..200 {
            m.tick(0); // → MainMenu（title/存档族态）
        }
        let mut canvas = ArgbImage::create(576, 576);
        for px in canvas.argb.iter_mut() {
            *px = 0xDEADBEEF;
        }
        {
            let mut g = SoftGraphics::new(&mut canvas);
            m.paint(&mut g);
        }
        assert_eq!(
            canvas.argb.iter().filter(|p| **p == 0xDEADBEEF).count(),
            0,
            "title/menu 态零残留"
        );
        // 游戏态（GameScene::paint 入口面——A 黄2 点名 scene.rs:5981）
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18)); // Modern 面（清屏门控域）
        let mut canvas2 = ArgbImage::create(240, 320);
        for px in canvas2.argb.iter_mut() {
            *px = 0xDEADBEEF;
        }
        {
            let mut g = SoftGraphics::new(&mut canvas2);
            sc.paint(&mut g, &img());
        }
        assert_eq!(
            canvas2.argb.iter().filter(|p| **p == 0xDEADBEEF).count(),
            0,
            "游戏态零残留（paint 入口全幅自铺）"
        );
        // 再迁移（对话/再游戏帧序）：同画布复绘零残留
        {
            let mut g = SoftGraphics::new(&mut canvas2);
            sc.paint(&mut g, &img());
        }
        assert_eq!(
            canvas2.argb.iter().filter(|p| **p == 0xDEADBEEF).count(),
            0,
            "迁移后复绘零残留"
        );
    }

    #[test]
    fn item_use_two_stage_sequence() {
        // 【批三】道具使用面三段：tap1=选中+预览、tap2 同行=执行、打断=不执行。
        let mut sc = scene((2, 2), &[]);
        sc.game_mode = GameMode::ItemPanel;
        assert!(!sc.ui_tap_two_stage(3), "tap1=选中不执行");
        assert_eq!(sc.ui_selected.map(|(r, _)| r), Some(3), "选中态");
        assert!(!sc.ui_tap_two_stage(3), "【Y1】连击 tap2 抑制");
        sc.ui_frame += crate::menu_family::TWO_STAGE_MIN_STEPS;
        assert!(sc.ui_tap_two_stage(3), "tap2 同行=执行（≥阈值）");
        assert_eq!(sc.ui_selected, None, "执行后清选中");
        sc.ui_selected = Some((1, sc.ui_frame));
        sc.ui_interrupt();
        assert_eq!(sc.ui_selected, None, "打断清选中不执行");
        assert!(!sc.ui_tap_two_stage(0), "换行 tap1");
        assert!(!sc.ui_tap_two_stage(2), "换行=换选不执行");
        assert_eq!(sc.ui_selected.map(|(r, _)| r), Some(2));
    }

    #[test]
    fn equip_replace_two_stage_sequence() {
        // 【批三】装备替换面三段（ItemPanel 装备行域——同机制另一面钉）
        let mut sc = scene((2, 2), &[]);
        sc.game_mode = GameMode::ItemPanel;
        assert!(!sc.ui_tap_two_stage(7), "装备格 tap1=选中");
        assert_eq!(sc.ui_selected.map(|(r, _)| r), Some(7));
        sc.ui_frame += crate::menu_family::TWO_STAGE_MIN_STEPS;
        assert!(sc.ui_tap_two_stage(7), "装备格 tap2=替换执行（≥阈值）");
        sc.ui_selected = Some((7, sc.ui_frame));
        sc.ui_interrupt();
        assert_eq!(sc.ui_selected, None, "打断不替换");
    }

    pub fn scene(player: (i32, i32), ents: &[(u8, i32, i32)]) -> GameScene {
        let width_table = numbers(include_str!("../../tests/fixtures/type_width_px.txt"));
        let walk = numbers(include_str!("../../tests/fixtures/tile_walkability.txt"));
        assert_eq!((walk[0], walk[1]), (0, 1), "合成 tile 0=墙 1=可走");
        let recs: Vec<(u8, u16, u16, i16, bool)> = ents
            .iter()
            .map(|&(t, x, y)| (t, (x << 5) as u16, (y << 5) as u16, 0, true))
            .collect();
        let mut sc = GameScene::load_floor(
            &maplv(),
            player.0 << 5,
            player.1 << 5,
            &recs,
            &width_table,
            &walk,
            images(),
            vec![None; 56],
        )
        .unwrap();
        sc.persist.floor = 2;
        sc.persist.current_floor = 2;
        let tables = crate::combat::CombatTables {
            enemy_base_hp: numbers(include_str!("../../tests/fixtures/enemybasehp.txt")),
            enemy_base_atk: numbers(include_str!("../../tests/fixtures/enemybaseatk.txt")),
            enemy_base_def: numbers(include_str!("../../tests/fixtures/enemybasedef.txt")),
            difficulty_multipliers: numbers(include_str!("../../tests/fixtures/difficulty_multipliers.txt")),
        };
        sc.enemies = crate::combat::scale_enemy_stats(&tables, 1);
        sc.enemy_base_hp = tables.enemy_base_hp.clone();
        sc.enemy_base_gold = numbers(include_str!("../../tests/fixtures/enemybasegold.txt"));
        sc
    }
