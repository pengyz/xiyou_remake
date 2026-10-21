# 对抗复核 1/3：保真红线 / 验证面 —— HD 素材探针设计

> 复核人视角：verifier（对抗复核，专挑保真红线与验证面漏洞）
> 复核对象：`docs/superpowers/specs/2026-10-21-hd-asset-probe-design.md`
> 时间：2026-10-21
> 状态：骨架已建，逐节补全中

## 0. 复核范围与证据锚

设计主张摘要：① render.rs 增加 density-aware draw 路径（d=1 与现实现逐字节恒等，留用）；
② 3x demo 图集（弃件，只进 mod/验证面）③ 无头 scale=3 classic/HD 双截图人工判读（visual 闸 SKIP、不建闸）。

证据锚：
- `crates/game-core/src/render.rs:670`、`render.rs:681`（设计点⑤声明后续行号以实际为准）
  — draw_image / draw_image_transformed 的 op 均记录 `img.width×img.height`（像素尺寸）；`render.rs:726` blit 最近邻 ×scale。
- `crates/game-data/tests/asset_roundtrip.rs` — unpack∘repack 逐字节恒等，只遍历 assets/raw 三族（maplv/sprite/packed_png）。
- `gates/cli.py:89` — assets-manifest ratchet 基线 138 条，计数仅针对 original JAR 内条目。
- `gates/cli.py:449` — cmd_check 总闸清单（10 项），visual 不在内。

## 1. d=1 恒等证明力

### 风险
TDD 双侧钉住若只覆盖 draw_image 单路径，d=1 退化只保住主 blit；transformed/图集切片/
parallax/HUD/anchor 任一漏钉即静默绿，恒等断言名不副实。

### 反例
transform=5 旋转组合 anchor：`render.rs:688` 用翻转后 bw/bh 算位移，d 若按逻辑尺寸去*×d 会漂，
夹具没盖到即绿；paint_tiles 图集"列数 8=索引语义"（asset-hardcode-audit 陷阱①）一错整片地图错位。

### 建议修正
恒等最小测试面 = draw_image（anchor 四角）+ transformed 8 变换 ×anchor + paint_tiles 列 8 索引切片
+ parallax + HUD + anchor_adjust；最易漏为 transformed 的 swap 后 anchor（render.rs:688）与图集列索引。
每条断言逐物理格字节恒等 + op 串恒等。

## 2. ops 流隔离

### 风险
`render.rs:670/681` op 记录像素尺寸 `width×height`；HD 素材像素宽=逻辑×d，若改动顺带把 op
记成像素尺寸，op 序列即漂移，L1-L3 对拍误报。

### 反例
同一场景 classic `drawImage(16x16,..)` vs HD `drawImage(48x48,..)`，oracle 对 op 串取等 → 零漂移断言崩，
且无 density 开关时被误判为保真回归。

### 建议修正
d 只作用于采样与物理格，op 一律记逻辑尺寸（与 density 无关）；加单元断言：classic/HD 同场景
op 串全集相等，仅 blit 输入像素分布不同。

## 3. demo 图集误入金样路径

### 风险
`asset_roundtrip.rs` 只扫 assets/raw 三族；demo 图集若以 spriteNN 命名、像素×3 但列数仍 8 丢进
assets/spriteNN 会触发 unpack/repack 恒等失败，却被误当"该修的错"顺手回退基线；若放错异目录则静默放大。
`gates/cli.py:89` assets-manifest 仅数 JAR 内 138 条，demo 不入 JAR = 完全不可见。

### 反例
把 demo 图集放 `assets/raw/sprite55.pak`（越 55 边界）或复用 sprite04 名 → 恒等门禁红；
或放 `analysis/` 下无人守卫，后续被脚本顺手收入金样推断。基线 138 条两种都拦不住。

### 建议修正
demo 图集固定独立目录（如 `analysis/hd-demo/`），asset_roundtrip.in_scope 显式排除 demo 前缀，
并加护栏测试：demo 名塞 in_scope 必须返回 false；任何进入 assets/raw 三族即 red。

## 4. 截图验收的证据级别

### 风险
visual 闸 SKIP、靠人工判读——若 commit Evidence 标 A 级，等于把"看起来对"冒充硬证据，
下一环节可能误采为保真结论。

### 反例
"demo 画丑但管线对" vs "管线错"：两张截图都是"看起来"。人工判读只能给 C 级；标 B/A 会把
纯视觉印象当可复现推断。

### 建议修正
截图类证据一律标 C 级并注明"本件不建闸、视觉画质不进证据"。"管线对"判据 = traces/L1-L3 零漂移
+ demo 遵守采样矩阵（d==s 直拷）；"管线错"判据 = HD 截图出现坐标偏移/错位/闪烁（违反一次映射约束）。

## 5. oracle 回归面

### 风险
探针改动落在渲染/核心面；若只跑 demo 的 scale=3 截图而漏 oracle 回归，d=1 回归可能带病入库。

### 反例
density 字段默认值处理错（默认≠1），scale 路径全盘漂移但 demo 截图"看起来对"，oracle 35/35 沉默漏报。

### 建议修正
合并主干前由提工者跑 `gates check` + oracle 35/35 帧比对 + `gates trace`（L1-L3 零漂移）；
独立复核人二次跑 `gates trace` 确认。时机 = 探针合主干前（非 demo 面验收后）。

## 总判定

**有条件通过**（C1–C5 全部吸收后方可派工）。

## 条件清单

- C1 d=1 恒等最小测试面：draw_image + transformed 8 变换×anchor + paint_tiles 列8索引 + parallax + HUD + anchor_adjust，逐物理格字节恒等 + op 串恒等。
- C2 op 一律记逻辑尺寸，与 density 无关；加 classic/HD op 串全集相等断言。
- C3 demo 图集固定独立目录；asset_roundtrip.in_scope 显式排除 demo 前缀 + 护栏测试。
- C4 截图 Evidence 一律标 C 级；"管线对/错"判据写入 commit 说明，视觉画质不进证据。
- C5 合主干前：提工者跑 `gates check` + oracle 35/35 + `gates trace`，复核人二次 `gates trace`。
