# S2 外层骨架落地记录（分层收缩搜索 · 第 1 段最小闭环）

**日期**：2026-11-20（实现 2026-11-22）
**范围**：提案 §7 S2（docs/superpowers/proposals/2026-11-20-hierarchical-solver-proposal.md 修订版）
**状态**：已实现，最小闭环可跑；全部判据已证实（标注处除外）

## 1. 敏感表进 game-core（单一真相源）

`crates/game-core/src/rules/sensitivity.rs`：

- `SensitivityRow { ety, trait_flags, eff_atk, can_kill, one_hit_atk, zero_loss_def, counter_loss }`
- `sensitivity_table(scaled, atk, def, hold23, hold24)`：对全部怪一次成表。
- `atk_breakpoints / def_breakpoints / atk_tier_drops / one_hit_drops`：+1 atk / +1 def 掉档分析。
- **零复制公式**：eff 委托 `combat::effective_attack_vs_type`（combat.rs:93-101，trait bit0=49/53/74+持23、bit1=69+持24 ⇒ ×2）；损血委托 `combat::predict_battle_hp_loss(eff, def, hp, eatk, edef, true)`（combat.rs:106-131；armor_gate=true=撞面开战口径，同 `battle.rs predict_engagement` 的 BumpEngage ⇒ no_first_strike）。
- 阈值语义：`one_hit_atk = hp + edef`（秒杀，hits=1 且损血=-1）；`zero_loss_def = eatk`（hits=1 且 eatk≤def）。
- 倍率与 trait 两维都覆盖（复核指出的两点）：测试网格 mult∈{1,30}×hold23/24 与 combat 公式逐点对照；trait 错配不翻倍、9999 饱和全零。

测试：`cargo test -p game-core --lib rules::sensitivity`（6 passed）。

## 2. 外层骨架 `crates/game-solver/src/outer.rs`

- 输入=**数据**：`docs/findings/block-interfaces.json`（schema `xiyou/block-interfaces/v2`），`load_block_interfaces` schema 不符→`SchemaMismatch`，缺字段→`MissingField`，均具名报错、**零回落**。
- 外层状态 =（块 node，keys[y/r/b]，hp，gold，atk，def，held:u128 道具位图，picked 块位图，buys）。
- 段计划 `plan_segments`：seg1=Run(f51→f54→f1..f10，起 (51,(1,11))，锚 boss75@f10b0)；seg2/segfinal 具名 `Off`（`s2_minimum_loop_segment1_only` / `needs_sensitivity(69)`）；f25..f49 支线 `Off(branch_floor_no_block_graph(25))`。
- 块图边：层内邻接=单元直接相邻或隔一格门格（door1/2/3 计钥匙需求；sealed4/sealed81/script_gate=**乐观通行**、记 `optimistic_gates` 计数——v1 口径，见 §5 边界）。
- 换层边优先级链：含到场格的块 → 含反向梯的块（孤梯格如 f52→f53 的 (11,11)）→ 全部邻接到场格的块（块间通道格如 f3 script→f2 的 (2,11)）→ 具名 skip。真数据第 1 段 **skipped_trans=0**。
- 磨属性（H6 已拍板）：ty77/78 export goods key=="StatPct"，每次购买成本=1 步（+price/1_000_000 兜底），atk/def 整数复利 `saturating_add(x*pct/100)`。
- 搜索：Dijkstra（步数优先）；锚判定用 `sensitivity_table`（缺怪→`needs_sensitivity` 具名）；`SearchBudget{max_pops}` 截断即 `truncated=true` 如实上报；同节点支配剪枝（picked 完全相同才比，held 超集保守不比——见 §5）。

测试：`cargo test -p game-solver --lib outer::tests`（6 passed）。

## 3. CLI 最小闭环

- `solver outer [--doc PATH] [--max-pops N] [--out PATH]`：第 1 段块路径 + Pareto 前沿落盘。
- `solver probe-f13 [--out PATH]`：矛盾探测器（引擎实测口径）。

## 4. 矛盾探测器结论（f13 火环——实测，非猜测）

任务书疑点「f13 有 31 堵 ty10 火环」**数字有误**（真数据实测见下）；结论**无矛盾**：

- `solver probe-f13` 实跑（引擎 `SolverInput.floors[13]`，terrain 与实体同源）：
  - ty10 火环共 **3 处**：f13 (6,4)(6,5)(6,6) 竖列（全塔另 29 处都在 f26，属支线）。
  - 楼梯=实体：ty8 (1,11)→f12、ty7 (11,11)→f14，两格均**可达**（火环=墙口径；楼梯格按引擎 `walkable||stair_like`（solve_local.rs:2117）放行——(1,11) 地形本判不可走，正是这种格）。
  - 金箍棒格 (6,3) 不经火环**不可达**（fire ring 恰好只隔离它，对应 f13b1 单块）。
- **判定**：火环不挡楼梯主径，「通关必备 ty10」与主线拓扑**无冲突**；f13 火环的机制语义=宝物门（36 金箍棒在 ty10 列内侧），S1 块图 anomalies「f13: hard@(6,5) 无邻接可走块（孤门/墙中门）」同源印证。

## 5. 已知边界（显式、不猜）

1. **乐观门口径**：sealed4/sealed81/script_gate 三种切块门按可通行、零代价，边带 `optimistic_gate` 注记。低估步数、高估可达 ⇒ 前沿是**乐观下界**。
2. **购买帽 `MAX_BUYS=256`**：f2b7 有免费 +3% StatPct 可无限买，不加帽状态空间不收敛（atk 饱和到 i32::MAX 才停）。256 次 ≈ atk 10→~3900，覆盖第 1 段全部阈值；放开=S3。
3. **支配剪枝保守**：同节点需 picked 完全相同才比较，held 超集维度不比 ⇒ 可能多留非支配状态（宁滥勿缺）。
4. **held 位图无去重语义**：bag 道具只记「持有位」，数量丢失（第 1 段无关紧要，多钥匙道具进 S3）。
5. **entry=null 层**（f51/f1 等 `no_entry`）：起点由 RunSpec.start 具名给出，loader 不报错。
6. **f52b8 商店**：只有 GiveGold(1000)，无 StatPct 挂点——第 1 段磨属性店实际在 f2b7（+3%）。gold 无消费出口（第 1 段 AddKeys 价>0 的店在 f6/f7，gold 记账但 purchase 分支只处理 StatPct）。

## 6. 实跑输出（第 1 段，`--max-pops 800000`）

```
pops=643024 truncated=false 前沿=1095 条 unapplied_exports=7793 needs_sensitivity=[] skipped_trans=0
最短可达锚：steps=52（killable=false，atk=11 杀不动 boss75——诊断面）
最短可杀：steps=64 atk=16 def=12 hp=5200（atk16>def15 ⇒ can_kill）
killable=true 共 53 条
```

样例路径（steps=64）：`f51:f51b1 → … → f2:f2b5 → f2:f2b4 → f2:f2b3 → f2:f2b1 → … → f10:f10b3 → f10:f10b2 → f10:f10b4 → f10:f10b8 → f10:f10b6 → f10:f10b5 → f10:f10b0`，全程带 `optimistic_gate` 注记。

`unapplied_exports=7793`：多数 const export 的 ty（31/32 HpUp 等）已由 `certified_monotone_pickup` 覆盖，未覆盖类型（如 ty13/14/15/33/37 特殊道具）按 None 计数上报——这是**覆盖面诊断**，不是错误。

## 7. 验证链

- `cargo test -p game-core --lib rules::sensitivity`：6 passed
- `cargo test -p game-solver --lib outer::tests`：6 passed
- `cargo test -p game-core -p game-solver`：全绿（见 commit message）
- `python3 gates/cli.py check`：PASS（见 commit）
- 变异对照表：见两笔 commit message（各 ≥3 发）
