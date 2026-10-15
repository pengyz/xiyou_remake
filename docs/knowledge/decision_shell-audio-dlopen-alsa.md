---
name: shell-audio-dlopen-alsa
description: 桌面壳 BGM 输出选「lewton 解码 + 运行时 dlopen libasound.so.2」而非 rodio/cpal：后者经 alsa-sys 需构建期 libasound2-dev + pkg-config，违反不依赖系统包约定且让 CI 的 cargo test --workspace 编不过
type: decision
created: 2026-10-15
sources: ["S1.3（crates/game-desktop/src/audio.rs，747ba65）；Bevy/ggez 等 CI 需显式 apt install libasound2-dev 的公开先例"]
---

壳音频 = `lewton`（纯 Rust Vorbis 解码）+ `libloading` 运行时加载 `libasound.so.2`，只取 8 个
符号（snd_pcm_open/set_params/writei/recover/drop/prepare/close、snd_strerror），常量取自
`/usr/include/alsa/pcm.h`（STREAM_PLAYBACK=0、ACCESS_RW_INTERLEAVED=3、FORMAT_S16_LE=2）。
缺库/缺产物/非 Linux ⇒ 打印原因与再生命令后无声运行，永不 panic。

**为什么：** game-desktop 是根 workspace 成员，门禁 `cargo test --workspace` 会编译它；cpal 的
alsa-sys 构建脚本要 pkg-config 找 `alsa.pc`（libasound2-dev），本机碰巧有、CI 的
ubuntu-latest 没有 ⇒ 门禁全红，且违反 tools/README「不依赖系统包」。dlopen 把依赖推迟到
运行时、零构建期系统依赖；libloading 0.8.9 已在 winit 依赖树里（零新增下载）。代价：
目前只有 Linux 后端。

**何时使用：** 加 Windows/macOS 音频时保持 `AudioSink`/`PcmDevice` 抽象，按 target 条件依赖
引入（WASAPI/CoreAudio 无需额外系统包，可考虑 cpal 的非 Linux 后端）；任何新依赖先查
是否带 `-sys` 构建期系统库需求。端到端验证法见
[gotcha_alsa-file-pcm-unthrottled](gotcha_alsa-file-pcm-unthrottled.md)。
