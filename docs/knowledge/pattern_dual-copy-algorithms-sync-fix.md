---
name: dual-copy-algorithms-sync-fix
description: 同一算法存在"全量/限量"双拷贝时（layout_full_text / layout_wrapped_text_limited），修复必须同步——漏改一份 = 偏差换个入口复发
type: pattern
created: 2026-10-12
sources: [迭代 26 修 overlay 全量版 \c 转义，迭代 30 发现打字机限量版同 bug 复发（preset_floor51 T372 44px）]
---

`layout_full_text`（overlay 全量）与 `layout_wrapped_text_limited`（对话框打字机
限量）是 Java `layoutWrappedText` 同一算法的两份 Rust 拷贝。迭代 26 修复了全量版
的 `\c` 转义处理（8 字符消费/0 宽/hex 校验），限量版漏改——floor51 的对话框
（走限量版）在 T372 复现同族偏差（尾 hex 计宽 6px ⇒ 换行点早 1 字符）。

**为什么：** 两份拷贝语义应恒等（Java 侧就是一个函数，第 4 参 limit 区分）；
任何一处修 bug、调宽度、改转义语义都必须问"另一份呢"。

**何时使用：** ① 修文本布局/转义类 bug 时，`grep -n "fn layout" crates/game-core/src/scene.rs`
先盘点全部拷贝；② 长期解 = 抽公共函数（limit 作参数），遗留重构项记在
docs/spec/completeness.md；③ code review 涉及布局函数时强制检查姊妹函数。
