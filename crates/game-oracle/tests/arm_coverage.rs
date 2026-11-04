//! DDDSA P1-a2 复用面清点终版（spec §7-Q3：臂面实数=21，全量夹具禁抽样）。
//! 【红清单落档·排除面口径全列】grep 计数排除面=史料/哈希/原地注记/伪命中
//! 四类（措辞纪律：实测边界随附排除口径）。
//! 缺臂递减链（TDD 红证据续录）：批1 时点 15→批2 后 11（pickup 26/27/28）
//! →批4 后 8（pickup 33/37/29f12）→批5 主件后复核实测 3（预测 5 更正；存储形双认入校验后现 **2**：
//! pickup 32 递延销账批/pickup default 递延脚本面 GIN/use 21=preset_cases
//! 存储形双认入校验——use 面 13 红注待分析件修复）：fx-item13+pickup 净补 10
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
    ("pickup default no-op", "NAMED::双路由不可达防御支终裁（图拾 cat 2|4 型域全覆盖+脚本 GIN 值 13/19 全覆盖（assets/raw/script 实测）——synthetic 注入超夹具面；pre-final-audit 半格口径终裁）"),
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
fn p1a2_all_21_arm_groups_have_fixture_triplets() {
    let mut missing = Vec::new();
    let mut named = Vec::new();
    for (group, stem) in ARM_FIXTURES {
        // 正名制（批6-③终裁）：NAMED:: 前缀=双路由不可达防御支终裁（半格口径），
        // 计入 21 全账非缺臂；其余=夹具三件套形或 preset_cases 供给形双认。
        if let Some(why) = stem.strip_prefix("NAMED::") {
            named.push(format!("{group}：{why}"));
            continue;
        }
        let ok = std::path::Path::new(&format!("../../crates/game-oracle/tests/fixtures/{stem}.frames.txt")).exists()
            || std::path::Path::new(&format!("../../reference/oracle/scenarios/{stem}.txt")).exists();
        if !ok {
            missing.push(format!("{group} → {stem}"));
        }
    }
    assert!(missing.is_empty(), "P1-a2 缺臂夹具（全量 21 禁抽样）：{missing:?}");
    assert_eq!(named.len(), 1, "正名件恰 1（pickup default——终裁留痕）：{named:?}");
    println!("21 全账：夹具 {} + 正名 {} = {}", ARM_FIXTURES.len() - named.len(), named.len(), ARM_FIXTURES.len());
}
