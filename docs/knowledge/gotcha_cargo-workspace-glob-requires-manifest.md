---
name: cargo-workspace-glob-requires-manifest
description: crates/* 作为 workspace members glob 时每个被命中目录都必须有 Cargo.toml，放杂物目录直接构建失败
type: gotcha
created: 2026-10-09
sources: ["2026-10-09 t7 执行（08c2c97）"]
---

根 `Cargo.toml` 的 `members = ["crates/*"]` 会把 glob 命中的**每个目录**按 workspace 成员加载，
目录缺 `Cargo.toml` 直接报错（t7 实测复现）。

**为什么：** cargo 的 glob members 不过滤"看起来像 crate"的目录，任何子目录都被当作成员候选。

**何时使用：** ①`crates/` 下只准放 crate 目录（每个自带 manifest，即自动入 workspace）——
不放 `target/`、临时目录、文档目录；②构建产物统一在仓库根 `target/`；
③新 crate 落地流程 = 建目录 + 建 manifest，无需改根 Cargo.toml。
