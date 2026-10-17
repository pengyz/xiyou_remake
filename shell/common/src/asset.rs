//! AssetSource 目录型实现（共享）：注入根目录 + `std::fs`。
//!
//! 自 shell/android `asset.rs` 的 `DirAssetSource` 与 shell/win `main.rs` 的
//! `DirAssets` 合并（task-5 双壳总审裁定 2026-10-18：生产面小重复 → 并入
//! shell/common 单源；两侧消费方以别名接线，测试随迁断言零改动）。
//!
//! # 契约（docs/spec/platform-contract.md §4）
//!
//! `read(rel) -> Result<Vec<u8>, String>`：rel = `/` 分隔、无前导 `/`、无 `..`；
//! 失败 ⇒ Err(人话诊断)、不 panic。根 = 平台注入的资产根（Windows `--assets`；
//! Android 宿主测试同此桩；APK 真实现 = `AAssetManager`，在 shell/android）。
//!
//! # 合并语义声明（唯一语义统一面，2026-10-18）
//!
//! 错误诊断 = `缺资产 {rel}（根 …）：{io}`（原 android 侧格式，含 rel 名与根路径，
//! 尾段保留原始 io 文本）。win 侧原裸 io 文本被此结构化诊断**包络**——win 的 panic
//! 模板（`缺资产 {name}（先跑 tools/extract-assets/extract.py）：{e}`）为前缀断言，
//! 双侧测试断言零改动全绿；游戏可达行为不变（缺资产路径恒 panic 终止）。

use game_platform::AssetSource;
use std::path::PathBuf;

/// rel 契约校验（`/` 分隔、无前导 `/`、无 `..`、无空段、无 `\`——Windows 逃逸面）。
pub fn validate_rel(rel: &str) -> Result<(), String> {
    if rel.is_empty() {
        return Err("资产 rel 为空（须为 `/` 分隔的相对路径）".to_string());
    }
    if rel.starts_with('/') {
        return Err(format!("资产 rel 不得带前导 `/`：{rel:?}"));
    }
    if rel.contains('\\') {
        return Err(format!("资产 rel 须用 `/` 分隔（不含 `\\`）：{rel:?}"));
    }
    if rel.split('/').any(|seg| seg.is_empty() || seg == ".." || seg == ".") {
        return Err(format!("资产 rel 含非法段（空段/`.`/`..`）：{rel:?}"));
    }
    Ok(())
}

/// 目录注入资产源：`std::fs` 读根下 rel 路径（Windows `--assets`；Android 宿主测试桩）。
#[derive(Debug, Clone)]
pub struct DirAssetSource {
    root: PathBuf,
}

impl DirAssetSource {
    pub fn new(root: impl Into<PathBuf>) -> DirAssetSource {
        DirAssetSource { root: root.into() }
    }
}

impl AssetSource for DirAssetSource {
    fn read(&self, rel: &str) -> Result<Vec<u8>, String> {
        validate_rel(rel)?;
        let path = self.root.join(rel.split('/').collect::<std::path::PathBuf>());
        std::fs::read(&path).map_err(|e| format!("缺资产 {rel}（根 {}）：{e}", self.root.display()))
    }
}
