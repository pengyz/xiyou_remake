//! 实体表与格子索引 —— Java spawnEntity（生成）/ m_100（格查）/ markEntityRemoved（移除）的忠实移植。
//!
//! 证据：reference/src/deobf/a.java（**行号锚定基准文件**：本文件裸
//! `a.java:NNNN` 一律指 deobf renamed 版；2026-10-16 逐行亲验更正旧行号）
//! - spawnEntity :6389-6428（像素取整 6394-6395、双数组登记 6403-6410、
//!   nextCellTypeId 格类型分配 + 槽登记 6412-6420、楼梯锚点 6423-6428）
//! - m_100（obf a.b(III)I，deobf renamed = findBlockingEntity）:8615 起
//!   （格类型→容量→槽位表查实体，跳过 solid==1）
//! - markEntityRemoved :6349-6387（solid=1 + 类目分派视觉字段）
//! - 宽度表 typeFrameWidth :589（88 项）+ dispatchSpriteSize 特例 :6306-6314
//!   （69→96px、72→32px）

use crate::render::ArgbImage;

/// 实体记录（SoA→AoS，refactor-plan §2 第 1 步）：字段对照 a.java 声明序（行号锚
/// 随字段）；**行为逻辑零改动**——仅存储布局。默认值=旧 Vec 预填值（可见 true、
/// 其余 0/false）——state_dump 金样整表引用含 100 槽尾部，尾默认必须逐字段对齐。
#[derive(Clone, Copy)]
pub struct Entity {
    /// entityType（a.java:129）
    pub type_: i32,
    /// entityParam（a.java:135）
    pub param: i16,
    /// entityPixelX（a.java:123）
    pub px: i32,
    /// entityPixelY（a.java:124）
    pub py: i32,
    /// entityState（a.java:137）：0=常态；1=markEntityRemoved 移除态〔开格动画〕；2=markEntityDesRemoved DES 态
    pub state: u8,
    /// entityVisible（a.java:132）：spawnEntity 默认 true；m_122 的 5/81/6/12/76/82/83 生成分支置 false
    pub visible: bool,
    /// entityRemoved（a.java:133）：DES/GUT/pullEntityFromCell 消耗用；sortEntitiesByY 排序跳过、paintEntityLayer 不画
    pub removed: bool,
    /// entitySpriteW（a.java:125）：精灵条带帧宽 px（dispatchSpriteSize 分派）
    pub sprite_w: i32,
    /// entitySpriteH（a.java:126）：精灵条带高 px（dispatchSpriteSize 分派）
    pub sprite_h: i32,
    /// entityAnimRow（a.java:130）：动画偏移表行号（dispatchAnimFields = animRowByType[type]）
    pub anim_row: i32,
    /// entityAnimFrame（a.java:127）：动画帧下标（advanceEntityFrames 推进）
    pub anim_frame: i32,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            type_: 0, param: 0, px: 0, py: 0, state: 0,
            visible: true, // 旧 Vec 预填 true（金样尾部对齐）
            removed: false,
            sprite_w: 0, sprite_h: 0, anim_row: 0, anim_frame: 0,
        }
    }
}

/// 实体表（100 槽，Java god class 的实体数组族）：Vec<Entity> cap100 +
/// count=entityCount（a.java:131）语义——**append-only**（槽三禁令②，四测钉死）。
pub struct EntityTable {
    pub items: Vec<Entity>,
    pub count: usize,
    /// 楼梯锚点（a.java:182-185 stairUp/DownMarkerX/Y；单体标量非实体数组）
    pub stair_up: Option<(i32, i32)>,
    pub stair_down: Option<(i32, i32)>,
}

impl EntityTable {
    pub fn new(capacity: usize) -> Self {
        Self {
            items: vec![Entity::default(); capacity],
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
    /// 槽登记追加：row[cap] = id（**1-based** 实体号），cap++。
    /// 【六写点收口】spawn / m_108 追加面（a.java:6412-6420 / 6808-6809 同构）。
    pub fn slot_push(&mut self, ct: usize, id: usize) {
        let cap = self.capacity[ct] as usize;
        self.slots[ct][cap] = id as u8;
        self.capacity[ct] += 1;
    }

    /// 摘除首个 id：续前移、cap-1、**序保持**（pullEntityFromCell 语义）。
    /// 【六写点收口】detach / m_108 摘除面（a.java:6770-6771/8880-8904）。
    pub fn slot_remove_first(&mut self, ct: usize, id: usize) -> bool {
        let cap = self.capacity[ct] as usize;
        for s in 0..cap {
            if self.slots[ct][s] as usize == id {
                for k in s..cap - 1 {
                    self.slots[ct][k] = self.slots[ct][k + 1];
                }
                self.capacity[ct] -= 1;
                return true;
            }
        }
        false
    }

    /// 按下标摘除（killAdjacentAt 的 removeSlotAt 面，a.java:7938+）：同压缩语义。
    pub fn slot_remove_at(&mut self, ct: usize, at: usize) {
        let cap = self.capacity[ct] as usize;
        for k in at..cap - 1 {
            self.slots[ct][k] = self.slots[ct][k + 1];
        }
        self.capacity[ct] -= 1;
    }

    /// 改写首个 from→to（sortEntitiesByY 槽修复①②面，a.java:6770-6771/6808-6809）。
    pub fn slot_replace_first(&mut self, ct: usize, from: u8, to: u8) {
        let cap = self.capacity[ct] as usize;
        for s in 0..cap {
            if self.slots[ct][s] == from {
                self.slots[ct][s] = to;
                return;
            }
        }
    }

    /// 过滤摘除（removeGroupAtCellByType 面，a.java:6434-6449）：drop(id)=true 者
    /// 摘除并回传 id 列表（调用方标 removed），余者序保持压实。返回=被摘 id 序。
    pub fn slot_remove_if(&mut self, ct: usize, drop: &mut dyn FnMut(u8) -> bool) -> Vec<u8> {
        let cap = self.capacity[ct] as usize;
        let mut kept: Vec<u8> = Vec::new();
        let mut dropped: Vec<u8> = Vec::new();
        for s in 0..cap {
            let id = self.slots[ct][s];
            if drop(id) {
                dropped.push(id);
            } else {
                kept.push(id);
            }
        }
        for (s, v) in kept.iter().enumerate() {
            self.slots[ct][s] = *v;
        }
        self.capacity[ct] = kept.len() as u8;
        dropped
    }

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

/// 渲染类别表（Java renderCategoryByType，initEntityTables 初始化 a.java:6268-6298）：
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

/// dispatchSpriteSize（a.java:6300-6325）：实体精灵尺寸分派。
/// type 69 → (96,32)；type 72 → (32,45)；type 67 → (32,32)（m_001(9) 换容器后走尾部）；
/// 图存在 → (typeFrameWidth[type], 图高)；图空 → (32,32)。
pub fn dispatch_sprite_size(type_id: i32, idx: usize, table: &mut EntityTable, width_table: &[i32], img_h: Option<i32>) {
    match type_id {
        69 => {
            table.items[idx].sprite_w = 96;
            table.items[idx].sprite_h = 32;
            return;
        }
        72 => {
            table.items[idx].sprite_w = 32;
            table.items[idx].sprite_h = 45;
            return;
        }
        67 => {}
        _ => {
            if let Some(h) = img_h {
                table.items[idx].sprite_w = *width_table.get(type_id as usize).unwrap_or(&32);
                table.items[idx].sprite_h = h;
                return;
            }
        }
    }
    table.items[idx].sprite_w = 32;
    table.items[idx].sprite_h = 32;
}

/// dispatchAnimFields（a.java:6327-6330）：动画字段初始化（行号 = animRowByType[type]，帧 0）。
pub fn dispatch_anim(type_id: i32, idx: usize, table: &mut EntityTable) {
    table.items[idx].anim_row = ANIM_TABLE_IDX[type_id as usize] as i32;
    table.items[idx].anim_frame = 0;
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
    table.items[idx].type_ = type_id;
    table.items[idx].px = snapped_x;
    table.items[idx].py = snapped_y;
    table.items[idx].removed = false;
    table.items[idx].visible = true; // spawnEntity 默认可见（m_122 特例分支随后置 false）
    table.items[idx].param = param;
    table.items[idx].state = 0;
    table.count += 1;

    // 格槽登记（Java var5 = dispatchSpriteSize 写入的 entitySpriteW>>5，≤0 取 1；a.java:6392-6400，槽登记 6412-6420）
    let width = width_cells(type_id, width_table);
    for dx in 0..width {
        let cx = (cell_x + dx) as usize;
        let mut ct = grid.cell_type[cell_y as usize][cx];
        if ct == 0 {
            grid.next_cell_type += 1;
            ct = grid.next_cell_type;
            grid.cell_type[cell_y as usize][cx] = ct;
        }
        grid.slot_push(ct as usize, idx + 1);
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
        if table.items[found as usize].type_ == type_id && table.items[found as usize].state != 1 {
            return found;
        }
    }
    -1
}

/// markEntityRemoved 核心（a.java:6349-6387）：solid=1（移除态）+ 类目分派视觉字段。
/// 返回 true = renderCategory==1（Java 此分支内调 buildMinimap，a.java:6370——
/// 调用方据此决定是否重绘小地图；cat2/4/8/16/32 一律不重建）：
/// - 类目 1：type 1/2/3 → (45,56) 行 6；type 11 → (45,57) 行 2；其余不变
/// - 类目 2/4 → (45,56) 行 10
/// - 类目 8/16/32 → (27,29) 行 6
pub fn remove(table: &mut EntityTable, idx: usize) -> bool {
    table.items[idx].state = 1;
    let t = table.items[idx].type_;
    match render_category(t) {
        1 => match t {
            1 | 2 | 3 => {
                table.items[idx].sprite_w = 45;
                table.items[idx].sprite_h = 56;
                table.items[idx].anim_row = 6;
                table.items[idx].anim_frame = 0;
            }
            11 => {
                table.items[idx].sprite_w = 45;
                table.items[idx].sprite_h = 57;
                table.items[idx].anim_row = 2;
                table.items[idx].anim_frame = 0;
            }
            _ => {}
        },
        2 | 4 => {
            table.items[idx].sprite_w = 45;
            table.items[idx].sprite_h = 56;
            table.items[idx].anim_row = 10;
            table.items[idx].anim_frame = 0;
        }
        8 | 16 | 32 => {
            table.items[idx].sprite_w = 27;
            table.items[idx].sprite_h = 29;
            table.items[idx].anim_row = 6;
            table.items[idx].anim_frame = 0;
        }
        _ => {}
    }
    render_category(t) == 1
}

/// markEntityDesRemoved（a.java:6332-6347）：DES 变体——solid=2 + 类目 2/8/16/32 → (27,29) 行 8。
pub fn remove_des(table: &mut EntityTable, idx: usize) {
    table.items[idx].state = 2;
    let t = table.items[idx].type_;
    match render_category(t) {
        1 => {}
        _ => {
            table.items[idx].sprite_w = 27;
            table.items[idx].sprite_h = 29;
            table.items[idx].anim_row = 8;
            table.items[idx].anim_frame = 0;
        }
    }
}

/// sortEntitiesByY（a.java:6732-6822）：实体按 pixelY 升序的冒泡排序（严格小于 ⇒ 等值保序）。
/// 每次交换（var2=var4-1 ↔ var5=var4）伴随：
/// - 槽修复①（var2 **旧**格，a.java:6770-6771）：var2+1 → var5+1（var2 的记录迁去 var5）
/// - 槽修复②（var2 **新**格 = var5 旧位，a.java:6808-6809）：var5+1 → var2+1
///
/// 两段合成完整双向修复；交换后双向 dispatchAnimFields 重置动画字段。
/// A 级验证：排序后槽表与 oracle FLD 137（cellSlotTable）逐格一致。
pub fn sort_by_y(table: &mut EntityTable, grid: &mut CellGrid) {
    if table.count == 0 {
        return;
    }
    let mut var3 = table.count as i32;
    while var3 >= 1 {
        let mut var1 = table.items[0].py;
        for var4 in 1..var3 {
            if !table.items[var4 as usize].removed {
                if table.items[var4 as usize].py < var1 {
                    let a = (var4 - 1) as usize;
                    let b = var4 as usize;
                    let bak = (
                        table.items[a].type_,
                        table.items[a].px,
                        table.items[a].py,
                        table.items[a].sprite_w,
                        table.items[a].sprite_h,
                        table.items[a].state,
                        table.items[a].removed,
                        table.items[a].visible,
                        table.items[a].param,
                    );
                    // 槽修复①：var2 旧格上 var2+1 → var5+1
                    fix_slot(grid, bak.1, bak.2, bak.3, a, b);
                    // var5 记录前移到 var2
                    table.items[a].type_ = table.items[b].type_;
                    table.items[a].px = table.items[b].px;
                    table.items[a].py = table.items[b].py;
                    table.items[a].sprite_w = table.items[b].sprite_w;
                    table.items[a].sprite_h = table.items[b].sprite_h;
                    table.items[a].state = table.items[b].state;
                    table.items[a].removed = table.items[b].removed;
                    table.items[a].visible = table.items[b].visible;
                    table.items[a].param = table.items[b].param;
                    dispatch_anim(table.items[a].type_, a, table);
                    // 备份记录落到 var5
                    table.items[b].type_ = bak.0;
                    table.items[b].px = bak.1;
                    table.items[b].py = bak.2;
                    table.items[b].sprite_w = bak.3;
                    table.items[b].sprite_h = bak.4;
                    table.items[b].state = bak.5;
                    table.items[b].removed = bak.6;
                    table.items[b].visible = bak.7;
                    table.items[b].param = bak.8;
                    dispatch_anim(table.items[b].type_, b, table);
                    // 槽修复②：var2 新格（= var5 旧位）上 var5+1 → var2+1
                    fix_slot(grid, table.items[a].px, table.items[a].py, table.items[a].sprite_w, b, a);
                }
                var1 = table.items[var4 as usize].py;
            }
        }
        var3 -= 1;
    }
}

/// sortEntitiesByY 交换的格槽修复片段：在 (px,py) 起的 width 格里找 id_from+1 → 写 id_to+1
/// （a.java:6770-6771 / 6808-6809 同构两段）。
fn fix_slot(grid: &mut CellGrid, px: i32, py: i32, sprite_w: i32, id_from: usize, id_to: usize) {
    let mut width = sprite_w >> 5;
    if width <= 0 {
        width = 1;
    }
    let cy = (py >> 5) as usize;
    let cx = (px >> 5) as usize;
    for d in 0..width {
        let ct = grid.cell_type[cy][cx + d as usize] as usize;
        grid.slot_replace_first(ct, (id_from + 1) as u8, (id_to + 1) as u8);
    }
}

/// advanceEntityFrames（a.java:6825-6848）：实体动画帧推进（frameCounter 奇数拍生效）。
/// 帧到表尾：solid==1 → pullEntityFromCell 格槽摘除；solid==2 → dispatchSpriteSize/dispatchAnimFields 重置 + solid=0；
/// 其余 → 帧 0。pullEntityFromCell 的槽摘除当前仅在移除动画完成后需要（调用方
/// `detach_from_cell`）——此处返回"待摘除"实体索引列表，由场景层执行。
pub fn advance_frames(
    table: &mut EntityTable,
    grid: &mut CellGrid,
    frame_counter: i64,
    width_table: &[i32],
    entity_imgs: &[Option<ArgbImage>],
) -> Vec<usize> {
    let mut detach = Vec::new();
    if table.count == 0 || (frame_counter & 1) == 0 {
        return detach;
    }
    let mut e = table.count as i32 - 1;
    while e >= 0 {
        let i = e as usize;
        if !table.items[i].removed && table.items[i].visible && table.items[i].anim_row > 0 {
            let row = table.items[i].anim_row as usize;
            let frame = table.items[i].anim_frame;
            if frame < ANIM_OFFSET_TABLE[row].len() as i32 - 1 {
                table.items[i].anim_frame += 1;
            } else {
                match table.items[i].state {
                    1 => {
                        detach.push(i);
                        detach_from_cell(grid, table, i);
                    }
                    2 => {
                        // a.java:6300 dispatchSpriteSize(type)：高度 = 类型图自身高
                        //（无 Some(h) 直通——DES 高度 29 不得驻留，迭代 49）
                        let t = table.items[i].type_;
                        let img_h = entity_imgs
                            .get(t as usize)
                            .and_then(|im| im.as_ref())
                            .map(|im| im.height);
                        dispatch_sprite_size(t, i, table, width_table, img_h);
                        dispatch_anim(t, i, table);
                        table.items[i].state = 0;
                    }
                    _ => table.items[i].anim_frame = 0,
                }
            }
        }
        e -= 1;
    }
    detach
}

/// pullEntityFromCell（a.java:6465-6481）：实体从所属格槽摘除 + removed=true。
pub fn detach_from_cell(grid: &mut CellGrid, table: &mut EntityTable, idx: usize) {
    table.items[idx].removed = true;
    let cy = (table.items[idx].py >> 5) as usize;
    let cx = (table.items[idx].px >> 5) as usize;
    let ct = grid.cell_type[cy][cx] as usize;
    grid.slot_remove_first(ct, idx + 1);
}

/// m_108（a.java:8880-8904）：路径缓动实体的格槽迁移——从**起点格** (from_x,from_y)
/// 的槽表摘除首个 `idx+1`（后续前移、容量−1；未找到则不动），再追加到实体**当前
/// 像素所在格**（格类型为 0 时 `++nextCellTypeId` 分配新类型）。
///
/// 忠实点：起点格由调用方传入（MOV 五参写的 f_int_129/130，全程不随实体移动更新）
/// ——同一实体重复调用（m_106 途中击杀后路径未尽再收尾，a.java:8848 + 8806）时，
/// 首次已迁走 ⇒ 二次摘除落空，但**追加照做**（removed 实体重新登记入槽，原版行为）。
pub fn relocate_slot(grid: &mut CellGrid, table: &EntityTable, idx: usize, from_x: i32, from_y: i32) {
    // 【六写点收口】摘除面 slot_remove_first + 追加面 slot_push（语义同原手写环）
    let ct = grid.cell_type[from_y as usize][from_x as usize] as usize;
    grid.slot_remove_first(ct, idx + 1);
    let cx = (table.items[idx].px >> 5) as usize;
    let cy = (table.items[idx].py >> 5) as usize;
    let mut nt = grid.cell_type[cy][cx];
    if nt == 0 {
        grid.next_cell_type += 1;
        nt = grid.next_cell_type;
        grid.cell_type[cy][cx] = nt;
    }
    grid.slot_push(nt as usize, idx + 1);
}

/// advanceBobPhase（a.java:6483-6491）：bob 浮沉相位推进（每拍 ±1，域 [-2,2] 翻转）。
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
        if table.items[i].type_ == type_id {
            table.items[i].state = 1;
        }
    }
}

/// m_049 removeGroupAtCellByType（a.java:6434-6449）：格 (cx,cy) 的槽组内
/// 按类型筛除（removed=true + 槽压缩）。
pub fn remove_group_at_cell_by_type(cx: i32, cy: i32, type_id: i32, table: &mut EntityTable, grid: &mut CellGrid) {
    let ct = grid.cell_type[cy as usize][cx as usize] as usize;
    if grid.capacity[ct] == 0 {
        return;
    }
    // 【六写点收口】过滤摘除面 slot_remove_if（序保持压实 + 摘除 id 回传标 removed）
    let dropped = grid.slot_remove_if(ct, &mut |id| table.items[id as usize - 1].type_ == type_id);
    for id in dropped {
        table.items[id as usize - 1].removed = true;
    }
}
