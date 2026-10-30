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
use game_core::tap_walk::{TapPlan, TargetKind};
use game_shell_common::touch::TouchGeom;

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

/// 用 core 格分类找一个"点地寻路"点（可达 Floor）和一个"探步/撞"点（Wall），逻辑坐标（STAGE-B-008：
/// 格语义全在 core tap_grid，壳只报格）。
fn field_points(h: &H) -> ((f32, f32), (f32, f32)) {
    let sc = h.sc();
    let geom = TouchGeom {
        cam_x: sc.view.cam_x,
        cam_y: sc.view.cam_y,
        tile_off: sc.layout.view_top_y,
        cols: sc.view.wide,
        rows: sc.view.high,
    };
    let grid = sc.tap_grid();
    let me = (sc.player_cell_x, sc.player_cell_y);
    let (mut walk, mut probe) = (None, None);
    for y in (40..sc.layout.screen_h - 40).step_by(8) {
        for x in (8..sc.layout.screen_w - 8).step_by(8) {
            let Some(c) = geom.cell_at(x, y) else { continue };
            if c == me {
                continue;
            }
            match grid.get(c.0, c.1).map(|k| k.kind) {
                Some(TargetKind::Floor) if walk.is_none() => walk = Some((x as f32, y as f32)),
                Some(TargetKind::Wall) if probe.is_none() => probe = Some((x as f32, y as f32)),
                _ => {}
            }
        }
    }
    (walk.expect("有地面格"), probe.expect("有墙格"))
}

/// 地图上置一个提示框（kind 0，无脚注图标）。
fn open_map_message(h: &mut H) {
    let sc = h.sc_mut();
    sc.overlay_active = true;
    sc.overlay_kind = 0;
    sc.overlay_text = Some("提示".into());
}

/// 【BUG-042】点地寻路的格换算须与游戏绘制同一格网：格 = 32 px（`player_cell_x = player_px >> 5`、实体 `cx << 5`）。
/// 期望点来自游戏绘制公式（格屏幕原点 = 相机 + view_top + 格 × 32，scene.rs paint_entities / paint_player 同式），
/// 不经触屏模块自身的换算（旧测试用 `cell_screen_xy` 求点 ⇒ 与被测同错，抓不到）。
/// 依次点可达范围内**每一个**在视口内的空格，每次都须恰好走到被点的格（单个目标可能因邻格实体让 walk_to 停在邻格而掩盖
/// ±1 格换算错误，复核 bug042-重要1）。`cam` = Some 时先把相机移到 x / y 不等的偏移（首图相机近原点，掩盖相机项与 x / y 对调）。
fn tap_walk_case(tag: &str, cam: Option<(i32, i32)>) {
    let mut h = to_map(tag);
    let set_cam = |h: &mut H| {
        if let Some((cx, cy)) = cam {
            let sc = h.sc_mut();
            sc.view.cam_x = cx;
            sc.view.cam_y = cy;
        }
    };
    set_cam(&mut h);
    let (px, py) = (h.sc().player_cell_x, h.sc().player_cell_y);
    let targets: Vec<(i32, i32)> = {
        let sc = h.sc();
        let grid = sc.tap_grid(); // 【STAGE-B-008】前提含"非陷阱危险格"（BFS 途中须避让）
        let in_map = |x: i32, y: i32| x >= 0 && y >= 0 && x < sc.view.wide && y < sc.view.high;
        let free = |x: i32, y: i32| {
            in_map(x, y)
                && sc.walkable[y as usize][x as usize]
                && sc.grid.capacity[sc.grid.cell_type[y as usize][x as usize] as usize] == 0
                && !grid.get(x, y).map_or(false, |c| c.danger)
        };
        let on_screen = |x: i32, y: i32| {
            let (sx, sy) = (sc.view.cam_x + (x << 5), sc.view.cam_y + sc.layout.view_top_y + (y << 5));
            sx >= 0 && sy >= sc.layout.view_top_y && sx + 32 <= sc.layout.screen_w && sy + 32 <= sc.layout.view_top_y + sc.view.view_h
        };
        let mut seen = std::collections::HashSet::from([(px, py)]);
        let mut q = std::collections::VecDeque::from([(px, py)]);
        let mut out = Vec::new();
        while let Some((x, y)) = q.pop_front() {
            if (x, y) != (px, py) && on_screen(x, y) {
                out.push((x, y));
            }
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (x + dx, y + dy);
                if free(n.0, n.1) && seen.insert(n) {
                    q.push_back(n);
                }
            }
        }
        out
    };
    assert!(targets.len() >= 2, "[{tag}] 前置：可达空格 ≥2（实得 {targets:?}）");
    for target in targets {
        set_cam(&mut h);
        let from = (h.sc().player_cell_x, h.sc().player_cell_y);
        let (sx, sy) = {
            let sc = h.sc();
            (sc.view.cam_x + (target.0 << 5) + 16, sc.view.cam_y + sc.layout.view_top_y + (target.1 << 5) + 16)
        };
        h.step(vec![], vec![(sx as f32, sy as f32)]);
        if target != from {
            assert!(h.sc().route.is_open(), "[{tag}] 点中绘制在 ({sx},{sy}) 的格 {target:?}：开寻路（玩家在 {from:?}）");
        }
        for _ in 0..160 {
            h.step(vec![], vec![]);
            if (h.sc().player_cell_x, h.sc().player_cell_y) == target && h.sc().walk_phase == 0 && !h.sc().route.is_open() {
                break;
            }
        }
        assert_eq!((h.sc().player_cell_x, h.sc().player_cell_y), target, "[{tag}] 从 {from:?} 点 {target:?}：走到被点的那一格");
        assert_eq!(h.sc().game_mode, GameMode::Map, "[{tag}] 走完回到地图行走态");
    }
}

/// 【BUG-043】用户实机："教程关中有绘制路径箭头的情况，当箭头存在时，用触屏移动会导致箭头错位，用虚拟摇杆移动不会"。
/// 序章首图：箭头自出生格起 ↑→→。点箭头上的第 2 格（点地路线现由 core BFS 给出，可能与箭头路线不同——spec §5.4）：
/// 逐拍画出的箭头都须是**原箭头路线**的尾段；参照局 = 用方向键沿**点地实际走的路线**逐格走到同一格
///（期望 = 经典按键链独立产生），走到后余下箭头 = 点地局。
#[test]
fn tutorial_arrows_stay_on_their_path_during_tap_walk() {
    let mut h = to_map("bug043");
    let original = h.sc().path_arrow_cells();
    assert!(original.len() >= 3, "前置：序章首图有路径箭头（实得 {original:?}）");
    let target = original[1];
    // 点地路线（被测输入）：点按前独立取 plan_tap 步序（目标端优先、自末端消费 ⇒ 第一步 = steps[len-1]）
    let steps: Vec<u8> = {
        let sc = h.sc();
        let plan = game_core::tap_walk::plan_tap(&sc.tap_grid(), (sc.player_cell_x, sc.player_cell_y), target);
        let TapPlan::Walk { steps, .. } = plan else { panic!("箭头格应为可达地面：{plan:?}") };
        steps
    };
    let (sx, sy) = {
        let sc = h.sc();
        (sc.view.cam_x + (target.0 << 5) + 16, sc.view.cam_y + sc.layout.view_top_y + (target.1 << 5) + 16)
    };
    h.step(vec![], vec![(sx as f32, sy as f32)]);
    assert!(h.sc().route.is_open(), "点箭头格 {target:?} = 开寻路");
    assert_eq!(&h.sc().walk_path_buffer[..], &steps[..], "点按后缓冲 = plan_tap 步序（count 为剩余游标，同一拍可能已消费首步）");
    for t in 0..160 {
        let cells = h.sc().path_arrow_cells();
        assert!(original.ends_with(&cells), "第 {t} 拍箭头 {cells:?} 偏离原路线 {original:?}");
        if (h.sc().player_cell_x, h.sc().player_cell_y) == target && h.sc().walk_phase == 0 && !h.sc().route.is_open() {
            break;
        }
        h.step(vec![], vec![]);
    }
    assert_eq!((h.sc().player_cell_x, h.sc().player_cell_y), target, "走到点中格");
    // 参照：另起一局，用方向键（面板同键码）沿同一路线逐格走到同一格
    let mut k = to_map("bug043key");
    for &d in steps.iter().rev() {
        let code = match d {
            0 => -2,
            1 => -1,
            2 => -4,
            _ => -3,
        };
        let from = (k.sc().player_cell_x, k.sc().player_cell_y);
        k.step(vec![KeyAction::Press(code)], vec![]);
        k.step(vec![KeyAction::Release], vec![]);
        for _ in 0..40 {
            if (k.sc().player_cell_x, k.sc().player_cell_y) != from && k.sc().walk_phase == 0 {
                break;
            }
            k.step(vec![], vec![]);
        }
    }
    for _ in 0..6 {
        k.step(vec![], vec![]);
        h.step(vec![], vec![]);
    }
    assert_eq!((k.sc().player_cell_x, k.sc().player_cell_y), target, "参照：方向键沿点地路线走到同一格");
    assert_eq!(h.sc().path_arrow_cells(), k.sc().path_arrow_cells(), "余下箭头：点地 = 方向键");
    assert_eq!(h.sc().path_arrows_visible, k.sc().path_arrows_visible);
}

#[test]
fn tap_on_drawn_walkable_cell_walks_player_there() {
    tap_walk_case("bug042", None);
}

#[test]
fn tap_walks_to_drawn_cell_with_camera_offset() {
    tap_walk_case("bug042cam", Some((37, -45)));
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

// ───────────────────────── BUG-039：点空白只在"继续 / 关闭"类情境代按确定

/// 地图上的普通提示框：点画面 = 关闭（与面板"确定"同效）。
#[test]
fn map_message_overlay_tap_closes_it() {
    let mut h = to_map("b39msg");
    let (walk, _) = field_points(&h);
    open_map_message(&mut h);
    h.step_no_paint(vec![], vec![walk]);
    assert!(!h.sc().overlay_active, "点画面关闭提示框");
    assert_eq!(h.sc().game_mode, GameMode::Map, "不寻路");
}

/// 原版用 ○/× 脚注标明的是否框（商人报价同款）：点画面不代按确定（-5 = 接受 / 购买）。
#[test]
fn map_yes_no_overlay_tap_does_nothing() {
    let mut h = to_map("b39yesno");
    let (walk, _) = field_points(&h);
    {
        let sc = h.sc_mut();
        sc.overlay_active = true;
        sc.overlay_kind = 4;
        sc.overlay_left_icon = 1;
        sc.overlay_right_icon = 2;
        sc.overlay_text = Some("报价".into());
    }
    let gold = h.sc().player.gold;
    h.step_no_paint(vec![], vec![walk]);
    assert!(h.sc().overlay_active, "是否框仍开着（等玩家按 接受 / 拒绝）");
    assert_eq!(h.sc().player.gold, gold, "未花钱");
}

/// 存档页点空白：不得覆盖当前槽（曾 = -5 存入高亮槽并弹"保存成功！"）。
#[test]
fn save_page_blank_tap_does_not_overwrite_slot() {
    let mut h = to_map("b39save");
    h.step(tap(-7), vec![]); // 游戏内菜单
    for _ in 0..3 {
        h.step(vec![], vec![]);
    }
    let kinds = h.sc().menu_kinds.clone();
    let save_idx = kinds.iter().position(|&k| k == 3).expect("菜单有保存项");
    for _ in 0..save_idx {
        h.step(tap(-2), vec![]);
        h.step(vec![], vec![]);
    }
    h.step(tap(-6), vec![]); // 保存游戏 → 存档槽（mode 7）
    h.step(vec![], vec![]);
    assert_eq!(h.sc().game_mode, GameMode::SaveSlot, "前置：存档槽页");
    let blank = (2.0, 2.0);
    assert!(
        matches!(h.sc().pointer_hit(2, 2), game_core::scene::PointerHit::None),
        "前置：(2,2) 不在任何槽行上"
    );
    h.step(vec![], vec![blank]);
    h.step(vec![], vec![]);
    assert!(!h.rt.stores.iter().any(|(k, _)| k == "MOT_L0@1"), "点空白不写存档");
    assert!(!h.sc().overlay_active, "未弹\"保存成功！\"");
}

// ───────────────────────── STAGE-B-008：core 点地规则（tap_cell / tap_grid）真资产钉

/// 独立 BFS（只走地形可走格，不经被测代码）：自玩家格的可达集。
fn terrain_reach(sc: &game_core::scene::GameScene) -> std::collections::HashSet<(i32, i32)> {
    let me = (sc.player_cell_x, sc.player_cell_y);
    let mut seen = std::collections::HashSet::from([me]);
    let mut q = std::collections::VecDeque::from([me]);
    while let Some((x, y)) = q.pop_front() {
        for n in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
            let inside = n.0 >= 0 && n.1 >= 0 && n.0 < sc.view.wide && n.1 < sc.view.high;
            if inside && sc.walkable[n.1 as usize][n.0 as usize] && seen.insert(n) {
                q.push_back(n);
            }
        }
    }
    seen
}

/// 【STAGE-B-008 · spec §0 隐藏通路】54 层：(4,2) 只能经未显形的隐形路径到达——点它不得走到、隐形路径不得显形。
#[test]
fn tap_does_not_leak_hidden_path_floor54() {
    let mut h = to_map("hidden54");
    h.sc_mut().change_floor(54, false, false);
    for _ in 0..60 {
        h.step(vec![], vec![]);
    }
    let target = (4, 2);
    // 前置（独立 BFS，不经被测代码）：只走地形可走格不可达；把隐形路径格算作可走则可达
    let (no_hidden, with_hidden) = {
        let sc = h.sc();
        let hidden: std::collections::HashSet<(i32, i32)> = sc.entities.items[..sc.entities.count]
            .iter()
            .filter(|e| e.type_ == 6 && !e.removed && !e.visible)
            .map(|e| (e.px >> 5, e.py >> 5))
            .collect();
        let reach = |with: bool| {
            let me = (sc.player_cell_x, sc.player_cell_y);
            let mut seen = std::collections::HashSet::from([me]);
            let mut q = std::collections::VecDeque::from([me]);
            while let Some((x, y)) = q.pop_front() {
                for n in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                    let inside = n.0 >= 0 && n.1 >= 0 && n.0 < sc.view.wide && n.1 < sc.view.high;
                    if inside && (sc.walkable[n.1 as usize][n.0 as usize] || (with && hidden.contains(&n))) && seen.insert(n) {
                        q.push_back(n);
                    }
                }
            }
            seen.contains(&target)
        };
        (reach(false), reach(true))
    };
    assert!(!no_hidden && with_hidden, "前置：(4,2) 只经隐形路径可达（实得 {no_hidden}/{with_hidden}）");
    let (sx, sy) = { let sc = h.sc(); (sc.view.cam_x + (target.0 << 5) + 16, sc.view.cam_y + sc.layout.view_top_y + (target.1 << 5) + 16) };
    h.step(vec![], vec![(sx as f32, sy as f32)]);
    for _ in 0..200 {
        h.step(vec![], vec![]);
    }
    let sc = h.sc();
    assert_ne!((sc.player_cell_x, sc.player_cell_y), target, "不得走到");
    assert!(sc.entities.items[..sc.entities.count].iter().filter(|e| e.type_ == 6 && !e.removed).all(|e| !e.visible), "隐形路径不显形");
}

/// 23 层：隐形墙（型 12，看起来是地面）——点它：路线照常走过去，撞上即显形并停在墙前，玩家从不站上隐形墙格（与方向键同效）。
#[test]
fn tap_into_hidden_wall_reveals_and_blocks_floor23() {
    let mut h = to_map("hidden23");
    h.sc_mut().change_floor(23, false, false);
    for _ in 0..60 {
        h.step(vec![], vec![]);
    }
    // 目标 = 首个经地形可走格（独立 BFS）可达、在屏的未显形型 12 格
    let (target, e12) = {
        let sc = h.sc();
        let reach = terrain_reach(sc);
        let on_screen = |x: i32, y: i32| {
            let (sx, sy) = (sc.view.cam_x + (x << 5), sc.view.cam_y + sc.layout.view_top_y + (y << 5));
            sx >= 0 && sy >= sc.layout.view_top_y && sx + 32 <= sc.layout.screen_w && sy + 32 <= sc.layout.view_top_y + sc.view.view_h
        };
        let found = (0..sc.entities.count)
            .map(|i| (i, &sc.entities.items[i]))
            .find(|(_, e)| e.type_ == 12 && !e.visible && !e.removed && reach.contains(&(e.px >> 5, e.py >> 5)) && on_screen(e.px >> 5, e.py >> 5))
            .expect("有可达在屏的未显形隐形墙");
        (((found.1.px >> 5, found.1.py >> 5)), found.0)
    };
    let walls12: Vec<(i32, i32)> = { let sc = h.sc(); sc.entities.items[..sc.entities.count].iter().filter(|e| e.type_ == 12 && !e.removed).map(|e| (e.px >> 5, e.py >> 5)).collect() };
    let (sx, sy) = { let sc = h.sc(); (sc.view.cam_x + (target.0 << 5) + 16, sc.view.cam_y + sc.layout.view_top_y + (target.1 << 5) + 16) };
    h.step(vec![], vec![(sx as f32, sy as f32)]);
    for _ in 0..240 {
        h.step(vec![], vec![]);
        let me = (h.sc().player_cell_x, h.sc().player_cell_y);
        assert!(!walls12.contains(&me), "玩家站上隐形墙格 {me:?}");
    }
    assert!(h.sc().entities.items[e12].visible, "撞上的隐形墙显形");
    assert_ne!((h.sc().player_cell_x, h.sc().player_cell_y), target);
}

/// 寻路中再点：排队改道（生产路径 scene_tap → tap_cell）。
#[test]
fn tap_during_walk_reroutes() {
    let mut h = to_map("queue");
    // 教程层（floor 1）带脚本剧情：打断排空后 dp 机被剧情接管（dp1 等键），排队按 spec"剧情接管 ⇒ 丢弃"
    // 属正确行为——本用例钉"排队改道"本身，换无剧情层。
    h.sc_mut().change_floor(5, false, false);
    for _ in 0..60 {
        h.step(vec![], vec![]);
    }
    let spawn = { let sc = h.sc(); (sc.player_cell_x, sc.player_cell_y) };
    // 可达在屏空格表（tap_walk_case 同式：地形可走、无实体槽、非危险）
    let cells: Vec<(i32, i32)> = {
        let sc = h.sc();
        let grid = sc.tap_grid();
        let reach = terrain_reach(sc);
        let on_screen = |x: i32, y: i32| {
            let (sx, sy) = (sc.view.cam_x + (x << 5), sc.view.cam_y + sc.layout.view_top_y + (y << 5));
            sx >= 0 && sy >= sc.layout.view_top_y && sx + 32 <= sc.layout.screen_w && sy + 32 <= sc.layout.view_top_y + sc.view.view_h
        };
        reach.into_iter()
            .filter(|&(x, y)| {
                on_screen(x, y)
                    && sc.grid.capacity[sc.grid.cell_type[y as usize][x as usize] as usize] == 0
                    && !grid.get(x, y).map_or(false, |c| c.danger)
            })
            .collect()
    };
    assert!(cells.len() >= 3, "前置：可达在屏空格 ≥3");
    let a = *cells.iter().max_by_key(|&&c| (c.0 - spawn.0).abs() + (c.1 - spawn.1).abs()).unwrap();
    let b = *cells
        .iter()
        .filter(|&&c| c != spawn)
        .max_by_key(|&&c| (c.0 - a.0).abs() + (c.1 - a.1).abs())
        .unwrap();
    let tap_cell_at = |h: &mut H, c: (i32, i32)| {
        let (sx, sy) = { let sc = h.sc(); (sc.view.cam_x + (c.0 << 5) + 16, sc.view.cam_y + sc.layout.view_top_y + (c.1 << 5) + 16) };
        h.step(vec![], vec![(sx as f32, sy as f32)]);
    };
    tap_cell_at(&mut h, a);
    for _ in 0..120 {
        let sc = h.sc();
        let at = (sc.player_cell_x, sc.player_cell_y);
        // 等到玩家既离开出生格、也不在 b 格上（点自己 = 停下，不是排队），且无浮层
        if sc.game_mode == GameMode::Dialog && at != spawn && at != b && !sc.overlay_active {
            break;
        }
        h.step(vec![], vec![]);
    }
    {
        let sc = h.sc();
        assert_eq!(sc.game_mode, GameMode::Dialog, "前置：寻路中");
        assert_ne!((sc.player_cell_x, sc.player_cell_y), spawn, "前置：已离开出生格");
        assert_ne!((sc.player_cell_x, sc.player_cell_y), b, "前置：不在 b 格上");
    }
    tap_cell_at(&mut h, b);
    assert_eq!(h.sc().tap_target, Some(b), "第二点 = 排队：目标标记立即换到 b");
    for _ in 0..300 {
        // 排空落格与排队执行之间有一拍间隙（Map wp0、路线已收、标记未清）：以标记清除为"全部收尾"判据
        if h.sc().game_mode == GameMode::Map && h.sc().walk_phase == 0 && !h.sc().route.is_open() && h.sc().tap_target.is_none() {
            break;
        }
        // 途中拾取提示浮层：按生产口径投 -5 关框（关框键属浮层，不清排队），关框后照常改道
        if h.sc().overlay_active && h.sc().game_mode == GameMode::Map {
            h.step(tap(-5), vec![]);
            h.step(vec![KeyAction::Release], vec![]);
            continue;
        }
        h.step(vec![], vec![]);
    }
    assert_eq!((h.sc().player_cell_x, h.sc().player_cell_y), b, "改道到第二次点中的格");
    assert_eq!(h.sc().tap_target, None, "收尾后无残留标记");
}

/// 战斗中点画面：不代按、不打断战斗，闪红（scene_tap 直调；战斗态由场景字段构造，不跑 tick）。
#[test]
fn tap_during_battle_is_rejected_without_key() {
    let mut h = to_map("battle");
    let (walk, _) = field_points(&h);
    let sc = h.sc_mut();
    sc.walk_phase = 5;
    let key = game_android::native::scene_tap(sc, walk.0 as i32, walk.1 as i32);
    assert_eq!(key, None, "不代按");
    assert_eq!(sc.walk_phase, 5, "战斗不被打断");
    assert!(sc.tap_reject.is_some(), "闪红");
}

/// 【计划 2 T11 执行期补充 · 复核 T10-12 R1】点地路线途经特殊拾取物弹框：经**生产路径**（Runtime::step 投 −5
/// → replay_scene_keys 的 interrupt_route_for_input 浮层期零操作 + press_key 关框）后继续走到目标。
/// 目标取玩家正前方 2 格（曼哈顿距离 2 ⇒ 任何最短路必经中间格），中间格放拾取型 25（踩上弹提示框）。
#[test]
fn pickup_overlay_mid_route_closes_and_resumes_via_production_keys() {
    let mut h = to_map("ovlresume");
    // 教程层（floor 1）带脚本剧情（踩到箭头末端格触发对话、剧情接管丢路线——spec 正确行为），换无剧情层
    h.sc_mut().change_floor(5, false, false);
    for _ in 0..60 {
        h.step(vec![], vec![]);
    }
    // 目标 = 新格分类下 ≥2 步可达的在屏地面格；拾取格 = 该路线（BFS 确定性）的首步格（必经）
    let (target, pick) = {
        let sc = h.sc();
        let from = (sc.player_cell_x, sc.player_cell_y);
        let reach = terrain_reach(sc);
        let in_map = |x: i32, y: i32| x >= 0 && y >= 0 && x < sc.view.wide && y < sc.view.high;
        let free = |x: i32, y: i32| {
            in_map(x, y)
                && sc.walkable[y as usize][x as usize]
                && sc.grid.capacity[sc.grid.cell_type[y as usize][x as usize] as usize] == 0
        };
        let on_screen = |x: i32, y: i32| {
            let (sx, sy) = (sc.view.cam_x + (x << 5), sc.view.cam_y + sc.layout.view_top_y + (y << 5));
            sx >= 0 && sy >= sc.layout.view_top_y && sx + 32 <= sc.layout.screen_w && sy + 32 <= sc.layout.view_top_y + sc.view.view_h
        };
        let step_to = |at: (i32, i32), d: u8| -> (i32, i32) {
            match d {
                0 => (at.0, at.1 + 1),
                1 => (at.0, at.1 - 1),
                2 => (at.0 + 1, at.1),
                _ => (at.0 - 1, at.1),
            }
        };
        let grid = sc.tap_grid();
        let mut out = None;
        'outer: for &t in reach.iter() {
            if t == from || !on_screen(t.0, t.1) || !free(t.0, t.1) {
                continue;
            }
            if let TapPlan::Walk { steps, .. } = game_core::tap_walk::plan_tap(&grid, from, t) {
                if steps.len() < 2 {
                    continue;
                }
                // 步序目标端优先 ⇒ 逆序消费还原实际途经格序；首步格 = 途径格中最靠近玩家、必经
                let mut at = from;
                let mut first = from;
                for (i, &d) in steps.iter().rev().enumerate() {
                    at = step_to(at, d);
                    if i == 0 {
                        first = at;
                    }
                }
                if free(first.0, first.1) {
                    out = Some((t, first));
                    break 'outer;
                }
            }
        }
        out.expect("有 ≥2 步可达、途径格可放拾取物的目标")
    };
    // 中间格放拾取型 25（append-only 实体表 + 重登格槽）
    {
        let sc = h.sc_mut();
        let idx = sc.entities.count;
        sc.entities.items[idx] = game_core::entity::Entity {
            type_: 25,
            px: pick.0 << 5,
            py: pick.1 << 5,
            ..Default::default()
        };
        sc.entities.count += 1;
        game_core::entity::reattach_to_cell(&mut sc.grid, &mut sc.entities, idx);
    }
    let (sx, sy) = { let sc = h.sc(); (sc.view.cam_x + (target.0 << 5) + 16, sc.view.cam_y + sc.layout.view_top_y + (target.1 << 5) + 16) };
    h.step(vec![], vec![(sx as f32, sy as f32)]);
    assert!(h.sc().route.is_open(), "前置：开寻路");
    // 走到拾取格：弹提示框（确定性——任何 2 步最短路必经中间格）
    let mut popped = false;
    for _ in 0..120 {
        h.step(vec![], vec![]);
        if h.sc().overlay_active {
            popped = true;
            break;
        }
    }
    assert!(popped, "前置：踩拾取格弹提示框");
    assert_eq!(h.sc().overlay_kind, 1, "拾取说明框（kind 1）");
    // 生产路径关框：Runtime::step 投 −5（replay_scene_keys：浮层期 interrupt_route_for_input 零操作 + press_key）
    h.step(tap(-5), vec![]);
    h.step(vec![KeyAction::Release], vec![]);
    assert!(!h.sc().overlay_active, "提示框已关（且未打断路线）");
    for _ in 0..120 {
        if h.sc().game_mode == GameMode::Map && h.sc().walk_phase == 0 && !h.sc().route.is_open() && h.sc().tap_target.is_none() {
            break;
        }
        h.step(vec![], vec![]);
    }
    assert_eq!((h.sc().player_cell_x, h.sc().player_cell_y), target, "关框后继续走到目标");
}
