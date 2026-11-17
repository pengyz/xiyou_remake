# 对抗复核：确定性求解器实施计划（2026-11-09）

- **对象**：`docs/superpowers/plans/2026-11-09-solver-implementation-plan.md`（472 行，commit `dbc8d86`，单文件 472 ++）
- **复核人**：独立攻审（只读取证 + 仅写本报告，未改动其他任何文件）
- **复核时仓库状态**：分支 `modern`；复核过程中 HEAD 由 `dbc8d86` 前进到 `6efd30e`（他人在途提交 `6efd30e fix(data,core): 修①批 BUG-052 装备 desc 错源+BUG-053 id22 主动换层+BUG-054 下行符空转`，其前置 `529dfd5 docs(core): 登记 BUG-052/053/054（求解器数据层扫描三发现）` 在 `dbc8d86` 之前）；报告落盘时又前进到 `c287be5 gate(data): 表内容哈希基线重锚`；`origin/modern` 落后；工作树仅 `?? android/build/` 与若干未跟踪 docs。计划文件与 HEAD 无 diff。
- **方法**：行号锚全部 `sed -n` 实测（不转引）；文件内容按终态原样粘贴；门禁/金样串行复跑取证。

---

## 总判

**需修订（暂不放行）**：**红 3 条 / 黄 14 条 / 绿 17 条**；**面2 十条关键断言抽验：9 条 PASS、1 条黄（引用不忠实性=未纳入，非矛盾）**。

- 3 条红全部是**用户裁定/红线未落文档**类（面4②、面4③、面5 items 红线），不是代码事实错误——计划的**底层技术事实基本全对**（抽验 9/10，附加 census 复证全中）。
- 修完 3 红 + 面1 的命令级错误（行406 包名、行425/427 旗标、3 处空过测试）后可放行进入批0。

| 面 | 判定 | 一句话 |
|---|---|---|
| 面1 计划自洽与可执行性 | **黄** | 数字/估时/依赖自洽，但 1 处命令按字面必失败 + 2 处旗标错 + 3 处测试空过风险 + 2 处行号错 |
| 面2 关键技术断言抽验 | **黄** | 十条抽验 9 PASS / 1 黄；§2.9 门禁与金样串行复跑均成立 |
| 面3 验收总表 §7 | **黄** | 无人工目测项，A4/A7/A8 机器判；A1-A3/A5/A6/A9/A10 共 5 处弱判据 |
| 面4 三条裁定对齐 | **红** | ①绿；②逐层五元组分叉判别器**缺失**；③QoL 全收集合法性**缺失** |
| 面5 红线与风险 | **红** | items.json 22-25 effect 不可信红线**未写入**（BUG-053 已明文警告求解器）；R1/R2/R3/R5 齐备 |
| 面6 夹带 | **绿** | `git show --stat dbc8d86` 恰 1 文件 |

---

## 面1 计划自洽与可执行性 —— 黄

### 绿（可复制执行的部分）

1. **批0 判据数字全部可复现**：`floors=55`（55 个 maplv）、`enemies=502`（sprite census 独立复算）、`scripts=68`（fixture 行数）、`enemies_table=35 / items=32 / shops=17`（json 行数实测，id 41..75 / 13..86 / 1..17）——批0 无一虚数。
2. **依赖与估时自洽**：估时 1-2 + 2-3 + 2-3 + 3-5 + 1-2 + 1-2 = 10–17d 串行链；行377 明确批0→批5 串行、"批0 的两项【待考证】可与批1 并行"，与行259/262 两项待考证（接线点、层号映射）数目一致。
3. **既有工具真实存在**：`python3 gates/cli.py status|check|original|reference|trace|visual|all|ratchet|ratchet-update|table-content-hash|table-evidence|zero-code-drill` 子命令全部存在；`tools/state_baseline.py` 存在（`:6` = `python3 tools/state_baseline.py --check    # 重跑并与入档清单逐字节核对（重构前后主判据）`）；`tools/route_gen.py` 存在（首行注释"手写路线的走步填充与校验（非规划器）"）；`cargo 1.94.0`；`crates/` = game-core / game-data / game-oracle / game-platform（game-solver 待批0 新建，计划已声明 `crates/game-solver`，不冒充已有）。
4. **TDD 真能红的批次**：批0 census 快照、批2 3×3 玩具夹具、批3 两层玩具图 diff（`/tmp/sol1` vs `/tmp/sol2`）= 真红；批4 `assert {'q1','q2','q3'} <= set(d)` = 真判。

### 黄（7 条）

| # | 问题 | 证据（实测） |
|---|---|---|
| Y1-1 | **行406 §6 回归门命令按字面必失败**：包名 `-p solver`，而计划自己在行268/310/331/350 用的是 `-p game-solver --bin solver` | `sed -n '406p'` → `   → cargo run -p solver -- query --all   # 三查询重跑`；`crates/` 下无 `solver` 包（`Cargo.toml:13 members = ["crates/*", "shell/*"]`，game-solver 尚未建） |
| Y1-2 | **行425 A3 旗标畸形**：`--ultimate36,40` 缺空格（应 `--ultimate 36,40` 或 `--ultimate=36,40`） | `sed -n '425p'` → `` | A3 | Q3 禁终极装备可达性 | `... -- query q3 --ultimate36,40`（集合=批0 裁定常量） | …`` |
| Y1-3 | **行427 A5 旗标风格与行350 不一致**：`--query --all`（旗标） vs `query --all`（子命令） | `sed -n '427p'` → `连跑 2 次 --query --all 到不同文件后 diff`；`sed -n '350p'` 为 `query --all` 子命令形态 |
| Y1-4 | **3 处 cargo test 过滤器空过风险**：过滤器零匹配时 `cargo test` 退出码 0，可"绿"在 0 个测试上 | 行293 `cargo test -p game-solver --lib 对拍`；行370/428 `cargo test -p game-oracle solver_replay`。实测：`grep -rn "solver_replay" --include=*.rs --include=*.py crates/ tools/` = **0 命中**（测试尚未存在，符合"批5 新增"语义，但命令本身无法区分"没写"与"没过"） |
| Y1-5 | **行168 行号错**：`save_floor_state → scene_persist.rs:383` | `grep -n "fn save_floor_state" crates/game-core/src/scene_persist.rs` → **398**（383 是 change_floor 内 min/max 更新区） |
| Y1-6 | **行115 引用文件不存在**：`crates/game-core/src/battle.rs:189-192` | 实际为 `crates/game-core/src/subsystem/battle.rs:190`（`ctx.enemy_base[idx].gold << 1`）；`crates/game-core/src/battle.rs` 不存在。计划 §8 已自标"转引未核"，故判黄不判红 |
| Y1-7 | **批1 TDD 红不真**：`GLV/SMS 状态不变` 属 B 类"无变化"断言——未实现转移时状态天然不变，断言天然通过，不能充当红测试 | `sed -n '286p'` → `- [ ] 分类单测：每 opcode 至少 1 条 fixture 语料驱动的转移断言（A 类改状态、B 类状态不变、C 类按约束）。`；`sed -n '297p'` → `**TDD**：先写"GLV/SMS 状态不变""RES easy 重置"等红测试 → 实现转移表。`。其中 A 类（改状态）可红，B 类（状态不变）天然空过 |

**附带黄（并入面2 第 7 条计数，不重复计）**：行131 "64 编号行、297 条指令、DES 41" 是 **assets/raw/script 侧**数字；fixture 实为 **296 条 / DES 40**。两侧差 = slot0/31 各少 1 DES、slot18 多 1 DES。计划未标明归属，批1 若拿 297/DES41 对 fixture 断言会误判。

---

## 面2 关键技术断言抽验 —— 十条：**9 PASS / 1 黄**

| # | 断言 | 实测命令与输出 | 判定 |
|---|---|---|---|
| 1 | ScriptOp=23 变体 | `awk '/pub enum ScriptOp/,/^}/' .../script_ops.rs \| grep -cE '^\s+[A-Z]'` → **23**；`sed -n '6p'` → `pub enum ScriptOp {`（Tak,Lay,Gin,Res,Add,See,Gut,Mov,DesAll,DesAt,Swd,Ces,Glv,End,Sms,RosMove,RosEquip,RosHp,RosFace,RosAtk,RosDef,RosNoop,Mvs） | **PASS** |
| 2 | maplv 13×13 | `xxd -l 8 assets/raw/maplv0` → `1a00 1a00 0000 0000`；文件 1356B = 4+13×13×4×2；`sed -n '69p' crates/game-data/src/maplv.rs` → `let width = (raw_width >> 1) as usize;`；`walk_replay_floor1.rs:26` → `let (cols, rows) = (13usize, 13usize);` | **PASS** |
| 3 | 初始状态 (300,10,10,51) 来自 intro.rs:361 且 GAME 段零读取 | `sed -n '361p' crates/game-core/src/intro.rs` → `pub fn new_game_stats() -> (i32, i32, i32, i32) { (300, 10, 10, 51) }`；`grep -rn "STLAYER\|EDLAYER\|WHOSYOURDADDY" --include=*.rs .` → **0 命中** | **PASS** |
| 4 | fixture 68 槽 vs raw 5 槽 `[0,18,27,31,63]` | fixture `awk END{print NR}`=**68**，空槽 [39,58,59,60]；python 逐槽 diff → 不一致 = **[0,18,27,31,63]**（slot0 raw 多 `DES_84_6_11`；slot18 fixture 多 `DES_1_4_4`；slot27 raw `DES_84_11_4` vs fixture `DES_84_6_11`；slot31 raw 多 `DES_10_6_7`；slot63 raw `DES_-59_0` vs fixture `DES_-66_0`）——与计划行128 完全一致 | **PASS** |
| 5 | combat.rs:106 `predict_battle_hp_loss` 闭式 | `sed -n '93,130p'` → `:106 pub fn predict_battle_hp_loss(`；hits 上整 115-119、多击 122、首击 127，落在计划 :115-120/:121-124/:126-128 区间内；`effective_attack_vs_type` 在 `:93`（&1→23、&2→24 ×2） | **PASS** |
| 6 | 三表 35/32/17 | python json 实测：enemies **35**（id 41..75）、items **32**（id 13..86）、shop_goods **17**（id 1..17） | **PASS** |
| 7 | GLV/SMS 空体、MVS 死码 | `script_ops.rs:264 pub(super) fn op_glv(&mut self) {}`、`:279 op_sms` 空体、`:269 op_end → self.transition_to(GameMode::Ending);`；`:318-320` MVS 双重死码注释（"语料 297 条指令 0 条 MVS + 全 55 层 sprite 型 82=0…语料不可达，为忠实而保"） | **PASS** |
| 8 | 考证文档结论引用忠实 | 计划**零引用** `docs/findings/2026-11-09-qol-passive-item-triggers.md`（`grep -n "BUG-\|bug-ledger\|qol-passive"` 计划 = 0 命中）。已引断言无矛盾：行388 `trait_flags（49/53/74=1、69=2）` ↔ 文档 `:302-306`（太公杖对 49/53/74）、`:396-399`（捆仙绳仅杨戬 69）；行159 `15筋斗云(TutorialSeenStack)` ↔ 文档 `:79-81`。**但漏 §6（`:562-571`）对求解器的四条必须建模结论**，尤其：**15 筋斗云 = 独立于踩梯的换层通道 + 探索区间门 [minFloorReached,maxFloorReached]（4 状态分量）**、**25 幸运币两处击杀点翻倍（`:447-452` a.java:10052-10058 + a.java:7950-7954，商人/脚本GIN/宝箱不翻倍）**——计划行115 只引 battle 一个点 | **黄**（无矛盾，未纳入） |
| 9 | 金样 74 GREEN | `state-dump-golden.json` `scenarios`=**74**；串行实测 `python3 tools/state_baseline.py --check` → `[GREEN] 金样逐字节等价：74 场景 sha 全同`（RC=0），与计划 A7 预期串逐字一致（`state_baseline.py:138`） | **PASS** |
| 10 | 通关终点脚本行 65 | `assets/raw/script:77` = `65 TAK_127_128 END_0`；`scene.rs:3891-3894` = `72 => { if self.persist.floor == 50 { self.load_level_script(65); } }` | **PASS** |

**附加复证（全 PASS，支撑计划 §2.2/§2.9）**：
- sprite census 独立 python 解析（55 文件 consumed==len）：总记录 **1703**、敌人 **502**、**45** 层有敌、type12=24、type13=1、stair7=54、stair8=51、type81=5、**type82=0**、type83=**52**、type72=**4**、敌人型 **35** 种；type83 param 集合 = 0..16,18,19,24..34,38,40..50,52,53,56,64,66,67，且 **param24/34 各 3 个**；type72 ROUTE = (2,96,224)/(15,288,32)/(29,96,64)/(50,192,160,ex=01) —— 全中计划 §2.2。
- **§2.9 门禁声明（行176）串行复跑成立**：`python3 gates/cli.py status` → **`PASS=13 FAIL=0 SKIP=1 TOLERATED=0`**（与计划原文一致）。说明：首跑曾得 PASS=12 FAIL=1（diff-trace FAIL），系与本复核的后台 `state_baseline --check` **并发抢 CPU** 所致；单独 `python3 gates/cli.py trace` → PASS(exit 0)。**计划该条为绿，非文档错误**。
- 行号锚批量 `sed -n` 核验（maplv 65/93/117、sprite.rs 54/107/133、enums 164/174/177、handoff 97/98/111/117、scene 252-253/612/926/3496/3804-3820/3891-3894、scene_persist 349/362/366/410/444/501/219、combat 42/51/93/106/170、subsystem/battle 33/79/85/110/122、script_ops 6/167/226/264/269/279/282/289/401/553、game-data/script 25/31/53/93/111、game-core/script 154/193、intro 361、pathfind 22-27、golden 49/226-247、state_baseline 6、route_gen 1-30、scene_script 14、script-dsl-semantics 152/§2:66/71/72）**全部命中**；仅两处错 = Y1-5、Y1-6。
- 轻微过列：计划把 `scene.rs:2045/3592/3739` 并列为 Bump/Route 调用点，实测 `:2045 RouteEngage`、`:3592 BumpEngage`、`:3739 Exchange`（第三点是 Exchange，非 Bump/Route）。

---

## 面3 验收总表 §7 —— 黄

### 绿（2）
- **无人工目测项**：A1–A10 全部是命令 + 输出断言，无"肉眼看图/看效果"项。
- **A4 / A7 / A8 严格机器判**：A4 python assert（行426）；A7 预期串 `[GREEN] 金样逐字节等价：74 场景 sha 全同` 与 `state_baseline.py:138` 逐字一致（本次实测同串）；A8 `FAIL=0`（行430）。

### 黄（5）
| # | 项 | 问题 | 证据 |
|---|---|---|---|
| Y3-1 | A1–A3（行423-425） | 有输出**形状**（`Q1 max_hp=<N> …`）但**无断言、无固定期望值**，失败需人读行 | `sed -n '419,432p'` 见"预期输出"列仅为格式串 |
| Y3-2 | A5（行427） | 判据命令 `--query --all` 旗标畸形（同 Y1-3），按字面与 `query --all` 不是同一入口 | `sed -n '427p'` |
| Y3-3 | A6（行428） | **未标注"批5 新增"**（A10 在行432 明标"（批1）"）；且 `cargo test -p game-oracle solver_replay` 过滤器当前 0 匹配 → exit 0 空过。§4 批5 行370 语境可推断归属，但 §7 总表本身暗示"已有可跑" | `grep -rn solver_replay crates/ tools/` = 0 命中；`sed -n '370p'` = 批5 判据 |
| Y3-4 | A9（行431） | `cargo tree -p game-solver -i game-solver` 半机器；"lib 无 fs/net 断言测试"**无具体命令**，实为人工/约定项 | `sed -n '431p'` |
| Y3-5 | A10（行432） | `cargo test -p game-solver --lib`（无过滤器）在"测试还没写"时同样 exit 0；"23 变体覆盖 + fixture 64 行对拍 0 差异"依赖行293 的 `对拍` 过滤器（空过面） | `sed -n '432p'`、`sed -n '293p'` |

---

## 面4 与用户三条裁定的对齐 —— **红（1 绿 / 2 红）**

| 裁定 | 定位（行号均 `sed` 实测） | 判定 |
|---|---|---|
| ① 三查询 = 剩余血量最大 / 最少杀怪 / 不拿终极装备能否通关 | **§1.2 行43-60**：行51 `Q1 剩余血量最大 … 最大化通关时 player.hp`、行52 `Q2 最少杀怪（=剩余怪物最多）… 最小化击杀数`、行53 `Q3 不拿终极装备能否通关 … 可达性 0/1` | **绿** |
| ② 逐层五元组（血/攻/防/钥匙/钱）回放校验 = 预测 vs 真引擎分叉判别器 | 全文仅 **行114**（`PlayerCombat 字段（逐层 checkpoint 的字段源）`）、**行189**（`model.rs … checkpoint 五元组+背包`）、**行322**（`checkpoint 字段对齐 PlayerCombat（combat.rs:51）`）——三处都是**字段来源/结构**声明；**没有任何"逐层 checkpoint 五元组预测值 vs 真引擎实测值比对、分叉即判失败"的机制**。批5 回放（行362）是"整条路线喂 oracle 走到 Ending"，不是逐层五元组预测/实测分叉判别 | **红** |
| ③ QoL 必收集（筋斗云可晚拿不可不拿；解合法性 = 通关 + QoL 全收集；Q2 最小杀怪在此前提下） | `grep -n "QoL\|全收集\|收集\|筋斗云\|可晚拿"` 仅命中 **行159**（物品清单里的 `15筋斗云(TutorialSeenStack)`）；**无"解合法性=通关+QoL 全收集"约束、无"筋斗云可晚拿不可不拿"、Q2 约束列只有"存在通关路线"（行52）** | **红** |

---

## 面5 红线与风险 —— **红（1 红 / 4 绿 / 1 黄）**

### 红（1）
**items.json 22-25 effect 不可信红线未写入计划**：
- 计划全文 `grep -n "BUG-\|bug-ledger\|effect_registry"` → **0 命中**；§2.7（行152-161）反而把 `items.json 全 32 行 effect key 实测` 当作**实测事实**呈现，批0 ingest 直接读 items.json。
- 而 `docs/bug-ledger.md:1035` = `## BUG-053 items.json effect 映射错位（22-25 四件·Stage B）`，明文"**影响：①求解器若按 items.json effect 建模=按错效果算**"。
- 该台账由 **`529dfd5`（在 `dbc8d86` 之前一个 commit）** 登记——计划作者可见，且同分支后一提交 `6efd30e` 已在修 BUG-053。**计划与台账直接冲突且未自标** → 必须补红线（"items.json 22-25 effect 字段禁作求解器效果源，以 bug-ledger/实考证为准"）。

### 绿（4）
| 风险 | 证据 |
|---|---|
| R5 双真相源（fixture vs assets/raw/script） | 行389：`R5 … 用错源=解不可回放 \| ingest 只读 fixture（批0 判据钉死）；assets/raw/script 仅语料参考` —— **裁定方向明确、可落地** |
| R1 状态爆炸 | 行385：`离散桶 + canonical 去重 + 有界求解（界可配）；报审时给界与耗时曲线；不承诺全局最优（§1.3）` |
| R2 隐藏墙双通道 | 行386：`可通行 = terrain ‖ 实体槽（pathfind…）` 对策已写（每步重建实体集 + 对拍） |
| R3 剧情强制脚本 | 行387：`A 类强制 ROS_2/DES 全量进状态机 + 回放兜底` |

### 黄（1）
R 表（行385-394）**缺"筋斗云独立换层通道"风险**：QoL 考证 §6（`:570`）明文它是"**独立于踩梯的换层通道**，且是全游戏唯一带探索区间门的换层路径"（4 状态分量）。层间 DAG（行322）只列"楼梯 param 指向 + LAY/GUT + 型72 触发"，未含 15 筋斗云换层边 → 会直接漏解/误判可达性（与面4②③ 同源）。

---

## 面6 夹带 —— **绿**

```
$ git show --stat dbc8d86
 dbc8d86 docs(solver): 确定性求解器实施计划（批0-批5 切片+三查询验收口径）
 docs/superpowers/plans/2026-11-09-solver-implementation-plan.md | 472 +++++
```
**恰 1 文件、472 行新增、零代码改动**，无夹带。（复核期间 HEAD 已被他人在途提交推进到 `6efd30e`，计划文件与 HEAD 无 diff。）

---

## 放行前必修清单（按优先级）

1. **【红·面4③】** 增写 QoL 合法性口径：解合法性 = 通关 + QoL 全收集；筋斗云可晚拿不可不拿；Q2 约束改为"存在**满足该合法性**的通关路线"。
2. **【红·面4②】** 增写逐层五元组（血/攻/防/钥匙/钱）checkpoint 的**预测 vs 真引擎分叉判别器**（每层落 checkpoint，oracle 回放逐层比对，任一层分叉即判失败）——现有批5 整路线回放不覆盖该裁定。
3. **【红·面5】** 写入 items.json 22-25 effect 不可信红线（引 `docs/bug-ledger.md:1035` BUG-053），并在 §2.7 标注"effect key 不作效果真相源"。
4. **【黄·面1】** 行406 改 `-p game-solver --bin solver`；行425 改 `--ultimate 36,40`；行427 改 `query --all`；行168 改 `scene_persist.rs:398`；行115 改 `crates/game-core/src/subsystem/battle.rs:190`（或维持 §8 转引标注）；行131 标明"297/DES41 = assets/raw/script 侧，fixture = 296/DES40"。
5. **【黄·面1/3】** 对行293/370/428/432 补"测试名必须存在"的判据（如 `cargo test … -- --list | grep -c` > 0），避免过滤器空过；批1 B 类"状态不变"断言改配 A 类红测试；A6 标注"（批5 新增）"；A9 给出具体命令。
6. **【黄·面2/5】** 引用 `docs/findings/2026-11-09-qol-passive-item-triggers.md` §6 四条必须建模结论（尤其筋斗云换层边进 R 表与层间 DAG、幸运币两处击杀点）。

---

*取证方式说明：所有行号锚均 `sed -n '<n>p'` 实测；文件内容按终态粘贴；门禁与金样为串行实跑结果（`gates/cli.py status` → PASS=13 FAIL=0 SKIP=1；`state_baseline.py --check` → 74 场景 sha 全同）。本报告为唯一写入文件。*
