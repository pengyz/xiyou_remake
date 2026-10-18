# 对抗审查：WIN-OP③ 指针缝 6bf57c2（行点击+hover）——**驳回 1 项整改**

> 审查人：adversary。对象：6bf57c2（game-core 指针缝 + shell/win hover/点击接线）。
> 对照：WIN-OP 微调裁定 (a)-(d) 四条件 + ③-1/③-2 硬条件。
> 方法：paint↔pointer 几何逐行追踪（A 级）、测试源码核、GREEN 独立复跑。

## 判定：**驳回（1 项硬条件违规：③-1 几何单源在背包栅格侧不成立，1px 错位 + 自证测试）**
——(a)(b)(c)、菜单侧 (d)、hover 边界、softkey_code 单源、零漂移全部通过；整改面单一、改后免复审全量（只核几何两处 + 补测）。

## 1. 通过面（亲验）

| 项 | 核验 | 结论 |
|---|---|---|
| (a) confirm_key 逐态 | 菜单族/Shop=-6、ItemPanel=-6（pointer_hit 各臂）；标题 N/A=已知面（icon 轮播无行带，基线滚轮/点击可操作） | ✅ |
| (b) select_row 只动光标+滚窗 | menu_cursor/item_cursor + scroll 窗同步、他态零变化（`_=>{}`）；测试 select_row_only_cursor_and_scroll | ✅ |
| (c) 合成=press+release 同拍 | pointer_click 返回 confirm_key → shell 合成点按（与 click_code 同款）；走 route_key 唯一键路 | ✅ |
| hover 零副作用 | pointer_hover：Game+Row 才 select_row；对话/玩法态恒 None（pointer_hit `_=>None`） | ✅ |
| softkey_code 单源（硬化建议已做） | click_key/click_code 共调 softkey_code（unwrap_or(-5)） | ✅ |
| 菜单侧几何同式（③-1） | menu_row_band = CENTER_Y-((menu_list_height+32)>>1)+16+pad+vis*lh 减 pad ⇒ 行带 = 盒顶+16+vis*lh，与 paint_menu_list 逐行同值（scene.rs:3001-3010 累积式对照）✓ 测试 y0 常量锚正确 | ✅ |
| 零漂移 | 独立复跑：game-core lib **92/92**（89+3）、game-desktop **29/29** | ✅ |

## 2. 驳回点（③-1 违规，claim/content 不符）

### F-1【硬条件③-1】item_cell_origin 与 paint_item_panel **差 1px**，且测试自证

- **paint 实际格原点**（scene.rs 逐行累积，A 级）：y = (SCREEN_H-item_box_h)>>1
  （:3409）→ +16（:3412）→ +font.height+4（:3420）→ +1（:3424）→ +10（:3427）
  = **盒顶 + 31 + font.height**；格画在 (col_x0=盒左+20, y)（:3411/:3428-3434）。
- **pointer 实现**（scene.rs:7719-7725）：oy = 盒顶 + **32** + font.height ⇒
  **下移 1px**。后果：每格视觉顶像素落进**上一格**命中带（行 0 顶像素=None）——
  正是 ③-1 要防的"绘制与命中两套几何"。
- **测试自证**（scene.rs:7865 `item_grid_pointer_hit_and_select`，:7873
  `let (ox, oy) = sc.item_cell_origin();`）：期望值取自**被测函数自身**——pointer 侧
  几何漂移 1px 测试照绿。菜单侧测试用常量锚（y0=160-((112+32)>>1)+16）是正确姿势，
  背包侧没有等价物。
- **claim 不符**："item_cell_origin 与 paint_item_panel 逐项同式（y=box_y+32+font.height）"
  ——注释与 commit 声称同式，实际 31≠32。

### 整改清单（可执行，改后我只复核三件）

1. **y 同值**：item_cell_origin 的 y 改为盒顶+**31**+font.height（=paint 累积式
   16+font.height+4+1+10），或**更好**：把格原点提为共享 fn（paint_item_panel 与
   pointer 共调，与 softkey_code 单源同款收口）——推荐后者，根除公式双份。
2. **测试改同源断言**：期望值**禁止取自 item_cell_origin() 自身**——用共享 fn 或
   字面常量锚（`(SCREEN_H-item_box_h)/2 + 31 + font.height`）并在注释锚 paint 行号
   （:3409-:3427 累积式）；补**顶像素归属**断言（格视觉顶像素命中本格、顶-1 像素命中
   上一格/None——±1px 锚照菜单侧规格补齐栅格版）。
3. **x 侧无恙**（盒左+20 双侧同值 ✓）不动；菜单侧不动。

## 3. 备忘

- hover 是 Stage B 加法面——decision_pointer-menu-stage-b 覆盖"指针操作"整体即可，
  若 decision 文本只写"点行"请补 hover 一词（文本级，随整改顺手）。
- 标题=已知面（icon 轮播无行带）登记 ✓ 与基线档能力（滚轮/点击/右键）组合可玩。

## 4. F-1 销账复核（d0a2a6e，2026-10-18）——**通过，整案收口**

| 整改项 | 核验 | 结论 |
|---|---|---|
| ① y 同值 + 真单源 | `item_cell_origin`（scene.rs:7721）y = 盒顶+**31**+font.height；paint_item_panel（:3428）与 pointer_hit（:7748）**共调同一 fn**（paint 侧内联累积式已删，逐项等价我复核：旧累积 16+fh+4+1+10=31+fh、col_x0=盒左+20 同值） | ✅ |
| ② 测试去自证 + 顶像素锚 | 期望=字面常量（(SCREEN_W-item_box_w)/2+20、(SCREEN_H-item_box_h)/2+31+paint_font().height）+ 注释锚 paint 累积式行号；`item_cell_origin()==(ox,oy)` 对常量断言（:7880）；顶像素归属双断言（格顶像素归本格 / 顶-1 = None） | ✅ |
| ③ x 侧/菜单侧未动 | diff 仅涉 item 侧/测试/decision——menu_row_band、菜单臂零变更 | ✅ |
| 顺手项 | decision 补 "hover 选中高亮"入 Stage B 面 | ✅ |
| 零漂移 | 我复跑 core lib 92/92 + desktop 29/29；oracle 9 箱金样差分全绿（其复验，paint 共调后像素级无痕——与我算术等价核互证） | ✅ |

**判定更新：WIN-OP③ 整案通过。**（驳回点 F-1 已销，无遗留。）
