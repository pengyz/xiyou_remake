# reference/oracle 差分报告（自动生成）

> 由 `python3 reference/oracle/run.py` 生成；**内容确定性**（无时间戳/耗时），复跑逐字节一致。trace 与截图在 `_out/`（gitignore），本报告只存结论与哈希。

## 1. 结论

| 判定项 | 结果 | 证据 |
|--------|------|------|
| 确定性：A1 vs A2 同输入 trace 逐字节一致 | **PASS** | trace sha256 `c71d1df83e84f53b…` vs `c71d1df83e84f53b…` |
| 语义等价：原始版(A) vs 无歧义版(B) 同输入 trace 逐字节一致 | **PASS** | trace sha256 `c71d1df83e84f53b…` vs `c71d1df83e84f53b…` |
| 源码投影等价：原始版(A) vs javac 源码投影(C) 同输入 trace 逐字节一致 | **PASS** | trace sha256 `c71d1df83e84f53b…` vs `c71d1df83e84f53b…` |
| stderr 对照（符号规范化后，辅助证据） | PASS（规范化后一致） | 见 §4 |
| T-变换 javap 归一化等价 | **PASS** | System/Thread→VTime/VThread 归一化后逐字节相同 |
| 运行成功（退出码 0 / tick 数 > 0） | **PASS** | A1 ticks=150, B1 ticks=150, C1 ticks=150 |

## 2. 对比对象

| 变体 | 来源 | a.class sha256 | CMidlet.class sha256 |
|------|------|----------------|----------------------|
| A 原始版 | `original/囧囧西游-大闹天宫.jar`（原样字节） | `128b7462031cb97dd7b9e873e2bc4f704da452928fa93382fad98d51d948d4fa` | `a3ce9ae3d24b0e091f915295ac4c8adfe5b94df25b05789e6e04ca113c0ae006` |
| B 无歧义版 | `analysis/rename-pipeline/renamed/`（t1 重映射产物） | `9fe6904ae55c1071191eb443547c41846133783dc581d46a530f4caca2ea95f3` | `730128b564f1d6d3a3d6a5bee29ab34f45f2e80966d5277ffaf237ae3d4792df` |
| C javac 源码投影 | `analysis/build/deobf`（deobf 源码 javac 编译产物） | `19a1b2973342216866098895c8b5eb63ee3991ddb6077b3f1816acc819047a23` | `023bf780e562d9fd841c228ae8da79a836f844d91f15d497676a33fef6f42772` |

三个变体施加**同一 T-变换**（常量池 Class 项重定向，见 §3）后运行。seed 版 `reference/seed/a.java` 是历史 CFR 投影（冻结件；存在重复成员名与 `this = v3` 等不可编译结构），按 AGENTS.md §1「字节码是最终真相」，运行时 A 变体取 JAR 内原始字节码。
C 变体是 deobf 源码（`reference/src/deobf/*.java`）经 javac 编译的 class：字节码由 javac 产出，版本/常量池布局/指令选择与原始字节码可能不同，T-变换只重定向 System/Thread 两个常量池 Class 项（§3），trace 等价性验证的是「状态向量/帧哈希/绘制操作流」语义层，不要求字节码本身相同。

## 3. T-变换（harness 适配，非游戏语义变更）

原版依赖墙钟与真实线程，直接跑 trace 不可复现。T-变换把 `java/lang/System`（currentTimeMillis/gc/out）与 `java/lang/Thread`（&lt;init&gt;/start/sleep/yield）的常量池 Class 项重定向到 `oracle/vt/VTime`、`oracle/vt/VThread`（虚拟时钟 + 协作调度），指令字节/LineNumberTable/异常表零改动（javap 归一化等价验证，归一化即把上述两组 owner 视为同一符号）。**全部变体施加同一变换**，故差分结论对变换不变。

| 变体 | 重定向引用计数 |
|------|----------------|
| A | java/lang/System→3, java/lang/Thread→6 |
| B | java/lang/System→3, java/lang/Thread→6 |
| C | java/lang/System→3, java/lang/Thread→6 |

## 4. 确定性策略（oracle policy，两端一致）

- **虚拟时钟**：`currentTimeMillis` 纯读；时间只在 `yield`(+1ms) 与 `sleep`(+ms) 推进。原版帧限速忙等（a.java:2424-2425）因此每帧确定性地空转 frameBudget 次后退出。
- **虚拟线程**：`Thread.start()` 仅登记，宿主在 `startApp` 返回后授权；同一时刻只有一个虚拟线程执行游戏代码，切换点在 start 授权/yield/sleep/线程结束（FIFO）。消除 JVM 调度不确定性。
- **输入投递**：固定脚本 `script-default.txt`，在 tick 边界（`serviceRepaints()`，a.java:2422）投递 keyPressed/keyReleased。
- **网络**：`Connector.open` 一律抛 IOException（确定性离线），驱动原版失败分支（a.java:8321 catch → -1）。
- **RMS**：内存实现（进程内静态 Map），跨运行为空。
- **音频**：Player/VolumeControl 状态机 stub，无声音输出，调用记入 EVT 流。
- **随机**：`java.util.Random` 保留原实现，seed 来自虚拟时钟 ⇒ 确定。
- **字形/字体**：shim 自定义度量 + 确定性示意字形（非设备字库）。文字布局证据看 trace 内 `OP drawString(...)` 操作流；像素哈希用于差分，不用于设备视觉保真（L4）。

## 5. trace 结构与统计

```
PRE ...
TICK 0001 vt=<虚拟毫秒>
FRAME sha=<像素sha256> w=240 h=320 png=frame-0001.png
  OPS count=<n>        # 绘制操作流（含 drawString 文本，≤256 条 + 总数）
  FLD <序号> <类型描述符> <值>   # 状态向量：class 文件声明序，不含符号名
INPUT press(-5)
OUT <System.out 行>
EVT <媒体/网络/线程事件>
END reason=<停止原因> ticks=<n>
```

- tick 数：A1=150 A2=150 B1=150 C1=150
- trace 行数：A1=72610 B1=72610 C1=72610
- 帧哈希链 sha256（全部 tick 的 FRAME 行摘要）：A1=`20bc7811cf9cb909…`，B1=`20bc7811cf9cb909…`，C1=`20bc7811cf9cb909…`
- 末 3 帧像素哈希（A1）：`b3b951c9fbedac6cb50851a7dbb7d2d1`, `8f26d6d792d3388397a8c769fb0ea754`, `a13e90baadd1e39a0d7b51cdc6d232dd`
- 末 3 帧像素哈希（B1）：`b3b951c9fbedac6cb50851a7dbb7d2d1`, `8f26d6d792d3388397a8c769fb0ea754`, `a13e90baadd1e39a0d7b51cdc6d232dd`
- 末 3 帧像素哈希（C1）：`b3b951c9fbedac6cb50851a7dbb7d2d1`, `8f26d6d792d3388397a8c769fb0ea754`, `a13e90baadd1e39a0d7b51cdc6d232dd`

## 6. stderr 对照（辅助）

异常堆栈含被重命名的成员符号；对照前经 `data/naming/remap-table.json` 规范化（`at <class>.<old>(` → 规范名），只处理堆栈行、不触碰游戏文本。

- A1 stderr 行数=0，B1 stderr 行数=0
- 规范化后比对：**PASS（规范化后一致）**

## 7. API 覆盖

见 [api-surface.md](api-surface.md)：MIDP/Nokia 引用 69 条全部由 `reference/shim/` 提供，缺失 0 条；JDK 引用 638 条存档。

## 8. 已知局限（后续阶段处理）

- 字形为示意图案、字体度量为 shim 常量 ⇒ 截图非设备像素级保真（Stage A 的 L4 视觉门禁在 P4 建立视觉比对器后另行标定）。
- 协作调度的线程交错点不同于真机并发语义；差分两端同一策略，未来 Rust 移植需复刻 oracle 策略（本文件是策略权威描述）。
- 网络永久离线、RMS 跨运行为空；联网/存档场景脚本待 P3 后补充。
- 本阶段差分只覆盖「机械重命名语义等价」；参考版 vs Rust 的 trace 差分（L1-L3）待 game-core/game-oracle 就绪后接入。

## 9. 变体 C（javac 源码投影）

第三个差分变体：**C = deobf 源码（`reference/src/deobf/a.java` + `CMidlet.java`）经 javac 编译的 class**，施加与 A/B 相同的 T-变换（§3）后运行同一输入脚本，产出同格式状态向量 trace。

与 A/B 不同之处：C 的 class 字节码由 javac（而非原始工具链）产出（源为 VF 投影+再编译）直接产出，版本号、常量池条目顺序、指令选择可能与 A/B 不同；但 §4 的 oracle 确定性策略（虚拟时钟/虚拟线程/tick 边界）与状态向量字段编号规则（class 文件声明序，不含符号名）与 A/B 完全同构 —— **只要 deobf 源码的字段声明顺序、方法行为与原始语义一致，trace 即应逐字节一致**。

| 判定项 | 结果 |
|--------|------|
| A vs C trace sha256 | **PASS** |
| B vs C trace sha256（辅助，B/C 均为 A 的派生表示） | **PASS** |

| 变体 | trace sha256（前 16 位） |
|------|---------------------------|
| A1 | `c71d1df83e84f53b…` |
| B1 | `c71d1df83e84f53b…` |
| C1 | `c71d1df83e84f53b…` |

C 变体源 class 目录：`analysis/build/deobf`。三方对拍命令：`python3 reference/oracle/run.py --diff-abc`（若省略 `--variant-c`，按约定路径 `analysis/build/deobf/` 自动发现；缺失则该命令以基础设施错误退出 2，不会误判为 FAIL）。

