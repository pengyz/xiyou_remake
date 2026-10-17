# 对抗审查：task-7「BUG-021 修复」实现复审（b587d11 + 78ac8cf）

> 审查人：adversary。对账基准：2026-10-17-mot-if-truncation-fidelity.md §7（三硬门）。
> 手段：代码逐行核对 + RED 独立复现（temp-archive）+ GREEN 独立复跑（真实工作区 + 隔离树双路）。

## 判定：**通过**（三硬门全过、RED 真实、GREEN 双路复现、三对账干净）；3 条备忘不卡闸

## 1. RED 抽查（独立复现）

temp-archive 法：`git archive fdba990`（父树）+ 只叠 b587d11 的
tests/mot_if_truncation.rs ⇒ `cargo test -p game-core --test mot_if_truncation` 得
**E0308 + E0599(take_parse_dirty/reset_entry_layout) + E0600** ——与其 message 逐字
同族（其 :120/:121/:162/:183/:203/:221 行号引用与输出一致）。**红证据真实**。
78ac8cf 的"回放法"红（摘 logic_tick 接线行 ⇒ NotFound panic 逐字入文）技术路线不同
但诚实可复核，接受。

## 2. 三硬门逐条（§7 对账基准）

### ① 字段级粒度·同字节停 —— ✅ 通过

menu_family.rs:734-777 逐行核：
- valid 字节：界检 `pos >= len` ⇒ break **不赋值**（旧值保留，Java readByte EOF 语义）；
- u8 字段：界检通过才 `self.$arr[k] = data[pos]`（:749-754）——RHS 失败不赋值 ✓；
- i32 字段：`pos+4 > len` ⇒ break 不赋值（:759-761）；4 字节整段原子提交 ✓
  （Java readInt 读 1-3 字节抛 ⇒ 整字段不写）；
- 字段序 floor/weapon/armor/hp/atk/def/钥×3/gold = a.java:8986-8995 逐字同序 ✓；
- break 即整体停（后续字段/后续槽留旧）= Java 异常弃解析 ✓；`!ok` 只置 parse_dirty
  不清表（:778-780）✓。
- 夹具佐证：`field_spans()` 静态偏移表 + `expected(len, base)` 按**字段跨度**导出期望
  （tests/mot_if_truncation.rs:49-66）——期望本身是字段级粒度，非整槽 ✓；marker 基态
  断言"未及字段留 marker"（:127-149）✓；L=31 命名回归（`&b[..31]` 不得 panic，:153-162）
  = 旧 panic 窗精确钉住 ✓。

### ② 四复位面全集 —— ✅ 通过

| 复位面 | 现状 | 核 |
|---|---|---|
| menu_family.rs:728-731（None/空） | 只置 parse_dirty，**不动数组** | ✅ |
| menu_family.rs:778-780（!ok） | 只置 parse_dirty，**无全清** | ✅ |
| scene.rs:3485-3487 enter_menu7 | `reset_entry_layout()` + load，复用 self.slot | ✅ |
| paint.rs:642-644 enter_menu_mode case8 | `self.slots.take().unwrap_or_else(new)` 复用既有对象 + reset_entry_layout + load | ✅（BootMachine 侧含） |

reset_entry_layout 只动度量+cursor/scroll（:669-688，锚 a.java:4354-4366）不动数据数组 ✓；
`SlotSelect::new()` 仅构造用（= Java !f_bool_15 首分配）✓。

### ③ ②c 回写环终态等价（含重复进入收敛）—— ✅ 通过

78ac8cf 核：
- 触发源双覆盖：`Screen::Game(sc).slot` 与 `Screen::Boot(m).slots` 都 take_parse_dirty
  （含 Boot 读档屏——Java case 7/8 共享入口的对偶面）✓；
- 回写 = `serialize_slot_if` 整表 → MOT_IF@1（retain+push+rms.persist）——单记录店
  覆盖写 ≡ Java delete+addRecord 终态（契约 §1.3 edge-d 等价性直接适用）✓；
  序列化序 valid/floor/weapon/armor/7×i32 BE = a.java:9034-9050 逐字同序 ✓；
- 重读源同步：stores（Boot 列表）+ sc.preset_mot_if（Game mode7/8，BUG-018 环）✓；
- 一次性触发（mem::take）+ logic_tick 尾接线 + "二拍不重写"测试锁 ✓；
- 收敛：回写产物恒合法表（6×(1|32)B）⇒ 下次解析必成功、不再置脏 = Java 写后可解析
  同收敛 ✓。
- 时序差异注记（非漂移）：Java 在 catch 内同步写、Rust 在同一 tick 的 logic_tick 尾写
  ——同一 tick 内落盘，盘面观测等价。

## 3. GREEN 独立复现（双路）

- **真实工作区** `cargo test -p game-core` ⇒ **全绿**（lib 89 + combat 6 + entity 12 +
  **mot_if_truncation 6** + mov_route 4 + render 16 + script 14 + slot_equip 2 +
  walk 4 = 153，EXIT=0）——"89+6+12 零回归 + 6/6 新夹具"精确兑现；
- **隔离树**（temp-archive 78ac8cf + 拷贝 assets/raw 与 oracle/scenarios）
  `cargo test -p game-desktop` ⇒ **27/27 全绿**（含行为锚 + 新回写环测试）。
- 【方法注记】temp-archive 首跑 `-p game-core --lib` 有 5 个 NotFound 失败——
  **我方方法环境伪差**（game_view.rs:208 等读 gitignored `assets/raw/maplv1`，archive
  不含）非回归；换真实工作区复跑即绿。temp-archive 法适用于无 gitignored 依赖的面。

## 4. 三对账 + TDD 三件套

| commit | 内容 | 声称 | 写范围 | 结论 |
|---|---|---|---|---|
| b587d11 | game-core 3 src + tests/mot_if_truncation.rs（4 文件 335+/68-） | BUG-021 ①② | task-7 scope 内 | ✅ |
| 78ac8cf | shell/win/src/main.rs 1 文件 65+ | BUG-021 ②c | task-7 scope 内 | ✅ |

两 commit 均携 (i) RED 逐字 (ii) 一致性声明 (iii) 复现指令 + Evidence(A/B 分级)/Confidence ✓。
放大件 3 取 (b) 登记已知边界 + 理由（"交接面不动，未见反例前不加行为"）——与我倾向一致
且理由已入 commit 与台账 ✓。

## 5. 备忘（不卡闸）

1. **38→27 计数迁移**：eabb815 抽取把 audio 面测试迁 shell/common（现 13 个）——
   game-desktop 计数变化属 shell-port 抽取面，归 task-3/5 复审对账（届时核迁移测试
   断言零字节 + shell/common 单源）。
2. 工作区有并行未提交 main.rs 改动（shell-port 迁移 WIP）——与 task-7 无涉，抽取
   复审时核。
3. serialize_slot_if 对 Boot 侧回写走 `slot.clone()`——语义正确，量大时无碍（6 槽）。

**BUG-021 台账**：已按 lead 授权翻「已修复」（引 b587d11/78ac8cf）。
