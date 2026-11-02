# 对抗复核 B：件2 工具栏同源 + 件3 两段式读档（攻击性审查）

> 复核员：对抗复核员 B（remake-verifier 纪律）
> 日期：2026-11-02
> 对象 commit：`9c698e1`（件2 工具栏同源 + 件3 两段式 UX 接线 + UI 面审计）+ `cbcb647`（件3 两段式 core 面；其中 menu_family 四钉随 `02b7413` 入史，归属本批）
> 用户裁定基线：读档行内二次点按（点=选中看信息，再点=加载）；按钮"几个点不形象"→补文字。
> 纪律：全程亲做，不改产品码；红/黄/绿 + 证据（A/B/C 分级）。破坏性验证在 `/tmp/xr-baseline-b`（HEAD 9c698e1 干净 worktree）进行，验后 `git checkout --` 复原，worktree 与主树产品码零改动。

## 0. 结论速览（总判）

**总判：条件通过——件2 修复为真、件3 核心序列为真，但审计表"存档槽=两段式"是虚登记（红），
且两段式存在自述失实面（row_hit 死码/滑动打断缺失）与零阈值连击面。建议批二收口前处置 R1–R3。**

| 维度 | 判定 |
|---|---|
| 件2 工具栏同源 | 🟢 真（格矩形单源，203px 真红复现，钉有牙）/ 🟡 label↔功能绑定双源采样 |
| 件3 两段式序列 | 🟢 tap1/tap2/换行/打断语义与裁定一致（钉有效）/ 🟡 连击零阈值、滑动不打断 |
| 全 UI 面审计表 | 🔴 存档槽虚登记 + 漏面清单见 §3 |
| 预览信息正确性 | 🟢 数据绑定正确 |
| 钉覆盖面 | 🟡 专项钉有效但"提示面"零钉（D3 实证）；"17 钉"归属见 §5 |
| 本体测试 | 🟢 干净 HEAD：core 318 / common 45 / android 180 全绿（与自述吻合）；共享树当前红=复核A WIP 所致（见 §7 通报） |

## 1. 两段式序列（tap1/tap2/打断/连击边界）

实现链（`step_boot`，`shell/android/src/runtime.rs:510` 起）：EquipDetail 态 tap →
`machine.pointer_hit` → `SlotSelect::tap_row_two_stage`（`crates/game-core/src/menu_family.rs:682`）。

| 攻击问 | 实况 | 判定 |
|---|---|---|
| tap1=Selected+预览+微提示？ | tap1：`selected=Some(row)`+`cursor=row`（行高亮+详情预览跟随）+"再点加载"（menu_family.rs:974 挂 paint_list） | 🟢（空槽例外：无效槽无预览面板可看，提示照旧邀点——原版保真面，微瑕） |
| tap2 同行=执行？ | `selected==Some(row)` → 清选+返回 true → 壳直发 `Press(-5)`=加载（runtime.rs:502-504） | 🟢 |
| tap 不同行=换选不执行？ | `tap_row_two_stage(j)`：selected 换 Some(j)，false（menu_family.rs:687-689） | 🟢 |
| tap1 后方向键/BACK=取消选中？ | `key != 0 → interrupt_select()`（runtime.rs:535-539）：清选不执行；方向键同时动光标（预览随之切换） | 🟢（同拍 tap1+按键=选中即被清，tap1 看似无效——边角，🟡） |
| tap1 后滑动=取消选中？ | **否**——滑动在读档面被整体丢弃：`vp_gesture_feed` 仅 scene 生效（runtime.rs:622 `self.scene.as_mut()`），`resolve_title_gestures` 非标题态丢手势。滑动既不换选也不清选 | 🔴→🟡（自述"按键/手势=interrupt_select（滑动=换选不执行）"失实；"以为滑动取消了→再点该行=加载"误触链成立） |
| 连击边界/阈值？ | **零时距判据**：同帧两 tap（双指/触点抖动双发，`taps` 数组逐个处理）或任意间隔快速双击=执行加载。无抬手去抖、无最小时距 | 🟡（裁定"再点=加载"字面合规，但防误触目标被"过快双击/双指"绕过；建议 tap2 间隔门槛或 Down/Up 去抖，属裁定内可加固项） |
| 点行外=打断？ | 仅 y 向成立——`BootMachine::pointer_hit` **忽略 x**（paint.rs:290 `_x`）；横向盒外同 y 也算行命中。`row_hit`（带 x 限界，menu_family.rs:693）**零调用方零测试=死码**，commit 自述"走 SlotSelect::row_hit"失实 | 🔴（自述失实+命中面宽于画框；tap2 在盒外横点也能触发加载） |

**打断语义小结**：按键/BACK ✅；滑动 ❌（不打断）；同行二次点按=执行（裁定内）；无时距防护。

## 2. 工具栏同源真伪（核心）

| 断言 | 实况 | 判定 |
|---|---|---|
| `toolbar_cell_rect` 单源（绘制+命中共调） | **真**——paint（`shell/android/src/portrait.rs:473-481`）与 hit（`shell/common/src/controls_ui.rs:66-77`）共调同一 fn（controls_ui.rs:117）；改格矩形必同变（同函数=构造性同源） | 🟢 |
| 203px 双算缝错格真红复现 | **真**——旧 hit `((x-mx)*n/mw)` 在 x=150（203px/4 格边界）得 Mid(0)，旧 paint 格1 起点=w*i=150 → 错格实锤；D1 破坏性实录（§5）："界线归属右格 left: Some(Mid(0)) right: Some(Mid(1))" 与自述红一致 | 🟢（修复真伪=真） |
| 界线归属右格/无缝平铺 | 格 i 右界=格 i+1 左界（同式逐格），钉断言在 | 🟢 |
| label=controls_ui::label 单源 | 文本查询单源 ✅（portrait.rs:439 与 native.rs:1642 同 fn） | 🟢 |
| label↔格↔功能绑定单源 | **否**——paint 采样 `sc.controls().mid`（native.rs:1626-1639，paint 时），hit 解析采样 `st.rt.toolbar_mid`（native.rs:1565-1567，上一拍 step 快照，runtime.rs:413）；两采样点间存在一拍偏斜窗口，mid 列表变化瞬间格→功能可错位。且钉的 label 断言空洞：只测**测试本地数组** `label(fns[i]).is_empty()`（controls_ui.rs:142-144），与生产 paint 的 mid 列表零关联，绑定未被钉 | 🟡 |
| `mid_n=0` 守卫 | `hit_toolbar` 先判 `mid_n==0 → None`（controls_ui.rs:59-61），无下溢 | 🟢 |

## 3. 全 UI 面审计表审查（豁免面理由 + 批三登记 + 漏面对拍）

审计表（9c698e1 commit message）6 行逐行核：

| 表行 | 裁决复核 | 判定 |
|---|---|---|
| 读档/存档槽=两段式（本批） | **虚登记**——两段式只接 Boot 读档列表（step_boot）。**scene 侧存档槽（GameMode::SaveSlot，覆盖存档）与 scene 侧读档（EquipDetail）单点直发 confirm**：`scene_tap`（`shell/android/src/native.rs:417`）→ `select_row` + 直发 `confirm_key(-5)`，`GameScene::pointer_hit` SaveSlot\|EquipDetail 行命中（scene.rs:10342-10349）。破坏性更大的存档面（误触=覆盖进度）零防护 | 🔴 R1 |
| 设置行=豁免直通 | 用户裁定"点按切换"；值循环=单击语义、可逆即时——理由成立 | 🟢 |
| 对话选项=豁免直通 | **对象存疑**——grep 触屏分派面：kind 3/4 是否框（道具使用确认/商人报价）无图标点按分派（`overlay_active → PointerHit::None`，`blank_tap_key` 对 yes/no 返 None）；触屏下经工具栏键应答。"点按即选择"在代码中找不到对应点按面 | 🟡 |
| 商店买/不买=豁免直通 | FunctionId::Buy/Refuse 按钮语义成立；但 **Shop 行命中（-6 直发）未登记**（见漏面） | 🟡 |
| 点怪=两段式（cfcc9ad） | tap_cell 先例在位（tap1 选中/tap2 开战） | 🟢 |
| 装备替换/道具使用=批三续审 | **登记在**（审计表+commit message 留痕）；但"显式留痕"仅存 commit message——代码无 TODO、docs 无跟踪条目，易随时间丢 | 🟡 Y6 |

**漏面对拍**（grep 触屏分派清单 vs 表，按指令"漏面=红"）——未登记面：

1. 标题轮播 tap（title_geom → -5 直发确认，BUG-039 注）；
2. IntroCutscene 点空白=快进 -5（blank_tap_key，paint.rs:322 附近）；
3. InGameMenu 行 -6 直发（菜单命令单点执行）；
4. Shop 行 -6 直发；
5. 工具栏钮单点直发：Undo（撤销上一步=破坏性面）/Bag/Observe/Menu/Skip 等（native.rs:656-660 → perform_action）；
6. "对话选项"面本身在触屏分派中不可定位（见上表）。

其中 1/2/3/4 为低危直通（导航/可逆），5 的 Undo 属破坏性面。→ 🔴 R3（漏面=红，按指令；实际危害分级见上）。

**批三登记核查**：装备替换/道具使用已在审计表显式登记为"批三续审"（未漏）；留痕强度=commit message only（🟡）。

## 4. 预览信息正确性（选中槽数据绑定）

- 详情面板（楼层/三围/三钥/金币/装备）绑定 `self.cursor` 索引的 per-slot 数组（`paint()`：`self.valid[self.cursor]`、`floor_num[c]`…，menu_family.rs 详情段）；tap1 同拍设 `cursor=row`+`select_row` 同步，换行 tap 立即换预览。**预览=该槽数据，非默认值/他槽**（数组按槽 1:1 索引） | 🟢
- 换选中预览跟着变：方向键动 cursor、tap 换行动 cursor，预览均随 cursor | 🟢
- 边角：无效槽（"---"）tap1 无预览面板可看但提示照旧（"点=选中看信息"对该槽无信息可看）；core 对无效槽 -5 确认本就有 `valid[cursor]` 门（paint.rs:599）不会误加载 | 🟡（微瑕）
- 预览绑定无专项钉（换行换预览/数据归属未测）——由 slot_equip 像素钉间接覆盖 | 🟡

## 5. 两段式钉 / 单源钉覆盖面（破坏性抽查）

**钉清单**（"17 钉"归属核查：`app_settings.rs` 恰 17 测=件1/接线面；件2/件3 专项钉实为 3 个+row_band 既有单源钉，"17 个两段式钉/单源钉"口径与仓库实际归属不符——如实列清单，覆盖度按逐钉判）：

| 钉 | 面 | 破坏性验证 |
|---|---|---|
| `toolbar_cell_hit_label_same_source`（controls_ui.rs:141） | 件2 格矩形+界线 | **D1 已验**：hit 回退旧双算式 → 红（203px 边界 Mid(0)/Mid(1) 错格实录）= 真红签名复现，钉有牙 |
| `slot_two_stage_select_then_execute`（menu_family.rs:2068） | 件3 序列 | **D2 已验**：tap1 直接执行 → 红（"tap1=选中不执行"断言炸）= 钉有牙 |
| `slot_selected_row_shows_retry_hint`（menu_family.rs:2089） | 件3 提示**文本** | 只测 `retry_hint_text()` 返回值 |
| （无） | 件3 提示**绘制**面 | **D3 已验**：删 paint_list 挂点（menu_family.rs:974-980）→ core 318 全绿——**提示面零钉**。Evidence 自报 A 级"'再点加载'提示面"过报（实为代码挂点目视+文本钉，绘制链无钉） |
| `hit_toolbar_*`（controls_ui.rs:95-104） | 件2 左中右分区 | 基线钉 |
| row_band 单源钉（menu_family.rs:1953 等，C-1 既有） | 行带几何 | 既有 |
| `touch_ux_rows_tests`×2 + `value_row/hidden_rows`（件1 四钉） | 设置行 | 件1 面（本批范围外，随 cbcb647 改写） |

- 破坏性程序：D1/D2 在 `/tmp/xr-baseline-b`（HEAD worktree）patch→跑→红→`git checkout --` 复原；D3 同法。主树产品码零改动。
- 基线（干净 HEAD 实跑）：`cargo test -p game-core` 318/318 ✅、`-p game-shell-common` 45/45 ✅、`-p game-android` 20 套件合计 180/180 ✅（8+12+17+1+5+7+6+4+16+9+36+20+4+2+4+11+14+4）——"android 180 全绿"自述**核实为真**（与自述逐数吻合）。

## 6. 发现汇总表（红/黄/绿）

| # | 级 | 发现 | 证据 |
|---|---|---|---|
| R1 | 🔴 | 存档槽（GameMode::SaveSlot，覆盖存档）与 scene 读档槽未接两段式，单点直发 confirm；审计表"读档/存档槽=两段式（本批）"虚登记 | scene.rs:10342-10349 + native.rs:417 + step_scene 无两段式调用（A 级） |
| R2 | 🔴 | commit 自述"EquipDetail 走 SlotSelect::row_hit"失实：row_hit 死码零调用；实际 pointer_hit 忽略 x=全宽命中，"点行外=打断"仅 y 向成立 | menu_family.rs:693（死码）/paint.rs:290（`_x`）/runtime.rs:510（用 pointer_hit）（A 级） |
| R3 | 🔴 | 审计表漏面（按指令漏面=红）：标题轮播/Intro 快进/InGameMenu 行/Shop 行/工具栏钮（Undo=破坏性）均未登记 | grep 分派面对拍（A 级清单） |
| Y1 | 🟡 | 连击零阈值：同帧双 tap/双指/快速双击=加载，无去抖 | runtime.rs:497 循环逐 tap 处理 + tap_row_two_stage 无时距（A 级） |
| Y2 | 🟡 | "滑动=换选/打断"自述失实：读档面滑动被丢弃（不清选不换选）→"以为取消再点=加载"误触链；tap1 与按键同拍=选中即清 | runtime.rs:622（scene-only）/native.rs:309-326（手势丢弃）（A 级） |
| Y3 | 🟡 | "再点加载"提示面零绘制钉（D3 实证）+ Evidence A 级过报；文案硬编码"加载"于共享 SlotSelect，批三接存档面需换"保存"语义 | menu_family.rs:710-712/974-980 + D3（A 级） |
| Y4 | 🟡 | 工具栏格→功能绑定双源采样（paint 时 vs 上一拍快照），一拍偏斜窗口；label 绑定钉空洞 | native.rs:1562-1567 vs 1626-1639 + controls_ui.rs:142-144（A 级） |
| Y5 | 🟡 | "对话选项"审计行在触屏分派中无对应面（kind 3/4 无图标点按）；裁决对象不明 | grep overlay 点按=空（B 级） |
| Y6 | 🟡 | 批三续审（装备替换/道具使用）登记仅存 commit message，代码/docs 无跟踪条目 | grep 批三=空（A 级） |
| Y7 | 🟡 | EquipDetail 态 slots=None 防御分支回落通用直通=静默单点执行（失防不报警） | runtime.rs:510-515 if-let 链（B 级） |
| G1 | 🟢 | toolbar_cell_rect 单源为真，203px 真红复现（D1），修复真伪=真 | D1 实录 |
| G2 | 🟢 | 两段式核心序列（tap1/tap2/换行/打断）与裁定一致，钉有效（D2） | D2 实录 |
| G3 | 🟢 | 预览=选中槽数据绑定正确，换选即换预览 | menu_family.rs 详情段 cursor 索引（A 级） |
| G4 | 🟢 | 干净 HEAD 三 crate 全绿，数字与自述吻合 | 实跑输出（A 级） |

## 7. 总判与给上游的行动项

**总判：条件通过。** 件2 同源修复与件3 核心序列经破坏性验证为真（D1/D2 钉有牙）；但按 remake-verifier 纪律，**审计表虚登记（R1）与自述失实（R2）不可带病收口**——"测试全绿"不等于面覆盖完整（§3 漏面对拍）。

给上游（批二作者）行动项：

1. **R1**：scene 侧存档/读档槽接件3 规则（复用 `tap_row_two_stage`+`interrupt_select`；存档面提示文案用"再点保存"），或把审计表改口"仅 Boot 读档"并显式记批三——二选一，禁止维持虚登记。
2. **R2**：要么接线 `row_hit`（收 x 限界），要么删死码+改 commit 自述；"点行外=打断"补 x 向语义。
3. **R3**：审计表补漏面 6 项（至少工具栏 Undo 破坏性面要裁决）。
4. **Y3**：补提示绘制钉（ops 面断言即可）；文案按面参数化（批三前置）。
5. **Y1/Y2**：向用户请裁连击时距/滑动打断是否要加固（裁定边界内的体验加固，不擅自加）。
6. **Y4**：paint/hit 的 mid 列表改同拍同源采样（或 hit 解析复用当拍 controls()）。

**⚠️ 协作面通报（非本批缺陷）**：共享树当前 `gates rust-tests` **红**——`runtime_key_feedback_reads_active_machine` 失败，系**复核A 未提交 WIP**（`app_settings()` 改读 `rt_key_haptic` 保值态）与该测试"直接改行再读 key_feedback"语义冲突所致；干净 HEAD 该测全绿。复核A 收口时需同步改 `key_feedback` 读取口径或该测试。本次复核全部测试在 `/tmp/xr-baseline-b` 干净 worktree 实跑，未受 WIP 污染；主树 WIP 未动。

### diff-case 登记（模板）

```markdown
## diff-case TWO-STAGE-01
- 输入序列: scene 内菜单→存档→槽行单点
- 现象: 单点即 confirm(-5) 直发=覆盖存档（无 tap1 选中态）；首分歧点=首 tap 即执行
- 分类: 移植错（触屏 UX 批二接线漏 scene 面）+ 审计表虚登记
- 处理: 回批二作者（R1）
- 门禁: cargo test -p game-android（干净 HEAD）→ 全绿（无钉盖此面，绿≠覆盖——正是本次红）
```
