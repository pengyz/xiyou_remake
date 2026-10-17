# 状态机转移表普查：gameMode × dialogPhase × walkPhase 三层 HSM（state-machine-survey）

> 【2026-10-17】纯普查不动代码。状态写点/厚度/臂数全部实测（python 扫描 + 逐处判读），
> 口径沿 dispatch-shape-survey v2："一处"=逻辑编辑点（文件级）。行号基准 = 当日 05:00 树，
> 引用以函数锚+a.java 注释锚为主。对象 = gameMode 顶层 + dialogPhase/walkPhase/route_phase
> 嵌套 + key_value 消费面。

## 0. 结论速览

1. **HSM 结构成立但枚举有缺口**：三层嵌套语义清晰（gameMode 顶层 → mode11×dialogPhase /
   mode3|11×walkPhase → routePhase 子层 + overlay 覆盖层），但 `GameMode` 枚举只收 9 值而
   实际使用 **15+ 值**（4/5/7/8/9/10/12/15/16/19 落 `Unknown(i32)`）——对照锚散在注释。
2. **转移密度**：gameMode 写点 **40 处**（scene 38 + script 2）→ 转移边 ~35 条；hub 态 = 3
   （地图，出边 12+）、4（菜单族中转）。**陷阱态 1 个**（20 Ending：只进不出 by design，
   f_bool_04 后任意键→1）；**悬空态 1 个**（99 NetWait：DC-3 已裁死，无转移入）。
3. **热冷分明**：tick 消费系（mode 3/11 每拍热）vs enter/tick_menu 事件冷态；热路径最大
   厚块 = tick_dialog dialog_phase 臂（48 行）与 route_phase 臂（57 行）。
4. **key_value 消费面 26 处 match**（scene 20 + BootMachine 6）：全部 **edge 消费**（press
   置值、tick 分支消费、尾清 `key_value = 0`〔a.java:4270/4276〕）；键码语义跨态复用
   （-7 返回/-6 确认为通用约定），其余各态私有。
5. **形状结论**：转移壳（match/if 分派）**不拆不表化**（对照体）；厚处理块（48/57 行）
   内的特例代码是解耦候选（与分派普查同源）；`GameMode` 枚举补全是低风险高收益补丁。

---

## 1. 状态枚举全景

### 1.1 gameMode（顶层）——实际 15+ 值，枚举收 9 值【缺口标注】

| 值 | 语义 | a.java 锚（run/m_000/paint case） | 入口触因 | 退出语义 | 枚举 |
|---|---|---|---|---|---|
| 0 | 启动画面（boot logo） | paint/run case 0 | 进程启动 | logo 时间线完 → 21（音效询问）/1 | Boot ✓ |
| 1 | 主菜单 | m_000 case 1 | 0→21→1 / 20 任意键 | 菜单激活 → 2/4/3/20 | MainMenu ✓ |
| 2 | 资源载入进度 | run case 2（a.java:3449-3541） | 1 激活/14 完成 | 进度≥100 → target_mode（LoadProgress） | Loading ✓ |
| 3 | **地图主玩法**（hub） | run case 3（a.java:3551-3679）/paint case 3（:2385-2422） | 各态返回/载入完成 | -7→4、-6→10、脚本→11、碰撞→9 等 | Map ✓ |
| 4 | 游戏内菜单 | m_000 case 4（a.java:4326-4337）/run case 4（:3686-3735） | 3 的 -7 | -7→in_game_menu_return_mode；项激活 → 7/15/16/19/3 | **Unknown ✗** |
| 5 | 敌人信息/帮助页 | run case 5（a.java:3737-3755）/paint case 5（:2497-2560） | 4 激活 / 13 道具使用 | -7/-6 → 3；-2/-1 翻页 | **Unknown ✗** |
| 7 | 存档槽（7/8 共享） | m_000 case 7（a.java:4342-4364）/run case 7（:3829-3878） | 4 激活 | -7 → menu_return_mode；-6/-5 确认写档 | **Unknown ✗** |
| 8 | 装备详情（7/8 共享） | 同上（case7/8 共享） | 7 内切换 | 同 7 | **Unknown ✗** |
| 9 | 炼丹 | run case 9（a.java:3885-3940）/paint case 9（:2690-2740） | 3 碰撞炼丹台 | -7 → 3；-6/-5 金币够则加属性 | **Unknown ✗** |
| 10 | 道具面板 | m_000 case 10（a.java:4370-4380）/run case 10（:3931-4009）/paint case 10（:2724-2792） | 3 的 -6 | -7 → 3；-6/-5 使用道具 | **Unknown ✗** |
| 11 | 对白/剧情（脚本模式） | paint case 11（a.java:2798-2896）；loadLevelScript 进入 | 脚本指令/碰撞剧情点 | 行尾收尾门→dialog_phase 4→相机收敛→3 | Dialog ✓ |
| 12 | 记录浏览器 | overlay case5（a.java:3325-3344）/paint_mode12 | 14 生死簿使用 | overlay 关 → 10 | **Unknown ✗** |
| 14 | 读条过场（新游戏） | run case 14（a.java:4137）粒子雨 4 分频 | 1 新游戏 | intro tick 完 → 2（new_game 加载链） | IntroCutscene ✓ |
| 15 | 帮助全文浮层 | m_000 case 15（a.java:4402-4413）/paint case 15（:2958-2962） | 4 激活 | -7 → menu_return_mode | **Unknown ✗** |
| 16 | 关于（图标条） | run case 16（a.java:4143-4188）/paint case 16（:2963-2990） | 4 激活 | -7/-6 → menu_return_mode；-5/-4/-3 翻转 | **Unknown ✗** |
| 19 | 商店（5 条目） | m_000 case 19（a.java:4439-4448）/run case 19（:4196-4226） | 4 激活 | -7 → 4（无 break 落 default 同效） | **Unknown ✗** |
| 20 | END 结局过场 | m_000 case 20（a.java:4443-4450）/run case 20（:4223-4228）/paint case 20（:3039-3067） | END_n 脚本（a.java:8445-8463） | f_bool_04 后任意键 → 1（70 帧后销毁） | Ending ✓ |
| 21 | 音效询问 | —（boot 序列） | 0 首启 | 应答 → 1 | SoundAsk ✓ |
| 99 | 联网等待 | **DC-3 裁死**（gameMode 写点无 99） | **无**（悬空） | —（联网簇死码） | NetWait ✓ |

**【普查要点】**：10 个活跃态落 `Unknown(i32)`——枚举补全 = 零行为差低风险补丁（§6）；
99 = 悬空态（死码已在 DC-3 裁决书钉死，保留枚举项作对照）。

### 1.2 dialogPhase（mode 11 内层，enums.rs:9-28 全值有据）

| 值 | 语义 | 写点触因 | 退出 |
|---|---|---|---|
| 0 ScriptStep | 脚本步进（execute_script_instruction 消费） | 行首装载/收尾重置 | 指令 → 1/2/3/4/5/6 |
| 1 Typewriter | 打字机（文本未完） | TAK 对话装载（:2145） | 打完 → 0（按键/自动） |
| 2 EntityRoute | 实体路径缓动（MOV 五参→m_106 逐拍） | MOV 五参（:2266） | 缓动完 → 3/0 |
| 3 Walk | 行走执行（消费 walkPathBuffer） | MOV 三参/探针臂 | 走完 → 0 |
| 4 AwaitCamera | 等相机到位（含行尾收尾门） | 收尾门（:1934）/相机悬置 | cameraAtTarget → 0/回图 |
| 5 LayCutscene | LAY 过场（换层清屏） | LAY 指令（:2155） | 过场完 → 0 |
| 6 TypewriterCamera | 打字机+相机门控（SEE 变体） | SEE（:2229） | 同 1 + 相机门 |
| Unknown(i32) | 保真保留任意值 | — | — |

### 1.3 walkPhase（mode 3/11 内层，enums.rs:61-75）

| 值 | 语义 | 写点触因 | 退出 |
|---|---|---|---|
| 0 Idle | 待机/探索（-5 探索与方向处理） | 初始化/归格/战斗结束 | 方向键 → 1；-5 → 2 |
| 1 Stepping | 步进动画（8px/拍，32px 归格） | 方向键（:2632/:2730/:4811） | 归格 → 0 |
| 2 ExploreConfirm | 探索确认弹窗 | -5 残影位填充（:4754） | 确认 → 0/5 |
| 3 ScriptWalk | 脚本寻路行走 | dialogPhase=3 消费 | 走完 → 0 |
| 4 Phase4 | m_063 使用点（语义待考证，机械名保留） | :3311/:5448/:5481 | → 0 |
| 5 Battle | 战斗（tick_battle） | 遇敌（:2719/:5026） | 战结 → 0（死亡动画 6 拍） |

### 1.4 辅助态

- **routePhase**（dialogPhase=2 的子层，0/1/5 有效 + 7=测试脏值）：0 复位（a.java:8291）
  /1 推进/5 完成迁槽——MOV 五参实体路径缓动的子状态机。
- **overlay 覆盖层**（`overlay_active`+`overlay_kind`）：横切态——kind4 商人/告示牌、kind5 日志
  浏览等，**优先于 mode 消费**（tick 入口拦截），独立小状态机（handle_overlay_key）。
- **facing**（0-3 朝向）：非状态机，是数据；route_dir/walk_path_buffer = 路径数据流。

---

## 2. 转移表（核心）——gameMode 写点 40 处 → 转移边逐条

**触因分类**：【键】key_value 分支 / 【脚本】DSL 指令 / 【碰撞】interact/step / 【载入】
LoadProgress 完成 / 【内部】逻辑内联 / 【收尾】相机/动画收敛。

| # | from | 触因 | to | 动作 | 锚 |
|---|---|---|---|---|---|
| 1 | —（进程） | 【内部】启动 | 0 | logo 时间线 | runLogoAnimation |
| 2 | 0 | 【内部】logo 完 | 21 | 音效询问 | boot 序列 |
| 3 | 21 | 【键】应答 | 1 | — | a.java boot |
| 4 | 1 | 【键】新游戏 | 2 | LoadProgress::new_game() | :4496（m_000 case 1） |
| 5 | 1 | 【键】继续游戏 | 3 | 载档 | activate_menu_item 1（:2989） |
| 6 | 1 | 【键】选项 | 4 | in_game_menu_return_mode=1 | m_000 case 4 |
| 7 | 1 | 【键】退出 | 20 | — | m_000 case 1?（EXIT 链） |
| 8 | 2 | 【载入】进度≥100 | target_mode | call_m000 置位 | LoadProgress::tick |
| 9 | 14 | 【内部】intro 完 | 2 | new_game 加载链 | run case 14（:537-547） |
| 10 | 14 | 【内部】粒子雨 4 分频 | （自环） | rng 粒子 | a.java:4137 |
| 11 | 3 | 【键】-7 | 4 | in_game_menu_return_mode=3 | run case 3（:4511） |
| 12 | 3 | 【键】-6 | 10 | 道具面板 | run case 3（:4515/:4359） |
| 13 | 3 | 【脚本/碰撞】剧情点 | 11 | loadLevelScript | :1692（try_run_scene） |
| 14 | 3 | 【碰撞】炼丹台 | 9 | — | :4962（interact） |
| 15 | 3 | 【脚本】END_n | 20 | m_000 case 20 | :2342-2346 |
| 16 | 4 | 【键】-7 | in_game_menu_return_mode | — | :2917 |
| 17 | 4 | 【键】项=存档 | 7 | loadSlotTable | activate_menu_item 3（:2993） |
| 18 | 4 | 【键】项=关于 | 16 | — | activate_menu_item 4（:2999） |
| 19 | 4 | 【键】项=帮助 | 15 | — | activate_menu_item 5（:3005） |
| 20 | 4 | 【键】项=商店 | 19 | — | activate_menu_item 11（:3009） |
| 21 | 4 | 【键】项=返回 | 3 | — | activate_menu_item 1（:3014） |
| 22 | 4 | 【键】项=帮助页 | 5 | — | activate_menu_item 17（:2988 族） |
| 23 | 5 | 【键】-7/-6 | 3 | — | run case 5（:3775） |
| 24 | 7/8 | 【键】-7 | menu_return_mode | — | run case 7（:3531） |
| 25 | 9 | 【键】-7 | 3 | — | run case 9（:3908） |
| 26 | 10 | 【键】-7 | 3 | — | run case 10（:3108） |
| 27 | 10 | 【内部】道具用完 | 3 | 出栈回图 | activate_item（:3204-3273 五处） |
| 28 | 10 | 【键】使用 13 | 5 | 敌情页 | activate_item 13（:3183） |
| 29 | 10 | 【键】使用 14 | 12 | 记录浏览器 | activate_item 14（:3190） |
| 30 | 11 | 【收尾】行尾收尾门 | dialog_phase 4 | 相机回玩家 | :1934（a.java:8597-8604） |
| 31 | 11 | 【收尾】相机收敛 | 3 | 回图 | tick_dialog（:2670/:2797） |
| 32 | 15 | 【键】-7 | menu_return_mode | — | :3679 |
| 33 | 16 | 【键】-7/-6 | menu_return_mode | — | run case 16（:4261） |
| 34 | 19 | 【键】-7 | 4 | 无 break 落 default | run case 19（:2962） |
| 35 | 20 | 【键】f_bool_04 后任意键 | 1 | 销毁 | run case 20（:4496） |

（script.rs harness 面独立：→11（:193）/→Ending（:491）；测试合成点不计。）

**转移密度矩阵**（出边→入边粗计）：

| 态 | 出度 | 入度 | 性质 |
|---|---|---|---|
| 3 Map | 12+ | 15+ | **hub**（一切回图） |
| 4 Menu | 7 | 2 | 中转 |
| 10 ItemPanel | 4 | 3 | 中转 |
| 11 Dialog | 1（→3） | 2 | 脚本进/收尾出 |
| 1 MainMenu | 4 | 2 | 入口 |
| 2 Loading | 1 | 3 | 管道 |
| 5/7/8/9/15/16/19 | 1-2 | 1-2 | 叶态（菜单族） |
| 12 Record | 1（→10） | 1 | 叶态 |
| 14 Intro | 2 | 1 | 管道 |
| 20 Ending | 1 | 2 | **陷阱态**（只进不出 by design；f_bool_04 后出） |
| 21 SoundAsk | 1 | 1 | 管道 |
| 99 NetWait | 0 | 0 | **悬空态**（DC-3 死码） |

**悬空态**：99（无入无出，死码钉死）；**陷阱态**：20（by design——结局吞键直至旗落）。
无其他悬空/陷阱：menu 族叶态均有 -7 返回出边。

---

## 3. 每态处理块厚度（实测）与热冷

| 态 | 消费函数 | 厚度（行/臂数） | 热冷 |
|---|---|---|---|
| 3（子态机） | tick() key_value 链（:4346-4520 六段 match 3-6 臂） | 每段 3-6 臂 × 1-6 行 | **热**（每拍） |
| 3 行走 | tick walk_phase match（5 臂，max 46） | 46 行 max | **热** |
| 11 脚本 | execute_script_instruction（命令模式 16 op 逐 op 函数） | parse_op 7 臂 + execute_op 独立函数 | **热**（脚本期每拍） |
| 11 对话 | tick_dialog dialog_phase match（7 臂，max 48） | 48 行 max（打字机+渲染混入） | **热** |
| 11 路径 | tick_entity_route route_phase match（4 臂，max 57） | 57 行 max（移动+战斗+钩子混入） | **热**（EntityRoute 期） |
| 4/19/7/15/5/9/16 | tick_menu4/19/7/15/5/9/16（各 key_value 3-7 臂） | 4-23 行/臂 | **冷**（事件性） |
| 10 | tick_item_panel key_value（7 臂，max 21） | 21 行 max | 冷 |
| overlay | handle_overlay_key + kind4/kind5 分支 | 4 臂 ×1-4 行 | 冷（但横切） |
| paint 系 | paint() mode 分派（9 臂，max 11）+ paint_entities cat8（93 行） | 渲染每帧 | **热**（渲染） |
| enter 系 | enter_menu4/7/9/15/16/19/enter_view… | 初始化块 4-22 行 | 冷（事件） |

**热路径痛点**（与分派普查交叉确认）：tick_dialog 48 行、tick_entity_route 57 行、
walk_phase 46 行——三个厚块都是"状态壳+特例逻辑混装"。

---

## 4. key_value 消费面归属（26 处 match + 41 写点）

**消费模型**（edge 语义，a.java:4270/4276 尾清锚）：`press_key` 置 `key_value = code`
（覆盖式取末键）→ 下一拍 tick 各态分支消费 → **尾清 `key_value = 0`**（每消费臂一处，
scene 17 处清零 + BootMachine 9 处）。**held 语义**：`key_held_code` 独立字段（行走连续步进
用），与 key_value 分离。

| 归属 | 处数 | 键码面 | 复用性 |
|---|---|---|---|
| mode 3 子分派（tick 六段） | 6 | -7/-6（暂停/道具）、49-55（行走方向 split up/down）、-5/53（探索）、-1/-3（菜单） | -7/-6/-5 跨态复用 |
| 菜单族 tick_menu4/19/7/15/5/9/16 | 7 | -7 返回/-6 确认/-2/-4 下移/-1/-3 上移 | **键码语义跨态复用**（同码不同动作） |
| tick_item_panel | 1 | 同菜单族 + 使用 | 私有 |
| handle_overlay_key | 1 | ±1 翻页/±lines 滚动 | overlay 私有 |
| tick_dialog（:2600） | 1 | 对话推进键 | 私有 |
| paint 无关（BootMachine tick 6 处） | 6 | boot/title 键序 | BootMachine 私有 |

**要点**：键码 -7/-6/-5/-1/-3 是**跨态通用约定**（返回/确认/上移/下移），但各态动作不同——
转移表化时须按 (state, key) 二元组而非 key 单维组织。

---

## 5. HSM 层次确认

```
gameMode（顶层 15+ 值）
├─ mode 3（地图）── walkPhase（0-5 行走子状态机）── [overlay 覆盖层横切]
├─ mode 11（脚本）─┬─ dialogPhase（0-6 脚本/对话子状态机）
│                  └─ dialogPhase=2 ── routePhase（0/1/5 实体路径子层）
├─ menu族 4/5/7/8/9/10/12/15/16/19 ──（无深层；key_value 直接消费）
└─ 管道 0/1/2/14/20/21/99
```

**合法组合**（代码实证）：mode3×walkPhase 0-5；mode11×dialogPhase 0-6；
dialogPhase=2×routePhase 0/1/5；overlay_active×任意 mode（横切优先）。
**未观测/非法**：dialogPhase≠0 @ mode3（测试合成除外，:7190 夹具）；walkPhase=3 @ mode11
未见独立写点（脚本行走经 dialogPhase=3）；routePhase @ mode3 不出现。**Unknown(i32) 保真兜底**
（Java 字节可容任意值）。

**对照锚总账**：run() case 树 = a.java:3551-3679（mode3）/3686-3735(4)/3737-3755(5)/3829-3878(7,8)/
3885-3940(9)/3931-4009(10)/4137-4136(14)/4143-4188(16)/4196-4226(19)/4223-4228(20)；
m_000() case 树 = a.java:4312-4318(3)/4326-4337(4)/4342-4364(7,8)/4370-4380(10)/4402-4413(15)/
4439-4448(19)/4443-4450(20)；paint case 树 = a.java:2385-2422(3)/2497-2560(5)/2679-2682(7)/
2690-2740(9)/2724-2792(10)/2798-2896(11)/2958-2962(15)/2963-2990(16)/3039-3067(20)。

---

## 6. 形状结论输入（三形 + 解耦候选）

| 形 | 代表 | 判定 |
|---|---|---|
| **厚转移块**（≥45 行处理块） | tick_dialog 48 行、tick_entity_route 57 行、walk_phase 46 行、paint_entities cat8 93 行 | **解耦候选**：壳（match/转移）保真不动；块内特例抽表/抽函数（与 dispatch-shape-survey §5 同源同法） |
| **薄壳转移** | tick_menu 系 3-7 臂 ×1-6 行、tick 六段 key 链、overlay 分支 | **不拆不表化**（对照体；转移动作=逐值对照 run/m_000 case） |
| **表驱动候选** | ① GameMode 枚举补全（10 个 Unknown 值）② 转移表本身（(mode,key)→(to,action) 可表化但**对照性受损**——不建议）③ 键码映射（-7/-6 通用约定文档化） | ① **低风险高收益补丁**；② 不做；③ 文档化即可 |

**HSM 模块化适用面**：三层嵌套已是事实结构，但 gameMode 值语义散在注释/枚举缺口——
**建议小步**：枚举补全（10 值）+ 每值 doc 注释锚（§1.1 表照抄）+ transition 不动；
**不建议**转移表化/事件总线化（保真对照体拆解=与"不拆 execute_script_instruction"同列红线）。

**工作量输入**（给下一步计划）：枚举补全 ~0.5 天（零行为差）；厚块特例抽出 ~2-3 天（依赖
表设计 v2 落地）；转移表化=不做。与 dispatch-shape-survey §6 步 3 合并执行更省回归墙钟。
