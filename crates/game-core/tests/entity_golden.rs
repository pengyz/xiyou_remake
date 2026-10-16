//! P3.2 实体模块 golden vectors（TDD）。
//!
//! 证据：deobf m_048（spawn）/ m_100（cell_entity）/ m_047（remove）/
//! m_133+m_134（findPath best-first，:9840-9936）/ m_061（地形可行走）。
//! BUG-006 重放向量使用真实 maplv51（原 jar 提取，fixtures/maplv51.bin）。

use game_core::entity::{self, CellGrid, EntityTable};
use game_core::pathfind::{find_path, Walkability};

fn width_table() -> Vec<i32> {
    include_str!("fixtures/type_width_px.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

fn grid51() -> (i32, i32, Vec<Vec<bool>>) {
    // m_121 + m_061：maplv51 → 13×13 地形可行走网格
    let data = include_bytes!("fixtures/maplv51.bin");
    let cols = 13i32;
    let rows = 13i32;
    let walk: Vec<i32> = include_str!("fixtures/tile_walkability.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect();
    let body = &data[4..4 + 676];
    let mut terrain = vec![vec![false; cols as usize]; rows as usize];
    for y in 0..rows as usize {
        for x in 0..cols as usize {
            // [2026-10-16 更正] rebuildWalkability（a.java:6964-6976）读法 =
            // **行距 wide·4、列距 2**（行尾 `var1 += var3 << 1` 叠加列循环 ×2）
            // ——与 scene.rs 载层 walkable 构建同式（其 A 级注：oracle FLD 134
            // 13×13 逐格复算匹配）。原 `(y*cols+x)*2` 步距 2W 仅行 0 吻合，
            // 行 ≥1 整体错位 ⇒ floor51 (1,11)→(3,10) 走廊被误判封死，是
            // "Rust find_path !found ≠ Java findPath found"分歧的真正根因。
            let tile = body[y * cols as usize * 4 + x * 2] as usize;
            terrain[y][x] = tile < walk.len() && walk[tile] == 1;
        }
    }
    (cols, rows, terrain)
}

#[test]
fn spawn_registers_cell_slot_and_stair_anchor() {
    // m_048（a.java:9664 区域）：像素取整、槽位登记、楼梯锚点
    let width_table = width_table();
    let mut t = EntityTable::new(100);
    let mut g = CellGrid::new(13, 13);
    // type 7（上楼梯）在像素 (96, 320) → 格 (3, 10)
    let idx = entity::spawn(&mut t, &mut g, 7, 96, 320, 0, &width_table, None);
    assert_eq!(idx, 0);
    assert_eq!(t.pixel_x[0], 96);
    assert_eq!(t.pixel_y[0], 320);
    assert_eq!(t.entity_type[0], 7);
    assert!(t.visible[0], "m_048 默认 visible=true");
    assert_eq!(t.stair_up, Some((96, 320)));
    // m_100 查询：格 (3,10) 上 type 7 → idx 0
    assert_eq!(entity::cell_entity(&g, &t, 3, 10, 7), 0);
    assert_eq!(entity::cell_entity(&g, &t, 3, 10, 8), -1);
}

#[test]
fn remove_marks_solid_and_lookup_skips() {
    // m_047（a.java:6325：solid=1）+ m_100 跳过 solid==1
    let width_table = width_table();
    let mut t = EntityTable::new(100);
    let mut g = CellGrid::new(13, 13);
    let a = entity::spawn(&mut t, &mut g, 26, 96, 320, 0, &width_table, None); // 黄门
    let b = entity::spawn(&mut t, &mut g, 26, 128, 320, 0, &width_table, None); // 同类型另一扇
    assert_eq!(entity::cell_entity(&g, &t, 3, 10, 26), a as i32);
    entity::remove(&mut t, a as usize);
    assert_eq!(t.solid[a as usize], 1);
    assert_eq!(entity::cell_entity(&g, &t, 3, 10, 26), -1, "已移除不再命中");
    assert_eq!(entity::cell_entity(&g, &t, 4, 10, 26), b as i32, "其余实体不受影响");
}

#[test]
fn remove_all_of_type_matches_des_negative() {
    // DES 负参批量移除（executeScriptInstruction DES 分支）
    let width_table = width_table();
    let mut t = EntityTable::new(100);
    let mut g = CellGrid::new(13, 13);
    entity::spawn(&mut t, &mut g, 44, 32, 32, 0, &width_table, None);
    entity::spawn(&mut t, &mut g, 44, 64, 32, 0, &width_table, None);
    entity::spawn(&mut t, &mut g, 45, 96, 32, 0, &width_table, None);
    entity::remove_all_of_type(&mut t, 44);
    assert_eq!(entity::cell_entity(&g, &t, 1, 1, 44), -1);
    assert!(t.solid[0] == 1 && t.solid[1] == 1);
    assert_eq!(entity::cell_entity(&g, &t, 3, 1, 45), 2, "异类型不受影响");
}

#[test]
fn find_path_straight_line_on_open_grid() {
    // 全开放网格：(1,1) → (5,1) 直线向右
    let walk = Walkability {
        terrain: vec![vec![true; 13]; 13],
        has_entity_slot: &|_, _| false,
        cols: 13,
        rows: 13,
    };
    let r = find_path(&walk, 1, 1, 5, 1);
    assert!(r.found);
    // 回溯方向映射（m_133）：父在左 ⇒ 行进向右（2）；4 步全 2
    assert_eq!(r.steps, vec![2, 2, 2, 2]);
}

#[test]
fn find_path_respects_terrain() {
    // 中间一堵墙：绕行或失败取决于网格——1×3 通道测试失败路径
    let terrain = vec![
        vec![true, false, true],
        vec![true, true, true],
    ];
    let walk = Walkability {
        terrain,
        has_entity_slot: &|_, _| false,
        cols: 3,
        rows: 2,
    };
    let r = find_path(&walk, 0, 1, 2, 0);
    // (0,1)→(1,1)→(2,1)→(2,0)：3 步（绕过 (1,0) 墙）
    assert!(r.found);
    // m_133 缓冲序 = 目标端优先：首 push 为目标侧末腿（up），消费走 --walkStepCount
    // 自末端（= 行走首段 right,right）——方向码 Up=1
    assert_eq!(r.steps, vec![1, 2, 2]);
}

/// BUG-006 重放：真实第 51 层网格上 (1,11) → (3,10)。
///
/// [证据链] oracle trace：playerCell(1,11)、SEE_3_10_166_166_1、f_int_148(walkStepCount)=3、
/// f_bool_26(walk_request_flag)=true（findPath 成功写点）。本测试用 Rust 重实现验证
/// Java 侧行为，并**更正 BUG-006 台账的根因**：原登记"寻路失败"——实为寻路成功
/// （3 步路径存在），冻结来自行尾收尾门（a.java:8568）覆写 dialogPhase 6→4、
/// 使已武装的行走永远不被执行。
#[test]
fn path_not_found_without_entity_slots_floor51() {
    let (cols, rows, terrain) = grid51();
    let walk = Walkability {
        terrain,
        has_entity_slot: &|_, _| false,
        cols,
        rows,
    };
    let r = find_path(&walk, 1, 11, 3, 10);
    // [2026-10-16 注] 步距修正后此测仍 !found——**结论未变但理由归正**：
    // 走廊 (1,11)→(3,10) 依赖 sprite51 实体槽（m_062）搭桥，纯地形网格
    // 不可达（对照下一测试：带槽 = found/3 步）。原"步距错位下的不可达"
    // 结论碰巧同向，理由以本注为准。
    assert!(!r.found);
}

#[test]
fn bug006_replay_with_sprite51_entities_full_chain() {
    // m_122 全链：sprite51 解析 → 类型码 switch 生成（含隐藏位与门参数表）
    // → findPath（m_134 可通行 = 地形 || m_062 实体槽）→ trace 实测 walkStepCount=3
    use game_core::sprite_spawn::{spawn_param_and_visibility, DoorTables};

    let (cols, rows, terrain) = grid51();
    let width_table = width_table();
    let mut t = EntityTable::new(100);
    let mut g = CellGrid::new(cols, rows);

    let data = include_bytes!("fixtures/sprite51.bin");
    let sp = game_data::sprite::SpriteTable::parse(data).expect("sprite51 解析");
    let doors = DoorTables {
        door12: include_str!("fixtures/door12.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
        door13: include_str!("fixtures/door13.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
    };

    for rec in &sp.records {
        let (param, _visible) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
        entity::spawn(&mut t, &mut g, rec.type_code as i32, rec.x as i32, rec.y as i32, param, &width_table, None);
    }
    assert_eq!(t.count, sp.records.len(), "全部记录生成");

    // m_134 可通行判定：地形 || m_062（格有实体槽）
    let walk = Walkability {
        terrain,
        has_entity_slot: &|x, y| {
            x >= 0 && y >= 0 && x < cols && y < rows
                && g.capacity[g.cell_type[y as usize][x as usize] as usize] > 0
        },
        cols,
        rows,
    };
    let r = find_path(&walk, 1, 11, 3, 10);
    // [2026-10-16 更正] 分歧根因 = grid51() 地形索引错位（见 grid51 注），**非**
    // find_path 语义——m_134/m_062（可通行 = isCellWalkable || 槽占用，
    // a.java:9967/6994）移植本已逐行忠实。步距修正后与 oracle 逐项吻合：
    // found + 3 步（f_bool_26=true + walkStepCount=3，BUG-006 证据链 D）。
    // 原"仍不可达/待 P3.3 考证"主张随步距修正撤销（原主张留档 git 历史）。
    assert!(r.found, "修正步距后应可达（oracle f_bool_26=true）");
    assert_eq!(r.steps.len(), 3, "walkStepCount=3（oracle f_int_148）");
    // 缓冲序 = 目标端优先（walk 自末端消费）：[2,2,1] ⇒ 行走序 up,right,right
    // = (1,11)→(1,10)→(2,10)→(3,10)——与 best-first 幻影头怪癖的平局序一致，
    // 此序列钉 m_134 插入/弹出怪癖的确定性。
    assert_eq!(r.steps, vec![2, 2, 1], "缓冲序 [2,2,1]");
}

/// m_108（a.java:8880-8904）：起点格摘除首个 idx+1（后续前移）→ 当前像素格追加
/// （格类型 0 时 ++nextCellTypeId）；二次调用摘除落空但追加照做（重复登记，原版行为）。
#[test]
fn relocate_slot_moves_registration_from_start_cell() {
    let width_table = width_table();
    let mut t = EntityTable::new(100);
    let mut g = CellGrid::new(13, 13);
    let a = entity::spawn(&mut t, &mut g, 72, 3 << 5, 7 << 5, 1, &width_table, None);
    let b = entity::spawn(&mut t, &mut g, 83, 3 << 5, 7 << 5, 1, &width_table, None);
    let c = entity::spawn(&mut t, &mut g, 83, 1 << 5, 8 << 5, 2, &width_table, None);
    let slots = |g: &CellGrid, x: usize, y: usize| -> Vec<u8> {
        let ct = g.cell_type[y][x] as usize;
        g.slots[ct][..g.capacity[ct] as usize].to_vec()
    };
    assert_eq!(slots(&g, 3, 7), vec![a as u8 + 1, b as u8 + 1]);
    let ct18 = g.cell_type[8][1];
    // 实体走到 (1,8)（已有格类型）→ 迁移：(3,7) 摘 a、(1,8) 追加于 c 之后
    t.pixel_x[a] = 1 << 5;
    t.pixel_y[a] = 8 << 5;
    entity::relocate_slot(&mut g, &t, a, 3, 7);
    assert_eq!(slots(&g, 3, 7), vec![b as u8 + 1], "后续前移");
    assert_eq!(g.cell_type[8][1], ct18, "已有格类型不重分配");
    assert_eq!(slots(&g, 1, 8), vec![c as u8 + 1, a as u8 + 1]);
    assert_eq!(entity::cell_entity(&g, &t, 1, 8, 72), a as i32);
    assert_eq!(entity::cell_entity(&g, &t, 3, 7, 72), -1);
    // 再走到 (1,10)（无格类型）→ 分配新类型 = 已分配数 + 1
    let next = (0..13).flat_map(|y| (0..13).map(move |x| (x, y))).map(|(x, y)| g.cell_type[y][x]).max().unwrap();
    t.pixel_y[a] = 10 << 5;
    entity::relocate_slot(&mut g, &t, a, 1, 8);
    assert_eq!(g.cell_type[10][1], next + 1, "++nextCellTypeId");
    assert_eq!(slots(&g, 1, 10), vec![a as u8 + 1]);
    assert_eq!(slots(&g, 1, 8), vec![c as u8 + 1]);
    // 同起点二次调用：摘除落空（起点格已无 a），追加照做 ⇒ (1,10) 重复登记
    entity::relocate_slot(&mut g, &t, a, 1, 8);
    assert_eq!(slots(&g, 1, 8), vec![c as u8 + 1], "起点格未找到不动");
    assert_eq!(slots(&g, 1, 10), vec![a as u8 + 1, a as u8 + 1]);
}

/// 槽三禁令②（refactor-behavior-matrix 1.2，Entity 迁移最高危红线）：
/// **槽位永不复用**——entityCount 单调 append-only 至换层重建（a.java spawnEntity
/// 尾 `entityCount++` 唯一增点；removeGroup/detach/markRemoved 均不回退计数）。
/// swap_remove/重编号 = 静默坏槽（帧哈希对内部序不敏感，全绿漂移前科）。
#[test]
fn spawn_indices_are_append_only_across_removals() {
    let wt = width_table();
    let mut t = EntityTable::new(100);
    let mut g = CellGrid::new(13, 13);
    let a = entity::spawn(&mut t, &mut g, 43, 0, 0, 10, &wt, None);
    let b = entity::spawn(&mut t, &mut g, 43, 32, 0, 20, &wt, None);
    let c = entity::spawn(&mut t, &mut g, 43, 64, 0, 30, &wt, None);
    assert_eq!((a, b, c), (0, 1, 2));
    // 三种移除面全走一遍：markEntityRemoved（solid=1）/ DES 变体（solid=2）/ detach（摘槽）
    entity::remove(&mut t, 1);
    entity::remove_des(&mut t, 2);
    entity::detach_from_cell(&mut g, &mut t, 0);
    assert_eq!(t.count, 3, "移除不回退 entityCount（append-only）");
    // 再生成：索引续增（3），**不得复用** 0/1/2
    let d = entity::spawn(&mut t, &mut g, 43, 96, 0, 40, &wt, None);
    assert_eq!(d, 3, "新实体索引=旧 count，槽位永不复用");
    assert_eq!(t.count, 4);
    // 留置形态：被移除实体记录本体不动（下标语义稳定）
    assert_eq!(t.entity_type[1], 43);
    assert_eq!(t.param[1], 20);
    assert_eq!(t.entity_type[2], 43);
    assert!(t.removed[0], "detach 标记留置");
    // 新槽登记 = idx+1 = 4（1-based 编码不变）
    let ct = g.cell_type[0][3] as usize;
    assert_eq!(&g.slots[ct][..g.capacity[ct] as usize], &[4]);
}

/// 六写点收口对照（refactor-behavior-matrix 1.3a）：**fix_slot 双格修复**——
/// sortEntitiesByY 交换伴随槽修复①②（a.java:6770-6771/6808-6809），宽实体
/// （型 69 宽 78 ⇒ 2 格）跨格槽表两行同步改写。
#[test]
fn sort_by_y_swap_fixes_slot_entries_across_all_width_cells() {
    let wt = width_table();
    let mut t = EntityTable::new(100);
    let mut g = CellGrid::new(13, 13);
    // A（型 69 双格）格 (0,10)-(1,10) param=1；B（型 43 单格）格 (5,5) param=2
    let a = entity::spawn(&mut t, &mut g, 69, 0, 10 << 5, 1, &wt, None);
    let b = entity::spawn(&mut t, &mut g, 43, 5 << 5, 5 << 5, 2, &wt, None);
    assert_eq!((a, b), (0, 1));
    let row = |g: &CellGrid, x: usize, y: usize| -> Vec<u8> {
        let ct = g.cell_type[y][x] as usize;
        g.slots[ct][..g.capacity[ct] as usize].to_vec()
    };
    assert_eq!(row(&g, 0, 10), vec![1], "前置：A 左格");
    assert_eq!(row(&g, 1, 10), vec![1], "前置：A 右格");
    assert_eq!(row(&g, 5, 5), vec![2], "前置：B 格");
    entity::sort_by_y(&mut t, &mut g);
    // 交换后：idx0=B 记录（y 小在前），idx1=A 记录
    assert_eq!(t.param[0], 2, "B 记录前移");
    assert_eq!(t.param[1], 1, "A 记录后落");
    // 槽修复①（A 旧格两行 1→2）+②（B 旧格 2→1）——双格两行全同步
    assert_eq!(row(&g, 0, 10), vec![2], "宽体左格同步修复");
    assert_eq!(row(&g, 1, 10), vec![2], "宽体右格同步修复");
    assert_eq!(row(&g, 5, 5), vec![1], "B 格回写");
}

/// 六写点收口对照（refactor-behavior-matrix 1.3b）：**detach_from_cell**——
/// removed=true + 槽行压缩（a.java pullEntityFromCell：摘 idx+1 后续前移、
/// 容量-1、**序保持**）。
#[test]
fn detach_from_cell_compacts_slot_row_in_order() {
    let wt = width_table();
    let mut t = EntityTable::new(100);
    let mut g = CellGrid::new(13, 13);
    for i in 0..3 {
        entity::spawn(&mut t, &mut g, 43, 3 << 5, 7 << 5, i as i16, &wt, None);
    }
    let ct = g.cell_type[7][3] as usize;
    assert_eq!(&g.slots[ct][..g.capacity[ct] as usize], &[1, 2, 3]);
    entity::detach_from_cell(&mut g, &mut t, 1);
    assert!(t.removed[1]);
    assert_eq!(&g.slots[ct][..g.capacity[ct] as usize], &[1, 3], "摘除后续前移、序保持");
    assert_eq!(g.capacity[ct], 2);
    // 留置形态：被摘实体记录本体不动（swap_remove/重编号禁令对照面）
    assert_eq!(t.entity_type[1], 43);
    assert_eq!(t.param[1], 1);
}

/// 槽三禁令③配套（refactor-behavior-matrix 1.5）：**sort_by_y 平局序**——
/// 严格小于 ⇒ 等值保序（a.java:6732-6822 冒泡 `pixelY[var4] < var1`）。
/// **禁 unstable sort**（等值实体交换序变 ⇒ 无帧锚场景静默漂移，A1 反例）。
#[test]
fn sort_by_y_tie_order_is_stable() {
    let wt = width_table();
    let mut t = EntityTable::new(100);
    let mut g = CellGrid::new(13, 13);
    // y=[10,5,10,5,10]，param=身份 0..4，各占独立格
    for (i, y) in [10, 5, 10, 5, 10].iter().enumerate() {
        entity::spawn(&mut t, &mut g, 43, (i as i32) << 5, y << 5, i as i16, &wt, None);
    }
    entity::sort_by_y(&mut t, &mut g);
    let order: Vec<i16> = (0..5).map(|i| t.param[i]).collect();
    assert_eq!(order, vec![1, 3, 0, 2, 4], "同 y 组内相对序不变（y=5 组 1,3；y=10 组 0,2,4）");
    let sorted_ys: Vec<i32> = (0..5).map(|i| t.pixel_y[i]).collect();
    assert_eq!(sorted_ys, vec![160, 160, 320, 320, 320], "y 升序");
}
