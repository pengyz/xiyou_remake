# gates/ — 门禁工具链

单一真相源是 `gates/cli.py`（AGENTS.md §5 的门禁表即其命令映射）。
**禁止**内嵌第二套扫描逻辑、禁止绕过/弱化/删除门禁；改门禁必须单独成 commit 并写明基线影响。

## 命令

| 命令 | 作用 | 何时跑 |
|---|---|---|
| `python3 gates/cli.py status` | 只读预检：门禁状态 + 基线对照（**动手前必跑**） | 每次会话开工 |
| `python3 gates/cli.py check` | 总闸（pre-commit 自动调用） | 任何 commit |
| `python3 gates/cli.py original` | JAR sha256 完整性 | 怀疑 original/ 被改 |
| `python3 gates/cli.py reference-seed` | reference/seed/a.java 完整性 | 怀疑 D3 锚点被改 |
| `python3 gates/cli.py reference` | 参考版回归（P1 oracle 就绪后启用） | 改 reference/ 后 |
| `python3 gates/cli.py trace` | 差分 L1–L3（P4 启用） | 改 game-data/game-core 后 |
| `python3 gates/cli.py visual` | 视觉 L4（P4 启用） | 改渲染后 |
| `python3 gates/cli.py table-content-hash` | 保真表内容哈希闸（单跑） | 怀疑 data/tables 被手改 |
| `python3 gates/cli.py table-content-hash-update` | 哈希闸基线重建（**勘误流程末步**，单独 commit 写明原因） | 表勘误+金样重过后 |
| `python3 gates/cli.py table-evidence` | 机械对照器（javap/deobf 提取值==表值第二侧） | 随总闸自动 |
| `python3 gates/cli.py zero-code-drill` | 零改码链演练实跑（survey §3 步 5=验收闸） | 里程碑/验收 |
| `python3 gates/cli.py all` | 全量 | 里程碑收尾 |

## 当前门禁清单（随阶段启用，SKIP 显式打印）

| 门禁 | 内容 | 状态 |
|---|---|---|
| `original-integrity` | `original/` sha256 对照 SHA256SUMS | 启用 |
| `assets-manifest` | JAR 资产清单 ratchet（对照 baseline.json） | 启用 |
| `reference-seed-integrity` | `reference/seed/a.java`（D3 不可变锚点）行数+sha256 对照 baseline.json | 启用 |
| `knowledge-format` | 知识库 frontmatter/type/文件名前缀 | 启用 |
| `docs-links` | 文档相对链接有效性 | 启用 |
| `naming-ledger` | 每条 rename 带 evidence 指针 | 台账创建后自动启用 |
| `rust-tests` | `cargo test --workspace` | crates/ 创建后自动启用 |
| `table-content-hash` | 保真表内容哈希闸（data/tables 三表 sha256 对照 `table_content_hash.json`；变更=勘误流程+金样重过——**非荣誉制**） | 启用（V2·P3） |
| `table-evidence` | 机械对照器进闸（javap/deobf 提取值==表值第二侧，防手抄互证同源错误；复用 `tools/table-evidence/`） | 启用（V2·P3） |
| `zero-code-drill` | 零改码链演练实跑（加假想新敌人=纯表+资源行，零 Rust 改动；失败面三类各一）——验收级，进 `all` 不进 `check` 热路径 | 启用（V2·P3） |
| `reference-regression` | 参考版 trace 逐 tick 不变 | P1 oracle 交付后启用 |
| `diff-trace` | 参考版 vs Rust 差分（L1–L3） | P4 启用 |
| `visual-diff` | 截图逐像素（白名单外） | P4 启用 |

> **mod 骨架四规则**（survey §2.2，`tools/mod-skeleton/`）：mod id 命名空间（≥1000 或
> 前缀）/金样运行禁 mod 表（断言）/保真表哈希闸护航（本清单 `table-content-hash`）/
> mod 独立基线（`tools/mod-skeleton/mod_baseline.json`——**不进金样闸**）。

## 设计原则（移植自 PAL3Decomp 的教训）

1. **SKIP/TOLERATED 显式打印**：静默通过的门禁等于没有门禁。
2. **基线 ratchet**：`data/status/baselines/baseline.json` 只升不降；更新基线必须在 commit 中说明
   "确认无回退"的证据。
3. **失败先定位层**（AGENTS.md §4）：门禁红 ≠ 反复重跑；也 ≠ 调阈值/加白名单。
4. **白名单唯一例外**：`gates/visual_whitelist.json`（P4 建立），变更需用户确认。

## hooks

```bash
bash gates/install-hooks.sh   # 挂接 pre-commit + commit-msg
```

- `pre-commit` → `gates/cli.py check`
- `commit-msg` → 触及 `data/naming/`、`reference/`、`original/` 的提交必须含 `Evidence`/`Confidence`
