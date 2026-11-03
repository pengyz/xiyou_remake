# P1-a2 批1 独立对抗复核（commit e19d08c）

> 复核员：remake-verifier 独立对抗复核（逐提交纪律）
> 对象：commit `e19d08c`（DDDSA P1-a2 批1：fx-item13 三件套+拾取断言+三残账清），未推送
> 规格：docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md §8 抽验表
> 纪律：AGENTS.md 证据分级 + remake-verifier 硬性禁止（不调阈值/不扩白名单/未运行不声称）
> 复核时间：2026-11-03

## 0. 复核范围与方法

- 对象：`e19d08c`（未推送）`git show --stat` = 7 文件 464+/4-：save.rs、mov_route_scene.rs、
  fixture_frames.rs、fx-item13.{txt,preset,frames.txt}、state-dump-golden.json——与申报面逐一对上。
- 实跑清单（全部本复核现场执行，非转述）：
  1. `python3 tools/frames_fixture.py gen …/fx-item13.txt`（A/B/C 三方重跑+重抽取）→ `git diff` 逐字节比；
  2. 帧链 sha 独立重算（python hashlib）；源 trace sha256 实测 A/B/C 三份；
  3. `cargo test -p game-oracle --test fixture_frames fx_item13`；
  4. `target/debug/deps/arm_coverage-*`（TDD 红件编译产物）跑红清单 + 藏三件套复现 RED=15；
  5. `python3 tools/state_baseline.py --check`（金样 61 全同）+ `python3 gates/cli.py check`（总闸）；
  6. deobf a.java 逐行 sed 取证（9244-9320 / 7534-7650 / 7700-7745 / 906-945）；
  7. golden json 新旧对账（HEAD~1 逐键比）；repo 级 grep 9285/9253 全量扫。
- 规格核对面：spec §8 抽验表 16 项 + §7-Q3 臂面 21 组清单。

## 1. 攻击点①夹具真伪（fx-item13 三件套）—— 🟢 绿

**三件套齐**：fx-item13.txt（24 行剧本，# ticks: 400）+ fx-item13.preset（MOT_IF/MOT_L0/SKY_WAR
三段，floor=0x34=52、x/y=3/7、facing=2 与剧本注一致）+ fx-item13.frames.txt（400 行帧记录）。

**400 帧链 sha 实跑核对** ✓：
- 帧记录 400 条、tick 1..400 连续；帧链独立重算
  `sha256("\n".join(shas)) = 350bd8b8f404856323cb4047e901fb5c6051e8d5f2560bda754d87a28b4b4f4b`
  = 头注声明值，逐字吻合；
- 源 trace `analysis/frames-fixture/runs/A-fx-item13/trace.txt` sha256 实测
  `a50efb53a881be687cd61f9f5061975102ce28a3871754b575492149bbf6df49` = 头注声明值 ✓；
- run-info：ok=true，ticks=400（max-ticks 收尾）✓。

**"A/B/C 三方 sha 一致"=真 oracle 三方对拍实证** ✓：`frames_fixture.py gen` 现场重跑
（本复核独立执行）：A=JAR 原字节、B=rename-pipeline/renamed（T-变换）、C=build/deobf
三变体各自独立跑 oracle.host.Runner，输出
`三方 trace sha256：A=a50efb53a881…，B=a50efb53a881…，C=a50efb53a881…`——
工具只在三方 sha 全同才写夹具（`len(set(shas.values()))!=1 → 不写夹具`，frames_fixture.py:230-232）。
**重生成逐字节等值**：gen 后 `git diff` 空、`git status` 净——夹具非手造，可由
`python3 tools/frames_fixture.py gen crates/game-oracle/tests/fixtures/fx-item13.txt` 一键复现。

**剧本语义**（对拍 a.java）：boot 槽载入 floor52 (3,7) 朝下→T300 -2 南步入 (3,8) 踩型13 实体格
→pickupItemType 臂→addItemToItemStack→描述 overlay→T380 -6 吞键收尾；INPUT 列与脚本事件
逐项一致（gen 自带 check() 校验，exit 0）。

## 2. 攻击点②臂覆盖（pickup 13..=25|85|86）—— 🟢 语义绿 / 🔴 新锚错引（见 §6 diff-case 2026-11-03-P1A2-B1-01）

**臂语义对拍 a.java pickupItemType（7534-7649）实测** ✓：
- `switch (var1)` case **13,14,15(fall-through),16..25,85,86 → addItemToItemStack(var1)**（7554）
  = 声称臂"13..=25|85|86"逐字吻合（15 带 tutorialSeen=true 副作用后 fall-through 入组，
  语义细微差已在此注记；fixture 只测首臂 13 不涉 15）；
- 相邻臂核对无误：26/27/28=黄/红/蓝钥匙计数、29/30=攻/防、31/32=加血、
  33..=40|79|80=tryEquip、default=return（与 spec §7-Q3 的 11 臂组划分一致）。

**13 型=什么道具**（a.java:906 itemDescriptions[0] + 944 itemUseCounts[0] 实测）：
「能看破敌人底细，显示敌人详细信息…[使用次数：无限]」= 查看敌人信息道具（怪鉴）；
use 臂=OpenPanel EnemyInfo（ITEM_EFFECTS 13 行）与描述自洽；uses=itemUseCounts[0]=**-1**
= 测注"uses=ITEM_USES[0]=-1" ✓（scene.rs:4320 ITEM_USES[0]=-1 两源对拍一致）。

**floor52 (3,8) 拾取链**：探针声称 type=13 x=96 y=256=格(3,8)——96/256=3×32/8×32 算术自洽；
行为链由金样端到端钉死（Java 400 帧 ↔ Rust 400 帧逐帧 sha 等值 + 测试断言
item_stack_size==1/item_types[0]==13/find_item(13)==0/终态 (3,8)）✓。

**"sprite 探针实证后删"=临时件清理干净** ✓：`git show --stat` 无任何探针文件；
worktree `git status` 无探针残留（仅 android/build 与未跟踪 docs）；fixtures 面内无临时件混入。

**【红】addItemToItemStack 锚 7721-7734 错引**：deobf a.java 实测该方法=**7727-7740**
（`private void addItemToItemStack` 7727 行起、7740 行止；7721-7734 落在
itemTypeToStackIndex 尾部+方法前半）。错引出现 3 处（commit message、
fx-item13.txt cover、fixture_frames.rs 测注）——详见 §6 diff-case 01。

## 3. 攻击点③机制发现（61 vs 60 增 1）—— 🟢 绿（纯追加实证 + 机制=设计内，发现已入账）

**61 vs 60 是否纯追加（旧行零动核）** ✓✓：HEAD~1 vs HEAD 的 state-dump-golden.json
scenarios 逐键对账（python 实跑）：`changed old rows: []、removed: []、added: ['fx-item13.state']`
——60 旧行 sha/bytes/ticks 零改动，恰 +1 行（fx-item13.state sha=3e6ca8fa…、
bytes=389720、ticks=163），61=60+1 纯追加。ratchet 无回退成立。

**机制判漂移→重冻：发现还是缺陷？** ——**发现（设计内机制按预期工作），非缺陷**：
`tools/state_baseline.py --check` 实测逻辑（main 120-137 行）：金样外新增场景
→ `+新增 {k}` 判 drift → RED return 1；字节漂移 `~字节漂移`、缺行 `-缺失` 同判。
即"新夹具=新 .state 入冻结集判漂移"是 **A2 行序冻结协议的守卫面**——新场景必须
显式入账冻结才放行，防新 dump 悄悄混基线。应该如此（ratchet 只升不降 + 一切基线
变更显式化），初提 FAIL 是机制生效的证据而非异常。

**发现入账=注记在** ✓：commit message 显式注记（"首提闸红更正入账：state-baseline
初提 FAIL=+新增 fx-item13.state…重冻一次补入后复绿（机制本身=夹具扩面必经冻结面，
非行为差）"）——现象、原因、处置三要素齐。【备注】注记仅落 commit message；
若后续同类扩面频发，建议入 docs/knowledge（不阻本批）。

## 4. 攻击点④三残账清

### ①9285 同族锚 4 处—— 🟡 黄（3 处实改属实、1 处点名项漏清 + 全序锚仍欠 + 真值核注记错行）

- **实改 3 处属实** ✓：mov_route_scene.rs:11/:417、save.rs:108 三处
  `9285-9287→9291-9297`，与 p1a1-sweep-review §2 点名清单逐一对上（该 3 行确属
  同族 -6 漂移错锚）；save.rs:4 `9253-9287→9253-9297` 亦为真实改进。
- **【黄】点名第 4 处漏清**：sweep-review §2 点名残留 4 处含
  `docs/spec/refactor-behavior-matrix.md:59`（9285-9291）——本批 --stat 未含该文件，
  **实测现存**："（a.java:9285-9291/9648-9673）"仍在 matrix.md:59。commit 声称
  "9285 同族锚 4 处清（复核点名）"与点名清单不一致（点名 4 清 3，另加 1 处未点名的
  save.rs:4）——声称-内容不符（gotcha_commit-claims-must-match-content 族）。
  repo 级 grep 9285 全量扫：仅剩 matrix.md:59 一处真锚（snapshot.json 两处=sha256
  哈希子串伪命中，前轮已注明不计）。
- **【黄】"全序"仍欠**：save.rs:4 表列 21 行字段（行 20 itemStackSize+N×(type,uses)=
  a.java:9300-9308、行 21 56×(short+byte[])=9310-9317），"字段序（a.java:9253-9297）"
  只覆盖至旗标段 skip 尾，行 20-21（9300-9317）不在区间内——"全序"应为 9253-9317，
  新锚仍欠 20 行（同族残）。
- **【黄】真值核注记行号错**：commit message"真值核=deobf a.java:9283-9299 实文
  （旗标读循环 9290-9292+skip 9294-9296）"——a.java 实测读循环=**9291-9293**、
  skip=**9295-9297**（与 p1a1 两轮复核实测逐字同）；commit 括注两处各差 1 行。
  **代码内新锚 9291-9297 本身正确** ✓（save.rs 表注/解析注/mov_route_scene 两处
  与 a.java 实文吻合）——错的是 commit message 的真值核括注。

### ②复用面 21 组映射表落测 doc—— 🟡 黄（表落位=可回指 ✓，但 2 处复用错标）

- **落测 doc 且可回指** ✓：映射表落 fixture_frames.rs:604-611 测注（FX-ITEM13 测试
  文档串），逐臂→夹具名可回指；行数核对=pickup 11 组（13..=25|85|86、26、27、28、
  29、30、31、32、33..=36|79、37..=40|80、default）+ use 10 行（13/14、16|17、18、
  19、20、21、85、86、default 覆盖 ITEM_EFFECTS 10 行）——与 spec §7-Q3"臂面 21"
  及 arm_coverage 编译产物内嵌期望表（21 组）三方一致。
- **【黄】复用错标 2 处**（实测对拍）：
  1. "use 13/14（preset-floor5 复用）"——preset-floor5 剧本实测 cover=
     "**道具14 全链**（mode12 记录浏览器）"：只覆盖 pickup14+use14（RecordBrowser）；
     **use 13（EnemyInfo 面板）无任何夹具覆盖**（fixtures/preset_cases 全库 grep
     EnemyInfo/menu5=0 命中），arm_coverage 期望表亦报"use 13 → fx-item13-use"缺
     ——应为"use 13=净补、use 14=preset-floor5 复用"。
  2. "21（preset-item21/2122 复用）"——preset_cases.rs:931-936 实测：
     preset-item2122=**消耗臂 20 拦截路径**（MOT_L0 栈注入 [20,21]，镜像格 interact
     拦截不消耗），preset-item21=消耗臂 21 正路径——preset-item2122 应归"20"行，
     现错挂"21"行。
- 【观察·不计】spec §7-Q3 复用清单点名 fx-boss67a，映射表未收（其 cover=击杀
  spawnBossEvent 掉落实体 DES 态，未见21 臂组内拾取/use 面）——spec 清单口径松、
  表未收不算错，登记备查。

### ③型 32 显式递延—— 🟡 黄（commit 注记在，表内未标）

- **显式递延注记在** ✓：commit message"型32『出现未拾』销账：pre-final-audit:122 源
  待定位——显式递延下批"（来源核对：docs/spec/pre-final-audit.md:122 确载
  "道具 32 长寿丹（🎨 出现未拾）…需补拾取/重入夹具"、type-closure.md:85 同载）。
- **【黄】映射表内未标递延**：fixture_frames.rs 映射表 pickup 列"32"无任何
  递延/销账标记，计划 P1-a2 行仍写"+型 32 销账"——仅读仓库文档者看不出 32 已递延
  （归宿=commit message 单点）；建议表内补"（递延下批·pre-final-audit:122）"。

## 5. 攻击点⑤红证据与夹带

### 5.1 arm_coverage 缺臂清单实录→补臂绿—— 🟡 黄（RED=15 复现属实，"实录"无落档）

- **RED=15 复现一例 ✓（实跑）**：TDD 红件编译产物仍在
  `target/debug/deps/arm_coverage-f75599c6270e1438`（源=crates/game-oracle/tests/arm_coverage.rs
  已删，Nov 3 20:25 构建）。现场执行：
  - 现状（fx-item13 三件套在位）：缺臂 **14** 项（pickup 26/27/28/29/30/31/32/33..=36|79/
    37..=40|80/default、use 13、use 21、use 85、use default）——fx-item13 臂已补 ✓；
  - **藏三件套复现 RED=15**：mv fx-item13.* 暂移后实跑 → 缺臂恰 **15** 项
    （首项"pickup 13..=25|85|86 → fx-item13"在列）→ 恢复三件套后 git status 净。
    "RED=15 缺臂"声称属实、机器可复现。
  - 缺臂清单语义抽查："pickup 26→fx-pickup-26 …"逐臂→期望夹具名成表（内嵌期望表
    21 组，与 spec §7-Q3 一致）✓。
- **"补臂绿"口径**："1 passed 4.07s 全帧比对"=fx_item13 测（本复核实跑 1 passed 3.88s ✓）；
  arm_coverage 本身此刻仍红（14 缺，按"全 21 绿时落 arm_coverage 强制测"递延）——
  措辞可读通，不判虚报。
- **【黄】"清单实录"无落档**：清单只存在于**已删源码**的编译产物 panic 输出；
  docs/、findings/、tests/ 全库 grep 无任何落档记录（arm_coverage.rs 未入库）。
  声称"清单实录"但仓库无可回指实录件——TDD 红证据归宿不足
  （AGENTS.md §7"产出的归宿是仓库"）；建议随下批把 arm_coverage.rs 或红清单
  落 findings/（或以 #[ignore] 测入库）。

### 5.2 夹带检查—— 🟢 绿

- `git show --stat` = 7 文件恰为申报面（3 代码+3 夹具+1 金样），无 gates/oracle 参考版/
  original/台账/门禁阈值夹带；夹具目录无临时件混入。
- 基线变更=state-dump-golden.json +1 行（申报"金样集扩面"+原因已写 commit，
  AGENTS.md §9 合规）；旧 60 行零动（§3 实证）。

### 5.3 金样 61 全同实跑—— 🟢 绿（双实跑）

- `python3 tools/state_baseline.py --check`（本复核实跑，exit 0）：
  `[GREEN] 金样逐字节等价：61 场景 sha 全同`——"60+1 场景全同"声称属实。
- `python3 gates/cli.py check` 总闸实跑（exit 0）：**PASS=13 FAIL=0 SKIP=0**
  （original-integrity / assets-manifest / reference-seed-integrity / knowledge-format /
  docs-links / naming-ledger / rust-tests / test-ratchet / table-content-hash /
  table-evidence / state-machine-anchors / asset-truth-manifest / state-baseline 全 ✓）
  ——"gates check 全绿"声称属实。
- fx_item13 测实跑：`1 passed; 0 failed … finished in 3.88s`（声称 4.07s，机器时差非虚报）。
- 【注记】test-ratchet total=931(基线929)：本批 +1 测（fx_item13）未 bump
  rust-test-counts.json（gate 判据只禁回退——不违规）；基线滞后 +2 中 +1 系前批累计，
  建议随下批 `ratchet-update` 归账（对账惯例，不阻本批）。

## 6. 差异分类登记（diff-case，verifier 模板）

### diff-case 2026-11-03-P1A2-B1-01（红）
- 输入序列: commit e19d08c diff 全文（3 处锚引）vs deobf a.java 实文
- 现象: addItemToItemStack 锚"a.java:7721-7734"错引——实测方法=7727-7740
  （grep -n 'private void addItemToItemStack' = 7727；方法尾 7740）。错引 3 处：
  commit message、fx-item13.txt:7 cover、fixture_frames.rs:601 测注。
- 分类: 投影错误（锚行错引；行为/夹具本体无损）
- 处理: 回本批修复——3 处锚改 7727-7740（未推送，可 amend 或补正 commit）；
  顺带修 diff-case 02/03/05/06 同批锚注记。
- 门禁: sed -n '7727,7740p' reference/src/deobf/a.java → addItemToItemStack 全文

### diff-case 2026-11-03-P1A2-B1-02（黄）
- 输入序列: commit message 真值核括注 vs deobf a.java:9283-9299 实文
- 现象: 括注"旗标读循环 9290-9292+skip 9294-9296"两处各差 1 行
  （实测 9291-9293 / 9295-9297）；代码内锚 9291-9297 正确
- 分类: 投影错误（commit message 注记行号；不影响代码注记）
- 处理: 补正 commit 说明（历史不改写——但本 commit 未推送，amend 即可清）
- 门禁: sed -n '9291,9297p' reference/src/deobf/a.java

### diff-case 2026-11-03-P1A2-B1-03（黄）
- 输入序列: p1a1-sweep-review §2 点名 4 处残留清单 vs e19d08c --stat/现存 grep
- 现象: 点名项 docs/spec/refactor-behavior-matrix.md:59（9285-9291）漏清仍存；
  commit"9285 同族锚 4 处清（复核点名）"实清 3 点名+1 未点名（save.rs:4）
- 分类: 投影错误（声称-内容不一致 + 漏清点名项）
- 处理: 回本批补清 matrix.md:59（→9291-9297），commit 正文措辞对齐实改清单
- 门禁: grep -rn 9285 crates/ docs/ tools/（除 review 史料与哈希伪命中）→ 应零

### diff-case 2026-11-03-P1A2-B1-04（黄）
- 输入序列: save.rs:4"字段序（a.java:9253-9297）"vs 表列 21 行字段实文
- 现象: 行 20（a.java:9300-9308）/行 21（9310-9317）不在区间内——"全序"仍欠
- 分类: 投影错误（锚区间欠 20 行）
- 处理: 9253-9297→9253-9317（或表拆段标注）
- 门禁: sed -n '9300,9317p' reference/src/deobf/a.java

### diff-case 2026-11-03-P1A2-B1-05（黄）
- 输入序列: fixture_frames.rs 映射表 vs preset-floor5.txt cover + preset_cases.rs:931-936
- 现象: "use 13/14（preset-floor5 复用）"——preset-floor5 仅覆盖道具 14 全链，
  use 13 无覆盖；"21（preset-item21/2122 复用）"——preset-item2122=臂 20 拦截路径
- 分类: 规格缺口（映射表复用错标，误导后续 arm_coverage 强制测验收）
- 处理: 表内改标：use 13=净补、preset-item2122 移"20"行
- 门禁: sed -n '1,8p' crates/game-oracle/tests/fixtures/preset-floor5.frames.txt

### diff-case 2026-11-03-P1A2-B1-06（黄）
- 输入序列: commit 声称"arm_coverage 缺臂清单实录"vs 仓库落档面
- 现象: 红清单无落档（arm_coverage.rs 已删未入库、docs 无记录），仅编译产物可复现
- 分类: 证据归宿缺口（事实为真、归宿缺）
- 处理: 补落档（红清单/arm_coverage.rs 入 findings 或 #[ignore] 测），或措辞改口径
- 门禁: 全库 grep 缺臂清单/p1a2_all_21 → 现仅编译产物字符串

### diff-case 2026-11-03-P1A2-B1-07（黄）
- 输入序列: 映射表/计划 P1-a2 行 vs 型 32 递延决定
- 现象: 递延注记仅落 commit message，映射表"32"行与计划行无递延标记
- 分类: 文档归宿缺口（注记在但单点）
- 处理: 表内补"（递延下批·pre-final-audit:122）"标记
- 门禁: sed -n "604,611p" crates/game-oracle/tests/fixture_frames.rs

## 7. 总判

### 红黄绿汇总

| 攻击点 | 判定 | 要点 |
|---|---|---|
| ① 夹具真伪 | 🟢 | 三件套齐；链 sha 350bd8b8…/源 trace a50efb53… 独立重算吻合；A/B/C 三方对拍=真（现场重跑三方 sha 全同）；重生成逐字节等值 |
| ② 臂覆盖 | 🔴 | 语义面绿（pickupItemType 13..=25\|85\|86→addItemToItemStack 逐字吻合；13 型=查看敌人信息道具 uses=-1 双源对拍；探针零残留）；**但 addItemToItemStack 锚 7721-7734 错引（实=7727-7740）×3 处** |
| ③ 机制发现 | 🟢 | 61=60+1 纯追加实证（旧行零动）；"+新增判漂移→重冻"=A2 冻结面设计内（state_baseline.py:120-137 实测）；发现已入账（commit 注记三要素齐） |
| ④ 三残账 | 🟡 | ①改 3 真/漏清 matrix.md:59+全序锚欠+真值核括注错行；②表落位可回指但 use13/preset-item2122 复用错标；③递延注记在但表内无标 |
| ⑤ 红证据 | 🟡/🟢 | RED=15 复现属实（藏三件套实跑恰 15）；"清单实录"无落档=黄；夹带净=绿；金样 61 全同+gates 13/13=绿 |

### 计分

- 🔴 红 **1**：diff-case 01（addItemToItemStack 锚 7721-7734→7727-7740，3 处；
  并致 commit"Evidence: A 级——deobf 锚真值核"声称不实）
- 🟡 黄 **6**：02 真值核括注错行 / 03 漏清点名项 matrix.md:59+"4 处清（复核点名）"口径不符 /
  04 全序锚欠 9300-9317 / 05 映射表复用错标×2 / 06 红清单未落档 / 07 型32 递延表内无标
- 🟢 绿：夹具真伪、臂语义、机制发现、夹带、金样 61、gates 13/13、拾取断言（400 帧全比 1 passed 实跑）

### 总判：**需修复（红 1 黄 6）——本 commit 不推送**

理由（逐提交纪律）：
1. 红项=本批核心语义锚错引且三处扩散（含入库夹具头注），并直接证伪 commit 的
   A 级"锚真值核"声称——属"锚伪"类，plan §0 明定红项回派修复后重新复核；
2. 黄 03 是本批卖点"同族锚清"的自我验收缺口（复核点名 4 处清 3），按
   gotcha_commit-claims-must-match-content 应同批对齐（未推送 amend 即可干净补正）；
3. 其余黄项修复成本均≤10 分钟（锚行号/表标记/落档），无一涉及行为面；
   行为面（夹具/断言/金样/门禁）本复核全部实跑绿——修复仅限注记与锚行，
   修后预计复验即绿可推。

**修复清单（回派）**：①3 处 7721-7734→7727-7740；②补清 matrix.md:59（→9291-9297）
并核对 commit 正文"4 处"口径；③save.rs:4→9253-9317；④commit 真值核括注改
9291-9293/9295-9297；⑤映射表改标（use 13=净补、preset-item2122→"20"行）+型32
补"（递延下批·pre-final-audit:122）"；⑥红清单/arm_coverage.rs 落档（或措辞改口径）。
修后复验判据：`grep -rn 9285`（除史料/哈希）零命中 + sed 三锚实文 + `gates check` 全绿
+ `state_baseline --check` 61 全同。

---
复核员注：本件全部判定基于现场实跑输出（贴于 §1-§5），无转述声称；未修改被审 commit
任何文件（arm_coverage 复现实验已恢复三件套，`git status` fixtures 面净）。
