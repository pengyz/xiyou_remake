# 架构评估：god-class 移植体与代码清理批 P2（task-22）

> 【2026-10-16】纯评估，未改任何代码。数据全部机械产出：`cargo clippy --workspace --all-targets`、
> 全仓 grep 精确引用计数、8 行滑窗重复块指纹、`wc -l`/fn 行号分布。结论供 lead 拍板，
> 拍板前不实施任何拆分。评估对象以 `crates/game-core/src/scene.rs`（god-class 移植体）为主，
> 覆盖 workspace 全部 4 crate。

## 拍板记录【2026-10-16 lead 裁决，验收本报告】

1. **文件级拆分 = 批准**（GameScene 结构体/签名不动、纯剪切移动、7 簇按风险升序）——
   作为 Stage B 开幕 refactor 执行；**时机 = clippy 批（task-21）落地后启动，实施前对最终树重验行号**。
2. **6 个 Java 签名逐参对应重参 = 永久保留**（`#[allow(clippy::too_many_arguments)]` +
   "对照性资产"注释显式化）——移植保真度的代价不是坏味道。
3. **3 个收结构体 = 批准**（BootAssets/Rect/PopSpec），排入拆分批一起做（避免与 clippy 批同文件冲突）。
4. **7 个真死候选 = 禁直接删**，走 `tools/dead_code_ruling.py` 逐个裁定（MVS/型82 先例）——排二波。
5. **execute_script_instruction 不拆 switch + 不做类型级拆分 = 同意**（逐值对照体 + trace 契约红线）。

拆分时机"Stage A 收官后"已兑现（tag 已贴）；实施批按 §5 顺序图派工。

## 0. 结论（拆/不拆/拆分顺序）

1. **值得拆，但只做「文件级拆分」，不做「类型级拆分」**：把 scene.rs 的 186 个 `impl GameScene`
   方法按簇拆成多个 `.rs` 文件（Rust 允许同类型 impl 块分布于多文件），**不拆 GameScene 结构体本身、
   不改任何签名与表达式**——纯剪切移动，行为零差。
2. **时机**：Stage A 收官后（task-6 四本账全绿 + tag）作为 Stage B 开幕第一个 refactor commit。
   Stage A 期间不动——scene.rs 是差分对拍的载体，收敛期改它徒增 44 场景重跑风险。
3. **拆分顺序（风险升序）**：① 测试模块外移（6399-7703 共 1305 行，最安全）
   → ② 渲染簇 paint*（~930 行，自包含只读）→ ③ 菜单族（~600 行，enter/tick/paint 三件套自闭）
   → ④ 装载/存档/换层（~560 行）→ ⑤ 脚本解释器（~660 行）→ ⑥ 交互/战斗（调用网最密，最后）。
4. **too many arguments 9 处**：Stage A 全部保留（移植忠实性资产）；Stage B 只做 3 个低风险项
   （BootMachine::new→资产结构体、几何 4 参→Rect、popup::spawn→规格结构体）；
   其余 6 个是 Java API 形状的逐参对应，**永久保留**（对照性价值 > 惯例）。
5. **冗余低于外观印象**：13 个"死函数"中 2 个是 trait 实现假阳性、4-5 个是 MIDP shim API 表面积
   （刻意保真），真死候选约 7 个；46 组重复块绝大多数是 shim 保真双实现与 golden 测试内联期望
   （防实现漂移的刻意重复）——真正可收敛的重复很少。

---

## ① too many arguments 9 个函数清单（clippy 8/7 阈值）

| # | 位置 | 签名（参数分组） | 调用面 | 忠实 vs 惯例判定 |
|---|---|---|---|---|
| 1 | game-data/png.rs:91 `unfilter_decode` | **缓冲**(raw, out) **几何**(w, h) **格式**(bit_depth, color_type) **调色板**(plte, trns) = 8 参 | 1（私有，仅 `decode` 调） | PNG unfilter 算法的自然签名；单点私有调用，打包 `PngDecodeCtx` 收益≈0。**保留** |
| 2 | game-core/entity.rs:186 `spawn` | **世界**(table, grid) **实体**(type_id, px, py, param) **查表**(width_table) **可选**(img_h) = 8 参 | ~15（scene + 测试 + oracle 夹具） | 引擎核心 API，逐参对应 Java `spawn` 语义；改签名波及 15+ 调用点。**Stage A 保留，Stage B 可收 `SpawnSpec`** |
| 3 | game-core/menu_family.rs:197 `paint_titled_box` | **图形**(g) **图集**(ui0, ui13) **内容**(segs) **几何**(x, y, w, h) = 8 参 | 2 | 几何 4 参是 MIDP 签名形状；与 #4/#7 同型。**Stage B 打包 Rect（低风险）** |
| 4 | game-core/menu_family.rs:451 `paint`（包体） | **图形**(g) **文本**(text) **几何**(x,y,w,h) **装饰**(arrows, frame_counter, font) = 10 参 | ~3 | 同上；`font`/`frame_counter` 保真透传。**Stage B 顺手收矩形，装饰参保留** |
| 5 | game-core/paint.rs:42 `draw_image_clipped` | (g, img, x, y, sx, sy, w, h) = 8 参 | **64+**（scene 51 处 + menu/intro/popup） | **MIDP `drawRegion` 逐参对应**——签名即 Java API 形状，是对照性资产。**永久保留** |
| 6 | game-core/paint.rs:229 `BootMachine::new` | 9 个资产容器（l0/l1/sflogo/ui/menu/mapbg/intro/end/load） | ~8（preset_cases 等测试） | 纯装配器；打包 `BootAssets` 结构体零语义风险。**Stage B 低风险首选** |
| 7 | game-core/popup.rs:51 `spawn` | **内容**(kind, value) **世界坐标**(wx, wy) **相机**(cam_x, cam_y) **查表**(digit_w) = 7 参 | 7 | 逐参对应 Java `spawn`（a.java:9697 语义锚）；`cam_x/cam_y` 独立传是忠实于 Java 的坐标换算时点。**Stage B 可收 `PopSpec`，但需保留坐标换算注释锚** |
| 8 | game-core/render.rs:153 `get_rgb` | (out, offset, scanlength, x, y, w, h) = 7 参 | **0** | **MIDP `Image.getRGB` 逐参对应**（shim 全集表面积，render.rs 头注释明言"逐行复刻"）。零调用但属保真资产。**永久保留** |
| 9 | game-core/scene.rs:675 `load_floor` | **地图**(maplv) **玩家**(px, py) **存档**(records) **查表**(width_table, tile_walkability) **资产**(images, floor_assets) = 8 参 | 8（oracle 3 场景 + desktop + scene 内 2 处 + 测试） | 世界构建大装配；收 `FloorSpec` 可行但会波及全部 oracle 夹具调用点。**Stage B 末位考虑** |

**共性**：9 个中 6 个（#2/5/7/8 + #3/4 的几何部分）是 **Java 签名逐参对应**——这类参数表不是"设计债"
而是"对照性资产"（改了会破坏与 a.java 的逐参对照与 grep 可验证性）。clippy 信号在此语境下是
**移植保真度的代价**而非纯坏味道。建议：`#[allow(clippy::too_many_arguments)]` + 一行注释
"逐参对应 Java 签名（对照性资产）" 作为显式化声明（Stage B 批内做，1 行/处）。

---

## ② god-class 拆分可行性（scene.rs 7703 行）

### 2.1 现状解剖

- **逻辑 1-6398 行**（186 个 fn 中 ~150 个）+ **`#[cfg(test)]` 6399-7703**（1305 行单测）。
- 最大单函数：`execute_script_instruction` **425 行**（1779-2203，脚本 DSL 分派）。
- 方法簇（行号带 + 职责 + 约行数）：

| 簇 | 行号带 | 代表方法 | 约行数 | 内聚性 |
|---|---|---|---|---|
| A 装载/存档/换层 | 570-1130 | load_floor*, change_floor, swap_floor, save_floor_state | ~560 | 高（只动 world 构建） |
| B 脚本解释器 | 1540-2203 | load_level_script, open_dialog_page, execute_script_instruction | ~660 | 高（DSL 分派自闭） |
| C tick/相机/输入 | 2288-2530, 4025-4530 | tick, tick_dialog, handle_field_input, lock_camera_on | ~750 | 中（调度中心） |
| D 交互/战斗/步效 | 4523-5290 | try_step, interact_with_cell, tick_battle, apply_step_cell_effects | ~770 | 中（互相调用密） |
| E 道具/商人/炼丹 | 2905-3250, 3616-3970, 5289-5445 | activate_item, use_item_stack, apply_merchant_offer | ~800 | 中高 |
| F 菜单族 | 2545-2760, 3243-3520, 3973-4025 | enter_menu4/5/7/9/15/16/19 + tick/paint 变体 | ~600 | 高（enter/tick/paint 三件套自闭） |
| G 渲染 paint 簇 | 5471-6400 | paint, paint_entities, paint_hud, paint_dialog, paint_status_bar | ~930 | **最高**（只读状态→像素） |

### 2.2 拆分方案与"零行为差"验证路径

**方案（推荐）**：`scene/` 目录化 + impl 块按簇分文件，GameScene 结构体字段不动：

```
game-core/src/scene/mod.rs        // struct GameScene + 全部字段 + pub API 面
game-core/src/scene/load.rs       // 簇 A
game-core/src/scene/script.rs     // 簇 B（与现有 script.rs trait 层区分，或命名 dsl.rs）
game-core/src/scene/tick.rs       // 簇 C
game-core/src/scene/interact.rs   // 簇 D
game-core/src/scene/items.rs      // 簇 E
game-core/src/scene/menu.rs       // 簇 F
game-core/src/scene/paint.rs      // 簇 G（与现有 game-core/src/paint.rs BootMachine 区分）
game-core/src/scene/tests.rs      // 6399-7703 外移（或保留 mod.rs 尾部）
```

**零行为差的机器验证路径**（每步独立 commit，全闸重跑）：
1. **纯移动判定**：`git diff --stat` 只见行增删对称（move-only）；`rustfmt` 前后 `cargo build` 字节
   级一致性可用 `cargo rustc -- --emit=mir` 或直接以测试闸为准。
2. **最强闸 = 现有测试网**：252 项全绿（golden 帧哈希 + 脚本单测 + save roundtrip）——任何行为漂移
   会立即炸 golden 帧哈希（逐像素 sha）。
3. **oracle 三方对拍**：`gates/cli.py scenarios` 44/44（A==B==C 逐 tick trace sha）；
   `gates/cli.py reference` 确定性=PASS。
4. **长窗帧锚**：prologue T1-9800 全帧比对（9800 拍）、preset 系列 30-3400 拍窗口。
5. **编译器把关**：拆文件不改可见性——所有 `pub`/`pub(crate)` 保持原样 ⇒ API diff 为空。

**风险红线**：只动文件布局。**不**把字段分组拆成子结构体（会改 `global_snapshot`/`apply_global`
的借用形状与状态向量）、**不**改任何方法签名、**不**"顺手"改表达式——任何一项都会把
"零行为差"降级为"低行为差"，与 Stage A 保真纪律冲突。

### 2.3 不拆的部分（明确否决）

- **execute_script_instruction 拆 switch**：425 行难看，但它是 a.java:8355-8490 的逐值对照体，
  拆开后 grep 对照性下降；且脚本 DSL 语义仍在收敛（MVS/未知 op 语义刚终裁）。**不拆**。
- **类型级拆分 GameScene**：状态向量/trace 序列化以 GameScene 整体为契约，拆 struct = 改契约。
  **不拆**（除非 Stage B 有独立 trace 契约版本化方案）。

---

## ③ 冗余扫描

### 3.1 死函数（精确引用计数：全仓除定义行零出现，排除 #[test] 与 trait impl）

| 定性 | 函数 | 判定 |
|---|---|---|
| **假阳性** | game-desktop/main.rs:860 `resumed`、:887 `window_event` | `impl ApplicationHandler for Runner`——winit 框架回调，源码内无显式调用是正常的 |
| **shim 表面积（保留）** | render.rs:153 `get_rgb`、:219 `set_color_rgb`、:411 `draw_arc` | render.rs 头注释："逐行复刻 `reference/shim/src/javax/microedition/lcdui/Graphics.java`"；`draw_arc` 本注明"参考版未调用，保真复刻以对齐 shim 全集"。**保真资产，不删** |
| **对偶 API（保留或删）** | game_view.rs:158 `transform_at` | 与 `terrain_at` 对偶的 Maplv 查询 API；terrain_at 有调用、transform_at 零调用。可注释性保留（对偶完整性）或走 dead_code_ruling 裁定 |
| **真死候选（待裁）** | menu_family.rs:72 `set_scroll`、:405 `needs_layout`；intro.rs:307 `new_game_stats`；render.rs **无**；combat.rs:107 `predict_hp_loss_vs_type`；camera.rs:90 `snap_anchor_to_player`；paint.rs:610 `set_rng_seed`；walk.rs:90 `tick_stepping` | 7 个：全仓仅定义行出现。其中 `predict_hp_loss_vs_type`/`tick_stepping`/`snap_anchor_to_player` 像是移植过程中预留的接线点（tick_stepping 可能对应未接线的行走路径）。**处置：逐个走 `tools/dead_code_ruling.py` 流程裁定 ☠️ 或登记待接线，禁止直接删**（AGENTS 保真期行为只增不减） |

### 3.2 重复代码块（8 行滑窗指纹：46 组）

| 定性 | 代表 | 判定 |
|---|---|---|
| shim 保真双实现（~12 组） | render.rs:380-421 `fill_arc`/`draw_arc` 几何体部分逐行同构 | 两函数的 `op()` 记录串不同（fillArc/drawArc）+ 椭圆扫描同构——是"逐行复刻 shim"的代价。**保留**（提取公共几何 helper 会破坏逐行对照） |
| golden 测试内联期望（~3 组） | scene.rs:5647 `SEG67_PIECES` 常量 vs :6936/:6972 测试内联期望值 | 测试**刻意**内联独立期望（防实现漂移被测试自身掩盖）。**保留**（测试惯例） |
| 待查/可收敛（~31 组） | logo_anim.rs:173/183 等 | 多为短模板重复（enter/tick/paint 三件套样板）。菜单族拆分（§2.2 ③）时顺手收敛样板即可，**单独去重不做** |

### 3.3 未用类型

粗扫未发现未用 struct/enum（`game-platform` trait 全有实现方，`enums.rs` raw/from_raw 对偶保留）。
`menu_family.rs:586` clippy 提示 `SlotSelect` 缺 `Default`——小项，Stage B 顺手。

---

## ④ 模块边界现状图

```
game-data（纯解析，无渲染）        game-platform（trait + 确定性基元）
├─ packed_png / png / inflate     ├─ Clock / GameRng / Renderer / Audio / Storage / Input / Font
├─ maplv / sprite / script        └─ hash（sha256，帧哈希）
└─ error
        │ 资源字节流                      │ trait 边界
        ▼                                ▼
game-core（无 I/O、确定性）──────────────┘
├─ scene.rs ★god-class 7703 行（逻辑 6398 + 测试 1305）
├─ game_view / camera / walk / pathfind      （世界/视角/行走）
├─ entity / combat / popup                   （实体/战斗/弹字）
├─ menu_family / intro / title / logo_anim   （UI 状态机）
├─ paint（BootMachine 状态机）/ render（MIDP shim）/ layout（坐标常量）
├─ save（RMS 二进制）/ script.rs（DSL 宿主 trait）/ audio（BGM 状态机）/ enums
        │
        ▼
game-desktop（winit 壳）── audio（cpal 混音）/ view / lifecycle / main
game-oracle（差分宿主，不计移植证据）── A/B/C 三方对拍 + 夹具
```

**边界健康度**：分层清晰（data→core→platform trait→shell），无逆向依赖；核心问题集中于
game-core 内部——scene.rs 单文件占 game-core 全部 10743 行的 **72%**，其余 21 个模块共享 28%。
scene.rs 与 menu_family/paint/render 的边界已有雏形（menu 三件套已外置），拆分是"顺势"而非"逆流"。

---

## 5. 实施建议汇总（待 lead 拍板）

| 项 | 建议 | 时机 | 风险 |
|---|---|---|---|
| scene.rs 文件级拆分（§2.2 七步） | **做**，按 ①→⑥ 顺序每步独立 commit + 全闸 | Stage A 收官后，Stage B 开幕 | 低（纯移动，golden 闸兜底） |
| 类型级拆 GameScene / 拆 execute_script_instruction | **不做** | — | 契约级 |
| too-many-arguments 9 处 | `#[allow]`+注释显式化；仅 3 处低风险收结构体（BootAssets/Rect/PopSpec） | Stage B 清理批 | 低 |
| Java API 形状签名（draw_image_clipped/get_rgb/load_floor 等） | **永久保留**（对照性资产） | — | — |
| 死函数 7 个真死候选 | 逐个 dead_code_ruling 裁定，禁直接删 | Stage B 清理批 | 低 |
| 重复块 | 不单独去重；随菜单族拆分顺手收敛样板 | 随 §2.2 ③ | 低 |
