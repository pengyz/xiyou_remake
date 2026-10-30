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

| BUG-007 | 【2026-10-28 证伪更正·非 bug（A 级）】标题项文字**不是** `menuLabels[kind]` 文本，而是美术条 menu 容器 [1]（68×102，6 行×17px）按 `17 * menuItemKinds[cursor]` 取行（a.java:2359-2368 paint case 1 drawIconStrip(f_Image_arr2_00[10][1], …, 17*kind)；Rust title.rs:470/495 同式）。该图六行依次为「开始游戏 / 继续游戏 / 设置 / 帮助 / 关于 / 退出」（2026-10-28 解包 assets/raw/menu 目视），与 run case 1 分派 kind 0..5 → mode 14/8/16/15/17/22 逐项一致；帮助 / 设置 / 关于三页自身的标题条也取同图第 3/2/4 行（sy=51/34/68，a.java:2960/3000、paint.rs About 臂）。原结论把标题文字误当 menuLabels（那是游戏内菜单的文字标签表，kind 3 =「保存游戏」→ mode 7 本就一致），"双分派表互不一致"不成立。原记载：【2026-10-16 重审·③原版确认（shim/投影排除）】映射链四环独立字节码事实（labels 下标 astore/kind=iload_1/iconst 注册/tableswitch 臂）+ 对照组：游戏内 activateMenuItem 同 kind3→mode7 语义正确=原版内双分派表互不一致。原记载：主菜单第 3/4/5 项文字-动作错位：「保存游戏」显示帮助页（"游戏描述：…"）、「设置」显示关于页（"版权所有：…"）、「帮助」70 帧粒子淡出后直接退出应用 | oracle menu-sweep 场景（scenarios/menu-sweep.txt）三方 PASS trace：FLD062 tick911/tick1361 文本实证 + END=all-threads-exited；分派表 run a.java:3374-3402（case3→15/case4→17/case5→22）对照注册表 m_010(0..5)（a.java:4282-4287） | 【2026-10-28 更正】无错位：标签源为美术条行而非 menuLabels。原记载：确定性错位（A 级行为；"原意 vs bug"不可考，C 级） | 【2026-10-28 更正】无。原记载：主菜单无可用保存/设置入口（保存/设置仅游戏内可达）；帮助从主菜单不可达 | 保留 | — | closed（2026-10-28 证伪：非 bug） |

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

> 【2026-10-27 实机复核·GREEN】（Xiaomi 14 / Android 16，本方运行，非用户操作）探针（adb shell，只写静音）：
> `open=ok sharing=1 perf=10 rate=44100 burst=4895 buffer=9790/9790 xrun=0`；`seg1 write=Ok(()) audio=2.020s wall=1.828s
> cpu=0.01s`（阻塞写 = 节拍，不自旋；墙钟短 0.19 s = 首块填缓冲 9790 帧 ≈ 0.22 s）；`seg2-after-idle audio=1.022s
> wall=0.838s cpu=0.00s`（idle 已丢弃缓冲、重启正常）；logcat 状态链 OPEN→STARTING→STARTED→PAUSING→PAUSED→
> FLUSHING→STARTING、`MMAP not used because AAUDIO_PERFORMANCE_MODE_LOW_LATENCY not requested`（NONE 走 legacy 实证）。
> 游戏进程（11:35）：`aaudio open ret=ok sharing=1 perf=10 …`；`dumpsys audio` 本进程 `AudioPlaybackConfiguration …
> type:AAudio … state:started`（蓝牙 A2DP 设备 10046）；音频线程 CPU ≈ 1%（3 s 窗 3 tick）。Confidence: 97

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

> 【2026-10-27 实机复核·GREEN】（本方运行）标题 `bgm Play { track: 2, gain: 0.6 }`（音量 60 ⇒ 0.6）；本方注入 HOME
> （11:36:29）⇒ 同毫秒 `bgm Stop` + `lifecycle hideNotify`，`dumpsys audio` 本进程播放配置 `state:started` → `state:paused`；
> 全程无 `bgm req=None` 刷屏（日志只随命令）。Confidence: 96

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

> 【2026-10-27 用户实机验收】用户在 Xiaomi 14（含本修复的包）上试玩确认："牛魔王问题解决了"。本方验证面 = 宿主组合关系钉 +
> 竖屏整窗无头截帧（f8dc52c）；实机自动核对脚本未跑（用户已验收，按 AGENTS §5 产品体验以用户试玩反馈为准）。

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

> 【2026-10-27 实机旁证】本方验证 = 宿主单测 + 竖屏整窗无头截帧（f8dc52c，金币 +1 右缘贴金币列）。另：用户 12:51 实玩
> 被动录屏（本方未注入）中金币 4→5 时 "+1" 弹字右缘贴在金币数值列正下方——属用户操作，按 AGENTS §5 不记为本方验证成果，
> 仅作旁证。

## BUG-031 Modern 布局字面量漏换轨 5 处（BUG-029 同型，全量扫描所得，已修复）

> 【2026-10-27】用户："全量扫描整体是否还有类似的布局公式问题"。静态扫描 game-core 全部绘制调用的屏幕派生字面量
> （折叠的 `320−k` / `240−k` / 屏心 120、160 与 SCREEN_W/H 常量），逐处对照 a.java 同块兄弟臂是否已换轨。
- **现象 / 根因**（Modern 适配缺口，非原版 bug，D4 不适用；共同签名 = 同一 Java 块的兄弟臂已随 layout，漏网的是
  编译期折叠字面量或残留 SCREEN 常量）：
  ① walkPhase==2 观察态四角箭头（scene.rs paint_player，a.java:5964-5967）：`148 = 160−12`、`109 = 120−11`、
  `215 = 240−25` 留 classic 位，同块下臂早随 viewBottom ⇒ modern 左右箭头停在画面中上、右箭头在画面中间；
  ② 炼丹（mode 9，a.java:2715-2722）：属性图标 x 97 / 加值 x 142（= 高亮行 x 62 + 35 / + 80）不随居中的行框 ⇒ modern
  落在框外左侧；
  ③ 敌人图鉴（mode 5，a.java:2511/2524/2595）：翻页箭头 x 112（= 120−8）、预测伤害列 `(SCREEN_W + 面板宽 >> 1) − 75`
  ⇒ modern 箭头偏左、伤害列不贴面板右缘（12×16 偏 72 px）；
  ④ 记录浏览（mode 12，a.java:2906-2914）：页码 CENTER_X、双箭头 ARROW_PAIR_LEFT/RIGHT_X 常量（Layout 早有
  center_x / arrow_pair_*，标题页已用）⇒ modern 不随屏心；
  ⑤ 开场链帮助/关于浮层（paint.rs paint_overlay_tail）：盒框 `(SCREEN_W/H − 盒) >> 1` ⇒ 开场链画布 > 240×320
  （win 壳 modern）时偏左上（Android 竖屏开场档恒 240×320，潜伏）。
- **复现**（修复前）：`cargo test --release -p game-core --lib`——`soul_view_corner_arrows_frame_the_screen` /
  `alchemy_rows_translate_with_highlight_row` / `encyclopedia_arrows_and_damage_column_follow_panel` /
  `record_browser_indicator_and_arrows_follow_center` / `boot_help_overlay_box_centered_on_layout` 5 测 RED。
- **影响**：Modern 下上述 5 个界面元素错位（①③④⑤ 用户实机竖屏可见，⑤ 仅 win 壳 modern 可见）。
- **Stage A 处理**：**已修复**——逐处换成 classic 回代 = Java 字面量的 layout 公式（屏心 / 屏宽 / 行内相对位）。
  classic 覆盖探针：每处修复点临时偏 1 px 均有 classic Java 锚测转红（① preset_floor49 / fx_chang_e_floor1_taps×2 /
  story_fullrun，② preset_alchemy，③ preset_item13（箭头 + 伤害列），④ preset_floor5，⑤ boot_menu_frames_match_tick_by_tick）
  ⇒ 公式运行期确实回代 Java 字面量；修后 game-oracle 全绿、Modern 帧基线 5 状态 sha 不变。
- **扫描余项（未改，如实登记）**：结局卷轴（mode 20，a.java:3043/3064）图 240 宽左贴、字幕 x 15——modern 右侧露白，
  需定"居中 + 两侧补带色"设计（待用户定）；logo 飞入起点 x 15（logo_anim.rs:394，a.java:10394）语义不明（低）；
  记录浏览箭头 y 的运算符优先级移植偏差见 BUG-032（classic 同错，另立）。
- **Stage B 修复方案**：—（扩展缝适配缺口，Stage A 即修类）。
Evidence: A 级——a.java 各行原文（兄弟臂 `240−…>>1` / `320−…` 形式为屏幕派生铁证）+ 5 测 RED/GREEN + classic 覆盖探针。
Confidence: 93

## BUG-032 记录浏览（mode 12）双箭头画错位置与朝向——运算符优先级误译 + 变换码直传（classic 移植偏差，扫描所得，已修复）

> 【2026-10-27】BUG-031 全量扫描时发现（静态读码 + javap 字节码核对），新建 Java 锚夹具实证。
- **现象**（classic 与 Modern 同错）：记录浏览有 ≥2 条日志时，翻页箭头画在屏幕中部（classic y 153）而非页码两侧
  （Java y 304）；且左箭头朝向错误（垂直翻转，Java 为水平翻转）。
- **复现**：新夹具 `crates/game-oracle/tests/fixtures/fx-record-arrows.*`（preset-floor5 同场景、存档日志改 3 条——原
  preset-floor5 仅 1 条、箭头不画 ⇒ 该行从未被帧锚覆盖；Java A==B==C 1200 帧）：修复前 `fx_record_arrows_frames_match`
  "TICK 621 帧不符（mode=12）"（y），只改 y 后 "TICK 701 帧不符（mode=12）"（翻到末页、左箭头首现 ⇒ 变换码）。
- **根因**（Stage A 移植偏差，非原版 bug，D4 不适用）：
  ① a.java:2908 `var46 += var24.fontHeight + 4 - 13 >> 1`：复合赋值右侧整体 = `(fh + 4 − 13) >> 1`（javap 5242-5255：
  `iload_2 … iconst_1 ishr iadd`）；Rust 译成 `(行底 + (fh + 4 − 13)) >> 1`，把页码行底也减半；
  ② a.java:2910 `drawImageWithNokiaTransform(…, 1)` 的 1 是变换表下标（→ `NOKIA_TRANSFORM_TABLE[1] = 8192` 水平翻转，
  Java trace `drawImageT(20x13,60,304,0,8192)`）；Rust 直传 1（shim 自定义表 1 = 垂直翻转）。
- **影响**：日志 ≥2 条时的记录浏览界面（道具 14 使用后）。
- **Stage A 处理**：**已修复**——①按字节码改为 `行底 + ((fh+4−13)>>1)`；②传 `NOKIA_TRANSFORM_TABLE[1]`。
  新夹具 1200/1200 帧 = Java；core 钉 `record_browser_arrow_y_sits_beside_indicator`（classic / modern 同式）。
- **Stage B 修复方案**：—。
Evidence: A 级——javap 字节码 5242-5255 + Java A==B==C 夹具 fx-record-arrows（trace sha256 e3027e66a623…）T621/T701 帧锚。
Confidence: 97

## BUG-033 Android 主菜单"退出"不结束进程、返回手势无反应——退出信号无壳消费者 + 标题态 BACK 死键（壳层移植缺口，用户实机，已修复）

> 【2026-10-27】用户实机（Xiaomi 14 竖屏）："主菜单右滑无法退出，主菜单选择退出也无法结束进程"。
- **现象**：① 主菜单选"退出"确认 → 退场动画（mode 22）播完后停在白屏，进程不退、前台仍是游戏；② 主菜单返回手势
  （边缘滑动 = KEYCODE_BACK）无任何反应。
- **复现**（修复前）：实机现场为用户操作所留，本方只读取证——截屏：游戏区整屏白、下方虚拟摇杆面板仍在；14:05 `ps` 见
  进程 30585 仍在（已运行 70 min）、`dumpsys activity` 的 topResumedActivity 仍为 `com.jjxy.game/android.app.NativeActivity`；
  logcat 标题态返回 = `tick batch=[Press(-7), Release]`、模式不变。宿主：`tests/exit_chain.rs` 对修复前源码编译即
  E0599（`Runtime` 无 `exit_requested` / `title_back_keys`——壳侧没有退出消费面）。
- **根因**（Android 壳移植缺口，非原版 bug，D4 不适用）：
  ① game-core 退出链正确：mode 22 第 71 拍置 `BootMachine::finished`（paint.rs:709-721 = reference a.java:4249-4256
  `CMidlet.m_000()` = `destroyApp(true)` + `notifyDestroyed()`）；Android 壳**无任何消费者**——
  `process::android::AndroidProcessControl`（ANativeActivity_finish）自骨架期从未接线（shell/android/src 无引用），
  BUG-020（win 壳）同型缺口 Android 侧未补；
  ② BACK 恒吞并译为右软键 -7（input.rs:91，BUG-020 口径：退出须走游戏退出链），而标题 tick 只有 -5/53 确认、
  -4/-2/54/56 下移、-3/-1/50/52 上移三臂（title.rs:382-415），-7 落 `_ => None` ⇒ 主菜单返回 = 死键。
- **影响**：Android 壳无法从游戏内退出（只能从最近任务划掉）；选"退出"后进程与 Activity 常驻白屏。
- **Stage A 处理**：**已修复**——
  ① 主循环每拍查 `Runtime::exit_requested()`（场景外 ∧ finished；win 壳 consume_exit 同口径）⇒
  `ProcessControl::request_exit` = `ANativeActivity_finish`（只发一次）；Activity 销毁（APP_CMD_DESTROY）后主循环退出 ⇒
  `_exit(0)` 结束进程（= MIDlet notifyDestroyed；不用 libc `exit`：此刻 Java 主线程阻塞在 onDestroy、binder/AAudio 线程
  仍活，`__cxa_finalize` 静态析构与之竞态；存档为同步写穿，无待刷数据）；
  ② 标题态 BACK（DOWN、非重复）= 选中"退出"项（menuItemKinds 5）+ 确认 -5——走同一条退出链（mode 22 退场动画
  71 拍 × 75 ms ≈ 5.3 s 后退出），不另造退出路径；非标题态保持 -7 原语义（子页返回 / 游戏内菜单）。属壳层输入映射
  （同虚拟键盘：平台输入译成既有按键），游戏层零改动、无帧金样影响。
  宿主钉 `shell/android/tests/exit_chain.rs` 3 测（菜单退出 → 动画播完才请求 / 返回同链且光标落「退出」/ 读档子页不劫持）。
  > 【2026-10-27 更正】② 已被 **BUG-034** 取代：返回走 mode 22 logo 退场动画，用户实机反馈形似"进程内重置、重播片头"、
  > 动画期再返回无效、开机 logo 期返回亦无效——"不符合用户习惯"。现为根界面返回 = 立即退出（见 BUG-034）；
  > ①（finished 消费 + 销毁后 `_exit`）不变。
- **Stage B 修复方案**：—。
Evidence: A 级——reference a.java:4249-4256 + CMidlet.java:29-33 原文；paint.rs:709-721 / title.rs:382-415 / input.rs:91；
实机现场只读取证（白屏截屏 / 进程常驻 70 min / logcat -7 批）；宿主 RED（E0599）/ GREEN 3 测；`llvm-readelf` 见
`ANativeActivity_finish`（UND，NEEDED libandroid.so）与 `_exit@LIBC`。**本方实机验证待补**（手机在用，未占前台）。
Confidence: 90

> 【2026-10-27 实机旁证·用户操作】本方装包（14:13:01）并 `am start` 启动（14:13:10 出帧）后，用户自行在标题做返回手势——
> 本方未注入、只被动读 logcat，按 AGENTS §5 **不记为本方验证**：`14:13:55.110 back at title → 退出项确认（退出链）` →
> `tick batch=[Press(-5), Release, Release] → key=-5` → `bgm Stop`（进 mode 22）；`14:13:56.177` 动画期第二次返回 =
> `[Press(-7), Release]`（非标题态，无效果）；`14:14:00.483 exit chain: finished → ANativeActivity_finish`（距确认 5.37 s
> ≈ 71 拍 × 75 ms）→ `14:14:01.570 exit chain: activity destroyed → process exit` → ActivityManager `Process com.jjxy.game
> (pid 5420) has died: cch CRE`；crash 缓冲为空，前台回桌面。

> 【2026-10-27 实机复核·GREEN】（Xiaomi 14，本方运行；装机包内 .so sha256 `04f3dcf2…` = HEAD 构建，cargo 无待重编）
> ① 菜单路径：`am start` → 标题 → 方向盘左 `tick batch=[Press(-3), Release] → key=-3`（截屏轮播 = "退出"）→ 确定
> `14:30:35.251 key=-5` + `bgm Stop` → `14:30:40.606 exit chain: finished → ANativeActivity_finish`（5.36 s）→
> `14:30:41.695 exit chain: activity destroyed → process exit` → ActivityManager `Process com.jjxy.game (pid 17774) has died:
> cch CRE`；② 返回路径：`input keyevent 4` → `14:31:22.312 back at title → 退出项确认（退出链）` → `14:31:27.689 finish`
> （5.38 s）→ `14:31:28.776 process exit` → `pid 32051 has died`。两次 crash 缓冲均空、前台回桌面、`dumpsys activity` 无本包
> 记录。边缘手势 → KEYCODE_BACK 的系统转换本方未注入，由用户实操日志旁证（`input-arrival type=1` = 键事件）。
> 旁注（用户操作，14:30:44 自桌面启动）：开机 logo 期（mode 0）连按返回 4 次 = `[Press(-7), Release]` 无效果，到标题后
> 返回才进退出链——logo 期不认键是原版行为（reference a.java:3349-3381 mode 0 臂不读 keyValue，非本缺陷），如需"logo 期返回
> 即退"属另立的壳层交互决定。Confidence: 96

## BUG-034 Android 返回手势退出不符合用户习惯——开机 logo 期无效、主菜单返回进入 logo 退场动画（形似重播片头）且动画期无效（壳层交互设计缺陷，用户实机，已修复）

> 【2026-10-27】用户实机（Xiaomi 14，装 BUG-033 包）："1. 刚启动时到主菜单弹出前，右滑无效；2. 到主菜单后，右滑不退出，
> 进程内重置，重新播放片头；3. 再次进入主菜单，右滑退出成立……这不符合用户习惯"。
- **现象 / 复现**（用户操作，本方只读 logcat 取证）：
  会话 1（pid 4475，15:11:58 自桌面启动）：开机 logo 期返回 ×2（15:12:02/03）= `tick batch=[Press(-7), Release] → key=-7`
  无效果；15:12:05 到主菜单；15:12:21.937 `back at title → 退出项确认（退出链）` → `bgm Stop`（进 mode 22）；动画期点按
  左/右软键与返回（15:12:24/25/27）均无效果；15:12:27.322 finish → 15:12:28.403 进程退出 → AM `pid 4475 has died`；
  **15:12:28.511 桌面收到一次真实触摸**（`MIUIInput … deviceId=7`，`(417.8, 400)`，落在游戏图标——退出关闭动画的目标
  `ShortcutIcon(囧囧西游)`）→ `PreStartUtils: preLaunchProcess com.jjxy.game` → 15:12:28.533 `Start proc 9020` → 桌面重新启动
  游戏 ⇒ 用户所见"进程内重置、重新播放片头"实为：logo 退场动画 + 进程退出后图标被点中重启。
  会话 2（pid 9020）：logo 期返回 ×8（15:12:30.9–35.6）均无效；15:12:36.160 主菜单返回 → mode 22；动画期返回 ×4 无效；
  15:12:41.536 finish → 15:12:42.567 进程退出。另 15:12:21.112 一次横滑起点 x=147（≈49 dp）在系统边缘手势区
  （`GestureStubLeft [0,268,62,2594]`）之外 ⇒ 作为普通触点进游戏（`keys=[]`），非返回。
- **根因**（BUG-033 壳层映射的交互设计缺陷，非原版 bug，D4 不适用）：
  ① 开机 logo 期 BACK = -7，而原版 mode 0 臂不读 keyValue（reference a.java:3349-3381）⇒ 启动后约 7 s 返回无效；
  ② BUG-033 把主菜单返回映射为"选中退出项 + 确认"⇒ 进 mode 22 = 原版退场动画 `runLogoAnimation(1, t)` 70 拍 ≈ 5.3 s
  （a.java:4249-4256），与开机 `runLogoAnimation(0, t)`（a.java:3369）同一 SNOWFISH logo ⇒ 用户看到"片头重播"；mode 22
  同样不读键 ⇒ 动画期再返回无效。
- **影响**：Android 竖屏全部"返回即退出"操作（用户最常用的退出方式）。
- **Stage A 处理**：**已修复**——根界面（无场景且 mode ∈ {0 开机 logo, 1 主菜单, 22 退场动画}）BACK（DOWN、非重复）=
  立即退出：`Runtime::back_requests_exit` 置壳层退出请求、丢弃该 BACK 的 -7；主循环见 `exit_requested` 即不再出拍并
  `ANativeActivity_finish`，Activity 销毁后 `_exit(0)`（BUG-033 ① 不变）。语义 = Nokia 红键（AMS 随时销毁 MIDlet）：原版退出
  链 `CMidlet.m_000` = `destroyApp(true)`（空方法，CMidlet.java）+ `notifyDestroyed()`，无存档副作用（存档同步写穿）⇒ 跳过
  动画不丢状态。其余界面 BACK 仍 = 右软键 -7（读档/设置/帮助/关于子页 → 主菜单；序章 → 跳过；游戏内 → 游戏内菜单）。
  菜单"退出"项仍走原版退场动画（动画期可再返回立即退出）。游戏层零改动、无帧金样影响。
  > 【2026-10-27 更正】"菜单退出仍走原版退场动画"已被 **BUG-035** 取代（用户实机：菜单退出未结束进程）——现菜单"退出"
  > 确认即立即退出；exit_chain.rs 测试集随之改为 BUG-035 口径（"菜单退出动画播完才请求""退场动画期返回即退"两测已替换）。
  宿主钉 `shell/android/tests/exit_chain.rs` 5 测（菜单退出动画播完才请求 / logo 期返回即退 / 主菜单返回即退且不进 mode 22 /
  退场动画期返回即退 / 读档子页·序章·游戏内不劫持）。
- **Stage B 修复方案**：—。
Evidence: A 级——用户实操 logcat（上列时间线，含 MIUIInput/PreStartUtils/ActivityManager 系统侧行）+ reference a.java:3349-3381 /
3369 / 4249-4256 + CMidlet.java（destroyApp 空、m_000）原文；宿主 RED（E0599：`back_requests_exit` 不存在；父树
`title_back_runs_the_same_exit_chain` 绿 = 旧行为"返回 → mode 22 → >60 拍才退"）/ GREEN 5 测。**本方实机验证待补**。
Confidence: 92

> 【2026-10-27 实机复核·GREEN】（本方运行，Xiaomi 14；装机 .so sha256 `5933882f…` = 8bae944 构建）开机 logo 期注入左边缘
> 右滑（经系统返回手势通路：`input swipe 3 1300 650 1300 250`）→ `16:56:15.331 back at root mode=Boot → 立即退出` → finish →
> `16:56:16.345 exit chain: activity destroyed → process exit` → ActivityManager `pid 864 has died: cch CRE`；主菜单注入右边缘
> 左滑 → `16:56:42.192 back at root mode=MainMenu → 立即退出` → `16:56:43.267 process exit` → `pid 1660 has died: cch CRE`。
> 两次 crash 缓冲空、前台回桌面。Confidence: 96

## BUG-035 Android 主菜单选"退出"后进程迟迟不结束——仍走 5.3 s 原版 logo 退场动画（与开机同款，形似片头），用户等不到即离开（壳层交互缺陷，用户实机，已修复）

> 【2026-10-27】用户实机（Xiaomi 14，装 BUG-034 包 15:27:03）："右滑退出已经有了，但主菜单选择退出没有结束进程"。
- **现象 / 复现**（用户操作，本方只读 logcat 取证）：pid 31946（16:38:58 启动）：16:39:07.470 方向盘左 `key=-3`（光标落
  "退出"）→ 16:39:08.184 确定 `key=-5` + `bgm Stop`（进 mode 22）→ 16:39:12.431 `lifecycle hideNotify`（确认后 4.25 s 上滑离开）→
  16:39:12.710 ActivityManager `Force stopping com.jjxy.game … SwipeUpClean`、`Killing 31946 … due to SwipeUpClean`——全程无
  `exit chain` 行：进程是被用户在最近任务里划掉强停的，不是退出链结束的。
- **根因**（BUG-034 未覆盖菜单路径，非原版 bug，D4 不适用）：菜单"退出" = kind 5 → mode 22，原版 `runLogoAnimation(1, t)`
  70 拍 ≈ 5.3 s 后才 `CMidlet.m_000`（reference a.java:4249-4256）；该动画与开机 `runLogoAnimation(0, t)`（a.java:3369）同为
  SNOWFISH logo——用户此前已称之为"重新播放片头"（BUG-034）⇒ 不被认作"正在退出"，等不到 5.3 s。（核对：隐藏后主循环
  **照常出拍**——pump.rs `pump_once` 只按时钟出拍、无可见性门控——本次若不划掉，约 1 s 后仍会经退出链结束；故非"隐藏
  即永不退出"。）
- **影响**：Android 菜单"退出"的体感 = 播一段片头、进程不退。
- **Stage A 处理**：**已修复**——`Runtime::step_boot` 拍后见游戏进 mode 22（原版退出链入口：closeAudioPlayer 已执行）即置
  壳层退出请求：同拍 `exit_requested` ⇒ 主循环不再出拍、`ANativeActivity_finish`，销毁后 `_exit(0)`；退出已发后不再呈现，
  窗口保留主菜单末帧随关闭动画收起（免画出 mode 22 首帧白底闪一下）。与根界面返回（BUG-034）同口径，m_000 无副作用 ⇒
  跳过动画不丢状态。游戏层零改动、无帧金样影响（win 壳仍播原版动画）。
  宿主钉 `shell/android/tests/exit_chain.rs`：`title_menu_exit_item_exits_immediately`（确认同拍即退出请求、mode 22、BGM 已停）+
  `menu_exit_after_returning_from_game_exits_immediately`（新游戏 → Map → 游戏内菜单"退出" kind 7 → 主菜单 → 菜单退出）。
- **Stage B 修复方案**：—。
Evidence: A 级——用户实操 logcat（上列，含 ActivityManager SwipeUpClean 强停行）+ reference a.java:3369 / 4249-4256 +
pump.rs `pump_once` 原文；宿主 RED（两测均断在"确认即退出请求"：`确认即退出请求（不等退场动画）` /
`回主菜单后菜单退出 = 立即退出`）/ GREEN 5/5。**本方实机验证待补**。Confidence: 93

> 【2026-10-27 实机复核·GREEN】（本方运行，同包）方向盘左 → 确定：`16:55:55.722` 同拍 `menu exit → 立即退出（不播 mode 22
> 退场动画）` + `bgm Stop` + `exit chain: exit requested → ANativeActivity_finish` → `16:55:56.764 process exit` → ActivityManager
> `pid 501 has died: cch CRE`；点确定到进程消失 1.18 s（旧版 = 5.3 s 动画 + 约 1 s）。crash 缓冲空。Confidence: 96

## BUG-036 Android 退出后约 1 s 内再点图标 ⇒ 新开的游戏被旧实例 `_exit` 一并杀掉、系统再重启一遍（BUG-033 `_exit` 时机缺陷，用户实机，已修复）

> 【2026-10-27】BUG-035 取证时于同段 logcat 发现（用户 16:38–16:40 反复退出/进入）；用户感知 = "又从片头重来一遍"。
- **现象 / 复现**（用户操作，本方只读 logcat 取证）：pid 32119：16:39:44.343 主菜单返回 → 16:39:44.367 `exit chain: exit
  requested → ANativeActivity_finish` → 16:39:45.384 ActivityTaskManager `START … cmp=com.jjxy.game/android.app.NativeActivity`
  （用户再点图标）→ 16:39:45.407 **同 pid 第二次** `boot android_main entered`（新实例进了将退出的旧进程）→ 16:39:45.829 旧实例
  `exit chain: activity destroyed → process exit` → 16:39:45.865 ActivityManager `Process com.jjxy.game (pid 32119) has died:
  fg  TOP` → 16:39:46.826 `Start proc 32638`（系统另起进程，从开机 logo 再来一遍）。另 16:38:43.460（pid 11794，`START` 后
  0.10 s；16:38:43.357 `The Process com.jjxy.game Already Exists in BG. So sending its PID: 11794`）/ 16:38:48.492（pid 12244，
  `START` 后 0.50 s）/ 16:38:58.708（pid 12667，`START` 后 0.05 s）三次 `has died: fg TOP` 同型——该段游戏侧日志已轮转，只余
  ActivityManager 行（B 级）。
- **根因**（BUG-033 移植设计缺陷，非原版 bug，D4 不适用）：BUG-033 在 `android_main` 尾"本实例发起过退出 ⇒ `_exit(0)`"。
  native_app_glue 支持同进程多实例（每次 `ANativeActivity_onCreate` 起一条新 `android_main` 线程）；退出后、旧实例销毁前
  （finish → onDestroy 约 1 s）用户再点图标，系统复用尚存的旧进程承载新实例 ⇒ 旧实例 `_exit` 连新实例一起杀掉 ⇒ 前台进程
  死亡，系统另起进程重启。
- **影响**：退出后约 1 s 内再进游戏 ⇒ 多一次闪退重启（再看一遍开机 logo）；连续操作时可连环发生。
- **Stage A 处理**：**已修复**——`process::InstanceCounter`：`ANativeActivity_onCreate`（主线程，早于该实例任何生命周期回调）
  登记、`android_main` 末注销；`process::end_process_after_destroy(exit_sent, remaining)` = 本实例发起过退出 ∧ 进程内已无其他
  实例才 `_exit(0)`，否则正常返回（glue 收尾、新实例继续）并记 `keep process` 日志。同进程再入安全性：Android 构建的进程级
  全局只有字形钩子（`render::GLYPH_HOOK`，同函数重设）、字体 `OnceLock`（重复 install 沿用首次）、`resource_cfg` `OnceLock`
  （只读）——均可重入（`ADVANCE_HOOK` 仅 core 单测设置）；16:39:45.407 第二实例初始化日志完整（storage / aaudio open ok）为旁证。
  宿主钉 `shell/android/tests/process_exit.rs` 4 测（单实例退出即结束 / 退出中再入保留进程、后者退出再结束 / 非本方销毁不结束 /
  计数不下溢）。
- **Stage B 修复方案**：—。
Evidence: A 级——pid 32119 时间线（同 pid 双 `android_main entered` + `has died: fg TOP` + 新 pid 重启）；B 级——三次 ActivityManager
同型迹象；全局状态清单 = `grep OnceLock|static mut|thread_local` 于 crates/ 与 shell/；宿主 RED（E0432：`InstanceCounter` /
`end_process_after_destroy` 不存在）/ GREEN 4/4。**本方实机验证待补**。Confidence: 90

> 【2026-10-27 实机复核·GREEN】（本方运行，同包）主菜单返回后 0.34 s 再 `am start`：`16:57:07.570 back at root mode=MainMenu`
> → `16:57:07.573 finish` → `16:57:07.908 START` → `16:57:07.931` **同 pid 6453 第二次** `boot android_main entered`（竞态复现）→
> `16:57:08.380 exit chain: activity destroyed, 1 other instance(s) alive → keep process` → 新实例 `16:57:15.040` 到主菜单（截屏
> 标题与面板正常、`aaudio open ret=ok`、字体沿用 `font install None in 0ms`）；随后该实例菜单退出 → `16:58:01.178 process exit`
> → ActivityManager `pid 6453 has died: cch CRE`。全程无 `fg  TOP` 死亡、crash 缓冲空。Confidence: 95

## BUG-040 点地寻路收尾不完整：途中按键打断后落进剧本"行尾收尾"写剧情旗标；单步寻路后脚本行走武装残留、下一步手动行走停不下来（Stage B 触屏寻路，2026-10-28 对抗复核发现，已修复）

> 【2026-10-28】设计 v0 对抗复核（`docs/reviews/2026-10-28-settings-controls-design-adversarial-review.md` §3 探针 P3）发现症状①；
> 修复取证时由单测发现同根症状②。
- **现象 / 复现**：
  - ① 点地寻路途中按任意键打断（地图非空闲时点画面也发 -5，同路）：相位轨迹 `Dialog/dp3 → Dialog/dp0 → Dialog/dp4 → Map/dp4`。dp0 执行
    `execute_script_instruction` 的行尾收尾：`script_line_flags[current_script_index] = true` + `lock_camera_on(0)` + dp4 等镜头回位。
    当前脚本旗标原为 false 时会误关该剧情的门（例：读档后 current_script_index=0、脚本行为空 ⇒ 写 flags[0]，0 号门见
    `try_run_scene` scene.rs:1623——可达性 B 级）。
  - ② 点相邻格（单步寻路）后再按一下方向键：连走 3 格仍 walkPhase 1，停不下来（单测实录 `(walk_phase, x, px&31) = (1, 5, 0)`，
    期望 `(0, 3, 0)`）。Map 态 walkPhase 1 步进不判墙（B 级推断：持续滑行）。
- **根因**（T2 移植设计缺陷，非原版 bug，D4 不适用）：WalkTo 复用脚本行走机器 dp3，dp3 两处收尾只凭 `route.is_open()` 认寻路。
  - 打断（`interrupt_route` 收束寻路段）后 dp3 失去寻路归属，于是走了脚本机收尾（①）；
  - 寻路回 Map 时只切模式、不解除 `script_walk_armed`。单步寻路的唯一一步由 dp3 wp0 弹出，不经 `advance_script_walk` 清武装，
    残留的武装让 Map 态下一次手动步进在步末走 `advanceScriptWalk` 臂：无步可弹，也不把 walkPhase 归 0（②）。
- **影响**：① 剧情旗标误写（可能锁住剧情），打断后多一段镜头回位等待；② 点地寻路之后的第一次手动行走失控。
- **Stage A 处理**：**已修复**。
  - 新增 `walkto_draining`：`interrupt_route` 时若处于 Dialog dp3 则置位；新 `walk_to`、剧情接管 `load_level_script` 时清位。
  - `walkto_owns_dp3()` = 段开放 ∨ 收尾中。
  - dp3 两处收尾统一走 `finish_walkto()`：段终位入栈 + 清收尾位 + 解除武装 + 回 Map。
  - 经典脚本行走（段未开放、未收尾）零改动；剧情接管后 dp 仍归剧情（新测钉住）。
  - 宿主钉：scene.rs `walkto_interrupt_tests`（2）、`walkto_single_step_tests`（1）。
- **Stage B 修复方案**：—。
Evidence: A 级——探针 P3 运行时实录（走 Android 生产路径 `Runtime::step`）；单测 RED：`walkto_interrupt_drains_to_map_without_script_tail`
轨迹 `[(Dialog,3)×4, (Dialog,0), (Dialog,4), (Map,4)]`、`single_step_walkto_then_manual_step_stops_at_next_cell` `(1,5,0) ≠ (0,3,0)`；
GREEN：game-core lib 175/175，game-core / game-shell-common / game-android / game-desktop 共 528 测全绿。B 级——①的有害可达性。
Confidence: 90

## BUG-038 触屏点按穿透浮层 / 场上非空闲时仍执行场上动作：框开着点地图会在框后挂寻路或起步撞格；道具确认框开着点别的格会用掉被点那件（Stage B 触屏层，2026-10-28 对抗复核实测，已修复）

> 【2026-10-28】设计 v0 对抗复核（`docs/reviews/2026-10-28-settings-controls-design-adversarial-review.md` §3 探针 P1a/P1b/P2）。
- **现象 / 复现**（Android 生产路径 `Runtime::step → step_scene → native::scene_tap`，宿主真资产）：
  - P1a：地图上弹着提示框时点可走格 ⇒ 框仍开着，mode 却 Map→Dialog dp3（寻路挂在框后，关框瞬间按键又打断它）。
  - P1b：同上点墙格 ⇒ walkPhase 0→1（框开着时 `try_step` / `interact_with_cell` 已执行，可拾取、开门、开战）。
  - P2：芭蕉扇使用确认框（kind 3，○/×）开着时点 1 号格 ⇒ 光标改到 1 号、千年月桂露被用掉（HP 300→448，次数 1→0）。
  - 同理（B 级，读码）：
    - Windows 鼠标悬停在确认框下的格子上会挪光标，之后按 Q（-6）用掉的是悬停那件；
    - 行走中、探索、战斗中点地会照样 `walk_to` / `probe_step`（不清 step_progress，可离格或打断战斗）。
- **根因**（T2/T3 触屏移植缺陷，非原版 bug，D4 不适用）：原版浮层打开期间场上不吃任何输入（a.java:3252-3345 浮层分支吞键后整拍返回），行走、战斗相位也不读场上输入（walkPhase 分派 a.java:3563-3679）。
  - 壳层点按只判 `game_mode == Map` 就走场上通道（native.rs `scene_tap`、win `pointer_tap`）；
  - core 的 Stage B 入口 `walk_to` / `probe_step` 不判场上是否空闲；
  - `pointer_hit` 不看浮层（scene.rs pointer_hit；Windows pointer_hover 同源）。
- **影响**：误用道具（不可撤）、框后误寻路、弹框期执行场上交互。
- **Stage A 处理**：**已修复（core 入口）**。
  - `GameScene::field_tap_allowed()`：无浮层 ∧ 非遮幅 ∧ walkPhase 0 ∧（Map 且未武装 ∨ 点地寻路自身的 dp3 处于格边界）。
  - `walk_to` / `probe_step` 不满足即拒绝，零状态变化。
  - `pointer_hit` 在浮层打开时恒 None。
  - 双壳同受益，壳层未改。点画面在弹框 / 列表里应做什么，见 BUG-039。
  - 宿主钉：scene.rs `touch_tap_guard_tests`（3）；`shell/android/tests/tap_routing.rs`（3，P1a/P1b/P2 生产路径）。
- **Stage B 修复方案**：—。
Evidence: A 级——探针实录；`tap_routing.rs` 三测在修复前 core 上 RED（P1a mode=Dialog、P1b walkPhase=1、P2 光标 1 + 道具已用），修复后 GREEN；
core 单测 RED = E0599（`field_tap_allowed` 不存在）→ GREEN。game-core / game-shell-common / game-android / game-desktop 534 测全绿。Confidence: 92

## BUG-039 点空白恒代按确定：存档页点空白覆盖当前槽、炼丹炉点空白花钱、游戏内菜单点空白执行高亮项（Stage B 触屏 / 鼠标，2026-10-28 对抗复核发现，已修复）

> 【2026-10-28】设计 v0 对抗复核（`docs/reviews/2026-10-28-settings-controls-design-adversarial-review.md` §4 S3②）。
- **现象 / 复现**：点按未命中任何行 / 格时，一律代按确定 -5。涉及三处：Android `native::scene_tap` 行外臂、`runtime::step_boot`、
  Windows `click_key`（非 Map 态）。
  - 宿主实测（`shell/android/tests/tap_routing.rs`，修复前 RED）：存档页点 (2,2) 空白 ⇒ 写入 `MOT_L0@1` 并弹"保存成功！"。
  - 同理（读码）：
    - 炼丹炉 = 花钱买高亮属性；
    - 游戏内菜单 = 执行高亮项（"退出"、"跳过教程"）；
    - 道具栏 = 选中高亮道具；
    - 设置页 = 切换高亮行；
    - 标题侧读档列表 = 读高亮槽。
  - 反向缺口：BUG-038 修复后，地图上弹着的普通提示框点画面没有反应（点按仍走场上通道、被拒）。
- **根因**（WIN-OP③ / T3 设计缺陷，非原版 bug，D4 不适用）："点画面 = 确定"是为对话推进设计的，但被一刀切用于所有非地图态；
  原版只有对话、提示框是"任意键继续"语义。
- **影响**：误覆盖存档（不可撤）、误花钱、误执行菜单项。
- **Stage A 处理**：**已修复**。
  - core 单源 `GameScene::blank_tap_key()`：
    - 对话、结局、普通提示框 → Some(-5)；
    - 原版 ○/× 脚注标明的是否框（道具使用确认 kind 3、商人报价 kind 4）→ None；
    - 要做决定的列表、地图、换层遮幅期 → None。
  - core 单源 `BootMachine::blank_tap_key()`：标题（只显示当前一项，点即确认该项——Windows 鼠标基线）、序章快进 → Some(-5)；
    读档列表、设置等 → None。
  - Android `scene_tap` 改为：场上空闲走场上通道（`field_tap_allowed`，BUG-038），否则先命中行，再取点空白键；`step_boot` 同用。
  - Windows：`pointer_tap` 只在 `field_tap_allowed` 时走场上通道；`click_key` 的点空白取活动机 `blank_tap_key`。
  - **行为变化**（含 Windows 鼠标）：上述列表界面点空白不再确认，改为点行；炼丹炉没有行命中，用按键或面板"确定"。
  - 宿主钉：
    - scene.rs `blank_tap_tests`（1）、paint.rs `boot_blank_tap_continues_only_in_title_and_intro`（1）；
    - `tap_routing.rs` 新增 3 测（提示框点击关闭 / 是否框点击无效 / 存档页点空白不写档）；
    - `native_parse.rs` 游戏内菜单点空白断言由 `Some(-5)` 改 `None`；
    - Windows `click_key` 断言按新签名改写。
- **Stage B 修复方案**：—（STAGE-B-007 功能键面板会把点画面规则并入同一查询）。
Evidence: A 级——`tap_routing.rs` 存档页 / 提示框两测、`native_parse.rs` 菜单点空白断言在修复前 RED（写入 `MOT_L0@1`、提示框未关、`Some(-5)`）；
core 两测 RED = E0599（`blank_tap_key` 不存在）。GREEN：game-core / game-shell-common / game-android / game-desktop 539 测全绿。Confidence: 92

## BUG-041 游戏内关掉声音后退出回标题，声音被回滚为开、标题曲照响（双机交接只有单向；Android / Windows 同病，2026-10-28 对抗复核发现，已修复）

> 【2026-10-28】设计 v0 对抗复核（`docs/reviews/2026-10-28-settings-controls-design-adversarial-review.md` §4 S1，复核者 B 读码提出，本方宿主实测证实）。
- **现象 / 复现**：新游戏 → 游戏内菜单 → 设置 → 关"声音" → 返回 → 退出（kind 7）回标题 ⇒ 声音开关变回开，标题曲 track 2 照响。
  - Android 宿主实测（`shell/android/tests/flow.rs` `in_game_sound_off_survives_return_to_title`，修复前 RED：`!rt.sound_enabled()` 失败）。
  - Windows 宿主实测（`return_to_title_keeps_in_game_sound_off`，修复前 `sound_state` = `Some(true)`）。
  - Windows 还会在下一拍 `persist_settings_if_changed` 把"开"写回 `app_settings.cfg`（读码）。
- **根因**（双机移植设计缺陷，非原版 bug，D4 不适用）：Java 是同一个对象，soundEnabled / volumeLevel / SKY_WAR（含 f_int_154 水位）跨模式连续。
  - Rust 拆成 BootMachine 与 GameScene 两台机：交接只有 BootMachine→GameScene 单向拷贝（handoff.rs `new_game_scene` / `slot_scene`）。
  - 回标题时，两壳都换回交接前那台 BootMachine（Android runtime.rs step_scene、Windows `swap_back_to_boot_if_requested`），它带着旧的 bgm 与 global。
  - 同理（读码）：之后若在标题侧切换设置置脏，会把陈旧的 global（旧水位）写回 SKY_WAR。
- **影响**：用户关掉的声音在回标题后自行恢复（Windows 连设置文件一起改回）；SKY_WAR 水位可能被旧值覆盖。
- **Stage A 处理**：**已修复**。
  - 新增 `handoff::return_to_title(machine, scene)`：回传 BgmState 与 global 快照（`apply_global`，不置脏，因为场景侧已写穿）。
  - 两壳在换回 BootMachine、`enter_title` 之前调用。
  - 未纳入：rng / bob / 图标条帧 / 标题光标等视觉相位（不影响状态正确性，另案评估）。
  - 宿主钉：handoff.rs `return_to_title_carries_sound_and_global_back`、Android `flow.rs` 一测、Windows shell_tests 一测。
- **Stage B 修复方案**：—。
Evidence: A 级——Android / Windows 两条宿主测试在修复前 RED（声音回滚为开），修复后 GREEN；handoff 单测 RED = E0425（`return_to_title` 不存在）。
B 级——陈旧水位写回 SKY_WAR 的路径（读码）。GREEN：game-core / game-shell-common / game-android / game-desktop 542 测全绿。Confidence: 92

## BUG-042 点地寻路基本不走：触屏格换算用 16 px（瓦片层子瓦片），游戏格是 32 px（Stage B 触屏 / 鼠标，用户实机，已修复）

> 【2026-10-28】用户实机："测一下触屏效果，我试过，无法触发走动"。
- **现象 / 复现**：Android 竖屏进地图后点画面上的空地格，悟空不动。
  - 设备日志（本方注入）：`scene_tap(292,324) mode=Map → None`——点按已走到场上通道（`field_tap_allowed` 为真），但无寻路。
  - 宿主复现（真资产、设备同档布局 12×16）：点游戏绘制在 (80,370) 的空格 (2,10)，`route.is_open()` 恒假。
  - 同一会话用户另一次尝试落在 scene mode 0（hideNotify 菜单 -7 = 已登记 BUG-014，原版保真），与本条无关。
- **根因**（Stage B 壳层移植错误，非原版 bug，D4 不适用）：`shell/common/src/touch.rs` `TouchGeom::TILE = 16`，注释"paint_tiles 16px 步进"。
  - `GameView::paint_tiles` 画的是**子瓦片**（每行 `wide*2` 块、16 px）；游戏格是 32 px：`player_cell_x = player_px >> 5`（scene.rs:960）、
    实体 `cx << 5`（scene.rs:2612）、`map_w_px = wide << 5`（game_view.rs:119）；`walkable` / `walk_to` / `probe_step` 都用这个格网。
  - 用 16 换算 ⇒ 格号约为真实的 2 倍：画面右半 / 下半的点按越界被忽略（例：x 292 → 列 18 ≥ 12），其余落到错误格（常为墙 ⇒ 探步撞墙）。
    10-26 实机记录的"点地寻路拾钥匙"可解释为点到左上区域、走到了别的可走格（B 级推断），当时只验"有没有走"，没核对落点。
  - 旧测试抓不到：点位用同模块的 `cell_screen_xy` 求（与被测同错），或手写 16 px 字面量。
- **影响**：Android 点地寻路 / 点墙探步、Windows 鼠标点地（`pointer_tap` 同用 `TouchGeom`）基本不可用或走错格。
- **Stage A 处理**：**已修复**。`TILE = 32`（日期更正注）。7 条旧测试按 32 px 改写字面量（原意不变）。
  - 宿主钉：`shell/android/tests/tap_routing.rs` `tap_on_drawn_walkable_cell_walks_player_there` / `tap_walks_to_drawn_cell_with_camera_offset`
    ——点位按**游戏绘制公式**求（相机 + view_top + 格 × 32 + 16），依次点可达范围内每个空格、须恰好走到被点的格，第二例先把相机移到 (37,-45)。
- **Stage B 修复方案**：—。
Evidence: A 级——只改常量 16 ↔ 32：新测试 RED（`route.is_open()` 假）↔ GREEN；变异实测：TILE=16、相机 x/y 对调、`cell_at` 列号 +1、
tile_off=0 均被杀；cols/rows 对调存活（12×13 图上只差边界一列，界内不可观测，如实记录）。设备日志 `scene_tap(292,324) mode=Map → None`。
B 级——格单位由 `>> 5` / `<< 5` 调用图推出。GREEN：game-core / game-shell-common / game-android / game-desktop 全绿。Confidence: 93

## BUG-043 教程路径箭头在点地寻路时错位（虚拟摇杆走正常；Stage B 触屏，用户实机，已修复）

> 【2026-10-28】用户实机："教程关中有绘制路径箭头的情况，当箭头存在时，用触屏移动会导致箭头错位，用虚拟摇杆移动不会"。
- **现象 / 复现**：序章首图出生即有箭头 ↑→→（SEE flag=1）。点箭头上的第 2 格：
  - 宿主实测（修复前）：点下那一拍箭头改画成以玩家为起点的 →→（(2,11)(3,11)，墙上），走完第一步箭头熄灭；方向键走到同一格余下 (3,10)。
- **根因**（Stage B 移植错误，非原版 bug）：箭头与脚本行走共用 `walkPathBuffer`（原版 MOV 亦然）。点地寻路 `walk_to` 驱动 dp3 脚本行走、
  覆写这个缓冲 ⇒ 绘制（a.java:5929-5956，自玩家格沿缓冲外扩）画成寻路路线；label216（a.java:5748-5761）按寻路步消费 / 熄灭箭头。
- **Stage A 处理**：**已修复**（只动点地寻路路径，经典面零变化）：
  - `walk_to` 开路时把可见箭头步序移入暂存 `walkto_arrows`；绘制与 `path_arrow_cells()` 读暂存；
  - label216 在暂存存在时作用于暂存，比较"刚走完那一步"的朝向（寻路下一步已在步末弹出转向；方向键逐格时二者一致）；
  - 寻路收尾 / 被剧情接管后放回 `walkPathBuffer`（剧情接管延迟到寻路释放 dp 后放回，撞格触发剧情时立即放回会被随后的
    `walk_step_count = 0` 清掉）；撞锁门 / 打不过两处原版清共用 `walkStepCount` 的位置同步清暂存；`find_path` 作废暂存。
  - 语义口径：点地 = 方向键**逐格**沿同一路线走（按住摇杆过拐角时原版会熄箭头，属原版行为，点地不模仿）。
  - 宿主钉：core `bug043_*` 7 条（沿箭头 / 偏离 / 打断 / 收尾缓冲同参照 / findPath 作废 / 撞剧情格 / 撞锁门）、android
    `tap_routing.rs` `tutorial_arrows_stay_on_their_path_during_tap_walk`（真资产，逐拍箭头须是原路线尾段、终态 = 方向键参照）。
- **另案（复核 R4，修复前既有，未修）**：`tick_battle` 击杀触发剧情后无条件 `route.resume()`，会把已被剧情收束的寻路段重新打开
  （BUG-025 同型风险）。待取证后单独处理。
  > 【2026-10-29 更正】已登记为 BUG-048 并修复（见 BUG-048 节）。
- **Stage B 修复方案**：—。
Evidence: A 级——宿主 RED（修复前箭头格 (2,11)(3,11) 不在原路线上；真资产终态 [] ≠ 方向键 [(3,10)]）→ GREEN；变异：`walked_facing`→`facing`、
撞门不清暂存、剧情接管立即放回、findPath 不清暂存均被杀；实机（Xiaomi 14，本方注入点按 (2,10)）连拍：↑→→ → 走动中 →→ 留在原格 → 到达后余 (3,10)。
classic：game-oracle 帧差分与 Windows modern 五状态帧不变。Confidence: 90

## BUG-044 点地寻路首步不判碰撞结果：首步被挡且不弹浮层时玩家"穿格"（站上怪物 / 炼丹炉、穿过隐形墙；Stage B 触屏，2026-10-29 spec 对抗复证实测，已修复——余项①② 随 STAGE-B-008 生产路径消除、③ 已修）

> 【2026-10-29】双操控方案 spec 对抗复核（`docs/reviews/2026-10-29-dual-control-schemes-spec-adversarial-review.md` C1，探针 A / E / F / I）。
- **现象 / 复现**（宿主探针，真资产，生产入口 `walk_to`）：
  - 1 层玩家 (8,2)，(9,2) 放型 70，hp=1（打不过），`walk_to(10,2)` ⇒ 玩家站到怪物格 (9,2)，未开战；
  - 23 层玩家 (5,3) 紧邻隐形墙 (5,4)，`walk_to(5,5)` ⇒ 轨迹 (5,3)→(5,4)→(5,5)，穿墙后墙才显形为型 11；
  - (9,2) 放炼丹炉型 9，`walk_to(10,2)` ⇒ 站上炉格；炉在正上方时 `walk_to` 炉格 ⇒ 进炼丹炉页时 wp=1 残留，离开后走进炉格。
- **根因**（Stage B 移植用法错误）：点地寻路借用 dp3 脚本行走消费机。wp0 臂先置 `walk_phase = 1` 再 `try_step`、不看返回值
  （`scene.rs:2903-2907`，原版 MOV 消费 a.java:4080-4084 同式——原版脚本路线由剧本保证可走，不会撞格）。`try_step` 只在
  "有浮层 ∧ wp==1" 时复位相位（`scene.rs:5156-5177`），所以首步撞上不弹浮层的阻挡（打不过的怪——Rust 缺浮层见 BUG-046、隐形墙、
  炼丹炉、联动门 32、实墙）时玩家照样走进去。第 2 步起走 `advance_script_walk`，撞格清步，表现正常（探针 C）。
- **Stage A 处理**：classic 不受影响（经典脚本行走的路线由剧本给出）。原版 MOV 消费机保持不动。
- **Stage B 修复方案**：点地寻路的首步 / 0 步撞击不经 dp3 wp0：0 步撞击在 Map walkPhase 0 直接 `try_step`（并开 route 段，保证撤销记账）；
  ≥1 步时撞击作为缓冲末步走 `advance_script_walk` 臂。纳入双操控方案（STAGE-B-008）前置任务，测试矩阵 =
  {怪（可胜 / 不可胜）、门（有 / 无钥匙）、炉（朝上 / 横向）、墙、隐形墙、剧情点、联动门} × {邻接、远端}，断言玩家格与相位。
Evidence: A 级——复核探针运行时实录（探针已删，复现条件见复核记录 §1）；`scene.rs:2903-2907` 逐行核对（本方 2026-10-29 复看）。Confidence: 85

> 【2026-10-29 更正】（第二轮复核 R2-3 / R2-13）
> - 范围不止"首步"：凡经 dp3 wp0 弹出的步都不判返回值——首步、踩 60 扣血后（`step_triggered` 时步末不调 `advance_script_walk`）的下一步、
>   wp4 开门动画结束后的下一步（后两者 B 级，未构造实例）。
> - 阻挡清单中的"实墙"不成立：find_path 只经"地形可走 ∨ 有槽"的格，纯地形墙不会成为路线上的步。
> - 炉例在 debug 构建下会在 `transition_to` 的 `debug_assert` 处 panic（Dialog → Alchemy 不在 `TRANSFER_EDGES`，`scene.rs:711-779`、`:1503`）。
> - 修复方案改为：dp3 wp0 臂加**仅点地寻路占用（`walkto_owns_dp3()`）时生效**的分支——弹步 → 置 wp1 → `ok = try_step`；`!ok` 且未开战则清步、
>   置 wp0，下拍 `finish_walkto`。经典 MOV 不占用，trace 零变化。另，点地的撞击（门 / 怪 / 炉等目标）一律在路线结束、回到 Map wp0 后执行，
>   不作为路线末步（炉页会被 `finish_walkto` 吞掉，R2-2）。
- **修复**（2026-10-29，计划 1 T2）：dp3 wp0 臂加点地寻路专用分支——弹步前记归属 `owned = walkto_owns_dp3()`；`try_step` 返回 false 且 wp 仍为 1
  （未开战、无浮层复位）⇒ 清步、归 wp0，下拍 `finish_walkto` 回 Map。被挡步把模式切走（朝上撞炉 ⇒ Alchemy）时就地收尾（收段、放回箭头、
  解除武装，不写回 Map 以保留炉页）。用弹步时的归属判定，是因为撞"撞格即触发"的剧情点（型 83）时 `load_level_script` 先收段（复核 T2-2）。
  宿主钉（core，8 条）：`bug044_tap_walk_first_step_into_blocker_stays_put`（炉 / 隐形墙 / 打不过的怪）、`bug044_tap_walk_first_step_into_winnable_enemy_still_fights`、
  `bug044_classic_script_walk_keeps_original_unchecked_first_step`（经典 MOV 保真钉，去门控即红）、`bug044_first_step_into_scene_trigger_matches_direction_key`、
  `bug044_step_after_trap_damage_into_blocker_stays_put`、`bug044_step_after_open_group_animation_into_blocker_stays_put`、
  `bug044_tap_alchemy_above_leaves_no_armed_walk`（release 运行，见余项①）。变异（整支失效 / 门控改为事后归属 / 去模式切换收尾 / 去 wp==1 / 去 wp=0）均被杀。
- **余项（未修，归 STAGE-B-008 计划 2"撞击一律在 Map wp0 执行"）**：
  ① 点地路线朝上进炉（首步或途中）走 Dialog → Alchemy，该边不在 `TRANSFER_EDGES` ⇒ debug 构建 `transition_to` 的 `debug_assert` panic（release 不受影响）；
  ② 途中一步朝上进炉（`advance_script_walk` 臂）后同拍被 `finish_walkto` 写回 Map，炉页被吞（spec 第二轮复核 R2-2）；
  ③ 被挡后收尾落零长位移笔（from == to），撤销要多按一次空操作（复核 T2-3；spec §5.2 定为零长笔不入栈）。
- **classic 口径**（复核 T2-6）：经典脚本行走（MOV）不占用 dp3，按键链与 trace 不经新分支；Windows classic 预设下的鼠标点地属 Stage B 输入，同样走新分支。

> 【2026-10-30 收口 · STAGE-B-008 计划 2】①② 生产路径不再出现：壳层点地改走 `tap_cell`，撞击一律在 Map wp0 执行（不进 dp3 路线），
> 炉自正下方朝上撞直接 Map → Alchemy（`tap_cell_alchemy_from_below_opens_page`）。旧入口 `walk_to` 仅剩宿主测试 / 脚本面调用，未改。
> ③ 已修：`RouteTracker::close` 零长段不入栈（`zero_length_segment_is_not_pushed`）。提交：151e3e3（③）/ 0aab8bb（tap_cell 撞击口径）/ 5a3e367（壳层改接）。

## BUG-045 撤销对战斗的逆放损坏，且撤销日志从不清栈（Stage B 撤销，2026-10-29 spec 对抗复核实测，已修复；剧情接管另见 BUG-048）

> 【2026-10-29】双操控方案 spec 对抗复核 C2（探针 B / D）。
- **现象 / 复现**（宿主探针，生产入口 `walk_to` → 战斗 → `undo_last`）：
  1. 被杀怪物死亡动画结束后已摘格槽（`removed = true`）。`revive_enemy`（`shell/common/src/undo_ui.rs:36-46`）只写 type / state=0 / visible，
     不恢复 `removed`、不重登格槽 ⇒ 撤销后怪物不画、不挡路：等于白杀一只还回血回金。
  2. `revive_enemy` 以**实体槽号**写 `enemies.hp`，而该表按 `type − 41` 索引 ⇒ 写坏全局怪物属性表（实测型 67 HP 1200→0）。
  3. 战斗快照在 `battle_enemy_hp` 赋值之前取值（`scene.rs:5413` 取、`:5422` 赋），记下的是旧值。
  4. 生产代码从不调用 `UndoJournal::clear` / `begin_combat`（仅测试调用）⇒ 换层 / 存档 / 剧情后仍可撤回；
     1 层寻路后换到 2 层再撤销，玩家被放到 1 层坐标 (8,2)。
- **根因**：touch-native-undo（T1/T2）撤销日志的逆放与清栈未接到生产路径；终审"清栈 ✅"只测了 journal 本身（测试自证）。
- **Stage A 处理**：classic 不受影响（撤销是 Stage B 新增功能）。
- **Stage B 修复方案**：复活恢复 `removed` 与格槽；HP 回写战斗 HP 语义而非属性表；快照取赋值后值；换层 / 读档 / 存档 / 剧情事件接 `clear`。
  测试走生产路径：`walk_to` → 战斗 → `undo_last` → 再撞同格必须开战；换层后 `undo_last` 无效。纳入 STAGE-B-008 前置任务；
  「自动战斗」开关排在本修复之后。
Evidence: A 级——复核探针运行时实录（探针已删）；`undo_ui.rs:36-46` 逐行核对（本方 2026-10-29 复看：`enemies.hp.get_mut(e.slot)`）。Confidence: 85
- **修复**（2026-10-29）：开战时把整格 `Entity` 存进撤销快照（改 state 之前取）；撤销按快照整格还原，已摘槽则 `entity::reattach_to_cell`
  重登主格；不再写 `enemies.hp`；路段开放的开战调 `begin_combat`（战斗中撤销 = Blocked）；`load_floor_data` / 存档 / `load_level_script`
  清栈。宿主钉：shell-common `bug045_undo_after_tap_battle_revives_enemy_in_place`（生产路径 walk_to → 战斗 → undo_last → 再撞即开战）、
  `bug045_undo_is_blocked_mid_tap_battle`；core `bug045_journal_clears_on_floor_change_save_and_script`。变异：删 `reattach_to_cell`、
  删 `begin_combat`、逐一删三处 `clear`、恢复 `enemies.hp` 按槽号写入均被杀（最后一项初版存活——夹具怪物恰在实体槽 0、型 41，表下标同为 0、
  原值恰等于复活血；测试改为先在该下标放哨兵值后被杀）。
  余项：怪物紧邻时开战，`route_event_suspend` 压一笔零长位移笔（既有行为，归计划 2"零长笔不入栈"）。
- **复核补修**（计划 1 T3 复核 `target/verify/review_plan1_T3.md`）：
  - R1/R2 不可逆生成：击杀掉落（boss 序 `spawn_boss_event`）/ 49 层 61 机关在战斗期间生成实体（实体表 append-only，计数增长）⇒ 不入战斗笔、
    清栈（`ClearReason::Irreversible`），续走照旧。修复前"击杀 → 撤销"循环可无限复制掉落，16 次后 `slot_push` 越界 panic（复核探针 G）。
  - R3 实体表重排：`reload_floor_entities`（跨层探针往返）清栈；`revive_enemy` 加身份校验（槽内须仍是同型同位的尸体），不盲写别的实体。
  - R4 未记账的消费：炼丹购买、商人交易、道具使用清栈（撤销按战前绝对值还原血金会白送回花掉的钱；B 级推断，按保守口径处理）。
  - R5/R6/R8：复活整格字段断言、`reattach_to_cell` 幂等（未摘槽即撤销不重复登记）、撤销后玩家位置由断言而非手工摆位。
  - R7（未改）：重登格槽追加到末尾，同格多实体时槽序可能变化——仅保证单占格等价（代码注释已写明）。
  - 宿主钉新增：core `bug045_kill_with_spawned_drops_pushes_no_combat_stroke`（68 对照 41）、`bug045_journal_clears_on_reload_and_unrecorded_spending`、
    `bug045_reattach_is_idempotent_when_still_in_cell`；shell-common `bug045_revive_skips_slot_holding_another_entity`。
    变异（去计数判据 / 去重载清栈 / 去炼丹、商人、道具清栈 / 去幂等 / 去身份校验）均被杀。
Evidence: A 级——生产路径宿主 RED → GREEN（复活后不在格槽、属性表被写坏、战斗中可撤、换层后仍可撤）；复核探针运行时实录（boss 掉落复制、重排后误写）；
全部变异被杀；全量测试 0 失败；gates trace PASS。Confidence: 85

## BUG-046 "你无法战胜它"提示框未移植：打不过的怪撞上去 Rust 静默挡住，原版弹 kind 0 提示（classic 移植缺口，2026-10-29 spec 对抗复核发现，已修复）

> 【2026-10-29】双操控方案 spec 对抗复核 C3。
- **现象**：手动撞上打不过的怪，原版清 `walkStepCount` 并弹提示框"你无法战胜它"；Rust 只清步数，不弹框。
- **根因**（classic 移植缺口）：`a.java:5487-5490`（`walkStepCount = 0; showOverlayMessage(0, "你无法战胜它", 0, 0)`）；
  Rust `scene.rs:5426-5432` 只留注释 `// "你无法战胜它" 提示浮层：交互批次`，全仓无此字符串。
  gameplay.md R-battle-4、fixture-plan-boss FX-BOSS-68B 已写明此浮层，trace 门禁的已录场景未覆盖。
- **影响**：classic 与参考版行为不一致（少一个浮层，及其吞键 / 复位 wp 的后果——原版正是靠这个浮层让 dp3 首步撞怪时复位相位，见 BUG-044）。
- **Stage A 处理**：**须修**（Rust 少了原版行为，属保真偏离而非原版 bug）。按 BUG-037 体例移植并加 Java 对拍用例；修复后 classic 向 Java 收敛，
  trace 基线变化须在提交说明中写明。排在 STAGE-B-008 之前。
- **Stage B 修复方案**：—（移植后触屏"打不过"沿用原版提示框，不另加闪红）。
Evidence: A 级——`reference/src/deobf/a.java:5487-5490` 原文；`rg "无法战胜" crates` 零命中（本方 2026-10-29 复看）。Confidence: 95
- **修复**（2026-10-29，计划 1 T5）：不可胜分支补 `show_overlay_message(0, "你无法战胜它", 0, 0)`。Java 对拍：`fx-boss68b`（25 层 (6,7) 朝上、atk310 = 型 68 缩放防御，
  T300 北撞 → T360 关框 → T420 再撞 → T480 关框；A/B/C 三方 trace sha256 一致 `9b6b8051…1b69`，抽帧 560，帧链 `a954f21e…0c54`）。
  取证：Java T301 起画面中央出现带边框提示框（6 个字形 = 6 字文案；T301–T360 静止），T361 框消失；T421 再弹、T481 消失。
  Rust RED：首个不一致帧 T301（Rust = 无框帧）→ GREEN 560 帧逐帧一致。既有 classic 回放与 29 个帧夹具无变化（含 fx-boss68a）；gates trace PASS，
  visual 为既有 SKIP（比对器未就绪）。连带：`bug044_tap_walk_first_step_into_blocker_stays_put` 的"打不过的怪"子例改为先断言提示框、关框后再断言收尾
  （点地首步撞打不过的怪：弹框期间仍为 Dialog dp3、路段开着，关框后回 Map——与手动撞同一格同样弹框）。
- **连带 classic 路径**（计划 1 T5 复核 `target/verify/review_plan1_T5.md`）：
  - 道具 20 对角瞬移的镜像格是打不过的怪：activateItem case 20（a.java:7848-7859）先 interactWithCell ⇒ 拒战提示，不再弹"无法移动到该位置"。
    移植前 Rust 弹的是后者。新夹具 `fx-item20b`（floor1 (7,6)、atk=1、镜像格 (5,6) 型 43；三方 trace `4c8048ae…`，560 帧）：去修复行 ⇒ T381 红，修复后逐帧一致。
  - MOV 驱动的 dp3 玩家行走 / advanceScriptWalk 撞上打不过的怪：移植前不弹框、wp 停 1 走进怪格，移植后与 Java 一致（a.java:5353-5358 复位）。
    剧本路线是否真会经过打不过的怪未做全语料可达性判定，无夹具（B 级，遗留）。
  - 按住方向键持续撞同一只打不过的怪只弹一次（浮层分支块尾清 keyHeldCode，a.java:3345；复核临时 Java 场景三方一致、Rust 600 帧一致）。
- 根因行 `scene.rs:5426-5432` 为修复前行号（修复后不可胜分支在 5468 附近）。
Evidence: A 级——Java 三方一致参照帧 + Rust 逐帧比对 RED→GREEN（fx_boss68b_refuse_frames_match、fx_item20b_teleport_onto_unbeatable_enemy_frames_match）。Confidence: 95

## BUG-047 探步 `probe_step` 不写朝向：按旧朝向走出一整格且不做碰撞，可穿墙（点远处墙格 / Windows 点击 / 滑动；Stage B 触屏，2026-10-29 spec 第二轮复核实测，已修复）

> 【2026-10-29】双操控方案 spec 第二轮对抗复核 R2-1（探针 R2-C / R2-C2，记录见 `docs/reviews/2026-10-29-dual-control-schemes-spec-adversarial-review.md` §6）。
- **现象 / 复现**（宿主探针，1 层真资产）：
  - 玩家 (2,5)、`facing = 3`（左），`probe_step(0)`（向下）⇒ 返回 true，玩家**向左**走到 (1,5)；
  - 玩家 (1,5)、`facing` 指向地形不可走的 (1,6)，`probe_step(1)`（向上，上方为空格）⇒ 玩家走进 (1,6)（穿墙）。
  - 现网触发：点远处墙格时 `classify_tap` 按主轴发 `ProbeStep`（`touch.rs:156-167`）；Windows 点击、`Dispatcher::swipe` 同入口。
- **根因**（Stage B 移植错误）：`probe_step(dir)`（`scene.rs:1942-1948`）以 `dir` 调 `try_step` 判目标格，但不写 `self.facing`；
  放行后 Map wp1 臂按 `self.facing` 位移（`scene.rs:4880-4905`）。方向键路径先置 `self.facing = facing` 再 `try_step`（`scene.rs:5150-5152`）。
  炼丹炉判 `self.facing == 1`（`scene.rs:5330`），所以探步进炉也随旧朝向错乱。
- **Stage A 处理**：classic 不受影响（探步是 Stage B 新增入口）。
  > 【2026-10-29 更正】（计划 1 T1 复核 R3）准确说法：classic 的按键链、game-oracle 回放与 trace 不经过 `probe_step`；但两个壳的触屏 / 鼠标入口
  > 不按预设区分（Windows `main.rs:670` 的 `pointer_tap` / 鼠标拖动、Android `native.rs:374` 的 `scene_tap`），classic 预设下同样触发本缺陷，也同样拿到修复。
  > 另，复核差分（探步 vs 方向键，9 种格 × 4 旧朝向 × 4 方向）显示修复前还有：带钥匙撞黄门先扣钥匙再按旧朝向走进墙格、胜后补步按旧朝向走、
  > 撞墙 / 商人等不转向（存档 facing 随之不同）、debug 构建横探炉 panic。根因段行号为修复前行号。
- **Stage B 修复方案**：`probe_step` 在 `try_step` 前置 `self.facing = dir`（与 `handle_field_input` 同式）。测试：{旧朝向 × 4} × {探步方向 × 4}，
  断言终点 = 探步方向邻格或原地；炉 × {朝上探步、横向探步且旧朝向为上}。列为 STAGE-B-008 前置任务。
- **修复**（2026-10-29）：`probe_step` 在 `try_step` 前置 `self.facing = dir`。宿主钉：core `bug047_probe_step_moves_in_probe_direction_for_every_old_facing`
  （16 组合）、`bug047_sideways_probe_into_alchemy_with_old_facing_up_does_not_open_it`、`bug047_upward_probe_into_alchemy_with_old_facing_down_opens_it`（正向）；
  删去该行的变异三条全被杀。复核补钉：`bug047_probe_step_matches_direction_key_on_doors_and_enemies`（黄门带钥匙 / 可胜怪 × 4 旧朝向，期望取方向键路径）、
  `field_taps_refused_unless_field_idle` 加"拒绝不改朝向"断言（把写朝向挪到守卫之前的变异被杀）、16 组合测试断言返回值。
Evidence: A 级——复核探针运行时实录（探针已删）；修复前 `probe_step`（scene.rs:1942-1948）无 facing 写入（本方 2026-10-29 复看）；
修复后删去写朝向一行 ⇒ 4 条 bug047 测试 FAILED（本方实跑）。Confidence: 90

## BUG-048 点地撞怪击杀触发剧情后，寻路段被重开、撤销栈被补入战斗笔（Stage B 点地寻路 / 撤销，BUG-043 复核 R4，已修复）

> 【2026-10-29】BUG-043 对抗复核 R4 登记为另案；双操控 spec 复核 m9 列为前置。
- **现象**：点地寻路撞怪开战、胜利特判触发剧情（35 层型 69、49 层型 70/71、50 层型 72 → `load_level_script`）：剧情接管已收段并清栈，
  随后胜利尾无条件 `route_combat_finished` + `route.resume`（修复前 `scene.rs:5744`，b80ee3c）⇒ 剧情运行中寻路段被重开（截断剧情 MOV 行走的风险，BUG-025 同型），
  撤销栈被补入战斗笔（剧情后可撤销复活 boss）。
- **根因**：Stage B T2 挂点只看 `battle_undo_snap` 是否存在，不看路段是否已被剧情接管。
- **Stage A 处理**：classic 不受影响（classic 战斗不开路段）。
- **修复**（2026-10-29）：胜利尾三分支——
  ① 剧情接管（`load_level_script` 已收段并清栈 ⇒ 路段与战斗挂起都已解除）：丢快照、不入栈、不续走；
  ② 正常（路段仍挂起）：入战斗笔（击杀有不可逆生成则按 BUG-045 清栈）并续走；
  ③ 战斗中经壳层按键链 `interrupt_route` 打断（路段挂起解除、战斗仍挂起、未清栈）：同 ②。续走只开零步段给胜利 restep 记位移笔，
     否则撤销战斗后玩家与复活的怪同格（计划 1 T4 复核 R1）；计划原稿"只看 `is_suspended`"会让该情形 `combat_pending` 永久为真、撤销恒 Blocked。
  宿主钉（core 4 条）：`bug048_victory_script_takeover_keeps_route_closed_and_journal_empty`（70@49）、
  `bug048_script_takeover_with_spawn_keeps_route_closed_and_journal_empty`（71@49，击杀有生成）、`bug048_plain_victory_still_pushes_combat_and_resumes`、
  `bug048_key_interrupt_mid_tap_battle_still_records_combat`（撤销顺序：先退回邻格再复活）。
  变异：分支条件改为"有快照即入栈续走"（≈修复前）⇒ 两条剧情接管钉红；按键打断不续走 ⇒ 按键钉红。
Evidence: A 级——宿主 RED（49 层型 70 击杀触发剧情后 route.is_open() 为真）→ GREEN；上述变异被杀；复核 `target/verify/review_plan1_T4.md` 探针
（69@35 / 71@49 / 72@50 剧情接管终态、按键打断重叠回退）；gates trace 实跑见提交说明。Confidence: 85

## BUG-049 Dialog 模式（mode 11）下弹出的提示框不画：画面无提示却吞键（classic 移植缺口，2026-10-30 计划 1 实机冒烟发现，已修复）

> 【2026-10-30】计划 1 Task 7 实机冒烟（Xiaomi 14 `c362a15b`）：25 层（fx-boss68b 预置）点打不过的型 68 ⇒ 玩家不动、画面无任何提示，
> 此后方向键无效（日志 `scene keys=[Press(-1)] mode=Dialog`）；按确定后才回 Map。同一只怪用方向键手动撞（Map 模式）提示框正常显示。
- **根因**（classic 移植缺口）：Java `paint` 的提示浮层在 mode switch **之后**的公共尾绘制（a.java:3129-3168：`!floorTransitioning && overlayActive`），
  switch 各 case 均以 break 结束、无提前 return（a.java:2385-3127 区间内无 `return`）⇒ 所有模式都画。Rust 只在 Map 尾与 SaveSlot / InGameMenu /
  Shop / ItemPanel / Help / Alchemy / RecordBrowser 分支调 `paint_overlay_box`，Dialog 分支（`scene.rs` paint 的 `GameMode::Dialog` 臂）漏画。
  BUG-046 移植后，点地寻路（借 dp3、处于 Dialog）首步撞打不过的怪会在 Dialog 模式弹拒战框，缺口由此在实机暴露；classic 下脚本行走（MOV dp3）
  撞上弹框的阻挡同样受影响。
- **Stage A 处理**：**须修**（Rust 少了原版行为）。Dialog 臂在 `paint_dialog` 之后补 `paint_overlay_box`（函数内自带 `!transitioning && overlay_active` 门）。
  其余未调 `paint_overlay_box` 的模式（About / Ending / EnemyInfo 等）在这些模式下能否激活浮层未考证（C 级），未改。
- **Stage B 修复方案**：—。
- **修复**（2026-10-30）：宿主钉 core `bug049_overlay_is_painted_in_dialog_mode`（Dialog 模式下提示框区域须被绘制；删去补画一行即红）。
  实机复测：同一场景点打不过的怪 ⇒ "你无法战胜它"框可见，确定关框后回 Map（`scene keys … mode=Map`）。全量测试 0 失败；gates trace PASS。
Evidence: A 级——a.java:3129-3168 公共尾 + switch 区间无 return；实机日志与截图（target/verify/device_plan1/b4_padbump.jpg 无框、b9_tap_refuse.jpg 有框）；
宿主 RED→GREEN。Confidence: 90
