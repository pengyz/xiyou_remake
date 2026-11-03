# P1-a1 三笔清账 对抗复核（2026-11-03）

> 独立对抗复核员 / remake-verifier 纪律 / 逐提交复核
> 复核对象：commit `739e7e1`（P1-a1 复核三笔清：口径更正/五锚扫/\r 钉）——**未推送，绿=lead 推送**
> 前轮：[2026-11-03-p1a1-yellows-review.md](2026-11-03-p1a1-yellows-review.md)（三笔随行项出处）
> 判定口径：红=需修复才可推；黄=留观/瑕疵不阻断；绿=通过。

## 0. 复核范围与方法

- 复核面：BUG-050 口径句、旧锚五处扫描、\r 臂第五测、夹带检查（四项攻击点）。
- 方法：`git show 739e7e1` 逐 hunk 静态核对 + grep 实测 + a.java 源锚回测 + 测试实跑
  （oracle quote 5 测 / core 全量 / 金样 60 --check）。
- 立场：怀疑一切"清账"；结论必须贴实际输出。

## 1. BUG-050 口径句（攻击点 1）—— 🟢 绿

- **口径句与前轮复核实证逐点一致**（yellows-review §2 实证 vs bug-ledger 补注）：
  - "23+ 不可达"=use-count 推理非流程实测 ← 前轮"22 从哪来：型域映射推理，非游戏流程实测；
    实为 use-count 槽数非栈位数" ✓ 同因。
  - "a.java:7534-7555 流程可入栈 **15 型**" ← 前轮"pickupItemType（a.java:7534-7555）
    case 13..=25 ∪ {85,86}=15 型" ✓。源锚回测：pickupItemType switch 15 个 case 标签
    （13–25 共 13 + 85/86），case 85/86 落 **7552-7553**（awk 实测行号逐字同前轮）。
  - "findItemStackIndex 85/86 可达 **24 件**" ← 前轮"findItemStackIndex（a.java:7664-7675）
    按精确 type 匹配……可达 24 件" ✓。源锚回测：findItemStackIndex@7664-7675
    `itemStackTypes[var3]==var1` 精确匹配；addItemToItemStack 未命中即 append
    （`itemStackTypes[itemStackSize]=var1; itemStackSize++`）——13..=34 共 22 件 +
    85/86 两独立型 = 24 件，机制成立（A 级复核）。
  - "上限表述以此为准；BUG-050 结论幸存（33 件双口径均不可达）" ← 前轮"API 上限 24、
    流程上限 15，皆 <32" ✓ 结论幸存表述同。
- **bug-ledger 原地补注不删史** ✓：追加 `【2026-11-03 口径更正】` 段于 BUG-050 条目尾，
  原"影响"栏 **"23+ 道具不可达（正常流程栈深≤22，ITEM_USES 22 槽域）"原文保留**、
  现象/复现/登记/trace id 四栏未动，读者可见"曾经错在哪"（AGENTS.md §3.6 证伪更正文化
  合规——日期更正+历史主张在场+补注引语"23+ 不可达"可对回原文）。
- 注记（不计缺陷）：补注引语"23+ 不可达"较原文"23+ **道具**不可达"少一字，可辨识；
  "15 型"与前轮"流程上限 15（件）"单位表述不同（15 型各自独立栈位=15 件上限，等价）。

## 2. 旧锚五处全扫（攻击点 2）—— 🟡 黄（点名清单全中且锚真，但"零残留"repo 实测不成立）

- **点名清单全中 → 9291-9297** ✓：`git show --stat` 4 文件面内，五处逐一对上——
  save.rs:15/54/138（表注/字段注/解析注）、scene.rs:600（load_floor_from_save 读回注）、
  4878（fidelity_fix_tests 引注）——全部 9285-9291→9291-9297，diff 仅这五 hunk，无多改。
- **新锚为真** ✓：a.java 实测（sed 9275-9305 逐行数）——readBoolean 回读循环 **9291-9293**、
  skip(128-count) **9295-9297**，合并区间 = **9291-9297**（与前轮 §4 双实测逐字同）；
  9285-9287 实际落在 inventoryEntryCount/journal 读循环，确属漂移错锚。
- **"grep 9285 零残留"实测不成立** ❌——全库（排除 target/）`grep -rn 9285` 仍命中
  **4 处同族漂移锚**（非 hash 伪命中）：
  | 位置 | 现锚 | 实际语义对应 | 判定 |
  |---|---|---|---|
  | crates/game-core/tests/mov_route_scene.rs:11 | 9285-9287 | 读回旗标=9291-9293 | 错锚（同族 -6 漂移） |
  | crates/game-core/tests/mov_route_scene.rs:417 | 9285-9287 | 读回旗标=9291-9293 | 错锚（同族 -6 漂移） |
  | crates/game-core/src/save.rs:108 | 9285-9287 | script_flags 读回=9291-9293 | 错锚（同族 -6 漂移） |
  | docs/spec/refactor-behavior-matrix.md:59 | 9285-9291 | 读档 flags=9291-9297 | 错锚（同族 -6 漂移） |

  （tools/method_reach/snapshot.json 两处 "9285" 为 sha256 哈希子串，伪命中不计。）
- **claim 口径裁定**：commit 正文"复核件点名清单全中，零残留"——"点名清单全中"为真；
  "零残留"仅在**点名五处内部**成立，repo 级 grep 9285 ≠ 0。措辞过宽=瑕疵；
  但本批申报面即"五处扫"（前轮 §4 建议原话），未虚报范围、未漏改点名项，
  4 处残留均为**点名清单外**的同族旧锚（前轮 §4 亦未点名）——非本批回归。
  裁定 🟡：措辞"零残留"应限定"点名五处"；残留 4 处（3 代码注记+1 spec 表）建议
  P1-a2 补扫（与前轮"随行扫齐"同一处理惯例）。
- 注记：全部漂移锚均恰好 -6 行（9285→9291/9287→9293），与 a.java 早期插入 6 行的历史一致，
  同族同因可一次扫净。

## 3. \r 臂第五测（攻击点 3）—— 🟢 绿

- **断言真伪读测** ✓：`carriage_return_pinned`：`e_quoted("x", Some("a\rb")).value ==
  "\"a\\rb\""`——实现核：e_quoted 0x0D 臂 `push_str("\\r")`（state_dump.rs:65），
  输入 a/CR/b → `"a\rb"`，断言字面逐字同；**真**。
- **Java 镜像核** ✓：TraceSink.java `quote()` case `'\r'` → `append("\\r")`——
  0x0D 两侧转义逐字同（A 级对拍），非 Rust 单侧自洽。
- **分支面闭合度**：四测原盖 7 转义臂中 6 臂（缺 \r），第五测补上后 **7/7 臂全钉**
  （\" \\ \n \r \t \u 代理对 + 空串/null 支）——前轮 §1 小注"\r 臂留待后续件"已清。
- **实跑** ✓：`cargo test -p game-oracle --lib p1a1_quoted` → **ok. 5 passed; 0 failed**
  （本复核实跑，carriage_return_pinned 在列且过）。
- 注记：`<0x20` 其余控制符（0x00-0x08/0x0B/0x0C/0x0E-0x1F）走 `\u` 兜底臂——
  与 Java default 支同构，未逐枚举钉但臂级对拍已闭（不计缺陷）。

## 4. 夹带检查（攻击点 4）—— 🟢 绿（面净+金样全同；注记：验证行计数失实）

- **`git show --stat 739e7e1` 仅本批面** ✓：4 文件=save.rs（3 处注记锚）/scene.rs
  （2 处注记锚）/state_dump.rs（+1 \r 测）/bug-ledger.md（+4 补注）——14+/5−，
  逐 hunk 核过与三笔申报面一一对上；无 gates/、data/status/、reference/、original/、
  data/naming/、rust-test-counts.json 夹带。工作区未跟踪文件（android/build/、
  评审文档等）不在提交内。
- **金样 60 全同实跑** ✓：`python3 tools/state_baseline.py --check` →
  **`[GREEN] 金样逐字节等价：60 场景 sha 全同`**（本复核实跑，与 commit"--check 60 全同"
  逐字同）——注记/测试批零行为面得证。
- **oracle 全量实跑** ✓：80 passed / 0 failed / 3 ignored（=前轮 79+1 新 \r 测，
  增量恰为申报的第五测，无未报行为改动）；quote 子集 5/0 与 commit 逐字同。
- **无其他未报改动** ✓：diff 内容=五处锚行换注+一测+一补注，别无代码逻辑变更。
- 注记 1（🟡 计数措辞失实）：commit 验证行"core **319**/0"与实跑不符——
  本复核 `cargo test -p game-core` 实跑 **320 passed / 0 failed / 2 ignored**，
  且 cb710f4 自身验证行即"core 320/0（319+1 边界测）"；739e7e1 未增减 core 测，
  应为 320/0——"319/0"系沿抄旧数。结论（0 失败）幸存，仅计数失实，不阻断。
- 注记 2（不计缺陷）：本批 +1 测未随批 `gates ratchet-update`——ratchet 实跑
  `[✓ PASS] test-ratchet — total=930(基线929) 无回退`（只升不降语义下合规）；
  基线欠修 1，下次随批上修即可。

## 5. 总判

- **红/黄/绿：0 红 2 黄 2 绿**——
  攻击点①口径句 🟢、②五锚扫 🟡（点名五处全中且新锚 9291-9297 为真，但
  "零残留"措辞过宽：repo grep 9285 仍中 4 处同族 -6 漂移错锚
  [mov_route_scene.rs:11/417、save.rs:108 之 9285-9287；refactor-behavior-matrix.md:59
  之 9285-9291]，均在点名清单外，非本批回归）、③\r 第五测 🟢（断言真+Java 镜像同+实跑 5/0）、
  ④夹带 🟢（--stat 净+金样 60 全同+oracle 80/0）；另验证行"core 319/0"计数失实（实跑 320/0）🟡。
- **可推送（绿=lead 推送）**——两黄均为措辞/补扫面，零行为回退、金样全同、门禁实跑无回退。
  推送前建议（零成本）：amend 验证行"core 319/0"→"320/0"、"零残留"→"点名五处零残留"；
  P1-a2 随行补扫 4 处同族锚（同 -6 漂移一次扫净）+ ratchet 基线上修 930。
