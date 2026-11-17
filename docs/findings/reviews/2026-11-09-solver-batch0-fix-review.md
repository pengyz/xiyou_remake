# 求解器批0 修复件复核（红1/黄2/黄3/fac2c65/amend 链）

- 日期：2026-11-09/10 ｜ 复核者：独立复核子代理（只写本报告，未改代码、未动基线、未推送）
- 对象：`df4e3ea..fac2c65`（modern，HEAD=`fac2c6573adfca630ba93422b7dcf3872c8f8c9f`，**全部未推送**）
  链 = `d5aeb7d`（QoL 考证件，未动）→ `3017386`（批0 feat，amend 自 `4084b94`）→ `eb38744`（件5，amend 自 `b38b694`）→ `fac2c65`（lead 单笔，新）
- 方法：**逐条实测**（自写 python 逐字节解析 / `sed` 实文 / `git diff`·reflog 取证 / 三闸独立重跑），不采信报审文字；首审 8 面已绿有据，本轮**只验修复落地 + 有无新伤**
- 前置：首审 `docs/findings/reviews/2026-11-09-solver-batch0-review.md`（总判「需修复后推送：红1」）

---

## 总判：**仍需修复 —— 1 项新增（修复轮自伤的行号锚漂移，注释级 3 行）；7 条修复声称全部实测落地，闸全绿**

| # | 声称 | 判 | 关键实测 |
|---|---|---|---|
| 1 | 红1 修（件5 表行 + 更正注） | 🟢 真落地 | `:70` = `\| 35 \| 10 ×1, 24 ×3, 34 ×3 \|`；自写 python 解析 55 文件 `consumed==filesize` → sprite35 type83 = `[[10],[24]×3,[34]×3]` **7 条**、全表 **52**、不同 param **48 含 24**、文档表逐层与实测**逐 floor 完全一致** |
| 2 | 黄2 修（state.rs 锚 7183-7194 + intro.rs:333 旧锚注） | 🟢 真落地（1 处残留） | 改动落在 `3017386`（`git diff 4084b94 3017386` 仅 state.rs 3+/2-）；`sed 7183,7194p` = 方法声明 + `startLoadTask` + 十步 `6,8,5,13,9,10,2,3,12,11` 同序；**残留：3017386 message 第18行仍写 `a.java:7183-7190`**（首审黄2 的「commit msg 半边」未修，列账） |
| 3 | 黄3 修（message 无「Cargo.lock+7 文件」） | 🟢 属实 | `git log --format=%B 3017386 \| grep -Ei 'Cargo.lock\|7 文件'` → **exit=1（0 命中）**；且该 message 与 `4084b94` **逐字节相同**（该措辞本就只在首审报审文案里，仓库内从未存在） |
| 4 | `fac2c65` 注释级单笔 | 🟢 四项核全过 | `git show fac2c65 -- intro.rs` = **恰 3+/1-，全部 `///` 注释行，零代码**；新锚 333-335 实文正确；Evidence 与我实测相符；**但引入下方「新增」** |
| 5 | amend 链完整 / 未改坏首审绿项 | 🟢 链完整；⚠️ 有一处改坏 | 4 commit 恰好；`d5aeb7d` 同 hash 未动；两笔 amend message 与原版**逐字节相同**；`git status --short -uno` 空；`b38b694..eb38744` 逐文件**仅** state.rs + 件5 文档；⚠️ 但 `fac2c65` 使首审面4a 已绿的两个 state.rs 锚（`intro.rs:334-336`/`361-363`）**+2 漂移失效**（见「新增」） |
| 6 | 三闸独立重跑 | 🟢 全绿 | `gates check` **PASS=13 FAIL=0 SKIP=0**（金样 74 场景 sha 全同）；`cargo test -p game-solver` **4 passed**；`ingest --summary` = **55/502/68/35/32/17**；另补跑 `gates trace` **exit=0**（fac2c65 动了 game-core，按 AGENTS §5 必跑） |
| 7 | ratchet 待更新条目不变 | 🟢 逐字一致 | `{'census_snapshot(test)': 4, 'game_solver(lib)': 0, 'shop_goods_defs(test)': 6, 'solver(bin)': 0}`，`total=1029(基线978)`；基线文件未被本批触碰 |
| **新** | **修复轮自伤：fac2c65 净插 2 行 → state.rs 三处 `intro.rs` 行号锚整体 +2 漂移** | 🔴/🟡 **阻塞** | 见下节「新增」——**破了首审面4a 两条已绿锚**，且 state.rs:3 顶着「行号现场实测」banner |

---

## 逐条实测

### 1 · 红1 修复 —— 🟢 真落地（独立逐字节复算，非文档脚本）

**判据**：`docs/findings/2026-11-09-floor-script-maplv-numbering.md:70` 改为 `10 ×1, 24 ×3, 34 ×3`（7 条）+ 原地日期更正注保留原主张；我方独立按 `SpriteTable::parse` 布局重解析 `assets/raw/sprite35` 并核全表。

**实测**：
- `:70` 实读 = `  | 35 | 10 ×1, 24 ×3, 34 ×3 |`（**7 条**）✓
- `:77-81` 更正注在位，且**原主张原样保留**：引「原记 `10 ×3, 34 ×3`（6 条）」→ 三条错因（10 实为 ×1 / 漏 24×3 / 6≠7 且与 `:75` 全集含 24 自相矛盾）→ 复现口径 → 「原主张保留于本注，不删历史记录（AGENTS §3.6）」✓
- **我方独立 python**（头 = `u16 count_lo + u16 count_hi` 拼 u32；记录 = `type 1B + x u16 + y u16 + extra[extra_len]`，`extra_len(83)=1`；**逐文件断言 `consumed==filesize`**）：
  ```
  files parsed: 55   consumed==filesize: all OK
  total records: 1703
  type83 count: 52
  distinct params: 48
  param set: [0..16, 18, 19, 24..34, 38, 40..50, 52, 53, 56, 64, 66, 67]
  sprite35 type83: [[10], [24], [24], [24], [34], [34], [34]]
  has 24: True count24 = 3
  ```
  → **10 ×1 / 24 ×3 / 34 ×3 = 7 条** ✓；全表 type83 合计 **52** ✓；不同 param **48** 且集合与 `:75` 全集**逐字相同、含 24** ⇒ 表行 ↔ 全集 ↔ 实测**三方自洽** ✓
- **文档表本身求和核**（把 `:64-73` 表逐行解析后与实测逐 floor 比对）：9 个已解析行合计 **43** + 加粗 `51` 行 9 条 = **52**；**逐 floor 与实测 `Counter` 全等**（唯一「MISMATCH」是我脚本没解析 `**51（序章层）**` 加粗行，补上即全等）⇒ 表内**无残留 51/错行** ✓
- 旁证：总记录 **1703**、type83 **52** 与首审面2/面4b 独立复算一致（首审当时已算出 52，只是文档表写错）。

### 2 · 黄2 修复 —— 🟢 state.rs 真改（附 1 处 message 残留）

**判据**：`crates/game-solver/src/state.rs` 锚改为 `a.java:7183-7194` 并注明 `intro.rs:333` 旧锚错；改动确在 `3017386`；`sed` 实文核。

**实测**：
- 改动位置：`git diff --stat 4084b94 3017386` = **仅 `crates/game-solver/src/state.rs`，3 insertions 2 deletions** ✓ 与声称「在 3017386」一致（`eb38744` 是继承，非再改：`git show eb38744:.../state.rs` 与当前逐字相同）。
- 现文（`state.rs:8-10`）：`a.java:7183-7194：:7183 方法声明、:7185-7194 十步同序；intro.rs:333 旧注标 a.java:7172-7179 实测落在 changeFloor(:7135-7181) 尾段，属错锚——game-core 非批0 授权改动面，列账待裁` ✓
- `sed -n '7183,7194p' reference/src/deobf/a.java` 实文：
  `7183 private void startNewGameLoad() {` / `7184 startLoadTask((byte)3, true)` / `7185-7194 stepLoadTask 6,8,5,13,9,10,2,3,12,11`
  —— 十步与 `intro.rs:337` `&[6, 8, 5, 13, 9, 10, 2, 3, 12, 11]` **逐个同序** ✓；`:7185-7194` 恰为十步、`:7184` 是 `startLoadTask` 不计入 ✓
- `sed -n '7170,7181p'` 实文 = `changeFloor` 体内尾段 + `return var5;`（`grep -n` 得 `7135: private boolean changeFloor(...)`、`7183: startNewGameLoad`）⇒ **7172-7179 ∈ changeFloor(7135-7181)** ✓ 与注文一致
- **残留（列账，非本轮声称范围但属首审黄2 半边）**：`git log --format=%B 3017386 | grep -n 7183-719` → 第 **18** 行仍 `（原版 a.java:7183-7190 startNewGameLoad 同序）`；该 message 与 `4084b94` **逐字节相同**（见下条）⇒ amend 只改了树、没改 message。**7183-7190 只覆盖 10 步中前 6 步**，与首审黄2 描述一致，**未修**。message 需 rewrite 历史才能改 ⇒ 建议列账，不阻塞。

### 3 · 黄3 修复 —— 🟢 属实

- `git log --format=%B 3017386 -1 | grep -n -Ei 'Cargo.lock|7 文件'` → **exit=1（0 命中）** ✓
- `diff <(git log --format=%B 4084b94 -1) <(git log --format=%B 3017386 -1)` → **exit=0（完全相同）** ⇒ 首审黄3 的「Cargo.lock+7 文件」措辞**只存在于首审报审文案**，仓库 commit message 从未含它；本轮无需、也未改 message。

### 4 · `fac2c65`（lead 单笔）—— 🟢 四项核全过

- **① diff 恰 3+/1- 且仅注释行**：`git show fac2c65 --stat` = `crates/game-core/src/intro.rs | 4 +++-`（3 insertions, 1 deletion）；逐行看：
  ```
  -    /// `startNewGameLoad`（a.java:7172-7179）：新游戏加载链。
  +    /// `startNewGameLoad`（a.java:7183-7194）：新游戏加载链。
  +    /// 【2026-11-09 更正】原注 7172-7179 系错锚（该区间落在 `changeFloor`(:7135-7181) 尾段），
  +    /// 实测 :7183 方法声明、:7185-7194 十步 stepLoadTask 同序。
  ```
  四行**全为 `///` 文档注释**，无任何可执行代码混入 ⇒ **零行为** ✓（旁证：`gates check` 的 `rust-tests` release 全绿 + `gates trace` exit=0）
- **② 新锚 sed 实文正确**：`intro.rs:333` = 新锚行；`:334-335` = 更正注两行；`changeFloor` 实测 `7135-7181` ✓
- **③ Evidence 与实测相符**：其 `Evidence: sed -n '7183,7194p' …（方法声明+十步同序）；sed -n '7170,7181p' 确认 7172-7179∈changeFloor` —— 我两条命令**分别复跑，输出与描述逐点吻合** ✓；`Confidence: 95` 合理
- **④ 授权表述**：message 写「授权=lead 2026-11-09（批0 越权面请示项，注释级单笔）」——与 state.rs「game-core 非批0 授权改动面，列账待裁」的请示链自洽 ✓
- ⚠️ **副作用未声明**：见「新增」。

### 5 · amend 链完整性 —— 🟢 链干净；⚠️ 1 处「改坏首审绿项」

**实测**：
- `git log --oneline df4e3ea..HEAD` = **恰 4**：`fac2c65 / eb38744 / 3017386 / d5aeb7d` ✓
- **`d5aeb7d` 真未动**：`git rev-parse d5aeb7d` = `d5aeb7d9dd380177fbea707e7b8fcd251bb4b136`，与首审记录（`d5aeb7d` 21:52:13 创建）同一 hash；reflog 自 `HEAD@{16} commit: …QoL…` 起**无任何对该对象的 rewrite** ✓
- **amend 实况（reflog 取证）**：实际只有 **2 笔 amend** + 1 新笔——
  `4084b94 --(23:26:57 commit amend)--> 3017386`；`10ac823 --(23:35:36 commit amend)--> eb38744`；`fac2c65 (23:48:24 commit 新笔)`。
  （若 lead 所称「三笔 amend」另有所指，我在 reflog 中**未找到第三笔**；`d5aeb7d` 未被 amend。）
- **message 原样核**：`diff` 原版 vs amend 版 message → **两对均 exit=0（逐字节相同）** ⇒ 除声明的改动外**零 message 变更**（连黄2 的 message 半边也没顺手改，见残留）
- **树改动核**：`git diff --stat 4084b94 3017386` = 仅 `state.rs`；`git diff --stat b38b694 eb38744` = **逐文件 2 个**：`crates/game-solver/src/state.rs`(3+/2-) + `docs/findings/2026-11-09-floor-script-maplv-numbering.md`(8+/1-) ⇒ **无意外文件** ✓
- **首审绿的其余件字节不变**：因 `b38b694..eb38744` 只动这 2 个文件 ⇒ `Cargo.toml / bin/solver.rs / ingest.rs / lib.rs / tests/census_snapshot.rs / Cargo.lock / QoL 598 行考证件` 全部与首审一致 ✓（抽查 `state.rs:29 Q3_EQUIP_SET = &[36, 40]`、`ingest.rs:3 BUG-053 … bug-ledger.md:1035` 均在位）
- `git status --short -uno` → **空**（tracked 干净）✓
- 越界检查：`git diff --name-only df4e3ea..HEAD | grep -E "reference/|original/|gates/|data/status|data/naming|bug-ledger|master-plan"` → **exit=1（NONE）** ✓；全链被改的**既有**文件只有 `M Cargo.lock`（首审已绿）+ `M crates/game-core/src/intro.rs`（fac2c65，已声明授权）
- ⚠️ **改坏项**：首审面4a 打 ✓ 的两条锚 `intro.rs:334-336` / `intro.rs:361-363` 现已不成立 → 见「新增」。

### 6 · 闸独立重跑 —— 🟢 全绿（含因 game-core 改动补跑的 trace）

| 闸/命令 | 我方实跑结果 | 判 |
|---|---|---|
| `python3 gates/cli.py check` | `PASS=13 FAIL=0 SKIP=0`，exit=0；`state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同`；`rust-tests — cargo test --release 全绿`；`assets-manifest {'maplv':55,'sprite':55}` | ✓ 与报审一致 |
| `cargo test -p game-solver` | `p2s0_s0_and_q3_constants / p2s0_lib_pure_no_io / p2s0_census_snapshot_matches_plan / p2s0_floor_geometry_sizes_present` → **4 passed; 0 failed**（lib/bin/doc 均 0）exit=0 | ✓ |
| `cargo run --release -p game-solver --bin solver -- ingest --summary` | `floors=55 enemies=502 scripts=68 enemies_table=35 items=32 shops=17` + floorsizes 行（`f0..f50/f52..f54=13x13`、`f51=12x13`、`f37:82` 最多、`f0/f22/f50:2` 最少）exit=0 | ✓ 六项与首审/报审逐字一致 |
| **补跑** `python3 gates/cli.py trace`（fac2c65 动 game-core，AGENTS §5 强制） | exit=**0**；`[✓ PASS] diff-trace` | ✓ |
| **补跑** `python3 gates/cli.py status` | `PASS=13 FAIL=0 SKIP=1 TOLERATED=0`（SKIP = `visual-diff 视觉比对器未就绪`，**显式打印**）；含 `reference-regression … refs=707 shim=69 missing=0`、`[oracle] 确定性=PASS 等价=PASS 运行=PASS stderr=PASS C等价=PASS` | ✓ |

### 7 · ratchet 状态 —— 🟢 与声称逐字一致、未动基线

- `gates check` 的 test-ratchet 行逐字：
  `[✓ PASS] test-ratchet — total=1029(基线978) 无回退；新增二进制待 ratchet-update: {'census_snapshot(test)': 4, 'game_solver(lib)': 0, 'shop_goods_defs(test)': 6, 'solver(bin)': 0}`
  —— 条目集合与计数**与声称完全相同**（4 条：census_snapshot 4 / game_solver 0 / shop_goods_defs 6 / solver 0）✓
- 语义仍为**提示不 FAIL**（首审面9 已核 `gates/cli.py:307-332`）；`data/status/` 未被本批触碰（越界 grep NONE）⇒ **基线由 lead 随批独立 commit，我未动** ✓

---

## 🔴/🟡 新增（本轮唯一阻塞项）：`fac2c65` 使 state.rs 三处 `intro.rs` 行号锚 +2 漂移

**机制（已证）**：`fac2c65` 在 `intro.rs:333` 处**净插 2 行**（1 行换 3 行），该行以下全部 **+2**。
用 `git show eb38744:crates/game-core/src/intro.rs | grep -n` 复核「改前」：`334: pub fn new_game()`、`361: pub fn new_game_stats()` —— 即 **eb38744 时 state.rs 的锚是对的（首审面4a 也实测 ✓），漂移确由 fac2c65 引入**，不是本轮之前就错。

| state.rs 现文（声称锚） | 现在实测 | 判 |
|---|---|---|
| `:7` `intro.rs:334-336` = `LoadProgress::new_game()` | 334/335 = 更正注两行、**336 = `pub fn new_game()`**、337 body、338 `}` ⇒ 真范围 **336-338** | **+2 漂**（decl 行仍在 336 内，body/`}` 已脱锚） |
| `:9` `` `intro.rs:333` 旧注标 a.java:7172-7179 …列账待裁`` | 333 现为**已更正的 `a.java:7183-7194` 行**，「7172-7179 旧注」已不存在；且「列账待裁」已被 fac2c65 实际裁掉 | **表述过期 + 指向错行** |
| `:13` `intro.rs:361-363 new_game_stats()` | 361/362 = doc 注、**363 = `pub fn new_game_stats()`**、364 body `(300,10,10,51)`、365 `}` ⇒ 真范围 **363-365** | **+2 漂**（首审当时 361=fn/362=body ✓） |

**连带（非本批文件，同被 +2 推移；仅列账，我未改）**：
- `docs/superpowers/plans/2026-11-09-solver-implementation-plan.md:124` 与 `:274` 的 `intro.rs:361` → 实为 363
- `docs/coverage-matrix.md:262` 的 `fn … crates/game-core/src/intro.rs:366` → 实为 **368**（现 366 是空行）；`:332` 的 `intro.rs:333` 仍指 `startNewGameLoad` 注释行**未漂**、`:391/:392` 的 `:328` 与 `:271` 的 `:35`、`:262` 的 `:231` **均 <334 不受影响**
- **首审前就已漂的旧账（非本轮引入）**：`docs/spec/dead-code-rulings.md:316` DC-8 写 `intro.rs:307`、`docs/spec/architecture-review.md:126` 同写 307 —— 首审已知（其正文注 `现锚=handoff.rs…`），不计入本轮

**定性**：
1. **零行为、零门禁影响**（13 闸 + trace 全绿，release 编译产物同）——纯注释行号锚；
2. 但它是**修复轮自己制造的、可实测证伪的锚错**，且**直接破掉首审面4a 两条已绿项**，属任务书「有没有把首审绿的东西改坏」的命中项；
3. `state.rs:3` 明写「**行号现场实测**」，其 `intro.rs` 锚却已与实测不符——与本批红1/黄2 同类（A 级证据指针失效），故**不建议按「列账后置」处理**。

**修法（1 笔注释级，约 3 行，我按授权只写本报告、未动手）**：
- `state.rs:7` → `intro.rs:336-338`；`:13` → `intro.rs:363-365`；
- `state.rs:9-10` → 改述为「`intro.rs` 旧注 `a.java:7172-7179` 已由 `fac2c65`（2026-11-09）原地更正为 7183-7194」，去掉已过期的「列账待裁」；
- 形态建议：**新起一笔 `docs(solver)` 小 commit**（再 amend `3017386` 会连带 rewrite `eb38744`，成本更高）；顺带可与 ratchet 基线 commit 同批但**不同 commit**（AGENTS §9 一 commit 一逻辑变更）。
- 附带列账（不阻塞）：`3017386` message 第18行 `a.java:7183-7190` 残留（需 rewrite 历史才能改）。

---

## 置信度

| 项 | 置信度 | 依据 |
|---|---|---|
| 红1 修复 + 52/48/sprite35 七条 | **97** | 自写逐字节解析、55/55 `consumed==filesize`、文档表逐层与实测全等；口径与 `game-data/src/sprite.rs` `extra_len` 表一致 |
| 黄2 state.rs 修复（7183-7194） | **95** | `sed` 实文 + `git diff 4084b94 3017386` 定位；message 残留为 grep 0 命中反证 |
| 黄3 | **95** | grep exit=1 + message 与原版逐字节相同 |
| fac2c65 零行为 / Evidence 相符 | **95** | diff 逐行全为 `///`；两条 sed 命令复跑吻合；闸+trace 兜底 |
| 链完整性 / 未夹带 | **95** | reflog + 两组 `diff` exit=0 + `--name-status` 逐文件；d5aeb7d 同 hash |
| 三闸 + ratchet | **95** | 全部本机实跑、逐字比对 |
| **新增锚漂移** | **95** | 改前（eb38744）/改后行号两头实测，机制（净插 2 行）与 3 处漂移一一对应 |
| 「三笔 amend」枚举完整性 | **85** | reflog 只见 2 笔 amend；若 lead 另有所指，我未找到第三笔对象 |

## 复核盲区（未能验到）

1. **未重跑首审 8 面全量**（按任务书只验修复）：依赖红线/变异判别力/TDD mtime 旁证/Q3·BUG-053 消费点等沿用首审结论，仅抽查 `Q3_EQUIP_SET`、`BUG-053` 注在位。
2. **两份考证件（598 + 123 行）未逐行全锚复查**：件5 本轮只动了 1 行 + 注（我逐字节比对 diff 范围），其余锚沿用首审抽验。
3. **`3017386` message 的 `7183-7190` 残留无法在不 rewrite 历史的前提下修** —— 是否接受由 lead 裁；我未做任何 rewrite。
4. **`docs/coverage-matrix.md` 是 `tools/cover_matrix.py` 产物**，其 `intro.rs:366` 漂移可能在下次重生成时自动修正 —— 我**未跑该工具**（会写仓库文件，超出「只写一个文件」授权）。
5. **未跑 `gates cli.py all`**（含 `visual` 等）：`status` 显示 `visual-diff` 为显式 SKIP（比对器未就绪），其余子闸在 `check`/`trace`/`status` 中已绿；本批未触及渲染面。
6. **未验证 `fac2c65` 之后 target/ 之外的构建缓存时间线**；TDD mtime（首审黄1）仍未解，与本轮无关。
7. **未核 `reference/` 侧**（本批零改 reference/original，越界 grep NONE）；`a.java` 行号只抽验了 7135/7170-7194/3482-3491 一带。
