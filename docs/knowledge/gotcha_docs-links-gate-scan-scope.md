---
name: docs-links-gate-scan-scope
description: docs-links 门禁默认只扫手写目录清单（docs/+prompts/+3个根文件），新增文档目录会漏检断链
type: gotcha
created: 2026-10-09
sources: ["2026-10-09 架构独立复核会话", "docs/reviews/2026-10-09-architecture-review.md#F1"]
---

`gates/cli.py` 的 `gate_docs_links()` 曾用硬编码目录清单（`[AGENTS.md, CLAUDE.md, README.md]` + `docs/` + `prompts/`）决定扫描哪些 `.md` 文件；凡是不在这个清单里的文档（如 `gates/README.md`、`data/README.md`、`data/naming/README.md`、`original/README.md`、`reference/README.md`）即使存在断链，门禁也会永远 PASS（已用注入坏链接实测复现）。

**为什么：** 这类"白名单式"扫描范围天然滞后于仓库增长——新增一个子目录的 README 就等于给门禁开了一个永久盲区，且不会有任何提示（SKIP/FAIL 都不会出现，是彻底的静默漏报，比 SKIP 更隐蔽）。
**何时使用：** 给任何"文档链接/格式/引用完整性"类门禁定扫描范围时，优先用 `git ls-files '*.md'`（或等价的"遍历实际存在的文件"方式）而不是手写目录清单；手写清单只能作为 git 不可用时的降级兜底。审查门禁代码时，看到 glob/硬编码路径列表要主动问"这个列表会随仓库增长自动扩大吗？"
