# 对抗审查：shell-port「Windows 壳迁移方案 v1」（四接缝 trait 化）

> 审查人：adversary（对抗性审查岗）。审查对象：shell-port 2026-10-17 提交的
> 「Windows 壳迁移方案：四接缝 trait 化」（基线 ba14d4e，写范围 shell/win/**）。
> 审查方式：逐条声称对照代码实况（grep/read 亲验），Java 真值锚点对照 reference/src/deobf/a.java。
> 产出目录：docs/findings/reviews/（lead 指定；历史复核书在 docs/reviews/ 与 docs/spec/*-review.md，不迁）。
> 纪律依据：AGENTS.md §1（保真只增不减）/§3（证据分级）/§5（门禁）/§9（提交规范）。

## 判定：**有条件通过**（方案可开工；F1–F4 为实现审查的准入条件，销账前不得提交迁移 commit）

方案的事实核查质量高——我核对的 14 处行号锚点**全部命中**，测试计数 26 精确。
但存在 1 个实质缺口（第五接缝漏迁）与 3 个判据/表述缺陷。以下逐条。

---

## 1. 已亲验为真的声称（A 级，逐一 grep/read 核过）

| 方案声称 | 实况锚 | 结论 |
|---|---|---|
| Rms 固定 8 键白名单 | shell/win/src/main.rs:172-178 | ✅ 真 |
| persist 内容不变不重写 | shell/win/src/main.rs:182-191 | ✅ 真 |
| persist 全部错误 `.ok()` 吞掉 | shell/win/src/main.rs:183,188 | ✅ 真 |
| Shell 持 rms + stores | shell/win/src/main.rs:435-436 | ✅ 真 |
| step 存档块 upsert+persist | shell/win/src/main.rs:757-781 | ✅ 真 |
| persist_global_if_dirty | shell/win/src/main.rs:740-755 | ✅ 真 |
| PcmDevice 本地 trait（`Send`、write→Result<(),String>、idle） | shell/win/src/audio.rs:214-219 | ✅ 真 |
| WinMmPcm QUEUE_DEPTH=4（BUG-019） | shell/win/src/audio.rs:422,459,479 | ✅ 真 |
| CHUNK_FRAMES=1024 / CHANNELS=2 / SAMPLE_RATE=44100 | shell/win/src/audio.rs:580/21/20 | ✅ 真 |
| Shell::finished() 读 BootMachine::finished | shell/win/src/main.rs:671-676 + crates/game-core/src/paint.rs:585 | ✅ 真 |
| 消费点 about_to_wait `el.exit()`、CloseRequested→el.exit() | shell/win/src/main.rs:1071-1074 / 1019 | ✅ 真 |
| title.rs:181 menu_kinds=[0..5]、kind 5→LogoAnim | crates/game-core/src/title.rs:181/235 | ✅ 真 |
| 测试计数 26 = main 13 + audio 6 + view 4 + lifecycle 3 | grep `#[test]` 逐文件清点 | ✅ 真（13/6/4/3 精确） |
| include_str! fixture 是编译期数据非运行时缝 | shell/win/src/main.rs:52-59,120-127 | ✅ 判定正确 |
| A4 依赖 assets/raw 与既有 shell_tests 同口径 | shell/win/src/main.rs:1228；CI 夹具自供给 gates/cli.py:220 | ✅ 真 |

## 2. 攻击面与整改条件

### F1【实质缺口·硬条件】BGM 资产读取是第五接缝，"四缝"清单漏之

- **证据**：shell/win/src/audio.rs:617-620 `track_path(dir,i)`、:629-630
  `track_path(audio_dir,i).exists()`、:665-666 `std::fs::read(&p)`（bgm-decode 线程装
  track2/track3）；入口签名 `start(audio_dir: &Path, device: &str)`（audio.rs:626）整个
  绑死文件系统路径。
- **证伪点**：方案问题 5 自问"四缝之外漏迁什么"，但四缝迁完后 `std::fs` 并不只剩
  FileStore/DirAssets——音频轨加载仍走 std::fs。Android APK assets 无路径可 `join`，
  "四缝 trait 化 ⇒ 可落 Android"的隐含结论不成立。
- **整改**（二选一，方案必须写明）：
  1. AssetSource（或独立 AudioAsset 面）覆盖 `track{i}.ogg` 字节供给，`start` 改收
     字节提供者；或
  2. 显式登记"第五接缝待办"，里程碑措辞降级为「四接缝迁移 + 已知第五缝未迁」，
     并在 docs/knowledge/architecture_shell-contract.md 接缝清单补行（该文现列
     接缝 4 为"BGM 目录注入 --audio"——路径注入这半边本身就是 Android 不可落的形态）。
  另注意映射：game-core 轨道表是 `menu.mid/game.mid`（crates/game-core/src/audio.rs:29），
  壳侧产物是 `track{i}.ogg` 重编码（tools/audio）——名字映射不得在迁移中丢。

### F2【判据缺口·硬条件】T1 白名单之争的根因在契约形状：load_all 不是 RMS 语义

- **证据（Java 真值）**：RMS 全部按**名**开、从不列店——
  a.java:4456（SKY_WAR 读）、8928（SKY_WAR 写）、8977（MOT_IF 读）、9030（MOT_IF 写）、
  9193/9259（MOT_L 写/读）；读 = `enumerateRecords().nextRecordId()` → `getRecord`
  （a.java:4457-4459/8978-8980/9260-9262，取**枚举首条**）；写语义**按店不同**：
  SKY_WAR = `getNumRecords()==0 ? addRecord : setRecord(1,…)`（a.java:8929-8954），
  MOT_IF = `deleteRecordStoreQuietly` 后 `addRecord`（a.java:9027,9052），
  MOT_L = 同款 delete+addRecord（a.java:9190,9244）；删 = `deleteRecordStoreQuietly`
  （a.java:9330-9335；SKY_WAR 读失败分支 a.java:4475-4478 = 删店+默认值+saveGlobalProgress）。
- **证伪点**：`load_all` 是"壳启动便利"不是 RMS 语义；T1 里"白名单 vs 列目录"两边都不是
  Java 形状。平铺单 blob 等价性**可以**成立（本游戏每店恒单记录：写前必 delete 或
  setRecord(1)；delete+add ≡ 覆盖），但方案必须给出这段书面论证，并注明它只在
  "每店单记录"前提下成立；契约不得自称 RecordStore 语义。
- **硬伤补充——键模型跨层**：`{name}@1` 键格式不止在壳里——game-core 也消费它
  （crates/game-core/src/paint.rs:638 `k.starts_with("MOT_IF@")`、scene.rs:7848
  `k.starts_with("MOT_L0@")`），preset 文本格式即 `KEY@1=hex`。若契约键类型去掉 `@1`
  改纯店名，迁移会越界改 game-core（超出 shell/win 写范围）。整改：键类型保持
  现字符串格式，或 (name, record) 二元 + 壳边界 1:1 转换并加断言；错误类型/键域
  归属的问询升级为**实现前必须由 task-2 书面裁定**，不允许两壳各自猜测（android-shell
  的暂定签名 `read(name)->Option<Vec<u8>>` 已与你们的 `read(rel)->Result<Vec<u8>,String>`
  分歧——形状分叉已实际发生）。
- **同缺陷面**：delete 面在暂定签名里没有。等价性论证若成立可不加，但须写明
  "delete 语义被覆盖写吸收"的推理与反例边界（多记录店即反例）。

### F3【判据缺口·硬条件】A2 幂等判据弱、T3 错误吞掉无测试、大小写平台差未覆盖

- A2 "mtime 不变"只证"没重写"不证"写对"，且 NTFS mtime 精度语义与 ext4 不同
  （跨平台存档环终会踩）。整改：A2 同时断言内容 hash；B1 才是主判据（见 F4 相邻项）。
- T3"错误不 panic"现无判据测试。整改：只读目录/注入 IO 失败用例，断言 persist 返回
  不 panic（现状 `.ok()` 语义，main.rs:188）。
- 大小写：Windows FS 不区分大小写、Android ext4 区分。白名单 `MOT_L0.bin` 若遇
  `mot_l0.bin` 杂散文件，两平台行为不同。整改：A1 加大小写用例，锁"精确名匹配"。
  > 【2026-10-18 更正】上句"锁'精确名匹配'"**指令错误**（shell-port 对账时证伪，我复核
  > 成立）：现状 `std::fs::read(dir.join("MOT_L0.bin"))` 在 Windows 天然大小写不敏感，
  > 小写杂散 `mot_l0.bin` 会被命中并以 `MOT_L0@1` 进 stores——"精确名匹配"是**行为变更**，
  > 违反迁移零漂移。正确口径：A1 用例**锁现状** = FS 原生语义（cfg(windows)/cfg(not)
  > 分支显式声明两平台差，返回键恒白名单规范名）；改精确匹配须 lead/用户裁定另立变更。
  > shell-port 的 `storage_lookup_follows_fs_case_rules_key_stays_canonical` 即此口径，销账。

### F4【判据缺口·硬条件】B1/B2/B4 双实现对照有共享误解风险（gotcha_shared-shim-assumption-self-confirming 在案），且旧实现删除后对照消失

- 整改：B1 的对照结果**落成字节级基线常量**（固定目录布局→期望 hex），旧 Rms 删除后
  仍可测；Rms 格式对偶的 Java 锚（a.java:8983-8996 读序 ↔ menu_family::load_slot_table）
  由既有 rms_roundtrip（main.rs:333）承载——保留即够，勿改断言。
- exit 语义口径（问题 3 的直接回答）：现状 = **粘滞信号每 tick 轮询一次**
  （main.rs:1072 在 `pacer.due` 内），winit `el.exit()` 置退出标志后事件循环本轮即终止
  ⇒ 实际只被调一次，但这是"循环先退"的巧合，不是"消费一次"的语义。A3 断言
  "后续拍不再调"要求实现加消费位 = **新语义**（壳内实现细节、游戏不可观测）——可以做，
  但不得写成"现状行为"。二选一照实写：(a) 断言"每轮 poll 至少调一次直到循环退出"；
  (b) 加幂等守卫并注明"壳内新增，游戏不可观测"。另 `WinitExit<'e>{el}` 倾向合理
  （与 el.exit() 逐字等价）；"全退出点 grep 可证"请给出确切模式与期望计数
  （现状 `el.exit()` 2 处：main.rs:1019/1073）。

### F5【建议】panic 文案锚定面不足

A4 只锚"缺资产"一处；实际 panic 文案三类：load_container（main.rs:61-70，含
"先跑 tools/extract-assets/extract.py"）、maplv{n}/sprite{n}（main.rs:131-137）、
Shell::new 的 png()/mapbg（main.rs:464-483）。逐条锚定或统一，防"逐字保留"只保住一处。

### F6【建议】明确"不动清单"

font_inject（std::fs 字体发现，main.rs:239-321——候选路径是 Linux/Windows 专属，
Android 需系统字体 API）、view::Pacer/Viewport、winit 输入翻译：本方案不迁可以
（rule of three），但请把"明确不动清单"写进方案，防实现期误带入。AGENTS §2 承诺的
Renderer/Input/Font trait 尚未抽，保持"第二个壳真实需求驱动"节奏即可。

## 3. 对方案 §6 五问的回答

1. **T1 键域白名单**：见 F2——最大漂移点属实，但根因是契约形状（load_all 非 RMS 语义）；
   白名单本身是"Java 只按这 8 个名开"的事实复刻，保留有理（a.java:4452/8924/8960/9190）。
2. **B1/B2/B4 够不够**：不够——共享误解 + 旧实现删除后对照消失；按 F4 补基线常量。
3. **exit 口径**：见 F4；"恰一次"是巧合不是语义，照实二选一表述。
4. **panic 逐字保留**：方向对（防文案漂移），但覆盖面不足（F5）；不算过度。
5. **漏迁接缝**：有——BGM 资产读取第五缝（F1）；font/view/input 属"明确不动"（F6）。

## 4. 证据分级与方法声明

- 本记录所有"证据"行均为 A 级（行号锚由本次 grep/read 亲验，非转引方案文本）。
- Java 真值锚点抽查于 reference/src/deobf/a.java 上述行号。
- 假设区：无（本记录不含 C 级结论）。
- 未验证项：实现未开工，B1/B2/B4 测试的实跑结果留待实现审查（届时查 git log 验证
  TDD 真实先行：行为锚 commit 与迁移 commit 分离）。

**销账条件**：F1–F4 各需在方案修订或实现中落地并可指认；实现审查提交时逐条对账，
任一未销 ⇒ 实现审查驳回。
