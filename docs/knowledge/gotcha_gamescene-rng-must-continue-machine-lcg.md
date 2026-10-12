---
name: gamescene-rng-must-continue-machine-lcg
description: GameScene 构造默认 new_seeded(0) 不是 Java 语义——machine 阶段已消费的 LCG 位置必须续接，否则首个场景侧消费者（战斗抖动）序列错位
type: gotcha
created: 2026-10-12
sources: [迭代35 preset_cases floor_crossing T801（A-floor1-to-floor2 重生 + 帧像素取证）]
---

Java 侧 `gameRandom` 是 **a.java:2324 的全局单例**：boot 时 `setSeed(VTime=0)`
之后标题雪花、菜单动画、战斗抖动（a.java:6559-6560 `randomBelow(5)±2px`）
全部从**同一条 LCG 消费链**连续取数，跨模式/换层/读档**不重置**。
Rust 侧 `GameScene.rng` 构造默认 `JavaRandom::new_seeded(0)`（scene.rs）只是
占位——preset harness 此前未把 `machine.rng` 续接进场景，导致场景首个
消费者（战斗目标 state==3 抖动）从序列位置 0 取数而 Java 在位置 ~K。

**为什么：** 不产生像素分歧的消费（标题雪花在 machine 侧各算各的）会一路
绿灯，直到首个**场景侧**消费者出现才爆炸——本例是首次纳入全帧对拍的战斗流
（T801 ±2px 抖动错位，帧差 632px、同轮廓异色）。旧冷场景 harness
（gameplay_floor1.rs build_scene）甚至留有 `random_below(240)/(150)` 手工
预消费 hack 佐证同一错位。这类错位**不查 rng 语义永远修不进主因**。

**何时使用：** 任何构造 `GameScene` 的新调用方（测试/未来桌面壳）必须在
handoff 处 `sc.rng = machine.rng.clone()`；排查"状态全对但个别精灵像素错位"
类分歧时，优先核对 `java.util.Random` 单例连续性（`#[derive(Clone)]` 就是
为此保留的）。修复后 preset 全套 8/8 PASS（floor_crossing T1-1500 全帧）。
