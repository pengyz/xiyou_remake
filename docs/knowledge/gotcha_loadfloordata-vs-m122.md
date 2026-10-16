---
name: loadfloordata-vs-m122
description: probe/ADD 跨层往返的"载目标层"必须用 m_122 语义 reload_floor_entities——load_floor_data 是整层初始化（清小地图/重建视图/存档键层号），换用即帧差（8f41e6c 连带修复）
type: gotcha
created: 2026-10-16
sources: [8f41e6c]
---

`loadFloorData`（Rust `load_floor_data`，crates/game-core/src/scene.rs:876）≠
Java `m_122`（a.java:9449）：m_122 是**纯实体装载**（entityCount 归零 + entityTypeImage
表 + sprite{n} 解析/存档还原），而 loadFloorData 还做整层初始化（清小地图、重建
视图/可走性格）。probe/ADD 的跨层**往返**必须走 m_122 语义（Rust
`reload_floor_entities`，scene.rs:1340）；误用 loadFloorData 会把当前层视图状态
清掉重建 ⇒ 帧差（8f41e6c 连带修复暴露）。

Java 往返的权威形状（两处同构，逐字核对过）：

- `probeCrossFloorSnapshot`（a.java:8640-8657）：
  `m_119(currentFloor)` 存当前 → `m_122(target)` 载目标 → 扫实体 →
  `initCellGrid()` + `m_122(currentFloor)` 还原；
- ADD 脚本指令（a.java:8454-8464）：
  同前半 `m_119 + m_122` 载目标 → `spawnEntity` 造实体 → `m_119(target)` **存目标层** →
  `initCellGrid() + m_122(currentFloor)` 还原——注意 ADD 多一步"存目标层"，probe 不存。

**为什么：** m_122 每次都把 `entityCount` 归零后整层重装（a.java:9453-9455 起），
是"换层看一眼再回来"的最小动作；loadFloorData 携带的视图/小地图副作用属于
**真正的进层**（changeFloor 路径），往返场景多跑一遍就制造 Java 没有的中间帧。
另外 8f41e6c 连带修复：load_floor_data 的存档态改按**目标层**取键（此前按
self.floor——跨层 probe/ADD 会恢复错层状态，probeCrossFloorSnapshot 首用即暴露）。

**何时使用：** 任何"临时换层做事再回来"的路径（probeCrossFloorSnapshot、ADD 跨层
spawn、loadLevelScript case 31 的层重排）——载目标层/还原都用
`reload_floor_entities`；只有 changeFloor 这类真进层才允许 load_floor_data。
review 时 grep `load_floor_data(` 的每个调用点确认它在"真进层"路径上。
