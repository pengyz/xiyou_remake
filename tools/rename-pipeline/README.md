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

## 工具链校验（Vineflower pin；跨平台改造见 tools/vf_env.py）

> 【2026-10-10 更正】本章原文描述 CFR 0.152 的 vendor/校验机制（t9-F2 建立）。
> 引擎已切换 Vineflower 1.12.0（决策与实测数据见
> docs/knowledge/decision_decompiler-cfr-to-vineflower.md），CFR 工具链退役移除；
> 机制原样继承：vendor 入库 → sha256 钉死 → urllib 下载兜底，单一真相源
> [`tools/vf_env.py`](../vf_env.py)。历史论证保留于本仓 git 历史（commit a7b5cde）。

| 项 | 值 |
|---|---|
| 引擎 | Vineflower 1.12.0（`tools/vendor/vineflower-1.12.0.jar`） |
| sha256 | `1dfcfe97…`（tools/vf_env.py 钉死，校验失败拒绝使用） |
| 产出 | `reference/src/deobf/*.java`（直出 UTF-8 中文）+ `analysis/build/deobf/*.class`（javac，oracle 变体 C） |


## 中文可读化（历史章节，已随引擎切换退役）

> 【2026-10-10 更正】v1 时代 CFR 产物需 `deunicode.py` 后处理（字面量 `\uXXXX` → UTF-8
> 明文，回环硬校验）。Vineflower 直出 UTF-8 中文（仅 5 处必要的控制字符转义），
> `deunicode.py` 已移除；中文 × 常量池 UTF-8 对照仍保留在 `--verify` 报告中。
> `reference/seed/` 保持历史 CFR 转义原样（D3 锚点一字节不动）。

## 产物

| 路径 | 内容 | 入库 |
|---|---|---|
| `reference/src/deobf/*.java` | Vineflower 再生 + 投影修复补丁的可编译 Java | ✓ |
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
