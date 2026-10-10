//! 行走状态机 —— tryStep / walkPhase-1 步进（8px）+ 相机边缘跟随 / walkPathBuffer 消费。
//!
//! 证据：reference/src/deobf/a.java
//! - tryStep :3543-3553（interact 通过 → walkPhase=1；弹窗打断 → 取消清零）
//! - run walkPhase-1 分支 :3546-3593（方向 ±8px；相机边缘跟随 ±8；32px 归格）
//! - dialogPhase-3 消费（:4062 区域）：`f_byte_12 = walkPathBuffer[--walkStepCount]`
//!   → tryStep；耗尽 → script_walk_armed=false + dialogPhase=0
//! - 步进 trace 实证：gameplay-floor1 tick1513-1516 玩家 y 328→336→344→352（4×8px）

use crate::camera::Camera;
use crate::enums::Dir;

pub const STEP_PX: i32 = 8;
pub const CELL_PX: i32 = 32;
/// 屏幕边缘跟随阈值（a.java:3556 等：106 = 240-... 的布局常量）
pub const EDGE_MARGIN: i32 = 106;

pub struct WalkState {
    /// facing / walkPathBuffer 方向码（0 下 1 上 2 右 3 左）
    pub facing: i32,
    pub pixel_x: i32,
    pub pixel_y: i32,
    pub cell_x: i32,
    pub cell_y: i32,
    pub step_progress_px: i32,
    /// f_byte_arr_42（100 步上限）
    pub walk_path_buffer: Vec<u8>,
    /// f_int_148（消费自末端：--count 取方向）
    pub walk_step_count: usize,
    pub script_walk_armed: bool,
    pub walk_request_flag: bool,
}

impl Default for WalkState {
    fn default() -> Self {
        Self {
            facing: 0,
            pixel_x: 0,
            pixel_y: 0,
            cell_x: 0,
            cell_y: 0,
            step_progress_px: 0,
            walk_path_buffer: Vec::new(),
            walk_step_count: 0,
            script_walk_armed: false,
            walk_request_flag: false,
        }
    }
}

/// 相机边缘跟随（run walkPhase-1 分支 a.java:3555-3577）。
/// 阈值逐方向核对（复核 R2/R3）：下 252-106=146；上 106；右 240-106=134；左 106。
pub fn camera_edge_follow(
    camera: &mut Camera,
    dir: Dir,
    player_px: &mut i32,
    player_py: &mut i32,
) {
    match dir {
        Dir::Down => {
            *player_py += STEP_PX;
            if *player_py + camera.offset_y + 16 > camera.view_h - EDGE_MARGIN {
                camera.set_offset(camera.offset_x, camera.offset_y - STEP_PX);
            }
        }
        Dir::Up => {
            *player_py -= STEP_PX;
            if *player_py + camera.offset_y + 16 < EDGE_MARGIN {
                camera.set_offset(camera.offset_x, camera.offset_y + STEP_PX);
            }
        }
        Dir::Right => {
            *player_px += STEP_PX;
            if *player_px + camera.offset_x + 16 > camera.view_w - EDGE_MARGIN {
                camera.set_offset(camera.offset_x - STEP_PX, camera.offset_y);
            }
        }
        Dir::Left => {
            *player_px -= STEP_PX;
            if *player_px + camera.offset_x + 16 < EDGE_MARGIN {
                camera.set_offset(camera.offset_x + STEP_PX, camera.offset_y);
            }
        }
    }
}

/// 完整步进 tick（a.java:3555-3593）：像素 ±8 + 相机边缘跟随 + 32px 归格
/// （progress 清零 + applyStepCellEffects hook）。
#[allow(clippy::too_many_arguments)]
pub fn tick_stepping(
    walk: &mut WalkState,
    camera: &mut Camera,
    dir: Dir,
    apply_step_effects: &mut dyn FnMut(&mut WalkState),
) {
    camera_edge_follow(camera, dir, &mut walk.pixel_x, &mut walk.pixel_y);
    walk.step_progress_px += STEP_PX;
    if walk.step_progress_px >= CELL_PX {
        walk.step_progress_px = 0;
        walk.cell_x = walk.pixel_x >> 5;
        walk.cell_y = walk.pixel_y >> 5;
        apply_step_effects(walk);
    }
}

/// dialogPhase-3 缓冲消费一步（a.java:4062 区域语义；方向码 → Dir）：
/// walkStepCount > 0 → 取 `buffer[--count]` 交给 tryStep；
/// == 0 → script_walk_armed=false、dialogPhase=0（恢复脚本）。
/// 返回 Option<方向码>（Some(dir) 时调用方执行 tryStep）。
pub fn next_script_walk_direction(walk: &mut WalkState) -> Option<Dir> {
    if walk.walk_step_count > 0 {
        walk.walk_step_count -= 1;
        Some(match walk.walk_path_buffer[walk.walk_step_count] {
            0 => Dir::Down,
            1 => Dir::Up,
            2 => Dir::Right,
            3 => Dir::Left,
            other => panic!("未知方向码 {other}（buffer 损坏）"),
        })
    } else {
        walk.script_walk_armed = false;
        None
    }
}
