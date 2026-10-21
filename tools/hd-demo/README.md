# tools/hd-demo — 弃件② 3x demo 图集生成器（高清素材探针）

设计件：[docs/superpowers/specs/2026-10-21-hd-asset-probe-design.md](../../docs/superpowers/specs/2026-10-21-hd-asset-probe-design.md)
（R-4 图集切片恒等、R-7 弃件隔离）。**弃件**：只证管线（帧序/帧数/列数/密度换算），
画质不作证据；永不入 `original/`、不入保真表、不进金样运行。

| 文件 | 入库 | 说明 |
|---|---|---|
| `gen_hd_demo.py` | ✓ | 生成器（确定性输出=同输入同哈希；硬断言见文件头文档） |
| `frame-hashes.txt` | ✓ | 帧哈希清单（104 帧 + 图集；生成器产出，供漂移复核） |
| `../../assets/hd-demo/` | ✗（gitignore） | 产物：`map_00_3x.png`（384×624 = tileset 128×208 ×3）+ `frames/tile_XXX.png` 帧序 PNG 清单 |

用法：

```bash
python3 tools/hd-demo/gen_hd_demo.py           # 生成 assets/hd-demo/ + frame-hashes.txt
python3 tools/hd-demo/gen_hd_demo.py --check   # 复核在盘产物与清单一致（不写盘）
```

帧语义（A 级）：源图集 = `assets/raw/map`（packed_png 容器）帧 00 = 地图图块 tileset
（`paint_tiles` 消费，game_view.rs `(v&7)<<4`/`(v>>3)<<4`、16×16 切片）；
帧 = 16×16 tile 槽位，列数 8（asset-hardcode-audit 陷阱① 索引语义，legacy 断言不动），
帧数 104，帧序 = tile id 升序。混合质量：maplv0-54 用量 top16 高频帧程序化精绘，
其余最近邻放大垫底。
