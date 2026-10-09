# assets/ — 派生资产（不入库，可再生）

由 `tools/extract-assets` 从 `original/囧囧西游-大闹天宫.jar` 生成：
解包资源、剥 2 字节头的 PNG、转换后的音频等。

**红线**：本目录内容一律可再生，不入库（.gitignore 排除）；修正逻辑写进工具，不动 `original/`。
