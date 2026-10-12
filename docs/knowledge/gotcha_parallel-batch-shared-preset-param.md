---
name: parallel-batch-shared-preset-param
description: run_variants_parallel 的 preset 是批级参数——混批不同预置的场景会静默丢失注入，症状是"带 preset 的场景全分歧、无 preset 的全吻合"
type: gotcha
created: 2026-10-12
sources: [迭代55 provision_tests.py 首版（全量再生 18/22 trace 分歧，T151 槽载入拍起）]
---

`run_variants_parallel(specs, frames, preset, dump_stride)` 的 `preset` 是
**整批共享**参数，不是 spec 元组字段。按 6 元组 `(…, preset_per_scenario)`
组装 specs 却按 5 元组传入时，预置**静默丢失**——所有场景都以空 RMS 启动。
无 compile/类型错误、rc=0、trace 照常产出 ⇒ 极难察觉。

**症状指纹：** 分歧场景集合 = 恰好所有带 `# preset:` 头的场景；无 preset 的
场景与历史基线逐帧全同。分歧起点 = 槽载入确认拍（空 RMS 载入路径分岔）。
用单跑 `run_variant`（显式传 preset）对同一场景复跑：若单跑与基线一致、
批跑不一致 ⇒ 即此坑。

**修复（2026-10-12 入库）：** specs 支持 6 元组（末位逐场景 preset 覆盖批级），
`provision_tests.py` 逐 spec 携带。任何新增批量运行代码：preset 必须逐场景
断言注入（如 `grep -c "preset=" stdout.txt` 或对拍首载入后的状态字段）。
