# P10 出层目标分层（字典序软契约）与 demand 诊断面

日期：2026-11-20。范围：`crates/game-solver`。状态：**已证实**（全部有测试与变异对照）。

## 1. 分层软契约（objective.rs）

新模块 [`crates/game-solver/src/objective.rs`](../../crates/game-solver/src/objective.rs) 定义出层比较
`cmp_exit(&ExitScore, &ExitScore)`，字典序：

1. **hp 大者优先**（软约束层 1）；
2. 同 hp ⇒ **gold 多者优先**（软约束层 2）；
3. 同 hp 同 gold ⇒ **steps 少者优先**；
4. 全等 ⇒ 调用方用路线字典序做决定性 tie-break（`solve_local.rs::better_exit`，契约外不改序）。

接线点（仅两处）：`solve_local_exit` / `solve_local_exit_toward_shops` 的出层择优
（`pick_exit`/`better_exit`，[solve_local.rs:618-631](../../crates/game-solver/src/solve_local.rs#L618-L631)）。
**层内搜索支配/剪枝不动**；legacy `solve_local`（Q1 面）仍走 `HpLoss`，不在本片范围。

### 为什么 λ=2 不做加权和

若把 gold 折算进 hp（λ=2：AddHp2000 价 1000 金，`data/tables/shop_goods.json` id12），
求解器会接受「用 1999 血换 1000 金」的路线——与「不为攒钱牺牲血」的用户裁定直接冲突；
且金币的可花费上界（未来商店/补钥消费）未建模，任何 λ 都是把未知收益当已知收益折现。
因此 gold 只做**同 hp 下的次级排序**，绝不进入血量加和。

### 为什么 demand 不进单层比较（2026-11-20 用户裁定修订）

初版任务书要求把「后继层钥匙需求」作为硬约束层（satisfies 优先）。修订裁定否决：
**钥匙是跨层动态资源**——可以回早层拾取、f7 商店黄钥 ×5¤50、`SwapYellowKey` 交换；
单层「入场钥匙 ≥ 后继需求」会把「先上楼再回头拿钥匙」的可行路线判成不可行，制造
假不可行。约束的正确位置是**链路层**（witness 逐层接力时考虑回拾/补钥，属下一片）。
因此本片把 demand 全部降级为**诊断观测面**：

- `objective::next_demand(levels, floor) -> Demand`：用 `floorgraph` 找该层全部
  `StairUp` 后继，多后继取逐色最大 needed（保守），后继 keydemand 状态非 `Ok`
  或缺规格 ⇒ `Undefined(tag)`。只调用 keydemand，不复制其逻辑，不硬编码层号。
- `WitnessFloor` 追加四字段（witness schema **v2→v3**）：
  `demand:[u8;3]`、`demand_undefined:Option<String>`、`keys_out:[u8;3]`、
  `key_shortfall:Option<{need,have}>`。
- `key_shortfall` **只落账**：不阻断、不改 `ok`、不改 witness 状态。缺口的层照常
  返回血最大解。

### 一步前瞻的局限

`next_demand` 只看「下一层的上行梯需求」，看不到：
回早层拾取的可能（动态资源）、商店补钥、`SwapYellowKey`、以及后两层的复合需求。
它给出的 `[u8;3]` 是**观测**不是**约束**；链路层真正的可行性判断（含回拾/补钥）
是下一片的工作。在此之前的链路推进仍由 `no_exit_toward` 等既有 blocked 机制把守。

## 2. 为什么不做单层硬约束（证据）

反证锚：`crates/game-core/src/scene_persist.rs:370` 附近 m_119 只序列化**存活**实体——
被拾取的钥匙从层实体表消失，但玩家钥匙袋保留 ⇒ 钥匙天然可跨层携带，需求可以在
别的层被满足。任何「本层必须留够 N 把」的单层谓词都无法表达「去 f1 拿了再回来」。
（链路层回拾/补钥的实现是下一片；本片只保证诊断面与软契约不被硬层污染。）

## 3. f29 口径更正（唯一动 keydemand 的点）

f29 取证（[2026-11-20-f29-reachability.md](2026-11-20-f29-reachability.md)）证明
`unreachable` 是「硬阻挡按永久」假设的最坏产物（ty72@(3,2)+ty11@(8,3) 双断环线，
双放 ⇒ needed=(0,0,0)）。`FloorStatus::Unreachable` 的输出 tag 改为
**`blocked_by_assumption`**（[keydemand.rs:110-114](../../crates/game-solver/src/keydemand.rs#L110-L114)），
变体名保留，CLI 与 JSON 同步。算法未动。
keydemand 全 54 层实测：**ok=49，no_entry=2（f1/f51），no_up_exit=2（f49/f50），
blocked_by_assumption=1（f29）**。

## 4. 行为影响（有意变更，回归锚）

- **f52 链路路线变化**（tests/pure_pickup.rs `OLD_CHAIN`，带 2026-11-20 更正注）：
  同 hp=427 下 gold 软层选中金更多的 **124 步**路线（旧 102 步）。血量、出口不变。
  f51/f53 无同 hp 金差异，逐字段不变。变更意图见上文 §1。
- f3 出层回归锚不变：`out_hp=4027`、`steps=88`（88 步/14 件的最优解仍是最优）。

## 5. `solver run` 前后对比

| 项 | 改动前（d58e440） | 改动后（本片） |
|---|---|---|
| floors | 7（51,52,53,54,1,2,3） | 7（同） |
| blocked | floor=4 reason=no_exit_toward | floor=4 reason=no_exit_toward（同） |
| f3 (out_hp, steps) | 4027, 88 | **4027, 88**（不变） |
| f3 keys_out / demand | （无字段） | keys_out=[0,1,0]，demand=[2,0,0]，key_shortfall={need:[2,0,0],have:[0,1,0]} |
| f52 route | 102 步（hp=427） | **124 步（hp=427，gold 更多）** |
| total_steps | 363 | 385 |
| schema | xiyou/witness/v2 | xiyou/witness/v3 |

## 6. 变异对照（4 发，全部先红后还原）

| # | 变异 | 预期 | 实测 |
|---|---|---|---|
| M1 | keys 层插在 hp 后 gold 前 | `demand_fields_are_diagnostic_only` 红 | 红（objective.rs:174，left=Greater right=Less） |
| M2 | gold 提到 hp 前 | `hp_beats_gold` 红 | 红（objective.rs:182） |
| M3 | next_demand 只取首个后继 | `next_demand_multiple_successors_takes_componentwise_max` 红 | 红（[1,0,0]≠[1,0,1]） |
| M4 | key_shortfall ⇒ ok=false（阻断） | `key_shortfall_reported_not_blocking` 红 | 红（objective.rs:288） |

M1 初发未红（判据 1 的 unequal-hp 用例抓不住「keys 插在 hp/gold 之间」），
已补同 hp keys-vs-gold 用例后复测变红——判据本身随变异补强。

## 7. 门禁与 ratchet

- `cargo test -p game-solver` 全绿（含新 `tests/objective.rs` 11 判据、
  solve_local.rs `p10_objective_pick` 4 单测）。
- `python3 gates/cli.py check`：**PASS=13 FAIL=0 SKIP=0**。
- `ratchet-update`：total **1244 → 1259**（+15：objective 11 + p10_objective_pick 4）。

## 8. 遗留假设

- 链路层「回拾/补钥」尚未实现：f3 仍留不下黄钥（keys_out.yellow=0），链路卡点仍在
  **f4 no_exit_toward**。诊断面已把缺口写进每层 JSON，供下一片消费。
- `f3_exit_reports_keys_out` 的 f3 用链路入场态（hp 3227/攻防 730，flags
  0xe337be_00000000，真商品表），与 exact_pickup 判据 1 同口径。
