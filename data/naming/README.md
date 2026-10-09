# data/naming/ — 命名台账（rename 证据账本）

**一条 rename 一条记录。** 由 `gates/cli.py check` 的 `naming-ledger` 门禁强制：
缺 `evidence`/`confidence` 或证据无可复现指针 ⇒ 提交被拒。

## ledger.jsonl 记录格式

```json
{
  "id": "N-0001",
  "old": "a.a(int,int)",
  "new": "Game.movePlayer(int,int)",
  "kind": "method",
  "evidence": "reference/seed/a.java:2834 触碰 this.a[][] 地图网格 + 调用方为 keyPressed 分支；analysis/ex/a.class javap 常量池无其他写点",
  "confidence": 85,
  "date": "2026-10-09",
  "status": "applied"
}
```

- `evidence` 必须含可复现指针：`文件:行` / 字节偏移（`0x…`）/ 可重跑命令。按 AGENTS.md §3 分 A/B/C 级。
- **A/B 级才能 `status: applied`**；C 级假设记 `status: hypothesis`，禁止据此改代码。
- `confidence`: 0–100，与 commit message 中的 Confidence 一致。
- 反悔/证伪：追加新记录 `status: reverted` 引用原 id，并在原条目加 `note`（不删历史，§3.6）。

## 纪律（AGENTS.md §3）

1. 禁止伪语义命名（`levelData_26` 与 `a/b/c` 同罪）；不确定保留 `_todo_`。
2. 禁止批量未验证 rename：逐符号独立取证。
3. 触及 `reference/` 的 commit 必须带 `Evidence`/`Confidence`（commit-msg hook 强制）。
