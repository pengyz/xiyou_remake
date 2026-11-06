//! P1-a1 dump 补面 6 组（DDDSA 前置件）——名集契约测试（A2 行序冻结对象=state_dump.rs push 序）。
//! 名=Java 字段名（javap rename-pipeline/renamed/a.class——statediff 名集相等约束）。

/// 【P1-a1 红测试】六组补面名必须在 dump_scene push 序在场（TDD 红先行：缺面=红）。
#[test]
fn p1a1_dump_face_contains_six_groups() {
    let src = include_str!("../src/state_dump.rs");
    let body = src.split("pub fn dump_scene").nth(1).expect("dump_scene 体在");
    for name in [
        "itemStackTypes",   // FLD202 道具栈类型（sc.player.item_types）
        "itemStackUses",    // FLD203 道具栈使用（sc.player.item_uses）
        "itemStackSize",    // FLD201 道具栈大小（sc.player.item_stack_size）
        "itemUseTargetType", // FLD058（Rust 真身=menu.overlay_kind_item——scene.rs:4330 注记）
        "tutorialSeen",     // FLD215（menu.tutorial_seen）
        "scriptLineFlags",  // FLD240（script_line_flags）
        "overlayText",      // FLD062（dialog.overlay_text）
    ] {
        assert!(
            body.contains(&format!("\"{name}\"")),
            "P1-a1 补面缺名：{name}（dump_scene push 序在场性——A2 冻结对象）"
        );
    }
}

/// 【P2 批0 红测试】写面归因登记 FLD 名↔state_dump push 面对照（TDD 红先行：登记挂名而
/// dump 无此键=红——FLD 映射失实防线；flds 空=dump 盲区注记不检）。
#[test]
fn p2b0_registry_flds_present_in_state_dump() {
    let src = include_str!("../src/state_dump.rs");
    let body = src.split("pub fn dump_scene").nth(1).expect("dump_scene 体在");
    for row in game_core::subsystem::WRITE_FACE_REGISTRY {
        for name in row.flds {
            assert!(
                body.contains(&format!("\"{name}\"")),
                "写面登记 FLD 失联：{}（face={} field={}）——state_dump push 面无此键",
                name,
                row.face,
                row.field
            );
        }
    }
}
