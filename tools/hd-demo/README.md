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

## 验收证据（A 级：管线断言/哈希恒等；画质 C 级不作证据）

- **v2（2026-10-21 用户反馈升级·细节补强）**：restyle 16 帧改 3x 原生 1px 精度细节——
  勾缝线/描边（灰缝、砖块轮廓、纹样亮边）、细纹理（石面颗粒/崩口、木纹、草叶）、
  焰心层次（内焰/外焰两色+火星）、调色小幅提亮对比（原调色板家族色系）。
  逐帧细节笔触记于 `frame-hashes.txt` detail 列。
- 确定性：`gen_hd_demo.py` 连跑两次，`map_00_3x.png + frames/` 聚合 sha256 同为
  `308a80b42c84269e9776a3d770bfd86d9d5442293a0179449d3278fa476e94ef`（v2，2026-10-21；
  不含 `compare/`（hd-visual 对照图产物）；v1=`fc2eb89c…`）。
- `--check`：104 帧 + 图集 384×624 在盘产物与 `frame-hashes.txt` 逐行一致。
- `cargo test -p game-data` 全绿（50 passed，含 R-7 护栏测试
  `demo_prefix_excluded_from_roundtrip_scope`）；`cargo test -p game-core --test hd_visual`
  5 passed（换 v2 图集复跑，结构判据零改动）。
- `python3 gates/cli.py check` = PASS 10 / FAIL 0 / SKIP 0。
- 帧哈希清单：`frame-hashes.txt`（源帧 00 PNG sha256
  `6e16f4b20c9f3e4c01e902a784ae9edc453cad4ee9f541d1bc30ac30f10ced95`）。

> 【归属勘误 2026-10-21】gen_hd_demo.py / frame-hashes.txt / README 初版与
> asset_roundtrip.rs 排除断言因并行共享 index 竞争，被 hd-render-path 的 commit
> `208bcf1`（feat(core): R-1 采样器）一并收录（内容原样、经双方核对无丢失）；
> 本节为同日补录的验收证据归档。不 rewrite 历史（并行工作区安全）。
