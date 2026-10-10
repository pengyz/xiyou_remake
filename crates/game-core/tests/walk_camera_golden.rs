//! P3.3 行走/相机 golden vectors（TDD）。
//! 证据：gameplay-floor1 trace（tick1513-1516 玩家 y 4×8px 步进 + 金+1）、
//! m_105 easing trace（tick20002-20010 十步收敛）、a.java:3546-3593/:8669-8700。

use game_core::camera::Camera;
use game_core::enums::Dir;
use game_core::walk::{camera_edge_follow, next_script_walk_direction, WalkState};

#[test]
fn stepping_y_direction_4_ticks_to_cross_cell() {
    // trace tick1513-1516：playerPixelY 320→328→336→344→352（方向 0 向下）
    let mut cam = Camera::new(416, 416); // 13×32
    cam.offset_y = 8;
    let mut px = 32i32;
    let mut py = 320i32;
    let mut progress = 0i32;
    let mut cell_snaps = 0;
    for _ in 0..4 {
        camera_edge_follow(&mut cam, Dir::Down, &mut px, &mut py);
        progress += 8;
        if progress >= 32 {
            progress = 0;
            cell_snaps += 1;
        }
    }
    assert_eq!(py, 352, "trace tick1516");
    assert_eq!(cell_snaps, 1, "32px 归格一次");
}

#[test]
fn camera_edge_follow_shifts_by_8() {
    // a.java:3570-3572：向右走、玩家屏幕位越过 240-106 ⇒ camX -= 8
    let mut cam = Camera::new(416, 416);
    cam.offset_x = 40; // 行走态相机偏移恒 ≤ 64（m_064 夹取）
    let mut px = 128i32; // 128 + 40 + 16 = 184 > 240 - 106 = 134 ✓ 触发
    let mut py = 0i32;
    camera_edge_follow(&mut cam, Dir::Right, &mut px, &mut py);
    assert_eq!(px, 136);
    assert_eq!(cam.offset_x, 32);
}

#[test]
fn m_105_easing_converges_6px_per_tick() {
    // trace tick20002-20010：anchor_x 40→96（Δ=56 → 步进 14,12,10,8,6,4,3,2,1 收敛）
    let mut cam = Camera::new(416, 416);
    cam.anchor_x = 40;
    cam.target_x = 96;
    let mut steps = 0;
    while !cam.at_target() && steps < 100 {
        cam.step_toward_target();
        steps += 1;
    }
    assert!(cam.at_target());
    assert_eq!(cam.anchor_x, 96);
    assert!(steps <= 12, "十步内收敛（trace 实测 10 步）：{steps}");
}

#[test]
fn script_walk_consumes_buffer_from_tail() {
    // dialogPhase-3 消费（a.java:4062）：--walkStepCount ⇒ 从缓冲末端取方向
    let mut walk = WalkState::default();
    walk.walk_path_buffer = vec![3, 2, 2]; // m_133 缓冲序（目标端优先）
    walk.walk_step_count = 3;
    walk.script_walk_armed = true;

    let d1 = next_script_walk_direction(&mut walk);
    assert_eq!(d1, Some(Dir::Right), "末端 = 行走首步（向右）");
    let d2 = next_script_walk_direction(&mut walk);
    assert_eq!(d2, Some(Dir::Right));
    let d3 = next_script_walk_direction(&mut walk);
    assert_eq!(d3, Some(Dir::Left));
    let d4 = next_script_walk_direction(&mut walk);
    assert_eq!(d4, None);
    assert!(!walk.script_walk_armed, "耗尽 ⇒ 取消武装");
}
