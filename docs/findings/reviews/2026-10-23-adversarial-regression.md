# 对抗复核 C：Android 呈现批回归面攻击性复核（2026-10-23）

> 角色：remake-verifier 纪律（对抗复核员 C）
> 对象：commit 链 9897357 / 16416cd / 88f4c6f（shell/android/**）
> 范围：classic 零漂移 + 全量测试真实性（回归面攻击）
> 纪律：不改产品码；只跑测试与取证。证据分级 A/B/C 遵循 AGENTS.md §3。

## 攻击点 1：classic 零漂移独立实证

- 状态：全部实证完毕
- 1a. `python3 gates/cli.py check` 计数：**12/12 PASS**（PASS=12 FAIL=0 SKIP=0，
  check-exit=0）——original-integrity/assets-manifest/reference-seed-integrity/
  knowledge-format/docs-links/naming-ledger/rust-tests/test-ratchet/table-content-hash/
  table-evidence/state-machine-anchors/asset-truth-manifest 全 ✓。**A 级**。
- 1b. `cargo test -p game-oracle --release`：**exit 0 全绿（71/71）**。金样帧哈希核心
  `preset_cases(test)` **35 passed**（369.7s，逐 tick `frame_sha` 比对，T1-500/T1-800
  全帧一致断言）+ fixture_frames 25/boot_replay 2/gameplay_floor1 5/l2_prologue 1/
  probe_floor2 1/walk_replay_floor1 2 全绿 ⇒ "35+ 帧哈希"声称**成立**。**A 级**。
- 1c. `git diff 9897357~1..HEAD --stat -- crates/game-core data/golden reference original` 是否为空：
  **绿——空**。全距 diff 仅 4 文件且全在 `shell/android/`
  （Cargo.toml +7 / build.rs +45 / src/native.rs +742 / tests/native_parse.rs +245）；
  `crates/game-core`、`data/golden`、`reference/`、`original/` 零改动。**A 级实证**。
- 备注（黄·链名不实）：任务名链 `9897357/16416cd/88f4c6f` 与 HEAD 一父链不一致——
  HEAD 链为 `a45af6d → 16416cd → 88f4c6f`；`9897357` 是首件同题异本（另一对象，
  diff 9897357→a45af6d = build.rs +45/tests +146 补强）。两者父提交同为 `0f50fd1`，
  故 1c 命令取 `9897357~1` 仍等价于 `0f50fd1`，零漂移断言有效，但链编号需更正为 a45af6d。
- 判定：1c **绿**（A 级）；链名 **黄**（不影响零漂移结论）；1a/1b 待实跑。

- 判定：1a **绿**（12/12）；1b **绿**（金样 35 帧哈希+全 71 绿）；1c **绿**（A 级）；
  链名 **黄**（不影响零漂移结论）。整体攻击点 1 = **绿**。

## 攻击点 2：宿主测试全量实跑

- 状态：实跑完毕，逐套报数如下（`cargo test -p game-android -p game-shell-common
  -p game-desktop --release`，exit 0）：
  | 测试二进制 | 实跑 | 基线 | 对账 |
  |---|---|---|---|
  | game_android(lib) | 5 ✓ | 未入账（盲区，见 5） | 新增 |
  | input_mapping(test) | 16 ✓ | 16 | 平 |
  | lifecycle(test) | 9 ✓ | 9 | 平 |
  | native_parse(test) | 18 ✓ | 未入账（新二进制） | 新增 |
  | storage_wiring(test) | 4 ✓ | 4 | 平 |
  | stubs_contract(test) | 11 ✓ | 11 | 平 |
  | track_loading(test) | 4 ✓ | 4 | 平 |
  | **game-android 小计** | **67/67** | — | **成员声称 67/67 ✓ 对账上** |
  | game_shell_common(lib) | 26/26 ✓ | 26 | 平（4a 收口） |
  | game-desktop xiyou(bin) | 40/40 ✓ | 40 | 平 |
  | **三 crate 合计** | **133/133 全绿** | — | 零失败 |
- 67 分解对账：47 保真面（input_mapping 16+lifecycle 9+storage_wiring 4+
  stubs_contract 11+track_loading 4=44 集成 + native.rs lib 3）+18 解析/几何/策略钉
  （native_parse）+2 装配自检（input.rs lib）= **67** ✓ 与 commit 声称吻合。
- 判定：**绿**（A 级，实跑 0 failed；67/67 声称属实）。

## 攻击点 3：TDD 红证据抽查

- 状态：已实证（含双红复现）
- 3a. 88f4c6f commit message 红证据三件套：**在**。①红测试代码先行（6 钉清单）+
  ②红输出留痕（编译红 E0425 `cannot find function fit_rect` + E0061 `blit takes 6
  arguments but 8 supplied`，10 错；二批 E0432 `unresolved import pick_resolution`
  + E0061 `map_touch_xy 4 vs 6 arguments`，5 错）+ ③commit 本体绿（67/67）。
- 3b. 16416cd commit message 红证据三件套：**在**。①tests/native_parse.rs 二钉先行
  （present_null_window_noop / surface_negotiate_null_window_noop）+ ②红输出留痕
  （E0600 `cannot apply unary ! to ()` + E0599 `no method named ensure_surface`）
  + ③绿 61/61。
- 3c. worktree 复现红（独立实证，`/tmp/adv-c-red`@88f4c6f）：
  - 刀 A（stride 跳行改坏：`y*stride`→`y*buf_w`）⇒ `blit_row_pitch_uses_stride_not_width`
    **红**，断点=行 0 尾隙断言（`width*4 处=行 0 尾隙，不得错位连写`，left
    [17,34,51,255]≠right [0,0,0,0]）——恰是 Bug1 病灶；同批 17 测绿 ⇒ 钉真钉、非全员脆。
  - 刀 B（map_touch_xy 负坐标钳 0）⇒ `touch_inverse_maps_through_game_rect` **红**，
    断点=「侧余触控区落在画布外（负坐标）」断言。
  - 双红均 0.02s 级复现，主工作树未动（`git worktree` 隔离）。
- 判定：**绿**（留痕在 commit message 内、非独立 artifact，属留痕形态偏弱但三件套齐；
  以双红复现补强——钉有效 A 级）。

## 攻击点 4：触控回归

- 状态：全部实证完毕
- 4a. T3 触点反变换（侧余负坐标）后 shell/common 原 26 测：**26/26 实跑全绿**
  （touch 8+undo_ui 3+audio 7+lib 6+rms 1+settings 1，`game_shell_common(lib)` 
  0 failed）——侧余负坐标改动在 android 侧 `map_touch_xy`，shell/common 零改动零扰。
  附红复现佐证：负坐标被钳 0 时 `touch_inverse_maps_through_game_rect` 立红（刀 B）。
  **A 级**。
- 4b. grid≠13 假布局注入测试：**在**（A 级，逐条）：
  - `landscape_matrix_game_rects`：13/18/15/37 格四档注入（fit_rect 416/576/480/1184）；
  - `fit_rect_overflow_shrinks_proportionally`：37 格@2x 越界注入；
  - `touch_inverse_maps_through_game_rect`：grid=18 注入（576 画布）+ 侧余负坐标断言；
  - `pick_resolution_matrix`/`pick_resolution_prefers_largest_scale`：假分辨率表
    注入 grid 18/22/24/15（≠13）+clamp 13 上下限。
  - 注入点=显式参数（map_touch_xy/fit_rect/pick_resolution 入参）+ `GRID_N/SCALE`
    常量（源 35-37 行，默认 18/2——非 13 常量本身即假布局面）。
- 4a. 26/26 全绿实证后收口。
- 判定：4a **绿**（A 级）；4b **绿**（A 级）。整体攻击点 4 = **绿**。

## 攻击点 5：ratchet 增量

- 状态：实证完毕（含门禁盲区新发现）
- 5a. 基线 `data/status/baselines/rust-test-counts.json`：47 键，sum=564=total 字段
  ✓ 自洽；`game_shell_common(lib)`=26、android 老五套均在账。
- 5b. 实跑对账（全绿，零回退——所有在账二进制计数均 ≥ 基线且恰平）：
  - 本批新增：`native_parse(test)` = 18（新二进制）；`game_android(lib)` 1→3+
  input 2=5（native.rs 单测 +2，历史 3 测含 input 2+native 1）。
  - 67 声称 = game-android 67 ✓（见 2 节）；相对 564 基线净增 20 测（native_parse 18
  + game_android(lib) +2），另 3 测为基线前欠账。
  - **黄·ratchet 盲区（本次新发现，A 级机械实证）**：`gates/cli.py::collect_test_counts`
  以 `target.kind ∈ ("lib","bin","test")` 过滤 compiler-artifact——game-android 的
  lib 产物 kind=`['cdylib','rlib']`（crate-type 双型）被整条丢弃 ⇒ 其 5 测**不进
  ratchet 任何账目**（基线 564 与门禁现计 582 均不含）。实证：`cargo test -p
  game-android --release --no-run --message-format=json` 显示 `game_android
  ['cdylib','rlib']`；门禁 detail=`total=582(基线564)…新增二进制 {'native_parse(test)': 18}`
  ——582+5=587 才是真值。**后果**：若 game_android(lib) 5 测被删/嵌套化静默失效，
  ratchet 不会红（其防删测保护在此盲）。无现实回归（rust-tests 门禁实跑覆盖到、全绿），
  属门禁保护面缺口。修复归 gates/ 单独 commit（改 kind 过滤接受 cdylib+rlib lib 产物
  并重跑 ratchet-update 补账），按纪律与功能变更分离。
- 5c. 待 update 项清单（基线更新须单独 commit 写明原因，AGENTS.md §2）：
  1. 新增 `"native_parse(test)": 18`（门禁已提示）；
  2. **补欠账** `"game_android(lib)": 5`（需先修 collect_test_counts 盲区，否则
  ratchet-update 重建仍收不进）；
  3. `"total"` 564 → 587（真值口径）；若暂不修盲区则门禁口径为 582，须在 commit
  注明差额 5 的来源；
  4. 复核无回退已做：47 个在账二进制全部 ≥ 基线（恰平），**无一降**。
- 判定：**黄**——无回退（A 级）、增量自洽，但 (a) 基线未入账（update 待办在案）
  + (b) ratchet 盲区使 game-android lib 测试不在保护面（门禁改进待办）。

## 总判

- 判定：**可交付（零红）**，带 3 黄项跟进。
- 红项清单：**无**。
- 黄项清单：
  1. **ratchet 盲区**（攻击点 5）：`collect_test_counts` 漏计 game-android lib 测试
     （kind=['cdylib','rlib'] 被过滤）——5 测（含本批 +2）不在防删测保护面；
     修复归 gates/ 单独 commit + ratchet-update 补账（native_parse(test): 18、
     game_android(lib): 5、total 564→587）。
  2. **链名不实**（攻击点 1 备注）：任务/复核对象称 `9897357/16416cd/88f4c6f`，
     HEAD 一父链实为 `a45af6d → 16416cd → 88f4c6f`；9897357 是首件同题异本
     （同父 0f50fd1，差 build.rs+45/tests+146）。零漂移断言不受影响，后续引用应改 a45af6d。
  3. **Cargo.lock 未随批入账**（本次跑测副产物暴露）：88f4c6f/a45af6d 的
     `shell/android/Cargo.toml` 新增依赖（cc/game-core/game-data）未配套提交
     `Cargo.lock` 更新——任何 `cargo test/build` 都会把工作树弄脏（本复核实跑后
     `git diff Cargo.lock` 即 +3 依赖行）。属提交卫生，建议单独 `chore` commit
     入账 lock（复核未动产品码，工作树留待 owner 处置）。
- 实跑计数汇总：
  - `python3 gates/cli.py check` = **12/12 PASS**（exit 0）；
  - `cargo test -p game-oracle --release` = **71/71 绿**（含 preset_cases 35 金样帧哈希，
    369.7s）；
  - `cargo test -p game-android -p game-shell-common -p game-desktop --release` =
    **133/133 绿**（game-android 67/67 + shell-common 26/26 + game-desktop 40/40）；
  - worktree 双红复现：stride 改坏 → 1 红（行 0 尾隙钉）；负坐标钳 0 → 1 红（侧余钉）。
- classic 零漂移：**真**（game-core/golden/reference/original 零改动，A 级 diff 空证）。
- 成员声称核验：67/67 ✓、35+ 帧哈希 ✓、12/12 ✓、grid≠13 注入测试在 ✓、
  红证据三件套在 ✓（留痕形态偏弱但双红复现补强）。
