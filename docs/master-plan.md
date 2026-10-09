# 《囧囧西游-大闹天宫》重制工程总体方案

> 状态：已定稿（2026-10-09）。仓库结构与红线以 [AGENTS.md](../AGENTS.md) §2 为准。原始 JAR 只读副本：
> `/home/peng/.dsh/attachments/v1/files/76/764cf01e.../囧囧西游-大闹天宫.jar`
> 工作副本：`analysis/game.jar`

## 0. 已锁定决策

| # | 决策项 | 结论 |
|---|---|---|
| D1 | 目标语言 | **Rust**（跨平台：桌面 + wasm，预留移动端）。淘汰 C++；TS 不采用 |
| D2 | 路线 | 两阶段：**Stage A 像素级一致** → **Stage B 现代化改造**。A 阶段预留扩展接口（分辨率/输入/字体/素材/音频） |
| D3 | 中间产物 | **参考版 Java 必须存在**：bytecode → 反混淆 Java（可编译、可运行、可追踪），作为全部差分验证的锚点，永不删除 |
| D4 | Bug 政策 | Stage A **保真保留原 bug 行为**，只登记到 bug 台账（`docs/bug-ledger.md`），Stage B 统一修复 |

## 1. 工程总路线

```
P0 资产盘点 → P1 反混淆(参考版Java) → P2 理解逻辑(规格书) → P3 Rust改写 → P4 差分验证
                     ↑__________________________________________________|
                          验证锚点：参考版 Java 全程存活
```

一致性核心思想：bytecode→Java 有反编译器背书，搬运风险全部集中在 Java→Rust 这一步，
因此用**机器差分测试**（状态 trace + 截图 diff）当裁判，不靠人肉 review。

## 2. Rust 架构与扩展接口（Stage A 留给 Stage B 的缝）

```
crates/
  game-data/       资源解析：packed PNG / maplv 网格 / sprite / script DSL（纯解析，无渲染）
  game-core/       纯游戏逻辑：状态机 / 脚本解释器 / 战斗 / 移动 / 道具（无 I/O、确定性）
  game-platform/   平台 trait：Clock / GameRng / Renderer / Audio / Storage / Input / FontProvider
  game-desktop/    winit + pixels（软件帧缓冲 → 整数倍缩放）
  game-wasm/       wasm-bindgen 后端
  game-oracle/     差分测试宿主：跑 core，产出 trace/状态快照/截图，与参考版比对
tools/
  extract-assets/  JAR → 原始资源 + 转换管线（.mid → .ogg）
  rename-pipeline/ P1 命名管线（javap/调用图分析 + LLM 命名 + JavaParser 符号改名）
  trace-diff/      参考版 trace vs Rust trace 比对器
reference/         参考版 Java + 自研 MIDP/Nokia shim + 追踪钩子（D3 产物）
```

扩展缝（Stage A 实现为 trait/常量，Stage B 只换实现）：

| 缝 | Stage A | Stage B |
|---|---|---|
| 分辨率 | 逻辑画布固定 240×320，整数倍缩放 + 黑边 | 自由缩放、宽屏 UI 重排 |
| 输入 | J2ME 键码 → 抽象动作（Move/Confirm/Cancel/Menu/Hotkey5…） | 手柄/触屏映射 |
| 字体 | 忠实字体方案（见风险 R2） | 现代字体/字号/排版 |
| 素材 | 原始像素素材 | 高清替换（data 层换实现） |
| 音频 | MIDI 离线转 OGG 还原播放 | 重制音轨 |
| 存档 | 兼容原 RMS 格式镜像 + JSON | 云存档/多槽位 |
| 脚本 DSL | 原指令集解释器（AST 化） | 可加新指令/关卡编辑器 |

## 3. 阶段与门禁

### P0 资产盘点（≈90% 完成）
- 产物：`docs/findings/jar-forensics.md`（资源格式考证）
- 残留：`sprite*` 帧格式、PNG 2 字节头含义、`maplv` 双网格语义 → 归入 P2 用代码考证

### P1 反混淆（参考版 Java）
1. CFR + Vineflower 双反编译交叉验证（CFR 已完成，见 `analysis/decomp/a.java`）
2. 语义命名管线：调用图角色标注 → LLM 逐方法命名 + 证据台账 → 人工抽查 → 符号改名
3. 自研 MIDP/CLDC + Nokia UI API shim（Java 实现，headless、虚拟时钟、固定种子、trace 输出）
   —— 双重作用：编译目标 **和** 差分测试 oracle（规避第三方模拟器对 DirectGraphics 兼容性风险）
4. 可选第二轨道：J2ME-Loader/MicroEmu 跑原版截图作视觉 golden
- **Gate**：参考版可完整游玩，主流程截图与原版一致，trace 可稳定复现

### P2 理解逻辑（规格书）
- 产物：`docs/spec/` —— 资源格式 spec、脚本 DSL 语法与指令语义表、游戏状态机、
  玩法规则（移动/战斗/道具/封印门/隐藏墙）、数值表、每模块行为契约
- **Gate**：不看源码能讲清"第 N 层怎么过、每条脚本指令什么效果"

### P3 Rust 改写
- 顺序：game-data 解析器 → game-core 状态机/脚本解释器 → 玩法规则 → 渲染 → UI/剧情 → 平台壳
- 每模块落地即配契约测试；数值常量逐条对照规格书
- **Gate**：55 层脚本全部可解析执行，游戏可通关

### P4 差分验证（金字塔）
- L1 资源级：解析器 golden test（55 maplv + 55 sprite + 全部 packed PNG 全量）
- L2 指令级：script DSL 双实现对照（同指令流 → 状态快照逐项比对）
- L3 帧级：同输入序列 + 虚拟时钟 + 固定种子，逐 tick 比对状态向量（坐标/HP/道具/模式/脚本PC）
- L4 视觉级：截图 diff（文本区域白名单，见 R2）
- L5 流程级：全关卡自动跑、存/读档往返、边界行为（摔死/战斗结算/死亡）
- **Gate（= Stage A 完成）**：L1–L3、L5 全绿，L4 除白名单外像素一致

## 4. 范围裁剪（需按 D4 精神记录）

| 子系统 | 处理 | 理由 |
|---|---|---|
| 雪鲤鱼平台登录/注册/短信充值/查询/客服 | **整体剔除**，相关代码记入 scope-decisions | 服务已死，且原版"试玩"模式即离线可玩，游戏本体逻辑不受影响 |
| HttpConnection 联网 | 剔除 | 同上 |
| MMAPI 播放 `.mid` | 转换后播放 OGG | 运行时不引入 MIDI 合成器 |

## 5. 风险登记

| ID | 风险 | 对策 |
|---|---|---|
| R1 | Nokia UI API 在第三方模拟器兼容性差 | 自研 shim 轨道为主（P1.3），模拟器轨道为辅 |
| R2 | 原版用 MIDP 系统字体渲染中文，字形/度量无法与现代字体像素一致 | 考证原机型字体度量 → 尽量位图化还原；L4 文本区域白名单；Stage B 换现代字体 |
| R3 | busy-wait 帧节奏绑定真机速度 | 虚拟时钟 + 固定步长，双端同构 |
| R4 | 原版存在 bug，差分测试会全部暴露 | D4：保真保留，BUG-xxx 台账，代码内标注引用 |
| R5 | `sprite*`/PNG 头等格式未完全考证 | P2 用参考版代码+运行时插桩考证 |
| R6 | RNG 用 currentTimeMillis 播种 | 双端统一 GameRng trait，验证模式固定种子 |

## 6. 里程碑

- M0 资产与格式考证（进行中，≈90%）
- M1 参考版 Java 可运行 + trace 输出（P1）
- M2 规格书齐备（P2）
- M3 Rust core+data 移植完成（P3）
- M4 差分验证通过 = **Stage A 完成（像素级一致）**
- M5 现代化改造（Stage B，含 bug 台账清偿）
