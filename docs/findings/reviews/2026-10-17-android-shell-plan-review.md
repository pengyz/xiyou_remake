# 对抗审查：android-shell「Android shell 方案 v1」

> 审查人：adversary（对抗性审查岗）。审查对象：android-shell 2026-10-17 提交的
> 「Android shell 方案 v1」（写范围 shell/android/**，crate game-android）。
> 审查方式：逐条声称对照代码实况与 Java 真值（grep/read 亲验）。
> 纪律依据：AGENTS.md §1（保真只增不减）/§3（证据分级）/§5（验证不打扰、白名单需用户确认）/§9。

## 判定：**驳回（v1 修订后重审）**

方案骨架方向正确、诚实声明合格，但含 **1 处保真红线越界**（音频焦点）、
**1 处会破坏 BUG-020 退出链的实质缺口**（BACK 键）、**1 处自相矛盾**（"照搬 audio.rs"
撞 APK assets 无 std::fs）、**1 处契约分叉**（与 shell-port 各自猜出的签名已不一致）。
以下逐节 PASS/FAIL。

## 0. 逐节判定

| 节 | 判定 | 理由 |
|---|---|---|
| §0 边界与诚实声明 | **PASS** | "未编译验证、不虚报可运行"诚实；[契约待定] 标注合规 |
| §1 crate 结构 | **PASS*** | glob 注册说法对（gotcha_cargo-workspace-glob-requires-manifest 属实）；*lewton 必须锁 `=0.10.2` 与 shell/win/Cargo.toml 同版，防双解码器漂移 |
| §2① Storage | **FAIL-条件** | 平铺单 blob 的 Java 等价性论证缺失（详见 F-2）；键模型 `{name}@1` 跨 game-core（paint.rs:638）不得擅改 |
| §2② ProcessControl | **PASS** | ANativeActivity_finish/JNI 双路径描述正确；finish 线程约束已标 TODO |
| §2③ AudioOut | **FAIL-条件** | "结构照搬 shell/win audio.rs"不成立：被照搬代码含 std::fs 资产读取（audio.rs:629-630,665-666），APK assets 无 std::fs（F-3）；且照搬=双拷贝漂移（F-4） |
| §2④ AssetSource | **PASS*** | AAssetManager 路线正确（NativeActivity `activity->assetManager` 免 JNI）；*签名已与 shell-port 暂定形状分歧（Option vs Result）——须 task-2 唯一裁决（F-5） |
| §3 触控映射 | **FAIL** | 硬件键整体缺席：BACK 不映射 ⇒ 系统默认 finish Activity ⇒ 绕过 m_000 退出链（F-1）；pointer 角色分配规则模糊（次要） |
| §4 生命周期 | **FAIL** | 音频焦点策略引入原版没有的可听行为，踩 AGENTS §1 红线（F-6）；onPause≈hide 近似与 Windows"完全遮挡"语义的差异未登记（F-7） |
| §5 呈现骨架 | **PASS** | TODO 边界诚实 |
| §6 构建说明 | **PASS*** | *minSdk "26/21" 自相矛盾，须统一（若砍 AudioTrack 兜底则 minSdk 26） |
| §7 TDD 顺序 | **PASS*** | 顺序合理；*真实性以实现期 git log 复核（测试先行独立 commit，红→绿可见） |
| §8 开放问题 | 见 §3 逐条回答 | |

## 1. FAIL 项与整改清单（可执行）

### F-1【红线级缺口】BACK 键缺映射会击穿 BUG-020 退出链

- **机制**：NativeActivity 收 KEYCODE_BACK；不消费 ⇒ 系统默认结束 Activity（进程终止）。
  而本项目退出契约是：游戏层发 `CMidlet.m_000` 信号（BootMachine::finished，
  crates/game-core/src/paint.rs:585，mode 22 退场动画 70 拍后）→ shell 消费才退出
  （BUG-020；shell/win/src/main.rs:1071-1074）。BACK 直接 finish = 不等信号、
  不走菜单确认链（-7 的语义是"返回/退出"经游戏态消费，key-code-contract §4），
  退出时机与路径全变；玩中被 BACK 打断还可能丢未保存进度。
- **整改**：input.rs 补硬件键映射——KEYCODE_BACK→-7 **且吞掉默认行为**（消费返回真）；
  KEYCODE_DPAD_*→-1..-4；音量键交系统（平台行为）；软菜单键若有→-6。TDD 用例：
  "BACK 投递 route_key(-7) 且事件标记 consumed"。

### F-2【保真论证缺失】Storage 平铺模型的 Java 等价性必须书面成立

- **Java 真值**（与 shell-port 审查同锚）：RMS 按名开不列店（a.java:4456/8928/8977/9193/9259）；
  读=枚举首条记录（a.java:4457-4459/8978-8980/9260-9262）；写=SKY_WAR setRecord(1)/addRecord
  （a.java:8929-8954）、MOT_IF/MOT_L=deleteRecordStoreQuietly+addRecord（a.java:9027,9052/9190,9244）；
  读失败=删店+默认+重写（a.java:4475-4478）。
- **缺口**：方案只说"文件格式逐项同构 Rms"，同构的是**壳**不是**真值**；等价性
  （每店单记录 ⇒ delete+add ≡ 覆盖写；delete 面可省）必须写成书面论证并给反例边界
  （多记录店即反例）。键名 `{name}@1`、`split('@')` 截断（shell/win/src/main.rs:175-186）
  同此。
- **整改**：README/方案补等价性小节（锚到上述 a.java 行）；`impl game_platform::Storage`
  等 task-2 成稿后逐条对账，形状以契约为唯一裁决。

### F-3【自相矛盾】"照搬 audio.rs"与 APK assets 冲突（第五接缝）

- 被照搬的代码含**路径型资产读取**：track_path（shell/win/src/audio.rs:617-620）、
  `.exists()`（:629-630）、`std::fs::read`（:665-666）——APK assets 无文件路径。
- **整改**：TrackStore 装载改走 AssetSource（`read("track{i}.ogg")`）；把"音频轨资产供给"
  明确列入 §2④ 消费面。这与 shell-port 审查 F1 是同一第五缝，两壳必须同口径。

### F-4【漂移风险】双拷贝未裁决（gotcha_dual-porting-surfaces-drift 前科在案）

- "照搬"= Mixer/TrackStore/lewton decode/audio_loop 两份拷贝；Storage 文件语义成第三份
  （game-core save.rs 序列化 + shell/win Rms + shell/android storage）。
- **整改**（需 lead 裁决写范围）：跨壳共享部分抽公共 crate（建议 shell/common；
  AGENTS §2 限定 game-platform 只放 trait 与类型，实现不得上移进去）；若暂不抽，
  必须写明同步责任 + 双壳对照测试（同一夹具双跑 golden），并把抽公共 crate 立为
  显式待办。

### F-5【契约分叉】两壳暂定签名已不一致，倒逼契约的风险成真

- android：`AssetSource::read(name)->Option<Vec<u8>>` + `exists`；
  shell-port：`read(rel)->Result<Vec<u8>,String>`。Storage 侧同样（load_all/persist vs 未知）。
- **整改**：两壳实现前以 task-2 成稿为唯一裁决；分歧点以书面形式交 platform-contract
  （我已把问询升级为裁定要求，见 shell-win 迁移方案审查 F2）；禁止任一壳先落地自定义
  形状再倒逼契约。

### F-6【保真红线】音频焦点策略（§4）= 原版没有的行为

- 原版 J2ME 无音频焦点概念（a.java 全文无 focus/Listener 相关；音频入口仅
  m_139/closeAudioPlayer，a.java:10158-10215）。"焦点丢→壳层静音、焦点回→从头 Play"
  是**可听行为**的新增，AGENTS §1 明文："原版没有的行为（含修复）禁止引入"。
  焦点回"从头 Play"也不是 JSR-135 重建语义的复刻——原版重建语义只挂在
  hideNotify→keyPressed 续播链（a.java:4509-4510/4665-4674），焦点事件不触发它。
- **整改**（二选一）：
  1. v1 砍除焦点处理（推荐——Android 全面屏手势下 onPause 已覆盖绝大多数退后台场景，
     与原版 hideNotify 语义对齐）；或
  2. 默认关闭的 opt-in + decision 文档登记为"平台层显式偏差" + **用户确认**
     （同 AGENTS §5 视觉白名单变更的先例），并给出证明：不触碰 anyKeyPlayed/mode/BgmState，
     只动 PcmDevice 输出层。
- §8-Q2 的直接回答：**现设计不够保守**。"纯壳层音量策略、游戏态零改动"是必要条件
  不是充分条件——可听行为本身已是保真面（差分验证跑的是 trace，玩家听到的是音频）。

### F-7【语义近似未登记】onPause≈hideNotify 的平台差

- Windows 侧锚是"完全遮挡/最小化才 hide、失焦不映射"（shell/win/src/main.rs:17-20,1056-1057；
  lifecycle.rs 模块注释 X11 FullyObscured）。Android onPause 不等于完全遮挡：
  分屏/自由窗口（API 24+）下"可见但失焦"只掉 onTopResumedActivityChanged，不掉 onPause
  （此点反而与 Windows 语义接近，是好的）；但画中画、锁屏亮屏等路径各异。
- **整改**：README 映射表逐条登记"Android 回调 → MIDP 语义"的近似边界
  （哪些场景会多发/漏发 hideNotify），至少覆盖：分屏、画中画、锁屏、来电。
  TDD 已有"pause/resume×3 幂等"（好），补"onTopResumedActivityChanged(false) 不发 Hide"用例。

### F-8【回归面】新 crate 进 workspace 改变测试计数基线

- `cargo test --workspace` 面扩大；281 测试计数基线（data/status/baselines）会被顶高。
  AGENTS §2：基线只能在确认无回退后更新且写 commit 说明。整改：game-android 测试
  入基线时显式更新+说明，不得静默；gates 8 项全绿作为 §7 验收硬项（方案已有，保留）。

### F-9【次要】杂项

- §3 pointer 角色"首触占方向盘"语义模糊（首触是软键时方向盘是否可同时用？）——
  改**按区域定角色**（触点落在方向盘区即方向盘、软键区即软键），TDD 补"软键+方向盘
  同时按"用例（原版实体键支持同时按：行走 keyHeldCode + 确认 keyValue 并存，
  key-code-contract §1）。
- §6 minSdk "26/21" 二选一写死。
- "55 层 maplv+sprite"数字勿写死：assets/raw 实有 maplv0..maplv54 共 55 个文件，但
  装载循环只取 1..=54（shell/win/src/main.rs:130 `.skip(1).take(54)`），maplv0 未载——
  main.rs:118 注释"55 层全量装载"与代码有 1 层出入（**C 级**：哪边为真未考证，可能是
  maplv0 死资源；方案引用时说"与 shell/win 装载同口径"即可）。

## 2. 已亲验为真的声称（A 级）

| 方案声称 | 实况锚 | 结论 |
|---|---|---|
| 键码面 -1..-7 与 key-code-contract 同表 | docs/spec/key-code-contract.md §4 | ✅ 真 |
| shell/win 鼠标映射：底部 30% 左 1/3=-6、右 1/3=-7、其余 -5 | shell/win/src/main.rs:13 | ✅ 真 |
| 隐藏期输入丢弃（Visibility.shown 门控） | shell/win/src/lifecycle.rs:28 + main.rs route_key | ✅ 真 |
| OS 键重复丢弃 gotcha 存在 | docs/knowledge/gotcha_os-key-repeat-is-not-keypressed.md | ✅ 真 |
| f_bool_01 latch 复刻（a.java:4495-4513） | shell/win/src/lifecycle.rs:16-28、main.rs:578-580 | ✅ 真 |
| hideNotify 消费链同构（mode3→4 弹菜单+closeAudioPlayer+anyKeyPlayed，菜单先音频后） | main.rs:581-612 → scene.rs:2821-2827（GameScene::hide_notify 全臂） | ✅ 真 |
| boot_key_audio_done 补做次序 | main.rs:448-450,584-594,818-819 | ✅ 真 |
| PcmDevice 契约 write=阻塞节拍/idle=丢缓冲 | shell/win/src/audio.rs:214-219 | ✅ 真 |
| Storage 键域 8 键全载（BUG-018 教训） | shell/win/src/main.rs:172-178 | ✅ 真 |
| workspace glob 自动纳入 | 根 Cargo.toml members=["crates/*","shell/*"]（ba14d4e） | ✅ 真 |

⚠️ 引用注意：game-core/src/audio.rs:215-216 注记"hideNotify 整体尚未接入任何宿主路径"
是**陈旧注释**（已被 shell/win 生产接线推翻：main.rs:1067-1068 每拍轮询 set_minimized →
hide_notify）。方案对此的描述是对的、注释是错的（另见 git 审计记录 2026-10-17）。

## 3. 对 §8 五问的直接回答

1. **砍 AudioTrack JNI 兜底**——同意砍。理由：本环境无法编译验证、JNI 骨架=第二个
   验证面、双路径=双漂移面；API<26 走 NullSink 诚实降级（与 shell/win
   start_without_assets_degrades_silently 同式，audio.rs:632-637）；minSdk 统一 26。
   日后真机需求另立任务（届时有验证设备，不是纸上骨架）。
2. **音频焦点**——不够保守，见 F-6：v1 砍或 opt-in+登记+用户确认。
3. **"桩+本地 seam 先行、trait impl 最后适配"**——应固定为策略，再加一条：
   形状以 task-2 成稿为唯一裁决，禁止两壳签名分叉倒逼契约（F-5）。
4. **cdylib+rlib 双 crate-type**——无实质影响：cargo test 走 rlib、cdylib 是 Android
   装载所需；host 侧多构建一个动态产物而已。保留。
5. **Viewport/overlay 归属**——不急上移（rule of three：win 鼠标反映射 vs android 虚拟
   手柄，几何不同）；真正需 lead 裁决的是 F-4 共享代码归属（shell/common?）与
   F-3 第五缝（AssetSource 供 track 字节，两壳同口径）。

## 4. 证据分级与方法声明

- §0/§2 表内全部为 A 级（行号锚本次亲验）；F-1 BACK 默认行为为平台机制（Android
  文档行为），标 **B 级**（可复现推理：未消费 BACK ⇒ 系统结束 Activity）；
  F-9 的"55 层"数字出入标 **C 级**（未考证哪边为真）。
- 假设区：F-1 的具体 finish 行为依赖 native_app_glue 版本与 Activity 派生方式，
  实现期需真机/文档复核后升 A 级。
- 未验证项：全部 NDK/JNI/AAudio 路径（方案已自认）；我亦无法在本环境验证——
  验收只到"宿主测试全绿 + README 验证状态矩阵如实标注"。

**重审触发**：android-shell 按 F-1..F-9 出 v2（或书面接受/反驳逐条），我复审后给新判定。
