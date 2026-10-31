# 双操控方案 壳层功能查询 / 工具栏 / 手势 / 设置 Implementation Plan（计划 3 / 3）

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development 或 executing-plans 逐任务实施。步骤用 checkbox 跟踪。
> 每任务循环：RED → GREEN → 变异 → 门禁 → 对抗复核子代理（附录 A 模板）→ 修复意见 → 提交。

**Goal:** 落地 spec（`docs/superpowers/specs/2026-10-29-dual-control-schemes-design.md`）§3 功能查询、§4 布局与工具栏、
§5.3 手势、§7 撤销呈现、§8 设置 / 提示卡 / 帮助，并按 §9 验收（一致性测试为裁判）。计划 2（core 点地规则）已全部合入。

**Architecture:** core 新增纯查询 `controls()`（情境 → 钮集合，每钮 `FunctionId` + `Action`）；壳（android 为主，win 只接新查询不改呈现）
据查询画工具栏 / 标注摇杆软键；手势分类在 shell-common（dp 阈值），执行走 core 既有入口（swipe_step / 观察 wp2 平移同式 / 排队）；
设置行增减走 core 新入口 `replace_shell_rows`（BootMachine 与 GameScene 各一份，交接两向）。classic 按键链、oracle 回放零变化。

**Tech Stack:** Rust（cargo test --offline），Python 3（gates）。

**范围外 / 待裁定：**
- ~~战斗中画面绘制的"攻击 / 撤退"按钮触屏直点~~（2026-10-31 已裁定：**不做**——战斗自动回合制、按钮为原版装饰，
  做可点=发明新交互且扩大误触面；教学误导由 T7 帮助文本说明。裁定记录见 spec §3.2 战斗行注）。
- 路线预览、手把手教程、掉血开关（spec §10 不做）。
- Windows 工具栏（Windows 保留现有可见撤销钮与鼠标点地，spec §2.2/§7）。

**通用约定（同计划 2）：**
- 纪律 AGENTS.md；commit 含"动手前三问"+ `Evidence:` / `Confidence:`。
- mtime gotcha：改源码后 `sleep 2; touch <文件>` 再跑测试。
- 全量：`cargo test --offline --no-fail-fast -p game-core -p game-shell-common -p game-android -p game-desktop -p game-oracle` 0 失败。
- 每任务 GREEN 后派独立复核（附录 A），记录 `target/verify/review_plan3_T<N>.md`，认同意见修完再提交。
- ratchet 更新统一在末任务单独 `gate(baselines)` 提交。
- 方向码：0 下、1 上、2 右、3 左。

---

### Task 1: core 功能查询 `controls()`——情境判定与钮集合（spec §3.1/§3.2 地图/对话/浮层行）

**Files:** 新建 `crates/game-core/src/controls.rs`（+ lib.rs 注册）；测试在本模块。

- `FunctionId` 枚举（每钮一个：Bag / Observe / Undo / CloudUp / CloudDown / Menu / Stop / Cancel / Fight / Prev / Next / Continue /
  ScrollUp / ScrollDown / Use / Back / Buy / Accept / Refuse / Close / Alchemy / Save / Read / Skip…，按 §3.2 表逐行落）。
- `Action` 枚举：`Key(i32)` / `Stop` / `Cancel` / `Fight` / `StopThen(i32)` / `Undo`。
- `Controls { left: Option<(FunctionId, Action)>, mid: Vec<(FunctionId, Action)>, right: Option<(FunctionId, Action)> }`。
- `GameScene::controls(&self) -> Controls`：情境由 core 状态派生（§3.1 口径），**无副作用**。
- RED：§3.2 的地图族行（空闲 / 手动行走 / 点地寻路中 / 已选中怪 / 观察含回锁期 / 战斗中）与对话 / 提示 / 确认 / 报价各族
  至少各一夹具断言钮集合与 Action（期望由 spec 表**手写**，不由实现推导）；观察回锁期（wp2 退出后窗口）单独一测。
- 变异：任一情境判据取反 / 删 → 对应行红。
- 复核重点：§3.2 每行键码逐臂对 tick 臂源码核对（plan 逐臂核对条款）；kind 1 吞键子类（scene.rs:4746-4755）按实际接受键列钮。

### Task 2: `controls()` BootMachine 侧 + 停下后打开 StopThen（spec §3.2 页面行 / §5.2 共用排队）

**Files:** `controls.rs`（BootMachine 查询或共用函数）；`scene.rs`（`PendingTap` 扩 `StopThen(i32)`：排空落 Map wp0 后发键，失效条件同排队）。

- RED：页面族（菜单 4 / 存档 7 / 设置 16 / 帮助 15 / 图鉴 5 / 背包 10 / 炼丹 9 / 读档 8 / 关于 17 / 片头 / 标题隐藏 / 遮幅全隐藏）+
  `ending_done` 前后；StopThen：寻路中"背包 / 菜单"排队、排空后开页、按键 / 浮层 / 剧情作废（复用 T7 失效面）。
- 复核重点：StopThen 与 clear_tap_state / press_key 的交互；炼丹页 −6 与离开 −7 双键。

### Task 3: 设置字段与设置行（spec §8）

**Files:** `shell/common/src/settings.rs`（AppSettings 增 `control_mode` / `auto_battle` / `tips_seen_pad` / `tips_seen_touch`）；
core `menu_family.rs` / `boot`（新入口 `replace_shell_rows(rows)`：同改 shell_options 与当前 options、盒高变、高亮保持在操控方式行）；
`handoff.rs`（shell_options 交接两向）。

- RED：坏值回默认；行增减（切触屏 ⇒ 自动战斗行出现）；`replace_shell_rows` 两机（Boot + GameScene）；交接两向（a→b / b→a）；
  行缺失时 Runtime 字段保留（`app_settings()` 不回默认）。
- 门禁：settings 既有 roundtrip 测试不回退。
- 复核重点：§8 同步规则逐条；盒高 / 高亮行为；L1 落盘拍。

### Task 4: 手势分类（点 / 滑 / 拖）与 Android 视口 Move 接入（spec §5.3）

**Files:** `shell/common/src/touch.rs`（`GestureTracker` 扩拖动判定：超阈且未松手持续 = 拖动；阈值 dp 定值并注明）；`shell/android/src/native.rs`
（视口 Move 事件：滑 → `swipe_step` 排队 / 空闲探步；拖（观察 wp2）→ 相机直写 modern 专用入口 + 主方向改 facing，钳位同 wp2 平移臂
scene.rs:4927-4963 同式；滑 / 拖不产生 Up 点按）。

- RED：阈值分类（dp 边界 ±ε）；滑动不产生点按；观察模式拖动改 cam / facing / 点按无效；寻路中滑 = 排队（复用 host 链）。
- core 若需观察平移入口（`pan_viewport(dx, dy)`，modern 专用、classic no-op）则先在 core 加纯入口并单测。
- 复核重点：拖动与 wp2 回锁期交互；阈值在 dpi 缩放下的取值；Move 注入对 guard（人工拖动检测）的影响——冒烟 guard 须能区分注入与人工。

### Task 5: 触屏档布局与工具栏呈现（spec §4.1/§4.2，android portrait）

**Files:** `shell/android/src/portrait.rs`（触屏档：V_t = 内容区高 − 64dp − G；s_t 取 [s_read, min(4,s_boot)] 内满足 ⌊W/32s⌋≥8 ∧
⌊V_t/32s⌋≥10 的最大 s，无满足者回退现公式）；新 `toolbar.rs`（绘制 + 命中：左右各 1/3、中间 ≤4 钮均分；不可用钮不画不可点；
按下执行 `Action`；沿用按键震动 / 按键音开关）；native.rs 接线（control_mode = 触屏时用触屏档 + 工具栏，摇杆档不变）。

- RED：三档窗口表（§4.1 三个估算窗 + 公式钉测试）；HUD 最窄宽（8 列）截图无重叠（重叠 ⇒ C_min 上调并记偏离）；
  隐藏钮不可点；按下走 Action（键码不固定 −6/−7）。
- 门禁：`gates visual`（classic 不受影响）；modern 帧仅 android 层变。
- 复核重点：视口高度变化对相机 / scene 档的影响；工具栏与手势区边界；65 行情境在呈现层的抽样。

### Task 6: 摇杆面板功能标注（STAGE-B-007，spec §3.3）

**Files:** `shell/android/src/portrait.rs`（panel 绘制）+ `native.rs`：摇杆模式左右软键位标注 `controls()` 左右 FunctionId 的功能名；
不可用灰显但照发 −6 / −7；方向 / 确定键不标注；portrait 不画画布内脚注图标。

- RED：标注随情境切换（地图 / 对话 / 报价各一）；灰显仍发键码；两键码效果不同时（如对话"继续"= −5）不标注。
- 复核重点：与 §9 双键码实跑的一致性挂钩。

### Task 7: 提示卡与帮助页（spec §8）

**Files:** `shell/android/src/native.rs`（提示卡：Android ∧ 该模式 tips_seen=false ∧ 首次可操作地图（Map wp0 ∧ 无浮层 ∧ 非遮幅）⇒ 壳调
core 显示提示（kind 0 原版框，一页），弹出即置位落盘）；core `scene.rs` / `paint.rs`（帮助文本覆盖入口接 BootMachine 两读点
paint.rs:844/897 与 GameScene 读点 scene.rs:3988，未覆盖显示原文）；壳按模式提供帮助文本（触屏文案含"战斗自动进行，画面中的攻击 / 撤退按钮为原版装饰"——2026-10-31 裁定）。

- RED：首弹条件与置位落盘；重启不再弹（持久化）；切换模式后另一模式的卡再弹；帮助覆盖两机 + 未覆盖回退；
  Windows 与 classic 永不弹。
- 复核重点：提示弹出与"首次可操作地图"判定的拍序（交接后第一拍）；读档 / 剧情接管期间不弹。

### Task 8: 撤销呈现收口（spec §7）

**Files:** android：摇杆模式去掉右上隐形热区（`undo_zone_hit` 不再被 android 分类调用）；触屏模式工具栏中间「撤销」钮
（地图空闲与已选中怪情境，来自 `controls()`）。Windows 保留现状。

- RED：摇杆模式点右上原热区 = 无动作（视口点按语义）；触屏工具栏撤销钮可点且减栈（复用 undo_last）。
- 复核重点：control_mode 切换后两种呈现互斥；触屏模式下热区是否保留（spec 只说摇杆去掉——触屏走工具栏，热区也应去，按 spec §7
  "触屏模式：工具栏中间撤销"执行，两模式皆不再用隐形热区）。

### Task 9: 一致性验收测试（spec §9 裁判，GameScene 面）

**Files:** 新 `crates/game-core/tests/controls_acceptance.rs`（或扩 controls.rs 集成测试）。

- 每个 `FunctionId` 独立后置条件谓词（如 Back ⇒ game_mode 上级；Stop ⇒ 路线打断；Fight ⇒ 开战）。
- 实跑：同一确定性夹具两份（非 Clone），一份执行 Action 实跑 tick（必要时每拍 paint），比对快照状态域（排除 key_value/key_held/反馈计时）。
- **双键码**：带标注功能同时实跑工具栏键码与摇杆物理键码（−6/−7），同谓词。
- 情境覆盖清单：§3.2 每行至少一夹具（kind 4 各子类、kind 1 吞键子类、ending_done 前后、炼丹、观察回锁、排队中）。
- 变异：隐藏钮画 / 点 / 发键任一 ⇒ 红。
- 复核重点：期望独立性（不得由 controls() 推导期望）；谓词本身逐一审。

### Task 10: BootMachine 面一致性 + 壳接线验收（android/win）

**Files:** `shell/android/tests/`（触屏布局三档、隐藏钮、设置持久化（切摇杆再切回自动战斗保留、重启不弹卡）、中途切模式
（地图中 / 设置页中 / 面板确定键切换 ⇒ key_held 归零无误点））；win：确认点击行为既有测试仍绿（新规则已在计划 2 接入）。

- 复核重点：切模式的 TouchMapper 重建（按住状态清零）；壳读行结果而非直写字段（§8 同步规则）。

### Task 11: 收口——偏离登记 / 台账 / ratchet / 实机冒烟

- `docs/spec/stage-b-deviations.md` STAGE-B-007/008 计划 3 部分；spec 执行期更正原地注；
  `gates all`（对照 status 无新增 SKIP）+ ratchet 单独提交；实机（`c362a15b` 或经裁定的替代机）冒烟：工具栏各情境、
  滑 / 拖、设置切换、提示卡、帮助、撤销钮、切模式无误点；七项回归（计划 2 冒烟项不回退）。

---

## 附录 A：复核提示词模板（同计划 2）

派 general-purpose 子代理：仓库 /home/peng/workspace/xiyou_remake；任务 <T 名>；改动 = git diff（未提交）；
设计依据 spec §<节>；只读 + 唯一可写 target/verify/review_plan3_T<N>.md；核查 ① spec 条目全覆盖（逐条反例）
② classic 零触达（调用面）③ 测试反证（变异）与期望独立性 ④ 相邻机制回归（BUG-038/039/040/043/044/045/047/048 钉）。
输出意见表（编号|严重度|意见|证据|建议修法）+ 已确认清单。
