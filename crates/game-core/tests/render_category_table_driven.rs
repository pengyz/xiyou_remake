//! V2-P2 点4：render_category 消费→表驱动（10 臂）+ floor_category_override。
//!
//! 消费点=entity.rs renderCategoryByType 静态 10 臂（initEntityTables
//! a.java:6268-6298）+ scene.rs entity_category 层相关覆写（m_122 尾
//! a.java:9669-9667：type 72 floor50=8、其余 32）。
//!
//! 一事实一宿主（survey §3 硬条件）：真值宿主=sprite_defs.render_category；
//! 旧 10 臂收编为**保真冻结镜像** [`entity::legacy_render_category_mirror`]，
//! 校验器断言双写 diff=0（本文件+tests/one_fact_one_host.rs）。

use game_core::entity;
use game_core::resource_cfg::{self, ResourceCfg};

fn global() -> &'static ResourceCfg {
    resource_cfg::global()
}

const SPRITE_SRC: &str = include_str!("../../../data/tables/sprite_defs.json");
const ANIM_SRC: &str = include_str!("../../../data/tables/anim_defs.json");
const SHEET_SRC: &str = include_str!("../../../data/tables/sheet_index.json");

/// 逐记录字段变异（sprite_defs 条目内按键换值——值以逗号收尾的整数字段）。
fn mutate_record_field(src: &str, type_id: u32, key: &str, new_val: &str) -> String {
    let anchor = format!("\"type_id\": {type_id},");
    let start = src.find(&anchor).expect("记录锚应命中");
    let key_pat = format!("\"{key}\": ");
    let kstart = src[start..].find(&key_pat).expect("字段键应命中") + start;
    let kend = src[kstart..].find(',').expect("整数值应以逗号收尾") + kstart;
    format!("{}\"{}\": {}{}", &src[..kstart], key, new_val, &src[kend..])
}

#[test]
fn task7_render_category_matches_legacy_10_arms() {
    // 逐型双写 diff=0：表驱动消费面 == 保真冻结镜像（10 臂原语义）
    for t in -2i32..130 {
        assert_eq!(
            entity::render_category(t),
            entity::legacy_render_category_mirror(t),
            "型 {t} 双写 diff=0"
        );
    }
    // 10 臂逐臂抽查（evidence 面：initEntityTables a.java:6268-6298）
    for (t, want) in [
        (1, 1), (12, 1), (13, 2), (32, 2), (33, 4), (40, 4), (41, 8), (73, 8),
        (76, 1), (81, 1), (82, 1), (83, 1), // 特例覆盖优先：76/81/82/83→1
        (77, 16), (78, 16),                 // 77/78→16（覆盖 41..=78 臂）
        (79, 4), (80, 4),                   // 79/80→4
        (72, 32), (84, 32), (87, 32),       // 72/84/87→32
        (85, 2), (86, 2),                   // 85/86→2
        (0, 0), (88, 0), (89, 0), (-1, 0),  // 域外/无类目→0
    ] {
        assert_eq!(entity::render_category(t), want, "型 {t} 臂值");
        assert_eq!(global().render_category(t), want, "型 {t} 表值");
    }
}

#[test]
fn task7_render_category_values_track_table() {
    // 变异表：型 44 的 render_category 8→16——表驱动消费面跟随表
    let mutated = mutate_record_field(SPRITE_SRC, 44, "render_category", "16");
    assert_ne!(mutated, SPRITE_SRC, "变异锚点应命中型 44 记录");
    let cfg = ResourceCfg::parse(&mutated, ANIM_SRC, SHEET_SRC).expect("变异表应可解析");
    assert_eq!(cfg.render_category(44), 16, "类目跟随表（变异 16）");
    assert_eq!(global().render_category(44), 8, "保真表原值 8 不动");
    assert_eq!(entity::render_category(44), 8, "进程级单例走保真表");
}

#[test]
fn task7_floor_category_override_table_driven() {
    let cfg = global();
    // m_122 尾（a.java:9669-9667）：type 72 类别层相关——floor50=8、其余 32
    assert_eq!(cfg.floor_category(72, 50), Some(8), "floor50 覆写在表（floor_category_override）");
    assert_eq!(cfg.floor_category(72, 49), None, "非 50 层无覆写→静态表 32");
    assert_eq!(cfg.floor_category(44, 50), None, "其余型无覆写");
    assert_eq!(entity::render_category(72), 32, "静态面仍 32（层相关覆写单独走 floor_category）");
}

#[test]
fn task7_floor_category_values_track_table() {
    // 变异表：floor_category_override 值 8→16
    let start = SPRITE_SRC.find("\"type_id\": 72,").expect("型 72 记录");
    let span_end = SPRITE_SRC[start..].find("\"type_id\": 73,").map(|p| start + p).unwrap_or(SPRITE_SRC.len());
    let span = &SPRITE_SRC[start..span_end];
    let mutated_span = span.replace("\"category\": 8", "\"category\": 16");
    assert_ne!(mutated_span, span, "变异锚点应命中 floor_category_override");
    let mutated = format!("{}{}{}", &SPRITE_SRC[..start], mutated_span, &SPRITE_SRC[span_end..]);
    let cfg = ResourceCfg::parse(&mutated, ANIM_SRC, SHEET_SRC).expect("变异表应可解析");
    assert_eq!(cfg.floor_category(72, 50), Some(16), "floor50 覆写值跟随表");
}
