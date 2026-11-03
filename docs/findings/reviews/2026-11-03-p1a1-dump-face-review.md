# 对抗复核：b7aeb2d（DDDSA P1-a1 — state_dump 补面 6 组 7 push + A2 重冻结）

> 独立对抗复核员（逐提交纪律，remake-verifier 纪律）产出。
> 复核对象：commit `b7aeb2d`（**未推送**；绿 = lead 推送）。
> 核对清单：`docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md` v3 §8 抽验表。
> 盲区依据：B 轮 1 攻审 `docs/superpowers/specs/2026-11-03-dddsa-attack-verification.md`
> （state_dump 对 item_types/uses/stack_size、itemUseTargetType、tutorialSeen、scriptLineFlags、overlayText 全盲）。

## 0. 复核对象与方法

- [x] commit 元信息与 --stat 核对：`git show b7aeb2d --stat` = 3 文件
  （`crates/game-oracle/src/state_dump.rs` +9、`crates/game-oracle/tests/dump_face.rs` +23 新建、
  `data/status/baselines/state-dump-golden.json` 240 行值变更）——无其他夹带。
- [x] spec 对照：spec v3:215「P1 必清 6 组（道具栈三字段/itemUseTargetType/tutorialSeen/
  scriptLineFlags[]/overlayText）」= 6 组 7 名，与 commit「6 组 7 push」一一对应，无漏名/多名。
- [x] 盲区对照：B 攻审（`docs/findings/reviews/2026-11-03-dddsa-attack-verification.md`:28/55-57）
  点名 5 组全盲——本次补面覆盖其全部（B 原文名 `overlayKindItem` 为 Rust 侧名误写，
  commit 按「名=Java 字段名」红线改用 `itemUseTargetType` 并注明 Rust 真身，方向正确）。
- [x] B 行序冻结红线原文（attack-verification.md:55-56）：「行序追加=金样全红，属预期，
  需 lead 裁准重冻结（E3 先例照抄）」——全量 sha 换血本身不是违规，核的是行序与旧行字节。

## 1. 攻击点 1：伪数/锚 — 6 组名与 javap FLD 序名源对拍 ✅ 绿

- [x] **javap 名源**（`javap -p -cp analysis/rename-pipeline/renamed a`）：7 名全在场，
  逐一对上 `reference/src/deobf/a.java` 字段真名（itemUseTargetType:82 / overlayText:86 /
  itemStackSize:225 / itemStackTypes:226 / itemStackUses:227 / tutorialSeen:239 /
  scriptLineFlags:264）。**错名=0 漏名=0**。
- [x] **FLD 序对拍**（契约源 `reference/oracle/fields.py` 生成式映射，javap 字段声明序 0 基）：
  `python3 reference/oracle/fields.py <7 名>` 实测：
  itemStackSize=FLD201、itemStackTypes=FLD202、itemStackUses=FLD203、itemUseTargetType=FLD058、
  tutorialSeen=FLD215、scriptLineFlags=FLD240、overlayText=FLD062——与 commit 声称
  （FLD202/203/201/058/215/240/062）**逐一吻合，错序=0**。
- [x] **itemUseTargetType Rust 真身注记**：`scene.rs:4330` 注记实存
  （「itemUseTargetType = (byte)py_key（a.java:7732）：kind1 浮层图标/名的数据源」）+
  同函数写点 `scene.rs:4332 set_overlay_kind_item(t)`；命名台账 N-0417
  （f_byte_07→itemUseTargetType，conf 94）/N-0445（tutorialSeen，conf 90）在账。
  commit 注记**对**。
- 旁注（非本 commit 引入）：`fields.py` 打印尾 2 行杂散（`{}`/`}`）——bc4411e 已勘误
  「真实槽=411，历史 413=打印杂散」；7 名序号远离尾部，不受影响。
- **判定：🟢 绿**（名/序/注记三线全对，A 级证据链完整）。

## 2. 攻击点 2：语义 — overlayText 编码与 scriptLineFlags 位宽/序 🔴 红（overlayText）/ 🟡 黄（长度语义）

- [x] **overlayText dump 出的字节**：`e_byte_arr(text.as_bytes())`（state_dump.rs:153）=
  UTF-8 字节、每元素**有符号十进制** head + 符号扩展 long 大端拼接 sha（arr_str 头注）。
- [x] **与 Java String 倾印语义等价吗？——不等价（A 级实证）**：
  a 侧 `StateDump.java:74-75` 对 String 走 `TraceSink.quote()`（`"` 包裹 + `\uXXXX` 转义
  UTF-16 码元，TraceSink.java:44-64）；真 trace 实样
  `FLD 062 java/lang/String "\u4f60\u65e0\u6cd5\u6218\u80dc\u5b83"`（frames-fixture C-fx-item17a）。
  Rust 侧为 `ARR[n] sha=#… head=…` 字节组——**两种值形态结构不同**。用项目自研
  statediff 实测亲证：`overlayText: 首分歧 T1 A="abc" R=ARR[3] sha=#00 head=97,98,99`。
  另有 null/空串合流：Java null→`"null"`，Rust `unwrap_or("")`→`ARR[0]`（state_dump.rs:153）。
  state_dump.rs 模块契约自称「同名同格式…与 A trace 逐字段差分」——此面**破「同格式」半约**；
  编码裁定仅在 push 行注记，未见裁准记录/未入 statediff SKIP（对照 walkPathBuffer 先例
  =显式 SKIP+语义不同注记，state_dump.rs:9-10）。A2 冻结后再改编码=60 场景再重冻一次。
- [x] **scriptLineFlags 位宽/序**：逐元素 boolean（Java `boolean[]` head= true/false、
  sha=0/1 符号扩展 long）——`e_bool_arr` 与 Java boolean[] ARR 格式**同构**
  （entityVisible/Removed 同编码器已随 49 面金样对齐过）；序=下标序 ✓。
  **但长度语义存疑**：Java `new boolean[scriptLineCount]`（a.java:2038-2039，每层
  levelScriptLines.length 变长、<128）vs Rust GameScene `vec![false;68]` 默认
  （scene_persist.rs:233）/存读路径 `vec![false;128]`（scene.rs:602）——ARR[len] 对拍
  在多数层不齐（walkPathBuffer 同类问题，但未注记/未入 SKIP）。🟡
- [x] itemStackTypes/Uses 长度同类疑点：Java `new byte[32]`（a.java:949-950）vs Rust
  `vec![0;16]`（scene_persist.rs:110-111）——ARR[16] vs ARR[32] 系统性不齐（移植层旧账，
  非本 commit 引入，但面值语义对拍终会撞上）。🟡
- **判定：🔴 overlayText 编码与 Java 倾印不等价（statediff 亲证首分歧）；🟡 scriptLineFlags/
  itemStack 长度语义未对齐亦未注记**。名集/格式外壳无误，但「语义等价」宣称不成立。

## 3. 攻击点 3：A2 重冻结 — golden diff 破坏性验证 🟢 绿（结构纯+行级 60/60）

- [x] **diff 形态**（`git show b7aeb2d -- state-dump-golden.json`）：删除 120 行、
  追加 120 行，**全部为 `sha256`/`bytes` 值行**（非 sha/bytes 行增删=0）；
  60 场景键集/键序/ticks 全不变，`line_order_freeze` 声明原样。
- [x] 全部 60 场景 sha 必然换血（每 tick 块新增 7 行 ⇒ 整文件字节变）——这正是 B 红线
  55-56「行序追加=金样全红，属预期，需 lead 裁准重冻结」所指的**合法全量重冻结形态**
  （E3 先例同款）。"旧行 sha 不变"在整文件 sha 层面不可能成立；**行级「旧行不变」**
  需以「重生成 dump→剔除 7 新名行→sha 对上旧金样」核销（见 §4/§5 实跑）。
- [x] 代码层旁证：state_dump.rs diff 仅 +9 行（walkPathBuffer 之后**尾插** 7 push+2 注释），
  既有 push 行零改动——「尾插保序（既有名序零移位）」成立。
- [x] **行级破坏性核销（决定性，本复核实跑 2026-11-03）**：重生成 60 场景 dump
  （run_dumps rc=0）→ 按 `^(itemStackTypes|…|overlayText) = ` 剔除 7 新名行 →
  **60/60 场景 sha 逐一等于旧金样（b7aeb2d^）**：旧行字节零变动、行序零移位、
  新增恰 7 行/tick（共剥 616,378 行 = 7×88,054 tick 块）；新 dump sha 亦 60/60 对新金样。
  「旧行 sha 不变」在**行级重构口径**下成立 ✓。
- **判定：🟢 结构纯重冻+行级字节纯度双双实证（60/60）**。

## 4. 攻击点 4：红证据三件套 ✅ 绿（测试真伪有保留意见 → 黄注）

- [x] **测试读真伪**（`crates/game-oracle/tests/dump_face.rs`，23 行）：`include_str!` 取
  state_dump.rs 源文，截 `pub fn dump_scene` 之后的体，逐名断言 `body.contains("\"name\"")`。
  **断言是真名集在场性测试**（缺名必红，已实证），但为**源文文本在场**而非 push 语句在场：
  注释中若出现引号包名会假阳性；且未断言 push **序**（报文却称「push 序在场性」）——
  严格 push 序解析器项目里已有（fields.py:68-72 `dump_order()` 正则），本测试未复用。🟡 保留意见。
- [x] **GREEN 实跑复现**（worktree @b7aeb2d，2026-11-03）：
  `cargo test -p game-oracle --test dump_face` →
  `test result: ok. 1 passed; 0 failed; 0 ignored` ——与 commit 声称逐字一致 ✓。
- [x] **RED 实跑复现**（还原 state_dump.rs 至 `b7aeb2d^` 态后跑同测试）：
  `test result: FAILED. 0 passed; 1 failed` + panic 报文
  `P1-a1 补面缺名：itemStackTypes（dump_scene push 序在场性——A2 冻结对象）`
  ——与 commit 声称②**逐字一致**（首缺名=itemStackTypes=数组序首名 ✓）。
  复现后已 `git checkout b7aeb2d --` 还原，worktree 干净 ✓。
- [x] 计数核对：`cargo test -p game-oracle` 合计 **75 passed / 0 failed**（74+1 ✓）；
  `cargo test -p game-core` **319 passed / 0 failed**（另 2 ignored+3 ignored=在案豁免）。
- **判定：🟢 红→绿三件套真实、报文逐字可复现；🟡 测试强度仅源文在场性（注释假阳性面/
  无序断言）为非阻断保留意见**。

## 5. 攻击点 5：红线 — 名集相等守卫 / --check 60 / 夹带 ✅ 绿（四线全绿）

- [x] **名集相等（statediff 守卫）在场**：`tools/statediff.py:87-95`——任一 tick 两侧名集
  不等即 `[RED] …名集不等` + exit 1（2026-10-16 bc4411e 修复交集静默绿）。
  **三态合成实测**（本复核 2026-11-03）：名不等→`[RED] T1 名集不等…漏字段=红` exit 1；
  名等值等→`分歧字段数=0` exit 0；值分歧→逐字段首分歧输出 exit 1。守卫**真**，非摆设。
- [x] **--check 60 全同实跑**：`python3 tools/state_baseline.py --check`（本复核实跑）→
  `[GREEN] 金样逐字节等价：60 场景 sha 全同` exit 0 ——与 commit 声称逐字一致 ✓。
- [x] **无夹带**：`git show --stat` = 3 文件（state_dump.rs +9 / dump_face.rs +23 /
  golden 240 行值变更）；无 gates/oracle/参考版/台账触碰；worktree tracked 面干净。
- [x] 计数：game-oracle 75/0（74+1）✓ core 319/0 ✓（实跑核对，见 §4）。
- [x] **行级「旧行不变」核销**：✅ 已收口（见 §3 末条：剔除 7 新名行→60/60 sha 对旧金样）。
- **判定：🟢 守卫/全同/夹带/行级四线绿**。

### 5.1 附带发现（非阻断，建议 lead 顺手清）

- [x] `gates/cli.py status` 全量实跑（本复核）：13 门全 ✓ PASS（含 rust-tests release 全绿、
  oracle 6/6 确定性/等价 PASS、reference-regression PASS）——commit「gates check 全绿」宣称成立。
- 🟡 **test-ratchet 门自报**：`新增二进制待 ratchet-update: {'dump_face(test)': 1}`
  （total 924 vs 基线 906，无回退、非阻断）——本 commit 未顺手升 ratchet 基线，留给后续步骤；
  按惯例基线更新须单独 commit 说明，可接受，但勿忘。
- 🟡 `data/status/baselines/trace-contract.json`（bc4411e 产出的 A2 行序冻结声明伴随件，
  `fields.py --contract` 生成）**已陈旧**：dump_order 仍 49 项止于 walkPathBuffer，
  不含 E3 四面与本次 7 面（现 push 面 60 项）。E3（2026-11-02）同样未重生成——
  漂移非本 commit 引入，但本 commit 又续了 7 项。无门禁消费它，故不红；
  建议随 P1 批次重跑 `python3 reference/oracle/fields.py --contract` 或显式声明废弃。
- 🟡 `scene.rs:4330` 注记内锚 `a.java:7732` 疑漂移（实测 `itemUseTargetType=(byte)var1`
  写点≈a.java:7738，7732 为 itemStackUses 行）；旧注记，非本 commit 引入。
- 🟡 statediff **全 trace 口径注记**（bc4411e 起既有）：A 侧 oracle trace 每 dump 块
  411 名全字段 vs Rust dump 60 名——名集相等守卫在真实全 trace 对上永不可能过
  （合成实测 A-only 名即红）；P1-a1 判据「statediff 名集相等」实义=**命名契约层**
  （dump 名集⊂Java FLD 名集，守卫在场），非全 trace 实跑绿。工具面口径欠一句
  「A 侧需面过滤」的使用注记，建议随 P1 差分实跑补。

## 6. 总判

| # | 攻击点 | 判定 |
|---|---|---|
| 1 | 伪数/锚（名/FLD 序/Rust 真身注记） | 🟢 绿（javap+fields.py+trace-contract 三源对拍全中） |
| 2 | 语义（overlayText 编码 / scriptLineFlags 位宽序） | 🔴 overlayText 与 Java String 倾印**不等价**；🟡 长度语义两处未注记（位宽/序本身 🟢） |
| 3 | A2 重冻结破坏性验证 | 🟢 绿（结构纯+行级 60/60 sha 对旧金样，纯追加实锤） |
| 4 | 红证据三件套 | 🟢 绿（RED/GREEN 报文逐字复现；🟡 测试强度=源文在场性） |
| 5 | 红线（名集守卫/--check 60/夹带） | 🟢 绿（守卫三态实测真；--check 逐字复现；3 文件无夹带） |

**总判：需修复（小修，一处红）——暂不推送。**

唯一红项 = **overlayText 编码裁定**：`e_byte_arr(text.as_bytes())`（UTF-8 字节组）与
裁判面 Java String 倾印（`TraceSink.quote`：`"` 包裹 + `\uXXXX` UTF-16 转义，
StateDump.java:74-75/TraceSink.java:44-64）**结构不可比**（statediff 亲证首分歧
`A="abc" R=ARR[3] sha=#00 head=97,98,99`）；且 `unwrap_or("")` 把 Java `null` 与
`""` 合流为 `ARR[0]`，丢一层状态区分。A2 冻结已把它烙进 60 金样——**推送前二选一落书**：

1. **对齐**：改 quote() 等价编码（UTF-16 码元 `\u%04x` 小写转义，算法 20 行内），
   未推送窗口内一次 A2 重冻即可（行级纯度方法本复核已备）；或
2. **显式裁定**：lead 落书「字符串面=UTF-8 字节组=终案」+ 模块「已知不对齐项」注记
   （walkPathBuffer 先例照抄，state_dump.rs:9-10）+ statediff SKIP/值映射处置 +
   null/空区分（如 `null` 独立面值）。

🟡 非阻断随行项（可推送后清）：scriptLineFlags 长度语义（Rust 68/128 固定 vs Java
`new boolean[levelScriptLines.length]` 每层变长，a.java:2038-2039）与 itemStackTypes/Uses
（Java byte[32] a.java:949-950 vs Rust vec![0;16] scene_persist.rs:110-111）在面值对拍
终会 ARR[len] 撞车——建议同批补「已知不对齐项」注记或开移植修正条目；
test-ratchet 待 ratchet-update（dump_face(test)）；trace-contract.json 陈旧重生成；
dump_face 测试可复用 fields.py `dump_order()` 正则升级为 push 级断言。

**完成判据（本复核）**：`python3 tools/state_baseline.py --check` → `[GREEN] 60 场景
sha 全同`；`cargo test -p game-oracle --test dump_face` → `ok. 1 passed`；
RED 复现 `FAILED. 0 passed; 1 failed`+报文逐字；`gates/cli.py status` 13 门全 ✓；
行级剔除核销 60/60。全部贴出如上。

---
*独立对抗复核员 · 2026-11-03 · 逐提交纪律（remake-verifier）*
