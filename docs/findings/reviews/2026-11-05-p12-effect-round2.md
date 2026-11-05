# P1-2 二轮复核：Equip 加成双动+黄四连清（commit 0bf322d）

> 独立对抗复核 · 二轮新视角 · 差分验证员纪律 · AGENTS.md 权威
> 复核对象：commit `0bf322d`（未推送，绿=lead 推送）
> 一轮：docs/findings/reviews/2026-11-05-p12-effect-review.md（红1黄1绿5 → 判需修复）
> 复核时间：2026-11-05

## 0. 结论速览

复核对象：`0bf322d`（未推送）。**红 0 / 黄 3 / 绿 4 —— 总判：可推送**（无行为/保真红；黄 3 均非行为面：①③建议 push 前 amend 顺手清，⑤钉 P1-3 前）。
一轮红1黄1（判需修复）已对照复验：**红-1 行为缺口确认已清**（registry 双臂代码逐字对拍 a.java + 实跑绿）；残留与新引入问题降级为黄。

| # | 攻击点 | 判 | 一句话 |
|---|---|---|---|
| 1 | 红-1 核：Equip 加成双动 | 🟡 **黄** | 红核已清：registry 双臂 `−=/+=` 与 a.java:7677-7697 逐字对拍 PASS、8/8 实跑绿、+10 断言能捕原 bug（revert 即红）；残留三黄：**新错锚 m_083 7688-7694**（应 7699-7709/循环 7702，effect_registry:210 新注+message 两处，抄自 scene.rs:886 既有陈旧注——本 commit 黄①正修同类病却新引入）、**def/防具臂零断言**（round-1 明要 atk/def 只交 atk）、**−半空转**（bonuses[0]=0 减0不可观测，删该行测试仍绿）+第三例"低档"标签实为同档重复 |
| 2 | 黄①锚：7528 stale 更正 | 🟢 **绿** | 对照表 **6 行锚全量 sed 复核全中**（超额完成"抽3行"）；7534/7537/7528∈gainGold 地标全对；scene.rs:4243 同批改对；message 无错误总锚残留；两处形制注：round-1 处方的总锚 7534-7649 未在 message 显式给出（改为组件地标+表内逐行锚，截断错误已消）、"截臂：switch 终=7631"字面读作 switch 止于 7631 为假（闭合 7648），意图=旧锚终点 |
| 3 | 黄②HpUp：9999 证伪→真语义 | 🟡 **黄** | 核心两问通过：9999 已删（仅剩证伪注记）、证伪原地入账 ✓、"编排面同值"失实注撤 ✓、真语义 registry=scene.rs:4393=a.java:5172-5178 sed 逐字 ✓；**"测试钉"严格读为假**——revert 到 9999 钳全测仍绿（hp==200 在新旧版同值），floor 分支与"非上界钳"均无判别测试（可写未写：hp=9900 判别钳、负 delta 判别地板）；round-1 未要求测试，但本件自报"钉"属过强声明 |
| 4 | 黄③三宪：floor_tier 独立域函数 | 🟢 **绿** | `impl ItemCtx` 已移除（grep 0）、floor_tier 独立函数体 vs a.java:7651-7662 逐字、签名 ctx+1≤2、调用点全改实跑绿；round-1 正式处方③全交付；注：entities 死字段残留（round-1"附带"项非正式处方，纯数据/冻结两宪不受影响，移除需走 schema 流程） |
| 5 | 黄④kind：七 kind 注册+8/8 覆盖 | 🟡 **黄** | 字面全真：ItemAction 七变体全注册（逐一比对枚举）、覆盖测 12 键全非 NoOp、8/8 实跑绿；但 round-1 原题**未闭**：Passive(4行)/TutorialSeenStack(id15) 仍缺（items.json 13 键只覆盖 11）、type→kind **书面约定未写**（round-1 正式处方④本体）、UseArmEffect 无条件 Consumed 与真实逐 kind 消耗不一致（OpenPanel 永不消耗等）——按 effect.key 接 pickup 将静默错发（此前 miss 是响亮 NoOp）；deadline"P1-3 前"未到非逾期，但"缺口清"仅按七 kind 窄读为真 |
| 6 | 验证账（8/8+328+金样74+23臂+夹带） | 🟢 **绿** | 三贴件独立实跑逐字同（8/8、328/0/2、GREEN74）；23 臂代码零动——**"区空"字面不成立**（diff=1 注释行 4243 锚更正，round-1 处方项、message 已披露），`=>` 臂行 23 字节不动、金样 GREEN 证零行为漂移；夹带净；总闸 PASS=13 FAIL=0 SKIP=0 |
| 7 | 三纪律（-F 文件制贴件） | 🟢 **绿** | 逐提交=单补正单元 ✓；贴件实真（独立复跑）✓；Evidence 字段缺席但仅触 core 非 §9 强制面、Confidence:92 在、验证段代偿；message **0 反引号**无 shell 吞段面，工艺合规；唯两处声明失准（锚 7688-7694、测试钉）已分别计入点1/点3 |

## 1. 红-1 核：Equip 加成双动

### 1.1 try_equip 逐字对拍（scene.rs:865 sed 实文 vs a.java 原文）

awk 逐行实文对拍 scene.rs:865-884 `try_equip` ↔ a.java:7677-7697 `tryEquip`：

| 件 | scene.rs:865-884 | a.java:7677-7697（sed） | 判 |
|---|---|---|---|
| 分段 | `if tier < 6`→m_083_index(weapon,0,6)；else→(armor,6,12) | `if (var1<6)`→equipTierIndex(weapon,0,6)；else if→(armor,6,12) | ✓ |
| 拒绝 | `if cur >= tier && !force { return false }` | `if ((var4=…) >= var1 && !var2) { var3=false } else {…}` | ✓ 等价 |
| 加成双动 | `atk -= bonuses[cur]; atk += bonuses[tier]`（def 同构，:871-872/:879-880） | `playerAtk -= bonuses[var4]; playerAtk += bonuses[var1]`（7684-7685 / 7691-7692） | ✓ 逐字 |
| 换槽 | `equipped_weapon = equip_types[tier]` | `equippedWeaponType = equipTierTypes[var1]` | ✓ |
| force | 参数化；pickup 调用点全 false（scene.rs:4278/4297、a.java:7642） | `!var2`，调用传 false | ✓ |

registry EquipEffect（effect_registry.rs:184-207）= 同结构去 force（pickup 唯一调用即 false；force=true 走 scene.rs script_ops:281，不经 registry）——**双臂加成代码正确，红-1 行为缺口已清**。

m_083_index（effect_registry.rs:211-220 / scene.rs:887-899）↔ a.java:7699-7709 `equipTierIndex`：`t=hi; while t>lo {t-=1; …} 0` ≡ `var3=hi; while(--var3>=var2){…} return 0`——扫描区间 [lo,hi-1]、hi 永不触达、未命中返 0，逐语义同 ✓（sed：7702 = `while (--var3 >= var2)`）。

### 1.2 🟡 新错锚（本轮引入）：7688-7694 ≠ 先减后比

- **effect_registry.rs:210 新注释**：`m_083_index（a.java:7688-7694 逐字：先减后比倒序…）`；**message 同句**：`先减后比 a.java:7688-7694`。
- sed 实文（awk 7677-7709）：**7688-7694 = tryEquip 防具臂**（7688 else-if 调 equipTierIndex(6,12)、7691-7692 def −=/+=、7694 `}`）；**先减后比 `--var3` 在 7702，函数体 = 7699-7709**；武器段调用 [0,6) 在 7681（范围外）。该锚对其声明的语义（先减后比逐字）**错**；对"分段调用"也只盖防具段漏武器段。正确锚 = **a.java:7699-7709（循环 7702）**。
- 根因：抄自 scene.rs:886 既有陈旧注（`equipTierIndex（a.java:7688-7694）`——本件未动该行；2c78e21 写入时函数在 7690 附近近似对，a.java 后经 +6 漂移失配；scene.rs:888 `a.java:7693 while(--table_count…)` 同族双错，均属既有欠账）。但**本件新写"逐字"并入 message = 新引入错锚**——恰是本 commit 黄①正修的同类病。判🟡。

### 1.3 🟡 测试半钉（round-1 处方"测试补 atk/def 断言"只交 atk）

`equip_arm_matches_match_semantics`（effect_registry.rs:356-377）实跑 8/8 绿：

- ✓ **+10 断言真且判别**：:370 `assert_eq!(c.player.atk, atk0 + 10, "…+bonuses[1](10)−bonuses[0](0)…")`——revert 加成则 atk=0≠10 即红→绿，红-1 的 += 半被钉住；message 算术（+10−0）如实。
- ✗ **−半空转**：空档起始 cur=0、bonuses[0]=0，`atk −= bonuses[cur]` 是减 0——**删该行测试仍绿**；判别需升级用例（tier1→tier2：0→10→10−10+30=30，缺减法则 40），未写。
- ✗ **def/防具臂零断言**：round-1 处方明写"测试补 **atk/def** 断言"，只交 atk；三次 apply 全 tier<6（武器臂），tier≥6 防具臂无用例。
- ✗ **第三例标签失实**：:374-76 注释"低档→失败"但入参仍 tier:1、现档亦 1 = **同档重复**（与第二例完全同构）；真降级（cur>tier，现有 tier2 贴 tier1）未测。
- 代码双臂本身正确（1.1 对拍 + a.java:7691-7692 sed），故判**黄非红**：行为缺口已清，测试处方半交付。

## 2. 黄①：7528 stale 锚更正

### 2.1 对照表行锚全量 sed 复核（任务"抽3行"→实做全量 6 行）

逐行 `awk NR>=a && NR<=b reference/src/deobf/a.java` 实文核：

| 表行 | 表锚 | sed 实文命中 | 判 |
|---|---|---|---|
| 13..=25\|85\|86 →Stack | **7537-7555** | 7537 `switch (var1) {`、7538 case15、7540-7553 case13-25/85/86、7554 addItemToItemStack、7555 return ——逐行命中 | 🟢 |
| 26/27/28 →Key | 7556-7564 | case26@7556、27@7559、28@7562、各 return@7558/7561/7564 | 🟢 |
| 29/30 →StatUp | 7565-7580 | case29@7565（floor≤10→++/否则+floorTier@7571）、case30@7573（def 同构@7579）、return@7572/7580 | 🟢 |
| 31/32 →HpUp | 7581-7587 | case31@7581 applyHpDelta(50×)@7583、case32@7585 applyHpDelta(200×)@7587（32 无 return 落穿，items.json 已注"落穿无害"） | 🟢 |
| 33..=36\|79 / 37..=40\|80 →Equip | 7632-7648 | case33@7632…case80@7641、tryEquip@7642、浮层双分支7643-7647、`}`@7648 | 🟢 |
| default →NoOp | 7630-7631 | `default:`@7630、`return;`@7631 | 🟢 |

**地标（message 声明，sed 实证）**：pickupItemType=**7534** ✓、switch=**7537** ✓、7528=`this.goldAmount += var1;`∈gainGold(7527-7532) **stale 确认** ✓、7631=default return（旧锚终点，7632 起截掉装备臂）✓。

### 2.2 修复落地核

- 表行1 锚 7528-7555 → **7537-7555** ✓（round-1 处方 7534/7538 之间取 switch 线，起点不再落 gainGold）；
- scene.rs:4243 同族 stale 锚同批改 ✓（7528→7534+日期注记；diff 唯一 hunk，见 §6）；
- message **无错误总锚残留** ✓（旧"7528-7631"双错总锚消失，改组件地标列举；装备臂 7632-7648 在表内不再被截）。

### 2.3 形制注（非伪锚）

- round-1 处方"message 总锚→7534-7649"未以单一字符串显式给出——以组件地标（7534起/7537/7631）+表内逐行锚替代，**截断病灶已消、信息等价**，形式未逐字落地；
- default 行注"（截臂：switch 终=7631）"：字面读"switch 止于7631"为**假**（switch 闭合@7648），意图=记录旧锚终点（round-1"截臂"语境），宜写"旧锚终=7631"。

**判：🟢 绿**（行锚全量 sed 命中；两处形制/措辞注，无伪锚）。

## 3. 黄②：HpUp 9999 证伪与真语义

### 3.1 核心两问（任务点名）：证伪入账？真语义锚核？

- **9999 已删**：grep 全仓 `9999`——effect_registry.rs 仅剩 :161 注记"9999 前值=C 级臆造已证伪"；`max_hp=9999` 代码行消失 ✓。a.java 本身无 9999（round-1 已证）✓。
- **证伪更正入账** ✓：新注记【黄②修复=移植错】原地写明"apply_hp_delta 真语义（scene.rs:4393 逐字）：hp≤-delta→hp=1 伤害地板非 9999 钳——9999 前值=C 级臆造已证伪；否则 hp+=delta"——错在哪（移植臆造）+真值来源（scene.rs:4393）+日期语境（commit 2026-11-05）齐；round-1 点名的失实注"编排面同值"已撤 ✓（grep 无残留）。
- **真语义与 a.java 锚核** ✓ 三方对拍：
  - a.java:5172-5184 `applyHpDelta`：`if (playerHp <= -var1) { playerHp=1; var1=playerHp-1; } else { playerHp+=var1; }`（awk 实文）；
  - scene.rs:4393 `apply_hp_delta`：同构逐字（floor 分支 hp=1，delta 归 1 供 popup 判号）；
  - registry HpUpEffect（:168-174）：`d = delta×floor_tier(floor); if hp <= -d { hp=1 } else { hp+=d }`——**状态面与前两者逐字同**；registry 不做 popup（delta 归号副作用=编排面，ItemOutcome 不携带，与两面并存架构一致，非缺口）。

### 3.2 🟡 "测试钉"声明过强

- message："……否则 hp+=delta——**注记+测试钉**"。
- 实核：唯一相关断言 `hpup_arm`（:354 `assert p.hp==200`，delta=200、tier1）——**在旧9999-clamp 下同样通过**（(0+200).clamp(0,9999)=200）；**revert 本修复，全部测试仍绿**（无任何判别用例：hp+d>9999 情形、负 delta 地板情形均未构造）。
- 即：注记 ✓ 真；"测试钉"按差分纪律（钉=红→绿可判别）**不成立**——恰是 round-1 对 equip 批过的"测试对该缺口免疫"同款病。
- round-1 对②的处方是"删钳或补 A 级锚+撤失实注释"（未要求测试）——处方本身全交付 ✓；黄因**自报"测试钉"过强**而立。清法（2 选 1，随修非阻断）：补 `hp=9900, delta=200 → hp=10100`（判别钳）与 `hp=5, delta=-10 → hp=1`（判别地板）两行用例；或措辞降级为"注记+正路径覆盖"。

**判：🟡 黄**（证伪入账与锚核全过；"测试钉"声明失准）。

## 4. 黄③：三宪——floor_tier 独立域函数

spec §3.2 三宪逐条 + round-1 处方③（"floor_tier 移出 ItemCtx"）交付核：

| 宪/处方 | 要求 | 实测 | 判 |
|---|---|---|---|
| 1 纯数据 | ctx 无方法/无行为 | `impl ItemCtx` 块**已整块移除**（grep `impl ItemCtx`=0）；floor_tier 改 `pub fn floor_tier(floor: i32) -> i32`（:64） | 🟢 |
| 2 字段冻结 | 结构固定不增删 | 8 字段原样（无中途增删） | 🟢 |
| 3 ctx+≤2 | 每 trait 方法=ctx+至多2语义参 | `fn apply(&self, ctx: &mut ItemCtx<'_>, params: &EffectParams) -> ItemOutcome` = ctx+**1** 参；全部 impl 同签名（grep 核 7 处） | 🟢 |
| 处方③本体 | floor_tier 移出为自由函数 | ✓ 落地；调用点同步改 `floor_tier(ctx.floor)`（StatUp :150、HpUp :168），旧方法调用 0 残留 | 🟢 |

- **函数体逐字核**：新 `floor_tier` vs a.java:7651-7662 `floorTier`（awk 实文）：`(floor-1)/10` 钳 `<0→0`、`>50→0`、`+1` ——四段逐字同 ✓；round-1 曾核过的等价性保持。
- **实跑**：p1a2 8/8 绿含 statup_floor_tier_arm（floor12→tier2=+2）✓。

### 残留注（不改判）

- round-1 §4 附带点名的 `entities` 死字段**仍未读**（grep effect_registry：仅 :53 声明 + :284 测试构造，六型+七 kind 效果零读取）——round-1 正式处方③只写 floor_tier，entities 属"归1.3一并修"附带语；equip_bonuses 已随红-1 接线（:194-195/:202-203 读取）✓。纯数据/冻结两宪不受影响；清法=P1-3 接线或走字段冻结 schema 流程移除，随修。
- 与 scene.rs 自身 `Self::scaled_by_floor_tier`（:2538，GameScene 静态方法）**不同域不冲突**——后者是编排面既有函数，非 ItemCtx 方法，三宪不辖。

**判：🟢 绿**（三宪全过、处方③本体交付、函数体逐字；entities 附带残留记注）。

## 5. 黄④：ItemAction use 七 kind 注册与覆盖测

### 5.1 字面三件套全真（任务点名，实跑）

- **七 kind 全注册** ✓：`builtin_effect` match 新增 `"OpenPanel"|"RemoveAdjacent"|"RemoveAllType"|"Heal"|"KillAdjacent4"|"MirrorTeleport"|"ChangeFloorUp" => &UseArmEffect`（:249-251）——与 item_effect.rs `enum ItemAction` 七变体**逐一比对同集**（OpenPanel/RemoveAdjacent/RemoveAllType/Heal/KillAdjacent4/MirrorTeleport/ChangeFloorUp）；
- **覆盖测存在且真** ✓：`kind_coverage_full_table`（:381-391）遍历 12 kind
  > 【2026-11-05 更正】"删任一注册臂即红（判别真）"为读码推断未实跑——
  > p13 复核 M5 变异实测证伪（ptr::eq 跨 CGU 恒 false 伪钉）；已按红-1 修复
  > 换 kind_name 值比较判别+变异自证 M2-M6 全红（见 p13-gates-review §1）。 断言 `!ptr::eq(builtin_effect(kind), &NoOpEffect)`——删任一注册臂即红（判别真）；
- **8/8 无 miss 实跑** ✓：`cargo test -p game-core --lib p1a2` → `8 passed; 0 failed; 322 filtered out`——与 message 贴件**逐字同**（round-1 为 7 测，本件 +kind_coverage=8）。

### 5.2 🟡 round-1 原题未闭（处方④本体）

round-1 §1.5/处方④的本体是"**type→kind 归一映射书面约定，钉给 P1-3**"+点名 TutorialSeenStack 缺口。实核残留：

1. **词表仍缺口**：items.json 13 个 effect.key vs registry 12 键——`Passive`（4 行：下行符/太公杖/捆仙绳/幸运币）、`TutorialSeenStack`（id15 筋斗云）**仍未注册**（python 全量比对：unregistered=[Passive, TutorialSeenStack]；registry-only=[Stack]）。即：id15 与 4 个 Passive 项按 effect.key 接线仍 miss→NoOp——round-1 点名的那一个（TutorialSeenStack）恰未清。
2. **书面约定未见**：全仓无新增 type→kind/effect.key 归一映射文字（grep 分发键/归一映射 仅 :223/:248 两处"分发键面"注，非映射表）；round-1 要的"表行语义（按 type 臂）与注册键（按词表）之间缺一层"仍缺。
3. **UseArmEffect 语义陷阱**（新增面）：七 kind 一律返回 `Consumed{overlay_tier:None}`，与 item_effect.rs `execute_item_action` 真实消耗**不一致**——OpenPanel 实返 `false`（不消耗）、RemoveAdjacent 条件消耗、ChangeFloorUp 视 change_floor。注释自认"行为面=execute_item_action 编排执行，本行=分发键可达性钉"（:222-223）——**若 P1-3 按 effect.key→builtin_effect 接 pickup/消耗判定，将静默错发**（此前 miss 是响亮 NoOp，现变静默 Consumed，风险面反而变大）。

### 5.3 缓冲事实（不升红的理由）

- 分发未切：`builtin_effect` 生产调用 **0**（grep 全仓仅 effect_registry 自身+测试）——黄④三个问题全部**潜伏**，无现行漂移；
- round-1 对④的定性本就是"P1-3 前必须补钉"（deadline=P1-3，非本件），且"本件不判错"——本件在 deadline 内；
- message"kind 缺口清"按其冒号后限定（"ItemAction use 七 kind……"）窄读为真；按 round-1 全口径（13 键词表）读则未清——声明口径有歧义，记注。

**判：🟡 黄**（字面三件套全绿；round-1 处方④本体未交——词表 2 键缺口+书面约定缺失+UseArmEffect 消耗语义陷阱，全部 P1-3 前必须补钉）。

## 6. 验证账

| 项 | message 贴件 | 独立实跑 | 判 |
|---|---|---|---|
| 等价测 8/8 | `8 passed; 0 failed; 0 ignored; 0 measured; 322 filtered out; finished in 0.00s` | `cargo test -p game-core --lib p1a2` → **逐字同**（8 测名单：stack/key/statup/statup_floor_tier/hpup/equip/**kind_coverage**/unknown_noop） | 🟢 |
| core 328/0 | `328 passed; 0 failed; 2 ignored; …12.47s` | `cargo test -p game-core --lib` → `328 passed; 0 failed; 2 ignored`（12.28s 机时差，计数逐字同；2 ignored=bug044 debug_assert+probe_bottom 既有） | 🟢 |
| 金样 74 --check | `[GREEN] 金样逐字节等价：74 场景 sha 全同` | `python3 tools/state_baseline.py --check` → **同句 GREEN**（后台 job 实跑复现） | 🟢 |
| 总闸 | — | `gates/cli.py check` → **PASS=13 FAIL=0 SKIP=0**（含 state-baseline GREEN 内嵌复现）；`status` 另示 visual-diff SKIP=1=P4 既有显式项 | 🟢 |
| 夹具 arm_coverage | round-1 在册 21 组 | `cargo test -p game-oracle --test arm_coverage` → `p1a2_all_21_arm_groups_have_fixture_triplets … ok` | 🟢 |
| 23 臂 switch 零动 | `23 臂 switch 照旧零动` | `git diff c819c73 0bf322d -- scene.rs` = **1 hunk / 1 行**，且为 :4243 **注释行**（7528→7534 锚更正）——**"区空"字面不成立**（区内非空），但：①注释-only 零代码；②message 已披露"scene.rs:4243 同族 stale 锚同批改"；③round-1 认账的既有陈旧锚、处方项提前修（round-1 原约 P1-3 随注一并更正）；④函数体 `=>` 臂数 **23**（grep 实测，与 round-1 口径同）且 23 行字节不动；⑤金样 GREEN 证零行为漂移 | 🟢（实质红线=23 臂代码零动成立；字面"区空"不成立已如实记） |
| 夹带净 | 2 文件 | `--name-only` = effect_registry.rs + scene.rs，与 message 申报面一致；`git diff HEAD` 空（tracked 无未提交改动）；untracked 仅既有复核 md×5 + android/build | 🟢 |
| dispatch 未切（两面并存） | 分发未切 | `grep -rn builtin_effect crates/ --include=*.rs \| grep -v effect_registry` = **0 命中**（无生产调用方） | 🟢 |

- 贴件真伪总核：三行 Evidence 输出均独立复跑**逐字复现**（非手粘）；唯一非逐字项=耗时（12.47s vs 12.28s，机时波动）。

**判：🟢 绿**（全项实跑通过；"区空"按实质口径绿、字面口径已如实披露）。

## 7. 三纪律裁定（-F 文件制贴件=工艺合规面）

**① 逐提交纪律** 🟢：单 commit 单逻辑（P1-2 补正=红-1+黄四连一个修复单元，与一轮 c819c73 同粒度）；type=fix scope=core 合规；message 自述"复核红1黄1 判回派修复"与一轮判决对应 ✓。唯 message 内两处声明失准（锚 7688-7694、测试钉）已分别计入点1/点3，不另计。

**② 差分验证员纪律** 🟢：贴件=命令输出重定向性质（三行独立复跑逐字复核为真，非手粘）；锚引用随行 sed 可验——对照表 6 行+地标+tryEquip/applyHpDelta/floorTier 对拍全中，**唯 m_083 锚 7688-7694 一处伪锚**（已计点1黄）；"sed 实文复核"就其所列对象（stack 行三地标）为真。

**③ AGENTS.md 权威** 🟢：
- **Evidence 字段缺席核**：message 无 `Evidence:` 行——核 §9 规则，Evidence 强制面=`data/naming/ / reference/ / original/`，本件仅触 `crates/game-core/`（2 文件）→ **非强制，不违规**；`Confidence: 92` 在 ✓；验证段三行输出代偿实质证据 ✓。
- 无门禁改动（check 13 绿实跑）、无阈值/白名单放宽、original/reference/data/naming 零触（diff 仅 2 core 文件）✓。

**④ -F 文件制贴件=工艺合规面** 🟢：
- message 全文 **0 个反引号**（`grep -c '\`'` = 0）——shell 命令替换风险面为零，一轮点名的"反引号吞段"病灶结构性不存在；
- 正文完整无失形：代码段 `atk/def -= bonuses[cur] += bonuses[tier]`、`[0,6)/[6,12)`、`hp≤-delta→hp=1` 等字节完好（git log 实读核）；
- 三贴件与实跑一致 → "贴件纪律"（证据不手粘）与"正文不被 shell 碰"两面均成立。
- 结论：**工艺合规**（较一轮 Python+-F 方案或更简：直接无反引号可走 -m 亦零风险）；合规面裁定同轮一"同族加固非掩盖"。

**判：🟢 绿**。

## 8. 总判

**攻击点级：红 0 / 黄 3 / 绿 4 —— 总判：可推送**（绿=lead 推送；无行为/保真红）。

| | 一轮 | 二轮 |
|---|---|---|
| 红 | 1（Equip 加成缺失=真行为缺口） | **0**（代码逐字对拍+实跑绿，缺口确认已清） |
| 黄 | 1（锚7528+HpUp9999+floor_tier+kind，展开4条） | 3（点1锚/测试半交付、点3测试钉、点4 round-1处方④本体） |
| 绿 | 5 | 4（点2锚、点4三宪、点6验证账、点7三纪律） |
| 判 | 需修复 | **可推送** |

**红-1 结案证据**：registry EquipEffect 双臂 `−=/+=` + 分段 m_083 倒查与 a.java:7677-7697/7699-7709 逐字等价（force=false= pickup 调用面）；+10 断言 revert 即红（判别真）；金样 74 GREEN。

**黄3（均非行为面，随修非阻断）**：
1. **点1**：m_083 锚 7688-7694 错（新注释+message 两处，应 7699-7709/循环 7702——抄自 scene.rs:886 既有陈旧注，本件新写"逐字"即新引入）；def 臂零断言（round-1 处方"atk/def"半交付）；−半空转（bonuses[0]=0）；第三例"低档"标签实为同档重复。
2. **点3**："测试钉"严格读不成立（revert 9999 全绿，无判别用例）——注记与锚核本身全过。
3. **点5**：round-1 处方④本体未交——Passive/TutorialSeenStack 2 键仍缺、type→kind 书面约定未写、UseArmEffect 无条件 Consumed 与真实消耗不一致（潜伏，P1-3 前必须补钉）。

**建议**（可推送前提下，按性价比）：
- **push 前顺手**（2 分钟）：改锚 7688-7694→7699-7709（2 处）；HpUp 补 2 行判别用例或降级"测试钉"措辞；def 臂补 1 条 tier≥6 断言（round-1 处方补全）。
- **P1-3 接线前**（硬门）：type→kind 书面约定 + Passive/TutorialSeenStack 归一 + UseArmEffect 消耗语义与 execute_item_action 对齐或显式"仅键可达"声明；entities 字段处置。
- 三处黄互不耦合、全部文档/测试/潜伏面，不改任何行为路径——**不构成推送阻断**。

> 修复后复验点（若 push 前随修）：锚 sed 复跑 2 处、p1a2 9/9（+def 断言）或 8/8 措辞版、gates check 13 绿、金样 74 GREEN。
