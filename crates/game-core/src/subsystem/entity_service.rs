//! entity_service face（P2 批0 骨架）——EntityService：spawn/remove/relocate 实体服务面（spec §3.1，…形参未定）。
//!
//! plan 批1=HostCtx 实体面 9 方法迁移账（HostCtx 拆小首件，HostCtx 余 12 暂留）。
//! 骨架不预设方法签名（spec …形=参形未定，防 C 级臆造；批1 终形）。

use crate::entity::{CellGrid, EntityTable};

/// EntityCtx——EntityService 专属 Ctx（纯数据）。
///
/// entities=写槽（entity 域经记录槽——写面归因登记 face=EntityCtx，flds=entityCount 族 10 键）；
/// grid/walkable=读槽。
pub struct EntityCtx<'a> {
    pub entities: &'a mut EntityTable,
    pub grid: &'a CellGrid,
    pub walkable: &'a Vec<Vec<bool>>,
}

/// EntityService——空壳 trait（spawn/remove/relocate 方法批1 终形）。
pub trait EntityService {}
