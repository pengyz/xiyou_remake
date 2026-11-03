//! DDDSA P1-a2 复用面清点终版（spec §7-Q3：臂面实数=21，全量夹具禁抽样）。
//! 【红清单落档·排除面口径全列】grep 计数排除面=史料/哈希/原地注记/伪命中
//! 四类（措辞纪律：实测边界随附排除口径）。
//! 批1 时点缺臂 15（TDD 红证据实录）→批2 后缺臂 **11**（fx-pickup-26/27/28 落）：fx-item13+pickup 净补 10
//! +use 4（fx-item13-use/preset-item21 归位修正后重计）——逐批收敛，全 21 绿时
//! 解除 #[ignore] 启用强制。

/// 臂组 ↔ 夹具 stem 清单（spec §7-Q3 枚举逐组；型32=递延标记见映射表）。
const ARM_FIXTURES: &[(&str, &str)] = &[
    ("pickup 13..=25|85|86", "fx-item13"),
    ("pickup 26", "fx-pickup-26"),
    ("pickup 27", "fx-pickup-27"),
    ("pickup 28", "fx-pickup-28"),
    ("pickup 29", "fx-pickup-29"),
    ("pickup 30", "fx-pickup-30"),
    ("pickup 31", "fx-pickup-31"),
    ("pickup 32（递延：销账批）", "fx-pickup-32"),
    ("pickup 33..=36|79", "fx-pickup-33"),
    ("pickup 37..=40|80", "fx-pickup-37"),
    ("pickup default no-op", "fx-pickup-default"),
    ("use 13（净补）", "fx-use-13"),
    ("use 14", "preset-floor5"),
    ("use 16|17", "fx-item17a"),
    ("use 18", "fx-item18a"),
    ("use 19", "fx-item19a"),
    ("use 20（含 preset-item2122）", "fx-item20a"),
    ("use 21", "preset-item21"),
    ("use 85（净补）", "fx-use-85"),
    ("use 86", "fx-boss68a"),
    ("use default no-op（净补）", "fx-use-default"),
];

#[test]
#[ignore = "P1-a2 进行中——全 21 夹具齐后解除启用（红清单落档件）"]
fn p1a2_all_21_arm_groups_have_fixture_triplets() {
    let mut missing = Vec::new();
    for (group, stem) in ARM_FIXTURES {
        let ok = std::path::Path::new(&format!("../../crates/game-oracle/tests/fixtures/{stem}.frames.txt")).exists();
        if !ok {
            missing.push(format!("{group} → {stem}"));
        }
    }
    assert!(missing.is_empty(), "P1-a2 缺臂夹具（全量 21 禁抽样）：{missing:?}");
}
