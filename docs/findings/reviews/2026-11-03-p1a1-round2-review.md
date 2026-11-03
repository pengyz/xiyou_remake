# P1-a1 红-1 修复（commit 752e985）二轮对抗复核（remake-verifier·新视角）

> **复核对象**：commit `752e985`（P1-a1 红-1 修复：overlayText quote() 等价编码 + null≠"" + itemStack 32 + scriptLineFlags 变长 + 断言换钉 + A2 重冻）——**未推送，绿=lead 推送**。
> **一轮复核**：docs/findings/reviews/2026-11-03-p1a1-dump-face-review.md（红 1 黄 2）。
> **二轮纪律**：remake-verifier——怀疑一切搬运，怀疑必须机器可复现；禁止放宽阈值/删用例；未运行不声称。
> **复核人**：独立对抗复核员（二轮·新视角）。
> **日期**：2026-11-03。

## §0 骨架与工作记录

（本文件按节回写；每节结论带证据指针。）

- [x] §1 quote() 逐字对拍（转义表 + 代理对双转义镜像）
- [x] §2 null 分支（None→"null" vs Some("")→`""`）
- [x] §3 byte[32] 溢出面（17/32 边界）
- [x] §4 68 槽语义（scriptLineFlags 变长、层覆盖）
- [x] §5 断言换钉（a.java:2038-2039 回指 + skip 链）
- [x] §6 A2 重冻纯追加复核（7 名 diff + 旧行零动）+ 红证据三件套
- [x] §7 无夹带与红线核 + §8 总判

## §1 quote() 逐字对拍 —— 🟢 绿（11/11 逐字节全同）+ 🟡 回归网缺口

- [x] **转义表逐字核**（e_quoted state_dump.rs:55-77 vs TraceSink.java:44-64）：
  五转义集确认为 **`\\` `\"` `\n` `\r` `\t`**（Java switch 五 case ↔ Rust match 五 arm
  0x22/0x5C/0x0A/0x0D/0x09），转义产物逐字符相同（`\\\"`→`\"`、`\\\\`→`\\`、
  `\\n`/`\\r`/`\\t`）。兜底同为 `c<0x20||c>0x7E` → `\u%04x` **小写 4 位 hex**
  （Java `String.format("\\u%04x", (int)c)` ↔ Rust `format!("\\u{:04x}", u)`——
  0x20..=0x7E 段双侧直出）。引号包裹双侧同。
- [x] **代理对双转义镜像 charAt**：Java charAt 逐 UTF-16 code unit；Rust
  `sv.encode_utf16()` 逐 code unit——😀(U+1F600)双侧同为 `\ud83d\ude00` **两个转义**
  （高位/低位代理各一），实测确认。unpaired surrogate 不可入 Rust str（UTF-8 不变量），
  仅 Java 单侧可能出现——overlay_text 数据域（脚本文本）不可达，记录为覆盖边界。
- [x] **worktree 对拍样例逐一断言（本复核实跑 2026-11-30）**：从 worktree 源
  `sed -n '55,77p'` 提取 e_quoted 值分支（与 /tmp probe 逐行 diff 仅尾部 Entry 构造差异，
  逻辑体 verbatim），与**真实 TraceSink.java**（javac 实编译 oracle.host.TraceSink）
  对拍11 样例：ASCII "abc" / 中文"你好世界" / emoji 代理对×2 "😀🎮" / 五转义字符
  `"\\\n\r\t` / 五转义混排 / 边界 0x20·0x7E·0x7F / 控制符 0x00·0x1F / 代理对+转义混排 /
  全混排 / null / 空串——**11/11 输出逐字节全同**（diff 空）。
- [x] 实存 trace 对照：prologue_trace_excerpt.txt:58 `FLD 062 java/lang/String
  "\u83e9\u63d0\u8001\u7956: \\cF8F8F8\u609f\u7a7a~\u609f\u7a7a~\uff01"` 形态
  （`\\c` 双反斜杠、CJK \u 转义、ASCII 直出）与 e_quoted 输出形态一致。
- 🟡 **回归网缺口（非阻断）**：worktree 内**无 e_quoted 单测**（state_dump.rs 无
  #[cfg(test)]；dump_face.rs 仅名集在场断言）；60 场景金样对 quote 面的覆盖实测
  （/tmp/r2_dumps/head 全量 grep）：`\\` 转义 28,797 行、含 `\n` 等转义 722 行、
  null 39,578 行——但**代理对转义 0 行、Some("")→`""` 0 行**。即五转义中的
  \r \t 与代理对/空串分支仅由本轮外部对拍覆盖，金样回归网罩不住。
  建议后续件补 e_quoted 单测（11 样例表即测例）。
- **判定：🟢 编码等价成立（A 级实证）；🟡 样例级回归钉缺失（非阻断）。**

## §2 null 分支 —— 🟢 绿（实现等价）+ 🟡 区分无钉

- [x] **None→"null" 逐字等价**：StateDump.java:67-68 `if (v == null) return "null";`
  （value() 首支）→ Java 倾印 null String = 无引号字面 `null`；e_quoted None =>
  `"null".to_string()` 同为无引号字面 `null`。quote() 自身 null→"null"（TraceSink.java:46-48）
  与 value() 分支双源同值。对拍样例第 10 条双侧 `null` ✓。
- [x] **Some("")→`""` 与 null 区分**：Java quote("") = `""`（引号内空）；e_quoted
  Some("") = `""`。对拍样例第 11 条双侧 `""` ✓。`null` ≠ `""` 输出形态不同，
  分支实现正确——旧 `unwrap_or("")` 把 None 合流为 `ARR[0]` 的问题已按 Option 分支更正。
- [x] StateDump.java:74-75 String→quote() 源锚实存 ✓（本轮 sed 亲验行号）。
- 🟡 **区分无测试钉**：worktree 无断言区分 None→`null` vs Some("")→`""`；
  60 金样中 `overlayText = ""` 实测 **0 行**（null 39,578 行）——空串分支无金样触达。
  实现正确但钉缺失（与 §1 回归网缺口同源）。
- **判定：🟢 null≠"" 语义等价实证；🟡 区分未被测试/金样钉住（非阻断）。**

## §3 byte[32] 溢出面 —— 🟡 黄（17/32 边界测试缺失）

- [x] **容量对齐核实**：Java `itemStackTypes/itemStackUses = new byte[32]`
  （a.java:949-950 实存）；Rust scene_persist.rs:110-111 `vec![0; 32]`（本轮 diff 确认）。
  16→32 收口正确。全库残留16 长构造仅测试侧（combat_golden.rs:19-20、
  walk_replay_floor1.rs:85-86 手构 PlayerCombat）——不触 32 槽面，非缺口但不一致。
- [x] **栈满 32 行为对拍**：Java addItemToItemStack（a.java:7727-7740）写
  `itemStackTypes[this.itemStackSize]` **无边界检查**——满 32 再入第33 种
  不同道具 = ArrayIndexOutOfBoundsException（原版崩溃面）。Rust scene.rs:4322
  `if n < self.player.item_types.len()` **静默丢弃**——满栈行为分歧
  （Java 崩 vs Rust 吞）。理论可达面：道具型全集 = ITEM_USES[22]（scene.rs:4317，
  22 型去重）→ itemStackSize 上限 22 < 32，**溢出实际不可达**（17..22 段可达！）。
  本轮分歧收窄到 23+：修复前 16 槽在17 型即分歧，修复后 32 槽覆盖全部可达域。
- [x] **17/32 边界测试**：全测面实查——`item_stack_size` 赋值最大 **2**
  （t2/touch_tap/item13 各测试）；**17 与 32 边界测试 = 无**。commit 自认
  「金样 60 未触不代表无险」但未补钉。17..22 可达域的行为等价（存入第17+ 件）
  无任何断言覆盖。
- 🟡 判定：容量修复方向对（A 级源锚 a.java:949-950 实证），但：
  ① 满 32 溢出行为分歧仍在（Java AIOOBE vs Rust 静默吞，D4 保真口径下
  Rust 应 panic/等价面而非静默丢——现不可达故不阻断）；② 17/32 边界测试双缺。
  建议后续件：补 17 件入栈断言（可达域）+ 显式注记 33 件溢出=已知不可达分歧。

## §4 68 槽语义 —— 🟢 代码等价 / 🟡 "每层变长"表述失实

- [x] **68 是哪层的——答：不是任何"层"的，是全局单表**：`levelScriptLines`
  全库仅一次赋值（a.java:1124，构造器 `a()` 内 `new String[]{...68 行}`）；
  `scriptLineCount = levelScriptLines.length`（a.java:2038）同在构造器内执行一次；
  `scriptLineFlags = new boolean[scriptLineCount]`（a.java:2039）一次分配。
  Java 侧旗标恒长 **68**（68 行 DSL = 全游戏脚本行表，非逐层表）。
- [x] **"每层变长"表述失实**：commit 与 scene.rs:602 注记称「每层变长
  （levelScriptLines.length）」——实际 levelScriptLines 为一次性全局表
  （grep 全库唯一赋值点 1124），不存在逐层变长语义。Rust
  `vec![false; dialog_data.scripts.len()]` 与 Java `new boolean[levelScriptLines.length]`
  **表达式同构**（都是"取脚本表长"），行为等价成立（表长恒 68）；但
  「每层变长」的语义描述误导（对拍面若真逐层变长，钉测需多层覆盖——不必）。
- [x] **变长钉测覆盖**：load_restores_script_flags_prefix_only 断言 len==68
  （maplv 合成夹具层，scripts.len()==68，level_scripts.txt 实测 68 行）——
  覆盖 1 层/1 表长。因真相=全局恒 68，单表长钉足够；但测试名/注记的
  「每层变长」框架与真相不符。旁证：scene_persist.rs:233 默认构造
  `vec![false; 68]` 硬编码68 未同步改 scripts.len()（与 scene.rs:602 表达式
  不一致，数值巧合相等——真相恒 68 故无行为差）。
- **判定：🟢 行为等价（表长恒 68 双侧同）；🟡 「每层变长」表述失实+默认构造
  硬编码残留（文档级，非阻断）。**

## §5 断言换钉 —— 🟢 绿（a.java:2038-2039 可回指，skip 链语义清）

- [x] **新钉可回指**：fidelity_fix_tests.rs:164 `assert_eq!(sc.script_line_flags.len(),
  68, "a.java:2038-2039 new boolean[levelScriptLines.length]")`——a.java:2038-2039
  实存逐字（`this.scriptLineCount = this.levelScriptLines.length;` /
  `this.scriptLineFlags = new boolean[this.scriptLineCount];`，grep 精确行号确认）✓。
- [x] **skip 链（save 侧真值）语义清**：Java 读回（a.java:9291-9297）= 读
  scriptLineCount 个 boolean 后 `skip(128-count)`——≥68 字节被跳过**不入 state**；
  Rust 解析侧 save.script_flags 保全 128 项原始值（parse_floor_save max(count,128)
  字节，save.rs:133-135），应用侧仅拷 min(scripts.len(), save.len())=68 项
  （scene.rs:603-604）。测试钉三件套：len==68（场景面仅68 槽）+
  [0]/[67] 真（前缀读回）+ `save.script_flags[100]` 真（跳过字节真值仅存 save 侧）
  + count()==2（跳过位未漏入场景面）——skip 链两端钉清 ✓。
  旁证 a.java:9221-9227 写侧 128 字节（<count 写真值/false 补齐）与 Rust 写侧
  （scene_persist.rs:561-565 get(k).unwrap_or(false) 补 128）同构 ✓。
- [x] **换钉正当性**：旧断言 `!sc.script_line_flags[100]` 在 68 槽形下 OOB 必 panic
  （错位钉），换钉必要；新钉强度≥旧钉（旧钉只测场景面单点，新钉三端齐测）。
- 🟡 锚微漂：skip 链注记引「a.java:9289-9291 不读入」——实测读回循环 9291-9293、
  skip 语句 9295-9297（9289-9291 为循环前导区）。方向对、行号欠准（±2~6 行）。
- **判定：🟢 换钉成立可回指，skip 链语义清；🟡 锚行微漂（非阻断）。**

## §6 A2 重冻纯追加 + 红证据三件套 —— 🟢 绿（60/60 行级纯度+红可复现）

- [x] **红证据三件套（二轮修复的红=改动前测试红？）——是**（本复核实跑）：
  worktree 还原 4 源文件至 b7aeb2d 态（impl 旧）+ 保留 752e985 新断言 →
  `cargo test -p game-core --lib load_restores_script_flags` →
  `test result: FAILED. 0 passed; 1 failed` + panic 报文
  `assertion left == right failed: a.java:2038-2039 new boolean[levelScriptLines.length]
  left: 128 right: 68`——**改动前红**=新钉对旧实现（128 槽）红，报文含源锚可回指 ✓。
  GREEN 复现：还原752e985 后同测试 `ok. 1 passed` ✓。红→绿链闭合。
  附：e_quoted 的红=statediff 分歧面（一轮复核 A="abc" vs R=ARR[3] 实证）+
  金样漂移红（state_baseline --check 对旧金样红）——非单测红，但红源真实存在。
- [x] **A2 重冻 diff 核（二轮换新面 7 名+长度变长后旧行零动？）——60/60 零动**
  （本复核实跑2026-11-03）：双态生成 dump（b7aeb2d impl 态 / 752e985 impl 态，
  同 STATE_OUT 挂点 run_dumps 各 60 场景）→ 剔除 4 受影响面行
  （overlayText/itemStackTypes/itemStackUses/scriptLineFlags）→
  **60/60 场景非受影响行 sha 逐一相等**（含新面其余3 名 itemStackSize/
  itemUseTargetType/tutorialSeen 与旧 45 面行，PROBLEMS=无）。
  受影响4 面变化形态逐条对上设计：itemStackTypes/Uses `ARR[16]`→`ARR[32]`
  （head 16 项不变、sha 变=32 槽符号扩展长拼接）；scriptLineFlags `ARR[128]`→
  `ARR[68]`；overlayText `ARR[0]`→`null`、`ARR[n]`→`"…"` quote 形。
  **纯追加口径在此 commit 的正确形态=「4 面改形、其余行零动」**——实证成立。
- [x] **golden diff 结构纯**（240 行变更复核）：`git diff b7aeb2d..752e985` golden
  240 行增删全部为 sha256/bytes 值行（非值行增删=0）——60 场景键集/键序/ticks 不变，
  `line_order_freeze` 声明原样。形态=b7aeb2d 首冻同款合法重冻。
- [x] `--check` 全同实跑：`python3 tools/state_baseline.py --check` →
  `[GREEN] 金样逐字节等价：60 场景 sha 全同`（与 commit 声称逐字一致）✓。
- [x] **§8 抽验表核对**（spec v3:252-272 抽验 16 项，本复核抽3 项复测）：
  #9 scene_tests 行数 `wc -l` = 6,102（v3 记 6,098=本 commit +4 行注记/断言，一致方向）；
  #13 一次性切先例 `git log | grep 313375b` 实存 ✓；fields.py overlayText=FLD 062
  生成式复现 ✓（与 commit FLD062 注记吻合）。抽验表方法均可复现、无 C 级混入。
- [x] 计数核对：`cargo test -p game-core` 319 passed/0 failed（2 ignored 在案）；
  `cargo test -p game-oracle` 75 passed/0 failed——与 commit 声称逐字一致 ✓。
  ratchet 基线 total 924=binaries 逐项和 924 ✓（本复核求和核对），game_core(lib)
  321 与实跑 321（319+2i）一致，dump_face(test):1 新增在账——非阻断记账①收口 ✓。
- [x] **非阻断记账三件核**：①ratchet update=total 924 ✓（已收口，见上）；
  ②trace-contract.json dump_order 陈旧——实测 dump_order 仍止于 walkPathBuffer
  （49 项），现 push 面60 项——与 commit 自认「随 P1 主体件再生成」一致，欠账在案 ✓；
  ③dump_face push 级断言升级=后续件补——实测 dump_face.rs 仍仅名集在场断言
  （含 e_quoted 名），commit 自认属实 ✓。三件均非阻断、留痕如实。
- **判定：🟢 红→绿三件套真实（改动前红实证）、A2 重冻行级纯度60/60、
  golden 结构纯、计数/记账全对上。**

## §7 无夹带与红线核 —— 🟢 绿

- [x] `git show 752e985 --stat` = 6 文件（scene.rs 2 / scene_persist.rs 4 /
  fidelity_fix_tests.rs 6 / state_dump.rs 27 / rust-test-counts.json 17 /
  state-dump-golden.json 240）——无 gates/oracle 参考版/台账/original 触碰；
  测试计数与金样为受控基线变更且 commit 正文写明原因（AGENTS.md §9 合规）。
- [x] 实现变更均带源锚注记（【P1-a1 黄①/黄②】+ 行号）——三问预检（§4）：
  层=移植层长度/编码不齐（非投影错）；消费者=state_dump 倾印面与 save/载入路径；
  证据=A 级源锚（a.java:949-950/2038-2039/9291-9297 + TraceSink.java:44-64）。
- [x] worktree tracked 面干净（复核后 git checkout 还原逐字核实）。
- **判定：🟢 无夹带、证据链合规。**

## §8 总判

| # | 攻击点 | 判定 |
|---|---|---|
| 1 | quote() 逐字对拍（转义表+代理对） | 🟢 11/11 逐字节全同（真 Java 对拍）；🟡 金样/单测无代理对·空串样例钉 |
| 2 | null 分支（null≠""） | 🟢 None→"null"/Some("")→`""` 双源等价；🟡 区分无测试钉 |
| 3 | byte[32] 溢出面 | 🟡 容量修复对（A 级锚）；17/32 边界测试双缺+满栈行为分歧留注 |
| 4 | 68 槽语义 | 🟢 行为等价（全局表恒 68）；🟡「每层变长」表述失实 |
| 5 | 断言换钉 | 🟢 a.java:2038-2039 可回指+skip 链两端钉清；🟡 锚行微漂 |
| 6 | A2 重冻+红证据+§8 抽验 | 🟢 旧行零动 60/60+改动前红实证+记账三件如实 |
| 7 | 红线/夹带 | 🟢 绿 |

**总判：可推送（绿=lead 推送）——0 红 5 黄（全部非阻断）。**

红-1（overlayText 编码不等价）修复实证成立：quote() 等价编码经真实 TraceSink.java
对拍 11/11 逐字节全同（含代理对双转义镜像 charAt、五转义、null/空串分支）；
黄①②（长度语义）收口方向对且 A 级源锚可回指；A2 重冻经行级破坏性核销
（旧行零动 60/60）确认非破坏性；红证据三件套「改动前红」实证（新钉对旧实现
`left: 128 right: 68`）；一轮红项全部闭环。

🟡 非阻断随行项（可推送后清，建议归入 P1 后续件）：
1. **quote() 样例级回归钉缺失**：e_quoted 无单测，金样无代理对转义（0 行）/
   Some("")→`""`（0 行）——建议补 11 样例单测（本轮对拍表即测例）；
2. **null/空串区分无测试钉**（同源）；
3. **byte[32] 17/32 边界测试双缺**+满 32 溢出行为分歧（Java AIOOBE vs Rust
   静默丢，实际不可达 23+——建议显式注记+可达域 17 件入栈断言）；
4. **「每层变长」表述失实**：levelScriptLines=a.java:1124 一次性全局 68 行表，
   非逐层变长——建议更正 scene.rs:602 注记与 commit 口径为「=表长（恒 68）」；
   顺带 scene_persist.rs:233 默认构造 `vec![false; 68]` 硬编码未同步；
5. **锚行微漂**：skip 链注记 9289-9291→实为读循环 9291-9293+skip 9295-9297。

**完成判据（本复核实跑贴证）**：quote 对拍 11/11 diff 空（真 TraceSink.java 编译）；
state_baseline --check `[GREEN] 60 场景 sha 全同`；旧行零动核销 60/60 PROBLEMS=无；
改动前红 `FAILED. 0 passed; 1 failed`+报文逐字；绿 `ok. 1 passed`；
core 319/0+oracle 75/0；ratchet 924=924 求和核对；golden diff 240 行纯值行。
全部实跑如上。

---
*独立对抗复核员（二轮·新视角）· 2026-11-03 · remake-verifier 纪律*
