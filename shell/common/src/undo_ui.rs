//! T3 撤销按钮执行面 + 撤销 UI（touch-native-undo-plan §0 边界 3）。
//!
//! game-core **零改动**红线下的接缝处置（裁定，ledger 见 commit）：T1
//! `UndoTarget` trait 文档写明"T2/T3 在 Scene 上实现"，但 core 零改红线不许落
//! impl ⇒ 落**壳侧 newtype 适配器**（trait 外部、类型本地，orphan 规则允许），
//! 经 scene pub 状态面逆放。语义锚：exp 恒 0（本作无经验字段，scene.rs:5238）、
//! 敌复活=开战前整格快照还原 + 按需重登格槽（BUG-045）。
//!
//! BACK 语义不动（-7 保真锚）：撤销只走按钮，不占用任何键码。

use game_core::scene::GameScene;
use game_core::undo_journal::{
    DoorState, EnemyRestore, PlayerVitals, UndoOutcome, UndoTarget,
};

/// [`UndoTarget`] 适配器（Scene 状态副本面；逆放只写状态、不碰推进语义）。
pub struct SceneUndo<'a>(pub &'a mut GameScene);

impl UndoTarget for SceneUndo<'_> {
    fn set_player_tile(&mut self, tile: (i32, i32)) {
        let sc = &mut *self.0;
        sc.player_cell_x = tile.0;
        sc.player_cell_y = tile.1;
        sc.view.player_px = tile.0 << 5;
        sc.view.player_py = tile.1 << 5;
        sc.step_progress = 0;
        sc.walk_phase = 0;
    }

    fn set_player_vitals(&mut self, v: PlayerVitals) {
        // exp 恒 0（本作无经验字段——T1 通用字段，scene.rs:5238 同锚）
        self.0.player.hp = v.hp;
        self.0.player.gold = v.gold;
    }

    fn revive_enemy(&mut self, e: EnemyRestore) {
        let sc = &mut *self.0;
        if e.slot >= sc.entities.items.len() {
            return;
        }
        // 【BUG-045①】死亡动画后已摘槽（removed=true）⇒ 还原后须重登格槽，否则不画、不挡路
        let cur = sc.entities.items[e.slot];
        // 身份校验（复核 T3-R3）：槽里须仍是这只怪的尸体（同型同位、已死或死亡动画中）；实体表若被重排（跨层探针重载，
        // 已清栈）则不盲写别的实体
        if cur.type_ != e.entity.type_ || cur.px != e.entity.px || cur.py != e.entity.py || !(cur.removed || cur.state == 1) {
            return;
        }
        let detached = cur.removed;
        sc.entities.items[e.slot] = e.entity; // 开战前整格快照（type / 位置 / 精灵 / 动画 / state=0 / visible）
        if detached {
            game_core::entity::reattach_to_cell(&mut sc.grid, &mut sc.entities, e.slot);
        }
        // 【BUG-045②】不写 sc.enemies.hp：该表按 type−41 索引的全局属性表，不是实体血量
    }

    fn set_item_count(&mut self, item_id: i32, count: i32) {
        // 【裁定】item_id 域：0/1/2=黄/蓝/红钥匙；其余=物品栈类型（item_uses）。
        // 上游无拾取/消耗笔生产者（T1/T2 未落构造点）⇒ 域未定；真生产者接线时
        // 若需改映射=只动本 match。成本：一处重映射。
        let sc = &mut *self.0;
        match item_id {
            0 => sc.player.yellow_keys = count,
            1 => sc.player.blue_keys = count,
            2 => sc.player.red_keys = count,
            t => {
                let idx = sc.player.find_item(t as u8);
                if idx >= 0 {
                    sc.player.item_uses[idx as usize] = count.clamp(0, 255) as u8;
                }
            }
        }
    }

    fn set_door_state(&mut self, cx: i32, cy: i32, state: DoorState) {
        let sc = &mut *self.0;
        for ent in sc.entities.items.iter_mut().take(sc.entities.count) {
            // 门实体 type 1|2|3（黄/红/蓝，interact_with_cell a.java:5391 族）
            if (1..=3).contains(&ent.type_) && ent.px >> 5 == cx && ent.py >> 5 == cy {
                ent.state = match state {
                    DoorState::Open => 1,    // markEntityRemoved
                    DoorState::Locked => 0,  // 复锁（状态副本）
                };
            }
        }
    }
}

/// 可回撤步数（撤销按钮"可回撤 N 步"数据面；plan §0 边界 3）。
pub fn undo_count(sc: &GameScene) -> usize {
    sc.undo.len()
}

/// 撤销一笔（journal LIFO 逆放）。战斗进行中= [`UndoOutcome::Blocked`]（中途
/// 不可撤，T1 契约）；栈空= [`UndoOutcome::Empty`]。
pub fn undo_last(sc: &mut GameScene) -> UndoOutcome {
    // 【STAGE-B-008 · spec §7】撤销同时清目标标记、怪物选中、排队动作
    sc.clear_tap_state();
    // journal 借出规避 sc.undo/self 双可变借用（core 零改 ⇒ 无 scene 级入口）
    let mut journal = std::mem::take(&mut sc.undo);
    let out = journal.undo(&mut SceneUndo(sc));
    sc.undo = journal;
    out
}

// ---------------------------------------------------------------- 撤销按钮 UI

fn glyph5(ch: char) -> [u8; 5] {
    match ch {
        'U' => [0b101, 0b101, 0b101, 0b101, 0b011],
        'N' => [0b110, 0b101, 0b101, 0b101, 0b101],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
        ' ' => [0, 0, 0, 0, 0],
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b110, 0b001, 0b010, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b011, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b011, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b110],
        _ => [0, 0, 0, 0, 0],
    }
}

/// 撤销按钮 UI（常驻"UNDO N"；窗口/背板缓冲上绘制，**不进游戏画布**——
/// canvas/oracle 金样零污染）。scale=光栅倍率（逻辑→物理 ×scale）。
pub fn draw_undo_button(
    buf: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    x: i32,
    y: i32,
    count: usize,
    scale: i32,
) {
    let s = scale.max(1);
    let (w, h) = (56 * s, 20 * s);
    // 底：半透明深色胶囊 + 亮描边（count=0 灰显=不可回撤）
    let face: u32 = if count > 0 { 0xF020_3048 } else { 0x8030_3030 };
    let edge: u32 = if count > 0 { 0xFF60_C0FF } else { 0xFF60_6060 };
    fill(buf, buf_w, buf_h, x, y, w, h, face);
    frame(buf, buf_w, buf_h, x, y, w, h, edge, s);
    // 文本 "UNDO {n}"（3x5 字模 ×s）
    let txt = format!("UNDO {count}");
    let mut cx = x + 4 * s;
    let cy = y + 6 * s;
    for ch in txt.chars().take(9) {
        for (gy, row) in glyph5(ch).iter().enumerate() {
            for gx in 0..3 {
                if row & (0b100 >> gx) != 0 {
                    fill(buf, buf_w, buf_h, cx + gx as i32 * s, cy + gy as i32 * s, s, s, 0xFFFF_FFFF);
                }
            }
        }
        cx += 4 * s;
    }
}

fn put(buf: &mut [u32], bw: usize, bh: usize, x: i32, y: i32, c: u32) {
    if x >= 0 && y >= 0 && (x as usize) < bw && (y as usize) < bh {
        buf[y as usize * bw + x as usize] = c;
    }
}

fn fill(buf: &mut [u32], bw: usize, bh: usize, x: i32, y: i32, w: i32, h: i32, c: u32) {
    for dy in 0..h {
        for dx in 0..w {
            put(buf, bw, bh, x + dx, y + dy, c);
        }
    }
}

fn frame(buf: &mut [u32], bw: usize, bh: usize, x: i32, y: i32, w: i32, h: i32, c: u32, t: i32) {
    fill(buf, bw, bh, x, y, w, t, c);
    fill(buf, bw, bh, x, y + h - t, w, t, c);
    fill(buf, bw, bh, x, y, t, h, c);
    fill(buf, bw, bh, x + w - t, y, t, h, c);
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::render::ArgbImage;
    use game_core::scene::{FloorAssets, SceneImages};
    use game_core::undo_journal::{CombatStroke, MoveStroke};

    fn solid(w: i32, h: i32) -> ArgbImage {
        ArgbImage::from_rgb(&vec![0xFF80_8080; (w * h) as usize], w, h, true)
    }
    fn many(n: usize) -> Vec<ArgbImage> {
        (0..n).map(|_| solid(24, 24)).collect()
    }

    fn fixture_numbers(name: &str) -> Vec<i32> {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        std::fs::read_to_string(repo.join("crates/game-core/tests/fixtures").join(name))
            .unwrap()
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect()
    }

    /// 轻量 scene（scene_test_support 同构：合成图像 + 仓库夹具 maplv1 + 真实数值表）。
    fn scene_with(player: (i32, i32), ents: &[(u8, i32, i32)]) -> GameScene {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let fix = repo.join("crates/game-core/tests/fixtures");
        let width_table = fixture_numbers("type_width_px.txt");
        let walk = fixture_numbers("tile_walkability.txt");
        let maplv = std::fs::read(fix.join("maplv1.bin")).unwrap();
        let images = SceneImages {
            backdrop: solid(77, 320),
            shadow: solid(24, 24),
            actor: solid(24, 24),
            actor_face: solid(24, 24),
            ui: many(25),
            stair_icons: (solid(24, 24), solid(24, 24)),
            open_anim: solid(24, 24),
            opened_door: solid(24, 24),
            entity: SceneImages::map_entity_images(&many(13), &many(23), &many(10), &many(20), &many(20)),
            popup: game_core::popup::PopupImages {
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
        };
        let recs: Vec<(u8, u16, u16, i16, bool)> =
            ents.iter().map(|&(t, x, y)| (t, (x << 5) as u16, (y << 5) as u16, 0, true)).collect();
        let floors: Vec<Option<FloorAssets>> = (0..56).map(|_| None).collect();
        let mut sc = GameScene::load_floor(&maplv, player.0 << 5, player.1 << 5, &recs, &width_table, &walk, images, floors)
            .unwrap();
        let tables = game_core::combat::CombatTables {
            enemy_base_hp: fixture_numbers("enemybasehp.txt"),
            enemy_base_atk: fixture_numbers("enemybaseatk.txt"),
            enemy_base_def: fixture_numbers("enemybasedef.txt"),
            difficulty_multipliers: fixture_numbers("difficulty_multipliers.txt"),
        };
        sc.enemies = game_core::combat::scale_enemy_stats(&tables, 1);
        sc.enemy_base_hp = tables.enemy_base_hp.clone();
        sc.enemy_base_gold = fixture_numbers("enemybasegold.txt");
        sc
    }

    fn scene() -> GameScene {
        scene_with((3, 10), &[])
    }

    #[test]
    fn undo_move_restores_tile_and_count() {
        let mut sc = scene();
        // 手动推位移笔（上游 walk_to 已在 core 侧入栈；此处直接钉逆放面）
        sc.undo.push_move(MoveStroke { from: (3, 10), to: (5, 10) });
        sc.player_cell_x = 5;
        sc.player_cell_y = 10;
        sc.view.player_px = 5 << 5;
        sc.view.player_py = 10 << 5;
        assert_eq!(undo_count(&sc), 1, "可回撤 1 步（按钮数据面）");
        let out = undo_last(&mut sc);
        assert!(matches!(out, UndoOutcome::Undone(_)), "撤销成功：{out:?}");
        assert_eq!(
            (sc.player_cell_x, sc.player_cell_y, sc.view.player_px, sc.view.player_py),
            (3, 10, 3 << 5, 10 << 5),
            "位置回起点（格+像素双面）"
        );
        assert_eq!(undo_count(&sc), 0, "栈减一");
    }

    #[test]
    fn undo_blocked_mid_combat_and_empty() {
        let mut sc = scene();
        sc.undo.begin_combat();
        assert_eq!(undo_last(&mut sc), UndoOutcome::Blocked, "战斗中途不可撤（T1 契约）");
        sc.undo.finish_combat(CombatStroke {
            enemy_slot: 0,
            enemy_type_id: 41,
            enemy_hp_before: 10,
            vitals_before: PlayerVitals { hp: 100, gold: 5, exp: 0 },
            items_before: vec![],
            enemy_snapshot: Default::default(),
        });
        let out = undo_last(&mut sc);
        assert!(matches!(out, UndoOutcome::Undone(_)), "战斗终结可撤：{out:?}");
        assert_eq!(undo_last(&mut sc), UndoOutcome::Empty, "栈空=Empty");
    }

    #[test]
    fn undo_button_draws_and_zero_greys() {
        let mut buf = vec![0u32; 480 * 320];
        draw_undo_button(&mut buf, 480, 320, 480 - 112, 0, 3, 2);
        assert!(buf.iter().any(|&p| p == 0xFFFF_FFFF), "按钮文本像素在");
        assert!(buf.iter().any(|&p| p == 0xFF60_C0FF), "count>0 亮描边在");
        let mut grey = vec![0u32; 480 * 320];
        draw_undo_button(&mut grey, 480, 320, 480 - 112, 0, 0, 2);
        assert!(grey.iter().any(|&p| p == 0xFF60_6060), "count=0 灰描边");
        assert!(!grey.iter().any(|&p| p == 0xFF60_C0FF), "count=0 无亮边（不可回撤显灰）");
    }

    /// 【BUG-045】点地撞怪开战 → 胜 → 撤销：怪在原格复活（可见、在格槽里、再撞即开战），
    /// 全局怪物属性表不变，玩家血 / 金回到战前。maplv1 教程层 (2,10)(3,10) 是可走格（教程箭头格）。
    #[test]
    fn bug045_undo_after_tap_battle_revives_enemy_in_place() {
        use game_core::enums::GameMode;
        use game_core::undo_journal::Stroke;
        let mut sc = scene_with((2, 10), &[(41, 3, 10)]);
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 500;
        sc.player.gold = 7;
        // 哨兵：属性表里与怪物实体槽号同下标的那一项（夹具中怪在实体槽 0、型 41 也落表下标 0，原值恰等于复活血 35，
        // 按槽号误写的变异区分不了）——改成不可能由复活写出的值，任何按实体槽号的写入都会把它冲掉（复核 / 变异②）
        let slot = game_core::entity::cell_entity(&sc.grid, &sc.entities, 3, 10, 41);
        assert!(slot >= 0, "前置：怪物在 (3,10)");
        sc.enemies.hp[slot as usize] = 987_654;
        let hp_table_before = sc.enemies.hp.clone();
        let ent_before = sc.entities.items[slot as usize];
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(3, 10);
        let mut saw_battle = false;
        let mut fc = 0;
        while fc < 3000 {
            sc.tick(fc, &wt);
            fc += 1;
            saw_battle |= sc.walk_phase == 5;
            if saw_battle && sc.battle_target < 0 && sc.game_mode == GameMode::Map && sc.walk_phase == 0 && !sc.route.is_open() {
                break;
            }
        }
        assert!(saw_battle, "前置：开战");
        for _ in 0..60 {
            sc.tick(fc, &wt); // 死亡动画 / 摘槽走完
            fc += 1;
        }
        assert!(sc.player.gold > 7, "前置：胜利得金");
        let mut undone_combat = false;
        for _ in 0..6 {
            match undo_last(&mut sc) {
                UndoOutcome::Undone(Stroke::Combat(_)) => {
                    undone_combat = true;
                    break;
                }
                UndoOutcome::Undone(_) => continue,
                other => panic!("撤销栈提前结束：{other:?}"),
            }
        }
        assert!(undone_combat, "栈里有战斗笔");
        let e = game_core::entity::cell_entity(&sc.grid, &sc.entities, 3, 10, 41);
        assert!(e >= 0, "怪物回到 (3,10) 的格槽");
        let ent = &sc.entities.items[e as usize];
        assert!(!ent.removed && ent.visible && ent.state == 0, "怪物活着且可见：{:?}", (ent.removed, ent.visible, ent.state));
        // 整格还原（复核 T3-R5）：死亡动画改过的视觉字段须回到开战前（anim_frame 除外——空闲动画照常推进）
        assert_eq!(
            (ent.type_, ent.px, ent.py, ent.param, ent.sprite_w, ent.sprite_h, ent.anim_row),
            (ent_before.type_, ent_before.px, ent_before.py, ent_before.param, ent_before.sprite_w, ent_before.sprite_h, ent_before.anim_row),
            "复活实体的型 / 位置 / 参数 / 精灵 / 动画行 = 开战前"
        );
        assert_eq!(sc.enemies.hp, hp_table_before, "全局怪物属性表不变");
        assert_eq!((sc.player.hp, sc.player.gold), (500, 7), "玩家血 / 金回到战前");
        // 位移笔逆放已把玩家放回战前格（复核 T3-R8：不手工摆位，免得掩盖位移撤销放错格）
        assert_eq!((sc.player_cell_x, sc.player_cell_y, sc.view.player_px, sc.view.player_py), (2, 10, 2 << 5, 10 << 5), "玩家回到战前格");
        // 再撞同格必须开战
        sc.press_key(-4);
        let mut fought_again = false;
        for _ in 0..10 {
            sc.tick(fc, &wt);
            fc += 1;
            fought_again |= sc.walk_phase == 5;
        }
        sc.release_key();
        assert!(fought_again, "撤销后再撞同格必须开战");
    }

    /// 【BUG-045 · 复核 T3-R3】战斗笔里的实体槽已换成别的实体（实体表被重排）⇒ 撤销不盲写。
    #[test]
    fn bug045_revive_skips_slot_holding_another_entity() {
        use game_core::undo_journal::CombatStroke;
        let mut sc = scene_with((2, 10), &[(42, 6, 10)]);
        let live_before = sc.entities.items[0];
        assert_eq!(live_before.type_, 42, "前置：槽 0 是活着的型 42");
        let mut ghost = live_before;
        ghost.type_ = 41;
        ghost.px = 3 << 5;
        sc.undo.finish_combat(CombatStroke {
            enemy_slot: 0,
            enemy_type_id: 41,
            enemy_hp_before: 35,
            vitals_before: PlayerVitals { hp: 1, gold: 1, exp: 0 },
            items_before: vec![],
            enemy_snapshot: ghost,
        });
        let _ = undo_last(&mut sc);
        assert_eq!(sc.entities.items[0], live_before, "槽 0 的型 42 不被快照覆盖");
    }

    /// 【BUG-045】路段开放的战斗进行中不可撤（begin_combat 生产调用）。
    #[test]
    fn bug045_undo_is_blocked_mid_tap_battle() {
        let mut sc = scene_with((2, 10), &[(41, 3, 10)]);
        // 刚好大于怪 41 的防御：打得过但要很多回合（预测掉血 < hp）
        sc.player.atk = fixture_numbers("enemybasedef.txt")[0] + 1;
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(3, 10);
        let mut fc = 0;
        while sc.walk_phase != 5 && fc < 40 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!(sc.walk_phase, 5, "前置：战斗中");
        assert_eq!(undo_last(&mut sc), UndoOutcome::Blocked, "战斗中撤销被拒");
    }

    /// 【STAGE-B-008 · spec §7】撤销同时清目标标记、怪物选中、排队动作。
    #[test]
    fn undo_clears_tap_state() {
        let mut sc = scene();
        // 手动推位移笔（同 undo_move_restores_tile_and_count：钉 undo_last 执行路径，不依赖战斗时序）
        sc.undo.push_move(MoveStroke { from: (3, 10), to: (5, 10) });
        sc.tap_target = Some((3, 10));
        sc.tap_reject = Some(((4, 10), sc.frame_counter + 8));
        let out = undo_last(&mut sc);
        assert!(matches!(out, UndoOutcome::Undone(_)), "撤销成功：{out:?}");
        assert_eq!(sc.tap_target, None);
        assert_eq!(sc.tap_reject, None);
    }
}
