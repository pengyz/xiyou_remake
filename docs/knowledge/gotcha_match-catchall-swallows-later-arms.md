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

> 【2026-10-15 补充】另一形态：`_ => {}` 不在链中间、而在末尾当「后续批次」占位
> （scene.rs interact_with_cell cat1 `_ => {} // 11 / 76/82：后续批次`）——不吞已有臂，但让**未移植臂静默放行**：
> 无 todo! 标记 ⇒ 覆盖矩阵 §5 的 todo! 扫描看不见；无场景撞击 ⇒ 帧对拍恒绿。型 10/11/12 撞击臂因此缺失多个迭代，
> 直到为 m_089 铺夹具场景时撞火首帧分歧（fixture_frames::fx_firebump_floor13，首分歧 T301）。对策：占位臂一律
> `t => todo!("… {t}")`（显式挡路、可机械扫描）；已存在的静默占位登记到 tools/cover_matrix.py `ARM_GAPS`（矩阵 §5.1）。
