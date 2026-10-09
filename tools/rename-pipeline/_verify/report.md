# javap 指令级等价验证报告 — 机械反混淆管线 v1

- 结论：**PASS — 重映射仅改变符号名，语义不变**
- 复现：`python3 tools/rename-pipeline/run.py --verify`（确定性输出，无时间戳 ⇒ 幂等）
- 输入：`original/囧囧西游-大闹天宫.jar`（只读）→ `analysis/rename-pipeline/orig/*.class`
- 对比对象：`analysis/rename-pipeline/orig/` vs `analysis/rename-pipeline/renamed/*.class`
- 映射表：`data/naming/remap-table.json`（581 条符号，机械改名 569 条）
- 语义叠加：`data/naming/ledger.jsonl`（applied 82 条，改写与验证均经两层符号还原：语义名→机械名→原始名）

## 验证方法（为什么这能证明“仅符号名变化”）

1. `javap -c -p -l` 分别反汇编改名前/后 class，逐指令文本落盘（`analysis/rename-pipeline/verify/<class>.{before,after}.javap.txt`）；
2. 归一化两份文本：常量池槽号 `#NNN → #<cp>`（重映射以常量池**末尾追加**新 Utf8/NameAndType 的方式实施，槽号位移是布局而非语义）；
3. 对改名后文本做**符号还原**：按映射表把新名（机械名 f_*/m_* 或台账叠加的语义名）替换回旧名。替换只发生在成员声明行与 `// Field|Method|InterfaceMethod` 注释中，字符串常量等不动；被限定引用（owner.name:desc）按 owner 查表，非限定引用按当前类查表；
4. 两份归一化文本**逐字节相同**（diff 0 行）⇒ 指令序列、每条指令操作数指向的具体成员（owner+name+desc）、异常表、成员签名、访问标志全部一致 —— 差异只剩符号名本身。
5. 属性块 raw 字节**逐块 sha256 对比**（常量池手术只动 name_index/nat_index，属性 payload 原样搬运，故可直接逐块比对）：**Code 及全部子属性逐块一致；LineNumberTable 不适用（本 class 不含该属性）**。（措辞校准：本 JAR 两个 class 均无 LineNumberTable/LocalVariableTable —— javap -l 为 0 处，此前「行号表一致」是空真表述，不作已校验项计。）逐块计数见下节。

同时跑四重自检（失败即退出码非 0）：① 重写后无任何 Fieldref/Methodref 仍指向旧 (owner,name,desc)；② 类内成员名（字段+方法合并）唯一；③ 产物无混淆短名残留（javap 声明 + CFR 源码双路扫描）；④ 改名前后全部属性块（含 Code 子属性）raw sha256 逐块相同。

## 逐类结果

| class | 成员数 | 指令数 | 归一化 sha256(前) | 归一化 sha256(后) | diff 行数 | 结论 |
|---|---|---|---|---|---|---|
| CMidlet | 7 | 41 | `03ccbe9a904f7c90…` | `03ccbe9a904f7c90…` | 0 | 一致 ✓ |
| a | 574 | 39154 | `05602b6efcf86866…` | `05602b6efcf86866…` | 0 | 一致 ✓ |

完整 diff（空 = 无差异）：`tools/rename-pipeline/_verify/diff-<class>.txt`。

## 属性块 raw 对比（逐块 sha256，含 Code 全部子属性）

### CMidlet —— 全部属性块逐块一致 ✓

| 属性块 | 层级 | 数量 | raw 字节逐块一致 |
|---|---|---|---|
| StackMap | Code 子属性 | 1 | ✓ |
| Code | 属性 | 5 | ✓ |

- `LineNumberTable` / `LocalVariableTable`：**0 处**（本 class 不含该属性 ⇒ **不适用**，不计作『已比对一致』）。

### a —— 全部属性块逐块一致 ✓

| 属性块 | 层级 | 数量 | raw 字节逐块一致 |
|---|---|---|---|
| StackMap | Code 子属性 | 135 | ✓ |
| Code | 属性 | 163 | ✓ |
| Exceptions | 属性 | 1 | ✓ |

- `LineNumberTable` / `LocalVariableTable`：**0 处**（本 class 不含该属性 ⇒ **不适用**，不计作『已比对一致』）。

## CMidlet 逐成员对比（7 个成员）

| # | 旧名 | 新名 | 指令数(前) | 指令数(后) | 归一化块一致 |
|---|---|---|---|---|---|
| 1 | a | f_CMidlet_00 | 0 | 0 | ✓ |
| 2 | a | f_a_00 | 0 | 0 | ✓ |
| 3 | CMidlet | CMidlet | 5 | 5 | ✓ |
| 4 | startApp | startApp | 23 | 23 | ✓ |
| 5 | pauseApp | pauseApp | 4 | 4 | ✓ |
| 6 | destroyApp | destroyApp | 1 | 1 | ✓ |
| 7 | a | m_000 | 8 | 8 | ✓ |

## a 逐成员对比（574 个成员）

| # | 旧名 | 新名 | 指令数(前) | 指令数(后) | 归一化块一致 |
|---|---|---|---|---|---|
| 1 | a | f_Font_00 | 0 | 0 | ✓ |
| 2 | a | f_int_00 | 0 | 0 | ✓ |
| 3 | b | f_int_01 | 0 | 0 | ✓ |
| 4 | c | f_int_02 | 0 | 0 | ✓ |
| 5 | a | f_String_00 | 0 | 0 | ✓ |
| 6 | a | f_String_arr_00 | 0 | 0 | ✓ |
| 7 | a | f_bool_00 | 0 | 0 | ✓ |
| 8 | a | f_Graphics_00 | 0 | 0 | ✓ |
| 9 | b | f_bool_01 | 0 | 0 | ✓ |
| 10 | d | f_int_03 | 0 | 0 | ✓ |
| 11 | a | gameMode | 0 | 0 | ✓ |
| 12 | b | f_byte_01 | 0 | 0 | ✓ |
| 13 | c | f_byte_02 | 0 | 0 | ✓ |
| 14 | a | f_DirectGraphics_00 | 0 | 0 | ✓ |
| 15 | e | f_int_04 | 0 | 0 | ✓ |
| 16 | a | f_Image_00 | 0 | 0 | ✓ |
| 17 | c | f_bool_02 | 0 | 0 | ✓ |
| 18 | b | f_String_arr_01 | 0 | 0 | ✓ |
| 19 | a | f_int_arr_00 | 0 | 0 | ✓ |
| 20 | a | f_Image_arr2_00 | 0 | 0 | ✓ |
| 21 | b | f_int_arr_01 | 0 | 0 | ✓ |
| 22 | a | f_int_arr2_00 | 0 | 0 | ✓ |
| 23 | a | f_InputStream_00 | 0 | 0 | ✓ |
| 24 | a | f_OutputStream_00 | 0 | 0 | ✓ |
| 25 | b | f_InputStream_01 | 0 | 0 | ✓ |
| 26 | f | f_int_05 | 0 | 0 | ✓ |
| 27 | g | f_int_06 | 0 | 0 | ✓ |
| 28 | c | f_String_arr_02 | 0 | 0 | ✓ |
| 29 | h | f_int_07 | 0 | 0 | ✓ |
| 30 | i | f_int_08 | 0 | 0 | ✓ |
| 31 | j | f_int_09 | 0 | 0 | ✓ |
| 32 | a | f_byte_arr_00 | 0 | 0 | ✓ |
| 33 | k | f_int_10 | 0 | 0 | ✓ |
| 34 | l | f_int_11 | 0 | 0 | ✓ |
| 35 | m | f_int_12 | 0 | 0 | ✓ |
| 36 | n | f_int_13 | 0 | 0 | ✓ |
| 37 | o | f_int_14 | 0 | 0 | ✓ |
| 38 | p | f_int_15 | 0 | 0 | ✓ |
| 39 | q | f_int_16 | 0 | 0 | ✓ |
| 40 | b | f_byte_arr_01 | 0 | 0 | ✓ |
| 41 | c | f_byte_arr_02 | 0 | 0 | ✓ |
| 42 | d | f_byte_03 | 0 | 0 | ✓ |
| 43 | r | f_int_17 | 0 | 0 | ✓ |
| 44 | s | f_int_18 | 0 | 0 | ✓ |
| 45 | b | f_Image_01 | 0 | 0 | ✓ |
| 46 | t | f_int_19 | 0 | 0 | ✓ |
| 47 | u | f_int_20 | 0 | 0 | ✓ |
| 48 | b | f_String_01 | 0 | 0 | ✓ |
| 49 | v | f_int_21 | 0 | 0 | ✓ |
| 50 | d | f_bool_03 | 0 | 0 | ✓ |
| 51 | e | f_byte_04 | 0 | 0 | ✓ |
| 52 | c | f_int_arr_02 | 0 | 0 | ✓ |
| 53 | w | f_int_22 | 0 | 0 | ✓ |
| 54 | x | f_int_23 | 0 | 0 | ✓ |
| 55 | e | f_bool_04 | 0 | 0 | ✓ |
| 56 | f | f_bool_05 | 0 | 0 | ✓ |
| 57 | f | f_byte_05 | 0 | 0 | ✓ |
| 58 | g | f_byte_06 | 0 | 0 | ✓ |
| 59 | h | f_byte_07 | 0 | 0 | ✓ |
| 60 | i | f_byte_08 | 0 | 0 | ✓ |
| 61 | y | f_int_24 | 0 | 0 | ✓ |
| 62 | z | f_int_25 | 0 | 0 | ✓ |
| 63 | c | f_String_02 | 0 | 0 | ✓ |
| 64 | d | f_String_03 | 0 | 0 | ✓ |
| 65 | d | f_String_arr_03 | 0 | 0 | ✓ |
| 66 | A | f_int_26 | 0 | 0 | ✓ |
| 67 | B | f_int_27 | 0 | 0 | ✓ |
| 68 | C | f_int_28 | 0 | 0 | ✓ |
| 69 | D | f_int_29 | 0 | 0 | ✓ |
| 70 | E | f_int_30 | 0 | 0 | ✓ |
| 71 | F | f_int_31 | 0 | 0 | ✓ |
| 72 | G | f_int_32 | 0 | 0 | ✓ |
| 73 | H | f_int_33 | 0 | 0 | ✓ |
| 74 | I | f_int_34 | 0 | 0 | ✓ |
| 75 | J | playerHp | 0 | 0 | ✓ |
| 76 | K | playerAtk | 0 | 0 | ✓ |
| 77 | L | playerDef | 0 | 0 | ✓ |
| 78 | j | equippedWeaponType | 0 | 0 | ✓ |
| 79 | k | equippedArmorType | 0 | 0 | ✓ |
| 80 | b | f_int_arr2_01 | 0 | 0 | ✓ |
| 81 | d | f_int_arr_03 | 0 | 0 | ✓ |
| 82 | M | f_int_38 | 0 | 0 | ✓ |
| 83 | l | f_byte_11 | 0 | 0 | ✓ |
| 84 | N | playerPixelX | 0 | 0 | ✓ |
| 85 | O | playerPixelY | 0 | 0 | ✓ |
| 86 | P | playerCellX | 0 | 0 | ✓ |
| 87 | Q | playerCellY | 0 | 0 | ✓ |
| 88 | m | f_byte_12 | 0 | 0 | ✓ |
| 89 | R | f_int_43 | 0 | 0 | ✓ |
| 90 | S | f_int_44 | 0 | 0 | ✓ |
| 91 | c | f_Image_02 | 0 | 0 | ✓ |
| 92 | e | f_int_arr_04 | 0 | 0 | ✓ |
| 93 | f | f_int_arr_05 | 0 | 0 | ✓ |
| 94 | g | f_bool_06 | 0 | 0 | ✓ |
| 95 | h | f_bool_07 | 0 | 0 | ✓ |
| 96 | i | f_bool_08 | 0 | 0 | ✓ |
| 97 | n | f_byte_13 | 0 | 0 | ✓ |
| 98 | o | f_byte_14 | 0 | 0 | ✓ |
| 99 | e | f_String_arr_04 | 0 | 0 | ✓ |
| 100 | g | entityPixelX | 0 | 0 | ✓ |
| 101 | h | entityPixelY | 0 | 0 | ✓ |
| 102 | i | f_int_arr_08 | 0 | 0 | ✓ |
| 103 | j | f_int_arr_09 | 0 | 0 | ✓ |
| 104 | k | f_int_arr_10 | 0 | 0 | ✓ |
| 105 | a | f_Image_arr_00 | 0 | 0 | ✓ |
| 106 | l | entityType | 0 | 0 | ✓ |
| 107 | m | f_int_arr_12 | 0 | 0 | ✓ |
| 108 | T | f_int_45 | 0 | 0 | ✓ |
| 109 | a | f_bool_arr_00 | 0 | 0 | ✓ |
| 110 | b | f_bool_arr_01 | 0 | 0 | ✓ |
| 111 | d | f_byte_arr_03 | 0 | 0 | ✓ |
| 112 | a | entityParam | 0 | 0 | ✓ |
| 113 | c | f_bool_arr_02 | 0 | 0 | ✓ |
| 114 | e | f_byte_arr_04 | 0 | 0 | ✓ |
| 115 | f | f_byte_arr_05 | 0 | 0 | ✓ |
| 116 | n | f_int_arr_13 | 0 | 0 | ✓ |
| 117 | c | f_int_arr2_02 | 0 | 0 | ✓ |
| 118 | g | f_byte_arr_06 | 0 | 0 | ✓ |
| 119 | h | f_byte_arr_07 | 0 | 0 | ✓ |
| 120 | d | f_bool_arr_03 | 0 | 0 | ✓ |
| 121 | U | f_int_46 | 0 | 0 | ✓ |
| 122 | j | f_bool_09 | 0 | 0 | ✓ |
| 123 | V | f_int_47 | 0 | 0 | ✓ |
| 124 | W | f_int_48 | 0 | 0 | ✓ |
| 125 | a | f_byte_arr2_00 | 0 | 0 | ✓ |
| 126 | b | f_byte_arr2_01 | 0 | 0 | ✓ |
| 127 | X | f_int_49 | 0 | 0 | ✓ |
| 128 | d | f_Image_03 | 0 | 0 | ✓ |
| 129 | Y | mapCellsWide | 0 | 0 | ✓ |
| 130 | Z | mapCellsHigh | 0 | 0 | ✓ |
| 131 | aa | f_int_52 | 0 | 0 | ✓ |
| 132 | ab | f_int_53 | 0 | 0 | ✓ |
| 133 | i | mapTerrainGrid | 0 | 0 | ✓ |
| 134 | j | mapTransformGrid | 0 | 0 | ✓ |
| 135 | a | f_bool_arr2_00 | 0 | 0 | ✓ |
| 136 | c | f_byte_arr2_02 | 0 | 0 | ✓ |
| 137 | k | f_byte_arr_10 | 0 | 0 | ✓ |
| 138 | d | f_byte_arr2_03 | 0 | 0 | ✓ |
| 139 | p | f_byte_15 | 0 | 0 | ✓ |
| 140 | e | f_bool_arr_04 | 0 | 0 | ✓ |
| 141 | ac | f_int_54 | 0 | 0 | ✓ |
| 142 | ad | f_int_55 | 0 | 0 | ✓ |
| 143 | ae | f_int_56 | 0 | 0 | ✓ |
| 144 | af | f_int_57 | 0 | 0 | ✓ |
| 145 | ag | f_int_58 | 0 | 0 | ✓ |
| 146 | ah | f_int_59 | 0 | 0 | ✓ |
| 147 | ai | f_int_60 | 0 | 0 | ✓ |
| 148 | aj | f_int_61 | 0 | 0 | ✓ |
| 149 | ak | f_int_62 | 0 | 0 | ✓ |
| 150 | al | f_int_63 | 0 | 0 | ✓ |
| 151 | k | f_bool_10 | 0 | 0 | ✓ |
| 152 | l | f_bool_11 | 0 | 0 | ✓ |
| 153 | am | currentFloor | 0 | 0 | ✓ |
| 154 | an | minFloorReached | 0 | 0 | ✓ |
| 155 | ao | maxFloorReached | 0 | 0 | ✓ |
| 156 | ap | f_int_67 | 0 | 0 | ✓ |
| 157 | aq | f_int_68 | 0 | 0 | ✓ |
| 158 | ar | alchemyUpgradeCount | 0 | 0 | ✓ |
| 159 | as | f_int_70 | 0 | 0 | ✓ |
| 160 | at | f_int_71 | 0 | 0 | ✓ |
| 161 | au | f_int_72 | 0 | 0 | ✓ |
| 162 | av | f_int_73 | 0 | 0 | ✓ |
| 163 | aw | f_int_74 | 0 | 0 | ✓ |
| 164 | ax | f_int_75 | 0 | 0 | ✓ |
| 165 | ay | alchemyPrice | 0 | 0 | ✓ |
| 166 | az | f_int_77 | 0 | 0 | ✓ |
| 167 | aA | f_int_78 | 0 | 0 | ✓ |
| 168 | aB | f_int_79 | 0 | 0 | ✓ |
| 169 | aC | f_int_80 | 0 | 0 | ✓ |
| 170 | f | f_String_arr_05 | 0 | 0 | ✓ |
| 171 | aD | f_int_81 | 0 | 0 | ✓ |
| 172 | l | f_byte_arr_11 | 0 | 0 | ✓ |
| 173 | f | f_bool_arr_05 | 0 | 0 | ✓ |
| 174 | aE | f_int_82 | 0 | 0 | ✓ |
| 175 | aF | f_int_83 | 0 | 0 | ✓ |
| 176 | aG | f_int_84 | 0 | 0 | ✓ |
| 177 | aH | f_int_85 | 0 | 0 | ✓ |
| 178 | b | f_short_arr_01 | 0 | 0 | ✓ |
| 179 | c | f_short_arr_02 | 0 | 0 | ✓ |
| 180 | d | f_short_arr_03 | 0 | 0 | ✓ |
| 181 | aI | f_int_86 | 0 | 0 | ✓ |
| 182 | g | f_String_arr_06 | 0 | 0 | ✓ |
| 183 | h | f_String_arr_07 | 0 | 0 | ✓ |
| 184 | m | f_byte_arr_12 | 0 | 0 | ✓ |
| 185 | i | f_String_arr_08 | 0 | 0 | ✓ |
| 186 | j | f_String_arr_09 | 0 | 0 | ✓ |
| 187 | n | f_byte_arr_13 | 0 | 0 | ✓ |
| 188 | aJ | f_int_87 | 0 | 0 | ✓ |
| 189 | aK | f_int_88 | 0 | 0 | ✓ |
| 190 | o | f_byte_arr_14 | 0 | 0 | ✓ |
| 191 | p | f_byte_arr_15 | 0 | 0 | ✓ |
| 192 | aL | f_int_89 | 0 | 0 | ✓ |
| 193 | aM | f_int_90 | 0 | 0 | ✓ |
| 194 | e | f_String_04 | 0 | 0 | ✓ |
| 195 | k | itemDescriptions | 0 | 0 | ✓ |
| 196 | l | equipDescriptions | 0 | 0 | ✓ |
| 197 | q | itemUseCounts | 0 | 0 | ✓ |
| 198 | aN | yellowKeyCount | 0 | 0 | ✓ |
| 199 | aO | blueKeyCount | 0 | 0 | ✓ |
| 200 | aP | redKeyCount | 0 | 0 | ✓ |
| 201 | aQ | goldAmount | 0 | 0 | ✓ |
| 202 | aR | itemStackSize | 0 | 0 | ✓ |
| 203 | r | itemStackTypes | 0 | 0 | ✓ |
| 204 | s | itemStackUses | 0 | 0 | ✓ |
| 205 | aS | f_int_96 | 0 | 0 | ✓ |
| 206 | o | equipTierBonuses | 0 | 0 | ✓ |
| 207 | t | equipTierTypes | 0 | 0 | ✓ |
| 208 | aT | f_int_97 | 0 | 0 | ✓ |
| 209 | aU | f_int_98 | 0 | 0 | ✓ |
| 210 | aV | f_int_99 | 0 | 0 | ✓ |
| 211 | aW | f_int_100 | 0 | 0 | ✓ |
| 212 | aX | f_int_101 | 0 | 0 | ✓ |
| 213 | aY | f_int_102 | 0 | 0 | ✓ |
| 214 | aZ | f_int_103 | 0 | 0 | ✓ |
| 215 | ba | f_int_104 | 0 | 0 | ✓ |
| 216 | m | f_bool_12 | 0 | 0 | ✓ |
| 217 | n | f_bool_13 | 0 | 0 | ✓ |
| 218 | m | objectTypeNames | 0 | 0 | ✓ |
| 219 | p | enemyBaseHp | 0 | 0 | ✓ |
| 220 | q | enemyBaseAtk | 0 | 0 | ✓ |
| 221 | r | enemyBaseDef | 0 | 0 | ✓ |
| 222 | s | enemyBaseGold | 0 | 0 | ✓ |
| 223 | t | enemyAtkScaled | 0 | 0 | ✓ |
| 224 | u | enemyDefScaled | 0 | 0 | ✓ |
| 225 | v | enemyHpScaled | 0 | 0 | ✓ |
| 226 | bb | f_int_105 | 0 | 0 | ✓ |
| 227 | bc | f_int_106 | 0 | 0 | ✓ |
| 228 | bd | f_int_107 | 0 | 0 | ✓ |
| 229 | be | f_int_108 | 0 | 0 | ✓ |
| 230 | bf | f_int_109 | 0 | 0 | ✓ |
| 231 | bg | f_int_110 | 0 | 0 | ✓ |
| 232 | w | f_int_arr_22 | 0 | 0 | ✓ |
| 233 | u | f_byte_arr_20 | 0 | 0 | ✓ |
| 234 | bh | currentScriptIndex | 0 | 0 | ✓ |
| 235 | n | levelScriptLines | 0 | 0 | ✓ |
| 236 | v | dialogueSpeakerType | 0 | 0 | ✓ |
| 237 | w | f_byte_arr_22 | 0 | 0 | ✓ |
| 238 | o | dialogueTexts | 0 | 0 | ✓ |
| 239 | bi | f_int_112 | 0 | 0 | ✓ |
| 240 | bj | f_int_113 | 0 | 0 | ✓ |
| 241 | g | f_bool_arr_06 | 0 | 0 | ✓ |
| 242 | bk | f_int_114 | 0 | 0 | ✓ |
| 243 | bl | f_int_115 | 0 | 0 | ✓ |
| 244 | q | f_byte_16 | 0 | 0 | ✓ |
| 245 | f | f_String_05 | 0 | 0 | ✓ |
| 246 | bm | scriptCursor | 0 | 0 | ✓ |
| 247 | bn | f_int_117 | 0 | 0 | ✓ |
| 248 | bo | f_int_118 | 0 | 0 | ✓ |
| 249 | r | f_byte_17 | 0 | 0 | ✓ |
| 250 | bp | f_int_119 | 0 | 0 | ✓ |
| 251 | bq | f_int_120 | 0 | 0 | ✓ |
| 252 | br | f_int_121 | 0 | 0 | ✓ |
| 253 | bs | f_int_122 | 0 | 0 | ✓ |
| 254 | bt | f_int_123 | 0 | 0 | ✓ |
| 255 | bu | f_int_124 | 0 | 0 | ✓ |
| 256 | bv | f_int_125 | 0 | 0 | ✓ |
| 257 | bw | f_int_126 | 0 | 0 | ✓ |
| 258 | bx | f_int_127 | 0 | 0 | ✓ |
| 259 | s | f_byte_18 | 0 | 0 | ✓ |
| 260 | t | f_byte_19 | 0 | 0 | ✓ |
| 261 | by | f_int_128 | 0 | 0 | ✓ |
| 262 | bz | f_int_129 | 0 | 0 | ✓ |
| 263 | bA | f_int_130 | 0 | 0 | ✓ |
| 264 | o | f_bool_14 | 0 | 0 | ✓ |
| 265 | e | f_short_arr_04 | 0 | 0 | ✓ |
| 266 | e | bossEventSpawns | 0 | 0 | ✓ |
| 267 | x | bossTypeOrder | 0 | 0 | ✓ |
| 268 | p | f_bool_15 | 0 | 0 | ✓ |
| 269 | h | f_bool_arr_07 | 0 | 0 | ✓ |
| 270 | y | f_byte_arr_24 | 0 | 0 | ✓ |
| 271 | x | f_int_arr_23 | 0 | 0 | ✓ |
| 272 | y | f_int_arr_24 | 0 | 0 | ✓ |
| 273 | z | f_int_arr_25 | 0 | 0 | ✓ |
| 274 | A | f_int_arr_26 | 0 | 0 | ✓ |
| 275 | B | f_int_arr_27 | 0 | 0 | ✓ |
| 276 | C | f_int_arr_28 | 0 | 0 | ✓ |
| 277 | D | f_int_arr_29 | 0 | 0 | ✓ |
| 278 | z | f_byte_arr_25 | 0 | 0 | ✓ |
| 279 | A | f_byte_arr_26 | 0 | 0 | ✓ |
| 280 | bB | f_int_131 | 0 | 0 | ✓ |
| 281 | bC | f_int_132 | 0 | 0 | ✓ |
| 282 | bD | f_int_133 | 0 | 0 | ✓ |
| 283 | bE | f_int_134 | 0 | 0 | ✓ |
| 284 | bF | f_int_135 | 0 | 0 | ✓ |
| 285 | bG | f_int_136 | 0 | 0 | ✓ |
| 286 | a | f_RecordStore_00 | 0 | 0 | ✓ |
| 287 | a | f_RecordEnumeration_00 | 0 | 0 | ✓ |
| 288 | a | f_ByteArrayOutputStream_00 | 0 | 0 | ✓ |
| 289 | a | f_DataOutputStream_00 | 0 | 0 | ✓ |
| 290 | a | f_DataInputStream_00 | 0 | 0 | ✓ |
| 291 | B | f_byte_arr_27 | 0 | 0 | ✓ |
| 292 | f | f_byte_arr2_05 | 0 | 0 | ✓ |
| 293 | bH | f_int_137 | 0 | 0 | ✓ |
| 294 | C | f_byte_arr_28 | 0 | 0 | ✓ |
| 295 | bI | f_int_138 | 0 | 0 | ✓ |
| 296 | D | f_byte_arr_29 | 0 | 0 | ✓ |
| 297 | f | f_short_arr_05 | 0 | 0 | ✓ |
| 298 | g | f_short_arr_06 | 0 | 0 | ✓ |
| 299 | h | f_short_arr_07 | 0 | 0 | ✓ |
| 300 | i | f_short_arr_08 | 0 | 0 | ✓ |
| 301 | u | f_byte_20 | 0 | 0 | ✓ |
| 302 | v | f_byte_21 | 0 | 0 | ✓ |
| 303 | E | f_byte_arr_30 | 0 | 0 | ✓ |
| 304 | E | f_int_arr_30 | 0 | 0 | ✓ |
| 305 | j | f_short_arr_09 | 0 | 0 | ✓ |
| 306 | k | f_short_arr_10 | 0 | 0 | ✓ |
| 307 | F | f_byte_arr_31 | 0 | 0 | ✓ |
| 308 | bJ | f_int_139 | 0 | 0 | ✓ |
| 309 | bK | f_int_140 | 0 | 0 | ✓ |
| 310 | i | f_bool_arr_08 | 0 | 0 | ✓ |
| 311 | G | f_byte_arr_32 | 0 | 0 | ✓ |
| 312 | H | f_byte_arr_33 | 0 | 0 | ✓ |
| 313 | I | f_byte_arr_34 | 0 | 0 | ✓ |
| 314 | J | f_byte_arr_35 | 0 | 0 | ✓ |
| 315 | K | f_byte_arr_36 | 0 | 0 | ✓ |
| 316 | L | f_byte_arr_37 | 0 | 0 | ✓ |
| 317 | l | f_short_arr_11 | 0 | 0 | ✓ |
| 318 | m | f_short_arr_12 | 0 | 0 | ✓ |
| 319 | M | f_byte_arr_38 | 0 | 0 | ✓ |
| 320 | N | f_byte_arr_39 | 0 | 0 | ✓ |
| 321 | O | f_byte_arr_40 | 0 | 0 | ✓ |
| 322 | w | f_byte_22 | 0 | 0 | ✓ |
| 323 | F | f_int_arr_31 | 0 | 0 | ✓ |
| 324 | j | f_bool_arr_09 | 0 | 0 | ✓ |
| 325 | k | f_bool_arr_10 | 0 | 0 | ✓ |
| 326 | bL | f_int_141 | 0 | 0 | ✓ |
| 327 | q | f_bool_16 | 0 | 0 | ✓ |
| 328 | r | f_bool_17 | 0 | 0 | ✓ |
| 329 | bM | f_int_142 | 0 | 0 | ✓ |
| 330 | x | f_byte_23 | 0 | 0 | ✓ |
| 331 | s | f_bool_18 | 0 | 0 | ✓ |
| 332 | P | f_byte_arr_41 | 0 | 0 | ✓ |
| 333 | bN | f_int_143 | 0 | 0 | ✓ |
| 334 | bO | f_int_144 | 0 | 0 | ✓ |
| 335 | bP | f_int_145 | 0 | 0 | ✓ |
| 336 | y | f_byte_24 | 0 | 0 | ✓ |
| 337 | t | f_bool_19 | 0 | 0 | ✓ |
| 338 | bQ | f_int_146 | 0 | 0 | ✓ |
| 339 | bR | f_int_147 | 0 | 0 | ✓ |
| 340 | G | f_int_arr_32 | 0 | 0 | ✓ |
| 341 | z | f_byte_25 | 0 | 0 | ✓ |
| 342 | Q | f_byte_arr_42 | 0 | 0 | ✓ |
| 343 | bS | f_int_148 | 0 | 0 | ✓ |
| 344 | g | f_byte_arr2_06 | 0 | 0 | ✓ |
| 345 | a | f_short_arr2_00 | 0 | 0 | ✓ |
| 346 | a | f_short_00 | 0 | 0 | ✓ |
| 347 | b | f_short_01 | 0 | 0 | ✓ |
| 348 | c | f_short_02 | 0 | 0 | ✓ |
| 349 | d | f_short_03 | 0 | 0 | ✓ |
| 350 | n | f_short_arr_13 | 0 | 0 | ✓ |
| 351 | o | f_short_arr_14 | 0 | 0 | ✓ |
| 352 | bT | f_int_149 | 0 | 0 | ✓ |
| 353 | u | f_bool_20 | 0 | 0 | ✓ |
| 354 | v | f_bool_21 | 0 | 0 | ✓ |
| 355 | w | f_bool_22 | 0 | 0 | ✓ |
| 356 | A | f_byte_26 | 0 | 0 | ✓ |
| 357 | H | difficultyMultipliers | 0 | 0 | ✓ |
| 358 | bU | f_int_150 | 0 | 0 | ✓ |
| 359 | bV | f_int_151 | 0 | 0 | ✓ |
| 360 | x | f_bool_23 | 0 | 0 | ✓ |
| 361 | B | f_byte_27 | 0 | 0 | ✓ |
| 362 | y | f_bool_24 | 0 | 0 | ✓ |
| 363 | z | f_bool_25 | 0 | 0 | ✓ |
| 364 | R | f_byte_arr_43 | 0 | 0 | ✓ |
| 365 | S | f_byte_arr_44 | 0 | 0 | ✓ |
| 366 | A | f_bool_26 | 0 | 0 | ✓ |
| 367 | B | f_bool_27 | 0 | 0 | ✓ |
| 368 | I | f_int_arr_34 | 0 | 0 | ✓ |
| 369 | J | f_int_arr_35 | 0 | 0 | ✓ |
| 370 | bW | f_int_152 | 0 | 0 | ✓ |
| 371 | bX | f_int_153 | 0 | 0 | ✓ |
| 372 | bY | f_int_154 | 0 | 0 | ✓ |
| 373 | C | f_bool_28 | 0 | 0 | ✓ |
| 374 | p | f_String_arr_15 | 0 | 0 | ✓ |
| 375 | bZ | f_int_155 | 0 | 0 | ✓ |
| 376 | D | f_bool_29 | 0 | 0 | ✓ |
| 377 | C | f_byte_28 | 0 | 0 | ✓ |
| 378 | a | f_Player_00 | 0 | 0 | ✓ |
| 379 | a | gameRandom | 0 | 0 | ✓ |
| 380 | K | f_int_arr_36 | 0 | 0 | ✓ |
| 381 | ca | f_int_156 | 0 | 0 | ✓ |
| 382 | T | f_byte_arr_45 | 0 | 0 | ✓ |
| 383 | cb | f_int_157 | 0 | 0 | ✓ |
| 384 | cc | f_int_158 | 0 | 0 | ✓ |
| 385 | L | f_int_arr_37 | 0 | 0 | ✓ |
| 386 | E | f_bool_30 | 0 | 0 | ✓ |
| 387 | U | f_byte_arr_46 | 0 | 0 | ✓ |
| 388 | D | f_byte_29 | 0 | 0 | ✓ |
| 389 | d | f_int_arr2_03 | 0 | 0 | ✓ |
| 390 | b | f_Image_arr_01 | 0 | 0 | ✓ |
| 391 | q | f_String_arr_16 | 0 | 0 | ✓ |
| 392 | r | f_String_arr_17 | 0 | 0 | ✓ |
| 393 | g | f_String_06 | 0 | 0 | ✓ |
| 394 | cd | f_int_159 | 0 | 0 | ✓ |
| 395 | V | f_byte_arr_47 | 0 | 0 | ✓ |
| 396 | ce | f_int_160 | 0 | 0 | ✓ |
| 397 | a | f_Object_00 | 0 | 0 | ✓ |
| 398 | cf | f_int_161 | 0 | 0 | ✓ |
| 399 | F | f_bool_31 | 0 | 0 | ✓ |
| 400 | G | f_bool_32 | 0 | 0 | ✓ |
| 401 | h | f_String_07 | 0 | 0 | ✓ |
| 402 | cg | f_int_162 | 0 | 0 | ✓ |
| 403 | a | f_long_00 | 0 | 0 | ✓ |
| 404 | b | f_long_01 | 0 | 0 | ✓ |
| 405 | ch | f_int_163 | 0 | 0 | ✓ |
| 406 | ci | f_int_164 | 0 | 0 | ✓ |
| 407 | a | f_HttpConnection_00 | 0 | 0 | ✓ |
| 408 | H | f_bool_33 | 0 | 0 | ✓ |
| 409 | W | f_byte_arr_48 | 0 | 0 | ✓ |
| 410 | cj | f_int_165 | 0 | 0 | ✓ |
| 411 | ck | f_int_166 | 0 | 0 | ✓ |
| 412 | a | a | 10277 | 10277 | ✓ |
| 413 | paint | paint | 3870 | 3870 | ✓ |
| 414 | run | run | 2443 | 2443 | ✓ |
| 415 | a | m_000 | 534 | 534 | ✓ |
| 416 | showNotify | showNotify | 10 | 10 | ✓ |
| 417 | hideNotify | hideNotify | 24 | 24 | ✓ |
| 418 | a | m_001 | 111 | 111 | ✓ |
| 419 | a | m_002 | 26 | 26 | ✓ |
| 420 | a | m_003 | 21 | 21 | ✓ |
| 421 | a | m_004 | 11 | 11 | ✓ |
| 422 | a | m_005 | 180 | 180 | ✓ |
| 423 | a | m_006 | 24 | 24 | ✓ |
| 424 | a | m_007 | 59 | 59 | ✓ |
| 425 | b | m_008 | 34 | 34 | ✓ |
| 426 | keyPressed | keyPressed | 32 | 32 | ✓ |
| 427 | keyReleased | keyReleased | 4 | 4 | ✓ |
| 428 | c | m_009 | 42 | 42 | ✓ |
| 429 | b | m_010 | 60 | 60 | ✓ |
| 430 | d | m_011 | 64 | 64 | ✓ |
| 431 | e | m_012 | 108 | 108 | ✓ |
| 432 | b | m_013 | 74 | 74 | ✓ |
| 433 | f | m_014 | 240 | 240 | ✓ |
| 434 | a | m_015 | 79 | 79 | ✓ |
| 435 | g | m_016 | 25 | 25 | ✓ |
| 436 | a | m_017 | 8 | 8 | ✓ |
| 437 | a | m_018 | 221 | 221 | ✓ |
| 438 | a | m_019 | 315 | 315 | ✓ |
| 439 | a | m_020 | 47 | 47 | ✓ |
| 440 | a | m_021 | 44 | 44 | ✓ |
| 441 | a | m_022 | 14 | 14 | ✓ |
| 442 | c | applyHpDelta | 49 | 49 | ✓ |
| 443 | a | m_024 | 20 | 20 | ✓ |
| 444 | h | m_025 | 19 | 19 | ✓ |
| 445 | i | m_026 | 229 | 229 | ✓ |
| 446 | j | m_027 | 177 | 177 | ✓ |
| 447 | a | tryStep | 54 | 54 | ✓ |
| 448 | a | interactWithCell | 496 | 496 | ✓ |
| 449 | k | applyStepCellEffects | 811 | 811 | ✓ |
| 450 | b | m_031 | 60 | 60 | ✓ |
| 451 | l | m_032 | 43 | 43 | ✓ |
| 452 | c | m_033 | 715 | 715 | ✓ |
| 453 | m | m_034 | 68 | 68 | ✓ |
| 454 | d | m_035 | 140 | 140 | ✓ |
| 455 | a | m_036 | 45 | 45 | ✓ |
| 456 | e | m_037 | 349 | 349 | ✓ |
| 457 | a | m_038 | 69 | 69 | ✓ |
| 458 | b | m_039 | 163 | 163 | ✓ |
| 459 | c | m_040 | 224 | 224 | ✓ |
| 460 | d | m_041 | 67 | 67 | ✓ |
| 461 | a | m_042 | 93 | 93 | ✓ |
| 462 | n | m_043 | 224 | 224 | ✓ |
| 463 | f | m_044 | 67 | 67 | ✓ |
| 464 | g | m_045 | 14 | 14 | ✓ |
| 465 | d | m_046 | 42 | 42 | ✓ |
| 466 | e | m_047 | 113 | 113 | ✓ |
| 467 | a | m_048 | 175 | 175 | ✓ |
| 468 | a | m_049 | 72 | 72 | ✓ |
| 469 | b | m_050 | 61 | 61 | ✓ |
| 470 | f | m_051 | 79 | 79 | ✓ |
| 471 | o | m_052 | 29 | 29 | ✓ |
| 472 | a | m_053 | 1299 | 1299 | ✓ |
| 473 | p | m_054 | 373 | 373 | ✓ |
| 474 | q | m_055 | 92 | 92 | ✓ |
| 475 | a | m_056 | 80 | 80 | ✓ |
| 476 | r | m_057 | 290 | 290 | ✓ |
| 477 | a | m_058 | 13 | 13 | ✓ |
| 478 | s | m_059 | 20 | 20 | ✓ |
| 479 | b | isCellWalkable | 21 | 21 | ✓ |
| 480 | t | m_061 | 60 | 60 | ✓ |
| 481 | c | m_062 | 28 | 28 | ✓ |
| 482 | h | m_063 | 210 | 210 | ✓ |
| 483 | i | m_064 | 201 | 201 | ✓ |
| 484 | a | m_065 | 55 | 55 | ✓ |
| 485 | a | changeFloor | 114 | 114 | ✓ |
| 486 | u | m_067 | 35 | 35 | ✓ |
| 487 | v | m_068 | 88 | 88 | ✓ |
| 488 | g | scaleEnemyStats | 82 | 82 | ✓ |
| 489 | a | scaledByFloorTier | 10 | 10 | ✓ |
| 490 | b | alchemyPriceFor | 17 | 17 | ✓ |
| 491 | h | m_072 | 62 | 62 | ✓ |
| 492 | b | m_073 | 78 | 78 | ✓ |
| 493 | a | applyMerchantOffer | 441 | 441 | ✓ |
| 494 | j | m_075 | 24 | 24 | ✓ |
| 495 | i | m_076 | 89 | 89 | ✓ |
| 496 | c | spendGold | 17 | 17 | ✓ |
| 497 | c | gainGold | 15 | 15 | ✓ |
| 498 | j | pickupItemType | 197 | 197 | ✓ |
| 499 | a | floorTier | 23 | 23 | ✓ |
| 500 | c | m_081 | 21 | 21 | ✓ |
| 501 | a | m_082 | 88 | 88 | ✓ |
| 502 | a | m_083 | 16 | 16 | ✓ |
| 503 | d | itemTypeToStackIndex | 18 | 18 | ✓ |
| 504 | k | addItemToItemStack | 56 | 56 | ✓ |
| 505 | b | consumeKeyForDoor | 68 | 68 | ✓ |
| 506 | l | useItemStack | 75 | 75 | ✓ |
| 507 | c | activateItem | 300 | 300 | ✓ |
| 508 | d | m_089 | 74 | 74 | ✓ |
| 509 | k | killAdjacentAt | 112 | 112 | ✓ |
| 510 | w | m_091 | 241 | 241 | ✓ |
| 511 | m | m_092 | 35 | 35 | ✓ |
| 512 | a | m_093 | 16 | 16 | ✓ |
| 513 | d | loadLevelScript | 71 | 71 | ✓ |
| 514 | b | tryRunScene | 441 | 441 | ✓ |
| 515 | n | m_096 | 107 | 107 | ✓ |
| 516 | a | m_097 | 14 | 14 | ✓ |
| 517 | a | executeScriptInstruction | 1347 | 1347 | ✓ |
| 518 | a | parseScriptInt | 20 | 20 | ✓ |
| 519 | b | m_100 | 48 | 48 | ✓ |
| 520 | d | m_101 | 37 | 37 | ✓ |
| 521 | d | m_102 | 47 | 47 | ✓ |
| 522 | l | m_103 | 33 | 33 | ✓ |
| 523 | e | m_104 | 82 | 82 | ✓ |
| 524 | x | m_105 | 135 | 135 | ✓ |
| 525 | y | m_106 | 457 | 457 | ✓ |
| 526 | e | m_107 | 42 | 42 | ✓ |
| 527 | f | m_108 | 122 | 122 | ✓ |
| 528 | o | spawnBossEvent | 46 | 46 | ✓ |
| 529 | z | m_110 | 106 | 106 | ✓ |
| 530 | A | m_111 | 193 | 193 | ✓ |
| 531 | p | m_112 | 212 | 212 | ✓ |
| 532 | b | m_113 | 707 | 707 | ✓ |
| 533 | q | m_114 | 256 | 256 | ✓ |
| 534 | e | m_115 | 300 | 300 | ✓ |
| 535 | a | m_116 | 5 | 5 | ✓ |
| 536 | B | m_117 | 47 | 47 | ✓ |
| 537 | C | m_118 | 12 | 12 | ✓ |
| 538 | r | m_119 | 125 | 125 | ✓ |
| 539 | a | m_120 | 27 | 27 | ✓ |
| 540 | s | m_121 | 180 | 180 | ✓ |
| 541 | t | m_122 | 717 | 717 | ✓ |
| 542 | b | m_123 | 28 | 28 | ✓ |
| 543 | m | m_124 | 24 | 24 | ✓ |
| 544 | a | m_125 | 139 | 139 | ✓ |
| 545 | D | m_126 | 275 | 275 | ✓ |
| 546 | b | m_127 | 96 | 96 | ✓ |
| 547 | E | m_128 | 36 | 36 | ✓ |
| 548 | e | m_129 | 62 | 62 | ✓ |
| 549 | F | m_130 | 125 | 125 | ✓ |
| 550 | a | m_131 | 26 | 26 | ✓ |
| 551 | u | m_132 | 12 | 12 | ✓ |
| 552 | a | m_133 | 247 | 247 | ✓ |
| 553 | g | m_134 | 125 | 125 | ✓ |
| 554 | a | predictHpLossVsType | 23 | 23 | ✓ |
| 555 | f | effectiveAttackVsType | 33 | 33 | ✓ |
| 556 | a | predictBattleHpLoss | 51 | 51 | ✓ |
| 557 | c | tickBattle | 382 | 382 | ✓ |
| 558 | a | m_139 | 120 | 120 | ✓ |
| 559 | G | m_140 | 10 | 10 | ✓ |
| 560 | g | randomBelow | 8 | 8 | ✓ |
| 561 | a | m_142 | 101 | 101 | ✓ |
| 562 | n | m_143 | 1059 | 1059 | ✓ |
| 563 | H | m_144 | 168 | 168 | ✓ |
| 564 | a | m_145 | 185 | 185 | ✓ |
| 565 | a | m_146 | 83 | 83 | ✓ |
| 566 | v | m_147 | 60 | 60 | ✓ |
| 567 | I | m_148 | 35 | 35 | ✓ |
| 568 | c | m_149 | 28 | 28 | ✓ |
| 569 | d | m_150 | 10 | 10 | ✓ |
| 570 | e | m_151 | 346 | 346 | ✓ |
| 571 | f | m_152 | 22 | 22 | ✓ |
| 572 | b | m_153 | 271 | 271 | ✓ |
| 573 | h | m_154 | 37 | 37 | ✓ |
| 574 | {} | {} | 69 | 69 | ✓ |

## 保留原名的成员（old == new，共 12 个）

| class | kind | 名称 | signature | 保留原因 |
|---|---|---|---|---|
| CMidlet | method | <init> | `()V` | 构造器/静态初始化 |
| CMidlet | method | startApp | `()V` | MIDP API override/实现（改名会改变虚分派语义） |
| CMidlet | method | pauseApp | `()V` | MIDP API override/实现（改名会改变虚分派语义） |
| CMidlet | method | destroyApp | `(Z)V` | MIDP API override/实现（改名会改变虚分派语义） |
| a | method | <init> | `()V` | 构造器/静态初始化 |
| a | method | paint | `(Ljavax/microedition/lcdui/Graphics;)V` | MIDP API override/实现（改名会改变虚分派语义） |
| a | method | run | `()V` | MIDP API override/实现（改名会改变虚分派语义） |
| a | method | showNotify | `()V` | MIDP API override/实现（改名会改变虚分派语义） |
| a | method | hideNotify | `()V` | MIDP API override/实现（改名会改变虚分派语义） |
| a | method | keyPressed | `(I)V` | MIDP API override/实现（改名会改变虚分派语义） |
| a | method | keyReleased | `(I)V` | MIDP API override/实现（改名会改变虚分派语义） |
| a | method | <clinit> | `()V` | 构造器/静态初始化 |

## 产物自检

- 字段/方法声明扫描（javap -p + CFR 源码双路）：混淆短名（`^[a-zA-Z]{1,2}$`）残留 **0 个**；类内字段+方法名全局唯一。
- CFR 局部变量名说明：CFR 输出中存在单字母局部变量名 `c`×3、`i`×2、`l`×1、`n`×39、`s`×7（共 52 处）。字节码无 LocalVariableTable，局部变量名是 CFR 按类型启发式生成的**投影层命名**（方法作用域内唯一、无歧义），不属于符号重映射范围；本目录 Java 保持 CFR 原样输出以便与 `bash tools/decompile.sh` 的再生产流程同构。

## deobf 中文可读化后处理（t12：字面量 \uXXXX → UTF-8）

对 CFR 产物 `reference/src/deobf/*.java` 做确定性后处理：**字符串/字符字面量内**码点 ≥ 0x00A0 的 `\uXXXX` 解码为 UTF-8 明文（含全部中文）；ASCII 范围转义（< 0x00A0，如 `\u0022`/`\u005C`/`\u0000`）一律保留以免词法歧义。只改源文本呈现 —— **回环硬校验** `encode(decode(原文)) == 原文` 逐字节成立（构建时校验，失败拒绝落盘），即运行期字符串值零变化；注释/标识符不动，解码产出字符永不为引号/反斜杠 ⇒ 词法结构零变化。`reference/seed/` 保持 CFR 转义原样（D3 锚点，一字节不动，reference-seed-integrity 锁定）。

| 文件 | 字面量数 | 解码字符 | 保留转义 | 回环 encode(decode)==CFR 原文 |
|---|---|---|---|---|
| CMidlet.java | 0 | 0 | 0 | ✓ |
| a.java | 818 | 8880 | 180 | ✓ |

中文字符串 × 常量池 UTF-8 对照（抽样 3 处，解码值逐字命中 class 常量池 Utf8 —— class 文件存明文 UTF-8，故此为「解码值不变」的另一路独立证据）：

| 来源 | 解码后字面量 | 常量池 UTF-8 命中 |
|---|---|---|
| a.java | `(内部版本` | ✓ |
| a.java | `无法再下一层了，这是你达到的最底层` | ✓ |
| a.java | `无法再上一层了，这是你达到的最高层` | ✓ |

## 备注

- 未改名成员（构造器 / `<clinit>` / `paint` `run` `keyPressed` 等 MIDP API override）在映射表中 `renamed=false`、old == new：override 不得改名，否则虚分派语义改变。
- `data/naming/ledger.jsonl` 为语义命名台账：`status=applied` 的记录叠加在机械映射之上（见上「语义叠加」行），`status=hypothesis` 一律不动代码；`remap-table.json` 保持纯机械层。
- 本报告不含时间戳：同一 JAR 重跑输出逐字节一致（见 `_verify/manifest.json`）。
