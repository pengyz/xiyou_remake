---
name: renaming-breaks-vf-patch-anchors
description: 语义改名触碰 vf-projection 补丁锚点会 PatchError，且失败后盘上残留 VF 原始输出极易被误读为"改名腐坏"
type: gotcha
created: 2026-10-15
sources: ["2026-10-15 命名回填第二批（dc470b2）二分定位实证"]
---

## 现象

`data/naming/ledger.jsonl` 的语义改名若触碰 `data/patches/vf-projection/*.json`
的**方法锚点**或**替换体 old/new 文本**，`tools/rename-pipeline/run.py` 会报
`方法锚点命中 0 次（须唯一）` / `锚点未命中` 并 exit=1。实例：改名 `m_113`/`m_100`
打断 VF-M113 / VF-ASCE / VF-SINGLES-TYPES 三文件共 5 处锚。

## 误导性副产物（本次排查曾被误导两轮）

run.py 失败时"拒绝落盘"仅指**补丁产物不写**——Vineflower 原始输出此时已在盘上。
未打补丁的 ~11 处槽位类型投影缺陷（`(this = this)`、`var10001`、`int`→`short`）
会让 git diff 看起来像"改名把代码改坏了"。**判读前先看退出码与 PatchError 文本**，
勿拿 git diff 腐坏计数当改名正确性判据。

## 正确做法

1. 改名前：`grep -rn "<机械名>(" data/patches/vf-projection/`；
2. 命中则**同批更新**锚点与替换体文本（补丁即数据，属设计内维护）；
3. 改名后四门：run.py 产物自检 → `--verify`（符号还原逐字节）→ javac（**真退出码**，
   `javac ... | tail -1` 会掩盖失败）→ cargo test。

## 附带教训

- 台账 id 自增前先查现存最大值：N-0301..0343 曾双占 t5 旧账（撞号不影响管线但毁账本可审计性）。
- run.py 失败后工作树里是 VF 原始输出，任何基于 diff 的判断都要先 `git checkout` 复位。
