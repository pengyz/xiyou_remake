# STAGE-B-004a 自审查记录（task-23，porter-arms，2026-10-19）

## 交付面（commit 8955443）

| 件 | 内容 | 判据/证据 |
|---|---|---|
| ①a 粒子生成域随画布 | intro 雨+标题粒子 randomBelow(240)→layout 域 | TDD 测 modern_particle_spawn_domain_spans_canvas；变异 RED（旧域 max x=218≤239） |
| ①b 软键图标悬空锚 | m_034_softkeys/paint_sound_prompt 锚→layout 字段 | TDD 测 modern_softkey_icons_anchor_to_layout；变异 RED（y=307 残留） |
| ①c 对话框底边距 | **未复现**（六态像素探查） | 判据测 modern_bottom_band_sides_never_black 现绿；复现态待补 |
| ② 标题画面 | 240×320 原尺寸居中+边缘钳制延展+按键提示锚位公式化 | headless 实拍 title-v3（居中+四带延展） |
| ③ 序章过场 | 立绘居中+黑带延展+字幕框公式+"跳过"锚 | headless 实拍 intro-280（全幅带+跳过锚+粒子全幅） |
| 壳换轨 | BootMachine.apply_layout（壳侧注入） | 同 sha 实证定位（不换轨 modern 走 classic 分支） |
| 工具挂点盲区 | state_baseline --include-ignored/--no-fail-fast + 豁免红/新红区分 | TOLERATED 22/60 机制实测；判据不放宽 |

## 分级策略执行核对

- 手绘素材=**居中+延展不放大**：标题图/序章立绘均原尺寸居中+边角色延展（v6 同法）✓
- 可重排 UI=**锚位公式化**：图标带 center_x−34、双箭头 center_x−60/+40（layout.rs:370-371
  公式表回代）、字幕框画布宽−20 居中、跳过贴右 ✓
- classic 零偏离：if/else 分支保留原式（标题）+layout 代入同值（粒子/软键/序章）✓

## 定性证据（金样现状）

- **预存阻塞非本批回归**：stash 对照——我改动前后帧失败同数（35）同点（TICK 68 mode=1）
  =STAGE-B-001 开机时间轴变化 fallout（b3d3cc5 #[ignore] 注"夹具重定时待办"）。
- **状态中立证明**：同测试 dump 逐字节 cmp 相同（gameplay-floor1/preset-help 两产样）。
- 金样 GREEN 需先清夹具重定时（STAGE-B-001 线待办）——归属待 lead 裁定。

## 待补/待裁定

1. ①c 复现态：六态探查未见"游戏视图底黑边"——需用户/lead 提供复现截图或状态。
2. 夹具重定时（64 枚 #[ignore] 的解除前置）——STAGE-B-001 线待办，金样 GREEN 前置。
3. task-24（menu 线）在途工作与本批共享 scene.rs——已 hunk 级剥离（commit 含 0 他线痕迹），
   其 menu_family.rs/scene.rs 菜单块改动保留工作树待其收尾。
