# 实现完整度测量方法论（v1）

> **收敛计划**：长线收敛的阶段划分/判定/批次见 [convergence-plan.md](convergence-plan.md)（迭代 72 起生效）。

> **长线目标（2026-10-12 用户正式设定）**：**持续收敛，收敛条件 = 完成整个
> 改写**（锚点 a.java 全部可观察行为的 Rust 复写），收敛判定 = 本文档
> 四条判据。每轮迭代完成后**必须执行语义回流**：本轮结论（新语义/新行号/
> 新表项）带 a.java 行锚写回 spec/findings/knowledge——回流缺失 = 迭代
> 未完结。迭代队列读 §3 矩阵与 §2a 未移植清单。
> **收敛判据（全部满足即收敛）**：
> 1. 符号面：112 个 applied 方法全部落档（fn 直证 / 内联确认 / 死内容-BUG 登记），
>    "未移植候选"清零；
> 2. 模式面：paint 21 臂 + run 16 臂全部实现，可达者 L3、不可达者 BUG-登记；
> 3. 内容面：DSL 15 码 Rust 引擎全覆盖（逐码有对拍证据）；楼层按区域抽样 L3
>    （每区 ≥1 层）；资源 138 条维持 L1；
> 4. 场景面：scenarios 套件全绿且每个场景的 `# cover:` 声明与 Rust 对拍一致。
> 不可达/死内容项按 D4 登记 BUG-ledger 后即视为"已收敛处置"（保留原行为）。

> 动机（2026-10-12，用户提出）：对拍基建成熟后"新用例首跑即绿"会成为常态——
> **测试全绿只证明"声明过的问题没有回归"，不回答"还剩多少没实现"**。
> 完整度需要独立于测试的**分母**：参考系自身的全集。本方法论定义分母、
> 分子登记规则、防钝化约束与更新 ritual。

## 1. 原则

1. **分母独立于测试**：全集从参考系锚点（`reference/src/deobf/a.java`、
   `data/naming/remap-table.json`、资源清单）机械导出，不由"我们写了哪些测试"
   反推——否则测什么什么就完整，循环论证。
2. **覆盖只增不降**（与 test-ratchet 同构）：登记过的覆盖项不可悄悄撤销；
   撤销 = 回退，须在 commit 说明。
3. **分子必须带证据指针**：每个"已验证"登记绑定 scenario 名 + tick 窗口 +
   trace sha（或 golden 名），`python3 tools/completeness.py --check` 校验存在性。
4. **验证强度分级**，不同级不等价：
   | 级 | 含义 | 当前主载体 |
   |---|---|---|
   | L0 | 已移植，无对拍 | 代码 |
   | L1 | 资源/golden 级 | game-data L1 golden |
   | L2 | op 流级对拍 | trace OPS diff |
   | **L3** | **全帧 sha 逐 tick 对拍** | preset_cases / gameplay 用例（主线） |
   | L4 | 视觉级（像素 diff） | 等字体方案（R2）定案后启用 |

## 2. 分母（四个维度，均为参考系全集）

| 维 | 分母 | 导出来源 | 当前值 |
|---|---|---|---|
| A 符号面 | 类字段+方法全集 | `data/naming/remap-table.json`（机械） | **581** = 413 字段 + 168 方法 |
| B 模式面 | `paint()`/`run()` 模式 switch **非空顶层 case 臂** | deobf `switch (this.gameMode)` 花括号配平提取（机械，语义权威 = `docs/findings/state-machine.md`） | **paint 21 / run 16** |
| C 内容面 | DSL 指令码 / 楼层 / 资源 | `docs/spec/p3-script-interpreter.md`（15 码）、maplv 55 层、assets-manifest 138 条 | **15 / 55 / 138** |
| D 场景面 | scenarios 脚本（覆盖面声明 `# cover:`） | `reference/oracle/scenarios/*.txt` | 16 场景 |

机械提取命令：`python3 tools/completeness.py`（确定性输出）。
已知口径注意：paint 空臂组（如 case 23-98 fallthrough）与 run 空操作组
（15/17）合法为空不计入；case 4 与 19 共享渲染臂按语义表归属。

## 2a. 符号面分子（迭代 31 起工具产出，人工终审）

`tools/completeness.py` 对 112 个 applied 方法做四信号匹配（语义名/蛇形名/
机械名前缀/证据行号引用）输出三档：

| 档 | 数值（2026-10-12） | 说明 |
|---|---|---|
| fn 直证 | **63** | Rust 侧有同名/蛇形/机械名前缀函数 |
| 内联移植候选 | **10** | Rust 注释引用同源 a.java 行号（如 markSlotEntry、handleOverlayKey）——人工终审后转正 |
| 未移植候选 | **36** | 迭代 31 已消费炼丹族（alchemyPriceFor/scaledByFloorTier → fn 直证 66）。**剩余真实缺口（迭代 32+ 队列，机制图谱已备）**：① 商人/告示牌链（最厚）：interact case16（render cat 16 = type 77/78 告示与商人，param 打包：高字节=标志位[&1=open 时 m_075、&2=效果表、&4=一次性]、低字节=报价序号+1；type77=文本源0（f_String_arr_06/07）、78/其余=源1（f_String_arr_08/09=商人 13 项））→ showOverlayMessage(kind4/5, 是/否图标) → overlay 分派 case4（-6/-5=applyMerchantOffer(true)/-7=(false)）与 case5（-3/-4=m_076 游标浏览、-5/-6/-7=关+回 mode10）→ applyMerchantOffer（m_074，13 项效果表：金币换钥/换血/加攻防 3%等 + param ^512/^256 翻转 + m_075 队列推进 + markEntityRemoved 一次性）+ 报价队列（m_075/m_076，f_byte_arr_14/15+f_int_88/89/90）；② spawnBossEvent/killAdjacentAt/findFloorGateEntity（boss/钹/暗道）；③ 物品栈族（useItemStack 全语义）。
  **迭代 32 L2 移植入库**：interact case16 分支（param 打包/文本表 06-09/kind4
  是否图标）+ overlay 分派 case4（applyMerchantOffer）/case5（报价浏览）+
  applyMerchantOffer 13 项效果表 + 报价队列（m_075/m_076）+ spendGold——
  结构逐行对照字节码。**L3 已收敛（迭代 33）**：preset-alchemy（floor12）T1-700 全帧一致
  ——真实行走键（raw 50/52/54/56，此前场景的 2/4/6/8 为无效键码空操作，
  全量审计修正）+ 炼丹三连购（HP×tier2/ATK×tier2/第三次不足→
  「没有足够的金钱」浮层）+ 商人/联动门 case16 全机制。**Java 侧取证的
  关键修正**：此前"Java 不开浮层"的判断有误——真实行走后 Java 帧序列
  与 Rust 逐帧一致（含不足浮层），此前分歧源于行走键空操作导致的
  场景状态从未到达触发点。floor4 的 type9 贴 y=0 崩溃行 = 不可达死内容
  （BUG-009）不变。
  **迭代 34 收敛（变换表修复 + 行走键审计）**：shim Graphics.drawImageTransformed
  补 8192/16384/24576 FLIP 臂（此前 default 恒等绘制 = 框体角件/底边未翻转
  ——用户报告的框体观感根因，真机 Nokia UI 实现这些常量故观感不同）；
  Rust draw_image_transformed 同步补臂；draw_edge_patch 经
  NOKIA_TRANSFORM_TABLE 路由 draw_image_transformed。行走键审计修正：
  场景数字键 2/4/6/8 = 无效键码空操作（真键 = raw 50/52/54/56），四场景
  16 行修正后真实行走生效。
  **新暴露缺口（迭代 35 核心，已定性）**：真实行走下 gameplay_floor1 系
  四用例全屏分歧——**harness 架构不匹配**：gameplay_floor1.rs 重放
  harness = 冷场景模型（load_floor 直接构造，跳过启动/菜单/载入链），
  而其场景 trace = 全流程录制（boot→菜单→槽载入→游玩）——冷场景重放
  全流程按键，状态从首个游戏 tick 分道（Java T537=存档选择界面、
  Rust=游玩画面，双侧实证）。**修复 = harness 统一**：四用例迁移至
  preset_cases 式全流程 harness（BootMachine→菜单→槽载入→场景移交），
  迁移后真实行走 + 战斗流差异逐帧考证（战斗流 = 本队列已知最大块）。
  **迭代 35 收敛（harness 统一 1/4 + rng 连续性缺口修复）**：floor_crossing
  迁入 preset_cases 全流程 harness（preset-floor1.preset，T1-1500 全帧）——
  **战斗流首次纳入全帧对拍**（行走 4 战 + state==3 抖动 + 伤害弹窗 + 换层
  遮幅）。迁移暴露并修复真缺口：`GameScene.rng` 构造默认 `new_seeded(0)`
  未续接 machine 阶段已消费的 LCG 位置（Java = a.java:2324 全局单例连续
  消费不重置）——战斗抖动为场景首个消费者故 T801 必爆。修复 = handoff 处
  `sc.rng = m.rng.clone()`（6 处 preset harness 统一补齐）；旧冷场景 harness
  的 `random_below(240)/(150)` 手工预消费 hack 即同一错位的遗迹。取证链：
  A-floor1-to-floor2 trace 曾于 10-12 18:43 被无 preset 再生覆盖（游戏卡
  空槽列表），已带 preset 再生并与 10-10 B trace 帧 sha 序列比对定位主机
  版本漂移；帧像素取证（ARGB 通道序陷阱见知识库 gotcha-oracle-png-channel-order）。
  余 3 用例（floor1-tour/floor2-tour/floor51-menu 系）迁移排队。
  **迭代 36 收敛（harness 统一 3/4）**：gameplay_floor1（T1-3000 全帧，
  含此前从未比对过的 T1701-3000 区段）与 floor2_tour（T1-2000 全帧，含
  T1501-2000 区段）迁入全流程 preset harness，**首跑即全帧一致**——rng
  连续性修复（迭代 35）即二者"真实行走分歧"的同一根因。新增覆盖首报：
  宝箱组开门（walkPhase 4 遮幅抖动）/ 型11 道具浮层 / 血瓶拾取 / 楼梯
  往返 floor2→1→2 / 蓝门重复弹关周期 / 金币拾取(+1) 首次获得 Rust L3。
  余 floor51_menu 1 例（序章 boot 型场景，非 preset 直达——需独立评估
  全流程重放或 preset 化，队列下批）。gameplay_floor1.rs 内 3 个诊断用例
  （dump_t544/dump_frame/dump_entity87）保留 ignored。
  **迭代 37 收敛（mode 11 全流程化）**：BootMachine 新游戏链补端口——
  mode 2 装载完成（非槽载入）由 `finished=true`（与 mode0/退出语义混用）
  改为专用 `new_game_ready` 信号 + m_000 case 3 软键/浮层清位
  （a.java:4312-4318）；`GameScene::apply_entry_step_effects` 公开入口
  （出生格 type83 → script 40，a.java:5708-5709/4315）。新用例
  preset_cases::prologue：**T1-9800 全帧一次通过**——boot logo/声音询问/
  标题/引子插画(mode14)/装载链(mode2)/冷启动移交（floor51 cell(1,11)
  hp300/atk10/def10 + bob/strip/backdrop/rng 连续量移交）/mode 11 对话层
  全程。旧冷场景 prologue_dialog（T577-9800 手调初值）退役。矩阵补正：
  mode 9 行（迭代 31/33 已 L3，二次校准漏更正）、mode 11 行（L2*→L3）。
  **迭代 38 收敛（harness 统一收官）**：floor51_menu 迁入全流程
  （preset_cases::floor51_menu，T1-9800 全帧首跑一致——序章全程 + 电影
  镜头等待态 -6/-7 大地图 UI 开关；旧 #[ignore] 版的 T9700-11099 窗口
  系旧版场景刻度已失效）。gameplay_floor1.rs 冷场景骨架
  replay_floor51_window 整体退役，仅余 3 个诊断用例（ignored）。
  ** gameplay_floor1 系四用例 harness 不匹配问题就此清零。
  **迭代 39 收敛（商人链 apply 语义真缺口修复 + 独立用例）**：新用例
  preset_cases::preset_merchant（A-preset-merchant T1-600 全帧）首跑即于
  T321（-6 接受报价拍）暴露 apply_merchant_offer 两处移植缺口：
  ① 返回值误用购买成功标志（var5）而非挂起标志（var6，a.java:7465
  return var6——入队/失败提示时 true、普通成交/移除时 false，浮层去留
  由此决定）；② 拒绝路径整体缺失（buy=false 且 flags&2==0 且 flags&1!=0
  → param ^256（非 ^512）+ 非一次性移除，a.java:7453-7460）。按 Java
  结构重构后 T1-600 全帧一致。附注：告示牌 param01（报价序号 1=攻防+3%
  项）整数运算 atk+atk*3/100 = +0（30 基数下整除为零），帧面无数值变化
  ——语义由 param^512 + markEntityRemoved 呈现（告示牌消失）。
  **迭代 40 收敛（长程/深探测两用例）**：floor1_tour（T1-8000 全帧，
  全项目最长 mode3 连续对拍——seed=7 固定走向大范围踩格，战斗/门/钥匙/
  道具/楼梯事件全类型混合，金 100→692 增量可见）与 walk_pure（T1-4000
  全帧，4 倍行走时长——相机边缘跟随全图巡回/parallax/实体动画长程相位）
  首跑全帧一致。附带修复两例静默退化 trace（A-walk-pure/A-floor1-tour
  18:2x 手工再生漏 preset，已带注入重生并核验 mode3；坑型补入知识库
  gotcha_scenario-preset-header-required）。
  **迭代 41 收敛（物品栈族 useItemStack 全语义）**：新用例
  preset_cases::preset_itemstack（preset-itemstack 场景 = item13 preset 变体
  注入 [19 型×1 次]，T1-500 全帧）。移植面：① useItemStack（a.java:7761-7781
  uses>0 → activate → 消耗且减至 0 时栈压实）；② activateItem 补消耗臂
  16/17（m_089 四邻格门控 + 文案）/18（全场型11 清除）/19（(atk+def)*74/10
  加血）+ m_100/m_089 助手；③ kind3 道具确认浮层分派（0 拍 no-op 系 T352
  实证）；④ 面板臂补 itemUseTargetType 装填；⑤ 文案表三处漂移修复
  （[6]/[21] 截断、[10] 威力加倍误植）。矩阵 mode 19 行同步修正为不可达
  死内容（迭代 41 核对）。
  **迭代 42 收敛（钹族 killAdjacentAt + activateItem 85/86）**：新用例
  preset_cases::preset_itemcymbal（preset-itemcymbal 场景 = 栈注入
  [86 型×1 次]，floor1 (3,10) 左邻 (2,10) 型43 怪），T1-600 全帧。移植面：
  ① activateItem 补 85（全场黄门清除）/86（四邻格 killAdjacentAt）消耗臂
  （a.java:7865-7886）；② killAdjacentAt（a.java:7923-7951：型5 宝箱类
  removeSlotAt + tryOpenChest 开则 walkPhase=4；型<67 且 cat8 且 state!=1
  怪物 → 移除 + 赏金〔持 25 型加倍〕，popup 落世界原点 (0,0)——Java 字面量；
  for 循环 removeSlotAt 压实后仍 slot++ 的跳位缺陷保真）；③ findFloorGate
  实体索引变体既有（load 落点步离已用），脚本 49/55 暗道换层命令与
  spawnBossEvent（需 boss 层夹具，现缺 40 层）留待后续批次。差分信号：
  激活前 (2,10) 撞怪阻挡 → 激活后金 100→104 + 左行通达。
  **迭代 43 收敛（boss 层夹具 + spawnBossEvent 链 + 49/55 暗道键）**：
  ① 夹具补齐 maplv40/sprite40（assets/raw 直拷入 fixtures；sprite40 29 条
  精确消费核验）；② spawnBossEvent（a.java:8903-8912）+ bossTypeOrder
  （2198：75/74/68/67/69/71）+ bossEventSpawns 七事件全表（2041-2104）
  + spawnEntity 网格吸附/DES 登场（entity::spawn/remove_des 既有）；③
  tick_battle 击杀钩子（a.java:10053-10070：bossTypeOrder 命中 → 事件；
  型73 → 脚本旗标 + loadLevelScript(22) + 40 层 spawnBossEvent(6)；型74 →
  脚本 21；型75 → 脚本 20 + battle_rerun_scene）；④ handleFieldInput 49/55
  暗道换乘键（a.java:5293-5306：f_bool_13 门控 + findFloorGateEntity
  param&255 目标层——f_bool_13 的 proximity 更新未移植、稳态 false 与
  Java 一致，臂已就位）。新用例 preset_cases::preset_itemboss40（40 层
  槽载入 (6,3) atk200 撞型73 boss 一拍斩 → 钩子全链 → 脚本 22 对话），
  T303 起 3037px 残差 = DES 登场动画相位差 ~2-3 帧（机制/坐标/状态全
  对齐），#[ignore] 挂起 + 战斗拍台账待办（residual 透明化）。
  **迭代 44 收敛（activateItem 20/21 消耗臂）**：todo!() 清除——20 对角
  瞬移（a.java:7832-7850：40 层禁用文案；镜像格 (w-1-x,h-1-y) interact
  放行 → placePlayerAtCell(m_024) + setCameraClamped 居中 +
  applyStepCellEffects + 消耗；拦截且无浮层 → 「无法移动到该位置」不
  消耗）；21 changeFloor(+1)（返回值即消耗与否，a.java:7851-7852）。
  新用例 preset_cases::preset_item2122（preset-item2122 场景 = 栈注入
  [20×1, 21×1]），T1-900 全帧首跑一致——覆盖 20 臂**拦截路径**（floor1
  出生点镜像格判定失败 → 文案浮层 → 栈不消耗；消耗/换层正路径随
  按层铺场景批次）。itemboss40 DES 登场相位残差的取证基建就位
  （stride-1 trace + 双侧 op 流转储；op 计数 872 vs 751+ 多行 drawString
  解析截断待解），战斗拍台账下批续办。
  **迭代 45 收敛（21 臂 changeFloor 正路径场景 + 转场相位残差定性）**：
  新场景 preset-item21（栈注入 [21×1]）+ preset_cases::preset_item21
  T1-900——T401 起 wipe 起始相位差一拍（Java T401 仍面板、T402 起
  dither-wipe→黑幕→落地；Rust T401 已进遮幅），#[ignore] 挂起 +
  状态转储逐拍台账待办（transitioning/wipe 计数器字段级）。机制本身
  已被 floor2-tour 系（楼梯触发 changeFloor）全帧验证——**挂起范围
  仅 mode10 面板内道具触发 changeFloor 的转场相位**。 |
   **迭代 46 收敛（item21 残差闭合 + 遮幅公共尾语义修正）**：台账法
   收口——A-preset-item21 全量转储字段级对表（T402：phase=2/closing=
   true/transitioning=true/**gameMode=3**/floor=1-pending=2）暴露双缺口：
   ① 遮幅点阵绘制只挂在 mode3 paint 路径（Java = paint 公共尾
   a.java:3177-3215，**所有 mode 后置共跑**）——提取 paint_wipe_tail
   于 mode10 路径复用；② changeFloor 未回 mode 3（Java 台账 T402
   gameMode=3 实证——面板内道具触发 changeFloor 后即回行走世界）——
   change_floor 内补 game_mode=3（楼梯路径原值即 3，无副作用）。
   preset_item21 摘除 #[ignore] 转全帧一致；boss40 DES 登场残差另批
   （mode 11 paint/advance 顺序，非本类）。
   **迭代 49/50 续办（itemboss40 残差根因定性 + 取证工具修复）**：
   ① 战斗拍台账建账：击杀拍 = T302 双侧对齐（f_int_151 0→1 冻结、
   popupV=[150,100] 首现、A1==A2 双跑 sha 一致 = Java 确定性无差）；
   ② 逐拍像素表：T302=0 全对齐锚点、T303-309 稳定 ~9000px、T310 起
   跳至 ~21900px（= DES 重派发拍；重派发高度语义修复已入 47 续办）；
   ③ **T303 空 paint 定性**：Java mode11 dialogPhase 门控该拍 0 ops
   （画布残影）vs Rust 全量 paint_dialog——残差本体；④ 取证工具修复：
   shim Graphics.flushOps 256 条截断 → 2048（转场拍实体 draw 入
   trace；reference 门禁全绿）。待办：mode11 dialogPhase 门控的
   paint 语义对表（paint case 11 a.java:2798-2900）→ 摘除挂起。
   **迭代 54 结论（entityAnimFrame 逐拍推进对表）**：Java/Rust 的 DES
   怪 frame 推进完全一致（T302 frame=1、T304 frame=2，advance 奇偶
   门控 fc ODD 同步）——游戏逻辑无差。残差根因定性 = **Java paint
   调度差异**（转场拍部分绘制/空拍，画布残影 vs Rust 全量重绘），
   需 shim paint 调用/时机诊断插桩专项收口。 |
   > 【2026-10-12 更正】"Java paint 调度差异"定性证伪：SRV/PAINT
   > 插桩 700/700 全跑 + FX 取证重跑（dumpStride=1）T303 = 872 op
   > 全量绘制。真根因见迭代 55。
   **迭代 53 取证（摘牌尝试=残差确认）**：itemboss40 摘除 #[ignore]
   后 T303 仍差（Rust 全量 paint_dialog 的 DES 怪 sprite 帧位差）——
   型73 boss 死亡动画的 entityAnimFrame 推进拍差需专项对表
   （battle_finishing 动画帧 advance 时机差），非单轮可闭。 |
   **迭代 52 取证**：speaker_types.txt 与 Java dialogueSpeakerType
   269 项逐项对表 0 diff（说话人表排除）——残差重新聚焦 = **型73
   boss 死亡动画相位差**（boss sprite 火焰帧 entityAnimFrame 推进，
   battle_finishing 动画拍对表待办）。
   **迭代 51 取证深化（dumpStride=1 全量 op 流 + 双上限 2048）**：
   Java T303 = 全量 873 op（实体层全画：六 poofs@y124、怪 sprite、
   状态条、+100 popup）；与 Rust op 流 872 条仅差 1（末尾
   drawSubstring 说话人名，内容同）。帧像素差 3037px 聚焦 = **型
   26/29/30/32 怪自身 sprite 的火焰发饰动画帧位差**（sptenemy 条带
   内多行动画，Java 帧位超前 Rust ~2）——entityAnimFrame 逐拍对表
   待办。ops 记录/截断双上限 256→2048 已修（flushOps + OPS.add）。
   **迭代 50 取证深化（T303 空 paint 确认）**：regen trace（cap 2048）
   T303 块 = 仅 TICK+FRAME 93 字节（0 ops 无异常）——Java mode11
   dialogPhase-0 该拍 paint 未调用或空体；A1==A2 双跑一致（非
   墙钟非确定）；Rust 为全量 paint_dialog。待 shim 诊断专项：
   paint 调用计数/时机插桩 → 空拍语义对表 → 摘除挂起。
   > 【2026-10-12 更正】"T303 空 paint" = **dumpStride=8 采样伪影
   > 误读**（run.py 硬编码 stride=8；非采样拍 postTick 不落
   > opsBlock，下拍 flushOps 覆盖）：FX 取证重跑（stride=1）T303
   > = 872 op 全量，与基线 700 帧 sha 全同——paint 每拍都在画。
   **迭代 47 续办（战斗拍台账 + 重派发高度语义修复）**：① Java 侧逐拍
   台账建账（T296-311：f_int_151 0→1 冻结、popupV=[150,100] 首现
   T302）——击杀拍与 Rust 完全对齐，「击杀拍差」假说二次证伪；
   ② **重派发高度语义修复**（a.java:6297 dispatchSpriteSize 无高度
   参数，高度 = entityTypeImage 自身高）：Rust advance_frames 重派发臂
   传 Some(sprite_h=29)（remove_des 的 DES 高度驻留）→ 类型图高≠29 的
   实体重派发后按错误高度绘制（越界读=火焰状杂纹）——改为取类型图
   自身高（entity_imgs 参数入 advance_frames）；③ itemboss40 的
   T310-700 段随之收口；**T303-309 段残差仍开放**（DES 登场泡绘制
   相位/源——泡绘制链 src/dst 逐字段对表待办）。**教训入册**：断言
   中和式验证产生假绿（迭代 49 首轮"通过"实为 assert 被临时 eprintln
   替换后未恢复）——挂起用例的复验必须恢复原断言后重跑。 |
   **迭代 55 收口（itemboss40 残差闭合，#[ignore] 摘除）**：真根因 =
   **对话气泡标记用错图集**——a.java:2887 标记图 =
   f_Image_arr2_00[12][f_byte_arr_22[dialogPage]]，row 12 =
   **face 包**（resourceNames[12]="face"，12 张全 26×22，hexdump
   零余字节）；Rust paint_dialog 误用 `images.entity[bubble]`（实体
   sprite 条带 245×39）→ T303-306 每拍在全屏 clip 下多画一条
   245×39 条带（x120 起）≈3037px 残差带。仪器 = FX-itemboss40
   取证变体（run.py 新增 dump_stride 参数）：T303 双侧 op 流
   872 vs 872，前 871 条逐字节全同，唯一分歧 op
   `drawImage(26x22,120,68,0)` vs `drawImage(245x39,120,68,0)`。
   修复：SceneImages.faces + paint_dialog 改 face 表 + 全构造点
   load_container("face", 12)。preset_itemboss40 T1-700 全帧一致，
   20/20 全绿 0 挂起。连带证伪：迭代 49/51"泡带帧位/火焰发饰"、
   52/53"boss 死亡动画拍差"定性（错图像素被误读为对方绘制差）；
   击杀拍/popup 链/entityAnimFrame 对表结论仍有效。 |
   **迭代 56（floor50 直达 L3 + 型72 层相关类别修复 + 终局链取证）**：
   ① preset_floor50 T1-300 全帧一致（maplv50/sprite50 fixtures 首装配；
   13×13 特例层；型72 最终 boss + 型83 触发器同格 (6,5)；首例 facing=2
   出生渲染）；② **真缺口修复 = renderCategoryByType[72] 层相关**
   （a.java:9669-9667 m_122 尾：floor50=8 可战斗/其余=32）——Rust 静态
   render_category 令 boss 走 cat32 臂（缺 4px ENTITY_LIFT），T237 op 流
   376 条唯一分歧 `setClip(136,113)` vs `setClip(136,117)` 实证；新增
   GameScene::entity_category（floor 感知）替换 6 处调用点；③ 终局链
   Java 侧取证（A-preset-ending 1500 拍）：撞格 (6,5) 先触发型83 触发器
   （param16 → script16 = TAK_123_126 GUT_37）→ GUT 跳 script37 =
   GLV_1 → 行尾冻结（mode3/dphase4）；键向表 a.java:5312（-1上/-2下/
   -3左/-4右，keyHeldCode 50/56/52/54 等价）；mode20 机制全解
   （a.java:3039-3067 paint / 4443-4450 m_000 入口 / 4223-4228 任意键
   回主菜单；end 卷轴 240×145 + dialogueTexts[260] 打字机 (fc&3)>>1）。
   **下迭代专项**：场景引擎 GUT/GLV 臂（script.rs 有、scene.rs 无）+
   行尾冻结 scene 语义（mode11→3 回转位置）+ mode20 实现（含
   SceneImages.ending 装配）+ 终局编排节拍（与 72 开战路径）。撞格 83
   臂已实现（interact_with_cell 83 arm 实证）。 |
   **迭代 57（终局链 Rust 前段全通 + mode20 实现 + 撞格停扫语义）**：
   ① 场景引擎补 GUT/GLV 两臂（GUT=a.java:8308-8311 旗标+loadLevelScript
   **命中即 var8=0 停扫**——缺停扫曾同拍同时触发对话+战斗，T301 状态
   dump 实证；GLV=8464-8466 纯推进）；② **mode20 全实现**：paint 臂
   （a.java:3039-3067 滚动/信箱/打字机 (fc&3)>>1，状态推进在 paint 内
   Java 原样）+ tick 任意键臂（4223-4228，game_mode=1 反向移交为已知缝，
   对拍窗口无键）+ END 入口 init（4443-4450 typewriterPos=1+残页 layout）
   + SceneImages.ending（end 包 240×145）全构造点装配；③ 撞格对话的
   walkPhase 净效应（tryStep wp=1 被对话链复位——T304+ wp=0 dump 实证，
   不复位则解冻后玩家滑步 T766 分歧 1045px）；④ preset_ending T1-907
   全帧一致（#[ignore] 挂起）：撞格83→script16 五页→GUT_37→GLV_1→
   冻结→mode3→-4 再撞→型72 cat8 开战（T902）→ case72 → script65
   （T912）→ END → mode20（T1152）→ 打字机满 46 字（T1240）静态——
   Java 侧全锚齐。**待办**：T908 起战斗节拍差（Java 901-911 共 11 拍 vs
   Rust 7 拍，itemboss40 同机制全绿 ⇒ 差异在 dp4 冻结态开战路径）。
   教训入册：先插方法后全局替换 ⇒ 委托变自递归（TCO 静默死循环 48min，
   gotcha_global-replace-hits-newly-added-method）。 |
   **迭代 60（floor35 铺层 + mode13/99/18 台账更正）**：① mode13/99
   全量 grep 无入口 ⇒ **死内容**登记（照 mode19 例）；② **mode18 行
   更正**：粒子系统 Rust 已实现（title::Particles，a.java:9830-9873
   忠实移植），boot_menu T1-150 粒子窗 + prologue case14 全帧绿＝L3
   （原 ❌ 系陈旧误报）；③ 楼层铺 floor35：fixtures + preset-floor35
   场景 + **bossEventSpawns[7] 补齐**（Java 表 8 条，Rust 原仅 7——
   case69 的 spawnBossEvent(7) 曾必越界；全 8 条逐元素对表 ALL MATCH）
   + preset_floor35 场景（Java 全链锚齐：T302 script24 五页 → T648
   冻结 → T762 开战 → **T768 case69→event[7]+script36** → T904 静态）。
   Rust T1-907 一致；T303 起整屏 ~46% 像素差。**迭代 61 实测定性**：
   双侧 animBob 均 0、型69 臂落点 (107,113±bob) 一致——非 sprite 位差；
   真差异 = **对话相机 Y 目标**：Java camY 自 -50 缓动收敛 **-82**
   （cameraTarget=226，非 speaker boss py192/玩家 160，Rust 收敛 -48
   且随页振荡）——floor35 对话相机锁目标语义待考
   （openDialogPage/lockCameraOn 链），#[ignore] 挂起。
   **迭代 63 续（收尾单点精化）**：T924 起差异 = 关闭后相机 Y 收尾——
   Java camY -50→-66 两拍后冻结（walk 边缘跟随 -8×2 至 py+cam+16>146
   的稳定位），Rust 收敛 -48 持续——mode11→3 翻转与 walk/ease 交错次序
   为最后待考单点（探针已清，注记精化）。
   **迭代 62 相机目标时间线全解**（cameraTarget FLD255/256）：页
   101/103/105（speaker69）→ (160,192)=boss px+32（a.java:8668
   var1==69 → cameraTargetX+=32 实证）；页 102/104（speaker0）→
   (128,160)=玩家；cam 缓动交替收敛 **-82（boss）/ -50（玩家）**；
   T642 关闭 → lockCameraOn(0) → (128,160) 收敛 -50；T762 再撞 →
   script36（页 162 speaker0）→ T902 关闭 → camY -50→**-66** 后冻结
   （dp=4/mode3 冻结截断闭合缓动半程——末段 16px 下移目标待考）。
   **CI 收敛确认（2026-13T03:30）**：feb6cb3 补交后 `check` 步 CI
   **success**——迭代 57 起的全红根因（E0063 缺字段）闭环。
   **迭代 63 真根因修复（floor35 T303-923 全帧一致）**：首差异 = 对话
   相机 X 缓动首步 ±10px——**lockCameraOn 型69 规则缺失**（a.java:8667-8668
   `var1==69 → cameraTargetX += 32`，全调用方共用）：open_dialog_page 锚块
   覆写 ease_target 时丢失 +32 → 相机不动（Java 首步 -24→-34）。修复两处
   （lockCameraOn 内 + open_dialog_page 锚块覆写点）。相机 Y 收尾已对齐
   （-66 = walk 边缘跟随稳定位）。**剩余 = 封门实体消失差**：T924 起
   Java 移除封门 (5,3)（玩家踏 (4,7) 后门消失露出墙，1198px；移除路径
   待考——非 bump/非 seen skip），Rust 未实现该移除 ⇒ 门残留。
   **迭代 64 赋值点定位**：T924 差异 = 门实体 drawImage src 45 vs 0
   （anim_idx=6/frame=1/solid=1 vs Java row0/frame0/solid=0）——赋值点 =
   **m_089 的 4 实体 markRemoved 方法**（a.java:7892-7919，var4-7 齐
   markRemoved 即 return true；触发链待考）。
   【2026-10-12 更正（迭代 64 终）】上记 m_089 定性**证伪撤回**——
   m_089（activateItem case 16/17 邻格门控）与本差异无关。真机制 =
   **paintTileLayer 头段的 walkPhase=4 组开门动画**（a.java:6990-7015）：
   对话关闭（T903 mode3）后脚本行走末步踏上 (4,6) **型76 组门踏板**
   （applyStepCellEffects case 76，a.java:5655-5705：param+1 组号 →
   f_short_arr_03 记录同组型4 封门 + wp=4，Rust 臂 scene.rs apply_step_
   cell_effects 早已实现）；wp4 期 tile 层 f_int_54 ±2 交替（Y=20±2），
   **每 paint 推进**计数 f_int_55：cnt=18 → 门 entityAnimRow=6/entity
   State=1（切 sptmap[12] opened_door，T924/926/928 = 帧 0/1/2，A 侧
   stride=1 trace OPS sx=0/45/90 实证）；cnt=24 → removeGroupAtCellBy
   Type(4)（T930 门除，**重置拍 tile 偏移仍取 20+bob=18**——a.java:7015
   `var2 = 20+f_int_54` 在 wp 清零后于块内计算）。逐拍锚（A35s1
   stride=1）：T907 wp=4/cnt=1、T924 cnt=18、T930 wp=0/cnt=0。
   **迭代 63 终勘误（67137f1 后续）**：门"移除"定性撤回——T924 起真
   差异 = **walkPhase4 tile bob 相位差**（双侧均进 wp4=tile24 震动砖
   态，Java tile 层 Y ±2 交替 setClip y 16/20 实证，Rust wp4 进入晚
   ~11 拍 ⇒ bob 相位错位 1198px）。**深层根因 = mode11→3 翻转时机**：
   Java 于 T903 翻转（相机未到位即翻转），Rust 于 ~913（收敛后）——
   **Java 对话期 cameraX 恒 128（=玩家 px，未向 boss px+32 缓动）**与
   a.java:8668 的 +32 规则矛盾——lockCameraOn 调用链专项（下一迭代
   首项）。
   【2026-10-12 更正（迭代 64 终）】上记两处定性**证伪撤回**：①
   "tile24 震动砖"——wp4 触发者是 (4,6) 的**型76 组门踏板**（非 tile24
   震动砖机制，tile 层 ±2 是 paintTileLayer 对 wp4 的遮幅表现）；②
   "mode11→3 翻转时机差"——stride=1 取证双侧 mode3 均 T903，翻转
   无差。真差异即上条更正所述 **wp4 推进的 paint/tick 挂点错位**：
   Java 在 paintTileLayer（paint 时点）推进，Rust 原挂在 tick 尾且先于
   advance_frames ⇒ 同拍门被多推一帧（T924 Rust src=45 vs Java src=0）
   + 重置拍 tile 偏移 20 vs 18（T930 一像素差）。修复：推进挪
   paint_tile_layer 头段（advance_open_group_paint，paint case 3/9/11
   
   **迭代 65（floor10 铺层：哪吒+门卫环+SWD 变换首验，楼层 L3 9/55）**：
   preset_floor10 T1-1200 全帧一致（一次收敛）。链：-2 光标 → -5 载入
   (6,8) → -5 关横幅 → 北行穿 (6,7) 隐形81 → 撞 (6,6) 门卫+83(param3)
   → script3（TAK_37 → MOV_0_6_5 脚本行走进环心 → DES_44/46×6 →
   CES 44/46 门卫×8 → SWD(81→4 显形封印门) → TAK_42）→ 关闭。新移植：
   ① 交互臂修正（a.java:5391-5530 cat1 switch **无 case 5**——门卫 bump
   放行、开启臂在 applyStepCellEffects case5；case 4 恒拦截+文案+停扫
   var8=0；case 81 **仅可见时**拦截——隐形期 continue 放行，floor10
   南北隐形封印门可穿为差分实证）；② 场景解释器补 MOV/DES/SWD/CES 四臂
   （a.java:8271/8311/8336/8386 逐臂对表：MOV 3 参=玩家脚本行走 findPath
   +dp3+lockCameraOn(0)；DES 负参批量/正参格上单只；SWD 81→4+DES 状态
   动画 state=2 行 8；CES=spawnEntity+markEntityDesRemoved+相机缓向新
   实体+scriptStepDelay cat 表）；③ dp3 脚本行走相位（a.java:4077-4114
   弹步 tryStep/8px 步进无边缘跟随/32px 归格）；④ **try_step 的 mode11
   强制 wp=0 权宜移除**（a.java:5341 无 mode11 特例——旧权宜会杀 dp3
   脚本行走，T504 实证）。
   【迭代 65 续：83 触发语义三判 + 回归根因】floor35/alchemy 回归逼出
   83 剧情点的完整触发模型（三处修正一次到位）：⑤ **bump 侧
   tryRunScene(param, false)**（a.java:5508 原文第二参 false）——
   script3 类 `gate &= from_walk`（门控表）使 **bump 不触发、applyStep
   到达才触发**（floor10 T401 实证：Java -1 撞 (6,6) 先行走 402-405、
   到达才开 script3 对话 408）；⑥ **var6 赋值仅在命中分支**
   （a.java:5509 `var6 = loadLevelScript(...)` 在 tryRunScene-true 内、
   且 loadLevelScript default 恒 return false ⇒ 命中后 var6=false 不置
   wp；seen 跳过时 var6 保持 true）——击杀后重步链（tick_battle 完成
   臂 try_step(facing) ⇒ T768 wp=1）依赖 seen-true 放行：floor35 T904
   实证（重步被堵 ⇒ wp 恒 5 ⇒ 对话关闭后无 (4,6) 行走 ⇒ wp4 组开门
   链消失）；⑦ 交互臂 catch-all 陷阱：新插 `_ => {}`（原意为 case 5）
   会**吞掉其后全部 match 臂**（83/9/77 静默失联，floor35 T301 实证）
   ——Rust match 首中语义，移植插臂必须用显式类型码。
   次级发现：**spriteN/maplvN = floor N（1 基，N∈1..54；floor
   55=ending 无需资产）**——早前按 0 基误读的层扫描表已作废。
   挂起：dp2=m_106 实体路径缓动（本窗口无 5 参 MOV 触发，
   todo 在位）、入口脚本 n[10] 牛魔王 CES 链（非本窗口路径）。
   【迭代 66（floor49 铺层：case70 击杀钩子 + 守卫宝箱配对开门 + script15
   伏击，楼层 L3 10/55）】preset_floor49 T1-2600 全帧一次收敛。链：载入
   (6,11) → (6,10)【陷阱首爆：60 邻接 -200】→ (5,10)/(7,10) 门卫@2（同格
   60 敌 bump 战×2）→ wp4 开 (6,9)@2 → (6,8)=83 首访 inert → (5,8)/(7,8)
   门卫@3（65 敌战×2）→ wp4 开 (6,7)@3 → 二访 (6,8) → script15 伏击
   （ROS_4_1 + CES_61×8 + CES_70(6,3) + TAK_118_121）→ 北上 61/70 战 →
   case70(floor49) → script35。新移植/考证：① **陷阱扣血臂整体移植**
   （a.java:5549-5645：61 夹击 -HP/2、62 邻接 -100、60 邻接 -200 + var4
   短路三分支场输入、armor40 火浣衣免疫、m_101 火焰上爬=拉旧格+上格重生
   DES 态 60——floor49 T305 实证缺口：Java -200 浮层 + HP 500→300，Rust
   无此臂）+ **tryRunScene 门控全表移植**（补 case 7/32 todo=m_102(23,12)
   跨层快照、15=m_102(currentFloor,5)?false:from_walk【守卫全清才可触发
   伏击——(6,8) 首访 inert 实证】、16/18/24/25/26/27/31/33/34 简单门、
   8/9/11/12/19/30/41-50/52/53/55/56 from_walk 族——旧表仅 0-4/46/47/66/67）。
   ② **type 5/81/4 精灵 param = extra+1**（a.java:9499-9502、83 无 +1
   =原字节）——floor49 守卫@2/3 ↔ 封印门@2/3 配对（tryOpenChest 同 param
   型5 存在即拒、型4 入组 wp4 开门；removeSlotAt 置 removed ⇒ 守卫清空
   是 case15 门的达成条件）。③ **mode18 火花系统 = 原版死代码 A 级**
   （a.java:3010-3037 双缓冲火花：f_byte_20 仅 3035=sparkCount 写、
   addSpark 仅循环内调——自播种为零闭环，randomBelow(15) 永不消费；
   Rust 无此臂 = 零行为差，复核登记关闭）。 |
   【迭代 79 终（B2a 敌族账本归零 ✅：floor47/34/10/10b/17 五窗五绿）】
   route_gen.py 固化手写路线流程（航点手拟 + BFS 走步填充 + 三重校验——
   修 door/bump 邻格选择 bug 后稳定）。覆盖增量：**41/42/43（floor34）、
   44/46 bump 战（floor10 NW——旧用例的 44/46 系 DES/CES 脚本臂，本批补
   bump 战语义）、45（floor10b）、48（floor17）、63（floor47）**。至此
   **敌族账本 41-79 全谱 ✅**（余 53 战=蓝门区、67/68/69/71/74=B3 boss）。
   全套 84 用例 0 失败 0 忽略。 |
   【迭代 79 续（floor38 铺层 ✅ B2a 敌族取样②，首跑即绿）】preset-floor38
   （出生 (10,11) 南段、存档黄钥=3），T1-6050 全帧一次收敛。覆盖增量：
   **47×2/49/56/58/59 六型七战**（金币 100→553 七连增帧证）+ 黄门×3
   ((8,11)/(4,11)/(11,8)) + **47 bump 时北守卫 57@2 未死 ⇒ 守卫-宝箱封印
   拒绝语义同证**（密度取皮书）。北半（55/78 商人 (5,2)/81 隐形封印门
   守卫联动 57@2）另窗处理——81 门=玉皇式守卫封印链的样本。 |
   【迭代 79（floor14 铺层 ✅ B2a 敌族取样①，首跑即绿）】preset-floor14
   （出生 (2,11) 左段、存档黄钥=3），T1-4790 全帧一次收敛。覆盖增量：
   **49/50/51/52/54 五敌型首战**（六型七战，金币 100→208 七连增帧证）+
   黄门×2 (3,9)/(6,8)。53 三兄弟+5@0 宝箱链围困蓝门区（无蓝钥）留待带钥
   重入。**路线方法论入册**（floor8/14 三轮教训）：① 手写短路线（规划器
   过度工程，用户裁定）+ 三重校验（连续性/可走/键向——曾漏相邻性致斜跳
   漂移）；② 存档参数按 preset_rms 布局逐字节核对（黄钥=b[23]，b[13] 是
   playerHp 字节——误写曾致 HP 1012 + 门钥不足）；③ 出生格须查"口袋地
   形"（floor14 右口袋唯一出口=楼梯踩上换层）；④ 取证直接看 trace 数值
   字段曲线（FLD200 金币/FLD083-084 像素坐标），帧面对比易误读。 |
   【迭代 78（floor41 铺层 ✅ P2 B1 首层，楼层 L3 14/55——首跑即绿零新
   移植）】preset-floor41 场景（MOT_L0 current_floor=41，出生 (7,11)），
   窗口 T1-1900 全帧**一次收敛**。覆盖增量：60-64 敌表族实战（64@(8,7)、
   62@(9,5) 两场 bump 战——HP 500→750→650、金 100→140→220 帧证）、
   **(10,2) 墙格双 83 触发器**（extra=0x0d/0x19=script13/25，同格双录
   sprite41[50][51]，bump 触达语义首验）、红门无钥提示浮层（(7,6)）、
   黄门开钥消耗（11,2，钥 2→1）、道具拾取（30@(8,11) 出生格踏格拾取
   HP+250）。fixtures：maplv41/sprite41（270B 零余）；路线脚本
   preset-floor41.txt（可走网格经 tile_walkability BFS 预核：(10,2)
   为墙格 ⇒ 83 走 bump 侧）。 |
   【迭代 77（floor46 摘挂 ✅ 楼层 L3 13/55，全套 24 绿 0 挂起——历史首次
   无挂起）】**根因 = apply_merchant_offer 缺 a.java:7456 臂**：购买路径
   `if(var1){ if(flags&2!=0){…} else if(flags&1!=0){ param^=256 +
   markEntityRemoved } }`——一次性商人（flags&1）**买/拒皆消耗**（拒绝臂
   7462 已有，购买臂 7456 缺）。floor46 链实证：T449 键 **-5（购买非拒绝）**
   → flags=0x01（&2==0、&1!=0）→ Java 商人 T450 起消失（白烟帧 0 → T451
   帧 1 → row6 表尾摘除），Rust 缺臂滞留本体（413px 单点）。
   **取证链（模板匹配 + OPS + 记录表三重）**：① sptenemy2[16]=29×48 白发
   老者（entityTypeImage[77]）、sptprop[3]=30×27 白云（cat16 底座，
   entityTypeImage[15]）——两侧都画（T449 Java OPS：30x27@(105,89) +
   29x48@(105,57)）；② tRNS 透明掩码模板匹配：Java T450 帧全窗口无
   sptenemy2[16] 像素（973/973 错）⇒ 老者非换形态而是**被移除**；T451
   OPS `drawImage(135x29,106,82)` clip 27×29 = open_anim 白烟；③ sprite46
   (8,9) 单条 t=77 extra=0x13——无独立云实体，云=cat16 底座随实体消亡。
   **挂起注记勘误**：迭代 67/68 的"白云形态 vs 真身立绘"定性**证伪**——
   所谓"Java 白云"实为 **open_anim 白烟帧与 sptprop[3] 云底座位置重叠**
   （(106,82) vs (105,89)，27×29 ⊂ 30×27 区域）的误读；sptenemy2[16]
   解码零差异、云底座两侧都画等结论仍成立。
   随修 **build_minimap 忠实化**：markEntityRemoved 仅 cat1 分支调
   buildMinimap（a.java:6367）——Rust 曾 cat16 商人移除也重绘（多）/
   脚本 REM·SWD(型11)·85-op(型1)·remove_adjacent（cat1 族）不重绘（少），
   统一改为 entity::remove 返回 cat1 判定驱动。flags 三判入册：
   **&1=一次性（买/拒皆 ^256+移除）、&2=多商品持久（购买循环+收尾块）、
   &4=防移除防护**。 |
   【迭代 71（floor43 摘挂推进：7868→413px）】两项 A 级修正：① **m_083
   先减后比**（a.java:7693 `while (--var3 >= var2)`——扫描 [lo,hi-1]，
   hi 永不触达；Rust 误译后减致 [12] 越界 panic）——修正后 80 甲拾取
   装备成功（def=150、arm=80 与 Java T1208 实值一致）；② **装备描述盒
   双参分解**（a.java:7595-7597：图标 itemUseTargetType=var1 物品类型、
   文本 equipDescriptions[var2]=tier——单参混用致 D 表越界错帧）。
   【迭代 74-75：单点三分解构】D 表重建（10 条截断文案补全）后单点变为
   **724px 盒文本换行差**：Rust 4 行 vs Java 3 行（绿/黄段接排错位，
   f43_boxtext_1205b.png）。初判 layout_wrapped_text 对 \c 转义宽度计量
   差——**埋点后修正定位**：a.java:5038-5042 `overlayLayoutCache`——
   paintWrappedText 每次 paint 以 **paint 参数**重跑 layoutFullText
   （show 时 w=180/h=234，paint 时 w/h 另有值），Rust paint_overlay_box
   的重排参数与 show 时不一致 ⇒ 行数差。待修：paint 时重排参数对齐
   （overlayLayoutCache 缓存语义）。同轮关闭：sptarm[9] 解码经 PIL 对照
   无误。
   【迭代 76 摘挂 ✅ 楼层 L3 12/55】paint_overlay_box 重排参数对齐
   （paint_hud 装备槽块移至 paint_status_bar 后的 HUD 路径修正）+
   装备槽图标/双参盒/m_083 全部落地后 T1-1700 全帧绿，preset_floor43
   摘挂（同轮 sptarm[9] 解码 PIL 对照无误，"解码颜色失真"证伪关闭）。 |
   【迭代 68（floor46 摘挂考证：单点精确化）】
   【迭代 70（floor43 铺层：80 银缕锁甲 + 64
   【迭代 70（floor43 铺层：80 银缕锁甲 + 64/65 敌表族，挂起）】
   preset_floor43 T1-拾取前全帧绿，(9,4) 80 银缕锁甲拾取 panic 挂起：
   m_083 装备档位查表 equip_types[12] 越界（表 12 项 vs 80 型 tier 索引
   12）——装备档位语义待考（80 甲的 tier 映射/m_083 边界）。已验证段：
   载入 (8,4) → (7,4) 65 战 → (6,4) → (6,3) 65 战 → 回 (8,4)（双 65
   敌表战全帧一致）。 |
   【迭代 69（floor42 铺层：DES 剧情移除双 boss 首验，楼层 L3 11/55）】
   preset_floor42 T1-1700 全帧**一次收敛**（零新移植——script14 的
   ROS_4_1/TAK/DES 臂全部为迭代 65-68 已移植件的首实战）。链：载入
   (7,11) → (7,10) → (6,10) 踩 83(param14) → script14（ROS_4_1 +
   TAK_92_97 六页 + DES_73_6_8 天蓬移除 + TAK_98_100 三页 + DES_70_6_7
   玉皇移除——双 boss 剧情消失非战斗）→ 关闭 → -5 field wp2 数字视。
   覆盖增量：DES 臂多类型实战（73/70）、83 param14→script14 的
   from_walk=true 到达触发（iter 68 语义的正面用例）、9 页长对话翻页
   节奏。 |
   【迭代 68（floor46 摘挂考证：单点精确化）】stride=1 取证（A46s1）+
   逐像素定位把 T451 分歧收敛为**单点：玩家/NPC 绘制顺序**——受领拍
   （-5，flags=0x1 仅关浮层）NPC markEntityRemoved → state=1 → cat1 画
   opened_door 白烟帧 0（两侧绘制一致 ✓）；分歧 = 玩家头饰 Java 在白烟
   **上**（paintEntityLayer 循环外兜底后画）、Rust 被白烟**盖**（循环内
   先画）——NPC 不满足玩家插入条件（npc_y < player_y）时的玩家兜底绘制
   时序待移植。同轮证伪两项：① "NPC 受领即移除"——实为 state=1 白烟绘
   制（remove 反致白烟消失，已回滚）；② "实体生成序谜题"——取证脚本
   LE/BE 误读（seed readU16BE），实体表两侧一致（T1-450 绿含 NPC 本体
   绘制）；77 双字节读经 sprite2 152/152 实证不成立（文件布局 1 字节，
   extra_len 回滚留注）。
   【迭代 68 续：埋点复核】paint_entities 玩家路径实证 = **FALLBACK
   （循环后）与 Java 同位**——Z 序非分歧源；残余差异 = **白烟内部纹理**
   （Java 致密 vs Rust 透底呈砖纹——opened_door 帧序/帧偏移待查：
   NPC anim_idx 应恒 0/帧 0，两侧公式一致却绘出不同帧，疑 Rust 帧字段
   被他人改写或 draw_image_clipped 的帧偏移取整差）。挂起注记已同步。
   > 【2026-10-14 更正（迭代 77）】上记"白烟内部纹理"定性**证伪**——
   > 分歧非烟纹理，而是**Java 侧烟 = 商人移除态（缺 7456 臂致 Rust 画
   > 本体）**；"两侧绘制一致 ✓"与"NPC anim_idx 应恒 0/帧 0"前提不成立
   > （NPC 实际进入 row6 白烟动画）。详见迭代 77 条目。 |
   【迭代 67（floor46 铺层：85 黄金钥匙 + 77 太白金星告示，挂起）】
   preset_floor46 T1-450 全帧绿，T451 起挂起：① **77 太白金星绘制态分
   歧**——Java 受领拍（-5，flags=0 仅关浮层）后 NPC 置 state=1 画
   opened_door 白烟，Rust 仍画本体（差异区 30×55 @ NPC 本体，逐像素实
   证）；② **m_122 实体生成序 ≠ sprite 记录序**（A trace 头
   1,3,60,9,8,31,... vs 记录序 1,1,...,3,3,62...——生成循环遍历语义待
   考）；③ 77 的 seed 双字节读（a.java:7321-7331）经 sprite2 152/152
   实证**不成立**——文件布局 77/78 = 1 字节（extra_len 曾误改 2 已回滚
   并留勘误注；door12 fixture 22 项修正同样回滚——13 项与 seed 526 行
   冲突待裁）。已确认无损件：交互臂 bump 77 → 报价入队 + kind4 浮层
   （T352）、-5 受领仅关浮层（flags=0 路径，brace 错位曾致 ^512+移除
   NPC——已修）、mode4 菜单开合（T552-652）。 |
   【迭代 65 对抗复核新登记】① **mode18 paint 期火花分支缺失**：
   a.java:3014-3034 paint case 18 对**每活跃粒子每 paint** 消费 1 次
   randomBelow(15) 并按位 addSpark ≤4（RNG 流消费者 + 视觉元素）——
   Rust Particles::paint_and_update 无此臂（title.rs:93-125 仅画+推进）。
   【2026-10-13 更正（迭代 66）】**证伪关闭**：该分支为原版死代码——
   火花缓冲 f_byte_20 全部写点 = 2207（分配置 0）与 3035（=sparkCount），
   addSpark 全部调用点都在 f_byte_20>0 的循环内 ⇒ 自播种为零永不激活，
   randomBelow(15) 在原版也永不消费 ⇒ Rust 缺臂零行为差（A 级：写者图
   完备）。② 遗留探针清理：
   preset_cases floor35 的 F35DUMP（PPM 落盘）已除；paint.rs:571
   PROBE_SLOTS（迭代 22 起，env 门控惰性）保留待查。③ 未用变量警告
   5 处（records1 类影子绑定：alchemy/save 系 4451/3758/2038/1813）——
   floor10/floor35 已核实无此问题，余者不涉换层窗口。 |
   **迭代 59 实现收口（mode12 L3）**：preset_floor5 T1-1200 全帧一致。
   ① save.rs：MOT_L 日志槽（f_int_89 对 = f_byte_arr_14/15）解析入
   FloorSave.journal（原 skip）；② 场景五件套：activate_item case14
   （game_mode=12 + open_record_page(0) + 软键右3）、open_record_page
   （m_076 钳位翻页 + kind 选表 + 页码指示）、paint_mode12（视差 +
   m_142 指示 + 双箭头 + 软键 + **公共尾 overlay 框**）、draw_shadow_text
   （m_142 8+1 描边：绕圈次序 7 步 delta + 末次 var2++ 主色落位居中点——
   68px 色反转定位）、overlay kind5 臂归位（-3/-4 翻页/-2/-1 滚动/
   -5/-6/-7 关回 mode10；商人报价浏览实为 kind4，merchant 场景无
   -3/-4 按键故重挂安全）；③ 拾取链 overlay_kind_item 补设（描述框
   图标 515px 缺失定位）；④ **itemStackUses 有符号 byte 语义**：
   -1=无限次（useItemStack else 无条件 activate 不计数 + 面板不画次数）
   ——u8 直读曾把 -1 读成 255 画出次数（T501 132px）。碰到的新坑：
   mot_if 元组 s[2]=武器类型（f_byte_arr_25，equipmentMaterialNames[-28]
   崩溃实证——负索引无钳位）；首拾描述 overlay 吞下一键（-6@380 被吞
   实证）。 |
   **迭代 58 收口（终局链 L3，mode20 摘挂）**：真根因 = 击杀动画完成臂的
   **类型钩子（a.java:10110-10145）未实现**——动画完成后 Rust 无 case72
   分支（script 停在 37、restep 滑入空格），"T908 战斗节拍差 11 拍 vs 7 拍"
   系采样拍误读（下游症状非相位差）。补齐 61（49 层封印门 puzzle 全条件）+
   69（35 层→spawnBossEvent(7)+script36）+ 70/71（49 层→script35）+
   72（50 层→script65），类型在 battle_target 清空**前**读取（restep 可开
   新战斗改写 target，Java 同序）；m_100 = entity_at_cell 辅助。摘除
   #[ignore]：preset_ending T1-1800 全帧一致，**mode 20 L3 达成（L2\*
   过渡类消灭）**，全套 126 通过 0 失败。 |
   **迭代 59 考古预登记（mode 12 = 下一实现目标）**：mode 12 = **道具 14
   记录浏览器**，可达性实锤——全 55 层 sprite 扫描：type14 实体唯一
   @floor5 (4,9)（55 层无缺文件，sprite55 不存在=该层无 sprite 数据）。
   语义：activateItem case14（a.java:7790 gameMode=12+m_000）→ m_000
   case12 = m_076(0)+softkeyRight=3（a.java:4386-4388）；m_076（7480-7503）
   = f_int_89 总条数钳位翻页 + f_byte_arr_14[kind]/15[idx] 选
   f_String_arr_06/09 文本 → showOverlayMessage(kind5) + f_String_04=
   "n / N"；键控走 overlay-kind5 分派（a.java:3325-3334：-3/-4 翻页、
   -5/-6/-7 关回 mode10）；paint case12（2866-2880）= backdrop(false)+
   m_142 页码指示 + 双箭头（f_int_90 钳位 paint [8][14] bob 摆动）+
   softkeyBar。**待查已清（本轮）**：数据表 = **SKY_WAR RMS 持久化日志**——
   f_int_89 = readShort（a.java:9275，m_110 读档链）；f_String_arr_06/09
   = init 常量字串表（a.java:823/887）；m_142 = **8+1 描边文字**
   （f_int_arr_36[0] 暗色 8 向 ±1 偏移 drawString + [1] 主色 1 次，
   a.java:10221-10236，页码 "n / N" 专用）。**下轮**：SKY_WAR 写入点
   （f_int_89++ 日志追加语义）→ preset SKY_WAR 预置格式扩展 →
   preset-floor5 场景（maplv5/sprite5 fixtures + 拾取道具14 + 面板使用
   → mode12 翻页/退出）→ Rust 实现（activateItem case14 + m_000 case12
   + m_076 + paint case12 + m_142 + overlay 分派 case5 归属核查〔Rust
   现将 kind5 用于商人报价浏览，Java case5=m_076——需甄别〕）。另：台账 mode18 行
   陈旧——粒子系统 Rust 已实现（title::Particles，boot_menu T1-150
   粒子窗全帧绿 + intro case14），行更正待下迭代补证。 |

口径教训：名字/行号自动匹配只能出候选名单，分子终审必须人工（本轮抽样 7/7
候选"未移植"实为内联移植）。三档随迭代人工更新，工具只保证候选不丢。

## 3. 分子登记（人维护，工具校验）

每迭代语义回流步骤更新本节矩阵；登记格式 = `mode/指令/层 × 等级 × 证据`。
证据示例：`scenario:`preset-save.txt` T340-700 L3`、`trace:`A-preset-save``。

### 3.1 模式面矩阵（B 维主表，2026-10-12 基线）

| mode | 语义（state-machine.md） | Rust 实现 | 验证级 | 证据 |
|---|---|---|---|---|
| 0 | boot 启动屏 | ✅ BootMachine | L3 | scenario:`boot-menu.txt` |
| 1 | 主菜单 | ✅ BootMachine | L3 | scenario:`boot-menu.txt` |
| 2 | 资源载入链 | ✅ BootMachine | L3 | scenario:`enter-game.txt` |
| 3 | 地图主玩法 | ✅ GameScene | L3 | scenario:`preset-floor2.txt` |
| 4 | 游戏内菜单 | ✅ | L3 | scenario:`preset-save.txt` |
| 5 | 敌人图鉴 | ✅ | L3 | scenario:`preset-item13.txt` T340-500 |
| （无 mode 6） | 战斗 = mode 3 子态（battleActiveFlag+tickBattle），非独立模式——2026-10-12 更正，矩阵初版误设 mode 6 行 | ✅ combat.rs | L3 | scenario:`floor1-to-floor2.txt`（4 战）+ `floor2-tour.txt`；迭代 35 起 Rust L3：`preset_cases::floor_crossing` T1-1500 全帧（抖动/伤害弹窗；前提 = rng 续接，见知识库 gotcha_gamescene-rng-must-continue-machine-lcg） |
| 7 | 存档槽（写回） | ✅ | L3 | scenario:`preset-save.txt` T340-560 |
| 8 | 读档槽 | ✅ | L3 | scenario:`preset-help.txt` T150-240 |
| 9 | 炼丹/加点 | ✅（迭代 31 实现 / 33 L3 收敛——本行 2026-10-12 二次校准时漏更正，迭代 37 补正） | L3 | scenario:`preset-alchemy.txt` T1-700 全帧（三连购 + 不足浮层） |
| 10 | 道具面板 | ✅ | L3 | scenario:`preset-item13.txt` |
| 11 | 对话/剧情层 | ✅（scene script/mode11 + BootMachine 新游戏链） | L3 | scenario:`prologue-dialog.txt`——迭代 37 起 Rust **全流程 T1-9800**（preset_cases::prologue，冷启动移交 + type83→script40 + 打字机/游标/翻页/电影镜头）；旧冷场景窗口版退役 |
| 12 | 道具14 记录浏览器 | ✅（迭代 59：activate_item case14 + m_076 + paint case12 + m_142 描边文字 + 日志槽解析） | L3 | scenario:`preset-floor5.txt`——Rust **T1-1200 全帧**（拾取道具14@floor5(4,9)→面板→使用→mode12 翻页/关闭） |
| 13 | 过渡（paint 仅视差，a.java:2881-2882） | ❌ 死内容 | — | 全量 grep 无 `gameMode = 13` 入口（2026-10-13）——不可达。【2026-10-15 升级】字节码值域证明：putfield gameMode 54 处值域 {0-5,7-12,14-17,19-22}（含计算来源闭包）∌ 13 ⇒ ☠️（[dead-code-rulings.md](dead-code-rulings.md) DC-3 值域；mode 13 无专属方法） |
| 14 | 新游戏引子插画 | ✅ BootMachine | L3 | scenario:`enter-game.txt` |
| 15 | 帮助浮层 | ✅ | L3 | scenario:`preset-help.txt` T450-700 |
| 16 | 设置 | ✅ | L3 | scenario:`preset-load-menu.txt` |
| 17 | 关于 | ✅ BootMachine | L3 | scenario:`menu-sweep.txt`（boot_replay 全量重放至 T1832） |
| 18 | 粒子特效层 | ✅（title::Particles：32 槽环 + fillRect + velX 摆动/size 增长消亡，a.java:9830-9873） | L3 | scenario:`boot-menu.txt` T1-150（含 T72+ 菜单粒子窗全帧绿，boot_replay）+ case14 引子（prologue T1-9800）——台账原 ❌ 系陈旧误报（2026-10-13 更正）。【2026-10-15 更正】本行把 mode 1/14 的标题粒子（paint a.java:2374/2955 调 updateAndDrawParticles）与 paint case 18（a.java:3003-3038 火花层，唯一调 addSpark）混为一谈：javap 普查 putfield gameMode 54 处无 18（含计算来源）⇒ **mode 18 不可达、❌ 死内容**（同 13/99；type-closure §3 已记 ☠️）；粒子系统 L3 归 mode 1/14。证据见 §3.4。【2026-10-15 终裁】☠️ DC-4（addSpark，[dead-code-rulings.md](dead-code-rulings.md)） |
| 19 | 商店 | ✅ enter/paint（Rust 侧） | ❌ 不可达死内容 | 原版 kind11（商店）从未加入任何菜单（a.java:4305-4333/4437-4441 全量 addMenuItem 核对，迭代 41）——activateMenuItem case11（a.java:4775）为死臂；Rust enter_menu19 同为死码保真 |
| 20 | 结束/退出确认（结局卷轴） | ✅（迭代 57 实现：paint/tick/END 入口 + SceneImages.ending） | L3 | scenario:`preset-ending.txt`——迭代 58 起 Rust **T1-1800 全帧**（终局链：撞格83→script16 五页→GUT_37/GLV_1 冻结→型72 cat8 开战→case72→script65→END→mode20 打字机卷轴；L2\* 过渡态消灭） |
| 21 | 音效询问 | ✅ BootMachine | L3 | 全部 preset 场景 T66-70 |
| 22 | 退出淡出 70 帧 | ✅ BootMachine | L3 | scenario:`menu-sweep.txt` T1762-1832（boot_replay 终点断言） |
| 99 | paint 特殊屏 | ❌ 死内容 | — | 全量 grep 无 `gameMode = 99` 入口（2026-10-13）——不可达。【2026-10-15 终裁】☠️ DC-3（字节码值域 ∌ 99 + f_bool_31 自锁；连带联网簇 m_148-m_154 七方法，[dead-code-rulings.md](dead-code-rulings.md)） |

L2* = Java 三方对拍绿但 Rust 侧未建用例（覆盖声明的过渡态）。

### 3.2 内容面矩阵（C 维）

- DSL 15 码：TAK/CES/MOV/DES/GIN/LAY/ROS/RES/GUT/SEE/END/GLV/SMS/SWD/ADD。
  已在 Java 侧验证 trace 中出现（prologue/gameplay 系列）；**Rust 解释器覆盖
  以 game-core script 引擎对拍为准——逐码登记待迭代 29+ 细化**。
  > 【2026-10-15 更正】实为 16 码：另有 **MVS**（a.java:8354 else 臂 → m_107），68 条脚本零使用、脚本文本源封闭
  > ⇒ ☠️ 死指令（DC-2，[dead-code-rulings.md](dead-code-rulings.md)）；逐码账本已迁 type-closure §2（16 码）。
- 楼层：**L3 对拍达成 11/55（floor1、2、5〔道具14/mode12，迭代 59〕、
  10〔哪吒+门卫环+SWD，迭代 65，T1-1200 全帧〕、12〔炼丹，迭代 31〕、
  42〔DES 剧情移除双 boss，迭代 69，T1-1700 全帧〕、
  49〔case70 击杀钩子+守卫宝箱配对+陷阱臂+script15 伏击，迭代 66，
  T1-2600 全帧〕、
  35〔深 boss 层+case69 钩子+wp4 组开门动画，迭代 60-64〕、
  40〔boss 击杀链，迭代 43-55〕、50〔迭代 56〕、51）**——floor51 于迭代 30 全帧收敛（迭代 29 曾暴露 ROS todo 与打字机
  限量布局转义偏差两处真实缺口，按层铺路线连续逼出真实缺口的实例）；
  floor50 直达 T1-300 全帧（迭代 56，maplv50/sprite50 首覆盖 + 首例
  facing=2 出生渲染）。
- DSL：ROS 六子码已移植（迭代 29）；布局算法双拷贝（全量/限量）已对齐转义
  语义——~~遗留重构项：抽公共函数消除双份拷贝~~【2026-10-15 销账】11ddb85 已合并
  （`layout_full_text` = `layout_wrapped_text_limited` 全长特例，与 Java a.java:4940-42 同构）；
  合并时发现冷路径偏差见下方新发现①。
- **mode 9 炼丹链（迭代 31 完成，L3）**：type9 炼丹台分支（interact_with_cell，
  a.java:5419-5429 阻挡+朝上触发）+ run case 9（买/换光标/不足浮层）+ paint
  case 9（题头框+花费行+三行属性）+ alchemyPriceFor/scaledByFloorTier。
  场景 = preset-alchemy（**floor12** type9 @cell(5,9)，(5,10) key2 向上踩台
  ——facing 语义实证：键2/-1 组 = facing1 = y-1 向上；键8/-2 组 = facing0 =
  y+1 向下），T1-700 全帧一致，A==B==C 三方 PASS。同迭代另修两处真缺口：
  拾取 floorTier 缺失（type29/30/31/32 层域倍率，floor12 +50 应 +100 实证）
  与 alchemyPriceFor 循环上界（1..k-1 应 1..k，价格 20 应 40 实证）。
  floor4 的 type9（cell(5,1)）贴 y=0 崩溃行 = 不可达死内容（BUG-009）。
- 资源：138 条 manifest 全部经 L1 golden（解析级）；渲染级随场景逐步覆盖。

【2026-10-15 卫生审计新发现（hygiene-core 11ddb85 副产物，B 级为主，待场景级定级）】

1. **文本排版冷路径与 Java 不一致**：合并后保留的 scene 版在两条冷路径与 Java 不符——
   非法 `\c` 转义跳过换行检查、未知转义处理不同；boot 侧（合并前的忠实版）语义正确。
   现有全部文本源均触发不到这两条路径（帧对拍恒绿的原因）；合并后只需修一处。
2. **存档槽装备框分支未移植**：场景存档界面总是画材质名，缺"有装备时画图标"分支
   （a.java:9116-9123、9136-9143）——存档装备≠0 时出现像素差。另 boot 侧甲胄索引
   上限 `>9` vs Java `>7`（真实甲胄类型下结果相同）。
3. **记录浏览读不到新记录**（B 级静态推断）：m_075 只写 `offer_queue_*`，全仓无读方；
   `open_record_page` 只读存档载入的 `journal` ⇒ 游戏中新获得记录不进浏览器，疑与原版不一致。
4. **打字机宽度量法偏差**：`overlay_content_w` 量截断后文本，Java 量整串——当前无读方，
   观测不到差异。

### 3.2a 收官批销账与新发现（2026-10-15 晚，Stage A 终审）

- **saveGlobalProgress 销账（判据① a 类清零）**：SKY_WAR 全局存档端到端移植
  （b62d9db）——格式层 GlobalSave（a.java:8931-8942/4463-4474 布局）+ 两机同构
  状态 + 壳持久化（损坏/缺失走 catch 语义 default_7 ⇒ [1]=true + 回写，
  a.java:4475-4478）。回标题触发点（4297-4299）随后续反向移交后置（账本裁定）。
- **型 10/11/12 撞击臂销账**（ee52191）：interact_with_cell 静默吞噬臂补全
  （a.java:5436-5456 逐值）；fx_firebump_floor13 帧验收解禁即绿（T301 钉点）。
  **覆盖缺口注记**：型 11 墙提示/型 12 撞击变形两个行为现有帧夹具未触达
  （变异存活实录），型 10 停扫分支亦然——行为由字节码背书，回归靠未来夹具。
- **m_122 层 12/39 恢复特例未移植**（B 级，B 任务交棒遗留①）：m_122 恢复臂的
  层 12/39 特例（a.java:9647-9666，当前行号 +3）未移植——重入这两层且有层
  存档时行为与 Java 不同。登记待后续批次。

### 3.3 场景面（D 维）

16 场景（tools/completeness.py 输出为准）。准入规则见 §4。

### 3.4 覆盖矩阵刷账（2026-10-15）

> 账面：[docs/coverage-matrix.md](../coverage-matrix.md)（`python3 tools/cover_matrix.py` v2.1 机器生成，
> `--check` 校验账面一致）；触达快照 `tools/method_reach/snapshot.json`（`python3 tools/method_reach.py`，
> JDI 实测）。本节只放判据与结论，逐方法明细与逐项论证只在矩阵（§4.1 三分账表）。

**数字对照**（三种口径在同一 Rust 树 + 同一快照上复算，矩阵汇总行同时打印）：

| 口径 | 绿 | 黄 | 白 | 计数对象 |
|---|---|---|---|---|
| v1（迭代 72，人工分组） | 44 | 1 | 58 | 103 伪名（v1 取名 bug：多参方法被吞成末参类型名） |
| v2（前任刷账：JDI 触达 + 朴素行锚） | 133 | 7 | 23 | 163 方法（javap name+descriptor） |
| **v2.1（本次：行锚归属治理 + 三分账）** | **143** | **7** | **13** | 同上；白 = a 3 / b 10 / 未分账 0 |
| **判据①收口（2026-10-15 二次刷账，见本节末）** | **151** | **1** | **11** | 同上；白 = a 1 / b ☠️ 10（终裁）；黄 1 带论证 ⇒ 判据①赤字 1 |

44/1/58 → 133/7/23 主因"账面滞后"（v1 人工分组漏账 + 伪名 bug，v1 白中 102 个真实方法现为绿）；
133/7/23 → 143/7/13 = c 类 11 项解决（10 绿 1 黄）+ addSpark 黄→白（唯一证据是漂移锚）。

**判据（v2.1 方法论）**
- 三色：绿 = 有移植账 ∧ Java 执行落在 Rust 全帧用例窗口内；黄 = 有移植账、无窗口触达；白 = 无移植账。
- 触达（A 级）：`tools/method_reach/MethodReach.java` 在类 a 每个方法首位置下一次性 JDI 断点，记首次命中
  t_hit = `Canvas.hooks.tick + 1`（shim `Canvas.serviceRepaints` 次序 paint → preTick(tick++) → 按键 → postTick
  已核：tick=k 时的执行最早反映在 TICK k+1 的帧）；测得运行的 FRAME 链与 Rust 用例消费的 A-trace 逐帧对照
  （42/44 equal；boot-menu / prologue-patient 无 A-trace，不计验证）。
- 可复现：本次全量重测 44 场景，phases / t_hit / frame_chain 与前任快照逐字段一致（唯一差异 = 新增 ticks_run 字段）。
- 窗口：由用例 `checked` 断言机械派生；多相位场景窗口内触达 = 各相位 ≥ 起点的并集取最小 t_hit。
- 移植账 = fn 直证（同名/蛇形/m_NNN）∪ v2.1 行锚 ∪ PORT_NOTES（ported 须带 `crates/…rs` + 标识符锚，
  absorbed 须带落在方法体内的 a.java 行锚，脚本逐条校验，失效即 ⚠）。
- v2.1 行锚归属：方言（显式 seed 锚及 seed 方言文件 → reference/seed/a.java；与 deobf 按方法序数对齐，运行时校验
  163 个、逐序参数类型一致）+ 名证锚（行内点名 Java 方法 ⇒ 只归被点名者，±20 行）+ 稳健锚（未点名 ⇒ 外扩 20 行
  仍须落在 VF 时代 15 个 deobf 版本方法区间的交集内）+ 字段初始化器行归 `<clinit>`/`<init>`。容差取自矩阵 §0
  自检：名证锚 430 条中 324 条精确重叠、22 条偏移 1-20 行。

**前任 WIP 审阅结论**
- 可靠：JDI 触达测量（A 级、确定性、FRAME 链自证"测到的就是被 Rust 验证的那次执行"）；用例窗口解析
  （逐用例核过 checked 断言形态，preset_help T240-800 等）；v1 复算口径原样保留可复现 44/1/58。
- 缺陷与修正（均在 tools/ 内）：
  1. **误归绿（主缺陷）**：行锚"落入现行方法体即算移植"。game-data 全部按 seed（CFR 冻结件）写的锚被按
     deobf 行号读（m_072 吃 28 条，实为 seed 的 sprite 读循环）；Rust 注释锚普遍漂移 1-20 行，漂进邻近小方法。
     9 个方法的 v2 移植证据全是错归属锚（m_006、releaseSceneImages、m_068、m_072、itemTypeToStackIndex、
     deleteRecordStoreQuietly、m_117、m_128、addSpark）——逐条核验后 8 个确有移植/吸收（补 PORT_NOTES），
     addSpark 实为死代码（转白 b）。另 spawnEntity / consumeKeyForDoor / cameraAtTarget / m_134 的锚也错，
     但有 fn 直证，色不变。
  2. 相位 1 ≠ 全程（MethodReach 在下一相位起点删旧断点）：preset-help 的"场景触达"少计 7 方法；同一场景若同时
     有 T1 与后起窗口用例会漏验（现无此组合，不致误归绿）→ 并集取最小 t_hit。
  3. `--check` 逐字节比对：并行改 crates/ 只要动行号即失败 → 改归一化账面比对（严格逐字节另报）。
  4. 无"吸收"口径、字段初始化器行不归属 static{}、移植扫描含 game-oracle/src（差分宿主）——已补 / 已排除。
- 独立审计（同日，只读子代理复核 11 条论断）：全部成立；补强 1 处（m_153 自门控），纠正 1 处措辞（save.rs 对
  SKY_WAR 是"既不编码也不解码"而非"只解析"）。PORT_NOTES 的机械校验只保证锚存在，语义等价靠人工 + 审计抽核
  （7 条逐条对 a.java 方法体）：add_item_to_stack 漏 itemUseTargetType 写（a.java:7729），无可见效应（调用方
  先置 overlay_kind_item；商人路径浮层同调用内被替换），已写进条目。
- 剩余风险（未消除）：Rust 侧证据仍是静态的——"该 fn 在该用例里真被执行"未测。帧全等对有可见效应的方法是
  强约束，对 RMS/音频等帧不可见副作用无约束（deleteRecordStoreQuietly、m_117、m_139、closeAudioPlayer 等的绿
  只证明 Java 执行落在已验窗口内）。补法见下"遗留"①。

**三分账汇总**（逐项论证与复现依据：矩阵 §4.1）
- 基线 23 白：
  - **a 3**：m_106 + m_108（MOV 五参实体路径缓动 dp2：levelScriptLines 五参 MOV 36 处，20 处起点在 jar 初始布置
    中有实体，如脚本 1 = sprite2 型 72@(3,7)、脚本 12 的 13 处全落 sprite40）；saveGlobalProgress（SKY_WAR 读写双缺：
    save.rs 对 SKY_WAR 无编解码、boot 恒走读失败回退（paint.rs mode21 注）、壳只改写 MOT_L*/MOT_IF ⇒ 选项/全局旗标
    跨会话不持久——壳层赤字，非铺层赤字；读臂在 m_000 case 21 a.java:4450-4479，方法级已绿）。
  - **b 9**：m_093（javap：walkPhase 写点常量 ∈{0,1,2,4,5}，永不为 3）；m_107（DSL 第 16 码 MVS，68 条脚本零使用，
    脚本文本源封闭）；m_148-m_154 联网簇（gameMode 写点无 99；m_153 还自门控——mode99 臂先要求 f_bool_31 已真，
    而其唯一置真点就在 m_153 内；且 master-plan §4 范围裁剪）。
  - **c 11**：m_001 / readU16BE（seed 方言锚）与 `<clinit>`（VF 把 static{} 赋值投影成字段初始化器 a.java:44）——
    归属规则修正后机械转绿；m_059 / findItemStackIndex / addItemToItemStack / m_103 / m_118 / m_120 / m_123
    （移植存在，fn 异名 / 内联 / 锚陈旧）与 m_008（I/O 清理，吸收）——PORT_NOTES 逐条校验。解决后现绿 10 / 黄 1
    （addItemToItemStack 仅 preset-floor5 触达，该场景无 Rust 帧用例）。
- v2.1 新暴露 9：b 1（addSpark：paint case 18，gameMode 写点无 18）/ c 8（核验恢复，见上）。

**判据①（convergence-plan §0.1）达成度**
- 真未移植（a）只剩 **3 个方法、2 块工作**：dp2 实体路径缓动（m_106 + m_108，随首个触发五参 MOV 的层落地，
  候选 floor2 script1 / floor40 script12）；壳层 SKY_WAR 读写（saveGlobalProgress + m_000 读臂）。
- b 10 待终审：论证为 B 级（javap putfield 普查 + 脚本文本源封闭）；终审通过按 type-closure ☠️ 口径不计赤字。
- 黄 7：3 项补帧用例即转绿（m_076、addItemToItemStack ← preset-floor5；m_102 ← preset-floor49，A-trace 均已存在）；
  4 项无任何场景触达（hideNotify、advanceScriptWalk、m_089、m_101），需新场景或不触达论证。
- ⇒ 判据①剩余 = a 3 + b 10 终审 + 黄 7（3 补用例 / 4 论证或新场景）。
  > 【2026-10-15 收口】b 10 已 ☠️ 终裁、黄 7 → 绿 6 + 论证 1；m_106 + m_108 已由 6a37155 移植并转绿 ⇒ 判据①剩余 =
  > **a 1**（saveGlobalProgress，壳层 SKY_WAR；见本节末「判据①收口」）。

**更正与顺带发现**（B 级，待定级；不在本次写入范围者只登记）
- §3.1 mode 18 行已原地更正：mode 18 不可达（javap 普查），粒子系统 L3 归 mode 1/14。mode 13/99 的"全量 grep
  无入口"同法升级为 javap 普查——gameMode 写点常量 ∈{0-5,7-12,14-17,19-22}，计算来源 menuReturnMode∈{1,4}、
  loadTargetMode∈{3}、inGameMenuReturnMode←gameMode（无 6/13/18/99）。
- DSL 实有第 16 码 MVS（a.java:8351 `!var3.equals("MVS")` 的 else 臂：型 82 配对换格），68 条脚本零使用 ⇒ 死指令；
  type-closure §2 只列 15 码，待补登。【2026-10-15 已补登】type-closure §2（16 码，MVS ☠️ DC-2）。
- readU16BE 名实不符：方法体读 u16 **LE**（低字节先，a.java:6950；有符号 i2s），名为 BE，台账 N-0268 证据
  "read()<<8|read()" 与字节码相反——命名台账疑误，待命名批复核。【2026-10-15 已更正】276ac96 改名 readShortLE
  （N-0268 名实 + 证据更正）；矩阵 TRIAGE 随之改键。
- crates/game-core/src/save.rs 模块文档的 SKY_WAR 布局写错：实为 bool×4 + byte 计数 + 计数×int + int×3
  （a.java:4463-4474 读 / 8928-8939 写）。
- Rust 侧静默简化（无 todo! 标记）：① scene.rs 层存档恢复的 floor2 特例忽略 scriptLineFlags[26]（注"按 false
  处理"）；② save_floor_state 恒写 Some，Java 在 entityCount==0 时留 null（m_119 a.java:9380-9381），决定该层重入
  走存档还是 sprite 权威数据；③ add_item_to_stack 多一道容量守卫（Java 无守卫；可叠类型 ≤15 < 容量 16，实际不可触发）。

**遗留**
1. Rust 侧动态覆盖：按用例采 Rust fn 执行集（如 cargo llvm-cov），与 Java 触达集逐方法求交——把"静态移植证据"
   升级为"同用例双侧执行"，消除上述剩余风险。
2. b 10 终审（☠️ 化）+ type-closure §2 补 MVS 码。【2026-10-15 销账】见下「判据①收口」①③。
3. 黄 7：preset-floor5 / preset-floor49 补 Rust 帧用例；其余 4 项论证或新场景。【2026-10-15 销账】见下②。
4. `tools/method_reach.py --rev REV --only 场景` 已可用（历史版场景同预算重测 + 与现行快照逐方法对比，不入快照；
   `--rev HEAD` 自测与快照逐方法一致）；尚未接入 cover_matrix（历史账面整表复算）。
5. 验证源盲区：矩阵只认 `crates/game-oracle/tests` 里读 `_out/A-*` 的帧用例。同日新增的
   `crates/game-core/tests/slot_equip_scene.rs`（夹具内嵌 Java 帧、场景脚本在 tests/fixtures/ 而非 scenarios/）对
   method_reach / collect_tests 不可见——其覆盖的 m_113/markSlotEntry 已由他例转绿，色不受影响，但口径缺口应补
   （并入 scenarios/，或让两工具识别夹具场景）。【2026-10-15 销账】两工具已识别夹具场景/夹具帧用例（下④）。

**判据①收口（2026-10-15 二次刷账，基线 47169cc）**——账面 151 / 1 / 11，白 = a 1 + b ☠️ 10，黄 1 带论证 ⇒ 判据①赤字 1

① **b 10 死代码终裁 ☠️**：裁决书 [dead-code-rulings.md](dead-code-rulings.md)（DC-1 m_093 / DC-2 m_107 / DC-3 联网簇
   m_148-m_154 / DC-4 addSpark，逐项主张/证据/可复现命令/结论）。证据从「javap putfield 普查」升级为机械证明：
   `tools/dead_code_ruling.py`（两 class 的 CFG + 字段值域不动点 + MVS 字符串数据域引理，3 轮收敛）在 168 方法中证死
   **恰为这 10 个**；JDI 49 场景零触达互证。cover_matrix 渲染时三重校验（字节码证死 + 裁决书节锚四段 + 零触达），
   不成立即 ⚠ 退回待裁。type-closure §3.1 销账。
② **黄 7 处置**：6 项转绿——m_076 / addItemToItemStack ← preset-floor5、m_102 ← preset-floor49（迭代 59/66 原用例在
   35e779f / 308765f 改作他用后缺位，夹具化恢复：帧期望自 A-trace 抽取入 `crates/game-oracle/tests/fixtures/*.frames.txt`，
   帧链 = 快照）；advanceScriptWalk ← fx-scriptwalk-floor3（floor3 script0 玩家三参 MOV 两步）、m_101 ←
   fx-knockback-floor43（型60 上邻击退：可退 / 墙挡）、m_089 ← fx-fan-floor13（芭蕉扇未命中 / 命中两臂）——三个新夹具
   场景，Java A/B/C 三方 trace 逐字节一致后抽帧（`tools/frames_fixture.py gen`，隔离工作区，不碰 `_out/`），Rust
   `crates/game-oracle/tests/fixture_frames.rs` 全帧绿。1 项论证——hideNotify：字节码唯一调用者 CMidlet.pauseApp@4 +
   平台离屏回调；oracle 驱动面只有按键事件（Runner/shim 无 pauseApp/hideNotify 调用，矩阵渲染时机械校验）⇒ 场景不可达
   但**非死代码**；语义由 game-desktop 宿主测试覆盖（ab8368a；gameMode 3→4 臂 47169cc 接通后
   `hide_in_mode3_opens_in_game_menu` 摘 #[ignore]）。
③ **MVS 第 16 码入账**：type-closure §2 改 16 码（MVS ☠️ DC-2）；§3.2 原地更正。
④ **工具口径**：cover_matrix 扫 game-oracle + game-core 两测试目录、识别夹具帧用例（`assert_eq!(sha, rec.sha…)` +
   `<stem>.frames.txt`，夹具帧链须 = 实测帧链）、只认 git 已追踪文件（并行在制测试不进账面）；method_reach 测量夹具场景
   （`crates/*/tests/fixtures/<stem>.txt` + 同名 `.frames.txt`，帧对照以夹具为准）；PORT_CRATES 删去不存在的 game-wasm；
   新增 YELLOW_NOTES（黄项论证机械校验）与 ARM_GAPS（无 todo! 臂缺口登记，矩阵 §5.1）。夹具帧用例另认「规格常量式」
   （mov_route_scene.rs：`const FLOOR2: Spec = Spec { frames: include_str!(…), ticks: N }` + 辅助函数内帧断言与
   `checked == spec.ticks` ⇒ 窗口 [1, N]）。抗改名：人工条目键与裁决对象经台账旧名→现名映射；v1 伪名钉 2eed129、
   v2 口径行号钉 a97d70e（dc470b2 批量改名 + deobf +3 行漂移后 v1 44/1/58 不变）。
⑤ **新发现（A 级帧证，未修——game-core/src 不在本任务范围）**：interactWithCell 型 10/11/12 撞击臂未移植——
   scene.rs cat1 `_ => {} // 11 / 76/82：后续批次` 吞掉（无 todo!，此前无场景撞击故全绿）；型10 帧级复现
   `fixture_frames::fx_firebump_floor13_frames_match`（#[ignore] BLOCKED，首分歧 T301 = Java 提示浮层），11/12 同臂
   （B 级）。补臂后摘 #[ignore] 即验收。
⑥ **更正**：type-closure §5「零出现型 ☠️ 候选」中 16/17/18/22/24 有运行期来源（boss 击杀事件 / 商人报价 / CES 脚本），
   非死内容，原地更正；余 25/82/87。
⑦ 剩余赤字 a 1：saveGlobalProgress（壳层 SKY_WAR 读写，3d42310 起 save.rs 布局备忘引其行锚——名证锚机械命中但
   注释明言「既不编码也不解码」，PORT_NOTES not-ported 压制）。m_106 + m_108 已由 6a37155 移植，经 mov_route_scene
   夹具帧用例（规格常量式，本次口径扩展识别）转绿；dp2 挂起项落地。
⑧ 名称口径：本节方法名为裁决/取证时名；dc470b2（语义命名第二批）后 m_093 = drawLeftTriangleArrow、m_076 =
   browseOfferQueue、m_102 = probeCrossFloorSnapshot（矩阵按现名渲染，cover_matrix / dead_code_ruling 经台账自动映射
   旧名，v1 复算伪名同样回溯，44/1/58 不变）。顺带发现（B 级，data/naming 不在本任务写入范围）：同批 N-0434 把 m_001
   命名 **readCountPrefix** 疑名实不符——方法体是 packed 图像容器装载（resourceNames[i] 流内逐张 u16 LE 长度前缀 +
   Image.createImage，a.java:4516-4549），非「读计数前缀」，待命名批复核。

复现：
```
python3 tools/method_reach.py --check      # 快照与场景/预置/改名类一致
python3 tools/cover_matrix.py --check      # 账面一致（Rust 行号/锚计数漂移归一化）
python3 tools/dead_code_ruling.py --check  # b 类 10 方法字节码证死（裁决书 dead-code-rulings.md）
python3 tools/frames_fixture.py check crates/game-oracle/tests/fixtures/fx-fan-floor13.frames.txt \
    --script crates/game-oracle/tests/fixtures/fx-fan-floor13.txt          # 夹具自洽（其余夹具同式）
J="javap -p -c analysis/rename-pipeline/renamed/a.class"
$J | grep -B1 'putfield.*Field walkPhase:' | grep -v 'putfield\|^--' | sed -E 's/^ *[0-9]+: *//' | sort | uniq -c   # 无 iconst_3
$J | grep -B1 'putfield.*Field gameMode:'  | grep -v 'putfield\|^--' | sed -E 's/^ *[0-9]+: *//' | sort | uniq -c   # 无 18/99
$J | grep -B1 'putfield.*Field f_bool_31:' | grep -v 'putfield\|^--' | sed -E 's/^ *[0-9]+: *//' | sort | uniq -c   # 唯一 iconst_1（m_153）
```

## 4. 防钝化规则（对"跑即绿"的三条约束）

1. **新用例准入声明**：每个新对拍用例入册时，必须在 commit 中声明它
   **新增的分母覆盖**（此前无该等级验证的 mode/指令/层/写路径）。
   不扩大覆盖的用例 = 回归网，允许存在但不计进度。
2. **覆盖 ratchet**：本文件矩阵中 L3+ 格子数只增不降（同 test-ratchet 精神）；
   发现假绿（如嵌套测试、缺 preset 头）修正后必须**重建对拍基准**而非
   保留绿状态。
3. **每报告给"下一最优目标"**：按分母大小 × 玩法权重选出未覆盖最大块。
   当前（2026-10-12 二次校准）：mode 11 对话层 Rust L3 化（现为 L2）>
   mode 9 炼丹链（Rust 未实现）> mode 19 商店场景（Rust 已实现零覆盖）>
   按层铺 51 深层（夹具已备）。矩阵初版的"mode 6 战斗 ❌"系误读
   （战斗非独立模式），方法论自身首日即捕获一处分母错误——分母提取
   同样要过证据关。

## 5. 更新 ritual

每迭代"语义回流"步骤固定动作：
1. `python3 tools/completeness.py`（分母可能因参考系重提取而变——理论上不变）；
2. 更新 §3 矩阵（新覆盖 + 证据指针）；
3. `python3 tools/completeness.py --check`（证据存在性）；
4. 本文件与代码同 commit 入库（完整性=文档+矩阵+证据一体）。
