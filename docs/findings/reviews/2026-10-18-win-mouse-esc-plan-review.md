# 对抗审查：Windows 操作适配三件（方案评审，lead 送审）

> 审查人：adversary。对象：①ESC→-7 ②鼠标菜单化三档（滚轮/左键/右键）③pointer_hit/
> handle_pointer 点行（game-core 几何查询+确认复用）。用户需求驱动。
> 考证亲验：reference/src/deobf 全目录 grep `pointerPressed|pointerReleased|
> pointerDragged|Pointer` = **零命中** ✓（原版无指针处理 = Stage B 加法，非移植项）。

## 判定：**①②通过（条件 2 条）；③有条件通过（3 硬条件 + Stage B 登记）**——可动手，条件随实现审查销账

## 0. Stage A/B 定性（本轮审查的轴心）

- **①② = 平台输入映射**（壳侧把物理键/鼠标事件合成为**既有键码**经 route_key 投递）：
  与 android 触控 overlay 同类——game-core 零改动、零新键码、折叠单键状态机语义不变。
  判定：**Stage A 可容**（输入设备翻译是壳的接缝 5 职责），登记 decision 即可。
- **③ = Stage B 行为加法**（game-core 新入口：光标跳行+直达确认——原版无此操作路径）。
  与音频焦点案（F-6 驳回）同属"原版没有的行为"，但两点不同：(a) **用户明确需求**
  （产品裁定先例：快速战斗调查→Stage B 加速，bug-ledger 调查结论行）；(b) 纯加法入口，
  不改既有路径。判定：**有条件放行**，条件 = ③-0 登记（下）+ 三硬条件，且**全部既有
  差分/golden/281+ 测试零漂移**是"行为只增不减"的机器判据（实现审查复跑）。

## 1. ③-0【硬条件·Stage B 登记】（AGENTS §1/§7，音频焦点案同款纪律）

- decision 文档（用户裁定：鼠标操作适配 = Stage B 加法，日期/需求原话）+ master-plan
  阶段面一行注记；架构文档接缝 5（输入映射）补"鼠标合成/点行"行。
- 无此登记 ⇒ 实现审查驳回（与 F-6 同尺）。

## 2. ③-1【硬条件·几何单源】（gotcha_dual-porting-surfaces-drift + BUG-008 前科）

pointer_hit 的行几何（行高/首行 y/框体原点/栅格步距）**必须复用 paint 布局的同一来源**
（SlotSelect 的 content_w/h/rows_per_page、menu_line_height、item panel 布局字段），
禁止在 pointer 侧自推坐标常量——否则绘制与命中两套几何 = 双拷贝漂移（BUG-008 字高
派生量教训直接适用）。TDD 增补：**行边界 y 与 paint 行 y 同源断言**（逐行锚测试：
pointer_hit(行中心 y) == 该行、pointer_hit(行边界±1) 分界正确）。

## 3. ③-2【硬条件·Row(u8) 语义定义】（现方案未定义，实现前必须定死）

1. **绝对行 or 可见行**：菜单可滚动（cursor_down/scroll_top 滚窗，menu_family.rs:785-797
   ——光标过界时 scroll_top/scroll_bottom 平移）。Row 若是可见窗口行号，点击必须
   `+ scroll_top` 映射绝对行；超窗行（滚动区外）命中归 None 还是 clamp——定死并测试。
2. **背包 4 列栅格**：Row(u8) 是格子一维索引还是行号？点(第 r 行第 c 格) → 光标索引
   = r*4+c？必须与 tick_item_panel 的光标语义（一维格索引）逐字对齐；半行尾格、
   空格位命中归谁。
3. **行间缝隙/部分可见行**：命中带定义（行高内有效 vs 边界归上/下/None）。

## 4. ③-3【硬条件·"复用 -5 路径"措辞错位，逐态指认 handler】（key-code-contract §4）

⚠️ 方案"走既有确认 handler（复用 -5 路径）"**以偏概全**——键码契约 §4 明文**同码不同
动作、各态确认键不同**：
- 标题菜单确认 = **-5/53**（title.rs:229 activate 臂）；
- 游戏内菜单族项激活 = **-6**（菜单→项激活）；
- 道具面板使用 = **-6**（道具面板→使用）。
要求：handle_pointer 逐态指认它调用的**具体 handler 函数**（title=activate 臂 / menu 族=
项激活臂 / item panel=使用臂），禁止"统一 -5 路径"措辞；TDD"点行副作用与键确认逐断言
等价"须按态分别对（点行 ≡ 对应态的对应键臂，不是恒 ≡ -5）。此为本方案最大语义风险点。

## 5. ①② 条件（轻）

- **C-a 滚轮态门控**：mode3 下 -1/-2 的 keyValue 臂是空的（scene.rs:4472-4484 只吃
  -7/-6），但 press(-1) 会置 key_held=-1 ⇒ 落拍即**走一步**（handle_field_input
  :4750 `facing = match self.key_held`）——滚轮误触 = 玩家窜步。要求：滚轮合成按
  游戏态门控（菜单/面板态才合成；壳侧有 game_mode() 可判），或书面接受"mode3 滚轮
  =走路"并登记（用户裁）。滚轮在对话态（mode11）= -1/-2 = 翻页/滚动（handleOverlayKey
  :1817 语义）= 自然行为 ✓ 保留。
- **C-b ESC 统一 -7 语义**：全态合成 -7 = 右软键全语义面（mode3 开菜单/面板返回/
  对话翻页关闭，key-code-contract §4）——与"解两问"预期一致且无副作用 ✓；唯一注意
  对话态 ESC=翻页（非关闭）属契约内 -7 语义，照实即可。

## 6. TDD 口径核（增补项）

方案三类测试成立；增补：③-1 同源断言、③-2 三定义测试（滚动映射/栅格索引/边界带）、
③-3 逐态等价断言（对**对应键臂**）。红证据三件套照 A 协议（含复现指令）。

## 7. 备忘

- pointer_hit/handle_pointer 落 BootMachine+GameScene 两实现 = 与 hide_notify 同款
  双入口结构（game-core 内单 API 双实现体可接受，非双移植面——语义在一处 spec 化）；
  若后续 wasm 壳复用，注意别演化出第三实现。
- 点行触发确认是**即发**语义（=按一下确认），无双击/长按语义——保持，勿扩展。
