# P3 渲染层移植规格（paint 调用树 + 像素模型）

> 状态：进行中。对拍裁判 = oracle trace `FRAME sha=`（TickHooks.afterPaint：
> 240×320 ARGB 逐像素大端 4 字节 SHA-256，截断 32 hex）。
> 本文是渲染层的**设计合同 + 对抗 review 台账**（2026-10-09 首轮）。

## 1. 分层与已完成范围

| 层 | 位置 | 对拍 | 状态 |
|---|---|---|---|
| SHA-256 | game-platform/hash.rs | FIPS 向量 | ✅ |
| zlib/deflate + PNG→ARGB | game-data/{inflate,png}.rs | 164/164 真实资源图 | ✅ |
| Graphics/Font shim 复刻 | game-core/render.rs | L1b golden 16 用例 | ✅ |
| logo 动画子系统 | game-core/logo_anim.rs | T40 双对拍 + T1-67 回放 | ✅ |
| paint 分发 + BootMachine | game-core/paint.rs | boot_replay T1-70 | mode 0/21 ✅ |
| 视口常量契约 | game-core/layout.rs | layout 推导测试 | boot 路径 ✅ |

**时序合同（差分实测）**：每 tick **logic 先 paint 后**——TICK n 的帧 =
第 n 次逻辑推进后的 paint（`run():3230 循环体[logic] → :4254-4255
repaint;serviceRepaints → Canvas:71-94 paint→afterPaint→preTick(tick++)→postTick`）。
按键投递：TICK n 的 INPUT 行在 preTick(n) 投递（paint#n 之后），由
logic#(n+1) 消费（延迟一拍；trace T70 press(-6) → T71 帧切换实证）。

## 2. 对抗 review 首轮（2026-10-09，非实现者独立复核）

结论：**PASS-with-risks**——零对拍破坏缺陷。已修复：

- D-1 `in_arc` 负弧递归缺 `% 360`（render.rs，对照 Graphics.java:322）——死代码
  路径（游戏唯一弧调用 fillArc(…,0,360) a.java:6663 短路），已补齐对齐 shim。
- D-2 `drawSubstring` 负 len：shim 空循环静默，Rust 原 panic——已按 shim 语义钳制
  （调用方 a.java:5048 的 len 是差值可能为负）。
- R-5 mode 0→21 清理分支补全：logoLayoutDone 复位（reset_layout_flag）+ f_int_02=75
  帧间隔字段化。
- R-8 layout.rs 头注收窄：仅覆盖 boot 渲染路径；camera 的 view_h=252、walk 的
  边缘阈值 106 是**独立游戏常量**（viewHeightPx 等），扩视口时不可盲目替换。

登记在案、后续批次处理：

- R-1 回放画布每 tick 新建，依赖"每帧全屏覆盖"不变量（boot 三分支成立；
  非全覆盖分支端口时需改持久画布）。
- R-2 同 tick 多键：回放取首个 press，Java keyPressed 覆盖式取末个
  （现 trace 每 tick 至多一键）。
- R-3 `LogoAnim::tick` 只实现 runLogoAnimation 的 var1==0 分支；mode 22 走 runLogoAnimation(1,…)
  是静默缺口（端口 mode 22 时必须先补 var1 维度）。
- R-6 OPS 流：Java 有 256 条截断 + opCount 头；Rust 全量无头。加 OPS 门禁前对齐。
- R-7 `size_code` 由行高反推 setFont op 文本，仅对游戏唯一字体 getFont(0,0,8)
  恒正确。

## 3. golden 覆盖缺口清单（后续补强输入）

1. drawArc 全方法（0.72–1.28 环带 + inArc 边界 `rel==arcDeg`）——无 golden、
   无游戏调用方。
2. fillArc/drawArc 部分弧：arcDeg<360（含负弧递归）、normStart<0、rx/ry≤0 臂。
3. PNG 防御分支：color-type 0/2/4/6 全部（tRNS 单色/真彩 key、灰度+alpha）——
   164 张语料全为 (depth,3,0)；filter 1/3/4 在 bpp>1 下零覆盖。
4. inflate 错误臂：cl16@i==0、LEN/NLEN 不互反、btype=3、过订阅表。
5. fillTriangle 屏外/负坐标顶点（ceil/floor 负半区）。
6. setClip 全屏外（clipW=0）与 clipRect/translate 组合。
7. Font.isWide 六区间端点；drawImageTransformed swap 变换(4-7)+非 0 anchor。
8. Canvas 时序：needRepaint=false 的 tick、同 tick 多事件投递。
9. createImage(String) 双路径资源名解析（Rust 走文件系统直读）。

## 3.5 mode 3 游戏画面勘察（2026-10-09，第七批后）

- **ops 截断上限 256**（Graphics shim `OPS.size() < 256`，a.java 侧同）：
  gameplay-floor1 的 mode 3 帧 256 ops = **截断值**——瓦片层吃满窗口，
  HUD/实体/软键的 ops 不可见，但 **FRAME sha 含全部绘制**（对拍不降级）。
- mode 3 画面构成（T544-3000 恒定）：parallax(true)（base_y = f_int_48-320+6，
  f_int_48=270 ⇒ -44）+ paintTileLayer 瓦片层（map 容器 [2][0] 128x208 tileset，
  16px 切片 + mapTransformGrid 变换）+ paintEntityLayer 实体层 + 小地图（optionChecked[1]
  =true（RMS catch）+ f_Image_03（buildMinimap 生成）+ 玩家点）+ paintHudPanel HUD + paintStatusBar
  状态栏 + 楼梯指示 + drawPopupLayer + 软键 (1,3)。
- **paintTileLayer 瓦片层**（a.java:6979-7035）：起点 x=cameraPixelX+(tileColStart<<4)、
  y=cameraPixelY+20+(tileRowStart<<4)；列窗 tileColStart..tileColEnd、行窗 tileRowStart..tileRowEnd；
  下标步进 mapCellsWide<<1（**双宽 stride**——terrain/transform 平行数组，
  装载链待考证 loadFloorData）；瓦片值 v：sx=(v&7)<<4、sy=(v>>3)<<4。
- **paintPlayerSprite 玩家**（a.java:5831-5893）：sptprop[5][0] 阴影 + actor[3][0]
  （123x138 = 3 行×41x46 帧，行=facingDirection、列=f_int_arr_03[f_int_38]）
  + walkPhase==2 的四向残影。
- **存档格式**：MOT_L{n}（save.rs parse_floor_save 全字段序 + python struct
  复算 golden）；SKY_WAR（optionChecked×4 + …）。
- **paintEntityLayer 实体渲染层**（a.java:6481-6713，2026-10-09 考证）：
  - 筛选：活跃且未移除；屏裁 `x∈[-w,viewWidthPx] && y∈[-12,20+viewHeightPx]`；
    首个玩家南侧实体之前插画玩家 paintPlayerSprite（单次，f_bool_arr_02[type] 除外）
  - 帧偏移 `= 精灵宽 × f_int_arr2_02[动画表idx][帧idx]`（二维动画偏移表）
  - 渲染类别 `f_byte_arr_03[type]`（initEntityTables a.java:6257-6284：1..12→1、
    13..32→2、33..40→4、41..78→8，特例 76/81/82/83→1、77/78→16、
    79/80→4、72/84/87→32、85/86→2）
  - switch 六类：1=条带精灵（类型 6/9 浮沉/右移特例）；2=条带/门贴图
    （27x29 上移动画）/阴影浮沉；4=阴影+浮沉条带；8=阴影+直立（67/69
    拼装表多片组合：f_byte_arr2_01 行×f_byte_arr2_00 元数据）；16=阴影
    浮沉/门贴图；32=多形态（f_byte_19 倍率 + f_int_127 高亮）
  - walkPhase==2 && f_bool_08：交互浮标（iconStripDx/Dy[frameCounter&7]
    波动 + drawDigitStrip 数值）
- **paintPlayerSprite 玩家**（a.java:5829-5891）：sptprop[5][0] 阴影 + actor[3][0]
  三向精灵（行=facingDirection、列=帧表 f_int_arr_03[f_int_38]×41px 的
  41x46 帧）；walkPhase==2 四向残影。
- **floor1 实体清单**（2026-10-09 盘点，sprite1.bin 权威解析）：35 实体，
  类型分布 {1:7, 7:1, 15:1, 26:5, 29:1, 30:1, 31:4, 32:1, 41:4, 42:1,
  43:3, 44:1, 45:1, 46:1, 83:3}——渲染类别仅 **3 类**：条带×11、门/阴影
  ×13、直立×11（无 67/69 拼装与 16/32 多形态）。
- **动画偏移表 f_int_arr2_02**（a.java:485-495 构造常量 10 行：
  {0},{0,1,0,2},{0,1,2},{0,1,2,1},{0,1,2,2,1,0},{0,1,2,3,2,1},
  {0,1,2,3,4},{3,4,5,6},{4,3,2,1,0},{2,1,0}）：帧偏移 =
  `精灵宽 × 表[动画表idx][帧idx]`；动画表 idx 由 f_byte_arr_06[type] 初始化
  （initEntityTables a.java:6315），帧推进在 advanceEntityFrames（每 tick 换帧——a.java:6819）。
- **removeGroupAtCellByType**（a.java:6421-6437）：组移除（f_byte_arr2_02 行×f_byte_arr2_03
  组成员，按类型筛除并置 removed）。
- **剩余件**（下批）：popup 环形队列 + walkPhase 1/5（步进/场上战斗/胜利
  跳格）+ m_104 交互 → gameplay-floor1 T1500+ 移动段全帧对拍；mode 11
  对话框；mode 4/19 暂停菜单；paintEntityLayer case 16/32 与 67/69 拼装（floor1 无）。

## 3.6 mode 3 场景端口与 T537-599 全帧对拍（2026-10-09，第八批）

**结论：gameplay-floor1 T537-599（63 帧）逐 tick FRAME sha 全对拍 PASS**
（`crates/game-oracle/tests/gameplay_floor1.rs`）——画面构成 100% 覆盖：
parallax + 瓦片层 + paintEntityLayer 实体层（含 paintPlayerSprite 玩家插入）+ buildMinimap 小地图 +
paintHudPanel HUD + paintStatusBar 状态栏 + 空弹层 + 软键。T544 ops 前 256 条亦逐条一致
（parallax 5 + 瓦片 250 + 实体层首条）。

本批新 A 级发现（全部有运行时 dump/像素复算证据）：

1. **mapTerrainGrid 双索引**：rebuildWalkability（a.java:6954-6972）用
   行距 wide·4、列距 2（var1 行尾 +=wide<<1 叠加列循环 ×2）；paintTileLayer
   用行距 wide·2、列距 1——同一数组的两套读法原版共存。
   证据：oracle FLD 134（f_bool_arr2_00）13×13 逐格复算匹配。
2. **sortEntitiesByY 槽修复是完整双向**：① var2 旧格 var2+1→var5+1（a.java:6752-6762）
   ② var2 新格（=var5 旧位）var5+1→var2+1（a.java:6776-6790）。
   排序后槽表与 FD 137（f_byte_arr2_03）128×16 逐格一致。
3. **mode 3 起点 = T537**（dense dumpStride=1 复跑实证：T536=mode2、
   T537=mode3）；T537-543 的 trace 无 ops 行只是 dumpStride=8 采样未打印，
   帧仍在演变（PNG 像素 diff 证实逐拍重绘）。
4. **advanceEntityFrames 帧时序**：T537 的 run 是 case 2（切换发生在 else 分支内）⇒ 本拍
   不推进；T538 起 case 3 以 frameCounter=tick-1 跑 advanceEntityFrames，**偶数拍推进**。
   T544 运行时锚（FLD 103）：type44（行 4）帧=4、type45（行 2）帧=1
   （0→1→2→0→1 一次回绕）。
5. **bob 相位链**：advanceBobPhase 初值 (0,false)（字段默认，无构造器初始化）；
   mode 1 T72-161 的 90 次调用 → 进 mode 3 时 (-2,true)；mode 8/2 不调
   advanceBobPhase；paintEntityLayer 每次 paint 首行调用（T537 首绘 → -1）。
6. **backdropScroll 相位链**：mode 8 parallax 每拍 -1（T500 绘后 -30）；
   mode 2 无 parallax；mode 3 首绘 T537 → -31，T544 → -38（trace 首列
   drawImage x 实证）。
7. **paintHudPanel 槽位**：武器框 x=159、甲框 x=193（var11 = 83+16(=99) 后 +60、
   +34——不是 83+60）。
8. **HUD 数值源**：MOT_L0 preset（HP500/ATK30/**DEF30**/黄钥1/金100/
   cell(3,10)）；小地图开关 = SKY_WAR 第 2 字节（=1 开）。
9. **entityGold 修正**：f_byte_arr_07[45]=23（a.java 构造字面量 + 运行时
   FLD 101 一致）；精灵哈希（FLD 019 像素级 sha）与 Rust 解码逐张一致。
10. **interactWithCell cat 8 语义**（walk 前史误读修正）：踩怪格不放行
   （var6=false）但触发场上战斗（walkPhase=5）；胜利后玩家跳入怪格
   （fixture FLD 083/084：T1248=(96,320) → T1256=(96,352)）。walk-pure
   场景注释"向上走廊可走"是错误网格时代的产物——权威网格 (3,9) 不可走，
   三次 -1 被阻与 Java 一致。

已入库模块：`crates/game-core/src/scene.rs`（GameScene：load_floor 装载链 +
build_minimap + paint 全链 + 稳态 tick）、`entity.rs` 扩展（视觉字段
sprite_w/h/anim_idx/frame、render_category、ANIM_TABLE_IDX/ANIM_OFFSET_TABLE、
dispatchSpriteSize/dispatchAnimFields/markEntityDesRemoved/markEntityRemoved/sortEntitiesByY/advanceEntityFrames/advanceBobPhase、pullEntityFromCell detach）。

## 3.7 移动段全帧对拍（2026-10-09，第九批）

**结论：gameplay-floor1 T537-1700（1164 帧）逐 tick FRAME sha 全对拍 PASS**
（含 mode 3 稳态 964 帧 + 一场完整战斗链 ~120 帧 + 战后收敛 ~80 帧）——
walkPhase 0/1/2/5 状态机、场上战斗、胜利跳格、popup 环形队列全部逐帧一致。

本批新 A 级发现（全帧对拍 + 全量 ops 诊断实证）：

1. **keyValue 是边沿触发**：run 主循环尾部 `keyValue = 0`（a.java:4266，
   else 分支；overlay 分支清双键 a.java:3339）——每拍清零，与 keyReleased
   无关。boot replay 的"粘性"注释是误读（按键恰好单拍消费故未暴露）。
2. **-5 双段语义**：第一次 -5 → 残影填充 + walkPhase=2 + battleDigitByType
   预计算（f_bool_08 = 有 13 号道具才画数字条）；第二次 -5 → f_bool_06 +
   m_104(0)（缓动相机锁回玩家）。T600/700 双 -5 实证 110 拍交互视点。
3. **m_026 残影蛇缓动**：目标 = 未锁定时视口中心偏移（**bob 耦合**）/
   锁定后玩家位；[0]=目标直赋，[1..3] 半距+var3 蛇形追尾；[3] 到位且
   锁定 → walkPhase=0。
4. **m_104/stepCameraTowardTarget 坐标是反演关系**：ease 坐标 = 视口中心
   −cameraPixel；回写 setCameraClamped((中心)−ease)。
5. **星光用裸 cameraPixelY**（a.java:6683 读字段而非 m_053 入参——入参带
   +view_top 偏移，星光 y 少 20px）。
6. **drawDigitStrip 尾格恒画**（a.java:9800-9803：负值标志 var5 死变量，
   第 11 格装饰字形无条件绘制——T1506 全量 ops + popupValue=[29,29,1]
   实证"1"后面跟装饰格）。
7. **战斗节奏**：交换拍 = f_int_151&3==0（起手第 1 拍即交换）；胜利 →
   f_bool_23 死亡动画 6 拍（markEntityRemoved 首拍置 state=1 → 27×29
   row6 播放）→ tryStep(facing) 胜利跳格 → battleTargetEntity=-1。
8. **RNG 消费链**（gameRandom 单例 seed=0）：mode 1 粒子 23×2 次
   （T72-161 每 4 拍 randomBelow(240)+randomBelow(150)）→ 战斗抖动
   每拍 2×randomBelow(5)（paintEntityLayer state==3 且过屏裁后）。
   randomBelow = (nextInt()>>>1)%n（a.java:10177，非 JDK bound 算法）。
9. **popup 环形队列**：30 槽 write 回绕；kind 5/6/7 用 ui[8][9]+六张
   散射表（f_byte_arr_32-37，计数 0..=8）；kind 1/2/3/4 用 map[2][6]
   图标条 / map[2][3/4/5] 数字带 + y 上升 4px/拍；>7 拍退役。
10. **玩家插入阻断表 f_bool_arr_02**：构造字面量 {T,F,F,F,F,T,T,T,T,F,T,F,F}
    （a.java:679）经 initEntityTables 拷贝——类型 0/5/6/7/8/10 占据插入
    窗口时不插画玩家（a.java:6510）。

诊断方法学：shim ops 上限 256 → 临时 2048 重编拿全量 ops 对拍（事后还原 +
reference 门禁重验 A==B==C 等价 PASS）。

## 3.8 换层全链对拍（2026-10-10，第十一批）

**结论：新场景 floor1-to-floor2（T537-1500，964 帧）逐 tick FRAME sha 全对拍
PASS**——行走（4 场战斗）→ 踩型 7 楼梯（11,1）→ changeFloor 遮幅（闭合
1..4 → 中段换层 → 开启 4..1）→ floor 2 落地（findFloorGateEntity 型 8
(1,1) → m_031 邻格 (1,2) → 居中相机）。场景已入 gates reference 套件
（A==B==C 9/9 PASS）。

本批新 A 级发现：

1. **changeFloor（a.java:7116-7160）**：strict 路径（键 49/55）查
   min/maxFloorReached（miscTexts 0/1 提示）；层界 0..55（f_int_68，
   a.java:808）；置 f_bool_16 + 记录 f_byte_23（目标层）/f_bool_18（下楼标记，
   型 7 上楼传 false）；m_119 存离层状态（RMS，Rust 不追踪）。
2. **遮幅是 paint 侧状态机**（a.java:3171-3215，公共尾）：f_bool_17 构造
   true=闭合相；++wipe>4 → 钳 4、翻相、**同 paint 中段换层**（loadFloorData
   + 落点 + buildMinimap + applyStepCellEffects）；开启相 --wipe≤0 → 收。
   黑格 40×30 网格 8px 步进、尺寸 wipe<<1、偏移 4-wipe、setColor(0)
   （不透明黑，shim opaqueColor |0xFF000000）。
3. **换层落点直传 f_bool_18**：findFloorGateEntity(true/false) 找型 7/型 8
   （param>>9==0）——上楼抵达（f_bool_18=false）落机型 8 下楼梯；
   m_031 落点 = 首个可行走邻格（上/下/左/右序，a.java:5797-5810）。
   特例：floor 0→m_024(1,2)、floor 50→(6,7)、floor 1 上行→(6,11)。
4. **m_054 槽修复的原版残留进入交互路径**：floor 1 排序后 (2,9) 的格槽
   含黄门 id（FLD 137 逐格一致）——上行踩 (2,9) 触发**开门**（耗黄钥匙
   spawnPopup kind1 图标 + m_047 cat1 重建小地图）。对拍实证原版槽表
   的"错位"是可交互语义的一部分。
5. **markEntityRemoved 的字节码共尾**（javap offset 188）：buildMinimap
   无条件执行于**所有类别**（Vineflower 反编译折叠进 case 1 是投影假象；
   cat 8/16/32 从 225 起不走 188）。此前"cat 1 才重建"的读法作废。
6. **小地图玩家点用 playerCellX/Y**（a.java:2397-2398）而非像素 >>5——
   上/左行步进中格坐标未更新而像素已进目标格（gameplay-floor1 窗口的
   下行恰好末拍翻转，掩盖至本批）。
7. **型 72（cat 32）渲染臂**（a.java:6645-6657）：非追踪实体静态贴
   （无 bob、y-(h-32)、src frameOff/0）；追踪实体（=f_int_127，脚本路线
   对象）按形态号 f_byte_19 取帧行（==3 时镜像）。

诊断方法学（二次验证）：FLD 134 早期解析曾有嵌套数组错位（错误地显示
(7,7) 不可走）——以 fixture 复算 + sha 双 trace 互证裁决；shim ops 上限
4096 通道二度用于换层拍全量 ops 对齐（用后还原 + reference 重验）。

## 3.9 floor2 巡回：宝箱/装备/浮层/组开门（2026-10-10，第十二批）

**结论：新场景 floor2-tour（T537-1356，820 帧）逐 tick FRAME sha 全对拍 PASS**
——型 11 道具浮层、血瓶拾取、蓝门"无钥匙"浮层、翻页钳制、宝箱组开门
（walkPhase 4）、换层往返、keyHeld 连走。场景入 gates 套件（A==B==C 11/11）。

本批新 A 级发现：

1. **overlay 冻结 run**（label510 a.java:3246）：overlayActive 时 run 走
   label510 分支——handleOverlayKey 消费 keyValue（±2 翻页带钳制
   flipOverlayPage a.java:5119-5136；其余键关闭）、**walkPhase 状态机与
   m_055 全部冻结**、块尾 **keyValue = keyHeldCode 双清**（a.java:3339——
   held 残留会让关层后首拍误触发 tryStep，floor2-tour T1238 实证）。
2. **浮层弹回周期**：按住方向键顶门时 press→弹、下个 press→关（label510
   消费），8 拍 press 间隔 → 8 拍弹/关交替（T1229-1300 实证）。
3. **宝箱链**（type5 → tryOpenChest a.java:7275-7295）：同 param 型 4 收集
   → walkPhase=4 → paintTileLayer 头段动画（f_int_55：18 拍置 removed 动画、
   24 拍 removeGroupAtCellByType(4) 收尾）+ f_int_54 ±2 瓦片抖动——**非
   f16 遮幅**。型 76 组门：param&0xff+1 组号、全图清组内 76 槽。
4. **装备**（tryEquip a.java:7658-7676）：武器档 1..5（types 33/34/35/79/36）、
   甲档 7..11（37/38/39/80/40）；降级拒绝（!force 且当前档>=新档）；属性按
   equipTierBonuses {0,10,30,70,120,220}×2 增减；equipDescriptions 弹层。
5. **拾取浮层**：addItemToItemStack（a.java:7692-7710）叠加次数
   itemUseCounts、showOverlayMessage(kind1) 带**图标+名**（f_byte_07 +
   entityTypeImage/objectTypeNames）；装备成功 kind1 装备描述、降级
   "你拥有更强力装备"（kind0）。
6. **层状态持久化**（m_119/m_122 存档分支，a.java:9360-9384/9455+）：
   **(9,1) "无战斗"差异的根源**——type41 在 floor1 巡回期已死（R×22 战斗），
   changeFloor 的 **m_119 只序列化存活实体**（removed/state==1 跳过）到
   f_byte_arr2_05[floor]（u16 计数 BE + 每实体 [type u8, x u16 BE, y u16 BE,
   visible u8, param u16 BE]，m_120 写），m_122 **存档分支优先**——死怪不
   复活、已开门/宝箱保持。floor2 重入特例：scriptLineFlags[26]==false 且
   (11,4) 无型 84 → spawnEntity(84, 352, 128, 0)（9635-9640）。m_122 尾：
   renderCategoryByType[72] floor50=8、其余 32。恢复实体 removed=false、
   visible 按存档字节。全窗 T537-1500（964 帧）对拍 PASS（含二次换层 +
   floor2 重入 + 死怪不复活的 L 步进 (9,1)）。

## 3.10 mode 11 对话勘察（2026-10-11，第十三批，待实现）

**L2 基础已备**：script.rs 的 ScriptEngine（P3.1）已建模 dialog_char_pos
（=f_int_33）、scene_delay（=f_int_118）、speaker（=f_byte_17）、
speaker_entity（=f_int_119）、dialog_page（=f_int_112）与 DSL 解释
（TAK/MOV/GUT/DES/SWD/LAY/ROS/CES/RES/ADD）。剩余为**帧时序接线**：

1. **触发链**：interactWithCell cat1 case83 → tryRunScene(param, true)
   （a.java:8055：scriptLineFlags 门控逐 case）→ loadLevelScript
   （a.java:8025-8048：gameMode=11、lockCameraOn(0)、dialogPhase=0、
   f_String_05=levelScriptLines[idx]、scriptCursor=0）。
2. **run case 11 相位机**（a.java:4001-4064）：m_055 + stepCameraTowardTarget
   常跑；phase0 = f_int_118 延迟后 executeScriptInstruction(scriptCursor)；
   phase1/6 = 打字机（frameCounter&1 每双拍 +1 字、\\c +8/\\r +2 转义跳宽、
   keyValue!=0 → layoutFullText 全量 + skipTextToLineEnd a.java:5125）；
   文本走完后 ±2 翻页 / 其余键 openDialogPage(page+1)（phase6 尾 m_103）；
   phase2 = m_106（实体路径缓动）；phase3 = scriptWalk（case 2 分支 4066-4120）。
3. **paint case 11**（a.java:2798-2896）：parallax+瓦片+实体+弹层+状态栏 +
   m_039 对话框（0,320-框高,80,框高）+ 右缘补条 + **说话者立绘**
   （f_byte_17>0 → entityTypeImage 立绘贴地；=0 → actor_face 头像；<0 → "?"）+
   paintWrappedText(85, 120+12, 129, 框高-12, **false 无三角**) + 翻页箭头
   ui[8][15] + **对话气泡**（f_byte_arr_22[f_int_112] ≥0 时 face 容器贴实体/玩家
   头顶，a.java:2871-2885）+ phase6 目标箭头 map[2][11]。
4. **数据**：dialogueTexts 269 条（夹具 dialogue_texts.txt ✓）、
   dialogueSpeakerType 269（speaker_types.txt ✓）、f_byte_arr_22 300 条
   （**无夹具**——需从 a.java:1465 提取）、levelScriptLines 68 行
   （level_scripts.txt ✓）。
5. **对拍目标**：现成 A-prologue-dialog trace（9800 ticks，A==B==C 已 PASS）——
   序章 = floor51 + script 0 全链（CES/MOV/TAK/ROS/GUT）。需 scene 支持
   floor51 资产（maplv51/sprite51 夹具已有）与序章 preset（无存档冷启动）。

**实现进度（93706ec + 本批）**：触发链/门控/DSL 子集（TAK/SEE/END/SMS）/
相位机/paint case 11 全部落库（旧对拍不破）。prologue 对拍 T584 三带差异
待调：(a) 状态栏"引子" m_142 outline 已实现但字形位置偏上（疑 BASELINE=17
的 draw_string 字形 y 范围语义，SMALL 字体 baseline=8/h=10）；(b) 实体动画
帧差；(c) 对话框文本内容差。相机已一致 (64,-228)；maplv51=12×13。

剩余（下批）：prologue T584 三带调试 + mode 4/19 暂停菜单 + cat 67/69 拼装臂。

剩余（下批）：changeFloor 楼梯换层（walkPhase 4 + 动画）、宝箱 m_073、
装备拾取 33-40/79/80、cat 16/32/67/69 渲染臂、mode 11 对话、mode 4/19。

## 3.11 序章全窗对拍收官（2026-10-11，第十六批）

prologue_dialog_frames_match **T577-9800 全窗 9224 帧 PASS**（摘除 ignore）。
本批收口的四类机制（全部 A 级证据，op 流逐条对拍定位）：

1. **f_bool_26 路径箭头子系统**（N-0403）：
   - SEE flag=1 → findPath(player→dialogMove) 成功 ⇒ pathArrowsVisible=true +
     walkPhase=0，**落穿 default** 相机 ease（a.java:8493-8501，case1 无 break）；
   - paintPlayerSprite 尾（setClip 全屏复位后）沿 walkPathBuffer 自顶（玩家侧
     第一步）外扩绘制 [8][22] 三联贴片 21×22：0 下(y+bob)/1 上(y+bob)/
     2 右(edge patch 镜像, x+bob)/3 左(x+bob)，setColor(136) 每步重设
     （a.java:5925-5952）；
   - 消费：applyStepCellEffects 尾 label216（a.java:5744-5757）顶步==朝向 →
     消费一步，否则整条清空+熄箭头；tryStep 撞格时 overlayActive&&walkPhase==1
     亦清（a.java:5349-5354）；
   - advanceScriptWalk（a.java:5818-5834）+ handleFieldInput 的 mode11 委托
     （a.java:5263-5265）——MOV_0 对白自动行走的机制位，序章 script 40 未触达。
2. **bob 跨模式状态**：animBobOffset 8 周期 [0,1,2,1,0,-1,-2,-1]；mode 11
   **无** case12 落穿（dialogPhase 各分支 break 外层 label534，a.java:4131
   不可达），推进仅来自 paintEntityLayer 头（a.java:6495）每 paint 一次；
   run case1（标题）每 tick 推进（a.java:3387）——BootMachine 已接同机制，
   场景初值 (2,false) 由 trace FLD 120=T120 dump 实测锚定（T576=2、T902=0）。
3. **paintWrappedText 忠实重写**（a.java:5040-5117）：整行按 \c 色段
   drawSubstring flush（原逐字 drawChar 会把转义字面画出）；跨页起始色继承、
   \r 恢复 overlaySavedColor、非法 hex 从反斜杠重画、var15 每行清 0 的
   `!= -1` 怪癖。
4. **夹具转义污染修复**：dialogue_texts.txt 旧提取把 trace 转义层 `\\c`
   当内容（19 条含 \c 转义的对白全部多一反斜杠）→ 从 a.java:1909-1930
   dialogueTexts 初始化器重新机械提取（行安全格式：\\/\n/\r 字面转义），
   三处消费端统一 unescape_text_fixture。同源 \x1e 分隔符残留不复存在。

### mode 10 道具面板（-6 进入）

run case3 头 keyValue 开关（a.java:3552-3562）-6 → gameMode=10 + m_000
case10 布局（a.java:4366-4377：4 列/8 行总/6 行可见/框 202×296）；run case10
键位（a.java:3931-4005：-7 返回、-6/-5 使用、-4/-3 行内环绕、-2/-1 翻行滚窗）；
paint case10（a.java:2724-2792：斜线背景（颜色驻留恰为 #000000）+ 框 + 4×N
格阵 + [2][10] 38×38 双帧选中框 + 翻页箭头 + 软键栏）。字段 N-0409..N-0417。
-7 在 mode3 → mode4（存档菜单）**未移植**——mode 4/19 批次待办。

## 4. 证据基线

- 像素模型：`reference/shim/src/javax/microedition/lcdui/{Graphics,Font,Image,Canvas}.java`
- 游戏侧：paint（a.java:2333-3221）、logo（10187-10553）、mode21
  （3047-3054/4204-4223/4425-4460）、paintSoftkeyBar（5970-5980）
- golden 裁判：`data/golden/render-golden.json`（Java 微驱动
  reference/oracle/src/oracle/host/RenderGolden.java 可再生）
- 回放：`crates/game-oracle/tests/boot_replay.rs`（A-boot-menu TICK 1-70）
