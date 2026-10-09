# reference/ — 参考版 Java（差分验证锚点，D3）

**红线**：永不删除、永不改行为。只允许语义等价的改名/拆分/格式化，且逐符号记入 `data/naming/`。

| 路径 | 内容 |
|---|---|
| `seed/a.java` | CFR 0.152 对 `a.class` 的反编译投影（8457 行，**不可变基准**，溯源见 docs/knowledge/reference_cfr_decompiler.md） |
| `src/`（P1 产出） | 反混淆后可编译的参考版 Java（命名以台账为准） |
| `shim/`（P1 产出） | 自研 MIDP-1.0/CLDC + Nokia UI API 子集（headless、虚拟时钟、固定种子、trace 输出） |
| `oracle/`（P1 产出） | 差分测试宿主：跑参考版并产出 trace/状态快照/截图 |

改 `reference/` 后必须跑 `python3 gates/cli.py reference`（trace 逐 tick 不变）。
