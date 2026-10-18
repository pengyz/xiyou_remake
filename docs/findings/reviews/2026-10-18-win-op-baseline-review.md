# 对抗审查：WIN-OP 基线件 548d25b（ESC→-7 + 鼠标态门控合成）+ ③微调裁定

> 审查人：adversary。对象：548d25b（ESC/滚轮/点击基线 + ③-0 登记三处）。
> 对照：2026-10-18-win-mouse-esc-plan-review.md 的 C-a/C-b/③-0 条件。

## 判定：**通过**（条件全销）；③ 微调**批准**（4 条配套条件）；1 条硬化建议

## 1. 条件销账

| 条件 | 核验 | 结论 |
|---|---|---|
| C-a 滚轮态门控 | `wheel_key(dy, mode)`：Map ⇒ None（main.rs:205 附近，注释写明"防 key_held 窜步"）、零位移 None、菜单=±1、对话=翻页保留；测试钉 Map×2/InGameMenu/Dialog/零位移 | ✅ 销 |
| C-b ESC 全语义面 | `Code(K::Escape) => -7` 入 key_code 主表（右软键全态语义，契约 §4）；测试断言 ESC→-7 | ✅ 销 |
| ③-0 Stage B 登记三处 | decision_pointer-menu-stage-b.md + master-plan M5 注记 + 架构文档接缝 5 行，随 commit 入库 | ✅ 销 |
| TDD A 协议 | RED 三件（E0425 `cannot find function wheel_key` ×5 逐字 + 一致性声明 + 复现指令 `cargo test -p game-desktop --release win_op_esc`）；wheel_key/click_key 为新符号（父树缺席，红族必然） | ✅ 销 |
| 零漂移 | 我独立复跑：game-desktop **29/29** + game-core **10 箱全 ok**（153 测，EXIT=0） | ✅ 销 |
| 三对账 | 4 文件（main.rs + 登记三件）内容/声称/写范围一致，pathspec 干净 | ✅ |

实现面核：右键=-7 全态、软键区优先（沿 click_code 旧例）、非软键左键=-5、Map 态点击
禁合成（防误触玩法键）——均与方案边界自检一致；点按=press+release 同拍投递
（沿 click_code 先例，下一拍消费）✓。
【测量自纠备案】首轮大小写不敏感 -match 误报 FAIL=10 → 大小写敏感复验更正——
与我 GBK/编码伪差同族（取证脚本测量面陷阱），自报留痕 ✓。

## 2. ③ 微调裁定：**批准**（比原设计更零语义）

`pointer_hit → Row{index, confirm_key}` + `select_row`（game-core 只动光标）+
**确认走 shell route_key 合成键**（press+release 进唯一键路）——批准理由：

1. 确认不再由 game-core 直调 handler（原设计的第二入口消失）⇒ 执行面=物理键完全同路
   （press_key → tick 消费臂），比我 ③-3"逐态指认 handler"要求**更强**；
2. 逐态语义由 **confirm_key 数据**承载（键码契约 §4 同码不同动作问题转为显式数据）；
3. 时序正确：合成键下一拍消费 = "点按"语义（与 click_code 同款），不即时推进逻辑。

**配套条件（实现审查销）**：
- (a) confirm_key **逐态取值测试钉死**：标题=与其消费臂一致（title.rs:229 activate 的
  键面是 -5/53 双码，取值须与实际臂一致）、菜单族=**-6**、道具面板=**-6**；
- (b) `select_row` 只动光标 + 滚窗同步（scroll_top），断言无其他状态变化；
  Row 语义三定义（绝对/可见行、栅格索引、命中带）照 ③-2 定死；
- (c) 合成确认=press+release 同拍（点按语义）；折叠语义与物理键一致（走 route_key
  即自动满足）；
- (d) 几何单源（③-1）照旧：行几何复用 paint 布局同源 + 行边界逐行锚。

## 3. 硬化建议（不卡闸）

click_key 的软键区谓词（main.rs:223 `y > h*7/10 && (x < w/3 || x > w*2/3)`）与
click_code 内部判区（:1009-1012）**双份**——边界测试钉的是 click_code 侧（:1798-1799），
click_key 侧漂移不会被现测试抓。建议判区单源（共享谓词或 click_key 先取 click_code
结果再映射）；③ 实现期顺手收掉即可。
