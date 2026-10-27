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

| BUG-008 | 【2026-10-17 证伪更正·①shim 根因修复】此前"①主嫌疑排除③维持"判定**被用户证伪**：Java 真值核=shim 字高 10→14（3423db7 根因修复：字高 10 错值致框体观感差，字体派生量字面量派生化）+ 双拷贝单源化（5d179da paintWrappedText/layoutWrappedText Java 真值核）+ 全量帧夹具再生成（303ffb9）。教训：alpha 直方图排除了合成语义但**没查字体度量**——shim 排除需覆盖全部几何/度量参数面。原记载：【2026-10-16 重审·①主嫌疑排除③维持】ui.png alpha 直方图二值 {0:86,255:938} ⇒ 半透明合成分歧机制不存在；灰缝=资产内不透明像素每 16px 值阶跃（任何渲染器同显）；右缘双柱=drawEdgePatch case1 水平翻转 f_int_arr_01[1]=8192（shim 显式处理）；变换表 clinit {0,8192,16384,24576,8462,270,90,8282}≡a.java:44 逐值。残留（中低）：变换码 8462/270/90/8282 落 shim default 兜底，真机语义待外证（mode11 不走该路径）。锚点 JAR 真实行为维持，真机记忆差异=渠道版/记忆误差（C 级）。原记载：mode11 对话框框体观感与真机记忆冲突：框体为石纹顶边（自带灰色接缝线）+ 右缘双柱装饰 + 贴屏幕底无下边框收口（框高 40px = 顶边16+填充24，无底边绘制代码）。玩家视角疑似"边框不规则/缺底边"缺陷；用户证词（真机游玩记忆）：真机对话框不是这样，判定为 bug | 任意 mode11 NPC 对话帧：scenario:preset-floor51.txt T245-372（帧 frame-0245/0372）+ trace:A-preset-floor51；序章场景同样复现（prologue 系） | 四层排除后定性为锚点 JAR 真实行为（非我方链路引入）：① 字节码 javap 验证切片常量（paintHudFrame 26/42/53/11、paint case11 段 214/176/229/149/129/85）与 deobf 投影一致——反编译未引入错误；② 资产 ui[8][0]（64×16 框铬条，段布局 角0-26/顶26-42/左42-53/右53-64）为原始 jar 字节（packed 格式 A 级可复现）；③ Rust 与 Java 渲染逐像素一致（仅文本换行 44px 另计）；④ 独立第三方 MIDP 实现（microemulator 2.0.4）载入原始 jar 复现同构 floor51 渲染（瓦片/色板/HUD/引子匾额一致）。若真机确有差异，唯一解释为真机运行的非本仓构建（渠道版本差异），本仓无法裁定 | 视觉观感（用户判定为缺陷） | Stage A 保真保留锚点行为（D3/D4） | Stage B 美化对话框框体（统一边缘+补底边收口），需先取得真机截图或他版本 jar 作对照 | ~~open~~ 已证伪并修复（2026-10-15；几何根因 = shim 字高派生，见下方【2026-10-15 证伪更正】） |

| BUG-009 | 【2026-10-16 重审·③原版坐实 A 级】型60 四邻块四方向 findBlockingEntity **先调后判**（javap 偏移 394/422/457/498 调用 vs 401/427/464/503 后置边界判）vs 型61/62 块先判后调短路守卫 ⇒ 原字节码复制粘贴不对称（投影忠实）；②①逐指令排除。【更正】此前补强注"型62 块 x-1 同病"**不成立**（62 块有守卫，逐指令核过）——该主张系 B 级误并。B 级观察：型60 块 x+1 边界判用 Z（high）非 Y（wide），非方图潜伏误判。原记载：（线索）floor4 出生于 y=0 行时原版游戏首拍即崩：`ArrayIndexOutOfBoundsException: Index -1 out of bounds for length 13`（a.b ← a.k ← run，T236 载入完成首 tick）——某处 y-1 邻居访问未钳制 | RMS 预置 current_floor=4 + cell(5,0)（scenario:`preset-alchemy.preset` 初版）：trace END=all-threads-exited ticks=236，stderr 堆栈实证；y≥1 出生正常 | 待定（原版对非法出生点的脆弱性；正常流程不会把玩家放在 y=0，疑似不可达状态） | 无正常流程影响（测试装置触发） | 保留原行为（崩溃即原版语义） | 测试装置规避：出生 y≥1 | lead |

| 调查结案 | **快速战斗机制调查（2026-10-16）：无此功能，A 级穷尽结案**——四类战况探针（bump 有伤/迎面有伤/磨血 1893 拍/收尾窗）+ 全键位空间 16 键等价类 + 字节码 41 键读点 + f_int_151 写点全景，全部零加速（探针 /tmp/fx-b* 可复跑）；「按键跳战」用户记忆判为版本差异/记忆误差。磨血战 2-3 分钟干看=原版设计实况；Stage B 战斗加速=新增功能（M5 清单，形态后议）。原版行为零新增收案。 | /tmp/fx-b{3,5,6,7,8}* 探针系列 + bug-root-cause 系列文档 | 调查结案（非 bug） | — | Stage B 新增加速（用户产品裁定） | lead 终裁（2026-10-16，用户复核「可能是我记错了」） |
| BUG-013 | **移植缺口：preset 路径 rng 种子未落实**（DC-11 三裁一外发现）：Java 构造期 setSeed(VTime)（a.java:2325，preset 实测 VTime=11813）为活语义；Rust BootMachine 构造固定 new_seeded(0) 且 set_rng_seed 零调用 ⇒ preset 装载的 11813 种子未接线。**不可观测**：rng 唯一 消费者=mode14 粒子雨（paint.rs:528-532），preset 全槽载入直入 mode3 不触 mode14，44 场景对拍 全绿不矛盾。 | dead-code-rulings.md DC-11 + VTime=11813 preset 实测 + paint.rs:528-532 消费面 | **移植缺口（非原版 bug，D4 不适用）**——原版有此行为 Rust 未接线 | 正常游玩不可达（低危） | Stage B 修复面清单第 5 项：考证 VTime 来源→set_rng_seed 接线→单测钉 | artifact-auditor DC-11 裁定批（2026-10-16 lead 裁决 (b) 登记排期） |
| BUG-012 | 【2026-10-16 机制 A 级实证·两处更正】全量转储钉死：满行=cellSlotCount[36]（格(6,11)），T302→T347 每 3 拍一只新嫦娥叠同格（槽值 36..51 十六个不同实体），第 17 只注册 [36][16] 越界即崩（实录 bug-012-fulldump-accumulation.md）。【更正①】崩溃指令非 CES_70（从未执行）=第 17 圈 **CES_84_6_11**；【更正②】机制非 m_108 槽迁移（槽值全不同排除同值重复）=**script0 重入循环×CES_84 逐圈注册**：撞 (2,11) 剧情点→case83 bump（a.java:5466）→tryRunScene **case0 门不看 from_walk**（8076-8080）→loadLevelScript(0) 重入且返回 false（8072-8075）→step 被挡玩家钉 (2,10) 脚本从头重跑=死循环逐圈注册。Stage B 修复面追加：case0 门缺 from_walk/返回值语义重入环。原记载：【2026-10-16 重审·未解反常撤销+③坐实 A 级】"槽计数全行≤1 与满 16 矛盾"系 **StateDump 截断头误读**（MAX_ARRAY_ELEMS=32、min(len,16) ⇒ row 16..127 从未打印）——②方向最重要证据失效；三疑点逐指令排除（byte[128][16] 分配✓/下标同 var6✓/bastore 截断语义✓）；A/B/C 三变体同栈坐实。机制 B 级候选=m_108 槽迁移重复追加无 count<16 钳制（累积行未全钉，需全量转储实证=遗留）。原记载：**原版 floor3 剧情链 CES_70 槽越界崩溃**：scene0 链对话推进至 `CES_70_5_8` 执行时 `ArrayIndexOutOfBoundsException: Index 16 of length 16`（spawnEntity a.java:6418 槽登记 `cellSlotTable[var6][16]`，byte[128][16] 表）——A/B/C 三变体同 tick 同点同因 | floor3 (2,11) 触发 scene0 链+对话推进（fx-scriptwalk 不推进对话不崩=此前未撞原因）；T350 崩溃锚点，C 栈实录 reference/oracle/scenarios/bug-draft-script0-ces70-slot.md（607f321） | 原版 bug（D4 保真）：正常游玩可达（B 级，floor3 普通格 bump/step 均触发⇒疑真机同崩）；**根因未全钉**：仪器化转储显示崩溃前槽计数全行 ≤1 与"满 16"矛盾，如实留档不下结论 | floor3 剧情链在 CES_70 后断（LAY_2/RES_0 语料点 Java 不可达） | Stage A：钉崩溃前缀夹具（FX-BUG-CES70-SLOT，同 BUG-009 体例）+ LAY/RES 单测为唯一验证面（同 MVS 先例）；Stage B 修根因。【2026-10-17 补证/更正】Rust release **T350 slot_push [36][16] panic = 与 Java AIOOBE 同拍同点同因**（崩溃语义保真达成，docs/spec/bug-012-softlock-report.md）；旧注"Rust 存活续跑=Java 独有崩溃状态"系 T349 采样假象（夹具止于崩前一拍）已证伪。用户实机"卡住脚本没执行"归因 =（a）三拍重入死区观感（原版同款，D4 保留）+（b）崩溃宿主呈现差异（桌面闪退/wasm 假死）——(b) 已按壳层统一收口（桌面 sc.tick catch_unwind→崩溃终止屏，P0-2 案 b）；TRANSFER_EDGES Dialog→Dialog 自环边补登（P0-1，debug 误杀修复） | items-evidence task-10 移交稿（2026-10-16 lead 收编） |
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

> 【2026-10-15 证伪更正·BUG-008 几何根因是我方 shim 字高，非原版行为】用户质疑"原版不应该有这种 bug，这不是原版的真值"后复查，**部分**推翻上表"锚点 JAR 真实行为"定性——上表 2026-10-16 重审的**资产级**结论仍成立（框体美术/灰缝/翻转变换是 JAR 真实）；被推翻的是"40px 框高属原版几何"：
> ① 字节码逐指令复核（javap renamed a.class，A 级）：框高**无字面常量**，全链为
> `Font.getFont(0,0,8).getHeight()（@29-37 putfield fontHeight）→ textLineHeight=fh+4（@851-858）→ 框高=(tlh<<1)+12（@949-958）→ 框顶=320-框高（@959-964）`——框高是**设备字体度量的运行时派生量**；deobf 投影与字节码逐指令一致。
> ② shim `Font.java` 自述"度量为 shim 自定义常量（非设备保真）"：SIZE_SMALL 高 10 是**模拟器参数**，不是真机值。
> ③ 设计整除性反证（B 级）：框饰条 16px 平铺、填充=框高-16=2h+4 须为 16 整倍 ⇒ h=14（框 48=16×3 三片全收）；h=10 时第三片饰条恰在 y=320 裁半——"缺底边"假象的直接来源。
> ④ 旧"四层证据"共享同一 shim 度量假设（A==B==C 三方同 shim 自洽≠真机、microemulator 自带字体亦 ~10px 高、Rust==Java 逐像素只证明两端同 shim 一致）。
> **修复（2026-10-15）**：shim `Font.java` 与 game-core `SoftFont` 双拷贝同步更正 SMALL 6/10/8→6/14/11；对话框随之变为 48px（272-320）、饰条三片全收、底边完整。字宽 6/宽字符 12 暂不动（换行形态成形于此，无 A 级真机证据前不猜）；`setFont(size=)` op 口径由字高反推改为携带 size 常量。设备级视觉保真仍属 L4 门禁（P4），真机截图对照后可再校。

> 【2026-10-17 补记·BUG-012 第二触发面确认（用户实机）】"天牢→2 层→应与牛魔王对话却卡住"
> 与已修的"见嫦娥切层卡死"**同机制两面**：paint 分派模式臂 return 跳过 paint_wipe_tail
> ⇒ transitioning 永真 ⇒ !transitioning 行走门永久关 ⇒ 自动走位不启动 ⇒ 自走触发型
> 剧情对话（嫦娥面 T328 / 牛魔王面 floor2-stair T328 TAK_8_9）永不触发。
> 时间线实证：用户卡死会话 = 10-17 13:16（save/SKY_WAR.bin 时间戳），跑在修复
> 521aada（10-17 17:47）**之前**的构建。覆盖证明：回归 bug012b_floor2_stair_
> transition_repro（fixture_frames.rs）即"切层→自走（T314 wsc=2 弹出、T318/322/326
> 走位 (3,11)→(4,11)→(5,11)）→ T328 剧情对话开启"全链 Java 锚逐拍断言，现绿。
> 结论：重建/拉取 521aada 后重玩即通；若新构建下仍卡且触发形态 = **踩入 (2,10)**，
> 则属上表"BUG-012 重入雷区"（D4 保留的原版死循环，另一回事），按 Stage B 引导避雷。

## BUG-014 inGameMenuReturnMode 无初值 → hideNotify 菜单 -7 返回落错 mode（疑似原版 bug）

> 【2026-10-15 更正·编号】本条原题 BUG-008，与上表对话框条目（BUG-008）跨会话并行登记撞号；
> 本条改号 **BUG-014**，历史 commit（47169cc/c98f729 等）中的 "BUG-008" 均指本条。

- **现象**：玩家在游戏内（mode 3）被遮挡/最小化触发 hideNotify 弹出菜单（mode 3→4）后按 -7"返回"，
  会落到 mode 0（启动 logo 支）而非游戏——条件是本次运行从未按过 -7（见根因）。A 级数据流、端到端未实跑。
- **复现**：进游戏（mode 3）→ 遮挡/最小化 → hideNotify（mode 3→4）→ 按 -7；
  EV-toggle-exit trace 中 FLD 012（inGameMenuReturnMode）从 T1 到 T440 恒为 0，佐证无初值=0。
- **根因**：`inGameMenuReturnMode`（f_byte_02）Java 无显式初值（默认 0）；唯一写点是 run case 3 的
  -7 键（a.java:3555）；hideNotify 进 mode 4 不写它（a.java:4502-4513 全体无赋值）。
- **Stage A 处理**：`保留但登记偏差`——原版行为=落 mode 0；Rust 现将该字段初值设为 3（scene.rs:826）
  等于顺手修了 bug，属移植偏差，应改回 0 对齐（3→4 臂接通时一并处理，见 master-plan §7）。

## BUG-015 replay_opt 驱动管线一拍偏移 → 剧情自走链 T309 起步分歧（已收敛，见更正）

> 【2026-10-17 更正·本条被 BUG-016 根因吸收】续查证伪"replay_opt 驱动管线一拍偏移"
> 定性：T1429 帧分歧真根因 = tick_dialog **相位 5 空臂桩**（LAY 续跑缺口，见下方
> BUG-016）——op_lay 停表于 phase 5，Java a.java:4123-4127 在 floorTransitioning ∧
> !wipeClosing（遮幅开启相）归 0 续跑脚本，Rust 空臂永停。补 Java 真值后
> story_fullrun 夹具 4000/4000 逐 tick 同构全绿（含 T309 段）——曾记的"T309 状态
> 偏移"系状态对照脚本的采样口径假象（Java FLD 每 8 拍采样 vs Rust 逐拍行对齐错位）。
> 原记载保留：录跑对拍 T1-1428 同构、T1429 分歧（op 流逐指令对拍坐标偏 (96,160)）、
> 状态回溯见 T309 差、bug012_replay 同参驱动吻合、预置逐字节排除（CRLF/LF 3B）。

## BUG-016 tick_dialog 相位 5 空臂桩 → LAY 剧情切层后脚本永停（用户实机 bug 真身，已修复）

> 【2026-10-17 用户实机报告 + 录跑定位】"嫦娥剧情过完、被玉帝打下天牢、3 层自动
> 切 2 层后应触发牛魔王对话脚本，但没有，卡死、输入无反应"。

- **机制链（A 级）**：script0 尾 `… TAK_20_21 → LAY_2 → ROS_1_4_7 → RES_0 → GUT_1`
  （level_scripts.txt:1）——`LAY_2` = 剧情切层指令（ScriptOp::Lay{floor} → op_lay），
  op_lay 置 `transitioning=true + dialog_phase=5` 并**停表**（无尾递归）；
  tick_dialog 相位机 **case 5 = 空臂桩**（scene.rs 原 `5 => {} // f_bool_16 组合
  （LAY 批）`）⇒ 遮幅走完 swap_floor 后脚本永不续跑 ⇒ ROS_1_4_7 落位 (4,7)、
  GUT_1 → script1（TAK_22"打下天牢" + 72 号实体=牛魔王走位）永不起 ⇒
  mode 停留 Dialog(11) 输入被吞 = "卡死、输入无反应"。
- **Java 真值（A 级，a.java:4123-4127）**：`case 5: if (floorTransitioning &&
  !wipeClosing) dialogPhase = 0;` ——遮幅开启相（换层已过）归 0 = 相位 0 臂
  `executeScriptInstruction` 续跑。
- **修复**：空臂补 Java 原语义（scene.rs tick_dialog case 5）；回归 =
  story_fullrun 夹具（录跑全链 4000 tick 逐帧同构，修复前 T1429 起分歧）。
- **触发形态表补登**：剧情切层（LAY）= 第四种到达形态，与"切层踩点/撞/踩入雷区"
  三形态并列；LAY 形态的续跑依赖本缝。
- **接线期待办（2026-10-17 同类排查发现）**：script.rs 新 ScriptOp 弥擎
  （8a455dd 拆分，金样零漂移、未接入活路径）的 `DialogPhase::LayCutscene` 同样
  需要本续跑缝——其 step() 只跑 ScriptStep、其余相位归宿主驱动；接线迁移时
  必须带上 case5 语义，否则本 bug 在新弥擎复活。

## BUG-017 TRANSFER_EDGES census 缺边 → 道具 13 使用 panic 且误标"原版等价终止"（已修复）

> 【2026-10-17 用户实机："物品栏使用火眼金睛，弹出游戏已终止——请确认真的符合原版吗？"】

- **结论先行：不是原版行为，是我们引入的**（用户预判"a.java/class 语义错位或 shim
  引入"方向正确——实为状态机重构自造断言）。Java 双跳 mode10→3→5（a.java:3939-
  3945 先置 gameMode=3、activateItem 7786-7788 内置 gameMode=5）无任何检查；
  Rust 忠实镜像双跳，但 `transition_to` 的 **TRANSFER_EDGES census assert**
  （303ca46 引入）缺 (Map→EnemyInfo) 边 ⇒ panic ⇒ catch_unwind ⇒ 终止屏。
- **连带发现**：终止屏文案"（原版崩溃语义等价终止）"为未经逐案核实的声称
  （P0-2 引入）——对 Rust 自造断言类 panic 是**错误声称**（本次误导用户）。
  已改为"（崩溃详情见控制台 GAME-CRASH 行）"；等价性须逐案以 Java 锚核实。
- **census 缺口台账**：521aada P0-1 自述"35 边漏登 ≥5"——本次为第三缺口实锤
  （一：Dialog→Dialog 自环等 5 条补于 P0-1；二：Map→RecordBrowser 由 preset_floor5
  暴露；三：Map→EnemyInfo 由本 bug 暴露）。**残余风险**：census 未系统补扫前，
  其余未登记边仍会以同形态 panic 暴露（现屏显已可自诊断）。
- **回归**：`item13_use_regression`（面板使用型13→EnemyInfo / 型14→RecordBrowser
  双测，含空日志回退分支注记）。

Evidence: ①Java a.java:3939-3945/7786-7788 双跳逐字（A 级）；②TRANSFER_EDGES 表
缺边实查 + transition_to assert 文本（scene.rs:1457-1461，A 级）；③修复前后回归
红绿对照（A 级）；④main.rs:722 catch_unwind 仅包 sc.tick、paint 在捕获外
（终止屏=tick 路径 panic，排除绘制侧，A 级）。
Confidence: 95

## BUG-018 保存存档后加载列表不可见——壳层存档环三断点（用户实机，已修复）
> 【2026-10-18 补册】台账是 bug 唯一索引；原文在修复 commit 48520b9 正文，此处补登。
- **现象**：游戏内"保存游戏"后，mode7/8 读档列表看不到新档；返回主菜单读档屏同样不可见
  （用户实机："保存存档后，加载存档，无法看到已保存的存档"）。
- **复现**（修复前）：游戏内保存槽 N → 进读档列表 / 返回主菜单读档屏，会话内恒不可见。
  回归锁：`rms_roundtrips_all_six_slots_and_mot_if`（shell/win/src/main.rs）。
- **根因**（移植缺口，非原版 bug，D4 不适用）：game-core 零 I/O ⇒ 落盘/回读全靠壳接线，
  三断点——①Rms::load 键域只载 MOT_L0-2 ⇒ 槽 3-5 保存**永不落盘**（保存块只改写已存在
  键，新键整体丢弃）；②sc.preset_mot_if 仅开机交接赋值一次 ⇒ mode7/8"每次进入重读槽表"
  （Java loadSlotTable 每次进入重读语义，scene.rs:3484-3486）读到开机空槽表；③
  BootMachine.preset_stores 为开机快照不随保存同步 ⇒ 主菜单读档屏陈旧。
- **影响**：存档环（保存→列表→读档）断裂：槽 3-5 数据丢失 + 全槽会话内不可见。
- **Stage A 处理**：已修复（48520b9）：①键域全 6 槽；②保存块 upsert 六槽 + 落盘后回写
  sc.preset_mot_if；③Boot 每步同步 machine.preset_stores。
- **Stage B 修复方案**：—（移植缺口，Stage A 即修类）。
Evidence: 修复 commit 48520b9 正文三断点代码定位（A 级）+ rms_save_roundtrip 回归绿
（A 级）。Confidence: 93

## BUG-019 Windows 音频严重拖慢+杂音——winmm 单缓冲同步写欠载/拖拍（用户实机，已修复）
> 【2026-10-18 补册】原文在修复 commit 39ea26a 正文，此处补登。
- **现象**：Windows 实机 BGM"严重拖慢、杂音"（两症状一因）。
- **复现**（修复前）：Windows 运行带 BGM（winmm waveOut 后端，43c7a57）即现。
- **根因**（移植缺陷，非原版 bug，D4 不适用）：write = Prepare+Write **单缓冲** → 忙等
  WHDR_DONE（2ms 轮询）→ Unprepare 返回。①杂音 = 块间设备排空空窗（underrun click）
  + 每块 Prepare/Unprepare 抖动；②拖慢 = Windows 定时器粒度 ~15ms 使 2ms 轮询实际
  15ms+ > 块长（1024 帧≈23ms）⇒ 输出速率低于实时、持续欠载拖拍。
- **影响**：Windows 壳音频体验损伤（Linux ALSA 路径无此症）。
- **Stage A 处理**：已修复（39ea26a）：waveOut 多缓冲环 QUEUE_DEPTH=4（audio.rs:422，
  ≈92ms 常驻队列）——write 只等最老槽 Done、恒 3 块在途；Prepare/Unprepare 收敛槽回收；
  Drop 逐头收尾；超时判据改"整队列无槽释放"。
- **Stage B 修复方案**：—（同上）；WASAPI 升级缝留待实机反馈。
Evidence: 修复 commit 39ea26a 正文（write 单缓冲定位 A 级、欠载/定时器机理 B 级）+
cargo test -p game-desktop 26/26 绿（A 级）。Confidence: 90

## BUG-020 主菜单"退出"无反应——CMidlet.m_000 退出信号缺 shell 消费者（用户实机，已修复）
> 【2026-10-18 补册】原文在修复 commit 80efab7 正文 + docs/knowledge/architecture_shell-contract.md。
- **现象**：主菜单"退出"→ 退场动画（mode 22，70 拍）走完后无反应，进程不退。
- **复现**（修复前）：主菜单选"退出"确认（title.rs:181 menu_kinds kind 5→LogoAnim，
  title.rs:235）→ 等 70 拍退场。
- **根因**（移植缺口，非原版 bug，D4 不适用）：game-core 退出链正确
  （BootMachine::finished 置位 paint.rs:585 = CMidlet.m_000 语义），shell **无任何消费者**
  （只处理窗口 X CloseRequested）⇒ 信号无人执行——平台职责缺口。
- **影响**：退出功能完全不可用（用户实机）。
- **Stage A 处理**：已修复（80efab7）：Shell::finished() 取信号 + about_to_wait 每拍
  消费 → el.exit()（shell/win/src/main.rs:1071-1074）；契约定性"游戏层只发信号、退出执行
  归 shell"入 docs/knowledge/architecture_shell-contract.md。
- **Stage B 修复方案**：—（同上）。
Evidence: 修复 commit 80efab7 正文（退出链逐环 A 级）+ oracle boot_replay"进程退出"
断言证 game-core 侧正确（A 级）。Confidence: 94

## BUG-021 MOT_IF 截断读失败：Java 半解析保留 vs Rust 冷态归零（+ Rust 守卫差 1 panic 窗）（已修复，2026-10-18）
> 【2026-10-17 adversary 考证·spec §9 假设 #4 升 A】platform-contract 移交疑点，逐行锚证实。
- **现象**：MOT_IF 截断/损坏读失败时两侧行为分叉——Java 保留**半解析槽表**并落盘（损坏字节
  派生的槽值出现在读档列表且可载入），Rust 归零冷态（读档列表全空）；另 Rust 在特定截断
  长度可**直接 panic**（Java 优雅降级从不崩）。
- **复现**：构造截断 MOT_IF（preset `MOT_IF@1=<hex>` 截尾，或 save/MOT_IF.bin 截断）→ 进读档
  UI。Java 链：loadSlotTable 渐进写数组（a.java:8983-8997）→ catch → markSlotEntry(-1)
  （9000-9001）不清数组（9007-9024）→ delete+addRecord 半解析落盘（9027-9052）。Rust：
  menu_family.rs:739-741 归零。panic 窗：valid 槽 31 字节尾恰剩 30 字节——守卫
  `pos+2+28`（menu_family.rs:723）差 1 vs 实读 31（727-736）⇒ 末 read_i32 切片越界。
  trace 指针：待补（修复任务钉截断夹具时回填；现复现不依赖 oracle，单测级）。
- **根因**：原版 quirk（半解析保留+持久化，D4 保真对象）+ Rust 两处移植偏差（冷态归零
  ≠半解析；守卫算术差 1）。连带 menu_family.rs:693 注释"markSlotEntry(-1)（全无效冷态）"
  仅首字节失败成立，需原地更正。
- **影响**：损坏存档后游戏态分叉（槽表 UI/可载入槽/后续落盘内容）；Rust 特定截断可硬崩
  （preset_mot_if 灌入在 shell catch_unwind 包裹外，main.rs:722 只包 sc.tick）。可达面=
  用户实机损坏存档/外部改写/Android 应用数据部分清理（B 级推断，无真实样本）。
- **Stage A 处理**：保真对齐——Rust 复刻半解析语义（含落盘）**按字段级粒度**（RHS 先求值、
  在 Java 抛出的同一字节停；守卫 ≥31 仅作安全不变量，不得当整槽解析预检——口径见
  考证记录 §7 放大件 1）+ 四复位面全集（scene.rs:3483 / paint.rs:640 含 BootMachine 侧 /
  menu_family.rs:698 / :740）+ 截断回归夹具逐长度全网格（A 级升格）。
  修复任务 = task-7（platform-contract）。
- **Stage B 修复方案**：损坏读失败规范化为"合法冷态"（修原版 quirk）或维持原版语义——
  届时产品裁定。
Evidence: ①a.java:8959-9005/9007-9059/4353 逐行（A 级）；②menu_family.rs:694-742 逐行
（A 级）；③守卫算术 30 vs 31（B 级静态字节计数，A 升格归修复夹具）；④考证记录
docs/findings/reviews/2026-10-17-mot-if-truncation-fidelity.md。
Confidence: 90

> 【2026-10-18 状态·已修复】修复 commit **b587d11**（game-core：①字段级粒度解析——同字节停、
> readInt 读 1-3 字节抛整字段不写；②半解析保留 + 四复位面全集 + parse_dirty 触发器；
> tests/mot_if_truncation.rs 6 测 = 0..=68 全截断网格 × 零基/marker 双基态 + L=31 panic
> 窗命名回归）+ **78ac8cf**（shell：②c 回写环——take_parse_dirty → serialize_slot_if
> 整表回写 MOT_IF@1 + stores/preset_mot_if 重读源同步 + 一次性触发）。adversary 复审
> **通过**（docs/findings/reviews/2026-10-18-bug-021-fix-review.md：三硬门逐条 + RED
> 独立复现 E0308/E0599/E0600 + GREEN 双路复现 game-core 153/153 与 game-desktop 27/27）。
>
> 【2026-10-18 补注·放大件 3 登记为已知边界】跨阶段数组持久 corner²：Java 槽表数组在
> 单一 `a` 实例跨 boot→game 交接存活，Rust BootMachine::slots/GameScene::slot 交接丢弃
> ——损坏 MOT_IF + 先有旧表时，boot 读档屏（半截留旧表尾）与游戏内读档屏（尾槽冷零）
> 尾槽对照分叉（corner²：损坏存档 × 交接 × 尾槽对照）。**处置 = 方案 (b) 登记已知边界**
> **不修**；**升级触发条件**：差分测试或实机观测到上述尾槽对照分叉（即有人真看见两屏
> 不一致）⇒ 升级方案 (a) 交接携带 slot 数据（BgmState"交接时以 BootMachine 状态整体
> 覆盖"先例），届时另立任务。

## BUG-022 扩大分辨率后切屏遮幅只盖 240×320——paint_wipe_tail 网格界写死 40×30（用户实机，已修复）
> 【2026-10-20】用户报障原文："当扩大分辨率后，切屏时，切换动画是按传统分辨率绘制的，没有覆盖全屏"。
- **现象**：modern 分辨率（如 `--preset modern --grid 13,13` = 416×416）下换层（切屏），
  遮幅切换动画只作用于左上 240×320 传统分辨率区域——全黑帧右缘 x≥240 / 下缘 y≥320
  仍露场景；中段点阵（wipe=2）末格止步 (234,314)，右下角格永不落点。
- **复现**（修复前）：modern 画布 416×416 → 触发换层（changeFloor / LAY 道具）→ 遮幅
  闭合相位全黑帧（wipe=4）像素 (240,0) 以右非黑。回归锁：
  `wipe_full_black_covers_whole_canvas` + `modern_wipe_dither_spans_whole_canvas`（scene.rs）。
- **根因**（modern 适配缺口，非原版 bug，D4 不适用——原版固定 240×320，40×30 格恰满屏
  无此问题）：paint_wipe_tail 遮幅网格循环界写死 `for row in 0..40`/`for col in 0..30`
  （= 320/8 × 240/8 传统格数，照搬 deobf a.java:3217-3222 `var176<40`/`var181<30`），
  未随 Layout 视口推导（STAGE-B-002 扩屏后网格不铺满画布）。
- **影响**：modern 分辨率下换层切换动画不覆盖全屏（用户实机可见）。
- **Stage A 处理**：**已修复**（本条目所在 commit）：格数改随 `self.layout.screen_w/screen_h`
  推导（ceil(屏/WIPE_CELL_PX)）；WIPE_CELL_PX=8 收编 layout.rs 设计常量（格距=动画外观，
  不随视口缩放）。classic 240/8=30、320/8=40 代入=原值——循环序（行主序）与填格参数
  逐项恒等，oracle 帧比对 35/35 全同（release 档）零漂移。
- **Stage B 修复方案**：—（扩展缝适配缺口，Stage A 即修类）。
Evidence: A 级——deobf a.java:3217-3222 循环界 40/30 + scene.rs paint_wipe_tail 旧
5939-5942 同款硬编码 + 全黑帧像素判据红绿（RED `[modern] 遮幅全黑帧像素 240 未覆盖`
→ GREEN 双臂全过，classic 臂恒过=零漂移锚）+ `cargo test --workspace --release` 全绿
（含 game-oracle 35/35）。Confidence: 95

## BUG-023 diff-trace 门禁恒红：ItemPanel→ItemPanel 自环边 census 缺登记（debug 误杀，已修复）
> 【2026-10-20】会话状态预检 `gates/cli.py status` 发现 diff-trace FAIL（preset_itemstack_frames_match），
> 非用户报障；按门禁纪律修复根因后红→绿。同 BUG-012 Dialog→Dialog、BUG-017 Map→EnemyInfo
> census 缺边误杀第三例。
- **现象**：`cargo test -p game-oracle --test preset_cases preset_itemstack_frames_match`
  （debug 档）panic：`未登记转移边：ItemPanel -> ItemPanel`（scene.rs transition_to 的
  debug_assert）；release 档（rust-tests 门禁）不触发——debug_assert 不编译，故仅 diff-trace 面暴露。
- **复现**（修复前）：preset-itemstack 场景 T? 面板确认浮层（kind3）按 -7"回面板" →
  scene.rs:4445 `overlay_kind == 3` -7 臂在 **ItemPanel 态**调 `transition_to(ItemPanel)`。
  回归锁：`itempanel_self_edge_regression::confirm_overlay_cancel_returns_to_panel`
  + `::itempanel_self_edge_registered`（scene.rs）。
- **根因**（Rust 自造 census 断言缺登记，非原版 bug，D4 不适用）：Java 原版该臂 =
  `gameMode = 10` **自赋值**（deobf a.java:3297 `var15.gameMode = 10;`——gameMode 本就 10，
  no-op）；transition_to 收口后 census debug_assert 要求边在册，TRANSFER_EDGES 缺
  ItemPanel→ItemPanel 自环边 ⇒ 合法自赋值处误杀。
- **影响**：L1-L3 差分门禁（debug 档）恒红；无游戏态影响（release 行为与 Java 同）。
- **Stage A 处理**：**已修复**（本条目所在 commit）：TRANSFER_EDGES 补登自环边
  （29e）+ state-machine-edges.md 补行 42（代码表/文档表 42=42 机械对照成立）。
- **Stage B 修复方案**：—。
Evidence: A 级——deobf a.java:3297 自赋值原文 + panic backtrace（scene.rs:4445 →
transition_to:1482）+ 红绿复现（RED 2 测同 panic → GREEN 2 测过 + oracle
preset_itemstack_frames_match 红→绿）。Confidence: 93

## BUG-024 remove_des 状态臂 cat0/4 过写（移植偏差，V2-P2 表驱动化时收回，已修复）
> 【2026-10-23 知会登记】v2-consumer-swap（task-7 V2-P2 消费替换）发现并修正；oracle 流
> 未覆盖（CES 型 36/40 于 53/55 层不在对拍场景），以单测钉住。
- **现象**：Rust remove_des 状态臂带 default 臂，对 render_category 0/4 也覆写 anim_row——
  Java 原版 switch **无 default 臂**（cat0/4 走 switch 不命中=不覆写）。
- **复现**：单元测试 `task7_mark_des_removed_java_switch_no_default`（型 36/40 场景钉死）。
- **根因**：移植期 switch 翻译多带 default 臂（38 写点归一时代的过写残留），非原版行为。
- **影响**：CES 型 36/40（53/55 层）的 mark_des_removed 外观过写——oracle 场景未覆盖此型此层，
  金样无从发现；行为差在特定型的动画行。
- **Stage A 处理**：**已修复**（随 V2-P2 表驱动化按 transition_overrides 真值收回，commit 2d27c47）；
  oracle 覆盖缺口如实登记（不补场景——对拍面扩充归 verify-batch 另议）。
- **Stage B 修复方案**：—。
Evidence: A 级——Java switch 无 default 臂（a.java remove_des 源）+单测钉死+表侧真值锚（220 逐值锚）。Confidence: 90
## 记录规范
- **现象**：玩家可见的行为描述，含层号/坐标/道具条件
- **复现**：确定性复现输入序列（trace id 或按键脚本）
- **根因**：参考版 Java 中的方法/行（反混淆命名后引用）
- **Stage A 处理**：`保真保留` / `保留但登记偏差`（差分白名单编号）

## BUG-025 剧情行走中连点"确定"⇒ 嫦娥走位戏错乱（壳层投键打断寻路误清脚本行走路径，用户实机，已修复）

> 【2026-10-26 用户实机报告】"跳过教程后，嫦娥脚本行为异常"——"不是 soft lock，没有锁死，脚本执行了，
> 但行为异常"。实机截图：一层嫦娥停在左走廊半路、玩家挡在其下，剧情草草收尾。
- **现象**：一层 script19（嫦娥走位戏 `CES_84_7_7 MOV_84_7_7_6_8 … MOV_84_1_1_10_1 MOV_0_6_1 DES_84_10_1 …`）
  进行中，嫦娥/玩家脚本行走期间点"确定"（触屏面板、键盘同口）⇒ 行走被截断停在半路；其后
  `MOV_84_<格>…` 按格找不到嫦娥而跳过、`DES_84_10_1` 落空 ⇒ 嫦娥滞留、剧情走样。与逻辑分辨率无关
  （classic 240×320 与竖屏 12×16 同表现）。
- **复现**：`cargo test -p game-android --release --test script_walk_taps`（修复前两测红："嫦娥走到 (6,8)"）；
  Java 锚夹具 `crates/game-oracle/tests/fixtures/fx-chang-e-floor1-taps.*`（A==B==C，2400 帧，T320 起每 16 拍 -5）
  按壳层投键口径重放（`fx_chang_e_floor1_taps_shell_key_path_frames_match`）修复前 TICK 387 帧不符。
- **根因**：Stage B 触屏寻路（b2dc630 / f4ce171）的 `GameScene::interrupt_route` 文档承诺"仅对 WalkTo/探步
  寻路段生效、经典脚本行走 key 链一行不动"，实现却**无条件**清 walkPathBuffer/walkStepCount；壳层每次投键先调它
  （android `replay_scene_keys`、win `route_key`、触屏 Dispatcher）⇒ 截断脚本 MOV 的实体路径（dp2）与玩家脚本
  行走（dp3）。第二入口：触屏点格寻路走进剧情点时寻路段不收束，剧情中 `press_key` 见段开放仍会打断。
  Java 锚：`keyPressed` 只写 keyValue/keyHeldCode（a.java:4663-4676），行走期按键不动路径。
- **影响**：所有含脚本行走的剧情在"边走边点"时走样（一层嫦娥戏最显眼）；对拍夹具直投 `press_key`、绕过壳层
  投键路径 ⇒ 门禁未覆盖。
- **Stage A 处理**：**已修复**——`interrupt_route` 仅在寻路段开放或挂起（战斗/拾取待续走）时清；
  `load_level_script` 剧情接管先收束寻路段。core 单测 `t2_interrupt_route_without_open_route_keeps_script_walk` /
  `t2_script_takeover_closes_walkto_route` + Java 锚壳层口径对拍 + Android 运行时全链两测。
- **Stage B 修复方案**：—。
Evidence: A 级——a.java:4663-4676 keyPressed 原文 + fx-chang-e-floor1-taps Java 三方一致帧锚（壳层投键口径 2400/2400 帧一致）。Confidence: 95

## BUG-026 changeFloor 误写 gameMode=3 ⇒ 剧情脚本行走踩楼梯切层即丢弃剩余脚本（移植偏差，已修复）

- **现象**：剧情中 MOV 让玩家走上楼梯格（一层 script19 尾 `MOV_0_11_1` 走上 (11,1) 楼梯 → 2 层续
  `MOV_0_1_11`）：Rust 切层即回 mode 3、余下脚本丢弃；Java 在 mode 11 续跑（2 层继续脚本行走）。
- **复现**：`fx_chang_e_floor1_taps_frames_match`（classic 引擎口径，press_key 直投）修复前 TICK 786 帧不符
  （Rust mode=3 / Java gameMode=11，T786 Java floorTransitioning=true、T800 2 层 walkStepCount=8 scriptCursor=184）。
- **根因**：Stage A 移植把道具 21 调用方语义（`changeFloor(+1); keyHeldCode=0; gameMode=3`，a.java:7863-7866）
  误挂进 `change_floor` 本体；Java `changeFloor`（a.java:7135-7180）不写 gameMode。楼梯步效果（a.java:5655/5661）、
  49/55 换乘键（5298/5304）、跳过教程（4779 调用方自写 gameMode=3）均不经此路回 mode 3。
- **影响**：classic 路径保真缺口（原版剧情链"走楼梯换层后继续演出"被截断）；oracle 场景此前未覆盖"脚本行走踩梯"。
- **Stage A 处理**：**已修复**——`change_floor` 不写模式；道具 21 臂按 Java 补 `key_held=0` + mode 3。
  A-preset-item21 等全部对拍夹具保持全绿。
- **Stage B 修复方案**：—。
Evidence: A 级——a.java:7135-7180 changeFloor 原文无 gameMode 写；a.java:7863-7866 道具 21 调用方写 gameMode=3；
fx-chang-e-floor1-taps Java 帧锚 T786。Confidence: 95

## BUG-027 Android BGM 全程无声——AAudio 设备实现三处缺陷（性能模式错值/从不启动/负超时）（用户实机，已修复）

> 【2026-10-26】shell/android/README.md 验证矩阵早记"音频（AAudio `openStream` 报 ILLEGAL_ARGUMENT ⇒ 静音降级）"；
> 用户："继续修声音问题"。
- **现象**：Android 壳 BGM 全程无声：logcat `aaudio open ret=err AAudioStreamBuilder_openStream：AAUDIO_ERROR_ILLEGAL_ARGUMENT`
  → `SilentOut` 无声降级。
- **复现**：实机探针 `shell/android/examples/aaudio_probe.rs`（直接驱动壳的 `AAudioPcm`；adb shell 运行、不起 Activity、
  只写静音；构建/运行命令见文件头）。修复前：`probe: open=err …ILLEGAL_ARGUMENT`，logcat
  `E/AudioStreamBuilder: illegal performanceMode = 1`（EX、SH 两次打开皆然）。仅改正①后：
  `probe: seg1 HANG（6.0s 内未写完 2s 音频；cpu=5.97s）`（写环满核自旋）。
- **根因**（Android 壳实现缺陷，非原版 bug，D4 不适用；骨架期"extern 签名与常量照头文件手写、未验证"遗留）：
  ① `AAUDIO_PERFORMANCE_MODE_LOW_LATENCY` 手写成 1——NDK `aaudio/AAudio.h:325-338` 该枚举自 `NONE = 10` 起算（LOW_LATENCY = 12）
  ⇒ `AudioStreamBuilder::validate()` 落 default 臂恒拒（AOSP android-16.0.0_r1 `AudioStreamBuilder.cpp:266-282`）；
  ② 打开后从不 `requestStart`：流停在 OPEN 不消费，写入 0 帧立返；
  ③ 写超时 -1 当"无限等待"：AAudio 负/零 = 不阻塞（legacy `AudioStreamTrack.cpp:505` `blocking = timeoutNanoseconds > 0`；
  MMAP `AudioStreamInternal.cpp:840/873` 截止 = now+timeout）⇒ ②③叠加 = 写环自旋、永不出声；
  ④ `idle` 用 `requestStop` 且其后不再 start（stop 先播完残留；契约"停止并丢弃"= pause → PAUSED → flush，AAudio.h:1566/1585）。
- **影响**：Android 壳无 BGM（改正①而不改②③ = 音频线程满核自旋）。
- **Stage A 处理**：**已修复**——`audio::StreamPcm` 流状态机（首写前 start、正超时分段写满、0 帧即 Err 不自旋、
  idle = pause → 等 PAUSED → flush）× `aaudio::NdkStream`（NDK 直绑）；打开参数改 SHARED + PERFORMANCE_MODE_NONE
  （常驻流不占 EXCLUSIVE、BGM 不要低延迟、与探针同走 legacy 路径——取舍见 audio.rs `STREAM_CONFIG`）。
  宿主钉 `shell/android/tests/aaudio_stream.rs`（假流按 AAudio 状态规则建模）。
- **Stage B 修复方案**：—。
Evidence: A 级——NDK 27.1.12297006 AAudio.h 枚举原文；AOSP android-16.0.0_r1 libaaudio 源码（validate / processData /
AudioStreamTrack::write）；实机探针 RED（logcat `illegal performanceMode = 1`；HANG cpu 5.97 s / 6.0 s）。Confidence: 92

> 【2026-10-26 补·设备切换重开】同组件潜在缺陷（出声后才会暴露）：插拔耳机/连蓝牙 ⇒ AAudio 把流置为断开——
> legacy 路径设备 ID 一变即 `forceDisconnect`（AOSP `AudioStreamLegacy.cpp:283-303`），其后 write 返回
> `AAUDIO_ERROR_DISCONNECTED`（`AudioStreamTrack.cpp:500-502`）、start 返回 INVALID_STATE（`AudioStream.cpp:169`）
> ⇒ audio_loop 按契约转无声，**本次运行 BGM 永久消失**。修：`StreamPcm::with_reopen` 挂 `NdkStream::open` 工厂——
> 断开即关旧开新、余量续写，每次 write 至多重开一次（新流仍失败 = Err）。宿主钉 `aaudio_stream.rs` 3 测
> （写中途断开 / 空闲期断开 / 重开有界）。**实机未验证**：需物理插拔耳机或切蓝牙（B 级：源码路径推理）。

## BUG-028 Android 壳 BGM 命令翻译残缺——不发 Stop / 不认 serial / 音量恒 1.0 / 每拍刷日志（壳层移植缺陷，已修复）

- **现象**（BUG-027 修好出声后即暴露）：切后台、设置里关声、mode 22 退出动画后音乐**照放**；回前台按键续播、
  设置里关声再开不从头播；BGM 音量恒满幅（应为 VolumeControl 级别/100，默认 60）；logcat 每拍一行
  `bgm req=None`（75 ms/拍，日志缓冲很快被刷满轮转）。
- **复现**：`cargo test --release -p game-android --test bgm_sync`——旧翻译逐行搬入 `Runtime::sync_bgm` 时
  命令序列仅 `[Play{2,1.0}, Play{3,1.0}]`（应为 `[Play2, Stop, Play2, Stop, Play2, Play3, Stop]`，gain 0.6）。
- **根因**：native.rs 主循环手写翻译只认 `bgm_request()` 轨道下标——变 None 不发 Stop、`last_bgm` 永不复位、
  同轨 serial 变（Java 关闭重建 = 从头播放，a.java:10178-10199）不重发、音量不读 `bgm_volume()`；且只在拍后
  同步、hideNotify 后不同步。shell-common 早有完整翻译 `BgmController`（win 壳在用），Android 壳另写了一份残缺拷贝
  （双拷贝漂移，`docs/knowledge/pattern_dual-copy-algorithms-sync-fix.md` 同型）。
- **影响**：Android 壳 BGM 启停/音量语义与原版（docs/spec/audio.md §1 状态表）不符。
- **Stage A 处理**：**已修复**——`Runtime` 持 `BgmController`（每拍末 + hideNotify 后同步），快照取数
  `BgmSnapshot::of_boot/of_scene` 收进 shell-common（win/android 双壳同源）；android_main 只注入音频线程出口
  （命令即日志，无变化不打印）。宿主钉 `shell/android/tests/bgm_sync.rs`（真资产全流程：标题 → 切后台 → 按键续播 →
  设置关/开声 → 进游戏 → 游戏中切后台）。
- **Stage B 修复方案**：—。
Evidence: A 级——宿主 RED/GREEN 逐字（旧翻译 `[Play{2,1.0}, Play{3,1.0}]` vs 期望序列）+ docs/spec/audio.md §1 状态表
（Java 锚 a.java:4157-4167/4502-4513/4663-4676/10158-10215）。Confidence: 93

## BUG-029 Modern 下牛魔王（型 72）对话立绘错位到地图中左、对话框头像格空——定尺剪裁臂 y 字面量 275 漏换轨（Modern 回归，用户实机，已修复）

> 【2026-10-27】用户实机截图（Xiaomi 14 竖屏 12×16，2 层天牢"牛魔王：恩恩，现在大哥带你从密道出去……"）：
> "牛魔王的对话头像位置不对""这个问题是 regression，classic 没有这个问题"。
- **现象**：Modern 布局下牛魔王说话页，立绘画在地图左中部（逻辑 (24+dx, 275)），底部对话框左侧头像格为空；
  其余说话者（型 >0 非 72 / 玩家头像）正常；classic 240×320 正常。
- **复现**（修复前）：`cargo test --release -p game-core --lib bottom_composition_is_translate_only`（组合关系钉补
  speaker=72 后）RED：modern 底部带缺 `("setClip",24,275)/("drawImage",24,275)` 两项（classic 有）。
- **根因**（Modern 适配回归，非原版 bug，D4 不适用）：a.java:2852 `drawImageClipped(var9, 24, 275, 0, 0, 32, 45)`
  的 275 = 320 − 45（定尺 32×45 贴屏底；同段另两臂写作 `320 − h`，a.java:2841-2849/2855-2857）。076ec69
  （STAGE-B-002 返修"modern 绘制坐标系整体换轨"，2026-10-18）把对话框锚 `dialog_block_y0` 与另两臂的
  `SCREEN_H − h` 换成 `screen_h − h`，72 臂是编译期折叠后的字面量、未被识别为屏高派生 ⇒ 漏改；9a23314 只补了
  水平居中 dx。对话框随之下移到屏底、72 立绘留在 y=275 ⇒ 自 076ec69 起 Modern 回归。组合关系钉
  `bottom_composition_is_translate_only` 只覆盖 speaker 0/1，漏 72 特例臂 ⇒ 门禁未拦。
- **影响**：Modern 下全部牛魔王对话页（2 层天牢链、50 层）立绘错位。classic 不受影响：screen_h = 320 ⇒ y = 275
  与 Java 同值。
- **Stage A 处理**：**已修复**——72 臂 y 改 `screen_h − 45`（classic 回代 275，零偏离）；组合关系钉补 speaker=72。
  classic 28 个 Java 锚帧夹具全绿；覆盖探针：72 臂 y 偏 1px ⇒ `story_fullrun_fixture_frames_match` TICK 1434
  （mode 11）帧不符 ⇒ classic 夹具确实画到牛魔王立绘、且 275 与 Java 逐像素一致；Modern 帧基线 5 状态 sha 不变。
- **Stage B 修复方案**：—（扩展缝适配缺口，Stage A 即修类）。
Evidence: A 级——a.java:2838-2857 原文 + 076ec69/9a23314 diff + 组合关系钉 RED/GREEN + classic 覆盖探针（T1434 帧锚）。
Confidence: 95

## BUG-030 Modern 下击败怪物的"金币 +N"弹字偏离金币列——kind 4 弹字 x 字面量 240 未跟随贴右的金币列（Modern 回归，用户实机，已修复）

> 【2026-10-27】用户实机（Xiaomi 14 竖屏 12×16）："击败怪物后，金币+1 的提示没有对齐到金币列，偏移了"。
- **现象**：Modern 布局下击败怪物（主动战胜利 / cat8 直杀）弹出的金币 +N 数字落在顶栏钥匙格下方，偏左 `screen_w − 240`
  （12×16 = 384 宽：144 px）；classic 正常（右缘对齐金币数值）。
- **复现**（修复前）：`cargo test --release -p game-core --lib gold_popup_tracks_gold_column`——弹字首位裁剪 x − 金币数值
  右锚：classic 0、modern 12×16 −144。
- **根因**（Modern 适配缺口，非原版 bug，D4 不适用）：`PopupRing::spawn` kind 4 照抄 a.java:9706 `popupX = 240`；
  STAGE-B-002 ② 把顶栏金币列改为贴右（`status_gold_num_x = screen_w − 3`），弹字未随动。旧注把 240 判为"C 类·语义
  不明（屏右缘 vs 跟随金币位 237+3），扩视口两者分叉，不派生"——实则金币列贴右后两解同值 `screen_w`，不存在分叉。
- **影响**：Modern 全部金币弹字（scene.rs 主动战胜利 / cat8 直杀两处 spawn）。
- **Stage A 处理**：**已修复**——`Layout::gold_popup_x`（classic `GOLD_POPUP_X = 240`；modern = 金币右锚 + 3），
  `GameScene::apply_layout` 同步给 `PopupRing`。classic 28 个 Java 锚帧夹具全绿；覆盖探针：classic 锚 240→241 ⇒
  fx_boss74a / fx_boss67a / fx_boss68a / fx_chang_e_floor1_taps / preset_floor49 等 7 测帧不符（首个 TICK 302）
  ⇒ classic 夹具确实画到金币弹字、240 与 Java 逐像素一致。
- **Stage B 修复方案**：—（扩展缝适配缺口，Stage A 即修类）。
Evidence: A 级——a.java:9706-9707 原文 + layout.rs 金币列贴右公式 + RED/GREEN + classic 覆盖探针。Confidence: 95
