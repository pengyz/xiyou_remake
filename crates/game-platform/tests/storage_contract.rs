//! Storage 契约测试（docs/spec/platform-contract.md §1）。
//!
//! 契约面 = `game_platform` 的 RMS 记录模型（`RMS_STORES`/`RMS_RECORD_ID`/
//! `rms_key`/`rms_store_name`）+ [`Storage`] trait。真实实现在壳层
//! （shell/win 文件版 RMS、shell/android 私有目录版），本 crate 只放 trait 与
//! 类型（AGENTS §2 红线）⇒ 此处以测试替身承载**契约语义的可执行文档** + API 形状锁。
//!
//! TDD RED 复现（A 强化协议 §7）：本文件单独应用到父树（`game_platform` 尚无
//! RMS 记录模型与 Storage）⇒ `cargo test -p game-platform --test storage_contract`
//! 编译失败（E0432/E0433）即红。

use game_platform::{rms_key, rms_store_name, Storage, RMS_RECORD_ID, RMS_STORES};

/// 测试替身：内存记录库（无平台 I/O）。persist 按契约丢弃域外键。
#[derive(Default)]
struct MemStore(std::cell::RefCell<std::collections::BTreeMap<String, Vec<u8>>>);

impl Storage for MemStore {
    fn load_all(&self) -> Vec<(String, Vec<u8>)> {
        self.0.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
    fn persist(&self, records: &[(String, Vec<u8>)]) {
        let mut map = self.0.borrow_mut();
        for (k, v) in records {
            if RMS_STORES.iter().any(|s| rms_key(s) == *k) {
                map.insert(k.clone(), v.clone());
            }
        }
    }
}

/// 记录键格式 `<store>@<record_id>`，记录号恒 1（oracle preset `KEY@1=hex` 同构）。
#[test]
fn rms_key_is_store_at_one() {
    assert_eq!(RMS_RECORD_ID, 1);
    assert_eq!(rms_key("MOT_L0"), "MOT_L0@1");
    assert_eq!(rms_key("SKY_WAR"), "SKY_WAR@1");
}

/// store 名回取：首个 `@` 之前；无 `@` 原样；空串空串；全输入域不 panic。
#[test]
fn rms_store_name_strips_record_id_safely() {
    assert_eq!(rms_store_name("MOT_L0@1"), "MOT_L0");
    assert_eq!(rms_store_name("SKY_WAR@1"), "SKY_WAR");
    assert_eq!(rms_store_name("MOT_L0"), "MOT_L0", "无 @ 的键原样返回");
    assert_eq!(rms_store_name(""), "", "空串入空串出");
    assert_eq!(rms_store_name("A@1@2"), "A", "取首个 @ 之前");
}

/// 键域 = Java 原版硬编码八店（a.java:4452 "SKY_WAR"、8960 "MOT_IF"、9190 "MOT_L"+0..=5）；
/// 且 rms_key 一一映射（防双壳白名单漂移）。
#[test]
fn rms_store_domain_is_the_java_eight() {
    assert_eq!(RMS_STORES, ["MOT_L0", "MOT_L1", "MOT_L2", "MOT_L3", "MOT_L4", "MOT_L5", "MOT_IF", "SKY_WAR"]);
    let keys: std::collections::BTreeSet<String> = RMS_STORES.iter().map(|s| rms_key(s)).collect();
    assert_eq!(keys.len(), RMS_STORES.len(), "rms_key 在键域上单射");
}

/// load_all：只报已持久记录，缺失 store 静默缺席（Rms::load 同语义）。
#[test]
fn storage_load_all_lists_persisted_records_only() {
    let s = MemStore::default();
    assert!(s.load_all().is_empty(), "空存储无记录");
    s.persist(&[(rms_key("MOT_L0"), vec![1, 2, 3])]);
    assert_eq!(s.load_all(), vec![(rms_key("MOT_L0"), vec![1, 2, 3])]);
}

/// persist：同键覆盖、新键创建（upsert）、未提及键不动。
#[test]
fn storage_persist_upserts_and_keeps_unmentioned() {
    let s = MemStore::default();
    s.persist(&[(rms_key("MOT_L0"), vec![1]), (rms_key("MOT_IF"), vec![9])]);
    s.persist(&[(rms_key("MOT_L0"), vec![2]), (rms_key("MOT_L5"), vec![5])]);
    let all = s.load_all();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0], (rms_key("MOT_IF"), vec![9]), "未提及键不动");
    assert_eq!(all[1], (rms_key("MOT_L0"), vec![2]), "同键覆盖");
    assert_eq!(all[2], (rms_key("MOT_L5"), vec![5]), "新键创建");
}

/// 域外键 = 契约违约 ⇒ 丢弃，不 panic（§1.1）。
#[test]
fn storage_persist_drops_foreign_keys_without_panic() {
    let s = MemStore::default();
    s.persist(&[(rms_key("STRAY"), vec![7]), (rms_key("MOT_IF"), vec![1])]);
    assert_eq!(s.load_all(), vec![(rms_key("MOT_IF"), vec![1])], "杂散键不得进记录域");
}

/// API 形状锁：trait 对象可用（壳层以 `Box<dyn Storage>`/`&dyn Storage` 接线）。
#[test]
fn storage_is_object_safe() {
    fn via_dyn(s: &dyn Storage, key: &str) -> Option<Vec<u8>> {
        s.load_all().into_iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
    let s = MemStore::default();
    s.persist(&[(rms_key("SKY_WAR"), vec![42])]);
    assert_eq!(via_dyn(&s, &rms_key("SKY_WAR")), Some(vec![42]));
}
