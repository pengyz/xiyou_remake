---
name: oracle-png-channel-order
description: oracle 帧 PNG 的 RGBA 四通道实存 (A,R,G,B)——PIL convert('RGB') 对比会得出 ~99% 假差异
type: gotcha
created: 2026-10-11
sources: [迭代25 T351 帧对比（99.2% 假残差 → 通道修正后 9.2% 真残差）]
---

oracle 宿主把 ARGB int 的 4 字节直接写进 PNG 的 RGBA 通道槽，即 PNG 的
R=alpha、G=R、B=G、（丢弃的）A=B。任何用 `PIL.Image.open(...).convert('RGB')`
取 (R,G,B) 的对比脚本，实际拿到的是 (255, R, G)，会把完全一致的两帧判成 ~99% 像素不符。

**为什么：** Java 侧写 PNG 用的是 `int[] ARGB` 直出（TickHooks.afterPaint），
未做 ARGB→RGBA 通道重排；帧哈希（hash_stream）不受影响，只有**人看的 PNG** 带这个坑。

**何时使用：** 一切以 frame-*.png 为对象的像素级对比/采样脚本。正确读法：
`img = open(...).convert('RGBA'); c0,c1,c2,c3 = px[x,y]; 真RGB = (c1,c2,c3)`。
两侧都来自 oracle PNG 时对称无影响；**Rust 帧 vs Java PNG** 对比时必须修正通道序。

> 【2026-10-11 更正】写侧已修复：PngWriter.java 直写 ARGB 序是病根（用户报告
> "Java 渲染色偏、Rust 没有"），已改为 R,G,B,A 正序；复跑 trace 逐字节一致
> （比对权威是 hash_stream，PNG 字节不参与）。修复前生成的历史 PNG 归档仍带
> 错序，读旧帧仍需本文换算；新 trace 的 PNG 可直接 convert('RGB') 使用。
