//! boot→title 回放（P3 渲染层）：Rust `BootMachine` 重放 A-menu-sweep trace
//! 的 mode 0 + mode 21 + mode 1 阶段，**逐 tick** 比对 FRAME sha（TICK n 的帧
//! = 第 n 次逻辑推进**后**的 paint；INPUT 在 preTick(n) 投递、logic#(n+1) 消费）。
//!
//! 覆盖：l0/l1 轮播（T1-31）、logo 时间线（T32-67）、声音询问（T68-70）、
//! title 菜单（T71-151：图标带动画、bob 箭头、粒子系统、光标移动 T90 -2、
//! T150 -5 确认「继续游戏」→ gameMode=8 后为读档链，属下一批范围）。

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
        repo().join("reference/oracle/_out/A-menu-sweep/trace.txt"),
    )
    .expect("缺 A-menu-sweep trace（先跑 python3 reference/oracle/run.py --scenarios）");
    let records = trace::parse(&trace_text);
    assert!(records.len() >= 40, "trace 过短");

    let mapbg = {
        let v = load_container("mapbg", 1);
        v.into_iter().next().unwrap()
    };
    let mut machine = BootMachine::new(
        decode_png("l0.png"),
        decode_png("l1.png"),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg,
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    let mut checked = 0usize;
    // 按键投递时序：TICK n 的 INPUT 行在 preTick(n)（paint#n 之后）投递，
    // 由 logic#(n+1) 消费（run 循环体开头读 keyValue）⇒ 延迟一拍传入 tick()
    let mut pending_key = 0i32;
    // **持久画布**（对抗 review R-1）：Java Canvas 的 screen 跨帧保留——
    // 非全屏覆盖分支（如 mode 14 首拍无引子图）依赖上一帧残影
    let mut screen = ArgbImage::create(240, 320);
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        machine.tick(key);
        if machine.finished {
            // menu-sweep 终点：T1760 press(-5) → kinds[5]=5「帮助」（BUG-007 实为
            // 退出）→ mode 22 → runLogoAnimation(1,·) 70 拍 → T1831 后进程退出
            assert_eq!(rec.tick, 1832, "menu-sweep 终点 = mode 22 动画 70 拍耗尽（T1762+70）");
            assert_eq!(machine.mode, 22, "终点模式");
            break;
        }
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
    // 覆盖锚：l0 T1-16 + l1 T17-31 + logo T32-67 + 声音询问 T68-70 + title T71-150
    assert_eq!(checked, 1831, "mode 0/21/1/8/16/15/17/22 全程 T1-T1831 帧必须全部比对");
}

/// enter-game 场景：0→21→1→14（引子）→2（加载）→ mode 3 边界。
/// 覆盖引子滚动机/调色循环/快进、加载链 42 tick、mode 2→3 切换拍。
#[test]
fn enter_game_frames_match_tick_by_tick() {
    let trace_text = std::fs::read_to_string(
        repo().join("reference/oracle/_out/A-enter-game/trace.txt"),
    )
    .expect("缺 A-enter-game trace（先跑 python3 reference/oracle/run.py --scenarios）");
    let records = trace::parse(&trace_text);
    let mapbg = {
        let v = load_container("mapbg", 1);
        v.into_iter().next().unwrap()
    };
    let mut machine = BootMachine::new(
        decode_png("l0.png"),
        decode_png("l1.png"),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg,
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    let mut checked = 0usize;
    let mut pending_key = 0i32;
    // **持久画布**（同 menu-sweep：Java Canvas 跨帧保留）
    let mut screen = ArgbImage::create(240, 320);
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        machine.tick(key);
        if machine.finished {
            // mode 2 完成 → m_000 case 3（游戏态初始化）——mode 3 游戏画面下批
            assert!(rec.tick >= 380 && rec.tick <= 430, "mode 3 边界应在 T382-430 区（加载 42 tick），实际 T{}", rec.tick);
            break;
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, 240, 320);
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
    assert!(checked >= 380, "引子+加载链帧数不足: {checked}");
}

/// INPUT 行 → 本 tick 边界投递的按键码（keyPressed；release 只清 keyHeldCode，
/// 对 keyValue 语义无影响——a.java:4650/4654）。同 tick 多键取首个 press。
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
