---
name: commit-claims-must-match-content
description: 同一批工具调用里 edit 失败后 bash commit 照跑，导致 commit message 声称了未落地的变更
type: gotcha
created: 2026-10-09
sources: ["2026-10-09 P1 整合会话（413a697）"]
---

同一 assistant 消息里"edit 文件 + bash 提交"打包执行时，若 edit 因 FS_STALE_VERSION 失败，
bash 提交仍会执行 ⇒ commit message 声称的变更**不在提交里**（实测 413a697 声称"bug 台账登记"
但 bug-ledger.md 未变）。

**为什么：** 工具调用互相独立，edit 失败不会中止后续 bash；而 message 是提前写好的，
不会自动校正。这类"声称-内容不一致"直接违反证据纪律，且 push 后难以干净补正（不许改写已推历史）。

**何时使用：** ①commit message 里声称的每一项变更，必须在 `git show --stat` 里可逐项核对；
②edit 与 commit 同批执行时，commit 前先确认 edit 成功（或改用先 edit、后单独 commit 的顺序）；
③发现不一致 → 立即追加补正 commit 说明差异，不改写历史。
