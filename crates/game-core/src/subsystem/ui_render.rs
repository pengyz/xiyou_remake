//! ui_render face（P2 批0 骨架）——UIRender：UiFrame 纯读快照面（spec §3.1 注③=只读边界件，不入互调通信面）。
//!
//! 形照 plan 批8 锚：`fn snapshot(&self) -> UiFrame`。实现迁移=批8（E3 RenderState 消费面）。

/// 【批8 占位】UI 快照帧（批8 终形随实现迁移定）。
pub struct UiFrame;

/// spec §3.1：UI 渲染纯读边界件（不入互调通信面——只读）。
pub trait UIRender {
    fn snapshot(&self) -> UiFrame;
}
