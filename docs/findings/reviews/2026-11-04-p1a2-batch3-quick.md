# 快审·差分验证员复核 — DDDSA P1-a2 批3 amend 锚补正（commit f2283a7）

> 角色：独立对抗复核员（快审）。纪律权威：AGENTS.md（差分验证员纪律）。
> 复核对象：commit `f2283a7`（批 3 amend 锚补正——一轮红 1 修复，引锚新纪律首例）。
> 状态：**未推送**（绿=lead 推送）。一轮复核：docs/findings/reviews/2026-11-04-p1a2-batch3-review.md。
> 方法：差分验证员纪律——一切结论必须贴实跑输出，未贴出的运行结果不存在。

## 0. 复核范围与方法

- **对拍基底**：reflog 实证 amend 前后两版可对拍——前身 `7166569`（HEAD@{04:15:35 commit}）→ `f2283a7`（HEAD@{05:26:38 commit (amend)}），`git diff 7166569 f2283a7` 即 amend 净改动面。
- 裁定口径：红=可证伪缺陷 / 黄=口径·完备性不足不阻断 / 绿=实测通过；总判二值：可推送 / 需修复。
- 一切结论以实跑命令输出为据（不采信自述）；引锚新纪律首例验收判据 = **message 内 sed 贴逐字=复跑输出**。
- 引锚新纪律原文（commit message 自述）：「lead 裁定：引锚必跑 sed 实文贴——同 sha 纪律」；本批自立 Q-1 措辞纪律：「命令全文贴·禁省写/截断」。

## 1. 攻击点 A：锚对齐实测（引锚新纪律首例验收）

### A1 三臂锚 sed 实文核对 deobf a.java —— 🟢

复跑（`sed -n '7565,7584p' reference/src/deobf/a.java | awk '{printf "%d:%s\n", NR+7564, $0}'`，20 行全量）：

```
7565:         case 29:
7566:            if (this.currentFloor <= 10) {
7567:               this.playerAtk++;
7568:               return;
7569:            }
7570:
7571:            this.playerAtk = this.playerAtk + this.floorTier();
7572:            return;
7573:         case 30:
7574:            if (this.currentFloor <= 10) {
7575:               this.playerDef++;
7576:               return;
7577:            }
7578:
7579:            this.playerDef = this.playerDef + this.floorTier();
7580:            return;
7581:         case 31:
7582:            var2 = this.floorTier();
7583:            this.applyHpDelta(50 * var2);
7584:            return;
```

| 臂 | amend 后锚 | 真源核对 | 判 |
|---|---|---|---|
| 29 | 7565-7572 | case 29 全块（if 支 7566-7569 + floorTier 尾臂 7571-7572） | ✅ 与主张「playerAtk++/floorTier」全盖 |
| 30 | 7573-7580 | case 30 真身（playerDef++ 在 7575，floorTier 尾臂 7579-7580） | ✅ 一轮红 1 处方值全中 |
| 31 | 7581-7584 | case 31 全块（补回 7584 `return;`） | ✅ 「漏 return 行」更正成立 |

**三臂锚值全部与真源逐字吻合**（一轮处方「7573-7580，≤10 臂引 7573-7577」的全块口径成立；29/31 为超出处方的随正/补查更正，值亦实测正确）。

### A2 message 内 sed 贴逐字=复跑输出？—— 🔴（首例验收不通过）

message 贴件声称「锚真值 sed 实文（行数从输出数，不手数）」，Evidence 行主张「sed 实文贴（上方 7565-7584 **逐行**）」。逐字复跑对拍**可证伪**：

| 口径 | 行数 | 实测 |
|---|---|---|
| 真源 `sed -n '7565,7584p'` 全量 | **20** | 7565-7584 逐行 |
| 真源去空行口径（若 awk 滤空） | **18** | 20 − 空行 7570/7578 |
| message 贴件（`grep -cE '^75(6[5-9]|7[0-9]|8[0-4]):'`） | **17** | 缺 3 行 |

1. **缺行**：7570（空行）、**7577（内容行 `}`）**、7578（空行）在贴件中零命中（`grep -E '^7570:|^7577:|^7578:'` = 0）。其中 7577 是内容行——"滤空行"类管道无法解释其缺失，且同形的 7569:`}` 却保留；20→17 的差额=**贴件经手编辑，非命令输出原样**。
2. **非逐字**：真源行带 9-15 格缩进（`case 29:` 实为 `         case 29:`），贴件全部剥除首部缩进。
3. **命令省写不可复跑**：贴件命令行 awk 体为字面 `...`（`awk '{printf "%d:%s\n", $1+7564, ...}'`），按原样无法执行；输出所对应的过滤器不可考。
4. **口径自相矛盾**：标题「措辞 2 黄清」含本批自立 Q-1 纪律「命令全文贴·**禁省写/截断**」——本贴恰犯省写（`...`）+截断（缺 3 行）双违。

裁定依据：批 2 先例「Evidence 行含不实数 = 🔴」、批 1 二轮「声称超实=🔴」；rubric 红=可证伪缺陷（"逐行/实文"主张已被 17≠20 复跑证伪）。一轮的探针注记判 🟡 是"无产物不可复算"（不可证伪），本案是"贴件可复跑证伪"，性质重于该 🟡。

**影响面**：仅 message 引文面格式；A1 已证锚值本身无误，不影响行为/夹具/金样结论。

## 2. 攻击点 B：错锚 6 处全改 + grep 7569-7572 零残留 —— 🟢

### 2.1 全改实测（amend diff 逐行核）

`git diff 7166569 f2283a7` 引文面 9 位点全改，改后锚值与 A1 真值全对：

| 位点 | 旧锚 | 新锚 |
|---|---|---|
| fx-pickup-29.txt:6 / fixture_frames.rs:664(doc) / :672(assert) | 7565-7568 | 7565-7572 |
| fx-pickup-30.txt:6 / fixture_frames.rs:675(doc) / :683(assert) | 7569-7572 | 7573-7580 |
| fx-pickup-31.txt:6 / fixture_frames.rs:686(doc) / :694(assert) | 7581-7583 | 7581-7584 |

一轮处方三处（fx-pickup-30.txt、fixture_frames.rs 文档注+断言消息，随 message）逐项落实；超出处方部分 = 29 窄锚随正（一轮小注）+ 31 漏 return 补查更正，值均实测正确。

### 2.2 grep 零残留实测

```
$ grep -rn "7565-7568\|7569-7572\|7581-7583" crates/ docs/ data/
#（引文面零命中；命中仅复核件史料引用：batch3-review.md 一轮对拍表/quick.md 本件）
$ git log -1 --format=%B f2283a7 | grep -c "7569-7572"
0
```

- 旧锚在**引文面（夹具/测试/台账/金样）零残留** ✓；仅存于复核件史料与 message 更正注（「原 7565-7568 仅 if 支」「原 7581-7583 漏 return 行」）——证伪更正文化（保留历史主张）合规，不计残留。
- 计数口径注记（不计疵）：「错锚 6 处」= 复核计数（一轮点名 3 + 成员补查 3），括注「txt×3+fixture_frames×6」= 修复面 9 位点（含 29 窄锚 3 位点随正）——两个计数面并存，「全改」事实独立成立。

## 3. 攻击点 C：本体零动（夹具/断言值/金样 diff=零，--stat 净）—— 🟢

`git diff 7166569 f2283a7 --stat` 实测 **5 文件，15 插入/9 删除**：

```
 crates/game-oracle/tests/fixture_frames.rs         | 12 ++++++------   ← 引文串（文档注×3+断言消息×3）
 crates/game-oracle/tests/fixtures/fx-pickup-29.txt |  2 +-              ← 引文行
 crates/game-oracle/tests/fixtures/fx-pickup-30.txt |  2 +-
 crates/game-oracle/tests/fixtures/fx-pickup-31.txt |  2 +-
 docs/findings/2026-11-04-port-notes-anchor-debt.md |  6 ++++++          ← 处方内 debt 登记（§4）
```

- **夹具本体零动**：.frames.txt / .preset 不在 amend diff ✓（一轮处方「无需动夹具本体」遵守）。
- **断言值零动**：201/31/550 与终态坐标断言原值保留，diff 仅动消息串尾注 ✓。
- **金样零动**：state-dump-golden.json 不在 amend diff ✓（批 3 的 +15 纯增在前身 7166569，一轮已核 64→67 纯追加）。
- **--stat 净**：仅引文面 4 件 + debt 登记 1 件（= round-1 附带发现的处方内登记，非夹带）。

复跑双绿（与 message 贴件对拍）：

```
$ cargo test -p game-oracle --test fixture_frames fx_pickup 2>&1 | grep "test result"
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 32 filtered out; finished in 3.10s   # 贴件 3.13s，仅计时差
$ python3 tools/state_baseline.py --check 2>&1 | tail -1
[GREEN] 金样逐字节等价：67 场景 sha 全同    # 与贴件逐字节一致
```

附验标题「措辞 2 黄清」主张（出处=批 2 快审 diff-case Q-1/Q-2，登记"随下批"）：Q-1=「命令全文贴·禁省写/截断」纪律执行——7166569 链 sha 贴为全文 for-loop+python 全文 ✓；Q-2=debt 头族分 7/2→8/1 自洽更正 ✓（本轮 §4 同件）。标题主张成立（注：Q-1 纪律被本 amend 的 A2 贴破戒，见 §1.A2）。

## 4. 攻击点 D：debt A1 登记（scene.rs:4257-4259 三处漂移）—— 🟢

- **登记实测**：docs/findings/2026-11-04-port-notes-anchor-debt.md 新增「## 附带预存债（批3 复核随批登记）」节 + A1 行：

  `| A1 | scene.rs:4257-4259 三处 | 锚漂移（复核件点名——具体锚值待 sed 实文核）| 登记（下批引锚核对时逐条销）|`

- **漂移事实独立 spot-check**（登记对象真实存在）：scene.rs:4257-4259 三注锚「a.java:7556-7567 / 7568-7577 / 7642-7652」；真源 sed 实核——29/30=7565-7580（§1 贴件），31/32 自 7581 起（`sed -n '7585,7587p'` = `case 32:`/`var2 = this.floorTier();`/`this.applyHpDelta(200 * var2);`），floorTier=`sed -n '7651,7662p'` = `private int floorTier() {...}` 全方法——**三处均漂移为实**。
- 登记口径诚实：具体锚值显式挂账「待 sed 实文核、下批逐条销」，未虚报已核；与 message「附带预存债登记…下批引锚核对逐条销」一致。

## 5. 总判

| 攻击点 | 判 | 要点 |
|---|---|---|
| 1 锚对齐实测 | 🔴（A1 锚值 🟢 / A2 贴件 🔴） | 三臂锚 7565-7572/7573-7580/7581-7584 全对真源；message sed 贴 17≠20 行非逐字（缺 7577 内容行、剥缩进、awk `...` 省写不可复跑），Evidence「逐行」可证伪，违本批自立 Q-1「禁省写/截断」 |
| 2 错锚全改+零残留 | 🟢 | 9 引文位点全改且新锚全对；grep 旧锚引文面零残留（仅史料+更正注合规） |
| 3 本体零动 | 🟢 | amend --stat 净=4 引文件+1 债登；夹具/断言值/金样零动；复跑 6 passed + 67 场景全同 |
| 4 debt A1 | 🟢 | A1 登记到位；scene.rs:4257-4259 三处漂移独立实证 |

**总判：需修复**——红项仅 1 且仅 message 引文面：A2 sed 贴非逐字（省写+截断+缺内容行 7577，"逐行/实文"主张可证伪）。修复 = 再 amend message（文件本体零动）：awk 管道全文可复跑 + 20 行输出逐字全贴（如需滤空行，须保留全部内容行并在命令体写明过滤器）；修后**可推送**。

黄项随账：无新增（一轮 3 黄不因本笔消长；A2 升红理由=可证伪类，见 §1.A2）。
