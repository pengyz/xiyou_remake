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
    let (checked, scene) = replay_opt(preset_text, records);
    (checked, scene.expect("应已交场景（槽载入完成 ⇒ gameMode 3）"))
}

/// 同 `replay`，但交场景前截止的夹具（FX-BUG-AIOOBE：Java 于 T237 dispatch 崩溃、
/// 夹具止于 T236）场景可缺席。
fn replay_opt(preset_text: &str, records: &[FrameRec]) -> (usize, Option<GameScene>) {
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
    (checked, scene)
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

/// FX-INT-6A：型6 首碰显形三元组 + 幂等 + 地形豁免（evidence-interact-types R-INT-6）。
/// floor5 (10,11) 左撞 (9,11) 型6 ⇒ 显形三元组 visible→state=2→animRow=9（a.java:5413-5415
/// 写序）→ 放行进（var7++ 地形豁免，该格地形不可走）→ advanceEntityFrames state=2 过渡完成
/// （a.java:6838-6843：dispatchSpriteSize+dispatchAnimFields+state=0）→ (10,11) 回退 →
/// 二次进入幂等（仅 var7++，三字段不改写）。T1-560。出生点不得钉边界格（BUG 见
/// fx_bug_aioobe_border_frames_match）。
#[test]
fn fx_int6a_reveal_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-int6a.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-int6a.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-int6a.preset"), &records);
    assert_eq!(checked, records.len(), "T1-560 全帧必须比对");
    assert_eq!(sc.floor, 5);
    assert_eq!(
        (sc.view.player_px >> 5, sc.view.player_py >> 5),
        (9, 11),
        "终态站上 (9,11)——型6 地形豁免放行（var7>0 跳过 isCellWalkable 终审）"
    );
    let e = (0..sc.entities.count)
        .find(|&i| sc.entities.entity_type[i] == 6)
        .expect("型6 实体在表");
    assert!(sc.entities.visible[e], "显形后 visible 恒真（a.java:5413）");
    assert_eq!(sc.entities.solid[e], 0, "state=2 过渡完成后归零（a.java:6842）");
    assert!(sc.player_bob_applied, "落格 f_bool_07=true（a.java:5650-5652）");
}

/// FX-BUG-AIOOBE：边界出生崩溃钉死（原版 bug 复现夹具，D4 保真不修——bug-ledger 待 lead 编号）。
/// floor5 (9,12) = 底边界格 ⇒ mode3 进场 applyStepCellEffects 型60 四邻扫描先调后判
/// （a.java:5596-5616 `findBlockingEntity(var45, var49+1, 60)` 于 y=12 取 cellTypeGrid[13]）
/// ⇒ Java T236 AIOOBE 主线程退出（trace 尾 EVT thread-exit；C 变体可读栈
/// findBlockingEntity a.java:8616 ← applyStepCellEffects a.java:5617 ← dispatchModeEntry
/// a.java:4320）。A/B/C 三变体同 tick 同点同因崩溃（三方 trace sha256 含崩溃尾一致）。
/// 夹具帧 1..236 = Java 可观测前缀；Rust cell_entity 越界返 -1（entity.rs）⇒ 存活续跑，
/// T237 起无 Java 参照帧（存活差异已记 bug 条目）。可达性 A 级：全 55 层边界格不可走
/// （tile_walkability 复算）⇒ 正常游玩不可达，仅 preset/存档直钉可复现。
#[test]
fn fx_bug_aioobe_border_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-bug-aioobe-border.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-bug-aioobe-border.txt"));
    let (checked, scene) = replay_opt(include_str!("fixtures/fx-bug-aioobe-border.preset"), &records);
    assert_eq!(checked, records.len(), "T1-236 = Java 崩溃前可观测前缀全帧对拍");
    assert!(
        scene.is_none(),
        "T236 截止时 Rust 尚未交场景——交场景在 T237，恰为 Java dispatchModeEntry 崩溃拍（同点对应）"
    );
}

/// FX-DOOR-2A：型2 红门开门（有钥匙）（evidence-interact-types R-INT-2 有钥匙半）。
/// floor40 (6,9) 北撞 (6,8) 红门 ⇒ consumeKeyForDoor(27)（a.java:7746-7752）popup→
/// redKeyCount-- → markEntityRemoved（state=1+开门动画，a.java:6355-6361）→ 放行进门格。
/// (6,7) 是型83(param12) 伏击点，本场景止步 (6,8)。T1-520。
#[test]
fn fx_door2a_open_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-door2a.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-door2a.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-door2a.preset"), &records);
    assert_eq!(checked, records.len(), "T1-520 全帧必须比对");
    assert_eq!(sc.floor, 40);
    assert_eq!(
        (sc.view.player_px >> 5, sc.view.player_py >> 5),
        (6, 9),
        "T280 撞门→进门格 (6,8)（帧窗 T288-T400）→ T400 南移回 (6,9)（终态）"
    );
    assert_eq!(sc.player.red_keys, 0, "redKeyCount 1→0（a.java:7749）");
    let d = (0..sc.entities.count)
        .find(|&i| sc.entities.entity_type[i] == 2)
        .expect("型2 红门在表");
    assert_eq!(sc.entities.solid[d], 1, "markEntityRemoved state=1（a.java:6350）");
}

/// FX-DOOR-2B：型2 无钥匙阻挡（负例，R-INT-2 无钥匙半）。红钥 0 ⇒ 双次北撞均
/// walkStepCount=0 +「你没有红钥匙」浮层（a.java:5392-5395）+ var6=false，门不消耗不开启。
#[test]
fn fx_door2b_nokey_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-door2b.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-door2b.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-door2b.preset"), &records);
    assert_eq!(checked, records.len(), "T1-520 全帧必须比对");
    assert_eq!(sc.floor, 40);
    assert_eq!(
        (sc.view.player_px >> 5, sc.view.player_py >> 5),
        (6, 9),
        "无钥匙恒阻挡，玩家不动（var6=false）"
    );
    assert_eq!(sc.player.red_keys, 0, "无钥匙无从消耗");
    let d = (0..sc.entities.count)
        .find(|&i| sc.entities.entity_type[i] == 2)
        .expect("型2 红门在表");
    assert_eq!(sc.entities.solid[d], 0, "门保持关闭（state 恒 0）");
}

/// FX-ITEM-17B：金勺子失败不消耗（负例，evidence-items §3.1/§2.2 失败分支）。floor13 无
/// 邻型11 ⇒ activateItem case17 返 false（a.java:7809-7816）⇒ useItemStack 短路不减次
/// （a.java:7766）⇒ 栈原封不动 +「你必须面对一堵墙使用」浮层。T1-560。
#[test]
fn fx_item17b_fail_no_consume_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-item17b.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-item17b.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-item17b.preset"), &records);
    assert_eq!(checked, records.len(), "T1-560 全帧必须比对");
    assert_eq!(sc.floor, 13);
    assert_eq!((sc.view.player_px >> 5, sc.view.player_py >> 5), (6, 8), "全程不移动");
    let i = sc.player.find_item(17) as usize;
    assert!(i < sc.player.item_stack_size as usize, "金勺子仍在栈（失败不出栈）");
    assert_eq!(sc.player.item_uses[i], 1, "uses 恒 1（失败不消耗，a.java:7766 短路）");
}

/// FX-ITEM-19A：千年月桂露回血精确值（evidence-items §2.4/§3.6）。atk=105 def=203 ⇒
/// (105+203)*74/10 = 2279（先乘后除整数截断，a.java:7830）→ hp 500+2279=2779 + 出栈。T1-560。
#[test]
fn fx_item19a_heal_exact_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-item19a.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-item19a.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-item19a.preset"), &records);
    assert_eq!(checked, records.len(), "T1-560 全帧必须比对");
    assert_eq!(sc.floor, 13);
    assert_eq!(sc.player.hp, 2779, "500 + (105+203)*74/10 = 500+2279（整数截断钉死）");
    assert_eq!(sc.player.find_item(19), -1, "uses 1→0 出栈（a.java:7766-7775）");
    assert_eq!(sc.player.item_stack_size, 0, "栈清空");
}

/// FX-ITEM-17A：金勺子凿墙成功臂（evidence-items §2.2/§3.4/§3.5）。floor29 (8,3) 型11
/// 「能挖的墙」⇒ m_089(11) 四邻命中 markEntityRemoved（a.java:7895-7919）+ 出栈 +
/// gameMode=3 ⇒ T520 北进 (8,3) 端到端通行。quirk 保真：提示语「面对」实为四邻全删。T1-620。
#[test]
fn fx_item17a_chisel_wall_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-item17a.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-item17a.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-item17a.preset"), &records);
    assert_eq!(checked, records.len(), "T1-620 全帧必须比对");
    assert_eq!(sc.floor, 29);
    assert_eq!((sc.view.player_px >> 5, sc.view.player_py >> 5), (8, 3), "凿墙后走入原墙格");
    assert_eq!(sc.player.find_item(17), -1, "成功消耗 1 次出栈");
    let w = (0..sc.entities.count)
        .find(|&i| sc.entities.entity_type[i] == 11)
        .expect("型11 墙在表");
    assert_eq!(sc.entities.solid[w], 1, "markEntityRemoved state=1（墙开）");
}

/// FX-ITEM-18A：玄明石全清当前层墙（evidence-items §2.3/§3.4）。floor37 型11 ×8 同拍
/// 全 markEntityRemoved（a.java:7817-7828 全表扫描）+ 出栈。T1-560。
#[test]
fn fx_item18a_clear_all_walls_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-item18a.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-item18a.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-item18a.preset"), &records);
    assert_eq!(checked, records.len(), "T1-560 全帧必须比对");
    assert_eq!(sc.floor, 37);
    let walls: Vec<usize> = (0..sc.entities.count)
        .filter(|&i| sc.entities.entity_type[i] == 11)
        .collect();
    assert_eq!(walls.len(), 36, "sprite37 型11 原生 36 堵（字节级解析 A 级：82 记录中 36×型11）");
    assert!(
        walls.iter().all(|&w| sc.entities.solid[w] == 1),
        "36 堵全 state=1（全层清墙，a.java:7820-7824）"
    );
    assert_eq!(sc.player.find_item(18), -1, "成功消耗 1 次出栈");
    assert_eq!((sc.view.player_px >> 5, sc.view.player_py >> 5), (6, 6), "全程不移动");
}

/// FX-BOSS-67A：啸天犬击杀钩子 spawnBossEvent(3) → 金勺子掉落（evidence-boss §2.2/
/// R-boss-2/3）。floor15 (6,5) 型67（与型5 门卫同格——门卫无 interact 臂透明）bump 战
/// 一击毙 ⇒ bossTypeOrder[3]=67 ⇒ spawnBossEvent(3) ⇒ bossEventSpawns[3]={17,6,4,0}
/// 刷金勺子(17)@(6,4) DES 态（a.java:2143）+ 金 +100。夹具帧链 A==B 双变体一致——
/// C 变体因 deobf 投影 a.java:6592 型67 拼装臂 byte[]→Object[] 强转 ClassCastException
/// 不可参与三方（A 级记录，见 reference/oracle/scenarios/bug-draft-aioobe-border.md
/// 附记 / lead 追踪项）。T1-560。
#[test]
fn fx_boss67a_kill_spawn_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-boss67a.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-boss67a.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-boss67a.preset"), &records);
    assert_eq!(checked, records.len(), "T1-560 全帧必须比对");
    assert_eq!(sc.floor, 15);
    assert_eq!(sc.player.gold, 200, "预置 100 + enemyBaseGold[26]=100");
    let drop = (0..sc.entities.count)
        .find(|&i| sc.entities.entity_type[i] == 17)
        .expect("spawnBossEvent(3) 刷金勺子(17)");
    assert_eq!(
        (sc.entities.pixel_x[drop] >> 5, sc.entities.pixel_y[drop] >> 5),
        (6, 4),
        "bossEventSpawns[3]={{17,6,4,0}} @格(6,4)"
    );
    assert_eq!(sc.entities.solid[drop], 0, "DES 态=过渡（a.java:6333-6345 置 state=2+animRow=8）\
        完成后 dispatchSpriteSize/AnimFields+state=0（a.java:6838-6843）——终态 0 即合同");
    assert_eq!(sc.entities.anim_idx[drop], 0, "dispatchAnimFields 归位 animRowByType[17]=0");
    let boss = (0..sc.entities.count)
        .find(|&i| sc.entities.entity_type[i] == 67)
        .expect("型67 在表");
    assert!(sc.entities.removed[boss], "啸天犬死亡动画后 pullEntityFromCell");
}

/// FX-BOSS-74A：巨灵神双钩子（evidence-boss §3.2/R-boss-2）：击杀 74 ⇒ ① spawnBossEvent(1)
/// 12 件掉落（a.java:2092-2141）② loadLevelScript(21) 遗言 TAK_59_59（无 tryRunScene
/// 门控）⇒ script_line_flags[21] 收尾置位。preset flags=6 跳过 script6 登场戏。T1-600。
#[test]
fn fx_boss74a_kill_hooks_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-boss74a.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-boss74a.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-boss74a.preset"), &records);
    assert_eq!(checked, records.len(), "T1-600 全帧必须比对");
    assert_eq!(sc.floor, 20);
    assert_eq!(sc.player.gold, 244, "预置 100 + enemyBaseGold[33]=144");
    assert!(sc.script_line_flags[21], "script21 遗言已跑完（收尾置位 a.java:8600）");
    // flags=6 预置即 scriptLineFlags[6]=true（登场戏门关，script6 全程未跑——断言
    // 该旗标恒真无意义，登场戏缺席由帧链与 script21 独占 currentScriptIndex 佐证）。
    // 12 件掉落（型:格）：29×3、26×3、30×3、32×3（a.java:2092-2141）
    let drops = [(29, 4, 4), (29, 4, 5), (29, 5, 4), (26, 7, 4), (26, 8, 4), (26, 8, 5),
                 (30, 4, 7), (30, 4, 8), (30, 5, 8), (32, 8, 8), (32, 7, 8), (32, 8, 7)];
    for (t, cx, cy) in drops {
        assert!(
            (0..sc.entities.count).any(|i| {
                sc.entities.entity_type[i] == t
                    && sc.entities.pixel_x[i] >> 5 == cx
                    && sc.entities.pixel_y[i] >> 5 == cy
            }),
            "掉落 型{t} @({cx},{cy}) 缺失"
        );
    }
}

/// FX-INT-11-F23：型12 变形臂 + 型11 静默臂双臂钉死（evidence-interact-types R-INT-11 +
/// 型12 行；closure-typist 建议升级场景）。floor23 (2,4) 型12 隐形墙（sprite23 原生 ×24）
/// 右撞 ⇒ case12（a.java:5449-5456）变形：type 12→11 + dispatchSpriteSize/AnimFields(11)
/// + state=2 + animRow=9 + visible=true（写序固定）⇒ 过渡完成归零（a.java:6838-6843）；
/// 二撞 ⇒ case11（a.java:5441-5448）floor==23 跳过文案但恒阻挡（var6=false+var8=0）。
/// T1-560。
#[test]
fn fx_int11_f23_transform_silent_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-int11f23.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-int11f23.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-int11f23.preset"), &records);
    assert_eq!(checked, records.len(), "T1-560 全帧必须比对");
    assert_eq!(sc.floor, 23);
    assert_eq!(
        (sc.view.player_px >> 5, sc.view.player_py >> 5),
        (1, 4),
        "双撞均阻挡（case12 var6=false / case11 var6=false）"
    );
    let w = (0..sc.entities.count)
        .find(|&i| sc.entities.pixel_x[i] >> 5 == 2 && sc.entities.pixel_y[i] >> 5 == 4)
        .expect("(2,4) 实体在表");
    assert_eq!(sc.entities.entity_type[w], 11, "case12 变形：type 12→11（a.java:5450）");
    assert!(sc.entities.visible[w], "变形臂 visible=true（a.java:5455）");
    assert_eq!(sc.entities.solid[w], 0, "state=2 过渡完成后归零（a.java:6842）");
    assert_eq!(sc.entities.anim_idx[w], 0, "dispatchAnimFields(11) 归位 animRowByType[11]=0");
    assert!(
        !sc.overlay_text.as_deref().unwrap_or("").contains("障碍物"),
        "case11 静默臂：floor==23 不显「障碍物：墙」文案（a.java:5442 条件假臂）"
    );
}

/// FX-ITEM-20A：相形变位对角瞬移的 40 层禁瞬移臂（closure :7833 销账行；
/// activateItem case20 a.java:7835-7838）。floor40 使用 ⇒ 「本层不能直接瞬移。」
/// 不瞬移不消耗（var2 恒 false ⇒ useItemStack a.java:7766 短路），两次使用幂等。
/// T1-680。
#[test]
fn fx_item20a_no_teleport_floor40_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-item20a.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-item20a.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-item20a.preset"), &records);
    assert_eq!(checked, records.len(), "T1-680 全帧必须比对");
    assert_eq!(sc.floor, 40);
    assert_eq!(
        (sc.view.player_px >> 5, sc.view.player_py >> 5),
        (6, 9),
        "未瞬移（40 层禁瞬移臂）"
    );
    let i = sc.player.find_item(20) as usize;
    assert!(i < sc.player.item_stack_size as usize, "相形变位仍在栈");
    assert_eq!(sc.player.item_uses[i], 3, "两次使用零消耗（uses 恒 3，a.java:7766 短路）");
}

/// FX-INT-11-N23：非23层型11 撞墙真臂（evidence-interact-types R-INT-11 真臂半 + :5442
/// 销账真臂钉，closure-typist 三次驳回销账路径）。floor29 (8,3) 型11 双撞 ⇒ case11
/// （a.java:5441-5448）currentFloor!=23 ⇒ 「障碍物：墙\n只有金勺子、玄明石可以凿开。
/// 或者剧情打开！」+ 恒阻挡（var6=false）+ var8=0 停扫。与 FX-INT-11-F23 假臂（23 层
/// 静默）成对 ⇒ :5442 双分支齐。独立场景（不动 fx-item17a 输入，护其帧链 sha 引用）。
/// T1-400。
#[test]
fn fx_int11_n23_wall_message_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-int11n23.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-int11n23.txt"));
    let (checked, sc) = replay(include_str!("fixtures/fx-int11n23.preset"), &records);
    assert_eq!(checked, records.len(), "T1-400 全帧必须比对");
    assert_eq!(sc.floor, 29);
    assert_eq!(
        (sc.view.player_px >> 5, sc.view.player_py >> 5),
        (8, 4),
        "双撞恒阻挡（var6=false）"
    );
    assert_eq!(
        sc.overlay_text.as_deref().unwrap_or(""),
        "障碍物：墙\n只有金勺子、玄明石可以凿开。或者剧情打开！",
        "case11 真臂文案逐字（a.java:5443，floor!=23）"
    );
    let w = (0..sc.entities.count)
        .find(|&i| sc.entities.entity_type[i] == 11)
        .expect("型11 墙在表");
    assert_eq!(sc.entities.solid[w], 0, "墙未开（state 恒 0）");
}
