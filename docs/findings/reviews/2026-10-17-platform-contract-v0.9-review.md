# 对抗审查：platform-contract「game-platform 平台契约 v0.9」

> 审查人：adversary。对象：docs/spec/platform-contract.md v0.9（四 trait：Storage/
> ProcessControl/AudioOut/AssetSource）。基线 ba14d4e。
> 纪律依据：AGENTS §1/§2/§3/§5/§9。发现分**阻塞/建议**两级，仅阻塞项卡闸（lead 定序 2026-10-17）。

## 判定：**有条件通过**——先裁纪律卡点（§1，解锁全链），契约本体 2 项阻塞（§3-B1/B2）销账后升 v1.0、双壳开工

---

## 1. 纪律卡点裁决（TDD 红提交 vs pre-commit 总闸，全链解锁项）

**裁定：方案 A（红证据入 commit 正文 + spec §7 记录表，绝不 --no-verify）+ 强化审计协议。**
B 案否决。

- **前提核实为真**（A 级）：总闸 `gates/cli.py check` 确含 `cargo test --workspace --release`
  （gates/cli.py:215-248 `gate_rust_tests`；纳入 check 见 :388-390），pre-commit 调 check
  （AGENTS §5）。故"红提交过不了总闸"成立，纪律冲突真实。
- **红线优先**："不得绕过/弱化/删除门禁"（AGENTS §5）> TDD 历史形状。`--no-verify` 永久禁止。
- **B 案否决理由**：死文件测试**从未真正红过**（不挂 mod = 不过编译 = 不运行），
  "测试先行"只剩历史形状、无事实；且引入静默跳过风险（本仓前科
  gotcha_nested-test-silently-skipped.md，test-ratchet 即为此设）与半成品悬挂风险。
  A、B 的差距仅历史形状，用审计协议补足（见下），不值得引入假绿面。
- **A 案强化协议**（每单元 commit 满足）：
  1. **RED 证据三件**入 commit 正文 + spec §7 表：(i) 失败输出逐字（含用例名或编译错误）；
     (ii) 声明"RED 所跑测试代码与本 commit 测试逐字一致"；(iii) **RED 复现指令**
     （把本 commit 的测试部分单独应用到父树运行即得红）；
  2. 测试与实现**分文件或可剥离 hunk**（复现红的前提）；
  3. message 携 `Evidence:`/`Confidence:`（AGENTS §9）；
  4. **审计权**：adversary 实现审查时按 (iii) 抽查 ≥1 单元复现红——复现不出 ⇒ 实现审查驳回。
- 附注（非阻塞）：新测试使计数只增，`gate_test_ratchet` 只在**减**时 FAIL（gates/cli.py:303-311），
  新增二进制会提示 `ratchet-update`（:312-314）——game-platform 加测试不卡闸，但基线
  更新须按纪律写 commit 说明（AGENTS §2）。

## 2. 逐条事实核验（方案声称 → 实况锚）

| 声称 | 实况锚 | 结论 |
|---|---|---|
| RMS 键域 8 店 Java 硬编码锚 | a.java:4452 "SKY_WAR" / 8960 "MOT_IF" / 9190 "MOT_L"+槽号 | ✅ 真 |
| 读=首条枚举记录（三处） | a.java:4457-4459 / 8978-8980 / 9260-9262 | ✅ 真 |
| SKY_WAR 写=setRecord(1)/空店 addRecord | a.java:8928-8954 | ✅ 真 |
| MOT_IF/MOT_L 写=delete 整店+addRecord | a.java:9027-9052 / 9190-9244 | ✅ 真 |
| SKY_WAR 读失败分支壳侧对偶 | shell/win/src/main.rs:487-498（catch 语义+置脏回写）+ 测试 `skywar_corrupt_resets_default`（main.rs:1577） | ✅ 真 |
| ProcessControl 信号半：finished + 脚本出口 | crates/game-core/src/paint.rs:221（`pub finished: bool`）/ :585（置位）+ script.rs:158（`fn exit_application`） | ✅ 真 |
| AudioOut 自 PcmDevice 迁入、双实现手写 unsafe impl Send | shell/win/src/audio.rs:214-219 / :250 / :437 | ✅ 真 |
| Android 映射（AAssetManager/finish/AAudio/私有目录） | §5 表；APK assets 不可 std::fs 与 android-shell v1 §2④ 一致 | ✅ 方向正确 |
| "pre-commit → check → cargo test --workspace --release" | gates/cli.py:234,388-390 | ✅ 真 |

## 3. 阻塞项（销账后方可升 v1.0 / 双壳开工）

### B1【等价性论证缺口·可达路径】§1.3 边角表缺 MOT_IF 读失败的**写副作用**

- **事实**（A 级）：`loadSlotTable` 的 catch 臂是 `markSlotEntry(-1)`
  （a.java:9000-9001），而 markSlotEntry **会整店重写 MOT_IF**
  （deleteRecordStoreQuietly + addRecord 全零槽表，a.java:9027-9052）。
  即：MOT_IF 损坏/截断时 Java **当场把它重置为合法零表并落盘**。
- **Rust 侧现状**：`load_slot_table` 损坏 ⇒ `SlotSelect::new()` 零表，**无回写**
  （crates/game-core/src/menu_family.rs:713-741；shell/win/src/main.rs:856 只回填内存）。
- **证伪点**：边角表 a/b/c 都论证"不可达"，但此差异**可达**（存档文件被截断/损坏是
  用户实机可发生事件——BUG-018 即用户实机存档环问题）。观测差异：损坏 MOT_IF 后
  Java 磁盘终态 = 合法零表；Rust 磁盘终态 = 损坏字节，直至下次保存才收敛。
  游戏可见态两边等价（零表），但"观测等价"若含存储终态则**不成立**。
- **整改**（二选一，书面落地 §1.3）：
  1. 补 edge d：声明"损坏 MOT_IF 读入 ⇒ 游戏态等价、存储终态差异存在且收敛于下次
     markSlotEntry"，并把口径从"观测等价"精确为"游戏态观测等价"（推荐——不动代码）；或
  2. 实现补回写对偶（load_slot_table 失败 ⇒ 零表序列化回写），行为完全对齐。
  同文顺带核对 MOT_L 读失败（a.java:9320-9322：catch 只 printStackTrace+返回 false，
  **无**回写）与 Rust 对偶一致（该路无差异，可入论证作对照）。

### B2【平台差异登记】音频轨资产供给（第五缝）未登记进契约范围声明

- **事实**：`track{i}.ogg` 走 `--audio` 目录 + `std::fs`（shell/win/src/audio.rs:617-620,
  629-630, 665-666）；§4.2 已声明"暂不走本 trait"，但 §0"不在本契约"清单与 §5 平台表
  **都没有它的 Android 落地处置**。android-shell v1 正因此踩坑（"照搬 audio.rs"撞
  APK assets 无 std::fs——见 android v1 审查 F-3）。
- **整改**（一句话级）：§0"不在本契约"清单 + §5 表脚注补一行："音频轨资产供给
  （--audio/track{i}.ogg）= 已知第五缝，不入本契约；Android 落地必须并入 AssetSource
  或单列 trait，不得沿用路径注入"。

## 4. 建议项（不卡闸）

- **C-2**：`persist` "域外键=契约违约"与"不得 panic"并存未定义——补一句"违约时实现
  丢弃该键并记日志（仍不 panic）"或改"契约保证调用方不传域外键（debug_assert 可选）"。
- **C-3**：ProcessControl "不返回"措辞误（`el.exit()` 正常返回、置退出标志）——
  改"调用后事件循环于当前回调返回后终止"。
- **C-5**：§0 "已经存在两个消费者"——shell/android 尚无代码（v1 方案在审），
  改"一现实在役 + 一在建（用户结构决策 ba14d4e）"。
- **C-7**：§7 表建议加"RED 复现指令"列（对应 §1 裁定协议第 1(iii) 条）。
- **C-8**：`rms_store_name` 签名 `-> &str` 建议显式标注 panic 面（键不含 `@` 时），
  或定为 `Option<&str>`——与"不得 panic"口径统一。

## 5. 假设区（C 级，不得入结论）

- android 侧 Storage 采用"注入根路径文件版"（§9 已自标 C 级）——同意处置：android-shell
  实现时验证；若改 SharedPreferences 更新 §5。
- Android AudioTrack 兜底（§5 表"或无声降级"）：android v1 已被裁定砍兜底/统一 minSdk 26
  （待 v2 落定），§5 届时写死单值，勿留"或"。

**销账方式**：B1/B2 改 spec 后知会我复核（文本级复核，一轮内出结论）；随后 §8 填审查结论、
升 v1.0。
