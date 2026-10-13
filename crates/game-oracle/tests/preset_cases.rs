//! preset 槽载入 + 短脚本场景用例集（spec p3-render §3.13）。
//!
//! 对拍窗口 = T240 起（槽载入完成后）。标题期粒子不可复现的根因：
//! a.java:2325 `setSeed(System.currentTimeMillis())` 的墙钟种子未被
//! ttransform 完全消除（构造器内的调用点在 A 变体仍指向真钟），故
//! 标题粒子帧（T71-239）Java 自身跨运行即不同（T73 sha 两次生成
//! 60015c67 vs 28e5a56a 实测）——Java 侧非确定性，非 Rust 缺陷；
//! 载入完成后的世界帧完全确定（槽载入链 + 菜单族短链 + 存档拍）。
//!
//! 迭代 21/22 定案：旧残差（T73/T77 粒子、T341 浮层）全部源于生成命令误用
//! _out/orig（原始字节真钟种子）——改用 variantA（ttransform 后）即确定性。
//! 迭代 24：preset_item13 追加（mode5 敌人图鉴）。
//! 迭代 25：T351/352 收敛——paint_menu5 八处移植笔误逐 op 对拍定位后修正，
//! T1-500 全帧一致。
//!
//! 路径：boot → 标题 → 载入进度（mode8）→ slot0 → mode2 步进链（含
//! step16 loadFloorSave）→ mode3 交 GameScene → 菜单族短链。
//! preset：`preset-floor1.preset`（MOT_L0 = floor1 (3,10) hp500/atk30/def30
//! 钥 1/0/0 金 100；MOT_IF = 槽表 slot0 有效）。

use game_core::menu_family::SlotSelect;
use game_core::render::{ArgbImage, SoftGraphics};
use game_core::scene::{GameScene, SceneImages};
use game_core::save::{parse_floor_save, parse_preset};
use game_core::sprite_spawn::{spawn_param_and_visibility, DoorTables};
use game_data::sprite::SpriteTable;
use game_oracle::trace;

pub fn repo() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn decode_png(bytes: &[u8]) -> ArgbImage {
    ArgbImage::from_decoded_png(game_data::decode_png(bytes).unwrap())
}

pub fn load_container(name: &str, count: usize) -> Vec<ArgbImage> {
    let data = std::fs::read(repo().join("assets/raw").join(name)).unwrap();
    game_data::PackedPng::parse(&data, count)
        .unwrap()
        .images
        .iter()
        .map(|sub| ArgbImage::from_decoded_png(game_data::decode_png(sub.bytes).unwrap()))
        .collect()
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

pub fn width_table() -> Vec<i32> {
    include_str!("../../game-core/tests/fixtures/type_width_px.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

pub fn tile_walkability() -> Vec<i32> {
    include_str!("../../game-core/tests/fixtures/tile_walkability.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

pub fn door_tables() -> DoorTables {
    DoorTables {
        door12: include_str!("../../game-core/tests/fixtures/door12.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
        door13: include_str!("../../game-core/tests/fixtures/door13.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
    }
}

fn input_key(inp: &Option<String>) -> i32 {
    if let Some(inp) = inp {
        if let Some(rest) = inp.strip_prefix("press(") {
            if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                return code;
            }
        }
    }
    0
}

#[test]
fn preset_load_menu_frames_match() {
    let repo_path = repo();
    let preset_text =
        std::fs::read_to_string(repo_path.join("reference/oracle/scenarios/preset-floor1.preset"))
            .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-preset-load-menu/trace.txt"),
    )
    .expect("缺 A-preset-load-menu trace（用 oracle host + -Doracle.preset 再生）");
    let records = trace::parse(&trace_text);

    // floor1 资产
    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite1).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let mut images_slot: Option<SceneImages> = None;
    let images_tmp = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    images_slot = Some(images_tmp);
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });

    // FloorSave（MOT_L0）
    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    // BootMachine + preset
    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);
    // rng 种子 = 0（默认）：trace 生成必须用 ttransform 后的 variantA
    //（a.java:2325 setSeed(VTime.currentTimeMillis()) → V(ctor)=0 确定；
    //  若误用 _out/orig 原始字节，种子=墙钟 ⇒ 标题粒子跨运行不可复现）

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    // 按键投递时序（同 boot_replay）：TICK n 的 INPUT 在 paint#n 后投递，
    // 由 logic#(n+1) 消费 ⇒ 延迟一拍。机阶段按值传 key；场景阶段
    // press/release 状态对（keyValue 与 keyHeldCode 分离，a.java:4659-4674）
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            // 槽载入完成（run case2 完成分支 → mode3）：交接 GameScene
            if machine.mode == 3 {
                if let Some(_cursor) = machine.slot_loaded {
                    let m = &machine;
                    let mut sc = game_core::scene::load_floor_from_save(
                        &maplv1,
                        &records1,
                        &width_table(),
                        &tile_walkability(),
                        images_slot.take().unwrap(),
                        floor_assets.clone(),
                        &save,
                    )
                    .unwrap();
                    sc.backdrop_scroll = m.backdrop.scroll();
                    if let Some(t) = &m.title {
                        sc.menu_cursor = t.cursor;
                        sc.bob_offset = t.bob;
                        sc.bob_rising = t.bob_rising;
                    }
                    sc.strip_frame = m.strip_frame;
                    sc.preset_mot_if = m
                        .preset_stores
                        .iter()
                        .find(|(k, _)| k.starts_with("MOT_IF@"))
                        .map(|(_, v)| v.clone());
                    if pending_key != 0 {
                        sc.press_key(pending_key);
                        pending_key = 0;
                    }
                    scene = Some(sc);
                }
            }
        }
        if rec.tick == 73 {
            if let Some(t) = &machine.title {
                eprintln!("title particles: {:?}", t.particles.dump_first());
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        if let Some(expect) = &rec.frame_sha {
            let mode = if let Some(sc) = &scene {
                sc.game_mode
            } else {
                machine.mode as i32
            };
            assert_eq!(
                &sha, expect,
                "TICK {} 帧不符（mode={}）",
                rec.tick, mode
            );
            checked += 1;
        }
    }
    assert_eq!(checked, records.len(), "T1-800 全帧必须比对");
}

/// 帮助链（mode15 帮助浮层）：preset-help 场景，窗口 T240-800。
/// 路径：菜单 kind5 帮助 → mode15 浮层全文（翻页/关闭）→ 返回 mode4。
#[test]
fn preset_help_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-floor1.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-preset-help/trace.txt"),
    )
    .expect("缺 A-preset-help trace（用 oracle host + -Doracle.preset 再生）");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite1).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 {
                if machine.slot_loaded.is_some() {
                    let m = &machine;
                    let mut sc = game_core::scene::load_floor_from_save(
                        &maplv1,
                        &records1,
                        &width_table(),
                        &tile_walkability(),
                        images.clone(),
                        floor_assets.clone(),
                        &save,
                    )
                    .unwrap();
                    sc.backdrop_scroll = m.backdrop.scroll();
                    if let Some(t) = &m.title {
                        sc.menu_cursor = t.cursor;
                        sc.bob_offset = t.bob;
                        sc.bob_rising = t.bob_rising;
                    }
                    sc.strip_frame = m.strip_frame;
                    sc.preset_mot_if = m
                        .preset_stores
                        .iter()
                        .find(|(k, _)| k.starts_with("MOT_IF@"))
                        .map(|(_, v)| v.clone());
                    if pending_key != 0 {
                        sc.press_key(pending_key);
                        pending_key = 0;
                    }
                    scene = Some(sc);
                }
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        if rec.tick >= 240 {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick, mode
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 561, "T240-800 应全比对");
}



/// 道具13使用链（mode10 → activateItem(13) → mode5 敌人图鉴）。
/// preset-item13.preset（MOT_L0 itemStack[0]=13×99）+ preset-item13.txt
/// （-6 道具面板 / -5 使用 → mode5 / -2/-1 翻页），窗口 T1-500 全帧。
/// 迭代 25 收敛：T351/352 残差 = paint_menu5 八处移植笔误（见 scene.rs 注释）。
#[test]
fn preset_item13_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-item13.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-preset-item13/trace.txt"),
    )
    .expect("缺 A-preset-item13 trace");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite1).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });

    let mot_l0 = &stores
        .iter()
        .find(|(k, _)| k.starts_with("MOT_L0@"))
        .expect("MOT_L0")
        .1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 && machine.slot_loaded.is_some() {
                let m = &machine;
                let mut sc = game_core::scene::load_floor_from_save(
                    &maplv1,
                    &records1,
                    &width_table(),
                    &tile_walkability(),
                    images.clone(),
                    floor_assets.clone(),
                    &save,
                )
                .unwrap();
                sc.backdrop_scroll = m.backdrop.scroll();
                if let Some(t) = &m.title {
                    sc.menu_cursor = t.cursor;
                    sc.bob_offset = t.bob;
                    sc.bob_rising = t.bob_rising;
                }
                sc.strip_frame = m.strip_frame;
                sc.preset_mot_if = m
                    .preset_stores
                    .iter()
                    .find(|(k, _)| k.starts_with("MOT_IF@"))
                    .map(|(_, v)| v.clone());
                let combat_tables = game_core::combat::CombatTables {
                        enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                };
                sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                // 则抖动序列错位（迭代 35 T801 分歧根因）
                sc.rng = m.rng.clone();
                if pending_key != 0 {
                    sc.press_key(pending_key);
                    pending_key = 0;
                }
                scene = Some(sc);
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        if let Some(expect) = &rec.frame_sha {
            let mode = if let Some(sc) = &scene {
                sc.game_mode
            } else {
                machine.mode as i32
            };
            assert_eq!(
                &sha, expect,
                "TICK {} 帧不符（mode={}）",
                rec.tick, mode
            );
            checked += 1;
        }
    }
    assert_eq!(checked, records.len(), "T1-500 全帧必须比对");
}
/// 物品栈消耗链（迭代 41，useItemStack a.java:7761 全语义）：preset-itemstack
/// 场景（MOT_L0 栈注入 [19 型×1 次]），T1-500 全帧。覆盖：mode10 面板 →
/// -5 确认浮层(kind3) → -6 使用 → activateItem(19) 加血 (atk+def)*74/10
/// （500→944）→ uses 1→0 → 栈压实（size 1→0）→ 空面板 → 返回行走。
#[test]
fn preset_itemstack_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-itemstack.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-preset-itemstack/trace.txt"),
    )
    .expect("缺 A-preset-itemstack trace");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite1).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });

    let mot_l0 = &stores
        .iter()
        .find(|(k, _)| k.starts_with("MOT_L0@"))
        .expect("MOT_L0")
        .1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 && machine.slot_loaded.is_some() {
                let m = &machine;
                let mut sc = game_core::scene::load_floor_from_save(
                    &maplv1,
                    &records1,
                    &width_table(),
                    &tile_walkability(),
                    images.clone(),
                    floor_assets.clone(),
                    &save,
                )
                .unwrap();
                sc.backdrop_scroll = m.backdrop.scroll();
                if let Some(t) = &m.title {
                    sc.menu_cursor = t.cursor;
                    sc.bob_offset = t.bob;
                    sc.bob_rising = t.bob_rising;
                }
                sc.strip_frame = m.strip_frame;
                sc.preset_mot_if = m
                    .preset_stores
                    .iter()
                    .find(|(k, _)| k.starts_with("MOT_IF@"))
                    .map(|(_, v)| v.clone());
                let combat_tables = game_core::combat::CombatTables {
                        enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                };
                sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                // 则抖动序列错位（迭代 35 T801 分歧根因）
                sc.rng = m.rng.clone();
                if pending_key != 0 {
                    sc.press_key(pending_key);
                    pending_key = 0;
                }
                scene = Some(sc);
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        if let Some(expect) = &rec.frame_sha {
            let mode = if let Some(sc) = &scene {
                sc.game_mode
            } else {
                machine.mode as i32
            };
            assert_eq!(
                &sha, expect,
                "TICK {} 帧不符（mode={}）",
                rec.tick, mode
            );
            checked += 1;
        }
    }
    assert_eq!(checked, records.len(), "T1-500 全帧必须比对");
}
/// 消耗臂 20 拦截路径（迭代 44，activateItem a.java:7832-7850）：preset-item2122
/// 场景（MOT_L0 栈注入 [20×1, 21×1]，floor1 (3,10)），T1-900 全帧。覆盖：
/// 20 对角瞬移的镜像格 interact 拦截 → 「无法移动到该位置」不消耗（栈保持
/// [20,21]）→ 确认浮层复现。21 臂的 changeFloor(+1) 需游标换位——独立场景
/// 批次（与按层铺合并）。
#[test]
fn preset_item2122_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-item2122.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-preset-item2122/trace.txt"),
    )
    .expect("缺 A-preset-item2122 trace");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite1).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });
    let records2: Vec<(u8, u16, u16, i16, bool)> = {
        let sp2 = SpriteTable::parse(
            &std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite2.bin")).unwrap(),
        )
        .unwrap();
        let doors = door_tables();
        sp2.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };
    floor_assets[2] = Some(game_core::scene::FloorAssets {
        maplv: std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv2.bin")).unwrap(),
        records: records2,
    });

    let mot_l0 = &stores
        .iter()
        .find(|(k, _)| k.starts_with("MOT_L0@"))
        .expect("MOT_L0")
        .1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 && machine.slot_loaded.is_some() {
                let m = &machine;
                let mut sc = game_core::scene::load_floor_from_save(
                    &maplv1,
                    &records1,
                    &width_table(),
                    &tile_walkability(),
                    images.clone(),
                    floor_assets.clone(),
                    &save,
                )
                .unwrap();
                sc.backdrop_scroll = m.backdrop.scroll();
                if let Some(t) = &m.title {
                    sc.menu_cursor = t.cursor;
                    sc.bob_offset = t.bob;
                    sc.bob_rising = t.bob_rising;
                }
                sc.strip_frame = m.strip_frame;
                sc.preset_mot_if = m
                    .preset_stores
                    .iter()
                    .find(|(k, _)| k.starts_with("MOT_IF@"))
                    .map(|(_, v)| v.clone());
                let combat_tables = game_core::combat::CombatTables {
                        enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                };
                sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                // 则抖动序列错位（迭代 35 T801 分歧根因）
                sc.rng = m.rng.clone();
                if pending_key != 0 {
                    sc.press_key(pending_key);
                    pending_key = 0;
                }
                scene = Some(sc);
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        if let Some(expect) = &rec.frame_sha {
            let mode = if let Some(sc) = &scene {
                sc.game_mode
            } else {
                machine.mode as i32
            };
            assert_eq!(
                &sha, expect,
                "TICK {} 帧不符（mode={}）",
                rec.tick, mode
            );
            checked += 1;
        }
    }
    assert_eq!(checked, records.len(), "T1-500 全帧必须比对");
}
/// 消耗臂 21 正路径（迭代 45，a.java:7851-7852）：preset-item21 场景
/// （MOT_L0 栈注入 [21×1]，floor1 (3,10)），T1-900 全帧。覆盖：面板 →
/// 确认浮层 → 使用 → changeFloor(+1) 遮幅换层 → floor2 落地 → 栈压实
/// 归零 → floor2 行走。21 臂 changeFloor 正路径与道具消耗压实同框。
/// 迭代 46 收口：根因 = 遮幅公共尾（a.java:3177-3215）只挂在 mode3
/// paint 路径 + changeFloor 未回 mode 3（A-preset-item21 T402 台账
/// gameMode=3 实证）——paint_wipe_tail 提取共跑 + change_floor 内
/// game_mode=3 后双缺口闭合。
#[test]
fn preset_item21_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-item21.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-preset-item21/trace.txt"),
    )
    .expect("缺 A-preset-item21 trace");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite1).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });
    let records2: Vec<(u8, u16, u16, i16, bool)> = {
        let sp2 = SpriteTable::parse(
            &std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite2.bin")).unwrap(),
        )
        .unwrap();
        let doors = door_tables();
        sp2.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };
    floor_assets[2] = Some(game_core::scene::FloorAssets {
        maplv: std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv2.bin")).unwrap(),
        records: records2,
    });

    let mot_l0 = &stores
        .iter()
        .find(|(k, _)| k.starts_with("MOT_L0@"))
        .expect("MOT_L0")
        .1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 && machine.slot_loaded.is_some() {
                let m = &machine;
                let mut sc = game_core::scene::load_floor_from_save(
                    &maplv1,
                    &records1,
                    &width_table(),
                    &tile_walkability(),
                    images.clone(),
                    floor_assets.clone(),
                    &save,
                )
                .unwrap();
                sc.backdrop_scroll = m.backdrop.scroll();
                if let Some(t) = &m.title {
                    sc.menu_cursor = t.cursor;
                    sc.bob_offset = t.bob;
                    sc.bob_rising = t.bob_rising;
                }
                sc.strip_frame = m.strip_frame;
                sc.preset_mot_if = m
                    .preset_stores
                    .iter()
                    .find(|(k, _)| k.starts_with("MOT_IF@"))
                    .map(|(_, v)| v.clone());
                let combat_tables = game_core::combat::CombatTables {
                        enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                };
                sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                // 则抖动序列错位（迭代 35 T801 分歧根因）
                sc.rng = m.rng.clone();
                if pending_key != 0 {
                    sc.press_key(pending_key);
                    pending_key = 0;
                }
                scene = Some(sc);
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        if let Some(expect) = &rec.frame_sha {
            let mode = if let Some(sc) = &scene {
                sc.game_mode
            } else {
                machine.mode as i32
            };
            assert_eq!(
                &sha, expect,
                "TICK {} 帧不符（mode={}）",
                rec.tick, mode
            );
            checked += 1;
        }
    }
    assert_eq!(checked, records.len(), "T1-500 全帧必须比对");
}
/// 钹消耗链（迭代 42，killAdjacentAt a.java:7923 全语义）：preset-itemcymbal
/// 场景（MOT_L0 栈注入 [86 型×1 次]，floor1 (3,10)），T1-600 全帧。覆盖：
/// mode10 面板 → 确认浮层 → 使用 → 四邻格 (2,10) 型43 怪清除 + 赏金 popup
/// （金 100→104，世界原点 (0,0)——Java 字面量）→ 栈压实 → -3 左行由阻挡
/// 变通行（激活前撞怪、激活后通达的强差分信号）。
#[test]
fn preset_itemcymbal_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-itemcymbal.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-preset-itemcymbal/trace.txt"),
    )
    .expect("缺 A-preset-itemcymbal trace");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite1).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });

    let mot_l0 = &stores
        .iter()
        .find(|(k, _)| k.starts_with("MOT_L0@"))
        .expect("MOT_L0")
        .1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 && machine.slot_loaded.is_some() {
                let m = &machine;
                let mut sc = game_core::scene::load_floor_from_save(
                    &maplv1,
                    &records1,
                    &width_table(),
                    &tile_walkability(),
                    images.clone(),
                    floor_assets.clone(),
                    &save,
                )
                .unwrap();
                sc.backdrop_scroll = m.backdrop.scroll();
                if let Some(t) = &m.title {
                    sc.menu_cursor = t.cursor;
                    sc.bob_offset = t.bob;
                    sc.bob_rising = t.bob_rising;
                }
                sc.strip_frame = m.strip_frame;
                sc.preset_mot_if = m
                    .preset_stores
                    .iter()
                    .find(|(k, _)| k.starts_with("MOT_IF@"))
                    .map(|(_, v)| v.clone());
                let combat_tables = game_core::combat::CombatTables {
                        enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                };
                sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                // 则抖动序列错位（迭代 35 T801 分歧根因）
                sc.rng = m.rng.clone();
                if pending_key != 0 {
                    sc.press_key(pending_key);
                    pending_key = 0;
                }
                scene = Some(sc);
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        if let Some(expect) = &rec.frame_sha {
            let mode = if let Some(sc) = &scene {
                sc.game_mode
            } else {
                machine.mode as i32
            };
            assert_eq!(
                &sha, expect,
                "TICK {} 帧不符（mode={}）",
                rec.tick, mode
            );
            checked += 1;
        }
    }
    assert_eq!(checked, records.len(), "T1-500 全帧必须比对");
}
/// 40 层 boss 击杀钩子（迭代 43，spawnBossEvent a.java:8903 + 击杀链
/// a.java:10053-10070）：preset-itemboss40 场景（槽载入 floor40 (6,3)，
/// atk200 面向型73 boss(6,2)），T1-700 全帧。覆盖：-1 撞入战斗（一拍斩
/// hp120）→ 击杀钩子（bossTypeOrder 未命中〔73 不在表〕→ case73 →
/// 脚本旗标 + loadLevelScript(22) + spawnBossEvent(6)（12 只 DES 态登场）→
/// battle_rerun_scene=false 收尾）+ 脚本 22 对话浮层。
/// 40 层 boss 击杀链（preset-itemboss40）：槽载入 (6,3) → -1 撞 boss(6,2)
/// → 一拍斩 → case73 脚本 + spawnBossEvent(6)（12 只 event-6 DES 登场）。
/// 迭代 55 收口（T1-700 全帧一致）：残差根因 = **对话气泡标记用错图集**
/// —— a.java:2887 标记图 = f_Image_arr2_00[12][f_byte_arr_22[dialogPage]]，
/// row 12 = **face 包**（resourceNames[12]="face"，12 张全 26×22，hexdump
/// 零余字节）；Rust 误用 `images.entity[bubble]`（实体 sprite 条带
/// 245×39）→ T303-306 每拍多画一条 245×39 条带（全屏 clip，x120 起）
/// ≈3037px 残差带。仪器 = FX-itemboss40 取证重跑（dumpStride=1）：
/// T303 双侧 op 流 872 vs 872，前 871 条逐字节全同，唯一分歧 op
/// `drawImage(26x22,120,68,0)` vs `drawImage(245x39,120,68,0)`。
/// 修复：SceneImages.faces 字段 + paint_dialog 改 face 表 + 全构造点装
/// load_container("face", 12)。
/// 【历史定性更正】以下旧定性全部证伪，读者可见错在哪：
/// - "T303 空 paint/0 ops"（迭代 49/50/54）= dumpStride=8 采样伪影
///   误读——非采样拍不落 OPS 块（run.py:185 硬编码），SRV/PAINT
///   700/700 实证 paint 每拍全量绘制；
/// - "Java paint 调度差异/画布残影"（迭代 54）同上证伪；
/// - "DES 登场泡 open_anim 帧位超前 ~2"（迭代 49/51）、"型26 火焰
///   发饰 sprite 差"（迭代 48）、"40 层相机 y 位差"（迭代 48，已
///   自行撤销）、"型73 boss 死亡动画拍差"（迭代 53）——皆是 245×39
///   错图上屏像素被误读为对方绘制差异；entityAnimFrame/popup 链/
///   击杀拍对表结论本身仍有效（那些机制两侧确实一致）。
#[test]
fn preset_itemboss40_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-itemboss40.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-preset-itemboss40/trace.txt"),
    )
    .expect("缺 A-preset-itemboss40 trace（gen: run_variants_parallel 单场景 frames=all）");
    let records = trace::parse(&trace_text);

    let maplv40 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv40.bin")).unwrap();
    let sprite40 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite40.bin")).unwrap();
    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite40).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    let records2: Vec<(u8, u16, u16, i16, bool)> = {
        let sp2 = SpriteTable::parse(&sprite1).unwrap();
        sp2.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[40] = Some(game_core::scene::FloorAssets {
        maplv: maplv40.clone(),
        records: records1.clone(),
    });
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records2.clone(),
    });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 {
                if machine.slot_loaded.is_some() {
                    let m = &machine;
                    let mut sc = game_core::scene::load_floor_from_save(
                        &maplv40,
                        &records1,
                        &width_table(),
                        &tile_walkability(),
                        images.clone(),
                        floor_assets.clone(),
                        &save,
                    )
                    .unwrap();
                    sc.backdrop_scroll = m.backdrop.scroll();
                    if let Some(t) = &m.title {
                        sc.menu_cursor = t.cursor;
                        sc.bob_offset = t.bob;
                        sc.bob_rising = t.bob_rising;
                    }
                    sc.strip_frame = m.strip_frame;
                    sc.preset_mot_if = m
                        .preset_stores
                        .iter()
                        .find(|(k, _)| k.starts_with("MOT_IF@"))
                        .map(|(_, v)| v.clone());
                    let combat_tables = game_core::combat::CombatTables {
                            enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    };
                    sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                    sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                    sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                    // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                    // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                    // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                    // 则抖动序列错位（迭代 35 T801 分歧根因）
                    sc.rng = m.rng.clone();
                    if pending_key != 0 {
                        sc.press_key(pending_key);
                        pending_key = 0;
                    }
                    scene = Some(sc);
                }
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick,
                    scene.as_ref().map(|s| s.game_mode).unwrap_or(-1)
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, records.len(), "T1-700 全帧必须比对");
}

/// 第 2 层直达（迭代 27，按层铺主线首层）：preset-floor2 场景（MOT_L0
/// current_floor=2），窗口 T1-600 全帧。覆盖：floor2 maplv/实体/拾取弹窗 +
/// 四方向走查 + mode10 道具面板开合。本层资产 = maplv2/sprite2（主层），
/// floor_assets[1] 同时装配（楼梯回 1 层可达）。
#[test]
fn preset_floor2_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-floor2.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-preset-floor2/trace.txt"),
    )
    .expect("缺 A-preset-floor2 trace（gen: run_variants_parallel 单场景 frames=all）");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let maplv2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv2.bin")).unwrap();
    let sprite2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite2.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite2).unwrap();
    let doors = door_tables();
    let records2: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    let records1: Vec<(u8, u16, u16, i16, bool)> = {
        let sp1 = SpriteTable::parse(&sprite1).unwrap();
        sp1.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });
    floor_assets[2] = Some(game_core::scene::FloorAssets {
        maplv: maplv2.clone(),
        records: records2.clone(),
    });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 {
                if machine.slot_loaded.is_some() {
                    let m = &machine;
                    let mut sc = game_core::scene::load_floor_from_save(
                        &maplv2,
                        &records2,
                        &width_table(),
                        &tile_walkability(),
                        images.clone(),
                        floor_assets.clone(),
                        &save,
                    )
                    .unwrap();
                    sc.backdrop_scroll = m.backdrop.scroll();
                    if let Some(t) = &m.title {
                        sc.menu_cursor = t.cursor;
                        sc.bob_offset = t.bob;
                        sc.bob_rising = t.bob_rising;
                    }
                    sc.strip_frame = m.strip_frame;
                    sc.preset_mot_if = m
                        .preset_stores
                        .iter()
                        .find(|(k, _)| k.starts_with("MOT_IF@"))
                        .map(|(_, v)| v.clone());
                    let combat_tables = game_core::combat::CombatTables {
                            enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    };
                    sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                    sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                    sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                    // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                    // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                    // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                    // 则抖动序列错位（迭代 35 T801 分歧根因）
                    sc.rng = m.rng.clone();
                    if pending_key != 0 {
                        sc.press_key(pending_key);
                        pending_key = 0;
                    }
                    scene = Some(sc);
                }
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick, mode
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, records.len(), "T1-600 全帧必须比对");
}
/// 商人链全流程（迭代 39，mode19 族 case16 收口）：preset-merchant 场景
/// （MOT_L0 直达第 2 层，type78 告示牌 @cell(11,7) param01=报价0），T1-600
/// 全帧。覆盖：右行撞牌 → interact case16 kind4 报价浮层（是/否图标）→
/// -6 applyMerchantOffer(true)（攻防各+3%、实体移除、浮层关）→ 告示牌
/// 消失验证。主层资产 = floor2（同 preset_floor2 形态）。
#[test]
fn preset_merchant_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-merchant.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-preset-merchant/trace.txt"),
    )
    .expect("缺 A-preset-merchant trace（gen: run_variants_parallel 单场景 frames=all）");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let maplv2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv2.bin")).unwrap();
    let sprite2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite2.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite2).unwrap();
    let doors = door_tables();
    let records2: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    let records1: Vec<(u8, u16, u16, i16, bool)> = {
        let sp1 = SpriteTable::parse(&sprite1).unwrap();
        sp1.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });
    floor_assets[2] = Some(game_core::scene::FloorAssets {
        maplv: maplv2.clone(),
        records: records2.clone(),
    });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 {
                if machine.slot_loaded.is_some() {
                    let m = &machine;
                    let mut sc = game_core::scene::load_floor_from_save(
                        &maplv2,
                        &records2,
                        &width_table(),
                        &tile_walkability(),
                        images.clone(),
                        floor_assets.clone(),
                        &save,
                    )
                    .unwrap();
                    sc.backdrop_scroll = m.backdrop.scroll();
                    if let Some(t) = &m.title {
                        sc.menu_cursor = t.cursor;
                        sc.bob_offset = t.bob;
                        sc.bob_rising = t.bob_rising;
                    }
                    sc.strip_frame = m.strip_frame;
                    sc.preset_mot_if = m
                        .preset_stores
                        .iter()
                        .find(|(k, _)| k.starts_with("MOT_IF@"))
                        .map(|(_, v)| v.clone());
                    let combat_tables = game_core::combat::CombatTables {
                            enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    };
                    sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                    sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                    sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                    // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                    // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                    // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                    // 则抖动序列错位（迭代 35 T801 分歧根因）
                    sc.rng = m.rng.clone();
                    if pending_key != 0 {
                        sc.press_key(pending_key);
                        pending_key = 0;
                    }
                    scene = Some(sc);
                }
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick, mode
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, records.len(), "T1-600 全帧必须比对");
}
/// floor1→floor2 换层全链（迭代 35，harness 统一）：floor1-to-floor2 场景
/// （preset-floor1 槽载入 (3,10)），T1-1500 全帧。覆盖：floor1 真实行走
/// （4 战）→ 踩型7楼梯(11,1) → changeFloor 遮幅 → floor2 落地 m_031 (1,2)
/// → 踩型8楼梯返回 → floor1 m_031 (10,1)。主层资产 = floor1；
/// floor_assets[1]/[2] 双层装配（换层互通）。战斗流首次纳入全帧对拍。
#[test]
fn floor_crossing_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-floor1.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-floor1-to-floor2/trace.txt"),
    )
    .expect("缺 A-floor1-to-floor2 trace（gen: run_variants_parallel 单场景 frames=all）");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let maplv2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv2.bin")).unwrap();
    let sprite2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite2.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite1).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    let records2: Vec<(u8, u16, u16, i16, bool)> = {
        let sp2 = SpriteTable::parse(&sprite2).unwrap();
        sp2.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });
    floor_assets[2] = Some(game_core::scene::FloorAssets {
        maplv: maplv2.clone(),
        records: records2.clone(),
    });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 {
                if machine.slot_loaded.is_some() {
                    let m = &machine;
                    let mut sc = game_core::scene::load_floor_from_save(
                        &maplv1,
                        &records1,
                        &width_table(),
                        &tile_walkability(),
                        images.clone(),
                        floor_assets.clone(),
                        &save,
                    )
                    .unwrap();
                    sc.backdrop_scroll = m.backdrop.scroll();
                    if let Some(t) = &m.title {
                        sc.menu_cursor = t.cursor;
                        sc.bob_offset = t.bob;
                        sc.bob_rising = t.bob_rising;
                    }
                    sc.strip_frame = m.strip_frame;
                    sc.preset_mot_if = m
                        .preset_stores
                        .iter()
                        .find(|(k, _)| k.starts_with("MOT_IF@"))
                        .map(|(_, v)| v.clone());
                    let combat_tables = game_core::combat::CombatTables {
                            enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    };
                    sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                    sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                    sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                    // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                    // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                    // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                    // 则抖动序列错位（迭代 35 T801 分歧根因）
                    sc.rng = m.rng.clone();
                    if pending_key != 0 {
                        sc.press_key(pending_key);
                        pending_key = 0;
                    }
                    scene = Some(sc);
                }
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick, mode
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, records.len(), "T1-1500 全帧必须比对");
}

/// 序章全流程（迭代 37，mode 11 Rust L3 全流程化）：prologue-dialog 场景
/// （boot 型，无 preset），T1-9800 全帧。覆盖：boot logo→声音询问→标题→
/// 新游戏引子（mode14 插画）→装载链（mode2）→ **新游戏冷启动移交**
/// （m_000 case 3：step8 统计 floor51/hp300/atk10/def10，a.java:3496-3506）
/// → 出生格 type83 触发 script 40 → mode 11 对话层（打字机/游标/翻页/
/// 电影镜头）至 T9800。取代旧冷场景版（gameplay_floor1.rs
/// prologue_dialog_frames_match T577-9800 手调初值窗口）。
#[test]
fn prologue_frames_match() {
    let repo_path = repo();
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-prologue-dialog/trace.txt"),
    )
    .expect("缺 A-prologue-dialog trace（先跑 python3 reference/oracle/run.py --scenarios）");
    let records = trace::parse(&trace_text);

    // floor51 冷启动资产（a.java:3485-3491：floor51、cell(1,11)=(32,352)）
    let maplv51 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv51.bin")).unwrap();
    let sprite51 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite51.bin")).unwrap();
    let sp51 = SpriteTable::parse(&sprite51).unwrap();
    let doors = door_tables();
    let records51: Vec<(u8, u16, u16, i16, bool)> = sp51
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    // 跳过教程（kind17）→ changeFloor(1)：层 1 资产备用
    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let records1: Vec<(u8, u16, u16, i16, bool)> = {
        let sp1 = SpriteTable::parse(&sprite1).unwrap();
        sp1.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[51] = Some(game_core::scene::FloorAssets {
        maplv: maplv51.clone(),
        records: records51.clone(),
    });
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });

    let mapbg2 = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg2[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.new_game_ready {
                // 冷启动场景（a.java:3496-3506 step8 统计 + m_001(3)/相机 step11）
                let mut sc = game_core::scene::GameScene::load_floor(
                    &maplv51,
                    32,
                    352,
                    &records51,
                    &width_table(),
                    &tile_walkability(),
                    images.clone(),
                    floor_assets.clone(),
                )
                .unwrap();
                sc.floor = 51;
                sc.min_floor = 51;
                sc.max_floor = 51;
                sc.player.hp = 300;
                sc.player.atk = 10;
                sc.player.def = 10;
                sc.enemies = game_core::combat::scale_enemy_stats(
                    &game_core::combat::CombatTables {
                        enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    },
                    1,
                );
                sc.enemy_base_hp = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                sc.build_minimap();
                sc.enter_view();
                // 连续量移交（Java 单实例语义）：bob/图标条帧/视差滚动/rng LCG 位置
                sc.bob_offset = machine.bob_offset;
                sc.bob_rising = machine.bob_rising;
                sc.strip_frame = machine.strip_frame;
                sc.backdrop_scroll = machine.backdrop.scroll();
                sc.rng = machine.rng.clone();
                // m_000 case 3 尾（a.java:4312-4318）：出生格 type83 → script 40
                sc.apply_entry_step_effects();
                if pending_key != 0 {
                    sc.press_key(pending_key);
                    pending_key = 0;
                }
                scene = Some(sc);
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick, mode
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, records.len(), "T1-9800 全帧必须比对");
}
/// floor51-menu 全流程（迭代 38，harness 统一收官）：floor51-menu 场景
/// （boot 型，序章场景 + mode10 大地图 UI 开关键），T1-9800 全帧。覆盖：
/// 序章全程（同 prologue 路径）+ 电影镜头等待态的 -6/-7 大地图 UI 开关。
/// 取代旧冷场景 #[ignore] 版（其 T9700-11099 窗口系旧版场景刻度，现场景
/// 已缩短为 9800 拍）。
#[test]
fn floor51_menu_frames_match() {
    let repo_path = repo();
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-floor51-menu/trace.txt"),
    )
    .expect("缺 A-floor51-menu trace（先跑 python3 reference/oracle/run.py --scenarios）");
    let records = trace::parse(&trace_text);

    // floor51 冷启动资产（a.java:3485-3491：floor51、cell(1,11)=(32,352)）
    let maplv51 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv51.bin")).unwrap();
    let sprite51 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite51.bin")).unwrap();
    let sp51 = SpriteTable::parse(&sprite51).unwrap();
    let doors = door_tables();
    let records51: Vec<(u8, u16, u16, i16, bool)> = sp51
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    // 跳过教程（kind17）→ changeFloor(1)：层 1 资产备用
    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let records1: Vec<(u8, u16, u16, i16, bool)> = {
        let sp1 = SpriteTable::parse(&sprite1).unwrap();
        sp1.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[51] = Some(game_core::scene::FloorAssets {
        maplv: maplv51.clone(),
        records: records51.clone(),
    });
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });

    let mapbg2 = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg2[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.new_game_ready {
                // 冷启动场景（a.java:3496-3506 step8 统计 + m_001(3)/相机 step11）
                let mut sc = game_core::scene::GameScene::load_floor(
                    &maplv51,
                    32,
                    352,
                    &records51,
                    &width_table(),
                    &tile_walkability(),
                    images.clone(),
                    floor_assets.clone(),
                )
                .unwrap();
                sc.floor = 51;
                sc.min_floor = 51;
                sc.max_floor = 51;
                sc.player.hp = 300;
                sc.player.atk = 10;
                sc.player.def = 10;
                sc.enemies = game_core::combat::scale_enemy_stats(
                    &game_core::combat::CombatTables {
                        enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    },
                    1,
                );
                sc.enemy_base_hp = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                sc.build_minimap();
                sc.enter_view();
                // 连续量移交（Java 单实例语义）：bob/图标条帧/视差滚动/rng LCG 位置
                sc.bob_offset = machine.bob_offset;
                sc.bob_rising = machine.bob_rising;
                sc.strip_frame = machine.strip_frame;
                sc.backdrop_scroll = machine.backdrop.scroll();
                sc.rng = machine.rng.clone();
                // m_000 case 3 尾（a.java:4312-4318）：出生格 type83 → script 40
                sc.apply_entry_step_effects();
                if pending_key != 0 {
                    sc.press_key(pending_key);
                    pending_key = 0;
                }
                scene = Some(sc);
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick, mode
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, records.len(), "T1-9800 全帧必须比对（floor51-menu）");
}
/// gameplay-floor1 全链（迭代 36，harness 统一 2/4）：gameplay-floor1 场景
/// （preset-floor1 槽载入 (3,10)），T1-3000 全帧。覆盖：mode3 自由玩法基线——
/// 行走子状态机/金币拾取(+1)/格子移动/实体动画与 bob/parallax 滚动（含战斗）。
/// 主层资产 = floor1；floor_assets[1]/[2] 双层装配。旧冷场景版
/// （gameplay_floor1.rs T537-1700 窗口）被本用例取代。
#[test]
fn gameplay_floor1_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-floor1.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-gameplay-floor1/trace.txt"),
    )
    .expect("缺 A-gameplay-floor1/trace.txt trace（gen: run_variants_parallel 单场景 frames=all）");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let maplv2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv2.bin")).unwrap();
    let sprite2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite2.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite1).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    let records2: Vec<(u8, u16, u16, i16, bool)> = {
        let sp2 = SpriteTable::parse(&sprite2).unwrap();
        sp2.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });
    floor_assets[2] = Some(game_core::scene::FloorAssets {
        maplv: maplv2.clone(),
        records: records2.clone(),
    });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 {
                if machine.slot_loaded.is_some() {
                    let m = &machine;
                    let mut sc = game_core::scene::load_floor_from_save(
                        &maplv1,
                        &records1,
                        &width_table(),
                        &tile_walkability(),
                        images.clone(),
                        floor_assets.clone(),
                        &save,
                    )
                    .unwrap();
                    sc.backdrop_scroll = m.backdrop.scroll();
                    if let Some(t) = &m.title {
                        sc.menu_cursor = t.cursor;
                        sc.bob_offset = t.bob;
                        sc.bob_rising = t.bob_rising;
                    }
                    sc.strip_frame = m.strip_frame;
                    sc.preset_mot_if = m
                        .preset_stores
                        .iter()
                        .find(|(k, _)| k.starts_with("MOT_IF@"))
                        .map(|(_, v)| v.clone());
                    let combat_tables = game_core::combat::CombatTables {
                            enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    };
                    sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                    sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                    sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                    // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                    // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                    // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                    // 则抖动序列错位（迭代 35 T801 分歧根因）
                    sc.rng = m.rng.clone();
                    if pending_key != 0 {
                        sc.press_key(pending_key);
                        pending_key = 0;
                    }
                    scene = Some(sc);
                }
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick, mode
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, records.len(), "T1-3000 全帧必须比对");
}

/// floor2 巡回全链（迭代 36，harness 统一 3/4）：floor2-tour 场景
/// （preset-floor1 槽载入 (3,10)），T1-2000 全帧。覆盖：floor1→floor2→
/// 宝箱组开门（walkPhase 4 遮幅抖动）+ 型11 道具浮层 + 血瓶拾取 +
/// 楼梯往返（floor2→1→2）+ 蓝门重复弹/关周期。主层资产 = floor1，
/// floor_assets[1]/[2] 双层装配。旧冷场景版（T537-1500 窗口）被本用例取代。
#[test]
fn floor2_tour_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-floor1.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-floor2-tour/trace.txt"),
    )
    .expect("缺 A-floor2-tour/trace.txt trace（gen: run_variants_parallel 单场景 frames=all）");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let maplv2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv2.bin")).unwrap();
    let sprite2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite2.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite1).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    let records2: Vec<(u8, u16, u16, i16, bool)> = {
        let sp2 = SpriteTable::parse(&sprite2).unwrap();
        sp2.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });
    floor_assets[2] = Some(game_core::scene::FloorAssets {
        maplv: maplv2.clone(),
        records: records2.clone(),
    });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 {
                if machine.slot_loaded.is_some() {
                    let m = &machine;
                    let mut sc = game_core::scene::load_floor_from_save(
                        &maplv1,
                        &records1,
                        &width_table(),
                        &tile_walkability(),
                        images.clone(),
                        floor_assets.clone(),
                        &save,
                    )
                    .unwrap();
                    sc.backdrop_scroll = m.backdrop.scroll();
                    if let Some(t) = &m.title {
                        sc.menu_cursor = t.cursor;
                        sc.bob_offset = t.bob;
                        sc.bob_rising = t.bob_rising;
                    }
                    sc.strip_frame = m.strip_frame;
                    sc.preset_mot_if = m
                        .preset_stores
                        .iter()
                        .find(|(k, _)| k.starts_with("MOT_IF@"))
                        .map(|(_, v)| v.clone());
                    let combat_tables = game_core::combat::CombatTables {
                            enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    };
                    sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                    sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                    sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                    // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                    // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                    // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                    // 则抖动序列错位（迭代 35 T801 分歧根因）
                    sc.rng = m.rng.clone();
                    if pending_key != 0 {
                        sc.press_key(pending_key);
                        pending_key = 0;
                    }
                    scene = Some(sc);
                }
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick, mode
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, records.len(), "T1-2000 全帧必须比对");
}
/// floor1 大巡回（迭代 40，按层铺深探测）：floor1-tour 场景（seed=7 固定
/// 走向），T1-8000 全帧——全项目最长 mode3 连续对拍。覆盖：大范围踩格
/// 探测战斗/门/钥匙/道具/楼梯事件全类型混合（金 100→692 拾取增量可见）。
/// 主层资产 = floor1；floor_assets[1]/[2] 双层装配。
#[test]
fn floor1_tour_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-floor1.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-floor1-tour/trace.txt"),
    )
    .expect("缺 A-floor1-tour/trace.txt trace（gen: run_variants_parallel 单场景 frames=all）");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let maplv2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv2.bin")).unwrap();
    let sprite2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite2.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite1).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    let records2: Vec<(u8, u16, u16, i16, bool)> = {
        let sp2 = SpriteTable::parse(&sprite2).unwrap();
        sp2.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });
    floor_assets[2] = Some(game_core::scene::FloorAssets {
        maplv: maplv2.clone(),
        records: records2.clone(),
    });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 {
                if machine.slot_loaded.is_some() {
                    let m = &machine;
                    let mut sc = game_core::scene::load_floor_from_save(
                        &maplv1,
                        &records1,
                        &width_table(),
                        &tile_walkability(),
                        images.clone(),
                        floor_assets.clone(),
                        &save,
                    )
                    .unwrap();
                    sc.backdrop_scroll = m.backdrop.scroll();
                    if let Some(t) = &m.title {
                        sc.menu_cursor = t.cursor;
                        sc.bob_offset = t.bob;
                        sc.bob_rising = t.bob_rising;
                    }
                    sc.strip_frame = m.strip_frame;
                    sc.preset_mot_if = m
                        .preset_stores
                        .iter()
                        .find(|(k, _)| k.starts_with("MOT_IF@"))
                        .map(|(_, v)| v.clone());
                    let combat_tables = game_core::combat::CombatTables {
                            enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    };
                    sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                    sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                    sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                    // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                    // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                    // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                    // 则抖动序列错位（迭代 35 T801 分歧根因）
                    sc.rng = m.rng.clone();
                    if pending_key != 0 {
                        sc.press_key(pending_key);
                        pending_key = 0;
                    }
                    scene = Some(sc);
                }
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick, mode
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, records.len(), "T1-8000 全帧必须比对");
}

/// 纯行走长程（迭代 40）：walk-pure 场景，T1-4000 全帧。覆盖：长距行走
/// 子状态机——相机边缘跟随全图巡回/parallax 滚动/实体动画相位长程一致性
/// （4 倍于 preset_floor2 的行走时长）。
#[test]
fn walk_pure_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-floor1.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-walk-pure/trace.txt"),
    )
    .expect("缺 A-walk-pure/trace.txt trace（gen: run_variants_parallel 单场景 frames=all）");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let maplv2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv2.bin")).unwrap();
    let sprite2 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite2.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite1).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    let records2: Vec<(u8, u16, u16, i16, bool)> = {
        let sp2 = SpriteTable::parse(&sprite2).unwrap();
        sp2.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });
    floor_assets[2] = Some(game_core::scene::FloorAssets {
        maplv: maplv2.clone(),
        records: records2.clone(),
    });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 {
                if machine.slot_loaded.is_some() {
                    let m = &machine;
                    let mut sc = game_core::scene::load_floor_from_save(
                        &maplv1,
                        &records1,
                        &width_table(),
                        &tile_walkability(),
                        images.clone(),
                        floor_assets.clone(),
                        &save,
                    )
                    .unwrap();
                    sc.backdrop_scroll = m.backdrop.scroll();
                    if let Some(t) = &m.title {
                        sc.menu_cursor = t.cursor;
                        sc.bob_offset = t.bob;
                        sc.bob_rising = t.bob_rising;
                    }
                    sc.strip_frame = m.strip_frame;
                    sc.preset_mot_if = m
                        .preset_stores
                        .iter()
                        .find(|(k, _)| k.starts_with("MOT_IF@"))
                        .map(|(_, v)| v.clone());
                    let combat_tables = game_core::combat::CombatTables {
                            enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    };
                    sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                    sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                    sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                    // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                    // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                    // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                    // 则抖动序列错位（迭代 35 T801 分歧根因）
                    sc.rng = m.rng.clone();
                    if pending_key != 0 {
                        sc.press_key(pending_key);
                        pending_key = 0;
                    }
                    scene = Some(sc);
                }
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick, mode
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, records.len(), "T1-4000 全帧必须比对");
}

/// 存档写回链（迭代 28）：preset-save 场景，窗口 T1-700 全帧。覆盖：
/// mode4 kind3 保存 → mode7 槽列表 → -5 确认写回（saveFloorState 序列化 +
/// markSlotEntry 槽 0 参效 + 「保存成功！」浮层）→ 关浮层 → 返回行走。
/// 迭代 27 教训落头：# preset: preset-floor1.preset 显式声明。
#[test]
fn preset_save_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-floor1.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-preset-save/trace.txt"),
    )
    .expect("缺 A-preset-save trace（gen: run_variants_parallel 单场景 frames=all）");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite1).unwrap();
    let doors = door_tables();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 {
                if machine.slot_loaded.is_some() {
                    let m = &machine;
                    let mut sc = game_core::scene::load_floor_from_save(
                        &maplv1,
                        &records1,
                        &width_table(),
                        &tile_walkability(),
                        images.clone(),
                        floor_assets.clone(),
                        &save,
                    )
                    .unwrap();
                    sc.backdrop_scroll = m.backdrop.scroll();
                    if let Some(t) = &m.title {
                        sc.menu_cursor = t.cursor;
                        sc.bob_offset = t.bob;
                        sc.bob_rising = t.bob_rising;
                    }
                    sc.strip_frame = m.strip_frame;
                    sc.preset_mot_if = m
                        .preset_stores
                        .iter()
                        .find(|(k, _)| k.starts_with("MOT_IF@"))
                        .map(|(_, v)| v.clone());
                    let combat_tables = game_core::combat::CombatTables {
                            enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    };
                    sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                    sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                    sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                    // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                    // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                    // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                    // 则抖动序列错位（迭代 35 T801 分歧根因）
                    sc.rng = m.rng.clone();
                    if pending_key != 0 {
                        sc.press_key(pending_key);
                        pending_key = 0;
                    }
                    scene = Some(sc);
                }
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
                        sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick, mode
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, records.len(), "T1-700 全帧必须比对");
}

/// mode12 记录浏览器（迭代 59，道具14 全链——按层铺 floor5 首覆盖）：
/// preset-floor5 场景（MOT_L0 current_floor=5 + MOT_L 日志槽预置
/// (kind0,idx2)），窗口 T1-1200 全帧。链：槽载入 (4,8) → -2 步入 (4,9)
/// 拾取道具14（首拾描述 overlay kind1 吞下一键）→ -6 面板 → -6 使用 →
/// activateItem case14 → mode12（m_000 case12 m_076(0) + paint case12
/// m_142 页码描边 + 日志 overlay kind5）→ -4/-3 m_076 钳位翻页 →
/// -6 关回 mode10。Java 锚：拾取 T266、面板 T501、浏览器 T781-912。
#[test]
fn preset_floor5_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-floor5.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-preset-floor5/trace.txt"),
    )
    .expect("缺 A-preset-floor5 trace（gen: run_variants_parallel 单场景 frames=all）");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let maplv5 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv5.bin")).unwrap();
    let sprite5 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite5.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite5).unwrap();
    let doors = door_tables();
    let records51: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    let records5: Vec<(u8, u16, u16, i16, bool)> = {
        let sp1 = SpriteTable::parse(&sprite1).unwrap();
        sp1.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };
    let records5: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[5] = Some(game_core::scene::FloorAssets {
        maplv: maplv5.clone(),
        records: records51.clone(),
    });
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records5.clone(),
    });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 {
                if machine.slot_loaded.is_some() {
                    let m = &machine;
                    let mut sc = game_core::scene::load_floor_from_save(
                        &maplv5,
                        &records51,
                        &width_table(),
                        &tile_walkability(),
                        images.clone(),
                        floor_assets.clone(),
                        &save,
                    )
                    .unwrap();
                    sc.backdrop_scroll = m.backdrop.scroll();
                    if let Some(t) = &m.title {
                        sc.menu_cursor = t.cursor;
                        sc.bob_offset = t.bob;
                        sc.bob_rising = t.bob_rising;
                    }
                    sc.strip_frame = m.strip_frame;
                    sc.preset_mot_if = m
                        .preset_stores
                        .iter()
                        .find(|(k, _)| k.starts_with("MOT_IF@"))
                        .map(|(_, v)| v.clone());
                    let combat_tables = game_core::combat::CombatTables {
                            enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    };
                    sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                    sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                    sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                    // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                    // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                    // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                    // 则抖动序列错位（迭代 35 T801 分歧根因）
                    sc.rng = m.rng.clone();
                    if pending_key != 0 {
                        sc.press_key(pending_key);
                        pending_key = 0;
                    }
                    scene = Some(sc);
                }
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            if (499..=505).contains(&(rec.tick as i32)) || (256..=272).contains(&(rec.tick as i32)) {
                if let Some(sc) = &scene {
                    eprintln!("DBG T{} py={} sp={} wp={} ov={}", rec.tick,
                        sc.view.player_py, sc.step_progress, sc.walk_phase, sc.overlay_active);
                }
            }
            let frame_snap = if (263..=270).contains(&(rec.tick as i32))
                || (499..=505).contains(&(rec.tick as i32))
                || (619..=625).contains(&(rec.tick as i32))
                || (748..=754).contains(&(rec.tick as i32))
                || (828..=834).contains(&(rec.tick as i32))
                || (908..=914).contains(&(rec.tick as i32))
                || (998..=1004).contains(&(rec.tick as i32))
            {
                Some(screen.argb.clone())
            } else {
                None
            };
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
            if let Some(snap) = frame_snap {
                let mut raw = Vec::new();
                for px in &snap {
                    raw.extend_from_slice(&px.to_be_bytes());
                }
                std::fs::write(format!("/tmp/f5-T{:04}.rgba", rec.tick), raw).unwrap();
            }
        }
        {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick, mode
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, records.len(), "T1-1200 全帧必须比对");
}

/// mode 9 炼丹链（迭代 31）：preset-alchemy 场景（floor12 type9 炼丹台
/// @cell(5,9)，(5,10) key2 向上踩台触发），窗口 T1-700 全帧。
/// 覆盖：type9 interact 分支 + run case9 买三属性 + paint case9 题头框。
#[test]
fn preset_alchemy_frames_match() {
    let repo_path = repo();
    let preset_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/scenarios/preset-alchemy.preset"),
    )
    .unwrap();
    let stores = parse_preset(&preset_text);
    let trace_text = std::fs::read_to_string(
        repo_path.join("reference/oracle/_out/A-preset-alchemy/trace.txt"),
    )
    .expect("缺 A-preset-alchemy trace（gen: run_variants_parallel 单场景 frames=all）");
    let records = trace::parse(&trace_text);

    let maplv1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite1 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let maplv12 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv12.bin")).unwrap();
    let sprite12 =
        std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite12.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite12).unwrap();
    let doors = door_tables();
    let records51: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    let records1: Vec<(u8, u16, u16, i16, bool)> = {
        let sp1 = SpriteTable::parse(&sprite1).unwrap();
        sp1.records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect()
    };
    let records1: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
        popup: game_core::popup::PopupImages {
            digit_strips: [map[3].clone(), map[4].clone(), map[5].clone()],
            icon_strip: map[6].clone(),
            battle_digits: ui[9].clone(),
        },
        spark_sheet: actor_c[2].clone(),
        corner_sprites: ui[23].clone(),
        map211: Some(map[11].clone()),
        map210: map[10].clone(),
        menu_strip: load_container("menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
    };
    let mut floor_assets: Vec<Option<game_core::scene::FloorAssets>> = vec![None; 56];
    floor_assets[12] = Some(game_core::scene::FloorAssets {
        maplv: maplv12.clone(),
        records: records51.clone(),
    });
    floor_assets[1] = Some(game_core::scene::FloorAssets {
        maplv: maplv1.clone(),
        records: records1.clone(),
    });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();

    let mapbg = load_container("mapbg", 1);
    let mut machine = game_core::paint::BootMachine::new(
        decode_png(&std::fs::read(repo_path.join("assets/raw/l0.png")).unwrap()),
        decode_png(&std::fs::read(repo_path.join("assets/raw/l1.png")).unwrap()),
        load_sflogo(),
        load_container("ui", 25),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        let key = pending_key;
        pending_key = input_key(&rec.input);
        if scene.is_some() {
            if pending_press != 0 {
                scene.as_mut().unwrap().press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                scene.as_mut().unwrap().release_key();
                pending_release = false;
            }
            scene
                .as_mut()
                .unwrap()
                .tick(rec.tick as i64 - 1, &width_table());
            if let Some(inp) = &rec.input {
                if let Some(rest) = inp.strip_prefix("press(") {
                    if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                        pending_press = code;
                    }
                } else if inp.starts_with("release(") {
                    pending_release = true;
                }
            }
        } else {
            machine.tick(key);
            if machine.mode == 3 {
                if machine.slot_loaded.is_some() {
                    let m = &machine;
                    let mut sc = game_core::scene::load_floor_from_save(
                        &maplv12,
                        &records51,
                        &width_table(),
                        &tile_walkability(),
                        images.clone(),
                        floor_assets.clone(),
                        &save,
                    )
                    .unwrap();
                    sc.backdrop_scroll = m.backdrop.scroll();
                    if let Some(t) = &m.title {
                        sc.menu_cursor = t.cursor;
                        sc.bob_offset = t.bob;
                        sc.bob_rising = t.bob_rising;
                    }
                    sc.strip_frame = m.strip_frame;
                    sc.preset_mot_if = m
                        .preset_stores
                        .iter()
                        .find(|(k, _)| k.starts_with("MOT_IF@"))
                        .map(|(_, v)| v.clone());
                    let combat_tables = game_core::combat::CombatTables {
                            enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                            difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    };
                    sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                    sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                    sc.enemy_base_gold = std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasegold.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect();
                    // rng 连续性（a.java:2324 单一 gameRandom 实例）：machine 阶段
                    // （标题雪花等）已消费的 LCG 位置必须带入场景——战斗抖动
                    // （a.java:6559-6560 randomBelow(5)）是场景首个消费者，若重置种子 0
                    // 则抖动序列错位（迭代 35 T801 分歧根因）
                    sc.rng = m.rng.clone();
                    if pending_key != 0 {
                        sc.press_key(pending_key);
                        pending_key = 0;
                    }
                    scene = Some(sc);
                }
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(game_core::paint::paint_font()));
            match scene.as_mut() {
                Some(sc) => sc.paint(&mut g, &tileset),
                None => machine.paint(&mut g),
            }
            if scene.is_some() && matches!(rec.tick, 321 | 441 | 581 | 584 | 640) {
                let sc = scene.as_ref().unwrap();
                println!(
                    "Rust@{}: mode={} cursor={} gold={} price={} upg={} hp={} atk={} def={}",
                    rec.tick, sc.game_mode, sc.alchemy_cursor, sc.player.gold,
                    sc.alchemy_price, sc.alchemy_upgrade_count, sc.player.hp,
                    sc.player.atk, sc.player.def
                );
            }
            if rec.tick == 581 {
                let mut buf = Vec::new();
                for &p in &screen.argb {
                    buf.extend_from_slice(&p.to_be_bytes());
                }
                std::fs::write("/tmp/rust-alch-581.bin", &buf).unwrap();
            }
                        sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        {
            if let Some(expect) = &rec.frame_sha {
                let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
                assert_eq!(
                    &sha, expect,
                    "TICK {} 帧不符（mode={}）",
                    rec.tick, mode
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, records.len(), "T1-700 全帧必须比对");
}
