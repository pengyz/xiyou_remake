# 层号 ↔ maplv 资源号 ↔ 脚本槽号：三套编号关系考证（求解器批0）

- 日期：2026-11-09 ｜ 状态：**已证实（A 级）**，假设项单列 §5 ｜ 产出批次：确定性求解器批0
- 动机：计划 §2.6 指出「GUTS 脚本 index(0..67) 与楼层号(1..54) 是两套编号」，批0 要求
  `SolverInput` 数据结构分字段；本文件给出**可复现的行号证据**与三类关联边。
- 复现：本文件所有分布/计数由 `assets/raw/sprite{0..54}` + `crates/game-core/tests/fixtures/level_scripts.txt`
  一次 python census 得出（记录头 `u32 LE 计数` + 逐记录 `type(1)+x(2)+y(2)+extra`，
  extra 宽度表 = `crates/game-data/src/sprite.rs:54 extra_len`）。

## 1. 结论（三套编号，禁止混用）

| 编号 | 取值域 | 含义 | 真相源 |
|---|---|---|---|
| `floor`（楼层号） | 0..55（运行时 1..54 有资源） | `currentFloor`，**与资源文件号恒等**：`maplv{N}` / `sprite{N}` | `a.java:9413-9416`、`a.java:9455/9497` |
| `script_idx`（脚本槽） | 0..67 | `level_scripts.txt` 行号（68 槽），**与 floor 无恒等关系** | `crates/game-core/tests/fixtures/level_scripts.txt`（`scene.rs:252-253` include_str 载入） |
| `type83.param` | 0..67 | **是 script_idx 不是 floor** | `crates/game-core/src/sprite_spawn.rs:48-51` |

求解器数据结构必须分字段 `floor` 与 `script_idx`（计划 §2.6）；`GUT_n`/`type83.param` 是脚本号，
`LAY_…`/`ADD_<floor>_…`/楼梯 `param&0xff` 才是楼层号。

## 2. floor N ↔ 资源 `maplv{N}`/`sprite{N}`（A 级）

- `reference/src/deobf/a.java:3187-3188`：
  `var35.currentFloor = var35.pendingFloor;` → `var35.loadFloorData(var35.currentFloor);`
- `reference/src/deobf/a.java:3499-3500`（加载链 `case 10`）：
  `var37.loadFloorData(var37.currentFloor);`
- `reference/src/deobf/a.java:9413-9416`：
  `private void loadFloorData(int var1) { int var3 = var1; … getResourceAsStream("maplv" + var3); … }`
- `reference/src/deobf/a.java:9455` `private void m_122(int var1)` → `a.java:9497`
  `getResourceAsStream("sprite" + var3)`；调用点 `a.java:8068 / 8469 / 8662` 传 `currentFloor`
  （另 `a.java:8057 m_122(24)` = `loadLevelScript` case31 的删型4 特例，非换层）。
- **Rust 运行时装载**：`shell/common/src/handoff.rs:97-118`
  `vec![None; 56]` + `enumerate().skip(1).take(54)` ⇒ **槽 0 与槽 55 恒为 `None`，只装 1..54**
  （`maplv{n}`/`sprite{n}` 同一循环读取）。
- **越界**：`crates/game-core/src/scene_persist.rs:362-369` `change_floor` `target<0` 拒、
  `target>55` 拒（注释锚 `a.java:808 floorCountMax=55`）；`:350-359` strict 路径再过
  `[min_floor, max_floor]` 区间门。
- **求解器侧**：批0 `ingest` 装 `maplv0..54` 全 55 槽（census `floors=55`），
  `f0..f50/f52..f54 = 13x13`、`f51 = 12x13`（`maplv51` 非方阵特例，见
  `crates/game-data/src/maplv.rs:31`）。

## 3. `script_idx` 0..67 与三类关联边（A 级）

### 3.0 脚本槽本身

- fixture 68 行，**空槽 = [39, 58, 59, 60]**（非空 64）。
- 上界：`crates/game-core/src/scene.rs:926-929`
  `try_run_scene(idx, …)` → `if idx >= self.dialog.dialog_data.scripts.len() as i32 { return false; }`（=68）。
- `crates/game-core/src/scene_persist.rs:501` `pub fn load_level_script(&mut self, idx: i32)`
  **只置 `current_script_index` + 取脚本行，不换层**（换层只有 `change_floor`）。

### 3.1 边 A：type83 场景门（param = 脚本号）

- 生成：`crates/game-core/src/sprite_spawn.rs:48-51` `83 => { hide_after_spawn = true; e0 }`
  ⇒ **param 单字节 = e0 = 脚本号**。
- 触发两处（同参不同第二参）：
  - `crates/game-core/src/scene.rs:3486-3500` bump 撞格臂，`try_run_scene(param, **false**)`
    → 命中则 `load_level_script(param)`；
  - `crates/game-core/src/scene.rs:4066-4073` 到达/踩格臂，`try_run_scene(param, **true**)`
    → 命中则 `load_level_script(idx)`（注释锚 `a.java:5708-5712`，序章 script 40 入口）。
- **按层分布（实测，48 个不同 param）**：

  | floor | type83 param（脚本号） |
  |---|---|
  | 1 | 19, 66, 67 |
  | 2 | 1, 2, 26, 33 |
  | 3 | 0, 27 |
  | 10 | 3, 4 ｜ 15: 5 ｜ 16: 28 ｜ 20: 6 ｜ 26: 30, 31 ｜ 29: 7, 32 |
  | 32 | 8 ｜ 33: 9 ｜ 36: 29 ｜ 38: 11 ｜ 39: 18 ｜ 40: 12 ｜ 41: 13, 25 ｜ 42: 14 |
  | 35 | 10 ×1, 24 ×3, 34 ×3 |
  | 48 | 38 ｜ 49: 15 ｜ 50: 16 |
  | **51（序章层）** | **40, 41, 42, 43, 44, 45, 46, 47, 48** |
  | 52 | 49, 50, 52, 53 ｜ 53: 56 ｜ 54: 64 |

  param 全集 = `[0..16, 18, 19, 24..34, 38, 40..50, 52, 53, 56, 64, 66, 67]`（48 个）。

  > 【2026-11-09 更正】上表 floor35 行原记「`10 ×3, 34 ×3`」（6 条），系**实测记录错误**：
  > ① `10` 实为 ×1；② 漏记 `24`×3；③ 该行 6 条 ≠ 实测 7 条，且与本行 param 全集含 `24` 自相矛盾。
  > 复核后逐字节复跑 `assets/raw/sprite35`（按 `SpriteTable::parse` 布局：type 1B + x/y 各 2B +
  > type83 extra 1B，`consumed==filesize` 校验通过）得 type83 = `[[10],[24],[24],[24],[34],[34],[34]]`
  > 共 **7** 条 ⇒ 全表 type83 合计 **52** 条、不同 param **48** 个（全集不变）。
  > 原主张保留于本注，不删历史记录（AGENTS §3 证伪更正文化）。

### 3.2 边 B：战斗胜利触发（`crates/game-core/src/scene.rs`）

- `:3806-3819` 型73 → `load_level_script(22)`（`floor==40` 时再 `spawn_boss_event(6)`）、
  型74 → `load_level_script(21)`、型75 → `try_run_scene(20, true)` + `load_level_script(20)`；
  实测分布：型73@floor40/42、型74@floor20/53、型75@floor10/53。
- `:3880-3895` 型69 且 `floor==35` → `load_level_script(36)`；
  型70/71 且 `floor==49` → `load_level_script(35)`；
  型72 且 `floor==50` → `load_level_script(65)`。

### 3.3 边 C：脚本内跳转（fixture 实测）

- `GUT_n`（`crates/game-core/src/script_ops.rs:167 op_gut` → `load_level_script(target)`，
  注释锚 `a.java:8308-8311`）目标集 = **{1, 12, 37, 57, 61, 62, 63}**——含 57/61/62/63，
  **直接证明脚本号可 >54，与 floor 不同域**。
- `LAY_2`（仅 slot0 一条；`script_ops.rs:68 op_lay` = `save_floor_state` + `set_pending_floor`）
  与 `ADD_2_72_11_10`（仅 slot7 一条）中的 `2` 才是**楼层号**。
- 楼梯（type7 上 / type8 下，`param & 0xff` = 目标层）：**floor50 是唯一无楼梯的层**；
  **floor0 与 floor44 从未被任何楼梯指向**（floor44 出边 (7→45),(8→43)；floor0 出边 (7→1)；
  floor51 出边 (7→52)；floor54 出边 (7→1),(8→53)）。

## 4. 入口完备性（脚本槽被谁引用）

- 被引用槽 = type83 param(48) ∪ GUT 目标(7) ∪ 战斗触发(6) = **59**；非空槽 64。
- **非空但无显式入口的槽 = [17, 23, 51, 54, 55]**（引用到的槽里没有空槽）。
  ⇒ 假设（C 级，待考）：间接入口（脚本内条件链/未移植臂）或死脚本。

## 5. 假设 / 待考（不得当结论用）

1. **floor0 在 Rust 运行时的可达性**：`a.java:3189-3190` `currentFloor == 0` 有专门落点
   `placePlayerAtCell(1, 2)`（Java 侧 floor0 是有效层），但 `handoff.rs:97-98` 槽 0 恒 `None`。
   求解器批0 装满 55 槽（含 maplv0），运行时侧是否可达 → 待考。
2. 无显式入口的脚本槽 `[17, 23, 51, 54, 55]` 的真实触发路径 → 待考。
3. `type73/74/75` 的 `param` 语义（73 有 1 字节 extra=1，74/75 无 extra）未纳入本结论。

## 6. 对求解器批0 的直接含义

- `SolverInput.floors[i]` 的下标即 floor 号（i = 0..54），与 `maplv{i}` 同源；
- 读 type83 `param` 时必须按 **script_idx** 解释，写入 `script_idx` 字段；
- `GUT_n` 目标同为 script_idx；`LAY`/`ADD`/楼梯 param 才写 `floor` 字段；
- 三类边是批1+「场景图/可达性」查询的全部入边，缺一边会漏可达层。
