# 覆盖矩阵（层次一账本，机器生成）

> 生成：`python3 tools/cover_matrix.py`（v2.1，2026-10-15 刷账：v2 重写 + v2.1 行锚归属治理；v1 = 迭代 72）。
> 触达数据：`python3 tools/method_reach.py` → `tools/method_reach/snapshot.json`。
> **请勿手改本文件**——改判定请改 tools/cover_matrix.py 的 PORT_NOTES/TRIAGE/YELLOW_NOTES/ARM_GAPS（逐条带证据，脚本校验锚点）。
> 判据文本见 [docs/spec/completeness.md](spec/completeness.md)「覆盖矩阵刷账（2026-10-15）」；白项 b 类死代码裁决书见 [docs/spec/dead-code-rulings.md](spec/dead-code-rulings.md)（`python3 tools/dead_code_ruling.py --check` 字节码机械复核）。

## 汇总：绿 152 / 黄 2 / 白 9（共 163 方法）

- 白项分账（现白 9）：a 真未移植 0 / b 不触达 9（☠️ 已裁死 9 / 待裁 0） / 未分账 0
- 刷账三分账（§4.1）：基线 23 白 = a 3（其后已移植 3）/ b 9 / c 11（c 已解决 → 现绿 11 / 黄 0）；v2.1 归属治理新暴露 7 项 = b 1 / c 6（c 经核验恢复移植账）
- 黄项：2（被无对拍用例的场景触达 0 = 补用例即可转绿；无任何场景触达 2 = 带不触达论证 2 / 待论证 0——convergence-plan §0.1）
- **判据①赤字**（convergence-plan §0.1：白项清零、黄项带论证）：白（非 ☠️）0 + 黄（无论证）0 = 0
- 口径复算（同一份 Rust 树）：v2 前任口径 绿 144 / 黄 3 / 白 16（差异见 §7）；v1 旧账（伪名口径，103 名）绿 44 / 黄 1 / 白 58（对照见 §6）

三色判定（逐方法，机械）：**绿** = 有移植账 ∧ 被帧级对拍用例窗口内的 Java 执行触达；**黄** = 有移植账 ∧ 无帧级窗口触达；**白** = 无移植账。**Stage A 判据（convergence-plan §0.1）：白项清零、黄项带不触达论证**。方法级绿 ≠ 方法内每个分支已验——臂级缺口见 §5（Rust `todo!` 挡路点）与 type-closure.md；绿也 ≠ 副作用已验：帧 sha 不含 RMS/音频，此类方法的绿只证明 Java 执行落在已验窗口内。

## 0. 移植账归属规则与自检（v2.1）

- 行锚方言：显式 `seed/a.java` 或 seed 方言文件（含显式 seed 锚的文件）的裸锚 → reference/seed/a.java；其余 → deobf。seed 与 deobf 以方法序数对齐（运行时校验 163 个、逐序参数类型一致）。
- 名证锚：锚所在行点名 Java 方法（现名/台账别名/引用签名）⇒ 只归属被点名者，容差 ±20 行；稳健锚：未点名 ⇒ 锚外扩 20 行仍须落在 VF 时代 20 个 deobf 版本（996a29d 起 + 工作区）方法区间的交集内；字段初始化器行归 <clinit>/<init>（不外扩）。
- 行锚精度自检（名证锚 489 条到被点名方法区间的最近距离）：0 行 330，1-5 行 25，6-10 行 9，11-20 行 9，21-50 行 8，>50 行 108——>20 行者多为「点名调用者、引用别处行」的语境锚，不予归属；v2 朴素口径（锚落区间即算）把漂移锚记给邻近小方法，§7 列出全部差异。
- 移植扫描范围：crates/{game-core,game-data,game-platform,game-desktop}/src（game-oracle 为差分宿主，不计移植证据）。

## 1. 对拍用例 → 场景 → 窗口（crates/game-oracle/tests、crates/game-core/tests 机械解析，只认 git 已追踪文件）

帧级用例两类：A-trace 用例（运行期读 `_out/A-<场景>/trace.txt`）与夹具帧用例（期望帧嵌入 `<场景>.frames.txt`，tools/frames_fixture.py 产出；夹具帧链须 = JDI 实测运行的帧链才计入验证）。

| 用例 | 场景（A-trace） | 窗口 | FRAME 对照 | 验证触达方法数 |
|---|---|---|---|---|
| `boot_replay::boot_menu_frames_match_tick_by_tick` | menu-sweep | T1-1831 | FRAME 一致 | 45 |
| `boot_replay::enter_game_frames_match_tick_by_tick` | enter-game | T1-380 | FRAME 一致 | 37 |
| `fixture_frames::preset_floor5_fixture_frames_match` | preset-floor5（夹具） | T1-1200 | FRAME 一致（夹具帧链 = 实测） | 89 |
| `fixture_frames::preset_floor49_fixture_frames_match` | preset-floor49（夹具） | T1-3400 | FRAME 一致（夹具帧链 = 实测） | 93 |
| `fixture_frames::fx_scriptwalk_floor3_frames_match` | fx-scriptwalk-floor3（夹具） | T1-700 | FRAME 一致（夹具帧链 = 实测） | 90 |
| `fixture_frames::fx_knockback_floor43_frames_match` | fx-knockback-floor43（夹具） | T1-600 | FRAME 一致（夹具帧链 = 实测） | 81 |
| `fixture_frames::fx_fan_floor13_frames_match` | fx-fan-floor13（夹具） | T1-830 | FRAME 一致（夹具帧链 = 实测） | 86 |
| `fixture_frames::fx_firebump_floor13_frames_match` | fx-firebump-floor13（夹具） | T1-480 | FRAME 一致（夹具帧链 = 实测） | 77 |
| `fixture_frames::fx_int6a_reveal_frames_match` | fx-int6a（夹具） | T1-560 | FRAME 一致（夹具帧链 = 实测） | 74 |
| `fixture_frames::fx_bug_aioobe_border_frames_match` | fx-bug-aioobe-border（夹具） | T1-236 | FRAME 一致（夹具帧链 = 实测） | 59 |
| `fixture_frames::fx_door2a_open_frames_match` | fx-door2a（夹具） | T1-520 | FRAME 一致（夹具帧链 = 实测） | 78 |
| `fixture_frames::fx_door2b_nokey_frames_match` | fx-door2b（夹具） | T1-520 | FRAME 一致（夹具帧链 = 实测） | 79 |
| `fixture_frames::fx_item17b_fail_no_consume_frames_match` | fx-item17b（夹具） | T1-560 | FRAME 一致（夹具帧链 = 实测） | 80 |
| `fixture_frames::fx_item19a_heal_exact_frames_match` | fx-item19a（夹具） | T1-560 | FRAME 一致（夹具帧链 = 实测） | 82 |
| `fixture_frames::fx_item17a_chisel_wall_frames_match` | fx-item17a（夹具） | T1-620 | FRAME 一致（夹具帧链 = 实测） | 86 |
| `fixture_frames::fx_item18a_clear_all_walls_frames_match` | fx-item18a（夹具） | T1-560 | FRAME 一致（夹具帧链 = 实测） | 81 |
| `fixture_frames::fx_boss67a_kill_spawn_frames_match` | fx-boss67a（夹具） | T1-560 | FRAME 一致（夹具帧链 = 实测） | 89 |
| `fixture_frames::fx_boss74a_kill_hooks_frames_match` | fx-boss74a（夹具） | T1-600 | FRAME 一致（夹具帧链 = 实测） | 100 |
| `fixture_frames::fx_int11_f23_transform_silent_frames_match` | fx-int11f23（夹具） | T1-560 | FRAME 一致（夹具帧链 = 实测） | 72 |
| `fixture_frames::fx_item20a_no_teleport_floor40_frames_match` | fx-item20a（夹具） | T1-680 | FRAME 一致（夹具帧链 = 实测） | 79 |
| `fixture_frames::fx_int11_n23_wall_message_frames_match` | fx-int11n23（夹具） | T1-400 | FRAME 一致（夹具帧链 = 实测） | 78 |
| `fixture_frames::fx_boss68a_kill_spawn_frames_match` | fx-boss68a（夹具） | T1-760 | FRAME 一致（夹具帧链 = 实测） | 88 |
| `fixture_frames::fx_bug_ces70_slot_frames_match` | fx-bug-ces70-slot（夹具） | T1-349 | FRAME 一致（夹具帧链 = 实测） | 83 |
| `preset_cases::preset_load_menu_frames_match` | preset-load-menu | T1-800 | FRAME 一致 | 85 |
| `preset_cases::preset_help_frames_match` | preset-help | T240-800 | FRAME 一致 | 35 |
| `preset_cases::preset_item13_frames_match` | preset-item13 | T1-500 | FRAME 一致 | 75 |
| `preset_cases::preset_itemstack_frames_match` | preset-itemstack | T1-500 | FRAME 一致 | 77 |
| `preset_cases::preset_item2122_frames_match` | preset-item2122 | T1-900 | FRAME 一致 | 81 |
| `preset_cases::preset_item21_frames_match` | preset-item21 | T1-900 | FRAME 一致 | 88 |
| `preset_cases::preset_itemcymbal_frames_match` | preset-itemcymbal | T1-600 | FRAME 一致 | 90 |
| `preset_cases::preset_itemboss40_frames_match` | preset-itemboss40 | T1-700 | FRAME 一致 | 97 |
| `preset_cases::preset_floor2_frames_match` | preset-floor2 | T1-600 | FRAME 一致 | 82 |
| `preset_cases::preset_merchant_frames_match` | preset-merchant | T1-600 | FRAME 一致 | 82 |
| `preset_cases::floor_crossing_frames_match` | floor1-to-floor2 | T1-1500 | FRAME 一致 | 94 |
| `preset_cases::prologue_frames_match` | prologue-dialog | T1-9800 | FRAME 一致 | 96 |
| `preset_cases::floor51_menu_frames_match` | floor51-menu | T1-9800 | FRAME 一致 | 96 |
| `preset_cases::gameplay_floor1_frames_match` | gameplay-floor1 | T1-3000 | FRAME 一致 | 87 |
| `preset_cases::floor2_tour_frames_match` | floor2-tour | T1-2000 | FRAME 一致 | 101 |
| `preset_cases::floor1_tour_frames_match` | floor1-tour | T1-8000 | FRAME 一致 | 92 |
| `preset_cases::walk_pure_frames_match` | walk-pure | T1-4000 | FRAME 一致 | 91 |
| `preset_cases::preset_save_frames_match` | preset-save | T1-700 | FRAME 一致 | 81 |
| `preset_cases::preset_floor35_frames_match` | preset-floor35 | T1-1200 | FRAME 一致 | 100 |
| `preset_cases::preset_floor10_frames_match` | preset-floor10 | T1-1310 | FRAME 一致 | 84 |
| `preset_cases::preset_floor46_frames_match` | preset-floor46 | T1-1700 | FRAME 一致 | 85 |
| `preset_cases::preset_floor41_frames_match` | preset-floor41 | T1-1900 | FRAME 一致 | 102 |
| `preset_cases::preset_floor44_frames_match` | preset-floor44 | T1-1600 | FRAME 一致 | 99 |
| `preset_cases::preset_floor45_frames_match` | preset-floor45 | T1-1700 | FRAME 一致 | 97 |
| `preset_cases::preset_floor48_frames_match` | preset-floor48 | T1-3440 | FRAME 一致 | 95 |
| `preset_cases::preset_floor14_frames_match` | preset-floor14 | T1-4790 | FRAME 一致 | 85 |
| `preset_cases::preset_floor38_frames_match` | preset-floor38 | T1-6050 | FRAME 一致 | 92 |
| `preset_cases::preset_floor47_frames_match` | preset-floor47 | T1-1350 | FRAME 一致 | 85 |
| `preset_cases::preset_floor34_frames_match` | preset-floor34 | T1-3030 | FRAME 一致 | 87 |
| `preset_cases::preset_floor10_nw_frames_match` | preset-floor10 | T1-1310 | FRAME 一致 | 84 |
| `preset_cases::preset_floor10b_frames_match` | preset-floor10b | T1-850 | FRAME 一致 | 84 |
| `preset_cases::preset_floor17_frames_match` | preset-floor17 | T1-850 | FRAME 一致 | 86 |
| `preset_cases::preset_floor42_frames_match` | preset-floor42 | T1-1700 | FRAME 一致 | 93 |
| `preset_cases::preset_floor43_frames_match` | preset-floor43 | T1-1500 | FRAME 一致 | 92 |
| `preset_cases::preset_alchemy_frames_match` | preset-alchemy | T1-700 | FRAME 一致 | 87 |
| `mov_route_scene::preset_floor2_mov_frames_match` | preset-floor2-mov（夹具） | T1-900 | FRAME 一致（夹具帧链 = 实测） | 95 |
| `mov_route_scene::preset_floor40_mov_frames_match` | preset-floor40-mov（夹具） | T1-1000 | FRAME 一致（夹具帧链 = 实测） | 105 |
| `slot_equip_scene::preset_save_equip_frames_match` | preset-save-equip（夹具） | T1-1040 | FRAME 一致（夹具帧链 = 实测） | 91 |

非帧级用例（不计入验证；列出以示归属）：

- `gameplay_floor1::gameplay_floor1_t544_ops_prefix_match`（gameplay_floor1.rs:227）：trace-partial，读 A-gameplay-floor1
- `gameplay_floor1::floor1_entity_draw_order_anchor`（gameplay_floor1.rs:290）：structural
- `gameplay_floor1::dump_t544_pixels`（gameplay_floor1.rs:341）：ignored
- `gameplay_floor1::dump_frame_pixels`（gameplay_floor1.rs:368）：ignored，读 A-gameplay-floor1
- `gameplay_floor1::dump_entity87`（gameplay_floor1.rs:410）：ignored
- `l2_prologue::l2_prologue_cursor_and_text_transitions_match_java`（l2_prologue.rs:69）：fixture
- `probe_floor2::probe_floor2_type78`（probe_floor2.rs:55）：structural
- `walk_replay_floor1::walk_pure_replay_pixel_timeline_matches_java`（walk_replay_floor1.rs:235）：fixture
- `walk_replay_floor1::walkability_grid_anchor`（walk_replay_floor1.rs:322）：structural
- crates/game-core/tests/combat_golden.rs：6 个（structural 6）
- crates/game-core/tests/entity_golden.rs：8 个（fixture 1 / structural 7）
- crates/game-core/tests/mov_route_scene.rs：2 个（fixture 2）
- crates/game-core/tests/render_golden.rs：16 个（structural 16）
- crates/game-core/tests/script_golden.rs：14 个（structural 14）
- crates/game-core/tests/slot_equip_scene.rs：1 个（fixture 1）
- crates/game-core/tests/walk_camera_golden.rs：4 个（structural 4）

场景触达（JDI 实测，全程 = 各相位并集取最小 t_hit；相位起点 = 1 ∪ 用例窗口起点）：

| 场景 | 帧数 | 结束 | 相位 | 触达方法数 | A-trace 对照 | 帧级用例 |
|---|---|---|---|---|---|---|
| boot-menu | 150 | reason=max-ticks ticks=150 | 1 | 39 | equal | — |
| enter-game | 500 | reason=max-ticks ticks=500 | 1 | 82 | equal | ✓ |
| floor1-to-floor2 | 1500 | reason=max-ticks ticks=1500 | 1 | 94 | equal | ✓ |
| floor1-tour | 8000 | reason=max-ticks ticks=8000 | 1 | 92 | equal | ✓ |
| floor2-tour | 2000 | reason=max-ticks ticks=2000 | 1 | 101 | equal | ✓ |
| floor51-menu | 9800 | reason=max-ticks ticks=9800 | 1 | 96 | equal | ✓ |
| fx-boss67a | 560 | reason=max-ticks ticks=560 | 1 | 89 | equal（夹具） | ✓ |
| fx-boss68a | 760 | reason=max-ticks ticks=760 | 1 | 88 | equal（夹具） | ✓ |
| fx-boss74a | 600 | reason=max-ticks ticks=600 | 1 | 100 | equal（夹具） | ✓ |
| fx-bug-aioobe-border | 236 | reason=max-ticks ticks=236 | 1 | 59 | equal（夹具） | ✓ |
| fx-bug-ces70-slot | 349 | reason=max-ticks ticks=349 | 1 | 83 | equal（夹具） | ✓ |
| fx-door2a | 520 | reason=max-ticks ticks=520 | 1 | 78 | equal（夹具） | ✓ |
| fx-door2b | 520 | reason=max-ticks ticks=520 | 1 | 79 | equal（夹具） | ✓ |
| fx-fan-floor13 | 830 | reason=max-ticks ticks=830 | 1 | 86 | equal（夹具） | ✓ |
| fx-firebump-floor13 | 480 | reason=max-ticks ticks=480 | 1 | 77 | equal（夹具） | ✓ |
| fx-int11f23 | 560 | reason=max-ticks ticks=560 | 1 | 72 | equal（夹具） | ✓ |
| fx-int11n23 | 400 | reason=max-ticks ticks=400 | 1 | 78 | equal（夹具） | ✓ |
| fx-int6a | 560 | reason=max-ticks ticks=560 | 1 | 74 | equal（夹具） | ✓ |
| fx-item17a | 620 | reason=max-ticks ticks=620 | 1 | 86 | equal（夹具） | ✓ |
| fx-item17b | 560 | reason=max-ticks ticks=560 | 1 | 80 | equal（夹具） | ✓ |
| fx-item18a | 560 | reason=max-ticks ticks=560 | 1 | 81 | equal（夹具） | ✓ |
| fx-item19a | 560 | reason=max-ticks ticks=560 | 1 | 82 | equal（夹具） | ✓ |
| fx-item20a | 680 | reason=max-ticks ticks=680 | 1 | 79 | equal（夹具） | ✓ |
| fx-knockback-floor43 | 600 | reason=max-ticks ticks=600 | 1 | 81 | equal（夹具） | ✓ |
| fx-scriptwalk-floor3 | 700 | reason=max-ticks ticks=700 | 1 | 90 | equal（夹具） | ✓ |
| gameplay-floor1 | 3000 | reason=max-ticks ticks=3000 | 1 | 87 | equal | ✓ |
| menu-sweep | 1832 | reason=all-threads-exited ticks=1832 | 1 | 45 | equal | ✓ |
| preset-alchemy | 700 | reason=max-ticks ticks=700 | 1 | 87 | equal | ✓ |
| preset-ending | 1800 | reason=max-ticks ticks=1800 | 1 | 96 | equal | — |
| preset-floor10 | 1310 | reason=max-ticks ticks=1310 | 1 | 84 | equal | ✓ |
| preset-floor10b | 850 | reason=max-ticks ticks=850 | 1 | 84 | equal | ✓ |
| preset-floor14 | 4790 | reason=max-ticks ticks=4790 | 1 | 85 | equal | ✓ |
| preset-floor17 | 850 | reason=max-ticks ticks=850 | 1 | 86 | equal | ✓ |
| preset-floor2 | 600 | reason=max-ticks ticks=600 | 1 | 82 | equal | ✓ |
| preset-floor2-mov | 900 | reason=max-ticks ticks=900 | 1 | 95 | equal（夹具） | ✓ |
| preset-floor34 | 3030 | reason=max-ticks ticks=3030 | 1 | 87 | equal | ✓ |
| preset-floor35 | 1200 | reason=max-ticks ticks=1200 | 1 | 100 | equal | ✓ |
| preset-floor38 | 6050 | reason=max-ticks ticks=6050 | 1 | 92 | equal | ✓ |
| preset-floor40-mov | 1000 | reason=max-ticks ticks=1000 | 1 | 105 | equal（夹具） | ✓ |
| preset-floor41 | 1900 | reason=max-ticks ticks=1900 | 1 | 102 | equal | ✓ |
| preset-floor42 | 1700 | reason=max-ticks ticks=1700 | 1 | 93 | equal | ✓ |
| preset-floor43 | 1500 | reason=max-ticks ticks=1500 | 1 | 92 | equal | ✓ |
| preset-floor44 | 1600 | reason=max-ticks ticks=1600 | 1 | 99 | equal | ✓ |
| preset-floor45 | 1700 | reason=max-ticks ticks=1700 | 1 | 97 | equal | ✓ |
| preset-floor46 | 1700 | reason=max-ticks ticks=1700 | 1 | 85 | equal | ✓ |
| preset-floor47 | 1350 | reason=max-ticks ticks=1350 | 1 | 85 | equal | ✓ |
| preset-floor48 | 3440 | reason=max-ticks ticks=3440 | 1 | 95 | equal | ✓ |
| preset-floor49 | 3400 | reason=max-ticks ticks=3400 | 1 | 93 | equal | ✓ |
| preset-floor5 | 1200 | reason=max-ticks ticks=1200 | 1 | 89 | equal | ✓ |
| preset-floor50 | 300 | reason=max-ticks ticks=300 | 1 | 69 | equal | — |
| preset-floor51 | 600 | reason=max-ticks ticks=600 | 1 | 97 | equal | — |
| preset-floor8 | 8610 | reason=max-ticks ticks=8610 | 1 | 96 | equal | — |
| preset-help | 800 | reason=max-ticks ticks=800 | 1,240 | 77 | equal | ✓ |
| preset-item13 | 500 | reason=max-ticks ticks=500 | 1 | 75 | equal | ✓ |
| preset-item21 | 900 | reason=max-ticks ticks=900 | 1 | 88 | equal | ✓ |
| preset-item2122 | 900 | reason=max-ticks ticks=900 | 1 | 81 | equal | ✓ |
| preset-itemboss40 | 700 | reason=max-ticks ticks=700 | 1 | 97 | equal | ✓ |
| preset-itemcymbal | 600 | reason=max-ticks ticks=600 | 1 | 90 | equal | ✓ |
| preset-itemstack | 500 | reason=max-ticks ticks=500 | 1 | 77 | equal | ✓ |
| preset-load-menu | 800 | reason=max-ticks ticks=800 | 1 | 85 | equal | ✓ |
| preset-merchant | 600 | reason=max-ticks ticks=600 | 1 | 82 | equal | ✓ |
| preset-save | 700 | reason=max-ticks ticks=700 | 1 | 81 | equal | ✓ |
| preset-save-equip | 1040 | reason=max-ticks ticks=1040 | 1 | 91 | equal（夹具） | ✓ |
| prologue-dialog | 9800 | reason=max-ticks ticks=9800 | 1 | 96 | equal | ✓ |
| prologue-patient | 21000 | reason=max-ticks ticks=21000 | 1 | 96 | equal | — |
| walk-pure | 4000 | reason=max-ticks ticks=4000 | 1 | 91 | equal | ✓ |

## 2. 绿：已移植 + 帧级对拍窗口触达（152）

| 方法 | 区间 | 首个验证（用例 / 场景 @t_hit） | 验证用例数 | 移植证据 |
|---|---|---|---|---|
| <init> | a.java:436-2331 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T1 | 60 | 行锚×59〔seed稳1/字段3/稳55〕（crates/game-core/src/audio.rs:26） |
| paint | a.java:2333-3227 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T1 | 61 | fn crates/game-core/src/intro.rs:203; crates/game-core/src/intro.rs:312 …；行锚×58〔seed稳2/名37/稳19〕（crates/game-core/src/intro.rs:1） |
| run | a.java:3229-4290 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T1 | 60 | 行锚×110〔名51/稳59〕（crates/game-core/src/audio.rs:15） |
| dispatchModeEntry | a.java:4292-4493 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T1 | 61 | 行锚×44〔名34/稳10〕（crates/game-core/src/audio.rs:14） |
| showNotify | a.java:4495-4500 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T1 | 60 | fn crates/game-desktop/src/main.rs:553；行锚×3〔名3〕（crates/game-desktop/src/lifecycle.rs:6） |
| loadPackedImages | a.java:4516-4549 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T33 | 61 | 人工：打包图像容器装载器（a.java:4516-4549：resourceNames[i] 开流、按 resourceImageCounts[i] 循环读 u16 LE 长度 → Image.createImage）——crates/game-data/src/packed_png.rs `pub fn parse`/`pub fn parse_partial` 解析 N×[u16LE len][PNG]（L1 golden 守护）+ crates/game-desktop/src/main.rs `fn load_container` 消费 = 移植替换形态；名实更正 e772fef（N-0434）；c 类行号方言误判已由 v2.1 规则修复；行锚×1〔seed名1〕（crates/game-data/src/packed_png.rs:17） |
| drawImageClipped | a.java:4551-4555 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T68 | 61 | fn crates/game-core/src/paint.rs:42；行锚×1〔名1〕（crates/game-core/src/paint.rs:40） |
| blit5Clip | a.java:4557-4561 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T223 | 58 | fn crates/game-core/src/scene.rs:6378（fn m_003_clip5）；行锚×1〔名1〕（crates/game-core/src/scene.rs:6377） |
| drawImageWithNokiaTransform | a.java:4563-4565 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T68 | 61 | fn crates/game-core/src/paint.rs:35；行锚×1〔名1〕（crates/game-core/src/paint.rs:34） |
| drawEdgePatch | a.java:4567-4603 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 60 | fn crates/game-core/src/menu_family.rs:115；行锚×1〔名1〕（crates/game-core/src/menu_family.rs:112） |
| createSolidImage | a.java:4605-4614 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T223 | 58 | 人工·吸收：静态纯函数（造 32×32 纯色 RGB 图，a.java:4606-4613）；仅有的两个调用点 loadFloorData a.java:9423-9424 结果被丢弃（javap：invokestatic createSolidImage 后紧跟 pop）⇒ 无可观测效应 |
| dimImage | a.java:4616-4632 | `boot_replay::enter_game_frames_match_tick_by_tick` / enter-game @T102 | 59 | fn crates/game-core/src/intro.rs:32；行锚×1〔名1〕（crates/game-core/src/intro.rs:30） |
| closeStreamsAndClear | a.java:4634-4661 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T33 | 61 | 人工·吸收：流关闭 + 三字段置 null（a.java:4635-4660）；javap 字段普查：f_InputStream_00/f_OutputStream_00/f_InputStream_01 读写者仅 readCountPrefix/closeStreamsAndClear/playBgm/m_148/m_151/<init>（I/O 方法）⇒ 无游戏态效应；Rust 以内存切片解码无流可关（crates/game-data/src/packed_png.rs `pub fn parse(data: &'a [u8], image_count: usize) -> Result<Self, Error>`） |
| keyPressed | a.java:4663-4676 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 61 | 行锚×11〔名11〕（crates/game-core/src/audio.rs:65） |
| keyReleased | a.java:4678-4680 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T73 | 61 | 行锚×4〔名4〕（crates/game-core/src/scene.rs:312） |
| resetMenuLayout | a.java:4682-4694 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 61 | fn crates/game-core/src/scene.rs:2537；行锚×2〔名2〕（crates/game-core/src/scene.rs:2536） |
| addMenuItem | a.java:4696-4712 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 61 | fn crates/game-core/src/scene.rs:2549；行锚×3〔名3〕（crates/game-core/src/scene.rs:462） |
| releaseSceneImages | a.java:4714-4728 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 60 | 人工·吸收：图像容器/entityTypeImage 置 null 释放内存（a.java:4715-4727）；读者在读前经 readCountPrefix 判空重载（a.java:4519）或 initEntityTables 重建 entityTypeImage（javap：put 仅 initEntityTables/releaseSceneImages）；Rust 构造期全量装载不释放（crates/game-core/src/scene.rs `懒加载资源容器（Rust 构造期全量装载，等价 no-op）`） |
| activateMenuItem | a.java:4730-4792 | `preset_cases::preset_load_menu_frames_match` / preset-load-menu @T301 | 4 | fn crates/game-core/src/scene.rs:2705；行锚×1〔名1〕（crates/game-core/src/scene.rs:2701） |
| drawIconStrip | a.java:4794-4816 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 61 | 行锚×3〔名3〕（crates/game-core/src/title.rs:6） |
| runIntroViewer | a.java:4818-4896 | `boot_replay::enter_game_frames_match_tick_by_tick` / enter-game @T102 | 3 | 行锚×3〔名3〕（crates/game-core/src/intro.rs:1） |
| showOverlayMessage | a.java:4898-4923 | `preset_cases::preset_merchant_frames_match` / preset-merchant @T261 | 28 | fn crates/game-core/src/scene.rs:1153；行锚×2〔名2〕（crates/game-core/src/scene.rs:388） |
| handleOverlayKey | a.java:4925-4938 | `preset_cases::preset_merchant_frames_match` / preset-merchant @T262 | 24 | 行锚×2〔名2〕（crates/game-core/src/scene.rs:4057） |
| layoutFullText | a.java:4940-4942 | `boot_replay::enter_game_frames_match_tick_by_tick` / enter-game @T102 | 33 | fn crates/game-core/src/scene.rs:1186；行锚×1〔名1〕（crates/game-core/src/scene.rs:1180） |
| layoutWrappedText | a.java:4944-5019 | `boot_replay::enter_game_frames_match_tick_by_tick` / enter-game @T102 | 38 | 行锚×4〔名2/稳2〕（crates/game-core/src/menu_family.rs:313） |
| paintWrappedText | a.java:5021-5111 | `boot_replay::enter_game_frames_match_tick_by_tick` / enter-game @T102 | 38 | fn crates/game-core/src/scene.rs:1241；行锚×6〔名3/稳3〕（crates/game-core/src/menu_family.rs:449） |
| flipOverlayPage | a.java:5113-5133 | `boot_replay::enter_game_frames_match_tick_by_tick` / enter-game @T165 | 15 | fn crates/game-core/src/scene.rs:1714；行锚×1〔名1〕（crates/game-core/src/scene.rs:1713） |
| skipTextToLineEnd | a.java:5135-5157 | `mov_route_scene::preset_floor2_mov_frames_match` / preset-floor2-mov @T293 | 4 | fn crates/game-core/src/scene.rs:1757；行锚×1〔名1〕（crates/game-core/src/scene.rs:1756） |
| parseHexColor | a.java:5159-5170 | `mov_route_scene::preset_floor2_mov_frames_match` / preset-floor2-mov @T262 | 25 | fn crates/game-core/src/menu_family.rs:40；行锚×1〔名1〕（crates/game-core/src/menu_family.rs:33） |
| applyHpDelta | a.java:5172-5191 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 12 | fn crates/game-core/src/scene.rs:5428；行锚×1〔名1〕（crates/game-core/src/scene.rs:5427） |
| placePlayerAtCell | a.java:5193-5199 | `preset_cases::preset_item21_frames_match` / preset-item21 @T405 | 5 | fn crates/game-core/src/scene.rs:1110（fn m_024）；行锚×1〔名1〕（crates/game-core/src/scene.rs:1109） |
| m_025 | a.java:5201-5207 | `fixture_frames::preset_floor5_fixture_frames_match` / preset-floor5 @T262 | 39 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:4270） |
| easeAfterimages | a.java:5209-5263 | `preset_cases::floor1_tour_frames_match` / floor1-tour @T602 | 10 | fn crates/game-core/src/scene.rs:4336；行锚×1〔名1〕（crates/game-core/src/scene.rs:4334） |
| handleFieldInput | a.java:5265-5331 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T237 | 58 | fn crates/game-core/src/scene.rs:4448；行锚×1〔名1〕（crates/game-core/src/scene.rs:4445） |
| tryStep | a.java:5333-5361 | `fixture_frames::preset_floor5_fixture_frames_match` / preset-floor5 @T261 | 47 | fn crates/game-core/src/scene.rs:4511；行锚×1〔名1〕（crates/game-core/src/scene.rs:4510） |
| interactWithCell | a.java:5363-5541 | `fixture_frames::preset_floor5_fixture_frames_match` / preset-floor5 @T261 | 48 | fn crates/game-core/src/scene.rs:4537；行锚×22〔名2/稳20〕（crates/game-core/src/scene.rs:4510） |
| applyStepCellEffects | a.java:5543-5808 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T237 | 57 | fn crates/game-core/src/scene.rs:5040；行锚×9〔名1/稳8〕（crates/game-core/src/scene.rs:5029） |
| stepOffStairs | a.java:5810-5823 | `preset_cases::preset_item21_frames_match` / preset-item21 @T405 | 3 | fn crates/game-core/src/scene.rs:1095（fn m_031）；行锚×1〔名1〕（crates/game-core/src/scene.rs:1094） |
| advanceScriptWalk | a.java:5825-5842 | `fixture_frames::fx_scriptwalk_floor3_frames_match` / fx-scriptwalk-floor3 @T308 | 1 | fn crates/game-core/src/scene.rs:5244；行锚×1〔名1〕（crates/game-core/src/scene.rs:5242） |
| paintPlayerSprite | a.java:5844-6007 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T237 | 58 | 行锚×2〔名1/稳1〕（crates/game-core/src/layout.rs:174） |
| paintSoftkeyBar | a.java:6009-6019 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T68 | 61 | fn crates/game-core/src/paint.rs:50（fn m_034_softkeys）；行锚×7〔名7〕（crates/game-core/src/layout.rs:281） |
| paintStatusBar | a.java:6021-6044 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T237 | 58 | fn crates/game-core/src/scene.rs:6328；行锚×1〔名1〕（crates/game-core/src/scene.rs:6327） |
| paintKeySlot | a.java:6046-6053 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T151 | 59 | fn crates/game-core/src/scene.rs:6369；行锚×3〔名3〕（crates/game-core/src/menu_family.rs:720） |
| paintHudPanel | a.java:6055-6115 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T237 | 58 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:6025） |
| paintTitledBox | a.java:6117-6135 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 57 | fn crates/game-core/src/menu_family.rs:197；行锚×1〔名1〕（crates/game-core/src/menu_family.rs:195） |
| paintHudFrame | a.java:6137-6165 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T237 | 58 | fn crates/game-core/src/scene.rs:5997；行锚×1〔名1〕（crates/game-core/src/scene.rs:5996） |
| paintBoxFrame | a.java:6167-6199 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 60 | fn crates/game-core/src/menu_family.rs:160；行锚×1〔名1〕（crates/game-core/src/menu_family.rs:159） |
| paintMiniFrame | a.java:6201-6210 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T151 | 59 | fn crates/game-core/src/menu_family.rs:219；行锚×1〔名1〕（crates/game-core/src/menu_family.rs:218） |
| paintNumber | a.java:6212-6241 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T151 | 59 | fn crates/game-core/src/menu_family.rs:232；行锚×1〔名1〕（crates/game-core/src/menu_family.rs:230） |
| initEntityTables | a.java:6243-6298 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T213 | 58 | 行锚×2〔名2〕（crates/game-core/src/entity.rs:102） |
| dispatchSpriteSize | a.java:6300-6325 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T223 | 58 | fn crates/game-core/src/entity.rs:151；行锚×2〔名2〕（crates/game-core/src/entity.rs:148） |
| dispatchAnimFields | a.java:6327-6330 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T223 | 58 | 行锚×1〔名1〕（crates/game-core/src/entity.rs:176） |
| markEntityDesRemoved | a.java:6332-6347 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T302 | 10 | 行锚×1〔名1〕（crates/game-core/src/entity.rs:295） |
| markEntityRemoved | a.java:6349-6387 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 38 | 行锚×1〔名1〕（crates/game-core/src/entity.rs:253） |
| spawnEntity | a.java:6389-6432 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T223 | 58 | fn crates/game-core/src/script.rs:145; crates/game-core/src/script.rs:531 |
| removeGroupAtCellByType | a.java:6434-6449 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T335 | 6 | fn crates/game-core/src/entity.rs:519；行锚×2〔名2〕（crates/game-core/src/entity.rs:517） |
| removeSlotAt | a.java:6451-6463 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T312 | 8 | fn crates/game-core/src/scene.rs:1119；行锚×1〔名1〕（crates/game-core/src/scene.rs:1118） |
| pullEntityFromCell | a.java:6465-6481 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T242 | 39 | 行锚×1〔名1〕（crates/game-core/src/entity.rs:443） |
| advanceBobPhase | a.java:6483-6492 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T72 | 61 | 行锚×2〔名2〕（crates/game-core/src/entity.rs:493） |
| paintEntityLayer | a.java:6494-6736 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T237 | 58 | 行锚×19〔名5/稳14〕（crates/game-core/src/render.rs:22） |
| sortEntitiesByY | a.java:6738-6829 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T223 | 58 | 行锚×1〔名1〕（crates/game-core/src/entity.rs:310） |
| advanceEntityFrames | a.java:6831-6857 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T238 | 58 | 行锚×1〔名1〕（crates/game-core/src/entity.rs:393） |
| drawParallaxBackdrop | a.java:6859-6886 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 60 | 行锚×3〔名3〕（crates/game-core/src/menu_family.rs:53） |
| buildMinimap | a.java:6888-6956 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T223 | 58 | fn crates/game-core/src/scene.rs:2220；行锚×1〔名1〕（crates/game-core/src/scene.rs:2218） |
| readShortLE | a.java:6958-6960 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T223 | 58 | 行锚×1〔seed名1〕（crates/game-data/src/packed_png.rs:18） |
| initCellGrid | a.java:6962-6967 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T223 | 58 | 人工：crates/game-core/src/entity.rs `pub fn new(cols: i32, rows: i32) -> Self`（CellGrid::new 四件初始化 = a.java:6957-6960；doc 点名「m_059 初始化」） |
| isCellWalkable | a.java:6969-6971 | `fixture_frames::preset_floor5_fixture_frames_match` / preset-floor5 @T261 | 45 | fn crates/game-core/src/pathfind.rs:31 |
| rebuildWalkability | a.java:6973-6992 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T223 | 58 | 行锚×3〔名3〕（crates/game-core/src/game_view.rs:9） |
| m_062 | a.java:6994-6996 | `fixture_frames::fx_bug_ces70_slot_frames_match` / fx-bug-ces70-slot @T303 | 7 | fn crates/game-core/src/pathfind.rs:39; crates/game-core/src/pathfind.rs:39（fn m_062） |
| paintTileLayer | a.java:6998-7060 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T237 | 58 | 行锚×6〔名6〕（crates/game-core/src/game_view.rs:2） |
| setCameraClamped | a.java:7062-7113 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T223 | 58 | 行锚×4〔名4〕（crates/game-core/src/camera.rs:58） |
| findFloorGateEntity | a.java:7115-7133 | `preset_cases::preset_item21_frames_match` / preset-item21 @T405 | 3 | fn crates/game-core/src/scene.rs:1074；行锚×2〔名2〕（crates/game-core/src/scene.rs:1069） |
| changeFloor | a.java:7135-7181 | `preset_cases::preset_item21_frames_match` / preset-item21 @T401 | 3 | fn crates/game-core/src/scene.rs:970；行锚×3〔名3〕（crates/game-core/src/scene.rs:347） |
| startNewGameLoad | a.java:7183-7195 | `preset_cases::floor51_menu_frames_match` / floor51-menu @T541 | 2 | 行锚×2〔名2〕（crates/game-core/src/intro.rs:279） |
| resetNewGameState | a.java:7197-7225 | `preset_cases::floor51_menu_frames_match` / floor51-menu @T570 | 2 | 人工：新游戏态复位分散移植：crates/game-desktop/src/main.rs `GameScene::load_floor(` 以 (32,352) 落点 = placePlayerAtCell(1,11)（a.java:7197 floor51 臂，新游戏 step8 已置 51 层）+ crates/game-core/src/scene.rs `difficulty_index: 0`（装备/跟随/金/道具栈/炼丹/难度构造默认 0 = a.java:7192-7193、7213-7217）+ crates/game-desktop/src/main.rs `scale_enemy_stats(&tables, 1)`（= a.java:7218 scaleEnemyStats(multipliers[0])） |
| scaleEnemyStats | a.java:7227-7257 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T220 | 58 | fn crates/game-core/src/combat.rs:56；行锚×1〔名1〕（crates/game-core/src/combat.rs:48） |
| scaledByFloorTier | a.java:7259-7263 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T261 | 1 | fn crates/game-core/src/scene.rs:3618；行锚×2〔名2〕（crates/game-core/src/scene.rs:293） |
| alchemyPriceFor | a.java:7265-7273 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T261 | 1 | fn crates/game-core/src/scene.rs:3608；行锚×2〔名2〕（crates/game-core/src/scene.rs:291） |
| addOption | a.java:7275-7292 | `preset_cases::preset_load_menu_frames_match` / preset-load-menu @T501 | 2 | 人工：crates/game-core/src/menu_family.rs `fn add(&mut self, label_idx: usize, checked: bool)`（OptionList::add = addOption：stringWidth+80 定框宽、首项 32+行高 = a.java:7270-7280；原 28 条行锚系 game-data seed 方言锚误归属） |
| tryOpenChest | a.java:7294-7314 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T312 | 7 | fn crates/game-core/src/scene.rs:1135；行锚×1〔名1〕（crates/game-core/src/scene.rs:1133） |
| applyMerchantOffer | a.java:7316-7479 | `preset_cases::preset_merchant_frames_match` / preset-merchant @T321 | 3 | fn crates/game-core/src/scene.rs:3798；行锚×2〔名2〕（crates/game-core/src/scene.rs:302） |
| pushOfferQueue | a.java:7481-7487 | `preset_cases::preset_floor46_frames_match` / preset-floor46 @T351 | 2 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:3777） |
| browseOfferQueue | a.java:7489-7514 | `fixture_frames::preset_floor5_fixture_frames_match` / preset-floor5 @T621 | 1 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:6123） |
| spendGold | a.java:7516-7525 | `preset_cases::preset_floor45_frames_match` / preset-floor45 @T501 | 1 | fn crates/game-core/src/scene.rs:3787 |
| gainGold | a.java:7527-7532 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T302 | 26 | fn crates/game-core/src/combat.rs:174 |
| pickupItemType | a.java:7534-7649 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 13 | fn crates/game-core/src/scene.rs:5277；行锚×2〔名2〕（crates/game-core/src/scene.rs:5263） |
| floorTier | a.java:7651-7662 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 8 | fn crates/game-core/src/scene.rs:5266；行锚×1〔名1〕（crates/game-core/src/scene.rs:5265） |
| findItemStackIndex | a.java:7664-7675 | `fixture_frames::preset_floor5_fixture_frames_match` / preset-floor5 @T265 | 30 | 人工：crates/game-core/src/combat.rs `pub fn find_item(&self, item_type: u8) -> i32`（doc「findItemStackIndex 忠实移植」，首个同型槽/无则 -1 = a.java:7659-7668；其 Rust 行锚 8485 陈旧，落入 executeScriptInstruction） |
| tryEquip | a.java:7677-7697 | `slot_equip_scene::preset_save_equip_frames_match` / preset-save-equip @T465 | 3 | fn crates/game-core/src/scene.rs:1193; crates/game-core/src/script.rs:155（fn m_082_set_dialog_target） …；行锚×1〔名1〕（crates/game-core/src/scene.rs:1191） |
| equipTierIndex | a.java:7699-7709 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 13 | fn crates/game-core/src/scene.rs:1215（fn m_083_index）；行锚×2〔名2〕（crates/game-core/src/scene.rs:1214） |
| itemTypeToStackIndex | a.java:7711-7725 | `fixture_frames::preset_floor5_fixture_frames_match` / preset-floor5 @T265 | 11 | 人工：crates/game-core/src/scene.rs `fn item_desc_index(&self, t: i32) -> usize`（85→20 / 86→21 / 其余 t-13 = a.java:7707-7716；add_item_to_stack 内联同式）；行锚×1〔名1〕（crates/game-core/src/scene.rs:3137） |
| addItemToItemStack | a.java:7727-7740 | `fixture_frames::preset_floor5_fixture_frames_match` / preset-floor5 @T265 | 1 | 人工：crates/game-core/src/scene.rs `fn add_item_to_stack(&mut self, t: i32)`（追加/叠加次数 + showOverlayMessage = a.java:7722-7731/7733；行锚 7702-7714 漂移落入 itemTypeToStackIndex）。未移植 itemUseTargetType 写（a.java:7732）——无可见效应：拾取路径调用前 已置 overlay_kind_item（crates/game-core/src/scene.rs `self.overlay_kind_item = t`），商人路径浮层同调用内被关闭/替换（2026-10-15 独立审计核）；行锚×2〔名2〕（crates/game-core/src/scene.rs:5345） |
| consumeKeyForDoor | a.java:7742-7768 | `mov_route_scene::preset_floor40_mov_frames_match` / preset-floor40-mov @T261 | 14 | fn crates/game-core/src/combat.rs:153 |
| useItemStack | a.java:7770-7788 | `fixture_frames::fx_fan_floor13_frames_match` / fx-fan-floor13 @T381 | 9 | fn crates/game-core/src/scene.rs:3114；行锚×1〔名1〕（crates/game-core/src/scene.rs:3112） |
| activateItem | a.java:7790-7899 | `preset_cases::preset_item13_frames_match` / preset-item13 @T351 | 11 | fn crates/game-core/src/scene.rs:2897；行锚×4〔名2/稳2〕（crates/game-core/src/scene.rs:2892） |
| m_089 | a.java:7901-7930 | `fixture_frames::fx_fan_floor13_frames_match` / fx-fan-floor13 @T381 | 3 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:3075） |
| killAdjacentAt | a.java:7932-7961 | `preset_cases::preset_itemcymbal_frames_match` / preset-itemcymbal @T401 | 1 | fn crates/game-core/src/scene.rs:3015；行锚×1〔名1〕（crates/game-core/src/scene.rs:3009） |
| m_091 | a.java:7963-8023 | `preset_cases::preset_item13_frames_match` / preset-item13 @T351 | 1 | 行锚×2〔名2〕（crates/game-core/src/layout.rs:326） |
| m_092 | a.java:8025-8038 | `preset_cases::preset_item13_frames_match` / preset-item13 @T401 | 1 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:3472） |
| loadLevelScript | a.java:8044-8072 | `mov_route_scene::preset_floor2_mov_frames_match` / preset-floor2-mov @T261 | 11 | fn crates/game-core/src/scene.rs:1544; crates/game-core/src/script.rs:191；行锚×3〔名3〕（crates/game-core/src/scene.rs:1542） |
| tryRunScene | a.java:8074-8230 | `mov_route_scene::preset_floor2_mov_frames_match` / preset-floor2-mov @T261 | 11 | fn crates/game-core/src/scene.rs:1419；行锚×10〔名6/稳4〕（crates/game-core/src/scene.rs:1417） |
| openDialogPage | a.java:8232-8257 | `mov_route_scene::preset_floor2_mov_frames_match` / preset-floor2-mov @T262 | 9 | fn crates/game-core/src/scene.rs:1567; crates/game-core/src/script.rs:214；行锚×2〔名2〕（crates/game-core/src/scene.rs:1565） |
| cameraAtTarget | a.java:8259-8261 | `fixture_frames::fx_boss74a_kill_hooks_frames_match` / fx-boss74a @T403 | 9 | fn crates/game-core/src/scene.rs:4017 |
| executeScriptInstruction | a.java:8263-8611 | `mov_route_scene::preset_floor2_mov_frames_match` / preset-floor2-mov @T262 | 11 | fn crates/game-core/src/scene.rs:1770; crates/game-core/src/script.rs:266；行锚×34〔seed稳1/名1/稳32〕（crates/game-core/src/enums.rs:152） |
| parseScriptInt | a.java:8613-8619 | `mov_route_scene::preset_floor2_mov_frames_match` / preset-floor2-mov @T262 | 11 | fn crates/game-core/src/scene.rs:1559; crates/game-core/src/script.rs:258；行锚×1〔seed名1〕（crates/game-data/src/script.rs:10） |
| findBlockingEntity | a.java:8621-8635 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T237 | 57 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:5615） |
| m_101 | a.java:8637-8644 | `fixture_frames::fx_knockback_floor43_frames_match` / fx-knockback-floor43 @T305 | 1 | fn crates/game-core/src/scene.rs:4997; crates/game-core/src/scene.rs:4997（fn m_101）；行锚×1〔名1〕（crates/game-core/src/scene.rs:4995） |
| probeCrossFloorSnapshot | a.java:8646-8666 | `fixture_frames::preset_floor49_fixture_frames_match` / preset-floor49 @T951 | 1 | fn crates/game-core/src/scene.rs:1398; crates/game-core/src/scene.rs:1537（fn m_102_current_floor）；行锚×3〔名3〕（crates/game-core/src/scene.rs:1394） |
| easeCameraToTarget | a.java:8668-8673 | `preset_cases::floor51_menu_frames_match` / floor51-menu @T902 | 2 | 人工：crates/game-core/src/scene.rs `// m_103：ease 相机目标 = 目标格`（SEE 臂）+ crates/game-core/src/scene.rs `// m_103：ease 目标 = SEE 目标格`（翻页臂）两处内联，四赋值 = a.java:8663-8666 |
| lockCameraOn | a.java:8675-8714 | `mov_route_scene::preset_floor2_mov_frames_match` / preset-floor2-mov @T261 | 17 | fn crates/game-core/src/scene.rs:4388；行锚×2〔名2〕（crates/game-core/src/scene.rs:1589） |
| stepCameraTowardTarget | a.java:8716-8742 | `mov_route_scene::preset_floor2_mov_frames_match` / preset-floor2-mov @T262 | 17 | fn crates/game-core/src/scene.rs:4416；行锚×1〔名1〕（crates/game-core/src/scene.rs:4414） |
| m_106 | a.java:8744-8876 | `mov_route_scene::preset_floor40_mov_frames_match` / preset-floor40-mov @T411 | 2 | 行锚×9〔名4/稳5〕（crates/game-core/src/entity.rs:466） |
| m_108 | a.java:8886-8910 | `mov_route_scene::preset_floor2_mov_frames_match` / preset-floor2-mov @T619 | 1 | 行锚×2〔名2〕（crates/game-core/src/entity.rs:461） |
| spawnBossEvent | a.java:8912-8921 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T302 | 6 | fn crates/game-core/src/scene.rs:3096；行锚×1〔名1〕（crates/game-core/src/scene.rs:3093） |
| saveGlobalProgress | a.java:8923-8957 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T68 | 4 | 人工：b62d9db：save::GlobalSave 格式层（a.java:8931-8942 写/4463-4474 读）+ BootMachine/GameScene 同构 global 状态（`fn global_snapshot`/`fn note_max_floor`）+ 壳 SKY_WAR@1 轮询持久化（crates/game-desktop/src/main.rs `fn persist_global_if_dirty`）+ catch 语义回写（default_7 ⇒ [1]=true）；回标题触发点（4297-4299）随后续反向移交后置（账本裁定）；行锚×5〔名5〕（crates/game-core/src/paint.rs:165） |
| loadSlotTable | a.java:8959-9005 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 57 | fn crates/game-core/src/menu_family.rs:615；行锚×1〔名1〕（crates/game-core/src/scene.rs:6663） |
| markSlotEntry | a.java:9007-9059 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 4 | 行锚×2〔名2〕（crates/game-core/src/scene.rs:6385） |
| paintSlotTable | a.java:9061-9185 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 57 | 行锚×7〔名4/稳3〕（crates/game-core/src/menu_family.rs:731） |
| saveFloorState | a.java:9187-9251 | `preset_cases::preset_load_menu_frames_match` / preset-load-menu @T341 | 3 | fn crates/game-core/src/scene.rs:1018；行锚×3〔名3〕（crates/game-core/src/save.rs:1） |
| loadFloorSave | a.java:9253-9328 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T220 | 56 | 行锚×3〔名3〕（crates/game-core/src/save.rs:1） |
| deleteRecordStoreQuietly | a.java:9330-9335 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T68 | 7 | 人工·吸收：删库后重建（a.java:9331-9334）；三调用点 a.java:4476（SKY_WAR 读失败→删→saveGlobalProgress 重建）/9027（MOT_IF）/9190（MOT_L{n}）均为删后即写；壳 RMS 以覆盖写吸收删+写（crates/game-desktop/src/main.rs `std::fs::write(&path, v).ok();`）；SKY_WAR 路径的持久化缺口归 saveGlobalProgress（a） |
| closeRecordStoreAndGc | a.java:9337-9369 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 57 | 人工·吸收：RecordStore/流关闭 + 五字段置 null + gc（a.java:9338-9368）；f_byte_arr_27 读者（dispatchModeEntry/loadSlotTable/loadFloorSave）均先 getRecord 赋值再读（a.java:4459/8980/9262）⇒ 无可观测效应 |
| initFloorStates | a.java:9371-9377 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T216 | 58 | 人工：crates/game-core/src/scene.rs `saved_floor_states: vec![None; 56]`（新游戏构造即 56 槽空 = a.java:9366-9370；读档路径 crates/game-core/src/scene.rs `scene.saved_floor_states = save.floor_states.clone()`） |
| m_119 | a.java:9379-9406 | `preset_cases::preset_load_menu_frames_match` / preset-load-menu @T341 | 6 | 行锚×2〔名2〕（crates/game-core/src/scene.rs:991） |
| m_120 | a.java:9408-9411 | `preset_cases::preset_load_menu_frames_match` / preset-load-menu @T341 | 6 | 人工：crates/game-core/src/scene.rs `fn save_floor_state(&mut self)`（u16 BE 写内联为 to_be_bytes = a.java:9403-9404）；行锚×1〔名1〕（crates/game-core/src/scene.rs:1010） |
| loadFloorData | a.java:9413-9453 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T223 | 58 | fn crates/game-core/src/scene.rs:876；行锚×5〔seed名1/名4〕（crates/game-core/src/game_view.rs:1） |
| m_122 | a.java:9455-9683 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T223 | 58 | 行锚×48〔seed稳1/名10/稳37〕（crates/game-core/src/menu_family.rs:842） |
| restoreFloorState | a.java:9685-9687 | `preset_cases::floor2_tour_frames_match` / floor2-tour @T1357 | 1 | 人工：crates/game-core/src/scene.rs `u16::from_be_bytes([d[*p], d[*p + 1]])`（层存档恢复 read_u16 闭包 = a.java:9680，m_122 恢复臂 a.java:9626-9638 内联） |
| spawnPopup | a.java:9697-9736 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 32 | 行锚×4〔名4〕（crates/game-core/src/combat.rs:123） |
| drawPopupLayer | a.java:9738-9797 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T237 | 58 | 行锚×3〔名3〕（crates/game-core/src/popup.rs:1） |
| drawDigitStrip | a.java:9799-9825 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 30 | fn crates/game-core/src/popup.rs:149；行锚×1〔名1〕（crates/game-core/src/popup.rs:145） |
| initParticles | a.java:9827-9837 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 60 | 人工：crates/game-core/src/title.rs `pub fn new() -> Particles`（32 槽粒子数组清零 + 游标 0 = a.java:9822-9830） |
| spawnParticle | a.java:9839-9851 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T73 | 60 | 行锚×1〔名1〕（crates/game-core/src/title.rs:79） |
| updateAndDrawParticles | a.java:9853-9880 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 60 | 行锚×2〔名2〕（crates/game-core/src/title.rs:11） |
| startLoadTask | a.java:9882-9892 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T201 | 58 | 人工：crates/game-core/src/intro.rs `pub fn new(target: i32, call_m000: bool, steps: &[i8]) -> LoadProgress`（loadTargetMode/loadCallInit/步表/计数复位 = a.java:9877-9884；readCountPrefix(15) 由构造期装载吸收）；行锚×1〔名1〕（crates/game-core/src/intro.rs:274） |
| stepLoadTask | a.java:9894-9896 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T201 | 58 | 行锚×1〔名1〕（crates/game-core/src/intro.rs:274） |
| findPath | a.java:9898-9963 | `fixture_frames::fx_bug_ces70_slot_frames_match` / fx-bug-ces70-slot @T303 | 6 | fn crates/game-core/src/pathfind.rs:54; crates/game-core/src/scene.rs:1738 …；行锚×2〔名2〕（crates/game-core/src/pathfind.rs:3） |
| m_134 | a.java:9965-9990 | `fixture_frames::fx_bug_ces70_slot_frames_match` / fx-bug-ces70-slot @T303 | 6 | fn crates/game-core/src/pathfind.rs:151; crates/game-core/src/pathfind.rs:151（fn m_134） |
| predictHpLossVsType | a.java:9992-9996 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T301 | 28 | fn crates/game-core/src/combat.rs:107；行锚×1〔名1〕（crates/game-core/src/scene.rs:4800） |
| effectiveAttackVsType | a.java:9998-10008 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T301 | 29 | fn crates/game-core/src/combat.rs:66；行锚×1〔名1〕（crates/game-core/src/scene.rs:4800） |
| predictBattleHpLoss | a.java:10010-10030 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T301 | 29 | fn crates/game-core/src/combat.rs:79；行锚×1〔名1〕（crates/game-core/src/combat.rs:77） |
| tickBattle | a.java:10032-10165 | `fixture_frames::fx_boss67a_kill_spawn_frames_match` / fx-boss67a @T302 | 25 | fn crates/game-core/src/scene.rs:4825；行锚×3〔名2/稳1〕（crates/game-core/src/scene.rs:326） |
| playBgm | a.java:10167-10215 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 60 | 行锚×3〔名3〕（crates/game-core/src/audio.rs:3） |
| closeAudioPlayer | a.java:10217-10224 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T68 | 60 | 行锚×4〔名4〕（crates/game-core/src/audio.rs:4） |
| randomBelow | a.java:10226-10228 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T73 | 60 | fn crates/game-platform/src/lib.rs:77 |
| drawOutlinedText | a.java:10230-10244 | `preset_cases::floor51_menu_frames_match` / floor51-menu @T577 | 3 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:6171） |
| runLogoAnimation | a.java:10246-10482 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T33 | 60 | 行锚×10〔名5/稳5〕（crates/game-core/src/logo_anim.rs:3） |
| paintLogoAnimation | a.java:10484-10512 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T32 | 60 | 行锚×3〔名3〕（crates/game-core/src/logo_anim.rs:15） |
| registerLogoItem | a.java:10514-10571 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T33 | 60 | 行锚×2〔名2〕（crates/game-core/src/logo_anim.rs:10） |
| modifyLogoItem | a.java:10573-10597 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T42 | 60 | 行锚×1〔名1〕（crates/game-core/src/logo_anim.rs:193） |
| removeLogoItem | a.java:10599-10614 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T37 | 60 | 行锚×3〔名3〕（crates/game-core/src/logo_anim.rs:3） |
| <clinit> | a.java:10886-10888 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T1 | 60 | 行锚×1〔字段1〕（crates/game-core/src/render.rs:476） |

## 3. 黄：已移植 + 无帧级窗口触达（2）

| 方法 | 区间 | 场景触达（无用例） | 移植证据 | 调用点（deobf） | 处置 |
|---|---|---|---|---|---|
| hideNotify | a.java:4502-4514 | **无场景触达** | fn crates/game-core/src/scene.rs:2574; crates/game-desktop/src/main.rs:518；行锚×7〔名7〕（crates/game-core/src/audio.rs:64） |  | 不触达论证（B，机械校验 ✓）：MIDP 生命周期回调（a.java:4502-4513）：类 a 内零调用点，字节码唯一调用者 CMidlet.pauseApp@4（invokevirtual a.hideNotify），另由平台在 Canvas 离屏时回调；oracle 宿主只反射调 startApp、场景脚本只有按键事件（Runner.loadScript `<tick> <keycode> <press|release>`），shim/宿主源码无任何 pauseApp/hideNotify 调用 ⇒ 任何场景（含夹具场景）不可达。**非死代码**（真机来电/切后台必达），属场景驱动面盲区（扩驱动面须改 reference/ 宿主）。语义由壳侧宿主测试覆盖（ab8368a）：crates/game-desktop/src/main.rs `fn in_game_hide_closes_bgm_and_first_key_resumes_game_track`、crates/game-desktop/src/main.rs `fn hide_in_game_menu_keeps_mode_4`、crates/game-desktop/src/main.rs `fn title_hide_resumes_menu_track_in_java_event_order`、crates/game-desktop/src/main.rs `fn hide_in_modes_0_and_21_clears_flag_without_resume`、crates/game-desktop/src/main.rs `fn hide_in_mode3_opens_in_game_menu`（gameMode 3→4 臂 a.java:4504-4507，47169cc 接通 crates/game-core/src/scene.rs `pub fn hide_notify(&mut self)` 后摘 #[ignore]） |
| m_107 | a.java:8878-8884 | **无场景触达** | fn crates/game-core/src/scene.rs:2195; crates/game-core/src/scene.rs:2195（fn m_107） …；行锚×3〔名3〕（crates/game-core/src/scene.rs:2191） | executeScriptInstruction:8573, executeScriptInstruction:8591 | 不触达论证（B，机械校验 ✓）：实体换格（a.java:8878-8884；MVS 臂唯一被调者）：Java 侧**任何场景（含夹具）不可达**——MVS 引理（docs/spec/dead-code-rulings.md DC-2，L1-L5 字节码机械复核 ✓）证 executeScriptInstruction 的 `ldc "MVS"; String.equals` 相等分支死（68 条脚本语料 `MVS` 子串 0 处），m_107 的 2 个调用点（executeScriptInstruction@1001/@1160，均在该臂内）全不可达 ⇒ 驱动面（oracle 场景脚本只有 `<tick> <keycode> <press|release>` 键事件，Runner.loadScript 无脚本注入面）无法驱进。**非移植缺口**：Rust 侧移植账在位（crates/game-core/src/scene.rs `fn m_107`，保真保留死分支）。语义由合成单测覆盖（FX-MVS-A 口径，fixture-plan §0 #10）：crates/game-core/src/scene.rs `mvs_pairs_by_param_low_byte_transfers_and_consumes`（配对+互传+消耗，钉 &255 掩码）、crates/game-core/src/scene.rs `mvs_mismatched_pair_key_leaves_world_untouched`（错配负例）、crates/game-core/src/scene.rs `mvs_two_way_transfer_piles_monsters_at_first_slot_cell`（双向互传，a.java:8561-8590）——语义等价断言齐；边界哨兵 crates/game-core/tests/script_golden.rs `mvs_is_p32_boundary`（should_panic 指引）。【2026-10-16 登记】按 convergence-plan §0.1「黄项带不触达论证」收口，与 hideNotify 同口径（场景驱动面盲区，非死代码判断） |

## 4. 白：无移植账（9）

| 方法 | 区间 | 分账 | 场景触达 | Rust 点名提及 | 论证 / 依据 |
|---|---|---|---|---|---|
| drawLeftTriangleArrow | a.java:8040-8042 | b ☠️ 已裁死 DC-1（字节码复核 ✓：调用点 2 处全不可达） | 无 | 0 | 左向三角箭头（a.java:8034-8036）：调用点 2 处（a.java:5980/5985）均在 paintPlayerSprite walkPhase case 3 道具轮盘臂；JAVAP 中 putfield walkPhase 共 23 处，前导常量 ∈{0,1,2,4,5}（无 3，亦无计算值写入）⇒ walkPhase 永不为 3，case 3 绘制臂与 run() `switch (walkPhase)`（a.java:3565）的 case 3 键处理臂（a.java:3647-3673）皆死代码；裁决 DC-1 |
| addSpark | a.java:9689-9695 | b ☠️ 已裁死 DC-4（字节码复核 ✓：调用点 4 处全不可达） | 无 | 0 | 火花登记（a.java:9683-9689）：调用点 4 处（a.java:3019-3031）均在 paint `switch (gameMode)` case 18（a.java:3003-3031）；gameMode 写点无 18（论证同 m_148 的 JAVAP 普查）⇒ 死代码（completeness 迭代 60 mode18 火花 ☠️）。v2 的唯一移植证据系漂移锚（combat.rs 注「spawnPopup kind 常量（…9679-9686…）」）；裁决 DC-4 |
| m_148 | a.java:10616-10645 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 8 处全不可达） | 无 | 0 | 联网簇·连接关闭（a.java:10610-10639）：调用点 run mode99 臂 a.java:4261、m_151/m_153（簇内）。mode 99 不可达：JAVAP putfield gameMode 54 处，常量 ∈{0-5,7-12,14-17,19-22}，计算来源 menuReturnMode∈{1,4}/loadTargetMode∈{3}/inGameMenuReturnMode←gameMode ⇒ 无 99；另 master-plan §4 范围裁剪（HttpConnection 联网整体剔除）；裁决 DC-3 |
| m_149 | a.java:10647-10649 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 3 处全不可达） | 无 | 0 | 联网簇·响应体 u16 读（a.java:10641-10643）：仅 m_150/m_153 调用（簇内）；簇不可达论证同 m_148；裁决 DC-3 |
| m_150 | a.java:10651-10653 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 2 处全不可达） | 无 | 0 | 联网簇·响应体 u32 读（a.java:10645-10647）：仅 m_153 调用（簇内）；同 m_148；裁决 DC-3 |
| m_151 | a.java:10657-10775 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 1 处全不可达） | 无 | 0 | 联网簇·HTTP POST（a.java:10651-10769）：仅 run() f_bool_31 臂调用（a.java:3229-3242）；JAVAP putfield f_bool_31：<init>/run×2 置 0、唯一置 1 者 m_153（mode99 内）⇒ 不可达；同 m_148；裁决 DC-3 |
| m_152 | a.java:10777-10788 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 3 处全不可达） | 无 | 0 | 联网簇·状态读（a.java:10771-10782）：调用点 run mode99 臂 a.java:4271 与 m_153；同 m_148；裁决 DC-3 |
| m_153 | a.java:10790-10874 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 1 处全不可达） | 无 | 0 | 联网簇·轮询/重试状态机（a.java:10784-10868）：唯一调用点 run mode99 臂 a.java:4267，且该臂先要求 f_bool_31 已真（a.java:4258），而 f_bool_31 唯一置真点就在 m_153 内（自门控）⇒ 即便 mode99 可达也永不执行；同 m_148（2026-10-15 独立审计补强）；裁决 DC-3 |
| m_154 | a.java:10876-10884 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 2 处全不可达） | 无 | 0 | 联网簇·hex 字符解码（a.java:10870-10878）：仅 m_151 调用（a.java:10730）；同 m_148；裁决 DC-3 |

### 4.1 刷账三分账（2026-10-15：基线 23 白 + v2.1 归属治理新暴露）

c 类 = 白色系工具方法论所致（移植存在但漏判 / 无可观测效应无「吸收」口径）；解决手段为归属规则修正或 PORT_NOTES 人工条目（逐条校验 Rust 锚），解决后现色随之变化。b 类 ☠️ = 死代码终裁（裁决书 + 字节码机械复核 + 全场景零触达三重校验；不计判据①赤字）。

| 方法 | 区间 | 来源 | 分账 | 现色 | 场景触达 | 论证 / 依据 |
|---|---|---|---|---|---|---|
| m_106 | a.java:8744-8876 | 基线23 | a 真未移植 → 已移植（现绿） | 绿 | preset-floor40-mov@T411; preset-floor2-mov@T607 | MOV 五参实体路径缓动（a.java:8738-8870）：run() dialogPhase 2 臂调用（a.java:4073）；dialogPhase=2 由 MOV 五参臂在 findBlockingEntity（原 m_100）命中实体时置（a.java:8288-8299）；levelScriptLines（a.java:1124-1193，68 条）含五参 MOV 36 处（脚本 1/2/4/5/7/8/10/12/19/30/57/61），其中 20 处起点（型,格）在 jar sprite{n} 初始布置中有对应实体（如脚本 1 `MOV_72_3_7_1_8` = sprite2 型 72@(3,7)；脚本 12 的 13 处全落 sprite40）⇒ 正常游玩可达。【2026-10-15 已移植】6a37155：crates/game-core/src/scene.rs `fn tick_entity_route(&mut self)`（doc 点名 m_106 a.java:8738-8870），帧级验证 = crates/game-core/tests/mov_route_scene.rs `fn preset_floor2_mov_frames_match` / crates/game-core/tests/mov_route_scene.rs `fn preset_floor40_mov_frames_match`（夹具场景 preset-floor2-mov / preset-floor40-mov）——即 convergence-plan P1 挂起项 dp2=m_106 已落地 |
| m_108 | a.java:8886-8910 | 基线23 | a 真未移植 → 已移植（现绿） | 绿 | preset-floor2-mov@T619 | 实体跨格槽表重登记（a.java:8880-8904）：唯一调用者 m_106（a.java:8806/8848；javap 调用图同）⇒ 随 m_106 可达、随 m_106 一并移植。【2026-10-15 已移植】6a37155：crates/game-core/src/entity.rs `pub fn relocate_slot(grid: &mut CellGrid, table: &EntityTable, idx: usize, from_x: i32, from_y: i32)` |
| saveGlobalProgress | a.java:8923-8957 | 基线23 | a 真未移植 → 已移植（现绿） | 绿 | boot-menu@T68; enter-game@T68 | SKY_WAR 全局进度写（a.java:8917-8951：optionChecked×4 + f_int_arr_35 + f_int_152..154）；调用点 dispatchModeEntry a.java:4299/4478、run a.java:4169（选项切换后）。Rust：save.rs 对 SKY_WAR 既不编码也不解码（只有 parse_floor_save/parse_preset）；game-core 无 I/O（crates/game-core/src/scene.rs `saveGlobalProgress（RMS 侧，无像素效应；game-core 无 I/O）`），boot 恒走 SKY_WAR 读失败回退（crates/game-core/src/paint.rs `RMS "SKY_WAR" 读档失败路径`）；壳 Rms 只改写 MOT_L*/MOT_IF（crates/game-desktop/src/main.rs `stores.push(("MOT_IF@1".into(), mot_if.clone()));`）⇒ 选项/全局旗标跨会话不持久（读写双缺，读臂在 dispatchModeEntry case 21 a.java:4450-4479）——壳层赤字（会话内帧不可见，非铺层赤字）。【2026-10-15 已移植】b62d9db：读写双补 + 壳持久化（见 PORT_NOTES） |
| drawLeftTriangleArrow | a.java:8040-8042 | 基线23 | b ☠️ 已裁死 DC-1（字节码复核 ✓：调用点 2 处全不可达） | 白 | 无 | 左向三角箭头（a.java:8034-8036）：调用点 2 处（a.java:5980/5985）均在 paintPlayerSprite walkPhase case 3 道具轮盘臂；JAVAP 中 putfield walkPhase 共 23 处，前导常量 ∈{0,1,2,4,5}（无 3，亦无计算值写入）⇒ walkPhase 永不为 3，case 3 绘制臂与 run() `switch (walkPhase)`（a.java:3565）的 case 3 键处理臂（a.java:3647-3673）皆死代码；裁决 DC-1 |
| m_107 | a.java:8878-8884 | 基线23 | b ☠️ 已裁死 DC-2（字节码复核 ✓：调用点 2 处全不可达） | 黄 | 无 | 实体换格（a.java:8872-8878）：调用点 2 处（a.java:8567/8585）均在 executeScriptInstruction 的 MVS 臂（a.java:8519-8592，`!var3.equals("MVS")` 之 else，DSL 第 16 码）；levelScriptLines（a.java:1124-1193）68 条脚本 MVS 出现 0 次；executeScriptInstruction 三调用点均传 dialogScriptLine（a.java:4021/8302/8437）；JAVAP：dialogScriptLine 唯一写点 loadLevelScript（aaload levelScriptLines），levelScriptLines 仅 <init> 赋值且无元素写 ⇒ 脚本文本源封闭、MVS 死指令（crates/game-core/src/script.rs `数据域 68 条脚本零使用`）；裁决 DC-2 |
| m_148 | a.java:10616-10645 | 基线23 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 8 处全不可达） | 白 | 无 | 联网簇·连接关闭（a.java:10610-10639）：调用点 run mode99 臂 a.java:4261、m_151/m_153（簇内）。mode 99 不可达：JAVAP putfield gameMode 54 处，常量 ∈{0-5,7-12,14-17,19-22}，计算来源 menuReturnMode∈{1,4}/loadTargetMode∈{3}/inGameMenuReturnMode←gameMode ⇒ 无 99；另 master-plan §4 范围裁剪（HttpConnection 联网整体剔除）；裁决 DC-3 |
| m_149 | a.java:10647-10649 | 基线23 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 3 处全不可达） | 白 | 无 | 联网簇·响应体 u16 读（a.java:10641-10643）：仅 m_150/m_153 调用（簇内）；簇不可达论证同 m_148；裁决 DC-3 |
| m_150 | a.java:10651-10653 | 基线23 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 2 处全不可达） | 白 | 无 | 联网簇·响应体 u32 读（a.java:10645-10647）：仅 m_153 调用（簇内）；同 m_148；裁决 DC-3 |
| m_151 | a.java:10657-10775 | 基线23 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 1 处全不可达） | 白 | 无 | 联网簇·HTTP POST（a.java:10651-10769）：仅 run() f_bool_31 臂调用（a.java:3229-3242）；JAVAP putfield f_bool_31：<init>/run×2 置 0、唯一置 1 者 m_153（mode99 内）⇒ 不可达；同 m_148；裁决 DC-3 |
| m_152 | a.java:10777-10788 | 基线23 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 3 处全不可达） | 白 | 无 | 联网簇·状态读（a.java:10771-10782）：调用点 run mode99 臂 a.java:4271 与 m_153；同 m_148；裁决 DC-3 |
| m_153 | a.java:10790-10874 | 基线23 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 1 处全不可达） | 白 | 无 | 联网簇·轮询/重试状态机（a.java:10784-10868）：唯一调用点 run mode99 臂 a.java:4267，且该臂先要求 f_bool_31 已真（a.java:4258），而 f_bool_31 唯一置真点就在 m_153 内（自门控）⇒ 即便 mode99 可达也永不执行；同 m_148（2026-10-15 独立审计补强）；裁决 DC-3 |
| m_154 | a.java:10876-10884 | 基线23 | b ☠️ 已裁死 DC-3（字节码复核 ✓：调用点 2 处全不可达） | 白 | 无 | 联网簇·hex 字符解码（a.java:10870-10878）：仅 m_151 调用（a.java:10730）；同 m_148；裁决 DC-3 |
| loadPackedImages | a.java:4516-4549 | 基线23 | c 判定存疑（工具方法论） | 绿 | boot-menu@T33; enter-game@T33 | packed 容器解码（a.java:4516-4549）已移植于 game-data packed_png.rs：其 seed 方言锚 `reference/seed/a.java:2657-2676` = seed `private void a(int n)`（序数对齐 = 本方法，原 m_001），v2 按 deobf 行号读成 paint 区间。解决：v2.1 方言规则（无需人工条目）。注：dc470b2 原命名 readCountPrefix，e772fef 名实更正为 loadPackedImages（N-0434，方法体=打包图像容器装载器）疑名实不符——方法体为 packed 图像容器装载（resourceNames[i] 流内 u16 LE 长度前缀 + PNG 逐张 Image.createImage，a.java:4516-4549），非「读计数前缀」；data/naming 不在本任务写入范围，待命名批复核 |
| closeStreamsAndClear | a.java:4634-4661 | 基线23 | c 判定存疑（工具方法论） | 绿 | boot-menu@T33; enter-game@T33 | I/O 清理无可观测效应（a.java:4634-4661；见 PORT_NOTES absorbed）：v2 无「吸收」口径 |
| readShortLE | a.java:6958-6960 | 基线23 | c 判定存疑（工具方法论） | 绿 | fx-boss67a@T223; fx-boss68a@T223 | u16 读原语（a.java:6952-6954）已移植于 game-data lib.rs read::u16_le：其 seed 锚引 `private static short a(InputStream)`（序数对齐 = 本方法）。解决：v2.1 方言 + CFR 签名点名规则。注：方法体为 LE（低字节先），原名 readU16BE 名实不符——【2026-10-15 更正】276ac96 已按 N-0268 名实更正改名 readShortLE（台账证据同步更正），本条随之改键 |
| initCellGrid | a.java:6962-6967 | 基线23 | c 判定存疑（工具方法论） | 绿 | fx-boss67a@T223; fx-boss68a@T223 | CellGrid::new 已移植（a.java:6956-6961；见 PORT_NOTES）：fn 异名且无行锚 |
| findItemStackIndex | a.java:7664-7675 | 基线23 | c 判定存疑（工具方法论） | 绿 | preset-floor5@T265; fx-boss74a@T301 | PlayerCombat::find_item 已移植（a.java:7658-7669；见 PORT_NOTES）：fn 异名 + 行锚陈旧 |
| addItemToItemStack | a.java:7727-7740 | 基线23 | c 判定存疑（工具方法论） | 绿 | preset-floor5@T265 | add_item_to_stack 已移植（a.java:7721-7734；见 PORT_NOTES）：fn 异名 + 行锚漂移 |
| easeCameraToTarget | a.java:8668-8673 | 基线23 | c 判定存疑（工具方法论） | 绿 | preset-floor51@T282; floor51-menu@T902 | 两处内联已移植（a.java:8662-8667；见 PORT_NOTES）：无 fn、注释点名无行锚 |
| initFloorStates | a.java:9371-9377 | 基线23 | c 判定存疑（工具方法论） | 绿 | fx-boss67a@T216; fx-boss68a@T216 | 构造期吸收（a.java:9365-9371；见 PORT_NOTES） |
| m_120 | a.java:9408-9411 | 基线23 | c 判定存疑（工具方法论） | 绿 | preset-load-menu@T341; preset-item21@T401 | 内联于 save_floor_state（a.java:9402-9405；见 PORT_NOTES） |
| restoreFloorState | a.java:9685-9687 | 基线23 | c 判定存疑（工具方法论） | 绿 | floor2-tour@T1357; preset-floor8@T2499 | 内联于层存档恢复（a.java:9679-9681；见 PORT_NOTES） |
| <clinit> | a.java:10886-10888 | 基线23 | c 判定存疑（工具方法论） | 绿 | boot-menu@T1; enter-game@T1 | static{}（a.java:10880-10882）：JAVAP static{} 0-51 putstatic f_int_arr_01 = {0,8192,16384,24576,8462,270,90,8282}，54-93 另造一个 int[8] 后丢弃（无 putstatic）；VF 将前者投影为字段初始化器 a.java:44，v2 不把字段初始化器行归属 <clinit>。Rust：crates/game-core/src/paint.rs `pub const NOKIA_TRANSFORM_TABLE: [i32; 8] = [0, 8192, 16384, 24576, 8462, 270, 90, 8282];`。解决：v2.1 字段初始化器归属规则 |
| addSpark | a.java:9689-9695 | 修正暴露 | b ☠️ 已裁死 DC-4（字节码复核 ✓：调用点 4 处全不可达） | 白 | 无 | 火花登记（a.java:9683-9689）：调用点 4 处（a.java:3019-3031）均在 paint `switch (gameMode)` case 18（a.java:3003-3031）；gameMode 写点无 18（论证同 m_148 的 JAVAP 普查）⇒ 死代码（completeness 迭代 60 mode18 火花 ☠️）。v2 的唯一移植证据系漂移锚（combat.rs 注「spawnPopup kind 常量（…9679-9686…）」）；裁决 DC-4 |
| createSolidImage | a.java:4605-4614 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | fx-boss67a@T223; fx-boss68a@T223 | 结果被丢弃的纯函数（a.java:4605-4614；见 PORT_NOTES absorbed）；v2 证据系漂移锚 |
| releaseSceneImages | a.java:4714-4728 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | boot-menu@T71; enter-game@T71 | 内存释放被构造期装载吸收（a.java:4714-4728；见 PORT_NOTES）；v2 证据系漂移锚 |
| resetNewGameState | a.java:7197-7225 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | enter-game@T410; floor51-menu@T570 | 新游戏复位分散移植（a.java:7191-7219；见 PORT_NOTES）；v2 证据系漂移锚 + seed 误读 |
| addOption | a.java:7275-7292 | v2 误读证据消散 | c 判定存疑（工具方法论） | 绿 | preset-load-menu@T501; menu-sweep@T581 | OptionList::add 已移植（a.java:7269-7286；见 PORT_NOTES）；v2 证据系 28 条 seed 误读。【2026-10-16 更正】origin 修正暴露→v2 误读证据消散（lead 裁决：改登记不改定义，机械复算为真值源）：现机械复算 v2=白（28 条 seed 误读已不复算）、v2.1 信号空（fn/refs 皆无）⇒ 已不满足『修正暴露 = v2 非白 ∧ v2.1 空』定义；移植账 = PORT_NOTES 人工（OptionList::add）。附记：menu_family.rs:1157 的 addOption 注释引 a.java:9911-9926 系漂移锚（真身 a.java:7269-7286），不构成 v2.1 信号，待校正 |
| itemTypeToStackIndex | a.java:7711-7725 | v2.1 锚已落地 | c 判定存疑（工具方法论） | 绿 | preset-floor5@T265; fx-fan-floor13@T341 | item_desc_index 已移植（a.java:7705-7719；见 PORT_NOTES）；v2 证据系漂移锚。【2026-10-16 更正】origin 修正暴露→v2.1 锚已落地（lead 裁决：改登记不改定义）：v2.1 名锚 crates/game-core/src/scene.rs `itemTypeToStackIndex`（a.java:7705-7719 语义，scene.rs:3137）注册（refs=〔名〕非空）⇒ 定义排除正确，机械复算为准 |
| deleteRecordStoreQuietly | a.java:9330-9335 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | boot-menu@T68; enter-game@T68 | 删后即写被壳覆盖写吸收（a.java:9324-9329；见 PORT_NOTES）；v2 证据系漂移锚 |
| closeRecordStoreAndGc | a.java:9337-9369 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | boot-menu@T91; fx-boss67a@T151 | RMS 清理无可观测效应（a.java:9331-9363；见 PORT_NOTES）；v2 证据系漂移锚 |
| initParticles | a.java:9827-9837 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | boot-menu@T71; enter-game@T71 | Particles::new 已移植（a.java:9821-9831；见 PORT_NOTES）；v2 证据系漂移锚 |

## 5. 臂级缺口：Rust `todo!` 显式挡路点（方法级绿/黄内的未移植分支）

- crates/game-core/src/scene.rs:5855：paintEntityLayer 空图标签分支：全语料不可达死码（evidence-todos.md T-3）

### 5.1 无 `todo!` 标记的臂级缺口（人工登记 ARM_GAPS，锚逐条校验；修复后锚失效即 ⚠ 提示销账）

- interactWithCell（a.java:5363-5541，现色 绿）：型 76/82 交互臂未移植——crates/game-core/src/scene.rs `_ => {} // 76/82：后续批次` 吞掉（无 todo! 标记，语义待考）。【2026-10-15 销账并改锚】原登记的型 10/11/12 撞击臂已移植（ee52191，a.java:5436-5456 逐值；fx_firebump_floor13_frames_match 解禁即绿，T301 钉点）——原锚 `_ => {} // 11 / 76/82` 失效即销账信号。【2026-10-16 更正】Java interactWithCell 无 case 76/82（default: continue=bump 放行），Rust `_ => {}` 已忠实——原“ARM_GAPS 在账”按 evidence-todos.md T-1 附带登记销账

## 6. 刷新前后对照（v1 伪名口径 → v2.1 逐方法）

v1 取名 bug：`l.split()[-1].split("(")[0]` ⇒ ≥2 参方法被吞成末参类型伪名，构造器成 `a`，`<clinit>` 漏收。下表按伪名列出其吞掉的真实方法与现色。

| v1 名（v1 色） | 真实方法 → 现色 |
|---|---|
| `a`（白） | <init>→绿 |
| `activateItem`（白） | activateItem→绿 |
| `activateMenuItem`（白） | activateMenuItem→绿 |
| `addItemToItemStack`（绿） | addItemToItemStack→绿 |
| `addMenuItem`（白） | addMenuItem→绿 |
| `advanceBobPhase`（白） | advanceBobPhase→绿 |
| `advanceEntityFrames`（白） | advanceEntityFrames→绿 |
| `advanceScriptWalk`（绿） | advanceScriptWalk→绿 |
| `alchemyPriceFor`（白） | alchemyPriceFor→绿 |
| `applyHpDelta`（白） | applyHpDelta→绿 |
| `applyMerchantOffer`（白） | applyMerchantOffer→绿 |
| `applyStepCellEffects`（白） | applyStepCellEffects→绿 |
| `boolean);`（白） | paintWrappedText→绿, paintEntityLayer→绿, changeFloor→绿, tryEquip→绿, tryRunScene→绿, startLoadTask→绿, predictHpLossVsType→绿, predictBattleHpLoss→绿 |
| `buildMinimap`（绿） | buildMinimap→绿 |
| `byte);`（白） | showOverlayMessage→绿, drawLeftTriangleArrow→白 |
| `cameraAtTarget`（绿） | cameraAtTarget→绿 |
| `closeAudioPlayer`（白） | closeAudioPlayer→绿 |
| `consumeKeyForDoor`（绿） | consumeKeyForDoor→绿 |
| `deleteRecordStoreQuietly`（白） | deleteRecordStoreQuietly→绿 |
| `drawParallaxBackdrop`（绿） | drawParallaxBackdrop→绿 |
| `drawPopupLayer`（白） | drawPopupLayer→绿 |
| `easeAfterimages`（绿） | easeAfterimages→绿 |
| `effectiveAttackVsType`（白） | effectiveAttackVsType→绿 |
| `findFloorGateEntity`（白） | findFloorGateEntity→绿 |
| `findItemStackIndex`（绿） | findItemStackIndex→绿 |
| `flipOverlayPage`（绿） | flipOverlayPage→绿 |
| `floorTier`（白） | floorTier→绿 |
| `handleFieldInput`（绿） | handleFieldInput→绿 |
| `handleOverlayKey`（白） | handleOverlayKey→绿 |
| `hideNotify`（白） | hideNotify→黄 |
| `initEntityTables`（白） | initEntityTables→绿 |
| `int);`（白） | drawImageClipped→绿, blit5Clip→绿, drawImageWithNokiaTransform→绿, drawEdgePatch→绿, createSolidImage→绿, dimImage→绿, drawIconStrip→绿, layoutFullText→绿, layoutWrappedText→绿, placePlayerAtCell→绿, interactWithCell→绿, stepOffStairs→绿, paintPlayerSprite→绿, paintStatusBar→绿, paintKeySlot→绿, paintHudPanel→绿, paintTitledBox→绿, paintHudFrame→绿, paintBoxFrame→绿, paintMiniFrame→绿, paintNumber→绿, dispatchSpriteSize→绿, dispatchAnimFields→绿, spawnEntity→绿, removeGroupAtCellByType→绿, removeSlotAt→绿, isCellWalkable→绿, m_062→绿, paintTileLayer→绿, setCameraClamped→绿, scaledByFloorTier→绿, pushOfferQueue→绿, gainGold→绿, equipTierIndex→绿, killAdjacentAt→绿, executeScriptInstruction→绿, findBlockingEntity→绿, m_101→绿, probeCrossFloorSnapshot→绿, easeCameraToTarget→绿, m_107→黄, m_108→绿, addSpark→白, spawnPopup→绿, drawDigitStrip→绿, spawnParticle→绿, findPath→绿, m_134→绿, playBgm→绿, runLogoAnimation→绿, registerLogoItem→绿, modifyLogoItem→绿 |
| `int[]);`（白） | drawOutlinedText→绿 |
| `itemTypeToStackIndex`（绿） | itemTypeToStackIndex→绿 |
| `java.io.IOException;`（白） | readShortLE→绿 |
| `java.lang.String);`（白） | parseScriptInt→绿 |
| `keyPressed`（白） | keyPressed→绿 |
| `keyReleased`（白） | keyReleased→绿 |
| `loadFloorData`（白） | loadFloorData→绿 |
| `loadFloorSave`（绿） | loadFloorSave→绿 |
| `loadLevelScript`（绿） | loadLevelScript→绿 |
| `loadSlotTable`（绿） | loadSlotTable→绿 |
| `lockCameraOn`（绿） | lockCameraOn→绿 |
| `m_000`（绿） | dispatchModeEntry→绿 |
| `m_001`（绿） | loadPackedImages→绿 |
| `m_008`（白） | closeStreamsAndClear→绿 |
| `m_025`（绿） | m_025→绿 |
| `m_059`（绿） | initCellGrid→绿 |
| `m_068`（绿） | resetNewGameState→绿 |
| `m_072`（白） | addOption→绿 |
| `m_076`（绿） | browseOfferQueue→绿 |
| `m_089`（白） | m_089→绿 |
| `m_091`（绿） | m_091→绿 |
| `m_092`（绿） | m_092→绿 |
| `m_106`（黄） | m_106→绿 |
| `m_113`（白） | paintSlotTable→绿 |
| `m_117`（绿） | closeRecordStoreAndGc→绿 |
| `m_118`（绿） | initFloorStates→绿 |
| `m_119`（绿） | m_119→绿 |
| `m_120`（白） | m_120→绿 |
| `m_122`（绿） | m_122→绿 |
| `m_123`（白） | restoreFloorState→绿 |
| `m_128`（白） | initParticles→绿 |
| `m_132`（白） | stepLoadTask→绿 |
| `m_148`（白） | m_148→白 |
| `m_149`（白） | m_149→白 |
| `m_150`（白） | m_150→白 |
| `m_151`（白） | m_151→白 |
| `m_152`（白） | m_152→白 |
| `m_153`（白） | m_153→白 |
| `m_154`（白） | m_154→白 |
| `markEntityDesRemoved`（绿） | markEntityDesRemoved→绿 |
| `markEntityRemoved`（绿） | markEntityRemoved→绿 |
| `markSlotEntry`（白） | markSlotEntry→绿 |
| `openDialogPage`（绿） | openDialogPage→绿 |
| `paint`（白） | paint→绿 |
| `paintLogoAnimation`（白） | paintLogoAnimation→绿 |
| `paintSoftkeyBar`（绿） | paintSoftkeyBar→绿 |
| `parseHexColor`（白） | parseHexColor→绿 |
| `pickupItemType`（绿） | pickupItemType→绿 |
| `pullEntityFromCell`（绿） | pullEntityFromCell→绿 |
| `randomBelow`（白） | randomBelow→绿 |
| `rebuildWalkability`（绿） | rebuildWalkability→绿 |
| `releaseSceneImages`（白） | releaseSceneImages→绿 |
| `removeLogoItem`（绿） | removeLogoItem→绿 |
| `resetMenuLayout`（白） | resetMenuLayout→绿 |
| `run`（白） | run→绿 |
| `runIntroViewer`（白） | runIntroViewer→绿 |
| `saveFloorState`（白） | saveFloorState→绿 |
| `saveGlobalProgress`（绿） | saveGlobalProgress→绿 |
| `scaleEnemyStats`（绿） | scaleEnemyStats→绿 |
| `showNotify`（白） | showNotify→绿 |
| `skipTextToLineEnd`（绿） | skipTextToLineEnd→绿 |
| `sortEntitiesByY`（白） | sortEntitiesByY→绿 |
| `spawnBossEvent`（绿） | spawnBossEvent→绿 |
| `spendGold`（白） | spendGold→绿 |
| `startNewGameLoad`（白） | startNewGameLoad→绿 |
| `stepCameraTowardTarget`（绿） | stepCameraTowardTarget→绿 |
| `tickBattle`（绿） | tickBattle→绿 |
| `tryOpenChest`（白） | tryOpenChest→绿 |
| `tryStep`（绿） | tryStep→绿 |
| `updateAndDrawParticles`（绿） | updateAndDrawParticles→绿 |
| `useItemStack`（绿） | useItemStack→绿 |
| `None`（未收） | <clinit>→绿 |

逐方法迁移计数（v1 色 → 现色）：未收→绿 1，绿→绿 44，白→绿 106，白→白 9，白→黄 2，黄→绿 1

### 6.1 v1 白项去向（逐真实方法）

- **账面滞后 → 绿 106**：<init>, paint, run, showNotify, drawImageClipped, blit5Clip, drawImageWithNokiaTransform, drawEdgePatch, createSolidImage, dimImage, closeStreamsAndClear, keyPressed, keyReleased, resetMenuLayout, addMenuItem, releaseSceneImages, activateMenuItem, drawIconStrip, runIntroViewer, showOverlayMessage, handleOverlayKey, layoutFullText, layoutWrappedText, paintWrappedText, parseHexColor, applyHpDelta, placePlayerAtCell, interactWithCell, applyStepCellEffects, stepOffStairs, paintPlayerSprite, paintStatusBar, paintKeySlot, paintHudPanel, paintTitledBox, paintHudFrame, paintBoxFrame, paintMiniFrame, paintNumber, initEntityTables, dispatchSpriteSize, dispatchAnimFields, spawnEntity, removeGroupAtCellByType, removeSlotAt, advanceBobPhase, paintEntityLayer, sortEntitiesByY, advanceEntityFrames, readShortLE, isCellWalkable, m_062, paintTileLayer, setCameraClamped, findFloorGateEntity, changeFloor, startNewGameLoad, scaledByFloorTier, alchemyPriceFor, addOption, tryOpenChest, applyMerchantOffer, pushOfferQueue, spendGold, gainGold, floorTier, tryEquip, equipTierIndex, activateItem, m_089, killAdjacentAt, tryRunScene, executeScriptInstruction, parseScriptInt, findBlockingEntity, m_101, probeCrossFloorSnapshot, easeCameraToTarget, m_108, markSlotEntry, paintSlotTable, saveFloorState, deleteRecordStoreQuietly, m_120, loadFloorData, restoreFloorState, spawnPopup, drawPopupLayer, drawDigitStrip, initParticles, spawnParticle, startLoadTask, stepLoadTask, findPath, m_134, predictHpLossVsType, effectiveAttackVsType, predictBattleHpLoss, playBgm, closeAudioPlayer, randomBelow, drawOutlinedText, runLogoAnimation, paintLogoAnimation, registerLogoItem, modifyLogoItem
- **账面滞后 → 黄 2**：hideNotify, m_107
- **仍白 9**（分账见 §4/§4.1）：drawLeftTriangleArrow〔b〕, addSpark〔b〕, m_148〔b〕, m_149〔b〕, m_150〔b〕, m_151〔b〕, m_152〔b〕, m_153〔b〕, m_154〔b〕

v1 绿/黄 → 现色变色（v1 过账或口径差，逐条）：m_106（黄→绿）

## 7. 前任 v2 口径 → v2.1 差异（同一份 Rust 树与触达快照复算）

| 方法 | v2 色 | 现色 | v2 行锚（朴素归属） | v2.1 行锚 | 人工条目 |
|---|---|---|---|---|---|
| loadPackedImages | 白 | 绿 | 0 | 1 | ported |
| createSolidImage | 绿 | 绿 | 2 | 0 | absorbed |
| closeStreamsAndClear | 白 | 绿 | 0 | 0 | absorbed |
| releaseSceneImages | 绿 | 绿 | 2 | 0 | absorbed |
| spawnEntity | 绿 | 绿 | 5 | 0 | — |
| readShortLE | 白 | 绿 | 0 | 1 | — |
| initCellGrid | 白 | 绿 | 0 | 0 | ported |
| resetNewGameState | 绿 | 绿 | 3 | 0 | ported |
| addOption | 白 | 绿 | 0 | 0 | ported |
| findItemStackIndex | 白 | 绿 | 0 | 0 | ported |
| consumeKeyForDoor | 绿 | 绿 | 2 | 0 | — |
| cameraAtTarget | 绿 | 绿 | 1 | 0 | — |
| easeCameraToTarget | 绿 | 绿 | 1 | 0 | ported |
| deleteRecordStoreQuietly | 绿 | 绿 | 2 | 0 | absorbed |
| closeRecordStoreAndGc | 绿 | 绿 | 2 | 0 | absorbed |
| initFloorStates | 白 | 绿 | 0 | 0 | ported |
| restoreFloorState | 绿 | 绿 | 1 | 0 | ported |
| addSpark | 黄 | 白 | 2 | 0 | — |
| initParticles | 绿 | 绿 | 1 | 0 | ported |
| m_134 | 绿 | 绿 | 6 | 0 | — |
| predictBattleHpLoss | 绿 | 绿 | 0 | 1 | — |
| <clinit> | 白 | 绿 | 0 | 1 | — |

