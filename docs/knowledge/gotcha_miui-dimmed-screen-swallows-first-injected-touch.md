---
name: miui-dimmed-screen-swallows-first-injected-touch
description: 小米实机屏幕已变暗时首个 adb 注入触点被系统吞（Input event injection failed）——游戏收不到，不是游戏输入缺陷
type: gotcha
created: 2026-10-27
sources: ["2026-10-27 BUG-033 实机复核：14:29:12 首个 input tap 丢失、14:29:55 重发送达"]
---

Xiaomi 14（HyperOS）屏幕已因无操作变暗时，`adb shell input tap` 的首个触点只被用来唤亮：system_server 侧日志
`MiuiInputManagerService: pokeUserActivity display id 0` → `MiInputDispatcher: set display0 dim state 0` →
`InputManager: Input event injection from pid … failed.`，游戏进程无 `input-arrival`；随后重发同一坐标正常送达
（`motion phase=Down … keys=[Press(-3)]`）。

**为什么：** 变暗态的首触在输入分发层被消费为唤亮，不投递前台窗口；失败痕迹只在 system_server 日志里，
按游戏 pid（`logcat --pid`）或 `-s jjxy-input` 过滤时完全看不到，看起来像"游戏没收到点按"。
**何时使用：** 实机注入"第一下没反应"时，先 `adb logcat -d | grep -E "injection .* failed|dim state"` 查系统侧，
别据此判游戏输入缺陷；验证脚本对关键注入要核对游戏侧 `tick batch` 日志后再进下一步，缺失即重发一次。
