# 二轮对抗复核：确定性求解器实施计划（修订版 4398b60）

- **对象**：`docs/superpowers/plans/2026-11-09-solver-implementation-plan.md`（**501 行**，commit **`4398b60`**，基于首版 `dbc8d86` 的 1 文件 **+56/−27**，未推送）
- **首审报告**：`docs/findings/reviews/2026-11-09-solver-plan-review.md`（红 3 / 黄 14 / 绿 17）
- **复核人**：独立二轮攻审（只读取证 + 仅写本报告，未改其他任何文件）
- **复核时仓库状态**：分支 `modern`，HEAD = `4398b60`（`git rev-parse --short HEAD` 实测），`modern...origin/modern [领先 4]`；工作树有**他人在途改动**（` M data/tables/items.json`、` M docs/bug-ledger.md`、` M gates/table_content_hash.json`、` M tools/item-defs/gen_items.py`、`?? docs/findings/2026-11-09-qol-passive-item-triggers.md`）——本复核全程只读，计划文件与 HEAD 无 diff（`git diff 4398b60 -- <plan>` = 空）。
- **方法**：全部 `sed -n` / `grep -n` / `git show` / `python3 gates/cli.py` 现场实测，**不转引**任何简报；粘贴均取自本次工具输出原样。

---

## 总判：**放行**（可进入批0）

| 维度 | 结论 |
|---|---|
| 三红（QoL 口径 / 逐层五元组分叉判别器 / items 22-25 effect 红线） | **3/3 已修**，全部落文且带证据锚 |
| 五命令修正 | **5/5 已修**，残留模式 grep 全 = 0 |
| 空过修复 | **6/6 处存在性闸已配**（批1/批2/批3/批5/A6/A10，另 A11 新增亦配）；A1–A3 内嵌 python assert；A9 给具体命令；批1 TDD 已改 A 类红测试 |
| 回归 | `git show --stat 4398b60` **恰 1 文件**；diff 23 行删除**全部为替换式改写**（无绿内容被删）；首版绿项（面2 抽验结论/批0 判据/§7 A1–A10 主体）全保留 |
| 门禁 | `python3 gates/cli.py check` → **PASS=13 FAIL=0 SKIP=0（exit 0）**；`status` → PASS=13 FAIL=0 SKIP=1 TOLERATED=0（与计划 §2.9 声明一致） |

**无红、无需再修**；仅 3 条非阻断残留观察（见文末，均非本次修订引入或不影响判据执行）。

---

## 一、三红核销（逐条 → 已修/未修 + 证据行号）

| 红 | 判定 | 证据（`grep -n`/`sed -n` 实测行号，指 `docs/superpowers/plans/2026-11-09-solver-implementation-plan.md`） |
|---|---|---|
| **红1** QoL 口径（解合法性=通关+QoL 全收集，筋斗云可晚拿不可不拿） | **已修** | **§0:21** `- [ ] **解合法性口径（三查询共同前提）**：合法解 = **通关 + QoL 全收集**（筋斗云**可晚拿不可不拿**，m03276 裁定）；Q1/Q2/Q3 的"存在路线"均指存在满足此口径的路线（§1.2）`；**§1.2:60** 展开为细则（含"晚到终点前补拿合法，整条路线不拿 = 非法解" + 考证件指针 `docs/findings/2026-11-09-qol-passive-item-triggers.md` §6）；**三查询约束**：:54 Q1 约束列 `存在**满足 §1.2 解合法性口径（通关+QoL 全收集）**的通关路线`、:55 Q2 `同上…（**Q2 不得靠"漏拿 QoL 件"取巧**）`、Q3 约束列 `**且满足解合法性口径**`（表行 54–56 实读）；:30 Review Focus 第 3 条、:32 第 5 条复述。**`grep -c "QoL\|全收集\|可晚拿"` = 9，命中行 21/30/32/54/55/60/163/169/418**（首审时仅命中行 159 的物品清单一处）。 |
| **红2** 逐层五元组分叉判别器（缺=红，m03266 口径） | **已修** | **§4 批5:382** 原文实读：`**逐层五元组分叉判别器（m03266 裁定）**：求解器输出**操作脚本**（非结论）→ 真引擎回放 → **每层落 checkpoint 断言五元组（血/攻/防/三色钥匙/金币，字段源 combat.rs:51）的求解器预测值 == 真引擎实测值**——任一层分叉 = **求解器 bug**（不是游戏 bug），判失败并输出分叉层与两侧数值。整路线到 Ending 只是必要条件，本项是充分判别器`；批5 判据配双闸 :391 `cargo test -p game-oracle -- --list \| grep -c checkpoint # 预期：≥1` + :393 `cargo test -p game-oracle checkpoint # 预期：逐层 0 分叉`；**验收表新增 A11:458**（判据含存在性闸 + `逐层 0 分叉（任一层分叉=求解器 bug，判失败）`）。字段源 `crates/game-core/src/combat.rs:51 = pub struct PlayerCombat {`（sed 48–62：hp/atk/def/yellow_keys/blue_keys/red_keys/gold… 实测在位）。 |
| **红3** items.json 22-25 effect 红线 | **已修** | **§0:20** 红线条文实读：`**items.json 22-25 effect 字段禁作求解器效果源（红线）**：docs/bug-ledger.md:1035 BUG-053「items.json effect 映射错位（22-25 四件）」明文"求解器若按 items.json effect 建模=按错效果算"（529dfd5 登记；22 已由 6efd30e 修、23/24/25 仍待被动判定点考证）…台账该条关闭前，批0 ingest 对该四件不得信 effect key（§2.7 同步标注）`；**§2.7:167** ⚠️ 标注（effect key 仅作清单索引，不作效果真相源）；**§2.7 四条必须建模表 :169–176**（169 引标题、171–172 表头、**173–176 = 25 幸运币 / 23 太公杖 / 24 捆仙绳 / 15 筋斗云** 四行）；**R12:419**（items 22-25 effect 不可信，对策=§0 红线 + ingest 不信 effect key）、**R11:418**（筋斗云独立换层通道）；**批3:340** 层间 DAG 显式补 `+ **筋斗云邻梯瞬移——独立于踩梯的换层通道，带探索区间门 4 分量，§2.7 表**`。 |

**外部锚点复证（不转引）**：

- `docs/bug-ledger.md:1035` = `## BUG-053 items.json effect 映射错位（22-25 四件·Stage B）`（`sed -n '1030,1040p'` 实读）；`git show 529dfd5 --stat` → `docs/bug-ledger.md | 51 +++++` 单文件登记 ✓（登记早于首版 `dbc8d86`，首审判其"计划与台账直接冲突"成立，现已消解）。
- `docs/findings/2026-11-09-qol-passive-item-triggers.md`：`## 6.` 在 **562**、`### 必须建模` 564、四行 568/569/570/571（25 幸运币 568、15 筋斗云 571，`grep -n` 实测）——计划 §2.7:169 引 **§6:562-571** 与现场一致 ✓。

---

## 二、五命令修正核销

| # | 项 | 判定 | 证据（实测） |
|---|---|---|---|
| 1 | §6:431 包名 `-p solver` → `-p game-solver --bin solver` | **已修** | :431 `→ cargo run --release -p game-solver --bin solver -- query --all   # 三查询重跑`；全文出现于 283/327/350/351/369/431/448/449/450/451 共 10 处，形态统一 |
| 2 | A3 `--ultimate36,40` → `--ultimate 36,40`（有空格） | **已修** | :450 `` `... -- query q3 --ultimate 36,40 \| python3 -c "…assert d['query']=='q3' and isinstance(d['reachable'],bool)…"` `` |
| 3 | A5 `--query --all` → `query --all` 统一 | **已修** | :451 `... -- query --all --out …`、:452 `连跑 2 次 query --all 到不同文件后 diff` |
| 4 | `scene_persist.rs:383` → `:398`（save_floor_state，自行 sed 实测） | **已修** | 计划内 :152 `save/swap/reload 链 :398/:410/:444`、:176 `scene_persist.rs:349 / 435 / 398 / 444`、:230 `scene_persist.rs:398/444`、§8:476 `**398(save_floor_state)**`；**现场实测**：`grep -n "fn save_floor_state" crates/game-core/src/scene_persist.rs` → 398；`sed -n '395,401p'` 第 398 行 = `    pub(super) fn save_floor_state(&mut self) {` |
| 5 | `battle.rs:189-192` → `subsystem/battle.rs:190`（gold<<1，自行 grep 实测） | **已修** | :119 `crates/game-core/src/subsystem/battle.rs:190`（实测 `ctx.enemy_base[idx].gold << 1`；**`crates/game-core/src/battle.rs` 文件不存在**，勿引）；**现场实测**：`sed -n '186,193p' crates/game-core/src/subsystem/battle.rs` → 190 行 = `            ctx.enemy_base[idx].gold << 1`；`ls crates/game-core/src/battle.rs` → 不存在 ✓；§8:476 同步补 `subsystem/battle.rs:…,**190(gold<<1)**` 与 `crates/game-core/src/battle.rs 文件不存在（ls 实证）` |

**残留模式 grep（应全 = 0，实测）**：`-p solver` = **0** · `ultimate36` = **0** · `:383` = **0** · `battle.rs:189` = **0** · `--query ` = **0**（`grep -c` 逐项执行，最后一项 0 致 grep exit 1 属正常）。

---

## 三、空过修复核销

| 项 | 判定 | 证据（行号实测） |
|---|---|---|
| 批1 存在性闸 | **已配** | :307 `cargo test -p game-solver --lib -- --list \| grep -c script_abs   # 预期：≥1（过滤器存在性闸，防空过）`；:309 同式 `grep -c 对拍` |
| 批2 存在性闸 | **已配** | :329 `cargo test -p game-solver -- --list \| grep -c solve_local    # 预期：≥1（存在性闸，防空过）` |
| 批3 存在性闸 | **已配** | :348 `cargo test -p game-solver -- --list \| grep -c solve_global   # 预期：≥1（存在性闸，防空过）` |
| 批5 存在性闸 | **已配** | :390 `… grep -c solver_replay   # 预期：≥1（批5 新增后；未写时此闸红=防空过，禁止 0 匹配 exit 0 当绿）`；:391 `… grep -c checkpoint` |
| A6 存在性闸 | **已配** | :453 `` `cargo test -p game-oracle -- --list \| grep -c solver_replay`（≥1 存在性闸）+ `cargo test -p game-oracle solver_replay` ``，并标 **批5 新增** |
| A10 存在性闸 | **已配** | :457 `` `cargo test -p game-solver --lib -- --list \| grep -c 对拍`（≥1 存在性闸，防空过）+ … `` |
| 通则 | **已落文** | :461 `> 所有 cargo test … <过滤器> 判据一律配 -- --list \| grep -c **存在性闸**（0 匹配 exit 0 ≠ 绿）` |
| A1–A3 内嵌 python assert | **已配** | :448 A1 `… \| python3 -c "import json,sys;d=json.load(sys.stdin);assert d['query']=='q1' and isinstance(d['max_hp'],int) and d['confidence'] in ('optimal','bounded-best','feasible');print(d)"`；:449 A2（`isinstance(d['min_kills'],int)`）；:450 A3（`isinstance(d['reachable'],bool)`）；:460 注 `A1–A3 命令内嵌 python assert（Y3-1 修正）：单查询行必须做 JSON 结构断言，禁止"打印就算过"` |
| A9 具体命令 | **已配** | :456 `` `cargo tree -p game-solver -i game-solver`（预期仅 game-solver 自身…）+ `grep -rnE 'std::fs\|std::net\|std::process\|std::time' crates/game-solver/src --include='*.rs' \| grep -v 'bin/solver.rs'` ``，预期 `tree 仅自身；grep **0 命中**`（首审"lib 无 fs/net 断言测试"的含糊判据已换成可复制命令） |
| 批1 TDD 改 A 类红测试 | **已配** | :301 `（**A 类"改状态"断言作 TDD 红测试；B 类"状态不变"只作回归护栏——天然空过，不得当红**；C 类按约束）`；**:314** `**TDD**：先写 A 类**改状态**红测试（RES easy 重置 / GIN 给金给物 / DES 删实体——断言状态字段变化，未实现时必红）→ 实现转移表至绿；B 类"状态不变"（GLV/SMS 空体等）作回归护栏随实现补上，不作红测试`（首审 Y1-7 "GLV/SMS 天然空过当红"已消解） |

---

## 四、回归核销

| 项 | 判定 | 证据 |
|---|---|---|
| `git show --stat 4398b60` 恰 1 文件 | **PASS** | `docs/superpowers/plans/2026-11-09-solver-implementation-plan.md \| 83 +++++++++++++++-------` → `1 file changed, 56 insertions(+), 27 deletions(-)`（与宣称 +56/−27 一致） |
| diff 全貌 = 纯增改、无删绿内容 | **PASS** | `git diff dbc8d86 4398b60 -- <plan>` 非头删除行共 **23 行**，逐行核对**全部为"替换式改写"**（每条 `-` 均有内容更全的 `+` 对应：Review Focus 2/3 条、Q1–Q3 约束列、Bump/Route/Exchange 行、掉落金行、语料规模行、change_floor 行、§2.8 change_floor 行、§3 分层理由行、批1 对拍命令行、批1 TDD 行、R4 行、§6 回归门命令、A1/A2/A3/A5/A6/A9/A10、§8 复核清单行）——**无任何绿内容被整段删除** |
| 首版绿项保留 | **PASS** | **批0 判据**：:283–286 `cargo test -p game-solver` / `… ingest --summary` / `# 预期输出（节选）：floors=55 enemies=502 scripts=68 enemies_table=35 items=32 shops=17` / `cargo tree …` / `python3 gates/cli.py check` 逐行原样在位；**§7 A1–A10 主体**：:448–457 十行全在（A11 为新增第 11 行），A4/A7/A8 未动；**面2 十条抽验所依赖的事实**：§2 census（:92 总记录 1703/敌人 502/type83=52/type72=4 等）、语料规模行（:135 归属改写后 raw 297/DES41 + fixture 296/DES40 双侧并列，首审附带黄已修）、§2.9 门禁声明（:191 PASS=13 FAIL=0 SKIP=1 TOLERATED=0）全部在位 |
| 行号锚抽 5 条 sed 实测 | **PASS** | 见下节（实测 6 条，全中） |

---

## 五、我实测的行号锚（`sed -n` 原样，6 条 ≥ 要求 5 条）

1. **`crates/game-core/src/scene_persist.rs:398`** → `    pub(super) fn save_floor_state(&mut self) {`（`sed -n '395,401p'`）——计划 :152/:176/:230/§8 的 `:398` 正确（首审 Y1-5 已修）。
2. **`crates/game-core/src/subsystem/battle.rs:190`** → `            ctx.enemy_base[idx].gold << 1`（`sed -n '186,193p'`）；且 `ls crates/game-core/src/battle.rs` → 不存在（首审 Y1-6 已修）。
3. **`crates/game-core/src/combat.rs:51`** → `pub struct PlayerCombat {`（:52–58 = hp/atk/def/yellow_keys/blue_keys/red_keys/gold）——五元组字段源成立。
4. **`crates/game-core/src/scene.rs:2045 / 3592 / 3739`** → 依次 `… BattleEvent::RouteEngage { entity_slot: slot }` / `… BattleEvent::BumpEngage { entity_slot: e as i32 }` / `… BattleEvent::Exchange`（`sed -n '2045p;3592p;3739p'`，输出按行号升序）——与计划 :116 行 `三种开战面：… 调用点实测 scene.rs:3592(Bump) / scene.rs:2045(Route) / scene.rs:3739(Exchange)` 一致。
5. **`docs/bug-ledger.md:1035`** → `## BUG-053 items.json effect 映射错位（22-25 四件·Stage B）`（`sed -n '1030,1040p'`）——红3 引用的台账行锚准确。
6. **`crates/game-core/src/combat.rs:106`** → `pub fn predict_battle_hp_loss(`（`sed -n '104,108p'`）——§2.4 公式锚仍在（抽验性质）。

---

## 六、闸结果

```
$ python3 gates/cli.py check        # 串行单独跑（无并发干扰），耗时约 10 min
[✓ PASS] original-integrity / assets-manifest / reference-seed-integrity /
         knowledge-format / docs-links / naming-ledger / rust-tests /
         test-ratchet / table-content-hash / table-evidence /
         state-machine-anchors / asset-truth-manifest / state-baseline
== 汇总 ==
PASS=13 FAIL=0 SKIP=0
EXIT=0
```

- **宣称 PASS=13 FAIL=0 属实，无红**，故**无需**跑 `state-baseline` 判并发假红（该预案未触发）。
- `python3 gates/cli.py status` 补跑 → **PASS=13 FAIL=0 SKIP=1 TOLERATED=0**，`[- SKIP] visual-diff — 视觉比对器未就绪（P4 启动后启用）` —— 与计划 :191 §2.9 声明逐字一致。
- **并发披露**：复核期间他人在途改动 4 个已跟踪文件（items.json / bug-ledger / table_content_hash / gen_items.py），且 HEAD 领先 origin 4 个 commit；`rust-tests`/`test-ratchet` 的 `total=1025(基线978)` 即来自并行新增测试——**门禁仍全绿，未出现假红**。
- `test-ratchet` 附带备注 `新增二进制待 ratchet-update: {'shop_goods_defs(test)': 6}` 仍为 PASS（计划 :191 已记录同族备注），非本次问题。

---

## 七、残留观察（非阻断，均不改总判）

1. **A8 括注与 `check` 实况略有出入（首版同文，非本次引入）**：:455 A8 预期写 `FAIL=0（visual-diff SKIP 显式打印，属已知项）`，但 `gates/cli.py:586-597 cmd_check` 的 13 个门**不含 `gate_visual()`**（`:524` 才在 status、`:605` 才在 all）——实测 `check` 输出 `SKIP=0` 且无 visual-diff 行；`SKIP=1 显式打印` 只发生在 `status`/`all`。A8 的核心判据 `FAIL=0` 成立，仅括注措辞不准。
2. **§2.9 快照漂移**：:191 记 `test-ratchet total=1022(基线978)`，现为 **1025**（他人并行提交所致）。计划该行是 2026-11-09 时点快照，非错误，批0 执行时以现场 `status` 为准即可。
3. **§8 的 qol 引用范围略宽**：§8:476 写 `qol-passive-item-triggers.md §6（:550-598 实读…）`，而 `## 6.` 实际在 562（550 是 §5 尾部条目）；§2.7:169 的精确引 `§6:562-571` 与现场一致，四条必须建模行 568–571 全在 562–571 内。属"实读范围"表述宽于"章节范围"，不影响可核性。

---

**纪律声明**：本复核只读取证，仅新建本报告 `docs/findings/reviews/2026-11-09-solver-plan-review-r2.md`；未修改计划、台账、基线或任何代码/数据文件；所有行号与命令均为现场 `sed`/`grep`/`git` 实测，无转引。
