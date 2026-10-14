---
name: survey-repo-state-before-research
description: 多会话推进的项目接任务先盘点 git log+knowledge，否则会重复调研已被解决的问题（实测：反编译引擎早已定案）
type: pattern
created: 2026-10-14
sources: ["2026-10-14 用户两连纠偏（你在干什么/指令是调研项目最新状态）"]
---

接手任务的第一步是**盘点仓库最新状态**（`git log` 自上次在场点 + `docs/knowledge/MEMORY.md`
+ `docs/spec/` 三账本），而不是直接对任务字面开工——本项目由多会话推进，几天内可长出
数百提交，很多"待调研问题"可能已被解决并留有 decision 条目。

**为什么：** 实测教训：用户要求"调研业界反编译方案"时，项目 180 提交前就已完成
CFR→Vineflower 引擎切换 + 可编译 + A==B==C 三方对拍（`decision_decompiler-cfr-to-vineflower`），
调研是重复劳动且延误了真正指令（项目状态盘点）。

**何时使用：** ①会话间隔后开工前；②任务涉及"调研/选型/方案"类问题时先查 knowledge
的 decision 条目是否已有结论；③发现"要调研的答案已在仓库"时，直接引用+更正，
并把时间投到真正未解的部分。
