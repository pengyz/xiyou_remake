# 项目共享知识库索引

规则见 [README.md](README.md)。每次写入新条目后在对应 type 段追加一行。

## Architecture

## Gotchas

- [全局替换命中刚插入的方法体](gotcha_global-replace-hits-newly-added-method.md) — 先插后换使新方法委托变自递归，尾位置 TCO 成 100% CPU 静默死循环；替换前逐处核对命中上下文
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
- [OS 键盘重复不是 keyPressed](gotcha_os-key-repeat-is-not-keypressed.md) — a.class 未覆写 keyRepeated，长按行走靠 keyHeldCode；壳透传重复 ⇒ 菜单长按连跳/对话连翻（36d1a9d→747ba65 回退）
- [provision 只认 A-<场景> trace 标签](gotcha_provision-trace-labels-a-stem-only.md) — crates 里 join EV-* trace 路径会让 rust-tests 门禁在 CI 缺夹具时 FAIL；取证期望值嵌常量
- [ALSA file: 捕获不节流](gotcha_alsa-file-pcm-unthrottled.md) — null 从设备无时钟，壳音频线程全速写 ~500 MB/s；捕获只宜秒级、通路冒烟用 null

## Patterns

- [引用锚定小节而非裸行号](pattern_cite-section-anchor-not-raw-line.md) — 文档互引写「#小节标题（行 N）」双锚；纯行号在追加后必漂移（实测 +19/+73）
- [双拷贝算法修复必须同步](pattern_dual-copy-algorithms-sync-fix.md) — layout_full_text/layout_wrapped_text_limited 同算法两份拷贝，修一份漏一份 = 偏差换入口复发（迭代 26→30 实证）；review 布局函数强制查姊妹
- [接任务先盘点仓库最新状态](pattern_survey-repo-state-before-research.md) — 多会话推进项目，几百提交可让"待调研问题"早已定案（decision 条目为证）；先 git log + knowledge 再动手

## Debug

- [同名重载消歧用 renamed javap](debug_renamed-javap-disambiguates-overloads.md) — CFR 文本的重载同名无法 grep 消歧；对 renamed class 跑 javap，`getfield #NNN` 一条定案
- [帧残差收敛用 op 逐行对拍](debug_op-stream-diff-convergence.md) — g.ops 前 256 条 vs trace OPS 块 diff，一次定位全部移植笔误；flushOps 块内 256 条上限（count=N 是全量）
- [oracle trace 相位模型](debug_oracle-trace-phase-model.md) — dump@tickN=post-paint；paint 消费同拍 mode-logic；Rust tick_N(fc=N-1)→paint_N 等价；paint 期推进不得挪 tick
- [match 兜底臂吞臂](gotcha_match-catchall-swallows-later-arms.md) — Java switch→Rust match 插臂用 `_ => {}` 会静默吞掉其后全部臂（unreachable 仅警告）；新臂必须显式类型码；末尾「后续批次」占位 `_ => {}` 亦静默放行未移植臂（改 todo!，2026-10-15 型10/11/12 实例）

## Decisions

- [反编译引擎 CFR→Vineflower](decision_decompiler-cfr-to-vineflower.md) — 实测数据驱动切换；含 iinc-vs-i2b 累加器判定规则（编译零错≠行为一致）
- [壳音频 dlopen ALSA 而非 rodio/cpal](decision_shell-audio-dlopen-alsa.md) — alsa-sys 构建期要 libasound2-dev，CI 缺包且违反不依赖系统包；运行时 dlopen 零构建依赖（代价：暂仅 Linux）

## Reference

- [Vineflower 反编译器（现行）](reference_vineflower-decompiler.md) — 位置/命令/产物特征/勿喂 LVT

- [CFR 0.152 反编译器](reference_cfr_decompiler.md) — 本项目反编译工具与再生命令
