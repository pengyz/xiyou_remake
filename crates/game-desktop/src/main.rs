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
//! 窗口：可缩放，整数倍缩放 + 信箱居中；失焦冻结逻辑与 BGM，回前台续跑（不追帧）。
//! BGM：game-core `bgm_request()` 驱动（docs/spec/audio.md）；产物由
//!       `cargo run --release --manifest-path tools/audio/Cargo.toml` 生成，缺失则无声运行。
//! 存档：./save/ 目录（MOT_L0-2 = 槽存档、MOT_IF = 槽表、SKY_WAR = boss 旗标），
//!       与 BootMachine 的 RMS 预置同格式——游戏内「保存游戏」即落盘。

mod audio;
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
    for n in 1..=54usize {
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
        floors[n] = Some(FloorAssets { maplv, records });
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
        for name in ["MOT_L0", "MOT_L1", "MOT_L2", "MOT_IF", "SKY_WAR"] {
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

    fn load() -> Option<FontVec> {
        for path in CANDIDATES {
            if let Ok(bytes) = std::fs::read(path) {
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
    mod tests {
        use super::*;
        use game_core::layout::{SCREEN_H, SCREEN_W, SOUND_PROMPT_TEXT};
        use game_core::render::{anchor, ArgbImage, SoftGraphics};

        /// 真链路端到端：注入 hook 后，经 SoftGraphics::draw_string 画 mode 21
        /// 提示文本，统计真字形像素。这是 shell 里 mode 21 文字是否可见的最小复现。
        #[test]
        fn hook_renders_mode21_text_via_softgraphics() {
            install(); // 注入真字形（与 shell 启动同一路径）
            let mut img = ArgbImage::create(SCREEN_W, SCREEN_H);
            {
                let mut g = SoftGraphics::new(&mut img);
                g.set_clip(0, 0, SCREEN_W, SCREEN_H);
                g.set_font(Some(game_core::paint::paint_font()));
                g.set_color(0x000000);
                g.fill_rect(0, 0, SCREEN_W, SCREEN_H);
                g.set_color(game_core::layout::WHITE);
                g.draw_string(SOUND_PROMPT_TEXT, 120, 160, anchor::TOP | anchor::HCENTER);
            }
            // 区域断言（防假绿）：文本带必须有像素、原点邻域必须干净——
            // ab_glyph 局部坐标 bug 的特征签名 = 字形堆在 (0,0) 且文本带为空。
            let (mut band, mut origin) = (0usize, 0usize);
            let w = SCREEN_W as usize;
            for y in 0..SCREEN_H as usize {
                for x in 0..w {
                    if img.argb[y * w + x] & 0x00FF_FFFF == 0 {
                        continue;
                    }
                    if (40..210).contains(&x) && (150..185).contains(&y) {
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

enum Screen {
    Boot(BootMachine),
    Game(Box<GameScene>),
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
        }
    }

    fn combat_tables() -> game_core::combat::CombatTables {
        game_core::combat::CombatTables {
            enemy_base_hp: fixture!("enemybasehp.txt"),
            enemy_base_atk: fixture!("enemybaseatk.txt"),
            enemy_base_def: fixture!("enemybasedef.txt"),
            difficulty_multipliers: fixture!("difficulty_multipliers.txt"),
        }
    }

    /// keyPressed / keyReleased 投递——Java Canvas 回调语义（a.java:4663-4680）：只改
    /// 状态、不推进逻辑，由下一逻辑拍消费；同拍多次按下覆盖式取末个（keyValue =
    /// keyHeldCode = code）；抬起只清 keyHeldCode（不论哪个键）。docs/spec/p3-render.md R-2。
    fn route_key(&mut self, code: i32, pressed: bool) {
        match (self.state.as_mut(), pressed) {
            (Some(Screen::Game(sc)), true) => sc.press_key(code),
            (Some(Screen::Game(sc)), false) => sc.release_key(),
            // BootMachine 各模式只读 keyValue（无 keyHeldCode 消费者）⇒ 抬起无效果
            (Some(Screen::Boot(_)), true) => self.pending_machine_key = code,
            _ => {}
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
            None => audio::BgmSnapshot::default(),
        }
    }

    /// 一逻辑拍（logic → paint，同 Java run 循环体），随后同步 BGM 请求。
    fn logic_tick(&mut self) {
        self.step();
        let snap = self.bgm_snapshot();
        self.bgm.update(snap);
    }

    fn paint_scene(&mut self, sc: &mut GameScene) {
        let mut g = SoftGraphics::new(&mut self.screen);
        g.set_clip(0, 0, SCREEN_W, SCREEN_H);
        g.set_font(Some(game_core::paint::paint_font()));
        sc.paint(&mut g, &self.assets.tileset);
    }

    fn step(&mut self) {
        match self.state.take() {
            Some(Screen::Game(mut sc)) => {
                // 存档落盘（游戏内「保存游戏」→ saved_slot_blobs/slot 变化即写穿）
                if sc.saved_slot_blobs != self.persisted_blobs {
                    let mut stores = std::mem::take(&mut self.stores);
                    for (k, v) in stores.iter_mut() {
                        if let Some(r) = k.strip_prefix("MOT_L") {
                            let idx: usize = r.trim_end_matches("@1").parse().unwrap();
                            if let Some(blob) = &sc.saved_slot_blobs[idx] {
                                *v = blob.clone();
                            }
                        }
                    }
                    let mot_if = serialize_slot_if(&sc.slot);
                    stores.retain(|(k, _)| !k.starts_with("MOT_IF@"));
                    stores.push(("MOT_IF@1".into(), mot_if.clone()));
                    self.rms.persist(&stores);
                    self.stores = stores;
                    self.persisted_blobs = sc.saved_slot_blobs.clone();
                }
                sc.tick(self.frame_counter as i64, &self.assets.width_table);
                self.frame_counter += 1;
                self.paint_scene(&mut sc);
                self.state = Some(Screen::Game(sc));
            }
            Some(Screen::Boot(mut machine)) => {
                machine.tick(std::mem::take(&mut self.pending_machine_key));
                // 交接一：读档（槽载入动画完成 → mode 3）
                if machine.mode == 3 {
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
                                let tables = Self::combat_tables();
                                sc.enemies = game_core::combat::scale_enemy_stats(&tables, 1);
                                sc.enemy_base_hp = tables.enemy_base_hp;
                                sc.enemy_base_gold = fixture!("enemybasegold.txt");
                                sc.rng = machine.rng.clone();
                                // 音频状态续接（Java 同一对象：soundEnabled/f_byte_28/
                                // f_Player_00 跨模式连续；m_000 case 3 已在 machine 侧执行）
                                sc.bgm = machine.bgm.clone();
                                self.persisted_blobs = sc.saved_slot_blobs.clone();
                                // 交接拍即画场景（Java 本拍 paint 已是 mode 3；harness 同）
                                self.paint_scene(&mut sc);
                                self.state = Some(Screen::Game(Box::new(sc)));
                                self.frame_counter += 1;
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
                    let tables = Self::combat_tables();
                    sc.enemies = game_core::combat::scale_enemy_stats(&tables, 1);
                    sc.enemy_base_hp = tables.enemy_base_hp;
                    sc.enemy_base_gold = fixture!("enemybasegold.txt");
                    sc.build_minimap();
                    sc.enter_view();
                    sc.bob_offset = machine.bob_offset;
                    sc.bob_rising = machine.bob_rising;
                    sc.strip_frame = machine.strip_frame;
                    sc.backdrop_scroll = machine.backdrop.scroll();
                    sc.rng = machine.rng.clone();
                    sc.bgm = machine.bgm.clone(); // 音频状态续接（同上）
                    sc.apply_entry_step_effects();
                    self.paint_scene(&mut sc);
                    self.state = Some(Screen::Game(Box::new(sc)));
                    self.frame_counter += 1;
                    return;
                }
                {
                    let mut g = SoftGraphics::new(&mut self.screen);
                    g.set_clip(0, 0, SCREEN_W, SCREEN_H);
                    g.set_font(Some(game_core::paint::paint_font()));
                    machine.paint(&mut g);
                }
                self.state = Some(Screen::Boot(machine));
                self.frame_counter += 1;
            }
            None => {}
        }
    }
}

/// 窗口键盘事件 → (键码, 按下?)。OS 自动重复 ⇒ None：MIDP 把重复交给 keyRepeated，
/// a.class 只覆写 keyPressed/keyReleased（`javap -p` 原始 a.class，A 级）⇒ 重复无效果；
/// 长按行走由 keyHeldCode 持续驱动（a.java:5308-5329，EV-hold-walk 实证）。
fn translate_key(physical: winit::keyboard::PhysicalKey, pressed: bool, repeat: bool) -> Option<(i32, bool)> {
    if repeat {
        return None;
    }
    key_code(physical).map(|c| (c, pressed))
}

/// 鼠标左键（画面坐标）→ 软键码：底部 30% 左三分之一 = -6、右三分之一 = -7、其余 = -5。
fn click_code(x: i32, y: i32) -> i32 {
    let (w, h) = (SCREEN_W, SCREEN_H);
    if y > h * 7 / 10 && x < w / 3 {
        -6
    } else if y > h * 7 / 10 && x > w * 2 / 3 {
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
    use winit::keyboard::{KeyCode, PhysicalKey};
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
                            SCREEN_W as u32 * INITIAL_SCALE,
                            SCREEN_H as u32 * INITIAL_SCALE,
                        ))
                        .with_min_inner_size(winit::dpi::PhysicalSize::new(SCREEN_W as u32, SCREEN_H as u32))
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
            match event {
                WindowEvent::CloseRequested => el.exit(),
                WindowEvent::RedrawRequested => self.present(),
                WindowEvent::Resized(_) => {
                    if let Some(w) = self.window.as_ref() {
                        w.request_redraw();
                    }
                }
                WindowEvent::Focused(focused) => {
                    // 失焦冻结：逻辑 tick 与 BGM 一并暂停；回前台从「现在 + 一拍」续跑。
                    // 壳级行为（非 Java hideNotify 语义，见 docs/spec/audio.md §4）。
                    let iv = self.shell.frame_interval();
                    if self.pacer.set_paused(!focused, Instant::now(), iv) {
                        self.shell.bgm.set_paused(!focused);
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse = Some((position.x, position.y));
                }
                WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                    if button != winit::event::MouseButton::Left || self.pacer.paused() {
                        return;
                    }
                    let Some((mx, my)) = self.mouse else { return };
                    let Some(vp) = self.viewport() else { return };
                    let Some((x, y)) = vp.to_screen(mx, my, SCREEN_W as u32, SCREEN_H as u32) else {
                        return; // 信箱区
                    };
                    let code = click_code(x, y);
                    // 点按 = 一次 press + release（下一拍消费 keyValue；不即时推进逻辑）
                    self.shell.route_key(code, true);
                    self.shell.route_key(code, false);
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    let pressed = event.state == ElementState::Pressed;
                    if event.physical_key == PhysicalKey::Code(KeyCode::F11) {
                        if pressed && !event.repeat {
                            if let Some(w) = self.window.as_ref() {
                                let fs = if w.fullscreen().is_some() { None } else { Some(Fullscreen::Borderless(None)) };
                                w.set_fullscreen(fs);
                            }
                        }
                        return;
                    }
                    if let Some((code, down)) = translate_key(event.physical_key, pressed, event.repeat) {
                        self.shell.route_key(code, down);
                    }
                }
                _ => {}
            }
        }

        fn about_to_wait(&mut self, el: &ActiveEventLoop) {
            let now = Instant::now();
            // 一次唤醒至多一拍（Java run 循环不补帧，view::Pacer）
            if self.pacer.due(now) {
                self.shell.logic_tick();
                let iv = self.shell.frame_interval();
                self.pacer.after_tick(now, iv);
                if let Some(w) = self.window.as_ref() {
                    w.request_redraw();
                }
            }
            // 必须设定时唤醒：winit 默认 ControlFlow::Wait 会睡到下一个事件（36d1a9d 实机根因）；
            // 冻结期则一直睡到事件（不空转）。
            el.set_control_flow(match self.pacer.wake_at() {
                Some(t) => ControlFlow::WaitUntil(t),
                None => ControlFlow::Wait,
            });
        }
    }

    impl Runner {
        fn viewport(&self) -> Option<view::Viewport> {
            let size = self.window.as_ref()?.inner_size();
            (size.width > 0 && size.height > 0)
                .then(|| view::Viewport::fit(size.width, size.height, SCREEN_W as u32, SCREEN_H as u32))
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
            view::blit(&mut buffer, &vp, &self.shell.screen.argb, SCREEN_W as u32, SCREEN_H as u32);
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
        // 按下 + 30 拍重复
        let (c, p) = translate_key(down, true, false).unwrap();
        h.shell.route_key(c, p);
        for _ in 0..30 {
            h.run_tick(&[]);
            assert!(translate_key(down, true, true).is_none(), "重复事件必须丢弃");
        }
        let (c, p) = translate_key(down, false, false).unwrap();
        h.shell.route_key(c, p);
        h.run_tick(&[]);
        assert_eq!(cursor(&h), 1, "长按只移一格（Java 无 keyRepeated）");
        // 同拍两键：-2 后 -1 ⇒ 只消费 -1
        let (c, _) = translate_key(down, true, false).unwrap();
        h.shell.route_key(c, true);
        let (c, _) = translate_key(up, true, false).unwrap();
        h.shell.route_key(c, true);
        h.run_tick(&[]);
        assert_eq!(cursor(&h), 0, "同拍多键取末个（-1：1→0），非逐个处理（会是 2→1）");
    }

    #[test]
    fn click_zones_in_screen_space() {
        assert_eq!(click_code(10, 300), -6);
        assert_eq!(click_code(230, 300), -7);
        assert_eq!(click_code(120, 300), -5);
        assert_eq!(click_code(10, 100), -5);
    }
}
