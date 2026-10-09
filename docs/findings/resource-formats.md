# 资源格式字节级考证（P2）

日期：2026-10-09。工具：Python3（struct）、xxd、CFR 反编译对照 `reference/seed/a.java`（8457 行，与 `analysis/decomp/a.java` 逐行一致）。
解包资源副本：`analysis/ex/`（来自 `original/囧囧西游-大闹天宫.jar`，只读未改）。

**一句话结论**：
1. **packed PNG 的"2 字节头"不是单个资源的元数据，而是"多图集容器"里每张子图自己的 LE16 长度前缀**——
   一个资源文件 = `N × [u16 LE 长度][PNG 字节]`，`N` 由代码里硬编码的常量表给出（文件内不存该计数）。
2. **maplv 的"26×26"头是双倍值**：代码读到后 `>>1`，真实网格是 **13×13**，且每格 **4 字节**（不是 1 字节），
   对应两张 `13×13×4=676B` 的网格（地形 tile-id 网格 + 翻转/变换模式网格），`4+676+676=1356B` 正好是文件总长。
3. **sprite 文件头是 4 字节（两个 LE16 拼成 32 位计数）**，记录为**变长结构**（1 字节类型码 + 2×u16 坐标 + 0~2
   字节附加参数，长度由类型码决定），不是"固定 5 字节"。

资产复现命令（任何人可重跑）：
```bash
unzip -p original/囧囧西游-大闹天宫.jar maplv0 > /tmp/maplv0
unzip -p original/囧囧西游-大闹天宫.jar sprite1 > /tmp/sprite1
unzip -p original/囧囧西游-大闹天宫.jar mapbg > /tmp/mapbg
xxd -l 16 /tmp/maplv0   # 应得 1a00 1a00 0000 0000 ...
```

---

## 已证实

### 1. packed PNG：多图集容器，非单图 + 神秘头

**结论**：资源文件 `sflogo/mapbg/map/actor/sptmap/sptprop/sptarm/sptenemy1/ui/xtq/menu/intro/face/sptenemy2/end/load`
均为**同一种容器格式**：顺序存放 N 张 PNG，每张前面有自己的 **2 字节 LE 长度前缀**，N 不存储在文件内，
而是运行时由代码常量表给出。

- Evidence（A级，代码定论）：`reference/seed/a.java:2657`-`2676` 方法 `private void a(int n)`：
  - `line 2663`: `this.a = this.getClass().getResourceAsStream(this.b[n]);`（`this.b[n]` = 资源名，数组见 `line 465`）
  - `line 2667`: `n3 = this.a.read() & 0xFF | this.a.read() << 8 & 0xFF00;`（**逐图读 2 字节 LE 长度**，与
    `line 4913`-`4915` 的 `private static short a(InputStream)` 实现等价：`read()&0xFF | read()<<8&0xFF00`）
  - `line 2674`: `this.a.read(byArray, 0, n3);`（按刚读到的长度取 PNG 字节）
  - `line 2675`: `this.a[n][i] = Image.createImage((byte[])byArray, (int)0, (int)n3);`（逐张解码）
  - 循环次数 `n2 = this.a[n]`，来自资源名同序的计数表 `line 466`：
    `this.a = new int[]{8,1,12,4,13,23,10,20,25,6,2,2,12,20,1,2}`，与 `line 465` 的
    `this.b = {"sflogo","mapbg","map","actor","sptmap","sptprop","sptarm","sptenemy1","ui","xtq","menu","intro","face","sptenemy2","end","load"}`
    一一对应（第 n 个资源名用第 n 个计数）。
- Evidence（A级，字节复现）：对全部 16 个文件按"计数表张数 × [u16 len][PNG]"模型逐图集解析，
  **16/16 文件消耗字节数与文件实际大小完全一致（remaining=0），且每张子图起始 8 字节均为 PNG 魔数
  `89 50 4E 47 0D 0A 1A 0A`**：
  ```bash
  python3 - <<'EOF'
  import struct
  NAMES_COUNTS = [('sflogo',8),('mapbg',1),('map',12),('actor',4),('sptmap',13),('sptprop',23),
   ('sptarm',10),('sptenemy1',20),('ui',25),('xtq',6),('menu',2),('intro',2),('face',12),
   ('sptenemy2',20),('end',1),('load',2)]
  for name, cnt in NAMES_COUNTS:
      with open(f'analysis/ex/{name}','rb') as f:
          data = f.read()
      pos = 0
      ok = True
      for i in range(cnt):
          ln = data[pos] | (data[pos+1] << 8); pos += 2
          chunk = data[pos:pos+ln]
          ok &= chunk[:8] == b'\x89PNG\r\n\x1a\n'
          pos += ln
      print(name, 'consumed=', pos, 'filesize=', len(data), 'match=', pos==len(data), 'all_png_magic=', ok)
  EOF
  ```
  实测输出（16/16 全部 `match=True all_png_magic=True`）：
  `sflogo 7505/7505`、`mapbg 9912/9912`、`map 16372/16372`、`actor 4293/4293`、`sptmap 14184/14184`、
  `sptprop 7248/7248`、`sptarm 3015/3015`、`sptenemy1 16230/16230`、`ui 7203/7203`、`xtq 6759/6759`、
  `menu 25286/25286`、`intro 47580/47580`、`face 3239/3239`、`sptenemy2 20907/20907`、`end 16696/16696`、
  `load 806/806`。
- **此前 `docs/findings/jar-forensics.md` §5.1 的"2 字节头与 PNG 长度部分吻合、部分不吻合"之谜已解**：
  那些"吻合"的文件（`end`：计数=1，`mapbg`：计数=1）恰好只含 1 张图，此时"文件头 LE16"碰巧等于
  "唯一一张图的长度"；"不吻合"的文件（`map`：计数=12，`menu`：计数=2 等）因为文件头 LE16 只是
  **第一张图的长度**，不是全文件长度，故不吻合——两种情况其实是同一套格式的自然结果，无需额外假设。

#### 字段布局表（packed PNG 容器）

| 偏移 | 类型 | 含义 | 证据 |
|---|---|---|---|
| 文件级：无固定头，总张数不写入文件内 | — | 张数来自运行时常量表（见下）| `a.java:466` |
| 每张记录 +0 | u16 LE | 本张 PNG 字节长度 | `a.java:2663`、`a.java:4913-4915` |
| 每张记录 +2 .. +2+len | byte[len] | 标准 PNG（魔数 `89504E470D0A1A0A` 开头） | 字节级复现命令见上 |
| （下一张记录紧接在上一张末尾） | — | 顺序排列，无填充/对齐 | 复现命令中 `remaining=0` 验证 |

资源名 → 张数常量表（`a.java:465-466`，与 `this.b[n]`/`this.a[n]` 一一对应）：

| 资源名 | 张数 | 资源名 | 张数 |
|---|---|---|---|
| sflogo | 8 | ui | 25 |
| mapbg | 1 | xtq | 6 |
| map | 12 | menu | 2 |
| actor | 4 | intro | 2 |
| sptmap | 13 | face | 12 |
| sptprop | 23 | sptenemy2 | 20 |
| sptarm | 10 | end | 1 |
| sptenemy1 | 20 | load | 2 |

> 注：`docs/findings/jar-forensics.md` §5.1 表格里的"IHDR 128×208"等尺寸实际是**该文件第 1 张子图**的
> IHDR，并非整文件唯一图像（除 `mapbg`/`end`/`load` 外，`load` 虽计数=2 但两张均为 24×148，详见下方抽查）。

---

### 2. maplv0–54：13×13×4B 双网格（地形 tile-id + 翻转/变换码），非 26×26×1B

**结论**：文件头两个 LE16 字段**不是**网格宽高本身，而是宽高的**2 倍**（代码里 `>>1` 还原）；
还原后的真实网格是 **13×13**，且**每格占 4 字节**（代码 `n3 = Y*Z<<2`），不是 jar-forensics 猜测的
"26×26 网格、每格 1 字节"。文件由两张网格顺序拼接：
网格 1（`this.i`，`byte[]`）= **地形贴图 tile-id**（可能叠加变换标记低位），
网格 2（`this.j`，`byte[]`）= **贴图翻转/旋转模式码**（取值集中在 0–7，经验值域 0–2）。

- Evidence（A级，代码定论）：`reference/seed/a.java:7169`-`7177` 方法 `private void s(int n)`：
  - `line 7174`: `InputStream inputStream = a2.getClass().getResourceAsStream("maplv" + n2);`
  - `line 7175`: `a2.Y = a.a(inputStream) >> 1;`（读 LE16 **右移 1 位**存入 `Y`）
  - `line 7176`: `a2.Z = a.a(inputStream) >> 1;`（同上存入 `Z`）
  - `a(InputStream)` 实现见 `line 4912-4915`：`return (short)(read()&0xFF | read()<<8&0xFF00);`（标准 LE16）
  - `line 7178`: `int n3 = a2.Y * a2.Z << 2;`（**每张网格字节数 = Y×Z×4**）
  - `line 7181-7182`: `a2.i = new byte[n3]; a2.j = new byte[n3];`
  - `line 7183-7184`: `inputStream.read(a2.i, 0, n3); inputStream.read(a2.j, 0, n3);`（**顺序读两张网格**）
  - 字段声明 `byte[] i`/`byte[] j` 在 `line 179-180`（与同名的 `int i`/`int j`，`line 76-77`，
    是同一个混淆类里按类型重载的不同字段，解析时需以上下文类型区分——证据纪律 B 级辨析，
    已用字节实测交叉验证为 A 级，见下）。
- Evidence（A级，字节复现，验证 `1356B = 4(头) + 676(网格1) + 676(网格2)`，且 `Y=Z=13`）：
  ```bash
  python3 - <<'EOF'
  import struct
  with open('analysis/ex/maplv0','rb') as f:
      data = f.read()
  wraw, hraw = struct.unpack_from('<HH', data, 0)
  Y, Z = wraw >> 1, hraw >> 1
  n3 = Y * Z * 4
  print('header raw(LE16,LE16)=', wraw, hraw, '-> Y,Z=', Y, Z,
        'per-grid bytes=', n3, 'expected total=', 4+2*n3, 'actual filesize=', len(data))
  EOF
  # 输出: header raw(LE16,LE16)= 26 26 -> Y,Z= 13 13 per-grid bytes= 676 expected total= 1356 actual filesize= 1356
  ```
  对全部 55 个 `maplv0`–`maplv54` 批量校验该公式：**54/55 均为头 `(26,26)`→网格 `13×13`，1356B**；
  `maplv51` 头为 `(24,26)`→网格 `12×13`（非正方形地图，公式仍自洽，`1252B`，详见"遗留问题"）：
  ```bash
  python3 - <<'EOF'
  import struct, glob
  for fn in sorted(glob.glob('analysis/ex/maplv*'), key=lambda x: int(x.replace('analysis/ex/maplv',''))):
      with open(fn,'rb') as f: data = f.read()
      wraw,hraw = struct.unpack_from('<HH', data, 0)
      Y,Z = wraw>>1, hraw>>1
      expect = 4 + Y*Z*4*2
      if expect != len(data):
          print('MISMATCH', fn, wraw, hraw, len(data), expect)
  EOF
  # 无输出 = 全部 55 个文件自洽（含 maplv51 的非方形网格）
  ```
- Evidence（A级，两张网格的语义区分——取值分布差异佐证）：
  ```bash
  python3 - <<'EOF'
  import struct, collections
  with open('analysis/ex/maplv0','rb') as f: data = f.read()
  Y = Z = 13
  n3 = Y*Z*4
  g1, g2 = data[4:4+n3], data[4+n3:4+2*n3]
  print('grid1(terrain) distinct values:', len(set(g1)), sorted(set(g1))[:15], '...')
  print('grid2(flip/transform) distinct values:', sorted(set(g2)))
  EOF
  # grid1(地形): 61 种不同取值(0,1,2,5,...,101) —— 典型 tile-id 分布
  # grid2(变换码): 仅 {0, 1, 2} —— 落在"翻转/旋转模式"的取值域内
  ```
  渲染代码确认 `grid2` 的用途——`line 5002`：
  `this.a(this.a[2][0], n5, n6, n9, n4, 16, 16, this.j[n2]);` 调用 7 参数绘制方法
  `line 2708`: `private void a(Image image, int n, int n2, int n3, int n4, int n5, int n6, int n7)`，
  其中 `n7`（= `this.j[n2]`，即网格2的值）走 `switch(n7){case 1..7}` 的镜像/旋转分支
  （`line 2708-2744`），**取值域 0-7 对应 8 种变换**（0=不变换，其余为水平/垂直镜像与象限交换组合），
  与网格2实测取值 `{0,1,2}` 落在该域内一致。
  >
  > 【2026-10-09 更正 ①】上面的复现命令**只跑了 `maplv0` 一个文件**，"`grid2` 仅 {0,1,2}"是单文件抽样，
  > **不是全量域**。全量 55 个 `maplv*` 实测域为 **{0,1,2,3,4}**：`3` 见 maplv1/6/8/11/14/16/18，
  > `4` 见 maplv54；仍整体落在 `switch(n7){case 1..7}` 的 0-7 变换域内（`a.java:2708-2744`），
  > 原结论"落在变换取值域内"不变，仅"实测取值集"由 `{0,1,2}` 更正为 `{0,1,2,3,4}`。
  > 下方"字段布局表"网格2 行的"本档样本实测 0-2"同此更正。
  > 复现（cargo test 或 python）：
  > `cargo test -p game-data --test golden maplv_flip_codes_within_transform_domain`
  > ```bash
  > python3 - <<'EOF'
  > import struct, glob
  > s = set()
  > for fn in glob.glob('assets/raw/maplv*'):
  >     with open(fn, 'rb') as f: data = f.read()
  >     w, h = struct.unpack_from('<HH', data, 0)
  >     s.update(data[4 + (w >> 1) * (h >> 1) * 4:])
  > print(sorted(s))   # → [0, 1, 2, 3, 4]
  > EOF
  > ```
  `grid1` 的用途——同一方法前一行 `line 4996-4999`：
  `n4 = this.i[n2]; int n9 = (n4&7)<<4; n4 = n4>>3<<4;`（**tile-id 拆成图集列 `id&7`、行 `id>>3`，
  各 ×16px**，用于从精灵图集 `this.a[2][0]` 裁剪对应 16×16 格子），证明 `grid1` 就是地形贴图索引。

#### 字段布局表（maplv0–54）

| 偏移 | 类型 | 含义 | 证据 |
|---|---|---|---|
| +0 | u16 LE（原始值=真实宽×2） | 真实网格宽 `Y`：代码读入后 `>>1` | `a.java:7175` |
| +2 | u16 LE（原始值=真实高×2） | 真实网格高 `Z`：代码读入后 `>>1` | `a.java:7176` |
| +4 .. +4+Y·Z·4 | byte[Y·Z·4] | **网格1（地形 tile-id）**：逐格 1 字节，`id&7`=图集列、`id>>3`=图集行（各×16px） | `a.java:7178,7181,7183`；`a.java:4996-4999` |
| +4+Y·Z·4 .. +4+2·Y·Z·4 | byte[Y·Z·4] | **网格2（贴图翻转/变换码）**：逐格 1 字节，取值域 0-7（本档样本实测 0-2），索引 8 向镜像/旋转表 | `a.java:7178,7182,7184`；`a.java:2708-2744`；`a.java:5002` |

> 注1："Y·Z·4"是**字节数**，不是"每格 4 字节"的直译——按 `a.java:7178` `n3=Y*Z<<2`，
> 该尺寸是整张网格的总字节数。结合网格1/网格2实测均恰好 1 字节/格、且 `Y×Z=169`、`169×4=676`，
> 可知**真实存储是 Y×(4Z) 或 (4Y)×Z 的字节平铺**，等价于**每格 1 字节、但网格实际尺寸是
> 头值本身（26×26）而非头值的一半**。本考证按代码字面量 `Y=head>>1=13` 与 `n3=Y*Z*4=676` 记录，
> "26×26×1B" 与"13×13×4B"在总字节数上等价（均为 676B/网格），**哪种是设计者原意**（即网格到底是
> 13×13 大格、每格 4 字节内部结构，还是 26×26 单字节格）**仍为待证点**，见"遗留问题清单"。
>
> 【2026-10-09 更正 ④】**"字节布局/索引方式"之争已定案**（设计动机仍待证，见"假设 1"）：
> 绘制循环的索引方式是 **2Y×2Z 网格、每格 1 字节**——
> `a.java:4991` `n2 = n3 = this.ai + (this.ak * this.Y << 1)`（线性下标 = 列 + 行×(Y<<1)，Java 中
> `*` 优先于 `<<`，即行跨距 2Y）、`a.java:5009` `n2 = n3 += this.Y << 1`（换行 +2Y）、
> `a.java:5004` `n5 += 16`（每格 16px）、`a.java:4996`/`a.java:5000` 逐格只取 1 字节
> （`this.i[n2]`/`this.j[n2]`）。故存储视角是**头值本身（26×26）的单字节格平铺**
> （`maplv51` 即 24×26），而非 Y×Z 大格的 4 字节结构；两种表述字节总量恒等
> （`4·Y·Z = 2Y·2Z·1`），本节记录的字节布局与全部复现结论不受影响。
> 附带行号更正：`grid2` 绘制调用 `this.a(…, this.j[n2])` 实为 `a.java:5000`（上文引 `line 5002` 偏 +2）。
> 复现：`grep -n 'this.ak \* this.Y << 1\|n3 += this.Y << 1\|n5 += 16\|this.i\[n2\]' reference/seed/a.java`
> → 命中 `4174 / 4202 / 4315 / 4324 / 4330 / 4335 / 4991 / 4996 / 5004 / 5009`
> （本注引用的是 `4991 / 4996 / 5004 / 5009`，其余命中属其他方法）；
> 字节恒等：`python3 -c "print(4*13*13 == 26*26)"` → `True`；
> 55 文件自洽：`cargo test -p game-data --test golden maplv_all_55_parse_zero_trailing`。

---

### 3. sprite0–54：4 字节计数头 + 变长记录（非固定 5 字节）

**结论**：文件头是 **4 字节**（两个 LE16 拼成一个 32 位小端整数，高位字在本语料实测恒为 0），
是**条目总数**；每条记录 = `1 字节类型码 + 2×u16(LE) 坐标 + 0~2 字节附加参数`，**附加参数长度由
类型码决定**（switch 分支），不是固定 5 字节——jar-forensics "sprite1=35 条 ×5B=184B 刚好吻合"
是**巧合**（该文件 35 条中 31 条〔【2026-10-09 更正】原写 ~~32 条~~ 为算术笔误（32+3+1=36≠35）；
经独立复算 31×0B+3×1B+1×2B=35 且 consumed==filesize，t8 评审发现〕恰好落在"0 附加字节"分支，3 条落在"1 附加字节"分支，1 条落在
"2 附加字节"分支，总字节数凑巧等于 184）。类型码即是**实体/对象类型 ID**，可直接查
`this.m`（88 个中文实体名数组，如"孙悟空/黄门/红门/封印门/门卫…"）得到语义名。

- Evidence（A级，代码定论）：`reference/seed/a.java:7264`-`7342` 方法内：
  - `line 7264`: `InputStream inputStream = a2.getClass().getResourceAsStream("sprite" + n4);`
  - `line 7270`: `int n6 = a.a(inputStream2) & 0xFFFF | a.a(inputStream2) << 16;`
    （**读两个 LE16，拼成 32 位计数**；非单个 u16）
  - `line 7271`: `for (int i = 0; i < n6; ++i) { ... }`（循环 n6 次）
  - 每条记录字段：`line 7275`: `int n7 = inputStream.read();`（1 字节类型码）；
    `line 7276`: `n3 = a.a(inputStream);`（u16 LE，x 坐标）；
    `line 7277`: `short s = a.a(inputStream);`（u16 LE，y 坐标）；
    其后 `switch(n7)`（`line 7278` 起，`case` 标签从 `line 7279` 起）按类型码决定追加 0/1/2 字节：
    - `case 6/12`（`line 7279-7284`）：**0 字节**附加，直接 `a2.a(n7,n3,s,0)` 并把结果设为"禁用"（`a2.a[idx]=false`）
    - `case 9/4/5/81/83`（`line 7285-7303`）：**1 字节**附加（`inputStream.read()`，部分 `+1`）
    - `case 7/8/57/59/70/71/72/73/77/78`（`line 7304-7331`）：**1~2 字节**附加（`read()`或`read()|read()<<8`）
    - `case 76/82`（`line 7332-7336`）：**2 字节**附加
    - `default`（`line 7337-7339`）：**0 字节**附加（`a2.a(n7,n3,s,0)`）
  - 条目构造函数 `private int a(int n, int n2, int n3, int n4)`（`line 4407`）：`n`=类型码,
    `n2`=x, `n3`=y, `n4`=附加参数；内部调用 `this.f(n, this.T)`（`line 4316-4336`，设定该实体的
    图像/高度）与 `this.g(n, this.T)`，并存位置（32px 网格对齐：`n2=n6<<5`）。
  - 类型码语义表：`line 545`，`this.m = new String[]{"孙悟空","黄门","红门","蓝门","封印门","门卫",
    "隐形路径","上楼梯","下楼梯","炼丹炉","云雾","能挖的墙","隐形墙","火眼金睛","生死簿","筋斗云",
    "芭蕉扇","金钩子","玄明石","千年月桂露",...}`（共 88 项，索引即类型码 `n7`）。
  >
  > 【2026-10-09 更正 ③】本节（含下方"字段布局表（sprite0–54）"）的 `a.java` 行号存在 **+2/+3 漂移**，
  > 以 `reference/seed/a.java`（8457 行，sha256 门禁锁定）逐行核对后的实测行号为准：
  > - 两个 LE16 拼计数 `int n6 = …`：实为 **`a.java:7270`**（表内引 `7272` 偏 +2）；
  > - 每条记录字段：类型码 `int n7 = inputStream.read();` = **7272**、x `n3 = a.a(inputStream);` = **7273**、
  >   y `short s = a.a(inputStream);` = **7274**（原文引 7275/7276/7277 偏 +3）；
  > - `switch (n7)`：**7275**（原文引 7278），`case` 标签自 **7276** 起（原文引 7279）；
  > - 各 case 区段实测：`6/12`=7276-7281、`9`=7282-7286、`4`=7287-7291、`5/81`=7292-7298、
  >   `83`=7299-7304、`7/8`=7305-7310、`57/59/70/71/72/73`=7311-7320、`77`=7321-7326、
  >   `78`=7327-7332、`76/82`=7333-7339、`default`=7340-7343（原文 7279-7339 各区间整体偏 +3 左右）。
  > 行号漂移**不改变**任何字节布局/字节消费结论（switch 分支语义与 55/55 复现结果均不受影响）。
  > 同源漂移也影响 `docs/findings/script-dsl-semantics.md` 对 `substring(n, n + 3)` 的引用
  > （其引 `a.java:6124`，实为 **6121**）——该文件不在本次更正范围，另行处理。
  > 复现：`grep -n 'a.a(inputStream2) & 0xFFFF\|int n7 = inputStream.read();\|switch (n7)\|substring(n, n + 3)' reference/seed/a.java`
  > → `2715 / 6121 / 7270 / 7272 / 7275`（`2715` 是 7 参数绘制方法里的另一个同名 `switch (n7)`，
  > 即网格2 变换分支，与 sprite 解析无关）；或 `sed -n '7270,7276p' reference/seed/a.java`。
- Evidence（A级，字节复现，变长解析器精确消费全部字节，佐证记录边界）：
  ```bash
  python3 /tmp/parse_sprite.py analysis/ex/sprite1
  # 输出: filesize=184 n6=35 consumed=184 remaining=0（35 条全部按 switch 规则解析完毕，零字节剩余/溢出）
  ```
  （解析脚本严格复刻 `line 7272-7340` 的 switch 分支字节消费逻辑，脚本见 `/tmp/parse_sprite.py`，
  复制粘贴即可复现——此为非仓库临时文件，用户可按下方代码重新生成）
  对 `sprite0,sprite1,sprite10,sprite25,sprite54` 及全部 55 个文件批量验证：**55/55 文件 `consumed==filesize`**。
  ```bash
  python3 - <<'EOF'
  import glob
  def read_u16le(buf,pos): return buf[pos]&0xFF | (buf[pos+1]<<8 & 0xFF00), pos+2
  bad=[]
  for fn in sorted(glob.glob('analysis/ex/sprite*'), key=lambda x:int(x.replace('analysis/ex/sprite',''))):
      with open(fn,'rb') as f: data=f.read()
      pos=0
      lo,pos=read_u16le(data,pos); hi,pos=read_u16le(data,pos)
      n6=(lo&0xFFFF)|(hi<<16)
      for i in range(n6):
          n7=data[pos]; pos+=1
          _,pos=read_u16le(data,pos); _,pos=read_u16le(data,pos)
          if n7 in (6,12): pass
          elif n7 in (9,4,5,81,83): pos+=1
          elif n7 in (7,8,76,82): pos+=2
          elif n7 in (57,59,70,71,72,73,77,78): pos+=1
          else: pass
      if pos != len(data): bad.append((fn, pos, len(data)))
  print('mismatched files:', bad if bad else 'NONE (55/55 OK)')
  EOF
  ```
- Evidence（A级，类型码取值域校验）：全部 55 个 sprite 文件中观测到的类型码集合为
  `{1..15, 21, 23, 25..86}`（最大值 86），落在 `this.m` 长度 88（索引 0–87）范围内，无越界——
  支持"类型码=`this.m`数组索引"的解读。
  >
  > 【2026-10-09 更正 ②】上面的集合表述 `{1..15, 21, 23, 25..86}` **过宽**：全量 55 个 sprite 文件中
  > **类型码 82 从未出现**（应与 0/16-20/22 并列为未用码，见"线索登记"）。准确的观测集合是
  > `{1..15, 21, 23, 25..81, 83..86}`（最大值 86、"落在 `this.m` 88 项内、无越界"的结论不变）。
  > 注意 `case 76/82` 的 switch 分支真实存在（`a.java:7333-7339`），只是语料中 82 号对象从未被
  > 任何关卡放置，成因未查证。
  > 复现（cargo test 或 python）：
  > `cargo test -p game-data --test golden sprite_type_code_set_matches_corpus`
  > ```bash
  > python3 - <<'EOF'
  > import glob
  > def rl(b, p): return b[p] | b[p + 1] << 8, p + 2
  > s = set()
  > for fn in glob.glob('assets/raw/sprite*'):
  >     d = open(fn, 'rb').read(); p = 0
  >     lo, p = rl(d, p); hi, p = rl(d, p)   # +0/+2 计数头（a.java:7270），记录自 +4 起
  >     for _ in range(lo | hi << 16):
  >         t = d[p]; s.add(t); p += 1
  >         _, p = rl(d, p); _, p = rl(d, p)
  >         p += 2 if t in (7, 8, 76, 82) else 1 if t in (4,5,9,57,59,70,71,72,73,77,78,81,83) else 0
  > print(sorted(s))   # → [1..15, 21, 23, 25..81, 83..86]，其中无 82
  > EOF
  > ```

#### 字段布局表（sprite0–54）

| 偏移 | 类型 | 含义 | 证据 |
|---|---|---|---|
| +0 | u16 LE | 条目计数低 16 位 | `a.java:7272` |
| +2 | u16 LE | 条目计数高 16 位（本语料实测恒为 0） | `a.java:7272` |
| 每条记录 +0 | u8 | **实体/对象类型码**，索引 `this.m[code]` 得中文名 | `a.java:7275`；`a.java:545` |
| 每条记录 +1 | u16 LE | x 坐标（像素，后续按 32px 网格对齐：`(x+16)>>5<<5`） | `a.java:7276`；`a.java:4409-4418` |
| 每条记录 +3 | u16 LE | y 坐标（同上对齐规则） | `a.java:7277`；`a.java:4409-4418` |
| 每条记录 +5 | 0/1/2 字节（由类型码决定） | 附加参数（含义依类型码变化：数量/朝向/子状态等，未逐类型码穷举，见遗留问题） | `a.java:7278-7340` |

> 【2026-10-09 更正 ③】上表证据列的 `a.java:7272/7275/7276/7277/7278-7340` 均有 +2/+3 行号漂移，
> 实测为 `7270`（计数）、`7272/7273/7274`（类型码/x/y）、`7275-7343`（switch 区段）；
> 逐条对照与复现命令见上文 §3 "代码定论" 证据块下的【2026-10-09 更正 ③】。字段布局本身不变。

---

### 4. spt* 图集与 sprite 记录、与 packed PNG 容器的关系

**结论**：`sprite{n}` 文件里的"类型码"**不是**直接索引 `spt*` 容器内的图片序号，而是先经过一层
"类型码 → 图集桶 + 桶内帧号"的**静态重映射表**（硬编码在构造函数里），再由该桶对应到某个 `spt*`
容器文件、取其中第几帧 `Image`。`Image.createImage(byte[],0,n)`（`line 2674`）正是解码 **packed PNG
容器内每一张子图**的调用点（见"已证实 1"），其解码结果存入 `this.a[资源索引][帧号]`（`Image[][]`）。

- Evidence（A级，代码定论）：
  - `line 7232-7257`：类型码到图集桶的重映射，例如
    `a3.a[n2] = a3.a[4][n2-1]`（`n2`=13..32 → 桶 4，即 `this.b[4]="sptmap"`，帧号 `n2-13`）、
    `a3.a[n2] = a3.a[6][n2-33]`（33..40 → 桶 6 `"sptarm"`）、
    `a3.a[n2] = a3.a[7][n2-41]`（41..60 → 桶 7 `"sptenemy1"`）、
    `a3.a[n2] = a3.a[13][n2-61]`（61..78 → 桶 13 `"sptenemy2"`，见 `line 465` 桶序号对照）。
  - 资源名桶序号对照（`line 465`，0-based）：
    `0 sflogo, 1 mapbg, 2 map, 3 actor, 4 sptmap, 5 sptprop, 6 sptarm, 7 sptenemy1, 8 ui, 9 xtq,
    10 menu, 11 intro, 12 face, 13 sptenemy2, 14 end, 15 load`。
  - `this.a[桶][帧]`（`Image[][]`）由 `line 2657-2676` 的 `a(int n)` 方法在**首次使用该桶时**
    触发资源加载（`this.a[n] == null` 判断，`line 2659`），逐帧 `Image.createImage` 解码 packed PNG
    容器内的子图（证据同"已证实 1"）。
- 结论定性：类型码（0-87，索引 `this.m`）→（硬编码分段映射）→（图集桶号 0-15，索引 `this.b`）
  → 图集桶内帧号 → 该 packed PNG 容器内第几张子图。链路完整，三种格式在运行时通过这条映射串联。

---

### 5. 线索登记（可疑/异常数据，不修，仅记录）

- **`maplv51` 非方形网格**：头为 `(24,26)` → 还原 `Y=12, Z=13`（12×13，不是其余 54 个关卡统一的
  13×13）。文件总长 `1252B` 与公式 `4+2×(12×13×4)=1252` 自洽，**非坏数据**，只是该层地图本身更窄。
  复现：`xxd -l 4 analysis/ex/maplv51` → `1800 1a00`（0x18=24, 0x1a=26）。记录为**线索而非 bug**
  （格式层面完全自洽，只是尺寸特例，不登记入 `docs/bug-ledger.md`）。
- **sprite 类型码 0/16-20/22 在全部 55 个文件中均未出现**：`this.m` 数组 0 号是"孙悟空"（玩家角色，
  大概率不作为"关卡放置物"写入 sprite 文件，而是运行时固定创建）；16-20/22 号具体为何未被任何关卡
  使用尚未查证，留作假设区条目。
  >
  > 【2026-10-09 更正 ②】**类型码 82 亦从未出现**，应并入本未用码清单（0/16-20/22/82）；
  > 详见 §3"类型码取值域校验"下的【2026-10-09 更正 ②】（`case 76/82` 分支存在但语料未用）。

---

## 假设（C 级，禁止据此改名/落台账）

1. **"26×26×1B 单字节格" vs "13×13×4B 复合格"哪个是设计者原意**：两者字节总量相同（676B/网格），
   当前只验证了"代码按 `Y=head>>1=13、单字节/格` 读取并消费"这一事实本身是 A 级（直接读代码），
   但"为什么头要存 2× 真实值"背后的设计动机未知——猜测可能是历史遗留（早期版本每格 2 字节，
   后改为 1 字节但头字段忘记同步减半）或是与某种 16px/32px 双精度坐标系统有关联。此猜测不改变
   已证实的字节布局结论，仅作背景猜测标注。
   > 【2026-10-09 更正 ④】本假设的"布局之争"部分已由绘制循环索引证据**定案**为 2Y×2Z 单字节格
   > （见 §2 注1 下方【2026-10-09 更正 ④】）；本条**仅保留"设计动机"假设**（头为何存 2× 真实值）。
2. **sprite 记录"附加参数"字段的具体业务含义**（数量/方向/子状态/关联对象ID等）未逐类型码穷举——
   仅看到字节消费规则（0/1/2 字节），未深挖每个值在游戏逻辑里对应什么语义（如 `case 83` 的 1 字节
   参数、`case 76/82` 的 2 字节参数具体是什么，需要结合 `this.a(n,n2,n3,n4)` 内部对 `n4` 的使用
   逐类型码跟踪，工作量较大，列入遗留问题）。
3. **网格2（翻转/变换码）与 DirectGraphics TRANS_* 常量的精确对应关系**：代码里 `switch(n7){case 1..7}`
   的几何变换逻辑已读（`line 2708-2744`），但未与 Nokia `com.nokia.mid.ui.DirectGraphics` 官方
   TRANS_MIRROR/TRANS_ROT90 等常量值逐一核对数值是否一致——此为合理推测但未做官方文档交叉比对，
   列为 B/C 之间的待升级结论。
4. **`script` 资源文件与 sprite/maplv 的直接代码级连接点未定位**：本次任务范围（packed PNG/maplv/sprite）
   未检索到以字面量 `"script"` 读取资源的调用点，`GUTS:` 段脚本指令（`MOV_/TAK_/DES_` 等）与
   sprite 记录类型码之间的具体映射关系（例如脚本里的对象编号是否就是 sprite 记录的插入顺序）
   未在本次考证中核实，列入遗留问题，不作结论。

---

## 遗留问题清单

1. sprite 记录中依类型码变化的 0/1/2 字节附加参数，逐类型码的业务语义未穷举（见"假设 2"）。
2. `maplv` 头字段为何存"真实值×2"（而非直接存真实值）的设计动机未知（见"假设 1"）。
3. `maplv51` 为何是全档案中唯一非 13×13 的关卡（12×13）——是否对应某个特殊剧情楼层，未与
   `script` 文本的 `GUTS:` 段 51 号关卡事件交叉核对。
4. 网格2（翻转码）数值域与 Nokia DirectGraphics 官方 TRANS_* 常量的精确映射未做官方文档比对（见"假设 3"）。
5. `script` 资源读取入口未在 `reference/seed/a.java` 中以字面量定位，sprite 记录与 `GUTS:` 脚本事件编号
   的关联关系未验证（见"假设 4"）。
6. sprite 类型码 0、16-20、22 在全部 55 层中均未被使用，具体原因（专用生成逻辑/测试遗留/未启用内容）未查证。
   〔2026-10-09 更正 ②：**82** 亦未被任何层使用，应并入本清单，见 §3 更正 ②。〕
7. 本次只对 55 个 `maplv*`/`sprite*` 中的典型样本做了逐字节抽查式交叉解读（如具体某条记录对应哪个
   `this.m` 实体名），未对全部条目做语义级人工复核，仅做了格式级（字节边界/计数）批量验证。
