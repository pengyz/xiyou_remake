//! m_133 findPath 忠实移植 —— best-first 搜索 + 父指针回溯。
//!
//! 证据：reference/src/deobf/a.java:9840-9905（findPath）+ m_134（:9908-9936）。
//! 忠实点（含原版怪癖）：
//! - 优先值 cost = |x-dst| + |y-dst| + |x-src| + |y-src|；
//! - 插入循环 `while (--var3 >= 0) { if cost < arr[var3] || var3 == 0 {…break} }`
//!   未命中时**不插入但 f_int_149 仍自增**（原版怪癖，逐行复刻）；
//! - 扩展上限 `++var3 < 100`；
//! - 回溯方向映射（由父指针差分，目标→起点反向行走）：dx=-1→2 / dx=+1→3 /
//!   dy=-w→0 / dy=+w→1；
//! - 可通行判定 = isCellWalkable(terrain) || m_062(格有实体槽)。

use crate::enums::Dir;

/// 方向码常量复用 enums::Dir（复核 R12/R5：消灭双真相源）。
pub const DIR_UP: u8 = Dir::Up as u8;
pub const DIR_DOWN: u8 = Dir::Down as u8;
pub const DIR_RIGHT: u8 = Dir::Right as u8;
pub const DIR_LEFT: u8 = Dir::Left as u8;

pub struct Walkability<'g> {
    /// f_bool_arr2_00[y][x]：地形可通行（m_061 由 maplv tile 表构建）
    pub terrain: Vec<Vec<bool>>,
    /// m_062：格有实体槽（容量 > 0）亦可通行
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

    fn m_062(&self, x: i32, y: i32) -> bool {
        if x >= 0 && x < self.cols && y >= 0 && y < self.rows {
            (self.has_entity_slot)(x, y)
        } else {
            false
        }
    }
}

/// m_133 返回值：found + 步数序列（缓冲序 = 目标端优先，游戏自末端消费）。
pub struct PathResult {
    pub found: bool,
    pub steps: Vec<u8>,
}

pub fn find_path(walk: &Walkability, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> PathResult {
    let cols = walk.cols;
    let start_idx = (from_x + from_y * cols) as i32;
    let src_x = from_x as i16;
    let src_y = from_y as i16;
    let dst_x = to_x as i16;
    let dst_y = to_y as i16;

    // m_133 头部：全量重建工作数组（Java 每次调用 new）
    let mut visited = vec![vec![0u8; cols as usize]; walk.rows as usize]; // f_byte_arr2_06
    let mut parent = vec![vec![0i16; cols as usize]; walk.rows as usize]; // f_short_arr2_00
    let mut prio: Vec<i16> = vec![0; 100]; // f_short_arr_13
    let mut queue: Vec<i16> = vec![0; 100]; // f_short_arr_14
    let mut open_count = 1i32; // f_int_149（Java 初始 1：零填充队列 + 计数自增）

    let mut var6 = from_x;
    let mut var7 = from_y;
    let mut var9: i32 = start_idx;
    // Java 头部：数组 new short[100]（零填充）+ f_int_149 = 1 —— 起点**不入队**，
    // 队头 [0,0] 是幻影元素（排序尾部 = best-first 弹出，幻影头最后才被消费，
    // 其邻格扩展为墙时无副作用）。逐行复刻，勿"修正"为起点入队。
    prio[0] = 0;
    queue[0] = 0;
    open_count = 1;

    let mut found = false;
    let mut expansions = 0i32;

    loop {
        // m_134 × 4（左、右、上、下——Java 顺序）
        m_134(walk, &mut visited, &mut parent, &mut prio, &mut queue, &mut open_count,
            var6 - 1, var7, var9, dst_x, dst_y, src_x, src_y, cols);
        m_134(walk, &mut visited, &mut parent, &mut prio, &mut queue, &mut open_count,
            var6 + 1, var7, var9, dst_x, dst_y, src_x, src_y, cols);
        m_134(walk, &mut visited, &mut parent, &mut prio, &mut queue, &mut open_count,
            var6, var7 - 1, var9, dst_x, dst_y, src_x, src_y, cols);
        m_134(walk, &mut visited, &mut parent, &mut prio, &mut queue, &mut open_count,
            var6, var7 + 1, var9, dst_x, dst_y, src_x, src_y, cols);

        if open_count <= 0 {
            break;
        }

        var9 = queue[(open_count - 1) as usize] as i32;
        var6 = var9 % cols;
        var7 = var9 / cols;
        open_count -= 1;
        visited[var7 as usize][var6 as usize] = 2;

        if var6 as i16 == dst_x && var7 as i16 == dst_y {
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
        let mut cur_x = var6;
        let mut cur_y = var7;
        let mut cur_idx = cur_x + cur_y * cols;
        loop {
            let parent_idx = parent[cur_y as usize][cur_x as usize] as i32;
            let diff = parent_idx - cur_idx;
            steps.push(match diff {
                -1 => DIR_RIGHT, // 父在左 ⇒ 行进向右
                1 => DIR_LEFT,
                d if d == -cols => DIR_DOWN,
                d if d == cols => DIR_UP,
                _ => return PathResult { found: false, steps: Vec::new() }, // Java: return false
            });
            cur_idx = parent_idx;
            cur_x = cur_idx % cols;
            cur_y = cur_idx / cols;
            if parent_idx == start_idx || cur_idx == start_idx {
                break;
            }
        }
        // Java 循环条件 `while (var14 != var1)`：父 != 起点；最后一步（父==起点）
        // 的方向已在上轮 push。上述 push 顺序即行走顺序（起点→目标）。
    }

    PathResult { found, steps }
}

/// m_134 忠实移植：邻格入开放队列（优先插入，含"未插入也计数"怪癖）。
#[allow(clippy::too_many_arguments)]
fn m_134(
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
        return; // Java 数组访问越界在此场景被行列界内保证（m_134 调用点邻格）
    }
    let dx16 = x as i16;
    let dy16 = y as i16;
    if visited[dy16 as usize][dx16 as usize] == 0
        && (walk.is_cell_walkable(dx16 as i32, dy16 as i32) || walk.m_062(dx16 as i32, dy16 as i32))
    {
        visited[dy16 as usize][dx16 as usize] = 1;
        parent[dy16 as usize][dx16 as usize] = came_from as i16;
        let cost = ((dx16 - dst_x).abs()
            + (dy16 - dst_y).abs()
            + (dx16 - src_x).abs()
            + (dy16 - src_y).abs()) as i16;
        let mut var3 = *open_count;

        loop {
            var3 -= 1;
            if var3 < 0 {
                break;
            }
            if cost < prio[var3 as usize] || var3 == 0 {
                let mut var5 = *open_count;
                while var5 > var3 + 1 {
                    prio[var5 as usize] = prio[(var5 - 1) as usize];
                    queue[var5 as usize] = queue[(var5 - 1) as usize];
                    var5 -= 1;
                }
                var3 += 1;
                prio[var3 as usize] = cost;
                queue[var3 as usize] = (dx16 + dy16 * cols as i16) as i16;
                break;
            }
        }

        *open_count += 1; // 原版怪癖：未命中插入也自增（逐行复刻）
    }
}
