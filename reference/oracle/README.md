# reference/oracle — 差分 trace 宿主（确定性运行 + 逐 tick 状态向量）

以**固定输入脚本 + 固定虚拟时钟 + 协作调度**驱动参考版（`a.class`/`CMidlet.class`
字节码）逐 tick 输出状态向量 trace 与截图 PNG，支撑差分验证。
当前首个差分用例：**原始版 vs t1 无歧义版**（机械重命名语义等价的运行时证明），
使 `gates/cli.py reference` 门禁从 SKIP 变为实际可跑。

## 用法

```bash
python3 reference/oracle/run.py             # 完整差分（gates reference 门禁入口）
python3 reference/oracle/run.py --selftest  # 快速确定性自测（A 跑两次逐字节比对）
python3 reference/oracle/run.py --ticks N   # 指定 tick 预算（默认 150）
python3 reference/oracle/run.py --selftest --ticks N --script <file>  # 自定义输入脚本
```

退出码：0 全部判定通过；1 任一判定失败；2 基础设施错误（编译/运行失败）。

判定项（全部通过才退出 0）：

1. **确定性**：A 变体同输入跑两次，`trace.txt` 逐字节一致；
2. **语义等价**：原始版(A) vs 无歧义版(B) 同输入 `trace.txt` 逐字节一致；
3. **T-变换等价**：harness 适配变换前后 javap 归一化逐字节相同（只动引用 owner）；
4. **stderr 规范化对照**：异常堆栈经 `data/naming/remap-table.json` 符号规范化后一致；
5. **运行健康**：退出码 0、tick 数 > 0、API 覆盖缺失 0。

## 组成

| 文件 | 职责 |
|------|------|
| [`run.py`](run.py) | 编排：构建 → 变体准备 → T-变换 → 运行 → 比对 → 报告 |
| [`ttransform.py`](ttransform.py) | T-变换：常量池 Class 项重定向（System→VTime、Thread→VThread），带白名单安全校验 |
| [`api_surface.py`](api_surface.py) | 外部 API 面 vs shim 覆盖核对（缺失即失败） |
| [`script-default.txt`](script-default.txt) | 默认输入脚本（tick 键码 press/release） |
| `src/oracle/vt/` | 虚拟时钟 VTime、协作调度 VScheduler、虚拟线程 VThread、StopSignal |
| `src/oracle/host/` | Runner（Java 入口）、TickHooks（tick 边界钩子）、StateDump（状态向量）、PngWriter、TraceSink |
| `_out/` | 运行产物（trace.txt、frame-*.png、run-info.txt、编译输出；**gitignore 不入库**） |
| [`_diff/`](_diff/) | 差分报告 [`report.md`](_diff/report.md) + API 覆盖 [`api-surface.md`](_diff/api-surface.md)（入库） |

## 确定性设计（策略权威描述，Rust 移植需复刻）

- **虚拟时钟**（T-变换）：`System.currentTimeMillis` 纯读；时间只在
  `Thread.yield`(+1ms)/`sleep`(+ms) 推进。原版帧限速忙等（a.java:2424-2425）
  因此每帧确定性地空转 frameBudget 次后退出；`Random.setSeed(currentTimeMillis)`
  （a.java:623）随之确定。
- **虚拟线程**（T-变换）：`Thread.start()` 仅登记，宿主在 `startApp` 返回后授权；
  同一时刻只有一个虚拟线程执行游戏代码，切换点在 start 授权/yield/sleep/线程结束（FIFO）。
- **tick 边界**：原版主循环每帧 `repaint(); serviceRepaints();`（a.java:2421-2422），
  shim 在 `serviceRepaints()` 内 ① paint 到离屏画布并采帧哈希 ② 投递脚本输入
  （keyPressed/keyReleased，同一虚拟线程）③ 记录状态向量（反射 dump，
  **只含值不含符号名**，字段按 class 文件声明序编号）。
- **trace 格式**：一行一记录（TICK/FRAME/OPS/FLD/INPUT/OUT/EVT/END），
  不含变体标识与符号名 ⇒ 两变体可直接逐字节比对。
- **T-变换等价性论证**：A/B 施加**同一**变换（只重定向 `java/lang/System`、
  `java/lang/Thread` 两个常量池 Class 项 → `oracle/vt/VTime`/`VThread`；
  指令字节、LineNumberTable、异常表零改动，javap 归一化验证），故
  「T(A) ≡ T(B)」的 trace 等价性对变换不变；与 t1 的 javap 指令级等价验证互补。
- **网络**：`Connector.open` 一律抛 IOException（确定性离线）；
  **RMS**：内存实现；**音频**：状态机 stub。

## 场景覆盖（默认脚本）

boot 资源加载 → 场景 21「是否开启声音？」（a.java:756-762 绘制、1536-1552 输入：
GAME_A(-6)=开声 / GAME_B(-7)=静音）→ 场景 1 登录/菜单（导航 -1/-3/-2/-4、确认 -5）→
UI 动画循环。更深场景（移动/战斗）的输入脚本是后续工作：用 `--script` 传入即可，
脚本格式见 [`script-default.txt`](script-default.txt)。

## 复现报告

`_diff/report.md` 内容确定性（无时间戳/耗时），`run.py` 复跑逐字节一致：
`python3 reference/oracle/run.py && git diff --stat -- reference/oracle/_diff`。
