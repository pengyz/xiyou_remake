# 快审·差分验证复核 — P1-a2 批 2 amend 补正（commit f6f5520）

> 角色：独立对抗复核员（快审·差分验证员纪律，AGENTS.md 权威）。
> 复核对象：commit `f6f5520`（批 2 amend 补正——一轮红 2 黄 4 修复，未推送，绿=lead 推送）。
> 一轮复核：docs/findings/reviews/2026-11-04-p1a2-batch2-review.md。
> 复核时点：2026-11-04 02:47–03:20 +08:00。工作目录 /home/peng/workspace/xiyou_remake。

## 0. 复核范围与方法

- 对象=commit `f6f5520`（amend 自 pre-amend `bfc8712`，reflog HEAD@{1} 可取原身位对拍）。
- 方法=**独立重算**（不采信 commit 自贴）：链 sha 由 `.frames.txt` 帧 sha 逐帧 join 重算；⚠ 清单由
  `tools/cover_matrix.py render()` 取 `info['bad']` 实测；臂面缺臂数由 `ARM_FIXTURES` × `.frames.txt`
  存在性重数；a.java 行锚对 `reference/src/deobf/a.java` 逐行核；本体 diff 用
  `git diff bfc8712 f6f5520` / `git diff 41885c5 f6f5520` 实跑。
- 口径：证据分级 A（字节/命令输出）/B（静态推理）/C（猜测），见 AGENTS.md §3。本篇判据均为 A 级实测。

## 1. 攻击点 1 — 红-1：26 臂链 sha 证伪更正（653a5008→92f980cc）

**独立重算 320 帧链 sha（`sha256("\n".join(帧sha))` = tools/frames_fixture.py `frame_chain` 口径）：**

| 臂 | 帧数 | 重算 chain16 | commit 贴 | 头注全 sha | 判 |
|---|---|---|---|---|---|
| fx-pickup-26 | 320 | `92f980cced9581b0` | `92f980cced9581b0` | `92f980cced9581b01867c9…e50418` | ✅ 三处一致 |
| fx-pickup-27 | 320 | `ea781bbac80d6ad5` | `ea781bbac80d6ad5` | `ea781bbac80d6ad5925d1c…4b264e` | ✅ |
| fx-pickup-28 | 320 | `a03d0fa01bf7e24f` | `a03d0fa01bf7e24f` | `a03d0fa01bf7e24f7d7fb2…5bb6e6` | ✅ |

**结论：红-1 证伪更正成立**——26 臂真链=92f980cc（重算逐位对上），27/28 两链亦重算一致。前版
`653a5008` 系写错对象（yk=0 重gen 前旧链残响），更正正确。

**四组实贴核验（逐字=可复跑命令输出？）：**

| 组 | commit 贴 | 独立实跑 | 判 |
|---|---|---|---|
| 链 sha | 3 行 chain | 重算 3/3 逐位同 | ✅ 数值实（命令体 `<join(帧sha)>` 为省写，非逐字可复跑） |
| 测试行 | `test result: ok. 3 passed; 0 failed` | 实跑=同前缀（完整行为 `3 passed; 0 failed; 0 ignored; 0 measured; 32 filtered out`） | ✅ 计数实（为前缀截断，非整行逐字） |
| 金样行 | `[GREEN] 金样逐字节等价：64 场景 sha 全同` | 金样 json `scenarios`=64（61 存量+3 新）；`--check` 见 §4 | ✅ 64 计数实 |
| 闸行 | `gates check PASS=13 FAIL=0` | 实跑 `PASS=13 FAIL=0 SKIP=0` | ✅ |

**653a5008 出处（"COMMIT_EDITMSG 全库仅存"）**：实测 `653a5008` 现存于
`.git/COMMIT_EDITMSG` + `docs/findings/reviews/…-review.md`（一轮复核引述=史料）+ 本篇。按 commit
自列排除面（史料=findings/reviews），非史料唯 `.git/COMMIT_EDITMSG` ⇒ 命意成立，但"全库仅存"
字面不严（史料件亦留印）。**措辞小疵，非造假**。

## 2. 攻击点 2 — 红-2：销 2+登 14+1 triage=17 算术自洽

**⚠ 清单实测（`cover_matrix.render()['bad']`）=15 条**，族分：PORT_NOTES 8 / TRIAGE 2 /
YELLOW_NOTES 4 / ARM_GAPS 1。其中：

- **锚失效（=登记面）14 条**：PORT_NOTES 8 + TRIAGE 1（saveGlobalProgress）+ YELLOW_NOTES 4 + ARM_GAPS 1；
- **另类 1 条**：TRIAGE「修正暴露」与机械复算不符（漏登 m_101/m_134/showNotify、多登 closeRecordStoreAndGc/
  deleteRecordStoreQuietly）。

**算术自洽：✅** 原 ⚠ 17 = 销 2 + 登 14（锚失效）+ 1（另类）；现 ⚠ 15 = 14 + 1（销 2 已从清单消失）。
17 = 2+14+1、15 = 14+1 均闭合。

**销 2 机制实证**：`git diff 41885c5 f6f5520 -- tools/cover_matrix.py` 仅改 PORT_NOTES 锚串
（`self.overlay_kind_item = t`→`self.menu.set_overlay_kind_item(t)`；`scene.saved_floor_states`→
`scene.persist.saved_floor_states`）——此 2 锚改后匹配=销；校验器逻辑（check_notes/_bad_rust_pairs）
**零改动**（非放宽门禁）。✅

**debt 行 3 归登记 + 实名对齐：✅** 原表行 3（initFloorStates `saved_floor_states: vec![None; 56]`）
由"销"改"登记（括注已加，锚仍失配）"，移入新表行 1，与实测锚失效逐字同串；登记面 14 实名=实测 14
锚失效**逐条一一对应**（我逐条比对 PORT_NOTES/TRIAGE/YELLOW/ARM 全 14 名）。

**措辞小疵（低危）**：debt 文**头**写"异族四类：PORT_NOTES 7 / TRIAGE 2 / YELLOW_NOTES 4 / ARM_GAPS 1
（+行 3 initFloorStates=PORT_NOTES 共 8 PORT_NOTES）"——其自身表格/commit/实测的**登记面**族分均为
**PORT_NOTES 8 / TRIAGE 1**（TRIAGE 第 2 条=另类，已单列"+1 另类"）。头注"7/2"与表"8/1"不自洽
（括注已把 PORT_NOTES 补正到 8，但 TRIAGE"2"未拆出另类）。commit message 用"PORT_NOTES 8/TRIAGE 1+1"
**正确**。仅账面措辞，不涉算术/实名。

## 3. 攻击点 3 — 黄四连核销

| # | 项 | 声称 | 实测 | 判 |
|---|---|---|---|---|
| ① | 26.txt 断言注记 | 0→1 改 **0→2** | diff 实证 `yellowKeyCount 0→1`→`0→**2**`；测试 `assert_eq!(sc.player.yellow_keys, 2, "双拾实况…")` | ✅ 注记与断言（==2）对齐 |
| ② | 28.txt 行锚 | 7562-7664→**7562-7564** | diff 实证；a.java:7562-7564=case 28 blueKeyCount++（`case 28:`7562/`blueKeyCount++`7563/`return`7564）| ✅ 锚=真源 |
| ③ | debt 异族四类 | "同族多锚"→异族四类 | 表已分 PORT_NOTES/TRIAGE/YELLOW_NOTES/ARM_GAPS 4 族（头注族分数目小疵见 §2）| ✅（措辞小疵）|
| ④ | arm_coverage 头注 | 补"批 2 后缺臂 **11**" | diff 实证；`ARM_FIXTURES`×`.frames.txt` 重数=21 组，have 10 / **missing 11** | ✅ 11 精确 |

26/27/28 三臂真源（`reference/src/deobf/a.java`）逐行核：case 26 yellowKeyCount++=7556-7558、
case 27 redKeyCount++=7559-7561、case 28 blueKeyCount++=7562-7564，与三测断言/注记全对齐。

## 4. 攻击点 4 — 本体未动坏（夹具/金样/闸）

**夹具三件套 diff=零（amend 只动账面）：✅（行为面零变动）** `git diff bfc8712 f6f5520 --stat`
仅 4 文件，全为账面/注记：arm_coverage.rs（`//!` 头注 1 行=缺臂 11）/fx-pickup-26.txt（`#` 断言注 1 行）/
fx-pickup-28.txt（`#` 锚注 1 行）/debt.md（账面）。**`.preset`+`.frames.txt` 零 diff**；`.txt` 仅 2 行
`#` 注释改，**事件行零变动**（`grep -vE '^[+-]\s*#'` 为空）⇒ 夹具行为面（事件/帧/预设）字节不动。
（严口径下 26/28.txt 非逐字节同，但仅注记措辞=账面，符合"amend 只动账面"。）

**金样 64 全同实跑**：金样 json `scenarios`=64（61 存量逐字节不动 + 3 新 fx-pickup-26/27/28 纯增，
`git diff 41885c5 f6f5520 -- …state-dump-golden.json` = 15 增 0 删）；`state_baseline --check` 实跑
= `[GREEN] 金样逐字节等价：64 场景 sha 全同`（exit 0）✅。

**--stat 净**：本笔 15 文件 = 3 夹具三件套(9) + arm_coverage.rs + fixture_frames.rs(3 新测) +
state-dump-golden.json(纯增) + coverage-matrix.md(regen，`--check` 逐字节一致) + debt.md(账面) +
cover_matrix.py(锚串销账)。**无 `gates/`、无 `reference/`、无 game-core/src、无既有测试删改**；
无阈值放宽/白名单扩/用例删。cover_matrix.py 改动=PORT_NOTES 账面锚串（销账机制），非校验器逻辑。

**门禁实跑**：
- `cargo test -p game-oracle --test fixture_frames fx_pickup` → `test result: ok. 3 passed; 0 failed` ✅
- `python3 tools/cover_matrix.py --check` → `逐字节一致` ✅
- `python3 tools/state_baseline.py --check` → `[GREEN] 金样逐字节等价：64 场景 sha 全同`（exit 0）✅
- `python3 gates/cli.py check` → `== 汇总 == PASS=13 FAIL=0 SKIP=0`（exit 0）✅
  13 门全 PASS；`test-ratchet` PASS（total=935 基线929 无回退，注：arm_coverage(test) 新增二进制待
  ratchet-update——test-adding commit 预期注记，非回退）。四组实贴（测试行/金样行/闸行/链 sha）**全实**。

## 5. 差异分类登记（diff-case）

无需回派的回退。遗留措辞小疵 2 项（红-1"全库仅存"/命令省写+测试行截断、红-2 debt 头族分"7/2"），
均账面措辞级、非行为/非算术/非证据造假，**不阻断推送**（lead 可择机随下批措辞统一）：

- **diff-case Q-1**（措辞）：红-1 commit 四组实贴中链 sha 命令体为省写 `<join(帧sha)>`、测试行为前缀
  截断、"653a5008 全库仅存"未计史料件。分类=措辞不严（非移植/非规格/非投影）。处理=下批措辞统一，无需回派。
- **diff-case Q-2**（措辞）：红-2 debt 文头族分"PORT_NOTES 7 / TRIAGE 2"与自身表/commit/实测
  登记面"8 / 1"不自洽（TRIAGE 第 2 条应拆出为另类）。分类=账面措辞。处理=随下批 debt 头注订正。

## 6. 总判

- **红-1**：✅ 修复核实（真链 92f980cc 独立重算逐位对上，四组实贴计数全实；措辞小疵 Q-1）。
- **红-2**：✅ 修复核实（17=2+14+1、15=14+1 算术闭合；行 3 归登记+14 实名逐条对齐；措辞小疵 Q-2）。
- **黄四连**：✅ 全核销（①0→2 ②7562-7564 ③异族四类 ④缺臂 11 精确）。
- **本体未动坏**：✅ 夹具行为面零 diff、金样 61 存量不动+3 纯增、--stat 净（无门禁/参考版/核心逻辑动）。

**红 0 / 黄 2（Q-1/Q-2，均措辞级）/ 绿 4 面（红-1 红-2 黄四连 本体）。**

> **总判：可推送**。两黄为账面措辞小疵（Q-1/Q-2），不涉行为/算术/证据造假，不构成回退；lead 推送时
> 可随下批措辞统一订正，不阻断。四组实贴（链 sha×3/测试行/金样行/闸行）已逐条独立复跑=全实。
