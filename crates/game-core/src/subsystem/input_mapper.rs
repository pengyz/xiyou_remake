//! input_mapper face（P2 批0 骨架）——InputMapper：RawInput→Action 输入映射面（spec §3.1）。
//!
//! 形照 spec §3.1：`fn map(&self, ev: RawInput) -> Action`。实现迁移=批7。

use crate::controls::Action;

/// 【批7 占位】原始输入事件（批7 终形随实现迁移定）。
pub struct RawInput;

/// spec §3.1：输入映射（Action=controls.rs 既有 pub enum，复用不新建）。
pub trait InputMapper {
    fn map(&self, ev: RawInput) -> Action;
}
