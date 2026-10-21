# 对抗复核 2/3：渲染管线技术面

- **复核人视角**：渲染管线对抗评审（adversarial review，攻击方）
- **复核对象**：`docs/superpowers/specs/2026-10-21-hd-asset-probe-design.md`（A 档决策：素材像素=逻辑尺寸×density，逻辑坐标零漂移；density-aware 直采样 d==s 直拷 / d>s 降采样 / d<s 升采样，d=1 恒等）
- **时间**：2026-10-21
- **证据锚**：`crates/game-core/src/render.rs`、`crates/game-core/src/game_view.rs`、`crates/game-core/src/menu_family.rs`、`crates/game-core/src/paint.rs`、`crates/game-core/src/scene.rs`（只读对账）

## 攻击点 1：采样矩阵非整除（d=3,s=2 / d=2,s=3 相位）

- **风险**：非整除比的最近邻相位误差**按逻辑格逐格重置**（src=⌊i·d/s⌋，i=物理格内偏移）：d=3,s=2 时每逻辑格 3 源 px 只采 2 个，恒丢第 3 个；d=2,s=3 时 2 源 px 铺 3 物理 px，恒为 2:1 不均分。两者都在每个逻辑边界重复 → 周期性条带/细节丢失，不是随机噪声。
- **反例**：d=3,s=2，逻辑格 x=0 的源 px (2,0) 处画 1px HD 竖线——物理 x=0,1 只取源 px 0,1，**该线整列蒸发**；d=2,s=3，逻辑格内源列 0,1 交替黑白 → 物理行渲染为「黑黑白」2:1 条纹（每逻辑 px 一周期）而非均匀灰。
- **建议修正**：box filter 必须用**覆盖加权 + 精确有理边界**（dest i 的支撑 = [i·d/s,(i+1)·d/s)），整数分子分母运算、禁浮点（game-core 确定性）。推导结论：**d=1 时覆盖支撑 [i/s,(i+1)/s) 完全落在单一源 px 内，加权平均=自身 ⇒ 严格退化为现有 set_pixel_block ×scale（render.rs:371-379），alpha==0 skip 不变 ⇒ d=1 恒等可行**；但若用 ⌊·⌋ 区间实现会在 d=1 产生空区间，禁止。

## 攻击点 2：anchor 语义混用

- **风险**：`anchor_adjust` 入参是**物理尺寸**（render.rs:672 `img.width * s`、:688 `bw * s`）。HD 下 img.width=逻辑×d，直接乘 s 得逻辑×d×s ⇒ 锚点偏移放大 d 倍。HCENTER 误差 = 逻辑宽×s×(d-1)/2（d=3,s=3,16px tile ⇒ 偏 16 逻辑 px）。同类混用遍布消费点：把 img.width/height 当逻辑尺寸用的坐标式全错。
- **反例**：scene.rs:886/3548/3901 `(32 - img.width) >> 1` 居中——HD 下 (32-48)>>1=-8，tile 偏出 8 逻辑 px；menu_family.rs:138-155 draw_edge_patch 镜像偏移用 `iw=img.width` 当图集逻辑宽 ⇒ 翻转片错位 (d-1) 倍残量。
- **建议修正**：必须改的调用点（grep 对账）：① render.rs:672、:688 → `img.width/d * s`（建议 ArgbImage 加 `logical_width()/logical_height()` 统一出口）；② menu_family.rs:136-155 iw/ih → 逻辑宽高；③ scene.rs:886、3548、3901 → logical_width()；④ render.rs:669 op 记录 `drawImage({}x{}…)` 现记 img.width（物理）——spec §1.6 要求记逻辑尺寸，须 /d，否则 ops 流 d>1 对拍文本漂移；⑤ draw_text:776 / draw_glyph:795 是字体逻辑度量，d=1 时不动，字体升密度时另议。

## 攻击点 3：draw_image_transformed 8 变换

- **风险**：render.rs:697-728 逐像素循环 `for sy in 0..h, sx in 0..w` + `img.argb[sy*w+sx]`——w/h 直接当**源像素**又当**落笔格**，每源 px 写 s×s 块。HD 下源 = 逻辑×d：画幅变 d 倍大（width*d*s 物理），且变换在源像素域做，相位随 d 漂移。
- **反例**：d=3 的 16×16 逻辑 sprite（源 48×48）走 transform=5(ROT90)：循环 48×48 次、落笔 48×48×s² 物理块 = 目标格 3 倍；`h-1-sy` 等镜像表达式在 48 尺度取反，逻辑镜像轴整个错位。
- **建议修正**：最小改法=**变换域保持逻辑**：循环改为逻辑格网 `lx:0..w/d, ly:0..h/d`；源像素取 `sample(img, lx, ly, d)`（攻1 的密度感知采样器）；`(dx,dy)` 的 match 表一行不动（只是输入输出都变逻辑单位）；落笔 `set_pixel_block(ox+dx*s, oy+dy*s, p)`（d==s 时可直拷 d×d 源块）。禁止先把 HD 缩回逻辑再走旧循环（spec §1.2 两次重采样红线）。

## 攻击点 4：图集切片 ×d 与列数索引语义

- **风险**：paint_tiles（game_view.rs:228-229）`sx=(v&7)<<4, sy=(v>>3)<<4` 是**逻辑 16px 格坐标**，经 draw_edge_patch 的 clip+整图偏移（paint.rs:43-48 同构）落笔——只要 draw_image 密度感知且 img 尺寸按逻辑理解，切片本身**无需 ×d**；真正会算错的是：(a) menu_family.rs:138-155 镜像偏移里的 iw/ih 当逻辑宽（攻2）；(b) 若改成真·子图截取，矩形必须 (sx·d, sy·d, 16d, 16d)，图集 stride=物理宽。
- **反例**：陷阱①「列数 8」= `v&7` 的**帧索引语义**，帧数不变、帧像素×d 后依然成立——它不随 d 缩放。会错的是宽度型断言：HD 图集 width=8×16×d，任何 `assert(tileset.width==128)` 或 `frame=w/16` 式推导列数的代码在 d=3 下得 384/列数 24 ⇒ 索引语义没坏、断言先坏。
- **建议修正**：切片偏移一律逻辑域（现状正确，TDD 钉住）；图集断言改为 `width/(16*d)==8`（列数 8 断言保持 legacy 语义）；draw_edge_patch 的 iw/ih 换 logical_width()；禁止对 v、列数、16px 格距乘 d（只允许像素域边界 ×d）。

## 攻击点 5：density 载体推荐

- **风险/失败模式**：ArgbImage 字段——失败模式：合成图/离屏画布（`create()` 默认 d=1）与 HD 源混画时单一 d 无法表达，密度随 clone 隐式传播、`PartialEq`（render.rs:257）语义含糊。显式参数——失败模式：draw_image/draw_image_transformed/draw_edge_patch/parallax ~20+ 调用点（scene.rs grep 即 15+ 处）逐点穿 d，漏一处=静默错位（正是攻2 的病灶放大）。
- **建议修正**：**推荐 ArgbImage 字段**（默认 1）+ `logical_width()/logical_height()` 访问器 + `debug_assert!(width % d == 0)`；理由：demo 的核心场景=「混密度并存」，per-image 天然携带；调用点保持原签名，把换算收敛在渲染层 3 个函数内。合成/离屏图强制 d=1 并文档声明。

## 攻击点 6：性能量级

- **风险**：scale=3、416×416 逻辑 → 1248×1248 物理 ≈ 1.56M 物理 px/帧全屏重采样。对比现 blit（render.rs:733-743）：173K 源 px × set_pixel_block 9 次 = 1.56M 次 set_pixel（含逐 px clip 检查）——**写入总量同阶**，新路径每物理 px 1 次采样+1 次写（NN）或 ≤4 tap（box 降采样）。
- **反例（不可接受的实现）**：逐物理 px 浮点乘除 + 堆分配临时行缓冲 ⇒ 实测 3-5×；若再叠 draw_image_transformed 逻辑格循环而每格走 f64 插值 ⇒ 60fps 桌面掉帧风险。
- **建议修正**：估算倍率 **NN ≈ 1×、box ≈ 2–3× 现 blit**（同阶可接受，demo 无闸不阻塞）；d==s 走 `copy_from_slice` 行拷（比现逐块 set_pixel 更快）；整数有理采样、栈上/复用行缓冲；落地前以基准测试实测，条件 6 钉上限 ≤3× 现 blit。

## 总判定

**有条件通过**。A 档（逻辑零漂移 + density 直采样）在渲染管线可行，且 d=1 恒等在覆盖加权 box / NN 下均可严格成立（攻1 推导）。但现状代码把 `img.width/height` 当逻辑尺寸用（anchor、变换循环、镜像偏移、居中式、op 记录）——HD 下全部错位，**换算收敛不完成前禁止派工实现**；采样器若用浮点或 ⌊·⌋ 空区间实现将同时击穿确定性与 d=1 恒等。

## 条件清单

1. 采样器 = 整数有理相位（src 支撑 [i·d/s,(i+1)·d/s) 覆盖加权 box，d==s 直拷），禁浮点、禁 ⌊·⌋ 空区间实现；d=1 退化恒等由 TDD 双侧帧哈希钉住。
2. 换算收敛：render.rs:672、:688、op 记录（:669）；menu_family.rs:136-155 iw/ih；scene.rs:886、3548、3901 —— 全部改 `logical_width()/logical_height()`（=w/d），一处不落。
3. draw_image_transformed：逻辑格网循环 + sample(lx,ly,d)，8 变换表逻辑域不动，落笔块 s×s。
4. 图集：`(v&7)<<4` 格距/列数 8 索引语义保持逻辑域不动；宽度型断言换算 `width/(16*d)==8`；只对像素边界 ×d。
5. density 载体 = ArgbImage 字段（默认 1）+ 访问器 + `width%d==0` 断言；离屏/合成图 d=1 强制。
6. 性能门槛：全屏重采样实测 ≤3× 现 blit（NN 目标 1×）；d==s 路径行拷直采。
