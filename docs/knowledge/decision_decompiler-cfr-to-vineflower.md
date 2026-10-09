---
name: decompiler-cfr-to-vineflower
description: 反编译引擎从 CFR 0.152 切换到 Vineflower 1.12.0 的决策、实测数据与判定规则（iinc-vs-i2b 累加器定则）
type: decision
created: 2026-10-10
sources: ["2026-10-10 T-A 可编译实体执行（oracle 变体 C 三方对拍 PASS）"]
---

**决策：** 可编译参考实体的反编译引擎 = Vineflower 1.12.0（vendor `tools/vendor/`，
sha256 钉死于 `tools/vf_env.py`）；CFR 0.152 工具链退役移除（`reference/seed/a.java`
作为历史投影冻结件保留，D3 锚点不变）。

**实测数据（同输入 = 重映射后 class，同裁判 = shim javac，-Xmaxerrs 全量计数）：**

| 路线 | 编译错误 | 结局 |
|---|---|---|
| CFR 0.152 + GOTO 补丁 | 修 8 洞后暴露 **1615** | 不可行，放弃 |
| Vineflower 1.12.0 直出 | **110**（18 方法/86 行） | 人肉修复 → **0** |
| StackMap→LVT 全量注入 | **4826** | 净负收益，否决 |
| StackMap→LVT 仅数值槽 | **741** | 仍劣于基线，否决 |

修复后 javac 零错，oracle 三方对拍 A（原始字节码）==B（重映射）==C（javac 源码）
150 tick / 72610 行 trace 逐字节一致——**源码投影忠实性升级为机器证明**。

**为什么 VF 胜出：** ①GOTO 控制流全部原生还原（CFR 需手工补 8 洞）；②槽位类型
推断缺陷面 110 vs 1615（差一个数量级）；③直出 UTF-8 中文（CFR 需 deunicode 后处理）；
④输出确定性（双跑逐字节一致）。

**为什么 LVT 注入失败：** 帧级区间推导有盲区（astore 转性点无帧观察，实测把
StringBuffer 段误标 `[B`），而 VF 消费 LVT 的启发式会放大错误（slot0 条目造出
幻影变量 `v0.xxx`、多段名与参数名打架）。给它"部分正确的类型信息"比不给更糟。
CLDC StackMap 本身仍是**类型真相表**（查证槽位类型时秒查 javap -v 的 StackMap 帧）。

**iinc-vs-i2b 累加器判定规则（本次最有复用价值的知识）：** 反编译器把局部变量
声明为 byte/short、而源码对其做复合赋值（`+=`/`++`）时，javac 重编译会自动插
i2b 截断——若原字节码用的是 `iinc`（整型窄槽加法、无 i2b），则**原变量必是
int，必须提升声明**；若原字节码是 `iadd/iinc…+i2b` 模式则原变量真是 byte，保留。
实测案例：`m_013` 的 `var10 += 17` 第 4 次迭代 137 被截为 -119，oracle 变体 C
首跑 FAIL（clip x=138 vs -118，差恰 256）——**编译零错 ≠ 行为一致**，只有逐 tick
对拍能抓这类静默偏差。

**何时使用：** ①移植/重编译任何反编译产物前，扫描 byte/short 复合赋值位点并按
本规则核对字节码；②怀疑行为偏差时先查坐标/数值差是否恰为 ±256/±65536；③给
反编译器喂类型信息前先想 VF 的消费启发式（负收益教训）。
