# P2 批2 对抗复核报告 — commits 2a2f6c8 + 760a1bb

> 被审件：`2a2f6c8` refactor(core)「Y1 注释终形」（1 文件 +4/-2）+ `760a1bb` feat(core)「P2 批2 BattleSubsystem resolve_bump trait 化（3 事件一次性切）」（4 文件 +504/-130），modern 分支
> 复核方式：对抗复核（只读；唯一写=本报告 + 任务书授权的变异与还原）；证据 A/B 级带实测输出；纪律=AGENTS.md（保真=金样裁判；证伪更正文化=日期更正不删原文）
> 日期：2026-11-07 · 被审 HEAD=`760a1bb5e0262c3b82da13e9697c51d9ad585bda` · parent=`2a2f6c878dac6856758fb2f1ef9ea9556335c1dd`

## 总判：可推送（七面全绿；面2 内 2 条黄-轻 + 面6 内 1 条黄-轻随批登记，不阻断）

签名扩展=**合规扩展**（spec:98 原文未删 + spec:111 日期更正注记格式合规 + ctx+≤2 恰满 + plan 批2「scene 战斗路径编排面→BattleOutcome」授权依据成立）；a.java 数值对拍**全对**（Engage 双臂 hp 源/先手/active/no_counter 逐项同、Exchange 数学 `dmg>0 → (&3)==0 → 扣血 → 反击/击杀 → 赏金 → boss 钩子 → tick++` 与 `gainGold` 的 `>0` popup 守卫逐行同）；`battle_enemy_hp` 8 非写点逐条实证 + resolver 外零真写点；4 query 点零改（diff 中根本不出现）+ math 单源 `crate::combat`；TDD 红档 mtime 早于 commit 43s、贴件 byte 级 `$(cat)` 相等；**变异独立复做**=hp 源互换 → `left: 50 / right: 40` 红、还原 `git diff` 空；fixture_frames 45/45 独立重跑绿；夹带=恰 1+4 文件全在声称域。

3 条黄-轻（均不阻断，建议随批勘误/批10 登记）：
1. **a.java 行锚过期**（HEAD 上不可复现）：`battle.rs:81/:116` 与 `scene.rs:3730` 的 `a.java:9983-9982`（HEAD 9983=地图格子写、真身 9992-9996）、`battle.rs:15/:133` 与 `scene_tests/entity_route_tests.rs:106` 的 `a.java:8769`（HEAD 8769=**空行**、真身 8775）——根因=deobf a.java 于 `dc470b2`(10-15,+3) 与 `d389ddf`(10-16,+6) 两次前移行号，指针未随更。
2. **`a.java:8760-8780` 覆盖不全**：HEAD 上 route 臂=8771-8787，被引区间止于 `routePhase=5`(8780)，**漏 8781 `battlePredictGate=false` 与 8783-8784 敌先手一击**（恰是该注释自述的「敌先手 raw」）。
3. **遗留直写披露只在 commit message、码内零注记**：`grep 遗留|批10` 在 scene.rs/battle.rs/mod.rs 零命中（plan 批10 确有「直写清零收尾核 / scope=382 直写站点」授权，披露内容属实、非夹带）。

---

## 面1 签名扩展裁定（审查点①，先裁定） — 🟢 绿 = **合规扩展**

**裁定：合规扩展（非违 spec 私改）。** 三条依据全部实测：

**① spec 原文未删 + 更正注记格式合规**（AGENTS.md §3.6「日期更正，不删原文」）：

```
$ sed -n '96,99p;111,112p' docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md
### 3.1 trait 面（A 补漏后八件）
```rust
trait BattleSubsystem { fn resolve_bump(&mut self, ctx: &mut Ctx, t: EnemyTypeId) -> BattleOutcome; }   ← 原文仍在（行98，未删未改）
trait DialogSubsystem { fn tick(&mut self, ctx: &mut Ctx) -> DialogStep; }
> 【2026-11-07 更正·批2】③ BattleSubsystem::resolve_bump 实现为 `resolve_bump(&mut self, ctx, t, ev: BattleEvent) -> BattleOutcome`（+ev 参——plan 批2「scene 战斗路径编排面→BattleOutcome」三事件面[途中开战/撞击开战/一击交换]需事件分辨；ctx+≤2 恰满 t+ev）。原文单 (ctx,t) 形不删备查。   ← 行111
> 【轮2修复】四处注记：

$ grep -n "更正·批2" docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md
111:> 【2026-11-07 更正·批2】③ …
```

- 日期头 `【2026-11-07 更正·批2】` ✓、原地标注 ✓、**原文单 (ctx,t) 形不删备查**（显式声明保留）✓ → 格式合规。
- 注记以 `③` 编号，与 spec 已有的批0/批1 更正注记同序列，非旁逸斜出。

**② ctx+≤2 恰满（形参计数）**：`battle.rs:78`

```rust
fn resolve_bump(&mut self, ctx: &mut BattleCtx, t: EnemyTypeId, ev: BattleEvent) -> BattleOutcome;
```

`t` + `ev` = 2 参，恰满「ctx + ≤2」上限；`BattleCtx` 类型替换 `Ctx` 属批0 已裁授权（批0-b 复核），本批未再扩。

**③ plan 批2 依据成立（「三事件需事件分辨」的授权链）**：

```
$ grep -n "批2" docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md
90:| 批2 | **BattleSubsystem**：resolve_bump trait 化（combat 函数族+scene 战斗路径编排面→BattleOutcome）；battle_enemy_hp 写面归因核 | 战斗夹具（preset 帧断言）+金样零重冻 | 1-1.5d |

$ sed -n '30p' docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md   ← 批0 现状普查行（表 1 BattleSubsystem）
| 1 | BattleSubsystem | `resolve_bump(&mut self, ctx, t) -> BattleOutcome` | combat.rs 198 行 8 fn（…）+scene.rs 战斗路径（2053/2060/3730-3732/3756/3772/3642） | 无 trait 面=函数族+scene 内联编排 |
```

- plan 原文**无「三事件」字样**（诚实披露点）；但批2 行明确「scene **战斗路径编排面** →BattleOutcome」，而 plan:30 现状普查已把该路径的 6 个调用点逐个点名：`2053/2060`=途中开战、`3642`=撞击开战、`3756/3772`=一击交换、`3730-3732`=预测查询——**三个事件面在 plan 中已作为同一「编排面」被圈定**，故「需事件分辨」是从 plan 调用点枚举的可复现推导，不是凭空扩权（B 级：调用点枚举可 grep 复现）。
- 注：plan:30 的「方法面」列仍是 2 参旧形——那是**批0 现状普查快照（迁移前状态）**，不是对新签名的否决；签名终形的记载责任在 spec，由 spec:111 更正注记承担（面1 ①）。
- 反事实核：若不加 `ev`，单一 `resolve_bump(ctx,t)` 无法区分「bump=基础表源+无先手」与「route=缩放表源+敌先手」（两臂 hp 源不同、`battlePredictGate` 相反）——数值上必须分辨（面2 实测）。

**残余提示（不改裁定）**：注记引用的是 plan 授权（架构签名注记的正确指针类型），非 A/B 码行；且更正与实现同 commit 落地（无「先 spec 后码」时序）。AGENTS.md 未要求时序，仅登记。

## 面2 a.java 数值对拍（保真核心，A 级） — 🟢 绿（核心全对；行锚过期 2 条黄-轻）

方法：以 **HEAD 的 `reference/src/deobf/a.java`（10889 行）** 为唯一裁判（D3 锚点），逐臂 `sed -n` 取实文，与 `crates/game-core/src/subsystem/battle.rs` / `scene.rs` 调用点逐项对照。

### 2.1 Engage 撞击臂（bump）：a.java:5475-5491 ↔ battle.rs:121-131 + scene.rs 调用点

| 项 | Java 实文（HEAD 行号） | Rust | 同 |
|---|---|---|---|
| 预测门 | `5476 var5 = predictHpLossVsType(var5,true)` / `5478 if (var5>=0 && var5<playerHp)` else 你无法战胜它 | `loss=predict_engagement(...,no_first_strike=true)`；`!(loss>=0 && loss<ctx.player.hp)` → `Refused` | ✓ |
| hp 源 | `5484 battleEnemyHp = enemyBaseHp[entityType[var5]-41]`（**基础表**） | `battle.rs:124 *ctx.battle_enemy_hp = ctx.enemy_base[idx].hp;` | ✓ |
| active | `5485 battleActiveFlag = true` | `battle.rs:125 battle_active = true` | ✓ |
| 门位 | `5486 battlePredictGate = true` | `battle.rs:126 battle_no_counter = true`（N-0354 对应） | ✓ |
| target | `5483 battleTargetEntity = var5` | `battle.rs:127 battle_target = entity_slot` | ✓ |
| 先手 | case8 臂无先手伤害 | `Engage { first_strike: 0 }` | ✓ |
| scene 侧 | `5481 walkPhase=5` / `5482 entityState=3` / `5488 walkStepCount=0` / `5489 showOverlayMessage(0,"你无法战胜它",0,0)` | scene.rs:3637-3680 Engage 臂 `walk_phase=5`+`state=3`；`_` 臂 `walk_step_count=0`+`show_overlay_message(0,"你无法战胜它",0,0)` | ✓ |

### 2.2 Engage 途中臂（route）：a.java:8771-8787 ↔ battle.rs:132-140 + scene.rs:2049-2074

| 项 | Java 实文 | Rust | 同 |
|---|---|---|---|
| 预测门 | `8774 var6 = predictHpLossVsType(var18,false)` / `8775 battleEnemyHp = enemyHpScaled[var18-41]`（**缩放表**） | `loss=predict_engagement(...,no_first_strike=false)`；`battle.rs:133 *ctx.battle_enemy_hp = ctx.enemies[idx].hp;` | ✓ |
| active/门位 | `8777 active=false` / `8781 battlePredictGate=false` | `battle.rs:134-135 active=false` / `no_counter=false` | ✓ |
| 敌先手 | `8783-8784 var7=enemyAtkScaled-playerDef; if(var7>0) playerHp-=var7` | `Engage{first_strike: ctx.enemies[idx].atk - ctx.player.def}` → scene.rs:2066 `if first_strike>0 { self.player.hp -= first_strike }` | ✓ |
| 场面置位 | `8776 walkPhase=5` / `8779 state=5` / `8780 routePhase=5` | scene.rs:2062-2064 `walk_phase=5`+`state=5`+`route_phase=5` | ✓ |
| 拒绝臂 | `8785-8787 walkPhase=1 / facingDirection=f_byte_19` | scene.rs `walk_phase=1+facing` | ✓ |

**两臂 hp 源相反（bump=base / route=scaled）与 `battlePredictGate` 相反，是本批数值保真的要害**——面5 变异正打此处。

### 2.3 Exchange 数学：a.java tickBattle 10032-10105 ↔ battle.rs:141-192 + scene.rs:3774-3869

| 步 | Java 实文 | Rust | 同 |
|---|---|---|---|
| dmg | `10039 var8 = effectiveAttackVsType(var1t) - enemyDefScaled[var1t-41]`；`<=0 → 10100 walkPhase=0` | `dmg = atk - enemies[idx].def`；`dmg<=0 → penetrates:false` → scene `!penetrates → walk_phase=0` | ✓ |
| 空拍 | `10041 (f_int_151 & 3) != 0 → 跳过（无伤害无 popup）` | `(*ctx.battle_tick & 3) != 0 → Exchange{penetrates:true,blow:false}` → scene 无 popup 无扣血 | ✓ |
| 扣血+popup | `10043 battleEnemyHp -= var8` / `10044 spawnPopup((byte)(5+f_int_151%3), var8, …)` | `battle.rs:170 hp -= dmg`；scene.rs:3793-3801 `popups.spawn(5 + battle_tick % 3, enemy_dmg, ex, ey…)` | ✓ |
| 反击 | `10046 if hp>0 { 10047 var8=enemyAtkScaled-playerDef; if>0 playerHp-=var8 }` | `blow:true, player_dmg:counter`；scene `if !kill { if player_dmg>0 { player.hp-=player_dmg } }` | ✓ |
| 击杀预测 | `10052 var8=predictHpLossVsType(var1t, battlePredictGate); if>0 spawnPopup((byte)2, …)` | `predicted_loss=predict_engagement(...,*ctx.battle_no_counter)`；scene `if predicted_loss>0 { popups.spawn(2, …) }` | ✓ |
| 赏金 | `10057-10063 if findItemStackIndex(25)>=0 gainGold(enemyBaseGold<<1) else gainGold(enemyBaseGold)` | `battle.rs:179-183 gold = enemy_base[idx].gold << 1（find_item(25)>=0）/ else gold`；scene `player.gold += gold` | ✓ |
| 赏金 popup 守卫 | `gainGold`（a.java:7527-7533）：`goldAmount += var1; if (var1>0) spawnPopup(4,…)` | scene.rs:3820-3830 `player.gold += gold; if gold > 0 { popups.spawn(4, gold, …) }` | ✓ |
| boss 钩子 | `10066 currentFloor>50 → false`；`bossTypeOrder` 逆扫 `spawnBossEvent`；`case 73/74/75` | scene.rs:3837-3861 `if persist.floor<=50` + `BOSS_TYPE_ORDER.rposition` + `match t {73…74…75…}` | ✓ |
| 收尾 | `battleSceneRerun = …; battleFinishing = true` | scene.rs:3862-3863 `battle_rerun_scene = rerun; battle_finishing = true` | ✓ |
| tick++ | `10105 f_int_151++`（在 `battleTargetEntity>=0` 的 else 内、dmg if/else **之后**，无条件） | scene.rs:3869 `self.battle_tick += 1`（同位：match 后、`else{ target<0 }` 分支外、`if !battle_finishing` 内） | ✓ |

**补充实证（跨面常量）**：`effectiveAttackVsType` Java=`traitFlagsByType[var1]` + `findItemStackIndex(23)/(24)` 护符双倍（a.java:9998-10007）↔ Rust `combat.rs:93-102` + `TRAIT1_AMULET=23 / TRAIT2_AMULET=24`（combat.rs:162/164）✓；`predictBattleHpLoss` 六参序与 Java 调用点 a.java:9995 一致（combat.rs:104-135）✓；`trait_flags` 索引语义 Java=`traitFlagsByType[型]` ↔ Rust=`enemy_base[型-41].trait_flags`（两表同源、基址差 41，批0 已定）✓。

**黄-轻 A（行锚过期，HEAD 不可复现）**：

```
$ sed -n '8769p;9983p' reference/src/deobf/a.java
（空行）
       this.f_short_arr_14[var3] = (short)(var1 + var2 * this.mapCellsWide);   ← 9983 是地图格子写
```

- `a.java:8769` 真身为 `8775 battleEnemyHp = enemyHpScaled[var18-41]`；`a.java:9983-9982` 真身为 `9992-9996 predictHpLossVsType` 定义。
- 根因（A 级，git 实测）：`git show e772fef:a.java`（10883 行）8769 **恰是** enemyHpScaled 赋值、`predictHpLossVsType` 在 9986；`git show 5d883b4:a.java`（10880 行）定义在 **9983**——即两枚指针在写入时（scene.rs 注记=`2c78e21` 2026-10-11）**是准的**，其后 `dc470b2`(10-15,+3) 与 `d389ddf`(10-16,+6) 两次前移行号，指针未随更。
- 本批新增的 `battle.rs:81/:116/:15/:133` **沿用了过期指针**（含 `scene_tests/entity_route_tests.rs:106` 的 8769）。数值/分支无一错（本表逐项验过），仅指针失效 → **黄-轻，不判红**；建议随批或批10 做一次 `a.java` 行锚勘误（把 9983→9992-9996、8769→8775、8760-8780→8771-8787）。

**黄-轻 B（新注覆盖不全）**：`a.java:8760-8780` 在 HEAD 上止于 `routePhase=5`(8780)，漏 `8781 gate=false` 与 `8783-8784 敌先手`——恰是该行注释自称的「敌先手 raw」。该区间共 4 处（`scene.rs:2053`、`battle.rs:15`、`battle.rs:133`、`battle.rs:289` 测试注），本 commit 新写；语义已逐项验对（上表），属指针区间问题。对照：`a.java:5470-5490`（5 处）覆盖 bump 臂 5475-5490 全部关键行 ✓ 无此问题。

## 面3 写面归因核 — 🟢 绿

**① `battle_enemy_hp` 全库 grep（实测 22 行）与声称 8 非写点逐条对**：

```
$ grep -rn "battle_enemy_hp" crates/ | wc -l
22
```

**A. resolver（battle.rs）内 14 行**——4 行是写/读点，10 行是 doc/字段/测试：

```
battle.rs:7/8     模块 doc 注释（写面归因说明）        非写点 ✓
battle.rs:38      `/// 一击交换（blow=true 时 battle_enemy_hp 已经 ctx 扣减）` doc   非写点 ✓
battle.rs:63      BattleCtx 字段声明 `pub battle_enemy_hp: &'a mut i32`            非写点 ✓
battle.rs:124     *ctx.battle_enemy_hp = ctx.enemy_base[idx].hp;   ← 写（bump 臂）
battle.rs:134     *ctx.battle_enemy_hp = ctx.enemies[idx].hp;      ← 写（route 臂）
battle.rs:170     *ctx.battle_enemy_hp -= dmg;                     ← 写（Exchange 减伤）
battle.rs:171     if *ctx.battle_enemy_hp > 0 {                    ← 读
battle.rs:260     测试构造 `battle_enemy_hp: &mut self.bh`          非写点 ✓
battle.rs:279/289 测试注释                                         非写点 ✓
battle.rs:285/295/360  测试断言（assert_eq! 读）                    非写点 ✓
```

**B. resolver 外 8 行 = commit 声称的 8 非写点，逐条实证**：

| 行 | 内容 | 判 |
|---|---|---|
| `scene.rs:348` | 字段声明 `pub battle_enemy_hp: i32` | 非写点 ✓ |
| `scene.rs:3661` | `self.battle_enemy_hp,`（undo snapshot 结构体读） | 非写点 ✓ |
| `scene.rs:3750` | `battle_enemy_hp: &mut self.battle_enemy_hp,`（battle_ctx 装配借用） | 非写点 ✓ |
| `scene.rs:3773` | 注释「写面=battle_enemy_hp 经 ctx 槽」 | 非写点 ✓ |
| `subsystem/mod.rs:46` | `WriteFaceRow { face:"BattleCtx", field:"battle_enemy_hp", … }` | 非写点 ✓ |
| `subsystem/mod.rs:237` | 测试构造 `battle_enemy_hp: &mut bh` | 非写点 ✓ |
| `scene_persist.rs:173` | `battle_enemy_hp: 0,`（persist init 初值） | 非写点 ✓ |
| `scene_tests/entity_route_tests.rs:106` | `assert_eq!(sc.battle_enemy_hp, …)`（测试读） | 非写点 ✓ |

**resolver 写点恰 3**（`battle.rs:124` bump 臂 / `:134` route 臂 / `:170` 减伤；`:171` 为读）——与 commit 声称「resolver 写 battle_enemy_hp 三事件」**逐条相符**。**resolver 外零真写点**（上表 22 行全列，无第 9 条写点）。

**② registry 43 行 Battle 10 面 vs BattleCtx 结构 vs resolver 实写字段对账**：

```
$ grep -c "    WriteFaceRow {" crates/game-core/src/subsystem/mod.rs
43
$ grep "    WriteFaceRow {" … | grep -o 'face: "[A-Za-z]*"' | sort | uniq -c
     10 face: "BattleCtx"      22 face: "DialogCtx"     1 face: "EntityCtx"
      4 face: "ItemCtx"         6 face: "ScriptCtx"     → 10+22+1+4+6 = 43 ✓
$ grep 'face: "BattleCtx"' …  → 恰 10 行：battle_active / battle_anim_tick / battle_digit_by_type /
  battle_enemy_hp / battle_finishing / battle_no_counter / battle_rerun_scene / battle_target /
  battle_tick / show_battle_digits  （与 BattleCtx 结构 battle.rs:60-69 的 10 个 &mut 字段同名同集 ✓）
```

| 分组 | 字段 | 谁写 |
|---|---|---|
| resolver 实写（4） | `battle_enemy_hp` / `battle_active` / `battle_no_counter` / `battle_target` | battle.rs:124-127、134-137、170 |
| 借而不用、Coordinator 直写（6） | `battle_anim_tick` / `battle_finishing` / `battle_rerun_scene` / `battle_tick` / `battle_digit_by_type` / `show_battle_digits` | scene.rs（如 `:3869 battle_tick+=1`、`:3872-3875 anim_tick`、`:3321 battle_digit_by_type` 直写） |

- 结论：**registry 行=写槽归属声明（批0 设计口径「每行=一个写槽」），非「当前实际写点」**；6 个借而不用槽与 plan 批10「直写清零」面一致，**无对账缺口**。
- `state_dump.rs` 对 `battle_enemy_hp` 走 `flds=&[]` 盲区（只 push `battleActiveFlag`）——与批0 盲区口径一致；`python3 tools/field_ownership.py --check` → `[GREEN] 归属表 186/186 字段唯一归属 exit=0`。

## 面4 predict 薄委托 — 🟢 绿

**① 4 个迁移面外 query 点零改（diff 实证）**：`git show 760a1bb -- crates/game-core/src/scene.rs | grep -E "^[+-].*predict_hp_loss\("` 结果**恰 3 行删除、0 行新增**，且 3 行全部是迁移面内（route `:2053`、bump `:3642`、tickBattle `:3756` 的 `let loss = …`）——

```
-                        let loss = self.predict_hp_loss(t, false);            ← route 迁移面
-                                let loss = self.predict_hp_loss(t, true);      ← bump 迁移面
-                                let loss = self.predict_hp_loss(t, self.battle_no_counter);  ← tickBattle 迁移面
```

**迁移面外 4 点（scene.rs:1020/2123/2473/3321）+ scene_paint.rs:349 + scene_tests:91/132 在 diff 中一行都不出现** ⇒ 签名/实参/语义零改（A 级：diff 行集）。签名仍是 `fn predict_hp_loss(&self, t: i32, no_counter: bool) -> i32`（diff 上下文行未变）。

**② math 单源=crate::combat 不动**：`scene.rs` 薄委托体只调 `crate::subsystem::battle::predict_engagement(...)`；`battle.rs:83-100 predict_engagement` 内部只调 `crate::combat::effective_attack_vs_type` + `crate::combat::predict_battle_hp_loss` —— `crate::combat` 在本 commit **零改动**（4 文件清单里没有 combat.rs）✓。

**③ 行为等价（逐行同构）**：旧体（diff 删除行）=`idx=(t-41)`、`atk=effective_attack_vs_type(&player, self.trait_flags(t))`、`predict_battle_hp_loss(atk, def, enemies[idx].hp/atk/def, no_counter)`；旧 `trait_flags(t)`=`enemy_base.get(t-41).map(trait_flags).unwrap_or(0)`。新 `predict_engagement`（battle.rs:90-99）**同 idx、同 trait_flags 取法、同 atk、同六参序** ⇒ 纯重构等价（A 级：源码逐行）。

## 面5 TDD + 变异 — 🟢 绿

**① TDD 红档实存**：`/tmp/ev_b2.txt` 48 行 3096B，含 21 错分类（8×E0433 `BattleEvent` / 7×E0223 / 3×E0425 `BattleResolver` / 1×E0599 `Refused` / 1×E0432 `unresolved import super::battle::BattleEvent` / 1×E0061 `takes 2 arguments but 3` / 2×`due to 21 previous errors`）。

**② 变异（独立做，非采信贴件）**：精确替换 `battle.rs:124` bump 臂 hp 源 `ctx.enemy_base[idx].hp` → `ctx.enemies[idx].hp`（`assert s.count(old)==1`）：

```
$ cargo test -p game-core --lib p2b2
test subsystem::battle::tests::p2b2_resolve_engage_modes ... FAILED
panicked at crates/game-core/src/subsystem/battle.rs:285:9:
assertion `left == right` failed: bump 面 battle_enemy_hp=基础表源
  left: 50
 right: 40
test result: FAILED. 2 passed; 1 failed; …   EXIT=101
```

→ **p2b2 必红且正是数值钉**（left 50/right 40 = 缩放表 vs 基础表）。**还原**（反向替换，count==1）→ `git diff --quiet` = `DIFF EMPTY`、`git status --porcelain -- crates/` 空、`p2b2 3 passed; 0 failed` ✓。

**③ 独立重跑**：

- `cargo test -p game-core --lib p2b2` → `test result: ok. 3 passed; 0 failed`（engage_modes / exchange_math / refused）✓
- `cargo test -p game-oracle --test fixture_frames` → **`test result: ok. 45 passed; 0 failed; 0 ignored`（125.58s，exit 0）** ✓（批2 判据「战斗夹具」）
- 金样 state-baseline（release）→ 见下「门禁独立重跑」。
- 总闸 `python3 gates/cli.py check` → 见下「门禁独立重跑」。

## 面6 夹带核 — 🟢 绿

```
$ git show 2a2f6c8 --name-only --format=""          →  crates/game-core/src/subsystem/entity_service.rs        （恰 1 文件）
$ git show 2a2f6c8 --numstat --format=""            →  4  2  crates/game-core/src/subsystem/entity_service.rs  （+4/-2 ✓）
$ git show 760a1bb --name-only --format=""          →  crates/game-core/src/scene.rs
                                                      crates/game-core/src/subsystem/battle.rs
                                                      crates/game-core/src/subsystem/mod.rs
                                                      docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md  （恰 4 文件）
$ git show 760a1bb --numstat --format=""
   142 121  crates/game-core/src/scene.rs
   358   6  crates/game-core/src/subsystem/battle.rs
     3   3  crates/game-core/src/subsystem/mod.rs
     1   0  docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md
   → +504 / -130 ✓ 与声称逐数相符
```

- 4 文件**全在声称域**：scene 战斗路径编排面 / subsystem battle 实现 / subsystem registry+签名锁 / spec 更正注记。**无夹带**。
- `mod.rs` 3/3 = 测试 `use` 行加 `BattleEvent` + `_sig_battle` 机器签名锁改 4 参（`pub fn _sig_battle<T: BattleSubsystem>(t: &mut T, ctx: &mut BattleCtx, x: EnemyTypeId, ev: BattleEvent) -> BattleOutcome { t.resolve_bump(ctx, x, ev) }`）——签名扩展的**必然伴随**（锁随形），在域内，非夹带。
- `2a2f6c8` 纯注释（头注释批0 措辞→批1 终形，原措辞以 `【历史轨迹】` 整句引文保留，`grep 历史轨迹`=1 命中、非历史轨迹行残留 `exit=1`），零行为改动；commit message 带 `Evidence` + `Confidence: 95` ✓。
- **Coordinator 遗留直写披露**：commit message 列「walk_phase/route_phase/entity state/player.hp/gold/popups/boss 钩子/undo/finishing/rerun/tick 直写（遗留面批10 清零）」——实读 scene.rs 逐项**属实**（如 `:3869 battle_tick+=1`、`:3804 player.hp-=player_dmg`、`:3820 gold+=`、`:3863 finishing=true`、`:2062 walk_phase=5` 等均为 Coordinator 直写），且 plan 批10 行确有「**直写清零收尾核**（scope=382 直写站点…）」授权 ⇒ **如实披露、面外遗留、非夹带**。唯一缺口=码内零注记（黄-轻 3，见总判）。

## 面7 TDD 真伪 — 🟢 绿

```
/tmp/ev_b2.txt        birth=2026-11-07 10:04:14  mtime=2026-11-07 10:15:27  48 行 3096B
commit 760a1bb        committed=authored=2026-11-07T10:16:10+08:00
⇒ 红档 mtime 早于 commit 43 秒 ✓（红先于码）
被改文件 mtime：battle.rs 09:45:38 / scene.rs 09:39:02 / mod.rs 09:32:12 / spec 09:38:03（均早于 commit ✓）
```

**贴件 `$(cat)` 风格核**：python 提取 commit message 自 `【Evidence 证据贴件】` 行后至 `Confidence:` 前的正文，与 `/tmp/ev_b2.txt` **全文 byte 级比对**：

```
EXACT MATCH (body vs ev): True
```

⇒ 非手粘（手粘几乎不可能 byte 级相等），`$(cat /tmp/ev_b2.txt) >> msg` 风格成立。

---

## 门禁独立重跑（本轮实测，2026-11-07 11:01–11:12）

**① 总闸**（`python3 -u gates/cli.py check`，13 门，输出落 `/tmp/gates_b2.log`）：

```
[✓ PASS] original-integrity / assets-manifest / reference-seed-integrity / knowledge-format /
         docs-links / naming-ledger / rust-tests（cargo test --release 全绿）/
         test-ratchet（total=988 基线 978 无回退）/ table-content-hash / table-evidence /
         state-machine-anchors（锚 6+边数对照全命中）/ asset-truth-manifest
[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同
== 汇总 ==
PASS=13 FAIL=0 SKIP=0
GATES_EXIT=0
```

→ **13/13 全绿、零 FAIL、零 SKIP**，含金样 **74 场景 sha 全同**（release 档，判据=批2「金样零重冻」）✓ 与贴件 ⑦⑧ 逐字一致。

**② 其余独立复跑**：

| 项 | 命令 | 实测 | 贴件声称 | 同 |
|---|---|---|---|---|
| TDD 单测 | `cargo test -p game-core --lib p2b2` | `3 passed; 0 failed` | 3/3 ok | ✓ |
| game-core lib | `cargo test -p game-core --lib` | `346 passed; 0 failed; 2 ignored` | 346/0/2 | ✓ |
| 战斗夹具 | `cargo test -p game-oracle --test fixture_frames` | `45 passed; 0 failed; 0 ignored`（125.58s，exit 0） | 45/45 | ✓ |
| 归属表 | `python3 tools/field_ownership.py --check` | `[GREEN] 186/186 字段唯一归属 exit=0` | 186/186 | ✓ |
| 工作树 | `git diff --quiet` / `git status --porcelain -- crates/` | `DIFF EMPTY` / 空 | 还原后空 | ✓ |

> 过程记录：本轮头两次门禁尝试因 `timeout 900/600` 短超时被杀（state-baseline 的 release 三测二进制约 10–14 min），**非门禁失败**；改 `timeout 3600 + python3 -u` 后一次跑完取得上述完整输出。


## 黄-轻登记（3 条，随批/批10 处理，不阻断）

1. **a.java 行锚过期**：`battle.rs:81/:116`（`9983-9982`）、`battle.rs:15/:133`（`8769`、`8760-8780`）、`scene.rs:3730`、`scene_tests/entity_route_tests.rs:106` —— HEAD 上分别落空行/地图格子/覆盖不全。根因=deobf `dc470b2`(+3)、`d389ddf`(+6) 行号前移未回填。**建议**：一次行锚勘误（9983-9982→9992-9996、8769→8775、8760-8780→8771-8787、10023-10149→10032-10105），纯注释 commit。
2. **route 注释区间漏敌先手行**（同上第 2 条，合并处理）。
3. **遗留直写码内零注记**：commit message 披露属实、plan 批10 有授权，但 `grep 遗留|批10` 在 scene.rs/battle.rs/mod.rs 零命中——建议在 `tick_battle`/route/bump 调用点的直写处补一行 `// 【遗留直写·批10 清零】` 注记，避免后续读者误以为批2 已迁完。

## 结论

**可推送。** 保真要害（两臂 hp 源/先手/门位、Exchange 4 拍数学、赏金守卫、boss 钩子、tick 位置）逐项与 HEAD a.java 实文同；写面归因与 registry 对账无缺口；predict 迁移面外 4 点零改、math 单源；TDD 红档与贴件真实性双向实证；变异独立复做判别力成立且已还原；夹带零。3 条黄-轻均为**注释/注记层**问题，建议随批或批10 勘误，不构成推送阻断。

Confidence: 92/100
