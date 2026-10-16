---
name: tryrunscene-fallthrough-arms-misread
description: tryRunScene 的 case 12 属 var3 &= var2 到达臂（a.java:8128-8147），probe(23,12) 探针臂是 case 32 独占（a.java:8194-8200）——误合并致 bump 早 7 拍触发 script12，8f41e6c 修复
type: gotcha
created: 2026-10-16
sources: [8f41e6c]
---

Java `switch` 的 fallthrough 分组必须从**实际 case 标签的堆叠**读归属，不能按
门表达式关键词反推——tryRunScene 的 case 12 曾被误并入 case 32 的探针臂
`!var2 & probeCrossFloorSnapshot(23, 12)`，导致 bump（from_walk=false）即触发
script12，比 Java 早 7 拍（mov_route floor40 T281 帧差，8f41e6c 修复）。

三臂结构（a.java:8038 tryRunScene，逐字核对过）：

- **到达臂**：case 8/9/11/**12**/19/30/41-45/48-50/52/53/55/56 共用
  `var3 &= var2;`（a.java:8128-8147，门在 8146）——只认 from_walk，**走到才触发**；
- **探针臂**：case 32 独占 `var3 &= !var2 & this.probeCrossFloorSnapshot(23, 12);`
  （a.java:8194-8200，门在 8198，前置 `scriptLineFlags[7]` 否决）——只认 bump 且
  目标层有型 12 实体；
- **case 7 是第三种自己的臂**（a.java:8123-8127）：`var3 &= var2 && !probe(23, 12)`
  且置 `scriptLineFlags[32] = true`——到达 + 探针复合，别再并错。

**为什么：** `probeCrossFloorSnapshot(23, 12)` 的 `12` 是**实体类型**（型 12 撞击墙，
probe 在 a.java:8645-8651 按 `entityType[var4] == var2` 扫），与**脚本号** case 12
数字撞名——按表达式里的 "12" grep 归属必然踩坑。反编译 fallthrough 把共体 case
竖排成标签堆叠（case X: case Y: …共享一个 body），归属只能沿标签列读；方法内
还有 case 7/32 两处 probe(23,12)，三处语义各不相同。修这类误读必须用 trace
时序窗口实证（A-trace gameMode 快照 T282=3、T288=11 ⇒ 触发窗口 (282,288]，
全字段差分 FLD 233=12、玩家 (6,8)→(6,7) ⇒ "走到才触发"），不能只看代码"像不像"。

**何时使用：** 移植/审阅任何 Java switch 多 case 共体段（尤其 tryRunScene 门臂、
interactWithCell 类目臂）时——先画"case 标签 → 共享 body"映射表再动手；门表达式
里出现层号/实体类型号/脚本号混排时（23/12 这类）逐个确认参数语义；bump/step
两向门（var2 = from_walk）拿不准就抽 trace 前后拍窗口定归属。
关联坑见 [gotcha_match-catchall-swallows-later-arms.md](gotcha_match-catchall-swallows-later-arms.md)
（Rust match 兜底吞臂——同属"臂归属"家族，方向相反：那是插臂位置错，这是读臂归属错）。
