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
use game_core::enums::entity_kind::{
    DOOR_BLUE, DOOR_RED, DOOR_YELLOW, FIRE_TRAP, SCRIPT_GATE, STAIR_DOWN, STAIR_UP,
};
use std::collections::{BTreeSet, VecDeque};

fn is_door(ty: i32) -> bool {
    matches!(ty, DOOR_YELLOW | DOOR_RED | DOOR_BLUE)
}

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
        DOOR_YELLOW => "黄",
        DOOR_RED => "红",
        DOOR_BLUE => "蓝",
        _ => "?",
    }
}

/// 本层为何给不出（或能给出）门的判定——**不允许静默回落**。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorStatus {
    /// 有出发出口，判定有效
    Ok,
    /// 本层没有任何「出发型出口」（例如换层靠脚本）：不是门的问题，不给结论
    NoExit,
    /// 出口存在，但全部门开着也走不到（真正的封死）
    BaselineUnreachable,
    /// 本层存在**摆在不可走格上的脚本门**（ty83）：它的可达性依赖运行期召唤物
    /// （典型：门的脚本自己在该格召出 ty6 火，火触发交互层的「楼梯/喷泉豁免地形」⇒ 格子才可进，
    ///  见 `crates/game-core/src/scene.rs` `interact_with_cell` 与 a.java:5470）。
    ///
    /// 喂了脚本召唤表时，判定走 [`crate::dynreach`] 的动态搜索，结论可用但仍打此标记
    /// （调用方据此知道该层结论来自「乐观上界」而非纯地形）；未喂表时等于静态近似，必须当成近似读。
    DynamicGate,
}

fn in_bounds(level: &LevelSpec, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < level.cols && y < level.rows
}

fn terrain_ok(level: &LevelSpec, x: i32, y: i32) -> bool {
    in_bounds(level, x, y) && level.terrain[(y * level.cols + x) as usize]
}

/// 格内是否有「楼梯/喷泉」类实体（ty6 三昧真火 / ty7 上行梯 / ty8 下行梯）。
/// 有 ⇒ 引擎的地形终审被豁免（`crates/game-core/src/scene.rs` `interact_with_cell`；a.java:5470
/// 的 `var7 > 0` 分支，var7 只在 `case 6/7/8` 自增）。与 `solve_local` 的 `stair_like` 同一条规则。
fn stair_like_at(level: &LevelSpec, x: i32, y: i32) -> bool {
    level
        .entities
        .iter()
        .any(|e| (FIRE_TRAP..=STAIR_DOWN).contains(&e.ty) && e.x == x && e.y == y)
}

/// 到场格与出发格集合：`STAIR_DOWN`=入场，`STAIR_UP`=目标；缺则回落 `level.start` / `exits`。
/// 到场格与出发格集合。返回 `None` = **本层没有可识别的出发出口**（换层靠脚本等），
/// 调用方必须如实上报，不许拿 `level.start` 顶替后当成「走不到」。
pub fn entry_and_goals(level: &LevelSpec) -> Option<((i32, i32), Vec<(i32, i32)>)> {
    let entry = level
        .entities
        .iter()
        .find(|e| e.ty == STAIR_DOWN)
        .map(|e| (e.x, e.y))
        .unwrap_or(level.start);
    let mut goals: Vec<(i32, i32)> = level
        .entities
        .iter()
        .filter(|e| e.ty == STAIR_UP)
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
    if goals.is_empty() {
        return None;
    }
    Some((entry, goals))
}

/// `entry` 能否走到任一 `goals`；`blocked` 是**被封住的门**的实体下标集合。
/// 本层「脚本门摆在静态不可走格」的格子，以及它们**在动态模型下**能否被走到。
///
/// 这是给报告用的实证面：静态模型对这些格子只能标「不可靠」，动态模型能给出 yes/no。
/// 返回 `(x, y, 脚本槽, 动态可达)`。
pub fn dynamic_probe(
    level: &LevelSpec,
    scripts: &crate::dynreach::ScriptSpawns,
) -> Vec<(i32, i32, i32, bool)> {
    let Some((entry, _goals)) = entry_and_goals(level) else {
        return Vec::new();
    };
    let empty = BTreeSet::new();
    level
        .entities
        .iter()
        .filter(|e| e.ty == SCRIPT_GATE && !terrain_ok(level, e.x, e.y) && !stair_like_at(level, e.x, e.y))
        .map(|e| {
            let reach = crate::dynreach::can_reach(level, entry, &[(e.x, e.y)], &empty, scripts);
            (e.x, e.y, e.param, reach)
        })
        .collect()
}

/// 逐门判定「必经 / 同色必要」。基线（门全开）都到不了出发梯 ⇒ 置 `baseline_unreachable`，
/// 此时不虚构必经门（该层缺的不是钥匙，是别的东西——如破墙道具或脚本换层）。
pub fn analyze_floor(level: &LevelSpec) -> (Vec<DoorFact>, FloorStatus) {
    analyze_floor_with(level, &crate::dynreach::ScriptSpawns::default())
}

/// 与 [`analyze_floor`] 相同，但把脚本召唤表喂进来：
/// 格子上「脚本门召出 ty6 火 ⇒ 地形豁免 ⇒ 该格才可进」这类自使能链只有动态模型算得出
/// （见 [`crate::dynreach`] 的模块文档与 `docs/findings/2026-11-18-f41-gate-reachability.md`）。
pub fn analyze_floor_with(
    level: &LevelSpec,
    scripts: &crate::dynreach::ScriptSpawns,
) -> (Vec<DoorFact>, FloorStatus) {
    let doors: Vec<(usize, i32, i32, i32)> = level
        .entities
        .iter()
        .enumerate()
        .filter(|(_, e)| is_door(e.ty))
        .map(|(slot, e)| (slot, e.ty, e.x, e.y))
        .collect();
    // 静态实体集里就有「脚本门摆在不可走格」⇒ 可达性依赖运行期召唤物（自使能链），不给 Ok。
    // 这一判必须先算：本层可能一扇「门」都没有，但照样存在这种不可靠的静态前提。
    let dynamic = level
        .entities
        .iter()
        .any(|e| e.ty == SCRIPT_GATE && !terrain_ok(level, e.x, e.y) && !stair_like_at(level, e.x, e.y));
    let ok_status = || {
        if dynamic {
            FloorStatus::DynamicGate
        } else {
            FloorStatus::Ok
        }
    };
    if doors.is_empty() {
        return (Vec::new(), ok_status());
    }
    let mk = |(slot, ty, x, y): (usize, i32, i32, i32), nec: bool, cn: bool| DoorFact {
        slot,
        ty,
        x,
        y,
        necessary: nec,
        color_needed: cn,
    };
    let Some((entry, goals)) = entry_and_goals(level) else {
        return (
            doors.iter().map(|d| mk(*d, false, false)).collect(),
            FloorStatus::NoExit,
        );
    };
    if !crate::dynreach::can_reach(level, entry, &goals, &BTreeSet::new(), scripts) {
        return (
            doors.iter().map(|d| mk(*d, false, false)).collect(),
            FloorStatus::BaselineUnreachable,
        );
    }
    // 同色必要：一次封一整色
    let mut colors_needed: BTreeSet<i32> = BTreeSet::new();
    for ty in doors.iter().map(|d| d.1).collect::<BTreeSet<_>>() {
        let set: BTreeSet<usize> = doors.iter().filter(|d| d.1 == ty).map(|d| d.0).collect();
        if !crate::dynreach::can_reach(level, entry, &goals, &set, scripts) {
            colors_needed.insert(ty);
        }
    }
    let out = doors
        .into_iter()
        .map(|d| {
            let mut set = BTreeSet::new();
            set.insert(d.0);
            let nec = !crate::dynreach::can_reach(level, entry, &goals, &set, scripts);
            mk(d, nec, colors_needed.contains(&d.1))
        })
        .collect();
    (out, ok_status())
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
        let (facts, status) = analyze_floor(&lv);
        assert_eq!(status, FloorStatus::Ok);
        assert_eq!(facts.len(), 1);
        assert!(facts[0].necessary, "走廊里唯一的门必开");
        assert!(facts[0].color_needed);
    }

    /// 2×2 有旁路：门在 (0,1) 不是必经。
    #[test]
    fn doors_bypassable_door_is_not_necessary() {
        let mut lv = level(2, 2, vec![true, true, true, true], (0, 0));
        lv.entities.push(door(2, 0, 1));
        let (facts, status) = analyze_floor(&lv);
        assert_eq!(status, FloorStatus::Ok);
        assert_eq!(facts.len(), 1);
        assert!(!facts[0].necessary, "有旁路 ⇒ 不必经");
        assert!(!facts[0].color_needed, "旁路存在 ⇒ 该色钥匙非必须");
    }

    /// 出发梯被地形封死：显式标记，不虚构必经门。
    #[test]
    fn doors_unreachable_baseline_is_flagged() {
        let mut lv = level(3, 1, vec![true, false, true], (0, 0));
        lv.entities.push(door(1, 1, 0));
        let (facts, status) = analyze_floor(&lv);
        assert_eq!(status, FloorStatus::BaselineUnreachable, "地形封死出发梯必须显式标记");
        assert!(facts.iter().all(|f| !f.necessary));
    }

    /// 无出发型出口的层（换层靠脚本）必须报 NoExit，**不许**回落成「走不到」。
    #[test]
    fn doors_floor_without_departure_exit_reports_no_exit() {
        let mut lv = level(3, 1, vec![true, true, true], (0, 0));
        lv.entities.push(door(1, 1, 0));
        lv.entities.push(EntSpec { ty: STAIR_DOWN, x: 0, y: 0, param: 0 });
        lv.exits = vec![ExitSpec { x: 0, y: 0, kind: ExitKind::Stair, index: 1 }];
        let (facts, status) = analyze_floor(&lv);
        assert_eq!(status, FloorStatus::NoExit, "只有到场梯没有出发梯 ⇒ NoExit");
        assert!(facts.iter().all(|f| !f.necessary), "不给结论，不虚构必经门");
    }

    /// 入场用 STAIR_DOWN(8)、目标用 STAIR_UP(7)，不理会写死的 start。
    #[test]
    fn doors_entry_and_goal_come_from_stairs_not_level_start() {
        // 1×4：start 写 (0,0)（不可走），到场梯在 (3,0)，出发梯在 (1,0)；
        // 门在 (2,0) 位于「到场梯 → 出发梯」必经段上。
        let mut lv = level(4, 1, vec![false, true, true, true], (0, 0));
        lv.entities.push(door(3, 2, 0));
        lv.entities.push(EntSpec { ty: STAIR_DOWN, x: 3, y: 0, param: 0 });
        lv.entities.push(EntSpec { ty: STAIR_UP, x: 1, y: 0, param: 0 });
        let (facts, status) = analyze_floor(&lv);
        assert_eq!(status, FloorStatus::Ok, "到场梯 (3,0) 应能走到出发梯 (1,0)");
        assert_eq!(facts.len(), 1);
        assert!(facts[0].necessary, "门卡在到场梯与出发梯之间 ⇒ 必经");
    }

    /// 脚本门（ty83）摆在不可走格 ⇒ 静态实体集判定不可靠，必须显式标 DynamicGate，
    /// 不许给 Ok（f41 (10,2) 的真实形态：门 13 的脚本召出 ty6 火，火触发地形豁免后格子才可进）。
    #[test]
    fn doors_gate_on_wall_cell_is_dynamic() {
        let mut lv = level(5, 1, vec![true, true, false, true, true], (0, 0)); // (2,0) 是墙
        lv.entities.push(EntSpec { ty: STAIR_DOWN, x: 0, y: 0, param: 0 });
        lv.entities.push(EntSpec { ty: STAIR_UP, x: 4, y: 0, param: 0 });
        lv.entities.push(EntSpec { ty: SCRIPT_GATE, x: 2, y: 0, param: 13 });
        let (_, status) = analyze_floor(&lv);
        assert_eq!(status, FloorStatus::DynamicGate, "门在墙格 ⇒ 静态判定不可靠");

        // 同格若有 ty6 火（运行期召唤的形态），豁免成立 ⇒ 静态可达，不再标 dynamic。
        lv.entities.push(EntSpec { ty: FIRE_TRAP, x: 2, y: 0, param: 0 });
        let (_, status2) = analyze_floor(&lv);
        assert_eq!(status2, FloorStatus::Ok, "格内有 ty6 ⇒ 地形终审被豁免 ⇒ 静态可达");
    }
}
