//! STAGE-B-003 复核硬 1：unpack∘repack = 全资产**逐字节恒等**（迁移可信的唯一判据）。
//! 恒等在 repack 端成立（PNG 重编码字节必变，故比对 repack(binary) == 原 binary）；
//! 任一资产不恒等 ⇒ 该资产挂起（禁部分通过：逐资产断言，失败清单=资产名+两侧字节数）。
//! 常驻 gates（复核硬 1："入 gates 常驻非一次性"）。
//!
//! 资产面全集 = assets/raw 里三格式族条目：maplv0-54（55）+ sprite0-54（55）+
//! packed_png 容器 18 名（RESOURCE_IMAGE_COUNTS 全域）。非三族条目（*.class/*.mid/
//! 散图 png/script 等）不属本管线。

use std::path::PathBuf;

fn repo() -> PathBuf {
    // CARGO_MANIFEST_DIR = <repo>/crates/game-data ⇒ 祖先第 2 级才是仓库根
    //（nth(1) 只到 crates/，读不到 assets/raw——shell-port 2026-10-18 勘误）。
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

/// demo 弃件前缀（R-7 弃件隔离）：`hd-demo*` = 高清素材探针 demo 产物名
/// （tools/hd-demo 生成、落 assets/hd-demo/，不入资产面）。
const DEMO_PREFIX: &str = "hd-demo";

/// 资产面全集（三格式族）：maplv*/sprite*/packed_png 容器 18 名。
/// demo 前缀显式排除（R-7）：demo 图集不入恒等门禁。
fn in_scope(name: &str) -> bool {
    if name.starts_with(DEMO_PREFIX) {
        return false;
    }
    name.starts_with("maplv")
        || name.starts_with("sprite")
        || game_data::image_count(name).is_some()
}

/// 全资产往返恒等：对 assets/raw 每个容器/表逐个 unpack→repack→逐字节比对。
#[test]
fn unpack_repack_byte_identity_all_assets() {
    let raw = repo().join("assets/raw");
    let mut checked = 0usize;
    let mut failed: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&raw).unwrap() {
        let e = entry.unwrap();
        let name = e.file_name().to_string_lossy().to_string();
        let path = e.path();
        if !path.is_file() || !in_scope(&name) {
            continue;
        }
        let orig = std::fs::read(&path).unwrap();
        // unpack → 中间态（图片+JSON）→ repack
        let mid = match game_data::asset_pipeline::unpack(&name, &orig) {
            Ok(m) => m,
            Err(e) => {
                failed.push(format!("{name}: unpack 失败 {e}"));
                continue;
            }
        };
        let repacked = match game_data::asset_pipeline::repack(&mid) {
            Ok(b) => b,
            Err(e) => {
                failed.push(format!("{name}: repack 失败 {e}"));
                continue;
            }
        };
        if repacked != orig {
            failed.push(format!(
                "{name}: 恒等破坏（原 {}B vs 回 {}B）",
                orig.len(),
                repacked.len()
            ));
        }
        checked += 1;
    }
    assert!(checked > 0, "未检出任何资产（资产面缺失）");
    assert!(
        failed.is_empty(),
        "STAGE-B-003 硬1 恒等破坏 {} 项：\n{}",
        failed.len(),
        failed.join("\n")
    );
}

/// R-7 弃件隔离护栏（demo 图集误入即红）：
/// ① demo 前缀名恒不入 in_scope（排除断言）；
/// ② legacy 三族仍须在范围（排除不误伤恒等面）；
/// ③ assets/raw 资产面不得出现 demo 前缀文件（demo 产物只能落 assets/hd-demo/）。
#[test]
fn demo_prefix_excluded_from_roundtrip_scope() {
    assert!(!in_scope(DEMO_PREFIX), "demo 前缀必须排除出恒等门禁");
    assert!(!in_scope("hd-demo-map_00_3x"), "demo 产物名必须排除");
    assert!(!in_scope("hd-demo"), "demo 前缀必须排除");
    // legacy 三族不许被排除误伤（恒等面锚）
    assert!(in_scope("maplv0"));
    assert!(in_scope("sprite0"));
    assert!(in_scope("map"));
    // 误入检测：assets/raw 是恒等门禁资产面，demo 前缀文件出现在这里 = 越界，即红
    let raw = repo().join("assets/raw");
    for entry in std::fs::read_dir(&raw).unwrap() {
        let e = entry.unwrap();
        let name = e.file_name().to_string_lossy().to_string();
        assert!(
            !name.starts_with(DEMO_PREFIX),
            "demo 产物误入资产面 assets/raw: {name}（应只落 assets/hd-demo/，R-7）"
        );
    }
}
