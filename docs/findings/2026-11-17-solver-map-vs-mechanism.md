# 求解器里的「机制」与「当前题面」必须分开

> 触发：用户质疑「资源约束和当前地图/题面设计绑在一起，将来做随机地图这条规则就变了，
> 把它固化到求解器合理吗？」——合理的是**判据**，不合理的是把**这一张图的事实**写成代码。

## 1. 三层划分

| 层 | 例子 | 能进代码吗 | 单一来源 |
|---|---|---|---|
| **机制** | 门要钥匙、楼梯上下、地形可走、击杀赏金公式 | 能，且必须唯一 | `crates/game-core`（`enums`/`combat`/`effect_registry`） |
| **判据** | 「封住这扇门能否走到出发梯」「累计供需是否断供」 | 能，写成对输入的函数 | `crates/game-solver`（`doors`/`keyflow`） |
| **题面事实** | 入口 (1,11)、起点 51 层、红门在 f10/f20/f40/f51、必经 33 扇 | **不能**，只入 findings/产物 | `docs/findings/*.md`、JSON |

## 2. 已做的清口（2026-11-17）

1. **入口常量隔离 + 可覆盖**：`crates/game-solver/src/bin/solver.rs` 的 `GLOBAL_ENTRY`/`GLOBAL_START_FLOOR`
   提为带出处的常量，并新增 `--entry X,Y` / `--start-floor N`。随机地图不再需要改代码。
2. **出口判定不再静默回落**：`doors::entry_and_goals` 返回 `Option`；新增 `FloorStatus{Ok,NoExit,BaselineUnreachable}`。
   没有出发型出口的层（f49：换层靠脚本）报 `NoExit`，不再伪装成「基线不可达」；
   `keyflow::FlowReport::no_exit_floors` 把这类层显式带进结论。
3. **机制常量归一**：`crates/game-core/src/enums.rs` 新增 `DOOR_YELLOW/RED/BLUE`、`KEY_YELLOW/RED/BLUE`；
   `doors.rs`/`keyflow.rs` 不再有 `1/2/3`、`26/27/28` 字面量。

## 3. 仍未做（明确记录，不假装已清）

- **同层「先拾取后开门」是建模假设**，不是机制：随机地图可能有「钥匙在门后」的布局。
  计划：把次序提成 `keyflow::PickupOrder` 参数，并在报告里打印所用次序；默认值保持不变。
- 怪的不可通过性仍不入门判定（见 `mandatory-doors.md` §4），需与 `blocking_kill_slots` 交叉核对。

## 4. 规矩（对后续所有求解器改动生效）

> 求解器里允许出现的常量只有**机制**，且必须来自 `game-core`；
> 图与题面的差异一律走输入；每个分析必须**声明前提**，前提不成立时**报错或显式标记**，
> 不许回落成某个看似合理的结论。
