# 考证：MOT_IF 截断/损坏读失败语义分歧（spec §9 假设 #4 → BUG-021）

> 考证人：adversary。委托：lead（platform-contract 移交，B 级疑点升 A）。
> 一轮时间盒；结论分"已证实/假设"（AGENTS §3）。
> **结论先行**：①游戏态分叉**已证实**（A 级逐行锚）；②附新发现——Rust 解析守卫差 1 字节
> 的 panic 窗（B 级静态，A 升格归修复任务）；③已登记 docs/bug-ledger.md **BUG-021**。

## 1. Java 真值（已证实，A 级逐行锚）

`loadSlotTable`（a.java:8959-9005）：

- **数组分配**（8961-8974）：仅 `!f_bool_15` 时新分配（首调=全零冷态）；
  `f_bool_15 = true` 置于 **8973（try 之前）**——失败后仍为 true，后续 markSlotEntry
  不再重载。
- **渐进解析**（8983-8997）：循环 6 槽写入共享数组——`slotValidFlags[v7] =
  readByte() != 0`（8984），有效槽 10 字段逐赋值（8986-8995）。Java 赋值 RHS 先求值：
  读到 EOFException 时**该字段保留旧值**，此前槽位/字段已写入 ⇒ 半解析态。
- **失败分支**（9000-9001）：`catch → markSlotEntry(-1)`；finally 关店（9002-9004）。
- MIDP 语义：ByteArrayInputStream 读越界抛 EOFException（JDK 规范），被
  `catch (Exception)` 收——解析中断即落此臂。

`markSlotEntry(-1)`（a.java:9007-9059）：

- 守卫 `if (!f_bool_15) loadSlotTable()`（9008-9010）——f_bool_15 已 true ⇒ 不重载；
- `var1 = -1` ⇒ 字段更新整段跳过（9012-9024），**不清数组**；
- 随后 `deleteRecordStoreQuietly("MOT_IF")`（9027）+ 序列化**当前数组**（9034-9050，
  含半解析残留）+ `addRecord`（9052）⇒ **半解析表被持久化**。

触发面：读档 UI 进入直调 `loadSlotTable()`（a.java:4353）；markSlotEntry 前置加载
（9008-9009）。

> **结论 1（已证实）**：截断 MOT_IF 时 Java 内存保留半解析槽表**并落盘**；仅
> "空店/首字节即失败"才等价冷态。platform-contract §9 #4 的读法成立。

## 2. Rust 现状（已证实，A 级逐行锚）

`menu_family::load_slot_table`（crates/game-core/src/menu_family.rs:694-742）：

- 695-700：None/空 ⇒ 冷态；713-737 渐进写 `self`；716-718/723-726 截断 ⇒ `ok=false;
  break`；**739-741 `*self = SlotSelect::new()` 全量归零**。
- 调用点：scene.rs:3485-3486（preset_mot_if 灌入）、paint.rs:644。
- 注释 :691-693"解析失败 → markSlotEntry(-1)（全无效冷态）"——**证伪**：仅首字节即
  失败时成立，半截解析路径 Java 保留半解析。【更正点，注释随修复任务原地更正】

> **结论 2（已证实）**：截断路径游戏态分叉——Java 半解析 vs Rust 冷态归零。

## 3. 新发现：Rust 守卫差 1 字节 ⇒ panic 窗（B 级，静态字节计数）

- 守卫 `pos + 2 + 28 > data.len()`（menu_family.rs:723）只保证 **30** 字节；
  实读 **31** 字节 = 3×u8（727-729）+ 7×i32（730-736）。
- `data.len() == pos+30` 时末个 `read_i32` 切片 `data[*p..*p+4]`（708-711）越界
  **panic**；Java 同窗口径是优雅 catch 降级，从不崩。
- 触发窗：某 valid 槽的 31 字节尾恰剩 30 字节（按槽位不同总长多值可达）。
- 崩溃位置：scene.rs:3485-3486 preset_mot_if 灌入在 shell `catch_unwind` 包裹面之外
  （shell/win/src/main.rs:722 只包 `sc.tick`）⇒ 该存档启动/交接即硬崩。
- 评级 **B**（静态字节计数，可复现推理）；**A 升格归修复任务**（截断回归夹具钉住）。

## 4. 可观测分叉面

| # | 面 | Java（半解析） | Rust（冷态/panic） | 证据 |
|---|---|---|---|---|
| 1 | 读档槽表 UI | 显示半解析槽（valid=true 半字段垃圾） | 全空 | a.java:9061+ paintSlotTable 直读数组 |
| 2 | 槽载入 | 可载半解析槽（损坏派生数值） | 无槽可载 | 槽字段即载入数据源 |
| 3 | 后续保存 | 半解析垃圾随整表持久（污染其余槽） | 持久零表 | a.java:9034-9050 vs scene.rs:3502 链 |
| 4 | panic 窗（§3） | 优雅降级 | 特定截断硬崩 | menu_family.rs:723 vs 727-736 |
| — | 一致面 | 空店/首字节失败 = 冷态 | 冷态 | 两侧无分叉 |

**可达性**：用户实机存档损坏/截断、外部工具改写、Android 应用数据部分清理——
**B 级推断**（BUG-018 前科=用户实机存档环出过真问题）；无真实损坏样本实测（假设区）。

## 5. 定性与处置（D4 口径）

- **原版 quirk** = 半解析保留 + 落盘——是原版行为，Stage A 必须复刻（AGENTS §1
  "行为只增不减"）⇒ Rust 冷态归零是**保真偏差**，修复任务 = 对齐半解析语义（含落盘）。
- **panic 窗** = 移植缺陷（原版无此行为）⇒ **Stage A 即须修**（守卫 ≥31，对齐 Java
  优雅降级——这是向 Java 靠拢的保真修复，不是"修 bug"），并补截断回归夹具。
- **Stage B 选项**：损坏读失败规范化为合法冷态（修原版 quirk）或维持原版语义——
  届时产品裁定。
- 已登记 docs/bug-ledger.md **BUG-021**（登记归本岗；修复另立任务）。

## 6. 假设区（不入结论）

- 损坏存档真实出现频率/形态（B 级推断，无实测样本）。
- "复刻半解析"的差分夹具口径（截断 preset 进 oracle 场景如何对拍）——修复任务设计时定。
- menu_family.rs:693 注释的原作者意图（C 级）：仅按代码证伪其表述，未考作者本意。

## 7. 实现口径裁定（2026-10-18，task-7 开工前；platform-contract 预警→adversary 裁定）

> 用途：复审的对账基准。三个放大件为裁定新增，复审逐条核。

- **①f_bool_15 数组复用层——确认成立**：Java case 7/8 入口只 loadSlotTable()+度量+
  光标滚动复位（a.java:4342-4367），数据数组仅 `!f_bool_15` 首分配（8961-8974）跨进入
  复用 ⇒ 半截旧值=上次表值。Rust 现状 scene.rs:3483/paint.rs:640 每次重建
  `SlotSelect::new()` = 旧值恒零。修复方向（去 valid 预清 + `reset_entry_layout()`）通过。
- **②None/空回写层——确认成立**：Java 无店/空记录同落 markSlotEntry(-1) 回写 MOT_IF
  （首进读档列表即写零表）；Rust parse_dirty 单面信号方案等价（含重复进入收敛）。
- **放大件 1（硬门）·字段级粒度**：守卫 30→≥31 仅可作安全不变量，**不许当整槽解析预检**。
  Java 失败粒度=字段级（RHS 先求值，readInt 1-3 字节 EOF ⇒ 整字段不写、此前字段已写）。
  实现须逐字段精确界检、在 Java 抛出的同一字节停；marker 网格夹具期望表必须按字段级
  语义逐长度导出。
- **放大件 2（硬门）·复位面全集 4 处**：scene.rs:3483、paint.rs:640（两入口改
  reset_entry_layout，**含 BootMachine 侧**）、menu_family.rs:698、:740（改不动数组+
  parse_dirty）。
- **放大件 3（登记即可）·跨阶段数组持久**：Java 数组跨 boot→game 交接存活（单 `a` 实例），
  Rust BootMachine::slots/GameScene::slot 交接丢弃——损坏 MOT_IF+先有旧表时尾槽跨交接
  复发本病（corner²）。两案：(a) 交接带 slot 数据（BgmState 先例）；(b) 登记已知边界 +
  BUG-021 补注。由执行方按成本选择并写明理由，复审核登记文本。
- 落盘对偶：SlotSelect.parse_dirty 单面信号 + shell 消费（避 gotcha_dual-porting-
  surfaces-drift）✓；动 shell/win/src/main.rs 前知会 shell-port ✓（task-7 备注合规）。
