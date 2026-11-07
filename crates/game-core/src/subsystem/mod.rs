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
//! ItemCtx=P1-2 effect_registry.rs:74 已存形态照录（不改史）；enemyBaseGold 等派生键不入（输出面非写面）。

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
    use super::battle::{BattleCtx, BattleEvent, BattleOutcome, BattleSubsystem, EnemyTypeId};
    use super::dialog::{DialogCtx, DialogStep, DialogSubsystem};
    use super::entity_service::{EntityCtx, EntityService};
    use super::input_mapper::{InputMapper};
    use super::item_service::ItemService;
    use super::input_mapper::RawInput;
    use super::persist_codec::{Blob, PersistCodec, SceneState};
    use super::script_host::{ScriptCtx, ScriptHost, ScriptStep};
    use super::ui_render::{UiFrame, UIRender};
    use crate::controls::Action;
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

    /// 【F3 判别力】逐 trait 机器锁签名（编译期钉形——改签名即红；spec §3.1 逐字形）。
    #[allow(dead_code)]
    mod sig_lock {
        use super::*;
        use crate::scene::ScriptOp;

        pub fn _sig_battle<T: BattleSubsystem>(t: &mut T, ctx: &mut BattleCtx, x: EnemyTypeId, ev: BattleEvent) -> BattleOutcome {
            t.resolve_bump(ctx, x, ev)
        }
        pub fn _sig_dialog_tick<T: DialogSubsystem>(t: &mut T, ctx: &mut DialogCtx) -> DialogStep {
            t.tick(ctx)
        }
        pub fn _sig_dialog_set_typewriter<T: DialogSubsystem>(t: &mut T, s: &str, y: i32, pos: i32) {
            t.set_typewriter(s, y, pos)
        }
        pub fn _sig_dialog_m082<T: DialogSubsystem>(t: &mut T, v: i32) {
            t.m_082_set_dialog_target(v)
        }
        pub fn _sig_persist_ser<T: PersistCodec>(t: &T, st: &SceneState) -> Blob {
            t.serialize(st)
        }
        pub fn _sig_persist_de<T: PersistCodec>(t: &T, blob: &Blob) -> SceneState {
            t.deserialize(blob)
        }
        pub fn _sig_script_op<T: ScriptHost>(t: &mut T, ctx: &mut ScriptCtx, op: ScriptOp) -> ScriptStep {
            t.execute_op(ctx, op)
        }
        pub fn _sig_input_map<T: InputMapper>(t: &T, ev: RawInput) -> Action {
            t.map(ev)
        }
        pub fn _sig_ui_snapshot<T: UIRender>(t: &T) -> UiFrame {
            t.snapshot()
        }
        // 【F4+批1】EntityService 9 方法机器钉名（批1 终形=逐字迁移自 HostCtx script.rs:128-168）。
        pub fn _sig_entity_cell_entity<T: EntityService>(t: &T, x: i32, y: i32, type_id: i32) -> i32 {
            t.cell_entity(x, y, type_id)
        }
        pub fn _sig_entity_entity_type<T: EntityService>(t: &T, idx: i32) -> i32 {
            t.entity_type(idx)
        }
        pub fn _sig_entity_remove_entity<T: EntityService>(t: &mut T, idx: i32) {
            t.remove_entity(idx)
        }
        pub fn _sig_entity_remove_all_of_type<T: EntityService>(t: &mut T, type_id: i32) {
            t.remove_all_of_type(type_id)
        }
        pub fn _sig_entity_swd_convert_type81<T: EntityService>(t: &mut T) {
            t.swd_convert_type81()
        }
        pub fn _sig_entity_entity_pixel<T: EntityService>(t: &T, idx: i32) -> (i32, i32) {
            t.entity_pixel(idx)
        }
        pub fn _sig_entity_reload_floor_entities<T: EntityService>(t: &mut T, floor: i32) {
            t.reload_floor_entities(floor)
        }
        pub fn _sig_entity_spawn_entity<T: EntityService>(t: &mut T, type_id: i32, px: i32, py: i32) -> i32 {
            t.spawn_entity(type_id, px, py)
        }
        pub fn _sig_entity_set_entity_route_flag<T: EntityService>(t: &mut T, idx: i32) {
            t.set_entity_route_flag(idx)
        }
    }

    /// 【F4 方法名机器断言】EntityService 9 方法名清单钉死
    /// （批1 终形=逐字迁移 HostCtx 实体面——防 doc 注释名字与实现漂移）。
    #[test]
    fn p2b1_entity_service_method_names() {
        let src = include_str!("entity_service.rs");
        for name in [
            "cell_entity", "entity_type", "remove_entity", "remove_all_of_type",
            "swd_convert_type81", "entity_pixel", "reload_floor_entities",
            "spawn_entity", "set_entity_route_flag",
        ] {
            assert!(
                src.contains(&format!("fn {name}(")),
                "EntityService 方法名漂移：{name} 失联（F4 名单断言）"
            );
        }
    }

    /// 【F3 判别力】其余 4 Ctx 纯数据构造（5 Ctx 之一 EntityCtx 见上）——
    /// ItemCtx=effect_registry.rs:74 照录构造（不改史）。
    #[test]
    fn p2b0_ctx_pure_data_constructible_all() {
        // BattleCtx：10 写槽+3 读槽
        let (mut ba, mut bh, mut btick) = (false, 0i32, 0i32);
        let (mut banim, mut bfin, mut bnc, mut brr) = (0i32, false, false, false);
        let (mut btgt, mut sdb, mut bdig) = (0i32, false, Vec::<i32>::new());
        let player = crate::combat::PlayerCombat {
            hp: 1, atk: 1, def: 1, yellow_keys: 0, blue_keys: 0, red_keys: 0,
            gold: 0, item_types: Vec::new(), item_uses: Vec::new(), item_stack_size: 0,
        };
        let enemies: Vec<crate::combat::EnemyDef> = Vec::new();
        let enemy_base: Vec<crate::combat::EnemyDef> = Vec::new();
        let bctx = BattleCtx {
            battle_active: &mut ba, battle_anim_tick: &mut banim,
            battle_digit_by_type: &mut bdig, battle_enemy_hp: &mut bh,
            battle_finishing: &mut bfin, battle_no_counter: &mut bnc,
            battle_rerun_scene: &mut brr, battle_target: &mut btgt,
            battle_tick: &mut btick, show_battle_digits: &mut sdb,
            player: &player, enemies: &enemies, enemy_base: &enemy_base,
        };
        assert_eq!(bctx.battle_active as *const bool as usize != 0, true);

        // DialogCtx：记录槽+3 值拷贝读
        let mut dialog = crate::scene::DialogState {
            dialog_phase: 0, dialog_page: 0, dialog_page_end: 0, dialog_speaker: 0,
            dialog_anchor: 0, dialog_char: 0, dialog_target: (0, 0),
            dialog_data: crate::scene::DialogData {
                scripts: Vec::new(), texts: Vec::new(), speakers: Vec::new(), bubbles: Vec::new(),
            },
            overlay_active: false, overlay_kind: 0, overlay_text: None,
            overlay_box_w: 0, overlay_box_h: 0, overlay_left_icon: 0, overlay_right_icon: 0,
            overlay_lines: Vec::new(), overlay_page_top: 0, overlay_page_bottom: 0,
            overlay_lines_per_page: 0, overlay_total_lines: 0, overlay_content_w: 0,
            overlay_saved_color: 0,
        };
        let dctx = DialogCtx { dialog: &mut dialog, key_value: 0, key_held: 0, frame_odd: false };
        assert_eq!(dctx.key_value, 0);

        // ScriptCtx：6 写槽+2 读槽
        let (mut si, mut sc, mut sd) = (0usize, 0usize, 0i32);
        let (mut sl, mut slf, mut swa) = (String::new(), Vec::<bool>::new(), false);
        let mut entities = crate::entity::EntityTable::new(4);
        let sctx = ScriptCtx {
            current_script_index: &mut si, script_cursor: &mut sc, script_delay: &mut sd,
            script_line: &mut sl, script_line_flags: &mut slf, script_walk_armed: &mut swa,
            player: &player, floor: 1,
                entities: &entities,
        };
        assert_eq!(*sctx.script_delay, 0);

        // ItemCtx：effect_registry.rs:74 照录（P1-2 已存形态不改史）
        let mut pc = crate::combat::PlayerCombat {
            hp: 1, atk: 1, def: 1, yellow_keys: 0, blue_keys: 0, red_keys: 0,
            gold: 0, item_types: Vec::new(), item_uses: Vec::new(), item_stack_size: 0,
        };
        let (mut ts, mut ew, mut ea) = (false, 0i32, 0i32);
        let ictx = crate::effect_registry::ItemCtx {
            player: &mut pc, floor: 1, tutorial_seen: &mut ts,
            equipped_weapon: &mut ew, equipped_armor: &mut ea,
            equip_types: [0; 12], equip_bonuses: [0; 12],
        };
        assert_eq!(ictx.floor, 1);
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
