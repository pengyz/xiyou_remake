# 对抗复核 A：设置层级（UX 批件1）— 攻击性复核

> 角色：remake-verifier 纪律 · 对抗复核员 A
> 对象 commit：`cbcb647`（+批二 `9c698e1`，设置面仅涉审计表"设置行豁免直通"，无独立设置改面）
> 用户裁定（复核基准）：操控方式"太隐晦"→ 值循环**显示模式名**；触屏模式下摇杆专属项
> （按键音/按键震动）**完全隐藏**（非灰显）。
> 纪律执行：不改产品码——全部破坏性验证后 `git checkout` 还原，复跑绿实录收尾；
> 结论带行号/命令证据；未跑不称绿。

**基线实录（破坏前）**：
```
cargo test -p game-android --test app_settings  → 17 passed; 0 failed (6.26s)
cargo test -p game-core --lib touch_ux          → 4 passed; 0 failed
```

## §1 值循环语义 — 🟢 绿（含 2 黄点）

**点按 = 摇杆模式⇄触屏模式文本切换**：✅
- 切换入口 = 行 toggle：`menu_family.rs:1494-1498` `toggle_shell_row` 翻 checked →
  `runtime.rs:429-444` `sync_control_rows` **同拍**检测翻转、重生成行集（`shell_rows`，
  `runtime.rs:33-63`）→ `replace_shell_rows`（`scene.rs:5092-5105` / `paint.rs:827-840`）。
- 值文本随模式重建：`runtime.rs:46-53`——`control_mode == Touch ⇒ "触屏模式"，否则 "摇杆模式"`。
- 循环边界：`ControlMode` 仅 `Pad/Touch` 两值（`settings.rs:9-13`）⇒ 值循环即二态 toggle，无第三态。

**显示模式名不是 ✓/×**：✅
- 绘制分支：`menu_family.rs:1593-1601`——`row_value(i) == Some ⇒ draw_string(值文本)`，
  **else 才画勾叉精灵**；值行无勾叉 clip。
- 钉测：`menu_family.rs:2009-2041` `value_row_paints_value_text_not_checkbox`
  （ops 转义形 `\u89e6\u5c4f\u6a21\u5f0f` 断言 + 值行带 278..302 无 `setClip(..,12,10)` 断言）；
  `runtime.rs:746-757` `control_mode_row_shows_value_text`（触屏/摇杆双态 value_text）。

**切模式后立即生效**：✅（行为面）/ 🟡（测试面）
- 行集重建同拍：`runtime.rs:422-423`（step 末 sync + persist）。
- pad/触屏映射重建：`native.rs:1587-1591` 主循环每拍轮询 `rt_control_mode != wiring.control_mode`
  ⇒ `set_control_mode`（`native.rs:558-572`）⇒ `apply_portrait`（`:586-602`），
  其中 `:588` `TouchMapper::new(plan.pad)` = **映射重建**；触屏档画布/工具栏同拍生效（`:591-597`）。

**边界：循环中按住态清零**：🟡
- 结构上：TouchMapper::new 复位 ⇒ key_held 归零（`native.rs:557` 注释即此条款）。
- 🟡 **黄点 1**：「按住态清零 / 映射重建」**无钉测断言**——`app_settings.rs:568-581`
  `control_mode_switch_relayout_and_toolbar_action` 文档声称"TouchMapper 重建清按住态"，
  实际唯一断言是 `:578 rt_control_mode == Touch`；`native.rs` 测试模块（:1709-1743）
  3 测均不涉 set_control_mode。属"注释声称、断言缺位"。
- 🟡 **黄点 2**：`native.rs:567` `if plan == self.portrait… { return None }`——若两模式
  规划相同（边界输入），不重建 TouchMapper ⇒ 按住态不清零；无测试覆盖此边界。
  实际风险低（切换必在设置页、非按住行走态，且设置行走 pointer_hit 直通——批二审计表
  明示"设置行豁免·直通"），但边界行为无定义测试。

## §2 完全隐藏真伪 — 🟢 绿（破坏性验证双红实录）

**隐藏 = 不渲染 + 导航跳过（非灰显）**：✅
- 隐藏谓词：`runtime.rs:42-45`——`if s.control_mode == ControlMode::Pad` 才 push
  `PREF_HAPTIC/PREF_CLICK`；触屏下两行**不进 rows**（缺席，非 checked 灰显）。
- 不渲染：`menu_family.rs:1586` paint 遍历 `0..self.len()`，行不在集 ⇒ 无绘制。
- 导航跳过：`menu_family.rs:1477-1479` `len() = labels.len() + shell.len()`——键 -2/-1
  环绕以 len 为界，触屏 pointer_hit 行带同源（`row_band`，:1549-1556）⇒ 键盘方向键
  与触屏焦点**都不落在隐藏行上**（集内无此行，索引空间不含它）。
- 钉测：`runtime.rs:760-772` `touch_mode_hides_key_rows_auto_battle_always`
  （触屏集 `all id != HAPTIC/CLICK` + 摇杆回归 + 自动战斗常显）；
  `menu_family.rs:2045-2063` `hidden_rows_absent_from_list_len`（触屏 len=5 / 摇杆 len=7）。
- **切回摇杆恢复**：✅ `app_settings.rs:286-289` 实录断言
  `ids.contains(PREF_HAPTIC) && ids.contains(PREF_CLICK)` + 页重建回 7 行。

**破坏性验证 DV-A（删隐藏谓词）**：
```
改：runtime.rs:42  if s.control_mode == ControlMode::Pad { → if true {
cargo test -p game-android --test app_settings
  → control_mode_toggle_chain_rows_and_persist FAILED
    app_settings.rs:273 "触屏重建：原两行 + 壳 3 行（按键两行隐藏——件1 裁定）" 断言红
cargo test -p game-android --lib
  → runtime::touch_ux_rows_tests::touch_mode_hides_key_rows_auto_battle_always FAILED
    runtime.rs:765 断言红
还原：git checkout -- shell/android/src/runtime.rs → 8/8 绿
```
钉测红 ⇒ "完全隐藏"真，谓词被删必炸。✅

## §3 高亮锚随行 id（删行注入） — 🟢 绿（含 1 黄点）

**实现**：✅
- `menu_family.rs:1357-1364` `HighlightAnchor{Core(usize), Shell(u8), End}`；
  `:1518-1529` `highlight_anchor()`（壳行按 **id** 取锚）；`:1532-1541` `resolve_highlight()`。
- 接线：`paint.rs:827-840` / `scene.rs:5092-5105` `replace_shell_rows` 重建 options 时
  `anchor = old.highlight_anchor(); list.highlight = list.resolve_highlight(anchor)`。

**删行注入（生产路径）**：✅ Pad 7 行（高亮 idx5=操控方式）→ 切触屏重建 5 行
（删按键震动/按键音两行）⇒ `app_settings.rs:274` 断言 `highlight_index()==3`
= 触屏行序的操控方式行——**随 id=4 跟随，不漂移**；切回 `:284` 同钉（3→3）。

**破坏性验证 DV-B（废锚跟随）**：
```
改：menu_family.rs highlight_anchor() 恒返 HighlightAnchor::End（废 id 跟随）
cargo test -p game-android --test app_settings control_mode_toggle
  → FAILED，app_settings.rs:274 "高亮停在操控方式行（触屏行序 2/3/4——件1）" 断言红
    （End 语义钳尾 ⇒ 高亮漂到自动战斗行=idx4，漂移实锤）
还原：git checkout → 绿
```
锚钉对"删行漂移"红敏感 ⇒ 删行后高亮保持在操控方式行为**真**。✅

- 🟡 **黄点 3**：`menu_family.rs:1536` `resolve_highlight` Shell 臂
  `position(|r| r.id == id).unwrap_or(0)`——锚定行 id **不在**新行集时落**首壳行**
  （`labels.len() + 0`），与 Core 臂 min 钳位 / End 臂钳尾相比属**未定义回退**，无单测。
  当前 UX 流不可达（切模式必经操控方式行 id=4，恒在行集），但垃圾输入/未来行集变更
  下会静默漂移到首壳行。`paint.rs:1076-1104` `t3_replace_shell_rows_boot_and_scene`
  只钉 End 变体（越界钳尾），Shell 缺 id 路径零覆盖。

## §4 设置持久化（RMS / 隐藏行值保留） — 🔴 红（用户裁定条款未满足）

**模式选择落盘 + 重启保持**：✅
- 落盘：`settings.rs:82-87`（`control_mode=pad/touch` 解析）+ `:100-122`（save 写全键，
  tmp+rename 原子写）；`runtime.rs:229-238` `persist_app_settings_if_changed` 每拍末
  快照变化即写（`runtime.rs:423`）⇒ **切换同拍落盘**（`app_settings.rs:275-276` 实录
  `control_mode=touch`；`:291-292` 切回 `pad`）。
- 重启保持：`runtime_with` 链 = `Runtime::new → open_app_settings(load)`（app_settings.rs:22-28），
  与生产 `open_app_settings`（runtime.rs:221-226）同序 ⇒ 模式跨重启保持。✅

**隐藏行的值保存态不丢（切回摇杆原值还在）**：🔴 **红——原值丢失**
- 缺陷定位：`runtime.rs:208-209`
  `key_haptic: row_checked(rows, PREF_HAPTIC, d.key_haptic)`——触屏行集**缺**
  `PREF_HAPTIC/PREF_CLICK` 行 ⇒ `row_checked` 落 `d.key_haptic`（**默认 true**，
  `settings.rs:42-43`），**不是保留原值**。`key_click` 同病（:209）。
- 链式损坏（二段）：
  1. **落盘损坏**：切触屏同拍 `persist_app_settings_if_changed`（runtime.rs:423→229-238）
     把快照（key_haptic=默认 true）写盘 ⇒ 用户原值 false **当拍即被覆盖**；
  2. **回切丢失**：切回摇杆时 `sync_control_rows`（runtime.rs:438）`s = self.app_settings()`
     读到的已是默认 true ⇒ 重建的按键震动/按键音行 checked=true，原值没了。
- 对照口径不一致（实锤）：`auto_battle` 缺行取 `self.rt_auto_battle`（Runtime 保留字段，
  runtime.rs:435）⇒ 保值——`app_settings.rs:293` 专门钉了"行缺失 ⇒ auto_battle 值保留
  落盘（spec §8）"；而 key_haptic/key_click **没有对应 rt 保留字段**，缺行回默认。
  同一函数两种口径，隐藏行保值需求未被实现。

**破坏性验证 DV-C（临时钉，跑毕已删）**：
```
临时钉：cfg 写 key_haptic=false → 切触屏 → 断言落盘/快照/切回后原值仍在
cargo test -p game-android --test app_settings tmp_adv_hidden
  → FAILED（app_settings.rs:679 临时钉）
    第一红："触屏期落盘 key_haptic=false 不丢"——切触屏当拍 cfg 被改写为 true
    （调序复跑）"触屏期快照 key_haptic=false 不丢"——app_settings() 快照亦回默认
还原：git checkout -- shell/android/tests/app_settings.rs → 17/17 绿
```
裁定条款实测**不成立**。修复路由：回 porter——补 `rt_key_haptic/rt_key_click`
保留字段（对齐 rt_auto_battle 口径）或缺行时取 saved 值而非 default，并补钉
（模式往返保值 + 触屏期落盘不覆盖）。

## §5 app_settings 17/17 测试改写 — 🟢 绿（含 1 黄点）

**改写覆盖新行为**：✅（基线 17/17 绿实录 6.26s；diff 见 `git show cbcb647 -- shell/android/tests/app_settings.rs`）
- `expected_rows`：加自动战斗行 + 值循环语义注记（"checked 语义保留=触屏?false"）；
- 新增 `expected_rows_touch(splash)`（触屏 3 壳行集）；
- 行数断言改写：设置页 6→7（:239）、触屏重建 7→5（:273）、切回 6→7（:289）；
- 高亮锚断言改写：5→3（:274）、回程 3（:284）；
- 语义反转改写："切回摇杆⇒自动战斗行消失"→"**常显保留**"（:287，件1 裁定覆盖 T3.2）。

**旧 ✓/× 测试删干净**：✅
- 操控方式行旧勾叉显示断言已随行数/锚断言改写；值行显式断言**不画**勾叉精灵
  （menu_family.rs:2032-2040）；余留 `checked` 布尔断言是行内机器语义
  （toggle_shell_row 翻转目标 / expected_rows 输入），非显示层 ✓/× 符号断言；
  勾叉精灵断言仅存于真勾叉行（开屏画面/按键震动/按键音，
  `shell_rows_extend_box_rows_and_paint` menu_family.rs:1666-1706）——正确归属。

**破坏性验证 DV-D（§5 一例：值文本回退探测覆盖位置）**：
```
改：shell_rows 操控方式行 value_text Some("摇杆/触屏模式") → None（回旧 ✓/× 显示）
cargo test -p game-android --test app_settings → 17 passed（全绿！）
cargo test -p game-android --lib touch_ux
  → control_mode_row_shows_value_text FAILED（runtime.rs:746 钉红）✅
还原：git checkout → 17/17 + touch_ux 2/2 绿
```
- 🟡 **黄点 4**：值文本（"显示模式名"）行为**不在 app_settings 17 测覆盖内**——
  17 测改写钉的是行集/行数/高亮锚/落盘；值文本钉在 runtime 单测（:746）与 core
  touch_ux（menu_family.rs:2009）两处。功能有钉（换显示必红）但分散，
  "17/17 改写覆盖新行为"的字面承诺仅覆盖行集维度；commit message 表述
  "expected_rows+…断言"与实际范围相符，但复核口径记黄：建议值文本断言
  下沉一份进 app_settings rows_of（如 rows_of 带 value 列）。

## 总判

| 攻击点 | 判定 | 关键证据 |
|---|---|---|
| §1 值循环语义 | 🟢（2 黄） | 值文本绘制分支 menu_family.rs:1593-1601 + 双钉；映射重建/按住态清零无钉测（黄1）；plan 相等边界未覆盖（黄2） |
| §2 完全隐藏真伪 | 🟢 | 隐藏谓词 runtime.rs:42-45；**DV-A 双红实录**（app_settings.rs:273 + runtime.rs:765）；切回恢复 :288 |
| §3 高亮锚随行 id | 🟢（1 黄） | **DV-B 红实录**（app_settings.rs:274 漂移被抓）；resolve Shell 缺 id 回退未定义（黄3） |
| §4 设置持久化 | 🔴 | 模式持久化绿；**隐藏行值保持红**：runtime.rs:208-209 缺行回默认 ⇒ 落盘当拍覆盖+回切丢失（**DV-C 红实录**）；与 rt_auto_battle 口径不一致 |
| §5 17/17 改写 | 🟢（1 黄） | diff 改写属实 + 17/17 绿；旧 ✓/× 清干净；**DV-D**：值文本不在 17 测内（黄4） |

**总判：🔴 一件红——批一（件1 设置层级）不收口。**
用户裁定两条款之一"触屏下摇杆专属项隐藏后**保存态不丢**（切回摇杆原值还在）"
实测违反（DV-C）：`key_haptic/key_click` 在触屏期快照与落盘**双丢**回默认。
路由：回 **porter** 修复（缺行保值对齐 rt_auto_battle 口径 + 补往返保值钉）；
黄点 1/2/3/4 随批补钉（不阻收口，登记本复核）。§2/§3 的破坏性红实录证明
显隐谓词与高亮锚两枚钉真实红敏感，非"自证绿"。

**破坏性还原声明**：DV-A/B/D 触碰产品码、DV-C 触碰测试码，全部 `git checkout` 还原；
收尾复跑 `app_settings 17/17 + touch_ux 2/2` 绿实录（5.80s/0.00s）；`git status`
仅剩新增文档（含兄弟复核文档），无 tracked 文件改动。
