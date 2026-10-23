//! T5 原生入口构建（task-14）：编译 NDK `native_app_glue.c` 进 .so——
//! `ANativeActivity_onCreate`（glue 提供）+ `android_main`（Rust 提供）构成
//! NativeActivity 标准入口对。仅 android 目标生效（宿主构建零副作用）。
//!
//! NDK 路径取序：ANDROID_NDK_HOME / ANDROID_NDK_ROOT / ANDROID_NDK /
//! 默认 ~/Android/Sdk/ndk/27.1.12297006（本仓实证路径）。

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "android" {
        return;
    }
    let ndk = std::env::var("ANDROID_NDK_HOME")
        .or_else(|_| std::env::var("ANDROID_NDK_ROOT"))
        .or_else(|_| std::env::var("ANDROID_NDK"))
        .unwrap_or_else(|_| {
            format!(
                "{}/Android/Sdk/ndk/27.1.12297006",
                std::env::var("HOME").unwrap_or_default()
            )
        });
    let glue = std::path::Path::new(&ndk).join("sources/android/native_app_glue");
    cc::Build::new()
        .file(glue.join("android_native_app_glue.c"))
        .include(&glue)
        .flag("-Wno-unused-parameter")
        // 【符号局部化规避】静态库符号默认进不了 dynsym（nm 小写 t ⇒ 系统
        // dlsym(ANativeActivity_onCreate) 仍失败）——编译期把胶水入口重命名为
        // glue_ANativeActivity_onCreate，由 Rust `#[no_mangle]` trampoline 提供
        // 真名导出（与 android_main 同导出机制，必进 dynsym）。
        .define("ANativeActivity_onCreate", "glue_ANativeActivity_onCreate")
        .compile("android_native_app_glue");
    // glue 依赖：libandroid（ALooper/AInput/ANativeWindow）+ liblog
    println!("cargo:rustc-link-lib=android");
    println!("cargo:rustc-link-lib=log");
    // 【关键】静态胶水库成员无 Rust 侧引用时不被拉进 .so（= 原崩因同类：
    // undefined symbol: ANativeActivity_onCreate）——`-u` 强制解析该符号，
    // 连带拉入定义它的 glue 目标文件并导出（系统 dlsym 入口）。
    println!("cargo:rustc-link-arg=-Wl,-u,glue_ANativeActivity_onCreate");
    println!("cargo:rustc-link-arg-cdylib=-Wl,-u,glue_ANativeActivity_onCreate");
    // 静态库符号默认被局部化（nm 小写 t，不进 dynsym ⇒ 系统 dlsym 仍失败）——
    // 显式提为动态导出（崩溃根治面第二半）。
    println!(
        "cargo:rustc-link-arg-cdylib=-Wl,--export-dynamic-symbol=glue_ANativeActivity_onCreate"
    );
}
