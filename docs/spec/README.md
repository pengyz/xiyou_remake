# docs/spec/ — 行为规格书（P2 产物）

行为合同：供 porter 逐条搬运、verifier 逐条核对。**每条规则必须带证据指针**
（`reference/…:行` / 字节偏移 / trace id）并标注可判定性（机器可判定 | 仅人工核对）。

| 文件 | 内容 |
|---|---|
| `resources.md` | 资源格式 spec（packed PNG 头 / maplv 双网格 / sprite 帧表） |
| `script-dsl.md` | 脚本 DSL 语法 + 指令语义表（CES/MOV/TAK/DES/ROS/SEE/…） |
| `state-machine.md` | 游戏状态机（run/paint 模式切换） |
| `gameplay.md` | 玩法规则（移动/战斗/道具/门/隐藏墙/商店） |
| `constants.md` | 数值表（逐值带来源行号） |
| `ui-text.md` | 文本排版（`\c` 颜色码/`\r` 换行/字体度量） |

规则模板见 `prompts/spec-writer.md`。原版 bug 在规则中以 `BUG-xxx` 标注（保真保留，D4）。
