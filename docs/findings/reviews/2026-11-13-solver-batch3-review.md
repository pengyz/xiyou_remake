# 求解器批3「全局分层求解 + 状态抽象」独立对抗复核报告

| 项 | 值 |
|---|---|
| 复核件 | 子代理 `a7c53f62`（独立对抗复核，未参与实现） |
| 被审对象 | `de80ad1..da60d84` 共 **14 笔**本地链（分支 `modern`，**未推送**） |
| 复核日期 | 2026-11-13 |
| 方法 | 逐项**自己跑**（贴件自证不算数）；只写本报告文件，**未改任何代码** |
| 产出要求 | 总判 + 逐项证据（命令/输出摘录/行号实测）+ 黄项清单 |

---

## 0. 总判

**🟡 黄 — 可推送（红 0，黄 7；全部零行为，随批修 / 挂批4）。**

- **红 0 项**：唯一红项 `R-1 F1 证伪更正缺失` 已由成员单笔 `71908d6` 修复，我**逐要素复验通过**（§11），转绿。
- **黄 7 项**（不阻断推送，见 §0.2）。
- 其余 8 个复核面（①②③⑤⑥⑦⑧⑩ + ④的四类换层边实锚）**绿**。

### 0.1 已闭合的红项（复验记录）

**R-1（原红）F1：`plan:343`「换层边 = 楼梯 param 指向 + LAY/GUT + 型72 触发…」半句缺原地日期更正注 —— ✅ 已修复，复验通过。**

- **原缺位取证（我自己跑的）**：plan 全文「更正」注只有 `:62 / :126 / :165`；`git log -S "LAY/GUT" --all` 只命中原始计划笔 `dbc8d86`；链内触碰 plan 的只有 `19b1684`（F4，2 hunk）；14 笔 message `grep "F1|非换层"` = 0。
- **口径修正（lead A 级证据，推翻先前定红前提）**：**LAY 确是换层**（`script_ops.rs:68-73 op_lay` = `save_floor_state + set_pending_floor + set_transitioning + set_dialog_phase(5)`；`a.java` 的 `q=5/dialogPhase=5` 只是渲染相位）→ **`plan:299`「LAY(换层)」为真，必须零改动**；**GUT 确非换层**（`script_ops.rs:167-169 op_gut` = `load_level_script` 脚本槽跳转）→ 只有 `plan:343` 的 `LAY/GUT` 半句有病。
- **修复笔**：`71908d6 docs(plans): 批3 复核红1 修复——plan:343 LAY/GUT 换层边原地日期更正`（单笔 docs，`1 file changed, 2 insertions(+)`，**0 删除**）。
- **复验（我 sed 实读，§11 详录）**：四要素齐（两命题 + 原句未删 + 日期 `【2026-11-13 更正·复核红1】`）、**`plan:299` 逐字节未动**（`git diff da60d84..HEAD --stat` 仅 1 文件 +2 行）、commit message 含**证伪链自记 + `Evidence:` + `Confidence: 95`**、我后台独立重跑 `gates/cli.py check` = **PASS=13 FAIL=0 SKIP=0（exit 0，金样 74 场景 sha 全同 GREEN）**。

### 0.2 黄项清单

| # | 项 | 证据 | 处置 |
|---|---|---|---|
| Y1 | `3e7d235`（ratchet 笔）message 写「`solve_global(test)=18`」，与 `data/status/baselines/rust-test-counts.json:63 "solve_global(test)": 17` 及实测 17 不符（18 = 存在性闸 `--list` 计数，含 lib 内 `solve_local::solve_global_c1::…` 1 个） | 见 §7.2 | 零行为，**随批修**（改 message 口径或注明「存在性闸=18」） |
| Y2 | `crates/game-solver/src/state.rs:39` doc 行引用「计划 **:60 / :163**」，因 F4 插行已漂移为 **:62 / :165** | 见 §5.3 | 零行为，**随批修** |
| Y3 | confidence 降级条件 `\|\| node_cap_hit` **无测试钉住**：变异去掉该支路 → 0 红（17 passed） | 见 §8.3 | 覆盖缺口，**挂批4**（有解路径属批4 e2e；成员 `tests/solve_global.rs:1096` 注已自认「有解时才可能 optimal→bounded-best」） |
| Y4 | R11「区间门 4 分量入 `SolverState.flags`」字面未落地；`crates/game-solver/src/solve_global.rs:11` doc 仍写「4 分量入 flags」 | 见 §9.4 | **裁定：canonical 理由成立，黄不阻断**；doc 措辞与实现对齐 **随批修** |
| Y5 | A-lite 条件④ 风险注记（子集上确界/支配吸收限定）只在报审对话，**未落库**（全仓 + 全历史 grep「上确界\|支配吸收」= 0） | 见 §3.4 | **挂批4**（B 提案补录；lead 已裁不阻断） |
| Y6 | F4 更正注的穷尽对照证据指针指向「批3 报审件」，而报审件是对话交付物**不入库** → 库内无对照命令可复现 | 见 §5.4 | **随批4** 把 1703 记录普查命令/结果落 `docs/findings/`（本报告 §5.2 已给出可复现命令） |
| Y7 | `71908d6` 更正注内联锚「a.java:**6261**」用的是 **seed 行号**；同仓 `script_ops.rs:67` doc 对同一 LAY handler 用 **deobf 行号 `a.java:8355-8362`**（实际 `reference/src/deobf/a.java:8361-8369`），findings/plan 其余 `a.java:NNNN` 也均为 deobf 编号 → 按主流约定 sed 会扑空（deobf:6261 是实体数组初始化）。commit `Evidence:` 字段已限定 `reference/seed/a.java:6261-6268` 故**可查**，且 seed 正是 D3 不可变锚点（`gates/cli.py:175-185 reference-seed-integrity`） | 见 §11.4 | 零行为，**随批修**（内联改标 `reference/seed/a.java:6261` 或 deobf `a.java:8361-8369`，二选一即可） |

---

## 1. commit 链与夹带（复核重点①）✅ 绿

```bash
$ git log de80ad1..da60d84 --oneline | wc -l
14
$ git log de80ad1..da60d84 --oneline
4121f9b C1   5bedd77 C2   d6dcbe6 C3a   75259e9 C3b   4024a9f C4
5092493 C5+C5a   6b56e57 C5b   6a80bcf F3   19b1684 F4   8665de7 C6a
6de9ec1 C6b-C6g   9bb24fe C6h   3e7d235 ratchet   da60d84 docs
```

```bash
$ git diff de80ad1..da60d84 --stat
 crates/game-solver/src/bin/solver.rs            | 131 ++++++-
 crates/game-solver/src/lib.rs                   |   1 +
 crates/game-solver/src/solve_global.rs          | 652 +++++++++++++++++++++++ (新)
 crates/game-solver/src/solve_local.rs           | 1001 ++++++++++++++++... (-124)
 crates/game-solver/src/state.rs                 |  77 +++
 crates/game-solver/tests/census_snapshot.rs     |   6 +
 crates/game-solver/tests/solve_global.rs        | 1121 +++++++++++++++++ (新)
 data/status/baselines/rust-test-counts.json     |   2 +-
 docs/superpowers/plans/2026-11-09-solver-implementation-plan.md | 4 +-
```

- **9 文件，全部落在 game-solver + 1 ratchet JSON + 1 doc** → `game-data / game-core / reference / gates` **生产码零改动** ✅（doc 仅 F4 两处更正注，ratchet 仅 JSON 计数）。
- 工作树 `git status --porcelain` 仅 untracked 历史复核件（`docs/findings/reviews/*`）+ `android/build/`，**src 干净**（变异已还原自证）。
- 残留自证（A-lite 条件③）：

```bash
$ grep -rnE "SOLVER_PROF|\[LAYER\]|\[P10\]|\[GLOBAL\]|arrivals|PROB" \
    crates/game-solver/src crates/game-solver/tests | wc -l
0
$ grep -rnw TEMP crates/game-solver/src crates/game-solver/tests | wc -l
0
$ git diff de80ad1..da60d84 | grep -cE '^\+.*(TEMP|PROB|SOLVER_PROF)'
0
```

---

## 2. TDD 红档（复核重点②）✅ 绿

15 个 `/tmp/red_b3_*.txt` 全在。抽 5 份核 **mtime 早于对应 commit**（防补档）：

| 红档 | mtime | 对应 commit 时间 | 判定 |
|---|---|---|---|
| `red_b3_c1.txt` | 11-12 10:54 | `4121f9b` 11:02 | ✅ 早 |
| `red_b3_c4.txt` | 11-12 14:19 | `4024a9f` 14:35 | ✅ 早 |
| `red_b3_f4.txt` | 11-12 18:08 | `19b1684` 18:11 | ✅ 早 |
| `red_b3_c6f.txt` | 11-13 19:49 | `6de9ec1` 20:17 | ✅ 早 |
| `red_b3_c6h.txt` | 11-13 20:57 | `9bb24fe` 21:06 | ✅ 早 |

- 内容核验：均为**真红**——编译红（`E0432` / `E0599` / `E0609` / `E0425`）或断言红（`panicked` + `FAILED`），非空非贴。
- **异常点说明（非红）**：`red_b3_c5d.txt` mtime 11-12 19:03，而 `script_cross_allowed` 符号 `git log -S` 显示仅 `8665de7`(11-13 16:49) 引入、`6de9ec1` 删除（逐 commit `git show <c>:…solve_local.rs | grep -c`：到 `19b1684`=0、`8665de7`=14）→ 红档**早于**符号引入笔 = 非补档；链内无独立 "C5d" 笔（C5d 测试并入 `8665de7` C6a），与成员 message 计数序列一致。

---

## 3. A-lite 四条件对照（复核重点③，lead 2026-11-13 裁定）✅ 绿

### 3.1 条件① 截断可观测 ✅
`crates/game-solver/src/solve_global.rs:78-82`：`pub truncated_layers: Vec<(i32, usize)>` / `pub node_cap: u64` / `pub node_cap_hit: bool`；`:642` to_json 输出 `"confidence","bound","expansions","node_cap","node_cap_hit","goals","truncated_layers"`。
confidence 三态实文（`solve_global.rs:450-456`）：

```rust
let confidence = if goals.is_empty() {
    "feasible"
} else if hit_bound || node_cap_hit || !truncated_layers.is_empty() {
    // R1 层界（裁定 A）：任一层被截断/节点上限触界/打界 ⇒ 探索不完整 ⇒ 不冒充 optimal
    "bounded-best"
} else {
    "optimal"
};
```

### 3.2 条件② 确定性判据 ✅
`node_cap_env()`（`solve_global.rs:250-260`，env `SOLVER_NODE_LIMIT`，默认 `unwrap_or(100_000)`）、`layer_node_limit()`（`solve_local.rs:436-447`，env `SOLVER_LAYER_LIMIT`，默认 `unwrap_or(200_000)`，doc 注明「**确定性=同 env 同结果**（截断点=节点计数=遍历序确定函数）」）→ **两上限默认值写死 + doc**，且 `100k / 200k` 与报审口径一致。

### 3.3 条件③ 双跑字节同 + TEMP=0 ✅
`cargo run --release -p game-solver --bin solver -- solve-global --bound 1e6 --out /tmp/rev_sol_run{1,2}.json` **两进程独立跑**：

```bash
$ sha256sum /tmp/rev_sol_run1.json /tmp/rev_sol_run2.json
0b0e8328b9d4e2ae3d0220da206c16be058f2a0929ef29e09182dd67ff3f8e19  run1
0b0e8328b9d4e2ae3d0220da206c16be058f2a0929ef29e09182dd67ff3f8e19  run2
```

**逐字节相同**（各约 5-6 min）。TEMP 残留见 §1（=0）。

### 3.4 条件④ B 提案风险注记 ✅（如实呈报）/ 落库缺口 → Y5
- 报审「六、遗留/风险」原文含：「批4 要真解需 B 提案（拾取槽出键商集，lead 已裁：**须子集上确界代表或限定效果被 hp/gold 支配吸收的槽位，任意先到者保留=不健全**）或支配口径再调」——**lead 点名的两个限定词都在**，与实测披露一致（§10）。
- 但全仓 + 全 git 历史 `grep "上确界|支配吸收|B 提案|子集上确界"` = **0** → 只在对话交付物里，**未落库** → **Y5（挂批4）**，不阻断。

---

## 4. 四类换层边实锚（复核重点④）✅ 绿（F1 单点见 §0.1）

> ⚠️ 锚文件辨析：`reference/src/deobf/a.java` 才是现役 a.java；`reference/seed/a.java` 是**另一份旧文件**（md5 不同、行号不同），抽验时勿混。

抽 3 条 sed 实文（a.java 锚 + Rust 实码对照）：

**（1）楼梯 param（F3，型21/22 上下行）**

```java
// reference/src/deobf/a.java:7863 / :7868
case 21: var2 = this.changeFloor(this.currentFloor + 1, false, false); break;
case 22: var2 = this.changeFloor(this.currentFloor - 1, true, false);  break;
// a.java:7135  private boolean changeFloor(int var1, boolean var2, boolean var3)
//   var3 = strict 区间门（:7138-7148 minFloorReached/maxFloorReached 拒）
// a.java:944   new byte[]{-1,-1,-1,-1,1,1,1,3,1,1,-1,-1,-1,1,1,1,1,1,1,1,1,1}
//   idx8→type21=1、idx9→type22=1（消耗计数）
```

Rust：`crates/game-solver/src/solve_local.rs:99-131` `ExitKind::{Stair,Script,Ending,Cloud,UseItem}` doc 自带全套锚；`useItemStack` 边界 `a.java:7769-7789`。✅（case21/22 `var3=false` = **无区间门**，与成员「楼梯不走 strict」一致）

**（2）筋斗云四段链 + 区间门（键49/55）**

```java
// a.java:5295-5306
if (this.f_bool_13 && (var1 = this.findFloorGateEntity(true)) >= 0) {
    this.changeFloor(this.entityParam[var1] & 255, false, true);   // case49: strict=true
}
// case55: changeFloor(..., true, true)                              // strict=true
// a.java:7115-7133 findFloorGateEntity：首个 type7/8 且 (param>>9)==0
// a.java:5767-5806 f_bool_13：tutorialSeen ∧ 与 stairUp/DownMarker 四邻
// a.java:7538-7539 pickupItemType case15: this.tutorialSeen = true;  // 唯一写点
```

Rust：`crates/game-solver/src/solve_local.rs:1374-1397 cloud_goals`——`item_types.contains(&15)`、`Manhattan==1` 邻梯、首个 `ty7/8 且 (param>>9)==0`、固定序 `[(7,false),(8,true)]`；**区间门不在层内**（doc 原话：min/maxFloorReached 是全局史，由 solve_global 落闸 → `solve_global.rs:383-389`）。✅

**（3）型72 → Ending（场景层）**

```rust
// crates/game-core/src/scene.rs:3891-3894
72 => { if self.persist.floor == 50 { self.load_level_script(65); } }
```

对照 `crates/game-solver/src/solve_local.rs` `ExitKind::Ending` 分支。✅

**（4）StepGate pending·went_down（C3a 落点链）**

```rust
// crates/game-core/src/scene_persist.rs:410-433 swap_floor 落点链
//   floor0 → (1,2)；floor50 → (6,7)；floor1 && !went_down → (6,11)
//   否则 find_floor_gate(went_down) → step_off_stairs
// :434-438   let want = if up { 7 } else { 8 };  +  (param >> 9) == 0
```

**F1 证伪更正格式**：见 §0.1（`plan:343` 需更正，`plan:299` 零改动；当前**缺位**=红，修复中）。

---

## 5. F4 QoL={15,21,23} 收窄（复核重点⑤）✅ 绿

### 5.1 更正注在位 ✅
`docs/superpowers/plans/2026-11-09-solver-implementation-plan.md:62`（整行 blockquote）：

> **【2026-11-12 更正·F4】** QoL 集收窄为可获得子集 **{15,21,23}** … 20/22/24 **世界零获取** … 原主张保留不删（AGENTS §3.6 证伪更正）

`:165`（QoL 行内）同款日期更正、原清单保留。→ **原地更正格式 ✓，行号与成员报审的 :62/:165 精确吻合**；链内触 plan 的唯一笔 `19b1684` 就是这两处（diff=2 hunk）。`crates/game-solver/src/state.rs:40` `pub const QOL_ITEMS: &[i32] = &[15, 21, 23];`。

### 5.2 零获取穷尽对照抽验（独立 A 级复现，成员称 sprite1703+GIN×4+shop17）✅

```bash
$ python3 tools/type_census.py   # 解析 1B type + u16 x + u16 y + extra_len，floor0..54
sprite0..54 共 1703 条记录（floor1..54 = 1701）      # 与成员「1703 记录普查」吻合
type15 → floor1  (320,352) = (10,11)                 # 与成员 15→f1(10,11) 吻合
type21 → floor45 (64,320)  = (2,10)                  # 与成员 21→f45(2,10) 吻合
type23 → floor19 (192,96)  = (6,3)                   # 与成员 23→f19(6,3) 吻合
type20/22/24 全世 0 枚                               # 世界零获取 ✓
$ python3 -c "import json;print(len(json.load(open('data/tables/shop_goods.json'))['rows']))"
17                                                  # 首行 src0/idx1=GiveGold 1000，非 item21
```

### 5.3 交叉引用漂移 → Y2 ✅（零行为）
`state.rs:39` doc 写「计划 **:60/:163** 已原地日期更正」，实际已漂到 **:62/:165**（F4 插行所致）。

### 5.4 证据指针 → Y6（报审件不入库）

---

## 6. 确定性（复核重点⑥）✅ 绿

```rust
// crates/game-solver/src/solve_global.rs:22
use std::collections::{BinaryHeap, BTreeMap, VecDeque};
// :311-316
let mut queue: BinaryHeap<(Reverse<u32>, u32, Reverse<u32>)> = BinaryHeap::new();
let mut by_key: BTreeMap<GKey, Vec<u32>> = BTreeMap::new();
// :571-600 floor_chain_dist：BTreeMap radj + VecDeque BFS 自层50，preds sort_unstable()
```

- `solve_local.rs` **仅两处 HashMap**：`:423 LAYER_MEMO`、`:1877 GATE_MEMO`（RefCell 线程局部 memo），用法只有 `.get().cloned()` / `.insert()`，**无任何迭代参与遍历序**；主搜 `:1173 by_key: BTreeMap<Key, Vec<u32>>` + VecDeque BFS、`:518 groups: BTreeMap`。
- → **无 HashMap 迭代序依赖** ✅。
- 层上限默认 **200_000**（`solve_local.rs:441-447`）、全局节点上限默认 **100_000**（`solve_global.rs:255-259`），均写死 + doc + env 可调 ✅。

---

## 7. 判据 4 命令独立重跑（复核重点⑦）✅ 绿（贴件自证 ≠ 我跑）

### 7.1 测试

```bash
$ cargo test --release -p game-solver          # exit 0
solve_global: 17 passed (25.98s)
solve_local : 11 passed
lib : 7 passed   census_snapshot : 4 passed    # --list | grep -c ': test' = 39
```

与成员口径 lib7 / census4 一致；`solve_global(test)=17`、`total=1075`（1057→1075 = +17 +1(lib 6→7)）与 ratchet JSON 一致。

### 7.2 存在性闸 vs ratchet message（→ Y1）

```bash
$ cargo test -p game-solver --release -- --list | grep -c solve_global
18        # 17 个 tests/solve_global.rs + 1 个 lib 内 solve_local::solve_global_c1::…
```

→ C6h message「存在性闸=18」**成立**；但 `3e7d235` ratchet message 写「solve_global(test)=18」与 JSON `:63 "solve_global(test)": 17` 及实测 17 **不符** → **Y1（零行为，随批修）**。

### 7.3 门禁

```bash
$ python3 gates/cli.py check
PASS=13 FAIL=0 SKIP=0   (exit 0)
  state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同
  test-ratchet   — total=1075(基线1075) 无回退
  original-integrity / assets-manifest / reference-seed-integrity / knowledge-format /
  docs-links / naming-ledger / rust-tests / table-content-hash / table-evidence /
  state-machine-anchors / asset-truth-manifest  全 PASS
```

（本轮复跑超 300s 转后台 job `bash-45894ef11053481b`，结果以本节首跑 + 后台 job 回收为准；见 §11 前的 job 回收。）

### 7.4 solve-global 双跑 diff ✅

见 §3.3：两进程 sha256 同 `0b0e8328…f8e19`。

---

## 8. 变异实验（复核重点⑧）✅ 绿（≥2 处改坏 → 恰 1 红 → 还原）

> 全部改完立刻 `cp` 还原，`git status --porcelain crates/game-solver/src/` 为空（干净）自证。

### 8.1 M1：QOL_ITEMS 常量改坏

```rust
// crates/game-solver/src/state.rs:40
- pub const QOL_ITEMS: &[i32] = &[15, 21, 23];
+ pub const QOL_ITEMS: &[i32] = &[15, 21];
$ cargo test --release -p game-solver
test p2s0_s0_and_q3_constants ... FAILED        # census_snapshot.rs:61-62
test result: FAILED. 3 passed; 1 failed         # 恰 1 红 ✅
```

### 8.2 M2：C6f 隐形 81 放行改坏

```rust
// crates/game-solver/src/solve_local.rs:1009-1017 classify
- 9 | 10 | 11 | 12 => blocked = true,
+ 9 | 10 | 11 | 12 | 81 => blocked = true,
$ cargo test --release -p game-solver --test solve_global
test solve_global_t穿隐形81封印_层10上行梯可达 ... FAILED
test result: FAILED. 16 passed; 1 failed        # 恰 1 红 ✅
```

### 8.3 M3（加分）：confidence 去掉 `|| node_cap_hit` → **0 红**

```rust
- } else if hit_bound || node_cap_hit || !truncated_layers.is_empty() {
+ } else if hit_bound || !truncated_layers.is_empty() {
$ cargo test --release -p game-solver --test solve_global
test result: ok. 17 passed; 0 failed            # 0 红 → 覆盖缺口 = Y3
```

原因（可复现推理）：现有用例里 `node_cap_hit=true` 总与 `goals.is_empty() → "feasible"` 先行分支共现，`bounded-best` 支路在 solve_global 测试里**无一条断言**（只有 `tests/solve_local.rs:703` 单层版钉了）。成员 `tests/solve_global.rs:1096` 注已自认「有解时才可能 optimal→bounded-best」。

---

## 9. 口径面（复核重点⑨）

### 9.1 C6d 链距优先 ✅（B 级）
`solve_global.rs:311-316` 队列 `BinaryHeap<(Reverse<u32>, u32, Reverse<u32>)>`（距离为主键、depth 次之、Reverse(id) 破平）+ `by_key: BTreeMap<GKey, Vec<u32>>`；`floor_chain_dist` :571-600 注释原文：「±1（符21/22 近似）会把 dist[f51] 压成 1…**符21 分支在纯楼梯图里 dist=MAX=最后探索（仍探索=完备性不依赖它）**」→ 自创启发式，但**完备性论证在位**，不因排序损失解 → A/B 级可落。

### 9.2 C6f 隐形 81 放行 ✅（A 级）
- 生产锚 `crates/game-core/src/scene.rs:3450-3451`「隐形期 continue 放行（floor10 南北隐形封印门可穿，**迭代 65 差分实证**）」。
- `crates/game-core/src/entity.rs:342` `table.items[idx].visible = true; // spawnEntity 默认可见（m_122 特例分支随后置 false）`。
- 求解侧 `solve_local.rs:1009-1017`：`9 | 10 | 11 | 12 => blocked = true,`，81 臂注（spawn 即隐形 / 仅 `op_swd` 改 visible 同帧 type→4）→ `_ => {}`。
- 变异反证见 §8.2（恰 1 红）。

### 9.3 C6g 根在出口 ✅（A/B 级）
`crates/game-solver/src/solve_local.rs:1147 fn run_root`、`:1208 if id != 0 { … continue; }` —— 根态恰在**记录出口之后**仍扩展，根不被跳过。

### 9.4 C6h 节点上限 + R11 字面偏离裁定
- C6h：`solve_global.rs:250-260` 默认 `100_000`、`solve_local.rs:436-447` 默认 `200_000`，写死 + doc ✅。
- **R11「区间门 4 分量入 `SolverState.flags`」字面偏离 → 我裁定：黄，不阻断（canonical 理由成立）**：
  - `plan:421` R11 要求 4 分量入 flags；`plan:178` 表定义 4 分量 = `tutorialSeen / f_bool_13 / findFloorGateEntity / changeFloor strict`（锚 `a.java:7538-7539, 5767-5806, 7115-7133, 7139-7151, 5295-5306`）。
  - 实现改为：`cloud_goals`（`solve_local.rs:1374-1397`）派生前三者 + 区间门在 `solve_global.rs:383-389` 由 `st.floors` min/max 落闸；`solve_global.rs:11` doc **仍写「4 分量入 flags」** = 措辞与实现不符。
  - **canonical 理由（行为等价、双源分歧风险）**：`tutorialSeen` 权威源=背包 `item_types.contains(&15)`（唯一写点 `pickup case15 a.java:7538-7539`），再抄进 flags 就是**双源**；`f_bool_13` 是逐 tick 派生几何（`a.java:5767-5806`），存值会与重算结果分叉；`findFloorGateEntity` 是**层数据**非状态；`changeFloor` 的 min/maxFloorReached 是**全局史**，已归 `st.floors`。塞 flags 只会**污染 K₀ 的 ScriptFlags 键域**（与脚本位争键）而无行为增益。
  - → 结论：**实作 canonical、doc 措辞错** → **Y4（随批修 doc）**，不阻断推送。

---

## 10. 遗留段一致性（复核重点⑩）✅ 绿

报审「六、遗留/风险」（lead 转发） vs 我复跑 sol1 JSON：

```json
{"confidence":"feasible","bound":1000000,"expansions":53311,"node_cap":100000,
 "node_cap_hit":true,"goals":[],
 "truncated_layers":[[54,200000],[3,200000],[4,200000],[5,200000],[6,200001],
                     [7,200000],[8,200001],[9,200000],[9,200001]]}
```

| 报审披露 | 我的实跑输出 | 判定 |
|---|---|---|
| `goals=[]`（批3 判据只要求确定性） | `goals:[]` | ✅ 一致 |
| f54 拾取变体 2^18 爆炸（truncated_layers 实录） | 有 `[54,200000]` | ✅ 一致 |
| f3-f9 fat layers（200k 截断） | `[3..9]` 全在（6/8 触 200001） | ✅ 一致 |
| 判据③依赖 C6h 节点上限（默认100k） | `node_cap:100000`、`node_cap_hit:true` | ✅ 一致 |
| **f53 16 怪 = 下一颗雷（未实测报数）** | JSON **无 floor53 截断条目**、`expansions:53311 < 100000` | ✅ 不矛盾（如实标注未实测，未编造数字） |
| e2e 不断言 goals 非空 | 与 §8.3 覆盖缺口同源 | ✅ 已披露 |

→ **披露与实跑输出逐字段一致，绿。**

---

## 11. F1 修复复验记录（commit `71908d6`）✅ 通过

```bash
$ git log -1 --format='%h %s' 71908d6
71908d6 docs(plans): 批3 复核红1 修复——plan:343 LAY/GUT 换层边原地日期更正
$ git show 71908d6 --stat
 docs/superpowers/plans/2026-11-09-solver-implementation-plan.md | 2 ++
 1 file changed, 2 insertions(+)          # 纯插入，0 删除
$ git diff da60d84..HEAD --stat
 docs/superpowers/plans/2026-11-09-solver-implementation-plan.md | 2 ++
```

**11.1 更正注四要素（`plan:344` sed 实文）** ✅

```markdown
> 【2026-11-13 更正·复核红1】原句「换层边 = 楼梯 param + LAY/GUT + 型72」中
> GUT 非换层边：`op_gut`=load_level_script 脚本槽跳转（script_ops.rs:167，
> findings floor-script-maplv:122），门脚本内由 tickdrive 链自然跟随；
> LAY 确为换层（op_lay=save_floor_state+set_pending_floor+set_transitioning，
> script_ops.rs:68；a.java:6261 q=5 为其渲染相位）但经既有 pending_floor 通道
> 被 solve_local gate_arrive 的 Drive::Pending 分支统一捕获，批3 不建独立边。
> 批3 按机制建四类边：楼梯 param/StepGate/筋斗云/UseItem（F1 偏离见批3 报审）。
> slot0 LAY_2 执行方待考证。
```

- ① **GUT 非换层**（`op_gut = load_level_script` 脚本槽跳转）✅
- ② **LAY 确为换层 + pending_floor 机制**（`Drive::Pending` 统一捕获、不建独立边）✅
- ③ **原句未删**（`@@ -341,6 +341,8 @@` 纯插入，原 bullet 原样在位）✅
- ④ **日期格式** `【2026-11-13 更正·复核红1】` ✅

**11.2 `plan:299` 零改动** ✅

```bash
$ git show da60d84:docs/…/2026-11-09-solver-implementation-plan.md | sed -n '299p'
$ sed -n '299p' docs/…/2026-11-09-solver-implementation-plan.md
  - **A 状态变换（进转移表）**：… ROS_1(瞬移)、LAY(换层)、SWD(封印门解除)、…
（两行逐字节相同；diff 全量仅 +2 行更正注）
```
→ 「LAY(换层)」真主张未被改错（改错会记红）✅

**11.3 commit message 证伪链 + 证据字段** ✅

- 「证伪链自记：批3 报审 F1 原话『LAY/GUT 均非换层』的 **LAY 半边自我证伪**（当时只看 a.java q=5 片段未看 op_lay 全貌）」—— 原话级自记，符合 AGENTS §3.6 证伪更正文化。
- `Evidence: crates/game-core/src/script_ops.rs:68-73(op_lay)/:167-170(op_gut); reference/seed/a.java:6261-6268(LAY handler q=5); docs/findings/2026-11-09-floor-script-maplv-numbering.md:98/:122` + `Confidence: 95` ✅（commit-msg 证据门禁可过）。
- 我逐条 sed 核锚：`op_lay` fn `:68`（body `:69 save_floor_state` / `:70 set_pending_floor` / `:71 set_transitioning` / `:73 set_dialog_phase(5)`）✅；`op_gut` fn `:167`（`:169 load_level_script(target)`）✅；findings `:98`「`LAY_2`（`script_ops.rs:68 op_lay` = `save_floor_state` + `set_pending_floor`）」、`:122`「`GUT_n` 目标同为 script_idx；`LAY`/`ADD`/楼梯 param 才写 `floor` 字段」✅；`reference/seed/a.java:6261` = `else if (string.equals("LAY")) { … this.q = (byte)5; }` ✅。

**11.4 锚文件辨析（→ Y7，黄）** ⚠️

- `reference/seed/a.java` = **D3 不可变锚点**（`gates/cli.py:175-185 reference-seed-integrity`：「seed/a.java 是全部差分验证的不可变投影基准」）；`reference/src/deobf/a.java` = 反混淆可读版，**两份行号不同**。
- 全仓文档的裸 `a.java:NNNN` 主流约定 = **deobf**（findings `a.java:7770-7789`、plan F3 `a.java:7864` 均落在 deobf；`script_ops.rs:67` doc 对同一 LAY handler 写 `a.java:8355-8362`，实际 deobf `:8361-8369` = `if (var3.equals("LAY")) { m_119; pendingFloor=…; floorTransitioning=true; dialogPhase=5; }`）。
- 故 `plan:344` 内联「a.java:6261」按主流约定会读到 deobf 的实体数组初始化 → **Y7（随批修，改标 `reference/seed/a.java:6261` 或 deobf `a.java:8361-8369`）**；因 commit `Evidence:` 已显式限定 seed，**可查、不阻断**。

**11.5 门禁复跑（我自己跑的）** ✅

```bash
$ python3 gates/cli.py check          # 后台 job bash-45894ef11053481b（71908d6 之后跑）
[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同
[✓ PASS] state-machine-anchors / asset-truth-manifest(138 entries)
== 汇总 == PASS=13 FAIL=0 SKIP=0
GATES_EXIT=0
```

- 本报告 §7.3 首跑（修复前）与本次后台复跑（修复后）**两次独立 PASS=13**；ratchet `total=1075` 取自 §7.3 首跑（此后仅落 1 笔 docs，测试数不变）。

**→ R-1 关闭：总判由红转黄。**

---

## 12. 结论

- **总判：🟡 黄 — 可推送。红 0 项，黄 7 项（全部零行为）。**
- **黄 7 项**：
  - **随批修**：Y1（`3e7d235` message「solve_global(test)=18」与 JSON/实测 17 不符）、Y2（`state.rs:39` 引用行号 `:60/:163` 已漂移为 `:62/:165`）、Y4（`solve_global.rs:11` doc「4 分量入 flags」与实作不符，R11 canonical 裁定见 §9.4）、Y6（F4 对照证据指针指向不入库的报审件）、Y7（`71908d6` 内联锚 `a.java:6261` 行号约定歧义）。
  - **挂批4**：Y3（confidence `|| node_cap_hit` 支路变异 0 红 = 覆盖缺口，有解路径属批4 e2e）、Y5（A-lite④ B 提案风险注记未落库）。
- 十项复核面：①链与夹带 ②TDD 红档 ③A-lite 四条件 ④四类换层边实锚（F1 单点已修复复验）⑤F4 收窄 ⑥确定性 ⑦判据 4 独立重跑 ⑧变异≥2 ⑨C6d/f/g/h+R11 裁定 ⑩遗留段一致性 —— **全部实测通过**。
- **推送建议：可推送**（黄项按上表随批修/挂批4；无红项阻断）。
