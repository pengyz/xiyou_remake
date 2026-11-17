# 对抗复核：DDDSA P2 八 trait 子系统面实施计划（草案）

> 复核日期：2026-11-07。复核员：对抗复核员（只读；唯一写=本报告）。
> 被审件：`docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md` @ d034126（117 行）。
> 上游依据：`docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md` v3（§3 八 trait 面/§3.2 Ctx 宪法二执行则/§3.3 构造面/§5 估算 8-15d 与前置/§7-Q1/Q4/Q5）。
> 纪律：AGENTS.md（证据 A/B 级、证伪更正、门禁表）。本报告全部实测均给命令与输出原文。

## 总判：**需修订**（3 红项 + 5 黄项）

| 面 | 结论 |
|---|---|
| 1. spec 忠实度 | 🟡 黄（八 trait 面/三宪/禁双轨/回退均照搬；**ShopGoods 批 P2 尾静默缩小=红 R1**；statediff 判据口径缺注记） |
| 2. HostCtx 拆解账 | 🟢 绿（21 方法实测全真，五类归属互斥完备，算术与批1 账全对） |
| 3. 批序合理性 | 🟢 绿（无前向依赖；判据基本可机器执行；2 条注记） |
| 4. 估算贴合 | 🟡 黄（10-13.5d ∈ spec 8-15d ✓；**"6,098 行（2026-11-07 实测）"与实测 6127 不符且"直填"定性失实=红 R2**；批9 估 basis 未量化） |
| 5. 判据与四纪律 | 🟡 黄（逐提交/禁双轨/整步回退三落实；**贴件纪律 $(cat) 终态制完全缺失**；回退缺基线同步细则） |
| 6. 起点基线 | 🟡 黄（978/74/13 三数全真且声明勿动 ✓；**依赖件 `field_ownership.py --check` 处于 RED 未披露=红 R3**） |
| 7. 交叉攻击 | 🟡 黄（所引命令/文件/符号逐个实存，无 P3 混入；1 处无源断言"3 协调器+8 域文件"；批9 名集判据口径缺口） |

---

## 面1 spec 忠实度 — 🟡 黄（含红 R1）

### ✅ 八 trait 面全数有归属批（spec §3.1 ↔ 计划十批逐一映射）

| spec §3.1 八件 | 计划归属批 | 实测核 |
|---|---|---|
| BattleSubsystem | 批2 | combat.rs 198 行 8 fn（to_records/trait_flags_for/find_item/scale_enemy_stats/effective_attack_vs_type/predict_battle_hp_loss/consume_key_for_door/gain_gold），计划点名 4 个全存在 ✅ |
| DialogSubsystem | 批3 | scene_dialog.rs 263 行，`struct DialogState`@5、`advance_typewriter`@49 ✅ |
| PersistCodec | 批4 | scene_persist.rs 585 行 9 fn + save.rs 338 行 15 fn ✅ |
| ScriptHost | 批5 | script_ops.rs `enum ScriptOp`@5、`parse_op`@392、`execute_op`@544 ✅ |
| ItemService | 批6 | effect_registry.rs `pub trait ItemEffect`@169、`ITEM_EFFECTS`@32、item_effect.rs `enum ItemAction`@8 ✅ |
| InputMapper | 批7 | controls.rs `pub enum Action`@44 ✅（spec L98"独立 trait 不进 BattleSubsystem"未回退） |
| EntityService | 批1 | entity.rs 621 行 28 fn + sprite_spawn.rs 55 行 + HostCtx 实体面 9 方法 ✅ |
| UIRender | 批8 | render.rs 1493 行 97 fn + scene_paint.rs `RenderState` 六字段（strip_frame/backdrop_scroll/bob_offset/bob_rising/open_anim_tick/tile_bob）已切 ✅ |

### ✅ C 三宪 / 执行则4-5 / 禁双轨 / 整步回退 照搬

- 计划:68 `**Ctx 三宪**（纯数据/字段冻结/ctx+≤2）+写面归因随批注记` ↔ spec §3.2 第 1-3 条 + 第 5 条（写面归因）✅
- 执行则4（too-many-args 8 处冻结不追溯）→ 计划:17 `不做…存量静态函数签名追溯改写（too-many-args 8 处冻结）` ✅；8 处清单指针实存于 `docs/superpowers/plans/2026-11-02-scene-module-decomposition-plan.md:10`（grep `load_floor_from_save` 命中该文件）✅
- 禁双轨 → 计划:66 `一次性切：旧面删除与新面同 commit——313375b 先例；同 id 双源=违规`；计划:101 验收4 ✅。313375b 实存（`第 1 步 SoA→struct 全量迁移（Entity 11 字段+427 站点，金样零漂移）`）✅；`docs/knowledge/gotcha_dual-porting-surfaces-drift.md` 存在 ✅
- 整步回退 → 计划:67 `回退纪律=整步 commit revert 连新增测试一起回（禁只回实现留新测试=恒红）` ✅（spec §5 "步 revert 连新增测试一起回"已带；**基线 ratchet 同步回退/禁 fix-forward 两半句未逐字带入=黄-轻**）
- R 系列未回退：R8 8-15d ✅（计划:83）、R7 构造面保护 ✅（批9）、R9 EnemyRegistry 后置 ✅（计划:16 不做）、R6 8 处并存 ✅

### 🔴 R1 — ShopGoods「批 P2 尾」被静默缩小（spec 忠实度硬伤）

```
$ grep -n "ShopGoods" docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md
89:- P1 **仅 ItemRegistry**；EnemyRegistry 后置（quirk 路由/域外三界复杂面）；ShopGoods 批 P2 尾；
$ grep -rn "ShopGoods" docs/ crates/ | wc -l   # 仅 spec + 两份 review 的文字，无代码、计划十批无归属
$ grep -n "ShopGoods" docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md
（零命中）
```
- spec §2.3 排定"ShopGoods 批 P2 尾"，`docs/findings/reviews/2026-11-03-dddsa-attack-adaptation.md:89` 亦裁定"ShopGoods 作为 ItemRegistry 第一个消费者并入 P2 尾或 P3 前（半天级）"。
- 计划十批无 ShopGoods 批，§0"不做"清单也未列 → **既不做也不声明 = 范围静默缩小**，违反"不得回退/缩小 spec 已裁决项"。
- 修订：补批9 前半件（半天级）或在 §0 显式声明"ShopGoods → P3 前"并给 spec 依据。

### 🟡 statediff 判据口径不完整（批9）

计划:81 `盲区清单完备核（statediff 名集相等=A 侧 409 名守卫）` 照抄 spec 措辞，但缺已知使用注记（`docs/findings/reviews/2026-11-03-p1a1-dump-face-review.md:135-138`）：

> statediff 全 trace 口径注记（bc4411e 起既有）：A 侧 oracle trace 每 dump 块 411 名全字段 vs Rust dump 60 名——**名集相等守卫在真实全 trace 对上永不可能过**（合成实测 A-only 名即红）；P1-a1 判据「statediff 名集相等」实义=命名契约层（dump 名集⊂Java FLD 名集，守卫在场），非全 trace 实跑绿。工具面口径欠一句「A 侧需面过滤」的使用注记。

实测佐证：`tools/statediff.py` L88-98 逐 tick `names_a != names_r` 即 `[RED]`；`reference/oracle/fields.py | wc -l` = 413 行（映射条目 ≈409，"A 侧 409 名"口径成立）；Rust `state_dump.rs` dump 面字符串字面量唯一 62 个。→ 批9 判据须补"A 侧需面过滤（命名契约层）"注记，否则执行者会在全 trace 上撞永红。

### 🟡 spec §5 前置件"直写清零在跑"未声明（实际已满足）

spec:155 `| P2 … | **直写清零在跑=前置（C）** |`；计划未提。实测：
```
$ python3 tools/field_ownership.py --report | tail -2
[报告·全口径] 多文件直写字段=44（含 shell/oracle 收件面）
[报告·定案口径] 生产面 self 直写跨文件字段=0（**终章清零判据**）
```
→ 前置实际已达成，计划只欠一句声明（黄-轻，非回退）。

---

## 面2 HostCtx 拆解账 — 🟢 绿（claims 病惯犯面全真）

```
$ awk '/^pub trait HostCtx/,/^}/' crates/game-core/src/script.rs | grep -c "fn "
21
$ grep -n "pub trait HostCtx" crates/game-core/src/script.rs
128:pub trait HostCtx {
$ sed -n '125,168p' crates/game-core/src/script.rs | grep -c "fn "
21
```
- 21 个 fn 落在 129-166 行，名单逐个核：find_path / cell_entity / entity_type / remove_entity / remove_all_of_type / swd_convert_type81 / entity_pixel / give_gold / pickup_item / snap_camera_to_player / locate_speaker_entity / set_typewriter / reload_floor_entities / spawn_entity / ces_camera / set_camera_anchor_target / exit_application / mvs_exchange / m_082_set_dialog_target / set_entity_route_flag / snap_camera_after_ros1。
- 五类归属互斥且并集=21：实体10 + 道具经济2 + 相机4 + 对话2 + 系统3 = **21** ✅（与计划:49 算术逐字相符）
- 批1 判据"HostCtx 方法数 21→12"= 21−9（实体 10 中 find_path 留 pathfind 域，实迁 9）✅ 算术成立。
- 锚点注记（无害）：计划写 `script.rs:125-168`，spec 写 `128-168`；实测 trait 起点=128，125-127 为 doc 注释行，区间内 fn 数仍=21，**计数与归属不受影响**（spec:121 已把"23→实测 21"修正在案，计划未回退 ✅）。
- 系统面 3 中 `lockCameraOn` 调用实存（`crates/game-core/src/camera.rs:1,6,90`）✅；`trait ProcessControl` 实存（`crates/game-platform/src/lib.rs:153`）✅。

---

## 面3 批序合理性 — 🟢 绿（2 注记）

- 依赖自洽，**无批依赖 M>N 的面**：批0 全前置；批1←批0；批5←批1（计划:90 显式）；批6←批0（计划:91）；其余 2/3/4/7/8 显式标"可并行候选"但亲做禁转包=顺序 1→9（计划:91-92）。实际耦合（批2 Battle、批6 Item 需 EntityService 的 ctx/夹具）由顺序执行覆盖，不构成前向依赖。
- 判据可机器执行：TDD 编译红（trait 缺位）→骨架绿（批0）；`state_baseline --check` 实存（`tools/state_baseline.py:101` `check = "--check" in sys.argv`，line6 文档 `python3 tools/state_baseline.py --check`）；金样 74（`data/status/baselines/state-dump-golden.json` scenarios=74）；gates check 13 项实列；47 场景（`ls reference/oracle/scenarios/*.txt | wc -l`=47）；`tap_two_stage`（scene.rs:4787/scene_test_support.rs:124）、`FRAME_OUT`（fixture_frames.rs:310-313）、fx-use 夹具（game-oracle/tests/fixtures/fx-use-85.txt 等）、"P1 45 夹具面"（fixture_frames test=45）全数实存 → 各批判据锚可复现。
- 注记①：验收1"HostCtx 21 方法账清零（或显式余留登记）"的**最终余留 8 方法**（find_path 1+相机 4+系统 3）只有计划 §1.2 的文字归属，批9 未设 machine 复核点（批5"HostCtx 余方法处置账"在序上先于批6，会先陈旧一次）。建议批9 补一条 `awk … | grep -c "fn"` 终值断言。
- 注记②：P3 编辑器（tools/editor）未混入 ✅（计划:16 不做清单），无范围膨胀。

---

## 面4 估算贴合 — 🟡 黄（含红 R2）

### ✅ 总估在 spec 区间内
逐批求和：0.5+1~1.5+1~1.5+1+1~1.5+1.5~2+1~1.5+1+0.5~1+1.5~2 = **10.0~13.5d**；计划:83 称"≈10-13d"（上界少记 0.5d，无害）；spec:155 `8-15d（A 修）` ✅ 区间内。

### 🔴 R2 — "6,098 行（2026-11-07 实测）"为假 + "直填"定性失实

```
$ wc -l crates/game-core/src/scene_tests/*.rs | tail -1
6127 crates/game-core/src/scene_tests/*.rs   # 12 文件，HEAD=d034126
（逐 commit 复算 git ls-tree+show）
21848e4 11-02 17:13 → 6098 … ba72769 11-02 23:24 → 6098
752e985 11-03 16:26 → 6102
cb710f4 11-03 18:45 → 6127   （此后至 d034126 恒 6127）
```
- 6098 只在 11-02 17:13–23:24 的提交上成立；计划 §1 标题"**现状普查（只读实测 2026-11-07）**"、计划:53 与计划:81 却写"6,098 行 / 6098 行"——**当日实测值是 6127**。该数系从 spec §3.3（11-03 轮2抽验"恰 6098"）陈旧继承，却被标为 11-07 实测 → 事实性错误（A 级可证伪）。
- 定性失实：宽口径直写站点实测 **382 处**（`grep -rEn "\b[a-z_][a-z_0-9]*\.[a-z_][a-z_0-9]* = "` 去 `==`，分布 t2_walkto_route_tests 147 / slot_equip_scene_tests 131 / pointer_seam 42 / entity_route 15 / blank_tap 14 / scene_test_support 8 / item13_use_regression 8 / touch_tap 5 / fidelity_fix 5 / itempanel_self_edge 4 / walkto_interrupt 3）；窄口径（`sc.`/`scene.`）154 处；`SceneState{`/`GameScene{` 字面量 5 处；**`scene()` 构造器已存在**（scene_test_support.rs:152 `pub fn scene(player: (i32,i32), ents: &[(u8,i32,i32)]) -> GameScene`）且已有 166 处调用 → 6127 行的主体是**测试逻辑与断言**，不是"直填构造"。
- 1.5-2d 判断：若 scope=382 个直写站点（机械迁移，313375b 先例单 commit 427 站点金样零漂移）→ 1.5-2d **够用**；若按字面"6098 行→域构造器"→ **严重低估**。计划未量化迁点基数 → 估 basis 不可复核。
- 修订：改数为 6127（或注明继承值+抽验日期），把批9 scope 明确为"N 个直写站点（附 grep 命令与当次输出）"。

---

## 面5 判据与四纪律 — 🟡 黄

| 纪律 | 计划落实 | 实测 |
|---|---|---|
| 逐提交（复核先于推送） | 计划:93-94 `每 commit 逐提交对抗复核制（commit→报审→复核件→红修/绿推）` | ✅ 落实 |
| 贴件纪律（$(cat) 终态） | 计划:101 `grep 双源判据随批贴件`、计划:64 `红绿输出随 message` | 🔻 **无 $(cat) 终态/未手编辑贴件制** —— 判据的"贴件"仅指 grep 结果，未要求以 `$(cat)` 终态原文入档（黄，需补一句） |
| 禁双轨 | 计划:66 + 计划:101 验收4 | ✅ 落实（313375b 先例、同 id 双源=违规） |
| 回退整步 | 计划:67-68 `整步 commit revert 连新增测试一起回（禁只回实现留新测试=恒红）` | ✅ 落实；**缺"基线 ratchet 同步回退 + 禁 fix-forward"半句**（spec §5，黄-轻） |

每批判据的机器可执行性：批0 编译红/金样 74 零重冻、批1 `grep -c "fn "` 账、批2 战斗夹具+金样、批3/4/7 夹具+盲区入档、批5 47 场景+金样、批6 Registry 单测无副作用、批8 纯读无 `&mut`、批9 全闸+ratchet —— 均可执行；唯一口径缺口=批9 statediff（见面1）。

---

## 面6 起点基线 — 🟡 黄（三数全真；依赖件潜伏红=R3）

### ✅ 三数全真且声明勿动
```
$ python3 -c "import json;d=json.load(open('data/status/baselines/rust-test-counts.json'));…sum…"  → 978
$ grep -c scenarios data/status/baselines/state-dump-golden.json（scenarios=74）→ 74
$ python3 gates/cli.py cmd_check  → 恰 13 项（original/assets_manifest/reference_seed_integrity/knowledge_format/docs_links/naming_ledger/rust_tests/test_ratchet/table_content_hash/table_evidence/state_machine_anchors/asset_manifest/state_baseline）
$ python3 gates/cli.py status     → PASS=13 FAIL=0 SKIP=1(visual-diff)；test-ratchet total=978（基线 978）
```
计划:5 `P1 已推基线（ratchet 978/金样 74/闸 13/13）为起点勿动` ✅ 与实测逐字相符。

### 🔴 R3 — 起点基线存在未披露的潜伏红：`field_ownership.py --check`
```
$ python3 tools/field_ownership.py --check; echo $?
[RED] 归属表：
  表缺字段 1：['enemy_base']
  表多余字段 2：['enemy_base_hp', 'enemy_base_gold']
1
```
- 根因：`crates/game-core/src/scene.rs:307 pub enemy_base: Vec<crate::combat::EnemyDef>`（b89d0df「P1-6② SoA→EnemyDef 一次性切」改字段未同步归属表；表最后更新 ba72769 早于该切）。表内仍留 `enemy_base_hp/enemy_base_gold/battle_enemy_hp` 三条 combat 域记录（`python3 -c` 逐条打印属实）。
- 该工具**不在 13 gate 内**（故 `gates status` 照绿），但它是计划:72 批0"写面归因登记（field-ownership 对照 state_dump FLD）"与批9"直写清零收尾核"的**依赖件**——按现计划动工首日即撞红，而 §1"现状普查（只读实测 2026-11-07）"未披露。
- 修订：批0 前置加半步"归属表同步 enemy_base（含 commit 说明）"，或在 §1 补披露该 RED 及处置归属。

---

## 面7 交叉攻击 — 🟡 黄

### ✅ 所引命令/文件/符号逐个实存（无死引用）
`state_baseline --check`、`gates check/all`、`field_ownership --check/--report`、`tap_two_stage`、`FRAME_OUT`、fx-use 夹具、313375b、`gotcha_dual-porting-surfaces-drift.md`、DC-9（plan:110）、`trait ProcessControl`（game-platform:153）、too-many-args 8 处清单（2026-11-02 plan:10 起）——全数命中。BattleOutcome/DialogStep/UiFrame/Blob 现不存在=按 spec §3.1 待新建类型，非缺陷（`enums.rs:11 ScriptStep` 已存在）。

### 🟡 无源断言
计划:50 `scene.rs 编排面（3 协调器+8 域文件编排注记归因——task-15 落件）`：全 docs grep 仅命中本计划自身与 `2026-11-02-scene-module-decomposition-plan.md:42 GameScene（协调器：状态机+域装配）`；"3 协调器+8 域文件"的出处未找到（field-ownership.json 实测为 187 字段/11 归属域，menu-ui42/dialog-vm22/tap-input21/entity19/render18/coordinator15/item-effect14/persist13/combat12/script6/env-infra5——与"3+8"无对应关系）。→ 需补证据指针或改写为可复现口径（B 级断言缺锚）。

### ✅ 自相矛盾 / 范围膨胀检查
- 批1"余 12 方法"算术正确（21−9）；§1.2"实体 10 含 find_path"与批1"实迁 9"不矛盾（find_path 单列留域）。
- 无 P3 编辑器工作混入（§0 不做清单）；无 EnemyRegistry 新建（R9 未回退）；无行为改动（§6-1 红线声明）。
- 唯一判据口径缺口=批9 statediff（面1 已列）。

---

## 红项清单（需修订）

1. **R1（面1）**：ShopGoods「批 P2 尾」（spec:89 §2.3 + attack-adaptation.md:89）既无归属批也未进"不做"清单 → 补半天级批或显式声明去向。
2. **R2（面4）**：计划 §1/§1.2/批9 的"6,098 行（2026-11-07 实测）"实测为 **6127**（6098 仅存在于 11-02 17:13–23:24 提交）；且"直填"实为 382 直写站点（主体为测试逻辑）→ 改数、补 grep 命令与当次输出、量化批9 scope。
3. **R3（面6）**：`python3 tools/field_ownership.py --check` 当前 **RED**（表缺 `enemy_base`、表多余 `enemy_base_hp/enemy_base_gold`，exit=1），是批0/判据依赖件却未在"现状普查"披露 → 批0 前置补表同步半步或显式披露处置。

## 黄项清单（建议修订）

- 贴件纪律：补"判据输出以 `$(cat)` 终态原文入档（未手编辑）"到通则（面5）。
- 回退纪律：补"基线 ratchet 同步回退 + 禁 fix-forward"（面5）。
- 批9 statediff：补"A 侧需面过滤（命名契约层），全 trace 名集相等永红"使用注记（面1）。
- spec §5 前置"直写清零在跑"补声明（实测已满足：生产面跨文件直写=0）（面1）。
- §1.2"3 协调器+8 域文件"补证据指针（面7）。
- 验收1 补批9 的 HostCtx 方法数终值 machine 断言（面3 注记①）。
- 计划:83 合计上界实为 13.5d（写 ≈10-13，无害，建议改 10-13.5）（面4）。

## Confidence: 84/100
