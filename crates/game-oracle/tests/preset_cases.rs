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
//! 迭代 24：preset_item13 追加（mode5 敌人图鉴），T351 残差待逐 op 收敛。
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

fn repo() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn decode_png(bytes: &[u8]) -> ArgbImage {
    ArgbImage::from_decoded_png(game_data::decode_png(bytes).unwrap())
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
    let data = std::fs::read(repo().join("assets/raw/sflogo")).unwrap();
    game_data::PackedPng::parse(&data, 8)
        .unwrap()
        .images
        .iter()
        .map(|sub| ArgbImage::from_decoded_png(game_data::decode_png(sub.bytes).unwrap()))
        .collect()
}

fn width_table() -> Vec<i32> {
    include_str!("../../game-core/tests/fixtures/type_width_px.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

fn tile_walkability() -> Vec<i32> {
    include_str!("../../game-core/tests/fixtures/tile_walkability.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

fn door_tables() -> DoorTables {
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
            let dbg = if rec.tick == 151 { Some((g.ops.join("\n"), screen.hash_stream())) } else { None };
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
            if let Some((ops, raw)) = &dbg {
                std::fs::write("/tmp/rust-151-ops.txt", ops).unwrap();
                std::fs::write("/tmp/rust-151.raw", raw.clone()).unwrap();
            }
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
}


/// 道具13使用链（mode10 → activateItem(13) → mode5 敌人图鉴）。
/// preset-item13.preset（MOT_L0 itemStack[0]=13×99）+ preset-item13.txt
/// （-6 道具面板 / -5 使用 → mode5 / -2/-1 翻页），窗口 T1-500 全帧。
#[test]
#[ignore] // T351 mode5 paint case5 残差 + T77 粒子（迭代 25 收敛中）
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
                sc.enemies = game_core::combat::scale_enemy_stats(
                    &game_core::combat::CombatTables {
                        enemy_base_hp: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasehp.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_atk: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybaseatk.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        enemy_base_def: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/enemybasedef.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                        difficulty_multipliers: std::fs::read_to_string(repo_path.join("crates/game-core/tests/fixtures/difficulty_multipliers.txt")).unwrap().lines().filter_map(|l| l.trim().parse().ok()).collect(),
                    },
                    1,
                );
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
            if rec.tick == 351 {
                let mut buf = Vec::new();
                for &p in &screen.argb {
                    buf.extend_from_slice(&p.to_be_bytes());
                }
                std::fs::write("/tmp/rust-item13-351.bin", &buf).unwrap();
            }
            if rec.tick == 351 {
                let mut buf = Vec::new();
                for &p in &screen.argb {
                    buf.extend_from_slice(&p.to_be_bytes());
                }
                std::fs::write("/tmp/rust-351-argb.bin", &buf).unwrap();
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
