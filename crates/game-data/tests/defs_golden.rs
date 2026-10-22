//! 正例 L1 golden：资源配置 V2 三表（survey §2.1 schema）全量解析+真表联调。
//!
//! 风格沿 `tests/golden.rs`（55 maplv+55 sprite+16 packed 先例）：
//! - **真表联调**（task-5 产物 `data/tables/*.json`，commit a284203）：89/11/16 规模断言+
//!   外键全通+帧序域全通；张数表实锚 a.java:453（2026-10-22 task-5 勘误，非 451）。
//! - **最小正例**（`tests/fixtures/defs_positive/minimal/`）：逐字段断言（§2.1 全字段含
//!   transition_overrides 事件条件规则形/assembly_profile/density 默认 1）。
//!
//! 五类拒载负例见 `tests/defs_negative.rs`（fixtures/defs_negative/ 五件）。

use game_data::defs::{self, DefBundle, SpriteDef};

fn repo_file(rel: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("读表失败 {}: {e}", path.display()))
}

fn fixture(rel: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/defs_positive")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("读 fixture 失败 {}: {e}", path.display()))
}

// ---------------------------------------------------------------- 真表联调（data/tables/，task-5 产物）

#[test]
fn real_tables_load_89_11_16() {
    let bundle = DefBundle::load(
        &repo_file("data/tables/sprite_defs.json"),
        &repo_file("data/tables/anim_defs.json"),
        &repo_file("data/tables/sheet_index.json"),
        &[], // assembly id 由 sheet_index.assembly_profiles 提供（SEG 片件表）
    )
    .unwrap_or_else(|e| panic!("真表联调拒载：{e}"));
    assert_eq!(bundle.sprite_defs.len(), 89, "sprite_defs 应 89 型（ANIM_TABLE_IDX 89 项）");
    assert_eq!(bundle.anim_defs.len(), 11, "anim_defs 应 11 行（ANIM_OFFSET_TABLE 帧序）");
    assert_eq!(bundle.sheet_index.containers.len(), 16, "sheet_index 应 16 容器");
    // 张数表=resourceImageCounts（实锚 a.java:453，2026-10-22 勘误；§2.1 记 451 已被 task-5 证伪）
    assert_eq!(
        bundle.sheet_index.container_counts,
        vec![8, 1, 12, 4, 13, 23, 10, 20, 25, 6, 2, 2, 12, 20, 1, 2]
    );
    // 型号域 0..=88 全量唯一（id 稳定性冻结）
    let mut ids: Vec<i64> = bundle.sprite_defs.iter().map(|s| s.type_id).collect();
    ids.sort_unstable();
    assert_eq!(ids, (0..=88).collect::<Vec<_>>(), "type_id 应恰为 0..=88");
    // assembly_profile 跨表全通（seg67 片件表 id 来自 sheet_index.assembly_profiles）
    assert_eq!(bundle.sheet_index.assembly_profiles, vec!["seg67".to_string()]);
    // sprite_ref 校验面：指空拒载、已知 id 通过
    assert!(bundle.validate_sprite_ref(None).is_err());
    assert!(bundle.validate_sprite_ref(Some(72)).is_ok());
}

/// 真表全量外键/帧序域零违例（联调判据：解析即校验，load 成功=全通）。
#[test]
fn real_tables_fk_and_frame_domain_clean() {
    let sprite_src = repo_file("data/tables/sprite_defs.json");
    let anim_src = repo_file("data/tables/anim_defs.json");
    let sheet_src = repo_file("data/tables/sheet_index.json");
    let bundle = DefBundle::load(&sprite_src, &anim_src, &sheet_src, &[])
        .expect("真表应全通（外键/帧序域零违例）");
    // 每个 anim_profile 都有引用方或孤悬允许（级联查询面可用）
    assert!(!bundle.referencing_type_ids("anim_row_0").is_empty());
}

// ---------------------------------------------------------------- 最小正例（§2.1 全字段逐断言）

#[test]
fn minimal_positive_fields() {
    let bundle = DefBundle::load(
        &fixture("minimal/sprite_defs.json"),
        &fixture("minimal/anim_defs.json"),
        &fixture("minimal/sheet_index.json"),
        &[],
    )
    .expect("最小正例应加载成功");
    let s72: &SpriteDef = bundle.sprite_defs.iter().find(|s| s.type_id == 72).unwrap();
    assert_eq!(s72.sheet.as_deref(), Some("sptenemy1"));
    assert_eq!(s72.sheet_index, Some(3));
    assert_eq!((s72.frame_w, s72.frame_h), (32, Some(45)));
    assert_eq!(s72.render_category, 32);
    assert_eq!(s72.anim_profile, "walk_3frame");
    assert_eq!(s72.assembly_profile.as_deref(), Some("seg67"));
    assert!(!s72.insert_block);
    assert_eq!(s72.floor_category_override.len(), 1);
    assert_eq!(s72.floor_category_override[0].floor, 50);
    assert_eq!(s72.floor_category_override[0].category, 8);
    // transition_overrides 事件条件规则形（§2.1 ★R3）
    assert_eq!(s72.transition_overrides.len(), 1);
    let t0 = &s72.transition_overrides[0];
    assert_eq!(t0.on, "mark_des_removed");
    assert_eq!(t0.evidence.as_deref(), Some("a.java:6332-6347"));
    assert_eq!(t0.by_category.len(), 1);
    assert_eq!(t0.by_category[0].0, "*");
    assert_eq!(t0.by_category[0].1.anim_row, Some(8));
    assert_eq!(t0.by_category[0].1.sprite_h, None);
    // density 显式 2（HD 预留字段）
    assert_eq!(s72.density, 2);
    // 逐值锚三元组（★R1）
    assert_eq!(s72.evidence.len(), 1);
    assert_eq!(s72.evidence[0].index, 72);
    assert_eq!(s72.evidence[0].src, "a.java:498-586");
    // 可空 sheet + density 默认 1
    let s0: &SpriteDef = bundle.sprite_defs.iter().find(|s| s.type_id == 0).unwrap();
    assert_eq!(s0.sheet, None);
    assert_eq!(s0.density, 1, "density 缺省应为 1（plan 拍板 2）");
    // anim 逐字段
    let walk = bundle.anim_defs.iter().find(|a| a.id == "walk_3frame").unwrap();
    assert_eq!(walk.frames, vec![0, 1, 2, 1]);
    assert_eq!(walk.step_ticks, 4);
    assert_eq!(walk.loop_mode, "cycle");
    assert_eq!(walk.evidence[0].src, "a.java:485-497");
    // sheet_index 逐字段
    assert_eq!(bundle.sheet_index.containers, vec!["sflogo".to_string(), "sptenemy1".to_string()]);
    assert_eq!(bundle.sheet_index.container_counts, vec![8, 4]);
    assert_eq!(bundle.sheet_index.assembly_profiles, vec!["seg67".to_string()]);
}
