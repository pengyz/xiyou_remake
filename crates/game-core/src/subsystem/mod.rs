//! DDDSA P2 批0 地基：subsystem 模块族（8 trait 面+Ctx 族+写面归因登记）。
//!
//! spec v3 §3.1（docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md:96-115）八 trait 面；
//! §3.2 Ctx 宪法三宪：①纯数据（无方法）②字段冻结 ③签名 ctx+≤2 参数。实现见各 face 文件。
//!
//! # 字段冻结 schema 变更流程
//! Ctx 纯数据结构的字段集 = **冻结 schema**。任何增/删/改名/改型必须：
//! 1. 同步 `WRITE_FACE_REGISTRY`（写面归因登记）——增删行随 Ctx 字段**同笔**；
//! 2. 同步 `data/status/field-ownership.json`（唯一归属；`python3 tools/field_ownership.py --check` 必绿）；
//! 3. 涉 dump 可见面时同步 state_dump FLD 对照（A2 冻结对象=键名+值序+字节输出——判定零重冻或显式重冻）；
//! 4. 单 commit 完成上述全部同步面（禁双轨/禁拆散）。
//! 禁：绕过登记直接加字段（DC-9 换马甲）；用读槽承载写面（读槽=&/值拷贝，写槽=&mut——写面归因只认写槽）。
//!
//! # 写面归因登记（field-ownership 对照 state_dump FLD）
//! `WRITE_FACE_REGISTRY` 每行=一个写槽：face（Ctx 结构名）+ field（=归属表键）+ domain（唯一归属域）+
//! flds（state_dump.rs push FLD 名对照；空=无 FLD——dump 盲区，plan P2 盲区清单面）。
//! ItemCtx=P1-2 effect_registry.rs:74 已存形态照录（不改史）；entityBaseGold 等派生键不入（输出面非写面）。

pub mod battle;
pub mod dialog;
pub mod entity_service;
pub mod input_mapper;
pub mod item_service;
pub mod persist_codec;
pub mod script_host;
pub mod ui_render;

/// 写面归因登记行（冻结 schema 变更流程第 1 步的同步对象）。
pub struct WriteFaceRow {
    /// 写面 Ctx 结构名。
    pub face: &'static str,
    /// 写槽字段名（=data/status/field-ownership.json 表键）。
    pub field: &'static str,
    /// field-ownership.json 唯一归属域。
    pub domain: &'static str,
    /// state_dump.rs push FLD 名对照（空=dump 盲区）。
    pub flds: &'static [&'static str],
}

/// 写面归因登记（43 行=Battle 10+Dialog 22+Entity 1+Item 4+Script 6——行数随 Ctx 写槽增减同笔）。
pub const WRITE_FACE_REGISTRY: &[WriteFaceRow] = &[
    // —— BattleCtx（combat 域 10 写槽）——
    WriteFaceRow { face: "BattleCtx", field: "battle_active", domain: "combat", flds: &["battleActiveFlag"] },
    WriteFaceRow { face: "BattleCtx", field: "battle_anim_tick", domain: "combat", flds: &[] },
    WriteFaceRow { face: "BattleCtx", field: "battle_digit_by_type", domain: "combat", flds: &[] },
    WriteFaceRow { face: "BattleCtx", field: "battle_enemy_hp", domain: "combat", flds: &[] },
    WriteFaceRow { face: "BattleCtx", field: "battle_finishing", domain: "combat", flds: &[] },
    WriteFaceRow { face: "BattleCtx", field: "battle_no_counter", domain: "combat", flds: &[] },
    WriteFaceRow { face: "BattleCtx", field: "battle_rerun_scene", domain: "combat", flds: &[] },
    WriteFaceRow { face: "BattleCtx", field: "battle_target", domain: "combat", flds: &[] },
    WriteFaceRow { face: "BattleCtx", field: "battle_tick", domain: "combat", flds: &[] },
    WriteFaceRow { face: "BattleCtx", field: "show_battle_digits", domain: "combat", flds: &[] },
    // —— DialogCtx（dialog-vm 域 22 写槽——经 DialogState 记录槽）——
    WriteFaceRow { face: "DialogCtx", field: "dialog_anchor", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "dialog_char", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "dialog_data", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "dialog_page", domain: "dialog-vm", flds: &["dialogPage"] },
    WriteFaceRow { face: "DialogCtx", field: "dialog_page_end", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "dialog_phase", domain: "dialog-vm", flds: &["dialogPhase"] },
    WriteFaceRow { face: "DialogCtx", field: "dialog_speaker", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "dialog_target", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_active", domain: "dialog-vm", flds: &["overlayActive"] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_box_h", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_box_w", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_content_w", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_kind", domain: "dialog-vm", flds: &["overlayKind"] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_left_icon", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_lines", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_lines_per_page", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_page_bottom", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_page_top", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_right_icon", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_saved_color", domain: "dialog-vm", flds: &[] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_text", domain: "dialog-vm", flds: &["overlayText"] },
    WriteFaceRow { face: "DialogCtx", field: "overlay_total_lines", domain: "dialog-vm", flds: &[] },
    // —— EntityCtx（entity 域 1 写槽——entities 记录槽）——
    WriteFaceRow { face: "EntityCtx", field: "entities", domain: "entity", flds: &[
        "entityCount", "entityType", "entityParam", "entityPixelX", "entityPixelY",
        "entityAnimRow", "entityAnimFrame", "entityState", "entityVisible", "entityRemoved",
    ] },
    // —— ItemCtx（P1-2 effect_registry.rs:74 照录 4 写槽）——
    WriteFaceRow { face: "ItemCtx", field: "player", domain: "entity", flds: &[
        "playerHp", "playerAtk", "playerDef", "goldAmount", "yellowKeyCount", "redKeyCount",
        "blueKeyCount", "itemStackSize", "itemStackTypes", "itemStackUses",
    ] },
    WriteFaceRow { face: "ItemCtx", field: "tutorial_seen", domain: "menu-ui", flds: &["tutorialSeen"] },
    WriteFaceRow { face: "ItemCtx", field: "equipped_weapon", domain: "item-effect", flds: &["equippedWeaponType"] },
    WriteFaceRow { face: "ItemCtx", field: "equipped_armor", domain: "item-effect", flds: &["equippedArmorType"] },
    // —— ScriptCtx（script 域 6 写槽）——
    WriteFaceRow { face: "ScriptCtx", field: "current_script_index", domain: "script", flds: &["currentScriptIndex"] },
    WriteFaceRow { face: "ScriptCtx", field: "script_cursor", domain: "script", flds: &[] },
    WriteFaceRow { face: "ScriptCtx", field: "script_delay", domain: "script", flds: &[] },
    WriteFaceRow { face: "ScriptCtx", field: "script_line", domain: "script", flds: &[] },
    WriteFaceRow { face: "ScriptCtx", field: "script_line_flags", domain: "script", flds: &["scriptLineFlags"] },
    WriteFaceRow { face: "ScriptCtx", field: "script_walk_armed", domain: "script", flds: &["scriptWalkArmed"] },
];

#[cfg(test)]
mod tests {
    use super::battle::{BattleSubsystem};
    use super::dialog::{DialogSubsystem};
    use super::entity_service::{EntityCtx, EntityService};
    use super::input_mapper::{InputMapper};
    use super::item_service::ItemService;
    use super::persist_codec::{PersistCodec};
    use super::script_host::{ScriptHost};
    use super::ui_render::{UIRender};
    use super::WRITE_FACE_REGISTRY;

    /// TDD 红→绿驱动：8 trait 形在（编译即断言——骨架判据=编译红「trait 缺位」→骨架绿）。
    #[test]
    fn p2b0_trait_shapes_present() {
        fn _shape<T: BattleSubsystem + DialogSubsystem + PersistCodec + ScriptHost + ItemService + InputMapper + EntityService + UIRender>() {}
        // 上行泛型界随定义即查：任一 trait 缺位⇒编译红（本笔 TDD 红证据面）。
        let _ = std::mem::size_of::<fn()>();
    }

    /// 登记行数=43（增删随 Ctx 写槽同笔——冻结 schema 流程第 1 步）。
    #[test]
    fn p2b0_registry_rows_43() {
        assert_eq!(WRITE_FACE_REGISTRY.len(), 43, "写面归因登记行数漂移：43=Battle10+Dialog22+Entity1+Item4+Script6");
    }

    /// 登记 (field,domain) 逐行对账 field-ownership.json（唯一归属——§3.2 执行则5）。
    #[test]
    fn p2b0_registry_rows_match_field_ownership() {
        let src = include_str!("../../../../data/status/field-ownership.json");
        for row in WRITE_FACE_REGISTRY {
            let dom = ownership_domain_of(src, row.field)
                .unwrap_or_else(|| panic!("归属表缺键：{}（registry face={}）", row.field, row.face));
            assert_eq!(dom, row.domain, "归属域漂移：{}（registry={} 归属表={}）", row.field, row.domain, dom);
        }
    }

    /// 纯数据构造面（ItemCtx 先例形=借用槽+值拷贝；无方法=Ctx 三宪①）。
    #[test]
    fn p2b0_ctx_pure_data_constructible() {
        let mut entities = crate::entity::EntityTable::new(4);
        let grid = crate::entity::CellGrid::new(2, 2);
        let walkable = vec![vec![true; 2]; 2];
        let ctx = EntityCtx { entities: &mut entities, grid: &grid, walkable: &walkable };
        assert!(ctx.walkable[0][0], "读面槽可达");
    }

    /// 从归属表 JSON 文本取 field 的 domain（无 serde 手工扫描：按 "field": "X" 块切，块内取 "domain": "Y"）。
    fn ownership_domain_of(src: &str, field: &str) -> Option<String> {
        let key = "\"field\": \"";
        let mut rest = src;
        while let Some(i) = rest.find(key) {
            let after = &rest[i + key.len()..];
            let name_end = after.find('"')?;
            let name = &after[..name_end];
            let tail = &after[name_end..];
            let next = tail.find(key).unwrap_or(tail.len());
            let chunk = &tail[..next];
            if name == field {
                let dkey = "\"domain\": \"";
                let j = chunk.find(dkey)?;
                let d = &chunk[j + dkey.len()..];
                let de = d.find('"')?;
                return Some(d[..de].to_string());
            }
            rest = tail;
        }
        None
    }
}
