# P1-a2 批4 独立对抗复核（逐提交纪律·差分验证员）

> 复核对象：commit `e2f172a`（DDDSA P1-a2 批4：fx-pickup-33/37/29f12 三件夹具+三账清+default 显式递延）
> 复核人：独立对抗复核员（差分验证员纪律，AGENTS.md 权威）
> 复核时间：2026-11-04
> 三纪律核验：sha 贴原文 / 引锚 sed 实文 / 实证段禁手粘
> 状态：未推送（HEAD=e2f172a，绿=lead 推送）

## 0. 结论速览

| # | 攻击点 | 判 |
|---|---|---|
| 1 | 三件真伪（33/37 tryEquip 共享体对拍 + 29f12 floorTier 算式核） | 🟢 |
| 2 | 链 sha 独立重算 + gen 重生成逐字节对拍 + 禁手粘贴件逐字核 | 🟢 |
| 3 | default 臂递延可达性实证 + 递延注记/路由清 | 🟢 |
| 4 | 三账清（审计基线/探针留档/矩阵⚠15+method_reach） | 🟡（账③ method_reach 口径） |
| 5 | 金样 70 纯追加 + 夹带净 + 批1-3 未动坏 + 边界注留档 | 🟢 |

**总判：可推送（带黄注 1）**——三件真伪、gen 逐字节重生成、金样、边界注全绿；唯账③
"method_reach 随重生成测"措辞含糊且按批3建议①读未兑现（见 §4），矩阵账面如实
零虚记，不阻断推送；建议下批清 fx-item25a stem 冲突后全量重测 method_reach。
门禁预检实跑：`python3 gates/cli.py status` ⇒ **PASS=13 FAIL=0 SKIP=1（visual-diff 显式
SKIP）TOLERATED=0**（exit 0）。

## 1. 攻击点1：三件真伪（33/37 tryEquip 共享体对拍 + 29f12 floorTier 算式核）🟢

### 1.1 引锚 sed 实文（独立复核，`reference/src/deobf/a.java`）

`sed -n '7565,7572p'`（case 29 臂，message/test 注释引锚）：
```java
         case 29:
            if (this.currentFloor <= 10) {
               this.playerAtk++;
               return;
            }

            this.playerAtk = this.playerAtk + this.floorTier();
            return;
```
`sed -n '7632,7648p'`（33..=40|79|80 tryEquip 共享体）：
```java
         case 33:
         case 34:
         case 35:
         case 36:
         case 37:
         case 38:
         case 39:
         case 40:
         case 79:
         case 80:
            if (this.tryEquip(var2, false)) {
               this.itemUseTargetType = (byte)var1;
               this.showOverlayMessage((byte)1, this.equipDescriptions[var2], (byte)0, (byte)3);
            } else {
               this.showOverlayMessage((byte)0, this.miscTexts[5], (byte)0, (byte)3);
            }
      }
   }
```
`sed -n '7651,7662p'`（floorTier）：
```java
   private int floorTier() {
      int var1;
      if ((var1 = (var1 = this.currentFloor - 1) / 10) < 0) {
         var1 = 0;
      }

      if (this.currentFloor > 50) {
         var1 = 0;
      }

      return var1 + 1;
   }
```
三锚行号与 message/test 注释主张**逐位吻合**（case 29 起 7565 / case 33 起 7632 /
floorTier 起 7651，`grep -n` 实测同值）。

### 1.2 武器/防具档位语义（33/37 共享体对拍）——A 级闭合

- 分组表 `a.java:952-953` 实文：`equipTierTypes = {0, 33, 34, 35, 79, 36, 0, 37, 38, 39, 80, 40}`；
  `equipTierBonuses = {0,10,30,70,120,220, 0,10,30,70,120,220}`。
  **33/34/35/79/36 落下标 1..=5（<6=武器档）；37/38/39/80/40 落下标 7..=11（≥6=防具档）**
  ⇒ "33..=36|79 装备武器组 / 37..=40|80 防具组"主张成立，与 message 的组界表述逐字同。
- `tryEquip`（a.java:7677-7695）：`var1<6` ⇒ 武器档（`playerAtk ± equipTierBonuses`，
  `equippedWeaponType=equipTierTypes[var1]`）；否则防具档（`playerDef ±`，`equippedArmorType`）。
  档位语义=仅当 `equipTierIndex(现)<新档`（或 var2 强制）才换装——升级单向。
- 调用面 `pickupItemType`（a.java:7534 起）：`var2 = equipTierIndex(var1, 0, 12)`（表下标）后
  共享体 `tryEquip(var2, false)`——33/37 确为**同体**（同 case 块、同调用），分档只发生在
  tryEquip 内部按 var2 值域。

### 1.3 29f12 floorTier 算式核 + 200→202 钉

- `floorTier`（7651-7662）：`((floor-1)/10)`，负钳 0、floor>50 钳 0，`+1`。
  算式核 **(12-1)/10+1 = 1+1 = 2** ✓（message 表述与实文同式）。
- case 29 臂（7565-7572）：floor>10 ⇒ `playerAtk += floorTier()` = 200+2 = **202**。
- 夹具钉：`fx-pickup-29f12.preset` MOT_IF/MOT_L0 解码 `atk=0x000000c8=200`、`floor=0x0c=12`
  ✓；测试断言 `assert_eq!(sc.player.atk, 202)`（fixture_frames.rs）——算式、预置、断言三点闭合。

### 1.4 实测值与场景链对拍

- 三件场景链 = **逐拍整屏像素 sha**（replay 机制 `assert_eq!(sha, rec.sha, "TICK {} 帧不符")`，
  fixture_frames.rs:339）——Rust 每拍渲染帧 sha 对 Java A 参照帧逐拍比对，`checked==320`
  全帧覆盖。档位/浮层/HUD 数值任何分歧即像素分歧，故 tryEquip 档位语义已被链全覆盖。
- 三测实跑：`cargo test -p game-oracle --test fixture_frames fx_pickup` ⇒
  `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 32 filtered out`
  （9 = 批2 的 26/27/28 + 批3 的 29/30/31 + 批4 的 33/37/29f12）✓ 与 message 贴件计数全同。
- 终态断言对拍：33 南步入 (11,11) + `equipped_weapon>0`；37 北步入 (9,7) + `equipped_armor>0`；
  29f12 北步入 (6,1) + `atk==202`——与各自脚本/preset 的位点与步向逐项一致
  （preset 解码：floor5 x=11 y=10 / floor9 x=9 y=8 / floor12 x=6 y=2，facing=1）。

## 2. 攻击点2：链 sha 独立重算 + gen 重生成逐字节对拍 + 禁手粘逐字核 🟢

### 2.1 链 sha 独立重算（自写脚本，非项目代码）

`frame_chain = sha256("\n".join(FRAME-sha 列))` 独立重算 vs 夹具头 vs message 贴件：

| 夹具 | 帧数 | 头声明 | 独立重算 | 同? | 贴件 short16 |
|---|---|---|---|---|---|
| fx-pickup-33 | 320 | c6b4effe3dc3d8d1…71a3a22 | 同 | ✓ | c6b4effe3dc3d8d1 |
| fx-pickup-37 | 320 | a7a6be3469347b52…3c17856 | 同 | ✓ | a7a6be3469347b52 |
| fx-pickup-29f12 | 320 | 99794e26e19c8eba…b5ed4bb4 | 同 | ✓ | 99794e26e19c8eba |

三件 TICK 连续 1..320 ✓。**贴件三行以独立脚本重跑提取 = diff 空**（实证）。

### 2.2 gen 重生成逐字节对拍（真伪终审）

`python3 tools/frames_fixture.py gen <脚本>` ×3 重跑（A/B/C 三变体 Java oracle 全链）：
- 三方 trace sha256 逐件一致（A=B=C）：33=`2d76319cbd31…`、37=`bfb4e92e9b9a…`、29f12=`c29c8e7ecd0c…`，
  与夹具头登记的源 trace sha 同值；
- 重生成 `.frames.txt` 与入库件 sha256 对拍：**REGEN BYTE-IDENTICAL**（三件全同）。
⇒ 夹具为 oracle 管线真实产出，**非手拼**；禁手粘主张的最强证据成立。

### 2.3 Evidence 贴件逐字核（禁手粘·重跑提取）

| 贴件行 | 重跑实测 | 判 |
|---|---|---|
| `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 32 filtered out; finished in 4.31s` | 计数全同（finished in 4.36s） | ✓（计时为天然非确定项） |
| `[GREEN] 金样逐字节等价：70 场景 sha 全同` | 逐字节同 | ✓ |
| 链 sha 三行 | 逐字节同（§2.1） | ✓ |

## 3. 攻击点3：default 臂递延可达性实证 🟢

### 3.1 可达性推理链核（A 级闭合）

1. **图拾守卫**：scene.rs:4122-4125（message"4123 区"实文）——实体分派 `2 | 4 => {
   self.pickup_item_type(t); entity::remove(...) }`，只有 renderCategory 2|4 的实体进 pickup；
2. **cat 2/4 型域**（renderCategoryByType，a.java:6268-6298；entity.rs `legacy_render_category_mirror`
   保真镜像实文）：cat2 = **13..=32 | 85 | 86**，cat4 = **33..=40 | 79 | 80** ⇒ 并集
   **13..=40|79|80|85|86**；
3. **dispatch 臂域**（scene.rs:4242 起 `pickup_item_type`）：显式臂 13..=25|85|86 / 26 / 27 / 28 /
   29 / 30 / 31 / 32 / 33..=36|79 / 37..=40|80 = **13..=40|79|80|85|86 与型域全同**，
   `default => {}`（"41+ 怪物类型不可达"注）。
⇒ **图拾路径 default 臂不可达**——推理链三点实文闭合，主张成立。

### 3.2 GIN 脚本面 + 递延注记显式 + 路由清

- GIN 面实证：script_ops.rs:75-77 `op_gin`（`GIN_a_b`，a.java:8424-8437——evidence-todos
  T-1 语料表"G118424 | 1804 已移植 | 4 | 活码"）：kind 0 ⇒ `self.pickup_item_type(value)`
  **裸型注入**（同 scene.rs:2765 商人臂 `self.pickup_item_type(18)` 同式直调）——脚本面可注入
  型域外值命中 default，夹具车成立。
- **递延注记显式** ✓：探针留档 §"default no-op 臂可达性分析（批4 递延注记）"（"显式递延，
  非遗漏"）+ message 正文同语。
- **路由清** ✓：arm_coverage.rs `ARM_FIXTURES` 21 组清点表保留 `("pickup default no-op",
  "fx-pickup-default")` 槽位，且 `p1a2_all_21_arm_groups_have_fixture_triplets`（解除 #[ignore]
  前强制）要求 fx-pickup-default.frames.txt 存在——递延项不丢账，最终 21 绿前必落。
  微瑕：该行未像 `"pickup 32（递延：销账批）"` 那样内联标"递延：脚本面批次"（注记在探针档
  单点），建议下批补内联标记。

## 4. 攻击点4：三账清 🟡（账①② 绿，账③ 部分黄）

### 账① 审计基线 f2283a7 钉 🟢

- `git rev-parse f2283a7^{tree} f25bd49^{tree}` ⇒ **两者同树** `0d4b57d09eb1dcf1…`（`git diff
  f2283a7 f25bd49` 空）——钉 f2283a7 与批3 终稿 f25bd49 内容等价，e2f172a^=f25bd49。
- "勿用 7166569 字面判据"与快审 v2 黄3 裁定一致（`docs/findings/reviews/2026-11-04-p1a2-batch3-quick-v2.md`：
  7166569→f25bd49 非零 diff，v2 步树同零 diff，"审计口径钉 f2283a7 基线"）——口径钉有据。

### 账② 探针留档 🟢

`docs/findings/2026-11-04-pickup-probe-notes.md`（本批 +28 行新增）含：位点表（13/26/27/28/
29/30/31/33/37/34-36/38/81-88）+ floor1 剧情触发面注记 + default 可达性分析——与 message
主张三项全符；33=sprite5 (11,11)/37=sprite9 (9,7)/29f12=sprite12 (6,1) 三处已由 §1.4 绿测交叉实证。

### 账③ 矩阵⚠15 稳 🟢 + method_reach 随测 🟡

- **矩阵重生成实证**：`python3 tools/cover_matrix.py --check` ⇒ `覆盖矩阵 --check：逐字节一致`
  （exit 0）——入库 coverage-matrix.md 与重渲染逐字节同，"重生成"主张可复现 ✓。
- **⚠15 稳**：`grep -c "^- ⚠"` f25bd49=15 / e2f172a=15 ✓（计数排除 §5.1 标题字符）。
- **method_reach 随重生成测 🟡**：
  - `tools/method_reach/snapshot.json` **未随批重测**（66 场景键中**无任何 fx-pickup-***，
    含批 2/3 的 26..31；快照零改动）；三件新夹具在矩阵中如实记 `T1-None | 无触达快照 | 0`
    （cover_matrix.py:1152 逻辑：快照缺条 ⇒ 不计验证窗——**零虚账**）。
  - `python3 tools/method_reach.py --check` 现 **exit=1**："夹具场景 stem 冲突：
    crates/game-core/tests/fixtures/fx-item25a.txt"——`reference/oracle/scenarios/fx-item25a.txt`
    与 game-core 夹具同名（fixture_scenarios 硬拒）。**30ff91c（2026-11-01）既有**，
    非本笔引入（`git merge-base --is-ancestor 30ff91c f25bd49` 成立）。
  - 批3 复核建议①"重跑 `tools/method_reach.py`"**未兑现**（任何 method_reach 调用现均
    崩于同一冲突）。"随重生成测"若指矩阵重生成内部的 frame_chain 口径核——措辞应改明；
    若指触达快照重测——未发生，按"未贴出的运行结果不存在"不成立。
  - 处置：🟡 口径更正/下批清——①修 fx-item25a stem 冲突（改名或挪位）②全量重测
    method_reach 刷快照 ③fx-pickup 系列行转"FRAME 一致（夹具帧链 = 实测）"。

## 5. 攻击点5：金样/夹带净/批1-3 未动坏/边界注 🟢

- **金样 67+3 纯追加**：`state-dump-golden.json` 条目级实数 f25bd49=67 → e2f172a=**70** ✓；
  diff 仅 3 组 ×5 行插入（+15/-0，fx-pickup-29f12/33/37.state，键序合规插入），无任何既有
  条目改写/删除 ✓。`python3 tools/state_baseline.py --check` ⇒ `[GREEN] 金样逐字节等价：
  70 场景 sha 全同`（与贴件逐字同）✓。
- **夹带净**：`git show --name-status e2f172a` 实测 **13 文件** = 3×(frames.txt+.preset+.txt)
  9 件 + fixture_frames.rs（**+36/-0 纯增**）+ state-dump-golden.json + coverage-matrix.md
  + pickup-probe-notes.md；gates/ reference/ original/ data/naming/ prompts/ **零触碰** ✓
  （1129 插入/6 删除=矩阵 6 行行号平移，无他）。
- **批 1-3 绿项未动坏**（实跑）：fixture_frames 全量 **41 passed / 0 failed**（含批1 fx-item13
  系、批2 26/27/28、批3 29/30/31 全绿）；gameplay_floor1 **2 passed**（3 ignored 为登记项）；
  preset_cases **35 passed**；game-oracle lib 5 passed——无回退。
- **边界注留档** ✓：fx-pickup-33.txt 头注第 4 行 `（**边界注**：首版 (11,12) 出界 y>11 致
  236 拍止）` + message 同文 + "改 (11,10) 南步后 320 拍全"（现夹具实测 320 帧 ✓）+
  复现类交叉注（fx-pickup-26 floor1 剧情 236 同数异因——26 夹具头注"sprite1 备选因 floor1
  剧情触发面弃"呼应）。

## 6. 三纪律核验

| 纪律 | 判 | 实证 |
|---|---|---|
| sha 贴原文 | 🟢 | 链 sha 三行独立脚本重跑**逐字 diff 空**；test result 计数全同（计时 4.31/4.36 非确定项除外）；state_baseline 行逐字同 |
| 引锚 sed 实文 | 🟢（附观察） | 三锚（7632-7648/7651-7662/7565-7572）独立 sed 复核与主张逐位吻合。观察：message 未贴 sed 实文（批3 先例为"20 行全贴"），仅记"随 sed 输出核"——锚真值经本档 §1.1 留实文补证，建议下批按先例随贴 |
| 实证段禁手粘 | 🟢 | gen 重生成逐字节同（最强证）+ state_baseline 逐字同 + 链行逐字复现 ⇒ 贴件可复现非手编 |

## 7. 总判

**可推送（带黄注 1）**。

- 绿：三件真伪（共享体/档位表/算式三点 A 级闭合）、gen 重生成逐字节对拍、链 sha 独立
  重算、default 递延可达性+显式注记+路由清、金样 67→70 纯追加、夹带净、批 1-3 全绿、
  边界注留档。
- 黄（唯一）：账③ "method_reach 随重生成测"口径——触达快照未随批重测、method_reach.py
  因 fx-item25a stem 冲突（既有）不可运行、批3 建议①未兑现；矩阵账面如实零虚记，
  不构成回退/虚账，故不阻断推送。
- 行动项（下批）：①清 fx-item25a stem 冲突 ②`tools/method_reach.py` 全量重测刷快照
  ③fx-pickup 系列矩阵行转 FRAME 一致 ④（微）ARM_FIXTURES default 行补内联递延标记
  ⑤（微）引锚 sed 实文随贴先例。
