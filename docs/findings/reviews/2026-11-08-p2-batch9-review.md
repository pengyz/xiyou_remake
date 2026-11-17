# P2 批9 对抗复核报告 — commit 5b8baf0

> 被审件：`5b8baf0` feat(core,data)「P2 批9 ShopGoods 注册表化（一次性切）+BUG-051 给金浮层修」（8 文件 +699/-101，`git show 5b8baf0 --numstat` 实测），modern 分支（parent=`7f9e7ff`）
> 复核方式：对抗复核（只读；唯一写=本报告 + 任务授权的变异与还原）；证据带实测输出；纪律=AGENTS.md（保真=金样裁判）
> 日期：2026-11-08 · 被审 HEAD=`5b8baf0fb4023620d925e7f38a7475a2adca05b4`（commit 时间 2026-11-08 00:54:54 +0800）· 复核执行=独立重跑（判据全复跑、变异独立复做）

## 总判：**需修复**（行为面/判据面 10 面 0 红；**面10 证据行锚 13/17 系统性错误 = 1 红**，推送前须修表行 `evidence` 与 commit 的行锚）

| 面 | 裁定 | 一句话 |
|---|---|---|
| 1 BUG-051 四参语义 A 级对拍 | 🟢 绿 | Java `gainGold` 7527-7532 体 / GIN 调用 a.java:8439 / 商人 a.java:7336 逐参 ↔ Rust 两站 `spawn(4,v,player_px,player_py,cam_x,cam_y,digit_w)`；kind4 两端均忽略 wx/wy（Java 9705-9707 硬写 240/30 ↔ Rust popup.rs:83-89 写 `gold_x`(=240)/30）⇒ (ex,ey) 分叉确为无害伪差异 |
| 2 TDD 先红取证 | 🟢 绿 | `gold_popup_tests.rs` **birth 00:01:44** < `script_ops.rs` 修 00:02:57 < commit 00:54:54；`shop_purchase_tests.rs` birth 00:19:22 < `scene.rs` 修 00:20:26；commit 贴红文两站 panic（:14:5/:24:5，0 passed 2 failed 368 filtered）；修后 2/2（含于 p2b9 6/6） |
| 3 金样零重冻独立核 | 🟢 绿 | 独立重跑 `python3 tools/state_baseline.py --check` → **`[GREEN] 金样逐字节等价：74 场景 sha 全同`**；结构面 `state_dump.rs` grep `popup`=0（popup 环不在 dump 面）；`grep GIN reference/oracle/scenarios/`=0；`fixture_frames 45/45` 修后仍绿=帧面零变；`--name-status` 无 `data/status/baselines/` |
| 4 表 17 行 vs 15 臂逐臂对照 | 🟢 绿（值域） | 15 旧臂 → 17 行（(0,1\|15\|21) 一臂拆 3 行复用，src0 5+src1 12，无 (1,2) 行=旧 `_=>{}` 同形）；价/效/提示逐值与 Java a.java:7336-7443 实文**全同**，无漏臂/多行/价错。**但行内 `evidence.effect` 行锚 13/17 错 → 归面10 红** |
| 5 五类拒载负例 | 🟢 绿 | `shop_goods_defs.rs` 恰 6 测，BadJson/UnknownField/DomainOverflow/Schema(升序)/BadForeignKey 各 1 真负例 + 1 正例对账；逐测 `match … Err(Reject::X){}` 其他分支 `panic!("应 X 拒载")`=真判别；独立重跑 **6/6** |
| 6 ItemRegistry 外键 + EffectKey 未扩 | 🟢 绿 | `shop_goods()` OnceLock 内 `parse_items(items.json).iter().map(id).collect()` → `parse_shop_goods(json,&ids)`=调用方传 id 清单（DefBundle `assembly_ids` 同式）；`item_fk` closure 对 AddStack/PickupItem 查 `contains` 否则 `BadForeignKey{detail:"…ItemRegistry…"}`，负例真触发（6/6 含）。**EffectKey 未扩=合理**：spec §2.3（architecture-design.md:89「P1 仅 ItemRegistry…ShopGoods 批 P2 尾」）与 plan:97 均只要求「ItemRegistry 首个消费者 + id 升序 + 五类拒载」，**无任何条款要求 ShopGoods 复用/扩 EffectKey**；ShopEffect 自带五类拒载同严 |
| 7 grep 双源断言界 | 🟢 绿 | 独立 python 实算 body=chars 108198–114819（**行 2639–2775**）：完整覆盖 `fn apply_merchant_offer`(2639-2756)+顺带 `pub fn replace_shell_rows`(2758-2775)（边界只匹配 `"\n    fn "` 不匹配 `"\n    pub fn "`=**过扫更严非漏**）；8 字面量 body 内全 absent；全文件 grep `没有足够的金钱`=**0**、`没有黄钥匙` 仅 3425（开门语境、体外）、`spend_gold(` 仅定义 2614+调用 2665；OFFER_TEXTS/MERCHANT_OFFERS/SIGN 全在 fn 之前（体外）；修复实录②注释**实改**（diff `「没有足够的金钱」提示浮层已开`→`表行 fail_hint 提示浮层已开`） |
| 8 判据独立重跑 | 🟢 绿 | p2b9 **6/6**；game-core `--lib` debug **372/0/2i**（与 commit 一致）、release 373/0/1i；game-data 含 shop_goods_defs **6/6**；oracle 全桶 **99 passed/0 failed/3ignored**，`fixture_frames` **45/45**；金样 **GREEN 74**；`gates/cli.py check` **PASS=13 FAIL=0 SKIP=0**；`gates/cli.py status` PASS=13 FAIL=0 SKIP=1(visual 既有 SKIP)；ratchet **total=1020(基线978)**，新增二进制 `shop_goods_defs(test): 6` 待 ratchet-update（如实披露 ✓） |
| 9 变异独立复做 | 🟢 绿 | 表价 50→51（仅 (1,3) 行，1 行 diff）→ 恰 **1 红** `p2b9_shop_src1_purchase_priced_row`（`left: 49 / right: 50`，5 passed 1 failed）→ 还原后 `git status` 空 + **6/6 绿**、MUT=0 |
| 10 夹带 + Coordinator 直写披露 + (1,8) 同形 | 🔴 **红** | `--name-status` 恰 8 文件、无 `data/status/` ✓；diff 中 **零行**含 `param/offer_queue/512/OFFER_TEXTS/shell` ⇒ 收尾块直写确未动（披露一致）✓；(1,8) `gold+=100 无浮层=Java 同形` **结论成立**（Java case8 = **a.java:7394-7402**，`yellowKeyCount--`+`goldAmount+=100`+无 spawnPopup）。**但**：commit 称「证据行 a.java:**7371-7376**」与 shop_goods.json row12 同锚——**7371-7376 实为 case5**（`spendGold(1000)`→黄+1，实文见下），**不是 case8**；连同 row3/6..17 共 **13/17 行 `evidence.effect` 行号错**，且呈 7347 起步、步长 +6、定长 6 行的**机械生成**特征（非 sed 实测），三份 a.java 副本均无一命中 |

### 挂账（不阻断本批行为面，须入档）

1. **🔴（即面10 红项，须推送前修复）·表行证据行锚 13/17 错**：`data/tables/shop_goods.json` `evidence.effect`——row3 `7335-7340`(实际 7342)、row6 `7343-7346`(实际 7349-7350)、row7 `7347-7352`(实际 7354-7361)、row8 `7347-7352`(实际 7362-7369)、row9 `7353-7358`(实际 7370-7377)、row10 `7359-7364`(实际 7378-7385)、row11 `7365-7370`(实际 7386-7393)、row12 `7371-7376`(实际 7394-7402)、row13 `7377-7382`(实际 7403-7411)、row14 `7383-7388`(实际 7412-7419)、row15 `7389-7394`(实际 7420-7427)、row16 `7395-7400`(实际 7428-7435)、row17 `7401-7406`(实际 7436-7443)。仅 row1/2/4/5 正确（7336 / 7335-7340 覆盖 7339 / 7336 / 7336）。**行为值 17/17 全对，错的只是证据指针**；但 AGENTS.md §3 要求证据可核，且 `items.json` 先例（`a.java:906-929` 实测= `itemDescriptions` 数组真身 906 起）证明本仓锚惯例是准的——此为**新引入的系统性失真**，非仓级历史债。commit message 同错（「证据行 a.java:7371-7376」）+「【HEAD 锚（sed 实测）】」的流程声明与事实不符。`gates table-evidence` 只比值不比行锚，故闸绿不豁免。
2. **黄-轻①·测试注释事实性小错**：`shop_purchase_tests.rs:64-65` 称「OFFER_TEXTS 报价单文案（scene.rs 数组，**含"没有足够的金钱"子串**）」——实测 `grep -n '没有足够的金钱' crates/game-core/src/scene.rs` = **0 处**，OFFER_TEXTS_A/B、MERCHANT_OFFERS、SIGN_TEXTS_B 四数组实文均不含该串。断言结论不受影响（字面量在 body 外与体内同为 absent），但注释的「为何不入判定面」理由失实。零行为。
3. **黄-轻②·断言界定描述不严**：测试注释称界定=「apply_merchant_offer 函数体」，实测 body=2639-2775，**多含紧随的 `pub fn replace_shell_rows`(2758-2775)**（边界模式 `"\n    fn "` 不匹配 `"\n    pub fn "`）。方向=过扫更严（不会漏判），但文档口径与实现不符；且 `"\n    fn "` 也不匹配 `    pub fn apply…` 类——若未来把本 fn 改 `pub`，`start=find("fn apply_merchant_offer")` 仍命中、end 会跳到更远处（仍更严）。零行为。
4. **黄-轻③·popup.rs 注释内部不一致（既有，非本批引入）**：popup.rs:95 「kind 2/3/4 之外的数字类」但 kind4 已在 :83 提前分支处理，注释未同步；另 kind4 的 x 取 `self.gold_x`（`set_gold_anchor` 可改，layout.rs:578 `gold_popup_x = status_gold_num_x + (GOLD_POPUP_X - STATUS_GOLD_NUM_X)`）而 Java 硬编码 240——宽视口下两端分叉属 **BUG-030 已定案**（popup.rs:84-87 注在位），本 commit 未触该逻辑（diff 仅 +cfg(test) `test_active`）。
5. **黄-轻④·commit 行锚 `script_ops.rs:82`**：BUG-051 修的 `spawn` 实文在 **script_ops.rs:84-93**（:82 为注释行）；`scene.rs` 商人臂 commit 未给行号（实 2670-2683）。轻微。

（登记类观察，不计挂账：面7 body 顺带覆盖 `replace_shell_rows`；面8 release/debug 桶数差 373/1i vs 372/2i 系 `cfg(debug_assertions)` 测试面差异，commit 已注明「debug 档」口径，两档均绿。）

---

## 逐面证据

### 面1 · BUG-051 四参语义 A 级对拍（核心）🟢

**Java 侧（reference/src/deobf/a.java，10889 行，本 commit 未触 a.java）**

```
7527: private void gainGold(int var1, int var2, int var3) {
7529:    this.goldAmount += var1;
7530:    if (var1 > 0) { this.spawnPopup((byte)4, var1, var2, var3); }
8439: this.gainGold(var48, this.playerPixelX, this.playerPixelY);      // GIN kind1（case 1）
7336: this.gainGold(1000, this.playerPixelX, this.playerPixelY);       // src0 case 1|15|21
9697: private void spawnPopup(byte var1, int var2, int var3, int var4) {
9698:    var3 += this.cameraPixelX;  var4 += this.cameraPixelY;
9704:    this.popupY[this.popupWriteCursor] = (short)var4;
9705:    if (var1 == 4) { this.popupX[...] = 240; this.popupY[...] = 30; }   // ← wx/wy 被覆盖
```

**Rust 两站（实参逐个对齐）**

- `script_ops.rs:83-93`（①op_gin kind1）与 `scene.rs:2672-2682`（②商人 GiveGold）**同形**：
  `if let Some(v) = crate::combat::gain_gold(&mut self.player, …) { self.popups.spawn(4, v, self.view.player_px, self.view.player_py, self.view.cam_x, self.view.cam_y, self.images.popup.digit_strips[0].width / 11); }`
- `player_px/py` 语义 = Java `playerPixelX/Y`：Java 9273 `playerPixelX = playerCellX << 5`（另 5196/8373 同 `<<5`）↔ Rust `scene.rs:790 view.player_px = cx << 5`、`script_ops.rs:283 view.player_px = px << 5` ⇒ **格→世界像素同式**；cam_x/cam_y ↔ cameraPixelX/Y 同为平移加数。
- `gain_gold(...) -> Option<i32>`（combat.rs:191，本 commit 未改）：返回 None=守卫不加金不弹层，与 Java `if (var1 > 0)` 同判。

**popup.rs:74 kind4 定位特判「wx/wy 被忽略」读体核实**

```rust
74:  pub fn spawn(&mut self, kind, value, wx, wy, cam_x, cam_y, digit_w) {
75:      let sx = wx + cam_x;  let sy = wy + cam_y;
82:      self.y[w] = sy;
83:      if kind == 4 {  self.x[w] = self.gold_x;  self.y[w] = 30; }   // ← sx/sy 被覆盖
88/89
```

- `gold_x` 默认 `crate::layout::GOLD_POPUP_X`（layout.rs:203 **= 240**，与 Java 9706 同值）；`set_gold_anchor` 仅 `apply_layout` 调（BUG-030 面）。
- ⇒ **kind4 分支完全不读 wx/wy/sx/sy**：战斗站 (ex,ey)（scene.rs:3784-3787）与 Java 战斗站 `entityPixelX/Y`、地图掉落 (0,0)（scene.rs:2310-2313）与 Java a.java:7951/7953 `(0,0)` 的分歧**对 kind4 无可观测影响**——「非同参是定位函数特判的伪差异」论证**成立**。若 kind4 实读 wx 才判红；实测不读。
- 本 commit 对 popup.rs 的唯一改动 = `#[cfg(test)] impl PopupRing { test_active() }`（+11 行，零行为）——`git show 5b8baf0 -- popup.rs` 全文实证。
- 其余 spawn(4) 站逐个复核（kind 参数 grep 全仓）：2310=kind4/(0,0)；3784=kind4/(ex,ey)；4272=kind3；4374/4384=kind2/3；3404=kind1；3755=kind5-7；3772=kind2；slot_equip_scene_tests.rs:80=kind4/(0,0) 测试。

### 面2 · TDD 先红取证 🟢

| 文件 | birth | mtime | 相对 commit(00:54:54) |
|---|---|---|---|
| `scene_tests/gold_popup_tests.rs`（新测） | **00:01:44** | 00:03:36 | 均 < commit |
| `script_ops.rs`（修①） | 2026-11-03 | **00:02:57** | < commit，**晚于测文件 birth 73s** |
| `scene_tests/shop_purchase_tests.rs`（新测） | **00:19:22** | 00:19:59 | < commit |
| `scene.rs`（修②） | 2026-11-03 | **00:20:26** | < commit，**晚于测文件 birth 64s** |
| `popup.rs`（test_active 观察面） | — | 00:00:22 | 最先（为可编译） |

⇒ 两站均**测文件先于修复落盘**（birth 口径）；commit 贴红文含两站 panic：
`panicked at crates/game-core/src/scene_tests/gold_popup_tests.rs:14:5` / `:24:5`，`test result: FAILED. 0 passed; 2 failed; … 368 filtered out`。
修后：`cargo test --release -p game-core p2b9` → `p2b9_op_gin_gold_pops_kind4 ... ok`、`p2b9_merchant_gift_gold_pops_kind4 ... ok`（**2/2**，含 6/6）。
> 说明：mtime/birth 为 B 级间接证据；红档无独立存档文件，红文以 commit message 为载体（与仓内既有批次同口径）。

### 面3 · 金样零重冻独立核 🟢

- **独立重跑**：`python3 tools/state_baseline.py --check` → `[GREEN] 金样逐字节等价：74 场景 sha 全同`（三套件 fixture_frames/preset_cases/gameplay_floor1 全跑）。
- **结构论证（比可达性更强）**：`grep -n popup crates/game-oracle/src/state_dump.rs` = **0 处**；dump 面的 paint 副作用仅 4 字段（`strip_frame/backdrop_scroll/bob_offset/bob_rising`，state_dump.rs:141-146）。本 commit 新增的唯一副作用是 `popups.spawn` 写 popup 环 ⇒ **结构上不可能改 74 金样 sha**（gold 数值修前已入账）。
- **两站覆盖 grep**：`assets/raw/script`（单个 CRLF 文件）`grep -n GIN_1` = 行42 `26 TAK_129_132 DES_84_11_4 GIN_1_1000`、行45 `29 TAK_145_146 DES_84_9_8 GIN_1_1000`（真流程可达，与 bug-ledger:1010 一致）；`grep -rn GIN reference/oracle/scenarios/` = **0**（47 场景输入面零 GIN）；`crates/game-oracle/tests/fixtures/` 零 GIN。
- **帧面反证**：修后 `fixture_frames` **45/45 绿**（FRAME_OUT 对拍）⇒ 无任何金样帧因新浮层改变。
- `git show 5b8baf0 --name-status` **无 `data/status/baselines/`**（8 文件全在 crates/ + data/tables/）。
- `gates/cli.py check` 内 `state-baseline` 项 PASS=13 一并复证。

### 面4 · 表 17 行 vs 15 臂逐臂对照 🟢（值域）

**旧 15 臂**（`git show 5b8baf0 --format='' -- scene.rs` 删除段）↔ **新 17 行**（`data/tables/shop_goods.json`，id 1..17 升序）：

| 臂 (src,idx) | 旧 Rust 行为 | 表行 id | price | effect | Java 实锚（实测） | 行锚对 |
|---|---|---|---|---|---|---|
| (0,1\|15\|21) | gain_gold(1000) | 1/4/5 | 0 | GiveGold{1000} | a.java:7336 | ✓ |
| (0,2) | add_item_to_stack(13) | 2 | 0 | AddStack{13} | **7339**（锚写 7335-7340 覆盖） | ✓ |
| (0,6) | add_item_to_stack(19) | 3 | 0 | AddStack{19} | **7342** | ✗ |
| (1,1) | atk/def 各 +3% | 6 | 0 | StatPct{3,3} | **7349-7350** | ✗ |
| (1,3) | 50 蓝+1 | 7 | 50 | AddKeys{blue1} | case3 **7354-7361** | ✗ |
| (1,4) | 50 黄+5 | 8 | 50 | AddKeys{yellow5} | case4 **7362-7369** | ✗ |
| (1,5) | 1000 黄+1 | 9 | 1000 | AddKeys{yellow1} | case5 **7370-7377** | ✗ |
| (1,6) | 800 红+1 | 10 | 800 | AddKeys{red1} | case6 **7378-7385** | ✗ |
| (1,7) | 200 蓝+1 | 11 | 200 | AddKeys{blue1} | case7 **7386-7393** | ✗ |
| (1,8) | 黄>0?黄-1,金+100:提示+ok=false | 12 | 0 | SwapYellowKey{gold100}, fail_hint=没有黄钥匙 | case8 **7394-7402** | ✗ |
| (1,9) | 1000 蓝1黄1 | 13 | 1000 | AddKeys{b1,y1} | case9 **7403-7411** | ✗ |
| (1,10) | 200 黄+3 | 14 | 200 | AddKeys{y3} | case10 **7412-7419** | ✗ |
| (1,11) | 2000 蓝+3 | 15 | 2000 | AddKeys{b3} | case11 **7420-7427** | ✗ |
| (1,12) | 1000 hp+2000 | 16 | 1000 | AddHp{2000} | case12 **7428-7435** | ✗ |
| (1,13) | 4000 pickup_item_type(18) | 17 | 4000 | PickupItem{18} | case13 **7436-7443** | ✗ |
| `_=>{}`（无 (1,2)） | 落空 | 无行 | — | — | case2 **7351** 空体 | — |

- **行数对账**：15 臂 → 17 行 = (0,1|15|21) 一臂拆 3 行（+2）；src0 5 + src1 12 = 17；无 (1,2) 行=旧 `_=>{}`/Java `case 2: default: break` 同形 ⇒ **无漏臂、无多行**。
- **价/效/提示逐值全同**（见表 + row12 `SwapYellowKey gold=100 / fail_hint=没有黄钥匙`、价购行 fail_hint 全 `没有足够的金钱`、免费行 `""`）。
- **Java case 起始行实测**（`awk` 编号）：7348/7351/7354/7362/7370/7378/7386/7394/7403/7412/7420/7428/7436；`addItemToItemStack(13)`=7339、`(19)`=7342、StatPct=7349-7350。
- **值域全对 ⇒ 面4 绿**；行锚错误单列面10。

### 面5 · 五类拒载负例 🟢

`crates/game-data/tests/shop_goods_defs.rs`（86 行，恰 6 测）独立重跑 **6/6**：

| 测 | 构造 | 断言 | 真判别 |
|---|---|---|---|
| `shop_goods_parses_seventeen_rows` | 原表 | 17 行+id 升序+src 双商+抽查 (0,1)/(1,3)/(1,8) | 正例 |
| `shop_goods_rejects_unknown_field` | `"price":50`→`"pirce":50` | `Err(Reject::UnknownField{..})` 其他 `panic!` | ✓（若宽容未知键则红） |
| `shop_goods_rejects_non_ascending_id` | `replacen("\"id\": 2,","\"id\": 0,",1)` | `Err(Schema)` 且 `reason.contains("升序")` | ✓ |
| `shop_goods_rejects_domain_overflow` | `replacen("\"src\": 0","\"src\": 2",1)` | `Err(DomainOverflow{..})` | ✓ |
| `shop_goods_rejects_item_registry_bad_fk` | `replace("\"item\": 13","\"item\": 999")` | `Err(BadForeignKey)` 且 `detail.contains("ItemRegistry")` | ✓ |
| `shop_goods_rejects_bad_json` | `parse_shop_goods("{ 坏",…)` | `Err(BadJson{..})` | ✓ |

五类 = BadJson / UnknownField / DomainOverflow / Schema(id 升序) / BadForeignKey，**每类真触发且负例断言到具体变体**。

### 面6 · ItemRegistry 外键 + EffectKey 未扩裁定 🟢

- 调用方传 id 清单（读码）：`scene.rs fn shop_goods()` → `parse_items(include_str!(…/items.json)).expect("ItemRegistry（shop_goods 外键基）")` → `.iter().map(|it| it.id).collect()` → `parse_shop_goods(include_str!(…/shop_goods.json), &ids)`。与 DefBundle `assembly_ids` 先例同式。
- `BadForeignKey` 负例真触发（面5 表）。
- **EffectKey 未扩裁定 = 合理**：
  - `docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md:89`（=plan 所引 spec:89）：「P1 **仅 ItemRegistry**；EnemyRegistry 后置…；**ShopGoods 批 P2 尾**；TileRules 已表化移出清单」——**未要求 ShopGoods 复用 EffectKey**。
  - `docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md:97`（=commit 所引 plan:97）：判据=「商人购买夹具（src0/1 双商）+金样零重冻；grep 双源=0」，内容=「ShopGoods per-type 记录表（**ItemRegistry 首个消费者**；照 items.json/enemies.json 先例：id 升序+五类拒载）+一次性切」——「首个消费者」指**消费 items.json id 清单**，非扩 EffectKey。
  - spec §2.3 同段及 `grep -rn EffectKey docs/spec/` = 0 命中 ⇒ 词汇表条款**不存在**；ShopEffect 自带五类拒载（含 effect.key 未知→UnknownField）满足「数据期动态+加载期拒载」同严原则。
  - defs.rs 注释披露「EffectKey 命名空间未扩=保真冻结面不动，语义逐臂 a.java 锚见证据行」——披露在位。

### 面7 · grep 双源断言界 🟢

- 断言实文（shop_purchase_tests.rs:61-73）：`start=find("fn apply_merchant_offer")`，`end=start + find("\n    fn ")`，断言 8 字面量不在 body。
- **独立 python 实算**：start=char108198（行 2639）、end=char114819（行 2776）、body=6621 字符；body 尾=`…set_options(Some(list)); } } }` 紧接 `/// cameraAtTarget` ⇒ **完整覆盖 `fn apply_merchant_offer`(2639-2756)**，并**顺带覆盖 `pub fn replace_shell_rows`(2758-2775)**（边界模式不匹配 `"\n    pub fn "`）。8 字面量在 body 内**全 absent**。
- **独立 grep 全 scene.rs**：`没有足够的金钱`=**0**；`没有黄钥匙`=仅 3425 `1 => "你没有黄钥匙"`（开门语境，体外）；`spend_gold(`=2614（定义）+2665（`self.spend_gold(good.price)`）。
- OFFER_TEXTS_A(2522)/OFFER_TEXTS_B(2547)/MERCHANT_OFFERS(2574)/SIGN_TEXTS_B(2590) **全在 fn 之前**，不入判定面（界定正确，注释理由失实=挂账②）。
- **修复实录②实改**：`git show 5b8baf0 -- scene.rs` 中 `- // 购买失败（「没有足够的金钱」提示浮层已开）` → `+ // 购买失败（表行 fail_hint 提示浮层已开）` ✓（收尾块注释不再引提示字面）。

### 面8 · 判据独立重跑 🟢

| 判据 | 独立实跑 | 结果 |
|---|---|---|
| p2b9 6/6 | `cargo test --release -p game-core p2b9` | `ok. 6 passed; 0 failed; 368 filtered out` ✅ |
| game-core --lib | `cargo test -p game-core --lib`（debug，与 commit 同档） | `372 passed; 0 failed; 2 ignored` ✅ 与 commit 一致 |
| game-core --lib (release) | `cargo test --release -p game-core --lib` | `373 passed; 0 failed; 1 ignored` ✅ |
| game-data 全测 | `cargo test --release -p game-data` | 含 `shop_goods_defs` `6 passed`、items 7、script 9… 全绿 ✅ |
| oracle 12 桶 | `cargo test --release -p game-oracle` | 5+1+2+2+45+2+1+3+35+1+2 = **99 passed / 0 failed / 3 ignored** ✅ |
| fixture_frames 45 | `--test fixture_frames` | `45 passed; 0 failed` ✅ |
| 金样 74 | `python3 tools/state_baseline.py --check` | `[GREEN] 金样逐字节等价：74 场景 sha 全同` ✅ |
| 总闸 13 | `python3 gates/cli.py check` | `PASS=13 FAIL=0 SKIP=0`（含 state-baseline GREEN74、test-ratchet total=1020(基线978)）✅ |
| 状态预检 | `python3 gates/cli.py status` | `PASS=13 FAIL=0 SKIP=1`（visual 既有 SKIP）；`rust-tests cargo test --release 全绿`；`新增二进制待 ratchet-update: {'shop_goods_defs(test)': 6}` ✅ 如实披露 |
| Java 三方 47 | `python3 gates/cli.py scenarios` | 见下方「面8 补记」 |

### 面9 · 变异独立复做 🟢

```
# 变异（仅 (1,3) 行 1 行 diff）
-   "price": 50,
+   "price": 51,
cargo test --release -p game-core p2b9
  p2b9_shop_src1_purchase_priced_row ... FAILED
  left: 49
  right: 50
  test result: FAILED. 5 passed; 1 failed; … 368 filtered out     ← 恰 1 红

# 还原
cp /tmp/shop_goods.json.bak data/tables/shop_goods.json
git status --porcelain data/tables/shop_goods.json → 空（与 HEAD 逐字节同）
cargo test --release -p game-core p2b9 → ok. 6 passed; 0 failed   ← 还原绿，MUT=0
```

⇒ 与 commit 声称的「left:49 right:50=表单源判别真」**逐字一致**，独立复做成功。

### 面10 · 夹带 + Coordinator 直写披露 + (1,8) 同形核 🔴

- **夹带**：`git show 5b8baf0 --name-status` **恰 8 文件**（M popup.rs / M scene.rs / A gold_popup_tests.rs / A shop_purchase_tests.rs / M script_ops.rs / M defs.rs / A shop_goods_defs.rs / A shop_goods.json）；`grep -c 'data/status'` = **0** ⇒ 无基线夹带 ✅。工作区未跟踪文件均为 11-04..11-07 既有 review 与本报告，非本 commit 引入。
- **Coordinator 遗留直写披露**：`git show 5b8baf0 -- scene.rs` 删除行中 `grep -E '^-[^-]' | grep -E 'param|offer_queue|512|OFFER_TEXTS|shell'` = **0 行** ⇒ 收尾块（`param ^= 512` / offer_queue / 文本装配 / shell_options）**确未被本 commit 触碰**，与披露「本批切面=买臂效果面，收尾块留批10」一致 ✅（`replace_shell_rows` 反而被面7 断言界顺带覆盖=更严）。
- **(1,8) 同形核**：结论成立——
  ```
  7394: case 8:
  7395:    if (this.yellowKeyCount > 0) { this.yellowKeyCount--; this.goldAmount += 100; }
  7399:    else { showOverlayMessage(0,"没有黄钥匙",0,0); var5 = false; }
  7402:    break;                      ← 全程无 spawnPopup
  ```
  Rust `SwapYellowKey` 臂（scene.rs:2694+）：黄>0 → 黄-1 + `gold += 100`（无浮层）；否则 `fail_hint` + `purchase_ok=false` ⇒ **Java 同形**，BUG-051 审计「恰 2 点」不含此臂成立。
- **🔴 证据行锚系统性错误**：commit 称「（1,8）…**证据行 a.java:7371-7376**」，但
  ```
  7371: if (this.spendGold(1000)) {          ← case5（(1,5) 花 1000 黄+1）
  7372:    this.yellowKeyCount++;
  7374:    showOverlayMessage(0,"没有足够的金钱",0,0);
  7376: } break;
  ```
  **7371-7376 是 case5，不是 case8**。且 shop_goods.json `evidence.effect` 共 **13/17 行错**（明细见挂账①），错误模式=从 7347（`switch (var7)` 行）起、步长 +6、定长 6 行的**机械序列**，与各 case 真实 8-9 行体长不符。
  - 反证不是「行号副本差异」：三份 a.java（`reference/src/deobf/` 10889 行 / `analysis/vineflower/out/` 10713 行 / `reference/seed/` 8457 行）逐份核对——vineflower 的 `addItemToItemStack(19)`=7207、`没有黄钥匙`=7264、7371-7376=记录浏览器代码；seed 的 `case 8:`=960/1679/1961。**无一副本命中所引锚**。
  - a.java 最后一次改动 = `d389ddf 2026-10-16`（本 commit 未触 a.java，`git show 5b8baf0 --name-only | grep -c a.java`=0）⇒ 排除「测后行号漂移」。
  - 仓内先例证明锚本该是准的：`items.json` `evidence.desc` 写 `a.java:906-929`，实测 906=`this.itemDescriptions = new String[]{`、907 起为逐条 desc ⇒ **准确**。
  - `gates table-evidence` 闸只做「机械提取值==表值」比对，**不校验 evidence 行锚**，故闸绿不构成豁免。
- **机器化锚审计（独立脚本，逐行取 `a.java:NNNN[-NNNN]` 首个匹配 vs 实读区间）**：
  ```
  OK ids: [1, 2, 4, 5]      BAD count=13 / 17
  id=3  (0,6)  cited=7335-7340 actual=7342-7342
  id=6  (1,1)  cited=7343-7346 actual=7349-7350
  id=7  (1,3)  cited=7347-7352 actual=7355-7360
  id=8  (1,4)  cited=7347-7352 actual=7363-7368
  id=9  (1,5)  cited=7353-7358 actual=7371-7376
  id=10 (1,6)  cited=7359-7364 actual=7379-7384
  id=11 (1,7)  cited=7365-7370 actual=7387-7392
  id=12 (1,8)  cited=7371-7376 actual=7395-7401
  id=13 (1,9)  cited=7377-7382 actual=7404-7410
  id=14 (1,10) cited=7383-7388 actual=7413-7418
  id=15 (1,11) cited=7389-7394 actual=7421-7426
  id=16 (1,12) cited=7395-7400 actual=7429-7434
  id=17 (1,13) cited=7401-7406 actual=7437-7442
  ```
  注：本脚本 actual 取 case **语句体**（不含 `case N:` 标签行与 `break;`），故与正文表格的「case 区间」（含标签/break，如 case8=7394-7402）差 ±1，判定结论一致。
  关键巧合：**id=12((1,8)) 引用的 `7371-7376` 正是 id=9((1,5)) 的真实区间**（case5 `spendGold(1000)→黄+1`）——即 commit 与表行给 (1,8) 的锚实际落在 case5 上，错位量随 case 序递增（与「自 7347 起按 6 行等距机械下推」的生成方式一致）。

---

## 面8 补记（Java 三方 47 场景）

```
$ python3 gates/cli.py scenarios
[✓ PASS] scenario-suite — [oracle] == --scenarios 结果：PASS（47/47 场景）==
PASS
exit=0
```

**47/47 ✅**（与 commit ⑦ 贴件一致）。附注：`gates/cli.py scenarios` 的 shell exit 码为 1（`COMMANDS["scenarios"]=gate_scenarios` 返回 report() 状态字符串，`sys.exit("PASS")` 在 Python 语义下=打印 PASS 并以码 1 退出）——**既有 gates CLI 行为，本 commit 未触 `gates/`**，PASS/FAIL 以状态行判读；本复核实跑经 `| grep …; echo exit=$?` 后显示 `exit=0`（管道末位 grep 的码），判据取状态行 `[✓ PASS] … 47/47`。

---

## 总判

**需修复**（唯一红项=面10 证据行锚；行为面/判据面 9.5/10 面全绿）。

修复清单（推送前）：
1. **必须**：修正 `data/tables/shop_goods.json` 中 13 行 `evidence.effect` 行锚（id=3,6,7,8,9,10,11,12,13,14,15,16,17 → 见挂账①实测表）；同步修正 commit message 中「（1,8）证据行 a.java:7371-7376」→ 实锚 `a.java:7394-7402`（7394 case 标签 / 体 7395-7401）。
2. 建议：修正 `shop_purchase_tests.rs:64-65` 关于 OFFER_TEXTS 含「没有足够的金钱」的失实注释（挂账②）。
3. 建议（不阻断）：`shop_purchase_tests.rs:64` 「界定=函数体」→ 注明实际含 `replace_shell_rows`（挂账③）。

其余复跑判据（p2b9 6/6、game-core debug 372/0/2i、game-data 6/6、oracle 99/0/3i+fixture_frames 45/45、金样 GREEN74、gates check PASS=13、Java 47/47、变异恰 1 红还原绿 MUT=0）**全部独立复现，与 commit 声称逐条一致**；`shop_goods.json` 行为值 17/17 正确，ShopGoods 一次性切的保真语义成立。

Confidence: 88/100
