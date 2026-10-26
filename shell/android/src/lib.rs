//! game-android —— Android 壳（平台壳族的第二个实现，shell/ 结构决策 2026-10-17）。
//!
//! 分层（AGENTS §2 / docs/knowledge/architecture_shell-contract.md）：本 crate 只做
//! 平台实现；游戏逻辑全在 game-core，平台 trait 与类型在 game-platform
//! （docs/spec/platform-contract.md v1.0），共享壳实现（FileRms/音频混音）在
//! game-shell-common（消费不拷贝，对抗审查 F-4 裁决）。
//!
//! 四 trait 的 Android 实现（契约 §5 映射表）：
//! ① Storage = 应用私有目录文件版 RMS（宿主注入根路径，FileRms 薄接线）；
//! ② ProcessControl = Activity.finish()（ANativeActivity_finish/JNI，不杀进程）；
//! ③ AudioOut = AAudio 阻塞 write（节拍语义同构；API<26 ⇒ NullSink 无声降级）；
//! ④ AssetSource = APK assets（AAssetManager，**不是 std::fs**）。
//!
//! # 诚实边界（本环境无 Android SDK/NDK）
//!
//! - ✅ 宿主已验证：触控→键码映射（含多点触控**折叠单键状态机**）、生命周期状态机、
//!   宿主桩契约语义（`cargo test -p game-android` 全绿）。
//! - ⚠️ 骨架未编译：一切 `#[cfg(target_os = "android")]` 原生调用（AAssetManager/
//!   AAudio/ANativeActivity_finish/NativeActivity 胶水）——代码骨架 + README 构建配方
//!   （cargo-ndk/gradle），**未经任何真机/交叉编译验证**，不虚报可运行。
//! - 完整验证状态矩阵见 README.md。
//!
//! 模块（各单元 TDD 红→绿逐一落地）：`input`（触控/硬件键→键码映射）、
//! `lifecycle`（Activity 生命周期→hideNotify/showNotify）、`process`/`audio`/`asset`
//! （契约四 trait 的 Android 实现与宿主桩）。

pub mod asset;
pub mod audio;
pub mod input;
pub mod lifecycle;
pub mod process;
pub mod pump;
pub mod storage;

/// NativeActivity 胶水骨架（宿主可编译=纯装配态+不透明类型；真正 NDK 调用在
/// `process::android`/`audio::aaudio`/`asset::apk` 的 cfg(android) 子模块）。
pub mod native;
