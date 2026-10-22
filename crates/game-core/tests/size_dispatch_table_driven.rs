//! V2-P2 点2：dispatch_sprite_size 特例 → `size_dispatch` 表驱动。
//!
//! 设计件：survey §2.1 size_dispatch（default/fallback/by_type 三臂）；
//! 消费点=entity.rs dispatchSpriteSize（a.java:6300-6325）特例 69→96×32 /
//! 72→32×45 / 67→break 落 fallback。
//!
//! quirks 留痕对齐：Q-002（67/69 的 loadPackedImages(9) 加载副作用不属尺寸语义，
//! 留码）/ Q-003（型 88 无 typeFrameWidth 项且无图 ⇒ fallback 32×32 不读表）。

use game_core::entity::{self, EntityTable};
use game_core::resource_cfg::{self, ResourceCfg};

fn width_table() -> Vec<i32> {
    include_str!("fixtures/type_width_px.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

fn global() -> &'static ResourceCfg {
    resource_cfg::global()
}

const SPRITE_SRC: &str = include_str!("../../../data/tables/sprite_defs.json");
const ANIM_SRC: &str = include_str!("../../../data/tables/anim_defs.json");
const SHEET_SRC: &str = include_str!("../../../data/tables/sheet_index.json");

fn parse_mutated(sprite_src: &str) -> ResourceCfg {
    ResourceCfg::parse(sprite_src, ANIM_SRC, SHEET_SRC).expect("变异表应可解析")
}

/// 走消费层装载面取尺寸产物（与生产同路：entity::dispatch_sprite_size 委托同一方法）。
fn dispatch(cfg: &ResourceCfg, type_id: i32, wt: &[i32], img_h: Option<i32>) -> (i32, i32) {
    let mut t = EntityTable::new(8);
    cfg.dispatch_sprite_size_into(type_id, 0, &mut t, wt, img_h);
    (t.items[0].sprite_w, t.items[0].sprite_h)
}

#[test]
fn task7_size_dispatch_specials_table_driven() {
    let wt = width_table();
    let cfg = global();
    // 特例三型（size_dispatch.by_type，a.java:6304-6314）
    assert_eq!(cfg.size_special(69), Some((96, 32)), "69→96×32");
    assert_eq!(cfg.size_special(72), Some((32, 45)), "72→32×45");
    assert_eq!(cfg.size_special(67), Some((32, 32)), "67→break 落 fallback（32×32）");
    assert_eq!(cfg.size_fallback(), (32, 32), "fallback 臂 32×32");

    // 装载产物逐型断言（与旧常量逐值等价——双写 diff=0 面）
    assert_eq!(dispatch(cfg, 69, &wt, None), (96, 32));
    assert_eq!(dispatch(cfg, 69, &wt, Some(77)), (96, 32), "69 特例盖过 default 臂（图高无关）");
    assert_eq!(dispatch(cfg, 72, &wt, None), (32, 45));
    assert_eq!(dispatch(cfg, 72, &wt, Some(77)), (32, 45));
    assert_eq!(dispatch(cfg, 67, &wt, None), (32, 32));
    assert_eq!(dispatch(cfg, 67, &wt, Some(77)), (32, 32), "67 break 臂不走 default（图高无关）");
    // default 臂：type_frame_width 表宽 + 运行时图高（a.java:6316-6319）
    assert_eq!(dispatch(cfg, 43, &wt, Some(77)), (wt[43], 77), "default=表宽×图高");
    // fallback：default 臂 var3==null 落空（a.java:6323-6324）
    assert_eq!(dispatch(cfg, 43, &wt, None), (32, 32), "无图落 fallback");
}

#[test]
fn task7_size_dispatch_values_track_table() {
    // 变异表：69 特例 96×32 → 100×33（"69" 键仅在 size_dispatch.by_type 出现）
    let mutated_src = SPRITE_SRC
        .replace("\"69\": {\n        \"frame_w\": 96,", "\"69\": {\n        \"frame_w\": 100,")
        .replace("\"69\": {\n        \"frame_w\": 100,\n        \"frame_h\": 32,", "\"69\": {\n        \"frame_w\": 100,\n        \"frame_h\": 33,");
    assert_ne!(mutated_src, SPRITE_SRC, "变异锚点应命中 size_dispatch.by_type.69");
    let cfg = parse_mutated(&mutated_src);
    let wt = width_table();
    assert_eq!(cfg.size_special(69), Some((100, 33)), "特例值跟随表");
    assert_eq!(dispatch(&cfg, 69, &wt, None), (100, 33), "装载产物跟随表（变异 100×33）");
    assert_eq!(dispatch(&cfg, 72, &wt, None), (32, 45), "未变异型不受影响");
}

#[test]
fn task7_size_dispatch_load_side_effect() {
    // Q-002：case 67/69 的 loadPackedImages(9) 是**加载副作用**，非尺寸语义——
    // 规则形只承载尺寸值，副作用锚以 load_packed_images 字段留痕，消费侧不执行
    // （game-core 无 I/O；Rust 侧装载在 shell 构造期全量完成）。
    let cfg = global();
    let q = cfg
        .quirks()
        .iter()
        .find(|q| q.quirk_id == "Q-002")
        .expect("quirks_in_code 应留痕 Q-002");
    assert_eq!(q.red_test_id, "task7-size-dispatch-load-side-effect");
    assert!(q.src.contains("6304"), "Q-002 锚 a.java:6304-6307");
}

#[test]
fn task7_type88_fallback() {
    // Q-003：typeFrameWidth 仅 88 项（型 0..87），型 88 无表项且无图——
    // dispatchSpriteSize fallback 32×32（不读表）。
    let wt = width_table();
    let cfg = global();
    assert_eq!(wt.len(), 88, "type_width_px 88 项（域 0..87）");
    assert_eq!(dispatch(cfg, 88, &wt, None), (32, 32), "型 88 走 fallback 32×32");
    let q = cfg.quirks().iter().find(|q| q.quirk_id == "Q-003").expect("quirks_in_code 应留痕 Q-003");
    assert_eq!(q.red_test_id, "task7-type88-fallback");
}

#[test]
fn task7_width_cells_specials_match_size_dispatch() {
    // 双写校验（点 5 预热）：width_cells 的 69→96/72→32 特例与 size_dispatch
    // 表产物**格数** diff=0（格槽宽=dispatchSpriteSize 产物>>5，a.java:6392；
    // 67 的 px 源不同（42 vs 32）但同落 1 格——断言锚=格数）。
    let wt = width_table();
    let cfg = global();
    for (ty, want_cells) in [(69, 3), (72, 1), (67, 1)] {
        let (w, _) = cfg.size_special(ty).expect("特例在表");
        assert_eq!(entity::width_cells(ty, &wt), (w >> 5).max(1), "型 {ty} 格槽宽=表产物>>5");
        assert_eq!(entity::width_cells(ty, &wt), want_cells, "型 {ty} 格槽宽锚值");
    }
    // default 型：width_cells 读 type_width_px 表（与 sprite_defs.type_frame_width 同值）
    assert_eq!(entity::width_cells(43, &wt), (wt[43] >> 5).max(1));
}
