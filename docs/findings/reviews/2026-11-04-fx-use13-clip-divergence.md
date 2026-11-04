# fx-use-13 TICK 341 帧差：clip/blit 边界指令级根因定位

> 角色：差分验证员（verifier）。产出=分析件，修复回 porter，**不改码**。
> 状态：**已钉死（前件中断于验证，本次补齐双侧锚 + 同拍对拍 + 像素复核三段）**。日期：2026-11-04。
> 分类模板见 §6；所有贴件均为可复跑命令的实输出（终态贴件纪律）。

## 0. 摘要

- **根因一句话**：Rust `paint_menu5` 的 `off_x<0`（= Java `n2<0`）臂把 `typeFrameWidth[t]`（本拍 = 35）
  当 icon 裁剪宽传给 `draw_image_clipped`，而 Java 该臂字面量是 **32**（= 上一行
  `paintMiniFrame(x,y,32,32)` 的框宽）⇒ 裁剪面右移 3px（x64–66）越出框右边框
  `drawLine(63,…)`，源图 col34–36 泄漏，TICK 341 帧差 5px。
- **修复一句话**：`crates/game-core/src/scene_menu.rs:611` 的 `fw` 改字面量 `32`（逐字对齐
  `a.java:2546`），else 臂不动；随后摘 `fixture_frames.rs:741` 的 `#[ignore]` 解红回归。

## 1. 背景与已知定位（前件成果并入，本次全部复核通过）

- 差分用例：**fx-use-13**（EnemyInfo 面板），**TICK 341 首分歧**（mode=5）；1..340 帧全过、
  341 首红——测试逐帧断言在 341 panic，即 341 是首分歧点（A 级）。
  复跑红证（本次实贴）：

  ```
  assertion `left == right` failed: TICK 341 帧不符（mode=5）
    left:  "88c8a0ba13b08e3d1cd3cedd758e1512"   ← Rust
    right: "ae1c885f4db1fcff1e5b866f91bcffa9"   ← Java（= 夹具 fx-use-13.frames.txt 0341 行）
  ```

- 首分歧像素区 = **(64–66, 171–179) 共 5px**（本次逐像素复算逐点吻合，见 §4）；
  样本 (64,171)：Java = (41,50,49) 面板底 vs Rust = (200,192,176) icon 亮像素（Rust 多画）。
- 调用层 = EnemyInfo 行内 **icon/clip 簇**（`paint case 5` 行绘制，a.java:2538–2549 /
  scene_menu.rs:603–614）：
  - 框：`fillRect(33,150,30,31)` + `drawLine(32,151,32,180)`（左）+ `drawLine(63,151,63,180)`（右）+ `drawLine(33,181,62,181)`（底）= **32×32 mini-frame，x∈[32,63], y∈[150,181]**
  - icon：`setClip(32,150,32|35,31)` + `drawImage(105x43,30,150,0)`
  - 同行后续：`setClip(68,154,10,13)/drawImage(30x13,58,154)`（攻图标）、`setClip(68,171,10,10)/drawImage(30x13,68,169)`（血图标）、`fillRect(84,171,46,11)`（血框）——**全部 x≥58，不覆盖溢出列 64–66**。
- 分类：**移植错**（clip/blit 边界语义，Rust 于 Java 裁剪面外泄 5px）。非参考 bug、非平台差异。

## 2. 双侧锚实文（可复跑贴件）

### 2.1 Java 侧：case-5 的 `n2<0` 分支 clip 宽 = 字面量 32

`reference/seed/a.java`（前件引用的 `n2` 命名即此，case-5 行内）：

```
$ awk 'NR>=905&&NR<=916{printf "%d: %s\n",NR,$0}' reference/seed/a.java
905:                         super.d(n22, n4 += n + 19 + 4, 32, 32);      ← paintMiniFrame 32×32
906:                         if (by > 40) {
907:                             n2 = 32 - ((a)object).h[by] >> 1;         ← h[] = typeFrameWidth
908:                             n = image.getHeight();
909:                             if (n > 31) {
910:                                 n = 31;
911:                             }
912:                             if (n2 < 0) {
913:                                 super.a(image, n22, n4 + 31 - n, -n2, 0, 32, n);   ← ★ clip 宽字面量 32
914:                             } else {
915:                                 super.a(image, n22 + n2, n4 + 31 - n, 0, 0, ((a)object).h[by], n);  ← else 臂才传表值
916:                             }
```

`reference/src/deobf/a.java`（同段，case-5 = `a.java:2498`）：

```
$ awk 'NR>=2538&&NR<=2549{printf "%d: %s\n",NR,$0}' reference/src/deobf/a.java
2538:                   var29.paintMiniFrame(var64, var105, 32, 32);                       ← 32×32 框
2539:                   if (var144 > 40) {
2540:                      var170 = 32 - var29.typeFrameWidth[var144] >> 1;                 ← n2（var170）
2541:                      if ((var177 = var126.getHeight()) > 31) {
2542:                         var177 = 31;
2543:                      }
2544:
2545:                      if (var170 < 0) {
2546:                         var29.drawImageClipped(var126, var64, var105 + 31 - var177, -var170, 0, 32, var177);   ← ★ 32
2547:                      } else {
2548:                         var29.drawImageClipped(var126, var64 + var170, var105 + 31 - var177, 0, 0, var29.typeFrameWidth[var144], var177);  ← 表值
2549:                      }
```

`drawImageClipped` 实现（clip 语义 = 裁剪面 (x,y,w,h) 半开区间）：

```
$ awk 'NR>=4551&&NR<=4555{printf "%d: %s\n",NR,$0}' reference/src/deobf/a.java
4551:    private void drawImageClipped(Image var1, int var2, int var3, int var4, int var5, int var6, int var7) {
4552:       this.f_Graphics_00.setClip(var2, var3, var6, var7);
4553:       this.f_Graphics_00.drawImage(var1, var2 - var4, var3 - var5, 0);
4554:       this.f_Graphics_00.setClip(0, 0, 240, 320);
4555:    }
```

### 2.2 Rust 侧：同逻辑臂传 `fw`（=35）

```
$ grep -n "off_x < 0" -B7 -A4 crates/game-core/src/scene_menu.rs
603:                 crate::menu_family::paint_mini_frame(g, x, y, 32, 32);
604:                 if t > 40 {
605:                     let img = self.images.entity.get(t).cloned().flatten();
606:                     if let Some(img) = img {
607:                         let fw = self.type_frame_width_for(t as i32);   ← fw 来源（表值，非源图宽）
608:                         let off_x = (32 - fw) >> 1;                      ← = Java var170/n2
609:                         let draw_h = (img.height).min(31);
610:                         if off_x < 0 {
611:                             draw_image_clipped(g, &img, x, y + 31 - draw_h, -off_x, 0, fw, draw_h);   ← ★ 传 fw（Java 此处是 32）
612:                         } else {
613:                             draw_image_clipped(g, &img, x + off_x, y + 31 - draw_h, 0, 0, fw, draw_h); ← 与 Java a.java:2548 一致，不动
614:                         }
```

配套定义（Rust 侧链路完整）：

- `paint::draw_image_clipped`（`crates/game-core/src/paint.rs:43-47`）= `set_clip(x,y,w,h)` → `draw_image(x-sx, y-sy)` → `reset_clip()`，与 Java `a.java:4551-4555` 同构。
- `type_frame_width_for`（`crates/game-core/src/scene.rs:4629-4631`）= `width_table[t]`，注释即
  「typeFrameWidth[type]（type_width_px 夹具的 i32 视图）」（`scene.rs:4628`）。
- OPS 流缝：`crates/game-core/src/render.rs:355`（shim `Graphics.flushOps` 对应物）。

**fw 是什么的宽**（任务问）：
- **不是** 30×13 那个属性图标（那是同行后续 `ui[7]` 的 `drawImage(30x13,58,154)`，另一条 op）；
- **不是** 源图宽（本拍源图 = 105×43）；
- **是** `typeFrameWidth[t]` 表值 = **35**（`a.java:589` 的 `byte[]` 表 / 夹具
  `crates/game-core/tests/fixtures/type_width_px.txt`），语义 = 该类型精灵的**类型绘制宽度**，
  在 `drawImageClipped` 里充当**源矩形宽**（取源图左起 `fw` 列）。表来源 Rust 没用错——
  错的是**分支用法**：把 else 臂的 `fw` 照抄进了 `n2<0` 臂。

## 3. 同拍逐指令对拍表（A 级：TICK 0341 双侧 ops 全量）

取证（两侧均为 TICK 0341 同一拍，前件因 Java stride 采样缺 341 拍 ops，本次用
`-Doracle.dumpStride=1` 取证模式补齐；Java 帧 sha 复现夹具值 `ae1c885f…` 确认同源）：

```bash
# Rust 侧（fixture_frames 的 FRAME_OUT 像素取证缝，测试断言前落盘）
FRAME_OUT=/tmp/fx13 FRAME_TICKS=341 \
  cargo test -p game-oracle --test fixture_frames fx_use_13 -- --ignored --nocapture
# → /tmp/fx13/fx-use-13-0341.rgba + fx-use-13-0341.ops.txt（panic 实贴见 §1）

# Java 侧（frames=all + dumpStride=1 取证跑，独立输出目录，不触碰夹具源 trace）
java -Djava.awt.headless=true -Dfile.encoding=UTF-8 -Dline.separator=$'\n' \
  -cp analysis/frames-fixture/classes:analysis/frames-fixture/variantA:original/囧囧西游-大闹天宫.jar \
  -Doracle.out=/tmp/fx13/java341b -Doracle.max=480 \
  -Doracle.script=$PWD/crates/game-oracle/tests/fixtures/fx-use-13.txt \
  -Doracle.preset=$PWD/crates/game-oracle/tests/fixtures/fx-use-13.preset \
  -Doracle.label=A-fx-use-13 -Doracle.watchdog=36000 \
  -Doracle.frames=all -Doracle.dumpStride=1 oracle.host.Runner
# → /tmp/fx13/java341b/frame-0341.png（FRAME sha=ae1c885f4db1fcff1e5b866f91bcffa9 = 夹具 0341 行）
```

对拍结果（脚本：两侧各取 `OPS count=180` 全量 op 序列逐条比）：

```
java ops: 180 | rust ops: 180 | diff count: 1
 idx110:
   java: setClip(32,150,32,31)
   rust: setClip(32,150,35,31)
```

**同拍 180 条 op，唯一参数分歧 = 1 条**。关键段（两侧逐字，仅第 110 条不同）：

| # | 语义 | Java (TICK 0341 A-run) | Rust (fx-use-13 0341) | 分歧 |
|---|------|------------------------|------------------------|------|
| 105/106 | 框底色 | `fillRect(33,150,30,31,#494d3e)` | 同 | — |
| 107/108 | 框左右边 | `drawLine(32,151,32,180)` / `drawLine(63,151,63,180,#191d1a)` | 同 | — |
| 109 | 框底边 | `drawLine(33,181,62,181,#191d1a)` | 同 | — |
| **110** | **icon 裁剪面** | **`setClip(32,150,32,31)`** | **`setClip(32,150,35,31)`** | **w: 32 vs 35（+3px）** |
| 111 | icon 绘制 | `drawImage(105x43,30,150,0)` | 同 | — |
| 112 | 裁剪复位 | `setClip(0,0,240,320)` | 同 | — |
| 113+ | 血/攻图标、属性框 | `setClip(68,154,10,13)`…`setClip(68,171,10,10)`…`fillRect(84,171,46,11)`… | 同 | — |

- 首个（也是唯一）参数分歧 = `setClip` 宽度；**off-by-N 的 N = 3**（35−32）。
- 语义裁决：Java `setClip` 为**替换式**裁剪（非交叠，MIDP 语义，两侧实现同构）；
  `drawImageClipped` 裁剪边界为**半开区间** `[x, x+w) × [y, y+h)`——两侧同口径，无端点歧义。
- 归属：分歧 op 位于 icon 绘制（`drawImage(105x43,30,150,0)`）之前，直接决定其可见面 ⇒
  该 op 即根因 op，其后 69 条 op 全部一致。

## 4. 像素级复核：35−32=3 宽差 vs 实测 5px（任务 2，算清）

复算命令与结果（Java `frame-0341.png` vs Rust `fx-use-13-0341.rgba`，240×320 全图逐点）：

```
diffs: 5
 (64,171) java=(41,50,49,255) rust=(200,192,176,255)
 (66,173) java=(41,50,49,255) rust=(230,185,30,255)
 (66,177) java=(41,50,49,255) rust=(230,185,30,255)
 (66,178) java=(41,50,49,255) rust=(230,185,30,255)
 (66,179) java=(41,50,49,255) rust=(230,185,30,255)
按列: x=64→1px, x=65→0px, x=66→4px   按行: y=171,173,177,178,179 各1px
```

**账目（3 是列数，5 是像素数——不存在“差 2px 未解释”）**：

1. **溢出面（列）**：半开区间裁剪下 Java clip `[32,64)`、Rust clip `[32,67)` ⇒ 溢出列恰
   **3 列 = 64,65,66**（= 35−32，无端点含/不含歧义）。行向 clip h=31 ⇒ 候选行 `[150,181)`。
2. **实际差异（像素）= 5**：溢出列里只有「源图该处**不透明**且颜色 ≠ Java 底色」的格子才成差。
   源图绘于 (30,150)，溢出列对应源列 34/35/36、源行 21–29：
   - x=64（源 col34）：1 格（y171）
   - x=65（源 col35）：**0 格**（该列源图透明 ⇒ Rust 不落墨，露出与 Java 相同的底色）
   - x=66（源 col36）：4 格（y173/177/178/179）
   ⇒ 3 列 × 9 行 = 27 个候选格中仅 5 格有可见内容；**单列可贡献多像素**（col66 占 4），
   所以 5 与 3 不是同一单位，5−3=2 不是“缺口”。
3. **Java 侧溢出列恒为底色**：5 点 Java 全 = (41,50,49) = `#293231` = 先画的
   `fillRect(27,133,186,54,#293231)`（op 序在 icon 之前）；Java 的 clip 挡住了 icon 落到 64–66。
4. **相邻 drawImage 不叠加抵消/不修复**：同拍后续所有 op 起点 x≥68
   （血图标 clip `[68,78)`、`fillRect(84,171,46,11)`）或止于 x≤62
   （`fillRect(33,150,30,31)` = `[33,63)`），**无一覆盖 64–66**；框右边框
   `drawLine(63,…)` 画在 icon 之前，且两侧 clip 都含 x=63 ⇒ 两侧同样被 icon 覆盖，
   故 x=63 两侧一致、无差（复算 x=63 零差异，吻合）。
5. **行向解释**：候选行 150–180，实际差 171–179 ⇒ 源图 col34–36 在源行 0–20 与 30 无内容
   （或恰为底色），仅 21–29 有内容。

## 5. 语义锚：Java 字面量 32 是什么 / Rust fw=35 从哪来（任务 3）

- **32 = 32×32 mini-frame 的框宽**：紧邻上一行 `paintMiniFrame(var64, var105, 32, 32)`
  （`a.java:2538` = `scene_menu.rs:603`，同拍同参）刚画出的属性图标小框；框的物理边界由
  `drawLine(63,151,63,180)`（右）钉死在 x=63。`n2=(32−fw)>>1` 是 icon 在框内的**居中偏移**；
  **`n2<0 ⇔ fw>32` ⇔ icon 比框宽**，此时 Java 按**框宽 32** 裁——语义 = “宽出框的 icon 裁进框内”，
  与 else 臂（fw≤32 时按类型绘制宽裁，避免画出源图多余列）互补。等价式：`clip = min(fw, 32)`。
- **fw=35 的来历**：`typeFrameWidth[t]` 表值（`a.java:589` `byte[]`；Rust 夹具
  `type_width_px.txt`）。本拍 `t>40` 且表值 =35 的项**唯一 = idx47**（≥41 区间内 35 仅此一项）
  ⇒ 显示类型 t=47（B 级推断，由 fw=35 + 分支条件 + 表唯一性闭合；两侧状态同拍同表）。
  Rust **没有用错表**，是把 else 臂的表值**照抄进**了 `n2<0` 臂（`scene_menu.rs:611` vs `:613`）。
- （C 级假设，禁入结论）源图 105×43 中 105 = 35×3，疑似 3 帧横排精灵条，Rust 溢出列
  col35/36 已越过本帧 nominal 区 `[0,35)` 取到邻帧像素——与实测 (230,185,30) 金黄色块相容，
  但未做源图逐列取证，仅记假设。

## 6. 根因定性（diff-case）

```markdown
## diff-case fx-use-13-t341
- 输入序列: reference/oracle 侧夹具场景 crates/game-oracle/tests/fixtures/fx-use-13.txt
            （boot→槽载入 floor2(3,5) 持道具13→300 -6 面板→340 -6 选中→380 -6 确认），480 tick
- 现象: TICK 341（mode=5，EnemyInfo 行视图首拍）帧 sha 双侧不符；
        唯一分歧 op = setClip(32,150,32,31)[Java] vs setClip(32,150,35,31)[Rust]（idx110/180）；
        像素差 5 点 = (64,171),(66,173),(66,177),(66,178),(66,179)，bbox x64–66 × y171–179
- 分类: 移植错（clip/blit 边界语义；Rust 于 Java 裁剪面外泄 3 列）
- 处理: 回 porter（改 crates/game-core/src/scene_menu.rs:611，见 §7）；不涉及白名单/阈值
- 门禁: cargo test -p game-oracle --test fixture_frames fx_use_13 -- --ignored --nocapture
        → FAILED（TICK 341 帧不符，双 sha 见 §1，即本红证据本身）
```

- Rust 侧错误实现段：`crates/game-core/src/scene_menu.rs:610-611`（`paint_menu5` 的 `off_x<0` 臂）。
- Java 对应语义锚：`reference/src/deobf/a.java:2545-2546`（字面量 32）/ `reference/seed/a.java:912-913`。
- 非同型对照（已排除）：`scene_menu.rs:850` 对话头像臂也传 `fw`，但 Java 对应段
  `a.java:2844-2849` else 臂**同样传 `typeFrameWidth[dialogSpeakerType]`**（`a.java:2848`）、
  type72 臂传 `32,45`（`a.java:2852`）——两侧一致，**不是**本 bug 的复制，勿顺手改。

## 7. 修复建议（不改码，回 porter）

- **改哪行（精确锚）**：`crates/game-core/src/scene_menu.rs:611`
  - 现：`draw_image_clipped(g, &img, x, y + 31 - draw_h, -off_x, 0, fw, draw_h);`
  - 建：`…, -off_x, 0, 32, draw_h);`（字面量对齐 `a.java:2546`；`:613` else 臂**不动**，
    它与 `a.java:2548` 已一致）
- **语义依据**：`off_x<0 ⇔ fw>32` ⇒ Java 按 32×32 mini-frame 框宽裁（`a.java:2538` 框 +
  `a.java:2546` 字面量 32，A 级）；Rust 传 `fw` 使裁剪面越出框右边框 `drawLine(63,…)`。
  等价写法 `fw.min(32)` 亦可（两臂合并时 else 臂 fw≤32 恒等），但**逐字对齐字面量更可证**。
- **回归面**：
  1. 解 `crates/game-oracle/tests/fixture_frames.rs:741` 的 `#[ignore = "RED: EnemyInfo 面板
     TICK 341 帧差…"]` ⇒ `cargo test -p game-oracle --test fixture_frames` 预期 **44 用例全绿**。
     可证性：同拍 180 条 op 仅此 1 条分歧 ⇒ 修后 ops 逐字一致 ⇒ 同一绘制管线 ⇒ 帧逐字一致；
     341 之后各行重复同型绘制，一并覆盖。
  2. bug 触发面 = `t>40 且 typeFrameWidth[t]>32` 的类型共 19 个：
     46(34) 47(35) 49(33) 53(36) 54(38) 56(34) 60(33) 63(36) 65(38) 66(33) 67(42) 69(78)
     70(38) 71(36) 73(39) 74(59) 75(36) 78(37) 87(36)——本夹具覆盖 t=47；其余类型建议后续
     夹具/抽样补臂（登记为完备性缺口，非本笔阻塞）。
  3. 金样与门禁：按批5 注记（`docs/findings/reviews/2026-11-04-p1a2-batch5-review.md` §2）
     修复后 `fx-use-13.state` 金样按预期红再固化；跑 `python3 gates/cli.py trace`（L1–L3）与
     `python3 gates/cli.py visual`（L4），白名单零变更。
  4. 明确**不**改：`scene_menu.rs:613`、`scene_menu.rs:850`、`paint.rs:43-47`、任何夹具/门禁/基线。

## 8. 证据分级与遗留假设

- **已证实（A 级）**：双侧 sed/grep 实文（§2）；同拍 180 vs 180 唯一 1 条分歧
  `setClip(32,150,32,31)`/`(32,150,35,31)`（§3）；全图逐点 5 像素及其坐标/颜色（§4）；
  Java 取证跑 FRAME sha = 夹具 0341 行 `ae1c885f…`；红证双 sha 复跑（§1）；
  `type_width_px.txt` 表值（≥41 且 =35 仅 idx47；>32 共 19 项）。
- **B 级（可复现静态推理）**：本拍显示类型 t=47；调用层归属 `paint case 5` 行绘制；
  clip 半开区间语义（由 `drawImageClipped` 实现 + 两侧 op 同构推出）。
- **假设（C 级，禁入结论）**：源图 105 = 35×3 的“3 帧横排精灵条”解释；x=65 零差异 =
  源图该列透明（无法区分“透明”与“恰好等于底色”，二者像素等价）。
- **遗留（不阻塞）**：其余 18 个 `fw>32` 类型的 EnemyInfo 行无夹具臂（完备性缺口，登记待补）。
- **只读预检（本笔仅新增本文档，未动任何代码/夹具/门禁）**，实贴：

  ```
  $ python3 gates/cli.py status
  [✓ PASS] … reference-regression / diff-trace / rust-tests / naming-ledger / docs-links …
  [- SKIP] visual-diff — 视觉比对器未就绪（P4 启动后启用）
  == 汇总 ==
  PASS=13 FAIL=0 SKIP=1 TOLERATED=0
  ```

  SKIP=1 为已知恒 SKIP 项（visual-diff 未就绪）⇒ 修复后的 L4 面在该门禁启用前由
  fx-use-13 逐帧夹具（480 帧）代偿；commit 前由 porter 跑 `python3 gates/cli.py check`。
