---
name: layout-from-device-formulas
description: 壳层动态布局禁设备特化常量——几何 = 设备输入（窗口/dpi/contentRect）+ 游戏数据（maplv 头/layout 常量/字高）+ 平台规范常量（48dp/16dp/32dp/12sp）→ 公式；设计取值只许 dp 令牌或无量纲比例并集中列表；测试钉算例 + 设备矩阵不变量
type: pattern
created: 2026-10-26
sources: ["2026-10-26 会话：竖屏首版 portrait.rs 用 12 列/30%/80px/字高 14 等魔数，用户质询'手机分辨率变了，你这些数字还准确吗？我们需要计算公式'"]
---

动态布局里每个数都必须能回答"从哪来"：要么是**设备输入**（`ANativeWindow` 尺寸、
`AConfiguration_getDensity`、glue `contentRect` 内缩），要么是**游戏数据**（全部 `maplv` 头 `>>1`
的最大格数、`layout::{SCREEN_W/H, CELL_PX, STATUS_BAR_H, HUD_FRAME_H, SOFTKEY_ICON_*}`、
`paint_font().height`），要么是**有出处的平台规范**（触控目标 48dp、边距 16dp、AOSP 手势区
`navigation_bar_gesture_height` 32dp、M3 Body Small 12sp / Label Large 14sp、按钮最小宽 64dp）。
剩下的审美取值只许用 dp 令牌或无量纲比例（十字键 3×3 格、确定钮 2 格、上限 192dp、描边 2dp），
集中成具名常量表。实现见 `shell/android/src/portrait.rs` 模块文档的公式表。

**为什么：** 首版 `MIN_SCENE_COLS=12` 恰是 1200px÷96 的 Xiaomi 14 解，`80px` 手势条、`/3` 标签倍率、
`4/5/6px` 描边同理——换分辨率或"显示大小"（dpi 变）即失准，且测试钉的也是这些数，
等于把设备特化钉成正确。px 字面量随密度失真，dp/比例才与分辨率无关。

**何时使用：** 写或审任何随设备变化的几何（壳层画布倍率/格数、触控面板、信箱、HUD 外挂件）。
测试分两层：① 一台真机的逐步推导算例，期望值为手算字面量（F-1）；② 设备矩阵（含同机改 dpi、
平板、折叠屏、窄屏）钉不变量——48dp 目标、面板下限、原分辨率下限、满图上限、整数倍极大性、可读字号。
设备事实先只读取证（`adb shell wm density`、`dumpsys window` 的 InsetsSource）再定常量：
Xiaomi 14 竖屏全屏窗 1200×2536 自设备 y=134 起、contentRect 底 2488（导航条 48px=16dp）。
