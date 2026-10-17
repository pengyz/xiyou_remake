# game-platform 平台契约层（platform-contract）

> 状态：**v1.0 终稿**（2026-10-17 adversary 方案通过 + 2026-10-18 adversary 实现复审通过，
> §8；TDD 实测记录 §7 已全录）。
> 范围：trait 与类型居 `crates/game-platform`（AGENTS §2 红线：只放 trait 与类型）；
> 实现只在平台壳 `shell/win`（Windows）/ `shell/android`（Android）。
> 证据指针口径：`路径:行`；可判定性：本文件的签名/语义由 `cargo test -p game-platform`
> 机器判定，等价性论证为书面推理（A/B 级证据锚逐条给出）。

## 0. 总则（从真实调用点反推，禁止过早抽象）

游戏层只产生**信号与数据**，shell 负责**执行与持久**（`docs/knowledge/architecture_shell-contract.md`
§现行架构）。本契约只抽**一在役 + 一在建两个消费者**（shell/win 现实现、shell/android 在建）的接缝，
共 4 个 trait：

| # | trait | 接缝 | 现实调用点（shell/win） |
|---|---|---|---|
| 1 | [`Storage`](#1-storage) | 存档持久（RMS） | `Rms`（`shell/win/src/main.rs:163-192`）+ Shell::step 存档块（`:757-782`） |
| 2 | [`ProcessControl`](#2-processcontrol) | 退出执行 | `Shell::finished`（`:671-676`）→ 消费点 `el.exit()`（`:1072-1074`） |
| 3 | [`AudioOut`](#3-audioout) | PCM 输出 | `PcmDevice`（`shell/win/src/audio.rs:214-219`）+ `audio_loop`（`:582-615`） |
| 4 | [`AssetSource`](#4-assetsource) | 资产读取 | `load_container`（`main.rs:61-70`）/ maplv/sprite（`:131-137`）/ Boot 装配（`:464-484`） |

**不在本契约**（避免过早抽象，接缝清单仍以 architecture_shell-contract.md 为准）：
Renderer/Font/Input/可见性——尚无第二实现的真实需求，落地时再抽（rule of three 平台版）。

**已知第五缝（B2 登记）**：音频轨资产供给（`--audio` 注入目录 + `track{i}.ogg`，
`shell/win/src/audio.rs:617-620`、`:626-638`）是平台相关接缝但**不在本版 trait 集**
（消费者形态 = exists 预检 + 异步装载，与 AssetSource 同步读不同形态）。
Windows 侧口径（shell-port 2026-10-18 确认）：`start()/track_path/REGEN_CMD` 留 win 壳、
不入共享。**Android 落地必须并入 [`AssetSource`](#4-assetsource) 或单列 trait，不得沿用路径注入**
（APK assets 无 `std::fs` 路径语义）——android-shell v1 已因此踩坑；Android 侧轨道读取走
`AssetSource::read("track{i}.ogg")`。

## 1. Storage

### 1.1 签名（crates/game-platform）

```rust
/// RMS 记录键格式 `<store>@<record_id>`。本游戏每店单记录，记录号恒为 1
/// （与 oracle preset 文件 `KEY@1=hex` 同构；Java 侧从不按记录号读，见 §1.3）。
pub const RMS_RECORD_ID: u32 = 1;

/// 游戏 RMS 记录域（Java 原版硬编码店名：a.java:4452 "SKY_WAR"、8960 "MOT_IF"、
/// 9190 "MOT_L"+槽号 0..=5）。键域进契约而非各壳自抄，防双移植面漂移。
pub const RMS_STORES: [&str; 8] =
    ["MOT_L0", "MOT_L1", "MOT_L2", "MOT_L3", "MOT_L4", "MOT_L5", "MOT_IF", "SKY_WAR"];

/// store 名 → 记录键（`rms_key("MOT_L0") == "MOT_L0@1"`）。
pub fn rms_key(store: &str) -> String;
/// 记录键 → store 名：取首个 `@` 之前的部分；键无 `@` ⇒ 原样返回；空串 ⇒ 空串。
/// **全输入域不 panic**（与 `shell/win/src/main.rs:185` `k.split('@').next().unwrap()`
/// 同构——split 恒返回首段）。
pub fn rms_store_name(key: &str) -> &str;

/// RMS 持久接缝：具名 store 的字节记录。
pub trait Storage {
    /// 载入记录域内**已持久**的全部记录（缺失 store 静默缺席）。
    /// 契约保证：只返回 [`RMS_STORES`] 域内记录，键 = [`rms_key`]；
    /// 平台上的外来数据（如 save/ 下杂散 .bin）**不得**出现在结果里。
    fn load_all(&self) -> Vec<(String, Vec<u8>)>;

    /// 持久化一批记录：同键覆盖、新键创建（upsert）；未提及的键不动。
    /// 传入域外键 = 契约违约 ⇒ **该键丢弃 + 可记日志，不 panic**。
    /// 任何失败 best-effort 吞掉（实现可自记日志），与 Java `catch {}` 同口径
    /// （a.java:8955-8956）；**任何输入都不得 panic**。
    fn persist(&self, records: &[(String, Vec<u8>)]);
}
```

### 1.2 调用点映射

| 现代码 | 迁移动作 |
|---|---|
| `Rms::load(dir) -> (Rms, Vec<(String,Vec<u8>)>)`（main.rs:168-180） | 构造 Windows 实现 + `load_all()`；8 键白名单改用 `RMS_STORES`/`rms_key` |
| `Rms::persist(&self, &[(String,Vec<u8>)])`（main.rs:182-191） | `Storage::persist`；`{NAME}.bin` 文件名映射 = 实现内 `rms_store_name` |
| Shell::step 存档 upsert（main.rs:761-781） | 保持调用方 upsert/retain 逻辑不变（键域知识归游戏侧，`MOT_L{idx}@1` 等） |
| `persist_global_if_dirty`（main.rs:740-755） | 同上，`SKY_WAR@1` |

### 1.3 与 Java RMS 的等价性论证（A/B 级）

Java 原版 RMS 用法三式（唯一真相 = bytecode/反编译锚）：

1. **读 = 首条枚举记录，从不按记录号读**：SKY_WAR `a.java:4457-4459`
   （`enumerateRecords → nextRecordId → getRecord`）；MOT_IF `a.java:8978-8980`；
   MOT_L{n} `a.java:9260-9262`。
2. **写两式**：
   - SKY_WAR：`openRecordStore(name,true)` → `getNumRecords()==0` 则 `addRecord`，
     否则 `setRecord(1,…)`（`a.java:8928-8954`）⇒ 店内恒单记录且 id=1；
   - MOT_IF / MOT_L{n}：`deleteRecordStoreQuietly(整店)` → `openRecordStore(name,true)` →
     `addRecord`（`a.java:9027-9052` / `a.java:9190-9244`）⇒ 整店替换为单记录。
3. **读失败分支**：`catch` → `deleteRecordStoreQuietly` + `optionChecked[1]=true` +
   `saveGlobalProgress()`（`a.java:4475-4478`）⇒ 观测终态 = 店内容被默认合法值替换；
   壳侧对偶 = parse 失败 → `apply_global(default_7)` + `global_dirty` → persist 覆写
   （`shell/win/src/main.rs:488-500`、`:740-755`，回归 `skywar_corrupt_resets_default`）。

**结论（口径精确化，B1）**：**Storage 层**在"同一读写操作序列"下观测等价——本游戏可达写路径
恒产生"每店单记录"，读路径恒取该记录 ⇒ true RMS ≡ "具名 store 单 blob 覆写"；`{NAME}.bin`
平铺实现不改变任何可观察读写结果（A 级：三式均有行锚；B 级：等价推导）。消费方（game 层）
发出的操作序列差异**不属于 Storage 模型差异**，见 edge d。record id 在键中的 `@1` 只是
shim/preset 格式约定，故契约把记录号定为常量 `RMS_RECORD_ID`。

**书面声明的边角差异（Storage 模型内）**：

| # | 差异 | 不可达/收敛论证 |
|---|---|---|
| a | 多记录店：Java 读首条（枚举序），平铺实现读唯一 blob | 游戏写路径恒 delete 整店+addRecord（或空店 addRecord）⇒ 恒单记录；oracle 预置 `KEY@1=hex` 亦单记录 |
| b | `setRecord(1)` 假定 id 1 存在；若店非空而 id 1 缺失，Java 抛异常被 `catch {}` 吞（保存丢）、平铺覆写成功 | SKY_WAR 只经"空店 addRecord"与"delete 后重存"两路，恒有 id 1；同 (a) |
| c | 错误口径：Java 吞异常（`a.java:8955-8956 catch {}`；`9054-9055` printStackTrace 不抛），平铺吞错 `.ok()` | 同为 best-effort，契约 §1.1 明文"不得 panic" |

**edge d（B1 补，读失败路径的消费方写副作用——可达路径，必须登记）**：

| 读失败点 | Java 存储终态 | Rust 存储终态 | 收敛 |
|---|---|---|---|
| SKY_WAR（`a.java:4475-4478`） | delete 整店 + `saveGlobalProgress()` 重存默认合法值 | `apply_global(default_7)` + dirty → persist 覆写合法值（`shell/win/src/main.rs:488-500`、`:740-755`，回归 `skywar_corrupt_resets_default`） | **当拍即收敛**（同为合法默认值） |
| MOT_IF（`a.java:9000-9001` catch → `markSlotEntry(-1)`，其尾部整店回写 `9026-9052`） | **整店回写**当前内存槽表（= markSlotEntry 序列化；解析中断时含半解析残留） | `menu_family.rs:739-741` 解析失败**只归零不回写** ⇒ 损坏字节原样留盘 | **下次保存**（markSlotEntry 正常路径 → 壳存档块 upsert persist，`shell/win/src/main.rs:761-781`）后两侧**各自收敛为合法可解析表**；但写回的是各自内存态（Java 半解析 vs Rust 冷态，§9 #4），**内容分叉**随 #4 游戏态分叉持续至对应槽被重新保存覆盖 |
| MOT_L{n}（`a.java:9320-9322` catch 仅 printStackTrace + `var2=false`，**无回写**） | 损坏字节保留 | 损坏字节保留（parse 失败无写回） | **两侧一致**（无分叉） |

即：Storage 层等价性不受影响（消费方写多写少，平铺与 RMS 同响应）；MOT_IF 读失败的**存储终态**
分叉已精确登记，且有收敛论证。连带的**游戏态**分歧（Java 解析中断后内存保留半解析表、
Rust 归零冷态）是 game-core 保真问题、超出本契约范围，登记为 §9 假设 #4 待考证。

## 2. ProcessControl

### 2.1 签名

```rust
/// 退出执行接缝。**游戏层只发信号**（`BootMachine::finished`：game-core
/// `paint.rs:221/585`，mode 22 退场 70 拍后置位 = CMidlet.m_000；脚本侧
/// `game-core/src/script.rs:157-158` 的 `exit_application` 同为信号），
/// **执行归 shell**（消费链 `shell/win/src/main.rs:671-676` → `:1072-1074`）。
pub trait ProcessControl {
    /// 请求平台终止应用。**正常返回**（置退出标志/退事件循环标志，进程可在随后的
    /// 壳收尾中结束；调用方不得假设进程立即消失）。幂等；不得 panic。
    /// 桌面 = 退出事件循环（进程随 main 返回而结束）；
    /// Android = Activity.finish()/ANativeActivity_finish（**不杀进程**）。
    fn request_exit(&mut self);
}
```

### 2.2 签名决策（取舍理由）

- **`&mut self`，无 `'static`/生命周期约束**：winit 的 `el.exit()` 只能在事件循环回调内调用
  （借用 `ActiveEventLoop`）。消费点 `about_to_wait`（`shell/win/src/main.rs:1062-1083`）
  本身即回调，两种实现均合法：①作用域内薄适配器直接 `el.exit()`；②exit-flag 信箱
  （AtomicBool）由宿主循环下一拍消费。契约不强加适配层，也允许它——行为等价（幂等保证）。
- **幂等**：信号消费循环每拍可重复调用（现状壳每拍检查 `finished()`，`el.exit()` 天然幂等）。
- **不叫 `exit`**：Android 侧执行是 finish Activity 而非进程退出，`request_exit` 表达
  "请求平台退出"而非"杀进程"；进程内绝无 `process::exit`（game-core 零进程 I/O 红线）。
- **`finished` 判定不入 trait**：那是 game-core 的信号语义（BootMachine::finished），
  shell 消费时读它；ProcessControl 只承接"执行退出"这一半。

## 3. AudioOut

### 3.1 签名（自 `shell/win/src/audio.rs:214-219` 的 `PcmDevice` 迁入，更名 AudioOut）

```rust
/// PCM 帧格式（write 的线上格式，开设备参数）：44.1 kHz 交错立体声 i16。
pub const SAMPLE_RATE: u32 = 44_100;
pub const CHANNELS: usize = 2;

/// 阻塞式 PCM 输出。**阻塞 write = 节拍**：音频线程以 write 返回速度推进
/// （`audio_loop`，`shell/win/src/audio.rs:582-615`；块长 CHUNK_FRAMES=1024 ≈23ms，
/// `:579-580`），故 write 必须阻塞到设备收下，不得无限缓冲（BUG-019：winmm
/// 单缓冲同步写拖慢+杂音 → 4 深常驻队列，`docs/knowledge/decision_shell-audio-dlopen-alsa.md`
/// 与 BUG-019 注册）。
pub trait AudioOut: Send {
    /// 写满交错 i16（SAMPLE_RATE × CHANNELS）。阻塞到设备收下。
    /// Err = **不可恢复**（调用方转无声，`audio.rs:603-611`）。
    fn write(&mut self, frames: &[i16]) -> Result<(), String>;
    /// 停止并丢弃缓冲（空闲）。混音器无活跃轨道时调用（`audio.rs:586-587`）。
    fn idle(&mut self);
}
```

### 3.2 签名决策

- **`Send` 保留**：设备构造后整体 move 进音频线程（`audio.rs:582` `Box<dyn PcmDevice>`
  move 进 `audio_loop` 线程；AlsaPcm/WinMmPcm 均手写 `unsafe impl Send`）。
- **Err = `String`**：与现状一致（`PcmDevice::write`、`decode_ogg` 均 `Result<_, String>`）；
  消费者只区分 ok/Err 并打日志，无类型化消费点 ⇒ 不引入错误枚举（禁止过早抽象）。
- **`idle` 与 `write` 分离**：空闲路径需停设备+丢缓冲（切断残响/省电），与写路径语义正交
  （`audio.rs:585-592` 空闲分支先 `dev.idle()` 再阻塞等命令）。

## 4. AssetSource

### 4.1 签名

```rust
/// 资产来源接缝：rel 路径读字节。
pub trait AssetSource {
    /// 读取 rel 路径资产。rel 约定：`/` 分隔、无前导 `/`、无 `..`；
    /// 根 = 平台注入的资产根（Windows：`--assets`，默认 assets/raw；
    /// Android：APK assets 根）。失败 ⇒ Err(人话诊断)，**不 panic**——
    /// panic 归壳侧且消息逐字保留（"缺资产 {name}（先跑
    /// tools/extract-assets/extract.py）：{e}"，`shell/win/src/main.rs:62-63`）。
    fn read(&self, rel: &str) -> Result<Vec<u8>, String>;
}
```

### 4.2 签名决策

- **Err = `String`**：调用点只做"成功 or 用错误文本拼 panic 消息"（`main.rs:62-63`、
  `:131-132`、`:468-469`），无类型化消费者；与 AudioOut 口径统一。
- **缺文件 = Err 而非 Option**：调用点要求把底层原因带进诊断文本（`{e}`），Option 丢信息。
- **rel 契约**：现状调用值均为纯文件名（"mapbg"/"maplv{n}"/"sprite{n}"/"l0.png" 等），
  契约放宽到子路径（`/` 分隔）以覆盖未来布局，但**当前无消费者使用子路径**（不预设目录列举）。
- **音频产物（`track{i}.ogg`）暂不走本 trait**：BGM 目录是独立注入缝（`--audio`，
  `audio.rs:617-620`），消费者形态不同（exists 预检 + 异步装载）；但**它是已知第五缝**
  （§0 B2 登记）：Android 落地必须并入本 trait 或单列 trait，不得沿用路径注入。

## 5. 平台实现映射表

| trait | Windows（shell/win） | Android（shell/android） |
|---|---|---|
| Storage | 文件版 RMS：`save/{NAME}.bin`（NAME=rms_store_name）、写-if-changed、吞错（`main.rs:167-191`）；save 目录 = `--save` 注入（exe 目录\save 默认，`main.rs:1131-1137`） | 应用私有目录文件版 RMS 逐项同构（注入根路径）；键域同 `RMS_STORES` |
| ProcessControl | winit `el.exit()`（事件循环回调内）或 exit-flag 信箱 + 宿主循环消费 | `ANativeActivity_finish` / Activity.finish()（不杀进程） |
| AudioOut | winmm waveOut 裸 FFI（4 深常驻队列，BUG-019）/ Linux ALSA dlopen；缺库 ⇒ NullSink 无声降级 | AAudio 阻塞 write（AudioTrack 兜底或无声降级） |
| AssetSource | `std::fs::read(--assets 根 / rel)` | `AAssetManager_open` + 全量读（APK assets 不可 `std::fs`） |

注（B2）：音频轨资产供给在 Windows = `--audio` 路径注入（`start()/track_path/REGEN_CMD`
留 win 壳）；Android **不得**沿用路径注入，轨道走 `AssetSource::read("track{i}.ogg")`
或单列 trait（§0 已知第五缝）。

注（文件名大小写，shell-port 2026-10-18）：存档文件名大小写跟随宿主 FS **原生语义**——
Windows 不区分（`MOT_L0.bin` 查找会命中 `mot_l0.bin`）、Android ext4 区分（不命中）。
契约不强行归一（行为锚：`storage_lookup_follows_fs_case_rules`，shell/win 锁现状）；
[`rms_key`](#1-storage) 产出的 store 名恒为大写原名（Java 原版字面），杂散大小写变体
属"外来数据"按 §1.1 处理。

## 6. 签名决策速览（shell-port/lead 四点问询的成文答案）

| # | 问题 | 裁定 |
|---|---|---|
| 1 | Storage::load_all 列目录 vs 键白名单 | **键白名单进契约**（`RMS_STORES`/`rms_key`）：实现只上报域内记录，杂散 .bin 不得进 stores；`{NAME}.bin` 命名映射归实现。错误口径：无错误返回，缺失=缺席，persist 吞错不 panic（Java `catch {}` 同语义） |
| 2 | AudioOut Send / Err 类型 | **`Send` 保留**；write `-> Result<(), String>`，Err=不可恢复（上层降级无声） |
| 3 | ProcessControl 生命周期/幂等 | `fn request_exit(&mut self)`：**无 'static/生命周期要求**、**幂等**；winit 直接 `el.exit()` 或 exit-flag 适配皆可 |
| 4 | AssetSource 错误/rel | `-> Result<Vec<u8>, String>`，Err=人话诊断；panic 留壳侧逐字保留；rel=`/` 分隔无前导/无 `..` |

## 7. TDD 记录（红→绿，A 强化协议——adversary 2026-10-17 裁定）

> 纪律冲突声明：pre-commit hook 实装（`.git/hooks/pre-commit` → `gates/cli.py check` →
> `cargo test --workspace --release`，`gates/cli.py:234,388-390`）。trait 新增的红阶段 =
> **编译失败**，编译失败的提交不可能过总闸 ⇒ "红提交进历史"与"不得绕过/弱化门禁"互斥。
> 裁定：红线"不得绕过/弱化门禁"优先于 TDD 历史形状，`--no-verify` 永久禁止；B 案
> （测试先行但死文件不挂 mod）**否决**（从未真正红过 + 静默跳过风险）。

**A 强化协议（每单元独立 commit，三项证据缺一即审查驳回）**：

1. **RED 输出逐字**：失败的用例名/编译错误全文入 commit message；
2. **一致性声明**：message 含"RED 测试代码与本 commit 逐字一致"；
3. **RED 复现指令**：可执行命令/步骤——把本 commit 的**测试部分单独应用到父树**即得红
   ⇒ 测试与实现**分文件**（测试在 `crates/game-platform/tests/`，实现在 `src/`），
   保证测试 hunk 可独立剥离复现。

每单元 commit 必携 `Evidence:`/`Confidence:` 字段；实现审查时 adversary 按 (iii) 抽查
≥1 单元复现红，复现不出 ⇒ 驳回。

| 单元 | RED 输出（逐字） | RED 复现指令 | GREEN | commit |
|---|---|---|---|---|
| Storage（键模型+trait） | `error[E0432]: unresolved imports game_platform::{rms_key, rms_store_name, Storage, RMS_RECORD_ID, RMS_STORES}`（tests/storage_contract.rs:12:21） | 仅把 `tests/storage_contract.rs` 应用到父树 41b5c93，跑 `cargo test -p game-platform --test storage_contract` | 7 测过（cargo-exit=0） | `a859408` |
| ProcessControl | `error[E0432]: unresolved import game_platform::ProcessControl`（tests/process_control_contract.rs:12:5） | 仅把 `tests/process_control_contract.rs` 应用到父树 a859408，跑 `cargo test -p game-platform --test process_control_contract`（等价指令 `git checkout a859408 -- crates/game-platform/src/lib.rs` 后跑同命令，已在 commit message 实跑存证） | 5 测过 | `485f542` |
| AudioOut | `error[E0432]: unresolved imports game_platform::{AudioOut, CHANNELS, SAMPLE_RATE}`（tests/audio_out_contract.rs:12:21） | 仅把 `tests/audio_out_contract.rs` 应用到父树 485f542，跑 `cargo test -p game-platform --test audio_out_contract` | 6 测过 | `cb86eb2` |
| AssetSource | `error[E0432]: unresolved import game_platform::AssetSource`（tests/asset_source_contract.rs:12:5） | 仅把 `tests/asset_source_contract.rs` 应用到父树 cb86eb2，跑 `cargo test -p game-platform --test asset_source_contract` | 4 测过 | `efb29bb` |

最终面：`cargo test -p game-platform` = lib 5 + 四契约 22 = **27 全绿**（cargo-exit=0、无警告）；
`cargo build --workspace` 零错。四项 (ii) 声明（"RED 测试代码与本 commit 逐字一致"）见各 commit message。

## 8. 对抗性审查记录

- 2026-10-17 送审 v0.9（本文件）：审查员 adversary，覆盖三必审面——
  ①Storage 键/记录模型 ↔ Java RMS 等价性（§1.3）；②Android 侧可落地性（§5）；
  ③TDD 真实性（§7 纪律冲突，请求裁决）。
- 2026-10-17 裁定（审查记录：`docs/findings/reviews/2026-10-17-platform-contract-v0.9-review.md`）：
  - ③TDD：**A 强化协议通过，B 否决**（§7 已按 (i)(ii)(iii) 三项证据落文）；
  - ②①契约 v0.9：**有条件通过**——逐条声称核验全真（a.java 各锚/Send/skywar_corrupt
    对偶/总闸前提均命中），B1（MOT_IF 读失败写副作用缺口）+ B2（音频轨第五缝未登记）
    两项阻塞销账后升 v1.0、双壳开工。
- B1/B2 销账（v0.91，本版）：§1.3 补 edge d（MOT_IF 回写分叉 + MOT_L 无回写对照 +
  存储终态收敛论证，口径精确为"Storage 层同操作序列等价"）；§0/§4.2/§5 补音频轨第五缝
  登记（Android 必须并入 AssetSource 或单列 trait）。非阻塞建议四项同步落文
  （persist 域外键=丢弃+日志；request_exit 正常返回措辞；"一在役+一在建"；
  rms_store_name 无 '@' 口径）。
- **2026-10-17 adversary 通过 / v1.0（附措辞修正 1 项）**：B1 三行表与口径收窄核实无误；
  §9 #4 半解析发现确认成立（adversary 亲证 a.java:8983-8997/9000-9001/9007-9024/9034-9050
  渐进写→不清数组→序列化残留）；措辞修正 = edge d MOT_IF 行"收敛为同一合法槽表"过强，
  已改为"各自收敛为合法表；内容分叉随 §9 #4 持续至对应槽被重新保存覆盖"。
  实现启动后 adversary 按 §7 (iii) 抽查 RED 复现。
- 入库备注：本文件 v1.0 全文经并发提交 `41b5c93`（"G1 对抗审计修正"）卷入入库——
  该 commit 的 message 未含本文件的 Evidence/Confidence（并发 `git add` 竞态）；
  证据字段由后续 TDD 单元 commit 与 §7 记录 commit 补记，内容以本文件为准。
- 实现落地（2026-10-18）：四单元 TDD 全绿（§7 表），实现复审交 adversary 按
  §7 (iii) 抽查 RED 复现——结论待回填。
- **2026-10-18 adversary 实现复审：通过**（审查记录：
  `docs/findings/reviews/2026-10-18-platform-contract-implementation-review.md`）：
  ①(iii) RED 独立复现成功——`git archive a859408` 父树导出 temp + 仅叠
  485f542 的 tests/process_control_contract.rs ⇒ `E0432 … :12:5` 与 commit message
  逐字吻合；②GREEN 终面独立复跑 = 27/27（5+7+5+6+4）EXIT=0；③四点实现面逐行
  亲验（签名逐字一致/负形状锁非放宽/RMS_STORES 无越权/替身语义一致）；
  ④三对账干净（内容/声称/写范围）。备忘三条（BOM 卫生/两操作坑落库/§7§8 回填）
  已随本次落文清账。

## 9. 遗留假设清单

| # | 假设 | 级别 | 处置 |
|---|---|---|---|
| 1 | 多记录 store / id≠1 店在游戏可达路径不可达 | B（写路径穷举 a.java:8928-8954/9027-9052/9190-9244） | 差分发现反例时升级 Storage 模型（多记录枚举） |
| 2 | Android 存储采用"注入根路径文件版 RMS 逐项同构"（android-shell v1 方案） | C（方案未落地） | android-shell 实现时验证；若改 SharedPreferences 则 §5 映射表更新 |
| 3 | record id 恒 1（`RMS_RECORD_ID`） | A（oracle preset `KEY@1=hex` + Java 写路径） | — |
| 4 | MOT_IF 读失败（截断/损坏）游戏态分歧：Java 解析中断后内存**保留半解析表**（`markSlotEntry(-1)` 不清数组，a.java:9007-9024），Rust `load_slot_table` 失败**归零冷态**（`menu_family.rs:739-741`）——Rust 注释 `:693` 声称"markSlotEntry(-1)=全无效冷态"仅在首字节即失败时成立 | B（代码语境推断，双方均可静态复现；adversary 2026-10-17 复核确认成立） | 超出本契约范围（game-core 保真问题）；移交 game-core 侧考证。**修复方向 = Rust 复刻半解析**（Stage A"行为只增不减"，原版 quirk 也是行为）；BUG-xxx 登记的是原版 quirk 本身防 Stage B 误"修复"，不是给 Rust 归零行为开豁免 |
