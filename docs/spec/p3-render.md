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
  路径（游戏唯一弧调用 fillArc(…,0,360) a.java:6650 短路），已补齐对齐 shim。
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
  16px 切片 + mapTransformGrid 变换）+ m_053 实体层 + 小地图（optionChecked[1]
  =true（RMS catch）+ f_Image_03（m_057 生成）+ 玩家点）+ m_037 HUD + m_035
  状态栏 + 楼梯指示 + drawPopupLayer + 软键 (1,3)。
- **paintTileLayer 瓦片层**（a.java:6966-7022）：起点 x=cameraPixelX+(tileColStart<<4)、
  y=cameraPixelY+20+(tileRowStart<<4)；列窗 tileColStart..tileColEnd、行窗 tileRowStart..tileRowEnd；
  下标步进 mapCellsWide<<1（**双宽 stride**——terrain/transform 平行数组，
  装载链待考证 loadFloorData）；瓦片值 v：sx=(v&7)<<4、sy=(v>>3)<<4。
- **m_033 玩家**（a.java:5831-5887）：sptprop[5][0] 阴影 + actor[3][0]
  （123x138 = 3 行×41x46 帧，行=facingDirection、列=f_int_arr_03[f_int_38]）
  + walkPhase==2 的四向残影。
- **存档格式**：MOT_L{n}（save.rs parse_floor_save 全字段序 + python struct
  复算 golden）；SKY_WAR（optionChecked×4 + …）。
- **m_053 实体渲染层**（a.java:6468-6700，2026-10-09 考证）：
  - 筛选：活跃且未移除；屏裁 `x∈[-w,viewWidthPx] && y∈[-12,20+viewHeightPx]`；
    首个玩家南侧实体之前插画玩家 m_033（单次，f_bool_arr_02[type] 除外）
  - 帧偏移 `= 精灵宽 × f_int_arr2_02[动画表idx][帧idx]`（二维动画偏移表）
  - 渲染类别 `f_byte_arr_03[type]`（m_043 a.java:6244-6271：1..12→1、
    13..32→2、33..40→4、41..78→8，特例 76/81/82/83→1、77/78→16、
    79/80→4、72/84/87→32、85/86→2）
  - switch 六类：1=条带精灵（类型 6/9 浮沉/右移特例）；2=条带/门贴图
    （27x29 上移动画）/阴影浮沉；4=阴影+浮沉条带；8=阴影+直立（67/69
    拼装表多片组合：f_byte_arr2_01 行×f_byte_arr2_00 元数据）；16=阴影
    浮沉/门贴图；32=多形态（f_byte_19 倍率 + m_127 高亮）
  - walkPhase==2 && f_bool_08：交互浮标（iconStripDx/Dy[frameCounter&7]
    波动 + drawDigitStrip 数值）
- **m_033 玩家**（a.java:5829-5885）：sptprop[5][0] 阴影 + actor[3][0]
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
  （m_043 a.java:6302），帧推进在 m_055（每 tick 换帧——a.java:6806）。
- **m_049**（a.java:6408-6424）：组移除（f_byte_arr2_02 行×f_byte_arr2_03
  组成员，按类型筛除并置 removed）。
- **剩余件**（下批）：m_053 端口（floor1 仅需 case 1/2/8）+ m_033 +
  m_037/m_035 HUD + m_057 小地图 → gameplay-floor1 全帧对拍；mode 11
  对话框；mode 4/19 暂停菜单。

## 4. 证据基线

- 像素模型：`reference/shim/src/javax/microedition/lcdui/{Graphics,Font,Image,Canvas}.java`
- 游戏侧：paint（a.java:2333-3221）、logo（10187-10553）、mode21
  （3047-3054/4204-4223/4425-4460）、paintSoftkeyBar（5970-5980）
- golden 裁判：`data/golden/render-golden.json`（Java 微驱动
  reference/oracle/src/oracle/host/RenderGolden.java 可再生）
- 回放：`crates/game-oracle/tests/boot_replay.rs`（A-boot-menu TICK 1-70）
