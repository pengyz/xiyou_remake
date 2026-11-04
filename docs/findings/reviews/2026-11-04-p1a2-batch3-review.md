# DDDSA P1-a2 批3 独立对抗复核（逐提交·差分验证员纪律）

> 复核对象：commit `7166569`（test(oracle): DDDSA P1-a2 批3——pickup 29/30/31 三臂夹具+措辞 2 黄清）
> 状态：**未推送**（绿 = lead 可推送）
> 复核员：独立对抗复核员（逐提交纪律 / 差分验证员纪律，AGENTS.md 权威）
> 复核日期：2026-11-04
> 前情：批 2 闭环（命令原文贴纪律生效）；本次为批 3 复核。

## 复核方法

- 逐攻击点红/黄/绿裁定（红=可证伪缺陷 / 黄=口径·完备性不足不阻断 / 绿=实测通过）；总判二值：可推送 / 需修复。
- 一切结论以**实跑命令输出**为据（不复述自述，只认机器输出）。
- 独立重算：不采信 commit message 自报值，全部从工作区/提交内容重算对拍。
- 逐字核对：命令全文/输出原文逐字比对，禁省写/截断/口径词。

## 0. 提交面概览

`git show --numstat 7166569` 实测 12 文件：

- 3×(fx-pickup-29/30/31 的 .frames.txt 324 行 + .preset 3 行 + .txt 19 行) = 9 个新夹具件（纯新增）；
- `crates/game-oracle/tests/fixture_frames.rs` +34/−0（纯追加 3 测，插于 fx_pickup_28 与 FX-ITEM-18A 之间，既有测试零改动）；
- `data/status/baselines/state-dump-golden.json` +15/−0；
- `docs/findings/2026-11-04-port-notes-anchor-debt.md` +3/−3（仅头族分更正）。

无夹带（详见 §5.3）。

## 1. 攻击点一：三臂真伪（29/30/31 与 a.java:7565-7583 对拍）——🔴（行为面绿；case 30 行锚错引=可证伪）

### 1.1 三臂语义与 Java 原文对拍

`reference/src/deobf/a.java` 逐行实核（`awk 'NR>=7565 && NR<=7587'`）：

| 行 | 真源（deobf a.java） | 本笔引用 | 夹具断言 | 判 |
|---|---|---|---|---|
| 7565-7568 | `case 29:` / `if (this.currentFloor <= 10) {` / `this.playerAtk++;` / `return;` | fx-pickup-29「a.java:7565-7568 playerAtk++（floor≤10）」 | `sc.player.atk == 201` | ✅ 引文与断言逐字吻合 |
| 7569-7572 | `}` / 空行 / `this.playerAtk = this.playerAtk + this.floorTier();` / `return;`（**case 29 的 floorTier 尾臂**） | fx-pickup-30「a.java:7569-7572 playerDef++（floor≤10）」 | `sc.player.def == 31` | ❌ **错引**：所引 4 行是 case 29 尾臂（playerAtk+floorTier），非 case 30 |
| 7573-7580 | `case 30:` / `if (this.currentFloor <= 10) {` / `this.playerDef++;` / `return;` / `}` / 空行 / `this.playerDef = ... + this.floorTier();` / `return;`（**case 30 真身**） | （未引） | — | 应引区间 |
| 7581-7583 | `case 31:` / `var2 = this.floorTier();` / `this.applyHpDelta(50 * var2);` | fx-pickup-31「a.java:7581-7583 applyHpDelta(50*floorTier)」 | `sc.player.hp == 550` | ✅ 吻合 |

- 语义面：29=playerAtk++（≤10）/ 30=playerDef++（≤10）/ 31=50×floorTier() ——与 Java 逐字相符 ✅；
- **错引面（🔴）**：case 30 的行锚 7569-7572 在 commit message、fx-pickup-30.txt、fixture_frames.rs 文档注+断言消息**三处同错**，全库无一处给 case 30 正确锚（真身 7573-7580，playerDef++ 在 7575）。commit message Evidence 行「a.java:7565-7583 逐字」的总区间无误，但正文逐臂映射「7565-7568/7569-7572/7581-7583」中段不实——"逐字对拍"主张对 30 臂**可证伪**（批 2 先例：Evidence 行含不实数 = 🔴；且批 2「行锚笔误」判 🟡 的前提是正确锚在别处存在——本笔不存在，重于该先例）。
- 小注：fx-pickup-29.txt 引 7565-7568 覆盖 ≤10 臂正确，但 fixture_frames.rs 文档注写「playerAtk++/floorTier」而锚未含 floorTier 尾臂（7569-7572）——引文范围窄于主张，随 30 臂错引一并更正即可。

### 1.2 floor≤10 vs floorTier() 分支钉（真源 7566-7580）+ floor12 实测——🟡（仅 ≤10 臂被夹具钉死）

- 三臂夹具均 floor3（≤10）：case 29/30 两分支在 floor3 **不可区分**（+1 与 +floorTier()=+1 同值）；case 31 亦然（50×tier，tier(3)=1）。夹具钉死的是 floor3 效应值，**分支判别与 tier 倍率无夹具覆盖**——floor>10 臂（29/30 的 +floorTier()、31 的 50×tier）在批 3 夹具面**未钉**。
- tier 语义锚（攻击点所指）实存：`crates/game-core/src/scene.rs:4257-4260`「29/30：floor ≤ 10 +1，否则 +floorTier()；31/32：50/200 × floorTier()……**floor12 实测 +50 应为 +100**」；实证记录在 `docs/spec/completeness.md:689`（迭代 31 真缺口：「拾取 floorTier 缺失（type29/30/31/32 层域倍率，floor12 +50 应 +100 实证）」）——**史录级锚，无 trace id/夹具留档**（可再生区的 floor12 实测未固化为夹具）。
- 实现面 `scene.rs:4261-4268`（29/30 分支 + 31/32 倍率）与 `floor_tier()`（scene.rs:4231-4239 ← a.java:7651-7662 语义（floor-1)/10 钳负、>50 归 0、+1）逐字相符 ✅。
- 附带发现（**本笔之外**的预存债）：scene.rs:4257-4259 自带行锚「a.java:7556-7567 / 7568-7577 / 7642-7652」三处均漂移（真源 7565-7580 / 7581-7587 / 7651-7662）——攻击点中「a.java:7568-7577」恰与此漂移注一致；建议随锚债账处置，不计入本笔裁定。

### 1.3 实测值钉 201/31/550 与场景链对拍——🟢

- 语义复算：floor3 → floorTier=(3-1)/10+1=1；29: 200→**201**（≤10 臂 +1）；30: 30→**31**；31: 500+50×1→**550**——与 Java 逐字一致 ✅。
- preset 基线实核：`python3 tools/frames_fixture.py preset floor=3 x=1 y=10 facing=1 hp=500 atk=200 def=30 sky=5`（30/31 同参改 x,y）重生成 .preset **逐字节一致 ×3**（hp=0x1f4=500/atk=0xc8=200/def=0x1e=30 实测在 MOT_L0 内）。
- 断言逐字钉：`fixture_frames.rs` 三测 assert_eq!（201/31/550 + 终态 (1,9)/(2,1)/(2,2)）实存；全帧比对 Java 参照（320 帧逐 sha）+ 金样 state dump 入冻结面（§5）——行为面由差分裁判钉死。

### 1.4 裁定

行为/语义/实测值 🟢；**case 30 行锚错引 🔴**（三处同错、无正确锚、"逐字"主张可证伪）；分支完备性 🟡（floor>10 臂无夹具，tier 语义靠 scene.rs 注释锚+completeness.md 史录）。

## 2. 攻击点二：链 sha 独立重算（7d113718/0fd72475/5f312e69）——🟢

### 2.1 三链逐位复算（命令原文口径 + 独立口径双复算）

| 臂 | commit 声称 | 消息命令复跑（逐字节同输出） | A-trace 独立重抽取 | 判 |
|---|---|---|---|---|
| 29 | 320 帧链 7d1137189486191f | frames=320 chain=7d1137189486191f | 320/320 body 全等 → 7d1137189486191f | ✅ |
| 30 | 320 帧链 0fd72475f22b9591 | frames=320 chain=0fd72475f22b9591 | 320/320 body 全等 → 0fd72475f22b9591 | ✅ |
| 31 | 320 帧链 5f312e6921351db6 | frames=320 chain=5f312e6921351db6 | 320/320 body 全等 → 5f312e6921351db6 | ✅ |

- 头注全链 sha（7d113718…b25 / 0fd72475…3d0 / 5f312e69…418）与 16 位截断值一致；`frame_chain=sha256("\n".join(shas))` 口径与 tools/frames_fixture.py `frame_chain()` 同式实核 ✅。

### 2.2 frames_fixture 重生成对拍 + 三方一致

- `python3 tools/frames_fixture.py gen <stem>.txt` ×3（重跑 Java oracle A/B/C 三变体）：三方 trace sha 实测 A=B=C（29: a1a349ca… / 30: 4473d219… / 31: 996fcd89…），与各夹具头注 sha 逐字吻合；重生成 .frames.txt 与入库件**逐字节一致 ×3**（`diff -q` 无差，git status 干净）——抽取无手改 ✅。
- `frames_fixture.py check` ×3 输出「夹具自洽」（TICK 连续 + INPUT 列=脚本事件）✅。
- 附注：夹具头注「帧链 sha256=…（= tools/method_reach 快照 frame_chain 口径）」中"口径"二字使用准确（公式同式，非声称快照内含此链——快照未重测见 §5.4）。

### 2.3 裁定

🟢（三链逐位复算全中 + 重生成逐字节复现 + 三方一致实证）。

## 3. 攻击点三：Q-1 措辞纪律执行——🟢（一处注记面小瑕随 §1 错引处置）

### 3.1 命令全文+输出原文逐字核

| message 贴件 | 复跑实测 | 判 |
|---|---|---|
| 链 sha for-loop 命令块（python -c 全文）+3 行输出 | 逐字节同输出（frames=320 chain=7d11…/0fd7…/5f31…） | ✅ 全文可执行、输出全同 |
| `cargo test -p game-oracle --test fixture_frames fx_pickup …\| grep "test result"` + 输出行 | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 32 filtered out; finished in 3.05s`（message=3.13s，仅计时差） | ✅ 结果字段全同 |
| `python3 tools/state_baseline.py --check …\| tail -1` + `[GREEN] 金样逐字节等价：67 场景 sha 全同` | 实跑输出与贴件**逐字节一致**（§5.2） | ✅ 全同 |

- 无省写/截断：两代码块命令完整（含 `2>&1 | grep`/`| tail -1` 管道原样）、输出无「…」省略；grep/tail 是命令自身口径，输出=该命令实际打印，可复现 ✅。
- 口径词：显式声明「"全库仅存"类口径本笔不使用——范围声称随 grep 实测边界」✅；「三测一次全绿」「金样=A/B/C 三方一致抽取（320 帧×3）」均为实测边界内事实（已独立验证）✅。
- 小瑕（🟡 注记面）：「位点探针实证（sprite3 t29 (1,9)/t30 (2,1)/t31 (2,2)）」称"实证"但探针输出未留档（runs/ 目录仅 trace/stdout/stderr/run-info，无探针产物）——按批 2 先例属"注记面不可独立复算"（🟡）；行为面已由全帧比对+终态断言+金样钉死，坐标主张获行为佐证。

### 3.2 裁定

🟢（Q-1 纪律执行到位：命令原文贴逐字可复现、无口径词）；探针注记 🟡 随账。

## 4. 攻击点四：Q-2 清（debt 族分自洽 + TRIAGE 第 2 条拆出实证）——🟢

### 4.1 族分 8/1/4/1=14+1 自洽（机械复核）

`docs/findings/2026-11-04-port-notes-anchor-debt.md` 登记面 14 行逐行点数：PORT_NOTES 行 1-8=**8** ✅ / TRIAGE 行 9=**1** ✅ / YELLOW_NOTES 行 10-13=**4** ✅ / ARM_GAPS 行 14=**1** ✅；另类 1 条单列 ✅。头注算术「原 ⚠ 17 = 销 2 + 登 14 + 1」=17 ✅。
对拍生成器真源：`python3 tools/cover_matrix.py` 重跑输出「⚠ 15 条校验告警」，嵌入清单（docs/coverage-matrix.md:15-29）逐条对账——PORT_NOTES×8 / TRIAGE[saveGlobalProgress]×1 / TRIAGE「修正暴露」×1（即另类）/ YELLOW_NOTES[m_107]×4 / ARM_GAPS×1，**与登记面一一对应** ✅。原「7/2 vs 8/1 不自洽」确证（旧头 7+2+4+1=14 与括注"共 8 PORT_NOTES"矛盾），更正后自洽 ✅。

### 4.2 TRIAGE 第 2 条拆出实证

- 拆出对象 = coverage-matrix.md:24「⚠ TRIAGE「修正暴露」与机械复算不符：漏登 ['m_101','m_134','showNotify']；多登 ['closeRecordStoreAndGc','deleteRecordStoreQuietly']」（生成器机械复算输出原文，与 debt 另类节内容逐字同）。
- 分类实证：该警告性质为**类成员清单不符**（漏登/多登名单），非「锚失效」（grep 字面锚失配）——归"另类（校验器口径差待裁）"成立；14 登记面全部为「锚失效」字样条目 ✅。异类分账与生成器两类警告形态一致。

### 4.3 裁定

🟢（族分机械自洽 + 拆分有生成器原文实证）。

## 5. 攻击点五：金样 67（纯追加 / --check 实跑 / 夹带净 / 批 1·2 未动坏）

### 5.1 64+3 纯追加——🟢（机械核）

`state-dump-golden.json` 顶层 scenarios 词典：旧 **64** → 新 **67**；added=**[fx-pickup-29.state, fx-pickup-30.state, fx-pickup-31.state]**、removed=**[]**、既有 64 条 changed=**[]**（python json 逐值对拍）。numstat +15/−0 ✅。

### 5.2 --check 实跑（state_baseline）——🟢

实跑原文：

```
$ python3 tools/state_baseline.py --check 2>&1 | tail -1
[GREEN] 金样逐字节等价：67 场景 sha 全同
```

与 commit message 贴件**逐字节一致** ✅（67=64+3 新臂入冻结面，逐字节全同）。

### 5.3 夹带净——🟢

- numstat 实测：11 文件纯新增 + debt doc 3/3（仅头族分更正）；fixture_frames.rs **+34/−0**（既有 38 测零改动，新 3 测插于 fx_pickup_28 后）。
- debt doc 改动仅头注族分更正段，无他内容夹带 ✅；golden 仅 3 条新增 ✅。无任何非主张面文件。

### 5.4 批 1/2 绿项（fx-item13/26/27/28）未动坏——🟢（一处簿记债 🟡）

- 文件面：批 1/2 夹具件不在本笔 diff ✅；
- 实跑面：`cargo test -p game-oracle --test fixture_frames` 全套 **38 passed / 0 failed**（含 fx_item13/26/27/28/29/30/31）✅；
- 簿记面（🟡）：批 1/2 均随 commit 重生成 docs/coverage-matrix.md（f6f5520 实测含 fx_pickup_26/27/28 行），**本笔未重生成**——`tools/cover_matrix.py --check` 实测报「账面不一致」，重生成 diff = 3 新测行（T1-None 无触达快照）+fixture_frames.rs 引用行号 +34 平移，无任何行删除、批 1/2 行原样 ✅。cover_matrix --check 不在 `gates/cli.py check` 热路径（cmd_check 清单实核无此项），不阻断"gates check 全绿"主张；但 method_reach 触达快照亦未随新场景重测（`先决：…场景…变化后重测`）——建议下笔随附 ①重跑 `tools/method_reach.py` ②重生成矩阵，与批 1/2 簿记实践对齐。

### 5.5 gates check 全绿——🟢

实跑原文（汇总面）：

```
$ python3 gates/cli.py check 2>&1 | tail -15
[✓ PASS] docs-links — 全部相对链接有效
[✓ PASS] naming-ledger — 全部记录带证据指针
[✓ PASS] rust-tests — cargo test --release 全绿
[✓ PASS] test-ratchet — total=938(基线929) 无回退；新增二进制待 ratchet-update: {'arm_coverage(test)': 1}
[✓ PASS] table-content-hash — 保真三表 sha256 对照基线通过（逐值锚 220 宿主；基线 table_content_hash.json）
[✓ PASS] table-evidence — 机械提取值（等值）
[check] ✓ anim_defs.json 复跑1=99a92f78e59fe530 复跑2=99a92f78e59fe530 盘上=99a92f78e59fe530
[check] ✓ sheet_index.json 复跑1=a19c815acb131d26 复跑2=a19c815acb131d26 盘上=a19c815acb131d26
[check] ✓ sprite_defs.json 复跑1=7f5e2e80e146c264 复跑2=7f5e2e80e146c264 盘上=7f5e2e80e146c264
[check] ✓ 确定性复跑同哈希 + 等值全绿
[✓ PASS] state-machine-anchors — 锚 6 + 边数对照；全命中
[✓ PASS] asset-truth-manifest — 真相源清单校验 PASS（entries=138，art 三态核通过）
[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：67 场景 sha 全同
== 汇总 ==
PASS=13 FAIL=0 SKIP=0
```

「gates check 全绿」主张成立 ✅（13 PASS / 0 FAIL / 0 SKIP，SKIP 显式打印=0）。附注：test-ratchet 打印「新增二进制待 ratchet-update: arm_coverage(test)」——ratchet 只升不降无回退（938>基线 929），待更新基线属既定流程非缺陷。

## 6. 总判

| 攻击点 | 判 | 要点 |
|---|---|---|
| 1 三臂真伪 | 🔴（行为面 🟢） | case 30 行锚 7569-7572 错引（真身 7573-7580），commit message / fx-pickup-30.txt / fixture_frames.rs 三处同错，"逐字对拍"可证伪；语义/实测值 201/31/550 全对 |
| 2 链 sha 重算 | 🟢 | 三链逐位复算全中；gen 重生成逐字节一致 ×3；A=B=C 三方实证 |
| 3 Q-1 措辞纪律 | 🟢（注记 🟡） | 命令全文+输出原文逐字复现全同、无省写/口径词；探针注记未留档 |
| 4 Q-2 清 | 🟢 | 8/1/4/1=14+1 与生成器 ⚠15 清单一一对应；TRIAGE 第 2 条=类清单不符非锚失效，拆出有实证 |
| 5 金样 67 | 🟢（簿记 🟡） | 64+3 纯追加机械核；--check 实跑 67 全同；夹带净（fixture_frames +34/−0 纯追加）；批 1/2 全套 38/0 未动坏；矩阵未随批重生成（--check 账面不一致，不入 gates 热路径） |

**总判：需修复**——红项仅 1 且为引文面：case 30 行锚 7569-7572 → **7573-7580**（≤10 臂引 7573-7577），须同步改 commit message、fx-pickup-30.txt、fixture_frames.rs（文档注+断言消息）三处（未推送，amend 即可）；改锚后**可推送**。行为面（三臂差分/金样/门禁）已全部实测钉死，无需动夹具本体。

黄项随账（不阻断）：
1. 29/30/31 的 floor>10 臂（+floorTier / 50×tier 倍率）无夹具覆盖，tier 语义仅 scene.rs:4260 注释锚 + completeness.md:689 史录——建议 P1-a2 后续批补 floor12 臂夹具；
2. 「位点探针实证」输出未留档（批 2 同类先例 🟡）；
3. docs/coverage-matrix.md 未随批重生成 + method_reach 触达快照未随新场景重测（cover_matrix --check 实测报账面不一致；与批 1/2 随 commit 刷账实践不一致）——下笔随附 ①`tools/method_reach.py` 重测 ②`tools/cover_matrix.py` 重生成。

附带发现（本笔之外预存债，随锚账处置）：scene.rs:4257-4259 注释自带行锚 7556-7567/7568-7577/7642-7652 三处均漂移（真源 7565-7580/7581-7587/7651-7662）。
