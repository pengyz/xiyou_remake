//! AAudio 输出（`audio::StreamPcm` 流状态机 + `abi`/`STREAM_CONFIG`）宿主钉。
//!
//! 实机根因（2026-10-26，examples/aaudio_probe.rs 在 Xiaomi 14 / Android 16 上取证）：
//! ① `PERFORMANCE_MODE_LOW_LATENCY` 手写成 1（头文件 = 12）⇒ openStream 恒 ILLEGAL_ARGUMENT
//!   （logcat `illegal performanceMode = 1`）⇒ BGM 全程静音降级；
//! ② 修①后：从不 requestStart + 负 timeout（AAudio 负/零 = 不阻塞）⇒ 写 2 s 静音 6 s 未完、
//!   CPU 5.97 s/6 s 自旋。
//! 假流按 AAudio 状态规则建模（AAudio.h 状态枚举；AudioStream.cpp systemStart/systemPause；
//! AAudioUtilities AAudio_isFlushAllowed），并按实机观测"未启动的流写入 0 帧立返"。

use game_android::audio::abi::*;
use game_android::audio::{
    StreamOps, StreamPcm, IDLE_SETTLE_TRIES, STREAM_CONFIG, WRITE_TIMEOUT_NS,
};
use game_platform::{AudioOut, CHANNELS, SAMPLE_RATE};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq)]
enum Call {
    Start,
    Pause,
    /// flush 调用时的流状态。
    Flush(i32),
    Stop,
    Wait(i32),
    Write { frames: usize, timeout_ns: i64 },
    /// 流被 Drop（= AAudioStream_close）。
    Close,
}

/// 假 AAudio 流：状态机 + 调用录制（录制表共享，便于设备 move 进线程后核对）。
struct FakeStream {
    log: Arc<Mutex<Vec<Call>>>,
    state: i32,
    /// 每次 write 最多收下的帧数（模拟缓冲只够收一部分）。
    per_call_cap: usize,
    /// 强制 write 返回值（故障注入）。
    write_result: Option<i32>,
    /// 强制 requestStart 返回值（故障注入）。
    start_result: Option<i32>,
    /// requestPause 是否被拒（不支持 pause 的流）。
    pause_unsupported: bool,
    /// 设备切换开关（共享：测试中途翻转）——断开后 write = DISCONNECTED（AudioStreamTrack.cpp:500-502）、
    /// start = INVALID_STATE（AudioStream.cpp:169）。
    disconnected: Arc<AtomicBool>,
}

impl FakeStream {
    fn new() -> (FakeStream, Arc<Mutex<Vec<Call>>>) {
        let log = Arc::new(Mutex::new(Vec::new()));
        let s = FakeStream {
            log: log.clone(),
            state: AAUDIO_STREAM_STATE_OPEN,
            per_call_cap: usize::MAX,
            write_result: None,
            start_result: None,
            pause_unsupported: false,
            disconnected: Arc::new(AtomicBool::new(false)),
        };
        (s, log)
    }

    fn rec(&self, c: Call) {
        self.log.lock().unwrap().push(c);
    }

    fn is_disconnected(&self) -> bool {
        self.disconnected.load(Ordering::SeqCst)
    }
}

impl Drop for FakeStream {
    fn drop(&mut self) {
        self.rec(Call::Close);
    }
}

impl StreamOps for FakeStream {
    fn request_start(&mut self) -> i32 {
        self.rec(Call::Start);
        if let Some(r) = self.start_result {
            return r;
        }
        if self.is_disconnected() {
            return AAUDIO_ERROR_INVALID_STATE;
        }
        match self.state {
            AAUDIO_STREAM_STATE_OPEN
            | AAUDIO_STREAM_STATE_PAUSING
            | AAUDIO_STREAM_STATE_PAUSED
            | AAUDIO_STREAM_STATE_STOPPING
            | AAUDIO_STREAM_STATE_STOPPED
            | AAUDIO_STREAM_STATE_FLUSHING
            | AAUDIO_STREAM_STATE_FLUSHED => {
                self.state = AAUDIO_STREAM_STATE_STARTED;
                AAUDIO_OK
            }
            _ => AAUDIO_ERROR_INVALID_STATE, // 已启动再 start = INVALID_STATE（systemStart）
        }
    }

    fn request_pause(&mut self) -> i32 {
        self.rec(Call::Pause);
        if self.pause_unsupported {
            return AAUDIO_ERROR_UNIMPLEMENTED;
        }
        if self.state == AAUDIO_STREAM_STATE_STARTED {
            self.state = AAUDIO_STREAM_STATE_PAUSING; // 异步：须等状态落定
            AAUDIO_OK
        } else {
            AAUDIO_ERROR_INVALID_STATE
        }
    }

    fn request_flush(&mut self) -> i32 {
        self.rec(Call::Flush(self.state));
        match self.state {
            AAUDIO_STREAM_STATE_OPEN
            | AAUDIO_STREAM_STATE_PAUSED
            | AAUDIO_STREAM_STATE_STOPPED
            | AAUDIO_STREAM_STATE_FLUSHED => {
                self.state = AAUDIO_STREAM_STATE_FLUSHED;
                AAUDIO_OK
            }
            _ => AAUDIO_ERROR_INVALID_STATE, // PAUSING 中 flush 被拒（AAudio_isFlushAllowed）
        }
    }

    fn request_stop(&mut self) -> i32 {
        self.rec(Call::Stop);
        self.state = AAUDIO_STREAM_STATE_STOPPED;
        AAUDIO_OK
    }

    fn state(&mut self) -> i32 {
        self.state
    }

    fn wait_for_state_change(&mut self, input: i32, _timeout_ns: i64) -> (i32, i32) {
        self.rec(Call::Wait(input));
        if self.state == AAUDIO_STREAM_STATE_PAUSING {
            self.state = AAUDIO_STREAM_STATE_PAUSED;
        }
        if self.state == input {
            (AAUDIO_ERROR_TIMEOUT, self.state)
        } else {
            (AAUDIO_OK, self.state)
        }
    }

    fn write(&mut self, frames: &[i16], timeout_ns: i64) -> i32 {
        let n = frames.len() / CHANNELS;
        self.rec(Call::Write { frames: n, timeout_ns });
        if let Some(r) = self.write_result {
            return r;
        }
        if self.is_disconnected() {
            return AAUDIO_ERROR_DISCONNECTED;
        }
        if self.state != AAUDIO_STREAM_STATE_STARTED {
            return 0; // 实机：未启动的流不消费，写入 0 帧立返
        }
        n.min(self.per_call_cap) as i32
    }
}

fn calls(log: &Arc<Mutex<Vec<Call>>>) -> Vec<Call> {
    log.lock().unwrap().clone()
}

const CHUNK: [i16; 1024 * 2] = [0i16; 1024 * 2];

/// 头文件常量钉值（NDK 27.1.12297006 sysroot aaudio/AAudio.h；行号见 audio.rs abi 注释）。
/// 性能模式枚举自 NONE = 10 起算——旧手写 LOW_LATENCY = 1 即实机根因①。
#[test]
fn abi_constants_match_ndk_header() {
    assert_eq!(
        (AAUDIO_PERFORMANCE_MODE_NONE, AAUDIO_PERFORMANCE_MODE_POWER_SAVING, AAUDIO_PERFORMANCE_MODE_LOW_LATENCY),
        (10, 11, 12),
        "AAudio.h:325-338"
    );
    assert_eq!((AAUDIO_SHARING_MODE_EXCLUSIVE, AAUDIO_SHARING_MODE_SHARED), (0, 1), "AAudio.h:311/316");
    assert_eq!((AAUDIO_DIRECTION_OUTPUT, AAUDIO_FORMAT_PCM_I16), (0, 1), "AAudio.h:52/70");
    assert_eq!(
        (AAUDIO_ERROR_DISCONNECTED, AAUDIO_ERROR_ILLEGAL_ARGUMENT, AAUDIO_ERROR_INVALID_STATE, AAUDIO_ERROR_TIMEOUT),
        (-899, -898, -895, -885),
        "AAudio.h:141 BASE=-900 起算；实机日志 -898 = AAUDIO_ERROR_ILLEGAL_ARGUMENT"
    );
    assert_eq!(
        (AAUDIO_STREAM_STATE_OPEN, AAUDIO_STREAM_STATE_STARTED, AAUDIO_STREAM_STATE_PAUSING, AAUDIO_STREAM_STATE_PAUSED),
        (2, 4, 5, 6),
        "AAudio.h:246 UNINITIALIZED=0 起算；实机日志 open 后 setState 0→2"
    );
    assert_eq!(result_name(-898), "AAUDIO_ERROR_ILLEGAL_ARGUMENT(-898)");
    assert_eq!(result_name(-1234), "AAudio result -1234");
}

/// 打开参数过得了 `AudioStreamBuilder::validate()` 的性能模式白名单（AOSP android-16.0.0_r1
/// AudioStreamBuilder.cpp:266-282：NONE/POWER_SAVING/LOW_LATENCY，其余 ILLEGAL_ARGUMENT），
/// 且取 SHARED + NONE（常驻流不占 EXCLUSIVE；BGM 不要低延迟——取舍见 STREAM_CONFIG 注释）。
#[test]
fn stream_config_passes_builder_validation() {
    let c = STREAM_CONFIG;
    assert!(
        [AAUDIO_PERFORMANCE_MODE_NONE, AAUDIO_PERFORMANCE_MODE_POWER_SAVING, AAUDIO_PERFORMANCE_MODE_LOW_LATENCY]
            .contains(&c.performance_mode),
        "性能模式 {} 不在 validate 白名单 ⇒ openStream ILLEGAL_ARGUMENT",
        c.performance_mode
    );
    assert_eq!((c.sharing_mode, c.performance_mode), (AAUDIO_SHARING_MODE_SHARED, AAUDIO_PERFORMANCE_MODE_NONE));
    assert_eq!(
        (c.sample_rate, c.channel_count, c.format, c.direction),
        (SAMPLE_RATE as i32, CHANNELS as i32, AAUDIO_FORMAT_PCM_I16, AAUDIO_DIRECTION_OUTPUT),
        "契约线上格式 44.1 kHz 交错立体声 i16"
    );
}

/// 首写前 requestStart（open 后 OPEN 态不消费），且只启动一次（已启动再 start = INVALID_STATE）。
#[test]
fn write_starts_stream_once_before_first_write() {
    let (s, log) = FakeStream::new();
    let mut pcm = StreamPcm::new(s);
    assert_eq!(pcm.write(&CHUNK), Ok(()));
    assert_eq!(pcm.write(&CHUNK), Ok(()));
    let c = calls(&log);
    assert_eq!(c.first(), Some(&Call::Start), "首个调用 = requestStart：{c:?}");
    assert_eq!(c.iter().filter(|x| **x == Call::Start).count(), 1, "只启动一次：{c:?}");
    assert_eq!(c.iter().filter(|x| matches!(x, Call::Write { .. })).count(), 2);
}

/// 每次 write 带正 timeout（负/零 = 不阻塞 ⇒ 实机自旋）；缓冲只收一部分时分段写满。
#[test]
fn write_blocks_with_positive_timeout_and_loops_partial_writes() {
    let (mut s, log) = FakeStream::new();
    s.per_call_cap = 300;
    let mut pcm = StreamPcm::new(s);
    assert_eq!(pcm.write(&CHUNK), Ok(()));
    let writes: Vec<(usize, i64)> = calls(&log)
        .into_iter()
        .filter_map(|c| match c {
            Call::Write { frames, timeout_ns } => Some((frames, timeout_ns)),
            _ => None,
        })
        .collect();
    assert!(WRITE_TIMEOUT_NS > 0);
    assert!(writes.iter().all(|&(_, t)| t == WRITE_TIMEOUT_NS), "全部阻塞写：{writes:?}");
    assert_eq!(
        writes.iter().map(|&(f, _)| f).collect::<Vec<_>>(),
        vec![1024, 724, 424, 124],
        "余量逐段续写直至 1024 帧收完"
    );
}

/// 超时仍 0 帧 = 流不消费 ⇒ Err（只写一次，绝不自旋）；负结果 ⇒ Err 带人话码名。
#[test]
fn stalled_or_failed_write_is_err_not_spin() {
    let (mut s, log) = FakeStream::new();
    s.write_result = Some(0);
    let mut pcm = StreamPcm::new(s);
    let e = pcm.write(&CHUNK).unwrap_err();
    assert!(e.contains("0 帧"), "{e}");
    assert_eq!(calls(&log).iter().filter(|c| matches!(c, Call::Write { .. })).count(), 1, "不自旋");

    let (mut s, _) = FakeStream::new();
    s.write_result = Some(AAUDIO_ERROR_DISCONNECTED);
    let e = StreamPcm::new(s).write(&CHUNK).unwrap_err();
    assert!(e.contains("AAUDIO_ERROR_DISCONNECTED(-899)"), "{e}");

    let (mut s, _) = FakeStream::new();
    s.start_result = Some(AAUDIO_ERROR_INVALID_STATE);
    let e = StreamPcm::new(s).write(&CHUNK).unwrap_err();
    assert!(e.contains("requestStart") && e.contains("INVALID_STATE"), "{e}");
}

/// idle = 停止并丢弃缓冲：pause → 等 PAUSING 落定为 PAUSED → flush（PAUSED 态才允许）；
/// 下一次 write 重新 start。
#[test]
fn idle_pauses_settles_flushes_and_next_write_restarts() {
    let (s, log) = FakeStream::new();
    let mut pcm = StreamPcm::new(s);
    pcm.write(&CHUNK).unwrap();
    log.lock().unwrap().clear();
    pcm.idle();
    assert_eq!(
        calls(&log),
        vec![Call::Pause, Call::Wait(AAUDIO_STREAM_STATE_PAUSING), Call::Flush(AAUDIO_STREAM_STATE_PAUSED)],
        "pause → 等 PAUSED → flush"
    );
    log.lock().unwrap().clear();
    pcm.write(&CHUNK).unwrap();
    assert_eq!(calls(&log).first(), Some(&Call::Start), "idle 后再写须重新启动");
    pcm.idle();
    pcm.idle();
    let pauses = calls(&log).iter().filter(|c| **c == Call::Pause).count();
    assert_eq!(pauses, 1, "已停再 idle = 无操作");
}

/// 未启动即 idle（audio_loop 起手空闲分支）= 无任何流调用。
#[test]
fn idle_before_first_write_is_noop() {
    let (s, log) = FakeStream::new();
    let mut pcm = StreamPcm::new(s);
    pcm.idle();
    assert!(calls(&log).is_empty(), "{:?}", calls(&log));
}

/// pause 不受支持 ⇒ 退化 requestStop（不 flush、不卡等待）；等待次数有上限。
#[test]
fn idle_falls_back_to_stop_and_wait_is_bounded() {
    let (mut s, log) = FakeStream::new();
    s.pause_unsupported = true;
    let mut pcm = StreamPcm::new(s);
    pcm.write(&CHUNK).unwrap();
    log.lock().unwrap().clear();
    pcm.idle();
    assert_eq!(calls(&log), vec![Call::Pause, Call::Stop]);

    /// 永不落定的流：PAUSING 卡住（等待恒超时）。
    struct Stuck(FakeStream);
    impl StreamOps for Stuck {
        fn request_start(&mut self) -> i32 {
            self.0.request_start()
        }
        fn request_pause(&mut self) -> i32 {
            self.0.request_pause()
        }
        fn request_flush(&mut self) -> i32 {
            self.0.request_flush()
        }
        fn request_stop(&mut self) -> i32 {
            self.0.request_stop()
        }
        fn state(&mut self) -> i32 {
            self.0.state()
        }
        fn wait_for_state_change(&mut self, input: i32, _t: i64) -> (i32, i32) {
            self.0.rec(Call::Wait(input));
            (AAUDIO_OK, input) // 报"变了"却仍是原态：循环只能靠次数上限退出
        }
        fn write(&mut self, f: &[i16], t: i64) -> i32 {
            self.0.write(f, t)
        }
    }
    let (s, log) = FakeStream::new();
    let mut pcm = StreamPcm::new(Stuck(s));
    pcm.write(&CHUNK).unwrap();
    log.lock().unwrap().clear();
    pcm.idle();
    let waits = calls(&log).iter().filter(|c| matches!(c, Call::Wait(_))).count();
    assert_eq!(waits, IDLE_SETTLE_TRIES, "等待有上限：{:?}", calls(&log));
}

/// 设备整体 move 进音频线程（`AudioOut: Send`；audio_loop 持 `Box<dyn AudioOut>`）。
#[test]
fn stream_pcm_is_send_and_object_safe() {
    let (s, log) = FakeStream::new();
    let mut dev: Box<dyn AudioOut> = Box::new(StreamPcm::new(s));
    let h = std::thread::spawn(move || {
        dev.write(&CHUNK).unwrap();
        dev.idle();
    });
    h.join().unwrap();
    assert_eq!(calls(&log).first(), Some(&Call::Start));
}

/// 重开工厂：每次开出一条新假流，登记其录制表与断开开关（测试据此核对/翻转）。
type Opened = Arc<Mutex<Vec<(Arc<Mutex<Vec<Call>>>, Arc<AtomicBool>)>>>;

fn factory(opened: &Opened, fail: bool) -> Box<dyn FnMut() -> Result<FakeStream, String> + Send> {
    let opened = opened.clone();
    Box::new(move || {
        if fail {
            return Err("openStream：AAUDIO_ERROR_NO_SERVICE(-881)".into());
        }
        let (s, log) = FakeStream::new();
        opened.lock().unwrap().push((log, s.disconnected.clone()));
        Ok(s)
    })
}

/// 设备切换（插拔耳机/连蓝牙）：写中途 DISCONNECTED ⇒ 关旧流、开新流、start、余量续写——
/// 本块不丢、调用方无感（Err 才会让 audio_loop 本次运行转无声）。
#[test]
fn disconnect_mid_write_reopens_and_finishes_chunk() {
    let (s, old) = FakeStream::new();
    let flag = s.disconnected.clone();
    let opened: Opened = Arc::new(Mutex::new(Vec::new()));
    let mut pcm = StreamPcm::new(s).with_reopen(factory(&opened, false));
    pcm.write(&CHUNK).unwrap();
    flag.store(true, Ordering::SeqCst); // 设备切换
    assert_eq!(pcm.write(&CHUNK), Ok(()), "断开后重开续写，调用方无感");
    assert_eq!(pcm.reopens(), 1);
    assert_eq!(calls(&old).last(), Some(&Call::Close), "旧流已关：{:?}", calls(&old));
    let fresh = opened.lock().unwrap()[0].0.clone();
    assert_eq!(
        calls(&fresh),
        vec![Call::Start, Call::Write { frames: 1024, timeout_ns: WRITE_TIMEOUT_NS }],
        "新流：先 start 再写满本块"
    );
}

/// 空闲（曲间/后台 pause）期间设备切换：下次 write 的 start 被拒（INVALID_STATE）⇒ 同样重开。
#[test]
fn disconnect_while_idle_reopens_on_next_start() {
    let (s, _old) = FakeStream::new();
    let flag = s.disconnected.clone();
    let opened: Opened = Arc::new(Mutex::new(Vec::new()));
    let mut pcm = StreamPcm::new(s).with_reopen(factory(&opened, false));
    pcm.write(&CHUNK).unwrap();
    pcm.idle();
    flag.store(true, Ordering::SeqCst);
    assert_eq!(pcm.write(&CHUNK), Ok(()));
    assert_eq!(pcm.reopens(), 1);
    assert_eq!(calls(&opened.lock().unwrap()[0].0).first(), Some(&Call::Start));
}

/// 每次 write 至多重开一次：新流也断开 / 重开失败 ⇒ Err（不死循环；契约 Err = 不可恢复）。
#[test]
fn reopen_is_bounded_and_failure_is_err() {
    // 新流开出即断开：本次 write Err，只重开一次
    let (s, _) = FakeStream::new();
    let flag = s.disconnected.clone();
    let opened: Opened = Arc::new(Mutex::new(Vec::new()));
    let opened2 = opened.clone();
    let mut pcm = StreamPcm::new(s).with_reopen(Box::new(move || {
        let (s, log) = FakeStream::new();
        s.disconnected.store(true, Ordering::SeqCst);
        opened2.lock().unwrap().push((log, s.disconnected.clone()));
        Ok(s)
    }));
    pcm.write(&CHUNK).unwrap();
    flag.store(true, Ordering::SeqCst);
    let e = pcm.write(&CHUNK).unwrap_err();
    assert!(e.contains("INVALID_STATE"), "新流 start 被拒：{e}");
    assert_eq!((pcm.reopens(), opened.lock().unwrap().len()), (1, 1), "每次 write 至多重开一次");

    // 重开本身失败：Err 带两段诊断
    let (s, _) = FakeStream::new();
    let flag = s.disconnected.clone();
    let mut pcm = StreamPcm::new(s).with_reopen(factory(&Arc::new(Mutex::new(Vec::new())), true));
    pcm.write(&CHUNK).unwrap();
    flag.store(true, Ordering::SeqCst);
    let e = pcm.write(&CHUNK).unwrap_err();
    assert!(e.contains("DISCONNECTED") && e.contains("重开失败") && e.contains("NO_SERVICE"), "{e}");
}
