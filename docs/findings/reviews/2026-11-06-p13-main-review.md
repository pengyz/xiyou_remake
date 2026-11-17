# P1-3 主件复核（fb1ddc6）——pickup_item_type 23 臂 match 删尽+表行/注册分发同 commit

> 独立对抗复核员 · 逐提交纪律 / 差分验证员纪律（AGENTS.md 权威）
> 对象：`fb1ddc6`（未推送，绿=lead 推送）；前情：硬门已闭环（e9d98ed）
> 复核时间：2026-11-06

## 0. 复核对象与命令基线

- **对象**：`fb1ddc60989b481f626f5f39b27f831d30653a85`（复核时 **HEAD=fb1ddc6**，`git diff --stat HEAD` 空=工作树与提交一致；**未推送**）。
- **面**：`git show --name-only fb1ddc6` = 恰 2 文件（`crates/game-core/src/scene.rs` + `crates/game-core/src/effect_registry.rs`）；`--stat` = `2 files changed, 218 insertions(+), 119 deletions(-)`，与申报逐字同。
- **"23 臂"口径复核**（沿用 P1-2 复核既定口径，docs/findings/reviews/2026-11-05-p12-effect-review.md:132）：旧 `pickup_item_type` 函数体**含 `=>` 的行数**=23、`=>` 出现 31 次（含 4 组内层 tier match）——我用 `git show fb1ddc6^` 对拍实测 **23 / 31 同数** ✓（非 case 标签数：Java case 面实为 32 型）。
- **复核命令基线**（全部在 HEAD 实跑，2026-11-06；非引 commit 贴件）：

| # | 命令 | 实测输出 |
|---|---|---|
| 1 | `cargo test -p game-core --lib p1a` | `14 passed; 0 failed; 0 ignored; 322 filtered out` |
| 2 | `cargo test -p game-core` | `334 passed; 0 failed; 2 ignored` |
| 3 | `cargo test -p game-oracle --test fixture_frames` | `45 passed; 0 failed`（118.77s） |
| 4 | `cargo test -p game-oracle --test arm_coverage -- --nocapture` | `1 passed`；`21 全账：夹具 20 + 正名 1 = 21` |
| 5 | `python3 tools/state_baseline.py --check` | `[GREEN] 金样逐字节等价：74 场景 sha 全同` |
| 6 | `python3 gates/cli.py status` | `PASS=13 FAIL=0 SKIP=1 TOLERATED=0`（SKIP=visual-diff 在册） |
| 7 | `cargo build -p game-core` | 4 warning（拆解见攻击点①） |
| 8 | `grep -c "match t" …` / `git show --stat` | 见攻击点①/⑥ |

## 1. 攻击点①：禁双轨核（核心红线）

### 子检 1 — pickup 区 `match t` = 0（🟢）

- `sed -n '4236,4302p' crates/game-core/src/scene.rs | grep -c "match t"` = **0**；该区（`pickup_item_type` 4236 起）仅存 `match out`（outcome→浮层桥接，按**结果**分派非按 t 分派）与 `matches!(kind, "Stack" | "TutorialSeenStack" | "Passive")`（kind 门）。**按 t 的行为分发已单轨化**：`let kind = pickup_kind_for_type(t); let params = pickup_params_for(t); … builtin_effect(kind).apply(&mut ctx, &params)`（scene.rs:4243-4261，唯一分发点）。
- 全文精确口径（排除 `match timeline` / `match transform` 伪命中）`grep -rn "match t[ {]"`：

| 文件:行 | 数 | 归属 | 面 |
|---|---|---|---|
| scene.rs:3445 / 3456 / 3485 | 3 | 门（entity_category=1 → 型1\|2\|3、door_code、缺钥文案） | 非 pickup |
| scene.rs:3753 | 1 | `trait_flags`（49/53/74→1、69→2 脚本旗） | 非 pickup |
| scene.rs:3834 | 1 | boss 档 / `script_line_flags` | 非 pickup |
| scene.rs:4093 | 1 | bob（型6/83 出生踩格） | 非 pickup |
| **scene.rs 小计** | **6** | — | **与 commit 贴件"6"同数 ✓** |
| effect_registry.rs:89 / 104 | 2 | `pickup_kind_for_type` / `pickup_params_for` | **新分发表本体**（kind、params 同表两列，非残留） |
| scene_menu.rs:276 | 1 | `tick_item_panel`（use 面 13/14/15/23/24/25 分组） | 非 pickup（道具栏**使用**面） |

⇒ **结论：pickup 行为分发单轨，6 处残留全非 pickup 面（门3/脚本旗2/bob1），逐处列证如上 ✓。**

### 子检 2 — 表行/注册分发同 commit（🟢）

`git show --name-only fb1ddc6` = 恰 `scene.rs` + `effect_registry.rs` 两文件，**表行（pickup_kind_for_type/pickup_params_for）、注册（builtin_effect 桥接）、桥接代码同笔落地**，无"先切分发、后补表行"的中间态可检出（313375b 禁双轨先例口径）✓。

### 子检 3 — 旧臂死码残留（🟡）

旧 `match t` 六分支函数体**已全删**（`git show fb1ddc6^` 对拍：`13..=25|85|86` / `26` / `27` / `28` / `29` / `30` / `31` / `32` / `33..=36|79` / `37..=40|80` / `_` 逐臂消失，零残留臂）——**但委托化遗留 2 处**：

1. **`GameScene::m_083_index`（scene.rs:886）成死码并触发新编译警告**：父提交 2 调用（旧 `try_equip` 体内 867/875）→ 本提交 **0 调用** ⇒ `cargo build` 4 warning 中 **1 条为本 commit 新增**（`warning: method m_083_index is never used`）。另 3 条为父提交已有（scene_dialog.rs:124 `text_len`、scene.rs:492 `set_undo`、scene_paint.rs:34 `add_ease_target_y`——三者父提交同样零调用方，`git grep` 复核）。更实质：它与活源 `effect_registry::m_083_index`（effect_registry.rs:289）**双实现并存**，恰是本 commit 自称要清的"同 id 双源"。
2. **`GameScene::floor_tier`（scene.rs:4230）成孤儿**：父提交 4 调用（4262/4265/4267/4268，**全部位于被删臂内**）→ 本提交 `grep -rn "\.floor_tier()" crates/` = **0**。`pub` 故不报警告，但已无任何调用面；commit 称"**本方法=薄委托，调用面零改动**"——调用点在**同一 commit 内被删除**，该表述与事实不符（应为"调用面随臂删除归零"）。

**① 判：🟡 黄**（核心红线=🟢：分发单轨、两文件同笔、旧臂零残留；黄=委托化死码 2 处——其一为本 commit 新增编译警告+双实现残留，其一使 commit 表述失实）

## 2. 攻击点②：逐臂等价（23 臂映射表 + 抽 5 臂深核）

### 2.1 23 臂 → 注册行映射表（对拍 a.java:7534-7649 sed 实文）

> 实文锚实测（`grep -n` 于 `reference/src/deobf/a.java`）：`pickupItemType`=**7534**、`switch(var1)`=**7537**、`case 15`=**7538**、`case 26`=**7556**、`case 29`=**7565**、`case 31`=**7581**、`default:`=**7630** + `return;`=**7631**、`case 33`=**7632**、`equipDescriptions[var2]`=**7644**、`miscTexts[5]`=**7646**、switch 收=**7648**、`floorTier()`=**7651**、`applyHpDelta()`=**5172**、`tryEquip()`=**7677**、`equipTierIndex()`=**7699**、`itemTypeToStackIndex()`=**7711**、`addItemToItemStack()`=**7727**、`itemUseCounts`=**944**、`equipDescriptions` 表=**930**、`equipTierTypes`=**953**、`equipTierBonuses`=**952**。

| 旧 Rust 臂（`=>` 行） | 类型面 | Java case 锚 | kind（pickup_kind_for_type） | params（pickup_params_for） | outcome → 桥接 |
|---|---|---|---|---|---|
| 1 | 13..=25 \| 85 \| 86（15 前置 tutorial） | 7538-7555 | 15=`TutorialSeenStack`；13,14,16..21=`Stack`；22..25=`Passive` | `Stack{15, tutorial:true}` / `Stack{t,false}` | `Consumed{None,None}` → 描述浮层 kind1（idx=t-13 / 85→20 / 86→21） |
| 2 | 26 | 7556-7564 | `Key` | `Key{yellow}` | `Consumed{None,None}` → 零浮层 |
| 3 | 27 | 〃 | `Key` | `Key{red}` | 零浮层 |
| 4 | 28 | 〃 | `Key` | `Key{blue}` | 零浮层 |
| 5 | 29 | 7565-7572 | `StatUp` | `StatUp{atk:1, def:0}` | floor≤10 +1，否则 ×floor_tier |
| 6 | 30 | 7573-7580 | `StatUp` | `StatUp{atk:0, def:1}` | 同上（def 面） |
| 7 | 31 | 7581-7583 | `HpUp` | `HpUp{delta:50}` | `hp_popup=Some(50×tier)` → `spawn_hp_popup` |
| 8 | 32 | 7584-7587（后落 41+ 组 return） | `HpUp` | `HpUp{delta:200}` | 同上 |
| 9 | 33..=36 \| 79 | 7632-7635, 7640 | `Equip` | tier = 1/2/3/5/**4**（force:false） | `Consumed{Some(tier)}`→`D[tier]`；`NotConsumed`→强装备提示 |
| 10 | 37..=40 \| 80 | 7636-7639, 7641 | `Equip` | tier = 7/8/9/11/10 | 同上 |
| 11 | `_`（41+ 怪物/不可达） | 7625-7631 default | `NoOp` | 占位 `Stack{t,false}`（NoOp 吞） | `NoOp` → 零动（=旧 `_ => {}`） |

> 注：上表 11 行=**语义臂**；"23 臂"=旧函数体含 `=>` 的**行**数口径（含 4 组内层 tier match 的 12 行），见 §0。

### 2.2 抽 5 臂深核（对拍原臂语义）

**① 13..=25 区间（栈增族）🟢**
- Java：`case 15: tutorialSeen=true;` **落穿**到 13/14/16..25/85/86 共同体 → `addItemToItemStack(var1)`（7538-7555）。
- 新：`kind(15)=TutorialSeenStack` → `builtin_effect` 归一到 `StackEffect`（硬门2）+ `params(15)=Stack{tutorial:true}` → `if *tutorial { *ctx.tutorial_seen = true }` 后栈增 ✓；22..25 kind=`Passive` 同 `StackEffect`（params `tutorial:false`）✓。
- 栈增三件逐字：`ITEM_USES`（effect_registry.rs:174）= `a.java:944` `{-1,-1,-1,-1,1,1,1,3,1,1,-1,-1,-1,1×10}` **全同**；`idx` = `itemTypeToStackIndex`（a.java:7711-7725：85→20、86→21、default `var0-13`）——StackEffect 内与桥接分支**两处同式** ✓；浮层 `showOverlayMessage(1, itemDescriptions[idx],0,3)` + `itemUseTargetType=(byte)var1`（7727-7740）= 桥接分支 `item_description(idx)`+`show_overlay_message(1,…)`，`set_overlay_kind_item(t)` 在函数头（既有写，等值）✓。

**② 26-28 三色钥匙 🟢**
- Java 7556-7564：`yellowKeyCount++` / `redKeyCount++` / `blueKeyCount++` 三 case 各自 return。
- `KeyEffect` 三布尔独立 `+=1`，params 26={y:1} / 27={r:1} / 28={b:1} 与旧臂字面同 ✓；outcome `Consumed{None,None}` 且 kind 不在桥接三族内 → 零浮层 ✓（旧臂亦零浮层）。

**③ 29-30 floorTier 分支 🟢**
- Java 7565-7580：`if (currentFloor<=10) playerAtk++; else playerAtk += floorTier();`（30 同构于 def）。
- `StatUpEffect`（effect_registry.rs:214-228）：`if ctx.floor <= 10 { (atk, def) } else { f = floor_tier(ctx.floor); (atk*f, def*f) }`。29=(1,0) ⇒ floor>10 时 `da = 1*f = f` ≡ 旧臂 `atk += floor_tier()`（**乘法 vs 加法在单位 1 下恒等**）；30=(0,1) ⇒ `dd = f` ✓。`floor_tier` 独立域函数（effect_registry.rs:130-137）与 a.java:7651-7662 逐字同（`(f-1)/10`、`t<0→0`、`f>50→0`、`+1`）✓。

**④ 31-32 hp（含红修面）🟡**
- Java 7581-7587：`var2 = floorTier(); applyHpDelta(50*var2)`（32=200）。新：`d = delta * floor_tier(ctx.floor)` ✓ 同式。
- hp 前半 = `applyHpDelta`（a.java:5173-5177 / scene.rs:4387-4392）逐字：`hp <= -d → hp = 1`，否则 `hp += d` ✓。
- popup 面见攻击点③（**黄**：分支条件未复刻 `delta = hp-1` 归零）。

**⑤ 33-40 装备（10 型）🟡**
- Java 7632-7647 单一 case 组：`var2 = equipTierIndex(var1,0,12)`（7699-7709 倒序扫）→ `tryEquip(var2,false)` → 成功 `showOverlayMessage(1, equipDescriptions[var2],0,3)`，失败 `miscTexts[5]`。
- **tier 映射手推**（对 `equipTierTypes=[0,33,34,35,79,36,0,37,38,39,80,40]`，a.java:953 倒序扫）：33→1、34→2、35→3、79→4、36→5、37→7、38→8、39→9、80→10、40→11 —— 与 `pickup_params_for` 十行**逐型全同** ✓（effect_registry.rs:114-123）。
- 行为体 = `EquipEffect` = `tryEquip` 完整体（force 并入，见④）；`D[tier]`（scene.rs:4364-4377）与 `equipDescriptions`（a.java:930-945）**逐项同文** ✓（含 index 0/6 空串）。
- **🟡 发现（未披露行为变更）**：`t=79` 的浮层 tier
  - 父提交：`let eq_tier = if t < 37 { … } else { match t {37=>7,38=>8,39=>9,80=>10,_=>11} }`（`git show fb1ddc6^` 实文）——79 走 else ⇒ **11** ⇒ `D[11]`＝"东海龙鳞…防御+220"（**甲文案**）。
  - 本提交：`pickup_params_for(79)=Equip{tier:4}` ⇒ `overlay_kind_pickup(79, 4)` ⇒ `D[4]`＝"万木之灵…攻击+120"（**武器 tier4 文案**）。
  - Java：`equipTierIndex(79,0,12)=4`（a.java:7699-7709 手推）+ `equipDescriptions[var2]`（7644）⇒ **= 本提交**。
  - ⇒ 该变更**方向=对齐 Java（父提交是错的）**，但：①commit message 未披露（message 只写"逐臂等价/映射表对拍"）；②**零测试钉**（`fx-pickup-33` 为 33..=36|79 组代表夹具、实拾型≠79；14 等价测无 overlay tier 断言；74 金样 sha 全同=无 79 场景）——属**未披露、未覆盖的行为变更**。

**② 判：🟡 黄**（5 臂中 3 臂全绿、④⑤ 各带 1 黄：⑤=79 未披露变更；④的边叉记入攻击点③）

## 3. 攻击点③：红修实录核（fx-pickup-31/32 hp_popup）

### 3.1 链路逐环核（🟢）

| 环 | 位置 | 实文 | 判 |
|---|---|---|---|
| 请求产生 | effect_registry.rs:244-251 | `if ctx.player.hp <= -d { hp = 1 } else { hp += d }`；`let popup = if d > 0 { Some(d) } else { None }`；`Consumed { overlay_tier: None, hp_popup: popup }` | 纯数据请求 ✓ |
| 类型载体 | effect_registry.rs:66 | `Consumed { overlay_tier: Option<i32>, hp_popup: Option<i32> }`（`Option` 编码"是否请求"，无副作用字段） | 纯数据 ✓ |
| 编排执行 | scene.rs:4259-4271 | 两个 `Consumed` 分支各 `if let Some(d) = hp_popup { self.spawn_hp_popup(d) }` | 编排面执行 ✓ |
| 同参逐字 | scene.rs:4291-4301 `spawn_hp_popup` vs scene.rs:4403-4412 `apply_hp_delta` 正分支 | 两边均 `popups.spawn(3, delta, view.player_px, view.player_py, view.cam_x, view.cam_y, images.popup.digit_strips[0].width / 11)` —— **7 参逐字同（kind=3、值、四坐标、字宽/11）** | ✓ |

- **P1-2 架构合规？→ 合规**：`HpUpEffect` 全程只触 `ctx.player/ctx.floor`（纯数据域），**不持有 `popups/view/images`**；浮层以 `ItemOutcome` 字段上交、由 `GameScene`（编排面）执行 = P1-2 定的"纯数据 outcome + 副作用请求上交"分工 ✓。
- **原臂 popup 语义 a.java 锚逐字核**：`applyHpDelta`（**a.java:5172**）→ 5173 `if (this.playerHp <= -var1) { playerHp = 1; var1 = playerHp - 1; }` → 5181-5189 `if (var1 < 0) spawnPopup(2,…) else if (var1 > 0) { if (difficultyIndex>0) var1<<=4; spawnPopup(3, var1, …) }` ⇒ **正 delta 走 kind 3** ✓ 与新链路一致；原臂 31/32 走 `apply_hp_delta`（旧 scene.rs 臂，`git show fb1ddc6^` 实文 `31 => self.apply_hp_delta(50 * self.floor_tier())`）⇒ 父提交浮层在、本提交改由 outcome 请求承载，**正常分支语义守恒** ✓。
- **45/45 复绿实跑（🟢）**：`cargo test -p game-oracle --test fixture_frames` = `45 passed; 0 failed`（含 `fx_pickup_31_hp_frames_match`、`fx_pickup_32_hp_frames_match`）——夹具为 **Java 三方 trace 逐 TICK 帧 sha**，浮层在/不在直接反映在帧里 ⇒ 正分支等价有硬证据 ✓。

### 3.2 黄：`hp <= -d` 边缘分叉（本 commit 引入）

- **分叉**：`hp <= -d` 时 Java/`apply_hp_delta` 把 `delta` 归 0（`var1 = playerHp - 1 = 0`）⇒ **不发浮层**；新 `HpUpEffect` 不改 `d`，`popup = if d > 0 { Some(d) }` 恒真（31/32 的 `d = 50/200 × floor_tier ≥ 50`）⇒ **发 +d 浮层**。父提交因走 `apply_hp_delta` 而与 Java 一致 ⇒ **本 commit 在该分支引入保真回退**。
- **可达性（两侧同构）**：需拾取时 `hp ≤ -50`。①Java 与 Rust 的战斗 strike 均**无 hp 地板**（Java a.java:10045 `this.playerHp -= var8`；Rust scene.rs:2076 `self.player.hp -= strike`、3788 `-= counter`——仅"踩格损失"路径有 `loss < playerHp` 守卫，a.java:5478/8774 ↔ scene.rs:2133 同构）；②**两侧均无 `hp <= 0` 死亡判定**（`grep "playerHp <= 0|playerHp < 0" a.java` 空、`grep "hp <= 0" crates/game-core/src` 空）⇒ 负 hp 状态可滞留，再拾 31/32 即触发。内容面：脚本资产 `assets/raw/script` 中 `ROS_3`（设 HP）**0 命中**，故"脚本置负"路径当前不可达；但战斗 strike 路径可达。
- **无钉**：14 等价测中 `hp_popup` 断言**全为 `None`**（`grep "hp_popup: Some" crates/` = 0），`hpup_arm_matches_match_semantics` 只测正分支（effect_registry.rs:499-508，`delta:200` + 正 hp）；边叉无任何测试。

### 3.3 范围外注记（非本 commit 引入，仅挂账）

- **difficultyIndex>0 的 popup `<<=4`（×16）**：Java 5185-5187 有、Rust `apply_hp_delta` 与新 `spawn_hp_popup` **均无**——父提交既有（scene.rs:4228 注自承"difficultyIndex=0 → 无 ×16"），本 commit 与父提交同缺 ⇒ 非回退，挂账。
- **失败提示文案**：Rust `show_overlay_message(0, "你拥有更强力装备", 0,3)`（scene.rs:4282）vs Java `miscTexts[5]` = **"你拥有更强力的装备，因此将捡到的丢弃了。"**（a.java:438 表第 5 项 / 7646 引用）——**父提交同字面**（`git grep` fb1ddc6^ 命中 2 处，源自 435e87a），且 spec 自相矛盾（`docs/spec/gameplay.md:270` 记全句、`docs/spec/p3-render.md:292` 记短句）⇒ 既有分歧，另开账。

**③ 判：🟡 黄**（红修主面 4 环全绿+45/45 帧证+P1-2 合规；黄=hp≤-d 边叉为本 commit 引入的保真回退、无钉，附 2 条范围外挂账）

## 4. 攻击点④：三设施薄委托

### 4.1 `GameScene::floor_tier` → `effect_registry::floor_tier`（等价 🟢 / 孤儿 🟡→已记①）

- 逐行同式（父臂内联体 vs effect_registry.rs:130-137）：`t = (floor-1)/10` → `t<0 ⇒ 0` → `floor>50 ⇒ 0` → `t+1`，**四步全同**，无分支增删 ✓；与 a.java:7651-7662 逐字同 ✓。
- 调用面：**0**（见①黄：原 4 调用全在被删臂内）——等价本身无分叉，但"调用面零改动"表述失实。

### 4.2 `GameScene::try_equip` → `EquipEffect`（等价 🟢 / 注释失实 🟡）

- **force 全语义核**：`EffectParams::Equip { tier, force }`（effect_registry.rs:57，本 commit 新增字段——**行为参数非物品定义参数**，`items.json` 零变 ✓ 实测 `git show --name-only` 仅 2 个 .rs 文件）。
  - 父 `try_equip` 内联：`if cur >= tier && !force { return false; }`（两段同构）⇒ 子 `EquipEffect`：`if cur >= tier && !force { return NotConsumed }` —— **条件式逐字** ✓；
  - 成功路径 `atk/def -= bonuses[cur]; += bonuses[tier]; equipped = equip_types[tier]` 双动 ✓ 同；
  - 返回值 `matches!(out, Consumed{..})` ≡ 旧 `bool`（`NotConsumed`→false / `Consumed`→true）✓。
- **m_083 同构**：`GameScene::m_083_index`（scene.rs:886-898）与 `effect_registry::m_083_index`（effect_registry.rs:289-299）**逐行同**（`t = hi; while t > lo { t -= 1; if types[t]==equipped { return t } } 0`）⇒ 行为零分叉；但**双实现并存**（前者已死，见①）。
- **script_ops:281 force 路径单源核**：`op_ros_equip` 实文 `self.try_equip(tier, true)`（script_ops.rs:281，行号与 commit 申报同 ✓）→ 委托 `Equip{tier, force:true}` → 跳过 cur 检查强换 ✓；脚本资产实含 `ROS_2_0` / `ROS_2_6` 两处 force 调用（`assets/raw/script` grep），**已走新单源** ✓。
- **🟡 注释失实**：scene.rs:867 新注称"本方法=薄委托，**调用面 4278/4297**/script_ops:281 **零改动**"——实测父提交 3 调用（4278/4297=两拾取装备臂 + script_ops:281），**本提交后仅剩 script_ops:281**（`grep "\.try_equip("` 全仓=1）：4278/4297 随 23 臂**同 commit 删除**，pickup 面已不再经过 `try_equip`（改由分发直连 `EquipEffect`）。

### 4.3 `GameScene::add_item_to_stack` → `StackEffect`（等价 🟢 / 注释行号 stale🟡）

- 委托序核（scene.rs:4303-4333）：`StackEffect.apply(Stack{type_id:t, tutorial:false})` → `drop(ctx)` → `set_overlay_kind_item(t)` → `idx = 85?20 : 86?21 : t-13` → `item_description(idx)` → `show_overlay_message(1,…,0,3)`。
- 与父内联体对拍：`ITEM_USES`/`find_item`/栈顶追加/越界守卫 `n < item_types.len()` 全部移入 `StackEffect` 且**表值与 a.java:944 逐字同**（见②-①）；`tutorial:false` ≡ 父 `add_item_to_stack` **从不写 `tutorial_seen`**（15 的 tutorial 只在旧 pickup 臂写，现由 `pickup_params_for(15).tutorial:true` 承载）✓；副作用三件（TargetType 写/idx/浮层）**留置编排面**且顺序与父一致（父=栈增→TargetType→浮层，子同）✓。
- 调用面：2671/2672（商人赠物 13/19，a.java:7667 面）零改动 ✓；第 3 调用面（父 4252=pickup 臂）随本 commit 删除（pickup 改直连 `StackEffect`）。
- **🟡 注释行号 stale**：scene.rs:4305 称"调用面 **2672/2673**/测试零改动"——现树实为 **2671/2672**（本 commit 首 hunk 净 -1 行导致整体位移）；且未注第 3 调用面已删。

### 4.4 编排副作用留置注记（🟢）

commit 明列三留置：①itemUse `TargetType` 写（`set_overlay_kind_item`，拾取面函数头既有写+商人面委托后写）；②描述浮层（`item_description`+`show_overlay_message(1,…)`）；③装备浮层/失败提示（`overlay_kind_pickup`/`show_overlay_message(0,…)`）——**全部落在 `GameScene` 编排层，`effect_registry` 侧零 UI 实调用**（`grep -n "popups\|overlay\|show_overlay\|images\." crates/game-core/src/effect_registry.rs` = 15 行，全为注释或字段名 `overlay_tier`/`hp_popup` 字面，**无 `popups.spawn` / `show_overlay_message` / `images.` 调用**）✓，与 P1-2 分层一致。

**④ 判：🟢 绿**（三委托等价核全过、行为分叉 0——79 归②、hp 边叉归③；2 条注释失实/行号 stale 为记录级，随①黄一并挂账）

## 5. 攻击点⑤：验证实跑全账

**全部在 HEAD=fb1ddc6 独立实跑**（2026-11-06，非引用 commit 贴件）：

| # | 项 | 命令 | 实测 | 与申报 |
|---|---|---|---|---|
| 1 | 等价测 14/14 | `cargo test -p game-core --lib p1a` | `14 passed; 0 failed; 0 ignored; 322 filtered out` | **逐字同 ✓** |
| 2 | core 全量 | `cargo test -p game-core` | `334 passed; 0 failed; 2 ignored`（lib 13.17s） | **同 ✓** |
| 3 | 夹具 45/45 | `cargo test -p game-oracle --test fixture_frames` | `45 passed; 0 failed`（118.77s vs 申报 104.38s，**时长差属机器负载，计数同**） | 同 ✓ |
| 4 | arm_coverage 21 | `cargo test -p game-oracle --test arm_coverage -- --nocapture` | `1 passed`；`21 全账：夹具 20 + 正名 1 = 21` | 同 ✓ |
| 5 | 金样 74 `--check` | `python3 tools/state_baseline.py --check` | `[GREEN] 金样逐字节等价：74 场景 sha 全同` | **逐字同 ✓** |
| 6 | `match t` 残留 6 | `grep -c "match t" crates/game-core/src/scene.rs` | `6` | 同 ✓（出处逐条见①；**口径=scene.rs 单文件**；全 crate 精确口径 9=6+effect_registry 新表 2+scene_menu use 面 1，附列防口径误读） |
| 7 | 总闸 | `python3 gates/cli.py status` | `PASS=13 FAIL=0 SKIP=1 TOLERATED=0`（SKIP=visual-diff 在册） | 绿 ✓ |
| 8 | 编译面 | `cargo build -p game-core` | 4 warning：**1 新**（`m_083_index never used`，本 commit 引入）+3 既有（`text_len`/`set_undo`/`add_ease_target_y`，父提交同样零调用方，`git grep` 复核） | 申报未提 ⇒ 记① |

- 14 等价测清单（`-- --list`）：`equip_arm_matches_match_semantics`、`equip_armor_tier_ge6_def_bonus`、`hpup_arm_matches_match_semantics`、`hpup_no_9999_clamp_discriminator`、`key_arm…`、`kind_coverage_full_table`、`stack_arm…`、`statup_arm…`、`statup_floor_tier_arm`、`type_to_kind_mapping_no_miss`、`unknown_effect_is_noop`、`use_arm_noop_discriminated` + **本 commit 新增** `p1a3::pickup_params_for_all_arms`、`p1a3::equip_force_semantics`（2/14，TDD 红先行面）。
- **覆盖缺口（如实记）**：`hp_popup: Some(...)` **无任何单测断言**（`grep -rn "hp_popup: Some" crates/` = 0）；`t=79` overlay tier **无钉**（夹具组代表 `fx-pickup-33` 实拾型≠79、金样 74 无 79 场景）；`hp <= -d` 边叉**无钉**——三处缺口分别对应黄②③。

**⑤ 判：🟢 绿**（6 项验证计数/输出与申报逐项复现，总闸 13 PASS；缺口已如实列并归账黄②③）

## 6. 三纪律核（终态制 / 引锚 sed / 夹带净 --stat）

### 6.1 `$(cat)` 终态制（🟢）

commit 贴件 6 项均标注"命令输出重定向，未手编辑；贴件=提交树终态"——我在 HEAD **逐项复跑，计数/输出全同**（14/14、334/0、45/45、21、`[GREEN] …74 场景 sha 全同`、6）⇒ 贴件确为终态可复现 ✓。
- 如实记：**TDD 红证据（E0425 cannot find function 三连）为历史态，HEAD 不可复现**——只能核"message 陈述 + 现绿 + `p1a3_tests` 两测确在（`-- --list` 实证 2/14）"，不能核红本身。

### 6.2 引锚 sed（🟡）

本 commit **新增锚 16 个**（`git show fb1ddc6 -- crates/ | grep "^+" | grep -o "a\.java:[0-9-]+"` 去重），我逐个 `grep -n`/`sed -n` 对 `reference/src/deobf/a.java` 实文核：

| 新锚 | 实文核 | 判 |
|---|---|---|
| 7534 / 7537-7648 / 7630-7631 | `pickupItemType`=7534、`switch`=7537、switch 收=7648、`default:`=7630 + `return;`=7631 | ✅ 逐字 |
| 7538-7539 / 7538-7555 / 7556-7564 / 7565-7580 / 7581-7587 / 7632-7648 / 7651-7662 | case15=7538、`addItemToItemStack`=7554、case26=7556、case29=7565、case31=7581、case33=7632、`floorTier()`=7651 | ✅ 逐字 |
| 7667（商人赠物路径·**新**） | 7667 实文 = `findItemStackIndex` 体内 `for (int var3 = 0; …)`；**商人赠物调用在 7339/7342**（`addItemToItemStack(13)` / `(19)`） | ❌ **错锚**；且**父提交原值 7333/7336（近准）被本 commit 改坏** |
| 7553（22..25 落穿·**新测注**） | 7553 = `case 86:`；`case 22`=**7548**、落穿调用=**7554** | ❌ **失准 1~5 行**（sed 可见但语义错位） |
| 7738 / 7727-7740 / `script_ops:281` | 7738=`itemUseTargetType=(byte)var1` ✅；7727=`addItemToItemStack` 定义 ✅；script_ops.rs:281=`self.try_equip(tier, true)` ✅ | ✅ |

**同函数既有 stale 锚未随改**（非本 commit 引入，但本 commit 恰好重写了这些函数体，属"顺手可清"）：scene.rs:863 `tryEquip（a.java:7668-7680）`→ 实文 **7677-7697**；scene.rs:885 `equipTierIndex（a.java:7688-7694）`→ **7699-7709**（硬门 6b676f5 只改了 effect_registry 两处 7699-7709，scene.rs 漏改）；scene.rs:4229 `floorTier（a.java:7642-7652）`→ **7651-7662**（本 commit 新注已用对值，旧注未同步）；scene.rs:4378 `equipDescriptions[script_cursor0] a.java:7595-7597`→ **7643-7644**。

**⑥ 引锚子项：🟡 黄**（16 新锚 14 中 2 失准，其中 1 为**主动改坏**的错锚；引锚纪律要求 sed 实文可核，7667 不满足）

### 6.3 夹带净 `--stat`（🟢）

`git show --name-only fb1ddc6` = 恰 `scene.rs`、`effect_registry.rs` 两文件；`--stat` = `2 files changed, 218 insertions(+), 119 deletions(-)` 与任务申报/commit 一致；**无测试、无 docs、无门禁、无 `items.json` 夹带**（`EffectParams::Equip` 增 `force` 字段为行为参数，物品定义零变——`--name-only` 实证）✓。

**⑥ 判：🟡 黄**（终态制🟢、夹带净🟢、引锚🟡）

## 7. 总判

### 7.1 六点分判

| # | 攻击点 | 判 | 一句话 |
|---|---|---|---|
| ① | 禁双轨核（核心红线） | 🟡 **黄** | 红线全绿（pickup 区 `match t`=0、两文件同笔、旧臂零残留）；黄=委托化死码 2 处（`m_083_index` **新增编译警告+双实现**、`floor_tier` 孤儿致"调用面零改动"失实） |
| ② | 逐臂等价（23 臂+抽 5 臂） | 🟡 **黄** | 3/5 臂深核全绿；黄=`t=79` 浮层 tier **11→4 未披露变更**（方向=对齐 Java `equipTierIndex(79)=4`，父提交为错，但零披露+零钉） |
| ③ | 红修实录（hp_popup） | 🟡 **黄** | 4 环链路+7 参逐字+45/45 帧证+P1-2 合规全绿；黄=`hp<=-d` 边叉为**本 commit 引入的保真回退**（Java/父不发浮层、本侧发），无钉 |
| ④ | 三设施薄委托 | 🟢 **绿** | floor_tier/try_equip(force 全语义+script_ops:281 单源)/add_item_to_stack 三处**行为分叉 0**；副作用留置合规（2 条注释行号失实随①挂账） |
| ⑤ | 验证实跑全账 | 🟢 **绿** | 14/14、334/0、45/45、21、金样 74、`match t`=6 六项 **HEAD 独立复跑与申报逐字同**；总闸 13 PASS；3 处覆盖缺口已归账②③ |
| ⑥ | 三纪律 | 🟡 **黄** | 终态制🟢（贴件全复现）、夹带净🟢（恰 2 文件 +218/-119）；黄=引锚 16 新锚 14 中 2 失准，含 **`a.java:7667` 错锚（父 7333/7336 被改坏，实文 7339/7342）** |

**红 0 / 黄 4 / 绿 2。**

### 7.2 黄账（按处置优先级）

| 序 | 项 | 证据 | 修法（量小） |
|---|---|---|---|
| 黄-① | **`hp <= -d` popup 分叉**（保真回退，本 commit 引入） | effect_registry.rs:244-251 vs a.java:5173-5177（`var1 = playerHp-1` 归零→不发）；可达性=无地板 strike（a.java:10045 / scene.rs:2076、3788）+两侧均无 hp≤0 死亡判定 | `HpUpEffect` 逐字复刻：`if hp <= -d { hp = 1; d = hp - 1 } else { hp += d }` 后再判 `d > 0` → popup；补 `hp_popup: Some/None` 双侧单测 |
| 黄-② | **`t=79` 浮层未披露变更** | 父 `eq_tier` else 臂 `_=>11`（`git show fb1ddc6^`）→ 子 `pickup_params_for(79)=tier 4`；Java `equipTierIndex(79)=4` + `equipDescriptions[var2]`(7644)；零测试钉（`fx-pickup-33` 实拾型≠79、74 金样无 79 场景） | commit message 补披露（"79 浮层 tier 11→4=对齐 a.java:7699-7709，父为错"）+ 补 overlay tier 钉测（p1a3 增 79→`overlay_kind_pickup` 取 tier 断言） |
| 黄-③ | **死码/双源 2 处 + 3 条调用面表述失实** | `cargo build` 4 warning 中 1 新（`m_083_index never used`，父 2 调用→0）；`floor_tier` 0 调用；try_equip 注"4278/4297 零改动"（父 3 调用→现 1，2 处随臂删）；add_item_to_stack 注"2672/2673"（现 2671/2672） | 删 `GameScene::m_083_index`（消警告+双源）；`floor_tier` 删或改注"暂无调用面"；两处注释改准 |
| 黄-④ | **引锚 2 失准** | `a.java:7667`（实文=findItemStackIndex 体内，商人赠物=**7339/7342**，父值 7333/7336 被改坏）；`a.java:7553`（实文=`case 86:`，22..25=`7548`、落穿=**7554**） | 改锚；顺手清同函数 4 条既有 stale 锚（863→7677-7697、885→7699-7709、4229→7651-7662、4378→7643-7644） |

### 7.3 总判

> ## 🔶 需修复（轻·黄 4 / 红 0）

**理由**：核心红线（禁双轨）与全部 6 项验证均绿，分发接线本体可信；但 4 黄中有 2 条触及 AGENTS 硬纪律——**黄①=保真回退**（"保真期行为只增不减"，Java/父提交在 `hp<=-d` 不发浮层，本提交发）、**黄②④=逐提交披露与引锚纪律**（未披露行为变更 + 锚被主动改坏）。均非架构问题、量小（约 10 行 + message 更正），且**提交未推送、amend 零成本**，不宜裸推。

**修复后复验（约 4 分钟）**：`cargo test -p game-core --lib p1a` + `cargo test -p game-core` + `cargo test -p game-oracle --test fixture_frames` + `arm_coverage -- --nocapture` + `python3 tools/state_baseline.py --check` + `gates/cli.py status`，再交推送。

**范围外挂账（非本 commit，另开账）**：①`difficultyIndex>0` popup `<<=4`（×16）Rust 双处同缺；②失败提示文案 Rust"你拥有更强力装备" vs Java `miscTexts[5]` 全句（a.java:438/7646；spec `gameplay.md:270` 与 `p3-render.md:292` 互相矛盾）；③scene.rs 4 条既有 stale 锚（见黄-④修法）。
