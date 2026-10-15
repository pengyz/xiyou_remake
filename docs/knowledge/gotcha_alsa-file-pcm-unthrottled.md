---
name: alsa-file-pcm-unthrottled
description: ALSA `file:FILE=…`（null 从设备）不按采样率节流，snd_pcm_writei 立即返回——冒烟捕获实测 ~500 MB/s 写盘，几十秒即数 GB
type: gotcha
created: 2026-10-15
sources: ["S1.3 壳 BGM Xvfb 冒烟（/tmp/xiyou-smoke*/cap.raw 4.1–4.6 GB / ~20 s）"]
---

用 `--audio-device "file:FILE=/tmp/cap.raw"` 捕获桌面壳的 BGM 输出时，alsa.conf 的
`pcm.file` 以 `null` 为从设备，**没有硬件时钟**：阻塞写不再按 44.1 kHz 实时节流，音频线程
全速渲染，实测 500–600 MB/s，约 20 s 冒烟就落下 4+ GB 原始 PCM。

**为什么：** null 插件只丢数据不计时；`file` 插件把收到的每一帧原样写文件。对壳来说这
等于「无限快的声卡」——依赖 writei 阻塞来控速的播放循环会空转满核。

**何时使用：** ①要校验「壳实际写出的样本」（如与 lewton 解码 × 增益逐样本比对）时，
只捕获几秒并立即截取头部（`head -c`）后删除，或改用有节流的设备；②冒烟只需验证通路/
暂停语义时用 `--audio-device null`（同样不节流但不写盘）；③凡以「捕获文件增长量」判断
是否在播放，结论只能是「有/无写出」，不能当速率证据（增长 ≠ 176400 B/s）。
