# 对抗审查：STAGE-B-003 方案——素材管线重构（PNG+JSON 零硬编码尺寸）

> 审查人：adversary。对象：unpack（二进制→PNG+JSON）+ runtime 双模 + 可选回程打包 +
> 零硬编码尺寸（用户裁定：素材精细化，禁代码硬编码宽高）。
> 对照：AGENTS §2 真相源红线 / §3 证据纪律 / gotcha_dual-porting-surfaces-drift /
> BUG-008 字高教训 / 001-002 登记制。

## 判定：**方向接受，6 硬要求**（①④⑥ 为方案自列复核点的加强，另加 2 条结构性硬要求）

## 1. 硬要求 1【①加强】回程打包从"可选"升**硬门禁**——byte-identity 是迁移可信的唯一判据

"可选回程打包（供逐字节对照保真锚）"——**不可选**。表示迁移的可信度只能来自
**unpack∘repack = 恒等**（全资产：55 maplv + 55 sprite + 全容器 + tileset + 调色板/
帧宽表）：`repack(unpack(x)) == x` 逐字节。注意难点如实：PNG 重编码字节必变（压缩），
恒等判据是 **repack 端用解码像素+JSON 参数重建 packed 二进制** == 原二进制——
打包器须复刻原编码选择（调色板序/帧切分/对齐）。任何一资产不恒等 ⇒ 该资产迁移挂起。
此门禁入 gates 或常驻测试（不只迁移时跑一次）。

## 2. 硬要求 2【结构性新增】decode-equivalence 常驻门禁 + 双模=双移植面

game-data 双模（二进制解析保留 + PNG+JSON runtime）= **双移植面**（
gotcha_dual-porting-surfaces-drift 前科：改任一面必须对照另一面）。要求：
- **逐资产解码等价测试**（常驻门禁）：`decode(json+png) == parse(binary)`——
  像素逐位 + 元数据逐字段（tileset 切片坐标/帧宽表/glyph 度量）；
- 明确两面分工登记：二进制解析器=oracle/工具链专用，PNG+JSON=runtime 专用；
  谁改哪面、对照测试在哪，写进计划；
- 二进制 runtime 路径的退役条件写明（PNG+JSON 全量接管后弃用 or 永久保留对照）。

## 3. 硬要求 3【②加强】零硬编码审计=机械清单法（禁"我记得都改了"）

- **grep 模式全集**（<<4 / >>4 / 16 / 32 / 128 / 208 / 帧宽表字面 / 字高行距公式）
  逐条**处置表**：改读 JSON / 公式推导 / 保留字面量+理由——三选一，零遗漏；
- (v&7)<<4/(v>>3)<<4 里 **8 = cols**（tileset.json{cols}）不是尺寸而是**索引语义**——
  改读 JSON cols 时须断言 legacy 值 8（防语义漂移）；TILE_PX=16 全部位移点同表处理；
- **字体派生量是 BUG-008 教训面**：字高/行距/shim 度量改源（glyph.json）必须
  **legacy 值逐项不变**（现行 SoftFont 度量 = glyph.json 内容的恒等断言），
  且注意 font_inject 真字形 hook 的布局依赖同源；
- 计数断言族（load_container ui=25/menu=2/sflogo=8 等，shell/win/src/main.rs:81-116）
  的去向：计数进 JSON 元数据后，**等价锚不许消失**——旧计数断言改为
  "JSON count == 二进制解析计数" 对照（升格为硬要求 2 的一部分）。

## 4. 硬要求 4【结构性新增】真相源层级——assets/art 不得成为第二真相源

AGENTS §2：original/ 是**唯一资产真相源**；assets/raw 现为 gitignored 再生成物
（gates 自供给）。assets/art PNG+JSON 二选一并**写明**：
(a) gitignored 生成物（unpack 工具 + gates 启动自供给，与 assets/raw 同模式）；
(b) 入库但带**生成关系哈希清单**（源 jar sha256 + 每资产源偏移 + 产物 sha256，
gates 校验防漂移）。
禁第三态（入库且无生成锚）。此条是本方案最大的结构性风险——"精细化改造"后
改 PNG 不改 original 会无声漂移。

## 5. 硬要求 5【③】JSON schema 版本化

每 JSON 带 `version` 字段 + schema 文档；未知字段策略（前向兼容：旧 runtime 读新 JSON
忽略未知字段？还是严格拒收？定死）；**canonical 格式**（键序/缩进/数字格式固定，
git diff 可读、字节稳定）。

## 6. 硬要求 6【④⑥】依赖序 + maplv 边界

- **003 元数据先行**：表示迁移（零行为变化 + 硬要求 1/2 等价门禁）→ 002 消费元数据
  （Layout 公式从 JSON 读 cell/尺寸）——002 的"公式自动放大"以 003 元数据为输入，
  反序会让 002 先硬编码再换源=两次动同一批文件；两单不重叠推进，依赖写进两计划。
- **maplv 是数据不是素材**：w/h/cells 被 Layout（002）与存档语义消费，变更=行为面
  不是渲染面——归**数据域**（assets/data/ 或独立目录），登记册按数据变更门禁走；
  精细化叙事只覆盖图形素材（tileset/ui/sprite/glyph）。sprite 帧宽表同理（数据表）。

## 7. ⑤ oracle 锚（核过，无异议）

Java 参照读原二进制（class 资源链）不动 ✓；但 Rust 侧 oracle 对拍经 runtime 解码 ⇒
硬要求 2 的解码等价门禁即 oracle 锚不扰的机器面（像素恒等 ⇒ paint 恒等）。
L1 golden（game-data 二进制解析）保留 ✓。

## 8. TDD/验收骨架建议

①repack 恒等全量套件（硬门 1）②解码等价逐资产套件（硬门 2）③零硬编码处置表的
机械审计脚本（grep 全集 + 期望计数，防新增硬编码回潮——ratchet 式）④glyph 度量
legacy 恒等 ⑤JSON schema/canonical 快照测。
