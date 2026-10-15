//! BGM 输出（桌面壳侧）。语义在 game-core（`game_core::audio`，docs/spec/audio.md）；
//! 本模块只把「应当在放的轨道」变成声音：
//!
//! - [`BgmController`]（主线程，每逻辑 tick 轮询 `bgm_request/bgm_serial/bgm_volume`）
//!   把状态变化翻译为 [`Cmd`]：轨道或 serial 变 ⇒ `Play`（从头播放，= Java 关闭重建
//!   a.java:10178-10199）；变 None ⇒ `Stop`（closeAudioPlayer）；失焦 ⇒ `Pause`（壳级冻结）。
//! - [`Mixer`]（音频线程）：按 OGG 的 `LOOPSTART/LOOPLENGTH` 无限循环（setLoopCount(-1)
//!   a.java:10184），幅度 = VolumeControl 级别/100（JSR-135：级别线性刻度、乘数映射由
//!   实现决定）。
//! - 输出：Linux 运行时 dlopen `libasound.so.2`（ALSA PCM，无构建期系统依赖——cpal/rodio
//!   需 libasound2-dev 头文件 + pkg-config，违反 tools/README 不依赖系统包约定且 CI 缺包）；
//!   其他平台/缺库/缺产物 ⇒ [`NullSink`] 静默降级，打印原因与再生命令，绝不 panic。

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

pub const SAMPLE_RATE: u32 = 44_100;
pub const CHANNELS: usize = 2;
/// 再生命令（缺产物时打印）。
pub const REGEN_CMD: &str = "cargo run --release --manifest-path tools/audio/Cargo.toml";

/// game-core BGM 状态快照（每 tick 轮询）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BgmSnapshot {
    /// `bgm_request()`：轨道表下标，None = 停止/无声。
    pub track: Option<usize>,
    /// `bgm_serial()`：第几个播放器（同轨变化 = 重建 = 从头播放）。
    pub serial: u32,
    /// `bgm_volume()`：VolumeControl 级别 0..=100（创建时下发）。
    pub volume: i32,
}

/// 发往播放后端的命令。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cmd {
    /// 从头播放 `track`，幅度乘数 `gain`。
    Play { track: usize, gain: f32 },
    Stop,
}

/// 命令接收端（音频线程句柄 / 静音 / 测试录制）。
pub trait AudioSink {
    fn send(&mut self, cmd: Cmd);
}

/// 静音后端（降级路径）。
pub struct NullSink;

impl AudioSink for NullSink {
    fn send(&mut self, _cmd: Cmd) {}
}

/// VolumeControl 级别 → 幅度乘数（线性，钳 0..=100）。
pub fn level_to_gain(level: i32) -> f32 {
    level.clamp(0, 100) as f32 / 100.0
}

/// 主线程侧：game-core 请求 → 播放命令（只在变化时发送）。
pub struct BgmController {
    sink: Box<dyn AudioSink>,
    last: BgmSnapshot,
}

impl BgmController {
    pub fn new(sink: Box<dyn AudioSink>) -> BgmController {
        BgmController { sink, last: BgmSnapshot::default() }
    }

    /// 每逻辑 tick 调用一次。
    pub fn update(&mut self, snap: BgmSnapshot) {
        if snap == self.last {
            return;
        }
        match snap.track {
            None => {
                if self.last.track.is_some() {
                    self.sink.send(Cmd::Stop);
                }
            }
            Some(track) => {
                // 轨道变 或 同轨 serial 变（Java close+createPlayer）⇒ 从头播放；
                // 同一播放器（serial 不变）则音量亦不变（setLevel 只在创建时，a.java:10189-10194）
                if self.last.track != Some(track) || self.last.serial != snap.serial {
                    self.sink.send(Cmd::Play { track, gain: level_to_gain(snap.volume) });
                }
            }
        }
        self.last = snap;
    }
}

// ---------------------------------------------------------------- 轨道与混音

/// 解码后的一条轨道：交错立体声 i16 + 循环区间（帧）。
#[derive(Debug)]
pub struct Track {
    pub pcm: Vec<i16>,
    pub loop_start: usize,
    /// 循环终点（不含）= LOOPSTART + LOOPLENGTH；到达即跳回 `loop_start`。
    pub loop_end: usize,
}

impl Track {
    /// 帧数（每声道样本数）。仅测试断言用。
    #[cfg(test)]
    pub fn frames(&self) -> usize {
        self.pcm.len() / CHANNELS
    }
}

/// OGG Vorbis → [`Track`]（lewton；校验 44.1 kHz 立体声 + 循环标签在界内）。
/// 无循环标签 ⇒ 整轨循环（loop_start=0）。
pub fn decode_ogg(bytes: &[u8]) -> Result<Track, String> {
    let mut rd = lewton::inside_ogg::OggStreamReader::new(std::io::Cursor::new(bytes))
        .map_err(|e| format!("OGG 头解析失败：{e:?}"))?;
    if rd.ident_hdr.audio_channels as usize != CHANNELS || rd.ident_hdr.audio_sample_rate != SAMPLE_RATE {
        return Err(format!(
            "需 {SAMPLE_RATE} Hz 立体声，实为 {} Hz {} 声道",
            rd.ident_hdr.audio_sample_rate, rd.ident_hdr.audio_channels
        ));
    }
    let tag = |k: &str| -> Option<usize> {
        rd.comment_hdr
            .comment_list
            .iter()
            .find(|(kk, _)| kk.eq_ignore_ascii_case(k))
            .and_then(|(_, v)| v.trim().parse().ok())
    };
    let (ls, ll) = (tag("LOOPSTART"), tag("LOOPLENGTH"));
    let mut pcm: Vec<i16> = Vec::new();
    while let Some(p) = rd.read_dec_packet_itl().map_err(|e| format!("解码失败：{e:?}"))? {
        pcm.extend_from_slice(&p);
    }
    let frames = pcm.len() / CHANNELS;
    if frames == 0 {
        return Err("空音轨".into());
    }
    let (loop_start, loop_end) = match (ls, ll) {
        (Some(s), Some(l)) if l > 0 && s + l <= frames => (s, s + l),
        (None, None) => (0, frames),
        _ => return Err(format!("循环标签越界：LOOPSTART={ls:?} LOOPLENGTH={ll:?}，帧数 {frames}")),
    };
    Ok(Track { pcm, loop_start, loop_end })
}

/// 已加载轨道表（解码线程写、音频线程读）。
pub type TrackStore = Arc<Mutex<Vec<Option<Arc<Track>>>>>;

/// 音频线程侧混音器（纯计算，可单测）。
pub struct Mixer {
    store: TrackStore,
    current: Option<usize>,
    pos: usize,
    gain: f32,
}

impl Mixer {
    pub fn new(store: TrackStore) -> Mixer {
        Mixer { store, current: None, pos: 0, gain: 1.0 }
    }

    pub fn apply(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::Play { track, gain } => {
                self.current = Some(track);
                self.pos = 0;
                self.gain = gain;
            }
            Cmd::Stop => self.current = None,
        }
    }

    /// 有声可出（有轨道请求）。轨道尚未解码完时亦为 true（输出静音占位）。
    pub fn active(&self) -> bool {
        self.current.is_some()
    }

    /// 当前播放位置（帧）。仅测试断言用（音频线程不读）。
    #[cfg(test)]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// 填满 `out`（交错立体声）。轨道未就绪 ⇒ 静音且不推进位置（就绪后从头播）。
    pub fn render(&mut self, out: &mut [i16]) {
        let track = match self.current {
            Some(t) => self.store.lock().unwrap().get(t).cloned().flatten(),
            _ => None,
        };
        let Some(track) = track else {
            out.fill(0);
            return;
        };
        let g = self.gain;
        for frame in out.chunks_exact_mut(CHANNELS) {
            if self.pos >= track.loop_end {
                self.pos = track.loop_start;
            }
            let base = self.pos * CHANNELS;
            for (c, s) in frame.iter_mut().enumerate() {
                *s = (track.pcm[base + c] as f32 * g).round().clamp(-32768.0, 32767.0) as i16;
            }
            self.pos += 1;
        }
    }
}

// ---------------------------------------------------------------- 输出设备

/// 阻塞式 PCM 设备（ALSA 实现；测试不触硬件）。
pub trait PcmDevice: Send {
    /// 写满交错 i16（阻塞到设备收下）。Err = 不可恢复。
    fn write(&mut self, frames: &[i16]) -> Result<(), String>;
    /// 停止并丢弃缓冲（空闲）。
    fn idle(&mut self);
}

#[cfg(target_os = "linux")]
mod alsa {
    //! 运行时 dlopen libasound.so.2。常量与签名取自 alsa-lib 1.2 头文件
    //! （/usr/include/alsa/pcm.h：SND_PCM_STREAM_PLAYBACK=0、SND_PCM_ACCESS_RW_INTERLEAVED=3、
    //! SND_PCM_FORMAT_S16_LE=2；snd_pcm_uframes_t=unsigned long、snd_pcm_sframes_t=long）。
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_int, c_long, c_uint, c_ulong, c_void};

    const STREAM_PLAYBACK: c_int = 0;
    const ACCESS_RW_INTERLEAVED: c_int = 3;
    const FORMAT_S16_LE: c_int = 2;
    /// 期望缓冲时延（µs）：决定切轨的最大滞后。
    const LATENCY_US: c_uint = 100_000;

    type Pcm = c_void;

    pub struct AlsaPcm {
        pcm: *mut Pcm,
        writei: unsafe extern "C" fn(*mut Pcm, *const c_void, c_ulong) -> c_long,
        recover: unsafe extern "C" fn(*mut Pcm, c_int, c_int) -> c_int,
        drop_: unsafe extern "C" fn(*mut Pcm) -> c_int,
        prepare: unsafe extern "C" fn(*mut Pcm) -> c_int,
        close: unsafe extern "C" fn(*mut Pcm) -> c_int,
        strerror: unsafe extern "C" fn(c_int) -> *const c_char,
        stopped: bool,
        _lib: libloading::Library,
    }

    // SAFETY：PCM 句柄只在音频线程使用（构造后整体 move 进线程，无共享）。
    unsafe impl Send for AlsaPcm {}

    impl AlsaPcm {
        pub fn open(device: &str, rate: u32, channels: u32) -> Result<AlsaPcm, String> {
            // SAFETY：加载系统库并按 alsa-lib 头文件签名取符号；符号缺失即报错返回。
            unsafe {
                let lib = libloading::Library::new("libasound.so.2")
                    .map_err(|e| format!("无法加载 libasound.so.2（ALSA 运行库）：{e}"))?;
                macro_rules! sym {
                    ($name:literal) => {
                        *lib.get($name).map_err(|e| format!("libasound 缺符号 {}：{e}", String::from_utf8_lossy($name)))?
                    };
                }
                let open: unsafe extern "C" fn(*mut *mut Pcm, *const c_char, c_int, c_int) -> c_int =
                    sym!(b"snd_pcm_open\0");
                let set_params: unsafe extern "C" fn(*mut Pcm, c_int, c_int, c_uint, c_uint, c_int, c_uint) -> c_int =
                    sym!(b"snd_pcm_set_params\0");
                let close: unsafe extern "C" fn(*mut Pcm) -> c_int = sym!(b"snd_pcm_close\0");
                let strerror: unsafe extern "C" fn(c_int) -> *const c_char = sym!(b"snd_strerror\0");
                let name = CString::new(device).map_err(|_| "设备名含 NUL".to_string())?;
                let mut pcm: *mut Pcm = std::ptr::null_mut();
                let rc = open(&mut pcm, name.as_ptr(), STREAM_PLAYBACK, 0);
                if rc < 0 {
                    return Err(format!("snd_pcm_open({device}) 失败：{}", err_str(strerror, rc)));
                }
                let rc = set_params(pcm, FORMAT_S16_LE, ACCESS_RW_INTERLEAVED, channels, rate, 1, LATENCY_US);
                if rc < 0 {
                    close(pcm);
                    return Err(format!("snd_pcm_set_params 失败：{}", err_str(strerror, rc)));
                }
                Ok(AlsaPcm {
                    pcm,
                    writei: sym!(b"snd_pcm_writei\0"),
                    recover: sym!(b"snd_pcm_recover\0"),
                    drop_: sym!(b"snd_pcm_drop\0"),
                    prepare: sym!(b"snd_pcm_prepare\0"),
                    close,
                    strerror,
                    stopped: false,
                    _lib: lib,
                })
            }
        }
    }

    unsafe fn err_str(f: unsafe extern "C" fn(c_int) -> *const c_char, rc: c_int) -> String {
        let p = f(rc);
        if p.is_null() {
            format!("errno {rc}")
        } else {
            CStr::from_ptr(p).to_string_lossy().into_owned()
        }
    }

    impl super::PcmDevice for AlsaPcm {
        fn write(&mut self, frames: &[i16]) -> Result<(), String> {
            // SAFETY：pcm 为 open 成功的句柄；缓冲区长度按帧数（声道数 2）换算。
            unsafe {
                if self.stopped {
                    (self.prepare)(self.pcm);
                    self.stopped = false;
                }
                let mut off = 0usize;
                let total = frames.len() / super::CHANNELS;
                while off < total {
                    let ptr = frames.as_ptr().add(off * super::CHANNELS) as *const c_void;
                    let n = (self.writei)(self.pcm, ptr, (total - off) as c_ulong);
                    if n < 0 {
                        // xrun/挂起等：recover 后重试；不可恢复 ⇒ 交上层降级
                        let rc = (self.recover)(self.pcm, n as c_int, 1);
                        if rc < 0 {
                            return Err(format!("snd_pcm_writei 失败：{}", err_str(self.strerror, rc)));
                        }
                    } else {
                        off += n as usize;
                    }
                }
                Ok(())
            }
        }

        fn idle(&mut self) {
            if !self.stopped {
                // SAFETY：同上。
                unsafe {
                    (self.drop_)(self.pcm);
                }
                self.stopped = true;
            }
        }
    }

    impl Drop for AlsaPcm {
        fn drop(&mut self) {
            // SAFETY：句柄仅在此关闭一次。
            unsafe {
                (self.close)(self.pcm);
            }
        }
    }
}

// ---------------------------------------------------------------- 音频线程

enum Msg {
    Cmd(Cmd),
    Quit,
}

/// 音频线程句柄（[`AudioSink`] 实现）。Drop 时通知线程退出并等待。
pub struct AudioThread {
    tx: Sender<Msg>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl AudioSink for AudioThread {
    fn send(&mut self, cmd: Cmd) {
        let _ = self.tx.send(Msg::Cmd(cmd));
    }
}

impl Drop for AudioThread {
    fn drop(&mut self) {
        let _ = self.tx.send(Msg::Quit);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// 每次写入的帧数（≈23 ms）。
const CHUNK_FRAMES: usize = 1024;

fn audio_loop(rx: Receiver<Msg>, mut dev: Box<dyn PcmDevice>, mut mixer: Mixer) {
    let mut buf = vec![0i16; CHUNK_FRAMES * CHANNELS];
    loop {
        // 空闲 ⇒ 停设备并阻塞等命令；活跃 ⇒ 非阻塞取完命令后写一块
        if !mixer.active() {
            dev.idle();
            match rx.recv() {
                Ok(Msg::Cmd(c)) => mixer.apply(c),
                Ok(Msg::Quit) | Err(_) => return,
            }
        }
        loop {
            match rx.try_recv() {
                Ok(Msg::Cmd(c)) => mixer.apply(c),
                Ok(Msg::Quit) => return,
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => return,
            }
        }
        if mixer.active() {
            mixer.render(&mut buf);
            if let Err(e) = dev.write(&buf) {
                eprintln!("[audio] 输出中断（{e}）——本次运行转为无声");
                // 继续消费命令直到退出，避免发送端阻塞/报错
                while let Ok(m) = rx.recv() {
                    if matches!(m, Msg::Quit) {
                        return;
                    }
                }
                return;
            }
        }
    }
}

/// 音频产物路径：`<dir>/track{i}.ogg`。
pub fn track_path(dir: &Path, i: usize) -> PathBuf {
    dir.join(format!("track{i}.ogg"))
}

/// 启动 BGM 输出；任何环节失败 ⇒ 打印原因并返回 [`NullSink`]（不 panic）。
///
/// 预取顺序：标题曲、游戏曲（可达轨道，docs/spec/audio.md §1）；其余轨道（BUG-010
/// 死资源）不解码——请求到未加载轨道时静音。
pub fn start(audio_dir: &Path, device: &str) -> Box<dyn AudioSink> {
    use game_core::audio::{TRACKS, TRACK_GAME, TRACK_MENU};
    let missing: Vec<String> = (0..TRACKS.len())
        .filter(|&i| !track_path(audio_dir, i).exists())
        .map(|i| track_path(audio_dir, i).display().to_string())
        .collect();
    if !missing.is_empty() {
        eprintln!(
            "[audio] BGM 产物缺失：{}\n[audio] 再生：{REGEN_CMD}（需先有 assets/raw，python3 tools/extract-assets/extract.py）\n[audio] 本次无声运行",
            missing.join(", ")
        );
        return Box::new(NullSink);
    }
    #[cfg(target_os = "linux")]
    let dev: Box<dyn PcmDevice> = match alsa::AlsaPcm::open(device, SAMPLE_RATE, CHANNELS as u32) {
        Ok(d) => Box::new(d),
        Err(e) => {
            eprintln!("[audio] {e}——本次无声运行");
            return Box::new(NullSink);
        }
    };
    #[cfg(not(target_os = "linux"))]
    {
        let _ = device;
        eprintln!("[audio] 本平台暂无音频后端（已实现：Linux ALSA）——本次无声运行");
        return Box::new(NullSink);
    }
    #[cfg(target_os = "linux")]
    {
        let store: TrackStore = Arc::new(Mutex::new(vec![None; TRACKS.len()]));
        let loader_store = store.clone();
        let dir = audio_dir.to_path_buf();
        std::thread::Builder::new()
            .name("bgm-decode".into())
            .spawn(move || {
                for i in [TRACK_MENU as usize, TRACK_GAME as usize] {
                    let p = track_path(&dir, i);
                    match std::fs::read(&p).map_err(|e| e.to_string()).and_then(|b| decode_ogg(&b)) {
                        Ok(t) => loader_store.lock().unwrap()[i] = Some(Arc::new(t)),
                        Err(e) => eprintln!("[audio] {} 不可用（{e}）——该轨静音；再生：{REGEN_CMD}", p.display()),
                    }
                }
            })
            .ok();
        let (tx, rx) = std::sync::mpsc::channel();
        let mixer = Mixer::new(store);
        let handle = std::thread::Builder::new()
            .name("bgm-out".into())
            .spawn(move || audio_loop(rx, dev, mixer))
            .ok();
        eprintln!("[audio] BGM 输出：ALSA 设备 {device}（{}）", audio_dir.display());
        Box::new(AudioThread { tx, handle })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 录制命令的测试后端。
    pub struct Recorder(pub Arc<Mutex<Vec<Cmd>>>);

    impl AudioSink for Recorder {
        fn send(&mut self, cmd: Cmd) {
            self.0.lock().unwrap().push(cmd);
        }
    }

    fn snap(track: Option<usize>, serial: u32, volume: i32) -> BgmSnapshot {
        BgmSnapshot { track, serial, volume }
    }

    /// 请求变化 → 切轨：启动静音 → menu → game（重建）→ 关声 → 开声（同轨新 serial ⇒ 从头）
    /// → 同一状态重复轮询不重发。
    #[test]
    fn controller_translates_request_changes() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut c = BgmController::new(Box::new(Recorder(log.clone())));
        c.update(snap(None, 0, 0));
        c.update(snap(Some(2), 1, 60));
        c.update(snap(Some(2), 1, 60)); // 无变化
        c.update(snap(Some(3), 2, 60));
        c.update(snap(None, 0, 0));
        c.update(snap(None, 0, 0));
        c.update(snap(Some(3), 3, 60));
        let got = log.lock().unwrap().clone();
        assert_eq!(
            got,
            vec![
                Cmd::Play { track: 2, gain: 0.6 },
                Cmd::Play { track: 3, gain: 0.6 },
                Cmd::Stop,
                Cmd::Play { track: 3, gain: 0.6 },
            ]
        );
    }

    fn ramp_track(frames: usize, loop_start: usize, loop_end: usize) -> Track {
        let mut pcm = Vec::with_capacity(frames * 2);
        for i in 0..frames {
            pcm.push(i as i16);
            pcm.push(-(i as i16));
        }
        Track { pcm, loop_start, loop_end }
    }

    fn store_with(t: Track, idx: usize) -> TrackStore {
        let mut v: Vec<Option<Arc<Track>>> = vec![None; 4];
        v[idx] = Some(Arc::new(t));
        Arc::new(Mutex::new(v))
    }

    /// 循环点：前奏放一次，之后在 [loop_start, loop_end) 无缝重复；Play 从头；Stop 静音。
    #[test]
    fn mixer_loops_restarts_and_silences() {
        let mut m = Mixer::new(store_with(ramp_track(100, 30, 80), 3));
        m.apply(Cmd::Play { track: 3, gain: 1.0 });
        let mut out = vec![0i16; 2 * 130];
        m.render(&mut out);
        let left: Vec<i16> = out.chunks(2).map(|f| f[0]).collect();
        let expect: Vec<i16> = (0..80).chain(30..80).map(|v| v as i16).collect();
        assert_eq!(left, expect, "前奏 0..80 后从 30 循环");
        assert_eq!(out[1], 0);
        assert_eq!(out[3], -1, "右声道独立");
        assert_eq!(m.position(), 80);

        // 循环回绕：loop_end 后回 loop_start（80 = loop_end ⇒ 回 30）
        let mut one = vec![0i16; 2];
        m.render(&mut one);
        assert_eq!(one[0], 30, "loop_end 后回 loop_start");

        // 重新 Play ⇒ 从头；Stop ⇒ 不活跃
        m.apply(Cmd::Play { track: 3, gain: 1.0 });
        m.render(&mut one);
        assert_eq!(one[0], 0);
        m.apply(Cmd::Stop);
        assert!(!m.active());
    }

    /// 音量：级别 60 ⇒ 乘数 0.6；未解码轨道 ⇒ 静音且不推进位置。
    #[test]
    fn mixer_gain_and_unloaded_track() {
        let mut m = Mixer::new(store_with(ramp_track(10, 0, 10), 2));
        m.apply(Cmd::Play { track: 2, gain: level_to_gain(60) });
        let mut out = vec![0i16; 2 * 6];
        m.render(&mut out);
        assert_eq!(out[2 * 5], 3, "5 × 0.6 = 3");
        m.apply(Cmd::Play { track: 1, gain: 1.0 }); // 未加载
        m.render(&mut out);
        assert!(out.iter().all(|&v| v == 0));
        assert_eq!(m.position(), 0);
        assert_eq!(level_to_gain(150), 1.0);
        assert_eq!(level_to_gain(-5), 0.0);
    }

    fn fixture() -> Vec<u8> {
        // tools/audio `--fixture` 产出（合成正弦，确定性；sha256 a1745344…）
        std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/loop-fixture.ogg")).unwrap()
    }

    /// OGG 解码 + 循环标签：夹具 LOOPSTART=11025 / LOOPLENGTH=22050，前奏 440 Hz、循环体
    /// 660 Hz、右声道反相——验证解码帧数、标签、零时延（过零周期）与声道。
    #[test]
    fn decode_fixture_with_loop_tags() {
        let t = decode_ogg(&fixture()).unwrap();
        assert_eq!((t.loop_start, t.loop_end), (11_025, 33_075));
        assert!(t.frames() >= 33_075, "帧数 {}", t.frames());
        // 过零计数估频：前奏段 ≈440 Hz、循环体 ≈660 Hz（各取 0.1 s）
        let zc = |from: usize| {
            (from..from + 4410)
                .filter(|&i| (t.pcm[2 * i] >= 0) != (t.pcm[2 * i + 2] >= 0))
                .count()
        };
        let (a, b) = (zc(2000), zc(20_000));
        assert!((80..=96).contains(&a), "前奏过零 {a}（期望 ≈88）");
        assert!((122..=142).contains(&b), "循环体过零 {b}（期望 ≈132）");
        let corr: i64 = (2000..6000).map(|i| t.pcm[2 * i] as i64 * t.pcm[2 * i + 1] as i64).sum();
        assert!(corr < 0, "右声道应反相");
    }

    /// 坏数据不 panic：非 OGG / 截断流 ⇒ Err。
    #[test]
    fn decode_rejects_garbage() {
        assert!(decode_ogg(b"not an ogg").is_err());
        let f = fixture();
        assert!(decode_ogg(&f[..200]).is_err());
    }

    /// 缺产物 ⇒ 降级为 NullSink（不 panic、不起线程）。
    #[test]
    fn start_without_assets_degrades_silently() {
        let mut s = start(Path::new("/nonexistent/xiyou-audio"), "null");
        s.send(Cmd::Play { track: 2, gain: 1.0 }); // no-op
    }
}
