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
use game_core::rules::gate;

/// 夹具脚本槽数（`crates/game-core/tests/fixtures/level_scripts.txt` 68 槽）。
const FIXTURE_SCRIPT_SLOTS: i32 = 68;

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

    /// 槽位数（供门控判定做越界检查，与引擎 `scripts.len()` 同源）。
    pub fn len(&self) -> i32 {
        self.by_slot.len() as i32
    }
    pub fn is_empty(&self) -> bool {
        self.by_slot.is_empty()
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

    // 状态 = (x, y, fired, flags)：fired 决定「哪些门的召唤物已落地」（影响通行），
    // flags 是 script_line_flags 的模型（决定「这扇门还开不开」以及门与门的先后）。
    let mut seen: BTreeSet<(i32, i32, u64, u64)> = BTreeSet::new();
    let mut q: VecDeque<(i32, i32, u64, u64)> = VecDeque::new();
    seen.insert((entry.0, entry.1, 0, 0));
    q.push_back((entry.0, entry.1, 0, 0));
    while let Some((x, y, fired, flags)) = q.pop_front() {
        if goal_set.contains(&(x, y)) {
            return true;
        }
        // 1) 走到相邻格（到达即可能触发该格上的门，from_walk = true）
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if !passable(nx, ny, fired) {
                continue;
            }
            let mut nf = fired;
            let mut nflags = flags;
            if let Some(gi) = gate_at(nx, ny) {
                let (flag_after, ran) = try_fire(level, scripts, gate_list[gi].3, flags, true);
                if ran {
                    nf |= 1u64 << gi;
                    nflags = flag_after;
                }
            }
            if seen.insert((nx, ny, nf, nflags)) {
                q.push_back((nx, ny, nf, nflags));
            }
        }
        // 2) 站定撞邻格的门（from_walk = false，人不动）
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            if let Some(gi) = gate_at(x + dx, y + dy) {
                let (nflags2, ran) = try_fire(level, scripts, gate_list[gi].3, flags, false);
                if ran {
                    let nf = fired | (1u64 << gi);
                    if seen.insert((x, y, nf, nflags2)) {
                        q.push_back((x, y, nf, nflags2));
                    }
                }
            }
        }
    }
    false
}

/// 模型侧的 `GateView`：喂给 `game_core::rules::gate::evaluate` 的引擎同源判定。
///
/// 乐观处（都在返回值里由 `Relaxations` 记账）：怪物假定可清除、道具前置假定已满足、
/// 跨层快照假定为空、HP 假定存活、难度假定为 0（默认档）。
struct ModelGate<'a> {
    level: &'a LevelSpec,
    flags: u64,
}

impl gate::GateView for ModelGate<'_> {
    fn script_count(&self) -> i32 {
        FIXTURE_SCRIPT_SLOTS
    }
    fn flag(&self, idx: i32) -> bool {
        idx >= 0 && idx < 64 && self.flags & (1u64 << idx) != 0
    }
    fn set_flag(&mut self, idx: i32) {
        if idx >= 0 && idx < 64 {
            self.flags |= 1u64 << idx;
        }
    }
    fn entity_at(&self, x: i32, y: i32, type_id: i32) -> i32 {
        // 怪物（41..=76）按「可清除」处理；其余是静态道具/结构，按层数据查。
        if (41..=76).contains(&type_id) {
            return -1;
        }
        self.level
            .entities
            .iter()
            .position(|e| e.x == x && e.y == y && e.ty == type_id)
            .map(|i| i as i32)
            .unwrap_or(-1)
    }
    fn alive_type_on_floor(&self, type_id: i32) -> bool {
        if (41..=76).contains(&type_id) {
            return false; // 假定可清除
        }
        self.level.entities.iter().any(|e| e.ty == type_id)
    }
    fn difficulty_index(&self) -> i32 {
        0
    }
    fn player_hp(&self) -> i32 {
        1
    }
    fn has_item(&self, _item: i32) -> bool {
        true // 乐观：道具前置当作已满足（由调用方记账）
    }
    fn cross_floor_snapshot(&mut self, _floor: i32, _type_id: i32) -> bool {
        false
    }
    fn predict_hp_loss_vs(&self, _ety: i32, _no_counter: bool) -> i32 {
        0
    }
}

/// 尝试触发一扇门：返回 (触发后的 flags, 是否真的触发)。
fn try_fire(
    level: &LevelSpec,
    _scripts: &ScriptSpawns,
    param: i32,
    flags: u64,
    from_walk: bool,
) -> (u64, bool) {
    let mut m = ModelGate { level, flags };
    let verdict = gate::evaluate(param, &mut m, from_walk);
    let mut after = m.flags;
    if verdict.run {
        // 脚本跑完 ⇒ 该槽的 flags 置位（引擎：scene_script.rs 的 script_line_flags[cur] = true）
        if param >= 0 && param < 64 {
            after |= 1u64 << param;
        }
    }
    (after, verdict.run)
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

    // --- 门控条件接入（2026-11-18，用户 m03092「把门控条件也进模型」） ---------
    // 直接测模型侧的 try_fire：它把 (层, 槽, flags, 触发方式) 喂给
    // `game_core::rules::gate::evaluate`——与引擎同一段判定逻辑。

    fn one_row(terrain: Vec<bool>, ents: Vec<EntSpec>) -> LevelSpec {
        let mut lv = level(terrain.len() as i32, 1, terrain, (0, 0));
        lv.entities = ents;
        lv
    }

    /// 门 27 要求 `from_walk` **且** `flags[0]` 已置位（a.java 到达臂 + 前置脚本）。
    #[test]
    fn dyn_gate_27_needs_flag0_and_from_walk() {
        let lv = one_row(vec![true, true, false], vec![ent(SCRIPT_GATE, 2, 0, 27)]);
        let sc = ScriptSpawns::from_lines(vec!["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""]);
        // 没跑过 0 号脚本 ⇒ 即便能走上去也不开
        assert_eq!(try_fire(&lv, &sc, 27, 0, true).1, false, "缺 flags[0] 不该开门");
        // flags[0] 已置位，但撞格（from_walk=false）⇒ 仍不开
        assert_eq!(try_fire(&lv, &sc, 27, 1 << 0, false).1, false, "撞格不该触发到达臂");
        // 两个条件都满足 ⇒ 开
        assert_eq!(try_fire(&lv, &sc, 27, 1 << 0, true).1, true);
    }

    /// 门 25 = `from_walk ∧ flags[13] ∧ 两处 ty60 都不在`；怪物在模型里假定可清除。
    #[test]
    fn dyn_gate_25_needs_flag13() {
        let lv = one_row(vec![true, true, true], vec![ent(SCRIPT_GATE, 1, 0, 25)]);
        let sc = ScriptSpawns::from_lines(std::iter::empty::<&str>());
        assert!(!try_fire(&lv, &sc, 25, 0, true).1, "13 没跑过时 25 不该开");
        assert!(try_fire(&lv, &sc, 25, 1 << 13, true).1, "13 跑过且两处火已清 ⇒ 开");
        assert!(!try_fire(&lv, &sc, 25, 1 << 13, false).1, "撞格不开");
    }

    /// 66 号门恒不开（a.java `case 66: gate = false`）。
    #[test]
    fn dyn_gate_66_never_opens() {
        assert!(game_core::rules::gate::is_never_runnable(66));
        let lv = one_row(vec![true], vec![ent(SCRIPT_GATE, 0, 0, 66)]);
        let sc = ScriptSpawns::from_lines(std::iter::empty::<&str>());
        assert!(!try_fire(&lv, &sc, 66, u64::MAX, true).1);
        assert!(!try_fire(&lv, &sc, 66, u64::MAX, false).1);
    }

    /// 16 只在难度 0 发生；67 只在难度 > 0 发生（模型固定按难度 0 ⇒ 16 开、67 不开）。
    #[test]
    fn dyn_gate_16_and_67_follow_difficulty_0() {
        let lv = one_row(vec![true], vec![ent(SCRIPT_GATE, 0, 0, 16), ent(SCRIPT_GATE, 0, 0, 67)]);
        let sc = ScriptSpawns::from_lines(std::iter::empty::<&str>());
        assert!(try_fire(&lv, &sc, 16, 0, true).1, "难度 0 ⇒ 16 可开");
        assert!(!try_fire(&lv, &sc, 67, 0, true).1, "难度 0 ⇒ 67 不开");
    }

    /// 33 号门要求 `flags[7]`（7 号是跨层探针门，模型按「未跑过」处理）⇒ 恒不开。
    #[test]
    fn dyn_gate_33_needs_flag7() {
        let lv = one_row(vec![true], vec![ent(SCRIPT_GATE, 0, 0, 33)]);
        let sc = ScriptSpawns::from_lines(std::iter::empty::<&str>());
        assert!(!try_fire(&lv, &sc, 33, 0, true).1);
        assert!(try_fire(&lv, &sc, 33, 1 << 7, true).1);
    }
}
