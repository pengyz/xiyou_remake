---
name: wrong-index-false-green
description: 从反编译源码目测推导数组索引 + 弱断言（只数变化次数/从不 assert 对拍计数>0）会产生连续多批全绿的假绿测试——必须用 oracle FLD 运行时 dump 逐格复算收口
type: gotcha
created: 2026-10-09
sources: ["2026-10-09 P3.5 gameplay-floor1 全帧对拍会话", "docs/spec/p3-render.md#3-6"]
---

walk_pure 行走重放测试带着**错误的可行走网格索引**（`(y*wide+x)*2`，行距 2·wide）连续多批全绿；改用权威索引（rebuildWalkability 实为行距 4·wide、列距 2，oracle FLD 134 逐格复算）后立刻红——旧测试从未真正对拍过 Java 行为。

**为什么：**
1. 网格索引是从反编译源码目测推导的（B 级证据）：`for (…; var1 += var3 << 1)` 的行增量与列循环 `var1 += 2` **叠加**后实际行距是 4·wide——目测时看漏了列循环对游标变量的消耗。没有运行时 dump 复算就落了测试。
2. 测试断言弱：`checked` 只在坐标与 trace 相等时自增但**从不 assert checked > 0**；位移数断言 `>= 8` 只要玩家动起来就过。错误网格恰好放行了被 Java 阻挡的上行步，玩家"动了"，断言全过。
3. 场景注释（walk-pure.txt "向上走廊已知可通行"）本身是错误理解的产物，又反向强化了信心——注释不能当证据。

**何时使用：** 从反编译源码推导出任何数组索引/算式后，先用 oracle FLD 运行时 dump（StateDump 可打印数组全量或 head）逐格复算再写进解析器/测试（A 级收口）；写对拍测试时禁止"只数变化次数"的弱断言，必须锚定权威时刻的权威值（零位移也可以是强断言）；同一条数据被两个消费方用不同索引读取（paintTileLayer 行距 2·wide/列距 1 vs rebuildWalkability 行距 4·wide/列距 2）在原版真实存在，不要用"应该一致"的直觉去修正任何一侧。
