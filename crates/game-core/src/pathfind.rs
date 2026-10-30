//! findPath findPath 忠实移植 —— best-first 搜索 + 父指针回溯。
//!
//! 证据：reference/src/deobf/a.java:9869-9928（findPath）+ relax_path_neighbors（:9908-9936）。
//! 忠实点（含原版怪癖）：
//! - 优先值 cost = |x-dst| + |y-dst| + |x-src| + |y-src|；
//! - 插入循环 `while (--table_count >= 0) { if cost < arr[table_count] || table_count == 0 {…break} }`
//!   未命中时**不插入但 f_int_149 仍自增**（原版怪癖，逐行复刻）；
//! - 扩展上限 `++table_count < 100`；
//! - 回溯方向映射（由父指针差分，目标→起点反向行走）：dx=-1→2 / dx=+1→3 /
//!   dy=-w→0 / dy=+w→1；
//! - 可通行判定 = isCellWalkable(terrain) || can_path_through_entity(格有实体槽)。

use crate::enums::Dir;

/// 方向码常量复用 enums::Dir（复核 R12/R5：消灭双真相源）。
pub const DIR_UP: u8 = Dir::Up as u8;
pub const DIR_DOWN: u8 = Dir::Down as u8;
pub const DIR_RIGHT: u8 = Dir::Right as u8;
pub const DIR_LEFT: u8 = Dir::Left as u8;

pub struct Walkability<'g> {
    /// walkableGrid[y][x]：地形可通行（rebuildWalkability 由 maplv tile 表构建）
    pub terrain: Vec<Vec<bool>>,
    /// can_path_through_entity：格有实体槽（容量 > 0）亦可通行
    pub has_entity_slot: &'g dyn Fn(i32, i32) -> bool,
    pub cols: i32,
    pub rows: i32,
}

impl Walkability<'_> {
    fn is_cell_walkable(&self, x: i32, y: i32) -> bool {
        if x >= 0 && x < self.cols && y >= 0 && y < self.rows {
            self.terrain[y as usize][x as usize]
        } else {
            false
        }
    }

    /// Java 反混淆原名 `canPathThroughEntity`（对照性锚——P0 命名批 2026-10-19 机械改名，语义不变）。
    fn can_path_through_entity(&self, x: i32, y: i32) -> bool {
        if x >= 0 && x < self.cols && y >= 0 && y < self.rows {
            (self.has_entity_slot)(x, y)
        } else {
            false
        }
    }
}

/// findPath 返回值：found + 步数序列（缓冲序 = 目标端优先，游戏自末端消费）。
pub struct PathResult {
    pub found: bool,
    pub steps: Vec<u8>,
}

pub fn find_path(walk: &Walkability, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> PathResult {
    let cols = walk.cols;
    let start_idx = from_x + from_y * cols;
    let src_x = from_x as i16;
    let src_y = from_y as i16;
    let dst_x = to_x as i16;
    let dst_y = to_y as i16;

    // findPath 头部：全量重建工作数组（Java 每次调用 new）
    let mut visited = vec![vec![0u8; cols as usize]; walk.rows as usize]; // f_byte_arr2_06
    let mut parent = vec![vec![0i16; cols as usize]; walk.rows as usize]; // f_short_arr2_00
    let mut prio: Vec<i16> = vec![0; 100]; // f_short_arr_13
    let mut queue: Vec<i16> = vec![0; 100]; // f_short_arr_14
    let mut open_count; // f_int_149（Java 初始 1：零填充队列 + 计数自增）

    let mut cur_x = from_x;
    let mut cur_y = from_y;
    let mut cur_idx: i32 = start_idx;
    // Java 头部：数组 new short[100]（零填充）+ f_int_149 = 1 —— 起点**不入队**，
    // 队头 [0,0] 是幻影元素（排序尾部 = best-first 弹出，幻影头最后才被消费，
    // 其邻格扩展为墙时无副作用）。逐行复刻，勿"修正"为起点入队。
    prio[0] = 0;
    queue[0] = 0;
    open_count = 1;

    let mut found = false;
    let mut expansions = 0i32;

    loop {
        // relax_path_neighbors × 4（左、右、上、下——Java 顺序）
        relax_path_neighbors(walk, &mut visited, &mut parent, &mut prio, &mut queue, &mut open_count,
            cur_x - 1, cur_y, cur_idx, dst_x, dst_y, src_x, src_y, cols);
        relax_path_neighbors(walk, &mut visited, &mut parent, &mut prio, &mut queue, &mut open_count,
            cur_x + 1, cur_y, cur_idx, dst_x, dst_y, src_x, src_y, cols);
        relax_path_neighbors(walk, &mut visited, &mut parent, &mut prio, &mut queue, &mut open_count,
            cur_x, cur_y - 1, cur_idx, dst_x, dst_y, src_x, src_y, cols);
        relax_path_neighbors(walk, &mut visited, &mut parent, &mut prio, &mut queue, &mut open_count,
            cur_x, cur_y + 1, cur_idx, dst_x, dst_y, src_x, src_y, cols);

        if open_count <= 0 {
            break;
        }

        cur_idx = queue[(open_count - 1) as usize] as i32;
        cur_x = cur_idx % cols;
        cur_y = cur_idx / cols;
        open_count -= 1;
        visited[cur_y as usize][cur_x as usize] = 2;

        if cur_x as i16 == dst_x && cur_y as i16 == dst_y {
            found = true;
        }

        expansions += 1;
        if expansions >= 100 || found {
            break;
        }
    }

    // 回溯重建（父指针差分 → 方向码）
    let mut steps: Vec<u8> = Vec::new();
    if found {
        let mut cur_x = cur_x;
        let mut cur_y = cur_y;
        let mut cur_idx = cur_x + cur_y * cols;
        loop {
            let parent_idx = parent[cur_y as usize][cur_x as usize] as i32;
            let diff = parent_idx - cur_idx;
            let dir = match diff {
                -1 => DIR_RIGHT, // 父在左 ⇒ 行进向右
                1 => DIR_LEFT,
                d if d == -cols => DIR_DOWN,
                d if d == cols => DIR_UP,
                // Java `return false`（a.java:9943-9944）：walkStepCount 已填的前缀
                // **保留**（findPath 写的是成员缓冲）——MOV 五参不判返回值、openWallOnRoute
                // 照样消费前缀，故部分步序原样带出。可达性（B 级）：回溯只会在父
                // 指针落到未入队格时失败，唯幻影队头 (0,0)；而 Java 弹出幻影后扩展
                // relax_path_neighbors(-1,0) 先抛 AIOOBE（本移植 relax_path_neighbors 越界早退不崩）——Java 侧不可达
                _ => return PathResult { found: false, steps },
            };
            steps.push(dir);
            cur_idx = parent_idx;
            cur_x = cur_idx % cols;
            cur_y = cur_idx / cols;
            if parent_idx == start_idx || cur_idx == start_idx {
                break;
            }
        }
        // Java 循环条件 `while (var14 != py_key)`：父 != 起点；最后一步（父==起点）
        // 的方向已在上轮 push。上述 push 顺序即行走顺序（起点→目标）。
    }

    PathResult { found, steps }
}

/// relax_path_neighbors 忠实移植：邻格入开放队列（优先插入，含"未插入也计数"怪癖）。
#[allow(clippy::too_many_arguments)]
fn relax_path_neighbors(
    walk: &Walkability,
    visited: &mut [Vec<u8>],
    parent: &mut [Vec<i16>],
    prio: &mut [i16],
    queue: &mut [i16],
    open_count: &mut i32,
    x: i32,
    y: i32,
    came_from: i32,
    dst_x: i16,
    dst_y: i16,
    src_x: i16,
    src_y: i16,
    cols: i32,
) {
    if x < 0 || y < 0 || x >= cols || y >= walk.rows {
        return; // Java 数组访问越界在此场景被行列界内保证（relax_path_neighbors 调用点邻格）
    }
    let dx16 = x as i16;
    let dy16 = y as i16;
    if visited[dy16 as usize][dx16 as usize] == 0
        && (walk.is_cell_walkable(dx16 as i32, dy16 as i32) || walk.can_path_through_entity(dx16 as i32, dy16 as i32))
    {
        visited[dy16 as usize][dx16 as usize] = 1;
        parent[dy16 as usize][dx16 as usize] = came_from as i16;
        let cost = (dx16 - dst_x).abs()
            + (dy16 - dst_y).abs()
            + (dx16 - src_x).abs()
            + (dy16 - src_y).abs();
        let mut table_count = *open_count;

        loop {
            table_count -= 1;
            if table_count < 0 {
                break;
            }
            if cost < prio[table_count as usize] || table_count == 0 {
                let mut open_n = *open_count;
                while open_n > table_count + 1 {
                    prio[open_n as usize] = prio[(open_n - 1) as usize];
                    queue[open_n as usize] = queue[(open_n - 1) as usize];
                    open_n -= 1;
                }
                table_count += 1;
                prio[table_count as usize] = cost;
                queue[table_count as usize] = dx16 + dy16 * cols as i16;
                break;
            }
        }

        *open_count += 1; // 原版怪癖：未命中插入也自增（逐行复刻）
    }
}

// ───────────────────────────────────────── T2 策略层（touch-native-undo §0 行为政策）

/// 探步入（一步撞击尝试）：自停靠格向目标墙格一步——暗墙命中=进入/揭示
/// （挂 openWallOnRoute a.java:8738-8870 / 撞击臂型 10/11/12），实墙=撞停。
/// 暗墙双通道（方向键踏步/触屏点墙滑动）由 T3 发起，本结构为 core 端探步接口。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeStep {
    /// 停靠格（探步起点）
    pub from: (i32, i32),
    /// 方向码（0 下 1 上 2 右 3 左——walkPathBuffer 同语义）
    pub dir: u8,
    /// 探入格（目标墙格）
    pub into: (i32, i32),
}

/// 寻路计划（策略层产物：直达/停靠/探步入）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutePlan {
    /// 步序（walkPathBuffer 语义：目标端优先，自末端消费）
    pub steps: Vec<u8>,
    /// 终位（直达=目标格；停靠=实际终点）
    pub dock: (i32, i32),
    /// 直达目标格
    pub reached: bool,
    /// 墙格目标的探步入
    pub probe: Option<ProbeStep>,
}

/// 目标→邻格方向码（0 下 1 上 2 右 3 左）。
fn dir_between(from: (i32, i32), into: (i32, i32)) -> u8 {
    match (into.0 - from.0, into.1 - from.1) {
        (0, 1) => 0,
        (0, -1) => 1,
        (1, 0) => 2,
        (-1, 0) => 3,
        d => panic!("非 4 邻接：{from:?}→{into:?}（{d:?}）"),
    }
}

impl Walkability<'_> {
    /// 可走格（terrain 面；实体槽/门/敌格可入=路径面语义，碰撞语义复用 tryStep）。
    pub fn walkable(&self, x: i32, y: i32) -> bool {
        self.is_cell_walkable(x, y)
    }
}

/// 寻路计划（touch-native 操作模型，plan §0）：
/// - 目标可走且可达 = 直达（原版 findPath 步序——零发明保真）；
/// - 目标可走不可达 = **朝目标方向最远可达点停靠**（投影最远→距目标最近→行主序）；
/// - 目标为墙 = **最近相邻格停靠 + 探步入**（一步撞击尝试）；相邻全不可达=退化停靠。
pub fn plan_route(walk: &Walkability, from: (i32, i32), to: (i32, i32)) -> RoutePlan {
    if from == to {
        return RoutePlan { steps: Vec::new(), dock: from, reached: true, probe: None };
    }
    if walk.walkable(to.0, to.1) {
        let direct = find_path(walk, from.0, from.1, to.0, to.1);
        if direct.found {
            return RoutePlan { steps: direct.steps, dock: to, reached: true, probe: None };
        }
        // 不可达可走格：朝目标方向最远可达点停靠
        let dock = furthest_reachable_toward(walk, from, to);
        let steps = if dock == from {
            Vec::new()
        } else {
            find_path(walk, from.0, from.1, dock.0, dock.1).steps
        };
        return RoutePlan { steps, dock, reached: false, probe: None };
    }
    // 墙格目标：最近相邻（路径最短）+探步入；相邻全不可达=退化为朝目标停靠
    let mut best: Option<((i32, i32), usize)> = None;
    for (ax, ay) in [(to.0, to.1 + 1), (to.0, to.1 - 1), (to.0 + 1, to.1), (to.0 - 1, to.1)] {
        if !walk.walkable(ax, ay) {
            continue;
        }
        let len = if (ax, ay) == from {
            0
        } else {
            let r = find_path(walk, from.0, from.1, ax, ay);
            if !r.found {
                continue;
            }
            r.steps.len()
        };
        let better = match best {
            None => true,
            Some((_, blen)) => len < blen,
        };
        if better {
            best = Some(((ax, ay), len));
        }
    }
    if let Some((adj, _)) = best {
        let steps = if adj == from {
            Vec::new()
        } else {
            find_path(walk, from.0, from.1, adj.0, adj.1).steps
        };
        return RoutePlan {
            steps,
            dock: adj,
            reached: false,
            probe: Some(ProbeStep { from: adj, dir: dir_between(adj, to), into: to }),
        };
    }
    let dock = furthest_reachable_toward(walk, from, to);
    let steps = if dock == from {
        Vec::new()
    } else {
        find_path(walk, from.0, from.1, dock.0, dock.1).steps
    };
    RoutePlan { steps, dock, reached: false, probe: None }
}

/// 朝目标方向最远可达点（保真口径=可达即 find_path 命中，扩展上限 100 同引擎）。
/// 排序：投影 (cell-start)·(target-start) 最大 → 距目标曼哈顿最小 → 行主序最小。
fn furthest_reachable_toward(walk: &Walkability, from: (i32, i32), to: (i32, i32)) -> (i32, i32) {
    let (fx, fy) = (to.0 - from.0, to.1 - from.1);
    let mut best: Option<(i32, i32, i32, i32)> = None; // (投影, -距目标, -行主序, 编码)
    let mut consider = |cell: (i32, i32), proj: i32, dist: i32, ord: i32| {
        let better = match best {
            None => true,
            Some((bp, bd, bo, _)) => (proj, -dist, -ord) > (bp, bd, bo),
        };
        if better {
            best = Some((proj, -dist, -ord, 0));
            Some(cell)
        } else {
            None
        }
    };
    let mut pick = from;
    {
        // 起点恒可达（投影 0）
        if let Some(c) = consider(from, 0, (to.0 - from.0).abs() + (to.1 - from.1).abs(), 0) {
            pick = c;
        }
        for y in 0..walk.rows {
            for x in 0..walk.cols {
                if (x, y) == from || !walk.walkable(x, y) {
                    continue;
                }
                if !find_path(walk, from.0, from.1, x, y).found {
                    continue;
                }
                let proj = (x - from.0) * fx + (y - from.1) * fy;
                let dist = (x - to.0).abs() + (y - to.1).abs();
                let ord = y * walk.cols + x;
                if let Some(c) = consider((x, y), proj, dist, ord) {
                    pick = c;
                }
            }
        }
    }
    pick
}

// ───────────────────────────────────────── RouteTracker（T1 journal 挂点契约执行面）

use crate::undo_journal::{CombatStroke, MoveStroke, UndoJournal};

/// WalkTo 路段追踪器：位移笔=段起终点（**中断/挂起=实际终点**）、战斗笔=终结时
/// ——plan §2 入栈时机契约 + task-11"挂起→journal 挂点→续走"。
/// 实际终点 = 玩家**最终落格**：步在途时打断，计入在途步的落格（那一步必然走完，
/// 由调用方 `GameScene::route_landing_cell` 给出；STAGE-B-008 复核 R1/R2/R5）。
#[derive(Debug, Default)]
pub struct RouteTracker {
    seg_from: (i32, i32),
    seg_open: bool,
    suspended: bool,
}

impl RouteTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// 挂起中（事件处理期——T2 呼吸暂停续走判定）。
    pub fn is_suspended(&self) -> bool {
        self.suspended
    }

    /// 段开放（WalkTo/探步寻路段进行中——press_key 打断判定）。
    pub fn is_open(&self) -> bool {
        self.seg_open
    }

    /// 开段（WalkTo 发起/续走）。
    pub fn begin(&mut self, from: (i32, i32)) {
        self.seg_from = from;
        self.seg_open = true;
        self.suspended = false;
    }

    /// 续走（挂起解除后新段自事件格起）。
    pub fn resume(&mut self, at: (i32, i32)) {
        self.begin(at);
    }

    /// 段终位（整段起终点入栈）。
    pub fn arrive(&mut self, at: (i32, i32), journal: &mut UndoJournal) {
        self.close(at, journal);
    }

    /// 事件挂起（撞敌/拾取/开门——段收于**实际终点**=事件格，挂起）。
    pub fn suspend(&mut self, at: (i32, i32), journal: &mut UndoJournal) {
        self.close(at, journal);
        self.suspended = true;
    }

    /// 战斗终结才入栈（T1 挂点契约），挂起解除。
    pub fn finish_combat(&mut self, stroke: CombatStroke, journal: &mut UndoJournal) {
        journal.finish_combat(stroke);
        self.suspended = false;
    }

    /// 打断（任意输入）：段收于**实际终点**（步在途时 = 在途步的落格，调用方负责计入）。
    pub fn interrupt(&mut self, at: (i32, i32), journal: &mut UndoJournal) {
        self.close(at, journal);
        self.suspended = false;
    }

    fn close(&mut self, at: (i32, i32), journal: &mut UndoJournal) {
        if self.seg_open {
            // 【STAGE-B-008】零长段（起点 = 终点：撞墙 / 撞门 / 紧邻开战）不入栈——撤销不多一次看不见效果的空操作
            if self.seg_from != at {
                journal.push_move(MoveStroke { from: self.seg_from, to: at });
            }
            self.seg_open = false;
        }
    }
}
