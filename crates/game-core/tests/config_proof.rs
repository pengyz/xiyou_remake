//! STAGE-B-002 合理性实证（用户提问 2026-10-18）："水平加一格是否改配置就能用？"
//! 答=是：Layout 公式面响应 grid 参数，零代码改动。本测=对照实证（API=字段面）。

use game_core::layout::Layout;

#[test]
fn config_only_grid_change_proof() {
    let base = Layout::modern(13, 13);
    let plus_w = Layout::modern(14, 13); // 水平加一格——仅参数不同
    let plus_h = Layout::modern(13, 14); // 垂直加一格

    // ① 水平+1 格 ⇒ 屏宽增加一个格宽（纯公式响应，零代码改动）
    let cell_w = plus_w.screen_w - base.screen_w;
    assert!(cell_w > 0, "水平+1 格 ⇒ 屏宽增大（格宽={cell_w}）");
    assert_eq!(plus_w.screen_h, base.screen_h, "水平变化不动高");

    // ② 垂直+1 格 ⇒ 屏高增加一个格高
    let cell_h = plus_h.screen_h - base.screen_h;
    assert!(cell_h > 0, "垂直+1 格 ⇒ 屏高增大（格高={cell_h}）");
    assert_eq!(plus_h.screen_w, base.screen_w, "垂直变化不动宽");

    // ③ UI 锚随屏公式移动（贴底基线=字段，随屏高平移）
    assert_eq!(
        plus_h.hud_base_y - base.hud_base_y,
        cell_h,
        "HUD 贴底基线随屏高公式平移（非常量）"
    );

    // ④ 最小钳不变式：过小 grid 钳回原分辨率（classic 下限）
    let tiny = Layout::modern(2, 2);
    let classic = Layout::classic();
    assert_eq!(tiny.screen_w, classic.screen_w, "最小钳=原分辨率（宽）");
    assert_eq!(tiny.screen_h, classic.screen_h, "最小钳=原分辨率（高）");
}
