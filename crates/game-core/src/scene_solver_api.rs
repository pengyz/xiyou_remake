//! 求解器直调门面（方案 B · 求解器批1；提案
//! `docs/superpowers/proposals/2026-11-09-game-core-solver-facade-proposal.md`
//! §1.1/§4）：3 个纯转发 fn，底层 `pub(super)` 零改动。
//!
//! 路径 `game_core::scene::solver_api::*`——必须挂 scene 树内（底层三 fn 均
//! `pub(super)`=`pub(in scene)`，挂 lib.rs 层编译不过）；`#[doc(hidden)]` 收口
//! （先例：lib.rs 对 `scene_test_support` 的 doc(hidden) 重导出）。
//!
//! # 模块契约（调用前置——违反即 panic，消费方必须保证）
//! - `parse_op` 三条前置，任一不满足即 panic：
//!   1. `script_cursor0 + 3 <= line.len()`；
//!   2. `script_cursor0` 落在 UTF-8 字符边界上——底层 `script_ops.rs:402`
//!      `&line[script_cursor0..script_cursor0 + 3]` 无守卫；`scene_script.rs:17`
//!      的起始门 `script_cursor0 + 3 > line.len()` **只拦字节越界、不拦字符
//!      边界**，非 ASCII 行可穿门后在切片处 panic；
//!   3. 该指令的**参数分隔符齐全**——`parse_script_int`（scene_script.rs:7-8）
//!      `line[from..].find(delim).expect("脚本整数分隔符缺失")`，分隔符缺失即
//!      panic（Java 同构：`a.java:8613 parseScriptInt` `indexOf`=-1 ⇒
//!      `substring(from,-1)` 同样抛异常）。
//!   真实语料（level_scripts.txt 68 槽）实测全 ASCII、指令 3 字节对齐（前置
//!   1/2 恒成立）；前置 3 由 **67/68 槽**满足，槽67 `TAK_264_268` 不满足——
//!   `from_fixtures`（scene.rs:253-254，`.trim()`）吃掉无 `\x1e` 末行的行尾
//!   空格，TAK 第二参分隔符随之消失。该槽经 DC-5 裁决**生产不可达**
//!   （`docs/spec/dead-code-rulings.md:170-200`），两侧（门面/底层）同因同崩，
//!   panic-parity 由 Commit2 G5b 语料行走的槽67 专测断言。
//! - `execute_op` / `execute_script_instruction`：入参须为已构造完成的
//!   `&mut GameScene`（load_floor 后），`script_line`/`script_cursor` 由调用方
//!   或上层装载。
//!
//! # R1 红线（提案 §5）
//! 本模块是求解器 **(a) 侧专用**直调面：(b) 对拍腿唯一合法路径是
//! `game_core::script::ScriptEngine`——禁止 (b) 侧引用 `scene::solver_api`，
//! 禁止以 GameScene 解释器路径（含 load_level_script+门面）充当 (b)，违者
//! 对拍判无效。
//!
//! # Y3 锚漂移
//! 本文件经 scene.rs 一行 `#[path]` 挂载挂入，挂载行使 scene.rs 全部行锚
//! **+1**（漂移量记于 Commit1 message；实施后全部 scene.rs 锚已重测）。

use super::{GameScene, ScriptOp};

/// 纯转发：`GameScene::parse_op`（script_ops.rs:401，`pub(super)` 零改动）。
/// 调用前置见模块契约 doc（越界/非字符边界 ⇒ 切片 panic）。
#[inline]
pub fn parse_op(line: &str, script_cursor0: usize) -> Option<ScriptOp> {
    GameScene::parse_op(line, script_cursor0)
}

/// 纯转发：`GameScene::execute_op`（script_ops.rs:553，`pub(super)` 零改动，
/// 内含 GIN 尾递归与 ScriptResolver 域写）。
#[inline]
pub fn execute_op(scene: &mut GameScene, op: ScriptOp) {
    scene.execute_op(op);
}

/// 纯转发：`GameScene::execute_script_instruction`（scene_script.rs:14）——
/// 行首起始门 + parse_op + execute_op 整步；行尾收尾（phase=4+锁相机）同源。
#[inline]
pub fn execute_script_instruction(scene: &mut GameScene) {
    scene.execute_script_instruction();
}

