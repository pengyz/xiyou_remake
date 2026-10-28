---
name: title-menu-labels-come-from-art-strip
description: 标题菜单项文字来自美术条 menu[1]（按 17×kind 取行），不是 menuLabels 文本表；把两者混为一谈会"发现"不存在的文字-动作错位（BUG-007 误诊，2026-10-28 证伪）
type: gotcha
created: 2026-10-28
sources: [docs/bug-ledger.md BUG-007, docs/reviews/2026-10-28-settings-controls-design-adversarial-review.md §5]
---

标题菜单（mode 1）显示的项名是图片：`drawIconStrip(f_Image_arr2_00[10][1], …, 17 * menuItemKinds[cursor])`（a.java:2359-2368；
Rust title.rs:470/495 同式）。menu 容器 [1] 为 68×102 的 6 行美术字，依次是「开始游戏 / 继续游戏 / 设置 / 帮助 / 关于 / 退出」，
与 kind 0..5 → mode 14/8/16/15/17/22 一一对应。`menuLabels`（scene.rs `MENU_LABELS`）只给游戏内菜单（mode 4）画文字用，
其中 kind 3 =「保存游戏」。

**为什么：** 两张菜单共用 kind 编号，但标签来源不同。只看 trace 里的分派结果、再拿 menuLabels 去对，就会得出"保存游戏显示帮助页"这类
错位结论；BUG-007 因此以 A 级行为证据挂了 12 天，2026-10-28 对抗复核者又沿用它攻击设计，解包美术条目视才推翻。

**何时使用：** 判断任何"文字与动作不符"类问题前，先确认画面上的字从哪来（美术条 / 文本表 / 字体绘制），并解包目视或比对像素；
引用 BUG-007 旧结论的文档一律以 2026-10-28 更正为准。
