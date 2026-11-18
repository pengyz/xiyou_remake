//! 脚本门（ty83 / `try_run_scene`）的**门控条件**——单一实现。
//!
//! 为什么要抽出来：门控条件原先内联在 `GameScene::try_run_scene` 的 `match idx` 里，
//! 求解器要判「这扇门开不开」只能另抄一份，抄的那份必然与引擎漂移（f41 门 25 那次误判
//! 就是这么来的：静态图不知道「格内有 ty6 ⇒ 地形豁免」，也说不出门与门之间的先后）。
//!
//! 现在：条件表只此一份（`evaluate`），引擎经 `impl GateView for GameScene` 喂它的真实状态，
//! 求解器经自己的 `GateView` 实现喂模型状态。两侧吃同一段判定逻辑。

#![allow(clippy::needless_range_loop)]

/// 门控判定需要读的状态。引擎与求解器各自实现。
///
/// 只有 `cross_floor_snapshot` 需要 `&mut`（引擎侧它要惰性序列化跨层快照）；
/// 其余都是只读查询。
pub trait GateView {
    /// 本层脚本槽数量（越界即不可触发）。
    fn script_count(&self) -> i32;
    /// `script_line_flags[idx]`：该槽是否已跑过。
    fn flag(&self, idx: i32) -> bool;
    /// 置位 `script_line_flags[idx]`（引擎的真写；求解器的模型写）。
    fn set_flag(&mut self, idx: i32);
    /// 单元格上的实体**索引**（`>=0` 表示存在且未移除），`-1` 表示没有。
    fn entity_at(&self, x: i32, y: i32, type_id: i32) -> i32;
    /// 本层是否还有存活的某类型实体（a.java:8637-8657 无换层分支）。
    fn alive_type_on_floor(&self, type_id: i32) -> bool;
    /// `menu.difficultyIndex`。
    fn difficulty_index(&self) -> i32;
    /// 当前 HP。
    fn player_hp(&self) -> i32;
    /// 背包里是否有某道具。
    fn has_item(&self, item: i32) -> bool;
    /// `probeCrossFloorSnapshot(floor, type)`。
    fn cross_floor_snapshot(&mut self, floor: i32, type_id: i32) -> bool;
    /// `predictHpLossVsType(ety, no_counter)`。
    fn predict_hp_loss_vs(&self, ety: i32, no_counter: bool) -> i32;
}

/// 求解器/离线模型在判定时**放宽掉的**条件（乐观假设，必须随结论一起报）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Relaxations {
    /// 「背包里有某道具」被当成真（如门 31 的 item16）。
    pub item: bool,
    /// 「HP 足够」被当成真（如门 24 的 `hp > 0`）。
    pub hp: bool,
    /// 跨层快照按「无存活目标」处理（门 7/32，未移植的跨层探针）。
    pub cross_floor: bool,
}

impl Relaxations {
    pub fn any(&self) -> bool {
        self.item || self.hp || self.cross_floor
    }
}

/// 门控判定结果。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GateVerdict {
    /// 是否放行（放行后由调用方执行脚本内容）。
    pub run: bool,
    /// 判定过程中置位的 flag（**即使不放行也可能置位**，如门 18 的失败标记）。
    pub set_flags: Vec<i32>,
}

/// 门控条件单一实现（原 `GameScene::try_run_scene` 的 `match` 逐臂搬出）。
///
/// `from_walk` 是**触发方式**：`true`=走上该格，`false`=从邻格撞（a.java:5463 `case 83` 传 false）。
pub fn evaluate<V: GateView + ?Sized>(param: i32, v: &mut V, from_walk: bool) -> GateVerdict {
    if param < 0 || param >= v.script_count() {
        return GateVerdict::default();
    }
    let mut set_flags: Vec<i32> = Vec::new();
    let mut gate = !v.flag(param);
    match param {
        0 => {
            if gate {
                set_flags.push(19);
            }
        }
        1 => gate &= v.flag(0) && !from_walk,
        2 => gate &= v.flag(1) && !from_walk,
        3 => gate &= from_walk,
        4 => gate &= v.flag(3) && from_walk,
        // 7/32 需 probeCrossFloorSnapshot(23,12) 跨层快照探针（未移植，显式挡路）。
        7 => {
            gate &= from_walk && !v.cross_floor_snapshot(23, 12);
            if gate {
                set_flags.push(32);
            }
        }
        // 到达臂：a.java:8128-8146 同体 `table_count &= script_cursor0`——**from_walk=true 才过**。
        8 | 9 | 11 | 12 | 19 | 30 | 41 | 42 | 43 | 44 | 45 | 48 | 49 | 50 | 52 | 53 | 55
        | 56 => {
            gate &= from_walk;
        }
        // 探针臂（a.java:8193-8201）：flags[7] 已跑 ⇒ 挡；否则 !from_walk ∧ 跨层有存活型12
        32 => {
            if v.flag(7) {
                gate = false;
            } else {
                gate &= !from_walk && v.cross_floor_snapshot(23, 12);
            }
        }
        // 15（a.java:8119-8126）：当前层有活型5 ⇒ false；否则 &= from_walk
        15 => {
            if v.alive_type_on_floor(5) {
                gate = false;
            } else {
                gate &= from_walk;
            }
        }
        16 => gate &= v.difficulty_index() == 0,
        // 18（a.java:8127-8144）：from_walk 且八格扫描命中 → false 且置 flags[18]
        18 => {
            gate &= from_walk;
            if gate
                && (v.entity_at(2, 2, 1) < 0
                    || v.entity_at(2, 4, 1) < 0
                    || v.entity_at(2, 6, 1) < 0
                    || v.entity_at(6, 2, 1) < 0
                    || v.entity_at(4, 6, 1) < 0
                    || v.entity_at(6, 6, 1) < 0
                    || v.entity_at(4, 2, 1) >= 0
                    || v.entity_at(6, 4, 1) >= 0)
            {
                gate = false;
                set_flags.push(18);
            }
        }
        24 => {
            gate &= !from_walk;
            if gate {
                gate = v.player_hp() > 0;
            }
        }
        25 => {
            gate &= from_walk
                && v.flag(13)
                && v.entity_at(2, 2, 60) < 0
                && v.entity_at(10, 2, 60) < 0;
        }
        26 => {
            if v.entity_at(11, 4, 84) < 0 {
                gate = false;
            }
        }
        27 => gate &= from_walk && v.flag(0),
        31 => {
            gate &= from_walk && v.has_item(16);
        }
        33 => gate &= v.flag(7),
        34 => {
            gate &= from_walk && !v.flag(10);
            if gate {
                let loss = v.predict_hp_loss_vs(69, true);
                gate &= loss < 0 || loss >= v.player_hp();
            }
        }
        // 5/6/10/13/14/17/20/21/22/23/28/29/35/36/37/38/39/40/51/54/57-65：直通（a.java:8098-8108）
        46 => gate &= from_walk && !v.flag(47),
        47 => gate &= from_walk && !v.flag(46),
        66 => gate = false,
        67 => gate &= v.difficulty_index() > 0,
        _ => {}
    }
    for &f in &set_flags {
        v.set_flag(f);
    }
    GateVerdict { run: gate, set_flags }
}

/// 判定该门是否**结构上永不触发**（用于报告「这门是死门」）。不需要状态。
pub fn is_never_runnable(param: i32) -> bool {
    param == 66
}
