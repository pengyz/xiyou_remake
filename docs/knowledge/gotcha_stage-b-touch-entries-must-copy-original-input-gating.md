---
name: stage-b-touch-entries-must-copy-original-input-gating
description: Stage B 触屏入口（点地寻路 / 探步 / 点行 / 点空白）借用原版状态机时，必须连同原版"何时读输入"的门控与完整收尾一起带过去——只判 game_mode 会穿透浮层，只认 route.is_open() 会在打断后落进剧本收尾（BUG-038~040）
type: gotcha
created: 2026-10-28
sources: [7991bb0, f68fb9b, a87951d, docs/reviews/2026-10-28-settings-controls-design-adversarial-review.md]
---

触屏路径（T2/T3/WIN-OP③）是原版没有的入口，却直接调用原版状态机（tryStep、dp3 脚本行走、各菜单的 -5/-6 臂）。
原版用两道门控决定何时读场上输入：浮层分支先于各模式吞键并整拍返回（a.java:3252-3345）；walkPhase 分派只在 0 相位读场上输入
（a.java:3563-3679）。新入口漏掉任何一道，就会在原版绝不会响应的时刻执行动作。2026-10-28 同一批里实测到三类：
- 只判 `game_mode == Map` ⇒ 框开着点地图会挂寻路或起步撞格，确认框下点格会用掉另一件道具（BUG-038）；
- "点画面 = 确定"一刀切 ⇒ 存档页点空白覆盖存档、炼丹炉点空白花钱（BUG-039）；
- WalkTo 复用 dp3 却只凭 `route.is_open()` 认归属 ⇒ 打断（收束寻路段）后落进剧本行尾收尾写剧情旗标；回 Map 不解除
  `script_walk_armed` ⇒ 单步寻路后手动一步停不下来（BUG-040）。

**为什么：** 借用的状态机自身只认原版输入通道（键值 + 帧节拍），新入口绕过了原版放在通道前面的门控；而"收尾"条件是为原版唯一的驱动方
（脚本 MOV）写的，换一个驱动方（WalkTo）就缺了它的退出条件。各自的单测都能绿，组合起来才暴露——这批缺陷全靠对抗复核的运行时探针
（走 `Runtime::step` 生产路径）才看见。

**何时使用：** 新增或修改任何触屏 / 鼠标 / 手势入口前：
1. 入口函数先过 core 的同一门控：`GameScene::field_tap_allowed()`（场上动作）、`pointer_hit` 浮层期恒 None、`blank_tap_key()`（点空白）；
   不要在壳里按 game_mode 自己判。
2. 借用原版机器（dp3、overlay 臂）时，列出原版驱动方的全部退出条件，逐条确认新驱动方也会触发；需要额外归属位就加（如 `walkto_draining`），
   并在剧情接管点清掉。
3. 用"弹框开着 / 行走中 / 战斗中 / 遮幅中各点一次"的生产路径测试（`shell/android/tests/tap_routing.rs` 体例）收口。
