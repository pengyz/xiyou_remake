# P6 复核：闭式解推广到「单调拾取层」（对抗式独立复核）

- 复核日期：2026-11-19
- 被复核对象：`modern` 分支未 push 的两个 commit
  - `64ae5f1` A 片：witness v2（`pure_pickup` / `pure_pickup_reason`）+ `ExitSolve.pure_pickup_reason` + G4 口径更正
  - `fb5c15b` B 片：`certified_monotone_pickup`（六轴非降认证）+ G3→G3′ + 回放逐轴断言 + 6 条新判据
- 父子链：`0a244ff` → `64ae5f1` → `fb5c15b`（HEAD）
- 复核者权限：只读 + 跑测试/门禁/CLI；**只写本文件**。变异与几何穷举全部在临时副本
  `/tmp/p6rev`（`git archive HEAD | tar -x`）与父提交 worktree `/tmp/p6base`（`git worktree add /tmp/p6base 0a244ff`）里进行。
- 纪律基线：`AGENTS.md`（§3 证据分级 A/B/C、禁调阈值掩饰、禁放宽断言、§3.6 证伪更正文化）。
  **实现者自述与「测试全绿」均不作为判据**；本文所有结论只引用我自己跑出的命令与输出。

---

## 1. 判定摘要表

| 面 | 判定 | 一句话证据（详见 §2） |
|---|---|---|
| F1 witness v2 契约 | **绿** | 真机产物 diff：v2 = v1 **纯加** 2 字段（`pure_pickup`/`pure_pickup_reason`），`meta.schema` `xiyou/witness/v1→v2`，无字段删除/重排；仓库内无任何「按 v1 严格比对」的消费方（仅 `tests/witness.rs` 断言字段序，已同步） |
| F2 f3 证伪（最重要） | **绿（结论对，归因需更正）** | 独立穷举 2^7 击杀集：`4027` 需要全捡 18 枚 ⇒ ≥6 杀 ⇒ 损血 ≥1908 ⇒ 上界 **2119**，**4027 在任何路线下都不可达**；但**真墙不是怪物而是 `ty83@(5,11)` 步臂脚本**（G2），且 f3 的 HP 最优 **3827 零战斗**（9 枚免费拾取），实现者「f3 必须建模战斗」的推论过头 |
| F3 单调认证健全性 | **绿** | 无 ty 效果硬编码（唯二出现：探针夹具 `types[0]=13`、`pickup_kind_for_type=="NoOp"` 派发闸）；`ItemCtx` 字段面穷尽（`player` + `floor/tutorial_seen/equipped_*` + 装备表），`kills/steps/flags/menus` **物理上不可达** ⇒ 无漏轴；真实下降动作存在（ty13 合并 `uses 2→1`）且被拒 |
| F4 背包合并缺口 | **绿** | 32 槽 + 已持 ty19 再拾 ty19 ⇒ `reason=replay_bag_mismatch`（**保守拒绝，无错误采纳**）；空背包正对照 ⇒ `adopted`。附：链上默认 `item_types=[]` 时更早以 `bag_no_room` 拒绝（自述未提） |
| F5 回放逐轴断言承重 | **黄** | hp 轴承重（改 `in_hp+Σ/2` ⇒ 9 条红）；但把 `atk/def`、三钥、背包**三个比较守卫各自删掉，整套判据仍 13 passed** ⇒ 三条断言在本包内**零覆盖**（纯防御代码） |
| F6 旧判据载荷变更 | **绿** | 判据 5 非永真：断言 `!pure` + `reason=pickup_not_certified` + `reason_ty=13` + `hp_expected=0`（无残值），同函数内 ty26 正对照真断言 `adopted`+`yellow=1`；附注 ty13 的「下降」归因不唯一（去掉 uses 守卫仍被 `is_gain` 拒） |
| F7 f51..f54 回归 | **绿** | 独立编译父提交二进制 + 逐字段 diff（6 层 × 15 字段）：**diff=0**；`f54 out_hp=3227` 仍在；`solver run` 汇总行逐字相同（`capped floors=6 total_steps=275 blocked=floor 3`） |
| F8 基线诚实性 | **黄** | `#[ignore]` **被算进** test-ratchet（`pure_pickup --list` = **14** = 13 跑 + 1 忽略）；即那条「f3 必须出解」的证伪记录会永远占一个基线名额 |
| F9 门禁/夹带/并发 | **绿** | 串行 `gates/cli.py check` = `PASS=13 FAIL=0 SKIP=0`（含 `test-ratchet total=1193(基线1183)`）；两 commit `--name-status` 共 7 个文件全在声明范围内；`git reflog` 线性无并行/amend 混入；`modern` 领先 `origin/modern` 13（未 push 属实） |
| F10 下一个撞界点判断 | **不认同（黄）** | f3 的真墙是**一次性开门脚本**（`ty83@(5,11)` param=27，`probe_gate=false`，脚本内容 = `TAK×2 + DES_84_6_11 + GIN_0_13`），不是战斗；零战斗最优 3827 已被我穷举证明 ⇒ 该做「脚本建模 + 代价下界支配」，战斗搜索是最后一档 |

计数：**绿 7 / 黄 3 / 红 0**（F5/F8 黄＝「有断言但无判据承重 / 忽略用例占基线名额」；F10 记黄＝「方向判断不认同，但非缺陷」）。

> 【2026-11-19 更正 · 针对上表 F2 行】（表格内无法插注，故紧贴表后落注，原文不删）
> 「`4027` 需要全捡 18 枚 ⇒ ≥6 杀 ⇒ **损血 ≥1908** ⇒ 上界 2119」中的 **1908 不成立**：
> 它是 `probe_bump` 用 **S0 的 `atk=def=10`** 去配**链态 `hp=3227`** 的混合态结果，两个数字不同源。
> 实测（`ba9783d` 批8 证据）：**730/730 下 7 杀合计损血 0**（返击也是 0）；**10/10 下合计 1958**
> （减去 `ty42` 的 50 恰为 1908 —— 即 1908 正是「10/10 且漏掉 ty42」的口径）。
> ⇒ 4027 在真链态下**是否可达仍未证**（730/730 损血项为 0，可达性取决于门脚本臂）；但
> 「4027 不可达」不能用 1908 当理由。另见 §3.3 的 3827 更正（1M 档实测 3877 > 3827）。

---

## 2. 每面详证

### F1 witness v2 契约 —— 绿

**代码锚**（`crates/game-solver/src/witness.rs`）：

- `:84` `pub const WITNESS_SCHEMA: &str = "xiyou/witness/v2";`
- `:204-209` `/// 【v2】…` `pub pure_pickup: bool` / `pub pure_pickup_reason: Option<String>`（声明在 `truncated` 之后 ⇒ 序列化位置在尾部）
- `:358` `fn record_floor(` / `:387` `pure_pickup: got.pure_pickup,`
- `:81-82` 模块注释明写「既有字段与顺序零改动」
- `crates/game-solver/src/solve_local.rs:442` `pub pure_pickup_reason: Option<String>`；`:596` 成功臂写入、`:608` 失败臂写入

**真机 diff（A 级）** —— 同一装载路径，父提交二进制 vs HEAD 二进制各自产出 witness JSON：

```
# 父提交（/tmp/p6base，独立 target）
/tmp/p6base-target/release/solver run --out /tmp/w_base.json
# HEAD
cargo run --release -q -p game-solver --bin solver -- run --out /tmp/w_head.json
```

实测：

```
base meta: {'schema': 'xiyou/witness/v1', 'start_floor': 51, 'start_entry': [1, 11], 'layers': 6, ...}
head meta: {'schema': 'xiyou/witness/v2', 'start_floor': 51, 'start_entry': [1, 11], 'layers': 6, ...}
base floor keys: [... 'ok', 'truncated']
head floor keys: [... 'ok', 'truncated', 'pure_pickup', 'pure_pickup_reason']
removed: set()   added: {'pure_pickup_reason', 'pure_pickup'}
top removed: set()   top added: set()
```

⇒ **v2 是纯加字段 + 版本号 bump**，没有删除、没有重排（尾插）。位置化消费者（按数组下标读）与结构体消费者都不受影响。

**消费方排查**：`grep -rn "witness" crates/ gates/ tools/ | grep -v "^crates/game-solver/src/witness.rs"` 的命中只有 `crates/game-solver/tests/witness.rs`（自己人，已同步）与 `src/bin/solver.rs`（只写不读）。仓库内**没有**任何 golden/schema 严格比对文件（`find . -name "*witness*"` 只返回 src/tests/两份 review md）。⇒ 「P5 回放误读」不成立。

**A 级产物**（真实 f1..f54 链）：

```
51 True False False step_arm_hazard_reachable/83 247
52 True False False step_arm_hazard_reachable/83 427
53 True False False step_arm_hazard_reachable/83 827
54 True False True  adopted                     3227
 1 True False False step_arm_hazard_on_entry/83  3227
 2 True False False step_arm_hazard_reachable/83 3227
```

**遗留（记入 §5）**：`docs/findings/2026-11-19-pure-pickup-closed-form.md` §1 的 G1 行仍写「f1 实测 41、f53 实测 51」，
与同文 §7.2 的更正块（`:214-215`）以及上表实测**互相矛盾**；§5 判据表的行号也整体漂移约 20 行。

---

### F2 f3 证伪是否成立 —— 绿（结论对，归因需更正；详见 §3）

独立手段：`/tmp/p6rev` 内自写 example `crates/game-solver/examples/p6_f3.rs`
（`load_floor` 真夹具 → 自绘 ASCII / 实体表 / `probe_bump` 实测损血 / 2^7 击杀集穷举 BFS），
以及一份只用 dump 结果的纯 Python 状态 BFS 交叉验证。

**实体表与实现者自述完全吻合**（35 枚实体，7 只 cat8，18 枚拾取，含两扇 `ty83`）：

```
pickups: ty26×8((4,1),(6,1),(5,2),(4,3),(6,3),(1,1),(11,8),(2,8))
         ty31×4((2,2),(11,7),(11,1),(2,9))  ty32×3((4,2),(5,1),(6,2))
         ty30@(2,1)  ty28@(5,3)  ty29@(1,9)
monsters: ty45@(1,3) ty45@(10,8) ty44@(1,7) ty43@(10,2) ty41@(7,5) ty42@(8,10) ty43@(3,5)
gates   : ty83@(2,11) param=0 (入场格)   ty83@(5,11) param=27 (可达)
shop    : ty77@(11,4) param=278
exits   : (11,11) Stair, (1,11) Stair        entry=(2,11)
```

**4027 不可能（A 级，两条独立证据）**：

1. `IN_HP_F3 + 800 = 4027` 要求收齐 18 枚（`Σhp = 3×200 + 4×50 = 800`）；
2. 穷举 128 个击杀集后：收齐 18 枚**最少也要杀 6 只**，该 6 杀集实测损血
   `638+638+384+112+112+24 = 1908`（`probe_bump` 在 `atk10/def10` 下实测）
   ⇒ 上界 `3227 + 800 − 1908 = 2119`。

> 【2026-11-19 更正】上面这个 `1908` **不能**作为「4027 不可达」的证据：它是 `probe_bump` 用
> **S0 的 `atk=def=10`** 去配**链态 `hp=3227`** 的**混合态**实测值（hp 来自链态、攻防来自 S0）。
> 真链态装备下实测：**730/730 时 7 杀合计损血 0**，连返击也是 0；**10/10 时合计 1958**
> （减去 `ty42@(8,10)` 的 50 恰为 1908 ⇒ 本节 1908 = 「10/10 且漏算 ty42」）。
> 因此 `3227 + 800 − 1908 = 2119` 这条上界**不成立**。结论层面：「4027 是否可达」**仍未证**
> （730/730 下损血项为 0，唯一未知是门脚本 `ty83@(5,11)` param=27 之后的拾取面，
> 见批8 复核 F2/F3 与 §3.3 更正）。本节的「真墙是脚本不是怪物」不受影响。

**但真正的第一道墙不是怪物**（这一点实现者说错了，见 §5）：

```
no-kill, no-hazard reach: 4 cells: [(1,11),(2,11),(3,11),(4,11)]    pickups: []
no-kill, hazard-allowed reach: 26 cells, pickups: 9 枚(全是免费区)
```

入场格所在的**口袋只有 `(5,11)` 一个出口，而那格正是 `ty83`**。所以在求解器的结构模型里
（hazard 格不可踩），f3 连一枚拾取都拿不到；一旦允许踩 `ty83@(5,11)`，才能拿到 9 枚免费拾取。
即：**闸门序上先响的是 G2（可达步臂），不是 G1（怪物）**——这与忽略判据的实测输出
`reason=step_arm_hazard_reachable` 一致，也与我跑出的链上 f2 同型。

**另附更正**：`probe_gate(f3, 0) = true` ⇒ 入场格那扇 `ty83 param=0` **可通过 G2a**
（`pure_pickup.rs:603-605` 的口径是 `ty==83 && probe_gate`）。而 `probe_gate(f3, 27) = false` ⇒
走廊那扇门**不满足**「门已开、脚本不跑」。所以实现者自述「G1/G2a/G2 三闸都失败」是错的：
G2a **通过**，G1 是第三顺位（G2 先响，轮不到它）。

---

### F3 单调认证的健全性 —— 绿

**零 ty 硬编码**：`crates/game-core/src/effect_registry.rs:536` 起 `certified_monotone_pickup(ty, floor)`
通体只有 `ty` 作为**入参**流转；全函数唯一的数值字面量出现在探针夹具
（`:562-576` `types=vec![0u8;32]; types[0]=13; uses[0]=2; item_stack_size=1; hp=100 atk=11 def=12 keys 3/4/5 gold=7`）
与闸门①`pickup_kind_for_type(ty)=="NoOp" ⇒ None`。效果语义全部来自 `ItemResolver.pickup` 真跑。

**轴清单是否遗漏**：`ItemCtx`（`effect_registry.rs:74-82`）= `player, floor, tutorial_seen, equipped_weapon,
equipped_armor, equip_types[12], equip_bonuses[12]`；`ItemResolver::pickup`（`crates/game-core/src/subsystem/item_service.rs:29-33`）
是 `pickup_kind_for_type` + `pickup_params_for` + `builtin_effect(kind).apply(ctx,&params)`，
`apply` 只拿得到 `&mut ItemCtx`，且 `ItemResolver` 是**无状态 ZST**（`:24-25`）⇒ 没有隐藏输入。
⇒ `kills / steps / flags / menus / floor_visited` 这些字段**在类型系统层面不可达**，
不是「忘了比较」而是「拿不到句柄」。`floor` 是入参（`ty` 效果可按层不同），已作为参数传入并采样。

**`tutorial_seen` 初值 false 的必要性**：它确实是 `ItemCtx` 的字段（教学位），初值 false 是
「新玩家」态；`tutorial_seen != false ⇒ None` 是**采样后置零断言**，不是掩盖——若某型会置教学位，
它在真实层里就是不可忽略的副作用，拒绝是正确方向（宁严不宽）。

**真实下降动作存在且被拒（A 级）**：`StackEffect`（`:183-220`）
`ITEM_USES[0] = -1`（`idx = ty-13` ⇒ ty13）+ 「已在背包则 `uses += ITEM_USES[idx]`」
⇒ 玩家已持 ty13 再拾 ty13 时 `uses 2→1`，**真的下降**。认证侧在 `:613-642` 合并分支
`if player.item_uses[i] < before_uses[i] { return None }` 拒绝。

**冗余但无害**：我删除该守卫（变异 E）后 `cargo test --release -p game-core --lib monotone_probe`
仍 **2 passed**——因为 ty13 的六轴向量全 0、`bag` 在合并分支不追加 ⇒ 步骤⑦ `!gain.is_gain()` 仍返 None。
即「uses 守卫」与「is_gain」在 ty13 上重叠，**无假阳性风险，但也没有判据能区分二者**。

---

### F4 背包合并缺口 —— 绿

在 `/tmp/p6rev` 的 `tests/pure_pickup.rs` 末尾追加两条**临时**判据（仓库未改，跑完即弃）：

```rust
// 32 槽、types[0]=19/uses[0]=1/stack=1，层内再放 ty19 + ty32
let mut ps = s0_spec(300);
ps.item_types = vec![19u8, 0u8 × 31];  ps.item_uses = vec![1u8; 32];  ps.item_stack_size = 1;
```

实测：

```
[F4]  ok=true trunc=false out_hp=500 pure=false reason=replay_bag_mismatch ty=0 stack=1
      types=[19,0,…] uses=[2,1,1,…]
[F4b] ok=true out_hp=500 pure=true  reason=adopted stack=1     ← 空背包 + ty19 纯追加
test result: ok. 2 passed
```

⇒ 合并路径确实**保守拒绝**，没有错误采纳。**自述不足**：`expected_after_all` 只在
`player_in.item_types` 里往后写；链上默认 `item_types=[]`（len 0）时，先触发的是
`pure_pickup.rs:685` 的 `bag_no_room`（`n0+bag_need > player_in.item_types.len()`），
我第一版实验（不给槽位）实测就是 `reason=bag_no_room`——所以「合并 ⇒ replay_bag_mismatch」
只在**槽位已预分配**时成立。

---

### F5 回放逐轴断言是否真承重 —— 黄

回放断言块：`crates/game-solver/src/pure_pickup.rs`
`:758 replay_hp_mismatch` / `:766 bag_no_room`（期望算不出来）/ `:770 replay_stat_mismatch` /
`:776 replay_key_mismatch` / `:788 replay_bag_mismatch`（`expected_after_all` 在 `:507`）。

变异（只改 `/tmp/p6rev`，跑 `cargo test --release -q -p game-solver --test pure_pickup`）：

| 变异 | 结果 |
|---|---|
| hp 上界改 `in_hp + Σ/2`（`:677`） | **9 failed / 4 passed**（判据 1/2/3/4/5/7/8/11/A1 全红，报 `replay_hp_mismatch ≠ adopted`）⇒ hp 轴**承重** |
| 键比较改 `if false`（`:775-779`） | **13 passed / 1 ignored（绿）** ⇒ 无判据覆盖 |
| 背包比较改 `if false`（`:786-790`） | **绿** ⇒ 无判据覆盖 |
| atk/def 比较改 `if false`（`:769`） | **绿** ⇒ 无判据覆盖 |
| `expected_after_all` 不再累 atk/def | **1 failed**（`monotone_floor_collects_keys_and_stats`，`tests/pure_pickup.rs:654`） |

⇒ 判据 8 断言的是 `ex.out_player.atk/def/keys`（**引擎返回值**），它**证明不了**比较守卫还在工作；
「期望侧建模」被覆盖，「比对侧守卫」三条**零覆盖**。要求里的「钥匙少拿但 hp 对」场景在当前数据上
**构造不出来**：`EffectParams::Key`（`effect_registry.rs:223-227`）是无条件 `+= 1`，没有上限/条件分支，
所以 real 数据里 key 轴不会与采样不一致——这解释了为什么删掉守卫仍全绿，也说明该守卫是纯防御。
（结论等级：黄＝「实现有断言但无判据承重」，不是错，也不是假绿。）

---

### F6 旧判据载荷变更 —— 绿

`crates/game-solver/tests/pure_pickup.rs:388-412` `pure_pickup_fails_closed_when_pickup_not_certified`：

```rust
// 正对照（Poison::Free，两枚纯加血）⇒ adopted / pickups=2 / out_hp=500
assert_eq!(st_free.reason, "adopted", "反证：两颗药全是纯加血时必须采用");
// 载荷（Poison::Uncertified = ty13）
assert!(!ex.pure_pickup, "会让字段下降的拾取物 ⇒ 必须回退通用搜索");
assert_eq!(st.reason, "pickup_not_certified");
assert_eq!(st.reason_ty, 13);
assert_eq!(st.hp_expected, 0, "未采用时不许留下上界残值");
assert_eq!(st.hp_replayed, 0);
```

非永真：若 ty13 被误认证，`!ex.pure_pickup` 与 `reason` 两行立刻红（变异 E 的组合实验证明只有
「移除 uses 守卫 + 探针不预置 13」才可能翻转）。`hp_expected==0` 是**真实强度**（防「采用了但坏血」中间态）。
`ty26` 正对照（同函数内 `Poison::Free` 用的是纯加血药，另一处 `non_monotone_pickup_still_fails_closed`
里 `ty26 ⇒ adopted, yellow=1`）确实断言「采纳」，不是空转。

**附注（归因不唯一）**：ty13 同时满足「合并 uses 下降」与「六轴零增益」两个拒绝条件，
测试只能证明 `Some→None`，证不出是**哪条**拒的（见 F3 变异 E）。

---

### F7 f51..f54 回归 —— 绿

父提交独立 worktree + 独立 target：

```
git worktree add /tmp/p6base 0a244ff
ln -sfn /home/peng/workspace/xiyou_remake/assets/raw /tmp/p6base/assets/raw
CARGO_TARGET_DIR=/tmp/p6base-target cargo build --release -p game-solver --bin solver   # 5.85s
```

两侧 `run --out` 产物按 15 个共字段（`floor,entry,exit,kind,route,in_hp,out_hp,in_player,out_player,
in_flags,out_flags,ents_in,ents_out,ok,truncated`）逐字段比：

```
common-field diffs: 0
parent: status=capped floors=6 total_steps=275 blocked=floor 3 reason=truncated layer_ok=Some(false) layer_truncated=Some(true)
HEAD  : status=capped floors=6 total_steps=275 blocked=floor 3 reason=truncated layer_ok=Some(false) layer_truncated=Some(true)
```

`f54 out_hp=3227` 保存（hp 链 300→247→427→827→3227→3227→3227）；自述「15 个共字段逐字段相同」**成立**。

---

### F8 基线诚实性 —— 黄

- `gates/cli.py:263-332` 的 test-ratchet 实现：`cargo test --workspace --release --no-run
  --message-format=json` 收集可执行文件，再逐二进制 `[exe, "--list", "--format", "terse"]`，
  统计以 `": test"` 结尾的行 ⇒ **`#[ignore]` 行同样以 `": test"` 结尾，会被计入**。
- 实测：`target/release/deps/pure_pickup-* --list` = **14**（= 13 会跑 + 1 条
  `monotone_floor_f3_hits_closed_form`，`tests/pure_pickup.rs:778-779`）。
- `data/status/baselines/rust-test-counts.json:83` `"total": 1183` **未被两个 commit 修改**；
  总闸实测 `test-ratchet — total=1193(基线1183) 无回退` ⇒ 自述的「1183→1193(+10)」是**实测总数**，
  不是基线 bump（基线仍是 1183，靠 `≥基线` 通过）。两种做法都不算回退，但「+10」的表述容易被读成改基线；
  依任务口径「`#[ignore]` 被算进基线 ⇒ 记黄」成立：这条『f3 必须出解』的**证伪记录**会永远占一个名额，
  永远不跑却顶住 ratchet。

---

### F9 门禁 / 夹带 / 并发 —— 绿

```
git show --name-status 64ae5f1 → M pure_pickup.rs / solve_local.rs / witness.rs / tests×2 / 一份 findings md
git show --name-status fb5c15b → M effect_registry.rs / pure_pickup.rs / tests pure_pickup / 同一份 findings md
```

七个文件全部落在任务声明范围内；**无** `gates/`、`original/`、`reference/`、`data/naming/`、
`data/status/baselines/` 改动（⇒ 也没偷改基线）。

```
git reflog -12   → HEAD@{0} fb5c15b (commit) / HEAD@{1} 64ae5f1 (commit) / HEAD@{2} 0a244ff …  线性
git branch -vv   → * modern fb5c15b [origin/modern: 领先 13]      (未 push 属实)
git status       → 仅 5 份未跟踪的 docs/findings/reviews/*.md（复核产物），源码零脏改
```

无 amend、无 rebase、无第二个并行 agent 的提交混入。

**总闸汇总行（A 级，串行重跑，我自己执行）**：

```
$ python3 gates/cli.py check
[✓ PASS] original-integrity — 1 个归档校验通过
[✓ PASS] assets-manifest — 与基线一致 {'entries': 138, 'classes': 2, 'maplv': 55, 'sprite': 55}
[✓ PASS] reference-seed-integrity — 8457 行，sha256 对照基线通过
[✓ PASS] knowledge-format — 64 条目合规
[✓ PASS] docs-links — 全部相对链接有效
[✓ PASS] naming-ledger — 全部记录带证据指针
[✓ PASS] rust-tests — cargo test --release 全绿
[✓ PASS] test-ratchet — total=1193(基线1183) 无回退
[✓ PASS] table-content-hash / table-evidence / state-machine-anchors / asset-truth-manifest / state-baseline
== 汇总 ==
PASS=13 FAIL=0 SKIP=0
EXIT=0
```

（首次尝试在**我并发跑变异测试**时卡在 `[cargo] <defunct>` + 0% CPU，是 target 目录锁争用，
不是仓库问题；三个 oracle 测试单独串行跑也是 5/35/45 passed。⇒ 门禁结论以串行这次为准。）

---

### F10 「下一个撞界点」判断 —— 黄（方向不认同）

实现者判断：「f3 需要建模战斗与踩格臂，不是继续扩拾取」。**前半句不成立**：

1. f3 的 HP 最优**根本不打怪**：9 枚免费拾取 = +600 ⇒ **3827**（我穷举 2^7 击杀集得上界表，§3）。
   为剩下的 200 hp 去杀 6 只怪要付 1908 hp，任何理性搜索都不会选。
2. f3 的**第一道墙是脚本不是战斗**：入口口袋唯一出口 `ty83@(5,11)` param=27，
   脚本内容（`crates/game-core/tests/fixtures/level_scripts.txt` 第 28 行 = `this.n[27]`，
   已用 `reference/seed/a.java:550` 的类数组逐字核对）：
   `TAK_113_117 TAK_133_135 DES_84_6_11 GIN_0_13`
   —— 两句对话 + `DES`（f3 上无 ty84 ⇒ 空操作）+ `GIN_0_13`（给钱或给道具，B 级，见§6）。
   `probe_gate(f3,27) = false` 说明这扇门**未开、脚本会跑**。
3. 真正需要新能力的是「**一次性开门脚本的可证中性建模**」（和上一片对 ty83 param=64 的做法同源，
   只是这次脚本有副作用），而不是把战斗塞进闭式解。
4. 若目标是**证明** 3827 最优（而不是只给出一个下界），用「代价下界支配」即可：
   任何要杀怪的路线，其代价下界 = `probe_bump` 的确定性损血（无随机、无反击），收益上界 = 剩余未拾取的
   `Σhp`。本例 6 杀集下界 1908 > 剩余收益 200 ⇒ 一步剪掉，**不需要战斗搜索**。
   只有出现「多段战斗后属性质变（如换装、等级触发）」时才需要真战斗搜索 —— f3 数据里没有这种结构。

---

## 3. F2 专项：f3 的必须击杀集与真实上界

### 3.1 几何骨架

```
      x=0         1         2
 0 #############
 1 #pp#ppp#.#.p#
 2 #.p#ppp#.DM.#
 3 #M.#ppp#.####
 4 #D###.##.#..#
 5 #..M...M....#
 6 #D##...#.####
 7 #M.##.##.#.p#
 8 #.p##.##.DMp#
 9 #pp##.##.####
10 #####.##M#..#
11 #......#.D..#     ← entry (2,11) 在这个一格宽的口袋里
12 #############
```

`#`=不可走 `.`=可走 `M`=怪 `p`=拾取 `D`=门(ty1)；
口袋 = `{(1,11),(2,11),(3,11),(4,11)}`，**唯一出口 = `(5,11)` = `ty83`**。

### 3.2 每枚拾取物的「最小击杀集」（2^7 穷举 BFS）

| 拾取物 | 坐标 | 收益 | 最小击杀集 | 需杀数 | 损血(atk10/def10) |
|---|---|---|---|---|---|
| ty26×5 + ty32×3 + ty28×1（自由区 9 枚） | 顶部房间 `(4,1)…(6,3)` | +600（3×200；ty26/ty28 的 hp 增量为 0） | ∅（但**必须踩** `ty83@(5,11)`） | 0 | 0 |
| ty26@(1,1), ty31@(2,2), ty30@(2,1) | 左上角 | +50 / +1def | ty45@(1,3), ty43@(3,5) | 2 | 638+112=750 |
| ty26@(2,8), ty31@(2,9), ty29@(1,9) | 左中 | +50 / +1atk | ty44@(1,7), ty43@(3,5) | 2 | 384+112=496 |
| ty26@(11,8), ty31@(11,7) | 右下 | +50 | ty45@(10,8), ty41@(7,5) | 2 | 638+24=662 |
| ty31@(11,1) | 右上 | +50 | ty43@(10,2), ty41@(7,5) | 2 | 112+24=136 |

（`ty26` 的黄钥 hp 增量为 0；自由区 9 枚 = ty26×5 + ty32×3 + ty28×1，hp 收益 = 3×200 = **600**。）

### 3.3 真实上界表（穷举 128 个击杀集，按 HP 排序）

| 策略 | 拾取数 | Σhp | 击杀数 | 损血 | HP 上界 | 判定 |
|---|---|---|---|---|---|---|
| **零战斗**（踩 ty83@(5,11)，只捡自由区） | 9 | 600 | 0 | 0 | **3827** | **真最优** |
| +只杀 ty42@(8,10) | 9 | 600 | 1 | 50 | 3777 | 劣 |
| +杀 ty41+ty42 | 9 | 600 | 2 | 74 | 3753 | 劣（＝实现者独立探针实测值） |
| 全捡 18 枚（任务书要求的 4027 路线） | 18 | 800 | **6** | **1908** | **2119** | **远劣且不可达 4027** |

⇒ **任务书要求的 `out_hp=4027` 在这个数据上不存在**：不是「难搜到」，是**数学上不可达**（缺口 1908）。

> 【2026-11-19 更正 · 上表「零战斗 3827 = 真最优」已被证伪】
> 批8 把 f3 的层预算放到 **1M 档**（`F3_LOCAL` 分支 `SOLVER_LAYER_LIMIT=1_000_000`）实测
> `out_hp=3877 > 3827`（复核 F3 记录；该档**仍 `truncated=true`**，所以 3877 只是**下界**，
> 不是最优证明）。⇒ 3827 **不是**真最优；`4027` 是否可达**仍未证**（理由见 §1/§2 的两条更正：
> 730/730 下损血项为 0，但 1908 不是有效理由）。
> 同一句「数学上不可达（缺口 1908）」的「缺口 1908」随之失效，改为「缺口量与可达性均未证」。
实现者的证伪结论**成立**，且比它自述的理由更强（它只证明了「全捡要杀 6 只、会掉血」，
没证明「最优解根本不打」）。

### 3.4 推论

- f3 的「战斗必要性」= **0**；200hp 的边际收益买不起任何一只怪的损血（最便宜 ty41 = 24hp，
  但它背后还挡着 112hp 的 ty43）。
- f3 的「脚本必要性」= **1**（唯一出口的 ty83 param=27）。
- 因此 P6 定理「不覆盖带踩格臂的层」在 f3 上是**正确拒绝**，但拒绝理由的可观测值
  `step_arm_hazard_reachable/83` 只说了「有可达步臂」，没说「这一层其实零战斗也能拿到 600」。
  这是**可观测性缺口**（不是错判）。

---

## 4. 变异实证表（全部在 `/tmp/p6rev`，跑完即恢复）

驱动：`/tmp/mut.py`（`shutil.copy` 备份 → 定点替换 → `cargo test` → 恢复）。
基线：`cargo test --release -q -p game-solver --test pure_pickup` = **13 passed; 0 failed; 1 ignored**。

| # | 变异点 | 文件:行 | 期望（若断言承重） | 实测 |
|---|---|---|---|---|
| A | 闭式 hp 上界 `in_hp + Σ/2` | `pure_pickup.rs:677` | 判据 1（f54 3227）红 | **9 failed / 4 passed**（1,2,3,4,5,7,8,11,A1 全红） |
| B | 三钥比较 `if false` | `pure_pickup.rs:775` | 判据 8 红 | **13 passed（绿）⇒ 不承重** |
| C | 背包比较 `if false` | `pure_pickup.rs:786` | 任一判据红 | **13 passed（绿）⇒ 不承重** |
| D | atk/def 比较 `if false` | `pure_pickup.rs:769` | 判据 8 红 | **13 passed（绿）⇒ 不承重** |
| E | 删核心合并 `uses` 下降守卫 | `effect_registry.rs:613-616` | 判据 9 / 探针红 | **core 探针 2 passed（绿）**：ty13 仍被 `is_gain` 拒 |
| F | `expected_after_all` 不累 atk/def | `pure_pickup.rs:520-521` | 判据 8 红 | **1 failed**（`monotone_floor_collects_keys_and_stats`，`tests/pure_pickup.rs:654`） |
| G | 背包合并场景（临时判据） | `/tmp` 追加 | 必须保守拒绝 | `reason=replay_bag_mismatch`，`pure=false`；正对照 `adopted` |

对照实现者自述「变异 3 发全红」：①② 我复现（A 红、E 的等价物在**探针层**不红，在**判据 9** 上也不红，
除非同时改探针夹具）；③「回放跳过 atk/def ⇒ 判据 8 红」——**只在「期望侧不累加」时成立（F）**；
若照字面「删掉比较」（D），**全绿**。这是自述与实测的第一处实质偏差（见 §5）。

---

## 5. 自述与实际不符处

| # | 自述 | 实测 | 严重度 |
|---|---|---|---|
| 1 | f3「G1/G2a/G2 三闸都失败」 | **G2a 通过**（`probe_gate(f3,0)=true`）；G2 先响，G1 轮不到 | 中（归因错，结论不受影响） |
| 2 | f3 必须建模**战斗**才能打 | 零战斗最优 3827 已穷举证明；真墙是 `ty83@(5,11)` 的**脚本** | 中（F10 方向） |
| 3 | 变异③「回放跳过 atk/def ⇒ 判据 8 红」 | 删比较 = **全绿**；只有「期望侧不累加」才红 | 中（变异设计含混，掩盖了三条守卫无覆盖） |
| 4 | 「合并会 `replay_bag_mismatch`」 | 仅**槽位预分配**时如此；链上默认 `item_types=[]` ⇒ 先 `bag_no_room` | 低 |
| 5 | 文档 §1 G1 行「f1 实测 41、f53 实测 51」 | 实测 `f1=step_arm_hazard_on_entry/83`、`f2/f51/f52/f53=step_arm_hazard_reachable/83`；与同文 §7.2 自相矛盾 | 低（docs 口径） |
| 6 | 文档 §5 判据表行号 | 整体漂移 ~20 行（如判据 5 实为 `tests/pure_pickup.rs:389`，文档写 `:369`） | 低 |
| 7 | 「六轴 `>=0` 守卫在真实数据上无反例（防御性代码）」 | 成立；同理**合并 uses 守卫**也是防御性（变异 E 绿） | 信息 |

**自述成立的部分**（逐条核过，不需更正）：f3 的 7 只怪坐标/35 枚实体/两扇 `ty83` 全对；
f51/f52/f53 → `step_arm_hazard_reachable/83`、f1 → `step_arm_hazard_on_entry/83` 全对；
f54 18 枚全 HpUp 与我独立 dump 一致（`-- 54 7,9` 实测：18 枚 = ty32×10 + ty31×8，`sum_pickup_hp=2400`、`min-kill=0`、`bound=5627`）；既有判据 5 载荷 ty26→ty13 且强度未减（见 F6）；
f51..f54 15 字段零漂移（见 F7）；`solver run` 汇总行逐字不变。

---

## 6. 未验证清单

1. **F8 的「基线该不该 bump」未定**：我证实了「实测 1193 / 基线 1183 / 闸门 PASS」，
   但「作者是否有意保持基线不动」只能从 diff 推断（基线文件确实未被改）。若下一位复核者认为
   基线应随计数上调，那是一条独立的基线纪律问题，不是我这次能判的。
2. **`GIN_0_13` 语义**：`docs/findings/script-dsl-semantics.md` §A2 推断 `GIN_0_n`=给钱（B 级未定），
   而 `docs/superpowers/proposals/2026-11-17-solver-correctness-remediation.md:272` 写「给 item13」。
   两份文档互相矛盾 ⇒ f3 开门脚本的副作用到底是 gold+13 还是 item13 未定。
   但**无论哪种都改变 `gold` 或背包**，不改变「G2 拒绝正确」与「HP 最优 3827」。
3. **`ty83` 脚本会在什么条件下重入**（`flags` 置位语义）未建模：我只证明了 `probe_gate(27)=false`，
   没证明「脚本跑一次后就标 flags、二次踩不再触发」。
4. **商店 `ty77@(11,4)` param=278 的可购买内容**未查：若能买 HpUp，f3 上界可能高于 3827
   （但闭式解不建模商店，故不影响 P6 判定）。
5. **`ty29/ty30` 的 atk/def 增量是否与当前属性档有关**（是否 tier 依赖）未逐型核对；
   若有档位依赖，则「回放 atk/def 守卫」在真实数据上会变成**可达**（届时它就不是纯防御了）。
6. **`modern` 分支另外 11 个未 push commit** 不在本次范围（只复核两片）。

---

## 7. 两句结论

1. **P6 能不能进 P5：能（有条件）**——两片都是纯加/纯内部改造，f51..f54 零漂移，f54 3227 保存，
   `certified_monotone_pickup` 的健全性（无漏轴、无假阳性、真实下降动作会被拒）已被我独立证实；
   进 P5 前建议只清三条**黄**：`#[ignore]` 退出 ratchet 计名额（或明确写入基线豁免）、
   给三钥/背包/atk-def 三条回放守卫补一条能被变异打红的判据、把 doc §1 的 G1 行与 §5 行号按实测更正。
2. **f3 该用哪种手段打：先「脚本建模 + 代价下界支配」，不要上战斗搜索**——
   f3 的真墙是 `ty83@(5,11)` 的一次性开门脚本（`TAK×2 + DES + GIN_0_13`），
   而 HP 最优 3827 是**零战斗**（9 枚免费拾取）；用 `probe_bump` 的确定性损血做代价下界、
   剩余拾取的 Σhp 做收益上界，一步就能剪掉所有「打怪换药」的分支。
   只有当出现「战后属性质变」的层（换装/升级改变后续可拾取集）时，才需要真正的战斗搜索。
