# 重构方案：领域聚合 + 命令模式（Stage B 架构基）

> 【2026-10-16】用户裁定采纳 OOP 聚合/命令模式解耦诉求（对抗复核通过后实施）。
> 前置文档：architecture-review.md（文件级拆分评估）、gotcha_dual-porting-surfaces-drift.md。

## 0. 目标与非目标

**目标**：①扩展性（新实体/道具/指令/特例不再捅 god-class scene.rs 7703 行）②消除误读温床
（大 switch fallthrough 语义——case12 误读早触发 7 拍、重复臂事故均此类）③可测试性（对象/指令
级单测）④特例 data-driven 化（floor 2/12/39 恢复特例散落）。

**非目标**：①行为变更（**零行为差**，252 测试+44/44 场景+帧哈希兜底）②性能优化 ③类型级拆分
（trace 契约红线，architecture-review 已否决）④execute_script_instruction 语义改动（16 op 逐值
对照体，只改组织不改语义）⑤Java OOP 照搬（按 Rust 姿态落地）。

## 1. 第 0 步【前置】：trace 映射层

**问题**：oracle 差分验证 FLD 状态倾印=SoA 字段直读（entityType[]/entityPixelX[]…逐字段 dump，
StateDump.java）；数据布局一变对拍基础设施失效。

**方案**：game-core 提供 `fn trace_snapshot(&self) -> TraceState` 适配层——TraceState=与
FLD 字段序一一对应的**契约结构**（FLD 026=keyValue/082=walkPhase/357=battleEnemyHp/…已实测
序号表）；oracle 侧 dump 逻辑不变（读 TraceState 序列化）。重构期间 TraceState 契约冻结。

**验证**：重建映射后 44/44 场景 A==B==C **逐字节不变**（基线重跑对照 sha）。

## 2. 第 1 步【低险高益】：数据聚合 Entity struct + 拆文件合流

**SoA→struct 对照表**（javap 字段清单锚定）：

| 现 SoA 数组 | Entity 字段 | 备注 |
|---|---|---|
| entityType[] | type_ | i32 |
| entityPixelX/Y[] | px/py | |
| entityState[] | state | |
| entityParam[] | param | |
| entityVisible[] | visible | |
| entityRemoved[] | removed | |
| animRow/animIdx… | 同名 | |
| cellSlotTable/Count | **保留独立**（空间索引，不动） | |
| enemyBaseHp/Atk…表 | **保留独立**（族表非实体实例数据） | |

**布局**：`Vec<Entity>`（cap 100 同现状 entityCount 语义）；渲染/碰撞热路径逐帧扫描=同序遍历
（cache 影响待 benchmark，Rust struct 连续存储通常优于散列 SoA）。

**拆文件合流**（替代 architecture-review 的 7 簇按函数类型剪切——按领域聚合）：
- `entity.rs`：Entity struct+表操作（spawnEntity/removeGroup/m_119/m_122/reload…）
- `script_ops.rs`：第 2 步落点（先建空壳）
- `restore_rules.rs`：特例批落点（先建空壳）
- `scene.rs`：状态机保留（tick/paint/dispatch）

**对照性保全**：每 struct 字段注释锚 a.java 字段名+行号；重构 commit 内附 SoA→struct 对照表。

**验证**：全闸+TraceState 逐字节+252 测试。

## 3. 第 2 步【痛点直击】：命令模式 ScriptOp

`execute_script_instruction`（425 行 switch，16 op）→：
```rust
enum ScriptOp { Tak(..), Gut(..), Mov(..), Lay(..), Res(..), Gin(..), Des(..),
                See(..), Sms(..), Mvs(..), Add(..), Ces(..), Ros(..), … }
fn parse_op(line) -> ScriptOp;
fn execute_op(scene, op);  // 每 op 独立函数，注释锚 a.java:8355-8490 逐值
```
**不变量**：parse/execute 拆分但**游标推进语义逐字节等价**（scriptCursor/lastDelimPos 语义
= gotcha 级坑，RED 测先行）。

**验证**：script_golden 全测+fx 夹具+44/44+UNKNOWN op 静默停摆语义（R-TODO-1b）保持。

## 4. 特例批【并行】：restore_rules 配置化

floor 2（spawn84 钩子）/12/39 恢复特例 → `RestoreRule { floor, guard, action }` 表（scene.rs
reload_floor_entities 内嵌逻辑外移）。验证：floor2_restore_respawns 测试保持+新增表驱动单测。

## 5. 执行序与验证矩阵

| 步 | 验证 | 回退 |
|---|---|---|
| 0 trace 映射 | 44/44 逐字节+基线 sha 对照 | 单 commit revert |
| 1 数据聚合+拆文件 | 全闸+TraceState+252 测试+帧哈希 | 同上 |
| 2 命令模式 | script_golden+fx 夹具+44/44 | 同上 |
| 特例批 | floor2 测试+表驱动单测 | 同上 |

**红线**：每步独立 commit 全闸；对照性映射表随步维护；行为差=立即停下定位（§4 三问）。

## 6. 工作量估计

第 0 步 0.5 天（契约结构+dumper 适配+基线重跑）；第 1 步 1-1.5 天（struct 迁移+全量替换+
拆文件）；第 2 步 1 天（16 op 拆分+游标语义 RED 测先行）；特例批 0.5 天。**合计 3-4 天**。

---

## 7. 【2026-10-16 对抗复核吸收】六条件（refactor-plan-review.md 9cdc7bc，判定=有条件通过）

1. **主判据换枪**：44/44 A==B==C 系纯 Java 三方（run.py:276-281 不跑 Rust）⇒ **主判据 =
   ① state_dump 金样逐字节等价 ② statediff 名集**相等**断言（非交集——statediff.py:74 `set(a)&set(r)`
   交集口径=漏字段静默绿）③ 行为面覆盖矩阵（槽复用/游标/恢复臂 quirk 逐项有测）**；44/44 降级附证。
2. **契约源=fields.py 413 槽生成式映射**（非手抄残表；FLD 000=f_Font_00 对象型槽在内）。
   **行序冻结声明**：state_dump.rs dump_scene 现行 push 序（frameCounter/gameMode/…）**保持不变**，
   TraceState 适配其序而非 FLD 序（FLD 行=Java 反射序、布局迁移天然不影响——两机制分列不混谈）。
3. **槽表三禁令**（Entity 迁移红线）：① 槽编码 1-based u8（entity.rs:443 `(idx+1)` 语义保持）
   ② **槽位永不复用**（entityCount 单调 append-only 至换层重建——swap_remove/重编号=静默坏槽）
   ③ 六写点（spawn/remove/fix_slot/detach/relocate/m_108）迁移清单化+一致性收口（双写点同一
   收口函数）。恢复臂 `entityVisible[var48]` 按循环下标回填 quirk（a.java:9638）**显式保留**（struct
   化禁顺手修正）。
4. **游标契约**：条件取参清单（MOV var55>0 续取 3 参/DES 负参分支/GUT 尾递归 8437/MOV-miss
   递归 8302）RED 测先行；**UNKNOWN 停摆=游标不动**（命令模式禁"解析失败仍消费 token"，R-TODO-1b
   钉住）；双面分列验证——script_golden 测 harness 面（script.rs ScriptEngine）、生产面
   execute_script_instruction 用 fx 夹具+44/44 状态面验证（DC-9 桥漏道具加成=双面漂移前科）。
5. **删性能论断**：SoA vs struct cache 方向实测后定（SoA 少字段扫描是主场，sort_by_y 平局序
   禁 unstable sort）。
6. **遗漏六件补全**：script.rs/camera.rs 双面策略（迁移时 harness 面同步或显式冻结）、DC-9 桥
   处置（随特例批修忠实或删）、测试内 SoA 直读点清单（probe_floor2 手算 cellTypeGrid sha 等）、
   工具三件（fields.py/statediff/cover_matrix）迁移路径、双解释器收敛决策（另立任务）、
   **工作量修正 5-7 天**。

### 实施前置（第 0 步扩展）：金样基线先行
重构前先固化 state_dump 金样基线（当前 HEAD 全场景 dump sha 清单）——第 1/2 步逐字节对照。
