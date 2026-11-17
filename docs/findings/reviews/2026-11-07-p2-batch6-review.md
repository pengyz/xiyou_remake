# P2 批6 对抗复核报告 — commit 9c9cc43

> 被审件：`9c9cc43` refactor(core)「P2 批6 ItemService trait 化（pickup 分发链收拢）+HostCtx 摘 2+menu 盲区登记」（8 文件 +202/-44，`git show --stat` 实测），modern 分支
> 复核方式：对抗复核（只读；唯一写=本报告 + 任务授权的变异与还原）；证据带实测输出；纪律=AGENTS.md（保真=金样裁判）
> 日期：2026-11-07 · 被审 HEAD=`9c9cc43`（`git log --oneline -1` 复核实测）· 复核执行=独立重跑（变异独立复做、判据全部复跑）

## 总判：可推送（9 面 **0 红**；1 条黄-中=既有保真缺口（非本批引入、全门禁盲区，挂账）+ 3 条黄-轻，均不阻断本批）

| 面 | 裁定 | 一句话 |
|---|---|---|
| 1 pickup 分发链唯一（核心） | 🟢 绿 | kind/params/apply 生产调用点恰 2（resolver 唯一链 + 桥接纯查表重取），桥接重取不碰 params/apply/ctx ⇒「非分发」成立 |
| 2 A 级三锚对拍 | 🟡 黄 | pickupItemType/addItemToItemStack 逐臂全对、gainGold 数值+守卫同；**gainGold 的 kind4 浮层分支在 Rust 生产 2 调用点缺**（既有缺口，非本批引入，见挂账①） |
| 3 界收窄语义 | 🟢 绿 | script.rs:249/:270 → `impl HostCtx+EntityService+DialogSubsystem+ItemService`；`load_level_script`/`advance_dialog_page`/`open_dialog_page` 不动；4 实现者拆分零行为 |
| 4 HostCtx 余 8 处置账 | 🟢 绿 | 10→8 对账成立，逐条与 plan §1.2（21 项）归宿一致；附黄-轻②`script_host.rs` 处置账表未随批更新 |
| 5 盲区销账 | 🟢 绿 | MENU_BLIND_ZONES 恰 4 项=plan:70 点名；PLAN_NAMED_PENDING 真身 `script_host.rs` 余 1（tap:autoBattle→批7）；批5「menu 归批7/批9」的改正=如实；附黄-轻③行锚 :69→:70 |
| 6 TDD+变异+判别力 | 🟢 绿 | 红档 mtime 早于 commit；变异独立复做=恰 1 红、还原复绿；**判别力核**：同变异下 t=41 四测全绿（不红）/t=25 恰 1 红，且 kind(25)=Passive≠kind(26)=Key |
| 7 判据独立重跑 | 🟢 绿 | p2b6 4/4、game-core --lib **359/0/2i**、fixture_frames **45/45**、`gates cli.py check` **PASS=13 FAIL=0 SKIP=0**（含 state-baseline `[GREEN] 74 场景 sha 全同`、ratchet total=1001≥978）；Java 三方见「面7 补记」 |
| 8 死码删+sig_lock | 🟢 绿 | `set_script_delay` 全库仅剩 `scene.rs:496` 墓碑注释（零代码引用）；sig_lock 2 形对 5 实现者（resolver+4 host） |
| 9 夹带+修复实录 | 🟢 绿 | `git show 9c9cc43 --name-status` 恰 8 文件、全在 crates/（基线/reference/gates 零触碰）；修复实录四则可验项全对 |

### 挂账（不阻断本批，须入档）

1. **黄-中（既有保真缺口，非本批引入）**：Rust 生产侧 `gain_gold` 两处调用丢弃 `Option` ⇒ 不弹 kind4 金币浮层（Java `a.java:7530` 必弹）。真实脚本可达（`assets/raw/script:42`/`:45` = `GIN_1_1000`）。全门禁盲区：47 场景零 `GIN_1`、state dump 无 popup 字段、fixture_frames 覆盖矩阵无该脚本帧。**建议**：登记 `docs/bug-ledger.md`/移植缺口清单，并在批9（`scene.rs:2653 match (src,idx)` 商人面，`apply_merchant_offer` 所在）与 `op_gin`（`script_ops.rs:82`）一并补 `if let Some(v) = gain_gold(..) { popups.spawn(4, v, px, py, ..) }`。
2. **黄-轻②**：`crates/game-core/src/subsystem/script_host.rs:6-14` 处置账表未随批6 更新——仍写「HostCtx 现 10 方法」「give_gold/pickup_item→**批6 ItemService**」（待办形）与「:14 menu 4+tap 1=显式 pending 批7/批9 销账」（menu 已批6 销账）；`PLAN_NAMED_PENDING` 注（:103）同残留「批9 …menu 登记」。零行为，纯 doc 陈旧。
3. **黄-轻③**：行锚 off-by-one——「plan:69 点名 4 项」真身=**plan:70**（:69 是 dialog 行）。commit message、`item_service.rs:7` 注、任务 brief 三处同偏；语义内容全对，按批5 行锚先例不判红。

---

## 面1 pickup 分发链唯一性（核心 A 级） — 🟢 绿

### ① kind/params 查表生产调用点全库清点（第二分发链=红判据）

```text
$ grep -rn "pickup_kind_for_type\|pickup_params_for" --include="*.rs" crates/
crates/game-core/src/effect_registry.rs:88:  pub fn pickup_kind_for_type(t: i32) -> &'static str   （PICKUP_ARM_TABLE 纯 find）
crates/game-core/src/effect_registry.rs:147: pub fn pickup_params_for(t: i32) -> EffectParams      （同表）
crates/game-core/src/subsystem/item_service.rs:30: let kind = ...pickup_kind_for_type(t);   ← resolver 唯一链
crates/game-core/src/subsystem/item_service.rs:31: let params = ...pickup_params_for(t);
crates/game-core/src/scene.rs:4303: let kind = ...pickup_kind_for_type(t);                    ← 桥接重取（仅浮层判据）
```

- 生产调用点 **恰 2**（定义 2 + 生产 2），无第三处 ⇒ **无第二分发链**。
- `builtin_effect(` 生产调用点另 2 处均为**固定字面**（`scene.rs:882` `builtin_effect("Equip")`、`scene.rs:4351` `builtin_effect("Stack")` = P1-3 既有 `add_item_to_stack` 单源委托），**非 type→kind 查表**，不构成第二分发链。

### ② resolver 本体（item_service.rs:27-32，sed 实文）

```rust
impl ItemService for ItemResolver {
    /// 分发链唯一（禁第二分发链）：kind/params 表查+builtin_effect(kind).apply。
    fn pickup(&mut self, ctx: &mut ItemCtx, t: i32) -> ItemOutcome {
        let kind = crate::effect_registry::pickup_kind_for_type(t);
        let params = crate::effect_registry::pickup_params_for(t);
        crate::effect_registry::builtin_effect(kind).apply(ctx, &params)
    }
```

序 = kind→params→apply，**唯一 Service→Registry::lookup→ItemEffect 序** ✓。

### ③ 桥接段 kind 重取逐行读（scene.rs:4274-4310，sed 实文）

```rust
self.menu.set_overlay_kind_item(t);
let out = { let mut ctx = self.item_ctx(); let mut r = ItemResolver; r.pickup(&mut ctx, t) };
match out {
  Consumed { overlay_tier: Some(tier), hp_popup } => { spawn_hp_popup?; overlay_kind_pickup(t, tier); }
  Consumed { overlay_tier: None, hp_popup } => {
      spawn_hp_popup?;
      // 桥接策略读取（纯查表非分发——分发链唯一在 resolver）。
      let kind = crate::effect_registry::pickup_kind_for_type(t);
      if matches!(kind, "Stack" | "TutorialSeenStack" | "Passive") { /* idx: 85→20 / 86→21 / else t-13 → item_description → show_overlay_message(1,&text,0,3) */ }
      ...
```

- 重取值只喂 `matches!(kind, …)` 浮层**展示判据**：不进 `pickup_params_for`、不进 `builtin_effect(...).apply`、不碰 `ctx` ⇒ **「纯查表非分发」成立**。
- 重取与 resolver 用**同一函数同一 `t`** ⇒ 同值；批前旧体为「单 kind 变量两用」，新体两次查表等值 ⇒ **行为等价，非红**。
- `ItemOutcome` 其余臂：`NotConsumed`→`show_overlay_message(0,"你拥有更强力装备",0,3)`；`NoOp`→`{}`（与批前一致）。

---

## 面2 A 级三锚对拍 — 🟡 黄（主干全对；附黄-中既有缺口）

### ① `a.java:7534 pickupItemType` ↔ Rust 桥接（🟢 逐臂对）

sed 实文（canonical=`reference/src/deobf/a.java`）逐臂核：

| Java 分支 | Java 行为 | Rust 对应 | 裁定 |
|---|---|---|---|
| 13..=25,85,86（含 15 落穿） | `tutorialSeen=true`（仅 15）→ `addItemToItemStack` + `return` | Stack/TutorialSeenStack/Passive → `Consumed{None}` → 描述浮层 `show_overlay_message(1,…)`；15 的 tutorial 写在 effect 侧 | ✓ |
| 26/27/28 | 钥匙 `++` 后 `return`，**无浮层** | `KeyEffect` → `Consumed{None}`，kind=Key 不进 `matches!` 浮层臂 → 无浮层 | ✓ |
| 29/30 | `atk/def`：`floor≤10 ? 1 : floorTier` | `StatUp` 同参（P1-3 表行未动） | ✓ |
| 31/32 | `applyHpDelta(50/200 × floorTier)` | `HpUp` → `hp_popup: Some(d)` → 桥接 `spawn_hp_popup(d)` | ✓ |
| 33-40,79,80 | `tryEquip` 成功→`overlay(1, equipDescriptions[tier],0,3)`；失败→`overlay(0, miscTexts[5],0,3)` | `Equip` → `Consumed{Some(tier)}`→`overlay_kind_pickup(t,tier)`；`NotConsumed`→`show_overlay_message(0,"你拥有更强力装备",0,3)` | ✓ |
| 41-84 / default | `return`（no-op） | 表外 → kind="NoOp" → `NoOpEffect` → `ItemOutcome::NoOp` → 桥接 `{}` | ✓ |

### ② `a.java:7727 addItemToItemStack` ↔ `StackEffect`（🟢）

- Java：`findItemStackIndex` / `itemTypeToStackIndex`（`:7720 default: var1 = var0 - 13;`）→ 空栈 push + `itemUseCounts[idx]` / 叠加 `+1` → `itemUseTargetType=(byte)var1; showOverlayMessage(1, itemDescriptions[idx], 0, 3)`。
- Rust：`effect_registry.rs:182-215 StackEffect::apply` —— idx `85→20 / 86→21 / else t-13`、`ITEM_USES[idx]`、`find_item` 空栈 push/叠加、返回 `Consumed{None,None}` → 桥接浮层 1 + 同描述文本。**数值/分支逐项同**（`effect_registry` 本批零改动，P1-3 既有等价三层承接）。

### ③ `a.java:7527 gainGold` ↔ Rust 给金（数值/守卫同；**浮层分支缺=黄-中**）

Java（sed 实文）：

```java
7527: private void gainGold(int var1, int var2, int var3) {
7528:    this.goldAmount += var1;
7529:    if (var1 > 0) {
7530:       this.spawnPopup((byte)4, var1, var2, var3);
```

Rust（`crates/game-core/src/combat.rs:190-198`）：

```rust
/// gainGold 忠实移植（a.java:7503-7503）：gold += n；n>0 时 popup kind4。
/// 返回 popup 值（n>0 时 Some(n)）。
pub fn gain_gold(player: &mut PlayerCombat, amount: i32) -> Option<i32> {
    player.gold += amount;
    if amount > 0 { Some(amount) } else { None }
}
```

- **数值** `gold += amount` ✓；**守卫** `amount>0` ✓（返回 `Some`）——守卫语义被保留为返回值，**popup 责任在调用方**。
- **但生产两处调用方丢弃 `Option`（不弹 kind4）**：

| # | Rust 调用点 | Java 对应 | 弹浮层？ |
|---|---|---|---|
| 1 | `crates/game-core/src/script_ops.rs:82` `op_gin` kind=1：`crate::combat::gain_gold(&mut self.player, value);` | `a.java:8439 gainGold(var48, playerPixelX, playerPixelY)` | Java 弹 / **Rust 不弹** |
| 2 | `crates/game-core/src/scene.rs:2661-2663` 商人 `(0, 1\|15\|21) => { gain_gold(&mut self.player, 1000); }` | `a.java:7336 gainGold(1000, playerPixelX, playerPixelY)` | Java 弹 / **Rust 不弹** |

- 对照：**战斗两站点弹**（`scene.rs:2322-2330`、`scene.rs:3836-3845` = `gold += gold; if gold > 0 { popups.spawn(4, gold, …) }`），与批2 复核「赏金 popup 守卫逐行同」一致 ⇒ 缺口**只在 GIN/赠金两处**。
- **可达性（A 级）**：`assets/raw/script:42` `26 TAK_129_132 DES_84_11_4 GIN_1_1000`、`:45` `29 TAK_145_146 DES_84_9_8 GIN_1_1000`（原始脚本，测试夹具 `crates/game-core/tests/fixtures/level_scripts.txt:27/:30` 同文）⇒ 真实关卡脚本会走 `GIN_1`。
- **门禁盲区（复核实测）**：① `grep -rln "GIN_1" reference/oracle/scenarios/` = **0**（47 场景无 `GIN_1`，Java 三方不触发）；② `tools/state_baseline.py` 与 `data/status/baselines/state-dump-golden.json` 无 `popup` 字段（74 场景金样不覆盖）；③ `fixture_frames.rs` 覆盖矩阵（preset-floor5/49、fx-scriptwalk-floor3、fx-knockback-floor43、fx-fan-floor13 等）无该脚本帧；④ `grep "popupKind" crates/` 仅 `popup.rs:36` 注释 ⇒ Rust 侧 trace 不发 popup 字段。
- **定色理由（为何黄不判红）**：该两行**不在 9c9cc43 diff 内**（`git show 9c9cc43 -- crates/game-core/src/scene.rs | grep '^@@'` = `@@ -24 / -492 / -4257 / -4274 / -4291`，无 2661；`script_ops.rs` 不在 8 文件内），`combat::gain_gold` 亦未改；批6 只迁 `give_gold` 签名，`Floor1Host::give_gold` 体逐字搬（见面3）⇒ **既有缺口，非本批引入**，按 AGENTS §1「保真期行为只增不减」须登记修复，但不构成本批红。

---

## 面3 界收窄语义 — 🟢 绿

### ① diff hunk 与两 fn 的 impl 界

```text
$ git show 9c9cc43 -- crates/game-core/src/script.rs | grep '^@@'
@@ -9,6 +9,7 @@        （+ use ItemService）
@@ -133,8 +134,7 @@   （Default impl 尾）
@@ -246,7 +246,7 @@   （step 界：+ ItemService）
@@ -267,7 +267,7 @@   （execute_script_instruction 界：+ ItemService）
@@ -525,8 +525,6 @@   （set_script_delay 死码删）
@@ -540,6 +538,13 @@ （NoopHost 拆 impl ItemService）
```

- `:193 load_level_script` = `impl HostCtx + EntityService`、`:210 advance_dialog_page` / `:216 open_dialog_page` = `impl HostCtx + DialogSubsystem` —— **三处均不在 hunk 内 ⇒ 未动** ✓（体内只调 `locate_speaker`/`snap_camera`/`set_typewriter`，无 GIN 内部调用）。
- `:249 step` / `:270 execute_script_instruction` = **`impl HostCtx + EntityService + DialogSubsystem + ItemService`**（四 trait，任务声称的界收窄）。
- **GIN 消费面 grep**：`grep -n "host.pickup_item\|host.give_gold" script.rs` → 仅 `:417 0 => host.pickup_item(v)`、`:418 1 => host.give_gold(v)`，且位于 `execute_script_instruction` 内 ⇒ 界收窄**恰好覆盖唯一消费面**，不外溢。
- `exit_application`/`mvs_exchange` 消费面仍仅 `script.rs:496/507`（批5 实测同）。

### ② 4 实现者拆分零行为

| 实现者 | 位置 | 批6 变化 | 行为 |
|---|---|---|---|
| NoopHost | `script.rs:541-547` | 新增 `impl ItemService`：pickup→`ItemOutcome::NoOp`、give_gold/pickup_item→`{}` | 批前同为 no-op ⇒ 零变化 |
| StubHost | `script_golden.rs:73-79` | 新增 impl，保留 `gold_calls.push` / `pickup_calls.push` | 记录器保留（`:276` 断言 `gold_calls == vec![1000]`） |
| ReplayHost | `l2_prologue.rs:52-58` | 从 `impl HostCtx` 摘出 2 方法 → 新 impl；give_gold/pickup_item 仍 `let _ = …`（原 `{}` 形） | 零变化 |
| Floor1Host | `walk_replay_floor1.rs:118-124` | `give_gold` 体 `let _ = game_core::combat::gain_gold(&mut self.player, amount);` **逐字搬**（diff 可见原体删除行与新体插入行完全相同） | 零变化 |

- `impl ItemService for` 全库 **5** 处（resolver + 4 host）；3 个 host 的 `pickup` 均返回 `ItemOutcome::NoOp`（无副作用）✓。

---

## 面4 HostCtx 余 8 处置账 — 🟢 绿（附黄-轻②）

- **实测 8 fn**：`awk '/trait HostCtx/,/^}/' script.rs | grep "fn "` → `find_path / snap_camera_to_player / locate_speaker_entity / ces_camera / set_camera_anchor_target / exit_application / mvs_exchange / snap_camera_after_ros1`（**恰 8**）。
- **对账链**：批5 处置账「余 10」（give_gold/pickup_item→批6 + 其余 8）→ 本批摘 2 ⇒ **余 8** ✓；`script.rs:134-136` doc 注实文「批1：…余 12 暂留。批3：…余 10 暂留。批6 摘 2（give_gold/pickup_item 已逐字迁 …HostCtx 余 8）」✓ 已随批更新。
- **逐条 vs plan §1.2（plan:41-52，21 项 = 实体 10 + 道具 2 + 相机 4 + 对话 2 + 系统 3）**：

| 余 8 方法 | 归宿（plan §1.2 / 批5 实测） | 裁定 |
|---|---|---|
| find_path | pathfind 域独立（plan「find_path 留 pathfind 域」） | ✓ 不漏不错派 |
| snap_camera_to_player / ces_camera / set_camera_anchor_target / snap_camera_after_ros1 | Coordinator 相机 4（不入 trait 面） | ✓ |
| locate_speaker_entity | 批3 dialog 面已注（plan 系统面 3 之一） | ✓ |
| exit_application / mvs_exchange | 平台边界 stub 留 HostCtx（仅 ScriptEngine 边界消费 `script.rs:496/507`） | ✓ |
| （摘除）give_gold / pickup_item | 批6 ItemService | ✓ 本批销账 |

- **无漏项、无错派**（漏项=黄、错派=红 ⇒ 本面绿）。
- **黄-轻②**：`subsystem/script_host.rs:6-14` 处置账表仍为批5 终形（「HostCtx 现 **10** 方法——本批无再收窄面」「give_gold/pickup_item→**批6 ItemService**」待办形；`:14`「menu 4+tap 1=显式 pending 批7/批9 销账」已过期）；`PLAN_NAMED_PENDING` 注（`:103`）残留「批7 …+批9 …menu 登记」。**零行为、纯 doc 陈旧**，与 `script.rs` 已更新的 doc 形成不一致 ⇒ 建议下批顺手同步。

---

## 面5 盲区销账 — 🟢 绿（附黄-轻③行锚）

### ① MENU_BLIND_ZONES 恰 4 项（item_service.rs:41-46，sed 实文）

```rust
pub const MENU_BLIND_ZONES: &[&str] = &[
    "menuCursor（MenuState 菜单游标）：菜单导航写面——dump 盲区（state_dump 无 menuCursor）",
    "itemCursor（MenuState 道具面板游标）：item 面板选择写面——dump 盲区",
    "recordPage（MenuState 记录页）：open_record_page 翻页写面——dump 盲区",
    "difficultyIndex（MenuState 难度档）：op_res easy 判据+存档头写面——dump 盲区（state_dump 无 difficultyIndex）",
];
```

- **plan 点名机器断言**：`plan:70 menu（menuCursor/itemCursor/recordPage/difficultyIndex）` ↔ `p2b6_menu_blind_zones_cover_plan_named` 对 4 名逐个 `assert!(m.contains(name))` ✓。
- **黄-轻③**：任务 brief / commit message 写「plan:69 点名 4 项」，`sed -n '69p'` 实文=`dialog（…）`，menu 行=**plan:70** ⇒ 行锚 off-by-one（语义全对，批5 行锚先例不判红）。

### ② PLAN_NAMED_PENDING 真身与批5 注记改正（如实）

- 真身 = `crates/game-core/src/subsystem/script_host.rs:105-107`：

```rust
pub const PLAN_NAMED_PENDING: &[&str] = &[
    "tap:autoBattle（批7 InputMapper 登记）",
];
```

- `:193` 断言 `assert_eq!(PLAN_NAMED_PENDING.len(), 1, "pending=tap1（menu4 已批6 销账）")`；`p2b6` 断言 `!pend.contains("menu:")` + `pend.contains("autoBattle")` ⇒ **machine 双钉** ✓。
- **批5 注记改正=如实**：批5 message 实文「PLAN_NAMED_PENDING=menu 4+tap 1 显式 pending（批7/批9 销账）」；plan:94 批6 行明文含「**menu 盲区登记**」，plan:95 批7 行含「tap 盲区登记（autoBattle）」，plan:98 批9 行=ShopGoods（**无 menu**）⇒ 批5 把 menu 记到批7/批9 属错派，批6 将 menu 就地销账、留 `tap:autoBattle` 给批7 = **按 plan 修正、如实** ✓。
- autoBattle 归批7 ✓（plan:95「tap 盲区登记（autoBattle）」）。

---

## 面6 TDD+变异+判别力 — 🟢 绿

### ① 红档 mtime < commit

```text
/tmp/ev_b6_red.txt   mtime=2026-11-07 18:29:45   < commit 9c9cc43 2026-11-07 19:23:37 ✓
/tmp/is_bak.rs        mtime=2026-11-07 18:33:50   ✓（变异前备份）
红档首行：error: unknown start of token: \u{b6}   （吞字 U+00B6）
另含：error[E0425]: cannot find type `ItemCtx` / cannot find value `ItemResolver` /
      cannot find value `MENU_BLIND_ZONES` / cannot find value `PLAN_NAMED_PENDING` /
      error[E0599]: no method named `give_gold` / `pickup_item` …（共 14 errors）
```

### ② 变异独立复做（复核员亲手，已还原）

- **M1**：`item_service.rs:30` → `pickup_kind_for_type(t + 1) //MUT` ⇒ `cargo test -p game-core --lib p2b6` = **3 passed; 1 failed**，恰 `p2b6_pickup_dispatch_single_source` 红（`left "NoOp" / right "Consumed { overlay_tier: None, hp_popup: None }"`）。
- **还原**：`git checkout -- crates/game-core/src/subsystem/item_service.rs` ⇒ `git status --porcelain crates/` **空**、复跑 **4/4 绿**。

### ③ 判别力核（任务点名）

- 同一变异下把夹具 `t=25` 改回首版 `t=41`（三处）⇒ **4 passed 全绿（变异不红）** ⇒ 独立证明首版夹具零判别力、t=25 修复成立。
- **异 kind 独立验证**：`PICKUP_ARM_TABLE` 实文 `(25, "Passive", …)` vs `(26, "Key", …)` ⇒ `kind(25) ≠ kind(26)` ✓（26/27/28=钥匙族，25=Passive；`t+1` 变异必然换 kind，25 位可捕获、41 位同 kind 不可捕获）。

### ④ p2b6 四测（item_service.rs tests，sed 实文）

`p2b6_pickup_dispatch_single_source`（Service 分发=Registry 链唯一源）、`p2b6_registry_lookup_side_effect_free`（kind/params 幂等 ×5 类型）、`p2b6_menu_blind_zones_cover_plan_named`（4 项 + pending 双断言）、`p2b6_boundary_method_shapes`（`fn _g<T: ItemService>` / `fn _p<T: ItemService>` 泛型签名锁）。

---

## 面7 判据独立重跑 — 🟢 绿

| 判据 | 命令 | 实测结果 |
|---|---|---|
| p2b6 四测 | `cargo test -p game-core --lib p2b6` | `4 passed; 0 failed` ✓ |
| game-core 全测 | `cargo test -p game-core --lib` | **`359 passed; 0 failed; 2 ignored`**（11.17s）✓ |
| 帧夹具 | `cargo test -p game-oracle --test fixture_frames` | **`45 passed; 0 failed`**（94.10s）✓ |
| 总闸 | `python3 gates/cli.py check` | **`PASS=13 FAIL=0 SKIP=0`**，含 `state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同`、`rust-tests — cargo test --release 全绿`、`test-ratchet — total=1001(基线978) 无回退` ✓ |

（Java 三方 scenario-suite 见下方「面7 补记」。）

---

## 面8 死码删 + sig_lock — 🟢 绿

- **set_script_delay**：`grep -rn "set_script_delay" crates/` = **仅** `crates/game-core/src/scene.rs:496`「【批6 死码删】set_script_delay 已删（批5 CES 写面收拢后零引用）」墓碑注释 ⇒ **零代码引用** ✓。
- **sig_lock 2 形**：`subsystem/mod.rs:210-219` 两 generic（`#[allow(dead_code)]`）：

```rust
pub fn _sig_item_give_gold<T: ItemService>(t: &mut T, amount: i32) { t.give_gold(amount); }
pub fn _sig_item_pickup_item<T: ItemService>(t: &mut T, item_id: i32) { t.pickup_item(item_id); }
```

- 覆盖面：`impl ItemService for` **5** 处（ItemResolver + NoopHost/StubHost/ReplayHost/Floor1Host）——即「3 实现者 + Noop + resolver」全被泛型签名锁编译锁住 ✓（与同型 `entity_service` 的 `_sig_entity_*` 形制一致）。

---

## 面9 夹带 + 修复实录 — 🟢 绿

### ① 夹带核（恰 8 文件）

```text
$ git show 9c9cc43 --name-status
M crates/game-core/src/scene.rs
M crates/game-core/src/script.rs
M crates/game-core/src/subsystem/item_service.rs
M crates/game-core/src/subsystem/mod.rs
M crates/game-core/src/subsystem/script_host.rs
M crates/game-core/tests/script_golden.rs
M crates/game-oracle/tests/l2_prologue.rs
M crates/game-oracle/tests/walk_replay_floor1.rs
```

- **恰 8**，全部在 `crates/`（core/oracle）；**基线 `data/status/`、`reference/`、`gates/`、`original/` 零触碰** ✓；HEAD=`9c9cc43`。

### ② 修复实录四则可验项

| # | 声称 | 复核 |
|---|---|---|
| ① 吞字 U+00B6 | 红档首行 `error: unknown start of token: \u{b6}` | ✓ `grep -rn $'\xb6' crates/` = 0（代码零残留） |
| ② PLAN_NAMED_PENDING 真身 | 「定位错 mod.rs→真身 script_host.rs」 | ✓ 真身在 `script_host.rs:105`；红档含 `cannot find value PLAN_NAMED_PENDING`（「曾误记 mod.rs」为 message 自述，无独立件，如实记为自述级） |
| ③ 桥接 scope E0425 | 「kind scope→纯查表重取」 | ✓ 红档含 `cannot find type ItemCtx` / `cannot find value ItemResolver`；现 `scene.rs:4303` 重取（见面1） |
| ④ 短名消费 | `let mut r = ItemResolver; r.pickup(&mut ctx, t)` | ✓ `scene.rs:4285` 实文短名消费（非全路径） |
| （判别力） | t=41 不红→t=25 | ✓ 面6 ③ 独立复做成立 |

### ③ 附带核

- 变异还原后工作树 `crates/` 干净（`git status --porcelain crates/` 空）；本报告为唯一新增写入。
- 既有未跟踪件（`docs/findings/reviews/2026-11-0*.md` 等）为历史复核报告，非本批夹带。

---

## 面7 补记（Java 三方 scenario-suite）

`python3 reference/oracle/run.py --scenarios`（复核员独立重跑，第二裁判 A==B==C，2026-11-07 20:08 起）：

```text
[oracle] == --scenarios 结果：PASS（47/47 场景）==
[oracle] 4/4 报告 → /home/peng/workspace/xiyou_remake/reference/oracle/_diff/scenarios.md
（抽样：prologue-dialog 9800 ticks A==B==C PASS sha=50f9c399…、prologue-patient 21000 ticks PASS、
 walk-pure 4000 ticks PASS、preset-save 700 ticks PASS 等，逐场景 sha 全同）
```

⇒ **47/47 PASS**，面7 补记绿。（注：47 场景零 `GIN_1`，故本套不覆盖挂账①的浮层缺口，见面2。）

---

## 总判与 Confidence

**总判：可推送。**

- 9 面 **0 红**；核心 A 级面（pickup 分发链唯一、三锚主干、界收窄、处置账、TDD/变异、判据重跑）全绿。
- 挂账 4 条（1 黄-中 + 3 黄-轻）**均非本批引入**、零行为或既有保真缺口，不构成本批阻断；其中黄-中（GIN/赠金金币浮层缺口）须按 AGENTS §1/§4 登记并排批修复（建议批9 商人面 + `op_gin` 一并）。

Confidence: 88/100
