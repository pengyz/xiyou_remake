//! item_service face（P2 批0 骨架）——ItemService：Registry 静态查表（分发）+Service 生命周期（pickup/use 编排）（spec §3.1 注①+§7-Q1）。
//!
//! spec §3.1 方法面=…形（未定形）；批6 终形=pickup/use 生命周期编排（分发链唯一：Service→Registry::lookup→ItemEffect 序，
//! 现 execute_item_action 形保持，禁第二分发链）。骨架不预设方法（防 C 级臆造）。

/// ItemService——空壳 trait（方法清单批6 终形；分发链唯一见 effect_registry）。
pub trait ItemService {}
