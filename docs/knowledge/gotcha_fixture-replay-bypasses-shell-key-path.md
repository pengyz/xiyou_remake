---
name: fixture-replay-bypasses-shell-key-path
description: oracle 夹具重放直调 press_key/release_key，绕过壳层"投键前先 interrupt_route()"——壳层在 press_key 外围调用的 core 方法不在 Java 对拍覆盖内；BUG-025 夹具全绿、实机剧情被连点截断即此盲区
type: gotcha
created: 2026-10-26
sources: ["27bd75d fix(core): BUG-025", "381a248 test(oracle): 嫦娥剧情链夹具 Modern 重跑"]
---

`crates/game-oracle/tests/fixture_frames.rs` 的重放驱动（`replay_opt` 及同构 `replay_logic`）按
MIDP keyPressed/keyReleased 口径**直调** `GameScene::press_key` / `release_key`；真实壳层投键却先调
`interrupt_route()`（win `route_key`、android `replay_scene_keys`、触屏 `ShellAction::Interrupt`）。
壳层在 press_key 外围多调的 core 方法因此不受 Java 帧对拍约束：BUG-025 旧 `interrupt_route` 无条件清
walkPathBuffer，剧情里连点"确定"即截断嫦娥 MOV / 玩家脚本行走——既有 6 个嫦娥链夹具全绿，实机却复现。
补位：`replay_logic_with(.., shell_keys: true, ..)` 按壳层口径重放
（`fx_chang_e_floor1_taps_shell_key_path_frames_match`，修复前变异即红"TICK 387 帧不符"）。

**为什么：** 对拍裁判的是"core 对同一输入序列的响应"，但输入在 harness 与壳层是两条投递面；
Stage B 新增的壳层调用（打断寻路、walk_to、probe_step、undo）在 harness 里根本不发生，
"route 未开 = 幂等零操作"的 classic 零影响承诺只写在文档注释里，没有测试兑现（实现并不成立）。
Java 侧剧情中投键合法且无副作用（`reference/src/deobf/a.java:4663-4680` keyPressed 只写
keyValue/keyHeldCode + 续播边沿），所以偏差只会在"壳层多做的那一步"里。

**何时使用：** ① 实机复现、夹具全绿 → 先对照壳层与 harness 的投键路径，别先怀疑分辨率/布局；
② 改壳层输入路径或任何"壳层在 press_key 前后调用的 core 方法" → 跑/补 `shell_keys=true` 重放，并给该方法补
"无寻路段时零操作"类 core 单测（如 `t2_interrupt_route_without_open_route_keeps_script_walk`）；
③ 新夹具编排输入时覆盖"剧情行走中投键"（连点确定/方向），这正是旧夹具缺的输入形态。
