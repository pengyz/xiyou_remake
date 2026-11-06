//! script_host face（P2 批0 骨架）——ScriptHost：脚本指令执行面（spec §3.1）。
//!
//! 形照 spec §3.1：`fn execute_op(&mut self, ctx: &mut Ctx, op: ScriptOp) -> ScriptStep`。
//! 实现迁移=批5（script_ops.rs execute_op 簇→impl ScriptHost）。

use crate::combat::PlayerCombat;
use crate::entity::EntityTable;
use crate::scene::ScriptOp;

/// 【批5 占位】脚本步结果（批5 终形随实现迁移定）。
pub struct ScriptStep;

/// ScriptCtx——ScriptHost 专属 Ctx（纯数据）。
///
/// 写槽=script 域 6 字段（写面归因登记 face=ScriptCtx）；player/entities=读槽。
pub struct ScriptCtx<'a> {
    pub current_script_index: &'a mut usize,
    pub script_cursor: &'a mut usize,
    pub script_delay: &'a mut i32,
    pub script_line: &'a mut String,
    pub script_line_flags: &'a mut Vec<bool>,
    pub script_walk_armed: &'a mut bool,
    pub player: &'a PlayerCombat,
    pub entities: &'a EntityTable,
}

/// spec §3.1：脚本宿主。
pub trait ScriptHost {
    fn execute_op(&mut self, ctx: &mut ScriptCtx, op: ScriptOp) -> ScriptStep;
}
