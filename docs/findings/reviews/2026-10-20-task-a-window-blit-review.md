# 对抗审查：task-A 窗口 blit 侧（shell-port，预提交复核）——**通过**

> 审查人：adversary。对象：shell/win 工作树交付（blit_1to1 + set_scale 管理 +
> task-13 接口对齐 + headless 全倍率出图）。写域 shell/win/。
> 手段：39 测独立复跑、断言保全 python 逐行证、classic 1× 恒等判据核、
> 死码 grep、headless 产物实查、事故面核。

## 判定：**通过**（commit+push 前附两件：RED 三件入 message + pathspec 提交）

## 1. classic 1× 恒等 —— ✅（blit 面字节级，ops 面留 task-13）

- `classic_1x_blit_identity`（view.rs:198-210）：blit_1to1 vs legacy_blit 同入出
  `assert_eq!(new_out, old_out)` **逐字节**（含信箱+alpha 归一）✓；
- alpha 归一式两函数同款（`0xFF00_0000 | (p & 0x00FF_FFFF)`）亲验 ✓；
- 39/39 独立复跑全绿（含新增 3）；ops 流字节恒等=task-13 光栅侧面（其复核收），
  shell 侧以 blit 字节恒等为界——分工清晰 ✓。

## 2. 对拍资产与断言保全 —— ✅（python 逐行证）

- legacy_blit 保留 `#[cfg(test)]` 作对拍 oracle（生产路径零引用）——不是死码是**对拍
  资产**，判定正确；
- 被测物换名 1 处（blit→legacy_blit）：**断言行 3/3 逐字节相同**（python 比对证），
  实际语义 delta = 1 注释行 + 1 调用换名——声称与内容严格一致 ✓；
- 新增 3 判据齐：`blit_1to1_pastes_backing_and_blacks_out_bars`（1:1+信箱+alpha）、
  `set_scale_rebuilds_canvas_at_physical_size`（resize 重建/同 scale 不重建）、
  `repaint_is_state_neutral`（帧计数/像素/状态三不动）✓。

## 3. 接口对齐与死码 —— ✅

- `canvas_graphics(screen, scale)` → `g.set_scale(scale)`（main.rs:483）、
  `Shell::set_scale`（:862，resize⇒背板重建+repaint）、present 链 = set_scale→blit_1to1、
  resize 调用点（:1464）接齐 ✓；
- **nn_upsample/Backing 零残留**（grep 无命中）✓ 过渡桩全退役；
- 字形物理光栅化 glue（font_inject scale×格高、px×ss 落点）= task-13 界面，
  落点判据归其复核 ✓（我届时按基线/格底锚收）。

## 4. headless 双图自验 —— ✅

- 机制在位（headless_run + DSH_FONT_SSAA=all 同状态逐倍率重绘出图，:1569-1641）+
  `repaint_is_state_neutral` 锁无状态推进 ✓；
- **产物实查**：%TEMP%\shota\shot_1x..4x.png 四件在（307KB/1.2MB/2.7MB/4.9MB——
  尺寸随 240×320→960×1280 物理面比例增长 ✓）；2× 字形真锐度目检=lead/用户视觉面
  （AGENTS §5 不打扰口径）✓。

## 5. 事故面核 —— 如实可收

nn_upsample `x*s` 双重偏移越界（曾挡总闸）——10 分钟修复、panic 期未入树 ✓；
自查自纠对拍构造错 ✓；render.rs:799 红窗归 task-13 in-flight（共享树分域清楚）✓。
共享树注意：当前暂存区混有 task-13 的 render.rs/scene.rs——**提交必须 pathspec
（仅 shell/win）**，G6 三机制教训。

## 6. commit+push 前置两件（A 协议）

1. **RED 三件入 commit message**（3 新测试的失败输出逐字/一致性声明/复现指令——
   预提交复核面我按约定在终验 commit id 时一并核）；
2. **pathspec 提交**（`git commit -- shell/win/...`，勿裸 commit/add -A）。

push 后附 commit id，我做终验（三对账+message 红证据抽查）。
