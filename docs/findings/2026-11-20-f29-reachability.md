# f29 可达性取证：为什么 `solver keydemand` 判 `unreachable`（2026-11-20）

> 只读取证。未改任何代码。实验脚本在 `/tmp/sprite29.py`、`/tmp/map29.py`、`/tmp/walk29.py`、
> `/tmp/f29_experiment.py`（不在仓库内）。复现命令全部可独立重跑。
> 证据分级沿用 `AGENTS.md` §3（A=字节级，B=代码语境推断，C=猜测）。

## TL;DR

**f29 的 `unreachable` 与钥匙无关，实现者的说法属实，且比他说的更强：**
f29 的入场格 (3,11) 到上行梯 (8,11) 的**唯一地形通路**是一条「左列上行 → 顶层横走廊 → 右列下行」
的环线；这条环线被两个实体**各切断一次**——`ty72` 联动门在 (3,2) 切左列、`ty11` 墙在 (8,3) 切右列。
**单独放开任何一个都不够，必须两个都放开才可达**（放开后 `needed=(0,0,0)`，因为本层没有任何
ty1/2/3 门和钥匙）。keydemand 的乐观模型（假设 2/3）本身无误，`unreachable` 是假设 5
（硬阻挡按永久不可通行）在「硬阻挡恰好处在唯一必经格」时的正确产物——但它是**建模假设下的
最坏情形结论**，不是「原版玩家到不了 f30」。

## 1. 四个问题的答案

### Q1：入口格与出口格的地形可走性（maplv29，A 级）

`maplv29` 头 4 字节 `1a 00 1a 00`；两张 676B 网格（13 列 × 26 行 × 2B），第一张为地形，
可走性判据 = `crates/game-core/src/scene_persist.rs:297-301`（`idx = y*cols*4 + x*2`，
`tile_walk[tile]==1`，表 = `crates/game-core/tests/fixtures/tile_walkability.txt`，104 项）。

| 格 | tile | walkable | 角色 |
|---|---|---|---|
| (3,11) | 0x19 (25) | 1 | 入场格（ty8 下行梯所在，→f28，param&0xff=28） |
| (8,11) | 0x01 (1)  | 1 | 上行梯（ty7，→f30，param&0xff=30） |

复现：`python3 /tmp/map29.py`（读 `assets/raw/maplv29` 逐格打印）。
**地形本身连通**：纯地形 BFS 从 (3,11) 可达 (8,11)（`python3 /tmp/walk29.py`，输出
`exit (8,11) reached: True`）。地形不是阻断源。

### Q2：`solver floors` 说 f29 可达，与 keydemand 矛盾吗？（B 级）

**不矛盾，两者回答不同的问题。** `crates/game-solver/src/floorgraph.rs:138-139`：
`reachable_by_stairs` 的文档原文「只沿**楼梯与脚本**边从 `start` 可达的**层集合**」——
它是**层间图的节点可达性**，不做层内走格 BFS（全文件不消费 `terrain`/实体阻挡，
`grep terrain floorgraph.rs` 仅 1 处且是测试夹具 `terrain: vec![true; 169]`，行 166）。
keydemand 回答的是「**本层内**从入场格走到上行梯」。f29 作为节点可达（f28→f29→f30 楼梯链），
但层内入口→出口走不通。**必有一错的前提不成立，两实现各对一半。**

复现：`./target/release/solver floors 2>&1 | grep '^f29 边'`
→ `f29 边 2: StairDown@(3,11)→f28  StairUp@(8,11)→f30`。

### Q3：`unreachable` 的真实原因（A 级实体数据 + B 级类目语义）

`assets/raw/sprite29` 全量解析（`count = u16@0 | u16@2<<16` = **6** 条，文件 41B 全部消费，
复现 `python3 /tmp/sprite29.py`；格坐标 = 像素 `>>5`，判据
`crates/game-solver/src/solve_local.rs:1131` `(sp.record.x >> 5, sp.record.y >> 5)`）：

| # | ty | 像素 | 格 | param | 类目 | keydemand 处理 | 语义 |
|---|---|---|---|---|---|---|---|
| 0 | 11 | (256,96) | **(8,3)** | — | cat1→硬阻挡臂 `9\|10\|11\|12`（keydemand.rs:317-321） | `blocked=true` | **墙**（引擎可被道具/剧情消除，但 keydemand 假设 5 按永久） |
| 1 | 8  | (96,352) | (3,11) | 284=0x11C | cat1 楼梯臂 | `entry` | 下行梯 → f28（param&0xff=28） |
| 2 | 7  | (256,352) | (8,11) | 286=0x11E | cat1 楼梯臂 | `goal` | 上行梯 → f30（param&0xff=30） |
| 3 | 83 | (96,320) | (3,10) | **7** | cat1 假设 2 | `gates_ignored`，放行 | 场景门 → 脚本槽 7；**槽 7 是空行**（level_scripts.txt 第 8 行，0 LAY），触发即空跑 |
| 4 | 72 | (96,64) | **(3,2)** | 0 | **cat32**（`entity.rs:256` `72\|84\|87=>32` 特例臂优先于 `41..=78=>8`） | 假设 5：`blocked=true` | **联动门**（不是怪；f50 才覆写为 cat8，keydemand.rs 模块文档） |
| 5 | 83 | (96,64) | (3,2) | **32** | cat1 假设 2 | `gates_ignored`，放行 | 场景门 → 脚本槽 32 = `TAK_123_126 GUT_37`（无 LAY，不改层） |

（坐标换算：256/32=8、96/32=3、352/32=11、320/32=10、64/32=2。）

**切断机理**（`python3 /tmp/walk29.py` 的地形图）：
13×26 网格中本层仅两条竖走廊 + 顶层横走廊连通：
左列 x=3（y=1..11）、右列 x=8（y=1..11）、顶行 y=1（x=3..8）。
- 路线 A（左列上行）：必经 **(3,2) = ty72 联动门** → 被 cat32 硬阻挡切断。
- 路线 B（经顶走廊到右列下行）：必经 **(8,3) = ty11 墙** → 被硬阻挡切断。
两刀各断一条、互不重叠 ⇒ 乐观模型（门可过/怪可清/钥匙无限）下也 0 可达 ⇒ `Unreachable`。
逐项排除：地形全通（Q1）；ty83 两门不挡（假设 2 放行，且槽 7 为空、槽 32 无 LAY，
即使按 `GatePolicy::Blocked` 对照也仍 `unreachable`，见实验表）；无 ty1/2/3 门、无钥匙、
无拾取、无商店、无 cat8 怪（monsters_ignored=0 与 CLI 一致）。

### Q4：独立复跑「三组放开实验」（A 级复现）

keydemand.rs 的 flood-fill 语义在 `/tmp/f29_experiment.py` 逐条复刻
（`build_model` + 地形 `(terrain||exempt)&&!blocked` + 4 邻接 BFS；类目表对齐
`entity.rs:251-263` legacy 镜像与 `resource_cfg.rs:319-325` floor 覆写）。结果：

```
baseline                  -> unreachable  hard_blocks=2 gates_ignored=2
relax ty11 only           -> unreachable  hard_blocks=1
relax ty72 only           -> unreachable  hard_blocks=1
relax both                -> reachable    hard_blocks=0 gates_ignored=2
baseline + gates BLOCKED  -> unreachable  hard_blocks=2 gates_ignored=2
```

复现：`python3 /tmp/f29_experiment.py`。
**实现者的定性正确**（只差 ty72+ty11、与钥匙无关），但方向上他若表述为「放开任一即可」
则不准确：**必须两实体同时放开**；放开后本层无门无钥匙 ⇒ `needed=(y=0,b=0,r=0)`。

### Q5：f29 独占资源清单与必经性

- **f29 层内没有任何可拾取实体**：无钥匙(26-28)、无拾取(13-40/79-86)、无商店(77/78)、
  无宝箱(5)——sprite29 全表见 Q3。**不存在独占资源**（对照
  `docs/findings/2026-11-18-item-acquisition-surface.md`，该文档亦未列 f29 任何道具行）。
- **但 f29 在主线链上是必经节点**：`solver floors` 显示 f28→f29 是 f29 唯一入边、
  f29→f30 是 f30 唯一入边、f30→f31→f32 依次单链；全部 68 个脚本槽里唯一 `LAY` 是
  slot0 `LAY_2`，**没有任何绕过 f29 的脚本/道具边**（floorgraph 也未登记 f29/f30 的
  ItemUp/ItemDown 边，`solver floors` 各层均只报 2 条楼梯边）。
  即：到 f30+ 必须过 f29，而 f29 层内 `unreachable` 意味着**按 keydemand 模型主线在
  f29 断头**——这正是「硬阻挡假设」需要具名标记的实际后果。
- 引擎侧两实体都非绝对不可过：slot8 脚本含 `MOV_72_3_2_8_4 ... DES_72_8_4`
  （把 ty72 从 (3,2) 搬走再销毁，A 级，level_scripts.txt 第 9 行）；ty11 墙按模块文档
  「可被道具或剧情消除」（B 级，keydemand.rs 假设 5 注释）。

## 2. f29 全部实体表

见 Q3 表格（6 条实体，ty/坐标/param/语义/类目/keydemand 处理全列）。

## 3. Confidence

- 实体表/坐标/param：**A 级**（sprite29/maplv29 字节级解析，41B/1356B 全消费，复现命令给出）。
- 切断机理与放开实验：**A 级数据 + B 级语义**（keydemand 语义在 Python 独立复刻并得同判定；
  类目表对齐 `entity.rs:251-263` 冻结镜像）。未在 Rust 侧跑放开实验（keydemand CLI 无
  放开开关，任务约束只读仓库）；Python 复刻与 Rust CLI 对 `gates_ignored=2`、
  `monsters_ignored=0`、`hazards=0`、`status` 逐项一致。
- 主线必经性：**B 级**（floorgraph 边表 + LAY 全表清点）。

## 4. 两句话结论

1. **f29 是主线必经节点（f28→f29→f30 是唯一楼梯链、无任何绕行边）但层内没有任何独占
   资源**；它 `unreachable` 只因 ty72 联动门 (3,2) 与 ty11 墙 (8,3) 恰好各自切断唯一
   环线的两侧，且必须两者同时被剧情/道具放开才通。
2. **该改**：keydemand 应给 f29（以及未来同类）输出具名的 `blocked_by_assumption`
   状态——`unreachable` 在这里不是「钥匙解不出」也不是「设计上到不了」，而是「假设 5
   把可被剧情消除的硬阻挡当永久墙」的最坏情形产物，与 `no_entry`/`no_up_exit` 一样
   属于「数字未定义、不可当 0 用」的结构性状态；具名化后 `floors`（层间可达）与
   `keydemand`（层内可达）的口径差也不再被误读为矛盾。
