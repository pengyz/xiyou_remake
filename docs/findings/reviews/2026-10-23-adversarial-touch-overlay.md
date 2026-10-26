# 对抗复核 B：触控区可视化（overlay_rects）"框=命中同源"真实性

> 角色：对抗复核员 B（remake-verifier 纪律）｜对象：commit `1700cec` 件2
> 复核日：2026-10-23｜工作区：/home/peng/workspace/xiyou_remake
> 证据分级按 AGENTS.md §3（A=硬证据/实跑输出，B=可复现静态推理，C=猜测不入结论）。
> **产品仓零改动**：破坏性实验全部在 `/tmp/xym-adv`（`git archive 1700cec` 快照）内做。

## 0. 对象与方法

**成员自述（commit 1700cec 件2）**：overlay_rects **单源**——菜单行=TitleMachine::menu_item_rects
（与 menu_hit 共调）、UNDO=shell/common UNDO_ZONE_W/H 常量（右上角）、虚拟键区=PadLayout::for_modern；
present 前 SoftGraphics 描边叠加；钉=overlay_rects_match_hit_tables。红实录：初猜 UNDO 右下、
同源钉抓住脱节（实为右上）已改。

**方法**：
1. 逐元核"画框坐标源 vs 命中测试坐标源"（A 级行号证据）；
2. 破坏性验证（`/tmp/xym-adv`）：改源常量/调用点，看框与命中是否同变；自写探针钉对拍
   `TouchMapper::feed`（真实命中引擎）vs overlay 框；
3. 覆盖清单核对、金样污染实跑（`tools/modern_frame_baseline.py --check`）、档位/时序核。

## 1. 同源真伪（核心）

### 1.0 逐元总表

| 元素 | 画框源 | 命中源 | 判定 |
|---|---|---|---|
| 菜单行 ×6 | `native.rs:488` `menu_item_rects(sw,sh,18)` | `native.rs:924` `menu_hit(...,18)` → `title.rs:373` 共调 `menu_item_rects` | **黄**：函数单源 ✓，bar=18 字面量双写，钉盲区 |
| UNDO | `native.rs:491-497` `(sw-UNDO_ZONE_W, 0, W, H)` | `touch.rs:53-54` `undo_zone_hit`（常量 W/H，右上） | **绿**（常量级）：exp1 破坏性同变 ✓；摆放公式双写（黄注） |
| dpad | `native.rs:500-505` `(cx-30, cy-30, 60, 60)` **硬编码** | `input.rs:167-171` `in_dpad_capture` **r=2×r_switch=60 圆** | **红**：实跑脱节（§1.3） |
| 软键 ×3 | `native.rs:507-515` `for_modern(grid).soft` 逐元 | `TouchMapper::layout.soft`（生产路径=for_modern(g)） | **绿**（生产路径）/黄：overlay 重建构造器而非消费活对象 `wiring.pad` |

**成员自述红实录核对**：`touch.rs:54` `undo_zone_hit` 确为右上（`y < UNDO_ZONE_H`、`x >= w-W`），
overlay 框 `(sw-W, 0)` 同侧——自述"已改"属实 ✓。

### 1.1 菜单行框 vs menu_hit —— 黄

- 同源面：两侧共调 `TitleMachine::menu_item_rects`（`title.rs:359`）✓，框==命中表**函数级单源成立**。
- 脱节面：`bar=18` 是**两处硬编码字面量**（`native.rs:488` 画框 / `native.rs:924` 命中），
  不是共享常量；改任一处即静默脱节。
- 钉盲区（exp2 实证）：钉自带字面 `menu_item_rects(576,576,18)` 重算期望值，**对调用点脱节全盲**。
- 附注（B 级）：`menu_item_rects` 注释称"paint 与 hit-test 共调"，但 `title.rs` 内只有 :359/:373
  两处引用（hit 自调）——paint 侧菜单行画在美术图里，"框==可见行位置"未经代码证实（见附录假设）。

### 1.2 UNDO 框 vs undo_zone_hit —— 绿（常量级）

- 框与命中同取 `UNDO_ZONE_W/H`（`touch.rs:48-50`），摆放同为右上（`sw-W, 0` vs `x≥w-W && y<H`）。
- exp1 破坏性验证：`UNDO_ZONE_W 56→70`，框与命中**同变**，钉绿 ⇒ 常量级同源为真。
- 黄注：摆放公式两处各自内联（`sw - W, 0` vs `screen_w - W`），非共享 fn；钉只验"框中心命中"
  （弱钉：错尺寸框也可能过）。

### 1.3 虚拟键框 vs 按键命中 —— 红（dpad）/绿（软键）

**dpad 框≠命中（核心红点）**：
- 命中：`input.rs:167-171` `in_dpad_capture` = **圆**，半径 `r_switch*2.0 = 60`（含拖拽捕获迟滞）；
  另 `dpad_dir` 内 `r_inner=12` 死区。
- 画框：`native.rs:500-505` **60×60 正方形**（cx±30）——半宽 30 只有捕获半径的 1/2，
  且 30/60 是字面量、**不派生自 `r_switch`**。
- A 级实跑（`/tmp/xym-adv/tests/adv_dpad.rs`，经 `TouchMapper::feed` 真实命中引擎）：

```
probe px=101 py=508 keys=[Press(-4)] inside_box=false box=(26,478,60,60)
```

  距方向盘心 45px 的触点被命中引擎捕获出键，但落在框外 ⇒ **框≠命中**。
  命中区面积 π·60²≈11310px² vs 框 3600px²（≈32%）——框外 68% 命中区无提示。
- 钉覆盖洞：`overlay_rects_match_hit_tables` 只断言 menu/undo/soft 三项，**dpad 零断言**
  ——脱节点恰在钉盲区（commit 自述"软键框==PadLayout 单源逐元"回避了 dpad）。

**软键**：框==`for_modern(grid).soft` 逐元 ✓；grid=24 实跑探针（soft 中心 feed 命中且在框内）绿。
但 overlay 用 `PadLayout::for_modern(grid)` **重建**布局，而非消费活命中对象 `wiring.pad`
（`TouchMapper::layout`）——fallback 路径（`ensure_surface` 读窗失败 ⇒ resolution=None ⇒
`pad=default_240x320`，`native.rs:430`）下命中用 240×320 布局、框用 for_modern，**完全脱节且钉抓不到**。
生产 android 路径 `native.rs:424` 同步写 `pad=for_modern(g)`，实机不触发（黄）。

### 1.4 破坏性验证实录（均在 /tmp/xym-adv，产品仓零改动）

| # | 改动 | 期望（同源真） | 实测 | 结论 |
|---|---|---|---|---|
| exp1 | `UNDO_ZONE_W` 56→70 | 框/命中同变，钉绿 | 钉 `overlay_rects_match_hit_tables` ok | 同源真（常量级）✓ |
| exp2 | 命中调用点 bar 18→25（框侧不动） | 若钉完备 ⇒ 红 | 钉**仍绿** | 钉盲区实证：调用点脱节抓不到 ✗ |
| exp3 | dpad 边界探针（心+45px） | 框覆盖命中点 | **红**：`keys=[Press(-4)] inside_box=false` | 框≠命中 ✗（红） |
| exp4 | grid=24 soft 中心探针 | 框覆盖命中点 | 绿 | soft 同源（构造器级）✓ |

## 2. 框覆盖完整性 —— 黄

**有框**：菜单行 ×6、UNDO、dpad、软键 ×3（=成员承诺清单 ✓）。

**无框（可点面清单核对）**：
- **地图点格走位区**：`touch.rs` `classify_tap` → `cell_at` 全格网（点空地=WalkTo、点墙=ProbeStep）
  ——命中区随相机（cam_x/y）移动，无框（动态区需动态画框，未做）；
- **T3 滑动手势区**（`GestureTracker` 全画布，滑动=方向踏步）——无框（非固定矩形，可接受但未显式说明）；
- **UI 态 pointer_hit 区**（`scene.rs:9012` 背包栅格、`menu_family.rs:1355` 行带，paint/pointer_hit
  ③-1 共调）——无框；
- BACK=硬件键（`input.rs` BUG-020 恒吞），无触控区 ⇒ 不需框，清单显式记一笔 ✓。

**语义注意**：overlay 不分态常显——战斗/地图态仍画菜单行框；框=静态区域几何，非"当前可点"。
判定：用户点名三区有框，但可点面 > 框面 ⇒ 黄。

## 3. 叠加不污染 —— 黄（现有金样不红；写进画布为真）

- **事实（A 级）**：描边**写进 st.canvas 游戏画布**（`native.rs:953-959`：`SoftGraphics::new(&mut st.canvas)`
  内 `draw_rect`），之后 `argb.clone()` → `present`——"present 前叠加"属实，但**不是独立合成层**，
  画布缓冲/呈现帧含品红描边。
- **金样帧哈希实跑**：`tools/modern_frame_baseline.py --check` 驱动 desktop `target/release/xiyou`
  headless 帧，**不经 android overlay 路径** ⇒ overlay 无法弄红现有任何金样
  （modern-frame-baseline / state-dump / trace 同理，均走 game-core/oracle）。绿点 ✓。
- **溢出发现（非件2 责任，登记待查）**：1700cec 上 `--check` 实跑**红**——title/intro/menu 三态字节漂移。
  归因实验：
  - `a838581`（10-19 基线冻结点）同环境复跑 **绿**（5 态 sha 全同）⇒ 排除字体/环境漂移；
  - `/tmp` 拆除件1 清屏块（其余=1700cec）仍红：title 漂移值与带件1 **相同**（`aeff30551645`，非件1），
    intro/menu 两版各异（件1 有额外影响但非主因）
  ⇒ 漂移由 a838581→1700cec 之间（2c 输入/menu_hit 链路嫌疑）引入，件1 叠加二次影响。
  差异分类：门禁基线漂移（非 overlay 范围）→ 路由：门禁/件1 责任人逐 commit 二分（d80d96e 起）。
- **风险**：常驻 debug 描边若将来启用设备帧哈希/截图金样即污染；量产前需可关开关或独立合成层。

## 4. 布局档覆盖（18/24 格 + scale） —— 黄

- `overlay_rects(grid)` 全参数化；命中侧同 grid（`surface=g*32`、`map_touch_xy` 设备→画布反映射）
  ⇒ scale 不变性成立（两侧都在画布坐标域，`buf_w/buf_h` 参数实际未用——`let _ = buf_w`）。
- 实跑：grid=24 soft 框==命中 ✓（exp4）；grid=18 dpad 红（§1.3，与档无关的结构性脱节）。
- 缺口：钉只跑 grid=18 单档（13/24 档未钉）；fallback 路径 pad/框脱节（§1.3）钉亦不可见。

## 5. 时序（每帧 vs 一次） —— 绿

每帧（20fps present 节拍内）`machine.paint` 后、clone→present 前重算 `overlay_rects` 并描边
（`native.rs:951-959`）——**每帧重描**而非一次；框值静态 ⇒ 无残影/闪烁（modern 每帧全画幅清屏=件1；
android 恒 Modern，`native.rs:728`）。若未来跑 classic（paint 不清全幅），描边跨帧同位覆盖暂无实害。

## 6. 总判 —— 红

**核心宣称"框坐标==命中表坐标（防二次盲区）"不成立。**

- **红**：dpad 框 60×60 硬编码 vs 命中捕获圆 r=60——A 级实跑红证据（命中点在框外），
  且钉对 dpad 零断言，脱节点恰在钉盲区（自我宣称的"防脱节"机制未覆盖最脱节处）。
- **黄**：①menu bar=18 字面量双写，钉对调用点脱节全盲（exp2）；②overlay 重建
  `for_modern` 而非消费活命中对象 `wiring.pad`（fallback 路径静默脱节）；③覆盖清单缺
  地图走位/滑动手势/pointer_hit 区，框不分态常显；④金样虽不红但描边写进游戏画布（未来污染面）。
- **绿点**：UNDO 常量级同源真（exp1 破坏性同变，含成员红实录核对属实）、软键生产路径同源、
  叠加不进现有金样路径、每帧重描无残留。

**建议（仅裁定，不改码）**：①dpad 框改由 `r_switch` 派生（捕获圆外接矩形或直接描圆）+
钉补 dpad 边界探针（feed 对拍）；②`bar` 提共享常量、钉改为对命中调用路径断言；
③overlay 直接消费 `wiring.pad`；④量产前描边可关/独立合成层。

**Confidence: 90**（dpad 红= A 级实跑+行号证据；覆盖/语义类判断 B 级）。

## 附. 遗留假设与未核项

1. **金样漂移逐 commit 二分未做**（已定界 a838581 绿 → 1700cec 红、件1 非主因）——待门禁责任人。
2. **框==可见行位置**未证：菜单 6 行画在标题美术图内，`menu_item_rects` 几何 vs a.java 行位置
   需美术坐标考证（B/C 级，不入结论）。
3. 实机设备截图核对未做（验证不打扰纪律：全程无头/scratch，未碰用户桌面）。
4. `gates/cli.py status` 内含 `gate_rust_tests`/`gate_trace` 等长门禁，100s 超时跑不完
   （rc=124，非故障）；本次以定向实跑为准：`cargo test -p game-android` → native_parse
   26/26 ok（含双钉 modern_paint_no_residual_pixels、overlay_rects_match_hit_tables）。
