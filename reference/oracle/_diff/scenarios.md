# 场景套件三方对拍（--scenarios）

每个场景 = 独立输入脚本 + tick 预算（脚本头部 `# ticks: N`），A（原始字节码）/B（重映射）/C（javac 源码）同输入运行，trace sha256 三方一致才 PASS。变体准备/T-变换与本报告均确定性。

| 场景 | ticks | 预置 | A==B==C | trace sha256（三方） |
|---|---|---|---|---|
| boot-menu.txt | 150 | ✓ | **PASS** | `eec4428d2172fd15…` |
| enter-game.txt | 500 | ✓ | **PASS** | `a2c7769d5d1d4f06…` |
| floor1-to-floor2.txt | 1500 | ✓ | **PASS** | `c7d8ba21b44f5e3c…` |
| floor1-tour.txt | 8000 | ✓ | **PASS** | `2ddbccac558656af…` |
| floor2-tour.txt | 2000 | ✓ | **PASS** | `ef320b45f4ebed38…` |
| floor51-menu.txt | 9800 | ✓ | **PASS** | `b441b91d3c6df283…` |
| gameplay-floor1.txt | 3000 | ✓ | **PASS** | `cceb91defd67b2ac…` |
| menu-sweep.txt | 2200 | ✓ | **PASS** | `aee5eab7ca5243f7…` |
| preset-alchemy.txt | 700 | ✓ | **PASS** | `e22bff045507f340…` |
| preset-ending.txt | 1800 | ✓ | **PASS** | `db95a28a7e58b219…` |
| preset-floor10.txt | 1310 | ✓ | **PASS** | `8d22ed4ea05a1808…` |
| preset-floor10b.txt | 850 | ✓ | **PASS** | `a264e9c7c74661bb…` |
| preset-floor14.txt | 4790 | ✓ | **PASS** | `a59b4fa487c6a6ab…` |
| preset-floor17.txt | 850 | ✓ | **PASS** | `ddf9d7607e36f80b…` |
| preset-floor2.txt | 600 | ✓ | **PASS** | `b761a90f0f600c63…` |
| preset-floor34.txt | 3030 | ✓ | **PASS** | `7c146538cf1274fb…` |
| preset-floor35.txt | 1200 | ✓ | **PASS** | `4cf082d4922e218b…` |
| preset-floor38.txt | 6050 | ✓ | **PASS** | `0396776e22ce0075…` |
| preset-floor41.txt | 1900 | ✓ | **PASS** | `6ec1769c11a43833…` |
| preset-floor42.txt | 1700 | ✓ | **PASS** | `656e376de4db86eb…` |
| preset-floor43.txt | 1500 | ✓ | **PASS** | `5efa6b32d891d5e6…` |
| preset-floor44.txt | 1600 | ✓ | **PASS** | `f33e8ca45e12e936…` |
| preset-floor45.txt | 1700 | ✓ | **PASS** | `4cf5275b9d771f31…` |
| preset-floor46.txt | 1700 | ✓ | **PASS** | `f34f5f66663dad65…` |
| preset-floor47.txt | 1350 | ✓ | **PASS** | `55c1510a168a25c9…` |
| preset-floor48.txt | 3440 | ✓ | **PASS** | `1f8cfe1aba222ded…` |
| preset-floor49.txt | 3400 | ✓ | **PASS** | `e9e16326ad108cfe…` |
| preset-floor5.txt | 1200 | ✓ | **PASS** | `b0c925e6363b57d1…` |
| preset-floor50.txt | 300 | ✓ | **PASS** | `34c530b689893cc3…` |
| preset-floor51.txt | 600 | ✓ | **PASS** | `2b947310894fd9f2…` |
| preset-floor8.txt | 8610 | ✓ | **PASS** | `ef5f61e3fd95ed11…` |
| preset-help.txt | 800 | ✓ | **PASS** | `6ac09bcf0c78895b…` |
| preset-item13.txt | 500 | ✓ | **PASS** | `c9b0291f9241b213…` |
| preset-item21.txt | 900 | ✓ | **PASS** | `c0b15b7f8e9b61c7…` |
| preset-item2122.txt | 900 | ✓ | **PASS** | `7651571d8050c3a4…` |
| preset-itemboss40.txt | 700 | ✓ | **PASS** | `679356cb79653316…` |
| preset-itemcymbal.txt | 600 | ✓ | **PASS** | `9838307ba6132220…` |
| preset-itemstack.txt | 500 | ✓ | **PASS** | `c8a966708ab4f20b…` |
| preset-load-menu.txt | 800 | ✓ | **PASS** | `038a1ef425fd01e4…` |
| preset-merchant.txt | 600 | ✓ | **PASS** | `1d187095753922ed…` |
| preset-save.txt | 700 | ✓ | **PASS** | `829b9bc90aaa32f1…` |
| prologue-dialog.txt | 9800 | ✓ | **PASS** | `50f9c399fe9f37ca…` |
| prologue-patient.txt | 21000 | ✓ | **PASS** | `2449a089659fecab…` |
| prologue-walk-storypoint.txt | 1500 | ✓ | **PASS** | `d273e5d9b34bf1ab…` |
| walk-pure.txt | 4000 | ✓ | **PASS** | `3ad6ef6ad65b0ce0…` |

## 覆盖面说明（脚本 `# cover:` 声明）

- **boot-menu.txt**：boot→声音询问→主菜单（mode 0/21/1）+ 菜单导航
- **enter-game.txt**：菜单确认→mode14 读条过场→mode2 资源载入→mode11 片头对话层→玩家落位序章地图（currentFloor=51, HP 498→300）
- **floor1-to-floor2.txt**：floor1->floor2 全链：行走(4战)→踩型7楼梯(11,1)→changeFloor遮幅→floor2落地
- **floor1-tour.txt**：第 1 层游走圈图——大范围踩格探测战斗/门/钥匙/道具/楼梯事件（走向由 seed=7 固定）
- **floor2-tour.txt**：floor2 巡回——型11 道具浮层、药水/大血拾取、宝箱 type4（组开门 walkPhase4）
- **floor51-menu.txt**：序章 6 句对话推进（菩提老祖/悟空）→SEE_3_10_166_166_1 电影镜头等待态→mode10 大地图 UI 开关（-6/-7 实测生效）
- **gameplay-floor1.txt**：真实玩法（存档预置直达第 1 层）——继续游戏→存档槽0确认→m_115 读 MOT_L0 预置（HP500/ATK30/金100）→mode3 自由玩法：行走子状态机/金币拾取(+1)/格子移动
- **menu-sweep.txt**：菜单扫荡——道具栏(8)进出+列表导航/设置(16)开关切换/帮助(15)翻页/关于(17)/退出确认(22)→70帧淡出→应用自毁(thread-exit 收尾路径)
- **preset-alchemy.txt**：preset 直达第 12 层（type9 炼丹台 @cell(5,9)，facing==1 向下踩触发）：
- **preset-ending.txt**：终局链全编排（mode 20 结局卷轴，按层铺深层终点）：槽载入 floor50
- **preset-floor10.txt**：route_gen 生成（航点手拟，走步 BFS 填充 + 三重校验）
- **preset-floor10b.txt**：route_gen 生成（航点手拟，走步 BFS 填充 + 三重校验）
- **preset-floor14.txt**：B2a 敌族取样 floor14（51×2/49/52/50×2/54 七战 + 黄门×2 (3,9)/(6,8)，
- **preset-floor17.txt**：route_gen 生成（航点手拟，走步 BFS 填充 + 三重校验）
- **preset-floor2.txt**：preset 直达第 2 层（MOT_L0 current_floor=2）槽载入 → mode3 走查
- **preset-floor34.txt**：route_gen 生成（航点手拟，走步 BFS 填充 + 三重校验）
- **preset-floor35.txt**：按层铺 floor35（深层 boss 层，case69 击杀钩子首验）：槽载入
- **preset-floor38.txt**：B2a 敌族取样② floor38 南半（47×2+49+56+57+58+59 六型七战 +
- **preset-floor41.txt**：按层铺 floor41（B1 敌表族首层：60-64 敌环 + 83 双触发器 + 红/黄门）：
- **preset-floor42.txt**：按层铺 floor42（DES 剧情移除双 boss 首验）：载入 (7,11) →
- **preset-floor43.txt**：按层铺 floor43（80 银缕锁甲拾取 + 64/65 敌表族战）：
- **preset-floor44.txt**：按层铺 floor44（B1：新敌型 66 首战 + 守卫-宝箱-封印门配对 + 中央 40 型装备 m_083）：
- **preset-floor45.txt**：按层铺 floor45（B1 三层：78/77 双商人 f_byte_arr_13/12 表语义 + 黄门）：
- **preset-floor46.txt**：按层铺 floor46（85 黄金钥匙拾取 + 77 太白金星告示首验）：
- **preset-floor47.txt**：route_gen 生成（航点手拟，走步 BFS 填充 + 三重校验）
- **preset-floor48.txt**：按层铺 floor48（B1 收官：83(script38)/宝箱@1 同格 bump + 62 战×2 +
- **preset-floor49.txt**：按层铺 floor49（case70 击杀钩子首验 + 守卫宝箱 param 配对开门 +
- **preset-floor5.txt**：mode12 记录浏览器（迭代 59，道具14 全链）：槽载入 floor5 (4,8) →
- **preset-floor50.txt**：第 50 层直达（深层终局区首层，maplv50/sprite50 双记录
- **preset-floor51.txt**：preset 直达第 51 层（深层首例，maplv51/sprite51）：槽载入 → mode3
- **preset-floor8.txt**：B2a 敌族取样 floor8（41/42/43/44/45/46/48 七型各一战 + 5@1 守卫宝箱 + 黄门）：
- **preset-help.txt**：preset 槽载入 → mode4 菜单 → kind5 帮助（mode15 浮层全文/翻页/关闭）
- **preset-item13.txt**：preset（含道具13火眼金睛）槽载入 → mode10 道具面板 → 使用13 →
- **preset-item21.txt**：消耗臂 21 正路径（a.java:7851-7852）：槽载入 floor1 (3,10) 栈
- **preset-item2122.txt**：消耗臂 20/21（a.java:7832-7852）：槽载入 floor1 (3,10) 栈
- **preset-itemboss40.txt**：40 层 boss 击杀钩子（a.java:10053-10070）：槽载入 (6,3) 面向型73
- **preset-itemcymbal.txt**：钹（86 型道具）消耗链（killAdjacentAt a.java:7923）：槽载入 floor1
- **preset-itemstack.txt**：物品栈消耗链（useItemStack a.java:7761）：槽载入 → mode10 面板 →
- **preset-load-menu.txt**：preset 槽载入（MOT_L0 floor1 (3,10) hp500/atk30/def30 钥1/0/0 金100）直达
- **preset-merchant.txt**：preset 直达第 2 层（type78 告示牌 @cell(11,7) param01=报价0）：右行撞牌
- **preset-save.txt**：preset 槽载入 → mode4 菜单 → kind3 保存（mode7 存档槽 → -5 确认写回
- **prologue-dialog.txt**：序章 6 句对话推进（菩提老祖/悟空）→SEE_3_10_166_166_1 电影镜头等待态→mode10 大地图 UI 开关（-6/-7 实测生效）
- **prologue-patient.txt**：序章耐心版——对话后全程静默让 SEE_ 电影镜头自走（试验场景）
- **prologue-walk-storypoint.txt**：BUG-006 走位实验（task-20 一锤定音判据）——序章对话推进至 SEE_3_10_166_166_1
- **walk-pure.txt**：纯行走对拍轨迹（预设第 1 层，避开实体格）——Rust ReplayHost vs Java

