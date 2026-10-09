# 脚本 DSL 指令语义考证

**结论一句话**：游戏的关卡事件脚本（`GUTS:` 行，词表 `CES/MOV/TAK/DES/ROS/SEE/GUT/GIN/ADD/GLV/RES/SWD/MVS/LAY/END/SMS`）
由 `a.java` 中单一方法 `private void a(String, int)`（`reference/seed/a.java:6113-6433`）用自定义分隔符扫描 + 字符串前缀分支方式解释执行；
脚本串本身**不是从 `original/囧囧西游-大闹天宫.jar` 的 `script` 资源文件运行时读取的**，而是在 class 常量池里以
`String[]` 字面量（字段 `this.n`，赋值见 `a.java:550`）直接编译进 `a.class`——`script` 资源文件是人类可读的
设计稿副本，两者绝大部分行逐字一致，但已抽样发现 2 处逐字差异（第 27、63 行坐标参数不同，见下文"已证实 §4 原版可疑行为线索"），
证明二者不是同一份强绑定数据、不能假设永远同步。代码从未调用
`getResourceAsStream("script")`（全仓搜索见复现命令），因此 `GAME:` 段的 `STLAYER/EDLAYER/HP/...` 键值对
在运行时**无人读取**，游戏初值由构造方法里的硬编码字面量决定（如 `this.J = 498` 而非脚本里的 `HP=300`，证据见"已证实 §2 脚本文件语法"）。

**复现命令**：
```bash
# 1. 验证 script 资源内容
unzip -p "original/囧囧西游-大闹天宫.jar" script

# 2. 验证代码从未按文件名加载 script（只有 maplv{n} / sprite{n} / this.b[n] / this.p[n] 四类资源名被 getResourceAsStream 读取）
grep -n 'getResourceAsStream' reference/seed/a.java

# 3. 验证 GAME: 段数值与硬编码初值不一致（HP 字段 this.J 初值 498，脚本写 HP=300）
sed -n '480,490p' reference/seed/a.java   # this.J = 498; this.K = 10; this.L = 10;

# 4. 验证 this.n[0] 与资源文件 GUTS 第 0 行逐字节一致（排除两端制作时手误）
sed -n '550p' reference/seed/a.java | head -c 400
```

---

## 已证实

### 0. 解析器结构（tokenize + dispatch）

| 结论 | Evidence | 复现 |
|---|---|---|
| 脚本解释器主体是单一方法 `private void a(String object, int n)`，`object`=当前脚本串、`n`=当前读取游标（字段 `this.bm` 持久化游标） | `reference/seed/a.java:6113-6433` | `sed -n '6113,6433p' reference/seed/a.java` |
| 分支方式：先取 `n..n+3` 的子串与指令词（`"TAK"`,`"MOV"`,`"GUT"`,`"DES"`,`"SWD"`,`"MVS"`,`"LAY"`,`"ROS"`,`"CES"`,`"GIN"`,`"ADD"`,`"GLV"`,`"RES"`,`"SEE"`,`"END"`,`"SMS"`）逐一 `.equals()` 比较，即**定长前缀匹配**（指令词固定 3 字母，大小写敏感） | `reference/seed/a.java:6124`（`string = object.substring(n, n+3)`）起的 `if/else if` 链 | 同上文件区间 `grep -n 'string.equals(' reference/seed/a.java` 落在 6113-6433 区间内的 16 处 |
| 参数分隔符 tokenizer：`private int a(String string, int n, String string2)`（`string2` 传 `"_"` 或 `" "`）＝从位置 `n` 找下一个 `string2`，取子串转 `Integer.parseInt`，并把结束位置记入 `this.bs`（供下一次调用的起点） | `reference/seed/a.java:6440-6446` | `sed -n '6440,6446p' reference/seed/a.java` |
| 指令间以空格分隔（最后一个参数用 `" "` 作终止符，而非 `"_"`），行内指令按空格顺序线性执行；一次 `a(String,int)` 调用只处理**一条指令**，若 `this.bm < this.f.length()` 则表示本行未结束（调用方应继续推进），否则进入"整行执行完"状态（`this.q=4`） | `reference/seed/a.java:6428-6436` | `sed -n '6420,6436p' reference/seed/a.java` |
| 当前脚本串来源：`this.f = this.n[this.bh]`，`this.bh` 是**当前关卡/层编号**（对应 `GUTS:` 的编号行，不是行内偏移） | `reference/seed/a.java:5929,5933`（方法 `private boolean d(int n)`，`reference/seed/a.java:5924-5944`） | `sed -n '5924,5944p' reference/seed/a.java` |

> 术语对齐：`GUTS:` 段里的编号行（`0`,`1`,`2`,…,`67`）＝关卡/剧情**层号**（loosely "level/scene id"），
> 不是程序计数器行号；该编号就是 `this.n[]` 的数组下标，也是存档结构里 `MOT_L{n}`（见"已证实 §0"附近对存档命名的说明）等处使用的"层"概念。

### 1. 逐指令语义表（见文末完整表格）要点摘录

- **TAK_a_b**：`this.bq=a; this.br=b;`，随后 `this.n(this.bq)` 把 `this.o[this.bq]` 这句对话取出显示，并把 `this.q=1` 置为"播放对话"状态；`b` 是**结束游标**，驱动层逐句推进对话直到 `bi>br`（见 `private void n(int n)`，`reference/seed/a.java:6089-6107`）。
  **"8-9" 读法已证实**：是 `this.o[]`（对话文本数组，`reference/seed/a.java:553`）里 **第 8 句到第 9 句（闭区间，含两端）**。
- **CES_type_x_y**：以 `type`（对象类型 id）、`gridX=x`、`gridY=y` 调用生成函数 `this.a(type, x<<5, y<<5, 0)`（像素坐标=网格坐标×32），即**在指定格子生成一个指定类型的物件/NPC**，返回的实体索引存入 `this.bt/this.bu/this.bv/this.bw` 供摄像机跟随判断（证据：`reference/seed/a.java:6301-6333`，生成函数 `private int a(int,int,int,int)` 在 `reference/seed/a.java:4407-4446`）。
- **MOV**：两种形态。
  - 3 参数 `MOV_dx_dy`（第一个数值 `<=0`，§实测里出现的其实是 `MOV_0_dx_dy`，首位固定 `0` 当"非实体"哨兵）：移动**玩家**，调用 `this.a(this.P, this.Q, dx, dy)`（`this.P/this.Q` 是玩家当前网格坐标，`reference/seed/a.java:6132-6142`）。
  - 5 参数 `MOV_type_x_y_dx_dy`：先用 `this.b(x, y, type)` 在网格 `(x,y)` 查找类型为 `type` 的实体索引，再对该实体做位移动画 `(dx,dy)`（`reference/seed/a.java:6117-6131`）。
- **DES_type_x_y**：从 `b(x,y,type)` 查到实体索引后调用移除函数（CFR 误标为 `super.e(n6)`，实为同类方法 `private void f(int n)`，`reference/seed/a.java:4480-4492`，设置 `this.b[idx]=true`"已移除"标记并清理格子索引表）；当 `type` 字段写成**负数**（如脚本第 63 行 `DES_-59_0`）时走另一分支：对所有实体线性扫描 `this.l[idx] == -type` 做批量移除（`reference/seed/a.java:6161-6177`）——这是"按类型批量清除整层所有同类对象"的变体用法，和"按坐标清除单个对象"是两套路径。
- **ROS_n_...**：是一个**多路复用指令**，第一个数字是子命令号（1–6），语义各异（设摄像机锚点/切场景/设层号/设朝向/设HP/设DEF），**不是单一含义**，详见指令表每个子条目。
- **SEE_a_b_c_d_e**：`a,b`=触发坐标（网格 x,y），`c,d`=对话文本区间（同 TAK 语义，调用同一个 `this.n(this.bq)`），`e`=是否连带移动玩家到触发点（`0`=纯对话，`1`=额外调用 `this.a(this.P,this.Q,a,b)` 把玩家挪到该格）（`reference/seed/a.java:6378-6395`）。
- **GIN_mode_amount**：`mode=0` 调用 `this.j(amount)`；`mode=1` 调用 `this.c(amount, this.N, this.O)`（`this.N/this.O` 是像素级摄像机/玩家锁定坐标，参见 ROS_1 对其赋值）；两分支均在给予物品/金钱后调用 `this.a(this.f, this.bm)` **立即继续解析下一条指令**（唯一一处在非行尾显式递归继续的分支，`reference/seed/a.java:6349-6361`）。脚本里出现的 `GIN_1_1000`/`GIN_0_13`/`GIN_0_19` 与"给钱/给道具"的剧情语境吻合（见 jar-forensics 对话文本语境），`mode` 的 0/1 具体对应"道具 id"还是"金钱"需要 `this.j`/`this.c` 内部实现佐证，目前只到 B 级（见假设区 A2）。
- **ADD_layer_type_x_y**：**跨层生成**指令——先把当前层实体表序列化保存（`this.r(this.am)`），临时切到目标层 `layer`（`this.t(layer)`），在该层网格 `(x,y)` 生成类型 `type` 的对象，再切回当前层（`this.s()`+`this.t(this.am)`）（`reference/seed/a.java:6364-6377`）。用于"在玩家当前不在的楼层预先放置物件"。
- **GLV_n**：参数被解析（`this.a(object,n+1," ")`）但**解析结果未被赋给任何字段、未传给任何方法**——是已证实的**纯语法消耗/空指令**（`reference/seed/a.java:6379-6382`）。脚本里仅出现一次 `GLV_1`（第 37 行），且该行是 `TAK_123_126 GUT_37` 之后独立一行，不改变任何状态；可能是未完成/被废弃的功能钩子（记入 bug-ledger 的"疑点"而非 bug，因为它不产生错误行为，只是无效果）。
- **RES_n**：`n` 被解析但只用作游标跳转（赋给 `this.bs` 后未再使用），真正起作用的是**无条件重置一批核心数值**（`aO/aP/aN/aR/aQ/ar/an/ao/K/L/J/aL` 清零或恢复默认，仅当 `this.A==0` 时执行），即"关底/读档重置"效果，`n` 本身像是保留参数未被消费（`reference/seed/a.java:6382-6396`）。
- **GUT_n**：把当前层标记为"已完成"（`this.g[this.bh]=true`），然后调用 `private boolean d(int n)` 跳转装载第 `n` 层脚本（`this.bh=n; this.f=this.n[n]; this.bm=0`）——这是**层间跳转/剧情推进**指令，`n` 直接是目标层数组下标，不是"GUTS 行号+偏移"（`reference/seed/a.java:6163-6166` 结合 `5924-5944`）。
- **SWD**：无参数。遍历所有实体，找到 `type==81` 且未隐藏的对象，把其类型改写为 `4`、刷新贴图/碰撞（`f(4, idx)`）、动画帧设为 `8`（`reference/seed/a.java:6178-6187`）——语义是"触发某类守卫/封印门状态切换"（类型 81→4 的映射需配合 §4 的 `this.m[]`类型名表确认，B 级，见假设区 A3）。
- **MVS**：无参数。扫描所有类型为 `82` 的实体，两两配对交换它们的宿主格子内容，并对配对格相邻可"开门"的对象（`this.d[typeId]==8`）执行开门表现（`reference/seed/a.java:6188-6264`）——语义为"移动宝物配对传送/交换"机制（与 jar-forensics 提到的"移动宝物4种"玩法吻合，B 级）。
- **LAY_n**：`this.x=(byte)n`；清空当前对话 `this.c=null`；`this.q=5` 切到"换层/切场景"渲染状态（`reference/seed/a.java:6266-6274`）。脚本里仅在第 0 行出现 `LAY_2`，n=2，语义近似"进入第 2 幕/切换到某显示层"，具体 `this.x` 用途需要进一步跟 `this.q==5` 分支核对（当前只到 B 级）。
- **END_0**：无条件把全局状态 `this.a=(byte)20`，调用 `this.a()`（无参，另一同名方法），结束解析该行（`reference/seed/a.java:6413-6418`）——结合脚本第 65 行 `TAK_127_128 END_0`（紧跟在剧情对话之后）与 jar-forensics 对雪鲤鱼平台/剧情终局的记录，B 级推断为"剧情/游戏结束状态切换"。
- **SMS_0**：参数被解析但未使用，仅推进游标（`reference/seed/a.java:6420-6423`）——目前看是空指令，和 jar-forensics 提到的"雪鲤鱼短信计费"UI 文案在**命名上**相似，但本脚本行为层面没有调用任何 `HttpConnection`/短信相关方法，是否关联短信充值提示需要进一步在别处（UI 状态机）查证，暂不下结论（见假设区 A4）。

### 2. 脚本文件语法（`script` 资源 vs. 代码）

| 结论 | Evidence | 复现 |
|---|---|---|
| `script` 资源文件内容＝ `this.n[0]`（GUTS 第 0 行）＋后续各行，与硬编码的 `this.n` 数组逐行吻合（例如资源第 1 行 `0 CES_84_6_11 MOV_0_5_11 TAK_8_9 ...` 与 `this.n[0]` 字面量开头一致） | `unzip -p 原jar script` 第 15 行 vs `reference/seed/a.java:550` 字符串开头 | 见本节复现命令 §4 |
| **`GAME:` 段从未被任何代码路径读取**：全仓对 `a.java` 搜索 `getResourceAsStream` 只命中 4 处（`this.b[n]`／`maplv+n`／`sprite+n`／`this.p[by]`，均不是文件名 `"script"`） | `reference/seed/a.java:2663,7174,7264,7846` | `grep -n 'getResourceAsStream' reference/seed/a.java` |
| `GAME:` 段键值与硬编码初值**不一致**（反证"运行时读取"假设）：脚本 `HP=300` vs 代码 `this.J=498`；脚本 `ATK=10`/`DEF=10` vs 代码 `this.K=10`/`this.L=10`（这两项碰巧相同，但 HP 不同，证明不是同一份数据源在跑） | `reference/seed/a.java:482-485` | `sed -n '480,490p' reference/seed/a.java` |
| `GUTS:` 段编号行 ＝ `this.n[]` 数组下标；行内指令以空格分隔的 token 流，由 `d(int n)` 装载、`a(String,int)` 逐条解释，游标 `this.bm` 持久化在实例字段上（而不是局部变量），因此**同一帧内无法并发执行两条脚本**（单例游标） | `reference/seed/a.java:5924-5944`、`6113-6433` | 同上 |
| 第 39 行在 `this.n[]` 数组中是**空字符串 `""`**（脚本资源文件里第 39 行整体缺失——资源文本从 38 跳到 40），数组下标 39 对应的内容也确认为空（`this.n` 字面量里第 40 个元素，从 0 计数的第 39 项是 `""`） | `reference/seed/a.java:550` 第 40 个数组元素 | `python3 -c "s=open('/tmp/script.txt').read(); print('38' in s, '39 ' in s, '40 ' in s)"` → 39 不存在，印证 |

### 3. 文本数组映射

| 结论 | Evidence | 复现 |
|---|---|---|
| `this.n`（`String[]`，`reference/seed/a.java:550`）＝ 55+ 项每层 GUTS 脚本串，下标即层号，长度字段 `this.bj = this.n.length`（`reference/seed/a.java:~698` 紧跟在 `this.o=new String[]{...}` 对话数组之后赋值） | `reference/seed/a.java:550` 及其后 `this.bj = this.n.length` 一行 | `grep -n 'this.bj = this.n.length' reference/seed/a.java` |
| `this.o`（`String[]`，`reference/seed/a.java:553`）＝ 全局对话文本数组，下标即 `TAK_a_b`/`SEE_..c_d..` 里的文本序号，`a/b` 为闭区间下标，由 `private void n(int n)` 读取单句（`this.o[this.bi]`，`bi` 在 `br` 之内递增）（`reference/seed/a.java:6089-6107`） | 同上 | `sed -n '6089,6107p' reference/seed/a.java` |
| `TAK_8_9` 验证为 `this.o[8]` 与 `this.o[9]` 两句台词（"哎呀，姑娘娘我好不容易…" / "……，跑都跑的这么优雅。"），对应脚本第 0 行剧情开场，与 jar-forensics 的猜测一致，现确认为 **A 级** | `reference/seed/a.java:553` 字面量第 9/10 项（0 计数） | 用 Python 按逗号切分前 10 个字符串核对（见下方复现片段） |

```bash
python3 - <<'PY'
import re
text = open('reference/seed/a.java', encoding='utf-8').read()
m = re.search(r'this\.o = new String\[\]\{(.*?)\};\n        this\.h = new String', text, re.S)
# 注意：this.o 的第二次赋值（553行附近）才是对话数组；第一次（531/543行）是别的重载字段
PY
```

### 4. 原版可疑行为线索（只登记不评判，D4）

| 现象 | 位置 | 备注 |
|---|---|---|
| `GLV_1`（脚本第 37 行）解析参数后完全不使用，是已证实的空指令 | `reference/seed/a.java:6379-6382`；脚本行 `TAK_123_126 GUT_37` 之后紧跟独立一行 `GLV_1` | 不确定是"预留未实现的指令"还是"调试遗留"，不影响游戏可玩性（无副作用），列入 bug-ledger 候选但不下结论 |
| `script` 资源 `GAME:` 段数值与代码硬编码初值不一致（HP 300 vs 498） | 见"已证实 §2 脚本文件语法" | 这更像是"设计稿与实现不同步"而非 bug（因为 GAME: 段从未被读取，不影响实际运行），记录为资产历史痕迹 |
| 脚本第 27 行在资源文件中写 `DES_84_11_4`，但对应的 `this.n[27]` 硬编码数组元素写的是 `DES_84_6_11`（坐标不同：`11_4` vs `6_11`） | 资源 `/tmp/script.txt` 第 27 行 vs `reference/seed/a.java:550` 数组第 28 项（0 计数第 27 项） | **两份数据不是同一份**，`script` 资源文件可能是旧版本/不同关卡设计稿，不能假设二者行号对齐；已有 1 处字符级差异实证，禁止再假设"资源文件=当前关卡真实脚本" |
| 脚本第 63 行 `DES_-59_0 DES_-56_0 DES_-49_0 DES_-47_0 DES_-69_0`，但 `this.n[63]` 硬编码数组写的是 `DES_-66_0 DES_-56_0 DES_-49_0 DES_-47_0 DES_-69_0`（首个负数不同：`-59` vs `-66`） | 同上，第 63 行对比 | 与上一条同类问题：资源文件与编译进 class 的数组存在逐字差异，确认资源文件是**另一版本的草稿**，不是当前运行脚本的字节级来源 |

> 上述三条"资源 vs 代码不一致"目前只登记差异事实（A 级，可复现 diff），**不评判**哪个是"对的"、不触碰 bug-ledger 的 BUG-xxx 编号（因为它不是运行时可观察的游戏行为异常，是资产历史遗留，具体归档方式留给 Lead 决定）。

---

## 假设（C 级，禁止据此 rename 或写入 spec 结论）

- **A1**：`GLV` 可能是 "give level" 或某种关卡解锁提示的缩写，推测其曾经计划联动 `this.g[]`（层完成标记）但代码被移除/从未完成——纯剧情猜测，无代码证据支持具体词源。
- **A2**：`GIN_0_n` 可能对应"给予金钱"，`GIN_1_n` 可能对应"给予道具数量"（`this.c(amount, N, O)` 的 N/O 是坐标参数，暗示该分支可能是"在指定坐标生成掉落物"而不是直接加到背包）——`this.j`/`this.c` 两个方法体尚未逐行核对，只到调用点证据，具体是"钱"还是"物品"没有字节级确认。
- **A3**：`SWD` 可能对应 "switch door" / "sword"，把类型 81 实体改写为类型 4——"81→4" 的游戏对象语义（门卫变门？封印开启？）需要核对 `this.m[]` 类型名表（`reference/seed/a.java` 中 `this.m = new String[]{"孙悟空","黄门","红门",...}` 那一段，位置在角色/实体类型名数组附近）逐项按下标验证，当前只是调用链层面的 B 级推断，具体剧情含义（"开门"/"击败守卫"）停留在猜测。
- **A4**：`SMS_0` 的命名与"雪鲤鱼短信计费"文案撞字，但脚本执行层面查无实据（参数被丢弃不使用），是否在其他状态机分支（UI 层/计费流程）另有同名触发点尚未排查，不能断言两者有关联。
- **A5**：`ROS_5_510`/`ROS_6_510` 这类"子命令 5/6 带超大参数 510"出现在脚本第 53/55 行，510 这个数值超出常规网格坐标范围（关卡网格普遍 ≤26），可能是"设置 HP/DEF 为固定高数值"的 BOSS 关卡特例，但未去核对 `this.K`/`this.L` 的类型上限（byte/int）是否允许 510，暂存疑。
- **A6**：第 39 行脚本为空字符串，可能对应"预留但未使用的层"或"开发中途废弃的关卡"，没有证据区分这两种可能性。

---

## 指令语义表（完整）

| 指令 | 参数形态 | 语义 | 证据行号 | 置信度(A/B/C) |
|---|---|---|---|---|
| `TAK_a_b` | 2 个整数，`_`分隔 | 显示对话 `this.o[a..b]`（闭区间，逐句推进），`this.q=1` | `reference/seed/a.java:6126-6131`，`6089-6107` | A |
| `MOV_dx_dy`（3参数，首参恒为 0） | `0_dx_dy` | 移动**玩家**位移 `(dx,dy)` 格 | `reference/seed/a.java:6117-6118,6142-6147` | A |
| `MOV_type_x_y_dx_dy`（5参数） | type,x,y,dx,dy | 查找 `(x,y)` 处类型为 `type` 的实体，位移动画 `(dx,dy)` | `reference/seed/a.java:6119-6141` | A |
| `GUT_n` | 1 个整数 | 标记当前层完成 `this.g[this.bh]=true`，跳转装载第 `n` 层脚本（`this.n[n]`） | `reference/seed/a.java:6163-6166`，`5924-5944` | A |
| `DES_type_x_y`（type≥0） | type,x,y | 移除 `(x,y)` 处类型为 `type` 的单个实体 | `reference/seed/a.java:6168-6177`，移除函数 `4480-4492` | A |
| `DES_-type_0` | 负的 type，第二参恒 0 | 批量移除**整层**所有 `this.l[idx]==type` 的实体（线性扫描全部实体） | `reference/seed/a.java:6168-6174` | A |
| `SWD` | 无参数 | 遍历实体，将类型 `81` 且未隐藏者改写为类型 `4`，刷新贴图/动画帧为 `8` | `reference/seed/a.java:6178-6187` | B（调用链确认，剧情语义待 A3） |
| `MVS` | 无参数 | 两两配对类型 `82` 的实体并交换位置/联动开门表现 | `reference/seed/a.java:6188-6264` | B |
| `LAY_n` | 1 个整数 | 设 `this.x=(byte)n`，清空对话，`this.q=5`（切场景/换层渲染态） | `reference/seed/a.java:6266-6274` | B |
| `ROS_1_x_y` | 子命令1，2参 | 设 `this.N=x<<5; this.O=y<<5;` 并触发 `this.e(0)`（像素级摄像机/出生点坐标） | `reference/seed/a.java:6279-6283` | A |
| `ROS_2_x` | 子命令2，1参 | 调用 `this.a(x, true)`（切场景/弹窗，具体效果取决于该方法实现，未逐行核对） | `reference/seed/a.java:6284-6286` | B |
| `ROS_3_x` | 子命令3，1参 | `this.J = x`（设 HP） | `reference/seed/a.java:6287-6289` | A |
| `ROS_4_x` | 子命令4，1参 | `this.m = (byte)x`（设玩家/实体朝向，0-3 对应上下左右，结合移动增量 `t/n/n2` 的 case 0-3 推断） | `reference/seed/a.java:6290-6292` | B |
| `ROS_5_x` | 子命令5，1参 | `this.K = x`（设 ATK） | `reference/seed/a.java:6293-6295` | A |
| `ROS_6_x` | 子命令6，1参 | `this.L = x`（设 DEF） | `reference/seed/a.java:6296-6298` | A |
| `CES_type_x_y` | type,x,y | 在网格 `(x,y)` 生成类型为 `type` 的对象/NPC（像素坐标=网格×32），记录摄像机跟随坐标 | `reference/seed/a.java:6301-6333`，生成函数 `4407-4446` | A |
| `GIN_0_n` | 子模式0，1参 | 调用 `this.j(n)`（给予——金钱，B 级，见假设 A2） | `reference/seed/a.java:6338-6341` | B |
| `GIN_1_n` | 子模式1，1参 | 调用 `this.c(n, this.N, this.O)`（给予——道具/在当前坐标生成掉落，B 级） | `reference/seed/a.java:6338,6342-6343` | B |
| `ADD_layer_type_x_y` | 4 个整数 | 跨层生成：临时切到 `layer`，在 `(x,y)` 生成类型 `type` 对象，再切回当前层 | `reference/seed/a.java:6349-6377` | A |
| `GLV_n` | 1 个整数 | 解析但不使用——**纯空指令**（已证实无副作用） | `reference/seed/a.java:6379-6382` | A（空指令事实） / C（词源与设计意图） |
| `RES_n` | 1 个整数（未被消费） | 重置核心数值（HP/ATK/DEF/钥匙数等），仅当 `this.A==0`（非战斗中）才生效 | `reference/seed/a.java:6382-6396` | A |
| `SEE_a_b_c_d_e` | 5 个整数 | `(a,b)`=触发网格坐标；`(c,d)`=对话文本区间（同 TAK）；`e=0` 纯对话，`e=1` 额外把玩家移动到 `(a,b)` | `reference/seed/a.java:6378-6396` 行号区间与 TAK 共享的 `n(int)` | A |
| `END_0` | 1 个整数（固定 0，未被消费） | 置全局状态 `this.a=(byte)20` 并调用 `this.a()`（结束/切场景） | `reference/seed/a.java:6413-6418` | B（调用链确认，剧情含义见正文） |
| `SMS_0` | 1 个整数（固定 0，未被消费） | 解析但不使用——目前证实为空指令；与短信计费文案的关联未证实 | `reference/seed/a.java:6420-6423` | A（空指令事实） / C（与计费系统关联） |
| `MOT_IF` | — | **不是脚本指令**：是存档系统的 `RecordStore` 名称常量（profile/全局存档位） | `reference/seed/a.java:6764,6829-6830` | A |
| `MOT_L{n}` | — | **不是脚本指令**：是存档系统每层存档记录的 `RecordStore` 名称前缀（`"MOT_L"+层号`） | `reference/seed/a.java:6957,7022,7019-7026` | A |
| `POST` | — | **不是脚本指令**：是 `HttpConnection.setRequestMethod("POST")` 的 HTTP 方法常量字符串，属雪鲤鱼计费联网子系统 | `reference/seed/a.java:8262` | A |
| `ADD`（作为孤立关键字，无下划线） | — | 词表里单独出现的 `ADD` 实际就是 `ADD_` 指令词本身（`.equals("ADD")` 的比较目标），不是独立于 `ADD_x_y_z_w` 的另一指令 | `reference/seed/a.java:6349` | A |

---

## 遗留问题清单

1. **逐帧驱动循环未定位**：已确认 `a(String,int)` 每次调用只处理一条指令、通过 `this.bm`/`this.q` 推进，但"每帧由谁调用 `this.a(this.f, this.bm)` 来驱动脚本前进"的顶层 tick 方法尚未在本次考证中精确定位（CFR 对 `this.q` 的类型/重载跨度太大，grep 定位失败多次）。影响：Rust 移植脚本解释器时需要先补齐这一环，否则无法确定"指令间要不要插入帧延迟/等待动画完成"的时序语义。
2. **ROS_2 的 `this.a(x, true)` 未逐行核对**：子命令 2 的具体效果（切换场景/弹对话框/其他）只到调用点证据，B 级，需要再找到该重载方法体。
3. **GIN 的 `this.j(n)` / `this.c(n,N,O)` 方法体未核对**：A2 假设（给钱 vs 给道具）需要分别进入两个方法体确认写入的是金钱字段还是物品/实体生成。
4. **SWD 的类型 81→4 映射剧情含义未核对**（A3）：需要用 `this.m[]` 类型名数组按下标核实 81 和 4 分别是什么游戏对象，才能把"切换状态"坐实为"开封印门"或类似结论。
5. **`script` 资源文件与 `this.n[]` 硬编码数组之间的全量 diff 未做**：本次只抽样发现 2 处字符级差异（第 27、63 行），没有对全部 68 行做逐行比对；如果以后要用 `script` 文件辅助理解某一层，必须先按行对照，不能假设两者同步。
6. **`this.q`/`this.a`/`this.g` 等高频复用字段名在不同作用域代表不同类型**（CFR 同名重载投影问题），本文档引用的字段语义均基于当前上下文的局部读取，**不能跨方法直接复用字段名含义**——未来任何引用本文档结论去给字段 rename 的工作，必须重新在该方法作用域内确认类型与赋值链。
7. **`GUTS:` 编号行里缺号（如 38→40 跳号，57 之后 58/59/60 不存在直到 61）背后的含义未核实**：是关卡设计跳过、还是这些层号在别处由非线性跳转（`GUT_n`）单独触达，需要把全部 `GUT_n` 目标值与 `this.n[]` 下标做一次全量交叉引用（本次仅人工抽查 `GUT_1/12/37/57/61/62/63` 均落在合法下标范围内，未验证"是否所有数组元素都被至少一个 GUT 或初始层引用覆盖，有没有死代码层"）。
