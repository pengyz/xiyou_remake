# game-android —— Android 壳（shell/android/）

> 平台壳族第二实现（结构决策 2026-10-17：平台壳归 `shell/`，`shell/win`=Windows 壳）。
> 分层：游戏逻辑在 `game-core`（零 I/O），平台 trait 在 `game-platform`
> （`docs/spec/platform-contract.md` v1.0），共享壳实现在 `game-shell-common`
> （消费不拷贝——对抗审查 F-4 裁决）。本 crate = 四 trait 的 Android 实现 +
> 触控/键映射 + 生命周期映射 + NativeActivity 装配骨架。

## 0. 诚实边界（本环境无 Android SDK/NDK——不虚报可运行）

> 【2026-10-26 更正】本机已具备 SDK/NDK（`~/Android/Sdk`，NDK 27.1）并已**实机跑通**
> （Xiaomi 14 / HyperOS，Android 16，窗口 2536×1200）：开机→标题（触屏轮播 C 方案）→
> 读档/设置子页→序章（真 CJK 字形）均实机验证（截图+`jjxy-input` 日志）。**未通**：
> 序章/读条之后进游戏（壳缺 Boot→GameScene 交接，读条完回落标题）、音频（AAudio
> `openStream` 报 ILLEGAL_ARGUMENT ⇒ 静音降级）。下表为 2026-10-18 原始口径，保留不删。
>
> 【2026-10-26 更正②】进游戏已通：`runtime.rs` 双机调度 + `game_shell_common::handoff`
> 交接（win 壳同源）——实机走通 序章→读条→交接→剧情对话→地图行走（方向盘/点地寻路
> 拾钥匙）→游戏内菜单；宿主钉 `tests/flow.rs`（新游戏/存档-退出-读档/只点屏幕三条全
> 流程，真资产每拍 step+paint）。音频仍未通（同上）。
>
> 【2026-10-27 更正③】音频已通（BUG-027/028）：AAudio 设备 = `audio::StreamPcm` 状态机 × NDK 流（SHARED +
> PERFORMANCE_MODE_NONE），BGM 命令翻译 = shell-common `BgmController`（Runtime 每拍 + hideNotify 同步）。实机：探针
> `shell/android/examples/aaudio_probe.rs`（adb shell 运行、只写静音）阻塞写节拍 + idle 丢弃重启、游戏内标题曲播放 /
> 切后台即停（docs/bug-ledger.md BUG-027/028 实机复核注）。下表"⏳ AAudio 负 timeout 阻塞口径"一项已结：负/零超时
> = **不阻塞**（AOSP AudioStreamTrack.cpp:505 + 实机探针自旋实证），实现改正超时 1 s。未实机验证：输出设备切换
> （插拔耳机/连蓝牙）后的流重开（`StreamPcm::with_reopen`，宿主钉 + 源码路径推理）。
>
> 【2026-10-27 更正④】退出链已通（BUG-033）：下表"⚠ 骨架·未编译"行中的 `process::android`（ANativeActivity_finish）
> 已交叉编译并实机验证——主菜单选"退出" / 标题态 BACK → mode 22 退场动画 → `ANativeActivity_finish` → Activity 销毁后
> `_exit(0)`；Xiaomi 14 两条路径 logcat `exit chain` 全链 + ActivityManager `has died`、crash 缓冲为空
> （docs/bug-ledger.md BUG-033 实机复核注）。BACK **不消费**时的系统 finish 行为（下表 ⏳ 行）仍未测（壳恒消费）。

| 状态 | 内容 | 判定方式 |
|---|---|---|
| ✅ 宿主已验证 | 触控/硬件键→键码映射（含**折叠单键状态机**）、生命周期状态机、宿主桩契约语义、Storage 根路径注入接线、音频轨 AssetSource 装载、装配态自检 | `cargo test -p game-android` 45/45 全绿 |
| ✅ 宿主已验证 | workspace 构建零错 | `cargo build --workspace` |
| ⚠️ 骨架·未编译 | `process::android`（ANativeActivity_finish）、`audio::aaudio`（AAudio 全部 extern）、`asset::apk`（AAssetManager 全部 extern）、native.rs 的 TODO 函数体（事件环/ANativeWindow 呈现） | `#[cfg(target_os="android")]` 门控；extern 签名照 NDK 头文件手写（shell/win winmm/ALSA 直绑同式），**未经交叉编译/真机验证** |
| ⏳ 待实机复核升 A | BACK 不消费⇒系统 finish 的行为锚（现 B 级）；AAudio 负 timeout 阻塞口径；`internalDataPath`≈`getFilesDir`；Android 回调→MIDP 近似（分屏/画中画/锁屏/来电） | 实机/文档核对后回写本表 |

**验收口径**：本 crate 的"完成"= 宿主测试全绿 + 四 trait 实现齐（原生部分明确 TODO 边界）+
构建说明可复现。**APK 可运行不在本次验收面内**——构建配方（§5）本身未经验证。

## 1. 四 trait 的 Android 实现（契约 §5 映射表）

| trait | Android 实现 | 宿主桩（测试面） |
|---|---|---|
| `Storage` | `storage::AndroidStorage`：注入根路径（应用私有目录 `activity->internalDataPath`）薄接线 `game-shell-common::rms::FileRms` | 临时目录实测（`tests/storage_wiring.rs`） |
| `ProcessControl` | `process::android::AndroidProcessControl`：`ANativeActivity_finish`（= JNI `Activity.finish()`，本身**不杀进程**）；壳在 Activity 销毁（APP_CMD_DESTROY）后 `_exit(0)` 结束进程（BUG-033，native.rs `android_main` 尾）；退出请求 = 根界面返回（BUG-034）或菜单「退出」确认（BUG-035，不播 mode 22 退场动画） | `process::HostProcessControl`（幂等观测） |
| `AudioOut` | `audio::aaudio::AAudioPcm`：AAudio 阻塞 write（API 26+；EXCLUSIVE→SHARED 回退）。**AudioTrack 兜底已砍**（对抗审查 §3-1）：失败/旧系统 ⇒ `NullSink` 无声降级，minSdk 26 | `audio::CountingPcm`（契约形状：全量写/Err 不可恢复/idle 丢弃） |
| `AssetSource` | `asset::apk::ApkAssetSource`：`AAssetManager_open` + 全量读（**不是 std::fs**；`activity->assetManager` 免 JNI） | `asset::DirAssetSource`（目录注入 std::fs） |

- 键模型（`RMS_STORES`/`rms_key`/`rms_store_name`）与文件语义**单源**在 game-platform 契约 /
  game-shell-common（本 crate 只测包装接线——对抗审查判据：主测试单源，防测试双份漂移）。
- 音频轨 `track{i}.ogg` 走 `AssetSource::read`（**B2 第五缝 Android 口径**：不得沿用路径
  注入；Windows 的 `--audio` 目录 = win 专用）。装载：`audio::load_tracks_from_assets`
  （缺失/损坏静音降级，同 shell/win `start()` 口径；BUG-010 死资源不解码）。

## 2. Storage ↔ Java RMS 等价性（Android 落地声明）

**引用不另立口径**：完整等价性论证（Java 三式行锚 a.java:4457-4459/8978-8980/9260-9262
读首条枚举记录；8928-8954/9027-9052/9190-9244 写整店替换；边角 a-d + edge d）见
`docs/spec/platform-contract.md` §1.3。Android 落地与 Windows 同构：

- 根路径 = 应用私有目录（NativeActivity `internalDataPath` ≈ `Context.getFilesDir()`），
  **宿主注入**（`docs/knowledge/decision_save-path-shell-injection.md`）——游戏层零路径；
- 平铺 `{店}.bin` + 写-if-changed + upsert + 域外键丢弃 + 吞错 best-effort = `FileRms` 单源；
- 平台差（登记非 bug）：文件查找跟随宿主 FS 原生语义（Android ext4 区分大小写 /
  Windows 不区分），契约 §5 脚注不强行归一。

## 3. 输入映射（触控/硬件键 → 游戏键码）

键码面（`docs/spec/key-code-contract.md` §4，与 shell/win `key_code()` 同表）：
`-1` 上 / `-2` 下 / `-3` 左 / `-4` 右 / `-5` 探索确认 / `-6` 确认使用（左软）/ `-7` 返回退出（右软）。

| 来源 | 映射 | 备注 |
|---|---|---|
| 虚拟方向盘（左下 overlay） | 主导轴 4 向 → -1..-4 | 死区 r_inner + 换向迟滞 r_switch 防抖；拖动换向 = Release 旧 + Press 新（保序） |
| 软键三钮（右下 overlay） | -6 / -5 / -7 | 按钮捕获语义：按下后漂移不换键，Up/Cancel 才释放 |
| KEYCODE_BACK | **根界面（开机 logo / 主菜单 / 退场动画）= 立即退出**；其余 **-7**；恒吞默认 | 根界面 = Nokia 红键语义（`CMidlet.destroyApp` 为空，跳过 logo 退场动画不丢状态；BUG-034）。【2026-10-27 更正】BUG-033 版"标题态 = 选中「退出」项 + 确认 -5"走 mode 22 logo 动画，用户实机形似重播片头、动画期返回无效，已弃。其余态 -7 = 子页返回主菜单 / 序章跳过 / 游戏内菜单；不吞 ⇒ 系统 finish Activity ⇒ 绕过 m_000 退出链（BUG-020）；repeat 丢键仍吞 |
| KEYCODE_DPAD_* | -1..-4 | repeat 丢弃（gotcha_os-key-repeat-is-not-keypressed） |
| CENTER/ENTER/SPACE | -5 | 与 win Enter\|Space 同表 |
| KEYCODE_MENU | -6 | 若设备有 |
| 音量键 | 不消费 | 交系统（用户设备操作，非游戏可听行为） |

**折叠单键状态机（实现硬门，对抗审查 v2 复审口径）**：多点触控折叠进**单一**键状态机，
禁 per-role/per-pointer held——press = `key_value = key_held = code` 覆盖式取末键
（scene.rs:4270-4274），release = 只清 `key_held=0` 不认键（:4276-4279）。按住方向盘(-1)
再按软键(-5) ⇒ 折叠为 `-5`（行走即停）；松任一持键触点 ⇒ `key_held=0` 且**行走不自动
恢复**（"松开软键恢复行走"= 发明行为 = 禁）。隐藏期输入不投递（Visibility 门控），但
BACK 仍吞默认。

## 4. 生命周期映射（onPause/onResume → hideNotify/showNotify）

`lifecycle::Visibility` = `f_bool_01` latch 复刻（a.java:4495-4513；CMidlet.java:19
startApp 后恒 shown）。隐藏消费全在 game-core（`hide_notify()`：mode3→4 弹菜单 +
closeAudioPlayer + anyKeyPlayed=true），壳只决定何时调；恢复不续播（等后续首键的
keyPressed 续播边沿，a.java:4665-4674，game-core 内执行）。

**近似边界登记（对抗审查 F-7）**：

| Android 场景 | 回调 | 映射 | 近似性 |
|---|---|---|---|
| 退后台/锁屏/来电 | onPause | ⇒ Hide | ≈完全遮挡，吻合 |
| 返回前台/解锁/通话结束 | onResume | ⇒ Show | 吻合 |
| 分屏/自由窗口「可见但失焦」 | onTopResumedActivityChanged(false) | **不发** Notify | 同 Windows 失焦不映射（用户决策 2026-10-15） |
| 画中画（可见非遮挡） | onPause(isInPictureInPictureMode=true) | **不发** Hide | 按「完全遮挡才 hide」锚（对抗审查 S1） |

**音频焦点不处理（对抗审查 F-6 砍除）**：原版 J2ME 无音频焦点概念（a.java 音频入口仅
m_139/closeAudioPlayer，:10158-10215）；焦点被抢期间的发声属平台近似边界，**不得**
引入原版没有的可听行为（AGENTS §1）。如日后需要 opt-in ⇒ decision 登记 + 用户确认
（另立任务，不在此悄悄加）。

## 5. 构建说明（cargo-ndk / gradle 配方——⚠️ 逐条未验证）

> 本环境无 Android SDK/NDK。以下配方按 cargo-ndk/AGP 惯例书写，**未跑通过**；
> 首次实机构建时逐条核对并回写 §0 矩阵。

```bash
# 1) 工具链
rustup target add aarch64-linux-android
cargo install cargo-ndk          # NDK 需 ANDROID_NDK_HOME 或 sdkmanager 安装

# 2) 产出 .so（cdylib；输出到 gradle 的 jniLibs）
#    【2026-10-26 实机核对】必须 -P 26：cargo-ndk 默认 API 级找不到 libaaudio
#    （ld.lld: unable to find library -laaudio）
cargo ndk -t arm64-v8a -P 26 -o android/app/src/main/jniLibs \
    build --release -p game-android

# 3) 打包 + 装机（本机已验证：gradle 8.5 离线 + AGP 8.2；Xiaomi 14 = adb -s c362a15b）
ANDROID_HOME=$HOME/Android/Sdk /tmp/gradle-8.5/bin/gradle -p android assembleDebug --offline
adb -s <serial> install -r android/app/build/outputs/apk/debug/app-debug.apk
adb -s <serial> shell am start -n com.jjxy.game/android.app.NativeActivity
adb -s <serial> logcat -s jjxy-input:D      # 输入/拍/字体/BGM 诊断日志
```

gradle 要点（`android/app/build.gradle`）：

```gradle
android {
    defaultConfig {
        minSdk 26                 // AAudio（AudioTrack 兜底已砍 ⇒ 统一 26）
        // NativeActivity 路线：manifest 里 activity 指 android.app.lib_name
    }
    sourceSets.main {
        // 资产：assets/raw 产物进 APK assets/（与 shell/win 装载同口径；
        // 音频轨 track{i}.ogg 亦放 assets/——B2：Android 走 AssetSource）
        assets.srcDirs = ['../assets']
        jniLibs.srcDirs = ['src/main/jniLibs']
    }
}
```

```xml
<!-- AndroidManifest.xml：NativeActivity（免 Java 代码；lib_name = cdylib 名） -->
<activity android:name="android.app.NativeActivity"
          android:app.lib_name="game_android" ... />
```

- **NativeActivity 路线（首选）**：`activity->assetManager`/`internalDataPath` 免 JNI。
- **Java Activity + JNI 路线（备选）**：AssetManager 经 `Context.getAssetManager()` JNI 取，
  finish 经 `Activity.finish()` JNI 调（`process::android` 注释含直连路径）。
- 运行时依赖：libaaudio.so（API 26+ 系统库）、libandroid.so；无第三方 so。

## 6. TDD 记录（红→绿；A 协议：红不进历史，RED 证据三件入 commit 正文）

| 单元 | 测试 | RED（取证方式） | GREEN | commit |
|---|---|---|---|---|
| 1 input 映射（含折叠硬门） | tests/input_mapping.rs（16） | E0432（树内取证，2026-10-18 早期窗口） | 16/16 | f2f95e0 |
| 2 lifecycle 状态机 | tests/lifecycle.rs（9） | E0432（树内取证） | 9/9 | b622d0e |
| 3 宿主桩三件+原生骨架 | tests/stubs_contract.rs（11） | 3×E0432（树内取证） | 11/11 | 41f4b7e |
| 3a 竞态修复（flaky） | — | 闸上实录（共用临时目录互删） | 3 轮连绿 | 0491a74 |
| 4 Storage 消费接线 | tests/storage_wiring.rs（4） | E0432（**树外 temp 法**） | 4/4 | ddba831 |
| 5 音频轨 AssetSource 装载 | tests/track_loading.rs（4） | E0432（**树外 temp 法**） | 4/4 | 6370b03 |
| 6 native 骨架 + README | src/native.rs 自检（1） | —（装配骨架非红绿单元，测试随实现同 commit 入树即绿） | 1/1 | 本 commit |

RED 复现指令随各 commit 正文（单元 4/5 起改**树外取证**：`git archive` 导出父树到
temp + 仅应用测试文件 + 运行——`docs/knowledge/pattern_tdd-red-evidence-with-gates`
+ `gotcha_cargo-mtime-stale-rlib-false-red` 的 temp 法，共享树零红）。

## 7. 其他纪律落位

- **过渡双壳对照 golden（S3）**：game-shell-common 单源已达成（解码/FileRms 主测试均在
  shell/common 测试面）⇒ 双跑 golden **不再建立**（F-4 过渡条款自然失效，无永久债）。
- **测试计数基线（F-8）**：本 crate 新增测试二进制
  （input_mapping/lifecycle/stubs_contract/storage_wiring/track_loading）待
  `gates/cli.py ratchet-update` 显式更新基线 + commit 说明——**归集成收口**（lead
  task-6），本 crate 不擅动 `data/status/baselines`。
- 已知小重复（提交对抗审查裁）：`asset::DirAssetSource`（宿主桩）与 shell/win
  `DirAssets`（win 侧参考实现）同为"目录+std::fs+rel 校验"薄层——如需并入
  game-shell-common 单源由 lead 裁决（写范围外）。

## 8. 遗留 TODO 边界（实现期/实机期）

1. native.rs TODO 函数体：事件环（native_app_glue）、ANativeWindow 呈现（等比信箱
   blit）、Viewport 反映射、75ms 拍环与 BgmController 轮询装配；
2. §0 ⏳ 行全部"实机复核升 A"项（BACK 锚 / AAudio 阻塞口径 / 回调近似表）；
3. AAudio 长稳（欠载/切轨滞后）实测（对照 BUG-019 教训）；
4. 字体缝（接缝 7）：Android 系统字体注入方案未在本次范围（game-core 占位字形可跑）。
