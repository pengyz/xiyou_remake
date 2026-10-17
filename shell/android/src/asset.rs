//! AssetSource 实现：资产读取（APK assets = `AAssetManager`，**不是 std::fs**）。
//!
//! # 契约（docs/spec/platform-contract.md §4）
//!
//! `read(rel) -> Result<Vec<u8>, String>`：rel = `/` 分隔、无前导 `/`、无 `..`；
//! 失败 ⇒ Err(人话诊断)、不 panic。根 = 平台注入的资产根（Windows `--assets`；
//! Android APK assets 根）。音频产物 `track{i}.ogg` 走本 trait 的 Android 落地
//! （B2 第五缝：Android 不得沿用路径注入）——Windows 仍走 `--audio` 目录（win 专用）。
//!
//! # 平台映射
//!
//! - 宿主桩 [`DirAssetSource`]：目录注入 + std::fs（宿主测试/开发跑）。
//! - Android [`apk::ApkAssetSource`]：`AAssetManager_open` + 全量读（NDK，免 JNI——
//!   NativeActivity 的 `activity->assetManager` 直接可得）。

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

/// 宿主桩资产源：注入根目录 + `std::fs`（Android 真实现见 [`apk`]——APK assets 无路径）。
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

/// Android APK assets 源（`#[cfg(target_os = "android")]` 门控）。
#[cfg(target_os = "android")]
pub mod apk {
    //! TODO(骨架·未验证)：`AAssetManager` NDK 直绑（android/asset_manager.h，
    //! libandroid.so）。本环境无 SDK/NDK——extern 签名照 NDK 头文件手写，
    //! **未经编译验证**（README 验证状态矩阵标注）。
    //!
    //! 取得 `AAssetManager*` 的两条路：①NativeActivity：`activity->assetManager`
    //! （免 JNI，首选）；②Java Activity：JNI `Context.getAssetManager()`。
    use super::validate_rel;
    use game_platform::AssetSource;
    use std::ffi::{c_char, c_void, CString};

    /// `AAssetManager`（不透明）。
    #[repr(C)]
    pub struct AAssetManager {
        _private: [u8; 0],
    }
    /// `AAsset`（不透明）。
    #[repr(C)]
    pub struct AAsset {
        _private: [u8; 0],
    }

    pub const AASSET_MODE_UNKNOWN: i32 = 0;

    #[link(name = "android")]
    extern "C" {
        fn AAssetManager_open(
            mgr: *mut AAssetManager,
            filename: *const c_char,
            mode: i32,
        ) -> *mut AAsset;
        fn AAsset_read(asset: *mut AAsset, buf: *mut c_void, count: usize) -> i32;
        fn AAsset_getLength64(asset: *mut AAsset) -> i64;
        fn AAsset_close(asset: *mut AAsset);
    }

    /// Android 资产源：APK assets（非 std::fs）。
    pub struct ApkAssetSource {
        mgr: *mut AAssetManager,
    }

    // SAFETY：AAssetManager* 由 NativeActivity 提供、生命周期覆盖应用运行期；
    // NDK 侧 AAssetManager_open 可从任意线程调用（文档口径，未验证）。
    unsafe impl Send for ApkAssetSource {}
    unsafe impl Sync for ApkAssetSource {}

    impl ApkAssetSource {
        /// `mgr` = `(*mut AAssetManager)`（native_app_glue 的 `activity->assetManager`）。
        pub fn new(mgr: *mut AAssetManager) -> ApkAssetSource {
            ApkAssetSource { mgr }
        }
    }

    impl AssetSource for ApkAssetSource {
        fn read(&self, rel: &str) -> Result<Vec<u8>, String> {
            validate_rel(rel)?;
            let name =
                CString::new(rel).map_err(|_| format!("资产 rel 含 NUL：{rel:?}"))?;
            // SAFETY：mgr 为宿主注入句柄；asset 打开后必 close。
            unsafe {
                let asset = AAssetManager_open(self.mgr, name.as_ptr(), AASSET_MODE_UNKNOWN);
                if asset.is_null() {
                    return Err(format!("缺资产 {rel}（APK assets 中不存在）"));
                }
                let len = AAsset_getLength64(asset);
                if len < 0 {
                    AAsset_close(asset);
                    return Err(format!("资产 {rel} 长度异常：{len}"));
                }
                let mut buf = vec![0u8; len as usize];
                let mut off = 0usize;
                while off < buf.len() {
                    let n = AAsset_read(
                        asset,
                        buf.as_mut_ptr().add(off) as *mut c_void,
                        buf.len() - off,
                    );
                    if n <= 0 {
                        AAsset_close(asset);
                        return Err(format!("资产 {rel} 读取中断（AAsset_read={n}）"));
                    }
                    off += n as usize;
                }
                AAsset_close(asset);
                Ok(buf)
            }
        }
    }
}
