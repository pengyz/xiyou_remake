---
name: folded-screen-literals-escape-layout-conversion
description: Modern 换轨时 SCREEN_W/H 符号替换抓不到编译期折叠的屏幕字面量（275=320−45、148=160−12、215=240−25、97=行x+35…）——同块兄弟臂已换、单臂漏网（BUG-029/030/031 共 7 处）；按"兄弟臂形式"判屏幕派生 + 1px 覆盖探针证 classic 回代
type: gotcha
created: 2026-10-27
sources: ["2c7ee10 BUG-029", "17a821f BUG-030", "f4af91e BUG-031", "7aad584 BUG-032"]
---

STAGE-B-002 把绘制坐标换成 layout 公式时，替换是按符号做的（`SCREEN_H − h` → `self.layout.screen_h − h`），
但反编译出的 Java 有大量**编译期折叠**的屏幕派生字面量，看上去不像屏幕尺寸：a.java:2852 牛魔王立绘 y `275`
（= 320 − 45，同块另两臂写作 `320 − h`）、观察态箭头 `148/109/215`（= 160−12 / 120−11 / 240−25，同块下臂是
`viewBottom − 29`）、炼丹图标 `97/142`（= 行 x 62 + 35 / + 80）、金币弹字 `240`（= 金币列 237 + 3）。它们留在
classic 位，同组元素却已移走 ⇒ Modern 下单个元素错位（立绘飘在地图中、弹字落在钥匙格下）。组合关系测
`bottom_composition_is_translate_only` 只枚举 speaker 0/1，漏 72 特例臂，门禁拦不住。

**为什么：** 字面量本身不带"我是屏幕派生"的标记；判据只能来自**同一 Java 块的兄弟臂**：兄弟臂若已是
`240 − … >> 1` / `320 − …` / viewBottom 变量形式，漏网值就是折叠后的同类量。classic 回代零偏离掩护了漏改——
classic 下字面量恰好正确，帧夹具全绿。另：扫描时顺带撞出 classic 移植偏差（BUG-032 运算符优先级
`var46 += fh + 4 − 13 >> 1` 误译、变换码表下标直传），这类只在"该行从未被帧锚覆盖"时存活。

**何时使用：** 改/审任何随布局变化的绘制代码时：① 逐处对照 a.java 同块兄弟臂形式判屏幕派生，换成 classic
回代 = Java 字面量的公式；② 组合关系测要枚举**全部特例臂**（说话者 72、kind 4 等），不能只测"代表值"；
③ **覆盖探针**证 classic：修复点临时偏 1 px，跑 game-oracle，看哪些 Java 锚测转红（转红 = 该元素确被帧锚覆盖、
公式运行期回代 Java 值；无一转红 = 未覆盖，须补 Java 夹具，如 fx-record-arrows 改存档日志条数让箭头真画）；
④ 实机管线无头核对：`shell/android/tests/headless_portrait_shots.rs` 的 `#[ignore]` 工具按 Xiaomi 14 输入整窗截帧。
