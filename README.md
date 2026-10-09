# 囧囧西游-大闹天宫 · Rust 重制

Nokia J2ME（MIDP-1.0）解谜 RPG《囧囧西游之大闹天宫》（SNOWFISH, 2010）的逆向重制工程：
反编译 → 理解逻辑 → Rust 跨平台改写 → 差分验证。
**Stage A 目标：像素级一致**；Stage B：现代化改造。

## AI First

本项目由 AI 实例驱动开发，纪律单一真相源：**[AGENTS.md](AGENTS.md)**（动手前必读）。
门禁：`python3 gates/cli.py status`（预检）/ `check`（提交总闸）。

## 仓库地图

| 路径 | 内容 |
|---|---|
| `original/` | 原始 JAR 只读归档（唯一资产真相源，sha256 校验） |
| `reference/` | 参考版 Java（差分验证锚点，D3）+ MIDP shim + oracle |
| `docs/master-plan.md` | 工程路线/阶段/门禁/风险/决策 |
| `docs/findings/` | 逆向考证记录（已证实/假设分节） |
| `docs/spec/` | 行为规格书（P2） |
| `docs/knowledge/` | AI 共享知识库（6 类） |
| `docs/bug-ledger.md` | 原版 bug 台账（Stage A 保真，Stage B 修） |
| `prompts/` | 角色提示词（re-analyst/spec-writer/porter/verifier） |
| `gates/` | 门禁工具 + git hooks |
| `crates/` | Rust workspace（game-data/core/platform/oracle/desktop/wasm） |
| `data/` | 命名台账 + 门禁基线 |
| `tools/` | 资源提取/改名管线/trace 比对 |

## 快速开始

```bash
bash gates/install-hooks.sh        # 挂接 git hooks
python3 gates/cli.py status        # 门禁预检（应全绿或显式 SKIP）
```

CI：`.github/workflows/gates.yml` 在 push/PR 到 `master` 时自动跑 `gates/cli.py check`
（本地等效命令同上，CI 不引入额外检查逻辑）。

## 当前进度

- [x] P0 资产盘点与混淆等级评估（`docs/findings/jar-forensics.md`）
- [ ] P1 反混淆（参考版 Java + 命名台账）
- [ ] P2 理解逻辑（规格书）
- [ ] P3 Rust 改写
- [ ] P4 差分验证 → Stage A 完成
