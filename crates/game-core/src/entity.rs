//! 实体表与格子索引 —— Java spawnEntity（生成）/ m_100（格查）/ markEntityRemoved（移除）的忠实移植。
//!
//! 证据：reference/src/deobf/a.java
//! - spawnEntity :6363（像素取整、双数组登记、nextCellTypeId 格类型分配、楼梯锚点）
//! - m_100 :8574 附近（格类型→容量→槽位表查实体，跳过 solid==1）
//! - markEntityRemoved :6323（solid=1 + 类目分派视觉字段）
//! - 宽度表 typeFrameWidth :589（88 项）+ dispatchSpriteSize 特例 :6274-6299（69→96px、72→32px）

/// 实体表（100 槽，Java god class 的实体数组族）。
pub struct EntityTable {
    pub entity_type: Vec<i32>,
    pub param: Vec<i16>,
    pub pixel_x: Vec<i32>,
    pub pixel_y: Vec<i32>,
    /// entityState：实体状态（0=常态；1=markEntityRemoved 移除态〔开格动画〕；2=markEntityDesRemoved DES 态）
    pub solid: Vec<u8>,
    /// entityVisible：可见标志（spawnEntity 默认 true；m_122 的 5/81/6/12/76/82/83 生成分支置 false）
    pub visible: Vec<bool>,
    /// entityRemoved：移除标志（DES/GUT/pullEntityFromCell 消耗用；sortEntitiesByY 排序跳过、paintEntityLayer 不画）
    pub removed: Vec<bool>,
    /// entitySpriteW：精灵条带帧宽 px（dispatchSpriteSize 分派）
    pub sprite_w: Vec<i32>,
    /// entitySpriteH：精灵条带高 px（dispatchSpriteSize 分派）
    pub sprite_h: Vec<i32>,
    /// entityAnimRow：动画偏移表行号（dispatchAnimFields = animRowByType[type]）
    pub anim_idx: Vec<i32>,
    /// entityAnimFrame：动画帧下标（advanceEntityFrames 推进）
    pub frame: Vec<i32>,
    pub count: usize,
    /// 楼梯锚点（spawnEntity：type 7→stairUpMarkerX/72，type 8→stairDownMarkerX/73）
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
            visible: vec![true; capacity],
            removed: vec![false; capacity],
            sprite_w: vec![0; capacity],
            sprite_h: vec![0; capacity],
            anim_idx: vec![0; capacity],
            frame: vec![0; capacity],
            count: 0,
            stair_up: None,
            stair_down: None,
        }
    }
}

/// 格子索引（m_059 初始化 + spawnEntity 增量登记）。
pub struct CellGrid {
    pub cols: i32,
    pub rows: i32,
    /// cellTypeGrid[y][x]：格类型（0=未分配，由 spawnEntity 分配递增新类型）
    pub cell_type: Vec<Vec<u8>>,
    /// cellSlotCount[格类型]：容量
    pub capacity: Vec<u8>,
    /// cellSlotTable[格类型][槽]：实体 id+1
    pub slots: Vec<Vec<u8>>,
    /// nextCellTypeId：已分配格类型数
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

/// 类型绘制宽度表（Java typeFrameWidth，a.java:589，88 项；夹具 type_width_px.txt）。
/// dispatchSpriteSize 特例：type 69 → 96px（3 格）、type 72 → 32px（1 格）；dispatchAnimFields：类型
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

/// 渲染类别表（Java renderCategoryByType，initEntityTables 初始化 a.java:6256-6284）：
/// 区间 1..12→1、13..32→2、33..40→4、41..78→8，**特例覆盖优先**：
/// 76/81/82/83→1、77/78→16、79/80→4、72/84/87→32、85/86→2。
/// type 0 / ≥88 → 0。
pub fn render_category(type_id: i32) -> u8 {
    match type_id {
        76 | 81 | 82 | 83 => 1,
        77 | 78 => 16,
        79 | 80 => 4,
        72 | 84 | 87 => 32,
        85 | 86 => 2,
        1..=12 => 1,
        13..=32 => 2,
        33..=40 => 4,
        41..=78 => 8,
        _ => 0,
    }
}

/// 动画偏移表行号表（Java animRowByType，a.java:498-586 构造常量，89 项；
/// 行内容见 [`ANIM_OFFSET_TABLE`]）。
pub const ANIM_TABLE_IDX: [u8; 89] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 4, 2, 3, 3,
    3, 3, 3, 3, 2, 3, 2, 2, 2, 3, 2, 3, 3, 3, 2, 3,
    5, 3, 3, 3, 3, 3, 3, 0, 0, 0, 2, 2, 0, 0, 0, 0,
    0, 0, 0, 0, 2, 0, 0, 0, 0,
];

/// 动画帧偏移表（Java animOffsetTable，a.java:485-497 构造常量 11 行）：
/// 帧偏移 = 精灵帧宽 × 表[行][帧下标]。
pub const ANIM_OFFSET_TABLE: [&[i32]; 11] = [
    &[0],
    &[0, 1, 0, 2],
    &[0, 1, 2],
    &[0, 1, 2, 1],
    &[0, 1, 2, 2, 1, 0],
    &[0, 1, 2, 3, 2, 1],
    &[0, 1, 2, 3, 4],
    &[3, 4, 5, 6],
    &[4, 3, 2, 1, 0],
    &[2, 1, 0],
    &[2, 3, 4],
];

/// dispatchSpriteSize（a.java:6287-6312）：实体精灵尺寸分派。
/// type 69 → (96,32)；type 72 → (32,45)；type 67 → (32,32)（m_001(9) 换容器后走尾部）；
/// 图存在 → (typeFrameWidth[type], 图高)；图空 → (32,32)。
pub fn dispatch_sprite_size(type_id: i32, idx: usize, table: &mut EntityTable, width_table: &[i32], img_h: Option<i32>) {
    match type_id {
        69 => {
            table.sprite_w[idx] = 96;
            table.sprite_h[idx] = 32;
            return;
        }
        72 => {
            table.sprite_w[idx] = 32;
            table.sprite_h[idx] = 45;
            return;
        }
        67 => {}
        _ => {
            if let Some(h) = img_h {
                table.sprite_w[idx] = *width_table.get(type_id as usize).unwrap_or(&32);
                table.sprite_h[idx] = h;
                return;
            }
        }
    }
    table.sprite_w[idx] = 32;
    table.sprite_h[idx] = 32;
}

/// dispatchAnimFields（a.java:6314-6317）：动画字段初始化（行号 = animRowByType[type]，帧 0）。
pub fn dispatch_anim(type_id: i32, idx: usize, table: &mut EntityTable) {
    table.anim_idx[idx] = ANIM_TABLE_IDX[type_id as usize] as i32;
    table.frame[idx] = 0;
}

/// spawnEntity 忠实移植：像素取整到格、实体字段初始化、格槽登记（按类型宽度
/// `width_cells` 逐格，复核 R4：type 69 = 3 格）、楼梯锚点。
/// 返回新实体索引。含 dispatchSpriteSize/dispatchAnimFields 尺寸/动画分派（img_h = 该类型精灵条带高，
/// None = entityTypeImage[type] 为 null）。
pub fn spawn(
    table: &mut EntityTable,
    grid: &mut CellGrid,
    type_id: i32,
    px: i32,
    py: i32,
    param: i16,
    width_table: &[i32],
    img_h: Option<i32>,
) -> usize {
    let idx = table.count;
    let cell_x = (px + 16) >> 5;
    let cell_y = (py + 16) >> 5;
    let snapped_x = cell_x << 5;
    let snapped_y = cell_y << 5;

    dispatch_sprite_size(type_id, idx, table, width_table, img_h);
    dispatch_anim(type_id, idx, table);
    table.entity_type[idx] = type_id;
    table.pixel_x[idx] = snapped_x;
    table.pixel_y[idx] = snapped_y;
    table.removed[idx] = false;
    table.visible[idx] = true; // spawnEntity 默认可见（m_122 特例分支随后置 false）
    table.param[idx] = param;
    table.solid[idx] = 0;
    table.count += 1;

    // 格槽登记（Java var5 = dispatchSpriteSize 写入的 entitySpriteW>>5，≤0 取 1；a.java:6399-6408）
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

/// markEntityRemoved 核心（a.java:6336-6374）：solid=1（移除态）+ 类目分派视觉字段：
/// - 类目 1：type 1/2/3 → (45,56) 行 6；type 11 → (45,57) 行 2；其余不变；
///   随后调用方重绘小地图（buildMinimap）
/// - 类目 2/4 → (45,56) 行 10
/// - 类目 8/16/32 → (27,29) 行 6
pub fn remove(table: &mut EntityTable, idx: usize) {
    table.solid[idx] = 1;
    let t = table.entity_type[idx];
    match render_category(t) {
        1 => match t {
            1 | 2 | 3 => {
                table.sprite_w[idx] = 45;
                table.sprite_h[idx] = 56;
                table.anim_idx[idx] = 6;
                table.frame[idx] = 0;
            }
            11 => {
                table.sprite_w[idx] = 45;
                table.sprite_h[idx] = 57;
                table.anim_idx[idx] = 2;
                table.frame[idx] = 0;
            }
            _ => {}
        },
        2 | 4 => {
            table.sprite_w[idx] = 45;
            table.sprite_h[idx] = 56;
            table.anim_idx[idx] = 10;
            table.frame[idx] = 0;
        }
        8 | 16 | 32 => {
            table.sprite_w[idx] = 27;
            table.sprite_h[idx] = 29;
            table.anim_idx[idx] = 6;
            table.frame[idx] = 0;
        }
        _ => {}
    }
}

/// markEntityDesRemoved（a.java:6319-6334）：DES 变体——solid=2 + 类目 2/8/16/32 → (27,29) 行 8。
pub fn remove_des(table: &mut EntityTable, idx: usize) {
    table.solid[idx] = 2;
    let t = table.entity_type[idx];
    match render_category(t) {
        1 => {}
        _ => {
            table.sprite_w[idx] = 27;
            table.sprite_h[idx] = 29;
            table.anim_idx[idx] = 8;
            table.frame[idx] = 0;
        }
    }
}

/// sortEntitiesByY（a.java:6719-6808）：实体按 pixelY 升序的冒泡排序（严格小于 ⇒ 等值保序）。
/// 每次交换（var2=var4-1 ↔ var5=var4）伴随：
/// - 槽修复①（var2 **旧**格，a.java:6752-6762）：var2+1 → var5+1（var2 的记录迁去 var5）
/// - 槽修复②（var2 **新**格 = var5 旧位，a.java:6776-6790）：var5+1 → var2+1
/// 两段合成完整双向修复；交换后双向 dispatchAnimFields 重置动画字段。
/// A 级验证：排序后槽表与 oracle FLD 137（cellSlotTable）逐格一致。
pub fn sort_by_y(table: &mut EntityTable, grid: &mut CellGrid) {
    if table.count == 0 {
        return;
    }
    let mut var3 = table.count as i32;
    while var3 >= 1 {
        let mut var1 = table.pixel_y[0];
        for var4 in 1..var3 {
            if !table.removed[var4 as usize] {
                if table.pixel_y[var4 as usize] < var1 {
                    let a = (var4 - 1) as usize;
                    let b = var4 as usize;
                    let bak = (
                        table.entity_type[a],
                        table.pixel_x[a],
                        table.pixel_y[a],
                        table.sprite_w[a],
                        table.sprite_h[a],
                        table.solid[a],
                        table.removed[a],
                        table.visible[a],
                        table.param[a],
                    );
                    // 槽修复①：var2 旧格上 var2+1 → var5+1
                    fix_slot(grid, bak.1, bak.2, bak.3, a, b);
                    // var5 记录前移到 var2
                    table.entity_type[a] = table.entity_type[b];
                    table.pixel_x[a] = table.pixel_x[b];
                    table.pixel_y[a] = table.pixel_y[b];
                    table.sprite_w[a] = table.sprite_w[b];
                    table.sprite_h[a] = table.sprite_h[b];
                    table.solid[a] = table.solid[b];
                    table.removed[a] = table.removed[b];
                    table.visible[a] = table.visible[b];
                    table.param[a] = table.param[b];
                    dispatch_anim(table.entity_type[a], a, table);
                    // 备份记录落到 var5
                    table.entity_type[b] = bak.0;
                    table.pixel_x[b] = bak.1;
                    table.pixel_y[b] = bak.2;
                    table.sprite_w[b] = bak.3;
                    table.sprite_h[b] = bak.4;
                    table.solid[b] = bak.5;
                    table.removed[b] = bak.6;
                    table.visible[b] = bak.7;
                    table.param[b] = bak.8;
                    dispatch_anim(table.entity_type[b], b, table);
                    // 槽修复②：var2 新格（= var5 旧位）上 var5+1 → var2+1
                    fix_slot(grid, table.pixel_x[a], table.pixel_y[a], table.sprite_w[a], b, a);
                }
                var1 = table.pixel_y[var4 as usize];
            }
        }
        var3 -= 1;
    }
}

/// sortEntitiesByY 交换的格槽修复片段：在 (px,py) 起的 width 格里找 id_from+1 → 写 id_to+1
/// （a.java:6752-6762 / 6763-6777 同构两段）。
fn fix_slot(grid: &mut CellGrid, px: i32, py: i32, sprite_w: i32, id_from: usize, id_to: usize) {
    let mut width = sprite_w >> 5;
    if width <= 0 {
        width = 1;
    }
    let cy = (py >> 5) as usize;
    let cx = (px >> 5) as usize;
    for d in 0..width {
        let ct = grid.cell_type[cy][cx + d as usize] as usize;
        for s in 0..grid.capacity[ct] as usize {
            if grid.slots[ct][s] == (id_from + 1) as u8 {
                grid.slots[ct][s] = (id_to + 1) as u8;
                break;
            }
        }
    }
}

/// advanceEntityFrames（a.java:6812-6838）：实体动画帧推进（frameCounter 奇数拍生效）。
/// 帧到表尾：solid==1 → pullEntityFromCell 格槽摘除；solid==2 → dispatchSpriteSize/dispatchAnimFields 重置 + solid=0；
/// 其余 → 帧 0。pullEntityFromCell 的槽摘除当前仅在移除动画完成后需要（调用方
/// `detach_from_cell`）——此处返回"待摘除"实体索引列表，由场景层执行。
pub fn advance_frames(table: &mut EntityTable, grid: &mut CellGrid, frame_counter: i64, width_table: &[i32]) -> Vec<usize> {
    let mut detach = Vec::new();
    if table.count == 0 || (frame_counter & 1) == 0 {
        return detach;
    }
    let mut e = table.count as i32 - 1;
    while e >= 0 {
        let i = e as usize;
        if !table.removed[i] && table.visible[i] && table.anim_idx[i] > 0 {
            let row = table.anim_idx[i] as usize;
            let frame = table.frame[i];
            if frame < ANIM_OFFSET_TABLE[row].len() as i32 - 1 {
                table.frame[i] += 1;
            } else {
                match table.solid[i] {
                    1 => {
                        detach.push(i);
                        detach_from_cell(grid, table, i);
                    }
                    2 => {
                        let t = table.entity_type[i];
                        let h = table.sprite_h[i];
                        dispatch_sprite_size(t, i, table, width_table, Some(h));
                        dispatch_anim(t, i, table);
                        table.solid[i] = 0;
                    }
                    _ => table.frame[i] = 0,
                }
            }
        }
        e -= 1;
    }
    detach
}

/// pullEntityFromCell（a.java:6452-6468）：实体从所属格槽摘除 + removed=true。
pub fn detach_from_cell(grid: &mut CellGrid, table: &mut EntityTable, idx: usize) {
    table.removed[idx] = true;
    let cy = (table.pixel_y[idx] >> 5) as usize;
    let cx = (table.pixel_x[idx] >> 5) as usize;
    let ct = grid.cell_type[cy][cx] as usize;
    let cap = grid.capacity[ct] as usize;
    for s in 0..cap {
        if grid.slots[ct][s] as usize == idx + 1 && s < cap {
            for k in s..cap - 1 {
                grid.slots[ct][k] = grid.slots[ct][k + 1];
            }
            grid.capacity[ct] -= 1;
            return;
        }
    }
}

/// advanceBobPhase（a.java:6470-6479）：bob 浮沉相位推进（每拍 ±1，域 [-2,2] 翻转）。
pub fn advance_bob(offset: &mut i32, rising: &mut bool) {
    if *rising {
        *offset += 1;
        if *offset > 1 {
            *rising = false;
        }
    } else {
        *offset -= 1;
        if *offset < -1 {
            *rising = true;
        }
    }
}

/// DES 负参变体：按类型批量移除（executeScriptInstruction DES 分支语义）。
pub fn remove_all_of_type(table: &mut EntityTable, type_id: i32) {
    for i in 0..table.count {
        if table.entity_type[i] == type_id {
            table.solid[i] = 1;
        }
    }
}
