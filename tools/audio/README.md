# tools/audio — BGM 离线转换（.mid → .ogg）+ 选轨语义取证

两件工具：

| 文件 | 用途 |
|---|---|
| `src/main.rs`（bin `midi2ogg`） | `assets/raw/{logostart,logoquit,menu,game}.mid` → `assets/audio/track{0..3}.ogg` + `manifest.json`（桌面壳播放用） |
| `evidence.py` + `evidence/*.txt` | BGM 选轨语义的 oracle 运行时取证（变体 A，逐 tick 全量），结论见 [docs/spec/audio.md](../../docs/spec/audio.md) |

## 一键再生

```bash
python3 tools/extract-assets/extract.py                      # 若 assets/raw 缺失
cargo run --release --manifest-path tools/audio/Cargo.toml   # → assets/audio/（gitignore，可再生）
```

可选参数：`-- --raw <dir> --out <dir> --sf2 <file>`。本目录是**独立 cargo workspace**
（`Cargo.toml` 空 `[workspace]` 表），不进仓库根 `cargo test --workspace`；构建产物在
`tools/audio/target/`（gitignore）。`Cargo.lock` 入库，依赖版本 `=` 钉死。

产物（2026-10-14 实测，两次运行 sha256 逐字节一致）：

| 文件 | 来源 | 圈长（帧 / 秒） | 大小 | SNR |
|---|---|---|---|---|
| track0.ogg | logostart.mid（死资源，BUG-010） | 151424 / 3.434 | 72 KB | 27.5 dB |
| track1.ogg | logoquit.mid（死资源，BUG-010） | 382336 / 8.670 | 160 KB | 27.8 dB |
| track2.ogg | menu.mid（标题曲） | 605056 / 13.720 | 642 KB | 22.9 dB |
| track3.ogg | game.mid（游戏曲） | 3200320 / 72.570 | 2714 KB | 25.2 dB |

## 管线

1. **输入完整性**：SoundFont 与 4 个 MIDI 逐个 sha256 校验（常量钉死于 `src/main.rs`；
   MIDI 源头 `original/` JAR 由 `gates/cli.py original` 校验）。轨道顺序取
   `game_core::audio::TRACKS`（a.java:2239，单一真相源）。
2. **合成**：rustysynth 44.1 kHz 立体声，序列器**循环模式**连续渲染两圈：第 1 圈=前奏
   （干净起音），第 2 圈=循环体（带上一圈的释音/混响尾）。圈长取序列器实际回绕点
   （= MIDI 末事件时刻按 64 样本块向上取整，≤1.45 ms 误差），并断言等于理论值。
   对应 Java `setLoopCount(-1)` 无限循环（a.java:10184）。
3. **增益**：4 轨联合峰值归一化到 -1 dBFS（同一增益，轨间相对响度不变）。
4. **编码**：rusty_vorbis q10 + ogg 封装。rusty_vorbis 首/末长块各有半窗无输出
   （实测解码 = 输入[1024..]），故前置 1024、后置 2048 零样本补偿；末页 granule =
   有效帧数（ffprobe `duration_ts` 精确等于 2×圈长）。
5. **Vorbis 注释**：`LOOPSTART`/`LOOPLENGTH`（样本）= 圈长——播放器播完前奏后在
   `LOOPSTART+LOOPLENGTH` 处跳回 `LOOPSTART`；另有 `TITLE`、`XIYOU_SOURCE`、`XIYOU_SOUNDFONT`。
6. **自检**（任一失败即退出 1，不写半成品清单）：lewton（与桌面壳同一解码器）回解 →
   帧数 ≥ 2×圈长、互相关零时延、波形 SNR ≥ 15 dB、循环标签回读一致；接缝误差打印
   （真实续接 vs 回跳，实测 rms ≤ 5e-9）。另经 ffmpeg 独立解码交叉验证（人工，非依赖）。

## SoundFont：来源与许可

`tools/vendor/TimGM6mb.sf2`（5,969,788 B，sha256
`c5378b62028c920cb11e4803327983fee2f2cdff5dc89c708e39da417e51c854`），许可文本
`tools/vendor/TimGM6mb-COPYING.txt`。

- 作者：Tim Brechbill（2004）、David Bolton（2010）；MuseScore 1.x 随附的 GM 音色库。
- 两个独立来源逐字节一致（2026-10-14 实算）：Debian `timgm6mb-soundfont_1.3.orig.tar.gz`
  （deb.debian.org pool）与 github.com/arbruijn/TimGM6mb（master）。
- **许可 GPL-2**（Debian copyright 记载作者声明样本为自制/公有领域/GPL）。本仓库按 GPL-2
  随附原文件与许可全文。渲染产物 `assets/audio/` 为本地可再生物、**不入库不分发**；
  若将来对外分发含渲染音频的构建，需评估 GPL-2 对派生音频的义务，或换用宽松许可音色库
  （如 FluidR3_GM，MIT，约 141 MB）。
- 选型理由：完整 GM 128 音色、体积约 6 MB（入库成本可接受），且来源可经两条独立渠道
  逐字节核验；宽松许可的完整 GM 库体积普遍大一个数量级（GeneralUser GS ≈ 32 MB）。

依赖 crate 许可：rustysynth MIT、rusty_vorbis Apache-2.0、ogg BSD-3-Clause、
lewton MIT/Apache-2.0；均纯 Rust、无系统包（tools/README.md 约定）。

## 已知偏差（Stage A 不覆盖音色保真）

- **音色**：原机由手机 MIDI 合成器发声；这里是 SoundFont 采样合成，音色/混响/响度曲线
  不同。Stage A 的像素级一致不含音频，此偏差显式接受。
- **循环接缝**：圈长按 64 样本块取整（≤1.45 ms）；MMAPI 循环重启时真机合成器的释音
  处理由实现决定，此处取「连续合成稳态」。
- **有损编码**：Vorbis q10（rusty_vorbis 的 q 刻度码率低于 libvorbis 同 q），相对合成
  输出 SNR 22–28 dB。
- **音量**：MMAPI VolumeControl 级别（游戏恒下发 60）到增益的映射由实现决定；桌面壳
  取线性幅度 level/100，见 docs/spec/audio.md §5。
