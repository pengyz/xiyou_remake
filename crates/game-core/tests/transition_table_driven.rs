//! V2-P2 点1：advance_frames 特例行 → `transition_overrides` 表驱动 + 状态迁移三拍 RED 测。
//!
//! 设计件：docs/spec/resource-config-survey.md v2 §2.1（顺序契约）/ §4（R3 状态条件臂）。
//! 三拍 = **生成拍**（spawn：查表初始化 dispatchAnimFields，a.java:6327-6330）→
//! **迁移拍**（transition_overrides 状态迁移覆写）→ **渲染拍**（ANIM_OFFSET_TABLE 帧偏移
//! 解析出实际外观）。逐拍断言防"生成时即带 row8 外观"陷阱（R3 反例 1）。
//!
//! **顺序契约钉死**（§2.1）：查表初始化 → 状态迁移覆写。反了 quirk 静默失效
//! （R3 反例 2）——型 12 变形臂的 `redispatch: [dispatchSpriteSize, dispatchAnimFields]`
//! 必须落在 type_id 写入之后、其余覆写字段之前（a.java:5450-5457 写序）：
//! 若重派排在覆写之后，dispatchAnimFields 会把 anim_row=9 吞回查表值 3。

use game_core::entity::{self, CellGrid, EntityTable, ANIM_OFFSET_TABLE, ANIM_TABLE_IDX};
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

/// 嵌入保真表原文（data/tables/sprite_defs.json）——变异表测试的底本。
const SPRITE_SRC: &str = include_str!("../../../data/tables/sprite_defs.json");
const ANIM_SRC: &str = include_str!("../../../data/tables/anim_defs.json");
const SHEET_SRC: &str = include_str!("../../../data/tables/sheet_index.json");

fn parse_mutated(sprite_src: &str) -> ResourceCfg {
    ResourceCfg::parse(sprite_src, ANIM_SRC, SHEET_SRC).expect("变异表应可解析")
}

// ───────────────────────────────────────── 三拍：mark_des_removed（state=2 行 8 族）

#[test]
fn task7_three_beat_mark_des_removed_row8() {
    let wt = width_table();
    let cfg = global();
    let mut t = EntityTable::new(64);
    let mut g = CellGrid::new(13, 13);
    // 型 44（cat 8）走 mark_des_removed "2|8|16|32" 臂 → (27,29) 行 8
    let e = entity::spawn(&mut t, &mut g, 44, 32, 32, 0, &wt, None);

    // ── 生成拍（顺序契约前半：查表初始化 dispatchAnimFields）──
    assert_eq!(t.items[e].anim_row, cfg.anim_row(44), "生成拍=查表初始化（sprite_defs.anim_row）");
    assert_eq!(t.items[e].anim_row, ANIM_TABLE_IDX[44] as i32, "查表初始化=animRowByType 保真值");
    assert_ne!(t.items[e].anim_row, 8, "防'生成时即带 row8 外观'陷阱（R3 反例 1）");
    assert_eq!(t.items[e].anim_frame, 0);
    assert_eq!(t.items[e].state, 0);
    assert_eq!((t.items[e].sprite_w, t.items[e].sprite_h), (32, 32), "生成拍=dispatchSpriteSize fallback");

    // ── 迁移拍（transition_overrides mark_des_removed，a.java:6332-6347）──
    entity::remove_des(&mut t, e);
    assert_eq!(t.items[e].state, 2, "迁移拍=state 2");
    assert_eq!(t.items[e].anim_row, 8, "迁移拍=表规则 row8（覆写胜过初始化）");
    assert_eq!((t.items[e].sprite_w, t.items[e].sprite_h), (27, 29), "迁移拍=表规则 27×29");
    assert_eq!(t.items[e].anim_frame, 0);

    // ── 渲染拍（帧偏移 = sprite_w × ANIM_OFFSET_TABLE[row][frame]）──
    let row = t.items[e].anim_row as usize;
    let frame = t.items[e].anim_frame as usize;
    assert_eq!(ANIM_OFFSET_TABLE[row], &[4, 3, 2, 1, 0], "渲染拍=行 8 帧序");
    assert_eq!(t.items[e].sprite_w * ANIM_OFFSET_TABLE[row][frame], 27 * 4, "渲染拍=行 8 首帧偏移");
}

#[test]
fn task7_three_beat_mark_removed_category_arms() {
    let wt = width_table();
    let mut t = EntityTable::new(64);
    let mut g = CellGrid::new(13, 13);
    // mark_removed（state=1）按 render_category 分派（a.java:6349-6387）
    let cases: [(i32, i32, (i32, i32)); 4] = [
        (2, 6, (45, 56)),  // cat 1 · 型 1|2|3 臂 → 行 6
        (11, 2, (45, 57)), // cat 1 · 型 11 臂 → 行 2
        (26, 10, (45, 56)), // cat 2|4 臂 → 行 10
        (44, 6, (27, 29)),  // cat 8|16|32 臂 → 行 6
    ];
    for (ty, want_row, want_dims) in cases {
        let e = entity::spawn(&mut t, &mut g, ty, 32, 32, 0, &wt, None);
        // 生成拍：查表初始化——绝不是迁移拍的覆写值
        assert_ne!(t.items[e].anim_row, want_row, "型 {ty} 生成拍不得即带迁移外观");
        entity::remove(&mut t, e);
        assert_eq!(t.items[e].state, 1, "型 {ty} 迁移拍=state 1");
        assert_eq!(t.items[e].anim_row, want_row, "型 {ty} 迁移拍=表规则行");
        assert_eq!((t.items[e].sprite_w, t.items[e].sprite_h), want_dims, "型 {ty} 迁移拍=表规则尺寸");
        assert_eq!(t.items[e].anim_frame, 0);
    }
}

#[test]
fn task7_three_beat_mark_removed_cat1_otherwise_noop() {
    let wt = width_table();
    let mut t = EntityTable::new(64);
    let mut g = CellGrid::new(13, 13);
    // cat 1 非 1|2|3/11 型（型 5）→ by_type.otherwise = noop（视觉字段不动）
    let e = entity::spawn(&mut t, &mut g, 5, 32, 32, 0, &wt, None);
    let before = (t.items[e].sprite_w, t.items[e].sprite_h, t.items[e].anim_row, t.items[e].anim_frame);
    assert!(entity::remove(&mut t, e), "cat 1 臂 build_minimap=true（a.java:6370）");
    assert_eq!(t.items[e].state, 1);
    assert_eq!(
        (t.items[e].sprite_w, t.items[e].sprite_h, t.items[e].anim_row, t.items[e].anim_frame),
        before,
        "otherwise=noop：视觉字段不动"
    );
}

#[test]
fn task7_mark_des_removed_java_switch_no_default() {
    // a.java:6332-6347 markEntityDesRemoved 的 switch **无 default 臂**：
    // 仅 case 1（return）与 case 2/8/16/32（行 8 覆写）——cat 0/4 不写视觉字段。
    // （2026-10-22 消费替换发现旧 Rust `_` 臂对 cat 0/4 过写 row8，
    //  与 a.java 不符；表规则 by_category="2|8|16|32"+otherwise=noop 为机械实锚。）
    let wt = width_table();
    let mut t = EntityTable::new(64);
    let mut g = CellGrid::new(13, 13);
    let e4 = entity::spawn(&mut t, &mut g, 36, 32, 32, 0, &wt, None); // 型 36 = cat 4
    let before = (t.items[e4].sprite_w, t.items[e4].sprite_h, t.items[e4].anim_row, t.items[e4].anim_frame);
    entity::remove_des(&mut t, e4);
    assert_eq!(t.items[e4].state, 2, "state 写入不受类目影响（a.java:6333）");
    assert_eq!(
        (t.items[e4].sprite_w, t.items[e4].sprite_h, t.items[e4].anim_row, t.items[e4].anim_frame),
        before,
        "cat 4 不在 switch case 2/8/16/32 ⇒ 视觉字段不动"
    );
}

// ───────────────────────────────────────── 三拍：bump_reveal（显形行 9 路径）

#[test]
fn task7_three_beat_type6_reveal() {
    // 型 6 隐形实体 bump 显形（a.java:5410-5417）：visible=true → state=2 → anim_row=9
    let wt = width_table();
    let cfg = global();
    let mut t = EntityTable::new(64);
    let mut g = CellGrid::new(13, 13);
    let e = entity::spawn(&mut t, &mut g, 6, 32, 32, 0, &wt, None);
    t.items[e].visible = false; // m_122 特例分支：型 6 生成即隐形
    let gen_row = t.items[e].anim_row;
    assert_eq!(gen_row, cfg.anim_row(6), "生成拍=查表初始化");
    assert_ne!(gen_row, 9, "生成拍不得即带显形行 9 外观");

    let out = cfg.apply_bump_reveal(&mut t, e, &wt, None);
    assert!(out.applied, "型 6 命中 bump_reveal 规则");
    assert!(!out.block_step, "型 6 不阻挡（continue 续扫，a.java:5417）");
    assert_eq!(t.items[e].visible, true, "迁移拍=visible（a.java:5413）");
    assert_eq!(t.items[e].state, 2, "迁移拍=state（a.java:5414）");
    assert_eq!(t.items[e].anim_row, 9, "迁移拍=显形行 9（a.java:5415）");
    // 渲染拍：行 9 帧序 [2,1,0]，首帧偏移 = 2×sprite_w
    let row = t.items[e].anim_row as usize;
    assert_eq!(ANIM_OFFSET_TABLE[row], &[2, 1, 0], "渲染拍=行 9 帧序");
    assert_eq!(t.items[e].sprite_w * ANIM_OFFSET_TABLE[row][0], t.items[e].sprite_w * 2);
}

#[test]
fn task7_three_beat_t12_morph() {
    // 型 12 撞击墙 bump 变形（a.java:5449-5457）：
    // type=11 → dispatchSpriteSize(11) → dispatchAnimFields(11) → state=2 → anim_row=9 → visible
    // ——**查表初始化（重派）夹在 type 写入与覆写字段之间**（顺序契约）。
    let wt = width_table();
    let cfg = global();
    let mut t = EntityTable::new(64);
    let mut g = CellGrid::new(13, 13);
    let e = entity::spawn(&mut t, &mut g, 12, 32, 32, 0, &wt, None);
    let gen_row = t.items[e].anim_row;
    assert_eq!(gen_row, cfg.anim_row(12), "生成拍=查表初始化（型 12 行）");
    assert_ne!(gen_row, 9, "生成拍不得即带行 9 外观");

    let img_h = Some(32); // 型 11 运行时图高（Q-001：重派需运行时图高，规则形装不下）
    let out = cfg.apply_bump_reveal(&mut t, e, &wt, img_h);
    assert!(out.applied);
    assert!(out.block_step, "型 12 阻挡（Q-004：var6=false）");
    assert_eq!(t.items[e].type_, 11, "迁移拍=变形为型 11");
    // 顺序契约核心断言：重派（查表初始化）先于覆写 ⇒ 最终行=9 而非查表值 3
    assert_eq!(t.items[e].anim_row, 9, "迁移拍=行 9 覆写胜过重派初始化（顺序契约）");
    assert_ne!(t.items[e].anim_row, cfg.anim_row(11), "若初始化后置会把行 9 吞回查表值（R3 反例 2）");
    assert_eq!(t.items[e].state, 2);
    assert_eq!(t.items[e].visible, true);
    // 重派尺寸按**变形后**型 11（Q-001）：default 臂 = width_table[11]×img_h
    assert_eq!(t.items[e].sprite_w, wt[11], "重派=dispatchSpriteSize(11) 表宽");
    assert_eq!(t.items[e].sprite_h, 32, "重派=运行时图高");
}

// ───────────────────────────────────────── 顺序契约（变异表）

#[test]
fn task7_order_contract_init_then_override() {
    // 变异表：mark_des_removed 规则 anim_row 8→7（与生成拍查表值不同）。
    // 覆写必须**后于**初始化生效：最终行=7；顺序反了会被初始化值吞掉。
    let mutated_src = SPRITE_SRC.replace("\"anim_row\": 8,", "\"anim_row\": 7,");
    assert_ne!(mutated_src, SPRITE_SRC, "变异锚点应唯一命中 mark_des_removed 臂");
    let cfg = parse_mutated(&mutated_src);
    let wt = width_table();
    let mut t = EntityTable::new(64);
    let mut g = CellGrid::new(13, 13);
    let e = entity::spawn(&mut t, &mut g, 44, 32, 32, 0, &wt, None);
    cfg.apply_mark_des_removed(&mut t, e);
    assert_eq!(t.items[e].anim_row, 7, "迁移值跟随表（变异行 7）——证明消费自表而非常量");
    assert_ne!(t.items[e].anim_row, 8, "旧行 8 不得残留");
    assert_eq!(t.items[e].anim_row, 7, "亦不得被查表初始化值 3 吞掉（顺序契约）");
}

#[test]
fn task7_transition_values_track_table() {
    // 变异表：mark_removed cat "2|4" 臂 anim_row 10→4 且尺寸 45×56→46×57。
    let mutated_src = SPRITE_SRC
        .replace("\"anim_row\": 10,", "\"anim_row\": 4,")
        .replace("\"sprite_w\": 45,\n          \"sprite_h\": 56,\n          \"anim_row\": 4,", "\"sprite_w\": 46,\n          \"sprite_h\": 57,\n          \"anim_row\": 4,");
    let cfg = parse_mutated(&mutated_src);
    let wt = width_table();
    let mut t = EntityTable::new(64);
    let mut g = CellGrid::new(13, 13);
    let e = entity::spawn(&mut t, &mut g, 26, 32, 32, 0, &wt, None);
    cfg.apply_mark_removed(&mut t, e);
    assert_eq!(t.items[e].anim_row, 4, "迁移值跟随表（变异行 4）");
    assert_eq!((t.items[e].sprite_w, t.items[e].sprite_h), (46, 57), "尺寸跟随表（变异 46×57）");
}

// ───────────────────────────────────────── quirks 留痕对齐（Q-00x）

#[test]
fn task7_anim_step_ticks_global() {
    // Q-005：anim_defs.step_ticks=2（全局节拍 a.java:6833 frameCounter&1）与
    // loop_mode=cycle（a.java:6850-6851 default 臂归零）**非行级表**——保留代码语义。
    let wt = width_table();
    let mut t = EntityTable::new(64);
    let mut g = CellGrid::new(13, 13);
    let e = entity::spawn(&mut t, &mut g, 43, 32, 32, 0, &wt, None); // 行 3 = [0,1,2,1]
    // 偶数拍不动（全局 step=2 机械值，与 anim_defs 行级字段无关）
    let _ = entity::advance_frames(&mut t, &mut g, 2, &wt, &[]);
    assert_eq!(t.items[e].anim_frame, 0, "偶数拍不推进（frameCounter&1 全局节拍）");
    let _ = entity::advance_frames(&mut t, &mut g, 3, &wt, &[]);
    assert_eq!(t.items[e].anim_frame, 1, "奇数拍推进一帧");
}

#[test]
fn task7_des_anim_completes_back_to_init() {
    // advance_frames（a.java:6825-6848）帧尽：state==2 → dispatchSpriteSize+
    // dispatchAnimFields 重派 + state=0 ——顺序契约的复原拍（查表初始化再胜出）。
    let wt = width_table();
    let cfg = global();
    let mut t = EntityTable::new(64);
    let mut g = CellGrid::new(13, 13);
    let e = entity::spawn(&mut t, &mut g, 44, 32, 32, 0, &wt, None);
    entity::remove_des(&mut t, e);
    assert_eq!(t.items[e].anim_row, 8);
    // 行 8 = 5 帧：推进 4 次到帧尾，第 5 拍触发复原
    for fc in [3i64, 5, 7, 9] {
        let _ = entity::advance_frames(&mut t, &mut g, fc, &wt, &[]);
    }
    assert_eq!(t.items[e].anim_frame, 4, "帧推进至表尾");
    let detach = entity::advance_frames(&mut t, &mut g, 11, &wt, &[]);
    assert!(detach.is_empty(), "state=2 不摘槽（state=1 才 detach）");
    assert_eq!(t.items[e].state, 0, "复原拍=state 0");
    assert_eq!(t.items[e].anim_row, cfg.anim_row(44), "复原拍=查表初始化（行 8 覆写被冲掉）");
    assert_eq!(t.items[e].anim_frame, 0);
}
