# 设置页新增「开屏画面 / 按键震动 / 按键音」——设计件（STAGE-B-006）

> 2026-10-28。用户诉求与裁定：
> - "游戏开头的开屏信息加开关去掉，开关放在游戏内设置菜单里，默认关闭，用户选择性打开，关闭后，启动直接现实游戏主菜单"；
> - "游戏设置里加入按键震动和按键音开关，默认都打开"；
> - 按键音规则："声音和按键音是分开的"。
>
> 设计 v0 经对抗复核（`docs/reviews/2026-10-28-settings-controls-design-adversarial-review.md`）修订为本件；复核意见
> S1（回标题回滚）已先行修复为 BUG-041，M3 / M5 / L1 的约束写入下文。

## 0. 背景事实（证据）

- **开屏 = mode 0 三屏**：l0（移动梦网）→ l1（百宝箱 / 9266）→ sflogo 动画，68 拍 × 100 ms ≈ 6.8 s（paint.rs `tick_mode0`；
  a.java:3349-3381；Android 节拍取 `Runtime::frame_interval_ms`，mode 0 = 100 ms）。mode 0 不读键、不用随机数，
  资源与 SKY_WAR 在开屏前由壳装载（runtime.rs `build_boot_machine` / `Runtime::new`）⇒ 跳过不漏初始化。
  新游戏后的序章（mode 14）不属开屏，本就可跳过。
- **设置页 = mode 16 `OptionList`**（menu_family.rs），两套入口：
  - 标题侧（BootMachine `enter_menu_mode` / `tick` 的 `GameMode::About` 臂）；
  - 游戏内（GameScene `enter_menu16` / `tick_menu16`）。
  - 原版只有两行：声音、小地图（`OPTION_LABELS`，a.java:811）。
  - 行文字用字体绘制（不需美术），盒宽 = 最宽标签 + 80，每行高 = 字高 + 8（`OptionList::add`，a.java:9911-9926）。
- **按键反馈现状**（shell/android/src/feedback.rs `cue_for`）：触感恒发；按键音跟随游戏内"声音"。按键音跟随声音是 2026-10-26 本方设计，
  不是用户裁定（头注只引用户原话"触发震动和按键声音"）。
- **设置文件**（shell/common/src/settings.rs `AppSettings` → `app_settings.cfg`，STAGE-B-001）只有 Windows 壳接入。
  Android 至今不读不写，声音开关每次启动回到开（复核 C6）。
- **原版无此三项**：全为新增行为（Stage B）。

## 1. 交互规格

| 行（自上而下） | 默认 | 作用 | 生效 |
|---|---|---|---|
| 声音 | 开 | 原有：背景音乐 | 立即 |
| 小地图 | 开 | 原有 | 立即 |
| 开屏画面 | **关** | 开 = 启动时播放开屏三屏；关 = 启动后第一帧即主菜单、标题曲直接开始 | 下次启动 |
| 按键震动 | 开 | 控制面板虚拟键按下的触感 | 立即 |
| 按键音 | 开 | 控制面板虚拟键按下的提示音；**与"声音"互不影响** | 立即 |

- 标题页「设置」与游戏内菜单「设置」都显示这 5 行；切换方式不变：确定 / ←→ / 点行。
- 新三行只在 Android 竖屏壳出现。classic 预设与 Windows 壳仍是原版两行。
- 系统层约束不变：
  - 系统"触感反馈"关闭时不震动；
  - 系统音量静音时按键音不响（`AudioManager.playSoundEffect(FX_KEY_CLICK, -1)` 的既有口径）。
- 所有取值跨启动保存在 Android `internalDataPath/app_settings.cfg`。声音开关一并保存，补齐 STAGE-B-001 在 Android 上的漏项。

## 2. 游戏核心（crates/game-core）

**2.1 壳层附加设置行**（core 只负责显示、切换、交接，不知道行的含义）
- 新类型 `ShellOption { id: u8, label: Vec<u16>, checked: bool }`。
  - `id` 由壳定义，core 不解释；
  - `label` 是 UTF-16 文本。
- 字段 `pub shell_options: Vec<ShellOption>`：BootMachine 与 GameScene 各一份，默认空。
  - 默认空 ⇒ classic、Windows、全部回放 / 夹具不变。
- `OptionList` 增加附加行列表：
  - 原两行（`labels` / `checked`）结构、绘制、按键语义一字不动；附加行排在其后。
  - 行几何与原行同式（`row_band`）；盒尺寸按 `add` 同一公式累加。
  - `len()` 返回总行数（`pointer_hit` / 高亮环绕据此）。
  - 新增 `OptionList::with_shell_rows(sound, minimap, &[ShellOption])`；`new()` 保持原样，内部等于零附加行。
  - **零附加行时绘制 op 序列逐字不变**（新测钉住；Windows modern 帧基线 `menu@200` 就是标题设置页，以此兜底）。
- 切换逻辑单源：`OptionList::toggle_shell_row(i) -> Option<(u8, bool)>`，两处入口共调。命中附加行时：
  - 翻转该行；
  - 同步到所在机器的 `shell_options`；
  - **不碰** bgm、小地图、`global` / `global_dirty`，即附加行不写 SKY_WAR（复核 M3）。
  - 原两行的处理分支不改，包括标题侧对任意原行置 `global_dirty` 的既有行为。
- 两处入口的高亮环绕由 `labels.len()` 改为 `len()`。无附加行时数值相同，行为不变。

**2.2 交接**（双机实现须随交接传递，复核 M3 / BUG-041 同理）
- 正向：`handoff::new_game_scene` / `slot_scene` 把 `machine.shell_options` 拷给场景。
- 反向：`handoff::return_to_title` 拷回。

**2.3 跳过开屏**：`BootMachine::skip_boot_splash() -> bool`。
- 只在 `mode == Boot` ∧ 尚未出拍（`phase == 0 && counter == 0`）∧ `!layout.boot_sound_prompt`（modern）时生效；否则不操作，返回 false。
- 把 `tick_mode0` 收尾抽成私有 `finish_boot_splash()`，自然播完与跳过共用。
  - logo 用 `get_or_insert_with(LogoAnim::new)` 取得后再 `clear_all` / `reset_layout_flag`。
    原 `as_mut().unwrap()` 在跳过时会 panic（复核 M5①）；自然路径 logo 恒为 Some，行为不变。
  - mode 22 绘制 `self.logo.as_ref().unwrap()`（paint.rs LogoAnim 臂）因此在跳过后仍成立。
  - 跳过时另把 `phase` 置 2，与自然播完同值。
- 等价判据（新测）：跳过后与自然播 68 拍后比较以下字段，须相同：
  - mode、标题机存在且光标为 0；
  - `bgm_request()`、`bgm.sound_enabled`；
  - `frame_interval_ms`、`finished`、`softkeys`、logo 存在。
- classic 门控：`boot_sound_prompt` 为真时返回 false（新测钉住）。classic 帧夹具与 boot 回放保证 `tick_mode0` 抽函数后 op 序列不变。
- 代码注释统一标 `// STAGE-B-006`。

## 3. 共享壳层（shell/common）

- `AppSettings` 增加字段：`show_splash`（默认 false）、`key_haptic`（默认 true）、`key_click`（默认 true）。
  - 文件格式仍是 `键=值` 行，缺键取默认。
  - **布尔坏值取默认**（原 `== "true"` 使坏值读成 false，复核 L1；声音行同改）。
  - `save` 写全部键（tmp + rename 原子写不变）。
- 不做降级兼容：旧版本只写已知键，降级会丢新键，属可接受范围。

## 4. Android 壳（shell/android）

- 行定义（壳常量）：
  - `PREF_SPLASH = 1`「开屏画面」
  - `PREF_HAPTIC = 2`「按键震动」
  - `PREF_CLICK = 3`「按键音」
- `Runtime::apply_app_settings(&AppSettings)`（启动时、首拍前调用，宿主可测）：
  - 写入 `machine.bgm.sound_enabled` / `volume_level`；
  - 生成三行 `shell_options`；
  - `show_splash == false` 时调用 `machine.skip_boot_splash()`。
  - 顺序满足复核 M5③：先写声音再跳开屏，否则 `enter_title` 按旧声音建播放器。
- `Runtime::app_settings(&self) -> AppSettings`：从活动机（有场景取场景，否则取 BootMachine）读出声音对与三行勾选，组成当前设置；
  缺行的字段取默认。
- `Runtime::key_feedback() -> (haptic, click)`：读活动机的两行勾选。
- `native.rs android_main` 接线：
  - `AndroidStorage::open` 之后读 `data_path/app_settings.cfg`，`Runtime::new` 后 `apply_app_settings`；
  - 每次 `rt.step` 之后比较 `rt.app_settings()` 与上次落盘值，不同即 `save`。在同一拍内完成，退出请求后不再出拍也不丢（复核 L1）；
  - `handle_input` 的反馈改为 `feedback::emit(fb, presses, haptic, click)`。
- 反馈策略 `feedback::cue_for(presses, haptic, click)`：
  - presses > 0 ∧（haptic ∨ click）⇒ `Some(Cue { haptic, click })`，否则 None；
  - 按键音只看"按键音"，不再看"声音"。
- 文档：feedback.rs 头注策略表、shell/android/README.md 的平台映射 / 设置说明原地更正（日期注）。

## 5. Windows 壳（shell/win）

- 不提供附加行、不跳开屏，交互不变。
- `persist_settings_if_changed` 的 `AppSettings` 字面量改为 `..上次装载值`：保留文件里的新键，Windows 不改写它们。
- 结果：modern 帧基线 5 状态不变、Windows shell_tests 不变。

## 6. 测试与验收（TDD，逐项 RED→GREEN）

| 层 | 测试 | 判据 |
|---|---|---|
| core menu_family | 零附加行 op 序列 = `new()`；附加行几何 / 盒尺寸；`toggle_shell_row` 只翻附加行 | RED：方法不存在 |
| core scene / paint | 两处设置页显示 5 行；切附加行只改 `shell_options`（bgm、小地图、global、global_dirty 不变）；高亮在 5 行内环绕 | 同上 |
| core paint | `skip_boot_splash` 等价判据；classic 返回 false；跳过后标题选"退出"再绘制 mode 22 不 panic | 同上 |
| shell/common | AppSettings 新键默认、往返、坏值取默认；交接正反向携带 `shell_options` | 同上 |
| android | `apply_app_settings`：无文件 ⇒ 首拍即主菜单、三行默认值；开屏开 ⇒ 仍走 mode 0；声音恢复。游戏内切换 ⇒ `app_settings` 快照变化；回标题保持 | 同上 |
| android feedback | `cue_for` 四象限；改写原"按键音随声音"测试（行为变更，登记 STAGE-B-006） | 原测改写 + 新测 |
| 门禁 | `gates/cli.py check` 12/12；ratchet 只增；classic 夹具 / boot 回放全绿；`tools/modern_frame_baseline.py --check` 不变 | — |
| 实机（手机连上后） | 首启直达主菜单；设置页 5 行截屏；开屏打开后重启出现三屏；震动 / 按键音分别关闭后用 `dumpsys vibrator_manager` / `dumpsys audio` 核对（`gotcha_android-key-feedback-jni-thread-and-switches` 的方法，以本方注入时刻对齐）；关声音重启仍关 | 截屏 + logcat / dumpsys |

## 7. 登记

- `docs/spec/stage-b-deviations.md` 新增 STAGE-B-006：偏离点 = 新增三行 + 开屏可跳过 + 按键音独立；用户原话；门控方式（壳提供行、
  modern 才跳开屏）；金样影响 = 无；改写的测试名单。
- STAGE-B-001 节加注：Android 已接入设置文件，声音跨启动保存。
- kairos 日志；知识库仅在实施中出现非显然的坑时写。

## 8. 不做

- Windows 侧的新行；音量滑条；设置文件降级兼容。
- 功能键面板（STAGE-B-007，另件）；筋斗云补移植（BUG-037，另件）。
- 小地图勾选不跨次保留（复核 M4）：先取证，属另案，本件不顺手改。
