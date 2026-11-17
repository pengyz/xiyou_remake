# 对抗复核二轮（复验）：DDDSA P2 八 trait 子系统面实施计划（amend 修订版）

> 复验日期：2026-11-07。复核员：对抗复核员（只读；唯一写=本报告）。
> 被审件：`docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md` @ **b918c2e**（144 行；前版 d034126）。
> 一轮报告：`docs/findings/reviews/2026-11-07-p2-plan-review.md`（3 红 5 黄，Confidence 84）。
> 复验性质：**审修订闭合，非全量重审**——逐项实测红黄闭合 + amend 质量 + 回归攻击。
> 纪律：AGENTS.md（证据 A/B 级、实测命令输出原文）。本报告全部结论均附命令与输出。

## 总判：**可执行（放行开工批0）**（3 红 5 黄全闭合；附 2 条黄-轻新注记，非阻断）

| 复验面 | 结论 |
|---|---|
| 1. R1 ShopGoods 补批 | 🟢 绿（批9 实存、spec:89 双向覆盖、切点语义属实；**锚 ±1 行注记**） |
| 2. R2 6,127/382 量化口径 | 🟢 绿（wc/grep 全复现、分布逐文件吻合、6098 已改写；**166 口径过宽注记**） |
| 3. R3 field_ownership 披露+批0 | 🟢 绿（--check 仍 RED 且与计划披露逐字吻合；187→186 口径属实） |
| 4. 黄五连 | 🟢 绿（5/5 闭合，引锚 p1a1:135-138 / 11-02 plan:42 全实存） |
| 5. amend 质量 | 🟢 绿（diff 仅计划 1 文件无夹带；闸 13/13+金样 74+ratchet 978 独立重跑全真） |
| 6. 回归攻击 | 🟢 绿（无依赖倒挂、无新假数、无新 phantom；次引 phantom 照实记录且真源已找回） |

---

## 面1 R1 闭合（ShopGoods 补批9）— 🟢 绿

### ✅ 切点实存且语义=商品属性（价格/效果/提示）

```
$ grep -n "match (src, idx)" crates/game-core/src/scene.rs
2653:            match (src, idx) {
$ grep -n "fn " crates/game-core/src/scene.rs | awk -F: '$1<2653' | tail -1
2632:     fn apply_merchant_offer(&mut self, buy: bool) -> bool {
```
逐臂语义抽验（scene.rs:2653 起，`apply_merchant_offer` 内）：
- **价格**：`spend_gold(50/1000/800/200/2000/4000)`、`crate::combat::gain_gold(&mut self.player, 1000)`；
- **效果**：`self.player.atk += atk*3/100`、`blue_keys/yellow_keys/red_keys` 增减、`add_item_to_stack(13/19)`；
- **提示**：`show_overlay_message(0, "没有足够的金钱", 0, 0)` 等共 11 处（该窗口 grep -c=11），失败置 `purchase_ok=false`。

→ 计划:96「scene.rs:2652 `match (src, idx)` 商人商品属性束（价格/效果/提示逐臂）」语义完全属实。
⚠ **黄-轻注记（锚 ±1 行）**：match 实在 **2653** 行，计划/commit 写 2652（2652 行=`if flags & 2 != 0 {`）。
且 amend 自贴证据窗「scene.rs:2645-2652」恰好截断在 match 前一行——自证贴件未含被引行本身。
全文件仅此一处 `match (src`，±1 自校正、零歧义，不阻断（与一轮对 125-168/128-168 的"锚点注记（无害）"同级）。

### ✅ spec:89 批归属被计划覆盖（grep 双向）

```
$ grep -n "ShopGoods" docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md
89:- P1 **仅 ItemRegistry**；EnemyRegistry 后置（quirk 路由/域外三界复杂面）；ShopGoods 批 P2 尾；
$ grep -n "ShopGoods" docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md
15:+ShopGoods 注册表化（spec:89"ShopGoods 批 P2 尾"）…（做清单）
96:| 批9 | **ShopGoods（P2 尾件——spec:89"ShopGoods 批 P2 尾"）**…0.5d |
99:合计 ≈**10.5-14d**（…含 ShopGoods 0.5d）
108:…序=0→1→…→9(ShopGoods)→10 收口
114:1. 八 trait 面全立…+ShopGoods 入注册表…
138: R1 ShopGoods 静默缩小→补批9（P2 尾件）…
```
spec 1 命中 ↔ 计划 6 处命中：做清单/批9/合计/序/验收/修订记录全链覆盖，**R1 闭合**。

### ✅ 附带核：次引 phantom 属实 + 真源已找回（回归面详证）

```
$ grep -n "ShopGoods" docs/findings/reviews/2026-11-03-dddsa-attack-adaptation.md; echo exit=$?
exit=1                      # 179 行文件零命中——一轮次引确系 phantom
$ sed -n '89p' docs/findings/reviews/2026-11-03-dddsa-attack-editor.md
ShopGoods 作为 ItemRegistry 的第一个消费者并入 P2 尾或 P3 前（半天级）；EnemyRegistry **后置**
```
「ItemRegistry 首个消费者」措辞的**真源=`2026-11-03-dddsa-attack-editor.md:89`（逐字实存）**——
计划:142-144 修订记录照实录入 phantom（grep exit=1）且主锚 spec:89 成立，无伪引用残留。
码面补充：`grep -rni "itemregistry\|item_registry" crates/` → **exit=1（类型名不在码面）**；
P1 实交物=items.json+`ItemDef`+`parse_items`（game-data/src/defs.rs:540，五类拒载 §2.3 同式）+
ITEM_EFFECTS；现 ItemDef 运行期消费者仅测试（items_defs.rs / p15_zero_code_drill.rs）→
「ShopGoods=首个**生产**消费者」B 级成立。

---

## 面2 R2 闭合（6,127 行/382 站点/6098 改写）— 🟢 绿

### ✅ 6,127 行=实跑 wc -l（12 文件对账）

```
$ wc -l crates/game-core/src/scene_tests/*.rs | tail -1
 6127 总计
$ ls crates/game-core/src/scene_tests/*.rs | wc -l
12
```
另核陈旧值归属链：`sed -n '130,133p' spec` → `### 3.3 …scene_tests 6,098 行直填构造面=风险`
+`【轮2抽验 ✅】scene_tests 6,098 行（wc 实测恰 6098）`（spec:130/133/264 三处）✓；
`git ls-tree cb710f4 … | cat-file | wc -l` → **6127** ✓（计划:59「spec §3.3 引 6098=2026-11-03 抽验值，cb710f4 后=6127」逐字属实）。

### ✅ 382 直写站点=重跑计划内 grep 命令，数吻合且分布逐文件对上

```
$ grep -rEn "\b[a-z_][a-z_0-9]*\.[a-z_][a-z_0-9]* = " crates/game-core/src/scene_tests/ | wc -l
382
$ （逐文件计数，sort -rn）
147 t2_walkto_route_tests / 131 slot_equip_scene_tests / 42 pointer_seam_tests /
 15 entity_route_tests / 14 blank_tap_tests / 8 scene_test_support /
  8 item13_use_regression / 5 touch_tap_guard_tests / 5 fidelity_fix_tests /
  4 itempanel_self_edge_regression / 3 walkto_interrupt_tests        → 合计 382
```
计划:97 分布 11 项**逐项逐数全吻合**，和=382 ✓；命令可执行、非编造（A 级）。

### ✅ 6098 陈旧值已改写 + scene() 构造器

- 计划正文 6098 仅存于 :59（归因 spec 抽验值）与 :138（修订记录史述）——**无冒充实测** ✓。
- `scene()` 构造器：`grep -rE "scene\(" crates/game-core/src/scene_tests/ | wc -l` → **166**（与一轮复核实测同口径、commit 自贴同值——**可复现非编造**）。
⚠ **黄-轻注记（口径过宽）**：166 分解=`scene(` 独立命中 141（含 `pub fn scene(` 定义 1 行 → **真调用点 140**）
+ 其他函数名 25（`arrows_scene(` 13 / `try_run_scene(` 6 / `menu_scene(` 4 / `mk_scene(` 2）。
计划:61 写「scene() 构造器已存=166 调用」标签偏宽；建议改「宽口径 166（真调用点 140）」。
不影响该处定性结论（140 或 166 均支撑「主体为测试逻辑非直填」）。
→ **R2 三要件（改数/量化/命令可执行）全闭合**。

---

## 面3 R3 闭合（field_ownership 披露 + 批0 第一动作）— 🟢 绿

### ✅ 实跑 --check 仍 RED，计划披露逐字吻合

```
$ python3 tools/field_ownership.py --check; echo exit=$?
[RED] 归属表：
  表缺字段 1：['enemy_base']
  表多余字段 2：['enemy_base_hp', 'enemy_base_gold']
exit=1
```
↔ 计划:63-66：「`--check` 现 **[RED]**（表缺 `enemy_base`/表多余 `enemy_base_hp`+`enemy_base_gold`，exit=1——b89d0df P1-6② 切换未同步归属表；**该工具不在 13 gate 内**故闸面不显）」——
逐字吻合 ✓；附核：
- `grep -rn "field_ownership" gates/` → **零命中**（"不在 13 gate 内"属实）；
- `git show --stat b89d0df` → 15 文件含 scene.rs，**不含 field-ownership.json**（根因归属属实）；
- `grep -n "enemy_base_hp\|enemy_base_gold" crates/game-core/src/scene.rs` → **exit=1**（SoA 已切走，表内两条确系多余）。

### ✅ 批0 第一动作=归属表同步（187→186 口径）入计划

```
$ python3 -c "…json.load('data/status/field-ownership.json')['fields']…"  → len=187
  表内实存：enemy_base_hp / enemy_base_gold / battle_enemy_hp（combat 域）
  表内缺：enemy_base（scene.rs:307 pub enemy_base: Vec<EnemyDef>，一轮已锚）
```
- 计划:87 批0「第一动作=field_ownership 归属表同步：enemy_base 入表 combat 域+删
  enemy_base_hp/enemy_base_gold 两条，**187→186** 新口径…`--check` 复绿即证」——
  算术 187−2+1=**186** ✓；`battle_enemy_hp` 为在役字段（scene.rs:346 `pub battle_enemy_hp: i32`，6 处引用）**不在删除清单** ✓ 正确。
- 计划:17-18 直写清零前置声明：`--report` 实跑 → `[报告·定案口径] 生产面 self 直写跨文件字段=0（**终章清零判据**）` ✓（计划引文略"字段"二字，微瑕不计）。
→ **R3 闭合**（披露+处置+判据三件齐）。

---

## 面4 黄五连 — 🟢 绿（5/5 闭合）

| # | 黄项（一轮清单） | 落点 | 实测 |
|---|---|---|---|
| 1 | 贴件 `$(cat)` 终态制 | 计划:79 通则 | 「**贴件纪律**=判据输出以 `$(cat)` 终态原文入档（禁手编辑）」✅ |
| 2 | 回退 ratchet 同步+禁 fix-forward | 计划:82 | 「+基线 ratchet 同步回退+禁 fix-forward」✅ |
| 3 | statediff 命名契约层注记 | 计划:97 批10 | 「dump 名集⊂Java FLD 名集+A 侧需面过滤——全 trace 名集相等**永红**（A 侧 411 名 vs Rust 60 名…`p1a1-dump-face-review.md:135-138`）」✅ |
| 4 | 直写清零前置声明 +"3+8"出处指针 | 计划:17-18 / :53-55 | 见下两证 ✅ |
| 5 | 合计 10.5-14d + 批10 HostCtx 终值断言 | 计划:99 / :115-116 | 见下两证 ✅ |

### 证3：p1a1-dump-face-review.md:135-138 实存且引用准确

```
$ sed -n '135,138p' docs/findings/reviews/2026-11-03-p1a1-dump-face-review.md   （文件共 178 行）
- 🟡 statediff **全 trace 口径注记**（bc4411e 起既有）：A 侧 oracle trace 每 dump 块
  411 名全字段 vs Rust dump 60 名——名集相等守卫在真实全 trace 对上永不可能过
  （合成实测 A-only 名即红）；P1-a1 判据「statediff 名集相等」实义=**命名契约层**
  （dump 名集⊂Java FLD 名集，守卫在场），非全 trace 实跑绿。…
```
计划:97 的三个数字与三个语义（411/60、永红、命名契约层）**逐字对应** ✅。

### 证4：出处指针 + impl GameScene 实测

```
$ sed -n '42p' docs/superpowers/plans/2026-11-02-scene-module-decomposition-plan.md
GameScene（协调器：状态机+域装配）
$ grep -rn "impl GameScene" crates/ | wc -l
13        # scene.rs×4、controls/item_effect/restore_rules/scene_dialog/scene_menu/
          # scene_paint/scene_persist/scene_script/script_ops 各 1（10 文件）
```
计划:53-55「协调器：状态机+域装配——出处指针 `…2026-11-02-scene-module-decomposition-plan.md:42`」
引文逐字实存 ✅；一轮无源断言「3 协调器+8 域文件」已删净（现值 13 个 impl 块，与旧"3+8"无对应——改写正确）。
附注：保留的「域文件编排注记归因=task-15 落件」系一轮已接受的旧文（task-15 实存：
`docs/findings/reviews/2026-11-02-plan-attack-feasibility.md:33`「task-15 步①–⑤ 连落…move-only」）；非本次新引入，不计。

### 证5：合计与终值断言

```
$ python3 -c "lo=[.5,1,1,1,1,1.5,1,1,.5,.5,1.5]; hi=[.5,1.5,1.5,1,1.5,2,1.5,1,1,.5,2]; print(sum(lo),sum(hi))"
10.5 14.0            # 11 批（批0-批10）逐批求和=10.5~14d，spec:155 8-15d 区间内 ✅
$ awk '/^pub trait HostCtx/,/^}/' crates/game-core/src/script.rs | grep -c "fn "
21                   # 计划:116 验收1 所嵌命令可执行；P2 末态断言=21→…→批10 终值 ✅
```

---

## 面5 amend 质量 — 🟢 绿

### ✅ diff 只动计划文档（无夹带）

```
$ git diff --name-only d034126..b918c2e
docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md      # 1 文件，+39/−12
$ git log --oneline -1 d034126
d034126 docs(plans): DDDSA P2 八 trait 子系统面实施计划（草案待复核）
```
单文件、纯计划正文修订，**无代码/基线/门禁夹带** ✅（另核工作区：本复验期间未见计划外改动混入该 diff）。

### ✅ 自测声称独立重跑：闸 13/13 + 金样 74 + ratchet 978 全真

```
$ python3 gates/cli.py status
… [✓ PASS] rust-tests / test-ratchet（total=978 基线978）/ table-content-hash …
[- SKIP] visual-diff — 视觉比对器未就绪（P4 启动后启用）
== 汇总 == PASS=13 FAIL=0 SKIP=1 TOLERATED=0
$ python3 -c "…state-dump-golden.json…"   → scenarios=74
$ python3 -c "…rust-test-counts.json…total…" → total=978
```
与计划:5「ratchet 978/金样 74/闸 13/13 为起点勿动」及 commit 自贴件（R2 6127/382/166、R3 RED exit=1、
spec:89、次引 exit=1、impl GameScene 13、HostCtx 21、Confidence: 92）**逐项复现一致** ✅。

---

## 面6 回归攻击 — 🟢 绿

### ✅ 无新批次依赖倒挂
计划:108 序=**0→1→2→3→4→5→6→7→8→9(ShopGoods)→10 收口**；§3 显式依赖批1←批0、批5←批1、批6←批0。
批9 只需 P1 既有 items.json/ItemDef（P1 收官件），不依赖批10 或任何后位批——**零前向依赖** ✅。
批9 与批2/批6 面不冲突（切的仅是 `apply_merchant_offer` 内 match→表，gain_gold/add_item_to_stack 调用保持）。

### ✅ 无新假数（关键数逐个复跑）
6,127 ✅ / 382+11 文件分布 ✅ / 187→186 ✅ / 411 vs 60 ✅（p1a1:135-138 原文）/
47 场景、45 夹具（ratchet fixture_frames(test)=45）、74、978、21、13 ✅ /
「五类拒载」✅（defs.rs:18-28 五类全列 + parse_items:540/parse_enemies:406「五类拒载 §2.3 同式」）/
「id 升序」✅（items_defs.rs:12-14 `assert!(w[0].id < w[1].id, "id 升序唯一")`；items.json 32 行/enemies.json 35 行 id 实测均升序）。

### ✅ 无新 phantom 指针
本轮计划新引锚逐一实存：p1a1-dump-face-review.md:135-138 ✅（178 行文件内）；
2026-11-02 plan:42 ✅；spec:89 ✅；spec:155 ✅（`| P2 … | **直写清零在跑=前置（C）** |` 逐字）；
cb710f4 ✅；313375b（一轮已核）；attack-adaptation.md:89 为**一轮遗留 phantom**，
计划:142-144 照实录入（grep exit=1 我方复证）且真源 attack-editor.md:89 找回——**不构成新错**。

### 新注记（黄-轻，非阻断，建议批0 提交时顺手改）
1. **锚 ±1**：计划:96 / commit「scene.rs:2652 match」→ 实在 **2653**；commit 自贴证据窗 2645-2652 未含 match 行。
2. **166 口径**：计划:61「scene() 构造器 166 调用」=宽口径 `grep -rE "scene\("` 值（含 25 个 `*_scene(` 其他函数+1 定义行），真调用点 **140**——建议标注口径。

---

## 结论

- 一轮 **3 红全闭**：R1 补批9（spec:89 双向覆盖+切点语义属实）；R2 改 6,127/382 量化口径（命令全复现、分布逐文件吻合、6098 归因改写）；R3 §1 披露 [RED]+批0 第一动作补表（187→186）。
- 一轮 **5 黄全闭**：`$(cat)` 终态制 / ratchet 同步回退+禁 fix-forward / statediff 命名契约层（p1a1:135-138 实存）/ 直写清零声明+「3+8」改写带指针（plan:42 实存、impl GameScene=13）/ 合计 10.5-14d+批10 终值断言。
- amend 干净（单文件）、自测三数（13/13、74、978）独立重跑全真；回归攻击未发现新倒挂/新假数/新 phantom。
- 残余 2 条黄-轻（锚 ±1 行、166 口径过宽）均为自校正级、不改批次范围/判据/依赖/估算。

**总判：可执行——放行开工批0**（批0 第一动作=field_ownership 归属表同步，`--check` 复绿即证；两条黄-轻随批0 提交顺手修正）。

## Confidence: 90/100
