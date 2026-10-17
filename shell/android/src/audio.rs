//! AudioOut 实现：PCM 输出（AAudio 阻塞 write = 节拍语义同构）。
//!
//! # 契约（docs/spec/platform-contract.md §3）
//!
//! `write(&[i16])` 阻塞到设备收下（**阻塞 write = 节拍**：音频线程以 write 返回速度
//! 推进，audio_loop 每块 CHUNK_FRAMES=1024 帧 ≈23ms——game-shell-common::audio）；
//! `idle()` 停止并丢弃缓冲；Err = 不可恢复（上层转无声）。线上格式 = 44.1 kHz
//! 交错立体声 i16（[`game_platform::SAMPLE_RATE`] / [`game_platform::CHANNELS`]）。
//!
//! # 平台映射
//!
//! - 宿主桩 [`CountingPcm`]：记录契约语义（全量写/Err 不可恢复/idle 丢弃）——宿主测试。
//! - Android [`aaudio::AAudioPcm`]：AAudio 阻塞 write（API 26+，不开 data callback）。
//!   **AudioTrack JNI 兜底已砍**（对抗审查 §3-1 裁定）：API<26 或 AAudio 失败 ⇒
//!   `NullSink` 无声降级（同 shell/win `start_without_assets_degrades_silently` 式），
//!   minSdk 统一 26。共享混音/解码/命令翻译消费 `game-shell-common::audio`
//!   （Mixer/Track/decode_ogg/audio_loop/BgmController——不落拷贝，F-4 裁决）。

use game_platform::{AudioOut, CHANNELS};

/// 宿主桩 PCM 设备：记录契约语义（不触硬件）。
///
/// 诚实边界：阻塞语义（write 阻塞到设备收下）**无法**在桩里模拟——桩只锁
/// "全量写 or 不可恢复 Err" 的契约形状；真阻塞语义由 AAudio 骨架承载（未验证）。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CountingPcm {
    /// 已收帧数（每帧 = CHANNELS 个 i16 样本）。
    pub frames_written: u64,
    /// idle() 调用次数。
    pub idle_calls: u64,
    /// 桩故障注入：累计帧数将超过该值时 write 返回不可恢复 Err。
    pub fail_after_frames: Option<u64>,
}

impl CountingPcm {
    pub fn new() -> CountingPcm {
        CountingPcm::default()
    }

    /// 故障注入构造（模拟设备失联）。
    pub fn failing_at(frames: u64) -> CountingPcm {
        CountingPcm { fail_after_frames: Some(frames), ..CountingPcm::default() }
    }
}

impl AudioOut for CountingPcm {
    fn write(&mut self, frames: &[i16]) -> Result<(), String> {
        let n = (frames.len() / CHANNELS) as u64;
        if let Some(limit) = self.fail_after_frames {
            if self.frames_written + n > limit {
                return Err(format!("桩设备故障（fail_after_frames={limit}）——不可恢复"));
            }
        }
        self.frames_written += n;
        Ok(())
    }

    fn idle(&mut self) {
        self.idle_calls += 1;
    }
}

/// Android AAudio 输出（`#[cfg(target_os = "android")]` 门控；宿主编译走 [`CountingPcm`]）。
#[cfg(target_os = "android")]
pub mod aaudio {
    //! TODO(骨架·未验证)：AAudio NDK 直绑（android/aaudio/AAudio.h，API 26+，
    //! libaaudio.so）。本环境无 SDK/NDK——extern 签名与常量照 NDK 头文件手写，
    //! **未经编译验证**（README 验证状态矩阵标注）。
    //!
    //! 阻塞语义：不开 data callback ⇒ `AAudioStream_write` 为阻塞模式；
    //! timeoutNanoseconds 负值 = 无限等待（NDK 文档口径——实现期核对升 A）。
    use game_platform::{AudioOut, CHANNELS, SAMPLE_RATE};
    use std::ffi::{c_char, c_void};

    /// AAudioStream（不透明）。
    #[repr(C)]
    pub struct AAudioStream {
        _private: [u8; 0],
    }
    /// AAudioStreamBuilder（不透明）。
    #[repr(C)]
    pub struct AAudioStreamBuilder {
        _private: [u8; 0],
    }

    pub const AAUDIO_OK: i32 = 0;
    pub const AAUDIO_DIRECTION_OUTPUT: i32 = 0;
    pub const AAUDIO_FORMAT_I16: i32 = 1;
    pub const AAUDIO_SHARING_MODE_EXCLUSIVE: i32 = 0;
    pub const AAUDIO_SHARING_MODE_SHARED: i32 = 1;
    pub const AAUDIO_PERFORMANCE_MODE_LOW_LATENCY: i32 = 1;
    /// 阻塞 write：负 timeout = 无限等待（NDK 口径，未验证）。
    pub const TIMEOUT_INFINITE_NS: i64 = -1;

    #[link(name = "aaudio")]
    extern "C" {
        fn AAudio_createStreamBuilder(builder: *mut *mut AAudioStreamBuilder) -> i32;
        fn AAudioStreamBuilder_delete(builder: *mut AAudioStreamBuilder) -> i32;
        fn AAudioStreamBuilder_setSampleRate(builder: *mut AAudioStreamBuilder, rate: i32);
        fn AAudioStreamBuilder_setChannelCount(builder: *mut AAudioStreamBuilder, channels: i32);
        fn AAudioStreamBuilder_setFormat(builder: *mut AAudioStreamBuilder, format: i32);
        fn AAudioStreamBuilder_setDirection(builder: *mut AAudioStreamBuilder, dir: i32);
        fn AAudioStreamBuilder_setSharingMode(builder: *mut AAudioStreamBuilder, mode: i32);
        fn AAudioStreamBuilder_setPerformanceMode(builder: *mut AAudioStreamBuilder, mode: i32);
        fn AAudioStreamBuilder_openStream(
            builder: *mut AAudioStreamBuilder,
            stream: *mut *mut AAudioStream,
        ) -> i32;
        fn AAudioStream_write(
            stream: *mut AAudioStream,
            buffer: *const c_void,
            num_frames: i32,
            timeout_nanos: i64,
        ) -> i32;
        fn AAudioStream_requestStop(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_close(stream: *mut AAudioStream) -> i32;
        fn AAudio_convertResultToText(result: i32) -> *const c_char;
    }

    fn err_text(result: i32) -> String {
        // SAFETY：convertResultToText 返回静态字符串（NDK 保证非空）。
        unsafe {
            let p = AAudio_convertResultToText(result);
            if p.is_null() {
                return format!("AAudio error {result}");
            }
            std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
        }
    }

    /// Android AAudio 设备句柄（阻塞 write 的 `AudioOut` 实现）。
    pub struct AAudioPcm {
        stream: *mut AAudioStream,
    }

    // SAFETY：设备句柄只在音频线程使用（构造后整体 move 进 audio_loop 线程；
    // AlsaPcm/WinMmPcm 同式，shell/win/src/audio.rs:250/437）。
    unsafe impl Send for AAudioPcm {}

    impl AAudioPcm {
        /// 开 AAudio 输出流（EXCLUSIVE 失败 ⇒ 回退 SHARED）。
        pub fn open() -> Result<AAudioPcm, String> {
            // SAFETY：builder/stream 出参有效；失败路径逐级清理。
            unsafe {
                let mut builder: *mut AAudioStreamBuilder = std::ptr::null_mut();
                let r = AAudio_createStreamBuilder(&mut builder);
                if r != AAUDIO_OK {
                    return Err(format!("AAudio_createStreamBuilder：{}", err_text(r)));
                }
                AAudioStreamBuilder_setSampleRate(builder, SAMPLE_RATE as i32);
                AAudioStreamBuilder_setChannelCount(builder, CHANNELS as i32);
                AAudioStreamBuilder_setFormat(builder, AAUDIO_FORMAT_I16);
                AAudioStreamBuilder_setDirection(builder, AAUDIO_DIRECTION_OUTPUT);
                AAudioStreamBuilder_setPerformanceMode(builder, AAUDIO_PERFORMANCE_MODE_LOW_LATENCY);
                AAudioStreamBuilder_setSharingMode(builder, AAUDIO_SHARING_MODE_EXCLUSIVE);
                let mut stream: *mut AAudioStream = std::ptr::null_mut();
                let mut r = AAudioStreamBuilder_openStream(builder, &mut stream);
                if r != AAUDIO_OK {
                    AAudioStreamBuilder_setSharingMode(builder, AAUDIO_SHARING_MODE_SHARED);
                    r = AAudioStreamBuilder_openStream(builder, &mut stream);
                }
                let _ = AAudioStreamBuilder_delete(builder);
                if r != AAUDIO_OK || stream.is_null() {
                    return Err(format!("AAudioStreamBuilder_openStream：{}", err_text(r)));
                }
                Ok(AAudioPcm { stream })
            }
        }
    }

    impl AudioOut for AAudioPcm {
        fn write(&mut self, frames: &[i16]) -> Result<(), String> {
            // SAFETY：stream 为 open 成功句柄；frames 是交错 i16，帧数 = len/CHANNELS。
            unsafe {
                let total = (frames.len() / CHANNELS) as i32;
                let mut off = 0i32;
                while off < total {
                    let ptr = frames.as_ptr().add((off as usize) * CHANNELS) as *const c_void;
                    let n = AAudioStream_write(self.stream, ptr, total - off, TIMEOUT_INFINITE_NS);
                    if n < 0 {
                        return Err(format!("AAudioStream_write：{}", err_text(n)));
                    }
                    off += n;
                }
                Ok(())
            }
        }

        fn idle(&mut self) {
            // SAFETY：stream 有效；requestStop 停流并丢弃未播缓冲（残响切断）。
            unsafe {
                let _ = AAudioStream_requestStop(self.stream);
            }
        }
    }

    impl Drop for AAudioPcm {
        fn drop(&mut self) {
            // SAFETY：句柄仅在此关闭一次。
            unsafe {
                let _ = AAudioStream_close(self.stream);
            }
        }
    }
}
