---
name: aaudio-ndk-constants-start-timeout
description: AAudio 直绑四坑——性能模式枚举自 10 起算（手写 1 = openStream 恒 ILLEGAL_ARGUMENT）、open 后必须 requestStart、write 负/零超时=不阻塞（非无限等待）、stop 会播完残留（丢弃须 pause→PAUSED→flush）；设备切换流即 DISCONNECTED 须重开；adb shell 探针取证不抢前台
type: gotcha
created: 2026-10-26
sources: ["305f678 BUG-027", "e4bac6c 设备切换重开", "NDK 27.1.12297006 aaudio/AAudio.h", "AOSP android-16.0.0_r1 media/libaaudio"]
---

手写 AAudio FFI 时四个"看起来对"的错误会叠成"全程无声"或"写环满核自旋"：
① `aaudio_performance_mode_t` 自 `NONE = 10` 起算（AAudio.h:325-338，LOW_LATENCY = 12），手写 1 ⇒
`AudioStreamBuilder::validate()` 恒拒（logcat `E/AudioStreamBuilder: illegal performanceMode = 1`）；
② openStream 后流停在 OPEN，不 `requestStart` 不消费——未启动的 legacy 流 write 立返 0 帧；
③ write 的 timeout **负/零 = 不阻塞**（AudioStreamTrack.cpp:505 `blocking = timeout > 0`；MMAP 截止 = now+timeout），
没有"负值 = 无限等待"；②③叠加 = 0 帧立返的自旋环（实机 6 s 内 CPU 5.97 s）；
④ `requestStop` 先播完已缓冲数据，要"停止并丢弃"须 pause → 等 PAUSED（pause 异步）→ flush（AAudio.h:1585 只在
OPEN/PAUSED/STOPPED/FLUSHED 允许）。另：插拔耳机/连蓝牙 ⇒ 流被 forceDisconnect（AudioStreamLegacy.cpp:283-303），
write 返 DISCONNECTED、start 返 INVALID_STATE——不重开就本次运行永久静音。

**为什么：** 这些全是"枚举不从 0 起""超时符号语义与 POSIX/ALSA 直觉相反""stop≠drop"的反直觉 API 约定；
骨架期"照头文件手写、未编译未实机"的 extern 层不会自己报错（编译过、链接过，只在运行时被服务端拒/空转）。

**何时使用：** 任何 NDK/系统库直绑（AAudio、AAssetManager、AInput…）：常量一律对头文件逐值核并在宿主测试钉值
（`shell/android/tests/aaudio_stream.rs` `abi_constants_match_ndk_header`），状态机抽 trait 用假流宿主测
（`audio::StreamOps`）。实机取证用 adb shell 跑的探针可执行（`shell/android/examples/aaudio_probe.rs`：
同一设备实现、只写静音、看门狗 + 墙钟/CPU 比判阻塞 vs 自旋，`logcat --pid` 收 libaaudio 自带诊断）——不起
Activity 不抢前台。注意 HyperOS 把 uid 2000 的 MMAP 策略收为 NEVER（`final mmap policy is 1`），探针只覆盖
legacy 路径；游戏取 PERFORMANCE_MODE_NONE 正是为了与探针同路径。
