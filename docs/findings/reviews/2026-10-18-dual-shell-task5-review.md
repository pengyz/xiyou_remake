# 对抗审查：task-5 双壳总审（shell/win 迁移 + shell/android 骨架）

> 审查人：adversary。对象：task-3（shell-port，四缝迁移+shell/common 抽取）与
> task-4（android-shell，game-android 骨架）交付面。子审查：2026-10-18-shell-win-
> migration-implementation-review.md（win 逐项）+ 本篇 android 面与双壳总核。
> 手段：GREEN 独立复跑、RED 树外复现、README 矩阵↔代码逐条对、断言字节级比对、grep 亲验。

## 判定：**通过**（双壳）——android 面 3 项裁定随附（死区 Up/DirAssetSource 并 common/字缝缺口登记）

## 1. 行为零漂移总核 —— ✅

- **win 面**：38 锚 = 29（game-desktop）+ 13（game-shell-common）计数精确闭合；
  断言 145/145 条唯一行 UTF-8 全文包含比对 **missing=0**（字节级零改动，豁免未动用、
  键序字面量原样、game-platform 自 efb29bb 零改动）。
- **android 面**：新 crate 零接触 game-core 行为面（构造上零漂移）；输入折叠单键状态机
  逐测锚 scene.rs:4270-4279（press 覆盖式置 key_value=key_held、release 只清 held
  不认键），`fold_*`/`drag_*`/`event_order_preserved`/`up_emits_release_only_for_
  pointers_that_pressed` 全对锚。
- GREEN 独立复现：game-android **45/45**（1+16+9+4+11+4）、game-shell-common
  **13/13**、game-desktop **29/29**、`cargo build --workspace` **零错**（EXIT=0）。

## 2. Android 真实性边界 —— ✅ 不虚报（README §0 矩阵 ↔ 代码逐条吻合）

| 矩阵行 | 亲验 |
|---|---|
| ✅ 宿主 45 测全绿 + build 零错 | 我独立复跑同值 ✓ |
| ⚠️ 骨架未编译（process::android / audio::aaudio / asset::apk extern + native TODO） | cfg 门控实存（asset.rs:57 / audio.rs:92 / process.rs:48，`#[cfg(target_os="android")]`），宿主编译不触 ✓；"extern 签名照 NDK 头文件手写、未经交叉编译/真机验证"如实标注 ✓ |
| ⏳ 待实机升 A（BACK 行为锚/AAudio timeout/internalDataPath≈getFilesDir/回调近似） | 与我 v2 核验清单第 3 条对账——**未升 A 并如实标注**，比虚报升 A 正确；BACK 侧代码按"恒 consumed"实现+测试锁（含 hidden 仍吞、repeat 丢键仍吞） |
| 验收口径（APK 可运行不在验收面） | = v2 §0 已批准口径 ✓ |

## 3. TDD 纪律 —— ✅

- 单元 1-5 均携 RED 三件 + Evidence/Confidence；**单元 5 我树外独立复现成功**：
  temp-archive 父树 5f541f0 + 仅叠 tests/track_loading.rs ⇒ `E0432: unresolved import
  game_android::audio::load_tracks_from_assets` ✓。
- 取证法演进如实记录（单元 1-3 树内 → 单元 4/5 树外 temp 法）；两次事故自报：
  单元 3 RED 窗口占红连坐（改法后不再）与 stubs_contract 临时目录 flaky 竞态
  （0491a74 修复、3 轮连绿）——诚实自报加分项。

## 4. 平台差异覆盖（八接缝 × 双壳终表）

| # | 接缝 | shell/win | shell/android | 状态 |
|---|---|---|---|---|
| 1 | 资产 | DirAssets（--assets 注入，std::fs） | ApkAssetSource（AAssetManager，非 std::fs） | ✅ |
| 2 | 存档 | FileRms（common 单源，exe 目录\save） | AndroidStorage→FileRms（internalDataPath 注入） | ✅ |
| 3 | 退出 | WinitExit（el.exit()，2→1） | AndroidProcessControl（ANativeActivity_finish 不杀进程） | ✅ |
| 4 | 音频 | winmm/ALSA PcmDevice | AAudioPcm（API26+，失败⇒NullSink；AudioTrack 兜底已砍=minSdk 26 单值） | ✅ |
| 5 | 输入 | winit→key_code | 触控 overlay+硬件键（折叠单键状态机、BACK 吞默认） | ✅ |
| 6 | 窗口/呈现 | winit + Pacer/Viewport | native.rs 骨架（⚠️ TODO） | ⚠️ 骨架（范围内） |
| 7 | 字体 | font_inject（系统字体发现） | **未覆盖**（native TODO 域内） | ⚠️ 已知缺口（见 §6-3） |
| 8 | 生命周期 | Visibility latch（遮挡/最小化） | Visibility latch（onPause/onResume）+ 近似表（PiP 走"可见非遮挡"锚） | ✅ |

## 5. 三项裁定

### 5-1【裁定·死区 Up 语义】android 的理解**正确**，无需改代码

【2026-10-18 更正】我 v2 口径"任一触点 Up ⇒ key_held=0"措辞不精确，正解=**任一持键
触点** Up。依据：Java Canvas keyReleased 仅为曾按下的键发生（事件配对语义）——死区指
（从未产生键）Up 发 Release = 发明无前件的 keyReleased，且会误清他指持有的 held。
其"死区指 Up 无事件"（input.rs:24,258）+ `up_emits_release_only_for_pointers_that_
pressed` 锁定 = 保真正确。

### 5-2【裁定·DirAssetSource/DirAssets】**并入 shell/common**（生产面重复）

android `asset::DirAssetSource`（src/ 公有面，asset.rs:36-52）与 win `DirAssets`
（main.rs:74）同为"目录注入 + std::fs + rel 校验"生产适配器 = 双拷贝判例
（gotcha_dual-porting-surfaces-drift）+ lead 给定判据（测试侧 stub 可留、生产面重复
则并 common）命中 ⇒ 并。执行：`game_shell_common` 提供 `DirAssets` 单源，win 与
android 均消费（android 可留类型别名）；**约束**：win 侧 panic 文案逐字锚
（panic_text_* 三测）必须继续绿（合并不得改 Err 文本形状）；执行归 task-6 集成收口
或 shell-port 下一小步，动前知会。

### 5-3【登记·接缝 7 字体】android 骨架阶段已知缺口

字体面（win font_inject 的系统字体发现）在 android 侧属 native/呈现 TODO 域，任务
范围（四 trait+输入+生命周期+骨架）本不含——**登记为已知缺口**，随呈现骨架任务补
（Android 字体走系统 Typeface/asset 字体，无 std::fs 发现路径，届时按 AssetSource 或
系统 API 口径设计）。

## 6. 备忘（不卡闸）

1. test-ratchet 基线 ratchet-update（新增 5 个测试二进制）→ task-6（约定）。
2. RED 窗口连坐观察项（共享总闸全 workspace）——android 已改树外法自发缓解，复发再立 knowledge。
3. README §7 既有 DirAssetSource 并入提示 = 与 5-2 裁定同向，销账即办。
