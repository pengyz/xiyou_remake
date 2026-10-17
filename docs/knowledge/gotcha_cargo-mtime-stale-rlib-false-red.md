---
name: cargo-mtime-stale-rlib-false-red
description: 还原源文件用 Copy-Item/git checkout 会带回旧 mtime ⇒ cargo 按指纹跳过重编、链旧 rlib，报"no X in the root"假红；touch 时间戳即复绿
type: gotcha
created: 2026-10-18
sources: ["2026-10-18 platform-contract Unit 2 RED 复现取证现场（git checkout 父树 + Copy-Item 还原）", "adversary 复审 temp-archive 复现法（docs/findings/reviews/2026-10-18-platform-contract-implementation-review.md）"]
---

TDD 取证做"父树复现红"时，若用 `git checkout <父sha> -- src/lib.rs` 换文件、再
`Copy-Item` 备份还原，还原文件带着**旧 LastWriteTime** ⇒ cargo 指纹判定"源未变"，
跳过 lib 重编、把**父版 rlib** 链给测试，报 `error[E0432]: … no X in the root`——
而 `cargo build --lib` 显示 Finished（同样链缓存）：两相矛盾的假红。

**为什么：** cargo 的 mtime 指纹是"源比产物新才重编"；Copy-Item 保留源时间戳、
`git checkout` 写出的是当时时刻，一来一回 mtime 倒退到产物之前。测试目标重编了、
lib 没重编 ⇒ 新测试链旧库。肉眼看文件内容全对（grep/read 都有符号），rustc 却说没有。

**何时使用：** 任何"换文件-跑-还原"的取证/回滚流程后。防法三选一：
①`git archive <sha>` 导出到 temp 目录跑（字节拷贝不碰共享工作区/索引，**推荐取证法**，
  天然免疫 mtime 与共享索引两类坑——adversary 复审实证）；
②还原后 `(Get-Item <file>).LastWriteTime = Get-Date` 触碰时间戳；
③`cargo clean -p <crate>` 强制重编。
