# Stage A 终审预检：四判据机械盘点（task-14）

> 【2026-10-16】只读审计产物。数据全部由工具机械产出（`python3 tools/cover_matrix.py`、
> `python3 gates/cli.py scenarios`、`cargo test --release --workspace`、
> `python3 gates/cli.py ratchet`、`python3 tools/dead_code_ruling.py --check`），
> 逐项标注取数时间与命令。本报告不含手编结论；分类判断（待刷快照/真缺口 等）为审计标注，
> 依据写在各行。审计期间工作树有并行队友在途改动（task-10 夹具批次），相关项已标注"过渡态"。

## 0. 总览：距 Stage A 完成还差多少（机械答案）

| 判据 | 判据原文（convergence-plan §0） | 现状 | 赤字 | 销账路径 |
|---|---|---|---|---|
| ① 覆盖矩阵 | 白项清零、黄项带论证 | 绿 146 / 黄 8 / 白 9；判据①赤字 **7** | 7 | 见 §1：黄 7 处置（6 补用例 + 1 待论证）；白 9 全 ☠️ 已裁死 |
| ② 状态差分 | oracle 场景套件全 PASS | **44/44 PASS**（A==B==C 三方对拍） | 0 | 无（含 task-10 新夹具后仍全绿） |
| ③ 帧对拍 | cargo test 全绿 + ratchet 无回退 | 全绿（252 项，0 失败；ratchet total=252 基线 223） | 0 | 无 |
| ④ 类型闭包 | 22 真格全部销账或终裁 | 22 真格中 ⬜ 18 + 🎨 3 + 过程行 1；已分类无遗漏（§4） | 22 待销 | 5 已触达待翻 / 6 夹具在做 / 8 补或裁 / 2+1 过程行 |

**机械结论**：判据②③ 已达成（0 赤字）；判据① 差 7 项（全部有明确销账路径，无"无路径"项）；
判据④ 差 22 格翻账（分类已核对无遗漏，其中 5 格证据已齐可立即翻、6 格等 task-10 夹具验证）。
距 Stage A 完成的剩余工作 = ① 的 7 项处置 + ④ 的 22 格销账 + task-6 收官核验（四本账全绿 + master-plan 终审表）。

---

## 1. 判据①：覆盖矩阵赤字清单（tools/cover_matrix.py）

取数：`python3 tools/cover_matrix.py`（2026-10-16 15:0x 重跑）。

汇总行（生成物 §汇总）：**绿 146 / 黄 8 / 白 9**（共 163 方法）。
判据①赤字 = 白（非 ☠️）0 + 黄（无论证）7 = **7**。

### 1.1 白项 9 = 全部 ☠️ 已裁死（不计赤字）——确认无真缺口

DC-3 联网簇 7 项（m_148…m_154）、DC-4 addSpark、DC-1 m_093 相关 1 项；
`python3 tools/dead_code_ruling.py --check` 输出「死代码裁决复核：全部成立 DC-1/DC-2/DC-3/DC-4/DC-5」。
**白（非 ☠️）= 0 ⇒ 白项清零判据已满足。**

### 1.2 黄项 8 逐项（判据①赤字 7 的构成）

| # | 方法 | 触达场景@t_hit | 现有覆盖 | 标注 | 销账路径 |
|---|---|---|---|---|---|
| 1 | `advanceScriptWalk` | fx-scriptwalk-floor3@T308 | `fx_scriptwalk_floor3_frames_match`（fixture_frames.rs:331，704 帧全链） | **待刷快照** | 夹具在库且全链断言；矩阵仍判黄系 method_reach 快照（10-15 20:51）未含 10-16 新夹具批次 ⇒ 刷 `tools/method_reach.py` 快照 + 重生成矩阵即应转绿 |
| 2 | `browseOfferQueue` | preset-floor5@T621 | `preset_floor5_fixture_frames_match`（:307，1204 帧全链） | **待刷快照** | 同上 |
| 3 | `addItemToItemStack` | preset-floor5@T265 | 同 #2 夹具（T265 ⊂ T1-1200 窗口） | **待刷快照** | 同上；另见 §4.1 c 类「判定存疑」已注明 |
| 4 | `m_089`（activateItem 未命中提示） | fx-fan-floor13@T381 | `fx_fan_floor13_frames_match`（:353，834 帧全链） | **待刷快照** | 同上 |
| 5 | `m_101` | fx-knockback-floor43@T305 | `fx_knockback_floor43_frames_match`（:342，604 帧全链） | **待刷快照** | 同上 |
| 6 | `probeCrossFloorSnapshot` | preset-floor49@T951 | `preset_floor49_fixture_frames_match`（:320，3404 帧全链） | **待刷快照** | 同上 |
| 7 | `m_107`（MVS 实体换格） | 无场景触达 | **DC-2 已裁**：MVS 为死指令（68 条脚本零出现），m_107 系其唯一被调者 ⇒ 不可达（dead-code-rulings.md DC-2 节） | **真缺口→待论证登记** | 非死代码本身（Rust 已移植 fn m_107），需在 YELLOW_NOTES 补「不触达论证」引用 DC-2 裁决；或按 DC-2 建立合成单测（task-10 FX-MVS-A `mvs_pair_scene` 在途）后转绿 |
| — | `hideNotify` | 无场景触达 | 不触达论证（B 级，机械校验 ✓）：MIDP 生命周期回调，oracle 宿主驱动面不可达；语义由壳侧 5 个宿主测试覆盖（ab8368a） | **已有论证（不计赤字）** | 无 |

### 1.3 过渡态专项：新夹具未刷快照

method_reach 快照 `tools/method_reach/snapshot.json` 生成于 **10-15 20:51**，
而 task-10 四批夹具落地于 **10-16 14:03–14:47**（903dc95 / f3506cc / 2d802fd / 4f2fc30），
快照必然不含 fx-int6a / fx-door2a/2b / fx-item17a/17b/18a/19a / fx-boss67a/74a 的触达数据
（fx-bug-aioobe-border 亦「无触达快照」）。**这些夹具的销账效应（型 2/6、道具 17/18/19、
boss 67/74 转绿）全部处于"待刷快照"状态**，刷新后预计：

- 黄 6（#1–#6）转绿；
- 型 2/6、道具 17/18/19 的账本行（§4「已触达待翻 5」）获得触达快照支撑。

**机械动作**：`python3 tools/method_reach.py`（JDI 重采）→ `python3 tools/cover_matrix.py`（重生成）。
⚠ 生成物禁手改（docs/knowledge decision：改配置源 ARM_GAPS/PORT_NOTES 后重生成）。

---

## 2. 判据②：oracle 场景套件（gates/cli.py scenarios）

取数：`python3 gates/cli.py scenarios`（2026-10-16 14:36–15:02 全量重跑）。

```
[✓ PASS] scenario-suite — [oracle] == --scenarios 结果：PASS（44/44 场景）==
```

- 场景数 **44**（reference/oracle/scenarios/*.txt 计数 44；`.preset` 为预置伴生文件不计）。
- 全部 **A==B==C 三方对拍 PASS**，trace sha256 三方一致。
- 本 run 同时验证了判据①前提：44 场景中含 10-16 新增夹具场景对应的 oracle 侧脚本无回退。
- 注：CLI exit code 1 系 `gate_scenarios` 返回值经 `sys.exit()` 打印的怪癖（stderr "PASS" 即证），
  非失败；矩阵与本报告以输出行 `[✓ PASS] scenario-suite … 44/44` 为准。
- **判据②现状：0 赤字。**

---

## 3. 判据③：帧对拍（cargo test + ratchet）

取数：`cargo test --release --workspace`（2026-10-16 14:36–14:5x）+ `python3 gates/cli.py ratchet`。

全绿清单（25 个测试目标，0 failed）：

| 测试目标 | 数量 | 测试目标 | 数量 |
|---|---|---|---|
| game-core lib | 72 | boot_replay | 2 |
| game-core/combat_golden | 6 | fixture_frames | 14 |
| game-core/entity_golden | 8 | gameplay_floor1 | 2（+3 ignored） |
| game-core/mov_route_scene | 4 | l2_prologue | 1 |
| game-core/render_golden | 16 | preset_cases | 35 |
| game-core/script_golden | 14 | probe_floor2 | 1 |
| game-core/slot_equip_scene | 2 | walk_replay_floor1 | 2 |
| game-core/walk_camera_golden | 4 | game-platform lib | 5 |
| game-data lib | 23 | game-oracle lib | 0 |
| game-data/golden | 9 | game-desktop(xiyou) lib | 25 |
| game-data/png_golden | 1 | Doc-tests ×4 | 0 |

- **ratchet：total=252（基线 223）无回退**（`gates/cli.py ratchet` ✓ PASS；14:50 时点曾为 249，
  252 系队友 fx-boss67a/74a 夹具测试入库后计数；两次均 ≥ 基线且单调不降）。
- gameplay_floor1 的 3 个 `#[ignore]` 为既有 dump/占位用例（非失败）。
- **判据③现状：0 赤字。**

---

## 4. 判据④：类型闭包 22 真格销账路径分类核对（docs/spec/type-closure.md）

机械提取（脚本解析状态格）：⬜ **18** 行 + 🎨 **3** 行（32 长寿丹、53 金刚力士、67 啸天犬）
= 21 行，加 67 行内含"战 ⬜"子格 1 个 = **22 真格**。
（另：MOV/SEE 两行状态 ✅ 但注文含历史 ⬜ 字样——系更正文化的历史主张保留，非真格；
155/159 行的 ⬜→✅ 更正标注不计。）

### 4.1 分类核对（对照 lead 分类：5 / 6 / 8 / 2+1 = 22）

| 分类 | 数 | 行 | 销账路径（机械） |
|---|---|---|---|
| **已触达待翻** | **5** | 型 2 红门、型 6 隐形路径、道具 17 金勺子、道具 18 玄明石、道具 19 千年月桂露 | task-10 四批夹具已全部入库（903dc95/f3506cc/2d802fd/4f2fc30）且 cargo 全绿；证据文档齐（evidence-interact-types R-INT-2/6、evidence-items §2.2–2.4）。**只差**：① method_reach 刷快照；② 账本翻格（日期更正体例）。与 task-12 禁动令一致（task-12 明示此 5 行"等 task-10 夹具触达后翻"——现夹具已落，条件解除） |
| **夹具在做** | **6** | boss 67 啸天犬（战子格）、boss 74 巨灵神、boss 68 太上老君、boss 84 嫦娥、道具 32 长寿丹（🎨 出现未拾）、53 金刚力士（🎨 带钥重入战） | 67/74：fx-boss67a/74a 已落（4f2fc30）待刷快照验证；68/84：task-13 fixture-plan-boss 设计中（evidence-boss 语义已齐）；32/53：🎨 半格，需补拾取/重入夹具或按半格口径终裁 |
| **补或裁** | **8** | 型 11、型 12、型 82、道具 15/23/24/25、boss 87 | 11/12：撞击臂已移植（ee52191）但 :5442 层条件随型 11 一并覆盖、FX 未立 ⇒ 补条件夹具或随主行注记；82：ARM_GAPS 已销账（Java 无 case 76/82，`_ => {}` 忠实——evidence-todos T-1 附带登记，2026-10-16 更正），FX-MVS-A 合成单测在途；15/23/24/25：task-12 被动四件 → 📦 降级（evidence-items §2 空操作臂+被动效果已移植）；87：task-12 → ☠️（spawnEntity(87)=0 命中存证） |
| **过程行 2+1** | **3** | :5442（型11 墙文案 !=23）、:7833（==40）+ 67 战子格 | :5442 随型 11 主行一并覆盖（注文明示）；:7833 语义销账（a.java:7832-7850 40 层禁瞬移臂，Rust scene.rs:2877-2881 已移植），保留 ⬜ 附 A 级理由（真臂零差分夹具）或补 floor40+item20 夹具；67 战子格随 FX-BOSS-67A 验证后并入主行翻格 |

**核对结论**：5+6+8+(2+1) = 22，与机械提取 22 真格**逐行对上，无遗漏、无重复**。
（lead 分类的"过程行 2+1"中"+1"= 67 行的战臂子格——67 状态格为 🎨（绘制已移植）但注文
"战 ⬜——delta"，故一行两格：绘制格随 67 主行、战格计入过程行组。机械解析口径如上。）

### 4.2 差异与注意项

1. **task-12 与 task-10 的交接时点已到**：task-12 禁动 5 行的理由是"等 task-10 夹具触达"，
   现 4 批夹具全部入库且全绿 ⇒ 该 5 行翻格条件成立（仍需刷快照作触达证据）。
2. **型 11/12 的过程行**（task-12 第 4 项）随主行注记同步——主行在"补或裁 8"，
   过程行 :5442 在"2+1"，两处需同批翻格避免账本自相矛盾。
3. MOV 行注文内"dp2=m_106 …⬜（挂起账本）"字样系历史主张（同一行已【更正 △→✅】6a37155），
   按更正文化保留；**不是**待销真格。SEE 行同理（【更正 ⬜→✅】帧锚复核完成）。

---

## 5. 审计期间观察（非本任务处置，供 lead 知悉）

1. **a.java 在途语义更正**（工作树未提交，非我改动）：paintEntityLayer 型 67 多段拼装的
   反编译类型失真修正（Object[] 强转 → byte[][] 逐字节读），带 javap 字节码 A 级依据
   （a.before.javap.txt 方法偏移 1153-1220）与日期更正体例。属 reference/ 语义更正类，
   收敛后需 `gates/cli.py reference` 逐 tick 不变 + Evidence/Confidence 提交字段。
2. task-10 在途未提交夹具：fx-boss67a/74a 三件套（frames/preset/txt）等（staged）。
3. 本报告生成时 `tools/cover_matrix.py` 重跑会把上述在途夹具扫进 §1 用例表（+6 行过渡态）；
   按生成物纪律**不手改**，待 task-10 提交后随 method_reach 刷快照一次性重生成。

## 6. 取数命令复现

```bash
python3 tools/cover_matrix.py                      # 判据①（重生成 docs/coverage-matrix.md）
python3 gates/cli.py scenarios                     # 判据②（44/44）
cargo test --release --workspace                   # 判据③（252 项全绿）
python3 gates/cli.py ratchet                       # 判据③（total=252 基线 223）
python3 tools/dead_code_ruling.py --check          # 判据①白项 ☠️ 机械复核（DC-1..5 全部成立）
```
