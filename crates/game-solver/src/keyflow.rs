//! 逐层累计供需对账：把「必经门清单」变成一条**时间轴上的资源序列**。
//!
//! 供需口径：
//! - **需求**：该层必经门按颜色的数量（每扇门消耗 1 把同色钥匙）。`DemandMode::AllDoors`
//!   则统计该层全部同色门（模拟「见门就开」的另一种极端）。
//! - **供给**：该层地面拾取的钥匙实体（ty 26=黄 / 27=红 / 28=蓝）。
//! - **假设（已在文档标注）**：同层先拾取后开门——拾取格若在门后则不成立，属保守方向的
//!   反例，留作保留项而不是当成结论。
//!
//! 它回答的问题：**走到第 N 层时，手里是否已经攒够该层要花的钥匙**；断供的层就是段内目标。

use crate::doors;
use crate::solve_local::LevelSpec;
use std::collections::BTreeMap;

/// 需求口径。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemandMode {
    /// 只统计必经门（最小开销）
    Necessary,
    /// 统计该层全部同色门（见门就开）
    AllDoors,
}

/// 颜色下标：0=黄 1=红 2=蓝。
pub const COLORS: [&str; 3] = ["黄", "红", "蓝"];

/// 门/钥匙类型码 → 颜色下标。类型码本身是**机制常量**，单一来源在 `game_core::enums`。
pub fn color_index(ty: i32) -> Option<usize> {
    use game_core::enums::entity_kind::*;
    match ty {
        DOOR_YELLOW | KEY_YELLOW => Some(0),
        DOOR_RED | KEY_RED => Some(1),
        DOOR_BLUE | KEY_BLUE => Some(2),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloorFlow {
    pub floor: i32,
    /// 该层需求（按颜色下标）
    pub demand: [i32; 3],
    /// 该层拾取供给
    pub pickup: [i32; 3],
    /// 结算后余额
    pub balance: [i32; 3],
    /// 该层哪些颜色发生断供（余额不足）
    pub broke: [bool; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowReport {
    pub rows: Vec<FloorFlow>,
    /// 每色首次断供层（无断供则 None）
    pub first_break: [Option<i32>; 3],
    /// 每色累计需求/累计供给（全程合计）
    pub total_demand: [i32; 3],
    pub total_pickup: [i32; 3],
    /// 无法给出门判定的层（无出发型出口，换层靠脚本）——结论必须显式带上
    pub no_exit_floors: Vec<i32>,
    /// 门判定依赖运行期召唤物的层（ty83 摆在不可走格上，靠脚本召唤 ty6 火触发地形豁免）。
    /// 这些层的门照常计数，但**必经判定只是静态近似**，结论里必须点名。
    pub dynamic_floors: Vec<i32>,
}

/// 逐层结算（升序）。
pub fn analyze(floors: &BTreeMap<i32, LevelSpec>, mode: DemandMode) -> FlowReport {
    let mut rows = Vec::new();
    let mut balance = [0i32; 3];
    let mut first_break: [Option<i32>; 3] = [None, None, None];
    let mut total_demand = [0i32; 3];
    let mut total_pickup = [0i32; 3];
    let mut no_exit_floors: Vec<i32> = Vec::new();
    let mut dynamic_floors: Vec<i32> = Vec::new();
    for (f, lv) in floors {
        let (facts, status) = doors::analyze_floor(lv);
        match status {
            doors::FloorStatus::Ok => {}
            doors::FloorStatus::DynamicGate => dynamic_floors.push(*f),
            _ => no_exit_floors.push(*f),
        }
        let mut demand = [0i32; 3];
        for d in &facts {
            let open = match mode {
                DemandMode::Necessary => d.necessary,
                DemandMode::AllDoors => true,
            };
            if open {
                if let Some(i) = color_index(d.ty) {
                    demand[i] += 1;
                }
            }
        }
        let mut pickup = [0i32; 3];
        for e in &lv.entities {
            // 只算钥匙拾取物；门本身在 demand 里处理
            if matches!(
                e.ty,
                game_core::enums::entity_kind::KEY_YELLOW
                    | game_core::enums::entity_kind::KEY_RED
                    | game_core::enums::entity_kind::KEY_BLUE
            ) {
                if let Some(i) = color_index(e.ty) {
                    pickup[i] += 1;
                }
            }
        }
        for i in 0..3 {
            balance[i] += pickup[i];
            total_pickup[i] += pickup[i];
            total_demand[i] += demand[i];
        }
        let mut broke = [false; 3];
        for i in 0..3 {
            if balance[i] < demand[i] {
                broke[i] = true;
                if first_break[i].is_none() {
                    first_break[i] = Some(*f);
                }
            } else {
                balance[i] -= demand[i];
            }
        }
        rows.push(FloorFlow {
            floor: *f,
            demand,
            pickup,
            balance,
            broke,
        });
    }
    FlowReport {
        rows,
        first_break,
        total_demand,
        total_pickup,
        no_exit_floors,
        dynamic_floors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solve_local::{EntSpec, ExitKind, ExitSpec, LevelSpec, PlayerSpec};
    use game_data::defs::EnemyDef;

    fn level(entities: Vec<EntSpec>) -> LevelSpec {
        LevelSpec {
            cols: 3,
            rows: 1,
            terrain: vec![true, true, true],
            start: (0, 0),
            entities,
            exits: vec![ExitSpec { x: 2, y: 0, kind: ExitKind::Stair, index: 1 }],
            player: PlayerSpec {
                hp: 100,
                atk: 5,
                def: 5,
                yellow: 0,
                blue: 0,
                red: 0,
                gold: 0,
                item_types: vec![],
                item_uses: vec![],
                item_stack_size: 0,
            },
            enemies: Vec::<EnemyDef>::new(),
            enemy_base: Vec::<EnemyDef>::new(),
            flags: 0,
        }
    }

    fn doors_map(items: Vec<(i32, Vec<EntSpec>)>) -> BTreeMap<i32, LevelSpec> {
        let mut m = BTreeMap::new();
        for (f, e) in items {
            m.insert(f, level(e));
        }
        m
    }

    /// 第 1 层就有一扇必经红门、却没有红钥匙 ⇒ 第 1 层断供。
    #[test]
    fn keyflow_breaks_when_floor_demands_more_than_carried() {
        let m = doors_map(vec![(1, vec![EntSpec { ty: 2, x: 1, y: 0, param: 0 }])]);
        let r = analyze(&m, DemandMode::Necessary);
        assert_eq!(r.first_break[color_index(27).unwrap()], Some(1), "红钥匙断在 f1");
        assert_eq!(r.total_demand, [0, 1, 0]);
    }

    /// 同层先拾取后开门：f1 拾到红钥匙再开红门 ⇒ 不断供。
    #[test]
    fn keyflow_pickup_before_door_covers_same_floor() {
        let m = doors_map(vec![(
            1,
            vec![
                EntSpec { ty: 2, x: 1, y: 0, param: 0 },
                EntSpec { ty: 27, x: 0, y: 0, param: 0 },
            ],
        )]);
        let r = analyze(&m, DemandMode::Necessary);
        assert!(r.first_break.iter().all(|b| b.is_none()), "同层拾取足够则不报断供");
        assert_eq!(r.rows[0].balance, [0, 0, 0]);
    }

    /// 全门口径比必经口径更早断供（两扇平行红门都能绕，但「见门就开」要两把钥匙）。
    #[test]
    fn keyflow_all_doors_mode_is_stricter() {
        let mut lv = level(vec![
            EntSpec { ty: 2, x: 1, y: 0, param: 0 },
            EntSpec { ty: 2, x: 1, y: 1, param: 0 },
            EntSpec { ty: 27, x: 0, y: 0, param: 0 },
        ]);
        // 3×2 全可走：(0,0)→(2,0)，(1,0)/(1,1) 两扇红门互为旁路
        lv.cols = 3;
        lv.rows = 2;
        lv.terrain = vec![true; 6];
        lv.exits = vec![ExitSpec { x: 2, y: 0, kind: ExitKind::Stair, index: 1 }];
        let m: BTreeMap<i32, LevelSpec> = [(1, lv)].into_iter().collect();
        let nec = analyze(&m, DemandMode::Necessary);
        let all = analyze(&m, DemandMode::AllDoors);
        assert_eq!(nec.total_demand[1], 0, "3×1 走廊里两扇门都可绕 ⇒ 必经 0");
        assert_eq!(all.total_demand[1], 2, "全门口径两扇都算");
        assert_eq!(all.first_break[1], Some(1), "全门口径下红钥匙不足");
    }
}
