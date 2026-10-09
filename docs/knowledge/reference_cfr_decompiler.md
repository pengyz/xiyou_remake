---
name: cfr-decompiler
description: CFR 0.152 反编译 a.class 的工具位置、命令与产物路径（参考版种子）
type: reference
created: 2026-10-09
sources: ["2026-10-09 P0 考证会话", "2026-10-09 t11 CFR sha256 核验（本地实算 + Maven Central 交叉核对）"]
---

> 【2026-10-10 更正】CFR 0.152 已**退役移除**（vendor/cfr_env/decompile 工具链）：
> 实测其投影修复面 1615 错不可行，引擎切换 Vineflower 1.12.0（决策与数据见
> [decision_decompiler-cfr-to-vineflower.md](decision_decompiler-cfr-to-vineflower.md)，
> 现行引擎参考 [reference_vineflower-decompiler.md](reference_vineflower-decompiler.md)）。
> `reference/seed/a.java`（CFR 产物）作为历史冻结件保留，本条的溯源价值不变。

CFR 0.152（Maven Central `org.benf:cfr:0.152`）用于反编译 JAR 内 class。

**位置与命令：** 依赖以 **vendor 入库**：`tools/vendor/cfr-0.152.jar`（首选，全新 clone 即可用）；
兼容旧工作区的缓存位置 `analysis/cfr.jar`（不入库）。定位/校验/获取的单一真相源是
`tools/cfr_env.py`。产物 `reference/seed/a.java`（入库，8457 行）：

```bash
java -jar tools/vendor/cfr-0.152.jar analysis/ex/a.class --outputdir analysis/decomp
# 一键入口（主逻辑在 .py，跨平台；.sh 仅薄包装）：
python3 tools/decompile.py        # 等价 bash tools/decompile.sh
```

产物溯源：源 `original/囧囧西游-大闹天宫.jar`（sha256 `764cf01e…db20ec`）→ 解包 `a.class`（sha256 `128b7462…d4fa`）。
P1 阶段建议用 Vineflower 交叉比对验证反编译正确性。
产物去向差异（2026-10-09 t12 补记）：`reference/seed/` 保持 CFR 原样转义投影（字符串内非 ASCII 一律 `\uXXXX` 形式，D3 锚点一字节不动）；`reference/src/deobf/` 为重映射产物并经**中文可读化后处理**——字面量内非 ASCII `\uXXXX` 机械解码为 UTF-8 明文（ASCII 转义保留；回环 `encode(decode(x))==x` 证明运行期字符串值零变化），实现见 `tools/rename-pipeline/deunicode.py`。

**完整性校验（2026-10-09 补记，t9-F2）：** `cfr.jar` 完整 sha256 =
`f686e8f3ded377d7bc87d216a90e9e9512df4156e75b06c655a16648ae8765b2`
（本地 `sha256sum` 实算，与 Maven Central `…/cfr-0.152.jar.sha256`
返回值逐字符一致；sha1 `48ef4892cfe8feffddbbd0ff077735140557db74`）。
哈希常量只在 `tools/cfr_env.py` 维护；`tools/decompile.py` 与
`tools/rename-pipeline/run.py` 使用前都经它校验，不匹配即 FAIL 并提示手动放置；
哈希若变更，必须重新与上游核对后更新该常量。

**跨平台与离线用法（2026-10-09 改造）：** 工具逻辑全在 `.py`（纯标准库），
`.sh` 仅是 POSIX 薄包装，**不依赖 curl**（下载兜底走 Python `urllib`）。
查找顺序：`tools/vendor/cfr-0.152.jar`（入库 ⇒ 离线可用）→ `analysis/cfr.jar`
（兼容缓存）→ urllib 下载到 vendor 路径；每步校验 sha256，下载物校验失败即删除。
无网且 vendor 缺失时，手动下载 CFR 0.152 核对上述 sha256 后放入 `tools/vendor/` 即可。
