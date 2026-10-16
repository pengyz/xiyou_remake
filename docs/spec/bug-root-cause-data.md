# bug 根因重审：数值/映射判例 + 死资源类快查（task-18）

> 裁定框架（用户拍板，2026-10-16）：怀疑顺序 **① shim → ② 投影 → ③ 原版**。
> 本文件对 BUG-004/007 两个数值/映射判例做 javap 字节码级根因重审，对 BUG-001/002/005/010/003
> 死资源/死分支类做打包快查。每条给三层排除（①shim/②投影/③原版）+ 字节码证据 + 台账建议。
>
> 证据口径：javap = `javap -p -c -s analysis/rename-pipeline/renamed/a.class`（改名类，
> dump 行 = 该输出内行号，字段号 = 常量池号）；seed = `reference/seed/a.java`（原始反编译投影，
> 单字母字段无签名区分）；deobf = `reference/src/deobf/a.java`（现行改名版，行号随再生成漂移，
> 以 dump 行为主锚）。**① shim 层排除通则**：shim 仅提供 MIDP 兼容（RecordStore/Media/Canvas），
> 不触碰游戏逻辑字段与常量池 ⇒ 任何"常量池字段引用/表序/常量值"级事实天然非 shim 引入。
> 取证日期：2026-10-16。

---

## 0. 判定速览

| bug | 问题 | ① shim | ② 投影 | ③ 原版 | 结论 |
|---|---|---|---|---|---|
| BUG-004 | 战斗 HP 初始化混用基础表/缩放表 | 排除 | 排除 | **确认** | ③ 原版数值设计不一致（字节码级两字段混用） |
| BUG-007 | 主菜单 3/4/5 项文字-动作错位 | 排除 | 排除 | **确认** | ③ 原版映射错位（双分派表互不一致） |
| BUG-001 | script 资源死数据 + 2 处内容差 | 排除 | 排除 | **确认** | ③ 原版打包冗余/数据漂移 |
| BUG-002 | "输入模式字段恒 0 死分支" | 排除 | **②伪像（证伪）** | — | ② seed 单字母签名歧义误认字段 + grep 漏检赋值；分支实为活码 |
| BUG-005 | 难度倍率表 1..4 槽不可达 | 排除 | 排除 | **确认** | ③ 原版死内容（DC-5 值域证明同源） |
| BUG-010 | logostart/logoquit BGM 死资源 | 排除 | 排除 | **确认** | ③ 原版死资源（实参恒 2/3） |
| BUG-003 | GUTS 负数参数 DES 指令 | 排除 | 排除 | **确认** | ③ 原版 DSL 设计语义（DES 负参=全型清除） |

---

## 1. BUG-004：战斗 HP 初始化混用两表（③ 原版确认）

### 1.1 事实：三个独立常量池字段

| 字段 | 常量号 | 角色 |
|---|---|---|
| `enemyBaseHp:[I` | **#1152** | 基础表（35 项） |
| `enemyHpScaled:[I` | **#1185** | 缩放表（scaleEnemyStats 写 = 基表 × 难度倍率） |
| `battleEnemyHp:I` | **#988** | 战斗中敌当前 HP |

### 1.2 两处初始化字节码（核心证据）

**A. 玩家主动战（bump 开战）→ 基础表**（interactWithCell，deobf a.java:5484）：

```
javap（interactWithCell）:
   208: getfield      #1152   // Field enemyBaseHp:[I
        …
   223: putfield      #988    // Field battleEnemyHp:I
```

**B. 敌主动战（怪物走位撞上玩家）→ 缩放表**（m_106 实体路径缓动，deobf a.java:8775）：

```
javap（m_106）:
   281: getfield      #1185   // Field enemyHpScaled:[I
        …
   290: putfield      #988    // Field battleEnemyHp:I
```

m_106 的语义上下文（deobf a.java:8744-8780）：`routePhase` 走位臂——`var11/var14` 按
`f_byte_19` 步向换格后 `if (entityType[f_int_127] != 72 && var11 == playerCellX &&
var14 == playerCellY)` 命中 ⇒ 以 `entityType[f_int_127]` 开战（**敌人主动**）。

**C. 损血预测与 UI 显示都用缩放表**：predictHpLossVsType（deobf a.java:9986-9990）读
#1185；状态栏敌血显示（dump 13254 `3357: getfield #1185`）读 #1185。
**D. 缩放表来源**（scaleEnemyStats，dump 28587-28606）：`#1185[i] = #1152[i] × mult`。

### 1.3 三层排除

- **① shim**：排除——三字段与两 putfield 均在游戏类常量池内，shim 无写点。
- **② 投影**：排除——两处 getfield 的常量池号**不同**（#1152 vs #1185），deobf 把两处分别
  标注 enemyBaseHp/enemyHpScaled 与字节码一致；投影无法把同一字段"错读"成两个字段号。
- **③ 原版**：**确认**——原版字节码自身在两个开战入口用了不同表；预测/显示（缩放）与
  玩家主动战初始化（基础）不一致 ⇒ A=0 首周目两表相等不可观察，A>0 时预测与实战脱节
  （ledger 现象描述不变）。

**台账建议**（bug-ledger.md BUG-004，lead 执笔）：根因列改「③ 原版数值设计不一致——
字节码级双字段混用（interactWithCell@208-223 用 #1152 基表、m_106@281-290 用 #1185
缩放表；预测/显示均 #1185）；shim/投影已排除（常量池级证据）」。

---

## 2. BUG-007：主菜单文字-动作错位（③ 原版确认）

### 2.1 映射链四环（全部字节码级）

**环 1 标签表**（`menuLabels`，deobf a.java:458-459）：显式下标构建（javap `<init>`：
`dup; bipush N; ldc_w "…"; aastore` 逐项）——序 = 0 新游戏 / 1 继续游戏 / 2 载入进度 /
**3 保存游戏 / 4 设置 / 5 帮助** / 6 关于 / 7 退出 / … / 12-17 商店系。

**环 2 注册**（`addMenuItem`，deobf a.java:4696-4713）：

```
javap（addMenuItem）:  iload_1 → i2b → bastore menuItemKinds[menuItemCount]
deobf a.java:4699      this.menuItemKinds[this.menuItemCount] = (byte)var1;
```

⇒ **kind := 标签下标**（注册参数即标签索引，无独立映射表）。

**环 3 主菜单注册序列**（dispatchModeEntry，javap @376-406）：`iconst_0; addMenuItem` …
`iconst_5; addMenuItem` 连续 6 发 ⇒ 主菜单 = 标签 0..5。

**环 4 分派**（run() @1649，javap）：

```
1649: tableswitch { // 0 to 5
        0: 1688 → bipush 14      （新游戏 → mode14）
        1: 1714 → bipush  8      （继续游戏 → mode8）
        2: 1735 → bipush 16      （载入进度 → mode16）
        3: 1756 → bipush 15      （保存游戏 → mode15）
        4: 1777 → bipush 17      （设置 → mode17）
        5: 1798 → bipush 22      （帮助 → mode22）
      default: 1810 }
```

### 2.2 错位定位（内在于原版）

| 槽位 | 标签 | 分派目标 | 实际效果（oracle trace 实证，ledger） | 语义期望 |
|---|---|---|---|---|
| 3 | 保存游戏 | mode 15 | 帮助页（"游戏描述：…"，常量池串见 deobf a.java:4407） | 存档位页 |
| 4 | 设置 | mode 17 | 关于页（"版权所有：…"，a.java:4428） | 设置页 |
| 5 | 帮助 | mode 22 | 70 帧粒子淡出后退出（END=all-threads-exited） | 帮助页 |

**对照组（同一 kind 两张表不一致）**：游戏内菜单 `activateMenuItem`（deobf a.java:4731+
switch `menuItemKinds[menuCursorIndex]`）中 **kind 3 → gameMode 7 = 存档位页**（语义正确）。
即原版内"kind 3"在主菜单分派（→15 帮助文本）与游戏内分派（→7 存档页）指向不同——
错位不是单表排序问题，而是**主菜单分派臂常量与标签/姊妹表不一致**。

### 2.3 三层排除

- **① shim**：排除——标签串、注册序列、tableswitch 常量全在游戏类内。
- **② 投影**：排除——环 1 是显式下标 aastore（不可能被投影重排）、环 2 是单指令
  （iload_1→bastore）、环 3 是 iconst 序列、环 4 是 tableswitch 显式目标；四环各自
  忠实且**相互独立**，投影不可能"同时错位"四环使其自洽地复现出该 bug。
- **③ 原版**：**确认**——错位在原版二进制的分派臂常量（3→15/4→17/5→22）。

**台账建议**（BUG-007）：根因列改「③ 原版映射错位——主菜单 run() tableswitch 臂常量
（kind3→mode15/4→17/5→22，javap @1649-1798）与标签表（环 1）及游戏内分派
（activateMenuItem kind3→mode7）不一致；shim/投影已排除（四环独立字节码事实）」。
Stage B 修菜单的前置考证（真机行为）不变。

---

## 3. 死资源/死分支类快查（BUG-001/002/005/010/003）

### 3.1 BUG-001 script 死数据（③ 确认）

字节码可证：`getResourceAsStream` 全类 4 调用点——

| 调用点（deobf） | 实参形态 | 资源族 |
|---|---|---|
| a.java:4523 | `resourceNames[var1]` | 图包（javap `<init>` 表：sflogo/mapbg/map/actor/sptmap/sptprop/sptarm/sptenemy1/ui/xtq/menu/intro/face/sptenemy2/end/loa…——**无 "script"**） |
| a.java:9416 | `"maplv" + var3` | 地图 |
| a.java:9497 | `"sprite" + var3` | 层实体 |
| a.java:10188 | `f_String_arr_15[var1]` | BGM 表 |

⇒ 无任何调用点可构造 "script" 路径；脚本语料 = `<init>` 内嵌 `levelScriptLines` 字符串
数组（javap anewarray+ldc+aastore，68 项）。资源文件与内嵌数组的 2 处内容差（GUTS 第
27/63 行）系**原版打包期数据漂移**（③）——两者都是原版内部事实。
**①/② 排除**：资源枚举表与调用点均为字节码事实。

### 3.2 BUG-002 "输入模式字段恒 0"（**② 伪像，证伪**）

- seed 事实：`switch (var5_3.i)`（seed a.java:1384）= run() 的 overlay 分派；同一 seed 内
  **a.java:3037 `this.i = by;`**（`a(byte by, String string, byte by2, byte by3)` =
  showOverlayMessage 开头）——赋值点本就在 seed 文本中，"全文 grep 无赋值点"系漏检。
- 签名歧义：raw seed 单字母字段同名不同签名并存——台账 N-0223 `i:B`→overlayKind、
  N-0189 `i:I`→menuBoxWidth；state-machine.md 误把该 switch 的 `i` 认成 **int** 版
  （"整个代码库中唯一的 int 类型 i"），遂得"恒 0 死分支"结论。
- javap 终证（run()）：

```
    85: astore        5                  // var5_3 = this
    87: getfield      #1070  overlayActive:Z
    90: ifeq          669
    95: getfield      #1096  overlayKind:B
    98: tableswitch { // 0 to 5 … }
   136: getfield      #868   gameMode:B
   141: bipush 15 / 151: bipush 17 …     // case 0 体 = gameMode 15/17 特判
```

  与 seed:1384-1385 结构逐点吻合 ⇒ switch 字段 = `overlayKind:B`（#1096）；
  **写点存在**（dump 19767 `7: putfield #1096 overlayKind:B`，即 showOverlayMessage 的
  `this.i = by`）⇒ 非恒 0、非死分支。
- **判定**：② 伪像（raw 投影的同名异签歧义 + grep 漏检）；③ 的"疑似死代码"主张**证伪**。
  **台账建议**：BUG-002 移入"考证更正"（现象=假阳性；行号指针 seed:1384/3037 + javap
  dump 15455-15466/19767），不再作为原版 bug 计数。

### 3.3 BUG-005 难度倍率表不可达（③ 确认）

表 `difficultyMultipliers`（deobf a.java:2234 `{1,30,60,80,100}`）；下标 =
difficultyIndex。**值域已机械证明 ≡ 0**（dead-code-ruling DC-5 存档往返归纳引理 L1-L5：
putfield 3 处 = iconst_0×2 + readByte 自拷贝×1 ⇒ {0}，`python3 tools/dead_code_ruling.py
--check` 长期复核）⇒ 槽 1..4（30/60/80/100）恒不被读。①/② 排除（值域证明走同一
工具链，与 shim 无关）；③ 确认 = 原版死内容（机制口径"难度读取点前置赋值、正常流程
mult=1"不变）。

### 3.4 BUG-010 logostart/logoquit 死 BGM（③ 确认）

`playBgm`（deobf a.java:10167）调用点全类 6 处，实参前驱常量**仅 (byte)2 / (byte)3**：
a.java:4161/4163（dispatchModeEntry 内）、4311/4321、4668/4670。javap 对应 iconst_2/
iconst_3；无任何 0/1 实参 ⇒ 轨道表 `f_String_arr_15[0]/[1]`（logostart.mid/logoquit.mid）
无播放入口。①/② 排除（实参常量是字节码事实）；③ 确认 = 原版死资源（原意推测 C 级不变）。

### 3.5 BUG-003 GUTS 负参 DES（③ 确认）

语料事实：script63（deobf a.java:1188）`DES_-66_0 DES_-56_0 DES_-49_0 DES_-47_0
DES_-69_0`。处理器事实（deobf a.java:8314-8327）：DES 解析值 <0 ⇒ 进"移除全表该型"臂
（`for var53 … entityType == var4 → markEntityRemoved`），≥0 ⇒ 定点移除。负参是
**原版 DSL 的"全型清除"惯用语**（同 script 63 的混沌 boss 清场语义自洽）——设计而非
投影损坏。①/② 排除；③ 确认 = 语义待考项可改判"已考证语义"（台账 BUG-003 转正）。

---

## 4. 复现命令（可机械复跑）

```bash
J='javap -p -c -s analysis/rename-pipeline/renamed/a.class'

# BUG-004：两处 battleEnemyHp putfield 的源表字段号（#1152 vs #1185）
$J | grep -n 'battleEnemyHp\|enemyBaseHp\|enemyHpScaled'

# BUG-007：环 2 kind:=下标 / 环 4 tableswitch 臂常量
$J | sed -n '/private void addMenuItem/,/^$/p' | head -20
$J | sed -n '/1649: tableswitch/,/default/p'   # 3→15 / 4→17 / 5→22

# BUG-002：overlayKind 写点 + run() 分派
$J | grep -n 'overlayKind'        # putfield #1096（dump 19767）存在

# BUG-001：资源调用点 4 处无 "script"
grep -n 'getResourceAsStream' reference/src/deobf/a.java

# BUG-010：playBgm 实参恒 2/3
grep -n 'playBgm((byte)' reference/src/deobf/a.java

# BUG-005：difficultyIndex 值域 {0}（DC-5 引理长期复核）
python3 tools/dead_code_ruling.py --check
```

## 5. 遗留（C 级，不入结论）

1. BUG-004「原意」不可考：两表混用是设计意图（如"玩家主动战给玩家让利"）还是笔误——
   真机行为对照才可判（Stage B 前置，同 BUG-007 的真机考证口径）。
2. BUG-007 的 mode 14/8/16 语义（新游戏/继续/载入三臂的目标页）本审未逐一打开——
   仅错位三槽（3/4/5）在判。
3. BUG-002 证伪后，state-machine.md 的"输入锁定/特殊输入模式"整节主张需随台账更正复核
   （本文件只裁"恒 0 死分支"一句为伪像；该文档其余断言未审）。
