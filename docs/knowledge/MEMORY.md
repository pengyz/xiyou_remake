# 项目共享知识库索引

规则见 [README.md](README.md)。每次写入新条目后在对应 type 段追加一行。

## Architecture

## Gotchas

- [并行批共享 preset 参数](gotcha_parallel-batch-shared-preset-param.md) — run_variants_parallel 的 preset 是批级参数，混批逐场景预置会静默丢失；指纹=带 preset 场景全分歧、无 preset 全吻合
- [javac 解析期掩蔽](gotcha_javac-parse-phase-masking.md) — 语法错挡住全部类型错误（实测"2 错"实为 4797）；计数必须 -Xmaxerrs 全量
- [docs-links 门禁扫描范围](gotcha_docs-links-gate-scan-scope.md) — 硬编码目录清单会漏检未列入的文档断链，应改用 git ls-files 全量扫描
- [基线哈希字段命名歧义](gotcha_baseline-hash-field-naming-ambiguity.md) — 多阶段产物的哈希字段名必须标注具体文件角色，不能只写阶段名
- [commit 声称必须与内容一致](gotcha_commit-claims-must-match-content.md) — 同批工具调用中 edit 失败但 bash commit 照跑 ⇒ message 声称了未落地变更（实测 413a697）；commit 前逐项核对 `git show --stat`
- [changedPaths 精确匹配语义](gotcha_agentteams-changedpaths-exact-match.md) — inScope 无尾斜杠=精确串非前缀，文件级登记被判 undeclared；契约粒度要与登记粒度对齐
- [cargo glob members 要求目录有 manifest](gotcha_cargo-workspace-glob-requires-manifest.md) — `crates/*` 每个命中目录都是成员候选，缺 Cargo.toml 直接报错；crates/ 只放 crate 目录
- [oracle 帧 PNG 通道序 (A,R,G,B)](gotcha_oracle-png-channel-order.md) — PIL convert('RGB') 取到 (255,R,G)，帧对比会得出 ~99% 假差异；须 convert('RGBA') 取后三通道
- [嵌套 #[test] 被静默跳过](gotcha_nested-test-silently-skipped.md) — 测试 fn 误嵌进上一函数体 ⇒ 编译过、libtest 不发现、假绿可存活多迭代；结论前先 `cargo test <name> -- --list` 核对
- [场景预置必须带 # preset: 头](gotcha_scenario-preset-header-required.md) — 缺头 ⇒ --scenarios 静默无注入、A==B==C 仍绿但语义漂移；再生 trace 一律走头解析
- [GameScene rng 必须续接 machine LCG](gotcha_gamescene-rng-must-continue-machine-lcg.md) — Java 单一 gameRandom 连续消费跨模式不重置；场景新建 seed 0 ⇒ 战斗抖动等首个场景消费者序列错位

## Patterns

- [引用锚定小节而非裸行号](pattern_cite-section-anchor-not-raw-line.md) — 文档互引写「#小节标题（行 N）」双锚；纯行号在追加后必漂移（实测 +19/+73）
- [双拷贝算法修复必须同步](pattern_dual-copy-algorithms-sync-fix.md) — layout_full_text/layout_wrapped_text_limited 同算法两份拷贝，修一份漏一份 = 偏差换入口复发（迭代 26→30 实证）；review 布局函数强制查姊妹

## Debug

- [同名重载消歧用 renamed javap](debug_renamed-javap-disambiguates-overloads.md) — CFR 文本的重载同名无法 grep 消歧；对 renamed class 跑 javap，`getfield #NNN` 一条定案
- [帧残差收敛用 op 逐行对拍](debug_op-stream-diff-convergence.md) — g.ops 前 256 条 vs trace OPS 块 diff，一次定位全部移植笔误；flushOps 块内 256 条上限（count=N 是全量）

## Decisions

- [反编译引擎 CFR→Vineflower](decision_decompiler-cfr-to-vineflower.md) — 实测数据驱动切换；含 iinc-vs-i2b 累加器判定规则（编译零错≠行为一致）

## Reference

- [Vineflower 反编译器（现行）](reference_vineflower-decompiler.md) — 位置/命令/产物特征/勿喂 LVT

- [CFR 0.152 反编译器](reference_cfr_decompiler.md) — 本项目反编译工具与再生命令
