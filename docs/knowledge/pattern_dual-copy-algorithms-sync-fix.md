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
任何一处修 bug、调宽度、改转义语义都必须问"另一份呢"。拷贝间还会长出语义分叉
（2026-10-15 实证：m_113 双拷贝装备槽一画图一画名、防具钳位两处皆错；
paintWrappedText 双拷贝 `\r` 恢复色范围一为行内一为跨 paint；layout 三拷贝
非法 `\c` 记账各不相同）——分叉落在测试盲区时双绿掩盖。

**何时使用：** ① 修文本布局/转义类 bug 时，`grep -n "fn layout" crates/game-core/src/scene.rs`
先盘点全部拷贝；② 长期解 = 抽公共函数（limit 作参数），遗留重构项记在
docs/spec/completeness.md；③ code review 涉及布局函数时强制检查姊妹函数。
> 【2026-10-15 更新 · 2026-10-17 合并整合后修订】长期解大半落地：m_113 族
> （`SlotSelect::paint_list`/`paint_equipment_frames`/`paint_detail_stats` 双宿主共用）
> 与 layoutWrappedText 族（`layout_wrapped_text_limited` 唯一实现 + `layout_full_text`
> 委托）已单源化——两族各有并行会话与本会话殊途同归的两版实现，2026-10-17 rebase
> 整合取并行会话落地版。**遗留：paintWrappedText 绘制双拷贝**
> （`OverlayEngine::paint` / `scene::paint_wrapped_text`）尚未合一，修一处必须查姊妹。
> 新代码禁止再复制算法体；合并语义分叉以 Java 原文逐条裁决（缺陷原样保真）。
