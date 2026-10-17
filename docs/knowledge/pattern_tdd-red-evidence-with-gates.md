---
name: tdd-red-evidence-with-gates
description: pre-commit 总闸（gates check 含 cargo test）下 TDD 红阶段的合规做法——红证据三件+复现审计；禁 --no-verify、禁死文件假先行
type: pattern
created: 2026-10-18
sources: ["docs/findings/reviews/2026-10-17-platform-contract-v0.9-review.md §1 裁定", "docs/spec/platform-contract.md §7"]
---

TDD 单元的红阶段不进 git 历史：每单元 commit = 测试 + 实现（分文件或可剥离 hunk），
RED 证据三件入 commit 正文与记录表——(i) 失败输出逐字（用例名/编译错误）；
(ii) "RED 所跑测试代码与本 commit 测试逐字一致"声明；(iii) **RED 复现指令**
（把本 commit 的测试部分单独应用到父树运行即得红）。绝不 `--no-verify`。

**为什么：** pre-commit 总闸 `gates/cli.py check` 含 `cargo test --workspace --release`
（gates/cli.py:234）——红提交（含 trait 新增的编译红）过不了总闸，与 AGENTS §5
"不得绕过/弱化/删除门禁"互斥；死文件假先行（测试不挂 mod）= 测试从未真正红过，
还引入静默跳过风险（gotcha_nested-test-silently-skipped 同类）与半成品悬挂。
复现指令让审查者事后可重放红，审计不依赖信任。

**何时使用：** 任何"测试先行"的代码单元（trait/修复/移植）；实现审查时按 (iii)
抽查 ≥1 单元复现红，复现不出即驳回。
