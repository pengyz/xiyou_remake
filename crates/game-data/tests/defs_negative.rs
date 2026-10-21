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
