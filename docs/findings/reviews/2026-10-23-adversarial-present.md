# 对抗复核 A：Android 呈现层修复有效性（stride / 信箱）

- 角色：remake-verifier 纪律（对抗复核员 A）
- 日期：2026-10-23
- 复核对象：`88f4c6f`（stride 跳行+信箱 fit_rect+选策略，重点）、`16416cd`（SIGSEGV 判空三修）、原生入口（§0 勘误）
- 范围：只读攻击性复核（含破坏性验证：改坏→钉红→逐字节还原），不改产品码
- **总判：需修复（红 1 / 黄 3 / 绿 4）**——详见 §6

## 0. 复核对象勘误

- 任务给的 `9897357` **仓库中不存在**（`git cat-file -t 9897357` → 无此对象）。
  "原生入口"实为 **`a45af6d`**（feat(android): T5 原生入口+事件环+呈现——ANativeActivity_onCreate 根治）。
- 复核链按 `88f4c6f` / `16416cd` / `a45af6d` 执行；`a45af6d` 入口装配不在重点攻击面，
  仅在攻击点 4/5 的 ANativeWindow_* 调用点全量普查中覆盖。
- 基线实跑（复核开始）：`cargo test -p game-android` → **67/67 绿**（与 88f4c6f 声明一致）。

## 1. 攻击点 1 —— stride 修复真伪：**绿**（破坏性验证证明钉是真钉）

- **机制核**：`shell/android/src/native.rs:218`——
  `off = ((y0+y) as usize) * (stride as usize) + (x0+x) as usize) * 4`：
  行距域（stride_px）与显示/拟合域（buf_w，fit_rect 用）显式分离，逐行跳行；
  `native.rs:202` `stride = stride_px.max(buf_w)`（行距不小于宽的防御）；
  present 侧缓冲长度按 `stride*bh*4` 造（`native.rs:428-429`）——行距域全链一致。
- **残留"按 width 连写"路径：无**。全仓 grep `memcpy|copy_from_slice|copy_within|ptr::copy`：
  窗口 blit 无连续拷贝（`render.rs:322` 的 copy_from_slice 属 game-core `Image.getRGB`
  的 scanlength 语义，非窗口 blit；`native.rs:203` `for b in dst.iter_mut()` 仅信箱清零，
  与行距无关）。
- **钉测真实性**：`shell/android/tests/native_parse.rs:172-183`——240 宽/256 行距/2 行假缓冲：
  - :182 断言**行 1 首像素落 `stride*4`（=1024）** ✓（任务问的正是这条）；
  - :183 断言 `width*4`（=960）处必须是 0（行 0 尾隙不许被连写侵入）——双向断言
    （该在的在 + 不该在的不在），单向断言骗不过。
- **破坏性验证（改坏→看钉红不红）**：把 :218 的 `stride` 改成 `buf_w`（=按 width 连写）：

  ```bash
  python3 - <<'EOF'
  p='shell/android/src/native.rs'; s=open(p).read()
  old="let off = (((y0 + y) as usize) * (stride as usize) + (x0 + x) as usize) * 4;"
  new="let off = (((y0 + y) as usize) * (buf_w as usize) + (x0 + x) as usize) * 4;"
  assert s.count(old)==1; open(p,'w').write(s.replace(old,new))
  EOF
  cargo test -p game-android --test native_parse blit_row_pitch
  ```

  实测输出：`blit_row_pitch_uses_stride_not_width ... FAILED`，
  panic 于 `native_parse.rs:183` `left: [17, 34, 51, 255] / right: [0, 0, 0, 0]`
  （行 1 被写进 width*4 尾隙）——**钉红 ✓，钉是真的**。改坏即红 ⇒ 修复有效性可证伪。
- 还原：`cp /tmp/native.rs.bak shell/android/src/native.rs`，`git status` 无 native.rs 修改 ✓。

## 2. 攻击点 2 —— 信箱数学：**红**（纯函数数学绿；运行时集成被残留调用击穿）

### 2.1 手工验算（绿）

`fit_rect(2670, 1200, 576, 576, 2)`（`native.rs:170-186`）：
w=h=576×2=1152 ≤ (2670,1200) ⇒ 不触发缩进；
x0=(2670−1152)/2=1518/2=**759** ✓；y0=(1200−1152)/2=48/2=**24** ✓
⇒ **(759, 24, 1152, 1152) 正确**。

四档矩阵钉（`native_parse.rs:189-203`）逐档手算复核全对：
A `(2670-832)/2=919,(1200-832)/2=184` ⇒ (919,184,832,832) ✓；
C `(2670-960)/2=855,(1200-960)/2=120` ⇒ (855,120,960,960) ✓；
D `(2670-1184)/2=743,(1200-1184)/2=8` ⇒ (743,8,1184,1184) ✓；
越界缩进档 `fit_rect(2670,1200,1184,1184,2)`：2368>1200 ⇒ k=1200/2368 ⇒
w=h=1200，`(2670-1200)/2=735` ⇒ (735,0,1200,1200) ✓。

### 2.2 边界（无 panic 级缺陷）

- **除零**：fit_rect 缩进为 f64 除法（0/正=0.0，浮点不炸）；w=0 时 blit 行循环 0 次，
  整数除法 `x*cw/w`（blit 内 NN 取源）不可达 ⇒ 无除零 panic。
- **负尺寸**：全程 u32，buf 宽高 `.max(0) as u32`（`native.rs:427`），`.saturating_sub`/`.max(1)` 齐备。
- **NN 越界（x+1 经典坑）：不存在**——最近邻是点采样 `argb.get(sy*cw+sx)`（.get 越界=跳过），
  `sx=x*cw/w` 因 x<w 恒 <cw，**无邻域 (x+1) 采样**，无越读。

### 2.3 红项：present() 残留 `setBuffersGeometry(240,320)` —— Bug2 运行时被击穿

- `native.rs:410-415`：present() **每帧**在 lock 前调
  `ANativeWindow_setBuffersGeometry(window, CANVAS_W=240, CANVAS_H=320, RGBA_8888)`。
  注释自称"格式协商幂等（INIT_WINDOW 已协商则**同参**无害）"——但 88f4c6f 已把
  ensure_surface 改为 **(0,0)**（`native.rs:377`），两处**不同参**；present 在 INIT_WINDOW
  之后执行 ⇒ **(240,320) 每帧覆盖 (0,0)**。
- 后果（推演，A 级锚=两处调用参数）：
  1. 实机 lock 缓冲恒 240×320 ⇒ 信箱钉的输入 (2670,1200) **永不出现**；
     fit_rect 实际收到 (240,320,576,576,2) ⇒ 缩进 (0,40,240,240)——
     **(759,24,1152,1152) 在实机不可达**（钉只测纯函数，集成不成立）；
  2. 240×320 buffer 被 SurfaceFlinger 拉满屏（2670×1200 ⇒ x11.125 / y3.75 非等比）
     ⇒ 方画幅压成 ≈2.97:1——**Bug2"压扁"症状原样回归**。88f4c6f 的 Bug2 根治
     被该残留调用逐帧覆盖。
- 残留证据（自证）：present() 文档注释 `native.rs:399` 仍写"setBuffersGeometry(240×320)
  后 1:1 直拷路径"——与新信箱设计矛盾，系旧文案未随 88f4c6f 清理。
- 复现（静态铁证，无需实机）：

  ```bash
  grep -n "ANativeWindow_setBuffersGeometry" -A4 shell/android/src/native.rs
  # :377 → (window, 0, 0, ...)   [ensure_surface，88f4c6f 修复]
  # :410 → (window, CANVAS_W as i32, CANVAS_H as i32, ...)  [present，残留=击穿点]
  ```

- 修复方向（供 lead 裁决，本复核不改产品码）：删除 :410-415 调用或改 (0,0) 与 :377 同参，
  并更正 :399 注释；补集成钉（模拟 buf=2670×1200 时 present 链路的几何断言）。

### 2.4 黄（子项）

1. **浮点截断 1px 欠缩**：缩进 `(((w as f64)*k) as u32)`（`native.rs:181-182`）是截断不是
   取整——分数倍缩进可能 1199.999…→1199，信箱 1px 不对称。本批钉值恰好过线
   （1200/2368×2368=1200.0000000000002）属"碰运气绿"，建议 `round()`。
2. **信箱清零=RGBA(0,0,0,0)**（`native.rs:203-205`）——alpha=0 在 RGBA8888 窗口是**透明条**
   （可能透出桌面/壁纸）而非黑条；窗口不透明性由系统合成决定，需实机确认观感。
3. **map_touch_xy(grid=0) ⇒ NaN**（w=0 浮点除）：不 panic；运行时 grid≥13
   （pick_resolution clamp 下限）不可达，纯函数边界留档。

## 3. 攻击点 3 —— RGBA8888 通道序：**绿**（字节序推演+破坏性验证双证）

- **字节序推演**：canvas 像素=0xAARRGGBB（`crates/game-core/src/render.rs:255` "非预乘 ARGB
  位模式（0xAARRGGBB）"，u32 负值=同一模式另一解释）；blit 取段
  a=px>>24, r=px>>16, g=px>>8, b=px&0xff（`native.rs:211-216`）；写序
  dst[0]=R, dst[1]=G, dst[2]=B, dst[3]=A（`native.rs:219-222`）。
  `WINDOW_FORMAT_RGBA_8888(=1)` 的缓冲内存字节序=[R,G,B,A] ⇒ **ARGB 字→RGBA 字节
  swizzle 正确**（无 swizzle 缺失、无 BGRA 颠倒）。
- **钉**：`native_parse.rs:104-113`——0xFF112233→[0x11,0x22,0x33,0xFF]、
  0x80445566→[0x44,0x55,0x66,0x80]（alpha 保真）。
- **破坏性验证**：dst[0] R→B 互换 ⇒ `blit_argb_to_rgba8888_byte_order ... FAILED`
  （另 2 枚 blit 钉同红）⇒ 钉真，写错通道必被抓。
- 非缺陷注记：`render.rs:328-330` hash_stream 输出 [A,R,G,B] 大端——帧哈希域
  （TickHooks），与显示域无混用（grep 全仓唯一消费点）。
- 黄注：canvas 非预乘 alpha 直写 Android 预乘合成域——半透明像素可能轻微色边；
  游戏主面 alpha=0xFF 影响面小，实机观感待验（同 §2.4-2 归口实机确认）。

## 4. 攻击点 4 —— SIGSEGV 判空完备性：**绿**（6/6 调用点全前置判空）

全量清单（`grep -rn "ANativeWindow_" shell/ --include=*.rs`，恰好 6 个调用点）：

| # | 调用点 | 行 | 前置判空 |
|---|---|---|---|
| 1 | setBuffersGeometry(0,0) | native.rs:377 | :369 `if window.is_null() { return false }` |
| 2 | ANativeWindow_getWidth | native.rs:383 | 同 :369（ensure_surface 顶部一次判空覆盖） |
| 3 | ANativeWindow_getHeight | native.rs:384 | 同 :369 |
| 4 | setBuffersGeometry(240,320)（残留，见 §2.3 红项） | native.rs:410 | :403 `if window.is_null() { return false }` |
| 5 | ANativeWindow_lock | native.rs:423 | 同 :403 |
| 6 | ANativeWindow_unlockAndPost | native.rs:434 | 同 :403 |

- `(*app).window` 其余流转：:597 传入 ensure_surface（函数内自判空）；
  :766 判空 → :770 使用（同线程，无 TOCTOU）。
- 判空钉实跑绿：`present_null_window_noop`（native_parse.rs:152）/
  `surface_negotiate_null_window_noop`（:162）——NULL ⇒ No-op 返回 false，不触 FFI。
- 无第 7 个裸调用点、无绕过判空的 FFI 直调。**判空完备 ✓**。

## 5. 攻击点 5 —— TERM_WINDOW 竞态：**绿**（防护三道+单线程不变量）

- **防护在哪行**：
  1. `native.rs:403`——present() 入口判空（NULL ⇒ No-op 返回 false）；
  2. `native.rs:766`——主循环每帧前 `if !(*app).window.is_null()`（TERM_WINDOW 后跳过
     渲染呈现）；
  3. **present 中途不可能被置 NULL**——glue 实锚（NDK 27.1.12297006
     `android_native_app_glue.c`）：cmd 分派 `:196-198`（pre_exec→onAppCmd→post_exec）
     全在 android_main 线程执行；window 仅两个写点且都同线程：
     **`:105`** `window = pendingWindow`（INIT_WINDOW，回调前）、
     **`:145`** `window = NULL`（TERM_WINDOW，post_exec=回调后）⇒ 与 present()
     （同线程 android_main 环）互斥，blit 期间无并发置 NULL。:766 读→:770 用同线程连贯。
  （与 16416cd commit 引用的 :105/:145 实测核对一致。）
- 复现命令：

  ```bash
  G=~/Android/Sdk/ndk/27.1.12297006/sources/android/native_app_glue/android_native_app_glue.c
  grep -n "window = \|onAppCmd\|post_exec_cmd" "$G"   # :105 / :145 / :196-198
  ```

- 黄注：present 途中无显式快照/锁（blit 全程裸用 window 指针）——当前由单线程不变量
  兜底；若日后 cmd 分派挪线程或引入并发 ANativeWindow_release，需在 :423 lock 后持
  局部快照重审。

## 6. 总判定：**需修复**（红 1 / 黄 3 / 绿 4）

### 红项清单（必须清零才可交付）

1. **[红] present() 残留 `setBuffersGeometry(240,320)`**（native.rs:410-415）：
   每帧覆盖 ensure_surface 的 (0,0) ⇒ 实机缓冲恒 240×320 ⇒ 信箱 (759,24,1152,1152)
   不可达 + 宽高比压扁（Bug2 症状回归）。88f4c6f 的 Bug2 修复**未真修对**（集成层被击穿）。
   修复=删该调用或改 (0,0)，更正 :399 注释，补集成钉。

### 黄项清单

1. [黄] fit_rect 缩进 `as u32` 截断（:181-182）——分数倍可欠 1px，建议 round()；
2. [黄] 信箱清零 alpha=0=透明条非黑条（:203-205），实机观感待验；
3. [黄] map_touch_xy grid=0 ⇒ NaN（不可达，边界留档）。
   （另黄注：present 中途无快照/锁，依赖单线程不变量，重构需重审。）

### 绿项清单

1. [绿] stride 跳行——width/stride 分离逐行跳行，无残留连写路径；
   钉（:182 行 1 落 stride*4 + :183 尾隙不侵）破坏性验证变红 ⇒ **真修对**；
2. [绿] RGBA8888 通道序 swizzle 正确（推演+破坏性验证）；
3. [绿] SIGSEGV 判空完备（6/6 调用点，判空钉实跑）；
4. [绿] TERM_WINDOW 防护（:403+:766+glue :105/:145 同线程不变量）。

### 攻击点归档

| 攻击点 | 判定 | 一句话 |
|---|---|---|
| 1 stride 真伪 | 绿 | 跳行机制真、钉可证伪（改坏即红）、无连写残留 |
| 2 信箱数学 | **红** | 数学/钉全对，但 present 残留 (240,320) 使修复实机失效 |
| 3 通道序 | 绿 | ARGB→RGBA swizzle 正确，钉真 |
| 4 判空完备 | 绿 | 6/6 调用点前置判空 |
| 5 TERM_WINDOW | 绿 | 403+766+glue 单线程不变量（:105/:145/:196-198） |

## 附录：破坏性验证留痕（3 组 mutate→test→restore，产品码零净改动）

| # | 破坏 | 实测红 | 证明 |
|---|---|---|---|
| 1 | blit 行距 stride→width | 1 FAILED（native_parse.rs:183 left:[17,34,51,255]） | stride 钉真 |
| 2 | fit_rect 居中 /2→/3 | 4 FAILED（letterbox/overflow/landscape/touch_inverse） | 信箱钉真 |
| 3 | blit dst[0]=R→B | 3 FAILED（byte_order/row_pitch/nn_scale） | 通道钉真 |

- 还原后复跑：`cargo test -p game-android` → **67 passed / 0 failed**；
  `git status`：native.rs 无修改（逐字节还原），仅 Cargo.lock 依赖同步
  （cc/game-core/game-data 入 game-android 列表，构建副作用）+ 未跟踪复核文档。
- 基线命令：`cargo test -p game-android`（67/67）；单钉复跑：
  `cargo test -p game-android --test native_parse <钉名>`。
