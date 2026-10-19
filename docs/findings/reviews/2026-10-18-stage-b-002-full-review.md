# 对抗审查：STAGE-B-002 全卷（九 commit + 小件）——有条件通过（硬 4 缺口 1 项）

> 审查人：adversary。对象：3569bfd/42ec97d/acf2847/e2c78d2/84016b9/cb27b19/c89c2d6/
> a119345/8ef2004 + 9795798（+3dc05ef ratchet 清账）。对照：002 版硬要求
> （命门/公式边界/小地图映射/登记册）+ 001 登记制。
> 手段：命门 diff 面核验、layout.rs 测试体逐条核、登记册 grep、RED 抽样对锚、
> 零漂移独立复跑、三对账。

## 判定：**有条件通过**——硬 1/2/3 全过且质量高；**硬 4 登记册条目缺失（阻塞 1 项）**；
另立**头号风险提示**（64 枚豁免验证债）

## 1. 硬 2 命门 —— ✅✅ 最强形式达成

- **测试面字节未动**：`git diff 3569bfd~1..8ef2004 -- crates/game-core/tests/ + game_view.rs`
  = **空**——walk_camera_golden 4 测、game_view 钳制 `(8,-210)`、ease 序列断言
  **一字未改**；scene.rs 全程 diff 仅 +3 条新断言、零既有断言变更。
  "classic 下断言一字不改仍绿"由**测试文件未被触碰**直接证明（比逐字比对更强）。
- **等价套件本体**（3569bfd）：`layout_classic_equals_constant_surface`（classic 逐值 =
  常量面）+ `camera_params_classic_substitution_is_identity`（106/64 代入化回原值断言）
  + `design_constants_modern_values_substitute_back`（⑤考证公式 classic 代入全回原值）+
  `ui_anchor_formulas_classic_identity_and_modern_pin`——命门判据机器化 ✓。
- 相机参数考证结论合理（EDGE_MARGIN=106/clamp=64 行为常量两预设同值，与尺寸稿无关）✓。

## 2. 硬 1 公式边界 —— ✅

两变体逐值断言齐：classic()==常量面全等；modern(4,4) **钳到原分辨率 ⇒ 视口推导类
逐值=classic**（最小=原分辨率锚）；modern(13,13) 的 416 面逐值钉（右块 175 起/金币
367/413 等）；`layout_modern_formula_derivation` 含任意格数抽样。min 下限语义
（≤7×10 钳 classic）与用户口径"任意格数、最小=原分辨率"一致 ✓。

## 3. 硬 3 小地图映射 + R6 —— ✅

- 映射表定死（MINIMAP_DOT_BY_CATEGORY：8 怪=红/2 物=白黄系/4 宝=紫/1·16·32 机关=炭黑）
  测试逐类断言 ✓；
- **R6 歧义修正（a119345，用户裁定）**：色相避门色 ≥25°（红 3°/黄 53°/蓝 197° 已被门
  楼梯占用 ⇒ 选品红 326°/紫 267°/无色系）+ 防撞门测试 `minimap_dots_disambiguated_
  from_base_palette`（精确互异+hue 距 ≥25°+低饱和豁免，断言消息即"色相混淆 …距 …°"）
  + 基底层型排除（门 1/2/3 与楼梯 7/8 不重复标注/覆盖）——与 claimed 行为红样本
  （"色相混淆 0° vs 3°"）语义吻合 ✓；
- classic 格粒 4/点 4 零偏离、modern 8/4 居格中（(ppc-dot)/2=0 化回原位）= 零偏离面 ✓。

## 4. 硬 4 登记册 —— ❌ 阻塞：STAGE-B-002 条目**未落**

stage-b-deviations.md 现仅 **STAGE-B-003 / STAGE-B-001 两节**（grep `^##` 亲验；
最后变更 bb894d0 = 003 登记行）。002 承诺的登记面（7 条考证表引用 / R6 语义偏离·
信息增密 / 暂不分级理由 / clamp 64 待并册 / 动态分辨率本身）目前只散在
layout.rs:361-372 注释与 commit message——**登记册无 002 节 = 硬 4 未销**。
整改（文本级一轮）：补 `## STAGE-B-002` 节——偏离点（动态逻辑分辨率/modern 新值/
小地图信息增密=玩家情报面变化）/用户裁定引用/考证表 7 条（指向 layout.rs 注释段）/
金样影响声明（**classic 默认零偏离 ⇒ 无新增豁免**——这点写明很重要）/clamp 64 待并册/
暂不分级理由/代码注释引用面。

## 5. TDD / 三对账 / 零漂移 —— ✅（附披露要求）

- RED：各 commit 三件套在文；行为红样本（色相混淆 0° vs 3° / fillRect 65,0,175,18
  未随屏）与代码断言面对锚（hue 抗撞测试 :795-846、UI 锚位测试 :724-731）吻合 ✓；
- 三对账：九 commit 全在 002 声明域（game-core layout/scene/walk + shell/win 缩放）；
  9795798=task-9 登记项代改（lead 派）✓；3dc05ef=ratchet 清账（收尾挂账归 task-6
  授权面）✓；
- 零漂移：core 167 + desktop 30 全绿独立复跑 ✓——**但计数须带豁免披露**：
  现网 `#[ignore]` **67 枚** = 64 枚 STAGE-B-001 帧回放豁免（登记册枚举、注释引 ID、
  64+3 与我 grep 逐数吻合）+ 3 个人工 dump 工具（gameplay_floor1，长期 ignored 非
  覆盖损失）。"167/30"是**已豁免面外**的零漂移——交付/汇报口径应带上这句。

## 6. 头号风险提示（非本次阻塞）

**L2/L3 帧金样全集（preset_cases 35/35、fixture_frames 19、boot_replay 2、mov/slot/shell 5）
离线**——验证债已诚实登记（夹具重制=Stage B 首个后续工程），但在其落地前，差分脊梁
只剩 L1 解析金样 + 行为测试 + Legacy 等价套件。要求：夹具重制**建 task 挂牌**
（owner/验收=64 枚豁免逐个摘除归零），豁免数不许再涨（新增必须进登记册）。

## 7. 备忘

- 8ef2004 shell 缩放（信箱+鼠标反映射+classic 零变化断言）随 shell 面过审 ✓；
  modern 试玩验收（--preset modern --grid 26,26）属用户人工面（AGENTS §5 验证不打扰）。
- gameplay_floor1 3 枚裸 `#[ignore]`（dump 工具）建议补理由标签，口径统一。
