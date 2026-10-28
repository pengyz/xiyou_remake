//! 视口点按路由宿主钉（Android 生产路径 `Runtime::step → step_scene → native::scene_tap`，真资产）。
//!
//! - BUG-038：浮层打开 / 场上非空闲时点画面不得穿透到下层（地图寻路、探步、道具格选择并使用）。
//!   RED 实录 = 2026-10-28 对抗复核运行时探针 P1a/P1b/P2（同一生产路径，修复前）：
//!   P1a 框开着点可走格 ⇒ mode Map→Dialog dp3（框后挂寻路）；P1b 点墙格 ⇒ walkPhase 0→1；
//!   P2 芭蕉扇确认框开着点 1 号格 ⇒ 千年月桂露被用掉（HP 300→448，次数 1→0）。

use game_android::input::KeyAction;
use game_android::native::{canvas_graphics, TitleGesture};
use game_android::runtime::{build_boot_machine, Runtime};
use game_core::enums::GameMode;
use game_core::layout::Layout;
use game_core::render::ArgbImage;
use game_shell_common::touch::{classify_tap, TapAction, TouchGeom};

fn tap(code: i32) -> Vec<KeyAction> {
    vec![KeyAction::Press(code), KeyAction::Release]
}

struct H {
    rt: Runtime,
    src: game_shell_common::asset::DirAssetSource,
    dir: std::path::PathBuf,
    boot_canvas: ArgbImage,
    scene_canvas: ArgbImage,
}

impl H {
    /// 一拍 + 绘制（换层遮幅推进挂 paint 公共尾，同 script_walk_taps）。
    fn step(&mut self, keys: Vec<KeyAction>, taps: Vec<(f32, f32)>) {
        self.rt.step(&self.src, &keys, Vec::new(), taps);
        let c = if self.rt.scene.is_some() { &mut self.scene_canvas } else { &mut self.boot_canvas };
        let mut gfx = canvas_graphics(c, 1);
        self.rt.paint(&mut gfx);
    }

    /// 一拍、不绘制（测试手工置的提示框没有排版数据）。
    fn step_no_paint(&mut self, keys: Vec<KeyAction>, taps: Vec<(f32, f32)>) {
        self.rt.step(&self.src, &keys, Vec::new(), taps);
    }

    fn sc(&self) -> &game_core::scene::GameScene {
        self.rt.scene.as_ref().expect("场景活动")
    }

    fn sc_mut(&mut self) -> &mut game_core::scene::GameScene {
        self.rt.scene.as_mut().expect("场景活动")
    }
}

impl Drop for H {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// 新游戏 → 序章对话推进到首个 Map（同 script_walk_taps 开场链）。
fn to_map(tag: &str) -> H {
    let raw = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw");
    let src = game_shell_common::asset::DirAssetSource::new(&raw);
    let dir = std::env::temp_dir().join(format!("jjxy-taproute-{}-{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&dir);
    let (rms, stores) = game_shell_common::rms::FileRms::load(&dir);
    let boot = Layout::modern(7, 10);
    let scene_layout = Layout::modern(12, 16);
    let mut rt = Runtime::new(build_boot_machine(&src, boot.clone()), stores, Box::new(rms));
    rt.apply_layouts(boot, scene_layout.clone());
    let mut h = H {
        rt,
        src,
        dir,
        boot_canvas: ArgbImage::create(240, 320),
        scene_canvas: ArgbImage::create(scene_layout.screen_w, scene_layout.screen_h),
    };
    let mut n = 0;
    while !h.rt.title_mode() && n < 400 {
        h.step(vec![], vec![]);
        n += 1;
    }
    let geo = h.rt.machine.title_geom().expect("标题");
    let ok = ((geo.zone_ok.0 + geo.zone_ok.2 / 2) as f32, (geo.zone_ok.1 + geo.zone_ok.3 / 2) as f32);
    h.rt.step(&h.src, &[], vec![TitleGesture::Tap(ok.0, ok.1)], vec![]);
    n = 0;
    while h.rt.scene.is_none() && n < 4000 {
        h.step(tap(-7), vec![]); // 序章 -7 跳过 → 读条 → 交接
        n += 1;
    }
    n = 0;
    while h.sc().game_mode != GameMode::Map && n < 4000 {
        h.step(if n % 4 == 0 { tap(-5) } else { vec![] }, vec![]);
        n += 1;
    }
    for _ in 0..4 {
        h.step(vec![], vec![]);
    }
    assert_eq!(h.sc().game_mode, GameMode::Map, "前置：到达首个地图");
    h
}

/// 用 scene_tap 同一几何找一个"点地寻路"点和一个"探步"点（逻辑坐标）。
fn field_points(h: &H) -> ((f32, f32), (f32, f32)) {
    let sc = h.sc();
    let geom = TouchGeom {
        cam_x: sc.view.cam_x,
        cam_y: sc.view.cam_y,
        tile_off: sc.layout.view_top_y,
        cols: sc.view.wide,
        rows: sc.view.high,
    };
    let walkable = |cx: i32, cy: i32| sc.walkable.get(cy as usize).and_then(|r| r.get(cx as usize)).copied().unwrap_or(false);
    let (mut walk, mut probe) = (None, None);
    for y in (40..sc.layout.screen_h - 40).step_by(8) {
        for x in (8..sc.layout.screen_w - 8).step_by(8) {
            match classify_tap(x, y, &geom, sc.layout.screen_w, sc.layout.screen_h, (sc.player_cell_x, sc.player_cell_y), &walkable) {
                TapAction::WalkTo(_) if walk.is_none() => walk = Some((x as f32, y as f32)),
                TapAction::ProbeStep(_) if probe.is_none() => probe = Some((x as f32, y as f32)),
                _ => {}
            }
        }
    }
    (walk.expect("有可走格"), probe.expect("有墙格"))
}

/// 地图上置一个提示框（kind 0，无脚注图标）。
fn open_map_message(h: &mut H) {
    let sc = h.sc_mut();
    sc.overlay_active = true;
    sc.overlay_kind = 0;
    sc.overlay_text = Some("提示".into());
}

#[test]
fn map_overlay_tap_on_walkable_cell_does_not_walk() {
    let mut h = to_map("p1a");
    let (walk, _) = field_points(&h);
    let before = (h.sc().player_cell_x, h.sc().player_cell_y);
    open_map_message(&mut h);
    h.step_no_paint(vec![], vec![walk]);
    assert_eq!(h.sc().game_mode, GameMode::Map, "框开着点可走格：不得在框后挂寻路（探针 P1a 曾变 Dialog dp3）");
    assert!(!h.sc().route.is_open(), "未开寻路段");
    h.sc_mut().overlay_active = false;
    for _ in 0..40 {
        h.step(vec![], vec![]);
    }
    assert_eq!((h.sc().player_cell_x, h.sc().player_cell_y), before, "关框后悟空不自行走动");
}

#[test]
fn map_overlay_tap_on_wall_cell_does_not_step() {
    let mut h = to_map("p1b");
    let (_, probe) = field_points(&h);
    open_map_message(&mut h);
    h.step_no_paint(vec![], vec![probe]);
    assert_eq!(h.sc().walk_phase, 0, "框开着点墙格：不得起步撞格（探针 P1b 曾 walkPhase 0→1）");
}

#[test]
fn use_confirm_open_tap_on_other_cell_uses_nothing() {
    let mut h = to_map("p2");
    {
        let sc = h.sc_mut();
        sc.player.item_types[0] = 16; // 芭蕉扇（无火 ⇒ 提示且不消耗）
        sc.player.item_uses[0] = 1;
        sc.player.item_types[1] = 19; // 千年月桂露（加血、消耗）
        sc.player.item_uses[1] = 1;
        sc.player.item_stack_size = 2;
    }
    h.step(tap(-6), vec![]); // 道具栏
    h.step(vec![], vec![]);
    h.step(tap(-5), vec![]); // 选 0 号 → 使用确认框（kind 3，○/×）
    h.step(vec![], vec![]);
    assert!(h.sc().overlay_active && h.sc().overlay_kind == 3, "前置：芭蕉扇使用确认框");
    let hp = h.sc().player.hp;
    let cell1 = {
        let (ox, oy) = h.sc().item_cell_origin();
        ((ox + 40 + 20) as f32, (oy + 20) as f32)
    };
    h.step(vec![], vec![cell1]);
    h.step(vec![], vec![]);
    let sc = h.sc();
    assert_eq!(sc.item_cursor, 0, "框开着点 1 号格：光标不得改到下层格");
    assert_eq!((sc.player.hp, sc.player.item_stack_size, sc.player.item_uses[1]), (hp, 2, 1), "千年月桂露未被使用（探针 P2 曾 HP 300→448）");
}
