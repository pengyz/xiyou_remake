//! Android 运行时无头流程截帧（`#[ignore]` 手动工具；AGENTS §5"验证不打扰"无头路径）：
//! 与 android_main **同一调度体**（runtime.rs）+ 物理背板 2×（canvas_graphics）+ 宿主同族
//! CJK 字体（NotoSansCJK SC 面），每拍 step+paint（设备主循环同构），在关键时刻落 PPM。
//!
//! 用法：`JJXY_SHOTS=/tmp/jjxy-shots cargo test -p game-android --release \
//!        --test headless_flow_shots -- --ignored --nocapture`（缺省输出 /tmp/jjxy-shots）
//! 帧：1_title 2_intro 3_scene 4_dialog 5_map 6_menu（1152² P6）。

use game_android::input::KeyAction;
use game_android::native::{canvas_graphics, TitleGesture};
use game_android::runtime::{build_boot_machine, Runtime};
use game_core::enums::GameMode;
use game_core::render::ArgbImage;

fn write_ppm(dir: &std::path::Path, name: &str, img: &ArgbImage) {
    let mut out = format!("P6\n{} {}\n255\n", img.width, img.height).into_bytes();
    for p in &img.argb {
        out.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, *p as u8]);
    }
    std::fs::write(dir.join(format!("{name}.ppm")), out).unwrap();
}

#[test]
#[ignore]
fn headless_flow_shots() {
    let font = game_shell_common::font::install(
        &[("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc".to_string(), 2)],
        &mut |m| eprintln!("{m}"),
    );
    let out = std::path::PathBuf::from(std::env::var("JJXY_SHOTS").unwrap_or_else(|_| "/tmp/jjxy-shots".into()));
    std::fs::create_dir_all(&out).unwrap();
    let raw = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw");
    let src = game_shell_common::asset::DirAssetSource::new(&raw);
    let save = out.join("save");
    let _ = std::fs::remove_dir_all(&save);
    let (rms, stores) = game_shell_common::rms::FileRms::load(&save);
    let machine = build_boot_machine(&src, game_core::layout::Layout::modern(18, 18));
    let mut rt = Runtime::new(machine, stores, Box::new(rms));
    let mut canvas = ArgbImage::create(1152, 1152);
    let mut drive = |rt: &mut Runtime, canvas: &mut ArgbImage, keys: &[KeyAction], g: Vec<TitleGesture>, t: Vec<(f32, f32)>| {
        rt.step(&src, keys, g, t);
        let mut gfx = canvas_graphics(canvas, 2);
        rt.paint(&mut gfx);
    };
    let tap = vec![(288.0, 300.0)];
    let mut n = 0;
    while !rt.title_mode() && n < 300 {
        drive(&mut rt, &mut canvas, &[], vec![], vec![]);
        n += 1;
    }
    for _ in 0..30 {
        drive(&mut rt, &mut canvas, &[], vec![], vec![]);
    }
    write_ppm(&out, "1_title", &canvas);
    let geo = rt.machine.title_geom().unwrap();
    let ok = ((geo.zone_ok.0 + geo.zone_ok.2 / 2) as f32, (geo.zone_ok.1 + geo.zone_ok.3 / 2) as f32);
    drive(&mut rt, &mut canvas, &[], vec![TitleGesture::Tap(ok.0, ok.1)], vec![]);
    for _ in 0..60 {
        drive(&mut rt, &mut canvas, &[], vec![], vec![]);
    }
    write_ppm(&out, "2_intro", &canvas);
    n = 0;
    while rt.scene.is_none() && n < 3000 {
        drive(&mut rt, &mut canvas, &[], vec![], if n % 3 == 0 { tap.clone() } else { vec![] });
        n += 1;
    }
    for _ in 0..12 {
        drive(&mut rt, &mut canvas, &[], vec![], vec![]);
    }
    write_ppm(&out, "3_scene", &canvas);
    let (mut k, mut calm, mut dialog) = (0, 0, false);
    while k < 4000 && calm < 30 {
        let mode = rt.scene.as_ref().unwrap().game_mode;
        if mode != GameMode::Map && !dialog && k > 20 {
            write_ppm(&out, "4_dialog", &canvas);
            dialog = true;
        }
        calm = if mode == GameMode::Map { calm + 1 } else { 0 };
        drive(&mut rt, &mut canvas, &[], vec![], if mode != GameMode::Map && k % 4 == 0 { tap.clone() } else { vec![] });
        k += 1;
    }
    write_ppm(&out, "5_map", &canvas);
    drive(&mut rt, &mut canvas, &[KeyAction::Press(-7), KeyAction::Release], vec![], vec![]);
    for _ in 0..6 {
        drive(&mut rt, &mut canvas, &[], vec![], vec![]);
    }
    write_ppm(&out, "6_menu", &canvas);
    eprintln!("font={font:?} intro_ticks={n} dialog_ticks={k} crashed={} → {}", rt.crashed, out.display());
    assert!(!rt.crashed);
}
