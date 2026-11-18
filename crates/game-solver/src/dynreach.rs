//! 动态可达性：状态 = **(位置, 已触发脚本门集合)**。
//!
//! # 为什么需要它
//!
//! 静态度量把「地形可走表」当成通行的充分条件，但引擎的通行是两段式：
//! `crates/game-core/src/scene.rs` 的 `interact_with_cell` 先做格内实体交互，**再**做地形终审，
//! 而地形终审带豁免——格内若有 ty6（三昧真火）/ty7（上行梯）/ty8（下行梯），
//! `stair_count > 0` ⇒ 跳过地形判定（a.java:5410/5418/5420 的 `var7++` 与 a.java:5470 的
//! `if (var6 && var7 <= 0)`）。这与 `solve_local` 的 `stair_like(6|7|8)` 是同一条规则。
//!
//! 于是「能不能进某格」不再是格子的静态属性：**门的脚本可以把 ty6 召唤到自己的格子上**，
//! 让原本的墙变得可进。f41 就是这么设计的（门 13 = `CES_6_10_2 CES_60_10_2`，
//! 在 (10,2) 召出 ty6 火与 ty60，随后该格才可进入、门 25 才满足 `from_walk`）。
//!
//! # 模型与可靠性
//!
//! - 状态 `(x, y, fired)`：`fired` 是「哪些脚本门已触发」的位图；召唤物**不建模**为个体，
//!   只按它们对通行的影响折算（ty6/7/8 ⇒ 该格豁免地形；其余如怪物 ⇒ 假定可清除）。
//! - 门可在两种时机触发：**走到格上**（`from_walk = true`）或**从邻格撞**（`from_walk = false`）。
//!   门的门控条件（`script_line_flags` / 实体存在性等）不在本模型内，故这里是**乐观**上界：
//!   只要「触发它」可能有用就允许触发。
//! - 乐观上界用途正确：「所有门都开、所有该触发的都触发」仍走不到 ⇒ 真的走不到（给出否定结论可靠）；
//!   而「某扇门在所有可行方案里都必须开」在此上界下成立 ⇒ 现实中必然也成立（必经判定保守、可靠）。
//!
//! 因此本模块给的是**必要条件**级别的结论，且把「用了动态模型」显式暴露给调用方。

use crate::solve_local::LevelSpec;
use crate::solve_local::EntSpec;
use game_core::enums::entity_kind::{FIRE_TRAP, SCRIPT_GATE, STAIR_DOWN};

/// 一条脚本召唤：`CES_<ty>_<x>_<y>`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spawn {
    pub ty: i32,
    pub x: i32,
    pub y: i32,
}

/// 脚本槽 → 召唤列表（只保留 `CES` 指令；其余指令与通行无关）。
#[derive(Debug, Clone, Default)]
pub struct ScriptSpawns {
    by_slot: Vec<Vec<Spawn>>,
}

impl ScriptSpawns {
    /// 从 `level_scripts.txt` 形态的槽表构建（每行一槽，槽尾可能带记录分隔符 `\u{1e}`）。
    pub fn from_lines<'a, I: IntoIterator<Item = &'a str>>(lines: I) -> Self {
        let mut by_slot = Vec::new();
        for line in lines {
            let line = line.trim_end_matches('\u{1e}').trim();
            let mut spawns = Vec::new();
            for tok in line.split_whitespace() {
                // CES_<ty>_<x>_<y>
                if let Some(rest) = tok.strip_prefix("CES_") {
                    let mut it = rest.split('_');
                    let (a, b, c) = (it.next(), it.next(), it.next());
                    if let (Some(a), Some(b), Some(c)) = (a, b, c) {
                        if let (Ok(ty), Ok(x), Ok(y)) = (a.parse(), b.parse(), c.parse()) {
                            spawns.push(Spawn { ty, x, y });
                        }
                    }
                }
            }
            by_slot.push(spawns);
        }
        Self { by_slot }
    }

    pub fn of(&self, slot: i32) -> &[Spawn] {
        if slot < 0 {
            return &[];
        }
        self.by_slot.get(slot as usize).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

fn terrain_ok(level: &LevelSpec, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < level.cols && y < level.rows && level.terrain[(y * level.cols + x) as usize]
}

fn stairs_at(level: &LevelSpec, x: i32, y: i32) -> bool {
    level
        .entities
        .iter()
        .any(|e| (FIRE_TRAP..=STAIR_DOWN).contains(&e.ty) && e.x == x && e.y == y)
}

/// 本层参与动态搜索的门：(实体下标, x, y, 脚本槽)。
fn gates(level: &LevelSpec) -> Vec<(usize, i32, i32, i32)> {
    level
        .entities
        .iter()
        .enumerate()
        .filter(|(_, e)| e.ty == SCRIPT_GATE)
        .map(|(slot, e)| (slot, e.x, e.y, e.param))
        .collect()
}

/// 动态可达性：`entry` 能否走到任一 `goals`。
///
/// `blocked_doors` 是**被封住的门**的实体下标（判定必经门时逐扇假设封住）。
pub fn can_reach(
    level: &LevelSpec,
    entry: (i32, i32),
    goals: &[(i32, i32)],
    blocked_doors: &std::collections::BTreeSet<usize>,
    scripts: &ScriptSpawns,
) -> bool {
    use std::collections::{BTreeSet, VecDeque};
    if goals.is_empty() {
        return false;
    }
    let goal_set: BTreeSet<(i32, i32)> = goals.iter().copied().collect();
    let gate_list = gates(level);
    if gate_list.len() > 62 {
        // 位图容量守卫：超过就退回「全部已触发」的单状态近似（本仓库实测 52 扇/层 ≤ 2）
        return can_reach_with_all_fired(level, entry, goals, blocked_doors, scripts, &gate_list);
    }
    let door_at: Vec<(i32, i32, usize)> = level
        .entities
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e.ty, 1 | 2 | 3))
        .map(|(slot, e)| (e.x, e.y, slot))
        .collect();
    let blocked_cell = |x: i32, y: i32| {
        door_at
            .iter()
            .any(|(dx, dy, slot)| *dx == x && *dy == y && blocked_doors.contains(slot))
    };
    // 已触发门集合 ⇒ 该格是否获得地形豁免（只计入影响通行的召唤物：ty6/7/8）。
    let exempt = |x: i32, y: i32, fired: u64| -> bool {
        gate_list.iter().enumerate().any(|(i, (_, gx, gy, slot))| {
            fired & (1u64 << i) != 0
                && *gx == x
                && *gy == y
                && scripts.of(*slot).iter().any(|s| (FIRE_TRAP..=STAIR_DOWN).contains(&s.ty) && s.x == x && s.y == y)
        })
    };
    let passable = |x: i32, y: i32, fired: u64| -> bool {
        (terrain_ok(level, x, y) || stairs_at(level, x, y) || exempt(x, y, fired)) && !blocked_cell(x, y)
    };
    let gate_at = |x: i32, y: i32| -> Option<usize> {
        gate_list.iter().position(|(_, gx, gy, _)| *gx == x && *gy == y)
    };

    let mut seen: BTreeSet<(i32, i32, u64)> = BTreeSet::new();
    let mut q: VecDeque<(i32, i32, u64)> = VecDeque::new();
    seen.insert((entry.0, entry.1, 0));
    q.push_back((entry.0, entry.1, 0));
    while let Some((x, y, fired)) = q.pop_front() {
        if goal_set.contains(&(x, y)) {
            return true;
        }
        // 1) 走到相邻格（到达即可能触发该格上的门）
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if !passable(nx, ny, fired) {
                continue;
            }
            let mut nf = fired;
            if let Some(gi) = gate_at(nx, ny) {
                nf |= 1u64 << gi;
            }
            if seen.insert((nx, ny, nf)) {
                q.push_back((nx, ny, nf));
            }
        }
        // 2) 站定撞邻格的门（from_walk=false，人不动）
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            if let Some(gi) = gate_at(x + dx, y + dy) {
                let nf = fired | (1u64 << gi);
                if seen.insert((x, y, nf)) {
                    q.push_back((x, y, nf));
                }
            }
        }
    }
    false
}

/// 门数超位图上限时的保守路径：假设本层所有脚本门都已触发。
fn can_reach_with_all_fired(
    level: &LevelSpec,
    entry: (i32, i32),
    goals: &[(i32, i32)],
    blocked_doors: &std::collections::BTreeSet<usize>,
    scripts: &ScriptSpawns,
    gate_list: &[(usize, i32, i32, i32)],
) -> bool {
    use std::collections::{BTreeSet, VecDeque};
    let goal_set: BTreeSet<(i32, i32)> = goals.iter().copied().collect();
    let mut spawned: BTreeSet<(i32, i32)> = BTreeSet::new();
    for (_, _, _, slot) in gate_list {
        for s in scripts.of(*slot) {
            if (FIRE_TRAP..=STAIR_DOWN).contains(&s.ty) {
                spawned.insert((s.x, s.y));
            }
        }
    }
    let door_at: Vec<(i32, i32, usize)> = level
        .entities
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e.ty, 1 | 2 | 3))
        .map(|(slot, e)| (e.x, e.y, slot))
        .collect();
    let passable = |x: i32, y: i32| {
        let blocked = door_at
            .iter()
            .any(|(dx, dy, slot)| *dx == x && *dy == y && blocked_doors.contains(slot));
        (terrain_ok(level, x, y) || stairs_at(level, x, y) || spawned.contains(&(x, y))) && !blocked
    };
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
            if passable(n.0, n.1) && seen.insert(n) {
                q.push_back(n);
            }
        }
    }
    false
}

/// 本层是否存在「脚本门摆在静态不可走格」——即静态模型不可靠、需要动态搜索。
pub fn needs_dynamic(level: &LevelSpec) -> bool {
    gates(level)
        .iter()
        .any(|(_, x, y, _)| !terrain_ok(level, *x, *y) && !stairs_at(level, *x, *y))
}

/// 供调用方构造测试/离线数据用：把 `CES` 召唤表直接喂进来的便捷入口。
pub fn spawns_of(level: &LevelSpec, scripts: &ScriptSpawns) -> Vec<(i32, i32, i32)> {
    let mut out = Vec::new();
    for e in level.entities.iter().filter(|e| e.ty == SCRIPT_GATE) {
        for s in scripts.of(e.param) {
            out.push((s.ty, s.x, s.y));
        }
    }
    out
}

#[allow(dead_code)]
fn _assert_ent_spec_has_param(_e: &EntSpec) {}

/// 68 槽脚本夹具（`crates/game-core/tests/fixtures/level_scripts.txt`）。
/// 与 `script_abs.rs` 读同一份文件——离线分析不另造脚本表。
pub fn fixture_scripts() -> ScriptSpawns {
    ScriptSpawns::from_lines(include_str!("../../game-core/tests/fixtures/level_scripts.txt").lines())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solve_local::{EntSpec, ExitKind, ExitSpec};
    use game_core::enums::entity_kind::STAIR_UP;

    fn level(cols: i32, rows: i32, terrain: Vec<bool>, start: (i32, i32)) -> LevelSpec {
        LevelSpec {
            cols,
            rows,
            terrain,
            start,
            entities: Vec::new(),
            exits: Vec::<ExitSpec>::new(),
            player: player(),
            enemies: Vec::new(),
            enemy_base: Vec::new(),
            flags: 0,
        }
    }

    fn player() -> crate::solve_local::PlayerSpec {
        crate::solve_local::PlayerSpec {
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
        }
    }

    fn ent(ty: i32, x: i32, y: i32, param: i32) -> EntSpec {
        EntSpec { ty, x, y, param }
    }

    /// 墙格上的门：脚本在该格召出 ty6 ⇒ 动态搜索可达；静态不可达。
    #[test]
    fn dyn_gate_spawn_makes_wall_cell_enterable() {
        let mut lv = level(5, 1, vec![true, true, false, true, true], (0, 0));
        lv.entities.push(ent(STAIR_DOWN, 0, 0, 0));
        lv.entities.push(ent(STAIR_UP, 4, 0, 0));
        lv.entities.push(ent(SCRIPT_GATE, 2, 0, 0)); // 脚本槽 0
        let scripts = ScriptSpawns::from_lines(vec!["CES_6_2_0"]); // 在 (2,0) 召出 ty6
        let blocked = std::collections::BTreeSet::new();
        assert!(
            can_reach(&lv, (0, 0), &[(4, 0)], &blocked, &scripts),
            "门脚本召出 ty6 ⇒ 地形豁免 ⇒ (2,0) 可进 ⇒ 可达"
        );
        // 同一层但没有脚本表 ⇒ 墙仍是墙
        assert!(!can_reach(&lv, (0, 0), &[(4, 0)], &blocked, &ScriptSpawns::default()));
        assert!(needs_dynamic(&lv), "门在墙格 ⇒ 需要动态模型");
    }

    /// 无脚本门的普通层不需要动态模型。
    #[test]
    fn dyn_plain_floor_needs_no_dynamic() {
        let lv = level(3, 1, vec![true, true, true], (0, 0));
        assert!(!needs_dynamic(&lv));
    }

    /// 撞格触发（人不动）也能开门——覆盖 from_walk=false 的路径。
    #[test]
    fn dyn_bump_can_fire_gate() {
        let mut lv = level(5, 1, vec![true, true, false, true, true], (0, 0));
        lv.entities.push(ent(SCRIPT_GATE, 2, 0, 0));
        let scripts = ScriptSpawns::from_lines(vec!["CES_6_2_0"]);
        // 站在 (0,0)/(1,0) 撞 (2,0) 的门即可触发召唤
        let blocked = std::collections::BTreeSet::new();
        assert!(can_reach(&lv, (1, 0), &[(2, 0)], &blocked, &scripts), "撞格触发后应可进入");
        let _ = ExitKind::Stair;
    }

    /// 真夹具钉桩：f41 的门 13 槽在 (10,2) 召出 ty6（这是「墙格变可进」的唯一原因），
    /// 门 25 槽在 (6,6) 召出下行符 22；f48 的门 38 槽也必须在自己的格子上召出可豁免地形的实体。
    #[test]
    fn dyn_fixture_slots_hold_the_self_enabling_spawns() {
        let s = fixture_scripts();
        assert!(
            s.of(13).iter().any(|sp| sp.ty == FIRE_TRAP && sp.x == 10 && sp.y == 2),
            "槽 13 必须含 CES_6_10_2（否则 f41 (10,2) 永远进不去）"
        );
        assert!(
            s.of(25).iter().any(|sp| sp.ty == 22 && sp.x == 6 && sp.y == 6),
            "槽 25 必须含 CES_22_6_6（下行符的唯一来源）"
        );
        assert!(
            s.of(38).iter().any(|sp| (FIRE_TRAP..=STAIR_DOWN).contains(&sp.ty) && sp.x == 4 && sp.y == 1),
            "槽 38 必须在 (4,1) 召出可豁免地形的实体（f48 门格）"
        );
    }

    /// CES 解析：只认 `CES_`，且容忍槽尾记录分隔符。
    #[test]
    fn dyn_parses_ces_slots() {
        let s = ScriptSpawns::from_lines(vec!["CES_6_10_2 CES_60_10_2\u{1e}", "TAK_1_2"]);
        assert_eq!(s.of(0), &[Spawn { ty: 6, x: 10, y: 2 }, Spawn { ty: 60, x: 10, y: 2 }]);
        assert!(s.of(1).is_empty());
        assert!(s.of(99).is_empty());
    }
}
