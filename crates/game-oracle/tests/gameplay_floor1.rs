//! P3.5 gameplay-floor1 全帧对拍：mode 3 游戏画面（A-gameplay-floor1 trace
//! T544-599 稳态窗）逐 tick FRAME sha 比对。
//!
//! 画面构成（paint case 3 a.java:2385-2422）：parallax + 瓦片层 + paintEntityLayer 实体层
//! （含 paintPlayerSprite 玩家插入）+ 小地图（buildMinimap 预生成）+ paintHudPanel HUD + paintStatusBar 状态栏
//! + drawPopupLayer（空）+ paintSoftkeyBar。
//!
//! 入口态构造（A 级证据）：
//! - 玩家/属性：preset-floor1 MOT_L0（cell(3,10)/HP500/ATK30/DEF30/黄钥1/金100）
//! - 实体：sprite1.bin 35 条 + m_122 可见性 + sortEntitiesByY 排序
//! - backdropScroll = -38（trace T544 首列 drawImage(77x320,-38)）
//! - bob = (offset -2, rising true)（mode 1 T72-161 的 90 次 advanceBobPhase 推进，
//!   序列 -1,-2,-1,0,1,2,1,0 循环，初值 (0,false) Java 字段默认）
//! - frameCounter：TICK n 拍 = n-1（run 循环变量，advanceEntityFrames 奇数拍推进）
//! - minimap on：SKY_WAR optionChecked[1]（preset 第 2 字节 = 0x01）

use game_core::entity;
use game_core::paint::paint_font;
use game_core::render::{ArgbImage, SoftGraphics};
use game_core::save::{parse_floor_save, parse_preset};
use game_core::scene::{GameScene, SceneImages};
use game_core::sprite_spawn::{spawn_param_and_visibility, DoorTables};
use game_oracle::trace;
use game_data::sprite::SpriteTable;

fn repo() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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

fn fixture_str(name: &str) -> String {
    std::fs::read_to_string(
        repo().join("crates/game-core/tests/fixtures").join(name),
    )
    .unwrap()
}

fn width_table() -> Vec<i32> {
    fixture_str("type_width_px.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

fn tile_walkability() -> Vec<i32> {
    fixture_str("tile_walkability.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

fn fixture_ints(name: &str) -> Vec<i32> {
    fixture_str(name)
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

fn door_tables() -> DoorTables {
    DoorTables {
        door12: fixture_str("door12.txt").lines().filter_map(|l| l.trim().parse().ok()).collect(),
        door13: fixture_str("door13.txt").lines().filter_map(|l| l.trim().parse().ok()).collect(),
    }
}

/// mode 3 入口态（装载链末态：loadFloorData + buildMinimap + 加载步 11）。
fn build_scene() -> GameScene {
    let maplv = std::fs::read(repo().join("crates/game-oracle/tests/fixtures/maplv1.bin")).unwrap();
    let sprite_bin =
        std::fs::read(repo().join("crates/game-oracle/tests/fixtures/sprite1.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite_bin).expect("sprite1 解析");
    let doors = door_tables();
    let records: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide_after_spawn) =
                spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide_after_spawn)
        })
        .collect();

    // 容器计数 = oracle FLD 018（a.java:451 名序）
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
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui,
    };

    let mut scene = GameScene::load_floor(
        &maplv,
        96,
        320,
        &records,
        &width_table(),
        &tile_walkability(),
        images,
    )
    .unwrap();
    scene.build_minimap();
    scene.enter_view();

    // preset-floor1：MOT_L0 HUD 数值 + SKY_WAR 小地图开关
    let preset_text =
        std::fs::read_to_string(repo().join("reference/oracle/scenarios/preset-floor1.preset"))
            .unwrap();
    let stores = parse_preset(&preset_text);
    let mot_l0 = stores
        .iter()
        .find(|(k, _)| k.starts_with("MOT_L0"))
        .map(|(_, v)| v.clone())
        .unwrap();
    let s = parse_floor_save(&mot_l0, 0).unwrap();
    scene.player.hp = s.player_hp;
    scene.player.atk = s.player_atk;
    scene.player.def = s.player_def;
    scene.player.yellow_keys = s.yellow_keys as i32;
    scene.player.blue_keys = s.blue_keys as i32;
    scene.player.red_keys = s.red_keys as i32;
    scene.player.gold = s.gold;
    scene.floor = s.current_floor as i32;
    // 敌方表：基础三表（夹具）× 难度乘数 1（difficultyIndex=0 → ×1，
    // scaleEnemyStats a.java:7229-7236）+ 赏金/基础 HP
    let base = game_core::combat::CombatTables {
        enemy_base_hp: fixture_ints("enemybasehp.txt"),
        enemy_base_atk: fixture_ints("enemybaseatk.txt"),
        enemy_base_def: fixture_ints("enemybasedef.txt"),
        difficulty_multipliers: fixture_ints("difficulty_multipliers.txt"),
    };
    let mult = base.difficulty_multipliers[s.difficulty_index as usize];
    scene.enemies = game_core::combat::scale_enemy_stats(&base, mult);
    scene.enemy_base_hp = base.enemy_base_hp.clone();
    scene.enemy_base_gold = fixture_ints("enemybasegold.txt");
    let sky_war = stores
        .iter()
        .find(|(k, _)| k.starts_with("SKY_WAR"))
        .map(|(_, v)| v.clone())
        .unwrap();
    scene.minimap_enabled = sky_war.get(1).map(|b| *b != 0).unwrap_or(false);

    // 进画面前置相位（A 级 trace/源码推导，见模块文档）：
    // mode 3 首绘在 T537；T544 首列 x=-38 ⇒ T537 绘后 -31 ⇒ 初值 -30
    // （mode 8 末次 parallax=T500 绘后 -30，mode 2 无 parallax）
    scene.backdrop_scroll = -30;
    // RNG 预推进：mode 1（T72-161）粒子消费——每 4 拍 spawnParticle
    // randomBelow(240)+randomBelow(150)（a.java:3376-3379），共 23 组；
    // 消费链已被 title/menu-sweep 全帧回放验证
    for _ in 0..23 {
        scene.rng.random_below(240);
        scene.rng.random_below(150);
    }
    scene.bob_offset = -2;
    scene.bob_rising = true;
    scene
}

/// T544-599 稳态全帧对拍（56 帧：实体动画推进 + bob + parallax 滚动）。
#[test]
fn gameplay_floor1_t537_1700_frames_match() {
    let trace_text = std::fs::read_to_string(
        repo().join("reference/oracle/_out/A-gameplay-floor1/trace.txt"),
    )
    .expect("缺 A-gameplay-floor1 trace（先跑 python3 reference/oracle/run.py --scenarios）");
    let records = trace::parse(&trace_text);

    let mut scene = build_scene();
    let tileset = {
        let map = load_container("map", 12);
        map[0].clone()
    };
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut checked = 0usize;
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        if !(537..=1700).contains(&rec.tick) {
            continue;
        }
        // mode 3 从 T537 开始（dense dumpStride=1 实证：T536=mode2、T537=mode3）。
        // T537 的 run 是 case 2（切换发生在其 else 分支内）⇒ 本拍无 advanceEntityFrames；
        // T538 起 case 3 以 frameCounter=tick-1 跑（偶数拍推进——运行时实证）。
        // 输入投递：TICK n 的 INPUT 在 preTick(n)（paint#n 后）→ logic#(n+1) 消费；
        // keyValue 为边沿触发（run 尾清零 a.java:4266），keyHeldCode 电平（release 清）
        if pending_press != 0 {
            scene.press_key(pending_press);
            pending_press = 0;
        }
        if pending_release {
            scene.release_key();
            pending_release = false;
        }
        if rec.tick > 537 {
            scene.tick(rec.tick as i64 - 1, &width_table());
        }
        if let Some(inp) = &rec.input {
            if let Some(rest) = inp.strip_prefix("press(") {
                if let Some(code) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) {
                    pending_press = code;
                }
            } else if inp.starts_with("release(") {
                pending_release = true;
            }
        }
        let sha;
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(paint_font()));
            scene.paint(&mut g, &tileset);
            sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
        }
        if let Some(expect) = &rec.frame_sha {
            assert_eq!(
                &sha, expect,
                "TICK {} 帧不符（mode 3 稳态：bob={} scroll={}）",
                rec.tick, scene.bob_offset, scene.backdrop_scroll
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 1164, "T537-1700 应有 1164 帧");
}

/// T544 ops 前缀对拍（诊断锚：Java shim 仅记前 256 条 = parallax 5 + 瓦片 250
/// + 实体层首条）。
#[test]
fn gameplay_floor1_t544_ops_prefix_match() {
    let trace_text = std::fs::read_to_string(
        repo().join("reference/oracle/_out/A-gameplay-floor1/trace.txt"),
    )
    .expect("缺 trace");
    // 抽 TICK 0544 的 OPS 明细行（FRAME 行在前；明细 = "  OPS count=" 后两空格缩进行）
    let mut expect: Vec<String> = Vec::new();
    let mut in_ops = false;
    let mut seen_count = false;
    for line in trace_text.lines() {
        if line.starts_with("TICK ") {
            if in_ops {
                break;
            }
            if line.starts_with("TICK 0544") {
                in_ops = true;
            }
            continue;
        }
        if !in_ops {
            continue;
        }
        if let Some(rest) = line.strip_prefix("  OPS count=") {
            let n: usize = rest.trim().parse().unwrap();
            assert_eq!(n, 705, "T544 真实 ops 数");
            seen_count = true;
            continue;
        }
        if seen_count && line.starts_with("  ") && !line.starts_with("  FLD") && !line.trim().is_empty() {
            expect.push(line.trim().to_string());
        }
    }
    assert!(expect.len() >= 256, "Java 明细行应 ≥256，实际 {}", expect.len());
    expect.truncate(256);

    let mut scene = build_scene();
    let map = load_container("map", 12);
    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    // 逐拍推进到 T544（bob/scroll 在 paint 内推进、帧在 tick 内推进）；
    // T544 拍就地取 ops 前缀比对
    let norm = |s: &str| s.replace(", ", ",");
    let mut ops_at_544: Vec<String> = Vec::new();
    for t in 537..=544i64 {
        if t > 537 {
            scene.tick(t - 1, &width_table());
        }
        let mut g = SoftGraphics::new(&mut screen);
        g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
        g.set_font(Some(paint_font()));
        scene.paint(&mut g, &tileset);
        if t == 544 {
            ops_at_544 = g.ops.clone();
        }
    }
    for (i, want) in expect.iter().enumerate() {
        let got = ops_at_544.get(i).map(|s| s.as_str()).unwrap_or("<END>");
        assert_eq!(norm(got), norm(want), "T544 op #{i}");
    }
}

/// 排序后实体表的 draw 序锚（sortEntitiesByY：pixelY 升序、等值保序）+ 玩家插入位。
#[test]
fn floor1_entity_draw_order_anchor() {
    let scene = build_scene();
    // 可见实体（visible 且未 removed）按序前几个（sprite1 spawn 序 + 排序）：
    // Y=32 行（idx0 type7 stair、idx1 41、idx2 42、idx3 41 —— 屏外但序内）
    let t: Vec<i32> = (0..scene.entities.count)
        .map(|i| scene.entities.entity_type[i])
        .collect();
    let y: Vec<i32> = (0..scene.entities.count)
        .map(|i| scene.entities.pixel_y[i])
        .collect();
    assert_eq!(y, {
        let mut s = y.clone();
        s.sort();
        s
    }, "sortEntitiesByY 排序后 pixelY 必须非降");
    // 首批（Y=32）：spawn 序 7,41,42,41
    assert_eq!(&t[..4], &[7, 41, 42, 41]);
    // 玩家插入锚：首个"玩家正下方 32px 邻域"实体 = (96,352) 的 type41
    let mut insert_at = None;
    for i in 0..scene.entities.count {
        let ex = scene.entities.pixel_x[i];
        let ey = scene.entities.pixel_y[i];
        if !scene.entities.removed[i]
            && scene.entities.visible[i]
            && 96 > ex - 32
            && 96 < ex + 32
            && 320 >= ey - 32
            && ey > 320
        {
            insert_at = Some(i);
            break;
        }
    }
    let i = insert_at.expect("玩家南侧实体存在");
    assert_eq!((scene.entities.pixel_x[i], scene.entities.pixel_y[i]), (96, 352));
    // type 83 三实体隐藏（m_122 case 83 → entityVisible=false）
    let hidden_83: usize = (0..scene.entities.count)
        .filter(|&i| scene.entities.entity_type[i] == 83 && !scene.entities.visible[i])
        .count();
    assert_eq!(hidden_83, 3);
    // 类别覆盖表锚（特例优先）
    assert_eq!(entity::render_category(76), 1);
    assert_eq!(entity::render_category(77), 16);
    assert_eq!(entity::render_category(72), 32);
    assert_eq!(entity::render_category(41), 8);
    assert_eq!(entity::render_category(15), 2);
}

/// 诊断（ignored）：导出 Rust T544 原始像素到 /tmp 供 python diff。
#[test]
#[ignore]
fn dump_t544_pixels() {
    let mut scene = build_scene();
    let map = load_container("map", 12);
    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    {
        let mut g = SoftGraphics::new(&mut screen);
        g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
        g.set_font(Some(paint_font()));
        scene.paint(&mut g, &tileset);
    }
    let mut raw = Vec::with_capacity((240 * 320 * 4) as usize);
    for p in &screen.hash_stream() {
        raw.push(*p);
    }
    std::fs::write("/tmp/rust-544.bin", raw).unwrap();
    let sha = game_platform::hash::sha256_hex(&screen.hash_stream())[..32].to_string();
    eprintln!("rust T544 sha={sha}");
}





/// 诊断（ignored）：导出 Rust 指定帧像素（`DUMP_TICK` 改目标拍）。
#[test]
#[ignore]
fn dump_frame_pixels() {
    let trace_text = std::fs::read_to_string(repo().join("reference/oracle/_out/A-gameplay-floor1/trace.txt")).unwrap();
    let records = trace::parse(&trace_text);
    let mut scene = build_scene();
    let map = load_container("map", 12);
    let tileset = map[0].clone();
    let mut screen = ArgbImage::create(game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
    let mut pending_press = 0i32;
    let mut pending_release = false;
    for rec in &records {
        if !(537..=1506).contains(&rec.tick) { continue; }
        if pending_press != 0 { scene.press_key(pending_press); pending_press = 0; }
        if pending_release { scene.release_key(); pending_release = false; }
        if rec.tick > 537 { scene.tick(rec.tick as i64 - 1, &width_table()); }
        if let Some(inp) = &rec.input {
            if let Some(rest) = inp.strip_prefix("press(") {
                if let Some(c) = rest.strip_suffix(')').and_then(|k| k.parse::<i32>().ok()) { pending_press = c; }
            } else if inp.starts_with("release(") { pending_release = true; }
        }
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_clip(0, 0, game_core::layout::SCREEN_W, game_core::layout::SCREEN_H);
            g.set_font(Some(paint_font()));
            scene.paint(&mut g, &tileset);
            if rec.tick == 1506 {
                std::fs::write("/tmp/rust-1506.ops", g.ops.join("\n")).unwrap();
            }
        }
    }
    std::fs::write("/tmp/rust-1506.bin", screen.hash_stream()).unwrap();
    eprintln!("rust T601 done");
}

/// 诊断（ignored）：actor_c[2]（星光表）尺寸与像素。
#[test]
#[ignore]
fn dump_actor2() {
    let actor_c = load_container("actor", 4);
    let a = &actor_c[2];
    eprintln!("actor[2] {}x{}", a.width, a.height);
    std::fs::write("/tmp/actor2.bin", a.hash_stream()).unwrap();
}



