---
name: pointer-menu-stage-b
description: 鼠标菜单操作（指针点击行）= Stage B 加法——用户裁定放行的纯加法入口与实现三硬条件
type: decision
created: 2026-10-18
sources: ["2026-10-18 用户需求（Windows 操作适配三件）", "adversary 方案审查 2026-10-18-win-mouse-esc-plan-review.md"]
---

# 决策：鼠标菜单操作（指针行点击）= Stage B 行为加法（用户裁定）

- **定性**：原版 a.java 无 pointerPressed/Released/Dragged（deobf 全目录 grep 零命中，
  A 级）⇒ 指针输入非移植保真项。ESC→-7 与滚轮/点击合成键码属**平台输入映射**
  （与 android 触控同类，Stage A 可容）；**直接点行选择 + hover 选中高亮**属 **Stage B 行为加法**
  （与音频焦点案 F-6 同类），因**用户明确需求**（产品裁定先例）+ 纯加法入口放行。
- **边界（实现三硬条件，adversary 裁定）**：
  ① 几何单源：pointer_hit 行几何复用 paint 布局同一来源（SlotSelect content_*/行距、
     item panel 布局字段），禁自推常量（BUG-008 字高派生量前科适用）；
  ② Row 语义定死：绝对行/可见行+scroll_top 映射逐态对齐 tick 光标语义；
  ③ 逐态 handler 指认：标题确认=-5/53、游戏内菜单激活=**-6**、道具使用=-6——
     禁"统一 -5"措辞（键码契约 §4 同码不同动作）。
- **不变量**：handle_pointer 只做"移动光标+触发既有确认臂"，零新游戏状态；
  对话态（mode11）指针返回 None 不点走剧情；行为只增不减（全部既有差分/golden
  测试零漂移为证明面）。
