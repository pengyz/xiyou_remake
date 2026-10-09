# reference/shim — 自研 MIDP-1.0 / CLDC + Nokia UI API 子集（Java 实现）

参考版运行时的**平台 shim**：只实现参考版字节码实际引用的 API 面，
供 [`reference/oracle`](../oracle/) 差分 trace 宿主驱动参考版运行。

**红线**：本目录只新增实现，不触碰 `original/`、`reference/seed/`（AGENTS.md §2）；
shim 语义偏差只允许**确定性、两端一致**的简化，且必须在
[oracle 差分报告 §4](../oracle/_diff/report.md) 的「确定性策略」登记。

## 覆盖的 API 面（自动核对）

[`reference/oracle/api_surface.py`](../oracle/api_surface.py) 扫描 `a.class` +
`CMidlet.class` 常量池全部 Fieldref/Methodref/InterfaceMethodref，在本目录编译产物上
做「类存在 + 成员沿继承链可解析」检查，报告存
[`_diff/api-surface.md`](../oracle/_diff/api-surface.md)（当前：**69 条
MIDP/Nokia 引用全部覆盖，缺失 0**）。

| 包 | 类 | 关键实现点 |
|----|----|-----------|
| `javax.microedition.lcdui` | Canvas / Displayable / Display / Graphics / Image / Font | 240x320 逻辑屏；tick 边界 = `serviceRepaints()`（钩子注入点）；PNG 解码（调色板 1/2/4/8 bit + tRNS）；src-over 整数混合；确定性示意字形 |
| `javax.microedition.midlet` | MIDlet | 生命周期 + `notifyDestroyed` 计数 |
| `javax.microedition.rms` | RecordStore / RecordEnumeration / RecordFilter / RecordComparator | 内存实现（进程内静态 Map），无 I/O |
| `javax.microedition.media` | Manager / Player / Controllable / Control / control.VolumeControl | 状态机 stub（UNREALIZED/REALIZED/PREFETCHED/STARTED），无音频输出，调用记入 trace EVT 流 |
| `javax.microedition.io` | Connector / Connection / InputConnection / OutputConnection / ContentConnection / HttpConnection | **网络永久离线**：`Connector.open` 一律抛 IOException（确定性），驱动原版失败分支（a.java:8321） |
| `com.nokia.mid.ui` | DirectGraphics / DirectUtils | `drawImage(Image,x,y,anchor,transform)` 8 种变换（Nokia 变换码） |

## 源码布局

```
reference/shim/src/
  javax/microedition/{lcdui,midlet,rms,media,io}/...
  com/nokia/mid/ui/...
```

编译入口在 [`reference/oracle/run.py`](../oracle/run.py)（javac → `oracle/_out/classes/`，
gitignore 不入库）。

## 已知语义简化（与真机差异，差分两端一致）

- **字体/字形**：度量为 shim 常量（见 Font.java 注释），字形为按字符码派生的
  确定性示意图案，**非设备字库**。文字布局证据看 trace 的 `OP drawString(...)` 操作流；
  像素哈希用于差分，不用于设备视觉保真（L4 视觉门禁在 P4 另行标定）。
- **音频**：无声音输出。
- **RMS**：跨进程不持久。
- **Graphics**：无 translate 状态（`getTranslateX/Y` 恒 0）、无 drawRegion/dithering；
  只实现参考版引用的方法集（api-surface.md 是权威清单）。

以上简化均为 oracle 策略层决策：未来 Rust 移植做 reference-regression 时以
[oracle 报告](../oracle/_diff/report.md) §4 为策略权威描述。
