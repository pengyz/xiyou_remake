//! T3 撤销按钮执行面 + 撤销 UI（touch-native-undo-plan §0 边界 3）。
//!
//! game-core **零改动**红线下的接缝处置（裁定，ledger 见 commit）：T1
//! `UndoTarget` trait 文档写明"T2/T3 在 Scene 上实现"，但 core 零改红线不许落
//! impl ⇒ 落**壳侧 newtype 适配器**（trait 外部、类型本地，orphan 规则允许），
//! 经 scene pub 状态面逆放。语义锚：exp 恒 0（本作无经验字段，scene.rs:5238）、
//! 敌复活=state 回 0 + type 还原（battle_undo_snap 字段面 scene.rs:5231）。
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
        if let Some(ent) = sc.entities.items.get_mut(e.slot) {
            ent.type_ = e.type_id;
            ent.state = 0; // 常态（markEntityRemoved 逆）
            ent.visible = true;
        }
        if let Some(hp) = sc.enemies.hp.get_mut(e.slot) {
            *hp = e.hp;
        }
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

    /// 轻量 scene（scene_test_support 同构：合成图像 + 仓库夹具；零 core 改动）。
    fn scene() -> GameScene {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let fix = repo.join("crates/game-core/tests/fixtures");
        let width_table: Vec<i32> = std::fs::read_to_string(fix.join("type_width_px.txt"))
            .unwrap()
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect();
        let walk: Vec<i32> = std::fs::read_to_string(fix.join("tile_walkability.txt"))
            .unwrap()
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect();
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
        let floors: Vec<Option<FloorAssets>> = (0..56).map(|_| None).collect();
        GameScene::load_floor(&maplv, 3 << 5, 10 << 5, &[], &width_table, &walk, images, floors)
            .unwrap()
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
}
