# P2 批0 复核黄清（F1-F4 回派笔）对抗复核 — commit 5c23f71

- 被审件：`5c23f71`（`docs+test(core): P2 批0 复核黄清 F1-F3（键名勘误+spec 回注+判别力补强）`，2 文件 +106/-6）
- 分支：`modern`；复核员：对抗复核（只读，唯一写=本报告）
- 上游：`docs/findings/reviews/2026-11-07-p2-batch0-b-review.md`（0 红 4 黄-轻 F1-F4）
- 复核时间：2026-11-07 07:50–08:01（全部测量均在**干净工作树=5c23f71 原样**下取得；3 次变异逐一 `git checkout` 还原，还原后 `git diff` 字节数=0）

> ⚠ 并发环境说明：08:03–08:05 起，**另一 agent 在同一工作树落笔批1/F4**（`subsystem/mod.rs` 追加 EntityService 9 形 `_sig_entity_*` + `p2b1_entity_service_method_names`、`entity_service.rs`、`script.rs`、`tests/script_golden.rs`，均为 insert-only，mtime 08:03:26–08:05:29）。这些改动**晚于**本报告全部测量时点（测试 07:52:19、闸面 08:00:34），不属被审件 5c23f71，未纳入本报告判据；被审两文件中 spec 仍零 diff，`mod.rs` 现存 diff 全为该并发批1 追加（不触碰本笔的 :17 注释、sig_lock、5 Ctx 用例）。在途红绿由批1 自己的复核负责。

## 0. 总判

**可推送**（0 红 / 0 阻断黄）。F1、F2、F3 三面全部**绿**；F4 按原裁定随批1，本笔未夹带（=符合裁定）。夹带 2 文件精确、闸面独立重跑 13/13 + 金样 74 全同。仅 4 条不阻断观察（§6）。

| 面 | 判 | 一句话 |
|---|---|---|
| F1 键名 | 🟢 绿 | 全仓 `entityBaseGold` 代码残留=0；注释实键 `enemyBaseGold` 与 state_dump.rs:175 一致 |
| F2 spec 更正 | 🟢 绿 | 日期更正注记、0 删除行、原文 :100/:105 逐字在位；内容与 plan 批4/批8 实锚相符 |
| F3 判别力 | 🟢 绿 | subsystem 测试全绿；3 形变异攻击全红（含 Ctx 字段）；还原后 diff=0；5/5 Ctx 全构造 |
| 夹带+闸面 | 🟢 绿 | `--name-only` 恰 2 文件；`gates check` PASS=13 FAIL=0 + 金样 74 场景 sha 全同 |
| 批0 一致性 | 🟢 绿 | 无生产代码改动、无新假数、无范围蔓延；F4 未夹带 |

## 1. F1 注释键名（entityBaseGold→enemyBaseGold）🟢

- **残留 grep（全仓，含 docs/tests/源码，排除 .git）**：
  ```
  $ grep -rn "entityBaseGold" . | grep -v "^./.git/"
  ./docs/findings/reviews/2026-11-07-p2-batch0-b-review.md:110   ← 上游复核报告（历史引用，豁免面）
  ./docs/findings/reviews/2026-11-07-p2-batch0-b-review.md:264   ← 同上（F1 行）
  ```
  ⇒ **代码/注释/spec 残留 = 0**；仅剩历史报告引用，落在任务豁免面内。
- **实键核**：`crates/game-oracle/src/state_dump.rs:175`
  `push(e_int_arr("enemyBaseGold", &enemy_base_gold_derived));` —— 键名 `enemyBaseGold`，且为 dump 派生键。
- **改后注释**：`crates/game-core/src/subsystem/mod.rs:17`
  `//! ItemCtx=P1-2 effect_registry.rs:74 已存形态照录（不改史）；enemyBaseGold 等派生键不入（输出面非写面）。`
- **旁证（"派生键不入写面"成立）**：`grep enemyBaseGold|entityBaseGold data/status/field-ownership.json` → 0 命中（归属表确实不含该派生键）；`enemyBaseGold` 在 oracle 夹具/测试中多处使用（`crates/game-oracle/tests/fixture_frames.rs:837/866/998` 等），语义一致。

## 2. F2 spec §3.1 日期更正注记 🟢

- **不删原文（硬判据）**：
  ```
  $ git show 5c23f71 --format="" -- docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md | grep -c "^-[^-]"
  0
  ```
  ⇒ spec diff **零删除行**，全部为 `+` 新增注记（`@@ -104,6 +104,10 @@`）。
- **原文 :100/:105 逐字在位**（复核后文件实测）：
  - `docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md:100`
    `trait PersistCodec    { fn serialize(&self, st: &SceneState) -> Blob; }`
  - `:105` `trait UIRender        { fn snapshot(&self, …) -> UiFrame; }               // A 补：显式纯读`
- **注记格式合规**（AGENTS.md §3 证伪更正文化）：`:107-110`
  `> 【2026-11-07 更正】P2 批0 落码与本节两处差异（plan 已裁定，原文不删备查）：…以上为 plan 授权收窄/扩展，非偏离。`
- **内容与 plan 实锚相符**（`docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md`，`## 2. 切片批次` 在 :76 ⇒ "plan §2" 指针有效）：
  - 注① ↔ plan **:92 批4**：`PersistCodec：serialize/deserialize（Blob↔域切片格式编解码面；PersistState 行为面不入——spec ④）` ✓
  - 注② ↔ plan **:96 批8**：`UIRender（只读边界件…）：snapshot(&self)->UiFrame 纯读（E3 RenderState 消费面）` ✓
  - 实现侧核：`subsystem/persist_codec.rs:13-14` 有 `serialize`+`deserialize`；`subsystem/ui_render.rs:10` 为 `fn snapshot(&self) -> UiFrame;`（无参）——注记描述与代码一致。

## 3. F3 判别力（重点）🟢

### 3.1 测试全绿（独立重跑）
```
$ cargo test -p game-core --release subsystem ; echo exit=$?
exit=0
Running unittests src/lib.rs … running 5 tests
test subsystem::tests::p2b0_ctx_pure_data_constructible ... ok
test subsystem::tests::p2b0_ctx_pure_data_constructible_all ... ok
test subsystem::tests::p2b0_registry_rows_43 ... ok
test subsystem::tests::p2b0_trait_shapes_present ... ok
test subsystem::tests::p2b0_registry_rows_match_field_ownership ... ok
test result: ok. 5 passed; 0 failed; …
（全部 integration 目标亦 0 failed）
```

### 3.2 变异攻击（判别力实证；每次变异后还原并核 diff=0）

| # | 变异 | 结果 | 关键输出 |
|---|---|---|---|
| A | `battle.rs` `resolve_bump(&mut self, ctx, t)` → 追加 `extra: bool` | 🔴 必红 | `exit=101`；`error[E0061]: this method takes 3 arguments but 2 arguments were supplied`；`could not compile game-core (lib test)` |
| B | `ui_render.rs` `snapshot(&self)` → `snapshot(&self, frame_no: u32)` | 🔴 必红 | `exit=101`；`error[E0061]: this method takes 1 argument but 0 arguments were supplied` |
| C（加测） | `battle.rs` 删除 `pub show_battle_digits: &'a mut bool`（Ctx 字段） | 🔴 必红 | `exit=101`；`error[E0560]: struct battle::BattleCtx<'_> has no field named show_battle_digits` |

- 每次还原后：`git checkout -- <file>` → `git diff \| wc -c` = **0**（审毕 `git status --porcelain -- crates/` 空）。
- 机制确认：`sig_lock` 7 形位于 `#[cfg(test)] mod tests` 内（`mod.rs:149`），`#[allow(dead_code)]` 仅抑制未调用告警，**类型检查照跑**——变异 A/B 均在 `lib test` 编译期被抓 ⇒ "改签名即红" 成立。
- 7 形清单（`mod.rs:151-170`）与 spec §3.1 逐字对齐：`_sig_battle`(resolve_bump)、`_sig_dialog_tick`(tick)、`_sig_persist_ser`(serialize)、`_sig_persist_de`(deserialize)、`_sig_script_op`(execute_op)、`_sig_input_map`(map)、`_sig_ui_snapshot`(snapshot 无参)。

### 3.3 5 Ctx 全覆盖（读测试源确认）
- `grep "pub struct [A-Za-z]*Ctx" crates/game-core/src/` ⇒ **全仓恰 5 个 Ctx**：`entity_service.rs:12 EntityCtx`、`battle.rs:18 BattleCtx`、`dialog.rs:15 DialogCtx`、`script_host.rs:16 ScriptCtx`、`effect_registry.rs:74 ItemCtx`。
- 构造分布：
  - `EntityCtx` → `p2b0_ctx_pure_data_constructible`（`mod.rs:139`）三槽全填。
  - `BattleCtx`（10 写槽+3 读槽全填）、`DialogCtx`（记录槽+3 值读）、`ScriptCtx`（6 写槽+2 读槽）、`ItemCtx` → `p2b0_ctx_pure_data_constructible_all`（`mod.rs:224`）。
  - `ItemCtx` 用的是 `crate::effect_registry::ItemCtx`，**7 字段逐一对上 `effect_registry.rs:74-82`**（player/floor/tutorial_seen/equipped_weapon/equipped_armor/equip_types/equip_bonuses）——"照录不改史"属实（subsystem 侧无自有 ItemCtx，`item_service.rs` 为空壳 trait）。
- ⇒ "5 Ctx 全覆盖" 字面与实际相符（跨两个用例合计 5/5，且构造为结构体字面量，字段增删/改型即编译红，变异 C 已证）。

## 4. 夹带与闸面 🟢

- **文件数**：`git show --name-only --format="" 5c23f71 | grep -c .` = **2**
  - `crates/game-core/src/subsystem/mod.rs`
  - `docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md`
  恰为裁定的 2 文件，无夹带（工作树其余为历史 untracked 复核报告与 `android/build/`，与本笔无关）。
- **闸面独立重跑**：`python3 gates/cli.py check` → `exit=0`
  ```
  PASS=13 FAIL=0 SKIP=0
  [✓ PASS] rust-tests — cargo test --release 全绿
  [✓ PASS] test-ratchet — total=984(基线978) 无回退
  [✓ PASS] state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同
  ```
  ⇒ 13/13 + 金样 74 场景独立复现，与 commit message 自报一致。

## 5. 与批0 一致性（无新假数/无范围蔓延）🟢

- **无生产代码改动**：mod.rs diff 三个 hunk —— `@@ -14,7 +14,7 @@`（模块 doc 注释，F1）、`@@ -97,14 +97,16 @@`（`#[cfg(test)]` 内 use 导入）、`@@ -142,6 +144,100 @@`（`#[cfg(test)] mod tests` 新增两段）。无一处落在非 test 代码。
- **注释内新计数逐条核过**：BattleCtx「10 写槽+3 读槽」= `battle.rs:19-33`（10 `&mut` + 3 只读 ✓）；ScriptCtx「6 写槽+2 读槽」= `script_host.rs:17-24` ✓；DialogCtx「记录槽+3 值拷贝读」= `dialog.rs:16-19` ✓。未引入无法复算的数。
- **F4 未夹带（按裁定）**：mod.rs 中 EntityService 仅存于 `_shape` 泛型存在性界（`mod.rs:115`），**无** `spawn/remove/relocate` 方法名机器断言 ⇒ 与"F4 随批1 终形落"裁定一致，无提前/越权实现。
- **spec 注记仅陈述 plan 已裁定的两处差异**，未夹带其他"更正"；未动 `data/status/baselines/`（ratchet 只增不减由闸面自证）。

## 6. 遗留观察（不阻断，供批1/后续笔参考）

1. **`p2b0_ctx_pure_data_constructible_all` 的运行期 assert 基本空转**：`bctx.battle_active as *const bool as usize != 0`（有效引用恒真）、`dctx.key_value==0` / `sctx.script_delay==0` / `ictx.floor==1`（构造值恒真）。真正判别力在**编译期结构体字面量完整性**（变异 C 已证），运行期断言只是形式。建议后续改成"槽可达"型断言（如经 ctx 回写再读）或明示"构造即断言"。
2. **ItemService / EntityService 无 `_sig` 锁**：ItemService spec 形=`…未定形`（空壳 trait 属"不预设"合规），EntityService 方法名=F4 明示随批1 ⇒ 属裁定内缺口，非本笔失职；批1 落终形时须按 F4 补机器断言。
3. **`#[allow(dead_code)] mod sig_lock` 可被整体删除而测试仍绿**（任何测试删除都不会红）——本笔变异只证明"改签名即红"，防"删锁"仍靠 review。若要机器化，可在闸面加"sig_lock 7 fn 存在性"grep 判据（建议批1 一并议）。
4. **变异抽样 3/8 形**（Battle 参数、UIRender 参数、BattleCtx 字段）：其余 4 形（Dialog/PersistCodec×2/ScriptHost/InputMapper）机制同为编译期钉形，未逐一变异；残余风险低但非零。

Confidence: 93/100
