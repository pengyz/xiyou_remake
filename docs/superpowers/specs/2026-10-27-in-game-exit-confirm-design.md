# 游戏内右滑退出确认框——设计件（STAGE-B-005）

> 【2026-10-27 更正·已被取代】按本件实施的游戏内 kind 4 样式版经用户实机评审否决（"视觉效果不好…直接用Android标准
> 关闭UI,作为shell层实现"），代码已撤回（92e1dfb）。现行 = 壳层 Android 标准 `AlertDialog`（26a4bf0；交互范围 Q1 地图态、
> Q2 确定即退出、Q3 未保存提醒不变；Q4 实现层改为壳层、游戏核心零改动）。详见 docs/spec/stage-b-deviations.md
> STAGE-B-005 更正注。本件保留作历史记录。

> 2026-10-27。用户诉求："游戏内右滑弹出退出确认菜单，样式就用游戏内弹框样式"。逐项裁定：
> Q1 触发范围 = **A 只在地图行走态**；Q2 确定后 = **A 直接退出进程**；Q3 文案 = **A 加未保存提醒**；
> Q4 实现层 = **方案 2 放进游戏核心**（独立模态状态，壳调用打开、核心发确认信号）。设计全文经用户
> "没有，继续推进"认可。

## 0. 背景事实（证据）

- 原版**没有**退出确认框：`f_String_arr_16`（联网/计费文本表，a.java:2254-2314）里的
  `"是否确定退出游戏？"`（a.java:2298）全文件无消费点（该数组仅 a.java:10796 `[15]` / 10863 `[48]` 取用）。
  本件第一行文案取自该原版字面量。
- 原版二选一弹框 = 提示浮层 kind 4（商人/告示牌，`show_overlay_message(4, text, 1, 2)`，scene.rs:5294/5303）：
  ui[0] 九宫边框居中（`menu_family::paint_box_frame`）+ 白字换行（`menu_family::paint_wrapped_lines`）+
  左下 ui[11] 图标 1、右下图标 2（paint 公共尾 a.java:3129-3168 / scene.rs `paint_overlay_box`）；
  键：-6/-5 接受、-7 拒绝（overlay 键分派 case 4，a.java:3308-3318）。
- ui[11]（36×10，三枚 12×10）实物：图标 1 = ○（确定）、2 = ×（取消）、3 = ↩（返回）。
- 游戏内现状：BACK/右滑经 input.rs 恒译 -7；地图上 -7 = 打开游戏内菜单（a.java:3553-3558）。面板右软键
  同发 -7——两者只在 native.rs `handle_input` 的 BACK 分支可区分。
- 无自动存档：退出进程丢失上次手动保存之后的全部进度（原版游戏内"退出" kind 7 同样不存档，a.java:4757-4769）。

## 1. 交互规格

| 项 | 规格 |
|---|---|
| 触发 | 游戏场景活动 ∧ `game_mode == Map` ∧ 原版浮层未开（`!overlay_active`）∧ 非换层遮幅（`!transitioning`）时，**系统 BACK / 边缘返回手势** ⇒ 打开确认框；该 BACK 不再投 -7 |
| 不触发 | 其余游戏内界面 BACK 照旧 = -7（菜单返回 / 背包返回 / 存档页返回 / 对话推进…）；面板右软键（-7）在地图上照旧打开游戏内菜单 |
| 框内键 | -5（确定键）/ -6（左软键）= **确认** ⇒ 关框 + 退出信号；-7（右软键 / 再次 BACK）= **取消** ⇒ 关框；其余键吞掉无效果 |
| 框内触屏 | 画面区点按不生效（不寻路、不探步、不撤销） |
| 文案 | `是否确定退出游戏？`（a.java:2298 原文）+ 换行 + `未保存的进度将会丢失`（新增）；全白字 |
| 样式 | 同原版 kind 4 文本浮层：布局 `layout_wrapped_text(text, OVERLAY_TEXT_W, OVERLAY_TEXT_MAX_H)`；盒宽 = content_w + 32、盒高 = 32 + 可见行 × (字高 + 4) + 18（有图标）；画布居中；左下图标 1（○）、右下图标 2（×）；软键 (1,2) |
| 冻结 | 框开期间游戏逻辑冻结（tick 头拦截即返回：行走/步进/战斗/实体帧/剧情全停）；paint 驱动的装饰态照旧（视差滚动、bob、飘字、开门动画）——与原版浮层同口径 |
| 切后台 | 框开时 hideNotify ⇒ 先关框（= 取消）再走原版 hide_notify（Map → 游戏内菜单 + 关音） |
| 确认后 | 壳置退出请求 ⇒ 现有退出链（不再出拍 → `ANativeActivity_finish` → 销毁后 `_exit(0)`，BUG-036 同进程实例计数照旧） |

## 2. 游戏核心（crates/game-core/src/scene.rs）

- **状态**（独立于原版 `overlay_*`，不冲掉已开的原版浮层）：
  `exit_confirm: Option<ExitConfirmBox>`（布局结果：行、总行数、每页行数、页高、行高、盒宽高）+
  `exit_confirmed: bool`（确认信号，粘滞至被取走）。
- **接口**：
  - `pub fn open_exit_confirm(&mut self) -> bool`：满足触发条件且未开 ⇒ 布局 + 打开，返回 true；否则 false。
  - `pub fn exit_confirm_open(&self) -> bool`。
  - `pub fn take_exit_confirmed(&mut self) -> bool`：取走确认信号（取后清零）。
- **tick**：首行 `frame_counter` 赋值后、原版 overlay 拦截**之前**：框开 ⇒ 按 §1 键表处理 → 清
  `key_value`/`key_held` → return。
- **paint**：只在 Map 臂（框只在 Map 打开，开期 tick 冻结 ⇒ mode 不变）：软键赋值改为
  `框开 ? (1,2) : (1,3)`（原 `(1,3)` 恒等于框关），原版浮层之后、遮幅尾之前画确认框。
- **单源**：浮层盒尺寸公式与底角图标几何抽为纯函数，原版 `show_overlay_message` / `paint_overlay_box`
  与确认框共用（等价重构：op 序列不变，由 classic 帧夹具与 Modern 帧基线兜底）。
- **hide_notify**：先 `exit_confirm = None` 再原逻辑。
- **门控**：无预设门控——仅壳调用 `open_exit_confirm` 才可达；classic 回放/夹具/差分不调用 ⇒ 零偏离。
- **注释**：全部新增点标 `// STAGE-B-005`。

## 3. Android 壳（shell/android）

- `runtime.rs`：
  - `pub fn back_opens_exit_confirm(&mut self) -> bool`：场景活动 ∧ 框未开 ⇒ `sc.open_exit_confirm()`。
  - `step_scene`：框开 ⇒ 丢弃本拍画面点按（不调 `scene_tap`）；tick 后 `sc.take_exit_confirmed()` ⇒
    `shell_exit = true`（日志 `exit confirm → 立即退出`）。
  - `overlay_state()`：Map ∧ 框未开 ⇒ Game，否则 Ui。
- `native.rs handle_input` BACK 分支：根界面 ⇒ 立即退出（BUG-034）；否则 `back_opens_exit_confirm()` ⇒ 丢弃该
  BACK 的 -7（日志 `back in game → 退出确认框`）；框已开 ⇒ BACK 照旧 -7（= 取消）。
- 面板软键图标：随 `rt.softkeys()`（核心 paint 置 (1,2)）自动显示 ○ / ×。

## 4. 测试与验证

- **核心单测**（scene.rs）：仅 Map/无浮层/非遮幅可开；-5、-6 确认（信号一次性）；-7 取消（之后 -7 照旧开游戏内菜单）；
  框开期间方向键不动玩家（多拍冻结）；paint 出框（ui[0] 盒 + ui[11] 图标 1/2 于公式位）且软键 (1,2)、关后 (1,3)；
  hide_notify 关框并进游戏内菜单；已开原版浮层时不开、原版浮层不受影响。
- **壳宿主测**（shell/android/tests）：新游戏到 Map → BACK 开框（不进游戏内菜单）；框开点按不寻路；-7 取消后照常；
  再 BACK + -5 ⇒ `exit_requested`；面板 -7（键批）在 Map 仍开游戏内菜单；游戏内菜单/对话/背包 BACK 不开框。
- **回归**：`gates/cli.py check`（classic 帧夹具全绿）+ `tools/modern_frame_baseline.py --check`（Modern 5 状态 sha 不变）。
- **实机**（本方运行）：Map 右滑弹框截屏核样式；确定 ⇒ 进程结束（AM `has died: cch`）；取消 ⇒ 继续行走；框开时 HOME ⇒ 回来见游戏内菜单。

## 5. 登记

- `docs/spec/stage-b-deviations.md` 新增 **STAGE-B-005**（偏离点：原版无退出确认框 / Java 锚 a.java:2298 未用字面量 +
  kind 4 样式；用户裁定原话与 A/A/A/方案 2；金样影响 = 无，classic 不可达）。
- `shell/android/README.md` 输入表 KEYCODE_BACK 行；台账无需新 BUG（新功能）。

## 6. 不做

- 不加"保存并退出"、不做自动存档；不改游戏内菜单"退出"（kind 7）语义；不加触屏点按框内按钮（虚拟摇杆为主，
  decision_android-virtual-pad-primary-input）；win 壳不接线（核心接口可复用，另立任务）。
