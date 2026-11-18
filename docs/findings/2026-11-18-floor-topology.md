# 层间拓扑（P0）：换层三类通道与可走范围

> 目的：witness-first 攻坚的第一步——先知道「主线怎么从起点走到终局」，再谈逐层求解。
> 产物：本文件 + [`data/tables/floor_transitions.json`](../../data/tables/floor_transitions.json) + `crates/game-solver/src/floorgraph.rs`。

## 一、机制（A 级，引擎同源）

| 通道 | 规则 | 锚点 |
|---|---|---|
| 楼梯 ty7（上）/ ty8（下） | 目标层 = `param & 0xff`；踩上即 `change_floor(target, went_down, false)` | `crates/game-core/src/scene.rs:3966-3978`（a.java:5660/5666） |
| 脚本 `LAY_x` | `op_lay` 写 `pending_floor=x` 并进入遮幅相位 5 | `crates/game-core/src/script_ops.rs:67-74`（a.java:8355） |
| 脚本门 ty83 | `param` = 脚本槽号；踩格（`from_walk=true`）或从邻格撞（`false`）触发 | `crates/game-core/src/scene.rs:3959-3967`、`crates/game-core/src/rules/gate.rs` |
| 道具 21 / 22 | 上行符 `change_floor(floor+1)`、下行符 `change_floor(floor-1)` | `crates/game-core/src/item_effect.rs:140-150` |

`LAY` 只出现在 **0 号槽**（`LAY_2`）——序章把玩家从 f51 放到主塔起点 f2 附近。
其余换层全靠楼梯与道具，这让静态图**基本完整**：楼梯边是常量数据，不需要跑引擎就能算。

## 二、数据结构与命令

- `crates/game-solver/src/floorgraph.rs`：`EdgeKind{StairUp,StairDown,Script,ItemUp,ItemDown}`、`FloorEdge{from,cell,to,kind,detail}`、`graph()`、`reachable_by_stairs()`、`lay_targets()`。
- `./target/release/solver floors [--out PATH]`：打印每层边、每层门槽号、可达层清单；`--out` 落 JSON。
- 判据：`fg_fixture_lay_slot0_goes_to_floor2`、`fg_stair_target_is_param_low_byte`、`fg_script_edge_from_gate`。

## 三、实测

```
105 条楼梯/脚本边；含道具边共 213 条；54 层；52 扇 ty83 门
从 f51 可达 53 层
不可达层（仅楼梯/脚本口径）: 0, 44
```

**塔的顺序**（与「序章在顶层 f51」一致）：

```
f51 --StairUp(2,2)--> f52 --StairUp(11,11)--> f53 --> f54
f54 --StairUp(7,1)--> f1
f1 .. f24 逐层上行（每层一上一下）
f24 --StairUp(6,1)--> f50      ← 终局层入口
```

- **f50 没有任何楼梯边**，只能从 f24 (6,1) 上去；进入后唯一一扇门在 `(6,5)`，槽号 **16**（`TAK_123_126 GUT_37`）。
- **f51 有 9 扇门**（槽 40..48），是序章分支；起点 `(1,11)` 那扇就是槽 40。
- **f49 只有一条下行梯**（`(1,11)→f48`），是支线末端而非主线。
- **f0** 无楼梯进入 ⇒ 只能靠 item22 下行符（已在前批取证：`f41` 门 25 → 槽 25 → `CES_22_6_6`）。
- **f44 没有任何楼层有楼梯通到它**（自己只有 `(1,1)→45` 上行、`(1,11)→43` 下行）⇒ 只能靠道具 21/22 进出的支线层。这是「楼梯口径不可达」的第二例，不是解析缺失。

## 四、对 witness 的直接含义

1. 主线是**一条链**，没有分叉选择：`f51 → f52 → f53 → f54 → f1 → … → f24 → f50`。所以「能否通关」= 这条链上**每一层都能带着还能活的入场状态出层**。
2. 支线（f0 幸运币、f44、f37 仓库、f41 下行符…）不是通关必需，但**可能决定血量是否够**——留给 P4 的「资源支线」钩子按需触发，而不是全图搜。
3. 终局链还差最后一环的实证：f50 门 16 之后，ty72 由哪条臂召出、`load_level_script(65)` 何时执行。**P5 的回放测试就是为此设的判据**，不在文档里猜。

## 五、未决 / 下一步

- P1：把 `solve_global` 的 `goals.is_empty() ⇒ "feasible"` 换成 `status{exhausted|capped|witness|model_broken}` + `blocked` 诊断（当前口径会谎报）。
- P2：boss 掉落数据化（f15 金勺子 / f35 芭蕉扇 / f53 捆仙绳 决定后期能力）。
- P3/P4/P5：单层「入场→出层」接口 + 逐层接力 + 真引擎回放。
