//! midi2ogg —— 《囧囧西游》BGM 离线转换：`assets/raw/*.mid` → `assets/audio/track{0..3}.ogg`。
//!
//! 管线（全部纯 Rust、确定性：同输入两次运行产物逐字节一致）：
//!   1. 输入完整性：SoundFont 与 4 个 MIDI 逐个 sha256 校验（钉死值见常量）；
//!   2. 合成：rustysynth 44.1 kHz 立体声，序列器**循环模式**连续渲染两圈——
//!      第 1 圈 = 前奏（干净起音），第 2 圈 = 循环体（含上一圈的释音/混响尾，
//!      与无限循环的稳态一致 ⇒ 接缝无断点）。圈长由序列器回绕点实测（= MIDI 末事件
//!      时刻按 64 样本块向上取整），对应 Java `setLoopCount(-1)`（a.java:10184）；
//!   3. 增益：4 轨**联合**峰值归一化到 -1 dBFS（保持轨间相对响度）；
//!   4. 编码：rusty_vorbis（q10）+ ogg 封装，Vorbis 注释写 LOOPSTART/LOOPLENGTH（样本）；
//!   5. 自检：lewton（与桌面壳同一解码器）回解 → 帧数覆盖、零时延对齐、SNR 门槛、
//!      循环标签回读、接缝误差。
//!
//! 用法：`cargo run --release --manifest-path tools/audio/Cargo.toml`
//!       [-- --raw <dir>] [--out <dir>] [--sf2 <file>]

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use game_platform::hash::sha256_hex;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};

const SAMPLE_RATE: u32 = 44_100;
/// rustysynth 默认块长（SynthesizerSettings::new 的 block_size）；序列器按块处理事件。
const BLOCK: usize = 64;
/// rusty_vorbis 品质（Vorbis `-q` 刻度，上限 10）。实测 menu 191 kbps / game 149 kbps。
const VORBIS_Q: f64 = 10.0;
/// rusty_vorbis 首个 2048 长块的左半窗无重叠伙伴 ⇒ 解码输出 = 输入[1024..]（实测
/// lewton 与 ffmpeg 两解码器一致）。前置 1024 个零样本补偿，使解码第 0 帧 = 源第 0 帧。
const ENCODER_PRIMING: usize = 1024;
/// 末块右半窗同理无输出：后置 2048 零样本保证 [0, 2·圈长) 全部可解码。
const TAIL_PAD: usize = 2048;
/// 联合归一化目标峰值（-1 dBFS）。
const PEAK_TARGET: f32 = 0.891;
/// 自检门槛：解码 vs 源（同增益）波形 SNR 下限。实测 q10 为 22–25 dB。
const MIN_SNR_DB: f64 = 15.0;

/// 入库 SoundFont（tools/vendor/，来源与许可见 tools/audio/README.md）。
const SF2_DEFAULT: &str = "tools/vendor/TimGM6mb.sf2";
const SF2_SHA256: &str = "c5378b62028c920cb11e4803327983fee2f2cdff5dc89c708e39da417e51c854";

/// 4 个 MIDI 的 sha256（2026-10-14 实算于 extract-assets 产物；源头 original/ JAR 由
/// gates original-integrity 校验）。顺序 = `game_core::audio::TRACKS`（a.java:2239）。
const MIDI_SHA256: [&str; 4] = [
    "22b3560fff3ca31229b538876727257a115e98e5f015617987be6f7c9eb7d2e3", // logostart.mid
    "ca7980a21fb22d274965bb25131bd1b71ab3270870e4dd61c30cc99a22f97afa", // logoquit.mid
    "c739c21221c9e687632cc1e9de977183e8c0559e044b9cf9b293f0914084c963", // menu.mid
    "4ee0260bdb1ab28b784198f238d8c02c876ad1b112c63866d390d5722a8eb63d", // game.mid
];

type Res<T> = Result<T, String>;

struct Rendered {
    name: &'static str,
    midi_sha: String,
    /// 一圈样本数（每声道）。
    period: usize,
    /// 交错立体声 f32，长度 = 2·period·2（前奏 + 循环体）。
    pcm: Vec<f32>,
    /// 第 3 圈开头 4410 帧（接缝自检：真实续接 vs 回跳到第 2 圈开头）。
    seam_probe: Vec<f32>,
    peak: f32,
    midi_len_s: f64,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> Res<Vec<u8>> {
    std::fs::read(path).map_err(|e| format!("读取 {} 失败：{e}", path.display()))
}

fn render(sf: &Arc<SoundFont>, name: &'static str, bytes: &[u8]) -> Res<Rendered> {
    let midi = Arc::new(
        MidiFile::new(&mut Cursor::new(bytes)).map_err(|e| format!("{name} 解析失败：{e:?}"))?,
    );
    let settings = SynthesizerSettings::new(SAMPLE_RATE as i32);
    let synth = Synthesizer::new(sf, &settings).map_err(|e| format!("合成器初始化失败：{e:?}"))?;
    assert_eq!(synth.get_block_size(), BLOCK, "rustysynth 块长变化：圈长换算前提失效");
    let mut seq = MidiFileSequencer::new(synth);
    seq.play(&midi, true);

    let (mut l, mut r) = (vec![0f32; BLOCK], vec![0f32; BLOCK]);
    let mut out: Vec<f32> = Vec::new();
    let mut resets: Vec<usize> = Vec::new();
    let mut prev = seq.get_position();
    let mut block = 0usize;
    // 渲染到第 2 次回绕之后再多 4410 帧（接缝探针）
    let mut stop_at: Option<usize> = None;
    loop {
        seq.render(&mut l, &mut r);
        for i in 0..BLOCK {
            out.push(l[i]);
            out.push(r[i]);
        }
        block += 1;
        let pos = seq.get_position();
        if pos < prev {
            resets.push(block - 1);
            if resets.len() == 2 {
                stop_at = Some(block * BLOCK + 4410);
            }
        }
        prev = pos;
        if let Some(s) = stop_at {
            if block * BLOCK >= s {
                break;
            }
        }
        if block > 20_000_000 / BLOCK {
            return Err(format!("{name}：序列器未回绕（>450 s），疑似 MIDI 结构异常"));
        }
    }
    let period_blocks = resets[1] - resets[0];
    if resets[0] != period_blocks {
        return Err(format!("{name}：首圈 {} 块 ≠ 稳态圈 {period_blocks} 块", resets[0]));
    }
    let period = period_blocks * BLOCK;
    let expect = ((midi.get_length() * SAMPLE_RATE as f64 / BLOCK as f64).ceil() as usize) * BLOCK;
    if period != expect {
        return Err(format!("{name}：圈长 {period} ≠ ceil(MIDI 长度/块)·块 = {expect}"));
    }
    let pcm = out[..period * 2 * 2].to_vec();
    let seam_probe = out[period * 2 * 2..(period * 2 + 4410) * 2].to_vec();
    let peak = pcm.iter().fold(0f32, |m, v| m.max(v.abs()));
    Ok(Rendered {
        name,
        midi_sha: sha256_hex(bytes),
        period,
        pcm,
        seam_probe,
        peak,
        midi_len_s: midi.get_length(),
    })
}

/// Ogg Vorbis 封装。粒度（granule）= 解码累计帧数：首个音频包 0，其后每包 +1024
/// （rusty_vorbis 全长块 n=2048、跳 1024）；末页粒度 = 有效帧数 `total`（支持按粒度
/// 截尾的解码器得到精确长度）。超出 `total` 的尾包强制同页，保证页粒度单调。
fn mux(packets: &[rusty_vorbis::EncodedPacket], comments: &[(&str, &str)], total: u64, serial: u32) -> Res<Vec<u8>> {
    use ogg::writing::PacketWriteEndInfo as E;
    if packets.len() < 4 {
        return Err("编码器未产出音频包".into());
    }
    let mut out = Vec::new();
    {
        let mut w = ogg::writing::PacketWriter::new(&mut out);
        let io = |e: std::io::Error| format!("ogg 写入失败：{e}");
        w.write_packet(packets[0].data.clone(), serial, E::EndPage, 0).map_err(io)?;
        let comment = rusty_vorbis::write_comment_header("xiyou-audio-tool (rusty_vorbis 0.1.1)", comments);
        w.write_packet(comment, serial, E::NormalPacket, 0).map_err(io)?;
        w.write_packet(packets[2].data.clone(), serial, E::EndPage, 0).map_err(io)?;
        let audio = &packets[3..];
        let granule = |k: usize| (k as u64) * 1024; // 第 k 个（0 基）音频包之后的累计帧数
        let cut = (0..audio.len()).rev().find(|&k| granule(k) <= total).unwrap_or(0);
        let mut segs = 0usize;
        for (k, p) in audio.iter().enumerate() {
            let last = k + 1 == audio.len();
            segs += p.data.len() / 255 + 1;
            let (info, gp) = if last {
                (E::EndStream, total)
            } else if k > cut {
                (E::NormalPacket, total) // 尾包：不结页（页粒度只取末个完成包）
            } else if k == cut || segs > 200 {
                segs = 0;
                (E::EndPage, granule(k))
            } else {
                (E::NormalPacket, granule(k))
            };
            w.write_packet(p.data.clone(), serial, info, gp).map_err(io)?;
        }
    }
    Ok(out)
}

struct Verified {
    frames: usize,
    snr_db: f64,
    loop_start: usize,
    loop_length: usize,
}

/// lewton 回解自检（桌面壳同一解码器）。
fn verify(ogg: &[u8], src: &[f32], period: usize) -> Res<Verified> {
    let mut rd = lewton::inside_ogg::OggStreamReader::new(Cursor::new(ogg))
        .map_err(|e| format!("lewton 拒收：{e:?}"))?;
    if rd.ident_hdr.audio_channels != 2 || rd.ident_hdr.audio_sample_rate != SAMPLE_RATE {
        return Err("声道/采样率不符".into());
    }
    let tag = |k: &str| -> Res<usize> {
        rd.comment_hdr
            .comment_list
            .iter()
            .find(|(kk, _)| kk.eq_ignore_ascii_case(k))
            .and_then(|(_, v)| v.parse().ok())
            .ok_or(format!("缺 {k} 标签"))
    };
    let (ls, ll) = (tag("LOOPSTART")?, tag("LOOPLENGTH")?);
    let mut dec: Vec<i16> = Vec::new();
    while let Some(p) = rd.read_dec_packet_itl().map_err(|e| format!("解码失败：{e:?}"))? {
        dec.extend_from_slice(&p);
    }
    let frames = dec.len() / 2;
    if frames < 2 * period {
        return Err(format!("解码帧数 {frames} < 前奏+循环体 {}", 2 * period));
    }
    // 零时延对齐：在 ±256 帧内找互相关峰，必须落在 0
    let mid = period / 2;
    let win = 8192.min(period / 2);
    let corr = |lag: i64| -> f64 {
        (mid..mid + win)
            .step_by(3)
            .map(|i| src[2 * i] as f64 * dec[(2 * (i as i64 + lag)) as usize] as f64)
            .sum()
    };
    let best = (-256i64..=256).max_by(|&a, &b| corr(a).partial_cmp(&corr(b)).unwrap()).unwrap();
    if best != 0 && corr(0) > 0.0 && corr(best) > corr(0) * 1.001 {
        return Err(format!("解码时延 {best} 帧 ≠ 0（编码器前置补偿失效）"));
    }
    let (mut sig, mut err) = (0f64, 0f64);
    for i in 0..2 * period * 2 {
        let a = src[i] as f64;
        let b = dec[i] as f64 / 32768.0;
        sig += a * a;
        err += (a - b) * (a - b);
    }
    let snr_db = 10.0 * (sig / err.max(1e-30)).log10();
    if snr_db < MIN_SNR_DB {
        return Err(format!("SNR {snr_db:.1} dB < 门槛 {MIN_SNR_DB} dB"));
    }
    Ok(Verified { frames, snr_db, loop_start: ls, loop_length: ll })
}

fn run(args: &[String]) -> Res<()> {
    let root = repo_root();
    let mut raw = root.join("assets/raw");
    let mut out_dir = root.join("assets/audio");
    let mut sf2 = root.join(SF2_DEFAULT);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut next = || it.next().cloned().ok_or(format!("{a} 缺参数"));
        match a.as_str() {
            "--raw" => raw = PathBuf::from(next()?),
            "--out" => out_dir = PathBuf::from(next()?),
            "--sf2" => sf2 = PathBuf::from(next()?),
            other => return Err(format!("未知参数 {other}")),
        }
    }

    let sf_bytes = read(&sf2)?;
    let sf_sha = sha256_hex(&sf_bytes);
    if sf_sha != SF2_SHA256 {
        return Err(format!(
            "SoundFont sha256 不符：{}\n  期望 {SF2_SHA256}\n  实际 {sf_sha}\n\
             正本：Debian timgm6mb-soundfont_1.3.orig.tar.gz 或 github.com/arbruijn/TimGM6mb（见 tools/audio/README.md）",
            sf2.display()
        ));
    }
    let sf = Arc::new(SoundFont::new(&mut Cursor::new(&sf_bytes)).map_err(|e| format!("SF2 解析失败：{e:?}"))?);
    eprintln!("[midi2ogg] SoundFont {} sha256 ✓", sf2.display());

    let mut tracks = Vec::new();
    for (i, name) in game_core::audio::TRACKS.iter().enumerate() {
        let path = raw.join(name);
        let bytes = read(&path).map_err(|e| format!("{e}\n先跑：python3 tools/extract-assets/extract.py"))?;
        let sha = sha256_hex(&bytes);
        if sha != MIDI_SHA256[i] {
            return Err(format!("{name} sha256 {sha} 与原始 JAR 产物不符（重跑 extract.py）"));
        }
        let t = render(&sf, name, &bytes)?;
        eprintln!(
            "[midi2ogg] track{i} {name}: MIDI {:.3}s → 圈长 {} 帧（{:.4}s），峰值 {:.3}",
            t.midi_len_s, t.period, t.period as f64 / SAMPLE_RATE as f64, t.peak
        );
        tracks.push(t);
    }
    let global_peak = tracks.iter().fold(0f32, |m, t| m.max(t.peak));
    let gain = PEAK_TARGET / global_peak.max(1e-6);
    eprintln!("[midi2ogg] 联合峰值 {global_peak:.3} → 增益 ×{gain:.4}");

    std::fs::create_dir_all(&out_dir).map_err(|e| format!("建目录 {} 失败：{e}", out_dir.display()))?;
    let mut manifest_tracks = Vec::new();
    for (i, t) in tracks.iter().enumerate() {
        let src: Vec<f32> = t.pcm.iter().map(|v| v * gain).collect();
        // 接缝：真实续接（第 3 圈开头）vs 回跳到第 2 圈开头
        let mut e = 0f64;
        for k in 0..t.seam_probe.len() {
            let d = (t.seam_probe[k] - t.pcm[t.period * 2 + k]) as f64 * gain as f64;
            e += d * d;
        }
        let seam_rms = (e / t.seam_probe.len() as f64).sqrt();

        let mut padded = vec![0f32; ENCODER_PRIMING * 2];
        padded.extend_from_slice(&src);
        padded.extend(std::iter::repeat(0f32).take(TAIL_PAD * 2));
        let mut enc = rusty_vorbis::VorbisEncoder::new(rusty_vorbis::VorbisEncoderConfig {
            quality: rusty_vorbis::quality01_from_vorbis_q(VORBIS_Q),
            ..Default::default()
        });
        enc.push_pcm_f32(&padded, 2, SAMPLE_RATE).map_err(|e| format!("编码失败：{e:?}"))?;
        enc.finish();
        let mut packets = Vec::new();
        loop {
            match enc.next_packet() {
                Ok(p) => packets.push(p),
                Err(rusty_vorbis::Error::Eof) => break,
                Err(e) => return Err(format!("编码失败：{e:?}")),
            }
        }
        let p = t.period.to_string();
        let src_tag = format!("{}:{}", t.name, &t.midi_sha[..16]);
        let sf_tag = format!("TimGM6mb.sf2:{}", &SF2_SHA256[..16]);
        let comments = [
            ("TITLE", t.name),
            ("LOOPSTART", p.as_str()),
            ("LOOPLENGTH", p.as_str()),
            ("XIYOU_SOURCE", src_tag.as_str()),
            ("XIYOU_SOUNDFONT", sf_tag.as_str()),
        ];
        let ogg = mux(&packets, &comments, (2 * t.period) as u64, 0x5859_0000 + i as u32)?;
        let v = verify(&ogg, &src, t.period).map_err(|e| format!("track{i} {} 自检失败：{e}", t.name))?;
        if v.loop_start != t.period || v.loop_length != t.period {
            return Err(format!("track{i} 循环标签回读不符"));
        }
        let file = format!("track{i}.ogg");
        std::fs::write(out_dir.join(&file), &ogg).map_err(|e| format!("写 {file} 失败：{e}"))?;
        let ogg_sha = sha256_hex(&ogg);
        eprintln!(
            "[midi2ogg] → {file} {} KB，{:.0} kbps，SNR {:.1} dB，接缝 rms {:.1e}，解码 {} 帧",
            ogg.len() / 1024,
            ogg.len() as f64 * 8.0 / (2.0 * t.period as f64 / SAMPLE_RATE as f64) / 1000.0,
            v.snr_db,
            seam_rms,
            v.frames
        );
        manifest_tracks.push(format!(
            "    {{\"index\": {i}, \"source\": \"{}\", \"source_sha256\": \"{}\", \"ogg\": \"{file}\", \
             \"ogg_sha256\": \"{ogg_sha}\", \"loop_start\": {}, \"loop_length\": {}, \"snr_db\": {:.1}}}",
            t.name, t.midi_sha, t.period, t.period, v.snr_db
        ));
    }
    let manifest = format!(
        "{{\n  \"tool\": \"xiyou-audio-tool {} (tools/audio)\",\n  \"soundfont\": {{\"file\": \"TimGM6mb.sf2\", \"sha256\": \"{SF2_SHA256}\"}},\n  \
         \"sample_rate\": {SAMPLE_RATE},\n  \"vorbis_q\": {VORBIS_Q},\n  \"joint_gain\": {gain:.6},\n  \"tracks\": [\n{}\n  ]\n}}\n",
        env!("CARGO_PKG_VERSION"),
        manifest_tracks.join(",\n")
    );
    std::fs::write(out_dir.join("manifest.json"), manifest).map_err(|e| format!("写 manifest 失败：{e}"))?;
    eprintln!("[midi2ogg] 完成 → {}", out_dir.display());
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(&args) {
        eprintln!("[midi2ogg] 失败：{e}");
        std::process::exit(1);
    }
}
