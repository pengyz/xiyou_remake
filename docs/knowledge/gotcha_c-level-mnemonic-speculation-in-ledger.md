---
name: c-level-mnemonic-speculation-in-ledger
description: mnemonic 语义推测混入结论的事故模式——SMS="平台短信调用"纯属望文生义（C 级），真臂只是 parseScriptInt+游标空转；副作用断言必须指认调用行，否则不入账
type: gotcha
created: 2026-10-16
sources: [closure-typist type-closure SMS 终裁 2026-10-16（commit 待 lead 补），docs/spec/type-closure.md:160]
---

"语义好听" ≠ "代码如此"：SMS 臂曾被写成"Java 侧平台短信调用，桌面不可复现
（吸收型）"入账——纯属助记词望文生义（SMS→短信→付费短信时代产物），C 级推测
伪装成合理结论混过审查。closure-typist 反编译取证反转（2026-10-16）：
**a.java:8515-8518 的 SMS 臂只有 `parseScriptInt` 读参 + `scriptCursor = lastDelimPos + 1`
跳行，无任何平台/短信调用**；Rust 桩（scene.rs:2053-2056）与之逐字等价，语料仅
script66 `SMS_0` 一处——"吸收型"实际成立的只是语义空转，终裁 ☠️。

**为什么：** 符号名/助记词与语义的联想是 C 级污染的高发源，且**推测越自圆其说越
难拦**——"付费短信残迹被桌面剥除"听起来天衣无缝，还顺势解释了为什么测不到副作用。
拦它只有一条硬规则：**副作用/平台调用类断言必须指认具体调用行；指认不出 = 不入账**
（本例证否法：对方法体 grep 调用清单，SMS 臂只有 2 行——parseScriptInt + 游标推进）。
AGENTS.md §3 已有"C 级只进 findings 假设区"，这条补的是它的**失效模式**：合理化
叙事会让 C 级伪装成 B 级"语境推断"入账，逐字摘录+调用清单才是照妖镜。

**何时使用：** 起草 spec/台账任何"外部副作用""平台调用""不可复现所以吸收"性质的
结论时——先要求给出调用行号 + 原文摘录；给不出就降级进 findings 假设区并注明
"待证否"。复核他人结论时对"叙事特别顺"的条目加倍警惕，优先做证否 grep
（整方法体的调用清单 vs 声称的副作用）。
