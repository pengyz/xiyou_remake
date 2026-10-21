# tools/table-evidence — 机械对照器（R2 金样验收闸·生成式侧）

> survey §4 R2 三件之一：**机械对照器**（javap/deobf 提取值做第二侧）防手抄互证
> 同源错误。本目录产出/守护 `data/tables/{sprite_defs,anim_defs,sheet_index}.json`。

## 组件（①提取器 ②生成器 ③比对器 ④--check）

| 文件 | 职责 |
|---|---|
| `extractor.py` | ① 从 `reference/src/deobf/a.java` **机械**提取表值+逐值行号锚（第二侧） |
| `generator.py` | ② 生成式回填三表：三键三元组 `{index, value:{事实名:值}, src:"a.java:line"}` |
| `comparator.py` | ③ 表值==提取值逐值对账，**错位逐行报**（独立推导，不 import generator） |
| `cli.py` | ④ `check`/`--check` = 等值比对 + **确定性复跑同哈希**（复跑1/复跑2/盘上三方一致） |
| `tests/test_equivalence.py` | 等值单测（stdlib unittest；本机 pytest 断言改写损坏弃用） |

## 用法

```bash
python3 tools/table-evidence/cli.py extract     # 提取摘要（第二侧）
python3 tools/table-evidence/cli.py generate    # 生成式回填 data/tables/*.json
python3 tools/table-evidence/cli.py compare     # 等值比对（错位逐行报）
python3 tools/table-evidence/cli.py check       # compare + 确定性复跑同哈希
python3 -m unittest discover -s tools/table-evidence/tests -v   # 等值单测
```

## schema（2026-10-21 与 crates/game-data/src/defs.rs 契约合流）

- `schema_version` = **2**（int = 解析器 `SCHEMA_VERSION`，§2.3）；
- 三元组 = **三键** `{index, value, src}`，value 结构化（事实名→值）——89/88/11/16/16
  逐值锚的 `src` 一律取**值所在行**；
- `density` 字段（记录级，默认 1，HD 通路预留）+ `quirks_in_code` 留痕（Q-001..Q-005）。

| 文件 | 封套键 | 要点 |
|---|---|---|
| sprite_defs.json | schema_version / sprite_defs(89) / transition_overrides / size_dispatch / quirks_in_code | 型→精灵属性；anim_row+type_frame_width=89/88 表宿主 |
| anim_defs.json | schema_version / anim_defs(11) | 帧序 11 行；step_ticks=2、loop_mode=cycle（全局节拍 Q-005） |
| sheet_index.json | schema_version / containers(16 字符串) / container_counts / slots(对象) / assembly_profiles / evidence(44) | SEG67 表中表 9 片件+3×5 段帧 |

transition_overrides 事件条件规则形（全局，按 render_category 分派）：
`mark_removed`(state=1，类目 1 内层 by_type 1|2|3/11 + build_minimap) /
`mark_des_removed`(state=2，2|8|16|32→行8) / `bump_reveal`(型6 显形行9、型12 变形)。
派发尺寸 `size_dispatch`：default=typeFrameWidth+图高 / fallback 32×32 / 67、69、72 特例。

## 保真表（逐值锚 220 = 89+88+11+16+16）

| 表 | 项数 | 源 | 宿主 |
|---|---|---|---|
| animRowByType | 89（型 0..88） | a.java:499-587 | `sprite_defs[].anim_row` + evidence |
| typeFrameWidth | 88（型 0..87） | a.java:590-677 | `sprite_defs[].type_frame_width` + evidence |
| ANIM_OFFSET_TABLE | 11 行 | a.java:486-496 | `anim_defs[].frames` + evidence |
| resourceNames | 16 | a.java:451（声明 450） | `sheet_index.containers` + evidence |
| resourceImageCounts | 16 | a.java:453 | `sheet_index.container_counts` + evidence |

迁入源核对：`crates/game-core/tests/fixtures/type_width_px.txt` == typeFrameWidth 提取值
（88 项，测试 `test_fixture_is_migration_source`）。

## 勘误（2026-10-21 机械实锤）

> 【更正】survey §0.3/§4 与任务卡记「容器名序=450、451=张数表」——机械实测：
> 名序**声明**=450、名序**值行**=451、张数表**赋值行=453**。三元组 src 一律取
> 值所在行（name→451，count→453）。同理 animRowByType 值域=499-587（任务卡
> 「498-586」为含声明行的近似值域）。
> RED 实证手抄风险：等值单测首轮即抓到一处手读字面错位（idx73/74 行 572/573）。

## 解析器 delta 清单（交 task-6 扩展严格键集，2026-10-21）

表文件=接口契约（plan §1 Phase 1）。真实数据压出的必改项：

1. `SPRITE_KEYS += ["anim_row", "type_frame_width"]`（89/88 保真表宿主，int）；
2. **可空**：`sheet`/`sheet_index`（型 0/82/83/88 无图 a.java:9455-9491）、`frame_h`
   （82 型=运行时图高 a.java:6318）、`type_frame_width`（型 88 无表项）；
   `frame_w` 恒 int（型 88 走 fallback 32×32 a.java:6322-6324）；
3. `ENVELOPE_SPRITE_KEYS += ["transition_overrides", "size_dispatch"]`（全局规则形
   与尺寸派发——按 render_category/型 分派的事件条件规则不能装进单型记录）；
4. `ENVELOPE_SHEET_KEYS += ["assembly_profiles"]`（SEG67 表中表 9 片件+3×5 段帧，
   型 67 以 `assembly_profile: "seg67"` 外键引用）；
5. 规则形扩展：`TRANSITION_KEYS += ["state","otherwise","by_type"]`；by_category 值键
   += `sprite_w/anim_frame/by_type/otherwise/build_minimap/noop/returns`
   （真臂含 类目1→型1|2|3/11 内层分派，survey 示例扁平形装不下——R3 实锤）；
6. `ANCHOR_KEYS=[index,value,src]` 已合流（value 自由形=事实名→值）——无需改动。

## 判据

- 等值单测全绿：表值 == 机械提取值（89+88+11+16+16 逐值 + 规则形逐臂）；
- `cli.py check`：确定性复跑同哈希（复跑1==复跑2==盘上）；
- 迁入源核对：type_width_px.txt == typeFrameWidth 提取值。
