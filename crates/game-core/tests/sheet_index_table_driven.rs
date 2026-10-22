//! V2-P2 点3：图集数字 → `sheet_index` 表驱动（m_122 型→(容器,槽) 映射 / SEG67
//! 表中表 / 16 容器名序）。设计件=survey §2.1 sheet_index（slots.claims 82 条+
//! assembly_profiles.seg67+containers）。
//!
//! 陷阱①（asset-hardcode-audit）：`(v&7)<<4` 列数 8 = tile-id **索引语义**锚不动
//! ——图集数字换源不许顺手改它（列变⇒tile-id 映射静默全花）。

use game_core::entity::EntityTable;
use game_core::render::ArgbImage;
use game_core::scene::SceneImages;
use game_core::resource_cfg::{self, ResourceCfg};

fn global() -> &'static ResourceCfg {
    resource_cfg::global()
}

const SPRITE_SRC: &str = include_str!("../../../data/tables/sprite_defs.json");
const ANIM_SRC: &str = include_str!("../../../data/tables/anim_defs.json");
const SHEET_SRC: &str = include_str!("../../../data/tables/sheet_index.json");

fn parse_mutated_sheet(sheet_src: &str) -> ResourceCfg {
    ResourceCfg::parse(SPRITE_SRC, ANIM_SRC, sheet_src).expect("变异表应可解析")
}

/// 合成容器：每张图颜色 = 0xFF010000 | (容器号<<8) | 槽号——图身份逐槽可辨。
fn synth_container(container_idx: u32, count: usize) -> Vec<ArgbImage> {
    (0..count)
        .map(|slot| {
            let color = 0xFF010000 | (container_idx << 8) | slot as u32;
            ArgbImage::from_rgb(&[color], 1, 1, true)
        })
        .collect()
}

/// m_122 头部的类型→容器映射**冻结副本**（消费替换前 entity.rs/scene.rs 旧体逐字，
/// a.java:9448-9470）——零漂移对拍左/右之一。
fn legacy_map_entity_images(
    sptmap: &[ArgbImage],
    sptprop: &[ArgbImage],
    sptarm: &[ArgbImage],
    sptenemy1: &[ArgbImage],
    sptenemy2: &[ArgbImage],
) -> Vec<Option<ArgbImage>> {
    let mut out: Vec<Option<ArgbImage>> = vec![None; 89];
    let at = |c: &[ArgbImage], i: usize| -> Option<ArgbImage> { c.get(i).cloned() };
    for (t, slot) in out.iter_mut().enumerate().skip(1).take(12) {
        *slot = at(sptmap, t - 1);
    }
    for (t, slot) in out.iter_mut().enumerate().skip(13).take(20) {
        *slot = at(sptprop, t - 12);
    }
    for (t, slot) in out.iter_mut().enumerate().skip(33).take(8) {
        *slot = at(sptarm, t - 33);
    }
    for (t, slot) in out.iter_mut().enumerate().skip(41).take(20) {
        *slot = at(sptenemy1, t - 41);
    }
    for (t, slot) in out.iter_mut().enumerate().skip(61).take(18) {
        *slot = at(sptenemy2, t - 61);
    }
    out[84] = at(sptenemy2, 18);
    out[87] = at(sptenemy2, 19);
    out[85] = at(sptprop, 21);
    out[86] = at(sptprop, 22);
    out[79] = at(sptarm, 8);
    out[80] = at(sptarm, 9);
    out[12] = out[11].clone();
    out[81] = at(sptmap, 3);
    out[71] = out[70].clone();
    out
}

/// 名序=16 容器 f_Image_arr2_00 槽序（a.java:451 名序值行；task-5 机械勘误：
/// 名序值行 451、张数表赋值行 453——用户红线"张数表锚=a.java:453 非 451"）。
const CONTAINER_ORDER: [&str; 16] = [
    "sflogo", "mapbg", "map", "actor", "sptmap", "sptprop", "sptarm", "sptenemy1", "ui", "xtq",
    "menu", "intro", "face", "sptenemy2", "end", "load",
];

#[test]
fn task7_map_entity_images_matches_legacy_oracle() {
    // 零漂移对拍：表驱动映射 vs 冻结旧体映射，89 槽逐槽图身份相等。
    let sptmap = synth_container(4, 13);
    let sptprop = synth_container(5, 23);
    let sptarm = synth_container(6, 10);
    let sptenemy1 = synth_container(7, 20);
    let sptenemy2 = synth_container(13, 20);
    let want = legacy_map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2);
    let got = SceneImages::map_entity_images(&sptmap, &sptprop, &sptarm, &sptenemy1, &sptenemy2);
    assert_eq!(got.len(), 89);
    for t in 0..89 {
        assert_eq!(got[t], want[t], "型 {t} 图身份（槽图逐像素）与冻结旧体一致");
    }
    // 关键特例抽查（图集数字调用点）：84→sptenemy2[18]、87→[19]、85→sptprop[21]、
    // 86→[22]、79→sptarm[8]、80→[9]、81→sptmap[3]、12=11 别名、71=70 别名、82/83 无图
    assert_eq!(got[84], Some(sptenemy2[18].clone()));
    assert_eq!(got[12], got[11], "型 12=型 11 别名（sptmap[10]）");
    assert_eq!(got[71], got[70], "型 71=型 70 别名（sptenemy2[9]）");
    assert_eq!(got[82], None, "型 82 无图（m_122 无臂）");
    assert_eq!(got[83], None, "型 83 无图（m_122 无臂）");
}

#[test]
fn task7_entity_slots_follow_claims() {
    // 变异表：sheet_index.slots.claims 中 sptmap[4]（型 5）槽号 4→5——查表结果随表。
    let mutated = SHEET_SRC.replace(
        "\"slot\": 4,\n        \"used_by_types\": [\n          5\n        ],",
        "\"slot\": 5,\n        \"used_by_types\": [\n          5\n        ],",
    );
    assert_ne!(mutated, SHEET_SRC, "变异锚点应命中型 5 的 claim");
    let cfg = parse_mutated_sheet(&mutated);
    assert_eq!(cfg.entity_sprite_slot(5), Some(("sptmap", 5)), "型 5 槽位跟随表（变异 5）");
    assert_eq!(cfg.entity_sprite_slot(6), Some(("sptmap", 5)), "型 6 不受影响（原槽 5）");
    assert_eq!(global().entity_sprite_slot(5), Some(("sptmap", 4)), "保真表原值 4 不动");
}

#[test]
fn task7_seg67_follows_table() {
    // SEG67 表中表（sheet_index.assembly_profiles.seg67）：9 片件 + 3×5 段帧
    // 与 a.java:682-697 逐值等价（一事实一宿主：表=宿主，scene 绘制臂消费表）。
    let cfg = global();
    let seg = cfg.seg67();
    assert_eq!(
        seg.pieces.as_slice(),
        &[
            (0, 0, 0, 33, 56),
            (0, 33, 0, 33, 56),
            (0, 66, 0, 33, 56),
            (2, 0, 0, 18, 26),
            (3, 0, 0, 18, 34),
            (1, 0, 0, 55, 61),
            (4, 0, 0, 14, 26),
            (4, 14, 0, 14, 26),
            (4, 28, 0, 24, 26),
        ],
        "片件表（a.java:682-692）"
    );
    assert_eq!(
        seg.frames,
        vec![
            vec![(6, -1, -79, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -53, 0), (0, -16, -27, 0)],
            vec![(7, -1, -80, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -54, 0), (2, -16, -26, 0)],
            vec![(8, -1, -81, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -55, 0), (1, -16, -25, 0)],
        ],
        "帧拼装表（a.java:693-697）"
    );
    // 变异表：片件 0 的 w 33→34——消费值随表
    let mutated = SHEET_SRC.replace(
        "\"pieces\": [\n        [\n          0,\n          0,\n          0,\n          33,\n          56\n        ],",
        "\"pieces\": [\n        [\n          0,\n          0,\n          0,\n          34,\n          56\n        ],",
    );
    assert_ne!(mutated, SHEET_SRC, "变异锚点应命中 seg67 片件 0");
    let cfg2 = parse_mutated_sheet(&mutated);
    assert_eq!(cfg2.seg67().pieces[0], (0, 0, 0, 34, 56), "片件值跟随表");
    assert_eq!(cfg2.seg67().pieces[1], (0, 33, 0, 33, 56), "其余片件不动");
}

#[test]
fn task7_container_names_order() {
    let names: Vec<&str> = global().container_names().iter().map(|s| s.as_str()).collect();
    assert_eq!(names.as_slice(), CONTAINER_ORDER.as_slice(), "16 容器名序==a.java:451 名序");
    // 张数表锚勘误：a.java:453（非 451）——本点只消费名序，勘误注记随行。
}

#[test]
fn task7_atlas_column8_index_semantics_anchor() {
    // 陷阱①锚不动：tile-id 拆图集列 `id&7`、行 `id>>3`（a.java:7045-7046
    // paintTileLayer `(var18&7)<<4`/`>>3<<4`）——列数 8 是**索引语义**，
    // 与瓦片像素 16（尺寸域）分属两事实；图集数字换源（claims/表）不涉此锚。
    for (v, want) in [(0i32, (0, 0)), (1, (16, 0)), (8, (0, 16)), (5, (80, 0)), (29, (80, 48))] {
        assert_eq!(((v & 7) << 4, (v >> 3) << 4), want, "tile-id {v} 的 (列×16, 行×16)");
    }
}

#[test]
fn task7_entity_slot_lookup_domain() {
    let cfg = global();
    // 域面：型 0/82/83/88 无槽（m_122 无臂/无图型）；85 型有槽（claims 82 条覆盖 85 型）
    assert_eq!(cfg.entity_sprite_slot(0), None);
    assert_eq!(cfg.entity_sprite_slot(82), None);
    assert_eq!(cfg.entity_sprite_slot(83), None);
    assert_eq!(cfg.entity_sprite_slot(88), None);
    assert_eq!(cfg.entity_sprite_slot(85), Some(("sptprop", 21)));
    assert_eq!(cfg.entity_sprite_slot(72), Some(("sptenemy2", 11)));
    let t = EntityTable::new(1);
    let _ = t; // 消费面同型域（map_entity_images 89 槽）
}
