//! Boot→GameScene 交接共享实现（2026-10-26 自 shell/win 迁出：Android 壳缺此面 ⇒
//! 读条完回落标题、进不了游戏）。资产装载 + 新游戏/读档两路场景构造 + 敌人数值表。
//! 双壳消费不拷贝（win 壳 load_assets/install_combat_tables/step 交接两路改调本模块）。
//!
//! 交接信号**一次性消费**（[`new_game_scene`] 清 `new_game_ready`、[`slot_scene`] 取走
//! `slot_loaded`）：旧 win 壳从不清零 ⇒ 游戏内退出回标题后下一拍即再开新游戏；读档后
//! 回标题再开新游戏会走槽分支误读旧档（两信号均为 Rust 侧信号位，非 Java 字段）。

use game_core::enums::GameMode;
use game_core::paint::BootMachine;
use game_core::render::ArgbImage;
use game_core::scene::{FloorAssets, GameScene};

macro_rules! fixture {
    ($t:literal) => {
        include_str!(concat!("../../../crates/game-core/tests/fixtures/", $t))
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect::<Vec<i32>>()
    };
}

/// 容器装载（PackedPng 解析 + 逐张 PNG 解码）。缺资产/解析失败=panic——文案逐字保留
/// （win 壳行为锚 panic_text_load_container_is_stable）。
pub fn load_container(src: &dyn game_platform::AssetSource, name: &str, count: usize) -> Vec<ArgbImage> {
    let data = src
        .read(name)
        .unwrap_or_else(|e| panic!("缺资产 {name}（先跑 tools/extract-assets/extract.py）：{e}"));
    game_data::PackedPng::parse(&data, count)
        .unwrap_or_else(|e| panic!("容器 {name} 解析失败：{e}"))
        .images
        .iter()
        .map(|sub| ArgbImage::from_decoded_png(game_data::decode_png(sub.bytes).unwrap()))
        .collect()
}

/// 场景资产（GameScene 构造输入；win 壳原 `Assets` 同形）。
pub struct GameAssets {
    pub tileset: ArgbImage,
    pub images: game_core::scene::SceneImages,
    pub floors: Vec<Option<FloorAssets>>,
    pub width_table: Vec<i32>,
    pub tile_walk: Vec<i32>,
}

/// 场景资产全量装载（容器解码 + 55 层 maplv/sprite 解析 + 表装配；win 壳原
/// `load_assets` 逐字迁入——摘要基线 assets_loading_matches_baseline_digest 钉）。
pub fn load_game_assets(src: &dyn game_platform::AssetSource) -> GameAssets {
    let mapbg = load_container(src, "mapbg", 1);
    let map = load_container(src, "map", 12);
    let actor_c = load_container(src, "actor", 4);
    let sptmap = load_container(src, "sptmap", 13);
    let sptprop = load_container(src, "sptprop", 23);
    let sptarm = load_container(src, "sptarm", 10);
    let sptenemy1 = load_container(src, "sptenemy1", 20);
    let sptenemy2 = load_container(src, "sptenemy2", 20);
    let ui = load_container(src, "ui", 25);
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
        menu_strip: load_container(src, "menu", 2)[1].clone(),
        battle_icon: map[8].clone(),
        battle_strip: map[7].clone(),
        ui: ui.clone(),
        faces: load_container(src, "face", 12),
        ending: load_container(src, "end", 1)[0].clone(),
        xtq: load_container(src, "xtq", 6),
    };
    // 55 层全量装载（楼梯换层即时可用；maplv+sprite 每层 <2KB）
    let door_tables = game_core::sprite_spawn::DoorTables {
        door12: include_str!("../../../crates/game-core/tests/fixtures/door12.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
        door13: include_str!("../../../crates/game-core/tests/fixtures/door13.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
    };
    let mut floors: Vec<Option<FloorAssets>> = vec![None; 56];
    for (n, slot) in floors.iter_mut().enumerate().skip(1).take(54) {
        let maplv = src
            .read(&format!("maplv{n}"))
            .unwrap_or_else(|e| panic!("缺资产 maplv{n}：{e}"));
        let sp = game_data::SpriteTable::parse(
            &src.read(&format!("sprite{n}"))
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
    GameAssets {
        tileset: map[0].clone(),
        images,
        floors,
        width_table: fixture!("type_width_px.txt"),
        tile_walk: fixture!("tile_walkability.txt"),
    }
}

/// 敌人数值表装配（读档 / 新游戏两路交接共用）：难度 1 缩放 + 基础 HP / 金币表。
pub fn install_combat_tables(sc: &mut GameScene) {
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

/// 交接二：新游戏（序章冷启动——与 prologue 用例同构；step8 统计 floor51/hp300/
/// atk10/def10，a.java:3496-3506）。调用方在 `machine.new_game_ready` 时调用；
/// **消费**该信号。BootMachine 跨模式状态（bob/图标带帧/视差/rng/音频/壳层附加设置行）续接。
pub fn new_game_scene(assets: &GameAssets, machine: &mut BootMachine) -> GameScene {
    machine.new_game_ready = false;
    let fa = assets.floors[51].as_ref().unwrap();
    let mut sc = GameScene::load_floor(
        &fa.maplv,
        32,
        352,
        &fa.records,
        &assets.width_table,
        &assets.tile_walk,
        assets.images.clone(),
        assets.floors.clone(),
    )
    .unwrap();
    sc.floor = 51;
    sc.min_floor = 51;
    sc.max_floor = 51;
    sc.player.hp = 300;
    sc.player.atk = 10;
    sc.player.def = 10;
    install_combat_tables(&mut sc);
    sc.build_minimap();
    sc.enter_view();
    sc.bob_offset = machine.bob_offset;
    sc.bob_rising = machine.bob_rising;
    sc.strip_frame = machine.strip_frame;
    sc.backdrop_scroll = machine.backdrop.scroll();
    sc.rng = machine.rng.clone();
    sc.bgm = machine.bgm.clone(); // 音频状态续接（Java 同一对象）
    sc.shell_options = machine.shell_options.clone(); // 【STAGE-B-006】壳层附加设置行续接
    sc.apply_entry_step_effects();
    sc
}

/// 交接一：读档（槽载入动画完成 → mode 3）。仅 `mode==Map && slot_loaded` 且存档
/// 可读时交接（返回 Some 并**消费** slot_loaded）；否则 None（信号保留，等档就绪）。
/// 跨模式续接同新游戏（音频 / 壳层附加设置行等）。
pub fn slot_scene(
    assets: &GameAssets,
    machine: &mut BootMachine,
    stores: &[(String, Vec<u8>)],
) -> Option<GameScene> {
    if machine.mode != GameMode::Map {
        return None;
    }
    let cursor = machine.slot_loaded?;
    let key = format!("MOT_L{cursor}@1");
    let (_, blob) = stores.iter().find(|(k, _)| *k == key)?;
    let save = game_core::save::parse_floor_save(blob, 0).unwrap();
    let fl = save.current_floor as usize;
    let fa = assets.floors.get(fl).cloned().flatten()?;
    let mut sc = game_core::scene::load_floor_from_save(
        &fa.maplv,
        &fa.records,
        &assets.width_table,
        &assets.tile_walk,
        assets.images.clone(),
        assets.floors.clone(),
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
    sc.preset_mot_if = stores
        .iter()
        .find(|(k, _)| k.starts_with("MOT_IF@"))
        .map(|(_, v)| v.clone());
    install_combat_tables(&mut sc);
    sc.rng = machine.rng.clone();
    // 音频状态续接（Java 同一对象：soundEnabled/track/bgm 跨模式连续；
    // dispatchModeEntry case 3 已在 machine 侧执行）
    sc.bgm = machine.bgm.clone();
    sc.shell_options = machine.shell_options.clone(); // 【STAGE-B-006】壳层附加设置行续接
    machine.slot_loaded = None;
    Some(sc)
}

/// 【BUG-041】回标题反向交接（游戏内"退出" kind 7 ⇒ scene 置 MainMenu ⇒ 换回 BootMachine）：
/// Java 是同一对象，soundEnabled / volumeLevel / SKY_WAR（含 f_int_154 水位）跨模式连续；
/// 双机实现须把这些从 GameScene 回传，否则换回的是交接前的旧值——游戏内关掉的声音回滚为开、
/// 下一次标题侧置脏会把陈旧水位写回 SKY_WAR。调用方随后照旧 `bgm.enter_title()`（dispatchModeEntry
/// case 1 音频半）。未纳入：rng / bob / 图标条帧 / 标题光标等视觉相位（另案评估，不影响状态正确性）。
/// 【STAGE-B-006】壳层附加设置行（开屏画面 / 按键震动 / 按键音）同理回传——游戏内改过的勾选不回滚。
pub fn return_to_title(machine: &mut BootMachine, sc: &GameScene) {
    machine.bgm = sc.bgm.clone();
    machine.apply_global(sc.global_snapshot()); // 场景侧已写穿 ⇒ 不置脏
    machine.shell_options = sc.shell_options.clone(); // 【STAGE-B-006】
}

/// 槽存档写穿（win 壳 step 存档块迁入；BUG-018 upsert 口径）：scene 侧
/// `saved_slot_blobs` 相对已落盘指纹 `persisted` 变化 ⇒ upsert `MOT_L{idx}@1` + 整表
/// `MOT_IF@1`，回写 `sc.preset_mot_if`（mode7/8 每次进入的重读源）。返回 true=stores
/// 已变（调用方 `Storage::persist`）；无变化=false（幂等）。
pub fn sync_scene_saves(
    sc: &mut GameScene,
    stores: &mut Vec<(String, Vec<u8>)>,
    persisted: &mut [Option<Vec<u8>>; 6],
) -> bool {
    if sc.saved_slot_blobs == *persisted {
        return false;
    }
    for idx in 0..6 {
        if let Some(blob) = &sc.saved_slot_blobs[idx] {
            let key = format!("MOT_L{idx}@1");
            match stores.iter_mut().find(|(k, _)| *k == key) {
                Some((_, v)) => *v = blob.clone(),
                None => stores.push((key, blob.clone())),
            }
        }
    }
    let mot_if = crate::rms::serialize_slot_if(&sc.slot);
    stores.retain(|(k, _)| !k.starts_with("MOT_IF@"));
    stores.push(("MOT_IF@1".into(), mot_if.clone()));
    *persisted = sc.saved_slot_blobs.clone();
    sc.preset_mot_if = Some(mot_if);
    true
}

/// MOT_IF 槽表整表写入 stores（BUG-021 ②c：`load_slot_table` 读失败 parse_dirty ⇒
/// 当前槽表整表落盘，Java delete+addRecord 同终态 a.java:9026-9052）。返回写入 blob
/// （调用方同步 Game 侧 preset_mot_if 重读源）。
pub fn put_slot_if(stores: &mut Vec<(String, Vec<u8>)>, slot: &game_core::menu_family::SlotSelect) -> Vec<u8> {
    let mot_if = crate::rms::serialize_slot_if(slot);
    stores.retain(|(k, _)| !k.starts_with("MOT_IF@"));
    stores.push(("MOT_IF@1".into(), mot_if.clone()));
    mot_if
}

/// SKY_WAR 全局存档写入 stores（saveGlobalProgress 收口；单条覆写）。
pub fn put_global(stores: &mut Vec<(String, Vec<u8>)>, g: &game_core::save::GlobalSave) {
    let blob = game_core::save::serialize_global_save(g);
    stores.retain(|(k, _)| !k.starts_with("SKY_WAR@"));
    stores.push(("SKY_WAR@1".into(), blob));
}

/// 启动装载 SKY_WAR（saveGlobalProgress 对偶，a.java:4458-4478）：损坏/缺失 ⇒
/// catch 语义（default_7 ⇒ [1]=true）+ 置脏回写（4477-4478）。
pub fn apply_stored_global(machine: &mut BootMachine, stores: &[(String, Vec<u8>)]) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::DirAssetSource;
    use game_core::paint::BootMachine;

    fn raw() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw")
    }

    fn machine(src: &dyn game_platform::AssetSource) -> BootMachine {
        let png = |n: &str| {
            game_core::render::ArgbImage::from_decoded_png(
                game_data::decode_png(&src.read(n).unwrap()).unwrap(),
            )
        };
        BootMachine::new(
            png("l0.png"),
            png("l1.png"),
            load_container(src, "sflogo", 8),
            load_container(src, "ui", 25),
            load_container(src, "menu", 2),
            load_container(src, "mapbg", 1)[0].clone(),
            load_container(src, "intro", 2),
            load_container(src, "end", 1),
            load_container(src, "load", 2),
        )
    }

    #[test]
    fn new_game_scene_cold_start_and_consumes_signal() {
        let src = DirAssetSource::new(&raw());
        let assets = load_game_assets(&src);
        let mut m = machine(&src);
        m.new_game_ready = true;
        let sc = new_game_scene(&assets, &mut m);
        assert_eq!((sc.floor, sc.min_floor, sc.max_floor), (51, 51, 51), "序章层 51");
        assert_eq!((sc.player.hp, sc.player.atk, sc.player.def), (300, 10, 10), "冷启动统计 a.java:3496-3506");
        assert!(!m.new_game_ready, "交接信号一次性消费（回标题后不得再开新游戏）");
        assert!(!sc.enemy_base_hp.is_empty(), "敌人数值表已装配");
    }

    /// oracle 预置存档夹具（reference/ 只读；格式 `键=hex`，win 壳 preset_stores 同口径）。
    fn preset_stores(name: &str) -> Vec<(String, Vec<u8>)> {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../reference/oracle/scenarios").join(name);
        std::fs::read_to_string(p)
            .unwrap()
            .lines()
            .filter_map(|l| l.split_once('='))
            .map(|(k, hex)| {
                let h = hex.trim();
                let bytes = (0..h.len() / 2).map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).unwrap()).collect();
                (k.trim().to_string(), bytes)
            })
            .collect()
    }

    #[test]
    fn slot_scene_requires_save_and_consumes_signal_only_on_handoff() {
        let src = DirAssetSource::new(&raw());
        let assets = load_game_assets(&src);
        let mut m = machine(&src);
        m.slot_loaded = Some(0);
        let stores = preset_stores("preset-floor1.preset");
        assert!(slot_scene(&assets, &mut m, &stores).is_none(), "未到 Map（读条未完）=不交接");
        m.mode = game_core::enums::GameMode::Map; // 读条完成（run case 2 → mode 3）
        assert!(slot_scene(&assets, &mut m, &[]).is_none(), "无档=不交接");
        assert_eq!(m.slot_loaded, Some(0), "未交接不消费（win 同口径：等档就绪）");
        let sc = slot_scene(&assets, &mut m, &stores).expect("有档=交接");
        assert!(sc.floor >= 1, "读档还原层号（实得 {}）", sc.floor);
        assert_eq!(m.slot_loaded, None, "交接即消费（回标题后新游戏不得误走槽分支）");
    }

    /// 【BUG-041】回标题反向交接：游戏内关掉的声音、游戏内抬升的全局存档（SKY_WAR）回传 BootMachine
    /// ——Java 是同一对象（soundEnabled / f_int_154 跨模式连续）；旧行为沿用交接前的 BootMachine 旧值。
    #[test]
    fn return_to_title_carries_sound_and_global_back() {
        let src = DirAssetSource::new(&raw());
        let assets = load_game_assets(&src);
        let mut m = machine(&src);
        m.new_game_ready = true;
        assert!(m.bgm.sound_enabled, "前置：交接前开声");
        let mut sc = new_game_scene(&assets, &mut m);
        sc.bgm.toggle_sound(); // 游戏内设置关声音
        let mut g = sc.global_snapshot();
        g.max_floor_watermark += 7; // 游戏内抬升水位
        sc.apply_global(g);
        return_to_title(&mut m, &sc);
        assert!(!m.bgm.sound_enabled, "声音开关回传（不回滚为开）");
        assert_eq!(m.global_snapshot(), sc.global_snapshot(), "全局存档回传（不留陈旧水位）");
        assert!(!m.take_global_dirty(), "回传不置脏（场景侧已写穿）");
    }

    /// 【STAGE-B-006】壳层附加设置行随交接往返：新游戏 / 读档正向带入场景，回标题反向带回（游戏内改过的值不回滚；
    /// 双机实现同 BUG-041 口径）。
    #[test]
    fn shell_options_ride_handoff_both_ways() {
        use game_core::menu_family::ShellOption;
        let src = DirAssetSource::new(&raw());
        let assets = load_game_assets(&src);
        let rows: Vec<ShellOption> = [(1u8, "开屏画面", false), (2, "按键震动", true), (3, "按键音", true)]
            .into_iter()
            .map(|(id, l, checked)| ShellOption { id, label: l.encode_utf16().collect(), checked })
            .collect();
        let mut m = machine(&src);
        m.shell_options = rows.clone();
        m.new_game_ready = true;
        let mut sc = new_game_scene(&assets, &mut m);
        assert_eq!(sc.shell_options, rows, "新游戏：带入场景");
        assert_eq!(m.shell_options, rows, "新游戏：正向是拷贝，机器侧不变");
        sc.shell_options[1].checked = false; // 游戏内关掉"按键震动"
        return_to_title(&mut m, &sc);
        assert_eq!(m.shell_options, sc.shell_options, "回标题：带回（不回滚为交接前的值）");
        assert!(!m.shell_options[1].checked);
        m.slot_loaded = Some(0);
        m.mode = game_core::enums::GameMode::Map; // 读条完成
        let want = m.shell_options.clone();
        let sc2 = slot_scene(&assets, &mut m, &preset_stores("preset-floor1.preset")).expect("有档=交接");
        assert_eq!(sc2.shell_options, want, "读档：带入场景");
        assert_eq!(m.shell_options, want, "读档：正向是拷贝，机器侧不变");
    }

    #[test]
    fn sync_scene_saves_upserts_slot_and_mot_if_once() {
        let src = DirAssetSource::new(&raw());
        let assets = load_game_assets(&src);
        let mut m = machine(&src);
        m.new_game_ready = true;
        let mut sc = new_game_scene(&assets, &mut m);
        let mut stores = vec![("SKY_WAR@1".to_string(), vec![1, 2, 3])];
        let mut persisted: [Option<Vec<u8>>; 6] = Default::default();
        assert!(!sync_scene_saves(&mut sc, &mut stores, &mut persisted), "无新存档=不写");
        sc.saved_slot_blobs[2] = Some(vec![9, 9]);
        assert!(sync_scene_saves(&mut sc, &mut stores, &mut persisted), "新存档=写穿");
        assert!(stores.iter().any(|(k, v)| k == "MOT_L2@1" && v == &vec![9, 9]), "槽 2 upsert");
        assert_eq!(stores.iter().filter(|(k, _)| k.starts_with("MOT_IF@")).count(), 1, "MOT_IF 整表一条");
        assert!(stores.iter().any(|(k, _)| k == "SKY_WAR@1"), "无关键不动");
        assert!(sc.preset_mot_if.is_some(), "回写 preset_mot_if（mode7/8 重读源）");
        assert!(!sync_scene_saves(&mut sc, &mut stores, &mut persisted), "指纹已同步=幂等");
    }

    #[test]
    fn global_save_roundtrip_and_corrupt_resets_dirty() {
        let src = DirAssetSource::new(&raw());
        let mut m = machine(&src);
        apply_stored_global(&mut m, &[("SKY_WAR@1".into(), vec![0xFF])]);
        assert!(m.take_global_dirty(), "损坏 ⇒ default_7 + 置脏回写（a.java:4477-4478）");
        let mut stores = Vec::new();
        put_global(&mut stores, &m.global_snapshot());
        put_global(&mut stores, &m.global_snapshot());
        assert_eq!(stores.iter().filter(|(k, _)| k.starts_with("SKY_WAR@")).count(), 1, "SKY_WAR 单条覆写");
        let mut m2 = machine(&src);
        apply_stored_global(&mut m2, &stores);
        assert!(!m2.take_global_dirty(), "合法存档 ⇒ 不置脏");
    }

    #[test]
    fn put_slot_if_replaces_whole_table_record() {
        // MOT_IF 读失败回写（BUG-021 ②c，markSlotEntry(-1) 对偶）：整表单条覆写
        let mut stores = vec![("MOT_IF@1".to_string(), vec![0xEE]), ("MOT_L0@1".to_string(), vec![1])];
        let slot = game_core::menu_family::SlotSelect::new();
        let blob = put_slot_if(&mut stores, &slot);
        assert_eq!(stores.iter().filter(|(k, _)| k.starts_with("MOT_IF@")).count(), 1, "MOT_IF 单条");
        assert!(stores.iter().any(|(k, v)| k == "MOT_IF@1" && *v == blob), "整表=当前槽表序列化");
        assert!(stores.iter().any(|(k, _)| k == "MOT_L0@1"), "槽存档不动");
    }
}
