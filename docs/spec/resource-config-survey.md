# 资源参数化调研 v2：动画/精灵/图集配置系统（修订版）

> 【2026-10-17 v2 修订】三路对抗复核 15 条件全吸收：复核 1/3（items/schema 面，结论经
> lead 转述）、复核 2/3（保真红线，resource-config-review-fidelity.md R1-R4）、
> 复核 3/3（工程排期，resource-config-review.md ①-⑤）。v1 主张保留+【v2 更正】原地标注。
> 纯调研不动代码。口径体例沿 test-count-taxonomy.md（计数首现标口径）。
>
> v2 修订记录：① schema 主件修正（override 改事件条件规则形 transition_overrides+
> assembly_profile+resourceImageCounts+quirks_in_code 留痕）② 锚定修正（逐值锚三元组
> 生成式+450/451 错位实锤修正+type_width_px evidence 补录）③ 金样验收闸（机械对照器进
> gates+帧夹具重生成红线+保真表哈希闸）④ 两份制四规则写死+blocked-by v2 三度悬空显式化+
> 一事实一宿主硬条件+sprite_ref 回写 ⑤ 零改码链降格+域外三界 ⑥ 外键政策成文+加载层失败
> 语义 ⑦ 工时修正+编辑点合并账本+合批主张改口 ⑧ 口径体例引用。

## 0. 结论速览（v2）

1. **动画面已半表化**（沿 v1）：`ANIM_TABLE_IDX`（animRowByType 89 项，a.java:498-586）+
   `ANIM_OFFSET_TABLE`（帧序 11 行，a.java:485-497）+ `SEG67_PIECES/SEG67_FRAMES` 先例。
2. **【v2 更正 R3】真硬编码含"状态条件臂"**：advance_frames 特例行不是静态 per-type 覆写
   （v1 的 `anim_row_override` 表达力不够）——实测 entity.rs:393-425 `state=1` 路径**按
   render_category 分派**（`1|2|3→{row6,h56}`、`2|4→{row10}`、`8|16|32→{row6}`）+
   remove_des `state=2` 族（a.java:6332-6347→row8）——**事件条件规则形**才能装下
   （schema 修正见 §2.1）。
3. **【v2 更正 R1】锚定粒度修正**：evidence 表级范围锚不达 data/naming 体例——改
   **逐值锚三元组** `{index, value, a.java:line}` 生成式；**锚错位实锤**：v1 schema 写
   `"a.java:451 (container order)"`——**容器名序=450、451=张数表**（resourceImageCounts
   `{8,1,12,4,13,23,10,20,25,6,2,2,12,20,1,2}`）——自错一行=粒度不足症状，v2 修正。
4. **【v2 新增 R2】金样验收闸三件**：机械对照器（javap/deobf 提取值做第二侧）进 gates
   （防手抄互证同源错误）+ 帧夹具重生成红线（表勘误红必须=表错，期望仅源自 Java 参照帧
   ——与 viewport V3 同款洞两案同修）+ 保真表内容哈希闸（非荣誉制）。
5. **【v2 降格】零改码链**：v1"全链零 Rust"→ **"同图集复用、消费替换完成后域内零改码"**
   +步 5 演练实跑为验收闸；**域外三界写明**（ANIM_TABLE_IDX 89 索引 panic/图集容器常量
   16 个/战斗表数组索引）。
6. **【v2 修正 3/3】合批主张改口**：v1"合批省时 13-15 天"被自家数字证伪（合批 > 两线分计
   之和）→ **"多花 2-4 天买基建一次成型"或分批**；按子面拆 commit 每 commit 一闸
   （爆炸半径史实反例）。

---

## 1. 硬编码普查（v2 沿用+两处修正）

四类实测沿 v1（动画面/精灵尺寸/图集索引/渲染分类；真硬编码 ≈35-40 逻辑编辑点[文件级口径]）。
**【v2 修正 3/3①】编辑点合并账本**：35-40 与表设计 v2 的编辑点**有重叠**（dispatch_sprite_size
特例/render_category/floor 特例两案都点名）——合并账本见 §5（与 v2 工时归属去重，
禁双计）。

**【v2 补 R1】schema 漏收已清点数据三件**：
① `resourceImageCounts`（a.java:451，16 容器张数表）——v1 schema 无张数 ⇒ 槽位合法性/
图集裁切边界无所依（ui[8][*]=25 张的裁切循环边界靠什么定？）；
② `SEG67_PIECES/SEG67_FRAMES`（表中表：9 片件 xywh + 3×5 段帧）——v1 override 字段装不下
片件数组 ⇒ 补 `assembly_profile` 字段（§2.1）；
③ type_width_px.txt 迁入 sprite_defs 时 values 的 **evidence 补录义务**（迁移即建档）。

**【v2 补 R3】显形行 9 写点**（spawn 特例 anim_row=9 路径）与状态迁移臂同批转译。

---

## 2. 配置系统方案 v2

### 2.1 schema 主件修正（1/3+2/3 R3 合流）

```jsonc
// sprite_defs.json（型→精灵属性）
{ "type_id": 72,
  "sheet": "sptenemy1", "sheet_index": 3,
  "frame_w": 32, "frame_h": 45,
  "render_category": 32,
  "anim_profile": "walk_3frame",                 // 外键 → anim_defs
  "assembly_profile": null,                       // ★R1 补：SEG67 片件表引用（表中表）
  "insert_block": false,
  "floor_category_override": [{ "floor": 50, "category": 8 }],
  // ★R3 修正：事件条件规则形（状态迁移驱动特例——静态 override 装不下）
  "transition_overrides": [
    { "on": "spawn_state1",                       // entity.rs:393-425 state=1 路径
      "by_category": { "1": {"anim_row":6,"sprite_h":56}, "2": {"anim_row":10,"sprite_h":56},
                       "8": {"anim_row":6}, "4": {"anim_row":2} },
      "evidence": "a.java:6332-6347 (markEntityDesRemoved 同族); entity.rs:393-425" },
    { "on": "mark_removed", "by_category": {...} },
    { "on": "mark_des_removed", "by_category": { "*": {"anim_row":8} },
      "evidence": "a.java:6332-6347" }
  ],
  "evidence": [ {"index": 72, "value": {"row": 3}, "src": "a.java:498-586"} ]   // ★R1 逐值锚
}

// anim_defs.json
{ "id": "walk_3frame", "frames": [0,1,2,1], "step_ticks": 4, "loop_mode": "cycle",
  "evidence": [ {"index": 3, "value": [0,1,2,1], "src": "a.java:485-497"} ] }

// sheet_index.json
{ "containers": [...],                            // ★R1 锚修正：a.java:450（非 451）
  "container_counts": [8,1,12,4,13,23,10,20,25,6,2,2,12,20,1,2],   // ★R1 补：a.java:451 张数表
  "slots": {...},
  "evidence": [ {"index": 8, "value": {"name":"ui","count":25}, "src": "a.java:450-451"} ] }
```

**quirks_in_code 留痕**：装不下规则形的残余特例（若有）显式登记 `{quirk_id, reason,
a.java 锚, 状态迁移三拍 RED 测 id}`——**禁静默留码**；消费替换顺序契约写死：
**查表初始化（dispatchAnimFields a.java:6327-6330）→ 状态迁移覆写（transition_overrides）**
（顺序反了 quirk 静默失效——R3 反例 2）。

> 【2026-10-21 注·HD 探针 R-5 回写】量产批本 schema 预留 **`density` 字段**（素材像素尺寸 =
> 逻辑尺寸×density，A 档决策=同逻辑尺寸+高像素密度）；运行时载体已裁定 = `ArgbImage`
> 字段（默认 1）+ `logical_width()/logical_height()` 访问器 + `width%d==0` 断言（渲染层
> 已落地 e4bdbd2）。**探针期零持久化 schema**（demo 图集不进任何配置表）——本字段注记
> 只锁未来归属，不改 v2 表设计。详见
> [hd-asset-probe-design.md](../superpowers/specs/2026-10-21-hd-asset-probe-design.md) v1.1 R-5。

### 2.2 两份制四规则写死（2/3 R4）

1. **mod 禁覆盖保真 id**（id 命名空间隔离：mod 型号 ≥ 1000 或前缀）；
2. **金样运行禁 mod 表**（保真面运行=纯保真表）；
3. **保真表内容哈希门禁**（哈希入 gates——非荣誉制；变更=勘误流程+金样重过）；
4. **mod 独立基线**（不进金样闸）。

**blocked-by v2 显式化【三度悬空统一解】**：本调研/状态机件 B（厚块抽出）/viewport 2c
（视口基线）同挂"表基建"依赖——**排期时统一解**（表基建=保真表哈希门禁+evidence 生成式+
校验器一次建成，三件消费方共享）。**一事实一宿主升级硬条件**：校验器断言
`sprite_defs.render_category == enemy_defs.sprite_ref 解引用值` **双写 diff=0**；
`sprite_ref` 回写 dispatch-shape-survey.md §4.1（1/3 条件 4——enemy_defs 瘦身为单字段引用）。

### 2.3 外键政策成文（1/3 条件 2）+ 加载层失败语义（1/3 条件 5）

**外键政策**：孤儿（sprite_ref 指空=拒载）/级联（sprite_defs 删行前校验引用）/跨表引用
（anim_profile→anim_defs、assembly_profile→SEG 表）/id 稳定性（保真 id 冻结、mod id 命名空间）。

**加载层失败语义（五类拒载）**：坏 JSON 拒载/坏外键拒载/域溢出拒载（型号域 0..88）/
帧序越界拒载（frames 下标超表长）/**unknown-field 策略=拒载**（防拼写错误静默）+
schema 版本字段 + **负例 golden**（五类各一 fixture 进 L1 门禁）。

---

## 3. 与表设计 v2 关联（v2 修正）

- **【v2 硬条件】一事实一宿主**：资源属性真值宿主=sprite_defs/anim_defs；enemy_defs 只存
  `sprite_ref` 单字段（校验器断言双写 diff=0）——**回写 dispatch-shape-survey.md §4.1**。
- **【v2 降格】零改码链**："同图集复用、消费替换完成后**域内**零改码"——
  **域外三界**：① ANIM_TABLE_IDX 89 索引域外新动画行=panic 边（扩表=改码）② 图集容器
  16 个常量域外=新容器=改码 ③ 战斗表数组索引（enemy_base_* 表行序）=扩表需重锚。
  **步 5 演练实跑=验收闸**（加假想新敌人=纯表+资源，验证零 Rust 改动；失败面设计
  【3/3④】：域外型/坏外键/quirk 型三类演练各一）。

---

## 4. 保真红线（v2 修正）

- **【v2 R1】锚定**：逐值锚三元组 `{index, value, a.java:line}` **生成式产出**（配机械
  对照器）；**450/451 错位已修正**（containers=450/counts=451）；type_width_px 迁移
  evidence 补录。
- **【v2 R2】金样验收闸三件**：
  1. **机械对照器进 gates**：javap/deobf 提取值做第二侧（脚本生成 evidence 三元组 +
     校验器比对——防手抄互证同源错误，R2 反例"转录错同源复制"）；
  2. **帧夹具重生成红线**：表勘误红**必须=表错**——夹具期望仅源自 Java 参照帧
     （与 viewport V3 同款洞两案同修：期望不从实现侧生成）；
  3. **保真表内容哈希闸**（§2.2 规则 3，非荣誉制）。
- **D4 quirk 保真**（v1 原则保留）：transition_overrides 规则形原样转译+顺序契约
  （§2.1）；**状态迁移三拍 RED 测**（生成拍/迁移拍/渲染拍逐拍断言——R3 反例 1 的
  "生成时即带 row8 外观"陷阱）。

---

## 5. 工作量与排期（3/3 修正版）

**【v2 更正 3/3②-1】估算依据**：每步明细+机器墙钟（金样 460s/闸 2-5min）+风险余量；
**编辑点合并账本**（与表设计 v2 去重——dispatch_sprite_size 特例/render_category/floor
特例归属本调研消费替换步，表设计 v2 的 hook/battle_kind 归属其批，禁双计）。

> 【2026-10-21 注·HD 探针 C3 回写】A5（game_view.rs paint_tiles 断言，本账本登记项）：
> HD 探针**未改 A5 生产行**，仅图集断言换算 `width/(16*d)==8`（commit 099ed71，陷阱①
> 列数 8 索引语义锚不动）——A5 归属仍归本调研消费替换步，探针不重计工时、不并账。

| 步 | 内容 | 工时 | 依据 |
|---|---|---|---|
| 1 | schema 定稿+保真表回填（逐值锚生成式+等值单测含负例五类） | 2-2.5 天 | 89+11+16+9 项逐值锚生成 |
| 2 | game-data 解析器+L1 golden+负例 golden | 1-1.5 天 | 五类拒载语义 |
| 3 | 消费替换（advance_frames 特例→transition_overrides/dispatch_sprite_size/图集数字/render_category 逐点 RED→GREEN，**顺序契约钉死**） | 2.5-3.5 天 | 4 点×金样墙钟+三拍 RED 测 |
| 4 | 机械对照器+保真表哈希闸进 gates+mod 骨架（**1 天装不下五件事【3/3⑤】**→1.5 天） | 1.5-2 天 | R2 三件+四规则 |
| 5 | 零改码链演练实跑（三类失败面设计+域外三界验证） | 0.5-1 天 | 3/3④ 验收标准 |
| — | R1-R4 条件件（增量） | +1-2 天 | lead 转述 3/3 条件件 |

**合计 8.5-11 天**（v1 的 6.5-7.5 作废）。

**【v2 更正 3/3②-2】合批主张改口**：v1"表设计+资源合批 13-15 天省时"被自家数字证伪
（合批 > 两线分计之和）——改为二选一：
- **A. 合批**（多花 2-4 天买**基建一次成型**：保真表哈希门禁/evidence 生成式/校验器
  一次建成三件共享——blocked-by v2 三度悬空的统一解）；
- **B. 分批**（表设计先行→资源批跟进；**按子面拆 commit 每 commit 一闸**——爆炸半径
  史实反例【3/3③-1】：大 diff 金样红的二分归因成本必须靠小 commit 消解）。
推荐 A（基建共享价值>2-4 天成本），由 lead 拍板。

---

## 6. 口径与复现

**口径体例**（test-count-taxonomy.md 引用）：`35-40[逻辑编辑点·文件级]`、`89[animRowByType
项数]`、`102[layout 常量]` 等首现标注；与表设计 v2 编辑点合并账本（§5）防双计。
复现命令沿 v1（§1 扫描器）+ 新增：`grep -n "resourceImageCounts" reference/src/deobf/a.java`
（451 张数表）、`grep -n "resourceNames" reference/src/deobf/a.java`（450 容器名序）。

---

## 7. 三路复核 15 条件吸收对照

| 条件 | 落点 |
|---|---|
| 1/3 schema 主件（+R3 合流） | §2.1 transition_overrides 事件条件规则形+assembly_profile+container_counts+quirks_in_code 留痕+顺序契约+三拍 RED 测 |
| 1/3 零改码链降格 | §3 "同图集复用、域内零改码"+域外三界+步 5 演练实跑验收闸 |
| 1/3 外键政策 | §2.3 孤儿/级联/跨表/id 稳定性 |
| 1/3 加载层失败语义 | §2.3 五类拒载+schema 版本+unknown-field 拒载+负例 golden |
| 1/3 条件 4 sprite_ref | §2.2/§3 双写 diff=0 硬条件+回写 dispatch-shape-survey §4.1 |
| 2/3 R1 锚定 | §2.1/§4 逐值锚三元组生成式+450/451 错位修正+type_width_px evidence 补录+三件漏收补齐 |
| 2/3 R2 金样闸 | §4 机械对照器进 gates+帧夹具重生成红线（V3 同款洞两案同修）+保真表哈希闸 |
| 2/3 R3 状态条件臂 | §2.1 transition_overrides+顺序契约+三拍 RED 测+显形行 9 |
| 2/3 R4 两份制 | §2.2 四规则写死+blocked-by v2 三度悬空显式化+一事实一宿主硬条件 |
| 3/3 ① 编辑点账本 | §1/§5 合并账本去重 |
| 3/3 ② 估算依据 | §5 每步明细+墙钟+余量 |
| 3/3 ②-2/③ 合批改口 | §5 A/B 二选一（多花 2-4 天买基建一次成型 or 分批每 commit 一闸） |
| 3/3 ④ 验收标准 | §3 步 5 三类失败面演练 |
| 3/3 ⑤ mod 骨架 | §5 步 4 1→1.5 天 |
| 口径体例 | §6 引用 test-count-taxonomy.md |

> 15 条件全部落回。待 lead 拍板（含合批 A/B 二选一）。
