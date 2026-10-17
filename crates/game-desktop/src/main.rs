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
const CANVAS_W: u32 = SCREEN_W as u32;
const CANVAS_H: u32 = SCREEN_H as u32;
const WINDOW_TITLE: &str = "囧囧西游-大闹天宫";

// ---------------------------------------------------------------- 资产装载

macro_rules! fixture {
    ($t:literal) => {
        include_str!(concat!("../../game-core/tests/fixtures/", $t))
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect::<Vec<i32>>()
    };
}

fn load_container(dir: &PathBuf, name: &str, count: usize) -> Vec<ArgbImage> {
    let data = std::fs::read(dir.join(name))
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

fn load_assets(dir: &PathBuf) -> Assets {
    let mapbg = load_container(dir, "mapbg", 1);
    let map = load_container(dir, "map", 12);
    let actor_c = load_container(dir, "actor", 4);
    let sptmap = load_container(dir, "sptmap", 13);
    let sptprop = load_container(dir, "sptprop", 23);
    let sptarm = load_container(dir, "sptarm", 10);
    let sptenemy1 = load_container(dir, "sptenemy1", 20);
    let sptenemy2 = load_container(dir, "sptenemy2", 20);
    let ui = load_container(dir, "ui", 25);
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
        menu_strip: load_container(dir, "menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui: ui.clone(),
        faces: load_container(dir, "face", 12),
        ending: load_container(dir, "end", 1)[0].clone(),
        xtq: load_container(dir, "xtq", 6),
    };
    // 55 层全量装载（楼梯换层即时可用；maplv+sprite 每层 <2KB）
    let door_tables = game_core::sprite_spawn::DoorTables {
        door12: include_str!("../../game-core/tests/fixtures/door12.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
        door13: include_str!("../../game-core/tests/fixtures/door13.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
    };
    let mut floors: Vec<Option<FloorAssets>> = vec![None; 56];
    for (n, slot) in floors.iter_mut().enumerate().skip(1).take(54) {
        let maplv = std::fs::read(dir.join(format!("maplv{n}")))
            .unwrap_or_else(|e| panic!("缺资产 maplv{n}：{e}"));
        let sp = game_data::SpriteTable::parse(
            &std::fs::read(dir.join(format!("sprite{n}")))
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

// ---------------------------------------------------------------- RMS 落盘

/// 文件版 RecordStore：save/ 下每 store 一文件。与 BootMachine.preset_stores
/// 同键名（MOT_L0-2 / MOT_IF / SKY_WAR）。
struct Rms {
    dir: PathBuf,
}

impl Rms {
    fn load(dir: &PathBuf) -> (Rms, Vec<(String, Vec<u8>)>) {
        let mut stores = Vec::new();
        // 【2026-10-17 BUG-018】原只载 MOT_L0-2 三键——槽 3-5 的保存永不落盘（丢失）。
        // 槽表 6 槽（menu_family::SlotSelect），键域须全 6 槽。
        for name in [
            "MOT_L0", "MOT_L1", "MOT_L2", "MOT_L3", "MOT_L4", "MOT_L5", "MOT_IF", "SKY_WAR",
        ] {
            if let Ok(data) = std::fs::read(dir.join(format!("{name}.bin"))) {
                stores.push((format!("{name}@1"), data));
            }
        }
        (Rms { dir: dir.clone() }, stores)
    }

    fn persist(&self, stores: &[(String, Vec<u8>)]) {
        std::fs::create_dir_all(&self.dir).ok();
        for (k, v) in stores {
            let name = k.split('@').next().unwrap();
            let path = self.dir.join(format!("{name}.bin"));
            if std::fs::read(&path).ok().as_deref() != Some(v.as_slice()) {
                std::fs::write(&path, v).ok();
            }
        }
    }
}

/// scene.slot → MOT_IF 序列化（SlotSelect::load_slot_table 读序的镜像：
/// 6 × (valid u8; valid 时 floor/weapon/armor u8 + hp/atk/def/三钥/金 7×i32 BE)）。
fn serialize_slot_if(slot: &game_core::menu_family::SlotSelect) -> Vec<u8> {
    let mut o = Vec::new();
    for k in 0..6 {
        o.push(slot.valid[k] as u8);
        if slot.valid[k] {
            o.push(slot.floor_num[k] as u8);
            o.push(slot.weapon[k] as u8);
            o.push(slot.armor[k] as u8);
            for v in [
                slot.hp[k],
                slot.atk[k],
                slot.def[k],
                slot.keys_yellow[k] as i32,
                slot.keys_blue[k] as i32,
                slot.keys_red[k] as i32,
                slot.gold[k],
            ] {
                o.extend_from_slice(&v.to_be_bytes());
            }
        }
    }
    o
}

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
        _ => return None,
    })
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
    let mut g = SoftGraphics::new(screen);
    g.set_clip(0, 0, SCREEN_W, SCREEN_H);
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
    rms: Rms,
    stores: Vec<(String, Vec<u8>)>,
    screen: ArgbImage,
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
}

impl Shell {
    /// BootMachine 装配（与测试 harness 同参）。
    fn new(
        assets_dir: &PathBuf,
        assets: std::sync::Arc<Assets>,
        rms: Rms,
        stores: Vec<(String, Vec<u8>)>,
        sink: Box<dyn audio::AudioSink>,
    ) -> Shell {
        let mapbg = load_container(assets_dir, "mapbg", 1);
        let png = |name: &str| {
            ArgbImage::from_decoded_png(
                game_data::decode_png(
                    &std::fs::read(assets_dir.join(name))
                        .unwrap_or_else(|e| panic!("缺 assets/raw/{name}（先跑 tools/extract-assets/extract.py）：{e}")),
                )
                .unwrap(),
            )
        };
        let mut machine = BootMachine::new(
            png("l0.png"),
            png("l1.png"),
            load_container(assets_dir, "sflogo", 8),
            load_container(assets_dir, "ui", 25),
            load_container(assets_dir, "menu", 2),
            mapbg[0].clone(),
            load_container(assets_dir, "intro", 2),
            load_container(assets_dir, "end", 1),
            load_container(assets_dir, "load", 2),
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
        machine.sptarm = load_container(assets_dir, "sptarm", 10);
        Shell {
            assets,
            rms,
            stores,
            screen: ArgbImage::create(SCREEN_W, SCREEN_H),
            state: Some(Screen::Boot(machine)),
            pending_machine_key: 0,
            frame_counter: 0,
            persisted_blobs: [None, None, None, None, None, None],
            bgm: audio::BgmController::new(sink),
            visibility: lifecycle::Visibility::new(),
            boot_key_audio_done: false,
            resume_armed: false,
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
        // press_key 内的续播是同步 m_139（Java 在 keyPressed 回调当时 createPlayer）
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

    /// 一逻辑拍（logic → paint，同 Java run 循环体），随后同步 BGM 请求。可见性不影响出拍：
    /// Java run 线程在隐藏期照常循环（hideNotify 只改状态，不停线程）。
    fn logic_tick(&mut self) {
        self.step();
        self.sync_bgm();
        self.persist_global_if_dirty();
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

    fn paint_scene(&mut self, sc: &mut GameScene) {
        let mut g = canvas_graphics(&mut self.screen);
        sc.paint(&mut g, &self.assets.tileset);
    }

    /// 崩溃终止屏（P0-2 壳层统一）：game-core panic 的全宿主一致呈现。
    /// 纯壳层画面，不进 game-core 渲染语义（保真面零侵入）。
    fn paint_crashed(&mut self) {
        use game_core::layout::{SCREEN_H, SCREEN_W};
        let mut g = canvas_graphics(&mut self.screen);
        g.set_clip(0, 0, SCREEN_W, SCREEN_H);
        g.set_color(0x20_00_00);
        g.fill_rect(0, 0, SCREEN_W, SCREEN_H);
        g.set_color(0xFF_FF_FF);
        g.set_font(Some(game_core::paint::paint_font()));
        let l1: Vec<u16> = "游戏已终止".encode_utf16().collect();
        // 【2026-10-17 BUG-017 口径更正】原"（原版崩溃语义等价终止）"为**未经逐案
        // 核实的声称**——census assert（transition_to 未登记边）等 Rust 自造断言
        // panic 并非原版崩溃（用户实机被误导）。等价性须逐案以 Java 锚核实后另记。
        let l2: Vec<u16> = "（崩溃详情见控制台 GAME-CRASH 行）".encode_utf16().collect();
        g.draw_string(&l1, SCREEN_W / 2, SCREEN_H / 2 - 12, 17);
        g.draw_string(&l2, SCREEN_W / 2, SCREEN_H / 2 + 8, 17);
    }

    /// 交接拍收尾（读档 / 新游戏两路共用）：本拍即画场景（读档路 Java 本拍 paint 已是
    /// mode 3；两路均与 harness 同构）并切入 GameScene。
    fn enter_scene(&mut self, mut sc: GameScene) {
        // SKY_WAR 交接拷贝（Boot→GameScene）：不拷则场景水位从 0 起步会把持久水位写回退
        if let Some(Screen::Boot(m)) = self.state.as_ref() {
            let g = m.global_snapshot();
            sc.apply_global(g);
        }
        self.paint_scene(&mut sc);
        self.state = Some(Screen::Game(Box::new(sc)));
        self.frame_counter += 1;
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
                                // 音频状态续接（Java 同一对象：soundEnabled/f_byte_28/
                                // f_Player_00 跨模式连续；m_000 case 3 已在 machine 侧执行）
                                sc.bgm = machine.bgm.clone();
                                self.persisted_blobs = sc.saved_slot_blobs.clone();
                                self.enter_scene(sc);
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
                    self.enter_scene(sc);
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

/// 鼠标左键（画面坐标）→ 软键码：底部 30% 左三分之一 = -6、右三分之一 = -7、其余 = -5。
fn click_code(x: i32, y: i32) -> i32 {
    let (w, h) = (SCREEN_W, SCREEN_H);
    let soft_bar = y > h * 7 / 10;
    if soft_bar && x < w / 3 {
        -6
    } else if soft_bar && x > w * 2 / 3 {
        -7
    } else {
        -5
    }
}

// ---------------------------------------------------------------- 窗口循环

fn run_window(shell: Shell) {
    use std::num::NonZeroU32;
    use winit::application::ApplicationHandler;
    use winit::event::{ElementState, WindowEvent};
    use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
    use winit::window::{Fullscreen, Window, WindowId};

    type W = std::rc::Rc<Window>;

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
                            CANVAS_W * INITIAL_SCALE,
                            CANVAS_H * INITIAL_SCALE,
                        ))
                        .with_min_inner_size(winit::dpi::PhysicalSize::new(CANVAS_W, CANVAS_H))
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
                WindowEvent::CloseRequested => el.exit(),
                WindowEvent::RedrawRequested => self.present(),
                WindowEvent::Resized(_) => {
                    if let Some(w) = self.window.as_ref() {
                        w.request_redraw();
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse = Some((position.x, position.y));
                }
                WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                    if button != winit::event::MouseButton::Left {
                        return;
                    }
                    let Some((mx, my)) = self.mouse else { return };
                    let Some(vp) = self.viewport() else { return };
                    let Some((x, y)) = vp.to_screen(mx, my, CANVAS_W, CANVAS_H) else {
                        return; // 信箱区
                    };
                    let code = click_code(x, y);
                    // 点按 = 一次 press + release（下一拍消费 keyValue；不即时推进逻辑）
                    self.shell.route_key(code, true);
                    self.shell.route_key(code, false);
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
                .then(|| view::Viewport::fit(size.width, size.height, CANVAS_W, CANVAS_H))
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
            view::blit(&mut buffer, &vp, &self.shell.screen.argb, CANVAS_W, CANVAS_H);
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

fn main() {
    let mut assets_dir = PathBuf::from("assets/raw");
    let mut save_dir = PathBuf::from("save");
    let mut audio_dir = PathBuf::from("assets/audio");
    let mut audio_device = String::from("default");
    let mut mute = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--assets" => assets_dir = PathBuf::from(args.next().unwrap_or_default()),
            "--save" => save_dir = PathBuf::from(args.next().unwrap_or_default()),
            "--audio" => audio_dir = PathBuf::from(args.next().unwrap_or_default()),
            "--audio-device" => audio_device = args.next().unwrap_or_else(|| "default".into()),
            "--mute" => mute = true,
            other => eprintln!("忽略未知参数 {other}"),
        }
    }
    println!("资产目录 = {assets_dir:?}；存档目录 = {save_dir:?}；BGM 目录 = {audio_dir:?}");
    font_inject::install();
    let assets = std::sync::Arc::new(load_assets(&assets_dir));
    let (rms, stores) = Rms::load(&save_dir);
    let sink: Box<dyn audio::AudioSink> = if mute {
        eprintln!("[audio] --mute：无声运行");
        Box::new(audio::NullSink)
    } else {
        audio::start(&audio_dir, &audio_device)
    };
    let shell = Shell::new(&assets_dir, assets, rms, stores, sink);
    run_window(shell);
}

#[cfg(test)]
mod shell_tests {
    //! 壳级集成测试：真 BootMachine → 读档交接 → GameScene，按 oracle 投递时序
    //! （TICK n 的输入在第 n 拍逻辑+重绘之后投递，第 n+1 拍消费）驱动 `Shell`，
    //! 与 Java 变体 A 的 EV-hold-walk trace 逐拍对照（期望值来源：
    //! `python3 tools/audio/evidence.py --fields gameMode,playerPixelX,playerPixelY,keyHeldCode,walkPhase
    //!   --script crates/game-desktop/tests/fixtures/hold-walk.txt`）。
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
            let assets = std::sync::Arc::new(load_assets(&dir));
            let save = std::env::temp_dir().join(format!("xiyou-shell-test-{}", std::process::id()));
            let (rms, _) = Rms::load(&save);
            let cmds = Arc::new(Mutex::new(Vec::new()));
            let now = Arc::new(Mutex::new(0));
            let sink = Box::new(Recorder(cmds.clone(), now.clone()));
            let shell = Shell::new(&dir, assets, rms, preset_stores(preset), sink);
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

    /// hideNotify 音频半 + 隐藏期门控 + keyPressed 续播（f_bool_28=true 分支 ⇒ game(3)），mode 3：
    /// - 遮挡当拍同步 Stop；anyKeyPlayed=true；f_bool_28 不变（a.java:4509-4510）；
    /// - 隐藏期键不投递（MIDP Event Delivery）：不消费续播边沿、不改 keyHeldCode；
    /// - 叠加最小化不重复 hide（f_bool_01 门控 4503）；两路都解除才 showNotify，show 不恢复 BGM；
    /// - 其后首键：m_139(3) 关闭后重建（a.java:4668-4670），按键当拍下发（Java keyPressed 同步
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

    /// hideNotify 的 gameMode 3→4 臂（a.java:4504-4507）：mode 3 隐藏 ⇒ gameMode=4 + m_000 case 4
    /// （a.java:4326-4337：条目 [1,3,4,5,7]——floor≤50 无 17；软键 (1,3)）。
    /// BLOCKED：game-core 无公开入口（m_000 case 4 = 私有 GameScene::enter_menu4）；菜单弹出属
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
    #[test]
    fn hold_walk_and_bgm_match_java_trace() {
        let ev = script("crates/game-desktop/tests/fixtures/hold-walk.txt");
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
        let raw = repo().join("assets/raw");
        let assets = std::sync::Arc::new(load_assets(&raw));
        let (rms, stores) = Rms::load(dir);
        let cmds = Arc::new(Mutex::new(Vec::new()));
        let now = Arc::new(Mutex::new(0));
        Shell::new(&raw, assets, rms, stores, Box::new(Recorder(cmds, now)))
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

    /// 点击分区（画面坐标，按 SCREEN_W/SCREEN_H 取点）：软键栏 = y > 高×7/10（边界行属
    /// 确认区）；栏内左 1/3 = -6、右 1/3 = -7、中间 = -5。
    #[test]
    fn click_zones_in_screen_space() {
        let (w, h) = (SCREEN_W, SCREEN_H);
        let bar = h - 20; // 软键栏内一行（240×320 下 = 300）
        assert_eq!(click_code(10, bar), -6);
        assert_eq!(click_code(w - 10, bar), -7);
        assert_eq!(click_code(w / 2, bar), -5);
        assert_eq!(click_code(10, h * 7 / 10), -5, "边界行（严格 >）不属软键栏");
        assert_eq!(click_code(10, h * 7 / 10 + 1), -6);
    }
}
