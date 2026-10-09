---
name: agentteams-changedpaths-exact-match
description: agent_teams_update_task 的 changedPaths 只认 inScope 的精确串（无尾斜杠=精确匹配而非前缀），文件级路径被判 undeclared
type: gotcha
created: 2026-10-09
sources: ["2026-10-09 t1 执行（84ab54e）"]
---

`agent_teams_update_task` 的 `changedPaths` 校验按 inScope **精确串**匹配：inScope 元素
`"tools/rename-pipeline"`（无尾斜杠）**不是前缀**，成员提交 `tools/rename-pipeline/run.py`
会被判 undeclared；只有按 inScope 原样登记才通过（t1 实测）。

**为什么：** 契约范围是白名单语义，串匹配而非路径树匹配；同类问题也导致 t2 的根 `Cargo.toml`
无法登记（inScope 未声明）。

**何时使用：** ①写质量任务契约时，inScope 元素的粒度要与成员实际会登记的 changedPaths 粒度
对齐（要么目录级两边都目录级，要么列文件）；②成员报 undeclared 时先核对 inScope 原文再改登记；
③范围确需扩大的，终态任务不可改契约 ⇒ 建后续任务正式入账（本仓 t7 先例）。
