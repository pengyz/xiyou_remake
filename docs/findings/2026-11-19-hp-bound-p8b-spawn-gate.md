# 批8b：hp 上界「生成通道」健全性补洞 + CLI 可见性（实现者自证）

- 日期：2026-11-19
- 基线：HEAD `ba9783d`（批8）。本批**不 push**。
- 触发：独立对抗复核 `docs/findings/reviews/2026-11-19-hp-bound-p8-review.md`
  的 **F2（黄：上界可采纳被活反例证伪）**、**F10（黄：p6 review 未更正）**、
  **F8（观察：CLI 静默不剪不可见）**。
- 纪律：`AGENTS.md`（§3 证据分级、§3.6 原地更正不删历史、§4 跨层三问、§9 小步提交）。

---

## 0. 跨层三问（§4）

1. **问题在哪一层？** 求解器（`crates/game-solver`）的**界模型**，不是引擎、不是门禁。
   引擎行为是对的（`rules/boss.rs` 击杀后 append 掉落实体）；求解器的三张实体表
   （`pickup_hp`/`drop_hp`/`shop_hp`）**只在层入口从根态 `ents` 播种**，于是「层入口不在
   `ents`、之后才被生成」的实体在生成后查表得 0 ⇒ 界低估 ⇒ 剪错。
2. **消费者实际读的是哪份数据？** `hp_bound::plan()` 读引擎真表：
   `rules/boss.rs` 的 `event_of`/`post_victory_event_of`/`event_spawns`（含 post-anim
   收尾组）、`effect_registry::pickup_kind_for_type` + `certified_pure_hp_gain`、
   `data/tables/shop_goods.json`。**本批没有复制任何一张表，也没有硬编码 ty 号**
   （唯一常量 `SHOP_REPEAT=1<<10` 是引擎 `scene.rs` 里 `flags&4` 的位语义）。
3. **证据级别？** **A 级**（可复现命令 + 引擎源码行 + 实跑输出），见 §2/§3。

---

## 1. 缺陷（复核 F2 的活反例，原文复述）

f35 杀 `ty69` 触发 post-anim 组 7 生成 `ty32×3`（`floor_tier(35)=4` ⇒ 800/颗，合 2400）。
杀前根态有 `ty69` ⇒ `drop_hp[69]=2400`（掉落被计入界），杀后 `ents=[ty32×3]` 而表里
**没有** 32（f35 根态无 hp 拾取）⇒ `entity_gain(32)=0`：

```
bound = hp = 200
should_prune(ord, best=301, hp=200, ents=[ty32×3], flags=MAX) = true
```

而该态**真实可达 hp=2600**（2600 ≥ 301）⇒ 剪掉了不劣于最优的支路 ⇒ 「上界可采纳」
作为一般主张**不成立**。判据 `unit_plan_off_for_post_kill_hp_spawn_f35` 把这段算术
原样钉进测试（不经搜索、纯谓词级，毫秒级）。

---

## 2. 修法：fail-closed 闸门（不「计入界」）

**为什么不走「把生成物计入界」**：那要求静态证明生成物集合可完整枚举——
脚本每条 `CES`/`ADD` 链、跨层 `ADD` 回跳、boss 掉落里再掉 boss 的递归、未知 op 的语义。
`entity_gain` 是「表里有该 ty ⇒ 每颗都算」，一旦某个生成物类型没播种，界就少算；证明不了
⇒ 按纪律只能 `Off`（整层不剪，绝不剪错）。

`plan()` 新增三步闸门，任一命中 ⇒ `HpBoundPlan::Off(原因名, 关联 ty/槽号)`：

| 闸门 | 触发条件（数据源） | Off 原因名 |
|---|---|---|
| 1b 击杀生成通道 | 本层**活着的** cat8 怪，`drop_gain(ty,floor)` > 0（`rules/boss.rs` 的 `event_of` ∪ `post_victory_event_of` 的 `event_spawns` 逐条 `certify_entity`） | `post_kill_hp_spawn`，ty=怪 |
| 2b 脚本生成通道 | 本层 ty83 门槽的 `CES_ty_cx_cy` / 同层 `ADD_floor_ty_cx_cy`，生成物 `spawn_gain` > 0 | `script_hp_spawn`，ty=槽号 |
| 3b 商店无界 | cat16 且 `param & SHOP_REPEAT != 0`（引擎 `flags&4` ⇒ 购买成功不清价、不移除 ⇒ 可无限复购）且效果是 `AddHp` | `shop_repeatable`，ty=店 |

**关键是闸门不看「表里是否已有同型」**：f10/f20/f40 这种「根态恰好有 ty32」的层同样
`Off`。放过它们等于用「当前语料不咬」当理由——复核点名禁止，判据
`unit_plan_off_for_kill_spawn_even_when_hp_type_registered` 专门钉这一点（见 §5 变异 M2）。

顺带修掉一处**同源**缺陷：`parse_slot` 原先把 `CES`/`ADD` 生成物的 hp **直接加进槽项
`konst`**，这与「生成物不在表里就漏算」是同一类错误的两面（生成物若是 hp 型会**双算**）。现
改为只认证、不入界（`certify_script_spawn`）。

---

## 3. 数据面：闸门到底咬不咬当前语料（A 级）

```
$ cargo run -q -p game-solver --example hp_bound_sweep | tail -3
...
floor=35 reason=post_kill_hp_spawn ty=69
...
floor=40 reason=post_kill_hp_spawn ty=73
summary {"on": 50, "post_kill_hp_spawn": 4}
```

**4 层被闸门挡住**（整层静态实体全活、difficulty 0、无 flags）：

| 层 | 怪 | 生成组 | 生成物 | 复核是否点名 |
|---|---|---|---|---|
| 10 | ty75 | 组 0 | ty32×3（+钥匙/药） | F2 实测 `drop_gain == after_kill_bound` |
| 20 | ty74 | 组 1 | ty32×3 | 同上 |
| 35 | ty69 | **组 7（post-anim）** | ty32×3 | **F2 活反例** |
| 40 | ty73 | 组 6 | ty32×3 | F2 实测 |

其余 **50 层仍 `On`**（剪枝收益面基本保留），例如 f1 `slots=3 pickup_types=2
root_bound=700`、f3 `slots=2 pickup_types=2 root_bound=1100`、f37 `4700`、f43 `4550`。
`script_hp_spawn` 当前 **0 层**（槽 62 的 `CES_69` 不在任何层的 ty83 param 里发威；
f35 先被击杀闸门挡）。`shop_repeatable` 当前 **0 层**（表里唯一 `AddHp` 商品是
`src=1 idx=12`，对应 f45 的 `param=780`，`flags=3` 无 repeat 位；带 repeat 位的两家店
是 `idx=5`/`idx=8` 的黄钥匙/换钱）——但闸门不依赖语料。

---

## 4. CLI 可见性（复核 F8）

`solver solve-local` 的 stderr 现在打一行（**JSON schema 不动**，stdout 逐字节不变）：

```
$ cargo run -q -p game-solver --bin solver -- solve-local --floor 1
hp_bound floor=1 enabled=true reason=on reason_ty=0 pruned_entry=0 pruned_pop=0 slots=3 pickup_types=2 root_hp=300 root_bound=700
{"floor":1,...,"max_hp":202,...}
```

- `enabled=false` 时 `reason` 具名卡点（`disabled_by_caller`/`no_hp_sources`/
  `post_kill_hp_spawn`/`script_hp_spawn`/`shop_repeatable`/…），未跑层=`not_run`
  ——**「静默不剪」不再可能**。
- `solver run` 分支**一行未动**：它的层路径 `hp_prune=false`，本来就不剪；改它会动
  `run` 的 stderr/JSON，破坏 f51..f54 逐字节（§6）。

---

## 5. 判据与变异对照表

| # | 判据 | 位置 | 钉什么 |
|---|---|---|---|
| A1 | `unit_plan_off_for_post_kill_hp_spawn_f35` | `hp_bound.rs` tests | f35：`plan` ⇒ `Off("post_kill_hp_spawn",69)`；同函数复算旧口径 `bound(200,[ty32×3],MAX)=200` 且 `should_prune(best=301)` = 真 ⇒ 低估 2400 |
| A2 | `unit_plan_off_for_kill_spawn_even_when_hp_type_registered` | 同上 | 严格 fail-closed：根态已有 ty32 时仍 `Off(75)`（f10 情形） |
| A3 | `unit_plan_off_for_script_ces_hp_spawn` | 同上 | 槽 62：f35 ⇒ `Off(("script_hp_spawn",62))`；f3 ⇒ `On`（ty69@3 只出 ty16，非 hp）⇒ 不过度 Off |
| A4 | `unit_plan_off_for_repeatable_addhp_shop` | 同上 | `flags&4`+`AddHp` ⇒ `Off(("shop_repeatable",78))`；同店去掉 repeat 位 ⇒ `On` 且 `bound=300+2000` |
| A5 | `unit_plan_counts_shop_pickup_item_hp` | 同上 | `PickupItem{item:32}` ⇒ 计入 200（不是当 0） |
| E1 | `hp_bound_f3_prunes_with_damage_and_keeps_optimum` | `tests/hp_bound.rs` | 既有：f3 300hp 开/关同解 + `pruned_*>0` + `root_bound=1100` |
| E2 | `hp_bound_f3_full_player_bound_is_constant_4027` | 同上 | 既有：f3 3227hp ⇒ 7 只 cat8 `predict==0`、`root_bound=4027` |
| E3 | `hp_bound_off_without_hp_sources` | 同上 | 既有：无 hp 源 ⇒ `no_hp_sources` |

**变异（各自真红后还原，`cp` 备份 `/tmp/hp_bound.green.rs`）**：

| 变异 | 改法 | 变红的判据（原始输出） |
|---|---|---|
| **M1** 去掉闸门 | `if d > 0 {` → `if false && d > 0 {` | **A1 + A2 红**：`assertion left == right failed: f35 杀 69 会生成 hp 型（组 7）⇒ 必须 Off，不许剪  left: ("on", 0)  right: ("post_kill_hp_spawn", 69)`；A2 `left: ("on", 0)  right: ("post_kill_hp_spawn", 75)`；`7 passed; 2 failed` |
| **M2** 放宽成「根态无 hp 拾取才 Off」 | `if d > 0 {` → `if d > 0 && ord.pickup_hp.is_empty() {` | **只有 A2 红**：`left: ("on", 0)  right: ("post_kill_hp_spawn", 75)`，`8 passed; 1 failed`（A1 仍绿 ⇒ 单靠 A1 钉不住「严格 fail-closed」，这就是 A2 的存在理由） |

TDD 记录（诚实口径）：A1 与闸门是同一次编辑落地的，**先红后绿是事后用 M1 复现的**——
M1（`if false && …`）正是「修复前」的等价形态，A1 在它下面真红（`left: ("on",0)`）；
恢复闸门 ⇒ A1/A2 全绿（`9 passed`）。A2 的承重性由 M2 单独验证（M2 下只有 A2 红）。

---

## 6. f51..f54 逐字节（`cmp` 证明）

HEAD 二进制 `/tmp/solver_head`（`git checkout --` 两个改动文件后 `cargo build`，再用
备份恢复）与改造后二进制，均在**同一 `--out` 路径**上跑（避免 stdout 里的路径字符串
污染比较）：

```
all  : json=SAME stderr=SAME sha256=5c430a6584b3dc38 floors=[51,52,53,54,1,2]
max1 : json=SAME stderr=SAME sha256=b767e3a63c0da3d7 floors=[51]
max2 : json=SAME stderr=SAME sha256=57bb878a02979fdd floors=[51,52]
max3 : json=SAME stderr=SAME sha256=e21f08c21e3269a9 floors=[51,52,53]
max4 : json=SAME stderr=SAME sha256=d13e90a6c64470e9 floors=[51,52,53,54]
max5 : json=SAME stderr=SAME sha256=ba53acb33d55d0d4 floors=[51,52,53,54,1]
max6 : json=SAME stderr=SAME sha256=04df41b4f4cf8dd9 floors=[51,52,53,54,1,2]
```

`all` 的 `5c430a6584b3dc38…` 与复核 F6 记的 `5c430a6584b3dc381ee8c…` 同一前缀；
`stderr` 同为 `status=capped floors=6 total_steps=275 blocked=floor 3 reason=truncated
layer_ok=Some(false) layer_truncated=Some(true)`。`--max-floors 1..4` 即 f51..f54 的逐层前缀。

---

## 7. 复核另两条同类风险（F2 附注）

1. **门槽「未执行却已置位」**（`rules/gate.rs` `evaluate` 的 case 0/18/7 `set_flags.push`）：
   **不低估，不改代码**。基谓词恒 `!v.flag(param)` ⇒ 槽一旦置位永不再触发，真实玩到也拿不到
   该槽奖励 ⇒ 从界里抹掉与真实一致。这是结构性论证，**不是**「因为当前无 hp op 才安全」；
   当前槽 18/19/32 确无 hp op，将来加了此论证仍成立。已写入 `hp_bound.rs` 模块文档
   「已知不健全边界」第 1 条。
2. **商店 `flags&4` 可二次购买**：**已加闸门**（`shop_repeatable`，见 §2 表格），不靠语料。
3. **商店 `PickupItem`**：按 `hp_gain_of_type` 计入（HpUp 精确、其余 0），判据 A5。

---

## 8. 口径降级（§3.6，三处一致）

| 位置 | 原主张 | 现口径 |
|---|---|---|
| `hp_bound.rs` 模块文档 | 「hp 可采纳上界」 | ~~可采纳~~ **已证伪**（f35 低估 2400）；现行=「当前 68 槽语料实测开/关同解 + f35 已被闸门挡成 Off」，并声明这是回归实测不是证明 |
| `should_prune` 注释 | 唯一剪点 | 加「**不是可采纳性证明**」 |
| `solve_local.rs` `prunable` | 「`bound` 是可采纳上界」 | 加 §3.6 更正段（原文保留） |
| `tests/hp_bound.rs` 头 | 「可采纳 ⇒ 不丢最优」 | 「**当前语料**实测不丢最优；不是可采纳性证明」 |
| p6 review `:30`/`:119`/`:398` | 「损血 ≥1908」「3827 真最优」 | 三处原地更正（见该文件） |

---

## 9. 门禁与 ratchet

- `python3 gates/cli.py check` ⇒ **PASS=13 FAIL=0 SKIP=0**（含 `test-ratchet`）。
- `rust-test-counts.json`：`total 1214 → 1219`（新增 5 条 lib 判据：A1–A5），**无任何
  用例数回退**。基线更新原因即本批新增判据；影响面仅基线文件本身。
- `cargo test -p game-solver` 全绿（lib 58、`hp_bound` 3、其余既有档）。

> 【2026-11-20 自我更正（本条更正本身已被证伪，全过程保留）】本条曾按复核 F10 改成
> 「`SKIP=0` 写错，实际 `SKIP=1`」，**这个更正是错的**：复核引的是 `gates/cli.py status`
> （14 门，含 reference-regression / diff-trace / visual-diff ⇒ `SKIP=1`），而上面那句写的是
> `gates/cli.py check`（13 门，不含这三门）⇒ 实测就是 `PASS=13 FAIL=0 SKIP=0`，原句正确；
> commit `ad5fece` 正文同口径，也无误。教训：门禁数字必须绑定命令名。

---

## 10. 还剩哪些已知不健全边界（诚实清单）

1. **未建模的生成通道**：闸门只覆盖「引擎表里可静态枚举」的三条（击杀掉落 / 脚本
   CES·ADD / 商店）。**跨层 `ADD` 回跳**、`GUT` 到别的槽后由该槽再生成的链、灾难性的
   「生成物再杀再生成」递归，靠 `DROP_DEPTH_MAX=4` 兜底而非证明 ⇒ 深度 >4 的链会被
   `drop_depth_exceeded` 挡成 Off（保守），但**深度 ≤4 的复杂链是否被完整枚举仍无证明**。
2. **`bound` 对「同 ty 多实例」是按颗计**：表里有该 ty ⇒ 每颗都算，这依赖「表项存在」；
   闸门只保证「击杀/脚本/商店三类生成物若含 hp 就必须整层 Off」，**其它**途径（剧情脚本
   未识别的 op、未来新增的 op）仍可能低估。`parse_slot` 的未知 op 会 `script_op_unknown`
   ⇒ Off，这是目前唯一的兜底。
3. **商店价格不建模**：「能买得起」不看金币（`bound` 直接加满额），这是**上界方向正确**
   的宽松，但意味着 `shop_hp` 项可能与真实可达性差距很大（不影响不低估）。
4. **f3 的 4027 可达性仍未证**（1M 档只到 3877 且 `truncated=true`）；本批没有推进这一点。
