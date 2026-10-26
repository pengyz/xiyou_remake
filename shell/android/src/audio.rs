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
//! - Android `aaudio::AAudioPcm` = 流状态机 [`StreamPcm`] × NDK 流 `aaudio::NdkStream`
//!   （libaaudio 直绑，API 26+，不开 data callback）。状态机宿主可测（[`StreamOps`] 假流，
//!   tests/aaudio_stream.rs）；实机契约探针 examples/aaudio_probe.rs（adb shell 跑、只写静音）。
//!   **AudioTrack JNI 兜底已砍**（对抗审查 §3-1 裁定）：API<26 或 AAudio 失败 ⇒
//!   `NullSink` 无声降级（同 shell/win `start_without_assets_degrades_silently` 式），
//!   minSdk 统一 26。共享混音/解码/命令翻译消费 `game-shell-common::audio`
//!   （Mixer/Track/decode_ogg/audio_loop/BgmController——不落拷贝，F-4 裁决）。

use game_platform::{AssetSource, AudioOut, CHANNELS, SAMPLE_RATE};
use game_shell_common::audio::{decode_ogg, Track, TrackStore};
use std::sync::{Arc, Mutex};

/// 宿主桩 PCM 设备：记录契约语义（不触硬件）。
///
/// 诚实边界：阻塞语义（write 阻塞到设备收下）**无法**在桩里模拟——桩只锁
/// "全量写 or 不可恢复 Err" 的契约形状；真阻塞语义由 AAudio 实现承载（实机探针核对）。
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

/// 音频轨装载（**B2 第五缝 Android 口径**：轨道字节走 [`AssetSource`]，不得沿用
/// 路径注入——契约 §0/§4.2 登记；Windows 仍走 `--audio` 目录 = win 专用）。
///
/// `wanted` = 要装载的轨道下标（可达轨道：`game_core::audio::TRACK_MENU`/`TRACK_GAME`；
/// BUG-010 死资源不解码，同 shell/win `start()` 预取顺序）；`slots` = 轨道表总格数
/// （`game_core::audio::TRACKS.len()`）。缺失/解码失败 ⇒ 该格 `None` 且日志
/// （静音降级，同 shell/win 口径），**不 panic**。解码/循环点/混音语义单源在
/// `game-shell-common::audio`（消费不拷贝，F-4 裁决）。
pub fn load_tracks_from_assets(
    source: &dyn AssetSource,
    wanted: &[usize],
    slots: usize,
) -> TrackStore {
    let mut store: Vec<Option<Arc<Track>>> = vec![None; slots];
    for &i in wanted {
        if i >= slots {
            continue; // 越界防御：忽略
        }
        match source.read(&format!("track{i}.ogg")).and_then(|b| decode_ogg(&b)) {
            Ok(t) => store[i] = Some(Arc::new(t)),
            Err(e) => eprintln!("[audio] track{i}.ogg 不可用（{e}）——该轨静音"),
        }
    }
    Arc::new(Mutex::new(store))
}

// ---------------------------------------------------------------- AAudio

/// NDK `aaudio/AAudio.h` 常量（NDK 27.1.12297006 sysroot；`:行号` = 该头文件）——宿主可见，
/// tests/aaudio_stream.rs 钉值。
///
/// 【2026-10-26 实机根因①】旧手写 `PERFORMANCE_MODE_LOW_LATENCY = 1`：头文件该枚举自
/// `NONE = 10` 起算（:325-338）⇒ `AudioStreamBuilder::validate()` 落 default 臂（AOSP
/// android-16.0.0_r1 media/libaaudio/src/core/AudioStreamBuilder.cpp:266-282）⇒ openStream
/// 恒 `AAUDIO_ERROR_ILLEGAL_ARGUMENT`（EXCLUSIVE/SHARED 两次皆然）⇒ 全程静音降级。
/// 实机 logcat（examples/aaudio_probe.rs）：`E/AudioStreamBuilder: illegal performanceMode = 1`。
pub mod abi {
    pub const AAUDIO_OK: i32 = 0; // :136
    // 错误码：`AAUDIO_ERROR_BASE = -900`（:141）起逐一递增（含 :161/:173/:179 的显式跳号）。
    pub const AAUDIO_ERROR_DISCONNECTED: i32 = -899; // :150
    pub const AAUDIO_ERROR_ILLEGAL_ARGUMENT: i32 = -898; // :155
    pub const AAUDIO_ERROR_INTERNAL: i32 = -896; // :161 = ILLEGAL_ARGUMENT + 2
    pub const AAUDIO_ERROR_INVALID_STATE: i32 = -895; // :166
    pub const AAUDIO_ERROR_INVALID_HANDLE: i32 = -892; // :173 = INVALID_STATE + 3
    pub const AAUDIO_ERROR_UNIMPLEMENTED: i32 = -890; // :179 = INVALID_HANDLE + 2
    pub const AAUDIO_ERROR_UNAVAILABLE: i32 = -889; // :186
    pub const AAUDIO_ERROR_NO_FREE_HANDLES: i32 = -888; // :191
    pub const AAUDIO_ERROR_NO_MEMORY: i32 = -887; // :196
    pub const AAUDIO_ERROR_NULL: i32 = -886; // :202
    pub const AAUDIO_ERROR_TIMEOUT: i32 = -885; // :207
    pub const AAUDIO_ERROR_WOULD_BLOCK: i32 = -884; // :212
    pub const AAUDIO_ERROR_INVALID_FORMAT: i32 = -883; // :217
    pub const AAUDIO_ERROR_OUT_OF_RANGE: i32 = -882; // :222
    pub const AAUDIO_ERROR_NO_SERVICE: i32 = -881; // :227
    pub const AAUDIO_ERROR_INVALID_RATE: i32 = -880; // :232

    pub const AAUDIO_DIRECTION_OUTPUT: i32 = 0; // :52
    pub const AAUDIO_FORMAT_PCM_I16: i32 = 1; // :70（INVALID=-1、UNSPECIFIED=0 之后）
    pub const AAUDIO_SHARING_MODE_EXCLUSIVE: i32 = 0; // :311
    pub const AAUDIO_SHARING_MODE_SHARED: i32 = 1; // :316
    pub const AAUDIO_PERFORMANCE_MODE_NONE: i32 = 10; // :325
    pub const AAUDIO_PERFORMANCE_MODE_POWER_SAVING: i32 = 11; // :333
    pub const AAUDIO_PERFORMANCE_MODE_LOW_LATENCY: i32 = 12; // :338

    // `aaudio_stream_state_t`：`UNINITIALIZED = 0`（:246）起逐一递增。
    pub const AAUDIO_STREAM_STATE_OPEN: i32 = 2; // :255
    pub const AAUDIO_STREAM_STATE_STARTING: i32 = 3; // :259
    pub const AAUDIO_STREAM_STATE_STARTED: i32 = 4; // :263
    pub const AAUDIO_STREAM_STATE_PAUSING: i32 = 5; // :267
    pub const AAUDIO_STREAM_STATE_PAUSED: i32 = 6; // :271
    pub const AAUDIO_STREAM_STATE_FLUSHING: i32 = 7; // :275
    pub const AAUDIO_STREAM_STATE_FLUSHED: i32 = 8; // :279
    pub const AAUDIO_STREAM_STATE_STOPPING: i32 = 9; // :283
    pub const AAUDIO_STREAM_STATE_STOPPED: i32 = 10; // :287

    /// 结果码人话名（日志/Err 诊断；未知码给数值）。
    pub fn result_name(r: i32) -> String {
        let name = match r {
            AAUDIO_OK => "AAUDIO_OK",
            AAUDIO_ERROR_DISCONNECTED => "AAUDIO_ERROR_DISCONNECTED",
            AAUDIO_ERROR_ILLEGAL_ARGUMENT => "AAUDIO_ERROR_ILLEGAL_ARGUMENT",
            AAUDIO_ERROR_INTERNAL => "AAUDIO_ERROR_INTERNAL",
            AAUDIO_ERROR_INVALID_STATE => "AAUDIO_ERROR_INVALID_STATE",
            AAUDIO_ERROR_INVALID_HANDLE => "AAUDIO_ERROR_INVALID_HANDLE",
            AAUDIO_ERROR_UNIMPLEMENTED => "AAUDIO_ERROR_UNIMPLEMENTED",
            AAUDIO_ERROR_UNAVAILABLE => "AAUDIO_ERROR_UNAVAILABLE",
            AAUDIO_ERROR_NO_FREE_HANDLES => "AAUDIO_ERROR_NO_FREE_HANDLES",
            AAUDIO_ERROR_NO_MEMORY => "AAUDIO_ERROR_NO_MEMORY",
            AAUDIO_ERROR_NULL => "AAUDIO_ERROR_NULL",
            AAUDIO_ERROR_TIMEOUT => "AAUDIO_ERROR_TIMEOUT",
            AAUDIO_ERROR_WOULD_BLOCK => "AAUDIO_ERROR_WOULD_BLOCK",
            AAUDIO_ERROR_INVALID_FORMAT => "AAUDIO_ERROR_INVALID_FORMAT",
            AAUDIO_ERROR_OUT_OF_RANGE => "AAUDIO_ERROR_OUT_OF_RANGE",
            AAUDIO_ERROR_NO_SERVICE => "AAUDIO_ERROR_NO_SERVICE",
            AAUDIO_ERROR_INVALID_RATE => "AAUDIO_ERROR_INVALID_RATE",
            _ => return format!("AAudio result {r}"),
        };
        format!("{name}({r})")
    }
}

use abi::*;

/// AAudio 输出流打开参数（宿主可见，tests 钉值；`aaudio::NdkStream::open` 逐项下发）。
///
/// 取舍（2026-10-26）：**SHARED + PERFORMANCE_MODE_NONE**（旧骨架 EXCLUSIVE→SHARED 回退 +
/// LOW_LATENCY，因常量错值从未开成过）。本流常驻整个进程（曲间 pause 不 close），而
/// AAudio.h:309 要求 EXCLUSIVE 流不用即关；BGM 无低延迟需求，只有 LOW_LATENCY 才走
/// MMAP 小缓冲（AudioStreamBuilder.cpp:174-175），非实时写线程易欠载；NONE 恒走 legacy
/// AudioTrack 路径——与实机探针同一路径（adb shell 的 uid 2000 被 HyperOS 收为 MMAP NEVER，
/// 探针日志 `final mmap policy is 1`），探针结论可直接外推到游戏进程。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamConfig {
    pub sample_rate: i32,
    pub channel_count: i32,
    pub format: i32,
    pub direction: i32,
    pub sharing_mode: i32,
    pub performance_mode: i32,
}

/// 游戏 BGM 输出流参数（契约线上格式 44.1 kHz 交错立体声 i16）。
pub const STREAM_CONFIG: StreamConfig = StreamConfig {
    sample_rate: SAMPLE_RATE as i32,
    channel_count: CHANNELS as i32,
    format: AAUDIO_FORMAT_PCM_I16,
    direction: AAUDIO_DIRECTION_OUTPUT,
    sharing_mode: AAUDIO_SHARING_MODE_SHARED,
    performance_mode: AAUDIO_PERFORMANCE_MODE_NONE,
};

/// 写超时：**正值 = 阻塞写**。负/零 = 不阻塞——legacy 路径 AudioStreamTrack.cpp:505
/// `blocking = timeoutNanoseconds > 0`；MMAP 路径 AudioStreamInternal.cpp:840/873 截止 =
/// now + timeout，负值首轮即"超时"返回（旧骨架注释"负值 = 无限等待"无出处：AAudio.h:1693/1701
/// 只写"零 = 不等待""超时返回 0 或部分帧数"）。1 s ≫ 任一档缓冲时长：在放的流必在其内收下一块。
pub const WRITE_TIMEOUT_NS: i64 = 1_000_000_000;

/// idle 等 PAUSING→PAUSED 的单次时限与次数（requestPause 异步，AAudio.h:1566；
/// flush 只在 OPEN/PAUSED/STOPPED/FLUSHED 允许，AAudio.h:1585）。
pub const IDLE_SETTLE_NS: i64 = 100_000_000;
pub const IDLE_SETTLE_TRIES: usize = 3;

/// AAudio 流的最小操作面（返回值口径 = NDK `aaudio_result_t`：负 = 错误）。Android 实现 =
/// `aaudio::NdkStream`（libaaudio 直绑）；宿主测试 = 假流（tests/aaudio_stream.rs）。
pub trait StreamOps {
    /// `AAudioStream_requestStart`。
    fn request_start(&mut self) -> i32;
    /// `AAudioStream_requestPause`（冻结数据流，不丢缓冲）。
    fn request_pause(&mut self) -> i32;
    /// `AAudioStream_requestFlush`（丢弃未播数据）。
    fn request_flush(&mut self) -> i32;
    /// `AAudioStream_requestStop`（播完已缓冲数据后停）。
    fn request_stop(&mut self) -> i32;
    /// `AAudioStream_getState`。
    fn state(&mut self) -> i32;
    /// `AAudioStream_waitForStateChange`：返回 (result, nextState)。
    fn wait_for_state_change(&mut self, input: i32, timeout_ns: i64) -> (i32, i32);
    /// `AAudioStream_write`（交错 i16，帧数 = len/CHANNELS）：已写帧数或负错误。
    fn write(&mut self, frames: &[i16], timeout_ns: i64) -> i32;
}

/// AAudio 阻塞写设备：[`AudioOut`] 契约 → AAudio 流状态机。
///
/// - `write`：未启动先 `requestStart`——open 后流处于 OPEN，不启动不消费（【实机根因②】
///   旧实现从不 start + 负 timeout：实机探针写 2 s 静音 6 s 未完、CPU 5.97 s/6 s 自旋）；
///   每次带正 [`WRITE_TIMEOUT_NS`]、分段写满；超时仍 0 帧 = 流不消费 ⇒ Err（绝不自旋）。
/// - `idle`：停止并丢弃缓冲 = requestPause → 等 PAUSED → requestFlush；下一次 write 重新
///   start。未启动 = 无操作（audio_loop 起手即 idle）。pause 被拒 ⇒ 退化 requestStop。
pub struct StreamPcm<S: StreamOps> {
    stream: S,
    started: bool,
}

impl<S: StreamOps> StreamPcm<S> {
    /// 包装一条已打开（OPEN 态）的流。
    pub fn new(stream: S) -> StreamPcm<S> {
        StreamPcm { stream, started: false }
    }

    /// 底层流（诊断用）。
    pub fn stream(&self) -> &S {
        &self.stream
    }
}

impl<S: StreamOps + Send> AudioOut for StreamPcm<S> {
    fn write(&mut self, frames: &[i16]) -> Result<(), String> {
        let total = frames.len() / CHANNELS;
        let mut off = 0usize;
        while off < total {
            if !self.started {
                let r = self.stream.request_start();
                if r != AAUDIO_OK {
                    return Err(format!("AAudioStream_requestStart：{}", result_name(r)));
                }
                self.started = true;
            }
            let n = self.stream.write(&frames[off * CHANNELS..total * CHANNELS], WRITE_TIMEOUT_NS);
            if n < 0 {
                return Err(format!("AAudioStream_write：{}", result_name(n)));
            }
            if n == 0 {
                return Err(format!(
                    "AAudioStream_write：{} ms 内 0 帧（流不消费）",
                    WRITE_TIMEOUT_NS / 1_000_000
                ));
            }
            off += n as usize;
        }
        Ok(())
    }

    fn idle(&mut self) {
        if !self.started {
            return;
        }
        self.started = false;
        if self.stream.request_pause() != AAUDIO_OK {
            let _ = self.stream.request_stop();
            return;
        }
        let mut state = self.stream.state();
        for _ in 0..IDLE_SETTLE_TRIES {
            if state == AAUDIO_STREAM_STATE_PAUSED {
                break;
            }
            let (r, next) = self.stream.wait_for_state_change(state, IDLE_SETTLE_NS);
            if r != AAUDIO_OK {
                break;
            }
            state = next;
        }
        let _ = self.stream.request_flush();
    }
}

/// Android AAudio 输出（`#[cfg(target_os = "android")]` 门控；宿主编译走 [`CountingPcm`]）。
#[cfg(target_os = "android")]
pub mod aaudio {
    //! AAudio NDK 直绑（android/aaudio/AAudio.h，API 26+，libaaudio.so）：extern 签名照
    //! NDK 27.1.12297006 头文件；常量/打开参数/状态机在父模块（宿主可测）。
    use super::abi::*;
    use super::{StreamOps, StreamPcm, STREAM_CONFIG};
    use std::ffi::c_void;

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
        fn AAudioStream_requestStart(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_requestPause(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_requestFlush(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_requestStop(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_getState(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_waitForStateChange(
            stream: *mut AAudioStream,
            input_state: i32,
            next_state: *mut i32,
            timeout_nanos: i64,
        ) -> i32;
        fn AAudioStream_write(
            stream: *mut AAudioStream,
            buffer: *const c_void,
            num_frames: i32,
            timeout_nanos: i64,
        ) -> i32;
        fn AAudioStream_close(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_getSharingMode(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_getPerformanceMode(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_getSampleRate(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_getFramesPerBurst(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_getBufferSizeInFrames(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_getBufferCapacityInFrames(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_getXRunCount(stream: *mut AAudioStream) -> i32;
    }

    /// 一条已打开的 AAudio 输出流（open 成功句柄；Drop 关闭）。
    pub struct NdkStream {
        stream: *mut AAudioStream,
    }

    // SAFETY：句柄只在音频线程使用（构造后整体 move 进 audio_loop 线程；
    // AlsaPcm/WinMmPcm 同式，shell/win/src/audio.rs:250/437）。
    unsafe impl Send for NdkStream {}

    impl NdkStream {
        /// 按 [`STREAM_CONFIG`] 打开输出流（OPEN 态，未启动）。
        pub fn open() -> Result<NdkStream, String> {
            let c = STREAM_CONFIG;
            // SAFETY：builder/stream 出参有效；builder 用后即删。
            unsafe {
                let mut builder: *mut AAudioStreamBuilder = std::ptr::null_mut();
                let r = AAudio_createStreamBuilder(&mut builder);
                if r != AAUDIO_OK {
                    return Err(format!("AAudio_createStreamBuilder：{}", result_name(r)));
                }
                AAudioStreamBuilder_setSampleRate(builder, c.sample_rate);
                AAudioStreamBuilder_setChannelCount(builder, c.channel_count);
                AAudioStreamBuilder_setFormat(builder, c.format);
                AAudioStreamBuilder_setDirection(builder, c.direction);
                AAudioStreamBuilder_setSharingMode(builder, c.sharing_mode);
                AAudioStreamBuilder_setPerformanceMode(builder, c.performance_mode);
                let mut stream: *mut AAudioStream = std::ptr::null_mut();
                let r = AAudioStreamBuilder_openStream(builder, &mut stream);
                let _ = AAudioStreamBuilder_delete(builder);
                if r != AAUDIO_OK || stream.is_null() {
                    return Err(format!("AAudioStreamBuilder_openStream：{}", result_name(r)));
                }
                Ok(NdkStream { stream })
            }
        }

        /// 实得流参数（服务端可能改写请求值）+ 欠载计数——日志/探针用。
        pub fn describe(&self) -> String {
            // SAFETY：stream 为 open 成功句柄；getter 无副作用。
            unsafe {
                format!(
                    "sharing={} perf={} rate={} burst={} buffer={}/{} xrun={}",
                    AAudioStream_getSharingMode(self.stream),
                    AAudioStream_getPerformanceMode(self.stream),
                    AAudioStream_getSampleRate(self.stream),
                    AAudioStream_getFramesPerBurst(self.stream),
                    AAudioStream_getBufferSizeInFrames(self.stream),
                    AAudioStream_getBufferCapacityInFrames(self.stream),
                    AAudioStream_getXRunCount(self.stream),
                )
            }
        }
    }

    impl StreamOps for NdkStream {
        fn request_start(&mut self) -> i32 {
            // SAFETY：stream 有效（以下同）。
            unsafe { AAudioStream_requestStart(self.stream) }
        }

        fn request_pause(&mut self) -> i32 {
            unsafe { AAudioStream_requestPause(self.stream) }
        }

        fn request_flush(&mut self) -> i32 {
            unsafe { AAudioStream_requestFlush(self.stream) }
        }

        fn request_stop(&mut self) -> i32 {
            unsafe { AAudioStream_requestStop(self.stream) }
        }

        fn state(&mut self) -> i32 {
            unsafe { AAudioStream_getState(self.stream) }
        }

        fn wait_for_state_change(&mut self, input: i32, timeout_ns: i64) -> (i32, i32) {
            let mut next = input;
            let r = unsafe { AAudioStream_waitForStateChange(self.stream, input, &mut next, timeout_ns) };
            (r, next)
        }

        fn write(&mut self, frames: &[i16], timeout_ns: i64) -> i32 {
            let n = (frames.len() / game_platform::CHANNELS) as i32;
            // SAFETY：frames 交错 i16，帧数 = len/CHANNELS（只读，不越界）。
            unsafe { AAudioStream_write(self.stream, frames.as_ptr() as *const c_void, n, timeout_ns) }
        }
    }

    impl Drop for NdkStream {
        fn drop(&mut self) {
            // SAFETY：句柄仅在此关闭一次。
            unsafe {
                let _ = AAudioStream_close(self.stream);
            }
        }
    }

    /// Android BGM 设备 = 流状态机 × NDK 流。
    pub type AAudioPcm = StreamPcm<NdkStream>;

    impl StreamPcm<NdkStream> {
        /// 打开设备（失败 ⇒ 调用方无声降级）。
        pub fn open() -> Result<AAudioPcm, String> {
            NdkStream::open().map(StreamPcm::new)
        }

        /// 实得流参数（见 [`NdkStream::describe`]）。
        pub fn describe(&self) -> String {
            self.stream().describe()
        }
    }
}
