// 发行版隐藏控制台框（2026-10-18 用户指正"双击启动弹 cmd 框"——cargo 默认
// console 子系统）；debug 构建保留控制台便于开发期日志。无控制台 ⇒ GAME-CRASH
// 行同步写 exe 目录 xiyou-crash.log（见 paint_crashed 消费点），诊断不断档。
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
//! game-desktop —— 桌面壳（窗口/输入/存档落盘/呈现/BGM）。让 game-core 真正可玩。
//!
//! 分层（AGENTS §2）：本 crate 只做平台实现；游戏逻辑全在 game-core，
//! 逐 tick 调用面与 oracle 差分 harness 完全同构（machine.tick/paint →
//! scene.press_key/release_key/tick/paint），因此差分语义在壳下原样成立。
//!
//! 用法：
//!   cargo run -p game-desktop                       # 资产取 ./assets/raw，BGM 取 ./assets/audio
//!   cargo run -p game-desktop -- --assets 路径      # 自定资产目录
//!   选项：--save 目录 / --audio 目录 / --audio-device ALSA设备名（默认 default）/ --mute
//! 键位：方向键/WASD = 行走与菜单；回车/空格 = 确认(-5)；
//!       Q/F1 = 左软键(-6)；E/F2 = 右软键(-7)；F11 = 全屏切换。
//!       鼠标左键：画面底部 30% 左三分之一 = -6、右三分之一 = -7、其余 = -5。
//! 输入语义 = Java Canvas 回调（docs/spec/p3-render.md R-2）：按下/抬起只改状态、
//!       由下一逻辑拍消费；同拍多键覆盖式取末个；OS 键盘自动重复丢弃（a.class 未覆写
//!       keyRepeated）——长按行走由 keyHeldCode 驱动（a.java:5308-5329）。
//! 窗口：可缩放，整数倍缩放 + 信箱居中（失焦暂停已按用户决策移除，2026-10-15）。
//! 可见性 = MIDP Canvas 生命周期（lifecycle.rs）：完全遮挡/最小化 ⇒ hideNotify（a.java:4502-4513：
//!       关 BGM + anyKeyPlayed=true；隐藏期键/鼠标不投递）；恢复可见 ⇒ showNotify；其后首个游戏键
//!       = keyPressed 续播边沿（a.java:4665-4674，game-core 内执行）。失焦不映射（失焦 ≠ 隐藏）。
//! BGM：game-core `bgm_request()` 驱动（docs/spec/audio.md）；产物由
//!       `cargo run --release --manifest-path tools/audio/Cargo.toml` 生成，缺失则无声运行。
//! 存档：./save/ 目录（MOT_L0-2 = 槽存档、MOT_IF = 槽表、SKY_WAR = boss 旗标），
//!       与 BootMachine 的 RMS 预置同格式——游戏内「保存游戏」即落盘。

use game_core::enums::GameMode;
mod audio;
mod lifecycle;
mod headless_shot;
mod view;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use game_core::layout::{SCREEN_H, SCREEN_W};
use game_core::paint::BootMachine;
use game_core::render::{ArgbImage, SoftGraphics};
use game_core::scene::{FloorAssets, GameScene};

/// mode 21 起的帧间隔 f_int_02（a.java:27 初值 75 / a.java:3377 复位 75）；GameScene
/// 阶段恒为此值（f_int_02 仅 a.java:3350/3377 两处写，均在 mode 0 内）。
const TICK_MS: u64 = 75;
/// 初始窗口倍数（逻辑像素）；之后随窗口尺寸自适应整数倍。
const INITIAL_SCALE: u32 = 2;
/// 逻辑画布尺寸（u32 视图，= game_core::layout::SCREEN_W/SCREEN_H，Stage A 240×320）——
/// 壳内呈现几何的唯一来源：窗口初始/最小尺寸、Viewport 适配、鼠标反映射、blit。
/// 【STAGE-B-002】随 `Shell.layout` 动态（classic = 240×320 原值零变化；
/// 原 CANVAS_W/H 常量已收编为 layout 派生）。

/// preset 解析（STAGE-B-002 ①）：`classic`（默认，原版恒等）/ `modern`（逻辑分辨率=
/// 任意格数，`grid` 缺省 13×13 = 416×416；最小钳位 240×320 由 Layout::modern 保证）。
fn parse_preset(name: &str, grid: Option<(i32, i32)>) -> game_core::layout::Layout {
    match name {
        "modern" => {
            let (w, h) = grid.unwrap_or((13, 13));
            game_core::layout::Layout::modern(w, h)
        }
        _ => game_core::layout::Layout::classic(),
    }
}
const WINDOW_TITLE: &str = "囧囧西游-大闹天宫";

// ---------------------------------------------------------------- 资产装载

macro_rules! fixture {
    ($t:literal) => {
        include_str!(concat!("../../../crates/game-core/tests/fixtures/", $t))
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect::<Vec<i32>>()
    };
}

/// 目录型资产源（接缝④）已抽 `game-shell-common::asset::DirAssetSource`（task-5
/// 双壳总审裁定 2026-10-18：DirAssetSource/DirAssets 生产面小重复 → 并入共享单源，
/// 防双壳拷贝漂移）。本地别名保既有调用面与测试（`DirAssets::new(&…)`）零改动
/// （adversary 裁定 A：机械接线）。
use game_shell_common::asset::DirAssetSource as DirAssets;

fn load_container(src: &dyn game_platform::AssetSource, name: &str, count: usize) -> Vec<ArgbImage> {
    let data = src
        .read(name)
        .unwrap_or_else(|e| panic!("缺资产 {name}（先跑 tools/extract-assets/extract.py）：{e}"));
    game_data::PackedPng::parse(&data, count)
        .unwrap_or_else(|e| panic!("容器 {name} 解析失败：{e}"))
        .images
        .iter()
        .map(|sub| ArgbImage::from_decoded_png(game_data::decode_png(sub.bytes).unwrap()))
        .collect()
}

struct Assets {
    tileset: ArgbImage,
    images: game_core::scene::SceneImages,
    floors: Vec<Option<FloorAssets>>,
    width_table: Vec<i32>,
    tile_walk: Vec<i32>,
}

fn load_assets(src: &dyn game_platform::AssetSource) -> Assets {
    let mapbg = load_container(src, "mapbg", 1);
    let map = load_container(src, "map", 12);
    let actor_c = load_container(src, "actor", 4);
    let sptmap = load_container(src, "sptmap", 13);
    let sptprop = load_container(src, "sptprop", 23);
    let sptarm = load_container(src, "sptarm", 10);
    let sptenemy1 = load_container(src, "sptenemy1", 20);
    let sptenemy2 = load_container(src, "sptenemy2", 20);
    let ui = load_container(src, "ui", 25);
    let images = game_core::scene::SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: game_core::scene::SceneImages::map_entity_images(
            &sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2,
        ),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container(src, "menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui: ui.clone(),
        faces: load_container(src, "face", 12),
        ending: load_container(src, "end", 1)[0].clone(),
        xtq: load_container(src, "xtq", 6),
    };
    // 55 层全量装载（楼梯换层即时可用；maplv+sprite 每层 <2KB）
    let door_tables = game_core::sprite_spawn::DoorTables {
        door12: include_str!("../../../crates/game-core/tests/fixtures/door12.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
        door13: include_str!("../../../crates/game-core/tests/fixtures/door13.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
    };
    let mut floors: Vec<Option<FloorAssets>> = vec![None; 56];
    for (n, slot) in floors.iter_mut().enumerate().skip(1).take(54) {
        let maplv = src
            .read(&format!("maplv{n}"))
            .unwrap_or_else(|e| panic!("缺资产 maplv{n}：{e}"));
        let sp = game_data::SpriteTable::parse(
            &src.read(&format!("sprite{n}"))
                .unwrap_or_else(|e| panic!("缺资产 sprite{n}：{e}")),
        )
        .unwrap_or_else(|e| panic!("sprite{n} 解析失败：{e}"));
        let records: Vec<(u8, u16, u16, i16, bool)> = sp
            .records
            .iter()
            .map(|rec| {
                let (param, hide) = game_core::sprite_spawn::spawn_param_and_visibility(
                    rec.type_code, &rec.extra, &door_tables,
                );
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect();
        *slot = Some(FloorAssets { maplv, records });
    }
    Assets {
        tileset: map[0].clone(),
        images,
        floors,
        width_table: fixture!("type_width_px.txt"),
        tile_walk: fixture!("tile_walkability.txt"),
    }
}

// ---------------------------------------------------------------- RMS 落盘（共享实现）

/// 文件版 RecordStore 语义核心已抽 `game-shell-common::rms`（lead F-4 裁决 2026-10-17，
/// 防双壳拷贝漂移）：`FileRms` 实现 `game_platform::Storage`（接缝①；键域/键名走契约
/// `RMS_STORES`/`rms_key`/`rms_store_name`）。本地别名保既有调用面与测试
/// （`crate::Rms`/`crate::serialize_slot_if`）零改动（adversary 裁定 A：机械接线）。
use game_shell_common::rms::{serialize_slot_if, FileRms as Rms};

// ---------------------------------------------------------------- 键位

fn key_code(physical: winit::keyboard::PhysicalKey) -> Option<i32> {
    use winit::keyboard::PhysicalKey::Code;
    use winit::keyboard::KeyCode as K;
    Some(match physical {
        Code(K::ArrowUp) | Code(K::KeyW) => -1,
        Code(K::ArrowDown) | Code(K::KeyS) => -2,
        Code(K::ArrowLeft) | Code(K::KeyA) => -3,
        Code(K::ArrowRight) | Code(K::KeyD) => -4,
        Code(K::Enter) | Code(K::Space) => -5,
        Code(K::KeyQ) | Code(K::F1) => -6,
        Code(K::KeyE) | Code(K::F2) => -7,
        // WIN-OP①（2026-10-18）：ESC = 右软键 -7 语义（mode3 开菜单；菜单/面板/对话
        // 态 = 关闭/返回/翻页——键码契约 §4 全态已核 C-b 无副作用）
        Code(K::Escape) => -7,
        _ => return None,
    })
}

/// WIN-OP②（2026-10-18，C-a 态门控）：滚轮 → 行导航键。Map（行走态）禁止合成
/// （-1/-2 的 keyValue 臂空转但 press 置 key_held ⇒ 落拍误走一步）；其余态合成：
/// 菜单/面板 = 光标上/下移，对话态 = 翻页/滚动（handleOverlayKey 语义，自然）。
fn wheel_key(dy: f64, mode: game_core::enums::GameMode) -> Option<i32> {
    use game_core::enums::GameMode as M;
    if mode == M::Map || dy == 0.0 {
        return None;
    }
    if dy > 0.0 {
        Some(-1)
    } else {
        Some(-2)
    }
}

/// WIN-OP③（2026-10-18）：鼠标点击 → 游戏键。软键区优先（全态，HUD 语义）；
/// 右键 = -7 返回/菜单（全态）；非软键区左键 = 确认（-5），但 Map 行走态禁
/// （防误触玩法键）——行点击（直接选行）另经 game-core handle_pointer（Stage B）。
fn click_key(x: i32, y: i32, w: i32, h: i32, mode: game_core::enums::GameMode, right: bool) -> Option<i32> {
    use game_core::enums::GameMode as M;
    if right {
        return Some(-7);
    }
    if let Some(code) = softkey_code(x, y, w, h) {
        return Some(code); // 软键区（-6/-7）——判区单源
    }
    if mode == M::Map {
        return None;
    }
    Some(-5)
}

// ---------------------------------------------------------------- 真字形注入（D2 字体缝）

mod font_inject {
    use std::sync::OnceLock;
    use ab_glyph::{Font, FontVec, PxScale, point};

    static FONT: OnceLock<Option<FontVec>> = OnceLock::new();

    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        "/usr/share/fonts/wqy-microhei/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    ];

    /// Windows 系统字体（2026-10-14 Windows 适配）：%WINDIR%\Fonts 下的 CJK 常驻字体。
    /// 路径不写死 C:——系统盘符可变，走 WINDIR 环境变量（缺省回退 C:\Windows）。
    fn windows_candidates() -> Vec<String> {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
        ["msyh.ttc", "msyhbd.ttc", "simhei.ttf", "simsun.ttc", "Deng.ttf"]
            .iter()
            .map(|f| format!("{windir}\\Fonts\\{f}"))
            .collect()
    }

    fn load() -> Option<FontVec> {
        let extra = windows_candidates();
        for path in CANDIDATES.iter().map(|s| s.to_string()).chain(extra) {
            if let Ok(bytes) = std::fs::read(&path) {
                match FontVec::try_from_vec_and_index(bytes, 0) {
                    Ok(f) => {
                        eprintln!("字形源：{path}");
                        return Some(f);
                    }
                    Err(e) => eprintln!("字体 {path} 不可用（{e}），试下一个"),
                }
            }
        }
        eprintln!("未找到 CJK 字体——文字保持差分占位字形");
        None
    }

    /// game_core::render::set_glyph_hook 的注入实现：系统 TTF 真 CJK 字形。
    /// 布局仍用 SoftFont 的格宽/格高（与差分验证过的排版一致），仅字形换真。
    pub fn install() {
        let font = load();
        let has_font = font.is_some();
        let _ = FONT.set(font);
        if has_font {
            game_core::render::set_glyph_hook(Some(hook_fn));
        }
        // 无字体 = **不装 hook**：game-core 差分占位字形原样保留（真回退；
        // 此前无条件装 hook 且 hook 内直接 return = 文字空白的假回退）
    }

    fn hook_fn(
        ch: u16,
        px: i32,
        py: i32,
        cw: i32,
        chh: i32,
        color: u32,
        put: &mut dyn FnMut(i32, i32, u32),
    ) {
        let Some(font) = FONT.get().and_then(|f| f.as_ref()) else { return };
        let scale = PxScale { x: chh as f32, y: chh as f32 };
        // 基线放格底上方 1px（CJK 无下伸部，格内视觉居中）
        let glyph = font
            .glyph_id(char::from_u32(ch as u32).unwrap_or(' '))
            .with_scale_and_position(scale, point(px as f32, (py + chh - 1) as f32));
        if let Some(outline) = font.outline_glyph(glyph) {
            // ab_glyph 回调坐标是 px_bounds **局部**坐标（outlined.rs draw() 文档：
            // "pixel coordinate inside the bounds"），必须平移 bbox 最小角才是画布
            // 绝对坐标——否则全部字形像素堆在 (0,0)，文本落点为空（mode21 文字不可见 bug）。
            let bb = outline.px_bounds();
            let (ox, oy) = (bb.min.x.round() as i32, bb.min.y.round() as i32);
            outline.draw(|gx, gy, a| {
                if a > 0.25 {
                    put(ox + gx as i32, oy + gy as i32, color);
                }
            });
        }
        let _ = cw;
    }

    #[cfg(test)]
    /// BUG-018 回归（2026-10-17 用户实机："保存存档后，加载存档，无法看到已保存的存档"）：
/// ①Rms 六槽全键域落盘/回读（原只载 MOT_L0-2 ⇒ 槽 3-5 保存丢失）；
/// ②MOT_IF 槽表序列化对偶（serialize_slot_if ↔ load_slot_table）。
/// 壳层接线（保存块 upsert + sc.preset_mot_if 回写 + BootMachine stores 同步）
/// 由 Shell::step 存档块承载，本测锁 Rms 层键域与格式对偶。
#[cfg(test)]
mod rms_save_roundtrip {
    use crate::{serialize_slot_if, Rms};

    #[test]
    fn rms_roundtrips_all_six_slots_and_mot_if() {
        let dir = std::env::temp_dir().join(format!("dsh-rms-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (rms, loaded0) = Rms::load(&dir);
        assert!(loaded0.is_empty(), "空目录无 store");
        let mut stores: Vec<(String, Vec<u8>)> = Vec::new();
        for idx in 0..6 {
            stores.push((format!("MOT_L{idx}@1"), vec![0xA0 + idx as u8; 8]));
        }
        let mut slot = game_core::menu_family::SlotSelect::new();
        for k in 0..6 {
            slot.valid[k] = true;
            slot.floor_num[k] = (k + 1) as i8;
        }
        stores.push(("MOT_IF@1".into(), serialize_slot_if(&slot)));
        stores.push(("SKY_WAR@1".into(), vec![7, 7, 7]));
        rms.persist(&stores);
        let (_, loaded) = Rms::load(&dir);
        for idx in 0..6 {
            let key = format!("MOT_L{idx}@1");
            assert_eq!(
                loaded.iter().find(|(k, _)| *k == key).map(|(_, v)| v.len()),
                Some(8),
                "{key} 落盘回读"
            );
        }
        let mot_if = loaded.iter().find(|(k, _)| k.starts_with("MOT_IF@")).unwrap();
        let mut sl2 = game_core::menu_family::SlotSelect::new();
        sl2.load_slot_table(Some(&mot_if.1));
        assert!(sl2.valid.iter().all(|&v| v), "槽表六槽回读有效");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod tests {
        use super::*;
        use game_core::layout::{prompt_cx, prompt_cy, SCREEN_H, SCREEN_W, SOUND_PROMPT_TEXT};
        use game_core::render::{anchor, ArgbImage};

        /// 真链路端到端：注入 hook 后，经 SoftGraphics::draw_string 画 mode 21
        /// 提示文本，统计真字形像素。这是 shell 里 mode 21 文字是否可见的最小复现。
        #[test]
        fn hook_renders_mode21_text_via_softgraphics() {
            install(); // 注入真字形（与 shell 启动同一路径）
            let mut img = ArgbImage::create(SCREEN_W, SCREEN_H);
            // mode 21 实际锚点（paint_sound_prompt 同式：画面中心）
            let (cx, cy) = (prompt_cx(), prompt_cy());
            {
                let mut g = crate::canvas_graphics(&mut img);
                g.set_color(0x000000);
                g.fill_rect(0, 0, SCREEN_W, SCREEN_H);
                g.set_color(game_core::layout::WHITE);
                g.draw_string(SOUND_PROMPT_TEXT, cx, cy, anchor::TOP | anchor::HCENTER);
            }
            // 区域断言（防假绿）：文本带必须有像素、原点邻域必须干净——
            // ab_glyph 局部坐标 bug 的特征签名 = 字形堆在 (0,0) 且文本带为空。
            // 文本带 = 锚点 x−80..x+90 × y−10..y+25（240×320 下即 40..210 × 150..185）。
            let band_x = (cx - 80) as usize..(cx + 90) as usize;
            let band_y = (cy - 10) as usize..(cy + 25) as usize;
            let (mut band, mut origin) = (0usize, 0usize);
            let w = SCREEN_W as usize;
            for y in 0..SCREEN_H as usize {
                for x in 0..w {
                    if img.argb[y * w + x] & 0x00FF_FFFF == 0 {
                        continue;
                    }
                    if band_x.contains(&x) && band_y.contains(&y) {
                        band += 1;
                    }
                    if x < 32 && y < 32 {
                        origin += 1;
                    }
                }
            }
            eprintln!("文本带像素 = {band}，原点 32×32 像素 = {origin}");
            assert!(band > 30, "mode21 文本带应画出 >30 像素，实得 {band}");
            assert_eq!(origin, 0, "原点邻域应无字形像素，实得 {origin}——非零即坐标未平移");
        }
    }
}

// ---------------------------------------------------------------- 应用状态

/// 逻辑画布上的绘制上下文：全屏裁剪 + 游戏字体（GameScene / BootMachine 绘制共用前置）。
fn canvas_graphics(screen: &mut ArgbImage) -> SoftGraphics<'_> {
    // 画布裁剪随画布尺寸（STAGE-B-002 返修三轮：常量 240×320 会把最先绘制的视差
    // 裁在经典区内 ⇒ 区外留黑矩形——黑区根因）；classic 画布 240×320 原值零变化。
    let (w, h) = (screen.width, screen.height);
    let mut g = SoftGraphics::new(screen);
    g.set_clip(0, 0, w, h);
    g.set_font(Some(game_core::paint::paint_font()));
    g
}

enum Screen {
    Boot(BootMachine),
    Game(Box<GameScene>),
    /// 崩溃终止屏（P0-2 壳层统一：game-core panic 被壳捕获后转入——
    /// 全宿主一致呈现"崩溃终止"，消除桌面闪退/wasm 假死的观感分裂）。
    Crashed,
}

struct Shell {
    assets: std::sync::Arc<Assets>,
    /// 接缝①：存档持久经 `game_platform::Storage` trait（实现 = game-shell-common::FileRms）
    rms: Box<dyn game_platform::Storage>,
    stores: Vec<(String, Vec<u8>)>,
    screen: ArgbImage,
    /// 布局面（STAGE-B-002）：画布/视口/鼠标反映射的尺寸源（classic=240×320 零变化）。
    layout: game_core::layout::Layout,
    state: Option<Screen>,
    /// machine 一次性键（keyValue：按下覆盖式写入，下一逻辑 tick 消费）
    pending_machine_key: i32,
    frame_counter: u64,
    /// 已落盘的槽存档指纹（scene 侧保存检测）
    persisted_blobs: [Option<Vec<u8>>; 6],
    /// BGM：每拍轮询 game-core 请求（audio.rs）
    bgm: audio::BgmController,
    /// Canvas 可见性（f_bool_01 latch + 遮挡/最小化两路观测，lifecycle.rs）
    visibility: lifecycle::Visibility,
    /// 挂起的 BootMachine 键，其 keyPressed 音频半已由 hide_notify 按 Java 次序补做
    /// ⇒ 下一拍 tick 入口的延迟调用须屏蔽（见 `hide_notify` / `step`）。
    boot_key_audio_done: bool,
    /// 上次同步时的 anyKeyPlayed（只用于记录续播边沿日志）
    resume_armed: bool,
    /// 跨机换机储备（kind7 退出→回标题，2026-10-18）：enter_scene 收储 BootMachine，
    /// 换回时还原——Java CMidlet 同对象保标题态（cursor/粒子/相位）的保真对偶。
    stashed_boot: Option<game_core::paint::BootMachine>,
    /// STAGE-B-001：全局应用设置（声音对）——save 目录旁 app_settings.cfg；
    /// settings_path=None = 平台未给路径（测试/无持久场景）不落盘。
    settings_path: Option<std::path::PathBuf>,
    settings_saved: Option<game_shell_common::settings::AppSettings>,
}

impl Shell {
    /// BootMachine 装配（与测试 harness 同参）。
    fn new(
        src: &dyn game_platform::AssetSource,
        assets: std::sync::Arc<Assets>,
        rms: Box<dyn game_platform::Storage>,
        stores: Vec<(String, Vec<u8>)>,
        sink: Box<dyn audio::AudioSink>,
        layout: game_core::layout::Layout,
    ) -> Shell {
        let mapbg = load_container(src, "mapbg", 1);
        let png = |name: &str| {
            ArgbImage::from_decoded_png(
                game_data::decode_png(
                    &src.read(name)
                        .unwrap_or_else(|e| panic!("缺 assets/raw/{name}（先跑 tools/extract-assets/extract.py）：{e}")),
                )
                .unwrap(),
            )
        };
        let mut machine = BootMachine::new(
            png("l0.png"),
            png("l1.png"),
            load_container(src, "sflogo", 8),
            load_container(src, "ui", 25),
            load_container(src, "menu", 2),
            mapbg[0].clone(),
            load_container(src, "intro", 2),
            load_container(src, "end", 1),
            load_container(src, "load", 2),
        );
        machine.preset_stores = stores.clone();
        // SKY_WAR 装载（saveGlobalProgress 对偶，a.java:4458-4478）：损坏/缺失 ⇒
        // catch 语义（default_7 ⇒ [1]=true）+ 置脏回写（4477-4478）。
        match stores.iter().find(|(k, _)| k.starts_with("SKY_WAR@")) {
            Some((_, blob)) => match game_core::save::parse_global_save(blob) {
                Ok(g) => machine.apply_global(g),
                Err(_) => {
                    machine.apply_global(game_core::save::GlobalSave::default_7());
                    machine.global_dirty = true;
                }
            },
            None => {
                machine.apply_global(game_core::save::GlobalSave::default_7());
                machine.global_dirty = true;
            }
        }
        machine.sptarm = load_container(src, "sptarm", 10);
        // 【STAGE-B-004a】BootMachine 布局换轨（与 GameScene::apply_layout 同口径）
        // ——不换轨则 modern 画布下标题/序章仍走 classic 分支（同 sha 实证）。
        machine.apply_layout(layout.clone());
        Shell {
            assets,
            rms,
            stores,
            screen: ArgbImage::create(layout.screen_w, layout.screen_h),
            layout,
            state: Some(Screen::Boot(machine)),
            pending_machine_key: 0,
            frame_counter: 0,
            persisted_blobs: [None, None, None, None, None, None],
            bgm: audio::BgmController::new(sink),
            visibility: lifecycle::Visibility::new(),
            boot_key_audio_done: false,
            resume_armed: false,
            stashed_boot: None,
            settings_path: None,
            settings_saved: None,
        }
    }

    /// 敌人数值表装配（读档 / 新游戏两路交接共用）：难度 1 缩放 + 基础 HP / 金币表。
    fn install_combat_tables(sc: &mut GameScene) {
        let tables = game_core::combat::CombatTables {
            enemy_base_hp: fixture!("enemybasehp.txt"),
            enemy_base_atk: fixture!("enemybaseatk.txt"),
            enemy_base_def: fixture!("enemybasedef.txt"),
            difficulty_multipliers: fixture!("difficulty_multipliers.txt"),
        };
        sc.enemies = game_core::combat::scale_enemy_stats(&tables, 1);
        sc.enemy_base_hp = tables.enemy_base_hp;
        sc.enemy_base_gold = fixture!("enemybasegold.txt");
    }

    /// keyPressed / keyReleased 投递——Java Canvas 回调语义（a.java:4663-4680）：只改
    /// 状态、不推进逻辑，由下一逻辑拍消费；同拍多次按下覆盖式取末个（keyValue =
    /// keyHeldCode = code）；抬起只清 keyHeldCode（不论哪个键）。docs/spec/p3-render.md R-2。
    ///
    /// keyPressed 的音频半（anyKeyPlayed 续播边沿，a.java:4665-4674）**不在壳侧调用**——
    /// game-core 已接：`GameScene::press_key` 按下即调、`BootMachine::tick` 入口对挂起键调
    /// （`bgm.on_key_pressed`）；壳再调即重复计数。
    fn route_key(&mut self, code: i32, pressed: bool) {
        // MIDP Canvas「Event Delivery」：key/pointer 回调只在 showNotify 之后、hideNotify
        // 之前投递——隐藏期的键不得消费续播边沿（否则看不见窗口时音乐被键唤醒）
        if !self.visibility.shown() {
            return;
        }
        match (self.state.as_mut(), pressed) {
            (Some(Screen::Game(sc)), true) => sc.press_key(code),
            (Some(Screen::Game(sc)), false) => sc.release_key(),
            // BootMachine 各模式只读 keyValue（无 keyHeldCode 消费者）⇒ 抬起无效果
            (Some(Screen::Boot(_)), true) => {
                self.pending_machine_key = code;
                self.boot_key_audio_done = false; // 新挂起键：音频半归 tick 入口
            }
            _ => {}
        }
        // press_key 内的续播是同步 playBgm（Java 在 keyPressed 回调当时 createPlayer）
        self.sync_bgm();
    }

    /// `WindowEvent::Occluded` 的新值 → 可能的 show/hide 边沿。
    fn set_occluded(&mut self, occluded: bool) {
        let edge = self.visibility.set_occluded(occluded);
        self.dispatch_notify(edge, "遮挡");
    }

    /// `Window::is_minimized()` 轮询值 → 可能的 show/hide 边沿。
    fn set_minimized(&mut self, minimized: bool) {
        let edge = self.visibility.set_minimized(minimized);
        self.dispatch_notify(edge, "最小化");
    }

    fn dispatch_notify(&mut self, edge: Option<lifecycle::Notify>, cause: &str) {
        match edge {
            Some(lifecycle::Notify::Hide) => self.hide_notify(cause),
            Some(lifecycle::Notify::Show) => self.show_notify(cause),
            None => {}
        }
    }

    /// hideNotify（a.java:4502-4513 / hideNotify@0-43）。外层 `if (f_bool_01)` 与末尾
    /// `f_bool_01 = false`（4503/4511）由 `visibility` 的 latch 承担：本函数只在
    /// shown→hidden 边沿调用一次。宿主只决定**何时**调用；状态转移全在 game-core。
    fn hide_notify(&mut self, cause: &str) {
        let before = self.bgm_snapshot().track;
        let mode = self.game_mode();
        // BootMachine 挂起键的 keyPressed 音频半在 tick 入口执行（延迟），Java 在按键回调
        // 当时执行。键在先、hide 在后（同一拍间隙）时，Java 次序 = 音频半 → hide；照延迟
        // 执行会颠倒为 hide → 音频半，把 hide 刚置的 anyKeyPlayed 当场消费掉（误续播）。
        // 故先补做该键的音频半，并让 step 屏蔽 tick 入口的那次调用。
        if let Some(Screen::Boot(m)) = self.state.as_mut() {
            if self.pending_machine_key != 0 && !self.boot_key_audio_done {
                m.bgm.on_key_pressed(m.mode);
                self.boot_key_audio_done = true;
            }
        }
        self.sync_bgm(); // 补做若续播 = Java 那次 createPlayer（随即被下面 close）
        match self.state.as_mut() {
            Some(Screen::Game(sc)) => {
                // 4504-4510 全臂已接（2026-10-15）：mode3→4 弹菜单 + closeAudioPlayer +
                // anyKeyPlayed（GameScene::hide_notify；游戏态在 game-core，壳只决定何时调用）
                sc.hide_notify();
            }
            Some(Screen::Boot(m)) => {
                // BootMachine 无 gameMode 3（读档/新游戏完成即交接 GameScene）⇒ 4504 恒假
                m.bgm.on_hide_notify();
            }
            Some(Screen::Crashed) | None => {}
        };
        // closeAudioPlayer 同步关闭（a.java:10208-10215）⇒ 立即下发 Stop，不等下一拍
        self.sync_bgm();
        eprintln!(
            "[lifecycle] hideNotify（{cause}）gameMode={mode}：mode3→4 弹菜单（若 3）+ closeAudioPlayer（BGM {before:?}→None）+ anyKeyPlayed=true",
        );
    }

    /// showNotify（a.java:4495-4500）：只置 f_bool_01（latch 承担），无游戏态副作用——
    /// BGM 不在此恢复，等其后首个键（keyPressed 续播边沿）。
    fn show_notify(&mut self, cause: &str) {
        eprintln!(
            "[lifecycle] showNotify（{cause}解除）gameMode={}：anyKeyPlayed={}（任意键续播）",
            self.game_mode(),
            self.any_key_played()
        );
    }

    /// 当前 gameMode（日志用）。
    fn game_mode(&self) -> GameMode {
        match &self.state {
            Some(Screen::Boot(m)) => m.mode,
            Some(Screen::Game(sc)) => sc.game_mode,
            // -1=无屏哨兵（原 i32 口径保真，非真实模式）；崩溃终止屏同哨兵
            Some(Screen::Crashed) | None => GameMode::Unknown(-1),
        }
    }

    /// anyKeyPlayed（a.java:40）：hideNotify 置位、keyPressed 清零。
    fn any_key_played(&self) -> bool {
        match &self.state {
            Some(Screen::Boot(m)) => m.bgm.any_key_played,
            Some(Screen::Game(sc)) => sc.bgm.any_key_played,
            Some(Screen::Crashed) | None => false,
        }
    }

    /// 下一拍的帧间隔 f_int_02：mode 0 每拍置 100（a.java:3350），切 mode 21 复位 75
    /// （a.java:3377）；Java run 循环每帧起点读取（a.java:3250）。
    fn frame_interval(&self) -> Duration {
        match &self.state {
            Some(Screen::Boot(m)) => Duration::from_millis(m.frame_interval_ms.max(1) as u64),
            _ => Duration::from_millis(TICK_MS),
        }
    }

    fn bgm_snapshot(&self) -> audio::BgmSnapshot {
        match &self.state {
            Some(Screen::Boot(m)) => audio::BgmSnapshot {
                track: m.bgm_request(),
                serial: m.bgm_serial(),
                volume: m.bgm_volume().unwrap_or(0),
            },
            Some(Screen::Game(sc)) => audio::BgmSnapshot {
                track: sc.bgm_request(),
                serial: sc.bgm_serial(),
                volume: sc.bgm_volume().unwrap_or(0),
            },
            Some(Screen::Crashed) | None => audio::BgmSnapshot::default(),
        }
    }

    /// `CMidlet.dispatchModeEntry` 退出信号（BootMachine::finished，mode 22 退场动画 70 拍后置位；
    /// game-core 只发信号不碰进程——退出是平台接口，shell 消费执行）。
    /// 【2026-10-17 BUG-020】原缺此消费者 ⇒ 主菜单"退出"退场动画走完后无人退出。
    fn finished(&self) -> bool {
        match self.state.as_ref() {
            Some(Screen::Boot(m)) => m.finished,
            Some(Screen::Game(_)) | Some(Screen::Crashed) | None => false,
        }
    }

    /// 退出链消费点（BUG-020 执行半，接缝③）：粘滞信号**每 tick 轮询**（现状口径，
    /// 无"消费一次"语义——el.exit() 实际只执行一次是"循环先退"的巧合；adversary F4
    /// 裁定 (a)照实声明），经 `ProcessControl::request_exit`（幂等）执行。返回
    /// true = 本次发出了退出请求（测试断言面）。信号消费归 shell：finished 判定
    /// 全在本方法（game-core 只置位）。
    fn consume_exit(&mut self, ctl: &mut dyn game_platform::ProcessControl) -> bool {
        if self.finished() {
            ctl.request_exit();
            true
        } else {
            false
        }
    }

    /// WIN-OP③ hover（2026-10-18）：命中行 → 光标跟随（选中高亮）。只动光标（零副作用）。
    fn pointer_hover(&mut self, x: i32, y: i32) {
        match self.state.as_mut() {
            Some(Screen::Game(sc)) => {
                if let game_core::scene::PointerHit::Row { index, .. } = sc.pointer_hit(x, y) {
                    sc.select_row(index);
                }
            }
            // C-2（2026-10-18）：标题侧设置面板（BootMachine::About）同享指针
            Some(Screen::Boot(m)) => {
                if let game_core::scene::PointerHit::Row { index, .. } = m.pointer_hit(x, y) {
                    m.select_row(index);
                }
            }
            _ => {}
        }
    }

    /// WIN-OP③ 点击：命中行 → 对位 + 返回该态确认键（shell 合成 press+release，
    /// 与 click_code 同款点按语义）；未命中 → click_key 基线（软键区/右键/确认）。
    fn pointer_click(&mut self, x: i32, y: i32, right: bool) -> Option<i32> {
        if !right {
            match self.state.as_mut() {
                Some(Screen::Game(sc)) => {
                    if let game_core::scene::PointerHit::Row { index, confirm_key } =
                        sc.pointer_hit(x, y)
                    {
                        sc.select_row(index);
                        return Some(confirm_key);
                    }
                }
                // C-2：标题侧设置面板同享（confirm=-5 翻转键同款）
                Some(Screen::Boot(m)) => {
                    if let game_core::scene::PointerHit::Row { index, confirm_key } =
                        m.pointer_hit(x, y)
                    {
                        m.select_row(index);
                        return Some(confirm_key);
                    }
                }
                _ => {}
            }
        }
        click_key(x, y, self.layout.screen_w, self.layout.screen_h, self.game_mode(), right)
    }

    /// 一逻辑拍（logic → paint，同 Java run 循环体），随后同步 BGM 请求。可见性不影响出拍：
    /// Java run 线程在隐藏期照常循环（hideNotify 只改状态，不停线程）。
    fn logic_tick(&mut self) {
        self.step();
        // 跨机换机检查（kind7 退出→回标题）：step 后信号已置，本拍内换机
        //（2026-10-18 整改：原挂 about_to_wait——壳级测试与 tick 语义的盲区）
        self.swap_back_to_boot_if_requested();
        self.persist_settings_if_changed(); // STAGE-B-001：声音对变更落盘
        self.sync_bgm();
        self.persist_global_if_dirty();
        self.persist_slot_if_if_dirty();
    }

    /// game-core BGM 请求 → 输出命令（只在变化时发送，audio.rs）。顺带观测 anyKeyPlayed
    /// 落沿 = keyPressed 续播边沿（音频半在 game-core 内执行），记一行日志。
    fn sync_bgm(&mut self) {
        let snap = self.bgm_snapshot();
        let armed = self.any_key_played();
        if self.resume_armed && !armed {
            eprintln!(
                "[lifecycle] keyPressed 续播边沿：anyKeyPlayed→false（gameMode={}，mode 0/21 不续播），BGM 请求 {:?}",
                self.game_mode(),
                snap.track
            );
        }
        self.resume_armed = armed;
        self.bgm.update(snap);
    }

    /// 画布像素导出（headless 截图用）。
    fn canvas_snapshot(&self) -> (i32, i32, Vec<u32>) {
        (self.screen.width, self.screen.height, self.screen.argb.clone())
    }

    fn paint_scene(&mut self, sc: &mut GameScene) {
        let mut g = canvas_graphics(&mut self.screen);
        sc.paint(&mut g, &self.assets.tileset);
    }

    /// 崩溃终止屏（P0-2 壳层统一）：game-core panic 的全宿主一致呈现。
    /// 纯壳层画面，不进 game-core 渲染语义（保真面零侵入）。
    fn paint_crashed(&mut self) {
        let (w, h) = (self.screen.width, self.screen.height);
        let mut g = canvas_graphics(&mut self.screen);
        g.set_color(0x20_00_00);
        g.fill_rect(0, 0, w, h);
        g.set_color(0xFF_FF_FF);
        g.set_font(Some(game_core::paint::paint_font()));
        let l1: Vec<u16> = "游戏已终止".encode_utf16().collect();
        // 【2026-10-17 BUG-017 口径更正】原"（原版崩溃语义等价终止）"为**未经逐案
        // 核实的声称**——census assert（transition_to 未登记边）等 Rust 自造断言
        // panic 并非原版崩溃（用户实机被误导）。等价性须逐案以 Java 锚核实后另记。
        let l2: Vec<u16> = "（崩溃详情见游戏目录 xiyou-crash.log）".encode_utf16().collect();
        g.draw_string(&l1, SCREEN_W / 2, SCREEN_H / 2 - 12, 17);
        g.draw_string(&l2, SCREEN_W / 2, SCREEN_H / 2 + 8, 17);
    }

    /// 交接拍收尾（读档 / 新游戏两路共用）：本拍即画场景（读档路 Java 本拍 paint 已是
    /// mode 3；两路均与 harness 同构）并切入 GameScene。
    fn enter_scene(&mut self, mut sc: GameScene, machine: game_core::paint::BootMachine) {
        // STAGE-B-002①：preset 写入 game-core 布局面——**经 apply_layout 换轨**
        //（直赋 layout 字段绕过视口/相机/绘制坐标系同步 = 返修二轮破绽根因）。
        sc.apply_layout(self.layout.clone());
        // SKY_WAR 交接拷贝（Boot→GameScene）：不拷则场景水位从 0 起步会把持久水位写回退
        // 【2026-10-18 整改】machine 经参数收储——调用链 step 已 take 走 state，
        // 此处再 take 只得 None（stash 恒空 ⇒ 换机静默无动作，用户实报）。
        let g = machine.global_snapshot();
        sc.apply_global(g);
        self.stashed_boot = Some(machine); // 跨机储备（kind7 退出回标题用）
        self.paint_scene(&mut sc);
        self.state = Some(Screen::Game(Box::new(sc)));
        self.frame_counter += 1;
    }

    /// STAGE-B-001：声音态=全局应用设置（不入游戏存档）。当前声音快照。
    fn sound_state(&self) -> Option<(bool, i32)> {
        match self.state.as_ref() {
            Some(Screen::Boot(m)) => Some((m.bgm.sound_enabled, m.bgm.volume_level)),
            Some(Screen::Game(sc)) => Some((sc.bgm.sound_enabled, sc.bgm.volume_level)),
            Some(Screen::Crashed) | None => None,
        }
    }

    /// STAGE-B-001：变更落盘（写-if-changed，best-effort 吞错——FileRms 同律）。
    fn persist_settings_if_changed(&mut self) {
        let Some((sound, vol)) = self.sound_state() else { return };
        let cur = game_shell_common::settings::AppSettings { sound_enabled: sound, volume_level: vol };
        if self.settings_saved == Some(cur) {
            return;
        }
        if let Some(p) = self.settings_path.clone() {
            cur.save(&p);
        }
        self.settings_saved = Some(cur);
    }

    /// STAGE-B-001：启动装载（shell 构造链调用）——应用保存的声音对。
    fn apply_loaded_settings(&mut self, s: game_shell_common::settings::AppSettings) {
        match self.state.as_mut() {
            Some(Screen::Boot(m)) => {
                m.bgm.sound_enabled = s.sound_enabled;
                m.bgm.volume_level = s.volume_level;
            }
            Some(Screen::Game(sc)) => {
                sc.bgm.sound_enabled = s.sound_enabled;
                sc.bgm.volume_level = s.volume_level;
            }
            Some(Screen::Crashed) | None => {}
        }
        self.settings_saved = Some(s);
    }

    /// 跨机换机检查（kind7 退出→回标题，2026-10-18）：scene 置 MainMenu 态 =
    /// 换机请求（Java gameMode=1 语义），换回 stashed BootMachine（标题态保真）。
    fn swap_back_to_boot_if_requested(&mut self) {
        let need = match self.state.as_ref() {
            Some(Screen::Game(sc)) => sc.game_mode == game_core::enums::GameMode::MainMenu,
            _ => false,
        };
        if need {
            if let Some(mut m) = self.stashed_boot.take() {
                m.mode = game_core::enums::GameMode::MainMenu;
                // dispatchModeEntry case 1 音频半（a.java:4295/4311）：f_bool_28=false + playBgm(2)
                //——2026-10-18 用户实报：换机后音乐未切回标题曲（stash 机器带着
                // enter_game 的 f_bool_28=true 状态，须按 Java case 1 语义复位）
                m.bgm.enter_title();
                self.state = Some(Screen::Boot(m));
            }
        }
    }

    /// SKY_WAR 持久化轮询（saveGlobalProgress 触发收口）：活动屏任一脏 ⇒ 落盘。
    fn persist_global_if_dirty(&mut self) {
        let (g, dirty) = match self.state.as_mut() {
            Some(Screen::Boot(m)) => (m.global_snapshot(), m.take_global_dirty()),
            Some(Screen::Game(sc)) => (sc.global_snapshot(), sc.take_global_dirty()),
            Some(Screen::Crashed) | None => return,
        };
        if !dirty {
            return;
        }
        let blob = game_core::save::serialize_global_save(&g);
        let mut stores = std::mem::take(&mut self.stores);
        stores.retain(|(k, _)| !k.starts_with("SKY_WAR@"));
        stores.push(("SKY_WAR@1".into(), blob));
        self.rms.persist(&stores);
        self.stores = stores;
    }

    /// MOT_IF 读失败回写（markSlotEntry(-1) 对偶，BUG-021 ②c）：槽表 parse_dirty
    /// （game-core `load_slot_table` 失败触发器 = Java catch 9000-9001）⇒ 当前槽表
    /// **整表落盘**（Java delete+addRecord 同终态，a.java:9026-9052）。回写后
    /// stores 同步（Boot 侧读档列表重读源）/ preset_mot_if 回写（Game 侧 mode7/8
    /// 重读源）⇒ 重复进入收敛（Java 写后可解析同收敛）。
    fn persist_slot_if_if_dirty(&mut self) {
        let dirty_slot = match self.state.as_mut() {
            Some(Screen::Game(sc)) => sc.slot.take_parse_dirty().then(|| sc.slot.clone()),
            Some(Screen::Boot(m)) => {
                m.slots.as_mut().and_then(|s| s.take_parse_dirty().then(|| s.clone()))
            }
            Some(Screen::Crashed) | None => None,
        };
        let Some(slot) = dirty_slot else { return };
        let mot_if = serialize_slot_if(&slot);
        let mut stores = std::mem::take(&mut self.stores);
        stores.retain(|(k, _)| !k.starts_with("MOT_IF@"));
        stores.push(("MOT_IF@1".into(), mot_if.clone()));
        self.rms.persist(&stores);
        self.stores = stores;
        if let Some(Screen::Game(sc)) = self.state.as_mut() {
            sc.preset_mot_if = Some(mot_if); // 重读源同步（BUG-018 环）
        }
    }

    fn step(&mut self) {
        match self.state.take() {
            Some(Screen::Game(mut sc)) => {
                // 存档落盘（游戏内「保存游戏」→ saved_slot_blobs/slot 变化即写穿）
                if sc.saved_slot_blobs != self.persisted_blobs {
                    let mut stores = std::mem::take(&mut self.stores);
                    // 【2026-10-17 BUG-018】原只改写已存在键——新键（空存档目录/槽3-5）
                    // 会被整体丢弃；改 upsert。保存后回写 sc.preset_mot_if（mode7/8
                    // 每次进入的重读源）——原缺此环 ⇒ 会话内加载列表看不到新存档。
                    for idx in 0..6 {
                        if let Some(blob) = &sc.saved_slot_blobs[idx] {
                            let key = format!("MOT_L{idx}@1");
                            match stores.iter_mut().find(|(k, _)| *k == key) {
                                Some((_, v)) => *v = blob.clone(),
                                None => stores.push((key, blob.clone())),
                            }
                        }
                    }
                    let mot_if = serialize_slot_if(&sc.slot);
                    stores.retain(|(k, _)| !k.starts_with("MOT_IF@"));
                    stores.push(("MOT_IF@1".into(), mot_if.clone()));
                    self.rms.persist(&stores);
                    self.stores = stores;
                    self.persisted_blobs = sc.saved_slot_blobs.clone();
                    sc.preset_mot_if = Some(mot_if);
                }
                // P0-2 壳层统一（bug-012-softlock-report.md §3）：game-core 崩溃
                // （如 T350 槽登记越界 = Java AIOOBE 同点同因）在此捕获 → 崩溃终止屏。
                // game-core 语义不动（保真零侵入）；终止呈现归宿主层。
                let fc = self.frame_counter as i64;
                let wt = self.assets.width_table.clone();
                let tick = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    sc.tick(fc, &wt);
                }));
                match tick {
                    Ok(()) => {
                        self.frame_counter += 1;
                        self.paint_scene(&mut sc);
                        self.state = Some(Screen::Game(sc));
                    }
                    Err(e) => {
                        let msg = e
                            .downcast_ref::<String>()
                            .map(String::as_str)
                            .or_else(|| e.downcast_ref::<&str>().copied())
                            .unwrap_or("");
                        eprintln!("GAME-CRASH: {msg}"); // 等价性逐案核实（BUG-017 口径）
                        // 发行版无控制台（windows_subsystem=windows）⇒ 同步落 exe 目录
                        // xiyou-crash.log，诊断不断档（2026-10-18 控制台框修复配套）。
                        if let Ok(exe) = std::env::current_exe() {
                            if let Some(dir) = exe.parent() {
                                let _ = std::fs::OpenOptions::new()
                                    .create(true)
                                    .append(true)
                                    .open(dir.join("xiyou-crash.log"))
                                    .and_then(|mut f| {
                                        use std::io::Write;
                                        let secs = std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .map(|d| d.as_secs())
                                            .unwrap_or(0);
                                        writeln!(f, "[{secs}] GAME-CRASH: {msg}")
                                    });
                            }
                        }
                        self.paint_crashed();
                        self.state = Some(Screen::Crashed);
                    }
                }
            }
            Some(Screen::Crashed) => {
                // 崩溃后保持终止屏（不再 tick game-core）
                self.paint_crashed();
            }
            Some(Screen::Boot(mut machine)) => {
                // 【2026-10-17 BUG-018】主菜单读档列表同步：machine.preset_stores 原为
                // 开机快照——会话中保存后返回主菜单，读档屏看不到新存档。每步同步真源。
                machine.preset_stores = self.stores.clone();
                let key = std::mem::take(&mut self.pending_machine_key);
                if std::mem::take(&mut self.boot_key_audio_done) {
                    // 该键的 keyPressed 音频半已在 hide_notify 中按 Java 次序执行：屏蔽 tick
                    // 入口的延迟调用（`if key != 0 { bgm.on_key_pressed(mode) }`）——否则 hide
                    // 置的 anyKeyPlayed 被这个 hide 之前的键消费（误续播）。tick 内无其他读写者。
                    let armed = machine.bgm.any_key_played;
                    machine.bgm.any_key_played = false;
                    machine.tick(key);
                    machine.bgm.any_key_played = armed;
                } else {
                    machine.tick(key);
                }
                // 交接一：读档（槽载入动画完成 → mode 3）
                if machine.mode == GameMode::Map {
                    if let Some(cursor) = machine.slot_loaded {
                        let key = format!("MOT_L{cursor}@1");
                        if let Some((_, blob)) = self.stores.iter().find(|(k, _)| *k == key) {
                            let save = game_core::save::parse_floor_save(blob, 0).unwrap();
                            let fl = save.current_floor as usize;
                            if let Some(fa) = self.assets.floors.get(fl).cloned().flatten() {
                                let maplv = fa.maplv.clone();
                                let records = fa.records.clone();
                                let mut sc = game_core::scene::load_floor_from_save(
                                    &maplv,
                                    &records,
                                    &self.assets.width_table,
                                    &self.assets.tile_walk,
                                    self.assets.images.clone(),
                                    self.assets.floors.clone(),
                                    &save,
                                )
                                .unwrap();
                                sc.backdrop_scroll = machine.backdrop.scroll();
                                if let Some(t) = &machine.title {
                                    sc.menu_cursor = t.cursor;
                                    sc.bob_offset = machine.bob_offset;
                                    sc.bob_rising = machine.bob_rising;
                                }
                                sc.strip_frame = machine.strip_frame;
                                sc.preset_mot_if = self
                                    .stores
                                    .iter()
                                    .find(|(k, _)| k.starts_with("MOT_IF@"))
                                    .map(|(_, v)| v.clone());
                                Self::install_combat_tables(&mut sc);
                                sc.rng = machine.rng.clone();
                                // 音频状态续接（Java 同一对象：soundEnabled/track/
                                // bgm 跨模式连续；dispatchModeEntry case 3 已在 machine 侧执行）
                                sc.bgm = machine.bgm.clone();
                                self.persisted_blobs = sc.saved_slot_blobs.clone();
                                self.enter_scene(sc, machine);
                                return;
                            }
                        }
                    }
                }
                // 交接二：新游戏（序章冷启动——与 prologue 用例同构）
                if machine.new_game_ready {
                    let fa = self.assets.floors[51].as_ref().unwrap();
                    let maplv51 = fa.maplv.clone();
                    let records51 = fa.records.clone();
                    let mut sc = game_core::scene::GameScene::load_floor(
                        &maplv51,
                        32,
                        352,
                        &records51,
                        &self.assets.width_table,
                        &self.assets.tile_walk,
                        self.assets.images.clone(),
                        self.assets.floors.clone(),
                    )
                    .unwrap();
                    sc.floor = 51;
                    sc.min_floor = 51;
                    sc.max_floor = 51;
                    sc.player.hp = 300;
                    sc.player.atk = 10;
                    sc.player.def = 10;
                    Self::install_combat_tables(&mut sc);
                    sc.build_minimap();
                    sc.enter_view();
                    sc.bob_offset = machine.bob_offset;
                    sc.bob_rising = machine.bob_rising;
                    sc.strip_frame = machine.strip_frame;
                    sc.backdrop_scroll = machine.backdrop.scroll();
                    sc.rng = machine.rng.clone();
                    sc.bgm = machine.bgm.clone(); // 音频状态续接（同上）
                    sc.apply_entry_step_effects();
                    self.enter_scene(sc, machine);
                    return;
                }
                machine.paint(&mut canvas_graphics(&mut self.screen));
                self.state = Some(Screen::Boot(machine));
                self.frame_counter += 1;
            }
            None => {}
        }
    }
}

/// 壳级键盘动作。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyAction {
    /// MIDP 键码 + 按下?（经 route_key 投递给游戏）
    Game(i32, bool),
    /// F11 按下沿：全屏切换（壳级，不进游戏输入）
    ToggleFullscreen,
}

/// 窗口键盘事件 → 壳动作（OS 自动重复的**唯一**过滤点）。重复 ⇒ None：MIDP 把重复交给
/// keyRepeated，a.class 只覆写 keyPressed/keyReleased（`javap -p` 原始 a.class，A 级）⇒
/// 重复无效果；长按行走由 keyHeldCode 持续驱动（a.java:5308-5329，EV-hold-walk 实证）。
/// F11 只取按下沿，其抬起同样不进游戏输入。
fn translate_key(physical: winit::keyboard::PhysicalKey, pressed: bool, repeat: bool) -> Option<KeyAction> {
    use winit::keyboard::{KeyCode, PhysicalKey};
    if repeat {
        return None;
    }
    if physical == PhysicalKey::Code(KeyCode::F11) {
        return pressed.then_some(KeyAction::ToggleFullscreen);
    }
    key_code(physical).map(|c| KeyAction::Game(c, pressed))
}

/// 窗口事件 → 遮挡观测（lifecycle.rs 的一路输入）。只有 `Occluded` 映射可见性；
/// `Focused` **不**映射（用户决策 2026-10-15：失焦 ≠ 隐藏，失焦暂停已移除）；
/// 最小化在 X11 无事件（iconify = unmap，winit 不转发），由 `about_to_wait` 轮询
/// `Window::is_minimized()`。
fn occlusion_signal(event: &winit::event::WindowEvent) -> Option<bool> {
    match event {
        winit::event::WindowEvent::Occluded(occluded) => Some(*occluded),
        _ => None,
    }
}

/// 软键判区单源（WIN-OP③ 硬化：click_key 与合成共用此判区，禁双份谓词）：
/// 底部 30% 左三分之一 = -6、右三分之一 = -7；其余 None（=非软键区）。
/// `w`/`h` = 画布逻辑尺寸（随 Layout；classic 240×320 原值零变化）。
fn softkey_code(x: i32, y: i32, w: i32, h: i32) -> Option<i32> {
    if y <= h * 7 / 10 {
        return None;
    }
    if x < w / 3 {
        Some(-6)
    } else if x > w * 2 / 3 {
        Some(-7)
    } else {
        None
    }
}

/// 鼠标左键（画面坐标）→ 软键码：底部 30% 左三分之一 = -6、右三分之一 = -7、其余 = -5。
/// 【预留·安卓触控缝】WIN-OP 触控映射（点按语义单一源）——生产点击链现走
/// pointer_hit/click_key（行命中优先/基线兜底），本函数为点按区语义的独立单源
/// （测试面消费）；安卓触控缝接线时作映射源（2026-10-19 P0 死码批有据标注）。
#[allow(dead_code)]
fn click_code(x: i32, y: i32, w: i32, h: i32) -> i32 {
    softkey_code(x, y, w, h).unwrap_or(-5)
}

// ---------------------------------------------------------------- 窗口循环

fn run_window(shell: Shell) {
    use std::num::NonZeroU32;
    use winit::application::ApplicationHandler;
    use winit::event::{ElementState, WindowEvent};
    use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
    use winit::window::{Fullscreen, Window, WindowId};
    use game_platform::ProcessControl; // 接缝③：request_exit 方法解析

    type W = std::rc::Rc<Window>;

    /// 接缝③：winit 退出执行适配器（`el.exit()` 幂等；只能在事件循环回调内调用 ⇒
    /// 作用域薄适配器，契约 platform-contract §2.2 方案①）。
    struct WinitExit<'e> {
        el: &'e ActiveEventLoop,
    }

    impl ProcessControl for WinitExit<'_> {
        fn request_exit(&mut self) {
            self.el.exit();
        }
    }

    struct Runner {
        shell: Shell,
        window: Option<W>,
        context: Option<softbuffer::Context<W>>,
        surface: Option<softbuffer::Surface<W, W>>,
        pacer: view::Pacer,
        mouse: Option<(f64, f64)>,
    }

    impl ApplicationHandler for Runner {
        fn resumed(&mut self, el: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }
            let win = std::rc::Rc::new(
                el.create_window(
                    Window::default_attributes()
                        // CJK 标题（X11 _NET_WM_NAME 为 UTF-8；字形由窗口管理器渲染）
                        .with_title(WINDOW_TITLE)
                        .with_inner_size(winit::dpi::LogicalSize::new(
                            self.shell.layout.screen_w as u32 * INITIAL_SCALE,
                            self.shell.layout.screen_h as u32 * INITIAL_SCALE,
                        ))
                        .with_min_inner_size(winit::dpi::PhysicalSize::new(
                            self.shell.layout.screen_w as u32,
                            self.shell.layout.screen_h as u32,
                        ))
                        // 可缩放：画面取能放下的最大整数倍、信箱居中（view.rs），像素锐利；
                        // 桌面分数缩放下合成器的整窗平滑拉伸由此规避（按物理像素重排）
                        .with_resizable(true),
                )
                .unwrap(),
            );
            self.context = Some(softbuffer::Context::new(win.clone()).unwrap());
            let ctx = self.context.as_ref().unwrap();
            self.surface = Some(softbuffer::Surface::new(ctx, win.clone()).unwrap());
            win.request_redraw();
            self.window = Some(win);
        }

        fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
            if let Some(occluded) = occlusion_signal(&event) {
                self.shell.set_occluded(occluded); // 完全遮挡 ⇔ hideNotify / 解除 ⇔ showNotify
                return;
            }
            match event {
                WindowEvent::CloseRequested => WinitExit { el }.request_exit(),
                WindowEvent::RedrawRequested => self.present(),
                WindowEvent::Resized(_) => {
                    if let Some(w) = self.window.as_ref() {
                        w.request_redraw();
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse = Some((position.x, position.y));
                    // WIN-OP③ hover：命中行 → 光标跟随（菜单/背包选中高亮）
                    if let Some(vp) = self.viewport() {
                        if let Some((x, y)) = vp.to_screen(
                            position.x,
                            position.y,
                            self.shell.layout.screen_w as u32,
                            self.shell.layout.screen_h as u32,
                        ) {
                            self.shell.pointer_hover(x, y);
                        }
                    }
                }
                WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                    let Some((mx, my)) = self.mouse else { return };
                    let Some(vp) = self.viewport() else { return };
                    let Some((x, y)) = vp.to_screen(
                        mx,
                        my,
                        self.shell.layout.screen_w as u32,
                        self.shell.layout.screen_h as u32,
                    ) else {
                        return; // 信箱区
                    };
                    let right = button == winit::event::MouseButton::Right;
                    if !right && button != winit::event::MouseButton::Left {
                        return;
                    }
                    // WIN-OP③：命中行→对位+确认键；未命中→基线（软键/右键/确认，态门控）
                    let Some(code) = self.shell.pointer_click(x, y, right) else {
                        return;
                    };
                    // 点按 = 一次 press + release（下一拍消费 keyValue；不即时推进逻辑）
                    self.shell.route_key(code, true);
                    self.shell.route_key(code, false);
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    // WIN-OP②：滚轮 → 行导航（C-a 态门控：Map 行走态不合成）
                    let dy = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => y as f64,
                        winit::event::MouseScrollDelta::PixelDelta(p) => p.y / 40.0,
                    };
                    if let Some(code) = wheel_key(dy, self.shell.game_mode()) {
                        self.shell.route_key(code, true);
                        self.shell.route_key(code, false);
                    }
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    let pressed = event.state == ElementState::Pressed;
                    match translate_key(event.physical_key, pressed, event.repeat) {
                        Some(KeyAction::Game(code, down)) => self.shell.route_key(code, down),
                        Some(KeyAction::ToggleFullscreen) => {
                            if let Some(w) = self.window.as_ref() {
                                let fs = if w.fullscreen().is_some() { None } else { Some(Fullscreen::Borderless(None)) };
                                w.set_fullscreen(fs);
                            }
                        }
                        None => {}
                    }
                }
                // 失焦/得焦：不映射 hideNotify/showNotify（见 occlusion_signal）
                WindowEvent::Focused(_) => {}
                _ => {}
            }
        }

        fn about_to_wait(&mut self, el: &ActiveEventLoop) {
            let now = Instant::now();
            // 一次唤醒至多一拍（Java run 循环不补帧，view::Pacer）
            if self.pacer.due(now) {
                // 最小化无窗口事件：每拍入口轮询（X11 读 _NET_WM_STATE_HIDDEN；Wayland 恒 None）
                if let Some(minimized) = self.window.as_ref().and_then(|w| w.is_minimized()) {
                    self.shell.set_minimized(minimized);
                }
                self.shell.logic_tick();
                // CMidlet.dispatchModeEntry 退出信号消费点（BUG-020，接缝③）：game-core 发信号、
                // shell 经 ProcessControl 执行退出（粘滞轮询、幂等——consume_exit）
                self.shell.consume_exit(&mut WinitExit { el });
                let iv = self.shell.frame_interval();
                self.pacer.after_tick(now, iv);
                if let Some(w) = self.window.as_ref() {
                    w.request_redraw();
                }
            }
            // 必须设定时唤醒：winit 默认 ControlFlow::Wait 会睡到下一个事件（36d1a9d 实机根因）。
            el.set_control_flow(ControlFlow::WaitUntil(self.pacer.wake_at()));
        }
    }

    impl Runner {
        fn viewport(&self) -> Option<view::Viewport> {
            let size = self.window.as_ref()?.inner_size();
            (size.width > 0 && size.height > 0)
                .then(|| {
                    let (cw, ch) = (
                        self.shell.layout.screen_w as u32,
                        self.shell.layout.screen_h as u32,
                    );
                    view::Viewport::fit(size.width, size.height, cw, ch)
                })
        }

        fn present(&mut self) {
            let Some(vp) = self.viewport() else { return }; // 最小化（0×0）不画
            let Some(surface) = self.surface.as_mut() else { return };
            if surface
                .resize(NonZeroU32::new(vp.win_w).unwrap(), NonZeroU32::new(vp.win_h).unwrap())
                .is_err()
            {
                return;
            }
            let Ok(mut buffer) = surface.buffer_mut() else { return };
            let (cw, ch) = (
                self.shell.layout.screen_w as u32,
                self.shell.layout.screen_h as u32,
            );
            view::blit(&mut buffer, &vp, &self.shell.screen.argb, cw, ch);
            let _ = buffer.present();
        }
    }

    #[cfg(target_os = "linux")]
    let event_loop = {
        use winit::platform::x11::EventLoopBuilderExtX11 as _;
        EventLoop::builder().with_x11().build().unwrap()
    };
    #[cfg(not(target_os = "linux"))]
    let event_loop = EventLoop::new().unwrap();
    let first = shell.frame_interval();
    let mut runner = Runner {
        shell,
        window: None,
        context: None,
        surface: None,
        pacer: view::Pacer::new(Instant::now(), first),
        mouse: None,
    };
    event_loop.run_app(&mut runner).unwrap();
}

/// 平台默认存档目录（**注入契约**，2026-10-17 决策）：存档/读档路径由外部 shell
/// 传入（`--save` / 未来宿主 API），game-core 保持无 I/O；平台路径规范各异——
/// Windows = exe 目录\save（便携、抗 cwd 变化），Android = 应用私有目录（宿主
/// 启动时注入）。默认仅在未显式传入时生效。
fn default_save_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("save")
}

fn main() {
    let mut assets_dir = PathBuf::from("assets/raw");
    let mut save_dir = default_save_dir();
    let mut audio_dir = PathBuf::from("assets/audio");
    let mut audio_device = String::from("default");
    let mut mute = false;
    let mut preset_name = String::from("classic");
    let mut grid: Option<(i32, i32)> = None;
    // headless 截图（自验工具 2026-10-18）：无窗口跑 N 拍 → dump PNG
    let mut headless_shot: Option<PathBuf> = None;
    let mut headless_frames: u64 = 120;
    let mut headless_newgame = false;
    let mut inject_spec: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--assets" => assets_dir = PathBuf::from(args.next().unwrap_or_default()),
            "--save" => save_dir = PathBuf::from(args.next().unwrap_or_default()),
            "--audio" => audio_dir = PathBuf::from(args.next().unwrap_or_default()),
            "--audio-device" => audio_device = args.next().unwrap_or_else(|| "default".into()),
            "--mute" => mute = true,
            "--preset" => preset_name = args.next().unwrap_or_else(|| "classic".into()),
            "--headless-shot" => headless_shot = Some(PathBuf::from(args.next().unwrap_or_default())),
            "--frames" => headless_frames = args.next().and_then(|v| v.parse().ok()).unwrap_or(120),
            "--headless-newgame" => headless_newgame = true,
            "--inject" => inject_spec = Some(args.next().unwrap_or_default()),
            "--grid" => {
                let g = args.next().unwrap_or_default();
                let mut it = g.split(',');
                grid = Some((
                    it.next().and_then(|v| v.parse().ok()).unwrap_or(13),
                    it.next().and_then(|v| v.parse().ok()).unwrap_or(13),
                ));
            }
            other => eprintln!("忽略未知参数 {other}"),
        }
    }
    println!("资产目录 = {assets_dir:?}；存档目录 = {save_dir:?}；BGM 目录 = {audio_dir:?}");
    font_inject::install();
    let src = DirAssets::new(&assets_dir);
    let assets = std::sync::Arc::new(load_assets(&src));
    let (rms, stores) = Rms::load(&save_dir);
    let sink: Box<dyn audio::AudioSink> = if mute {
        eprintln!("[audio] --mute：无声运行");
        Box::new(audio::NullSink)
    } else {
        audio::start(&audio_dir, &audio_device)
    };
    let layout = parse_preset(&preset_name, grid);
    let mut shell = Shell::new(&src, assets, Box::new(rms), stores, sink, layout);
    // STAGE-B-001：全局声音设置装载（save 目录旁 app_settings.cfg）——不入游戏存档
    {
        let settings_path = default_save_dir().join("app_settings.cfg");
        let s = game_shell_common::settings::AppSettings::load(&settings_path);
        shell.settings_path = Some(settings_path);
        shell.apply_loaded_settings(s);
    }
    if let Some(out) = headless_shot {
        headless_run(shell, out, headless_frames, headless_newgame, inject_spec.as_deref());
        return;
    }
    run_window(shell);
}

/// headless 截图跑法（自验工具）：无窗口 N 拍 → 画布 dump PNG。
/// `newgame`=boot 后在标题按确认进游戏（验证 mode 3 视图面用）。
fn headless_run(mut shell: Shell, out: PathBuf, frames: u64, newgame: bool, inject: Option<&str>) {
    let mut pressed_at: Vec<(u64, i32)> = Vec::new();
    // --inject "tick:key,tick:key"（2026-10-19 自验通道扩展：任意键序列注入）
    if let Some(spec) = inject {
        for pair in spec.split(',') {
            if let Some((t, k)) = pair.split_once(':') {
                if let (Ok(t), Ok(k)) = (t.trim().parse(), k.trim().parse()) {
                    pressed_at.push((t, k));
                }
            }
        }
    }
    if newgame {
        // 标题确认（-5）在 boot 完成后投递——保守按拍轮询 title 就绪再按
        // 时序：boot→标题（~150 拍）→确认新游戏→序章→跳过→游戏视图
        for k in (150..230).step_by(20) {
            pressed_at.push((k, -5)); // 标题确认（新游戏）
        }
        for k in (230..340).step_by(10) {
            pressed_at.push((k, -6)); // 跳过序章过场
        }
    }
    for tick in 1..=frames {
        for &(t, code) in &pressed_at {
            if t == tick {
                shell.route_key(code, true);
                shell.route_key(code, false);
            }
        }
        shell.logic_tick();
    }
    // 画布 dump（Shell.screen = ArgbImage）
    let (w, h, px) = shell.canvas_snapshot();
    let png = headless_shot::encode_png(w, h, &px);
    let ok = std::fs::write(&out, &png).is_ok();
    eprintln!(
        "[headless-shot] {frames} 拍后 dump {}×{} → {}（{}）",
        w,
        h,
        out.display(),
        if ok { "OK" } else { "写失败" }
    );
}

#[cfg(test)]
mod shell_tests {
    //! 壳级集成测试：真 BootMachine → 读档交接 → GameScene，按 oracle 投递时序
    //! （TICK n 的输入在第 n 拍逻辑+重绘之后投递，第 n+1 拍消费）驱动 `Shell`，
    //! 与 Java 变体 A 的 EV-hold-walk trace 逐拍对照（期望值来源：
    //! `python3 tools/audio/evidence.py --fields gameMode,playerPixelX,playerPixelY,keyHeldCode,walkPhase
    //!   --script shell/win/tests/fixtures/hold-walk.txt`）。
    use super::*;
    use std::sync::{Arc, Mutex};

    struct Recorder(Arc<Mutex<Vec<(u64, audio::Cmd)>>>, Arc<Mutex<u64>>);

    impl audio::AudioSink for Recorder {
        fn send(&mut self, cmd: audio::Cmd) {
            let t = *self.1.lock().unwrap();
            self.0.lock().unwrap().push((t, cmd));
        }
    }

    fn repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// preset 文件（`KEY@1=hex`）→ RMS stores（与 oracle -Doracle.preset 同格式）。
    fn preset_stores(name: &str) -> Vec<(String, Vec<u8>)> {
        let text = std::fs::read_to_string(repo().join("reference/oracle/scenarios").join(name)).unwrap();
        text.lines()
            .filter_map(|l| l.split_once('='))
            .map(|(k, hex)| {
                let bytes = (0..hex.trim().len() / 2)
                    .map(|i| u8::from_str_radix(&hex.trim()[2 * i..2 * i + 2], 16).unwrap())
                    .collect();
                (k.trim().to_string(), bytes)
            })
            .collect()
    }

    /// 输入脚本 → [(tick, code, pressed)]（oracle 格式 `<tick> <code> <press|release>`）。
    fn script(path: &str) -> Vec<(u64, i32, bool)> {
        std::fs::read_to_string(repo().join(path))
            .unwrap()
            .lines()
            .filter_map(|l| {
                let f: Vec<&str> = l.split('#').next()?.split_whitespace().collect();
                (f.len() == 3).then(|| (f[0].parse().unwrap(), f[1].parse().unwrap(), f[2] == "press"))
            })
            .collect()
    }

    struct Harness {
        shell: Shell,
        cmds: Arc<Mutex<Vec<(u64, audio::Cmd)>>>,
        now: Arc<Mutex<u64>>,
        tick: u64,
    }

    impl Harness {
        fn new(preset: &str) -> Harness {
            let dir = repo().join("assets/raw");
            let src = DirAssets::new(&dir);
            let assets = std::sync::Arc::new(load_assets(&src));
            let save = std::env::temp_dir().join(format!("xiyou-shell-test-{}", std::process::id()));
            let (rms, _) = Rms::load(&save);
            let cmds = Arc::new(Mutex::new(Vec::new()));
            let now = Arc::new(Mutex::new(0));
            let sink = Box::new(Recorder(cmds.clone(), now.clone()));
            let shell = Shell::new(
                &src,
                assets,
                Box::new(rms),
                preset_stores(preset),
                sink,
                game_core::layout::Layout::classic(),
            );
            Harness { shell, cmds, now, tick: 0 }
        }

        /// 跑第 n 拍（logic+paint），然后投递脚本中 tick == n 的事件（preTick(n)）。
        fn run_tick(&mut self, events: &[(u64, i32, bool)]) {
            self.tick += 1;
            *self.now.lock().unwrap() = self.tick;
            self.shell.logic_tick();
            for &(t, code, pressed) in events {
                if t == self.tick {
                    self.shell.route_key(code, pressed);
                }
            }
        }

        fn scene(&self) -> &GameScene {
            match &self.shell.state {
                Some(Screen::Game(sc)) => sc,
                _ => panic!("T{} 尚未进入场景", self.tick),
            }
        }


        /// 跑到第 `t` 拍（含），沿途按脚本投递。
        fn run_to(&mut self, t: u64, events: &[(u64, i32, bool)]) {
            while self.tick < t {
                self.run_tick(events);
            }
        }

        /// 当前 game-core BGM 请求（宿主契约 bgm_request）。
        fn bgm(&self) -> Option<usize> {
            self.shell.bgm_snapshot().track
        }

        /// 已下发的播放命令（含拍号）。
        fn commands(&self) -> Vec<(u64, audio::Cmd)> {
            self.cmds.lock().unwrap().clone()
        }

        /// 标题菜单光标（BootMachine mode 1）。
        fn title_cursor(&self) -> i32 {
            match &self.shell.state {
                Some(Screen::Boot(m)) => m.title.as_ref().unwrap().cursor,
                _ => panic!("T{} 不在标题", self.tick),
            }
        }
    }

    /// 读档入 floor1 的输入（= hold-walk.txt 前 8 行）：T71 标题 + menu(2)；T237 mode 3 + game(3)。
    const LOAD_FLOOR1: [(u64, i32, bool); 8] = [
        (70, -6, true),
        (72, -6, false),
        (100, -2, true),
        (102, -2, false),
        (150, -5, true),
        (152, -5, false),
        (200, -5, true),
        (202, -5, false),
    ];

    const PLAY_MENU: audio::Cmd = audio::Cmd::Play { track: 2, gain: 0.6 };
    const PLAY_GAME: audio::Cmd = audio::Cmd::Play { track: 3, gain: 0.6 };

    /// STAGE-B-001（R1）：toggle→下一拍检测→落盘 时序。
    #[test]
    fn settings_toggle_persists_next_tick() {
        let mut h = Harness::new("preset-floor1.preset");
        let dir = std::env::temp_dir().join(format!("xiyou-settings-tick-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("app_settings.cfg");
        let _ = std::fs::remove_file(&p);
        h.shell.settings_path = Some(p.clone());
        h.run_to(250, &LOAD_FLOOR1);
        // 初始（默认开声）首拍落盘
        h.run_tick(&[]);
        assert!(p.exists(), "首拍落盘（默认态）");
        // toggle 关声（模拟设置翻转——直接置 bgm 态，时序=下一拍检测）
        if let Some(Screen::Game(sc)) = h.shell.state.as_mut() {
            sc.bgm.sound_enabled = false;
            sc.bgm.volume_level = 0;
        }
        assert!(!game_shell_common::settings::AppSettings::load(&p).sound_enabled == false, "未落盘前=旧值");
        h.run_tick(&[]);
        let s = game_shell_common::settings::AppSettings::load(&p);
        assert!(!s.sound_enabled, "toggle 后下一拍落盘（关声持久）");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// kind7 退出→换机回标题（2026-10-18 用户实报修复的壳级回归）：scene 置
    /// MainMenu 信号 → swap_back_to_boot_if_requested 换回 stashed BootMachine。
    /// **本测试即首版缺失面**——若有它，"stash 收储时 state 已被 take 走"当场红。
    #[test]
    fn kind7_exit_swaps_back_to_boot() {
        let mut h = Harness::new("preset-floor1.preset");
        h.run_to(250, &LOAD_FLOOR1);
        assert!(matches!(h.shell.state, Some(Screen::Game(_))), "前置：在场景中");
        // 注入游戏内菜单 kind7（退出）确认——模拟用户操作
        if let Some(Screen::Game(sc)) = h.shell.state.as_mut() {
            sc.game_mode = GameMode::InGameMenu;
            sc.menu_kinds = vec![7];
            sc.menu_item_count = 1;
            sc.menu_cursor = 0;
            sc.key_value = -6;
        }
        h.run_tick(&[]);
        assert!(
            matches!(h.shell.state, Some(Screen::Boot(_))),
            "kind7 退出后须换回 BootMachine（标题画面）"
        );
        // BGM 随换机（2026-10-18 用户实报）：换机后须回标题曲
        h.run_tick(&[]);
        assert_eq!(h.bgm(), Some(2), "换机后 BGM 须切回标题曲 track2（实得 {:?}）", h.bgm());
    }

    /// hideNotify 音频半 + 隐藏期门控 + keyPressed 续播（f_bool_28=true 分支 ⇒ game(3)），mode 3：
    /// - 遮挡当拍同步 Stop；anyKeyPlayed=true；f_bool_28 不变（a.java:4509-4510）；
    /// - 隐藏期键不投递（MIDP Event Delivery）：不消费续播边沿、不改 keyHeldCode；
    /// - 叠加最小化不重复 hide（f_bool_01 门控 4503）；两路都解除才 showNotify，show 不恢复 BGM；
    /// - 其后首键：playBgm(3) 关闭后重建（a.java:4668-4670），按键当拍下发（Java keyPressed 同步
    ///   createPlayer）；第二次按键无音频副作用（anyKeyPlayed 已清，4674）。
    ///
    /// gameMode 3→4 臂见 `hide_in_mode3_opens_in_game_menu`（BLOCKED）。
    #[test]
    fn in_game_hide_closes_bgm_and_first_key_resumes_game_track() {
        let mut h = Harness::new("preset-floor1.preset");
        h.run_to(250, &LOAD_FLOOR1);
        assert_eq!((h.scene().game_mode, h.bgm(), h.scene().bgm.in_game), (GameMode::Map, Some(3), true));
        let serial = h.scene().bgm_serial();
        let n = h.commands().len();

        h.shell.set_occluded(true);
        assert!(!h.shell.visibility.shown());
        assert_eq!(h.bgm(), None, "closeAudioPlayer（a.java:4509）");
        assert!(h.scene().bgm.any_key_played, "anyKeyPlayed=true（a.java:4510）");
        assert!(h.scene().bgm.in_game, "f_bool_28 不受 hide 影响");
        assert_eq!(h.commands()[n..], [(250, audio::Cmd::Stop)], "Stop 于 hide 当拍同步下发");

        h.shell.route_key(-2, true);
        assert_eq!(h.scene().key_held, 0, "隐藏期 keyPressed 不投递");
        assert!(h.scene().bgm.any_key_played, "隐藏期按键不得消费续播边沿");
        h.shell.route_key(-2, false);
        h.shell.set_minimized(true); // 第二路来源：已隐藏 ⇒ 无第二次 hideNotify
        for _ in 0..5 {
            h.run_tick(&[]);
        }
        assert_eq!(h.bgm(), None, "隐藏期逻辑照跑，BGM 保持关闭");
        h.shell.set_occluded(false);
        assert!(!h.shell.visibility.shown(), "仍最小化 ⇒ 仍隐藏");
        h.shell.set_minimized(false);
        assert!(h.shell.visibility.shown());
        assert_eq!(h.bgm(), None, "showNotify 不恢复 BGM（a.java:4495-4500 无音频调用）");
        assert_eq!(h.commands().len(), n + 1, "隐藏期间无其他命令");

        let t = h.tick;
        h.shell.route_key(-5, true);
        assert!(!h.scene().bgm.any_key_played, "anyKeyPlayed 清零（a.java:4674）");
        assert_eq!(h.bgm(), Some(3), "f_bool_28=true ⇒ m_139(3)（a.java:4669-4670）");
        assert_eq!(h.scene().bgm_serial(), serial + 1, "close 置 null ⇒ 重建（从头播放）");
        h.shell.route_key(-5, false);
        assert_eq!(h.commands()[n + 1..], [(t, PLAY_GAME)], "续播于按键当拍下发");
        h.run_tick(&[]);
        h.shell.route_key(-5, true);
        h.shell.route_key(-5, false);
        h.run_tick(&[]);
        assert_eq!(h.scene().bgm_serial(), serial + 1, "第二次按键无音频副作用");
        assert_eq!(h.commands().len(), n + 2);
    }

    /// 4504 只认 gameMode==3：游戏内菜单（mode 4，-7 打开，a.java:3555-3557）中隐藏 ⇒ 模式不变、
    /// 只关 BGM；恢复后首键续播 game(3)，且该键照常被 run case 4 消费（-2 下移）。
    #[test]
    fn hide_in_game_menu_keeps_mode_4() {
        let mut h = Harness::new("preset-floor1.preset");
        let mut ev = LOAD_FLOOR1.to_vec();
        ev.extend([(250, -7, true), (252, -7, false)]);
        h.run_to(260, &ev);
        assert_eq!((h.scene().game_mode, h.bgm()), (GameMode::InGameMenu, Some(3)));
        let (cursor, count) = (h.scene().menu_cursor, h.scene().menu_item_count);
        h.shell.set_minimized(true);
        assert_eq!((h.scene().game_mode, h.bgm()), (GameMode::InGameMenu, None), "mode≠3 ⇒ gameMode 不变（a.java:4504）");
        h.shell.route_key(-2, true);
        h.shell.route_key(-2, false);
        h.run_tick(&[]);
        assert_eq!(h.scene().menu_cursor, cursor, "隐藏期键不投递");
        h.shell.set_minimized(false);
        h.shell.route_key(-2, true);
        h.shell.route_key(-2, false);
        assert_eq!(h.bgm(), Some(3), "首键续播 game(3)");
        h.run_tick(&[]);
        let expect = if cursor < count - 1 { cursor + 1 } else { 0 };
        assert_eq!((h.scene().game_mode, h.scene().menu_cursor), (GameMode::InGameMenu, expect));
    }

    /// 标题（BootMachine，f_bool_28=false 分支 ⇒ 续播 menu(2)）+ 挂起键与 hide 的 Java 次序：
    /// ① 最小化 → Stop；隐藏期键不投递；恢复后首键在被消费的那拍续播 menu(2)（BootMachine::tick
    ///    入口的音频半）；
    /// ② 键在先、hide 在后（同一拍间隙）：Java keyPressed 时 anyKeyPlayed=false ⇒ 不续播，hide 再置
    ///    true ⇒ 该键被 tick 消费后仍静音、标志保持；
    /// ③ 同上但按键前 anyKeyPlayed 已为 true：Java keyPressed 当场 createPlayer、hide 随即 close
    ///    ⇒ Play+Stop 同拍，终态静音且标志为 true。
    #[test]
    fn title_hide_resumes_menu_track_in_java_event_order() {
        use audio::Cmd::Stop;
        let mut h = Harness::new("preset-floor1.preset");
        h.run_to(99, &LOAD_FLOOR1[..2]);
        assert_eq!((h.shell.game_mode(), h.bgm(), h.title_cursor()), (GameMode::MainMenu, Some(2), 0));
        let n = h.commands().len();
        // ①
        h.shell.set_minimized(true);
        assert_eq!((h.bgm(), h.shell.any_key_played()), (None, true));
        h.shell.route_key(-2, true);
        h.run_tick(&[]);
        assert_eq!(h.title_cursor(), 0, "隐藏期键不投递");
        h.shell.set_minimized(false);
        h.shell.route_key(-2, true);
        assert_eq!(h.bgm(), None, "BootMachine 的音频半随挂起键在 tick 入口执行");
        h.run_tick(&[]);
        assert_eq!((h.bgm(), h.shell.any_key_played(), h.title_cursor()), (Some(2), false, 1));
        let t1 = h.tick;
        // ②
        h.shell.route_key(-1, true);
        h.shell.set_occluded(true);
        let t2 = h.tick;
        h.run_tick(&[]);
        assert_eq!(
            (h.bgm(), h.shell.any_key_played(), h.title_cursor()),
            (None, true, 0),
            "hide 之前的键照常被消费，但不得消费 hide 置的标志"
        );
        // ③
        h.shell.set_occluded(false);
        h.shell.route_key(-2, true);
        h.shell.set_occluded(true);
        let t3 = h.tick;
        h.run_tick(&[]);
        assert_eq!((h.bgm(), h.shell.any_key_played(), h.title_cursor()), (None, true, 1));
        // 恢复后首键续播
        h.shell.set_occluded(false);
        h.shell.route_key(-1, true);
        h.run_tick(&[]);
        assert_eq!((h.bgm(), h.shell.any_key_played(), h.title_cursor()), (Some(2), false, 0));
        let t4 = h.tick;
        assert_eq!(
            h.commands()[n..],
            [(99, Stop), (t1, PLAY_MENU), (t2, Stop), (t3, PLAY_MENU), (t3, Stop), (t4, PLAY_MENU)],
            "①hide/续播 ②hide ③keyPressed 续播+hide close 同拍 ④续播"
        );
    }

    /// 续播的 gameMode 门控 `gameMode != 21 && gameMode != 0`（a.java:4666）：logo（mode 0）/
    /// 声音询问（mode 21）中 hide → 恢复 → 按键：只清 anyKeyPlayed、不续播；BGM 时间线与不隐藏时
    /// 相同（T71 menu(2) 为唯一命令——此时无播放器，close 不产生 Stop）。
    // 【STAGE-B-001 豁免 2026-10-18】mode 21（声音询问）已从 boot 链移除（有意不一致） 【2026-10-19 解禁：classic 预设门控恢复 mode 21/原版时间轴，本测复绿】
    //——本测对 mode 21 的前提消失；hide 语义在 mode 0/其他态的覆盖由同族测试兜。
    #[test]
    fn hide_in_modes_0_and_21_clears_flag_without_resume() {
        let mut h = Harness::new("preset-floor1.preset");
        h.run_to(10, &[]);
        assert_eq!(h.shell.game_mode(), GameMode::Boot);
        h.shell.set_occluded(true);
        assert!(h.shell.any_key_played());
        h.shell.set_occluded(false);
        h.shell.route_key(-5, true);
        h.run_tick(&[]);
        assert_eq!(
            (h.shell.game_mode(), h.bgm(), h.shell.any_key_played()),
            (GameMode::Boot, None, false),
            "mode 0：清标志、不续播"
        );
        h.run_to(69, &[]);
        assert_eq!(h.shell.game_mode(), GameMode::SoundAsk);
        h.shell.set_minimized(true);
        h.shell.set_minimized(false);
        assert!(h.shell.any_key_played());
        h.run_to(72, &LOAD_FLOOR1[..2]);
        assert_eq!((h.shell.game_mode(), h.bgm(), h.shell.any_key_played()), (GameMode::MainMenu, Some(2), false));
        assert_eq!(h.commands(), vec![(71, PLAY_MENU)], "mode 21 不续播；-6 照常开声");
    }

    /// hideNotify 的 gameMode 3→4 臂（a.java:4504-4507）：mode 3 隐藏 ⇒ gameMode=4 + dispatchModeEntry case 4
    /// （a.java:4326-4337：条目 [1,3,4,5,7]——floor≤50 无 17；软键 (1,3)）。
    /// BLOCKED：game-core 无公开入口（dispatchModeEntry case 4 = 私有 GameScene::enter_menu4）；菜单弹出属
    /// game-core 状态机、壳不复写。
    #[test]
    fn hide_in_mode3_opens_in_game_menu() {
        let mut h = Harness::new("preset-floor1.preset");
        h.run_to(250, &LOAD_FLOOR1);
        assert_eq!(h.scene().game_mode, GameMode::Map);
        h.shell.set_occluded(true);
        let sc = h.scene();
        assert_eq!(sc.game_mode, GameMode::InGameMenu, "gameMode=4（a.java:4505）");
        assert_eq!(sc.menu_kinds, vec![1, 3, 4, 5, 7], "m_000 case 4（a.java:4326-4337）");
        assert_eq!(sc.softkeys, (1, 3));
        assert_eq!((sc.bgm_request(), sc.bgm.any_key_played), (None, true));
    }

    /// 可见性只认遮挡：失焦/得焦不映射（用户决策 2026-10-15：失焦 ≠ 隐藏）。
    #[test]
    fn focus_is_not_visibility() {
        use winit::event::WindowEvent;
        assert_eq!(occlusion_signal(&WindowEvent::Occluded(true)), Some(true));
        assert_eq!(occlusion_signal(&WindowEvent::Occluded(false)), Some(false));
        assert_eq!(occlusion_signal(&WindowEvent::Focused(false)), None);
        assert_eq!(occlusion_signal(&WindowEvent::Focused(true)), None);
        assert_eq!(occlusion_signal(&WindowEvent::CloseRequested), None);
    }

    /// 长按连走 + BGM 切轨（Java EV-hold-walk 逐拍对照）：
    /// - 单次 press(-3) 长按 70 拍：战斗（walkPhase 5，T261-271）后连走两格
    ///   96→32（T273-280，每拍 8px），之后被阻挡原地；press(-2) 同理一格 320→352（T376-379）；
    /// - BGM：T71 menu（Java EVT createPlayer@T71）、T237 game（close+createPlayer@T237）。
    // 【STAGE-B-001 豁免 2026-10-18】BGM 切轨拍对照 Java EVT（createPlayer@T71） 【2026-10-19 解禁：classic 预设门控恢复 mode 21/原版时间轴，本测复绿】
    //——跳过询问后 boot 时间轴缩短，切轨拍必然漂移（有意不一致，登记册名单在案）。
    #[test]
    fn hold_walk_and_bgm_match_java_trace() {
        let ev = script("shell/win/tests/fixtures/hold-walk.txt");
        let mut h = Harness::new("preset-floor1.preset");
        let mut xs = std::collections::BTreeMap::new();
        let mut ys = std::collections::BTreeMap::new();
        for _ in 0..430 {
            h.run_tick(&ev);
            if h.tick >= 237 {
                let sc = h.scene();
                xs.insert(h.tick, sc.view.player_px);
                ys.insert(h.tick, sc.view.player_py);
            }
        }
        let java_x: Vec<(u64, i32)> =
            vec![(272, 96), (273, 88), (274, 80), (275, 72), (276, 64), (277, 56), (278, 48), (279, 40), (280, 32)];
        for (t, x) in java_x {
            assert_eq!(xs[&t], x, "T{t} playerPixelX");
        }
        assert!((280..=375).all(|t| xs[&t] == 32), "被阻挡后长按不再移动（Java T280-330 x=32）");
        let java_y: Vec<(u64, i32)> = vec![(375, 320), (376, 328), (377, 336), (378, 344), (379, 352)];
        for (t, y) in java_y {
            assert_eq!(ys[&t], y, "T{t} playerPixelY");
        }
        assert!((237..=375).all(|t| ys[&t] == 320));
        let cmds = h.cmds.lock().unwrap().clone();
        assert_eq!(
            cmds,
            vec![
                (71, audio::Cmd::Play { track: 2, gain: 0.6 }),
                (237, audio::Cmd::Play { track: 3, gain: 0.6 }),
            ],
            "BGM 切轨拍与 Java EVT（createPlayer@T71 / close+createPlayer@T237）一致"
        );
    }

    /// OS 自动重复无效果 + 同拍多键覆盖式取末个（R-2）：
    /// - 标题菜单长按 -2 并注入每拍一次的重复事件：光标只移 1 格（重复被 translate_key 丢弃）；
    /// - 同拍先按 -2 再按 -1：下一拍只消费末个（-1），光标回到 0。
    fn skywar_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xiyou-skywar-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn shell_at(dir: &PathBuf) -> Shell {
        shell_with_layout(dir, game_core::layout::Layout::classic())
    }

    fn shell_with_layout(dir: &PathBuf, layout: game_core::layout::Layout) -> Shell {
        let raw = repo().join("assets/raw");
        let src = DirAssets::new(&raw);
        let assets = std::sync::Arc::new(load_assets(&src));
        let (rms, stores) = Rms::load(dir);
        let cmds = Arc::new(Mutex::new(Vec::new()));
        let now = Arc::new(Mutex::new(0));
        Shell::new(&src, assets, Box::new(rms), stores, Box::new(Recorder(cmds, now)), layout)
    }

    /// SKY_WAR 端到端：水位抬升 ⇒ 落盘；重开同目录 ⇒ 状态恢复（saveGlobalProgress）。
    #[test]
    fn skywar_persists_across_restart() {
        let dir = skywar_dir("a");
        let mut shell = shell_at(&dir);
        if let Some(Screen::Boot(m)) = shell.state.as_mut() {
            m.note_max_floor(50);
        }
        shell.logic_tick();
        let blob = std::fs::read(dir.join("SKY_WAR.bin")).expect("SKY_WAR 落盘");
        assert_eq!(
            game_core::save::parse_global_save(&blob).unwrap().max_floor_watermark,
            50
        );
        let mut shell2 = shell_at(&dir);
        shell2.logic_tick();
        match shell2.state.as_ref() {
            Some(Screen::Boot(m)) => {
                assert_eq!(m.global_snapshot().max_floor_watermark, 50, "重启恢复水位")
            }
            _ => panic!("二代壳应在 Boot"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// STAGE-B-002 ⑤①②：preset 解析 + 画布随 Layout（classic 零变化=化回原值断言）。
    #[test]
    fn preset_selects_layout_and_canvas_follows() {
        let l = parse_preset("classic", None);
        assert_eq!((l.screen_w, l.screen_h), (240, 320), "classic 零变化");
        assert_eq!(
            (l.screen_w as u32 * INITIAL_SCALE, l.screen_h as u32 * INITIAL_SCALE),
            (480, 640),
            "窗口初值化回原值"
        );
        let m = parse_preset("modern", Some((13, 13)));
        assert_eq!((m.screen_w, m.screen_h), (416, 416), "modern 13×13 格");
        assert_eq!(parse_preset("modern", None).screen_w, 416, "modern 默认 13×13");
        // 画布随 Layout：壳屏缓冲 = layout 尺寸
        let dir = skywar_dir("preset");
        let shell = shell_with_layout(&dir, parse_preset("modern", Some((13, 13))));
        assert_eq!((shell.screen.width, shell.screen.height), (416, 416), "画布随 Layout");
        let shell = shell_with_layout(&dir, parse_preset("classic", None));
        assert_eq!((shell.screen.width, shell.screen.height), (240, 320), "classic 画布原值");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// SKY_WAR 损坏 ⇒ catch 语义（default_7 ⇒ [1]=true）并回写合法值（a.java:4475-4478）。
    #[test]
    fn skywar_corrupt_resets_default() {
        let dir = skywar_dir("b");
        std::fs::write(dir.join("SKY_WAR.bin"), [0xFFu8; 3]).unwrap();
        let mut shell = shell_at(&dir);
        shell.logic_tick();
        match shell.state.as_ref() {
            Some(Screen::Boot(m)) => {
                assert_eq!(m.global_snapshot().option_checked[1], true, "catch 分支 [1]=true")
            }
            _ => panic!("应在 Boot"),
        }
        let blob = std::fs::read(dir.join("SKY_WAR.bin")).unwrap();
        assert!(game_core::save::parse_global_save(&blob).is_ok(), "已回写合法值");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// BUG-021 ②c 落盘对偶环（markSlotEntry(-1) delete+addRecord，a.java:9000-9001
    /// →9026-9052）：槽表 parse_dirty（game-core 读失败触发器）⇒ MOT_IF@1 整表回写
    /// + stores 同步（Boot 侧读档列表重读源）；触发器一次性消费（重复拍不重写）。
    #[test]
    fn slot_if_parse_dirty_triggers_mot_if_writeback() {
        use crate::serialize_slot_if;
        let dir = skywar_dir("motif");
        let mut shell = shell_at(&dir);
        // 模拟"读失败已发生"的 game-core 侧状态：半解析表在内存 + 触发器置位
        let mut slot = game_core::menu_family::SlotSelect::new();
        slot.valid[0] = true;
        slot.hp[0] = 1234;
        slot.floor_num[0] = 7;
        slot.parse_dirty = true;
        if let Some(Screen::Boot(m)) = shell.state.as_mut() {
            m.slots = Some(slot.clone());
        }
        shell.logic_tick();
        let blob = std::fs::read(dir.join("MOT_IF.bin")).expect("MOT_IF 回写落盘");
        assert_eq!(
            blob,
            serialize_slot_if(&slot),
            "整表落盘 = markSlotEntry(-1) delete+addRecord 对偶（半解析表原样持久）"
        );
        assert!(
            shell.stores.iter().any(|(k, v)| k == "MOT_IF@1" && *v == blob),
            "stores 同步（BUG-018 读档列表重读源）"
        );
        // 触发器一次性：第二拍不重写
        let mtime = std::fs::metadata(dir.join("MOT_IF.bin")).unwrap().modified().unwrap();
        shell.logic_tick();
        assert_eq!(
            std::fs::metadata(dir.join("MOT_IF.bin")).unwrap().modified().unwrap(),
            mtime,
            "parse_dirty 一次性消费，重复拍不重写"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn key_repeat_ignored_and_last_press_wins() {
        let mut h = Harness::new("preset-floor1.preset");
        let ev = vec![(70u64, -6, true), (72, -6, false)];
        while h.tick < 99 {
            h.run_tick(&ev);
        }
        let cursor = |h: &Harness| match &h.shell.state {
            Some(Screen::Boot(m)) => m.title.as_ref().unwrap().cursor,
            _ => panic!(),
        };
        assert_eq!(cursor(&h), 0);
        let down = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowDown);
        let up = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowUp);
        let game = |k, pressed| match translate_key(k, pressed, false) {
            Some(KeyAction::Game(c, p)) => (c, p),
            other => panic!("期望游戏键，实得 {other:?}"),
        };
        // 按下 + 30 拍重复
        let (c, p) = game(down, true);
        h.shell.route_key(c, p);
        for _ in 0..30 {
            h.run_tick(&[]);
            assert!(translate_key(down, true, true).is_none(), "重复事件必须丢弃");
        }
        let (c, p) = game(down, false);
        h.shell.route_key(c, p);
        h.run_tick(&[]);
        assert_eq!(cursor(&h), 1, "长按只移一格（Java 无 keyRepeated）");
        // 同拍两键：-2 后 -1 ⇒ 只消费 -1
        let (c, _) = game(down, true);
        h.shell.route_key(c, true);
        let (c, _) = game(up, true);
        h.shell.route_key(c, true);
        h.run_tick(&[]);
        assert_eq!(cursor(&h), 0, "同拍多键取末个（-1：1→0），非逐个处理（会是 2→1）");
        // F11：只取按下沿（重复/抬起均无动作），且从不进游戏输入
        let f11 = winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F11);
        assert_eq!(translate_key(f11, true, false), Some(KeyAction::ToggleFullscreen));
        assert_eq!(translate_key(f11, true, true), None);
        assert_eq!(translate_key(f11, false, false), None);
    }

    /// WIN-OP 三件（2026-10-18 用户需求）：①ESC→-7（mode3 开菜单/面板=关闭返回，
    /// 键码契约右软键语义）②鼠标基线：滚轮/点击合成键码且**态门控**（C-a：Map 行走态
    /// 不合成 -1/-2，防 key_held 窜步；对话态滚轮=翻页保留）③右键→-7 全态。
    #[test]
    fn win_op_esc_and_mouse_baseline() {
        use winit::keyboard::{KeyCode, PhysicalKey};
        // ① ESC → -7（按下/抬起都路由；重复事件另有测试钉）
        let esc = PhysicalKey::Code(KeyCode::Escape);
        match translate_key(esc, true, false) {
            Some(KeyAction::Game(c, p)) => {
                assert_eq!(c, -7, "ESC 必须映射右软键 -7（mode3=开菜单/面板=关闭）");
                assert!(p);
            }
            other => panic!("ESC 未映射成游戏键：{other:?}"),
        }
        // ② 滚轮态门控（C-a）：Map=不合成；菜单/对话态=合成 -1/-2
        assert_eq!(wheel_key(1.0, GameMode::Map), None, "Map 态滚轮禁止（防窜步）");
        assert_eq!(wheel_key(-1.0, GameMode::Map), None);
        assert_eq!(wheel_key(1.0, GameMode::InGameMenu), Some(-1), "菜单态滚轮=上移");
        assert_eq!(wheel_key(-1.0, GameMode::Dialog), Some(-2), "对话态滚轮=下移/翻页（自然）");
        assert_eq!(wheel_key(0.0, GameMode::MainMenu), None, "零位移无动作");
        // ③ 点击基线：软键区优先全态；右键=-7 全态；非软键区左键=确认但 Map 态禁
        let (w, h) = (SCREEN_W, SCREEN_H);
        assert_eq!(click_key(5, h - 5, SCREEN_W, SCREEN_H, GameMode::Map, false), Some(-6), "软键区全态生效");
        assert_eq!(click_key(5, 5, SCREEN_W, SCREEN_H, GameMode::Map, true), Some(-7), "右键=返回全态");
        assert_eq!(click_key(5, 5, SCREEN_W, SCREEN_H, GameMode::Map, false), None, "Map 态点击不误触");
        assert_eq!(click_key(5, 5, SCREEN_W, SCREEN_H, GameMode::InGameMenu, false), Some(-5), "菜单态点击=确认");
        assert_eq!(w > 0, true); // 占位消未用告警
    }

    /// 点击分区（画面坐标，按 SCREEN_W/SCREEN_H 取点）：软键栏 = y > 高×7/10（边界行属
    /// 确认区）；栏内左 1/3 = -6、右 1/3 = -7、中间 = -5。
    #[test]
    fn click_zones_in_screen_space() {
        let (w, h) = (SCREEN_W, SCREEN_H);
        let bar = h - 20; // 软键栏内一行（240×320 下 = 300）
        assert_eq!(click_code(10, bar, w, h), -6);
        assert_eq!(click_code(w - 10, bar, w, h), -7);
        assert_eq!(click_code(w / 2, bar, w, h), -5);
        assert_eq!(click_code(10, h * 7 / 10, w, h), -5, "边界行（严格 >）不属软键栏");
        assert_eq!(click_code(10, h * 7 / 10 + 1, w, h), -6);
    }

    /// mode 22 退出信号（CMidlet.dispatchModeEntry 信号半，BUG-020 消费链的信号源）：标题"退出"
    /// （kind 5，title.rs:181 menu_kinds=[0..5]）→ 退场动画 70 拍 → finished 置位、之后粘滞。
    /// Java 锚（A 级）：run case 22（a.java:4249-4252）`++bootPhaseCounter ≤ 70 →
    /// runLogoAnimation(1,t)`，否则退出进程 ⇒ counter 1..=70 为动画、71 置位。
    /// 现状消费口径 = **粘滞信号每 tick 轮询**（main.rs about_to_wait 内 `if finished()`）：
    /// 本测锁信号半的时序与粘滞；执行半（el.exit()）语义 = 幂等请求、循环自然终止。
    #[test]
    fn exit_signal_flips_after_mode22_animation_and_sticks() {
        let mut h = Harness::new("preset-floor1.preset");
        let mut ev: Vec<(u64, i32, bool)> = LOAD_FLOOR1[..2].to_vec();
        // 标题菜单光标 0 → 5（kind 5 = 退出）：每拍一键（同拍多键取末个，R-2）
        for i in 0..5u64 {
            ev.push((75 + 2 * i, -2, true));
            ev.push((76 + 2 * i, -2, false));
        }
        ev.push((87, -5, true));
        ev.push((88, -5, false));
        h.run_to(90, &ev);
        assert_eq!(
            (h.shell.game_mode(), h.shell.finished()),
            (GameMode::LogoAnim, false),
            "kind 5 确认 ⇒ mode 22 退场动画（title.rs:235）"
        );
        // 逐拍推进：动画期 counter ≤ 70 且 finished=false；counter=71 置位；之后粘滞
        let mut flip: Option<(u64, i32)> = None;
        for _ in 0..90 {
            h.run_tick(&[]);
            let Some(Screen::Boot(m)) = h.shell.state.as_ref() else {
                panic!("mode 22 期间应保持 Boot");
            };
            assert_eq!(h.shell.finished(), m.finished, "Shell::finished 即 BootMachine::finished");
            if m.finished {
                flip = Some((h.tick, m.counter));
                break;
            }
            assert!(m.counter <= 70, "动画期 counter ≤ 70（a.java:4249-4252），实得 {}", m.counter);
        }
        let (t, counter) = flip.expect("70 拍退场动画后 finished 应置位（BUG-020 信号半）");
        assert_eq!(counter, 71, "++bootPhaseCounter=71 ⇒ 越过 ≤70 ⇒ finished（a.java:4249-4252）");
        eprintln!("finished 置位于 T{t}（mode 22 第 {counter} 拍）");
        // 粘滞：置位后每轮轮询都读到 true（消费半幂等，无"消费一次"语义）
        h.run_tick(&[]);
        assert!(h.shell.finished(), "退出信号粘滞（现状 = 每 tick 轮询）");
        h.run_tick(&[]);
        assert!(h.shell.finished(), "退出信号粘滞（重复轮询仍为 true）");
    }

    /// 接缝③ 判据（B3 缝测）：退出执行经 `game_platform::ProcessControl` trait 发出
    /// ——finished 前零请求；置位后**每轮轮询恰一次**（粘滞口径 = 现状 `if finished()`
    /// 每 tick 轮询；一次性由 el.exit() 幂等 + 循环终止保证，壳不加消费位）。
    #[test]
    fn exit_chain_goes_through_process_control_trait() {
        struct Probe(Arc<Mutex<usize>>);
        impl game_platform::ProcessControl for Probe {
            fn request_exit(&mut self) {
                *self.0.lock().unwrap() += 1;
            }
        }
        let mut h = Harness::new("preset-floor1.preset");
        let mut ev: Vec<(u64, i32, bool)> = LOAD_FLOOR1[..2].to_vec();
        for i in 0..5u64 {
            ev.push((75 + 2 * i, -2, true));
            ev.push((76 + 2 * i, -2, false));
        }
        ev.push((87, -5, true));
        ev.push((88, -5, false));
        h.run_to(90, &ev);
        assert_eq!(h.shell.game_mode(), GameMode::LogoAnim, "kind 5 → mode 22");
        let calls = Arc::new(Mutex::new(0usize));
        let mut probe = Probe(calls.clone());
        // 动画期：每轮 consume_exit 均 false、零请求；置位轮：true + 恰一次
        let mut flip = None;
        for _ in 0..90 {
            let requested = h.shell.consume_exit(&mut probe);
            if requested {
                flip = Some(h.tick);
                break;
            }
            assert_eq!(*calls.lock().unwrap(), 0, "退场动画期不得发出退出请求");
            h.run_tick(&[]);
        }
        let t = flip.expect("finished 置位后 consume_exit 应发出请求");
        assert_eq!(*calls.lock().unwrap(), 1, "置位后首个轮询恰一次请求");
        assert!(h.shell.consume_exit(&mut probe), "粘滞信号：轮询即请求");
        assert!(h.shell.consume_exit(&mut probe), "粘滞信号：重复轮询仍请求");
        assert_eq!(*calls.lock().unwrap(), 3, "每轮轮询一次（F4 口径 a，无消费位）");
        eprintln!("退出请求链起点 T{t}");
    }
}

// ---------------------------------------------------------------- 行为锚（迁移前基线）
//
// 【2026-10-17 迁移准备】四接缝 trait 化（Rms→Storage / PcmDevice→AudioOut /
// 退出链→ProcessControl / 资产→AssetSource）的**行为保持判据测试**：锁现状语义，
// 迁移后必须仍绿。纪律（adversary 裁定 2026-10-17）：本模块断言文本与期望值即
// "行为零漂移"判据——**不许改**；迁移期只许机械换被测物（Rms→FileRms 等）。
// 证据锚：docs/findings/reviews/2026-10-17-shell-win-migration-plan-review.md F3/F4/F5。

#[cfg(test)]
mod behavior_anchors {
    use super::*;

    fn repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn tmp(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xiyou-anchor-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn fnv1a(data: &[u8]) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for &b in data {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }

    fn img_sig(img: &ArgbImage) -> (i32, i32, u64) {
        let bytes: Vec<u8> = img.argb.iter().flat_map(|p| p.to_le_bytes()).collect();
        (img.width, img.height, fnv1a(&bytes))
    }

    /// catch_unwind 取 panic 文案（文案逐字保留判据用）。
    fn panic_payload(f: impl FnOnce() + std::panic::UnwindSafe) -> String {
        let e = match std::panic::catch_unwind(f) {
            Ok(_) => panic!("应 panic 而未 panic"),
            Err(e) => e,
        };
        e.downcast_ref::<String>()
            .cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    }

    /// panic 文案类 1（load_container）：缺容器 / 容器解析失败（文案逐字保留，F5）。
    #[test]
    fn panic_text_load_container_is_stable() {
        let dir = tmp("panic1");
        let d = dir.clone();
        let msg = panic_payload(move || {
            let _ = load_container(&DirAssets::new(&d), "mapbg", 1);
        });
        assert!(
            msg.starts_with("缺资产 mapbg（先跑 tools/extract-assets/extract.py）："),
            "缺资产文案（逐字保留）：{msg}"
        );
        std::fs::write(dir.join("mapbg"), b"garbage").unwrap();
        let d = dir.clone();
        let msg = panic_payload(move || {
            let _ = load_container(&DirAssets::new(&d), "mapbg", 1);
        });
        assert!(msg.starts_with("容器 mapbg 解析失败："), "解析失败文案（逐字保留）：{msg}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// panic 文案类 2（load_assets 内联）：缺 maplv{n} / sprite{n} 解析失败（逐字保留，F5）。
    #[test]
    fn panic_text_maplv_sprite_is_stable() {
        let raw = repo().join("assets/raw");
        let dir = tmp("panic2");
        for name in [
            "mapbg", "map", "actor", "sptmap", "sptprop", "sptarm", "sptenemy1", "sptenemy2",
            "ui", "menu", "face", "end", "xtq", "maplv1", "sprite1", "maplv2", "sprite2",
        ] {
            std::fs::copy(raw.join(name), dir.join(name)).unwrap();
        }
        let d = dir.clone();
        let msg = panic_payload(move || {
            let _ = load_assets(&DirAssets::new(&d));
        });
        assert!(msg.starts_with("缺资产 maplv3："), "缺 maplv 文案（逐字保留）：{msg}");
        std::fs::copy(raw.join("maplv3"), dir.join("maplv3")).unwrap();
        std::fs::write(dir.join("sprite2"), b"garbage").unwrap();
        let d = dir.clone();
        let msg = panic_payload(move || {
            let _ = load_assets(&DirAssets::new(&d));
        });
        assert!(msg.starts_with("sprite2 解析失败："), "sprite 解析文案（逐字保留）：{msg}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// panic 文案类 3（Shell::new 的 png()）：**保留现状 quirk**——前缀写死
    /// "assets/raw"（与 --assets 实际值无关），逐字锚定防"顺手修正"漂移（F5）。
    #[test]
    fn panic_text_boot_png_missing_is_stable() {
        let raw = repo().join("assets/raw");
        let dir = tmp("panic3");
        std::fs::copy(raw.join("mapbg"), dir.join("mapbg")).unwrap();
        let assets = std::sync::Arc::new(load_assets(&DirAssets::new(&raw)));
        let (rms, stores) = Rms::load(&dir.join("save"));
        let d = dir.clone();
        let msg = panic_payload(move || {
            let _ = Shell::new(
                &DirAssets::new(&d),
                assets,
                Box::new(rms),
                stores,
                Box::new(audio::NullSink),
                game_core::layout::Layout::classic(),
            );
        });
        assert!(
            msg.starts_with("缺 assets/raw/l0.png（先跑 tools/extract-assets/extract.py）："),
            "Boot 装配文案（逐字保留，含写死前缀 quirk）：{msg}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 资产摘要基线（A4/F4）：load_assets 全链（容器解码 + maplv/sprite 解析 + 表装配）
    /// 的内容摘要。采集：2026-10-17 迁移前实跑（基线 ba14d4e），迁移后必须原值。
    /// 口径与既有 shell_tests 相同（依赖 repo assets/raw）。
    #[test]
    fn assets_loading_matches_baseline_digest() {
        let a = load_assets(&DirAssets::new(&repo().join("assets/raw")));
        // 【采集模式】迁移前基线采集（2026-10-17，基线 ba14d4e）
        eprintln!("CAP tileset = {:?}", img_sig(&a.tileset));
        eprintln!("CAP backdrop = {:?}", img_sig(&a.images.backdrop));
        eprintln!("CAP actor = {:?}", img_sig(&a.images.actor));
        eprintln!("CAP menu_strip = {:?}", img_sig(&a.images.menu_strip));
        eprintln!("CAP ending = {:?}", img_sig(&a.images.ending));
        eprintln!(
            "CAP counts = {:?}",
            (a.images.faces.len(), a.images.ui.len(), a.images.xtq.len())
        );
        eprintln!(
            "CAP floors_shape = {:?}",
            (a.floors.len(), a.floors[0].is_none(), a.floors[55].is_none())
        );
        let f1 = a.floors[1].as_ref().unwrap();
        let f51 = a.floors[51].as_ref().unwrap();
        let f54 = a.floors[54].as_ref().unwrap();
        eprintln!(
            "CAP floors = {:?}",
            (
                fnv1a(&f1.maplv),
                f1.records.len(),
                fnv1a(&f51.maplv),
                f51.records.len(),
                fnv1a(&f54.maplv),
                f54.records.len()
            )
        );
        eprintln!(
            "CAP width_table = {:?}",
            (
                a.width_table.len(),
                fnv1a(&a.width_table.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>())
            )
        );
        eprintln!(
            "CAP tile_walk = {:?}",
            (
                a.tile_walk.len(),
                fnv1a(&a.tile_walk.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>())
            )
        );
        // 基线常量（采集输出原样冻结，迁移后必须逐值一致）
        assert_eq!(img_sig(&a.tileset), (128, 208, 6176440970670976425), "tileset 摘要");
        assert_eq!(img_sig(&a.images.backdrop), (77, 320, 2071193407069312118), "backdrop 摘要");
        assert_eq!(img_sig(&a.images.actor), (123, 138, 3341378134956609564), "actor 摘要");
        assert_eq!(img_sig(&a.images.menu_strip), (68, 102, 10121320654456436141), "menu_strip 摘要");
        assert_eq!(img_sig(&a.images.ending), (240, 145, 17415109984890522671), "ending 摘要");
        assert_eq!(
            (a.images.faces.len(), a.images.ui.len(), a.images.xtq.len()),
            (12, 25, 6),
            "容器计数"
        );
        assert_eq!(
            (a.floors.len(), a.floors[0].is_none(), a.floors[55].is_none()),
            (56, true, true),
            "楼层槽形"
        );
        let f1 = a.floors[1].as_ref().unwrap();
        let f51 = a.floors[51].as_ref().unwrap();
        let f54 = a.floors[54].as_ref().unwrap();
        assert_eq!(
            (
                fnv1a(&f1.maplv),
                f1.records.len(),
                fnv1a(&f51.maplv),
                f51.records.len(),
                fnv1a(&f54.maplv),
                f54.records.len()
            ),
            (3356879460706134156, 35, 8737405491698357268, 26, 11509423688834281314, 37),
            "楼层资产摘要"
        );
        assert_eq!(
            (
                a.width_table.len(),
                fnv1a(&a.width_table.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>())
            ),
            (88, 7539032330958311365),
            "width_table 摘要"
        );
        assert_eq!(
            (
                a.tile_walk.len(),
                fnv1a(&a.tile_walk.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>())
            ),
            (104, 5917174173154521492),
            "tile_walk 摘要"
        );
    }
}
