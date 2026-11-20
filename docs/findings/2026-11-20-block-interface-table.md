# 54 层块合并与接口导出表（分层收缩搜索 S1 片）

> 任务书：`docs/superpowers/proposals/2026-11-20-hierarchical-solver-proposal.md` §7 S1。
> 产出：本文件（方法/口径/逐层摘要/异常/假设）+ 机器可读
> [block-interfaces.json](block-interfaces.json)（schema `xiyou/block-interfaces/v1`）。
> 复现脚本：`/tmp/block_iface.py`（一次性，不入库）；
> `python3 /tmp/block_iface.py` 即可全量重算（依赖 `assets/raw`、`data/tables`、fixtures、`/tmp/keydemand.json`）。

## 1. 方法与口径（全部复用既有引擎语义，零自造）

### 1.1 数据源与解析（A 级格式，已有取证）

| 数据 | 解析 | 证据锚 |
|---|---|---|
| 地形 | `assets/raw/maplv{n}`：u16 raw_width(=真实宽×2)/u16 raw_height，网格1 tile 在 `idx=y*width*4+x*2`，`walk = tile_walkability[tile]==1` | `crates/game-solver/src/solve_local.rs:1096-1108` |
| 实体 | `assets/raw/sprite{n}`：u32 count + (u8 ty, u16 x, u16 y, extra)；格坐标 `x>>5, y>>5` | `docs/findings/resource-formats.md`；`crates/game-solver/src/ingest.rs` |
| 实体参数 | `spawn_param_and_visibility`（7/8 目标层 low byte；4/5/81/57/59/70/71/72/73 = e0+1；77/78 组号+门表；83 = script slot） | `crates/game-core/src/sprite_spawn.rs:23` |
| 分类 | `render_category`（sprite_defs.json 89 行）+ 层覆写（仅 ty72@f50→cat8） | `crates/game-core/src/subsystem/script_host.rs:138`、`crates/game-core/src/resource_cfg.rs:314-322` |
| 脚本 | `crates/game-core/tests/fixtures/level_scripts.txt` 68 槽；`LAY_` 前缀 → lay_targets（现仅 slot 0 → LAY_2） | `crates/game-solver/src/floorgraph.rs` |
| 门表 | `door12.txt`/`door13.txt` | `crates/game-solver/src/bin/solver.rs:81` |
| boss 掉落 | EVENT_SPAWNS 8 组 + `event_of`/`post_victory_event_of`（floor≤50；73@40→组6；69@35→组4+7） | `crates/game-core/src/rules/boss.rs:9-131`（A 级 a.java:2041-2197） |

### 1.2 块合并（零钥匙连通分量）

- **可走格** = 地形可走 ∥ 格内 ty6/7/8 豁免（scene.rs `stair_count` 两段式终审；
  keydemand 同口径 cat1 6/7/8 可通行，`docs/findings/2026-11-20-key-demand.md` §1）。
- **割点（阻断格）**：ty1/2/3 门（耗钥，`crates/game-core/src/enums.rs:185-191`：1=黄/2=红/3=蓝，26/27/28 钥）、
  ty4/81 封印门（杀敌/脚本前置，keydemand 按 `gates_ignored` 忽略，本表**保守按割点**——见 §5 偏差）、
  ty83 脚本门（`try_run_scene` 前置，`crates/game-solver/src/dynreach.rs` 的 flags 模型）、
  ty77/78 商店（cat16 硬阻挡）、ty9/10/11/12 与 cat32（84/87、非50 的 72）硬阻挡。
- **怪（cat8 = 41..75，+72@f50）按可通行参与连通**（战斗恒可能，不耗钥）；
  ty76 是 cat1 障碍（`sprite_defs.json` render_category=1）不属怪。
- **必杀怪** = 该怪格是割点：去掉它（其余全通）后其可走邻格落入 ≥2 个连通分量（无绕行）。
  全塔仅 2 格：f51 ty41@(7,10)、f52 ty45@(11,7)。
- **归属**：阻断格并入其邻接分量（入场侧优先：入口=ty8 首槽，keydemand 同口径；否则取最小分量号）；
  门/脚本门常成「门+门」链（如 f40 红 door2@(6,8)→gate83@(6,7) 串联），割点标注按**阻断格簇**整体判：
  簇的邻接分量 ≥2 时，向每个非归属侧分量标注簇内每个门/脚本门。
- **导出面三分类**（用户裁定）：
  - **常数类**：29-32 丹（拾取即生效，+floor_tier(所在层)）、33-40/79-80 装备（tier 固定，effect=EQUIP，
    `data/tables/items.json`）、钥匙 26/27/28、工具 13/14/15、被动 23/24（effect key 无时点依赖）。
  - **时点类**：商店 ty77/78（价格/效果查 `data/tables/shop_goods.json`，应用逻辑
    `crates/game-core/src/scene.rs:2522 apply_merchant_offer`；id6 StatPct 免费可重复
    `atk += atk*pct/100` 复利 scene.rs:2568）、item19 千年月桂露（Heal ×74/10，时点）、item25 幸运币（仅 f0 静态生成）。
  - **拓扑类**：effect key ∈ {RemoveAdjacent, RemoveAllType, ChangeFloorUp, ChangeFloorDown, MirrorTeleport, KillAdjacent4}
    —— 21 上行符/22 下行符/16 芭蕉扇(对 ty10)/17 金勺子/18 玄明石(对 ty11)/85 黄金钥匙(RemoveAllType target=1)/86 日月无极钹(KillAdjacent4)。
    **说明**：裁定原文只列 21/22/17/18/16；85/86 按 effect key 同判拓扑（`crates/game-core/src/effect_registry.rs` 同族算子），多出 2 条在 f37/f46，见 §4。
- **出口**：ty7/ty8 楼梯（`param&0xff` 目标层，floorgraph 同口径 `crates/game-solver/src/floorgraph.rs`）、
  ty83 带LAY 的脚本边（现仅 f3 slot0→f2）、boss 击杀触发（`boss.rs::event_of`/`post_victory_event_of`，drops 已转录进 JSON）。

## 2. 一致性判据自验（脚本输出，全 PASS）

| # | 判据 | 结果 |
|---|---|---|
| 1 | 每层块内 cells 之和 = 可走格总数（楼梯/火豁免计入；孤立阻断格单列成块保持守恒） | ✅ 55/55 层 |
| 2 | 每个拾取实体（13..40/79..86）、商店（77/78）、门（1/2/3）恰好归属一个块 | ✅ 无多归属（割点标注不计归属） |
| 3 | 逐层割点门数(按色) ≥ keydemand needed：**54/54 层无违例**（f4 cut d1:7≥2、f46 cut d1:8≥3） | ✅ |
| 4 | 54 层全跑完（另含 f0 隐藏层）；f0/f1/f44/f49/f50 单独说明 | ✅ 见 §4 |
| 5 | 与 floorgraph 对账：`cargo run --release -p game-solver -- floors` 输出 **105 条边**，与本表 f1..54 的 104 楼梯 + 1 脚本边**逐条相等**（集合 diff 为空）；f0 的 StairUp(1,1)→f1 不在 solver 建图范围 | ✅ |

复现：`python3 /tmp/block_iface.py`（判据 3 需先 `cargo run --release -p game-solver -- keydemand --out /tmp/keydemand.json`）。

## 3. 全塔统计

- 块数：f1..54 共 **867 块**，每层 3..33 块；分布 `{3:1, 6:3, 8:2, 9:3, 10:5, 13:7, 14:2, 15:2, 16:2, 17:4, 18:3, 19:5, 20:2, 21:5, 22:4, 26:1, 31:1, 32:1, 33:1}`。块最多：f41(33)、f34(32)、f26(32)。
- 割点门（去重格数）：**黄 door1=231、蓝 door3=42、红 door2=17、封印 ty4=33、封印 ty81=5**（标注次数 428，一门分隔 k 块记 k−1 次）；脚本门割点格 33。
- 必杀怪：**2 格 4 次标注**（f51 ty41、f52 ty45）。
- 导出面：**常数类 448、时点类 30（全部是商店 18×77 + 12×78）、拓扑类 3**（f45 item21 上行符@(2,10)、f46 item85 黄金钥匙@(3,8)、f37 item86 日月无极钹@(5,5)）。静态 sprite 塔内无 item16/17/18/19/22/25——它们经 boss 掉落（16@f35、17@f15 击杀67、24+27@f50）或脚本/商店进场，已记入对应块的 boss_kill exits.drops。
- 与提案 §2 预期偏差：提案预期「割点数适中、导出面三分类可枚举」成立；偏差两条见 §4 末。

## 4. 逐层摘要

割点列 `d1/d2/d3`=黄/红/蓝门，`s4/s81`=封印门，`g`=脚本门（均为标注次数）；导出面列为三分类计数。

| 层 | 块数 | 割点 | 必杀怪 | 常数 | 时点 | 拓扑 |
|---|---|---|---|---|---|---|
| f1 | 16 | d1:7 | 0 | 13 | 0 | 0 |
| f2 | 19 | d3:1 g:2 s4:6 | 0 | 6 | 1 | 0 |
| f3 | 22 | d1:5 g:3 | 0 | 18 | 1 | 0 |
| f4 | 17 | d1:7 d3:1 | 0 | 10 | 1 | 0 |
| f5 | 18 | d1:5 | 0 | 12 | 0 | 0 |
| f6 | 13 | d1:8 | 0 | 10 | 2 | 0 |
| f7 | 17 | d1:9 d3:1 | 0 | 11 | 1 | 0 |
| f8 | 18 | d1:9 d3:1 s4:1 | 0 | 16 | 0 | 0 |
| f9 | 20 | d1:11 d3:2 | 0 | 13 | 0 | 0 |
| f10 | 22 | d1:4 d2:1 g:2 s4:2 s81:2 | 0 | 3 | 0 | 0 |
| f11 | 13 | d1:7 d3:1 s4:1 | 0 | 12 | 0 | 0 |
| f12 | 18 | d1:6 d3:1 | 0 | 13 | 2 | 0 |
| f13 | 10 | d1:1 | 0 | 1 | 0 | 0 |
| f14 | 21 | d1:9 d3:2 s4:1 | 0 | 12 | 0 | 0 |
| f15 | 15 | d1:6 s4:1 | 0 | 5 | 1 | 0 |
| f16 | 17 | d1:8 d3:1 | 0 | 13 | 1 | 0 |
| f17 | 13 | d1:2 d3:1 s4:4 | 0 | 8 | 0 | 0 |
| f18 | 19 | d1:8 d3:2 | 0 | 14 | 1 | 0 |
| f19 | 19 | d1:6 d3:1 | 0 | 11 | 0 | 0 |
| f20 | 16 | d1:2 d2:1 d3:2 s4:1 | 0 | 8 | 0 | 0 |
| f21 | 6 | d1:4 | 0 | 4 | 1 | 0 |
| f22 | 6 | — | 0 | 0 | 0 | 0 |
| f23 | 6 | d3:1 | 0 | 2 | 1 | 0 |
| f24 | 10 | — | 0 | 0 | 0 | 0 |
| f25 | 20 | d2:7 | 0 | 9 | 0 | 0 |
| f26 | 32 | d2:9 g:6 | 0 | 0 | 0 | 0 |
| f27 | 9 | d1:1 | 0 | 2 | 1 | 0 |
| f28 | 8 | d3:4 | 0 | 5 | 1 | 0 |
| f29 | 9 | g:1 | 0 | 0 | 0 | 0 |
| f30 | 10 | s4:1 | 0 | 0 | 0 | 0 |
| f31 | 17 | d1:8 | 0 | 14 | 2 | 0 |
| f32 | 19 | d1:5 d3:1 g:2 s4:1 | 0 | 14 | 0 | 0 |
| f33 | 22 | d1:6 d3:2 g:1 s81:2 | 0 | 7 | 1 | 0 |
| f34 | 32 | d1:32 s4:18 | 0 | 16 | 0 | 0 |
| f35 | 8 | g:6 s4:1 | 0 | 0 | 0 | 0 |
| f36 | 13 | d1:4 | 0 | 2 | 1 | 0 |
| f37 | 10 | d1:2 d3:2 | 0 | 33 | 1 | 1 |
| f38 | 22 | d1:6 d2:1 d3:3 g:1 s4:1 s81:1 | 0 | 9 | 1 | 0 |
| f39 | 14 | d1:4 d3:1 | 0 | 5 | 2 | 0 |
| f40 | 14 | d1:1 d2:1 d3:1 g:1 | 0 | 5 | 0 | 0 |
| f41 | 33 | d1:22 d3:2 | 0 | 16 | 0 | 0 |
| f42 | 13 | d1:4 g:1 | 0 | 19 | 1 | 0 |
| f43 | 21 | d1:8 d3:1 | 0 | 8 | 0 | 0 |
| f44 | 10 | s4:1 | 0 | 5 | 0 | 0 |
| f45 | 21 | d1:6 d2:1 s4:2 | 0 | 6 | 2 | 1 |
| f46 | 21 | d1:8 d3:2 | 0 | 3 | 1 | 1 |
| f47 | 19 | d1:7 d3:2 | 0 | 10 | 1 | 0 |
| f48 | 15 | d1:2 d3:3 s4:1 | 0 | 9 | 1 | 0 |
| f49 | 13 | d2:1 g:3 s4:6 | 0 | 0 | 0 | 0 |
| f50 | 3 | — | 0 | 0 | 0 | 0 |
| f51 | 26 | d1:2 d2:1 d3:2 g:12 s4:2 | 2 | 7 | 0 | 0 |
| f52 | 21 | d1:2 g:5 | 2 | 9 | 1 | 0 |
| f53 | 13 | g:1 | 0 | 2 | 0 | 0 |
| f54 | 9 | g:1 | 0 | 18 | 0 | 0 |

### 特殊层说明

- **f0（隐藏/调试层）**：不建图（`crates/game-solver/src/bin/solver.rs` 只 load floors 1..=54）。含 129 可走格、12 块、静态 item25@(6,5)、StairUp(1,1)→f1（floorgraph 105 条不含此边）。JSON 内保留供取证。
- **f1 / f51（无入口 no_entry）**：无 ty8。f1 经 GLOBAL_ENTRY=(1,11) 顶格进（solver.rs），f51 是序章（GLOBAL_START_FLOOR=51，经 slot0 LAY_2 脚本边 → f2 开局）。
- **f49 / f50（无上行 no_up_exit）**：无 ty7。f50 终局 3 块 14 可走格；f49 仍有门/脚本门结构供支线。
- **f44**：结构正常（有 ty8 入口 [1,11] + ty7），与 f1/f49/f50 不同，无需特判；任务书列它作「单独说明」核对项，此处确认无异常。
- **f29**：本表 status=ok（9 块，1 脚本门）；keydemand 报 `unreachable` 是其**硬阻挡按永久不可通行**的保守假设所致（key-demand.md §2 假设5），本表将 ty4/81/83 按割点建模后不存在该死锁，属口径差异非数据矛盾。

### 与提案 §2 预期的偏差（2 条）

1. **导出面时点类几乎只有商店**：静态塔内无 item19/25/16/17/18/22 的拾取格（时点/拓扑道具经 boss 掉落与脚本进场），拓扑类静态仅 3 格；分层收缩若按「每块导出面枚举」设计，时点/拓扑面主要挂在 boss_kill exits 的 drops 与脚本门 spawns 上。
2. **割点远多于钥匙需求**：全塔 328 个门割点 vs keydemand needed 最大 3；大量门只分隔支线宝箱区，非主通路。分层收缩应以「入口块→上行梯块」的主块序为主轴，支线块割点作界内枚举，不能拿割点总数当分支因子。

## 5. 异常清单（A/B 级，均可复现）

1. **35 处孤立阻断格**（无邻接可走块，单列成 singleton 块）：f13 hard@(6,5)；f24 sealed4@(6,1)(6,2)；f25 door2@(1,7)(11,7)；f26 25 处（含 gate83@(6,7)、door2@(6,8)(6,9)(6,10)，其余 hard=cat32/9-12）；f36 hard@(9,8)（同格挂 gate83 slot29，无 LAY）；f38 door3@(5,6)；f41 door1×5。含义：这些门/门后区域与任何可走块都不邻接（被硬地形包死），引擎上不可达，求解器可整块剪掉。
2. **f40 红门-脚本门链**：door2@(6,8) 唯一上方邻格是 gate83@(6,7)（slot12，无 LAY），簇整体分隔上下两块——keydemand needed=(y1,b0,r1) 与本表簇割点 (d1:1,d2:1,d3:1,g:1) 一致（判据3 曾在此抓出按单格标注漏计红门的 bug，已改簇口径）。
3. **f34/f41 密集门阵**：f34 32 处 d1 标注 + 18 处 s4（keydemand 只遇 14 门——其 BFS 空间被入口 [1,1] 侧的封印门/硬阻挡截断）；f41 22 处 d1（keydemand 报 21，差 1 为孤立 door1）。不影响判据 3（需求侧）。
4. **ty83 总量**：sprite 层 52 个（f1..54），本表 exits 内 49（含 1 个 LAY 边）+ 孤立/割点标注若干；slot0 是唯一 LAY 槽。gate66（永不开）、gate16/67（难度条件）等动态前置只在 firing 判定层起作用（dynreach.rs），本表一律按「脚本门割点」保守处理。
5. **f0 隐藏层**：见 §4。

## 6. 假设区（C 级，未混入结论）

1. ty4/81 封印门按割点（保守）是本任务书「必开门」的解读；若 S2 采用 keydemand 的 `gates_ignored`（乐观）口径，块数会减少（尤其 f2/f34/f49 的 s4 簇），需在 S2 明确选边。
2. 85/86 归拓扑类是按 effect key 同族算子的扩展（裁定原文未列）；若 S2 只按原裁定四件套（21/22/17/18/16）做拓扑枚举，f37/f46 这两格退化成常数拾取。
3. 「必杀怪」按纯拓扑割点定义（任务书原文）；未叠加「打不过」属性过滤——保守基线（300hp/atk10/def10）的 predict 损血属战斗代价层（S3），本表只在 JSON 的 cut_monsters 里给了 ty 供下片查 `CombatTables`。
4. 孤立阻断格「引擎上不可达」基于静态地形+实体；若脚本 CES_ 在运行时于其邻格生成 ty6/7/8 豁免格（如 f26 gate83@31 的 spawns），可达性会变——本表已把每个脚本门的 spawns 列表写入 JSON，S2 可按需重放。
