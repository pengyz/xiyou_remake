# 项目共享知识库索引

规则见 [README.md](README.md)。每次写入新条目后在对应 type 段追加一行。

## Architecture

## Gotchas

- [docs-links 门禁扫描范围](gotcha_docs-links-gate-scan-scope.md) — 硬编码目录清单会漏检未列入的文档断链，应改用 git ls-files 全量扫描
- [基线哈希字段命名歧义](gotcha_baseline-hash-field-naming-ambiguity.md) — 多阶段产物的哈希字段名必须标注具体文件角色，不能只写阶段名
- [commit 声称必须与内容一致](gotcha_commit-claims-must-match-content.md) — 同批工具调用中 edit 失败但 bash commit 照跑 ⇒ message 声称了未落地变更（实测 413a697）；commit 前逐项核对 `git show --stat`
- [changedPaths 精确匹配语义](gotcha_agentteams-changedpaths-exact-match.md) — inScope 无尾斜杠=精确串非前缀，文件级登记被判 undeclared；契约粒度要与登记粒度对齐
- [cargo glob members 要求目录有 manifest](gotcha_cargo-workspace-glob-requires-manifest.md) — `crates/*` 每个命中目录都是成员候选，缺 Cargo.toml 直接报错；crates/ 只放 crate 目录

## Patterns

- [引用锚定小节而非裸行号](pattern_cite-section-anchor-not-raw-line.md) — 文档互引写「#小节标题（行 N）」双锚；纯行号在追加后必漂移（实测 +19/+73）

## Debug

- [同名重载消歧用 renamed javap](debug_renamed-javap-disambiguates-overloads.md) — CFR 文本的重载同名无法 grep 消歧；对 renamed class 跑 javap，`getfield #NNN` 一条定案

## Decisions

## Reference

- [CFR 0.152 反编译器](reference_cfr_decompiler.md) — 本项目反编译工具与再生命令
