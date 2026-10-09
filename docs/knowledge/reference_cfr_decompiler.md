---
name: cfr-decompiler
description: CFR 0.152 反编译 a.class 的工具位置、命令与产物路径（参考版种子）
type: reference
created: 2026-10-09
sources: ["2026-10-09 P0 考证会话"]
---

CFR 0.152（Maven Central `org.benf:cfr:0.152`）用于反编译 JAR 内 class。

**位置与命令：** 下载到 `analysis/cfr.jar`（不入库），产物 `reference/seed/a.java`（入库，8457 行）：

```bash
java -jar analysis/cfr.jar analysis/ex/a.class --outputdir analysis/decomp
```

产物溯源：源 `original/囧囧西游-大闹天宫.jar`（sha256 `764cf01e…db20ec`）→ 解包 `a.class`（sha256 `128b7462…d4fa`）。
P1 阶段建议用 Vineflower 交叉比对验证反编译正确性。
