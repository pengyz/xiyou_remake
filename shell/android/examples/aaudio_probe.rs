//! 实机 AAudio 契约探针：直接驱动壳的 `audio::aaudio::AAudioPcm`（与游戏同一实现），
//! 核对 `game_platform::AudioOut` 契约在真机上是否成立——**adb shell 运行，不起 Activity、
//! 不抢前台；只写静音样本（零可闻输出）**。
//!
//! ```text
//! cargo ndk -t arm64-v8a -P 26 build --offline --release -p game-android --example aaudio_probe
//! adb push target/aarch64-linux-android/release/examples/aaudio_probe /data/local/tmp/
//! adb shell /data/local/tmp/aaudio_probe            # 输出 probe: 行；libaaudio 日志 = logcat --pid
//! ```
//!
//! 判据（每段独立看门狗，超时 = HANG 而非卡死探针）：
//! - open 成功；
//! - 写 N 秒静音的墙钟 ≈ N 秒（阻塞 write = 节拍；远快于 N ⇒ 不阻塞/未消费）；
//! - 进程 CPU 时间 ≪ 墙钟（≈ 墙钟 ⇒ 忙等自旋）；
//! - `idle()` 后再写仍按节拍推进（停流后可重启）。

#[cfg(target_os = "android")]
fn main() {
    use game_android::audio::aaudio::AAudioPcm;
    use game_platform::{AudioOut, CHANNELS, SAMPLE_RATE};
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    const CHUNK: usize = 1024; // = game_shell_common::audio::CHUNK_FRAMES

    /// 进程 CPU 时间（/proc/self/stat utime+stime，单位 clock tick；Android CLK_TCK=100）。
    fn cpu_ticks() -> u64 {
        let s = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
        let rest = s.rsplit_once(')').map(|(_, r)| r).unwrap_or("");
        let f: Vec<&str> = rest.split_whitespace().collect();
        // rest[0]=state(字段3) ⇒ utime=字段14=rest[11]、stime=字段15=rest[12]
        let n = |i: usize| f.get(i).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        n(11) + n(12)
    }

    println!("probe: pid={} rate={SAMPLE_RATE} ch={CHANNELS}", std::process::id());
    let mut pcm = match AAudioPcm::open() {
        Ok(p) => {
            println!("probe: open=ok {}", p.describe());
            p
        }
        Err(e) => {
            println!("probe: open=err {e}");
            return;
        }
    };

    // 每段在工作线程写，主线程看门狗：未在 limit 内完成 ⇒ HANG 并退出进程。
    let silence = vec![0i16; CHUNK * CHANNELS];
    for (label, secs, idle_before) in [("seg1", 2.0f64, false), ("seg2-after-idle", 1.0, true)] {
        if idle_before {
            pcm.idle();
        }
        let chunks = ((secs * SAMPLE_RATE as f64) / CHUNK as f64).ceil() as usize;
        let (tx, rx) = mpsc::channel();
        let buf = silence.clone();
        let (t0, c0) = (Instant::now(), cpu_ticks());
        let worker = std::thread::spawn(move || {
            let mut res = Ok(());
            for _ in 0..chunks {
                res = pcm.write(&buf);
                if res.is_err() {
                    break;
                }
            }
            let _ = tx.send(res);
            pcm
        });
        let limit = Duration::from_secs_f64(secs * 2.0 + 2.0);
        match rx.recv_timeout(limit) {
            Ok(res) => {
                let wall = t0.elapsed().as_secs_f64();
                let cpu = (cpu_ticks() - c0) as f64 / 100.0;
                let audio = (chunks * CHUNK) as f64 / SAMPLE_RATE as f64;
                println!(
                    "probe: {label} write={res:?} audio={audio:.3}s wall={wall:.3}s cpu={cpu:.2}s cpu/wall={:.2}",
                    if wall > 0.0 { cpu / wall } else { 0.0 }
                );
                pcm = worker.join().expect("worker");
            }
            Err(_) => {
                let cpu = (cpu_ticks() - c0) as f64 / 100.0;
                println!(
                    "probe: {label} HANG（{:.1}s 内未写完 {secs}s 音频；cpu={cpu:.2}s）",
                    limit.as_secs_f64()
                );
                std::process::exit(2);
            }
        }
    }
    pcm.idle();
    println!("probe: done {}", pcm.describe());
}

#[cfg(not(target_os = "android"))]
fn main() {
    eprintln!("aaudio_probe 仅 Android 目标（见文件头构建/运行命令）");
}
