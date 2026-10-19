# STAGE-B-004a 自审查记录（task-23，porter-arms，2026-10-19）

## 交付面（commit 8955443）

| 件 | 内容 | 判据/证据 |
|---|---|---|
| ①a 粒子生成域随画布 | intro 雨+标题粒子 randomBelow(240)→layout 域 | TDD 测 modern_particle_spawn_domain_spans_canvas；变异 RED（旧域 max x=218≤239） |
| ①b 软键图标悬空锚 | m_034_softkeys/paint_sound_prompt 锚→layout 字段 | TDD 测 modern_softkey_icons_anchor_to_layout；变异 RED（y=307 残留） |
| ①c 对话框底边距 | **误报关闭**（2026-10-19 lead 裁定） | 六态探查零黑边+lead 复现图两例（shot-m-520/340）像素核**底行零黑像素**——深色带=对话框自身 HUD_FILL(293231) 底色非 artifact；判据测 modern_bottom_band_sides_never_black **保留看护** |
| ② 标题画面 | 240×320 原尺寸（横=居中/**纵=底对齐**贴底栏）+ 边缘**平色**延展带+按键提示锚位公式化（2026-10-19 措辞勘误：纵非居中） | headless 实拍 title-v4（纯蓝带） |
| ③ 序章过场 | 立绘居中+黑带延展+字幕框公式+"跳过"锚 | headless 实拍 intro-280（全幅带+跳过锚+粒子全幅） |
| 壳换轨 | BootMachine.apply_layout（壳侧注入） | 同 sha 实证定位（不换轨 modern 走 classic 分支） |
| 工具挂点盲区 | state_baseline --include-ignored/--no-fail-fast + 豁免红/新红区分 | TOLERATED 22/60 机制实测；判据不放宽 |

## 分级策略执行核对

- 手绘素材=**居中+延展不放大**：标题图（横居中/纵底对齐）/序章立绘（居中）原尺寸+**平色**延展带（用户验收修正：纯蓝/素材边缘色，无渐变波浪纹）✓
- 可重排 UI=**锚位公式化**：图标带 center_x−34、双箭头 center_x−60/+40（layout.rs:370-371
  公式表回代）、字幕框画布宽−20 居中、跳过贴右 ✓
- classic 零偏离：if/else 分支保留原式（标题）+layout 代入同值（粒子/软键/序章）✓

## 定性证据（金样现状）

- **预存阻塞非本批回归**：stash 对照——我改动前后帧失败同数（35）同点（TICK 68 mode=1）
  =STAGE-B-001 开机时间轴变化 fallout（b3d3cc5 #[ignore] 注"夹具重定时待办"）。
- **状态中立证明**：同测试 dump 逐字节 cmp 相同（gameplay-floor1/preset-help 两产样）。
- 金样 GREEN 需先清夹具重定时（STAGE-B-001 线待办）——归属待 lead 裁定。

## 待补/待裁定

1. ~~①c 复现态~~ **误报关闭**（2026-10-19）：lead 复现图 shot-m-520/340 底行像素核
   零黑（两侧=沙地 f87b7a、中段=HUD_FILL 293231 对话框底色）——lead 目测误报确认，
   判据测保留看护。
2. 夹具重定时（64 枚 #[ignore] 的解除前置）——**lead 裁定另立任务**（不并 task-23），
   派 items-evidence 执行；线索已交接（35 失败同点 TICK 68 mode=1）；完成后金样
   GREEN 路径=TOLERATED 60→0。
3. task-24（menu 线）在途工作与本批共享 scene.rs——已 hunk 级剥离（commit 含 0 他线痕迹），
   其 menu_family.rs/scene.rs 菜单块改动保留工作树待其收尾。
