# P2 批4 对抗复核报告 — commit b495fa5

> 被审件：`b495fa5` refactor(core)「P2 批4 PersistCodec」（2 文件 +250/-59：`crates/game-core/src/subsystem/persist_codec.rs` 新建 + `crates/game-core/src/scene_persist.rs`），modern 分支
> 复核方式：对抗复核（只读；唯一写=本报告 + 任务书授权的变异与还原）；证据 A/B 级带实测输出；纪律=AGENTS.md（保真=金样裁判；证伪更正文化=日期更正不删原文）
> 日期：2026-11-07 · 被审 HEAD=`b495fa5` · parent=`b7faf97`

## 总判：可推送（八面无红；面8 黄=贴件两修披露仓库内不可见）

**核心 A 级（字节布局保真）绿**：`PersistCoder::serialize` 新写手 vs 旧 `serialize_slot_blob`（`git show b495fa5^:…/scene_persist.rs`）逐字段顺序/宽度/字节序/截断链全同（7×u8 头→cell i16 BE→hp·atk·def i32 BE→三钥匙 i16→gold i32→alchemy i16→journal 计数 i16+对→128 旗标→栈计数 u8+对→56 层态 i16 长+块/None=-1i16）；`x as u8` 与旧 `(x as i8) as u8` 两补截断恒等。预存字节钉（`slot_equip_scene_tests.rs` `save_blob_layout_matches_java_and_roundtrips` 等，b495fa5 前存量）现过=新旧布局等价的独立实证。

**判据独立重跑全绿**：`p2b4` 3/3、`--lib` 352 passed/0 failed/2 ignored、`fixture_frames` 45/45（167.32s）、金样 `[GREEN] 金样逐字节等价：74 场景 sha 全同`、`gates/cli.py status` PASS=13 FAIL=0 SKIP=1（visual-diff P4 前恒 SKIP）、总闸 `gates/cli.py check` 见面7。**变异独立复做**：`cell_x to_be_bytes→to_le_bytes` → `p2b4_layout_offsets` 红文与贴件⑤逐字同（`cell_x BE i16 / left: [52, 18]`），`git checkout --` 还原 `git diff` 空、回原 3/3 绿。

**三条黄注（均不阻断）**：① 面1——commit 内 a.java 行锚 7 处对 HEAD 实文不解析（9192→实 9201 等），考古=存量漂移被复制（d389ddf 2026-10-16 补丁插行使 deobf 整体下移），语义内容逐条对上；② 面6——commit 称变异「**仅** p2b4_layout_offsets 红」实测 **2 红**（roundtrip 同红，夹具 cell_x=168≠0），判别力更强但「仅」字失实；③ 面8——reflog 证 commit 被 amend 两次且三树 byte-identical（纯 message 两修），仓库内无该披露/`--cleanup=verbatim` 教训记录，「【修复实录】无（一次成型）」仅对代码为真。

---

## 面1 字节布局保真（核心 A 级） — 🟢 绿（附黄注①）

**新写手 vs 旧写手逐字段对**（HEAD `persist_codec.rs:36-85` vs `git show b495fa5^:crates/game-core/src/scene_persist.rs` 内联 `serialize_slot_blob` 写体）：

| 段 | 旧（b495fa5^） | 新（persist_codec.rs） | 判 |
|---|---|---|---|
| 7×u8 头 | difficulty/current/min/max/facing/weapon/armor 逐 `push` | :38-44 同序同宽 | 全同 |
| cell | x,y `i16::to_be_bytes` | :46-47 | 全同 |
| hp/atk/def | 3×`i32::to_be_bytes` | :48-50 | 全同 |
| 三钥匙 | 3×`i16` | :51-53 | 全同 |
| gold | `i32` | :54 | 全同 |
| alchemy | `i16` | :56 | 全同 |
| journal | 计数 `i16`+逐对 `push` | :58-62 | 全同 |
| 旗标 | 恒 128 字节 | :64-128 `.get(k).unwrap_or(false)` | 全同 |
| 栈 | 计数 `u8`+对 | :69-73 | 全同 |
| 56 层态 | i16 长+块 / None=`-1i16` | :75-83 | 全同 |

**截断链**：旧 `(x as i8) as u8`（i32→i8→u8）与新 `x as u8`——两补截断恒等（低 8 位相同），值域全覆盖实证（fixture 含 0x1234/0x0A0B/0x01020304 等高位值，layout+roundtrip 双绿）。

**a.java 真源抽锚（3 处）——⚠ 黄注①：行锚对 HEAD 实文不解析（存量漂移）**：

```
$ grep -n "private void saveFloorState\|writeByte(this.equippedWeaponType)\|writeShort(this.playerCellX)\|writeShort(this.alchemyUpgrades)" reference/src/deobf/a.java
9187:   private void saveFloorState(int var1) {
9201:         this.f_DataOutputStream_00.writeByte(this.equippedWeaponType);
9203:         this.f_DataOutputStream_00.writeShort(this.playerCellX);
9212:         this.f_DataOutputStream_00.writeShort(this.alchemyUpgrades);
```

commit 引用 `a.java:9192/9193`（equip）、`9197-9198`（cell）、`9206`（alchemy）、`9207-9212`（journal）、`9182-9244`（写侧范围）、`9260-9308`（读侧）、`7163-7164`（note_max_floor 水位）——HEAD 实文分别为 9201/9202、9203-9204、9212、9213-9218、9187 起、`private boolean loadFloorSave`=9253 起、7165-7168（`if (var1 < this.minFloorReached)…maxFloorReached`）。**考古定性（`git log -S 'a.java:9192'` + 逐版 grep）**：`65cccb0`(2026-10-15) 时 deobf 中 equip 恰=9192（引用当时为真）→ `d389ddf`(2026-10-16) 在 ~6586 行 paint67 补丁插入使整体下移（equip→9201，净 +9）；本 commit 是**复制存量注释**而非新造，且仓内他处同类漂移并存（`save.rs` journal 引 9207-9211 实为 9215-9219、`scene.rs:578` 引 9257-9299 实为 9253 起）。**语义内容**（写序/字段归属）与 HEAD 实文逐条对上，字节布局结论不受影响；行号指针不解析属证据指针质量黄注（建议后续统一勘误，非本批阻断）。

**附注（黄）**：layout 断言消息里的算式 `7+2+2+12+6+2+2+2+128+1+56×2=278` 实算=276（漏 gold i32=4 的正确式=7+4+12+6+4+2+2+128+1+112=278）——**断言值 278 正确且绿**（独立验证：`b[165]`=栈计数、`b.len()==278` 过；旧存量钉 `len==37+2*2+128+1+56*2=282` 含 journal 对亦过），仅注释公式笔误。

## 面2 roundtrip 语义 — 🟢 绿

- **真逐字段、非松散相等**：`p2b4_roundtrip_floor_save`（`persist_codec.rs:117-149`）末行 `assert_eq!(back, st, "存读档往返夹具：serialize→save::parse_floor_save 逐字段相等")`——`FloorSave` derive `PartialEq`（`save.rs:33` `#[derive(Debug, Default, Clone, PartialEq)]`），派生实现逐字段比较全部 **23 字段**（含 `journal: Vec<(u8,u8)>`、`script_flags: Vec<bool>`、`floor_states: Vec<Option<Vec<u8>>>` 的逐元素比较）。变异实证其判别力：cell_x 扰动时该测试同红（见面6），diff 直接打印两完整 `FloorSave` 全字段差异。
- **deserialize 委托 panic 语义 vs 旧 handoff**：新 `persist_codec.rs:90` `crate::save::parse_floor_save(&blob.0, 0).expect("MOT_L 槽位记录可解析")` vs 旧消费者 `shell/common/src/handoff.rs:192` `game_core::save::parse_floor_save(blob, 0).unwrap()`——**同语义**：`Err` ⇒ panic（消息不同，均为 panic-on-malformed）。`save.rs:109` `parse_floor_save` 本体未被本 commit 改动（diff 恰 2 文件），读侧真源单源不变。

## 面3 SceneState alias 裁定 — 🟢 绿

- **spec/plan 裁定检索**：spec `docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md:100` `trait PersistCodec { fn serialize(&self, st: &SceneState) -> Blob; }`（注④ :120「只管格式编解码行为不入」）；plan `docs/superpowers/plans/2026-11-07-dddsa-p2-trait-plan.md:92` 批4「Blob↔域切片格式编解码面」。**两处均未规定 SceneState 为独立 struct**——批0 占位「批4 终形=PersistState 族切片集」的措辞在本 commit diff 中解释为「23 字段族实指 `save::FloorSave`」（`persist_codec.rs:6-9` 模块 doc 披露），零双 struct。
- **域切片意图符合性**：`FloorSave`（`save.rs:34-64` 实数 23 字段）= MOT_L 槽位记录的跨域切片（menu difficulty / persist floor 族 / entity facing·cell / player 数值·道具栈 / item 装备 / alchemy / script 旗标 / journal / 56 层态）——正是格式面需要的全部状态；行为面（load_floor/change_floor）留 scene_persist（spec ④）。
- **引用面 3 行实核**：`crates/game-core/src/subsystem/mod.rs:106`（use）、`:165`（`_sig_persist_ser` 签名锁）、`:168`（`_sig_persist_de` 签名锁）；另有 `scene_persist.rs:531-533`（scene_state 返回类型）。
- **23=23 对账**：`scene_state()`（`scene_persist.rs:531-572`）恰填 23 字段 vs `FloorSave` 23 字段——**对上**（声称 23 属实）。

## 面4 禁双轨核 — 🟢 绿

- **旧内联字节写真删**：`grep -n "to_be_bytes" crates/game-core/src/scene_persist.rs` = **0 行**；`grep -rn "o.push(self\." crates/` = 0；`serialize_slot_blob` 旧写体残留 grep = 0（仅薄适配 `:575` = `scene_state()` + `PersistCoder.serialize(&st).0`）。`impl PersistCodec` 全仓唯一（`persist_codec.rs:34`）。
- **单定义+调用面**：`serialize_slot_crates` 全仓仅 `scene_persist.rs:575` 定义 + 3 调用点（`scene_menu.rs:451`、`scene_tests/slot_equip_scene_tests.rs:1423/1448`）。
- **23=23 对账**（面3）：scene_state 23 字段 = FloorSave 23 字段，**对上**。
- **理论分歧（不可达，代码已注披露）**：`item_stack_size` 经 scene_state 截 u8 后按 u8 计数写对——若 >255 旧写全对数/新写截断数；实际 `item_types=vec![0;32]` 栈上限 32 不可达。`saved_floor_states` 恒 56（`scene_persist.rs:75` `vec![None; 56]` 初始化、`scene.rs:600` 由 parse(恰56) 覆盖）→ 新写 `.get(k)` vs 旧 `[k]` 索引 panic 分歧不可达（注释已标「安全化」）。

## 面5 盲区 4 对账 — 🟢 绿（附微注）

- **PERSIST_BLIND_ZONES 实文**（`persist_codec.rs:103-109`）5 条，前 4 条恰点名 minFloor/maxFloor/pendingFloor/wentDown，各带写点锚。
- **写点锚 sed 实测全对**：`scene_persist.rs:381` min_floor / `:383` max_floor / `:384` note_max_floor 水位 / `:386` pending_floor / `:387` went_down、setter `:35/:36/:37`、`script_ops.rs:67` set_pending_floor / `:95` set_min_floor / `:96` set_max_floor、`scene.rs:581/582` 载入——逐点 sed 命中。
- **plan 点名行**：plan:68 盲区清单标题、**plan:71** `persist（min/maxFloor/pendingFloor/wentDown）`——commit 称「plan:69」**行号偏 2**（微注；点名 4 项内容一致）。
- **点名机器断言真伪**：`p2b4_blind_zones_cover_plan_named`（`persist_codec.rs:203-209`）= `PERSIST_BLIND_ZONES.join("\n")` 后对 `["minFloor","maxFloor","pendingFloor","wentDown"]` 逐名 `contains` 断言——**真机器断言**（批3 红1 教训沿用），非文案自证。
- `a.java:7163-7164` 行锚漂移同黄注①（HEAD 实文 7165-7168，语义=min/maxFloorReached 写点对上）。

## 面6 TDD+变异 — 🟢 绿（附黄注②）

- **红档早于 commit**：`/tmp/ev_b4_red.txt` mtime=**2026-11-07 14:36:37** < 首次 commit（reflog `c56fc64` @15:10:25）< 两次 amend（15:27:26 / 15:37:12）；361 字节 5 行编译错与贴件①逐字同（E0425 `PersistCoder`×2 + E0425 `PERSIST_BLIND_ZONES` + E0560 `SceneState` 无 `difficulty_index`/`current_floor`）。贴件终态 `/tmp/ev_b4.txt`(15:09:17)、`/tmp/ev_b4_anchor.txt`(15:09:46) 均早于首次 commit。
- **变异独立复做（授权写）**：`persist_codec.rs:46` `st.player_cell_x.to_be_bytes()`→`to_le_bytes()`（加 `// MUTATION` 标记）：

```
---- subsystem::persist_codec::tests::p2b4_layout_offsets ----
assertion `left == right` failed: cell_x BE i16
  left: [52, 18]
 right: [18, 52]
test result: FAILED. 1 passed; 2 failed; 0 ignored; 1 filtered out   ← p2b4_roundtrip_floor_save 同红
```

  - layout 红文与贴件⑤**逐字同**（`cell_x BE i16 / left: [52, 18]`）✓；
  - ⚠ **黄注②**：commit 称「**仅** p2b4_layout_offsets 红」——实测 **2 红**（roundtrip 因夹具 `player_cell_x: 168`≠0 同红，diff 打印全字段差异）。判别力比声称更强，但「仅」字失实（贴件⑤未贴失败汇总行，无法对出其当次是否亦 2 红）。
- **还原核**：`git checkout -- crates/game-core/src/subsystem/persist_codec.rs` → `git diff --stat`=**空**、`git status --porcelain`（非 ??）=空 → 复跑 `p2b4` **3 passed; 0 failed**。变异-还原闭环完整。

## 面7 判据独立重跑 — 🟢 绿

| 判据 | 命令 | 独立结果 |
|---|---|---|
| p2b4 三测 | `cargo test -p game-core --lib p2b4` | **3 passed; 0 failed**（还原后复跑同） |
| game-core --lib | `cargo test -p game-core --lib` | **352 passed; 0 failed; 2 ignored** |
| 差分夹具 | `cargo test -p game-oracle --test fixture_frames` | **45 passed; 0 failed**（167.32s） |
| 金样（release） | `python3 tools/state_baseline.py --check` | **`[GREEN] 金样逐字节等价：74 场景 sha 全同`**（exit=0） |
| 门禁预检 | `python3 gates/cli.py status` | **PASS=13 FAIL=0 SKIP=1**（SKIP=visual-diff，P4 前恒定，status 显式打印） |
| 总闸 | `python3 gates/cli.py check` | 见下方补记 |

总闸实测（独立复跑）：

```
$ python3 gates/cli.py check
[✓ PASS] … rust-tests — cargo test --release 全绿
[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同
== 汇总 ==
PASS=13 FAIL=0 SKIP=0
exit=0
```

与 commit 声称「总闸 PASS=13 FAIL=0 SKIP=0」**逐字同**（口径差解释：`status` 打印的 SKIP=1=visual-diff 不入 check 清单，P4 前恒定，status 显式打印不静默——两者均如实）。七项判据全部独立重跑取得，**面7 绿**。

## 面8 夹带 — 🟡 黄（披露不充分；夹带/Coordinator 属实）

- **恰 2 文件**：`git show b495fa5 --name-status` = `M crates/game-core/src/scene_persist.rs`、`M crates/game-core/src/subsystem/persist_codec.rs`——**恰 2 文件，无夹带** ✓。工作树 tracked 全净（`git status --porcelain` 非 ?? 无项；`git diff b495fa5` 空），HEAD=b495fa5。
- **HostCtx N/A 披露属实**：`persist_codec.rs:15` 模块 doc「HostCtx 余 10 方法无存档消费——界收窄/实现者拆 N/A（批4）」+ commit 条6；核 `grep HostCtx` 无任何 persist 存档消费方法（impl 面=script/oracle 测试 stub）——**如实**。
- **Coordinator 遗留披露属实**：commit 条7——`scene_state()` 读 menu/persist/player/facing/equip/alchemy/script_line_flags（编排面读汇集非直写）✓；persist 域写点留 scene_persist（`change_floor :381-387`/setter `:35-37`/载入 `scene.rs:581-582`）= 行为面留 scene_persist（spec ④）✓——**披露与实际相符，非隐匿**。
- ⚠ **黄（贴件两修披露不充分）**：reflog 证 commit 两次 amend：`c56fc64`(15:10:25) → `714f4b2`(15:27:26) → `b495fa5`(15:37:12)；`git diff --stat` 三树 **byte-identical**（两修为**纯 message 修正**，代码零损）。但 commit message（含【修复实录】无（一次成型））与 `docs/` 全文 grep `verbatim|两修|cleanup=verbatim|amend` **零命中**——`--cleanup=verbatim` 贴件教训在仓库内**不可见**（若在会话侧披露则本注可降为绿注）。定性：代码面「一次成型」为真（三树同），message 两修未披露=证据披露不充分，**不构成夹带、不阻断推送**。

---

## 修复清单（推送前）

无必须项（零红）。建议（可随批5/顺手）：

1. **【建议·面1 黄注①】** a.java 行锚统一勘误（persist_codec.rs 内 9192/9197-9198/9206/9207-9212/9182-9244/9260-9308/7163-7164 → HEAD 实文 9201/9203-9204/9212/9213-9218/9187-9247/9253 起/7165-7168），或按证伪更正体例标「行锚随 d389ddf 下移」；`save.rs`/`scene.rs` 同类漂移一并登记。
2. **【建议·面1】** layout 断言消息算式勘误：`7+2+2+12+6+2+2+2+128+1+56×2=278` → `7+4+12+6+4+2+2+128+1+56×2=278`（断言值不变）。
3. **【建议·面6 黄注②】** commit/message 口径更正：变异实为 layout+roundtrip 双红（「仅」→「layout 判别红（roundtrip 同红）」）。
4. **【建议·面8】** 贴件两修（reflog 双 amend，树零变）与 `--cleanup=verbatim` 教训入库（docs/knowledge 或本报告留档）。

Confidence: 89/100
