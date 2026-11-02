# scene.rs 真模块拆解方案 v3（轮 2 独立对抗复核修复稿——可落地）

> 2026-11-02。v1 经轮 1 三角度对抗（A 架构纯度/B 契约安全/C 可行性）——**三件同向
> 攻出基线陈旧+验证枪引错两大要害**，本 v2=重基线+纠枪+补缺。轮 2 独立复核修复后=v3 落地。
> 证据源：docs/findings/reviews/2026-11-02-plan-attack-{architecture,contract,feasibility}.md。
>
> 【轮2修复·2026-11-02】独立复核（不采信轮 1 结论、逐项重新取证）：§0-§5 关键声称全部
> 重验——git show 313375b/76cb6c2/8a455dd/efc6fe5 实证、tools/state_baseline.py --check 实存
> 且 gates/cli.py 确未挂（`gates status` 13 门禁无此项）、47/56/31/49/628/171/2777/scene_menu.rs:6
> 等逐个复现（详见各行 ✅ 注）。攻出 3 处 v2 陈旧/错误（E1 enum+22 op 归位面、E2 too-many-args
> 现数 8、E3 paint 副作用 3 字段不在金样面）+ §6 四问全部裁定写入。v3=就地可执行。

## 0. 重基线（v1 最大错误=把已办当待办）

| v1 说"待做" | 实况（A 级 commit 实证） |
|---|---|
| 步 2 Entity 聚合 | ✅ **已全量**（313375b/76cb6c2，entity.rs:18-56，余量≈0）【轮2复验 ✅：commit 实存；entity.rs:19 struct Entity 11 字段】 |
| 步 3 ScriptOp 命令模式 | ✅ **已落**（8a455dd：enum+parse_op/execute_op 16 op 分离，scene_script.rs:41-221）【轮2修复 E1：**enum ScriptOp 在 scene.rs:827，22 个 op_* 执行函数在 scene.rs:2021-2271+（~250 行）**，仅 parse_op(:41)/execute_op(:192) 在 scene_script.rs——"scene_script.rs:41-221"不含 enum 与 op 实现体；"16 op"=16 标签，实现函数 22 个（16+ROS 6 子码+DES 2 形态）。归位面=scene.rs enum+22 op impls 与 scene_script.rs parse/execute **3 处汇 1 文件**】 |
| 步 4 restore_rules 表 | ✅ **已落**（efc6fe5：RESTORE_RULES scene.rs:726）【轮2复验 ✅：表 scene.rs:726、执行器 apply_restore_rules scene_persist.rs:406——"现分居两文件"属实】 |
| "TraceState 契约（步 0）" | ⚠️ **命名错**：全仓无此类型——真机制=`dump_scene` 49 项适配+fields.py 生成式契约+line_order_freeze（SoA→struct 金样零漂移实证吸收力成立）【轮2复验 ✅：state_dump.rs dump_scene 恰 49 个 push(e_；fields.py dump_order()+line_order_freeze 声明在】 |
| 文件搬家"4 步" | 实为 **5 步**（今日 scene_script.rs 已落，v1 漏列）【轮2复验 ✅：f419b68/78822d4/bf780ac/838f5fb/5111171 五 move commit 实存】 |

**真实剩余工作**（全部）：
1. **文件归位**（move-only，【轮2修复 E1】非"小件"）：script_ops.rs=enum（scene.rs:827）+22 op
   impls（scene.rs:2021-2271+，~250 行）+parse/execute（scene_script.rs）3 处汇 1 文件；
   restore_rules.rs=表（scene.rs:726）+执行器（scene_persist.rs:406）汇 1 文件；
2. **域状态切片**（档 B-lite，**8-15 人日，v1 零估算**）：GameScene 散字段→域切片；
3. **验证基建纠偏**：state_baseline --check 挂 gates（现未挂）【轮2复验 ✅：tools/state_baseline.py
   --check 实存；grep gates/ 零命中，`gates status` 13 门禁无此项】+【轮2修复 E3】dump 面扩
   paint 副作用字段（strip_frame/backdrop_scroll/bob_offset/bob_rising 现**不在** 49 项金样面，
   仅 openAnimTick/tileBob 在）——追加 push+金样重冻结（走 state_baseline 既有"行序变更=金样红
   需 lead 裁"协议）。

## 1. 目标架构（档位裁定=渐进，A/B 修正版）

**裁定依据**（A 实证）：档 B 整体过度——interact_with_cell 跨 7 域 208 hits，"传参不传整引用"
=参数面爆炸（与 too-many-args 永久保留红线互斥）；combat 纯计算已落 combat.rs（171 行零耦合）。
【轮2复验 ✅：interact_with_cell scene.rs:4025-4333（309 行，轮 1 计 208 hits/7 域口径成立）；
combat.rs 恰 171 行；activate_item scene.rs:2777 恰 3 调用入口（scene.rs:3004/3019、scene_menu.rs:347）】

```
GameScene（协调器：状态机+域装配）
├── entity.rs        ✅ 已落（AoS 聚合）
├── script_ops.rs    ✅ 命令模式已落（归位后：parse/execute+16 op 同文件）
│                    【轮2修复 E1：✅=逻辑已落，**文件待 V1 归位新建**（现无 script_ops.rs）；
│                     归位面=enum scene.rs:827+22 op impls scene.rs:2021-2271+（非仅 16）+
│                     parse/execute scene_script.rs——3 处汇 1】
├── restore_rules.rs ✅ 表已落（归位后：表+执行器同文件）
│                    【轮2修复：✅=逻辑已落，**文件待 V1 归位新建**（现无 restore_rules.rs）；
│                     表 scene.rs:726+执行器 scene_persist.rs:406】
├── scene_persist.rs → persist 收口（self 面实测仅 132 处——可收口）【轮2复验 ✅ 与轮1测量一致】
├── scene_paint.rs   → render 收口（⚠️ 只读前提不成立——见 §2 红线）
├── **item/effect**  新域：activate_item:2777 三入口+模式跳转污染 → 表驱动（仿 RestoreAction）
│                    【轮2裁定·边界：item 面板字段（item_*/overlay_kind_item/overlay_icon/
│                     overlay_item_name）归 **menu/UI** 域；本域只收效果逻辑+效果状态+表】
├── **menu/UI**      新域：30+ 字段（非 render 子域——A 修正）【轮2复验 ✅：menu_*/slot/options/
│                    item_*/encyc_ 族 30+ 字段口径可复现】
├── **dialog-VM**    新域：tick_dialog（现错居 scene_menu.rs:6）【轮2复验 ✅：恰在 scene_menu.rs:6】
├── **tap/input**    新域：tap_walk.rs 628 行+11 字段（已有雏形）【轮2复验 ✅：恰 628 行；11 字段=
│                    walkto_arrows/battle_undo_snap/ui_selected/ui_frame/auto_battle/tap_target/
│                    tap_reject/tap_bump/tap_followup_ready/tap_selected/tap_pending（scene.rs:545-573）】
└── 准域对账（已有，划界不重造）：undo_journal/RouteTracker/PopupRing/BgmState/WalkState
```

**域切片纪律**（B 修 R）：**一次性切禁双写**+每域**字段唯一归属表**（防"新两处真相"——
项目 gotcha 先例）+script.rs ScriptEngine/camera.rs harness 面同步/冻结条款（DC-9 漂移前科）。

## 2. 红线（修正版——v1 两处回退更正）

1. **零行为差**；游标推进语义逐字节等价（scriptCursor/lastDelimPos）——RED 钉集**引用现状**
   （script_surfaces_op_dispatch_cursor_parity 16 op 双面钉已在）【轮2复验 ✅：test 在
   scene_tests/entity_route_tests.rs:222】；parity 补薄变体（MOV 3/5参、
   DES 负值、ROS 子码）；**禁中间态字段化**；
2. **render 侧写状态=原版行为**（paint 内 strip_frame/backdrop_scroll/bob_offset/open_anim_tick
   ——A 红线）：render_adapter"只读"前提作废；paint 副作用清单随迁+金样钉；
   【轮2修复 E3·钉机制补全】副作用**时点不动、状态搬家、观测钉三件**：
   ① 变更点保留在 paint 内同一调用时点（条件分支内副作用=行为，禁抽离——tile_bob"重置拍
   也取 bob"（scene.rs:5215 注）类时序语义随迁，a.java 锚随注）；
   ② 字段归属 render 域（归属表显式划入，前序域切片禁触碰）；
   ③ 观测钉：**金样扩面**——strip_frame/backdrop_scroll/bob_offset/bob_rising 现不在 49 项
   dump 面（轮2复验：state_dump.rs 仅 openAnimTick:106/tileBob:105 两项 paint 族在面），
   V0 追加 push+金样重冻结（lead 裁）；兜底=帧哈希 31 夹具（render 漂移必现）+行为面矩阵补行。
   【原记保留】"paint 副作用清单随迁+金样钉"原句无机制载体，经 E3 补全后方成立。
3. execute_script_instruction/16 op 逐值对照（a.java:8220-8568）不动；too-many-args
   【轮2修复 E2：原记"6 处"陈旧——现数 **8 处**（game-core 全量 grep）：scene.rs:623
   load_floor_from_save、scene_paint.rs:8 paint_overlay_box、render.rs:730 draw_region_scaled、
   logo_anim.rs:125 register、menu_family.rs:120 draw_edge_patch、menu_family.rs:402
   paint_wrapped_lines、pathfind.rs:151 relax_path_neighbors、walk.rs:91 tick_stepping】
   不动（指名清单，防止随文件搬家漏看）；
   cellSlotTable/族表不并入 Entity；
4. **回退设计**（B #5 红补）：回退单元=**整步 commit revert 禁 fix-forward**；checkpoint=gates
   输出贴 commit；语义分歧=revert 至最近全绿。

## 3. 验证矩阵（C 纠枪——v1 引错主判据）

| 枪 | 实况 | 用途 |
|---|---|---|
| "金样 47 场景+35 帧哈希" | **口径错**：47 系纯 Java 三方（A==B==C）**不跑 Rust**；实数=**56 场景/帧夹具 31**【轮2复验 ✅：reference/oracle/scenarios/*.txt 恰 47；state-dump-golden.json scenarios 恰 56；*.frames.txt 恰 31】 | Java 侧参考回归 |
| **state_baseline --check** | **Rust 拆解主判据**（refactor-plan §7.1 换枪决议——v1 被回退）【轮2复验 ✅：refactor-plan.md §7 条件 1"主判据换枪"原文在】；**未挂 gates=先补挂**【轮2复验 ✅：实测未挂】 | Rust 行为面 |
| statediff 名集相等+行为面矩阵 | v1 漏（B #4）【轮2复验 ✅：tools/statediff.py:73-98 名集相等断言（2026-10-16 修静默绿）在】 | 完备性守卫 |
| 域级单测边界 | 只断言**可指回 a.java/trace 的语义不变量**；等价判据只认差分三件 | 防实现耦合测 |

【轮2修复·差分三件指名（§5.4 引用收口）】：①金样逐字节=state_baseline --check（56 场景）
②名集相等=tools/statediff.py ③帧哈希=fixture_frames 31 夹具（*.frames.txt）。
（"state_baseline --check+全部差分三件"原句句①③口径重叠，实为三件各指其枪。）

## 4. 执行序（重排——余量实估）

| 步 | 内容 | 估 |
|---|---|---|
| V0 | state_baseline --check 挂 gates（验证基建先行）+【轮2修复 E3】dump 面扩 paint 副作用 4 字段（strip_frame/backdrop_scroll/bob_offset/bob_rising 追加 push）+金样重冻结（lead 裁） | 0.5-1d |
| V1 | 文件归位（script_ops=enum+22 op impls+parse/execute 3 处汇 1；restore_rules=表+执行器汇 1，move-only）【轮2裁定 Q4：先行正确；估修正 0.5d→0.5-1d（E1 迁移量非小件）；判据=diff 纯移动（无逻辑行变更）+--check 绿】 | 0.5-1d |
| V2 | **域状态切片**（一次性切+字段归属表）：persist 收口→tap/input→dialog-VM→item/effect→menu/UI→render 副作用清单化 | **8-15d**（C 实估，分域见下） |
| V3 | 完成判据验收（§5） | 1d |

【轮2裁定 Q1·V2 内序】执行序成立，补一条硬前置：**V2.0=全表字段归属一次性裁定**（0.5d，
不是每域各裁各表）——①item 面板字段归 menu/UI、效果状态归 item/effect（§1 边界）；
②paint 变更 6 字段（strip_frame/backdrop_scroll/bob_offset/bob_rising/open_anim_tick/tile_bob）
显式划 render 域，**前序切片禁触碰**（保证"render 副作用清单化"殿后不返工）。
依赖核：item/effect 表驱动仿 RestoreAction ⇒ 必须在 V1（restore_rules 归位）之后 ✅序对；
dialog-VM 先于 menu/UI（tick_dialog 先脱 scene_menu.rs、menu 族字段后整切——同文件两次
触碰合法，字段唯一归属防双写）；persist 首切=低耦合（self 132）练"一次性切"纪律。

【轮2裁定 Q3·8-15d 分域分解】（B 估=面宽×字段数，C 估=实测行数/耦合度）：

| 域 | 内容 | 估 | 出口判据（可测） |
|---|---|---|---|
| V2.0 | 字段归属表全表裁定+双写扫描器 | 0.5d | 全表唯一归属打印；扫描零命中 |
| persist 收口 | scene_persist 扩域（self 132 面） | 1d | --check 绿；域级单测独立构造 |
| tap/input | 11 字段+tap_walk.rs 628 行域化 | 1d | 同上+touch_tap_guard_tests 绿 |
| dialog-VM | tick_dialog+dialogPage/dialogPhase（金样面字段） | 1d | --check 绿（此 2 字段在 49 项面）|
| item/effect | activate_item 三入口表驱动（仿 RestoreAction） | 2-3d | 新增道具不改 GameScene 主体（§5.1 git diff 断言）|
| menu/UI | 30+ 字段切片（最大面） | 2-3d | --check 绿+行为面矩阵补行 |
| render 副作用清单化 | 变更点留 paint+字段归 RenderState+金样扩面已钉 | 1-2d | 帧哈希 31 夹具绿+行为面矩阵补行 |
| 合计 | | **8.5-12.5d**+金样红缓冲 1-2d | ⇒ **8-15d 成立** |

【轮2裁定 Q4·V1/V2 先后】V1 先行正确：V2 不再整搬 V1 归位件（只在域内追加/字段迁移）；
重叠面仅 scene.rs（V1 搬走 enum/22 op/RESTORE_RULES，V2 搬散字段）=零重做。若倒序，V2 域
文件先立、V1 再汇入则二次搬家——禁。

## 5. 完成判据（C 补——可测断言）

1. **新增道具/新 op/新恢复特例不改 GameScene 主体**（域内闭合）——git diff 断言可测；
2. 域级单测独立构造（不 build 全 GameScene）；
3. 字段归属表全表唯一归属+双写扫描零命中；
4. state_baseline --check+全部差分三件绿。
   【轮2修复·指枪收口】=①`python3 tools/state_baseline.py --check`（56 场景金样逐字节）
   ②`tools/statediff.py` 名集相等 ③fixture_frames 帧哈希 31 夹具全绿——**贴实际输出**。

## 6. 待轮 2 复核修复问题（【轮2修复】四问已全部裁定——原问保留，裁定随附）

1. 域切片顺序（V2 六件）依赖关系核（item/effect 表驱动依赖 restore_rules 先例——顺序对？）；
   **【轮2裁定】顺序对，补硬前置 V2.0 全表字段归属一次性裁定（item 面板↔item/effect 边界、
   paint 变更 6 字段划 render 域禁前序触碰）；dialog-VM→menu/UI 同文件两次触碰合法
   （唯一归属防双写）**。详见 §4 Q1 注。
2. paint 副作用清单化的边界（原版行为保留 vs 渲染可测性）；
   **【轮2裁定】时点不动、状态搬家、观测钉三件：副作用留 paint 同调用时点（条件分支内
   副作用=行为，抽离=改时序）；字段归 render 域；可测性从观测钉来——金样扩面 4 字段
   （strip_frame/backdrop_scroll/bob_offset/bob_rising，V0 追加 push+重冻结 lead 裁）+
   帧哈希 31 夹具+行为面矩阵补行，不从搬副作用来**。详见 §2 红线 2 E3 注。
3. 8-15d 估算的分域分解（B/C 估算方法核）；
   **【轮2裁定】8-15d 成立：分域表 8.5-12.5d+金样红缓冲 1-2d（§4 Q3 表）；每域出口判据
   =--check 绿+归属表零双写+行为面矩阵补行；menu/UI（30+ 字段）与 item/effect（表驱动）
   各 2-3d 为大头，persist/tap/dialog 各 1d 为快切**。
4. 文件归位 V1 与域切片 V2 的先后（归位后 V2 还要再动同文件？）；
   **【轮2裁定】V1 先行正确；V2 不整搬 V1 归位件、重叠面仅 scene.rs 且搬的不是同物
   （enum/op/表 vs 散字段）=零重做；V1 估修正 0.5-1d（E1 归位量=enum+22 op impls ~250 行+
   parse/execute，非小件）；V1 判据=diff 纯移动+--check 绿**。

### 【轮2修复】新增复核注记（轮 1 未列）

- **E1**：§0/§1 ScriptOp 归位面纠错——enum 在 scene.rs:827、22 op impls 在 scene.rs:2021-2271+
  未归位（v2 "scene_script.rs:41-221"漏两者）；"16 op"是标签数、实现函数 22 个。
- **E2**：§2.3 too-many-args "6 处"陈旧——现 8 处（指名清单见红线 3）。
- **E3**：§2.2 "金样钉"缺载体——strip_frame/backdrop_scroll/bob_offset/bob_rising 不在 49 项
  dump 面；V0 金样扩面+重冻结补上（state_baseline 既有 lead 裁协议）。
- v2 §1 架构图 script_ops.rs/restore_rules.rs 的 ✅ 系"逻辑已落"非"文件已在"——已原地标注
  （文件待 V1 新建），防读者误跳步。

> 【轮2 抽验记录】313375b/76cb6c2/8a455dd/efc6fe5 git show 实证 ✅；RESTORE_RULES scene.rs:726
> ✅；parse_op :41/execute_op :192 ✅；dump_scene 49 push ✅；fields.py line_order_freeze ✅；
> tools/state_baseline.py --check 存在且 gates 未挂（gates status：13 门禁零命中）✅；
> 47/56/31 场景夹具数 ✅；tap_walk 628 行/11 字段 ✅；combat 171 行 ✅；activate_item:2777 三入口 ✅；
> tick_dialog scene_menu.rs:6 ✅；interact_with_cell 4025-4333（309 行）✅；
> script_surfaces_op_dispatch_cursor_parity ✅；refactor-plan §7 条件 1 换枪原文 ✅。
> 抽验方法：git show / wc -l / grep -c / python json 计数，命令可复现。
