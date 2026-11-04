# P1-a2 批2 对抗复核报告（commit bfc8712）

> 复核人：独立对抗复核员（逐提交纪律·差分验证员纪律，AGENTS.md 权威）
> 复核对象：`bfc8712` test(oracle): DDDSA P1-a2 批2——pickup 26/27/28 三臂夹具+两账清
> 状态：未推送；本报告结论 = lead 推送与否的依据
> 前情：批 1 三轮闭环（措辞钉实测边界纪律生效）
> 复核方法：逐攻击点红/黄/绿 + 实测复跑（不复述自述，只认机器输出）

## 0. 复核方法与口径

- 只认**实跑命令输出**与**逐字对拍**；commit message 自述仅作待验主张。
- 红=可证伪缺陷；黄=口径/完备性不足但不阻断；绿=实测通过。
- 总判二值：可推送 / 需修复。

## 1. 三臂真伪（fx-pickup-26/27/28 ↔ a.java:7556-7564）——🟡（夹具本体绿，26 臂 commit 金样身位红）

### 1.1 case 26 黄/27 红/28 蓝 对拍——🟢

`reference/src/deobf/a.java` 逐行实核：

| 行 | 真源 | 夹具断言 | 判 |
|---|---|---|---|
| 7556-7558 | `case 26: this.yellowKeyCount++; return;` | `sc.player.yellow_keys == 2`（双拾，见 §2） | ✓ |
| 7559-7561 | `case 27: this.redKeyCount++; return;` | `sc.player.red_keys == 1` | ✓ |
| 7562-7564 | `case 28: this.blueKeyCount++; return;` | `sc.player.blue_keys == 1` | ✓ |

黄/红/蓝语义、计数方向（++）、case 分派与拾取键语义逐字吻合。

### 1.2 A/B/C 三方一致 + 链 sha 独立重算——🔴（3 链中 1 链与产物不符）

独立重算（`frame_chain = sha256("\n".join(shas))`，= tools/frames_fixture.py 口径）：

| 臂 | A/B/C trace sha（实测三方一致） | 帧链实测 | commit 声称 | 判 |
|---|---|---|---|---|
| 26 | ffdcda46…（A=B=C ✓，=头注） | 320 帧 → **92f980cc…** | 236 帧链 **653a5008…** | 🔴 不符 |
| 27 | 5d66743b…（A=B=C ✓） | 320 帧 → ea781bbac… | 320 帧链 ea781bbac… | ✓ |
| 28 | de3b4174…（A=B=C ✓） | 320 帧 → a03d0fa0… | 320 帧链 a03d0fa0… | ✓ |

- 26 臂：夹具=320 帧链 92f980cc…（头注正确、独立重算一致）；commit"金样=A/B/C 三方一致抽取（236/320/320 帧链 653a5008…）"的 236/653a5008 **均与产物不符**——`653a5008` 全库唯一出现处 = `.git/COMMIT_EDITMSG`，236 前缀链实测=ce275bc7… 亦非其值。判：三次 regen 叙事中的早期数字**未随终版刷新**混入 Evidence 行，"3 链 sha"作 A 级证据 = 1/3 不实。
- 夹具帧 = A-trace 帧逐 sha 重算：三臂 320/320 全等 ✓（抽取无手改）。

### 1.3 判定

夹具本体（帧链/三方一致/抽取）🟢；commit 金样身位注记（26 臂）🔴。位点探针注记（sprite2 (3,4)/sprite8 (2,2)/sprite3 (5,3)）在 trace 文本中仅有 head 前缀、全量在 sha 体不可直读 ⇒ 不可独立复算，行为面已由全帧比+断言+金样钉死（🟡 注记面，见 §2）。

## 2. 实证注记核（26 臂 floor1→floor2 + yk 计数）——🟡

- **语义链可复现部分**：
  - `yk=0 显式` ✓：fx-pickup-26.preset MOT_L0 yk 字段实测=0x0000（生成器缺省 g("yk",1)=1 ⇒ "显式置零"属实）；
  - **yk 计数断言钉死** ✓：`assert_eq!(sc.player.yellow_keys, 2, …)` + 全帧比对 Java 参照（=差分裁判）+ 金样 state dump 入冻结面。
- **历史叙事不可独立复算**（🟡）："floor1 剧情 236 拍止 / floor2 yk 默认非零 3 / 三次 regen" 无已提交产物承载（analysis/=可再生区、floor1 弃臂 run 未留档）——属过程注记，非行为面，但按措辞纪律应标"历史观察（工作区未留档）"。
- **场景脚本注记 stale**（🟡）：fx-pickup-26.txt 断言行写 `yellowKeyCount 0→1`，与实测断言 `==2`（双拾）矛盾——早期 regen 注记未随终版刷新（与 §1.2 的 236/653a5008 同根因）。
- **行锚笔误**（🟡）：fx-pickup-28.txt 写 `a.java:7562-7664`，应为 `7562-7564`（fixture_frames.rs 与 commit 正确）。
- 判定：🟡（核心钉死面绿；注记面 3 处 stale/笔误）。

## 3. 两账清——🔴（账②销账面不实）

### 3.1 账①：措辞排除面四类全列——🟢

- 四类=史料（findings/reviews）/哈希（金样 sha 碎）/原地注记（更正留痕）/伪命中（哈希巧合串）：
  arm_coverage.rs 头注、port-notes-anchor-debt.md、commit 三处一致全列 ✓（措辞纪律"实测边界随附排除口径"达标）。
- 代表复跑（grep 计数分面）：`saved_floor_states` repo 级命中 = crates 真锚 15 / findings 史料 3 / 锚账面 4——分面口径与排除面吻合。小瑕：未钉具体 grep 命令（可复现性靠口径词而非命令）。

### 3.2 账②：⚠ 17→15（销 3=rename 现形？）——🔴

实测（旧工具 bfc8712^ 重跑 stdout「⚠ 17 条校验告警」→ 新「⚠ 15 条」）：

| 族 | 旧 | 新 | 销 |
|---|---|---|---|
| PORT_NOTES | 10 | 8 | 2 |
| TRIAGE | 2 | 2 | 0 |
| YELLOW_NOTES | 4 | 4 | 0 |
| ARM_GAPS | 1 | 1 | 0 |
| 合计 | **17** | **15** | **2** |

- 17→15 ✓ 实测吻合（含与批1 round3 复核件点名数对上——round3 同载 17/15 两数）。
- **"销 3"不实 ⇒ 红**：实销 **2**——
  1. addItemToItemStack ✓ rename 现形实证：`scene.rs:4244/4335 self.menu.set_overlay_kind_item(t)`；
  2. initFloorStates clone-assign ✓：`scene.rs:599 scene.persist.saved_floor_states = save.floor_states.clone()`；
  3. **debt 行 3（`saved_floor_states: vec![None; 56]`）标"销"与产物矛盾**：重生成矩阵仍输出 `⚠ PORT_NOTES[initFloorStates] 锚失效：…saved_floor_states: vec![None; 56]`（matrix L17）——锚仍指 scene.rs，字面量已迁 scene_persist.rs:75（grep 实证），本批只加了括注"现居 scene_persist.rs"未改锚 ⇒ 警告未消。
- **算术自相矛盾**：销 3 ⇒ 17-3=14≠15；17→15 恰为销 2 才自洽——commit 与 debt 表的"销 3/登 15"两数不可能同时成立。

### 3.3 15 预存债入档（点名数吻合）——🟡

- debt doc 入档 ✓、总账 17 行 ✓、"复核件点名数"对上 ✓（round3 复核件点名 17/15 两数）。
- 但：①登记面实名仅 14 条（行 4-10 + 行 11-17 折叠"余 7 条"），与"登 15"差 1——差的正是仍失效却标销的行 3；②行 11-17 标"同族多锚条目"实为 TRIAGE(2)/YELLOW_NOTES(4)/ARM_GAPS(1) 异族，表头"PORT_NOTES 锚失效清单"名不副实。

### 3.4 判定

账①🟢；账②🔴（销 3 不实、算术矛盾）+🟡（登记面折叠 14≠15、族标不准）。

## 4. 金样 64——🟢

- **61+3 纯追加（旧行零动）** ✓：numstat `15  0`（零删除）；scenarios 键序 sorted ✓；插位 fx-knockback-floor43…fx-record-arrows 之间（字母序正确）；旧行字节零动。
- **--check 64 全同实跑** ✓：`python3 tools/state_baseline.py --check` →
  `[GREEN] 金样逐字节等价：64 场景 sha 全同`（exit 0）。
- 判定：🟢。

## 5. 夹带检查——🟡

- **--stat 仅本批面** ✓：15 文件 = 3×3 夹具（.frames.txt/.preset/.txt）+ fixture_frames.rs + arm_coverage.rs + state-dump-golden.json + coverage-matrix.md + port-notes-anchor-debt.md + cover_matrix.py，无夹带；cover_matrix.py 重生成幂等（跑后 git status 干净，矩阵=生成器真输出）。
- **批 1 绿项未动坏** ✓：fixture_frames 全套 35/35 过（含 `fx_item13_pickup_stack_arm_frames_match`）；金样旧行零动。
- **红证据三件套（缺臂清单递减）——🟡**：实测现缺 **11**（ARM_FIXTURES 21 组 − 已有 10 = 11：pickup-29/30/31/32/33/37/default + use-13/preset-item21/use-85/use-default）。递减链=TDD 红 15 →批1 销 1（fx-item13）→批2 销 3 ⇒ **15→11**；attack 期望的"15→12"口径未落任何产物，arm_coverage.rs 头注仍停"批1 时点缺臂 15"、本批递减数**未入账**。test 本体动态查存在性（不会假绿），属红清单记账不全。
- 判定：🟡（清单递减未落账；无夹带、无回退）。

## 6. 总判

| 项 | 判 |
|---|---|
| 1 三臂真伪 | 🟡（夹具/对拍/三方一致🟢；26 臂 commit 金样身位🔴） |
| 2 实证注记 | 🟡（yk 钉死🟢；26.txt 断言注记 stale、28.txt 行锚笔误、历史叙事无留档） |
| 3 两账清 | 🔴（销 3 不实=实销 2、算术矛盾）+🟡（登记面 14≠15、族标不准） |
| 4 金样 64 | 🟢 |
| 5 夹带检查 | 🟡（缺臂递减 15→11 未落账） |

- **红项（2）**：
  1. §1.2 commit"236/320/320 帧链 653a5008…"中 26 臂身位与产物（320/92f980cc…）不符，Evidence"3 链 sha"1/3 不实；
  2. §3.2 账②"销 3"不实（实销 2；debt 行 3 仍失效却标销；17→15 与销 3 算术互斥）。
- **黄项（4）**：fx-pickup-26.txt 断言注记 0→1 stale；fx-pickup-28.txt 行锚 7562-7664 笔误；debt 登记面折叠 14≠15+异族标同族；arm_coverage 缺臂递减 15→11 未落账。
- **绿项**：a.java:7556-7564 逐字对拍、A/B/C 三方逐字节一致（三臂）、夹具=A trace 抽取独立重算全等、27/28 链 sha、yk=0 显式+断言钉 2+全帧比、金样 61+3 纯追加+`--check [GREEN] 64 全同`（实跑）、--stat 干净+矩阵重生成幂等、fx-item13 未动坏、fixture_frames 35/35。

**总判：需修复**——夹具与金样本体不动；红 2 + 黄 4 全部是措辞/账面问题，建议 **amend**（未推送）：改 commit 26 臂金样身位（320/92f980cc…）、"销 3"改"销 2/登 15（行 3 归登记）"、26.txt 断言行 0→2、28.txt 行锚 7562-7564、debt 表行 3 改登记+补实名、arm_coverage 头注补"批2 后缺臂 11"。修复后可推送。

## 附：实测命令留痕

```
python3 gates/cli.py check                                 → PASS=13 FAIL=0 SKIP=0（全绿，commit 自述"全绿"属实；
                                                             test-ratchet 报 935(基线929) 无回退，arm_coverage(test) 待 ratchet-update=新增待记）
cargo test -p game-oracle --test fixture_frames            → 35 passed, 0 failed
cargo test -p game-oracle --test fixture_frames fx_pickup  → 3 passed（26/27/28）
cargo test -p game-oracle --test arm_coverage              → 1 ignored（P1-a2 进行中）
python3 tools/state_baseline.py --check                    → [GREEN] 金样逐字节等价：64 场景 sha 全同
python3 tools/cover_matrix.py（旧/新）                     → ⚠ 17 条 / ⚠ 15 条校验告警
帧链独立重算：26=92f980cc…(320帧) / 27=ea781bbac… / 28=a03d0fa0…；A=B=C trace sha 三臂一致
git diff --numstat bfc8712^ bfc8712 -- state-dump-golden.json → 15 0（纯追加）
sed -n '7556,7564p' reference/src/deobf/a.java             → case26/27/28 三条 key++ 逐字
grep 实证：scene.rs:4244 set_overlay_kind_item(t)、scene.rs:599 persist.saved_floor_states、
          scene_persist.rs:75 saved_floor_states: vec![None; 56]（行 3 锚仍失效）
缺臂实算：21 组 − 10 已有 = 11 缺（pickup-29/30/31/32/33/37/default、use-13、preset-item21、use-85、use-default）
```
