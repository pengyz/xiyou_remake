# 对抗复核：refactor-plan.md（领域聚合 + 命令模式）——敌方意见

> 【2026-10-16】角色 = **敌方**：攻击 [refactor-plan.md](refactor-plan.md)（89 行）找漏洞。
> 格式：每攻击点「风险 / 反例 / 建议修正」三段；末尾总判定 + 条件清单。
> 证据口径：javap（renamed a.class）、deobf/seed 行号、工具源码逐行（statediff.py/
> state_dump.rs/fields.py/run.py）、gotcha_dual-porting-surfaces-drift.md、实体/脚本两侧
> Rust 源码。核过的面（含挑不出问题的）在 §8 全列。

---

## A1. 验证判据充分性：「252 测试 + 44/44 + 帧哈希」是三件武器两件半空枪

**风险**
- **44/44 场景 A==B==C 根本不跑 Rust**。run.py:276-281 定义：A=原始版字节码、B=无歧义版
  （rename-pipeline renamed class）、C=javac 源码投影——三方都是 **Java**，验证的是
  改名/投影管线保真，对 Vec<Entity>/ScriptOp 重构**零可见性**。把它列为第 0/1/2 步的
  主验证（方案 §1/§5"44/44 逐字节+基线 sha 对照"）= 验证武器与交付物不对题。
- 252 测试是真防线，但方案未列**行为面覆盖矩阵**：SoA→struct 的高危面里，
  sort_by_y 平局序（entity.rs:317 起的逐值冒泡交换序）、恢复臂 index 回填 quirk
  （a.java:9638 `entityVisible[var48]` 按循环下标而非槽号）、槽表 1-based 编码——
  这些是"测试绿但行为漂移"的候选盲区：帧哈希只对渲染序敏感，纯逻辑态（槽表内部序、
  removed 槽位留置形态）漂移可全绿。
- statediff（Rust↔A 真对拍）只比**名字交集**（tools/statediff.py:74
  `common = sorted(set(a) & set(r))`）——字段缺失静默缩小对拍面，工具自身不红。

**反例**
- 构造性反例：step 1 若把 sort_by_y 换成 `sort_unstable_by`（"顺手现代化"），平局
  y 值实体交换序变 → 部分场景帧哈希红，但**无夹具钉平局序的场景全绿**；再若恢复臂
  改成按实体对象迭代（把 `entityVisible[var48]` 的下标 quirk "修正"成槽号语义），
  fx-bug-ces70-slot 之外的恢复路径可全绿漂移——这正是 D4 保真红线要抓的行为差。
- 今天两次实证同类机制：BUG-002 复核（grep 漏检赋值 → 假死码结论"自洽成立"）、
  seed/deobf 行号 +6 漂移（指针陈旧无人报警）——"验证面没打到=结论照绿"有前科。

**建议修正**
- 主判据替换为三件：①**state_dump 金样等价**（重构前 dump_scene 输出冻结为金样，
  重构后 trace_snapshot 序列化对同一金样场景**逐字节**比）②statediff **全量断言**
  （unmatched 名集非空即红，交集比对升级为两侧名集相等）③行为面覆盖矩阵一节
  （迭代序/槽压缩/越界/恢复 quirk 各挂测试名，无测试的面补 RED 测再动布局）。
- 44/44 降级为附证（"Java 管线未动"），不进重构验证列。

## A2. TraceState 契约冻结：残表重造 + 双机制混谈 + 第 0 步自带字节序风险

**风险**
- **FLD 序号表是残表**：方案举"026/082/357/359/302…已实测序号表"（5 个示例）；
  全量映射**已存在**——`reference/oracle/fields.py`（413 行输出，FLD 000=f_Font_00 起，
  按 renamed class 字段声明序生成，带 --index/--timeline 查询）。方案不引它、另起
  "实测序号表"= 重复造轮子 + 双表漂移风险（第三面的表格版）。
- **双倾印机制混谈**：FLD 行 = `StateDump.java:59` **反射按 Java 类字段声明序**产出
  （Java 侧，Rust 重构不动，FLD 号天然稳定）；`state_dump.rs` = 按**语义名**导出
  **49 项** `name = value`（这才是吃 SoA 字段直读、受布局迁移影响的一面）。
  "TraceState=与 FLD 字段序一一对应"把契约套在了错误的坐标系上。
- **statediff 交集比对**（A1 已述）⇒ "契约冻结"没有完整性守卫：TraceState 漏 10 个
  字段，statediff 照样绿（common 变小）。
- **第 0 步本身可能改 dump 字节序**：现 dump_scene 的行序=函数内 push 序
  （frameCounter/gameMode/keyValue/…，state_dump.rs:71 起）；若 TraceState 按 FLD 序
  序列化，行序即变。statediff 名键比对无感，但任何按序消费者（金样文件、diff 文本、
  逐字节基线）当场红——lead 追问的"会不会本身就改变 dump 字节序"答案是**会，若不钉序**。

**反例**
- FLD 000 = `f_Font_00`——**对象型字段也在 413 槽向量里**；TraceState"契约结构"必须
  覆盖对象型的序列化形态（StateDump 的 toString/sha 投影），方案示例全是标量，
  0.5 天工作量按 5 个标量口径成立、按 413 槽含对象型口径不成立。
- fields.py 头注"映射只依赖 renamed class 的字段声明序（管线幂等 ⇒ 映射稳定）"——
  契约源应当是它，而不是"已实测"手抄表。

**建议修正**
- TraceState 定义改为：**生成式**取 fields.py 全量表（name↔FLD 双向），字段序显式
  = fields.py 序；序列化行序显式冻结（保留现 push 序或全局改 FLD 序，二选一并写死）。
- 验证补三条：旧 dump_scene vs 新序列化**金样逐字节等价**；statediff 升级名集相等
  断言；第 0 步 commit 自证（同场景重构前后 dump sha 相同）。
- 明确写入方案："FLD 行属 Java 侧、与 Rust 布局解耦"——第 0 步真实交付物只是
  state_dump.rs 的数据源适配层，工作量重估。

## A3. struct 迁移边界：槽表 1-based 编码 / append-only / 六写点双写无守卫

**风险**
- **槽表编码 = 1-based u8**（Java `cellSlotTable[..] = (byte)(var1 + 1)`，spawn
  a.java:6418、restore 9626-9640；Rust entity.rs:443 `(idx + 1) as u8`）——Vec 迁移
  若引入 swap_remove/紧凑化/重编号，槽项静默指错实体，且多数断言（帧级）不查槽表。
- **槽位永不复用**（lead 追问的"entityRemoved=true 的槽复用逻辑"实况）：Java spawn
  **append-only**（`entityCount++` 单调，a.java:6403-6410），removed 槽位留置到换层
  重建——方案"cap 100 同现状 entityCount 语义"**未写明三条禁令**（禁 swap_remove、
  禁重编号、清场仅 initCellGrid/m_122 重建路径）。
- **cellSlotTable 独立保留**⇒ 一致性靠 spawn/remove/remove_des/fix_slot/
  detach_from_cell/relocate_slot/m_108 **六写点**自觉双写，无收口不变量；struct
  迁移把实体侧改得面目全非时，六写点是最后的双写义务，方案只字未提。
- **恢复臂 index quirk**：`entityVisible[var48] = var36`（a.java:9638）按**循环下标**
  回填而非 spawn 槽号——struct 化按对象迭代即"修正"成另一语义（保真期=行为差）。

**反例**
- fx-bug-ces70-slot 夹具钉的正是槽位边界类崩溃（BUG-011/012 CES_70 槽越界同族）——
  该类语义在本仓已经炸过一次。
- m_108（relocate_slot，a.java:8886）：摘除=**移位压缩**（后续前移、count--），
  追加=1-based 写回——entity_golden.rs:209 有直测，但测试钉的是现行编码；换 Vec
  而不换测试语义时红得对，换编码顺手"现代化"则可能连测试一起改（审查必须盯测试 diff）。

**建议修正**
- 对照表补一行"槽编码 = (idx+1):u8，禁重编号/禁 swap_remove（append-only 到换层重建）"；
- 槽表读写收口单一 `grid::register/unregister/relocate`（六写点合一）+ debug_assert
  双向互查（槽项→实体存在、实体→槽项在册）；
- 恢复臂保留 index-based 回填并逐值注释（a.java:9638 quirk 原样）；
- 重构 commit 的验证矩阵点名 entity_golden/mov_route_scene/fx-bug-ces70-slot 三件必绿。

## A4. 命令模式游标坑：定位半过时 + 真坑是条件取参与停摆 + 第三面风险

**风险**
- **游标坑定位过时**：lastDelimPos（Java 34 处引用）在 Rust 侧**已纯化**——
  scene.rs:1554 `fn parse_script_int(line, from, delim) -> (i32, usize)` 返回结束位，
  不再有共享字段。方案把"游标语义=gotcha 级坑"挂在 lastDelimPos 上，打的是旧靶。
  **真坑 = 条件取参**：MOV 的 `var55>0` 决定是否继续取 3 参（a.java:8274-8287）、
  DES 负参分支（8314-8327 取参形态随符号变）、SEE 串取 5 参、GUT 尾递归
  （8437 executeScriptInstruction 自递归）、MOV-miss 递归（8302）——parse_op 若
  "静态 arity 一次取全参"，条件取参即错位。
- **UNKNOWN op 静默停摆**（R-TODO-1b）：else 链尾**不推进 scriptCursor**（游标停在
  标签处反复命中）——典型命令模式实现"解析失败仍消费 token"会直接改行为；enum
  必须有 `Unknown` 且 execute = 完全 no-op（含游标不动）并有 RED 测钉住。
- **第三面风险**：script.rs `ScriptEngine + HostCtx` 有**自己的 16 op 分派**
  （script.rs:333-441 GUT/DES/LAY/CES/GIN/RES…），仅 harness 驱动、生产不走
  （gotcha_dual-porting-surfaces-drift.md 明文"禁造第三面"）；`enum ScriptOp` 落
  新文件 script_ops.rs 而两面不同步 = 第三面实锤。且**步骤 2 的验证套件错位**：
  script_golden 测的是 harness 面（`set_script_line_for_test`），44/44 不跑 Rust
  （A1）——生产面拆分后真正对拍的只剩 fx 夹具一件。

**反例**
- DC-9 前科（gotcha 同文档）：combat.rs 桥公式漏道具二倍加成——两面各自绿、跨面
  漂移已发生过一次；ScriptOp 拆分是同机制的更大暴露面。
- MOV 条件取参反例：parse_op<Mov> 静态取 5 参 → 对 `MOV_0_5_11`（3 参玩家行）会多吃
  2 个 token，游标直接错位（fx-scriptwalk-floor3 红，但只有该夹具红——覆盖薄）。

**建议修正**
- parse_op 契约改为"按 op 结构**条件**取参，返回 (op, 停止位)"，逐值对照
  a.java:8355-8490；每个 op 的取参分支进表（含负参/条件 arity）；
- Unknown 臂 = 游标不动显式契约 + RED 测（R-TODO-1b 语义随迁）；
- script.rs harness 面同批改为**桥接** ScriptOp（禁第三面），或方案明确"本步不动
  script.rs，漂移风险挂账 + 出步骤后对拍义务"；
- 步骤 2 验证补：生产面 execute_script_instruction 等价替换前后 fx 夹具逐帧（主判据），
  script_golden 降为 harness 面附证。

## A5. 性能反例：论据方向反了 + 顺手换 sort 的帧序风险

**风险**
- 方案称"Rust struct 连续存储通常优于散列 SoA"——**方向反**：paint/碰撞这类
  少字段全扫描恰是 SoA 的缓存友好主场（拉 3-4 条数组的缓存线 vs 每实体拉整 struct
  +对齐 padding）；100 实体两者都进 L1，结论无害但论据错误，且与"非目标②性能优化"
  自相矛盾（一边声明不做一边下结论）。
- 热路径 paintEntityLayer 全实体扫描 + sort_by_y **逐值冒泡**（entity.rs:317 起）——
  平局序=渲染层叠序；struct 化的"顺手现代化"（unstable sort/iterator 化）会换序。

**反例**
- 构造：100 实体 × Entity{14 字段} ≈ 5-6KB/帧扫描（struct）vs 4-5 条数组线（SoA）；
  J2ME 逻辑帧预算下都不构成热点——但若某机型 L1 紧张，struct 反而是回退方向。
  （本条按"论据错误"计，不按"性能回退"计。）
- 平局序：现有帧哈希对拍含渲染序，但**平局 y 的交换序**只有在夹具恰好含平局实体时
  才被钉——未审计哪些场景含平局（覆盖矩阵无此项）。

**建议修正**
- 删"通常优于散列 SoA"论断，改为"100 实体量级两者均 L1 常驻，性能非本步判据；
  迁移前后各跑一次 paint 基准留档（100 实体帧时）"；
- sort_by_y 保持逐值冒泡语义（禁 unstable sort），平局序补一条单测。

## A6. 遗漏清单：方案没提的六件

**风险**
1. **script.rs/camera.rs 双面**（A4③ 详）——方案零提法。
2. **DC-9 桥处置**：combat.rs:107 `predict_hp_loss_vs_type`（☠️ 带"缺道具加成"缺陷
   注记，dead-code-rulings DC-9）——第 1 步"entity.rs 表操作合流"若波及 combat 调用面，
   缺陷注记随文件移动失锚；方案未列"不动 combat.rs"或"随迁保锚"。
3. **探针/工具直接读 SoA**：probe_floor2.rs:137-140 **测试内手算 cellTypeGrid ARR sha**
   （"sha computed in test"）直读 GameScene 字段——struct 迁移当天红（或被顺手改语义）；
   state_dump.rs（49 项）/statediff.py（交集比对）/fields.py（生成式映射）三件的
   迁移路径只提了 dump 一件。
4. **debug 面盘点缺失**：method_reach=Java JDWP 侧不受影响 ✓，但游戏内 debug/探针
   与工具面对 SoA 字段名的依赖未逐件盘点（§8 列了核到的，方案应固化成清单）。
5. **双解释器 harness 迁移策略**：l2_prologue/script_golden 走 ScriptEngine+HostCtx；
   步骤 2 拆生产面后 harness 面是桥接、同拆、还是冻结——方案空白。
6. **工作量低估**：第 0 步按"契约结构 5 标量"估 0.5 天，按全量 413 槽含对象型形态 +
   金样基线 + statediff 升级是另一量级；双面同拆 + 工具迁移后"3-4 天"整体偏乐观。

**反例**
- probe_floor2.rs 的手算 sha 与 state_dump.rs 头注"walkPathBuffer 已知不对齐项
  （differ 会标记，属预期）"——工具面已有**已知容差**语义，迁移时若顺手"对齐"即行为差。

**建议修正**
- 方案补两节：「工具/探针迁移清单」（逐文件：state_dump/statediff/fields/
  probe_floor2/…，含 walkPathBuffer 容差语义原样保留）与「双面处置」（script.rs/
  camera.rs：桥接或冻结，禁第三面）；
- DC-9 明确"第 1 步不动 combat.rs"或"随迁保锚+缺陷注记同步"；
- 工作量按双面+工具口径重估（建议 5-7 天）或砍范围（第 1 步只迁数据不动文件合流）。

---

## 7. 总判定

**有条件通过**——方向（领域聚合/命令模式/数据驱动特例）与非目标边界（零行为差、
不动类型级拆分）合理，但**验证武器与交付物错位**（A1/A2）、**槽表契约未写明**（A3）、
**双面/工具面遗漏**（A4/A6）四类缺陷任一落地都会造成"测试绿但行为漂移"或现场红。

**条件清单（全满足才可实施）**：
1. 主判据替换：state_dump 金样逐字节等价 + statediff 名集相等断言 + 行为面覆盖矩阵
   （A1）；44/44 降级附证。
2. TraceState 契约源 = fields.py 生成式全量表；dump 行序显式冻结并写入方案；
   对象型槽序列化形态明确（A2）。
3. 对照表补"槽编码 1-based u8 / append-only 禁重编号"一行；槽表六写点收口 +
   debug_assert；恢复臂 index quirk 逐值保留（A3）。
4. parse_op 条件取参契约 + Unknown 停摆游标不动 RED 测；script.rs 双面处置写明
   （桥接或冻结挂账）；步骤 2 验证主判据改 fx 夹具逐帧（A4）。
5. 删"通常优于 SoA"论断；sort_by_y 禁 unstable；paint 基准留档（A5）。
6. 补工具/探针迁移清单 + DC-9 处置 + 工作量重估（A6）。

## 8. 核过的面（含挑不出问题的，留痕）

| 面 | 结论 |
|---|---|
| 族表保留决策（enemyBaseHp/Atk… 不进 struct） | ✅ 合理——非实例数据 |
| 文件拆分按领域聚合（替代按函数类型 7 簇剪切） | ✅ 优于原案 |
| restore_rules 配置化（floor 2/12/39 特例外移） | ✅ 方向对；guard/action 表结构需含 floor2 补刷钩子的 flags[26] 双重角色（fixture-plan-boss §4.1 已有断言） |
| 每步独立 commit + 单 commit 回退 | ✅ 合理 |
| RED 测先行纪律（游标/指令） | ✅ 合理（但见 A4 的 RED 靶点修正） |
| 非目标④ execute_script_instruction 语义不动 | ✅ 与 A4 条件取参契约一致才成立 |
| cap 100 数量边界 | ✅ fx-bug-aioobe-border 已钉（但见 A3 append-only 未写明） |
| 第 0 步"oracle 侧 dump 逻辑不变" | ⚠️ 部分成立——StateDump.java 确实不变，但 state_dump.rs 必变，方案两面混称 |
| method_reach/java 字段序映射 | ✅ 不受 Rust 布局影响（fields.py 生成式） |
| 工作量 3-4 天 | ⚠️ 见 A6.6（低估） |
