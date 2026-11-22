# S2 外层骨架 F2 修复 定向复核报告

**日期**：2026-11-22
**复核者**：独立定向复核 agent（不参与实现；只验五点，不重开面）
**被告**：commit `6f63e14`（父 `15dd40c` + `b147e62`；注：本地 HEAD `5033e3a` 与其树完全一致，系 amend 产生的双哈希，内容同一）
**纪律**：只写本文件；未 commit；变异实验在 `/tmp/rev24`（`git archive 6f63e14` 副本 + assets 软链）

## 计数：绿 4 / 黄 1 / 红 0 —— **可推**

---

## 1. F2 修复正确性 —— 🟢 绿

**① 单一真相源核对**（`crates/game-core/src/rules/sensitivity.rs:75`）：
`counter_loss` 由 `predict_battle_hp_loss(eff, def, e.hp, e.atk, e.def, true)` 直算导出，
公式全部委托 `crates/game-core/src/combat.rs:106-131`（`predict_battle_hp_loss`，
a.java:9981-9993 忠实移植），sensitivity.rs 内**零复制公式**。
`outer.rs:845-861` 的 `anchor_killable(enemies, boss_ty, atk, def, hp, held)` 只消费表行：

```rust
let killable = r.can_kill && r.counter_loss >= 0;
let fatal = killable && r.counter_loss >= hp;
```

**② 公式独立重算**（Python 逐行复刻 combat.rs 公式，boss75 hp=100 atk=65 def=15）：

| 输入 | loss | (killable, fatal) | 期望 | 判 |
|---|---|---|---|---|
| atk=16 def=13 hp=5200 | (65−13)×99 = **5148** < 5200 | (true, **false**) | (true,false) | ✓ |
| atk=16 def=12 hp=5200 | (65−12)×99 = **5247** ≥ 5200 | (true, **true**) | (true,true) | ✓ |
| atk=10 def=13 | eff≤def15 ⇒ **−1** | (false, false) | (false,·) | ✓ |

**③ 变异**（`/tmp/rev24`，重编译实跑）：
- **M-A**：`fatal` 恒 false ⇒ `anchor_killable_requires_survivable_counter_loss` **FAILED** ✓；
- **M-B**：删 `counter_loss >= 0` 项 ⇒ 测试**仍绿（不红）**。逐分支证明这是**行为等价变异**：
  `loss = −1 ⇔ eff ≤ edef`，与 `can_kill = eff > edef` 互斥，故 `can_kill` 已蕴含
  `counter_loss ≥ 0`——该条款是防御性冗余，删不掉任何行为，测试抓不到属正常而非漏网。

**Confidence: 97**

## 2. 旗舰数字 —— 🟢 绿

- 被告产物 `/tmp/outer_seg1_v2.json`（2026-11-22 17:38）：frontier **1095**、killable **53**、
  fatal **8**、存活 **45**、最短存活击杀 **steps=66 atk=16 def=13 hp=5200** ✓；
- 因 json 时间戳早于 commit（17:52），用 `/tmp/rev24` 从 `6f63e14` 原码**从头重跑**
  `solver outer --max-pops 800000`：**四个数字逐一复现**（1095/53/8/45，min alive 66/16/13/5200）；
- fatal 抽查 2 条对敏感表手算：
  - steps=64 atk16/def12/hp5200 ⇒ 损血 5247 ≥ 5200 fatal ✓；
  - steps=69 atk16/def14/hp4550 ⇒ 损血 (65−14)×99=5049 ≥ 4550 fatal ✓。

**Confidence: 97**

## 3. F6 —— 🟢 绿（一处文档指针小疵，不计黄）

- `dominated_seen` 的 **steps 参数确已删**（`outer.rs:786-789`，签名只剩 seen/state）；
- doc 注释写明前置判据：「支配剪枝不含步数维，健全性依赖『所有转移成本 ≥1，
  堆按 steps 弹出 ⇒ 归档条目 steps 必 ≤ 当前』。加 0 成本边前必须先改这里」✓；
- §5.2 已更正为 `10×1.03^256 ≈ 19300`（原 ~3900 加更正注）✓。
- 小疵：findings 两处写「未交付项**见 §5.7**」，但文档**不存在 §5.7 标题**——三项实际
  落在 §5 第 9 条。指针悬空，语义可循（同文档内可找到），不误导。

**Confidence: 95**

## 4. F3/F4 —— 🟢 绿

- **F3**：§5.1 新增敏感性段——「1095/1095 条全带 optimistic_gate 注记；收紧 sealed/script
  门后锚可达性**未实测**，显式声明为待 S3 验证的乐观下界；S3 若消费 52/66 步当真值
  必须先跑收紧版对比」——如实，未把乐观下界伪装实测 ✓；
- **F4**：§5 新增边界 **7**（跨层回拾未建，引 m04867，列 S3 验收）、**8**（外层不结算
  打怪：金/属性/钥匙全零、hp 只增，前沿系统性乐观，列 S3 验收）、**9**（LB 下界表未建 /
  H5 前沿宽度未跑 / 饱和阈值闭式未做，列 S3 验收清单）✓；
- 原超claim「全部判据已证实」已在文首**原地加日期更正注**（【2026-11-22 复核更正】），
  不删历史 ✓；§6 旗舰段亦原地更正（steps=64 → steps=66 存活口径）✓。

**Confidence: 96**

## 5. 门禁与夹带 —— 🟢 绿

- `python3 gates/cli.py check` 复跑：**PASS=13 FAIL=0 SKIP=0**（与 commit message 一致）；
  state-baseline 绿（金样 74 场景 sha 全同）——ratchet 无虚报；
- `git show 6f63e14 --name-status`：仅 3 文件（`solver.rs` / `outer.rs` /
  `docs/findings/2026-11-20-outer-skeleton-s2.md`），**无越界**；
- 工作区唯一未跟踪文件是上一轮复核报告（本报告的前身），非本 commit 夹带。

**Confidence: 96**

---

## 一句话结论

**能推**：F2 修复三路独立验证（公式重算/真数据重跑复现/致命变异 M-A 红）全过，
旗舰数字与文档修订如实；唯一黄点是变异 M-B 按「判据必红」口径不成立——但可证明
`counter_loss>=0` 是被 `can_kill` 蕴含的防御性冗余（删除为行为等价），非正确性缺陷，
另有 §5.7 悬空指针一处小疵，均不阻断。

## 附：复核产物（/tmp，不入库）

- `/tmp/rev24/`：6f63e14 archive 副本（M-A/M-B 变异 + solver 重跑 `outer_rerun.json`）
- `/tmp/rev24-target/`：隔离 cargo target
