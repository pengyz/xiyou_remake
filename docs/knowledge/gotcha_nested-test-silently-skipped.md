---
name: nested-test-silently-skipped
description: #[test] fn 被误嵌套进另一函数体内时 libtest 静默跳过——cargo test 照样绿，假绿可存活多个迭代
type: gotcha
created: 2026-10-11
sources: [迭代26 preset_help（82ef6c0 嵌套提交，迭代 23"首跑 PASS"实为从未执行）]
---

测试函数缩进粘进上一个函数体尾部（上一个 `}` 后漏顶格）时，`#[test]` 变成
普通嵌套 item：**编译通过、`cargo test` 列表里不出现、不报错、不告警**。
该用例声称的"首跑 PASS"无法由该嵌套函数产生（嵌套函数跑 0 个测试）。

**为什么：** libtest 只发现顶层 `#[test]`；rustc 对未使用的嵌套 fn 顶多发
dead_code warning，测试文件里极易被忽略。preset_help 自 82ef6c0 起三个迭代
从未真正执行，直到提顶层后立刻暴露 3 处真实误植（overlayActive 漏置位/
\c 转义宽度/-6 关层语义）。

**何时使用：** 任何"测试通过"结论先核对 `cargo test <name>` 输出里**确实
列出了该测试名且 ok**（`0 filtered`）；review 测试代码时检查每个 `#[test]`
顶格。防御性命令：`cargo test -- --list | grep <name>`。
