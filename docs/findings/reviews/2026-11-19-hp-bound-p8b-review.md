# 批8b 独立对抗性复核：`bcaceaf`（hp 上界生成通道闸门）+ `70f9436`（p6 三处更正）

- 复核对象：`bcaceaf`（7 文件 +658/−17，和 `70f9436`（文档 1 文件 +487），父提交 `ba9783d`
- 复核方式：**不看自述当证据**。父/HEAD 各建 `/tmp` 副本独立编译；变异；语料变异；用**父提交真代码**
  复算旧口径；逐 face 给判定 + 复现命令 + 关键输出。
- 只读边界：本复核**只写本文件**，未改代码、未 commit、未 push（`git status` 里 7 份既有未跟踪
  review 仍全部 `??`）。

> 【2026-11-19 复核·本片结论】计数 **绿 8 / 黄 2 / 红 0**。核心机制（击杀生成 / 脚本生成 /
> 商店 repeat / PickupItem / `parse_slot` 双算面）**都成立**，且 f2 活反例、f10/f20/f35/f40 的
> Off 面均能独立复现。两处黄是**泛化断言**而非今日缺陷：①`plan()` 的「槽位一次性」论证对
> **槽 32** 不成立（该槽 flag 永不置位 ⇒ 可重复触发），而模块文档写了「将来若加 hp op，本论证
> 仍成立，不需改代码」；②`parse_slot` 改动被描述成修「双算」，实际修的是**反方向**（置位后
> `konst` 消失导致的**低估**）——两处都不改变「今日语料下解不丢」的实测，但都会在下次改语料/
> 改判据时变成陷阱。

---

## 0. 复核环境与可用命令

```bash
# 父提交副本（用来跑「修复前」真代码，不是手算重建）
rm -rf /tmp/x && mkdir -p /tmp/x && cd /home/peng/workspace/xiyou_remake \
  && git archive ba9783d | tar -x -C /tmp/x && ln -s "$PWD/assets/raw" /tmp/x/assets/raw
# HEAD 副本（变异用，保持仓库干净）
rm -rf /tmp/y && mkdir -p /tmp/y && git archive HEAD | tar -x -C /tmp/y \
  && ln -s "$PWD/assets/raw" /tmp/y/assets/raw
# 两边分别用独立 target 目录，避免互相污染
CARGO_TARGET_DIR=/tmp/xt cargo ...   # /tmp/x（父）
CARGO_TARGET_DIR=/tmp/yt cargo ...   # /tmp/y（HEAD）
```

关键锚点（行号以 HEAD 为准）：

| 位置 | 内容 |
|---|---|
| `crates/game-solver/src/hp_bound.rs:619-637` | `plan()` 1b：活 cat8 且 `drop_gain>0` ⇒ `Off("post_kill_hp_spawn", ty)`（`:633`） |
| `crates/game-solver/src/hp_bound.rs:427-436` | `certify_script_spawn`：生成物 `spawn_gain>0` ⇒ `Off("script_hp_spawn", slot)`（`:434`） |
| `crates/game-solver/src/hp_bound.rs:388-403` | cat16：`param & SHOP_REPEAT != 0` 且 `AddHp` ⇒ `Off("shop_repeatable", ty)`；`PickupItem` 走 `hp_gain_of_type` |
| `crates/game-solver/src/hp_bound.rs:139` | `const SHOP_REPEAT: i32 = 1 << 10` |
| `crates/game-solver/src/hp_bound.rs:518-541` | `parse_slot`：`RES`/`GUT`/`LAY`（`LAY` ⇒ `ended = true`，`:538-541`） |
| `crates/game-solver/src/hp_bound.rs:89-96` | 模块文档「已知不健全边界」，`:96` 是「不需改代码」那句 |
| `crates/game-core/src/scene_script.rs:57-64` | `on_line_end`：`if cur != 32 { script_line_flags[cur] = true; }` |
| `crates/game-core/src/rules/gate.rs:96-104` | `evaluate` case 7：`from_walk && !cross_floor_snapshot(23,12)`，命中后 `set_flags.push(32)` |
| `crates/game-core/src/rules/gate.rs:107-114` | `evaluate` case 32：`flag(7)` 为真 ⇒ 挡；否则 `!from_walk && cross_floor_snapshot(23,12)` |
| `crates/game-core/src/script_ops.rs:67-73` | `op_lay` = `save_floor_state` + `set_pending_floor(floor)` + `set_transitioning(true)` + `phase=5` |
| `crates/game-core/src/script.rs:403-415` | `ScriptOp::Res` 仅 `difficulty_index==0`：三钥清零 / `gold=4` / `atk=def=10` / **`hp=400`** / 层史 1..3 |

---

## F1 闸门完备性（片头号面）—— 黄

**做法**：独立枚举「搜索中途能凭空多出 hp」的全部通道（不照抄自述的 7 条），逐条核对
`plan()`/`parse_slot`/`certify_entity` 是否覆盖；再对「看起来漏了」的两条做实证。

| # | 通道 | 引擎锚 | 本片实现是否覆盖 | 我的实证 |
|---|---|---|---|---|
| ① | 击杀 post-anim 组 | `rules/boss.rs` `EVENT_SPAWNS` 组 7（69@35 ⇒ ty32×3）、`post_victory_event_of` | ✅ 1b `Off` | 父代码复现（见 F4） |
| ② | boss 掉落 / 递归掉落 | `solve_local.rs:1193 apply_kill_drops` ← `event_of`/`post_victory_event_of` | ✅ 1b + `DROP_DEPTH_MAX=4` 深链 ⇒ `Err` | 深层链无数据；`drop_gain` 递归到 :depth>4 即 Off |
| ③ | 脚本 `CES_`/`ADD_`（含跨层 ADD、GUT 尾链） | `script_ops.rs:243 op_ces` / `:126 op_add` 均 `spawn(...,0,...)` | ✅ `certify_script_spawn(ty)`，`param` 恒 0 与引擎一致 | 我逐字核了引擎参数（两处 `spawn` 都传 `param=0`，与 `spawn_gain(ty,0,…)` 对齐）；A3 只测 CES，ADD 面语料 `ADD_2_72_11_10` 在 f2 生成 ty72（`event_of(72,2)=None` ⇒ drop 0），今日无 hp 增量 |
| ④ | 商店 `AddHp`/`PickupItem`/`flags&4` 可重复买 | `scene.rs:2522 apply_merchant_offer`（`:2586` AddHp；收尾块 `if flags & 4 == 0` 才清价/移除） | ✅ repeat 位 + AddHp ⇒ Off；PickupItem 计入 | M3/M4 变异真红（见 F7） |
| ⑤ | `GIN_0_x` 授 hp 道具 | `script_ops.rs:77-80` | ✅ `hp_gain_of_type` 计入 `konst` | 语料 `GIN_0_13/19` 均 0 增量 |
| ⑥ | 门脚本自身 `RES`/`ROS_3` hp 写 | `script.rs:403-415`（RES=400 easy）/`script_ops.rs:294`（ROS_3 绝对写） | ✅ `abs_write`/`res_write` 计入 | **我一度判红，实证后改判黄**（见下） |
| ⑦ | 宝箱 `try_open_chest` | `scene.rs:799`；`advance_open_group_paint`（`:4473`）只 `remove_group_at_cell_by_type(...,4,...)` | ✅ 只移除、不生成 | 复核上轮结论仍成立（重读代码，非复述） |
| ⑧ | 恢复规则 / 换层携带 / 陷阱生成 | `restore_rules.rs`（f2 ty84、f39 38/40/7/8）、`carry_monsters_to`、`fire_trap_climb_up`(ty60) | ✅ 无 hp 型 | f12 实测**根本没有 ty78/ty83** ⇒ `ParamRewrite` 今日是 no-op |

**改判记录（我自己先判红后改黄）**：`parse_slot` 在 `LAY` 处 `ended = true`（`:538-541`），而
引擎侧 `LAY` **不终止脚本**：`op_lay` 只 `set_pending_floor` + `set_transitioning`（`script_ops.rs:67-73`），
`run_instruction` 一条一条执行、`ScriptOp::Lay{cursor: e+1}` 照样推进游标（`script_ops.rs:414-419`），
且仓库里已有测试 `scene_solver_api.rs:555-575 smoke_fixture_slot0_execute_readback` 机械驱动槽 0
到底并断言 `sc.player.hp == 400`、`current_script_index == 1`（`GUT_1` 生效）。语料槽 0 正好是
`… LAY_2 ROS_1_4_7 ROS_2_0 ROS_2_6 RES_0 GUT_1`，**且 f3 的 `ty83 param=0` 就在入场格 (2,11)**。
这条看起来是活漏检（300hp easy 踩上去 +100，界里记 0）。于是我用两种独立实证否证了它：

```bash
# (1) 把 RES_0 从语料槽 0 删掉，f3 20k 档结果不变
cd /tmp/y && python3 - <<'EOF'
p='crates/game-core/tests/fixtures/level_scripts.txt'; s=open(p).read()
assert s.count('RES_0 GUT_1')==1; open(p,'w').write(s.replace('RES_0 GUT_1','GUT_1'))
EOF
CARGO_TARGET_DIR=/tmp/yt cargo run -q --release --example probe_res   # 我写的探针
# 输出（有 RES_0 / 删掉 RES_0 两种语料，逐字相同）：
# start(2,11): max_hp=426 min_kills=0 truncated=true route_len=30
# start(4,11): max_hp=426 min_kills=2 truncated=true route_len=28
# (2) 只留 (1,11) 出口，强制路线穿过 (2,11)
# exit=(1,11) only: reachable=false max_hp=None route=[]
```

结论：**搜索从不踩这扇门**；强制穿过时整层直接 `reachable=false`——因为 `op_lay` 已经
`set_pending_floor(2)`+`set_transitioning(true)`，这扇门在本层模型里是**换层门**，不是层内加血点。
所以「槽 0 的 LAY 之后计入 0」与求解器模型自洽，且 `416` 上界不会被低估。我先前那条红撤回。

**仍留两处黄（泛化面，非今日缺陷）**：

1. **槽 32 的「一次性」论证不成立**。模块文档 `:89-96` 写：「基谓词恒 `!v.flag(param)` ⇒ 置位后
   该槽永不再触发 …… 将来若加 hp op，本论证仍成立，不需改代码」。但 `on_line_end` 对 32 是
   **特殊跳过**（`scene_script.rs:57-64`：`if cur != 32 { … = true }`，`script.rs:480-489` 同），
   而 `evaluate` case 32 的判据只看 `flag(7)`/`from_walk`/跨层快照（`gate.rs:107-114`），
   case 7 只把 32 当作**别人置位**的 flag（`gate.rs:96-104`）。因此只要 `flag(7)==0`，param 32 的门
   **可以被反复撞击重触发**；界里只记一次 ⇒ 若该槽含 hp op 就是**无有限上界**的低估。实证（把槽 32
   换成 `GIN_0_32`，配一扇 param=32 的门）：

   ```
   plan=On root_bound(300, ents, flags=0)=500
   after-1-fire state: hp=500 flags=0 bound=700
   should_prune(hp=500,best=1500)=true
   after-3-fire state: hp=900 flags=0 bound=1100
   should_prune(hp=900,best=1500)=true
   ```

   即「每撞一次 +200、界始终只 +200」——撞两次就到 900，界却按 1100 判「< 1500 可剪」，
   而真实可继续撞到 1500 以上。今日语料槽 32 = `TAK_79_79`（无 hp op），且要触发还需
   `cross_floor_snapshot(23,12)` 为真 ⇒ **潜伏**，不是现状错误。但「不需改代码」是**假承诺**：
   修法只有 3 行（`param==32` 且 `konst/res_write/abs_write>0` ⇒ `Off("slot32_repeatable",32)`），
   或把该句改成「槽 32 永不置位 ⇒ 若加 hp op 必须加闸门」。

2. **kill-hook 直载脚本（`load_level_script(20/21/22)`）的 flag 不在 `parse_slot` 视野**：
   语料 20/21/22 = `TAK_40_40`/`TAK_59_59`/`TAK_75_75`（无 hp 写），今日无害；但这是**另一条
   不经 ty83 的脚本执行入口**（`scene.rs:3655-3702` 击杀死门 `73/74/75` 直调），
   `plan()` 只按 ty83 param 收集槽 ⇒ 若这些槽将来含 hp 写会漏检。建议登记进边界清单。

结论：**今日语料的通道枚举完备**（①–⑧ 逐条有着落，没有找到能在当前数据上剪掉真实最优的反例）；
不健全的是**对语料演化的承诺**（槽 32 + kill-hook 槽）。

---

## F2 闸门是否过宽 —— 绿

- 4 层 Off = f10(ty75)/f20(ty74)/f35(ty69)/f40(ty73)，正是 `boss_order()`=[75,74,68,67,69,71]
  里**表内有 hp 掉落的四只**；其余 50 层 On。sweep 独立复跑逐字一致：

  ```bash
  cargo run -q --release -p game-solver --example hp_bound_sweep
  # {"on": 50, "post_kill_hp_spawn": 4}
  ```

- **A2 的严格性不是洁癖**：M2（`if d > 0 && ord.pickup_hp.is_empty()`）只红 A2 —— 说明「根态已有
  ty32 就不再 Off」会放过 f10/f20/f40 这类「根态已有药、但击杀仍生 hp」的层，是真低估。
- **代价实测 ≈0**（父代码同探针，f10/f20/f35/f40 从 (1,11) 冷启动，剪枝开/关）：
  `f10 1ms vs 0ms`、`f35 5ms vs 0ms`、`f20/f40 两档都 0ms 且都 reachable=false`。
- **证据缺口（如实记）**：`f51..f54` 逐字节相同**不能**证明这 4 层「解不变」——`solver run` 在第 3 层
  就截断，根本走不到 f10/f20/f35/f40。故现状只能主张「Off 不损正确性（不剪就不可能剪错）」+
  「入口态代价可忽略」，**深层（击杀后大 hp 态）的剪枝收益未量化**。这是精度风险，不是健全性风险。

---

## F3 `parse_slot` 「双算修复」的正确性 —— 黄（修对了，归因说反了）

**改动**：父版 `certify_entity(a[1],0,floor,shops,0,&mut t.konst)?` / `(a[0],…)`（把生成物 hp 直接
加进槽项 `konst`）⇒ HEAD 改 `certify_script_spawn(...)`（生成物 gain>0 就 `Off`）。

**父版的真缺陷是低估，不是双算**。槽项 `konst` 只在 `flags[slot]==0` 时计入 `bound`；门一触发
`on_line_end` 置位 ⇒ `konst` 消失，而生成物不在 `pickup_hp`/`drop_hp`（表只在层入口从根态
`ents` 播种）⇒ `entity_gain=0` ⇒ 界**塌回 hp**。用**父提交真代码**（`/tmp/x`，语料槽 62 临时改成
`CES_32_1_1`，层里只放一扇 `ty83 param=62`）复现：

```
plan(1,0,...) = On            # 父版无闸门
bound(hp=300, ents=[], flags=0)            = 500    # 300 + 槽 62 的 200
bound(hp=300, ents=[ty32], flags=1<<62)    = 300    # 触发后 konst 消失、生成物按 0 计
should_prune(best=301, hp=300)             = true   # 剪掉真实可达 500 的态
```

把 HEAD 的 `hp_bound.rs` 拷进同一副本再跑：`plan = Off("script_hp_spawn",62)` ⇒ 修复有效。
**「双算」确实存在但是安全方向**（根态已有 ty32 时：`entity_gain(32)=200` + `konst=200` =
多算 200，只会少剪），所以 commit message 的「原写法是同一类错误的另一面（会双算）」**不是**
被修掉的那个缺陷；真正被修掉的是反方向的塌陷。

**测试覆盖缺口**：A3 只断言 `Off`，没有像 A1 那样把「旧口径 bound 塌陷」的算术钉进测试
（A1 钉了 `bound(200,[ty32×3],MAX)=200` + `should_prune(301)=true`）。建议补一条同构断言：
`bound(hp, ents=[spawn], flags=1<<slot)` 的塌陷值 == hp，否则将来有人把闸门换成「计入生成物」
就会**静默**退化（今天只有 A3 的 `Off` 会红）。

---

## F4 A1–A5 判据的有效性 —— 绿

逐条真跑变异（`/tmp/y`，`CARGO_TARGET_DIR=/tmp/yt`，每条还原后重跑确认 9 passed）：

| 变异 | 改动 | 结果 |
|---|---|---|
| M1 | `if d > 0` → `if false && d > 0` | **A1+A2 红**：`left ("on",0)` vs `right ("post_kill_hp_spawn",69)`(:865) / `(…,75)`(:908)，7 passed 2 failed |
| M2 | `if d > 0 && ord.pickup_hp.is_empty()` | **只 A2 红**，8 passed 1 failed |
| **M3（我加）** | `param & SHOP_REPEAT != 0` → `if false && …` | **只 A4 红**：`left ("on",0)` / `right ("shop_repeatable",78)`，8 passed 1 failed |
| **M4（我加）** | `ShopEffect::PickupItem{item} => *out += …` → `{ item: _ } => {}` | **只 A5 红**（`right: 500` vs `left: 300`），8 passed 1 failed |

- A2 单独承重（M2 只红它）⇒ 「严格 fail-closed」确有判据，不是装饰。
- **没有自证陷阱**：A1 里「旧口径」是手工重建的 `HpBoundOrd`，但我在**父提交真代码**上跑同场景
  得到逐字相同的三个数：

  ```bash
  # /tmp/x（ba9783d 真源码）
  parent plan(35, ents=[69]) = On (闸门缺失)
  bound(hp=200, after=[ty32×3], flags=MAX) = 200, 真实可达 = 2600
  should_prune(best=301, hp=200, after) = true
  bound(hp=200, root, MAX) = 2600
  # 同一探针在 HEAD（/tmp/y）
  parent plan = Off(post_kill_hp_spawn,69)
  ```

  即 A1 的「修前会剪」不是实现者手算的自我印证——真代码就是这么剪的。
- 小缺口：A3 无 ADD 面（`ADD_2_72_11_10` 只在 f2 生成 ty72，drop 0），A3/A5 未钉算术（见 F3）。

---

## F5 f51..f54 逐字节 —— 绿

```bash
# HEAD 与父提交各自编译二进制，跑同一命令
cargo run -q --release -p game-solver --bin solver -- run --out /tmp/w_head.json     # HEAD
(CARGO_TARGET_DIR=/tmp/xt … )                                                       # /tmp/x 父版
cmp /tmp/w_head.json /tmp/w_parent.json   # 无输出
sha256sum /tmp/w_head.json /tmp/w_parent.json
# 5c430a6584b3dc381ee8ca86cbe3cd00c9c39b972119cb39d294ec58455036fb  (两文件同值)
# stderr 逐字相同：status=capped floors=6 total_steps=275 blocked=floor 3
#                  reason=truncated layer_ok=Some(false) layer_truncated=Some(true)
```

补充：**不带 `--out` 的 compact stdout 也逐字节相同**（`sha256=b896bc7a…`，15153 B）。
上一轮复核记的 `5c430a65…` 是 `--out` 的 pretty JSON；两个口径我都核过，一致。`DEFAULT_MAX_FLOORS=64`
（`witness.rs:87`），所以「6 层」是截断而非缺层。

---

## F6 基线诚实性 —— 绿

```bash
python3 gates/cli.py ratchet
# [✓ PASS] test-ratchet — total=1219(基线1219) 无回退
git diff ba9783d bcaceaf -- data/status/baselines/rust-test-counts.json
# - "game_solver(lib)": 53,  + "game_solver(lib)": 58,   total 1214 → 1219
```

- 1214→1219 = +5，与 lib 里新增的 5 条（A1–A5）**逐条对上**；`hp_bound(test)` 仍 3、
  `solver(bin)` 仍 0、examples 不计入门禁 ⇒ `hp_bound_sweep.rs` 没偷名额。
- `#[ignore]` 无新增（现有只在 `game-oracle/arm_coverage`1、`preset_cases`1、`fixture_frames`1、
  `gameplay_floor1`3、`game-solver/pure_pickup`2）。**存量**问题（ignore 被算进 ratchet）是上一轮
  F8 已登记的，本片未加剧，我不重复计。

---

## F7 变异可复现 —— 绿

M1/M2 与自述**完全一致**（M1 ⇒ A1+A2；M2 ⇒ 只 A2），各自还原后 `cargo test -q --lib hp_bound`
⇒ `9 passed; 0 failed; 0 ignored; 49 filtered out`。我另加 M3/M4（见 F4 表）也各自只红对应判据，
说明 A4/A5 同样承重。**TDD 诚实性**：commit message 明写「红是事后用 M1 复现的等价『修复前』形态，
诚实记录」，未冒称先红 ⇒ 这一点我认可，不另记黄。

---

## F8 p6 三处更正的质量 —— 黄（更正本身绿，仓库他处仍有旧结论）

**三处都在原地**（`git show 70f9436` 只有 1 文件 +487，全部是追加；原文表格/推导未删）：

| 位置 | 更正内容 | 我的独立核对 |
|---|---|---|
| `:20-50`（F2 行后 blockquote） | 1908 = `probe_bump` 用 S0 `atk=def=10` 配链态 `hp=3227` 的**混合态**；730/730 七杀损血 **0**；10/10 合计 **1958**，减 `ty42` 的 50 恰为 1908 | 我写 `probe10`（父代码）实跑：`10/10: [(45,638),(45,638),(44,384),(43,112),(41,24),(42,50),(43,112)] sum=1958`；`730/730` 全 0 ⇒ **逐字吻合** |
| `:104-130` | `3227+800−1908=2119` 不成立；「4027 是否可达未证」；「真墙是脚本不是怪物」不受影响 | 与 `script.rs:403-415` 一致（RES 只看 difficulty，与 hp 无关 ⇒ 「hp 与攻防不同源」成立） |
| `:368-400`（§3.3） | 3827「真最优」被 1M 档 `out_hp=3877 > 3827` 证伪（仍 truncated ⇒ 只是下界）；「数学上不可达（缺口 1908）」失效 | 3877 出自批8 F3 记录，我未重跑 1M（成本高）；但**方向**与 3827 非最优一致，且更正明写「不是最优证明」 |

**残留（黄的原因）**：`docs/findings/2026-11-19-pure-pickup-closed-form.md:303` 仍写「即使假装这三道闸
不存在，**HP 上界也不可能是 4027**：…全捡必然要战斗」，紧跟的 P7 blockquote 只说「730/730 战斗近乎免费、
3827 不是最优」，**没有撤回 4027 不可达**。grep `3827|4027|1908` 的其余命中（`f3-gate27-walk-only.md`、
`reviews/2026-11-19-gate-bump-p7-review.md:19`）与新口径一致。⇒ 本片三处更正合格，但**仓库里还有一处
半旧结论**；按 §3.6 只需在 `pure-pickup-closed-form.md` 原文加一条「4027 可达性未证」的更正，不必删。

---

## F9 CLI 影响面 —— 绿

- `crates/game-solver/src/bin/solver.rs:512-537`：只有 `solve-local` 分支在 **stderr** 多打一行
  `hp_bound floor=… enabled=… reason=… reason_ty=… pruned_entry=… pruned_pop=… slots=… pickup_types=… root_hp=… root_bound=…`；
  stdout 的 JSON 与 `run` 分支一字未动（`git diff` 里 `solver.rs` 只有这一处 + 新 fn）。
- **没有消费者**：`grep -rn "solve-local" --include=*.py|*.rs|*.sh|*.yml` 只命中 `solver.rs` 自身的
  用法串；`gates/` 里走的是 `solver run`。所以 stderr 新增不会污染任何以 stderr 为判据的测试/门禁。
- 实测：`solver solve-local --floor 1` ⇒ stderr `hp_bound floor=1 enabled=true reason=on … slots=3
  pickup_types=2 root_hp=300 root_bound=700`（与 f3 测试的 1100 同构，差在 floor1 夹具），stdout
  `max_hp=202`；`--floor 2` ⇒ `enabled=false reason=not_run …` 全零。
- **观察（不记黄）**：floor ≥2 时 `solve-local` 本身是 `SolveInput::placeholder`（`level:None`），
  所以那行永远 `not_run/0` —— 可见性只对 floor 1 有意义。若复核要的是「任意层可见」，得让 CLI 走真夹具；
  当前语义没错，只是容易误读。

---

## F10 夹带与诚实性 —— 绿

```bash
git show --name-status --format="" bcaceaf
# A examples/hp_bound_sweep.rs  M bin/solver.rs  M src/hp_bound.rs  M solve_local.rs
# M tests/hp_bound.rs  M data/status/baselines/rust-test-counts.json
# A docs/findings/2026-11-19-hp-bound-p8b-spawn-gate.md
git show --name-status --format="" 70f9436
# A docs/findings/reviews/2026-11-19-p6-monotone-pickup-review.md
git status --porcelain | head
# 7 份既有 review 仍为 ??（未被这两笔夹带）
```

- 无 `data/naming/`、`reference/`、`original/`；无他人未跟踪 review。
- **没有放宽断言**：`tests/hp_bound.rs` 的 diff 全是注释（标题 + 口径更正 blockquote + 三层描述），
  一条 `assert` 都没删；`src/hp_bound.rs` 的删除行只有：文档标题两句、`if let ShopEffect::AddHp{amount} = g.effect`
  （改成 `match`）、两条 `certify_entity(...,&mut t.konst)`、以及统计里的两行（改成 `slot_count()/pickup_type_count()`）。
- commit message 交代了 TDD 非真先红、交代了 f51..f54 的 sha、交代了 ratchet 变更；Confidence 88。

---

## 计数与结论

**绿 8**（F2 F4 F5 F6 F7 F9 F10 + F1 的通道枚举本体）/ **黄 2**（F1 的泛化承诺、F3 归因+覆盖）/
**红 0**。

Confidence：对**本复核结论** 90（父/HEAD 双二进制、变异 4 条、语料变异 3 次、父代码复算均一手；
唯一未复跑的是 1M 档 3877，已标明）。对 **`bcaceaf` 作为机制实现** 88（闸门逻辑 fail-closed 正确、
A1/A2 真承重、sweep 与 f2 反例可复现；扣分在「槽 32 承诺」与「CES 归因」两处表述，不是实现错误）。

**两句结论**：

1. **`bcaceaf`+`70f9436` 可以合并**——代码面 0 红，f2 活反例被真闸门关掉、f51..f54 与父提交逐字节
   相同、判据各自承重；两处黄是**文档/泛化表述**，可在合并前后用一笔 doc 更正补掉，不构成阻塞。
2. **必须现在补的不健全边界只有一条**：槽 32（`on_line_end` 永不置位 ⇒ 可重复触发）——要么 3 行
   `Off`，要么把「将来若加 hp op，本论证仍成立，不需改代码」改成「必须加闸门」；其次是
   `CES/ADD` 那条判据补一个「旧口径塌陷」的算术断言（防静默退化），kill-hook 槽 20/21/22 登记为
   未覆盖面即可（今日无 hp 写）。
