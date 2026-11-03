# 对抗复核 A：DDDSA 架构设计 v1 —— 架构适配性视角攻击

> 2026-11-03。复核员 A（架构适配性）。攻击对象：[DDDSA 架构设计 v1](../../superpowers/specs/2026-11-03-dddsa-architecture-design.md)。
> 纪律：remake-verifier——怀疑一切设计主张，判定必须带证据指针（代码行/符号引用数）；
> 证据分级按 AGENTS.md §3（A=硬证据，B=静态推理，C=直觉）。C 级不得作为裁定唯一依据。
> 产出=判定表（§1–§5 攻击点逐项）+ 修订清单（§6）+ §7 待裁决 7 问裁定。

## 0. 攻击对象与方法

- **对象**：[DDDSA 架构设计 v1](../../superpowers/specs/2026-11-03-dddsa-architecture-design.md)（116 行草案）。
- **方法**：对每个设计主张做"实证复核"——凡可证伪的数字/锚点/先例主张，逐一到
  `reference/src/deobf/a.java`、`crates/game-core/`、`docs/spec/`、`tools/` 取证；
  判定三档：**成立** / **成立但口径需修** / **不成立（伪/陈旧）**。
- **证据纪律**：A=行号/行数/实测引用数；B=静态推理；C=直觉。裁定只引 A/B。
- **关键取证结果速览**：

| 设计主张 | 实测 | 级 |
|---|---|---|
| sprite_defs 89 型表 | `defs.rs:47` 型号域 0..=88 共 89 型 ✅ | A |
| interact_with_cell 208 hits/7 域 | 2026-11-02 review 表（combat:67 entity:61 walk:36 visual:30 menu:9 script:3 route:2）轮2复验 ✅；现 scene.rs:3413-3719 共 307 行 | A |
| "pickup_item_type 220 行 match" | **伪**：Rust 67 行（scene.rs:4225-4291）；a.java:7534-7633 共 100 行 | A |
| "a.java:7528-7620 逐值锚" | 锚行漂移：实为 7534-7633 | A |
| ItemAction 九 case 先例 | Rust `ItemAction` 实为 **7 变体**+ITEM_EFFECTS 表已落（item_effect.rs:8-47），use 路径已表驱动 | A |
| too-many-args "6/8 处" | 现数 **8 处**指名清单（2026-11-02 E2 修正，plan:85-88） | A |
| 实体 20-50/层 | EntityTable cap=100（scene_persist.rs:74）；Entity=11 扁平字段（entity.rs:19-42） | A |
| 显式共享上下文无先例 | **伪**：`HostCtx` trait 已存（script.rs:128-168，23 方法 fat trait） | A |
| 金样 60 场景 | state-dump-golden.json scenarios=60 ✅（11-02 时 56，已增） | A |
| 新内容零改码可演练 | tools/zero-code-drill 已实跑（新敌人=纯表行，`git status --porcelain crates/` 空断言） | A |

## 1. 攻击点 1：ECS 裁定复核（§0）

**判定：裁定方向成立（组件=数据采纳 + 运行时否决），但依据栏一处低估一处高估，需修口径。**

| 子主张 | 判定 | 证据 |
|---|---|---|
| 组件=数据采纳 | **成立** | 89 型属性行已是组件化数据（defs.rs:47；zero-code-drill 实证"加新敌人=纯表行零 Rust 改动"） |
| System 调度器否决 | **成立** | 调度=MOV 脚本 VM（16 op 封闭，dispatch-shape-survey.md:22）+ tick 状态机；另造调度器=两套真相属实 |
| Archetype/Query 否决 | **成立** | EntityTable cap=100（scene_persist.rs:74），格索引=cellSlotTable 已落；吞吐确实无需求 |
| 全 ECS 否决 | **成立** | SoA→AoS 刚付过成本（entity.rs:12），且金样**整表锚定 AoS 布局**（entity.rs:17"state_dump 金样整表引用含 100 槽尾部，尾默认必须逐字段对齐"）——再翻新=直接击穿金样 |
| "组件思想=物品注册自然模型" | **成立但高估了注册表** | 见下 |

**被低估的 ECS 适配面——不是 buff 爆炸**：问题设定里的"实体 buff/状态组合爆炸"实测**不存在**——
Entity 11 个扁平字段、状态面=state 三值(0/1/2)+visible/removed 布尔（entity.rs:19-42），**无任何可叠加状态槽**；
玩家态=hp/atk/def/三色钥匙/金/equip tier 标量组。本作组合爆炸面≈0，ECS 组合收益的前提不成立。
真正的低估面在**数据侧参数组合**：pickupItemType 的 29/30（+1 或 +floorTier）、31/32（50/200×floorTier）
是"属性行×楼层档"参数化（a.java:7556-7577）——这恰是"组件=数据"已覆盖的面，§0 依据栏应补这条实证，
防 Stage B 以"组合爆炸"为由翻案。

**被高估的面——注册表够不够**：注册表解决**分发**，不解决**耦合**。实证：`execute_item_action`
的副作用横跨 mode 分派/实体表/小地图/相机/换层，item_effect.rs:52-53 自注"【双归域·编排面】
动作执行=跨域副作用……不可迁面显式留置"；MirrorTeleport 单臂触 interact_with_cell+place_player_at+
set_camera+apply_step_cell_effects+transition_to 五域（item_effect.rs:110-133）。ItemEffect trait
搬不动这层——"注册表+trait 通信"的分工必须写明：**注册表管分发，Coordinator 管编排**（§3.1 提了但低估工作量，见攻击点 3）。

## 2. 攻击点 2：注册表设计（§2）

**判定：方向对（数据行=属性 + EffectKey=行为指针 + 注册表=桥），但两处边界错，params 必须裁强类型。**

### 2.1 "220 行 match" 数字伪 + 现状误判（不成立）

- `pickup_item_type` 实测：Rust **67 行**（scene.rs:4225-4291）、a.java **100 行**（7534-7633）。
  "220 行 match" 无出处——引用数必须可复现，此数应删。
- 更重要：**注册表化已完成一半**。`item_effect.rs` 已有 `ItemAction` enum（7 变体，强类型命名参数）、
  `ITEM_EFFECTS` 静态表（item_effect.rs:32-45）、`item_actions()` 查表、`execute_item_action` 分发，
  自注"新增道具=加行（C1 §5.1 git diff 断言）"（item_effect.rs:143-145）。
  §2.2 把"已落的 use 路径表驱动"与"待做的 pickup 路径表化+items.json 数据化"混为一谈——
  评审者据此会误估 P1 工作量与新颖度。a.java 锚也应更正为 7534-7633。

### 2.2 params 弱类型（JSON map）= 重演 pickup_item_type 脆弱 —— 裁决：强类型

- **脆弱根源实证**：pickupItemType 的魔法 int 区间（13..=25|85|86、33..=36|79、37..=40|80 的
  tier 映射，a.java:7534-7620）无类型载体；迭代 70 已实证错帧事故："传类型 80=D 表越界错帧"
  （scene.rs:4262-4266 注释，A 级）。
- JSON map `params` 把这个脆弱性从"match 臂魔法数"换成"运行期字符串键+类型断言"——**更糟**：
  编译器不再把关，键拼写/类型错误推迟到运行期。
- **既有先例恰好是强类型**：`ItemAction::Heal { mul: i32, div: i32, msg_fmt: &'static str }`、
  `RemoveAdjacent { target: i32, msg }`（item_effect.rs:8-28）——参数是命名字段。
- **裁决**：effect→**每个 EffectKey 一个强类型 Params**（enum 变体字段或 per-effect 结构体）；
  JSON `params` 只存在于**数据面**，加载期经 schema 校验转换成强类型（五类拒载的
  UnknownField/DomainOverflow 语义天然覆盖参数键/值域校验，mod-skeleton README 已镜像同名
  Reject 变体）；`ItemEffect::apply` 只收强类型。**"表驱动"≠"弱类型"**——typed enum 的行也是表。

### 2.3 注册表 vs match 的可测试性

- 注册表胜在增量可测（新 EffectKey=新测试，§6.4 已对）且与编辑器数据面同构；
- 但 match 是**穷尽**的、注册表不是——"未知 effect 语义"（§2.2 待裁决）直接决定测试边界：
  加载期拒载 ⇒ 未知键不可能进运行期 ⇒ 测试面封闭；运行期静默 ⇒ 测试面有暗角。
  裁定见 §5 Q1。

## 3. 攻击点 3：trait 面五件（§3.1）

**判定：漏三错一；interact_with_cell 收敛方向对，但 5-8d 包含它=低估。**

### 3.1 漏了谁

| 缺口 | 证据 | 修订 |
|---|---|---|
| **输入映射** | Dispatcher/classify_tap 已是独立面（touch.rs:156-167；probe_step 语义纠葛见 2026-10-29 review R2-1）；§7 Q3 自己也在问"battle 的输入面在哪切"——答案：输入映射是独立 trait（InputMapper：输入事件→intention），**不进** BattleSubsystem | 五件→六件 |
| **实体域** | **最大遗漏**：`HostCtx` trait 已存（script.rs:128-168）——脚本 VM 的实体/相机/对话副作用已走显式上下文注入，23 方法。五件清单既不列 EntityService 也不声明 HostCtx 与新 trait 面的关系；不吸收=脚本域与新子系统**双轨**（两套实体访问面） | +EntityService（吸收/更名 HostCtx），或显式写"HostCtx 即实体域 trait，不再新建" |
| **渲染/UI** | §1 架构图有 UIRender 域，§3.1 五 trait 清单没有——图文不一致。paint 簇 930 行内聚最高（architecture-review.md:79），是纯读域 | 显式写"UIRender=纯读域（消费状态快照→像素），不入 trait 通信面"——把"漏"变成"有意排除" |

### 3.2 错划了谁

- **PersistCodec**：只画了 `serialize/deserialize(SceneState)`——SceneState 是九域聚合，
  codec 编解码整个聚合意味着任何域字段变更都动 codec；应拆"**存档格式编解码**（Blob↔域切片）"
  与"**存档行为**（PersistState 状态机，scene_persist.rs 579 行已落）"两个边界，trait 面只管格式。

### 3.3 interact_with_cell 208 hits 收敛进 Coordinator 现实吗

- **实证成立**：208 hits/7 域（combat:67 entity:61 walk:36 visual:30 menu:9 script:3 route:2，
  2026-11-02 review 表，轮2复验 ✅）；现 307 行（scene.rs:3413-3719）。
- **方向对**：编排收敛进 Coordinator+trait 分发、"渐进：每域迁移时直写点改 trait 调用"——
  与 2026-11-02 攻击结论一致（档 B 整体过度被否，渐进被裁）。
- **但估算低估**：仅 tick_battle（207 行，scene.rs:3746）+ apply_step_cell_effects（210 行，
  scene.rs:3999）+ interact_with_cell（307 行）三个高纠缠函数就 724 行；再加 **scene_tests
  6,098 行直填字段构造面**（scene_persist/sc.persist.journal 直赋、sc.entities.items[e].removed
  直写，slot_equip_scene_tests.rs:229/1302）——域切片一动测试构造面全响。此风险 2026-11-02
  攻击 R7 已立（"域切片须保构造面或适配层"），**DDDSA §5 没接**，必须补进 P2 判据。

## 4. 攻击点 4：Ctx 设计（§3.1）

**判定：高危——仓库已有前车之鉴（HostCtx），不立宪必成上帝袋。**

- **前车实证（A 级）**：`HostCtx`（script.rs:128-168）就是"显式共享上下文"在本仓的第一次尝试，
  结果=**23 方法 fat trait**（find_path/cell_entity/…/spawn_entity/…/exit_application/mvs_exchange），
  横跨实体/相机/对话/楼层/应用生命周期——这是 Ctx 的宿命预演。Ctx 若做成"玩家态/相机/布局/队列"
  数据袋，每个子系统伸手取字段，只是把 too-many-args 换成 too-many-fields，红线精神
  （依赖面显式可审）被绕过。
- **与 too-many-args 红线的张力裁定**：红线数值应写死 **8 处**（2026-11-02 E2 修正：原"6 处"陈旧，
  现 8 处指名清单=scene.rs:623 load_floor_from_save、scene_paint.rs:8 paint_overlay_box、
  render.rs:730 draw_region_scaled、logo_anim.rs:125 register、menu_family.rs:120 draw_edge_patch、
  menu_family.rs:402 paint_wrapped_lines、pathfind.rs:151 relax_path_neighbors、walk.rs:91
  tick_stepping，plan:85-88）；§3.1 现写"6/8 处"含糊，应改为"8 处指名清单（2026-11-02 E2）"。
- **Ctx 合法化三原则（不写进红线则否决 Ctx）**：
  1. **只聚合不服务定位**：Ctx 字段=纯数据（玩家态/相机/布局快照/消息队列），
     禁含 `&mut dyn` 服务引用；服务面走 trait 参数（子系统按需收窄自己的 trait）；
  2. **字段面冻结纪律**：Ctx 加字段=架构评审登记——复用 too-many-args 的"指名清单+永久保留"机制，
     防逐字段膨胀成上帝袋；
  3. **签名约束**：trait 方法 = `ctx + ≤2 个语义参数`；禁止 ctx 与散参双轨（要么进 ctx 要么显式参数，不两占）。
- 三条缺一，Ctx=B 级推断必成上帝袋（HostCtx fat 化是 A 级既成事实）。§7 Q2 据此裁定。

## 5. §7 待裁决 7 问逐项裁定

| # | 问题 | 裁定建议 | 依据 |
|---|---|---|---|
| 1 | 未知 effect：拒载 vs 静默 | **两层分治**：① 数据面（编辑器/mod 表）未知 effect=**加载期拒载**（与五类拒载同严，§6.3 已有）；② 运行期"未注册 effect"**不可能发生**（加载期已拒）；③ "登记-不执行"（R-TODO-1b 语义）只用于**已注册但执行失败**与 classic 保真面的 `_ => {}` 不可达臂（scene.rs:4287，对齐 a.java:7608-7620 default return）。三个语义不得混用 | gotcha_match-catchall：`_ => {}` 吞未移植臂是**坑**不是语义范式 |
| 2 | Ctx vs too-many-args | **接受 Ctx，但按 §4 三原则立宪**（纯数据聚合/字段面冻结登记/ctx+≤2）；红线表述改"8 处指名清单" | HostCtx fat 化前车（script.rs:128）；E2 修正（plan:85） |
| 3 | trait 面边界 | **五件→六件半**：+InputMapper（输入→intention，battle 输入面在 Mapper→Coordinator 切，不进 BattleSubsystem）；+EntityService 或声明 HostCtx 即实体域 trait；UIRender 显式=纯读域不入通信面；PersistCodec 拆"格式编解码"与"存档行为" | §3.1 表；touch.rs:156-167 先例；script.rs:128 双轨风险 |
| 4 | EnemyRegistry 入 P1？ | **不入 P1，也不在 P2 新建**：89 型属性行已表化（task-5 三表）+Combat 纯计算已落（combat.rs 171 行零耦合，plan:37）；剩余编排（tick_battle）本就归 Coordinator（P2）。现在立 EnemyRegistry=空壳登记处。P3 编辑器需"新敌人行为键"时再补 EffectKey 面，与 ItemRegistry 同式 | zero-code-drill 已覆盖"新敌人零改码"数据面 |
| 5 | 编辑器形态 | **Web 静态页够（MVP），但校验必须复用真解析器**：JS 侧 schema 复刻会漂移——mod-skeleton README 红线"拒载以真加载层为准……防镜像自证"。正解=game-data 解析器编 wasm 进编辑器（D1 Rust→wasm 既定）或编辑器产出一律过 CI 真解析器闸（drill.py harness 先例）。宿主读写形态（桌面 fs/本地服务）不关键，**校验后端同源**才是关键 | tools/mod-skeleton/README.md；tools/zero-code-drill/README.md |
| 6 | 注册表运行期 vs 加载期 | **数据期动态（加载期注册），禁运行期动态**：① 表哈希闸（gates/table_content_hash.json）与金样禁 mod 表（两份制规则 2/3）都建立在"运行前表集已定"；运行期换表=哈希闸失效；② 差分验证要求同输入同表=同 trace，运行期注册破坏可复现性；③ 既有 ITEM_EFFECTS/mod-skeleton 全是加载期。"动态"=编辑器改表→重载，非 so 插件 | mod-skeleton 四规则；AGENTS.md §3 证据纪律 |
| 7 | 三阶段顺序与估算 | **顺序对**（表→trait→编辑器），**估算重校**：P1 3-5d 可信（use 路径已表驱动，剩 pickup 表化+items.json schema）；**P2 5-8d 不可信**——三高纠缠函数 724 行+scene_tests 6,098 行构造面+HostCtx 双轨裁决，改 **8-15d** 或显式砍范围（interact_with_cell 只收编排注记、不迁函数体）；P3 5-8d 前置=P1 schema 定稿。判据每阶段补 `python3 gates/cli.py all` + zero-code-drill 式零改码断言 | 实测函数行数（§3.3）；R7 测试保护风险 |

## 6. 修订清单（改哪节 + 改成什么）

| # | 改哪节 | 改成什么 | 优先级 |
|---|---|---|---|
| R1 | §0 依据栏（"吞吐无需求"行） | 补两条实证：①"Entity 11 扁平字段/三值 state（entity.rs:19-42）组合面≈0，buff 爆炸前提不成立"；②"零改码已实证（tools/zero-code-drill：新敌人=纯表行）"；并写明"注册表管分发，Coordinator 管编排"分工 | P0 |
| R2 | §2.2 注释"pickup_item_type 220 行 match" | 删伪数。改："pickup 路径 match=a.java:7534-7633（100 行）/Rust scene.rs:4225-4291（67 行）；use 路径已表驱动（item_effect.rs ITEM_EFFECTS，C1 已立），待做=pickup 表化+items.json 数据化"。§2.2 锚"a.java:7528-7620"更正为 7534-7633 | P0 |
| R3 | §2.1/§2.2 params | "params 弱类型 JSON map"→"effect 对应**强类型 Params**（enum 变体字段）；JSON 只存数据面，加载期 schema 校验转强类型（UnknownField/DomainOverflow 覆盖参数键/值域）；apply() 只收强类型"。补一句："表驱动≠弱类型" | P0 |
| R4 | §2.2 未知 effect 行 | 按 §5 Q1 三层分治改写（数据面拒载/运行期不可能/登记-不执行仅限执行失败+classic 不可达臂） | P0 |
| R5 | §3.1 trait 清单 | 五件→六件半：+InputMapper；+EntityService（吸收/更名 HostCtx，声明双轨合并）；UIRender 显式=纯读域不入通信面；PersistCodec 拆"格式编解码/存档行为" | P1 |
| R6 | §3.1 Ctx 段 | 补三原则（纯数据聚合禁服务定位/字段面冻结登记/签名 ctx+≤2）；"6/8 处"改"8 处指名清单（2026-11-02 E2）" | P0 |
| R7 | §3.1 编排收敛 + §5 P2 判据 | 补测试构造面保护条款："域切片须保 scene_tests（6,098 行）直填字段构造面或提供适配层"（2026-11-02 R7 延续） | P1 |
| R8 | §5 P2 行 | 估算 5-8d→**8-15d** 或显式砍范围（interact_with_cell 只收编排注记不迁体）；每阶段判据补 gates all+零改码断言 | P1 |
| R9 | §7 Q4 相关（§5 阶段表） | EnemyRegistry 显式后置：P1 不做、P2 不新建、P3 按需补行为键面 | P2 |
| R10 | §6 红线 | 加两条：⑥"Ctx 字段面冻结（加字段=架构评审登记）"；⑦"编辑器/校验后端=真解析器同源（防镜像自证）" | P1 |

## 7. 结论

- **架构大方向成立**：组件=数据+注册表分发+OOP 子系统主体，对本作（格子 RPG、实体≤100 槽、
  MOV 脚本调度、差分验证约束）是正确取舍；ECS 运行时否决有金样实证背书。
- **四个攻击点共 10 处修订**：2 处伪数字（220 行、6 处）、1 处危险设计（params 弱类型）、
  1 处前车重演风险（Ctx→HostCtx 二世）、3 处边界缺口（InputMapper/EntityService/测试构造面）、
  1 处估算失真（P2）、2 处红线补充。
- **对 P1 落地的净效果**：工作量可信（use 路径已表驱动），但 schema 定稿必须先裁强类型
  （R3），否则 items.json 一发布就是带类型债的公共接口。
- 遗留假设（C 级，不作裁定依据）：P2 8-15d 仍为估算区间，非实测；Stage B 是否引入
  叠加状态（buff）未知——若引入，届时重新评估 ECS 组件组合面，本裁定只约束 v1。
