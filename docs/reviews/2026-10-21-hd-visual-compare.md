# HD 素材探针·验证件③ 对照说明 —— scale=3 classic vs demo 图集双截图

> 2026-10-21。设计件：[hd-asset-probe-design.md](../superpowers/specs/2026-10-21-hd-asset-probe-design.md)
> v1.1/v1.2（R-6/R-7/R-8）；证据纪律按 [hd-probe-review-fidelity.md](../findings/reviews/2026-10-21-hd-probe-review-fidelity.md)
> C4：**机器判据 = A 级；截图人工判读 = C 级（本件不建闸，visual 闸 SKIP，判读≠验证）**。
>
> harness：`crates/game-core/tests/hd_visual.rs`（纯渲染测试，AGENTS §5 无头纪律：
> 零窗口/零用户桌面交互）。产出 PNG：`assets/hd-demo/compare/`（gitignore 区，R-7 弃件面）。

## 0. 对照锚（同场景同相机同帧，对照变量唯一）

| 项 | 值 |
|---|---|
| 场景段 | maplv5 相机 (-144,-16)（玩家 (248,126) 步 11 居中）、瓦片层 tile_off=20 |
| 可见窗口 | rows 1..17 × cols 9..24（272 格；transform=0 格 209、transform=1 格 63） |
| 倍率 | **scale=3**（必选：模糊根源=scale≥2 NN 放大，scale=1 对照=假阴性） |
| ①classic | `assets/raw/map` 帧 00（128×208，d=1） |
| ②HD | `assets/hd-demo/map_00_3x.png`（384×624，d=3，弃件②） |
| 唯一对照变量 | **仅瓦片层贴图源不同**；底色/相机/帧/HUD/精灵/文字 overlay 电池逐字节同配置 |

场景选取：该窗口是全 55 关最优段——**16 个 restyle 精绘帧全数在窗**（236 格）
+ 24 个 NN 垫底帧（36 格）= 混密度并存预演（量产"分批换素材"前瞻）。
restyle 帧类别（gen_hd_demo `_classify` 同启发式，C 级）：stone=id 1/22/17/24/15/38/39、
wall=id 10/46/47/2/30/9/45/49/44（本窗为宫殿内景，草/水类帧不在窗内）。

## 1. 机器判据（A 级——`cargo test -p game-core --test hd_visual` 5/5 绿（4 机器判据 + 1 标注产物））

| # | 判据（R-8） | 结果 |
|---|---|---|
| ① | 非素材区（HUD/精灵/文字）两图**逐像素恒等**（换素材零行为漂移） | 103,833 px 全恒等（HUD 38,880 / 面板+数字条 34,560 / 文字 2,502 / 精灵 27,891） |
| ① | classic/HD op 串**全集相等**（R-6：op 记逻辑尺寸） | 恒等 |
| ① | 差异像素只落瓦片层落笔面（对照变量唯一） | 素材区差异 211,668 px，区外零差异 |
| ② | 逐格对照（transform=0 可见格）：差异⟺restyle 帧 | restyle 格 181/181 有差异、NN 格 28/28 逐像素恒等 |
| ③ | demo 图集帧序/帧数 | 104 帧（8×13 格网）= NN 88（逐像素 == 源帧 NN×3，不串位）+ restyle 16（alpha 掩码 == 源帧 NN×3、槽位同构；帧序锚 = HAND_IDS 集合恰好 16） |
| ④ | d==s 直采样像素级（真图集 d=3@scale=3） | 1:1 行拷逐像素恒等（透明槽跳过语义同 set_pixel） |

判据②是"哪个 tile 变清晰"的 A 级证据；下方人工判读只是复述它。

## 2. 人工判读四问（C 级——画质/观感不作证据）

材料：`classic_scale3.png` / `hd_scale3.png` / `diff_scale3.png`（全帧 720×960）
+ 并排裁剪 `pair_torch_stone.png`（左 classic | 右 HD）、`pair_floor_mixed.png`（同）
+ **标注版（交付首选）** `annotated/`（见 §3）。

1. **高清 tile 边缘是否清晰**：是（仅绿框格）。火把金饰（wall 精绘帧）从 NN 糊团
   变硬边平面色+暗色描线；灰石回纹砖（stone 精绘帧）线稿锐利、色块平直；墙缘暗线
   成型（classic 侧为 16px 源图 ×3 的晕染糊边）。黄框格（白花窗/深色槽等 NN 垫底）
   两图目视恒等——正是判据②"NN 格零漂移"的视觉复述。
   > 【2026-10-21 更正】初版把"白花窗"误记为 stone 精绘帧；标注版证明白花窗是
   > NN 垫底帧（黄框、两图恒等），清晰度收益落点以绿框格为准（判据② A 级）。
2. **混密度交界表现**：交界无错位/无闪烁（逐格判据②已证 NN 格恒等、帧序不串位）；
   目视上精绘格（平色硬边）与 NN 垫底格（糊）相邻时画风跳变明显——这是弃件②
   "混合质量=量产分批换素材预演"的预期表现，非管线缺陷。标注版中交界=绿框与
   黄框相邻处，替换范围一眼可见。
3. **非素材区无变化**：是。HUD 带/面板/数字条/文字/精灵（金箱、白烟）两图目视
   逐像素一致——与机器判据①（103,833 px 恒等）互证。
4. **整体观感**：管线对（机器判据全绿 ⇒ R-8 "demo 画丑但管线对"分支成立）；
   demo 程序化精绘的画质上限（调色板压平偏平偏亮）如实呈现，画质不作证据、
   不据此判断管线。

## 3. 标注版对照图（lead 追加·用户诉求"把高清资源标注出来"）

产物 `assets/hd-demo/compare/annotated/`（测试 `hd_annotated_replacement_scope_products`
产出；**标注只画在产物副本**——机器判据仍对拍无标注原图，断言语义零改动；两图
框位置严格同格，断言钉）：

| 文件 | 说明 |
|---|---|
| `classic_scale3_annotated.png` / `hd_scale3_annotated.png` | 全帧 720×960 + 角落图例 |
| `pair_torch_stone_annotated.png` / `pair_floor_mixed_annotated.png` | **交付首选**：并排裁剪（左 classic \| 右 HD）+ 底部图例条 |

框语义（scale=3 每格 48px 物理、框 2px；绿/黄取亮色避开灰白石砖主色）：

| 框 | 含义 | 格数（maplv5 可见窗 255 格） |
|---|---|---|
| 绿框 | **重绘高清帧**（restyle 精绘，真正换画的资源） | 219 格（16 帧） |
| 黄框 | **仅放大帧**（NN 垫底：高清分辨率、像素内容=原图放大） | 36 格（24 帧） |
| 无框 | **未替换**（d=1 精灵/挂幡/宝箱/HUD/文字） | 非素材区 103,833 px + 底色区（v=0 格/顶底缘） |

图例（每张标注图角落/并排图底部）：三色框含义 + 场景/相机/scale
（MAPLV5 CAM-144,-16 SCALE3，LEFT=CLASSIC RIGHT=HD）。

## 4. 复现与边界

```bash
python3 tools/hd-demo/gen_hd_demo.py        # 生成弃件② 图集（若 assets/hd-demo 缺失）
cargo test -p game-core --test hd_visual    # 机器判据 + 双 PNG + 标注版产出
cargo test -p game-core                     # 全量（含 density_draw/task-1 既有判据）
```

- **弃件纪律（R-7）**：demo 图集只进 mod/验证面，永不入 `original/`/保真表/金样运行；
  PNG 产出在 gitignore 区，删除条件=量产素材接入再验收通过后。
- **未闭环（R-1/3-3 C1）**：d>1 且 d≠s 采样面待真素材复验；本对照 d==s 直采样
  不行使该分支。demo 画质上限不作证据（设计件 §3 风险）。
- **`--hd-assets`（shell，可选）**：实机试玩开关（`assets/hd-demo/map_00_3x.png`
  替换 tileset，其余素材不动）。按 R-8：**用户试玩反馈≠验证成果**，不得记为验证证据。
