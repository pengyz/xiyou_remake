# Bug 台账（Stage A 记录，Stage B 修复）

> 政策（D4）：Stage A 忠实保留原版行为（含 bug），代码内以 `BUG-xxx` 注释引用本表；
> 差分测试将其登记为"已知偏差"；Stage B 统一修复并回填"修复方案/提交"列。

| ID | 现象 | 复现 | 根因（待填） | 影响 | Stage A 处理 | Stage B 修复方案 | 状态 |
|---|---|---|---|---|---|---|---|
| BUG-000 | （模板）示例：某层踩到传送点偶发卡死 | 输入序列 … | 待分析 | 轻微 | 保留原行为 | 待定 | open |
| BUG-001 | `script` 资源是**死数据**：运行时从未读取它，真源是编译进 class 的内嵌数组 `this.n[]`；且资源文件与内嵌数组有 2 处内容差异（GUTS 第 27、63 行） | `grep 'getResourceAsStream' reference/seed/a.java` 无 "script" 命中；逐行 diff 资源 vs `this.n[]`（见 `docs/findings/script-dsl-semantics.md`） | 随包发的疑似设计文档副本，编译期被拍平进常量池 | 无玩家可见影响（行为以内嵌数组为准） | 保留资源原样，行为以内嵌数组为准 | 决定删冗余资源或反向以资源为准 | open |
| BUG-002 | `run()` 输入模式分支 `switch(var5_3.i)` 中 `int i` 字段全程无赋值（恒 0），case 0 恒定命中——疑似死代码 | 全文 grep 无该字段赋值点（见 `docs/findings/state-machine.md`） | 待字节码复核（反编译可能丢信息） | 无（死代码） | 保留原样 | 可安全删除 | open |
| BUG-003 | 【2026-10-16 重审·转正③语义自洽】DES 负参=全型清除臂（a.java:8314-8327 有分支），GUTS script63 语义自洽，非异常。原记载：GUTS 脚本存在负数参数指令（`DES_-66_0` 型） | 见 `docs/findings/script-dsl-semantics.md` 假设节 | 待定 | 未定 | 保留原行为 | 待定 | lead |
| BUG-004 | 【2026-10-16 重审·③原版确认（shim/投影排除）】battleEnemyHp 两处 putfield 常量池号不同（interactWithCell@208-223 用 enemyBaseHp #1152、m_106@281-290 用 enemyHpScaled #1185）——投影不可能把单字段读成两个常量号。原记载：战斗 HP 初始化混用两表：玩家主动战敌 HP 用基础表 `p[]`（`a.java:3519`），而损血预测/敌人主动战用缩放表 `v[]`（`a.java:6599/7683`）——A=0 首周目不可观察，A>0（读档后）预测与实战脱节 | 读档后主动挑战任一敌，对比开战前预测损血与实际损血（见 `docs/spec/gameplay.md`） | 初始化路径分叉（3519 vs 6595/7683） | 中（预测误导玩家） | Stage A 保真保留 | 统一初始化来源 | open |
| BUG-005 | 难度倍率表 `H[1..4]={30,60,80,100}` 正常流程不可达（机制口径 2026-10-09 补正：非「A 恒 0」而是**难度读取点前置赋值**、正常流程 mult=1，见 `gameplay.md` R-battle-14 补正注） | 静态可达性：`H` 引用点分析（见 `docs/spec/constants.md`） | 待定 | 无（死内容） | 保留 | 可删或补入口 | open |

| BUG-006 | 【2026-10-16 结案·A 级实证·非 bug】走位实验实证：T1110 行尾门→gameMode3 自由玩法→T1245 走位踩点 (3,10) 当拍 currentScriptIndex 40→41+续篇「很好，你已经学会太空步了。」（script41 TAK_167_167）；**A 级设计意图直证**：SEE 页 166 原文「好了，现在请试着按方向键移动到这里。」=游戏自己指示走位。「死锁」定性撤销→**等待走位（完成态+自由玩法）**，实录 reference/oracle/scenarios/prologue-walk-storypoint.md。Rust 型83 到达触发链工作正常。原记载：【2026-10-16 重审·定性撤销】原「序章脚本死锁」主张撤销——①shim 排除（纯状态机链）②投影排除（javap 逐指令：SEE 臂 lookupswitch 2230-2238+fall-through 2292 phase6→行尾门 2386-2403≡a.java:8501-8604）③实为**交互式剧情点链**：pathArrowsVisible 全库唯一写点=SEE 成功臂（a.java:8508）⇒ findPath 成功（"寻路失败"确死）；sprite51 九剧情点 param=40-48 链+SEE 目标 (3,10) 站 param=41 剧情点 ⇒ 玩家 step 踩点（a.java:5710）触发续篇——测试输入没走位=驱动缺口非挂死（B+，待走位实验升 A）。重定性=剧情点链未走位。原记载：序章脚本死锁：`SEE_3_10_166_166_1` 的玩家寻路失败导致剧情永久卡住。`executeScriptInstruction` SEE 分支（deobf a.java:8265-8270）仅在 `m_133(playerCell, target)` 返回 true 时置 `f_bool_27=true` 启动行走；实测序章地图（currentFloor=51）玩家 (1,11) → 目标 (3,10) 路径被阻，m_133 返回 false 但步数队列 `f_int_148=3` 已被写入 ⇒ 队列永无人消费、`f_byte_16=4` 永不结束 | oracle 深场景实测（2026-10-10）：狂按/每 100 tick 节奏/纯静默+探测三种输入脚本 21k ticks 全部收敛同一冻结态（scriptCursor=43、对话停在"菩提老祖: 好了"）；scenarios/prologue-{dialog,patient}.txt + fields.py 时间线 | 待定（B+ 级：行为确定性复现，但"真机是否同样卡住/是否有解锁输入序列"未证实——不排除需要特定按键序列穿过 mode11 子状态 2 选项支） | 剧情无法推进到自由玩法（第 1 层不可达，影响深场景覆盖路线） | 保留 | 换路线：菜单扫荡/道具栏/战斗入口等不经序章的覆盖（已落地 menu-sweep 场景）；深玩法覆盖可研究存档预置（RMS 预灌）绕过序章 | open |

| BUG-007 | 【2026-10-16 重审·③原版确认（shim/投影排除）】映射链四环独立字节码事实（labels 下标 astore/kind=iload_1/iconst 注册/tableswitch 臂）+ 对照组：游戏内 activateMenuItem 同 kind3→mode7 语义正确=原版内双分派表互不一致。原记载：主菜单第 3/4/5 项文字-动作错位：「保存游戏」显示帮助页（"游戏描述：…"）、「设置」显示关于页（"版权所有：…"）、「帮助」70 帧粒子淡出后直接退出应用 | oracle menu-sweep 场景（scenarios/menu-sweep.txt）三方 PASS trace：FLD062 tick911/tick1361 文本实证 + END=all-threads-exited；分派表 run a.java:3374-3402（case3→15/case4→17/case5→22）对照注册表 m_010(0..5)（a.java:4282-4287） | 确定性错位（A 级行为；"原意 vs bug"不可考，C 级） | 主菜单无可用保存/设置入口（保存/设置仅游戏内可达）；帮助从主菜单不可达 | 保留 | 待定（若 Stage B 修菜单需先考证真机版菜单行为） | open |

| BUG-008 | 【2026-10-16 重审·①主嫌疑排除③维持】ui.png alpha 直方图二值 {0:86,255:938} ⇒ 半透明合成分歧机制不存在；灰缝=资产内不透明像素每 16px 值阶跃（任何渲染器同显）；右缘双柱=drawEdgePatch case1 水平翻转 f_int_arr_01[1]=8192（shim 显式处理）；变换表 clinit {0,8192,16384,24576,8462,270,90,8282}≡a.java:44 逐值。残留（中低）：变换码 8462/270/90/8282 落 shim default 兜底，真机语义待外证（mode11 不走该路径）。锚点 JAR 真实行为维持，真机记忆差异=渠道版/记忆误差（C 级）。原记载：mode11 对话框框体观感与真机记忆冲突：框体为石纹顶边（自带灰色接缝线）+ 右缘双柱装饰 + 贴屏幕底无下边框收口（框高 40px = 顶边16+填充24，无底边绘制代码）。玩家视角疑似"边框不规则/缺底边"缺陷；用户证词（真机游玩记忆）：真机对话框不是这样，判定为 bug | 任意 mode11 NPC 对话帧：scenario:preset-floor51.txt T245-372（帧 frame-0245/0372）+ trace:A-preset-floor51；序章场景同样复现（prologue 系） | 四层排除后定性为锚点 JAR 真实行为（非我方链路引入）：① 字节码 javap 验证切片常量（paintHudFrame 26/42/53/11、paint case11 段 214/176/229/149/129/85）与 deobf 投影一致——反编译未引入错误；② 资产 ui[8][0]（64×16 框铬条，段布局 角0-26/顶26-42/左42-53/右53-64）为原始 jar 字节（packed 格式 A 级可复现）；③ Rust 与 Java 渲染逐像素一致（仅文本换行 44px 另计）；④ 独立第三方 MIDP 实现（microemulator 2.0.4）载入原始 jar 复现同构 floor51 渲染（瓦片/色板/HUD/引子匾额一致）。若真机确有差异，唯一解释为真机运行的非本仓构建（渠道版本差异），本仓无法裁定 | 视觉观感（用户判定为缺陷） | Stage A 保真保留锚点行为（D3/D4） | Stage B 美化对话框框体（统一边缘+补底边收口），需先取得真机截图或他版本 jar 作对照 | open |Stage B 美化对话框框体（统一边缘+补底边收口），需先取得真机截图或他版本 jar 作对照 | open |

| BUG-009 | 【2026-10-16 重审·③原版坐实 A 级】型60 四邻块四方向 findBlockingEntity **先调后判**（javap 偏移 394/422/457/498 调用 vs 401/427/464/503 后置边界判）vs 型61/62 块先判后调短路守卫 ⇒ 原字节码复制粘贴不对称（投影忠实）；②①逐指令排除。【更正】此前补强注"型62 块 x-1 同病"**不成立**（62 块有守卫，逐指令核过）——该主张系 B 级误并。B 级观察：型60 块 x+1 边界判用 Z（high）非 Y（wide），非方图潜伏误判。原记载：（线索）floor4 出生于 y=0 行时原版游戏首拍即崩：`ArrayIndexOutOfBoundsException: Index -1 out of bounds for length 13`（a.b ← a.k ← run，T236 载入完成首 tick）——某处 y-1 邻居访问未钳制 | RMS 预置 current_floor=4 + cell(5,0)（scenario:`preset-alchemy.preset` 初版）：trace END=all-threads-exited ticks=236，stderr 堆栈实证；y≥1 出生正常 | 待定（原版对非法出生点的脆弱性；正常流程不会把玩家放在 y=0，疑似不可达状态） | 无正常流程影响（测试装置触发） | 保留原行为（崩溃即原版语义） | 测试装置规避：出生 y≥1 | lead |

| BUG-013 | **移植缺口：preset 路径 rng 种子未落实**（DC-11 三裁一外发现）：Java 构造期 setSeed(VTime)（a.java:2325，preset 实测 VTime=11813）为活语义；Rust BootMachine 构造固定 new_seeded(0) 且 set_rng_seed 零调用 ⇒ preset 装载的 11813 种子未接线。**不可观测**：rng 唯一 消费者=mode14 粒子雨（paint.rs:528-532），preset 全槽载入直入 mode3 不触 mode14，44 场景对拍 全绿不矛盾。 | dead-code-rulings.md DC-11 + VTime=11813 preset 实测 + paint.rs:528-532 消费面 | **移植缺口（非原版 bug，D4 不适用）**——原版有此行为 Rust 未接线 | 正常游玩不可达（低危） | Stage B 修复面清单第 5 项：考证 VTime 来源→set_rng_seed 接线→单测钉 | artifact-auditor DC-11 裁定批（2026-10-16 lead 裁决 (b) 登记排期） |
| BUG-012 | 【2026-10-16 机制 A 级实证·两处更正】全量转储钉死：满行=cellSlotCount[36]（格(6,11)），T302→T347 每 3 拍一只新嫦娥叠同格（槽值 36..51 十六个不同实体），第 17 只注册 [36][16] 越界即崩（实录 bug-012-fulldump-accumulation.md）。【更正①】崩溃指令非 CES_70（从未执行）=第 17 圈 **CES_84_6_11**；【更正②】机制非 m_108 槽迁移（槽值全不同排除同值重复）=**script0 重入循环×CES_84 逐圈注册**：撞 (2,11) 剧情点→case83 bump（a.java:5466）→tryRunScene **case0 门不看 from_walk**（8076-8080）→loadLevelScript(0) 重入且返回 false（8072-8075）→step 被挡玩家钉 (2,10) 脚本从头重跑=死循环逐圈注册。Stage B 修复面追加：case0 门缺 from_walk/返回值语义重入环。原记载：【2026-10-16 重审·未解反常撤销+③坐实 A 级】"槽计数全行≤1 与满 16 矛盾"系 **StateDump 截断头误读**（MAX_ARRAY_ELEMS=32、min(len,16) ⇒ row 16..127 从未打印）——②方向最重要证据失效；三疑点逐指令排除（byte[128][16] 分配✓/下标同 var6✓/bastore 截断语义✓）；A/B/C 三变体同栈坐实。机制 B 级候选=m_108 槽迁移重复追加无 count<16 钳制（累积行未全钉，需全量转储实证=遗留）。原记载：**原版 floor3 剧情链 CES_70 槽越界崩溃**：scene0 链对话推进至 `CES_70_5_8` 执行时 `ArrayIndexOutOfBoundsException: Index 16 of length 16`（spawnEntity a.java:6418 槽登记 `cellSlotTable[var6][16]`，byte[128][16] 表）——A/B/C 三变体同 tick 同点同因 | floor3 (2,11) 触发 scene0 链+对话推进（fx-scriptwalk 不推进对话不崩=此前未撞原因）；T350 崩溃锚点，C 栈实录 reference/oracle/scenarios/bug-draft-script0-ces70-slot.md（607f321） | 原版 bug（D4 保真）：正常游玩可达（B 级，floor3 普通格 bump/step 均触发⇒疑真机同崩）；**根因未全钉**：仪器化转储显示崩溃前槽计数全行 ≤1 与"满 16"矛盾，如实留档不下结论 | floor3 剧情链在 CES_70 后断（LAY_2/RES_0 语料点 Java 不可达） | Stage A：钉崩溃前缀夹具（FX-BUG-CES70-SLOT，同 BUG-009 体例）+ LAY/RES 单测为唯一验证面（同 MVS 先例）；Stage B 修根因 | items-evidence task-10 移交稿（2026-10-16 lead 收编） |
| BUG-011 | **deobf 投影型 67 拼装臂 byte[]→Object[] 强转**：C 变体（deobf 重编译）画型 67 即崩 `ClassCastException: [B cannot be cast to [Ljava.lang.Object;` at paintEntityLayer(a.java:6592)——deobf 把片件表 `f_byte_arr2_00`（byte[][]）行读成 Object[] 再取列，byte[] 行运行期不可强转；**原字节码取值语义必然不同（反编译类型投影失真）** | `python3 tools/frames_fixture.py gen crates/game-oracle/tests/fixtures/fx-boss67a.txt`（floor15 型67 在屏即触发 paint）：C trace T236 尾 thread-exit；A/B 同场景逐字节一致；C 崩溃栈 analysis/frames-fixture/runs/C-fx-boss67a/stderr.txt | **非原版 bug（投影层缺陷）**：字节码是真相、a.java 是投影（AGENTS.md §1）；影响=T-2 移植取证源被污染（adbdee0 按污染文本实现，但 Rust 渲染与 A 帧 560/560 一致=像素级等价成立）+ 型67 场景三方对拍降级 A==B | 无正常游玩影响（渲染正确） | 修复=按 javap 字节码更正 a.java:6588-6601 类型投影（语义等价修正）+ C 变体重建回归三方 | items-evidence task-10 移交稿（2026-10-16 lead 收编） | **已修复（2026-10-16）**：按 javap 字节码语义更正并以"补丁即数据"正规化登记 `data/patches/vf-projection/VF-ACC-PAINT67.json`（原投影文本留档于补丁 old 字段）+ run.py 重生成 javac 零错重建 C 变体 + fx-boss67a 三方回归 **A==B==C**（trace sha256=f9e024c48237…，560 帧）；【2026-10-16 更正】原"型67 场景三方对拍降级 A==B"已恢复三方 |
| BUG-010 | `logostart.mid`/`logoquit.mid` 两条 BGM 是**死资源**：轨道表 `f_String_arr_15`（a.java:2239）下标 0/1 无任何播放入口——全部 6 处 `m_139` 调用实参只有 2（menu）/3（game）；mode 0 启动 logo 与 mode 22 退出动画反而各调 `closeAudioPlayer`（a.java:3376/4487） | 字节码：`javap -c -p analysis/rename-pipeline/renamed/a.class` 中 m_139 前驱指令全为 `iconst_2`/`iconst_3`（run@6271/6281、m_000@410/560、keyPressed@41/50）；运行时：EV-toggle-exit/EV-ending-title/A-menu-sweep trace 的 `f_byte_28` 只取 -1/2/3（`python3 tools/audio/evidence.py`，见 `docs/spec/audio.md`） | 确定性不可达（A 级）；原意推测（C 级）：logostart 配 mode 0 logo、logoquit 配 mode 22「退出 logo」动画，收尾时改为静音 | 无（玩家听不到这两首；JAR 多带 17KB） | 保真保留：game-core 不播放，`audio::TRACKS` 保留表项；转换工具照常产出 track0/1（备 Stage B） | 待定（若补播需先考证真机行为） | open |

> 【2026-10-16 更正/补强·BUG-009】根因已定位（A 级）：**applyStepCellEffects 型60 四邻扫描
> 「先调 findBlockingEntity 后判边界」**（a.java:5596-5616；型62 块 x-1 同病 a.java:5575-5590）——
> 双子例均可达：y=0/x=0 → `cellTypeGrid[y][-1]` ⇒ Index **-1**（原现象，preset-alchemy floor4 (5,0)）；
> y=12/x=12 → `cellTypeGrid[13][…]` ⇒ Index **13**（新复现，floor5 (9,12) 底边界格）。崩溃点逐字：
> `findBlockingEntity`（a.java:8616 `cellTypeGrid[var2][var1]` 越界读）← `applyStepCellEffects`
> （a.java:5617）← `dispatchModeEntry`（a.java:4320 case 3）——mode3 进场即执行 ⇒ T236 载入完成
> 首 tick 崩、主线程退出。对照：型61/62 块**先判后调**，同方法三种写法并存，型60 块唯一全四向
> 先调后判。Rust 侧差异：`entity::cell_entity` 越界返 -1 **不崩**（T1-236 可观测前缀对拍一致，
> T237 起 Java 已死无参照）。Stage A **保留原行为**（D4）；Stage B 修复 = 四向调用挪到边界判定后
> （对齐型61/62 写法）或 cellTypeGrid 访问钳制。一键复现：fx-bug-aioobe-border 夹具
> （crates/game-oracle/tests/fixtures/，`python3 tools/frames_fixture.py gen …` 重生成，A/B/C
> 三方同点崩溃含崩溃尾逐字节一致 = 忠实性证据）；"疑似不可达"论断升级：全 55 层边界格
> tile_walkability 复算为空 ⇒ **正常游玩不可达**（A 级），测试装置触发成立。
> 【2026-10-09 更正·非反编译引入】应质疑复查，四级证据证明该错位是**原版 JAR 字节码的真实行为**，非反编译/投影引入：
> ① 字节码静态：`javap` 原始 `<init>` 的 menuLabels 逐项 aastore 序列（@427-481：[0]新游戏 [1]继续游戏 [2]载入进度 [3]保存游戏 [4]设置 [5]帮助 [6]关于 [7]退出）与反编译源码逐字一致；
> ② run() 内 6 个 gameMode(a:B) 写点（@19763-21295，等距 ~327B/case 臂）写入 14/8/16/15/17/22 且各带 f_byte_01=1——与源码 a.java 分派一致；
> ③ 『游戏描述』『版权所有』ldc 均在 m_000（混淆名 a()V）内——mode 15/17 进帮助/关于文本是 m_000 真实分派；
> ④ 行为级：oracle `--scenarios` 对 menu-sweep 跑 **A(原字节码)==B==C(javac 反编译源码)** 三方 trace 逐字节 PASS（run.py:588-599 场景套件契约）。
> "实际游玩不可感知"的原因分析（B/C 级）：常用保存/设置入口在**游戏内右软键菜单**（mode 4/19，独立入口），主菜单错位项极少被依赖；且错位后各键仍有响应（载入进度→设置、保存游戏→帮助页、设置→关于页、帮助→退出），玩家易当作文案就这样。
> 原意推测（C 级）：分派表 0..5=引子/读档/设置/帮助/关于/退出 本自洽，疑为 m_010(0..5) 注册标签时**应取索引 {0,1,4,5,6,7} 而误取 0..5**——文案表与注册表脱节的典型收尾事故；方向与行为等价，不可考处保留待定。

## BUG-008 inGameMenuReturnMode 无初值 → hideNotify 菜单 -7 返回落错 mode（疑似原版 bug）

- **现象**：玩家在游戏内（mode 3）被遮挡/最小化触发 hideNotify 弹出菜单（mode 3→4）后按 -7"返回"，
  会落到 mode 0（启动 logo 支）而非游戏——条件是本次运行从未按过 -7（见根因）。A 级数据流、端到端未实跑。
- **复现**：进游戏（mode 3）→ 遮挡/最小化 → hideNotify（mode 3→4）→ 按 -7；
  EV-toggle-exit trace 中 FLD 012（inGameMenuReturnMode）从 T1 到 T440 恒为 0，佐证无初值=0。
- **根因**：`inGameMenuReturnMode`（f_byte_02）Java 无显式初值（默认 0）；唯一写点是 run case 3 的
  -7 键（a.java:3555）；hideNotify 进 mode 4 不写它（a.java:4502-4513 全体无赋值）。
- **Stage A 处理**：`保留但登记偏差`——原版行为=落 mode 0；Rust 现将该字段初值设为 3（scene.rs:826）
  等于顺手修了 bug，属移植偏差，应改回 0 对齐（3→4 臂接通时一并处理，见 master-plan §7）。

## 记录规范
- **现象**：玩家可见的行为描述，含层号/坐标/道具条件
- **复现**：确定性复现输入序列（trace id 或按键脚本）
- **根因**：参考版 Java 中的方法/行（反混淆命名后引用）
- **Stage A 处理**：`保真保留` / `保留但登记偏差`（差分白名单编号）
