---
name: no-verify-never
description: --no-verify 永禁与 git add 失败必验退出码——绕闸补救的错误与正确补救路径（2026-10-18 实例）
type: gotcha
created: 2026-10-18
sources: ["2026-10-18 shell-port 自首（DirAssetSource 合并 73c9d1b → --no-verify amend）"]
---

# --no-verify 永禁；git add 失败必验退出码

**病灶一：`git add` 撞锁静默失败**——共享工作区并发提交时 `git add` 撞 index.lock
可能静默失败（退出码非 0），pathspec commit 不含未跟踪文件 ⇒ 提交内容残缺
（干净 checkout 编译失败）。**纪律：每次 git add 必验退出码**（守卫脚本断言
`ADD_EXIT=0`）；提交前 `git diff --cached --stat` 三对账。

**病灶二：急躁补救绕闸**——发现残缺后用 `--no-verify` amend "快速修复" =
**永久禁止**（TDD A 强化协议明令）。即便内容与闸验树零差异，程序上绕闸属实，
且一旦形成肌肉记忆必在真红时放行坏提交。

**正确补救路径**（本次采用）：
1. 残缺补全后**同内容、不带 --no-verify** 重新提交/重落——让 pre-commit 总闸真实执行；
2. commit message 加"纪律更正"行（审计线索完整）；
3. 自首 + 内容零差异 + 即时补救 = 不回滚；**再犯升级**（第二次暂停任务流转）。

**同族**：gotcha_shared-workspace-bare-commit-sweeps-index（裸 commit 卷索引）——
共享工作区的 git 纪律三件套：pathspec 提交、add 验退出码、闸永不绕。
