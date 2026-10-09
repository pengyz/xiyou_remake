# 2026-10-09 架构独立复核报告

复核人：AI 实例（独立复核会话，非本仓库此前的开发会话）。
范围：`gates/cli.py` 门禁工具、git hooks、`AGENTS.md` 规则、`prompts/` 角色提示词、
`docs/knowledge/` 知识库规范、`data/naming/` 台账规范、`data/status/baselines/baseline.json`
基线、`.gitignore`、`original/`/`reference/` 红线保护、CI/工程化缺口。

## 方法说明

1. **先只读通读**：AGENTS.md → README/master-plan/bug-ledger/jar-forensics/spec-README →
   gates/README+cli.py+hooks+install-hooks.sh → prompts/README+4 角色 → knowledge/README+MEMORY →
   data/README+naming/README+baseline.json → original/README + reference/README。
2. **实测基线**：`python3 gates/cli.py status` 与 `check`，记录初始 PASS/SKIP 分布作为对照组。
3. **逐门禁构造坏输入**：对每个"启用"状态的门禁，在工作区临时注入一个该门禁理论上应该
   拦截的坏案例（伪造 frontmatter 缺字段、断链、篡改 SHA256SUMS 副本、缺证据的台账行、
   confidence=0 边界、tamper reference/seed/a.java），验证门禁**真的输出 FAIL**，而不是
   仅凭读代码猜测行为。验证后用备份文件逐一还原，`git status --short` 确认工作区干净、
   `diff` 确认关键文件字节级还原。
4. **实测 hooks**：真实 `git commit` 一次缺 Evidence/Confidence 的台账改动，确认被
   commit-msg hook 拦截（exit 1）；再补全后确认通过；用 `git reset --soft` + `git restore
   --staged` 撤销测试 commit，不留痕迹。
5. **交叉核对规则自洽性**：AGENTS.md 门禁表命令 vs gates/README.md 命令表 vs
   gates/cli.py COMMANDS 字典 vs prompts/ 四角色完成判据，逐项核对是否一一对应。
6. **发现问题先取证（A 级：实测命令输出），再修复**；每类修复单独 commit，commit message
   按 AGENTS.md §9 格式，涉及"动手前三问"的跨层修改在 commit message 中明确回答。
7. **修复不弱化门禁**：所有修复均为"新增检查覆盖面"或"澄清歧义命名"，未删除/放宽/
   静默跳过任何既有检查项。F4（original-integrity 的设计局限）与 F6（分支保护）判定为
   "设计局限/决策项"，只在本报告提出建议，不自行修改。

## Findings

| id | 严重级 | 问题 | 证据 | 修复状态 | 遗留建议 |
|---|---|---|---|---|---|
| F1 | high | `docs-links` 门禁覆盖面缺口：硬编码只扫 `docs/`、`prompts/`、根目录 3 个文件（AGENTS.md/CLAUDE.md/README.md），未覆盖 `gates/README.md`、`data/README.md`、`data/naming/README.md`、`original/README.md`、`reference/README.md`。这些文档可以断链而门禁永远 PASS。 | 向 `gates/README.md` 注入一条指向不存在文件的 markdown 链接，修复前 `python3 gates/cli.py check` 仍输出 `[✓ PASS] docs-links`（漏报）；同样方式验证 `data/naming/README.md` | **已修复**（commit `ba75c66`）：改为 `git ls-files '*.md'` 枚举全部已追踪 md 文件，天然覆盖未来新增文档。修复后同一注入变成 `[✗ FAIL]`，验证两个此前漏报的文件均被正确拦截，之后已完整还原无痕迹。 | 无（已闭环） |
| F2 | medium | `baseline.json` 字段 `decomp.seed_sha256` 命名歧义：字段名暗示"seed 文件（`reference/seed/a.java`）的哈希"，但实际存的是反编译**前**的 `a.class`（字节码）哈希，二者是不同文件、不同哈希值。违反 AGENTS.md §3 的"禁止伪语义命名/禁止歧义结论"精神（虽非 rename 场景，但同样是误导性标识）。 | `sha256sum reference/seed/a.java` = `21107ff3…4c1bd`；`unzip -p original/囧囧西游-大闹天宫.jar a.class \| sha256sum` = `128b7462…8d4fa`（与旧字段值相同）。两者不同，证实旧字段名与其实际存储内容不符。 | **已修复**（commit `e00542c`）：拆分为 `seed_a_java_sha256`（真正的 a.java 哈希，新增）与 `seed_source_class_sha256`（原值，重命名澄清用途），`notes` 中按 AGENTS.md §3.6 原地加【2026-10-09 更正】说明，不删除历史含义。 | 无（已闭环） |
| F3 | high | D3 条款规定"参考版 Java 必须永久存在"，`reference/seed/a.java` 是全部差分验证的不可变锚点，但在本次复核前**没有任何门禁校验其完整性**。`baseline.json` 虽记录了 `seed_a_java_lines: 8457`，但该字段从未被 `gates/cli.py` 读取或比对。该文件可被意外改动（哪怕一行）而不触发任何 FAIL，直到后续阶段（P1 参考版回归/P4 差分）才可能暴露，定位成本高且违反"失败先定位层"的设计原则（因为门禁本身就没有信号）。 | 修复前 `grep -n "seed/a.java" gates/cli.py` 为空（无门禁逻辑引用该路径）。构造复现：`echo '// tamper test' >> reference/seed/a.java` 后 `python3 gates/cli.py check` 修复前仍全绿。 | **已修复**（commit `515cb7b`）：新增 `gate_reference_seed_integrity()`，对照 F2 新增的 `seed_a_java_sha256` 做 sha256 + 行数双重校验；接入 `status`/`check`/`all`，新增独立子命令 `reference-seed`；同步更新 `gates/README.md` 门禁清单与命令表。修复后同一 tamper 测试变成 `[✗ FAIL] reference-seed-integrity — sha256 不匹配…行数不匹配…`，之后用备份文件还原，`diff` 确认字节级一致。 | 无（已闭环）。后续若 `reference/seed/` 下有多文件化需求，门禁需跟进扩展（目前只有单文件 `a.java`，与红线文档描述一致）。 |
| F4 | low | `original-integrity` 门禁的设计局限：它只验证 `original/` 下 jar 与**同目录** `SHA256SUMS` 的内部一致性，不能检测"jar 内容和 SHA256SUMS 被同时篡改且保持自洽"的攻击——此时门禁仍 PASS。真正的防线是 git 历史审查（任何改动都会在 `git diff`/`git log` 中留痕），门禁只是"忘记更新哈希"类无意失误的安全网，不是防蓄意篡改的密码学证明。 | 构造测试：同时修改 jar 字节内容并用新 jar 重新生成匹配的 SHA256SUMS，`python3 gates/cli.py check` 仍输出 `[✓ PASS] original-integrity`；`git status --short original/` 显示两个文件均被修改（说明真正的检测手段是 git diff 而非门禁本身）。测试后已用备份完整还原两个文件。 | **未修复（设计局限，不属于"bug"）**。按任务要求"门禁修复不得弱化门禁"且"若认为某检查设计错误需要放宽，只在报告中提出建议，不许自行修改"，此项不是可以单方面加强的点（jar 是二进制大文件，常规的"签名"或"多方哈希"方案超出本次复核授权范围，属于架构决策）。 | **建议用户/Lead 拍板**：若需要防御"同时篡改 jar+SHA256SUMS"场景，可考虑（a）在 commit message 规范中为触及 `original/` 的提交同样要求 Evidence/Confidence（当前 commit-msg hook 只覆盖 `data/naming/` 与 `reference/`，未覆盖 `original/`）；或（b）引入第三方哈希见证（如将哈希也发到 issue/知识库留痕）。本次不自行实现，因为这是红线保护机制的架构变更，需要用户确认。 |
| F5 | low | 工程化缺口：仓库已 `public` 托管于 GitHub（`pengyz/xiyou_remake`），但 push/PR 无任何 CI 自动跑门禁，完全依赖本地 git hooks（可被 `--no-verify` 绕过或未安装）。 | `find .github -type f` 修复前为空。 | **已修复**（commit `33d8e43`）：新增 `.github/workflows/gates.yml`，push/PR 到 `master` 时跑 `python3 gates/cli.py check`（与本地 pre-commit 完全等效，不引入额外检查逻辑），并额外跑一次 `status` 展示 SKIP 项（`if: always()`，仅可见性，不作为失败判据）。本地已验证等效命令 `python3 gates/cli.py check` → `PASS=5 FAIL=0 SKIP=2`；workflow YAML 已用 `yaml.safe_load` 校验语法合法。 | 无（已闭环）。CI 本身已就位，是否要求 CI 通过才能合并（分支保护）见 F6。 |
| F6 | low | 无分支保护：GitHub 仓库未配置 `master` 分支保护规则（无法强制要求 PR + CI 通过才能合并，任何有写权限的人仍可直接 push 到 master）。 | `git branch -a` 仅显示 `master`/`origin/master`，无保护配置可直接观测（分支保护是 GitHub 仓库设置，非本地 git 配置，`gh api` 可查但修改需要仓库管理员权限且是服务端状态变更）。 | **未处理**。这是对公共 GitHub 仓库设置的变更，属于高风险操作（影响所有协作者的工作流、且是服务端状态非本地可逆文件），按任务要求"若加 CI，实现一个最小可用的 workflow 并本地验证"（已完成），分支保护未在任务要求的"实现"范围内，且本就是需要用户拍板的仓库治理决策。 | **建议用户/Lead 拍板**：若希望强制门禁生效（防止任何人直接 push 未过 CI 的代码到 master），应在 GitHub 仓库设置里开启 "Require status checks to pass before merging"（关联 F5 新增的 `gates` workflow）与 "Require pull request before merging"。该操作需要仓库管理员执行，本次复核不代为操作。 |

### 未发现问题的维度（已验证，无需修复）

- `knowledge-format` 门禁：缺字段、type 非法、文件名前缀不符三类坏输入均被正确拦截；
  frontmatter 正则对 body 中出现的 `---`（分隔线）不会误判（非贪婪 `(.*?)` + 锚定首个
  `---\n...\n---\n` 块）。
- `naming-ledger` 门禁：缺 evidence/confidence、JSON 格式错误、evidence 缺可复现指针
  均被正确拦截。注意 `confidence: 0` 边界值不会被误判为"缺失"（`rec.get(key) != 0` 的
  显式豁免逻辑正确处理了 falsy-but-valid 的 0 值）。
- `naming-ledger` 的 `evidence` 字段 schema 与 `data/naming/README.md` 文档描述的
  `id/old/new/kind/evidence/confidence/date/status` 字段一致，门禁强制的必填子集
  （`id/old/new/evidence/confidence`）是文档描述的真子集，无矛盾。
- `knowledge-format` 的 `VALID_TYPES`（`architecture/gotcha/pattern/debug/decision/reference`）
  与 `docs/knowledge/README.md` 文档中列出的 6 类完全一致，无遗漏无多余。
- commit-msg hook 的匹配逻辑（`^(data/naming/|reference/)` 前缀锚定、revert/merge 跳过、
  大小写不敏感的 Evidence/证据 + Confidence/置信度 关键词匹配）经真实 `git commit` 验证：
  缺字段时正确 BLOCKED（exit 1），补全后正确通过（exit 0）。
- hooks 安装状态：`.git/hooks/pre-commit`、`.git/hooks/commit-msg` 均为指向
  `gates/hooks/` 的符号链接，与 `gates/install-hooks.sh` 的实现一致，当前已正确安装。
- AGENTS.md 门禁表命令（`status/check/original/reference/trace/visual/all`）与
  `gates/cli.py` 的 `COMMANDS` 字典（修复后新增 `reference-seed`）一一对应，无遗漏；
  与 `gates/README.md` 的命令表（修复后已同步新增一行）一致。
- `prompts/` 四角色完成判据（re-analyst→`reference`、spec-writer→`check`、
  porter→`trace`、verifier→`all`）均是 `gates/cli.py` 真实存在的命令，无空指向。
- `.gitignore` 的 `assets/*` + `!assets/README.md` 否定模式实测验证有效
  （`git check-ignore -v assets/README.md` 返回"未被忽略"）。
- `original/` 红线：`SHA256SUMS` 本身被 git 跟踪（`git ls-files original/` 可见），
  意味着对 `original/` 的任何改动（含 SHA256SUMS 自身）都会在 `git diff`/`git log`
  中留下历史痕迹，这是比门禁本身更根本的保护层（见 F4 讨论）。
- `baseline.json` 的 `assets` ratchet 字段（`entries/classes/maplv/sprite`）命名清晰、
  无歧义，只有 `decomp.seed_sha256` 一个字段存在命名问题（已作为 F2 修复）。

## 遗留建议汇总（需用户/Lead 拍板）

1. **F4 相关**：是否要求触及 `original/` 的 commit 同样强制 Evidence/Confidence
   （当前 commit-msg hook 只覆盖 `data/naming/` 与 `reference/`）。
2. **F6 相关**：是否在 GitHub 仓库设置中为 `master` 开启分支保护
   （Require PR + Require status checks，关联本次新增的 `gates` CI workflow）。
3. 以上两项均为**仓库治理/红线强度**的架构决策，本次复核按任务要求"不自行放宽/加强
   红线类设计选择"，仅记录供拍板。

> 【2026-10-09 决议（用户拍板，Lead 实施）】
> 1. **F4 采纳方案 (a)**：`commit-msg` hook 已扩展覆盖 `original/`——触及 `original/`、
>    `reference/`、`data/naming/` 的提交一律强制 `Evidence`/`Confidence`。
> 2. **F6 采纳**：`master` 开启分支保护（Require PR + Require status checks（`check`）+
>    0 审批 + enforce_admins）并启用 auto-merge；直接 push master 被禁止，
>    变更一律经分支 + PR + CI 绿合入，工作流写入 AGENTS.md §9。
