# 零硬编码审计机械清单（STAGE-B-003 硬3）

> 2026-10-18 建档。作者：android-shell（task-9）。纯文档任务，零代码改动。
> 范围：`crates/` + `shell/` 全域（.rs）。方法：**机械清单法**——先定义 grep 扫描
> 集与命中计数，再逐条处置（改 JSON / 公式化 / 保留+理由 三选一），每条带
> `file:line` 指针（AGENTS §2：spec 结论必须带证据指针）。
> ⚠️ JSON 字段**只标注不定义**——schema 归硬5（task-8，`asset_pipeline` 中间态）；
> 本表引用处仅记"改 JSON（字段名由 task-8 定）"。

## 0. 处置口径

| 处置 | 含义 | 约束 |
|---|---|---|
| **改 JSON** | 数值来源换为资产中间态 JSON（task-8 schema），代码读值 | 必须**断言 legacy 值**（golden/decode-equivalence 锚，task-10）——读 JSON 后旧值不许漂 |
| **公式化** | 由已具名常量/字体度量推导（如 `font.height+4`），消灭重复字面量 | 派生式本身须有 Java 行锚（a.java:NNNN） |
| **保留+理由** | 不动 | 理由=非素材派生量/行为面字面锚/双拷同步面等，逐条写明 |

**两个陷阱（lead 指定，全表适用）**：
- **陷阱①**：`(v&7)<<4` / `id>>3` 里的 `8`（= `&7`/`>>3` 的模数）是**图集列数索引语义**
  不是尺寸——改读 JSON 时**必须断言 legacy 值 8**（列宽 8 变动会静默重排全部 tile-id
  映射 = 地图全花）。见 §3。
- **陷阱②**：字体派生量是 **BUG-008 教训面**（`docs/bug-ledger.md` BUG-008；
  `crates/game-core/src/render.rs:66-69` 注）：glyph 改源须**legacy 度量逐项恒等**
  （SMALL=6/14/11、MEDIUM=8/14/11、LARGE=12/20/16、BOLD+1；行高派生
  `font.height+4`/`+6`）。见 §4。

> 【2026-10-18 更正】对抗复核（docs/findings/reviews/2026-10-18-task9-hardcode-audit-review.md）
> 整改①②：Java 行号锚 3 处错位（A1/A2/陷阱① 的 :5002-5003、A3 的 :5006、D5 的 :812）
> 已逐条机械重校（grep 命中行=真锚）并全表改"方法名（行 N）"双锚格式——错位处原地
> ~~划线~~留痕不删。Rust 侧锚复核抽验全中，不变。§1 补计数口径声明。
> **同源错位延伸**（本次重校新发现）：`crates/game-data/src/maplv.rs:126-132` 注释引
> :5002-5003 同为陈旧锚（真锚 paintTileLayer（a.java:7045-7046））——注释修正在
> game-data 写域（task-8 侧），本表登记不代改。

## 1. 机械扫描集（可复现 recipe）

**计数口径声明**（【2026-10-18 更正】整改②）：下表计数 = **命中行数**（同行多次命中计 1），
非命中次数。复核人实测对照（次数计 P1=23/P2=211/P5=37/P6=15，本表不采用）；P2 拆注：
`<<\s*5` 单模式 = 98 行、`<<\s*5|>>\s*5` 双模式合并 = 149 行——表内 98 为前者
（原表模式栏写双模式而数用单模式，不自洽，已改）。

| # | 模式 | 命中（全域 .rs，行计数） | 语义类 |
|---|---|---|---|
| P1 | `<<\s*4` \| `>>\s*4` | 21 行（含 game-platform lib.rs:56 LCG `<<48` 前缀误中 1 行，语义排除） | 16px 瓦片坐标系 + UI 行距 16 |
| P2 | `<<\s*5`（单模式；双模式合并=149 行，见上拆注） | 98 行 | 32px 格坐标系（cell↔px） |
| P3 | `\b16\b` | game-core/src 223 + game-data/src 31 + shell 若干 | 混合（多数=mode 号/类目/hex/UTF-16 注，非尺寸） |
| P4 | `\b32\b` | 同上混合 | 混合（格距/类目/hud 槽位） |
| P5 | `\b128\b` | 30 行 | 实体槽容量/存档字段长/图集高（混合，逐条判） |
| P6 | `\b208\b` | 14 行 | 图集高/图鉴面板宽/存档槽表高 |
| P7 | `width_table\|type_width\|sprite_w\|frame_w\|animOffset` | 209+32 | 帧宽/动画表（数据域） |
| P8 | `字高\|行距\|line_h\|font\|glyph\|SoftFont`（game-core/src） | 82 | 字体派生量（陷阱②面） |
| P9 | `load_container(\|RESOURCE_IMAGE_COUNTS\|张数\|子图计数` | 842（含测试复制） | 容器计数断言族（ui=25 等） |
| P10 | `img_sig\|摘要\|128, 208` | 若干 | 素材尺寸/摘要断言 |

P3/P4 的"混合"处理：机械清单不漏扫，但处置表**逐条只列尺寸/步进/度量语义命中**；
非尺寸命中（mode 编码、实体类目、hex 基数 16、UTF-16 计数、算法掩码）归
§5"排除类"（计数+代表锚+保留理由），不逐条铺 223 行噪声。此口径为本表对
"机械"的定义：**扫描可复现、分类可核对、尺寸类零漏项**。

## 2. 处置表（尺寸/步进/度量语义命中，逐条）

### A 类：瓦片/格坐标系（16px 瓦片、32px 格）——**数据域**（见 §6 归域裁定）

| # | file:line | 内容 | 处置 | 理由/约束 |
|---|---|---|---|---|
| A1 | crates/game-data/src/maplv.rs:126-130 | `tile_atlas_col(id)=id&7`（~~a.java:5002~~【2026-10-18 更正：错位；真锚 paintTileLayer（a.java:7045）`int var5=(var18&7)<<4`】 | **改 JSON**（图集列数） | 陷阱①：改读 JSON **断言 legacy 8**；`<<4` 的 16=瓦片 px（A3） |
| A2 | crates/game-data/src/maplv.rs:132-136 | `tile_atlas_row(id)=id>>3`（~~a.java:5003~~【2026-10-18 更正：错位；真锚 paintTileLayer（a.java:7046）`var18=var18>>3<<4`】 | **改 JSON**（图集列数） | 同上，`>>3` 模数=列数 8，断言 legacy 8 |
| A3 | crates/game-data/src/maplv.rs:18 | 每步 16px 注记（~~a.java:5006 `n5+=16`~~【2026-10-18 更正：:5006=overlayPageHeight；真锚 paintTileLayer（a.java:7043/7055）列/行步进 `var6+=16`/`var7+=16`】） | **公式化**（引 TILE_PX） | 注记行，实值消费在 game_view（A5） |
| A4 | crates/game-core/src/layout.rs:86,89 | `CELL_PX=32`、`TILE_PX=16`（具名中枢） | **保留+理由** | 已公式化中枢（2026-10-15 卫生批 layout.rs:26）；值=数据域格距，不属素材精细化 |
| A5 | crates/game-core/src/game_view.rs:179-180 | `sx=((v&7)<<4)`、`sy=((v>>3)<<4)`（tileset 128×208 的格→像素） | **公式化**（×TILE_PX）+ 8 列断言 | 陷阱①核心命中：`&7`/`>>3`=索引语义；改 JSON 须断言列数 8 |
| A6 | crates/game-core/src/game_view.rs:111-138 | `>>4` 视口瓦片范围（col/row_start/end） | **公式化**（/TILE_PX） | 像素→格换算，消费面 paintTileLayer 索引式（a.java:7040-7042 tileColStart/RowStart）同源 |
| A7 | crates/game-core/src/game_view.rs:165-166 | `<<4` 格→像素反投影（col_start<<4） | **公式化**（×TILE_PX） | 同上 |
| A8 | crates/game-core/src/game_view.rs:182-190 | 绘制步进 `px+=16`、`py+=16`、edge patch 16×16 | **公式化**（×TILE_PX） | 瓦片步进 |
| A9 | crates/game-core/src/game_view.rs:261-266 | 测试锚 `((v&7)<<4,(v>>3)<<4)==(32,0)`、`==128` | **保留+理由** | legacy 断言面（改 JSON 后正是断言 legacy 8/16 的锚） |
| A10 | crates/game-core/src/camera.rs:32,60-77 | `cells<<5`/`>>5` 地图像素尺寸与夹取 | **公式化**（×CELL_PX） | loadFloorData（a.java:9423-9424）`mapPixelWidth=mapCellsWide<<5` + setCameraClamped（a.java:7062）夹取同式 |
| A11 | crates/game-core/src/game_view.rs:38,81-82,103,126 | `wide<<5`/`high<<5` mapPx、夹取 `(n+2)<<5` | **公式化**（×CELL_PX） | 同上 |
| A12 | crates/game-core/src/entity.rs:332-335 | `(px+16)>>5` 格吸附 | **公式化**（CELL_PX 半格偏移） | +16=半格（CELL_PX/2） |
| A13 | crates/game-core/src/entity.rs:348,510 | `sprite_w>>5` 格槽宽登记 | **保留+理由** | 数据域：槽登记语义 = spawnEntity（a.java:6389）段内（:6392-6420；game-core 侧旧名"open_n"），`>>5`=类目步进非素材尺寸 |
| A14 | crates/game-core/src/script.rs:393-394,415,448 | `x<<5`/`cx<<5` 脚本格→像素 | **公式化**（×CELL_PX） | save.rs:9 同式注记 |
| A15 | crates/game-core/src/save.rs:9 | `pixel=cell<<5` 注记 | **保留+理由** | 文档注记（存档格式说明） |
| A16 | crates/game-core/src/scene.rs:583-584,1301-1302,2161-2162,2191-2192,2279-2280,2318-2319,2442,2570-2571,3358-3359,5282-5283,6206-6207,6575-6576 | `<<5` 格→像素（12 生产点） | **公式化**（×CELL_PX） | 同 A14；测试面 `<<5`（scene.rs:6710-8464 约 55 处）随式替换或**保留+理由**（测试字面锚，"禁自证"注释见 scene.rs:8090） |
| A17 | crates/game-core/src/walk.rs:62-80 | `+16` 视口边缘判定（半格） | **公式化**（CELL_PX/2） | EDGE_MARGIN 语义同 scene.rs:4502-4523 |
| A18 | crates/game-core/src/entity.rs:334 注、scene.rs:4552-4567 | `cam_x±16` 通知判定 | **公式化**（±CELL_PX/2） | 同 A17 |

### B 类：素材图集/容器尺寸——**素材域**

| # | file:line | 内容 | 处置 | 理由/约束 |
|---|---|---|---|---|
| B1 | crates/game-core/src/game_view.rs:229 | 测试 `(tiles[0].width,height)==(128,208)` | **保留+理由** | legacy 摘要断言（task-10 计数断言族）；值来源随 B2 改 JSON |
| B2 | crates/game-data/tests/golden.rs:122 | "map 第 1 张 128×208 = 8 列 × 13 行 16px 格"注记 | **改 JSON**（图集布局元数据） | 128=8×16、208=13×16——列数/行数是**素材元数据**（陷阱①断言 legacy 8） |
| B3 | shell/win/src/main.rs:2252 | `img_sig(tileset)==(128,208,哈希)` | **保留+理由** | 素材摘要断言（真相源哈希清单 = task-10 硬4 落位处） |
| B4 | shell/win/src/main.rs:2253-2256 | backdrop 77×320 / actor 123×138 / menu_strip 68×102 / ending 240×145 摘要 | **保留+理由** | 同 B3 |
| B5 | crates/game-core/src/layout.rs:92-97 | `PLAYER_FRAME_W/H=41/46`、`OPEN_ANIM_W/H=27/29` | **改 JSON**（帧尺寸=素材切片元数据） | 与 entity.rs:391 (27,29) 行 6 特例同源 |
| B6 | crates/game-core/src/layout.rs:148-156 | 边框件 `BORDER_CORNER_W/MID_W/SIDE_W/STRIP_H=26/16/11/16`、源偏移 `BORDER_SIDE_SRC_L/R=42/53`（ui[8][0] 64×16 条带） | **改 JSON**（切片几何） | ui 条带切片尺寸=素材元数据；paintBoxFrame 消费 |
| B7 | crates/game-core/src/menu_family.rs:159-162,211,864 | `set_clip(…,26,16)`、`(…,14,16)`、`(…,18,16)` 切片 | **改 JSON**（切片几何） | 同 B6 |
| B8 | crates/game-core/src/scene.rs:6509-6524 | 对话框条带 `set_clip(…,16,16)`/`(229,…,11,16)` 步进 16 | **改 JSON**（切片几何）+ **公式化**（步进=2h+4=48/3，BUG-008 注 render.rs:68） | 步进 16 与字体框高 48 耦合（陷阱②联动：改源须恒等） |

### C 类：帧宽/动画表——**数据域**（lead 硬6 归域：Layout+存档语义消费）

| # | file:line | 内容 | 处置 | 理由/约束 |
|---|---|---|---|---|
| C1 | crates/game-core/src/entity.rs:224 + tests/fixtures/type_width_px.txt | typeFrameWidth 88 项（构造器 a()（a.java:436）表头（a.java:589）） | **改 JSON**（task-8 中间态字段） | 数据域：变更=行为面（格槽宽/绘制宽）；改 JSON 须逐项 legacy 断言（task-10） |
| C2 | crates/game-core/src/entity.rs:267 | animOffsetTable 11 行（构造器 a()（a.java:485-497）） | **改 JSON**（同上） | 数据域；scene.rs:7239 ROW3=[0,1,2,1] 消费锚 |
| C3 | crates/game-core/src/entity.rs:285-307 | dispatchSpriteSize 特例 sprite_w=96/32/45/27（dispatchSpriteSize（a.java:6300-6323）） | **保留+理由** | 数据域**分派语义**（Java switch 特例），非素材尺寸读值 |
| C4 | crates/game-core/src/entity.rs:231,301 + scene.rs:6586-6588 | `width_table.get(..).unwrap_or(32)` 兜底 32 | **保留+理由** | 越界兜底=防御语义（Java 越界行为另考）；32=格距非帧宽 |
| C5 | crates/game-core/src/entity.rs:226,239-244 | width_cells 类目→宽度（1/2/8/16/32→27 等） | **保留+理由** | 数据域类目分派（dispatchSpriteSize（a.java:6300）同型） |
| C6 | crates/game-core/src/entity.rs:10,34 | entitySpriteW（字段声明（a.java:125））注记 | **保留+理由** | 文档注记 |
| C7 | crates/game-core/src/scene.rs:5951,7212-7237 | 帧偏移=帧宽×animOffset（open_n 算式）消费点 | **公式化** | 随 C1/C2 改 JSON 后走表值 |

### D 类：字体派生量——陷阱②（BUG-008 面）

| # | file:line | 内容 | 处置 | 理由/约束 |
|---|---|---|---|---|
| D1 | crates/game-core/src/render.rs:87-90 | 度量常量 SMALL(6,14,11)/MEDIUM(8,14,11)/LARGE(12,20,16)、BOLD+1（:92-94） | **保留+理由** | shim Font 构造/度量（reference/shim/src/javax/microedition/lcdui/Font.java:32-62）**双拷贝同步面**（render.rs:69 注，pattern_dual-copy-algorithms-sync-fix）；glyph 改源须逐项恒等（陷阱②） |
| D2 | crates/game-core/src/render.rs:98-106 | is_wide 区段（0x2E80.. 等，Font.isWide（Font.java:99-106）） | **保留+理由** | 同 D1 |
| D3 | crates/game-core/src/menu_family.rs:318,508,786 | `line_height=font.height+4`（构造器 a()（a.java:471）`textLineHeight=fontHeight+4`） | **公式化**（已公式化）✓ | 派生式具 Java 锚 |
| D4 | crates/game-core/src/scene.rs:1349,1432,1786,2308,2535,6373,6503 | `line_h=font.height+4`（7 生产点） | **公式化**（抽 `text_line_height()` 单源） | 与 D3 同式多点——公式化收益=防行距分叉 |
| D5 | crates/game-core/src/scene.rs:2788-2789 | `menu_line_height=font.height+6`、`menu_visible_rows=238/lh` | **保留+理由** | 已公式化；真锚 resetMenuLayout（a.java:4682）`menuLineHeight=fontHeight+6`（:4683）（~~a.java:812~~【2026-10-18 更正：:812=optionRowHeight=fontHeight+8，张冠李戴】） |
| D6 | crates/game-core/src/scene.rs:3011,7721 | `pad=(menu_line_height-font.height)>>1` | **公式化**（已公式化）✓ | 居中 pad |
| D7 | crates/game-core/src/menu_family.rs:670-671 | "度量全部由 fontHeight 派生；(10+4)/12 字面量在字高 10 时同值、字高 14 后分叉"注 | **保留+理由** | BUG-008 根因修复面自注 |
| D8 | crates/game-core/src/menu_family.rs:1038 | wideCharWidth=12、fontHeight=14 注记 | **保留+理由** | 注记（值=D1 派生；Java 侧字段派生式 wideCharWidth/fontHeight（a.java:25-26）） |
| D9 | crates/game-core/src/menu_family.rs:639 | `slotsPerPage=min(6,208/14)`（dispatchModeEntry（a.java:4292）slotsPerPage/slotListHeight（a.java:4354-4364）） | **公式化**（208→SLOT_LIST_MAX_H、14→font.height） | 陷阱②：改派生须与 legacy 值 6 恒等（mot_if_truncation.rs:207 同式锚） |
| D10 | crates/game-core/src/layout.rs:324-327 | `SLOT_LIST_MAX_H=208`、`ENCYC_PANEL_W=208` | **保留+理由** | 已具名；=dispatchModeEntry 布局字面（a.java:4354-4364，数据域 UI 布局） |
| D11 | crates/game-core/src/scene.rs:1787,2309,2539,2554,6374,6504 | 对话框两行块高 `(line_h<<1)+12`（6 生产点） | **公式化**（抽 `dialog_block_h()`） | 2 行+12 padding=Java 布局式；陷阱②联动（框高 48=16×3 与 B8 耦合） |

### E 类：容器计数（ui=25 断言族）——**素材域**

| # | file:line | 内容 | 处置 | 理由/约束 |
|---|---|---|---|---|
| E1 | crates/game-data/src/packed_png.rs:33-50 | `RESOURCE_IMAGE_COUNTS` 16 容器（构造器 a()：resourceNames/resourceImageCounts（a.java:450-453）） | **改 JSON**（素材元数据清单） | 计数断言族锚随迁 task-10 decode-equivalence（"不许消失"） |
| E2 | crates/game-data/src/packed_png.rs:4,79 | "文件内不存张数"注记 | **保留+理由** | 格式事实（资源格式 doc 表 :83-94） |
| E3 | crates/game-data/src/packed_png.rs:123、tests/golden.rs:213、tests/png_golden.rs:121 | 计数断言（"{name} 子图张数"） | **保留+理由** | legacy 断言面=task-10 硬2 挂点（改 JSON 后断言 JSON 值==legacy 值） |
| E4 | shell/win/src/main.rs:62-157 | load_container(name, count) 16 组字面计数（生产装载） | **公式化**（读 image_count()/JSON） | 生产面 16 处重复表；改 JSON 后单一来源 |
| E5 | crates/game-core/src/scene.rs:28 | `CONTAINER_NAMES` 16 名序（构造器 a()：resourceNames（a.java:450-451）） | **保留+理由** | 顺序=数据语义（f_Image_arr2_00 槽序），非计数 |
| E6 | 测试复制面：crates/game-core/tests/slot_equip_scene.rs:65,158-214、mov_route_scene.rs:83,208-278、crates/game-oracle/tests/probe_floor2.rs:8,73-106、fixture_frames.rs:55,124-226,911-926、gameplay_floor1.rs:30,93-126,263-419、boot_replay.rs:23,34-129、preset_cases.rs:42,131-1287 | load_container 辅助+计数（约 120 行复制） | **保留+理由**（过渡） | 测试夹具复制；task-10 decode-equivalence 落地后统一由元数据驱动，届时销账 |
| E7 | crates/game-data/src/lib.rs:21 | "55 maplv + 55 sprite + 16 packed PNG"注记 | **保留+理由** | 文档注记（55=文件数非装载数，装载 1..=54 见审计 C 级出入记录） |

### F 类：UI 几何残余内联字面量（16/32 族）——**素材域/行为面混合**

| # | file:line | 内容 | 处置 | 理由/约束 |
|---|---|---|---|---|
| F1 | crates/game-core/src/scene.rs:3937-3939 | `alchemy_cursor<<4`（行步进 16px） | **公式化**（×行距常量） | 行距=UI 布局（数据域）；`<<4` 在此是**行距**非瓦片（P1 误中类） |
| F2 | crates/game-core/src/popup.rs:19,68,109-126 | RISE_Y5 表（含 16）+ `sx+16`/`16+y[i]` | **保留+理由** | 动画表=Java 字面锚（行为面）；非素材派生 |
| F3 | crates/game-core/src/paint.rs:761 | overlay `x+16,y+16` 内边距 | **保留+理由** | Java 布局字面（盒内边距） |
| F4 | crates/game-core/src/menu_family.rs:785-789,1344-1347 | 行首=盒顶+16（③-1 单源注） | **保留+理由** | 已单源化（row_band），字面=Java 布局锚 |
| F5 | crates/game-core/src/scene.rs:3013,3421,3775-3858 | `y+=16`/`bx+16` 等盒内步进 | **保留+理由** | Java 布局字面（像素坐标类，非素材派生量） |
| F6 | crates/game-core/src/layout.rs:52-331 其余 102 常量 | 已具名布局中枢 | **保留+理由** | 2026-10-15 卫生批产物（layout.rs:26"143 行逐处"）；改 JSON 须逐常量 legacy 断言，收益低——留数据域 |
| F7 | shell/android/src/input.rs（PadLayout::default_240x320） | 虚拟手柄几何 56/252/12/30/软键矩形 | **保留+理由** | Stage B **新增** overlay（原版无），非素材派生；后续可配置化（Stage B 面另立） |
| F8 | shell/android/src/native.rs | `CANVAS_W/H=240/320` | **公式化**（引 layout::SCREEN_W/H） | 与 game-core layout 同值双写，应单源 |
| F9 | crates/game-core/src/layout.rs:37-45 | `SCREEN_W/H=240/320`、CENTER | **保留+理由** | 已具名中枢（逻辑画布=数据域布局） |

### G 类：排除类（P3/P4/P5 命中的非尺寸语义）——保留+理由（整类）

| 类 | 代表锚（非全列） | 计数量级 | 理由 |
|---|---|---|---|
| mode/标签编号 | enums.rs:284（About=16）、scene.rs:3169-3172（gate 16/17） | P3 命中大半 | 状态机语义编号，非尺寸（key-code-contract/edges 文档面） |
| 实体类目码 | entity.rs:226,239-244,388-435（cat 1/2/8/16/32） | ~40 | 数据域类目分派（a.java 锚随行） |
| hex/位运算基数 | menu_family.rs:36-48（from_str_radix 16）、png.rs:185-233（`<<24\|<<16`） | ~15 | 算法常量（ARGB 打包/解析基数） |
| UTF-16 计数 | scene.rs:1370、script.rs:246 | ~5 | 文本语义（Java length=码元数） |
| 算法掩码/表 | game-platform/src/lib.rs:56（LCG `<<48`）、inflate.rs:139-227 | ~10 | 算法面（P1 的 `<<4` 误中即此类） |
| 存档字段长 | save.rs:15,55,107,139-141（128 字节 script_flags）、entity.rs:217-218（128 槽） | P5 全部 | **数据域**存档格式语义（loadFloorSave（a.java:9253）script_flags 读段（:9291-9296）），保留 |
| 战斗/脚本字面 | paint.rs:504、popup.rs 表、scene.rs:142,155 | ~30 | 行为面数值（非素材），Stage A 保真对象 |

## 3. 陷阱①专节：`(v&7)<<4` 的 8 = 图集列数索引语义

- **事实锚**：`crates/game-data/src/maplv.rs:126-136`（`id&7`/`id>>3`）；Java 真锚 =
  **paintTileLayer（a.java:7045-7046）** `int var5=(var18&7)<<4`、`var18=var18>>3<<4`
  （~~a.java:5002-5003~~【2026-10-18 更正：错位，该处=overlay 行/页计算】）；
  消费点 `crates/game-core/src/game_view.rs:179-180`。
  **机械校验**（防再漂）：`grep "& 7\) << 4|>> 3 << 4" reference/src/deobf/a.java`
  ⇒ 全文件唯一命中 7045-7046 即真锚（行号漂移时以命中行为准，方法名 paintTileLayer 定位）。
- **语义**：tile-id 是图集**格坐标编码**（低 3 位列、高位行）；`8`（=7+1、>>3 的模数）
  是**图集列数**，`16` 是瓦片像素。二者性质不同：16 是尺寸（可随素材变），
  8 是**索引基数**——列数变化会重排全部 tile-id→格 的映射（地图静默全花，
  trace 只在 L4 视觉层暴露）。
- **处置约束**：A1/A2 改读 JSON（图集列数/瓦片 px 元数据）时必须：
  ①断言 legacy 值 `8`（及 `16`）；②decode-equivalence（task-10）逐 tile-id
  比对 col/row 不变；③行锚注释保留（真锚 paintTileLayer（a.java:7045-7046））。
- **相关测试锚**：game_view.rs:261-266（`((v&7)<<4,(v>>3)<<4)==(32,0)`/`==128`）、
  maplv.rs:145-154（13×13 头 26×26）。改后这些断言=legacy 恒等判据，不许删。

## 4. 陷阱②专节：字体派生量（BUG-008 教训面）

- **事实锚**：`crates/game-core/src/render.rs:64-96`（度量三元组与 size 码，
  shim Font（reference/shim/src/javax/microedition/lcdui/Font.java:32-62）**双拷贝同步**）；
  BUG-008 根因注 render.rs:66-69
  （SMALL 高 10→14 更正：框体填充 2h+4 须为 16 整倍，对话框 48=16×3）。
- **派生链清单**（本表 D3-D11）：`textLineHeight=height+4`（构造器 a()（a.java:471））、
  `menu_line_height=height+6`（resetMenuLayout（a.java:4683））、
  `slotsPerPage=min(6,208/height)`（dispatchModeEntry（a.java:4354-4364））、
  dialog 块 `(line_h<<1)+12`、`pad=(lh-fh)>>1`。
- **处置约束**：glyph 改源（如换真字体渲染）时——
  ①度量三元组**逐项恒等**（6/14/11、8/14/11、12/20/16、BOLD+1）；
  ②派生式输出逐项恒等（行高/页行数/框高）；③shell 侧真字形 hook 只换像素
  不换度量（shell/win font_inject 同口径：布局仍用 SoftFont 格宽格高）。
  违反即重演 BUG-008（UI 半底片裁切）。

## 5. 数据域 vs 素材域归域裁定（复核硬6）

> 裁定人：android-shell，2026-10-18（lead 硬6 口径派发 + 本表逐项复核落位）。
> 口径（lead 原文）：maplv 与 sprite 帧宽表归**数据域**（Layout+存档语义消费，
> 变更=行为面，走数据门禁）；图形素材（容器图/tileset/字体点阵）归**素材域**。

| # | 项 | 归域 | 裁定理由 | 拍板 |
|---|---|---|---|---|
| R1 | maplv 三网格值（terrain/flip/walk，maplv.rs:110-124） | **数据域** | 存档/碰撞/Launch 语义直接消费（scene.rs rebuildWalkability）；改动=行为面 | android-shell 2026-10-18（复核 lead 硬6） |
| R2 | tile-id→图集映射的列数 8/瓦片 16（maplv.rs:126-136） | **数据域消费的素材元数据**（跨界件） | 素材属性（图集布局）被数据编码引用——按陷阱①管理：改 JSON 须断言 legacy | android-shell 2026-10-18 |
| R3 | sprite 帧宽表 typeFrameWidth（entity.rs:224） | **数据域** | Layout+格槽登记消费（entity.rs:348 `sprite_w>>5`），变更=碰撞/绘制行为 | android-shell 2026-10-18（复核 lead 硬6） |
| R4 | animOffsetTable 11 行（entity.rs:267） | **数据域** | 动画帧序=行为面（open_n 算式，差分 trace 可见） | android-shell 2026-10-18 |
| R5 | 容器图（ui/map/spt*/actor/face… 容器内容） | **素材域** | 纯图形像素；变更走视觉门禁（L4 白名单机制） | android-shell 2026-10-18 |
| R6 | tileset 图（map[0] 图集像素本体） | **素材域** | 同 R5；其**布局**（8 列 13 行）见 R2 跨界件 | android-shell 2026-10-18 |
| R7 | 字体点阵/glyph 源 | **素材域** | 像素面（visual）；但度量=行为面（R8） | android-shell 2026-10-18 |
| R8 | 字体度量（SoftFont 三元组+派生链） | **行为面（双拷同步面）** | shim Font.java 同构；变更=布局行为（BUG-008 面），**不入素材精细化叙事** | android-shell 2026-10-18 |
| R9 | UI 布局常量（layout.rs 102 项） | **数据域**（行为面） | Layout 消费，变更=差分行为；改 JSON 须逐常量 legacy 断言 | android-shell 2026-10-18 |
| R10 | 容器计数表 RESOURCE_IMAGE_COUNTS | **素材域**（清单属性） | 资产清单元数据（构造器 a()：resourceNames/resourceImageCounts（a.java:450-453）两数组）；断言族随 task-10 | android-shell 2026-10-18 |
| R11 | 存档字段长/实体槽容量（128 族） | **数据域** | 存档格式语义（save.rs 字段表），非素材 | android-shell 2026-10-18 |

**归域对处置的影响**：数据域件改 JSON 后**走数据门禁**（差分 L1-L3 回归必须全绿，
不许以"素材精细化"名义变更）；素材域件走视觉门禁（L4 白名单纪律）。
跨界件（R2）双门禁并用。

## 6. JSON 字段引用标注（不定义——硬5=task-8）

本表"改 JSON"处置涉及的字段面（**仅标注，schema 归 task-8 `asset_pipeline` 中间态**）：

| 引用处 | 需求字段（标注名，非定义） | 消费 |
|---|---|---|
| A1/A2/B2 | 图集布局：列数 legacy=8、行数 legacy=13、瓦片 px legacy=16 | maplv 格→像素映射 |
| B5/B6/B7/B8 | 切片几何：帧宽高、条带件尺寸、源偏移 | paint 切片 |
| C1 | typeFrameWidth[88] | entity 绘制/格槽 |
| C2 | animOffsetTable[11 行] | open_n 动画 |
| E1 | 容器名→子图计数（16 容器） | PackedPng 解析 |

改 JSON 统一前提：**legacy 值断言**（上表括注值）+ task-10 decode-equivalence
逐项比对通过；任一值漂=挂起（禁部分通过，同 task-8 恒等口径）。

## 7. 移交与联动

| 去向 | 内容 |
|---|---|
| task-8（硬1/硬5） | 上表"改 JSON"的 schema 落位；C1/C2/E1 字段进中间态 |
| task-10（硬2/硬4） | E3 计数断言族并入 decode-equivalence（"不许消失"）；B3/B4 摘要并入真相源哈希清单 |
| 渲染/呈现骨架任务 | B5-B8 切片几何 JSON 化随动（依赖 task-8 schema） |
| Stage B 后续 | F7 虚拟手柄几何可配置化（新增面，非本审计保真对象） |

## 8. 遗留假设与不确定性

| # | 假设/出入 | 级别 | 处置 |
|---|---|---|---|
| 1 | maplv0 是否死资源（55 文件 vs 装载 1..=54 的 1 层出入） | C（未考证，前科见 2026-10-17 审计 F-9） | 不下结论；E7 注记如实 |
| 2 | P3/P4 非尺寸命中逐条未铺表（按 §1 口径归排除类） | B（机械分类可核对） | 排除类代表锚可抽验；如需全列可重跑 recipe |
| 3 | "改 JSON"字段名以 task-8 schema 为准（本表标注名非定义） | —（硬5 边界） | §6 声明 |
| 4 | shell/android 侧 native.rs TODO 体未含硬编码（骨架注释内数字除外） | A（本次 grep 亲验） | — |
