//! BGM 输出（桌面壳侧）：共享面（BgmController/Track/decode_ogg/Mixer/audio_loop 等）
//! 已抽 `game-shell-common::audio`（lead F-4 裁决 2026-10-17，防双壳拷贝漂移），
//! 本模块保留**平台面**：winmm waveOut / ALSA dlopen 设备实现、`--audio` 路径型轨迹
//! 装载（已知第五缝，Android 轨道走 AssetSource::read("track{i}.ogg")）与音频线程句柄。
//!
//! 语义见 game-shell-common::audio 与 docs/spec/audio.md。输出：Linux 运行时 dlopen
//! `libasound.so.2`（ALSA PCM，无构建期系统依赖——cpal/rodio 经 alsa-sys 需
//! libasound2-dev 头文件 + pkg-config，违反 tools/README 不依赖系统包约定且 CI 缺包）；
//! Windows = `winmm.dll` waveOut 裸 FFI（mmsystem.h 直绑、同款零构建依赖，[`winmm`]）；
//! 其他平台/缺库/缺产物 ⇒ [`NullSink`] 静默降级，打印原因与再生命令，绝不 panic。

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use game_platform::AudioOut;

pub use game_shell_common::audio::*;

/// 再生命令（缺产物时打印）。路径型装载面（第五缝）= Windows 壳专用。
pub const REGEN_CMD: &str = "cargo run --release --manifest-path tools/audio/Cargo.toml";

// ---------------------------------------------------------------- Windows 输出（winmm waveOut）

#[cfg(target_os = "windows")]
mod winmm {
    //! winmm.dll `waveOut`（WAVE_MAPPER 默认输出设备，44.1 kHz 立体声 16-bit PCM 原样直出，
    //! 重采样由驱动侧完成）。签名与常量取自 mmsystem.h / Windows SDK（`WAVEFORMATEX`、
    //! `WAVEHDR`、`waveOutOpen/Close/PrepareHeader/UnprepareHeader/Write/Reset`、
    //! WAVE_MAPPER=0xFFFFFFFF、WAVE_FORMAT_PCM=1、WHDR_DONE=1、MMSYSERR_NOERROR=0）。
    //!
    //! 与 [`super::alsa`] 同款**头文件直绑**姿态：零新增 crate（windows-rs/cpal 均可作
    //! 升级缝，WASAPI/IAudioClient 留待实机反馈后再议）——target 侧仅 `#[link(winmm)]`
    //! 导入库（Windows SDK / mingw 自带），Linux 构建与运行零变化。
    //!
    //! **Windows 实机验收待用户**（验证不打扰铁律：本环境仅 `cargo check
    //! --target x86_64-pc-windows-msvc` 交叉编译验证，不代跑实机）。
    use game_platform::AudioOut;
    use std::os::raw::{c_char, c_void};
    use std::time::{Duration, Instant};

    const WAVE_MAPPER: u32 = 0xFFFF_FFFF; // mmsystem.h
    const WAVE_FORMAT_PCM: u16 = 0x0001;
    const WHDR_DONE: u32 = 0x0000_0001;
    const CALLBACK_NULL: usize = 0; // dwCallback 无回调 ⇒ 用 WHDR_DONE 轮询判完成
    const MMSYSERR_NOERROR: u32 = 0;
    /// 单块写入等待上限（远超 CHUNK_FRAMES≈23 ms 正常耗时）；超限=设备失联 ⇒
    /// 不可恢复 Err（audio_loop 按既有语义转无声运行）。
    const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

    /// WAVEFORMATEX（mmsystem.h；repr(C) 逐字段同布局）。
    #[repr(C)]
    struct WaveFormatEx {
        w_format_tag: u16,
        n_channels: u16,
        n_samples_per_sec: u32,
        n_avg_bytes_per_sec: u32,
        n_block_align: u16,
        w_bits_per_sample: u16,
        cb_size: u16,
    }

    /// WAVEHDR（mmsystem.h；x64 下 48 字节，DWORD_PTR=usize）。
    #[repr(C)]
    struct WaveHdr {
        lp_data: *mut c_char,
        dw_buffer_length: u32,
        dw_bytes_recorded: u32,
        dw_user: usize,
        dw_flags: u32,
        dw_loops: u32,
        lp_next: *mut WaveHdr,
        reserved: usize,
    }

    type Handle = *mut c_void; // HWAVEOUT = HANDLE

    #[link(name = "winmm")]
    extern "system" {
        fn waveOutOpen(lphwo: *mut Handle, u_device_id: u32, pwfx: *const WaveFormatEx, dw_callback: usize, dw_instance: usize, fdw_open: u32) -> u32;
        fn waveOutClose(hwo: Handle) -> u32;
        fn waveOutPrepareHeader(hwo: Handle, pwh: *mut WaveHdr, cbwh: u32) -> u32;
        fn waveOutUnprepareHeader(hwo: Handle, pwh: *mut WaveHdr, cbwh: u32) -> u32;
        fn waveOutWrite(hwo: Handle, pwh: *mut WaveHdr, cbwh: u32) -> u32;
        fn waveOutReset(hwo: Handle) -> u32;
    }

    /// 在途缓冲深度（2026-10-17 BUG-019）：4 × chunk(1024 帧 ≈ 23ms) ≈ 92ms 排队。
    /// 单缓冲"写一块等一块"的两病根由此消除：①块间设备空窗=欠载杂音；②Windows
    /// 定时器粒度下 2ms 轮询严重超时 ⇒ 每块都拖拍。队列恒保 3 块在途吸收轮询抖动。
    const QUEUE_DEPTH: usize = 4;

    struct Slot {
        data: Vec<i16>,
        hdr: WaveHdr,
        submitted: bool,
    }

    pub struct WinMmPcm {
        hwo: Handle,
        slots: Vec<Slot>,
        next: usize,
    }

    // SAFETY：设备句柄只在音频线程使用（构造后整体 move 进线程，无共享）——与 AlsaPcm 同。
    unsafe impl Send for WinMmPcm {}

    impl WinMmPcm {
        pub fn open(device: &str, rate: u32, channels: u32) -> Result<WinMmPcm, String> {
            let _ = device; // WAVE_MAPPER = 系统默认输出；设备选择暂不在本层（与 ALSA "default" 同约定）
            let block_align = (channels * 2) as u16; // 16-bit/样本
            let fmt = WaveFormatEx {
                w_format_tag: WAVE_FORMAT_PCM,
                n_channels: channels as u16,
                n_samples_per_sec: rate,
                n_avg_bytes_per_sec: rate * channels * 2,
                n_block_align: block_align,
                w_bits_per_sample: 16,
                cb_size: 0,
            };
            // SAFETY：fmt 为 repr(C) WAVEFORMATEX；句柄出参有效；dwCallback=NULL 无回调。
            unsafe {
                let mut hwo: Handle = std::ptr::null_mut();
                let r = waveOutOpen(&mut hwo, WAVE_MAPPER, &fmt, CALLBACK_NULL, 0, 0);
                if r != MMSYSERR_NOERROR || hwo.is_null() {
                    return Err(format!("waveOutOpen 失败（winmm MMRESULT {r}）"));
                }
                let slots = (0..QUEUE_DEPTH)
                    .map(|_| Slot {
                        data: Vec::new(),
                        // SAFETY：WaveHdr 为 repr(C) POD，全零 = 未提交态。
                        hdr: unsafe { std::mem::zeroed() },
                        submitted: false,
                    })
                    .collect();
                Ok(WinMmPcm { hwo, slots, next: 0 })
            }
        }
    }

    impl AudioOut for WinMmPcm {
        fn write(&mut self, frames: &[i16]) -> Result<(), String> {
            if frames.is_empty() {
                return Ok(());
            }
            let cbwh = std::mem::size_of::<WaveHdr>() as u32;
            // 取空闲槽（未提交 / 已 Done）——**不等刚提交的本块**：设备恒有
            // QUEUE_DEPTH-1 块在途（单缓冲同步写的块间空窗由此消除，BUG-019）。
            let t0 = Instant::now();
            let idx = loop {
                let mut found = None;
                for k in 0..self.slots.len() {
                    let i = (self.next + k) % self.slots.len();
                    let s = &self.slots[i];
                    if !s.submitted || (s.hdr.dw_flags & WHDR_DONE) != 0 {
                        found = Some(i);
                        break;
                    }
                }
                if let Some(i) = found {
                    break i;
                }
                if t0.elapsed() > WRITE_TIMEOUT {
                    return Err("waveOut 队列超时（设备失联）".to_string());
                }
                std::thread::sleep(Duration::from_millis(2));
            };
            self.next = (idx + 1) % self.slots.len();
            let slot = &mut self.slots[idx];
            // SAFETY：slot 未在途（未提交或已 Done ⇒ 设备不再读其头/数据）——
            // 回收旧头、改写 data（仅此处可 realloc，安全）后重新 Prepare+Write。
            unsafe {
                if slot.submitted {
                    let _ = waveOutUnprepareHeader(self.hwo, &mut slot.hdr, cbwh);
                    slot.submitted = false;
                }
                slot.data.clear();
                slot.data.extend_from_slice(frames);
                slot.hdr = std::mem::zeroed();
                slot.hdr.lp_data = slot.data.as_ptr() as *mut c_char;
                slot.hdr.dw_buffer_length = (frames.len() * 2) as u32;
                let r = waveOutPrepareHeader(self.hwo, &mut slot.hdr, cbwh);
                if r != MMSYSERR_NOERROR {
                    return Err(format!("waveOutPrepareHeader 失败（MMRESULT {r}）"));
                }
                let r = waveOutWrite(self.hwo, &mut slot.hdr, cbwh);
                if r != MMSYSERR_NOERROR {
                    let _ = waveOutUnprepareHeader(self.hwo, &mut slot.hdr, cbwh);
                    return Err(format!("waveOutWrite 失败（MMRESULT {r}）"));
                }
                slot.submitted = true;
            }
            Ok(())
        }

        fn idle(&mut self) {
            // SAFETY：句柄有效；Reset 丢弃排队缓冲并把各头标 Done（回收在 write 环节）。
            unsafe {
                let _ = waveOutReset(self.hwo);
            }
        }
    }

    impl Drop for WinMmPcm {
        fn drop(&mut self) {
            // SAFETY：句柄仅在此关闭一次（Reset 唤醒在途头后逐个 Unprepare 再关）。
            unsafe {
                let _ = waveOutReset(self.hwo);
                let cbwh = std::mem::size_of::<WaveHdr>() as u32;
                for s in &mut self.slots {
                    if s.submitted {
                        let _ = waveOutUnprepareHeader(self.hwo, &mut s.hdr, cbwh);
                        s.submitted = false;
                    }
                }
                let _ = waveOutClose(self.hwo);
            }
        }
    }
}

// ---------------------------------------------------------------- Linux 输出（ALSA dlopen）

#[cfg(target_os = "linux")]
mod alsa {
    //! 运行时 dlopen libasound.so.2。常量与签名取自 alsa-lib 1.2 头文件
    //! （/usr/include/alsa/pcm.h：SND_PCM_STREAM_PLAYBACK=0、SND_PCM_ACCESS_RW_INTERLEAVED=3、
    //! SND_PCM_FORMAT_S16_LE=2；snd_pcm_uframes_t=unsigned long、snd_pcm_sframes_t=long）。
    use game_platform::AudioOut;
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
                let strerror: unsafe extern "C" fn(c_int) -> *const c_char = sym!(b"snd_pcm_strerror\0");
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

    impl AudioOut for AlsaPcm {
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

// ---------------------------------------------------------------- 音频线程句柄与轨迹装载

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

/// 音频产物路径：`<dir>/track{i}.ogg`。路径型装载面（第五缝）= Windows 壳专用；
/// Android 轨道走 `AssetSource::read("track{i}.ogg")`。
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
    let opened: Result<Box<dyn AudioOut>, String> =
        alsa::AlsaPcm::open(device, SAMPLE_RATE, CHANNELS as u32).map(|d| Box::new(d) as Box<dyn AudioOut>);
    #[cfg(target_os = "windows")]
    let opened: Result<Box<dyn AudioOut>, String> =
        winmm::WinMmPcm::open(device, SAMPLE_RATE, CHANNELS as u32).map(|d| Box::new(d) as Box<dyn AudioOut>);
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    let opened: Result<Box<dyn AudioOut>, String> = {
        let _ = device;
        Err("本平台暂无音频后端（已实现：Linux ALSA / Windows winmm waveOut）".to_string())
    };
    let dev: Box<dyn AudioOut> = match opened {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[audio] {e}——本次无声运行");
            return Box::new(NullSink);
        }
    };
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
        eprintln!("[audio] BGM 输出：{device}（{}）", audio_dir.display());
        Box::new(AudioThread { tx, handle })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// 缺产物 ⇒ 降级为 NullSink（不 panic、不起线程）。
    #[test]
    fn start_without_assets_degrades_silently() {
        let mut s = start(Path::new("/nonexistent/xiyou-audio"), "null");
        s.send(Cmd::Play { track: 2, gain: 1.0 }); // no-op
    }
}
