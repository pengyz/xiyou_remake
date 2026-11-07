# 数据驱动的领域子系统架构（DDDSA）设计书 v3（轮 2 独立复核修复终稿）

> 2026-11-03。v1 经轮 1 三角度对抗（A 架构适配/B 验证兼容/C 编辑器工程，30+ 修订）
> +用户数据模型总则。v2=重基线+纠伪数+补盲区+立宪。证据源：
> docs/findings/reviews/2026-11-03-dddsa-attack-{adaptation,verification,editor}.md。
> 【轮2修复·2026-11-03】独立复核员（remake-verifier，不采信轮 1 结论、逐条自取证）就地升 v3：
> 修正 7 处陈旧/错引（锚行/计数/盲区口径/误读传播），§7 六问全部裁定写入，§8 附独立抽验表。
> 所有更正以"【轮2修复】"原地标注，不删历史主张（AGENTS §3 证伪更正文化）。

## 0. 数据模型总则（用户裁定 2026-11-03——本设计第一原则）

**per-type 属性束一律聚合为记录**（ItemDef/EnemyDef/TileDef…一行一实体）；**共享矩阵保留
表形**（ANIM_OFFSET_TABLE/图集索引/字形表/帧表）。判据：数据在答"某实体是什么"→聚合；
"某类资源怎么摆"→保留表。原版 SoA 类型表（itemDescriptions[]/enemyHp[] 按 id 横索引）
=Java ME 内存妥协，Rust 平台下只剩维护税（跨表插值漏一处=静默错）。
存量迁移：89 型怪族表/道具描述表→per-type 记录（值逐字迁+V2 逐值锚重生+金样护航）。
【轮2抽验 ✅】89 型=defs.rs:46-48 TYPE_ID 0..=88 实测；ITEM_USES[22]（scene.rs:4303）/
itemDescriptions[22]（scene.rs:4325）两张 SoA 横表实存。

## 1. ECS 适配性裁定（A 修正版）

| ECS 要素 | 裁定 | 修正后依据 |
|---|---|---|
| 组件=数据 | **采纳** | §0 总则的同义面；真正欠估扩展面=**属性行×楼层档参数化**（floorTier 类） |
| System 调度器 | 否决 | MOV 脚本 VM=既有调度器，另造=两套真相 |
| Archetype/Query | 否决 | 实体 20-50/层（buff 组合爆炸实测≈0——Entity 11 字段/三值 state） |
| 运行期动态（so 插件） | **否决**（C/B 同裁） | 编译期行为注册+加载期数据解析（wasm+确定性） |

【轮2抽验 ✅】Entity 11 字段/三值 state（entity.rs:19-42 逐字段清点：type_/param/px/py/
state/visible/removed/sprite_w/sprite_h/anim_row/anim_frame 恰 11，state 0/1/2 三值）；
floorTier 参数化=pickupItemType 29-32 臂（a.java:7565-7588，现行实测）。
【轮2修复】"实体 20-50/层"为 B 级量级估计（EntityTable cap=100，scene_persist.rs:74），
不作裁定依据；裁定依据=组合面≈0 与金样整表锚定 AoS（entity.rs:17 注）两条硬事实。

**编排归属**（A）：注册表只管**分发**；跨域编排（execute_item_action 跨 5 域/interact_with_cell
跨 7 域 208 hits）归 Coordinator——注册表不是编排垃圾桶。
【轮2抽验 ✅】execute_item_action 跨 5 域=MirrorTeleport 单臂实证（item_effect.rs:112-133：
interact_with_cell/place_player_at_cell/set_camera/apply_step_cell_effects/transition_to）；
interact_with_cell 208 hits/7 域（combat:67 entity:61 walk:36 visual:30 menu:9 script:3
route:2，域分解和=208）现 scene.rs:3413-3718 共 306 行。

## 2. 物品注册表（P1 落点——A/B 修正版）

### 2.1 数据 schema（items.json）
```json
{ "id": 15, "name": "…", "icon": 3, "desc": "…",
  "effect": "TutorialFlag", "params": { …强类型 schema… }, "pickup_msg": "…" }
```
- **强类型 params**（A 红修）：每 effect 的 params 有 schema（加载期校验转换），禁 JSON map 弱类型
  ——迭代 70 错帧=魔法数脆弱前科；
  【轮2抽验 ✅】前科实存：scene.rs:4276-4280 注"迭代 70 floor43 80 甲实证：传类型 80=
  D 表越界错帧"（A 级）；
- **表归属**（B 裁）：id<1000 内建行=**保真表**（table-content-hash 闸扩 FIDELITY_TABLES+
  勘误流程）；id≥1000=mod 行**分文件**+mod 独立基线；红线重述："保真面**不走 mod 装载**"
  （走注册表无妨，注册表本身两份制）。
  【轮2抽验 ✅】FIDELITY_TABLES 现=三表（gates/cli.py:406）；扩面=items.json 内建行入列。

### 2.2 行为注册（编译期注册+加载期解析——C/B 同裁）
```rust
trait ItemEffect { fn apply(&self, ctx: &mut Ctx, params: &EffectParams) -> ItemOutcome; }
// 内建效果集=Rust 67 行 match（a.java:7534-7633 共 100 行）逐字抽象——A 纠伪数：
// v1 "220 行 match"不实；且 use 路径已表驱动（item_effect.rs ITEM_EFFECTS 先例）
```
> 【轮2修复】锚行三更正（独立抽验，实测 deobf a.java 现行）：
> ① "a.java:7534-7633 共 100 行"**截断装备臂**——pickupItemType 方法实为 **7534-7649（116 行）**、
> switch 本体 **7537-7648（112 行）**；7633 切在装备臂 case 标签列中段，臂行为体
> （7642-7647 tryEquip/showOverlayMessage）被截出范围。"逐字抽象"的锚必须含装备臂；
> ② "Rust 67 行"✅ 实测成立：scene.rs:4225-4291 恰 67 行（fn 体；match 本体 4229-4286）；
> ③ v1"220 行 match"不实维持原判（无出处，删）。

- **未知 effect 语义**（B 纠 v1 错配）：item 域原版=default **no-op 不消耗**（a.java:7608-7620）
  ——"R-TODO-1b 停摆"是**卡死**语义属另一域，禁混谈。**加载期拒载**（schema 未知键）+
  **运行期 no-op 不消耗**（值合法但 effect 未注册=登记不执行，行为面测试钉）；
  > 【轮2修复】"a.java:7608-7620"**错引**：default: return 实为 **7630-7631**；
  > 7608-7620 是 fall-through case 标签列（61-73，同属 no-op 组但非 default 臂本体）。
  > scene.rs:4289 注释同源错引（锚漂移前科第三起——state-machine-anchors 闸"前科两次"之后），
  > P1 落地时随注释一并更正。两层语义（加载期拒载/运行期 no-op 不消耗）本身 ✅ 维持。

- **等价机器证明**（B）：ItemAction 三层回归先例（逐值锚+夹具+金样）——**每 dispatch 臂
  一条 Java 帧锚夹具**（现状 16/86 臂无夹具=先例留尾，P1 补齐）。
  > 【轮2修复】两处措辞纠偏：
  > ① "等价机器证明"过度承诺——B 原语=**三层回归等价（逐 case 注释锚+逐 case 帧夹具+
  > 金样 60），不是形式证明**；机器强度=夹具对该臂的覆盖度，改称"等价三层回归"；
  > ② "16/86 臂无夹具"语义歧义致 §7-Q3 误读为"86 臂总数"——B 原文是**型 16（三昧真火）/
  > 型 86（KillAdjacent4）两臂无专属帧夹具**。P1 臂面实数与成本裁定见 §7-Q3（全量 21 臂组，
  > 禁抽样；净补 ≈11 条；型 32"出现未拾"销账 pre-final-audit:122）。

### 2.3 注册表扩展面（C 收紧）
- P1 **仅 ItemRegistry**；EnemyRegistry 后置（quirk 路由/域外三界复杂面）；ShopGoods 批 P2 尾；
  TileRules 已表化移出清单。
  【轮2补充】EnemyRegistry 后置的**可测触发条件**见 §7-Q5（zero-code-drill"纯表行零改码"
  判据被击穿=启动信号；不设日历触发）。

## 3. 子系统 trait 通信面（P2 落点——A/B 修正版）

### 3.1 trait 面（A 补漏后八件）
```rust
trait BattleSubsystem { fn resolve_bump(&mut self, ctx: &mut Ctx, t: EnemyTypeId) -> BattleOutcome; }
trait DialogSubsystem { fn tick(&mut self, ctx: &mut Ctx) -> DialogStep; }
trait PersistCodec    { fn serialize(&self, st: &SceneState) -> Blob; }
trait ScriptHost      { fn execute_op(&mut self, ctx: &mut Ctx, op: ScriptOp) -> ScriptStep; }
trait ItemService     { …与 ItemRegistry 职责合并/明示（C Q3）：Registry=分发、Service=pickup 编排… }
trait InputMapper     { fn map(&self, ev: RawInput) -> Action; }          // A 补
trait EntityService   { fn spawn/remove/relocate(&mut self, …); }         // A 补（HostCtx 23 方法 fat trait 前车=拆小）
trait UIRender        { fn snapshot(&self, …) -> UiFrame; }               // A 补：显式纯读
```
> 【2026-11-07 更正】P2 批0 落码与本节两处差异（plan 已裁定，原文不删备查）：
> ① PersistCodec 实现为 serialize+deserialize 双向（plan §2 批4「Blob↔域切片格式编解码面」），
> 本节单行仅列 serialize；② UIRender::snapshot 实现为 `snapshot(&self) -> UiFrame` 无参
> （plan 批8 锚行），本节 `…` 为参形未定占位。以上为 plan 授权收窄/扩展，非偏离。
> 【2026-11-07 更正·批2】③ BattleSubsystem::resolve_bump 实现为 `resolve_bump(&mut self, ctx, t, ev: BattleEvent) -> BattleOutcome`（+ev 参——plan 批2「scene 战斗路径编排面→BattleOutcome」三事件面[途中开战/撞击开战/一击交换]需事件分辨；ctx+≤2 恰满 t+ev）。原文单 (ctx,t) 形不删备查。
> 【2026-11-07 更正·批3】④ DialogSubsystem 实现为 3 方法（tick+set_typewriter/m_082_set_dialog_target——Host 边界 2 方法自 HostCtx 逐字迁入，lead 批3 派工「HostCtx 再收窄的面逐字迁+界收窄」授权；tick 形不变，ctx+≤2 不涉）。原文单 tick 形不删备查。
> 【轮2修复】四处注记：
> ① **ItemService 切法终案落书**（C Q3 终结，详见 §7-Q1）：Registry=静态查表面（分发），
> Service=道具生命周期运行面（pickup/use 编排），分发链唯一，跨域副作用归 Coordinator；
> ② "HostCtx **23** 方法"→实测 **21 方法**（script.rs:128-168 逐 fn 清点；轮 1 计 23 亦误）。
> fat trait 前车结论不变；
> ③ **UIRender=只读边界件**，列此清单为消除图文不一致（A 裁"有意排除"口径），**不入子系统
> 互调通信面**——渲染消费状态快照，不参与 trait 互调；
> ④ **PersistCodec 只管存档格式编解码**（Blob↔域切片）；存档行为（PersistState 状态机，
> scene_persist.rs 已落）不入 trait 面（A §3.2 拆界）。

### 3.2 Ctx 宪法（A 立宪防上帝袋——HostCtx 前车）
1. **纯数据**（无方法/无行为）；2. **字段冻结**（改 Ctx=显式 schema 变更流程）；
3. **ctx+≤2**（每 trait 方法参数=ctx+至多 2 个语义参数，违者拆方法）。
> 【轮2修复·并存口径+写面归因二执行则（§7-Q4 裁定）】：
> 4. **与 too-many-args 并存**：两机制作用域不同不互斥——task-15 too-many-args **8 处指名
>    清单**（load_floor_from_save/paint_overlay_box/draw_region_scaled/register/
>    draw_edge_patch/paint_wrapped_lines/relax_path_neighbors/tick_stepping，plan §2.3 E2）
>    =存量静态函数签名**冻结不追溯**；Ctx 三宪只约束 **P2 新增** trait 方法签名（新增面不豁免）；
>    trait 方法触达 8 处同域逻辑=经 Coordinator 适配调用（8 处签名不动）。
> 5. **写面归因**：Ctx 字段按 data/status/field-ownership.json **唯一归属**（task-15 既落件）；
>    trait 方法只写本域字段，跨域变更返回 outcome 由 Coordinator 落笔（C2 编排注记归因）；
>    Ctx 字段写集清单与 state_dump FLD 对照登记——禁"直写换马甲"（DC-9 桥漏字段教训）。

### 3.3 P2 判据补件（A/B）：scene_tests 6,098 行直填构造面=风险——P2 验收含
"测试构造面收敛"（夹具直填→域构造器）+**迁移禁双轨**（B：逐臂一次性切，match 删除与
表行同 commit（313375b 先例），同 id 双源=违规）。
【轮2抽验 ✅】scene_tests 6,098 行（wc 实测恰 6098）；313375b 实存（SoA→AoS 427 站点
一次落毕、金样零漂移——"一次性切"形态先例）；"同 id 双源=违规"与 field-ownership
"双写=同字段写点跨归属域文件"同源，无冲突。

## 4. 编辑器（P3 落点——C 重写）

| 面 | 裁定 |
|---|---|
| 形态 | **本地 Python 服务**（tools/editor/server.py）——静态页+File System Access 不可移植（CORS/写前无校验缝）；JSON 落盘**必经服务端校验闸** |
| 校验防漂移 | **Rust 唯一真值**（defs.rs 拒载）+schema 从 Rust **单源生成**+JS 镜像仅 UX 缓存（**夹具等值测试**仿 zero-code-drill 真解析器对照，禁镜像自证） |
| 验收 | 零改码演练**机器化四段**（编辑器产出→真校验器拒载闸→game-oracle 无头加载断言→零改码+哈希双查）——**禁实机试玩充当验收** |
| 保真隔离 | 同 §2.1 两份制（mod 行分文件+独立基线） |

【轮2抽验 ✅】零依赖地基实存：tools/zero-code-drill/drill.py（真解析器对照+零改码断言
先例）；defs.rs 五类拒载（BadJson/BadForeignKey/DomainOverflow/FrameIndexOutOfBounds/
UnknownField）+SCHEMA_VERSION 严格键集。本节与 C 攻审裁定逐条一致，无修订。

## 5. 迁移三阶段（B 补验证前置件——估算修正版）

| 阶段 | 内容 | 估（含验证前置件 +1-2d） | 前置件（B） |
|---|---|---|---|
| P1 物品注册表化 | §2 落地+**dump 补面**（B 红修：state_dump 对道具栈/overlay/tutorialSeen 全盲——A2 重冻结或盲区清单+夹具断言）+**每臂 Java 帧锚夹具** | 4-7d | effect 集冻结（A）；dump 盲区清零 |
| P2 子系统 trait 化 | §3 落地+直写清零收尾+测试构造面收敛 | **8-15d**（A 修，原 5-8d 不可信） | **直写清零在跑=前置**（C） |
| P3 编辑器 MVP | §4 落地+全流程演练 | 5-8d（items 单表）/8-12d（四表） | zero-code-drill 扩展 |

> 【轮2修复·dump 补面口径两更正+清单完备（§7-Q2 裁定）】：
> ① "overlay **全盲**"不实——overlayActive/overlayKind **在面**（state_dump.rs:79-80）；
> 真盲区=overlay 文案/布局面（overlayText、boxW/H、left/right icon、lines、分页、savedColor）。
> 道具栈（itemStackTypes[]/itemStackUses[]/itemStackSize）与 tutorialSeen 盲 ✅ 维持；
> ② "3 项盲区"不完备——**P1（item 域）必清 6 组**：道具栈三字段（scene.rs:580-584/
> 2386-2407 写集）、itemUseTargetType（=menu.overlay_kind_item，双写点 4227/4319）、
> tutorialSeen（=menu.tutorial_seen）、scriptLineFlags[]、overlayText（文案本体）。
> 其余域盲区（dialog 域 typewriterPos/dialogSpeaker/dialogAnchor/dialogTarget/overlay
> 布局面、menu 域 menuCursor/itemCursor/recordPage/difficultyIndex、persist 域
> min/maxFloor/pendingFloor/wentDown、tap 域 autoBattle 等）=**P2 盲区清单随域迁移登记**，
> 禁默认无清单；
> ③ 补面机制钉死：A 侧 trace=反射全集（实测 409 名，含上述盲区名），statediff 名集相等
> （2026-10-16 防静默绿修复）即完备性守卫——补面=Rust 侧 dump 追加（**A2 行序冻结：追加于
> 标量段尾+金样重冻结，E3 先例**）→ 名集两侧相等+金样可见；不能进 dump 的字段走
> "盲区清单+夹具内断言"兜底（B §1.3-b）。

**验证预算分层（R5 口径原样写入——B 前置件 6）**：每 commit=单测 ≈13s；每步=state_baseline
--check（60 场景 dump sha）≈4min；步末/里程碑=Java 三方 47 场景≈18min+`gates all`。
"每步金样全同"专指前者（4min 面），**P1 判据=4min 面+18min 面双绿**，禁只跑其一。
【轮2抽验 ✅】金样 60 场景（state-dump-golden.json 实测 60）；47 场景=纯 Java 三方
（reference/oracle/scenarios 恰 47）——两套面不同，不得混称。

**回退设计**（B 补）：整步 commit revert+基线 ratchet（禁 fix-forward）——task-15 纪律沿用。
回退单位=P1/P2 内部**原子步**（≤0.5d 可操作），非整阶段；步 revert 连新增测试一起回
（禁只回实现留新测试=恒红）。

## 6. 红线（终版）
1. classic 零漂移（金样 60+state_dump 补面后全同每步实证）；
   【轮2修复】口径钉子：每步=state_baseline --check（60 场景）；步末=Java 三方 47 场景
   +gates all（§5 验证预算分层）；
2. 逐臂一次性切禁双轨（B）；
3. 强类型 params+加载期 schema 校验（A）；
4. Ctx 三宪（A）+二执行则（并存口径/写面归因，§3.2-4/5）；
5. 保真表 id<1000 哈希闸扩面+勘误流程；mod 行分文件（B）；
6. 行为面测试纪律：每 dispatch 臂 Java 帧锚夹具（B）；
   【轮2修复】臂面=21 臂组清点（§7-Q3）；型 16/86 与拾取入栈臂=P1 必补，禁分层抽样；
7. 域测只断言 a.java 可回指不变量（task-15 纪律沿用）。

## 7. §7 六问轮 2 裁决（原问题保留——裁定直接写入）

1. **ItemService/ItemRegistry 职责合并的具体切法（C Q3 未给终案）**
   【轮2裁定·终案】**不合并，两层明示**：
   - **ItemRegistry=静态查表面**：加载期构建（内建行+mod 行两份制），id→ItemDef{强类型
     params、行为指针、uses 预算、展示列}；只读 lookup，零副作用零运行态；
   - **ItemService=运行期道具生命周期面**：pickup（入栈/uses 叠加/描述 overlay 触发）与
     use（消耗判定/uses 递减/栈压实/触发行为序）；
   - **分发链唯一**：Service→Registry::lookup→ItemEffect/ItemAction 序执行（现
     item_effect.rs execute_item_action 形态保持）；禁第二条分发链（match 与表并行=
     双面，gotcha_dual-porting-surfaces-drift 前科）；
   - **编排边界**：跨域副作用（interact_with_cell/换层/相机）经 Coordinator 落笔+编排
     注记归因（C2）；字段归属照 field-ownership.json 不动（player 栈=entity 域、
     tutorialSeen/overlay_kind_item=menu-ui 域、equip=item-effect 域）。
   - 可测判据：Registry 单测=查表+拒载（无副作用断言）；Service=pickup/use 生命周期夹具；
     编排=git diff 断言（新 EffectKey 零碰旧码）+注记归因。

2. **dump 补面清单的完备性（B 列 3 项盲区——还有吗？grep 实测）**
   【轮2裁定】B 的"3 项"不完备且 overlay 口径错，grep 实测完备清单见 §5【轮2修复】注②：
   P1 必清 6 组（道具栈三字段/itemUseTargetType/tutorialSeen/scriptLineFlags[]/overlayText），
   P2 盲区清单随域迁移登记（dialog/menu/persist/tap 域余项）。机制=A 侧反射全集（409 名）
   vs Rust 54 名，statediff 名集相等为完备性守卫；补面走 A2 行序冻结协议（E3 先例）。

3. **P1 每臂夹具（86 臂→86 条？分层抽样 vs 全量）的成本裁定**
   【轮2裁定】"86 臂"不成立——系 B 原文"型 16/型 86 两臂无夹具"被 v2 压缩歧义、再被本问
   误读为总数。**臂面实数=21**（pickup dispatch 11 臂组：13..=25|85|86、26、27、28、29、
   30、31、32、33..=36|79、37..=40|80、default no-op + ITEM_EFFECTS use 10 行）。裁定：
   **全量按臂组 21 条补齐，禁分层抽样**（抽样=覆盖暗角，verifier 纪律禁）；已有 ≈10 条可复用
   （fx-item17a/b、18a、19a、20a/b、preset-floor5 13/14、preset-item21/2122、fx-boss67a、
   fx-boss68a/b）；净补 ≈11 条+型 32"出现未拾"销账。成本 +1-2d 已含 P1 4-7d 区间，不另加。

4. **Ctx 三宪与 task-15 too-many-args 永久保留的并存口径**
   【轮2裁定】并存不互斥，作用域分列（详见 §3.2 执行则 4/5）：too-many-args 8 处指名清单
   =存量静态函数签名冻结不追溯；Ctx 三宪=P2 新增 trait 方法签名约束（不豁免新增面）；
   两者共同服务"依赖面显式可审"。写面归因（field-ownership 唯一归属+outcome 归 Coordinator
   落笔）防 Ctx 成为直写换马甲。

5. **EnemyRegistry 后置的触发条件（何时启动）**
   【轮2裁定】**可测触发，不设日历**：当 zero-code-drill"新敌人=纯表行零改码"
   （`git status --porcelain crates/` 空断言）被需求**击穿**（新敌人行为必须改 Rust=
   行为钩子需求出现），或 P3.5 敌人编辑面立项裁"行为键"时启动；形态与 ItemRegistry 同式
   （编译期行为注册+加载期数据解析），quirk 路由（QuirkPathRequired 族）/域外三界
   （ANIM_TABLE_IDX 89/容器 16/张数表）/型号命名空间（<89/89..999/≥1000）单独立项估算，
   不搭 P1-P3 便车。89 型属性行照旧 task-5 三表+combat.rs 纯计算，不等注册表。

6. **v2 内部一致性终核**
   【轮2裁定】终核发现 7 处不一致/陈旧，全部已修：①a.java 锚 7534-7633 截断装备臂；
   ②7608-7620 错引 default；③HostCtx 23→21 方法；④"16/86 臂"歧义与本问"86 臂"误读
   （连锁）；⑤"overlay 全盲"与 state_dump.rs:79-80 自相矛盾；⑥"3 项盲区"不完备；
   ⑦"等价机器证明"过度承诺。修复后互引核对：数据模型总则↔注册表（SoA 表迁移+强类型）、
   注册表↔trait 面（分发链唯一/编排归 Coordinator）、trait 面↔编辑器（schema 单源生成
   经 Registry 面）、迁移↔红线（一次性切/金样口径/臂面清点）、全部↔task-15 既落件
   （field-ownership 唯一归属/直写清零前置/too-many-args 冻结）——**无矛盾，每阶段
   判据均可测**（P1：dump 补面+21 臂夹具+双面金样；P2：构造面收敛+直写清零+gates all；
   P3：四段机器化演练）。

## 8. 轮 2 独立抽验记录（自取证——不信轮 1 结论，逐条实测）

| # | 声称 | 抽验方法 | 判定 |
|---|---|---|---|
| 1 | Rust 67 行 match | wc/sed scene.rs:4225-4291 | ✅ 恰 67 行（match 本体 4229-4286） |
| 2 | a.java:7534-7633 共 100 行 | sed deobf a.java 全 switch 清点 | ❌ 截断装备臂→修（7534-7649/switch 7537-7648） |
| 3 | default no-op a.java:7608-7620 | awk 定位 default: return | ❌ 错引→修（7630-7631） |
| 4 | HostCtx 23 方法 | awk trait 块 fn 清点（script.rs:128-168） | ❌ 21 方法→修 |
| 5 | state_dump 道具栈/tutorialSeen 盲 | grep item_stack/item_types/tutorial_seen =0 命中 | ✅ 盲 |
| 6 | state_dump overlay 全盲 | grep overlay → :79-80 在面 | ❌ 半盲→修 |
| 7 | ITEM_EFFECTS 先例 | grep item_effect.rs:32-45 | ✅ 10 行表/7 变体实存 |
| 8 | 金样 60 场景 | json 实测 | ✅ 60（11-02 时 56，v2"60+"非陈旧） |
| 9 | scene_tests 6,098 行 | wc | ✅ 恰 6098 |
| 10 | interact_with_cell 208 hits/7 域 | 域分解和=208；scene.rs:3413-3718 共 306 行 | ✅（307/306 行差 1 行计法，hits 口径成立） |
| 11 | Entity 11 字段/三值 state | entity.rs:19-42 逐字段 | ✅ |
| 12 | execute_item_action 跨 5 域 | item_effect.rs:112-133 MirrorTeleport 臂 | ✅ 五副作用面 |
| 13 | 313375b 一次性切先例 | git log | ✅ 实存（SoA→AoS 427 站点金样零漂移） |
| 14 | 迭代 70 错帧前科 | scene.rs:4276-4280 注 | ✅ |
| 15 | FIDELITY_TABLES 现三表 | gates/cli.py:406 | ✅ |
| 16 | task-15 既落件无冲突 | field-ownership.json 11 归属域/187 字段唯一归属；too-many-args 8 处；直写清零在跑 | ✅ 与 §3.2/§5/§7-Q4 一致 |

**完成判据（本复核）**：v3 就地落盘；7 处修复全带【轮2修复】原地注记；§7 六问裁定写入；
抽验 16 项方法可复现（wc/grep/sed/git 命令如列）；无 C 级猜测混入裁定（"实体 20-50/层"
已降级标注）。
