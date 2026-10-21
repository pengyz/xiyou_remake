//! 负例 golden（L1）：资源配置 V2 三表**五类拒载**各一 fixture（survey §2.3 加载层失败语义）。
//!
//! 契约：`docs/spec/resource-config-survey.md` v2 §2.1（schema 字段定义）+ §2.3（五类拒载：
//! 坏 JSON / 坏外键 / 域溢出 0..88 / 帧序越界 / unknown-field=拒载）+ schema 版本字段必校验。
//!
//! TDD 纪律：每类拒载先红后绿（RED 证据入 commit message）。
//!
//! fixture 清单（`tests/fixtures/defs_negative/`）：
//! 1. `bad_json/sprite_defs.json` —— 坏 JSON 拒载
//! 2. `bad_foreign_key/` —— 坏外键拒载（anim_profile 孤儿 + assembly_profile 跨表悬空）
//! 3. `domain_overflow/sprite_defs.json` —— 域溢出拒载（型号域 0..=88）
//! 4. `frame_out_of_bounds/` —— 帧序越界拒载（frames 下标超表长）
//! 5. `unknown_field/sprite_defs.json` —— unknown-field=拒载（防拼写静默）

use game_data::defs::{self, Reject};

fn fixture(rel: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/defs_negative")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("读 fixture 失败 {}: {e}", path.display()))
}

/// 拒载错误必须带定位（路径/偏移），否则排障无从下手（§2.3 错误信息含定位）。
fn assert_located(err: &Reject) {
    let msg = err.to_string();
    assert!(
        msg.contains("偏移") || msg.contains("路径") || msg.contains('['),
        "拒载错误信息缺定位：{msg}"
    );
}

// ---------------------------------------------------------------- 第 5 类：unknown-field=拒载

#[test]
fn reject_class5_unknown_field_record() {
    let src = fixture("unknown_field/sprite_defs.json");
    let err = defs::parse_sprite_defs(&src).expect_err("记录级未知字段必须拒载");
    assert!(
        matches!(&err, Reject::UnknownField { field, .. } if field == "anim_profil"),
        "期望 UnknownField(anim_profil)，实际 {err:?}"
    );
    assert_located(&err);
}

#[test]
fn reject_class5_unknown_field_envelope() {
    let src = fixture("unknown_field/sprite_defs_envelope.json");
    let err = defs::parse_sprite_defs(&src).expect_err("封套级未知字段必须拒载");
    assert!(
        matches!(&err, Reject::UnknownField { field, .. } if field == "sprite_defs_tpyo"),
        "期望 UnknownField(sprite_defs_tpyo)，实际 {err:?}"
    );
    assert_located(&err);
}

#[test]
fn reject_class5_unknown_field_nested() {
    let src = fixture("unknown_field/sprite_defs_nested.json");
    let err = defs::parse_sprite_defs(&src).expect_err("嵌套条目未知字段必须拒载");
    assert!(
        matches!(&err, Reject::UnknownField { field, .. } if field == "categry"),
        "期望 UnknownField(categry)，实际 {err:?}"
    );
    assert_located(&err);
}

// ---------------------------------------------------------------- 第 3 类：域溢出拒载（型号域 0..=88）

#[test]
fn reject_class3_domain_overflow() {
    let src = fixture("domain_overflow/sprite_defs.json");
    let err = defs::parse_sprite_defs(&src).expect_err("型号域溢出必须拒载");
    assert!(
        matches!(err, Reject::DomainOverflow { value: 89, .. }),
        "期望 DomainOverflow(89)，实际 {err:?}"
    );
    assert_located(&err);
}

/// 边界保真：域端点 0 与 88 均合法（0..=88 共 89 型，对应 ANIM_TABLE_IDX 89 项）。
#[test]
fn accept_domain_endpoints_0_and_88() {
    for type_id in [0, 88] {
        let src = format!(
            r#"{{ "schema_version": 2, "sprite_defs": [ {{
              "type_id": {type_id}, "sheet": "sptenemy1", "sheet_index": 0,
              "frame_w": 16, "frame_h": 16, "render_category": 1,
              "anim_profile": "walk_3frame", "assembly_profile": null,
              "insert_block": false, "floor_category_override": [],
              "transition_overrides": [], "evidence": [] }} ] }}"#
        );
        defs::parse_sprite_defs(&src)
            .unwrap_or_else(|e| panic!("type_id={type_id} 应在域内：{e}"));
    }
}

// ---------------------------------------------------------------- 第 2 类：坏外键拒载

fn load_fk_bundle(assembly_ids: &[&str]) -> Result<defs::DefBundle, Reject> {
    defs::DefBundle::load(
        &fixture("bad_foreign_key/sprite_defs.json"),
        &fixture("bad_foreign_key/anim_defs.json"),
        &fixture("bad_foreign_key/sheet_index.json"),
        assembly_ids,
    )
}

#[test]
fn reject_class2_bad_foreign_key_anim_profile() {
    let err = load_fk_bundle(&[]).expect_err("anim_profile 孤儿必须拒载");
    assert!(
        matches!(err, Reject::BadForeignKey { .. }),
        "期望 BadForeignKey，实际 {err:?}"
    );
    assert_located(&err);
    assert!(
        err.to_string().contains("no_such_anim"),
        "错误信息必须点名悬空外键值：{err}"
    );
}

#[test]
fn reject_class2_bad_foreign_key_assembly_profile() {
    let sprite = r#"{ "schema_version": 2, "sprite_defs": [ {
      "type_id": 72, "sheet": "sptenemy1", "sheet_index": 0,
      "frame_w": 32, "frame_h": 45, "render_category": 32,
      "anim_profile": "walk_3frame", "assembly_profile": "seg67_pieces",
      "insert_block": false, "floor_category_override": [],
      "transition_overrides": [], "evidence": [] } ] }"#;
    let err = defs::DefBundle::load(
        sprite,
        &fixture("bad_foreign_key/anim_defs.json"),
        &fixture("bad_foreign_key/sheet_index.json"),
        &[], // SEG 片件表为空 ⇒ assembly_profile 跨表悬空
    )
    .expect_err("assembly_profile 跨表悬空必须拒载");
    assert!(
        matches!(err, Reject::BadForeignKey { .. }),
        "期望 BadForeignKey，实际 {err:?}"
    );
    assert!(err.to_string().contains("seg67_pieces"), "错误信息必须点名悬空外键值：{err}");
}

#[test]
fn reject_class2_bad_foreign_key_sheet() {
    let sprite = r#"{ "schema_version": 2, "sprite_defs": [ {
      "type_id": 72, "sheet": "no_such_sheet", "sheet_index": 0,
      "frame_w": 32, "frame_h": 45, "render_category": 32,
      "anim_profile": "walk_3frame", "assembly_profile": null,
      "insert_block": false, "floor_category_override": [],
      "transition_overrides": [], "evidence": [] } ] }"#;
    let err = defs::DefBundle::load(
        sprite,
        &fixture("bad_foreign_key/anim_defs.json"),
        &fixture("bad_foreign_key/sheet_index.json"),
        &[],
    )
    .expect_err("sheet 图集容器悬空必须拒载");
    assert!(
        matches!(err, Reject::BadForeignKey { .. }),
        "期望 BadForeignKey，实际 {err:?}"
    );
    assert_located(&err);
}

/// sprite_ref 指空=拒载（§2.3 孤儿政策；enemy_defs.sprite_ref 单字段引用的装载前校验面）。
#[test]
fn reject_class2_sprite_ref_dangling() {
    let sprite = r#"{ "schema_version": 2, "sprite_defs": [ {
      "type_id": 72, "sheet": "sptenemy1", "sheet_index": 0,
      "frame_w": 32, "frame_h": 45, "render_category": 32,
      "anim_profile": "walk_3frame", "assembly_profile": null,
      "insert_block": false, "floor_category_override": [],
      "transition_overrides": [], "evidence": [] } ] }"#;
    let bundle = defs::DefBundle::load(
        sprite,
        &fixture("bad_foreign_key/anim_defs.json"),
        &fixture("bad_foreign_key/sheet_index.json"),
        &[],
    )
    .expect("最小合法 bundle 应加载成功");
    let err = bundle.validate_sprite_ref(None).expect_err("sprite_ref 指空必须拒载");
    assert!(matches!(err, Reject::BadForeignKey { .. }), "期望 BadForeignKey，实际 {err:?}");
    assert_located(&err);
    let err = bundle
        .validate_sprite_ref(Some(999))
        .expect_err("sprite_ref 悬空必须拒载");
    assert!(matches!(err, Reject::BadForeignKey { .. }), "期望 BadForeignKey，实际 {err:?}");
}

/// id 稳定性（§2.3）：保真 id 冻结 ⇒ type_id 重复=拒载。
#[test]
fn reject_duplicate_type_id() {
    let sprite = r#"{ "schema_version": 2, "sprite_defs": [ {
      "type_id": 72, "sheet": "sptenemy1", "sheet_index": 0,
      "frame_w": 32, "frame_h": 45, "render_category": 32,
      "anim_profile": "walk_3frame", "assembly_profile": null,
      "insert_block": false, "floor_category_override": [],
      "transition_overrides": [], "evidence": [] },
      { "type_id": 72, "sheet": "sptenemy1", "sheet_index": 1,
      "frame_w": 32, "frame_h": 45, "render_category": 32,
      "anim_profile": "walk_3frame", "assembly_profile": null,
      "insert_block": false, "floor_category_override": [],
      "transition_overrides": [], "evidence": [] } ] }"#;
    let err = defs::DefBundle::load(
        sprite,
        &fixture("bad_foreign_key/anim_defs.json"),
        &fixture("bad_foreign_key/sheet_index.json"),
        &[],
    )
    .expect_err("type_id 重复必须拒载（id 稳定性冻结）");
    assert_located(&err);
}

/// 级联政策（§2.3）：删 sprite_defs 行前校验引用——引用清单查询面。
#[test]
fn cascade_reference_listing() {
    let sprite = r#"{ "schema_version": 2, "sprite_defs": [ {
      "type_id": 72, "sheet": "sptenemy1", "sheet_index": 0,
      "frame_w": 32, "frame_h": 45, "render_category": 32,
      "anim_profile": "walk_3frame", "assembly_profile": null,
      "insert_block": false, "floor_category_override": [],
      "transition_overrides": [], "evidence": [] } ] }"#;
    let bundle = defs::DefBundle::load(
        sprite,
        &fixture("bad_foreign_key/anim_defs.json"),
        &fixture("bad_foreign_key/sheet_index.json"),
        &[],
    )
    .expect("最小合法 bundle 应加载成功");
    assert_eq!(bundle.referencing_type_ids("walk_3frame"), vec![72]);
    assert!(bundle.referencing_type_ids("no_such_anim").is_empty());
}

// ---------------------------------------------------------------- 第 4 类：帧序越界拒载（frames 下标超表长）

#[test]
fn reject_class4_frame_index_out_of_bounds() {
    let err = defs::DefBundle::load(
        &fixture("frame_out_of_bounds/sprite_defs.json"),
        &fixture("frame_out_of_bounds/anim_defs.json"),
        &fixture("frame_out_of_bounds/sheet_index.json"),
        &[],
    )
    .expect_err("frames 下标超表长必须拒载");
    assert!(
        matches!(err, Reject::FrameIndexOutOfBounds { index: 5, table_len: 4, .. }),
        "期望 FrameIndexOutOfBounds(index=5, table_len=4)，实际 {err:?}"
    );
    assert_located(&err);
}

/// 边界保真：末槽下标（==表长-1）合法。
#[test]
fn accept_frame_index_at_last_slot() {
    let anim = r#"{ "schema_version": 2, "anim_defs": [ {
      "id": "walk_3frame", "frames": [0, 3], "step_ticks": 4,
      "loop_mode": "cycle", "evidence": [] } ] }"#;
    defs::DefBundle::load(
        &fixture("bad_foreign_key/sprite_defs.json").replace("no_such_anim", "walk_3frame"),
        anim,
        &fixture("bad_foreign_key/sheet_index.json"), // container_counts=[4] ⇒ 表长 4
        &[],
    )
    .expect("frames=[0,3] 表长 4 应加载成功");
}

// ---------------------------------------------------------------- 第 1 类：坏 JSON 拒载

#[test]
fn reject_class1_bad_json() {
    let src = fixture("bad_json/sprite_defs.json");
    let err = defs::parse_sprite_defs(&src).expect_err("坏 JSON 必须拒载");
    assert!(
        matches!(err, Reject::BadJson { .. }),
        "期望 Reject::BadJson，实际 {err:?}"
    );
    assert_located(&err);
}

// ---------------------------------------------------------------- schema 版本字段必校验（§2.3）

#[test]
fn reject_schema_version_mismatch() {
    let src = r#"{ "schema_version": 1, "sprite_defs": [] }"#;
    let err = defs::parse_sprite_defs(src).expect_err("schema 版本不符必须拒载");
    assert!(
        matches!(err, Reject::SchemaVersion { .. }),
        "期望 Reject::SchemaVersion，实际 {err:?}"
    );
    assert_located(&err);
}

#[test]
fn reject_schema_version_missing() {
    let src = r#"{ "sprite_defs": [] }"#;
    let err = defs::parse_sprite_defs(src).expect_err("schema 版本缺失必须拒载");
    assert!(
        matches!(err, Reject::SchemaVersion { .. }),
        "期望 Reject::SchemaVersion，实际 {err:?}"
    );
}
