---
name: layout-derived-sentinel-ui-stall
description: 原版"像素计时哨兵"（如序章快进 scroll_x=240）在 Modern 宽画幅下小于行宽 ⇒ 按键反把进度拉回、连按永停；且序章/字幕排版在 paint 内完成，只 tick 不 paint 的宿主驱动永不推进
type: gotcha
created: 2026-10-26
sources: ["Android 实机反馈：选开始游戏后序章点了不动、进不了游戏", "intro.rs key_mash_fast_forward_completes_intro_in_both_presets 红→绿", "shell/android/tests/flow.rs 只 step 不 paint 致假红"]
---

原版把序章按键快进写成 `introScrollX = 240`（px 计时哨兵，classic 行宽 ≤220 ⇒ 一按即越过
`行宽+10` 门槛翻页）。Modern 字幕框宽=屏宽−20（576 画幅 556）⇒ 行宽 ~540：每次按键反而把进度
**拉回** 240，连按=永停本行（实机"序章点了不动"）。修=快进到行尾 `max(240, 行宽+10)`，classic 恒 240。

**为什么：** 哨兵常量与 classic 排版宽隐式耦合（C 类语义，代码注释已标"不派生 SCREEN_W"），
Modern 改了排版宽却没改哨兵。另一半坑：`runIntroViewer` 的滚动机读**上次 paint** 的布局锚
（`overlay.layout_done()`），宿主测试若只 `step` 不 `paint`，序章永远不推进——会误判为逻辑 bug。

**何时使用：** Modern 下任何"按键推进/翻页/滚动"类 UI 卡住时，先查原版是否有与 240/220 宽绑定的
计时哨兵；写宿主流程钉时每拍必须 step+paint（与设备主循环同构，见 flow.rs `drive`）。
Modern 帧基线 intro/game 两态会随快进修复漂移（快进更早翻页 ⇒ 序章更早结束），属预期。
