# 对抗审查：WIN-OP③ 补面 2856879（设置面板 OptionList hover/点行）——有条件通过

> 审查人：adversary。对象：2856879（menu_family OptionList 行带/选行 + scene 指针臂）。
> 对照：WIN-OP 微调裁定 (a)-(d) 四条件同尺 + F-1 整改口径（值对 + 字面锚测试 = 最低标准）。

## 判定：**有条件通过**（2 条件随推送销账）——值与测试全对，但 1 处声称不符 + 1 个未列已知面

## 1. 条件核验

| 项 | 核验 | 结论 |
|---|---|---|
| (a) confirm_key 逐态 | tick_menu16（scene.rs:4224-4247）：`-5\|-4\|-3` 翻转选项、`-7\|-6` 返回 ⇒ **confirm=-5 正确**（a.java:4143-4188 锚与代码注释一致）；注意此态确认键是 -5 非菜单族 -6——逐态数据设计正确兑现（③-3 风险点的正面样本） | ✅ |
| (b) select_row 只动高亮 | set_highlight 仅动 highlight + 测试断言他态零变化（hp 等） | ✅ |
| (c) 合成键唯一键路 | pointer_click 返回 confirm_key → shell 合成 press+release（既有接线不动） | ✅ |
| (d) 几何值 | row_band（menu_family.rs:1318-1322）= (SCREEN_H-box_h)/2+16+(fh+8)*i、行高 fh+8——与 OptionList::paint 内联累积（:1327 盒顶、:1330 +16、:1352 +fh+8、高亮 :1331-1333）**逐项同值**（我逐行核）；测试=字面常量锚（box_h=32+2*row_h 累加式 a.java:9911-9926）+ 行界 ±1px 归属（顶像素归本行/顶-1=None/行界归下行），无自证 | ✅ 值+测试达 F-1 标准 |
| 零漂移 | core lib **93/93**（92+1 新测） | ✅ |

## 2. 条件（随推送销账）

### C-1【声称与内容不符】"paint/pointer **共调**"为假——同式未共调

row_band 文档注释与 commit 声称"paint 与 pointer_hit 共调"，但 OptionList::paint
（menu_family.rs:1325-1354）**未调用 row_band**，行 y 仍是内联累积——公式双份仍在
（F-1 失效模式的结构面残留，值目前相同 + 字面锚测试兜底，风险=未来 paint 侧改动静默漂移）。
处置二选一：**(i) paint 循环/高亮真调 row_band**（一两行，根除双份——推荐）；或
**(ii) 注释与 commit 表述改"同式 + 字面锚"**。禁止保留"共调"措辞（gotcha_commit-
claims-must-match-content）。

### C-2【未列已知面·覆盖不对称】标题侧设置面板（BootMachine OptionList）无指针

设置面板在**两个游戏阶段各有一份消费面**（Java 单对象两相，hide_notify 同款结构）：
GameScene tick_menu16（本提交已修）**与 BootMachine paint.rs:487-523**（title 菜单
kind 2 → GameMode::About，title.rs:232——标题侧"设置"）。shell 指针只派发
`Screen::Game`（pointer_hover/pointer_click）⇒ **标题侧设置面板 hover/点行仍无效**。
用户反馈"设置界面 hover 无选中效果"——若指标题侧设置，本提交只修了一半；
已知面清单（标题=图标轮播、存档槽=下一候选）**漏列此面**。处置二选一：
(i) 本次补修（BootMachine::pointer_hit/select_row 同款 + shell Screen::Boot 派发——
row_band 已在 menu_family 共享，便宜）；(ii) 登记"标题侧设置=候选面"并**向用户确认
所指设置界面**，另立小任务。不许静默留白。

## 3. 备忘

- 行带命中与 x 无关（盒外横向点击也命中行）——与菜单臂同口径（一致性 OK），
  若日后收紧横向命中带，菜单/设置两臂同改。
- GameMode::About.raw()==16（enums.rs:348 锚）命名沿保真历史（设置面板挂 About 态），
  新读者注意，勿"顺手改名"。
