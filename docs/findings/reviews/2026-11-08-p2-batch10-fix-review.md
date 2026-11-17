# P2 批10 修复件对抗复核（2026-11-09）

- **宣称修复件**：commit amend 后 `f07992c`（替 `37df838`；26 文件 +232/-253）+ `3e732e1` 重基；宣称面10 全修
- **实测仓库态**：`/home/peng/workspace/xiyou_remake` @ branch `modern`，HEAD=`3e732e1` → 父=`37df838`（=首审报审件本体）
- **口径**：6 个复核面逐面红/黄/绿；全部断言带实测输出（sed/grep/git/cargo 原文）
- **总判：需修复（修复件缺失）**——`f07992c` 在本仓库**不存在**（ref/对象库/全 commit 扫描 0 命中），首审面9 红三项、面5 黄、面7 黄**一项未落**

| 面 | 判 | 一句话 |
|---|---|---|
| 1 面9 红三项全落核 | **红 ×3** | 界注仍旧注 / 游离 #[test] 仍旧双挂 / message census 仍 0 与 8 并存 |
| 2 census 实测 | **红** | `cargo test --workspace --no-run` → 真实 warning **7 条**（grep 计 8 行含汇总），面5 黄未销 |
| 3 批8黄②两 body 实界 | 黄 | 实界实测=**1386-1411 / 1633-1643** 与宣称值一致，但 scene.rs:1407 / input_mapper.rs:38 / message 三处**仍旧值** |
| 4 回归 | 绿 | p2b10 2/2、gates PASS=13 FAIL=0（state-baseline [GREEN] 74 场景）、game-core 全测 526 passed/0 failed、金样零漂移 |
| 5 夹带 | 黄 | 实际报审件 `37df838`=**30 文件 +246/-150**（宣称 26 文件 +232/-253 无对应对象）；scene.rs 生产码零行为 ✅；3e732e1 恰 1 文件 ✅ |
| 6 claims-must-match | **红** | 26 文件/+232/-253 无对象可核；392→320 ✅、SceneSetup 40 字段 ✅、HostCtx 8 名 ✅（后三项实测为真） |

---

## 面0 修复件存在性 — 红（前置，决定全表）

```
$ git rev-parse --verify f07992c
fatal: 有歧义的参数 'f07992c'：未知的版本或路径不存在于工作区中。
$ git cat-file -t f07992c
fatal: Not a valid object name f07992c
$ git log -1 --format='%B' f07992c
fatal: 有歧义的参数 'f07992c'…
```

全对象扫描（含不可达 commit，控制组验证方法有效）：

```
$ for c in $(git cat-file --batch-all-objects --batch-check | awk '$2=="commit"{print $1}'); do
    git grep -q "camera_at_target(2777)" $c -- 2>/dev/null && echo "FOUND in $c"; done
(scan done)                       # 0 命中
$ git grep -c "SceneSetup" HEAD -- crates/game-core/src/scene_tests/scene_test_support.rs
HEAD:crates/game-core/src/scene_tests/scene_test_support.rs:3   # 控制组：同方法能命中 → 扫描有效
$ git log --all --oneline -S'camera_at_target(2777)'
（空）
```

reflog 链（批10 后**无**第四次 amend，即修复 amend 从未发生）：

```
3e732e1 HEAD@{2026-11-08 04:23:46 +0800}: commit: gate(check): state-baseline 注释勘误…
37df838 HEAD@{2026-11-08 04:23:15 +0800}: commit (amend): refactor(core): P2 批10…
ca2ae02 HEAD@{2026-11-08 04:05:31 +0800}: commit (amend): refactor(core): P2 批10…
916ef0d HEAD@{2026-11-08 03:51:32 +0800}: commit: refactor(core): P2 批10…
```

- 首审报告 `docs/findings/reviews/2026-11-08-p2-batch10-review.md` mtime=**Nov 8 05:43**（> 37df838 的 04:23），报审件确为 `37df838`（30 文件 +246/-150），与首审一致。
- 主仓库与全部 10 个临时 worktree 均**无** `camera_at_target(2777)` 文本；主仓库 `git status --porcelain`（排除 ??）= **0 行**（无未提交修复）。
- 结论：**宣称的修复动作在本仓库未发生**；下述各面均是对当前真实态（=首审报审件）的复测。

---

## 面1 面9 红三项全落核 — 红 ×3（三项全未落）

### ① shop_purchase_tests.rs 界注 — ❌ 仍旧注（与实况相反）

```
$ sed -n '60,70p' crates/game-core/src/scene_tests/shop_purchase_tests.rs
    /// 【grep 双源=0（plan:97 判据）】apply_merchant_offer 体无内联价/提示字面量。
    #[test]
    fn p2b9_shop_goods_single_source_no_inline_values() {
        let scene_src = include_str!("../scene.rs");
        let start = scene_src.find("fn apply_merchant_offer").expect("fn 在档");
        // 界定=apply_merchant_offer 函数体（到下一 fn replace_shell_rows 前止——
        // 超出体的后续 helper 不入判定面；体内注释亦须零字面=过扫更严）。
        // 【2026-11-08 更正】原注称 OFFER_TEXTS 报价单文案含「没有足够的金钱」子串
        // ——实测 0 处（复核建议②），失实注释删除：本界真正的意义=单源判定聚焦
```

- 宣称新文「到 fn camera_at_target(2777) 前止——含 replace_shell_rows(2758-2776)」→ `grep -rn "camera_at_target(2777)" --include=*.rs .` = **0 行**；`grep -rn "replace_shell_rows(2758-2776)"` = **0 行**。
- 该文件也不在 `37df838` name-status 内（`git show --name-only 37df838 | grep -c shop_purchase` = **0**）。
- **首审面9 红①原样复现。**

### ② entity_route 游离 #[test] 删 — ❌ 未删

HEAD 实态：

```
$ grep -n "#\[test\]" crates/game-core/src/scene_tests/entity_route_tests.rs
21 / 81 / 84 / 136 / 165 / 184 / 196 / 208 / 230
$ sed -n '78,95p' …
    #[test]                     ← :81
    /// 批3 红1②兜底断言（对称 op_mov dialog_target 断言……plan:69 明文盲区项）。
    #[test]                     ← :84（双 #[test] 夹 doc）
    fn see_op_seeds_dialog_target() { … }
    fn route_enemy_battle_when_stepping_into_player() {   ← :92 无 #[test]
```

`37df838` 对该文件的 diff 全量（仅 2 hunk，无任何 `#[test]`/fn 删除刀）：

```
$ git diff 37df838^ 37df838 -- crates/game-core/src/scene_tests/entity_route_tests.rs
@@ -1,5 +1,5 @@
-    use super::scene_test_support::{scene, ROW};
+    use super::scene_test_support::{scene, ROW, SceneSetup, apply_setup};
@@ -184,8 +184,7 @@
-        sc.route_entity = 0;
-        sc.facing = 0;
+        apply_setup(&mut sc, SceneSetup { route_entity: Some(0), facing: Some(0), ..Default::default() });
```

且该双 `#[test]` 与 never-used fn **仍在实报 warning**（见面2 原文）。**首审面9 红②原样复现。**

### ③ message census 一致 — ❌ 0 与 8 仍并存于同一 message

```
$ git log -1 --format='%B' 37df838 | grep -n "census\|warning\|警告"
1:refactor(core): P2 批10 …（…+workspace 警告清零）
38:## workspace 警告清零（批9 遗留黄2 同面）
41:entity_route_tests 游离 #[test] 删）——终态警告 census=0。
119:⑧ workspace 警告 census：
120:  warning 计数=8
$ … | grep -c "census=0"        → 1
$ … | grep -c "warning 计数=8"  → 1
```

正文仍留「cargo fix --workspace --tests 自动修 20 文件」（行39，宣称已撤下该项=未撤）。
**首审面9 红③原样复现；面5 同因。**

### ④ 计数措辞（FontMetrics/双分号/死码）— ❌ 未改

`git log -1 --format='%B' 37df838 | grep -n "FontMetrics\|双分号\|死码\|cargo fix"` 仍输出行 39-40：

```
39:cargo fix --workspace --tests 自动修 20 文件+手动 12 处（FontMetrics 未用 import 删×2/
40:死码 allow×5[preset 判别力保留]/preset_cases 双分号×2/images_slot 语句级 allow/
```

而实测 FontMetrics 未用 import 现存 **4 处 import / 3 处报 warning**（`head -1` 于 fidelity/entity/slot_equip/pointer_seam 四文件均含 `use crate::render::FontMetrics;`），与「×2」「实删 3 行」均不符（首审已判「diff 实删 3 行」）。

---

## 面2 census 实测（面5 黄销核） — 红：警告未清零

```
$ cargo test --workspace --no-run 2>&1 > /tmp/b10fix_nowarn.log   # EXIT=0
$ grep -c "^warning" /tmp/b10fix_nowarn.log   → 8
$ grep -c "warning"   /tmp/b10fix_nowarn.log  → 8
```

逐条（7 条真实警告 + 1 条汇总，行号即当前源码实位）：

```
6:warning: unused import: `crate::render::FontMetrics`
7: --> crates/game-core/src/scene_tests/fidelity_fix_tests.rs:1:9
14:warning: duplicated attribute
15:  --> crates/game-core/src/scene_tests/entity_route_tests.rs:84:5
22:warning: unused import: `crate::render::FontMetrics`
23: --> crates/game-core/src/scene_tests/entity_route_tests.rs:1:9
28:warning: unused import: `crate::render::FontMetrics`
29: --> crates/game-core/src/scene_tests/slot_equip_scene_tests.rs:1:9
34:warning: variable does not need to be mutable
35:    --> crates/game-core/src/scene_tests/slot_equip_scene_tests.rs:1080:33
44:warning: unused variable: `cy0`
45:    --> crates/game-core/src/scene_tests/t2_walkto_route_tests.rs:2570:19
52:warning: function `route_enemy_battle_when_stepping_into_player` is never used
53:  --> crates/game-core/src/scene_tests/entity_route_tests.rs:92:8
60:warning: `game-core` (lib test) generated 7 warnings (run `cargo fix --lib -p game-core --tests` to apply 4 suggestions)
```

- 7 条与首审列举的 8 项遗留**逐一同位**（3×FontMetrics import、:84 双属性、:1080 unused mut、:2570 cy0、:92 never-used），**一条未修**。
- 宣称「测试面警告=0」与「--tests 8 条已逐条修（defs_golden allow 条件化 + slot_equip 未用 mut 两新修）」：`crates/game-data/tests/defs_golden.rs:1-12` 头注无条件化 cfg 改动痕迹、`slot_equip_scene_tests.rs:1080` 仍为 `let (mut right, mut map_w)` → **两新修均未落**。
- **面5 黄未销（实测仍 7/8）**。

---

## 面3 批8黄②两 body 实界 — 黄（实界为真、三处载体未改）

实测（HEAD 源码 awk/sed 行锚）：

```
$ grep -n "fn perform_action\|fn interrupt_route" crates/game-core/src/scene.rs
1385:    pub fn perform_action(&mut self, a: crate::controls::Action) -> bool {
1632:    pub fn interrupt_route(&mut self) {
$ sed -n '1411,1415p' → 1411 `        }`（match 闭）/ 1412 `    }`（fn 闭）/ 1414 doc / 1415 engage_selected fn
$ sed -n '1643,1646p' → 1643 `        }`（if 闭）/ 1644 `    }`（fn 闭）/ 1646 interrupt_route_for_input doc
```

| 项 | 宣称值（修复件） | 实测 | 判 |
|---|---|---|---|
| perform_action body | 1386-1411 | **1386-1411**（fn@1385，1412=`}`） | ✅ 值对 |
| interrupt_route body | 1633-1643 | **1633-1643**（fn@1632，1644=`}`） | ✅ 值对 |
| 「1631-1643=批7 当刻 fn 全跨」注 | 应落 | 未落 | ❌ |

但**载体三处仍为旧值**（修复未落的直接证据）：

```
$ grep -rn "1631-1643" --include=*.rs .
./crates/game-core/src/scene.rs:1407: …（interrupt_route body=1631-1643 不触 tap_pending，序等价 A 级）
./crates/game-core/src/subsystem/input_mapper.rs:38: …interrupt_route body=1631-1643 不触 tap_pending
$ git log -1 --format='%B' 37df838 | grep -n "1407-1455"
（37df838 message 批8黄②段仍写「perform_action body=1407-1455，interrupt_route body=1631-1643」）
$ grep -rn "1386-1411\|1633-1643" --include=*.rs .
（0 命中——只在 docs/findings/reviews/* 首审报告里有）
```

**面7 黄未销**（实界确认为首审给的 1386-1411 / 1633-1643，但注与 message 未改）。

---

## 面4 回归 — 绿

```
$ cargo test -p game-core --lib p2b10
test subsystem::script_host::tests::p2b10_host_ctx_final_8_methods ... ok
test subsystem::script_host::tests::p2b10_blind_zones_cover_all_plan_named ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 374 filtered out

$ python3 gates/cli.py check
[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同
== 汇总 ==
PASS=13 FAIL=0 SKIP=0        （EXIT=0，13 项全列于 /tmp/b10fix_gates.log）

$ cargo test -p game-core          # 全 targets
test result: ok. 374 passed; 0 failed; 2 ignored …（lib）
… 共 23 行 test result，汇总 passed=526 failed=0；grep -c "FAILED"=0；EXIT=0

$ git show --name-status --format='' 37df838 | grep -Ec 'data/|status/|baselines'  → 0
  （30 文件全为 .rs；3e732e1 仅 gates/cli.py）→ 金样零漂移 ✅
$ git status --porcelain | grep -v '^??' | wc -l  → 0   （工作树无未提交改动）
```

---

## 面5 夹带 — 黄（对象数与宣称不符；两子项绿）

- **对象**：宣称 `f07992c` = 26 文件 +232/-253 → **无该对象**；实际报审件 `37df838`：

```
$ git show --shortstat --format='' 37df838
 30 files changed, 246 insertions(+), 150 deletions(-)
$ git show --name-status --format='' 37df838   # 30 行，全 M/*.rs
```

  30 文件全属宣称类别（scene_tests/subsystem/集成测试/scene.rs，均为测试构造面+注记+判据），**无 data/status/baselines/生产逻辑夹带**。
- **scene.rs 生产码零行为**：diff 唯一 hunk=`+#[cfg(test)]`（mod shop_purchase_tests 前补条件编译），无任何执行语句改动 ✅。
- **3e732e1 恰 1 文件**：`gates/cli.py | 2 +-`（1 insertion 1 deletion）✅。
- 判黄仅因**宣称的 26 文件/+232/-253 与实测 30/+246/-150 不符**（属面6 同因）。

---

## 面6 claims-must-match — 红（修复件数项无对象）+ 三项实测为真

| 宣称 | 实测 | 判 |
|---|---|---|
| 修复件 26 文件 +232/-253 | 对象不存在；实际 `37df838`=**30 文件 +246/-150** | ❌ 无对象/数值不符 |
| 界注更正落文件 | `grep camera_at_target(2777)`=0 | ❌ |
| 游离 #[test] 真删（:81/:84/:92） | 三处原样在位 | ❌ |
| message census 全文一处口径 | 同 message 内 census=0 与 warning 计数=8 并存 | ❌ |
| FontMetrics 3 行/双分号 2 处/死码 allow 4 处措辞改实测 | message 仍 ×2/×5/×2（行39-40） | ❌ |
| 批8黄②两 body 改实界 1386-1411/1633-1643 | 三载体仍旧值 | ❌ |
| 测试面警告=0（含 defs_golden allow 条件化、slot_equip 未用 mut 两新修） | 实测 7 条真实警告，两新修均未落 | ❌ |
| 撤下「cargo fix 20 文件」 | message 行39 仍在 | ❌ |
| D 项 392→320 | `git grep … c2dbd9c`=**392**，HEAD=**320** | ✅ |
| SceneSetup 恰 40 字段 | `grep -c "pub .*: Option"`=**40** | ✅ |
| HostCtx 恰 8 名 | awk trait 体 `grep -o "fn [a-z_0-9]*"`=8：find_path/snap_camera_to_player/locate_speaker_entity/ces_camera/set_camera_anchor_target/exit_application/mvs_exchange/snap_camera_after_ros1 | ✅ |

---

## 需修复项（按优先级）

1. **修复件缺失（阻断）**：`f07992c` 未在本仓库落地——请确认修复是否在别的克隆/未提交；若确未做，则按首审三条（界注一行、entity_route 三刀、message census 单口径）+ 面5 七条警告实清 + 面7 两 body 实界三载体，重新 amend 并给出可核 hash。
2. **面9 红三项**：①shop_purchase_tests.rs:64-65 界注改实文；②entity_route_tests 删 :81/:84 双 `#[test]` 与 :92 never-used fn；③message 正文 census=0 与判据段「warning 计数=8」二选一，全文一致（并同步撤下「cargo fix 20 文件」「FontMetrics ×2」等失实计数）。
3. **面5 黄**：7 条测试面警告实清后再称 0（3×FontMetrics import / :84 双属性 / :1080 unused mut / :2570 cy0 / :92 fn）。
4. **面7 黄**：scene.rs:1407、input_mapper.rs:38、message 批8黄②段三处 body 范围改 1386-1411 / 1633-1643（或补「1631-1643=批7 当刻 fn 全跨」注）。
5. **面5/6 对象数**：报审自述须与 `git show --shortstat` 一致（实测 30 文件 +246/-150）。

Confidence: 94
