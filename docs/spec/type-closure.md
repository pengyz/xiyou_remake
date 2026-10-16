# 类型闭包账本（2026-10-14 起 = Stage A 收敛主判据）

> **决议（迭代 78，用户拍板）**：完成判据从「55/55 层各一行 L3」改为**类型闭包**——
> 每个可交互类型都交互过、每个怪物型都战斗过（或证明不可战/死代码）、每个特殊脚本
> 逻辑点都覆盖过。层不再是进度单位，改为**取样载体**：spriteN 普查有类型/分支增量
> 的层才铺；无增量的层记「普查派生覆盖」一行（带普查证据），不跑对拍。
> 层遍历测试（55 层逐一 L3）降级为**可选补充**，类型闭包完成后另行决策。
>
> 依据：B1 四连层（floor41/44/45/48，迭代 78）全部首跑即绿零新移植——层 variation
> 是数据（实体表/脚本内容/param），代码路径是类型级共享的。玉皇大帝封印（m_102
> 守卫门控 + 击杀钩子）、封印门配对开门（tryOpenChest→wp4）等"层专属机制"实为
> 类型级臂 + 层数据；真正的层条件分支仅 17 处 currentFloor 比较 + 少数层相关
> renderCategory（§4，可枚举）。

## 0. 账本与判据

四本账全绿 + 原四条硬判据（矩阵无白项/状态差分零行为分歧/帧全绿挂起清零/gates
8/8）= Stage A 收敛。覆盖状态标记：

- ✅ 已交互/已战斗（有 trace 锚点或用例名）
- 🎨 仅绘制/仅出现（画面对拍过但未走交互臂）
- 📦 数据派生（同臂他型已验，本型差异为表数据——静态对表佐证）
- ⬜ 未覆盖（= 迭代队列）
- ☠️ 死代码候选（待终裁，格式同 mode13/99）

## 1. 实体类型账本（88 型）

静态语义源：objectTypeNames（a.java:954）/ renderCategoryByType（a.java:6240-6295）/
animRowByType（:498）/ typeFrameWidth（:589）/ entityTypeImage 映射（m_122 :9454-9482）。
交互臂：bump switch（a.java:5378-5530，case 族 1-12/16/32/76/81/83 + cat2/4 拾取 +
cat8 战斗 + cat16 商人）与 applyStepCellEffects（:5637-5715，case 1-8/76/83）。

### 1.1 cat1 地形/机关（bump/step 双臂）

| 型 | 名 | 状态 | 证据/注 |
|---|---|---|---|
| 1 | 黄门 | ✅ | floor45 (11,5)/floor48 (4,6) 开门；consumeKeyForDoor 26 |
| 2 | 红门 | ✅ | 开门臂（code 27）未见交互——delta【2026-10-16 翻 ✅】FX-DOOR-2A/2B 双臂帧夹具（f3506cc，fixture_frames 16/16）：fx_door2a_open_frames_match（floor40 (6,9) 北撞 ⇒ consumeKeyForDoor(27) a.java:7746-7752 popup→redKeyCount 1→0→markEntityRemoved state=1（:6355-6361）→放行进门格，T1-520 全帧）+ fx_door2b_nokey_frames_match（无钥匙负例：双撞恒挡「你没有红钥匙」a.java:5392-5395，门 state 恒 0）——原文"未见交互"销 |
| 3 | 蓝门 | ✅ | floor_crossing 蓝门重复弹关周期（迭代 36） |
| 4 | 封印门 | ✅ | floor42/49/44/48 wp4 组开门 |
| 5 | 门卫(宝箱) | ✅ | floor42/49 守卫-宝箱配对；floor44/45 双配对 |
| 6 | 隐形路径 | ✅ | bump case6 显形臂（visible+state2+row9）未见——delta【2026-10-16 翻 ✅】FX-INT-6A 三方（903dc95，fixture_frames 16/16）：fx_int6a_reveal_frames_match（floor5 (10,11) 左撞 (9,11) ⇒ 显形三元组 visible→state=2→animRow=9 写序 a.java:5413-5415 → 地形豁免放行 → 过渡归零 a.java:6838-6843 → 二次进入幂等，T1-560 全帧+终态断言）——原文"显形臂未见"销 |
| 7 | 上楼梯 | ✅ | floor_crossing 楼梯往返 |
| 8 | 下楼梯 | ✅ | 同上 |
| 9 | 炼丹炉 | ✅ | mode9 全流程（alchemy） |
| 10 | 云雾(三昧真火) | ✅ |【2026-10-15 晚销账】移除臂 ✅ fx-fan-floor13（芭蕉扇 m_089 命中）+ 撞击臂已移植（ee52191，a.java:5436-5440）+ 帧级 fixture_frames::fx_firebump_floor13_frames_match 解禁全预算绿（RED 钉 T301）。【史】 bump case10 提示浮层；与 16 芭蕉扇联动——delta。【2026-10-15】移除臂（芭蕉扇 m_089 命中）✅ 夹具场景 fx-fan-floor13；**撞击臂 Rust 未移植**（a.java:5436-5440；scene.rs cat1 `_ => {} // 11 / 76/82：后续批次` 吞掉，无 todo!）——帧级复现 fixture_frames::fx_firebump_floor13_frames_match（#[ignore] BLOCKED，首分歧 T301），矩阵 §5.1 |
| 11 | 能挖的墙 | ✅ | bump case11 + **currentFloor!=23 层条件**（§4）——delta。【2026-10-15】撞击臂同 10 被 `_ => {}` 吞（代码阅读 B 级，矩阵 §5.1）【2026-10-16 同步】撞击臂已随主行（型10）移植（ee52191，a.java:5436-5456 逐值：型11 = **23 层静默、其余提示 + 阻挡 + 停扫**）——原文"被 `_ => {}` 吞"系移植前状态，已过时（保留历史）。【2026-10-16 翻 ✅（已交互口径）】交互臂三面帧锚：fx_item17a（凿开 (8,3) 穿行，2d802fd）/fx_item18a（36 堵全清）/floor2-mov（m_106 途经 (2,8) 开墙）；bump 提示臂已移植（ee52191）但**无独立帧夹具**（fx_item17a 输入走面板 300-380 未撞墙）——残余两缺口挂 §4 :5442 行与 FX-INT-11-F23 扩展候选（23 层静默负例 + 非 23 层提示臂），不随本格销 |
| 12 | 隐形墙 | ✅ | 图=型11 别名；bump case12 语义待查——delta。【2026-10-15】case12 = 显形为型11（state2/row9/visible）+ 阻挡（a.java:5449-5456）；Rust 同被 `_ => {}` 吞（B 级，矩阵 §5.1）【2026-10-16 同步】撞击臂已随主行（型10）移植（ee52191，a.java:5449-5456 逐值：变形为型11 + dispatch_sprite_size/anim 重派 + state2/animRow9/visible + 阻挡**续扫**〔Java 无停扫〕）——原文"Rust 同被 `_ => {}` 吞"系移植前状态，已过时（保留历史）。【2026-10-16 驳回翻 ✅ 依据】"floor40-mov 逐帧"不成立——sprite40 字节级解析 29 记录 **0×型11、0×型12**（consumed==filesize，A 级），floor40 无此二型可触；case12 变形臂全夹具语料**零触达**（floor23 sprite 仅 probe 快照装载，mov_route_scene.rs:245-246 自注"测试须补 floor23"）⇒ 保守留 ⬜（防虚报）。夹具建议：FX-INT-11-F23 一并钉 floor23 型12 bump 变形 + 型11 静默双臂【2026-10-16 翻 ✅】建议即落地（ec6fb55，A/B/C 三方逐字节一致，fixture_frames 18/18）：fx_int11_f23_transform_silent_frames_match（floor23 (2,4) 型12 右撞 ⇒ case12 变形写序全钉 type12→11 + dispatchSpriteSize/AnimFields(11) + state2 + animRow9 + visible（a.java:5449-5456）⇒ 过渡归零（:6838-6843）；T1-560 全帧 + FLD type sha T302 跳变（12→11）/T402 归零实证） |
| 76 | 路点 | ✅ | floor35/42/46 wp4 组门踏板 |
| 81 | 封印门(隐形) | ✅ | floor10 隐形81 |
| 82 | 传怪点 | ✅ | case82 spawn+hidden；出层层待普查——delta【2026-10-16 终裁 ✅ 死码臂忠实实现】裁决口径=**有臂可验且已验 ⇒ ✅（用例名覆盖）+ 死码实证入注**（与 87「无臂可验 ⇒ ☠️」区分）：① 载入臂 spawn+hidden ✅（scene.rs:951-953 m_122 特例隐藏表含 82）；② MVS 传怪臂 ✅（scene.rs:2103-2191 + m_107 :2195，a.java:8519-8592 逐值；单测 mvs_pairs_by_param_low_byte_transfers_and_consumes〔scene.rs:6784〕+ bd735f6 三钉 quirk）；③ **实机不可达（双重死码 A 级）**：语料 297 条指令 0 条 MVS + 全 55 层 sprite 型82=0（evidence-todos T-1，型83 有 52 个对照）——夹具永远打不到，单测是唯一验证面。出层普查同步销：82 无任何出生源（CES/ADD/GIN/bossEventSpawns/商人五类均无） |
| 83 | 剧情点 | ✅ | floor10/41/42/46/48（case15/14/13/38 门控族） |

### 1.2 cat2 道具（拾取共享臂 + 每型使用效果）

| 型 | 名 | 状态 | 证据/注 |
|---|---|---|---|
| 13 | 火眼金睛 | ✅ | item13 用例 |
| 14 | 生死簿 | ✅ | preset_floor5_fixture_frames_match（道具14 全链 T1-1200：拾取→面板→使用→mode12 翻页/关闭） |
| 15 | 筋斗云 | ⬜ | delta【2026-10-16 复核留 ⬜（预翻 📦 被驳回）】使用臂空操作=原版语义（a.java:7795-7800）✓；被动授予侧已核（拾取置 tutorial_seen=f_bool_12 scene.rs:5272/409，f_bool_13=stair_hint 楼梯指示门控 scene.rs:411）✓；**但消费侧 49/55 快捷键臂存在 A 级分歧**：scene.rs:4476-4477 先 `key_value=0` 后 `let up = key_value == 49` ⇒ `up` 恒 false，49 上行臂被并入 55 下行臂——a.java:5295/5301 为**分立 case**（49→findFloorGateEntity(true)+changeFloor(…,false,…)、55→false+true）。“被动效果移植已核”对本型不成立 ⇒ 保守不翻 📦（防虚报）；待分歧裁定（BUG 候选，建议 bug-ledger 登记 + 49/55 双向夹具）后翻 |
| 16 | 芭蕉扇 | ✅ | 【2026-10-15】使用臂 m_089(10) 未命中提示 / 命中灭火两臂 ✅ 夹具场景 fx-fan-floor13（T1-830 全帧，preset 道具栏预置）；正常游玩来源 = 杨戬(69) 击杀 spawnBossEvent(4)（§5 更正） |
| 17 | 金勺子 | ✅ | 凿墙联动型11——delta。来源 = 啸天犬(67) 击杀 spawnBossEvent(3)（§5 更正）【2026-10-16 翻 ✅】FX-ITEM-17A/17B 双臂（2d802fd，fixture_frames 16/16）：fx_item17a_chisel_wall_frames_match（floor29 (8,3) 型11 ⇒ m_089(11) 四邻命中 markEntityRemoved a.java:7895-7919 + uses 1→0 出栈 + T520 端到端通行，T1-620 全帧）+ fx_item17b_fail_no_consume_frames_match（失败不消耗负例：case17 返 false a.java:7809-7816 ⇒ useItemStack 短路 a.java:7766，栈 uses 恒 1） |
| 18 | 玄明石 | ✅ | 同上——delta。来源 = 天宫商人(78) 报价 13 → pickupItemType(18)（a.java:7432，§5 更正）【2026-10-16 翻 ✅】FX-ITEM-18A（2d802fd）：fx_item18a_clear_all_walls_frames_match（floor37 型11×36 同拍全 markEntityRemoved a.java:7817-7828 全表扫描 + uses 1→0 出栈，T1-560 全帧 + 36 堵全 state=1 断言）——原文"同上"系误并 17 语义，实际效果=**震开当前层全部墙**（evidence-items §2.3） |
| 19 | 千年月桂露 | ✅ | delta【2026-10-16 翻 ✅】FX-ITEM-19A 精确值（2d802fd）：fx_item19a_heal_exact_frames_match（atk105+def203 ⇒ (105+203)*74/10=2279 先乘后除整数截断钉死 a.java:7830 → hp 500+2779→2779 + uses 1→0 出栈 a.java:7766-7775，T1-560 全帧） |
| 20 | 相形变位 | ✅ | floor42 script14 ROS_4_1（脚本侧使用） |
| 21 | 上行符 | ✅ | preset_item21_frames_match / preset_item2122_frames_match（使用臂帧测） |
| 22 | 下行符 | ✅ | preset_item2122_frames_match（21+22 配对使用臂帧测）；来源 script25 CES_22_6_6（§5 更正）；来源 script25 `CES_22_6_6`（§5 更正） |
| 23 | 太公杖 | 📦 | delta【2026-10-16 预翻 📦 被动派生】使用臂空操作系原版语义（a.java:7795-7800 fall-through `default: break`，返回 false 不消耗，uses=-1 永久）；被动效果移植已核 = effective_attack_vs_type（combat.rs:66-72，a.java:9992-10002：特性位 &1 且背包有 23 ⇒ 攻击 ×2），单测 combat_golden（"无 23 号不翻倍"/"23 号 + &1 ⇒ ×2"） |
| 24 | 捆仙绳 | 📦 | delta。来源 = 玉皇大帝二阶段(71) 击杀 spawnBossEvent(5)（§5 更正）【2026-10-16 预翻 📦 被动派生】使用臂空操作系原版语义（a.java:7795-7800 fall-through，uses=-1 永久）；被动效果移植已核 = effective_attack_vs_type（combat.rs:66-72：特性位 &2 且背包有 24 ⇒ 攻击 ×2），单测 combat_golden（"&2 需要 24 号"⇒ ×2） |
| 25 | 幸运币 | 📦 | delta【2026-10-16 预翻 📦 被动派生】使用臂空操作系原版语义（a.java:7795-7800 fall-through，uses=-1 永久）；被动效果移植已核 = 金翻倍**两掉落点**（scene.rs:3036-3039 战杀 + 4855-4856 killAdjacentAt，`find_item(25)>=0 ⇒ enemyBaseGold<<1`，a.java:10046-10052/7944-7948）；帧夹具 FX-ITEM-25A 列扩展候选（fixture-plan §3） |
| 26 | 黄钥匙 | ✅ | 拾取常见 |
| 27 | 红钥匙 | ✅ | 拾取常见 |
| 28 | 蓝钥匙 | ✅ | floor48 T480 帧证拾取 |
| 29 | 威力丹 | ✅ | floor48 拾取 |
| 30 | 金刚丹 | ✅ | floor41 出生格拾取（HP+250 帧证） |
| 31 | 回春丸 | ✅ | floor41/44 拾取 |
| 32 | 长寿丹 | 🎨 | floor45/48 出现未拾 |
| 85 | 黄金钥匙 | ✅ | floor46 85 臂（全场黄门） |
| 86 | 日月无极钹 | ✅ | preset_itemcymbal_frames_match（钹使用臂帧测） |

### 1.3 cat4 装备（m_083/tryEquip 共享链）

| 型 | 名 | 状态 | 证据/注 |
|---|---|---|---|
| 33-39 | 棒/甲 tier1-7 | 📦 | 80/40 已验全链（拾取→m_083 档位→tryEquip→外观/数值/描述盒）；余为 equip_types/描述表数据——静态对表佐证 |
| 40 | 紫金龙鳞甲 | ✅ | floor44（atk 30→250 + D 表盒） |
| 79 | 乌金棍 | ✅ | floor49 |
| 80 | 银缕锁甲 | ✅ | floor43（def 150 + 先减后比实证） |

### 1.4 cat8 敌族（战斗共享臂：预测/抖动/伤害弹窗/击杀钩子）

战斗臂已由 20+ 型全验（41-66 全谱 + boss 若干）（floor35/41/42/44/45/46/49/50）。敌型差异 = 敌表数据
（enemyBaseHp/Atk/Def/Gold、battleDigitByType、typeFrameWidth、animRow）。

| 型 | 名 | 状态 | 证据/注 |
|---|---|---|---|
| 60 | 兜率宫星君 | ✅ | floor41/44/46 战；【2026-10-15】步邻效果臂（applyStepCellEffects 上邻击退 m_101 + 扣血 200，a.java:5600-5631）✅ 夹具场景 fx-knockback-floor43（可退 / 墙挡两臂，T1-600） |
| 61 | 无量护法 | ✅ | floor42/45/49 战 |
| 62 | 兜率宫老仙 | ✅ | floor41/44/48 战 |
| 63 | 二郎执旗将 | ✅ | floor47 (11,6) 战（迭代 79；出生口袋唯一出口=楼梯的又一例） |
| 64 | 杀破狼 | ✅ | floor41/44 战 |
| 65 | 二郎骁骑将 | ✅ | floor42/43/45 战 |
| 66 | 尊天韦驮 | ✅ | floor44 首战（迭代 78） |
| 41 | 天宫小犬 | ✅ | floor34 (5,4) 战（迭代 79） |
| 42 | 天宫大犬 | ✅ | floor34 (7,8) 战（迭代 79） |
| 43 | 伴炉童子 | ✅ | floor34 (11,8) 战（迭代 79） |
| 44 | 护庭小神 | ✅ | floor10 NW (1,3) bump 战（迭代 79——旧 floor10 用例的 44 系脚本臂 DES/CES，本块补 bump 战） |
| 45 | 守园仙婢 | ✅ | floor10b (4,11) 战（迭代 79） |
| 46 | 护庭校尉 | ✅ | floor10 NW (2,4) bump 战（迭代 79） |
| 49 | 巨力士 | ✅ | floor14 (8,8) 战（迭代 79） |
| 50 | 执火道人 | ✅ | floor14 (8,5)/(4,5) 战×2（迭代 79） |
| 51 | 奎木狼 | ✅ | floor14 (2,8)/(4,8) 战×2（迭代 79） |
| 52 | 执瓶仙侍 | ✅ | floor14 (9,9) 战（迭代 79） |
| 53 | 金刚力士 | 🎨 | floor14 (5,2) 绘制；战：蓝门区三兄弟留待带钥重入 |
| 54 | 护持迦蓝 | ✅ | floor14 (6,5) 战（迭代 79） |
| 47 | 巡天卫士 | ✅ | floor38 (3,10)/(1,10) 战×2（迭代 79；bump 时北守卫未死⇒宝箱封印拒绝语义同证） |
| 49 | 巨力士 | ✅ | floor14 (8,8)；floor38 (7,11)（迭代 79） |
| 56 | 护庭神将 | ✅ | floor38 (5,11) 战（迭代 79） |
| 58 | 护丹老道 | ✅ | floor38 (11,9) 战（迭代 79；78 天宫商人同层 (5,2) 北半另窗） |
| 59 | 伏魔韦驮 | ✅ | floor38 (10,10)/(10,7) 战×2（迭代 79） |
| 67 | 啸天犬 | ✅ | m_001(9) 多段拼装绘制臂已移植；战 ⬜——delta【2026-10-16 战 ⬜→✅】FX-BOSS-67A（4f2fc30）：fx_boss67a_kill_spawn_frames_match（floor15 bump 战一击毙 ⇒ bossTypeOrder[3]=67 ⇒ spawnBossEvent(3) ⇒ bossEventSpawns[3] 刷金勺子(17)@(6,4) DES 态 a.java:2143/6333-6345/6838-6843 + 金 100→200，T1-560 全帧 + 掉落格/状态/anim 断言）；帧链 **A==B 双变体一致**——**C 腿断系 deobf 投影缺陷**（a.java:6592 型67 拼装臂 byte[]→Object[] 强转 CCE，bug-draft-paint67 投影稿），**非移植问题**，不阻本格 |
| 68 | 太上老君 | ⬜ | boss 战+钩子——delta |
| 69 | 杨戬 | ✅ | preset-floor35「case69 击杀钩子首验」= 杨戬战实证（击杀钩子随战触发）+ floor35 T1-1200 全帧 |
| 70 | 玉皇大帝 | ✅ | floor49 战 + floor42 DES 剧情消失（封印语义：m_102 守卫门控） |
| 71 | 玉皇大帝(二阶段) | ✅ | preset-floor49「case70 击杀钩子 + 守卫宝箱 param 配对开门」= 玉帝链实证 + m_102 守卫门控（preset_floor49_fixture_frames_match T1-3400） |
| 72 | 牛魔王 | ✅ | ending 开战（cat8/cat32 层相关 renderCategory ✅ 迭代 56） |
| 73 | 天蓬元帅 | ✅ | floor42 DES 剧情移除 |
| 74 | 巨灵神 | ✅ | boss 战+钩子——delta【2026-10-16 翻 ✅】FX-BOSS-74A 三方（4f2fc30）：fx_boss74a_kill_hooks_frames_match（floor20 击杀 ⇒ 双钩子：① spawnBossEvent(1) 12 件掉落 a.java:2092-2141 逐格断言〔29×3/26×3/30×3/32×3〕② loadLevelScript(21) 遗言 TAK_59_59 无 tryRunScene 门控 ⇒ script_line_flags[21] 收尾置位 a.java:8600，T1-600 全帧） |
| 75 | 哪吒 | ✅ | floor10 boss |

### 1.5 cat16 商人 / cat32 特殊

| 型 | 名 | 状态 | 证据/注 |
|---|---|---|---|
| 77 | 太白金星 | ✅ | floor46 商人全链（报价/受领/消耗白烟，迭代 77 七臂语义） |
| 78 | 天宫商人 | ✅ | floor45（f_byte_arr_13 表 + 37px 宽变体 + atk 报价） |
| 84 | 嫦娥 | ⬜ | 图 sptenemy2[18]；出层层待普查——delta |
| 87 | 菩提老祖 | ☠️ | 图 sptenemy2[19]；出层层待普查——delta【2026-10-16 终裁 ☠️ 实体面死】（evidence-boss §4.2 存证，本次独立复跑一致）：全语料 `spawnEntity(87|CES_87|ADD_*_87)` **0 命中**，`\b87\b` 全扫仅对话发言者表（a.java:1358 起）/logo/gameMode default 簇/renderCategory[87]=32（:6294）/entityTypeImage（:9478）/lockCameraOn 显式跳过（:8672）；enemyBaseHp n=35（:1044-1080 表长 A 级）⇒ 数值上不可参战；击杀/收尾 switch 无 case87（:10070-10086/:10115-10154）。**对话面非死**：87 = 序章对话发言者（dialogueSpeakerType）+ `lockCameraOn(87)` 跳过锚定——文本面数据派生，序章对话用例（prologue_frames_match T1-9800）已覆盖。出层链 = 序章 script40-53 教学 + script67 混沌引子（case67 已移植 scene.rs:1528）——"出层层待普查"由 evidence-boss §4 普查销 |

## 2. 脚本 DSL 臂账本（16 码）

> 【2026-10-15 更正】原题「15 码」漏列第 16 码 **MVS**（executeScriptInstruction a.java:8354
> `!var3.equals("MVS")` 的 else 臂，型 82 传怪点配对换格 → m_107）——覆盖矩阵审计发现，补登于表末（☠️）。

| 码 | 状态 | 证据/注 |
|---|---|---|
| TAK | ✅ | 多页对话/翻页（floor35/42/46 九页） |
| CES | ✅ | floor35/42 spawn+DesRemoved |
| DES | ✅ | floor42 双 boss 剧情移除 |
| ROS | ✅ | floor42 ROS_4_1 |
| GUT | ✅ | ending script37（停扫语义） |
| GLV | ✅ | 同上 |
| SWD | ✅ | floor10 变换 + 型11 全场清除臂 |
| END | ✅ | 序章 script40 入口 |
| MOV | ✅ | dp3 脚本行走 ✅（【2026-10-15】玩家三参 MOV 的 advanceScriptWalk 臂 ✅ 夹具场景 fx-scriptwalk-floor3：floor3 script0 `MOV_0_5_11`）；dp2=m_106 五参实体路径缓动 ⬜（挂起账本）【2026-10-15 更正：△→✅】6a37155 移植 m_106 + m_108，mov_route_scene floor2（script1/2 型72）/ floor40（script12 伏击 13 战）全帧绿 |
| GIN | ✅ | 【2026-10-15 晚 普查】语料 scripts26-29（嫦娥族：DES_84_×4 + GIN_x_y）；触发层=型84 所在层（出层普查同批）；**臂未移植**（executeScriptInstruction 无 GIN 分派，落 todo!()）【2026-10-16 更正】臂已移植（7934035，a.java:8355-8490 逐值对照，4 单测 script_gin_gives_gold_and_recurses〔含尾递归断言〕全绿）——原文"臂未移植"系移植前普查 |
| LAY | ✅ | 【2026-10-15 晚 普查】语料 script0（floor3）`LAY_2`；触发层=floor3 脚本链后段（fx-scriptwalk T1-700 窗口止于 LAY 之前=未触达实证）；**臂未移植**（无分派，落 todo!()）【2026-10-16 更正】臂已移植（7934035，a.java:8355-8490 逐值对照，4 单测 script_lay_jumps_floor_with_wipe 全绿）——原文"臂未移植"系移植前普查 |
| RES | ✅ | 【2026-10-15 晚 普查】语料 script0（floor3）`RES_0`（LAY 同窗后段）；**臂未移植**（无分派，落 todo!()）【2026-10-16 更正】臂已移植（7934035，a.java:8355-8490 逐值对照，4 单测 script_res_resets_run_stats_on_easy〔easy/非 easy 双分支〕全绿）——原文"臂未移植"系移植前普查 |
| SEE | ✅ | 【2026-10-15 晚 普查】语料 scripts40-45 等 14 条（序章镜头族）；触发层=prologue/剧情层；臂已移植（scene.rs `op == "SEE"`，m_103 ease）；序章 T1-9800 含电影镜头推进=疑似覆盖，待帧锚/变异复核后转 ✅【2026-10-16 更正：⬜→✅ 帧锚复核完成】A-prologue-dialog 状态倾印逐 tick 锚定：T0584 phase=1/cursor=12（TAK_255_259）→T0782 cursor=24（TAK_165_165）→**T0902 phase=6/cursor=43 = SEE_3_10_166_166_1 执行**→T0962 phase=4（行尾收尾门 a.java:8568 覆写，BUG-006 冻结态）；prologue_frames_match T1-9800 逐 tick 9800 FRAME 全帧比对绿（2026-10-16 实跑 460s）=SEE 及其后 8898 拍全被帧哈希锚定；双分支单测 see_bug006_replay_cursor_advances_walk_not_armed / see_path_found_arms_walk（script_golden，a.java:8466-8481）。变异复核未做（scene.rs 不在本任务写域）——判据=帧锚+双分支单测；若日后变异复核证伪按更正文化翻回 |
| SMS | ☠️ | 【2026-10-15 晚 普查】语料 script66（`SMS_0`，短信/付费时代残迹）；触发层=script66 门链待查；臂=两行桩（scene.rs `op == "SMS"`：读参跳行=外部副作用吸收型〔Java 侧平台短信调用，桌面不可复现〕，待 ☠️/吸收终裁）【2026-10-16 终裁 ☠️】吸收型空转桩：两端皆无语义可移植——Java 臂本身即读参跳行（a.java:8515-8518：`parseScriptInt` + `scriptCursor=lastDelimPos+1`，**无任何平台/短信调用**），Rust 桩（scene.rs:2053-2056）与之逐字等价；语料仅 script66 `SMS_0` 一处（a.java:1191）。> 【2026-10-16 更正】原文"Java 侧平台短信调用，桌面不可复现"系 mnemonic 推测（C 级），反编译投影中不存在该调用——"外部副作用吸收"实际成立的只是**语义空转**这一层（付费短信时代残迹被上游剥除或本就是桩）。触发链（script66 门链）查否不影响裁决（臂无副作用）；若字节码级复核（javap）发现投影吞掉的调用，按更正文化翻回 |
| ADD | ✅ | 【2026-10-15 晚 普查】语料 script7（`ADD_2_72_11_10`：生成型72@(11,10) 伏击）；触发层=floor8 脚本链后段；**臂未移植**（无分派，落 todo!()）【2026-10-16 更正】臂已移植（7934035，a.java:8355-8490 逐值对照，4 单测 script_add_spawns_entity_same_floor 全绿）——原文"臂未移植"系移植前普查 |
| MVS | ☠️ | 【2026-10-15 补登】死指令：68 条脚本（a.java:1124-1193）零出现且脚本文本源封闭（字节码 MVS 引理 L1-L5）——终裁 DC-2（[dead-code-rulings.md](dead-code-rulings.md)）；Rust 不移植（script.rs 已注）【2026-10-16 更正】"Rust 不移植"仅对 script.rs P3.1 解释器成立（mvs_is_p32_boundary panic 边界）——scene.rs **已实现** MVS 臂作保真死码臂（a.java:8519-8592 逐值，三钉单测 bd735f6，型82 配对互传 m_107）；☠️ 死指令裁定本身不变（语料 0 + 型82 出层 0 双重死码） |

## 3. 模式面

引用 completeness.md §3.1 模式矩阵（B 维主表）。死内容终裁台账：mode13/99
（☠️ 已裁死）、mode18 火花（☠️ 死代码 A 级）、mode19 商店场景（零覆盖待裁）。

### 3.1 死代码终裁销账（覆盖矩阵白项 b 类，2026-10-15）

裁决书：[dead-code-rulings.md](dead-code-rulings.md)（主张/证据/可复现命令/结论）；机械复核
`python3 tools/dead_code_ruling.py --check`（字节码 CFG + 字段值域不动点：168 方法中证死恰为下表 10 个）；
cover_matrix.py 渲染时三重校验（字节码证死 + 裁决书节锚 + JDI 51 场景零触达）后记 ☠️，不计判据①赤字。

| 裁决 | 方法 | 关联面 | 状态 | 切断点（字节码） |
|---|---|---|---|---|
| DC-1 | m_093（现名 drawLeftTriangleArrow） | walkPhase 3 道具轮盘（绘制臂 + run 键处理臂） | ☠️ | V(walkPhase)={0,1,2,4,5}：paintPlayerSprite@815 tableswitch 键 3 死边 |
| DC-2 | m_107 | DSL 第 16 码 MVS（§2） | ☠️ | MVS 引理：executeScriptInstruction@669 相等分支死（脚本文本源封闭、68 条零 MVS） |
| DC-3 | m_148 / m_149 / m_150 / m_151 / m_152 / m_153 / m_154 | 联网簇 + mode 99 | ☠️ | V(gameMode)∌99：run@674 键 99 死边；V(f_bool_31)={0}：run@4 ifeq 恒跳（唯一置真点在 m_153 内，自锁） |
| DC-4 | addSpark | mode 18 火花层 | ☠️ | V(gameMode)∌18：paint@30 键 18 死边 |

> 销账口径：上表 10 方法自覆盖矩阵「白·b 不触达论证候选」转「白·b ☠️ 已裁死」（白项计数不变、判据①赤字扣除）；
> mode 13/18/99 的「全量 grep 无入口」论证同步升级为字节码值域证明（gameMode 值域
> {0-5,7-12,14-17,19-22}，计算来源 menuReturnMode∈{0,1,4} / loadTargetMode∈{0,3} / inGameMenuReturnMode←gameMode）。

## 4. 层条件分支账本（17 处 currentFloor + 层相关 renderCategory）

| 位置 | 条件 | 状态 | 证据/注 |
|---|---|---|---|
| a.java:3189/3192/3195 | ==0 / ==50 / ==1&&wentDown | ✅/✅/✅ | boot/prologue/floor50/floor1 引子旗 |
| :4326 | >50 引子 UI | ✅ | floor51_menu/prologue |
| :5428 | 炼丹价 scaledByFloorTier | ✅ | mode9（<=10 段） |
| :5442 | 型11 墙文案 !=23 | ⬜ | **随型11 delta 一并覆盖**【2026-10-16 驳回翻 ✅ 依据】"floor40-mov 逐帧"不成立（sprite40 29 记录 0×型11/0×型12，字节级解析 A 级）；!=23 **真臂**无帧夹具（fx_item17a 输入走面板 300-380 未撞墙、fx_item18a 面板路径、floor40 无型11），**假臂**（23 层静默 a.java:5442-5444）=FX-INT-11-F23 扩展候选——两分支均未触达，保守留 ⬜ 待夹具【2026-10-16 增量复核（ec6fb55 后）】**假臂已钉**：FX-INT-11-F23 二撞 case11 floor==23 跳文案恒阻挡（fx_int11_f23_transform_silent_frames_match 显式断言 overlay 无「障碍物」= a.java:5442 条件假臂 + FLD overlayText 全程零变化）⇒条件**判定方向**已钉（Rust 若 `!=23` 写反必红）。**真臂仍缺独立帧锚——更正"item17a/18a 真臂帧锚"说法**：两夹具走面板路径（300-380 -6）不撞墙，case11 臂根本未执行、条件未求值（A 级：输入脚本 + 断言清单）；真臂文案=a.java:5443 源引 + ee52191 逐值移植，提示机制帧锚于 case10 同型 prompt（fx_firebump_floor13，floor13≠23，showOverlayMessage 同路径，B 级推导）。**销账路径（一次输入即闭）**：fx-item17a.txt 在 T250 加 `-1 撞 (8,3)` 一次（面板流程前）⇒ 真臂文案帧锚落地即翻 ✅。按 82 行定例口径（有臂可验**且已验** ⇒ ✅）真臂未验 ⇒ 本格保守留 ⬜ |
| :6022 | >50 状态栏 | ✅ | floor50/51 |
| :7165 | m_119(currentFloor) 层缓存 | ✅ | 换层链（floor2/floor_crossing） |
| :7191/7193/7195 | 换层钳制 ==1/51/50 | ✅ | floor2/51/50 |
| :7557/7565 | 炼丹 <=10 | ✅ | mode9 floor5 |
| :7644 | 难度 (floor-1)/10 | ✅ | 全战斗用例 |
| :7648 | >50 战斗 UI | ✅ | ending 战 |
| :7833 | ==40 | ✅ | 待查语义（case40）——delta【2026-10-16 语义销账】= `activateItem(byte)` switch（a.java:7784 方法，switch :7787）**case 20「对角瞬移」道具臂**内 `currentFloor==40`（现 a.java:7836，账本行号漂移 -3）→ 40 层禁瞬移提示"本层不能直接瞬移。"（:7836-7838），else 镜像格 `interactWithCell` 放行才传送（:7839-7850）——**与 boss 无关**，勿与另一处 ==40（a.java:10074 击杀钩子 case73 spawnBossEvent(6)）混记：那处已由 preset-itemboss40 覆盖。Rust 已移植（scene.rs:2877-2881，注引 a.java:7832-7850）。**保留 ⬜ 的 A 级理由**：40 层禁用**真臂**零差分夹具——场景语料穷举（reference/oracle/scenarios/ 全量）无 floor40+item20 激活：preset-item2122 只触 floor1 **假臂**（镜像格拦截分支），preset-itemboss40/preset-floor40-mov 不经 activateItem；grep "本层不能直接瞬移" 全仓仅实现行 scene.rs:2881（零测试触达）。禁用臂夹具（floor40+item20 场景）落地前不翻 ✅【2026-10-16 复核·按 AGENTS 死码/不可达判定标准】☠️ **证不出**⇒保持 ⬜：型20 有脚本出生源 `CES_20_4_4`（a.java:1143，`DES_1_4_4` 同格换生成）⇒ 可拾取持有，楼梯 ±1（:7855/7860）可携至 40 层 ⇒ 禁用**真臂可达**，不满足死码判定所需的值域/CFG 级不可达证明。夹具现状：floor40+item20 禁用臂场景既不在本批亦不在扩展候选（fixture-plan §3）——建议补列扩展候选 FX-ITEM-20A（floor40 栈含 20 → 使用 → 「本层不能直接瞬移。」且不消耗）【2026-10-16 翻 ✅】建议即落地（ec6fb55，A/B/C 三方逐字节一致）：fx_item20a_no_teleport_floor40_frames_match（floor40 uses=3 **两次使用** ⇒ case20 禁用真臂命中「本层不能直接瞬移。」不瞬移不消耗幂等（a.java:7835-7838，var2 恒 false ⇒ useItemStack :7766 短路），T1-680 全帧 + uses 恒 3 断言）；假臂（镜像格拦截分支）=preset-item2122 既有帧锚——**双分支齐**，行销 |
| :7855/7860 | 楼梯 ±1 | ✅ | 楼梯往返 |
| renderCategory[72] | floor50 特判 | ✅ | 迭代 56 |

## 5. 迭代队列（按账本差值，每轮先普查 spriteN 再定目标）

**普查首报（迭代 78，tools/type_census.py --all，54 层全扫，consumed==filesize
全过）**：

- **派生覆盖层 7 个**：floor21/22/24/27/28/44/50（类型集 ⊆ 已验集，无需铺）。
- **敌族 41-59 余量 = 3 层取样可闭**：floor8（41-46+48）、floor14（49-54 一次
  全含）、floor38（47+55-59 一次全含）+ 63 随手（floor47 delta 仅 63）。
- **零出现型 8 个（☠️ 候选，A 级普查证据）**：16 芭蕉扇、17 金勺子、18 玄明石、
  22 下行符、24 捆仙绳、25 幸运币、82 传怪点、87 菩提老祖——54 层实体表均无。
  注：82 在 m_122 隐藏列表（5/81/6/12/76/82/83）且 bump 臂存在 ⇒ 可能脚本
  显形而非死代码；16/17/18/22/24/25 疑脚本授予（GIN?）或死内容——逐个终裁。
  > 【2026-10-15 更正】「零出现」只对 spriteN **初始布置**成立，不等于无来源——5 个已查到运行期来源，
  > 均非死内容：16 ← 杨戬(69，floor35) 击杀、17 ← 啸天犬(67，floor15) 击杀、24 ← 玉皇大帝二阶段(71) 击杀
  > （floor49：case61 钩子 a.java:10127 生成；floor53 的 71 因 currentFloor>50 不发事件）——tickBattle 击杀钩子
  > a.java:10059-10068 按 bossTypeOrder {75,74,68,67,69,71}（:2198）调 spawnBossEvent(i)，bossEventSpawns[4]/[3]/[5]
  > 分别为 16@(5,5)/17@(6,4)/24@(6,2)（:2041 表）；18 ← 天宫商人(78) 报价 13（applyMerchantOffer
  > `pickupItemType(18)` a.java:7432）；22 ← script25 `CES_22_6_6`。余 25/82/87 在 CES/ADD 脚本、bossEventSpawns、
  > 商人报价、GIN 四类来源中均未见（非穷举，仍为候选）；82 另有 MVS 臂引用但 MVS 已证死（DC-2）。
- **boss 单点层**：68 太上老君 = floor25 独有；71+74 = floor53；84 嫦娥 =
  floor16/26/36；86 钹 = floor37 独有。
- **最小道具层**：floor13 = [1,7,8,10,36]（云雾+道袍，4 层最小）；12 隐形墙 +
  !=23 层条件 = floor23（[3,7,8,12,31,32,77]）；13 生死簿 = floor52；
  23 太公杖 = floor19；14 生死簿外 21 上行符 = floor45 既有（补撞即可）。

队列（按普查差值定批次）：

1. **B2a 敌族取样 3+1 层**：floor8、floor14、floor38、floor47（63）→ 41-59+63
   全闭（配合 enemybase* 表静态对表）。
2. **B2b 道具/特殊型批**：floor13（10 云雾+36）、floor23（12 隐形墙+!=23）、
   floor52（13 生死簿+11 能挖墙）、floor19（23）、floor45 补撞 21、floor25
   （68 boss+2 红门）、floor26（84 嫦娥+10）、floor37（86 钹+11）、floor51
   （6 隐形路径+2 红门——51 已有用例，route 增量）。
3. **B3 boss 批**：floor53（71 二阶段玉皇+74 巨灵神+6）、floor20（74 备选）、
   a.java:7833 ==40 语义查证。
   > 【2026-10-16 更正】7833 ==40 语义已查证（§4 行内销账）：activateItem case20 对角瞬移 40 层禁用，
   > **非 boss 关联**，自 B3 移出；余项 = floor40+item20 禁用臂差分夹具（随 item20 delta 批补）。
4. **B4 脚本臂批**：GIN/LAY/RES/SEE/SMS/ADD 普查→触发或终裁；零出现型
   16/17/18/22/24/25/82/87 逐个终裁（脚本授予 vs 死内容）；m_102 跨层快照、
   dp2=m_106 随触发层。
   > 【2026-10-15】零出现型中 16/17/18/22/24 已查到运行期来源（非死内容，见上「零出现型」更正），终裁对象
   > 缩为 25/82/87；DSL 第 16 码 MVS 已 ☠️ 终裁（§2、§3.1 DC-2）。
5. 每批完成：账本翻格 + 证据回填；无人触达的 ⬜ 逐个 ☠️ 终裁或补用例。

## 6. 层普查派生覆盖规则

对未跑层 N：`spriteN` 记录的类型集 ⊆ 账本 ✅/📦 集合，且无 §4 未覆盖层条件、
无 §2 未覆盖脚本码引用 ⇒ 记一行「N 派生覆盖（类型普查 = {...}，无增量）」。
普查脚本待工具化（tools/type_census.py，读 original/ jar 内 spriteN）。

## 7. 最后翻格批·注文骨架（2026-10-16 预写；证据指针占位，落地后回填并移入各行）

> **本节为预写草稿**：未落格、未翻状态、**禁止作为结论引用**（各行状态此刻仍 ⬜）。
> 用途 = 夹具（FX-INT-11-F23 / FX-ITEM-20A / FX-BOSS-68A / FX-LAY-2A）与 15 号修复落地后，
> 机械回填 `⟨…⟩` 占位即完成翻格。消费后本节转「已消费」标记（不删历史）。
>
> **裁决口径范式（82 行定例，随批推广）**：有臂可验且已验 ⇒ **✅**（trace 锚点或用例名）；
> 无臂可验 ⇒ **☠️**（死码/不可达 A 级实证入注）；被动/表数据派生 ⇒ **📦**。
> 体例照旧：日期更正 + 历史主张原地保留 + 每处翻格带证据指针（commit hash 或 a.java 行号）。
>
> 【2026-10-16 消费进度】条目 2（型12）、4（:7833）已随 ec6fb55 落行翻 ✅；
> 条目 3（:5442）假臂已钉、**真臂缺独立帧锚**（"item17a/18a 真臂锚"说法被输入脚本反证）留 ⬜
> ——销账=fx-item17a.txt 加 T250 一撞（见行内增量复核）；条目 1/5/6 待命（15 等 porter 修复、68/84 等主窗批）。

1. **15 筋斗云 ⬜→✅**（待 porter 修 scene.rs:4476-4477 `key_value`/`up` 求值顺序）：
   【2026-10-⟨日⟩ 翻 ✅】49/55 臂方向分歧已修（⟨fix commit⟩，⟨回归测试名⟩——修后 49=findFloorGateEntity(true)
   对应 a.java:5295、55=false 对应 :5301 分立 case）；被动链全核 = 拾取置位 tutorial_seen（scene.rs:5272，
   f_bool_12）+ 贴梯门控 stair_hint（:411，f_bool_13）+ 快捷键臂（⟨修复后行号⟩，a.java:5293-5306）；
   使用臂空操作系原版语义（a.java:7795-7800）。原文"📦 被驳回/留 ⬜"复核记录保留。
2. **型12 隐形墙 ⬜→✅**（待 FX-INT-11-F23）：
   【2026-10-⟨日⟩ 翻 ✅】FX-INT-11-F23（⟨commit⟩）⟨测试名⟩：floor23 撞型12 ⇒ 变形为型11（state2/row9/visible
   + dispatch 重派 + 阻挡续扫，a.java:5449-5456）——case12 变形臂帧锚 ⟨帧窗⟩。历史"驳回翻格依据"更正保留。
3. **:5442 型11 墙文案 !=23 ⬜→✅**（同批）：
   【2026-10-⟨日⟩ 翻 ✅】!=23 两分支同钉：假臂=23 层静默仍阻挡（a.java:5442-5444，⟨F23 帧窗⟩）；
   真臂=非 23 层提示（a.java:5443 文案，⟨fx_item17a 前置撞墙阶段 或 F23 非 23 参照窗——待落地形态⟩）。
4. **:7833 ==40 ⬜→✅**（待 FX-ITEM-20A）：
   【2026-10-⟨日⟩ 翻 ✅】FX-ITEM-20A（⟨commit⟩）⟨测试名⟩：floor40 栈含 20 ⇒ activateItem case20 命中
   `currentFloor==40` 禁用臂（a.java:7836-7838「本层不能直接瞬移。」+ 不消耗）；对照放行臂
   = preset-item2122 floor1 镜像格拦截分支（假臂）。语义考证史（activateItem case20 对角瞬移，2026-10-16）保留。
5. **68 太上老君 ⬜→✅**（待 FX-BOSS-68A 主窗）：
   【2026-10-⟨日⟩ 翻 ✅】FX-BOSS-68A（⟨commit⟩）⟨测试名⟩：floor25 战 68（HP4500/Atk560/Def310 a.java:1072）
   ⇒ 击杀 spawnBossEvent(2) ⇒ bossEventSpawns[2] 红钥匙×4 十字（a.java:2142）⟨帧窗+四钥格断言⟩。
   fixture-plan-boss 余 8 窗 = 二波扩展候选（注文明列不随本格销）。
6. **84 嫦娥 ⬜→✅**（待 FX-LAY-2A 主窗）：
   【2026-10-⟨日⟩ 翻 ✅】FX-LAY-2A（⟨commit⟩）⟨测试名⟩：script0 链 ⇒ `LAY_2` 出层（a.java:8355-8362）
   ⇒ `RES_0` easy 重置（:8470-8490）⇒ `GUT_1` 交接停扫 ⟨帧窗⟩——销 84 行核心出层语义；
   登场三出生源/scene26-31 门控余窗 = 二波扩展候选（注文明列）。

> 收敛口径（lead 2026-10-16 拍板）：fixture-plan-boss 10 窗不全做——2 主窗销行 + 8 窗转二波扩展候选；
> 执行序 items-evidence（F23/item20）→ 68/84 主窗（同人串行，tests/ 域不并行）→ 本节六行连翻。
