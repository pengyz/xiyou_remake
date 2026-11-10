//! 求解器直调门面（方案 B · 求解器批1；提案
//! `docs/superpowers/proposals/2026-11-09-game-core-solver-facade-proposal.md`
//! §1.1/§4）：3 个纯转发 fn，底层 `pub(super)` 零改动。
//!
//! 路径 `game_core::scene::solver_api::*`——必须挂 scene 树内（底层三 fn 均
//! `pub(super)`=`pub(in scene)`，挂 lib.rs 层编译不过）；`#[doc(hidden)]` 收口
//! （先例：lib.rs 对 `scene_test_support` 的 doc(hidden) 重导出）。
//!
//! # 模块契约（调用前置——违反即 panic，消费方必须保证）
//! - `parse_op` 三条前置，任一不满足即 panic：
//!   1. `script_cursor0 + 3 <= line.len()`；
//!   2. `script_cursor0` 落在 UTF-8 字符边界上——底层 `script_ops.rs:402`
//!      `&line[script_cursor0..script_cursor0 + 3]` 无守卫；`scene_script.rs:17`
//!      的起始门 `script_cursor0 + 3 > line.len()` **只拦字节越界、不拦字符
//!      边界**，非 ASCII 行可穿门后在切片处 panic；
//!   3. 该指令的**参数分隔符齐全**——`parse_script_int`（scene_script.rs:7-8）
//!      `line[from..].find(delim).expect("脚本整数分隔符缺失")`，分隔符缺失即
//!      panic（Java 同构：`a.java:8613 parseScriptInt` `indexOf`=-1 ⇒
//!      `substring(from,-1)` 同样抛异常）。
//!   真实语料（level_scripts.txt 68 槽）实测全 ASCII、指令 3 字节对齐（前置
//!   1/2 恒成立）；前置 3 由 **67/68 槽**满足，槽67 `TAK_264_268` 不满足——
//!   `from_fixtures`（scene.rs:253-254，`.trim()`）吃掉无 `\x1e` 末行的行尾
//!   空格，TAK 第二参分隔符随之消失。该槽经 DC-5 裁决**生产不可达**
//!   （`docs/spec/dead-code-rulings.md:170-200`），两侧（门面/底层）同因同崩，
//!   panic-parity 由 Commit2 G5b 语料行走的槽67 专测断言。
//! - `execute_op` / `execute_script_instruction`：入参须为已构造完成的
//!   `&mut GameScene`（load_floor 后），`script_line`/`script_cursor` 由调用方
//!   或上层装载。
//!
//! # R1 红线（提案 §5）
//! 本模块是求解器 **(a) 侧专用**直调面：(b) 对拍腿唯一合法路径是
//! `game_core::script::ScriptEngine`——禁止 (b) 侧引用 `scene::solver_api`，
//! 禁止以 GameScene 解释器路径（含 load_level_script+门面）充当 (b)，违者
//! 对拍判无效。
//!
//! # Y3 锚漂移
//! 本文件经 scene.rs 一行 `#[path]` 挂载挂入，挂载行使 scene.rs 全部行锚
//! **+1**（漂移量记于 Commit1 message；实施后全部 scene.rs 锚已重测）。

use super::{GameScene, ScriptOp};

/// 纯转发：`GameScene::parse_op`（script_ops.rs:401，`pub(super)` 零改动）。
/// 调用前置见模块契约 doc（越界/非字符边界 ⇒ 切片 panic）。
#[inline]
pub fn parse_op(line: &str, script_cursor0: usize) -> Option<ScriptOp> {
    GameScene::parse_op(line, script_cursor0)
}

/// 纯转发：`GameScene::execute_op`（script_ops.rs:553，`pub(super)` 零改动，
/// 内含 GIN 尾递归与 ScriptResolver 域写）。
#[inline]
pub fn execute_op(scene: &mut GameScene, op: ScriptOp) {
    scene.execute_op(op);
}

/// 纯转发：`GameScene::execute_script_instruction`（scene_script.rs:14）——
/// 行首起始门 + parse_op + execute_op 整步；行尾收尾（phase=4+锁相机）同源。
#[inline]
pub fn execute_script_instruction(scene: &mut GameScene) {
    scene.execute_script_instruction();
}


#[cfg(test)]
mod tests {
    use super::super::scene_test_support::{scene_setup, SceneSetup};
    use super::super::{GameScene, ScriptOp};
    use crate::enums::GameMode;

    /// fixture 语料槽数（scene_persist.rs:232 硬编码 vec![false;68] 同源）。
    const SLOTS: usize = 68;

    /// 槽67 索引（DC-5 不可达槽，见 slot67_* 专测）。
    const SLOT67: usize = SLOTS - 1;

    /// 合成场景：走廊 (5,3)，场上 type4@（6,3）+ type84@（7,3）两实体
    /// （DES/CES/MOV 目标面可命中）；dialog 相位复位、game_mode=Dialog
    /// （entity_route_tests::exec 同款驱动前置）。
    fn new_scene() -> GameScene {
        scene_setup(
            (5, 3),
            &[(4, 6, 3), (84, 7, 3)],
            SceneSetup {
                game_mode: Some(GameMode::Dialog),
                dialog_dialog_phase: Some(0),
                ..SceneSetup::default()
            },
        )
    }

    /// G5a 签名锁（提案 §3 G5a）：三转发 fn 签名逐字钉死。
    /// 改签名 ⇒ 本测试编译红（mismatched types）。
    #[test]
    fn g5a_signature_lock_three_forwards() {
        let _: fn(&str, usize) -> Option<ScriptOp> = super::parse_op;
        let _: fn(&mut GameScene, ScriptOp) = super::execute_op;
        let _: fn(&mut GameScene) = super::execute_script_instruction;
    }

    /// G5b 行为锁（解析面，fn#1 `parse_op`）：语料 **67 槽（0..=66）** 逐行
    /// 游标行走，门面 vs 底层 `GameScene::parse_op`（scene 树内 pub(super)
    /// 直调）逐变体、逐游标相等。变体用 `mem::discriminant`（ScriptOp 无
    /// PartialEq/Debug），游标用 `ScriptOp::cursor()`（scene.rs pub 游标契约 fn）。
    /// 槽67（`TAK_264_268`，DC-5 不可达）不入行走——`from_fixtures` trim 吃掉
    /// 无 `\x1e` 末行的行尾空格 ⇒ TAK 第二参分隔符缺失 ⇒ 两侧同因同崩，
    /// panic-parity 见 `slot67_dc5_unreachable_parse_panic_parity`。
    #[test]
    fn g5b_parse_parity_fixture_all_lines() {
        let scripts = new_scene().dialog.dialog_data.scripts;
        assert_eq!(scripts.len(), SLOTS, "fixture 应为 68 槽");
        for (idx, line) in scripts.iter().enumerate().take(SLOT67) {
            let mut cur = 0usize;
            let mut steps = 0usize;
            loop {
                if cur + 3 > line.len() {
                    break; // 起始门同款：字节越界即停（parse_op 无守卫，防 panic）
                }
                let via_facade = super::parse_op(line, cur);
                let via_direct = GameScene::parse_op(line, cur);
                assert_eq!(
                    std::mem::discriminant(&via_facade),
                    std::mem::discriminant(&via_direct),
                    "槽{idx} cursor={cur}：门面 vs 底层 parse 变体不等"
                );
                let next_facade = via_facade.map(|op| op.cursor());
                let next_direct = via_direct.map(|op| op.cursor());
                assert_eq!(
                    next_facade, next_direct,
                    "槽{idx} cursor={cur}：门面 vs 底层 parse 游标不等"
                );
                match next_facade {
                    Some(n) if n > cur => cur = n,
                    _ => break,
                }
                steps += 1;
                assert!(steps < 10_000, "槽{idx} parse 游标未推进（死循环）");
            }
        }
    }

    /// (a) 侧驱动：单槽起手，循环到游标停摆（末次调用触发行尾收尾）。
    fn drive_facade(sc: &mut GameScene, idx: usize) {
        sc.script_line = sc.dialog.dialog_data.scripts[idx].clone();
        sc.script_cursor = 0;
        sc.current_script_index = idx;
        let mut steps = 0usize;
        loop {
            let c0 = sc.script_cursor;
            super::execute_script_instruction(sc);
            steps += 1;
            assert!(steps < 100_000, "槽{idx} 门面驱动未收敛");
            if sc.script_cursor == c0 {
                break;
            }
        }
    }

    /// 底层直调驱动（同款循环，scene 树内 pub(super) 面）。
    fn drive_direct(sc: &mut GameScene, idx: usize) {
        sc.script_line = sc.dialog.dialog_data.scripts[idx].clone();
        sc.script_cursor = 0;
        sc.current_script_index = idx;
        let mut steps = 0usize;
        loop {
            let c0 = sc.script_cursor;
            sc.execute_script_instruction();
            steps += 1;
            assert!(steps < 100_000, "槽{idx} 底层驱动未收敛");
            if sc.script_cursor == c0 {
                break;
            }
        }
    }

    /// 逐字段快照（GameScene 无 derive——手挑 pub 面；脚本解释器全写面 +
    /// 实体表 + 网格 + 菜单/持久/浮层域）。渲染/相机/寻路缓冲等非脚本写面
    /// 不入快照（见 dump 尾注释）。
    fn dump(sc: &GameScene) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        // —— 视口/相机（ROS_1 写 px/py）——
        v.push(format!(
            "view=({},{},{},{},{},{},{},{},{},{})",
            sc.view.wide,
            sc.view.high,
            sc.view.view_w,
            sc.view.view_h,
            sc.view.clip_w,
            sc.view.clip_h,
            sc.view.cam_x,
            sc.view.cam_y,
            sc.view.player_px,
            sc.view.player_py
        ));
        v.push(format!("view.terrain={:?}", sc.view.terrain));
        v.push(format!("view.transform={:?}", sc.view.transform));
        // —— 实体表（DES/CES/ADD/MOV/SWD 写面）——
        v.push(format!(
            "entities.count={} stair={:?},{:?}",
            sc.entities.count, sc.entities.stair_up, sc.entities.stair_down
        ));
        for i in 0..sc.entities.count {
            let e = &sc.entities.items[i];
            v.push(format!(
                "entity[{i}]=({},{},{},{},{},{},{},{},{},{},{})",
                e.type_,
                e.param,
                e.px,
                e.py,
                e.state,
                e.visible,
                e.removed,
                e.sprite_w,
                e.sprite_h,
                e.anim_row,
                e.anim_frame
            ));
        }
        v.push(format!(
            "grid=({},{},{:?},{:?},{:?})",
            sc.grid.cols, sc.grid.rows, sc.grid.cell_type, sc.grid.capacity, sc.grid.slots
        ));
        v.push(format!("walkable={:?}", sc.walkable));
        v.push(format!(
            "minimap={}/{:?}",
            sc.minimap.is_some(),
            sc.minimap_enabled
        ));
        // —— 步进/玩家面 ——
        v.push(format!(
            "frame=({}, {}, {}, {}, {}, {})",
            sc.frame_counter,
            sc.view_bottom,
            sc.facing,
            sc.walk_phase,
            sc.player_frame,
            sc.player_bob_applied
        ));
        v.push(format!(
            "player=({},{},{},{},{},{},{},{},{:?},{:?})",
            sc.player.hp,
            sc.player.atk,
            sc.player.def,
            sc.player.yellow_keys,
            sc.player.blue_keys,
            sc.player.red_keys,
            sc.player.gold,
            sc.player.item_stack_size,
            sc.player.item_types,
            sc.player.item_uses
        ));
        v.push(format!(
            "enemies=({},{})",
            sc.enemies.len(),
            sc.enemy_base.len()
        ));
        v.push(format!(
            "alchemy=({},{},{},{},{},{})",
            sc.alchemy_price,
            sc.alchemy_gain_hp,
            sc.alchemy_gain_atk,
            sc.alchemy_gain_def,
            sc.alchemy_cursor,
            sc.alchemy_upgrade_count
        ));
        v.push(format!(
            "merchant=({}, {:?}, {:?}, {}, {}, {})",
            sc.merchant_entity,
            sc.offer_queue_src,
            sc.offer_queue_idx,
            sc.offer_queue_cursor,
            sc.key_value,
            sc.key_held
        ));
        v.push(format!(
            "step=({},{},{},{})",
            sc.player_cell_x, sc.player_cell_y, sc.step_progress, sc.walk_step_count
        ));
        v.push(format!(
            "battle=({},{},{},{},{},{},{},{})",
            sc.battle_target,
            sc.battle_enemy_hp,
            sc.battle_tick,
            sc.battle_anim_tick,
            sc.battle_finishing,
            sc.battle_rerun_scene,
            sc.battle_no_counter,
            sc.battle_active
        ));
        // —— 换层/路线（LAY/MOV 写面）——
        v.push(format!(
            "floor=({},{},{},{})",
            sc.transitioning, sc.wipe_closing, sc.wipe, sc.route_entity
        ));
        v.push(format!(
            "route=({},{},{},{},{:?})",
            sc.route_dir, sc.route_phase, sc.route_progress, sc.route_entity, sc.route_start
        ));
        v.push(format!(
            "equip=({},{},{:?},{:?},{:?})",
            sc.equipped_weapon,
            sc.equipped_armor,
            sc.equip_bonuses,
            sc.equip_types,
            sc.open_group
        ));
        // —— 模式/脚本域 ——
        v.push(format!(
            "mode=({:?},{},{:?},{},{})",
            sc.game_mode, sc.transition_count, sc.last_transition, sc.ending_scroll, sc.ending_anim
        ));
        v.push(format!(
            "script=({}, {:?}, {:?}, {}, {}, {})",
            sc.script_line,
            sc.script_cursor,
            sc.script_line_flags,
            sc.script_delay,
            sc.current_script_index,
            sc.script_walk_armed
        ));
        // —— dialog 域（TAK/SEE/open_dialog_page 写面）——
        v.push(format!(
            "dialog=({}, {}, {}, {}, {}, {}, {:?})",
            sc.dialog.dialog_phase,
            sc.dialog.dialog_page,
            sc.dialog.dialog_page_end,
            sc.dialog.dialog_speaker,
            sc.dialog.dialog_anchor,
            sc.dialog.dialog_char,
            sc.dialog.dialog_target
        ));
        v.push(format!(
            "overlay=({}, {}, {:?}, {}, {}, {}, {}, {:?}, {}, {}, {}, {}, {}, {})",
            sc.dialog.overlay_active,
            sc.dialog.overlay_kind,
            sc.dialog.overlay_text,
            sc.dialog.overlay_box_w,
            sc.dialog.overlay_box_h,
            sc.dialog.overlay_left_icon,
            sc.dialog.overlay_right_icon,
            sc.dialog.overlay_lines,
            sc.dialog.overlay_page_top,
            sc.dialog.overlay_page_bottom,
            sc.dialog.overlay_lines_per_page,
            sc.dialog.overlay_total_lines,
            sc.dialog.overlay_content_w,
            sc.dialog.overlay_saved_color
        ));
        // —— menu 域（RES/装备/拾取浮层写面）——
        v.push(format!(
            "menu=({}, {}, {}, {:?}, {}, {}, {:?}, {}, {}, {}, {}, {}, {}, {}, {:?})",
            sc.menu.difficulty_index,
            sc.menu.tutorial_seen,
            sc.menu.stair_hint,
            sc.menu.menu_return_mode,
            sc.menu.menu_cursor,
            sc.menu.menu_item_count,
            sc.menu.menu_kinds,
            sc.menu.menu_box_width,
            sc.menu.menu_line_height,
            sc.menu.menu_visible_rows,
            sc.menu.menu_list_height,
            sc.menu.menu_scroll_top,
            sc.menu.menu_visible_bottom,
            sc.menu.overlay_kind_item,
            sc.menu.item_cursor
        ));
        // —— persist 域（LAY save_floor_state / RES 写面）——
        v.push(format!(
            "persist=({}, {}, {}, {}, {}, {}, {:?}, {:?}, {:?}, {})",
            sc.persist.floor,
            sc.persist.current_floor,
            sc.persist.min_floor,
            sc.persist.max_floor,
            sc.persist.pending_floor,
            sc.persist.went_down,
            sc.persist.journal,
            sc.persist.saved_floor_states,
            sc.persist.preset_mot_if,
            sc.persist.global_dirty
        ));
        v.push(format!(
            "misc=({}, {}, {:?}, {:?}, {:?}, {}, {:?}, {:?})",
            sc.rainbow_phase,
            sc.path_arrows_visible,
            sc.walk_path_buffer,
            sc.afterimage_x,
            sc.afterimage_y,
            sc.view_locked,
            sc.width_table,
            sc.tile_walk
        ));
        v
        // 不入快照（非脚本写面/构造期恒等/无 pub 读面）：images、layout、bgm、
        // render、route(RouteTracker)、undo、tap、popups(私有槽)、rng、
        // object_type_names、minimap 像素本体、walkto_driving(私有)。
    }

    /// G5b 行为锁（行驱动面，fn#3 `execute_script_instruction`）：语料 67 槽
    /// （0..=66）逐槽双跑——门面 vs 底层 pub(super) 直调，终态逐字段相等。
    /// 改接同签名他 fn ⇒ 状态分叉 ⇒ 红。槽67 见 `slot67_dc5_unreachable_*`。
    #[test]
    fn g5b_execute_parity_fixture_all_lines() {
        let scripts = new_scene().dialog.dialog_data.scripts;
        assert_eq!(scripts.len(), SLOTS, "fixture 应为 68 槽");
        for idx in 0..SLOT67 {
            let mut via_facade = new_scene();
            let mut via_direct = new_scene();
            drive_facade(&mut via_facade, idx);
            drive_direct(&mut via_direct, idx);
            assert_eq!(
                dump(&via_facade),
                dump(&via_direct),
                "槽{idx}：门面驱动 vs 底层直调 终态逐字段不等"
            );
        }
    }

    /// 提取 panic 消息（`Option::expect` 的 String payload 与 &str 皆兼容）。
    fn panic_msg(r: Result<(), Box<dyn std::any::Any + Send>>) -> String {
        match r {
            Ok(()) => String::new(),
            Err(e) => e
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "<非字符串 panic payload>".into()),
        }
    }

    /// G5b 行为锁（fn#2 `execute_op` 直跑面）：语料每条指令 parse 后分别经
    /// 门面 `execute_op` / 底层 `execute_op` 落到两个同构场景，终态逐字段相等。
    /// 行驱动面（`g5b_execute_parity_*`）走的是 fn#3 行驱动、内部用底层
    /// execute_op，故 fn#2 必须单列——否则「改接同签名空实现」测不出（实测）。
    #[test]
    fn g5b_execute_op_parity_direct_per_instruction() {
        let scripts = new_scene().dialog.dialog_data.scripts;
        assert_eq!(scripts.len(), SLOTS, "fixture 应为 68 槽");
        for (idx, line) in scripts.iter().enumerate().take(SLOT67) {
            let mut cur = 0usize;
            let mut steps = 0usize;
            while cur + 3 <= line.len() {
                let via_facade = super::parse_op(line, cur);
                let via_direct = GameScene::parse_op(line, cur);
                assert_eq!(
                    std::mem::discriminant(&via_facade),
                    std::mem::discriminant(&via_direct),
                    "槽{idx} cursor={cur}：parse 变体不等"
                );
                let (Some(op_f), Some(op_d)) = (via_facade, via_direct) else {
                    break; // 未知 3 字母 op：两面同停摆（script_ops.rs 兜底臂）
                };
                let next = op_f.cursor();
                let mut sc_f = new_scene();
                let mut sc_d = new_scene();
                for sc in [&mut sc_f, &mut sc_d] {
                    sc.script_line = line.clone();
                    sc.script_cursor = cur;
                    sc.current_script_index = idx;
                }
                super::execute_op(&mut sc_f, op_f);
                sc_d.execute_op(op_d);
                assert_eq!(
                    dump(&sc_f),
                    dump(&sc_d),
                    "槽{idx} cursor={cur}：门面 execute_op vs 底层 execute_op 终态逐字段不等"
                );
                if next <= cur {
                    break;
                }
                cur = next;
                steps += 1;
                assert!(steps < 10_000, "槽{idx} execute_op 行走未推进（死循环）");
            }
        }
    }

    /// 槽67 专测（DC-5 不可达）：`from_fixtures` trim 吃掉无 `\x1e` 末行的
    /// 行尾空格 ⇒ `TAK_264_268` 第二参分隔符缺失 ⇒ `parse_script_int`
    /// expect panic。生产不可达（dead-code-rulings.md:170-200 终裁
    /// `loadLevelScript(67)` 门控恒 false），此处断言**门面 vs 底层同因同崩**
    /// （panic-parity）：消息逐字相等 + 均为分隔符缺失。
    #[test]
    fn slot67_dc5_unreachable_parse_panic_parity() {
        let scripts = new_scene().dialog.dialog_data.scripts;
        assert_eq!(scripts.len(), SLOTS, "fixture 应为 68 槽");
        assert_eq!(
            scripts[SLOT67], "TAK_264_268",
            "from_fixtures trim 产物（原始末行为 TAK_264_268␠，a.java:1192 带尾空格）"
        );
        let via_facade = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = super::parse_op(&scripts[SLOT67], 0);
        }));
        let via_direct = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = GameScene::parse_op(&scripts[SLOT67], 0);
        }));
        let mf = panic_msg(via_facade);
        let md = panic_msg(via_direct);
        assert!(!mf.is_empty(), "门面侧应在缺分隔符处 panic");
        assert_eq!(mf, md, "槽67：门面 vs 底层 panic 消息不等（非同因）");
        assert!(mf.contains("脚本整数分隔符缺失"), "实际 panic: {mf}");
    }

    /// 槽67 专测（DC-5 不可达）：行驱动双跑 panic-parity（fn#3 面）。
    #[test]
    fn slot67_dc5_unreachable_execute_panic_parity() {
        let mut via_facade = new_scene();
        let mut via_direct = new_scene();
        let r_f = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            drive_facade(&mut via_facade, SLOT67);
        }));
        let r_d = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            drive_direct(&mut via_direct, SLOT67);
        }));
        let mf = panic_msg(r_f);
        let md = panic_msg(r_d);
        assert!(!mf.is_empty(), "门面侧应在缺分隔符处 panic");
        assert_eq!(mf, md, "槽67：门面 vs 底层 panic 消息不等（非同因）");
        assert!(mf.contains("脚本整数分隔符缺失"), "实际 panic: {mf}");
    }

    /// 冒烟：守卫前置①——多字节 UTF-8 行 + 非字符边界游标 ⇒ parse_op 切片
    /// panic（scene_script.rs:17 起始门只拦字节越界，拦不住 char boundary）。
    #[test]
    #[should_panic]
    fn smoke_parse_multibyte_non_boundary_panics() {
        let _ = super::parse_op("ééé", 0); // 0..3 跨字符边界 ⇒ byte index not a char boundary
    }

    /// 冒烟：守卫前置②——游标字节越界 ⇒ parse_op 切片 panic。
    #[test]
    #[should_panic]
    fn smoke_parse_out_of_bounds_panics() {
        let _ = super::parse_op("ABC", 1); // 1+3=4 > len 3 ⇒ byte index out of bounds
    }

    /// 冒烟：短行收尾——起始门（scene_script.rs:17）拦字节越界，不 panic：
    /// phase=4（AwaitCamera）+ flags[current_script_index]=true + 相机回玩家。
    #[test]
    fn smoke_execute_short_line_finishes() {
        let mut sc = new_scene();
        sc.script_line = "AB".to_string();
        sc.script_cursor = 0;
        sc.current_script_index = 0;
        super::execute_script_instruction(&mut sc);
        assert_eq!(sc.dialog.dialog_phase, 4, "行尾收尾 phase=4");
        assert!(sc.script_line_flags[0], "收尾置 flags[idx]");
    }

    /// 冒烟：fixture 语料 parse→execute→回读（提案 §4 Commit2）。
    /// 槽0 链含 RES_0（easy 重置）+ GUT_1（跳槽1跑完收尾）——回读金/血/攻/防/
    /// 三钥/相位/旗标。
    #[test]
    fn smoke_fixture_slot0_execute_readback() {
        let mut sc = new_scene();
        drive_facade(&mut sc, 0);
        assert_eq!(sc.dialog.dialog_phase, 4, "链尾收尾 phase=4");
        assert!(sc.script_line_flags[0], "槽0 GUT 置旗标");
        assert_eq!(sc.current_script_index, 1, "GUT_1 跳槽1");
        assert_eq!(sc.player.gold, 4, "RES_0 easy 重置 gold=4");
        assert_eq!(sc.player.hp, 400, "RES_0 easy 重置 hp=400");
        assert_eq!(sc.player.atk, 10, "RES_0 easy 重置 atk=10");
        assert_eq!(sc.player.def, 10, "RES_0 easy 重置 def=10");
        assert_eq!(
            (
                sc.player.yellow_keys,
                sc.player.blue_keys,
                sc.player.red_keys
            ),
            (0, 0, 0),
            "RES_0 easy 重置三钥=0"
        );
    }

    /// 冒烟：GIN 行回读（槽26 = TAK+DES+GIN_1_1000，单行链无 GUT/RES）——
    /// gain_gold 经门面执行后 gold=1000（初始 0）。
    #[test]
    fn smoke_fixture_gin_line_gold_readback() {
        let mut sc = new_scene();
        drive_facade(&mut sc, 26);
        assert_eq!(sc.player.gold, 1000, "GIN_1_1000 经门面加金");
        assert_eq!(sc.dialog.dialog_phase, 4, "行尾收尾 phase=4");
    }
}
