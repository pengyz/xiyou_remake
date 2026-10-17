---
name: save-path-shell-injection
description: 存档/读档路径由外部 shell 注入的契约——平台路径规范差异（Windows exe 目录 vs Android 应用私有目录），game-core 零 I/O
type: decision
created: 2026-10-17
sources: ["2026-10-17 用户架构决策（BUG-018 存档环修复连带）"]
---

# 决策：存档/读档路径由外部 shell 注入（平台路径规范差异）

- 日期：2026-10-17
- 状态：已采纳（用户决策）

## 内容

存档与读档的**文件路径属于平台层**，由外部 shell 传入，游戏层不自造路径：

- `game-core` 保持无 I/O（存档写虚拟 RMS 字段，落盘/回读全在 shell 接线）；
- `game-desktop`（Windows shell）：`--save <dir>` 显式注入；未注入时默认
  `exe目录\save`（便携、抗 cwd 变化——原 cwd 相对 `save/` 换目录启动即"丢档"）；
- Android（后续）：宿主启动时注入应用私有目录（app-private files dir），
  走同一注入缝（`--save` / 未来宿主 API），**不得**在游戏层写死任何平台路径。

## 依据

- 平台路径规范各异（Windows 便携/exe 相对 vs Android 应用私有存储）——
  路径解析是平台职责（D1 跨平台决策的自然延伸）；
- BUG-018（2026-10-17）实证：存档环断点之一即路径/键域接线问题，
  壳层接线是存档持久性的唯一真相源。

## 影响面

- 新平台接入（Android/wasm）：宿主传入路径即可，游戏层零改动；
- 资产/音频目录同理（`--assets`/`--audio` 已是注入缝，平台默认可同式演进）。
