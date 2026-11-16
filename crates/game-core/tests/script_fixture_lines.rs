//! 夹具脚本行口径（2026-11-16 · 0x1E 剥离）。
//!
//! `DialogData::from_fixtures` 是**生产**装载路径（scene_persist.rs:219）：
//! 它必须与 Java 原始行逐字符同构——不含提取器追加的 U+001E，且保留行尾空格
//! （TAK 的 page_end 以 `' '` 为终止符）。
//! 裁定依据：docs/superpowers/proposals/2026-11-16-script-command-pipeline-proposal.md §0′.4′。
use game_core::scene::fixture_script_line;

fn production_lines() -> Vec<String> {
    include_str!("fixtures/level_scripts.txt")
        .lines()
        .map(fixture_script_line)
        .collect()
}

#[test]
fn no_record_separator_left() {
    for (i, l) in production_lines().iter().enumerate() {
        assert!(!l.contains('\u{1e}'), "槽 {i} 仍含 U+001E：{l:?}");
        assert!(!l.contains('\r'), "槽 {i} 含 CR");
    }
}

#[test]
fn line_lengths_match_java_chars() {
    let ls = production_lines();
    assert_eq!(ls.len(), 68, "68 槽脚本行");
    assert_eq!(ls[0].len(), 164, "槽 0 = Java 164 字符");
    assert_eq!(ls[6], "TAK_53_58 ", "槽 6 行尾空格是 Java 数据的一部分");
    assert_eq!(ls[40].len(), 43, "序章槽 40：SEE 消费后 cursor=43");
    assert!(ls[40].ends_with(' '), "行尾空格不得被 trim 吃掉");
    assert_eq!(ls[67], "TAK_264_268 ", "末槽尾空格保留，否则 TAK page_end 解析 panic");
}

#[test]
fn production_matches_test_table() {
    // 测试表口径（script_golden.rs:15-30）：split("\u{1e}\n") + trim_end_matches('\u{1e}')
    let table: Vec<String> = include_str!("fixtures/level_scripts.txt")
        .split("\u{1e}\n")
        .map(|s| s.trim_end_matches('\u{1e}').to_string())
        .collect();
    assert_eq!(production_lines(), table, "生产装载与测试表必须逐字符一致");
}
