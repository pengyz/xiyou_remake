//! STAGE-B-003 复核硬 2：decode-equivalence **常驻门禁**（双移植面防护）。
//!
//! 背景（docs/findings/reviews/2026-10-18-stage-b-003-asset-pipeline-review.md §2）：
//! game-data 双模（二进制解析保留 + PNG+JSON runtime）= **双移植面**
//! （gotcha_dual-porting-surfaces-drift 前科：改任一面必须对照另一面）。
//!
//! # 两面分工登记
//!
//! | 面 | 职责 | 位置 |
//! |---|---|---|
//! | 二进制解析 | oracle/工具链专用（L1 golden、repack 恒等、差分对照的被对侧） | `game_data::{packed_png,maplv,sprite}` |
//! | PNG+JSON 中间态 | runtime 专用（美术可改、零硬编码尺寸） | `game_data::asset_pipeline`（task-8） |
//!
//! **退役条件**：PNG+JSON 全量接管后二进制 *runtime* 路径弃用，但二进制解析器
//! **永久保留**为 oracle/工具链面——本对照测试永久常驻（不过"迁移完就删"）。
//!
//! # 判据（task-8 落地后补全）
//!
//! 逐资产 `decode(json+png) == parse(binary)`：
//! - **像素逐位**（ARGB 数组逐 u32 相等）；
//! - **元数据逐字段**（tileset 切片坐标 / 帧宽表 / glyph 度量 / JSON count）；
//! - 计数断言族（ui=25/menu=2/sflogo=8 …）升格为 **JSON count == 二进制解析计数**
//!   对照——旧锚**不许消失**（本文件骨架期先锁二进制半侧）。
//!
//! # 骨架期（task-8 前）已绿部分（2026-10-18，task-10 预备）
//!
//! 1. `count_assertion_family_binary_side_must_not_vanish`：16 容器全量张数锚
//!    （`RESOURCE_IMAGE_COUNTS`，源 `reference/seed/a.java:465-466`）+ 零余字节
//!    + 子图 PNG 魔数——JSON 半侧落地后在此逐项对照，锚永不消失；
//! 2. `binary_side_container_pixel_inventory`：容器逐图 (w,h,像素 fnv) 清单 =
//!    等价对照的**被对侧基线**（回归棘轮；等价判定本身用两面 live 对照，不靠常量）。
//!
//! 【task-8 落地后补】（红→绿，A 协议）：`decode(json+png) == parse(binary)`
//! 逐资产对照 + 元数据逐字段对照 + JSON count 对照。

use std::path::PathBuf;

use game_data::{decode_png, image_count, PackedPng, RESOURCE_IMAGE_COUNTS};

fn repo() -> PathBuf {
    // CARGO_MANIFEST_DIR = <root>/crates/game-data ⇒ nth(2) = 仓库根
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

/// 夹具目录：`XIYOU_ASSETS_RAW` 可配置（与 tests/golden.rs 同口径）。
fn raw_dir() -> PathBuf {
    let dir = std::env::var("XIYOU_ASSETS_RAW")
        .map(PathBuf::from)
        .unwrap_or_else(|_| repo().join("assets/raw"));
    assert!(
        dir.join("maplv0").exists(),
        "资产面缺失 {}（先跑 tools/extract-assets/extract.py）",
        dir.display()
    );
    dir
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// 子图像素签名：(宽, 高, ARGB 逐像素 fnv)——"像素逐位"对照的最小签名面。
fn img_sig(w: i32, h: i32, argb: &[u32]) -> (i32, i32, u64) {
    let bytes: Vec<u8> = argb.iter().flat_map(|p| p.to_le_bytes()).collect();
    (w, h, fnv1a(&bytes))
}

/// 计数断言族并入本门（硬 2：JSON count == 二进制解析计数的**二进制半侧**）。
/// 16 容器全量：`RESOURCE_IMAGE_COUNTS`（ui=25/menu=2/sflogo=8/…）逐项：
/// 解析张数 == 常量表、零余字节、每子图 PNG 魔数。锚不许消失。
#[test]
fn count_assertion_family_binary_side_must_not_vanish() {
    let dir = raw_dir();
    assert_eq!(RESOURCE_IMAGE_COUNTS.len(), 16, "计数断言族 = 16 容器全量");
    for (name, count) in RESOURCE_IMAGE_COUNTS {
        assert_eq!(image_count(name), Some(count), "{name} 常量表自洽");
        let data = std::fs::read(dir.join(name)).unwrap_or_else(|e| panic!("缺资产 {name}：{e}"));
        let (packed, consumed) = PackedPng::parse_partial(&data, count as usize)
            .unwrap_or_else(|e| panic!("{name} 解析失败：{e}"));
        assert_eq!(packed.images.len(), count as usize, "{name} 子图张数（JSON count 对照的被对侧）");
        assert_eq!(consumed, data.len(), "{name} 有余字节");
        for (i, img) in packed.images.iter().enumerate() {
            assert_eq!(img.bytes[..8], game_data::PNG_MAGIC[..], "{name}[{i}] 子图魔数");
        }
    }
}

/// 二进制面容器像素清单（等价对照的被对侧基线）：逐容器逐子图
/// (宽, 高, 像素 fnv) 滚动签名 + 容器总图数。
/// 采集：2026-10-18 task-10 预备（回归棘轮；等价判定用两面 live 对照，不靠本常量）。
#[test]
fn binary_side_container_pixel_inventory() {
    let dir = raw_dir();
    let mut inventory: Vec<(String, usize, u64)> = Vec::new();
    for (name, count) in RESOURCE_IMAGE_COUNTS {
        let data = std::fs::read(dir.join(name)).unwrap();
        let packed = PackedPng::parse(&data, count as usize).unwrap();
        let mut roll = 0xcbf2_9ce4_8422_2325u64;
        for img in &packed.images {
            let dec = decode_png(img.bytes).unwrap_or_else(|e| panic!("{name} 子图 PNG 解码失败：{e}"));
            let (w, h, sig) = img_sig(dec.width, dec.height, &dec.argb);
            assert!(w > 0 && h > 0, "{name} 子图尺寸为正");
            for part in [sig, w as u64, h as u64] {
                roll ^= part;
                roll = roll.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        inventory.push((name.to_string(), packed.images.len(), roll));
    }
    // 基线常量（2026-10-18 采集冻结；等价半侧落地后本表保持不变 = 二进制面零漂移）
    let expect: Vec<(&str, usize, u64)> = vec![
        ("sflogo", 8, 11089179762632221827),
        ("mapbg", 1, 9058403433171722916),
        ("map", 12, 13544595845242340935),
        ("actor", 4, 11446024832752310510),
        ("sptmap", 13, 149433402259614508),
        ("sptprop", 23, 2862608535259371581),
        ("sptarm", 10, 12188481851838406633),
        ("sptenemy1", 20, 3503349959782742919),
        ("ui", 25, 100831171128411964),
        ("xtq", 6, 9978050737082674684),
        ("menu", 2, 16054828451292776814),
        ("intro", 2, 15010574701990624177),
        ("face", 12, 442084169469772576),
        ("sptenemy2", 20, 8658669591137320228),
        ("end", 1, 1871265099071333537),
        ("load", 2, 9458531815363033743),
    ];
    for ((name, n, roll), (ename, en, eroll)) in inventory.iter().zip(expect.iter()) {
        assert_eq!(name, ename, "清单序（=RESOURCE_IMAGE_COUNTS 序）");
        assert_eq!(n, en, "{name} 图数");
        assert_eq!(roll, eroll, "{name} 像素清单滚动签名（二进制面漂移）");
    }
}

// TODO(task-8 红→绿)：decode(json+png) == parse(binary) 逐资产对照——
// 像素逐位（img_sig 全量相等）+ 元数据逐字段（tileset 切片坐标 / 帧宽表 /
// glyph 度量 / JSON count == RESOURCE_IMAGE_COUNTS）。
