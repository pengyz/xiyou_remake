# DDDSA 架构设计 v1 对抗复核 B（验证/保真兼容视角）

> 2026-11-03。攻击对象：[DDDSA 架构设计 v1](../../superpowers/specs/2026-11-03-dddsa-architecture-design.md)。
> 角色：对抗复核员 B（remake-verifier 纪律）——验证兼容/保真红线攻击面。
> 证据基准：`crates/game-oracle/src/state_dump.rs`（dump 面实测）、`data/status/baselines/state-dump-golden.json`（60 场景清单）、
> `crates/game-oracle/tests/fixtures/`（31 帧夹具实证）、`crates/game-core/src/{scene,item_effect}.rs`（拾取/消费写集）、
> `gates/cli.py:404-420`（保真三表哈希闸）、知识库 `decision_asset-replacement-via-mod-channel` /
> `gotcha_dual-porting-surfaces-drift`、`docs/spec/evidence-todos.md` R-TODO-1b、
> 2026-11-02 两份对抗复核（R3.1 一次性切 / R5 验证预算实测）。

## 1. 攻击点 1：金样护航充分性（§5）——"每步金样 60 全同"对 P1 拾取写集是盲的

**攻击**：§5/§6.1 以"金样 60 场景字节等价每步实证"为 P1 主判据。实测 dump 面后裁定：**不够**——
P1 的核心行为面（`pickup_item_type` 拾取链 + `add_item_to_stack` 入栈）恰好落在 state_dump 的盲区。

### 1.1 dump 面清点（A 级，state_dump.rs 实测）

`dump_scene` 现行 push 序 = 标量 38 行（[state_dump.rs:75-112](../../../crates/game-oracle/src/state_dump.rs)）
+ 数组 11 行（:137-147）= **49 项**，外加 E3 扩面的 paint 副作用 **4 字段**（:114-120，2026-11-02 lead 裁准）。
逐字段对照拾取写集（scene.rs:4225-4321）：

| pickup_item_type 写集 | dump 覆盖 | 说明 |
|---|---|---|
| `yellow/red/blueKeyCount`（型 26-28） | ✅ | fx-boss68a/b 帧夹具已锚 |
| `player.atk/def`（29/30）、`hp`（31/32）、`gold` | ✅ | 数值臂可见 |
| `equippedWeapon/ArmorType`（33-40/79/80） | ✅ | try_equip 成功臂可见 |
| overlay 文案/时机（show_overlay_message） | ⚠️ 半覆盖 | overlayActive/Kind/Page/Phase 在 dump，**文案本体不在** |
| **`item_types[]/item_uses[]/item_stack_size`**（13-25/85/86 入栈） | ❌ **盲** | `grep item_stack\|item_types\|item_uses state_dump.rs` = 0 命中（A 级） |
| `menu.overlay_kind_item`（描述图标源） | ❌ 盲 | scene.rs:4227/4319 双写点 |
| `menu.tutorial_seen`（型 15 置位） | ❌ 盲 | scene.rs:4232 |
| `script_line_flags[]`（遗言收尾等） | ❌ 盲 | fx-boss74a 靠夹具内断言兜 |

即：**拾取→入栈、消耗→减 uses/栈压实（scene.rs:2384-2407）、叠加 uses（:4313-4314）三条写路径
金样 60 全同全盲**。dispatch 化若把 ITEM_USES 表或压实逻辑搬错，只要实体/数值面不动，金样照样绿。

### 1.2 31 帧夹具拾取覆盖实证（A 级，fixture 清单）

帧哈希夹具 = 31 个 `*.frames.txt`（27 game-oracle + 4 game-core），锚=Java 帧 sha 逐 tick
（FrameRec{tick, sha, input}，fixture_frames.rs:70-86）——凡有夹具的路径是强锚。拾取面覆盖：

| 拾取臂 | 夹具实证 | 状态 |
|---|---|---|
| 型 14 入栈→面板→使用全链 | preset-floor5（type-closure.md:67「道具14 全链 T1-1200」） | ✅ |
| 型 17 入栈（金勺子） | fixture-plan.md:261「T220 拾取后 `itemStackTypes[0]=17`、uses=[1]」+fx-boss67a | ✅ 但断言在夹具内 Rust 断言，非 dump |
| 型 26/27/28 钥匙 | fx-boss68a/b（redKeyCount 0→1→2，fixture-plan-boss.md:141） | ✅ |
| 型 32 长寿丹 | pre-final-audit.md:122 自认「🎨 出现未拾，需补拾取夹具」 | ❌ 缺 |
| 型 29/30/31、33-40/79/80、13/15/16/18-25/85/86 入栈臂 | 无专属拾取夹具（fx-item* 全是**消费**臂；preset-item* 是 MOT_L0 **预置栈**非拾取） | ❌ 缺 |

**关键区分**：60 金样场景里 preset-item13/itemstack/item21/item2122/itemcymbal/itemboss40 等
是「预置栈→使用」链，不经 `add_item_to_stack`；拾取入栈路径的夹具只有 preset-floor5 + fx-boss67a 两条。

### 1.3 裁定

1. **P1 开工前补 dump 盲区**（择一，须写入设计书）：
   a. `state_dump` 追加 `itemStackTypes[]/itemStackUses[]/itemStackSize/overlayKindItem/tutorialSeen`
      五行——注意 A2 行序冻结协议：行序追加=金样全红，属预期，需 lead 裁准重冻结（E3 先例照抄）；
   b. 或显式裁定这五字段不进 dump，但必须列「金样盲区字段清单」并以**夹具内断言**逐字段兜底。
   禁止默认（现状）：无清单、无断言、绿了就当覆盖。
2. **P1 每个 dispatch 臂至少一条拾取夹具**（Java 帧锚）或等价单测（红线 4 的前置——
   「新 EffectKey=新测试」必须先补「旧 EffectKey 也有测试」）；型 32「出现未拾」按
   pre-final-audit.md:122 行动项销账。
3. §5「金样 60 全同」措辞钉死口径：**每步=state_baseline --check（60 场景 dump sha，4min）；
   步末=Java 三方 47 场景（18min）**——两套是不同面（R5 口径，2026-11-02-feasibility §3.3 实测），
   P1 判据须两者全绿，不能只跑前者。

## 2. 攻击点 2：保真面隔离（§6.5 两份制）——items.json 内建表归属裁定

**攻击**：§6.5/红线 5 说「编辑内容走 mod 通道（id≥1000，金样禁 mod 表）」，但 §2.1 的 items.json
同时承载**内建表**（原版道具行）。内建行算保真表还是 mod 表？设计书没裁。混淆的直接后果：
哈希闸红（合法 mod 改动触发保真基线）或金样污染（改原版道具数值不过勘误流程）。

**归属裁定（清晰版）**：

| 行 | 归属 | 闸 | 变更流程 |
|---|---|---|---|
| items.json 内建行（id<1000，即 a.java itemDescriptions/ITEM_USES/pickupItemType 的表化事实） | **保真表** | 进 `FIDELITY_TABLES`/table-content-hash 闸（gates/cli.py:405 现只记三表，**扩到 items.json**）+ 金样 60 重过 | 勘误流程（逐值锚更新+金样重过+基线单独 commit），等同 data/tables 三表 |
| items.json mod 行（id≥1000 / mod: 前缀） | **mod 表** | mod 独立基线（mod_baseline.json，不进金样闸） | 自由改，金样运行禁装载 |

依据（A/B 级）：`decision_asset-replacement-via-mod-channel` 红线 1「保真表=原版事实，table-content-hash
闸护航；新内容挂 mod 侧，金样运行禁 mod 表、mod 独立基线」；P1 本质是把**代码里的保真事实**
（ITEM_USES[22] 表 scene.rs:4307、itemDescriptions[22] scene.rs:4326+、pickup match 数值臂）
搬进 items.json——事实换了宿主，保真属性不变。

**三条硬要求**：

1. **分文件不混装**：内建行与 mod 行**不许同一物理文件**（`items.json` 保真 + `items_mod.json` mod），
   或至少哈希闸只锁「内建行规范化子集」。混装单文件=mod 哈希波动触发保真闸红/或反过来
   为消红而豁免 mod 段=闸被掏空——两头都是自毁。
2. **红线 5 重述**：「原版保真面不走编辑器数据面」的正确语义=保真面**不装载 mod 行**；
   但保真面**必须**读内建 items.json（否则 §2.2 dispatch 化=classic 留 match、mod 走注册表
   =双轨，见 §4）。§1 架构图注「classic 保真面不走此层」措辞误导，须改为「classic 不走 mod 装载层」。
3. **§4「改表不改行为可证」限缩**：仅对 name/desc/icon 等展示列成立；effect/params/uses 列
   **就是行为**，改=行为变更，必红金样/夹具。设计书须把 items.json 列分为「展示列（可自由改）
   vs 行为列（勘误流程）」两类标注，编辑器 P3 的「改行为=表行变更显式登记」（§4 安全栏）与此对齐。

## 3. 攻击点 3：注册表行为等价（§2.2）——"逐字抽象"的机器证明面

**攻击**：§2.2「内建效果=现 match 语义逐字抽象（a.java:7528-7620 逐值锚，ItemAction 九 case 先例）」
——一句注释锚不是机器证明。拆两问：①ItemAction 先例实际是怎么证的、够不够格当 P1 的复用依据；
②未知 effect 语义与 R-TODO-1b 的对齐是**错配**。

### 3.1 ItemAction 先例的证据解剖（A 级，item_effect.rs）

先例形态：`ITEM_EFFECTS` 10 行表（13/14/16/17/18/85/19/86/20/21）→ 7 个 `ItemAction` 变体，
「执行序=表序，行为面零差」（item_effect.rs:6-7）+ C1 判据「新增道具不改 GameScene 主体（git diff 断言）」。
其等价证据=**三层回归等价**，不是形式证明：

1. 逐 case 注释锚（a.java:7786-7886 每臂行号，人工但可回指）；
2. 逐 case 夹具（fx-item17a/b、18a、19a、20a/b + preset-floor5（13/14）+ preset-item21/2122
   （消耗臂 20/21）+ preset-itemcymbal/itemboss40）——Java 帧 sha 逐 tick（A 级）；
3. 金样 60 state_baseline dump sha。

**教训**：机器证明的强度=夹具对该臂的覆盖度，不是注释锚的数量。16（三昧真火）/86（KillAdjacent4）
等臂至今无专属帧夹具（fixture 清单实证），等价只靠人工对照兜底——先例本身留着尾巴。

### 3.2 P1 复用要求

1. pickup dispatch 化必须凑齐同三层：①a.java:7525-7621 + add_item_to_stack 7721-7734 逐值锚注释；
   ②**每 dispatch 臂一条 Java 帧锚夹具**（§1.3-2 的补件）；③金样 60 全同（先补 §1.3-1 盲区）。
2. ITEM_USES[22]/itemDescriptions[22] 两张常量表进 items.json 后=数值表，须逐值锚
   （tools/table-evidence 生成式，220 逐值锚宿主同款）+ table-content-hash 闸扩面（§2）。
3. 「220 行 match→一行 dispatch」的等价判据=**行为面零差**（三层回归）+ §5.1 git diff 断言
   （新 EffectKey 零碰旧码），不是「代码短了」。

### 3.3 未知 effect 语义：R-TODO-1b 对齐是跨域错配（红）

§2.2「未知 effect=…登记-不执行（运行期，对齐 R-TODO-1b 静默停摆语义）」——**错**：

| 域 | 原版未知语义 | 证据 |
|---|---|---|
| 脚本 VM 未知 op | **停摆**：游标不动、dialogPhase 不改、每拍原地重解析=实打实卡死 | evidence-todos.md:139-141（R-TODO-1b） |
| item 域未知类型 | **default return**：无副作用、不消耗、不冻结 | scene.rs:4275 `_ => {} // 41+ 怪物类型不可达（a.java:7608-7620 default return）`；消费侧「未知型=不消耗（原 _ => false 语义）」item_effect.rs:152 |

**裁定**：运行期未知 effect=Java default 臂语义（no-op、不消耗、游戏不冻结）；「登记」只许是
可观测性输出，不得改行为面。加载期（classic 保真面）未知 EffectKey=拒载（五类拒载同严，红线 3）。
**禁止**把 R-TODO-1b 停摆语义套进 item 域——那会把「静默放过」改成「卡死」，直接击穿保真。

## 4. 攻击点 4：迁移期双轨（P1-P3）——一次性切裁定

**攻击**：P1-P3 期间 items.json 与残余 match 并存吗？设计书没说。若双轨（表+match 并行、
对照期互验），就是知识库先例的最坏形态：

- `gotcha_dual-porting-surfaces-drift`：同一 Java 方法两套移植面**各自绿、编译绿**，
  漂移只在跨面对照时暴露——DC-9 桥漏道具二倍加成=已发生实例；
- 2026-11-02-plan-attack-contract.md R3.1：SoA→struct **一次性切、禁双写对照期**，
  先例 commit `313375b`（427 站点一次落毕、金样零漂移）。

**裁定**：

1. **逐臂一次性切，禁双源**：同一道具 id 的行为真相只允许一份（items.json 行 + ItemEffect 实现）。
   match 臂删除与表行生效**同一 commit**；回退=整步 revert（见 §5）。P1 内部可按 dispatch 臂
   分多个原子步（每步金样绿），但每臂切完即删旧分支——**禁止**「表和 match 同时生效对照」。
2. **P1-P3 跨阶段并存是部分并存（不同 id 未迁移），不是双轨**：允许「已迁移 id 走表、
   未迁移 id 走 match」的分批形态；违规形态是「同一 id 两边都判」。
3. **§1 图注矛盾点名**：「classic 保真面不走此层」若按字面执行 ⇒ classic 留 match、mod 走注册表
   =永久双轨，P1「金样 60 全同」根本无从谈起（金样跑的就是 classic 面）。必须裁定
   **注册表服务两面**：保真面装载内建行（勘误流程管），mod 面加装 id≥1000 行；classic 不走的
   是 mod 装载，不是注册表本身（呼应 §2-2）。

## 5. 攻击点 5：验证预算与回退设计

**攻击**：§5 估算 3-5d/5-8d/5-8d 只算了实现面。验证成本实测已有（2026-11-02-feasibility R5）：
每 commit=单测 13s；每步=state_baseline --check **4min**；步末/里程碑=Java 三方 47 场景 **18min**
+ gates all。配比核算：

| 阶段 | 实现估 | 验证增量（本视角要求） | 合计裁定 |
|---|---|---|---|
| P1 | 3-5d | dump 盲区补面（A2 重冻结 lead 裁准）+ 拾取夹具补缺（§1.3）+ 每 EffectKey 新测试（红线 4）+ 勘误流程接入 items.json 哈希闸（§2）≈ **+1-2d** | 3-5d **顶格成立**，前提：effect 集冻结为现 match 全集不长新效果（复核 C 同款条件） |
| P2 | 5-8d | interact_with_cell 208 hits 跨 7 域收敛的回归面 + C2 终值复测 ≈ +1-2d | 5-8d 顶格 |
| P3 | 5-8d | mod 独立基线 + 零改码演练（tools/zero-code-drill 已机器化 15/15）+ 金样禁 mod 验证 ≈ +1d | 5-8d 可行 |

「每步金样」若误指 18min 套件：~10 步=3h 光验证，挤压实现（R5 已裁：每步=4min 前者）。
设计书须把 R5 分层口径**原样写进 §5**，「金样 60 全同」注明=state_baseline 60 场景。

**回退设计（现缺，必须补）**：

1. **整步 revert 纪律沿用**：每步=一个原子 commit（实现+测试+金样绿同时落）；步末 18min Java
   三方红 ⇒ **整步 git revert**（连新增测试一起回，禁只回实现留新测试=恒红），按 §4 三问定位层
   后再重做——不许调阈值/加白名单掩盖（verifier 红线）。
2. **基线 ratchet 只升不降**：state-dump-golden.json/table_content_hash.json 更新必须单独 commit
   写明原因（现行 state_baseline 协议沿用，A2 行序变更=红→lead 裁准重冻结，E3 先例）。
3. **回退单位=P1/P2 内部原子步**，不是整个阶段——阶段级 revert 成本（5-8d）不可接受，
   步级 revert（≤0.5d）才可操作。

## 6. 攻击点 6：§7 七问验证视角逐问裁定

| # | 问题 | 验证视角裁定 |
|---|---|---|
| 1 | §2.2 未知 effect 语义 | **加载期拒载**（classic 面 EffectKey 未注册=五类拒载同严）；**运行期 no-op 不消耗**=Java default 臂（a.java:7608-7620 / activate `_=>false`）；否决「R-TODO-1b 停摆对齐」（跨域错配，§3.3）。登记=可观测性，不得改行为 |
| 2 | Ctx 与 too-many-args 张力 | Ctx 聚合面可采纳，验证判据=行为面零差（金样+帧夹具逐 tick）；要求 Ctx 字段**显式列写集清单**与 state_dump FLD 对照，禁止隐式膨胀吞字段（DC-9 桥教训）；too-many-args 6/8 处永久保留不触=红线不动 |
| 3 | trait 五件边界 | battle 只出 BattleOutcome（数值+枚举），弹字/浮层/popup 时机（补遗#1 事件序断言）归编排层翻译成 UI 调用——UI 切在编排层，否则事件序不可断言。漏项补：PersistCodec 须含版本/校验和字段；ItemService::pickup 覆盖 26-40/79/80 全魔法区间（不只栈道具 13-25） |
| 4 | EnemyRegistry 入 P1？ | **否，后置 P2 后/P3 前**。P1 验证面已满（dump 盲区+夹具补缺+每 EffectKey 测试）；89 型属性行的 fixture 覆盖比 item 域更弱，入 P1=金样护航稀释。数据表化（属性行）可以先行，**行为钩子**注册后置 |
| 5 | 编辑器形态 | 本视角不裁形态（见复核 C 裁定：本地服务+真校验器），唯一要求=落盘 JSON 一律过真校验器（Rust 真值唯一）+ 保真行哈希闸 + mod 基线独立 + 金样禁 mod 装载，四件缺一不可 |
| 6 | 注册表运行期 vs 加载期 | **数据期动态**（加载期注册 EffectKey+数据表装载，schema 可机器判定）；**否决** so 插件级运行期动态=向金样/trace 面外注入行为=新增不可复现源，保真期红线 |
| 7 | 三阶段顺序与估算 | 顺序 OK（数据面→通信面→编辑器）；3-5/5-8/5-8 顶格成立但须补验证前置件 +1-2d/阶段（§5 表）；「每步金样」按 R5 口径钉死=state_baseline 4min，步末 47 场景 18min+gates all |

## 7. 结论：P1 开工前置件清单（gate——缺一不开工）

1. **dump 盲区裁定落书**：item 栈三字段+overlayKindItem+tutorialSeen 进 dump（A2 重冻结），
   或显式盲区清单+夹具断言兜底（§1.3-1）；
2. **拾取夹具补缺**：每 dispatch 臂一条 Java 帧锚夹具/等价单测；销 pre-final-audit「型 32 未拾」；
3. **items.json 归属落书**：内建行=保真表（哈希闸扩面+勘误流程）、mod 行分文件（id≥1000），
   红线 5 重述为「不走 mod 装载」（§2）；
4. **未知 effect 语义改稿**：删「R-TODO-1b 停摆对齐」，改 default 臂 no-op 不消耗（§3.3）；
5. **一次性切纪律落书**：逐臂原子 commit、禁同 id 双源、整步 revert 纪律+基线 ratchet 协议（§4/§5）；
6. **§5 预算口径改稿**：R5 三层（13s/4min/18min）原样写入，验证前置件 +1-2d/阶段。

**总评**：架构分层（§0/§1）与 ItemAction 表驱动路线在验证视角成立；但 v1 把「金样 60 全同」
当万能护航，恰恰对 P1 的核心写集（道具栈）是盲的——上述 6 件前置件补齐前，P1 判据
「金样 60 全同」是**弱保证**，不足以护航注册表化。
