//! AudioOut 契约测试（docs/spec/platform-contract.md §3）。
//!
//! 契约面 = `game_platform::AudioOut`（自 shell/win `audio.rs:214-219` 的
//! `PcmDevice` 迁入更名）+ PCM 线上格式常量。真实实现在壳层（winmm waveOut /
//! ALSA dlopen / Android AAudio），本 crate 只放 trait 与类型（AGENTS §2 红线）
//! ⇒ 此处以测试替身承载契约语义的可执行文档 + API 形状锁（含 Send 超 trait 锁）。
//!
//! TDD RED 复现（A 强化协议 §7）：本文件单独应用到父树（`game_platform` 尚无
//! AudioOut/常量）⇒ `cargo test -p game-platform --test audio_out_contract`
//! 编译失败（E0432）即红。

use game_platform::{AudioOut, CHANNELS, SAMPLE_RATE};

/// 测试替身：记录写入帧的内存设备（不触硬件）。
#[derive(Default)]
struct MemDevice {
    written: Vec<i16>,
    idles: u32,
    fail: bool,
}

impl AudioOut for MemDevice {
    fn write(&mut self, frames: &[i16]) -> Result<(), String> {
        if self.fail {
            return Err("设备不可恢复故障".to_string());
        }
        self.written.extend_from_slice(frames);
        Ok(())
    }
    fn idle(&mut self) {
        self.idles += 1;
    }
}

/// 线上格式常量 = 开设备参数（shell/win `audio.rs:20-21` 同值；winmm/ALSA open 用之）。
#[test]
fn pcm_wire_format_constants() {
    assert_eq!(SAMPLE_RATE, 44_100, "44.1 kHz（OGG 产物与 Java Player 同源）");
    assert_eq!(CHANNELS, 2, "交错立体声 i16");
}

/// write 的帧形状：交错 i16、以 CHANNELS 对齐（audio_loop 缓冲 = CHUNK_FRAMES×CHANNELS）。
#[test]
fn write_accepts_interleaved_frames() {
    let mut d = MemDevice::default();
    let frames = vec![0i16; CHANNELS * 4];
    d.write(&frames).unwrap();
    assert_eq!(d.written.len(), CHANNELS * 4, "逐帧透传，不重排");
}

/// Err = 不可恢复（上层降级无声，`audio.rs:603-611`）；携带人话诊断。
#[test]
fn write_err_is_unrecoverable_signal() {
    let mut d = MemDevice { fail: true, ..Default::default() };
    let err = d.write(&[0i16; CHANNELS]).unwrap_err();
    assert!(!err.is_empty(), "Err 携带诊断文本");
}

/// idle = 停止并丢弃缓冲（空闲路径 `audio.rs:586-587`：先 idle 再阻塞等命令）。
#[test]
fn idle_marks_device_idle() {
    let mut d = MemDevice::default();
    d.idle();
    d.idle();
    assert_eq!(d.idles, 2);
}

/// 形状锁①：对象可用（audio_loop 以 `Box<dyn AudioOut>` 持有设备）。
#[test]
fn audio_out_is_object_safe() {
    fn via_dyn(d: &mut dyn AudioOut) {
        d.idle();
    }
    let mut d = MemDevice::default();
    via_dyn(&mut d);
}

/// 形状锁②：**Send 超 trait**——设备整体 move 进音频线程（`audio.rs:582`
/// `Box<dyn PcmDevice>` move 进 `audio_loop`；AlsaPcm/WinMmPcm 均 `unsafe impl Send`）。
#[test]
fn audio_out_is_send_across_threads() {
    let mut d = MemDevice::default();
    d.write(&[1i16; CHANNELS * 2]).unwrap();
    let dev: Box<dyn AudioOut> = Box::new(d);
    let dev = std::thread::spawn(move || {
        let mut dev = dev;
        dev.idle(); // 音频线程内消费
        dev
    })
    .join()
    .unwrap();
    drop(dev);
}
