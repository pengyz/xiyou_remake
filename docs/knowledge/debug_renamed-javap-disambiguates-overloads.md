---
name: renamed-javap-disambiguates-overloads
description: 遇到 CFR 同名重载字段/方法歧义时，用 renamed class 的 javap 复核（getfield #NNN 直指唯一符号），不要靠文本猜
type: debug
created: 2026-10-09
sources: ["2026-10-09 t5 执行（2a4a135）"]
---

CFR 反编译文本里同名不同类型的字段（`a`/`A` 重载家族）无法用 grep 消歧；正确排查路径是
对 `tools/rename-pipeline` 产出的 renamed class 跑 `javap -c -p`——机械层已把每个符号唯一名化，
`getfield #NNN`/`getfield f_int_26` 直指唯一字段，一条命令定案（t5 实测：场景门控 `A`
= byte `f_byte_26`，与文本滚动+难度索引共用的 int `f_int_26` 是两个字段）。

**为什么：** 字节码常量池是符号真相源；反编译文本只是投影，重载同名在投影里已坍缩。

**何时使用：** ①考证遇到"A 到底是哪个字段/哪个重载"时；②spec/findings 里的字段级结论
一律以 renamed javap 为最终证据；③语义命名冲突（同一语义疑似两字段）先查 `#NNN` 再定假设。
