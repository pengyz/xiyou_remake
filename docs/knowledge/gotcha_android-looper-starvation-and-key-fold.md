---
name: android-looper-starvation-and-key-fold
description: NativeActivity 主循环 `while ALooper_pollAll(16)>=0` 在触摸 Move 流下饿死 tick；按键批"取末动作"把点按 [Press,Release] 折成 0——两者叠加=触屏全不通
type: gotcha
created: 2026-10-26
sources: ["Xiaomi 14 设备 DIAG 日志：长按 1000ms 逻辑拍断档 1053ms、batch=[Press(-5), Release] 后标题不动", "shell/android/src/pump.rs + native_parse 钉"]
---

Android 壳"操作不通"的根因是两个平台层坑叠加，与菜单设计无关：① 内层 `while ALooper_pollAll(16) >= 0`
在 120Hz Move 事件流下每次都有就绪源、永不退出 ⇒ 按住期间 tick/paint 全停；② 每拍把键动作批 fold 成
"最后一个动作"，点按（或①导致的抬手后同批）`[Press(c), Release]` ⇒ 0，键被吞。

**为什么：** glue 的 process_input 本身排空整队，外层再按"有事件就继续等"循环 = 事件越密越饿；
BootMachine 只读 keyValue（keyPressed 覆盖式写、keyReleased 不清 keyValue），Release 不应抹掉同拍按下
（win 壳 `pending_machine_key` 早已如此）。修法：`pump_once` 非阻塞 poll(0) 排空→到期出拍→poll(余量)
阻塞；`boot_key_from_batch` 按下锁存、抬起对 BootMachine 无效；paint 每拍一次（标题粒子/图标带在
paint 内推进，多画=动画变快）。

**何时使用：** 改 android_main 主循环/输入批处理时；任何"点了没反应但日志有 Press"的触屏问题先查
批内容（`tick batch=` 日志）与拍间隔，再怀疑命中几何。宿主钉：`pump_keeps_ticking_under_continuous_touch_moves`、
`boot_key_latch_keeps_tap_press_across_release`。
