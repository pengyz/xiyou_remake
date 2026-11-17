# P1-6① 独立对抗复核（1e00e14）——EnemyDef 聚合数据面

> 复核员：独立对抗复核员（逐提交纪律·差分验证员纪律，AGENTS.md 权威）
> 对象：`1e00e14` feat(data): DDDSA P1-6① EnemyDef 聚合数据面——gen_enemies.py+enemies.json 35 行+defs.rs EnemyDef+parse_enemies（**未推送**，绿=lead 推送）
> 范围：单 commit `git show 1e00e14`（4 文件 +835/-0）
> 日期：2026-11-06
> 状态：**已完成**（骨架→逐节写回；全部判据=独立复跑/读码/源对拍/4 发变异实跑，非采信贴件）
> 总判：**红 0 / 黄 2 / 绿 4 —— 可推送**（两黄=注释/文档级，建议 amend 后推）

## 0. 结论速览（红/黄/绿 + 总判）

| # | 攻击点 | 判 | 一句话 |
|---|---|---|---|
| ① | 逐值迁真伪（五源对拍+确定性） | **绿**（附注） | **35 行×4 字段=140 值全量对拍 0 差异**（强于抽 5 行）+ mult 5 项 + trait 35 项全同；确定性重跑同 sha `2c6260e0a02eea9d`、`git diff` 空；附注：difficulty 锚申报 2236 实测 **2234**（偏 2 行，沿袭 combat.rs:13/19 既有同偏，本件扩散到 commit/json/test 三处） |
| ② | 行数 35 对账（成员纠偏） | **绿** | 独立 sed/grep 计数四表各 **35**；89=TYPE 域宽 0..=88（defs.rs `TYPE_ID_MAX=88` + sprite_defs 实测 89 行）⇒ **纠偏属实**；id=41..75 ↔ t-41 横索引（a.java:5484/2579/7951… + scene.rs:2311/3642）对拍成立 |
| ③ | 五类拒载+behavior 白名单 | **黄** | 3 负例**真判别**（4 发变异：删白名单/关域检/关范围/改 json trait → 各自红，跑完还原树净）；trait 源锚 `a.java:6266-6267` 逐行精确、三方一致；**黄因=声明>断言**：`enemies_defs.rs:31` doc 列"非整数"负例但**不存在**（实 3 负例） |
| ④ | 红线（cellSlot/sprite·drop/消费面） | **黄** | 三条实质全过：EnemyDef 恰 6 字段无 cell/slot；drop/loot 在 Rust **0 命中**、sprite 走独立表；`parse_enemies` 仅测试引用、`combat.rs:14/60` SoA 原封（②未切=预期态）；**黄因=`defs.rs:387` 引据反了**：称"族表按 spec v3 §0 裁定不并入"，而 §0 原文恰是"per-type 聚合、89 型怪族表→per-type 记录"，且"族表"=enemyBase 四表（本件并入对象）——注释与同 hunk 代码自相矛盾 |
| ⑤ | 验证（6 测+金样+夹带净） | **绿**（附注） | 6/7/**337（lib）**/74 全部独立复跑同值；金样 `[GREEN] 74 场景 sha 全同` EXIT=0；夹带恰 4 文件 +835，reference/original/gates/naming **零触及**；附注：core"全量"贴件=lib 首条，真全量 489/0 全绿 |
| ⑥ | 三纪律 | **绿** | 贴件六项全部从提交树复现同值（工作树三件 sha 与 `git show` 逐一相同）；申报 35/89/5/69=2 均 grep 实测复现；Evidence+Confidence:93 在 |

**红 0 / 黄 2（③doc 超宣负例、④引据反 §0）/ 绿 4（①②⑤⑥，①⑤附注）—— 总判：可推送**
（两黄均为**注释/文档级**，不涉数值、行为、回退；commit 未推送，建议 lead 推送前顺手 amend：
① 删 `enemies_defs.rs:31` 的"非整数"或补一个 `hp:"x"` 负例；
② `defs.rs:387` 去掉"族表/spec §0"改引 `refactor-plan.md:41` 空间索引裁。
跟进项（不阻本件）：`enemies.json` 按 P1-1 先例单独 commit 进 FIDELITY_TABLES/哈希基线+逐值钉。）

## 1. 攻击点①：逐值迁真伪（五源抽 5 行对拍 + 确定性复跑）——**绿**（附注：difficulty 锚偏 2 行）

### 1.1 五源实文（sed 抽取，非采信贴件）

| 源 | 实锚（我方 grep/sed 实测） | 实文要点 |
|---|---|---|
| enemyBaseHp | `a.java:1044`（4 表区 1044-1122） | `this.enemyBaseHp = new int[]{35,45,35,50,60,…,120,444,100}` |
| enemyBaseAtk | `a.java:1081` | `…{18,20,38,42,32,…,150,199,65}` |
| enemyBaseDef | `a.java:1118` | `…{1,2,3,6,8,12,110,…,50,66,15}`（单行内联） |
| enemyBaseGold | `a.java:1121` | `…{1,2,3,6,5,8,100,…,100,144,30}`（单行内联） |
| difficultyMultipliers | **实测 `a.java:2234`**（申报 2236，见 1.4） | `this.difficultyMultipliers = new int[]{1, 30, 60, 80, 100};` |
| traitFlagsByType | `a.java:6266-6267` **逐行精确** | `[49]=[53]=[74]=1;` / `[69]=2;` |

### 1.2 对拍：不止抽 5 行——**35 行 × 4 字段全量对拍 0 差异**

独立脚本（我方 python，非生成器代码）解析 a.java 四表 + difficulty + trait，
与 `data/tables/enemies.json` 逐行逐字段比对：

```
counts: 35 35 35 35  mult: 5 [1, 30, 60, 80, 100]
json rows: 35 ids: 41 .. 75
full mismatch ids: []        ← 35×4=140 值全同
trait mismatch: []           ← 35 项 trait 全同
```

任务指定的 5 行抽样（json 实文 ↔ 源实文）：

| id | json（hp/atk/def/gold/trait） | 源（enemyBaseHp/Atk/Def/Gold[i] + 6266-6267） | 判 |
|---|---|---|---|
| 41 | 35/18/1/1/0 | hp[0]=35 atk[0]=18 def[0]=1 gold[0]=1；无 trait | ✅ |
| 49 | 260/85/5/18/**1** | hp[8]=260 atk[8]=85 def[8]=5 gold[8]=18；`[49]=1` | ✅ |
| 53 | 320/120/15/30/**1** | hp[12]=320 atk[12]=120 def[12]=15 gold[12]=30；`[53]=1` | ✅ |
| 69 | 1500/600/250/800/**2** | hp[28]=1500 atk[28]=600 def[28]=250 gold[28]=800；`[69]=2` | ✅ |
| 75 | 100/65/15/30/0 | hp[34]=100 atk[34]=65 def[34]=15 gold[34]=30；无 trait | ✅ |

### 1.3 确定性复跑（同 sha + git diff 空）

```
$ python3 tools/enemy-defs/gen_enemies.py
enemies.json → …（35 行；difficulty 5 项）
$ sha256sum data/tables/enemies.json | cut -c1-16
2c6260e0a02eea9d          ← 与 commit 贴件同
$ git diff --stat data/tables/enemies.json   # 空
$ git status --porcelain data/tables/enemies.json   # 空
```

工作树三件（gen_enemies.py / enemies.json）sha 与 `git show 1e00e14:` 版本**逐一相同**
（`11250682a2b73cc1` / `2c6260e0a02eea9d`）——贴件=提交树终态成立。

### 1.4 附注（黄级证据指针失准，不改值）

`difficultyMultipliers` 锚申报 **a.java:2236**，我方实测 **a.java:2234**
（`sed -n '2232,2237p'`：2233=difficultyIndex、**2234=difficultyMultipliers**、2236=sparkFrames）——
偏 2 行，读者按锚跳转落到 sparkFrames。**同偏系沿袭既有**：`crates/game-core/src/combat.rs:13/19`
早已写 `a.java:…/2236`（本件复制既有注释，非新引入）；但本件把它同时写进
**commit message + enemies.json note + 测试断言消息**三处，扩散了失准锚。
值/语义（全局表非 per-type）正确：`a.java:7224/9319 = scaleEnemyStats(difficultyMultipliers[difficultyIndex])`
= 按难度下标取全局倍率，非按型索引 ✅。
另：`evidence.hp` 区间 `a.java:1044-1130` 实际四表止于 1122（1123 起是 levelScriptLines）——区间偏宽不漏表，附注。

## 2. 攻击点②：行数 35 对账（成员纠偏申报）——**绿**

### 2.1 独立实测源表行数（grep/sed 计数，非采信生成器）

```
$ sed -n '/this.enemyBaseHp = new int\[\]/,/^      };/p' reference/src/deobf/a.java | grep -cE '^\s*[0-9]+,?\s*$'
35
$ … enemyBaseAtk … → 35
$ … enemyBaseDef … | tr ',' '\n' | grep -cE '[0-9]+' → 35
$ … enemyBaseGold … → 35
```

四表各 **35** ⇒ `enemies.json` 35 行 = 源表长，**申报正确**；lead 派工的 89 系
**TYPE 域宽**而非敌表长——独立佐证：

- `crates/game-data/src/defs.rs:46-48`：`TYPE_ID_MAX: i64 = 88` ⇒ `0..=88` = **89**；
- `resource_cfg.rs:307/492/778` panic 文案字面 `型号域 0..=88（sprite_defs 89 项）`；
- `data/tables/sprite_defs.json` 实测 **89 行**，type_id min/max = 0/88；
- 敌方四表（enemyBase*）**无一为 89 行**。

⇒ 成员纠偏（35 ≠ 89）**属实**，commit message "申报=grep 计数实测"与我方实测同值。

### 2.2 id=41..75 域 ↔ SoA 横索引 t-41 对拍

| 侧 | 实测 |
|---|---|
| Java | `a.java:5484` `battleEnemyHp = enemyBaseHp[entityType[e] - 41]`；`a.java:2579/7951/7953/10054/10057` `enemyBaseGold[t - 41]`；`a.java:2561/2567/2573` `enemyAtk/Def/HpScaled[var144 - 41]`；`a.java:7984-7986/8775/8783/10039/10044` 同形 |
| Rust | `scene.rs:2311/2313` `enemy_base_gold[(t - 41)]`、`scene.rs:3642` `enemy_base_hp[(t - 41)]`（**消费面仍在**，见 §4） |
| json | `rows[i].id == 41 + i`（测试 `enemies_row_count_matches_source` 断言 + 我方全量核 `ids: 41 .. 75`） |

四表同长 35 + 同一 t-41 横索引 ⇒ id 域 41..75 与 SoA 横索引**同构成立**。

### 2.3 附注（既有面，非本件）

`objectTypeNames` 实测 **88 项**（`a.java:955-1042`，数组 954 开 1043 闭），
与 TYPE 域 0..=88（89）差 1——即 type 88 有名无名之别属**既有事实**（spec v3 §0
"89 型"按域宽口径仍成立），本 commit 未触及，留档供 lead 留意。

## 3. 攻击点③：五类拒载 + behavior 白名单（负例真伪 + trait 源锚）——**黄**（声明>断言：负例实 3 类，doc 却列"非整数"）

### 3.1 读测（`crates/game-data/tests/enemies_defs.rs` 全 58 行已读）

6 测试 = 行数对账 / trait 抽查 / **3 负例**（unknown_field、domain_overflow、param_overflow）/ C1 演练。
行为码序（`defs.rs parse_enemy`）：**行键白名单 → id+域检 → behavior 键白名单 → need_int → trait 范围**——
即 commit 修复实录所称"白名单前置"（防 need_int 先于键检返回 Schema 而非 UnknownField）**属实**。

### 3.2 跑 + 判别力变异（4 发，发发见红，跑完即还原、树净）

```
$ cargo test -p game-data --test enemies_defs   → 6 passed; 0 failed
变异A 删 behavior 键白名单  → enemies_reject_unknown_field  FAILED（5 passed/1 failed）
变异B 域检 0..=88 → 0..=MAX → enemies_reject_domain_overflow FAILED
变异C trait 范围 0..=255 → 关 → enemies_reject_param_overflow FAILED
变异D json id=49 trait 1→0 → enemies_trait_keys_spot FAILED（抽锚非空转）
$ git diff --stat   → 空（defs.rs/enemies.json 均还原）
```

⇒ 3 负例 + trait 抽查**全部真判别**（非恒真桩）；behavior 白名单**确在挡拼写静默**。

### 3.3 trait 源锚抽查

- `a.java:6266` `this.traitFlagsByType[49] = …[53] = …[74] = 1;`
- `a.java:6267` `this.traitFlagsByType[69] = 2;`
- 消费侧（既有，②未切）`scene.rs:3738 fn trait_flags` = `49|53|74=>1, 69=>2, _=>0`，与 json **三方一致**。
- json 全量 35 项 trait 与源稀疏赋值比对 `trait mismatch: []`。

### 3.4 黄因：声明 > 断言

`enemies_defs.rs:31` doc 注释写 **"五类拒载负例：未知字段/域溢出/trait 溢出/非整数"**（列 4 项）、
`:1` 模块注释写"五类拒载负例"——**实际负例只有 3 个测试，"非整数"负例不存在**
（`difficulty_multipliers` 非整数/缺数组路径、`hp` 非整数路径均无负例）。
commit message 写"五类负例×3"（带 ×3，口径尚诚实），但**文件内 doc 是超宣**。
其余类（BadJson/SchemaVersion/坏外键）由既有 `defs_negative.rs` 覆盖（`:272/:285/:296` 实读），
机制本身可信（`parse_envelope` 实读：BadJson→封套键集→SchemaVersion 三段俱在）；
缺的是**本件自写路径的非整数负例**。
次级附注：trait 范围越界复用 `Reject::FrameIndexOutOfBounds`（注释却写 "ParamsRange"）——
变体语义与"帧序"不符，属命名借位，可随 ② 换名或改注。
处置：随 amend 删去 doc 中"非整数"三字，或补一个 `hp: "x"` 负例（一并更优）。

## 4. 攻击点④：红线（cellSlotTable 未并 / sprite·drop 不并 / 消费面未切）——**黄**（三条红线实质全过；唯 defs.rs:387 引据反了）

### 4.1 cellSlotTable/族表未并 —— 实质 ✅，引据 ❌

- **实质过**：`EnemyDef` 字段恰 6 项（`id/hp/atk/def/gold/trait_flags`，`defs.rs:390-398` 实读），
  无任何 cell/slot/空间索引字段；`grep cellSlot crates/game-data/src/defs.rs` 仅命中注释行。
- **引据错（黄因）**：`defs.rs:387` 注 `⚠cellSlotTable/族表（空间索引与实例数据）按 spec v3 §0 裁定**不并入**`——
  1. **spec v3 §0 原文相反**：§0 只有两裁——"per-type 属性束一律聚合为记录"、"共享矩阵保留表形"，
     且点名 `enemyHp[]` 为 SoA 横表例、"存量迁移：**89 型怪族表**/道具描述表→per-type 记录"；
     §0 **从未裁定"族表不并入"**，更未点名 cellSlotTable。
  2. **"族表"在本仓术语=enemyBase HP/ATK/DEF/GOLD**（`dispatch-shape-survey.md:63` "族表 HP/ATK/DEF/GOLD"、
     `refactor-plan.md:42` "enemyBaseHp/Atk…表 保留独立（族表非实体实例数据）"）——
     而这四张表**正是本 commit 并入 EnemyDef 的对象**：注释与同一 hunk 内的代码自相矛盾；
     括注"（空间索引与实例数据）"亦与 refactor-plan 原文"族表**非**实体实例数据"相悖。
  3. 真正的历史引据是 `refactor-plan.md:41`（"cellSlotTable/Count 保留独立（空间索引，不动）"）
     与 `scene-module-decomposition-plan.md:90`（"cellSlotTable/族表不并入 Entity"，旧 Entity 语境）——
     且 `refactor-plan.md:41-42` 的"保留独立"已被 DDDSA §0 **就地取代**（后者 2026-11-03 更新）。
  ⇒ 建议 amend：把该行改成 `cellSlotTable/Count（空间索引，refactor-plan §布局）不并入 EnemyDef`，
  删去"族表/spec §0"字样，防后来者据误注以为四表不该聚合。

### 4.2 sprite/drop 无 SoA 源不并 —— ✅ 声明在且属实

- 声明在：commit message 正文"字段=hp/atk/def/gold/trait_flags（按现有 SoA 源全集——
  **sprite/drop 无 SoA 源不并**…）"；
- 属实（我方 grep）：Rust 侧 `grep -rni "enemy_drop|drop_table|loot" crates/` = **0 命中**；
  sprite 侧走 `sprite_defs.json`（独立 per-type V2 表，`SPRITE_KEYS` 已含 render_category 等），
  与敌方 SoA 簇无交集；`EnemyDef` 确无 sprite/drop 字段。

### 4.3 消费面未切（①=纯数据面）—— ✅ ②未做的正常态

```
$ grep -rn "enemy_base_hp" crates/game-core/src/combat.rs
14:    pub enemy_base_hp: Vec<i32>,          ← SoA 仍在
60:        hp: base.enemy_base_hp.iter().map(|v| v * multiplier).collect(),
$ grep -rn "parse_enemies\|EnemyDef" crates/ --include=*.rs   （除定义与本件测试）
→ 仅 defs.rs 定义 + tests/enemies_defs.rs；game-core 零引用
```
`scene.rs:306-307/2311/2313/3642` 的 SoA 读写与 `t-41` 索引原封未动 ⇒ ①纯数据面、②切消费面未做 = **预期态**。

### 4.4 附注（跟进项，不阻本件）

`enemies.json` 尚未进 `FIDELITY_TABLES`/`gates/table_content_hash.json`（现 4 项：sprite/anim/sheet/items），
生成器 `gen_enemies.py` 也未接任何闸（`grep gen_enemies gates/` = 0），
且本件测试**未钉 hp/atk/def/gold 逐值**（只钉行数/id/mult/trait）——
即当前手改 hp 值可全绿通过。
按 **P1-1 先例**（`73bc9ea` 数据面 → `022dd3d gate: FIDELITY_TABLES 扩 items.json（门禁单列）`），
应随后**单独 commit** 补哈希闸 + 逐值锚（AGENTS §2 门禁单列）；②切消费面时应一并补非整数负例与逐值钉。

## 5. 攻击点⑤：验证（6 测试 + 金样 74 --check + 夹带净）——**绿**（附注：core"全量"贴件=lib 目标）

全部**独立复跑**（非采信贴件）：

| 判据 | commit 贴件 | 我方复跑 | 判 |
|---|---|---|---|
| enemies_defs | 6 passed | `6 passed; 0 failed`（6 测试逐名过） | ✅ |
| items_defs（互斥断言不回归） | 7 passed | `7 passed; 0 failed` | ✅ |
| core 测试 | `337 passed` | lib 目标 `337 passed; 0 failed; 2 ignored`；**全目标合计 489 passed / 0 failed / 2 ignored**（21 个测试目标=lib+20 集成，另 Doc-tests 0，22 条 test result 全绿） | ✅（附注） |
| 金样 | `[GREEN] 金样逐字节等价：74 场景 sha 全同` | `python3 tools/state_baseline.py --check` → `[GREEN] …74 场景 sha 全同`，EXIT=0 | ✅ |
| 行数对账 | `rows: 35 / 源长 35` | 见 §2（sed 实测 35/35/35/35） | ✅ |
| 生成器确定性 | `2c6260e0a02eea9d` | 重跑同 sha + `git diff` 空 | ✅ |
| 夹带净 | 4 文件 +835/-0 | `git show --name-only` 恰 4 件（defs.rs/tests/enemies.json/gen_enemies.py）；`grep ^(reference\|original\|gates\|data/naming)/` = **NONE** | ✅ |

**附注（标签偏窄，非数值假）**：贴件 `=== core 全量 === 337 passed` 实为
`cargo test -p game-core` 的**首条（lib unittests）** test result；真"全量"= 489
（lib 337 + 20 个集成测试目标 152，Doc-tests 0；`Running` 目标 21、test result 行 22）。
数值本身真实、全绿无回退；建议后续把标签写成 "core lib 337 / 全目标 489"。

**①不动消费面 ⇒ 金样应不变**：74 场景 sha 全同 ✓，与"纯数据面"预期一致（金样不含 enemies.json 读路径）。

## 6. 攻击点⑥：三纪律（$(cat) 终态制 + 申报=grep 计数实测）——**绿**

### 6.1 终态制（贴件=提交树，命令输出未手编辑）

逐项从**提交树**复现同值：`git show 1e00e14:` 版文件与工作树 sha 逐一相同
（gen `11250682a2b73cc1`、enemies.json `2c6260e0a02eea9d`）；
6/7/337/74/35/sha 六项贴件**全部独立复跑复现**（§5 表）。
commit message 含 `Evidence:` 与 `Confidence: 93` 字段 ✓。

### 6.2 申报=grep 计数实测

| 申报 | 我方实测 | 判 |
|---|---|---|
| `a.java enemyBaseHp 源长: 35` | `sed -n '/enemyBaseHp = new int\[\]/,…/p' \| grep -cE '^\s*[0-9]+,?\s*$'` = **35**（四表同法各 35） | ✅ |
| `lead 89 = TYPE 域宽 0..=88 误非敌表长` | `TYPE_ID_MAX=88` + `型号域 0..=88` panic 字面 + sprite_defs 实测 89 行 | ✅ |
| `difficulty 5 项` | 源 `{1,30,60,80,100}` 长 5，json 同 | ✅ |
| trait `49/53/74=1、69=2` | `a.java:6266-6267` 逐行实文 + 全量 35 项 0 差异 | ✅ |

附注：贴件中 `rows: 35 difficulty: 5` 两行系 ad-hoc 命令（仓库无对应脚本），
但其值我已用独立实现复现同值——可复现性成立。

## 7. 总判

**可推送（红 0 / 黄 2 / 绿 4）**——数值面、行数对账、判别力、红线实质、验证、三纪律六项全过硬；
两黄均为文档/注释级（③测试 doc 超宣"非整数"负例；④`defs.rs:387` 把 §0 引反并与自身并表行为矛盾），
**不改任何数值、不改任何行为、无回退风险**，且 commit 未推送 ⇒ 建议 lead 推送前 amend 两处（成本≈两行注释）。

### 7.1 我方执行的验证清单（全部本机独立复跑）

```
python3 tools/enemy-defs/gen_enemies.py            # 确定性：同 sha + git diff 空
python3 <全量对拍脚本>                              # 35×4+mult+trait → mismatch []
sed -n '/enemyBaseHp = new int\[\]/,…/p' | grep -c  # 四表各 35
cargo test -p game-data --test enemies_defs        # 6 passed
cargo test -p game-data --test items_defs          # 7 passed
cargo test -p game-core                            # lib 337 / 全目标 489 passed, 0 failed
python3 tools/state_baseline.py --check            # [GREEN] 74 场景 sha 全同, EXIT=0
变异 A/B/C/D × cargo test（删白名单/关域检/关范围/改 trait）→ 各自红 → 还原树净
python3 gates/cli.py check                         # 总闸（见 7.3）
```

### 7.2 跟进项（不阻本件，建议排期）

1. **③** `enemies_defs.rs:31` doc 删"非整数"或补非整数负例（`hp:"x"` / `difficulty_multipliers:[…,"x"]`）。
2. **④** `defs.rs:387-388` 注释改引 `refactor-plan.md:41`（空间索引不并）+ 删"族表/spec §0"字样。
3. **锚勘误**：`a.java:2236` → `2234`（commit message / enemies.json note / 测试断言消息 / combat.rs:13,19 五处同改，一次勘误流程）。
4. **闸跟进（P1-1 先例 73bc9ea→022dd3d 门禁单列）**：`enemies.json` 入 `FIDELITY_TABLES`+哈希基线、
   `gen_enemies.py` 接 `--check` 确定性闸、测试补 hp/atk/def/gold 逐值钉（现手改数值可全绿）。
5. ②切消费面时一并把 `scene.rs:3738 trait_flags` 硬 match 换读 `EnemyDef.trait_flags`（双真相源收敛）。

### 7.3 总闸（我方独立复跑，非采信）

```
$ python3 gates/cli.py check
[✓ PASS] original-integrity / assets-manifest / reference-seed-integrity / knowledge-format
[✓ PASS] docs-links / naming-ledger / rust-tests / test-ratchet / table-content-hash
[✓ PASS] table-evidence（三表确定性复跑同哈希 + 等值全绿）
[✓ PASS] state-machine-anchors / asset-truth-manifest
[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同
== 汇总 == PASS=13 FAIL=0 SKIP=0        EXIT=0
```

附注：`test-ratchet` 报 `total=978(基线929) 无回退；新增二进制待 ratchet-update:
{'enemies_defs(test)': 6, 'items_defs(test)': 7, 'p15_zero_code_drill(test)': 3, 'arm_coverage(test)': 1}`
——本件 6 测入待更名单，但 items/p15 等**既有件同样在列**，属基线滞后（929→978 待一次性更新），
非本件独有红；`table-content-hash` 通过=因 enemies.json 尚未入闸（见 4.4 跟进项 4）。
