# 项目最新状态调研报告（2026-10-14）

> 范围：`2d53af9`（调研者上次在场）→ `b083e6a`（HEAD），**180 个提交 / 300 文件 / +54,695 −9,437 行**。
> 方法：git 历史脉络 + docs/spec 三账本 + gates 源码 + knowledge 决策条目 + 代码规模统计。
> 结论先行：**项目已完成"参考版可编译 + 引擎切换 + 三方对拍"闭环，game-core 移植进入
> 类型闭包收敛期，且游戏已可桌面游玩（game-desktop）**。

## 1. 演进分期（我缺席期间的脉络）

| 阶段 | 提交区间 | 内容 |
|---|---|---|
| T-A 收官 | `3a25e57`…`36f5104` | **变体 C（javac 源码投影）接入 oracle 三方对拍**；反编译引擎 **CFR→Vineflower 1.12.0**（vendor+sha256）；投影修复补丁 v2 + **可编译门禁**；CFR 工具链退役（seed 作历史投影冻结，D3 锚点不变） |
| 对拍扩容 | `e16eb71`…`afd7143` | 深场景套件、trace 语义化、执行提速 100×/4-5×（采样）；RMS 存档预置直达真实玩法；语义命名第二批 40 条 + 第三批；BUG-006/007 登记 |
| P3 移植主体 | `41d8fce`…`09cdccb` | P3.1 脚本解释器（TDD golden + L2 + **diff-trace 门禁激活**）→ P3.2 实体/寻路 → P3.3 行走/相机 → P3.5 行走对拍门禁 → 渲染基础设施（软件光栅器+PNG/inflate+SHA-256 L1b golden）→ paint 调用树 → logo 动画逐 tick 回放（TICK 1-67）→ mode 21/layout → 对抗 review 收口 |
| 方法论定型 | `09458a5`…`226adb1` | 长线收敛计划（四条硬判定+三支柱）、覆盖矩阵生成器、状态差分工具、**类型闭包判据**（替代逐层 L3 行，用户拍板） |
| 最新 | `09cdccb`/`b083e6a` | **game-desktop 桌面壳**（winit 0.30+softbuffer，480×640 2x、75ms 定步、键鼠输入、读档/新游戏双交接、文件版 RMS 存档、55 层资产全装载）+ D2 字形钩子缝；ratchet 基线收录 |

## 2. 反编译器问题——已被项目解决（回应此前质疑的落点）

`docs/knowledge/decision_decompiler-cfr-to-vineflower.md`（2026-10-10）留有完整实测：
CFR+GOTO 补丁 = 修 8 洞后暴露 **1615 错误**（放弃）→ **Vineflower 1.12.0 直出 110
（18 方法/86 行）→ 人肉修复 = 0**；LVT 注入两路线（4826/741）实测净负收益否决；
修复后 oracle **A==B==C 三方 150 tick/72610 行 trace 逐字节一致**——源码投影忠实性
是机器证明。CFR 行号仍锚定 seed（引用体系未断）。此前"调研业界反编译器"是
重复劳动——教训：**动手调研前先读知识库**（引用前验证规则已有，执行不严）。

## 3. 量化现状

| 维度 | 数据 | 出处 |
|---|---|---|
| Rust 代码 | game-core **12,425 行**（scene 5829 / menu_family 929 / paint 785 / render 691 / logo_anim 565 / script 540 / entity 502 …）+ desktop/platform/oracle **10,243 行** | wc -l |
| 门禁 | **12 个**：original / assets_manifest / knowledge_format / docs_links / reference_seed_integrity / naming_ledger / rust_tests / **test_ratchet** / reference / **scenarios** / trace / visual | gates/cli.py |
| 场景套件 | **44 个场景脚本**（boot-menu、menu-sweep、floor1-tour、gameplay-floor1、preset-alchemy、preset-ending…），每个带 `# cover:` 声明，gate_scenarios 跑 A==B==C 三方对拍 | reference/oracle/scenarios/ |
| 语义命名 | 台账三批累计（82+40+第三批），remap-table 581 机械名全量 | data/naming/ |
| 知识库 | **20 条目**（含 javac-parse-phase-masking、gamescene-rng-machine-lcg、oracle-png-channel-order 等实战 gotcha；引擎切换 decision） | docs/knowledge/ |
| Bug 台账 | BUG-001…007 | docs/bug-ledger.md |

## 4. 方法论（收敛期的判定体系）

- **收敛判定**（convergence-plan §0，四条硬判定）：覆盖矩阵无白项（黄项带不触达论证）/
  状态差分零行为分歧（statediff.py 白名单外）/ 帧对拍全绿（挂起清零或带证）/
  completeness + **类型闭包账本无 ⬜**；
- **三支柱**：cover_matrix（静态账本，遗漏变赤字）/ statediff（状态向量逐字段）/
  场景驱动铺层（按矩阵赤字定向）；
- **completeness v1**（2026-10-12 用户设定）：收敛 = 完成整个改写；112 applied 方法
  全落档 / paint 21 臂 + run 16 臂 / DSL 15 码 / 场景面 `# cover:` 全对拍。
  核心洞察已入档："测试全绿只证明声明过的问题没回归，不回答还剩多少没实现"；
- **每轮 ritual**：差分对拍 → 语义回流（spec/findings/knowledge 带行锚）→ gates 全绿
  → 推送；新触达 Java 方法当轮入册。

## 5. 收敛进度

| 账本 | 状态 |
|---|---|
| 覆盖矩阵（103 方法） | 绿 44 / 黄 1 / 白 58 |
| 类型闭包（113 项） | ✅ 73 / 📦+☠️ 7 / **⬜ 36** |
| 楼层 L3（旧判据，已降级为取样） | 14/55 |
| 帧级挂起 | 0（迭代 77 清零；dp2/m_102 随触发层） |

**剩余工作**：B2 道具使用臂+特殊型（10+16 云雾芭蕉扇、凿墙族、86…）、B3 boss 钩子
（67/68/71/74 + floor53/54 终盘群）、B4 脚本臂（GIN/LAY/RES/SEE/SMS/ADD 终裁 +
m_102/dp2）、P4 收官审计（全量回归+矩阵终审+像素抽查，tag `stage-a-complete`）。

## 6. 风险（沿 convergence-plan §4 + 新增）

| 风险 | 状态 |
|---|---|
| 新机制税（每 3-4 层一个） | 矩阵赤字前置 + 批次聚类，进行中 |
| 终盘 boss 群 floor53/54 | P3 单列预留多轮 |
| game-desktop 新壳（刚落地） | D2 字形钩子缝已留；桌面壳属 Stage A 外的分发面，需确认是否纳入收敛判据 |
| 12 门禁全量运行时长 | scenarios 套件重（milestone 级）；日常用 check 子集 |

## 7. 建议的下一步（供拍板）

1. 按账本差值继续 B2/B3/B4 批次（36 ⬜ 清零是主战场）；
2. game-desktop 与 Stage A 收敛判据的关系需要用户裁定（建议：桌面壳作为**验收载体**
   不进判据，P4 像素抽查可借它出图）；
3. 本次调研教训入库（见 knowledge 附注）。
