//! V2-P2 点5：一事实一宿主校验器——双写 diff=0 断言（survey §3 硬条件）。
//!
//! 任务卡原文：「校验器断言 sprite_defs.render_category==enemy_defs.sprite_ref
//! 解引用双写 diff=0」。enemy_defs 属**表设计批**（dispatch-shape-survey §4.1
//! sprite_ref 瘦身修订，不在本批——plan §0.1 lead 拍板）；本批把校验器建成
//! **四宿主对拍**并在 enemy_defs 落地时可直接加侧：
//!
//! 1. 表宿主：`sprite_defs.render_category`（真值宿主，survey §3）；
//! 2. 代码宿主：`entity::legacy_render_category_mirror`（10 臂保真冻结镜像——
//!    enemy_defs.sprite_ref 解引用值在**今日仓形**下的等价面：消费方经
//!    render_category 函数拿到的值）；
//! 3. 解引用链宿主：型→sprite_ref（sheet_index.slots.claims 型→(容器,槽)）与
//!    sprite_defs.sheet/sheet_index 同事实双写（表间双写，防 DC-9 同型漂移）；
//! 4. 邻接双写：anim_row vs ANIM_TABLE_IDX 镜像、size_dispatch 特例 vs 旧常量
//!    （width_cells 格数）、type_frame_width vs type_width_px 夹具。
//!
//! 校验器非荣誉制：`ResourceCfg::from_embedded` 装载即跑（漂移=装载失败）；
//! 校验器有牙齿自检=注入漂移必须报 diff≠0（防"断言恒真"假校验）。

use game_core::entity;
use game_core::resource_cfg::{self, ResourceCfg};

fn global() -> &'static ResourceCfg {
    resource_cfg::global()
}

const SPRITE_SRC: &str = include_str!("../../../data/tables/sprite_defs.json");
const ANIM_SRC: &str = include_str!("../../../data/tables/anim_defs.json");
const SHEET_SRC: &str = include_str!("../../../data/tables/sheet_index.json");

fn mutate_record_field(src: &str, type_id: u32, key: &str, new_val: &str) -> String {
    let anchor = format!("\"type_id\": {type_id},");
    let start = src.find(&anchor).expect("记录锚应命中");
    let key_pat = format!("\"{key}\": ");
    let kstart = src[start..].find(&key_pat).expect("字段键应命中") + start;
    let kend = src[kstart..].find(',').expect("整数值应以逗号收尾") + kstart;
    format!("{}\"{}\": {}{}", &src[..kstart], key, new_val, &src[kend..])
}

fn parse_of(sprite_src: &str) -> ResourceCfg {
    ResourceCfg::parse(sprite_src, ANIM_SRC, SHEET_SRC).expect("表应可解析")
}

#[test]
fn task7_one_fact_one_host_validator_passes() {
    // 保真面四宿主对拍全绿（双写 diff=0）
    let report = global().validate_one_fact_one_host().expect("保真表双写应 diff=0");
    assert_eq!(report, "OK", "校验器通过面");
}

#[test]
fn task7_validator_catches_render_category_drift() {
    // 注入漂移：型 44 类目 8→16（表侧动、镜像侧不动）——校验器必须报 diff≠0
    let mutated = mutate_record_field(SPRITE_SRC, 44, "render_category", "16");
    assert_ne!(mutated, SPRITE_SRC);
    let err = parse_of(&mutated)
        .validate_one_fact_one_host()
        .expect_err("render_category 双写漂移应被校验器捕获");
    assert!(err.contains("render_category"), "报告应指明漂移事实：{err}");
    assert!(err.contains("44"), "报告应指明型号：{err}");
}

#[test]
fn task7_validator_catches_anim_row_drift() {
    // 注入漂移：型 43 行号 3→4（表 vs ANIM_TABLE_IDX 镜像）
    let mutated = mutate_record_field(SPRITE_SRC, 43, "anim_row", "4");
    assert_ne!(mutated, SPRITE_SRC);
    let err = parse_of(&mutated)
        .validate_one_fact_one_host()
        .expect_err("anim_row 双写漂移应被校验器捕获");
    assert!(err.contains("anim_row"), "报告应指明漂移事实：{err}");
}

#[test]
fn task7_validator_catches_sprite_ref_drift() {
    // 注入漂移：型 5 的 claim 槽位 4→5（claims vs sprite_defs.sheet_index 表间双写）
    let mutated = SHEET_SRC.replace(
        "\"slot\": 4,\n        \"used_by_types\": [\n          5\n        ],",
        "\"slot\": 5,\n        \"used_by_types\": [\n          5\n        ],",
    );
    assert_ne!(mutated, SHEET_SRC);
    let err = ResourceCfg::parse(SPRITE_SRC, ANIM_SRC, &mutated)
        .expect("变异 sheet 应可解析")
        .validate_one_fact_one_host()
        .expect_err("sprite_ref 解引用双写漂移应被校验器捕获");
    assert!(err.contains("sprite_ref"), "报告应指明漂移事实：{err}");
}

#[test]
fn task7_validator_catches_size_dispatch_drift() {
    // 注入漂移：69 特例 96→100（size_dispatch vs 旧常量冻结值）
    let mutated = SPRITE_SRC.replace(
        "\"69\": {\n        \"frame_w\": 96,",
        "\"69\": {\n        \"frame_w\": 100,",
    );
    assert_ne!(mutated, SPRITE_SRC);
    let err = parse_of(&mutated)
        .validate_one_fact_one_host()
        .expect_err("size_dispatch 双写漂移应被校验器捕获");
    assert!(err.contains("size_dispatch"), "报告应指明漂移事实：{err}");
}

#[test]
fn task7_type_frame_width_double_write_diff_zero() {
    // type_frame_width 88 项（sprite_defs）vs type_width_px.txt 夹具——迁移双写
    // diff=0（task-5 R1 迁入值；消费面 width_table 参数仍走夹具，本测钉双写不漂）。
    let wt: Vec<i32> = include_str!("fixtures/type_width_px.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect();
    assert_eq!(wt.len(), 88, "type_width_px 88 项（域 0..87）");
    let cfg = global();
    for (t, want) in wt.iter().enumerate() {
        let got = cfg.bundle().sprite_defs[t].type_frame_width;
        assert_eq!(got, Some(*want as i64), "型 {t} type_frame_width 双写 diff=0");
    }
    // Q-003 面：型 88 无表项（type_frame_width=null）
    assert_eq!(cfg.bundle().sprite_defs[88].type_frame_width, None, "型 88 无 typeFrameWidth 项");
}
