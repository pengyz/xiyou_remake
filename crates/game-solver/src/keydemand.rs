//! 【keydemand】只读分析：**每层从入场格走到任一上行出口，最少需要多少把各色钥匙**。
//!
//! 本模块**只产出数字与判据**，不接进 `witness` / `solve_local` / `exact_pickup` 的任何
//! 调用点（零接线）。数据全部来自既有层图 `LevelSpec`（`load_floor` 装载），不新造解析路径。
//!
//! ## 建模口径（每条都指向既有引擎语义，不新立规则）
//!
//! **通行判定 = 直调 `entity_category_with_floor` + 逐字复刻
//! `crates/game-core/src/scene.rs:3251 interact_with_cell` 的 match 臂**（该函数不可直接调用：
//! 它要整台 `Scene`；本模块只复刻「哪些格能进」的判定，不复刻任何其它引擎语义）：
//!
//! | 类目 | 类型 | 引擎行为（scene.rs 锚） | 本模块 |
//! |---|---|---|---|
//! | cat1 | 1/2/3 门 | `consume_key_for_door` 成功则 `entity::remove`（永久开） | 耗 1 钥进入，`opened` 位图 |
//! | cat1 | 4 / 81 | 封印门：恒拦截 / 仅可见时拦截（脚本或杀敌才消失） | **假设 2**：按可通行 + `gates_ignored` |
//! | cat1 | 83 | 场景门：`try_run_scene` 命中才拦截，否则放行（脚本/flag 前置） | **假设 2**：按可通行 + `gates_ignored` |
//! | cat1 | 5 / 76 / 82 | bump 臂无 case（5 是宝箱、76/82 交换型）⇒ 放行 | 可通行 |
//! | cat1 | 6 / 7 / 8 | `stair_count += 1` ⇒ **豁免地形终审**（末尾 `if allowed && stair_count <= 0`） | 可通行；7=出口、8=入口、6 计入 `hazards` |
//! | cat1 | 9 / 10 / 11 / 12 | 炼丹台 / 三昧真火墙 / 墙 / 撞击墙：`allowed=false` | **硬阻挡**，计入 `hard_blocks` |
//! | cat2/4 | 26/27/28 | 踩格拾取（`applyStepCellEffects` 的 `2|4` 臂 + `entity::remove`） | 拾取槽，`picked` 位图 |
//! | cat2 | 32 | 拾取类（数据表 `sprite_defs` 里 *类型* 32 = cat2） | 可通行（**不是门**） |
//! | cat8 | 41..=78 | 撞敌：可胜则开战（**打不过就挡住**） | **假设 3**：按可清 + `monsters_ignored` |
//! | cat16 | 77/78 | 告示牌/商人：触发即 `allowed=false` | **硬阻挡**（不买不撞就过不去），计入 `hard_blocks` |
//! | cat32 | 72/84/87 | `32 => allowed = false`（联动门；72 在 f50 被覆写为 cat8 怪） | **硬阻挡**，计入 `hard_blocks` |
//!
//! - **入场格**：该层 `ty=8`（`STAIR_DOWN`）所在格（多槽取**首个槽序**）。**没有 ty8 ⇒
//!   [`FloorStatus::NoEntry`]**，绝不静默回落到 `LevelSpec::start`
//!   （`doors.rs::entry_and_goals` 的回落口径在这里被明确否决）。
//! - **出口**：该层全部 `ty=7`（`STAIR_UP`）所在格，到达任一即成功。**没有 ty7 ⇒
//!   [`FloorStatus::NoUpExit`]**（f49/f50 就是这种，不编造）。
//! - **地形**：`LevelSpec::terrain`（= maplv `tile_walkability` 同义）；楼梯/火格豁免可在
//!   非地形格上站（同 `interact_with_cell` 末行口径）。
//! - **门预算**：本层门数 > [`MAX_DOORS`]（24，`u32` 掩码）⇒ [`FloorStatus::TooManyDoors`]
//!   **具名拒绝**，不给近似数字。
//! - **钥匙守恒**：本层内钥匙只有「开门扣」与「层内拾取加」两种收支，因此
//!   `held_c = entry_c + |picked∩color_c| − |opened∩color_c|` **恒等成立**，`held` 不进状态；
//!   状态只需 `(pos, picked, opened)`——这是精确枚举可行的关键（无任何状态裁剪）。
//! - **最小需求**：对 `(y,b,r) ∈ 0..=[`MAX_ENTRY_KEYS`]³` 按 total 升序逐点**真跑** BFS
//!   （可行集对分量单调 ⇒ 已知可行向上封闭、已知不可行向下封闭，双向剪枝），输出
//!   Pareto 最小集 + 代表向量 `needed`（total 最小、再字典序）。命中网格边界 ⇒
//!   `grid_bound=true`（真需求可能更大）。网格内全不可行时用无限钥匙探针区分
//!   [`FloorStatus::Unreachable`]（乐观模型下也到不了）与
//!   [`FloorStatus::NeedsMoreThanGrid`]（> 4 把，本网格报不出来）。
//!
//! ## 必须如实标注的假设（CLI 与 JSON 同步输出计数）
//!
//! 1. **只算「走到任一上行梯」**：不算去支线/宝箱/商店路上花的钥匙 ⇒ 真·全程需求可能更高。
//! 2. **条件门的开启条件不建模**：`ty=83`（`SCRIPT_GATE`，脚本/flag 前置）与封印门族
//!    `ty=81/4`（脚本或杀敌前置）一律**按可通行**处理，计入 [`FloorDemand::gates_ignored`]。
//!    门是通路不是钥匙消耗，故它们**不进**门预算（不占 `doors`）。
//! 3. **怪物一律假定可清除**（不扣血、不耗时）⇒ 需求**偏低**，计入
//!    [`FloorDemand::monsters_ignored`]。
//! 4. **火/陷阱按引擎豁免处理（可站）**：`ty=6`（`FIRE_TRAP`）可通行并计入
//!    [`FloorDemand::hazards`]；而 `ty=10` 是「三昧真火墙」，引擎**阻挡**，归入
//!    [`FloorDemand::hard_blocks`]，两者不许混为一谈。
//! 5. **硬阻挡按永久不可通行**（`ty=9` 炼丹台 / `ty=10` 三昧真火墙 / `ty=11` 墙 /
//!    `ty=12` 撞击墙 / cat16 商人·告示牌 / cat32 联动门），计入
//!    [`FloorDemand::hard_blocks`]。引擎里 `ty=11/12` 可被道具或剧情消除、`ty=9` 与商人
//!    可交互 ⇒ 落在必经之路上的硬阻挡会让本模块的需求**偏高**，极端时整层被误判
//!    [`FloorStatus::Unreachable`]（真实案例：f29）。本条**超出任务书 4 条**，单列以示区别。
//!
//! 钥匙向量口径 = **`(y, b, r)`（黄/蓝/红）**，与 JSON 的 `needed_{yellow,blue,red}` 一一对应。
use crate::solve_local::LevelSpec;
use game_core::enums::entity_kind::{
    DOOR_BLUE, DOOR_RED, DOOR_YELLOW, FIRE_TRAP, KEY_BLUE, KEY_RED, KEY_YELLOW, STAIR_DOWN, STAIR_UP,
};
use game_core::subsystem::script_host::entity_category_with_floor;
use std::collections::VecDeque;

/// 入场钥匙网格上限（每色 0..=4，与任务书一致）。
pub const MAX_ENTRY_KEYS: i32 = 4;
/// 门掩码位宽上限（`u32`）；超过 ⇒ [`FloorStatus::TooManyDoors`] 具名拒绝。
pub const MAX_DOORS: usize = 24;
/// 拾取掩码位宽上限（`u128`）；真实 54 层最多 14 个，超限属夹具异常。
pub const MAX_PICKUPS: usize = 64;
/// JSON schema 标识。
pub const SCHEMA: &str = "xiyou/keydemand/v1";
/// 钥匙色序：`[y, b, r]`（黄/蓝/红）。
pub const KEY_COLORS: [&str; 3] = ["yellow", "blue", "red"];

const DIRS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

/// 层判定状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorStatus {
    /// 正常算出需求（含 0）。
    Ok,
    /// 该层无 `ty=8` 下行梯 ⇒ 没有入场格（**不回落到 `start`**）。
    NoEntry,
    /// 该层无 `ty=7` 上行梯 ⇒ 没有目标（f49/f50）。
    NoUpExit,
    /// 门数 > [`MAX_DOORS`] ⇒ 具名拒绝，不给数字。
    TooManyDoors,
    /// 连「怪物可清、条件门可过、钥匙无限」的乐观模型都到不了出口（硬阻挡切断）。
    Unreachable,
    /// 乐观模型可达，但入场钥匙网格 `0..=4` 内无解（需要 > 4 把）。
    NeedsMoreThanGrid,
    /// 防御性：拾取实体数 > [`MAX_PICKUPS`]（真实 54 层不会出现）。
    TooManyPickups,
}

impl FloorStatus {
    /// CLI/JSON 的稳定小写 tag。
    pub fn tag(self) -> &'static str {
        match self {
            FloorStatus::Ok => "ok",
            FloorStatus::NoEntry => "no_entry",
            FloorStatus::NoUpExit => "no_up_exit",
            FloorStatus::TooManyDoors => "too_many_doors",
            FloorStatus::Unreachable => "unreachable",
            FloorStatus::NeedsMoreThanGrid => "needs_more_than_grid",
            FloorStatus::TooManyPickups => "too_many_pickups",
        }
    }
}

/// 单层需求结论。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloorDemand {
    pub floor: i32,
    pub status: FloorStatus,
    /// 入场格（`ty=8` 首个槽位）；`None` ⇒ 该层无入场。
    pub entry: Option<(i32, i32)>,
    /// 该层全部 `ty=7` 出口格（去重、升序）。
    pub up_exits: Vec<(i32, i32)>,
    /// 门实体数（`ty=1|2|3`）。
    pub doors: usize,
    /// 钥匙拾取实体数（`ty=26|27|28`）。
    pub key_pickups: usize,
    /// 假设 2：按可通行处理的条件门数（`ty=83` + `ty=81/4`）。
    pub gates_ignored: usize,
    /// 假设 3：按可清处理的怪物数（cat8）。
    pub monsters_ignored: usize,
    /// 假设 4：按可站处理的火/陷阱数（`ty=6`）。
    pub hazards: usize,
    /// 引擎里硬阻挡的实体数（`ty=9/10/11/12` + cat16 商人 + cat32 联动门）。
    pub hard_blocks: usize,
    /// Pareto 最小的可行入场钥匙向量集（`[y,b,r]`，按 (total, 字典序) 升序）。
    pub pareto: Vec<[i32; 3]>,
    /// 代表向量（Pareto 集里 total 最小、再字典序最小）；无解/拒绝 ⇒ `None`。
    pub needed: Option<[i32; 3]>,
    /// `needed` 是否有分量顶到网格上界 [`MAX_ENTRY_KEYS`]（真需求可能更大）。
    pub grid_bound: bool,
    /// 本次判定真跑过的 BFS 次数（剪枝后）。
    pub bfs_runs: u32,
    /// 本次判定展开过的状态总数（诊断用）。
    pub states: u64,
}

impl FloorDemand {
    pub fn min_needed_y(&self) -> Option<i32> {
        self.needed.map(|v| v[0])
    }
    pub fn min_needed_b(&self) -> Option<i32> {
        self.needed.map(|v| v[1])
    }
    pub fn min_needed_r(&self) -> Option<i32> {
        self.needed.map(|v| v[2])
    }

    /// CLI 单行：`floor, entry, up_exits, needed(y,b,r), gates_ignored, monsters_ignored,
    /// hazards, status`。
    pub fn cli_line(&self) -> String {
        let entry = match self.entry {
            Some((x, y)) => format!("({x},{y})"),
            None => "-".to_string(),
        };
        let ups = if self.up_exits.is_empty() {
            "-".to_string()
        } else {
            self.up_exits
                .iter()
                .map(|(x, y)| format!("({x},{y})"))
                .collect::<Vec<_>>()
                .join("|")
        };
        let needed = match self.needed {
            Some([y, b, r]) => {
                let warn = if self.grid_bound { "!" } else { "" };
                format!("(y={y},b={b},r={r}){warn}")
            }
            None => "-".to_string(),
        };
        format!(
            "f{}, entry={}, up_exits={}, needed={}, gates_ignored={}, monsters_ignored={}, hazards={}, status={}",
            self.floor,
            entry,
            ups,
            needed,
            self.gates_ignored,
            self.monsters_ignored,
            self.hazards,
            self.status.tag()
        )
    }
}

/// 条件门策略（判据主线 = `Open`；`Blocked` 只用于文档里的敏感性对照，不进 CLI）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatePolicy {
    /// 假设 2：`ty=83` / `ty=81,4` 一律可通行（默认）。
    Open,
    /// 敏感性对照：条件门按阻挡处理（会低估可达性、抬高 `Unreachable`）。
    Blocked,
}

// ---------------------------------------------------------------- 格模型

#[derive(Debug, Clone, Default)]
struct Cell {
    passable: bool,
    doors: Vec<usize>,
    pickups: Vec<usize>,
}

struct Model {
    cols: i32,
    rows: i32,
    cells: Vec<Cell>,
    /// 门槽 → 色号（0=黄 1=蓝 2=红）
    door_color: Vec<usize>,
    /// 拾取槽 → 色号（0=黄 1=蓝 2=红）
    pickup_color: Vec<usize>,
    entry: Option<usize>,
    goals: Vec<usize>,
    gates_ignored: usize,
    monsters_ignored: usize,
    hazards: usize,
    hard_blocks: usize,
}

fn color_of_door(ty: i32) -> Option<usize> {
    match ty {
        DOOR_YELLOW => Some(0),
        DOOR_BLUE => Some(1),
        DOOR_RED => Some(2),
        _ => None,
    }
}

fn color_of_key(ty: i32) -> Option<usize> {
    match ty {
        KEY_YELLOW => Some(0),
        KEY_BLUE => Some(1),
        KEY_RED => Some(2),
        _ => None,
    }
}

/// 单层模型构建（纯函数，只读 `level`；`floor` 只喂 `entity_category_with_floor` 的层覆写表）。
fn build_model(floor: i32, level: &LevelSpec, policy: GatePolicy) -> Model {
    let cols = level.cols;
    let rows = level.rows;
    let n = (cols * rows).max(0) as usize;
    let mut cells: Vec<Cell> = vec![Cell::default(); n];
    let mut exempt = vec![false; n];
    let mut blocked = vec![false; n];

    let mut m = Model {
        cols,
        rows,
        cells: Vec::new(),
        door_color: Vec::new(),
        pickup_color: Vec::new(),
        entry: None,
        goals: Vec::new(),
        gates_ignored: 0,
        monsters_ignored: 0,
        hazards: 0,
        hard_blocks: 0,
    };

    for e in level.entities.iter() {
        let (x, y) = (e.x, e.y);
        if x < 0 || y < 0 || x >= cols || y >= rows {
            continue;
        }
        let ci = (y * cols + x) as usize;
        // 入口/出口与类目无关：只看 type 码（梯子锚点是全局常量）
        if e.ty == STAIR_UP && !m.goals.contains(&ci) {
            m.goals.push(ci);
        }
        if e.ty == STAIR_DOWN && m.entry.is_none() {
            m.entry = Some(ci);
        }

        match entity_category_with_floor(e.ty, floor) {
            1 => match e.ty {
                1 | 2 | 3 => {
                    let slot = m.door_color.len();
                    m.door_color.push(color_of_door(e.ty).expect("门色"));
                    cells[ci].doors.push(slot);
                }
                // 封印门（4 恒拦截 / 81 仅可见时拦截）与场景门 83：条件不建模 ⇒ 假设 2
                4 | 81 | 83 => {
                    m.gates_ignored += 1;
                    if policy == GatePolicy::Blocked {
                        blocked[ci] = true;
                    }
                }
                // 楼梯/火：stair_count ⇒ 豁免地形终审
                6 | 7 | 8 => {
                    exempt[ci] = true;
                    if e.ty == FIRE_TRAP {
                        m.hazards += 1;
                    }
                }
                // 9 炼丹台 / 10 三昧真火墙 / 11 墙 / 12 撞击墙：引擎 allowed=false
                9 | 10 | 11 | 12 => {
                    m.hard_blocks += 1;
                    blocked[ci] = true;
                }
                // 5 宝箱 / 76 / 82：bump 臂无 case ⇒ 放行
                _ => {}
            },
            2 | 4 => {
                if let Some(c) = color_of_key(e.ty) {
                    let slot = m.pickup_color.len();
                    m.pickup_color.push(c);
                    cells[ci].pickups.push(slot);
                }
                // 其余 cat2/4（含 **类型 32**）是捡取类，放行
            }
            8 => m.monsters_ignored += 1,
            // 告示牌/商人：触发即 allowed=false
            16 => {
                m.hard_blocks += 1;
                blocked[ci] = true;
            }
            // 联动门（72/84/87；72 在 f50 覆写为 cat8 怪）
            32 => {
                m.hard_blocks += 1;
                blocked[ci] = true;
            }
            _ => {}
        }
    }

    for y in 0..rows {
        for x in 0..cols {
            let ci = (y * cols + x) as usize;
            let terrain_ok = *level.terrain.get(ci).unwrap_or(&false);
            cells[ci].passable = (terrain_ok || exempt[ci]) && !blocked[ci];
        }
    }

    m.cells = cells;
    m.goals.sort_unstable();
    m.goals.dedup();
    m
}

// ---------------------------------------------------------------- 精确可达性

/// BFS：给定入场钥匙向量，判断乐观模型下能否从入场格走到任一出口。
///
/// `entry_keys = [y, b, r]`。返回 `(可达, 状态数)`。
fn search(m: &Model, entry: usize, entry_keys: [i32; 3]) -> (bool, u64) {
    let n = m.cells.len();
    let cols = m.cols;
    let rows = m.rows;
    let mut color_mask: [u128; 3] = [0; 3];
    for (slot, &c) in m.pickup_color.iter().enumerate() {
        color_mask[c] |= 1u128 << slot;
    }
    let mut door_mask: [u32; 3] = [0; 3];
    for (slot, &c) in m.door_color.iter().enumerate() {
        door_mask[c] |= 1u32 << slot;
    }

    // 入场格的拾取物即刻到手（人站在这格），入场格若是门格也按「站上去」先付费
    let mut p0: u128 = 0;
    for &slot in &m.cells[entry].pickups {
        p0 |= 1u128 << slot;
    }
    let mut o0: u32 = 0;
    for &d in &m.cells[entry].doors {
        let c = m.door_color[d];
        let held = entry_keys[c] + (p0 & color_mask[c]).count_ones() as i32
            - (o0 & door_mask[c]).count_ones() as i32;
        if held <= 0 {
            return (false, 0);
        }
        o0 |= 1u32 << d;
    }

    let mut goal = vec![false; n];
    for &ci in &m.goals {
        goal[ci] = true;
    }
    if goal[entry] {
        return (true, 0);
    }

    let mut visited: std::collections::HashSet<(u32, u128, u32)> = std::collections::HashSet::new();
    let mut q: VecDeque<(u32, u128, u32)> = VecDeque::new();
    visited.insert((entry as u32, p0, o0));
    q.push_back((entry as u32, p0, o0));
    let mut states: u64 = 1;

    while let Some((pos, picked, opened)) = q.pop_front() {
        let (x, y) = ((pos as i32) % cols, (pos as i32) / cols);
        for &(dx, dy) in &DIRS {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= cols || ny >= rows {
                continue;
            }
            let ni = (ny * cols + nx) as usize;
            if !m.cells[ni].passable {
                continue;
            }
            // 开门：同格多门按槽序依次付费（引擎同格反向槽序扫，付费总额与顺序无关）
            let mut no = opened;
            let mut denied = false;
            for &d in &m.cells[ni].doors {
                if no & (1u32 << d) != 0 {
                    continue;
                }
                let c = m.door_color[d];
                let held = entry_keys[c] + (picked & color_mask[c]).count_ones() as i32
                    - (no & door_mask[c]).count_ones() as i32;
                if held <= 0 {
                    denied = true;
                    break;
                }
                no |= 1u32 << d;
            }
            if denied {
                continue;
            }
            // 拾取：每实体只能捡一次（位图天然去重）
            let mut np = picked;
            for &slot in &m.cells[ni].pickups {
                np |= 1u128 << slot;
            }
            if goal[ni] {
                return (true, states);
            }
            if visited.insert((ni as u32, np, no)) {
                states += 1;
                q.push_back((ni as u32, np, no));
            }
        }
    }
    (false, states)
}

/// 入口格 + 出口格（**无 ty8 时 `entry=None`，绝不回落 `start`**）。
pub fn entry_and_up_exits(level: &LevelSpec) -> (Option<(i32, i32)>, Vec<(i32, i32)>) {
    let mut entry = None;
    let mut ups: Vec<(i32, i32)> = Vec::new();
    for e in level.entities.iter() {
        if e.ty == STAIR_DOWN && entry.is_none() {
            entry = Some((e.x, e.y));
        }
        if e.ty == STAIR_UP && !ups.contains(&(e.x, e.y)) {
            ups.push((e.x, e.y));
        }
    }
    ups.sort_unstable();
    (entry, ups)
}

/// 单点可达性（判据 1 的独立复核口）：`entry_keys=[y,b,r]`。
///
/// `floor` 缺省取 0（层覆写表对「能不能走」无影响：覆写只把怪物类目改成别的类目）。
pub fn reachable_with_entry(
    level: &LevelSpec,
    entry: (i32, i32),
    goals: &[(i32, i32)],
    entry_keys: [i32; 3],
) -> bool {
    reachable_with_entry_on(0, level, entry, goals, entry_keys)
}

/// 同 [`reachable_with_entry`]，但显式给层号（走层覆写表）。
pub fn reachable_with_entry_on(
    floor: i32,
    level: &LevelSpec,
    entry: (i32, i32),
    goals: &[(i32, i32)],
    entry_keys: [i32; 3],
) -> bool {
    if level.cols <= 0 || level.rows <= 0 {
        return false;
    }
    let cols = level.cols;
    let mut m = build_model(floor, level, GatePolicy::Open);
    if m.cells.is_empty() {
        return false;
    }
    let ci = (entry.1 * cols + entry.0) as usize;
    if entry.0 < 0 || entry.1 < 0 || ci >= m.cells.len() {
        return false;
    }
    m.goals = goals
        .iter()
        .map(|&(x, y)| (y * cols + x) as usize)
        .filter(|&i| i < m.cells.len())
        .collect();
    m.goals.sort_unstable();
    m.goals.dedup();
    if m.goals.is_empty() {
        return false;
    }
    search(&m, ci, entry_keys).0
}

/// 逐层算需求（默认 [`GatePolicy::Open`]）。
pub fn analyze_floor(floor: i32, level: &LevelSpec) -> FloorDemand {
    analyze_floor_with(floor, level, GatePolicy::Open)
}

/// 逐层算需求（可切换条件门策略；CLI 固定 `Open`）。
pub fn analyze_floor_with(floor: i32, level: &LevelSpec, policy: GatePolicy) -> FloorDemand {
    let m = build_model(floor, level, policy);
    let (entry_xy, up_exits) = entry_and_up_exits(level);
    let mut d = FloorDemand {
        floor,
        status: FloorStatus::Ok,
        entry: entry_xy,
        up_exits,
        doors: m.door_color.len(),
        key_pickups: m.pickup_color.len(),
        gates_ignored: m.gates_ignored,
        monsters_ignored: m.monsters_ignored,
        hazards: m.hazards,
        hard_blocks: m.hard_blocks,
        pareto: Vec::new(),
        needed: None,
        grid_bound: false,
        bfs_runs: 0,
        states: 0,
    };
    if d.doors > MAX_DOORS {
        d.status = FloorStatus::TooManyDoors;
        return d;
    }
    if d.key_pickups > MAX_PICKUPS {
        d.status = FloorStatus::TooManyPickups;
        return d;
    }
    if d.up_exits.is_empty() {
        d.status = FloorStatus::NoUpExit;
        return d;
    }
    let Some(entry) = m.entry else {
        d.status = FloorStatus::NoEntry;
        return d;
    };

    // 候选枚举：按 total 升序、再字典序；用可行/不可行的分量单调封闭双向剪枝。
    let mut cands: Vec<[i32; 3]> = Vec::new();
    for y in 0..=MAX_ENTRY_KEYS {
        for b in 0..=MAX_ENTRY_KEYS {
            for r in 0..=MAX_ENTRY_KEYS {
                cands.push([y, b, r]);
            }
        }
    }
    cands.sort_by_key(|v| (v[0] + v[1] + v[2], *v));

    let mut feasible: Vec<[i32; 3]> = Vec::new();
    let mut infeasible: Vec<[i32; 3]> = Vec::new();
    for v in cands {
        if feasible.iter().any(|f| le(f, &v)) {
            continue; // 已可行 f ≤ v ⇒ v 可行但非 Pareto 最小
        }
        if infeasible.iter().any(|u| le(&v, u)) {
            continue; // 已不可行 u ≥ v ⇒ v 不可行
        }
        d.bfs_runs += 1;
        let (ok, st) = search(&m, entry, v);
        d.states += st;
        if ok {
            feasible.push(v);
        } else {
            infeasible.push(v);
        }
    }

    if !feasible.is_empty() {
        let mut pareto: Vec<[i32; 3]> = feasible
            .iter()
            .copied()
            .filter(|v| !feasible.iter().any(|f| f != v && le(f, v) && !le(v, f)))
            .collect();
        pareto.sort_by_key(|v| (v[0] + v[1] + v[2], *v));
        d.grid_bound = pareto.iter().any(|v| v.iter().any(|&c| c >= MAX_ENTRY_KEYS));
        d.needed = pareto.first().copied();
        d.pareto = pareto;
        d.status = FloorStatus::Ok;
        return d;
    }

    // 网格内全不可行：无限钥匙探针区分「根本到不了」与「> 4 把」
    d.bfs_runs += 1;
    let (unbounded_ok, st) = search(&m, entry, [99, 99, 99]);
    d.states += st;
    d.status = if unbounded_ok {
        FloorStatus::NeedsMoreThanGrid
    } else {
        FloorStatus::Unreachable
    };
    d
}

fn le(a: &[i32; 3], b: &[i32; 3]) -> bool {
    a[0] <= b[0] && a[1] <= b[1] && a[2] <= b[2]
}

// ---------------------------------------------------------------- JSON

/// 逐层结果 → JSON（`meta.schema = xiyou/keydemand/v1`）。
pub fn to_json(floors: &[FloorDemand]) -> serde_json::Value {
    let rows: Vec<serde_json::Value> = floors
        .iter()
        .map(|d| {
            let entry = d
                .entry
                .map(|(x, y)| serde_json::json!([x, y]))
                .unwrap_or(serde_json::Value::Null);
            let needed = match d.needed {
                Some([y, b, r]) => serde_json::json!({
                    "y": y, "b": b, "r": r,
                    "yellow": y, "blue": b, "red": r,
                    "order": "(y,b,r)",
                }),
                None => serde_json::Value::Null,
            };
            serde_json::json!({
                "floor": d.floor,
                "status": d.status.tag(),
                "entry": entry,
                "up_exits": d.up_exits.iter().map(|(x, y)| serde_json::json!([x, y])).collect::<Vec<_>>(),
                "needed": needed,
                "min_needed_y": d.min_needed_y(),
                "min_needed_b": d.min_needed_b(),
                "min_needed_r": d.min_needed_r(),
                "pareto": d.pareto.iter().map(|v| serde_json::json!([v[0], v[1], v[2]])).collect::<Vec<_>>(),
                "grid_bound": d.grid_bound,
                "doors": d.doors,
                "key_pickups": d.key_pickups,
                "gates_ignored": d.gates_ignored,
                "monsters_ignored": d.monsters_ignored,
                "hazards": d.hazards,
                "hard_blocks": d.hard_blocks,
                "bfs_runs": d.bfs_runs,
                "states": d.states,
            })
        })
        .collect();
    serde_json::json!({
        "meta": {
            "schema": SCHEMA,
            "key_order": "y,b,r => yellow,blue,red",
            "entry_rule": "ty8 (STAIR_DOWN) 首个槽位；无 ty8 => no_entry（不回落到 start）",
            "exit_rule": "该层全部 ty7 (STAIR_UP)，到达任一即成功；无 ty7 => no_up_exit",
            "assumptions": [
                "只算走到任一上行梯；不算支线/宝箱/商店路上的钥匙消耗",
                "条件门 ty83 与封印门 ty81/4 的开启条件不建模，按可通行（gates_ignored 计数）",
                "怪物一律假定可清除，不扣血不耗时（monsters_ignored 计数）",
                "火 ty6 按引擎豁免可站（hazards 计数）；三昧真火墙 ty10 属硬阻挡（hard_blocks）",
                "硬阻挡 ty9/10/11/12 + cat16 商人 + cat32 联动门按永久不可通行（hard_blocks 计数）⇒ 需求可能偏高，极端时整层 unreachable（f29）",
            ],
            "grid": {"max_entry_keys_per_color": MAX_ENTRY_KEYS, "max_doors": MAX_DOORS},
        },
        "floors": rows,
    })
}
