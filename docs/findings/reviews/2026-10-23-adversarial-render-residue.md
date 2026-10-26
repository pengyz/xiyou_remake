# 对抗复核 A：状态切换渲染残留修复有效性（攻击性复核）

> 角色：remake-verifier 纪律 · 对抗复核员 A
> 对象：commit `1700cec`（BootMachine::paint Modern 全画幅清屏）+ `d80d96e`
> 修前现场图：`assets/hd-demo/style-sample/anomaly_continue_menu.png`（title→存档菜单三景错叠）
> 复核日期：2026-10-23
> 纪律约束：不改产品码；破坏性验证必须当场还原；结论按 红/黄/绿 分级并附行号证据。

## §1 清屏真伪（BootMachine::paint Modern 清屏是否 additive-only）

**判：绿（清屏真实且 additive-only），附一条黄（钉不在 game-core，见 §5）。**

- **diff 实证（A 级）**：`git show 1700cec -- crates/game-core` = 仅
  `crates/game-core/src/paint.rs` **+8 行**（行 392-399），零删除、零其他文件；
  清屏以 `if self.layout.preset == crate::layout::Preset::Modern`（paint.rs:396）门控，
  行 398 `g.fill_rect(0, 0, self.layout.screen_w, self.layout.screen_h)`。
  classic 路径（Preset≠Modern）不进臂——**行为只增不减，classic 零漂移**。
  `d80d96e` 未触碰 paint.rs（只 title.rs/native.rs/tests），无叠加清屏风险。
- **DEADBEEF 破坏性验证（A 级，已当场还原）**：
  1. 基线：`cargo test -p game-android --test native_parse modern_paint_no_residual_pixels` → `ok. 1 passed`；
  2. 注释 paint.rs:398 清屏行（`// DEADBEEF-DISABLED`）→ 重跑 → **红**：
     `assertion failed ... 残留 4 px / left: 4 / right: 0`，`test result: FAILED. 0 passed; 1 failed`；
  3. 还原（git diff 空）→ 重跑 → `ok. 1 passed`。
  ⇒ 清屏行是零残留断言的**唯一被测因**，钉有真咬合力，非假绿。
- **附带覆盖面实证（A 级）**：破坏态下 `cargo test -p game-core` 全 21 套件
  **286 测过 +1 忽略全绿**（含 render_golden/hd_visual/style_sample 金样）——game-core 侧
  金样全部不咬 Modern 清屏，回退唯一哨兵=game-android 那根钉（并入 §5 黄项）。

## §2 状态迁移全覆盖（其他状态机 paint 入口有无同类残留病）

**判：黄（迁移链三段当下不残留，但 GameScene 侧同病面未封：入口无清屏+一处真缺口+零钉）。**

- **机器清单（A 级）**：`Machine::paint` 入口仅两处——`BootMachine::paint`
  （paint.rs:390，已加 Modern 清屏）与 `GameScene::paint`（scene.rs:5981，**入口无任何清屏**）。
  壳层不清屏兜底：android 每帧直接画在持久 `st.canvas`（native.rs:954）、
  win `paint_scene`/`repaint` 画在持久 `self.screen` 背板（main.rs:999/988）。
  ⇒ 残留防线**全压在 game-core 各 painter 自铺**，GameScene 与修前 BootMachine 同构风险面。
- **迁移链逐段核对（B 级，逐分支读代码）**：
  - title→存档：BootMachine::EquipDetail（paint.rs:405 起，backdrop+slots+softkeys 无全幅自铺）
    ——修前病灶本体，现由入口清屏兜住 ✓；
  - 存档→游戏：win main.rs:1024 `enter_scene` → `paint_scene` → GameScene 首帧，
    分支全幅自铺 ✓（android 侧 GameScene 未接入，实机链只到 title/存档，native.rs:680 仅 BootMachine）；
  - 游戏→对话→游戏：`paint_dialog`（scene.rs:6795）首行 `paint_parallax(g,true)` 全幅 ✓。
- **GameScene 各分支自铺核查（A 级）**：paint_menu_list:3177 / paint_slot_list:3786 /
  paint_menu5:3953 / paint_mode12:6718 / paint_menu9:4104 / paint_menu15:3854 /
  paint_dialog:6795 全部以 `paint_parallax`（scene.rs:6129，v6 钳制延展=顶/底带补填+列循环铺满
  screen_w，Modern 全幅）开帧 ✓。
- **真缺口（红点①，黄级总评因不在迁移链）**：`paint_ending`（scene.rs:6660）不开 parallax，
  行 6681 顶带 `g.fill_rect(0, 0, SCREEN_W, img_y0)` 用 **classic 常量 SCREEN_W**（240）
  而非 `self.layout.screen_w`（576）——Modern 下顶带右 336px 不铺底，结局图横滚
  （`-ending_scroll`，行 6684）宽度亦按 SCREEN_W 钳 ⇒ **同病残留可达**（Ending 态）。
  邻行 6682 底带反而是 layout.screen_w（不一致本身即硬编码漏网之证）。
- **结构性复发面（黄）**：GameScene 无入口清屏 ⇒ 未来任何新 mode 分支忘开
  parallax/自铺即复发；修法应对称化（GameScene::paint Modern 入口同款清屏或补钉），
  本次不修（对抗复核不改产品码）。

## §3 金样不污染（全画幅清屏对金样帧哈希的影响）

**判：绿（实跑两次全绿，金样零污染）。**

`cargo test -p game-oracle --release` **实跑两遍**（第一遍 441s 级主套件 + 第二遍带
计数落盘 `/tmp/oracle_counts_20261023.log`），两遍 exit 0、0 failed。逐套件报数（第二遍）：

| 套件 | 过 | 忽略 | 败 |
|---|---|---|---|
| unittests src/lib.rs | 0 | 0 | 0 |
| boot_replay | 2 | 0 | 0 |
| fixture_frames | 25 | 0 | 0 |
| gameplay_floor1 | 2 | 3 | 0 |
| l2_prologue | 1 | 0 | 0 |
| preset_cases（帧对拍主套件） | 35 | 0 | 0 |
| probe_floor2 | 1 | 0 | 0 |
| walk_replay_floor1 | 2 | 0 | 0 |
| Doc-tests | 0 | 0 | 0 |
| **合计** | **68** | **3** | **0** |

金样帧哈希（fixture_frames/preset_cases/walk_pure_frames_match/prologue_frames_match
等逐帧对拍）对 1700cec 全画幅清屏**零感知** ⇒ classic 路径像素零漂移的 A 级旁证
（清屏 Modern 门控，金样跑 classic 预设不变式）。注意：金样全绿**不能**证明 Modern
无残留（金样不覆盖 Modern 迁移帧——该面仅由 §5 的单钉守护，见 §5 黄）。

## §4 classic 零漂移（gates check 报数 + classic 路径仅 additive 核对）

**判：绿（总闸 12/12 全绿 + classic 路径 diff 仅 additive + 金样零漂移三重旁证）。**

- **`python3 gates/cli.py check` 实跑报数（A 级，2026-10-23）**：
  `PASS=12 FAIL=0 SKIP=0`（exit 0），逐项：
  original-integrity ✓ / assets-manifest ✓（entries=138）/ reference-seed-integrity ✓
  （8457 行 sha256 对照）/ knowledge-format ✓（48 条）/ docs-links ✓ / naming-ledger ✓ /
  **rust-tests ✓（cargo test --release 全绿）** / test-ratchet ✓（total=597，基线 590，无回退）/
  table-content-hash ✓（三表复跑同哈希 99a92f78/a19c815a/7f5e2e80）/ table-evidence ✓ /
  state-machine-anchors ✓（锚 6+边数对照全命中）/ asset-truth-manifest ✓（138 条三态核）。
  全程无 SKIP/TOLERATE 静默项（check 汇总 SKIP=0）。
- **classic 仅 additive 核对（A 级）**：1700cec 对 game-core 的全部改动=paint.rs +8 行
  且整体 `if preset == Modern` 门控（§1）；classic 分支不进臂、零删除、零改值。
  d80d96e 未触碰渲染路径（title.rs 命中几何 / native.rs 触屏链 / tests）。
- **金样旁证（A 级）**：§3 game-oracle --release 双跑 68 过/0 败（classic 帧哈希）+
  check 内 workspace --release 全绿 ⇒ classic 像素/逻辑零漂移成立。
  ⚠️ 口径声明：classic 零漂移成立**不等于** Modern 正确——Modern 残留面仅 §5 单钉守护。

## §5 钉测覆盖面（DEADBEEF 钉是否只测 BootMachine）

**判：黄（钉有真咬合力但唯一且只罩 BootMachine——单点哨兵 + 破坏面零旁证）。**

- **DEADBEEF 全仓普查（A 级）**：`grep -rn DEADBEEF`（排除 target/）全仓仅
  `shell/android/tests/native_parse.rs` 3 处（:365 注释 / :378 铺脏 / :384 计数）——
  即 `modern_paint_no_residual_pixels`（:364）**一根钉，只测 `BootMachine::paint`**
  （一次 paint 的 title→menu 迁移帧，200 tick 后 576×576 脏画布零残留断言）。
  **是，只在 BootMachine**；`GameScene::paint`/各子 painter 无任何脏画布对拍钉。
- **咬合力反证（A 级，破坏态实测）**：注释清屏行后 `cargo test -p game-core`
  **21 套件 286 测过 +1 忽略全绿**（render_golden/hd_visual/style_sample 金样均不咬）——
  清屏回退的唯一哨兵=此钉（破坏态它红：残留 4 px）。钉真红真绿非假绿，
  但**单点失效面**：钉被删/改断言即无任何检测。
- **双件钉并列（A 级）**：件2 `overlay_rects_match_hit_tables`（:389）覆盖框==命中表
  同源，与清屏钉同套件；suite 实跑 `native_parse` 共 26 测（单钉运行报 `25 filtered out`）。
- **建议（不改产品码，转 porter/verifier 排期）**：①GameScene::paint 补同款
  DEADBEEF 迁移帧钉（至少 InGameMenu/Dialog/SaveSlot 三迁移）；②`paint_ending`
  SCREEN_W 硬编码（scene.rs:6681）补 Modern 顶带全幅断言；③清屏钉升格 game-core
  层（当前壳层测试守护 game-core 行为，层次错位）。

## 总判

**可交付（修复本体有效，classic 零漂移成立）——无红项阻断；2 黄转 porter 排期 + 1 黄转 verifier。**

| 攻击点 | 判 | 一句话 |
|---|---|---|
| §1 清屏真伪 | **绿** | Modern 门控 additive-only；DEADBEEF 破坏性验证 绿→红（残留 4px）→还原绿 |
| §2 状态迁移全覆盖 | **黄** | 迁移链三段不残留，但 GameScene::paint 入口同病面未封 |
| §3 金样不污染 | **绿** | game-oracle --release 双跑 68 过/3 忽略/0 败 |
| §4 gates check | **绿** | PASS=12 FAIL=0 SKIP=0；classic 仅 additive 三重旁证 |
| §5 钉测覆盖面 | **黄** | DEADBEEF 钉唯一且只罩 BootMachine；game-core 286 测破坏态全绿=零旁证 |

**红点清单（不阻断本次交付，但必须登记跟踪）**：
1. `crates/game-core/src/scene.rs:6681` `paint_ending` 顶带 `fill_rect(0,0,SCREEN_W,img_y0)`
   用 classic 常量 SCREEN_W——Modern 576 下顶带右 336px + 横滚图右缘不铺 = **同病残留可达**
   （既有缺陷，非 1700cec 引入，不在本次复核迁移链；无钉无图证故计黄中红点）。

**整改建议（转 porter / verifier 排期，本次不动产品码）**：
- [porter] GameScene::paint（scene.rs:5981）Modern 入口对称化清屏（与 paint.rs:396 同款），
  或至少先修 paint_ending 的 SCREEN_W 硬编码（:6681 顶带对齐 :6682 的 layout.screen_w 写法）；
- [verifier] ①清屏钉升格 game-core 层（现由壳层 game-android 测守护 game-core 行为=层次错位）；
  ②补 GameScene 迁移帧 DEADBEEF 钉（InGameMenu/Dialog/SaveSlot 三迁移 + Ending 顶带断言）。

**复核纪律执行记录**：破坏性验证仅注释 paint.rs:398 一行（`// DEADBEEF-DISABLED`），
验证后即时还原，`git diff` 为空、钉复绿——产品码零改动；所有报数均为本次实跑输出，
无"跑过了"式未贴数声称。遗留假设：GameScene Modern 残留面为**静态推演**（分支覆盖核查），
缺实机/迁移帧钉实证——已如实标注，不混入"已证实"。

**【2026-10-23 并行 WIP 声明】**：21:28 起工作区出现非本复核的未提交产品码改动
（paint.rs/title.rs/native.rs/native_parse.rs，标注【R3】，疑似并行会话工作）。
mtime 对账：game-oracle 报数 21:16:58 落定、gates check 21:23:10 落定、破坏性验证
更早且已还原——**本报告全部数字跑在 1700cec 原树上，未覆盖 R3 WIP**；R3 改动
由其作者另行验证，本复核不认不背。
