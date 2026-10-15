---
name: os-key-repeat-is-not-keypressed
description: 桌面壳不得把 OS 键盘自动重复当 keyPressed 投递——a.class 未覆写 keyRepeated，长按行走靠 keyHeldCode；透传会让菜单长按连跳、对话长按连翻
type: gotcha
created: 2026-10-15
sources: ["36d1a9d（repeat 透传为 press）→ 747ba65 回退；javap -p 原始 a.class；EV-hold-walk 变体 A 取证"]
---

MIDP 对按住不放的键调用 `Canvas.keyRepeated`，不再调 `keyPressed`；原始 `a.class`
（`javap -p assets/raw/a.class`）只覆写 `keyPressed`/`keyReleased`，`keyRepeated` 走基类空实现
⇒ **重复事件对游戏无效果**。长按连续行走由 `keyHeldCode` 驱动（handleFieldInput
a.java:5308-5329 每拍读 held 键 tryStep），按下一次即可连走；oracle 取证 EV-hold-walk：单次
press(-3) 长按 70 拍，战斗后连走两格（96→32）。

**为什么：** 「按住方向键不连走」的实机症状真因是事件循环未设 `ControlFlow::WaitUntil`
（逻辑只在事件到达时推进），而非吞了重复事件。把重复透传为 press 掩盖了症状，却在
keyValue 驱动的模式里引入原版没有的行为：标题/游戏菜单长按光标连跳、mode 11 对话长按
-5 连翻页、关浮层清 keyHeldCode（a.java:3345）后被重复事件重新拉起行走。

**何时使用：** 改壳输入路由时：重复事件一律丢弃（`translate_key` 返回 None）；按下/抬起
只改状态、不即时推进逻辑（R-2：同拍多键覆盖式取末个）；回归用
`key_repeat_ignored_and_last_press_wins` 与 `hold_walk_and_bgm_match_java_trace`（两者均经
变异验证——重复透传 / 恢复即时 tick 时各自失败）。
