# 项目共享知识库 — 写入规则

面向所有 AI 实例（DSH / Claude Code / Codex / 其他协作工具）：会话中遇到下面时刻之一，按本规则写入本目录。
（机制移植自 PAL3Decomp 的 `docs/knowledge/`，适配 J2ME 重制管线场景。）

## 写入时机

1. **commit 后** — 用户主动提及刚 commit 的内容
2. **用户纠正方法** — "别这样做"、"这不对" → 多为 `pattern` / `gotcha`
3. **用户确认非显然方法有效** — 接受了非常规选择 → `decision`（正反馈也写，避免知识库只剩避雷）
4. **遇到平台坑/反直觉行为** — CFR/javap/MIDP shim/Rust/资源格式的坑 → `gotcha` / `debug`
5. **发现门禁失效或流程漏洞** — hook 未生效、校验有盲区、基线口径不一致 → `gotcha`（这类最贵，漏掉必重复踩）

判断标准：另一个 AI 实例接手这个项目时，这条知识能帮它少走弯路吗？不能 → 不写。

## 类型（6 类，文件名前缀 = type）

| type | 记什么 | 何时写 | 正文结构 |
|---|---|---|---|
| `architecture` | 管线/架构决策、层职责边界、为什么 A 不选 B | 方案被确认采纳、非显然取舍 | 事实一句 + **为什么：** + **何时使用：** |
| `gotcha` | 工具/格式/门禁的坑、反直觉行为 | 触碰某工具导致非预期结果、发现校验盲区 | 现象 + **为什么：** + **何时使用：** |
| `pattern` | 命名/改写/验证的约定与反模式 | 同类问题第二次出现、确立约定 | 规则 + **为什么：** + **何时使用：** |
| `debug` | 排查路径、关键命令、诊断手段 | 找到高效诊断手段时 | 动作（命令/路径）+ **为什么：** + **何时使用：** |
| `decision` | 已验证的取舍（含正反馈） | 用户接受非显然方案、事后确认走对 | 选择 + **为什么：** + **何时使用：** |
| `reference` | 外部资源指针、本机环境、工具位置 | 搭建可复用工具链时 | 资源位置 + 一句用途（免 Why/When） |

## 不写入

- 代码/spec 本身能表达的 → 读代码
- `git log` / 台账（`data/naming/`、`docs/bug-ledger.md`）能查到的具体条目 → 那里是权威
- `AGENTS.md` 已有的规则 → 不重复
- 临时调试状态、进行中的工作 → 不持久化
- 具体某个 rename/格式考证的过程 → 证据在台账/findings，这里只写"通用规律"

即使用户明确要求保存上述内容，也要先反问"其中有什么*非显然*的部分？"——那部分才值得记。

## 文件结构

每条知识 1 个文件：`docs/knowledge/<type>_<slug>.md`。frontmatter 必填：

```markdown
---
name: <kebab-case-slug>
description: <一行钩子，决定未来召回相关性>
type: architecture | gotcha | pattern | debug | decision | reference
created: YYYY-MM-DD
sources: [<commit-hash 或 session 描述>]
---

<事实陈述，一句>

**为什么：** ...
**何时使用：** ...
```

`reference` 可省 Why/When，其他五类必填。

## 写入流程

1. Read `docs/knowledge/MEMORY.md` 与本文件
2. 用 description 与现有条目对照 → 已有相似条目则**更新**（含日期更正），不新建
3. 写入 `docs/knowledge/<type>_<slug>.md`
4. 在 `MEMORY.md` 对应 type 段追加一行索引
5. 同步追加一行到 `docs/knowledge/kairos/YYYY/MM/YYYY-MM-DD.md`

## Kairos 日志（事件流）

`docs/knowledge/kairos/YYYY/MM/YYYY-MM-DD.md` 是 commit/会话级事件流，作为未来蒸馏的增量素材。
每次写入 knowledge 条目 → 同步追加一行；没产生条目但发生项目级动作（大批量 rename、阶段结论）→ 仅追加一行 kairos。
格式：`- HH:MM — <动作>：<一句话>`，单文件 ≤ 100KB。

## 引用前的验证（防 stale）

引用本目录提到的文件/命令/符号前必须：文件 → Read/ls 验证存在；符号 → grep 台账/代码验证未被重命名；
命令 → `--help` 或实跑验证。验证失败 → 不引用 + 标记为失效候选。

## 证伪更正文化（AGENTS.md §3.6）

条目被推翻时**原地**加 `> 【YYYY-MM-DD 更正】…` 说明，不删除原主张。
