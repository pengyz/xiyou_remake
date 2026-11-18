//! 层间转移图：把「换层」的三类真实通道收成一张表。
//!
//! 三类通道（A 级，引擎同源）：
//! 1. **楼梯**：`ty=7`（上）/`ty=8`（下）实体，目标层 = `param & 0xff`
//!    （`crates/game-core/src/scene.rs:3966-3978`，a.java:5660/5666）；
//! 2. **脚本 `LAY_x`**：`op_lay` 写 `pending_floor=x`（`crates/game-core/src/script_ops.rs:67-74`，a.java:8355）；
//!    触发通道是 ty83 门（`param`=槽号）⇒ 边 = 门所在层 → `LAY` 目标层；
//! 3. **道具**：上行符 21=`change_floor(floor+1)`、下行符 22=`change_floor(floor-1)`
//!    （`crates/game-core/src/item_effect.rs:140-150`）——这两条是**资源边**，需要持有道具，
//!    本模块只登记「存在这条边」，是否拿得起交给预算层。
//!
//! 只做数据，不做判定：可行性由 `run_witness` 拿着这张表去搜。

use crate::solve_local::LevelSpec;
use std::collections::{BTreeMap, BTreeSet};

/// 换层通道类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// 上楼梯（ty=7）。
    StairUp,
    /// 下楼梯（ty=8）。
    StairDown,
    /// 脚本 `LAY_x`（经 ty83 门触发）。
    Script,
    /// 上行符（item 21，`floor+1`）。
    ItemUp,
    /// 下行符（item 22，`floor-1`）。
    ItemDown,
}

/// 一条层间边。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloorEdge {
    pub from: i32,
    pub cell: (i32, i32),
    pub to: i32,
    pub kind: EdgeKind,
    /// 说明（楼梯 param / 门槽号），用于报告与排错。
    pub detail: String,
}

/// 脚本槽 → `LAY` 目标层（只为带 `LAY` 的槽建项）。
///
/// 与 `dynreach::fixture_scripts` 读同一份夹具（68 槽），不另造脚本表。
pub fn lay_targets_from_lines<'a, I: IntoIterator<Item = &'a str>>(lines: I) -> BTreeMap<i32, i32> {
    let mut out = BTreeMap::new();
    for (slot, line) in lines.into_iter().enumerate() {
        let line = line.trim_end_matches('\u{1e}').trim();
        for tok in line.split_whitespace() {
            if let Some(rest) = tok.strip_prefix("LAY_") {
                if let Ok(target) = rest.parse::<i32>() {
                    out.insert(slot as i32, target);
                }
            }
        }
    }
    out
}

/// 68 槽夹具的 `LAY` 目标表（与 `script_abs`/`dynreach` 同源）。
pub fn lay_targets() -> BTreeMap<i32, i32> {
    lay_targets_from_lines(
        include_str!("../../game-core/tests/fixtures/level_scripts.txt").lines(),
    )
}

/// 本层的层间边（不含道具边，道具边对每层都成立，由 `item_edges` 单独给）。
pub fn edges_of(floor: i32, level: &LevelSpec, lays: &BTreeMap<i32, i32>) -> Vec<FloorEdge> {
    let mut out = Vec::new();
    for e in &level.entities {
        // 楼梯：目标层 = param 低字节（引擎 `param & 0xff`）
        if e.ty == 7 || e.ty == 8 {
            let to = e.param & 0xff;
            if to != floor {
                out.push(FloorEdge {
                    from: floor,
                    cell: (e.x, e.y),
                    to,
                    kind: if e.ty == 7 { EdgeKind::StairUp } else { EdgeKind::StairDown },
                    detail: format!("ty{} param={}", e.ty, e.param),
                });
            }
        }
    }
    // 脚本 LAY：门（ty83）的 param 是槽号，槽里有 LAY_x ⇒ 层间边
    for e in &level.entities {
        if e.ty != 83 {
            continue;
        }
        if let Some(&to) = lays.get(&e.param) {
            if to != floor {
                out.push(FloorEdge {
                    from: floor,
                    cell: (e.x, e.y),
                    to,
                    kind: EdgeKind::Script,
                    detail: format!("gate slot={} LAY_{}", e.param, to),
                });
            }
        }
    }
    out
}

/// 道具换层边（每层恒有，需持有道具）。
pub fn item_edges(floor: i32) -> Vec<FloorEdge> {
    vec![
        FloorEdge {
            from: floor,
            cell: (-1, -1),
            to: floor + 1,
            kind: EdgeKind::ItemUp,
            detail: "item21 上行符".into(),
        },
        FloorEdge {
            from: floor,
            cell: (-1, -1),
            to: floor - 1,
            kind: EdgeKind::ItemDown,
            detail: "item22 下行符".into(),
        },
    ]
}

/// 整张图：层号升序、每层边有序（楼梯/脚本按格序，随后道具边）。
pub fn graph(levels: &BTreeMap<i32, LevelSpec>) -> BTreeMap<i32, Vec<FloorEdge>> {
    let lays = lay_targets();
    let mut out = BTreeMap::new();
    for (floor, level) in levels {
        let mut v = edges_of(*floor, level, &lays);
        v.extend(item_edges(*floor));
        out.insert(*floor, v);
    }
    out
}

/// 只沿**楼梯与脚本**边（不吃道具边）从 `start` 可达的层集合——回答「主线能走到哪」。
pub fn reachable_by_stairs(levels: &BTreeMap<i32, LevelSpec>, start: i32) -> BTreeSet<i32> {
    let lays = lay_targets();
    let mut seen: BTreeSet<i32> = BTreeSet::new();
    let mut stack = vec![start];
    while let Some(f) = stack.pop() {
        if !seen.insert(f) {
            continue;
        }
        let Some(level) = levels.get(&f) else { continue };
        for e in edges_of(f, level, &lays) {
            if levels.contains_key(&e.to) {
                stack.push(e.to);
            }
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solve_local::EntSpec;

    fn level(entities: Vec<EntSpec>) -> LevelSpec {
        LevelSpec {
            cols: 13,
            rows: 13,
            terrain: vec![true; 169],
            start: (1, 11),
            entities,
            exits: Vec::new(),
            player: crate::solve_local::PlayerSpec {
                hp: 300,
                atk: 10,
                def: 10,
                yellow: 0,
                blue: 0,
                red: 0,
                gold: 0,
                item_types: vec![0; 40],
                item_uses: vec![0; 40],
                item_stack_size: 0,
            },
            enemies: Vec::new(),
            enemy_base: Vec::new(),
            flags: 0,
        }
    }

    fn ent(ty: i32, x: i32, y: i32, param: i32) -> EntSpec {
        EntSpec { ty, x, y, param }
    }

    /// 夹具里只有 0 号槽带 LAY，且目标是 2（序章把玩家放到主塔起点）。
    #[test]
    fn fg_fixture_lay_slot0_goes_to_floor2() {
        let lays = lay_targets();
        assert_eq!(lays.get(&0), Some(&2), "0 号槽 LAY_2 = 序章落到 f2");
        assert_eq!(lays.len(), 1, "68 槽里只有一处 LAY（当前数据）");
    }

    /// 楼梯目标层取 param 低字节（引擎 `param & 0xff`）。
    #[test]
    fn fg_stair_target_is_param_low_byte() {
        let lv = level(vec![ent(7, 2, 2, 0x0103), ent(8, 3, 3, 45)]);
        let lays = BTreeMap::new();
        let e = edges_of(1, &lv, &lays);
        assert!(e.iter().any(|e| e.kind == EdgeKind::StairUp && e.to == 3 && e.cell == (2, 2)));
        assert!(e.iter().any(|e| e.kind == EdgeKind::StairDown && e.to == 45));
    }

    /// 门 + 含 LAY 的槽 ⇒ 脚本层间边。
    #[test]
    fn fg_script_edge_from_gate() {
        let lv = level(vec![ent(83, 6, 5, 0)]);
        let mut lays = BTreeMap::new();
        lays.insert(0, 2);
        let e = edges_of(51, &lv, &lays);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].kind, EdgeKind::Script);
        assert_eq!(e[0].to, 2);
        assert_eq!(e[0].cell, (6, 5));
    }
}
