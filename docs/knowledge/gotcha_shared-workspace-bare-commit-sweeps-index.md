---
name: shared-workspace-bare-commit-sweeps-index
description: 多 agent 共享工作区里裸 git commit 会把队友暂存文件一并提交——commit 必须带 pathspec（git commit -- <路径>），禁裸 commit / add -A
type: gotcha
created: 2026-10-18
sources: ["117aba7 索引竞态事故（shell-port 行为锚 commit 误卷 platform-contract Unit 2 文件）", "ba14d4e/41b5c93 G1 同族（git add -A 卷入 save/*.bin）"]
---

Agent Teams 并行开发时裸 `git commit` 提交的是**整个索引**，会把其他成员
`git add` 的暂存文件连带提交，造成 commit 声称/内容/写范围三不符。

**为什么：** git 的提交单位是索引不是"我的文件"；共享工作区里索引是多方共用的
暂存区。本仓已两起：ba14d4e（`git add -A` 卷入用户存档二进制，41b5c93 前向更正）、
117aba7（行为锚 commit 误卷队友 Unit 2 文件，未推送守卫式拆分）。后果是
gotcha_commit-claims-must-match-content 的复发机制——声称与内容不符不是笔误，
是提交面失控。

**何时使用：** 任何成员任何提交。规则（lead 2026-10-18 立）：**pathspec 提交强制**——
`git commit -- <路径>…`，禁裸 `git commit`、禁 `git add -A`；提交前
`git diff --cached --stat` 自查提交面 = 声称面 = 写范围三对账；事故后拆分用
未推送 soft-reset + pathspec 部分提交，并留审计留痕。

> 【2026-10-18 并入】同族并发坑两枚（多 agent 共享工作区，防另立条目漂移）：
> ①**index.lock 撞锁**——并发 `git add/commit` 报 `Unable to create .git/index.lock`；
> 活锁（有存活 git 进程，多为队友 commit 正跑 10 分钟级 pre-commit 总闸）**禁清**，
> 等锁主完事或延迟重试；确认无存活 git 进程（`Get-Process git` 空）才是死锁，
> 按 git 报错指引手删。②**LNK1104 无法打开 exe**——并发 `cargo` 同 target 目录，
> 链接器撞上队友测试产物文件锁，是瞬态竞争不是代码错，重试即过；不要当编译失败
> 反复改代码。取证免疫法（绕开共享工作区/索引/mtime 三坑）见
> `gotcha_cargo-mtime-stale-rlib-false-red.md` 的 `git archive` temp 复现法。

> 【2026-10-18 补·索引扫荡三机制齐（G6 案实证）】第三机制：`git commit --amend`
> （无 pathspec）同样**索引全扫**（卷入 lead 集成件 6 件）。三机制 = `git add -A`
> （G1）/ 裸 `git commit`（117aba7）/ `git commit --amend`（G6）。收敛解：
> `git reset --mixed <父>` + `git add <我的路径>`（**验退出码**——撞锁静默失败即提交
> 残缺，G6 首因）+ `git commit -- <路径>`。两条结构性防线：**总闸验证工作树而非
> 提交面**（过闸 ≠ 提交内容一致，提交后必 `git show --stat` 三对账）；**amend 后必须
> 重过闸**（`--no-verify` 零容忍，自首亦记违规）。
