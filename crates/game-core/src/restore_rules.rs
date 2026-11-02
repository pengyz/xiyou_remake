//! task-15 V1 文件归位（v3 §0 真实剩余 1）：restore_rules=类型+表+执行器汇 1（move-only）。
use super::*;

/// 恢复特例动作（m_122 层特例 switch 的三个 case）。
pub enum RestoreAction {
    /// case 2（a.java:9650-9652）：`!flags[flag] && find(find) < 0` → spawn（像素坐标）。
    SpawnIfAbsent {
        flag: usize,
        find: (i32, i32, i32),
        spawn: (i32, i32, i32, i16),
    },
    /// case 12（a.java:9653-9661）：find 命中 → param = lo | DOOR13_HI[hi_idx] << 8
    ///（原式 6|door13_hi[5]<<8 与 5|door13_hi[4]<<8）。
    ParamRewrite {
        find: (i32, i32, i32),
        lo: i16,
        hi_idx: usize,
    },
    /// case 39（a.java:9664-9671）：find 命中 → removeGroup(find) + spawn。
    /// **像素字面量保真**：Java `spawnEntity(8, 11, 1, 38)` 字节码常量即像素 (11,1)
    /// ⇒ 落 cell(0,0)（javap 1446-1454/1483-1492 逐字）——原版实况，D4 不"修正"。
    SwapSpawn {
        find: (i32, i32, i32),
        spawn: (i32, i32, i32, i16),
    },
}

/// 层恢复特例规则（floor → 动作表）。
pub struct RestoreRule {
    pub floor: i32,
    pub actions: &'static [RestoreAction],
}

/// m_122 恢复特例表（a.java:9648-9673 switch(currentFloor) 逐 case）。
pub static RESTORE_RULES: &[RestoreRule] = &[
    RestoreRule {
        floor: 2,
        actions: &[RestoreAction::SpawnIfAbsent {
            flag: 26,
            find: (11, 4, 84),
            spawn: (84, 352, 128, 0),
        }],
    },
    RestoreRule {
        floor: 12,
        actions: &[
            RestoreAction::ParamRewrite { find: (1, 1, 78), lo: 6, hi_idx: 5 },
            RestoreAction::ParamRewrite { find: (11, 1, 78), lo: 5, hi_idx: 4 },
        ],
    },
    RestoreRule {
        floor: 39,
        actions: &[
            RestoreAction::SwapSpawn { find: (11, 1, 7), spawn: (8, 11, 1, 38) },
            RestoreAction::SwapSpawn { find: (11, 11, 8), spawn: (7, 11, 11, 40) },
        ],
    },
];

impl GameScene {
    /// 应用层恢复特例（m_122 尾 switch(currentFloor)，a.java:9648-9673）——
    /// RESTORE_RULES 表驱动；flags[26] 双重角色与 case 39 像素字面量 quirk 随表保留。
    pub(super) fn apply_restore_rules(
        &self,
        entities: &mut EntityTable,
        grid: &mut CellGrid,
    ) {
        let Some(rule) = RESTORE_RULES.iter().find(|r| r.floor == self.persist.floor) else {
            return;
        };
        for action in rule.actions {
            match *action {
                RestoreAction::SpawnIfAbsent { flag, find, spawn } => {
                    // flags[26] 双重角色：守卫=旗标关 + 目标格无活同型（a.java:9650-9652）
                    if self.script_line_flags[flag] {
                        continue;
                    }
                    if entity::cell_entity(grid, entities, find.0, find.1, find.2) >= 0 {
                        continue;
                    }
                    let img_h = self.images.entity[spawn.0 as usize].as_ref().map(|i| i.height);
                    entity::spawn(
                        entities,
                        grid,
                        spawn.0,
                        spawn.1,
                        spawn.2,
                        spawn.3,
                        &self.width_table,
                        img_h,
                    );
                }
                RestoreAction::ParamRewrite { find, lo, hi_idx } => {
                    // a.java:9653-9661：find 命中 → param = lo | door13_hi[hi_idx] << 8
                    let idx = entity::cell_entity(grid, entities, find.0, find.1, find.2);
                    if idx >= 0 {
                        entities.items[idx as usize].param =
                            lo | ((DOOR13_HI[hi_idx] as i16) << 8);
                    }
                }
                RestoreAction::SwapSpawn { find, spawn } => {
                    // a.java:9664-9671：find 命中 → removeGroup + spawn（像素字面量保真）
                    if entity::cell_entity(grid, entities, find.0, find.1, find.2) < 0 {
                        continue;
                    }
                    entity::remove_group_at_cell_by_type(find.0, find.1, find.2, entities, grid);
                    let img_h = self.images.entity[spawn.0 as usize].as_ref().map(|i| i.height);
                    entity::spawn(
                        entities,
                        grid,
                        spawn.0,
                        spawn.1,
                        spawn.2,
                        spawn.3,
                        &self.width_table,
                        img_h,
                    );
                }
            }
        }
    }
}
