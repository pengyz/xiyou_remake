//! dialog face（P2 批0 骨架）——DialogSubsystem：tick 对话推进面（spec §3.1）。
//!
//! 形照 spec §3.1：`fn tick(&mut self, ctx: &mut Ctx) -> DialogStep`。
//! 实现迁移=批3（scene_dialog.rs tick_dialog 簇→impl DialogSubsystem）。

use crate::scene::DialogState;

/// 【批3 占位】对话步结果（批3 终形随实现迁移定——TypewriterStep 同族语义）。
pub struct DialogStep;

/// DialogCtx——DialogSubsystem 专属 Ctx（纯数据）。
///
/// dialog=记录槽（DialogState 22 字段经槽写——写面归因登记 face=DialogCtx）；
/// key_value/key_held/frame_odd=值拷贝读面（coordinator 域只读，不计写集）。
pub struct DialogCtx<'a> {
    pub dialog: &'a mut DialogState,
    pub key_value: i32,
    pub key_held: i32,
    pub frame_odd: bool,
}

/// spec §3.1：对话子系统。
pub trait DialogSubsystem {
    fn tick(&mut self, ctx: &mut DialogCtx) -> DialogStep;
}
