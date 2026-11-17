# 求解器批0 对抗复核（game-solver 骨架 + ingest + census + 两考证件）

- 日期：2026-11-09 ｜ 复核者：独立复核子代理（只写本报告，未改代码/未动基线/未推送）
- 对象：`df4e3ea..b38b694`（3 commit：`d5aeb7d` QoL 考证件 598 行 → `4084b94` 骨架+ingest+census 7 文件 287 行 → `b38b694` 三套编号考证 116 行），HEAD=`b38b694`
- 方法：逐面**实测**（读码 / 独立重跑 / 独立 python 重解析 / 真变异 / 构建产物取证），不采信报审文字
- **总判：需修复后推送 —— 红点 1（仅 docs，1 行）；代码 / 门禁 / census / 行号锚 全部实证通过**

---

## 面1 · 依赖红线 —— 🟢 绿

**判据**：`crates/game-solver/Cargo.toml` 只允许 game-data/game-core 单向依赖、无平台 crate；lib 零 fs/net/process/time；「I/O 只在 bin」属实（bin 确有 I/O）。

**实测**：
- `Cargo.toml` 实读：`[dependencies]` 恰 `game-data = { path = "../game-data" }` + `game-core = { path = "../game-core" }`，无其他依赖、无平台 crate；`[[bin]] solver`。
- `cargo tree -p game-solver -i game-solver` → **仅自身一行**（无反向边）。
- 反向边旁证：`grep -rn "game-solver" --include=Cargo.toml . | grep -v crates/game-solver` → **0 命中**（game-core/game-data/shell 均未引用）。
- workspace 收编：根 `Cargo.toml:16` `members = ["crates/*", "shell/*"]`（glob，非显式清单，故无需改根 Cargo）。
- lib 纯度 grep（独立跑，排除 bin 口径）：`grep -rnE "std::fs|std::net|std::process|std::time" crates/game-solver/src --include=*.rs | grep -v bin/solver.rs` → **exit=1（0 命中）**，与报审 A9 一致。
- 「I/O 只在 bin」**属实**：bin 侧确有 `std::env::args`(solver.rs:12)、`std::process::exit`(:15)、`std::fs::read`(:21-22)；`include_str!` 为编译期嵌入（非运行时 I/O）。测试侧 `tests/census_snapshot.rs:16-17` 也有 `std::fs::read` —— 与 `lib.rs:4` 「文件 I/O 只在 `src/bin/solver.rs` 与测试侧」的声明**一致**，口径未被滥用（排除的只有 bin，测试侧照实计入且已被 lib.rs 声明覆盖）。
- 无 I/O 间接面：lib 只调 `Maplv::parse` / `SpriteTable::parse` / `parse_enemies|items|shop_goods` / `spawn_param_and_visibility`（纯解析/纯函数）。

## 面2 · census 权威性 —— 🟢 绿（无自证）

**判据**：独立重跑六项；再用**非 census 代码路径**交叉 2-3 项。

**实测**：
- 重跑 `cargo run --release -p game-solver --bin solver -- ingest --summary` →
  `floors=55 enemies=502 scripts=68 enemies_table=35 items=32 shops=17`（与报审一致），第 2 行 floorsizes：`f0..f50/f52..f54=13x13`、`f51=12x13`、`f37:82` 最多、`f0/f22/f50:2` 最少。
- 独立交叉（**完全绕开 game-solver 代码**）：
  | 项 | 独立路径 | 结果 |
  |---|---|---|
  | floors=55 | `ls assets/raw/maplv* \| wc -l` / `sprite*` | **55 / 55** ✓ |
  | scripts=68 | `awk 'END{print NR}' game-core/tests/fixtures/level_scripts.txt` | **68**（`wc -l`=67，末行无换行；Rust `.lines()`=68 ✓） |
  | enemies_table / items / shops | `python3 -c 'json.load(...)[rows]'` | **35 / 32 / 17** ✓ |
  | enemies=502 | **自己写 python 按 sprite 格式重解析**（`u32 LE 计数` + `type(1)+x(2)+y(2)+extra`，extra_len 表照 `game-data/src/sprite.rs:54`），逐文件断言 `consumed==len` | **type41..75 = 502** ✓，总记录 **1703** ✓（与测试 :52 断言一致） |
- 结论：六项全部被**异构路径**复证，非 census 自证。

## 面3 · TDD 时序（红先行）—— 🟢 绿（旁证充分；直接 mtime 已灭失 → 盲区）

**判据**：commit message 的红证据是否真输出、有无 `$(cat …)` 未展开残留；tests<ingest 顺序是否可独立佐证。

**实测**：
- `git log --format='%H %ci'`：`d5aeb7d` 21:52:13 → `4084b94` 22:23:28 → `b38b694` 22:23:28（后两者同秒，连续操作，无异常）。
- message 红证据 grep：`git log -1 --format=%B 4084b94 | grep 'cat /tmp\|$(cat'` → **exit=1（无残留）**；贴的是真 cargo 输出形态（`panicked at crates/game-solver/tests/census_snapshot.rs:37:5` + `left/right` + `test result: FAILED. 3 passed; 1 failed`），格式与我现场复现**逐字同构**。
- **直接 mtime 证据已灭失**：`crates/game-solver/*` 6 文件 + `Cargo.lock` 全部 = `2026-11-09 22:16:07.191227629`（**7 文件同一纳秒** = 提交前 7 分钟一次批量重写；对比 QoL 文档仍保 `15:44:52.989`、`d5aeb7d` 文件 mtime 未被重置 → 说明 mtime 平时是保真的，这 7 个文件是被**真改写**了）。故 message 里「tests 20:09:19 < ingest.rs 20:09:43」**无法用当前 mtime 直接复核**。
- **构建产物旁证（独立、强）**：
  - `target/debug/.fingerprint/game-core-d8cfd4819a93ff26/invoked.timestamp` = **20:09:19.163** —— 20:09:19 确有一次 cargo 调用；
  - 该时刻**没有任何 game-solver 产物**（最早 game-solver 编译产物 = `incremental/game_solver-*/…` 与 5 个 fingerprint 目录 **20:09:43.646–20:09:43.805**）；
  - 即：20:09:19 那次调用里 game-solver **编译失败**（lib.rs `pub mod ingest;` 找不到文件 ⇒ manifest 已解析、deps 已建、lib 红），20:09:43 ingest 落盘后**首次成功**——与「测试先写、实现后写」的红先行**自洽**，且 20:09:43 正是 message 声称的 ingest.rs mtime；
  - 后续 20:10:40 / 20:10:59 / 20:17:19 release 指纹 = 红复现/门禁跑批时间线；22:16:08 `target/release/solver` = 22:16:07 批量重写后的重建。
- 结论：**红先行证据链自洽且有异源旁证**，但原始 mtime 已被 22:16:07 重写覆盖（列黄+盲区）。红复现本身我已**独立再做两次**（见面8），证明测试非恒真。

## 面4 · S0 / 两考证件行号锚 —— 🟢 锚全中；**🔴 红点 1（件5 实测表错）**

### 4a. S0 锚逐个实测 —— 全中

| 声称锚 | 实测 | 判 |
|---|---|---|
| `a.java:3482-3491` case 8 | `3482 case 8:` / `3484 currentFloor=minFloorReached=51` / `3486 playerHp=300` / `3487 atk=10` / `3488 def=10` / `3489-3491` 三色钥匙=0 | ✓ |
| `intro.rs:334-336` steps | `334 pub fn new_game()` / `335 …&[6, 8, 5, 13, 9, 10, 2, 3, 12, 11]` / `336 }` | ✓ |
| 与 `a.java:7183-7190` 同序 | `7183 startNewGameLoad()` … `7185-7194 stepLoadTask 6,8,5,13,9,10,2,3,12,11` —— **顺序逐个相同** ✓，但**锚范围只盖 10 步中的前 6 步**（7183-7190 到 step10 为止） | ⚠️ 黄2 |
| `handoff.rs:158-163` 落实 | `158 floor=51 /159 min=51 /160 max=51 /161 hp=300 /162 atk=10 /163 def=10`；调用点 `shell/win/src/main.rs:1014 new_game_scene(...)` ✓ | ✓ |
| `intro.rs:361-363 new_game_stats()` 零调用死码 DC-8 | `361 pub fn new_game_stats()` + `362 (300,10,10,51)`；全仓 grep `new_game_stats` 只有定义+文档引用，**零调用方** ✓；`docs/spec/dead-code-rulings.md:316` = DC-8 标题行 ✓；state.rs 声称的现锚 `preset_cases.rs:2722/2941` 实测均为 `sc.persist.floor/min/max=51` ✓ | ✓ |
| `scene.rs:926-929` 脚本上界 | `928 try_run_scene` / `929 if idx >= …scripts.len() { return false }`（fixture=68）✓ | ✓ |

### 4b. 件5（`b38b694`）锚与数字 —— 抽验 10+ 处全中，**除 1 行**

实测命中：`a.java:3187-3188`（currentFloor→loadFloorData）、`a.java:3499-3500`（case10 loadFloorData）、`a.java:9413-9416`（maplv 读取）、`a.java:9497`（sprite 读取）、`handoff.rs:97-98`（`vec![None;56]` + `skip(1).take(54)`）、`sprite_spawn.rs:48-51`（`83 => hide_after_spawn, e0`）、`scene_persist.rs:362-369`（`target<0` / `target>55` 拒）、`script_ops.rs:167 op_gut`、`scene.rs:3806-3819`（73→22(+floor40 boss6) / 74→21 / 75→try_run_scene(20)+20）、`scene.rs:3880-3895`（69+floor35→36 / 70|71+floor49→35 / 72+floor50→65）。

**独立 python 全量复算（我自己的解析，非文档脚本）**：
- 总记录 1703 ✓；type83 出现 **52** 次、**48 个不同 param**，全集 `[0..16,18,19,24..34,38,40..50,52,53,56,64,66,67]` **逐字相同** ✓
- 空槽 `[39,58,59,60]` ✓；GUT 目标 `{1,12,37,57,61,62,63}` ✓；被引用槽并集 **59** ✓；非空无显式入口 `[17,23,51,54,55]` ✓
- 楼梯：floor50 唯一无楼梯 ✓；floor0/floor44 从未被指向 ✓
- 逐层 type83 分布表（§3.1）**24 行中 23 行逐字相同**——**只有 floor35 行错**（见红点1）
- §4 的 `48 ∪ 7 ∪ 6 = 59`、§5 三条假设均显式标「假设/待考」✓（`docs/findings/…numbering.md:96-104`）

### 🔴 红点1：件5 §3.1「按层分布（实测，48 个不同 param）」floor35 行错

- 原文 `docs/findings/2026-11-09-floor-script-maplv-numbering.md:70`：
  `  | 35 | 10 ×3, 34 ×3 |`（6 条）
- 字节级复现（独立解析 `assets/raw/sprite35`）：
  ```bash
  python3 - <<'EOF'
  import struct
  def el(t): return 2 if t in (7,8,76,82) else (1 if t in (4,5,9,57,59,70,71,72,73,77,78,81,83) else 0)
  d=open('assets/raw/sprite35','rb').read(); cnt=struct.unpack('<I',d[:4])[0]; off=4; out=[]
  for _ in range(cnt):
      t=d[off]; off+=5; ex=d[off:off+el(t)]; off+=el(t)
      if t==83: out.append(list(ex))
  print(out)
  EOF
  # → [[10], [24], [24], [24], [34], [34], [34]]   共 7 条
  ```
- **真值 = `10 ×1, 24 ×3, 34 ×3`（7 条）**；文档写成 `10 ×3, 34 ×3`（6 条）——①把 10 误记为 ×3 ②**漏掉 24 ×3**。
- 自相矛盾证据：同一文档 `:75` 的 param 全集**含 24**，但分布表里 24 无任何 floor 归属；且文档表内 type83 合计 51 ≠ 实测 52。
- 定性：这是**标着「实测 / 已证实（A 级）」却通不过字节复现**的条目，直接触犯 AGENTS §3（A 级必须可复现）。结论面（三套编号、边A/B/C、§4 入口完备性）**不因此改变**，但表本身必须原地更正（AGENTS §3.6 带日期更正）。
- 修法（1 行）：`| 35 | 10, 24 ×3, 34 ×3 |`（并在表下注一句「2026-11-09 复核更正：原记 10×3 漏 24×3」）。

### 4c. QoL 考证件（`d5aeb7d`，598 行）抽验 —— 🟢 绿

- 结构纪律：头部显式声明 A/B/C 分级（`:4`），每节带「存疑点/假设区」（`:281/:380/:433/:509`），C 级显式标注（如 `:588`「C 级，未运行时验证」）——**未见 C 写成已证实**。
- 锚抽验：`a.java:7790 activateItem` / `7798-7806 case15/23/24/25 落 default 空分支` ✓；`a.java:944 itemUseCounts` ✓；`a.java:7868-7872 case22 = changeFloor(currentFloor−1)` ✓。四条抽验全中。

## 面5 · Q3 / BUG-053 红线 —— 🟢 绿

- `crates/game-solver/src/state.rs:28`：`pub const Q3_EQUIP_SET: &[i32] = &[36, 40];` —— **恰 [36,40]**（用户裁定 A），注释 `:24-27` 说明 79/80 次级档不入禁 ✓；测试 `tests/census_snapshot.rs:58` 断言 `&[36,40]` ✓。
- `ingest.rs:3-5` 红线注释**真写**，引 `docs/bug-ledger.md:1035` —— 实测 `1035` 行正是 `## BUG-053 items.json effect 映射错位（22-25 四件·Stage B）` ✓。
- **effect 消费点**：`grep -rn "effect" crates/game-solver/` → **只有 `ingest.rs:3` 那条注释**；`ingest()` 只按 `it.id` 取外键（`:80` `items.iter().map(|it| it.id)`），**没有任何地方把 `ItemDef.effect` 当效果源** ✓。字段随 `Vec<ItemDef>` 被携带但未被消费（红线禁的是「作效果源」，不违规）。

## 面6 · 夹带 —— 🟢 绿

- `git diff --stat df4e3ea..b38b694` = **9 行文件**：`Cargo.lock(+8)`、game-solver 6 源文件（`Cargo.toml 13 / bin/solver.rs 50 / ingest.rs 103 / lib.rs 9 / state.rs 28 / tests/census_snapshot.rs 76`）、2 个 findings 文档（598 + 116）——与三 commit 声称**完全吻合**。
- 逐 commit `--name-status`：`d5aeb7d` 只 A 那 598 行文档；`4084b94` = `M Cargo.lock` + 6 个 A；`b38b694` 只 A 116 行文档。
- 越界检查：`git diff --name-only df4e3ea..b38b694 | grep -E "game-core|master-plan|bug-ledger|gates/|data/status|reference/|original/"` → **NONE_TOUCHED**（本批零改 game-core / 计划 / 台账 / 门禁 / 基线 / 参考版 / 原始归档）✓。
- `git status --porcelain --untracked-files=no` → **空**（tracked 干净）✓；工作区仅历史遗留 untracked 的 reviews/*.md 与 `android/build/`（非本批产物）。
- 报审文案小误：称「Cargo.lock+7 文件 287 行」，实为**共 7 文件含 Cargo.lock**（6 源文件+lock），行数 287 ✓（见黄3）。

## 面7 · 闸独立重跑 —— 🟢 绿

- `python3 gates/cli.py check`（我本机全程重跑）：
  `PASS=13 FAIL=0 SKIP=0`；`rust-tests — cargo test --release 全绿`；`state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同`；`assets-manifest {'maplv': 55, 'sprite': 55}`；`docs-links 全部相对链接有效`（覆盖两个新文档）。
  **FAIL 点名：无**。与报审「PASS=13 FAIL=0、金样 74 GREEN」逐字一致。
- `cargo test -p game-solver` → **4 passed; 0 failed**（含 doc-tests 0）✓。

## 面8 · 变异判别力 —— 🟢 绿（两次真变异，均红→还原→绿）

1. **改 expected**：`(55,502,68,35,32,17)` → `…,16` ⇒ `p2s0_census_snapshot_matches_plan FAILED`；`sed` 还原后 `git diff --stat crates/game-solver` 空 ✓。
2. **改实现（stub census.floors=0）** ⇒
   `panicked at crates/game-solver/tests/census_snapshot.rs:37:5 / left: (0, 502, 68, 35, 32, 17) / right: (55, 502, 68, 35, 32, 17) / FAILED. 3 passed; 1 failed`；
   `git checkout -- crates/game-solver/src/ingest.rs` 后 `git status --porcelain --untracked-files=no` 空 ✓。
- 结论：测试**对 expected 与实现两侧都敏感**，非恒真桩；我复现的 panic 位置/形态与 4084b94 message 贴的红证据**同构**（其 left 全零 = 六字段全 stub 的那次复现）。

## 面9 · ratchet 提示 —— 🟢 绿（如实、不阻塞）

- 实跑 `gates check` 的 test-ratchet 行**逐字**：
  `[✓ PASS] test-ratchet — total=1029(基线978) 无回退；新增二进制待 ratchet-update: {'census_snapshot(test)': 4, 'game_solver(lib)': 0, 'shop_goods_defs(test)': 6, 'solver(bin)': 0}`
  —— 与报审**完全一致**（4 个条目、计数全对）✓。
- 语义核（`gates/cli.py:307-332`）：`newbins = 当前有、基线无` 只挂**提示**，不产生 FAIL ⇒ **不阻塞推送**。
- 归属：`data/status/baselines/rust-test-counts.json`（total=978）里 4 个 key **全缺**；但 `shop_goods_defs(test)` 所在的 `crates/game-data/tests/shop_goods_defs.rs` 由 `c2dbd9c`（P2 批9）引入且 `c2dbd9c` **是 `df4e3ea` 的祖先**、`df4e3ea` 时基线已缺它 ⇒ **该条是批0 之前的既有欠账**，批0 只新增 3 条（`census_snapshot(test)` / `game_solver(lib)` / `solver(bin)`）。
- 处置（按任务要求只报告、不动基线）：**可后置**，但更新须走 `python3 gates/cli.py ratchet-update` + **独立 commit 正文写明原因与影响**（AGENTS §2 / `gates/cli.py:336` 注释同款要求）；且顺带应把批9 欠的 `shop_goods_defs` 一并入基线（total 978→1035）。

---

## 总判

**需修复后推送（红点 1，仅 docs，1 行）；除此之外本批无任何阻塞项。**

| 级 | 项 | 处置 |
|---|---|---|
| 🔴 红1 | 件5 `…floor-script-maplv-numbering.md:70` floor35 行实测错（真值 `10, 24×3, 34×3` 7 条，文档 `10×3, 34×3` 6 条，漏 `24×3`），与同文 `:75` 全集自相矛盾，标 A 级却过不了字节复现 | 推送前（或作为紧随的独立 commit）**原地 1 行更正 + 日期更正注**（AGENTS §3.6） |
| 🟡 黄1 | TDD 直接 mtime 证据灭失：7 文件+`Cargo.lock` 同纳秒 `22:16:07` 批量重写，message 的 `20:09:19<20:09:43` 只能靠构建产物旁证（20:09:19 有 cargo 调用无 game-solver 产物=编译红；20:09:43 首次成功=声称的 ingest mtime） | 列账；后续大改尽量避免提交前统一重写 mtime |
| 🟡 黄2 | `state.rs`/commit msg 引锚 `a.java:7183-7190` 只覆盖 10 个 `stepLoadTask` 中前 6 个（真范围 7183-7194；**顺序结论本身实测正确**）；`intro.rs:333` 既存注释锚 `a.java:7172-7179` 亦为旧漂移（非本批） | 列账，下次改该文件时顺手扩到 7194 |
| 🟡 黄3 | 报审文案「Cargo.lock+7 文件 287 行」= 实为**共 7 文件**（6 源 + lock）287 行 | 文案勘误，仓库无误 |
| 🟡 黄4 | ratchet 提示含批0 之前的 `shop_goods_defs(test):6` 欠账（`c2dbd9c` 引入、起点即欠） | 列账；ratchet-update 独立 commit 时一并补 |
| 🟢 绿 | 面1 依赖红线 / 面2 census 六项异构复证 / 面3 红先行旁证 / 面4 行号锚 10+ 处全中 / 面5 Q3+BUG-053 红线 / 面6 零夹带 / 面7 闸 13 PASS 0 FAIL / 面8 双向变异红 / 面9 ratchet 如实不阻塞 | 全部实测通过 |

**置信度**：红1 = **95**（字节级复现，独立解析，且与文档自身全集矛盾）；面2/面5/面6/面7/面8 = **95**（全部独立重跑）；面3 = **80**（旁证强但原始 mtime 已灭失，无法 100% 还原写入顺序）；面4 锚 = **90**（抽验 15+ 处全中，但两份 714 行考证件未逐行全查）。

## 复核盲区（未能验到）

1. **20:09 原始文件 mtime**：被 22:16:07 批量重写覆盖，`~/.bash_history` 停在 11-04（无本会话记录）→ 「测试先于实现落盘」只能靠构建产物旁证，不能直接复核。
2. **22:16:07 批量重写的动机未取证**（疑提交前统一润色/格式化；7 文件同纳秒说明是单次批量操作）——未找到该次操作的日志。
3. **QoL 598 行文档未逐锚全查**（抽验 4 处全中：activateItem 空分支 / itemUseCounts / case22 changeFloor / 分级结构）；其余 ~500 行锚未验。
4. **件5 §2/§3.2 的运行时行为**（槽 0/55 恒 `None`、`change_floor` 越界拒、战斗触发在运行时是否真被走到）只做了静态读码 + 字节分布，**未跑 trace/差分**（本批也未改 game-core，按 AGENTS §5 无需跑 trace 闸）。
5. **只变异了 census 测试两处**（expected / 实现）；`p2s0_floor_geometry_sizes_present` 的 1703 断言、`p2s0_lib_pure_no_io`、`p2s0_s0_and_q3_constants` 未做变异。
6. **未跑 `gates/cli.py all` 全量**（trace/visual/reference 子闸）——本批 diff 零触及 game-core/reference/original/gates，`gates check` 13 项（含 rust-tests release + state-baseline 74 金样）已绿，判断为不适用；若 lead 要求里程碑级全量，需另跑。
7. **未验证 release 档下 4 测试的独立输出**（我跑的是 debug 档 `cargo test -p game-solver`；release 档由 `gates check` 的 `rust-tests` 覆盖并 PASS）。
