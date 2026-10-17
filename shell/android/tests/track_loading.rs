//! 单元 5 契约测试：音频轨装载走 AssetSource（B2 第五缝口径：Android 轨道**不得**
//! 沿用路径注入——轨道字节 = `AssetSource::read("track{i}.ogg")`）。
//!
//! 主测试（解码/循环点/混音）单源在 game-shell-common 测试面；本文件只锁
//! **装载接线面**：资产源供字节→解码→TrackStore 装配、缺失/损坏静音降级不 panic。
//! 夹具跨读 shell/common/tests/fixtures/loop-fixture.ogg（单源不复制；
//! golden 值由 shell/common 主测试钉，此处仅接线交叉核对）。

use game_android::asset::DirAssetSource;
use game_android::audio::load_tracks_from_assets;
use std::path::PathBuf;

fn fixture() -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../common/tests/fixtures/loop-fixture.ogg"),
    )
    .expect("shell/common 共享夹具（单源）")
}

/// 每用例独立临时目录（tag 区分；避开并行互删竞态——2026-10-18 教训）。
fn tmp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ga-track-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 资产源供字节 → 解码 → 装配进 TrackStore（槽位对齐 wanted/slots）。
#[test]
fn loads_track_bytes_via_asset_source() {
    let root = tmp_root("load");
    std::fs::write(root.join("track2.ogg"), fixture()).unwrap();
    let store = load_tracks_from_assets(&DirAssetSource::new(&root), &[2], 4);
    let guard = store.lock().unwrap();
    let t = guard[2].as_ref().expect("track2 应装载");
    assert_eq!((t.loop_start, t.loop_end), (11_025, 33_075), "接线交叉核对（golden 在 shell/common）");
    assert!(guard[0].is_none() && guard[1].is_none() && guard[3].is_none(), "未请轨道不装");
    let _ = std::fs::remove_dir_all(&root);
}

/// 缺失轨道 ⇒ 该格 None（静音降级），不 panic（同 shell/win start() 口径）。
#[test]
fn missing_track_stays_silent() {
    let root = tmp_root("missing");
    let store = load_tracks_from_assets(&DirAssetSource::new(&root), &[3], 4);
    assert!(store.lock().unwrap().iter().all(|t| t.is_none()));
    let _ = std::fs::remove_dir_all(&root);
}

/// 损坏字节 ⇒ 解码失败转静音（None），不 panic。
#[test]
fn corrupt_track_stays_silent() {
    let root = tmp_root("corrupt");
    std::fs::write(root.join("track0.ogg"), b"not an ogg").unwrap();
    let store = load_tracks_from_assets(&DirAssetSource::new(&root), &[0], 4);
    assert!(store.lock().unwrap()[0].is_none(), "损坏转静音");
    let _ = std::fs::remove_dir_all(&root);
}

/// 越界下标（>= slots）忽略不 panic（防御面）。
#[test]
fn out_of_range_index_is_ignored() {
    let root = tmp_root("range");
    let store = load_tracks_from_assets(&DirAssetSource::new(&root), &[9], 4);
    assert_eq!(store.lock().unwrap().len(), 4);
    assert!(store.lock().unwrap().iter().all(|t| t.is_none()));
    let _ = std::fs::remove_dir_all(&root);
}
