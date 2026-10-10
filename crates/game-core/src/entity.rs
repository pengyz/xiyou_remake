//! 实体表与格子索引 —— Java m_048（生成）/ m_100（格查）/ m_047（移除）的忠实移植。
//!
//! 证据：reference/src/deobf/a.java
//! - m_048 :6348（像素取整、双数组登记、f_byte_15 格类型分配、楼梯锚点）
//! - m_100 :8574 附近（格类型→容量→槽位表查实体，跳过 solid==1）
//! - m_047 :6308（solid=1 + 类目分派视觉字段）
//! - 宽度表 f_byte_arr_07 :589（88 项）+ m_044 特例 :6250-6262（69→96px、72→32px）

/// 实体表（100 槽，Java god class 的实体数组族）。
pub struct EntityTable {
    pub entity_type: Vec<i32>,
    pub param: Vec<i16>,
    pub pixel_x: Vec<i32>,
    pub pixel_y: Vec<i32>,
    /// f_byte_arr_04：solid 标志（1=已移除/不可交互——Java 复用为"已移除"）
    pub solid: Vec<u8>,
    /// f_bool_arr_00：隐藏标志
    pub hidden: Vec<bool>,
    /// f_bool_arr_01：移除标志（DES/GUT 消费用）
    pub removed: Vec<bool>,
    /// f_int_arr_12：路线/动画参数（MOV 72 型写 1；m_047 按类目写）
    pub route_or_anim: Vec<i32>,
    pub count: usize,
    /// 楼梯锚点（m_048：type 7→f_int_70/72，type 8→f_int_71/73）
    pub stair_up: Option<(i32, i32)>,
    pub stair_down: Option<(i32, i32)>,
}

impl EntityTable {
    pub fn new(capacity: usize) -> Self {
        Self {
            entity_type: vec![0; capacity],
            param: vec![0; capacity],
            pixel_x: vec![0; capacity],
            pixel_y: vec![0; capacity],
            solid: vec![0; capacity],
            hidden: vec![false; capacity],
            removed: vec![false; capacity],
            route_or_anim: vec![0; capacity],
            count: 0,
            stair_up: None,
            stair_down: None,
        }
    }
}

/// 格子索引（m_059 初始化 + m_048 增量登记）。
pub struct CellGrid {
    pub cols: i32,
    pub rows: i32,
    /// f_byte_arr2_02[y][x]：格类型（0=未分配，由 m_048 分配递增新类型）
    pub cell_type: Vec<Vec<u8>>,
    /// f_byte_arr_10[格类型]：容量
    pub capacity: Vec<u8>,
    /// f_byte_arr2_03[格类型][槽]：实体 id+1
    pub slots: Vec<Vec<u8>>,
    /// f_byte_15：已分配格类型数
    next_cell_type: u8,
}

impl CellGrid {
    pub fn new(cols: i32, rows: i32) -> Self {
        Self {
            cols,
            rows,
            cell_type: vec![vec![0u8; cols as usize]; rows as usize],
            capacity: vec![0u8; 128],
            slots: vec![vec![0u8; 16]; 128],
            next_cell_type: 0,
        }
    }
}

/// 类型绘制宽度表（Java f_byte_arr_07，a.java:589，88 项；夹具 type_width_px.txt）。
/// m_044 特例：type 69 → 96px（3 格）、type 72 → 32px（1 格）；m_045：类型
/// 1/2/8/16/32 → 27px（27>>5 = 0 ⇒ ≤0 取 1 格）。其余 = 表值 >> 5，≤0 取 1。
pub fn width_cells(type_id: i32, width_table: &[i32]) -> i32 {
    let px = match type_id {
        69 => 96,
        72 => 32,
        _ => width_table.get(type_id as usize).copied().unwrap_or(32),
    };
    let cells = px >> 5;
    if cells <= 0 { 1 } else { cells }
}

/// m_048 忠实移植：像素取整到格、实体字段初始化、格槽登记（按类型宽度
/// `width_cells` 逐格，复核 R4：type 69 = 3 格）、楼梯锚点。
/// 返回新实体索引。
pub fn spawn(
    table: &mut EntityTable,
    grid: &mut CellGrid,
    type_id: i32,
    px: i32,
    py: i32,
    param: i16,
    width_table: &[i32],
) -> usize {
    let idx = table.count;
    let cell_x = (px + 16) >> 5;
    let cell_y = (py + 16) >> 5;
    let snapped_x = cell_x << 5;
    let snapped_y = cell_y << 5;

    table.entity_type[idx] = type_id;
    table.pixel_x[idx] = snapped_x;
    table.pixel_y[idx] = snapped_y;
    table.removed[idx] = false;
    table.hidden[idx] = true; // m_048 默认 hidden=true（可见化由 switch 决定）
    table.param[idx] = param;
    table.solid[idx] = 0;
    table.count += 1;

    // 格槽登记（Java var5 = m_044 写入的 f_int_arr_08>>5，≤0 取 1；a.java:6351）
    let width = width_cells(type_id, width_table);
    for dx in 0..width {
        let cx = (cell_x + dx) as usize;
        let mut ct = grid.cell_type[cell_y as usize][cx];
        if ct == 0 {
            grid.next_cell_type += 1;
            ct = grid.next_cell_type;
            grid.cell_type[cell_y as usize][cx] = ct;
        }
        let cap = grid.capacity[ct as usize];
        grid.slots[ct as usize][cap as usize] = (idx + 1) as u8;
        grid.capacity[ct as usize] += 1;
    }

    if type_id == crate::enums::entity_kind::STAIR_UP {
        table.stair_up = Some((snapped_x, snapped_y));
    } else if type_id == crate::enums::entity_kind::STAIR_DOWN {
        table.stair_down = Some((snapped_x, snapped_y));
    }
    idx
}

/// m_100 忠实移植：格类型→容量→槽位查实体；跳过 solid==1（已移除）。
pub fn cell_entity(grid: &CellGrid, table: &EntityTable, x: i32, y: i32, type_id: i32) -> i32 {
    if x < 0 || y < 0 || x >= grid.cols || y >= grid.rows {
        return -1;
    }
    let ct = grid.cell_type[y as usize][x as usize] as usize;
    let cap = grid.capacity[ct] as usize;
    let mut found;
    for slot in 0..cap {
        found = grid.slots[ct][slot] as i32 - 1;
        if table.entity_type[found as usize] == type_id && table.solid[found as usize] != 1 {
            return found;
        }
    }
    -1
}

/// m_047 核心：solid=1（移除）。视觉字段分派（f_int_arr_08/09/12/10 按类目）随
/// P3.3 战斗/渲染考证补全——当前调用方（脚本 DES_/战斗）不读这些字段。
pub fn remove(table: &mut EntityTable, idx: usize) {
    table.solid[idx] = 1;
}

/// DES 负参变体：按类型批量移除（executeScriptInstruction DES 分支语义）。
pub fn remove_all_of_type(table: &mut EntityTable, type_id: i32) {
    for i in 0..table.count {
        if table.entity_type[i] == type_id {
            table.solid[i] = 1;
        }
    }
}
