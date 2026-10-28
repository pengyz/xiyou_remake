# 设置页新增「开屏画面 / 按键震动 / 按键音」Implementation Plan（STAGE-B-006）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Android 竖屏壳的两处设置页（标题侧 / 游戏内）在"声音、小地图"之后新增 开屏画面（默认关）、按键震动（默认开）、
按键音（默认开）三行；开屏关 ⇒ 启动第一帧即主菜单；按键音与"声音"互不影响；全部取值（含声音）跨启动保存在
`internalDataPath/app_settings.cfg`。classic 预设与 Windows 壳交互不变。

**Architecture:** core 只提供"壳层附加行"的显示 / 切换 / 交接（`ShellOption`，不解释含义）与 `skip_boot_splash()`；
行的定义、设置文件读写、按键反馈策略全在 Android 壳（`runtime.rs` / `feedback.rs` / `native.rs`）。零附加行 ⇒ 绘制 op 流与
改前逐字相同（classic 夹具、boot 回放、Windows modern 帧基线兜底）。

**Tech Stack:** Rust（game-core / game-shell-common / game-android / xiyou win 壳），cargo test，cargo-ndk + gradle，adb 实机。

设计件：`docs/superpowers/specs/2026-10-28-settings-splash-key-feedback-design.md`（`ba7e92e`）。

**每个任务的纪律（用户指示"每一步对抗性复核 + TDD 验证"）：**
1. 先写测试，跑一次确认 RED（编译失败或断言失败，记下失败原因）；
2. 最小实现到 GREEN；跑本 crate 全量测试；
3. 对抗复核：派独立审查代理读 `git diff` + 设计件，专找行为缺陷 / 漏改调用点 / 测试漏洞，结论写
   `target/verify/review_T<n>.md`；逐条处理（修复或写明驳回理由）后重跑测试；
4. commit（pre-commit 自动跑 `gates/cli.py check`；commit message 含"动手前三问"：层 / 消费者 / 证据级）。

**实施细化（相对设计件 §4 的一处位置调整）：** "每拍后比较快照、变化即写"放进 `Runtime`（`open_app_settings` 记住路径与
已落盘值，`step` 末比较落盘），native.rs 只调一次 `open_app_settings`。时机与设计件相同（同一拍内落盘），好处是宿主测试能直接
覆盖写盘路径。

---

## File Structure

| 文件 | 职责 | 任务 |
|---|---|---|
| `crates/game-core/src/menu_family.rs` | `ShellOption`、`set_shell_option`、`OptionList::{with_shell_rows, toggle_shell_row, shell_rows}`、`len()` 计总行、绘制行助手 | T1 |
| `crates/game-core/src/paint.rs` | BootMachine `shell_options` 字段 + 标题侧设置页（入口 / 切换 / 环绕）；`skip_boot_splash` + `finish_boot_splash` | T2、T3 |
| `crates/game-core/src/scene.rs` | GameScene `shell_options` 字段 + 游戏内设置页（入口 / 切换 / 环绕） | T2 |
| `shell/common/src/handoff.rs` | 正向（新游戏 / 读档）与反向（回标题）携带 `shell_options` | T4 |
| `shell/common/src/settings.rs` | `AppSettings` 三个新键 + 布尔坏值取默认 + 写全部键 | T5 |
| `shell/win/src/main.rs` | `persist_settings_if_changed` 保留文件里的新键（`..上次值`） | T5 |
| `shell/android/src/runtime.rs` | `PREF_*`、`shell_rows`、`apply_app_settings` / `app_settings` / `open_app_settings` / 每拍落盘；T7 加 `key_feedback` | T6、T7 |
| `shell/android/src/native.rs` | 启动 `open_app_settings`；`handle_input` 反馈取 `key_feedback()` | T6、T7 |
| `shell/android/src/feedback.rs` | `cue_for(presses, haptic, click)` / `emit` + 头注策略表更正 | T7 |
| `shell/android/tests/app_settings.rs` | Android 设置宿主钉（新文件） | T6 |
| `shell/android/tests/feedback.rs` | 策略测试改写（行为变更）+ 运行时读数测试 | T7 |
| `docs/spec/stage-b-deviations.md`、`shell/android/README.md`、kairos | 登记与说明 | T8 |
| `data/status/baselines/rust-test-counts.json` | ratchet（工具生成，单独 commit） | T9 |

---

### Task 1: core `OptionList` 壳层附加行

**Files:** Modify `crates/game-core/src/menu_family.rs`（`OptionList` 定义 ~1290、`impl OptionList` ~1326-1430；文件尾新增测试模块）。

- [ ] **Step 1: 改前特征测试（重构不变量）**——在 HEAD 实现上先钉零附加行的 op 流摘要。

```rust
#[cfg(test)]
mod shell_row_tests {
    //! 【STAGE-B-006】壳层附加设置行（设计件 §2.1）。
    use super::*;
    use crate::layout::Layout;
    use crate::render::FontMetrics;

    fn ui() -> Vec<ArgbImage> {
        (0..25).map(|_| ArgbImage::from_rgb(&vec![0xFF202020; 24 * 24], 24, 24, true)).collect()
    }

    fn ops(list: &OptionList, layout: &Layout) -> Vec<String> {
        let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
        let mut g = SoftGraphics::new(&mut img);
        g.set_font(Some(crate::paint::paint_font()));
        list.paint(&mut g, &ui(), &crate::paint::paint_font(), layout);
        g.ops.clone()
    }

    fn digest(v: &[String]) -> String {
        game_platform::hash::sha256_hex(v.join("\n").as_bytes())[..32].to_string()
    }

    /// 零附加行 ⇒ op 流与改前逐字相同。期望 = 改前（HEAD ba7e92e）实现实测摘要——重构不变量，不是被测自证。
    #[test]
    fn zero_shell_rows_paint_matches_pre_change_ops() {
        let cases = [
            (true, false, 0, Layout::classic(), "<HEAD 实测>"),
            (false, true, 1, Layout::classic(), "<HEAD 实测>"),
            (true, true, 1, Layout::modern(12, 16), "<HEAD 实测>"),
        ];
        for (sound, minimap, hl, layout, expect) in cases {
            let mut list = OptionList::new(sound, minimap);
            list.highlight = hl;
            assert_eq!(digest(&ops(&list, &layout)), expect, "sound={sound} minimap={minimap} hl={hl}");
        }
    }
}
```

先以占位期望跑一次取实测摘要，填回后在 HEAD 上 GREEN（证明钉的是改前行为）。

- [ ] **Step 2: 写 RED 测试**（同模块追加）：

```rust
    fn rows() -> Vec<ShellOption> {
        [(1u8, "开屏画面", false), (2, "按键震动", true), (3, "按键音", true)]
            .into_iter()
            .map(|(id, l, checked)| ShellOption { id, label: l.encode_utf16().collect(), checked })
            .collect()
    }

    #[test]
    fn with_no_shell_rows_equals_new() {
        for layout in [Layout::classic(), Layout::modern(12, 16)] {
            let a = OptionList::new(true, false);
            let b = OptionList::with_shell_rows(true, false, &[]);
            assert_eq!(ops(&a, &layout), ops(&b, &layout));
            assert_eq!(b.len(), 2);
        }
    }

    /// 附加行几何 = 原行同式：盒高 = 32 + 5×行高；盒宽 = 最宽标签 + 80；行 i 顶 = 盒顶 + 16 + 行高×i；
    /// 每行标签 / 勾选框与原行同式（a.java:2959-2990 / 9911-9926）。
    #[test]
    fn shell_rows_extend_box_rows_and_paint() {
        let font = crate::paint::paint_font();
        let layout = Layout::modern(12, 16);
        let mut list = OptionList::with_shell_rows(true, false, &rows());
        assert_eq!(list.len(), 5);
        let row_h = font.height() + 8;
        let box_h = 32 + 5 * row_h;
        let top = (layout.screen_h - box_h) / 2 + 16;
        for i in 0..5 {
            assert_eq!(list.row_band(i, &layout), (top + row_h * i as i32, row_h), "行 {i}");
        }
        let widest = ["声音", "小地图", "开屏画面", "按键震动", "按键音"]
            .iter()
            .map(|s| font.string_width(&s.encode_utf16().collect::<Vec<u16>>()))
            .max()
            .unwrap();
        let box_w = widest + 80;
        list.highlight = 4;
        let got = ops(&list, &layout);
        let cx = ((layout.screen_w - box_w - 22) >> 1) + 11;
        assert!(got.contains(&format!("fillRect({cx},{},{box_w},{row_h})", top + row_h * 4)), "高亮条在第 5 行、宽 = 盒宽");
        for (i, (label, checked)) in [("开屏画面", false), ("按键震动", true), ("按键音", true)].into_iter().enumerate() {
            let u: Vec<u16> = label.encode_utf16().collect();
            let w = font.string_width(&u);
            let lx = (layout.screen_w - w - 16) >> 1;
            let ry = top + row_h * (i as i32 + 2);
            let at = got.iter().position(|o| o.starts_with("drawString(") && o.contains(&format!(",{lx},{},0)", ry + 4)))
                .unwrap_or_else(|| panic!("{label} 标签"));
            let clip = format!("setClip({},{},12,10)", lx + 2 + w, ry + ((font.height() - 2) >> 1));
            assert_eq!(got[at + 2], clip, "{label} 勾选框位置");
            let sy = if checked { 0 } else { 12 };
            assert!(got[at + 3].starts_with(&format!("drawImage(img24x24,{},{},", lx + 2 + w, ry + ((font.height() - 2) >> 1) - sy)), "{label} 勾选态");
        }
    }

    #[test]
    fn toggle_shell_row_flips_only_shell_rows() {
        let mut list = OptionList::with_shell_rows(true, false, &rows());
        assert_eq!(list.toggle_shell_row(0), None, "声音行不归附加行");
        assert_eq!(list.toggle_shell_row(1), None, "小地图行不归附加行");
        assert_eq!(list.checked, vec![true, false], "原两行勾选不动");
        assert_eq!(list.toggle_shell_row(2), Some((1, true)));
        assert_eq!(list.toggle_shell_row(4), Some((3, false)));
        assert_eq!(list.toggle_shell_row(5), None, "越界");
        let r = list.shell_rows();
        assert_eq!((r[0].checked, r[1].checked, r[2].checked), (true, true, false));
        list.set_highlight(4);
        assert_eq!(list.highlight_index(), 4, "附加行可选中");
        list.set_highlight(5);
        assert_eq!(list.highlight_index(), 4, "越界不选");
    }

    #[test]
    fn set_shell_option_writes_by_id() {
        let mut r = rows();
        set_shell_option(&mut r, 2, false);
        set_shell_option(&mut r, 9, true);
        assert_eq!(r.iter().map(|o| o.checked).collect::<Vec<_>>(), vec![false, false, true]);
    }
```

（`drawImage` op 的图像标签格式以 render.rs 实际输出为准，Step 2 跑 RED 时核对并按实际格式写断言。）

Run: `cargo test --offline -p game-core --lib shell_row_tests` → 预期编译失败：`ShellOption` / `with_shell_rows` / `toggle_shell_row` 不存在。

- [ ] **Step 3: 实现**

```rust
/// 【STAGE-B-006】壳层附加设置行：core 只显示 / 切换 / 交接，不解释含义（`id` 由壳定义）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellOption {
    pub id: u8,
    /// 行文字（UTF-16，字体绘制）。
    pub label: Vec<u16>,
    pub checked: bool,
}

/// 【STAGE-B-006】按 id 回写机器侧附加行勾选（两处设置页入口共用）。
pub fn set_shell_option(rows: &mut [ShellOption], id: u8, checked: bool) {
    for r in rows.iter_mut().filter(|r| r.id == id) {
        r.checked = checked;
    }
}
```

`OptionList` 加私有字段 `shell: Vec<ShellOption>`；`new` = `with_shell_rows(sound, minimap, &[])`；`with_shell_rows` 先
`add(0)` / `add(1)` 再逐行 `grow_box(&row.label)` + push；`add` 的非首行半抽成 `grow_box(label)`（公式不变）；
`len()` = 原行 + 附加行；`set_highlight` 用 `len()`；私有 `row(i) -> (&[u16], bool)`；`paint` 行循环改 `0..self.len()` 取
`row(i)`；`toggle_shell_row(i)` = `shell.get_mut(i.checked_sub(labels.len())?)` 翻转返回 `(id, checked)`；`shell_rows()` 只读。

- [ ] **Step 4: GREEN** `cargo test --offline -p game-core --lib` 全绿（含 `option_tests` / `modern_center_tests` 与 Step 1 特征测试）。
- [ ] **Step 5: 对抗复核** → `target/verify/review_T1.md`，处理结论。
- [ ] **Step 6: commit** `feat(core): STAGE-B-006 设置页壳层附加行（OptionList 显示 / 切换；零附加行 op 流不变）`

### Task 2: 两台机器的 `shell_options` + 设置页接线

**Files:** Modify `crates/game-core/src/paint.rs`（BootMachine 字段 / `new` / `enter_menu_mode` About 臂 / `tick` About 臂 + tests）、
`crates/game-core/src/scene.rs`（GameScene 字段 / 构造 / `enter_menu16` / `tick_menu16` + tests）。

- [ ] **Step 1: RED 测试**
  - paint.rs `tests`：`title_settings_shell_rows_toggle_only_shell_options`——`boot_state()` 置 3 行 `shell_options`，
    `mode = About` + `enter_menu_mode()` ⇒ 5 行；`tick(-1)` 从 0 环绕到 4、`tick(-2)` 回 0；高亮移到 2，记下
    `bgm.sound_enabled` / `global_snapshot()`，`tick(-5)` ⇒ `shell_options[0].checked` 翻转、面板该行同步翻转，
    声音 / 快照不变、`take_global_dirty() == false`；高亮 0 再 `tick(-5)` ⇒ 声音翻转且置脏（原行为不变）。
  - scene.rs `pointer_select_tests`（或新模块）：`in_game_settings_shell_rows_toggle_only_shell_options`——`scene((2,2),&[])`
    + modern 布局 + 3 行 `shell_options`，`enter_menu16()` + `game_mode = About` ⇒ 5 行；`key_value=-1; tick_menu16()` 环绕到 4；
    高亮 3，`key_value=-5; tick_menu16()` ⇒ `shell_options[1].checked` 翻转，`bgm.sound_enabled` / `minimap_enabled` /
    `minimap.is_some()` / `global_snapshot()` / `global_dirty` 不变；`pointer_hit` 第 4 行中点 ⇒ `Row{index:3, confirm_key:-5}`。
  - 预期 RED：`shell_options` 字段不存在（编译失败）。
- [ ] **Step 2: 实现**：两机加 `pub shell_options: Vec<crate::menu_family::ShellOption>`（默认空）；入口改
  `OptionList::with_shell_rows(…, &self.shell_options)`；切换臂先 `if let Some((id, v)) = opts.toggle_shell_row(i)` ⇒
  `set_shell_option(&mut self.shell_options, id, v)`，`else` 原分支一字不动；`-2/-1` 环绕改 `len()`。注释 `// STAGE-B-006`。
- [ ] **Step 3: GREEN** `cargo test --offline -p game-core`（lib + 集成：classic 夹具 / boot 回放 / 帧测试）。
- [ ] **Step 4: 对抗复核** → `review_T2.md`。
- [ ] **Step 5: commit** `feat(core): STAGE-B-006 两处设置页显示并切换壳层附加行（不碰声音 / 小地图 / SKY_WAR）`

### Task 3: `skip_boot_splash`

**Files:** Modify `crates/game-core/src/paint.rs`（`tick_mode0` 收尾抽 `finish_boot_splash`；新增 `skip_boot_splash`；tests）。

- [ ] **Step 1: RED 测试**（paint.rs `tests`）：
  - `skip_boot_splash_equals_natural_finish`：两台 `boot_state()` 都 `apply_layout(Layout::modern(12,16))`；甲 `tick(0)` × 68，
    乙 `skip_boot_splash() == true`；比较 mode（MainMenu）、`title_cursor() == Some(0)`、`bgm_request()`、
    `bgm.sound_enabled`、`frame_interval_ms`（75）、`finished`、`softkeys`、`logo.is_some()`、`phase`、`counter`、`image.is_none()`。
    再对声音关的一对（`bgm.sound_enabled=false`）重复：两者 `bgm_request() == None`。
  - `skip_boot_splash_refused_in_classic_or_after_first_tick`：classic `boot_state()` ⇒ false 且 mode 仍 Boot / phase 0；
    modern 先 `tick(0)` ⇒ false 且 `counter == 1`；非 Boot 态 ⇒ false。
  - `exit_after_skip_paints_logo_anim_without_panic`：modern 跳过 ⇒ 标题光标 5 + `tick(-5)` ⇒ mode LogoAnim ⇒ `paint` 不 panic。
  - 预期 RED：`skip_boot_splash` 不存在。
- [ ] **Step 2: 实现**：`finish_boot_splash()` = 原 `tick_mode0` `else` 体，logo 取得改 `get_or_insert_with(LogoAnim::new)`；
  `skip_boot_splash()` 守卫 `mode==Boot && phase==0 && counter==0 && !layout.boot_sound_prompt`，置 `phase=2` 后调
  `finish_boot_splash()`，返回 true。
- [ ] **Step 3: GREEN** `cargo test --offline -p game-core`（boot 回放 / classic 夹具锁 `tick_mode0` 抽函数不变）。
- [ ] **Step 4: 对抗复核** → `review_T3.md`。
- [ ] **Step 5: commit** `feat(core): STAGE-B-006 BootMachine::skip_boot_splash（modern、首拍前；与自然播完共用收尾）`

### Task 4: 交接携带 `shell_options`

**Files:** Modify `shell/common/src/handoff.rs`（`new_game_scene` / `slot_scene` / `return_to_title` + tests）。

- [ ] **Step 1: RED**：`shell_options_ride_handoff_both_ways`——机器置 3 行；`new_game_scene` ⇒ 场景同 3 行；场景翻一行后
  `return_to_title` ⇒ 机器得到翻后的值；`slot_scene`（preset-floor1）⇒ 场景同机器值。预期断言失败（场景为空）。
- [ ] **Step 2: 实现**：两处正向 `sc.shell_options = machine.shell_options.clone()`；反向 `machine.shell_options = sc.shell_options.clone()`；
  `return_to_title` 头注补一句。
- [ ] **Step 3: GREEN** `cargo test --offline -p game-shell-common -p game-android -p game-desktop`。
- [ ] **Step 4: 对抗复核** → `review_T4.md`。
- [ ] **Step 5: commit** `feat(platform): STAGE-B-006 交接正反向携带壳层附加行`

### Task 5: `AppSettings` 新键 + Windows 保留新键

**Files:** Modify `shell/common/src/settings.rs`、`shell/win/src/main.rs`（`persist_settings_if_changed` + 新测试）。

- [ ] **Step 1: RED**（settings.rs）：`stage_b006_keys_defaults_roundtrip_and_bad_values`——默认 (false, true, true)；
  全键往返；旧文件只有两键 ⇒ 新键取默认；`show_splash=yes` / `key_haptic=0` / `key_click=` / `sound_enabled=off` ⇒ 各取默认。
  预期编译失败（字段不存在）。
- [ ] **Step 2: 实现**：三字段 + Default；`load` 布尔键统一 `set_bool`（只认 `true` / `false`，其余保留默认）；`save` 写 5 键。
  既有测试的结构体字面量补 `..AppSettings::default()`。Windows 字面量先改 `..Default::default()` 只为编译通过。
- [ ] **Step 3: RED（Windows）**：`persist_keeps_stage_b006_keys`——文件预写 `show_splash=true,key_haptic=false,key_click=false`，
  `apply_loaded_settings(load)`，进游戏后关声音、跑一拍 ⇒ 重读文件：声音关 ∧ 三新键仍是 (true,false,false)。
  预期断言失败（被默认值覆盖）。
- [ ] **Step 4: 实现**：`..self.settings_saved.unwrap_or_default()`。
- [ ] **Step 5: GREEN** `cargo test --offline -p game-shell-common -p game-desktop`（含 `settings_toggle_persists_next_tick`）。
- [ ] **Step 6: 对抗复核** → `review_T5.md`。
- [ ] **Step 7: commit** `feat(platform): STAGE-B-006 设置文件新增 开屏画面 / 按键震动 / 按键音 三键（坏值取默认；Windows 不改写）`

### Task 6: Android 设置接入（行定义 + 启动应用 + 每拍落盘）

**Files:** Modify `shell/android/src/runtime.rs`、`shell/android/src/native.rs`（`android_main` 一行接线）；Create `shell/android/tests/app_settings.rs`。

- [ ] **Step 1: RED**（`tests/app_settings.rs`，真资产 + 临时目录）：
  - `no_file_starts_at_main_menu_with_default_rows`：`open_app_settings(无文件)` ⇒ 首拍前 `machine.mode == MainMenu`、
    `frame_interval_ms() == 75`、三行 (false,true,true)；一拍后 `bgm_request() == Some(2)`、`title_mode()`；未写文件。
  - `splash_on_plays_boot_screens`：文件 `show_splash=true` ⇒ 首拍后仍 Boot、`frame_interval_ms()==100`；68 拍内到主菜单。
  - `saved_sound_off_restored_before_title`：文件 `sound_enabled=false` ⇒ 跳过开屏后 `bgm_request() == None`、`!sound_enabled()`。
  - `title_settings_toggle_persists_same_step`：标题光标移到"设置"（kind 2）确认 ⇒ About 5 行；高亮 2 ⇒ Press(-5) 那一拍结束时
    文件 `show_splash=true`；用该文件新建 Runtime ⇒ 首拍后仍 Boot。
  - `in_game_toggle_persists_and_survives_return_to_title`：`reach_map` 同款进 Map，开游戏内设置（菜单 kind 4），5 行；
    高亮 3（按键震动）Press(-5) ⇒ 同拍文件 `key_haptic=false`；退出回标题 ⇒ `app_settings().key_haptic == false`、文件不再变。
  - 预期 RED：`open_app_settings` / `app_settings` 不存在。
- [ ] **Step 2: 实现**（runtime.rs）：`PREF_SPLASH=1 / PREF_HAPTIC=2 / PREF_CLICK=3`；`shell_rows(&AppSettings)`；
  `Runtime` 加 `settings_file: Option<(PathBuf, AppSettings)>`；`apply_app_settings`（先声音对、再三行、`!show_splash` 才
  `skip_boot_splash()`）；`app_settings()`（活动机读，缺行取默认）；`open_app_settings(path)`；`step` 末
  `persist_app_settings_if_changed()`。native.rs：`let mut rt = …; rt.open_app_settings(data_path.join("app_settings.cfg"));`。
- [ ] **Step 3: GREEN** `cargo test --offline -p game-android`；`cargo ndk … build -p game-android` 编译通过（native 接线只在 android 目标编译）。
- [ ] **Step 4: 对抗复核** → `review_T6.md`。
- [ ] **Step 5: commit** `feat(android): STAGE-B-006 设置文件接入（启动应用 / 开屏可跳过 / 同拍落盘；声音跨启动保存）`

### Task 7: 按键反馈策略（按键音独立于声音）

**Files:** Modify `shell/android/src/feedback.rs`、`shell/android/src/runtime.rs`（`key_feedback`）、`shell/android/src/native.rs`（`handle_input`）、
`shell/android/tests/feedback.rs`。

- [ ] **Step 1: RED**：改写 `cue_policy_haptic_always_click_follows_game_sound_switch` 为
  `cue_policy_haptic_and_click_follow_their_own_switches`（四象限 + 无按下 + 两项全关无反馈 + 多次按下合并）；新增
  `runtime_key_feedback_reads_active_machine`（开场链读 BootMachine 行、交接后读 GameScene 行、无行时 (true,true)）。
  预期编译失败（`emit` 参数个数 / `key_feedback` 不存在）。
- [ ] **Step 2: 实现**：`cue_for(presses, haptic, click)` = `presses>0 ∧ (haptic ∨ click)` ⇒ `Some(Cue{haptic, click})`；`emit` 同参；
  `Runtime::key_feedback()`；`handle_input` 改 `let (haptic, click) = st.rt.key_feedback(); emit(fb, presses, haptic, click)`；
  feedback.rs 头注策略表"按键音"行原地更正（日期注）；`Runtime::sound_enabled` 文档去掉"按键音据此门控"。
- [ ] **Step 3: GREEN** `cargo test --offline -p game-android`；ndk 编译通过。
- [ ] **Step 4: 对抗复核** → `review_T7.md`。
- [ ] **Step 5: commit** `feat(android): STAGE-B-006 按键震动 / 按键音各看各的开关（按键音不再跟随声音）`

### Task 8: 文档登记

- [ ] `docs/spec/stage-b-deviations.md` 新增 STAGE-B-006 节（偏离点 / 用户原话 / 门控 / 金样影响 = 无 / 改写的测试名单）；
  STAGE-B-001 节加注"Android 已接入设置文件"。
- [ ] `shell/android/README.md` 平台映射 / 设置说明原地更正（日期注）。
- [ ] 设计件 §4 加一行实施注（落盘比较放进 Runtime）。
- [ ] kairos 日志。
- [ ] `python3 gates/cli.py check` 全绿后 commit `docs(docs): STAGE-B-006 登记 + README 更正 + kairos`。

### Task 9: ratchet

- [ ] `python3 gates/cli.py ratchet-update` ⇒ 只增；单独 commit `gate(baselines): ratchet 入账（STAGE-B-006 …）`。

### 实机验证（手机 `c362a15b` 连上后）

- [ ] 重建 so + APK，`adb -s c362a15b install -r`；先确认焦点 = `com.jjxy.game` 再注入（暗屏先唤醒，首个注入会被吞）。
- [ ] 清数据后首启：第一帧即主菜单（截屏 + logcat 首拍 mode）。
- [ ] 标题设置页 5 行截屏；游戏内设置页 5 行截屏。
- [ ] 打开"开屏画面"⇒ 强停重启出现三屏；关回 ⇒ 重启直达主菜单。
- [ ] "按键震动"关 ⇒ 本方注入虚拟键按下时刻 `dumpsys vibrator_manager` 无新记录；"按键音"关 ⇒ `dumpsys audio` 无新音效事件；
  另一项仍有。"声音"关 + "按键音"开 ⇒ 无 BGM 但有按键音。
- [ ] 关声音后重启仍关。
- [ ] BUG-038~041 回归抽查（触屏点按 / 寻路收尾 / 回标题声音）。
