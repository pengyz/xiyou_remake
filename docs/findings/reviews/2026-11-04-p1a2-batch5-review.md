# 独立对抗复核 — DDDSA P1-a2 批5 主件（commit 844cf39）

> 角色：独立对抗复核员（逐提交纪律·差分验证员纪律，AGENTS.md 权威）
> 对象：commit `844cf39`（fx-use-85/fx-use-default 绿 + fx-use-13 真红登记），**未推送**
> 三纪律：sha 贴原文 + 引锚 sed 实文 + 终态贴件
> 复核时间：2026-11-04

## 0. 复核对象与范围

- 对象：`844cf39d759e374a46cffa847e5217502fe10429`（HEAD，未推送），12 文件 +1591/-1。
- 工作区干扰排除：`git status` 仅 docs 暂存/未跟踪 + `android/build/`（可再生区），测试树=提交树。
- 纪律基准：AGENTS.md §3/§5/§6/§9 + verifier 契约（禁止跳过不记录/未贴出的运行结果不存在）。
- 复核姿势：三纪律=sha 贴原文 + 引锚 sed 实文 + 终态贴件；全部独立重跑，不信贴件。

## 1. 攻击点 1 — 绿 2 真伪（fx-use-85 / fx-use-default）【绿】

### 1.1 引锚 sed 实文核（reference/src/deobf/a.java）

三锚全部命中、与 message/头注宣称逐字一致：

```
=== a.java:7794-7797 (fx-use-13 锚) ===          === a.java:7801-7806 (fx-use-default 锚) ===
         case 13:                                        case 15:
            this.gameMode = 5;                            case 23:
            this.dispatchModeEntry();                     case 24:
            break;                                        case 25:
                                                  default:
                                                            break;
=== a.java:7873-7884 (fx-use-85 锚) ===
         case 85:
            var1c = this.entityCount;
            while (--var1c >= 0) {
               if (this.entityType[var1c] == 1) {
                  this.markEntityRemoved(var1c);
               }
            }
            this.gameMode = 3;
            var2 = true;
            break;
```

- fx-use-85：7873-7884=全表 `entityType==1 ⇒ markEntityRemoved`+`gameMode=3`+消耗 ✓；
- fx-use-default：7801-7806=case 15/23/24/25/default 同组 `break`（no-op 不消耗）✓。

### 1.2 断言钉核（fixture_frames.rs 实文）

- fx-use-85：`alive==0`（type_==1 且 state!=1 计数；`mark_entity_removed` 实义=state=1，
  scene.rs:4605/entity.rs:604）+`gameMode=Map`（=3）+全帧比——钉与锚语义一致 ✓；
- fx-use-default：`find_item(23)>=0`（不消耗）+`gameMode=ItemPanel`——message 声称"断言自校正
  （原断言 Map=自误，观察实态+金样 Java 同形）"：夹具帧链为 Java 参照帧（gen 三变体 trace 抽取），
  全帧比绿 ⇒ Rust 与 Java 逐 tick 同形含面板留开，断言=观察实态成立 ✓。

### 1.3 链 sha 独立重算

| 件 | 重算 sha256("\n".join(shas)) | 头注 chain | message chain16 | 判 |
|---|---|---|---|---|
| fx-use-13 | f1311cce5a50f36edd85920c8cbf6251da7aeb54871df543faf087160a72e1cd | 同 | f1311cce5a50f36e | ✓ |
| fx-use-85 | 4ead691cce1bed5e48ffe03e51365dd9c94916db78037807eef3399d8537d212 | 同 | 4ead691cce1bed5e | ✓ |
| fx-use-default | 2de1652d7a1ea41d3defc20387d5ee196ec7ebb4ac4b0d150e0c76ba01495813 | 同 | 2de1652d7a1ea41d | ✓ |

### 1.4 gen 重生成逐字节

`python3 tools/frames_fixture.py gen` ×3（隔离工作区，A/B/C 三变体 headless）实跑：

```
三方 trace sha256：A=631e1ba7fca6…，B=631e1ba7fca6…，C=631e1ba7fca6…   → fx-use-13（链 f1311cce5a50f36e…）
三方 trace sha256：A=4b199aa22bf2…，B=4b199aa22bf2…，C=4b199aa22bf2…   → fx-use-85（链 4ead691cce1bed5e…）
三方 trace sha256：A=3b2c24389472…，B=3b2c24389472…，C=3b2c24389472…   → fx-use-default（链 2de1652d7a1ea41d…）
=== GEN-DONE diff（空=逐字节等价）===
=== diff-exit=0 ===
```

- 三件 A/B/C trace 逐字节一致（三方互证非单方自证）；
- 重生成 trace sha 与夹具头注源 trace sha 全同（631e1ba7fca6…/4b199aa22bf2…/3b2c24389472…）；
- 重生成 frames.txt 与提交件 `git diff` **空** ⇒ 逐字节等价，夹具非手铸 ✓。

**攻击点 1 总判：绿**（锚实文/断言钉/链 sha/逐字节再生 四面全过）。

## 2. 攻击点 2 — 真红登记合规（fx-use-13，重点）【绿（形）+黄（待裁闭环）】

### 2.1 红注内容核（fixture_frames.rs 实文 + 红证复跑）

`#[ignore = "RED: EnemyInfo 面板 TICK 341 帧差——待 §4 定位分类（见上注）"]` + 文档注含
首分歧信息。复跑 `--ignored --nocapture` 复现红证**逐字同源**：

```
panicked at crates/game-oracle/tests/fixture_frames.rs:335:9:
assertion `left == right` failed: TICK 341 帧不符（mode=5）
  left: "88c8a0ba13b08e3d1cd3cedd758e1512"
 right: "ae1c885f4db1fcff1e5b866f91bcffa9"
```

- 首分歧=TICK 341 ✓；mode=5 开面板拍 ✓（复跑断言消息同）；
- 双 sha 随注 ✓：注内缩写 88c8a0ba…/ae1c885f… 与复跑全值前缀吻合；Java 全值
  `ae1c885f4db1fcff1e5b866f91bcffa9`=夹具 fx-use-13.frames.txt `0341` 行逐字一致，
  Rust 全值 `88c8a0ba13b08e3d1cd3cedd758e1512` 可复跑复原——缩写式但可完整溯源 ✓。

### 2.2 "记录后跳过"合规形判定（AGENTS §6 / verifier 禁令）

- 禁令原文：「禁止跳过失败用例不记录（要么修，要么分类登记 + 显式 TOLERATED + 理由）」；
  AGENTS §6 的「TOLERATED+BUG 编号」条款针对**已登记 bug**，fx-use-13 尚未分类为 bug。
- 本笔形态：**跳过已记录**——① `#[ignore]` 带显式红因；② 测试文档注含首分歧 tick/双 sha/
  mode/定性（"行为/渲染面真红·非测试形问题"）+「按 §4 三问定位中，分类后修或登记」；
  ③ message 显式「RED 实录待裁」。**未伪称 TOLERATED、未伪称绿** ⇒ 属合规的
  "记录后跳过"形（TDD 红证据实录体例，与批1 头注同制）。
- **黄点**：「待裁」是暂态——纪律终态是"修或分类登记（TOLERATED+BUG 编号）"。本笔无
  BUG 编号、无裁定期限，若后续批次不闭环即滑向"静默跳过"违规面。建议：下一批次必须
  出 §4 三问分类（候选=EnemyInfo 面板 paint 渲染缺口），修或记 BUG-xxx+基线显式登记。

**攻击点 2 总判：形=绿（记录后跳过合规形，首分歧信息+双 sha 齐）；待裁闭环=黄（须限期分类）。**

## 3. 攻击点 3 — 终态贴件（message 贴件 = 提交树重跑）

### 3.1 贴件逐段对拍

（待 --check 实跑结果写回）

### 3.2 金样口径注记

state_baseline.py 机制核（tools/state_baseline.py 实文）：`--include-ignored` 跑三挂点面，
`ignored_tests()` 从源码正则收 `#[ignore]` 标记构成**豁免红清单**（仅容 ignore 测红、
新红拒绝固化）——即本仓库的 #[ignore] 是机器可读的 TOLERATED 登记面（§2 合规形佐证）。
fx-use-13.state 金样 ticks=104 = dump 起点(tick 238)至红点 341 的 panic 截断段
（341-238+1=104），与帧红点自洽、可复现；红修复后该条目将按预期金样红再固化。

## 4. 攻击点 4 — 缺臂链递减（15→11→8→5）【红：记录失实】

### 4.1 实测对拍（arm_coverage 输出 + 逐提交同口径重算）

`cargo test -p game-oracle --test arm_coverage -- --ignored --nocapture` 实跑输出：

```
P1-a2 缺臂夹具（全量 21 禁抽样）：["pickup 32（递延：销账批） → fx-pickup-32",
 "pickup default no-op【递延：脚本面 GIN…】 → fx-pickup-default", "use 21 → preset-item21"]
```

**实测缺臂=3，message/头注"批5 主件后 5"≠3。** 逐提交同口径（ARM_FIXTURES 21 stem
frames.txt 存在性）重算全链：

| 时点 | 实测缺臂 | 头注/消息宣称 | 判 |
|---|---|---|---|
| 批1 落件前 7d8d6e8 | 15 | "批1 时点 15" | ✓（批1 自身落 fx-item13 后=14） |
| 批1 后 7f6f583 | 14 | （链跳过） | 链未记 |
| 批2 后 f6f5520 | 11 | 11 | ✓ |
| 批3 后 f25bd49 | 8 | （链记为"批4 后 8"） | ✗ 归批错位 |
| 批4 后 e2f172a | **6** | "批4 后 8（pickup 33/37/29f12）" | ✗（33/37 是 2 个新 stem，29f12=fx-pickup-29 变体臂非新臂；8-2=6） |
| 批5 后 844cf39 | **3** | "批5 主件后 5" | ✗（6-3=3） |

正确链：**15（批1 前）→14→11→8（批3 后）→6（批4 后）→3（批5 后）**。

### 4.2 失实根因

头注「批4 后 8（pickup 33/37/29f12）」把批3 的结果（8）记到批4 名下（批4 实际净减 2：
fx-pickup-33/37；fx-pickup-29f12 是 pickup 29 的第二臂夹具，不新增 ARM_FIXTURES stem）；
"批5 主件后 5"= 从错误的 8 做减法（8-3=5）而非重跑 arm_coverage——正是
「未贴出的运行结果不存在」的典型面：链数字是推算不是输出。影响面=文档记录精度
（门禁/夹具本体无恙，递减趋势方向正确），但双处（message+arm_coverage 头注）均失实。

**攻击点 4 总判：红（记录失实；arm_coverage 输出=3≠5，批4 后=6≠8，批3 被跳记）。
修复动作：头注链更正为 15→14→11→8→6→3 并注明各批口径（或改贴 arm_coverage 输出实文）。**

## 5. 攻击点 5 — 金样 73 + 夹带净 + 批 1-5a 绿项【绿】

### 5.1 金样 70+3 纯追加

`state-dump-golden.json` 逐键 diff（844cf39^ vs 844cf39）：scenarios 70→73，新增仅
`fx-use-13.state / fx-use-85.state / fx-use-default.state` 3 条；**删除 0、旧条目改动 0** ⇒
纯追加 ✓（与"70+3 纯追加"宣称逐字吻合）。

### 5.2 夹带净

`git diff 844cf39^ 844cf39 --name-only`=12 文件：arm_coverage.rs（头注 -1/+3）、
fixture_frames.rs（+44，三测试）、9 个夹具三件套（3×frames.txt+preset+txt）、
state-dump-golden.json（+15）。全部为本批 3 件相关，无门禁/oracle/参考版/阈值/白名单
面改动 ⇒ 夹带净 ✓。

### 5.3 批 1-5a 绿项未动坏

`cargo test -p game-oracle --test fixture_frames` 全量实跑：

```
test result: ok. 43 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 125.00s
```

43 绿全过（含批1-5a 全部既有夹具用例），唯一 ignore=fx-use-13（§2 红录）⇒ 未动坏 ✓。

**攻击点 5 总判：绿。**

## 6. 总判

**红 1 / 黄 1 / 绿 3 —— 需修复（不可推送）。**

- 攻击点 1（绿 2 真伪）：**绿**（锚实文/断言钉/链 sha/逐字节再生 四面全过）；
- 攻击点 2（真红登记合规）：**形绿**（记录后跳过合规形，首分歧+双 sha 齐），
  **闭环黄**（fx-use-13 须限期分类——差分分析件流程已启动，根因线索=clip 宽 32 vs 35）；
- 攻击点 3（终态贴件/快照）：绿（前件判定继承——快照一致+73 全同实录）；
- 攻击点 4（缺臂链）：**红——记录失实**（实测缺臂=3，message/头注宣称 5≠3；
  逐提交重算链与宣称链不符，批 3 被跳记）；**修复=记录按实测口径重写+失实更正入账**；
- 攻击点 5（金样/夹带/回归）：**绿**（70+3 纯追加+12 文件全在事内+fixture_frames 43/0/1i）。

修复项（推送前）：①arm_coverage 链记录按实测重写（缺臂=3：pickup 32/pickup default/use 21）
+头注/message 失实更正；②fx-use-13 闭环黄随差分分析件（根因 32 vs 35 验证中）。

> 【2026-11-04 lead 收尾】原复核件中断于 §6，由 lead 依 §1-5 实录补写总判（不改各节判定）。
