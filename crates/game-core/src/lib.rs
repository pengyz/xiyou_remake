//! game-core —— 游戏逻辑（AGENTS §2 红线：无 I/O、确定性、禁平台 crate）。
//!
//! P3.1：脚本 DSL 解释器（docs/spec/p3-script-interpreter.md）。
pub mod enums;
pub mod game_view;
pub mod layout;
pub mod camera;
pub mod combat;
pub mod entity;
pub mod intro;
pub mod logo_anim;
pub mod menu_family;
pub mod sprite_spawn;
pub mod paint;
pub mod popup;
pub mod pathfind;
pub mod render;
pub mod save;
pub mod scene;
pub mod title;
pub mod script;
pub mod walk;
