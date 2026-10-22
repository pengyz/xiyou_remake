---
name: subagent-delegation-unstable
description: 本会话子 agent 后端高危（8+ 早夭）——重活转包必挂；药方=teammate 亲做+具体首动作破局+"骨架先行落盘"防中断丢产出
type: gotcha
created: 2026-10-22
sources: [cc90183f, 5e7ed1e1, 394fa0d2, 8fe4aa09, 208379e6]
---

2026-10-21/22 会话实测：子 agent（subagent 类工具）**8+ 次中途失败无产出**，覆盖
复核、生成、验收各类任务。转包失败会把 teammate 拖停摆（其回合把任务整体转包
给子 agent，子 agent 死 = 零产出 + member inactive）。

**有效药方**（3 次验证：hd-demo-atlas / v2-parser-reject / hd-render-path）：
1. **禁转包**——重活派 teammate 时直接写明"亲自 inline，禁止委托 subagent"；
2. **具体首动作破局**——停摆后重派不给抽象指令，给"立即 write 某文件/写第一类
   红测试"级别的首个动作，做起来后就顺了；
3. **骨架先行落盘**——长文档类任务要求"先 write 骨架再逐节填充"，中断也留半成品
   （三路对抗复核的 fidelity/pipeline 件就是骨架救回来的）；
4. 子 agent 遗言有价值（环境坑结论如 pytest 损坏→stdlib unittest），**转递主线**
   防重踩。
