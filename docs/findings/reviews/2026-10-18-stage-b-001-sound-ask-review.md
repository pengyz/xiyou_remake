# 对抗审查：Stage B 第一单——开机声音询问移除（有意不一致方案）

> 审查人：adversary。对象：跳过 mode 21（SoundAsk）+ 默认开声 + 声音态持久化 + 新建
> docs/spec/stage-b-deviations.md 登记册。用户裁定驱动（诺基亚时代特征不需确认）。
> 本审查含**一处分类证伪**（③）与三项硬要求。

## 判定：**有条件通过**——③必须重分类（不是"零偏离 bug 修复"，是偏离点 3）+ 3 硬要求

## 1. ③分类证伪【本审查核心】

**声称**："apply_global 不装载 sound_enabled/volume ⇒ 补装载 = 纯 bug 级断链修复，
Java 侧 RMS 读档本就装载"。

**证伪**（soundEnabled 写点全集 grep，A 级）：a.java 全部赋值点仅
:2241（构造 true）/ :4157-4159（设置翻转）/ :4233-4234（询问 -7 答）/ :4239-4241
（询问 -6 答）/ :4484（mode 21 入场 false）——**没有任何 RMS→soundEnabled 的装载**。
:7290 是**存侧**镜像（`optionChecked[last] = soundEnabled`），读侧不回灌。
volumeLevel 同构（:2240/:4159/:4234/:4241 写点，无装载）。

**结论**：原版**从不持久化生效声音态**——每次启动都经 mode 21（false → 询问 → 答案）
重新决定。因此：
- 跳过询问（偏离 ①）后若不补装载，声音态永远停在构造默认 true（无视用户上次选择）；
- 补 apply_global 装载 = **偏离点 3：声音态跨启动持久化（原版无此行为）**，是偏离 ① 的
  必然连带、方向符合用户意图，但**必须登记为有意不一致**，不是零偏离修复。
- 整改：登记册第 1 条含**三个偏离点**（①询问屏移除 ②前置 false 移除 ③声音态持久化），
  ③附上表 grep 证据作对照锚。

## 2. 金样/差分影响【硬要求 2】

mode 21 在 boot 链上（mode 0 末 → 21 → 答 → mode 1）⇒ **凡经 SoundAsk 的夹具
tick 序列全部漂移**（boot_replay、EV-toggle-exit 等启动链场景首当其冲；A/B/C oracle
trace 是 Java 侧不动，动的是 Rust 侧对拍期望）。要求：
1. **影响枚举**：登记册"金样影响"字段列**具体夹具名/场景名**与处置（改期望/白名单/跳过），
   不许"如有则改"的空泛表述；
2. **白名单机制**：有意不一致的差分豁免必须**引用偏离 ID**（STAGE-B-001 式，
   仿 AGENTS §6 bug-偏差 TOLERATED+编号 先例），基线变更写 commit 说明；
3. 禁静默改期望 ✓（你已声明）——加上"每处改动注释带 STAGE-B-001"闭环。

## 3. 其余核验

| 项 | 核验 | 结论 |
|---|---|---|
| ① 跳过点 | paint.rs:796 `mode=SoundAsk`→`MainMenu`；Java 锚 a.java:3371-3373 ✓ | 方向可行 |
| ② 默认开声 | BgmState::new sound_enabled=true（crates/game-core/src/audio.rs:86）+ 跳过 :4484 前置 false ✓ | ✓ |
| ④ minimap/全局装载半边 | **关键点核过**：SKY_WAR RMS 读在 Java 是 case 21 **入场**干的（a.java:4450-4479）——跳过 mode 21 不得连带丢掉全局装载。Rust 侧该半边结构独立：shell boot 构造链 apply_global（shell/win/src/main.rs:487-498，SKY_WAR 解析+corrupt 回退）✓ 不随跳过丢失。minimap 默认 true 来自 a.java:4477 catch 与 Rust 初值——TDD 须钉"跳过后 minimap_enabled 仍 true + option_checked 装载生效" | ⚠️ 待 TDD 钉 |
| 登记册 | docs/spec/stage-b-deviations.md（偏离点/理由/对照锚/金样影响）= Stage B 有意不一致的正确纪律面（首条立范式） | ✓ 概念通过 |

## 4. TDD 增补【硬要求 3】

三测（boot 直入 MainMenu/默认开声/apply_global 装载+round-trip）之上补**第 4 测**：
**跳过后标题 BGM 启动**——mode 21 在 BGM 链上（回答后进 m_000 case 1 = enter_title
m_139(2)），跳过路径必须验证 MainMenu 入口仍触发 `bgm.request()==Some(2)`（=标题曲），
否则"直入标题"会带出无声标题屏的新 bug。第 5 测建议：全局装载半边（minimap_enabled
初值 true + option_checked[0..] 装载后设置面板显示正确）。

## 5. 登记册必备字段（立范式）

每条偏离：**ID（STAGE-B-xxx）/ 偏离点（Java 锚 vs Rust 行为）/ 用户裁定原话+日期 /
理由 / 金样影响（具体名单）/ 代码注释引用（`// STAGE-B-xxx`，仿 BUG-xxx 式）/ 状态**。
首条即按此填——③以偏离点 3 入册（见 §1）。

## 6. 修订裁定（2026-10-18 用户二次裁定：声音=全局状态不入存档）——**方向通过，4 项随账**

修订（③改独立 settings 文件、GlobalSave 70/71 只写不读、BgmSnapshot 增 sound_enabled）
**优于原案**：声音态归平台层全局设置、出游戏存档系统——比我 §1 要求的"登记为偏离点 3"
更干净（持久化面离开 game-core/RMS 语义域）。**但四项不得丢失/须精确化**：

- **R1（防丢失）**：TDD ①"boot 直入 MainMenu 无 SoundAsk"必须含**跳过后标题 BGM 启动**
  断言（`bgm.request()==Some(2)`）——原硬要求 3 在修订 TDD 列表中消失，无声标题屏
  风险未消。另补：toggle 后 BgmSnapshot 变化检测落盘的时序测（设置翻转→下一拍检测→
  文件落盘）。
- **R2（防丢失）**：金样影响枚举 + STAGE-B-001 白名单（原硬要求 2）**修订未提**——
  SoundAsk 跳过使 boot 链 tick 漂移依旧存在（boot_replay/EV-toggle-exit 等），"④金样
  零漂移（存档格式不变）"只覆盖存档格式面，不覆盖 trace/夹具面。枚举+豁免引用 ID
  照旧必须。
- **R3（登记册措辞精确化）**：偏离点应为——①询问屏移除（a.java:3371-3373/4450-4484）
  ②前置 false 移除（:4484）③**声音+音量改为全局设置持久**（含 volume_level：原版
  volumeLevel 亦从不持久——写点仅 :2240/:4159/:4234/:4241，须一并入偏离点）④存档镜像
  字段（option_checked 声音位）改**只写不读**（注意：原版读侧本就不回灌 soundEnabled
  ——若 Rust 现有勾选框初始化曾读 option_checked[0]，改源=行为变化，入册；若本就不读，
  此点标"趋同非偏离"）。
- **R4（新）**：settings.rs 语义与 FileRms 同律——吞错 best-effort 不 panic、写-if-
  changed、键值格式定死（sound_enabled/volume_level）；双壳同语义（Android 落同款
  save 目录旁）；BgmSnapshot 新字段**不得扰动 Cmd 管线**（BgmController 比对面是
  track/serial——加字段加测试锁 cmd 流不变）。

存档格式零漂移（GlobalSave 序列化不动、round-trip 测照绿）✓ 方向正确。
偏离登记 #1 照旧 + R3 措辞。实现提交按 R1-R4 过审。
