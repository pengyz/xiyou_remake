//! boot 启动屏回放（P3 渲染层）：Rust `BootMachine` 重放 A-boot-menu trace
//! 的 mode 0 阶段，**逐 tick** 比对 FRAME sha（TICK n 的帧 = 第 n 次逻辑推进
//! **后**的 paint，时序合同见 `game-core/src/paint.rs` BootMachine 注释）。
//!
//! 覆盖：l0/l1 加载轮播（T1-T31）、logo 时间线 1..N（T32 起）。
//! mode 21 切换（T67）之后不在本测试范围。

use game_core::paint::BootMachine;
use game_core::render::{ArgbImage, SoftGraphics};
use game_oracle::trace;

fn repo() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn decode_png(name: &str) -> ArgbImage {
    let bytes = std::fs::read(repo().join("assets/raw").join(name)).unwrap();
    ArgbImage::from_decoded_png(game_data::decode_png(&bytes).unwrap())
}

fn load_sflogo() -> Vec<ArgbImage> {
    let data = std::fs::read(repo().join("assets/raw/sflogo")).unwrap();
    game_data::PackedPng::parse(&data, 8)
        .unwrap()
        .images
        .iter()
        .map(|sub| ArgbImage::from_decoded_png(game_data::decode_png(sub.bytes).unwrap()))
        .collect()
}

/// 逐 tick 重放：每 tick paint → 比对 FRAME sha → tick 逻辑。/// 逐 tick 重放：每 tick paint → 比对 FRAME sha → tick 逻辑。
#[test]
fn boot_menu_frames_match_tick_by_tick() {
    let trace_text = std::fs::read_to_string(
        repo().join("reference/oracle/_out/A-boot-menu/trace.txt"),
    )
    .expect("缺 A-boot-menu trace（先跑 python3 reference/oracle/run.py --scenarios）");
    let records = trace::parse(&trace_text);
    assert!(records.len() >= 40, "trace 过短");

    let mut machine = BootMachine::new(decode_png("l0.png"), decode_png("l1.png"), load_sflogo());
    let mut checked = 0usize;
    for rec in &records {
        // TICK n 的帧 = 第 n 次逻辑推进后的 paint（时序合同：logic 先 paint 后）
        machine.tick();
        if machine.finished {
            // mode 0 结束（timeline 35 耗尽，logic#68 清理切 gameMode=21）：
            // TICK 68 起是 mode 21 分支，属下一批端口范围
            break;
        }
        let mut screen = ArgbImage::create(240, 320);
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, 240, 320); // serviceRepaints 入口（Canvas.java:71）
            g.set_font(Some(game_core::paint::paint_font()));
            machine.paint(&mut g);
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        if let Some(expect) = &rec.frame_sha {
            assert_eq!(
                &sha, expect,
                "TICK {} 帧哈希不符（paints={}）",
                rec.tick, machine.paints
            );
            checked += 1;
        }
    }
    // 覆盖锚：l0 轮播 T1-16 + l1 轮播 T17-31 + logo 时间线 T32-67（FRAME 行逐 tick 全有）
    assert_eq!(checked, 67, "mode 0 全程 67 帧必须全部比对");
}
