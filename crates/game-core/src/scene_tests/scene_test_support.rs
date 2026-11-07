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
        assert_eq!(sc.tap.ui_selected.map(|(r, _)| r), Some(3), "选中态");
        assert!(!sc.ui_tap_two_stage(3), "【Y1】连击 tap2 抑制");
        sc.tap.ui_frame += crate::menu_family::TWO_STAGE_MIN_STEPS;
        assert!(sc.ui_tap_two_stage(3), "tap2 同行=执行（≥阈值）");
        assert_eq!(sc.tap.ui_selected, None, "执行后清选中");
        sc.tap.ui_selected = Some((1, sc.tap.ui_frame));
        sc.ui_interrupt();
        assert_eq!(sc.tap.ui_selected, None, "打断清选中不执行");
        assert!(!sc.ui_tap_two_stage(0), "换行 tap1");
        assert!(!sc.ui_tap_two_stage(2), "换行=换选不执行");
        assert_eq!(sc.tap.ui_selected.map(|(r, _)| r), Some(2));
    }

    #[test]
    fn equip_replace_two_stage_sequence() {
        // 【批三】装备替换面三段（ItemPanel 装备行域——同机制另一面钉）
        let mut sc = scene((2, 2), &[]);
        sc.game_mode = GameMode::ItemPanel;
        assert!(!sc.ui_tap_two_stage(7), "装备格 tap1=选中");
        assert_eq!(sc.tap.ui_selected.map(|(r, _)| r), Some(7));
        sc.tap.ui_frame += crate::menu_family::TWO_STAGE_MIN_STEPS;
        assert!(sc.ui_tap_two_stage(7), "装备格 tap2=替换执行（≥阈值）");
        sc.tap.ui_selected = Some((7, sc.tap.ui_frame));
        sc.ui_interrupt();
        assert_eq!(sc.tap.ui_selected, None, "打断不替换");
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
        let __gold: Vec<i32> = numbers(include_str!("../../tests/fixtures/enemybasegold.txt"));
        sc.enemy_base = tables.to_records(&__gold);
        sc.enemies = crate::combat::scale_enemy_stats(&sc.enemy_base, 1);
        sc
    }

/// 【P2 批10 D·域构造器】测试构造面收敛（plan:37/§4.6）：scene 构造行后连续纯赋值 run=
/// 构造输入非运行时写——机械迁移为本配置结构体（构造段 112 站点→struct 字面量，40 字段=
/// sed 实测观察字段全集）。穿插站点=测试逻辑（plan 自认「主体为测试逻辑非直填」）留原位
/// 登记；复合算符站点同。夹具字节等价（金样零漂移）。313375b 机械迁移先例同口径。
#[derive(Default, Debug)]
pub struct SceneSetup {
    pub alchemy_upgrade_count: Option<i32>,
    pub dialog_dialog_data_scripts: Option<Vec<String>>,
    pub dialog_dialog_phase: Option<i32>,
    pub dialog_overlay_text: Option<Option<String>>,
    pub equipped_armor: Option<i32>,
    pub equipped_weapon: Option<i32>,
    pub facing: Option<i32>,
    pub game_mode: Option<GameMode>,
    pub images_backdrop: Option<crate::scene::ArgbImage>,
    pub key_value: Option<i32>,
    pub layout: Option<crate::layout::Layout>,
    pub menu_difficulty_index: Option<i32>,
    pub menu_menu_box_width: Option<i32>,
    pub menu_menu_cursor: Option<i32>,
    pub menu_menu_item_count: Option<i32>,
    pub menu_menu_kinds: Option<Vec<i8>>,
    pub menu_menu_line_height: Option<i32>,
    pub menu_menu_list_height: Option<i32>,
    pub menu_menu_scroll_top: Option<i32>,
    pub menu_menu_visible_bottom: Option<i32>,
    pub menu_overlay_kind_item: Option<i32>,
    pub menu_stair_hint: Option<bool>,
    pub menu_tutorial_seen: Option<bool>,
    pub persist_current_floor: Option<i32>,
    pub persist_floor: Option<i32>,
    pub persist_journal: Option<Vec<(i32, i32)>>,
    pub persist_max_floor: Option<i32>,
    pub persist_min_floor: Option<i32>,
    pub persist_preset_mot_if: Option<Option<Vec<u8>>>,
    pub player_atk: Option<i32>,
    pub player_blue_keys: Option<i32>,
    pub player_def: Option<i32>,
    pub player_gold: Option<i32>,
    pub player_hp: Option<i32>,
    pub player_red_keys: Option<i32>,
    pub player_yellow_keys: Option<i32>,
    pub route_entity: Option<i32>,
    pub script_cursor: Option<usize>,
    pub script_line: Option<String>,
    pub walk_phase: Option<i32>,
}

/// 域构造器入口（scene() 变体通用：构造行后以本配置施加构造输入）。
pub fn scene_setup(player: (i32, i32), ents: &[(u8, i32, i32)], setup: SceneSetup) -> GameScene {
    let mut sc = scene(player, ents);
    apply_setup(&mut sc, setup);
    sc
}

pub fn apply_setup(sc: &mut GameScene, s: SceneSetup) {
    if let Some(v) = s.alchemy_upgrade_count { sc.alchemy_upgrade_count = v; }
    if let Some(v) = s.dialog_dialog_data_scripts { sc.dialog.dialog_data.scripts = v; }
    if let Some(v) = s.dialog_dialog_phase { sc.dialog.dialog_phase = v; }
    if let Some(v) = s.dialog_overlay_text { sc.dialog.overlay_text = v; }
    if let Some(v) = s.equipped_armor { sc.equipped_armor = v; }
    if let Some(v) = s.equipped_weapon { sc.equipped_weapon = v; }
    if let Some(v) = s.facing { sc.facing = v; }
    if let Some(v) = s.game_mode { sc.game_mode = v; }
    if let Some(v) = s.images_backdrop { sc.images.backdrop = v; }
    if let Some(v) = s.key_value { sc.key_value = v; }
    if let Some(v) = s.layout { sc.layout = v; }
    if let Some(v) = s.menu_difficulty_index { sc.menu.difficulty_index = v; }
    if let Some(v) = s.menu_menu_box_width { sc.menu.menu_box_width = v; }
    if let Some(v) = s.menu_menu_cursor { sc.menu.menu_cursor = v; }
    if let Some(v) = s.menu_menu_item_count { sc.menu.menu_item_count = v; }
    if let Some(v) = s.menu_menu_kinds { sc.menu.menu_kinds = v; }
    if let Some(v) = s.menu_menu_line_height { sc.menu.menu_line_height = v; }
    if let Some(v) = s.menu_menu_list_height { sc.menu.menu_list_height = v; }
    if let Some(v) = s.menu_menu_scroll_top { sc.menu.menu_scroll_top = v; }
    if let Some(v) = s.menu_menu_visible_bottom { sc.menu.menu_visible_bottom = v; }
    if let Some(v) = s.menu_overlay_kind_item { sc.menu.overlay_kind_item = v; }
    if let Some(v) = s.menu_stair_hint { sc.menu.stair_hint = v; }
    if let Some(v) = s.menu_tutorial_seen { sc.menu.tutorial_seen = v; }
    if let Some(v) = s.persist_current_floor { sc.persist.current_floor = v; }
    if let Some(v) = s.persist_floor { sc.persist.floor = v; }
    if let Some(v) = s.persist_journal { sc.persist.journal = v; }
    if let Some(v) = s.persist_max_floor { sc.persist.max_floor = v; }
    if let Some(v) = s.persist_min_floor { sc.persist.min_floor = v; }
    if let Some(v) = s.persist_preset_mot_if { sc.persist.preset_mot_if = v; }
    if let Some(v) = s.player_atk { sc.player.atk = v; }
    if let Some(v) = s.player_blue_keys { sc.player.blue_keys = v; }
    if let Some(v) = s.player_def { sc.player.def = v; }
    if let Some(v) = s.player_gold { sc.player.gold = v; }
    if let Some(v) = s.player_hp { sc.player.hp = v; }
    if let Some(v) = s.player_red_keys { sc.player.red_keys = v; }
    if let Some(v) = s.player_yellow_keys { sc.player.yellow_keys = v; }
    if let Some(v) = s.route_entity { sc.route_entity = v; }
    if let Some(v) = s.script_cursor { sc.script_cursor = v; }
    if let Some(v) = s.script_line { sc.script_line = v; }
    if let Some(v) = s.walk_phase { sc.walk_phase = v; }
}
