---
name: agent-edit-tool-keeps-mtime-cargo-stale
description: 代理的文件编辑工具改源码后 mtime 可能不变，cargo 判定无变化、沿用旧测试二进制——改完先 touch 再跑测试
type: gotcha
created: 2026-10-29
sources: ["2026-10-29 计划 1 Task 1：str_replace 新增测试后 cargo test 只跑出旧的 2 条测试（197 filtered），touch 后才出现 Compiling game-core 与第 3 条测试"]
---

代理用编辑工具（如 str_replace / fs_write）改 Rust 源码后，文件 mtime 有时不更新。cargo 按 mtime 判断是否重编译，于是直接运行**旧的测试二进制**：
新加的测试不出现、改过的实现不生效，RED / GREEN / 变异结论全部失真。

**为什么：** 2026-10-29 计划 1 Task 1 实测——str_replace 加了第 3 条 `bug047_*` 测试，`cargo test -p game-core --lib bug047` 仍只列出 2 条、
输出无 `Compiling game-core`；`touch crates/game-core/src/scene.rs` 后重跑才编译并出现第 3 条。实施子代理同日也遇到"加修复后第一次仍 FAILED、
再跑才过"的同一现象。`sed -i` 会更新 mtime，不受影响。

**怎么用：** 用编辑工具改完源码、跑测试前先 `touch <改过的文件>`；判读测试结果前确认输出里有 `Compiling <crate>`，或核对测试条数与预期一致。
变异测试每次改动 / 恢复后同样要 touch。给实施 / 复核子代理的提示词里写明这一条。
