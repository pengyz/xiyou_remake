# tools/rename-pipeline — 机械反混淆管线 v1

字节码级**签名唯一化重映射**管线：把 `original/` 固化 JAR 里同名重载的混淆符号
（`a`/`b`/`c`/…/`am`/`bZ` 等）按类型/签名机械唯一名化，再经 CFR 再生**无歧义版 Java**，
并用 javap 指令级 diff 证明「仅符号名变化、语义不变」。

**只做确定性机械变换，不做语义命名**（语义命名由 `data/naming/ledger.jsonl` 台账驱动、
叠加在本管线映射表之上）。

## 用法

```bash
python3 tools/rename-pipeline/run.py            # 构建：重映射 + CFR 再生
python3 tools/rename-pipeline/run.py --verify   # 构建 + javap 指令级等价验证与报告
```

一键、幂等：输出纯确定性（无时间戳），同一 JAR 重跑逐字节一致
（指纹见 `tools/rename-pipeline/_verify/manifest.json`）。

## 命名方案（v1，见 `renamer.py`）

| 目标 | 规则 | 例 |
|---|---|---|
| 字段 | `f_<type>_<NN>`（type 记号 + 类内同 type 序号 2 位） | `int a` → `f_int_07` |
| 方法 | `m_<NNN>`（类内声明序号 3 位） | `a(int,int)` → `m_042` |

- type 记号：`I→int B→byte Z→bool S→short C→char J→long F→float D→double`、
  `Lpkg/Cls;→Cls`、数组 = 元素记号 `+_arr`（多维 `_arr2/_arr3`…）。
- 改名对象：混淆名 `^[a-zA-Z]{1,2}$`（本 JAR 混淆器产出空间）**且** private/static
  （不可能 override 外部 API）的字段/方法。
- 保留原名：构造器、`<clinit>`、`paint`/`run`/`keyPressed`/`startApp` 等 MIDP API
  override/实现 —— 改名会改变虚分派语义。
- 序号取 class 文件声明序 ⇒ 幂等、可复算。

## 实现方式（为什么不用 ASM）

**常量池手术**（`classfile.py`，纯 Python，无二进制依赖）：向常量池**末尾追加**新
Utf8/NameAndType 条目，只把 ① 成员声明的 `name_index`、②
Fieldref/Methodref/InterfaceMethodref 的 `name_and_type_index` 改指向新条目。
既有索引一律不变 ⇒ **Code 及全部子属性逐块一致**（属性 raw 字节原样搬运，`--verify`
逐块 sha256 机器校验）；**LineNumberTable 不适用（本 class 不含该属性）**，指令流原样。
（措辞校准 t9-F1：本 JAR 两个 class 均无 LineNumberTable —— javap -l 为 0 处，
早先「`Code`/`LineNumberTable` 零改动」是空真表述，易误读为校验过行号表。）
与 ASM 同等的字节码改写语义，且可审计、离线可跑、输出确定。

安全性硬校验（任一失败即退出码非 0）：
1. 重写后无任何成员引用仍指向旧 `(owner, name, desc)`；
2. 类内字段+方法名全局唯一；
3. 疑似混淆名但非 private/static 的成员 ⇒ 拒绝自动改名（防误伤 override）；
4. 常量池出现 MethodHandle/InvokeDynamic/Dynamic ⇒ 拒绝继续（超出安全范围）；
5. 产物双路扫描（`javap -p` + CFR 源码）：混淆短名残留 0 个；
6. 改名前后全部属性块（含 Code 子属性）raw sha256 逐块相同。

## javap 等价验证（`--verify`，`verifier.py`）

`javap -c -p -l` 分别反汇编改名前/后 class → 归一化（常量池槽号 `#NNN → #<cp>`）
→ 对改名后文本按映射表做**符号还原** → 两份文本逐字节比对。
归一化后 **diff = 0 行** ⇒ 指令序列、每条指令操作数指向的具体成员
（owner+name+desc）、异常表、成员签名、访问标志全部一致，差异只剩符号名。
另做**属性块 raw 逐块 sha256 对比**：**Code 及全部子属性逐块一致；LineNumberTable
不适用（本 class 不含该属性）**（本 JAR 两 class 均无 LineNumberTable/LocalVariableTable，
javap -l 为 0 处，不把「无此属性」计作「已比对一致」）。

报告与逐类 diff 归档在 [`_verify/`](_verify/)（`report.md`、`diff-<class>.txt`）。

## 工具链校验（CFR pin，t9-F2；跨平台改造见 tools/cfr_env.py）

CFR 0.152 完整 sha256 =
`f686e8f3ded377d7bc87d216a90e9e9512df4156e75b06c655a16648ae8765b2`
（本地实算，并与 Maven Central `cfr-0.152.jar.sha256` 交叉核对一致）。
定位/校验/获取的**单一真相源**是 [`tools/cfr_env.py`](../cfr_env.py)：
查找顺序 = 入库 vendor 副本 [`tools/vendor/cfr-0.152.jar`](../vendor/) → `analysis/cfr.jar`
缓存 → Python `urllib` 下载（**不依赖 curl**，Windows 亦可），每步都校验上述 sha256，
不匹配即 FAIL 并提示手动放置（下载物校验失败即删除，不污染缓存）。
逻辑全在 `.py`：[`tools/decompile.py`](../decompile.py) 承载反编译对比，
[`tools/decompile.sh`](../decompile.sh) 只是 POSIX 薄包装。
离线/缓存用法见
[`docs/knowledge/reference_cfr_decompiler.md`](../../docs/knowledge/reference_cfr_decompiler.md)。

## 中文可读化后处理（t12）

CFR 产物再做一次确定性后处理（[`deunicode.py`](deunicode.py)）：字符串/字符**字面量内**
码点 ≥ 0x00A0 的 `\uXXXX` 解码为 UTF-8 明文（含全部中文）；ASCII 范围转义（< 0x00A0，
如 `\u0022`/`\u005C`）保留以免词法歧义；注释/标识符不动。
只改源文本呈现：回环 `encode(decode(x)) == x` 逐字节成立（构建时硬校验，失败拒绝落盘）
⇒ 运行期字符串值零变化；`--verify` 另抽 3 处中文字符串与 class 常量池 UTF-8 逐字对照。
`reference/seed/` 保持 CFR 转义原样（D3 锚点一字节不动）。

## 产物

| 路径 | 内容 | 入库 |
|---|---|---|
| `reference/src/deobf/*.java` | CFR 再生的无歧义版 Java（重映射 class 的投影） | ✓ |
| `data/naming/remap-table.json` | 全量 old→new 符号映射表（含 kind/signature/证据） | ✓ |
| `tools/rename-pipeline/_verify/` | javap 等价验证报告 + diff | ✓ |
| `analysis/rename-pipeline/` | 提取/重映射 class、javap 反汇编（可再生） | ✗ |

## 范围与限制

- **不改类名**：`a` / `CMidlet` 保持原名（契约范围是字段/方法符号）。
- **不碰 `original/` 与 `reference/seed/`**：original 只读解包到 `analysis/`；
  `reference/seed/` 的完整性由 `gates/cli.py reference-seed` 保护，本管线不触碰。
- **局部变量名不在范围**：字节码无 LocalVariableTable，`reference/src/deobf/` 里的
  `n`/`c`/`s`… 是 CFR 按类型启发式生成的投影层命名（方法作用域内唯一、无歧义）。
  产物保持 CFR 原样输出（契约：「重映射后经 CFR 再生」），报告中已透明披露计数。
- 语义命名**叠加**于本管线映射之上（台账驱动）：`data/naming/ledger.jsonl` 中
  `status: applied` 的记录会在构建时把对应机械名替换为语义名（改写与 javap 验证同用
  「语义名→机械名→原始名」两层还原），`remap-table.json` 保持纯机械层逐字节不变；
  `status: hypothesis` 不动代码。schema 见 [`data/naming/README.md`](../../data/naming/README.md)。
