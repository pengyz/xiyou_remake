# 项目共享知识库索引

规则见 [README.md](README.md)。每次写入新条目后在对应 type 段追加一行。

## Architecture

## Gotchas

- [docs-links 门禁扫描范围](gotcha_docs-links-gate-scan-scope.md) — 硬编码目录清单会漏检未列入的文档断链，应改用 git ls-files 全量扫描
- [基线哈希字段命名歧义](gotcha_baseline-hash-field-naming-ambiguity.md) — 多阶段产物的哈希字段名必须标注具体文件角色，不能只写阶段名
- [commit 声称必须与内容一致](gotcha_commit-claims-must-match-content.md) — 同批工具调用中 edit 失败但 bash commit 照跑 ⇒ message 声称了未落地变更（实测 413a697）；commit 前逐项核对 `git show --stat`
- [changedPaths 精确匹配语义](gotcha_agentteams-changedpaths-exact-match.md) — inScope 无尾斜杠=精确串非前缀，文件级登记被判 undeclared；契约粒度要与登记粒度对齐

## Patterns

## Debug

## Decisions

## Reference

- [CFR 0.152 反编译器](reference_cfr_decompiler.md) — 本项目反编译工具与再生命令
