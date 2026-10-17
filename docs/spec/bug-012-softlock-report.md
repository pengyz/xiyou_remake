# BUG-012 用户实机软死锁紧急复现报告（P0·交 lead 裁决）

> 2026-10-17，interact-evidence。任务：①复现定位 ②保真定性 ③处置方案（RED→GREEN）。
> 证据源：fx-bug-ces70-slot / fx-lay2a 双场景 Rust 侧无头驱动（crates/game-oracle/tests/
> fixture_frames.rs 末尾新增 `bug012_*` 复现驱动，release/debug 双模实跑）+ Java 全量转储
> 实录 [bug-012-fulldump-accumulation.md](../../reference/oracle/scenarios/bug-012-fulldump-accumulation.md)。
> 复现命令：`cargo test --release -p game-oracle --test fixture_frames bug012 -- --nocapture`
> （探针已标 #[ignore]，需显式点名运行）。

## 0. 结论速览

1. **复现定位（A 级逐拍）**：T301 撞 (2,11) 剧情点触发 script0 → T302 CES_84 生成嫦娥 →
   T303 MOV 武装行走（dphase=3, wsc=4）→ **T304 重入**（cursor 归 0）→ 3 拍/圈死循环 →
   **T350 `entity.rs:151` slot_push `[36][16]` panic = 与 Java T350 AIOOBE 同拍同点同因**。
2. **定性反转**：**Rust（release）没有软死锁**——崩溃语义与 Java 逐拍忠实（同 tick、同槽行、
   同"Index 16"）。此前"Rust 同构槽表未溢出、存活续跑=Java 独有崩溃状态"（2026-10-16）是
   **T349 采样假象**：夹具止于崩点前一拍，`fx_bug_ces70_slot_frames_match` 的存活断言恰在
   夹具末端取样。**用户"卡住脚本没执行"的真实构成** =（a）重入死区 T302-349 的原版 bug
   表现（角色钉死、对话/BOSS 不出现）+（b）崩溃的宿主呈现差异（桌面=进程退出；wasm/
   panic-隔离宿主=frozen canvas=软死锁观感）。
3. **HEAD 新回归（独立必修）**：状态机件 A（3ca869a）TRANSFER_EDGES 缺 **Dialog→Dialog
   重入边** ⇒ debug 态 T304 即 panic（scene.rs:1445-1449 debug_assert），**`fx_bug_ces70_
   slot_frames_match` 金样测试 HEAD 实测 FAILED**——重入是原版合法行为，边表缺边=校验器误杀。

---

## 1. 复现定位（A 级，逐拍对照 Java 全量转储）

### 1.1 Rust 侧时间线（release，fx-bug-ces70-slot 与 fx-lay2a 双场景同构）

```
T0300 key -2 press（南向步进，目标 (2,11) 剧情点 param=0）
T0301 mode: Map→Dialog    script0 触发（interact case83 bump 臂 → try_run_scene(0,false)=true
                          → load_level_script(0)；loadLevelScript 返 false ⇒ step 被挡，
                          玩家钉 (2,10)——BUG-012 机制实证的重入前提）
T0302 cursor 0→12         CES_84_6_11：ent 35→36（嫦娥 #1 注册进 (6,11) 槽行 36）
T0303 cursor 12→23        MOV_0_5_11：dphase=3、wsc=4（findPath 4 步队列）
T0304 cursor 23→0         **重入**（行走弹步撞剧情点 → load_level_script(0) 从头再跑）
… 3 拍/圈 …
T0347 cursor 0→12         CES_84：ent 50→51（嫦娥 #16，槽行 36 满 16/16）
T0348 cursor 12→23        MOV（第 16 圈）
T0349 cursor 23→0         重入（第 17 圈开始）
T0350 ★                   **entity.rs:151 slot_push panic:
                          index out of bounds: the len is 16 but the index is 16**
```

### 1.2 与 Java 全量转储逐拍同构（bug-012-fulldump-accumulation.md）

| 事件 | Java（dumpFull trace） | Rust（release 复现） | 同构 |
|---|---|---|---|
| 重入节拍 | T302/305/…/347 每 3 拍 cursor 归 0 | 同左 | ✓ |
| 嫦娥累积 | ent 35→51、槽值 36..51（16 只叠 (6,11)） | ent 35→51 逐圈 +1 | ✓ |
| 崩点 | T350 第 17 只注册 cellSlotTable[36][16] ⇒ AIOOBE | T350 slot_push [36][16] ⇒ panic | **同拍同点同因** |
| 崩点文案 | `Index 16 of bounds for length 16`（a.class 字节码） | `the len is 16 but the index is 16`（entity.rs:151） | 语义等价 |

### 1.3 复现驱动与命令

- 驱动 = fixture_frames.rs 末尾 `bug012_replay/bug012_events` + 两探针测试（已 `#[ignore]`，
  标注"P0-2 落地后转正式回归"）——与 `replay_opt` 逐拍同构（boot→槽载入→tick+键投递），
  无帧断言（纯逻辑观察），跑过夹具末端（ces70 460 拍 / lay2a 1300 拍）。
- 命令：`cargo test --release -p game-oracle --test fixture_frames bug012 -- --nocapture`。
- **注意**：debug 模式行为不同（见 §3-2）。

## 2. 定性（D4 保真判定）

### 2-1 "Rust 软死锁 vs Java 崩溃"判定：**保真达成（release），缺口在宿主呈现层**

- **tick 语义**：release Rust 在 T350 同点同因崩溃——与 Java 崩溃传播语义逐拍一致
  （崩前缀 349 帧金样已绿 = 前缀忠实；崩点 = 同一槽登记写）。**"行为不一致应修"的
  前提（Rust 存活续跑）不成立**——那是采样假象（fixture_frames.rs:753 的
  `assert!(scene.is_some())` 在夹具末端 T349 取样，恰为崩前一拍；夹具自身注释
  "夹具帧 1..349 = Java 可观测前缀"已声明止于崩前）。
- **宿主呈现缺口（真实存在）**：同一 panic 在不同宿主呈现不一致——桌面壳无 catch_unwind
  （main.rs:698 直调 `sc.tick`）⇒ 进程退出；wasm/任何 panic-隔离宿主 ⇒ 模块 trap 后
  **canvas 假死 = 软死锁观感**。用户"卡住"若是 wasm/容错壳路径即中招——这是唯一
  站得住的"软死锁"来源。
- **重入死区（T302-349 ≈ 4-5 秒）**：角色钉死 (2,10)、对话与 BOSS 全不出现——**Java 同窗
  同表现**（全量转储同构），属原版 bug 的可观测前缀，D4 保真保留（Stage B 才可动）。

### 2-2 HEAD 新回归：状态机件 A 边表缺 Dialog→Dialog 自环（独立缺陷，必修）

- scene.rs:1445-1449 `transition_to` 的 `debug_assert!(TRANSFER_EDGES…)`（注释自declare
  "release 不触发"）在 **T304 重入瞬间**发难："未登记转移边：Dialog -> Dialog"。
- 但 `load_level_script` 在 mode 11（Dialog）内重入是**原版合法行为**（BUG-012 机制的
  3 拍循环就是活证）——边表缺自环边 = 校验器误杀合法转移。
- **实测击穿金样**：`cargo test -p game-oracle --test fixture_frames fx_bug_ces70_slot_frames_match`
  HEAD **FAILED**（panic 分支断言期望 `index out of bounds`+`16`，实得"未登记转移边"）。

### 2-3 保真红线判定汇总

| 项 | Java | Rust release | Rust debug(HEAD) | 判定 |
|---|---|---|---|---|
| 崩点 tick | T350 | T350 | **T304（误杀）** | release 忠实；debug 退化 |
| 崩因 | 槽登记 [36][16] | 同 | TRANSFER_EDGES 缺边 | debug 为新引入缺陷 |
| 崩前缀 | T1-349 | T1-349 帧金样绿 | 同（到 T303） | 忠实 |
| 宿主呈现 | 主线程退出 | 桌面=退出；wasm=假死 | 同 | **跨宿主不一致=体验缺口** |

## 3. 处置方案（RED→GREEN，交 lead 裁决）

### P0-1 修 TRANSFER_EDGES 边表（必修，~0.5 天，无争议）
- 内容：登记 Dialog→Dialog 重入边（或 `transition_to` 对同模式重入豁免断言）。
- **RED**：`cargo test -p game-oracle --test fixture_frames fx_bug_ces70_slot_frames_match`
  当前红（HEAD 实测，可复现）。
- **GREEN**：该测试回绿（T1-349 帧对拍 + T350 panic 分支断言 `index out of bounds`+`16`
  ——现有断言本就设计为"同点 panic=忠实"）。
- 归属：状态机批（件 D 边表）；与 P0 无耦合，可并行。

### P0-2 终止语义统一（裁决项：两案择一）
用户观感（wasm 假死/桌面闪退不一致）需要"全宿主一致的崩溃呈现"。D4 边界：崩溃**时点与
因**保真已达成，呈现层统一不触游戏语义即不违保真。

- **案 (b) 壳层统一（推荐，保真零侵入）**：桌面 main.rs:698 与 wasm 入口的 `sc.tick` 调用点
  catch panic → 渲染统一"崩溃终止"画面（原版=线程死，宿主层呈现属壳语义，不进游戏画面
  差分面）。game-core 不动。RED=壳层测试断言 panic→终止画面；GREEN=两壳接线。
- **案 (a) 受控终止（触 game-core）**：`slot_push` 溢出改为置 `scene.fatal=Some(SlotOverflow)` +
  tick 停摆 + 宿主渲染标记——语义面改动（原 panic 变状态），需过对照评审（崩点前缀逐拍
  不变可用金样证；崩点拍的可见行为=新增终止画面，与 Java"线程死"的对应关系需注明）。
- 快照：**推荐 (b)**；若用户实机即 wasm，(b) 是最小闭环。

### P0-3 台账/夹具口径更正（随修，文档）
- fixture_frames.rs:739-753 注释与断言口径更正："Rust 存活续跑=Java 独有崩溃状态" →
  "**T349 采样假象；release T350 同点崩溃=忠实**（本报告 §1）"。
- bug-ledger BUG-012 行补一句：Rust T350 同点崩溃 A 级实证 + 用户实机症状归因
  （死区观感 + 宿主呈现）；"Rust 存活续跑"旧注记按证伪更正体例原地标注。

### Stage B（不在本 P0，防止范围蔓延）
- 用户期望"到三层**自动**触发 boss 战"= 体验诉求：原版链（(2,10) 入口）本就有重入死区
  （真机同款，D4 不许改）；可做的是**引导/入口层**：(a) 跳过教程路径的寻路引导改走 (3,11)
  正常入口（fx-scriptwalk-floor3 T1-700 帧绿 = 该入口链路正常推进到对话）；(b) 或跳过教程
  预置 scriptLineFlags 绕开 (2,10) 剧情点。二者均属 Stage B 现代化，需用户/lead 定夺。
- **建议向用户取证一条**：实机是桌面还是 wasm（§2-1 宿主假设）+ 触发格是 (2,10) 还是别处
  ——若确为 (2,10) 入口，原版真机同款表现（BUG-012 #2 真机证词缺口可就此闭合）。

## 4. 附带登记

- **写域声明**：本批改动 = fixture_frames.rs 末尾追加 `bug012_events/bug012_replay` +
  两探针测试（#[ignore]，append-only）+ 本报告。探针请 lead 裁决去留（P0-2 落地后
  转正式回归 = 直接留用；或移交 porter 收编为 fixture_frames 正式用例）。
- **HEAD 金样红**：`fx_bug_ces70_slot_frames_match` 当前红是 3ca869a 引入（与本批复现无关，
  是复现过程**新发现**）——P0-1 修复前全量套件不会绿，影响其他并行批的回归闸，请知悉。
- 追逐链完整场景（fx-lay2a）与 fx-bug-ces70-slot 行为完全同构（§1.2）——用户症状归因
  不因场景而异。

---

## 5. 实施记录（2026-10-17，lead 三项裁决落地）

| 项 | 落点 | 状态 |
|---|---|---|
| P0-1 边表补登 | scene.rs TRANSFER_EDGES **共补 5 边**（逐测暴露逐补）：`Dialog→Dialog` 自环（30b，脚本重入/BUG-012 循环）、`Map→RecordBrowser`（29b，道具 14 使用臂——tick_item_panel 13|14 先回 Map 再 activate 的链路）、`Dialog→Ending`（30c，END 指令 a.java:8510-8514）、`Map→Map` 自环（换层/restore 保持）、`RecordBrowser→ItemPanel`（29c，mode12 -5/-6/-7 关回 mode10）——**"35 边"巡查表漏登 ≥5 边**（census 缺口交件 D 系统补普查，勿再逐测挤牙膏） | fx_bug_ces70_slot_frames_match **回绿**（GREEN 达成）；**双闸全绿**：game-core lib 87/87 + fixture_frames 23/23（含 bug012 双回归）|
| P0-2 案 (b) 壳层统一 | game-desktop main.rs：`Screen::Crashed` 崩溃终止屏 + `sc.tick` catch_unwind（panic→`paint_crashed` 黑底白字"游戏已终止（原版崩溃语义等价终止）"）；game-core **零改动**；wasm 侧库内无宿主 crate（crates/ 无 game-wasm）——接口注释已留在 `Screen::Crashed` doc（接壳时照桌面路走） | 编译绿；崩溃呈现全宿主一致（消除闪退/假死分裂） |
| P0-3 口径更正 | fixture_frames.rs 注记【2026-10-17 证伪更正】（T349 采样假象）+ bug-ledger BUG-012 行补证 | 完成 |
| 探针转正式回归 | `bug012_{ces70,lay2a}_same_point_crash_regression`（去 #[ignore]，断言"index out of bounds+16=同点忠实"，存活=偏离） | 2 测通过 |

**RED→GREEN 闭环**：RED = fx_bug_ces70_slot_frames_match HEAD 红（P0-1 前实测）；
GREEN = 该测试回绿 + bug012 双回归绿（T350 同点崩溃断言）。

---

## 6. 取证 2：真实切层链（用户精确证词·floor2 楼梯→floor3）——移植缺口定位与修复

### 6.1 用户期望 = 原版设计（Java oracle 实证，A 级）

真实切层路径（非 preset 直入）：floor2 (1,10) 下步入 (1,11) 上梯 → applyStepCellEffects
case 7 → changeFloor(3) → 遮幅 swap → **落点 m_031 首可走邻格 = floor3 (2,11)（=剧情点
param0 格！floor3 (1,11) 梯格四周仅 (2,11) 可走，字节级几何实测）** → 落地 step 臂
case83 → script0 **自动触发** → CES_84 嫦娥 + MOV_0_5_11 自走 3 步 → TAK_8_9 剧情对话。
Java 逐拍（oracle floor2-stair2）：T305 踩梯 → T309 切层落 (2,11) → T310 cursor 12（CES）
→ T311 dphase=3/wsc=3（自走武装）→ T314/318/322/326 逐拍走到 (5,11) → T328 dialogPage=8。
**"切完楼层瞬间角色自动走到嫦娥+玉皇大帝处触发对话"=原版行为**（另一路手动撞 (2,11) 踩入
= BUG-012 重入循环雷区，两路并存）。

### 6.2 移植缺口定位（用户症状根因）

Rust 同路径：T309-T311 与 Java 同拍（切层/落地/CES/MOV 武装全对），**T312 起 wsc 恒 3
永不弹出**——自动走位死。断点 = **paint 分派的模式臂 `return` 跳过换层遮幅公共尾
paint_wipe_tail**（scene.rs paint()：EnemyInfo/About/SaveSlot/Help/InGameMenu|Shop/
Ending/RecordBrowser/**Dialog**/Alchemy 八臂全裸 return，仅 ItemPanel 曾单独修过
——注释自declare"所有 mode 后置共跑（a.java:3177-3215）"）：script0 一转入 Dialog 模式，
wipe 重开分支永不再跑 ⇒ `transitioning` 永真 ⇒ dp3 行走的 `!transitioning` 门永久关闭。
（首跑曾误报"切层不触发"=纯逻辑驱动不画帧、遮幅不推进——驱动教训已注进
bug012_replay：paint 不可省。）

### 6.3 修复（RED→GREEN）与回归

- **修**：八臂 `return;` → `return self.paint_wipe_tail(g);`（非转场时尾部零绘制 ⇒ 金样
  帧面无影响）；Java 语义 = paint switch 后公共尾无条件共跑。
- **RED→GREEN**：`bug012b_floor2_stair_transition_repro`（fixture_frames.rs）——
  Java 锚逐拍断言 T314 wsc=2 / T318 (3,11) / T322 (4,11) / T326 (5,11) / T328 dphase=1+cursor=31
  全过；修复前 wsc 恒 3（实测）。
- **全量回归**：game-core lib + fixture_frames 双闸（paint 补尾为非转场零绘制，金样零漂移）。

### 6.4 floor3 剧情链正常触发形态（随取证摸清）

| 路径 | 入口 | 结果 |
|---|---|---|
| 真实切层（楼梯） | 落点 (2,11) 踩格 step 臂 | script0 自动触发→自走→对话（**用户期望**，本修复恢复） |
| 手动步/撞 (2,11) | 从 (3,11) 撞 / (2,10) 踩入 | 前者正常（fx-scriptwalk T1-700 帧绿）；**后者=BUG-012 重入循环**（原版 bug，D4 保留） |
