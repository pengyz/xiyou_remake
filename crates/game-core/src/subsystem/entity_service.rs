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

/// EntityService——实体服务面（批1 终形）：HostCtx 拆小首件，实体面 9 方法逐字迁移
/// （签名/doc 逐字自 script.rs HostCtx；too-many-args 冻结不追溯——执行则4）。
/// 【F4 方法名机器断言】方法名清单由 mod.rs p2b1_entity_service_method_names 钉死；
/// 名单=cell_entity/entity_type/remove_entity/remove_all_of_type/swd_convert_type81/
/// entity_pixel/reload_floor_entities/spawn_entity/set_entity_route_flag。
pub trait EntityService {
    /// findBlockingEntity：格 (x,y) 上 type_id 类型的实体索引（无则 -1）。
    fn cell_entity(&self, x: i32, y: i32, type_id: i32) -> i32;
    /// 实体类型读取（entityType[idx]）。
    fn entity_type(&self, idx: i32) -> i32;
    /// markEntityRemoved：移除实体。
    fn remove_entity(&mut self, idx: i32);
    /// DES 负参变体：按类型批量移除（Java 全表遍历，Host 侧持有实体表）。
    fn remove_all_of_type(&mut self, type_id: i32);
    /// SWD：类型 81 → 4 改写（Java 写五字段：hidden/texture/solid/anim/param）。
    fn swd_convert_type81(&mut self);
    fn entity_pixel(&self, idx: i32) -> (i32, i32);
    /// m_119/m_122：层实体表重载。
    fn reload_floor_entities(&mut self, floor: i32);
    /// spawnEntity：像素坐标生成实体 → 索引。
    fn spawn_entity(&mut self, type_id: i32, px: i32, py: i32) -> i32;
    /// MOV 5 参分支：实体 72 的路线标志写（entityAnimRow[idx]=1，a.java:8275-8271）。
    fn set_entity_route_flag(&mut self, idx: i32);
}
