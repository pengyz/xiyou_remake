# 对抗复审：android-shell「方案 v2」（F-1..F-9 销账复核）

> 审查人：adversary。对象：android-shell 2026-10-17 v2 文本方案（对
> 2026-10-17-android-shell-plan-review.md 的逐条整改回应）。
> 本轮性质：文本方案复审（实现仍冻结至契约 v1.0 B1/B2 销账 = task-2 通过）。

## 判定：**通过（文本方案层面）**——9 项整改全部接受方向正确；1 项口径修正（§2，实现硬门）+ 3 项建议（§3）

## 1. 销账复核（F-1..F-9）

| 项 | v2 整改 | 复核结论 |
|---|---|---|
| F-1 BACK/硬件键 | BACK→-7 且吞默认；DPAD→-1..-4；DPAD_CENTER/ENTER→-5；MENU→-6；音量键交系统 | ✅ 销账。退出唯一路径保 m_000 信号链正确；BACK 默认机制留 B 级、实现期升 A 的处置合规（AGENTS §3 分级） |
| F-2 等价性 | README 引用契约 §1.3 + 平台落地同构声明；弃自造 split('@') | ✅ 销账。键模型统一到 RMS_STORES/rms_key/rms_store_name（与 game-core paint.rs:638 消费面兼容）；B1(MOT_IF) 以契约 v1.0 为准引用 = 正确依赖 |
| F-3 第五缝 | TrackStore 装载改 AssetSource::read("track{i}.ogg")；exists 预检改 try-read；lewton 锁 =0.10.2 | ✅ 销账。与 shell-port 同口径；版本锁定防双解码器漂移正确 |
| F-4 双拷贝 | 同步责任入 README + 双壳对照 golden + 抽 crate 立待办交 lead | ✅ 销账（过渡方案合格）。抽 shell/common 裁决在 lead；裁决前按过渡案执行 |
| F-5 契约分叉 | 四签名全部对齐契约成稿；弃 Option/exists | ✅ 销账 |
| F-6 音频焦点红线 | **整体砍除**（不订阅焦点回调）；焦点被抢期间持续发声登记平台近似边界 | ✅ 销账。红线清除；来电/锁屏经 onPause→Hide 覆盖，与原版 hideNotify 链语义吻合 |
| F-7 生命周期近似登记 | 锁屏/来电/退后台=Hide；分屏可见失焦=不发 Hide（与 win 失焦不映射同口径）；画中画=Hide 登记偏差；灭屏=Hide | ✅ 销账。登记表诚实；画中画项见 §3-S1 建议 |
| F-8 基线纪律 | ratchet-update 显式 commit + 说明 | ✅ 销账 |
| F-9 杂项 | pointer 按区域定角色；minSdk 26；不写死"55 层" | ✅ 销账。"55 层"不转引的处置正确（该出入是我方 git 审计 G3 的 C 级项，不上结论） |

§7 TDD 协议按契约审查 §1 裁定（方案 A + RED 证据三件 + 复现指令）执行——✅ 口径正确。
crate-type/Viewport 处置维持前判——✅。

## 2. 口径修正（【2026-10-17 更正】我方措辞，实现硬门）

> 【2026-10-17 更正】我在 v1 审查 F-9 整改里写的「软键+方向盘同时按（行走 keyHeldCode
> + 确认 keyValue **并存**）」措辞不精确，**不构成行为判据**。以下为精确口径，实现与
> TDD 用例以此为准。

**多点触控必须折叠进单一键状态机，禁止 per-role/per-pointer 独立 held 状态**：

- 权威语义（A 级锚）：`GameScene::press_key(code)` = `key_value = code; key_held = code`
  （覆盖式取末键，crates/game-core/src/scene.rs:4270-4274，a.java:4663-4676）；
  `release_key()` = `key_held = 0`（**不论哪个键**，scene.rs:4276-4279）；
  壳侧 route_key 投递语义（shell/win/src/main.rs:531-556）。
- 由此：按住方向盘(-1)再按软键(-5) ⇒ 事件流 press(-1)、press(-5)，状态折叠为
  `key_value = key_held = -5`（行走即停）；松开任一触点 ⇒ `key_held = 0`，**行走不自动恢复**
  （原版实机同款：keyHeldCode 单字段被覆盖）。这是保真行为，"松开软键恢复行走"= 发明行为 = 禁止。
- 「按区域定角色」只决定**触点→键码的翻译**（不互相抢占角色），不决定状态共存；
  两指事件仍按时间序逐个进 press_key/release_key 折叠。
- TDD 用例改写为：①方向盘 press(-1)→软键 press(-5) 序列 ⇒ 断言折叠终态
  （key_value/key_held 口径 = 与两键物理按下的 Java 回调序列逐事件等价）；②任一触点
  Up/Cancel ⇒ key_held=0；③事件顺序保序（后按覆盖先按）。
- 注：mode3 行走消费面含 49-55（key-code-contract §2），触控方向盘映射沿 win 的 -1..-4
  表（win 实测可行走）——翻译表本身不动，仅折叠语义如上。

## 3. 建议项（不卡闸）

- **S1 画中画**：Windows 语义锚是"完全遮挡才 hide"（main.rs:17-20）；PiP 是可见非遮挡
  ⇒ 按锚**不该发 Hide**。v2 把 PiP 映射为 Hide 并登记偏差=诚实可接受；但实现期可用
  `isInPictureInPictureMode`/`onPictureInPictureModeChanged` 把 PiP 归入"可见失焦不发 Hide"
  （更贴锚），届时把偏差表该行改为已消除。两条路都合规，倾向后者。
- **S2**：硬件键盘 KEYCODE_SPACE→-5 补齐（与 win Enter|Space→-5 同表，main.rs:230）；
  小节即可。
- **S3**：F-4 双壳对照 golden 会触碰 shell/win 测试面（跨写范围）——实施前和 shell-port
  通气或请 lead 排写范围，避免双写。

## 4. 实现期我将核（预告，非本轮结论）

1. git log 验证 TDD 真实先行（RED 证据三件 + 按复现指令抽查 ≥1 单元）；
2. §2 折叠语义的 TDD 用例逐条对锚；
3. BACK 吞默认的 A 级证据（onInputEvent 返回值 + native_app_glue 语义）回写 README；
4. README 验证状态矩阵如实标注未编译验证面；
5. cargo test -p game-android 全绿 + build --workspace 零错 + gates 8/8。
