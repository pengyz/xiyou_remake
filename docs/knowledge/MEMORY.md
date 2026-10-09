# 项目共享知识库索引

规则见 [README.md](README.md)。每次写入新条目后在对应 type 段追加一行。

## Architecture

## Gotchas

- [docs-links 门禁扫描范围](gotcha_docs-links-gate-scan-scope.md) — 硬编码目录清单会漏检未列入的文档断链，应改用 git ls-files 全量扫描
- [基线哈希字段命名歧义](gotcha_baseline-hash-field-naming-ambiguity.md) — 多阶段产物的哈希字段名必须标注具体文件角色，不能只写阶段名

## Patterns

## Debug

## Decisions

## Reference

- [CFR 0.152 反编译器](reference_cfr_decompiler.md) — 本项目反编译工具与再生命令
