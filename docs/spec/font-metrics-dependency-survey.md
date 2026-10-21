# 字体度量依赖梳理：TTF 替换前置（font-metrics-dependency-survey）

> 【2026-10-19】纯梳理不动代码。机械扫描+逐处判读（口径：一处=逻辑编辑点〔文件级〕；
> 计数首现标注）。诉求：字体模糊→TTF 替换，**前置=布局依赖梳理+解耦**（字体高度被 UI
> 布局依赖）。与代码质量轮收尾并行。边界：asset-hardcode-audit.md（陷阱②字体面字面量
> 处置）/ resource-config-survey.md（资源 schema）/ STAGE-B-003 硬5（asset_pipeline）
> 三方已建档——本件聚焦**度量抽象与 TTF 解耦**，引用不重复。

## 0. 结论速览

1. **TTF 字形渲染已实现，缺口在度量**：D2 字形缝（`GlyphHook`/`set_glyph_hook`，
   render.rs:683-694）已由 shell/win 注入系统 TTF 真 CJK 字形（wqy-microhei，
   shell/win/src/main.rs:286-293）——**但 hook 只替换字形像素，不换度量**：`char_width`
   仍是 SoftFont 固定步进（ASCII 8px/CJK 2×），TTF 变宽字形被定宽步进排版 ⇒ 间隙不均/
   截断错位。**"字体模糊"的 TTF 替换诉求实际 = 字形（已解）+ 度量（本件）双件**。
2. **度量依赖三层**（§1/§2）：度量→几何派生（~12 处公式：行高 +4/+6、盒高 2h+4=48、
   标签居中 (32-h)>>1、optionRow h+8…）/ 度量→渲染（draw_string 步进循环 40 调用面）/
   度量→逻辑（wrap 129 换行/打字机推进/截断）。**BUG-008 实证**：字高 10→14 一处改
   ⇒ 框体观感全链+**全量帧夹具再生成**（303ffb9）——度量耦合史实。
3. **解耦方案**（§3）：`FontMetrics` 抽象（height/width/line_height/measure(text)→(w,h)）+
   **双预设**：classic=SoftFont 现值（6/14/11、8/14/11、12/20/16、BOLD+1、is_wide 2×，
   与 reference/shim Font.java 双拷贝同步）零偏离 / modern=TTF 实测度量（变宽）。
   预设门控判例沿用（classic 保真 vs modern 清晰化）。
4. **帧夹具影响**（§5）：classic 预设=字形+度量全保留 ⇒ **夹具不动**；modern=TTF 字形+
   变宽度量 ⇒ 帧变=**新帧基线双预设两套期望**（BUG-008 全量再生成先例 303ffb9 =
   重定标成本实证）。
5. 分工边界：asset-hardcode-audit 陷阱②管字面量处置（改 JSON/公式化）、本件管度量抽象
   （FontMetrics 接缝）与 TTF 变宽、resource-config/硬5 管资源 schema——三件共享
   "legacy 度量恒等"断言面。

---

## 1. 字体度量依赖普查（68 处度量引用[font.height/char_width/baseline/ascii_width]）

### 1.1 度量定义面（源头）

| 项 | 现状 | 锚 |
|---|---|---|
| SoftFont 结构 | `{ascii_width, height, baseline, style, size}`（render.rs:65-83） | Font.java:47-62 双拷贝同步 |
| 三档度量 | SMALL=(6,14,11)、MEDIUM=(8,14,11)、LARGE=(12,20,16)；BOLD aw+1（get_font，render.rs:86-94） | 2026-10-15 字高更正（10→14，BUG-008 根因修复 3423db7） |
| 步进规则 | `char_width = ascii_width × (is_wide ? 2 : 1)`；is_wide=CJK/全角区段表（render.rs:97-108） | Font.java:99-106 |
| 字形生成 | draw_glyph=确定性 hash 5 列点阵（占位）+ D2 hook 缝（render.rs:574-590） | Graphics.java:440-462 |
| **缺口** | hook 只换像素不换度量；measure=步进累加（substring_width，Font.java:87-96）——TTF 变宽无实测通道 | — |

### 1.2 依赖点逐处（位置/公式/调用面/影响面）

**几何派生类（~12 处）**：

| # | 位置 | 公式 | a.java 锚 | 字形度量变化影响 |
|---|---|---|---|---|
| 1 | menu_family.rs:508 | `line_height = font.height + 4`（textLineHeight） | a.java:471 | 行距全链（对话/浮层/图鉴分页） |
| 2 | menu_family.rs:1018/1098 | `(32 - font.height) >> 1` 标签居中（材质名"无"） | a.java:9107-9115 | 装备槽标签位移 |
| 3 | menu_family.rs:1037-1038 | `(32 - wideCharWidth >> 1)` 同族 | a.java:9107-9115 | 同上 |
| 4 | menu_family.rs:1052 | `content_h = 112 + lh*per`，lh=font.height+4=18、per=6 ⇒ 185（FRAME_Y 同源） | — | 菜单盒高连锁（FRAME_Y=185） |
| 5 | menu_family.rs:1316 | `box_h = 32 + (font.height + 8)`（optionRowHeight） | a.java:812 | 选项行盒高 |
| 6 | menu_family.rs:1398 | `menuLineHeight - fontHeight >> 1`、高亮 `120-(menuBoxWidth>>1)` | a.java menu 系 | 高亮条位移 |
| 7 | menu_family.rs:670 | (10+4)/12 字面量=**字高 10 时代遗留**（现 14 派生已公式化） | a.java:4354-4364 | 已公式化（3423db7） |
| 8 | render.rs:68 注 | 对话框框高 `2h+4=48=16×3`（16px 切片平铺整倍约束） | — | **框体切片耦合**（asset-hardcode-audit B8：步进 16 与框高 48 耦合） |
| 9 | scene.rs:1793 | 对白布局高 `(line_h << 1) + 12`（2 行+padding） | a.java:4944-5019 | 对话框内容高 |
| 10 | scene.rs wrap | `layout_wrapped_text_limited(w=129, …)` 文本宽 129 派生链 | a.java:8246 | **换行边界**（截断/分页） |
| 11 | 软键基准 | layout SOFTKEY 族（softkey_base_y 等，非直接度量但文本锚 y 基于行高） | — | 间接 |
| 12 | 文本 bbox | overlay 文本 y=234/240/212（viewport 复核已列固定排版） | a.java:3036-3039 | 浮层排版 |

**渲染类（40 调用面）**：draw_string/draw_text 族 40 处（scene 13/menu_family 13/
intro 5/render 8/paint 1）——步进循环 `px += font.char_width(ch)`（render.rs:570）+
glyph 绘制 `draw_glyph(ch, px, py)`（:571，字形框=char_width×font.height）。
TTF 变宽 ⇒ 步进须改实测 advance。

**逻辑类**：`substring_width`/`string_width`（换行判定/居中偏移/截断）——
打字机 `dialog_char` 推进（scene.rs:1793 limit=typewriterPos）、wrap 分页游标
（layout_wrapped_text_limited 的行列表）、`anchor_adjust`（BASELINE 用 font.baseline）。

---

## 2. 耦合形态分类

| 形 | 内容 | 度量变化影响 | 解耦难度 |
|---|---|---|---|
| **度量→几何派生** | §1.2 公式族（+4/+8/2h+4/(32-h)>>1/129…） | 几何全链位移（BUG-008 实证） | 中（公式已引用 font.height——**天然半解耦**：换度量源即联动，但舍入/整倍约束 2h+4=16×3 须保） |
| **度量→渲染** | draw_string 步进循环+glyph 框 | 字形间隙/重叠 | 中（步进换 measure/advance 通道） |
| **度量→逻辑** | wrap 129 换行/打字机推进/截断/分页 | 文本流断点变化=帧变+对话节奏变 | **高**（对话节奏=行为面，打字机 tick 数=f(字宽)） |

**BUG-008 教训面**（度量耦合实证）：字高 10→14 单点改 ⇒ ①框体观感全链 ②**全量帧夹具
再生成**（303ffb9）③双拷贝同步（shim Font.java↔SoftFont，pattern_dual-copy）。
⇒ 度量改动成本=帧基线重定标级别，**FontMetrics 抽象必须把"度量源"收敛到单点**。

---

## 3. 解耦方案草案：FontMetrics 抽象 + 双预设

```rust
pub trait FontMetrics {
    fn height(&self) -> i32;
    fn char_advance(&self, ch: u16) -> i32;          // 变宽实测通道（classic=定宽步进）
    fn measure(&self, s: &[u16]) -> (i32, i32);      // (w, h)——换行/居中统一入口
    fn line_height(&self) -> i32;                    // classic=height+4（a.java:471 锚）
    fn baseline(&self) -> i32;
}

pub struct ClassicMetrics { /* SoftFont 现值：6/14/11、8/14/11、12/20/16、BOLD+1、is_wide 2× */ }
pub struct ModernMetrics  { /* TTF 实测：fontdue/ab_glyph 度量+字形栅格化 */ }
```

- **布局公式从 metrics 派生**：§1.2 公式族的 `font.height`→`metrics.height()`、
  `char_width(ch)`→`metrics.char_advance(ch)`——**公式原式搬**（viewport 锚随迁同纪律：
  legacy 值恒等断言+取整逐 site）；
- **双预设门控判例沿用**（预设门控先例=GlyphHook None/Some）：
  **classic = 保真字形零偏离**（占位 hash 字形+现度量=差分逐像素可比，夹具不动）；
  **modern = TTF 清晰化**（真字形+变宽度量=观感档）；
- **legacy 恒等断言**（陷阱②共享面）：classic 度量逐项恒等（6/14/11、8/14/11、12/20/16、
  BOLD+1、行高 +4/+6、2h+4=48）——与 asset-hardcode-audit 陷阱②断言面同批（双拷贝
  同步 pattern 保留）。

---

## 4. TTF 渲染面

**现状**（已实现层）：
- 字形生成=draw_glyph 确定性 hash 5 列点阵（占位，Graphics.java:440-462 同构）+
  **D2 hook 缝**（render.rs:683-694）——shell/win 注入系统 TTF 真 CJK 字形
  （wqy-microhei/zenhei 路径表+msyh/simhei/simsun/Deng Windows 回退，shell/win/src/
  main.rs:252-293）；
- **TTF 替换点**（剩余）：①字形光栅化已通（hook_fn）②**度量通道未通**（char_advance 实测）
  ③换行/打字机消费 measure。

**依赖权衡**（零依赖哲学 vs fontdue/ab_glyph）：
- 当前 hook 实现已带 TTF 光栅化（shell 侧已引 TTF 库——核实 shell/win Cargo 依赖后记；
  若已引则 ab_glyph/fontdue 无新增成本）；
- game-core 侧**零依赖保持**：FontMetrics trait 在 core（无 IO/无平台 crate 红线），
  TTF 实测实现在 shell（注入回调同 GlyphHook 模式）——**core 不引 fontdue**；
- 备选：harness 侧占位（classic）+壳侧注入（modern）已是现架构——本件只补度量通道。

---

## 5. 帧夹具影响

| 预设 | 字形 | 度量 | 夹具 |
|---|---|---|---|
| **classic** | 占位 hash 字形（现状） | SoftFont 现值（恒等断言） | **不动**（64 帧夹具+金样 55 零差） |
| **modern** | TTF 真字形 | 变宽实测 | **帧变=新帧基线**（双预设两套期望） |

- **BUG-008 重定标先例**（303ffb9 全量帧夹具再生成）：度量改动=全量再生成成本实证——
  modern 基线生成走同管线（frames_fixture 重生成+冻结），**classic 期望保持字节不动**；
- 双预设两套期望的管理：classic=保真金样面（55[金样面]锁）；modern=独立视口基线族
  （viewport 调研 2c 自证基线同款——modern 期望不与 classic 差分，自证冻结）；
- **夹具协议**（viewport V3 同款）：modern 期望仅源自"modern 预设运行"的参照帧——
  禁从 classic 期望派生（防洗期望）。

---

## 6. 工作量与排期（估算件，实施计划另行）

| 步 | 内容 | 工时 |
|---|---|---|
| 1 | FontMetrics trait+ClassicMetrics 包装（legacy 恒等断言，零行为差） | 1-2 天 |
| 2 | 度量消费替换（§1.2 公式族/渲染步进/逻辑 measure 逐类，每类一闸） | 2-3 天 |
| 3 | ModernMetrics（TTF 实测度量+char_advance 通道接 hook） | 1-2 天 |
| 4 | modern 帧基线生成+冻结（双预设两套期望管线） | 1-2 天 |

合计 **5-9 天**（不含 android 壳 TTF 适配——shell/android 字体供给另属壳活）。
排期建议：**前置=表基建批**（legacy 恒等断言共享 asset-hardcode 陷阱②断言面）；
与 viewport Phase 2a（Layout 契约）互不触碰（度量源 vs 布局锚分层），可并行或后置。

## 7. 三方分工边界（避重声明）

| 件 | 管什么 | 本件关系 |
|---|---|---|
| asset-hardcode-audit（陷阱②） | 字体派生量**字面量处置**（改 JSON/公式化/保留，P8 82 命中） | 共享 legacy 恒等断言面；本件不动字面量只改度量源 |
| resource-config-survey | 资源 schema（sprite/anim/sheet） | 字体不入资源 schema（度量=行为面非资源面） |
| STAGE-B-003 硬5（task-8 asset_pipeline） | 资产中间态 JSON | 字形资产可入（TTF 文件管理），度量值不入 |
| viewport-adaptation | 视口/布局锚 | 度量→几何公式与 Layout 契约分层（公式原式搬共享纪律） |
