# P8 复核：单层搜索的「hp 可采纳上界剪枝」（对抗式独立复核）

- 复核日期：2026-11-19
- 被复核对象：`ba9783d`（HEAD，本地未 push），父 `e67630c`
  - 新增 `crates/game-solver/src/hp_bound.rs`（719 行）
  - 改 `crates/game-solver/src/solve_local.rs`（+140/−5，`Search.hp_prune`/`kill_objective`/`BestHp`）
  - 新增 `crates/game-solver/tests/hp_bound.rs`（3 条集成）、改 `examples/f3_bound.rs`（+80）
  - 改 `data/status/baselines/rust-test-counts.json`（total 1201→1214）
- 复核者权限：**只读仓库**；只写本文件。全部变异/探针实验在临时副本
  `/tmp/p8_head`（`git archive ba9783d | tar -x`）、父提交副本 `/tmp/p8_parent`。
- 纪律基线：`AGENTS.md`（§3 证据分级 A/B/C、禁调阈值掩饰、禁放宽断言、§3.6 证伪更正文化）。
  **实现者自述与「测试全绿」均不作为判据**；下面每条只引用我自己跑出的命令与输出。

---

## 1. 判定摘要表

| 面 | 判定 | 一句话证据（详见 §2） |
|---|---|---|
| F1 f3「零损血 ⇒ bound≡4027」 | **绿**（且是本片最重要结论） | 用**被复核 commit 自带的** `F3_FIGHT=1` 复算：f3 的 7 只 cat8 对 3227hp/730atk/730def 玩家 `predict_loss=0` ×7（连反击也是 0）；`root_bound=4027=3227+800`；200k 档实测 `pruned_entry=0 pruned_pop=0` ⇒ **hp 维在 f3 真实链态下恒等、剪枝零收益，正解方向被证伪** |
| F2 上界的可采纳性 | **黄**（主张被反例证伪，当前语料不咬） | 实测 on/off 在 f3（hp=100/300/3227）+ 22 个真实入场格 + 46 层 sweep **全部同解**；但我构造出**活的越界通道**：f35 击杀 ty69 会生成 ty32×3（tier 4 ⇒ 800/颗 = **2400**），击杀前 `bound` 记 `+2400`，击杀后这 3 颗进 `ents` 却按 `0` 计（`pickup_hp` 只用根态实体播种）⇒ `should_prune(best=301, hp=200, spawned)=true` 而该态真实可达 hp=2600 |
| F3 300hp 激活数字 | **绿**（我先前误判，见 §3 我改判 1） | `pruned_entry=2264 / pruned_pop=1500 / max_hp=826 / route 34 项` 逐字段复现；差异全因我自己把 `SOLVER_LAYER_LIMIT` 设成 20000（`F3_LOCAL` 硬编码 200000） |
| F4 层路径禁用是否诚实 | **绿** | `solve_layer_with` 路径 `Search::from_entry` 的 `hp_prune` 默认 `false`（只有 `solve_local()` 置 true）；两处剪点都在 `if let HpBoundPlan::On(ord) = &hplan` 内；代码注释原文写明「一度启用后 `solver run` 由 floors=6 退化为 f52 `no_exit_toward`」+「须把 best 限定到该查询的目标出口集合（未做，不猜）」⇒ 无悄悄少剪、无偷偷做 best 限定 |
| F5 基线诚实性 | **绿** | 父提交工作区实测 `--test gate_bump --list`=**10**、`--test pure_pickup --list`=**15**（基线里是 5/14）⇒「并入父提交既有未记基线 +5/+1」属实；HEAD `--lib`=53、`--test hp_bound`=3，与 1201→1214 一致；新测试文件无 `#[ignore]` |
| F6 f51..f54 逐字节 | **绿** | 父/HEAD 各跑 `solver run --out`，sha256 同为 `5c430a6584b3dc381ee8ca86cbe3cd00c9c39b972119cb39d294ec58455036fb`；stderr 同为 `status=capped floors=6 total_steps=275 blocked=floor 3 reason=truncated` |
| F7 变异表可复现 | **绿** | M1（`bound` 减 600）⇒ 红 2 条，报错文本与 commit message 一字不差（`on=(…626,0,32) vs off=(…426,0,30)`、`root_bound 500`）；M2（`<`→`<=`）⇒ 只有严格性单测红；还原后 3/3 + 53/53 全绿 |
| F8 fail-closed 的代价 | **绿**（附 1 条可观测性观察） | 68 槽语料出现的 op 全在 16-op 白名单内（TAK/MOV/DES/CES/ROS/SEE/GIN/GLV/LAY/GUT/ADD/END/SMS/SWD），54 层里**没有任何一层**因 `script_op_unknown`/`slot_out_of_range`/`pickup_not_certified` 而 Off；9 层 Off 全部是 `no_hp_sources`（真无血源）。观察：`solver` CLI 完全不打印 `hp_bound` 统计 |
| F9 门禁与夹带 | **绿**（我先前误读，见 §3 我改判 2） | `python3 gates/cli.py check` = `PASS=13 FAIL=0 SKIP=0`（与自述一致）；`--name-status` 6 个文件全在声明范围，无 `data/naming/`、`reference/`、`original/`、他人 review 夹带 |
| F10 结论诚实性 | **黄** | commit message 对判据 1 写「部分达成」、明写「3827 非真最优」——该点诚实；但**被证伪的旧结论仍原样留在 `docs/findings/reviews/2026-11-19-p6-monotone-pickup-review.md:375`**（「零战斗…**3827**…**真最优**」，且 §F2 断言「4027 数学上不可达、损血 ≥1908」），本 commit 未按 §3.6 原地更正 |

计数：**绿 8 / 黄 2 / 红 0**。

其中两条黄的承重不同：F2 是**技术性缺陷（主张不成立，但今天不改输出）**；F10 是**流程纪律缺陷（证伪文化未落地）**。两者都不是「测试全绿」能掩盖的，也不能靠加断言补上。

---

## 2. 逐面详证

### F1 f3「零损血 ⇒ bound≡4027」——绿（本片最重要结论）

**复现命令（不需要任何自定义代码，直接用 commit 自带的探针）**

```bash
git archive ba9783d | tar -x -C /tmp/p8_head   # 仅副本；资产用 symlink
ln -s "$PWD/assets/raw" /tmp/p8_head/assets/raw
cd /tmp/p8_head && CARGO_TARGET_DIR=/tmp/p8_tgt F3_FIGHT=1 \
  cargo run --release -q -p game-solver --example f3_bound
```

**关键输出**

```
fight ty=45 at (1,3)  predict_loss=0      fight ty=41 at (7,5)  predict_loss=0
fight ty=45 at (10,8) predict_loss=0      fight ty=42 at (8,10) predict_loss=0
fight ty=44 at (1,7)  predict_loss=0      fight ty=43 at (3,5)  predict_loss=0
fight ty=43 at (10,2) predict_loss=0
```

7 只 cat8 全为 0。再用自建探针（`/tmp` 副本内 `examples/p8_probe.rs`）直调
`predict_engagement(..., false)`（**带反击**）同样是 0 ⇒ 自述「一击必杀零损血」成立，且比自述更强。

**机制核对（为什么「零损血 ⇒ bound 恒定」成立）**

- f3 的 hp 拾取面只有 7 枚：`ty31×4`（各 +50）、`ty32×3`（各 +200，tier 1）⇒ 合计 +800；
  `slots=2`（门 param 0 与 27）经 `parse_slot` 算出 **0** 贡献（f3 的两槽无 `GIN_0`/`RES`/`ROS_3`）。
- 绑定发生顺序上，所有 7 枚都已在根态 `ents` 里（`pickups=2` 指两个**类型**，不是两枚）。
- 因此 `bound(hp, ents, flags) = hp + Σ_{未被捡走} 增量 + 0`，而「捡走一枚」= `hp += 增量` 且该实体从 `ents` 消失 ⇒ **bound 是严格不变量 4027**。
  这是拾取通道的设计不变量（`bound` 与拾取臂共用 `certified_pure_hp_gain`），本身是对的。
- 由于击杀无损血，`kills` 也不改变 hp ⇒ **任何可达态 bound 恒 = 4027**；而 `should_prune` 是严格 `bound < best_hp`，`best_hp ≤ 4027` ⇒ 永不成立。

**实测确认（200k 档 f3 真实链态）**

```
f3 SAME=true  max_hp=3827  truncated=true  pruned_entry=0  pruned_pop=0
f3 plan: enabled=true reason=on slots=2 pickups=2 root_hp=3227 root_bound=4027
```

**结论**：在 f3 这个被判据 1 指定的层，「hp 可采纳上界」**要么恒等于真实上界、要么（若换玩家）在损血处才有剪枝空间**；真实链态下 `pruned=0`。
⇒ **「hp 上界是 f3 的正解」这个方向本身被证伪**。这不是实现 bug，是问题结构：f3 的 hp 维在满装态下是**守恒量**，剪枝对象不存在。

**顺带定位旧结论的错因（A 级、可复现）**：p6 review 的「6 杀损血 ≥1908」用的是 `atk=def=10`（S0 装备）配 f3 的 `hp=3227`：

```
P8_MODE=ents P8_FLOORS=3 P8_ATK=10 P8_DEF=10 P8_HP=3227
  ty45→638, ty45→638, ty44→384, ty43→112, ty41→24, ty42→50, ty43→112     （7 杀合计 1958）
P8_MODE=ents P8_FLOORS=3 P8_ATK=730 P8_DEF=730 P8_HP=3227
  七行全 0
```

`1958 − 50 = 1908` 正是旧文里的缺口数字（旧文表格里「只杀 ty42 损血 50」也对得上）
⇒ **旧前提的根因是「f3 入场 hp 用了链态、atk/def 用了 S0」的混搭**，不是求解器错。

### F2 上界的可采纳性——黄（主张被反例证伪，当前语料不咬）

**(a) 实测同解面（这是实现者主张的部分，成立）**

```
# f3 三档入场 hp，on/off 逐字段（reachable/truncated/max_hp/min_kills/conf/route/keys/起点）
f3 hp=100  SAME=true max_hp=626  pruned=1097/728
f3 hp=300  SAME=true max_hp=826  pruned=2264/1500
f3 hp=3227 SAME=true max_hp=3827 pruned=0/0
# 22 个真实入场格定向扫描（f51 1,11 / f52 2,11 / f53 6,10 / f54 7,9 / f1 6,11 / f2 1,2 /
#   f10 f15 f16 f20 f32 f33 f35 f36 f37 f38 f39 f40 f41 f42 f43 f48）：全部 SAME=true
# 46 层 sweep（/tmp/sweep300.txt）：mismatch=0
f52: pruned_entry=886 pruned_pop=143  on 0.62s vs off 3.39s（≈5.5×）  max_hp=752 同解
f54: truncated=true 6.25s，仍同解
```

即：在当前语料 + 当前入口下，界**没有剪掉任何最优解**，且 f52 上有真实加速（5.5×）。

**(b) 但我构造出活的越界通道（这是 F2 的判定依据）**

通道：**击杀 boss 生成的 HpUp 实体，在 `bound` 里从「记 2400」变成「记 0」**。

```bash
# /tmp 副本内自建探针：对每层的 boss 型实体，比对
#   ① 击杀前该实体在 bound 里的贡献（= drop_gain）
#   ② 击杀后 3 颗生成物进 ents 后 bound 的贡献（= Σ entity_gain）
P8_MODE=drop P8_HP=300 P8_FLOORS="10,20,35,40"
```

```
f10 ty=75@(6,2) drop_gain=+600  hp_spawns=3x200  after_kill_bound=+600   root hp-pickup types=[32]
f20 ty=74@(6,6) drop_gain=+1200 hp_spawns=3x400  after_kill_bound=+1200  root hp-pickup types=[31,32,31,31]
f35 ty=69@(4,6) drop_gain=+2400 hp_spawns=3x800  after_kill_bound=+0     root hp-pickup types=[]
f40 ty=73@(6,2) drop_gain=+2400 hp_spawns=3x800  after_kill_bound=+2400  root hp-pickup types=[32,31]
```

（探针另外打印的 `deficit=` 用的是 `hp-100` 基线，读法应为 `drop_gain − after_kill_bound`：
f35 = 2400，f10/f20/f40 = 0。）

f35 是反例：`ty69@35` 击杀触发组 4（芭蕉扇）+ **post-anim 组 7（`ty32×3`）**，
`floor_tier(35)=4` ⇒ `200×4=800/颗`，共 **2400**（`hp_bound.rs` 与 `rules/boss.rs` 都这么算，没问题）；
但 f35 根态**没有任何 ty32**，而 `plan()` 只把**根态 `ents`** 里的 cat2|4 播种进 `pickup_hp`
（静态实体循环只做 `certify_entity` 的 Err 检查、**不插入**表）：

```rust
// hp_bound.rs:456-470（根态播种）
for e in ents { ... 2 | 4 => { ord.pickup_hp.insert(e.ty, g); } ... }
// hp_bound.rs:479-489（静态实体：只认证，不入表）
for e in &level.entities { if seen_types.contains(...) { continue; } certify_entity(...)?; }
```

⇒ 击杀 ty69 后，3 颗 ty32 已在 `ents` 里（`apply_kill_drops` 与 `drop_gain` 同表，`solve_local.rs:1193-1208`），
但 `entity_gain(32)` 走 `shop_hp → pickup_hp → drop_hp` 全落空 ⇒ 按 **0** 计。

**直接执行剪点判据**（同一份 `HpBoundOrd`、`flags=MAX` 关掉槽项）：

```
should_prune(ord, best=301, hp=200, ents=[ty32,ty32,ty32], flags=MAX) = true
而该态真实可达 hp = 200 + 800×3 = 2600
```

⇒ **`bound` 不是可采纳上界**：存在可达态使 `bound` 比真实最优低 2400。实现者自述 2 的
「界可采纳」在**一般意义上不成立**（当前语料上不咬，见 (c)）。

**(c) 为什么今天没咬（也说明它为什么危险）**

- f35 全场只有 1 只 cat8（ty69），实测 `predict_engagement` 在 `atk=def≥600` 时 `loss=0`，
  在 600 以下则伤害致死（bump 臂不存在）⇒ **要么白打、要么必死，没有「先掉血再杀」的中间态**；
  白打时 `hp=300`、`best=300`、`bound=300`，严格 `<` 恰好不触发。
- 其余产出 hp 的 boss 层（f10/f20/f40）根态都有 ty32/ty31 ⇒ `pickup_hp` 已播种，dropped 实体按同值计入，
  `drop_gain == after_kill_bound`（见上表），不产生差额。
- 一旦将来出现「有损血的其他怪 + 掉 hp 的 boss + 根态无同型拾取」的层，就会**静默剪掉真实最优**，
  而且因为 `pruned_entry>0` 看起来「机制在生效」，很难被发现。

**(d) 其余通道逐条排查（都排除，附证据）**

| 通道 | 结论 | 证据 |
|---|---|---|
| `ROS_3` 脚本绝对写 hp | 当前语料**不存在** | 68 槽扫 + `parse_slot` 已按 `abs_write=max` 建模 |
| `RES_0`（easy 重置 hp=400） | 唯一实例在槽 0，且**在 `LAY_2` 之后** | `parse_slot` 在 LAY 截断；引擎侧 `tickdrive` 在 `persist.pending_floor != 0` 立即 return ⇒ 该 `RES_0` **永不执行**；实测 f3 300hp 档 on/off 同解佐证 |
| `GIN_0` 拾取 | 槽 27/28 的 `GIN_0_13`/`GIN_0_19` 是 ty13/19，**不是 HpUp**（HpUp 只有 31/32） | `effect_registry.rs:129-130` |
| `ADD_`/`CES` 生成 hp 物 | 语料无 `ADD_*_31_*`/`CES_31/32`（只有 `CES_36`） | `grep -o "CES_3[0-9]_\|ADD_[0-9]*_3[0-9]_" crates/game-core/tests/fixtures/level_scripts.txt` |
| boss 掉落 | **见 (b)：唯一缺口** | 上表 |
| 商店 `AddHp` | f3 的 `ty77 param=278` 未开价（`(p>>8)&2==0`）⇒ 无购买面；solver 买后删实体、bound 只计一次，自洽 | `scene.rs:2540-2620` vs `solve_local.rs:2348-2372` |
| 背包/道具回血 | 搜索臂只处理道具 21/22（上下层符），不改 hp | `record_useitem` |
| 门槽脚本 hp（引擎真值） | 门执行走真引擎 + `readback_script` 回读，非手抄 | `gate_arrive` |

**(e) 第二条结构性风险（当前不咬，记录在案）**：`bound` 只按「`flags` 该位为 0」计槽，
但 `gate.rs` 的 `case 0` 会 `push flag 19`、`case 18` push 18、`case 7` 置 32
⇒ 槽 18/19/32 可能「未执行却已置位」，槽项被提前抹掉。实测槽 19=`CES_84_7_7 …`、槽 18=`CES_20_4_4 …`、
槽 32=`TAK_79_79`，**均无 hp op** ⇒ 现状无害；一旦这 3 个槽将来带上 hp 项即变成低估。

### F3 300hp 激活数字与 `F3_LOCAL` 口径——绿（我先前的怀疑是错的）

```bash
cd /tmp/p8_head && CARGO_TARGET_DIR=/tmp/p8_tgt F3_LOCAL=1 \
  cargo run --release -q -p game-solver --example f3_bound
```

```
limit=   20000 out_hp=3427 truncated=true
limit=  200000 out_hp=3827 truncated=true
limit= 1000000 out_hp=3877 truncated=true          # 三档全部 truncated ⇒ 3877 只是下界
```

`F3_LOCAL` 分支走的是 `solve_local`（**全层 max_hp + min_kills 口径**，不是 `exit_toward`），
并在代码里硬编码 `SOLVER_LAYER_LIMIT=200000`。我直接调 `solve_local`（300hp、入口 `(2,11)`、200k）：

```
max_hp=826 min_kills=0 truncated=true route=34 项
hp_bound enabled=true reason=on slots=2 pickup_types=2 root_bound=1100
         pruned_entry=2264 pruned_pop=1500            # 与 commit message 逐字段相同
```

**附**：`f3_bound` 打印的 `hp_bound enabled=false reason=not_run` 出现在 **exit_toward/层路径**上——
那里 `run_root_with` 走的是 `HpBoundPlan::Off("prune_not_enabled")` 分支但**不写 stat**，stat 保持入口清零后的
`not_run`。诊断含义略含混（「没问过」vs「问了但关着」），建议后续区分（不阻塞）。

### F4 层路径禁用是否诚实——绿

- 机制：`Search::from_entry` 里 `hp_prune: false`（`solve_local.rs:1902`），只有 `solve_local()`
  在 `solve_local.rs:299` 置 true；两处剪点（pop 侧 2727、入队侧 2854）都包在
  `if let HpBoundPlan::On(ord) = &hplan` 里 ⇒ 层路径**结构性不可能剪**。
- 记录：`solve_local.rs:767-772` 注释原文——「层路径**不启用** hp 剪枝 … 一度启用后 `solver run`
  由 floors=6 退化为 f52 `no_exit_toward` … 同层若要让剪枝生效，须把 best 限定到该查询的目标出口集合
  （**未做，不猜**）」。
- 我核了「目标出口限定 best」是否真的没做：`BestHp::note` 只按 `g.kind == Stair` 记 `kills_stair`，
  `hp` 取**全部** goal 的最大值，`solve_local_exit_toward` 的 `toward` 只在**解集挑选**阶段用
  （`solve_local.rs:507-570`），不进 `BestHp` ⇒ 确实没做，注释与实现一致。

### F5 基线诚实性——绿

```bash
git show ba9783d -- data/status/baselines/rust-test-counts.json
#  game_solver(lib) 49→53 | gate_bump(test) 5→10 | hp_bound(test) 新增 3 | pure_pickup(test) 14→15 | total 1201→1214
git log --oneline -- data/status/baselines/rust-test-counts.json   # 父 4be401e 及更早都没动过该文件
# 父提交工作区实测（/tmp/p8_parent）
cargo test --release -p game-solver --test gate_bump --test pure_pickup --list
#   → gate_bump 10 个、pure_pickup 15 个（基线记的是 5 / 14）
# HEAD 实测
cargo test -p game-solver --lib          # 53 passed, 0 ignored
cargo test --test hp_bound               # 3 passed, 0 ignored
```

⇒ 「并入父提交既有未记基线（+5/+1）」**属实**，不是把本 commit 的新账混进旧账；
新文件无 `#[ignore]`。**提醒（既有门禁性质，非本 commit 引入）**：ratchet 用 `exe --list` 计数，
仓库现有 4 处真 `#[ignore]`（`gameplay_floor1.rs:340/367/417`、`slot_equip_scene_tests.rs:732`）
同样占名额 ⇒ 基数虚高，建议另开一笔记账，不属本次扣分项。

### F6 f51..f54 逐字节——绿

```bash
cd /tmp/p8_parent && cargo run --release -p game-solver --bin solver -- run --out /tmp/run_parent.json
cd /tmp/p8_head   && cargo run --release -p game-solver --bin solver -- run --out /tmp/run_head.json
sha256sum /tmp/run_parent.json /tmp/run_head.json
#   两边同为 5c430a6584b3dc381ee8ca86cbe3cd00c9c39b972119cb39d294ec58455036fb
# 两边 stderr 同为：
#   status=capped floors=6 total_steps=275 blocked=floor 3 reason=truncated layer_ok=Some(false) layer_truncated=Some(true)
```

顺带取到真实链态（供后续引用）：`f51 300→247`、`f52 247→427`（出层 atk/def=730）、`f53 427→827`、
`f54 827→3227`（`pure_pickup=adopted`）、`f1 3227`、`f2 3227`、f3 未出层。

### F7 变异表可复现——绿

副本内备份 `cp src/hp_bound.rs /tmp/hp_bound.orig.rs` 后逐条改：

| 变异 | 结果 |
|---|---|
| M1 `fn bound` 首行改 `hp as i64 - 600` | `cargo test -p game-solver --test hp_bound` = **2 failed / 1 passed**；失败文本 `on=(true,false,Some(626),Some(0),32) vs off=(true,true,Some(426),Some(0),30)`、`root_bound 500`（与 commit message 一字不差） |
| M2 `bound < best_hp as i64` → `<=` | 只有 `hp_bound::tests::unit_strict_less_than_is_the_only_prune_point` 红（平局不得剪）；3 条集成仍绿 ⇒ 该单测是唯一承重点，诚实记录在案 |
| 还原 | `--test hp_bound` 3/3 绿、`--lib` 53/53 绿 |

### F8 fail-closed 的代价——绿（附 1 条观察）

```bash
P8_MODE=stat P8_FLOORS="1..54" P8_HP=300
# 45 层 reason=on；9 层 Off("no_hp_sources")：f13 f22 f24 f26 f29 f30 f49 f50
# 零层因 script_op_unknown / slot_out_of_range / pickup_not_certified 而 Off
```

68 槽实际出现的 op 集合 = `{TAK, MOV, DES, CES, ROS, SEE, GIN, GLV, LAY, GUT, ADD, END, SMS, SWD} ⊂ 白名单`
⇒ 白名单在当前语料**零代价**，「大多数层静默不剪」不成立（Off 的 9 层是真无血源）。
**观察**：`bin/solver.rs` 里没有任何 `hp_bound` 字样 ⇒ `solver run/solve-local` 的产物**看不到**
`reason/slots/root_bound/pruned_*`；stat 只能从库 API / `f3_bound` 读。生产里若某层静默不剪，
CLI 层面是**完全不可见**的（既有的 `pill_dom_stat` 至少还在 witness JSON 里）。

### F9 门禁与夹带——绿

```bash
python3 gates/cli.py check     # 汇总：PASS=13 FAIL=0 SKIP=0（与 commit message 一致）
git show --name-status ba9783d
#   M crates/game-solver/examples/f3_bound.rs
#   A crates/game-solver/src/hp_bound.rs
#   M crates/game-solver/src/lib.rs
#   M crates/game-solver/src/solve_local.rs
#   A crates/game-solver/tests/hp_bound.rs
#   M data/status/baselines/rust-test-counts.json
```

6 个文件全在声明范围内：无 `data/naming/`、`reference/`、`original/`、无他人 review 文档被顺手改。
工作区 7 个未跟踪 review 文档均为既存产物（不属于本 commit）。

### F10 结论诚实性——黄

- **诚实的地方**：commit message 判据 1 写「**部分达成**：200k out_hp=3827/truncated=true、1M 3877 ⇒ 3827 非真最优」，
  没有把未达成写成达成；门禁/ratchet/变异的数字我全部复现。
- **不诚实的地方面（流程）**：被本次实测**证伪**的旧结论仍原样留在仓库里，未按 `AGENTS.md §3.6`
  「原地标注日期更正、不删除历史主张」处理：

```
docs/findings/reviews/2026-11-19-p6-monotone-pickup-review.md:375
  | **零战斗**（踩 ty83@(5,11)，只捡自由区） | 9 | 600 | 0 | 0 | **3827** | **真最优** |
docs/findings/reviews/2026-11-19-p6-monotone-pickup-review.md:20 / :104-107
  「4027 需要全捡 18 枚 ⇒ ≥6 杀 ⇒ 损血 ≥1908 ⇒ 4027 在任何路线下都不可达」
```

  这两句与本次 A 级实测（7 杀零损血）**直接冲突**；而 `docs/findings/2026-11-19-f3-gate27-walk-only.md:104-107`
  与 `pure-pickup-closed-form.md:310-311` 已写「3827 不是最优」——**仓库内部现在是自相矛盾的**。
  建议在本 commit 或紧邻的 docs commit 里，于 p6 review 相应段落加一行
  「> 【2026-11-19 更正】… 见 2026-11-19-hp-bound-p8-review.md §F1」。
- 另：`3827 真最优` 的**结论**已被 1M 档 3877 证伪，但 `4027 是否可达` 仍**未知**（1M 仍 truncated）；
  本次只证伪了旧文的**理由**（损血 1908），没有证明 4027 可达。措辞上不要再往「4027 可达」滑。

---

## 3. 我改判记录

**我改判 1（F3）**：此前我读到「300hp 入层 `pruned_entry=2264/pruned_pop=1500`、`max_hp=826`、`route_len=34`」
时，用 `SOLVER_LAYER_LIMIT=20000` 直调 `solve_local` 得到 `16/48、max_hp=426、route 30 项`，一度怀疑数字不实。
证据：`F3_LOCAL` 分支硬编码 `SOLVER_LAYER_LIMIT=200000`；按 200k 重跑后与自述**逐字段相同**。
⇒ 是我自己的界设错，自述 2 的激活数字**属实**。

**我改判 2（F9）**：我先跑 `gates/cli.py status` 看到 `SKIP=1`（visual-diff 未就绪），据此怀疑 commit message
的「SKIP=0」失真。证据：`gates/cli.py check` 的汇总行是 `PASS=13 FAIL=0 SKIP=0`——`status` 是预检清单
（显式打印未就绪项），`check` 是总闸，两者口径不同。⇒ 「SKIP=0」**属实**，此前的怀疑不成立。

---

## 4. Confidence

**我对本复核结论的置信度：90。**
扣分理由：F2 的反例是「谓词级 + 表级」实证（`should_prune` 实跑为 true、`bound` 实跑为 0、真实可达 hp=2600），
但**我没有在该层造出真实的 on/off 输出差异**——因为 f35 只有一只怪且「白打或必死」，构造不出
「掉血后再杀 boss」的中间态；因此「今天不改任何输出」这一句建立在「语料范围 + 免费/致命二值」的论证上，
而非穷举。其余各面（F1/F3/F5/F6/F7/F9）都是可一字复现的命令输出，但 F1 的「7 杀零损」只对
`3227/730/730` 这一个链态成立（换玩家就不再零损），表述时不要外推。F8 的「白名单零代价」只覆盖当前 68 槽语料。

**ba9783d 作为「机制实现」的置信度：70（可留主干，但须改文档口径）。**
扣分：① 「界可采纳」是不成立的强主张（F2 反例）；② 被证伪的旧结论未原地更正、仓库自相矛盾（F10）；
③ hp stat 在 CLI 不可见（F8 观察）。不扣：核心机制、fail-closed、层路径禁用、变异、基线、逐字节回归都经得起复现。

---

## 5. 两句话结论

**（1）ba9783d 能不能留主干？**
**能留**——它是纯增量的、默认只在 `solve_local`（不是 witness 消费的层路径）上生效，f51..f54 逐字节不变、
门禁 13/0/0、变异可复现，且它诚实地报告了「判据 1 未达成」；但合并前必须把「界可采纳」降级为
「在当前 68 槽语料上实测同解，且存在已知反例（f35 boss 掉落 2400）」并把 f35 通道写进代码注释，
同时按 §3.6 在 p6 review 原地补更正注记——否则下一片会带着「已证明可采纳」的错觉继续加剪枝。

**（2）f3 的下一步该攻什么（给定 hp 维已证明无效）？**
**别再加 hp 维的剪枝**：f3 的真实链态里 `bound ≡ 4027`、`pruned=0`，这条线在 f3 上是数学死路；
下一步应攻**状态维**——即 1M 档仍 `truncated` 的真墙：`ty83@(5,11)`（param=27）的步臂/一次性脚本与
`PillDom` 被 `step_arm_hazard_reachable ty=83`（G4）关掉后的状态爆炸。可先做两件小事验收：
① 把 f3 的门臂脚本建模成「走上去才生效」的显式边（而不是靠 G4 一刀切禁用药维支配）；
② 给 max_hp 目标加**状态数**上界/支配（同一 `(pos, ents, flags, player)` 只留 hp 最高且 steps 最小），
在不开 hp 剪枝的前提下把 1M 的 24.6s/42 waypoints 往前推，再谈 3877→4027 是否真可达。
