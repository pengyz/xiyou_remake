---
name: op-stream-diff-convergence
description: 帧残差收敛首选 op 逐行对拍——Rust g.ops 前 256 条 vs trace OPS 块 diff，一次定位全部移植笔误
type: debug
created: 2026-10-11
sources: [迭代25 preset_item13 T351/352 收敛（paint_menu5 九处笔误）]
---

帧 sha 不符时，直接对拍绘制 op 流而非像素：测试里 `g.ops.join("\n")` 落盘，
与 Java trace 的 `OPS` 块逐行 `diff`，首个分歧行即根因坐标。
注意 shim `Graphics.flushOps`（Graphics.java:53）**块内上限 256 条**，
头部 `OPS count=N` 的 N 是全量计数——可对拍窗口只有前 256 条；超出部分只能靠帧级区域 diff 兜底。

**为什么：** 像素 diff 只给"哪里不一样"，op diff 给"哪一条绘制命令的哪个参数不一样"。
实测 preset_item13 T351 残差 7077px，帧级对比反复无法定位；op 对拍一次暴露
paint_menu5 九处移植笔误（+45 误代 +40、x 基准 +3 误代 +15、y+=2 缩进错位、
精灵索引 [1] 误代 battle_icon、分隔线 -1 误代 -22、行首 x 未按 Java for-update 重置等）。

**何时使用：** 任何帧级残差收敛；先 op diff（定位快），op 窗口外的残差再用
帧级区域聚类（16px 块统计）补充。前提是两侧 op 字符串格式一致——
Rust `render.rs` 的 op 格式串必须与 shim `Graphics.op()` 逐字符对齐（含逗号后无空格）。
