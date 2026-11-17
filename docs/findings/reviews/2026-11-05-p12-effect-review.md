# P1-2 复核：trait ItemEffect + 内建注册六型（commit c819c73）

> 独立对抗复核 · 逐提交纪律 · 差分验证员纪律 · AGENTS.md 权威
> 复核对象：commit `c819c73`（未推送，绿=lead 推送）
> 前情：P1-1（ItemDef+items.json 32 行）已推
> 复核时间：2026-11-05

## 0. 结论速览

复核对象：`c819c73`（未推送）。**红 1 / 黄 1 / 绿 5 —— 总判：需修复**（Equip 加成+锚修正后可推）。

| # | 攻击点 | 判 | 一句话 |
|---|---|---|---|
| 1 | 逐字抽象真伪（核心） | 🔴 **红** | Equip 臂漏换装加成（equip_bonuses 死字段、测试未钉）；锚 7528 陈旧、7631 截掉装备臂（spec §2.2-① 明禁同类错）；HpUp 引入源中不存在的 9999 钳且注释"编排面同值"失实 |
| 2 | 等价测 7/7 | 🟢 绿 | 7 测试名与 message 逐条对上，实跑 7 passed/0 failed/322 filtered，与贴件逐字同 |
| 3 | 未知两层语义 | 🟢 绿 | 加载期拒载（P1-1 测试在册实跑绿）+ 运行期 NoOp 不消耗（实跑绿）；锚 a.java:7630-7631 `default: return` sed 实文命中 |
| 4 | Ctx 三宪 | 🟡 **黄** | 签名 ctx+params≤2 ✓、字段冻结 ✓；但 ItemCtx 带 `floor_tier()` 方法与三宪-1"纯数据（无方法/无行为）"字面冲突（纯读无害，宜改自由函数），另 entities/equip_bonuses 两死字段 |
| 5 | 红线：switch 零动+两面并存 | 🟢 绿 | `git diff e87ecc1 c819c73 -- scene.rs` 空；两面并存注记在模块头；builtin_effect 无生产调用=分发确未切（P1-3 面） |
| 6 | 验证账 | 🟢 绿 | core lib 327/0（2 ignored 为既有）+ 夹具 arm_coverage 21 全账绿 + 金样 74 --check GREEN 实跑 + 夹带净（仅 2 文件） |
| 7 | 三纪律（含 shell 工艺修正） | 🟢 绿 | 反引号吞段→Python+-F 判定=**合规改进非掩盖**；Evidence 两行我实跑逐字复核为真；无门禁绕过 |

## 1. 逐字抽象真伪（核心）

### 1.1 模块头逐臂对照表（7 臂→6 注册行）逐行核

逐行 `sed -n 'X,Yp' reference/src/deobf/a.java` 实文核（引锚纪律）：

| 表行 | 表锚 | sed 实文命中 | 判 |
|---|---|---|---|
| 13..=25\|85\|86 →Stack | 7528-7555 | 7528 起在 **gainGold 尾部**（`this.goldAmount += var1;`），`private void pickupItemType` 实为 **7534**；case 15@7538、addItemToItemStack@7554、return@7555 均在范围内 | 🟡 **起点错 6 行**（内容段对，起点锚陈旧） |
| 26/27/28 →Key | 7556-7564 | case 26@7556、27@7559、28@7562、return@7564 —— 逐行命中 | 🟢 |
| 29/30 →StatUp | 7565-7580 | case 29@7565（`currentFloor<=10 →++` 否则 `+floorTier()`）、30@7573 —— 逐行命中 | 🟢 |
| 31/32 →HpUp | 7581-7587 | case 31@7581 `applyHpDelta(50*var2)`、32@7585 `applyHpDelta(200*var2)` —— 命中（32 无 return 落穿至 default，表未注，items.json 证据已注"落穿无害"） | 🟢 |
| 33..=36\|79 / 37..=40\|80 →Equip | 7632-7648 | case 33@7632…80@7641、tryEquip@7642、else showOverlayMessage —— 两行共锚命中 | 🟢 |
| default →NoOp | 7630-7631 | `default:`@7630、`return;`@7631 —— 逐字命中（spec §2.2 轮2已纠过的正确锚，本件沿用对） | 🟢 |

- **7 臂→6 注册行**：表 7 行（Equip 占两行）→ builtin_effect 六键（Stack/Key/StatUp/HpUp/Equip/_→NoOp）——账平 ✓。
- 🟡 **行 1 起点锚 7528 错**：`pickupItemType` 现行 **7534**（d389ddf 已 +6 漂移且为 c819c73 祖先，`git merge-base --is-ancestor` 实证 YES）；7528 落在 gainGold 内。表内其余 5 锚全部现值精确。同源陈旧：scene.rs:4243 注释亦写"a.java:7528 起"（既有欠账，spec §2.2 曾记"锚漂移前科"）。
- 🟡 **commit message 总锚 "a.java:7528-7631" 双错**：起点同上错 6 行；**终点 7631 截掉装备臂（7632-7648）** —— 这正是 spec §2.2 轮2修复①点名禁止的同类截断错（原文："7633 切在装备臂 case 标签列中段……'逐字抽象'的锚必须含装备臂"）。plan P1-2 行自写锚为 **7534-7649**，与 message 不一致。

### 1.2 67 行 match 原文 vs 注册行语义等价

- "67 行"核：scene.rs pickup_item_type fn 体 4242-4308 恰 67 行 ✓（4308-4242+1=67）；spec §2.2 轮2修复②原判"恰 67 行"沿用成立。
- **抽 3 臂深核**：

**① Stack 增栈** 🟢
- Java：`addItemToItemStack`（7727-7740）findItemStackIndex 命中→uses 叠加，未命中→栈顶追加 `itemUseCounts[var3]`；`itemTypeToStackIndex`（7711-7725）85→20、86→21、否则 `-13`；`itemUseCounts`@944 `[-1,-1,-1,-1,1,1,1,3,1,1,-1,-1,-1,1×11]`。
- scene.rs add_item_to_stack：idx 映射、ITEM_USES 常量表、find_item 分支——与 Java 逐项同 ✓。
- StackEffect：tutorial 置位（15 臂）+ 同构 push/merge——**状态面逐字等价** ✓。差：不写 `itemUseTargetType`、不发 showOverlayMessage（ItemOutcome 未带此副作用字段）——编排面归因待 P1-3 接线时对齐，本件分发未切故不致漂移，**黄记不红**。

**② Key 三色** 🟢
- Java case 26/27/28：yellow/red/blue KeyCount++ 各带 return（7556-7564）。
- KeyEffect：三 bool 分别 ++，params `Key{yellow,red,blue}` 与 items.json `Key{color}` 载荷镜像可推导（color→bool 三选一）。测试 yellow+blue→(1,0,1) 实跑绿。语义等价 ✓。

**③ StatUp floorTier 双例** 🟢
- Java case 29/30：`floor≤10 → ++`，否则 `+= floorTier()`（7565-7580）。
- StatUpEffect：`floor≤10 → (atk,def)` 本身，否则 `×floor_tier()`——**注意乘法形**：params 给的是 +1 基数（atk:1），`1×tier==+tier`，与 Java `+floorTier()` 等价；双例测试 floor3→+1、floor12→+2 实跑绿。
- floor_tier 实现与 a.java:7651-7662 逐行同（(f-1)/10 钳 0、f>50→0、+1）✓。
- 🟡 注：若未来 params 给 atk:2，`2×tier ≠ +tier`——泛化形与 Java 写死 +1 不同域，当前 items.json 只有 stat:atk/def 单位基数，**现实数据域内等价**，泛化越界属 P1-3+ 风险，记注不判错。

**附抽验 Equip 臂（任务未点名但属核心声明面）** 🔴——见 1.3。

### 1.3 🔴 核心红：Equip 臂漏换装加成（"逐字抽象"声明不实）

- **Java tryEquip（7677-7695）三步**：①档判 `equipTierIndex(cur)>=tier→fail`；②**`playerAtk -= bonuses[cur]; playerAtk += bonuses[tier]`（7684-7685）**；③`equippedWeaponType = types[tier]`。防具分支同构（7691-7693 改 def）。
- **scene.rs try_equip（865-884）**：同三步，`atk/def -= equip_bonuses[cur] += equip_bonuses[tier]` 实写在案。
- **EquipEffect（effect_registry.rs）**：只做①档判 +③换槽，**②加成完全缺失**——成功换装后 atk/def 原地不动。
- 佐证三连：
  1. `equip_bonuses: [i32;12]` 在 ItemCtx 声明（:59）+ 测试构造（:275）**但全文件零读取**——死字段，说明加成本被计划入 ctx 却未接线；
  2. 测试 `equip_arm_matches_match_semantics` 只断言 `equipped_weapon==33` 与 outcome，**不断言 atk+10**——等价测对该缺口免疫（红→绿实录只修了空档特判，未覆盖加成）；
  3. plan/spec 对 Equip 臂的"逐字"要求含 tryEquip 全语义（spec §2.2 行为注册=67 行 match 逐字抽象，try_equip 是 match 臂内调用链一环）。
- **影响面**：P1-2 分发未切（红线守住），当前无运行时漂移、金样绿不受扰；但注册表侧行为面自称"逐字抽象+等价实证"，Equip 臂不实——**P1-3 切换即暴露**（换装不加攻防=保真破坏）。修复：EquipEffect 成功分支补 `ctx.player.{atk,def} -=/+= equip_bonuses[cur/want]`，测试补 atk/def 断言（红→绿）。

### 1.4 🟡 HpUp 臂：9999 钳为无源常量 + 注释失实

- `let max_hp = 9999; // apply_hp_delta 钳上界=编排面同值（hp clamp 逐字待 P1-3 对齐）`（effect_registry.rs:169-170）。
- 实查：**全仓 grep 9999**——a.java 无、scene.rs apply_hp_delta 无任何上钳（`hp += delta` 直加，仅 lethal 分支 `hp<=-delta→1`）、编排面无 9999。注释"编排面同值"**失实**（无同值可对）。
- 语义差：Java `applyHpDelta(50×200×tier)` 正 delta 无上界；注册版 clamp(0,9999) 在 hp+d>9999 时截断。当前数据域（初始 498/300/1000、+2000 事件@7430）难触 9999，**现实不可达**，但"逐字"声明下引入无源常量=猜测性实现，违 §4 三问（证据答不出 A/B 级）。
- 修：删钳或补 A 级锚（若编排面将来真加钳，同 commit 双面同改）；注释"编排面同值"须撤。

### 1.5 🟡 kind 词表错位隐患（P1-3 接线风险，本件不判错）

- items.json effect.key 实测词表：`TutorialSeenStack/Key/StatUp/HpUp/Equip/OpenPanel/Passive/Heal/…`（13 键）。
- builtin_effect 匹配词表：`"Stack"/"Key"/"StatUp"/"HpUp"/"Equip"`——**无 "TutorialSeenStack"**（id15 走 `_→NoOp`），而模块表行 1 却把 13..=25|85|86 整臂映到 `Stack`。
- 即：**按 effect.key 调 builtin_effect 时 id15 及全部非键值堆叠臂（13,14,16..25,85,86 的 effect.key 是 OpenPanel/Passive/RemoveAllType 等）都会 miss→NoOp**。P1-2 分发未切故不爆；P1-3 若按"effect.key→builtin_effect"接线将塌方。表行语义（按 type 臂）与注册键（按词表）之间缺一层 type→kind 归一映射的书面约定——**黄记，P1-3 前必须补钉**。

## 2. 等价测 7/7

测试名清单（effect_registry.rs `mod p1a2_tests`）+ 实跑 `cargo test -p game-core p1a2_tests`：

| # | 测试名 | 对应臂 | 实跑 |
|---|---|---|---|
| 1 | `stack_arm_matches_match_semantics` | Stack tutorial+栈增（type 15, tutorial=true→ts 置位+stack_size=1） | ✅ ok |
| 2 | `key_arm_matches_match_semantics` | Key 三色（yellow+blue→(1,0,1)） | ✅ ok |
| 3 | `statup_arm_matches_match_semantics` | StatUp floor≤10 +1（floor3→(1,1)） | ✅ ok |
| 4 | `statup_floor_tier_arm` | StatUp floor12 +tier（tier2→atk 2） | ✅ ok |
| 5 | `hpup_arm_matches_match_semantics` | HpUp mul×tier（delta200, tier1→hp 200） | ✅ ok |
| 6 | `equip_arm_matches_match_semantics` | Equip 成功（tier1→type33）+失败双态（再贴→NotConsumed） | ✅ ok |
| 7 | `unknown_effect_is_noop` | 未知 kind→NoOp | ✅ ok |

- 实跑：`7 passed; 0 failed; 0 ignored; 0 measured; 322 filtered out` —— 与 commit message 贴件**逐字同** ✓。
- message 声明的覆盖点（Stack tutorial+栈增 / Key 三色 / StatUp +1 与 floor12 +tier 双例 / HpUp mul×tier / Equip 成功失败双态 / 未知 NoOp）与 7 测试逐项对上，无虚报 ✓。
- 🟡 但"等价"强度注意：7 测均为**注册面自证**（对拍预期值），非对 scene.rs match 的差分对拍；Equip 测不断言 atk/def（见 1.3 红），StatUp 泛化形未覆盖（见 1.2③注）。**测试名清单与实跑=绿；覆盖深度缺口已归 1.3/1.2 记账**。

## 3. 未知两层语义

**第一层·加载期拒载** 🟢
- `crates/game-data/tests/items_defs.rs::items_reject_unknown_effect_key`：`"OpenPanel"→"OpenPane1"` 替换后 `parse_items` 返回 `Reject::UnknownField`——实跑 `cargo test -p game-data --test items_defs` **7 passed**（含本测）✓。
- 另 `items_reject_unknown_field`（拼写防呆）同绿。加载期拒载=P1-1 既钉面，本件引用属实。

**第二层·运行期 NoOp 不消耗** 🟢
- `unknown_effect_is_noop`：`builtin_effect("NoSuchKind")` → `ItemOutcome::NoOp`（非 Consumed=不消耗）——实跑绿 ✓。
- 锚 a.java:7630-7631 sed 实文：`default:` / `return;` —— **逐字命中** ✓（spec §2.2 轮2已把旧错锚 7608-7620 纠到 7630-7631，本件模块头沿用正确锚）。
- 模块头声明"加载期拒载（defs.rs 未知键=P1-1 已钉）+ 运行期 no-op 不消耗"两层措辞与实测一致，无混谈（未把 R-TODO-1b 卡死语义扯入）✓。

## 4. Ctx 三宪形

spec §3.2 三宪逐条：

| 宪 | 要求 | 实测 | 判 |
|---|---|---|---|
| 1 纯数据 | 无方法/无行为 | ItemCtx 8 字段全 pub 纯槽；**但带 `impl ItemCtx { fn floor_tier() }` 方法** | 🟡 |
| 2 字段冻结 | 改 Ctx=显式 schema 流程 | 结构固定 8 字段，本件新建无中途增删 | 🟢 |
| 3 ctx+≤2 | 每 trait 方法=ctx+至多 2 语义参 | `fn apply(&self, ctx: &mut ItemCtx, params: &EffectParams) -> ItemOutcome` = ctx+**1** 参 | 🟢 |

- 🟡 **第 1 宪字面张力**：`floor_tier()` 是 ItemCtx 固有方法。辩护面：纯读（&self、无副作用、无字段写），非"行为"，属派生查询；严判面：宪文写死"无方法"。参照 spec 宪法本意=防上帝袋（HostCtx 21 方法前车），floor_tier 单一纯读不构成袋化。**判黄不判红**，建议改自由函数 `fn floor_tier(floor: i32) -> i32` 消除字面争议（一行迁移）。
- 🟢 签名核（任务点名项）：ctx+params≤2 —— **过**，且无第三参、无返回 ctx。
- 🟡 附带：`entities`、`equip_bonuses` 两字段在六型效果中零读取（equip_bonuses 见 1.3 红因），"字段冻结"面冻结了一个未接线的面——归 1.3 一并修。

## 5. 红线：23 臂 switch 零动 + 两面并存注记

- **git diff 空核**：`git diff e87ecc1 c819c73 -- crates/game-core/src/scene.rs` → **0 行**；commit `--name-only` 仅 `effect_registry.rs` + `lib.rs`（+371/-0）✓。23 臂 switch（pickup_item_type 4242-4308）**零动** ✓——口径实测：含 `=>` 行 **23 行**（`=>` 出现 31 次，含 4 个内层 tier match），与 P1-1 复核沿用的"23 臂"行数口径吻合。
- **两面并存注记**：模块头 L15-17 明写"pickup_item_type 23 臂 switch 本件不动（P1-3 一次性切）——注册表侧行为面建好+等价实证，分发未切" ✓；commit message 同声明 ✓。
- **分发未切=spec P1-3 面**：plan P1-3 行"pickup/use dispatch 化（match 删除与表行同 commit 一次性切——禁双轨）"——本件未切符合分期 ✓。`builtin_effect` 生产调用方 grep=0（仅 lib.rs 挂模块），死面待 P1-3 接线，无双轨并行风险 ✓。
- 🟢 红线全绿。附注：scene.rs:4243 注释锚"a.java:7528 起"为既有陈旧（非本件引入，本件未动 scene.rs 故不担责），P1-3 切换时随注一并更正（spec §2.2 已有此约）。

## 6. 验证账

| 项 | 声明 | 实跑 | 判 |
|---|---|---|---|
| core 327/0 | message 贴件 `7 passed … 322 filtered`（p1a2 面） | lib 全量 `327 passed; 0 failed; 2 ignored`（2 ignored=bug044 debug_assert 门控+probe_bottom 实验探针，均既有非本件）；p1a2 过滤面 7/0 与贴件逐字同 | 🟢 |
| 夹具 21 全账 | arm_coverage 全绿在册 | `cargo test -p game-oracle --test arm_coverage` → `p1a2_all_21_arm_groups_have_fixture_triplets … ok`（1 passed） | 🟢 |
| 金样 74 --check | `[GREEN] 金样逐字节等价：74 场景 sha 全同` | `python3 tools/state_baseline.py --check` → **同句 GREEN 实跑复现**；`gates/cli.py check` 内嵌 state-baseline 亦 GREEN | 🟢 |
| 夹带净 | — | `git status` 未跟踪仅既有 android/build+4 复核 md（含本件）；commit 内 2 文件与 message 申报面一致，无夹带 | 🟢 |
| 总闸 | — | `gates/cli.py check` → **PASS=13 FAIL=0 SKIP=0**（含 rust-tests/test-ratchet 959≥929/diff-trace/state-machine-anchors 全绿） | 🟢 |

- test-ratchet 提示"新增二进制待 ratchet-update: {arm_coverage, items_defs}"=P1-1 批既有待办（PASS 不阻断），非本件引入。

## 7. 三纪律裁定（含 shell 反引号吞段工艺修正）

**① 逐提交纪律** 🟢：单 commit 单逻辑变更（trait+六型+测试+注册，一气呵成属 P1-2 一步）；message 工艺更正段自曝前科（前版反引号被 shell 吞段）——按证伪更正文化**原地披露而非删除**，合规。

**② 差分验证员纪律** 🟢：Evidence 贴件=命令输出重定向（message 自述"未手编辑"）；两行贴件（7 passed 322 filtered / 金样 GREEN 74）我**独立实跑逐字复核为真**，非手粘。锚引用随行（表 7 行锚+default 锚+floorTier 锚 sed 可验）——除 1.1 已记的 7528 起点与 message 总锚两处失准外无伪锚。

**③ AGENTS.md 权威** 🟢：Evidence/Confidence 字段在（Confidence:91）；未触 data/naming/reference/original（Evidence 字段非强制但已备）；无门禁改动；无阈值放宽。

**④ shell 反引号工艺修正判定（任务点名：合规改进 or 掩盖？）→ 合规改进**：
- 机理属实：shell 双引号内反引号=命令替换，`git commit -m "…\`default: return\`…"` 确会吞段失形——前版 message 的 `default: return` 代码段失形即此因。
- 修法合规：**Python 原文写文件 + `git commit -F`** = shell 零解释面，正文（含代码段）字节保真；这是"贴件纪律"（证据不手粘）的对偶面——**正文不被 shell 碰**，同族加固而非绕过。
- 非掩盖判据：(a) 前科**自曝**入 message（掩盖者不自曝）；(b) 更正后本版 message 反引号代码段实文完好（`default: return` 等在 message 中完整呈现，git show c819c73 可验）；(c) 工艺修正不触碰任何证据行内容——两行 Evidence 独立复跑仍真；(d) 未绕过任何门禁（commit-msg 证据门禁对 core 面本非强制，且 message 自带 Evidence）。
- 结论：**贴件纪律同族的对偶面加固（禁 shell 碰正文），合规改进，非掩盖**。

## 8. 总判

**攻击点级：红 1 / 黄 1 / 绿 5（=0. 表）—— 总判：需修复（不可直推）。**
（条目级展开：红 1 条=Equip 加成；黄 4 条=锚 7528 起点、message 总锚 7631 截臂、HpUp 9999 无源、floor_tier 方法+kind 词表缺口；绿 5 攻击点。）

- 🔴 必修（1）：**Equip 臂补换装加成**（equip_bonuses 接线 + 测试补 atk/def 断言红→绿）——否则"逐字抽象"声明对 Equip 臂不实，P1-3 切换即保真破坏。
- 🟡 建议随修（4）：①表行 1 锚 7528→7534（或 7538 起点）+ message 总锚 7528-7631→7534-7649（含装备臂，消 spec §2.2-① 同类截断）；②HpUp 删 9999 无源钳+撤"编排面同值"失实注释；③floor_tier 移出 ItemCtx（自由函数）消三宪字面争议；④kind 词表（builtin_effect 五键 vs items.json 13 键，TutorialSeenStack 缺口）补 type→kind 映射书面约定，钉给 P1-3。
- 🟢 保持：分发未切红线、7/7 测试真绿、两层未知语义真、验证账全真、shell 工艺修正合规。

> 修复后复验点：EquipEffect 加成断言红→绿实录 + 全锚 sed 复跑 + `gates check` 13 绿 + 金样 74 GREEN。
