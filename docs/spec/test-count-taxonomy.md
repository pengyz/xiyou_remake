# 测试计数口径总注记（全局唯一定义，引用即引本文件）

> 【2026-17 lead 钉】"口径病"五次出现（dispatch review-D2/resource review×2/viewport
> review×2）后全局归一。**四面四数，互不矛盾，引用必须标面**：

| 数 | 面 | 基线文件 | 验证语义 |
|---|---|---|---|
| **55** | state-dump 状态金样面 | data/status/baselines/state-dump-golden.json | 状态倾印逐字节（fx 夹具 18+preset 30+其他 7） |
| **45** | oracle 三方面对拍面 | reference/oracle/scenarios/*.txt | A==B==C 状态差分（gates scenarios） |
| **21+** | fixture_frames 帧夹具面 | crates/game-oracle/tests/fixtures/*.frames.txt | 渲染帧哈希逐字节 |
| **44** | method_reach 触达面 | tools/method_reach 快照 | 方法触达覆盖率 |

**引用体例**：数字首现标面（如"55[状态金样]"）；跨面比较=口径错误；"金样"默认指
状态金样面（55），"帧基线"指帧夹具面（21+）。
