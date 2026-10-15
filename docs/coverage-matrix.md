# 覆盖矩阵（层次一账本，机器生成）

> 生成：`python3 tools/cover_matrix.py`（v2.1，2026-10-15 刷账：v2 重写 + v2.1 行锚归属治理；v1 = 迭代 72）。
> 触达数据：`python3 tools/method_reach.py` → `tools/method_reach/snapshot.json`。
> **请勿手改本文件**——改判定请改 tools/cover_matrix.py 的 PORT_NOTES/TRIAGE（逐条带证据，脚本校验锚点）。
> 判据文本见 [docs/spec/completeness.md](spec/completeness.md)「覆盖矩阵刷账（2026-10-15）」。

## 汇总：绿 143 / 黄 7 / 白 13（共 163 方法）

- 白项分账（现白 13）：a 真未移植 3 / b 不触达论证候选 10 / 未分账 0
- 刷账三分账（§4.1）：基线 23 白 = a 3 / b 9 / c 11（c 已解决 → 现绿 10 / 黄 1）；v2.1 归属治理新暴露 9 项 = b 1 / c 8（c 经核验恢复移植账）
- 黄项：7（其中 3 项被无对拍用例的场景触达 = 补用例即可转绿；4 项无任何场景触达，待不触达论证——convergence-plan §0.1）
- 口径复算（同一份 Rust 树）：v2 前任口径 绿 133 / 黄 7 / 白 23（差异见 §7）；v1 旧账（伪名口径，103 名）绿 44 / 黄 1 / 白 58（对照见 §6）

三色判定（逐方法，机械）：**绿** = 有移植账 ∧ 被帧级对拍用例窗口内的 Java 执行触达；**黄** = 有移植账 ∧ 无帧级窗口触达；**白** = 无移植账。**Stage A 判据（convergence-plan §0.1）：白项清零、黄项带不触达论证**。方法级绿 ≠ 方法内每个分支已验——臂级缺口见 §5（Rust `todo!` 挡路点）与 type-closure.md；绿也 ≠ 副作用已验：帧 sha 不含 RMS/音频，此类方法的绿只证明 Java 执行落在已验窗口内。

## 0. 移植账归属规则与自检（v2.1）

- 行锚方言：显式 `seed/a.java` 或 seed 方言文件（含显式 seed 锚的文件）的裸锚 → reference/seed/a.java；其余 → deobf。seed 与 deobf 以方法序数对齐（运行时校验 163 个、逐序参数类型一致）。
- 名证锚：锚所在行点名 Java 方法（现名/台账别名/引用签名）⇒ 只归属被点名者，容差 ±20 行；稳健锚：未点名 ⇒ 锚外扩 20 行仍须落在 VF 时代 15 个 deobf 版本（996a29d 起 + 工作区）方法区间的交集内；字段初始化器行归 <clinit>/<init>（不外扩）。
- 行锚精度自检（名证锚 430 条到被点名方法区间的最近距离）：0 行 324，1-5 行 10，6-10 行 4，11-20 行 8，21-50 行 3，>50 行 81——>20 行者多为「点名调用者、引用别处行」的语境锚，不予归属；v2 朴素口径（锚落区间即算）把漂移锚记给邻近小方法，§7 列出全部差异。
- 移植扫描范围：crates/{game-core,game-data,game-platform,game-desktop}/src（game-oracle 为差分宿主，不计移植证据）。

## 1. 对拍用例 → 场景 → 窗口（crates/game-oracle/tests 机械解析）

| 用例 | 场景（A-trace） | 窗口 | FRAME 对照 | 验证触达方法数 |
|---|---|---|---|---|
| `boot_replay::boot_menu_frames_match_tick_by_tick` | menu-sweep | T1-1831 | FRAME 一致 | 45 |
| `boot_replay::enter_game_frames_match_tick_by_tick` | enter-game | T1-380 | FRAME 一致 | 37 |
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
| `preset_cases::preset_floor10_frames_match` | preset-floor10 | T1-1410 | FRAME 一致 | 84 |
| `preset_cases::preset_floor46_frames_match` | preset-floor46 | T1-1700 | FRAME 一致 | 85 |
| `preset_cases::preset_floor41_frames_match` | preset-floor41 | T1-1900 | FRAME 一致 | 102 |
| `preset_cases::preset_floor44_frames_match` | preset-floor44 | T1-1600 | FRAME 一致 | 99 |
| `preset_cases::preset_floor45_frames_match` | preset-floor45 | T1-1700 | FRAME 一致 | 97 |
| `preset_cases::preset_floor48_frames_match` | preset-floor48 | T1-3600 | FRAME 一致 | 95 |
| `preset_cases::preset_floor14_frames_match` | preset-floor14 | T1-4890 | FRAME 一致 | 85 |
| `preset_cases::preset_floor38_frames_match` | preset-floor38 | T1-6150 | FRAME 一致 | 92 |
| `preset_cases::preset_floor47_frames_match` | preset-floor47 | T1-1450 | FRAME 一致 | 85 |
| `preset_cases::preset_floor34_frames_match` | preset-floor34 | T1-3130 | FRAME 一致 | 87 |
| `preset_cases::preset_floor10_nw_frames_match` | preset-floor10 | T1-1410 | FRAME 一致 | 84 |
| `preset_cases::preset_floor10b_frames_match` | preset-floor10b | T1-950 | FRAME 一致 | 84 |
| `preset_cases::preset_floor17_frames_match` | preset-floor17 | T1-950 | FRAME 一致 | 86 |
| `preset_cases::preset_floor42_frames_match` | preset-floor42 | T1-1700 | FRAME 一致 | 93 |
| `preset_cases::preset_floor43_frames_match` | preset-floor43 | T1-1500 | FRAME 一致 | 92 |
| `preset_cases::preset_alchemy_frames_match` | preset-alchemy | T1-700 | FRAME 一致 | 87 |

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

场景触达（JDI 实测，全程 = 各相位并集取最小 t_hit；相位起点 = 1 ∪ 用例窗口起点）：

| 场景 | 帧数 | 结束 | 相位 | 触达方法数 | A-trace 对照 | 帧级用例 |
|---|---|---|---|---|---|---|
| boot-menu | 150 | reason=max-ticks ticks=150 | 1 | 39 | 无 A-trace | — |
| enter-game | 500 | reason=max-ticks ticks=500 | 1 | 82 | equal | ✓ |
| floor1-to-floor2 | 1500 | reason=max-ticks ticks=1500 | 1 | 94 | equal | ✓ |
| floor1-tour | 8000 | reason=max-ticks ticks=8000 | 1 | 92 | equal | ✓ |
| floor2-tour | 2000 | reason=max-ticks ticks=2000 | 1 | 101 | equal | ✓ |
| floor51-menu | 9800 | reason=max-ticks ticks=9800 | 1 | 96 | equal | ✓ |
| gameplay-floor1 | 3000 | reason=max-ticks ticks=3000 | 1 | 87 | equal | ✓ |
| menu-sweep | 1832 | reason=all-threads-exited ticks=1832 | 1 | 45 | equal | ✓ |
| preset-alchemy | 700 | reason=max-ticks ticks=700 | 1 | 87 | equal | ✓ |
| preset-ending | 1800 | reason=max-ticks ticks=1800 | 1 | 96 | equal | — |
| preset-floor10 | 1410 | reason=max-ticks ticks=1410 | 1 | 84 | equal | ✓ |
| preset-floor10b | 950 | reason=max-ticks ticks=950 | 1 | 84 | equal | ✓ |
| preset-floor14 | 4890 | reason=max-ticks ticks=4890 | 1 | 85 | equal | ✓ |
| preset-floor17 | 950 | reason=max-ticks ticks=950 | 1 | 86 | equal | ✓ |
| preset-floor2 | 600 | reason=max-ticks ticks=600 | 1 | 82 | equal | ✓ |
| preset-floor34 | 3130 | reason=max-ticks ticks=3130 | 1 | 87 | equal | ✓ |
| preset-floor35 | 1200 | reason=max-ticks ticks=1200 | 1 | 100 | equal | ✓ |
| preset-floor38 | 6150 | reason=max-ticks ticks=6150 | 1 | 92 | equal | ✓ |
| preset-floor41 | 1900 | reason=max-ticks ticks=1900 | 1 | 102 | equal | ✓ |
| preset-floor42 | 1700 | reason=max-ticks ticks=1700 | 1 | 93 | equal | ✓ |
| preset-floor43 | 1500 | reason=max-ticks ticks=1500 | 1 | 92 | equal | ✓ |
| preset-floor44 | 1600 | reason=max-ticks ticks=1600 | 1 | 99 | equal | ✓ |
| preset-floor45 | 1700 | reason=max-ticks ticks=1700 | 1 | 97 | equal | ✓ |
| preset-floor46 | 1700 | reason=max-ticks ticks=1700 | 1 | 85 | equal | ✓ |
| preset-floor47 | 1450 | reason=max-ticks ticks=1450 | 1 | 85 | equal | ✓ |
| preset-floor48 | 3600 | reason=max-ticks ticks=3600 | 1 | 95 | equal | ✓ |
| preset-floor49 | 3400 | reason=max-ticks ticks=3400 | 1 | 93 | equal | — |
| preset-floor5 | 1200 | reason=max-ticks ticks=1200 | 1 | 89 | equal | — |
| preset-floor50 | 300 | reason=max-ticks ticks=300 | 1 | 69 | equal | — |
| preset-floor51 | 600 | reason=max-ticks ticks=600 | 1 | 97 | equal | — |
| preset-floor8 | 8710 | reason=max-ticks ticks=8710 | 1 | 96 | equal | — |
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
| prologue-dialog | 9800 | reason=max-ticks ticks=9800 | 1 | 96 | equal | ✓ |
| prologue-patient | 21000 | reason=max-ticks ticks=21000 | 1 | 96 | 无 A-trace | — |
| walk-pure | 4000 | reason=max-ticks ticks=4000 | 1 | 91 | equal | ✓ |

## 2. 绿：已移植 + 帧级对拍窗口触达（143）

| 方法 | 区间 | 首个验证（用例 / 场景 @t_hit） | 验证用例数 | 移植证据 |
|---|---|---|---|---|
| <init> | a.java:436-2331 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T1 | 36 | 行锚×52〔seed稳3/字段3/稳46〕（crates/game-core/src/audio.rs:26） |
| paint | a.java:2333-3227 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T1 | 37 | fn crates/game-core/src/intro.rs:203; crates/game-core/src/intro.rs:312 …；行锚×56〔名37/稳19〕（crates/game-core/src/intro.rs:1） |
| run | a.java:3229-4290 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T1 | 36 | 行锚×109〔名51/稳58〕（crates/game-core/src/audio.rs:15） |
| m_000 | a.java:4292-4493 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T1 | 37 | 行锚×43〔名33/稳10〕（crates/game-core/src/audio.rs:14） |
| showNotify | a.java:4495-4500 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T1 | 36 | fn crates/game-desktop/src/main.rs:543；行锚×3〔名3〕（crates/game-desktop/src/lifecycle.rs:6） |
| m_001 | a.java:4516-4549 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T33 | 37 | 行锚×1〔seed名1〕（crates/game-data/src/packed_png.rs:17） |
| drawImageClipped | a.java:4551-4555 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T68 | 37 | fn crates/game-core/src/paint.rs:42；行锚×1〔名1〕（crates/game-core/src/paint.rs:40） |
| blit5Clip | a.java:4557-4561 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T223 | 34 | fn crates/game-core/src/scene.rs:5687（fn m_003_clip5）；行锚×1〔名1〕（crates/game-core/src/scene.rs:5686） |
| drawImageWithNokiaTransform | a.java:4563-4565 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T68 | 37 | fn crates/game-core/src/paint.rs:35；行锚×1〔名1〕（crates/game-core/src/paint.rs:34） |
| drawEdgePatch | a.java:4567-4603 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 36 | fn crates/game-core/src/menu_family.rs:115；行锚×1〔名1〕（crates/game-core/src/menu_family.rs:112） |
| m_006 | a.java:4605-4614 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T223 | 34 | 人工·吸收：静态纯函数（造 32×32 纯色 RGB 图，a.java:4606-4613）；仅有的两个调用点 loadFloorData a.java:9420-9421 结果被丢弃（javap：invokestatic m_006 后紧跟 pop）⇒ 无可观测效应 |
| dimImage | a.java:4616-4632 | `boot_replay::enter_game_frames_match_tick_by_tick` / enter-game @T102 | 35 | fn crates/game-core/src/intro.rs:32；行锚×1〔名1〕（crates/game-core/src/intro.rs:30） |
| m_008 | a.java:4634-4661 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T33 | 37 | 人工·吸收：流关闭 + 三字段置 null（a.java:4635-4660）；javap 字段普查：f_InputStream_00/f_OutputStream_00/f_InputStream_01 读写者仅 m_001/m_008/m_139/m_148/m_151/<init>（I/O 方法）⇒ 无游戏态效应；Rust 以内存切片解码无流可关（crates/game-data/src/packed_png.rs `pub fn parse(data: &'a [u8], image_count: usize) -> Result<Self, Error>`） |
| keyPressed | a.java:4663-4676 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 37 | 行锚×11〔名11〕（crates/game-core/src/audio.rs:65） |
| keyReleased | a.java:4678-4680 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T73 | 37 | 行锚×4〔名4〕（crates/game-core/src/scene.rs:295） |
| resetMenuLayout | a.java:4682-4694 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 37 | fn crates/game-core/src/scene.rs:2040；行锚×2〔名2〕（crates/game-core/src/scene.rs:2039） |
| addMenuItem | a.java:4696-4712 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 37 | fn crates/game-core/src/scene.rs:2052；行锚×3〔名3〕（crates/game-core/src/scene.rs:433） |
| releaseSceneImages | a.java:4714-4728 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 36 | 人工·吸收：图像容器/entityTypeImage 置 null 释放内存（a.java:4715-4727）；读者在读前经 m_001 判空重载（a.java:4519）或 initEntityTables 重建 entityTypeImage（javap：put 仅 initEntityTables/releaseSceneImages）；Rust 构造期全量装载不释放（crates/game-core/src/scene.rs `懒加载资源容器（Rust 构造期全量装载，等价 no-op）`） |
| activateMenuItem | a.java:4730-4792 | `preset_cases::preset_load_menu_frames_match` / preset-load-menu @T301 | 3 | fn crates/game-core/src/scene.rs:2169；行锚×1〔名1〕（crates/game-core/src/scene.rs:2165） |
| drawIconStrip | a.java:4794-4816 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 37 | 行锚×3〔名3〕（crates/game-core/src/title.rs:6） |
| runIntroViewer | a.java:4818-4896 | `boot_replay::enter_game_frames_match_tick_by_tick` / enter-game @T102 | 3 | 行锚×3〔名3〕（crates/game-core/src/intro.rs:1） |
| showOverlayMessage | a.java:4898-4923 | `preset_cases::preset_merchant_frames_match` / preset-merchant @T261 | 17 | fn crates/game-core/src/scene.rs:1090；行锚×2〔名2〕（crates/game-core/src/scene.rs:359） |
| handleOverlayKey | a.java:4925-4938 | `preset_cases::preset_merchant_frames_match` / preset-merchant @T262 | 14 | 行锚×2〔名2〕（crates/game-core/src/scene.rs:3517） |
| layoutFullText | a.java:4940-4942 | `boot_replay::enter_game_frames_match_tick_by_tick` / enter-game @T102 | 20 | fn crates/game-core/src/scene.rs:1123；行锚×1〔名1〕（crates/game-core/src/scene.rs:1117） |
| layoutWrappedText | a.java:4944-5019 | `boot_replay::enter_game_frames_match_tick_by_tick` / enter-game @T102 | 23 | 行锚×4〔名2/稳2〕（crates/game-core/src/menu_family.rs:313） |
| paintWrappedText | a.java:5021-5111 | `boot_replay::enter_game_frames_match_tick_by_tick` / enter-game @T102 | 23 | fn crates/game-core/src/scene.rs:1178；行锚×6〔名3/稳3〕（crates/game-core/src/menu_family.rs:449） |
| flipOverlayPage | a.java:5113-5133 | `boot_replay::enter_game_frames_match_tick_by_tick` / enter-game @T165 | 10 | fn crates/game-core/src/scene.rs:1544；行锚×1〔名1〕（crates/game-core/src/scene.rs:1543） |
| skipTextToLineEnd | a.java:5135-5157 | `preset_cases::floor51_menu_frames_match` / floor51-menu @T631 | 2 | fn crates/game-core/src/scene.rs:1579；行锚×1〔名1〕（crates/game-core/src/scene.rs:1578） |
| parseHexColor | a.java:5159-5170 | `preset_cases::preset_floor35_frames_match` / preset-floor35 @T302 | 13 | fn crates/game-core/src/menu_family.rs:40；行锚×1〔名1〕（crates/game-core/src/menu_family.rs:33） |
| applyHpDelta | a.java:5172-5191 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 9 | fn crates/game-core/src/scene.rs:4805；行锚×1〔名1〕（crates/game-core/src/scene.rs:4804） |
| placePlayerAtCell | a.java:5193-5199 | `preset_cases::preset_item21_frames_match` / preset-item21 @T405 | 5 | fn crates/game-core/src/scene.rs:1047（fn m_024）；行锚×1〔名1〕（crates/game-core/src/scene.rs:1046） |
| m_025 | a.java:5201-5207 | `preset_cases::preset_floor2_frames_match` / preset-floor2 @T262 | 25 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:3730） |
| easeAfterimages | a.java:5209-5263 | `preset_cases::floor1_tour_frames_match` / floor1-tour @T602 | 9 | fn crates/game-core/src/scene.rs:3796；行锚×1〔名1〕（crates/game-core/src/scene.rs:3794） |
| handleFieldInput | a.java:5265-5331 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 35 | fn crates/game-core/src/scene.rs:3908；行锚×1〔名1〕（crates/game-core/src/scene.rs:3905） |
| tryStep | a.java:5333-5361 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T261 | 28 | fn crates/game-core/src/scene.rs:3958；行锚×1〔名1〕（crates/game-core/src/scene.rs:3957） |
| interactWithCell | a.java:5363-5541 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T261 | 29 | fn crates/game-core/src/scene.rs:3984；行锚×6〔名2/稳4〕（crates/game-core/src/scene.rs:3957） |
| applyStepCellEffects | a.java:5543-5805 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 34 | fn crates/game-core/src/scene.rs:4420；行锚×9〔名1/稳8〕（crates/game-core/src/scene.rs:4409） |
| stepOffStairs | a.java:5807-5820 | `preset_cases::preset_item21_frames_match` / preset-item21 @T405 | 3 | fn crates/game-core/src/scene.rs:1032（fn m_031）；行锚×1〔名1〕（crates/game-core/src/scene.rs:1031） |
| paintPlayerSprite | a.java:5841-6004 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 35 | 行锚×2〔名1/稳1〕（crates/game-core/src/layout.rs:174） |
| paintSoftkeyBar | a.java:6006-6016 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T68 | 37 | fn crates/game-core/src/paint.rs:50（fn m_034_softkeys）；行锚×7〔名7〕（crates/game-core/src/layout.rs:281） |
| paintStatusBar | a.java:6018-6041 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 35 | fn crates/game-core/src/scene.rs:5637；行锚×1〔名1〕（crates/game-core/src/scene.rs:5636） |
| paintKeySlot | a.java:6043-6050 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T151 | 35 | fn crates/game-core/src/scene.rs:5678；行锚×3〔名3〕（crates/game-core/src/menu_family.rs:720） |
| paintHudPanel | a.java:6052-6112 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 35 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:5334） |
| paintTitledBox | a.java:6114-6132 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 33 | fn crates/game-core/src/menu_family.rs:197；行锚×1〔名1〕（crates/game-core/src/menu_family.rs:195） |
| paintHudFrame | a.java:6134-6162 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 35 | fn crates/game-core/src/scene.rs:5306；行锚×1〔名1〕（crates/game-core/src/scene.rs:5305） |
| paintBoxFrame | a.java:6164-6196 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 36 | fn crates/game-core/src/menu_family.rs:160；行锚×1〔名1〕（crates/game-core/src/menu_family.rs:159） |
| paintMiniFrame | a.java:6198-6207 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T151 | 35 | fn crates/game-core/src/menu_family.rs:219；行锚×1〔名1〕（crates/game-core/src/menu_family.rs:218） |
| paintNumber | a.java:6209-6238 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T151 | 35 | fn crates/game-core/src/menu_family.rs:232；行锚×1〔名1〕（crates/game-core/src/menu_family.rs:230） |
| initEntityTables | a.java:6240-6295 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T213 | 34 | 行锚×2〔名2〕（crates/game-core/src/entity.rs:98） |
| dispatchSpriteSize | a.java:6297-6322 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T223 | 34 | fn crates/game-core/src/entity.rs:147；行锚×2〔名2〕（crates/game-core/src/entity.rs:144） |
| dispatchAnimFields | a.java:6324-6327 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T223 | 34 | 行锚×1〔名1〕（crates/game-core/src/entity.rs:172） |
| markEntityDesRemoved | a.java:6329-6344 | `preset_cases::preset_itemboss40_frames_match` / preset-itemboss40 @T302 | 3 | 行锚×1〔名1〕（crates/game-core/src/entity.rs:291） |
| markEntityRemoved | a.java:6346-6384 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 26 | 行锚×1〔名1〕（crates/game-core/src/entity.rs:249） |
| spawnEntity | a.java:6386-6429 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T223 | 34 | fn crates/game-core/src/script.rs:145; crates/game-core/src/script.rs:531 |
| removeGroupAtCellByType | a.java:6431-6446 | `preset_cases::preset_floor44_frames_match` / preset-floor44 @T798 | 3 | fn crates/game-core/src/entity.rs:483；行锚×1〔名1〕（crates/game-core/src/entity.rs:481） |
| removeSlotAt | a.java:6448-6460 | `preset_cases::preset_floor17_frames_match` / preset-floor17 @T312 | 5 | fn crates/game-core/src/scene.rs:1056；行锚×1〔名1〕（crates/game-core/src/scene.rs:1055） |
| pullEntityFromCell | a.java:6462-6478 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T242 | 26 | 行锚×1〔名1〕（crates/game-core/src/entity.rs:439） |
| advanceBobPhase | a.java:6480-6489 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T72 | 37 | 行锚×2〔名2〕（crates/game-core/src/entity.rs:457） |
| paintEntityLayer | a.java:6491-6727 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 35 | 行锚×13〔名5/稳8〕（crates/game-core/src/render.rs:22） |
| sortEntitiesByY | a.java:6729-6820 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T223 | 34 | 行锚×1〔名1〕（crates/game-core/src/entity.rs:306） |
| advanceEntityFrames | a.java:6822-6848 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T238 | 35 | 行锚×1〔名1〕（crates/game-core/src/entity.rs:389） |
| drawParallaxBackdrop | a.java:6850-6877 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 36 | 行锚×3〔名3〕（crates/game-core/src/menu_family.rs:53） |
| buildMinimap | a.java:6879-6947 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T223 | 34 | fn crates/game-core/src/scene.rs:1863；行锚×1〔名1〕（crates/game-core/src/scene.rs:1861） |
| readU16BE | a.java:6949-6951 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T223 | 34 | 行锚×1〔seed名1〕（crates/game-data/src/packed_png.rs:18） |
| m_059 | a.java:6953-6958 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T223 | 34 | 人工：crates/game-core/src/entity.rs `pub fn new(cols: i32, rows: i32) -> Self`（CellGrid::new 四件初始化 = a.java:6954-6957；doc 点名「m_059 初始化」） |
| isCellWalkable | a.java:6960-6962 | `preset_cases::preset_floor2_frames_match` / preset-floor2 @T261 | 30 | fn crates/game-core/src/pathfind.rs:31 |
| rebuildWalkability | a.java:6964-6983 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T223 | 34 | 行锚×3〔名3〕（crates/game-core/src/game_view.rs:9） |
| m_062 | a.java:6985-6987 | `preset_cases::floor51_menu_frames_match` / floor51-menu @T902 | 2 | fn crates/game-core/src/pathfind.rs:39; crates/game-core/src/pathfind.rs:39（fn m_062） |
| paintTileLayer | a.java:6989-7051 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 35 | 行锚×8〔名8〕（crates/game-core/src/game_view.rs:2） |
| setCameraClamped | a.java:7053-7104 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T223 | 34 | 行锚×5〔名5〕（crates/game-core/src/camera.rs:58） |
| findFloorGateEntity | a.java:7106-7124 | `preset_cases::preset_item21_frames_match` / preset-item21 @T405 | 3 | fn crates/game-core/src/scene.rs:1011；行锚×2〔名2〕（crates/game-core/src/scene.rs:1006） |
| changeFloor | a.java:7126-7172 | `preset_cases::preset_item21_frames_match` / preset-item21 @T401 | 3 | fn crates/game-core/src/scene.rs:928；行锚×2〔名2〕（crates/game-core/src/scene.rs:330） |
| startNewGameLoad | a.java:7174-7186 | `preset_cases::floor51_menu_frames_match` / floor51-menu @T541 | 2 | 行锚×2〔名2〕（crates/game-core/src/intro.rs:279） |
| m_068 | a.java:7188-7216 | `preset_cases::floor51_menu_frames_match` / floor51-menu @T570 | 2 | 人工：新游戏态复位分散移植：crates/game-desktop/src/main.rs `GameScene::load_floor(` 以 (32,352) 落点 = placePlayerAtCell(1,11)（a.java:7194 floor51 臂，新游戏 step8 已置 51 层）+ crates/game-core/src/scene.rs `difficulty_index: 0`（装备/跟随/金/道具栈/炼丹/难度构造默认 0 = a.java:7189-7190、7210-7214）+ crates/game-desktop/src/main.rs `scale_enemy_stats(&tables, 1)`（= a.java:7215 scaleEnemyStats(multipliers[0])） |
| scaleEnemyStats | a.java:7218-7248 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T220 | 34 | fn crates/game-core/src/combat.rs:56；行锚×1〔名1〕（crates/game-core/src/combat.rs:48） |
| scaledByFloorTier | a.java:7250-7254 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T261 | 1 | fn crates/game-core/src/scene.rs:3078；行锚×2〔名2〕（crates/game-core/src/scene.rs:276） |
| alchemyPriceFor | a.java:7256-7264 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T261 | 1 | fn crates/game-core/src/scene.rs:3068；行锚×2〔名2〕（crates/game-core/src/scene.rs:274） |
| m_072 | a.java:7266-7283 | `preset_cases::preset_load_menu_frames_match` / preset-load-menu @T501 | 2 | 人工：crates/game-core/src/menu_family.rs `fn add(&mut self, label_idx: usize, checked: bool)`（OptionList::add = addOption：stringWidth+80 定框宽、首项 32+行高 = a.java:7267-7277；原 28 条行锚系 game-data seed 方言锚误归属） |
| tryOpenChest | a.java:7285-7305 | `preset_cases::preset_floor17_frames_match` / preset-floor17 @T312 | 4 | fn crates/game-core/src/scene.rs:1072；行锚×1〔名1〕（crates/game-core/src/scene.rs:1070） |
| applyMerchantOffer | a.java:7307-7470 | `preset_cases::preset_merchant_frames_match` / preset-merchant @T321 | 3 | fn crates/game-core/src/scene.rs:3258；行锚×2〔名2〕（crates/game-core/src/scene.rs:285） |
| m_075 | a.java:7472-7478 | `preset_cases::preset_floor46_frames_match` / preset-floor46 @T351 | 2 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:3237） |
| spendGold | a.java:7507-7516 | `preset_cases::preset_floor45_frames_match` / preset-floor45 @T501 | 1 | fn crates/game-core/src/scene.rs:3247 |
| gainGold | a.java:7518-7523 | `preset_cases::preset_floor10_frames_match` / preset-floor10 @T302 | 21 | fn crates/game-core/src/combat.rs:174；行锚×1〔名1〕（crates/game-core/src/combat.rs:172） |
| pickupItemType | a.java:7525-7640 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 10 | fn crates/game-core/src/scene.rs:4657；行锚×1〔名1〕（crates/game-core/src/scene.rs:4643） |
| floorTier | a.java:7642-7653 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 8 | fn crates/game-core/src/scene.rs:4646；行锚×1〔名1〕（crates/game-core/src/scene.rs:4645） |
| findItemStackIndex | a.java:7655-7666 | `preset_cases::preset_floor10_frames_match` / preset-floor10 @T302 | 24 | 人工：crates/game-core/src/combat.rs `pub fn find_item(&self, item_type: u8) -> i32`（doc「findItemStackIndex 忠实移植」，首个同型槽/无则 -1 = a.java:7656-7665；其 Rust 行锚 8485 陈旧，落入 executeScriptInstruction） |
| tryEquip | a.java:7668-7688 | `preset_cases::preset_floor43_frames_match` / preset-floor43 @T1205 | 2 | fn crates/game-core/src/scene.rs:1130; crates/game-core/src/script.rs:155（fn m_082_set_dialog_target） …；行锚×1〔名1〕（crates/game-core/src/scene.rs:1128） |
| m_083 | a.java:7690-7700 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 10 | fn crates/game-core/src/scene.rs:1152（fn m_083_index）；行锚×2〔名2〕（crates/game-core/src/scene.rs:1151） |
| itemTypeToStackIndex | a.java:7702-7716 | `preset_cases::preset_item2122_frames_match` / preset-item2122 @T351 | 4 | 人工：crates/game-core/src/scene.rs `fn item_desc_index(&self, t: i32) -> usize`（85→20 / 86→21 / 其余 t-13 = a.java:7704-7713；add_item_to_stack 内联同式） |
| consumeKeyForDoor | a.java:7733-7759 | `preset_cases::preset_floor34_frames_match` / preset-floor34 @T301 | 11 | fn crates/game-core/src/combat.rs:153 |
| useItemStack | a.java:7761-7779 | `preset_cases::preset_item2122_frames_match` / preset-item2122 @T401 | 3 | fn crates/game-core/src/scene.rs:2578；行锚×1〔名1〕（crates/game-core/src/scene.rs:2576） |
| activateItem | a.java:7781-7890 | `preset_cases::preset_item13_frames_match` / preset-item13 @T351 | 4 | fn crates/game-core/src/scene.rs:2361；行锚×4〔名2/稳2〕（crates/game-core/src/scene.rs:2356） |
| killAdjacentAt | a.java:7923-7952 | `preset_cases::preset_itemcymbal_frames_match` / preset-itemcymbal @T401 | 1 | fn crates/game-core/src/scene.rs:2479；行锚×1〔名1〕（crates/game-core/src/scene.rs:2473） |
| m_091 | a.java:7954-8014 | `preset_cases::preset_item13_frames_match` / preset-item13 @T351 | 1 | 行锚×2〔名2〕（crates/game-core/src/layout.rs:326） |
| m_092 | a.java:8016-8029 | `preset_cases::preset_item13_frames_match` / preset-item13 @T401 | 1 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:2932） |
| loadLevelScript | a.java:8035-8063 | `preset_cases::preset_floor35_frames_match` / preset-floor35 @T301 | 6 | fn crates/game-core/src/scene.rs:1374; crates/game-core/src/script.rs:191；行锚×3〔名3〕（crates/game-core/src/scene.rs:1372） |
| tryRunScene | a.java:8065-8221 | `preset_cases::preset_floor35_frames_match` / preset-floor35 @T301 | 5 | fn crates/game-core/src/scene.rs:1274；行锚×5〔名1/稳4〕（crates/game-core/src/scene.rs:1272） |
| openDialogPage | a.java:8223-8248 | `preset_cases::preset_floor35_frames_match` / preset-floor35 @T302 | 5 | fn crates/game-core/src/scene.rs:1397; crates/game-core/src/script.rs:214；行锚×2〔名2〕（crates/game-core/src/scene.rs:1395） |
| cameraAtTarget | a.java:8250-8252 | `preset_cases::preset_itemboss40_frames_match` / preset-itemboss40 @T503 | 6 | fn crates/game-core/src/scene.rs:3477 |
| executeScriptInstruction | a.java:8254-8602 | `preset_cases::preset_floor35_frames_match` / preset-floor35 @T302 | 6 | fn crates/game-core/src/scene.rs:1592; crates/game-core/src/script.rs:266；行锚×19〔名1/稳18〕（crates/game-core/src/enums.rs:150） |
| parseScriptInt | a.java:8604-8610 | `preset_cases::preset_floor35_frames_match` / preset-floor35 @T302 | 6 | fn crates/game-core/src/scene.rs:1389; crates/game-core/src/script.rs:258；行锚×3〔seed名1/名2〕（crates/game-core/src/scene.rs:1388） |
| m_100 | a.java:8612-8626 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 34 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:4992） |
| m_103 | a.java:8659-8664 | `preset_cases::floor51_menu_frames_match` / floor51-menu @T902 | 2 | 人工：crates/game-core/src/scene.rs `// m_103：ease 相机目标 = 目标格`（SEE 臂）+ crates/game-core/src/scene.rs `// m_103：ease 目标 = SEE 目标格`（翻页臂）两处内联，四赋值 = a.java:8660-8663 |
| lockCameraOn | a.java:8666-8705 | `preset_cases::preset_floor35_frames_match` / preset-floor35 @T301 | 11 | fn crates/game-core/src/scene.rs:3848；行锚×2〔名2〕（crates/game-core/src/scene.rs:1419） |
| stepCameraTowardTarget | a.java:8707-8733 | `preset_cases::preset_floor35_frames_match` / preset-floor35 @T302 | 11 | fn crates/game-core/src/scene.rs:3876；行锚×1〔名1〕（crates/game-core/src/scene.rs:3874） |
| spawnBossEvent | a.java:8903-8912 | `preset_cases::preset_itemboss40_frames_match` / preset-itemboss40 @T302 | 2 | fn crates/game-core/src/scene.rs:2560；行锚×1〔名1〕（crates/game-core/src/scene.rs:2557） |
| loadSlotTable | a.java:8950-8996 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 33 | fn crates/game-core/src/menu_family.rs:615；行锚×1〔名1〕（crates/game-core/src/scene.rs:5795） |
| markSlotEntry | a.java:8998-9050 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 3 | 行锚×2〔名2〕（crates/game-core/src/scene.rs:5694） |
| m_113 | a.java:9052-9176 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 33 | 行锚×9〔名4/稳5〕（crates/game-core/src/menu_family.rs:731） |
| saveFloorState | a.java:9178-9242 | `preset_cases::preset_load_menu_frames_match` / preset-load-menu @T341 | 2 | fn crates/game-core/src/scene.rs:959；行锚×2〔名2〕（crates/game-core/src/save.rs:1） |
| loadFloorSave | a.java:9244-9319 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T220 | 32 | 行锚×3〔名3〕（crates/game-core/src/save.rs:1） |
| deleteRecordStoreQuietly | a.java:9321-9326 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T68 | 6 | 人工·吸收：删库后重建（a.java:9322-9325）；三调用点 a.java:4476（SKY_WAR 读失败→删→saveGlobalProgress 重建）/9018（MOT_IF）/9181（MOT_L{n}）均为删后即写；壳 RMS 以覆盖写吸收删+写（crates/game-desktop/src/main.rs `std::fs::write(&path, v).ok();`）；SKY_WAR 路径的持久化缺口归 saveGlobalProgress（a） |
| m_117 | a.java:9328-9360 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T151 | 33 | 人工·吸收：RecordStore/流关闭 + 五字段置 null + gc（a.java:9329-9359）；f_byte_arr_27 读者（m_000/loadSlotTable/loadFloorSave）均先 getRecord 赋值再读（a.java:4459/8971/9253）⇒ 无可观测效应 |
| m_118 | a.java:9362-9368 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T216 | 34 | 人工：crates/game-core/src/scene.rs `saved_floor_states: vec![None; 56]`（新游戏构造即 56 槽空 = a.java:9363-9367；读档路径 crates/game-core/src/scene.rs `scene.saved_floor_states = save.floor_states.clone()`） |
| m_119 | a.java:9370-9397 | `preset_cases::preset_load_menu_frames_match` / preset-load-menu @T341 | 5 | 行锚×2〔名2〕（crates/game-core/src/scene.rs:938） |
| m_120 | a.java:9399-9402 | `preset_cases::preset_load_menu_frames_match` / preset-load-menu @T341 | 5 | 人工：crates/game-core/src/scene.rs `fn save_floor_state(&mut self)`（u16 BE 写内联为 to_be_bytes = a.java:9400-9401）；行锚×1〔名1〕（crates/game-core/src/scene.rs:957） |
| loadFloorData | a.java:9404-9444 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T223 | 34 | fn crates/game-core/src/scene.rs:843；行锚×4〔seed名1/名3〕（crates/game-core/src/game_view.rs:1） |
| m_122 | a.java:9446-9674 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T223 | 34 | 行锚×53〔seed稳45/名6/稳2〕（crates/game-core/src/menu_family.rs:842） |
| m_123 | a.java:9676-9678 | `preset_cases::floor2_tour_frames_match` / floor2-tour @T1357 | 1 | 人工：crates/game-core/src/scene.rs `u16::from_be_bytes([d[*p], d[*p + 1]])`（层存档恢复 read_u16 闭包 = a.java:9677，m_122 恢复臂 a.java:9623-9635 内联） |
| spawnPopup | a.java:9688-9727 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 24 | 行锚×4〔名4〕（crates/game-core/src/combat.rs:123） |
| drawPopupLayer | a.java:9729-9788 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 35 | 行锚×3〔名3〕（crates/game-core/src/popup.rs:1） |
| drawDigitStrip | a.java:9790-9816 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T237 | 23 | fn crates/game-core/src/popup.rs:149；行锚×1〔名1〕（crates/game-core/src/popup.rs:145） |
| m_128 | a.java:9818-9828 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 36 | 人工：crates/game-core/src/title.rs `pub fn new() -> Particles`（32 槽粒子数组清零 + 游标 0 = a.java:9819-9827） |
| spawnParticle | a.java:9830-9842 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T73 | 36 | 行锚×1〔名1〕（crates/game-core/src/title.rs:79） |
| updateAndDrawParticles | a.java:9844-9871 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 36 | 行锚×2〔名2〕（crates/game-core/src/title.rs:11） |
| m_131 | a.java:9873-9883 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T201 | 34 | 人工：crates/game-core/src/intro.rs `pub fn new(target: i32, call_m000: bool, steps: &[i8]) -> LoadProgress`（loadTargetMode/loadCallInit/步表/计数复位 = a.java:9874-9881；m_001(15) 由构造期装载吸收）；行锚×1〔名1〕（crates/game-core/src/intro.rs:274） |
| m_132 | a.java:9885-9887 | `preset_cases::preset_alchemy_frames_match` / preset-alchemy @T201 | 34 | 行锚×1〔名1〕（crates/game-core/src/intro.rs:274） |
| findPath | a.java:9889-9954 | `preset_cases::floor51_menu_frames_match` / floor51-menu @T902 | 2 | fn crates/game-core/src/pathfind.rs:54; crates/game-core/src/script.rs:122 …；行锚×1〔名1〕（crates/game-core/src/pathfind.rs:3） |
| m_134 | a.java:9956-9981 | `preset_cases::floor51_menu_frames_match` / floor51-menu @T902 | 2 | fn crates/game-core/src/pathfind.rs:145; crates/game-core/src/pathfind.rs:145（fn m_134） |
| predictHpLossVsType | a.java:9983-9987 | `preset_cases::preset_floor10_frames_match` / preset-floor10 @T301 | 23 | fn crates/game-core/src/combat.rs:107；行锚×2〔名2〕（crates/game-core/src/combat.rs:106） |
| effectiveAttackVsType | a.java:9989-9999 | `preset_cases::preset_floor10_frames_match` / preset-floor10 @T301 | 24 | fn crates/game-core/src/combat.rs:66；行锚×3〔名3〕（crates/game-core/src/combat.rs:64） |
| predictBattleHpLoss | a.java:10001-10021 | `preset_cases::preset_floor10_frames_match` / preset-floor10 @T301 | 24 | fn crates/game-core/src/combat.rs:79；行锚×1〔名1〕（crates/game-core/src/combat.rs:77） |
| tickBattle | a.java:10023-10156 | `preset_cases::preset_floor10_frames_match` / preset-floor10 @T302 | 20 | fn crates/game-core/src/scene.rs:4205；行锚×3〔名2/稳1〕（crates/game-core/src/scene.rs:309） |
| m_139 | a.java:10158-10206 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T71 | 36 | 行锚×3〔名3〕（crates/game-core/src/audio.rs:3） |
| closeAudioPlayer | a.java:10208-10215 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T68 | 36 | 行锚×4〔名4〕（crates/game-core/src/audio.rs:4） |
| randomBelow | a.java:10217-10219 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T73 | 36 | fn crates/game-platform/src/lib.rs:77 |
| m_142 | a.java:10221-10235 | `preset_cases::floor51_menu_frames_match` / floor51-menu @T577 | 2 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:5480） |
| runLogoAnimation | a.java:10237-10473 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T33 | 36 | 行锚×10〔名5/稳5〕（crates/game-core/src/logo_anim.rs:3） |
| paintLogoAnimation | a.java:10475-10503 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T32 | 36 | 行锚×3〔名3〕（crates/game-core/src/logo_anim.rs:15） |
| registerLogoItem | a.java:10505-10562 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T33 | 36 | 行锚×2〔名2〕（crates/game-core/src/logo_anim.rs:10） |
| modifyLogoItem | a.java:10564-10588 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T42 | 36 | 行锚×1〔名1〕（crates/game-core/src/logo_anim.rs:193） |
| removeLogoItem | a.java:10590-10605 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T37 | 36 | 行锚×3〔名3〕（crates/game-core/src/logo_anim.rs:3） |
| <clinit> | a.java:10877-10879 | `boot_replay::boot_menu_frames_match_tick_by_tick` / menu-sweep @T1 | 36 | 行锚×1〔字段1〕（crates/game-core/src/render.rs:476） |

## 3. 黄：已移植 + 无帧级窗口触达（7）

| 方法 | 区间 | 场景触达（无用例） | 移植证据 | 调用点（deobf） |
|---|---|---|---|---|
| hideNotify | a.java:4502-4514 | **无场景触达** | fn crates/game-desktop/src/main.rs:503；行锚×6〔名6〕（crates/game-core/src/audio.rs:64） |  |
| advanceScriptWalk | a.java:5822-5839 | **无场景触达** | fn crates/game-core/src/scene.rs:4624；行锚×1〔名1〕（crates/game-core/src/scene.rs:4622） | run:3568, handleFieldInput:5268, applyStepCellEffects:5736 |
| m_076 | a.java:7480-7505 | preset-floor5@T621 | 行锚×1〔名1〕（crates/game-core/src/scene.rs:5432） | run:3331, run:3334, m_000:4387 |
| addItemToItemStack | a.java:7718-7731 | preset-floor5@T265 | 人工：crates/game-core/src/scene.rs `fn add_item_to_stack(&mut self, t: i32)`（追加/叠加次数 + showOverlayMessage = a.java:7719-7728/7730；行锚 7702-7714 漂移落入 itemTypeToStackIndex）。未移植 itemUseTargetType 写（a.java:7729）——无可见效应：拾取路径调用前 已置 overlay_kind_item（crates/game-core/src/scene.rs `self.overlay_kind_item = t`），商人路径浮层同调用内被关闭/替换（2026-10-15 独立审计核）；行锚×1〔名1〕（crates/game-core/src/scene.rs:4725） | applyMerchantOffer:7330, applyMerchantOffer:7333, pickupItemType:7545 |
| m_089 | a.java:7892-7921 | **无场景触达** | 行锚×1〔名1〕（crates/game-core/src/scene.rs:2539） | activateItem:7799, activateItem:7807 |
| m_101 | a.java:8628-8635 | **无场景触达** | fn crates/game-core/src/scene.rs:4377; crates/game-core/src/scene.rs:4377（fn m_101）；行锚×1〔名1〕（crates/game-core/src/scene.rs:4375） | applyStepCellEffects:5611, applyStepCellEffects:5621 |
| m_102 | a.java:8637-8657 | preset-floor49@T951 | fn crates/game-core/src/scene.rs:1367（fn m_102_current_floor）；行锚×1〔名1〕（crates/game-core/src/scene.rs:1364） | tryRunScene:8121, tryRunScene:8146, tryRunScene:8195 |

## 4. 白：无移植账（13）

| 方法 | 区间 | 分账 | 场景触达 | Rust 点名提及 | 论证 / 依据 |
|---|---|---|---|---|---|
| m_093 | a.java:8031-8033 | b 不触达论证候选 | 无 | 0 | 左向三角箭头（a.java:8031-8033）：唯一调用点 paintPlayerSprite walkPhase case 3 道具轮盘（a.java:5977/5982）；JAVAP 中 putfield walkPhase 共 23 处，前导常量 ∈{0,1,2,4,5}（无 3，亦无计算值写入）⇒ walkPhase 永不为 3，case 3 绘制臂与 run() `switch (walkPhase)`（a.java:3565）的 case 3 键处理臂（a.java:3647-3673）皆死代码 |
| m_106 | a.java:8735-8867 | a 真未移植 | 无 | 4（crates/game-core/src/enums.rs:14） | MOV 五参实体路径缓动（a.java:8735-8867）：run() dialogPhase 2 臂调用（a.java:4073）；dialogPhase=2 由 MOV 五参臂在 m_100 命中实体时置（a.java:8285-8296）；levelScriptLines（a.java:1124-1193，68 条）含五参 MOV 36 处（脚本 1/2/4/5/7/8/10/12/19/30/57/61），其中 20 处起点（型,格）在 jar sprite{n} 初始布置中有对应实体（如脚本 1 `MOV_72_3_7_1_8` = sprite2 型 72@(3,7)；脚本 12 的 13 处全落 sprite40）⇒ 正常游玩可达。Rust：crates/game-core/src/scene.rs `todo!("m_106 实体路径缓动（MOV 对拍批）")`；即 convergence-plan P1 挂起项 dp2=m_106 |
| m_107 | a.java:8869-8875 | b 不触达论证候选 | 无 | 0 | 实体换格（a.java:8869-8875）：唯一调用点 executeScriptInstruction 的 MVS 臂（a.java:8516-8589，`!var3.equals("MVS")` 之 else）；levelScriptLines（a.java:1124-1193）68 条脚本 MVS 出现 0 次；executeScriptInstruction 三调用点均传 dialogScriptLine（a.java:4021/8299/8434）；JAVAP：dialogScriptLine 唯一写点 loadLevelScript（aaload levelScriptLines），levelScriptLines 仅 <init> 赋值且无元素写 ⇒ 脚本文本源封闭、MVS 死指令（crates/game-core/src/script.rs `数据域 68 条脚本零使用`） |
| m_108 | a.java:8877-8901 | a 真未移植 | 无 | 0 | 实体跨格槽表重登记（a.java:8877-8901）：唯一调用者 m_106（a.java:8803/8845；javap 调用图同）⇒ 随 m_106 可达、随 m_106 一并移植；Rust 无对应实现 |
| saveGlobalProgress | a.java:8914-8948 | a 真未移植 | boot-menu@T68; enter-game@T68; floor51-menu@T68 | 3（crates/game-core/src/audio.rs:200） | SKY_WAR 全局进度写（a.java:8914-8948：optionChecked×4 + f_int_arr_35 + f_int_152..154）；调用点 m_000 a.java:4299/4478、run a.java:4169（选项切换后）。Rust：save.rs 对 SKY_WAR 既不编码也不解码（只有 parse_floor_save/parse_preset）；game-core 无 I/O（crates/game-core/src/scene.rs `saveGlobalProgress（RMS 侧，无像素效应；game-core 无 I/O）`），boot 恒走 SKY_WAR 读失败回退（crates/game-core/src/paint.rs `RMS "SKY_WAR" 读档失败路径`）；壳 Rms 只改写 MOT_L*/MOT_IF（crates/game-desktop/src/main.rs `stores.push(("MOT_IF@1".into(), mot_if.clone()));`）⇒ 选项/全局旗标跨会话不持久（读写双缺，读臂在 m_000 case 21 a.java:4450-4479）——壳层赤字（会话内帧不可见，非铺层赤字） |
| addSpark | a.java:9680-9686 | b 不触达论证候选 | 无 | 0 | 火花登记（a.java:9680-9686）：唯一调用点 paint `switch (gameMode)` case 18（a.java:3003-3031）；gameMode 写点无 18（论证同 m_148 的 JAVAP 普查）⇒ 死代码（completeness 迭代 60 mode18 火花 ☠️）。v2 的唯一移植证据系漂移锚（combat.rs 注「spawnPopup kind 常量（…9679-9686…）」） |
| m_148 | a.java:10607-10636 | b 不触达论证候选 | 无 | 0 | 联网簇·连接关闭（a.java:10607-10636）：调用点 run mode99 臂 a.java:4261、m_151/m_153（簇内）。mode 99 不可达：JAVAP putfield gameMode 54 处，常量 ∈{0-5,7-12,14-17,19-22}，计算来源 menuReturnMode∈{1,4}/loadTargetMode∈{3}/inGameMenuReturnMode←gameMode ⇒ 无 99；另 master-plan §4 范围裁剪（HttpConnection 联网整体剔除） |
| m_149 | a.java:10638-10640 | b 不触达论证候选 | 无 | 0 | 联网簇·响应体 u16 读（a.java:10638-10640）：仅 m_150/m_153 调用（簇内）；簇不可达论证同 m_148 |
| m_150 | a.java:10642-10644 | b 不触达论证候选 | 无 | 0 | 联网簇·响应体 u32 读（a.java:10642-10644）：仅 m_153 调用（簇内）；同 m_148 |
| m_151 | a.java:10648-10766 | b 不触达论证候选 | 无 | 0 | 联网簇·HTTP POST（a.java:10648-10766）：仅 run() f_bool_31 臂调用（a.java:3229-3242）；JAVAP putfield f_bool_31：<init>/run×2 置 0、唯一置 1 者 m_153（mode99 内）⇒ 不可达；同 m_148 |
| m_152 | a.java:10768-10779 | b 不触达论证候选 | 无 | 0 | 联网簇·状态读（a.java:10768-10779）：调用点 run mode99 臂 a.java:4271 与 m_153；同 m_148 |
| m_153 | a.java:10781-10865 | b 不触达论证候选 | 无 | 0 | 联网簇·轮询/重试状态机（a.java:10781-10865）：唯一调用点 run mode99 臂 a.java:4267，且该臂先要求 f_bool_31 已真（a.java:4258），而 f_bool_31 唯一置真点就在 m_153 内（自门控）⇒ 即便 mode99 可达也永不执行；同 m_148（2026-10-15 独立审计补强） |
| m_154 | a.java:10867-10875 | b 不触达论证候选 | 无 | 0 | 联网簇·hex 字符解码（a.java:10867-10875）：仅 m_151 调用（a.java:10727）；同 m_148 |

### 4.1 刷账三分账（2026-10-15：基线 23 白 + v2.1 归属治理新暴露）

c 类 = 白色系工具方法论所致（移植存在但漏判 / 无可观测效应无「吸收」口径）；解决手段为归属规则修正或 PORT_NOTES 人工条目（逐条校验 Rust 锚），解决后现色随之变化。

| 方法 | 区间 | 来源 | 分账 | 现色 | 场景触达 | 论证 / 依据 |
|---|---|---|---|---|---|---|
| m_106 | a.java:8735-8867 | 基线23 | a 真未移植 | 白 | 无 | MOV 五参实体路径缓动（a.java:8735-8867）：run() dialogPhase 2 臂调用（a.java:4073）；dialogPhase=2 由 MOV 五参臂在 m_100 命中实体时置（a.java:8285-8296）；levelScriptLines（a.java:1124-1193，68 条）含五参 MOV 36 处（脚本 1/2/4/5/7/8/10/12/19/30/57/61），其中 20 处起点（型,格）在 jar sprite{n} 初始布置中有对应实体（如脚本 1 `MOV_72_3_7_1_8` = sprite2 型 72@(3,7)；脚本 12 的 13 处全落 sprite40）⇒ 正常游玩可达。Rust：crates/game-core/src/scene.rs `todo!("m_106 实体路径缓动（MOV 对拍批）")`；即 convergence-plan P1 挂起项 dp2=m_106 |
| m_108 | a.java:8877-8901 | 基线23 | a 真未移植 | 白 | 无 | 实体跨格槽表重登记（a.java:8877-8901）：唯一调用者 m_106（a.java:8803/8845；javap 调用图同）⇒ 随 m_106 可达、随 m_106 一并移植；Rust 无对应实现 |
| saveGlobalProgress | a.java:8914-8948 | 基线23 | a 真未移植 | 白 | boot-menu@T68; enter-game@T68 | SKY_WAR 全局进度写（a.java:8914-8948：optionChecked×4 + f_int_arr_35 + f_int_152..154）；调用点 m_000 a.java:4299/4478、run a.java:4169（选项切换后）。Rust：save.rs 对 SKY_WAR 既不编码也不解码（只有 parse_floor_save/parse_preset）；game-core 无 I/O（crates/game-core/src/scene.rs `saveGlobalProgress（RMS 侧，无像素效应；game-core 无 I/O）`），boot 恒走 SKY_WAR 读失败回退（crates/game-core/src/paint.rs `RMS "SKY_WAR" 读档失败路径`）；壳 Rms 只改写 MOT_L*/MOT_IF（crates/game-desktop/src/main.rs `stores.push(("MOT_IF@1".into(), mot_if.clone()));`）⇒ 选项/全局旗标跨会话不持久（读写双缺，读臂在 m_000 case 21 a.java:4450-4479）——壳层赤字（会话内帧不可见，非铺层赤字） |
| m_093 | a.java:8031-8033 | 基线23 | b 不触达论证候选 | 白 | 无 | 左向三角箭头（a.java:8031-8033）：唯一调用点 paintPlayerSprite walkPhase case 3 道具轮盘（a.java:5977/5982）；JAVAP 中 putfield walkPhase 共 23 处，前导常量 ∈{0,1,2,4,5}（无 3，亦无计算值写入）⇒ walkPhase 永不为 3，case 3 绘制臂与 run() `switch (walkPhase)`（a.java:3565）的 case 3 键处理臂（a.java:3647-3673）皆死代码 |
| m_107 | a.java:8869-8875 | 基线23 | b 不触达论证候选 | 白 | 无 | 实体换格（a.java:8869-8875）：唯一调用点 executeScriptInstruction 的 MVS 臂（a.java:8516-8589，`!var3.equals("MVS")` 之 else）；levelScriptLines（a.java:1124-1193）68 条脚本 MVS 出现 0 次；executeScriptInstruction 三调用点均传 dialogScriptLine（a.java:4021/8299/8434）；JAVAP：dialogScriptLine 唯一写点 loadLevelScript（aaload levelScriptLines），levelScriptLines 仅 <init> 赋值且无元素写 ⇒ 脚本文本源封闭、MVS 死指令（crates/game-core/src/script.rs `数据域 68 条脚本零使用`） |
| m_148 | a.java:10607-10636 | 基线23 | b 不触达论证候选 | 白 | 无 | 联网簇·连接关闭（a.java:10607-10636）：调用点 run mode99 臂 a.java:4261、m_151/m_153（簇内）。mode 99 不可达：JAVAP putfield gameMode 54 处，常量 ∈{0-5,7-12,14-17,19-22}，计算来源 menuReturnMode∈{1,4}/loadTargetMode∈{3}/inGameMenuReturnMode←gameMode ⇒ 无 99；另 master-plan §4 范围裁剪（HttpConnection 联网整体剔除） |
| m_149 | a.java:10638-10640 | 基线23 | b 不触达论证候选 | 白 | 无 | 联网簇·响应体 u16 读（a.java:10638-10640）：仅 m_150/m_153 调用（簇内）；簇不可达论证同 m_148 |
| m_150 | a.java:10642-10644 | 基线23 | b 不触达论证候选 | 白 | 无 | 联网簇·响应体 u32 读（a.java:10642-10644）：仅 m_153 调用（簇内）；同 m_148 |
| m_151 | a.java:10648-10766 | 基线23 | b 不触达论证候选 | 白 | 无 | 联网簇·HTTP POST（a.java:10648-10766）：仅 run() f_bool_31 臂调用（a.java:3229-3242）；JAVAP putfield f_bool_31：<init>/run×2 置 0、唯一置 1 者 m_153（mode99 内）⇒ 不可达；同 m_148 |
| m_152 | a.java:10768-10779 | 基线23 | b 不触达论证候选 | 白 | 无 | 联网簇·状态读（a.java:10768-10779）：调用点 run mode99 臂 a.java:4271 与 m_153；同 m_148 |
| m_153 | a.java:10781-10865 | 基线23 | b 不触达论证候选 | 白 | 无 | 联网簇·轮询/重试状态机（a.java:10781-10865）：唯一调用点 run mode99 臂 a.java:4267，且该臂先要求 f_bool_31 已真（a.java:4258），而 f_bool_31 唯一置真点就在 m_153 内（自门控）⇒ 即便 mode99 可达也永不执行；同 m_148（2026-10-15 独立审计补强） |
| m_154 | a.java:10867-10875 | 基线23 | b 不触达论证候选 | 白 | 无 | 联网簇·hex 字符解码（a.java:10867-10875）：仅 m_151 调用（a.java:10727）；同 m_148 |
| m_001 | a.java:4516-4549 | 基线23 | c 判定存疑（工具方法论） | 绿 | boot-menu@T33; enter-game@T33 | packed 容器解码（a.java:4516-4549）已移植于 game-data packed_png.rs：其 seed 方言锚 `reference/seed/a.java:2657-2676` = seed `private void a(int n)`（序数对齐 = m_001），v2 按 deobf 行号读成 paint 区间。解决：v2.1 方言规则（无需人工条目） |
| m_008 | a.java:4634-4661 | 基线23 | c 判定存疑（工具方法论） | 绿 | boot-menu@T33; enter-game@T33 | I/O 清理无可观测效应（a.java:4634-4661；见 PORT_NOTES absorbed）：v2 无「吸收」口径 |
| readU16BE | a.java:6949-6951 | 基线23 | c 判定存疑（工具方法论） | 绿 | preset-alchemy@T223; preset-ending@T223 | u16 读原语（a.java:6949-6951）已移植于 game-data lib.rs read::u16_le：其 seed 锚引 `private static short a(InputStream)`（序数对齐 = readU16BE）。解决：v2.1 方言 + CFR 签名点名规则。注：方法体为 LE（低字节先），名称 BE 不符——命名台账疑误（data/naming 不在本任务写入范围） |
| m_059 | a.java:6953-6958 | 基线23 | c 判定存疑（工具方法论） | 绿 | preset-alchemy@T223; preset-ending@T223 | CellGrid::new 已移植（a.java:6953-6958；见 PORT_NOTES）：fn 异名且无行锚 |
| findItemStackIndex | a.java:7655-7666 | 基线23 | c 判定存疑（工具方法论） | 绿 | preset-floor5@T265; preset-floor10@T302 | PlayerCombat::find_item 已移植（a.java:7655-7666；见 PORT_NOTES）：fn 异名 + 行锚陈旧 |
| addItemToItemStack | a.java:7718-7731 | 基线23 | c 判定存疑（工具方法论） | 黄 | preset-floor5@T265 | add_item_to_stack 已移植（a.java:7718-7731；见 PORT_NOTES）：fn 异名 + 行锚漂移 |
| m_103 | a.java:8659-8664 | 基线23 | c 判定存疑（工具方法论） | 绿 | preset-floor51@T282; floor51-menu@T902 | 两处内联已移植（a.java:8659-8664；见 PORT_NOTES）：无 fn、注释点名无行锚 |
| m_118 | a.java:9362-9368 | 基线23 | c 判定存疑（工具方法论） | 绿 | preset-alchemy@T216; preset-ending@T216 | 构造期吸收（a.java:9362-9368；见 PORT_NOTES） |
| m_120 | a.java:9399-9402 | 基线23 | c 判定存疑（工具方法论） | 绿 | preset-load-menu@T341; preset-item21@T401 | 内联于 save_floor_state（a.java:9399-9402；见 PORT_NOTES） |
| m_123 | a.java:9676-9678 | 基线23 | c 判定存疑（工具方法论） | 绿 | floor2-tour@T1357; preset-floor8@T2499 | 内联于层存档恢复（a.java:9676-9678；见 PORT_NOTES） |
| <clinit> | a.java:10877-10879 | 基线23 | c 判定存疑（工具方法论） | 绿 | boot-menu@T1; enter-game@T1 | static{}（a.java:10877-10879）：JAVAP static{} 0-51 putstatic f_int_arr_01 = {0,8192,16384,24576,8462,270,90,8282}，54-93 另造一个 int[8] 后丢弃（无 putstatic）；VF 将前者投影为字段初始化器 a.java:44，v2 不把字段初始化器行归属 <clinit>。Rust：crates/game-core/src/paint.rs `pub const NOKIA_TRANSFORM_TABLE: [i32; 8] = [0, 8192, 16384, 24576, 8462, 270, 90, 8282];`。解决：v2.1 字段初始化器归属规则 |
| addSpark | a.java:9680-9686 | 修正暴露 | b 不触达论证候选 | 白 | 无 | 火花登记（a.java:9680-9686）：唯一调用点 paint `switch (gameMode)` case 18（a.java:3003-3031）；gameMode 写点无 18（论证同 m_148 的 JAVAP 普查）⇒ 死代码（completeness 迭代 60 mode18 火花 ☠️）。v2 的唯一移植证据系漂移锚（combat.rs 注「spawnPopup kind 常量（…9679-9686…）」） |
| m_006 | a.java:4605-4614 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | preset-alchemy@T223; preset-ending@T223 | 结果被丢弃的纯函数（a.java:4605-4614；见 PORT_NOTES absorbed）；v2 证据系漂移锚 |
| releaseSceneImages | a.java:4714-4728 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | boot-menu@T71; enter-game@T71 | 内存释放被构造期装载吸收（a.java:4714-4728；见 PORT_NOTES）；v2 证据系漂移锚 |
| m_068 | a.java:7188-7216 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | enter-game@T410; floor51-menu@T570 | 新游戏复位分散移植（a.java:7188-7216；见 PORT_NOTES）；v2 证据系漂移锚 + seed 误读 |
| m_072 | a.java:7266-7283 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | preset-load-menu@T501; menu-sweep@T581 | OptionList::add 已移植（a.java:7266-7283；见 PORT_NOTES）；v2 证据系 28 条 seed 误读 |
| itemTypeToStackIndex | a.java:7702-7716 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | preset-floor5@T265; preset-item21@T351 | item_desc_index 已移植（a.java:7702-7716；见 PORT_NOTES）；v2 证据系漂移锚 |
| deleteRecordStoreQuietly | a.java:9321-9326 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | boot-menu@T68; enter-game@T68 | 删后即写被壳覆盖写吸收（a.java:9321-9326；见 PORT_NOTES）；v2 证据系漂移锚 |
| m_117 | a.java:9328-9360 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | boot-menu@T91; menu-sweep@T151 | RMS 清理无可观测效应（a.java:9328-9360；见 PORT_NOTES）；v2 证据系漂移锚 |
| m_128 | a.java:9818-9828 | 修正暴露 | c 判定存疑（工具方法论） | 绿 | boot-menu@T71; enter-game@T71 | Particles::new 已移植（a.java:9818-9828；见 PORT_NOTES）；v2 证据系漂移锚 |

## 5. 臂级缺口：Rust `todo!` 显式挡路点（方法级绿/黄内的未移植分支）

- crates/game-core/src/scene.rs:931：changeFloor 已达层提示浮层（miscTexts[0/1]，交互批次）
- crates/game-core/src/scene.rs:936：changeFloor 层界提示浮层（miscTexts[2/3]，交互批次）
- crates/game-core/src/scene.rs:1291：tryRunScene case {idx} 的 m_102(23,12) 跨层探针
- crates/game-core/src/scene.rs:1844：executeScriptInstruction 的 {op:?} @op_start={} entry_cursor={} line={line:?}（对应脚本对拍批实现）
- crates/game-core/src/scene.rs:1980：m_106 实体路径缓动（MOV 对拍批）
- crates/game-core/src/scene.rs:5109：paintEntityLayer case 8 的 67 多段拼装
- crates/game-core/src/scene.rs:5166：paintEntityLayer 空图标签分支（objectTypeNames 文本，floor1 不可达）

## 6. 刷新前后对照（v1 伪名口径 → v2.1 逐方法）

v1 取名 bug：`l.split()[-1].split("(")[0]` ⇒ ≥2 参方法被吞成末参类型伪名，构造器成 `a`，`<clinit>` 漏收。下表按伪名列出其吞掉的真实方法与现色。

| v1 名（v1 色） | 真实方法 → 现色 |
|---|---|
| `a`（白） | <init>→绿 |
| `activateItem`（白） | activateItem→绿 |
| `activateMenuItem`（白） | activateMenuItem→绿 |
| `addItemToItemStack`（绿） | addItemToItemStack→黄 |
| `addMenuItem`（白） | addMenuItem→绿 |
| `advanceBobPhase`（白） | advanceBobPhase→绿 |
| `advanceEntityFrames`（白） | advanceEntityFrames→绿 |
| `advanceScriptWalk`（绿） | advanceScriptWalk→黄 |
| `alchemyPriceFor`（白） | alchemyPriceFor→绿 |
| `applyHpDelta`（白） | applyHpDelta→绿 |
| `applyMerchantOffer`（白） | applyMerchantOffer→绿 |
| `applyStepCellEffects`（白） | applyStepCellEffects→绿 |
| `boolean);`（白） | paintWrappedText→绿, paintEntityLayer→绿, changeFloor→绿, tryEquip→绿, tryRunScene→绿, m_131→绿, predictHpLossVsType→绿, predictBattleHpLoss→绿 |
| `buildMinimap`（绿） | buildMinimap→绿 |
| `byte);`（白） | showOverlayMessage→绿, m_093→白 |
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
| `int);`（白） | drawImageClipped→绿, blit5Clip→绿, drawImageWithNokiaTransform→绿, drawEdgePatch→绿, m_006→绿, dimImage→绿, drawIconStrip→绿, layoutFullText→绿, layoutWrappedText→绿, placePlayerAtCell→绿, interactWithCell→绿, stepOffStairs→绿, paintPlayerSprite→绿, paintStatusBar→绿, paintKeySlot→绿, paintHudPanel→绿, paintTitledBox→绿, paintHudFrame→绿, paintBoxFrame→绿, paintMiniFrame→绿, paintNumber→绿, dispatchSpriteSize→绿, dispatchAnimFields→绿, spawnEntity→绿, removeGroupAtCellByType→绿, removeSlotAt→绿, isCellWalkable→绿, m_062→绿, paintTileLayer→绿, setCameraClamped→绿, scaledByFloorTier→绿, m_075→绿, gainGold→绿, m_083→绿, killAdjacentAt→绿, executeScriptInstruction→绿, m_100→绿, m_101→黄, m_102→黄, m_103→绿, m_107→白, m_108→白, addSpark→白, spawnPopup→绿, drawDigitStrip→绿, spawnParticle→绿, findPath→绿, m_134→绿, m_139→绿, runLogoAnimation→绿, registerLogoItem→绿, modifyLogoItem→绿 |
| `int[]);`（白） | m_142→绿 |
| `itemTypeToStackIndex`（绿） | itemTypeToStackIndex→绿 |
| `java.io.IOException;`（白） | readU16BE→绿 |
| `java.lang.String);`（白） | parseScriptInt→绿 |
| `keyPressed`（白） | keyPressed→绿 |
| `keyReleased`（白） | keyReleased→绿 |
| `loadFloorData`（白） | loadFloorData→绿 |
| `loadFloorSave`（绿） | loadFloorSave→绿 |
| `loadLevelScript`（绿） | loadLevelScript→绿 |
| `loadSlotTable`（绿） | loadSlotTable→绿 |
| `lockCameraOn`（绿） | lockCameraOn→绿 |
| `m_000`（绿） | m_000→绿 |
| `m_001`（绿） | m_001→绿 |
| `m_008`（白） | m_008→绿 |
| `m_025`（绿） | m_025→绿 |
| `m_059`（绿） | m_059→绿 |
| `m_068`（绿） | m_068→绿 |
| `m_072`（白） | m_072→绿 |
| `m_076`（绿） | m_076→黄 |
| `m_089`（白） | m_089→黄 |
| `m_091`（绿） | m_091→绿 |
| `m_092`（绿） | m_092→绿 |
| `m_106`（黄） | m_106→白 |
| `m_113`（白） | m_113→绿 |
| `m_117`（绿） | m_117→绿 |
| `m_118`（绿） | m_118→绿 |
| `m_119`（绿） | m_119→绿 |
| `m_120`（白） | m_120→绿 |
| `m_122`（绿） | m_122→绿 |
| `m_123`（白） | m_123→绿 |
| `m_128`（白） | m_128→绿 |
| `m_132`（白） | m_132→绿 |
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
| `saveGlobalProgress`（绿） | saveGlobalProgress→白 |
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

逐方法迁移计数（v1 色 → 现色）：未收→绿 1，绿→绿 40，绿→白 1，绿→黄 3，白→绿 102，白→白 11，白→黄 4，黄→白 1

### 6.1 v1 白项去向（逐真实方法）

- **账面滞后 → 绿 102**：<init>, paint, run, showNotify, drawImageClipped, blit5Clip, drawImageWithNokiaTransform, drawEdgePatch, m_006, dimImage, m_008, keyPressed, keyReleased, resetMenuLayout, addMenuItem, releaseSceneImages, activateMenuItem, drawIconStrip, runIntroViewer, showOverlayMessage, handleOverlayKey, layoutFullText, layoutWrappedText, paintWrappedText, parseHexColor, applyHpDelta, placePlayerAtCell, interactWithCell, applyStepCellEffects, stepOffStairs, paintPlayerSprite, paintStatusBar, paintKeySlot, paintHudPanel, paintTitledBox, paintHudFrame, paintBoxFrame, paintMiniFrame, paintNumber, initEntityTables, dispatchSpriteSize, dispatchAnimFields, spawnEntity, removeGroupAtCellByType, removeSlotAt, advanceBobPhase, paintEntityLayer, sortEntitiesByY, advanceEntityFrames, readU16BE, isCellWalkable, m_062, paintTileLayer, setCameraClamped, findFloorGateEntity, changeFloor, startNewGameLoad, scaledByFloorTier, alchemyPriceFor, m_072, tryOpenChest, applyMerchantOffer, m_075, spendGold, gainGold, floorTier, tryEquip, m_083, activateItem, killAdjacentAt, tryRunScene, executeScriptInstruction, parseScriptInt, m_100, m_103, markSlotEntry, m_113, saveFloorState, deleteRecordStoreQuietly, m_120, loadFloorData, m_123, spawnPopup, drawPopupLayer, drawDigitStrip, m_128, spawnParticle, m_131, m_132, findPath, m_134, predictHpLossVsType, effectiveAttackVsType, predictBattleHpLoss, m_139, closeAudioPlayer, randomBelow, m_142, runLogoAnimation, paintLogoAnimation, registerLogoItem, modifyLogoItem
- **账面滞后 → 黄 4**：hideNotify, m_089, m_101, m_102
- **仍白 11**（分账见 §4/§4.1）：m_093〔b〕, m_107〔b〕, m_108〔a〕, addSpark〔b〕, m_148〔b〕, m_149〔b〕, m_150〔b〕, m_151〔b〕, m_152〔b〕, m_153〔b〕, m_154〔b〕

v1 绿/黄 → 现色变色（v1 过账或口径差，逐条）：advanceScriptWalk（绿→黄）, m_076（绿→黄）, addItemToItemStack（绿→黄）, m_106（黄→白）, saveGlobalProgress（绿→白）

## 7. 前任 v2 口径 → v2.1 差异（同一份 Rust 树与触达快照复算）

| 方法 | v2 色 | 现色 | v2 行锚（朴素归属） | v2.1 行锚 | 人工条目 |
|---|---|---|---|---|---|
| m_001 | 白 | 绿 | 0 | 1 | — |
| m_006 | 绿 | 绿 | 2 | 0 | absorbed |
| m_008 | 白 | 绿 | 0 | 0 | absorbed |
| releaseSceneImages | 绿 | 绿 | 2 | 0 | absorbed |
| spawnEntity | 绿 | 绿 | 4 | 0 | — |
| readU16BE | 白 | 绿 | 0 | 1 | — |
| m_059 | 白 | 绿 | 0 | 0 | ported |
| m_068 | 绿 | 绿 | 3 | 0 | ported |
| m_072 | 绿 | 绿 | 28 | 0 | ported |
| gainGold | 绿 | 绿 | 0 | 1 | — |
| findItemStackIndex | 白 | 绿 | 0 | 0 | ported |
| itemTypeToStackIndex | 绿 | 绿 | 2 | 0 | ported |
| addItemToItemStack | 白 | 黄 | 0 | 1 | ported |
| consumeKeyForDoor | 绿 | 绿 | 1 | 0 | — |
| cameraAtTarget | 绿 | 绿 | 1 | 0 | — |
| m_103 | 白 | 绿 | 0 | 0 | ported |
| deleteRecordStoreQuietly | 绿 | 绿 | 2 | 0 | absorbed |
| m_117 | 绿 | 绿 | 2 | 0 | absorbed |
| m_118 | 白 | 绿 | 0 | 0 | ported |
| m_120 | 白 | 绿 | 0 | 1 | ported |
| m_123 | 白 | 绿 | 0 | 0 | ported |
| addSpark | 黄 | 白 | 1 | 0 | — |
| m_128 | 绿 | 绿 | 1 | 0 | ported |
| m_134 | 绿 | 绿 | 5 | 0 | — |
| predictBattleHpLoss | 绿 | 绿 | 0 | 1 | — |
| <clinit> | 白 | 绿 | 0 | 1 | — |

