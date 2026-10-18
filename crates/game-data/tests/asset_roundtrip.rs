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

/// 资产面全集（三格式族）：maplv*/sprite*/packed_png 容器 18 名。
fn in_scope(name: &str) -> bool {
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
