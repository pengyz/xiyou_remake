//! game-desktop —— 桌面壳（窗口/输入/存档落盘/呈现）。让 game-core 真正可玩。
//!
//! 分层（AGENTS §2）：本 crate 只做平台实现；游戏逻辑全在 game-core，
//! 逐 tick 调用面与 oracle 差分 harness 完全同构（machine.tick/paint →
//! scene.press_key/release_key/tick/paint），因此差分语义在壳下原样成立。
//!
//! 用法：
//!   cargo run -p game-desktop                       # 资产取 ./assets/raw
//!   cargo run -p game-desktop -- --assets 路径      # 自定资产目录
//! 键位：方向键/WASD = 行走与菜单；回车/空格 = 确认(-5)；
//!       Q/F1 = 左软键(-6)；E/F2 = 右软键(-7)。
//! 存档：./save/ 目录（MOT_L0-2 = 槽存档、MOT_IF = 槽表、SKY_WAR = boss 旗标），
//!       与 BootMachine 的 RMS 预置同格式——游戏内「保存游戏」即落盘。

use std::path::PathBuf;

use game_core::layout::{SCREEN_H, SCREEN_W};
use game_core::paint::BootMachine;
use game_core::render::{ArgbImage, SoftGraphics};
use game_core::scene::{FloorAssets, GameScene};

const TICK_MS: u64 = 75;
const SCALE: usize = 2;
const W_U: usize = SCREEN_W as usize;
const H_U: usize = SCREEN_H as usize;

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
        let _ = FONT.set(load());
        game_core::render::set_glyph_hook(Some(hook_fn));
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
            outline.draw(|gx, gy, a| {
                if a > 0.5 {
                    put(gx as i32, gy as i32, color);
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
            let lit = img
                .argb
                .iter()
                .filter(|&&p| p & 0x00FF_FFFF != 0)
                .count();
            eprintln!("注入真字形后 mode21 文本像素 = {lit}");
            assert!(lit > 30, "真字形应画出 >30 像素，实得 {lit}（若为 0 说明 hook 未生效）");
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
    /// machine 一次性键（按下沿，下一逻辑 tick 消费）
    pending_machine_key: i32,
    frame_counter: u64,
    /// 已落盘的槽存档指纹（scene 侧保存检测）
    persisted_blobs: [Option<Vec<u8>>; 6],
}

impl Shell {
    fn combat_tables() -> game_core::combat::CombatTables {
        game_core::combat::CombatTables {
            enemy_base_hp: fixture!("enemybasehp.txt"),
            enemy_base_atk: fixture!("enemybaseatk.txt"),
            enemy_base_def: fixture!("enemybasedef.txt"),
            difficulty_multipliers: fixture!("difficulty_multipliers.txt"),
        }
    }

    fn logic_tick(&mut self) {
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
                {
                    let mut g = SoftGraphics::new(&mut self.screen);
                    g.set_clip(0, 0, SCREEN_W, SCREEN_H);
                    g.set_font(Some(game_core::paint::paint_font()));
                    sc.paint(&mut g, &self.assets.tileset);
                }
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
                                self.persisted_blobs = sc.saved_slot_blobs.clone();
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
                    sc.apply_entry_step_effects();
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

// ---------------------------------------------------------------- 窗口循环

fn run_window(mut shell: Shell) {
    use std::num::NonZeroU32;
    use winit::application::ApplicationHandler;
    use winit::event::{ElementState, WindowEvent};
    use winit::event_loop::{ActiveEventLoop, EventLoop};
    use winit::window::{Window, WindowId};

    type W = std::rc::Rc<Window>;

    struct Runner<'a> {
        shell: &'a mut Shell,
        window: Option<W>,
        context: Option<softbuffer::Context<W>>,
        surface: Option<softbuffer::Surface<W, W>>,
        next_tick: std::time::Instant,
        mouse: Option<(i32, i32)>,
    }

    impl ApplicationHandler for Runner<'_> {
        fn resumed(&mut self, el: &ActiveEventLoop) {
            let win = std::rc::Rc::new(el
                .create_window(
                    Window::default_attributes()
                        // 窗口标题保持 ASCII——系统标题栏字体常缺 CJK，会渲染成方框
                        .with_title("XiYou DX2 (Rust) | Arrows/WASD move, Enter ok, Q/E softkeys")
                        .with_inner_size(winit::dpi::LogicalSize::new(
                            (W_U * SCALE) as u32,
                            (H_U * SCALE) as u32,
                        ))
                        .with_resizable(false),
                )
                .unwrap());
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
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse = Some((position.x as i32, position.y as i32));
                }
                WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                    if button != winit::event::MouseButton::Left {
                        return;
                    }
                    // J2ME 触屏习惯：底部左角=左软键(-6)、底部右角=右软键(-7)、其余=确认(-5)
                    let Some((mx, my)) = self.mouse else { return };
                    let (w, h) = ((SCREEN_W as i32) * SCALE as i32, (SCREEN_H as i32) * SCALE as i32);
                    let code = if my > h * 7 / 10 && mx < w / 3 {
                        -6
                    } else if my > h * 7 / 10 && mx > w * 2 / 3 {
                        -7
                    } else {
                        -5
                    };
                    match self.shell.state.as_mut() {
                        Some(Screen::Game(sc)) => {
                            sc.press_key(code);
                            sc.release_key();
                        }
                        Some(Screen::Boot(_)) => self.shell.pending_machine_key = code,
                        None => {}
                    }
                    self.shell.logic_tick();
                    self.next_tick =
                        std::time::Instant::now() + std::time::Duration::from_millis(TICK_MS);
                    if let Some(w) = self.window.as_ref() {
                        w.request_redraw();
                    }
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.repeat {
                        return;
                    }
                    let Some(code) = key_code(event.physical_key) else {
                        return;
                    };
                    match event.state {
                        ElementState::Pressed => {
                            match self.shell.state.as_mut() {
                                Some(Screen::Game(sc)) => sc.press_key(code),
                                Some(Screen::Boot(_)) => self.shell.pending_machine_key = code,
                                None => {}
                            }
                            self.shell.logic_tick();
                            self.next_tick =
                                std::time::Instant::now() + std::time::Duration::from_millis(TICK_MS);
                            if let Some(w) = self.window.as_ref() {
                                w.request_redraw();
                            }
                        }
                        ElementState::Released => {
                            if let Some(Screen::Game(sc)) = self.shell.state.as_mut() {
                                sc.release_key();
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
            let now = std::time::Instant::now();
            let mut ran = false;
            while now >= self.next_tick {
                self.shell.logic_tick();
                ran = true;
                self.next_tick += std::time::Duration::from_millis(TICK_MS);
            }
            if ran {
                if let Some(w) = self.window.as_ref() {
                    w.request_redraw();
                }
            }
        }
    }

    impl Runner<'_> {
        fn present(&mut self) {
            if self.window.is_none() {
                return;
            }
            let Some(surface) = self.surface.as_mut() else { return };
            let w = (W_U * SCALE) as u32;
            let h = (H_U * SCALE) as u32;
            surface.resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap()).ok();
            let mut buffer = surface.buffer_mut().unwrap();
            let pixels: &mut [u32] = &mut buffer;
            for y in 0..H_U {
                for x in 0..W_U {
                    let rgb = 0xFF00_0000u32 | (self.shell.screen.argb[y * W_U + x] & 0x00FF_FFFF);
                    for sy in 0..SCALE {
                        let row = (y * SCALE + sy) * w as usize;
                        for sx in 0..SCALE {
                            pixels[row + x * SCALE + sx] = rgb;
                        }
                    }
                }
            }
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
    let mut runner = Runner {
        shell: &mut shell,
        window: None,
        context: None,
        surface: None,
        next_tick: std::time::Instant::now() + std::time::Duration::from_millis(TICK_MS),
        mouse: None,
    };
    event_loop.run_app(&mut runner).unwrap();
}

fn main() {
    let mut assets_dir = PathBuf::from("assets/raw");
    let mut save_dir = PathBuf::from("save");
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--assets" {
            assets_dir = PathBuf::from(args.next().unwrap_or_default());
        } else if a == "--save" {
            save_dir = PathBuf::from(args.next().unwrap_or_default());
        }
    }
    println!("资产目录 = {assets_dir:?}；存档目录 = {save_dir:?}");
    font_inject::install();
    let assets = std::sync::Arc::new(load_assets(&assets_dir));
    let (rms, stores) = Rms::load(&save_dir);

    // BootMachine 装配（与测试 harness 同参）
    let mapbg = load_container(&assets_dir, "mapbg", 1);
    let mut machine = BootMachine::new(
        ArgbImage::from_decoded_png(
            game_data::decode_png(&std::fs::read(assets_dir.join("l0.png")).expect("缺 assets/raw/l0.png"))
                .unwrap(),
        ),
        ArgbImage::from_decoded_png(
            game_data::decode_png(&std::fs::read(assets_dir.join("l1.png")).expect("缺 assets/raw/l1.png"))
                .unwrap(),
        ),
        load_container(&assets_dir, "sflogo", 8),
        load_container(&assets_dir, "ui", 25),
        load_container(&assets_dir, "menu", 2),
        mapbg[0].clone(),
        load_container(&assets_dir, "intro", 2),
        load_container(&assets_dir, "end", 1),
        load_container(&assets_dir, "load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container(&assets_dir, "sptarm", 10);

    let shell = Shell {
        assets,
        rms,
        stores,
        screen: ArgbImage::create(SCREEN_W, SCREEN_H),
        state: Some(Screen::Boot(machine)),
        pending_machine_key: 0,
        frame_counter: 0,
        persisted_blobs: [None, None, None, None, None, None],
    };
    run_window(shell);
}
