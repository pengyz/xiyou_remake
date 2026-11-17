# 对抗复核报告：DDDSA P2 批0 笔B — subsystem 骨架（aeb6964）

- 复核对象：commit `aeb6964`（分支 modern），14 文件 +350/−2 —— DDDSA P2 批0 笔B「subsystem 骨架 + 可见性 2 刀 + 测试 5 件 + plan 原地更正」
- 复核角色：对抗复核员（只读，唯一写=本报告）
- 复核日期：2026-11-07
- 证据分级：全部结论带实测输出（A=命令输出/编译与测试回显；B=代码语境+可复现静态推理）
- 复核命令环境：`/home/peng/workspace/xiyou_remake`，工作区在复核前后均为「被跟踪文件零改动」

## 总判：**可推送**（0 红 / 4 黄-轻 / 6 绿）

七面无阻断项。spec 忠实度逐字对照通过、Ctx↔registry↔归属表三方对账零不一致、可见性 2 刀实证为 lint 预防面且 diff 零行为、5 件测试全绿且变异攻击下两两判别、TDD 红档与绿态因果可独立复现、13/13 闸面全绿、14 文件零夹带。
4 条黄-轻（全部非阻断）见「发现清单」，建议批1–批2 顺手消化。

| 面 | 复核项 | 判定 |
|---|---|---|
| 1 | spec §3.1 八 trait 忠实度 / 空壳裁定 / 占位注记 | 🟢 绿（附 2 建议） |
| 2 | Ctx 族严读 + WRITE_FACE_REGISTRY 对账 | 🟡 黄（轻）：mod.rs:17 注释键名失实 |
| 3 | 可见性 2 刀零行为 + E0446 预防面 | 🟢 绿 |
| 4 | 测试真伪 + 变异攻击判别力 | 🟢 绿（附 1 建议） |
| 5 | TDD 红证据存在性与绿态因果 | 🟢 绿 |
| 6 | 闸面独立重跑 | 🟢 绿 |
| 7 | 夹带 / plan 更正格式 / 警示归属 | 🟢 绿 |

---

## 面1：spec 忠实度（防杜撰/偏形）🟢 绿

**基准**：`docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md` §3.1 = 行 **:96–115**（`subsystem/mod.rs:3` 的 `:96-115` 引用属实）。

### 1.1 八 trait 逐字对照（spec 行 ↔ 实现）

| spec 行 | spec 形 | 实现 | 判定 |
|---|---|---|---|
| :98 | `fn resolve_bump(&mut self, ctx: &mut Ctx, t: EnemyTypeId) -> BattleOutcome;` | `subsystem/battle.rs:38` `fn resolve_bump(&mut self, ctx: &mut BattleCtx, t: EnemyTypeId) -> BattleOutcome;` | 仅 `Ctx`→`BattleCtx`（plan 批0「每 trait 专属 Ctx」授权），**其余逐字同** |
| :99 | `fn tick(&mut self, ctx: &mut Ctx) -> DialogStep;` | `subsystem/dialog.rs:24` `fn tick(&mut self, ctx: &mut DialogCtx) -> DialogStep;` | 同上，其余逐字同 |
| :100 | `fn serialize(&self, st: &SceneState) -> Blob;` | `subsystem/persist_codec.rs:13` **逐字同**；`:14` 另有 `fn deserialize(&self, blob: &Blob) -> SceneState;` | serialize 逐字 ✓；deserialize 为 spec 单行外新增（见 F1） |
| :101 | `fn execute_op(&mut self, ctx: &mut Ctx, op: ScriptOp) -> ScriptStep;` | `subsystem/script_host.rs:29` `fn execute_op(&mut self, ctx: &mut ScriptCtx, op: ScriptOp) -> ScriptStep;` | 仅 `Ctx`→`ScriptCtx`，其余逐字同 |
| :102 | `trait ItemService { …与 ItemRegistry 职责合并/明示（C Q3）… }` | `subsystem/item_service.rs:7` `pub trait ItemService {}` + 「批6 终形」注记 | **空壳裁定=合规**（见 1.2） |
| :103 | `fn map(&self, ev: RawInput) -> Action;` | `subsystem/input_mapper.rs:12` | **逐字全同**（无 ctx，spec 也无） |
| :104 | `fn spawn/remove/relocate(&mut self, …);` | `subsystem/entity_service.rs:19` `pub trait EntityService {}`；方法名仅存于 doc `entity_service.rs:1` / `:18` | **空壳裁定=合规**（见 1.2） |
| :105 | `fn snapshot(&self, …) -> UiFrame;` | `subsystem/ui_render.rs:10` `fn snapshot(&self) -> UiFrame;` | `…` 被收窄为无参（见 F2） |

**结论**：六个有形 trait 中，四个签名与 spec 逐字同（差异仅 `Ctx`→专属 Ctx，plan 批0 明文授权）；`InputMapper::map` 完全无差异。**未发现任何凭空杜撰的方法、参数或返回型。**

### 1.2 item_service / entity_service 空壳裁定：**合规（不属漏实现）**

- spec:102 内容只有 `…` 与职责注记（C Q3 终案），**没有方法名、没有签名** —— 填任何签名都是 C 级臆造，违反 AGENTS.md §3「禁止猜测性结论 / A/B 级才能写入 spec 结论」。
- spec:104 有方法名但参形是 `…` 且无返回型；Rust trait fn 不支持 varargs，**任何声明都必须臆造完整类型**。实现侧把方法名保留在 doc 注释（`entity_service.rs:1`、`:18`），终形由 plan 批1（实体面 9 方法）承接。
- plan 批0 的范围本就是「subsystem 模块族立项 + 每 trait 专属 Ctx + 写面登记」，不含方法实现。
- ⇒ 「spec 形而未写」= **合规**（骨架阶段正确地选择「不预设」）；**不是漏实现**。风险仅在于方法名面无机器断言（见 F3）。

### 1.3 占位型 6/6 全带【批N 占位】注记 ✓（A 级 grep 实证）

| 占位型 | 注记 | 位置 |
|---|---|---|
| `BattleOutcome` | 【批2 占位】 | `subsystem/battle.rs:8` |
| `DialogStep` | 【批3 占位】 | `subsystem/dialog.rs:8` |
| `SceneState` | 【批4 占位】 | `subsystem/persist_codec.rs:5` |
| `ScriptStep` | 【批5 占位】 | `subsystem/script_host.rs:10` |
| `RawInput` | 【批7 占位】 | `subsystem/input_mapper.rs:7` |
| `UiFrame` | 【批8 占位】 | `subsystem/ui_render.rs:5` |

另 `Blob(pub Vec<u8>)`、`EnemyTypeId(pub i32)` 为 newtype 而非占位，均带理据注释（Blob=safe.rs RMS 字节 B 级；EnemyTypeId=i32 铸型）——归类正确，未被误标为占位。

---

## 面2：Ctx 族严读 + WRITE_FACE_REGISTRY 三方对账 🟡 黄（轻）

**方法**：不只信 Rust 测试，另用独立 Python 解析 registry 与归属表复算（A 级）。

### 2.1 行数与算术核 ✓

- registry（`crates/game-core/src/subsystem/mod.rs` `WRITE_FACE_REGISTRY`）行数 = **43**
- 按 face 分组：**DialogCtx 22 / BattleCtx 10 / ScriptCtx 6 / ItemCtx 4 / EntityCtx 1** ⇒ 10+22+1+4+6 = **43** ✓（与 commit message 的 43 算术核一致）

### 2.2 与 `data/status/field-ownership.json` 逐行对账 ✓（独立复算）

- 归属表 **186** 条唯一 field，无重复；域分布：menu-ui 42 / dialog-vm 22 / tap-input 21 / entity 19 / render 18 / coordinator 15 / item-effect 14 / persist 13 / **combat 11** / **script 6** / env-infra 5。
- registry 43 行的 `(field, domain)` 与归属表 **零不一致**，43 个 field 全部在表内。
- `combat` 域 11 键 = BattleCtx **10 写槽** + `enemy_base`（= BattleCtx **读槽**，正确地未登记为写面）。
- `dialog-vm` 域 22 键 ≡ `scene_dialog.rs:5-39` 的 DialogState 22 字段 ≡ registry DialogCtx 22 行（漏登 []、多登 []）。
- `script` 域 6 键 ≡ ScriptCtx 6 写槽；registry 引用的全部 domain（entity/item-effect/menu-ui/persist/combat/dialog-vm/script）在归属表中真实存在。

### 2.3 严读攻击点：读槽是否被误登记为写面 → **零命中** ✓

| Ctx | 读槽 | registry 中是否出现（按 face+field 精确匹配） |
|---|---|---|
| BattleCtx | `player` / `enemies` / `enemy_base` | 否 ✓（唯一 `field:"player"` 行的 face=**ItemCtx**，`mod.rs:82`，`&mut PlayerCombat` 写槽，(face,field) 键不冲突） |
| ScriptCtx | `player` / `entities` | 否 ✓（`entities` 的写面行 face=EntityCtx） |
| EntityCtx | `grid` / `walkable` | 否 ✓ |
| DialogCtx（值拷贝） | `key_value` / `key_held` / `frame_odd` | 否 ✓（前二归 coordinator 域，frame_odd 不在归属表——它们是函数参数拷贝而非状态字段） |
| ItemCtx（值拷贝） | `floor` / `equip_types` / `equip_bonuses` | 否 ✓ |

**结论：严读=执行（读槽登记即漏洞）——本轮无一读槽被登记，漏洞面为 0。**

### 2.4 反向：写槽是否漏登 → **零漏登** ✓

- BattleCtx 10/10、ScriptCtx 6/6、EntityCtx 1/1、ItemCtx 4/4 全部有登记行。
- DialogCtx 结构上只有 1 个 `&mut` 槽 `dialog: &'a mut DialogState`（记录槽），其 22 行=展开该记录槽内部 22 字段；3 个值拷贝读槽不属写面。口径已在 `mod.rs:14-17/40` 文档声明（行粒度：Dialog 展开到字段级、Entity/Item 收敛到槽级并用 `flds` 展开），**非缺陷**。

### 2.5 flds ↔ state_dump 双向核 ✓

`crates/game-oracle/src/state_dump.rs`（`dump_scene` @ :96，push 键 60 个）：

- registry 32 个非空 `flds` **全部**出现在 dump 体内（Battle 1 `battleActiveFlag`；Dialog 5 `dialogPage/dialogPhase/overlayActive/overlayKind/overlayText`；Script 3 `currentScriptIndex/scriptLineFlags/scriptWalkArmed`；Entity 10 个 entity*；Item `player`→10 键 + `tutorialSeen` + `equippedWeaponType` + `equippedArmorType`）。
- 反向「dump 有键但 registry `flds` 空」= **[]**。

### 2.6 🟡 F1（黄-轻，非阻断）：`mod.rs:17` 注释引用了不存在的键

`crates/game-core/src/subsystem/mod.rs:17` 写 `entityBaseGold 等派生键不入`，但全仓 grep `entityBaseGold` **仅此一处**；dump 实存键名为 **`enemyBaseGold`**（state_dump.rs 实测；另见 `docs/findings/reviews/2026-11-06-p16-data-review.md:36`）。
→ 注释键名失实（文档级、零行为）。建议改为 `enemyBaseGold`，并按 AGENTS.md §8「引用前必须验证仍存在」自检。

---

## 面3：可见性 2 刀零行为 + E0446 预防面 🟢 绿

### 3.1 `git show aeb6964` 逐行核

| 文件 | 变化 | 行为影响 |
|---|---|---|
| `crates/game-core/src/script_ops.rs:5` | `-pub(crate) enum ScriptOp` → `+pub enum ScriptOp` | 仅可见性关键字 |
| `crates/game-core/src/scene.rs:58-59` | `-pub(crate) use script_ops::ScriptOp;` → `+pub use …`；`+pub use scene_dialog::DialogState;` | 仅 `pub use` / 新增 facade 复出 |
| `crates/game-core/src/lib.rs` | `+pub mod subsystem;`（1 行） | 新模块声明 |

除可见性、`pub use`、新模块声明外，**diff 零行为**（无逻辑行、无表达式、无语句改动；SceneState/DialogState 等无逻辑变化）。`ScriptOp`/`DialogState` 在 game-core 之外无消费者（`crates/` 内除 game-core 外 grep 零命中）⇒ 放宽可见性是纯 API 面。

### 3.2 E0446 预防面：**A 级实证**

把两刀临时还原为 commit 前状态（`pub(crate) enum ScriptOp` + `pub(crate) use`），`cargo check -p game-core --release --lib` 实测输出：

```
warning: type `ScriptOp` is more private than the item `ScriptHost::execute_op`
  --> crates/game-core/src/subsystem/script_host.rs:29:5
29 |     fn execute_op(&mut self, ctx: &mut ScriptCtx, op: ScriptOp) -> ScriptStep;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method `ScriptHost::execute_op` is reachable at visibility `pub`
warning: `game-core` (lib) generated 1 warning
```

⇒ 「pub trait 签名里出现 pub(crate) 类型 ⇒ 私有类型进公开接口」的 lint 被**独立复现**，可见性 2 刀确为该 lint 的预防面，理由成立（现行工具链下为 lint warning，非硬错误，故「预防面」表述准确）。已还原。

### 3.3 restore_rules 先例实存 ✓

`crates/game-core/src/scene.rs:60` `pub use restore_rules::{RestoreAction, RestoreRule, RESTORE_RULES};`，`restore_rules.rs:5` `pub enum RestoreAction`、`:29` `pub struct RestoreRule` —— 私有 mod + `pub use` facade 同形先例（任务给的「:59 附近」属实）。

---

## 面4：测试真伪 + 变异攻击 🟢 绿

### 4.1 基线绿（A 级）

```
cargo test -p game-core --release subsystem
test subsystem::tests::p2b0_registry_rows_43 ... ok
test subsystem::tests::p2b0_trait_shapes_present ... ok
test subsystem::tests::p2b0_ctx_pure_data_constructible ... ok
test subsystem::tests::p2b0_registry_rows_match_field_ownership ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 339 filtered out
```

```
cargo test -p game-oracle --release --test dump_face
test p1a1_dump_face_contains_six_groups ... ok
test p2b0_registry_flds_present_in_state_dump ... ok
test result: ok. 2 passed; 0 failed
```

> ⚠ 命令坑（如实记录）：任务给的 `cargo test -p game-oracle --release dump_face` **只命中 1 个测试**（`dump_face` 过滤词匹配不到 `p2b0_registry_flds_present_in_state_dump`），已按「或按测试名」改用 `--test dump_face` 跑全 2 个。p2b0 四测 + dump_face 对照测 **全绿**。

### 4.2 变异攻击（判别力）——两个对账测均变红 ✓

**M-A+B（`mod.rs:43` 一行同时改 `field:"battle_activeX"` + `flds:&["battleActiveFlagX"]`）：**

```
# oracle
test p2b0_registry_flds_present_in_state_dump ... FAILED
panicked at crates/game-oracle/tests/dump_face.rs:33:13:
写面登记 FLD 失联：battleActiveFlagX（face=BattleCtx field=battle_activeX）——state_dump push 面无此键
test result: FAILED. 1 passed; 1 failed      [exit=101]

# game-core
test subsystem::tests::p2b0_registry_rows_match_field_ownership ... FAILED
panicked at crates/game-core/src/subsystem/mod.rs:130:36:
归属表缺键：battle_activeX（registry face=BattleCtx）
test result: FAILED. 3 passed; 1 failed      [exit=101]
```

**M-C（`battle.rs` `pub trait BattleSubsystem`→`BattleSubsystemX`）：**

```
error[E0432]: unresolved import `super::battle::BattleSubsystem`
error: could not compile `game-core` (lib test) due to 1 previous error; 4 warnings emitted
[exit=101]
```

与红档 `/tmp/ev_b_red.txt` 首行**逐字同**，且 `4 warnings emitted` 亦与红档一致 ⇒ 红↔绿因果可独立复现。

**还原验证**：三个变异文件全部 `cp` 回备份后，`git status --porcelain -- crates/` = 空、`git diff --stat` = 空；复跑 game-core 4 测 + oracle 2 测**全绿**。

### 4.3 🟡 F3（黄-轻，非阻断）：`p2b0_ctx_pure_data_constructible` 覆盖面小于其名

只构造了 **EntityCtx**（5 个 Ctx 之一）；且「Ctx 纯数据·无方法（三宪①）」**无机器断言**（带方法的结构同样能过该测试）。建议批1–2 补 BattleCtx/DialogCtx/ScriptCtx/ItemCtx 构造用例，或把测试名收窄为 `entity_ctx_pure_data_constructible`。

---

## 面5：TDD 红证据 🟢 绿

- `/tmp/ev_b_red.txt` **存在**，744 字节，mtime **Nov 7 06:29**；commit 时间 06:51 ⇒ 红档**早于** commit ✓。
- 内容：**8×E0432**（battle / dialog / entity_service×2 / input_mapper / item_service / persist_codec / script_host / ui_render）+ `error: could not compile `game-core` (lib test) due to 8 previous errors; 4 warnings emitted` + `[exit=0]`，与 commit message Evidence ① 逐字同。
- 因果抽查（A 级）：见 4.2 M-C —— 移除任一 trait 即复现同一 `E0432: unresolved import`；骨架在即绿。**红=trait 缺位、绿=骨架在** 的因果链成立。
- 附带观察（不阻断）：红档末行 `[exit=0]` 是 `cargo … | tee` 未设 `pipefail` 的管道退出码；错误文本本身已证明编译失败（真编译失败 exit=101，已由 M-C 复测）。建议红档采集脚本加 `set -o pipefail`。

---

## 面6：闸面独立重跑 🟢 绿

| 闸 | 实测输出 | 判定 |
|---|---|---|
| `python3 gates/cli.py check` | `PASS=13 FAIL=0 SKIP=0`，`EXIT=0`（输出存 `/tmp/rev_b_gates_check.txt`） | 🟢 |
| 其中 state-baseline | `[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同` | 🟢 |
| `python3 gates/cli.py ratchet`（单跑） | `[✓ PASS] test-ratchet — total=983(基线978) 无回退`，exit 0 | 🟢 |
| `python3 tools/field_ownership.py --check` | `[GREEN] 归属表 186/186 字段唯一归属（零双写/零缺口）`，EXIT=0 | 🟢 |
| `python3 tools/state_baseline.py --check`（独立单跑，不经 gates） | `[GREEN] 金样逐字节等价：74 场景 sha 全同`，`EXIT=0` | 🟢 |

**ratchet 口径澄清**：任务所称「ratchet 978」= **基线值 978**；当前实测 `total=983`。978→983 **恰 +5**，与本笔新增 5 个测试（game-core 4 + oracle 1）精确吻合，且 `data/status/baselines/baseline.json` **不在** 14 文件内（未擅自改基线，符合「基线只能确认无回退后更新」）。

---

## 面7：夹带 / plan 更正 / 警示归属 🟢 绿

### 7.1 `git show aeb6964 --name-only` = 恰 14 文件，全部在声称域

1. `crates/game-core/src/lib.rs`
2. `crates/game-core/src/scene.rs`
3. `crates/game-core/src/script_ops.rs`
4–12. `crates/game-core/src/subsystem/{battle,dialog,entity_service,input_mapper,item_service,mod,persist_codec,script_host,ui_render}.rs`（9 个）
13. `crates/game-oracle/tests/dump_face.rs`
14. `docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md`

⇒ subsystem 族 + scene/script_ops 可见性 + 测试 + plan，**零夹带**（无 data/、无 reference/、无门禁、无基线文件）。

### 7.2 plan `:63` 原地更正符合裁定格式 ✓

diff `@@ -64,6 +64,7 @@` **仅插入** 1 行：
`  > 【2026-11-07 更正】本段为批0 动作前状态记录；0c9080f 后已 [GREEN] 186/186。`
落在 **plan:67**，紧贴潜伏红段（段起 **plan:63** `- ⚠ **潜伏红披露**（复核 R3）：…现 **[RED]**`）尾部；**原主张未删**（diff 无删除行）⇒ 符合 AGENTS.md §3 `> 【日期 更正】` 证伪更正格式。commit 写「:63」= 段锚，准确。

### 7.3 「非本笔警示 10 条」= 如实录、且全在未触碰文件 ✓

`cargo check -p game-core -p game-oracle --release --all-targets` 实测输出 **`game-core` (lib test) generated 10 warnings**，10 条逐一核对文件：

- `menu_family.rs:1241 / :1873 / :1423`（unused `FontMetrics`）+ `menu_family.rs:2116`（fn never used）
- `scene_tests/fidelity_fix_tests.rs:1`、`scene_tests/entity_route_tests.rs:1`、`scene_tests/slot_equip_scene_tests.rs:1`（unused `FontMetrics`）、`slot_equip_scene_tests.rs:1079`（unused mut）、`t2_walkto_route_tests.rs:2594`（unused `cy0`）
- `controls.rs:360`（unused mut）

**10/10 全部落在本笔 14 文件之外**（既有警告），且本笔未声称「零警告」⇒ 属如实记录，非夹带、非新引入。
（补充：仓库全量 grep `警示` = 0 命中 —— 该词不在任何文档/代码中，故只能按上述 cargo 警告口径解释。）

---

## 发现清单（全部非阻断，建议批1–批2 消化）

| # | 面 | 级别 | 发现 | 建议 |
|---|---|---|---|---|
| F1 | 2 | 黄-轻 | `subsystem/mod.rs:17` 注释引用不存在的键 `entityBaseGold`（实存 `enemyBaseGold`） | 改注释键名；引用前 grep 验证（AGENTS.md §8） |
| F2 | 1 | 黄-轻 | spec:100 单行无 `deserialize`、spec:105 为 `snapshot(&self, …)`，实现分别为「+deserialize」与无参 —— plan §1.1/批4/批8 已裁定，但 **spec §3.1 未回注** | 在 spec §3.1 加 `> 【2026-11-07 更正】` 注记，说明 plan 授权的两处差异（不删原文） |
| F3 | 1/4 | 黄-轻 | `p2b0_trait_shapes_present` 只断言 8 个 trait **存在**，不锁 spec 签名；`p2b0_ctx_pure_data_constructible` 只构造 EntityCtx 且「无方法」无断言 | 补 `_sig` 辅助函数形如 `fn _sig<T: BattleSubsystem>(t:&mut T, ctx:&mut BattleCtx, x:EnemyTypeId) -> BattleOutcome { t.resolve_bump(ctx, x) }` 逐 trait 机器锁签名；补其余 Ctx 构造用例 |
| F4 | 1 | 黄-轻 | EntityService 的 `spawn/remove/relocate` 方法名只存于 doc 注释，无任何机器断言防漂移 | 批1 落终形时以编译期断言/测试钉住方法名清单 |

---

## 闸面与测试汇总（复核时点）

```
gates/cli.py check ........ PASS=13 FAIL=0 SKIP=0  EXIT=0
  ├ state-baseline ......... [GREEN] 金样逐字节等价：74 场景 sha 全同
  └ test-ratchet ........... total=983(基线978) 无回退
gates/cli.py ratchet ....... [✓ PASS] total=983(基线978)  exit=0
field_ownership.py --check . [GREEN] 归属表 186/186（零双写/零缺口） EXIT=0
game-core subsystem tests .. 4 passed / 0 failed（339 filtered）
game-oracle dump_face ...... 2 passed / 0 failed
变异攻击 ................... 2 红（exit 101×2）→ 还原后 6 测全绿、git diff 空
```

Confidence: 91/100
