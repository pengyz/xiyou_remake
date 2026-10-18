//! 存档槽装备框场景级对拍（completeness 2026-10-15 新发现②）：Rust 全帧 vs Java A 变体帧。
//!
//! 覆盖 paintSlotTable 装备框段（a.java:9104-9143）两条移植 × 两分支：
//! - boot mode 8（`SlotSelect::paint`）：MOT_IF 槽0 79/80（双 `>7` 钳位图标）、槽1 0/0
//!   （材质名“无”）、槽2 34/38（直取图标）、槽3 0/41（合成值：甲 i=8 是 `>7⇒9` 与旧误植
//!   `>9⇒8` 的唯一分歧点）、槽4 无效——下扫/上扫逐帧比对；
//! - scene mode 7（`GameScene::paint_slot_list`）：同表下扫 + 存档写回（markSlotEntry
//!   a.java:9013-9014 拷当前装备：拾 80 甲后存入空槽4 ⇒ “无” + 甲图标）。
//!
//! 夹具（crates/game-core/tests/fixtures/）：
//! - `preset-save-equip.preset`：RMS 预置（MOT_IF 新设计；MOT_L0/SKY_WAR 逐字取自
//!   `reference/oracle/scenarios/preset-floor43.preset`，装备 0/0——载入侧装备读回
//!   〔a.java:9260-9261〕Rust 未移植，另行登记，本场景不触发）；
//! - `preset-save-equip.txt`：输入脚本（带 `# ticks:`/`# preset:` 头，可原样并入
//!   `reference/oracle/scenarios/` 场景套件）；
//! - `preset-save-equip.frames.txt`：Java 参照帧（逐 TICK `tick sha [INPUT]`），由 A 变体
//!   trace 抽取——期望值嵌入夹具而非运行期读 `_out/`（provision 只认 scenarios/ 下的
//!   `A-<stem>`，见 docs/knowledge/gotcha_provision-trace-labels-a-stem-only.md）。
//!
//! 再生（无头；产物落 reference/oracle/_out/，gitignore）：
//! ```text
//! python3 - <<'EOF'
//! import importlib.util, re; from pathlib import Path
//! s = importlib.util.spec_from_file_location("r", "reference/oracle/run.py")
//! run = importlib.util.module_from_spec(s); s.loader.exec_module(run)
//! scen = Path("crates/game-core/tests/fixtures/preset-save-equip.txt").resolve()
//! ticks = int(re.search(r"^#\s*ticks:\s*(\d+)", scen.read_text(), re.M).group(1))
//! cls = run.build_shim_host(); orig = run.ensure_game_classes()
//! vc = run.find_variant_c(None); run.ttransform_variants(orig, vc)
//! run.run_variants_parallel([(cls, run.OUT / f"variant{v}", f"{v}-preset-save-equip", ticks,
//!     scen, str(scen.with_suffix(".preset"))) for v in "ABC"], frames="off", dump_stride=8)
//! EOF
//! ```
//! 三方 trace sha256 须一致；再按 TICK 抽 `FRAME sha=` 与 `INPUT` 行写回 frames 夹具。
//!
//! 资产：assets/raw（门禁 provision 自供给，tools/extract-assets 由 original/ 再生）。

use game_core::enums::GameMode;
use game_core::paint::BootMachine;
use game_core::render::{ArgbImage, SoftGraphics};
use game_core::save::{parse_floor_save, parse_preset};
use game_core::scene::{FloorAssets, GameScene, SceneImages};
use game_core::sprite_spawn::{spawn_param_and_visibility, DoorTables};
use game_data::sprite::SpriteTable;

fn repo() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn raw(name: &str) -> Vec<u8> {
    let p = repo().join("assets/raw").join(name);
    std::fs::read(&p).unwrap_or_else(|e| {
        panic!("缺 {}（{e}）：先跑 python3 reference/oracle/provision_tests.py", p.display())
    })
}

fn decode_png(bytes: &[u8]) -> ArgbImage {
    ArgbImage::from_decoded_png(game_data::decode_png(bytes).unwrap())
}

fn load_container(name: &str, count: usize) -> Vec<ArgbImage> {
    let data = raw(name);
    game_data::PackedPng::parse(&data, count)
        .unwrap()
        .images
        .iter()
        .map(|sub| decode_png(sub.bytes))
        .collect()
}

fn numbers<T: std::str::FromStr>(text: &str) -> Vec<T> {
    text.lines().filter_map(|l| l.trim().parse().ok()).collect()
}

/// 帧夹具一行：(tick, Java FRAME sha, INPUT)。
struct FrameRec {
    tick: u32,
    sha: String,
    input: Option<String>,
}

fn frame_records() -> Vec<FrameRec> {
    include_str!("fixtures/preset-save-equip.frames.txt")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let mut it = l.split_whitespace();
            FrameRec {
                tick: it.next().unwrap().parse().unwrap(),
                sha: it.next().unwrap().to_string(),
                input: it.next().map(str::to_string),
            }
        })
        .collect()
}

fn press_code(inp: &Option<String>) -> i32 {
    inp.as_deref()
        .and_then(|s| s.strip_prefix("press("))
        .and_then(|s| s.strip_suffix(')'))
        .and_then(|k| k.parse().ok())
        .unwrap_or(0)
}

/// 夹具自洽：帧夹具的 INPUT 列与输入脚本逐事件一致（脚本 → Runner 投递 → trace）。
#[test]
fn preset_save_equip_fixture_inputs_match_script() {
    let script: Vec<String> = include_str!("fixtures/preset-save-equip.txt")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let p: Vec<&str> = l.split_whitespace().collect();
            format!("{} {}({})", p[0].parse::<u32>().unwrap(), p[2], p[1])
        })
        .collect();
    let recs = frame_records();
    let from_trace: Vec<String> = recs
        .iter()
        .filter_map(|r| r.input.as_ref().map(|i| format!("{} {i}", r.tick)))
        .collect();
    assert_eq!(from_trace, script);
    assert_eq!(recs.len(), 1040, "# ticks: 1040");
    assert!(recs.iter().enumerate().all(|(i, r)| r.tick as usize == i + 1), "TICK 连续");
}

/// T1-1040 全帧一致 + 两移植 × 两分支均实际进入比对（覆盖计数断言）。
#[test]
fn preset_save_equip_frames_match() {
    let preset_text = include_str!("fixtures/preset-save-equip.preset");
    let stores = parse_preset(preset_text);
    let records = frame_records();

    let width_table: Vec<i32> = numbers(include_str!("fixtures/type_width_px.txt"));
    let walkability: Vec<i32> = numbers(include_str!("fixtures/tile_walkability.txt"));
    let doors = DoorTables {
        door12: numbers(include_str!("fixtures/door12.txt")),
        door13: numbers(include_str!("fixtures/door13.txt")),
    };
    let maplv43 = raw("maplv43");
    let sp = SpriteTable::parse(&raw("sprite43")).unwrap();
    let records43: Vec<(u8, u16, u16, i16, bool)> = sp
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
        ui: ui.clone(),
        faces: load_container("face", 12),
        ending: load_container("end", 1)[0].clone(),
        xtq: load_container("xtq", 6),
    };
    let mut floor_assets: Vec<Option<FloorAssets>> = vec![None; 56];
    floor_assets[43] = Some(FloorAssets { maplv: maplv43.clone(), records: records43.clone() });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();
    let combat_tables = game_core::combat::CombatTables {
        enemy_base_hp: numbers(include_str!("fixtures/enemybasehp.txt")),
        enemy_base_atk: numbers(include_str!("fixtures/enemybaseatk.txt")),
        enemy_base_def: numbers(include_str!("fixtures/enemybasedef.txt")),
        difficulty_multipliers: numbers(include_str!("fixtures/difficulty_multipliers.txt")),
    };

    let mut machine = BootMachine::new(
        decode_png(&raw("l0.png")),
        decode_png(&raw("l1.png")),
        load_container("sflogo", 8),
        ui.clone(),
        load_container("menu", 2),
        mapbg[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = sptarm.clone();

    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    // 覆盖计数：(boot 图标, boot 材质名, 场景图标, 场景材质名) 帧数
    let mut cover = [0usize; 4];
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    // 驱动时序与 game-oracle preset_cases 同构（INPUT 在 paint 后投递 ⇒ 下一拍逻辑消费）
    for rec in &records {
        let key = pending_key;
        pending_key = press_code(&rec.input);
        if let Some(sc) = scene.as_mut() {
            if pending_press != 0 {
                sc.press_key(pending_press);
                pending_press = 0;
            }
            if pending_release {
                sc.release_key();
                pending_release = false;
            }
            sc.tick(rec.tick as i64 - 1, &width_table);
            match rec.input.as_deref() {
                Some(i) if i.starts_with("press(") => pending_press = press_code(&rec.input),
                Some(i) if i.starts_with("release(") => pending_release = true,
                _ => {}
            }
        } else {
            machine.tick(key);
            if machine.mode == GameMode::Map && machine.slot_loaded.is_some() {
                let m = &machine;
                let mut sc = game_core::scene::load_floor_from_save(
                    &maplv43,
                    &records43,
                    &width_table,
                    &walkability,
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
                sc.enemies = game_core::combat::scale_enemy_stats(&combat_tables, 1);
                sc.enemy_base_hp = combat_tables.enemy_base_hp.clone();
                sc.enemy_base_gold = numbers(include_str!("fixtures/enemybasegold.txt"));
                // rng 续接 machine LCG（docs/knowledge/gotcha_gamescene-rng-must-continue-machine-lcg.md）
                sc.rng = m.rng.clone();
                if pending_key != 0 {
                    sc.press_key(pending_key);
                    pending_key = 0;
                }
                scene = Some(sc);
            }
        }
        // 覆盖登记：本拍将画的详情面板装备框分支（a.java:9107/9127 条件）
        let panel = match scene.as_ref() {
            Some(sc) if sc.game_mode == GameMode::SaveSlot => Some((2usize, &sc.slot)),
            Some(_) => None,
            None if machine.mode == GameMode::EquipDetail => machine.slots.as_ref().map(|s| (0usize, s)),
            None => None,
        };
        if let Some((base, s)) = panel {
            let c = s.cursor as usize;
            if s.valid[c] {
                for v in [s.weapon[c], s.armor[c]] {
                    cover[base + (v == 0) as usize] += 1;
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
        let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode);
        assert_eq!(sha, rec.sha, "TICK {} 帧不符（mode={}）", rec.tick, mode);
        checked += 1;
    }
    assert_eq!(checked, 1040, "T1-1040 全帧必须比对");
    let [boot_icon, boot_name, scene_icon, scene_name] = cover;
    assert!(
        boot_icon > 0 && boot_name > 0 && scene_icon > 0 && scene_name > 0,
        "两移植 × 两分支须全部进入比对: boot 图标/名={boot_icon}/{boot_name} 场景 图标/名={scene_icon}/{scene_name}"
    );
    // 存档写回终态（Java T801 = press@T800 次拍 markSlotEntry(4)：f_byte_arr_25/26[4] = 0/80）
    let sc = scene.expect("≤T416 应已交场景（Java gameMode=3 首见 T416 采样）");
    assert_eq!(sc.game_mode, GameMode::Map);
    assert_eq!((sc.equipped_weapon, sc.equipped_armor), (0, 80));
    assert!(sc.slot.valid[4]);
    assert_eq!((sc.slot.weapon[4], sc.slot.armor[4]), (0, 80));
}
