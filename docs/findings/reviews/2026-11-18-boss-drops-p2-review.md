# a80f21c 对抗复核 — boss 击杀掉落数据化（P2）

- 复核对象：commit `a80f21c`（分支 `modern`，未 push）
  `feat(core+solver): boss 击杀掉落数据化——rules::boss 单一来源，求解器建「击杀 boss→掉落实体」边`
- 复核者立场：独立对抗复核（不采信实现者自述，逐条自验）
- 复核纪律：只读仓库（仅写本报告，未 commit、未 push）；变异实验全部临时且已 `git checkout` 还原，
  结束态 `git status --porcelain` 输出为空
- 环境：`/home/peng/workspace/xiyou_remake`（`cargo test --release` 口径）

## 1. 判定

**红 0 / 黄 4 / 绿 6**（问题条目：黄 4 条 Y1–Y4、红 0；面级：F1/F2/F3/F4/F7/F8 绿，F5/F6 黄）

4 条黄项不阻断入库，逐条见 §3。

**Confidence: 88**

**结论：能进 P3。** 掉落表与 `a.java` 字节码逐条一致（含组号与顺序，50 条），页面上「单一来源」成立，
求解器的「击杀 boss → 掉落实体」边是**真的建起来**的（不是 DES 死边），74 金样与 4 条 boss 帧金样全绿，
门禁 `PASS=13 FAIL=0 SKIP=0` 为实跑结果。四条黄项都不改变正确性结论，其中 Y1（闭包臂无判据）与
Y3（无「拾取入账」端到端判据）建议在后续片补测，不作为 P3 门禁。

## 2. 逐面验证

### F1 数据保真（绿）

**做了什么**：自己写脚本机械抽取，不用实现者的任何脚本/测试断言。

```bash
python3 /tmp/verify_boss_2026_11_18.py
```

脚本逻辑（`/tmp/verify_boss_2026_11_18.py`，全文 30 行）：
1. 定位 `reference/src/deobf/a.java` 里 `this.bossEventSpawns = new byte[][]{`（a.java:2041）与
   `this.bossTypeOrder`（a.java:2198），取中间行；
2. 逐行 `re.findall(r'-?\d+', line)` 取整数，遇 `}` 收一组，`while groups and not groups[-1]: groups.pop()`
   去掉外层 `};` 造成的空组 → 四元组列表；
3. 解析 `crates/game-core/src/rules/boss.rs` 的 `static EVENT_SPAWNS` … `\n];`，按 `// [n]` 注释分段，
   每段用 `s\((-?\d+),\s*(-?\d+),\s*(-?\d+),\s*(-?\d+)\)` 抽四元组；
4. 两组整体 `==` 比较（含组数、每组条数、组内顺序）。

**结果**（原文输出）：

```
java 组数/条数: 8 [12, 12, 4, 1, 1, 5, 12, 3]
rust 组数/条数: 8 [12, 12, 4, 1, 1, 5, 12, 3]
逐条(含顺序)一致: True
总条数: 50
```

- 组行区间：[0] 2042-2091、[1] 2092-2141、[2] 2142、[3] 2143、[4] 2144、[5] 2145、[6] 2146-2195、[7] 2196
- `bossTypeOrder`（a.java:2198）= `{75, 74, 68, 67, 69, 71}` = `crates/game-core/src/rules/boss.rs` 的 `boss_order()`
- event71（组 5）= `{24,6,2,0, 32,5,1,0, 32,6,1,0, 32,7,1,0, 27,6,6,0}` —— **恰 5 条，无 26 钥匙**
- ty 值域 `{16,17,24,26,27,29,30,32}`，与 `boss.rs` 完全一致

**副产物（文档注记，非本提交引入）**：`docs/spec/evidence-boss.md` §R-boss-3 的 8 行表格**表值与字节码一致**
（idx5 也是 5 条），但文档代码块标的行号有漂移：

| 文档标注 | 实际 | 差 |
|---|---|---|
| `// a.java:8906-8915`（spawnBossEvent） | `private void spawnBossEvent(int var1)` 在 a.java:8912，体 8912-8921 | 6 行 |
| `if (++this.battleAnimTick > 5)` 标 `10102-10113` | 实际 a.java:10112 | 10 行 |

本提交新模块（`boss.rs`）引用的是**正确**行号（8912-8921），所以这是文档预存漂移，不是本提交的错误。
建议顺手在下次碰 `evidence-boss.md` 时更正，并在原文标注日期。

**判定**：绿。

### F2 单一来源（绿，附 nit Y4）

**做了什么**：

```bash
grep -rn "BOSS_TYPE_ORDER\|BOSS_EVENT_SPAWNS\|bossTypeOrder\|bossEventSpawns" --include=*.rs crates/
grep -rn "event_spawns\|boss_order\|rules::boss" crates/game-solver/
grep -rn "17, 6, 4\|16, 5, 5\|24, 6, 2" --include=*.rs crates/ | grep -v rules/boss.rs
```

**结果**：
- 第一组命中**全部是注释/文档**：`crates/game-core/src/scene.rs:156`、`:3668`（「旧 BOSS_TYPE_ORDER 已迁移」类注释），
  `crates/game-core/src/rules/boss.rs:3/9/11/16/42/109/118/127/130/204`（模块自述注释），以及 oracle 测试的说明文字。
  **没有任何一份活着的第二份表**。
- `spawn_boss_event` 只有一处定义 `crates/game-core/src/scene.rs:2246`，两个调用点 `:3680`（击杀钩子）、`:3761`（胜利收尾）。
- 求解器侧 `rules::boss` 只出现在 `crates/game-solver/src/solve_local.rs:801/802/805`（`apply_kill_drops` 内消费）与
  `:782` 注释、测试文案；**第三组 grep 为空** ⇒ 求解器里没有硬编码的掉落四元组。
- 旧表删除可从 diff 直接看到：`git show a80f21c -- crates/game-core/src/scene.rs | grep '^-'` 里整段
  `const BOSS_TYPE_ORDER` / `const BOSS_EVENT_SPAWNS` 被删。

**nit（Y4）**：`crates/game-core/src/scene.rs:3759` 仍是 `post_victory_event_of(69, self.persist.floor)`。
「组号字面量」确实已从引擎消失，但「型 69 → 收尾组 7」这条映射仍写在引擎的 `match` 臂里（函数签名被迫收 `ety`），
即单一来源只到「组号→表」，未到「型→组」。不影响行为，属设计层小遗憾。

**判定**：绿（单一来源成立）。

### F3 引擎行为不变（绿）

**做了什么 3.1（顺序变更的独立审计）**——实现者承认「73@40 的 spawn 从 `load_level_script(22)` 之后移到之前」。

Java 原序（a.java:10076-10082）：

```java
case 73:
  this.scriptLineFlags[this.currentScriptIndex] = true;
  this.loadLevelScript(22);
  if (this.currentFloor == 40) this.spawnBossEvent(6);
```

新 Rust 序：先 `event_of(73, 40)` 得 `Some(6)` → `spawn_boss_event(6)`，再进 `match t` 置 flag + `load_level_script(22)`
（`git show a80f21c -- crates/game-core/src/scene.rs` 的 `@@ -3689,19 +3671,20 @@` hunk：旧 `if self.persist.floor == 40 { self.spawn_boss_event(6); }` 被删）。
⇒ **顺序确实与 Java/旧 Rust 相反**，自述属实。

**无害性（静态、可复现）**：两侧触及的状态集合不相交。

| 侧 | 函数 | 触及 |
|---|---|---|
| spawn | `crates/game-core/src/scene.rs:2246-2263` `spawn_boss_event` | 仅 `self.entities` / `self.grid` / `self.width_table` |
| spawn | `crates/game-core/src/entity.rs:322-370` `entity::spawn` | 自由函数，入参 `(table, grid, type_id, px, py, param, width_table, img_h)`；`let idx = table.count;` 追加；不读 `script_index`/`dialog`/`game_mode` |
| load | `crates/game-core/src/scene_persist.rs:500` `load_level_script` | `interrupt_route`、`undo.clear(ScriptEvent)`、`clear_tap_state`、`set_walkto_draining(false)`、`set_script_walk_armed(false)`、`transition_to(Dialog)`、dialog page/phase、`lock_camera_on(0)`、`current_script_index`、`script_line` clone、`set_script_cursor(0)` |
| load | `crates/game-core/src/scene.rs:897-908` `transition_to` | 仅 `game_mode` 与模式计数器 |

`load_level_script` **不清 entities、不改 grid、不碰 open_group / des 计数**；`interrupt_route`
（`crates/game-core/src/scene.rs:1516`）只动 route / walk_path_buffer / walk_step_count / undo。
故实体表内容与槽号在两序下逐字节相同。

**做了什么 3.2（金样确实走到三条路径）**——逐条读断言：

| 金样 | 覆盖路径 | 关键断言 |
|---|---|---|
| `crates/game-oracle/tests/fixture_frames.rs:831` `fx_boss67a_kill_spawn_frames_match` | 67 击杀 → 组 3（1 件 17） | 掉落 `17@(6,4)` 存在、终态 `state==0`、`anim_row==0`、`gold==200`、boss `removed` |
| `:860` `fx_boss74a_kill_hooks_frames_match` | 74 击杀 → 组 1（12 件） | 12 件掉落齐、`script_line_flags` 置位（case74 走 `load_level_script(21)`） |
| `:992` `fx_boss68a_kill_spawn_frames_match` | 68 击杀 → 组 2（4 红钥匙） | `(6,5)(7,6)` 留存、`(5,6)(6,7)` 已拾、`red_keys==2`、`gold==1000`、型 68 `removed` |
| `crates/game-oracle/tests/preset_cases.rs:1644` `preset_itemboss40_frames_match` | **73@40 → 组 6（12 只 DES）+ `load_level_script(22)`** | T1-700 全帧对 Java trace |
| `:4155` `preset_floor35_frames_match` | **69@35 → 组 4 击杀拍 + 组 7 收尾拍** | T1-1200 全帧对 Java trace |

实跑：`cargo test -p game-oracle --release fx_boss` → `4 passed; 0 failed`；
`preset_itemboss40_frames_match` / `preset_floor35_frames_match` 各 `1 passed`。
⇒ 本次改的三条路径（击杀钩子 / 73@40 顺序 / 69@35 收尾）**都有逐帧级判据覆盖**，且全绿。

**做了什么 3.3（收尾严格等价）**：`post_victory_event_of` 函数体为
`if ety == 69 && floor == 35 { Some(7) } else { None }`，且 diff 显示 `self.load_level_script(36);` 是**上下文行**（仍在 `if` 内）
⇒ floor≠35 时既不 spawn 也不 load，与旧 `match vt { 69 => if self.persist.floor == 35 { … } }` 严格等价。
Java 侧 a.java:10112-10149 的 post-anim switch 里只有 `case 69` 调 `spawnBossEvent(7)`（`case 61` 是另一条 71@49 机制），Rust 覆盖一致。

**判定**：绿。

### F4 求解器边正确性（绿；「DES 死边」有确定答案 = **不是死边**）

**4.1 调用点与双计**：
- `crates/game-solver/src/solve_local.rs:1579-1581` 闭包 `ClosureCand::Kill` 臂：`ns.ents.remove(slot)` **之后** `apply_kill_drops(&mut ns, ety, self.input.floor)`；
- `:1683` `Cell::Monster` 臂：同样在 `remove(slot)` 之后调用；
- `expand_inner`（`crates/game-solver/src/solve_local.rs:1611`）开头 `if let Some(s) = self.closure_step(st) { out.push(s); return; }`
  ⇒ 同一父态要么走闭包步、要么走四臂，**互斥**；两臂又都先 `remove` 再 push，所以同一只怪不可能被两路各发一次掉落。

**4.2 掉落 EntRec 与引擎 `spawn_entity` 一致**：
`apply_kill_drops`（`crates/game-solver/src/solve_local.rs:799-812`）push
`EntRec { ty: s.ty, x, y, param: s.extra }`；`EntRec`（`:752`）只有 `ty/x/y/param` 四字段，与引擎
`entity::spawn` 写 `entity_type/px/py/entityParam` 的语义一一对应（px = x<<5）。槽号：引擎恒取 `table.count` 追加
（`crates/game-core/src/entity.rs:330`），求解器先 `remove(boss)` 再 push ⇒ 掉落槽号必大于 boss 槽号，与引擎一致。

**4.3 DES 态（本片最可能出错的点）——确定答案：不是死边。**
- Java 的可交互门是 `interactWithCell`（a.java:5363）里的实体扫描条件
  **`if (this.entityState[var4] != 1)`（a.java:5377）** —— 即 state==2（DES 登场动画）**仍然可交互**；
  `markEntityDesRemoved`（a.java:6332-6347）只把 state 置 2 并换 27×29 的 animRow8，是登场动画而非「不可拾取」。
- 求解器 `readback_ents`（`crates/game-solver/src/solve_local.rs:772-780`）的过滤谓词是
  `*i < sc.entities.count && !e.removed && e.state != 1` —— 与上面那条 `!= 1` 逐字同构，state==2 被当作活体保留。
- `EntRec` 根本没有 state 字段，`classify`（`:1236`）按 `entity_category_with_floor` 判定：掉落用到的
  16/17/24/26/27/29/30/32 全部命中 `13..=32 => 2`（`crates/game-core/src/entity.rs:251-264`）⇒ `Cell::Pickup`。
- `pickup_step`（`:2151`）不自写效果表，直接调引擎 `try_step` + 8 tick + readback。

⇒ 掉落在模型里是「击杀后立刻可踩的 cat2 实体」，与引擎 `entityState != 1` 语义一致。**边真的建起来了。**

**4.4 去重/剪枝不会把边丢掉**：`Key`（`crates/game-solver/src/solve_local.rs:836-845`）含 `ents: Vec<EntRec>`，
`key_of`（`:846-863`）逐态克隆 ⇒ 只因掉落不同的两个态不会被合并；`weakly_ge` 只在同 K₀ 内判支配。

**4.5 跨层不泄漏**：`solve_global` 按层存 `ents.insert(st.floor, g.post_ents.clone())`
（`crates/game-solver/src/solve_global.rs:523-524`），目标层不在 map 时用 `sprite_full(tlevel)`（`:561-563`）；
`monsters_alive_of`（`:690-713`）只数 cat==8 ⇒ 掉落（cat2）不影响怪物存活计数。

**4.6 只生成实体、不凭空入账**：`apply_kill_drops` 只 push `EntRec`，不动 `ns.player` 的钥匙/背包/gold
（M4 变异实验反证：加了 `ns.player.yellow += 1` 立即被 `crates/game-solver/tests/solve_local.rs:952` 抓到）。

**4.7 同格顺序**：引擎 `entity::spawn` 恒 append、无单元格占用检查；求解器 `ns.ents` 也是 Vec append，
`build_scene`（`:2325`）按 `(ty, 格)` 首配回灌 param。两序一致（先 remove boss 再 push 掉落）。

**判定**：绿。

### F5 判据是否真的判（黄）

**做了什么**：读全部 10 条新增判据 + 6 次变异自证（M1–M6，见 §4）。

**结果**：
- 判据不是空断言。`non_boss_kill_spawns_no_drops`（`crates/game-solver/tests/solve_local.rs:964-987`）用的是
  `assert_eq!(g.post_ents, vec![EntRec{ty:7,x:2,y:1,param:2}])` 精确等值，M5（把 `events` 改成一桌全发）能把它打红。
- 8 条 `rules::boss` 单测实跑 `running 8 tests … 8 passed`；`solve_local` `running 15 tests … 15 passed`（基线 13→15 ✓）。
- `fixture_3x3` 参数化**没有**动既有语义：`git show a80f21c^:crates/game-solver/tests/solve_local.rs` 95-140 行的原文
  与新 `fn fixture_3x3(p) -> SolveInput { fixture_3x3_ty(41, p) }` 逐字段等同（3×3、terrain 中排三格 true、start (0,1)、
  `ty41@(1,1) param0` + `ty7@(2,1) param2`、exit stair (2,1)）。测试文件 `git show | grep '^-'` 仅 3 行删除：
  一行 `use` 列表、一行 `SolveInput,`、一行 `ty: 41,` ⇒ 无既有断言被放宽。

**黄点（Y1、Y2、Y3）**：
- **Y1：闭包 Killing 臂的掉落调用点无任何判据。** M6 变异（删掉 `:1579-1581` 的那次 `apply_kill_drops`）后
  `cargo test -p game-solver --release --test solve_local` 仍 `15 passed; 0 failed`。即「两个击杀臂都建了边」这句
  只有 `Cell::Monster` 臂被验证。
- **Y2：`boss_kill_spawns_drop_entity_from_core_rules_only`（`:933-962`）用 `post_ents.contains(&EntRec{17,6,4,0})`**，
  对「多发」不敏感：M5 让 `events` 恒返回 `Some(0..7)` 时它仍然绿，只有 `non_boss_kill_spawns_no_drops` 变红。
- **Y3：没有任何判据证明掉落实体「可被踩到并计入钥匙/背包」**。夹具落点 `17@(6,4)` 在 3×3 夹具外，
  测试只证明「实体被 push 进 post_ents」。可拾取性目前是**代码路径级**论证（§4.3）+ 引擎金样 `fx_boss68a` 的拾取断言，
  而不是求解器侧的端到端判据。

**判定**：黄（判据真实可变异复现，但 Y1/Y3 是真实覆盖缺口）。

### F6 文档诚实性（黄）

逐条核对自述主张：

| 自述 | 核实 | 结论 |
|---|---|---|
| 8 组共 50 条与 a.java 逐条一致 | §F1 脚本输出 `逐条(含顺序)一致: True`、`总条数: 50` | ✓ |
| event71 是 5 条（不是 6 条） | 组 5 = 5 条，无 26 钥匙 | ✓ |
| `markEntityDesRemoved` 标的是刚 spawn 的掉落 | a.java:8912-8921 `var3 = spawnEntity(...)` 后紧跟 `markEntityDesRemoved(var3)`，`var3` 每轮循环重绑定 | ✓ |
| 求解器不复制任何表值 | §F2 第三组 grep 为空 | ✓ |
| 74 金样 sha 全同 | 门禁实跑 `state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同` | ✓ |
| 门禁 PASS=13 | 实跑 `PASS=13 FAIL=0 SKIP=0` | ✓ |
| 新增 10 条判据 | 8（core）+ 2（solver） | ✓ |
| 「引擎行为**逐字节**不变」 | 实测口径是「74 场景 state-dump sha 全同 + 4 条帧金样全帧一致」；且 73@40 的 spawn/load 顺序确实变了 | ⚠ 措辞过强（可观测面等价，非字节等价） |
| 「74/75 的剧本臂…返回 None 的情况在注释说明」 | `event_of(74,·) = Some(1)`、`event_of(75,·) = Some(0)`（两者都有掉落组），返回 `None` 的是「额外组」不是事件 | ⚠ 简称易误读；模块内 doc（`boss.rs:127-137`）是对的 |

未发现「伪锚」或「实测但无命令」式断言：commit 给出的锚点 `a.java:8912-8921`、`:10059-10093`、`:10110-10149`
我逐条用 `grep -n` 定位，除 `10059` 是 `var8 = this.entityType[...]`（10060）上方的空行外，均在合理范围内。

**判定**：黄（主张全部成立，两处措辞偏强/偏含糊）。

### F7 门禁与基线（绿，已实跑）

```bash
python3 gates/cli.py check
```

输出（13 个 gate 全绿，末行 `PASS=13 FAIL=0 SKIP=0`）：

```
[✓ PASS] original-integrity      — 1 个归档校验通过
[✓ PASS] assets-manifest         — {'entries': 138, 'classes': 2, 'maplv': 55, 'sprite': 55}
[✓ PASS] reference-seed-integrity— 8457 行 sha256 通过
[✓ PASS] knowledge-format        — 64 条目
[✓ PASS] docs-links
[✓ PASS] naming-ledger
[✓ PASS] rust-tests              — cargo test --release 全绿
[✓ PASS] test-ratchet            — total=1143(基线1143) 无回退
[✓ PASS] table-content-hash
[✓ PASS] table-evidence
[✓ PASS] state-machine-anchors   — 锚 6 + 边数对照
[✓ PASS] asset-truth-manifest
[✓ PASS] state-baseline          — [GREEN] 金样逐字节等价：74 场景 sha 全同
PASS=13 FAIL=0 SKIP=0
```

- ratchet：`git show a80f21c -- data/status/baselines/rust-test-counts.json` 的改动只有
  `game_core(lib) 401→409`、`solve_local(test) 13→15`、`total 1133→1143`，其余键未动。
  `gate_test_ratchet`（`gates/cli.py:307`）逐键只许增不许减 ⇒ **只升不降成立**，且无「删测试凑绿」。
- 分项与实际一致：`cargo test -p game-core --release --lib rules::boss` 输出 `running 8 tests`，同 binary 段
  `401 filtered out` ⇒ 该 binary 总 409；`cargo test -p game-solver --release --test solve_local` 输出
  `running 15 tests`、`0 filtered out` ⇒ 15。与基线两个键逐一对上。
- 未改阈值/上限/白名单：diff 中不含 `gates/`、不含 `data/status/baselines/state-dump-golden.json`、不含任何 `--ignore`/`SKIP` 配置。

**判定**：绿。

### F8 夹带与红线（绿）

```bash
git show --stat a80f21c
git diff --name-only a80f21c^ a80f21c
```

6 个文件、413 增 / 43 删：

| 文件 | 规模 | 是否该动 |
|---|---|---|
| `crates/game-core/src/rules/boss.rs` | +279（新） | ✓ 新模块 |
| `crates/game-core/src/rules/mod.rs` | +1（`pub mod boss;`） | ✓ |
| `crates/game-core/src/scene.rs` | 59 行（删旧表 + 改 3 处调用） | ✓ |
| `crates/game-solver/src/solve_local.rs` | +38 | ✓ |
| `crates/game-solver/tests/solve_local.rs` | 73 行 | ✓ 判据 |
| `data/status/baselines/rust-test-counts.json` | 6 行 | ✓ 基线随测试数 |

- `original/`、`reference/` **零改动**（`git show --name-only` 不含两目录；`gates check` 的 original-integrity 与
  reference-seed-integrity 也全绿）。
- 没有顺手改别的模块（无 `game-data`/`game-oracle`/`gates/`/`docs/` 变更）。

**判定**：绿。

## 3. 黄项清单（4 条，均不阻断 P3）

### Y1 [F5] 闭包击杀臂的掉落调用点无判据
- **现象**：`crates/game-solver/src/solve_local.rs:1579-1581` 的 `apply_kill_drops` 调用被删掉后，
  `cargo test -p game-solver --release --test solve_local` 仍 `15 passed; 0 failed`。
- **证据**：变异 M6（§4），`test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`。
- **影响**：commit 声称「两个击杀臂都建了边」，实际只有 `Cell::Monster` 臂被验证。若将来闭包臂的调用被误删/改错，
  回归检测不到，会出现「参数化闭包打 boss 不掉落」的静默错误。
- **修法**：在 `crates/game-solver/tests/solve_local.rs` 加一条闭包臂夹具：closure 覆盖 boss 格（cat8 或 boss 型）
  → 闭合推进击杀 → 断言 `post_ents` 含对应掉落 `EntRec`。

### Y2 [F5] boss 测用 `contains` 弱断言
- **现象**：`crates/game-solver/tests/solve_local.rs:933-962` 只断言「集合包含 `17@(6,4)`」，不看是否有额外掉落。
- **证据**：变异 M5（让 `events` 恒返回 `Some(0)..=Some(7)`）后该测仍绿，只有 `non_boss_kill_spawns_no_drops` 红。
- **影响**：只能靠守卫测兜住「过触发」，单测自身对多发不敏感。
- **修法**：boss 测改用 `assert_eq!` 精确比对 `post_ents`（去掉 boss 后应恰为组 3 的 1 条）。

### Y3 [F4] 缺「掉落可被拾取并入账」的端到端判据
- **现象**：`17@(6,4)` 在 3×3 夹具外，测试只证明实体被 push；可拾取性靠代码路径论证（a.java:5377 `entityState != 1`）
  与引擎金样 `fx_boss68a`（红钥匙拾取）间接担保。
- **证据**：`crates/game-solver/tests/solve_local.rs:933-962` 无拾取步；`apply_kill_drops` 无 player 写入。
- **影响**：若将来 `classify`/`pickup_step` 对 state 的口径变化，可能出现「边建了但走不到」的静默退化。
- **修法**：加一条夹具把 boss 与掉落放在可达格，击杀后踩格拾取，断言 `player.red_keys/blue_keys/…` 或背包变化。

### Y4 [F2] post-anim 的「型→组」仍是引擎字面量
- **现象**：`crates/game-core/src/scene.rs:3759` `post_victory_event_of(69, self.persist.floor)`——型 69 写在引擎里。
- **证据**：`grep -n "post_victory_event_of" crates/game-core/src/scene.rs` ⇒ `:3664` 定义、`:3759` 调用。
- **影响**：单一来源只到「组号→表」，不到「型→组」；新增收尾型（若将来有）要改引擎。
- **修法**：把收尾型也纳入 `rules::boss`（例如 `post_victory_events: [(ety, floor, group)]` 表），引擎只查表。

## 4. 变异自证记录（M1–M6）

> 纪律：每轮变异 → 跑指定测试 → `git checkout -- <file>` 还原 → `git status --porcelain` 为空。
> 六轮结束后已确认工作树干净（`git status --porcelain` 无输出）。

| # | 变异点 | 期望 | 实测红态 |
|---|---|---|---|
| M1 | `crates/game-core/src/rules/boss.rs:79` `s(17, 6, 4, 0)` → `s(17, 6, 5, 0)` | core 表测 + solver 穿透测红 | `event_67_is_exactly_one_gold_spoon_at_6_4` FAILED（`boss.rs:170` `left: [BossSpawn { ty: 17, x: 6, y: 5, extra: 0 }] right: [... x: 6, y: 4 ...]`）；`full_table_matches_java_source` FAILED（`boss.rs:271` `event 3 与 a.java 字节组不一致 left: [(17, 6, 5, 0)] right: [(17, 6, 4, 0)]`）；`crates/game-solver/tests/solve_local.rs:946` `boss_kill_spawns_drop_entity_from_core_rules_only` FAILED。其余 6 条 core 测绿 |
| M2 | 删 `boss.rs` `event_of` 首行 `if floor > 50 { return None; }` | 索引规则测红 | `event_of_matches_boss_order_indices` FAILED（`boss.rs:210` `left: Some(0) right: None`），其余 7 绿 |
| M3 | `apply_kill_drops` 首行插 `if true { return; }` | boss 测红、non_boss 绿（证明二者独立） | `boss_kill_spawns_drop_entity_from_core_rules_only` FAILED；`non_boss_kill_spawns_no_drops` passed |
| M4 | `apply_kill_drops` 在 push 前加 `ns.player.yellow += 1;` | 抓「凭空发钥匙」 | `crates/game-solver/tests/solve_local.rs:952` FAILED：`掉落不得凭空发钥匙（踩格才 pickup）：PlayerSpec { ... yellow: 1, ... }` |
| M5 | `event_of` 恒返回 `Some(0)..Some(7)`（一桌全发） | 守卫测必须红 | `non_boss_kill_spawns_no_drops` FAILED（`solve_local.rs:978`）；boss 测**仍绿** ⇒ 过触发只由守卫测兜住（Y2 依据） |
| M6 | 删掉闭包臂 `crates/game-solver/src/solve_local.rs:1579-1581` 的 `apply_kill_drops` | 应有测红 | **全绿**：`test result: ok. 15 passed; 0 failed; … 0 filtered out` ⇒ 闭包臂无判据（Y1 依据） |

结论：数据表、索引规则、掉落生成、「不发钥匙」四条都有真实判据；**闭包臂调用点与「拾取入账」两条没有**。

## 5. 实现者自述中说错/说弱的地方

1. **「引擎行为逐字节不变」**——口径不准。实际保证是「74 场景 state-dump 逐场景 sha 全同 + 4 条 boss 帧金样逐帧一致」，
   而 73@40 的 `spawn_boss_event(6)` 与 `load_level_script(22)` 顺序**确实互换了**（自述也承认）。可观测面等价 ≠ 字节等价。
2. **「74/75 的剧本臂…返回 None 的情况在注释说明」**——易读成 `event_of(74,·)`/`event_of(75,·)` 返回 `None`。
   实际是 `Some(1)`/`Some(0)`（两者都有掉落组），返回 `None` 的是「额外组（收尾）」这一层。模块内 doc 是对的，commit 正文含糊。
3. **「两个击杀臂」的覆盖暗示**——commit 与注释让人以为两臂都受判据保护，实际只有 `Cell::Monster` 臂（Y1，M6 实证）。
4. **「74 金样 sha 全同」**——用词与门禁原文一致，但它是 `state-dump`（状态快照）不是像素帧；本次另外 4 条 `fx_boss*` 才是帧级，
   两者不要混说。
5. 轻微：commit 正文引 `a.java:10059-10093`，杀怪块实际从 `10060 var8 = this.entityType[this.battleTargetEntity];` 起
   （10059 是空行）；不影响结论。

未发现虚构证据、伪造锚点、或「已实测」但给不出命令的断言。

## 6. 未验证 / 无法验证

1. **组 0（75@50）与组 5（71@49）没有金样级引擎侧对拍**：只有 `rules::boss` 单测。引擎侧走的是同一段
   `spawn_boss_event` 代码，故正确性为 B 级（表一致 + 同一代码路径）推断，非端到端实证。
2. **掉落落点是否在真实 `maplv` 上可走**：未逐层解析地图校验；因此「掉落能被拾取」只在代码路径层面成立（Y3）。
3. **闭包击杀 boss 的实际语义场景**：M6 只证明无判据，未构造出「闭包臂真打到 boss」的夹具，故闭包臂是否在真实搜索里被触发、
   触发的掉落槽序是否与引擎一致，未实证。
4. **未跑 `gates/cli.py all`**（含 `trace`/`visual`/`reference` 全量）：与自述口径一致，只跑 `check`；
   若要 P3 合入前的完整门禁，需另行跑全量。
5. **Java 侧 `case 61`（71@floor49）** 的 spawn 细节只做了静态确认（未见对应 Rust 分支的等价性检查），
   它与本次改动的 73/69 无交集，故未展开。

---

复核者：独立对抗复核（subagent）
日期：2026-11-18
备注：本报告未 commit，等待人工裁定入库。
