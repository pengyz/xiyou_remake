//! 临时探针（迭代 33 判定用，用后即删）：floor2 type78 告示 (11,7) 的
//! 载入期格槽注册与 case16 触发行为。
use game_core::render::ArgbImage;
use game_core::scene::GameScene;
use game_core::sprite_spawn::{spawn_param_and_visibility, DoorTables};
use game_data::sprite::SpriteTable;

pub fn load_container(name: &str, count: usize) -> Vec<ArgbImage> {
    let data = std::fs::read(repo().join("assets/raw").join(name)).unwrap();
    game_data::PackedPng::parse(&data, count)
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

pub fn repo() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}


#[test]
fn probe_floor2_type78() {
    let repo_path = repo();
    let maplv = std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/maplv2.bin")).unwrap();
    let sprite = std::fs::read(repo_path.join("crates/game-oracle/tests/fixtures/sprite2.bin")).unwrap();
    let sp = SpriteTable::parse(&sprite).unwrap();
    let doors = door_tables();
    let records: Vec<(u8, u16, u16, i16, bool)> = sp
        .records
        .iter()
        .map(|rec| {
            let (param, hide) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            (rec.type_code, rec.x, rec.y, param, !hide)
        })
        .collect();
    for r in records.iter().filter(|r| r.0 == 78) {
        println!("record type78 px=({}, {}) param={} visible={}", r.1, r.2, r.3, r.4);
    }

    let mapbg = load_container("mapbg", 1);
    let map = load_container("map", 12);
    let actor_c = load_container("actor", 4);
    let sptmap = load_container("sptmap", 13);
    let sptprop = load_container("sptprop", 23);
    let sptarm = load_container("sptarm", 10);
    let sptenemy1 = load_container("sptenemy1", 20);
    let sptenemy2 = load_container("sptenemy2", 20);
    let ui = load_container("ui", 25);
    let images = game_core::scene::SceneImages {
        backdrop: mapbg[0].clone(),
        shadow: sptprop[0].clone(),
        actor: actor_c[0].clone(),
        actor_face: actor_c[1].clone(),
        stair_icons: (map[1].clone(), map[2].clone()),
        open_anim: map[9].clone(),
        opened_door: sptmap[12].clone(),
        entity: game_core::scene::SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2),
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
    };

    let mut scene = GameScene::load_floor(
        &maplv,
        320,
        224,
        &records,
        &width_table(),
        &tile_walkability(),
        images,
        vec![None; 56],
    )
    .unwrap();

    for e in 0..scene.entities.count {
        let t = scene.entities.entity_type[e];
        if t == 78 || t == 77 {
            println!(
                "spawned type{} idx={} px=({},{}) removed={} solid={}",
                t, e,
                scene.entities.pixel_x[e], scene.entities.pixel_y[e],
                scene.entities.removed[e], scene.entities.solid[e]
            );
        }
    }
    let ct = scene.grid.cell_type[7][11] as usize;
    println!(
        "grid (11,7): cell_type={} capacity={} slots={:?}",
        ct, scene.grid.capacity[ct], scene.grid.slots.get(ct)
    );
    // 与 Java trace FLD 的 ARR sha 对比（行优先展平）
    let flat_ct: Vec<u8> = scene.grid.cell_type.iter().flatten().copied().collect();
    println!(
        "cellTypeGrid ARR[{}] sha computed in test (见 Java trace FLD 对应 ARR 行)",
        flat_ct.len()
    );
    let mut flat_slots: Vec<u8> = Vec::new();
    for pool in &scene.grid.slots {
        flat_slots.extend_from_slice(pool);
    }
    println!("slots 总长 {}", flat_slots.len());
    scene.facing = 2;
    let ok = scene.try_step(2);
    println!(
        "try_step(2): ok={} mode={} px=({},{}) overlay={} kind={} merchant_entity={}",
        ok, scene.game_mode, scene.view.player_px, scene.view.player_py,
        scene.overlay_active, scene.overlay_kind, scene.merchant_entity
    );
}
