//! BGM/PCM 共享面（game-shell-common）：语义在 game-core（`game_core::audio`，
//! docs/spec/audio.md）；本模块只把「应当在放的轨道」变成 PCM 字节流——**纯字节、
//! 零 `std::fs`/路径**（adversary 抽取判据 1：Android 由 AssetSource 供字节）。
//!
//! 自 shell/win `audio.rs` 迁入（lead F-4 裁决 2026-10-17，防双壳拷贝漂移）：
//!
//! - [`BgmController`]（主线程，每逻辑 tick 轮询 `bgm_request/bgm_serial/bgm_volume`）
//!   把状态变化翻译为 [`Cmd`]：轨道或 serial 变 ⇒ `Play`（从头播放，= Java 关闭重建
//!   a.java:10178-10199）；变 None ⇒ `Stop`（closeAudioPlayer——含 hideNotify 的关闭，
//!   lifecycle.rs；失焦不产生任何命令）。
//! - [`Mixer`]（音频线程）：按 OGG 的 `LOOPSTART/LOOPLENGTH` 无限循环（setLoopCount(-1)
//!   a.java:10184），幅度 = VolumeControl 级别/100（JSR-135：级别线性刻度、乘数映射由
//!   实现决定）。
//! - [`audio_loop`]（音频线程体）：驱动 `game_platform::AudioOut` 设备——阻塞 write =
//!   节拍，空闲 ⇒ `idle()`。设备实现（winmm waveOut / ALSA dlopen / AAudio）在各壳。
//!
//! **已知第五缝不入本模块**（docs/spec/platform-contract.md §0/§4.2）：BGM 轨迹装载
//! `start()`/`track_path`/`REGEN_CMD`（`--audio`/`track{i}.ogg` 路径注入）= Windows 壳
//! 专用；Android 轨道走 `AssetSource::read("track{i}.ogg")`。轨道命名映射：
//! game-core 轨道表 `menu.mid/game.mid` → 壳侧产物 `track{i}.ogg`（tools/audio 重编码）。

use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use game_platform::AudioOut;

/// PCM 帧格式（契约常量单一来源，game-platform §3；原 shell/win audio.rs:20-21 同值）。
pub use game_platform::{CHANNELS, SAMPLE_RATE};

/// 每次写入的帧数（≈23 ms）。音频线程体实现细节（不在契约面）。
pub const CHUNK_FRAMES: usize = 1024;

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

impl BgmSnapshot {
    /// 开场链 BootMachine 的快照（win/android 双壳同源取数，防拷贝漂移）。
    pub fn of_boot(m: &game_core::paint::BootMachine) -> BgmSnapshot {
        BgmSnapshot { track: m.bgm_request(), serial: m.bgm_serial(), volume: m.bgm_volume().unwrap_or(0) }
    }

    /// 游戏场景 GameScene 的快照。
    pub fn of_scene(sc: &game_core::scene::GameScene) -> BgmSnapshot {
        BgmSnapshot { track: sc.bgm_request(), serial: sc.bgm_serial(), volume: sc.bgm_volume().unwrap_or(0) }
    }
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
/// 无循环标签 ⇒ 整轨循环（loop_start=0）。纯字节入参（第五缝装载在各壳）。
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

// ---------------------------------------------------------------- 音频线程体

/// 发往音频线程的消息（壳侧句柄构造 `Msg::Cmd`；`Quit` = 线程收尾）。
pub enum Msg {
    Cmd(Cmd),
    Quit,
}

/// 音频线程体：空闲 ⇒ 停设备并阻塞等命令；活跃 ⇒ 非阻塞取完命令后写一块
/// （`AudioOut::write` 阻塞 = 节拍）。write 不可恢复 Err ⇒ 转无声（继续消费
/// 命令直到退出，避免发送端阻塞/报错）。
pub fn audio_loop(rx: Receiver<Msg>, mut dev: Box<dyn AudioOut>, mut mixer: Mixer) {
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

#[cfg(test)]
mod tests {
    //! 随迁测试（自 shell/win/src/audio.rs 整体迁入，断言零字节改动；2026-10-18）。
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

    /// PcmDevice 契约面（audio_loop 驱动行为，接缝②行为锚，2026-10-17 迁移前基线）：
    /// 空闲起手先 `idle()`；活跃期每轮 `write` 一块（CHUNK_FRAMES×CHANNELS 样本）；
    /// Stop ⇒ 回到 `idle()` 后阻塞等命令。迁移后此面由 game_platform::AudioOut 承载
    /// （trait 本体迁出、本地只留实现），**事件序不得变**（断言零改动纪律）。
    #[test]
    fn audio_loop_device_contract_idle_write_sequence() {
        use super::Msg;
        use game_platform::AudioOut;
        use std::sync::mpsc::channel;

        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        enum Ev {
            Idle,
            Write(usize),
        }

        struct MockDev {
            log: Arc<Mutex<Vec<Ev>>>,
            fail: bool,
        }

        impl AudioOut for MockDev {
            fn write(&mut self, frames: &[i16]) -> Result<(), String> {
                self.log.lock().unwrap().push(Ev::Write(frames.len()));
                if self.fail {
                    Err("注入故障".into())
                } else {
                    Ok(())
                }
            }
            fn idle(&mut self) {
                self.log.lock().unwrap().push(Ev::Idle);
            }
        }

        let log: Arc<Mutex<Vec<Ev>>> = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = channel();
        let dev = Box::new(MockDev { log: log.clone(), fail: false });
        let mixer = Mixer::new(store_with(ramp_track(100, 0, 100), 2));
        let handle = std::thread::spawn(move || audio_loop(rx, dev, mixer));

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        tx.send(Msg::Cmd(Cmd::Play { track: 2, gain: 1.0 })).unwrap();
        while !log.lock().unwrap().iter().any(|e| matches!(e, Ev::Write(_))) {
            assert!(std::time::Instant::now() < deadline, "5s 内未见 write");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        tx.send(Msg::Cmd(Cmd::Stop)).unwrap();
        while *log.lock().unwrap().last().unwrap() != Ev::Idle {
            assert!(std::time::Instant::now() < deadline, "5s 内未见 Stop 后 idle");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        tx.send(Msg::Quit).unwrap();
        handle.join().unwrap();

        let events = log.lock().unwrap().clone();
        assert_eq!(events.first(), Some(&Ev::Idle), "空闲起手先 idle()（audio.rs 空闲分支）");
        assert_eq!(events.last(), Some(&Ev::Idle), "Stop 后回到 idle() 阻塞等命令");
        let writes = events[1..events.len() - 1].to_vec();
        assert!(!writes.is_empty(), "活跃期应有 write");
        assert!(
            writes.iter().all(|e| *e == Ev::Write(CHUNK_FRAMES * CHANNELS)),
            "write 块长恒为 CHUNK_FRAMES×CHANNELS = {}，实得 {writes:?}",
            CHUNK_FRAMES * CHANNELS
        );
    }

    /// write 不可恢复 Err ⇒ audio_loop 转无声（不再 write），收 Quit 后线程正常退出
    /// （不 panic）。与 `start` 的降级口径一致（audio.rs:603-611）。
    #[test]
    fn audio_loop_write_error_degrades_silently() {
        use super::Msg;
        use game_platform::AudioOut;
        use std::sync::mpsc::channel;

        struct FailingDev {
            writes: Arc<Mutex<usize>>,
        }

        impl AudioOut for FailingDev {
            fn write(&mut self, _frames: &[i16]) -> Result<(), String> {
                *self.writes.lock().unwrap() += 1;
                Err("注入故障".into())
            }
            fn idle(&mut self) {}
        }

        let writes = Arc::new(Mutex::new(0usize));
        let (tx, rx) = channel();
        let dev = Box::new(FailingDev { writes: writes.clone() });
        let mixer = Mixer::new(store_with(ramp_track(100, 0, 100), 2));
        let handle = std::thread::spawn(move || audio_loop(rx, dev, mixer));

        tx.send(Msg::Cmd(Cmd::Play { track: 2, gain: 1.0 })).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while *writes.lock().unwrap() == 0 {
            assert!(std::time::Instant::now() < deadline, "5s 内未见 write");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        tx.send(Msg::Quit).unwrap();
        handle.join().unwrap();
        assert_eq!(*writes.lock().unwrap(), 1, "故障后转无声：不再 write");
    }
}
