# P3.1 脚本解释器移植——目标方案与验收合同（2026-10-10）

> 对应 master-plan §7「game-core 移植」的第一模块。TDD：本文的每个 golden vector
> 都锚定 Java 侧证据（deobf 行号 / oracle trace tick），测试先于实现（红→绿）。
> 独立复核由非实现者执行。

## 1. 数据域（全部从 deobf 源机械提取，夹具入库）

| 数据 | 来源（deobf a.java） | 夹具 |
|---|---|---|
| 层脚本表 `levelScriptLines`（68 项） | :1124 起字符串感知提取 | `crates/game-core/tests/fixtures/level_scripts.txt` |
| 对白文本 `dialogueTexts`（269 项） | 同法 | `.../dialogue_texts.txt` |
| 说话人类型 `dialogueSpeakerType`（269 B） | :1194 | `.../speaker_types.txt` |
| 对象类型名 `objectTypeNames`（88 项，含"菩提老祖"=87） | 同法 | `.../object_type_names.txt` |

关键锚点：第 **40** 项 = `"TAK_255_259 TAK_165_165 SEE_3_10_166_166_1 "` =
第 51 层（序章）脚本（prologue trace 实证 f_String_05）。

## 2. 解释器状态模型（Rust `ScriptState` ↔ Java 字段，命名对齐台账）

| Rust | Java 字段 | 证据 |
|---|---|---|
| `script_cursor` | `scriptCursor`（FLD245） | trace tick578=12/1002=24/1202=43 |
| `dialog_phase` | `dialogPhase`（FLD024→N-0104 dialogPhase） | 0 步进/1·6 打字机/3 行走/4 等相机/5 战斗 |
| `dialog_page` / `speaker` / `dialog_char_pos` | `f_int_112`/`f_byte_17`/`f_int_33` | openDialogPage（原名 m_096，台账 N-0090）a.java:8185-8210 |
| `overlay_text` | `overlayText`（N-0114） | trace FLD062 六句序章对白实证 |
| `tak_page_start/end` | `f_int_120/121` | a.java:8227-8231 |
| `scene_delay` | `f_int_118`（CES 派生延迟） | a.java:8379-8384 |
| `game_mode`/`walk_phase`/`script_walk_armed`/`walk_request_flag` | 已命名（台账） | — |
| `player stats`（hp/atk/def/三钥匙/金/物品栈/炼丹/层里程碑） | RES_ 重置集 | a.java:8466-8479 |

## 3. 指令语义合同（16 opcode，deobf executeScriptInstruction a.java:8216-8564）

解析规则：3 字符 opcode；`parseScriptInt`（a.java:8560-8567）= 从 pos 起找分隔符
（`_` 或空格），`f_int_122` = 分隔符位，cursor = `f_int_122 + 1`；行尾门
`cursor + 3 <= len` 否则 no-op（序章 cursor=43 冻结态实证）。

| opcode | 语义（行号锚点） | P3.1 覆盖 |
|---|---|---|
| TAK_a_b | 页区间开对白：phase=1，openDialogPage(a) | **全**（含 m_096） |
| GUT_n | flags[idx]=true → loadLevelScript(n)（:8269-8273 + :7997-8012） | **全** |
| GLV_n / SMS_n | 解析后仅 cursor 推进（空指令，:8456/:8486） | **全** |
| LAY_n | m_119(层重载 hook) + layer=n + **f_bool_16=true**（:8317）+ phase=5 + overlayText=null | **全**（m_119 走 hook） |
| ROS_sub | 1=玩家坐标/3=HP/4=朝向/5=ATK/6=DEF 直写，2=m_082 hook | **全**（2 走 hook） |
| CES_a_b_c | m_048 生成实体 + 相机锚 + scene_delay（**按类目表 f_byte_arr_03[type]**：类目 1/8/16→5，2/4→8，默认 0；类目表初始化 a.java:6204-6252——2026-10-10 复核 R3 更正，原误写为按裸 type） | 实体操作走 hook |
| MOV_(3|5 参) | 5 参 y 也用 `_` 分隔；**cursor 在分支前推进**（:8245）；实体命中→findPath→phase=2（无条件），未命中尾递归；玩家 findPath 成功→**script_walk_armed=true + phase=3 + walkPhase=0 + m_104(0)**（:8265-8267，三写点缺一不可） | 路径走 hook |
| DES_t_x_y | m_100 查得→m_047 移除（hook）；t<0 按类型批量 | 移除走 hook |
| GIN_k_v | 0=pickupItem/1=gainGold（hook）+ **尾递归继续下一条**（唯一显式递归，:8395-8397） | **全** |
| ADD_floor_t_x_y | 跨层生成（m_119/m_122/m_048 hooks） | hooks |
| SWD | 类型 81→4 改写（五字段写，hook） | hooks |
| SEE_a_b_c_d_e | 先 openDialogPage(c)，cursor 推进，phase=6 + m_103 hook；e=1 时 findPath 门控 walkPhase=0+f_bool_26 | **全**（路径走 hook） |
| END_0 | gameMode=20 + m_000（exit hook） | **全** |
| MVS | 数据域 68 条脚本 **零使用** → P3.2 边界（panic 指引 deobf a.java:8490-8564） | 显式未实现 |

### 3.1 行尾收尾门（2026-10-10 复核 R4 补，a.java:8551-8563）

方法尾部（dispatch 之外）恒执行：`if cursor < line.len() { return; }` 否则
**dialogPhase=4 + scriptLineFlags[current]=true（index≠32 时）+ m_104(0)**。
两个触发面：①dispatch 消费完行（cursor>=len）；②入口 cursor+3>len（越过行尾）。
序章冻结态（cursor=43、phase=4、SEE 已置 6 被覆写）即由此门产生——BUG-006 的
phase 4 与教学页 166（"好了，现在请试着按方向键移动"）均已入 L2 断言。

## 4. L2 对拍设计（TDD 核心）

**向量来源分层**（防循环论证）：
- **L2a 指令级**：单 opcode → 状态迁移，期望值 = deobf 语义人工推导 + 夹具数据
  （如 TAK_255_259 → cursor=12, page=255, speaker=87, overlay="菩提老祖: \cF8F8F8…"）
- **L2b 时间线级**：floor1-序章真实轨迹（oracle trace 实测，prologue-dialog 场景）：
  `cursor 0→12(t578)→24(t1002)→43(t1202)`，对白文本序列 6 句（255..259 + 165），
  引擎按「step + advance_dialog_page ×按键」重放，逐点断言
- **BUG-006 复现向量**：SEE_3_10_166_166_1 在 find_path=false 注入下 cursor 仍推进
  43、walk 不武装——死锁语义的回归锚

**game-oracle crate**：oracle trace 解析器（TICK/INPUT/FLD 行）+ 从夹具 trace 提取
`(tick, script_cursor, overlay)` 时间线与 Rust 引擎重放比对；`cargo test -p game-oracle`
= `diff-trace` 门禁（SKIP→PASS）。

## 5. 红线与边界

- game-core 无 I/O、无平台依赖（夹具 include_str! 编译期嵌入，仅测试）
- 实体/相机/地图副作用全部经 `HostCtx` trait 边界（P3.2 实体模块落地真实实现；
  测试用确定性 stub）——脚本级状态迁移（cursor/phase/页/文本/旗标）本期全量保真
- MVS 为显式 P3.2 边界（数据零使用，panic 指引证据行号）
- game-platform 只放 trait + 确定性类型（VirtualClock=75ms/tick 对齐 oracle VTime）


---

## 复核记录（2026-10-10，独立复核者非实现者）

裁决 fail → 修复 → 复验。Findings 全收口：
- R1(high) MOV 5 参分支：y 分隔符错误/cursor 不推进/终点用错 → 重写（a.java:8233-8268 忠实序）
- R2(high) MOV 玩家分支缺 walk-arm 三写点 → 补齐 + 测试断言
- R3(high) CES 延迟查类目表而非裸 type → spec 与实现更正 + 类目表夹具（a.java:6204-6252）
- R4(high) 行尾门非 no-op（phase=4+旗标+相机）→ 新增 §3.1 + 双触发面实现 + BUG-006 冻结态断言
- R5(high) char_pos 按 UTF-8 字节 → chars().count()（UTF-16 单位）
- R6/R7(medium) RES 补 f_int_89=0；LAY 补 f_bool_16
- R8(medium) HostCtx 文档与签名一致化 + entity_type 方法
- R9(medium) ROS_1 像素直写落地
- R10(medium) 证据指针校准（m_096→openDialogPage；行号以当前文件复核为准）
- R11/R12(low) 断言去同表互证（speaker=87 字面量）；TDD 红绿历史随本 commit 入库
