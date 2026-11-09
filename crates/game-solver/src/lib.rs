//! game-solver：确定性求解器（P3 编辑器前置基建，计划 docs/superpowers/plans/2026-11-09-solver-implementation-plan.md）。
//!
//! **纯度边界**（计划 :222）：lib 下所有模块=纯函数/纯数据——输入 `SolverInput`（由 bin 或测试构造），
//! 输出解；**禁文件/网络/时钟/进程 I/O**（p2s0_lib_pure_no_io 断言 + 判据 `cargo tree` 无反向边）。
//! 文件 I/O 只在 `src/bin/solver.rs` 与测试侧。
//!
//! 批0=输入构造 `ingest`（maplv/sprite/fixture 脚本/三表 → `SolverInput`）+ census 快照。
pub mod ingest;
pub mod state;
