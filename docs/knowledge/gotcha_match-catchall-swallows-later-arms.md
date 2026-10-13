---
name: match-catchall-swallows-later-arms
description: Java switch→Rust match 移植插新臂时用 `_ => {}` 兜底会吞掉其后的全部原有臂——必须用显式类型码
type: gotcha
created: 2026-10-13
sources: [迭代65 floor35 T301（83 臂失联→怪物臂误触发）+ preset_floor10 全帧收敛]
---

把 Java `switch` 的 case 语义插进 Rust `match` 链中间时，若新臂写成
`_ => {}`（本意"case 5 无语义、放行"），会**静默吞掉位于其后的全部原有
臂**——Rust match 首中即停，被吞臂（如 83 剧情点/9 炼丹台/77 商人）永不再
命中，且编译器只给 unreachable-pattern 警告不报错。

现象（迭代 65 floor35 T301 实证）：83 臂失联 → bump 落到怪物臂 → 战斗
误触发；门控行为整体错位。

**为什么：** Java switch 按 case 值精确分发、无"插入顺序"概念；Rust match
按**源码顺序**首中——`_` 是全捕获，位置即语义。

**何时使用：** 任何"在既有 match 链中间补 case"的移植——新臂一律写显式
匹配值（`5 => {}` 并注释"不可写 `_`"）；插完跑 `cargo build` 确认无
unreachable 警告，再跑覆盖该 match 的差分用例。
