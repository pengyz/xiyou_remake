# 资源配置语义 V2 实施计划（resource-config-v2）

> 2026-10-21。用户指令："落地资源管线 V2，落地资源配置语义，把资源大小等信息从代码中
> 解耦出来；分析方案，用团队实现，跑测试，复核"。设计件 = [resource-config-survey.md](../../spec/resource-config-survey.md)
> **v2**（三路对抗复核 15 条件已吸收，R1-R4 条件件齐）
> ——本计划只做排期与派工，不改设计主张。

## 0. lead 拍板（survey 遗留决策）

1. **合批/分批**：采 **分批 + 基建共享**（survey §5 二选一的 B 路 + A 的精神）——
   本批 = 资源配置语义（三表+消费替换+基建三件一次建成）；表设计批（enemy_defs/
   dispatch-shape §4.1 sprite_ref 瘦身、状态机件 B）后续批**共享基建**，不在本批。
2. **density 挂点**：v2 schema 预留 `density` 字段注记已回写（探针 R-5，2026-10-21）
   ——本批 schema 定稿时把该字段**写进 sprite_defs**（默认 1），与 HD 渲染通路对齐。
3. **R-2 批次二**（img.width→logical_* ~20 处）**不在本批**（那是 HD 密度消费点，
   v1.2 已登记）；本批解耦的是**数据表**（89/88/16/11/SEG67 等常量）。
4. 资源字节**不动**（本批零 repack）；高清替换的重制/重打包是量产批。

## 1. 阶段与判据（TDD 红证据先行，一 commit 一逻辑变更）

### Phase 1 表基建（task-5 + task-6 并行，schema 文件=接口契约）
- **task-5 schema/表/生成式**（写域 `data/tables/` + `tools/table-evidence/`）：
  sprite_defs.json / anim_defs.json / sheet_index.json 按 survey §2.1 定稿（含
  transition_overrides 事件条件规则形+assembly_profile+container_counts+quirks_in_code
  留痕+密度字段）；保真表回填 = **逐值锚三元组 `{index,value,a.java:line}` 生成式产出**
  （机械对照器=javap/deobf 提取值第二侧，防手抄互证同源错误——R2）；type_width_px 88 项
  迁入 sprite_defs 带 evidence 补录（R1）。判据：表值==机械提取值等值单测全绿。
- **task-6 解析器/拒载**（写域 `crates/game-data/`）：三表解析器+**五类拒载**（坏 JSON/
  坏外键/域溢出 0..88/帧序越界/unknown-field=拒载）+schema 版本字段+负例 golden 五件+
  L1 golden（tests/golden.rs 风格）。外键政策（survey §2.3：孤儿拒载/级联/跨表/id 冻结）。

### Phase 2 消费替换（task-7，挂 5+6）
- 写域 `crates/game-core/`：advance_frames 特例行→transition_overrides（**顺序契约
  钉死**：查表初始化→状态迁移覆写，反了静默失效）、dispatch_sprite_size 特例、图集
  数字、render_category 逐点 RED→GREEN；**状态迁移三拍 RED 测**（生成/迁移/渲染拍）；
  显形行 9 路径同批。一事实一宿主：校验器断言 sprite_defs.render_category==enemy_defs
  sprite_ref 解引用双写 diff=0。

### Phase 3 闸+演练（task-8，挂 7）
- 写域 `gates/` + `tools/`：保真表**内容哈希闸**进 gates（非荣誉制）、机械对照器进
  闸、mod 骨架（四规则：id 命名空间/金样禁 mod/哈希闸/mod 独立基线）、**零改码链演练
  实跑**（加假想新敌人=纯表+资源，验证零 Rust 改动；失败面三类各一——域外型/坏外键/
  quirk 型）。

### Phase 4 verifier 终检（task-9，挂 8）
- 19+条件核销（survey 15 条件落点 + R1-R4 条件件 + 本计划拍板项）、`gates check`/`trace`
  终跑、红项回派修复。**测试全绿≠完结，以复核为准**。

## 2. 零漂移红线

- classic 帧哈希/op 串零漂移（金样/oracle 对拍必绿）；保真表变更=勘误流程+金样重过；
- 帧夹具重生成红线：夹具期望仅源自 Java 参照帧，Java 未变⇒夹具永不重生成（R2 反例
  "重生成洗错"）；表勘误红必须=表错。
- 探针/HD 件不掺入（density 字段只写表默认 1，不引渲染改动）。

## 3. 团队写域（互不相交）

| task | 写域 | 依赖 |
|---|---|---|
| task-5 表+生成式 | `data/tables/`、`tools/table-evidence/` | — |
| task-6 解析器 | `crates/game-data/` | —（与 5 以 schema 文件为契约并行） |
| task-7 消费替换 | `crates/game-core/` | 5+6 |
| task-8 闸+演练 | `gates/`、`tools/` | 7 |
| task-9 终检 | `docs/reviews/` | 8 |

提交纪律：`git commit --only <路径>`（共享 index 竞态教训）；每 commit 过总闸；
TDD 红证据三件套入 commit message。
