//! 【2026-11-18】f41 同格双门链回归：门 13 → 清掉两处 ty60 火 → 门 25 → 拾下行符(22)。
//!
//! 判据来源（A 级，逐处可复现）：
//! - `sprite41` 原始记录序：`#49 ty60@(2,2)`（静态火）、`#50 ty83 param=13@(10,2)`、
//!   `#51 ty83 param=25@(10,2)`（同格双门，13 在前）。
//! - 门 13 属「无门控直通簇」（`scene.rs` try_run_scene 注释列 5/6/10/13/14/…）⇒ 首访必过；
//!   跑过之后 `scene_script.rs` 行尾置 `script_line_flags[13] = true`，此后 13 关闭。
//! - 门 25（`scene.rs` try_run_scene 的 `25 =>` 臂）：
//!   `from_walk && flags[13] && entity_at_cell(2,2,60) < 0 && entity_at_cell(10,2,60) < 0`。
//! - 脚本 13 = `CES_6_10_2 CES_60_10_2`（在 (10,2) 召出一只 ty60）；脚本 25 = `CES_22_6_6`。
//!
//! ⇒ 设计意图是**链式**而非竞争：先跑 13（同时召出第二处火）→ 清掉两处火 → 再踩才开 25。
//! 本测试把这条链逐环节钉住，防止「同格双门谁先命中」再次失据（该行为此前零覆盖）。

use super::scene_test_support::{images, numbers};
use crate::enums::DialogPhase;
use crate::scene::GameScene;

fn repo() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn raw(name: &str) -> Vec<u8> {
    std::fs::read(repo().join("assets/raw").join(name)).unwrap_or_else(|e| panic!("缺资产 {name}：{e}"))
}

/// 用真实 maplv41 + sprite41 装载 f41（`load_floor` 与探针/对拍测试同构）。
fn load_f41() -> (GameScene, Vec<i32>) {
    let maplv = raw("maplv41");
    let sp = game_data::SpriteTable::parse(&raw("sprite41")).unwrap();
    let doors = crate::sprite_spawn::DoorTables {
        door12: numbers(include_str!("../../tests/fixtures/door12.txt")),
        door13: numbers(include_str!("../../tests/fixtures/door13.txt")),
    };
    let recs: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|r| {
            let (param, hide) = crate::sprite_spawn::spawn_param_and_visibility(r.type_code, &r.extra, &doors);
            (r.type_code, r.x, r.y, param, !hide)
        })
        .collect();
    let width_table = numbers(include_str!("../../tests/fixtures/type_width_px.txt"));
    let walk = numbers(include_str!("../../tests/fixtures/tile_walkability.txt"));
    let mut sc =
        GameScene::load_floor(&maplv, 0, 0, &recs, &width_table, &walk, images(), vec![None; 56]).unwrap();
    sc.persist.floor = 41;
    sc.persist.current_floor = 41;
    sc.persist.min_floor = 41;
    (sc, width_table)
}

/// 把玩家放到 (x,y) 并跑一次「踩格」副作用（= 行走落到该格的同一条路径，`from_walk=true`）。
fn stand(sc: &mut GameScene, x: i32, y: i32) {
    sc.view.player_px = x << 5;
    sc.view.player_py = y << 5;
    sc.apply_step_cell_effects();
}

/// 脚本相位为 ScriptStep 时逐拍推进，直到脚本跑完（相位离开 ScriptStep）。
fn run_script_to_end(sc: &mut GameScene, width_table: &[i32]) {
    for i in 0..200i64 {
        if DialogPhase::from_raw(sc.dialog.dialog_phase) != DialogPhase::ScriptStep {
            return;
        }
        sc.tick(i, width_table);
    }
    panic!("脚本 200 拍仍未跑完（相位 {:?}）", sc.dialog.dialog_phase);
}

fn cell_ty(sc: &GameScene, x: i32, y: i32, ty: i32) -> i32 {
    sc.entity_at_cell(x, y, ty)
}

/// ⚠ 前提警示：本测试把玩家**直接放到 (10,2)**（`stand` 只跑踩格副作用，不校验可走性）。
/// 而 f41 的 (10,2) 地形不可走（见 `f41_gate_cell_not_walkable_open_question`）⇒
/// 本测试钉的是**门控逻辑**，不是「玩家实际能走到那里」。可达性缺口单列测试，不得当作已通。
#[test]
fn f41_gate13_then_clear_two_fires_then_gate25_drops_down_arrow() {
    let (mut sc, width_table) = load_f41();

    // ---- 初始态：两处火只有 (2,2) 那一个，(10,2) 尚未召出；门 13 未跑
    assert!(cell_ty(&sc, 2, 2, 60) >= 0, "f41 (2,2) 应有静态 ty60 火");
    assert!(cell_ty(&sc, 10, 2, 60) < 0, "f41 (10,2) 初始无 ty60（由脚本 13 召出）");
    assert!(!sc.slots.script_line_flags[13], "初始门 13 未跑");
    assert!(cell_ty(&sc, 6, 6, 22) < 0, "初始无下行符");

    // ---- 第一段：首次踩 (10,2) ⇒ 门 13 命中（直通），并在 (10,2) 召出第二处火
    stand(&mut sc, 10, 2);
    run_script_to_end(&mut sc, &width_table);
    assert!(sc.slots.script_line_flags[13], "首访门 13 必过，跑完置 flags[13]");
    assert!(
        cell_ty(&sc, 10, 2, 60) >= 0,
        "脚本 13 = CES_6_10_2 CES_60_10_2 ⇒ 在 (10,2) 召出一只 ty60"
    );
    assert!(cell_ty(&sc, 6, 6, 22) < 0, "此时门 25 未开，不应有下行符");

    // ---- 第二段：清掉两处火（(2,2) 静态 + (10,2) 召出）。
    // 这里直接走引擎的「脱槽」原语，把「玩家把火推走/熄灭」这件事隔离成门控前提，
    // 不在此测试里复刻火的上爬动画（另有 STAGE-B-008 陷阱对拍覆盖）。
    for (cx, cy) in [(2i32, 2i32), (10, 2)] {
        let idx = cell_ty(&sc, cx, cy, 60);
        assert!(idx >= 0, "({cx},{cy}) 此时应有 ty60 可清");
        crate::entity::detach_from_cell(&mut sc.grid, &mut sc.entities, idx as usize);
    }
    assert!(cell_ty(&sc, 2, 2, 60) < 0 && cell_ty(&sc, 10, 2, 60) < 0, "两处火均已清");

    // ---- 第三段：再踩 (10,2) ⇒ 门 13 已被 flags[13] 关掉，门 25 命中，生成下行符
    stand(&mut sc, 10, 2);
    run_script_to_end(&mut sc, &width_table);
    assert!(
        sc.slots.script_line_flags[25],
        "两处火清空 + flags[13] ⇒ 门 25 命中（三重条件全满足）"
    );
    assert!(
        cell_ty(&sc, 6, 6, 22) >= 0,
        "脚本 25 = CES_22_6_6 ⇒ 在 (6,6) 生成下行符实体"
    );

    // ---- 第四段：踩 (6,6) 拾取 ⇒ 下行符入包（玩家能真正拿到）
    stand(&mut sc, 6, 6);
    run_script_to_end(&mut sc, &width_table);
    let n = sc.player.item_stack_size as usize;
    assert!(
        sc.player.item_types[..n].contains(&22),
        "踩 (6,6) 后下行符(22) 入背包（实得 {n} 件）"
    );
}

/// 反证：**不清火**时门 25 永远打不开——这条钉住「链式」而非「谁先命中」的竞争解释。
#[test]
fn f41_gate25_stays_closed_while_fires_remain() {
    let (mut sc, width_table) = load_f41();

    // 连踩两次 (10,2) 但一处火都不清：第二次只可能命中原生门控，25 必须仍关闭
    stand(&mut sc, 10, 2);
    run_script_to_end(&mut sc, &width_table);
    assert!(sc.slots.script_line_flags[13], "首次踩仍过门 13");

    stand(&mut sc, 10, 2);
    run_script_to_end(&mut sc, &width_table);
    assert!(
        !sc.slots.script_line_flags[25],
        "两处火都在 ⇒ 门 25 的 entity_at_cell 条件不满足，不得开启"
    );
    assert!(cell_ty(&sc, 6, 6, 22) < 0, "门 25 未开 ⇒ 不生成下行符");
}

/// 【未决缺口】f41 (10,2) 是两扇场景门的所在格，但地形不可走（cell_type 51）；
/// 而门 25 的门控要求 `from_walk`（必须走进该格）⇒ 「玩家能否真的站到 (10,2)」
/// 是这条链成立的前提，当前**未证实**。本测试把这个矛盾显式钉在这里，
/// 防止上面的链式测试被读成「f41 下行符已确认可得」。
#[test]
fn f41_gate_cell_not_walkable_open_question() {
    let (sc, _wt) = load_f41();
    assert!(!sc.walkable_at(10, 2), "f41 (10,2) 地形不可走（cell_type 51）——链式测试的站位是非法的");
    assert!(sc.walkable_at(2, 2), "f41 (2,2) 地形可走（火陷阱所在格）");
    // 火的上爬前提：上格必须可走且无槽。两处火的上格都是墙 ⇒ fire_trap_climb_up 不会发生。
    assert!(!sc.walkable_at(2, 1), "(2,2) 火的上格 (2,1) 不可走 ⇒ 火爬不走");
    assert!(!sc.walkable_at(10, 1), "(10,2) 火的上格 (10,1) 不可走 ⇒ 火爬不走");
}
