# data/ — 台账与基线

| 路径 | 内容 | 权威性 |
|---|---|---|
| `data/naming/` | 命名台账（rename 证据账本） | rename 的唯一权威记录 |
| `data/status/baselines/` | 门禁基线（ratchet，只升不降） | 门禁判定的对照物 |
| `data/status/_scratch/` | 临时测量（不入库） | 无权威性 |

规则：基线更新必须先确认无回退并在 commit message 写明证据；
台账条目格式见 `data/naming/README.md`（由 `naming-ledger` 门禁强制）。
