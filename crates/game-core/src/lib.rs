//! game-core —— 游戏逻辑（AGENTS §2 红线：无 I/O、确定性、禁平台 crate）。
//!
//! P3.1：脚本 DSL 解释器（docs/spec/p3-script-interpreter.md）。
pub mod enums;
pub mod camera;
pub mod entity;
pub mod sprite_spawn;
pub mod pathfind;
pub mod script;
pub mod walk;
