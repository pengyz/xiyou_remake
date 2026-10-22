# 资源配置 V2·终检复核报告（hd-final-verifier / remake-verifier，task-9）

> 2026-10-22。独立终检（全程亲自 inline，未委托）：只看 diff + 证据 + 实跑结果。
> 复核对象 = V2 P1-P3 commit 链：P1 `a284203`（三表+220 逐值锚生成式+对照器）+
> `3821889..85734b1` 6 commit（解析器+五类拒载+golden+真表联调）；P2 `2d27c47`/`31982c4`/
> `05de96f`/`280c31d`/`7e4432e`（消费替换 5 点+一事实一宿主校验器）；P3 `9247706`/`7e2cce3`
> （mod 骨架+零改码链演练+哈希闸+table-evidence 闸）。
> 依据 = [resource-config-survey.md](../spec/resource-config-survey.md) §7 15 条件吸收表 +
> [v2-plan](../superpowers/plans/2026-10-21-resource-config-v2-plan.md) 拍板项与零漂移红线。
> 纪律（remake-verifier）：禁放宽阈值/白名单/删用例；未贴出的运行结果不存在——§2 全为本机实跑。

## 1. 条件核销表（survey 15 条件 + R1-R4 + 计划拍板项）

### 1.1 survey §7 15 条件

| # | 条件 | 落点 | 在场证据（实测锚） | 核销 |
|---|---|---|---|---|
| 1 | 1/3 schema 主件（+R3 合流） | §2.1 transition_overrides 事件条件规则形 + assembly_profile + container_counts + quirks_in_code 留痕 + 顺序契约 + 三拍 RED 测 | sprite_defs.json：transition_overrides 3 事件规则形（mark_removed 四臂/mark_des_removed/bump_reveal）、quirks_in_code Q-001..Q-005 逐条带 `src`+`red_test_id`、assembly_profile 外键 + sheet_index.assembly_profiles（seg67 9 片件+3×5 段帧）、container_counts 16；顺序契约 `resource_cfg.rs:8-9` 注记 + `task7_order_contract_init_then_override`；三拍 `task7_three_beat_*`（5 测） | ✅ |
| 2 | 1/3 零改码链降格 | §3 同图集复用/域内零改码 + 域外三界 + 步 5 演练实跑验收闸 | `tools/zero-code-drill/drill.py`（8 步 15 断言）+ `gate_zero_code_drill`（cmd_all）；**本机实跑 15/15 PASS**（[0b]/[7] `git diff`+`git status --porcelain` crates/ 双查皆空——见 §2.4） | ✅ |
| 3 | 1/3 外键政策 | §2.3 孤儿/级联/跨表/id 稳定性 | `defs_negative.rs`：`reject_class2_bad_foreign_key_{anim_profile,assembly_profile,sheet}`（孤儿）、`reject_class2_sprite_ref_dangling`（跨表）、`cascade_reference_listing`（级联）、`reject_duplicate_type_id`（id 稳定）——16 测全绿（§2.1 同跑） | ✅ |
| 4 | 1/3 加载层失败语义 | §2.3 五类拒载 + schema 版本 + unknown-field 拒载 + 负例 golden | `defs_negative.rs` 16 测：class1 坏 JSON / class2 外键 / class3 值域（+端点 0/88 接受）/ class4 帧索引（+末槽接受）/ class5 unknown-field×3（record/envelope/nested）/ schema_version 缺失与不匹配；`defs_golden.rs` 正例 3 测（真表 89/11/16 联调） | ✅ |
| 5 | 1/3 条件 4 sprite_ref | §2.2/§3 双写 diff=0 硬条件 + 回写 dispatch-shape-survey §4.1 | 双写 ✓：`one_fact_one_host.rs` 6 测（四宿主对拍：表宿主/保真冻结镜像/解引用链/邻接双写；漂移四类自检=注入即红，非荣誉制，`from_embedded` 装载即跑）。回写 ✓（终检红-1 催办后 lead 补【v2 回写注记】，§4.1 sprite_ref 瘦身/宿主/校验器承接齐，复验在场） | ✅（红-1 闭环） |
| 6 | 2/3 R1 锚定 | §2.1/§4 逐值锚三元组生成式 + 450/451 错位修正 + type_width_px evidence 补录 + 三件漏收补齐 | `tools/table-evidence/{generator,comparator,extractor}.py`（机械提取禁手抄）；表内三元组合计 496 条（gate 口径 220 宿主逐值等值）；错位修正=表侧逐字段锚（name→451 / count→**453**）+ survey 453 勘误核验 ✓（§2.6）；三件漏收=container_counts/SEG67 表中表/张数表均入表 | ✅ |
| 7 | 2/3 R2 金样闸 | §4 机械对照器进 gates + 帧夹具重生成红线 + 保真表哈希闸 | `gate_table_evidence`（复用 comparator 不重写第二套扫描）+ `gate_table_content_hash`（fail-closed，§2.3 复现）；帧夹具红线成文（survey §2.9/§4 步 2："夹具期望仅源自 Java 参照帧，表勘误红必须=表错"）+ V2 链 `data/golden` 零改动实证（§2.7） | ✅ |
| 8 | 2/3 R3 状态条件臂 | §2.1 transition_overrides + 顺序契约 + 三拍 RED 测 + 显形行 9 | `2d27c47`：scene.rs 消费改查表（`task7_three_beat_type6_reveal` 显形行 9、`task7_three_beat_t12_morph` 型 12 变形+Q-001、`task7_three_beat_mark_removed_category_arms` 四臂）；顺序契约测试 `task7_order_contract_init_then_override`（查表初始化→覆写，反序=红） | ✅ |
| 9 | 2/3 R4 两份制 | §2.2 四规则写死 + blocked-by v2 三度悬空显式化 + 一事实一宿主硬条件 | `tools/mod-skeleton/`（四规则 loader + 16 测 OK：mod 禁覆盖保真 id/金样禁 mod/哈希护航/mod 独立基线）；survey:116 "blocked-by v2 显式化【三度悬空统一解】"；`7e4432e` 校验器非荣誉制（装载即跑+有牙齿） | ✅ |
| 10 | 3/3 ① 编辑点账本 | §1/§5 合并账本去重 | survey §5 合并账本（含 HD 探针 A5 归属注记同账本去重） | ✅ |
| 11 | 3/3 ② 估算依据 | §5 每步明细 + 墙钟 + 余量 | survey §5 步骤表（8.5-11 天，逐步依据列） | ✅ |
| 12 | 3/3 ②-2/③ 合批改口 | §5 A/B 二选一 | lead 拍板①"分批+基建共享"落档 v2-plan（`7210902`，A/B 之争收口留痕） | ✅ |
| 13 | 3/3 ④ 验收标准 | §3 步 5 三类失败面演练 | drill 失败面 A/B/C 实跑：域外型（0..=88 域溢出拒载+mod 命名空间拒载）/坏外键（孤儿拒载）/quirk 型（QuirkPathRequired→Q-101 登记后过）——均带**真解析器**拒载报文（防镜像自证，§2.4） | ✅ |
| 14 | 3/3 ⑤ mod 骨架 | §5 步 4（1→1.5 天） | `tools/mod-skeleton/`（loader.py + mod_baseline.json + tests 16） | ✅ |
| 15 | 口径体例 | §6 引用 test-count-taxonomy.md | survey §6 口径体例段（首现标注+合并账本防双计） | ✅ |

**小计：15/15 全落（#5 经终检红-1 拦截→lead 补回写→复验闭环）。**

### 1.2 v2-plan 拍板项与零漂移红线

| 项 | 判据 | 实测 | 核销 |
|---|---|---|---|
| 拍板① 分批+基建共享 | 落档计划件 | v2-plan 7210902 + 实际按 task-5..9 分批走 | ✅ |
| 拍板② schema density 本批写入（默认 1） | 三表带 density 且不引渲染改动 | sprite_defs 89 行 `density:1` 全同值；`git diff a284203~1..HEAD -- crates/game-core/src/render.rs` **空**（HD 件不掺入） | ✅ |
| 拍板③ R-2 批次二不在本批 | 换算收敛点不被触碰 | render.rs/menu_family.rs/logo_anim.rs/intro.rs 全空 diff；diff 中 `logical_width/with_density` 命中 0 | ✅ |
| 拍板④ 资源字节零变更 | original/资产清单不动 | gates check `original-integrity` PASS + `assets-manifest` PASS（entries=138） | ✅ |
| 红线·classic 帧哈希/op 串零漂移 | 金样/oracle 对拍必绿 | `git diff a284203~1..HEAD -- data/golden data/status reference/ original/` **全空**；check `rust-tests` 全绿；trace diff-trace PASS（§2.2） | ✅ |
| 红线·帧夹具重生成红线 | 期望仅源自 Java 参照帧 | 红线成文 + V2 链零夹具重生成（golden/fixtures diff 空） | ✅ |
| 红线·探针/HD 件不掺入 | density 只写表默认 1 | 见拍板② | ✅ |

---

## 2. 实跑记录（本机 2026-10-22）

### 2.1 `python3 gates/cli.py check`（12/12 总闸）

```
[✓ PASS] original-integrity — 1 个归档校验通过
[✓ PASS] assets-manifest — 与基线一致 {'entries': 138, 'classes': 2, 'maplv': 55, 'sprite': 55}
[✓ PASS] reference-seed-integrity — 8457 行，sha256 对照基线通过
[✓ PASS] knowledge-format — 43 条目合规
[✓ PASS] docs-links — 全部相对链接有效
[✓ PASS] naming-ledger — 全部记录带证据指针
[✓ PASS] rust-tests — cargo test --release 全绿
[✓ PASS] test-ratchet — total=526(基线475) 无回退；新增二进制待 ratchet-update:
         {'defs_golden(test)': 3, 'defs_negative(test)': 16, 'one_fact_one_host(test)': 6,
          'render_category_table_driven(test)': 4, 'sheet_index_table_driven(test)': 6,
          'size_dispatch_table_driven(test)': 5, 'transition_table_driven(test)': 10}
[✓ PASS] table-content-hash — 保真三表 sha256 对照基线通过（逐值锚 220 宿主）
[✓ PASS] table-evidence — 机械提取值（等值）+ 确定性复跑同哈希
[✓ PASS] state-machine-anchors — 锚 6 + 边数对照；全命中
[✓ PASS] asset-truth-manifest — entries=138，art 三态核通过
== 汇总 == PASS=12 FAIL=0 SKIP=0
```

ratchet 读法：**存量零回退**（total 526≥475，逐二进制无一降）；7 个新二进制（P1/P2 测试面）
待 lead `ratchet-update` 随收尾入基线——与 HD 探针期 density_draw(16) 同款处置。

### 2.2 `python3 gates/cli.py trace`（L1-L3，复核人二次 trace）

```
[✓ PASS] diff-trace — cargo test -p game-oracle 全绿（exit 0）
（boot_replay 2 / fixture_frames 25 / gameplay_floor1 5 / l2_prologue 1 /
  preset_cases 35 / probe_floor2 1 / walk_replay_floor1 2 = L1-L3 全套件）
TRACE_EXIT=0
```

进程由本 verifier 亲自持有（debug 档 `cargo test -p game-oracle`，preset_cases 单二进制
247min CPU 量级）。注：HD 终检阶段（10-21）曾有一次 trace 跑到 preset_cases 中段被环境杀
（stdout 截断无 result 摘要、无 FAILED/panicked 行 = 进程中断非断言红，当日机器并发 3-4 套
重型测试）；本次同口径完整跑 PASS，classic 零漂移结论以本跑为准。

### 2.3 TDD 红证据三件套抽查复现（3 commit，worktree 独立复现）

| commit | 声称 RED | 独立复现 | 判定 |
|---|---|---|---|
| `a284203`（三表+生成式+对照器） | 等值单测 RED 8 fail（表缺 6+字面错位暴露，抓到 idx73/74 手读错位）→ 回填 → 16/16 绿 | ✅ worktree@a284203：基线 16/16 OK；**表缺红**（mv data/tables）→ FAILED(failures=7)；**字面错位红**（按其自述 idx73/74 造 anim_row 漂移）→ FAILED(failures=2) 且报文逐字咬合：`[anim_row 三元组.value] index=74 表值=5 提取值=2 src=a.java:573`；复原 → OK | ✅（红类逐类复现；表缺红计数 7 vs 自述 6+2 混态，状态不同红数不同，类别吻合） |
| `2d27c47`（transition_overrides+三拍） | 编译红 E0432 `game_core::resource_cfg` 未找到 → 行为红 → GREEN | ✅ worktree@85734b1 + 后置测试：`error[E0432]: unresolved imports game_core::resource_cfg … could not find resource_cfg in the game_core` **逐字复现** | ✅ |
| `7e2cce3`（哈希闸+对照器进闸） | 哈希基线挪走 → `[✗ FAIL] 闸基线缺失`（fail-closed）→ 回置 → PASS | ✅ 临时副本（/tmp，未碰真仓基线）：PASS → 移走基线 → `[✗ FAIL] table-content-hash — 闸基线缺失 table_content_hash.json` → 回置 → PASS **逐字复现** | ✅ |

三件套形态：红测试代码随 commit 在库（transition_table_driven.rs 10 测/等值单测 16/闸 fail-closed）；
红输出在 commit message 留痕且**复现一致**；GREEN 输出=本机 check/单测实跑（§2.1/§2.5）。

### 2.4 零改码链演练独立验真（防"假想敌人假纯表"作弊面）

`python3 tools/zero-code-drill/drill.py` 本机实跑：**15/15 PASS**。验真要点：

- **假想敌人真纯表**：id=1000 mod 行（anim_profile 复用 `anim_row_0`、资源行复用 sptenemy1/3）
  只写 mod 表——保真三表一字未动（[0]/[7] 哈希双查绿），全程零 Rust 文件改动；
- **git diff crates/ 空断言可信**：drill 用 `git diff -- crates/` + `git status --porcelain -- crates/`
  双查（含未追踪）在 [0b]/[7] 前后双卡；本 verifier 另独立核 `git diff/status -- crates/` 为空；
  真解析器 harness 搭在临时目录（path-dep crates/game-data），不入仓不触 crates/；
- **三类失败面走真解析器**（防镜像自证）：域溢出拒载报文 `值 89 超出域 0..=88（路径 sprite_defs[89].type_id）`、
  坏外键拒载报文 `anim_profile "anim_row_nope" 在 anim_defs 中不存在（孤儿拒载）`、
  quirk 型 `QuirkPathRequired→Q-101 登记后通过`；规则 2 金样混入 mod 表断言触发；
- 结论：**域内零改码链为真**（纯表加行可生效；域外三界=拒载/登记，不假承诺）。

### 2.5 闸自洽三件

1. **门禁自测**：check 12/12 见 §2.1（闸清单 10→12 后全数自证）。
2. **table-content-hash 三表 sha256 对账**：`python3 gates/cli.py table-content-hash` PASS；
   本 verifier 独立重算（hashlib sha256 逐表）：sprite_defs `7f5e2e80…` / anim_defs `99a92f78…` /
   sheet_index `a19c815a…` 三表 **MATCH** 基线；comparator `cli.py check` 等值✓ + 复跑1=复跑2=盘上同哈希。
3. **survey 453 勘误核验**：`reference/src/deobf/a.java:453` = `resourceImageCounts = new int[]{8,1,…}`
   （张数表实锚 **453**），:450-452 = resourceNames 名序（名条目锚 451）——表侧 evidence 已按
   name→451 / count→453 逐字段分锚（正确）；lead 待提交勘误（survey 两处 451→453+原地更正注记）
   与实锚一致 ✓。观察：survey §2.1 文档样例行 `src: a.java:450-451` 对 count=25 略糙（真值 453），
   建议顺手修（表侧无此问题）。

### 2.6 classic 零漂移抽验

`git diff a284203~1..HEAD -- data/golden data/status reference/ original/` = **空**（金样
t544-tiles.ops/render-golden.json、ratchet 基线、参考版、original 归档零变更）；check
rust-tests 全绿（含 render_golden Java-sha 16/16、fixture_frames 25、preset_cases 35 面）；
L1-L3 trace 见 §2.2。

## 3. 红项/黄项

| # | 级 | 项 | 定位（§4 三问） | 处置 |
|---|---|---|---|---|
| 红-1 | 红 | `sprite_ref` 回写 dispatch-shape-survey §4.1 缺失（条件 5 半落）：survey:120/:136 明文要求回写，目标文件 grep 零命中，§4.1 EnemyDef 仍旧形 | ①层=文档回写（登记有、写入无，与 HD 探针红-1 同类）；②消费者=表设计批 enemy_defs 实现者（读旧形会重蹈双写，违"一事实一宿主"）；③证据=A（grep 零命中可复现） | 报 lead → **已闭环（lead docs commit）**：§4.1【v2 回写注记】已补（sprite_ref 单字段/真值宿主=sprite_defs.render_category/双写 diff=0 由 one_fact_one_host 承接/表设计批加载侧断言/本批不动 enemy_defs）——措辞与建议一致，**复验在场** ✅ |
| 观-1 | 观察 | survey §2.1 样例行 `src: a.java:450-451` 对 count 值锚糙（真值 453，表侧已分锚） | 文档样例层；消费者=读者；证据 A | **已修**：样例行改分锚 `name a.java:450 / count a.java:453`，453 勘误两处活主张原地更正（历史叙述不动）——复验在场 ✅ |

（TDD 红证据、演练、闸、零漂移各项均未见放宽阈值/白名单/删用例痕迹。）

## 4. 遗留假设/待复验清单

1. ~~红-1 待复验~~ → **已闭环**：dispatch-shape-survey §4.1【v2 回写注记】与 one_fact_one_host
   承诺一致（复验在场，§3 红-1 行）；453 勘误+样例分锚一并复验 ✅。
2. **ratchet 基线**：7 个新测试二进制（defs_golden 3 / defs_negative 16 / one_fact_one_host 6 /
   render_category_table_driven 4 / sheet_index_table_driven 6 / size_dispatch_table_driven 5 /
   transition_table_driven 10）待 lead `ratchet-update` 入基线（check 已确认存量零回退）。
3. **表设计批（enemy_defs/item_defs）**：sprite_ref 瘦身与一事实一宿主加侧落在后续批
   （校验器已留接口——四宿主对拍可直接加 enemy_defs 侧）。
4. quirk Q-001..Q-005 为"装不下规则形"的显式登记（禁静默留码）——量产/表设计批如能表化
   需按勘误流程消账（当前为设计内合法留码）。

---

## 5. 结论

survey 15 条件 **15/15 全落**（#5 经终检拦截红-1→lead 回写→复验闭环）；计划拍板 3 项与
零漂移红线 3 条全过；TDD 红证据抽查 3/3 独立复现（含红类咬合报文）；零改码链演练 15/15
独立验真（真纯表+真解析器）；闸自洽三件齐（check 12/12、三表 sha256 独立 MATCH、453 勘误
核验一致）；classic 零漂移实证（金样/oracle/original 四面 diff 全空 + **trace L1-L3 PASS**）。
**红项 1 件已闭环（含复验），观察 1 件已修**——终检无遗留红项。

Evidence: A 级——check/trace 实跑输出、worktree 三连红复现（表缺 7 fail/错位 2 fail 带
`index=74 表值=5 提取值=2 src=a.java:573` 咬合/E0432/闸 fail-closed）、drill 15/15 含真解析器
拒载报文、三表 sha256 独立重算 MATCH、a.java:453 实锚核对、git diff 零漂移四面全空；
B 级——消费替换 diff 逐点核对（render_category 10 臂→表查+保真冻结镜像只读、顺序契约注记）；
C 级——无（本报告不涉人工判读）。
Confidence: 92
