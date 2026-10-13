# 覆盖矩阵（层次一账本，机器生成）

> 生成：`python3 tools/cover_matrix.py`（迭代 72 起）。方法清单 = 改名类
> javap 全量（103 方法，含构造/重载去重）。三色：绿=已移植+场景已验、
> 黄=已移植+场景未验、白=无移植账。**Stage A 判据：白项清零、黄项带论证**。

## 汇总：绿 44 / 黄 1 / 白 58（共 103）

> v1 粒度 = 子系统注解（人工维护于 cover_matrix.py COVERAGE 表，数据源
> completeness.md）。v2 = 字节码调用图插桩真覆盖。differ 层（状态向量）
> 见 tools/statediff.py。

## 绿：已移植+场景已验（44）

### boot/载入/存读档
- loadFloorSave
- loadSlotTable
- m_000
- m_001
- m_059
- m_068
- m_091
- m_117
- m_118
- m_119
- m_122
- saveGlobalProgress
### mode/菜单族
- m_092
### 交互/门/钥匙
- consumeKeyForDoor
- markEntityDesRemoved
- markEntityRemoved
- pullEntityFromCell
### 战斗/赏金
- removeLogoItem
- scaleEnemyStats
- spawnBossEvent
- tickBattle
### 绘制/状态栏/HUD
- buildMinimap
- drawParallaxBackdrop
- paintSoftkeyBar
- updateAndDrawParticles
### 脚本 DSL
- advanceScriptWalk
- flipOverlayPage
- loadLevelScript
- openDialogPage
- skipTextToLineEnd
### 行走/相机
- cameraAtTarget
- easeAfterimages
- handleFieldInput
- lockCameraOn
- m_025
- rebuildWalkability
- stepCameraTowardTarget
- tryStep
### 道具/装备/炼丹
- addItemToItemStack
- findItemStackIndex
- itemTypeToStackIndex
- m_076
- pickupItemType
- useItemStack

## 黄：已移植+未验证/挂起（1）

### 已知挂起（黄）
- m_106

## 白：无移植账（58）

### 未入账
- a
- activateItem
- activateMenuItem
- addMenuItem
- advanceBobPhase
- advanceEntityFrames
- alchemyPriceFor
- applyHpDelta
- applyMerchantOffer
- applyStepCellEffects
- boolean);
- byte);
- closeAudioPlayer
- deleteRecordStoreQuietly
- drawPopupLayer
- effectiveAttackVsType
- findFloorGateEntity
- floorTier
- handleOverlayKey
- hideNotify
- initEntityTables
- int);
- int[]);
- java.io.IOException;
- java.lang.String);
- keyPressed
- keyReleased
- loadFloorData
- m_008
- m_072
- m_089
- m_113
- m_120
- m_123
- m_128
- m_132
- m_148
- m_149
- m_150
- m_151
- m_152
- m_153
- m_154
- markSlotEntry
- paint
- paintLogoAnimation
- parseHexColor
- randomBelow
- releaseSceneImages
- resetMenuLayout
- run
- runIntroViewer
- saveFloorState
- showNotify
- sortEntitiesByY
- spendGold
- startNewGameLoad
- tryOpenChest

