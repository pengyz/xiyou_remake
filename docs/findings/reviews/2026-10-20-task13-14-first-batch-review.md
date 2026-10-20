# 对抗审查：task-13/14 首批 37b53a4（光栅×倍率 + 字形物理光栅化 + origin 层核心）——**通过**

> 审查人：adversary。对象：37b53a4（render.rs +232 / scene.rs +51 / main.rs 面=被卷
> 的 shell-port 集成件，task-A 已审）。对照：三判据（classic 1× 双面恒等 /
> 度量与像素分离含基线格底锚 / 平移×缩放可交换）。
> 手段：断言包含比对（python）、GREEN 独立复跑、oracle ops 面复跑、RED 抽核、三对账。

## 判定：**通过（首批）**；3 条注记不卡闸（task-25 声称面确认 / 豁免簿分类 /
余量批次续审）

## 1. ① classic 1× 双面恒等 —— ✅

- **断言包含比对**（python）：render.rs 19→26 断言行 **missing=0**、scene.rs 332→332
  **逐行无缺**——既有断言零改动（命门同款字节面）✓；
- **oracle ops 字节级**：其 GREEN 声明 t544 ops 零偏离（ops 记调用方逻辑坐标、origin
  隐式不入 ops 面）——我独立复跑 `gameplay_floor1` 2 过 ✓（3 ignored=dump 工具类）；
- GREEN 复跑：game-core 132(lib)+9 箱全绿（197 过 +1 探针类 ignore）、game-desktop
  39/39 ✓（其 message 时点 37 系 task-A 终态前快照，时间线自洽）。

## 2. ② 度量与像素分离 —— ✅

- SoftFont 度量面零改动（height+4 行高派生链、基线字段）+ `font_metrics_tests`
  classic 恒等断言（6/14/11、8/14/11、12/20/16 逐项）✓；
- 字形物理光栅化：GlyphHook 签名 +scale 末参（图形层=倍率单源真相）、钩子入参=逻辑
  含 origin / put=物理坐标、基线落点 `(py+chh-1)*ss`（ab_glyph 平移教训保留）✓；
  `stroke_2x2_is_real_physical_pixels`（2×2 真物理像素非放大色块）+
  `placeholder_glyph_blocks_at_scale2` 双面钉 ✓。

## 3. ③ 平移×缩放可交换 —— ✅

- `phys_x = (x + origin_x) × scale`——**平移先于缩放**（设计面直接可交换）；
- `translate_scale_commute`：origin 平移绘制像素 == 显式平移参数绘制（同 scale）逐字节 ✓；
- ops 面不带 origin（纯平移不污染逻辑坐标流）✓；`draw_line_raw` 拆分避 drawRect
  双倍平移陷阱 + paint_parallax 全幅豁免 + paint_wipe_tail 归零 + 8 菜单臂
  dx=bottom_panel_dx()——组合面纪律完整（立绘漏平移类问题已有后续 51473af 组合钉）。

## 4. TDD / 三对账 —— ✅

- RED：符号缺席法（set_scale/set_origin/dots_to_physical 父树 0 命中）+ 复现指令 +
  过程入册（E0503 借用冲突 10min 红窗、shell-port 通报）✓；
- 三对账：task-13/14 声称面与内容一致；**被卷面（shell-port main.rs 集成件）经 G7
  事故登记 + be4fee8 message 前向更正+分域清单闭合**（37b53a4 message 未含这些面一事
  已记 G7，不再重复计过）✓。

## 5. 注记（不卡闸）

1. **task-25 声称面**：FontMetrics trait/ClassicMetrics（font-metrics-dependency-
   survey §3 引）在 diff 内，message 以"font_metrics_tests 判据/证据"携带——若 task-25
   系 lead 独立派单则声称面成立，请 lead 确认挂链；若非则补声称。
2. **豁免簿分类记账**：新增 1 枚 `#[ignore]`（probe_bottom_region_pixel_metrics，
   51473af 代）属**探针类**（依赖 analysis/ 可再生工作区，自包含化后转正）——不属
   64 枚帧回放族冻结面，但豁免簿应分三类记账（帧回放 64 / 探针 / dump 工具）免混淆。
3. **余量批次**：开场链接线（title/intro/logo_anim/Loading——用户"加载界面没对齐"主面）
   + 菜单帧组合纯平移场景级判据 + headless 双图——随后续提交续审（与 shell-port 物理
   背板启用联动）。
