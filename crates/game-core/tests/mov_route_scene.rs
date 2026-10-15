//! MOV 五参实体路径缓动（dp2 = m_106 + m_108）场景级对拍：Rust 全帧 vs Java A 变体帧。
//!
//! 覆盖（completeness §3.4 判据① a 类 m_106/m_108，a.java:8274-8309 / 8738-8904）：
//! - floor2 script1 `MOV_72_3_7_1_8`：型72 自 (3,7) 经 下/左/左 到 (1,8)——m_106 case0
//!   弹步 + 途经 (2,8) 型11 墙 markEntityRemoved（开墙）、case1 8px 步进 + 相机目标追
//!   实体 + 动画行 1 帧推进、同拍递归弹下一步、步尽 m_108 迁槽 + dispatchAnimFields；
//!   paintEntityLayer case32 型72 追踪分支（f_byte_19 帧行，左 = 镜像 drawEdgePatch）。
//! - floor2 script2 `MOV_72_1_8_1_10` + `DES_72_1_10`：撞 (1,8) 的触发依赖 m_108 已把型72
//!   迁入 (1,8)；DES 命中 (1,10) 依赖第二次 m_108 迁槽——槽表语义的端到端证据。
//! - 读档旗标：MOT_L0 scriptLineFlags[0]=1（script1 门控 a.java:8081-8082 `flags[0] & !walk`）
//!   ——load_floor_from_save 读回旗标（a.java:9285-9287）才可能触发 script1。
//! - floor40 script12 伏击（12 条非 72 型 MOV + `MOV_73`）：每只怪的路径经 x=6 列下行，
//!   下一格 = 玩家格 (6,7) ⇒ m_106 case0 途中开战（缩放表 HP、walkPhase=5、
//!   battleActiveFlag=false、state=5、routePhase=5、敌先手）→ case5 tickBattle(false)
//!   击杀收尾 → dp0（余步丢弃）；battleTick 跨战连续 ⇒ 13 场开战→击杀间隔不齐；
//!   型73 途中战击杀 → case73 钩子 loadLevelScript(22)（dp2 被脚本装载截断）→ script22
//!   → mode3 tickBattle(true) 收尾 restep。
//!
//! 夹具（crates/game-core/tests/fixtures/）：
//! - `preset-floor2-mov.preset`：MOT_IF/SKY_WAR 逐字取自 reference/oracle/scenarios/
//!   preset-floor2.preset；MOT_L0 = 同文件改玩家格 (4,7) + scriptLineFlags[0]=1；
//! - `preset-floor40-mov.preset`：MOT_IF/SKY_WAR 逐字取自 preset-itemboss40.preset；
//!   MOT_L0 = 同文件改玩家格 (6,9)、hp/atk/def=1000、钥匙 0/0/1（黄/蓝/红）；
//! - `<stem>.txt`：输入脚本（`# ticks:`/`# preset:` 头）；
//! - `<stem>.frames.txt`：Java 参照帧（逐 TICK `tick sha [INPUT]`），由 A 变体 trace 抽取。
//!
//! 再生（无头；产物落 reference/oracle/_out/，gitignore）：
//! ```text
//! python3 - <<'EOF'
//! import importlib.util, re; from pathlib import Path
//! s = importlib.util.spec_from_file_location("r", "reference/oracle/run.py")
//! run = importlib.util.module_from_spec(s); s.loader.exec_module(run)
//! scen = Path("crates/game-core/tests/fixtures/preset-floor2-mov.txt").resolve()
//! ticks = int(re.search(r"^#\s*ticks:\s*(\d+)", scen.read_text(), re.M).group(1))
//! cls = run.build_shim_host(); orig = run.ensure_game_classes()
//! vc = run.find_variant_c(None); run.ttransform_variants(orig, vc)
//! run.run_variants_parallel([(cls, run.OUT / f"variant{v}", f"{v}-preset-floor2-mov", ticks,
//!     scen, str(scen.with_suffix(".preset"))) for v in "ABC"], frames="off", dump_stride=8)
//! EOF
//! ```
//! （floor40 同法，stem 换 `preset-floor40-mov`。）三方 trace sha256 须一致；再按 TICK 抽
//! `FRAME sha=` 与 `INPUT` 行写回 frames 夹具。
//!
//! Java 侧锚点（A 变体 dumpStride=1 取证 trace，FLD 243 dialogPhase / 257 f_int_127 /
//! 258 routePhase（原 f_byte_18）/ 259 f_byte_19 / 342 walkStepCount / 082 walkPhase /
//! 263 battleActiveFlag）：
//! - floor2：T606 MOV（f_int_127=12、步数 3）→ T607-T619 缓动（每步 4 拍）→ T619 收尾
//!   f_int_127=-1 → T620 dp4 → T630 mode3；T711 script2 → T852 MOV（步数 2）→ T861 收尾
//!   → T862 DES → T863 dp4 → T870 mode3。
//! - floor40：T285 script12 → T410 首条 MOV → routePhase=5 开战拍 423/448/480/504/532/564/
//!   600/640/684/720/760/804/907（walkPhase T423 起恒 5、battleActiveFlag 恒 false）→
//!   T911 currentScriptIndex=22 → T933 mode3 → T939 walkPhase=1（restep）→ T943 玩家 (6,6)。
//!
//! 资产：assets/raw（门禁 provision 自供给，tools/extract-assets 由 original/ 再生）。

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

/// 一个场景夹具（`fixtures/<stem>.{preset,txt,frames.txt}` + 层号）。
struct Spec {
    floor: usize,
    preset: &'static str,
    script: &'static str,
    frames: &'static str,
    ticks: usize,
}

const FLOOR2: Spec = Spec {
    floor: 2,
    preset: include_str!("fixtures/preset-floor2-mov.preset"),
    script: include_str!("fixtures/preset-floor2-mov.txt"),
    frames: include_str!("fixtures/preset-floor2-mov.frames.txt"),
    ticks: 900,
};

const FLOOR40: Spec = Spec {
    floor: 40,
    preset: include_str!("fixtures/preset-floor40-mov.preset"),
    script: include_str!("fixtures/preset-floor40-mov.txt"),
    frames: include_str!("fixtures/preset-floor40-mov.frames.txt"),
    ticks: 1000,
};

/// 帧夹具一行：(tick, Java FRAME sha, INPUT)。
struct FrameRec {
    tick: u32,
    sha: String,
    input: Option<String>,
}

fn frame_records(spec: &Spec) -> Vec<FrameRec> {
    spec.frames
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
fn check_fixture_inputs(spec: &Spec) {
    let script: Vec<String> = spec
        .script
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let p: Vec<&str> = l.split_whitespace().collect();
            format!("{} {}({})", p[0].parse::<u32>().unwrap(), p[2], p[1])
        })
        .collect();
    let recs = frame_records(spec);
    let from_trace: Vec<String> = recs
        .iter()
        .filter_map(|r| r.input.as_ref().map(|i| format!("{} {i}", r.tick)))
        .collect();
    assert_eq!(from_trace, script);
    assert_eq!(recs.len(), spec.ticks, "# ticks");
    assert!(recs.iter().enumerate().all(|(i, r)| r.tick as usize == i + 1), "TICK 连续");
    let header = format!("# ticks: {}", spec.ticks);
    assert!(spec.script.lines().any(|l| l.trim() == header), "脚本头 {header}");
}

#[test]
fn preset_floor2_mov_fixture_inputs_match_script() {
    check_fixture_inputs(&FLOOR2);
}

#[test]
fn preset_floor40_mov_fixture_inputs_match_script() {
    check_fixture_inputs(&FLOOR40);
}

/// 全流程重放（boot → 标题 → 载入 → 槽0 → 场景移交 → 脚本输入）+ 逐 TICK 帧 sha
/// 比对；`observe(tick, scene)` 在每拍逻辑后、绘制前调用（覆盖登记）。
fn run_scene(spec: &Spec, mut observe: impl FnMut(u32, &GameScene)) -> GameScene {
    let stores = parse_preset(spec.preset);
    let records = frame_records(spec);

    let width_table: Vec<i32> = numbers(include_str!("fixtures/type_width_px.txt"));
    let walkability: Vec<i32> = numbers(include_str!("fixtures/tile_walkability.txt"));
    let doors = DoorTables {
        door12: numbers(include_str!("fixtures/door12.txt")),
        door13: numbers(include_str!("fixtures/door13.txt")),
    };
    let maplv = raw(&format!("maplv{}", spec.floor));
    let sp = SpriteTable::parse(&raw(&format!("sprite{}", spec.floor))).unwrap();
    let floor_records: Vec<(u8, u16, u16, i16, bool)> = sp
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
    floor_assets[spec.floor] = Some(FloorAssets { maplv: maplv.clone(), records: floor_records.clone() });

    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();
    assert_eq!(save.current_floor as usize, spec.floor);
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
            if machine.mode == 3 && machine.slot_loaded.is_some() {
                let m = &machine;
                let mut sc = game_core::scene::load_floor_from_save(
                    &maplv,
                    &floor_records,
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
        if let Some(sc) = scene.as_ref() {
            observe(rec.tick, sc);
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
        let mode = scene.as_ref().map(|s| s.game_mode).unwrap_or(machine.mode as i32);
        let dp = scene.as_ref().map(|s| s.dialog_phase).unwrap_or(-1);
        assert_eq!(sha, rec.sha, "TICK {} 帧不符（mode={mode} dp={dp}）", rec.tick);
        checked += 1;
    }
    assert_eq!(checked, spec.ticks, "全帧必须比对");
    scene.expect("应已交场景")
}

/// 场景格 (x,y) 槽表中型 t 的实体下标。
fn at(sc: &GameScene, x: i32, y: i32, t: i32) -> Vec<usize> {
    let ct = sc.grid.cell_type[y as usize][x as usize] as usize;
    (0..sc.grid.capacity[ct] as usize)
        .map(|s| sc.grid.slots[ct][s] as usize - 1)
        .filter(|&id| sc.entities.entity_type[id] == t)
        .collect()
}

/// floor2：T1-900 全帧一致 + 型72 路径缓动各语义面实际进入比对（覆盖计数断言）。
#[test]
fn preset_floor2_mov_frames_match() {
    let stores = parse_preset(FLOOR2.preset);
    let mot_l0 = &stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).expect("MOT_L0").1;
    let save = parse_floor_save(mot_l0, 0).unwrap();
    assert!(save.script_flags[0] && save.script_flags[1..].iter().all(|f| !f), "预置旗标 = 仅 [0]");
    // dp2 拍数（脚本 0/1/2）、型72 镜像/非镜像帧行拍数、开墙/迁槽/DES 命中拍
    let mut dp2_ticks = [0usize; 3];
    let (mut mirrored, mut unmirrored) = (0usize, 0usize);
    let (mut wall_opened_at, mut relocated_18_at, mut des_hit_at) = (None, None, None);
    let mut flags_loaded = false;
    let sc = run_scene(&FLOOR2, |tick, sc| {
        flags_loaded |= sc.script_line_flags[0];
        if sc.game_mode == 11 && sc.dialog_phase == 2 {
            dp2_ticks[sc.current_script_index.min(2)] += 1;
            let e = sc.route_entity;
            if e >= 0 && sc.entities.entity_type[e as usize] == 72 {
                if sc.route_dir == 3 {
                    mirrored += 1;
                } else {
                    unmirrored += 1;
                }
            }
        }
        if wall_opened_at.is_none() && at(sc, 2, 8, 11).iter().any(|&id| sc.entities.solid[id] == 1) {
            wall_opened_at = Some(tick);
        }
        if relocated_18_at.is_none() && !at(sc, 1, 8, 72).is_empty() {
            relocated_18_at = Some(tick);
        }
        if des_hit_at.is_none() && at(sc, 1, 10, 72).iter().any(|&id| sc.entities.solid[id] == 1) {
            des_hit_at = Some(tick);
        }
    });
    assert!(flags_loaded, "读档旗标须读回（a.java:9285-9287）");
    // Java 锚：script1 dp2 = T606-T618（13 拍）、script2 dp2 = T852-T860（9 拍）
    assert_eq!(dp2_ticks, [0, 13, 9], "dp2 拍数（脚本 0/1/2）");
    assert!(mirrored > 0 && unmirrored > 0, "左行镜像/非镜像帧行均须绘制: {mirrored}/{unmirrored}");
    assert_eq!(wall_opened_at, Some(611), "m_106 case0 开墙拍（弹第 2 步 → 下一格 (2,8)）");
    assert_eq!(relocated_18_at, Some(619), "m_108 迁槽拍（步尽收尾）");
    assert_eq!(des_hit_at, Some(862), "DES_72_1_10 命中拍（依赖第二次 m_108）");
    assert_eq!(sc.game_mode, 3);
    assert_eq!(sc.route_entity, -1, "f_int_127 收尾 = -1");
    assert!(sc.script_line_flags[1] && sc.script_line_flags[2], "script1/2 行尾旗标");
}

/// floor40：T1-1000 全帧一致 + script12 伏击 13 场途中战斗（12 小怪 + 型73 boss）。
#[test]
fn preset_floor40_mov_frames_match() {
    // Java 锚（A 变体 dumpStride=1 取证 trace）：开战拍 = routePhase 0/1→5 的拍
    const BATTLE_STARTS: [u32; 13] = [423, 448, 480, 504, 532, 564, 600, 640, 684, 720, 760, 804, 907];
    let mut starts = Vec::new();
    let mut prev_phase = 0;
    let mut battle_flag_false_in_dp2 = true;
    let mut boss_hook_at = None;
    let mut route_types = Vec::new();
    let sc = run_scene(&FLOOR40, |tick, sc| {
        if sc.game_mode == 11 && sc.dialog_phase == 2 {
            if sc.route_phase == 5 && prev_phase != 5 {
                starts.push(tick);
                battle_flag_false_in_dp2 &= !sc.battle_active && sc.walk_phase == 5;
                route_types.push(sc.entities.entity_type[sc.route_entity as usize]);
            }
            prev_phase = sc.route_phase;
        } else {
            prev_phase = 0;
        }
        if boss_hook_at.is_none() && sc.current_script_index == 22 {
            boss_hook_at = Some(tick);
        }
    });
    assert_eq!(starts, BATTLE_STARTS, "13 场途中开战拍（battleTick 跨战连续 ⇒ 间隔不齐）");
    assert!(battle_flag_false_in_dp2, "途中战 battleActiveFlag=false + walkPhase=5");
    assert_eq!(route_types, [58, 58, 58, 57, 57, 57, 56, 56, 56, 59, 59, 59, 73], "MOV 顺序 = 脚本序");
    assert_eq!(boss_hook_at, Some(911), "型73 击杀钩子 loadLevelScript(22)（a.java:10071-10073）");
    assert_eq!(sc.game_mode, 3);
    assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, 6), "mode3 tickBattle(true) 收尾 restep 北移");
    assert!(sc.script_line_flags[12], "script12 旗标（case73 钩子置当前脚本）");
}
