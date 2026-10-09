# 场景套件三方对拍（--scenarios）

每个场景 = 独立输入脚本 + tick 预算（脚本头部 `# ticks: N`），A（原始字节码）/B（重映射）/C（javac 源码）同输入运行，trace sha256 三方一致才 PASS。变体准备/T-变换与本报告均确定性。

| 场景 | ticks | 预置 | A==B==C | trace sha256（三方） |
|---|---|---|---|---|
| boot-menu.txt | 150 | ✓ | **PASS** | `8d10a18f51373016…` |
| enter-game.txt | 500 | ✓ | **PASS** | `71376fbdcabe2c28…` |
| floor1-tour.txt | 8000 | ✓ | **PASS** | `10a98bf49537da8d…` |
| gameplay-floor1.txt | 3000 | ✓ | **PASS** | `3977b709354114c3…` |
| menu-sweep.txt | 2200 | ✓ | **PASS** | `b50d3d1999c9a39b…` |
| prologue-dialog.txt | 9800 | ✓ | **PASS** | `5e2625b6bc40496c…` |
| prologue-patient.txt | 21000 | ✓ | **PASS** | `7e8dc80846ab484f…` |

## 覆盖面说明（脚本 `# cover:` 声明）

- **boot-menu.txt**：boot→声音询问→主菜单（mode 0/21/1）+ 菜单导航
- **enter-game.txt**：菜单确认→mode14 读条过场→mode2 资源载入→mode11 片头对话层→玩家落位序章地图（currentFloor=51, HP 498→300）
- **floor1-tour.txt**：第 1 层游走圈图——大范围踩格探测战斗/门/钥匙/道具/楼梯事件（走向由 seed=7 固定）
- **gameplay-floor1.txt**：真实玩法（存档预置直达第 1 层）——继续游戏→存档槽0确认→m_115 读 MOT_L0 预置（HP500/ATK30/金100）→mode3 自由玩法：行走子状态机/金币拾取(+1)/格子移动
- **menu-sweep.txt**：菜单扫荡——道具栏(8)进出+列表导航/设置(16)开关切换/帮助(15)翻页/关于(17)/退出确认(22)→70帧淡出→应用自毁(thread-exit 收尾路径)
- **prologue-dialog.txt**：序章 6 句对话推进（菩提老祖/悟空）→SEE_3_10_166_166_1 电影镜头等待态→mode10 大地图 UI 开关（-6/-7 实测生效）
- **prologue-patient.txt**：序章耐心版——对话后全程静默让 SEE_ 电影镜头自走（试验场景）

