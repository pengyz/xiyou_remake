# f41 双门链的可达性核查 + changeFloor 落点规则（2026-11-18）

## 一、changeFloor 落点（A 级，`crates/game-core/src/scene_persist.rs`）

`change_floor(target, down, strict)`（:348）只做四件事：层界/负层/超 55 提示、`save_floor_state()`（m_119 存活实体序列化）、
`set_transitioning(true)`、写 `pending_floor/went_down`。**它不直接换图**——真正换层在遮幅中段 `swap_floor()`：

| 目标层条件 | 落点 | 出处 |
|---|---|---|
| `floor == 0` | `place_player_at_cell(1, 2)` | `scene_persist.rs` swap_floor 首分支 |
| `floor == 50` | `place_player_at_cell(6, 7)` | 同上 |
| `floor == 1 && !went_down` | `place_player_at_cell(6, 11)` | 同上 |
| 其余 | `find_floor_gate(went_down)` → `step_off_stairs(gx, gy)` | 同上 |

⇒ **f0 的到场点是 (1,2)**，不是上行梯所在的 (1,1)。f0 只有两个实体：上行梯 `ty7@(1,1)`、
幸运币 `ty25@(6,5)` ⇒ 用下行符从 f1 下到 f0 后，玩家落在 (1,2)，走到 (6,5) 取币，
再从 (1,1) 上行梯回 f1。这条路线与「f0 仅下行符可达」一致。

## 二、f41 两处火能否清掉：**不能**（A 级，实测）

`fire_trap_climb_up(x, y-1, idx)`（`crates/game-core/src/scene.rs:3941`）的三个前置：
上格 `walkable_at(x, y-1)` ∧ 上格**无槽** ∧ 上格不是玩家格。只有玩家站在火的正下方（火在上邻）时才调用。

f41 实测（真实 `maplv41` + `sprite41`，经 `GameScene::load_floor` + `walkable_at`）：

| 火位置 | 上格 | 上格可走 | 结论 |
|---|---|---|---|
| `ty60@(2,2)`（静态） | (2,1) | **false** | 火爬不走，永久留在 (2,2) |
| `ty60@(10,2)`（脚本 13 召出） | (10,1) | **false** | 火爬不走，永久留在 (10,2) |

⇒ 门 25 的三重前提（`flags[13]` ∧ `(2,2)` 无 ty60 ∧ `(10,2)` 无 ty60）里，后两条**靠现有机制清不掉**。

## 三、更严重：门 25 所在格本身不可走

f41 的可走性图（`.` 可走 / `#` 不可走，13×13，80 格可走）：

```
#############
#.#..#.#..#.#
#....#.#..#.#   ← (10,2) = '#'
#.#.##.##.#.#
...
```

- `(10,2)`：`cell_type = 51`，`walkable = false`；两扇 `ty83`（param 13 / 25）就摆在这个**墙格**上。
- 门 25 的门控要求 `from_walk == true`（必须**走进去**才算），撞格路径传的是 `false`
  （`scene.rs:3502` 注释：「bump 侧第二参 = false —— script3 类 `gate &= from_walk` 门控使 bump 不触发、到达才触发」）。

⇒ **「走进去」与「该格不可走」直接矛盾**。这与「玩家在原版里确实拿到过下行符」相冲突，说明模型里还有一个未建模的机制
（候选：破墙/地形变更、脚本 MOV 把玩家搬到该格、或 `walkable` 表在开门后被重建）。

## 四、对已提交测试的更正要求

`734fc23` 的链式测试把玩家**直接放置**在 (10,2) 再跑踩格副作用——它验证的是**门控逻辑**，
不是「玩家实际能到那里」。已在测试文件里加前提警示，并新增 `f41_gate_cell_not_walkable_open_question`
显式钉住上述矛盾，防止被读成「f41 下行符已确认可得」。

## 五、待办（按优先级）

1. 找 f41 `(10,2)` 的可达机制：查 `walkable` 表是否可被脚本/道具改写（`rebuildWalkability` 调用点）、
   f41 是否有 MOV/破墙脚本路径能落到该格。
2. 若确证不可达：门 25 只能靠别的路径满足，或下行符另有来源 —— 需重新枚举 22 的全部来源
   （当前结论是「唯一来源 = 脚本槽 25」）。
3. 求解器侧：`doors` 的 `passable` 与 `solve_local` 的 `stair_like` 口径仍需单源（既有 F1 项）。
