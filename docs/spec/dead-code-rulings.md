# 白项 b 类死代码裁决书（判据①收口，2026-10-15）

> 状态：**终裁 ☠️**（10 方法 / 4 裁决）。对象 = [覆盖矩阵](../coverage-matrix.md) §4 白项 b 类。
> 判据位置：[convergence-plan](convergence-plan.md) §0.1（白项清零）——☠️ 方法不计判据①赤字。
> 机械复核：`python3 tools/dead_code_ruling.py --check`；`tools/cover_matrix.py` 每次渲染同调三重校验
> （字节码证死 + 本文 `## DC-n` 节点名该方法且四段俱全 + JDI 全场景零触达），任一不成立即在矩阵打 ⚠ 并退回「待裁」。
> 更正规则（AGENTS.md §3.6）：本裁决若被推翻，原地标注日期更正，不删原文。
> 名称与行号口径：dc470b2（语义命名第二批）之后的改名类与 deobf（a.java 10883 行）。裁决时名 m_093 已改名
> **drawLeftTriangleArrow**（N-0443）；m_107、m_148-m_154、addSpark 保留原名。工具按台账旧名→现名自动映射。

## 0. 公共框架

**前提（闭世界，逐条可查）**

1. 原版 JAR 仅 2 个 class（a、CMidlet；`python3 gates/cli.py status` 的 assets-manifest 基线 classes=2）；
   CLDC 1.0 无反射、无动态类装载。
2. 被裁 10 方法全部 `private`（m_154 为 private static）——只能被类 a 自身 invokespecial/invokestatic 调用。
3. 类 a 的字段只经 putfield 写入；CMidlet 字节码对 a 的字段零 putfield（只写自身 f_a_00 / f_CMidlet_00）。
4. 入口点 13 个 = 平台可回调的 a 的全部非 private 方法（`<init>`、paint、run、showNotify、hideNotify、
   keyPressed、keyReleased）+ CMidlet 全部 5 个方法 + a 的 `static {}`。

**推理（保守上近似，`tools/dead_code_ruling.py` 头注）**

- `javap -p -c -s` 解析两 class → 逐方法 CFG（顺序 / 条件 / goto / switch 边 + 异常表边）。
- 字段值域 V(F) = 存活代码中**可达**的 putfield F 写点的产生值并集 ∪ {0}（JVM 缺省值）。产生值只认 putfield 的
  直接前驱为整型常量（iconst/bipush/sipush/ldc int）或 `getfield G`（拷贝 V(G)），且 putfield 本身不是跳转目标；
  任何其他形态 ⇒ V(F) = ⊤（不据此剪枝）。
- 死边：`getfield F; tableswitch/lookupswitch` 中 V(F) 全部到不了的目标；`getfield F; if<cond>` 与
  `getfield F; 常量; if_icmp<cond>` 中对 V(F) 恒不发生的分支（守卫指令是跳转目标 ⇒ 不剪）；另加 DC-2 的 MVS 引理死边。
- 方法存活 = 入口点 ∪ 存活方法中**方法内可达**调用点的被调者；值域与存活集交替迭代至不动点（实测 3 轮）。
  每一步都是真实执行集合的上近似 ⇒ 终态判死的方法在任何执行中都不会被调用。
- 结果：168 方法（a 163 + CMidlet 5）中证死 **10 个，恰为 b 类 10 方法**（无多无少）；其余 158 方法在本上近似下
  不能证死（≠ 已证可达）。

**动态旁证（A 级）**：tools/method_reach JDI 实测 51 场景（scenarios/ 44 + 夹具场景 7）对这 10 方法零命中
（`tools/method_reach/snapshot.json`；cover_matrix 渲染时校验，任一命中即判与裁决矛盾）。

**共用可复现命令**

```bash
python3 tools/dead_code_ruling.py           # 完整报告：每调用点的充分切断边、守卫字段值域、MVS 引理 L1-L5
python3 tools/dead_code_ruling.py --check   # 机械复核（退出码 0 = 全部成立）
J="javap -p -c analysis/rename-pipeline/renamed/a.class"   # 改名类（python3 tools/rename-pipeline/run.py 再生）
f() { $J | grep -B1 "putfield.*Field $1:" | grep -v 'putfield\|^--' | sed -E 's/^ *[0-9]+: *//' | sort | uniq -c; }
```

## DC-1 道具轮盘箭头 m_093 / drawLeftTriangleArrow（walkPhase 3 臂）

**主张**：m_093（现名 drawLeftTriangleArrow；a.java:8034-8036，fillTriangle 三角箭头）在任何执行中都不会被调用。

**证据**

- 调用点仅 2 处，均在 paintPlayerSprite：@1062、@1103（deobf a.java:5980/5985），位于 `switch (walkPhase)`
  （@812 getfield walkPhase → @815 tableswitch 2..5）的 case 3 臂（@970，deobf a.java:5969 起，道具轮盘绘制）。
- walkPhase 写点 23 处：iconst_0×13（其中 run@3307 位于死臂内）、iconst_1×4、iconst_2×1、iconst_4×3、iconst_5×2——
  无 iconst_3、无计算值写入 ⇒ V(walkPhase) = {0,1,2,4,5} ∌ 3 ⇒ @815→@970 死边。
- 同一守卫亦封死 run() `switch (walkPhase)`（@2501 tableswitch 0..5）的 case 3 键处理臂（deobf a.java:3647-3673：
  轮盘光标 f_int_96 ±1 与 useItemStack 快捷使用）；f_int_96 的 4 个写点全在该死臂内 ⇒ V(f_int_96) = {0}
  （@1034 ifle 亦单独切断 @1062）。
- 动态：51 场景零命中。

**可复现命令**

```bash
f walkPhase                                   # 13 iconst_0 / 4 iconst_1 / 1 iconst_2 / 3 iconst_4 / 2 iconst_5，无 iconst_3
$J | grep -c 'Method drawLeftTriangleArrow'   # 2（均在 paintPlayerSprite；裁决时名 m_093）
python3 tools/dead_code_ruling.py | sed -n '/^## DC-1/,/^## DC-2/p'
```

**结论**：☠️ 死代码（A 级字节码 + B 级值域推理 + A 级 JDI 零触达）。Stage A 保真：Rust 不移植 m_093
（drawLeftTriangleArrow）与 walkPhase 3 两臂（无可观测行为可复写）；Stage B 若欲启用道具轮盘，属新功能另行立项。

## DC-2 DSL 第 16 码 MVS：m_107

**主张**：executeScriptInstruction 的 MVS 臂（deobf a.java:8519-8592，`!var3.equals("MVS")` 的 else 臂：型 82
传怪点配对换格）及其唯一被调者 m_107（a.java:8872-8878，实体换格）永不执行。

**证据**（MVS 引理，字节码逐步，工具逐条 ✓）

- L0 比较点 executeScriptInstruction@663-669：`aload_3; ldc "MVS"; String.equals; ifeq 1185`——相等分支
  （fall-through @672）即 MVS 臂；m_107 两调用点 @1001/@1160（deobf a.java:8567/8585）只在该臂内。
- L1 局部 3 的引用写入仅 @1（aconst_null）与 @33（`aload_1; iload_2; iload_2; iconst_3; iadd;
  String.substring(II)` = 参数 1 的 3 字符窗口）；方法内 8 处 astore_1（混淆器局部槽复用）均不能前向到达 @25 的
  接收者装载 ⇒ 比较串 = 脚本行参数的子串或 null。
- L2 executeScriptInstruction 全部 3 个调用点（run@5054、自递归 @334/@1796）首参均为 `getfield dialogScriptLine`。
- L3 dialogScriptLine 唯一写点 loadLevelScript@42 = `levelScriptLines[currentScriptIndex]`（aaload）。
- L4 levelScriptLines 唯一写点 `<init>`@6883 = `anewarray String[68]` + 68×（ldc 字符串常量; aastore）；其余 2 处读取
  （`<init>`@12252 arraylength、loadLevelScript@34 aaload）无元素写、数组引用不外泄 ⇒ 脚本文本源封闭。
- L5 68 条脚本串（deobf a.java:1124-1193）含子串 "MVS" 0 条（DSL 码频次：TAK 81 / CES 48 / MOV 47 / DES 40 /
  ROS 30 / SEE 30 / GUT 7 / GIN 4 / SWD 3 / ADD 1 / END 1 / GLV 1 / LAY 1 / RES 1 / SMS 1）。
- ⇒ 相等分支对任何执行不成立；动态：51 场景零命中。

**可复现命令**

```bash
python3 tools/dead_code_ruling.py | sed -n '/^## MVS 引理/,$p'   # L1-L5 逐步 ✓
$J | grep -n 'String MVS'                                        # 唯一比较点
python3 - <<'EOF'   # L5 数据侧复核（deobf 投影）
import re; s = open('reference/src/deobf/a.java', encoding='utf-8').read()
b = s[s.index('this.levelScriptLines = new String[]{'):]; b = b[:b.index('};')]
ls = re.findall(r'"((?:\\.|[^"\\])*)"', b); print(len(ls), sum('MVS' in l for l in ls))   # 68 0
EOF
```

**结论**：☠️ 死指令 + 死方法。[type-closure](type-closure.md) §2 DSL 码表补第 16 码 MVS（☠️）。Rust：
crates/game-core/src/script.rs 已注「数据域 68 条脚本零使用」，保持不移植。

## DC-3 联网簇 m_148-m_154（gameMode 99 + f_bool_31 自门控）

**主张**：m_148（连接关闭）、m_149（u16 读）、m_150（u32 读）、m_151（HTTP POST）、m_152（状态读）、m_153
（轮询/重试状态机）、m_154（hex 解码）全部永不执行。

**证据**

- 簇外调用点只有 run() 的 5 处：@12（m_151，run 入口 `if (f_bool_31)` 臂，@1 getfield f_bool_31 → @4 ifeq）与
  gameMode 99 臂内 @6458（m_148）/ @6499（m_153）/ @6516、@6524（m_152）（deobf a.java:4257-4273）。其余调用点全在
  簇内（m_150→m_149；m_151→m_148/m_154；m_153→m_148/m_149/m_150/m_152）。
- gameMode 99 不可达：putfield gameMode 54 处——常量 {0,1,2,3,4,5,7,8,9,10,11,12,14,15,16,17,19,20,21,22}
  （另 run@6470/@6542 两处 iconst_0 在死臂内不计）；计算写入 3 类：menuReturnMode（iconst_1×4 / iconst_4×4 ⇒
  {0,1,4}）、loadTargetMode（m_131@2 iconst_3 ⇒ {0,3}）、inGameMenuReturnMode（run@2482 ← gameMode，自闭包）
  ⇒ V(gameMode) ∌ 99 ⇒ run@674 tableswitch（0..99）→@6424 死边（paint@30 同理封死 paint case 99 特殊屏）。
- f_bool_31 自门控：写点 4 处——`<init>`@15167 iconst_0（存活）、run@6464/@6508 iconst_0 与 m_153@407 iconst_1
  （均在死代码内）⇒ V(f_bool_31) = {0} ⇒ run@4 ifeq 恒跳 ⇒ m_151 不可达；且即便 mode 99 可达，其臂先要求
  f_bool_31 已真（deobf a.java:4258），而唯一置真点在 m_153 内——自锁（2026-10-15 独立审计补强）。
- f_int_165 写点 7 处全在死代码内 ⇒ {0}（@6480 if_icmpge 是 m_153 调用点的又一重切断）。
- 另：master-plan §4 范围裁剪（HttpConnection 联网整体剔除）。动态：51 场景零命中。

**可复现命令**

```bash
f gameMode                    # 无 bipush 99、无 bipush 18、无 iconst_6/bipush 13；getfield 来源 3 类
f menuReturnMode; f loadTargetMode; f inGameMenuReturnMode
f f_bool_31                   # 3 iconst_0 / 1 iconst_1（唯一置真点 = m_153@407）
python3 tools/dead_code_ruling.py | sed -n '/^## DC-3/,/^## DC-4/p'
```

**结论**：☠️ 死代码簇（m_148 / m_149 / m_150 / m_151 / m_152 / m_153 / m_154 共 7 方法）。mode 99（paint/run 两臂）
一并 ☠️（[completeness](completeness.md) §3.1 mode 99 行）。

## DC-4 火花层 addSpark（paint gameMode 18 臂）

**主张**：addSpark（a.java:9683-9689，火花登记）永不执行。

**证据**

- 调用点 4 处 paint@5126/@5141/@5156/@5172（deobf a.java:3019-3031），全在 paint `switch (gameMode)`
  （@27 getfield gameMode → @30 tableswitch 0..99）的 case 18 臂（@5026，deobf a.java:3003-3038）。
- V(gameMode) ∌ 18（同 DC-3：无 bipush 18，计算来源值域 {0,1,3,4} 与自闭包）⇒ @30→@5026 死边。
- 区分：标题/引子粒子（updateAndDrawParticles，mode 1/14 的 paint 调用 a.java:2374/2955）存活且已 L3；死的是
  case 18 火花层（completeness §3.1 mode 18 行 2026-10-15 更正）。动态：51 场景零命中。

**可复现命令**

```bash
f gameMode | grep -c 'bipush *18$'     # 0
python3 tools/dead_code_ruling.py | sed -n '/^## DC-4/,/^## 守卫/p'
```

**结论**：☠️ 死代码；mode 18 ☠️（同 13/99）。

## 附：裁决范围外的相邻结论（不在 b 类，免混淆）

- **hideNotify 不是死代码**：字节码唯一调用者 CMidlet.pauseApp@4，另由平台在 Canvas 离屏时回调（来电/切后台必达）；
  只是 oracle 场景驱动面不含生命周期事件 ⇒ 场景不可达。矩阵按黄项「不触达论证」处置（§3，宿主测试覆盖），不入本裁决书。
- **mode 13**：gameMode 值域同样不含 13（DC-3 值域），paint case 13 仅视差背景、无专属方法——随 mode 99/18 一并 ☠️
  （completeness §3.1）。

## DC-5 scene67 混沌引子 TAK_264_268 不可达 ☠️（2026-10-16 终裁）

> 【2026-10-16 更正】本节初版（task-7，当日上午）判「✗ 工具无法证明，留待人工」——当时
> tools/dead_code_ruling.py 无存档往返引理，difficultyIndex 值域收敛 ⊤。当日补「**存档往返
> 归纳引理**」（L1-L5 机械校验）+ **DC-5 路径证明**（P1-P3）后全过 ⇒ **终裁 ☠️**。初版主张与
> 缺口清单原样保留在 §史（不删历史）。背景见 [evidence-boss.md](evidence-boss.md) §4.2。

**主张**：tryRunScene case67 门控 `var3 &= this.difficultyIndex > 0`（deobf a.java:8219-8220）任何
正常执行恒 false ⇒ 型83(param67) 触发不成立 ⇒ `loadLevelScript(67)` 不可达 ⇒ script67
`"TAK_264_268 "`（a.java:1192；对话 264-268 = 混沌世界引子，speaker 87 菩提老祖，a.java:1459-1463）
永不执行。

**终裁证明（2026-10-16，扩引理后工具输出逐字）**：☠️ 证死。`python3 tools/dead_code_ruling.py
--check` → `死代码裁决复核：全部成立 DC-1/DC-2/DC-3/DC-4/DC-5`（exit 0）。

**存档往返归纳引理（L 系，MVS 引理同型机械校验）**：

- L1 ✓ putfield a.difficultyIndex 共 3 处 = 恒定 2 处（<init>@14117=0, resetNewGameState@142=0）+
  非恒定 1 处（loadFloorSave@99：`getfield f_DataInputStream_00; invokevirtual readByte()B` 且链内无跳转目标）
- L2 ✓ 流血统：f_DataInputStream_00 / f_byte_arr_27 非空 putfield 6 处均为
  `new ByteArrayInputStream(getfield f_byte_arr_27)` / `RecordStore.getRecord` 形态
  （dispatchModeEntry@157✓, @179✓, loadSlotTable@145✓, @167✓, loadFloorSave@66✓, @88✓）
- L3 ✓ 槽 1 配对 + 自拷贝：loadFloorSave 首流读@96 → putfield difficultyIndex@99；
  saveFloorState 首流写@72 = `writeByte(getfield difficultyIndex)` 自拷贝
- L4 ✓ 记录库写口封闭：MOT_L 库写者 = ['saveFloorState']（唯一）；其余写口 MOT_IF / SKY_WAR 不同库
- L5 ✓ 归纳：difficultyIndex 全写点 = {0, 0, 自拷贝} ⇒ 读回值 ∈ 字段历史值集；初值 0（<init> 写 0
  + JVM 缺省）⇒ ≡ 0（跨会话持久同归纳：存档值 = 某次存档时刻的字段值）
  ⇒ 非恒定写点按 ('copy', a.difficultyIndex) 计 ⇒ **值域 {0}**（含缺省 0）

**DC-5 路径证明（P 系）**：

- P1 ✓ 值域(a.difficultyIndex) = {0}（存档往返引理生效）
- P2 ✓ tryRunScene(67) 恒 false：tableswitch 键 67→@880；ifle@885 fall→iconst_1@888 ∈死边表，
  jump→iconst_0@892（恒 0 ⇒ iand ⇒ var3=0 ⇒ ireturn 0）
- P3 ✓ loadLevelScript 实参清点：常量=[20, 21, 22, 35, 36, 65]（∌67）；动态点 2/2 受同参
  tryRunScene 守卫（interactWithCell@631、applyStepCellEffects@778）；GUT 语料目标=
  [1, 12, 37, 57, 61, 62, 63]（∌67，executeScriptInstruction@421 解析臂）
- ⇒ loadLevelScript(67) 无可达调用点 ⇒ script67 永不装入 ⇒ TAK_264_268 死。

### 史：初版缺口清单（task-7 当日上午所记，已被上列引理/证明闭合，留档不删）

**工具结论**（初版）：`python3 tools/dead_code_ruling.py` 引擎实测 **两条证明链各差 1 处模型闭合点**：

### 链 1：tryRunScene(67) ≡ false —— 被 difficultyIndex 值域 ⊤ 阻断（A 级工具输出）

- 字段值域（`field_evidence` 原文）：
  `difficultyIndex：putfield 3 处 = iconst_0×2，invokevirtual #1374×1 ⇒ 值域 ⊤（含缺省 0）`
- 写点明细（引擎 `an['writes']` 原文）：
  - `<init>@14117`：prev=`iconst_0` src=0 counted=True
  - `resetNewGameState@142`：prev=`iconst_0` src=0 counted=True
  - `loadFloorSave@99`：prev=`invokevirtual #1374`（= DataInputStream.readByte，deobf a.java:9258）
    src=('top', 'invokevirtual #1374') counted=True
- 机制：引擎 putfield 产生指令只认「常量 / getfield 拷贝」两形态（`const_of` / `field_writes`），
  readByte 调用形态 ⇒ 值域 ⊤ ⇒ `dead_edges` 对 difficultyIndex 守卫全部不剪（与 DC-1/DC-3 的
  walkPhase/gameMode 常量写点不同，本字段多一个**非常量写点**）。
- tryRunScene 死边清单（引擎 `an['dead']` 原文）= **空**；可达偏移 **373/373**（全可达）⇒
  case67 段（@885 fall 边）未被剪。
- case67 守卫字节码（javap，tryRunScene@880-894）：

```
880: iload_3
881: aload_0
882: getfield      #795   // Field difficultyIndex:B
885: ifle          892
888: iconst_1
889: goto          893
892: iconst_0
893: iand
894: istore_3
```

  ⇒ 若值域能收敛 {0}，引擎将把 @885 fall 边（→888 iconst_1）剪为死边、case67 恒 false——
  **只差链 1 的闭合引理**（对照 case16 守卫 @857-871 `getfield; ifne` 同因不剪）。

### 链 2：loadLevelScript(67) 无其它入口 —— 被动态参数阻断（A 级）

loadLevelScript 调用点实参形态（javap）：
- 2 处 `getfield entityParam; saload`（bump 臂 a.java:5469 / 到达臂 a.java:5712，参数 = **关卡数据**，
  静态无界）；
- GUT 臂 `iload`（a.java:8313，实参 = 脚本文本解析值）；
- 常量实参 `bipush` = {20, 21, 22, 35, 36, 65}（tickBattle 族，a.java:10073-10152）——**不含 67** ✓。
- GUT 目标语料（A 级，grep `GUT_[0-9]*` 7 处）= {1, 12, 37, 57, 61, 62, 63}——**不含 67** ✓。
- ⇒ 唯一潜在入口 = entityParam 动态路径，其成立性恰由链 1（tryRunScene(67) 恒 false）封死；
  GUT 语料边界属 MVS 引理 L5 同型论证（工具仅有 bespoke 的 MVS 引理，无通用数据域引理）。

### 缺口清单（初版所列；2026-10-16 已全部闭合）

1. **存档往返归纳引理**（已闭合 ✓）：difficultyIndex 写点 = {iconst_0×2, readByte×1}；readByte 的值源 =
   record store，而存档写点（loadFloorSave 的对偶 saveFloorState，deobf a.java:9181/9190）写回同一字段
   旧值 ⇒ 归纳恒 0。→ **已实现**：tools/dead_code_ruling.py `save_roundtrip_lemma`（L1-L5 机械校验，
   `--check` 纳入 DC-5）。
2. GUT 实参语料边界（L5 同型）与 entityParam 关卡数据普查——链 2 的收尾材料。
   （已闭合 ✓：P3 的 GUT 语料目标清点 + 2 个动态点同参 tryRunScene 守卫形状校验，均为机械检查。）

**可复现命令**

```bash
python3 tools/dead_code_ruling.py --check    # 含 DC-5：全部成立 DC-1/DC-2/DC-3/DC-4/DC-5
python3 tools/dead_code_ruling.py | sed -n '/^## 存档往返归纳引理/,$p'   # L1-L5 + P1-P3 工具输出
$J | grep -n 'difficultyIndex'                    # 写点 3 处 + getfield 若干
grep -o 'GUT_[0-9]*' reference/src/deobf/a.java | sort | uniq -c   # GUT 目标无 67
```

**结论**：**☠️ 死代码（scene67 / TAK_264_268 永不执行；型 87 菩提老祖混沌引子 = 死内容）**。
证明 = 存档往返引理 L1-L5 + 路径证明 P1-P3（上列工具输出）；`--check` 已含 DC-5，随
tools/dead_code_ruling.py 机械复核长期生效（引理任一步失效即复核失败告警，不得静默销账）。
初版「工具无法证明」作废（见顶部更正）。scene67 不需补用例；type-closure 出层列可据此销账。

---

## DC-6..DC-12 Rust 侧死码裁定批（2026-10-16，架构评估 task-22 附带产出）

> 背景：task-22 架构评估冗余扫描产出 7 个真死候选（`crates/*/src` 非 `#[test]` 且全仓
> 除定义行零出现）。本批按 MVS/82 先例（机械校验 + 语义等价锚）逐个三裁一：
> ☠️ 死码 / 🔒 保真保留（shim 表面积）/ ✅ 有活调用（假阳性更正）。
> **全部只裁定不删除**（AGENTS 保真期行为只增不减；删除另行派工）。
> 与 DC-1..5 不同：那些是 Java 字节码侧证死（`tools/dead_code_ruling.py` CFG 不动点）；
> 本批是 **Rust 移植侧零调用 + 语义由内联实现等值持有** 的机械证明，复核命令随附。

**统一机械校验（引用计数口径）**：`grep -rw '<name>' crates --include='*.rs' | wc -l` == 1（仅定义行）。
7 个候选在裁定时点全部满足；接入 `tools/dead_code_ruling.py --check` 的 Rust 侧复核表
（`RUST_DEAD`），任一函数被接线（计数 > 1）即告警复核，不得静默沿用旧裁定。

### DC-6 `ParallaxBackdrop::set_scroll`（menu_family.rs:72）☠️

**主张**：backdropScroll 字段在 Java 侧的**唯一写点**是 drawParallaxBackdrop 内自减回绕
（a.java:6862-6863：`if (--this.backdropScroll < -154) this.backdropScroll = 0;`），
无任何外部写入点。Rust 拆分出的写桥 `set_scroll`（"场景接管"方向）无 Java 对应语义。

**机械校验**：
- 零调用 ✓（统一口径）
- Java 写点清点 ✓：`grep -n 'backdropScroll' reference/src/deobf/a.java` = 字段声明:150、
  自减写:6862、清零写:6863、读:6873 —— 除构造缺省外无外部 putfield
- 读桥 `scroll()` 为活（40+ 处拆分同步：`sc.backdrop_scroll = m.backdrop.scroll()`），
  写方向在拆分架构下无源可传（scene 不回写 backdrop）

**裁定**：**☠️ 死码**（拆分后的单向同步桥，写方向无消费者；自减语义由 ParallaxBackdrop::paint
内联持有——menu_family.rs:86 `self.scroll -= 1;` + 回绕判定）。

### DC-7 `OverlayEngine::needs_layout`（menu_family.rs:405）☠️

**主张**：布局失效判定 `cached_text.as_deref() != Some(text)`（:406）已被 paint() 内联持有
（menu_family.rs:463 同表达式），提取方法无调用方。

**机械校验**：零调用 ✓；内联等价锚 ✓（:406 定义式 == :463 paint 内判定，逐字同构）。

**裁定**：**☠️ 死码**（冗余提取，语义内联持有）。

### DC-8 `LoadProgress::new_game_stats`（intro.rs:307）☠️

**主张**：返回 `(300, 10, 10, 51)`（a.java:3482-3489 步骤 8 新游戏初始状态）的数据函数零调用；
其数据已由调用方**内联等值**落实。

**机械校验**：
- 零调用 ✓
- 内联等价锚 ✓：game-desktop/main.rs:766-771 `sc.floor = 51; sc.min_floor = 51;
  sc.max_floor = 51; sc.player.hp = 300; sc.player.atk = 10; sc.player.def = 10;`
  （+ preset_cases.rs:2600/2810 `sc.player.hp = 300;`）——四元组 (300,10,10,51) 全数对上

**裁定**：**☠️ 死码**（数据已内联落实，值一致无行为缺口）。**附带卫生建议**（不阻塞裁定）：
Stage B 可将两处硬编码接线为 `new_game_stats()` 单一真相源，防未来漂移。

### DC-9 `combat::predict_hp_loss_vs_type`（combat.rs:107）☠️（附缺陷注记）

**主张**：predictHpLossVsType 参数桥，零调用。**且桥公式不忠实**：

| | atk 来源 | 依据 |
|---|---|---|
| Java `predictHpLossVsType`（a.java:9992-9995） | `effectiveAttackVsType(var1)`（含道具 23/24 二倍攻击加成，a.java:9997-10004） | 字节码语义 |
| Rust 桥 `predict_hp_loss_vs_type` | `player.atk`（裸攻击，**缺道具加成**） | combat.rs:117 |
| Rust 忠实实现 `GameScene::predict_hp_loss`（scene.rs:4813，**有活调用**） | `effective_attack_vs_type(&self.player, self.trait_flags(t))` | scene.rs:4816-4825 |

**机械校验**：
- 零调用 ✓
- Java 侧 predictHpLossVsType 活调用 7 处（a.java:5288/5476/8212/8774/8852/10048 + 定义9992）
  ⇒ Java 语义可达；Rust 对应行为由 scene.rs:4813 持有（公式与 a.java:9992-9995 逐项对上）
- 桥与忠实实现的差异仅在 atk 来源（桥缺 effectiveAttack 的 `*2` 道具加成）

**裁定**：**☠️ 死码（带缺陷的冗余桥）**——幸零调用，缺陷未传播；忠实语义已由 scene.rs:4813
`predict_hp_loss` 持有。**删除时不得"顺手修桥再接线"**：若未来遇敌预演 UI（a.java:8774/8852 臂）
需要此桥，应改用 effective_attack 公式（以本节对照表为准）。

### DC-10 `Camera::snap_anchor_to_player`（camera.rs:90）☠️

**主张**：lockCameraOn(0) 头部锚点公式 `anchor = ((view - 32) >> 1) - offset`（camera.rs:91-92）
零调用；同公式已由生产实现内联持有。

**机械校验**：
- 零调用 ✓（含 harness 测试亦无）
- 内联等价锚 ✓：scene.rs:4400-4401 `ease_cam_x = ((self.view.view_w - 32) >> 1) - self.view.cam_x;`
  （lock_camera_on 头部，a.java:8662-8663 语义）——与 camera.rs:91-92 逐字同构（字段名映射
  anchor_x↔ease_cam_x、offset_x↔cam_x）
- 附注：函数体 `let _ = (player_px, player_py);` 丢弃形参（Java 语义本不依赖玩家坐标，
  参数系移植痕迹）

**裁定**：**☠️ 死码**（语义由 scene.rs:4400-4401 内联持有；camera.rs 本身系对拍 harness 面，
walk_camera_golden/walk_replay_floor1 直驱其构件，本方法连 harness 都未使用）。

### DC-11 `BootMachine::set_rng_seed`（paint.rs:610）⏳ 三裁一外第四态——报 lead 裁

**主张**：Java `setSeed(VTime)`（a.java:2325）是 **a() 构造期活语义**——VTime 取值随前置行为
变化（preset 场景实测 11813，T73 粒子暴力反推；非 preset 场景 0）。Rust 侧：
- 构造固定 `JavaRandom::new_seeded(0)`（paint.rs:277）——与非 preset（VTime=0）**等价**；
- `set_rng_seed` 零调用 ⇒ **preset 路径的 11813 种子未落实**。

**为什么当前不可观测**：rng 唯一消费者 = mode14 runIntroViewer 粒子雨 4 分频
（paint.rs:528-532，a.java:4137-4136）；44 个 oracle 场景中 preset 系全部槽载入直入 mode3，
**不经过 mode14** ⇒ 11813 种子差异不产生帧差 ⇒ 44/44 对拍绿与本缺口并存不矛盾。

**机械校验**：
- 零调用 ✓
- rng 写点清点 ✓：`grep -n 'rng' crates/game-core/src/paint.rs` = 构造:277、消费:531-532、
  set_rng_seed:611-612 —— 无其他写点
- Java 对照 ✓：a.java:2325 `this.setSeed(VTime);`（构造期调用）

**裁定**：**⏳ 待裁（三裁一外）**——非死码（Java 语义活）、非 shim 表面积、非假阳性。
属**条件性行为缺口**：preset 路径若未来触达 mode14 或 rng 消费面扩展，粒子序列将与
Java 分叉。**请 lead 裁**：(a) 接线（preset 装载点调 `set_rng_seed(11813)`，VTime 来源
需考证常驻化）；(b) 登记已知偏差（docs/bug-ledger.md，BUG-xxx）；(c) 维持现状附本节注记。

### DC-12 `walk::tick_stepping`（walk.rs:90）☠️

**主张**：完整步进编排（camera_edge_follow + step_progress += 8 + 32px 归格 +
apply_step_effects hook，a.java:3561-3593）零调用；组成语义由两路分别持有。

**机械校验**：
- 零调用 ✓（scene 与 harness 测试均不经此编排）
- 内联等价锚 ✓：scene.rs:2371/2379-2380 `self.step_progress += 8; … >= 32 … = 0;`
  （生产步进内联）；`camera_edge_follow` 为活（walk.rs:53，walk_camera_golden/
  walk_replay_floor1 直驱对拍）
- 结构判定：walk.rs 模块整体系**对拍 harness 面**（golden 测试分件直驱构件与 Java trace
  对照），tick_stepping 是其上的编排便利层——连 harness 都未接线

**裁定**：**☠️ 死码**（编排冗余：语义由 scene.rs:2371-2380 内联 + camera_edge_follow（活）
分件持有）。

---

### 本批汇总

| DC | 函数 | 裁定 | 一句话依据 |
|---|---|---|---|
| DC-6 | ParallaxBackdrop::set_scroll | ☠️ | Java backdropScroll 无外部写点，写桥无消费者 |
| DC-7 | OverlayEngine::needs_layout | ☠️ | 判定式由 paint() 内联持有（:463） |
| DC-8 | LoadProgress::new_game_stats | ☠️ | 数据由 main.rs:766-771 内联等值落实 |
| DC-9 | combat::predict_hp_loss_vs_type | ☠️ | 零调用 + **桥公式缺道具加成（带缺陷）**，忠实实现=scene.rs:4813 |
| DC-10 | Camera::snap_anchor_to_player | ☠️ | 公式由 scene.rs:4400-4401 内联持有 |
| DC-11 | BootMachine::set_rng_seed | **⏳ 待裁** | Java setSeed(VTime) 活语义；preset 种子 11813 未落实（当前不可观测）——**报 lead** |
| DC-12 | walk::tick_stepping | ☠️ | 编排冗余，scene 内联 + 构件分件持有 |

**复核命令**：`python3 tools/dead_code_ruling.py --check`（含 RUST_DEAD 表零调用复核；
任一函数被接线即告警）。6 个 ☠️ 不需补用例；DC-11 等 lead 裁后另行销账。
