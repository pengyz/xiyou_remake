# P0 地形半绘修复对照结论（dec3360·modern 换层全绘面）

> 验证人：boss-evidence（独立验证）。对象：**dec3360**（根因=换层 load_floor_data
> 重建视图不复跑 apply_layout 换轨；修复=view 重建后强制 set_layout_dims）。
> 方法：GUI 复现工具对照（/tmp/save-f3 直入+inject）+ **scratch worktree 判据移植
> pre/post 对拍**（真修点独立复现）+ classic 对照 + 专色判据复核。零主树扰动。

## 判定表

| # | 要点 | 判定 | 证据 |
|---|---|---|---|
| ① modern floor3 地形全绘 | ✅ **成立** | f3-post.png（416×416）：26 格地形全列绘出、右下空白消失、天空带/底栏 HUD（500/200/30）正常；象限覆盖 右/左比=1.01（对称满绘） |
| ② pre/post 真修点对照 | ✅ **独立复现红→绿** | scratch worktree（9e04c84=dec3360^）移植判据测试 → **红：「modern 换层后右半须绘出 tile（现 0/110 专色）——半绘 artifact」**（与声明变异 RED 形逐字合）；post 树双判据全绿（modern_floor_switch_tiles_cover_full_view + modern_layout_range_covers_full_view 各 1 passed） |
| ③ classic 对照全绘 | ✅ **成立** | f3-classic4.png（240×320）：3 层地形全绘（SoundAsk 需**右软键 -7** 过屏——"软键 2"实测=键码 2 无效、-7 有效）；HUD 500/200/30 同态 |
| ④ 专色判据复核 | ✅ **判据健全** | 0xFF00FF00 纯色 tileset + x300..410@y200 计数 >20/110——专色防背板误计 ✓；单行采样对 tile 全 1 合成图充分；game_view 单元判据（cam_x+col_end×16≥416 覆盖式断言）与场景级互补 ✓ |

## 关键澄清（复现工具边界·供 lead 收档）

- **GUI 复现工具（save 直入）在 pre 上不红**（我方实证：f3-pre.png 与 f3-post.png
  **逐字节等同** f113a312，含换层窗逐 2 拍采样 206-214 全程全绘）——根因=
  **壳侧入口 apply_layout（shell/win/src/main.rs:921）覆盖了 load_floor_from_save
  路径**；真正坏路径=游戏内换层 `load_floor`（scene.rs:1100 一族，无壳补轨）。
- ⇒ 该复现工具的定位=**post 验收面**（全绘确认 ✓）；pre 红面须走 TDD 判据路径
  （真修点 load_floor_data 直测）——本审 scratch 移植判据达成独立复现。
- **game-core 层 vs 壳层边界**：save-load 路径在 game-core 面仍裸奔（壳入口外的
  调用方无补轨）——porter 在途红测 `modern_save_load_tiles_cover_full_view`
  （scene.rs:7104，当前 TDD 红相）正是补该缺口，与本审发现同源互证（其完工后
  save/load 面判据齐）。

## 复现命令

```bash
# post 全绘面（GUI）
./target/release/xiyou --preset modern --grid 13,13 --mute --save /tmp/save-f3 \
  --inject "60:2,150:-2,155:-5,170:-5,190:-5" --headless-shot f3-post.png --frames 260
# classic 对照（SoundAsk=右软键 -7 过屏）
./target/release/xiyou --preset classic --mute --save /tmp/save-f3 \
  --inject "<-7 撒点>,380:-2,390:-5,410:-5,430:-5" --headless-shot f3-classic.png --frames 520
# pre 红独立复现（scratch worktree，零主树）
git worktree add /tmp/f3pre-wt dec3360^ && cd /tmp/f3pre-wt
# 移植 modern_floor_switch_tiles_cover_full_view + synthetic_sptarm 进 scene.rs 测试 mod 后：
cargo test --release -p game-core --lib modern_floor_switch_tiles_cover_full_view
# ⇒ 红："现 0/110 专色——半绘 artifact"
```

**总结论：dec3360 修复达成——modern 换层地形全绘（GUI+真修点判据双证）、
classic 零偏离、专色判据健全。附边界澄清一条（save 面壳层豁免/在途判据补缺）。**
