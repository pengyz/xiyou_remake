//! 竖屏整窗无头预览（`#[ignore]` 手动工具；AGENTS §5"验证不打扰"无头路径）：
//! Xiaomi 14 实测输入（1200×2536 / 480dpi / 全关卡最大格数）→ plan_portrait →
//! 每拍 step + 活动档画布 paint + 面板 paint → compose_layers 整窗（android_main 竖屏分支同构）→ PPM。
//!
//! 用法：`JJXY_SHOTS=/tmp/jjxy-portrait cargo test -p game-android --release \
//!        --test headless_portrait_shots -- --ignored --nocapture`
//! 帧：1_title 2_intro 3_scene 4_map_hold_right 4b_exit_confirm 5_menu（整窗 1200×2536）。

use game_android::input::KeyAction;
use game_android::native::{canvas_graphics, TitleGesture};
use game_android::portrait::{compose_layers, paint_panel, plan_portrait, MapExtent, PanelState, PlanInput, PortraitPlan};
use game_android::runtime::{build_boot_machine, Runtime};
use game_core::enums::GameMode;
use game_core::render::ArgbImage;

struct Host {
    plan: PortraitPlan,
    canvas: ArgbImage,
    panel: ArgbImage,
    icons: ArgbImage,
}

impl Host {
    /// 主循环一拍（native.rs 竖屏分支同序：活动档 → 画布 paint → 面板 paint → 合成）。
    fn frame(&mut self, rt: &mut Runtime, held: Vec<i32>) -> Vec<u8> {
        let c = rt.active_canvas(&self.plan);
        let (cw, ch) = c.physical();
        if (self.canvas.width, self.canvas.height) != (cw, ch) {
            self.canvas = ArgbImage::create(cw, ch);
        }
        {
            let mut g = canvas_graphics(&mut self.canvas, c.scale as u32);
            rt.paint(&mut g);
        }
        paint_panel(&mut self.panel, &self.plan.geom, &PanelState { held, softkeys: rt.softkeys() }, &self.icons);
        let (w, h) = self.plan.window;
        let cr = c.rect_in(self.plan.game);
        let p = self.plan.panel;
        let mut buf = vec![0u8; (w * h * 4) as usize];
        compose_layers(
            &mut buf,
            w,
            h,
            w,
            &[(&self.canvas.argb, cw, ch, cr.x, cr.y), (&self.panel.argb, p.w, p.h, p.x, p.y)],
        );
        buf
    }
}

fn write_ppm(dir: &std::path::Path, name: &str, w: i32, h: i32, rgba: &[u8]) {
    let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
    for px in rgba.chunks_exact(4) {
        out.extend_from_slice(&px[..3]);
    }
    std::fs::write(dir.join(format!("{name}.ppm")), out).unwrap();
}

#[test]
#[ignore]
fn headless_portrait_shots() {
    let font = game_shell_common::font::install(
        &[("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc".to_string(), 2)],
        &mut |m| eprintln!("{m}"),
    );
    let out = std::path::PathBuf::from(std::env::var("JJXY_SHOTS").unwrap_or_else(|_| "/tmp/jjxy-portrait".into()));
    std::fs::create_dir_all(&out).unwrap();
    let raw = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw");
    let src = game_shell_common::asset::DirAssetSource::new(&raw);
    let plan = plan_portrait(&PlanInput {
        window: (1200, 2536),
        dpi: 480,
        inset_top: 0,
        inset_bottom: 0,
        map: MapExtent::from_assets(&src),
    });
    eprintln!("plan scene={:?} boot={:?} game={:?} panel={:?}", plan.scene, plan.boot, plan.game, plan.panel);
    let save = out.join("save");
    let _ = std::fs::remove_dir_all(&save);
    let (rms, stores) = game_shell_common::rms::FileRms::load(&save);
    let mut rt = Runtime::new(build_boot_machine(&src, plan.scene.layout()), stores, Box::new(rms));
    rt.apply_layouts(plan.boot.layout(), plan.scene.layout());
    let (w, h) = plan.window;
    let mut host = Host {
        canvas: ArgbImage::create(1, 1),
        panel: ArgbImage::create(plan.panel.w, plan.panel.h),
        icons: game_shell_common::handoff::load_container(&src, "ui", 25)[11].clone(),
        plan,
    };
    // 活动档逻辑画布中心（点按推进对话/序章用）
    let center = |rt: &Runtime, host: &Host| {
        let (lw, lh) = rt.active_canvas(&host.plan).logical();
        vec![(lw as f32 / 2.0, lh as f32 / 2.0)]
    };
    let mut step = |rt: &mut Runtime, host: &mut Host, keys: &[KeyAction], g: Vec<TitleGesture>, t: Vec<(f32, f32)>| {
        rt.step(&src, keys, g, t);
        host.frame(rt, Vec::new())
    };
    let mut n = 0;
    let mut last = Vec::new();
    while !rt.title_mode() && n < 300 {
        last = step(&mut rt, &mut host, &[], vec![], vec![]);
        n += 1;
    }
    for _ in 0..30 {
        last = step(&mut rt, &mut host, &[], vec![], vec![]);
    }
    write_ppm(&out, "1_title", w, h, &last);
    let geo = rt.machine.title_geom().unwrap();
    let ok = ((geo.zone_ok.0 + geo.zone_ok.2 / 2) as f32, (geo.zone_ok.1 + geo.zone_ok.3 / 2) as f32);
    step(&mut rt, &mut host, &[], vec![TitleGesture::Tap(ok.0, ok.1)], vec![]);
    for _ in 0..60 {
        last = step(&mut rt, &mut host, &[], vec![], vec![]);
    }
    write_ppm(&out, "2_intro", w, h, &last);
    n = 0;
    while rt.scene.is_none() && n < 3000 {
        let t = if n % 3 == 0 { center(&rt, &host) } else { vec![] };
        last = step(&mut rt, &mut host, &[], vec![], t);
        n += 1;
    }
    for _ in 0..12 {
        last = step(&mut rt, &mut host, &[], vec![], vec![]);
    }
    write_ppm(&out, "3_scene", w, h, &last);
    let (mut k, mut calm) = (0, 0);
    while k < 4000 && calm < 30 {
        let mode = rt.scene.as_ref().unwrap().game_mode;
        calm = if mode == GameMode::Map { calm + 1 } else { 0 };
        let t = if mode != GameMode::Map && k % 4 == 0 { center(&rt, &host) } else { vec![] };
        step(&mut rt, &mut host, &[], vec![], t);
        k += 1;
    }
    // 按住"右"一拍：面板按下反馈
    rt.step(&src, &[], vec![], vec![]);
    last = host.frame(&mut rt, vec![-4]);
    write_ppm(&out, "4_map_hold_right", w, h, &last);
    // 【STAGE-B-005】地图态系统返回 ⇒ 退出确认框（kind 4 样式 + 面板 ○ / ×）；右软键 -7 取消后再走原流程
    assert!(rt.back_opens_exit_confirm(), "地图态返回 = 退出确认框");
    for _ in 0..4 {
        last = step(&mut rt, &mut host, &[], vec![], vec![]);
    }
    write_ppm(&out, "4b_exit_confirm", w, h, &last);
    step(&mut rt, &mut host, &[KeyAction::Press(-7), KeyAction::Release], vec![], vec![]);
    assert!(!rt.scene.as_ref().unwrap().exit_confirm_open(), "-7 取消");
    step(&mut rt, &mut host, &[KeyAction::Press(-7), KeyAction::Release], vec![], vec![]);
    for _ in 0..6 {
        last = step(&mut rt, &mut host, &[], vec![], vec![]);
    }
    write_ppm(&out, "5_menu", w, h, &last);
    // 游戏内菜单子页（用户报"存档UI没有居中"）：光标移到目标项 → 确定 → 截帧 → 右软键返回
    for (target, mode, name) in
        [(2, GameMode::SaveSlot, "6_save"), (3, GameMode::About, "7_settings"), (4, GameMode::Help, "8_help")]
    {
        if rt.scene.as_ref().unwrap().game_mode != GameMode::InGameMenu {
            step(&mut rt, &mut host, &[KeyAction::Press(-7), KeyAction::Release], vec![], vec![]);
            step(&mut rt, &mut host, &[], vec![], vec![]);
        }
        let mut guard = 0;
        while rt.scene.as_ref().unwrap().menu_cursor != target && guard < 12 {
            step(&mut rt, &mut host, &[KeyAction::Press(-2), KeyAction::Release], vec![], vec![]);
            step(&mut rt, &mut host, &[], vec![], vec![]);
            guard += 1;
        }
        step(&mut rt, &mut host, &[KeyAction::Press(-5), KeyAction::Release], vec![], vec![]);
        for _ in 0..8 {
            last = step(&mut rt, &mut host, &[], vec![], vec![]);
        }
        assert_eq!(rt.scene.as_ref().unwrap().game_mode, mode, "{name}：进入子页");
        write_ppm(&out, name, w, h, &last);
        step(&mut rt, &mut host, &[KeyAction::Press(-7), KeyAction::Release], vec![], vec![]);
        for _ in 0..4 {
            step(&mut rt, &mut host, &[], vec![], vec![]);
        }
    }
    eprintln!("font={font:?} intro_ticks={n} dialog_ticks={k} crashed={} → {}", rt.crashed, out.display());
    assert!(!rt.crashed);
}

/// 【BUG-029/030 实机管线无头核对】Xiaomi 14 竖屏整窗：story-fullrun 存档（天牢 2 层）读档 → 踩点自走 →
/// 牛魔王对话页（说话者 72，对话框在屏）截整窗；再走一次金币弹字（`spawn(4,…)`，与击杀两处同入口）截整窗。
///
/// 用法：`JJXY_SHOTS=/tmp/jjxy-portrait cargo test -p game-android --release --test headless_portrait_shots \
///        -- --ignored headless_portrait_niumowang_dialog_and_gold_popup --nocapture`
/// 帧：9_niumowang_dialog 10_gold_popup（整窗 1200×2536 PPM）。
#[test]
#[ignore]
fn headless_portrait_niumowang_dialog_and_gold_popup() {
    let _ = game_shell_common::font::install(
        &[("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc".to_string(), 2)],
        &mut |m| eprintln!("{m}"),
    );
    let out = std::path::PathBuf::from(std::env::var("JJXY_SHOTS").unwrap_or_else(|_| "/tmp/jjxy-portrait".into()));
    std::fs::create_dir_all(&out).unwrap();
    let raw = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw");
    let src = game_shell_common::asset::DirAssetSource::new(&raw);
    let plan = plan_portrait(&PlanInput {
        window: (1200, 2536),
        dpi: 480,
        inset_top: 0,
        inset_bottom: 0,
        map: MapExtent::from_assets(&src),
    });
    let save = out.join("save-niumowang");
    let _ = std::fs::remove_dir_all(&save);
    let (rms, _) = game_shell_common::rms::FileRms::load(&save);
    let stores = game_core::save::parse_preset(include_str!("../../../crates/game-oracle/tests/fixtures/story-fullrun.preset"));
    let mut rt = Runtime::new(build_boot_machine(&src, plan.scene.layout()), stores, Box::new(rms));
    rt.apply_layouts(plan.boot.layout(), plan.scene.layout());
    let (w, h) = plan.window;
    let mut host = Host {
        canvas: ArgbImage::create(1, 1),
        panel: ArgbImage::create(plan.panel.w, plan.panel.h),
        icons: game_shell_common::handoff::load_container(&src, "ui", 25)[11].clone(),
        plan,
    };
    let tap = |c: i32| vec![KeyAction::Press(c), KeyAction::Release];
    // 推进拍只画活动档画布（逻辑推进需 paint：打字机/遮幅挂 paint 尾）；整窗合成只在截帧时做
    let mut step = |rt: &mut Runtime, host: &mut Host, keys: &[KeyAction], g: Vec<TitleGesture>| {
        rt.step(&src, keys, g, vec![]);
        let c = rt.active_canvas(&host.plan);
        let (cw, ch) = c.physical();
        if (host.canvas.width, host.canvas.height) != (cw, ch) {
            host.canvas = ArgbImage::create(cw, ch);
        }
        let mut gfx = canvas_graphics(&mut host.canvas, c.scale as u32);
        rt.paint(&mut gfx);
    };
    // 标题 → 继续游戏（轮播 1）→ 读档槽 → 确认 → 读条交接
    let mut n = 0;
    while !rt.title_mode() && n < 300 {
        step(&mut rt, &mut host, &[], vec![]);
        n += 1;
    }
    let geo = rt.machine.title_geom().expect("标题");
    let mid = |z: (i32, i32, i32, i32)| ((z.0 + z.2 / 2) as f32, (z.1 + z.3 / 2) as f32);
    while rt.machine.title_cursor() != Some(1) {
        let p = mid(geo.zone_next);
        step(&mut rt, &mut host, &[], vec![TitleGesture::Tap(p.0, p.1)]);
    }
    let ok = mid(geo.zone_ok);
    step(&mut rt, &mut host, &[], vec![TitleGesture::Tap(ok.0, ok.1)]);
    assert_eq!(rt.machine.mode, GameMode::EquipDetail, "继续游戏 → 读档槽");
    step(&mut rt, &mut host, &tap(-5), vec![]);
    n = 0;
    while rt.scene.is_none() && n < 600 {
        step(&mut rt, &mut host, &[], vec![]);
        n += 1;
    }
    assert_eq!(rt.scene.as_ref().expect("读档交接").floor, 2, "天牢 2 层");
    // 踩点自走 → 对话；推页直到牛魔王（72）说话页的对话框在屏
    let mut k = 0;
    let mut holding = false;
    let mut seen = String::new();
    loop {
        let sc = rt.scene.as_ref().unwrap();
        let ev = format!(
            "f{} {:?} pg{} sp{} ph{} pl({},{})",
            sc.floor,
            sc.game_mode,
            sc.dialog_page,
            sc.dialog_speaker,
            sc.dialog_phase,
            sc.view.player_px >> 5,
            sc.view.player_py >> 5
        );
        if ev != seen {
            if std::env::var("JJXY_TRACE").is_ok() {
                eprintln!("k={k} {ev}");
            }
            seen = ev;
        }
        let shown = sc.game_mode == GameMode::Dialog
            && sc.dialog_speaker == 72
            && sc.overlay_text.is_some()
            && !sc.overlay_active
            && sc.dialog_phase != 4;
        if shown {
            for _ in 0..12 {
                step(&mut rt, &mut host, &[], vec![]); // 打字机走几拍
            }
            break;
        }
        assert!(k < 8000, "8000 拍内未见牛魔王说话页（{seen}）");
        // Map：按住"下"直到起步/进剧情再抬起（一格一按，script_walk_taps 同口径）；Dialog：逐页确认
        let keys = match sc.game_mode {
            GameMode::Dialog => {
                if holding {
                    holding = false;
                    vec![KeyAction::Release]
                } else if k % 20 == 0 {
                    tap(-5)
                } else {
                    vec![]
                }
            }
            GameMode::Map if !holding && k % 40 == 0 => {
                holding = true;
                vec![KeyAction::Press(-2)]
            }
            GameMode::Map if holding && sc.walk_phase != 0 => {
                holding = false;
                vec![KeyAction::Release]
            }
            _ => vec![],
        };
        step(&mut rt, &mut host, &keys, vec![]);
        k += 1;
    }
    let mut last = host.frame(&mut rt, Vec::new());
    write_ppm(&out, "9_niumowang_dialog", w, h, &last);
    // 金币弹字（kind 4）：同击杀两处 spawn 入口
    {
        let sc = rt.scene.as_mut().unwrap();
        let dw = sc.images.popup.digit_strips[0].width / 11;
        sc.popups.spawn(4, 1, 0, 0, sc.view.cam_x, sc.view.cam_y, dw);
    }
    rt.step(&src, &[], vec![], vec![]);
    last = host.frame(&mut rt, Vec::new());
    write_ppm(&out, "10_gold_popup", w, h, &last);
    eprintln!("load_ticks={n} dialog_ticks={k} crashed={} → {}", rt.crashed, out.display());
    assert!(!rt.crashed);
}
