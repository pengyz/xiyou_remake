# P1-3 硬门四件 + 随修三黄 独立对抗复核 — commit `6b676f5`

> 复核员：独立对抗复核（逐提交纪律·差分验证员纪律，AGENTS.md 权威）
> 对象：commit `6b676f5`（effect_registry.rs 单文件 +81/-6，未推送，绿=lead 推送）
> 日期：2026-11-05

## 0. 复核范围与方法

- 对象：commit `6b676f5`（`type(core): P1-3 硬门四件+随修三黄…`），`--name-only` = 仅 `crates/game-core/src/effect_registry.rs`（+81/-6，477 行终态）。未推送，绿=lead 推送。
- 方法（AGENTS.md §1/§3/§9 权威）：只读取证（git show 全 diff / a.java 行级对拍 / items.json 全表 / round-2 处方 / spec §0）→ **变异实验**（每次改后 `git checkout -- crates/game-core/src/effect_registry.rs` 还原，终态 `git status --porcelain -- crates/` 空）→ 三贴件独立实跑。
- 处方基线：`docs/findings/reviews/2026-11-05-p12-effect-round2.md:206-207`（四硬门 + 三黄 push 前顺手）、`:123`（entities 残留原指）、`:133`（round-2 对 kind_coverage 的"判别真"声明——本件证伪，见 §1）。
- 红/黄/绿判据：红 = 硬门主张不成立/可检出性缺失/保真缺口；黄 = 交付但有欠账或钉不真；绿 = 主张与实证一致。

## 1. 硬门① type→kind 映射 — 🔴 红（书面表 ✓ 全对；"测试钉"结构性空转=伪钉）

**1a. 模块头 32 型映射表逐型核：绿。**

- 表（**effect_registry.rs:26-38**，【硬门1】，8 数据行 :31-38）kind 分组 = Stack{13,14,16,17,18,19,20,21,85,86} ∪ TutorialSeenStack{15} ∪ Passive{22,23,24,25} ∪ Key{26,27,28} ∪ StatUp{29,30} ∪ HpUp{31,32} ∪ Equip武器{33,34,35,79,36} ∪ Equip防具{37,38,39,80,40} = **32 型，无重复无遗漏**。
- 与 `data/tables/items.json`（rows=32，id 集 {13..40,79,80,85,86}）**逐型完全一致** ✓。
- 与 a.java pickup 面（7537 switch）对拍：15 tutorialSeen 落穿→同栈臂、13/14/16..25/85/86 同一 `addItemToItemStack` 臂（7540-7554）、26/27/28 键、29/30 属性、31/32 HpUp、33..40/79/80 tryEquip（7642）→ **8 族分组与原版 switch 语义同构** ✓。Equip 两段 tier（33,34,35,79,36=1..5 / 37,38,39,80,40=7..11）与 items.json tier 字段逐行同 ✓。
- 欠账（黄，非本表）：**10 型（13,14,16..21,85,86）items.json 的 `effect.key` 是 use 键（OpenPanel/Remove…），与表内 `kind="Stack"`（pickup 面）不同**——同名字段双语义，模块头未注此分歧，P1-3 接线者易误读。

**1b. `type_to_kind_mapping_no_miss` 真伪：🔴 伪（结构性空转，永绿）。**

- 测试读码（:404-422）：32 行硬编码 `(type, kind)` 配对 → `builtin_effect(kind)` → `assert!(!std::ptr::eq(builtin_effect(kind), &NoOpEffect), "型 {t} kind {kind} miss")`。
- 变异实证（`git checkout` 还原法）：
  - **M4** 把配对改错 `(13,"Stack")`→`(13,"OpenPanel")` → **11/11 仍全绿**（配对本身未被钉，断言只问 kind 是否可达）。
  - **M5** 删归一臂 `"TutorialSeenStack" | "Passive" => &StackEffect,`（:272，15/22..25 四型真实 miss）→ **11/11 仍全绿**（本测试与 `kind_coverage_full_table` 双双不报）。
- 根因（探针 ptr_probe/ptr_probe2 实测，已还原）：`std::ptr::eq(builtin_effect(kind), &NoOpEffect)` **恒 false**——P1 同一测试内两处 `&NoOpEffect` 强转→true；P2 `builtin_effect("NoSuchKind")`（必然命中 `_ => &NoOpEffect`）vs 测试处 `&NoOpEffect`→**false**；P3 StackEffect vs NoOp→false；P4 三者 data addr 全=0x1（ZST 悬垂地址）。即 **lib 函数返回处与 test 模块强转处生成的是两个不同 trait-object vtable 实例（跨 codegen unit），`ptr::eq` 连同 vtable 一起比 → 断言 `!eq` 恒成立、永不 panic**。
- 结论：硬门①的"测试钉（32 型逐型可达）"**不可判别**——P1-3 前置保护失效：任何人删/改注册臂，门仍绿。
- 连带证伪（须按 AGENTS §3.6 原地更正）：round-2 处方文 `2026-11-05-p12-effect-round2.md:133`"kind_coverage 删任一注册臂即红（判别真）"**为读码推断，未实跑，现已证伪**（M5 同一实验，`kind_coverage_full_table` 亦绿）。
- 修复方向（供 lead，不在本件范围）：断言改为对 `builtin_effect(kind)` 做**可判别比较**（如 `std::any::type_name` 不可用时，用行为指纹：对每 kind 施加同一输入断言输出差异，或把 NoOp 换成 panic 版哨兵 Effect 注册到 `_` 臂的测试替身），或让 `_ => &NoOpEffect` 走一个带 `unreachable!` 的 test-only 分支。

## 2. 硬门② Passive 归一 — 🟢 绿（归一臂与原 match 同构；两处措辞过强记黄）

**2a. 归一臂注册与对拍：绿。**

- 新臂 `"TutorialSeenStack" | "Passive" => &StackEffect`（effect_registry.rs:272），与既有 Stack 臂同实现 `&StackEffect`。
- a.java pickup 面活查（sed 实文）：`case 15:`（a.java:7538）→ `this.tutorialSeen = true;`（7539，**无 if 直写**）→ 落穿 `case 13/14/16..21/22/23/24/25/85/86`（7540-7553）共用 **7554 `this.addItemToItemStack(var1); return;`**（7555）→ **"15=tutorial 变体、22..25=被动族"在 pickup 面与原 match 语义等价** ✓（15 的差别只在落穿前的 `tutorialSeen` 全局标志，栈增行为与同族完全一致；Rust 侧 tutorialSeen 状态由别处承接，本臂只管 kind 可达）。
- **Stack{tutorial} params 编码等价：绿**——15 的 items.json `effect.key=TutorialSeenStack`、params 空；原版 tutorialSeen 是全局 bool（非 per-stack params），归一到 `&StackEffect` 不引入任何 per-item 状态差异，语义面等价。
- `find_item` 活查（addItemToItemStack=7727-7740 → findItemStackIndex 7664-7675）：23（a.java:10001）、24（10003）、25（7950/10052）、16（8198）均有活查 ✓；ITEM_USES（a.java:944）与 registry ITEM_USES（effect_registry.rs:122）**逐字同** ✓。

**2b. 措辞过强（🟡 ×2，不改行为）：**

- 🟡 **"22..25=被动族 find_item 活查"对 22 过强**：全仓 `findItemStackIndex(22)` **0 命中**（活查只在 23/24/25/16）。且 22=下行符，`activateItem`（a.java:7793 switch）有**独立 `case 22`（7868）** = use 面可用（desc"使用次数：1次"同证）；其"被动"语义仅来自 a.java:918 描述文本"对某些自恋的神仙伤害加倍。[放在道具栏中有效]"（= items.json `Passive{AntiNarcissusDamage}` 的 evidence，描述面 B/C 级、无行为锚）。模块头把 22 与 23/24/25 并称"被动族 find_item 活查"缺 22 的活查证据。**pickup 面不受影响**（22 在 7548 与同族共臂，P1-3 分发无差异）。
- 🟡 **kind≠effect.key 的 10 型分歧未注**：13/14/16..21/85/86 的 items.json `effect.key` 为 use 键（OpenPanel/RemoveAdjacent/RemoveAllType/Heal/MirrorTeleport/ChangeFloorUp/KillAdjacent4），表内 kind 却是 `Stack`（pickup 面）。两者分属 use/pickup 两面本可并存，但模块头【硬门1】表未声明"此表只管 pickup 面"，P1-3 接线者可能拿 items.json 的 key 去对表。

## 3. 硬门③ 消耗语义 — 🟢 绿（声明交付如处方；🟡 附注：NoOp 无测试钉）

- **交付**：`UseArmEffect.apply` 返回 `ItemOutcome::NoOp`（effect_registry.rs:248），doc 注 :241-244 三连——"【硬门3 仅键可达声明】本行=分发键可达性钉（kind 覆盖面）；本层零行为（apply 返回 NoOp=不消耗不副作用）——行为面=item_effect.rs **execute_item_action 编排（P1-3 分发未接线）**；前值 Consumed=潜伏面已消：若误接线会假报消耗"。**execute_item_action 编排未接线注记在 ✓**（:243，另 commit message 同句）。round-2 处方 `:207` 允许"对齐**或**显式'仅键可达'声明"→ 取后者，逐字交付 → 门③成立。
- **"前值 Consumed=假报消耗隐患已消"的证伪链**：
  - 谁前值：`UseArmEffect.apply` 原返回 `ItemOutcome::Consumed { overlay_tier: None }`——`git show 6b676f5` diff 的 `-` 行即前值（同文件同位置），非推测。
  - 现值：`ItemOutcome::NoOp`（diff `+` 行 + :248 实文）→ 本层不产生消耗副作用，"误接线即假报消耗"的潜伏面在**当前树已消** ✓。
  - **revert 证据（🟡 附注）**：变异 M2 把 :248 改回 `Consumed { overlay_tier: None }` → **11/11 仍全绿**——`UseArmEffect` 全仓无任何测试引用，NoOp 被改回 Consumed 不可检出。即"已消"是**代码事实**而非**被钉事实**，防回归缺口留给 P1-3 接线者（接线时必须补行为钉，否则该隐患以回归形式复发）。

## 4. 硬门④ entities 归域 — 🟢 绿（裁定与 spec §0 一致）；🟡 残留：round-2 原指的 `ItemCtx.entities` 死字段未清

- **注记交付**：effect_registry.rs:40-43【硬门4】"ItemDef.icon.entity_type=entity 图面引用（images.entity[t] 渲染面/sprite_defs 引用面）——归 entity/sprite 域外键**非 item 自有字段**（items.json icon 即该外键句柄；P1-6 EnemyDef 聚合同则）"。
- **与 spec §0 总则一致 ✓**（docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md:10-19）："per-type 属性束聚合为记录；共享矩阵保留表形；判据=数据答'某实体是什么'→聚合、'某类资源怎么摆'→留表"。icon.entity_type 描述的是"这张图面属于哪个 entity"（跨表引用），不属道具自有属性束 → 声明为 FK、句柄留 ItemDef、引用目标留 entity/sprite 域，正是 §0 的"引用留域"而非"复制进记录"。**归域裁定正确**。
- **既有强制背书**：`crates/game-data/src/defs.rs:389 pub entity_type: i32`、`:445/:489` `Reject::BadForeignKey { … icon.entity_type 域外 … }`（域外拒载，P1-1 已存在的强制，非本件新增，也未被本件弱化 ✓）。
- 🟡 **残留（round-2 处方原指未清）**：`2026-11-05-p12-effect-round2.md:123` 点名的 entities 残留是 **`ItemCtx.entities` 死字段**（零读取，清法="P1-3 接线或字段冻结 schema 流程移除，随修"）。本件只交付了 icon FK 注记，`ItemCtx.entities` 仍在（effect_registry.rs:72 字段声明、:309 仅测试构造，**文件内零 `ctx.entities` 读取**；其他文件的 `.entities` 均属 Scene 非 ItemCtx）。commit 把"entities 字段处置"重释为 icon FK，二者不是同一件事：FK 注记 ✓ 交付，死字段处置 ✗ 未动。不阻塞（无行为面），但 P1-3 接线时若读它会立即从"死"变"活"，需在 P1-3 任务里显式决定（接线 or 走 schema 流程删）。

## 5. 三黄核销 — ①🟢 ②🟢 ③🟡

**黄① m_083 锚改写：🟢 核销。** 两处均已改 `a.java:7699-7709`（effect_registry.rs:200、:229）。sed 实文核：`equipTierIndex` decl=a.java:7699、`while (--var3 >= var2)`=7702、`return` 终=7708 → 新锚**逐字正确**，旧锚 `7688-7694`（实为前一方法区）在本文件 0 残留。记账不阻塞：`crates/game-core/src/scene.rs:886` 仍留旧锚 `equipTierIndex（a.java:7688-7694）`——**既有欠账**（非本 commit 引入，本件未动 scene.rs，与"仅 effect_registry.rs"的夹带净一致），建议随 P1-3 顺手改齐。

**黄② HpUp 判别用例：🟢 核销（判别真，实跑必红）。** `hpup_no_9999_clamp_discriminator`（:440-448）：`c.floor=3`、`HpUpEffect.apply(delta=20000)` → `assert_eq!(c.player.hp, 20000, "无 9999 钳（判别：臆造语义下=9999 应红）")`。
- "20000 不钳" ✓ 实跑绿（11/11 内）。
- "revert 9999 必红" ✓ **变异 M1**：在 apply 内插入 `if ctx.player.hp > 9999 { ctx.player.hp = 9999; }` → 本用例 **FAILED（10 passed; 1 failed）**，随后 `git checkout` 还原。判别真。

**黄③ def 臂 tier≥6 断言：🟡 半核销（加半钉真、减半空转——与 round-2 预判同）。** `equip_armor_tier_ge6_def_bonus`（:453-463）按处方字面交付：tier=7 → `Consumed{Some(7)}`、`equipped_armor==37`、`assert_eq!(c.player.def, def0 + 10, "def 双动=+bonuses[7](10)−bonuses[6](0)")`。
- **加半钉真**：**变异 M6** 删 `ctx.player.def += ctx.equip_bonuses[tier as usize];`（:222）→ 本用例 **FAILED（10 passed; 1 failed）** ✓（即 round-2 红-1"丢加成"若复发必红）。
- **减半空转**：**变异 M3** 删 `ctx.player.def -= ctx.equip_bonuses[cur as usize];`（:221）→ **11/11 仍全绿**——因 `equip_bonuses[6]=0`（effect_registry.rs:315 常量），tier=7 时 `−bonuses[6]` 恒为 0，断言里那一项不可判别。断言文案写了"双动"，实际只钉到"加"。要钉减半需用 `bonuses[6]≠0` 的档位（或直接对 m_083_index 返回值断言）。

## 6. 验证实跑 — 全绿（三贴件独立复现逐字同）

| 项 | 命令 | 结果 | 与 message 贴件 |
|---|---|---|---|
| 等价测 | `cargo test -p game-core --lib p1a2` | `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 322 filtered out; finished in 0.00s` | **逐字同 ✓** |
| core 全量 | `cargo test -p game-core --lib` | `331 passed; 0 failed; 2 ignored`（12.40s） | message 未贴（任务要求实跑，已跑） |
| 金样 | `python3 tools/state_baseline.py --check` | `[GREEN] 金样逐字节等价：74 场景 sha 全同`（job bash-de02a6b2e2874155, exit 0） | **逐字同 ✓** |
| 状态预检 | `python3 gates/cli.py status` | `PASS=13 FAIL=0 SKIP=1 TOLERATED=0`（SKIP=visual-diff，P4 已知恒 SKIP，显式打印） | 合规 |
| 夹带净 | `git show --name-only 6b676f5` | 仅 `crates/game-core/src/effect_registry.rs`；变异后 `git checkout` 还原，终态 `git status --porcelain -- crates/` 空；untracked 仅 `android/build/` + 既有复核 md×5 + 本件 | **净 ✓** |

**变异实验账（每次后 `git checkout -- crates/game-core/src/effect_registry.rs` 还原，6/6 均已还原，工作树终态 clean）：**

| # | 变异 | 结果 | 判定 |
|---|---|---|---|
| M1 | HpUp 插 9999 钳 | `hpup_no_9999_clamp_discriminator FAILED` | 黄②判别**真** ✓ |
| M2 | UseArmEffect 改回 `Consumed{None}` | 11/11 绿 | NoOp **无钉** 🟡 |
| M3 | 删 `def -= bonuses[cur]` | 11/11 绿 | 黄③**减半空转** 🟡 |
| M4 | 配对 `(13,"Stack")`→`(13,"OpenPanel")` | 11/11 绿 | 硬门①**配对未钉** 🔴 |
| M5 | 删归一臂 `"TutorialSeenStack" \| "Passive"` | 11/11 绿 | 硬门①**miss 不可检** 🔴 |
| M6 | 删 `def += bonuses[tier]` | `equip_armor_tier_ge6_def_bonus FAILED` | 黄③**加半真** ✓ |
| 探针 | `ptr::eq(builtin_effect("NoSuchKind"), &NoOpEffect)` | **false**（data addr 同=0x1） | 断言恒成立=**永绿根因** 🔴 |

## 7. 三纪律贴件核对 — 贴件实真 ✓；"测试钉"主张 ✗

**① 逐提交纪律：绿。** 单 commit 单文件（`git show --name-only` 仅 effect_registry.rs），type=`feat` scope=`core`，subject 与正文同粒度（四硬门+三黄 = round-2 `:206-207` 一份处方的核销单元，与 round-2 自身提交粒度同例）。未混提门禁/台账/参考版；`gates/cli.py status` PASS=13 FAIL=0（无被弱化门禁）。

**② 差分验证员纪律：贴件实真 ✓，钉可判别性 ✗。**
- 贴件逐字复现 ✓：`test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 322 filtered out; finished in 0.00s` 与独立实跑**逐字同**；`[GREEN] 金样逐字节等价：74 场景 sha 全同` 与 `state_baseline.py --check` **逐字同**（exit 0）。core 331/0 未贴但实跑绿。
- 未放宽阈值 ✓：金样比对走原 `tools/state_baseline.py --check`，74 场景 sha 全同，无白名单/容差改动（本 commit 未触 `gates/`、`data/status/baselines/`）。
- **钉可判别性 ✗（本件唯一红）**：commit message 声称 `type_to_kind_mapping_no_miss 测试钉（32 型逐型可达）`，但 M4/M5 双证 + ptr 探针证其**结构性空转**——`!std::ptr::eq(builtin_effect(kind), &NoOpEffect)` 因跨 CGU vtable 实例不同而恒真，删注册臂/改错配对均 11/11 绿。同病波及 `kind_coverage_full_table`（:433 同式）。按"钉=红→绿可判别"，此主张不成立。

**③ AGENTS.md 权威：绿。** 反引号 **0** 个（message 内无 shell 风险字符）；`Evidence:` 行缺省**合规**（仅触 `crates/game-core`，§9 强制面=data/naming/·reference/·original/ 未触及）；`Confidence: 93` 在位；`reference/` 回归 PASS（gates status：`reference-regression refs=707 shim=69 missing=0`）、`diff-trace` PASS、`table-content-hash` PASS——本件未触参考版/资源表/门禁，符合 §4 三问（层=Rust 移植侧登记面；消费者=测试与后续 P1-3 分发；证据=A 级 a.java 行号 + B 级 items.json）。

## 8. 总判 — 🔴 红1 · 🟡 黄5 · 🟢 绿6 → **需修复（改钉后可推送）**

| 攻击点 | 判 | 一句话 |
|---|---|---|
| ① type→kind | 🔴 | 书面表 32/32 与 items.json·a.java 逐型全对 ✓；但 `type_to_kind_mapping_no_miss` **伪钉**（M4/M5 双证 + ptr::eq 跨 CGU 恒 false → 永绿），连带证伪 round-2 `:133`"删任一注册臂即红" |
| ② Passive 归一 | 🟢 | 归一臂与 a.java pickup 同构（15 落穿同臂、22..25 同臂、ITEM_USES 逐字同）；🟡 22"find_item 活查"过强、🟡 kind≠effect.key 10 型分歧未注 |
| ③ 消耗语义 | 🟢 | "仅键可达/本层零行为/execute_item_action 未接线"注记在 :241-244；前值 `Consumed{None}` 由 diff 证、现值 NoOp 潜伏面已消；🟡 M2 证 NoOp 无钉（回退不可检出） |
| ④ entities 归域 | 🟢 | icon.entity_type=FK 与 spec §0 聚合判据一致，defs.rs:445/:489 域外拒载既有强制未弱化；🟡 round-2 `:123` 原指 `ItemCtx.entities` 死字段（:72 声明/:309 测试构造，零读取）未处置 |
| 黄① 锚 | 🟢 | 两处 :200/:229 = `a.java:7699-7709` sed 实文正确；scene.rs:886 旧锚=既有欠账记账 |
| 黄② HpUp | 🟢 | M1 加 9999 钳 → 用例必红，判别真 |
| 黄③ def 断言 | 🟡 | 字面交付；M6 删 `+=` 必红（加半真），M3 删 `-=` 仍绿（`bonuses[6]=0` 减半空转） |
| 验证/夹带 | 🟢 | 11/11、331/0、金样 74 GREEN、gates 13/0/1SKIP 全独立实跑；夹带净=仅 effect_registry.rs |

**阻塞项（唯一）**：硬门①的"测试钉"不可判别——两个 no-miss 测试永绿，等于给 P1-3（23 臂一次性切换）前置的**注册面保护缺位**；一旦接线时注册臂被误删/误改，全门静默通过，只能靠金样场景偶然兜底。
**修复（小，限 effect_registry.rs）**：把两处 `!std::ptr::eq(builtin_effect(kind), &NoOpEffect)` 换成可判别断言——推荐 `_` 臂在 `#[cfg(test)]` 下返回 `panic!("kind miss: …")` 的哨兵 Effect，或对每个 kind 施加同一输入断言输出指纹差异；同时给 `rows` 配对与模块头表做**同源断言**（M4 证配对未钉）。修后须重跑 M4/M5 两变异确认变红，再 11/11 + 331/0 + 金样 74 复验。
**非阻塞黄（随 P1-3 一并处理）**：黄②两处措辞（22 活查、kind≠key 分歧注）、③ NoOp 补钉、④ `ItemCtx.entities` 决定接线 or schema 删除、黄③ 减半钉（换 `bonuses[6]≠0` 档位或直钉 m_083_index）、scene.rs:886 旧锚。

**结论：需修复**（红1 修复且变异回归转红后可推送；其余黄不阻塞推送）。
