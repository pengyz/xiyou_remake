# 状态机转移边集（35 边维护文档）——状态机件 D

> 2026-10-17 建档（state-machine-refactor-plan 件 D）。**维护义务**：PR 改
> `game_mode` 转移写点（transition_to 调用点）须**同步本表 + scene.rs
> `TRANSFER_EDGES`**，gates `state-machine-anchors` 机械对照两表行数/值一致。
> 数据源：state-machine-survey §2（40 写点→35 边逐条）。

## 1. 边表（TRANSFER_EDGES 单一代码源=scene.rs；本表=可读镜像+维护说明）

**编码**：`(from, to)` = GameMode 变体（case 号见 enums.rs GameMode doc 映射表）；
`to = None` = **动态目标**（menu_return_mode/in_game_menu_return_mode/target_mode
变量直写——Java 原样，边表记录"允许动态"）。

| # | from | 触因 | to | 动作 | 锚 |
|---|---|---|---|---|---|
| 1 | —（进程） | 【内部】启动 | Boot(0) | logo 时间线 | runLogoAnimation |
| 2 | Boot(0) | 【内部】logo 完 | SoundAsk(21) | 音效询问 | boot 序列 |
| 3 | SoundAsk(21) | 【键】应答 | MainMenu(1) | — | a.java boot |
| 4 | MainMenu(1) | 【键】新游戏 | Loading(2) | LoadProgress::new_game() | :4496 |
| 5 | MainMenu(1) | 【键】继续游戏 | Map(3) | 载档 | activate_menu_item 1（:2989） |
| 6 | MainMenu(1) | 【键】选项 | InGameMenu(4) | in_game_menu_return_mode=1 | m_000 case 4 |
| 7 | MainMenu(1) | 【键】退出 | Ending(20) | — | m_000 case 1?（EXIT 链） |
| 8 | Loading(2) | 【载入】进度≥100 | **动态**（target_mode） | call_m000 置位 | LoadProgress::tick |
| 9 | IntroCutscene(14) | 【内部】intro 完 | Loading(2) | new_game 加载链 | run case 14（:537-547） |
| 10 | IntroCutscene(14) | 【内部】粒子雨 4 分频 | IntroCutscene(14)（自环） | rng 粒子 | a.java:4137 |
| 11 | Map(3) | 【键】-7 | InGameMenu(4) | in_game_menu_return_mode=3 | run case 3（:4511） |
| 12 | Map(3) | 【键】-6 | ItemPanel(10) | 道具面板 | run case 3（:4515/:4359） |
| 13 | Map(3) | 【脚本/碰撞】剧情点 | Dialog(11) | loadLevelScript | :1692（try_run_scene） |
| 14 | Map(3) | 【碰撞】炼丹台 | Alchemy(9) | — | :4962（interact） |
| 15 | Map(3) | 【脚本】END_n | Ending(20) | m_000 case 20 | :2342-2346 |
| 16 | InGameMenu(4) | 【键】-7 | **动态**（in_game_menu_return_mode） | — | :2917 |
| 17 | InGameMenu(4) | 【键】项=存档 | SaveSlot(7) | loadSlotTable | activate_menu_item 3（:2993） |
| 18 | InGameMenu(4) | 【键】项=关于 | About(16) | — | activate_menu_item 4（:2999） |
| 19 | InGameMenu(4) | 【键】项=帮助 | Help(15) | — | activate_menu_item 5（:3005） |
| 20 | InGameMenu(4) | 【键】项=商店 | Shop(19) | — | activate_menu_item 11（:3009） |
| 21 | InGameMenu(4) | 【键】项=返回 | Map(3) | — | activate_menu_item 1（:3014） |
| 22 | InGameMenu(4) | 【键】项=帮助页 | EnemyInfo(5) | — | activate_menu_item 17（:2988 族） |
| 23 | EnemyInfo(5) | 【键】-7/-6 | Map(3) | — | run case 5（:3775） |
| 24 | SaveSlot(7)/EquipDetail(8) | 【键】-7 | **动态**（menu_return_mode） | — | run case 7（:3531） |
| 25 | Alchemy(9) | 【键】-7 | Map(3) | — | run case 9（:3908） |
| 26 | ItemPanel(10) | 【键】-7 | Map(3) | — | run case 10（:3108） |
| 27 | ItemPanel(10) | 【内部】道具用完 | Map(3) | 出栈回图 | activate_item（:3204-3273 五处） |
| 28 | ItemPanel(10) | 【键】使用 13 | EnemyInfo(5) | 敌情页 | activate_item 13（:3183） |
| 29 | ItemPanel(10) | 【键】使用 14 | RecordBrowser(12) | 记录浏览器 | activate_item 14（:3190） |
| 30 | Dialog(11) | 【收尾】行尾收尾门 | （非模式转移：dialog_phase=4） | 相机回玩家 | :1934（a.java:8597-8604） |
| 31 | Dialog(11) | 【收尾】相机收敛 | Map(3) | 回图 | tick_dialog（:2670/:2797） |
| 32 | Help(15) | 【键】-7 | **动态**（menu_return_mode） | — | :3679 |
| 33 | About(16) | 【键】-7/-6 | **动态**（menu_return_mode） | — | run case 16（:4261） |
| 34 | Shop(19) | 【键】-7 | InGameMenu(4) | 无 break 落 default | run case 19（:2962） |
| 35 | Ending(20) | 【键】f_bool_04 后任意键 | MainMenu(1) | 销毁 | run case 20（:4496） |

（script.rs harness 面独立：→Dialog（:193）/→Ending（:491）；测试合成点不计。
 RECORD-browser(12) 出口=overlay 关→ItemPanel(10)（overlay case5，:3325-3344）。）

## 2. 维护义务（PR 判据）

1. **改转移写点**（增/删/改 `self.transition_to(...)` 调用）⇒ 同步：
   ① scene.rs `TRANSFER_EDGES` 常量 ② 本表对应行 ③ 触因/锚列。
2. **新枚举值**（GameMode 新变体）⇒ 同步 enums.rs 映射表 + 本表新边 + 测试
   （近形防混淆测参照 EnemyInfo=5/Help=15 模式）。
3. **机械对照**：gates `state-machine-anchors` 校验 `TRANSFER_EDGES` 与本表
   边数一致（35±新增须两侧同改）+ 动态目标 None 编码一致。
4. **改键码约定** ⇒ key-code-contract.md §4 同步（(state,key) 二元组织约束不变）。

## 3. "不做三件"的散点维护代价由本件兜

state-machine-refactor-review A3 红线"不做三件"（转移表化/事件总线/HSM 框架化）
成立——但散点维护代价由**本件文档化义务**兜底：加新转移=改收口+边表（A' 结构解），
边表即 PR 对照面（无框架亦有机械校验）。
