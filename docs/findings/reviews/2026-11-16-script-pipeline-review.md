# 对抗复核：脚本命令管线 + 执行端可装配 提案（commit `523c04f`）

> 复核对象：`docs/superpowers/proposals/2026-11-16-script-command-pipeline-proposal.md`（704 行）
> 复核方式：不采信提案任何结论，全部从 `reference/src/deobf/a.java`、`analysis/rename-pipeline/renamed/a.class`（`javap -p -c`）、
> `crates/game-core/src/*` 与真实夹具 `crates/game-core/tests/fixtures/level_scripts.txt` 独立重推。
> 复核者：红队（独立于设计者）。日期：2026-11-16。

## 结论

**有前提可推**（前提见 §P）｜ Confidence **76** ｜ 绿 **2** / 黄 **8** / 红 **2**

一句话：提案的**方向**（engine 向 Java 对齐、生产单一执行器、按 host 拆解）与**4 条 engine 侧伪造的定性**基本站得住；
但它的**头条 P0（PE-01）在现有数据口径下不可触发**，其后果证据与作者自己在 §B.4 写下的 11 字节事实自相矛盾；
而 §A.3 的「48/68 槽免 tick」判据把「无相位跃迁」当成「无状态增量」，实际 48 个「P 槽」里 **43 个含世界态/相位写面**，
只有 5 个真正无副作用。这两处若照单执行，会分别造成**定级错配**与**求解结论静默错误**。

### 三个必答问题

1. **PE-01 成立吗** → **部分成立**：结构缺陷真实（engine 多一道 Java 没有的尾门，锚点 `a.java:8574-8580` 是伪锚），
   但它在当前全部 68 槽上都**不可触发**（行尾 `0x1E` 使末游标恒为 `len-1`，不满足 `cursor >= len`），
   因此「每槽必触发 / 槽 6 整页对话跳过 / P0」不成立；只有剥离 `0x1E` 的 solver 表会触发，且触发后 `Obs` 也看不见。
2. **「GameScene 与 Java 一致」成立吗** → **不成立（局部一致）**：END/LAY/ROS_4/CES 这 4 点 GameScene 确实与 Java 一致，
   但生产路径存在**反向偏差**：`RES` 侧 GameScene 未移植 Java 的 `inventoryEntryCount=0`，反而多写了 Java 没有的 `journal.clear()`；
   提案 PE-06 把方向写反成「GameScene 对」，照此裁定会把 engine 中**唯一正确的那一条**删掉。
3. **分类表有没有漏状态增量** → **有，且是系统性的**：`TAK`/`SEE`（相位）、`CES`/`DES`（实体增删）、`SWD`（type81→4 改写）、
   `ROS_*`（属性/装备）、`GIN_1`（金币）、`RES`（整轮重置）、`ADD`（楼层重装+spawn）都写状态，
   却被 §A.3 归入「纯 P 类」；48 个 P 槽中 43 个含这些指令，真正的纯 P 槽只有 5 个（槽 37、39、58、59、60）。

---

## §P 推送前提（缺一不可）

1. **先定语料口径**：明确 `dialogScriptLine` 是否保留 `0x1E`。保留 ⇒ PE-01 是**死代码/潜伏**（P2）；
   剥离（Java 保真口径）⇒ PE-01 立即变活（P0 合理）。**不能两种口径都写在文档里还下「恒成立」的结论。**
2. **重做 §A.3 判据**：按「是否写世界态/相位」而不是「是否含 MOV/LAY/END/GUT」分类；至少把 TAK/SEE/CES/DES/SWD/ROS/GIN/RES/ADD 标为有状态增量。
3. **先裁定 PE-06/07/11，再谈 S5 零变化**：`inventoryEntryCount`（engine 有、GameScene 无字段）、`initCellGrid`、`idx==31` 三者未决，
   S5 切换后生产行为就不是「零变化」。
4. **`Obs` 扩容清单必须覆盖相位之外的量**：`dialog_phase`、`ticks`、`dialog_page/_end`、`script_delay`、`dialog_target`、
   `walk_request_flag/path_arrows_visible/walk_phase`、`pending_floor/transitioning`、`overlay_text`、相机锚点。
5. **S5 调用点清单修正**：文档正文写「9 处」、同页表格相加为 28、实测 grep 为 **22 处**；且需附 host 适配器字段迁移表。

---

## 逐面结论

### F1 javap 判读完备性 — 绿

**结论：设计者的字节码判读成立。** 尾汇聚点确为 no-op，且不存在第二条隐蔽收尾出口。

复现：
```bash
javap -p -c analysis/rename-pipeline/renamed/a.class > /tmp/a.javap
# executeScriptInstruction 声明于 /tmp/a.javap:30960
```
关键偏移（`/tmp/a.javap` 内）：

| 偏移 | 指令 | 语义 |
|---|---|---|
| 15-22 | `iload_2; iconst_3; iadd; length; if_icmpgt 2401` | 入口门 `cursor+3 > len ⇒ 2401` |
| 2386-2400 | `getfield scriptCursor; getfield dialogScriptLine; length; if_icmpge 2431; return` | 尾检查：`cursor >= len ⇒ 2431:return`，否则 `2400:return` |
| 2401-2430 | `iconst_4; putfield dialogPhase; scriptLineFlags[cur]=true; lockCameraOn(0)` | **只由偏移 22 进入** |
| 2431 | `return` | 尾检查 `>=` 分支的落地 |

- 方法内只有两个 `return`（2400、2431）；所有 op 臂均 `goto 2386`（102/322/337/360/384/425/566/660/1182/1242/1453/1687/1799/1967/2000/2106/2310/2353/2362）；
- 写 `scriptCursor` 的点（86/208/563/657/1179/1224/1450/1684/1784/1964/1997/2103/2215/2350/2383）**全部**汇入 2386；
- 跳 2401 的**唯一**来源是偏移 22。

源码侧同一结论：`reference/src/deobf/a.java:8596-8610`
```java
this.scriptCursor = var2 + 1;
}                                   // if (var2 + 3 <= len) { … dispatch … }
if (this.scriptCursor < this.dialogScriptLine.length()) {
   return;
}
} else {                            // 入口门 else：phase=4 + flags + lockCameraOn(0)
   this.dialogPhase = 4; …
```
即 `cursor >= len` 时**落到方法尾隐式 return**，不写任何字段。⇒ 尾部 no-op，PE-01 的前提为真。

> 但「结构可达」≠「条件可满足」。见 F2：`cursor >= len` 在现有 68 槽上**一次也不成立**。

### F2 PE-01 的「掩盖机制」 — 黄

**结论：掩盖机制本身成立且解释力充分；但作者据此得到的「每槽命中」结论与其自身事实矛盾，且漏掉了「当前生产影响 = 0」这一关键结论。**

已实测确认（A 级）：

| 事实 | 证据 |
|---|---|
| 夹具槽 6 = `"TAK_53_58 \u{1e}"`，11 字节 | rustc `include_str!` 实测 |
| `\'\u{1e}\'.is_whitespace() == false`（Rust） | 同上；故 `.lines().map(trim)` **不剥** `0x1E` |
| `dialogue_table()` 侧为 10 字节 | `crates/game-solver/src/script_abs.rs:158-166`（`split("\u{1e}\n")` + `trim_end_matches('\u{1e}')`） |
| `Obs` 无 `dialog_phase`、无 `ticks` | `crates/game-solver/src/script_abs.rs:73-83` 字段仅 `cursor/index/flags/stats[8]/facing/px/py/entities` |
| 每行末游标 = `len-1` | 实测：67 个非空行**全部**以 `" \x1e"` 结尾（无例外） |

`TAK` 在 `"TAK_53_58 \u{1e}"` 上解析：第二参以空格（下标 9）为止 ⇒ `cursor = 10 = len-1` ⇒ `10 >= 11` 为**假** ⇒ **尾门不触发**。
由于每行最后一个分隔符都是那个尾空格（实测 67/67 行），**任何槽的末命令都不会让 `cursor` 到达 `len`**。

因此：
- (a) 侧（GameScene，11 字节）走**下一拍入口门**（`10+3 > 11`），与 Java 逐字段相同；
- (b) 侧（ScriptEngine + 10 字节表）走**本拍尾门**，但写出的 `phase/flags/camera` 与 Java 的入口门**同值**，且 `cursor` 不变
  ⇒ `Obs` 恒等 ⇒ `assert_eq!` 恒绿。**掩盖机制解释成立**。

矛盾点：提案 `:237` 写「末命令消费后 `script_cursor == line.len()` **恒成立**」，而 `:255-260`（§B.4）自己写「(a) 行 = 11 字节、`cursor=10`」。
前者是 10 字节口径，后者是 11 字节口径，**同一份语料不能同时成立**。`:177` 甚至已经正确写出「行尾 TAK 的收尾必然发生在对话结束后的下一拍」，
与 `:15`/`:237` 的「同一拍、整页跳过」直接冲突。⇒ PE-01 的**后果证据三条（槽 6 / 槽 0 / 槽 65）在现有口径下全部不成立**。

漏掉的机制：**A/B 对拍之所以绿，不只是「相位不可见」，而是因为尾门与入口门写的是同一组字段值**（`phase=4`、`flags[cur]`、锁相机）。
即使把 `phase` 纳入观测，只要两侧最终都经过入口门一次（驱动循环会继续调用到 `(index,cursor)` 不变），终态相位也相同；
真正的差异只有**拍数**（同一 tick vs 下一 tick）。所以 §B.4 的「扩 `dialog_phase`」**不足以**捕获 PE-01，
必须比对**逐 tick 序列**（`ticks` 只是其中一部分）——这一点提案在 §B.4 推论里写对了，但在 §D.4 的可选清单里只列了三项。

### F3 §B 的 13 处不一致 — 黄

抽验 6 处（`END`/`LAY`/`ROS_4`/`CES`/`RES`/`ADD`），逐条给判：

| 编号 | 提案主张 | 复核 | 判定 |
|---|---|---|---|
| PE-01 | engine 尾门虚构 | 成立（F1/F2），但**当前不可触发** | 主张真、后果假 |
| PE-02 | engine `END` 调 `host.exit_application()`，Java 无 | 成立：`a.java:8516-8520` 只有 `gameMode=20; dispatchModeEntry(); scriptCursor=…`；`crates/game-core/src/script.rs:493-497` 确调 `exit_application` | ✅ |
| PE-03 | engine 把 `m_119`（存档）接成 `reload_floor_entities`（装载） | 成立：`a.java:9379` 统计存活实体写 `f_byte_arr2_05[floor]`；`a.java:9455` 才是装载；`script_ops.rs:68-74` 走 `save_floor_state` | ✅ |
| PE-04 | engine `ROS_4` 缺 `(byte)` | 成立：`script.rs:390` 直写 `facing_direction`；`script_ops.rs:300` 有 `(v as i8) as i32`。**但锚点 `a.java:8375` 是 ROS_1 的 `playerPixelY`，不是 ROS_4**（ROS_4 在其后 case 4） | 主张真、锚错 |
| PE-05 | engine `CES` 用静态 `type_categories` | 成立：`a.java:8395-8428` 用 `switch(this.renderCategoryByType[type])`，该表在 `m_122` 尾（`a.java:9679-9682`）按层改写；`script.rs:407-410` 用静态表 | ✅ |
| PE-06 | 「GameScene 对」，`inventory_entry_count` 由 GameScene 正确实现 | **不成立**：`inventory_entry_count` 全仓只出现在 `script.rs:77/126/459` 与 solver 注释，**GameScene 无此字段/无写点**；GameScene 反而多写 `self.persist.journal.clear()`（`script_ops.rs:114`） | ❌ 方向反了 |
| PE-07 | 两侧都缺 `initCellGrid` | 成立（`a.java:8444-8470` vs `script_ops.rs:120-143`） | ✅ |

「GameScene 与 Java 一致」的**四点**（END/LAY/ROS_4/CES）经逐行核对**确实一致**——不是「两边都没移植」：
- END：`op_end`（`script_ops.rs:269-276`）= `transition_to(Ending)` + `set_dialog_char(1)` + 对 `overlay_text` 的 1 字符限版，
  正对应 `dispatchModeEntry` case 20 的 `typewriterPos=1` + `layoutWrappedText(overlayText,…,1)`（`a.java:4442-4448`）；
  逐帧重排在 `scene_paint.rs:816-836` 用 `texts[260]`，亦对应 `a.java:3039-3080`。
- LAY / ROS_4 / CES 如表中所述。

但「一致」不能上升为全局断言：`RES` 是**反向偏差**（GameScene 缺 `inventoryEntryCount=0`、多 `journal.clear()`），
`ADD` 是**双侧共同缺失**（`initCellGrid`）。⇒ 回答用户 Q2：**不成立；生产路径确有反向偏差，且提案把方向写反了一条。**

### F4 「S4a 生产零变化」 — 黄

分两层看：

- **S4a 的「生产零变化」字面成立且已被正确限定**：`:551` 风险行写明「不在生产路径上（生产仍走 GameScene）⇒ 生产零风险」。
  这一点复核无异议（`scene_dialog.rs:117` 是唯一生产入口，走的是 `GameScene::execute_script_instruction`）。
- **但 S5 的「零变化」有反例**：S5 把生产切到 engine 后，
  `RES`（槽 0）会写 `inventory_entry_count=0`——这是 Java 有、GameScene 没有的状态；切换即改变生产状态。
  提案 §0.9 只承认「ADD 的 `initCellGrid`/`floor_assets` 守卫、END 的 case 20 可观测面」两类，**漏了 RES 这一条**。
  同理 `load_level_script` 的 `idx==31` 重载（PE-11）只有 engine 有，若 31 层可进入，S5 后行为变化。
- 构造的反例（无需运行即可判定）：槽 0 末段 `RES_0 GUT_1`；engine 路径下 `RES` 写 `inventory_entry_count=0`，
  GameScene 路径下不写 ⇒ 逐字段等价不成立 ⇒ 「S5 零变化」需要以 PE-06 已裁定为前提。

⇒ 判黄：不是「差异被两侧共同忽略」这么简单——`RES` 是**单侧多写**，而且方向被 PE-06 写反。

### F5 §A.3 的 W/P 判据 — 红

判据原文（`:237` 附近）：「含 `Mov`/`Lay`/`End`/`Gut` 的槽 = 20 槽 ⇒ 48/68 槽是纯 P 类」。
**该判据把「无相位跃迁」偷换成「纯表现」，与 §A.2.2 自己的表冲突**（`:168-169` 明写 `TAK` 是、`SEE` 是，二者都写 `dialog_phase`）。

实测（python 解析 `level_scripts.txt`）：

```
P 槽（48 个：非 W 槽）中，含 TAK 或 SEE 的 = 37 个
P 槽中完全不含 TAK/SEE 的 = [9, 11, 13, 18, 25, 37, 38, 39, 58, 59, 60]（11 个，其中 39/58/59/60 为空槽）
```

**要求给的反例槽：槽 42** = `TAK_171_171 SEE_7_10_172_172_0 SEE_7_10_173_173_1`（`flag=1`）。
按 §A.3 它是「纯 P」，但：
- `TAK` 写 `dialog_phase=1`（`a.java:8270-8278`），脚本机随即停在打字机相位等按键；
- `SEE` 无论 `find_path` 成功与否都写 `dialog_phase=6`（`a.java:8493-8515`：case 1 无 break，落穿 case 0/default），
  且 phase==6 时打字机还要 `camera_at_target()` 才推进（`scene_dialog.rs:118-121`）。
⇒ 槽 42 **必须 tick**（至少 2 个相位 + 相机帧），`find_path` 失败只影响 `path_arrows_visible/walk_phase`（表现面），不影响「需要 tick」这一结论。
再叠加：phase 4（相机收尾）本身也要等 `camera_at_target()` 才回 Map（`scene_dialog.rs` 的 `4 =>` 支），
所以「免 tick」在最宽松的意义上也只有 5 个槽成立，**不是 48**。

⇒ 判红：不是措辞问题，是**能力主张的量化错误**（48 → ≤11，且 37 个槽直接反例）。
提案 §F.1 第 3 条已自承「这是可达性判据、不是相位收敛证明」，但 §0.1/§0.7 的收益叙述仍建立在 48/68 上。

### F6 分类表完备性（最要命） — 红

按 `ScriptOp` 全部变体逐条核对「标成纯表现 vs 实际写状态」：

| 指令 | §A.3 归类 | 实际写面 | 判定 |
|---|---|---|---|
| `TAK` | P | `dialogPageStart/End`、`scriptCursor`、`dialogPhase=1`、`openDialogPage`（`a.java:8270-8278`） | **漏（相位+页状态）** |
| `SEE` | P | `dialogMoveX/Y`、`dialogPageStart/End`、`seeModeFlag`、`pathArrowsVisible`、`walkPhase`、`dialogPhase=6`（`a.java:8493-8515`） | **漏** |
| `CES` | P | `spawnEntity` + `markEntityDesRemoved` + 相机居中 + `scriptStepDelay`（`a.java:8395-8428`） | **漏（世界态：增实体）** |
| `DES` | P | `markEntityRemoved`（`a.java:8320-8344`） | **漏（世界态：删实体）** |
| `ADD` | P | `m_119`×2 + `m_122`×2 + `spawnEntity` + `initCellGrid`（`a.java:8444-8470`） | **漏（世界态：楼层/网格）** |
| `SWD` | P | type81→type4、`entityVisible/State/AnimRow`（`a.java:8345-8359`） | **漏** |
| `ROS_2/5/6` | P | 装备/HP/atk/def 写面（`a.java:8368-8394`） | **漏** |
| `GIN_1` | P | `gainGold` → 金币 + 弹字（`a.java:8429-8443`、`pickupItemType`） | **漏** |
| `GIN_0` | P | `pickupItemType` 23 臂：钥匙/atk/def/hp/tutorial/itemStack（`a.java:7534-7648`） | **漏** |
| `RES` | P | 整轮重置：钥匙/itemStack/gold=4/alchemy/atk/def/hp=400/`inventoryEntryCount=0`（`a.java:8476-8492`） | **漏** |
| `GLV` | P | **无副作用**（`a.java:8473-8475` 只 parse+advance；`script_ops.rs:261-264` 空体） | ✅ 归类正确 |
| `SMS` | P | 覆盖层文本/动画（表现面，但仍是 `overlay_text` 状态） | 边界（建议标 P-带状态） |
| `MVS` | — | 语料 0 次 | 不适用 |
| `MOV`/`LAY`/`END`/`GUT` | W | 相位/终止/换层 | ✅ |

量化：48 个「P 槽」中 **43 个**含上表至少一条有状态指令；真正的纯 P 槽只有 **5 个**（37=`GLV_1`，39/58/59/60 空）。
⇒ 若按 §A.3 实现「P 类 fast-forward」，会跳过 `CES/DES/ADD/SWD/ROS/GIN/RES/TAK/SEE` 的状态增量 →
**求解器会给出错误的可达解**（静默 bug，且 `Obs` 不含这些字段时对拍也照绿）。这是本次复核中风险最高的一条。

### F7 测试矩阵强度 — 黄

- 方向正确：提案已自警「2×2 矩阵成立的前提是先统一语料口径并把 `phase`/`ticks` 纳入观察面，否则只是把同一盲点复制四份」（§B.4 推论），
  这句判断经复核成立。
- 但 `Obs` 扩容清单（§D.4 只列 `+dialog_phase +ticks +path_arrows_visible`）**不够**：PE-01 在终态上不可见，
  只在**第几拍**可见 ⇒ 必须逐 tick 序列化；而「谁写的门」差异要靠 `phase` + `ticks` 才能定位。
  另外 `dialog_page/_end`、`script_delay`、`dialog_target`、`walk_request_flag`、`pending_floor/transitioning`、`overlay_text`、
  相机锚点、实体 route 状态都缺，PE-08/09/10 类差异同样会隐身。
- 建议最小清单（按 PE 覆盖）：`dialog_phase`（每 tick）、`tick` 序、`script_cursor`、`script_line_flags[68]`、`dialog_page`/`page_end`、
  `script_delay`、`dialog_target(x,y)`、`walk_request_flag`/`path_arrows_visible`/`walk_phase`、`pending_floor`/`floor_transitioning`、
  `overlay_text`、相机 `(cam_x,cam_y,ease_target)`、玩家 `(px,py,facing,anim_row)`、实体集（type/state/anim_row/route）、
  `inventory_entry_count`/`item_stack_size`/钥匙/`gold`/atk/def/hp。
- 另：矩阵是「2 执行器 × 2 handler」= 4 格，而**生产只有 1 个执行器**（GameScene）。第 4 格（engine×GameScene-host）
  才是 S5 前的真正验收格；提案未点明哪一格是切换闸门。

### F8 诚实性 — 黄

优点（应记）：
- 全文带 A/B/C 分级、`PE` 编号、行锚、复现命令；§0.9 主动承认 END case20 / ADD 只到 B 级；§F.1/§H 自承 48/68 不是收敛证明、
  GUT 闭包未算；§B.5 把两处争点**拒绝凭代码定**、要求 trace。这在同类提案里属高水位。
- §0.9 明确「结构改造不产生新能力」，没有把管线化包装成求解能力提升。

问题：
1. **把未验证的推断写成结论**：`:237`「每槽命中」「恒成立」，与 §B.4 事实冲突；`:15`「槽 6 对话页整页跳过」在现有口径下不可能发生。
2. **把「收尾门」的锚挂在 `a.java:8574-8580`（`script.rs:511` 注释）**，而该区间与尾门无关；这是伪锚，且是 PE-01 的立论基础之一。
3. **PE-06 方向写反**（「GameScene 对」），会误导 S4b 删掉 engine 唯一正确的那一条。
4. **§A.1 第 22 行 `ROS` 默认子码「不取参、不推进 ⇒ 下一拍重解析同一 token」**与 Java（`a.java:8368-8381` 无条件 `scriptCursor=lastDelim+1`）
   及自身代码（`script_ops.rs:542` `RosNoop { cursor: v2+1 }`）都矛盾——纯文档错，但会误导 handler 实现。
5. **工作量/调用点数字三处不一致**：正文「9 处」、表格相加 28、实测 grep **22 处**（见 F9）。

⇒ 判黄：不是「包装能力」，而是**关键结论的证据强度与结论强度不匹配**（C/B 级推断被写成 A 级事实）。

### F9 内建执行器可删性 — 黄

- **调用点实测 22 处**（`GameScene::execute_script_instruction`，不含注释与签名锁）：
  `scene_dialog.rs:117`（生产唯一入口）、`scene_solver_api.rs:60/149/166/544`、
  `scene_tests/slot_equip_scene_tests.rs` 14 处、`scene_tests/entity_route_tests.rs:10/73`、
  `script_ops.rs:197`（MOV 实体臂尾递归）。另 engine 自身 3 处：`script.rs:254`、`entity_route_tests.rs:255/267`；
  `scene_solver_api.rs:97` 是签名锁（`let _: fn(&mut GameScene)`）不是调用。
- 内建 impl 本身**没有**被非脚本路径复用：`op_*` 的外部调用只有 `scene_tests/gold_popup_tests.rs:11`（测试）。
  所以「删 `scene_script.rs:14-36`」在编译层面可行。
- 但删不掉的不是这段函数，而是**它依赖的私有面**：`op_*` 的 23 个函数体读写 `self.view/dialog/entities/persist/images/popups/script_line_flags/...`，
  `ADD` 还依赖 `floor_assets[...].is_some()` 守卫（`script_ops.rs:120-143`）；这些都不在提案给出的
  `WorldHost`/`PresentationHost` 两个 trait 的职责表里（§C 的接口只有 `step_delay_for`、`save_floor`、`spawn_entity`、
  `snap_camera_to_player`、`exit_application` 等少量方法）。⇒ **S5 的 ~650 行「移动」实为「重写 + 字段迁移」**，
  且必须先设计一个 `GameSceneHost` 适配器把上述私有面借出去（Rust 借用层面还需拆分 `&mut self`）。
- 结论：可删，但代价被显著低估；S5 表应补「host 适配器字段迁移清单 + GameScene 借用拆分方案」，否则该步不可执行。

### F10 证据纪律 — 黄

- `git show --stat 523c04f`：**仅 1 文件、704 insertions**，无代码/资源/门禁改动。A/B/C 分级总体守纪，
  C 级（`ROS`=rotate 之类）留在假设表、未混入结论。
- 行锚抽验 8 处：
  - 准：`a.java:8273`(TAK)、`8361`(LAY)、`8368`(ROS)、`8395`(CES)、`8444`(ADD)、`8476`(RES)、`8493`(SEE)、`8516`(END)、`8521`(SMS)、`8320`(DES)；
    `script_ops.rs:68/114/120/269/300/542` 亦准。
  - 错：**`a.java:8375` 被 PE-04 当作 ROS_4 的 `(byte)` 截断点，实际是 ROS_1 的 `playerPixelY`**；
    **`a.java:8574-8580` 被 `script.rs:511` 注释当作「Java 尾门」，实际该区间是 DES 负参/移除逻辑**。
- 自证问题：A/B 对拍用 `side_b` 驱动 `ScriptEngine`、`side_a` 驱动 `GameScene`，两者**共享 `Obs` 与语料**，
  而 `Obs` 恰好不含差异维度——属「用同一套观测面证明自己」。提案 §B.4 自己指出了这一点，算已止血。

### F11 红线夹带 — 绿

- `git show --stat 523c04f`：只新增 `docs/superpowers/proposals/2026-11-16-script-command-pipeline-proposal.md`（704 行）。
- 未触及 `original/`、`reference/`、`gates/`、`data/status/baselines/`；commit message 含 `Evidence:` 与 `Confidence: 88`。
- 无阈值调整、无金样放宽（无任何代码/基线文件变更）。

### F12 定级复核 — 黄

**PE-01 定 P0 不正确。** 分两个影响面：

| 影响面 | 现状 | 判 |
|---|---|---|
| 生产游戏 | engine 不跑生产；且即便跑，当前 68 槽行含 `0x1E`，末游标 `len-1` ⇒ 尾门**不触发** ⇒ **影响为 0** | 无影响 |
| 求解器 | `side_b`（10 字节表）每槽末命令触发尾门，但只改 `phase`/`flags`/相机；`flags` 与 Java 入口门同值、`cursor` 不变 ⇒ **终态与 `Obs` 全等** | 无结果影响 |
| 潜在风险 | 若 S7 统一口径时剥离 `0x1E`（Java 保真口径），尾门立即激活：`TAK` 的 `phase=1` 被改写为 4 ⇒ 对话页整页跳过 | 届时才是 P0 |

⇒ 这是「**真 bug 但影响面被夸**」的典型：结构缺陷值得修（S4a 必做），但定级应为 **P1（口径依赖的潜伏缺陷）**，
且在文档中必须写清「当前不触发、剥离 `0x1E` 后触发」。把「P0 + 每槽命中 + 整页跳过」当作现状，
会让后续排期与风险沟通失真（例如误以为生产已经在丢对话）。

---

## 复现命令（全部只读）

```bash
# 1) 字节码
javap -p -c analysis/rename-pipeline/renamed/a.class > /tmp/a.javap
grep -n "private void executeScriptInstruction" /tmp/a.javap        # 30960
awk '/executeScriptInstruction/,/parseScriptInt/' /tmp/a.javap | grep -n "2386:\|2397:\|2401:\|2431:"

# 2) Java 源码
sed -n '8596,8612p' reference/src/deobf/a.java                      # 尾检查 + 入口门 else
sed -n '8516,8520p' reference/src/deobf/a.java                      # END 无 exit
sed -n '9379,9455p' reference/src/deobf/a.java                      # m_119 存档 / m_122 装载

# 3) 语料不变量：67 个非空行全部以 " \x1e" 结尾 ⇒ 末游标恒为 len-1
python3 - <<'EOF'
raw=open('crates/game-core/tests/fixtures/level_scripts.txt',encoding='utf-8',newline='').read()
lines=raw.split('\n')
print(len(lines), all(l.endswith(' \x1e') for l in lines[:-1] if l.strip('\x1e')))
EOF

# 4) 真正纯 P 槽只有 5 个
python3 - <<'EOF'
raw=open('crates/game-core/tests/fixtures/level_scripts.txt',encoding='utf-8',newline='').read()
L=raw.split('\n'); W={0,1,2,3,4,5,7,8,10,12,16,19,23,30,31,56,57,61,62,65}
ops=lambda i:[t for t in L[i].strip('\x1e').split(' ') if t]
world=('CES','DES','ADD','ROS','GIN','SWD','RES','TAK','SEE')
pure=[i for i in range(68) if i not in W and not any(o.split('_')[0] in world for o in ops(i))]
print(pure)
EOF

# 5) 调用点计数
grep -rn "execute_script_instruction" crates/game-core/src --include=*.rs | grep -v "fn \|//" | wc -l
```

## 附：与提案一致、予以确认的部分

- 68 槽 / 296 指令 / 各 opcode 计数（TAK 81、MOV 47、GUT 7、DES 40、SWD 3、LAY 1、ROS 30、CES 48、GIN 4、ADD 1、GLV 1、RES 1、SEE 30、END 1、SMS 1、MVS 0）
  与含 MOV/LAY/END/GUT 的 20 槽清单**完全一致**；
- `m_119`=存档 / `m_122`=装载、`renderCategoryByType` 是可变世界态（仅 `[72]` 按层改写，`a.java:9679-9682`）——成立；
- engine 侧 `exit_application`、`m_119→reload_floor_entities`、`ROS_4` 缺 `i8`、`CES` 静态类目表 —— 4 条伪造/错接全部证实；
- GameScene 在 END/LAY/ROS_4/CES 四点与 Java 一致（非「两侧都没移植」）——成立。
