---
name: cite-section-anchor-not-raw-line
description: findings/spec 引用应锚定小节标题+行号并存；纯行号在文件追加后必漂移（实测 +19/+73）
type: pattern
created: 2026-10-09
sources: ["2026-10-09 t8 评审观察 2"]
---

文档互引一律写"`<文件>#<小节标题>` + 当前行号"双锚，不得只写裸行号。

**为什么：** 行号在文件被追加/更正后必漂移（实测 `resource-formats.md` 追加 83 行后，
行内代码引用偏 19/73 行——commit 时刻精确、之后失效）；小节标题稳定且可 grep 定位。

**何时使用：** ①findings/spec/台账的 evidence 指针格式统一为
`docs/findings/x.md#小节标题（行 N，随追加漂移）`；②发现行号失效时按小节定位即可，
不必因此发更正（行号是快照，小节是锚）；③评审/验收核对证据指针时以小节为准。
