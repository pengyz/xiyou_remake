# 对抗复核报告（三轮）：solver-batch2 提案二轮修订验证

- **对象**：`docs/superpowers/proposals/2026-11-10-solver-batch2-proposal.md`（313 行，HEAD `5b8409a`，父 `eace26f`，分支 modern，+18/-15）
- **判据**：二轮报告 `docs/findings/reviews/2026-11-10-solver-batch2-proposal-review-r2.md`（红1=R2-1、黄1=Y2-1）原文
- **聚焦**：只验 ①R2-1 修法(a) 是否真落地、②Y2-1 是否真修、③有无引入新伤/夹带
- **方法**：sed/grep 行号实测、`git diff eace26f 5b8409a` 逐段核对、三理由与二轮证伪链逐条逻辑比对、门禁与金样只读独立重跑
- **范围**：只写本文件；未改任何代码/提案/计划

---

## 总判：**红 0 · 黄 1 · 绿 12 → 需小修（黄项随修一并处理）**

| 类 | 条目 |
|---|---|
| 红 0 | R2-1 修法(a) 六处核全部落地，二轮红项闭合 |
| 黄 1 | **Y3-1（新伤）**：§6-8（:263）新增残句「若 Exit#1 不可达则 **max_hp**/min_kills=None 而 reachable=true」——`max_hp=None` 半句与本件 §2.1（:113）「max_hp 含 Exit#2、不随 min_kills 收域」自相矛盾（详见 §五） |
| 绿 12 | 见 §一~§六 逐条 |

---

## 一、R2-1 修法(a) 落地核（全绿，6/6 处 + 断言面）

### 1. §1.4② 整段（:91）——绿
- 公式已改：`min_kills = min_{π 合法 ∧ sₙ.pos ∈ Exit#1} kills(π)`（旧式 `min_{π 合法}` grep = **0** 命中）。
- 三理由在位：(1) Path B 承认证伪链（黄钥(5,10)→门(6,9)→gate19→slot19 四腿 find_path 全通→(11,1) 换层，`kills(π)=0`，脚本内三战=T(s,a) 内部演化）；(2) 度量与答案在脚本域分叉（0=无主动交战≠杀0怪，实际三战三杀；Exit#1 域内两口径重合）；(3) 入域则有出口脚本的层 min_kills 恒 0 + 路线须为出层航点序列（计划 :321/:322，sed 实测 :321=「三个出口都出解…带路线」、:322=「路线输出为航点序列」）。
- 机制锚三枚 sed 全命中（见 §三）；「本件静态复测在位」属实。
- 收尾句「Exit#2 不因此作废：全额入 reachable 与 max_hp（:112）」——新文件 :112 = §2.1 Exit#2 条目，内部引指正确。

### 2. 量化域三分句——绿（实测 :113；任务引 :112 系旧文件行号，+1 漂移）
- `reachable` = Exit#1∪Exit#2 任一可达；`max_hp` = 全部合法路取最大（**含 Exit#2**，附 hp 直接观测/读于 pending_floor 置位时刻未跨层的论证）；`min_kills` = 只在终态∈Exit#1 取小。
- 「max_hp 不随 min_kills 收域」显式论证（计划 :321 对 max_hp 未限域；收域使「仅脚本可出层」层失真为 None）——与 T3(iii) 改写交叉引用一致。

### 3. 五处域注——绿（5/5）
| 处 | 实测行 | 域注原文 |
|---|---|---|
| §2.1 Exit#2 | :112 | 「该边入 `reachable`/`max_hp`，不入 `min_kills` 量化域」 |
| §2.2.4 | :126 | 「该终止边入 `reachable`/`max_hp`、不入 `min_kills` 量化域」 |
| §3.2-4（任务引 :174，实测 :175，同为 +1 漂移） | :175 | 「该边入 reachable/max_hp、不入 min_kills 量化域，§1.4 ②」 |
| T9 | :153 | 「终止边入 reachable/max_hp、不入 min_kills 量化（§1.4 ②）」 |
| C5 | :246 | 验收列「入 reachable/max_hp、不入 min_kills 量化，§1.4 ②」 |

### 4. 四处 ≥3 断言带 Exit#1 限定——绿（4/4）
- T6（:150）「（§3.2 实测下界；域=Exit#1，§1.4 ②）」；§3.2-5（:176）「（**Exit#1 域**——0-1 BFS 口径=步行出层…本下界在域内恒真，与 Path B 不矛盾）」；BR-6（:231）「（Exit#1 域，§1.4 ②）…Exit#1 域内 kills=实际杀怪，白走漏杀必被 ≥3 断言拦下」；§6-8（:263）「以上 `min_kills` 均为 Exit#1 域口径（§1.4 ②）」。
- 0-1 BFS=3 的口径与域自洽：39 格避敌区/78 格全通均为**步行到 (11,1)** 的性质，正是 Exit#1 域，与 Path B（kills=0，不入域）不再冲突——二轮红的矛盾链已被域裁定切断。

### 5. T3(iii) 改同域下界——绿
- 新文（:157）：「对每条终态 ∈ Exit#1 的合法路 π 有 `min_kills ≤ kills(π)`」+ 测试实例化（route_min_kills 取等号；`route_max_hp` 终态∈Exit#1 时满足、∈Exit#2 不参与）+ 旧式在 Path B 下为假 `3≤0` 的留痕指针（§8-9）。
- 旧式 grep：`min_kills ≤ kills(route_max_hp)` / `kills(route_max_hp)` 全文 = **0** 命中（除新文自身的域内变体不含该串）。
- 逻辑核：旧式取 route_max_hp（终态可∈Exit#2）时 kills 计数在脚本域失真 ⇒ 3≤0 假；改同域后该断言为 min 定义的直接推论，恒真且可测 ✓。

### 6. None 形状首跑核对——存在性绿，内容半错入黄（Y3-1）
- BR-6（:231）补「及 Exit#1 本身钥经济可达性（0-1 BFS=门全开口径）= T6 真层输出核对」；§6-8（:263）展开「若 Exit#1 不可达则 max_hp/min_kills=None 而 reachable=true，T6 形状断言首跑核对」——首跑核对**已落位**（任务所指核对项在位，实际展开在 §6-8 而非 BR-6 本行，两处互指）。
- 但其中 `max_hp=None` 半句为假，见 §五 黄 Y3-1。

---

## 二、三理由 vs 二轮 §三 证伪链兼容性（逐条，全兼容）

| 理由 | 与二轮 §三 的关系 | 判 |
|---|---|---|
| (1) 承认 Path B 存在、kills(π)=0 | **不反称 drive 不完成**：全文 grep「不完成/drive 不/无法完成/跑不通」= 0 命中；§8-9 明写 (c) 实跑证伪「不采纳…静态机制复测与 census 反支持 Path B 成立」——即按「drive 完成」的二轮链接受事实，只改量化域 | ✓ 兼容 |
| (2) 度量/答案脚本域分叉、Exit#1 域重合 | 恰是二轮「0 杀 vs 实际三杀」矛盾的**吸收**而非否认：0 留在 reachable/max_hp，min_kills 换域。Exit#1 域重合性经核成立——Exit#1 路必不踩 gate(6,9)（踩则 slot19 触发、终态落 Exit#2），故域内无脚本执行、每次 BumpEnemy 胜=一杀 | ✓ 自洽 |
| (3) 恒 0 退化 + 航点序列格式 | 后半（计划 :321/:322 路线=出层航点序列、脚本路止于触发格）sed 实测支撑 ✓；前半「凡有出口脚本的层 min_kills **恒 0**」系**过度概括**（门在强怪后时脚本路亦可 ≥1 杀）——附注，不计数：floor1 实证为 0 成立，且裁定不依赖该全称量词 | 附注 |
| **不坍塌 Exit#2/T9** | Exit#2 保留在 reachable/max_hp（:112/:113）；T9 保留「slot19 触发→终止边断言」只加域注（:153）；C5/§2.2.4 同步——二轮回派单要求的五处同步全部在位 | ✓ 未坍塌 |

**结论**：修法(a) 是「换量化域」而非「否认二轮证伪链」，二轮红的两说法二难（在域内 ⇒ 0<3 假；不在域内 ⇒ 四处声称坍塌）被「Exit#2 出域只出 min_kills」干净解开。

---

## 三、行号抽验（19 条全命中：新增锚 11 + 主锚 8）

**新增锚（二轮修订/commit message 引，全部 sed 实测在位）**：
1. `scene_script.rs:48-54` = `if !ok { disarm; if walk_phase != 5 { 清 phase/steps } }`——phase==5 保留步数，「保步数」注确 ✓
2. `scene.rs:3841-3843` = `if !battle_rerun_scene && allow_restep { try_step(facing) }`——胜利重步 ✓
3. `scene.rs:3255` = `self.advance_script_walk();`（Dialog 分支内）——「续走」✓ 精确命中
4. `scene.rs:3507` = 注「（tick_battle → try_step，T768）必须放行置 wp=1」——T768 注 ✓
5. `script_abs.rs:76` = `pub flags: Vec<bool>,`（Obs.flags）✓
6. `script_abs.rs:118` = `pub fn from_scene(sc: &GameScene) -> Self`（Snapshot 补 difficulty/floor）✓
7. `script_abs.rs:191` = `pub fn drive(sc: &mut GameScene, idx: usize) -> Obs`（同段 :208 `flags: sc.script_line_flags.clone()` 亦核）✓
8. `scene_persist.rs:386` = `self.persist.pending_floor = target;` ✓
9. 计划 `:321` = 「三个出口都出解：reachable / max_hp（带路线）/ min_kills（带路线）」——max_hp 未限域的引证成立 ✓
10. 计划 `:334` = TDD 起手式「杀不过则 Refused 不计杀」✓
11. `effect_registry.rs:122` = `(26, "Key", EffectParams::Key { yellow: true, … })`（Path B 黄钥前提）✓

**6 主锚复测**：`scene.rs:927` try_run_scene / `:931` gate 初值 `!script_line_flags[idx]` / `:3315` pub try_step / `:3386` 门臂 `1 | 2 | 3 =>` / `:4069` 踩格 `try_run_scene(…, true)` / `:1553` pub probe_step——**6/6 命中**（附 `:2046` RouteEngage、`scene_persist.rs:426-427` 入口特例亦复测命中）。

**HEAD 一致性**：`git diff --stat b976803 5b8409a` 恰 1 文件（提案本体 313 行新增），b976803 后无任何源码变更 ⇒ 提案「行号以 b976803 实测、二轮新增锚同 HEAD 复测」的声明成立。

---

## 四、Y2-1 处置核（绿）

- **C5 行（:246）新文**逐项对上二轮建议稿：「side_a drive + **Obs 读回** flags/stats/px/py/entities（`script_abs.rs:191`，flags 字段 `:76`）+ Snapshot 补 difficulty/floor（`:118`）+ **harness 读 `scene.persist.pending_floor`**（`scene_persist.rs:386`，**非 Snapshot/Obs 字段**）作终止边」——两处失准（Snapshot 读回、pending_floor 归属）均已纠正 ✓；验收列另带域注 ✓。
- **§8 勘误注条 9/10 在位**：条9（:312，R2-1 域裁定留痕，含三选一取 (a) 及 (b)(c) 弃取理由、旧式 3≤0 留痕）；条10（:313，C5 旧「Snapshot 读回」→ 新口径留痕）；标题行同步「条 1-8=一轮，条 9-10=二轮」（:302）✓。

---

## 五、新伤扫描：黄 Y3-1（新引入，1 条）

### Y3-1（黄）：§6-8（:263）「max_hp=None 而 reachable=true」与 §2.1 域裁定自相矛盾

- **原文**（5b8409a 新增，`git diff` 实证旧文无此句）：「若 Exit#1 不可达则 **max_hp/min_kills=None** 而 reachable=true，T6 形状断言首跑核对」。
- **矛盾链**：§2.1（:113）本轮刚裁定 `max_hp = 全部合法路上取最大（**含 Exit#2**）`，并以「收域反而使『仅脚本可出层』的层失真为 None」为由**拒绝** max_hp 收域。floor1 上 Exit#2（Path B）与 Exit#1 的钥经济相互独立（Path B 只需黄钥(5,10)+门(6,9)，0 杀 0 门阻实测）⇒ 若 Exit#1 钥经济失败，reachable=true 必由 Exit#2 贡献 ⇒ 正确输出应为 **`min_kills=None`、`max_hp=Some`（脚本路终态 hp）、reachable=true**。:263 的 `max_hp=None` 是旧全域口径的残句——恰好是本轮裁定明令禁止的「收域失真」。
- **连带**：与 T3(i)（:157 `reachable=false ⇒ 两 None`）合读会推出「reachable=true ⇔ 两 None」的假对偶；且 T6 形状硬断言 `"min_kills":<int>`，该情形下 null 会使 T6 首跑红——:263 已把首跑核对挂上（好），但未指出 T6 形状需备 None 变体，修句时宜一并写明。
- **不计红理由**：不使任何 TDD 断言在 floor1 实配下为假（floor1 双出口可达性待首跑，Path B 保底 Exit#2）；位于 §6 待复核注而非形式化/TDD 面；一处措辞即可修。
- **修法建议**：改「若 Exit#1 不可达则 `min_kills=None` 而 `reachable=true`、`max_hp=Some`（由 Exit#2 路给出，§2.1 不收域）；T6 形状断言（`min_kills:<int>`）此情形首跑核对」。

### 其他扫描（绿）
- 旧表述残留 grep：「在出口集合上取最优」=0、「不区分（两种出口）」=0、旧式 `min_{π 合法} kills`=0、`kills(route_max_hp)`=0、「Snapshot 读回」仅存 :313 勘误注留痕（允许）——**0 处正文残留**。
- 三理由 (3)「恒 0」过度概括 = 附注（§二），不计数。

---

## 六、夹带与闸（绿）

- **夹带**：`5b8409a` **恰 1 文件** = 提案本体（+18/-15）；commit message 逐项列 R2-1/Y2-1 处置与同步点，与 diff 实际改动面一一对应；无代码/计划/门禁改动。HEAD=5b8409a、父=eace26f ✓。
- **闸只读独立重跑**：`python3 gates/cli.py check` → **PASS=13 FAIL=0 SKIP=0**（含 `state-baseline [GREEN] 金样逐字节等价：74 场景 sha 全同`、`test-ratchet total=1046`、`rust-tests cargo test --release 全绿`、`state-machine-anchors 锚 6+边数全命中`）。
- **金样双确认**：独立复跑 `python3 tools/state_baseline.py --check` → **exit 0，`[GREEN] 金样逐字节等价：74 场景 sha 全同`**。

---

## closing

**总判：红 0 · 黄 1 · 绿 12 → 需小修（黄项随修一并处理）。**

**红项**：无。R2-1 修法(a) 真修——公式/三理由/机制锚（§1.4②）、量化域三分句（:113）、五处域注（:112/:126/:175/:153/:246）、四处 ≥3 断言域限定（:150/:176/:231/:263）、T3(iii) 同域下界（旧式 grep=0）、None 首跑核对（BR-6↔§6-8）全部落地；与二轮 §三 证伪链逐条兼容（不反称 drive 不完成、不坍塌 Exit#2/T9）。Y2-1 已修（C5 :246 + §8 条9/10）。行号抽验 19 条全命中、旧表述正文残留 0、5b8409a 恰 1 文件、闸 13/13 + 金样 74 独立双跑全绿。

**黄项（回派必答，1 条）**：
- **Y3-1**：§6-8（:263）新增残句「若 Exit#1 不可达则 max_hp/min_kills=None 而 reachable=true」——`max_hp=None` 与 §2.1（:113）「max_hp 含 Exit#2、不收域」直接矛盾（Path B 保底 ⇒ 该情形 max_hp=Some、仅 min_kills=None），系旧全域口径残句；修句时同步注明 T6 `min_kills:<int>` 形状在该情形的 None 变体首跑核对。
