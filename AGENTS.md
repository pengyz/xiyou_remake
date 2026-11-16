# AGENTS.md — AI 纪律总纲（单一真相源）

> 面向所有 AI 实例（DSH/Claude Code/Codex/其他）。本文件是纪律的**唯一权威**；
> 其他文档与本文件冲突时，以本文件为准并提交更正。人类决策见 [docs/master-plan.md](docs/master-plan.md)。

## 0. 项目与已锁定决策

《囧囧西游-大闹天宫》（Nokia J2ME / MIDP-1.0，SNOWFISH 2010）重制：
反编译 → 理解 → **Rust 跨平台改写** → 差分验证。~~两阶段：**Stage A 像素级一致** → **Stage B 现代化**。~~

> 【2026-11-17 用户裁定：**不保留 Stage A**】「像素级一致」不再作为目标与验收契约；
> Rust 实现即产品、即规格（见 D5）。参考版与差分设施保留，但角色降为历史参照与回归防护。

| # | 决策 | 内容 |
|---|---|---|
| D1 | 语言 | Rust（桌面 + wasm，预留移动端）。C++/TS 已否决 |
| D2 | 路线 | ~~先像素级一致，后现代化~~ **【2026-11-17 终止】** 直接在 Rust 实现上演进；扩展缝（分辨率/输入/字体/素材/音频/存档）照旧预留 |
| D3 | 锚点 | 参考版 Java **永久归档、不得删除或篡改**（bytecode → 反混淆 Java）；~~一切差分验证的裁判基准~~ **【2026-11-17 改判】不再是行为规格**，仅作历史参照与取证来源 |
| D4 | Bug | ~~Stage A 保真保留原版 bug~~ **【2026-11-17 改判】** Rust 侧行为以设计意图为准；[docs/bug-ledger.md](docs/bug-ledger.md) 保留为历史背景，修不修按收益决定 |
| D5 | 权威 | **Rust 实现（game-core 及其上层）是唯一行为真相源**。行为争议由我们的代码、规格书、金样裁决；Java 无否决权 |
| D6 | 证据 | 资源/资产**格式**结论仍须 A/B 级字节级取证；**行为类结论不再要求原版证据**，以内部一致性 + 回归为准 |

## 1. 哲学（决策优先级）

**逻辑一致性 > 代码优雅 > 工具便利 > 个人偏好。**

- **裁判是我们自己的实现，不是"看起来对"，也不再有外部证人**。正确性由三条判定：
  内部一致性（多路径/多实现必须给同一结果）+ 回归金样（改动不得静默改变既有行为）+
  不变量（资源下的性质，如血量单调、钥匙守恒）。人肉 review 只能发现风格问题，不能证明等价。
- **参考版 Java 是历史存档**：永不删除、永不篡改；但**它不定义 Rust 行为**——
  禁止以"原版如此"为唯一理由修改 Rust 逻辑（见 D5/D6）。
- **反编译是信息有损的**：原版字节码/反编译文本仅作为**数据格式与历史取证来源**，
  不是行为规格；数据提取结论仍须 A/B 级证据。
- **行为变更必须可追溯**：有意改变行为时，在 commit message 写明意图并同步更新金样/基线；
  无意的行为漂移由回归门禁拦下。

## 2. 仓库结构与红线

| 路径 | 职责 | 红线 |
|---|---|---|
| `original/` | 原始 JAR 只读归档（唯一资产真相源） | **任何字节修改 = 严重违规**；由 `gates/cli.py original` 校验 sha256 |
| `reference/` | 参考版 Java：反混淆产物 + 自研 MIDP shim + trace 钩子 | 只允许**语义等价**的改名/拆分/格式化；行为变化必须差分证据逐 tick 不变 |
| `crates/game-data/` | 资源解析（packed PNG/maplv/sprite/script DSL） | 纯解析无渲染；解析结果必须过 L1 golden test |
| `crates/game-core/` | 游戏逻辑（状态机/脚本解释器/战斗/道具） | **无 I/O、确定性**：只用虚拟时钟与 GameRng；禁止引用平台 crate |
| `crates/game-platform/` | 平台 trait（Clock/GameRng/Renderer/Audio/Storage/Input/Font） | 只放 trait 与类型；实现只在 game-desktop/game-wasm |
| `crates/game-oracle/` | 差分测试宿主（trace/快照/截图产出） | 比对逻辑不得为"通过测试"而放宽阈值 |
| `gates/` | 门禁工具与 git hooks | 不得绕过/弱化/删除门禁；改门禁必须跑门禁自测并写明基线影响 |
| `docs/spec/` | 规格书（P2 产物） | 每条结论必须带证据指针（代码行/字节偏移/资源偏移/trace id） |
| `docs/knowledge/` | 跨会话共享知识库 | 写入规则见其 [README](docs/knowledge/README.md)；引用前必须验证仍存在 |
| `docs/findings/` | 考证记录 | **必须区分"已证实 / 假设"**，假设不得混入结论 |
| `data/naming/` | 命名台账（rename 的证据账本） | 每条 rename 必须有 evidence 字段（见 `data/naming/README.md`） |
| `data/status/baselines/` | 门禁基线（ratchet） | 基线只能在确认无回退后更新，更新必须写 commit 说明 |
| `prompts/` | 角色提示词 | 新任务必须先选角色 prompt（见 §7） |
| `analysis/` `assets/` `target/` | 可再生工作区 | 不入库；由 `original/` + 工具可复现 |

## 3. 证据纪律（本项目最核心的纪律）

证据分级（与台账/commit message 中的 `Confidence` 配套）：

| 级 | 定义 | 例 |
|---|---|---|
| **A** | 硬证据：字节偏移、javap 字节码、资源 hexdump、运行时 trace | `maplv0` 头 `1a00 1a00` = u16 26×26；`a.java:7178` 读 `maplv{n}` |
| **B** | 代码语境 + 调用图推断（可复现的静态推理） | 方法触碰 `Graphics` ⇒ 渲染层；`TAK_8_9` 两端都是文本区间 ⇒ 对话指令 |
| **C** | 直觉/剧情猜测/命名相似 | "ROS 可能是 rotate" |

规则：

1. **A/B 级才能落台账、改名、写入 spec 结论**；C 级只能进 findings 的"假设区"，禁止据此 rename。
2. **禁止伪语义命名**：`levelData_26`、`mapThing2` 这类地址/序号后缀伪装的名字没有语义价值，
   与 `a/b/c` 无异。不确定含义就保留 `_todo_`，并在台账记 C 级假设。
3. **禁止猜测性格式结论**：资源格式的"已证实"必须能给出字节级复现命令（hexdump/python 一行验证）。
4. **禁止批量未验证 rename**：一个符号一条台账记录，证据独立可查。
5. **禁止试错式修改**：不理解机制前不动解析器/解释器。先考证（hexdump → 反编译代码 → 运行时插桩），
   画出完整使用图景，再动手。
6. **证伪更正文化**：结论被推翻后，在原文**原地标注日期更正**（`> 【2026-10-09 更正】…`），
   不删除历史主张。读者必须能看到"曾经错在哪"。

## 4. 动手前三问（跨层修改的强制预检）

修改任何跨层内容（资源格式/参考版/核心逻辑/门禁/台账规则）前，必须**只读地**回答，
并把答案写进 commit message：

1. **问题在哪一层？**（反编译投影错？格式理解错？Rust 移植错？门禁本身错？）
2. **消费者实际读的是哪个文件/哪份数据？**（解析器读原始字节还是转换产物？测试夹具来自哪里？）
3. **证据是什么级别？**（答不出 A/B 级证据 ⇒ 不许动手，先去取证。）

## 5. 强制检查点（门禁表）

先跑只读预检 `python3 gates/cli.py status` —— 它列出各门禁的启用状态与已知恒红/SKIP 项。
**SKIP 与 TOLERATED 必须显式打印，绝不静默通过。**

| 时机 | 门禁 | 命令 | 失败处理 |
|---|---|---|---|
| 动手前 | 状态预检 | `python3 gates/cli.py status` | 看清 SKIP/恒红项再动 |
| **任何 commit** | 总闸（pre-commit 自动） | `python3 gates/cli.py check` | 任一 FAIL ⇒ 不提交 |
| 改 `data/naming/` / `reference/` / `original/` | 证据门禁（commit-msg 自动） | commit message 含 `Evidence`/`证据` | 缺证据字段 ⇒ 拒绝 commit |
| 改 `reference/` 后 | 归档完整性 | `python3 gates/cli.py reference` | 归档不得被改（historical 只读，不是行为对照） |
| 改 game-data/game-core 后 | 回归（L1–L3） | `python3 gates/cli.py trace` | 与上一基线比**任何意外回退** ⇒ 停止，先定位层（§4）；有意变更须更新基线并写意图 |
| 改渲染后 | 视觉回归（L4） | `python3 gates/cli.py visual` | 白名单外像素不得意外变化（有意改渲染须更新基线） |
| 里程碑收尾 | 全量 | `python3 gates/cli.py all` | 对照 master-plan 阶段 Gate |

- **门禁失败不是"多跑几次"能解决的**：按 §4 定位层，修复根因，不许调阈值/加白名单掩盖。
- **验证不打扰（2026-10-15 用户指示）**：实机/UI 验证一律走**无头路径**——Xvfb 独立显示、
  game-oracle 逻辑级 harness、纯函数测试；**禁止在用户桌面上抢前台（windowactivate）、
  发键、抓屏**。产品体验由用户人工试玩后反馈；归因时不得把用户的操作记为验证成果。
- 视觉白名单（字体区域等）是**唯一**允许的例外清单，其变更需要用户确认并记录。

## 6. Bug 台账纪律（D4 的执行细则）

- 运行中发现可疑行为 ⇒ 记入 [docs/bug-ledger.md](docs/bug-ledger.md)（现象/复现/影响/trace id），
  代码侧以 `BUG-xxx` 注释引用；**修不修按收益决定（D4 改判）**，修时须说明动机与影响面。
- 差分测试遇到已登记 bug：登记为"已知偏差"，在基线中显式 TOLERATED 并注明 BUG 编号；
  有意改变该行为时按「有意变更」流程更新基线。
- 禁止无记录的顺手改行为——每次行为变更都必须能追溯到 commit 与基线更新。

## 7. 提示词纪律

详见 [prompts/README.md](prompts/README.md)。要点：

- 新任务必须先选定角色 prompt（re-analyst / spec-writer / porter / verifier）；
- 任何 prompt 的产出必须包含：证据引用、完成判据（跑哪个门禁）、遗留假设清单；
- **产出的归宿是仓库**（台账/spec/代码/知识库），不是聊天记录——聊天里的结论不许作为后续依据。

## 8. 知识库

[`docs/knowledge/`](docs/knowledge/README.md)：跨会话共享知识（6 类 type：architecture/gotcha/pattern/debug/decision/reference）。
遇到平台坑、门禁失效、被纠正的方法、确认有效的取舍时，按其 README 写入；
**引用前必须验证**文件/命令仍存在（"知识库说 X 存在" ≠ "X 现在存在"）。

## 9. 提交规范

```
<type>(<scope>): <subject>          # type: feat/fix/docs/kb/gate/refactor/test/chore
                                    # scope: data/core/platform/ref/gates/docs/tools
<空行>
<正文：改了什么、为什么、证据与影响>
<触及 data/naming/ / reference/ / original/ 时必须含：>
Evidence: <A/B 级证据描述>
Confidence: <0-100>
```

小步提交：一个 commit 一个逻辑变更。门禁改动单独成 commit，不得与功能变更混提。

**master 保护（2026-10-09 起，F6 决议 + 同日修订）**：owner（admin）**豁免**保护规则——
可直推 `master`，也可在 `dev` 等分支开发后本地合入再直推，**无需 PR**。
安全网三层：本地 `pre-commit`（`gates check`）+ `commit-msg`（证据门禁）实时拦截，
CI 每次 push `master` 自动体检（红了就地修或 revert）。
外部贡献者（非 admin）必须走 PR 且 CI `check` 绿才能合入（0 审批，auto-merge 可用）：

```bash
# owner 日常：直推即可
git push origin master
# 外部贡献 / 需要评审的大变更（可选）：
git push -u origin <branch> && gh pr create --fill && gh pr merge --auto
```

CI 与本地 `gates/cli.py check` 完全等效。CI 只在 push `master` 与 PR 时触发，`dev` 分支推送不跑。
