# 2026-11-03 P1-a1 黄五连清账 对抗复核（commit cb710f4，逐提交纪律）

> 角色：独立对抗复核员（remake-verifier 纪律）。复核对象：`cb710f4`（未推送，绿=lead 推送）。
> 背景：二轮复核（[2026-11-03-p1a1-round2-review.md](2026-11-03-p1a1-round2-review.md)）判 5 黄随 P1-a2 清，成员提前清了。
> 判定口径：红=需修复才可推；黄=留观/瑕疵不阻断；绿=通过。
> 实跑口径：裸 worktree（/tmp/wt_cb710f4 @ cb710f4）复现钉测；主工作区跑全量与金样（裸树缺生成夹具，环境性红不计提交缺陷——见 §5）。

## 0. 总判

**可推送（绿=lead 推送）——0 红 1 黄（攻击点②推理链表述，非阻断）。**
五黄清账逐项核销：①四测真且 worktree 复现绿、②边界钉真+BUG-050 全栏（惟"22 上限/
23+ 不可达"推理链与 a.java 锚不合——结论幸存）、③证伪更正合规双留痕、④skip 锚
9291-9297 双处实测吻合、⑤无夹带+ratchet 929 对账平+金样 60 全同实跑。
建议随 P1-a2 带一句口径更正（22=use-count 槽域；API 上限 24/流程上限 15），
勿改本 commit 历史。

## 1. 攻击点① e_quoted 四测 —— 🟢 绿（断言逐字真、镜像 charAt 成立；小注：\r 臂未钉）

- **读测试逐字**（state_dump.rs:187-209，worktree 4/4 实跑 ok）：
  1. `surrogate_pair_two_escapes`：`e_quoted("x", Some("\u{1F600}")).value == "\"\\ud83d\\ude00\""`。
     真伪核：U+1F600 的 UTF-16 = **D83D DE00**（(0x1F600-0x10000)→高 0xD83D/低 0xDE00 计算复核）；
     `e_quoted` 走 `encode_utf16` 逐单元 `\\u{:04x}`（小写）→ `"\ud83d\ude00"` 与断言逐字同。
     **镜像 charAt 成立**：TraceSink.java:44-64 quote() = `for i<s.length(); s.charAt(i)` 逐
     UTF-16 单元、`String.format("\\u%04x", (int)c)` 小写——代理对双 `\u` 转义两侧逐字同（A 级对拍）。
  2. `empty_string_is_quoted_not_null`：`Some("")→"\"\""` 逐字钉 ✓（实现：循环零次+首尾引号）。
  3. `null_is_literal_null`：`None→"null"` 逐字钉 ✓（= StateDump.java:67-68 value() null 支无引号字面）。
  4. `five_escapes_and_non_ascii`：`a"b\c\nd\te中 → "\"a\\\"b\\\\c\\nd\\te\\u4e2d\""` 逐字钉 ✓
     ——五形态 `\" \\ \n \t \u4e2d` 与 e_quoted 转义臂逐一核过，断言真。
- **空串/null/五转义逐字钉**：四断言全为字面量相等（非 includes/正则）——钉在 ✓。
- **worktree 复现**：`cargo test -p game-oracle --lib p1a1_quoted` → `ok. 4 passed; 0 failed`（本复核实跑）。
- 小注（不阻断）：转义臂 `\r`(0x0D) 与 `<0x20` 其余控制符未钉（四测盖 7 臂中 6 臂）；
  二轮建议 11 样例表，交付 4 例——分支面基本闭合，\r 臂留待后续件。

## 2. 攻击点② byte[32] 边界 —— 🟡 黄（结论安全，"22/23+"推理链与 a.java 锚不合）

- **22 从哪来**：**型域映射推理，非游戏流程实测**。a.java:7711-7725 `itemTypeToStackIndex`：
  default=`var0-13`（13..=34→idx 0..=21，@7721）+ case 85→20 / 86→21（@7714-7717）；
  itemUseCounts 22 槽（a.java:944）。
  测试实证的是 **API 面**：直调 `add_item_to_stack(13..=34)` 22 件→size 22（断言过）。
- **推理链核（a.java 锚）**——"33 件不可经 add 达成=23+ 不可达"中间环节**不成立**：
  - `findItemStackIndex`（a.java:7664-7675）按**精确 type** 匹配（`itemStackTypes[var3]==var1`），
    85/86 与 33/34 是不同栈位——同一 API 面再调 `add_item_to_stack(85/86)` 可达 **24 件**，
    "**23+ 不可达**"在其自身实证口径（直调 API）下即被 a.java:7714-7717 特例反驳。
  - 反向：**游戏流程可入栈型集**只有 pickupItemType（a.java:7534-7555，case 85/86 @7552-7553）case
    13..=25 ∪ {85,86}=**15 型**（26-28=钥匙计数、29-32=属性 buff、33/34 不在 switch）
    ——"可达上限 22 件"游戏流程不可达，实为 use-count 槽数非栈位数。
  - **结论幸存**：两种口径下 33 件（byte[32] 溢出）均不可经 add 达成（API 上限 24、
    流程上限 15，皆 <32）——BUG-050"潜在/金样未触"定性成立；测试各断言本身为真
    （22 件确至 22、置 32 后 add 确不动=`n < len` 守卫 a.java:7731-7733 直写对拍成立）。
- **BUG-050 条目规范性（docs/bug-ledger.md）**：现象（Rust 静默吞 vs Java AIOOBE）✓/
  复现（测试名）✓/影响（含"正常流程栈深≤22"——宽松但真，实为≤15）✓/trace id 栏
  **N/A+理由**（静态对拍无运行时 trace）✓——四栏全、分类自标"保真缺口"诚实。
  注记：①a.java:7721-7734 锚覆盖决定行（7731-7733 直写）但起点早 6 行（方法实为
  7727 起）；②"Stage A 不修（D4）"——D4 文义管"原版 bug 保留"，本条是 Rust 分歧，
  借 D4 免修属政策延用（条目已诚实自标缺口，可接受）。
- 判定：🟡——"22 槽可达上限实证/23+ 不可达"表述与 a.java 锚链不合（上限口径混用+
  85/86 漏计），溢出结论与条目登记无误。

## 3. 攻击点③ 证伪更正 —— 🟢 绿（原地日期更正在、历史主张可见、留痕双落）

- scene.rs:602-604：`【2026-11-03 更正】"每层变长"表述证伪：levelScriptLines=全局 68
  行表（a.java:1124）非逐层`——日期更正 ✓；**历史主张以引语保留**（「每层变长」字样
  在场、读者可见错在哪）✓；原锚 a.java:2038-2039 标"锚保真"留痕 ✓（a.java:2038-2039
  实测=`scriptLineCount=levelScriptLines.length`+`new boolean[scriptLineCount]`，吻合）。
- a.java:1124 实测核对：`this.levelScriptLines = new String[]{…}` 构造器一次性全局表 ✓。
- scene_persist.rs:233：`vec![false; 68], // levelScriptLines 全局 68 行表（a.java:1124）
  ——硬编码 68=正确实现留痕（2026-11-03 复核实证）`——**留痕注释落了** ✓。
- 全库 grep「每层变长」：仅存两处更正注记内引语，无未更正残留 ✓。
- fidelity_fix_tests.rs:161-165 同步换注（旧 128 错位钉叙述保留于更正语境内）✓。

## 4. 攻击点④ skip 锚更正 —— 🟢 绿（9291-9297 双处，a.java 实测吻合）

- a.java（reference/src/deobf/a.java）实测：读回循环 **9291-9293**
  （`for var11<scriptLineCount: scriptLineFlags[var11]=readBoolean()`）、
  skip **9295-9297**（`if (scriptLineCount<128) skip(128-scriptLineCount)`）
  ——**9291-9297 正确**（9289-9291 旧锚落在循环前导区，确错）。
- 双处核对（fidelity_fix_tests.rs）：:151 注释 `（a.java:9291-9297——2026-11-03 锚行更正）`
  + :168 断言报文 `（a.java:9291-9297 不读入）`——双处全改 ✓，与二轮实测行号逐字同。
  全库 grep「9289」已零残留（旧锚清干净）✓。
- 范围外观察（不计入本批）：同链 `9285-9291` 旧区间仍存 5 处（scene.rs:600/4878、
  save.rs:15/54/138——描述同一"读 Boolean+skip"语义，实为 9291-9297），属同类漂移
  残留；本批声称面即"双处"属实未虚报，建议 P1-a2 随行扫齐。

## 5. 攻击点⑤ 夹带检查 —— 🟢 绿（--stat 面净、ratchet 929 对账平、金样 60 全同实跑）

- **--stat 仅本批面**：6 文件=scene.rs（更正注）/scene_persist.rs（留痕注）/
  fidelity_fix_tests.rs（双锚更正+边界测）/state_dump.rs（4 quote 测）/
  rust-test-counts.json（ratchet）/bug-ledger.md（BUG-050）——与五攻击点逐一对上，
  无 gates/oracle 参考版/original/台账夹带 ✓。基线随批变更=受控且 commit 正文写明
  原因（AGENTS.md §9 合规）✓。
- **ratchet 929 对账**：worktree 实跑 `gates cli.py ratchet` → `[✓ PASS] test-ratchet
  — total=929(基线929) 无回退`；口径复原 924（752e985 总额，已吸收 b7aeb2d
  dump_face(test):1）+4 quote 测+1 边界测=929 ✓——"新增四测+dump_face"入账无重无漏。
  game_core(lib) 322=320 过+2 ignored 对上；game_oracle 79/0=--list 82−3 ignored ✓。
- **计数对拍**（主区实跑）：core `ok. 320 passed; 0 failed; 2 ignored`（=319+1 边界测，
  与 commit 逐字同）；oracle 4/4 quote+全包 79 过 ✓。
- **金样 60 全同实跑**：`python3 tools/state_baseline.py --check` →
  **`[GREEN] 金样逐字节等价：60 场景 sha 全同`**（主区实跑，与 commit 声称逐字同）✓。
- 方法注记：裸 worktree（无生成夹具）下 fixture/logo/t544 等 9 测与金样 30 面为
  **环境性红**（缺 target 生成物），主区同跑全绿——非提交缺陷；worktree 只用于
  独立钉测复现（4 quote+1 边界=5/5 过，恰为本批新增面）。
