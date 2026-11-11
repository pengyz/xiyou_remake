//! 【求解器批2】T1 存在性闸（提案 §3.1 T1；计划 :328 `-- --list | grep -c solve_local ≥1`）。
//!
//! 断言口径：只钉 **JSON 形状/字段类型**（防空过），不钉具体数值——数值断言归 T3/T6/T10，
//! 本测试跨切片恒绿。
use game_solver::solve_local::{solve_local, SolveInput};

/// 取 `"key":` 之后的标量值 token（到下一个顶层 `,` 为止）；route 数组除外（取到末尾 `}` 前）。
fn scalar<'a>(json: &'a str, key: &str) -> &'a str {
    let pat = format!("\"{key}\":");
    let i = json.find(&pat).unwrap_or_else(|| panic!("缺字段 {key}：{json}"));
    let rest = &json[i + pat.len()..];
    let end = rest.find(',').unwrap_or(rest.rfind('}').expect("JSON 无右括号"));
    &rest[..end]
}

#[test]
fn p2s1_t1_solve_local存在性闸与json形状() {
    // T1 存在性闸：本测试名含 `solve_local` ⇒ `cargo test -p game-solver -- --list | grep -c solve_local ≥1`
    let r = solve_local(&SolveInput::placeholder(1));
    let j = r.to_json();

    // 逐字段类型断言（T6 形状的前身；字段序固定=确定性要求，§3.3 :257）
    assert_eq!(scalar(&j, "floor"), "1", "floor 为 int：{j}");
    assert_eq!(scalar(&j, "difficulty"), "0", "difficulty=int（BR-4 占位字段）：{j}");
    let reach = scalar(&j, "reachable");
    assert!(
        reach == "true" || reach == "false",
        "reachable 为 bool：{j}"
    );
    for key in ["max_hp", "min_kills"] {
        let v = scalar(&j, key);
        let ok = v == "null" || v.parse::<i32>().is_ok();
        assert!(ok, "{key} 为 int|null：{j}");
    }
    let conf = scalar(&j, "confidence");
    assert!(
        matches!(conf, "\"optimal\"" | "\"bounded-best\"" | "\"feasible\""),
        "confidence 三态枚举：{j}"
    );
    // route=[...]（末字段）：元素须为字符串、逗号分隔
    let ri = j.find("\"route\":[").unwrap_or_else(|| panic!("缺 route：{j}"));
    let rest = &j[ri + "\"route\":".len()..];
    assert!(rest.ends_with('}'), "route 是末字段、其后为右括号：{j}");
    let arr = &rest[..rest.len() - 1]; // 剥掉最外层 }
    assert!(arr.starts_with('[') && arr.ends_with(']'), "route 为数组：{j}");
    let inner = &arr[1..arr.len() - 1];
    if !inner.is_empty() {
        assert!(inner.starts_with('"') && inner.ends_with('"'), "route 元素为字符串：{j}");
        for part in inner.split("\",\"") {
            assert!(!part.is_empty(), "route 元素非空：{j}");
        }
    }
}
