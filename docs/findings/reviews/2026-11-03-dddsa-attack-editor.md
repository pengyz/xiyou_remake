# DDDSA 架构设计 v1 对抗复核 C（编辑器/工程可行性视角）

> 2026-11-03。攻击对象：[DDDSA 架构设计 v1](../../superpowers/specs/2026-11-03-dddsa-architecture-design.md)。
> 角色：对抗复核员 C（remake-verifier 纪律）——编辑器形态/工程现实攻击面。
> 证据基准：`crates/game-data/src/defs.rs`（五类拒载真值）、`tools/mod-skeleton/`（Python 镜像先例）、
> `tools/zero-code-drill/`（零改码演练已机器化）、知识库 `gotcha_dual-porting-surfaces-drift` /
> `pattern_dual-copy-algorithms-sync-fix`（两处真相先例）。

## 1. 攻击点 1：编辑器形态（§4）——"Web 静态页+本地 JSON 读写"不可直接落地

**攻击**：§4 形态栏"tools/editor/ 静态页+本地 JSON 读写"在浏览器沙箱下**没有可移植实现**：

| 候选 | 现实约束 | 裁定 |
|---|---|---|
| `file://` 静态页 fetch 本地 JSON | CORS 禁读；写=不可能 | ✗ |
| File System Access API（showOpen/SaveFilePicker） | 仅 Chromium 系；Firefox/Safari 不支持；每次写需用户手势或保留句柄+权限弹窗；**写盘前无法拦截执行校验** | ✗（做兜底导出可以，做主读写不行） |
| 桌面工具（Tauri/Electron） | 引入新栈（Node/Rust-GUI），与 D1"跨平台+wasm 预留"及 tools/ 现有 Python 工具链（gates/cli.py、drill.py、loader.py 全是 Python）不一致 | ✗ 过重 |
| **本地服务（tools/editor 起 Python http，静态页由它伺服）** | Python 工具链零新增；fs 读写直白；**校验可在服务端跑真校验器**（见 §2）；浏览器全兼容；演练脚本同一进程族可串联 | **✓ 可落地裁定** |

**裁定**：编辑器=`tools/editor/` 静态 HTML/JS 表单 + `server.py`（stdlib http.server 级别的薄服务：
GET 读表 / POST 写表→写盘前过校验闸→落盘）。JSON 永远经服务端校验闸落盘，JS 侧不持有写权限。
兜底：纯静态页+手动下载/上传 JSON 仅作 P3 逃生通道，不作主形态。File System Access API 留作
后续"导出到任意路径"增强，不进 MVP 判据。

## 2. 攻击点 2：校验后端复用（§4）——跨语言重复实现=两处真相

**攻击**：§4"五类拒载后端复用"一句含糊。五类拒载的**真值在 Rust**（`crates/game-data/src/defs.rs:18-28`
BadJson/BadForeignKey/DomainOverflow/FrameIndexOutOfBounds/UnknownField + SCHEMA_VERSION=2 严格键集）。
Web 编辑器是 JS——若 JS 再写一份校验，就是知识库 `pattern_dual-copy-algorithms-sync-fix`（函数级双拷贝
漂移，迭代 26/30 两度复发）与 `gotcha_dual-porting-surfaces-drift`（双面各自绿、DC-9 桥漏字段）的
**跨语言升级版**：连编译器都帮不上忙。仓库已有第三份镜像先例：`tools/mod-skeleton/loader.py` 的
Reject 变体 Python 同名镜像——其防漂移机制=zero-code-drill 的"**真解析器对照（防镜像自证）**"
（harness path-dep 真 crates/game-data，保真侧拒载以真加载层为准）。

**裁定（三层防漂移，缺一不可）**：
1. **真值唯一**：JS/表单侧**不实现**校验规则。服务端写闸调**真校验器**——`crates/game-data` 加
   `bin/validate`（或零依赖 harness，同 drill.py 成熟做法），输入 JSON → 退出码+Reject 文案。
   编辑器表单的错误提示=真校验器输出的透传，不是 JS 重算。
2. **schema 单源生成**：字段键集/域（id 域、type 0..=88、mod≥1000）以 Rust 常量为唯一源，生成
   `tools/editor/schema/*.schema.json`（build 脚本或单测快照）；门禁比对生成物防手改
   （对齐 `decision_generated-docs-edit-via-config-source`：改源不改生成物）。
   新增 EffectKey=改 Rust 注册表→重生成 schema，JS 表单的下拉项也从生成物读。
3. **镜像仅作 UX 缓存 + 等值测试**：JS 若要做即时表单内提示（用户体验），只许是真校验器的
   薄镜像，且**夹具等值测试**强制：`defs_negative.rs` 16 负例+正例 fixture 语料同时跑
   Rust 真校验器与 JS 镜像，verdict 必须逐条一致；不一致=门禁红。镜像注释必须回指 defs.rs 行号
   （DC-9"桥公式缺字段"教训：镜像更新必须注明真值来源行）。

## 3. 攻击点 3：编辑器验收演练（§4）——"零改码全流程演练"机器化

**攻击**：§4 验收"新物品全流程演练：编辑器加物品→游戏可见/可拾取→零改码"若靠人肉点编辑器+试玩，
就是 verifier 禁令"未贴出的运行结果不存在"。**好消息：演练机器化已有现成地基**——
`tools/zero-code-drill/drill.py` 已实现同类命题（假想新敌人 mod 行加载+五类拒载失败面+零改码断言
`git status --porcelain crates/` 为空+保真哈希闸），P3 演练不是从零造。

**裁定**：演练=zero-code-drill 扩展（drill-items 或 drill.py 加阶段），机器化管线四段：

| 段 | 内容 | 断言（任一失败 ⇒ 退出码 1） |
|---|---|---|
| E1 编辑器产出 | 用编辑器**服务端写闸**产出 items.json（测试走 server API 而非 UI 点击——UI 层单测另列） | 产出文件过真校验器（五类拒载全绿） |
| E2 拒载闸 | 同一文件故意坏一字段（unknown-field/域外 id/坏外键）送写闸 | 真校验器逐类拒载、错误文案含定位（路径/偏移） |
| E3 游戏加载断言 | game-oracle 逻辑级 harness（**无头**，守"验证不打扰"红线）加载 mod items.json：新物品出现在注册表、拾取路径 ItemOutcome 可断言 | 加载成功+拾取断言通过；金样 60 场景字节等价（mod 表不进金样=规则 2 既有） |
| E4 零改码收尾 | `git diff/status -- crates/` 为空 + 保真表哈希闸绿（复用 drill.py [0b]/[7]） | 双查通过 |

**注意**：E3 的"游戏可见/可拾取"必须落在 game-oracle 状态断言（ItemRegistry 含新 id、
pickup 返回值），**禁止**要求实机截图/人工试玩充当验收——那既违验证不打扰红线，也不可机器复跑。

## 4. 攻击点 4：P1/P2/P3 估算现实性

| 阶段 | 原估 | 工程拆解 | 裁定 |
|---|---|---|---|
| P1 物品注册表化 | 3-5d | items.json schema+生成、ItemEffect trait、pickup 220 行 match→dispatch（**逐 case 逐值锚抽象**，ItemAction 九 case 先例）、魔法区间表化、金样 60 全同回归、每 EffectKey 新测试 | **3-5d 成立但顶格**——前提是内建 effect 集**冻结为现 match 九 case**，不顺手加新效果；若 effect 集还在长，5-8d |
| P2 子系统 trait 化 | 5-8d | 五 trait 抽取+Ctx+interact_with_cell 208 hits/7 域编排收敛+C2 终值复测 | **偏紧**：§3.2 自认"直写清零（在跑）"——若 P2 开工时直写清零未完，P2 继承该工作量，5-8d 不够；**修正估=前置条件：直写清零先收尾，否则 8-12d** |
| P3 编辑器 MVP | 5-8d | server.py+写闸（真校验器 bin）+schema 生成+表单+drill 扩展（§3 四段）+镜像等值测试 | **5-8d 仅覆盖 items.json 单表**；§4 数据面写了 items/enemies/dialogs/maps **四表**——四表全做=表单×4+外键面×4，实估 8-12d。**裁定：P3 MVP 收敛为 items 单表**，enemies/dialogs/maps 编辑面留 P3.5 另估 |

**核心质疑**：三估共 13-21d，风险全落在"表集还在长"（P1）和"直写清零没完"（P2）两个未定价前置件上。
建议 P1 开工条件=effect 集冻结清单落纸；P2 开工条件=直写清零合入（或显式并入 P2 估）。

## 5. 攻击点 5：注册表扩展面（§2.3）——P1 防膨胀裁定

**攻击**：§2.3 列 EnemyRegistry/TileRules/ShopGoods"同模式推广"，若一并入 P1，P1 直接膨胀：

- **TileRules/RestoreRules**：已表化（§2.3 自己写明）——**零工作量，不是扩展面**，从清单里删掉免得虚增范围；
- **EnemyRegistry**：复杂度远超物品——89 型属性行有 quirk 路由（QuirkPathRequired，Q-001/Q-003 族）、
  域外三界（ANIM_TABLE_IDX 89/容器 16/张数表扩张）、mod 型号命名空间（<89/89..999/≥1000 三段）——
  这些**已经是 zero-code-drill 和 mod-skeleton 的既有语义**，再叠"行为钩子注册"= P1 里再造一个 P1；
- **ShopGoods**：小，但依赖物品表（商品=物品 id 外键）——顺序上必须在 ItemRegistry 之后。

**裁定**：**P1 只做 ItemRegistry**（物品=最小闭环、§2 的核心新构件、外键面最小）。分批：
ShopGoods 作为 ItemRegistry 的第一个消费者并入 P2 尾或 P3 前（半天级）；EnemyRegistry **后置**
（P3 后或独立阶段）——且它的"零改码加新敌人"命题 zero-code-drill 已经证过加载面，
行为钩子化等编辑器闭环跑通、模式被验证后再套。§2.3 原则（数据行=属性/behavior key=行为指针）
保留为**指导原则**，不作为 P1 交付清单。

## 6. 攻击点 6：§7 待裁决 7 问工程视角逐个裁定

| # | 问题 | 工程裁定 | 理由/依据 |
|---|---|---|---|
| 1 | 未知 effect：拒载 vs 静默 | **加载期拒载**（与红线 3"注册表加载期拒载语义=五类拒载同严"自洽）；运行期"登记-不执行"仅限保真面已存在的 R-TODO-1b 静默停摆语义复刻，mod 面禁静默 | 行为注册=编译期静态 register（见 Q6），加载期注册表已闭合，未知键=数据错误，fail-fast 才能进 E2 拒载闸；静默不执行会让"编辑器加物品没效果"变成不可定位的软故障 |
| 2 | Ctx vs too-many-args | **Ctx 聚合面成立，保留 too-many-args 6/8 处不触**（架构面结论照抄，工程补充：Ctx 字段显式、禁止 Ctx 变成新的直写后门——子系统写 Ctx 字段=写对方域，编排注记要能归因） | 张力本质是"直写换了个马甲"；C2 判据（编排注记归因）能管住 |
| 3 | trait 五件边界 | 一处修正：**ItemService::pickup 与 ItemRegistry 职责重叠**——registry=数据/行为表（静态），service=运行时效果应用；建议合并或明示 registry 只做查表。其余四件从工程面无漏 | pickup 分发链（match→dispatch→ItemOutcome）一条为好，两处=又一个双面 |
| 4 | EnemyRegistry 入 P1？ | **否**（§5：后置；P1 只做 ItemRegistry，ShopGoods 批 P2 尾） | 防 P1 膨胀；quirk 路由/域外三界复杂度另算 |
| 5 | **编辑器形态** | **本地 Python 服务**（tools/editor/server.py 伺服静态页+读写闸）；浏览器 File System Access API 不可作主路径（仅 Chromium、写前无校验缝）；桌面工具过重 | §1 全表；Python=tools/ 既有栈，服务端=真校验器唯一挂点 |
| 6 | **注册时机**（运行期 vs 加载期） | **数据期动态、代码期静态注册**：行为注册=编译期 `registry.register`（Rust 静态，非 so 插件）；数据引用 EffectKey=加载期解析+拒载。**不做 so/动态库级插件** | D1 含 wasm——动态库不可用；差分验证要确定性装载（同输入同注册表）；"动态"只需做到"加数据不改码"，不需要"加行为不重编译"（加新 EffectKey 本来就是新代码=新测试，红线 4） |
| 7 | **顺序与估算** | 顺序 P1→P2→P3 同意（P3 依赖 P1 表化）；估算修正见 §4：P1 3-5d（effect 冻结为前提）、P2 5-8d（直写清零收尾为前提，否则 8-12d）、P3 5-8d **仅 items 单表**（四表=8-12d）。演练脚本骨架（zero-code-drill 扩展）可与 P1 并行搭 | §4 拆解表 |

## 7. 结论与回传要点

**总体**：DDDSA v1 工程面**有条件可行**——架构方向（OOP 子系统+注册表）与仓库现状咬合好，
但 §4 编辑器栏三处必须改写才可落地：

1. **形态**：静态页+本地 JSON 读写 → **本地 Python 服务（tools/editor/server.py）**，JSON 落盘必经服务端校验闸；
2. **校验复用**："后端复用"不是调用而是**三层防漂移**：真校验器唯一真值（Rust bin）+ schema 从 Rust 单源生成 + JS 镜像仅 UX 缓存且夹具等值测试强制（先例：mod-skeleton 镜像+drill 真解析器对照；反例：DC-9/迭代 26-30 双拷贝漂移）；
3. **验收**：全流程演练=zero-code-drill 扩展四段机器化（编辑器产出→真校验器拒载闸→game-oracle 无头加载断言→零改码+哈希双查），禁实机试玩充当判据。

**附裁定**：P1 仅 ItemRegistry（EnemyRegistry 后置、ShopGoods 批 P2 尾、已表化的 TileRules 删出清单）；
P3 MVP 收敛 items 单表；P2 前置=直写清零收尾；注册时机=编译期行为注册+加载期数据解析，不做 so 插件。

**完成判据**（本复核自身）：本文档落盘 `docs/findings/reviews/2026-11-03-dddsa-attack-editor.md`；
攻击点全部带证据指针（defs.rs 行号/工具目录/知识库条目），无 C 级猜测混入裁定。
