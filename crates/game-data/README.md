# game-data —— 资源解析（纯解析 crate）

《囧囧西游-大闹天宫》（Nokia J2ME / MIDP-1.0，SNOWFISH 2010）三种已考证资源格式的 Rust 解析器 +
script DSL 词法拆分辅助。字段语义严格对照
[`docs/findings/resource-formats.md`](../../docs/findings/resource-formats.md) 的字段布局表，
**不发明行为**；所有字段注释带证据指针（`文件:行`/字节偏移）。

## 红线（AGENTS.md §2）

- **纯解析**：无渲染、无平台 crate、零第三方依赖；库内**不做任何 I/O**（只吃 `&[u8]`）。
- 解析结果必须过 L1 golden test（见下节「测试」）。
- 语义待证点不落结论：如 maplv「每格 4 字节 vs 26×26 每格 1 字节」之争保持 findings 注 1 的待证状态。

## 格式一览

### ① packed PNG 多图集容器 —— `packed_png`

`N × [u16 LE 长度][PNG 字节]`，N 不写入文件内，来自常量表 `RESOURCE_IMAGE_COUNTS`
（`reference/seed/a.java:465-466`）。

| 偏移 | 类型 | 含义 | 证据 |
|---|---|---|---|
| 文件级 | — | 无固定头；张数来自常量表 | `a.java:465-466` |
| 每张记录 +0 | u16 LE | 本张 PNG 字节长度 | `a.java:2667`、`a.java:4914-4915` |
| 每张记录 +2..+2+len | byte[len] | 标准 PNG（魔数 `89504E470D0A1A0A`） | findings:80 |
| 下一张紧接末尾 | — | 无填充/对齐，解析零余字节 | findings:81 |

### ② maplv 关卡 —— `maplv`

头 `u16 LE ×2`（原始值 = 真实尺寸 ×2，代码 `>>1` 还原）+ 两张等长网格（各 `Y·Z·4` 字节）。

| 偏移 | 类型 | 含义 | 证据 |
|---|---|---|---|
| +0 | u16 LE（=宽×2） | `width = raw>>1` | `a.java:7175` |
| +2 | u16 LE（=高×2） | `height = raw>>1` | `a.java:7176` |
| +4..+4+Y·Z·4 | byte[Y·Z·4] | 网格 1（地形 tile-id）：`id&7`=图集列、`id>>3`=图集行 | `a.java:7178,7181,7183,4998-4999` |
| +4+Y·Z·4..+4+2·Y·Z·4 | byte[Y·Z·4] | 网格 2（翻转/变换码 0-7） | `a.java:7182,7184,2708-2744,5000` |

每张网格 `Y·Z·4` 字节（`a.java:7178` `n3 = Y*Z<<2`）。两种视角字节等价（`4·Y·Z = 2Y·2Z·1`）：
「Y×Z 大格 ×4 字节」（findings 注 1 表述）与「2Y×2Z 格 ×1 字节」（引擎索引：行跨距 `Y<<1`，
`a.java:4991`/`a.java:5009`，每步 16px `a.java:5002`）。本 crate 存平铺字节网格（= 引擎 `byte[] i`/`j`），
并提供后者视角的 `tile_at`/`flip_at`（列数 = `raw_width`）。「设计者原意」仍为待证点（findings 假设 1）。

### ③ sprite 对象表 —— `sprite`

4 字节头（两个 `u16 LE` 拼 `u32` 计数）+ 变长记录（1B 类型码 + 2×u16 坐标 + 0/1/2B 附加参数）。

| 偏移 | 类型 | 含义 | 证据 |
|---|---|---|---|
| +0 | u16 LE | 计数低 16 位 | `a.java:7270` |
| +2 | u16 LE | 计数高 16 位（语料恒 0） | `a.java:7270` |
| 记录 +0 | u8 | 类型码（`this.m[code]` 得中文名） | `a.java:7272`、`a.java:545` |
| 记录 +1 / +3 | u16 LE | x / y 坐标（像素） | `a.java:7273-7274` |
| 记录 +5 | 0/1/2 B | 附加参数**原始字节**（长度 = `extra_len(code)`） | `a.java:7275-7341` |

`extra_len` 逐类型码来自 switch 分支：2B = `7/8/76/82`；1B = `4/5/9/57/59/70/71/72/73/77/78/81/83`；
其余（含 `6/12` 与 `default`）= 0B。switch 里的值后处理（`+1`、与 `this.m[]`/`this.n[]` 拼接、
`case 76/82` 高字节 `+1`）属解释器层语义，解析层只还原原始字节（findings 假设 2 明确未穷举）。

### ④ script DSL 词法 —— `script`（供后续解释器）

指令间以空格分隔；每条 = 定长 3 字母指令词（`a.java:6121`）+ `_` 分隔的整数参数
（`a.java:6440-6446`，末参数以空格结尾）。

- `split_instr_token("CES_84_6_11")` → `("CES", [84, 6, 11])`；`"DES_-59_0"` → `("DES", [-59, 0])`
- `lex_instr_stream(stream)` → 空格切分整条指令流
- `parse_guts_line("0 CES_84_6_11 …")` → `GutsLine { layer: 0, instrs: […] }`（`GUTS:` 段行 = `层号 + 指令流`）
- 只切词，**不解释语义**、不校验参数个数；`MOT_IF`/`MOT_L{n}`/`POST` 等非指令常量不在其内
  （[`docs/findings/script-dsl-semantics.md`](../../docs/findings/script-dsl-semantics.md)）。

## 用法

```rust
use game_data::{image_count, Maplv, PackedPng, SpriteTable, split_instr_token};

// packed PNG：张数来自常量表（文件内不存）
let count = image_count("mapbg").unwrap() as usize;
let packed = PackedPng::parse(&mapbg_bytes, count)?;
for img in &packed.images {
    // img.bytes 即标准 PNG（魔数开头），零拷贝切片
}

// maplv：头 >>1 得尺寸，两张 Y·Z·4 字节网格
let maplv = Maplv::parse(&maplv0_bytes)?;
let id = maplv.tile_at(5, 3).unwrap();
let (col, row) = (Maplv::tile_atlas_col(id), Maplv::tile_atlas_row(id));

// sprite：计数头 + 变长记录
let table = SpriteTable::parse(&sprite1_bytes)?;
assert_eq!(table.count() as usize, table.records.len());

// script DSL 词法
let instr = split_instr_token("MOV_72_3_7_1_8")?;
assert_eq!(instr.args, vec![72, 3, 7, 1, 8]);
```

严格入口 `parse` 要求**零余字节**（余字节 ⇒ `Error::TrailingBytes`）；
需要前缀解析时用 `parse_partial` 返回消费字节数。

## 测试（golden test 全量覆盖）

`cargo test --workspace`：

- **55 个 maplv**（含 maplv51 非方阵 12×13/1252B 特例）+ **55 个 sprite** + **16 个 packed PNG**
  全部解析且零余字节；尺寸/张数/取值域锚点与 findings 实测一致（文件内注释带行号）。
- 夹具从 `assets/raw/` 读取（[`tools/extract-assets/extract.py`](../../tools/extract-assets/extract.py)
  的产物）；**路径可配置**：环境变量 `XIYOU_ASSETS_RAW` 指向任一含同名资源的目录；
  默认路径缺失时测试自动调 extract.py 从 `original/` 解包生成（CI 全新 checkout 亦可跑）。

## workspace 布局说明

- [`Cargo.toml`](../../Cargo.toml)（仓库根）是**仓库唯一**的 `[workspace]` 定义（`members = ["crates/*"]`）：
  [`gates/cli.py`](../../gates/cli.py) 的 `gate_rust_tests` 以仓库根为 cwd 运行 `cargo test --workspace`，
  而 cargo 只向上查找 manifest，故 workspace 根必须落在仓库根（t7 已单根化，原 `crates/Cargo.toml`
  双根结构删除）。
- glob `crates/*` 的约束：**crates/ 下每个子目录必须是含 `Cargo.toml` 的 crate**（如 `game-core`、
  `game-platform` 落地时各自带 manifest 即自动入workspace）；不要在 `crates/` 下放非 crate 目录
  （构建产物统一在仓库根 `target/`）。
