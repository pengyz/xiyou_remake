---
name: parallel-commit-shared-index
description: 多 teammate 并行 git 提交的共享 index 竞态与共享闸 TDD 张力——--only 显式路径+工作树保编译+红证据进 message/commit 本体绿
type: gotcha
created: 2026-10-22
sources: [208bcf1, 2da157d]
---

多 teammate 并行工作时三类反复踩的坑（2026-10-21/22 会话各实锤）：

1. **共享 git index 竞态**：`git add`+`git commit` 两步之间别人也在 add ⇒
   提交混入他人文件（208bcf1 误收 hd-demo 4 文件）。**药方：`git commit --only
   <显式路径> -- <msg>`**。注意 `--only` **不收未跟踪文件**——新文件先
   `git add <file>` 再 `--only`（一次踩坑记录）。
2. **index.lock 阻塞**：他人 commit 在 pre-commit 闸跑数分钟会持锁——**不删锁**
   （会毁在途提交），等锁清或让对方先落。
3. **共享闸 TDD 张力**：pre-commit 跑全量测试 ⇒ 任何人的半成品 WIP 编译红会堵死
   全员提交。**药方：工作树保持可编译**（空挂载起步）；红证据（红测试代码+红输出）
   进 commit message 留痕，**commit 本体必须绿**；逐类小绿 commit 缩短阻塞窗口。

另：markdown 链接在引用块内**跨行截断会被 docs-links 闸判链**（`](` 与目标同行），
同会话踩两次——写文档时链接目标勿换行。
