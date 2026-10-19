# 2026-10-19 STAGE-B-004b 游戏内菜单面分级适配（menu 线）自审查

> 执行：interact-evidence（task-24）。范围：游戏内菜单（InGameMenu）①菜单框重排
> ②底部 HUD 信息栏贴底随宽 ③高亮/排布随框。分级策略：**框体=可重排 UI 锚位公式化、
> 悟空立绘=素材居中不放大**。写域=menu_family.rs（scene.rs 接线 hunk 见 §4）。

## 1. 交付内容

| 项 | 落点 | 内容 |
|---|---|---|
| ①③ 框体/高亮/箭头锚位公式化 | menu_family.rs `InGameMenuBox` + `in_game_menu_box()` | classic 代入（center 120/160）逐值化回 a.java:2431-2444/2486-2487 原公式；modern 随 `layout.center` 重排；高亮矩形（hl_x/hl_w=条目宽，Java 原样）与滚动箭头随框派生 |
| ② 底部 HUD 信息栏贴底随宽 | menu_family.rs `HudPanelAnchor` + `hud_panel_anchor()` | x=0、w=screen_w（STAGE-B-002 v5 底部带背景同款拉伸判据：画布宽>经典 240 ⇒ 铺满）；classic 代入=(0, view_bottom, 240, h) 与现行 `bottom_panel_dx()+SCREEN_W` 逐值同；立绘/属性条内容锚仍走 Layout（hud_face_ax/hud_icon_x 居中）= **立绘素材居中不放大** |
| 接线 | scene.rs `paint_menu_list`（3 hunk：几何块/高亮/文本+彩虹+箭头）、`paint_hud`（1 hunk：frame 锚改 anchor） | 全部替换为 menu_family 公式取值；classic 路径逐值不变 |

**框体拉伸语义裁定**：框体=内容尺寸**居中不拉伸**（六项内容框拉伸随宽观感差）；
若日后要拉伸，`in_game_menu_box` 一处可改（v5 判据）。底栏（②）=随宽铺满（v5 判据本体）。

## 2. TDD 红证据三件套

- **判据先行**（测试代码与本 commit 逐字一致，menu_family.rs
  `stage_b_004b_menu_anchor_tests` 3 测）：
  `in_game_menu_box_classic_substitutes_to_java_literals`（classic 代入化回原值——
  bx/by/pad/first_row_y/hl_x/hl_w/arrow 逐值对照 a.java:2433-2435/2438/2444/2486-2487）；
  `in_game_menu_box_centers_on_layout_modern`（modern center 224/208：框心/高亮心=layout center）；
  `hud_panel_anchor_classic_zero_deviation_modern_full_width`（classic (0,252,240,50) 零偏离
  + modern (0,276,448,50) 贴底随宽）。
- **RED 复现指令**：把 `in_game_menu_box` 的 bx/by 公式还原为 legacy 120/160 常量锚
  （或 `hud_panel_anchor` 还原为 `(screen_w-240)/2` 偏移+240 宽）⇒ modern 断言破即红；
  视觉 RED = 下方 before 图（框偏左上、底栏 240 居中留缝）。
- **GREEN**：`cargo test -p game-core` 全绿（187 过含金样帧测 = **classic 零偏离**）；
  visual GREEN = after 图（框居中、底栏贴底随宽）。

## 3. 自验证据（headless 抓图对照，命令可复现）

复现命令（选中菜单=序章段脚本收尾进 mode3 窗口按 -7）：

```text
KEYS="200:-5,300:-5,400:-5,500:-5,560:-5,620:-5,680:-5,740:-5,800:-5,860:-5,920:-5,980:-5,1040:-5,1100:-7,1150:-7,1200:-7,1250:-7,1300:-7"
./target/release/xiyou --headless-shot <out.png> --frames 1400 --headless-newgame --preset modern --grid 14x13 --inject "$KEYS"   # modern
./target/release/xiyou --headless-shot <out.png> --frames 1400 --headless-newgame --inject "$KEYS"                            # classic
```

| 图 | 观察 | sha256 |
|---|---|---|
| before-modern（stash 本改动重编） | 框 x≈55..180（心≈120=legacy 锚，偏左）、y 心≈160 偏上；底栏 240 宽居中、两侧留缝、软键角标孤立 | ebf06c6f6636ece9193333194ad9b1e32e8fcc5badb07147f905369e2bd5a7d2 |
| after-modern | 框心≈208=layout center_x（居中）✓；高亮随框 ✓；底栏贴底随宽（边到边、软键角标并入底带）✓；悟空立绘原尺寸居左 ✓ | db1f3408418159779b91a7abae4ee274452985e769836b853a02c7508b280c2a |
| after-classic（240×320） | 原貌：框 (120,160) 居中、底栏 240 全宽、立绘原位——**零偏离视觉确认** | e615a53eb7e91421eb1c8be650c1829bf57ece62d5f77ca6d1ab4bfa5945753e |

金样：game-core 187 全绿（walk_camera/render golden 等帧测含在内）；fixture_frames
活跃 3 测绿（22 ignore = STAGE-B-001 开机时序夹具欠账，非本线）。

## 4. 写域与留痕说明

- **menu_family.rs**（task-24 写域）：+138 行 = 锚位域（InGameMenuBox/HudPanelAnchor +
  两公式 fn）+ TDD 测试 3 个。
- **scene.rs**（越出声明写域——接线必需，task-23 已完工 8955443 无冲突；4 hunk 全部
  为本任务留痕）：paint_menu_list 几何块（3068-3094）/高亮（3101）/文本彩虹箭头（3114-3125）、
  paint_hud frame 锚（6401-6406）。
- 本审查记录（docs/findings/reviews/）随 commit 入库。

## 5. 遗留/待裁定

1. **框体拉伸**：本次裁定=内容尺寸居中不拉伸；若用户观感要随宽，改
   `in_game_menu_box` 一处（v5 判据）+ 重出对照图。
2. **两套 HUD 锚一致性**：底栏（hud_panel_anchor 全宽）与顶栏（status_fill 全宽）
   已同为"随宽铺满"族；若后续顶栏改锚样式，本 fn 需同步。
3. fixture_frames 22 ignore 的开机时序重定标（STAGE-B-001 欠账）完成后，
   菜单画面夹具可入册帧级验收。
