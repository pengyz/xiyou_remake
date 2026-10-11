# 实现完整度测量方法论（v1）

> 动机（2026-10-12，用户提出）：对拍基建成熟后"新用例首跑即绿"会成为常态——
> **测试全绿只证明"声明过的问题没有回归"，不回答"还剩多少没实现"**。
> 完整度需要独立于测试的**分母**：参考系自身的全集。本方法论定义分母、
> 分子登记规则、防钝化约束与更新 ritual。

## 1. 原则

1. **分母独立于测试**：全集从参考系锚点（`reference/src/deobf/a.java`、
   `data/naming/remap-table.json`、资源清单）机械导出，不由"我们写了哪些测试"
   反推——否则测什么什么就完整，循环论证。
2. **覆盖只增不降**（与 test-ratchet 同构）：登记过的覆盖项不可悄悄撤销；
   撤销 = 回退，须在 commit 说明。
3. **分子必须带证据指针**：每个"已验证"登记绑定 scenario 名 + tick 窗口 +
   trace sha（或 golden 名），`python3 tools/completeness.py --check` 校验存在性。
4. **验证强度分级**，不同级不等价：
   | 级 | 含义 | 当前主载体 |
   |---|---|---|
   | L0 | 已移植，无对拍 | 代码 |
   | L1 | 资源/golden 级 | game-data L1 golden |
   | L2 | op 流级对拍 | trace OPS diff |
   | **L3** | **全帧 sha 逐 tick 对拍** | preset_cases / gameplay 用例（主线） |
   | L4 | 视觉级（像素 diff） | 等字体方案（R2）定案后启用 |

## 2. 分母（四个维度，均为参考系全集）

| 维 | 分母 | 导出来源 | 当前值 |
|---|---|---|---|
| A 符号面 | 类字段+方法全集 | `data/naming/remap-table.json`（机械） | **581** = 413 字段 + 168 方法 |
| B 模式面 | `paint()`/`run()` 模式 switch **非空顶层 case 臂** | deobf `switch (this.gameMode)` 花括号配平提取（机械，语义权威 = `docs/findings/state-machine.md`） | **paint 21 / run 16** |
| C 内容面 | DSL 指令码 / 楼层 / 资源 | `docs/spec/p3-script-interpreter.md`（15 码）、maplv 55 层、assets-manifest 138 条 | **15 / 55 / 138** |
| D 场景面 | scenarios 脚本（覆盖面声明 `# cover:`） | `reference/oracle/scenarios/*.txt` | 16 场景 |

机械提取命令：`python3 tools/completeness.py`（确定性输出）。
已知口径注意：paint 空臂组（如 case 23-98 fallthrough）与 run 空操作组
（15/17）合法为空不计入；case 4 与 19 共享渲染臂按语义表归属。

## 3. 分子登记（人维护，工具校验）

每迭代语义回流步骤更新本节矩阵；登记格式 = `mode/指令/层 × 等级 × 证据`。
证据示例：`scenario:`preset-save.txt` T340-700 L3`、`trace:`A-preset-save``。

### 3.1 模式面矩阵（B 维主表，2026-10-12 基线）

| mode | 语义（state-machine.md） | Rust 实现 | 验证级 | 证据 |
|---|---|---|---|---|
| 0 | boot 启动屏 | ✅ BootMachine | L3 | scenario:`boot-menu.txt` |
| 1 | 主菜单 | ✅ BootMachine | L3 | scenario:`boot-menu.txt` |
| 2 | 资源载入链 | ✅ BootMachine | L3 | scenario:`enter-game.txt` |
| 3 | 地图主玩法 | ✅ GameScene | L3 | scenario:`preset-floor2.txt` |
| 4 | 游戏内菜单 | ✅ | L3 | scenario:`preset-save.txt` |
| 5 | 敌人图鉴 | ✅ | L3 | scenario:`preset-item13.txt` T340-500 |
| （无 mode 6） | 战斗 = mode 3 子态（battleActiveFlag+tickBattle），非独立模式——2026-10-12 更正，矩阵初版误设 mode 6 行 | ✅ combat.rs | L3 | scenario:`floor1-to-floor2.txt`（4 战）+ `floor2-tour.txt` |
| 7 | 存档槽（写回） | ✅ | L3 | scenario:`preset-save.txt` T340-560 |
| 8 | 读档槽 | ✅ | L3 | scenario:`preset-help.txt` T150-240 |
| 9 | 炼丹/加点 | ❌ | — | |
| 10 | 道具面板 | ✅ | L3 | scenario:`preset-item13.txt` |
| 11 | 对话/剧情层 | ⚠️ 部分（序章 Java 侧验证） | L2* | trace:`A-prologue-dialog`（Rust 侧未建） |
| 12/13 | 过渡（B 级语义） | ❌ | — | |
| 14 | 新游戏引子插画 | ✅ BootMachine | L3 | scenario:`enter-game.txt` |
| 15 | 帮助浮层 | ✅ | L3 | scenario:`preset-help.txt` T450-700 |
| 16 | 设置 | ✅ | L3 | scenario:`preset-load-menu.txt` |
| 17 | 关于 | ✅ BootMachine | L3 | scenario:`menu-sweep.txt`（boot_replay 全量重放至 T1832） |
| 18 | 粒子特效层 | ❌ | — | |
| 19 | 商店 | ✅ enter/paint | L0（无任何场景覆盖——待迭代） | — |
| 20 | 结束/退出确认 | ⚠️ menu-sweep Java 侧 | L2* | trace:`A-menu-sweep` |
| 21 | 音效询问 | ✅ BootMachine | L3 | 全部 preset 场景 T66-70 |
| 22 | 退出淡出 70 帧 | ✅ BootMachine | L3 | scenario:`menu-sweep.txt` T1762-1832（boot_replay 终点断言） |
| 99 | paint 特殊屏 | ❌ | — | |

L2* = Java 三方对拍绿但 Rust 侧未建用例（覆盖声明的过渡态）。

### 3.2 内容面矩阵（C 维）

- DSL 15 码：TAK/CES/MOV/DES/GIN/LAY/ROS/RES/GUT/SEE/END/GLV/SMS/SWD/ADD。
  已在 Java 侧验证 trace 中出现（prologue/gameplay 系列）；**Rust 解释器覆盖
  以 game-core script 引擎对拍为准——逐码登记待迭代 29+ 细化**。
- 楼层：夹具已备 3/55（1/2/51），L3 对拍达成 2/55（floor1、floor2）+ floor51 直达
  场景三方 PASS、Rust L3 残差 = 对话层文本换行（迭代 30 修，BUG-008 另记框体观感）。
- DSL：ROS 六子码已移植（迭代 29，floor51 层脚本踩中 todo 后补——按层铺路线
  逼出真实缺口的第一个实例）。
- 资源：138 条 manifest 全部经 L1 golden（解析级）；渲染级随场景逐步覆盖。

### 3.3 场景面（D 维）

16 场景（tools/completeness.py 输出为准）。准入规则见 §4。

## 4. 防钝化规则（对"跑即绿"的三条约束）

1. **新用例准入声明**：每个新对拍用例入册时，必须在 commit 中声明它
   **新增的分母覆盖**（此前无该等级验证的 mode/指令/层/写路径）。
   不扩大覆盖的用例 = 回归网，允许存在但不计进度。
2. **覆盖 ratchet**：本文件矩阵中 L3+ 格子数只增不降（同 test-ratchet 精神）；
   发现假绿（如嵌套测试、缺 preset 头）修正后必须**重建对拍基准**而非
   保留绿状态。
3. **每报告给"下一最优目标"**：按分母大小 × 玩法权重选出未覆盖最大块。
   当前（2026-10-12 二次校准）：mode 11 对话层 Rust L3 化（现为 L2）>
   mode 9 炼丹链（Rust 未实现）> mode 19 商店场景（Rust 已实现零覆盖）>
   按层铺 51 深层（夹具已备）。矩阵初版的"mode 6 战斗 ❌"系误读
   （战斗非独立模式），方法论自身首日即捕获一处分母错误——分母提取
   同样要过证据关。

## 5. 更新 ritual

每迭代"语义回流"步骤固定动作：
1. `python3 tools/completeness.py`（分母可能因参考系重提取而变——理论上不变）；
2. 更新 §3 矩阵（新覆盖 + 证据指针）；
3. `python3 tools/completeness.py --check`（证据存在性）；
4. 本文件与代码同 commit 入库（完整性=文档+矩阵+证据一体）。
