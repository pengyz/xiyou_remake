# DDDSA P2 子系统 trait 化——切片实施计划

> 2026-11-07。起草：v2-consumer-swap（remake-porter）；派工：lead（P1 全段收官后 P2 首步）。
> 状态：**计划稿——先过对抗复核，后动手**。红线：本件不写代码只出计划；
> P1 已推基线（ratchet 978/金样 74/闸 13/13）为起点勿动。
> 依据：`docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md` v3
> （§3 八 trait 面/§3.2 Ctx 宪法+二执行则/§3.3 测试构造面收敛/§5 估算 8-15d 与
> 验证预算分层 R5/§6 红线/§7-Q1 ItemService 终案、Q4 并存口径）。

---

## 0. 范围与不做什么

**做**：§3.1 八 trait 面落地（子系统通信面解耦）+HostCtx fat trait 拆小（21 方法前车）
+ShopGoods 注册表化（spec:89"ShopGoods 批 P2 尾"）+直写清零收尾+测试构造面收敛
+P2 盲区清单随域迁移登记。
> 前置件声明（spec:155"直写清零在跑=前置（C）"）：**已达成**——2026-11-07 实测
> `python3 tools/field_ownership.py --report` 定案口径"生产面 self 直写跨文件=0"。
**不做**：P3 编辑器（tools/editor）；EnemyRegistry 正式化（§7-Q5 可测触发未击穿）；
行为改动（保真零漂移红线 §6-1）；存量静态函数签名追溯改写（too-many-args 8 处冻结）。

---

## 1. 现状普查（只读实测 2026-11-07）

### 1.1 八 trait 面现状对照

| # | trait（spec §3.1） | 方法面 | 现状实测 | 缺口判定 |
|---|---|---|---|---|
| 1 | BattleSubsystem | `resolve_bump(&mut self, ctx, t) -> BattleOutcome` | combat.rs 198 行 8 fn（predict_battle_hp_loss/scale_enemy_stats/trait_flags_for/effective_attack_vs_type 等）+scene.rs 战斗路径（2053/2060/3730-3732/3756/3772/3642） | 无 trait 面=函数族+scene 内联编排 |
| 2 | DialogSubsystem | `tick(&mut self, ctx) -> DialogStep` | scene_dialog.rs 263 行 9 fn（DialogState+advance_typewriter 已域化 +tick_dialog 编排注记归因） | 状态已域化；**trait 面缺** |
| 3 | PersistCodec | `serialize(&self, st) -> Blob`（+deserialize） | scene_persist.rs 585 行 9 fn（serialize_slot_blob 等）+save.rs 338 行 15 fn | 格式编解码与存档行为混居（spec ④：**行为不入**，只切编解码） |
| 4 | ScriptHost | `execute_op(&mut self, ctx, op) -> ScriptStep` | script.rs HostCtx（**21 方法** fat trait，script.rs:128-166 trait 主体逐 fn 清点=21 ✅与 spec ②同）+script_ops.rs 575 行（ScriptOp enum+parse_op/execute_op 已汇 1） | fat trait 前车=**拆小**；execute_op 已有形但无 Ctx 面 |
| 5 | ItemService | pickup/use 生命周期编排 | effect_registry.rs 748 行（ItemEffect trait+builtin 注册 P1 已落）+item_effect.rs 159 行（ItemAction 表）+scene.rs pickup_item_type | Registry=分发已立（§7-Q1）；**Service 编排面未立** |
| 6 | InputMapper | `map(&self, ev: RawInput) -> Action` | controls.rs 703 行 22 fn+tap_walk.rs 685 行 40 fn | 输入解析散两域；**RawInput→Action 抽象缺** |
| 7 | EntityService | `spawn/remove/relocate(&mut self, …)` | entity.rs 621 行 28 fn+sprite_spawn.rs 55 行 | HostCtx 实体方法（spawn_entity/remove_entity/remove_all_of_type/entity_pixel/cell_entity/entity_type/swd_convert_type81/reload_floor_entities/set_entity_route_flag）**=fat trait 待拆** |
| 8 | UIRender | `snapshot(&self, …) -> UiFrame`（纯读） | render.rs 1493 行 97 fn+scene_paint.rs 925 行（E3 RenderState 六字段已切） | 纯读 snapshot 面缺（spec ③：**只读边界件，不入互调通信面**） |

### 1.2 调用面切点（scene/game-core 侧）

- **HostCtx 21 方法拆解账**（fat trait 拆小唯一对象）：
  - 实体面 **10**：find_path/cell_entity/entity_type/remove_entity/remove_all_of_type/
    swd_convert_type81/entity_pixel/spawn_entity/reload_floor_entities/
    set_entity_route_flag → **批1 EntityService**（find_path 留 pathfind 域视归属）；
  - 道具/经济面 2：give_gold/pickup_item → **批6 ItemService**；
  - 相机面 4：snap_camera_to_player/ces_camera/set_camera_anchor_target/
    snap_camera_after_ros1 → Coordinator（相机域，不入 trait 面——域方法已存）；
  - 对话面 2：set_typewriter/m_082_set_dialog_target → **批3 DialogSubsystem**；
  - 系统面 **3**：exit_application/mvs_exchange/locate_speaker_entity
    （跨界件：lockCameraOn 调用/写 dialogAnchorEntity——批3 注记归属）→
    Coordinator/平台边界（ProcessControl 已有 trait）。
  - 计数对账：10+2+4+2+3=**21**（script.rs:128-166 sed 实文逐 fn 清点）。
- **scene.rs 编排面**（协调器：状态机+域装配——出处指针
  `docs/superpowers/plans/2026-11-02-scene-module-decomposition-plan.md:42`；
  域文件编排注记归因=task-15 落件）：
  跨域编排留 Coordinator；trait 方法只写本域字段（field-ownership.json 唯一归属），
  跨域变更返回 outcome 由 Coordinator 落笔（§3.2 执行则 5）。
- **测试构造面**：scene_tests/ **6,127 行**（12 文件，2026-11-07 `wc -l` 实测；
  spec §3.3 引 6098=2026-11-03 抽验值，cb710f4 后=6127）中**直写站点 382 处**
  （宽口径 `grep -rEn "\b[a-z_][a-z_0-9]*\.[a-z_][a-z_0-9]* = "` 去 == 实测；
  scene() 构造器已存=166 调用，主体为测试逻辑非"直填"）→**批10 收敛**
  （scope=站点迁移量化口径，非行数迁移）。
- ⚠ **潜伏红披露**（复核 R3）：`python3 tools/field_ownership.py --check` 现 **[RED]**
  （表缺 `enemy_base`/表多余 `enemy_base_hp`+`enemy_base_gold`，exit=1——b89d0df
  P1-6② 切换未同步归属表；该工具不在 13 gate 内故闸面不显）→**批0 第一动作=归属表
  同步**（小笔补表=门禁工具数据表同步非功能变更，见批0 行）。
- **P2 盲区清单**（dump 盲区随域迁移登记，§5 轮2注②）：
  dialog（typewriterPos/dialogSpeaker/dialogAnchor/dialogTarget/overlay 布局面）、
  menu（menuCursor/itemCursor/recordPage/difficultyIndex）、
  persist（min/maxFloor/pendingFloor/wentDown）、tap（autoBattle）——
  各域迁移批**随批登记**（能进 dump 走 A2 行序冻结+金样重冻；不能进=夹具内断言兜底）。

---

## 2. 切片批次（每批=可独立复核；≤0.5d 原子步粒度内再分）

> 通则（每批全含）：**TDD 红先行**（trait/实现缺位编译红→落码绿，红绿输出随 message）；
> **贴件纪律**=判据输出以 `$(cat)` 终态原文入档（禁手编辑）；
> **等价三层**（域夹具断言+state_baseline --check 74 场景零重冻+对照表全锚 a.java）；
> **禁双轨口径**（一次性切：旧面删除与新面同 commit——313375b 先例；同 id 双源=违规）；
> **gates 13/13+ratchet 只升不降**；**回退纪律**=整步 commit revert 连新增测试一起回
> （禁只回实现留新测试=恒红）+基线 ratchet 同步回退+禁 fix-forward；
> **Ctx 三宪**（纯数据/字段冻结/ctx+≤2）+写面归因随批注记。

| 批 | 内容 | 判据要点 | 估 |
|---|---|---|---|
| 批0 | **地基**（第一动作=**field_ownership 归属表同步**：enemy_base 入表 combat 域+删 enemy_base_hp/enemy_base_gold 两条，187→186 新口径——门禁工具数据表同步非功能变更，`--check` 复绿即证）；subsystem 模块族立项（game-core/src/subsystem/）；每 trait 专属 Ctx 纯数据结构+字段冻结 schema 变更流程文档+写面归因登记（field-ownership 对照 state_dump FLD） | 编译红（trait 缺位）→骨架绿；归属表 `--check` 绿；零行为变更=金样 74 零重冻 | 0.5d |
| 批1 | **EntityService**（HostCtx 拆小首件）：实体面 9 方法→EntityService trait+impl；HostCtx 余 12 方法暂留（后续批拆） | 等价三层；HostCtx 方法数 21→12 显式账；spawn/remove 夹具 | 1-1.5d |
| 批2 | **BattleSubsystem**：resolve_bump trait 化（combat 函数族+scene 战斗路径编排面→BattleOutcome）；battle_enemy_hp 写面归因核 | 战斗夹具（preset 帧断言）+金样零重冻 | 1-1.5d |
| 批3 | **DialogSubsystem**：tick(ctx)->DialogStep（scene_dialog 已域化面 trait 化）；对话盲区清单登记（typewriterPos 等） | 打字机/翻页夹具（fx-use 系可复用）；盲区登记入档 | 1d |
| 批4 | **PersistCodec**：serialize/deserialize（Blob↔域切片**格式编解码面**；PersistState 行为面不入——spec ④）；persist 盲区登记（min/maxFloor 等） | 存读档往返夹具+save.rs 联测；金样零重冻 | 1-1.5d |
| 批5 | **ScriptHost**：execute_op(ctx,op) trait 化（script_ops 既有形+HostCtx 余方法处置账） | 47 场景 Java 三方（步末 18min 面）+金样 | 1.5-2d |
| 批6 | **ItemService**（§7-Q1 终案）：pickup/use 生命周期编排面；分发链唯一（Service→Registry::lookup→ItemEffect 序——现 execute_item_action 形保持）；禁第二分发链；menu 盲区登记 | pickup/use 生命周期夹具（P1 45 夹具面复用）；Registry 单测无副作用断言 | 1-1.5d |
| 批7 | **InputMapper**：RawInput→Action 抽象（controls+tap_walk 消费面）；tap 盲区登记（autoBattle） | 输入映射夹具（tap_two_stage 两段式等价）；金样零重冻 | 1d |
| 批8 | **UIRender**（只读边界件，不入互调通信面）：snapshot(&self)->UiFrame 纯读（E3 RenderState 消费面） | 纯读断言（无 &mut）；帧夹具（FRAME_OUT 对拍面） | 0.5-1d |
| 批9 | **ShopGoods（P2 尾件——spec:89"ShopGoods 批 P2 尾"）**：scene.rs:2652 `match (src, idx)` 商人商品属性束（价格/效果/提示逐臂）→ShopGoods per-type 记录表（**ItemRegistry 首个消费者**；照 items.json/enemies.json 先例：id 升序+五类拒载）+一次性切（match 臂删+表行同 commit） | 商人购买夹具（src0/1 双商）+金样零重冻；grep 双源=0 | 0.5d |
| 批10 | **收口**：测试构造面收敛（**scope=382 直写站点**——宽口径实测分布：t2_walkto 147/slot_equip 131/pointer_seam 42/entity_route 15/blank_tap 14/scene_test_support 8/item13_use_regression 8/touch_tap 5/fidelity_fix 5/itempanel_self_edge 4/walkto_interrupt 3；机械迁移（313375b 先例 427 站点单 commit 金样零漂移）→域构造器/断言收敛）；盲区清单完备核（statediff 判据=**命名契约层**：dump 名集⊂Java FLD 名集+A 侧需面过滤——全 trace 名集相等**永红**（A 侧 411 名 vs Rust 60 名，合成实测 A-only 名即红；`docs/findings/reviews/2026-11-03-p1a1-dump-face-review.md:135-138`））；HostCtx 方法数终值 machine 断言；直写清零收尾核 | 构造面等价（夹具字节等价）；名集对照（命名契约层口径）；全闸+ratchet | 1.5-2d |

合计 ≈**10.5-14d**（spec §5 估 8-15d 区间内；每批含验证前置件；含 ShopGoods 0.5d）。

---

## 3. 依赖与顺序

- 批0 为全前置（Ctx 族+模块骨架）。
- 批1 依赖批0；批5（ScriptHost）依赖批1（HostCtx 拆小后余面处置才清晰）；
  批6（ItemService）依赖批0（Registry P1 已立）；其余批 2/3/4/7/8 可与 1 并行候选
  （**亲做禁转包=顺序执行**，序=0→1→2→3→4→5→6→7→8→9(ShopGoods)→10 收口）。
- **禁止跨批混提**：每批 1..n 个原子 commit（≤0.5d 步），每 commit 逐提交对抗复核制
  （commit→报审→复核件→红修/绿推）。

## 4. 验收口径（P2 完成判据）

1. 八 trait 面全立（UIRender 边界件单独口径）+ShopGoods 入注册表+HostCtx 21 方法
   拆解账清零（或显式余留登记）+**批10 终值 machine 断言**
   （`awk '/^pub trait HostCtx/,/^}' crates/game-core/src/script.rs | grep -c "fn "`）；
2. **等价三层全绿**：域夹具+state_baseline --check 74 场景零重冻+Java 三方 47 场景（步末 18min 面）；
3. gates 13/13+ratchet 只升不降（起点=978）；
4. 迁移禁双轨全批核（grep 双源判据随批贴件）；
5. P2 盲区清单登记完备（dialog/menu/persist/tap 域）；
6. scene_tests 构造面收敛（**382 直写站点**→域构造器/断言收敛，grep 命令+当次输出随批贴件）；
7. Ctx 三宪+写面归因全批注记合规（field-ownership 对照）。

## 5. 风险与前车

- **HostCtx fat trait 前车**（21 方法上帝袋）：EntityService 等**拆小面**（spawn/remove/
  relocate 族）而非大一统 Service；每 trait 方法 ctx+≤2 违者拆方法。
- **直写换马甲风险**（DC-9 桥漏字段教训）：Ctx 写集清单与 state_dump FLD 对照登记随批。
- **双面前科**（gotcha_dual-porting-surfaces-drift）：禁 match 与表并行——切即删旧面。
- **验证预算**（R5）：每 commit 单测 ≈13s+state_baseline --check ≈4-5min；步末=Java 三方
  47 场景≈18min+gates all；禁只跑其一。

## 6. 修订记录

- 2026-11-07 初稿（v2-consumer-swap）：骨架+普查+十批切片；待对抗复核。
- 2026-11-07 复核修订（对抗复核 3 红 5 黄→全清；报告
  `docs/findings/reviews/2026-11-07-p2-plan-review.md`）：
  R1 ShopGoods 静默缩小→补批9（P2 尾件）；R2 "6098 直填"失实→6,127 行/382 直写站点
  量化口径（批10 basis 重写）；R3 field_ownership 潜伏红→§1 披露+批0 第一动作补表；
  黄：贴件 $(cat) 终态制入通则/回退补 ratchet 同步+禁 fix-forward/批10 statediff
  补命名契约层注记（全 trace 永红）/直写清零前置声明（已达成）/"3+8"改写带出处指针/
  验收1 HostCtx 终值断言/合计 10.5-14d。附注：复核件 R1 次引
  `2026-11-03-dddsa-attack-adaptation.md:89` 实测未命中（grep exit=1），主锚
  spec:89 实存——照实录入备查。
