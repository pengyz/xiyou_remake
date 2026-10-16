---
name: generated-docs-edit-via-config-source
description: coverage-matrix 等生成物禁止手改——判定/销账/注记一律改配置源（tools/cover_matrix.py 的 ARM_GAPS 等）后重生成；手改只作显式声明的过渡态（6467342）
type: decision
created: 2026-10-16
sources: [6467342]
---

生成物文档（`docs/coverage-matrix.md` 等，文件头自declare "请勿手改本文件"）的
判定、销账、注记**一律改配置源后重生成**：coverage-matrix 的配置源是
`tools/cover_matrix.py` 的 PORT_NOTES/TRIAGE/YELLOW_NOTES/ARM_GAPS
（ARM_GAPS 在 :309-315，脚本逐条校验锚点）。手改生成物仅限**显式声明的过渡态**，
且 commit message 必须写明持久化路径（6467342 是实例）。

2026-10-16 实例：lead 指令手改 coverage-matrix 两处刷账，成员核对文件头自declare
后顶回生成物性质、按指令落手改但在 commit 里声明"过渡态"（机械扫描部分重生成
自动对齐无害；§5.1 人工注记重生成即丢，须 task-6 落 ARM_GAPS 后重生成）；
lead 在同一 commit 自纠："让成员手改生成物系指挥失误，销账正确路径=改配置源"。

**为什么：** 生成物与配置源双写必漂移——手改会在下一次 `python3 tools/cover_matrix.py`
重生成时**无痕蒸发**（git diff 里分不清有意修改与生成噪声）；配置源带锚点机械校验
与 Evidence 字段，是可审计的单一真相源。机械扫描段（§5 todo! 清单）的行号锚由
脚本实扫源码生成，本就不该手工维护。

**何时使用：** 动任何生成物文档之前——先看文件头自declare 并找到配置源落点；
收到"手改生成物"类指令时顶回并给出配置源改动方案，确需立即落账则按过渡态三件套
（commit 标过渡态 + 列持久化任务 + 日期更正体例留痕）。判断某文档是否生成物：
grep 文件头 "请勿手改" / "生成物"，再找对应 tools/ 脚本。
