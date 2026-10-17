//! AssetSource 契约测试（docs/spec/platform-contract.md §4）。
//!
//! 契约面 = `game_platform::AssetSource`（rel 路径读字节）。真实实现在壳层
//! （Windows `std::fs::read` 目录注入 / Android `AAssetManager` APK assets），
//! 本 crate 只放 trait 与类型（AGENTS §2 红线）⇒ 此处以测试替身承载契约语义的
//! 可执行文档 + API 形状锁。
//!
//! TDD RED 复现（A 强化协议 §7）：本文件单独应用到父树（`game_platform` 尚无
//! AssetSource）⇒ `cargo test -p game-platform --test asset_source_contract`
//! 编译失败（E0432）即红。

use game_platform::AssetSource;

/// 测试替身：内存资产表（键 = rel 路径）。
#[derive(Default)]
struct MemAssets(std::collections::HashMap<String, Vec<u8>>);

impl AssetSource for MemAssets {
    fn read(&self, rel: &str) -> Result<Vec<u8>, String> {
        self.0.get(rel).cloned().ok_or_else(|| format!("缺资产 {rel}"))
    }
}

/// 命中 ⇒ 逐字节返回（load_container/main.rs:61-70 的消费语义）。
#[test]
fn read_returns_exact_bytes() {
    let mut a = MemAssets::default();
    a.0.insert("mapbg".to_string(), vec![0x89, 0x50, 0x4E, 0x47]);
    assert_eq!(a.read("mapbg").unwrap(), vec![0x89, 0x50, 0x4E, 0x47]);
}

/// 缺失/失败 ⇒ Err 携带人话诊断（panic 归壳侧逐字保留，spec §4.2）。
#[test]
fn read_missing_is_err_with_diagnostic() {
    let a = MemAssets::default();
    let err = a.read("maplv99").unwrap_err();
    assert!(!err.is_empty(), "Err 携带诊断文本（壳侧拼进 panic 消息）");
}

/// rel 约定：`/` 分隔、无前导 `/`、无 `..`；纯文件名是现状调用形态
/// （"mapbg"/"maplv{n}"/"sprite{n}"），子路径形如 "sub/x.bin"。
#[test]
fn rel_keys_are_plain_relative_paths() {
    let mut a = MemAssets::default();
    a.0.insert("maplv1".to_string(), vec![1]);
    a.0.insert("sub/x.bin".to_string(), vec![2]);
    assert_eq!(a.read("maplv1").unwrap(), vec![1], "现状形态：纯文件名");
    assert_eq!(a.read("sub/x.bin").unwrap(), vec![2], "契约放宽：子路径");
    assert!(a.read("/maplv1").is_err(), "前导 / 不属契约形态");
    assert!(a.read("../save/MOT_L0.bin").is_err(), ".. 不属契约形态（禁越根）");
}

/// 形状锁①：对象可用（壳层以 `&dyn AssetSource` 接线，多消费点只读共享）。
#[test]
fn asset_source_is_object_safe_with_shared_ref() {
    fn twice(a: &dyn AssetSource, rel: &str) -> usize {
        a.read(rel).map(|b| b.len()).unwrap_or(0) + a.read(rel).map(|b| b.len()).unwrap_or(0)
    }
    let mut a = MemAssets::default();
    a.0.insert("ui".to_string(), vec![0; 5]);
    assert_eq!(twice(&a, "ui"), 10, "read 取 &self：共享引用可重复读");
}
