# BUG-006 / BUG-008 根因重审：行为/观感判例的三层判定

> 框架（用户裁定怀疑顺序）：**① shim/移植层与真机不一致 → ② a.class→a.java 投影引入 → ③ 原版行为**。
> 证据源：`analysis/javap_renamed_a.txt`（字节码）、`reference/src/deobf/a.java`（投影）、
> `reference/shim/src/javax/microedition/lcdui/`（shim）、`assets/raw/script` + `assets/png/ui.png`（资产）、
> `crates/game-core/tests/{entity_golden,script_golden}.rs`（移植侧实证）。
> 取证日期：2026-10-16。证据分级照 AGENTS.md §3；推断显式标注 B 级。

## 0. 裁决摘要

| 判例 | ① shim/移植层 | ② 投影 | ③ 原版 | 重定性 |
|---|---|---|---|---|
| BUG-006 序章 SEE"死锁" | 排除（shim 无参与面）；**单列移植重实现分歧**（Rust find_path !found ≠ Java findPath found） | **排除**（A 级 javap 逐指令吻合） | **"死锁"定性撤销**：原版是交互式剧情点链（走到下一剧情点 step 触发续篇），"冻结"= 驱动脚本未走位 | 疑似非 bug（B+），待走位实验升 A |
| BUG-008 对话框观感 | **主嫌疑排除**（A 级：alpha 二值 + 变换码显式处理 + 微模拟器交叉）；残留 = 变换码 8462/270/90/8282 真机语义外证缺口（mode11 框体不走该路径） | **排除**（A 级：变换表 clinit + 绘制链参数与 javap 背书一致） | **维持**：石纹顶边+灰缝=资产内像素、无下边框=代码事实 | 维持 ③（Stage A 保真 / Stage B 美化） |

---

## 1. BUG-006 序章 SEE"寻路失败死锁"

### 1.1 定性沿革（保留历史主张）

- 台账原登记（2026-10-10）：`SEE_3_10_166_166_1` 玩家寻路失败、步数队列死写 ⇒ 永久卡住（B+，"真机是否同样卡住"未证实）。
- 【更正 1，移植侧】`entity_golden.rs:122-126`："原登记'寻路失败'——实为寻路成功（3 步路径存在），冻结来自行尾收尾门覆写 dialogPhase 6→4、使已武装的行走永远不被执行"；同文件 `bug006_replay_with_sprite51_entities_full_chain` 又**诚实地保留分歧**：Rust 重实现连实体槽一起算仍 `!found`，"walkStepCount=3 的来源需在 P3.3 行走模块考证"。
- 【更正 2，本文】下文 1.3 用 javap + 剧情点链把两说合并：oracle（=投影 Java+shim）实证 findPath **成功**（见1.2 证据 D），死锁定性不成立——续篇需要玩家**走到下一剧情点**，三种测试输入都没有走位。

### 1.2 A 级证据

**A. SEE 臂字节码 ≡ 投影（② 排除）**——`analysis/javap_renamed_a.txt` executeScriptInstruction 内
SEE 分派（`ldc "SEE"` @2110）后的处理：

```
2226: getfield  f_int_117        // seeModeFlag
2230: lookupswitch { // 2
        0: 2256
        1: 2259
        default: 2292 }
2256: goto 2292                   // case 0 → 公共尾
2259: ... m_133(playerCellX, playerCellY, f_int_114, f_int_115)  // findPath，offset 2276
2279: ifeq 2292                   // 失败 → 公共尾
2282: iconst_1
2284: putfield  f_bool_26         // pathArrowsVisible = true
2287: iconst_0
2289: putfield  f_byte_11         // walkPhase = 0
2292: bipush  6
2295: putfield  f_byte_16         // dialogPhase = 6（**case1 成功/失败均落到此**）
2298: ... f_int_114, f_int_115
2307: invokespecial m_103:(II)V   // easeCameraToTarget(dialogMoveX, dialogMoveY)
2310: goto 2386
```

≡ a.java:8500-8510（`case 1:` findPath→pathArrowsVisible+walkPhase，**fall-through** 到
`case 0/default:` dialogPhase=6 + easeCameraToTarget）——含 fall-through 语义逐指令吻合。

**B. 行尾收尾门 ≡ 投影（② 排除）**：

```
2386: getfield scriptCursor; ... String.length()
2397: if_icmpge 2431             // cursor ≥ len → 收尾块
2400: return                     // 否则返回（下拍续指令）
2401: iconst_4
2403: putfield f_byte_16          // dialogPhase = 4
...  m_104(0)（lockCameraOn，@2427）
```

≡ a.java:8594-8604。⇒ "phase 6→4 覆写"是**字节码事实**，非投影杜撰。

**C. MOV 玩家臂对照**（javap 357-380 ≡ a.java:8310-8314）：m_133 成功 ⇒ `f_bool_27=1`
（scriptWalkArmed）+ `f_byte_16=3`（dialogPhase=3，进 phase3 行走循环）+ walkPhase=0 +
lockCameraOn(0)。**SEE 臂不进这条路径**——SEE 只置 pathArrowsVisible（f_bool_26），不置
scriptWalkArmed、不进 phase 3。行走队列的消费点在 phase 3（a.java:4080-4085）与
箭头跟随修剪（a.java:5751-5761）。

**D. findPath 在 oracle 重放中成功（死锁定性撤销的核心证据）**：`pathArrowsVisible`
（f_bool_26）全库**唯一 `= true` 写点**在 SEE 成功臂（a.java:8508；其余 5355/5761 均为 false）。
oracle trace 冻结态 `f_bool_26=true` + `walkStepCount=3`（entity_golden.rs:124 引）⇒
**findPath(1,11→3,10) 在真实网格上返回 true、3 步箭头队列已生成**。
（Rust `find_path` 重实现 `!found` 是**移植层重实现分歧**，与死锁根因无关——m_134/m_062/
isCellWalkable 语义考证未结，见 1.4。）

**E. 剧情点链 = 原版推进设计**：`crates/game-core/tests/fixtures/sprite51.bin`（原 jar 提取）
解析出 9 个 type 83 剧情点，param 即脚本号：(1,11)→40、**(3,10)→41**、(6,11)→42、(7,10)→43、
(10,7)→44、(8,4)→45、(5,4)→46、(6,1)→47、(2,4)→48（复现命令同 evidence-todos.md 附录 B 口径）。
语料 `assets/raw/script`：line 40 = `TAK_255_259 TAK_165_165 SEE_3_10_166_166_1`，
line 41 = `TAK_167_167 SEE_4_10_168_168_0 ROS_4_1 SEE_2_8_169_169_0 SEE_2_8_170_170_1`。
触发链：**step** 踩中剧情点 → applyStepCellEffects case 83 `tryRunScene(param, true)`
（a.java:5710）→ 到达臂 `var3 &= var2`（a.java:8128-8147，case 41 在列）→ script 41 续篇；
bump 侧 `tryRunScene(param, false)`（a.java:5466）被 from_walk 门挡掉。

### 1.3 语义总结（三层判定）

- **① shim/移植层：排除**。SEE/寻路/行尾门/phase 机全是纯状态机，链上无 Graphics/InputStream/
  key API；oracle 三种输入脚本（狂按/节奏/静默）21k ticks 收敛同一状态 ⇒ 输入投递非变量。
  （单列：Rust `find_path` 与 Java `findPath` 结果分歧属**移植保真缺口**，不改变本判例定性。）
- **② 投影：排除**（A 级 javap，证据 A/B/C）——SEE 臂、行尾门、MOV 臂逐指令吻合。
- **③ 原版行为：死锁定性撤销**——SEE = 镜头 + 对话 166 + 路径箭头（f_bool_26 + 3 步队列），
  **设计即"玩家沿箭头走到 (3,10)、step 剧情点(param 41) 触发续篇"**；行尾门 phase 6→4 →
  cameraAtTarget → gameMode 3（a.java:4113-4116）= 回到自由玩法，不是挂死。
  三种测试输入均未走位到 (3,10) ⇒ "冻结"是**驱动缺口**（没按设计操作），不是游戏死锁。
- 可判定性：**机器可判定**——冻结态后发方向键走位到 (3,10) step，观测 scriptLineFlags[41]
  / gameMode / cursor 推进即闭合。

### 1.4 剩余判据与遗留

1. **走位实验（一锤定音）**：oracle 冻结态注入方向键走到 (3,10) 踩点 → 若 script 41 触发，
   BUG-006 销账为"非 bug（交互设计）+ 驱动缺口"；若路径实际不可达/踩点不触发，回到 ③
   死锁并以实验 trace 重审。
2. **walkStepCount=3 的写点归属**（P3.3 行走模块）：findPath 成功写 vs 此前寻路残留——
   证据 D 已证 findPath 成功，此项降为记录性考证。
3. **Rust find_path 重实现分歧**：m_134 可通行 = `isCellWalkable || m_062`（a.java:9967）的
   m_062 语义、实体槽闭包与 Java 对齐后同步 entity_golden 断言。
4. **"变异复核证伪过一次（冻结态）"与账面并存**：type-closure:167 记"变异复核未做（判据=
   帧锚+双分支单测）"；lead 口述曾变异复核被冻结态证伪。按更正体例两说并存：若变异法在
   冻结态下无观测面（后续 8898 拍帧全同），则变异法对该臂不可用，判据维持帧锚+单测。

---

## 2. BUG-008 mode11 对话框框体观感（石纹顶边/灰缝/右缘双柱/无下边框）

### 2.1 定性沿革（保留历史主张）

台账现定性："四层排除后为锚点 JAR 真实行为"（javap 切片常量 / 资产字节 / Rust=Java 逐像素 /
microemulator 2.0.4 交叉复现），真机差异唯一解释为渠道版本。本次按用户怀疑顺序 **① shim 优先**
重审绘制链，结果：① 主链排除、残留一处外证缺口（不影响 mode11 框体）；②③ 维持。

### 2.2 A 级证据

**A. 绘制链（a.java:2808-2870，paint mode11）**：`paintHudFrame(0, y, 80, h)` 框底 →
顶段平铺 `while (x<214) { setClip(x, y, 16, 16); drawImage(ui[8][0], x-26, y, 0); x += 16; }`
（2817-2824）→ `drawEdgePatch(ui[8][0], 214, y, 0, 0, 26, 16, 1)` 右缘镜像补丁（2826）→
`fillRect(80, y+16, 149, h-16)` 石纹填充（2829-2830）→ 右柱 `while (y<…) { setClip(229, y, 11, 16);
drawImage(ui[8][0], 176, y, 0); }`（2833-2837）→ 文本 `paintWrappedText(overlayText, 85, y+12, 129, h-12)`
（2863）——**全程无底边绘制**，框高 = 顶 16 + 填充 h-16。切片常量（26/42/53/11、214/176/229/
149/129/85）已由台账 javap 背书。

**B. alpha 语义分歧机制排除（① 主嫌疑落空）**：`assets/png/ui.png` = 64×16 调色板 PNG，
解码后 alpha 直方图 **{0: 86, 255: 938}**（二值，无半透明像素；PIL convert('RGBA') 复现）。
⇒ shim 的 src-over（Graphics.java:14 "alpha=0 跳过，alpha=255 直写"）与真机"alpha 当掩码"
两种合成语义在此资产上**逐像素等价**——半透明合成差异机制不存在。

**C. 灰缝线 = 资产内不透明像素**：顶段平铺取 ui[8][0] 列 [26,42)；row 8 采样：tile 左缘
col26-27 = (41,50,49,255)，右缘 col41 = (33,38,32,255)，下一 tile 起点 col42 = (41,50,49,255)——
**值阶跃 (33,38,32)→(41,50,49) 每 16px 重复 = 玩家所见"灰色接缝线"**，全部 alpha=255。
⇒ 任何合规渲染器（含真机）同显，非渲染链产物。

**D. 变换链（右缘双柱）**：`drawEdgePatch`（a.java:4567-4597）case 1 →
`drawImageWithNokiaTransform(img, x, y, 1)` → `f_DirectGraphics_00.drawImage(…, f_int_arr_01[1])`
（4564）。变换表 clinit 字节码（javap static{} 0-51）：`{0, 8192, 16384, 24576, 8462, 270, 90, 8282}`
≡ a.java:44（② 投影忠实）。shim `drawImageTransformed`（Graphics.java:349-396）显式处理
0/1-7/8192/16384/24576（8192=FLIP_HORIZONTAL，注释"诺基亚常量，真机有效"）；
**8462/270/90/8282 落 `default` = 无变换静默处理**。

### 2.3 语义总结（三层判定）

- **① shim：主嫌疑排除（A 级）**——alpha 机制不存在（证据 B）、灰缝与渲染无关（证据 C）、
  mode11 框体变换只走 8192 且显式处理（证据 D）+ 微模拟器交叉复现同构。
  **残留 ① 风险（中低，外证缺口）**：变换码 8462/270/90/8282 的真机语义未证——shim 按
  无变换兜底，若真机语义为旋转/镜像，凡走 drawEdgePatch case 4-7 的界面会与真机分歧；
  **mode11 框体只用 case 1，不受影响**。8192 = 水平翻转的真机语义目前是 shim 注释级证词。
- **② 投影：排除（A 级）**——变换表 clinit 逐值吻合（证据 D）、绘制链常量/图序吻合
  （证据 A + 台账 javap）。
- **③ 原版：维持**——石纹顶边+灰缝为资产字节（证据 B/C）、右缘双柱为 drawEdgePatch 镜像
  补丁（证据 D）、"贴屏底无下边框"为代码事实（证据 A）。真机观感差异的残余解释 =
  渠道版本差异或记忆误差（台账结论保留）。
- 可判定性：**机器可判定**（像素 diff：ui[8][0] 平铺帧 vs 真机截图——待真机素材）；
  变换码真机语义需实机/官方 API 文档（外部证据）。

### 2.4 剩余判据与遗留

1. **变换码外证**：8462/270/90/8282（f_int_arr_01[4..7]）的真机 DirectGraphics 语义——
   查 Nokia DirectGraphics API 文档或实机截图；顺带固化 8192/16384/24576 语义
   （当前 shim 注释级）。若证实 shim 兜底错误，按 ① 重开 BUG-008 的**非框体**界面判例。
2. **真机对照**（台账原路线）：取得真机截图/他版本 jar 后与 assets/png/ui.png 平铺帧 diff。
3. Stage B 美化（统一边缘+补底边）仍按 D2 排期，不影响 Stage A 保真。

---

## 3. 方法论沉淀（供 bug-ledger 根因字段沿用）

1. **判"死锁"前先判"驱动是否按设计操作"**：剧情点链（step 触发）类推进，输入脚本没走位
   ≠ 游戏挂死——重放实验要按设计意图驱动（走位/踩点），否则把驱动缺口记成原版 bug。
2. **投影是否忠实，用 javap 逐指令对表**（lookupswitch 分派、fall-through、常量表 clinit），
   不靠"反编译看着合理"；fall-through 语义是误读高发区（参 gotcha_tryrunscene-fallthrough-arms-misread）。
3. **观感差异先分解"资产像素 vs 合成语义 vs 几何参数"**：alpha 直方图/切片采样一行 Python
   即可把"资产自带"与"渲染链引入"切开；合成语义分歧只在半透明像素上可观测。
