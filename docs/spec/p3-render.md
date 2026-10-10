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
  路径（游戏唯一弧调用 fillArc(…,0,360) a.java:6644 短路），已补齐对齐 shim。
- D-2 `drawSubstring` 负 len：shim 空循环静默，Rust 原 panic——已按 shim 语义钳制
  （调用方 a.java:5044 的 len 是差值可能为负）。
- R-5 mode 0→21 清理分支补全：f_bool_30 复位（reset_layout_flag）+ f_int_02=75
  帧间隔字段化。
- R-8 layout.rs 头注收窄：仅覆盖 boot 渲染路径；camera 的 view_h=252、walk 的
  边缘阈值 106 是**独立游戏常量**（f_int_59 等），扩视口时不可盲目替换。

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

## 4. 证据基线

- 像素模型：`reference/shim/src/javax/microedition/lcdui/{Graphics,Font,Image,Canvas}.java`
- 游戏侧：paint（a.java:2333-3217）、logo（10187-10553）、mode21
  （3047-3054/4204-4223/4425-4460）、paintSoftkeyBar（5970-5980）
- golden 裁判：`data/golden/render-golden.json`（Java 微驱动
  reference/oracle/src/oracle/host/RenderGolden.java 可再生）
- 回放：`crates/game-oracle/tests/boot_replay.rs`（A-boot-menu TICK 1-70）
