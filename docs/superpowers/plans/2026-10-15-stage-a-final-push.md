# Stage A 收官三役实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
> 【本计划执行方式已由用户指定：**Native 全部落地**（2026-10-15「制定计划全部落地」，且明令不再派外部子代理）⇒ 直接用 superpowers:executing-plans 在本会话内实施。】

**Goal:** 关闭 Stage A 全部剩余缺口（判据①最后一项 saveGlobalProgress + 已知 `_ => {}` 静默吞噬的型 10/11/12 撞击臂），随后 P4 终审并 tag `stage-a-complete`。

**Architecture:** ①SKY_WAR：`save::GlobalSave`（格式层已落地）为唯一载体，BootMachine/GameScene 各持一份拷贝 + dirty 位（与 Java 单类共享字段同构；双拷贝风险由同一结构体与同一更新助手收敛），壳按 dirty 轮询持久化 `SKY_WAR.bin`、启动时装载、损坏走 Java 重置分支。②撞击臂：在 scene.rs 的 step/interact 型分派补 10/11/12 三臂（语义以 a.java 字节码 + fx_firebump_floor13 参照帧为准）。③P4：全量门禁 + oracle 全套件 + 账面终态 + tag。

**Tech Stack:** Rust（game-core 无 I/O 确定性 / game-desktop 壳）、Java 参照版（差分裁判）、gates/cli.py 门禁。

**Spec:** docs/spec/convergence-plan.md §0 四条硬判据；docs/spec/audio.md §4（SKY_WAR 关联）；e5391aa 裁决书与 A 任务交棒报告（2026-10-15）；BUG-008（docs/bug-ledger.md）。

## Global Constraints

- `original/` 任何字节禁改；`reference/` 只许语义等价变更，提交必须含 `Evidence:`/`Confidence:`（AGENTS §2/§3/§9）。
- game-core 无 I/O、确定性：只用虚拟时钟与 GameRng；持久化动作全在壳侧。
- 帧对拍零回退：任何任务结束 `cargo test --release --workspace` 全绿且 ratchet 只增不减。
- 验证一律无头（Xvfb/game-oracle/纯函数测试），禁止碰用户桌面（AGENTS §5）。
- D4：保真保留原版 bug（含 SKY_WAR 读失败重置语义、水位 `<51` 条件）。
- 命名改动须查 `data/patches/vf-projection/` 锚点共维（知识库 gotcha_renaming-breaks-vf-patch-anchors）。

## Review Focus

1. **SKY_WAR 截断/损坏 blob 不得 panic**——Java 走 catch → `optionChecked[1]=true` 重置 + 重写（a.java:4475-4478）；Task 1 的 `global_reset_corrupt` 测试钉住。
2. **水位条件两处写法差异**：dispatch case1 用 `watermark < maxFloor`（4297），换层用 `maxFloor > watermark`（7163），且都带 `maxFloor < 51`（51 层结局不计）；Task 1 的 `watermark_bump_conditions` 测试钉住 50/51 边界与两处入口。
3. **声音勾选 ↔ soundEnabled 镜像**（a.java:7284 `optionChecked[optionCount-1]=soundEnabled`，本作 optionCount=2 ⇒ [0]=声音）；Task 1 的 `settings_toggle_marks_dirty` 钉住 [0]/[1] 与 BgmState/minimap_enabled 同步。
4. **撞击臂逐值一致**：型 10/11/12 的位移/镜像/数值取自字节码，fx_firebump_floor13 从 T301 起像素判据 + 变异击杀（撤臂必红）；Task 2 钉住。
5. **既有对拍不受持久化接线扰动**：slot_equip_scene/mov_route_scene/preset_cases 35 项保持全绿；Task 1 的全量回归步钉住。

---

### Task 1: SKY_WAR 全局存档接线（saveGlobalProgress 移植收口）

**Files:**
- 已落地（本轮前置）：`crates/game-core/src/save.rs`（`GlobalSave` / `parse_global_save` / `serialize_global_save` + 3 测试）
- Modify: `crates/game-core/src/paint.rs`（BootMachine，`pub struct BootMachine` @142）
- Modify: `crates/game-core/src/scene.rs`（GameScene 状态与两处水位/触发点）
- Modify: `crates/game-desktop/src/main.rs`（壳持久化：启动装载 + dirty 轮询写）
- Test: 各文件 `#[cfg(test)]` 模块（game-core 两处 + game-desktop shell_tests）

**Interfaces:**
- Consumes: `crate::save::{GlobalSave, parse_global_save, serialize_global_save, GlobalSave::default_7}`。
- Produces（两机同构，供壳轮询）：
  - `impl BootMachine { pub fn global_snapshot(&self) -> GlobalSave; pub fn take_global_dirty(&mut self) -> bool; pub fn apply_global(&mut self, g: GlobalSave); }`
  - `impl GameScene { 同上三方法；另 pub fn note_max_floor(&mut self, floor: i32)（换层水位抬升，a.java:7163-7164）}`
  - 壳侧：`fn persist_global(&mut self)`（dirty ⇒ snapshot ⇒ serialize ⇒ 写 `<save>/SKY_WAR`）；启动 `load_global()` 语义 = `parse_global_save` 失败 ⇒ `GlobalSave::default_7()` 并回写（Java 4475-4478）。

- [ ] **Step 1: 写失败测试（game-core 状态与触发）**

```rust
// paint.rs boot 测试模块
#[test] fn settings_toggle_marks_dirty()      // 勾选翻转 ⇒ take_global_dirty()==true 且 snapshot 对应 [0]/[1] 变化
#[test] fn watermark_bump_conditions()        // Boot dispatch case1：max_floor=50 抬升并 dirty；51 不动（a.java:4297 `maxFloor<51`）
// scene.rs 测试模块
#[test] fn scene_watermark_bump_marks_dirty() // note_max_floor(50) 抬水位；note_max_floor(51) 不动
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cargo test --release -p game-core global|watermark|settings_toggle`
Expected: FAIL（方法不存在）

- [ ] **Step 3: 实现两机字段与方法**

BootMachine 与 GameScene 各加 `global: GlobalSave`、`global_dirty: bool`；三方法体照 Interfaces；Boot 侧设置勾选翻转处（mode 16 分派）与 dispatch case1（a.java:4297-4299 语义）置 dirty；GameScene 换层 `max_floor` 抬升处调 `note_max_floor`（a.java:7163-7164，条件 `floor>max_floor` 后 `floor<51 && floor>watermark`）。交接（新游戏/读档构造）把 Boot 的 `global` 拷入 GameScene。

- [ ] **Step 4: 跑测试确认通过**

Run: 同 Step 2 命令。Expected: PASS 全绿。

- [ ] **Step 5: 写失败测试（壳持久化与损坏重置）**

```rust
// main.rs shell_tests（临时目录做 save 目录）
#[test] fn skywar_persists_across_restart()  // 改设置 ⇒ 步进触发写 ⇒ 新 Shell 装载 ⇒ 状态恢复
#[test] fn skywar_corrupt_resets_default()   // 写坏 blob ⇒ 装载后 [1]==true 且文件被重写为合法值
```

- [ ] **Step 6: 跑测试确认失败，再实现壳装载/持久化**

`persist_global`/`load_global` 照 Interfaces；触发点 = 每 tick 步进后轮询两机 `take_global_dirty()`（与 sync_bgm 同侧）。SKY_WAR 文件与 MOT_L* 同目录（沿用壳 RMS 路径）。

- [ ] **Step 7: 全量回归 + 提交**

Run: `cargo test --release --workspace`（35 preset + mov + slot + 新测试全绿）→ `python3 gates/cli.py check` 8/8 → 提交（含 Evidence/Confidence；ratchet 只增）。

### Task 2: 型 10/11/12 撞击臂移植（`_ => {}` 静默吞噬清偿）

**Files:**
- Modify: `crates/game-core/src/scene.rs`（step/interact 型分派的吞噬臂；执行期以 Step 1 的失败测试定位真身——四个已知 `_ => {}` 均非此臂）
- Test: `crates/game-core/tests/fixtures_frames*` 中 `fx_firebump_floor13_frames_match`（现 `#[ignore = "BLOCKED…"]`）

**Interfaces:**
- Consumes: 既有 `fx-firebump-floor13` 参照帧夹具（A 任务产出，T301 起分歧钉点）。
- Produces: 三臂实现（型 10/11/12 的撞击/弹射语义）；验收测试解除 ignore 后常驻。

- [ ] **Step 1: 解禁验收测试并确认失败**

把 `fx_firebump_floor13_frames_match` 的 `#[ignore]` 摘除。Run: `cargo test --release -p game-oracle --test fixture_frames fx_firebump`
Expected: FAIL 于 T301（同时宣告吞噬臂真身所在路径）。

- [ ] **Step 2: 读 Java 三臂语义并移植**

按失败帧反查 a.java 的 step/interact 型 switch（10/11/12 三臂；行锚以当前 deobf 行号为准并核对字节码），逐值移植（位移量/镜像/触发条件）。不许"看着像"——数值取自代码。

- [ ] **Step 3: 验收测试通过 + 变异击杀**

Run: 同 Step 1。Expected: PASS（T1-全预算）。变异检查：分别撤一臂/改一常量 ⇒ 至少对应帧红。

- [ ] **Step 4: 全量回归 + 提交**（同 Task 1 Step 7 格式；Evidence 含 T301 钉点与变异结果）

### Task 3: P4 终审与 `stage-a-complete` tag

**Files:**
- Modify: `docs/spec/completeness.md`（m_122 层 12/39 特例登记（B 级）、夹具盲区注记核对、saveGlobalProgress/撞击臂销账）
- Modify: `docs/spec/type-closure.md` / `docs/coverage-matrix.md`（工具再生 + 终态数）
- Modify: `docs/master-plan.md`（里程碑行 + 遗留清单更新）
- Tag: `stage-a-complete`

**Interfaces:**
- Consumes: Task 1/2 的代码与测试；e5391aa 的裁决书与工具链（cover_matrix/dead_code_ruling/method_reach）。
- Produces: Stage A 完成态声明（四判据逐条对照数字）。

- [ ] **Step 1: 账面收尾**：completeness 登记 m_122 层 12/39 特例（B 级，重入两层且有层存档时行为差异）与 Task 1/2 的销账更正；`python3 tools/cover_matrix.py` 再生（预期 绿 153+/黄 1/白 10☠️ + a 0）；`--check` 逐字节一致。

- [ ] **Step 2: oracle 全套件**：主目录跑 `python3 reference/oracle/run.py --scenarios`（刷新 `reference/oracle/_diff/scenarios.md`）+ `--selftest`；全 PASS。

- [ ] **Step 3: 全量门禁**：`python3 gates/cli.py all`（或 check + trace + visual）全绿；ratchet 终态吸收。

- [ ] **Step 4: master-plan 终态 + tag**：四判据对照表（数字写死）；`git tag stage-a-complete && git push origin master --tags`；提交（Evidence/Confidence）。
