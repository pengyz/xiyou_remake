//! boot 启动屏回放（P3 渲染层）：Rust `BootMachine` 重放 A-boot-menu trace
//! 的 mode 0 + mode 21 阶段，**逐 tick** 比对 FRAME sha（TICK n 的帧 = 第 n 次
//! 逻辑推进**后**的 paint，时序合同见 `game-core/src/paint.rs` BootMachine 注释）。
//!
//! 覆盖：l0/l1 加载轮播（T1-31）、logo 时间线（T32-67）、声音询问（T68-70，
//! T70 的 -6 按键切 gameMode=1 后为 title 菜单，属下一批端口范围）。

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

fn load_container(name: &str, count: usize) -> Vec<ArgbImage> {
    let data = std::fs::read(repo().join("assets/raw").join(name)).unwrap();
    game_data::PackedPng::parse(&data, count)
        .unwrap()
        .images
        .iter()
        .map(|sub| ArgbImage::from_decoded_png(game_data::decode_png(sub.bytes).unwrap()))
        .collect()
}

fn load_sflogo() -> Vec<ArgbImage> {
    load_container("sflogo", 8)
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

    let mut machine = BootMachine::new(
        decode_png("l0.png"),
        decode_png("l1.png"),
        load_sflogo(),
        load_container("ui", 25),
    );
    let mut checked = 0usize;
    // 按键投递时序：TICK n 的 INPUT 行在 preTick(n)（paint#n 之后）投递，
    // 由 logic#(n+1) 消费（run 循环体开头读 keyValue）⇒ 延迟一拍传入 tick()
    let mut pending_key = 0i32;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        machine.tick(key);
        if machine.finished {
            // gameMode 已切 1（title 菜单未端口）：T70 press(-6) 由 logic#71 消费
            assert_eq!(rec.tick, 71, "模式切换应在 T71（T70 的 -6 延迟一拍消费）");
            assert_eq!(machine.mode, 1, "-6 ⇒ gameMode=1（a.java:4213-4220）");
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
                "TICK {} 帧哈希不符（paints={} mode={}）",
                rec.tick, machine.paints, machine.mode
            );
            checked += 1;
        }
    }
    // 覆盖锚：l0 轮播 T1-16 + l1 T17-31 + logo T32-67 + 声音询问 T68-70
    assert_eq!(checked, 70, "mode 0+21 全程 70 帧必须全部比对");
}

/// INPUT 行 → 本 tick 边界投递的按键码（keyPressed；release 只清 keyHeldCode，
/// 对 keyValue 语义无影响——a.java:4639/4654）。同 tick 多键取首个 press。
fn input_key(input: &Option<String>) -> i32 {
    let Some(text) = input else { return 0 };
    for ev in text.split(' ') {
        if let Some(rest) = ev.strip_prefix("press(").and_then(|r| r.strip_suffix(')')) {
            if let Ok(code) = rest.parse::<i32>() {
                return code;
            }
        }
    }
    0
}
