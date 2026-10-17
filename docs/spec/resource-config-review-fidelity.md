# 对抗复核 2/3：resource-config-survey.md（保真红线攻击面·敌方视角）

> 复核人：boss-evidence（**多路复核 2/3**，攻击面 = 保真红线/验证）。
> 对象：[resource-config-survey.md](resource-config-survey.md)（188 行）。
> 同轮兄弟件：[resource-config-review.md](resource-config-review.md)（3/3 工程排期面，
> interact-evidence）。格式：每攻击点「风险 / 反例 / 建议修正」；末尾总判定 + 条件。
> 口径体例（lead 已裁）：场景计数必标口径——**55 = state-dump 金样面**（fx18+preset30+7）、
> **45 = oracle 三方对拍面**、帧夹具 = 夹具场景面；"55 vs 45"=不同测试面不矛盾（D2 归源
> 有效），文档易混批评成立、计数标注体例采纳。
> 证据口径：a.java:448-453/485-586/6300-6330 逐行、entity.rs 状态臂逐行、schema 草案
> 逐字段、frames_fixture.py 头注。

---

## R1. 帧序/尺寸=原版事实：锚定完备性——粒度是**表级**不是逐值，且锚有一处实错

**风险**
- **evidence 列粒度不达 data/naming 体例**：体例精神=「一个符号一条记录，证据独立可查」；
  schema 草案 sprite_defs 行的 evidence = `"a.java:498-586 (animRowByType);
  a.java:6300-6325 (dispatchSpriteSize)"`——**89 项 animRowByType 共享同一范围锚**，
  行行相同。逐值锚应为「index→值→行号」三元组（如 task-3 手册体例 `idx26 =
  a.java:1071 = 1200`）；表级范围锚在转录出错时无法指认"哪个值错在哪一行"。
- **锚错位实锤**：schema `sheet_index.json` 注 `"evidence": "a.java:451 (container order)"`——
  **容器名序 = resourceNames = a.java:450**；451 是 `resourceImageCounts = new int[]{8,1,12,…}`
  （张数表）。§4 的"a.java:451/450-453"混写同病。主张"逐值锚"的文档里锚本身错一行——
  恰是粒度不足的症状。
- **schema 漏收已清点数据**：①**resourceImageCounts**（a.java:451，16 容器张数
  `{8,1,12,4,13,23,10,20,25,6,2,2,12,20,1,2}`）——sheet_index.json 有 containers/slots
  但**无张数**，ui[8][*]=25 张的槽位合法性/图集裁切边界无所依；②**SEG67_PIECES/
  SEG67_FRAMES**（表中表：9 片件 + 3×5 段帧）——§1 清点"照例外置"但 schema 三表
  **无片件表形状**（anim_row_override/size_override 装不下带 xy 偏移的片件数组）；
  ③type_width_px.txt 现有数据文件迁入 sprite_defs 时 values 的 evidence 补录义务未列。

**反例**
- 转录错例：animRowByType 89 项里把型 67 与 69 的行值对调（相邻、同表、肉眼难辨）——
  表级锚同指一范围；金样 55 场景（state-dump 面）**不含帧序断言**；帧夹具面若未含
  67/69 渲染窗 ⇒ 全绿带错入库，且事后范围锚无法定位到行。
- resourceImageCounts 缺失例：sheet_index 给 "ui" 放 `[*]` 槽——裁切循环边界靠什么定？
  实现者三选一（读 png 尺寸/硬编码 25/新表字段）= 又一次未锚定决定。

**建议修正**
- evidence 列升级**逐值锚**：`{index, value, a.java:line}` 三元组、生成式产出（配 R2
  机械对照器）；修 451→450 锚错位（连带 §4 混写勘误）；
- schema 补齐：`sheet_index.frame_counts`（resourceImageCounts 逐值锚）、`piece_defs`
  （SEG67 形态：`{sheet_idx, dx, dy, sx, sy, w, h}` + 段帧表）、type_width_px 迁移行
  evidence 补录（源头 typeFrameWidth 表锚）。

## R2. 金样帧夹具策略：「同表值=同渲染字节」验收闸**前提含转录**，覆盖面有洞

**风险**
- 验收闸证明力=「新系统（喂转录值）渲染 == 旧金样（原版常量渲染）」——只对**被夹具
  覆盖的渲染窗**成立；**未覆盖型/帧序无字节级对拍**：帧夹具面只渲染少数型，state-dump
  55 场景（金样面）**不断言帧序**——转录错误在未覆盖型上全绿通过。
- 步骤 1 "表==常量等值单测含负例"的**第二侧是谁**没写：迁移后原常量从代码删除；
  若 expected 侧=手抄表 ⇒ **两份手抄互证**（同源错误互相认可），等值单测变仪式。
  必须以**机械提取**（javap/deobf 解析的原表值）为第二侧。
- **保真表冻结无执行机制**："不可改（仅 evidence 勘误）"是政策不是闸（文件荣誉制）；
  且**帧夹具重生成红线未写**：勘误改表后金样红——是表改错还是该重生成夹具？必须写死
  "夹具期望字节仅源自 Java 参照帧提取（frames_fixture.py 头注）、Java 未变 ⇒ 夹具
  **永不重生成** ⇒ 红=表错"，否则重生成会把表错误洗成新期望（viewport 复核 V3 同款洞）。
- 计数口径混用：§4 "金样 55 场景 + 帧夹具 240×320 逐字节零差"未标口径（55=state-dump
  面/帧夹具=夹具面）——lead 已裁采纳计数标注体例，本调研应带头。

**反例**
- 两份手抄互证例：转录把 `ANIM_OFFSET_TABLE` 行 3 写成 `[0,1,2,0]`（原 `[0,1,2,1]`），
  expected 表由同一人同日手抄成同样的 `[0,1,2,0]`——等值单测绿、金样绿（该行的型未入
  夹具渲染窗）、游戏走路动画末帧错。
- 重生成洗白例：勘误改错 animRowByType[52] → 夹具红 → 有人启用"以当前 Rust 渲染为
  期望"的新档选项重新生成 → 红转绿、错入基线。

**建议修正**
- 等值单测第二侧 = **机械对照器**（javap/deobf 提取原表值 → 与保真表全量 diff，0 容差），
  对照器进 gates（保真表 vs 原版提取值 diff 非空即红 = 勘误流程的机械面）；
- 帧夹具重生成红线入 §4：期望字节仅源自 Java 参照帧，**表勘误红=表错**，禁"以实现
  渲染为期望"选项；
- 验收闸补覆盖面三层防线表述：机械对照器（全量转录正确性）/夹具字节（覆盖渲染窗）/
  金样状态（55 面逻辑态）——各管一段，不许互相冒充；
- 场景计数标口径（55=state-dump 面 / 25=夹具面 / 45=oracle 三方面）。

## R3. advance_frames 特例行：D4 quirk 是**状态条件臂**，静态 override 字段表达力不够

**风险**
- 实测特例行形态（entity.rs:393-425 状态迁移臂 + remove_des state=2 族，a.java:6332-6347
  语义=27×29 + animRow 8）：`state=1` 路径按 render_category 分派——`1|2|3 =>
  { sprite_h=56; anim_row=6 }`、`… => anim_row=2`、`2|4 => { … anim_row=10 }`、
  `8|16|32 => { … anim_row=6 }`。**事件条件行为**（触发=状态迁移/生成变体，条件=
  render_category），不是 per-type 静态值。
- schema 的 `anim_row_override: null` 是**静态 per-type 覆写**——装不下条件臂。两条
  出路都有坑：①强行静态化=**顺手规整**（D4 明令禁止，行为差）；②留码不表="零改码
  完整链"在这些型上不成立（新敌人走同类特例仍要改码）。
- §4"入表后保真表值原样（不顺手规整），注记 quirk 来源"——原则对但**表达形态空白**；
  实施者按 schema 草案静态字段做就违 D4。且消费替换**顺序契约**未写：查表初始化
  （dispatchAnimFields a.java:6327-6330）→ 状态迁移覆写（特例臂）的先后是行为面。

**反例**
- 构造：`remove_des` 的 state=2→row8 塞进 `anim_row_override: 8`（静态）——生成时
  （state=0）该型就带 row8 外观，与原版"生成行=animRowByType、DES 后才 row8"不同帧；
  夹具若不含该型生成窗=绿。
- 构造：特例留码但渲染已切表驱动——码里覆写若跑在查表**之前**，表值覆盖 quirk，
  quirk 静默失效（顺序耦合=行为面，步骤 3"逐点 RED→GREEN"须钉顺序）。

**建议修正**
- override 字段改**事件条件规则形**：`state_overrides: [{on: "mark_removed", by_category:
  {1: {row: 6, h: 56}, 2: {row: 10, h: 56}, 8: {row: 6}}, …}, {on: "mark_des_removed", …}]`
  （trigger→effect 行表），特例行逐条转译 + a.java 锚；
- 表达力不足者**显式留码**：`quirks_in_code: ["advance_frames…a.java:…"]` 留痕（D4 的
  诚实表达：不硬塞、不规整、有账）；
- 消费替换顺序契约：查表初始化 → 状态迁移覆写；RED 测按状态迁移事件断言 row 序列
  （生成→state1→state2 三拍）。

## R4. 两份制边界：改表金样红的**归因链**未建，且第三次挂 v2 悬空

**风险**
- **归因问题**（lead 问"改表后金样红归因能分清吗"）：两表若同 loader 合并装载（真游戏
  路径），mod 表同 type_id 行**覆盖**保真行 ⇒ 渲染变 ⇒ 金样红——红因是"重构回归"还是
  "mod 行生效"？调研未定：①命名空间/优先级（mod 禁覆盖保真 type_id？还是叠加？）
  ②金样运行时 mod 表装载策略（禁用/空表/独立命名空间）。
- **保真表冻结无执行**（R2 同）：需哈希门禁（original/ sha256 同款），流程不是闸。
- **v2 悬空 ×3**："两份制沿表设计 v2 §4.5 同"——表设计 v2 未拍板（state-machine 件 B、
  viewport 2c 同病）；且"enemy_defs 瘦身修订（sprite_ref 单字段）"是对 v2 的**修订建议**
  ——v2 不采纳则"一事实一宿主"破防（render_category/width 双写回归 = DC-9 桥漏字段
  同型漂移机制）。

**反例**
- 覆盖归因例：mod 表给 type_id=72 加行（换皮 frame_w=48）→ loader 合并覆盖保真行
  32×45 → 72 相关夹具全红——实施者先怀疑重构、烧半天 diff 渲染代码，真因在数据装载
  优先级。若金样禁 mod 表，此红不该出现（出现即 loader bug）——不写这条纪律，
  归因只能人肉。
- v2 不采纳例：enemy_defs 留 render_category + sprite_defs 也存 ⇒ 双写；改一处忘
  另一处 ⇒ 消费面取哪份成疑——又一个 DC-9。

**建议修正**
- 两份制四规则写死：①mod 行不得与保真 type_id 冲突（校验器拒）②金样/夹具运行
  **mod 表禁用**（装载参数显式）③保真表冻结=**内容哈希进门禁**（改=闸红，勘误走
  哈希更新流程）④mod 表独立基线名下断言（改表红=mod 基线红，归因天然分层）；
- 三案 v2 依赖统一显式 `blocked-by: 表设计 v2 决议`，或先落无依赖子集（保真表+
  机械对照器+哈希闸均不依赖 v2）；
- 一事实一宿主升级为**硬条件**：v2 不采纳 sprite_ref 瘦身 ⇒ 校验器断言 enemy_defs
  引用一致性（双写内容 diff=0），防 DC-9 同型漂移。

---

## 5. 附带一致性抽查

| 项 | 结论 |
|---|---|
| a.java:485-497（animOffsetTable 11 行）/498-586（animRowByType 89 项） | ✅ 与 task-3 取证区间吻合 |
| a.java:6300-6330 dispatchSpriteSize 4 臂（69→96×32/72→32×45/67 换容器/兜底 32×32） | ✅ 与 task-3 逐值吻合（6300-6325） |
| **a.java:451 (container order)** | ❌ **锚错位**：容器名序=resourceNames=**450**；451=resourceImageCounts（R1） |
| render_category 10 臂区间 | ✅ 与 entity.rs:242 吻合（72/84/87→32 族同 84 考证） |
| "真硬编码 ≈35-40 逻辑编辑点" | ✅ 分项自洽（5+4+20+2-3+2）；图集 ~20 调用点待逐点清单（实施期产物） |
| 55/45 场景计数 | ✅ 已裁不同测试面（D2 有效）；引用须标口径（R2） |
| 帧夹具=240×320 Java 参照帧 | ✅ frames_fixture.py 头注自证（重生成红线见 R2） |

---

## 6. 总判定

**有条件通过**——四类清点扎实、半表化现状判断准确、一事实一宿主裁定正确、零改码链
方向对；但保真红线面四洞：锚粒度表级且有一处实错（R1）、验收闸前提含转录+覆盖面有洞+
冻结无执行（R2）、D4 quirk 静态字段表达力不够（R3）、两份制归因链未建+v2 三度悬空（R4）。
任一洞落地 = "表错带绿入库"或"红无法归因"。

**条件清单（全满足才可实施）**：
1. evidence 逐值锚（index/值/行号三元组，生成式）+ 修 451→450 + schema 补
   frame_counts/piece_defs/type_width_px 迁移 evidence（R1）；
2. 等值单测第二侧=机械对照器并进 gates + 帧夹具重生成红线 + 验收闸三层防线表述 +
   计数标口径（R2）；
3. override 改事件条件规则形 + 特例行逐条转译留锚 + 不足者显式留码 + 消费顺序契约
   状态迁移 RED 测（R3）；
4. 两份制四规则写死（mod 禁覆盖/金样禁 mod/哈希门禁/mod 独立基线）+ 三案 v2 依赖
   显式化 + 一事实一宿主硬条件（R4）。

## 7. 核过的面（含挑不出问题的，留痕）

| 面 | 结论 |
|---|---|
| 动画面半表化现状（ANIM_TABLE_IDX/ANIM_OFFSET_TABLE/SEG67 先例） | ✅ 与 entity.rs/scene.rs 吻合 |
| 一事实一宿主裁定（真值宿主=sprite_defs/anim_defs） | ✅ 方向正确（v2 采纳风险见 R4） |
| game-data 归属 + L1 golden 门禁 | ✅ 与 AGENTS §2 红线一致 |
| 加新敌人零改码链（3-4 行表+资源） | ✅ 前提=特例行表形补齐（R3） |
| 分工边界（资源/数值/转移壳/视口四面切割） | ✅ 清晰 |
| render_category 区间 match=表形态 | ✅ 判断正确 |
| 工作量 6.5-7.5 天 + 合批排期 | ⚠️ R1-R4 条件落地另需 +1-2 天（机械对照器/哈希闸/条件规则形） |
| 尺寸真值不随视口（与分辨率调研切割） | ✅ 边界正确 |
