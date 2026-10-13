---
name: oracle-trace-phase-model
description: oracle trace 逐拍语义——dump@tickN = post-paint 状态；paint 消费同拍 mode-logic 更新；Rust harness tick_N(fc=N-1)→paint_N 与之逐拍等价
type: debug
created: 2026-10-13
sources: [迭代64 preset_floor35 T924-930 收敛（A35s1 stride=1 取证 + OPS sx 反推）]
---

oracle trace 与 Rust preset 测试 harness 的**逐拍相位模型**（迭代 64 floor35 T924
考证确立，此前多次因误读此模型空转数小时）：

- Java 主循环一次迭代（trace 标号 TICK N）的顺序 =
  `[preTick_N 输入] → [postTick_N / run case 模式逻辑，frameCounter=N-1] → [paint_N] → [FRAME sha + OPS + FLD dump]`。
  **dump@tickN = post-paint 状态**（含同拍 mode-logic 的写入与 paint 自身的写入——
  paintTileLayer 头段的 wp=4 推进就写在 paint 里）。
- Rust preset 测试循环 = `[应用 pending 输入] → scene.tick(fc=rec.tick-1) → paint → sha 比对`。
  因 `tick_N(fc=N-1)` ≡ Java 迭代 N 的 mode-logic（同 fc 同奇偶门），Rust 的
  paint_N 与 Java 的 paint_N 消费**同拍**更新——这是 20+ 测试全绿的相位基础。
- **推论（排障用）**：Java 中"写在 paint 路径"的状态推进（paintTileLayer 的
  f_int_55/f_int_54、advanceBobPhase）若被移植到 Rust tick 路径，会与同拍
  `advance_frames`（tick 内、奇偶门控）产生**同拍顺序倒置**——实体先被多推一帧，
  paint 用错帧偏移。凡涉及"paint 时点推进的状态"，Rust 必须同样挂 paint 时点。

**为什么：** 迭代 64 曾据 stride=8 trace 的 dump 值反推进而自相矛盾（cnt 首增拍
907 vs 908 反复跳），直到 `--dump-stride 1` 全量取证 + 逐拍读 OPS 的 setClip/dx
反推 sx 才定案：dump 标签、fc 值、paint 时点三者错位一格即推导全错。

**何时使用：** 任何"两侧状态 dump 相同但帧 sha 不同"或"推进计数差一拍"类分歧，
先对照本模型核对两侧写入点（tick 还是 paint）与 dump 标签语义，再动手改代码；
取证用 `python3 reference/oracle/run.py --diff-abc --dump-stride 1`（或直接调
`run.variant_cmd` 单变体），配合 `fields.py --timeline` 逐拍读。
