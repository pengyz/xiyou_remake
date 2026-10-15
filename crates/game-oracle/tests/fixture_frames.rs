//! 帧夹具对拍（fixture-frames，2026-10-15 判据①收口）：Rust 全帧 vs Java A 变体帧，期望帧
//! 嵌入夹具，**运行期不读 `reference/oracle/_out/`**（同 crates/game-core/tests/slot_equip_scene.rs；
//! 理由见 docs/knowledge/gotcha_provision-trace-labels-a-stem-only.md）。
//!
//! 覆盖矩阵（docs/coverage-matrix.md §3 黄项）——tools/cover_matrix.py 以夹具注释头的 `A-<stem>`
//! 认领场景，且夹具帧链 sha256 = tools/method_reach 快照该场景 frame_chain 才把窗口计入验证：
//! - `preset-floor5`（scenarios/，迭代 59 原用例的夹具化）：m_076（现名 browseOfferQueue，mode12 记录翻页）、
//!   addItemToItemStack（拾取道具14）；
//! - `preset-floor49`（scenarios/，迭代 66 原用例的夹具化）：m_102（现名 probeCrossFloorSnapshot，tryRunScene
//!   case15 守卫门控）；
//! - `fx-scriptwalk-floor3`（夹具场景）：advanceScriptWalk（script0 玩家三参 MOV 两步行走）；
//! - `fx-knockback-floor43`（夹具场景）：m_101（型60 上邻击退：可退 / 墙挡两臂）；
//! - `fx-fan-floor13`（夹具场景）：m_089（芭蕉扇未命中提示 / 命中灭火两臂）+ 型10 撞击提示。
//!
//! 夹具（crates/game-oracle/tests/fixtures/）与再生命令（无头；不触碰 _out/ 的写入只发生在
//! `gen` 的隔离工作区 analysis/frames-fixture/）：
//! ```text
//! python3 tools/frames_fixture.py extract reference/oracle/_out/A-preset-floor5/trace.txt \
//!     --stem preset-floor5 --out crates/game-oracle/tests/fixtures/preset-floor5.frames.txt \
//!     --script reference/oracle/scenarios/preset-floor5.txt          # floor49 同式
//! python3 tools/frames_fixture.py gen crates/game-oracle/tests/fixtures/fx-scriptwalk-floor3.txt
//! python3 tools/frames_fixture.py gen crates/game-oracle/tests/fixtures/fx-knockback-floor43.txt
//! python3 tools/frames_fixture.py gen crates/game-oracle/tests/fixtures/fx-fan-floor13.txt
//! ```
//! 资产：assets/raw（门禁 provision 自供给，tools/extract-assets 由 original/ 再生）；55 层全量
//! 装载（与壳 crates/game-desktop 同构——换层/跨层脚本可用任意层数据）。

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

fn frame_records(text: &str) -> Vec<FrameRec> {
    text.lines()
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

/// 夹具自洽：INPUT 列 = 场景脚本事件（脚本 → Runner 投递 → trace），TICK 连续且帧数 = `# ticks:`。
fn assert_fixture_matches_script(recs: &[FrameRec], script: &str) {
    let events: Vec<String> = script
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let p: Vec<&str> = l.split_whitespace().collect();
            format!("{} {}({})", p[0].parse::<u32>().unwrap(), p[2], p[1])
        })
        .collect();
    let from_trace: Vec<String> = recs
        .iter()
        .filter_map(|r| r.input.as_ref().map(|i| format!("{} {i}", r.tick)))
        .collect();
    assert_eq!(from_trace, events, "夹具 INPUT 列与场景脚本不一致");
    let ticks: usize = script
        .lines()
        .find_map(|l| l.strip_prefix("# ticks:"))
        .and_then(|v| v.trim().parse().ok())
        .expect("脚本缺 `# ticks:` 头");
    assert_eq!(recs.len(), ticks, "帧数 = # ticks:");
    assert!(recs.iter().enumerate().all(|(i, r)| r.tick as usize == i + 1), "TICK 连续");
}

/// 场景图像（与 preset_cases / slot_equip_scene 同构）。
fn scene_images() -> SceneImages {
    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    SceneImages {
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
        xtq: load_container("xtq", 6),
    }
}

/// 1-54 层 maplv/sprite 全量（sprite 记录 → (型, x, y, param, 可见)，与壳同式）。
fn all_floors() -> Vec<Option<FloorAssets>> {
    let doors = DoorTables {
        door12: numbers(include_str!("../../game-core/tests/fixtures/door12.txt")),
        door13: numbers(include_str!("../../game-core/tests/fixtures/door13.txt")),
    };
    let mut floors: Vec<Option<FloorAssets>> = vec![None; 56];
    for (n, slot) in floors.iter_mut().enumerate().take(55).skip(1) {
        let sp = SpriteTable::parse(&raw(&format!("sprite{n}"))).unwrap();
        let records = sp
            .records
            .iter()
            .map(|rec| {
                let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
                (rec.type_code, rec.x, rec.y, param, !hide)
            })
            .collect();
        *slot = Some(FloorAssets { maplv: raw(&format!("maplv{n}")), records });
    }
    floors
}

/// 逐拍重放：boot（标题→读档菜单→槽0）→ mode3 交 GameScene → 场景输入；每拍 paint 后与夹具帧
/// 比对。驱动时序与 preset_cases 同构（INPUT 在 paint 后投递 ⇒ 下一拍逻辑消费）。
/// 返回 (比对帧数, 终态场景)。
fn replay(preset_text: &str, records: &[FrameRec]) -> (usize, GameScene) {
    let stores = parse_preset(preset_text);
    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();
    let floors = all_floors();
    let cur = floors[save.current_floor as usize].clone().expect("存档层资产");
    let width_table: Vec<i32> = numbers(include_str!("../../game-core/tests/fixtures/type_width_px.txt"));
    let walkability: Vec<i32> = numbers(include_str!("../../game-core/tests/fixtures/tile_walkability.txt"));
    let combat_tables = game_core::combat::CombatTables {
        enemy_base_hp: numbers(include_str!("../../game-core/tests/fixtures/enemybasehp.txt")),
        enemy_base_atk: numbers(include_str!("../../game-core/tests/fixtures/enemybaseatk.txt")),
        enemy_base_def: numbers(include_str!("../../game-core/tests/fixtures/enemybasedef.txt")),
        difficulty_multipliers: numbers(include_str!("../../game-core/tests/fixtures/difficulty_multipliers.txt")),
    };
    let images = scene_images();
    let ui = load_container("ui", 25);
    let mut machine = BootMachine::new(
        decode_png(&raw("l0.png")),
        decode_png(&raw("l1.png")),
        load_container("sflogo", 8),
        ui,
        load_container("menu", 2),
        load_container("mapbg", 1)[0].clone(),
        load_container("intro", 2),
        load_container("end", 1),
        load_container("load", 2),
    );
    machine.preset_stores = stores.clone();
    machine.sptarm = load_container("sptarm", 10);

    let tileset = load_container("map", 12)[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut scene: Option<GameScene> = None;
    let mut pending_key = 0i32;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in records {
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
            if machine.mode == 3 && machine.slot_loaded.is_some() {
                let m = &machine;
                let mut sc = game_core::scene::load_floor_from_save(
                    &cur.maplv,
                    &cur.records,
                    &width_table,
                    &walkability,
                    images.clone(),
                    floors.clone(),
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
                sc.enemy_base_gold = numbers(include_str!("../../game-core/tests/fixtures/enemybasegold.txt"));
                // rng 续接 machine LCG（docs/knowledge/gotcha_gamescene-rng-must-continue-machine-lcg.md）
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
        let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode);
        assert_eq!(sha, rec.sha, "TICK {} 帧不符（mode={}）", rec.tick, mode);
        checked += 1;
    }
    (checked, scene.expect("应已交场景（槽载入完成 ⇒ gameMode 3）"))
}

/// mode12 记录浏览器（迭代 59 原 preset_floor5 用例，35e779f 起改作他用后缺位——此处夹具化恢复）：
/// 槽载入 (4,8) → -2 步入 (4,9) 拾取道具14（addItemToItemStack 首拾描述浮层 kind1）→ -6 面板 →
/// -6 使用 → mode12（m_076(0) + paint case12）→ -4/-3 m_076 钳位翻页 → -6 关回 mode10。T1-1200。
#[test]
fn preset_floor5_fixture_frames_match() {
    let records = frame_records(include_str!("fixtures/preset-floor5.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("../../../reference/oracle/scenarios/preset-floor5.txt"));
    let (checked, sc) = replay(include_str!("../../../reference/oracle/scenarios/preset-floor5.preset"), &records);
    assert_eq!(checked, records.len(), "T1-1200 全帧必须比对");
    assert_eq!(sc.floor, 5);
    assert!(sc.player.find_item(14) >= 0, "道具14 已入栈（addItemToItemStack）");
}

/// 按层铺 floor49（迭代 66 原用例，308765f 起改作他用后缺位——此处夹具化恢复）：门卫/守卫宝箱配对
/// + (6,8) 型83 首访 inert（tryRunScene case15：m_102(当前层,5) 守卫在场 ⇒ false）→ 二访伏击
/// script15 → 61/70 战 → case70 → script35。T1-3400。
#[test]
fn preset_floor49_fixture_frames_match() {
    let records = frame_records(include_str!("fixtures/preset-floor49.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("../../../reference/oracle/scenarios/preset-floor49.txt"));
    let (checked, sc) = replay(include_str!("../../../reference/oracle/scenarios/preset-floor49.preset"), &records);
    assert_eq!(checked, records.len(), "T1-3400 全帧必须比对");
    assert_eq!(sc.floor, 49);
}

/// advanceScriptWalk：floor3 (3,11) -3 撞 (2,11) 型83(param 0) ⇒ script0；`MOV_0_5_11` 玩家
/// 两步脚本行走（dialogPhase 3 + 每步 applyStepCellEffects → advanceScriptWalk）→ TAK_8_9。T1-700。
#[test]
fn fx_scriptwalk_floor3_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-scriptwalk-floor3.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-scriptwalk-floor3.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-scriptwalk-floor3.preset"), &records);
    assert_eq!(checked, records.len(), "T1-700 全帧必须比对");
    assert_eq!((sc.view.player_px >> 5, sc.view.player_py >> 5), (5, 11), "MOV_0_5_11 终点");
}

/// m_101：floor43 (3,5) -4 步入 (4,5)（上邻 (4,4) 型60）⇒ 击退至 (4,3) + 扣血 200；-1 步入 (4,4)
/// ⇒ 上邻 (4,3) 型60、其上 (4,2) 为墙 ⇒ 不退 + 再扣 200。T1-600。
#[test]
fn fx_knockback_floor43_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-knockback-floor43.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-knockback-floor43.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-knockback-floor43.preset"), &records);
    assert_eq!(checked, records.len(), "T1-600 全帧必须比对");
    assert_eq!(sc.player.hp, 100, "500 − 200 × 2（a.java:5628-5631）");
}

/// m_089：floor13 (6,8) 芭蕉扇×3：未命中（四邻无型10）⇒ 提示浮层；上移 (6,7) 再用 ⇒ 命中灭
/// (6,6) 三昧真火（次数 3→2）→ 上移 (6,6)。T1-830。
#[test]
fn fx_fan_floor13_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-fan-floor13.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-fan-floor13.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-fan-floor13.preset"), &records);
    assert_eq!(checked, records.len(), "T1-830 全帧必须比对");
    let i = sc.player.find_item(16);
    assert!(i >= 0 && sc.player.item_uses[i as usize] == 2, "命中一次消耗 1（a.java:7766）");
    assert_eq!((sc.view.player_px >> 5, sc.view.player_py >> 5), (6, 6), "灭火后可通行");
}

/// 移植缺口复现（2026-10-15 fx-fan 场景铺设时发现）：撞型10 三昧真火 ⇒ Java interactWithCell case10
/// （a.java:5436-5440）提示浮层 + 阻挡；Rust interact_with_cell cat1 `_ => {} // 11 / 76/82：后续批次`
/// 吞掉型 10/11/12 撞击臂（无 todo! 标记，此前无场景触达故帧对拍恒绿）。T1-480，首分歧 T301。
/// game-core 补臂后摘除 #[ignore]（本用例即其帧级验收）。
#[test]
fn fx_firebump_floor13_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-firebump-floor13.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-firebump-floor13.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-firebump-floor13.preset"), &records);
    assert_eq!(checked, records.len(), "T1-480 全帧必须比对");
    assert_eq!((sc.view.player_px >> 5, sc.view.player_py >> 5), (6, 7), "型10 阻挡（var6=false）");
}
