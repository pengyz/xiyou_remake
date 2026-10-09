---
name: vineflower-decompiler
description: Vineflower 1.12.0 的工具位置、命令、产物路径与已知行为特征（现行反编译引擎）
type: reference
created: 2026-10-10
sources: ["2026-10-10 T-A 可编译实体执行", "决策条目 decision_decompiler-cfr-to-vineflower.md"]
---

Vineflower 1.12.0 是现行反编译引擎（取代 CFR 0.152，见
[decision_decompiler-cfr-to-vineflower.md](decision_decompiler-cfr-to-vineflower.md)）。

**位置与命令：** `tools/vendor/vineflower-1.12.0.jar`（sha256 钉死于
[tools/vf_env.py](../../tools/vf_env.py)：vendor → urllib 下载兜底，校验失败拒绝使用）。
再生：`python3 tools/rename-pipeline/run.py`（管线内调用 `java -jar vf.jar --silent
<renamed/*.class> <outdir>`）。

**产物特征（实测）：** 类体声明 3 空格缩进；字符串字面量原生 UTF-8 中文（仅 5 处
必要的控制字符 `\\u0000`/`\\uffff` 转义）；输出确定性（同输入双跑逐字节一致）；
GOTO 控制流全部原生还原；残余缺陷集中在无 LocalVariableTable 的槽位类型推断
（110 处，由 data/patches/vf-projection/ 18 条声明式补丁修复，javac 零错 + oracle
三方对拍终审）。

**已知行为怪癖（喂 LVT 时）：** slot0 条目不叫 "this" 会造幻影变量（`v0.f_xxx`
替代 `this.f_xxx`）；多段同槽名与参数名打架；部分正确的外部类型信息会被放大成
级联错误（全量 LVT 注入实测 4826 错 vs 基线 110）——**不要给它喂 LVT**。
