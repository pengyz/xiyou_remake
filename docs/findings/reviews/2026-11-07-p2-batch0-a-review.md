# P2 批0① 对抗复核报告 — commit 0c9080f

> 被审件：`0c9080f` fix(data,docs)「P2 批0① 归属表 enemy_base 同步+计划两黄（复核随批0）」（modern 分支）
> 复核方式：对抗复核（只读；唯一写=本报告）；证据 A/B 级带实测输出；纪律=AGENTS.md
> 日期：2026-11-07 · HEAD=`0c9080fd969f4c8f8dc1ae7ea6048a640dc17edc` · parent=`b918c2e7306bf1ce0c2278256214f1440e77a781`

## 总判：可推送

七面全绿（含 1 条黄-轻待裁定，不阻断）：R3 闭合实测 GREEN 186/186 exit=0、修前红态独立复现逐字一致、diff 手术式恰 7+/12-、新条措辞同源属实、186 算术三方一致、计划两黄闭合且 2652 零残留、夹带/历史干净、闸面 13/13+金样 74+ratchet 978 三数全真。

---

## 面1 R3 闭合实测 — 🟢 绿

**修后绿态（独立重跑）**：

```
$ python3 tools/field_ownership.py --check
[GREEN] 归属表 186/186 字段唯一归属（零双写/零缺口）
EXIT=0
```

**修前红态（独立复现，非手抄贴件）**：`git show b918c2e:data/status/field-ownership.json` → `/tmp/rev_parent_table.json`，importlib 加载 `tools/field_ownership.py` 后将 `m.TABLE` 指向该临时文件跑 `main()`（`sys.argv=["--check"]`）→ 输出逐字：

```
[RED] 归属表：表缺字段 1：['enemy_base']
      表多余字段 2：['enemy_base_hp', 'enemy_base_gold']
[exit=1]
```

与 commit 贴件红态（表缺 enemy_base/表多余 enemy_base_hp+enemy_base_gold）逐字一致。A 级（工具实测输出）。

**diff 手术式核**（`git show 0c9080f --numstat`）：

```
5  10  data/status/field-ownership.json
2   2  docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md
合计 7+/12-  ← 恰=声称
```

- JSON 文件只动 3 个字段行（`cat -A` 核变更行）：删 `"field": "enemy_base_hp"`、删 `"field": "enemy_base_gold"`、加 `"field": "enemy_base"`；**无缩进重排、无全量重写**（每条 entry 5 行 {field,domain,why} 结构原位增删，上下文行未触碰）。
- 两文件是 commit 仅有的 2 files（`--name-only` 核）。

## 面2 新条措辞同源性 — 🟢 绿

新条逐字实文：

```json
{"field": "enemy_base", "domain": "combat", "why": "combat 纯计算域（combat.rs to_records 聚合 EnemyDef；battle 初始血/金/trait 源）"}
```

同源核验（B 级，逐锚实存）：

| why 子句 | 实锚 | 实文 |
|---|---|---|
| combat.rs to_records 聚合 EnemyDef | `crates/game-core/src/combat.rs:24` | `pub fn to_records(&self, gold: &[i32]) -> Vec<game_data::defs::EnemyDef>`（:25-34 遍历 `enemy_base_hp.len()` 聚合 id=41+i/hp/atk/def/gold/`trait_flags_for(41+i)`） |
| battle 初始血源 | `scene.rs:3642` | `self.battle_enemy_hp = self.enemy_base[(t - 41) as usize].hp;` |
| 金源 | `scene.rs:2311/2313/3793/3795`、`scene_menu.rs:658` | `.get(...).map(|r| r.gold)` |
| trait 源 | `scene.rs:3741-3743` | `.get((t - 41) as usize).map(|r| r.trait_flags)` |
| 字段本体 | `scene.rs:307` | `pub enemy_base: Vec<crate::combat::EnemyDef>,`（:306 doc=「敌方基础记录（gold/trait 源+battle 初始血源 3642 面）」） |

装配面旁证：`shell/common/handoff.rs:137`、`undo_ui.rs:251` `sc.enemy_base = tables.to_records(&__gold)`；`combat.rs:40 pub use game_data::defs::EnemyDef`。

旧两条确为遗留：`b89d0df`（P1-6② 敌人记录切换）`git show -- scene.rs` 显示 `-pub enemy_base_hp: Vec<i32>` / `-pub enemy_base_gold: Vec<i32>` 及其读点删除。**措辞属实、无失实（claims 病面阴性）**。

## 面3 186 算术 — 🟢 绿

- 父表 count=**187**（enemy_base 缺、hp/gold 在）；HEAD 表 count=**186**（enemy_base 在、hp/gold 缺）、无重复字段。
- 独立数两路：`struct_fields()` = 186 unique；table entries = 186 unique。
- 187−2+1 = **186** ✓ 三方一致（工具输出 / 结构体扫表 / JSON 计数）。
- 表元数据：`schema=field-ownership-1`，`rule=每字段唯一归属（v3 §1 域切纪律）；双写=同字段写点跨归属域文件`。
- `--report` 重跑 exit=0：多文件直写=44（含 shell/oracle 收件面）、生产面 self 直写跨文件=**0**。

## 面4 计划两黄闭合 — 🟢 绿（+1 黄-轻待裁定，见黄项清单）

**:96 锚 ±1**：plan `:96` 实文含 `scene.rs:2653 \`match (src, idx)\``（python `line[:80]` 与 commit 贴件逐字一致）。`sed -n` 场景核 `crates/game-core/src/scene.rs`：**2653 行=`match (src, idx) {` 实存**（2652=`if flags & 2 != 0 {`，2651=`if buy {`，2650=空行；2640 起 `apply_merchant_offer`：`(0,1|15|21)=>gain_gold 1000`、`(1,3)=>spend_gold 50+blue_keys`、`(1,4)=>yellow_keys+5` 商人商品属性逐臂）。B 级。

**:61 宽口径注记**：plan `:61` 实文=`  scene() 构造器已存=166 调用（宽口径；真调用点 140），主体为测试逻辑非"直填"）→**批10 收敛**`（与 commit 贴件逐字）。166 复现：`grep -rE "scene\(" crates/game-core/src/scene_tests/ | wc -l`=**166**；`\bscene\(`=141；`fn scene\(` 仅 1 行（`scene_test_support.rs:152`）→ 真调用点 **140** ✓。A 级（命令输出）。

**2652 全文零残留**：
- `grep 2652` plan → exit=1（零）；`docs/` md（除 findings/reviews）→ exit=1（零）。
- `crates/ shell/ tools/` 代码 7 处命中全为 fixture 帧号/TICK/哈希片段（`story-fullrun.frames.txt:2656`、`walk_pure_excerpt.txt:2719` 等）——非锚、语义无关。
- `scene.rs:2652` 字面仅存于 `findings/reviews/2026-11-07-p2-plan-review-2.md:37,238`——历史复核记录、语义仍确（指旧错锚），不属残留。

## 面5 贴件重跑 — 🟢 绿

判据命令全部独立重跑（同命令同输出，非手抄）：

| 贴件声称 | 独立重跑 | 结果 |
|---|---|---|
| `--check` [GREEN] 186/186 exit=0 | 同命令 | ✅ 逐字一致 |
| 红态 表缺 enemy_base / 表多余 hp+gold exit=1 | 父表注入复现 | ✅ 逐字一致 |
| diff 7+/12-（两文件） | `git show 0c9080f --numstat` | ✅ 5+10 / 2+2 |
| plan:61/:96 贴文 | python `line[:80]` | ✅ 逐字（含全角与 `**批10 收敛**`） |
| 2652 grep exit=1 | 同命令 | ✅ exit=1 |

## 面6 夹带/历史 — 🟢 绿

- **diff 只含声称 2 文件**：`git show 0c9080f --name-only` = `data/status/field-ownership.json` + `docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md`，恰 2 个。
- **历史恰 1 笔**：`git log --oneline b918c2e..HEAD` = `0c9080f` 恰 1 笔（4665186 撤销声称——无中间态残留）。
- **工作区干净**：`git status --porcelain` 无 tracked 修改；`git diff --cached --stat` 空（index 空）。
- **三个 untracked 复核报告未卷入**：`?? docs/findings/reviews/2026-11-07-p16-3-integrity.md`、`?? 2026-11-07-p2-plan-review-2.md`、`?? 2026-11-07-p2-plan-review.md`（另有 `android/build/` 及 10 个历史 untracked 报告，均不入 commit）。
- commit message 自述两次教训（首版 json 全量重写 935+/940- 假 diff 被 checkout 回滚；`cut -c1-100` 字节截断 mojibake + `--amend` 误扫 3 个 staged 报告后 `reset --soft` 拆出）与终态观测一致。

## 面7 闸面 — 🟢 绿

- `python3 gates/cli.py check` 独立重跑：**PASS=13 FAIL=0 SKIP=0、GATES_EXIT=0**。13 项全列：original-integrity / assets-manifest / reference-seed-integrity / knowledge-format / docs-links / naming-ledger / rust-tests / **test-ratchet total=978(基线978)** / table-content-hash / table-evidence / state-machine-anchors / asset-truth-manifest / **state-baseline 金样 74 场景 sha 全同**。
- `python3 tools/state_baseline.py --check` 独立重跑：**exit=0**，`[GREEN] 金样逐字节等价：74 场景 sha 全同`（stderr 仅 `preset_cases.rs` 3 条既有 lint warning：2732/2951 冗余分号、138 images_slot 未读——非本笔引入）。
- 声称 **13/13 + 金样 74 + ratchet 978** 三数全真。A 级（门禁工具实测输出）。

## 黄项清单

1. **🟡 黄-轻（待裁定，不阻断）**：plan `:63` 潜伏红披露仍写「`python3 tools/field_ownership.py --check` 现 **[RED]**（表缺 enemy_base…exit=1）」——本笔修后实际已 GREEN，该句成**过时陈述**（commit 只改 `:61/:96` 两行，未同步 `:63` 披露）。缓解因素：披露段是复核 R3 的历史记录且指向「批0 第一动作=归属表同步」，描述的是 commit 时刻前状态、非当前态主张。裁定倾向：计划内时态失配、非事实失实；建议下笔顺手加一行原地日期更正（AGENTS.md §3 证伪更正文化）标注「2026-11-07 批0① 后已复绿」，或维持历史记录原样。
2. （非本笔夹带，登记备查）`docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md:271` 仍引「187 字段唯一归属」——task-15 既落件抽验记录，日期锚定 2026-11-03、当时为真，属历史陈述；knowledge 无 187 残留（grep exit=1）。

## Confidence: 92/100
