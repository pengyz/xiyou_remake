# P1-a2 批5-a 复核（逐提交纪律·差分验证员纪律）

> 复核对象：commit `cdbc7c1`（DDDSA P1-a2 批5-a：五行动项清账）——**未推送**。
> 复核员：独立对抗复核员（差分验证员纪律，AGENTS.md 权威）。
> 日期：2026-11-04。三纪律=sha 贴原文 + 引锚 sed 实文 + 实证段禁手粘。
> sha 贴原文：`cdbc7c156a25d6184100de2d08dbe6c0c3a2e5d6`（`git log -1 --format=%H` 实测，与 `git show --name-status` 同源）。

## 0. 结论

- **总判：需修复（不可推送）**——① 红 1（method_reach --check 终态 exit=1，与贴件"快照一致"直接矛盾，证据陈旧非终态实证）；黄 3（再生锚改坏、双家归属注记缺、grep 修形命令未入档）；②③⑤ 绿（各带附注）。
- 攻击点红黄绿：① 🔴 / ② 🟢（附注）/ ③ 🟢 / ④ 🟡 / ⑤ 🟢（金样实跑见 §5）。

| # | 攻击点 | 判定 | 一句话 |
|---|---|---|---|
| 1 | stem 冲突清 | 🔴 | 改名本体干净，但 `--check` 终态 exit=1（fx-coin25a 快照缺失），贴件"快照一致"不成立于提交树 |
| 2 | method_reach 全量重测 | 🟢 | A-trace=equal 85/85 实证；批3 建议①②兑现；diff 非"纯触达数"（帧链全量重哈希，根因=19d6177 行尾钉死传播） |
| 3 | 矩阵行自转与总账 | 🟢 | fx-pickup 行转形真、总账 147/2/6 与前 141/8/12 对拍实、矩阵字节级可复现 |
| 4 | 禁手粘 + grep -F 边界 | 🟡 | 贴件 7 行逐字全同（无手编辑）；"修形"命令未入档——边界可裁但口子敞着 |
| 5 | 金样/夹具净/批 1-4 | 🟢 | 金样 70 条目未触碰 + `state_baseline --check` 实跑"[GREEN] 70 场景 sha 全同"；夹带净；批 1-4 测试 46/0 全绿 |

## 1. 攻击点① stem 冲突清 🔴

### 1.1 改名本体（引锚 sed 实文）

- `git show cdbc7c1 --name-status`：`R100 crates/game-core/tests/fixtures/fx-item25a.{preset,txt} → fx-coin25a.*`、`R099 fx-item25a.frames.txt → fx-coin25a.frames.txt`（相似度 99%）。scenarios 侧不动——`ls reference/oracle/scenarios/ | grep 25a` ⇒ `fx-item25a.txt` `preset-item25a.preset` 原样留存 ✓。
- mov_route_scene.rs：doc 头 + `fn fx_coin25a_lucky_coin_doubles_gold_frames_match` + `include_str!`×3（preset/script/frames）随更；旧 stem 在 crates/ 零残留（`grep -rn "fx-item25a\|fx_item25a" --include="*.rs" crates/` = 空）。改名干净（文件/fn/引用面）✓。
- 计数注记：claim "mov_route_scene.rs×4+fixtures 头×4"——mov 实改 5 行（fn+3 include+doc，"×4"按标识符位计勉强）；fixtures 头实改 2 行（frames.txt L1 名 + L4 再生命令 3 处替换）——"×4"按替换位计勉强；非实质问题。

### 1.2 method_reach --check 实跑核（🔴 核心）

```
$ python3 tools/method_reach.py --check
[method_reach] STALE fx-coin25a: 快照缺失
[method_reach] 快照陈旧 1 项
[exit code: 1]
```

两次实跑（干净树，HEAD=cdbc7c1）同果。与 commit Evidence「=== method_reach --check ===
[method_reach] 快照一致」**直接矛盾**。

机理（tools/method_reach.py:315-335 实文）：`--check` 校验集 = `SCEN.glob("*.txt")` stem ∪
`fixture_scenarios(tracked_only=True)` stem。提交终态 fx-coin25a.txt 已入库 ⇒ 进校验集 ⇒
快照无 fx-coin25a 条目 ⇒ STALE exit=1。贴件"快照一致"唯一成立的时点=改名**未 `git add`** 的
过渡态（docstring「在制夹具不进全量/校验」过滤未入库夹具）——即 Evidence 采集于提交前瞬态、
不描述提交树。逐提交纪律下：**实证段必须对应提交终态** ⇒ 证据陈旧 = 红。
（30ff91c 预存根因 exit=1 属实：旧态 fixture stem=fx-item25a 亦无快照条目 ⇒ 同样 STALE；
本批把失败项从 fx-item25a 换名成 fx-coin25a，exit=1 未清。）

修复：`python3 tools/method_reach.py --only fx-coin25a`（工具 docstring 明示的在制夹具补测径）
补入快照 → `--check` 复跑 exit=0 → 更新 Evidence。

### 1.3 双家归属注记（🟡）

- `grep -rn "双家\|coin25a\|fx-coin" docs/ data/`（排除本复核件）= **空**。双家归属仅存于
  cdbc7c1 commit message，repo 内无注记。fx-coin25a.txt 头留 `# preset: preset-item25a.preset`
  （provision 侧指针）+ `FX-ITEM-25A` 特性号，属弱归属线索，不足以当"注记"。
- **再生锚被改坏**（🟡）：fx-coin25a.frames.txt L4「再生：…extract reference/oracle/_out/A-fx-coin25a/trace.txt
  --stem fx-coin25a …」——`ls reference/oracle/_out/` 实存 `A-fx-item25a`、无 `A-fx-coin25a`
  ⇒ 复现命令指向不存在路径（改名随改锚文但 scenarios/_out 侧不动，锚与实文脱节）。A 级证据的
  字节级复现命令失效（AGENTS.md §3.3）。L2「源 trace：…/A1/trace.txt」为预存旧文（e2f172a 同），不计本笔。
- 账面联动缺口（观察，预存非本笔引入）：fx_coin25a 用例在矩阵零入账——`let spec = Spec{…}`
  不被 cover_matrix.py spec_consts（const/static）识别 ⇒ kind=frame-unbounded（矩阵
  「mov_route_scene.rs：3 个（fixture 2 / frame-unbounded 1）」实文）；且夹具帧链
  `d040316c…`（1600 帧，源自 30ff91c A/B/C 三方 1600-tick 跑）≠ 快照 fx-item25a 帧链
  `45e1aa0f…`（900 帧，脚本头 `# ticks: 900`）——即便入账，"FRAME 一致（夹具帧链 = 实测）"
  亦失配。claim"矩阵行随更"空转：旧矩阵 `grep -c "item25\|coin25"`=0，无行可随更。

## 2. 攻击点② method_reach 全量重测 🟢（附注）

- **A-trace=equal 声明实证 ✓**：`json` 机械核——新快照 85 场景（66 旧+19 新）`a_trace.match`
  分布 = `{'equal': 85}`，逐场景零例外。
- **批 3 建议①兑现 ✓**：批3 复核（2026-11-04-p1a2-batch3-review.md:143）「建议下笔随附
  ①重跑 tools/method_reach.py ②重生成矩阵」——本批快照全量刷（66→85）+ 矩阵重生成均落地。
- **diff ≠ "纯触达数更新"（假设驳回，附注）**：66 公共场景结构 diff = 全部仅 `a_trace`+`frame_chain`
  变（`phases` 触达数据 **0 变**）——变的是帧链全量重哈希（66/66，如 boot-menu
  `db3d9256…→d12a7a97…`），另 19 新场景入账。重哈希根因 = `19d6177`「oracle 差分链 Windows 适配
  ——JVM 编码/行尾钉死」reference/ 口径传播后未重测，本批全量刷顺带吸收；JDI 运行链与
  A-trace 链同值同步（match=equal 维持）⇒ 无行为差异、无回退。commit message 措辞
  「全量刷（A-trace=equal 逐场景）」本身准确，不构成失实；diff 组成须注记（本复核补此注）。
- 缺口归 ①：全量刷漏测 fx-coin25a（过渡态未入库被过滤）⇒ --check 红（§1.2）。

## 3. 攻击点③ 矩阵行自转与总账对拍 🟢

- **fx-pickup 行转形 ✓**：e2f172a 矩阵 fx-pickup-26/33 行=`无触达快照 | 0`；本批=`FRAME 一致
  （夹具帧链 = 实测） | 78/83`（矩阵 L64/L70 实文）——转形机制真实：快照补入 fx-pickup-* 场景后
  `verified_hits` 取到触达，且 `--check`"夹具帧已变"项对全部已入库夹具零报（帧链=实测链过核）。
- **总账对拍 ✓**：新矩阵头「绿 147 / 黄 2 / 白 14」+「判据①赤字 = 6」；前账 e2f172a 矩阵头
  「绿 141 / 黄 8 / 白 14」+「赤字 = 12」——commit "绿 147/黄 2/赤字 6（前 141/8/12）" 逐数对上。
  "重测清欠"机制核：旧黄 8 中 6 = "被无对拍用例的场景触达 6" → 新黄 2 该类归零，欠账清偿路径真实
  （新场景触达入账转绿），非改判口径。
- **重生成字节级对拍 ✓**：`python3 tools/cover_matrix.py` 重跑 ⇒ stdout 贴件行与 commit Evidence
  「覆盖矩阵 → docs/coverage-matrix.md（绿 147 / 黄 2 / …；⚠ 15 条校验告警）」**逐字全同**；
  `git diff docs/coverage-matrix.md` = 空（矩阵可字节复现，"请勿手改"纪律未破）。⚠ 15 条 =
  矩阵 ⚠ 行实数（锚失效类预存债，e2f172a 同列，非本批引入）。

## 4. 攻击点④ 禁手粘贴件逐字 + "grep -F 修形"边界 🟡

### 4.1 重跑提取 diff

- 贴件段 1「=== method_reach --check ===」：**diff 非空**——实跑输出与贴件相反（§1.2），红计①。
- 贴件段 2「=== 矩阵重生成 ===」：重跑 stdout 逐字全同 ✓（§3）。
- 贴件段 3「=== fx-pickup 行 ===」7 行：逐行 `in matrix` 机械核 **7/7 逐字全同**，无手编辑字符 ✓。
  复原形 = 两段 grep 拼接（fx-pickup-26 组 5 行 + fx-pickup-33 组 2 行），非单命令文件序；
  省略场景触达表 2 行（| fx-pickup-26 |…| 70 行系）属范围选择，无隐匿不利输出（被略行内容一致）。

### 4.2 边界判定（grep -F 修形 = 口子吗？）

**裁定：命令侧加工 ≠ 贴件编辑，界限可划清，但须三条件闭合：**

1. **合规加工**（允许）：grep -F/sed -n 等**整行选择、排序、拼接**，且 (a) 确切命令入档、
   (b) 只选行不改行内任何字节、(c) 不得选择性隐匿不利输出（FAIL/STALE/告警须随贴或显式声明范围）。
2. **违规**（禁止）：对贴件内容做任何人工字节改动（含"顺手修数字/空格/补省略号"）；
   sed 替换类**改写行内容**视同手编辑（此类产物已非"实文"，应贴原始输出另附说明）。
3. **口子状态**：本笔"grep -F 修形一次"未记确切命令——复原靠猜（我方反推=两次 grep 拼接才复现
   顺序）。内容侧 7/7 逐字清白，但可复现性要件 (a) 不满足 ⇒ **口子敞着，判黄**；
   补记确切命令即闭合。另：本笔 3 贴件段中 1 段陈旧（§1.2）说明"禁手粘"必要不充分——
   纪律须同时锁"终态性"（证据采集于 commit 后的提交树）。

## 5. 攻击点⑤ 金样 70 全同 + 夹具净 + 批 1-4 未动坏 🟢

- **金样 70**：`data/status/baselines/state-dump-golden.json` 条目机械数 = **70** ✓；
  `git show cdbc7c1 --stat | grep golden` = 0（本批零触碰）✓。实跑 `python3 tools/state_baseline.py --check`
  ⇒ （见下"实跑回填"）。
- **夹具净**：`git ls-files --others crates/game-core/tests/fixtures/ reference/oracle/scenarios/`
  = 空（无未追踪夹带）；本批 7 文件全在事内（3 rename + mov_route_scene.rs + arm_coverage.rs +
  coverage-matrix.md + snapshot.json），零夹带 ✓。
- **批 1-4 绿项未动坏**（实跑）：
  - `cargo test -p game-oracle --test fixture_frames` ⇒ **41 passed / 0 failed**（批1 fx-item13、
    批2 pickup 26/27/28、批3 29/30/31、批4 33/37/29f12 全绿）✓；
  - `cargo test -p game-core --test mov_route_scene` ⇒ **5 passed / 0 failed**（含
    `fx_coin25a_lucky_coin_doubles_gold_frames_match` ✓——改名后测试本体真绿）✓；
  - `cargo test -p game-oracle --test arm_coverage` ⇒ 编译过 + `p1a2_all_21_arm_groups_have_fixture_triplets`
    保持 ignored（P1-a2 进行中注记），ARM_FIXTURES default 行内联【递延：脚本面 GIN——图拾不可达
    （cat 2|4 型域全覆盖）】标记与 claim ④ 一致 ✓。

### 实跑回填（state_baseline --check）

```
$ python3 tools/state_baseline.py --check
[GREEN] 金样逐字节等价：70 场景 sha 全同
[exit code: 0]
```

实跑通过（release 全跑 fixture_frames/gameplay_floor1/preset_cases 后 70 场景 sha 全同），
与批4 复核基线一致 ✓。攻击点⑤ 全项 🟢。

## 6. 三纪律自查

- sha 贴原文 ✓（§0）。引锚 sed 实文 ✓（§1.2 tools/method_reach.py:315-335、矩阵 L8/L13/L64/L70、
  fixtures 头 L1-L4、arm_coverage L20 均 sed/cat -A 实读）。实证段禁手粘 ✓（本复核全部证据
  命令重跑产出，无手粘）。
