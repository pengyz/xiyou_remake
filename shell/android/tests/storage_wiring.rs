//! 单元 4 契约测试：Storage 的 Android 接线（消费 game-shell-common::FileRms，不落拷贝）。
//!
//! 真值锚：docs/spec/platform-contract.md §1（Storage 契约）+ §1.3（Java RMS 等价性）
//! + decision_save-path-shell-injection（根路径宿主注入：Android = 应用私有目录
//! internalDataPath）。文件语义主测试单源在 game-shell-common（对抗审查 F-4/抽取面
//! 审查判据）——本文件只锁**包装接线面**：根路径注入、域过滤委托、对象安全、六槽+全 8 键域。

use game_android::storage::AndroidStorage;
use game_platform::{rms_key, Storage, RMS_STORES};
use std::path::PathBuf;

fn tmp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ga-storage-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// 根路径注入：persist 落在注入的根；换根（另一私有目录）读不到——路径注入精确生效
/// （Android 的根 = activity->internalDataPath，宿主注入，游戏层零路径）。
#[test]
fn storage_roundtrips_via_injected_root() {
    let root_a = tmp_root("a");
    let root_b = tmp_root("b");
    let (s, loaded) = AndroidStorage::open(&root_a);
    assert!(loaded.is_empty(), "空根无 store");
    s.persist(&[(rms_key("MOT_L0"), vec![1, 2, 3]), (rms_key("SKY_WAR"), vec![9])]);
    let (s2, stores) = AndroidStorage::open(&root_a);
    assert_eq!(stores.len(), 2, "落盘回读：{stores:?}");
    assert_eq!(s2.load_all(), stores, "load_all 与构造载入一致");
    let (_s3, other) = AndroidStorage::open(&root_b);
    assert!(other.is_empty(), "换根不串档");
    let _ = std::fs::remove_dir_all(&root_a);
    let _ = std::fs::remove_dir_all(&root_b);
}

/// 域过滤委托（契约 §1.1）：域外键丢弃不 panic，load_all 只报域内记录。
#[test]
fn storage_drops_out_of_domain_keys() {
    let root = tmp_root("domain");
    let (s, _) = AndroidStorage::open(&root);
    s.persist(&[("STRAY@1".to_string(), vec![7]), (rms_key("MOT_IF"), vec![1])]);
    let all = s.load_all();
    assert_eq!(all, vec![(rms_key("MOT_IF"), vec![1])], "杂散键不得进记录域");
    let _ = std::fs::remove_dir_all(&root);
}

/// 全 8 键域（BUG-018 教训：六槽 MOT_L0-5 + MOT_IF + SKY_WAR 全载）——包装层回归。
#[test]
fn storage_keeps_full_eight_store_domain() {
    let root = tmp_root("domain8");
    let (s, _) = AndroidStorage::open(&root);
    let recs: Vec<(String, Vec<u8>)> = RMS_STORES
        .iter()
        .enumerate()
        .map(|(i, name)| (rms_key(name), vec![i as u8; 4]))
        .collect();
    s.persist(&recs);
    let (s2, stores) = AndroidStorage::open(&root);
    assert_eq!(stores.len(), 8, "八店全落盘：{stores:?}");
    assert_eq!(s2.load_all(), recs, "键序 = RMS_STORES 序、值逐字节");
    let _ = std::fs::remove_dir_all(&root);
}

/// 对象安全：`Box<dyn Storage>`（壳循环持 trait 对象同式）。
#[test]
fn storage_is_object_safe() {
    let root = tmp_root("dyn");
    let (s, _) = AndroidStorage::open(&root);
    let d: &dyn Storage = &s;
    d.persist(&[(rms_key("MOT_L5"), vec![5])]);
    assert_eq!(d.load_all(), vec![(rms_key("MOT_L5"), vec![5])]);
    let _ = std::fs::remove_dir_all(&root);
}
