---
name: dual-porting-surfaces-drift
description: 同一 Java 方法两套 Rust 移植面并存（scene.rs 生产体 vs script.rs HostCtx harness 面；camera.rs 第二套相机状态）——改任一面必须对照另一面，禁造第三面；DC-9 桥漏道具加成=漂移已发生实例
type: gotcha
created: 2026-10-16
sources: [0b37d77（task-7 死码裁定入库，DC-9/DC-10 落账）, e0078fc（死码复核扩引理）, docs/spec/dead-code-rulings.md DC-9/DC-10]
---

同一个 Java 方法在 game-core 里存在**两套移植面**且无同步机制：`executeScriptInstruction`
既有 scene.rs:1778-2201 的生产主移植体（425 行，a.java:8355-8490 逐值对照，直接改
world/GameView 状态），又有 script.rs 的 `ScriptEngine` + `HostCtx` trait 面
（script.rs:162/121，仅 StubHost/ReplayHost 等测试宿主驱动，**生产路径不走 HostCtx**）；
相机同病——camera.rs `Camera{anchor_x/anchor_y, offset_x/offset_y, target_*}`（camera.rs:18-33）
第二套状态与生产 `GameView.cam_x/ease_cam_x`（scene.rs:545 等）并行。DC-9/DC-10 的
"内联 vs 错桥"结构即双面产物：DC-9 `combat::predict_hp_loss_vs_type` 桥公式
**缺道具二倍加成**（dead-code-rulings.md:413"桥公式缺道具加成（带缺陷），忠实实现=scene.rs:4813"）
= 漂移已发生的实例；DC-10 `Camera::snap_anchor_to_player` 公式由 scene.rs:4400-4401 内联持有。

**为什么：** 双面各自被不同驱动喂养（生产面=world 差分对拍；harness 面=script_golden/
l2_prologue 单测）——一边修了另一边不改时**编译全绿、两边测试也各自绿**，漂移只有跨面
人工对照或差分对拍才能暴露；而"桥"形态（harness 面去引用生产面公式）比"内联"形态更容易
漏字段——DC-9 的道具加成就是这么丢的。与 [pattern_dual-copy-algorithms-sync-fix.md](pattern_dual-copy-algorithms-sync-fix.md)
同族但更重：那是**函数级双拷贝**，这是**模块级双面**（接口形态都不同，肉眼更难对照）。

**何时使用：** 动 execute_script_instruction / 相机 / 寻路等"双面"逻辑前——先 grep 两面
（scene.rs 生产体 + script.rs/camera.rs harness 面），逐字段逐公式对照再改；review 时凡
harness 面（HostCtx 实现、camera.rs、script.rs）新增/修改必问"生产面是否同步、桥公式
来源行号是否注明"；**新建代码禁止再造第三面**——新逻辑只进生产面，harness 面按需桥接；
harness 面的对拍价值保留，但引用其行为做结论时必须标注"非生产路径"。
