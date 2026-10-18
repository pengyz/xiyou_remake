# 对抗审查：task-8 素材管线 unpack/repack（STAGE-B-003 硬 1/5/6）——**通过**

> 审查人：adversary。对象：6eb5ff4（骨架+maplv）/ 75a8187（sprite）/ e3cdc18（packed_png
> + 恒等门禁）。android-shell 列席归域三点已锁定（maplv/sprite raw 原始值语义）。
> 手段：RED temp-archive 独立复现、schema 三验收面逐条核、raw 语义逐行核、三对账、
> 零漂移独立复跑。

## 判定：**通过**（硬 1/5/6 全销 + TDD/三对账/零漂移）；2 条建议不卡闸

## 1. 硬 1 恒等门禁 —— ✅ 销（设计与口径均达要求）

- **机制**（e3cdc18）：Mid.images 逐字节 PNG **不重编码**（调色板序/帧切分/对齐由逐字节
  保留承担）+ 帧序清单 `<name>_<i:02>.png` + repack = u16 LE 长度前缀重组 + 缺图/魔数/
  u16 域校验——恒等由构造保证，比"重新编码后比像素"更强。
- **判据口径**（tests/asset_roundtrip.rs）：逐资产断言、失败清单=资产名+两侧字节数、
  **禁聚合一锅端**（:3, :46-63）；`checked > 0` 防资产面缺失（:67）；常驻 gates ✓。
- **全集口径**：in_scope = maplv*/sprite*/image_count 域（18 容器名）——126 资产实测
  （55+55+16 在库；lead 首版过滤器漏的 8 容器名已补，e3cdc18 message 清单可核）。
- repo() nth(2) 勘误修正注明（:12-14）✓。

## 2. 硬 5 JSON schema —— ✅ 三验收面齐

| 面 | 核验 | 结论 |
|---|---|---|
| 信封 | version/format/name（mod.rs:9-47 schema 文档定稿）；**version≠1 拒绝**（mod.rs:123-125 显式） | ✅ |
| canonical 写出 | 键序固定/2 空格缩进/十进制/16 元素每行（json.rs:93-95）/最小转义（:74，仅 \" \\ \u00XX）/尾 LF——exact-string 测试钉（json.rs:330-351） | ✅ |
| 宽松读取 | 空白不敏感、未知字段容忍（含 bool/null 字面量，json.rs:172）、**重复键拒绝**（:194）、**小数拒绝**（数字=整数，:299 起）——测试逐条钉 | ✅ |

写出恒 canonical ⇒ 字节稳定 git diff 可读 ✓（正是硬 5 要的性质）。

## 3. 硬 6 数据域边界（android-shell 列席三点）—— ✅ raw 原始值语义守住

- **maplv**：`raw_width/raw_height` 存文件头 u16 LE **原始值**（"可能为奇数 ⇒ 必须存
  原始值，不得由尺寸反推"，mod.rs:26 + maplv.rs:4）；terrain/flip 原始整数数组 ✓。
- **sprite**：count_lo/count_hi **原始值**保留（count=lo|hi<<16 须 == records 长度，
  sprite.rs:48-60）；extra = 原始字节整数数组且 `extra.len() == extra_len(type_code)`
  校验拒绝（:73-77）；"语义后处理留解释器层，解析层不发明"（:6）——归域纪律正确。
- 校验负例测试（count/records 不符、extra 长度不符拒绝）+ count_hi 非零保留 ✓（:124-141）。

## 4. TDD / 三对账 / 零漂移 —— ✅

- **RED 抽查独立复现成功**：temp-archive 6eb5ff4^ + 叠 asset_roundtrip.rs ⇒
  `E0433: cannot find asset_pipeline in game_data` ×2，与 6eb5ff4 message 逐字吻合；
  e3cdc18 的红=16 项恒等破坏清单逐字入文 + 同法复现指令 ✓；"红不入树纪律"
  （判据先行版居 target/*.red）= A 协议合规执行 ✓。
- **三对账**：三 commit 全 crates/game-data（asset_pipeline + tests），内容/声称/写范围一致。
- **零漂移独立复跑**：cargo test -p game-data ⇒ 35+1+2+9+1 = **48 全绿**（含 task-10
  骨架 decode_equivalence 2 测），与声称精确一致。

## 5. 建议（不卡闸）

1. 恒等失败清单现给"两侧字节数"——建议加**首差异偏移**（调试一次定位），随下次迭代。
2. 恒等门禁依赖 assets/raw 在库（gitignored 自供给面）——CI 首跑注意夹具供给口径
   （gates 自供给机制已有，提示即可）。

## 6. 流程事故备案（PS 管道改码）

lead 报：PowerShell 管道改 UTF-8 源码致 GBK 误读损坏（当场发现 write 整文重建、
损坏态未入树）——已按其请落 knowledge 同族补注（与我两起 GBK 测量伪差同教训：
**涉中文取证/编辑一律 python utf-8 或 read/write 工具，禁 PS 管道转码**）。

## 7. 设计取向裁定（platform-contract ①之问：逐字节保留 vs 像素级中间态）——**逐字节保留满足且强于硬 1**

- 硬 1 的意图是**迁移可信 = 表示无损**（repack∘unpack = 恒等）。我原措辞假定中间态是
  重编码 PNG（像素级）才写"打包器复刻原编码选择"；**逐字节保留 PNG 子图 + 帧序 JSON**
  使恒等**由构造成立**——无损的最强形式，无需补编码复刻、无需改口径。✅ 取向认可。
- "编辑器改图后 repack 嵌新字节、**编辑资产不承诺恒等**"语义正确：恒等判据作用于
  **未编辑资产**（管线保真），不作用于精细化改图后（改图=有意变更）。
- **附带一条**（随 003 后续迭代）：改图后 repack 面须有自己的校验门（新字节嵌入 +
  容器重建过解析侧校验：计数/extra_len/u16 域），改图资产正确性由硬 2 解码等价门
  （task-10）+ 运行时金样承担——两门分工写进 003 登记册一句即可。
