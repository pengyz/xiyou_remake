---
name: generated-anchors-need-second-side
description: 保真表逐值锚必须"生成式产出+机械第二侧对照"——手抄互证同源错误防不住；P1 首轮 RED 即抓到手抄错位 idx73/74 实证
type: pattern
created: 2026-10-22
sources: [a284203, 592fe65]
---

资源配置 V2 的锚定纪律（resource-config-survey R1/R2 落地实证）：

1. **逐值锚三元组 `{index, value, a.java:line}` 生成式产出**（脚本从 deobf/javap
   机械提取），禁手抄——表级范围锚在转录错时无法指认哪行错。
2. **机械对照器=第二侧**（tools/table-evidence）：提取值==表值比对 + --check
   确定性复跑；防"两份手抄互证同源错误互相认可"（R2 反例）。
3. **实证**：P1 首轮等值单测即抓到手抄错位 idx73/74（表值 5 vs 提取值 2，
   a.java:573）——生成式不是仪式，真咬合。
4. **勘误文化配套**：锚错位被实锚更正后（如张数表 451→453），活主张原地标注
   日期更正、历史叙述不删（证伪更正文化）；表侧 evidence 逐字段分锚
   （name/count 分行），样例行锚也要分。
