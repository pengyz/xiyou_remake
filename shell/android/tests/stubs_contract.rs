//! 单元 3 契约测试：宿主桩三件（ProcessControl/AudioOut/AssetSource）契约语义
//! （TDD RED 先行）。真值锚：docs/spec/platform-contract.md §1-§4 契约面
//! （幂等不 panic/全量写阻塞/Err 不可恢复/idle 丢弃/rel 契约/Err=人话诊断）。
//!
//! 诚实边界：桩只锁**契约形状**——AAudio 真阻塞、AAssetManager 全量读、
//! ANativeActivity_finish 幂等的原生语义在 #[cfg(target_os="android")] 骨架内，
//! 未经验证（README 验证状态矩阵）。

use game_android::asset::{validate_rel, DirAssetSource};
use game_android::audio::CountingPcm;
use game_android::process::HostProcessControl;
use game_platform::{AssetSource, AudioOut, ProcessControl, CHANNELS, SAMPLE_RATE};

// ---------------------------------------------------------------- ProcessControl

/// 退出请求幂等：重复 request_exit 不 panic、终态一致（exit_requested）。
#[test]
fn exit_request_is_recorded_and_idempotent() {
    let mut p = HostProcessControl::new();
    assert!(!p.exit_requested());
    p.request_exit();
    p.request_exit();
    p.request_exit();
    assert!(p.exit_requested());
    assert_eq!(p.calls(), 3, "调用可观测；副作用终态幂等");
}

/// 对象安全：`Box<dyn ProcessControl>`（audio_loop/壳循环持 trait 对象同式）。
#[test]
fn process_control_is_object_safe() {
    let mut p: Box<dyn ProcessControl> = Box::new(HostProcessControl::new());
    p.request_exit();
}

// ---------------------------------------------------------------- AudioOut

/// 全量写 = 帧计数（CHANNELS=2 交错 i16：2048 样本 = 1024 帧）。
#[test]
fn counting_pcm_write_counts_frames() {
    let mut d = CountingPcm::new();
    assert_eq!(d.write(&vec![0i16; 2048]), Ok(()));
    assert_eq!(d.frames_written, 1024);
    assert_eq!(d.write(&[]), Ok(()), "空写无害");
    assert_eq!(d.frames_written, 1024);
}

/// idle = 停止并丢弃缓冲（桩计数语义）。
#[test]
fn counting_pcm_idle_counts() {
    let mut d = CountingPcm::new();
    d.idle();
    d.idle();
    assert_eq!(d.idle_calls, 2);
}

/// Err = 不可恢复：故障后持续 Err（上层转无声），不 panic。
#[test]
fn counting_pcm_failure_is_unrecoverable() {
    let mut d = CountingPcm::failing_at(0);
    assert!(d.write(&vec![0i16; 2048]).is_err(), "首写即超限 ⇒ Err");
    assert!(d.write(&vec![0i16; 2]).is_err(), "不可恢复");
    assert_eq!(d.frames_written, 0, "失败写不计入");
}

/// Send：设备整体 move 进音频线程（audio_loop 同式，契约 `AudioOut: Send`）。
#[test]
fn counting_pcm_moves_into_audio_thread() {
    let mut d = CountingPcm::new();
    d.write(&vec![0i16; 4]).unwrap();
    let h = std::thread::spawn(move || {
        let mut d = d;
        d.write(&vec![0i16; 8]).unwrap();
        d
    });
    let d = h.join().unwrap();
    assert_eq!(d.frames_written, 2 + 4);
}

// ---------------------------------------------------------------- AssetSource

fn tmp_root() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ga-asset-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::write(dir.join("mapbg"), b"BG").unwrap();
    std::fs::write(dir.join("sub").join("sprite1"), b"SP").unwrap();
    dir
}

/// 目录注入 + rel 读字节（宿主桩；Android 对偶 = APK assets）。
#[test]
fn dir_asset_reads_root_relative() {
    let root = tmp_root();
    let s = DirAssetSource::new(&root);
    assert_eq!(s.read("mapbg").unwrap(), b"BG");
    assert_eq!(s.read("sub/sprite1").unwrap(), b"SP", "rel 子路径（`/` 分隔）");
    let _ = std::fs::remove_dir_all(&root);
}

/// 缺资产 ⇒ Err 含人话诊断（rel 名 + 原因），不 panic（契约 §4：panic 归壳侧）。
#[test]
fn dir_asset_missing_gives_diagnostic() {
    let root = tmp_root();
    let s = DirAssetSource::new(&root);
    let e = s.read("maplv99").unwrap_err();
    assert!(e.contains("maplv99"), "诊断含 rel 名：{e}");
    assert!(e.contains("缺资产"), "诊断含人话前缀：{e}");
    let _ = std::fs::remove_dir_all(&root);
}

/// rel 契约：`/` 分隔、无前导 `/`、无 `..`/`.`/空段/`\`——非法 rel ⇒ Err 不 panic。
#[test]
fn dir_asset_rejects_illegal_rel() {
    let root = tmp_root();
    let s = DirAssetSource::new(&root);
    for bad in ["", "/etc/passwd", "a/../b", "../x", "a\\b", "a//b", "./x"] {
        assert!(s.read(bad).is_err(), "{bad:?} 应拒绝");
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// validate_rel 独立契约面（两实现共用的校验）。
#[test]
fn validate_rel_contract() {
    assert!(validate_rel("mapbg").is_ok());
    assert!(validate_rel("audio/track0.ogg").is_ok());
    assert!(validate_rel("").is_err());
    assert!(validate_rel("/a").is_err());
    assert!(validate_rel("a/../b").is_err());
}

/// 线上格式常量与契约一致（44.1 kHz 交错立体声）。
#[test]
fn pcm_format_matches_contract() {
    assert_eq!(SAMPLE_RATE, 44_100);
    assert_eq!(CHANNELS, 2);
}
