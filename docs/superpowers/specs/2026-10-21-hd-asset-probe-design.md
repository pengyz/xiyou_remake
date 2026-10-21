# 高清素材探针设计（HD-asset probe）——设计件

> 2026-10-21。主线背景：资源高清化（用户诉求：原素材为 Nokia 分辨率像素画，modern 放大后
> 模糊，拟重制高清素材）。架构决策 **A 档已锁定**（用户裁定）：同逻辑尺寸 + 高像素密度
> （@2x/@3x 模式）——逻辑坐标/帧尺寸/碰撞/金样零漂移，素材以 density 声明，渲染按 density
> 直采样到物理画布。本件 = **探针**（用户裁定："开发阶段只用来做验证，你自己绘制部分常见
> tile 的高清图，比如地图图块，替换这些贴图到游戏渲染中，这样我们可以看效果"）。
> full 素材生产（AI 放大/画师）与 sprite/图集配置系统（[resource-config-survey.md](../../spec/resource-config-survey.md)
> v2）后续另件。**先经 3 路对抗复核，通过后派工**。

## 0. 边界（三件：1 留用 + 1 弃件 + 1 验证）

| # | 件 | 内容 | 归属 |
|---|---|---|---|
| ① | density-aware draw 路径 | render.rs 素材携带 density（默认 1）；d==scale 时 1:1 直拷零重采样；**d=1 与现实现逐字节恒等** | **留用件**（量产渲染层同款） |
| ② | 3x demo 图集 | 常见 tile（地图图块为主）程序化精绘 + 其余最近邻放大垫底——**混合质量 = 量产"分批换素材、混密度并存"预演** | **弃件**（只进 mod/验证面，永不进保真面） |
| ③ | 无头视觉对照 | 同场景 scale=3 classic vs HD 双截图（纯渲染测试/Xvfb 产出）；可选 shell `--hd-assets` 开关 | 验证（visual-diff 门禁未就绪=SKIP，**本件不建闸**，效果由用户人工判读） |

**不做**：素材生产管线（AI 放大）、sprite/图集配置系统 v2、visual-diff 门禁建设、
布局/碰撞/相机改动、保真面任何变更。

## 1. 技术要点

1. **模糊根源**：`draw_image` 最近邻 ×scale（render.rs draw_image，task-13 ④ 硬边保像素味）。
   高清收益只在 **scale≥2** 可见——对照场景必须 scale=3，否则假阴性。
2. **直采样约束**：高清源 → 物理画布**一次映射**（目标物理格 = 逻辑格×scale；源 = 逻辑格×d）。
   禁止"缩回逻辑尺寸再放大"两次重采样（细节被吃 = demo 失真）。
3. **d=1 恒等**：现实现 = 新路径在 d=1 的退化情形，TDD 双侧钉住（classic 帧哈希零漂移）。
4. **采样矩阵**：d==s 直拷；d>s 降采样；d<s 升采样（算法待复核定夺：最近邻 vs box filter）。
5. **消费面盘点**：draw_image / draw_image_transformed（8 变换）/ paint_tiles 图集切片
   （源坐标×d；**图集列数 8 = 索引语义**（asset-hardcode-audit 陷阱①）——HD 图集帧数不变、
   帧像素×d，切片索引语义不动）/ parallax / HUD / anchor_adjust（以逻辑尺寸入参）。
6. **ops 流**：drawImage op 记录逻辑尺寸 ⇒ d=1 恒等；demo 面 ops 不进 L1-L3 对拍。
7. **density 载体**：ArgbImage 字段 vs 显式参数——待复核定夺。
8. **图集生成红线**：demo 图集帧数/帧序与原图集**完全一致**（只换像素密度），切片边界
   由帧宽×d 推导；陷阱① 的列数 8 断言 legacy 值不变。

## 2. 判据

- **TDD 红→绿**：d=1 恒等测试（红：现路径无 density 概念）+ 采样矩阵测试 +
  classic 零漂移（oracle 35/35 帧比对 + `gates check`/`trace` 全绿）。
- **视觉**：scale=3 同场景双截图（classic vs HD），用户人工判读 = 验收本体。
- **弃件隔离**：demo 图集不入 `original/`、不入保真表、不进金样运行。

## 3. 风险（预登记，待复核攻防）

- 截图对照证据效力弱（visual 闸 SKIP）——以"用户人工判读"为验收，不冒充闸。
- 程序化绘制画质上限——demo 只证管线正确与清晰度收益，不证最终画质。
- 软光栅逐像素采样性能（scale=3 全屏）——量级待实测。

---

# v1.1 三路对抗复核吸收（2026-10-21，原地标注不删 v1 主张）

> 三路复核均**有条件通过**：[1/3 保真红线](../../findings/reviews/2026-10-21-hd-probe-review-fidelity.md)
> C1-C5、[2/3 渲染技术](../../findings/reviews/2026-10-21-hd-probe-review-pipeline.md) 条件 1-6、
> [3/3 范围排期](../../findings/reviews/2026-10-21-hd-probe-review-scope.md) C1-C8。逐条吸收如下。

## R-0 lead 裁决：采样算法（2/3"覆盖加权 box" vs 3/3"最近邻"条件冲突）

**探针裁定 = 最近邻（NN）**，理由：①探针场景 d=3、s=3 ⇒ 走 d==s 1:1 直采样，降采样路径
不被行使；②NN 与 task-13 ④"硬边保像素味"哲学一致；③性能 1×（box 2-3×）；④范围最小。
**覆盖加权 box（整数有理相位，禁浮点/禁 ⌊·⌋ 空区间）登记为量产备选**——2/3 攻1 的
"d=3,s=2 恒丢第3源px/d=2,s=3 条带"分析随登记保留，量产真素材阶段按画质需求再裁。

## R-1 采样器（吸收 2/3 条件 1+6、3/3 C1）

- NN 整数相位映射 `src = l * d / s`（整数运算，禁浮点）；d==s 行拷直采。
- **d=1 退化恒等** = `set_pixel_block ×scale` 逐字节相同，TDD 双侧帧哈希钉住。
- 性能门槛：全屏重采样 ≤3× 现 blit（NN 目标 1×）。
- 【3/3 C1】留用件 d>1 分支登记"**未闭环待真素材复验**"；真素材接入再验收判据
  （量产素材到货时重跑 d≠s 采样矩阵+画质人工四问）。

## R-2 换算收敛清单（吸收 2/3 条件 2——派工硬前置，一处不落）

`img.width/height` 被当逻辑尺寸用的全部点改 `logical_width()/logical_height()`（=w/d）：
`render.rs:672`（anchor_adjust 入参）、`:688`（transformed anchor）、`:669`（op 记录）；
`menu_family.rs:136-155`（iw/ih 镜像偏移）；`scene.rs:886`、`:3548`、`:3901`（(32-img.width)>>1 居中式）。

## R-3 transformed 8 变换（吸收 2/3 条件 3）

逻辑格网循环 + `sample(lx,ly,d)`；8 变换表**逻辑域不动**；落笔块 s×s。anchor×transform
矩阵测试进判据（3/3 C1）。

## R-4 图集切片（吸收 2/3 条件 4、3/3 C3、1/3 C1）

`(v&7)<<4` 格距/列数 8 **索引语义逻辑域不动**（陷阱①断言锚不许动）；宽度型断言换算
`width/(16*d)==8`；只对像素边界 ×d。现状 clip+整图偏移切片方式免 ×d（2/3 对账结论）。

## R-5 density 载体（吸收 2/3 条件 5、3/3 C2）

**运行时 = `ArgbImage` 字段（默认 1）+ `logical_width()/logical_height()` 访问器 +
`width%d==0` 断言**；离屏/合成图强制 d=1。**持久化 = 探针零 schema**（demo 图集不进
任何配置表）；resource-config-survey §2.1 注记预留 `density` 字段给量产批。

## R-6 ops 流隔离（吸收 1/3 C2）

op 一律记**逻辑尺寸**（与 density 无关）；加 classic/HD **op 串全集相等**断言
（同场景同帧两源跑出的 ops 序列逐条恒等 = 换素材零行为漂移证明）。

## R-7 demo 弃件隔离（吸收 1/3 C3、3/3 C7）

demo 图集固定独立目录 `assets/hd-demo/`；`asset_roundtrip` 的 in_scope **显式排除
demo 前缀 + 护栏测试**（误入即红）；demo PNG 不入库（assets/ 本就 gitignore）、
生成脚本+帧哈希清单入 `tools/hd-demo/`；弃件删除条件 = 量产素材接入再验收通过后。
**--hd-assets 开关去留二选一裁定**（观察-1，2026-10-21）：量产素材接入时裁——
转正为正式 HD 素材开关，或随弃件一并删除；探针期保留现状（可选开关，试玩反馈≠验证成果）。

## R-8 验收两层（吸收 1/3 C4、3/3 C4/C5/C6）

- **机器判据**（进 TDD/commit 证据）：d=1 恒等（帧哈希+op 串）、d==s 直采样恒等、
  非素材区恒等、demo 图集帧序/帧数断言。
- **人工判读四问**（截图，Evidence 标 **C 级**，判读≠验证）：①高清 tile 边缘是否清晰
  ②混密度交界表现 ③非素材区无变化 ④整体观感。"demo 画丑但管线对"与"管线错"的区分
  = 机器判据全绿+四问①③过 ⇒ 管线对；机器判据红 ⇒ 管线错。
- 截图只走**无头纯渲染测试**（AGENTS §5 验证不打扰）；用户 `--hd-assets` 试玩反馈
  **不得记为验证成果**。
- 【3/3 C5 勘误】"oracle 35/35"无可复现口径 → 统一写 **`cargo test -p game-oracle`
  全绿** + `gates check`/`gates trace` 全绿（合主干前置，1/3 C5：提工者跑+复核人二次 trace）。

## R-9 排期与 commit 拆分（吸收 3/3 C5）

3-4.75 人日、机器墙钟 ≤2-3h；commit 拆 6 块（换算收敛 R-2 / 采样器 R-1 / transformed
R-3 / 图集断言 R-4 / demo 图集 R-7 / 对照 harness R-8），一 commit 一逻辑变更一闸。

## R-10 边界声明与回写（吸收 3/3 C8、C3）

四界边界（font-metrics 先例）：与 resource-config-survey（v2 配置系统）、
asset-hardcode-audit（硬3——本探针**不碰** C1/C3/C5/P5/P7 编辑点，A5 并入其合并账本）、
font-metrics（TTF 度量）、master-plan 素材行（其"data 层换实现"字面与探针改 render.rs
有缝——**注记**：A 档 density 直采样是缝内实现，量产素材替换仍走 data 层；回写待本探针
验收后一并做）。

## 吸收对照表

| 来源 | 条件 | 落点 |
|---|---|---|
| 1/3 | C1 测试面 | R-1/R-3/R-4（transformed×anchor 矩阵+列8索引进判据） |
| 1/3 | C2 op 隔离 | R-6 |
| 1/3 | C3 demo 防入 | R-7 |
| 1/3 | C4 证据级别 | R-8 |
| 1/3 | C5 回归清单 | R-8 |
| 2/3 | 1 采样器 | R-0/R-1 |
| 2/3 | 2 换算收敛 | R-2 |
| 2/3 | 3 transformed | R-3 |
| 2/3 | 4 图集 | R-4 |
| 2/3 | 5 载体 | R-5 |
| 2/3 | 6 性能 | R-1 |
| 3/3 | C1 未闭环登记 | R-1 |
| 3/3 | C2 持久化 | R-5 |
| 3/3 | C3 硬3 归属 | R-10 |
| 3/3 | C4 验收两层 | R-8 |
| 3/3 | C5 排期/口径 | R-8/R-9 |
| 3/3 | C6 不打扰 | R-8 |
| 3/3 | C7 处置条款 | R-7 |
| 3/3 | C8 边界回写 | R-10 |

## v1.2 lead 裁决·R-2 批次二登记（2026-10-21，hd-render-path grep 对账新发现）

task-1 交付时 grep 对账发现 **R-2 七点清单外的同类消费点**（Java getWidth/getHeight 移植点
用 img.width/height 当逻辑尺寸；d=1 时零漂移、d>1 时同病）。裁定：**登记不扩圈**——
探针 demo 只有 map 图集走 d=3，以下点均消费 d=1 素材，探针期不动（保持最小爆炸半径）。
**量产批"换算收敛第二批"清单**（届时按 R-2 同法全改 logical_*()）：
① 同式居中/宽度推导：menu_family.rs:1032、:242-243（digits 条切片）、:109（parallax
tile.height 步进）、scene.rs:6496-6497（HUD 槽居中）、:6516/:6520/:6689（ending）、
:3891（draw_h=img.height）；② 布局/游戏逻辑消费：scene.rs:1151/1158/1517/1543/1565/
2282/2294/2462/5072/5370（im.height→spawn 等）；③ logo_anim.rs:162-266、intro.rs:222-236。
（与 3/3 C1"未闭环待真素材复验"同款处置：登记+量产再验收。）

另记：test-ratchet 新二进制 density_draw(16) 待 ratchet-update（基线 data/status/baselines
归 lead 写域）——集成期经 verifier 确认无回退后随集成 commit 更新。
