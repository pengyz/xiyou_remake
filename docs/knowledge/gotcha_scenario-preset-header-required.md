---
name: scenario-preset-header-required
description: 场景脚本用 RMS 预置必须在头部声明 `# preset: <file>`——缺头时套件静默无注入，A==B==C 仍绿但语义漂移
type: gotcha
created: 2026-10-12
sources: [迭代 28 preset-help/load-menu 重生成（旧档由会话外显式传参生成，套件一直无预置跑）]
---

`run.py --scenarios` 只认场景脚本头部的 `# preset: <file>` 声明来注入 RMS 预置；
文件名与 `<场景名>.preset` 的巧合一致不是机制。缺头 ⇒ 不注入 ⇒ 游戏在空 RMS 上
照常运行，**A==B==C 依然全绿**（三方同样无注入），但与"该场景声称的覆盖面"
（以及 Rust 对拍基准 trace 的生成条件）不符——实测 preset-help/load-menu/item13
三个场景无头存活多个迭代。

**为什么：** 注入失败只有在"声明的文件不存在"时才硬失败
（RecordStore.applyPreset FileNotFoundException → 静态初始化错，首个 RMS 触点
即死，trace `END reason=failed`）；"根本没声明"则完全静默。判定预置是否生效
看帧内容（如槽列表应显示有效槽而非 `----`）或 FLD 里 SKY_WAR 数组。

**何时使用：** 新增 preset 场景时必须带 `# preset:` 头；再生 trace 用
`run.py` 的头解析语义（或直接跑 `--scenarios`），**不要**按文件名约定自行推导
预置路径。ttransform/套件外的单场景驱动若图省事传错路径，失败形态是
T≈67 的 ExceptionInInitializerError（音效窗首触 SKY_WAR），见
preset_rms.py 头注释的 SKY_WAR 读点。


**手工单场景再生的同型坑（迭代 40 补记）：** 逐场景 `run_variants_parallel`
再生时漏传 `preset=` 参数 ⇒ 同样产出"卡空槽列表"的静默退化 trace
（2026-10-12 18:2x 批次：A-walk-pure/A-floor1-tour/A-floor1-to-floor2 三例，
帧数齐全、A==B==C 可绿，唯独永不进入 mode 3）。再生后**必须核验 trace
状态转储出现 `FLD 010 byte 3`**（进入 mode 3）才算再生有效。
