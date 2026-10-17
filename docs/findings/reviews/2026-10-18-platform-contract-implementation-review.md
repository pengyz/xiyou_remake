# 对抗审查：platform-contract 四单元 TDD 实现复审

> 审查人：adversary。对象：game-platform 四契约单元实现
> （a859408 Storage / 485f542 ProcessControl / cb86eb2 AudioOut / efb29bb AssetSource）。
> 审查方式：§7 (iii) RED 复现抽查 + 四点实现面复核（均独立复跑/逐行核对，非转引）。

## 判定：**通过**（RED 抽查真实、27 测独立复现全绿、四点实现面无越权）；3 条备忘不卡闸

## 1. TDD 真实性抽查（A 协议 (iii) 执行）

- **RED 复现（485f542，独立执行）**：`git archive a859408`（父树）导出 game-platform 至
  temp + 只叠加 485f542 的 tests/process_control_contract.rs（archive 字节级拷贝）→
  `cargo test --test process_control_contract` ⇒
  **`error[E0432]: unresolved import game_platform::ProcessControl … :12:5`**——
  与其 message 逐字吻合。**红证据真实**。（temp 独立目录复现，不触共享工作区/索引。）
- **GREEN 复现（efb29bb 终面，独立执行）**：archive efb29bb → `cargo test` ⇒
  **27/27 全绿**（lib 5 + storage 7 + process_control 5 + audio_out 6 + asset_source 4）
  与声称"5+22"精确一致，EXIT=0。
- 四 commit 均含 (i) RED 逐字 (ii) 一致性声明 (iii) 复现指令 + Evidence/Confidence ✓
  （message 抽读 a859408 全文、余三件格式同款）。

## 2. 三对账（内容/声称/写范围，G5 同款核对项）

| commit | 内容面 | 声称 | 写范围 | 结论 |
|---|---|---|---|---|
| a859408 | 2 文件（lib.rs + storage_contract.rs） | Storage 单元 | crates/game-platform | ✅ |
| 485f542 | 2 文件（lib.rs +17 / process_control_contract.rs +86 = 103） | ProcessControl 单元 | 同上 | ✅ |
| cb86eb2 | 2 文件（lib.rs +21 / audio_out_contract.rs +93） | AudioOut 单元 | 同上 | ✅ |
| efb29bb | 2 文件（lib.rs +16 / asset_source_contract.rs +62） | AssetSource 单元 | 同上 | ✅ |

无卷入面（pathspec 时代），链 efb29bb→cb86eb2→9f63a63→485f542→a859408 核实。

## 3. 四点实现面复核

| # | 复核点 | 结论 |
|---|---|---|
| ① 签名与 v1.0 spec 一致 | `load_all/persist`（lib.rs:133-143）、`request_exit(&mut self)`（:153-160）、`AudioOut: Send + write/idle`（:176-181）、`read(rel)->Result<Vec<u8>,String>`（:190-197）、常量 RMS_RECORD_ID/RMS_STORES/SAMPLE_RATE/CHANNELS + rms_key/rms_store_name（:111-125） | ✅ 形状逐字一致；lib.rs 只 trait+类型+纯函数（AGENTS §2 红线）✅，测试替身全在 tests/ |
| ② 负形状锁非顺手放宽 | "无 'static/无 Send"锁（process_control_contract.rs:56-80，Rc 载体证非 Send 可编译）锚 spec §2.2"无 'static/生命周期约束"+trait 无 supertrait；AudioOut 侧反而有**正** Send 锁（:78-84，spec §3.2） | ✅ 契约要求的锁，非放宽；防未来加约束挤掉 winit 作用域适配器 |
| ③ RMS_STORES 无越权 | 键域 = a.java:4452 "SKY_WAR"/8960 "MOT_IF"/9190 "MOT_L"+0..=5 八店（前轮已核 A 级）；实现侧无第二套白名单 | ✅ |
| ④ 替身语义 = spec | 域外键丢弃不 panic（storage_contract.rs:81-83，§1.1）；Err 携人话诊断不 panic（asset_source_contract.rs:32-37，§4.2；audio_out_contract.rs:51-56，§3.1 Err=不可恢复）；idle/幂等语义测试齐 | ✅ |

## 4. 备忘（不卡闸）

1. **message BOM**：四 commit subject 均带 UTF-8 BOM 前缀（`\ufefffeat(platform):`，
   PowerShell 捕获产物）——前科已登 git-audit G5；后续 commit 用 `git commit -F` 或
   去 BOM 捕获。
2. **两个操作坑入库协调**：platform-contract 自报"Cargo mtime 缓存陷阱（Copy-Item 还原
   旧时间戳 ⇒ cargo 链旧 rlib 假红）"与"Windows 并发 index.lock/LNK1104（锁活时禁清）"
   待入库 gotcha——请落库；其中锁一条建议**并入**
   `gotcha_shared-workspace-bare-commit-sweeps-index`（同族，防双条目口径漂移），
   mtime 一条独立成条。我 temp-archive 复现法天然免疫 mtime 坑（导出时间戳=当下）。
3. spec §7 TDD 记录表/§8 收口回填后我做文本级终核（一轮）。

## 5. 证据分级声明

§1/§2/§3 全部 A 级（复现输出/commit 实测/逐行锚亲验）；无假设区条目。
