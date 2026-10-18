---
name: shell-contract
description: 外部 shell 契约——实现一个平台壳（Windows/Android/…）必须提供的部件清单、平台接缝现状与 game-platform 路线
type: architecture
created: 2026-10-17
sources: ["2026-10-17 用户问（退出无反应 BUG-020 连带架构确认）", "AGENTS.md §2 仓库结构表", "docs/knowledge/decision_save-path-shell-injection.md"]
---

# Shell 契约：实现一个平台壳要做什么

## 现行架构（三层）

| 层 | 职责 | I/O |
|---|---|---|
| `game-core` | 游戏逻辑（状态机/脚本/战斗/存档**序列化**）；确定性、虚拟时钟、GameRng | **零 I/O**（存档写虚拟 RMS 字段、退出只发信号） |
| `game-platform` | 平台 trait 与类型（**现仅 Clock/GameRng/hash**——见路线） | 只定义 |
| shell（`game-desktop` / 未来 `game-android`…） | 一切平台依赖的实现与接线 | 全部 |

**原则**：游戏层只产生**信号与数据**，shell 负责**执行与持久**。路径/进程/设备都是平台职责。

## 平台接缝清单（shell 必须实现/注入）

> 【2026-10-18 实现落位】契约 trait 已实装于 `crates/game-platform`（spec：
> docs/spec/platform-contract.md v1.0）：#2=Storage、#3=ProcessControl、#4=AudioOut、
> #1=AssetSource；#5/#6/#8 为 shell 事件面契约（键码/呈现/生命周期）。
> **双壳已实现**：shell/win（5 commit，四缝 trait 化、行为零漂移）+ shell/android
> （8 commit，四 trait Android 版、45/45 测、验证矩阵见其 README）。

| # | 接缝 | 契约 | game-desktop 现实现 |
|---|---|---|---|
| 1 | **资产路径** | 注入 `--assets`（Android 宿主传应用资源目录） | `load_assets(dir)` |
| 2 | **存档路径 + 持久** | 注入 `--save`（见 decision_save-path-shell-injection）；消费 `saved_slot_blobs`/`slot` 变化写穿、回写 `preset_mot_if` | `Rms`（save/ 文件版 RecordStore）+ Shell::step 存档块 |
| 3 | **退出进程** | game-core 发 `CMidlet.m_000` 信号（`BootMachine::finished`），**shell 消费并退出**——游戏层永不 `process::exit` | `Shell::finished()` → `el.exit()`（BUG-020 补） |
| 4 | **音频输出** | 实现 `PcmDevice`（`write(frames) -> Result`，阻塞=节拍；`idle()`）；BGM 目录注入 `--audio` | `audio.rs`：winmm waveOut / ALSA |
| 5 | **输入映射** | 平台键/触控/**指针** → 游戏键码（-7..-6 软键 / -5 确认 / -4..-1 方向；键码契约见 key-code-contract）；指针行点击=Stage B 加法（decision_pointer-menu-stage-b：pointer_hit/handle_pointer 零新语义） | winit 事件 → press_key/release_key + handle_pointer |
| 6 | **窗口/呈现** | 逻辑拍节奏（`frame_interval`）+ 画布呈现（240×320 等比适配） | winit + `view::Pacer/Viewport` |
| 7 | **字体** | 真机字体度量注入（shim Font 度量为保真基准；桌面 CJK 候选发现） | `font_inject::install()` |
| 8 | **可见性/生命周期** | hide/show 语义（`set_minimized` → hideNotify 语义） | `about_to_wait` 轮询 |

## game-platform 路线（AGENTS.md 承诺集的落位计划）

现状 trait 子集：`GameClock`/`GameRng`/hash。待抽（随第二个 shell 的需求逐步上移，避免过早抽象）：

- `Storage`（=接缝 2 的 trait 化：read/write/list record stores）
- `ProcessControl`（=接缝 3：exit）
- `AudioSink`/`PcmDevice`（=接缝 4，现居 game-desktop::audio）
- `Renderer`/`Font`/`Input`（=接缝 5-7）

**演进规则**：第二个平台（Android）落地时把其真实需求抽上来（rule of three 的平台版）；
在此之前接缝以本文清单 + `--xxx` 注入缝为准，game-core 零改动即可接新壳。

## 补充登记（2026-10-18 双壳收口）

- **第五缝（BGM 轨道装载）**：`start()/track_path()/REGEN_CMD` = Windows 专用
  路径型装载（留 shell/win）；Android 轨道走 `AssetSource::read("track{i}.ogg")`
  （单元 5 已落，B2 处置）。音频共享面（Mixer/Track/decode/audio_loop）=
  `shell/common`（game-shell-common）单源，双壳消费。
- **接缝 7（字体）**：android 侧未覆盖 = **已知缺口**（native TODO 域，随呈现
  骨架任务补；README §8 同向登记）——裁定 5-3。
- **已知边界**：跨阶段数组持久 corner²（BUG-021 补注，取方案 (b) 不修，升级
  触发条件=差分真发现反例）；BACK 等四项待实机升 A（README 验证矩阵 ⏳ 行）。
- **纪律沉淀**：TDD 红证据三件套+temp-archive 取证法（pattern_tdd-red-evidence-
  with-gates）；共享树随时绿+红相不入树+pathspec 提交（gotcha_shared-workspace-
  bare-commit-sweeps-index）。

## 自检：新壳最小验收

1. 传齐 1/2/4 三路径（assets/save/audio）；
2. 退出链：主菜单"退出" → 退场动画 70 拍 → 进程退出；
3. 存档环：保存 → 回主菜单 → 读档列表可见 → 重启仍在；
4. 音频：BGM 连续无欠载；
5. `gates/cli.py check` 8/8（game-core 侧与壳无关，恒绿）。
