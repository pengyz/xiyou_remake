# 对抗复核 C（remake-verifier 纪律）——Android 触屏/渲染修复批回归面

- 日期：2026-10-23
- 复核员：对抗复核员 C（remake-verifier）
- 对象 commit 链：`50081d8`（输入传输层）→ `d80d96e`（menu_hit+音频装配）→ `1700cec`（清屏+触控框）
- 工作区：`/home/peng/workspace/xiyou_remake`
- 性质：攻击性回归复核（红/黄/绿 + 实跑证据），**不改产品码**（破坏性验证已还原，sha 复位实证）
- 方法：全量实跑对账 + 链级 git 考据 + strings/sha 工件实证 + worktree 红复现

> 【并发工作区声明】复核期间共享工作区有并发修改，两次污染本复核实跑，均已甄别：
> ①20:48–20:55 另一复核员做红复现（paint.rs 被临时注释清屏行→20:54:55 还原）——
> 首轮 gates check（≈20:50）rust-tests 假红（残留 4 px）由此而来，不可采信；
> ②20:59:26 lead 依本复核中途警报落 `bbda363`（误删两测复原）——测试面读数以
> 该时点后为准。本报告最终读数均带源码 sha256 前后钉入（见 §1d）。

## §1 全量实跑对账 —— 绿

### 1a. `cargo test --workspace --release` —— 全绿，逐套报数

47 套件：**591 passed / 0 failed / 4 ignored（含 ignored=595）**，零失败零漂移。

| 套件 | passed | failed | ignored | 套件 | passed | failed | ignored |
|---|---|---|---|---|---|---|---|
| game_android(lib) | 5 | 0 | 0 | game_data(lib) | 35 | 0 | 0 |
| input_mapping(test) | 16 | 0 | 0 | asset_roundtrip(test) | 2 | 0 | 0 |
| lifecycle(test) | 9 | 0 | 0 | decode_equivalence(test) | 3 | 0 | 0 |
| native_parse(test) | 26* | 0 | 0 | defs_golden(test) | 3 | 0 | 0 |
| storage_wiring(test) | 4 | 0 | 0 | defs_negative(test) | 16 | 0 | 0 |
| stubs_contract(test) | 11 | 0 | 0 | golden(test) | 9 | 0 | 0 |
| track_loading(test) | 4 | 0 | 0 | png_golden(test) | 1 | 0 | 0 |
| game_core(lib) | 145 | 0 | 1 | xiyou(bin) | 40 | 0 | 0 |
| combat_golden(test) | 6 | 0 | 0 | game_oracle(lib) | 0 | 0 | 0 |
| config_proof(test) | 1 | 0 | 0 | boot_replay(test) | 2 | 0 | 0 |
| density_draw(test) | 16 | 0 | 0 | fixture_frames(test) | 25 | 0 | 0 |
| entity_golden(test) | 12 | 0 | 0 | gameplay_floor1(test) | 2 | 0 | 3 |
| hd_visual(test) | 6 | 0 | 0 | l2_prologue(test) | 1 | 0 | 0 |
| mot_if_truncation(test) | 6 | 0 | 0 | preset_cases(test) | 35 | 0 | 0 |
| mov_route_scene(test) | 4 | 0 | 0 | probe_floor2(test) | 1 | 0 | 0 |
| one_fact_one_host(test) | 6 | 0 | 0 | walk_replay_floor1(test) | 2 | 0 | 0 |
| render_category_table_driven(test) | 4 | 0 | 0 | game_platform(lib) | 5 | 0 | 0 |
| render_golden(test) | 16 | 0 | 0 | asset_source_contract(test) | 4 | 0 | 0 |
| script_golden(test) | 14 | 0 | 0 | audio_out_contract(test) | 6 | 0 | 0 |
| sheet_index_table_driven(test) | 6 | 0 | 0 | process_control_contract(test) | 5 | 0 | 0 |
| size_dispatch_table_driven(test) | 5 | 0 | 0 | storage_contract(test) | 7 | 0 | 0 |
| slot_equip_scene(test) | 2 | 0 | 0 | game_shell_common(lib) | 26 | 0 | 0 |
| style_sample(test) | 1 | 0 | 0 | transition_table_driven(test) | 10 | 0 | 0 |
| undo_journal(test) | 11 | 0 | 0 | walk_camera_golden(test) | 4 | 0 | 0 |
| walk_route(test) | 11 | 0 | 0 | | | | |

\* 该行为 1700cec 时点（26）；`bbda363` 复原两测后=28（见 §3c）。总账 595→597。

**「host 75/75」声称对账：成立（绿）。** `cargo test -p game-android` = 5+16+9+
26+4+11+4 = **75/75**；逐 commit 自洽 50081d8=74→d80d96e=73→1700cec=75（+1/−1/+2
与 native_parse fn 数 25→24→26 严格同轨）。**但** 74→73 的 −1 非"去重"而是删两测
（§3c）。"47/47 保真"为旧口径子集（44 集成+lib3，lib 现为 5），总数可对账、子集
口径随批漂移，记口径注。

### 1b. `python3 gates/cli.py check` —— **12/12**（干净重跑）

`[✓ PASS]×12`：original-integrity / assets-manifest / reference-seed-integrity /
knowledge-format / docs-links / naming-ledger / **rust-tests（cargo test --release
全绿）** / **test-ratchet（total=597，基线 590，无回退）** / table-content-hash /
table-evidence / state-machine-anchors / asset-truth-manifest。汇总 PASS=12 FAIL=0
SKIP=0。首轮 20:50 的 11/12（rust-tests 假红）为并发破坏窗口污染，已甄别（§头部声明）。

### 1c. `cargo test -p game-oracle --release` —— **金样 35 零漂移**

`preset_cases(test) 35 passed / 0 failed`（274s 全帧比对）；同套 boot_replay 2、
fixture_frames 25、gameplay_floor1 2(+3 ignored)、l2_prologue 1、probe_floor2 1、
walk_replay_floor1 2 全绿。**35/35 零漂移成立（绿）。**

### 1d. 实跑过程源码钉（防并发污染）

复核期间五个关键源文件 sha256 前后对账：paint.rs/render.rs/title.rs/native.rs
全程未变（0368c96f/8195d0cb/0b375e0d/3f33c8d7）；native_parse.rs 于 20:59 被
`bbda363`（lead 修复 R1）合法变更（7df24815→7272d929），已核其 diff=逐字复原两测。

## §2 触屏五链回归 —— 4 绿 + 1 红（menu_hit 链路）

| 链 | 测试钉 | 实跑 | 判 |
|---|---|---|---|
| menu_hit | `native_parse.rs::menu_hit_rows_and_gaps`（6 行四角+行隙+(350,542)+区外） | 绿 | **红**（几何钉绿≠链路对，见 R3） |
| walk_to | `tap_chain_buf_to_walk_to`（buf→map_touch_xy→classify_tap→WalkTo 全链）+ `walk_route.rs` 11 测 | 26/26、11/11 绿 | 绿 |
| BACK(-7) | `back_maps_to_minus7_and_consumed`、`back_repeat_dropped_but_still_consumed`、`hidden_drops_input_but_back_still_consumed`、`key_press_maps_to_game_codes`（AKEYCODE_BACK=(Press(-7),true) 恒吞 BUG-020） | 绿 | 绿 |
| UNDO | `undo_zone_hit_corners`（右上 56×20 实证）、`tap_classifies_walk_wall_probe_and_undo`、undo_ui 3 测、`overlay_rects_match_hit_tables`（UNDO 框中心命中） | 绿 | 绿 |
| 滑动踏步 | `swipe_maps_to_facing_and_one_step_per_gesture`（一格/次）、`swipe_wall_stop_no_repeat`（撞墙即停）、`interrupt_signal_on_every_channel` | 绿 | 绿 |

## §3 TDD 红抽查（1700cec / d80d96e）

### 3a. 红三件套在 commit message —— 在（绿）

- `d80d96e`：红测试代码先行（menu_hit 钉）／红输出留痕（E0428×2+E0599+E0382，
  且含【2026-10-23 更正】两度误记"ndk 0 错"）／commit 本体绿（73/73+ndk 0 错+
  strings 实证）。三件套齐、更正文化合规。
- `1700cec`：红测试代码先行（双钉）／行为红一段（UNDO 框中心未命中——单源钉抓
  脱节）／commit 本体绿（75/75+ndk 重链）。三件套齐。

### 3b. worktree 复现一红 —— 成功（绿，已还原）

1. 破坏：注释 `BootMachine::paint` Modern 清屏 `g.fill_rect(0,0,screen_w,screen_h)`；
2. 钉红：`cargo test -p game-android --release --test native_parse modern_paint_
   no_residual_pixels` → **FAILED：残留 4 px**（DEADBEEF 钉抓住清屏缺失）；
3. 还原：`git checkout -- paint.rs`，sha256 复位 `0368c96f…`（与破坏前钉一致），
   `git diff` 空；复跑 native_parse **28/28 绿**。
- 附注：残留仅 4 px（各态 painter 几乎铺满）——钉有效性成立但灵敏度低，属可接受。

### 3c. 【红·复核中已修】d80d96e"去重"实为删除两个真实测试（R1）

`git show d80d96e -- native_parse.rs` 实证：E0428"测试段重复去重"的修法是把两段
**真实测试整体删除**换上 menu_hit 钉：

| 被删测试 | 来源 | 性质 |
|---|---|---|
| `title_cover_bottom_keeps_logo_visible` | 50081d8 底锚唯一钉（画布底行=源底 logo 色+y+h=画布底） | 标题截断修复唯一回归保护被下一笔 commit 删除 |
| `title_button_hit_diff_smoke` | 3cb1605 存量（pointer_hit 命中面） | 存量钉被顺带删除 |

fn 机械对账 25→24（−2+1）→26→28（复原+2）；删除期全仓无副本；`title_fill_rect_
bottom` 一度沦为死代码。commit 称"去重"与实况（删 2 增 1、74→73）不符=误导性
提交说明+ratchet 违规面。**处置：本复核 20:52 中途警报 → lead 20:59:26 落
`bbda363` 逐字复原两测+证伪更正留痕（历史不删改），ratchet live 597 对账。闭环。**

## §4 音频链 —— 绿

- **装配测试在**：`track_loading.rs` 4/4（AssetSource::read→decode→TrackStore 装配、
  loop 点 (11025,33075) 交叉核对、缺失/损坏转静音、越界忽略）；Mixer/选轨面
  `shell/common/src/audio.rs`（mixer_loops_restarts_and_silences、mixer_gain_and_
  unloaded_track、controller_translates_request_changes、audio_loop 契约）+ `audio_
  out_contract` 6/6，全绿。
- **五哨兵串 strings 实证在 .so：5/5 FOUND**——`aaudio thread spawn` / `aaudio open
  ret` / `loaded ok`（track{i} loaded ok）/ `bgm req=` / `playBgm track=`（另
  `menu_hit(` 日志串亦在）。两份 .so（/tmp/jniLibs+target 新件、android/app 旧件）
  均含五哨兵（哨兵属 d80d96e，旧件亦有）。
- 注：native.rs 装配点（cfg-android）本身无宿主测试，靠 ndk 编译+哨兵串+设备二分
  覆盖——既有口径，非本批缺口。

## §5 ratchet —— 黄

- `python3 gates/cli.py ratchet` / gates check 内嵌：**PASS — total=597（基线
  590）无回退**。逐二进制对账：47 键仅 `native_parse(test)` 变动（21→28，+7，
  含 bbda363 复原 2），其余全部恰平，**无一降**。
- **待 update 清单**（基线更新须单独 commit 写明原因，AGENTS.md §2）：
  1. `"native_parse(test)": 21 → 28`；
  2. `"total": 590 → 597`；
  3. 其余 45 键零变动，无需动。
- **黄·快照盲区**：ratchet 只比"当前 vs 基线快照"，看不见链内删测——R1（d80d96e
  删 2 测）因基线更老（21<28）而数值无感，靠 git fn 清单对账补证才现形。数值绿
  ≠ 链内无删测；建议 ratchet 增"HEAD 前 N 个 commit 内 fn 清单 diff"辅助面。

## §6 classic 面 —— 绿（注 1 处签名加宽）

`git diff 50081d8^..1700cec -- crates/game-core/src/` 逐块核：

| 文件 | 变更 | classic 影响 |
|---|---|---|
| paint.rs | +`title_fill_rect_bottom`（additive；随 R1 一度死代码，bbda363 后有钉）；`BootMachine::paint` Modern-only 清屏（`if preset==Modern`） | 零（guard 单边） |
| render.rs | `draw_image_cover` 签名 +`anchor_bottom: bool`；false 分支=旧居中公式逐字不变 | **唯一非纯 additive**（签名加宽）；全仓唯一调用点=title.rs Modern 分支（grep 实证 classic 零调用） |
| title.rs | Modern 分支 `draw_image_cover(...,true)` 1 行；+`menu_item_rects`/`menu_hit`（additive） | 零（classic `draw_image` 分支未动） |

- classic 运行时路径零改动；金样旁证 `preset_cases` 35/35 零漂移。
- 判定：行为 additive 成立；字面"仅 additive"有 1 处签名加宽（draw_image_cover，
  Modern-only API），记注不记红。

## §7 差异/问题登记（diff-case 模板）

## diff-case R1 —— d80d96e"去重"删除两个真实测试 【已闭环】
- 输入序列: `git show d80d96e -- shell/android/tests/native_parse.rs`；fn 清单
  25→24→26→28 机械对账
- 现象: `title_cover_bottom_keeps_logo_visible`（50081d8 底锚唯一钉）+
  `title_button_hit_diff_smoke`（3cb1605 存量）被整体删除；"去重"表述与实况
  （删 2 增 1）不符；`title_fill_rect_bottom` 一度死代码
- 分类: 测试资产回退 + 误导性提交说明（ratchet 快照盲区外穿）
- 处理: **已修**——lead `bbda363`（20:59:26）按 50081d8 原文逐字复原两测+证伪
  更正留痕；复跑 28/28 绿、ratchet 597 对账（复核C 硬发现①闭环）
- 门禁: `python3 gates/cli.py ratchet` → PASS total=597(基线590)（注意其盲区）

## diff-case R2 —— 打包面 .so 不含 1700cec（双路径工件分叉）【黄】
- 输入序列: sha256+strings 对照三份 .so
- 现象: `target/aarch64-linux-android/release/libgame_android.so`(20:21:23) 与
  `/tmp/jniLibs/arm64-v8a/libgame_android.so` 同 sha=`25a23422…`（含 1700cec：
  "dpad"/"undo" overlay 标签在）；`android/app/src/main/jniLibs/arm64-v8a/
  libgame_android.so`(20:13:19) sha=`7e9efb4f…`（**不含 1700cec**：dpad/undo=0），
  且 gradle intermediates 20:13:20 已按旧件打过包。源码 mtime（paint.rs 20:19:53、
  native.rs 20:20:54）晚于旧件构建
- 分类: 交付工件链分叉——装机源（/tmp/jniLibs）新、APK 打包源（android/app
  jniLibs）旧；`cargo ndk -o android/app/src/main/jniLibs` README 配方未走完
- 处理: lead 已采"装机前 sha 对账"纪律（bbda363 声明）；仍建议重链后同步
  android/app jniLibs 再打 APK，**设备复测结论以 sha=25a23422 链为准**
- 门禁: 无现成门禁覆盖 jniLibs 新鲜度；建议装机脚本加 sha 报账

## diff-case R3 —— menu_hit 链"切换+确认一步到位"未接线 【红·待修】
- 输入序列: `shell/android/src/native.rs:922-931` tap 链 + `crates/game-core/src/
  paint.rs:307-321` select_row + `title.rs:237-247` menu_kinds
- 现象: tap 命中行 i ⇒ `st.machine.select_row(i)`——**BootMachine::select_row 只
  处理 EquipDetail/About，MainMenu（标题）态 no-op**，TitleMachine.cursor 不动
  （全仓无其他写点，grep 双证）；随后 Press(-5) 确认的是**原光标项**而非被点行。
  commit 声称"tap 菜单项 i ⇒ cursor=i + 确认直发"，"切换"半边无实现。且：
  ①`menu_hit_rows_and_gaps` 只钉几何纯函数，无链路断言（绿≠链路对）；
  ②实机"开始游戏"点 (350,542) 落 row5（y528-556）⇒ kind5=LogoAnim（kind0 才是
  IntroCutscene=开始游戏），钉仅断言 `is_some()`，语义错位无保护；
  ③EquipDetail/About 态 select_row 收到的是**标题行网格**下标，与槽位/选项行带
  几何不符（错网格喂光标）
- 分类: 移植/接线错（2c 裁定实施缺半边）+ 钉未及链路
- 处理: 回 porter——select_row 补 MainMenu→title.cursor 分支（或 tap 链直改
  cursor）；补"tap 行 i ⇒ 确认 item i"链路钉与 (350,542)→预期项语义钉；行序与
  用户"开始游戏"预期的对齐交 lead 裁定
- 门禁: `cargo test -p game-android --release --test native_parse` → 28/28 绿
  （正证缺口：全绿挡不住本错）

## diff-case R4 —— menu_item_rects"paint 与 hit-test 共调"表述不实 【黄】
- 现象: `menu_item_rects` 调用方仅 menu_hit+overlay_rects（native.rs:488），
  **title paint 不调用**（可见菜单=icon strip+箭头底条，非 6 行）；"单源"实覆盖
  hit-test+overlay 两端
- 分类: 表述过宽（黄，非行为回退）
- 处理: 勘误表述；若需 paint 同源另接线

## diff-case Y5 —— pending_taps 消费无模式门控 【黄·待设备日志】
- 现象: pending_taps 仅"Up 且画布内且 keys 空"入队，主循环 menu_hit 消费**无
  mode 判断**——游戏态 keys 空 tap 若落 576² 标题行带（y378-556、x80-496）会注入
  Press(-5)（游戏内=确认/开火语义）
- 分类: 潜在输入污染（B 级推断，未实机证实）
- 处理: 用 `jjxy-input menu_hit(...)→Some/None` 日志设备核实；建议消费处加
  `mode==MainMenu` 门控

## 总判

**有条件可交付**——门禁/金样/五链四链面全绿，但 2c 核心声称存在 1 红。

- 实跑计数：workspace **591+0f+4i（=595→597）**；host（game-android）**75/75**
  对账成立（复原后 77）；gates **12/12**（干净重跑）；金样 **preset_cases 35/35
  零漂移**；ratchet **597≥590 无回退**；五哨兵 **5/5 strings 实证**；红复现
  **残留 4 px→还原→28/28 绿**。
- 红（1）：**R3 menu_hit 链"切换+确认一步到位"未接线**——select_row 标题态
  no-op，tap 行 i 确认的是原光标项；(350,542)"开始游戏"点实落 LogoAnim 行；
  无链路钉。这是 d80d96e 的核心交付声称，须回 porter 补接线+链路钉后再验收。
- 黄（4）：R2 打包 .so 双路径分叉（android/app jniLibs 旧件不含 1700cec；装机
  源 /tmp/jniLibs 新件 OK）；R4 "paint 共调"表述不实；Y5 tap 消费无模式门控；
  ratchet 待 update（native_parse 21→28、total 590→597）+快照盲区。
- 绿：全量实跑、gates 12/12、金样 35 零漂移、TDD 三件套+红复现、音频装配+五哨兵、
  classic 面行为 additive（金样旁证）。
- 已闭环：R1 误删两测（本复核发现→bbda363 逐字复原+证伪更正，测试面 28/28 绿）。
- 破坏性验证已还原：paint.rs sha 复位 `0368c96f…`，`git diff` 空。
