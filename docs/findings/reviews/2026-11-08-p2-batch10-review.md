# P2 批10 对抗复核（2026-11-08）

- **报审件**：`37df838`（30 文件 +246/-150）+ 拆分件 `3e732e1`（gates/cli.py 注释 1 行）
- **口径**：报审自列八点 + 核心回归，逐面红/黄/绿；全部断言带实测输出
- **总判：需修复**（面9 红×1、面5/面7 黄×2、其余 7 面绿）

| 面 | 判 | 一句话 |
|---|---|---|
| 1 D 对账 | 绿 | 392→320 复跑一致，SceneSetup 恰 40 字段，run 行删净 |
| 2 F HostCtx | 绿 | trait 内恰 8 `fn`，8 名逐一=宣称名单 |
| 3 C 终裁注记 | 绿 | script_host.rs:7 实文覆盖宣称四簇 |
| 4 E 盲区 | 绿 | 14 token 四域齐全，PLAN_NAMED_PENDING=&[] |
| 5 判据 | 黄 | 判据主体全绿；**警告 census 宣称 0、实测 8（--tests），同一 message 内 0/8 自相矛盾** |
| 6 勘误 7 类 | 绿 | battle 4 锚+a.java 语义+persist 9201+mod 3读+ease 27-34 全对 |
| 7 批8黄②更正 | 黄 | :1406→:1407 与 1646 两值实测为真；**「perform_action body=1407-1455」「interrupt_route body=1631-1643」两范围与实界不符** |
| 8 夹带 | 绿 | 30 文件全属宣称类别，scene.rs 仅+1 行 cfg(test)，3e732e1 恰 1 文件 |
| 9 claims-must-match | **红** | **「批9两黄清·shop_purchase 界注更正」宣称已清、实未落（文件不在 commit，HEAD 仍旧注）**；「entity_route 游离 #[test] 删」宣称删、diff 无此刀；census 0 vs 8 |
| 10 落库工艺 | 绿 | reflog 链 916ef0d→ca2ae02→37df838 实证；批8黄②更正段实载 |

---

## 面1 D 对账 — 绿

plan 原命令（`grep -rEn "\b[a-z_][a-z_0-9]*\.[a-z_][a-z_0-9]* = " crates/game-core/src/scene_tests/ | grep -v " == "`）：

- `c2dbd9c`=**392**，`37df838`=**320**，HEAD=**320**（−112，与「−112 字面量化+40 apply 体构造层写」算术闭合）。
- SceneSetup 恰 **40** 字段（`grep -c "pub .*: Option"`=40；apply_setup 体内 `if let Some`=40）；锚 190/234/240 sed 全中。
- 抽 3 个 run 端点（t2_walkto/slot_equip/pointer_seam）均已 apply_setup+`SceneSetup {` 形，旧 run 直写删净。

## 面2 F HostCtx — 绿

`awk '/^pub trait HostCtx/,/^}' crates/game-core/src/script.rs | grep -c "fn "` = **8**；8 名=find_path/snap_camera_to_player/locate_speaker_entity/ces_camera/set_camera_anchor_target/exit_application/mvs_exchange/snap_camera_after_ros1，与宣称名单逐一相同。`cargo test -p game-core --lib p2b10` → **2 passed**（host_ctx_final_8 + blind_zones_cover_all）。

## 面3 C 终裁注记 — 绿

`script_host.rs:7` 实文=「【P2 批10 C·Coordinator 遗留直写终裁】…op_* 编排体 / apply_merchant_offer 收尾块 / paint 簇 97 fn / ease 相机簇 4 setter 均归编排面登记不迁」——宣称四簇全覆盖。

## 面4 E 盲区 — 绿

测试体内 14 token=dialog5[typewriterPos/dialogSpeaker/dialogAnchor/dialogTarget/overlay]+persist4[minFloor/maxFloor/pendingFloor/wentDown]+menu4[menuCursor/itemCursor/recordPage/difficultyIndex]+tap1[autoBattle]；`PLAN_NAMED_PENDING: &[&str] = &[]` @ script_host.rs:111。

## 面5 判据全独立重跑 — 黄（判据主体全绿；警告 census 矛盾）

独立重跑结果：

- p2b10：**2 passed / 0 failed** ✅
- `gates/cli.py check`：**PASS=13 FAIL=0 SKIP=0**，含 state-baseline「[GREEN] 金样逐字节等价：74 场景 sha 全同」✅
- `cargo test -p game-core --lib`（debug 全量）：**374 passed / 0 failed / 2 ignored** ✅
- `cargo test -p game-oracle --test fixture_frames`：**45 passed / 0 failed**（97.27s）✅
- 金样零漂移：`37df838 --name-status` 无 data/status/baselines 任何文件（grep=0）✅
- Java 三方：scenarios.md **47 行 PASS**，`3ad6ef6ad65b` 在档 ✅

**警告 census（黄因）**：

- `cargo build --workspace`（不带 --tests，touch lib.rs 强制重编）= **0 warning** ✅（lib/bin 面清零为真）。
- `cargo build --workspace --tests` = **8 warning** ❌：
  1. 3× unused import `crate::render::FontMetrics`（fidelity_fix_tests.rs:1 / entity_route_tests.rs:1 / slot_equip_scene_tests.rs:1）
  2. duplicated attribute（entity_route_tests.rs:84，:81 与 :84 双 `#[test]` 夹 doc）
  3. unused mut（slot_equip_scene_tests.rs:1080 `let (mut right, mut map_w)`）
  4. unused variable cy0（t2_walkto_route_tests.rs:2570）
  5. never used fn（entity_route_tests.rs:92 `route_enemy_battle_when_stepping_into_player` 无 `#[test]`）
- **8 条均为批9 及更早遗留**（逐条对 `c2dbd9c` 同文存在：FontMetrics import@1 ×3、:84 双 #[test]、:92 无 #[test] 同文；`let (mut right` c2dbd9c:1079、`let (cx0, cy0)` c2dbd9c:2594 同文，行号仅 +1/−24 漂移；批10 diff 未触碰这些代码行，也未删任何 FontMetrics 使用点）——批10 既未引入也未清除。
- **同一 commit message 自相矛盾**：正文（行41）「终态警告 census=0」 vs 判据段（行119-120）「⑧ workspace 警告 census：warning 计数=8」。

## 面6 勘误 7 类实核 — 绿

- battle.rs 4 锚：8760-8780→8771-8787、8769→8775、9983-9982→9992-9996×2 全对；`reference/src/deobf/a.java:8771-8787` 实含先手 8783-8784（`var7 = var10.enemyAtkScaled[var18 - 41] - var10.playerDef`）、8775=enemyHpScaled 行、9992-9996=predictHpLossVsType 全体。
- persist_codec:43 9192→9201，a.java:9201=`writeByte(equippedWeaponType)` ✅（:44 equippedArmorType 仍标 9193=未宣称面）。
- mod.rs ScriptCtx 2读→3读注+entities 去 mut+缩进 ✅。
- ui_render ease 锚 26-33→27-34，scene_paint.rs:27-34=4 setter（set_ease_cam_x/y+set_ease_target_x/y）✅。

## 面7 批8黄②更正注三锚 — 黄

37df838 message 尾段【批8 黄② 原地更正】实载。三值实核：

| 宣称 | 实测 | 判 |
|---|---|---|
| 批注真身=**1407**（原 :1406） | `1407: // pending=StopThen(c) 装配已在 resolver（interrupt_route body=1631-1643…）` | ✅（跨 4 commit 追踪：6ce6f85 批7 当刻=1406 为真，7f9e7ff 批8 起漂至 1407，c2dbd9c/37df838 同 1407；更正对批7/批8 message 历史主张成立） |
| interrupt_route_for_input doc 行=**1646** | `1646: /// 【STAGE-B-008 · 复核 T4-R4】按键输入口的打断…`（fn@1650） | ✅ 精确 |
| perform_action body=**1407-1455** | fn@1385，body=**1386-1411**（1412=`}` 闭合；1414-1415=engage_selected、1427-1454=swipe_step、1455=空行） | ❌ 起点非 body 首行（1407=InterruptThen 臂内批注行）、终点越界跨入 engage_selected/swipe_step 至空行；「（perform_action body 首行）」标注亦不实（body 首行=1386） |
| interrupt_route body=**1631-1643** | fn@1632，body=**1633-1643**（1631=末 doc 行、1632=fn 签名、1644=`}`） | ⚠ 终点=body 末行✅；起点偏 2 行（1631-1643 恰=批7 当刻 fn 全跨 1631-1643，批8 起 +1 漂移后未同步；scene.rs:1407 批注与 input_mapper.rs:37-38 均沿用同值，三处自洽但与 HEAD 实界差 2） |

结论：更正的核心（:1406→:1407）为真，但「以本注为准」的两个 body 范围均非实界 → 黄（行锚精度瑕疵，非行为问题）。

## 面8 夹带 — 绿

- 30 文件全清单（name-status）：game-core src 17 + tests 4 + game-data 1 + game-oracle 2 + shell/android 6——全部属「测试构造面/警告修/勘误/注记」宣称类别。
- scene.rs 生产码 diff=**+1 行**（`#[cfg(test)]` @4631，mod shop_purchase_tests 前，gold_popup_tests 同形）→ 零行为。
- controls.rs mut→非 mut、menu_family.rs 删 3×FontMetrics import+allow、item_service.rs 删 import、input_mapper/battle/persist/mod/ui_render=注释或测试面 → 零行为。
- `3e732e1` 恰 1 文件（gates/cli.py，60→74 场景 docstring）。

## 面9 claims-must-match — 红

逐项对照（宣称→实测）：

| 宣称 | 实测 | 判 |
|---|---|---|
| 30 文件 / +246 / −150 | numstat 实测 30 文件 +246 −150 | ✅ |
| 392→320 / 112 站点 / 40 字段 / 8 名 | 全部复跑一致 | ✅ |
| **批9两黄清①：shop_purchase_tests.rs 界注措辞更正（实界=camera_at_target 前含 replace_shell_rows）** | **37df838 name-status 不含该文件（grep=0）；916ef0d/ca2ae02 同样不含；HEAD 与工作树 `:64-65` 仍旧注「到下一 fn replace_shell_rows 前止——超出体的后续 helper 不入判定面」——正是批9-fix-review §4 判「与实际相反」的原文；全文无 camera_at_target/2777 字样；git status 无未提交修改** | ❌ **宣称已清、实未落** |
| 批9两黄清②：mod 补 #[cfg(test)] | scene.rs:4631 `#[cfg(test)]` 已补 ✅ | ✅ |
| 警告清零「entity_route_tests 游离 #[test] 删」 | 批10 对该文件仅 2 个 hunk（import 行 + :184 迁移），无 #[test] 删除；HEAD :81/:84 双 #[test]、:92 never-used fn 仍在并实报 warning | ❌ 宣称删、diff 无此刀 |
| 终态警告 census=0 | `--workspace --tests`=8（见面5）；message 判据段自报 8 | ❌ 与实测及自身判据段矛盾 |
| FontMetrics 未用 import 删×2 | diff 实删 3 行，全部在 menu_family.rs | ⚠ 计数差 1（次要） |

其余（amend 链、批8黄②段载入、金样未动）均实证为真。

## 面10 落库工艺披露 — 绿

reflog 实证：`916ef0d (commit) → ca2ae02 (amend) → 37df838 (amend) → 3e732e1 (commit)`，与宣称链一致；37df838 message 含【批8 黄② 原地更正】段=宣称「载」=实载。

---

## 需修复项（按优先级）

1. **面9 红·界注**：把批9-fix-review §4 建议的一行界注更正真正落进提交（改为「到 `fn camera_at_target`(scene.rs:2777) 前止——含 replace_shell_rows(2758-2776)」），或若决定不在本批修，则从 message 撤下「两黄清」中该项的完成宣称（改登记为挂账）。当前状态=宣称与仓库直接冲突。
2. **面5/面9·census**：message 正文「census=0」与判据段「计数=8」二选一并使全文一致；「entity_route 游离 #[test] 删」「FontMetrics 删×2」两个子宣称与 diff 不符，需删改或补刀。8 条测试面警告为批9 遗留，可实际清除（3×FontMetrics import、:81 双 #[test]、:92 补 #[test] 或删、unused mut、cy0）后再称 0。
3. **面7·行锚**（非阻塞）：更正注中「perform_action body=1407-1455」「interrupt_route body=1631-1643」建议改为实界（1386-1411 / 1633-1643），或注明「1631-1643=批7 当刻 fn 全跨」。

Confidence: 92
