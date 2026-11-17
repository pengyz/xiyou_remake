# 批6-① 复核：ada1ab5 红2 闭环=断言自校正（夹具期望误读证伪）

> 角色：独立对抗复核员（逐提交纪律·差分验证员纪律，AGENTS.md 权威）。
> 对象：commit `ada1ab5`（未推送，绿=lead 推送）。日期：2026-11-04。
> 状态：骨架（各节待逐项取证回填）。

## 0. 摘要与总判

- **对象**：`ada1ab5`（父=`7daa642`，未推送；`git branch -r --contains` 空=绿即 lead 推送）。
- **攻击点结果**：证伪链实证 **绿**、断言修正红线 **绿**、ignore/红注同步 **绿**（附 2 黄）、金样零冻 **绿**、夹带净+三纪律 **绿**。
- **发现计数**：红 **0** / 黄 **4**（全部=注释与措辞精度，零行为影响）/ 绿 5；另有 1 项父件遗留观察（非本 commit 引入）。
- **总判：可推送**。核心红线（"改断言=改考题？"）经独立取证证伪——Java 侧终态实测=Map(3)，原断言 EnemyInfo 才是错的一方。

| # | 攻击点 | 裁定 |
|---|---|---|
| 1 | 证伪链实证（双侧时间线零分歧+第三拍=关闭） | 🟢 绿（附黄：对拍分母措辞） |
| 2 | 断言修正红线（是否迁就实现） | 🟢 绿 |
| 3 | ignore/红注同步（红1 历史注记保留） | 🟢 绿（附黄×2） |
| 4 | 金样零冻 state_baseline --check 73 | 🟢 绿 |
| 5 | 夹带净 + 三纪律贴件逐字 | 🟢 绿 |

## 1. 攻击点1：证伪链实证（双侧 gameMode 时间线零分歧）

**裁定：🟢 绿**（附黄 Y4：对拍分母措辞）。

### 1.1 Java 侧独立取证（不信任 commit 贴件，全部复跑）

A-run trace 终态与时间线（`python3 reference/oracle/fields.py --timeline analysis/frames-fixture/runs/A-fx-use-13/trace.txt gameMode`）：

```
# tick | gameMode
    8 | 0   70 | 21  72 | 1  152 | 8  202 | 2
  240 | 3   302 | 10 342 | 5  382 | 3
（末态：FLD010 @tick480 = 3 = Map；tail 连续 3）
```

trace 本体可信度：夹具头钉 `sha256=631e1ba7…`，与盘上文件 `sha256sum` **逐字节一致**；A/B/C 三跑 trace 三方同 sha（`631e1ba7…`）——帧链源自批5（`844cf39`），**早于本 commit，未被本 commit 触碰**（`git diff 844cf39 ada1ab5 -- fx-use-13.frames.txt` 空）。

### 1.2 Rust 侧实测（STATE_OUT 复跑提取）

`STATE_OUT=/tmp/b6a-state cargo test --release -p game-oracle --test fixture_frames fx_use_13` → `fx-use-13.state.txt`（243 个 TICK 块）提取转移：

```
tick 238: None→3   （scene 建立，dump 起点）
tick 301: 3→10     （300 拍 -6 按下，次拍生效）
tick 341: 10→5     （340 拍 -6 → activateItem case13）
tick 381: 5→3      （380 拍 -6 → 关闭）
末态 tick480 = 3
```

### 1.3 双侧对拍（独立复算，非引用 commit 数字）

| 项 | 值 |
|---|---|
| Java FLD010 采样点 | 71（stride=8 + 事件拍） |
| Rust STATE_OUT dump 点 | 243 |
| **共有 tick** | **37** |
| **分歧 tick 数** | **0** |
| 双侧末态 | (480, 3) == (480, 3) ✅ |

**转移括夹一致性**（Rust 精确转移落 Java 相邻采样括夹内）：

```
Rust@301 ∈ (300,302)  Java 3→10   ✅
Rust@341 ∈ (340,342)  Java 10→5   ✅
Rust@381 ∈ (380,382)  Java 5→3    ✅
```

**帧链补足逐拍面**：`fx-use-13.frames.txt` 480 帧 = Java trace FRAME sha 逐 tick 抽取（本次独立比对：Java trace 帧 480 vs 夹具帧 480，**不符=0**）；测试断言 `checked==records.len` 全帧比对通过 ⇒ 每一拍视觉态双侧一致。若 Java 在 301/341/381 任一拍与 Rust 差一个 mode（Map/面板渲染迥异），帧 sha 必裂——**帧链=逐拍 mode 等价的兜底证明**。

### 1.4 "第三拍=关闭"输入语义（a.java 锚 sed 实文，awk 逐行取证）

```
$ awk 'NR>=3551&&NR<=3561' reference/src/deobf/a.java   ← mode 3（Map）
3559:  case -6:
3560:     var26.gameMode = 10;            ← 300 拍：Map→道具面板

$ awk 'NR>=3931&&NR<=3946' reference/src/deobf/a.java   ← mode 10（ItemPanel）
3937:  case -6:  case -5:
3942:     case 13:  case 14:
3944:        var21.gameMode = 3;
3945:        var21.activateItem(...);     ← 340 拍：activateItem case13 ⇒ mode 5

$ awk 'NR>=3729&&NR<=3739' reference/src/deobf/a.java   ← mode 5（EnemyInfo）
3732:  case -7:  case -6:
3734:     var24.gameMode = 3;             ← 380 拍（第三拍）：-6 = 关闭⇒Map，非确认
3735:  case -5: … default: break;         （落入 break，无 activateItem）

$ awk 'NR>=7790&&NR<=7797' reference/src/deobf/a.java
7794:  case 13:  this.gameMode = 5; this.dispatchModeEntry(); break;
```

三拍语义链闭合：`300 -6 ⇒10`（开面板）→ `340 -6 ⇒ activateItem case13 ⇒ 5`（开 EnemyInfo）→ `380 -6 ⇒ 3`（**关闭**）。原夹具 cover 注"380 -6 确认 ⇒ activateItem case13"两处皆误（activateItem 在 340 拍、380 拍是关闭）——**误读实锤**。

### 1.5 黄 Y4（措辞）

commit 称"逐 tick 对拍"，实测分母=**37 个共有采样点**（Java FLD010 stride=8，非 480 全点）；480 全拍等价由帧链承担（§1.3）。结论不受影响，"逐 tick"宜注明覆盖口径。

## 2. 攻击点2：断言修正红线（改断言≠改考题）

**裁定：🟢 绿**。红线判据：*若 Java 终态真是 EnemyInfo 而 Rust 是 Map，改断言=改考题迁就实现*。独立取证结论——**Java 终态实测=Map(3)**，三方一致：

1. **动态**：A-run trace FLD010 @480 = 3（§1.1，trace 源自批5 早于本 commit）；
2. **静态**：a.java:3733-3734 mode5 `-6 ⇒ gameMode=3`，第三拍必关（§1.4）；
3. **帧面**：tick381–480 帧由 Rust（Map 态）与 Java 参照帧逐帧相等（测试全过）⇒ Java 末段渲染即 Map。

故原断言 `GameMode::EnemyInfo` **与 Java 参考版自身的 A-run trace 相矛盾**——错的是断言（把 a.java:7794-7797 的"开臂"静态误读为终态），不是 Rust。改 `GameMode::Map` = 断言向参考版对齐，**非迁就实现**。

反向印证：同批 `fx_use_default_noop` 断言仍钉 `GameMode::ItemPanel`（型23 no-op 不关面板）——修正是**逐夹具**的，非 blanket 放宽；EnemyInfo 臂面（341–380 帧）仍由 480 全帧断言覆盖，未丢失检验面。

## 3. 攻击点3：ignore/红注同步

**裁定：🟢 绿**（附黄 Y3）。

- `#[ignore = "RED2: …"]` 已摘除 ✅；复跑 `fx_use` → `3 passed; 0 failed; 0 ignored`（在册运行）✅。
- 红2 注改闭环 ✅：新增`【2026-11-04 红2 闭环=断言自校正】`块（含零分歧实测+第三拍关闭语义+原断言自误定性），尾行"红2 登记后回 ignore；红2 修复+复核后解"→"红2 闭环（断言自校正）后解 ignore=本用例在册运行"。
- **红1（clip）历史注记保留** ✅：`【2026-11-04 红1闭环+红2 登记】`整块（fw=35→32、a.java:2546、scene_menu.rs:611、帧比对全过）在 diff 中为**上下文未动**。
- 红2 原登记主张（"Rust Map vs fixture/Java=EnemyInfo"）**未删除**，按 AGENTS §3.6 证伪更正文化由上方带日期新注原地标注推翻——历史主张可见 ✅。
- 🟡 **Y3（注序）**：红2 闭环新注插在红1 闭环旧注**之前**（时间序 844cf39→7daa642→ada1ab5 被排成 待裁→闭环→登记），阅读时先见闭环后见登记，建议后续顺手下沉至红1 注之后（纯可读性，不阻断）。
- 观察（父件遗留，**非本 commit 引入**）：`【RED·待裁】`尾句"——非测试形"悬空断句，系 `7daa642` 截断原文所致，本 commit 未恶化。

## 4. 攻击点4：金样零冻（state_baseline --check 73 全同）

**裁定：🟢 绿**。独立实跑（非引用贴件）：

```
$ python3 tools/state_baseline.py --check
[GREEN] 金样逐字节等价：73 场景 sha 全同          （exit 0）
```

总闸复核（本 commit 树 HEAD=ada1ab5）：

```
$ python3 gates/cli.py check
[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：73 场景 sha 全同
[✓ PASS] rust-tests — cargo test --release 全绿
…
== 汇总 == PASS=13 FAIL=0 SKIP=0
```

本 commit 仅动测试断言/注释/夹具注（2 文件，零 runtime 代码），state 面本应不变——但未止于推理，**实跑 73 sha 全同**坐实。金样零冻 ✅。

## 5. 攻击点5：夹带净 + 三纪律贴件逐字

**裁定：🟢 绿**。

- **夹带净**：`git show ada1ab5 --stat` = 恰 2 文件（`fixture_frames.rs` +10/-4、`fx-use-13.txt` +6/-2），与声称一致；工作区未跟踪件（`android/build/`、批5 复核件）**未入 commit** ✅。
- **三纪律贴件复跑逐字核对**（在提交树实跑，非手粘）：

| commit 贴件 | 本次复跑 | 核对 |
|---|---|---|
| `3 passed; 0 failed; 0 ignored; 0 measured; 41 filtered out` | 同左逐字 | ✅ |
| `[GREEN] 金样逐字节等价：73 场景 sha 全同` | 同左逐字 | ✅ |
| `分歧 tick 数=0  Rust 末态=(480, '3')  Java 末态=(480, '3')` | 独立复算=0 / (480,3)/(480,3) | ✅ |
| `tick 238: None→3 / 301: 3→10 / 341: 10→5 / 381: 5→3` | STATE_OUT 提取四行全同 | ✅ |

- **终态贴**：全部命令于 HEAD=ada1ab5 执行（提交树终态），无过渡态贴 ✅；**禁手粘**：贴件格式=命令输出原样（含 `=== … ===` 分节头）✅。
- commit message 含 `Evidence:`+`Confidence: 95`，格式合 §9 ✅（本 commit 未触及 data/naming/reference/original，证据门禁非必需但已备）。

## 6. 发现清单（红/黄/绿逐项）

| # | 级 | 发现 | 影响 | 处置建议 |
|---|---|---|---|---|
| — | 🔴 红 | **无** | — | — |
| Y1 | 🟡 | 夹具注重复："300 -6 面板 → **340 -6 选中 → 340 -6 选中**⇒activateItem"（编辑残留，读作 300→340→340） | 纯注释，零行为 | 后续顺手去重（删前一个"340 -6 选中 →"） |
| Y2 | 🟡 | "341-**381** 帧区/帧面"端点歧义：实测 mode5=**341–380（40 帧）**，tick381 dump=3 且帧=Map；夹具注+断言消息**两处**同写 | 纯注释/消息，零行为 | 改 341–380 或写半开区间 [341,381) |
| Y3 | 🟡 | 红2 闭环新注插于红1 闭环旧注**之前**，注释时间序倒置 | 可读性 | 后续下沉至红1 注后 |
| Y4 | 🟡 | "逐 tick 对拍"实际分母=37 共有采样点（Java FLD010 stride=8）；480 全拍由帧链承担 | 措辞精度，结论不受影响 | 补注覆盖口径 |
| 观察 | ⚪ | `【RED·待裁】`尾句"——非测试形"悬空（`7daa642` 截断遗留） | 可读性 | 非本 commit 引入，随 Y3 一并顺手修 |

**绿项**：① 证伪链实证（37 点零分歧+转移括夹一致+480 帧链+四枚 a.java 锚）；② 断言改 Map 三方独立实证（trace/静态/帧面），非改考题；③ ignore 摘除+红2 闭环注+红1 历史注记完整保留；④ `state_baseline --check` 73 全同+总闸 PASS=13/FAIL=0；⑤ 夹带恰 2 文件+三贴件复跑逐字+终态贴。

## 7. 总判

**可推送**（绿）。红线问题"改断言=改考题？"被独立取证证伪：Java 参考版自身 A-run trace 终态=Map(3)，原断言 EnemyInfo 才是与参考版矛盾的一方；证伪链（双侧时间线 37 共有点零分歧、转移括夹一致、480 帧链全过、四枚 a.java 输入语义锚）完整闭合；金样 73 全同、总闸 13 PASS、夹带净、贴件三纪律复跑逐字吻合。4 项黄全部为注释/措辞精度（Y1 夹具注双写、Y2 端点 381 歧义、Y3 注序、Y4 对拍分母），零行为影响，可随批6-②顺手清，不阻断推送。
