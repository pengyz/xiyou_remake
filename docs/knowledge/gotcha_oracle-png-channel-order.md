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
