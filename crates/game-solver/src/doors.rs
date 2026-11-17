//! 必经门清单：把 C4-a 的「充分 + 必要」判据从怪格推广到门格。
//!
//! 判定口径（与 `solve_local::blocking_kill_slots` 同构）：
//! - **必要**：单独封住这扇门 ⇒ 从本层落到本层出发梯不可达。这一扇非开不可。
//! - **同色必要**：封住该色全部门 ⇒ 不可达。说明该色钥匙至少要有一把。
//!
//! 起止点取自实体而非写死的 `level.start`：`STAIR_DOWN`(ty=8) 是**到场格**，
//! `STAIR_UP`(ty=7) 是**出发格**（`enums.rs:166-168`）。`LevelSpec::start` 是全局入口
//! （(1,11) 来自 51 层状态），逐层分析用它会在「到场梯不在 (1,11)」的层上判错。
//!
//! 只看「地形 + 门」，不把怪当固定墙（怪是击杀决策），也不看钥匙数量——
//! 它回答「拓扑上是否必经」，不是「钥匙够不够」。「够不够」由钥匙来源清单另核。

use crate::solve_local::LevelSpec;
use std::collections::{BTreeSet, VecDeque};

/// 门的一行事实。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoorFact {
    /// `LevelSpec::entities` 下标
    pub slot: usize,
    /// 1=黄 2=红 3=蓝
    pub ty: i32,
    pub x: i32,
    pub y: i32,
    /// 单独封住它 ⇒ 出发梯不可达（非开不可）
    pub necessary: bool,
    /// 封住同色全部门 ⇒ 不可达（该色钥匙至少一把必须）
    pub color_needed: bool,
}

pub fn color_name(ty: i32) -> &'static str {
    match ty {
        1 => "黄",
        2 => "红",
        3 => "蓝",
        _ => "?",
    }
}

fn in_bounds(level: &LevelSpec, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < level.cols && y < level.rows
}

fn terrain_ok(level: &LevelSpec, x: i32, y: i32) -> bool {
    in_bounds(level, x, y) && level.terrain[(y * level.cols + x) as usize]
}

/// 到场格与出发格集合：`STAIR_DOWN`=入场，`STAIR_UP`=目标；缺则回落 `level.start` / `exits`。
pub fn entry_and_goals(level: &LevelSpec) -> ((i32, i32), Vec<(i32, i32)>) {
    let entry = level
        .entities
        .iter()
        .find(|e| e.ty == 8)
        .map(|e| (e.x, e.y))
        .unwrap_or(level.start);
    let mut goals: Vec<(i32, i32)> = level
        .entities
        .iter()
        .filter(|e| e.ty == 7)
        .map(|e| (e.x, e.y))
        .filter(|p| *p != entry)
        .collect();
    if goals.is_empty() {
        goals = level
            .exits
            .iter()
            .map(|e| (e.x, e.y))
            .filter(|p| *p != entry)
            .collect();
    }
    (entry, goals)
}

/// `entry` 能否走到任一 `goals`；`blocked` 是**被封住的门**的实体下标集合。
fn can_reach(level: &LevelSpec, entry: (i32, i32), goals: &[(i32, i32)], blocked: &BTreeSet<usize>) -> bool {
    if goals.is_empty() {
        return false;
    }
    let goal_set: BTreeSet<(i32, i32)> = goals.iter().copied().collect();
    let goal_set = &goal_set;
    // 门格 → 该格上的门槽位（一格叠多扇时任一被封即封）
    let door_at: Vec<(i32, i32, usize)> = level
        .entities
        .iter()
        .enumerate()
        .filter(|(_, e)| (1..=3).contains(&e.ty))
        .map(|(slot, e)| (e.x, e.y, slot))
        .collect();
    let is_blocked = |x: i32, y: i32| {
        door_at.iter().any(|(dx, dy, slot)| *dx == x && *dy == y && blocked.contains(slot))
    };
    // 楼梯格豁免地形（`solve_local.rs:1183`：地形终审 = 楼梯豁免 `stair_count>0`）。
    let passable = |x: i32, y: i32| terrain_ok(level, x, y) || goal_set.contains(&(x, y)) || (x, y) == entry;
    let mut seen: BTreeSet<(i32, i32)> = BTreeSet::new();
    let mut q: VecDeque<(i32, i32)> = VecDeque::new();
    seen.insert(entry);
    q.push_back(entry);
    while let Some((x, y)) = q.pop_front() {
        if goal_set.contains(&(x, y)) {
            return true;
        }
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let n = (x + dx, y + dy);
            if !seen.contains(&n) && passable(n.0, n.1) && !is_blocked(n.0, n.1) {
                seen.insert(n);
                q.push_back(n);
            }
        }
    }
    false
}

/// 逐门判定「必经 / 同色必要」。基线（门全开）都到不了出发梯 ⇒ 置 `baseline_unreachable`，
/// 此时不虚构必经门（该层缺的不是钥匙，是别的东西——如破墙道具或脚本换层）。
pub fn analyze_floor(level: &LevelSpec) -> (Vec<DoorFact>, bool) {
    let doors: Vec<(usize, i32, i32, i32)> = level
        .entities
        .iter()
        .enumerate()
        .filter(|(_, e)| (1..=3).contains(&e.ty))
        .map(|(slot, e)| (slot, e.ty, e.x, e.y))
        .collect();
    if doors.is_empty() {
        return (Vec::new(), false);
    }
    let (entry, goals) = entry_and_goals(level);
    let mk = |(slot, ty, x, y): (usize, i32, i32, i32), nec: bool, cn: bool| DoorFact {
        slot,
        ty,
        x,
        y,
        necessary: nec,
        color_needed: cn,
    };
    if !can_reach(level, entry, &goals, &BTreeSet::new()) {
        return (doors.iter().map(|d| mk(*d, false, false)).collect(), true);
    }
    // 同色必要：一次封一整色
    let mut colors_needed: BTreeSet<i32> = BTreeSet::new();
    for ty in doors.iter().map(|d| d.1).collect::<BTreeSet<_>>() {
        let set: BTreeSet<usize> = doors.iter().filter(|d| d.1 == ty).map(|d| d.0).collect();
        if !can_reach(level, entry, &goals, &set) {
            colors_needed.insert(ty);
        }
    }
    let out = doors
        .into_iter()
        .map(|d| {
            let mut set = BTreeSet::new();
            set.insert(d.0);
            let nec = !can_reach(level, entry, &goals, &set);
            mk(d, nec, colors_needed.contains(&d.1))
        })
        .collect();
    (out, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solve_local::{EntSpec, ExitKind, ExitSpec, LevelSpec, PlayerSpec};
    use game_data::defs::EnemyDef;

    fn player() -> PlayerSpec {
        PlayerSpec {
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
        }
    }

    fn level(cols: i32, rows: i32, terrain: Vec<bool>, start: (i32, i32)) -> LevelSpec {
        LevelSpec {
            cols,
            rows,
            terrain,
            start,
            entities: vec![],
            exits: vec![ExitSpec {
                x: cols - 1,
                y: rows - 1,
                kind: ExitKind::Stair,
                index: 1,
            }],
            player: player(),
            enemies: Vec::<EnemyDef>::new(),
            enemy_base: Vec::<EnemyDef>::new(),
            flags: 0,
        }
    }

    fn door(ty: i32, x: i32, y: i32) -> EntSpec {
        EntSpec { ty, x, y, param: 0 }
    }

    /// 走廊 3×1：唯一的门必开。
    #[test]
    fn doors_single_corridor_door_is_necessary() {
        let mut lv = level(3, 1, vec![true, true, true], (0, 0));
        lv.entities.push(door(1, 1, 0));
        let (facts, unreachable) = analyze_floor(&lv);
        assert!(!unreachable);
        assert_eq!(facts.len(), 1);
        assert!(facts[0].necessary, "走廊里唯一的门必开");
        assert!(facts[0].color_needed);
    }

    /// 2×2 有旁路：门在 (0,1) 不是必经。
    #[test]
    fn doors_bypassable_door_is_not_necessary() {
        let mut lv = level(2, 2, vec![true, true, true, true], (0, 0));
        lv.entities.push(door(2, 0, 1));
        let (facts, _) = analyze_floor(&lv);
        assert_eq!(facts.len(), 1);
        assert!(!facts[0].necessary, "有旁路 ⇒ 不必经");
        assert!(!facts[0].color_needed, "旁路存在 ⇒ 该色钥匙非必须");
    }

    /// 出发梯被地形封死：显式标记，不虚构必经门。
    #[test]
    fn doors_unreachable_baseline_is_flagged() {
        let mut lv = level(3, 1, vec![true, false, true], (0, 0));
        lv.entities.push(door(1, 1, 0));
        let (facts, unreachable) = analyze_floor(&lv);
        assert!(unreachable, "地形封死出发梯必须显式标记，不静默给空表");
        assert!(facts.iter().all(|f| !f.necessary));
    }

    /// 入场用 STAIR_DOWN(8)、目标用 STAIR_UP(7)，不理会写死的 start。
    #[test]
    fn doors_entry_and_goal_come_from_stairs_not_level_start() {
        // 1×4：start 写 (0,0)（不可走），到场梯在 (3,0)，出发梯在 (1,0)；
        // 门在 (2,0) 位于「到场梯 → 出发梯」必经段上。
        let mut lv = level(4, 1, vec![false, true, true, true], (0, 0));
        lv.entities.push(door(3, 2, 0));
        lv.entities.push(EntSpec { ty: 8, x: 3, y: 0, param: 0 });
        lv.entities.push(EntSpec { ty: 7, x: 1, y: 0, param: 0 });
        let (facts, unreachable) = analyze_floor(&lv);
        assert!(!unreachable, "到场梯 (3,0) 应能走到出发梯 (1,0)");
        assert_eq!(facts.len(), 1);
        assert!(facts[0].necessary, "门卡在到场梯与出发梯之间 ⇒ 必经");
    }
}
