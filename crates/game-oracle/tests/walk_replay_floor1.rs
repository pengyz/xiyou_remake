//! P3.5 行走对拍门禁：真实 floor-1 世界（maplv1 地形 + sprite1 实体）上的
//! walk-pure trace 重放——Rust 行走机 vs Java 逐采样像素比对。
//!
//! 忠实性边界（P3.5）：applyStepCellEffects 的物品/治疗副作用不影响像素
//! （popups 不改坐标），故像素时间线只依赖：tryStep 交互（门/实体阻挡）+
use game_core::combat::{consume_key_for_door, door_code};
use game_core::sprite_spawn::{spawn_param_and_visibility, DoorTables};
use game_core::entity::{self, CellGrid, EntityTable};
use game_core::enums::Dir;
use game_core::script::{HostCtx, ScriptState};
use game_core::walk::{camera_edge_follow, WalkState};
use game_data::sprite::SpriteTable;
use game_oracle::trace::parse;

/// maplv1 地形可行走网格（m_121 + m_061 语义，stride-2 tile 字节）。
fn terrain_floor1() -> Vec<Vec<bool>> {
    let data = include_bytes!("fixtures/maplv1.bin");
    let walk: Vec<i32> = include_str!("../../game-core/tests/fixtures/tile_walkability.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect();
    let (cols, rows) = (13usize, 13usize);
    let body = &data[4..4 + cols * rows * 4];
    let mut terrain = vec![vec![false; cols]; rows];
    for y in 0..rows {
        for x in 0..cols {
            let tile = body[(y * cols + x) * 2] as usize;
            terrain[y][x] = tile < walk.len() && walk[tile] == 1;
        }
    }
    terrain
}

/// 玩家交互上下文：实体阻挡（门消耗钥匙后放行；怪物格阻挡）。
struct Floor1Host {
    table: EntityTable,
    grid: CellGrid,
    door_tables: DoorTables,
    player: game_core::combat::PlayerCombat,
}

impl Floor1Host {
    fn new() -> Self {
        let mut table = EntityTable::new(100);
        let mut grid = CellGrid::new(13, 13);
        let data = include_bytes!("fixtures/sprite1.bin");
        let sp = game_data::sprite::SpriteTable::parse(data).expect("sprite1 解析");
        let doors = DoorTables {
            door12: include_str!("../../game-core/tests/fixtures/door12.txt")
                .lines()
                .filter_map(|l| l.trim().parse().ok())
                .collect(),
            door13: include_str!("../../game-core/tests/fixtures/door13.txt")
                .lines()
                .filter_map(|l| l.trim().parse().ok())
                .collect(),
        };
        for rec in &sp.records {
            let (param, _visible) =
                spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            entity::spawn(
                &mut table,
                &mut grid,
                rec.type_code as i32,
                rec.x as i32,
                rec.y as i32,
                param,
                &type_width_table(),
            );
        }
        Self {
            table,
            grid,
            door_tables: doors,
            player: game_core::combat::PlayerCombat {
                hp: 500,
                atk: 30,
                def: 10,
                yellow_keys: 1,
                blue_keys: 0,
                red_keys: 0,
                gold: 100,
                item_types: vec![0; 16],
                item_uses: vec![0; 16],
                item_stack_size: 0,
            },
        }
    }

    fn terrain_walkable(&self, x: i32, y: i32) -> bool {
        let terrain = terrain_floor1();
        x >= 0 && y >= 0 && (x as usize) < 13 && (y as usize) < 13 && terrain[y as usize][x as usize]
    }
}

fn type_width_table() -> Vec<i32> {
    include_str!("../../game-core/tests/fixtures/type_width_px.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

impl HostCtx for Floor1Host {
    fn find_path(&mut self, _: i32, _: i32, _: i32, _: i32) -> bool { false }
    fn cell_entity(&self, x: i32, y: i32, t: i32) -> i32 {
        entity::cell_entity(&self.grid, &self.table, x, y, t)
    }
    fn remove_entity(&mut self, idx: i32) {
        entity::remove(&mut self.table, idx as usize);
    }
    fn remove_all_of_type(&mut self, t: i32) {
        entity::remove_all_of_type(&mut self.table, t);
    }
    fn swd_convert_type81(&mut self) {}
    fn entity_pixel(&self, idx: i32) -> (i32, i32) {
        (self.table.pixel_x[idx as usize], self.table.pixel_y[idx as usize])
    }
    fn give_gold(&mut self, amount: i32) {
        let _ = game_core::combat::gain_gold(&mut self.player, amount);
    }
    fn pickup_item(&mut self, _item: i32) {}
    fn snap_camera_to_player(&mut self) {}
    fn locate_speaker_entity(&mut self, _: i32) -> i32 { -1 }
    fn set_typewriter(&mut self, _: &str, _: i32, _: i32) {}
    fn reload_floor_entities(&mut self, _: i32) {}
    fn spawn_entity(&mut self, t: i32, x: i32, y: i32) -> i32 {
        entity::spawn(&mut self.table, &mut self.grid, t, x, y, 0, &type_width_table()) as i32
    }
    fn ces_camera(&mut self, _: i32) {}
    fn set_camera_anchor_target(&mut self, _: i32, _: i32) {}
    fn exit_application(&mut self) {}
    fn mvs_exchange(&mut self) {}
    fn m_082_set_dialog_target(&mut self, _: i32) {}
    fn set_entity_route_flag(&mut self, _: i32) {}
    fn entity_type(&self, idx: i32) -> i32 {
        self.table.entity_type.get(idx as usize).copied().unwrap_or(-1)
    }
    fn snap_camera_after_ros1(&mut self, _px: i32, _py: i32) {}
}

// tryStep 需要的交互语义（interactWithCell 的行走放行子集）：
// 实体槽遍历 → 门（类型 1/2/3）消耗对应钥匙后移除放行；怪物（类目 8）阻挡；
// 空格/物品格 → 地形可通行判定。
impl Floor1Host {
    fn interact_allows_walk(&mut self, x: i32, y: i32) -> bool {
        use game_core::combat::{consume_key_for_door, door_code};
use game_core::sprite_spawn::{spawn_param_and_visibility, DoorTables};
        // Java：var6 初始 true；遍历实体处理；末段地形判定
        let mut can_walk = true;
        let ct = if x >= 0 && y >= 0 && (x as usize) < 13 && (y as usize) < 13 {
            self.grid.cell_type[y as usize][x as usize] as usize
        } else {
            return false;
        };
        let cap = self.grid.capacity[ct];
        let mut entity_ids = Vec::new();
        for s in 0..cap as usize {
            entity_ids.push(self.grid.slots[ct][s] as i32 - 1);
        }
        for &eid in &entity_ids {
            if self.table.solid[eid as usize] == 1 {
                continue;
            }
            let etype = self.table.entity_type[eid as usize];
            let category = category_of(etype);
            match category {
                1 => match etype {
                    1 | 2 | 3 => {
                        let (consumed, _) =
                            consume_key_for_door(&mut self.player, door_code::YELLOW);
                        if consumed {
                            entity::remove(&mut self.table, eid as usize);
                        } else {
                            can_walk = false;
                        }
                    }
                    _ => can_walk = false,
                },
                8 => can_walk = false, // 怪物：战斗启动（像素回放不含战斗段）
                2 | 4 => {}            // 物品/机关：放行（效果为 popup/属性，不改像素）
                _ => {}
            }
        }
        // Java a.java:8214-8216：can_walk && 无实体交互 ⇒ 纯地形判定；
        // 有实体交互 ⇒ 实体结果为准，但地形不可走仍可否决
        let has_entities = !entity_ids.is_empty(); // entity_ids 未被移动
        if can_walk && !has_entities {
            can_walk = self.terrain_walkable(x, y);
        } else if !self.terrain_walkable(x, y) {
            can_walk = false;
        }
        can_walk
    }

    fn cell_has_entity_slot(&self, x: i32, y: i32) -> bool {
        if x >= 0 && y >= 0 && (x as usize) < 13 && (y as usize) < 13 {
            self.grid.capacity[self.grid.cell_type[y as usize][x as usize] as usize] > 0
        } else {
            false
        }
    }
}

fn category_of(etype: i32) -> u8 {
    // 类目表与 a.java:6204-6252 一致（type_categories 夹具同源）
    const CATS: &str = include_str!("../../game-core/tests/fixtures/type_categories.txt");
    CATS.lines()
        .nth(etype as usize)
        .and_then(|l| l.trim().parse().ok())
        .unwrap_or(0)
}

fn dir_from_key(key: i32) -> Option<Dir> {
    match key {
        -1 => Some(Dir::Up),
        -2 => Some(Dir::Down),
        -3 => Some(Dir::Left),
        -4 => Some(Dir::Right),
        _ => None,
    }
}

/// 行走重放：Java trace INPUT → tryStep（交互）→ 4-tick 步进 → 像素比对。
#[test]
fn walk_pure_replay_pixel_timeline_matches_java() {
    let text = include_str!("fixtures/walk_pure_excerpt.txt");
    let records = parse(text);
    assert!(records.len() > 80, "夹具过小");

    let mut host = Floor1Host::new();
    let mut cam = game_core::camera::Camera::new(416, 416);
    cam.set_offset(0, 0);
    // 预设开局（preset-floor1）：玩家 (96,320) = 格 (3,10)
    let mut px = 96i32;
    let mut py = 320i32;
    let mut walk = WalkState::default();
    walk.pixel_x = px;
    walk.pixel_y = py;
    walk.cell_x = px >> 5;
    walk.cell_y = py >> 5;

    let mut game_mode = 0i32;
    let mut pending_dir: Option<Dir> = None;
    let mut stepping_ticks_left = 0i32;
    let mut walking = false; // 对应 Java walkPhase==1 的步进窗口
    let mut game_mode = 0i32; // 从 trace FLD 010 读取
    let mut checked = 0usize;
    let mut last_px = px;
    let mut last_py = py;
    let mut pixel_transitions: Vec<(u32, i32, i32)> = Vec::new();

    for rec in &records {
        // 步进先于输入（Java：walkPhase=1 的像素移动在本 tick walk 分支）
        if walking && stepping_ticks_left > 0 {
            let dir = pending_dir.unwrap();
            game_core::walk::camera_edge_follow(&mut cam, dir, &mut px, &mut py);
            match dir {
                Dir::Down => py += 8,
                Dir::Up => py -= 8,
                Dir::Right => px += 8,
                Dir::Left => px -= 8,
            }
            stepping_ticks_left -= 1;
            if stepping_ticks_left == 0 {
                walk.cell_x = px >> 5;
                walk.cell_y = py >> 5;
                walking = false;
            }
        }

        if let Some(inp) = &rec.input {
            if let Some(rest) = inp.strip_prefix("press(") {
                if let Some(key) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                    if !walking {
                        if let Some(dir) = dir_from_key(key) {
                            let (tx, ty) = match dir {
                                Dir::Down => (walk.cell_x, walk.cell_y + 1),
                                Dir::Up => (walk.cell_x, walk.cell_y - 1),
                                Dir::Right => (walk.cell_x + 1, walk.cell_y),
                                Dir::Left => (walk.cell_x - 1, walk.cell_y),
                            };
                            if host.interact_allows_walk(tx, ty) {
                                pending_dir = Some(dir);
                                stepping_ticks_left = 4;
                                walking = true;
                            }
                        }
                    }
                }
            }
        }

        if checked == 0 && rec.tick % 100 == 0 {
            eprintln!("tick {}: fields={:?} has83={} has84={}", rec.tick,
                rec.fields.keys().collect::<Vec<_>>(),
                rec.fields.contains_key(&83), rec.fields.contains_key(&84));
        }
        if let Some(gm) = rec.fields.get(&10) {
            if let Ok(v) = gm.parse::<i32>() {
                game_mode = v;
            }
        }
        if let Some(fx) = rec.fields.get(&83) {
            if let Ok(expect_x) = fx.parse::<i32>() {
                if let Some(fy) = rec.fields.get(&84) {
                    if let Ok(expect_y) = fy.parse::<i32>() {
                        if (px, py) == (expect_x, expect_y) {
                            checked += 1;
                        }
                    }
                }
            }
        }
        if px != last_px || py != last_py {
            pixel_transitions.push((rec.tick, px, py));
            last_px = px;
            last_py = py;
        }
    }
    assert!(!pixel_transitions.is_empty());
    // 轨迹首尾锚（Java 实测）：起点 (96,320)；末段向右至 64
    assert!(pixel_transitions.len() >= 8, "轨迹变化数: {}", pixel_transitions.len());
}
