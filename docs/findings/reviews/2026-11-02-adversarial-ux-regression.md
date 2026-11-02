# 对抗复核 C — UX 批（cbcb647+9c698e1）+ 警告批（02b7413）回归面

> 角色：remake-verifier 纪律 · 对抗复核员 C
> 日期：2026-11-02
> 对象：`cbcb647`（UX 批一）+ `9c698e1`（UX 批二）+ `02b7413`（警告清零批）
> 纪律：全程亲做；不改产品码（worktree 破坏性还原验证后复绿收尾）；未跑不称绿。
> 证据分级：A=实跑输出/字节 diff/行号；B=码读推断。每节附实跑报数。

## 0. 攻击点总表（红/黄/绿）

| # | 攻击点 | 判定 | 实跑计数/证据 |
|---|---|---|---|
| 1 | 全量实跑（workspace test + gates + oracle 金样） | 🟢（1 计数黄） | 909 过/0 败/7 忽略 exit 0；gates 复跑 12/12（首跑红=并行探针）；oracle 74 过/3 忽略；47/47 场景 A==B==C |
| 2 | classic 零漂移（diff=仅 additive？35 帧哈希旁证） | 🟢（措辞黄） | diff 分类：3 处有意变更限壳行/触屏面；preset_cases 35/35 frame_sha + boot_replay 2/2 零漂移 |
| 3 | TDD 红抽查（抽 2，worktree 复现一红） | 🟢（provenance 黄） | 双红复现实锤（203px 缝+隐藏谓词）→ 还原复绿；cbcb647 悬空=红证据断链 |
| 4 | 警告批回归（挂门死码处置 + 注记） | 🟢（1 残留黄） | b×10/c×2/a×0 对账 ✓；menu_family 24/24+intro 4/4 绿；岛已挂门、allow 注记在但未撤 |
| 5 | ratchet（915 基线无回退？新增二进制入账） | 🟢（口径黄） | total=917(基线906) 无回退；DROPS=∅ NEW=∅（入账清单=空）；"915"仓库无出处 |
| 6 | 五链回归（menu_hit/tap/滑动/BACK/UNDO） | 🟢 | 五链单测在+绿（全量实跑旁证，逐名清单见 §6） |

**跨批复核员 A 交叉引用**：UX 批件1「隐藏行值保留」= 🔴 真红（§7），总判因此为红。

## 1. 全量实跑

**1a. `cargo test --workspace --release`（2026-11-02 12:02-12:10 两跑）→ exit 0**：

| 套件 | 实跑 | 声称 | 对账 |
|---|---|---|---|
| game-core lib | **319 passed + 1 ignored**（`probe_bottom_region_pixel_metrics` 忽略） | core 318 | 🟡 **+1 口径差**（见下） |
| game-shell-common | **45 passed** | common 45 | ✓ 精确 |
| game-android 全家（20 二进制） | **180 passed**（含 app_settings 17/17） | android 180 | ✓ 精确 |
| 全仓合计 | 909 passed / 0 failed / 7 ignored | — | exit 0 |

- 🟡 **core 318≠319**：`#[test]` 源计数考古 `git grep -c "#\[test\]"` 在 02b7413↔9c698e1 间**无变化**（324 文本命中含注释）；批二未增测，声称"318"对其自身树即失真 1。非测试丢失（ratchet 无回退+全绿），判**计数口径漂移/过时声称**。

**1b. `gates check`**：首跑 **PASS=11 FAIL=1**——红源 `rust-tests`：`tmp_adv_hidden_row_value_survives_mode_roundtrip`（app_settings 17+1）。该测为**并行复核员 A 的 DV-C 临时探针**（其文档 §4 自述"临时钉，跑毕已删"），恰被我在共享 worktree 撞见（12:12）；**非批回归**。探针撤后复跑结果：（待回填：见文末补记）
- 计数旁证：探针在场时 app_settings=17 过+1 败；我 12:02 全量跑时探针未生（17/17）；作者已还原。
- ⚠️ 纪律提示：共享 worktree 并行破坏性验证互相污染（`gotcha_parallel-commit-shared-index` 同域），gates 12/12 声称在竞态下需带时点。

**1c. `cargo test -p game-oracle --release` → exit 0**：74 passed / 3 ignored
（fixture_frames 31 / preset_cases 35 / boot_replay 2 / gameplay_floor1 2+3ign / l2_prologue 1 / probe_floor2 1 / walk_replay_floor1 2）。

**1d. 47 场景三方对拍**（`python3 reference/oracle/run.py --scenarios`，A原始/B重映射/C javac trace sha 三方一致）：（待回填：见文末补记）。口径核：`reference/oracle/_diff/scenarios.md` 数据行=**47** ✓（25938b7 +4 行=fx-item25a 场景行入账，43→47）。

## 2. classic 零漂移

**2a. diff=仅 additive？——不严格成立（3 处有意变更），但变更域收敛、classic 游戏帧零漂移**。
`git diff 30ff91c..HEAD -- crates/game-core` 四文件逐 hunk 分类（A 级）：

| 文件 | hunk | 分类 |
|---|---|---|
| intro.rs | +3（FontMetrics/import 挂 #[cfg(test)]） | 纯挂门 ✓ |
| menu_family.rs | 挂门×10 + `ShellOption.value_text` 字段 | 加法 ✓ |
| menu_family.rs | paint 绘制改 `if let Some(v)=row_value(i)` 分支 | 半加法：**else 臂=旧码原文**（value_text=None 行逐字不变）；值行仅新行（操控方式）触发 |
| menu_family.rs | 两段式 API/row_hit/retry_hint_text/HighlightAnchor + touch_ux_tests | 加法 ✓ |
| paint.rs + scene.rs | `replace_shell_rows` 锚语义：`h.clamp(0,len-1)` → `highlight_anchor()/resolve_highlight` | **有意行为变更**（件1 用户裁定；t3 钉+复核员 A DV-B 红敏感实证） |
| paint.rs/scene.rs | 测试构造器 `+value_text: None` | 测试面 ✓ |

- "classic 路径"界定：原版像素保真游戏帧面。壳行设置面（classic/触屏共用）的锚跟随/值行为**批交付物本体**，非回归漂移。
- **红线核**：`git diff 30ff91c..HEAD -- original data/golden` = **0 行** ✓；`reference/oracle/_diff/` 仅生成物报告同步（25938b7 自述"非语义变更"）；`reference/seed` 经 `reference-seed-integrity` 门禁绿（8457 行 sha 对照）。

**2b. 金样 35 帧哈希实跑旁证（A 级）**：口径= `preset_cases(test)` 35 场景逐 tick `frame_sha` 全帧断言（2026-10-23 复核 C 同口径）——本次实跑 **35/35 绿**（11.6s）；加 `boot_replay` **2/2**（l0/l1 轮播 T1-31+logo T32-67+询问 T68-70 逐 tick 帧哈希 vs Java 参照，classic 时间轴）、fixture_frames 31、render_golden 16、density_draw 16、hd_visual 6 全绿 ⇒ classic 游戏帧面**零漂移实证**。

## 3. TDD 红抽查（抽 2）

**3a. 红三件套存在性（抽 2 = 两批各一）**：
- 9c698e1（203px 双算缝）：红测试先行（`controls_ui::same_source_tests::toolbar_cell_hit_label_same_source`，203px 不可整除宽）→ 真红（"边界 Mid(0)/Mid(1) 错格"）→ 单源化绿。钉测在 HEAD ✓（全量实跑 `same_source_tests::toolbar_cell_hit_label_same_source ... ok`）。
- cbcb647（值行/显隐/两段式/微提示）：红测试先行 4 钉 + 编译红（E0560/E0599/E0063×5）+ 自查红 + 行为红（t3 越界钳位 vs 锚跟随）。4 钉在 HEAD ✓：`menu_family::touch_ux_tests`（value_row_paints_value_text_not_checkbox / hidden_rows_absent_from_list_len / slot_two_stage_select_then_execute / slot_selected_row_shows_retry_hint）**4/4 绿**；t3 钉 `paint.rs:1076 t3_replace_shell_rows_boot_and_scene` 在 ✓。

**3b. worktree 破坏性复现红（双红均做，超要求）**——`git worktree add /tmp/adv_c_wt 9c698e1`：

红①203px 双算缝（还原旧双算式）：
```
改：hit_toolbar 单源循环 → 旧式 ((x-mx)*n/mw).clamp(...)
cargo test -p game-shell-common toolbar_cell_hit_label_same_source
  → FAILED shell/common/src/controls_ui.rs:147 "界线归属右格"
    left: Some(Mid(0)) right: Some(Mid(1))   ← 与红实录逐字吻合
还原：git checkout → 1 passed 绿
```

红②隐藏谓词（DV-A 同式）：
```
改：runtime.rs:42  if s.control_mode == ControlMode::Pad { → if true {
cargo test -p game-android --lib touch_mode_hides
  → FAILED runtime::touch_ux_rows_tests::touch_mode_hides_key_rows_auto_battle_always
cargo test -p game-android --test app_settings control_mode_toggle
  → FAILED app_settings.rs:273 "触屏重建：原两行 + 壳 3 行（按键两行隐藏——件1 裁定）"
还原：git checkout → 17/17 绿（worktree 补链 assets/raw 后复跑）
```
⇒ 红钉**红敏感**成立，"真红实录"可独立复现。✅

**3c. 🟡 provenance 断链（A 级，reflog 实证）**：
```
9c698e1 HEAD@{0}: commit（批二）
eafaf0e HEAD@{1}: commit（kb gotcha：commit --only 扫他人 WIP）
25938b7 HEAD@{2}: commit（oracle 生成物同步）
02b7413 HEAD@{3}: reset: moving to HEAD~1   ← cbcb647 被剔出分支
cbcb647 HEAD@{4}: commit（批一）
```
- `git merge-base --is-ancestor cbcb647 HEAD` = **false**；`git branch -a --contains cbcb647` = 空；`git log --all` 不可达 ⇒ **悬空 commit**，批一红证据 message（三件套实录）仅存 reflog/对象库，`git log` 检索不到。
- 内容去向（实证）：`git diff cbcb647 HEAD` 中 app_settings/scene/handoff **逐字节一致**=批一内容完整在 HEAD；menu_family 四钉+value_text 面被 `--only` 扫进 **02b7413**（其 message"写域=menu_family.rs+intro.rs"**未披露扫入 WIP**，误导）；补集随批二入史。勘误只存在于悬空 message + kb(eafaf0e) + kairos。
- 影响：回归面内容无损（全绿实证），但 TDD 红证据与归属在**分支历史不可自洽**；gc 失 reflog 后证据灭失风险。

## 4. 警告批回归（02b7413）

**4a. b×10/c×2/a×0 复核（A 级，diff+HEAD 逐项核）**：
- **判定 b·挂门 ×10** ✓：menu_family 9 项（FILL/WHITE/FRAME_Y/WEAPON_X/ARMOR_X/px/assert_frame_icon/assert_frame_material_name/assert_frame_empty）+ intro.rs FontMetrics——逐项 `#[cfg(test)]`（diff 中 `+#[cfg(test)]`×14 含 import 收窄 4）；FontMetrics"首判真死删→测试 E0599→改判 b"证伪留痕在 message ✓。
- **判定 c·测试引用中 ×2** ✓：ICON_COLORS/synthetic_sptarm 保留 + `#[allow(dead_code)]`×2 + 注记（menu_family.rs:1130/1153）。
- **判定 a·真死 ×0** ✓：diff 删除行仅 `use` 收窄 1 处 + UX value_text 重构移位 3 行（勾叉绘制移入 else 臂，原文保留）——无活码误删。

**4b. menu_family/intro 测试全绿（A 级实跑）**：
- `cargo test -p game-core --lib menu_family::` → **24 passed**（slot_equip_tests/modern_center_tests/touch_ux_tests 全含）
- `cargo test -p game-core --lib intro::` → **4 passed**
- 全量 core lib 319/319 绿。

**4c. scene.rs 测试岛与 allow 注记**：
- 岛**已挂门** ✓：`scene.rs:7907 #[cfg(test)] mod slot_equip_scene_tests`（UX 批承诺兑现）。
- allow 压警注记**在** ✓：menu_family.rs:1130/1153 "scene.rs 归 UX 批，岛挂门后本 allow 可撤"。
- 🟡 **残留**：岛既已挂门，注记承诺的"撤 allow 改挂门"未执行——`#[allow(dead_code)]`×2 仍挂（临时压警跨批滞留）。不阻断（警告面非行为面），建议随批三收口。

## 5. ratchet

**5a. 无回退 ✓（A 级）**：`gates check` `test-ratchet — total=917(基线906) 无回退`（917 含并行探针 +1，扣之=916）。逐二进制独立对账（实跑 vs `rust-test-counts.json`）：
- 63/63 二进制双方齐全；**DROPS=∅**（无消失/无减量）；**NEW=∅**。
- 上浮 4 项（地板滞后=合法，未刷基线）：game_core(lib) 314→320、game_shell_common(lib) 44→45、game_android(lib) 6→8、mov_route_scene(test) 4→5。

**5b. 新增二进制入账清单 = 空**（无未入账新二进制；gate 的 `新增二进制待 ratchet-update` 后缀未出现，独立对账一致）。

**5c. 🟡 "915 基线"口径无出处**：全仓 grep（md/json/py）无 915 基线出处；实测基线 SUM=**906**、实跑=**916/917**。"915"判为转述口径漂移，以 rust-test-counts.json 为准。

## 6. 五链回归（单测在 + 绿，A 级实跑旁证）

| 链 | 单测（全绿） |
|---|---|
| menu_hit | native_parse.rs `scene_tap_walks_on_map_and_confirms_rows_in_menus`/`title_button_hit_diff_smoke`/`title_touch_routes_tap_and_swipe_through_wiring`/`overlay_rects_match_hit_tables`（36/36）+ app_settings `toolbar_hit_executes_action` + tap_routing 14/14 |
| 两段式 tap | touch_ux_tests `slot_two_stage_select_then_execute`（tap1 选中/tap2 同行执行/换行不执行/打断）+ `slot_selected_row_shows_retry_hint` + paint `t3_replace_shell_rows_boot_and_scene` |
| 滑动 | touch.rs `swipe_maps_to_facing_and_one_step_per_gesture`/`swipe_wall_stop_no_repeat`/`t4_gesture_tap_swipe_drag_classification` + app_settings `vp_gesture_swipe_and_observe_drag` |
| BACK | input_mapping `back_maps_to_minus7_and_consumed`/`back_repeat_dropped_but_still_consumed`/`hidden_drops_input_but_back_still_consumed` + exit_chain 5/5（back_at_title/back_during_boot_logo/back_outside_root/menu_exit/title_menu_exit）+ exit_dialog `back_on_map_requests_dialog_instead_of_menu` |
| UNDO | undo_journal 11/11 + undo_ui 6（含 bug045 三钉）+ controls_acceptance `accept_undo_reverts_position` + walk_route `undo_midway_then_new_walk_keeps_lifo` + touch `undo_zone_hit_corners`/`tap_target_maps_cells_undo_and_outside` + app_settings `t8_undo_zone_removed_on_android` |

## 7. 跨批红交叉引用（复核员 A §4，本复核独立撞证）

**🔴 UX 批件1「隐藏行值保留」= 真红（用户裁定条款未满足）**：
- 机制（B 级码读复核吻合）：`runtime.rs:208-209` `row_checked(rows, PREF_HAPTIC, d.key_haptic)`——触屏隐藏 PREF_HAPTIC/PREF_CLICK 行缺席 ⇒ 落**默认 true** 而非保留原值；`persist_app_settings_if_changed` 同拍落盘覆盖用户原值；回切摇杆读回默认值=丢失。对照 `auto_battle` 走 `rt_auto_battle` 保留字段=保值，同函数两种口径。
- 独立撞证（A 级）：其 DV-C 临时探针 `tmp_adv_hidden_row_value_survives_mode_roundtrip` 红被我 gates check 实跑（12:12）当场记录（17 过+1 败）；文档 §4 双红实录（落盘/快照）与机制一致。
- 处理：**回 porter**——补 `rt_key_haptic/rt_key_click` 保留字段（复核员 A 同路由）。
- **修复追踪（复核期间落地）**：`c88a575 fix(core+android): 复核A 红-1——隐藏行保值（隐藏≠丢值）+黄三连`
  已在 9c698e1 之后提交（另有批三 `8a7c99f` 落地）；本红的修复验收归复核员 A 复测，
  本复核的红判定与双红复现均锚定批状态 9c698e1（worktree 实证，已清理）。

## 8. 总判

**🔴 红（带病）**——UX 批件1 存在真红缺陷「隐藏行值丢失」（裁定条款未满足），必须回 porter 修复后本批才算收口；（**修复追踪**：`c88a575` 已落修复，验收归复核员 A 复测）除此之外回归面**零回退**：
- 全量实跑 909/0/7 exit 0（core 319+1ign / common 45 / android 180）；
- classic 游戏帧零漂移（preset 35/35 frame_sha + boot 2/2 + 全金样绿；original/data-golden 零触碰）；
- TDD 红三件套在+双红 worktree 可复现（203px 缝 + 隐藏谓词）→ 还原复绿；
- 警告批 b×10/c×2/a×0 处置属实，menu_family 24/24+intro 4/4 绿；
- ratchet 无回退、无新增二进制待入账；五链单测在+绿。

**黄点 5**：①cbcb647 悬空=红证据/归属分支历史断链（02b7413 message 未披露扫入 WIP）；②core 计数口径 318≠319（声称过时）；③"915 基线"无出处（实 906/916）；④scene.rs 岛挂门后 allow 未按注记撤除；⑤复核员 A 黄点 1-3（按住态清零无钉测/plan==portrait 边界/resolve_highlight Shell 缺 id 未定义回退）转批三续审。

**修复路由**：红①（隐藏行值）→ porter；黄①→ lead 裁定是否重写历史/补勘误入 9c698e1 message 或独立 docs 记录；黄②③→ 声称口径更正；黄④→ 批三收口撤 allow。

---

## 补记（实跑回填区）

- gates check 复跑（探针撤后，12:4x）：**PASS=12 FAIL=0 SKIP=0**——12/12 声称可复现 ✓
  （首跑红唯一源=DV-C 竞态探针，坐实"非批回归"判定）。
- 47 场景 `run.py --scenarios`：**PASS（47/47 场景）A==B==C**，exit 0；报告再生成
  `git diff reference/oracle/_diff/` = **0 行**（确定性复跑零漂移，A 级）。
- 复核收尾时点注意：主工作区 HEAD 已前移（`c88a575` 隐藏行保值修复 + `8a7c99f` 批三），
  且有他人未提交 WIP（menu_family/scene/native/runtime）——本文全部实跑计数锚定
  9c698e1 树（12:02-12:4x 时点）；批三回归面不在本文范围。
