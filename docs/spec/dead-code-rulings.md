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
