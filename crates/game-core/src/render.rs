//! 参考版像素模型的 Rust 复刻：`javax.microedition.lcdui` shim（Graphics/Font/Image）
//! 的逐字节确定性软件光栅器。
//!
//! # 为什么逐行复刻 shim 而不是"实现 MIDP 规范"
//!
//! 差分裁判是 trace 里的 `FRAME sha=`（TickHooks.afterPaint：`getRGB` 出的 240×320
//! ARGB 流按每像素大端 4 字节喂 SHA-256）。要使 Rust 侧产出相同 sha，像素模型必须
//! 与 Java shim **逐位一致**——包括 shim 的自选自由度（确定性 hash 字形、字体度量
//! 常量、`translate` 只移动 clip、arc 判定的 double 阈值）。这些自由度两端同源即
//! 不影响等价性判定（设备级视觉保真是 L4/P4 的事）。
//!
//! # 证据（A 级）
//!
//! - 混合/图元/字形：`reference/shim/src/javax/microedition/lcdui/Graphics.java`
//! - 字体度量：`reference/shim/src/javax/microedition/lcdui/Font.java`
//! - 帧哈希：`reference/oracle/src/oracle/host/TickHooks.java:48-67`
//!
//! # double 一致性说明
//!
//! `fill_triangle`/`fill_arc` 内的乘除与 ceil/floor 是 IEEE-754 精确操作，Java/Rust
//! 逐位一致。`atan2` 有 libm ULP 风险，但其结果只喂给 `in_arc`，而游戏唯一调用
//! `fillArc(x,y,32,32,0,360)`（a.java:6667）走 `arcDeg >= 360 ⇒ true` 短路，返回值
//! 被丢弃，ULP 差异不影响任何像素。
//!
//! # Java int 位语义
//!
//! 画布像素存 `u32`（0xAARRGGBB 位模式，Java int 的负值只是同一模式的另一解释）；
//! 所有中间运算用 `i32`/`u32` 按位复制 Java 行为。混合除法的操作数恒非负，安全。

/// anchor 常量（`Graphics.java:31-37`）。
pub mod anchor {
    pub const HCENTER: i32 = 1;
    pub const VCENTER: i32 = 2;
    pub const LEFT: i32 = 4;
    pub const RIGHT: i32 = 8;
    pub const TOP: i32 = 16;
    pub const BOTTOM: i32 = 32;
    pub const BASELINE: i32 = 64;
}

/// Nokia DirectGraphics 变换码（`com/nokia/mid/ui/DirectGraphics.java:11-18`）。
pub mod transform {
    pub const NONE: i32 = 0;
    pub const MIRROR_ROT180: i32 = 1;
    pub const MIRROR: i32 = 2;
    pub const ROT180: i32 = 3;
    pub const MIRROR_ROT270: i32 = 4;
    pub const ROT90: i32 = 5;
    pub const ROT270: i32 = 6;
    pub const MIRROR_ROT90: i32 = 7;
}

/// Font 尺寸/风格常量（`Font.java:16-26`）。
pub mod font_size {
    pub const SMALL: i32 = 8;
    pub const MEDIUM: i32 = 16;
    pub const LARGE: i32 = 32;
}
pub mod font_style {
    pub const PLAIN: i32 = 0;
    pub const BOLD: i32 = 1;
}

/// 字体度量（shim 同源常量，`Font.java:47-62`）。
///
/// 与 shim 同源：SMALL=6/14/11，MEDIUM=8/14/11，LARGE=12/20/16，BOLD 步进 +1。
/// SMALL 高 14（2026-10-15 更正，原 10）：16px 切片平铺的 UI 需要框体填充
/// 2h+4 为 16 整倍（对话框 48=16×3），h=10 会裁半底片（BUG-008 根因）。
/// **与 reference/shim Font.java 双拷贝同步**（pattern_dual-copy-algorithms-sync-fix）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoftFont {
    /// ASCII 字符步进宽（像素）。
    pub ascii_width: i32,
    /// 行高（像素）。
    pub height: i32,
    /// 基线距顶部（像素）。
    pub baseline: i32,
    pub style: i32,
    /// MIDP size 码（SMALL=8/MEDIUM=16/LARGE=32）——op 文本 `setFont(size=…)`
    /// 与 shim `Graphics.java:116` 同构（记 size 常量，非由字高反推）。
    pub size: i32,
}

impl SoftFont {
    /// `Font.getFont(face, style, size)`。face 在 shim 中不影响度量（系统/等宽同度量）。
    pub fn get_font(_face: i32, style: i32, size: i32) -> SoftFont {
        let (mut aw, h, bl) = match size {
            font_size::SMALL => (6, 14, 11),
            font_size::LARGE => (12, 20, 16),
            _ => (8, 14, 11),
        };
        if style & font_style::BOLD != 0 {
            aw += 1;
        }
        SoftFont { ascii_width: aw, height: h, baseline: bl, style, size }
    }

    /// `Font.isWide`：CJK/全角区域按 2 倍步进（`Font.java:99-106`）。
    fn is_wide(ch: u16) -> bool {
        (0x2E80..=0x9FFF).contains(&ch)
            || (0xA960..=0xA97F).contains(&ch)
            || (0xAC00..=0xD7FF).contains(&ch)
            || (0xF900..=0xFAFF).contains(&ch)
            || (0xFF00..=0xFF60).contains(&ch)
            || (0xFFE0..=0xFFE6).contains(&ch)
    }

    /// `charWidth`。
    pub fn char_width(&self, ch: u16) -> i32 {
        if Self::is_wide(ch) { self.ascii_width * 2 } else { self.ascii_width }
    }

    /// `substringWidth(str, offset, len)`：逐字符累加（`Font.java:87-96`）。
    pub fn substring_width(&self, s: &[u16], offset: usize, len: usize) -> i32 {
        let mut w = 0;
        for &ch in &s[offset..offset + len] {
            w += self.char_width(ch);
        }
        w
    }

    /// `stringWidth`。
    pub fn string_width(&self, s: &[u16]) -> i32 {
        self.substring_width(s, 0, s.len())
    }
}

/// 字体度量抽象（task-25 ①，font-metrics-dependency-survey §3）：几何派生/渲染
/// 步进/逻辑 wrap 三层消费统一入口。**classic=SoftFont 现值恒等**（零偏离断言）
/// ；modern=TTF 实测变宽（壳侧注入，core 零依赖——GlyphHook 同模式）。
pub trait FontMetrics {
    /// 字形高（classic：SMALL/MEDIUM=14、LARGE=20）。
    fn height(&self) -> i32;
    /// 字符步进宽（变宽实测通道；classic=定宽 + is_wide 2×）。
    fn char_advance(&self, ch: u16) -> i32;
    /// 文本测量 (w, h)——换行/居中统一入口（h=line_height）。
    fn measure(&self, s: &[u16]) -> (i32, i32);
    /// 行距（classic=height+4，a.java:471 锚；BUG-008 教训=度量全链联动）。
    fn line_height(&self) -> i32;
    /// 基线距顶（classic：SMALL/MEDIUM=11、LARGE=16）。
    fn baseline(&self) -> i32;

    /// 派生便捷：charWidth 同构（classic=char_advance 同值——68 处消费面零形状迁移）。
    fn char_width(&self, ch: u16) -> i32 {
        self.char_advance(ch)
    }

    /// 派生便捷：substringWidth 同构（逐字符步进累加，Font.java:87-96）。
    fn substring_width(&self, s: &[u16], offset: usize, len: usize) -> i32 {
        let mut w = 0;
        for &ch in &s[offset..offset + len] {
            w += self.char_advance(ch);
        }
        w
    }

    /// 派生便捷：stringWidth 同构。
    fn string_width(&self, s: &[u16]) -> i32 {
        self.substring_width(s, 0, s.len())
    }
}

impl FontMetrics for SoftFont {
    fn height(&self) -> i32 {
        self.height
    }
    fn char_advance(&self, ch: u16) -> i32 {
        self.char_width(ch)
    }
    fn measure(&self, s: &[u16]) -> (i32, i32) {
        (self.string_width(s), self.height + 4)
    }
    fn line_height(&self) -> i32 {
        self.height + 4
    }
    fn baseline(&self) -> i32 {
        self.baseline
    }
}

/// 变宽步进测量 hook（task-25 ③）：壳侧 TTF 实测注入（GlyphHook 同模式）。
/// `(size, ch) -> 步进宽 px`；未注入 ⇒ ModernMetrics 回退 classic 定宽步进。
pub type AdvanceHook = fn(i32, u16) -> i32;

static mut ADVANCE_HOOK: Option<AdvanceHook> = None;

/// 注入变宽步进实测通道（壳侧 TTF 度量；core 零依赖红线保持）。
pub fn set_advance_hook(h: Option<AdvanceHook>) {
    unsafe { ADVANCE_HOOK = h }
}

fn advance_hook() -> Option<AdvanceHook> {
    unsafe { ADVANCE_HOOK }
}

/// modern 度量（task-25 ③）：**变宽**步进=TTF 实测（hook 注入）；height/baseline
/// 族=classic 同值（度量高族不动——BUG-008 教训：度量高改=全链联动，帧布局面
/// 分级稳定）。未注入 hook ⇒ 回退 classic 定宽（is_wide 2×）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModernMetrics {
    /// MIDP size 码（SMALL=8/MEDIUM=16/LARGE=32）——步进 hook 实测参数。
    pub size: i32,
    pub style: i32,
}

impl FontMetrics for ModernMetrics {
    fn height(&self) -> i32 {
        SoftFont::get_font(0, self.style, self.size).height
    }
    fn char_advance(&self, ch: u16) -> i32 {
        match advance_hook() {
            Some(h) => h(self.size, ch),
            None => SoftFont::get_font(0, self.style, self.size).char_width(ch),
        }
    }
    fn measure(&self, s: &[u16]) -> (i32, i32) {
        let mut w = 0;
        for &ch in s {
            w += self.char_advance(ch);
        }
        (w, self.height() + 4)
    }
    fn line_height(&self) -> i32 {
        self.height() + 4
    }
    fn baseline(&self) -> i32 {
        SoftFont::get_font(0, self.style, self.size).baseline
    }
}

/// classic 度量包装（task-25 ①）：SoftFont 现值——**恒等断言零偏离**（TDD 钉：
/// 6/14/11、8/14/11、12/20/16、BOLD aw+1、is_wide 2×、line_height=height+4）。
pub struct ClassicMetrics {
    pub font: SoftFont,
}

impl FontMetrics for ClassicMetrics {
    fn height(&self) -> i32 {
        self.font.height
    }
    fn char_advance(&self, ch: u16) -> i32 {
        self.font.char_width(ch)
    }
    fn measure(&self, s: &[u16]) -> (i32, i32) {
        (self.font.string_width(s), self.font.height + 4)
    }
    fn line_height(&self) -> i32 {
        self.font.height + 4
    }
    fn baseline(&self) -> i32 {
        self.font.baseline
    }
}

/// 画布/离屏图：width×height 的非预乘 ARGB 位模式（0xAARRGGBB）。
///
/// 对应 shim `Image`（`int[] argb`）。初始全 0（Java `new int[..]` 语义）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgbImage {
    pub width: i32,
    pub height: i32,
    pub argb: Vec<u32>,
    /// 【HD 素材探针·R-5 载体】素材像素密度（A 档）：**像素尺寸 = 逻辑尺寸 × density**。
    /// classic 默认 1（= 现行为逐字节恒等）。绘图/anchor/op 消费面一律走
    /// [`Self::logical_width`] / [`Self::logical_height`]（=w/d），像素缓冲索引才用
    /// `width/height`（物理域）。离屏/合成图（[`Self::create`] / [`Self::from_rgb`] /
    /// `dim_image` 派生）强制 d=1：画布本身是物理像素网格，不携带密度。
    pub density: i32,
}

impl ArgbImage {
    /// `Image.createImage(w, h)`。离屏画布 ⇒ 强制 density=1（R-5）。
    pub fn create(width: i32, height: i32) -> ArgbImage {
        ArgbImage { width, height, argb: vec![0; (width * height) as usize], density: 1 }
    }

    /// 资源解码产物 → 可绘制图（位模式直通，无转换）。classic 素材 d=1；
    /// HD 素材经 [`Self::with_density`] 声明密度。
    pub fn from_decoded_png(png: game_data::DecodedPng) -> ArgbImage {
        ArgbImage { width: png.width, height: png.height, argb: png.argb, density: 1 }
    }

    /// `Image.createRGBImage(rgb, w, h, processAlpha)`（`Image.java:59-66`）。
    /// 合成图 ⇒ 强制 density=1（R-5）。
    pub fn from_rgb(rgb: &[u32], width: i32, height: i32, process_alpha: bool) -> ArgbImage {
        let argb = rgb
            .iter()
            .map(|&v| if process_alpha { v } else { v | 0xFF000000 })
            .collect();
        ArgbImage { width, height, argb, density: 1 }
    }

    /// 【R-5】声明素材像素密度：像素尺寸 = 逻辑尺寸 × density。
    /// `width%d==0` / `height%d==0` 断言（不许静默截断）；d≥1。
    pub fn with_density(mut self, density: i32) -> ArgbImage {
        assert!(density >= 1, "density 必须 ≥1（实际 {density}）");
        assert!(
            self.width % density == 0 && self.height % density == 0,
            "width/height 必须被 density 整除（{}x{}，d={density}）",
            self.width,
            self.height
        );
        self.density = density;
        self
    }

    /// 【R-2/R-5】逻辑宽 = 像素宽/density（消费面统一出口；d=1 ⇒ =width 恒等）。
    pub fn logical_width(&self) -> i32 {
        self.width / self.density
    }

    /// 【R-2/R-5】逻辑高 = 像素高/density。
    pub fn logical_height(&self) -> i32 {
        self.height / self.density
    }

    /// `Image.getRGB(rgbData, offset, scanlength, x, y, w, h)`（`Image.java:80-86`）。
    pub fn get_rgb(&self, out: &mut [u32], offset: usize, scanlength: usize, x: i32, y: i32, w: i32, h: i32) {
        for row in 0..h {
            let src = ((y + row) * self.width + x) as usize;
            let dst = offset + row as usize * scanlength;
            out[dst..dst + w as usize].copy_from_slice(&self.argb[src..src + w as usize]);
        }
    }

    /// TickHooks.afterPaint 的帧哈希输入流：逐像素大端 4 字节（`TickHooks.java:53-59`）。
    pub fn hash_stream(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.argb.len() * 4);
        for &p in &self.argb {
            out.extend_from_slice(&[(p >> 24) as u8, (p >> 16) as u8, (p >> 8) as u8, p as u8]);
        }
        out
    }
}

/// 像素级 `Graphics` 复刻。目标画布通过借用注入。
///
/// `record_ops` 为 `Some` 时把每个绘制调用追加为 shim `op(...)` 同格式字符串
/// （trace 里 OPS 流的对拍载体，`Graphics.java:64-73`）；`None` 时零开销。
pub struct SoftGraphics<'a> {
    target: &'a mut ArgbImage,
    color: u32,
    font: Option<SoftFont>,
    clip_x: i32,
    clip_y: i32,
    clip_w: i32,
    clip_h: i32,
    /// 【task-13 显示分辨率画布】光栅倍率：逻辑坐标进、**物理坐标光栅**（原语
    /// ×倍率精确换算；像素素材最近邻块写）。classic/逻辑面 =1 ⇒ 逐像素恒等。
    scale: i32,
    /// 【task-14 origin 平移层】逻辑 x 平移（开场/菜单经典构图整体居中）：
    /// 物理 = (逻辑 + origin_x) × scale——**平移先于缩放**（可交换性判据锚）。
    /// classic =0 ⇒ 恒等；ops 串记录调用方**原始逻辑坐标**（oracle ops 字节恒等）。
    origin_x: i32,
    /// OPS 流（shim `Graphics.flushOps` 的对应物；不带 "OPS count=" 头）。
    pub ops: Vec<String>,
}

impl<'a> SoftGraphics<'a> {
    /// `new Graphics(target)`：clip = 整幅画布，字体默认 MEDIUM。
    pub fn new(target: &'a mut ArgbImage) -> SoftGraphics<'a> {
        let (w, h) = (target.width, target.height);
        SoftGraphics {
            target,
            color: 0,
            font: Some(SoftFont::get_font(0, font_style::PLAIN, font_size::MEDIUM)),
            clip_x: 0,
            clip_y: 0,
            clip_w: w,
            clip_h: h,
            scale: 1,
            origin_x: 0,
            ops: Vec::new(),
        }
    }

    /// 【task-13】光栅倍率（画布=逻辑×scale；壳层建物理背板后设此）。
    /// 默认 1（classic/逻辑面恒等）。
    pub fn set_scale(&mut self, s: i32) {
        self.scale = s.max(1);
    }

    /// 【task-14】逻辑 x 平移（0=恒等）。
    pub fn set_origin(&mut self, x: i32) {
        self.origin_x = x;
    }

    /// 当前逻辑 x 平移（全幅层豁免存取用）。
    pub fn origin(&self) -> i32 {
        self.origin_x
    }

    /// 画布**逻辑**尺寸（=物理背板/scale；全幅层铺界用——禁 SCREEN 常量）。
    pub fn canvas_logical(&self) -> (i32, i32) {
        (self.target.width / self.scale, self.target.height / self.scale)
    }

    /// 逻辑 → 物理（平移先于缩放）。
    fn phys_x(&self, x: i32) -> i32 {
        (x + self.origin_x) * self.scale
    }

    fn phys_y(&self, y: i32) -> i32 {
        y * self.scale
    }

    /// 最近邻块写（像素素材/占位字形用）：1 逻辑像素 = scale×scale 物理块。
    fn set_pixel_block(&mut self, x: i32, y: i32, argb: u32) {
        let s = self.scale;
        for dy in 0..s {
            for dx in 0..s {
                self.set_pixel(x + dx, y + dy, argb);
            }
        }
    }

    fn op(&mut self, s: String) {
        self.ops.push(s);
    }

    /// `getColor`。
    pub fn get_color(&self) -> u32 {
        self.color
    }

    /// `setColor(int rgb)`：`rgb & 0xFFFFFF`。
    pub fn set_color(&mut self, rgb: u32) {
        self.color = rgb & 0xFFFFFF;
        self.op(format!("setColor(#{:06x})", self.color));
    }

    /// `setColor(r, g, b)`。
    pub fn set_color_rgb(&mut self, r: i32, g: i32, b: i32) {
        self.set_color((((r << 16) | (g << 8) | b) as u32) & 0xFFFFFF);
    }

    /// `setFont`。`None` 对应 shim 传 null（后续文字绘制会 panic，模拟 NPE）。
    pub fn set_font(&mut self, font: Option<SoftFont>) {
        self.op(format!("setFont(size={})", font.map(|f| f.size).unwrap_or(-1)));
        self.font = font;
    }

    fn font(&self) -> SoftFont {
        self.font.expect("NullPointerException（shim：font==null 时文字度量崩溃）")
    }

    /// `setClip`：与画布取绝对交集并**替换**当前剪裁区（`Graphics.java:120-131`）。
    /// 逻辑坐标进（origin 平移 + scale 换算后与物理画布求交）。
    pub fn set_clip(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.op(format!("setClip({x},{y},{w},{h})"));
        let (x, y, w, h) = (self.phys_x(x), self.phys_y(y), w * self.scale, h * self.scale);
        let nx = x.max(0);
        let ny = y.max(0);
        let nx2 = (x + w).min(self.target.width);
        let ny2 = (y + h).min(self.target.height);
        self.clip_x = nx;
        self.clip_y = ny;
        self.clip_w = (nx2 - nx).max(0);
        self.clip_h = (ny2 - ny).max(0);
    }

    /// 复位剪裁区=**整幅物理画布**（"取景后复位"类绘制用；禁 SCREEN 常量）。
    /// classic 画布=240×320 ⇒ op 文本与常量复位逐字同（oracle ops 对拍零偏离）。
    pub fn reset_clip(&mut self) {
        let (w, h) = (self.target.width, self.target.height);
        self.op(format!("setClip(0,0,{w},{h})"));
        self.clip_x = 0;
        self.clip_y = 0;
        self.clip_w = w;
        self.clip_h = h;
    }

    /// `clipRect`：与当前剪裁区取交（`Graphics.java:133-143`）。
    pub fn clip_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let (x, y, w, h) = (self.phys_x(x), self.phys_y(y), w * self.scale, h * self.scale);
        let x2 = (self.clip_x + self.clip_w).min(x + w);
        let y2 = (self.clip_y + self.clip_h).min(y + h);
        let nx = self.clip_x.max(x);
        let ny = self.clip_y.max(y);
        self.clip_x = nx;
        self.clip_y = ny;
        self.clip_w = (x2 - nx).max(0);
        self.clip_h = (y2 - ny).max(0);
        self.op(format!("clipRect({x},{y},{w},{h})"));
    }

    /// `translate`：**只移动 clip**（shim 特有，`Graphics.java:145-149`；坐标原点不动）。
    pub fn translate(&mut self, x: i32, y: i32) {
        self.clip_x += x;
        self.clip_y += y;
        self.op(format!("translate({x},{y})"));
    }

    // ---------------- 基础图元 ----------------

    /// `setPixel`：clip 判断 → alpha 0 跳过 / 255 直写 / 其他 src-over（`Graphics.java:161-181`）。
    fn set_pixel(&mut self, x: i32, y: i32, argb: u32) {
        if x < self.clip_x || y < self.clip_y || x >= self.clip_x + self.clip_w || y >= self.clip_y + self.clip_h {
            return;
        }
        let a = (argb >> 24) & 0xFF;
        let idx = (y * self.target.width + x) as usize;
        if a == 0 {
            return;
        }
        if a == 255 {
            self.target.argb[idx] = argb;
            return;
        }
        let dst = self.target.argb[idx];
        let ia = 255 - a;
        let blend = |s: u32, d: u32| (s * a + d * ia) / 255;
        let r = blend((argb >> 16) & 0xFF, (dst >> 16) & 0xFF);
        let g = blend((argb >> 8) & 0xFF, (dst >> 8) & 0xFF);
        let b = blend(argb & 0xFF, dst & 0xFF);
        let da = (((dst >> 24) & 0xFF) * ia) / 255 + a;
        self.target.argb[idx] = (da << 24) | (r << 16) | (g << 8) | b;
    }

    fn opaque_color(&self) -> u32 {
        0xFF000000 | self.color
    }

    /// `fillRect`（`Graphics.java:187-195`）。逻辑坐标进；scale 后=精确物理矩形。
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.op(format!("fillRect({x},{y},{w},{h},#{:06x})", self.color));
        let c = self.opaque_color();
        let (x, y) = (self.phys_x(x), self.phys_y(y));
        let (w, h) = (w * self.scale, h * self.scale);
        for yy in y..y + h {
            for xx in x..x + w {
                self.set_pixel(xx, yy, c);
            }
        }
    }

    /// `drawRect`：四条 `drawLine`（`Graphics.java:197-203`）。
    /// 【task-13】经 line_raw（物理）防双倍换算；op 流=drawRect+4×drawLine 逐字同。
    pub fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.op(format!("drawRect({x},{y},{w},{h},#{:06x})", self.color));
        let c = self.opaque_color();
        let (px, py) = (self.phys_x(x), self.phys_y(y));
        let (pw, ph) = (w * self.scale, h * self.scale);
        for (x1, y1, x2, y2) in [
            (px, py, px + pw - 1, py),
            (px, py + ph - 1, px + pw - 1, py + ph - 1),
            (px, py, px, py + ph - 1),
            (px + pw - 1, py, px + pw - 1, py + ph - 1),
        ] {
            self.op(format!("drawLine({x1},{y1},{x2},{y2},#{:06x})", self.color));
            self.line_raw(x1, y1, x2, y2, c);
        }
    }

    /// `drawLine`：Bresenham（`Graphics.java:205-227`）。逻辑坐标进。
    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.op(format!("drawLine({x1},{y1},{x2},{y2},#{:06x})", self.color));
        let c = self.opaque_color();
        let (x1, y1) = (self.phys_x(x1), self.phys_y(y1));
        let (x2, y2) = (self.phys_x(x2), self.phys_y(y2));
        self.line_raw(x1, y1, x2, y2, c);
    }

    /// 物理坐标 Bresenham（pub 面已换算；drawRect 内部复用防双倍平移/缩放）。
    fn line_raw(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: u32) {
        let (dx, dy) = ((x2 - x1).abs(), (y2 - y1).abs());
        let (sx, sy) = (if x1 < x2 { 1 } else { -1 }, if y1 < y2 { 1 } else { -1 });
        let mut err = dx - dy;
        let (mut x, mut y) = (x1, y1);
        loop {
            self.set_pixel(x, y, c);
            if x == x2 && y == y2 {
                break;
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// `fillTriangle`：顶点按 y 稳定冒泡排序 + double 边缘插值（`Graphics.java:229-259`）。
    /// 逻辑坐标进；scale 后顶点=物理。
    pub fn fill_triangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32) {
        self.op(format!(
            "fillTriangle({x1},{y1},{x2},{y2},{x3},{y3},#{:06x})",
            self.color
        ));
        let c = self.opaque_color();
        let (x1, y1, x2, y2, x3, y3) = (
            self.phys_x(x1),
            self.phys_y(y1),
            self.phys_x(x2),
            self.phys_y(y2),
            self.phys_x(x3),
            self.phys_y(y3),
        );
        let (mut x1, mut y1, mut x2, mut y2, mut x3, mut y3) = (x1, y1, x2, y2, x3, y3);
        // 顶点排序（确定性）：与 shim 相同的三次比较交换（x/y 同步换）
        if y1 > y2 {
            std::mem::swap(&mut y1, &mut y2);
            std::mem::swap(&mut x1, &mut x2);
        }
        if y2 > y3 {
            std::mem::swap(&mut y2, &mut y3);
            std::mem::swap(&mut x2, &mut x3);
        }
        if y1 > y2 {
            std::mem::swap(&mut y1, &mut y2);
            std::mem::swap(&mut x1, &mut x2);
        }
        for y in y1..=y3 {
            let (xl, xr) = if y < y2 {
                (edge(x1, y1, x3, y3, y), edge(x1, y1, x2, y2, y))
            } else {
                (edge(x1, y1, x3, y3, y), edge(x2, y2, x3, y3, y))
            };
            let lo = xl.min(xr).ceil() as i32;
            let hi = xl.max(xr).floor() as i32;
            for x in lo..=hi {
                self.set_pixel(x, y, c);
            }
        }
    }

    /// `fillArc`（`Graphics.java:261-286`）。游戏唯一调用为全圆 0..360（a.java:6667）。
    pub fn fill_arc(&mut self, x: i32, y: i32, w: i32, h: i32, start_angle: i32, arc_angle: i32) {
        self.op(format!(
            "fillArc({x},{y},{w},{h},{start_angle},{arc_angle},#{:06x})",
            self.color
        ));
        let c = self.opaque_color();
        let cx = x as f64 + w as f64 / 2.0;
        let cy = y as f64 + h as f64 / 2.0;
        let rx = w as f64 / 2.0;
        let ry = h as f64 / 2.0;
        let mut norm_start = start_angle % 360;
        if norm_start < 0 {
            norm_start += 360;
        }
        for yy in y..y + h {
            for xx in x..x + w {
                let dx = xx as f64 + 0.5 - cx;
                let dy = yy as f64 + 0.5 - cy;
                if rx <= 0.0 || ry <= 0.0 {
                    continue;
                }
                let nx = dx / rx;
                let ny = dy / ry;
                if nx * nx + ny * ny > 1.0 {
                    continue;
                }
                if in_arc(f64::atan2(ny, nx), norm_start, arc_angle) {
                    self.set_pixel(xx, yy, c);
                }
            }
        }
    }

    /// `drawArc`（`Graphics.java:288-311`）。参考版未调用，保真复刻以对齐 shim 全集。
    pub fn draw_arc(&mut self, x: i32, y: i32, w: i32, h: i32, start_angle: i32, arc_angle: i32) {
        self.op(format!(
            "drawArc({x},{y},{w},{h},{start_angle},{arc_angle},#{:06x})",
            self.color
        ));
        let c = self.opaque_color();
        let cx = x as f64 + w as f64 / 2.0;
        let cy = y as f64 + h as f64 / 2.0;
        let rx = w as f64 / 2.0;
        let ry = h as f64 / 2.0;
        let mut norm_start = start_angle % 360;
        if norm_start < 0 {
            norm_start += 360;
        }
        for yy in y..y + h {
            for xx in x..x + w {
                let dx = xx as f64 + 0.5 - cx;
                let dy = yy as f64 + 0.5 - cy;
                let nx = if rx > 0.0 { dx / rx } else { 0.0 };
                let ny = if ry > 0.0 { dy / ry } else { 0.0 };
                let r2 = nx * nx + ny * ny;
                if !(0.72..=1.28).contains(&r2) {
                    continue;
                }
                if in_arc(f64::atan2(ny, nx), norm_start, arc_angle) {
                    self.set_pixel(xx, yy, c);
                }
            }
        }
    }

    // ---------------- 图像 ----------------

    /// `drawImage(img, x, y, anchor)`（`Graphics.java:336-340`）。
    /// 逻辑坐标进；像素素材**最近邻 ×scale**（硬边保像素味——task-13 ④）。
    /// 【R-2 换算收敛】op 记录与 anchor 一律用**逻辑尺寸**（=w/d，与 density 无关；
    /// R-6 ops 流隔离）。d=1 ⇒ 逻辑=像素，逐字节恒等。
    /// 【问题1】cover 撑满裁切（NN）：把 img 等比放大到盖满 dst_w×dst_h（居中、
    /// 溢出裁切）——Modern 标题铺满用（用户裁定：贴图满幅）。classic 路径零调用。
    pub fn draw_image_cover(&mut self, img: &ArgbImage, dst_w: i32, dst_h: i32) {
        let (iw, ih) = (img.width.max(1), img.height.max(1));
        let (dw, dh) = (dst_w.max(1), dst_h.max(1));
        let k = (dw as f64 / iw as f64).max(dh as f64 / ih as f64);
        let sw = (iw as f64 * k).ceil();
        let sh = (ih as f64 * k).ceil();
        let ox = ((dw as f64 - sw) / 2.0).floor();
        let oy = ((dh as f64 - sh) / 2.0).floor();
        for y in 0..dh {
            for x in 0..dw {
                let sx = (((x as f64 - ox) * iw as f64 / sw).floor() as i32).clamp(0, iw - 1);
                let sy = (((y as f64 - oy) * ih as f64 / sh).floor() as i32).clamp(0, ih - 1);
                let px = img.argb.get((sy * iw + sx) as usize).copied().unwrap_or(0);
                self.set_pixel(x, y, px);
            }
        }
    }

    pub fn draw_image(&mut self, img: &ArgbImage, x: i32, y: i32, anchor: i32) {
        self.op(format!(
            "drawImage({}x{},{x},{y},{anchor})",
            img.logical_width(),
            img.logical_height()
        ));
        let s = self.scale;
        let adj = anchor_adjust(anchor, img.logical_width() * s, img.logical_height() * s);
        let (ox, oy) = (self.phys_x(x) - adj[0], self.phys_y(y) - adj[1]);
        self.blit(img, ox, oy);
    }

    /// Nokia `DirectGraphics.drawImage(img, x, y, anchor, transform)`
    /// （`Graphics.drawImageTransformed`，`Graphics.java:346-376`）。
    /// 【R-2 换算收敛】op 记录与 anchor bbox（swap 后）一律**逻辑尺寸**。
    /// 【R-3】逻辑格网循环 + 块内相位采样：8 变换表**逻辑域不动**（表一行不改，
    /// 仅坐标单位改逻辑格），落笔块 s×s——d==s 时=物理域精确几何变换（③矩阵
    /// 逐像素实证），d=1 时=set_pixel_block 块写（①双侧恒等）。
    /// 【3/3 C1 未闭环】d>1 且 d≠s 的块内相位面待真素材复验。
    pub fn draw_image_transformed(&mut self, img: &ArgbImage, x: i32, y: i32, anchor: i32, transform: i32) {
        self.op(format!(
            "drawImageT({}x{},{x},{y},{anchor},{transform})",
            img.logical_width(),
            img.logical_height()
        ));
        let s = self.scale;
        let d = img.density;
        let swap = (4..=7).contains(&transform);
        let (lw, lh) = (img.logical_width(), img.logical_height());
        let bw = if swap { lh } else { lw };
        let bh = if swap { lw } else { lh };
        let adj = anchor_adjust(anchor, bw * s, bh * s);
        let ox = self.phys_x(x) - adj[0];
        let oy = self.phys_y(y) - adj[1];
        let w = img.width;
        for ly in 0..lh {
            for lx in 0..lw {
                // 变换码表 = shim DirectGraphics 自定义 1..=7（Graphics.java:346-376
                // 权威分派，与 MIDP 旋转族同名不同义）：
                //   1=MIRROR_ROT180(垂直翻转) 2=MIRROR(水平翻转) 3=ROT180
                //   4=MIRROR_ROT270 5=ROT90 6=ROT270 7=MIRROR_ROT90
                // 另支持诺基亚 FLIP 常量 8192/16384/24576（shim 同步实现，
                // a.java:44 f_int_arr_01 值域）。迭代 34：框体角件/底边翻转
                // 此前两侧均未实现（shim default 恒等 + Rust 无臂）= 锚点与
                // 真机观感分歧的根因，现两侧同步翻转。
                // 【R-3】表语义/行文不动，坐标单位 = 逻辑格（(w,h)→(lw,lh)）。
                let (dx, dy) = match transform {
                    1 => (lx, lh - 1 - ly),
                    2 => (lw - 1 - lx, ly),
                    3 => (lw - 1 - lx, lh - 1 - ly),
                    4 => (ly, lx),
                    5 => (lh - 1 - ly, lx),
                    6 => (ly, lw - 1 - lx),
                    7 => (lh - 1 - ly, lw - 1 - lx),
                    8192 => (lw - 1 - lx, ly),
                    16384 => (lx, lh - 1 - ly),
                    24576 => (lw - 1 - lx, lh - 1 - ly),
                    _ => (lx, ly),
                };
                // 落笔块 s×s：目的子像素 NN 采源子像素（相位 src=l*d/s），
                // 块内镜像/转置与码表同构（d==s 双射=物理域精确变换）。
                for iy in 0..s {
                    for ix in 0..s {
                        let (a, b) = in_cell_src_sub(transform, ix, iy, d, s);
                        let p = img.argb[((ly * d + b) * w + (lx * d + a)) as usize];
                        if (p >> 24) & 0xFF == 0 {
                            continue;
                        }
                        self.set_pixel(ox + dx * s + ix, oy + dy * s + iy, p);
                    }
                }
            }
        }
    }

    /// `Graphics.blit`（非 transpose 形态即 drawImage 内部路径，`Graphics.java:378-390`）。
    ///
    /// 【R-1 密度感知最近邻采样器】一次映射（spec §1.2 禁"缩回逻辑再放大"）：
    /// 目标物理格 = 逻辑格×scale、源 = 逻辑格×density，NN 整数相位 **`src = l*d/s`**。
    /// 三段路径（条件 6 性能门槛 ≤3× 现 blit）：
    /// - **d=1**：原 `set_pixel_block ×scale` 块写逐字节恒等（classic 热路径零开销；
    ///   = 新采样在 d=1 的退化情形，判据① 双侧钉）；
    /// - **d==s**：`src=l` 行拷直采（1:1 零相位运算零重采样；原始 `copy_from_slice`
    ///   会绕过 set_pixel 的 clip/alpha 语义，故经 1:1 索引逐像素直采落笔）；
    /// - **d≠s 且 d>1**：增量式相位累加（纯整数、禁浮点、零分配；相位序列与
    ///   `l*d/s` 逐项相等）。
    /// alpha==0 跳过 / 255 直写 / 其他 src-over 与旧 blit 同（set_pixel 不变）。
    /// 【3/3 C1 未闭环】d>1 且 d≠s 分支待真素材复验（量产素材到货时重跑 d≠s 采样
    /// 矩阵 + 画质人工四问，R-1/R-8）。
    fn blit(&mut self, img: &ArgbImage, x: i32, y: i32) {
        let s = self.scale;
        let d = img.density;
        let w = img.width;
        let (lw, lh) = (img.logical_width(), img.logical_height());
        if d == 1 {
            // d=1 退化 = 旧 blit 原算法（1 源像素 = s×s 物理块）。
            for sy in 0..img.height {
                for sx in 0..img.width {
                    let p = img.argb[(sy * w + sx) as usize];
                    if (p >> 24) & 0xFF == 0 {
                        continue;
                    }
                    self.set_pixel_block(x + sx * s, y + sy * s, p);
                }
            }
            return;
        }
        if d == s {
            // d==s 行拷直采：src = l（1:1）。
            for ly in 0..lh {
                for iy in 0..s {
                    let row = ((ly * d + iy) * w) as usize;
                    let py = y + ly * s + iy;
                    for dx in 0..lw * s {
                        let p = img.argb[row + dx as usize];
                        if (p >> 24) & 0xFF == 0 {
                            continue;
                        }
                        self.set_pixel(x + dx, py, p);
                    }
                }
            }
            return;
        }
        // d>1 且 d≠s：NN 整数相位 src = l*d/s（增量式相位，禁浮点）。
        for ly in 0..lh {
            // 行相位：sy = ly*d + iy*d/s（每逻辑格重置，与全局 l*d/s 同序列）
            let mut sy = ly * d;
            let mut y_acc = 0;
            for iy in 0..s {
                if iy > 0 {
                    y_acc += d;
                    while y_acc >= s {
                        sy += 1;
                        y_acc -= s;
                    }
                }
                let row = (sy * w) as usize;
                let py = y + ly * s + iy;
                // 列相位：sx = dx*d/s（全局增量式 == 逐格 lx*d + ix*d/s）
                let mut sx = 0;
                let mut x_acc = 0;
                for dx in 0..lw * s {
                    if dx > 0 {
                        x_acc += d;
                        while x_acc >= s {
                            sx += 1;
                            x_acc -= s;
                        }
                    }
                    let p = img.argb[row + sx as usize];
                    if (p >> 24) & 0xFF == 0 {
                        continue;
                    }
                    self.set_pixel(x + dx, py, p);
                }
            }
        }
    }

    // ---------------- 文字 ----------------

    /// `drawChar`。
    pub fn draw_char(&mut self, ch: u16, x: i32, y: i32, anchor: i32) {
        self.op(format!("drawChar({ch},{x},{y},{anchor})"));
        self.draw_glyph(ch, x, y, anchor);
    }

    /// `drawString`。
    pub fn draw_string(&mut self, s: &[u16], x: i32, y: i32, anchor: i32) {
        self.op(format!("drawString({},{x},{y},{anchor})", quote_utf16(s)));
        self.draw_text(s, 0, s.len(), x, y, anchor);
    }

    /// `drawSubstring`。
    ///
    /// len<0 或越界时与 shim 同语义 = 空循环静默无操作
    /// （Font.substringWidth/Graphics.drawText 的 `i < offset+len` 不执行；
    /// 对抗 review D-2：调用方 a.java:5054 的 len 是差值，可能为负）。
    pub fn draw_substring(&mut self, s: &[u16], offset: i32, len: i32, x: i32, y: i32, anchor: i32) {
        self.op(format!(
            "drawSubstring({},{offset},{len},{x},{y},{anchor})",
            quote_utf16(s)
        ));
        let from = offset.max(0) as usize;
        let to = (offset + len).clamp(0, s.len() as i32) as usize;
        if from >= to || to > s.len() {
            return;
        }
        self.draw_text(s, from, to, x, y, anchor);
    }

    /// `drawText`：先整体度量锚点，再逐字符步进（`Graphics.java:424-438`）。
    fn draw_text(&mut self, s: &[u16], from: usize, to: usize, x: i32, y: i32, anchor: i32) {
        let font = self.font();
        let w = font.substring_width(s, from, to - from);
        let h = font.height;
        let mut adj = anchor_adjust(anchor, w, h);
        if anchor & anchor::BASELINE != 0 {
            adj[1] = font.baseline;
        }
        let mut px = x - adj[0];
        let py = y - adj[1];
        for &ch in &s[from..to] {
            self.draw_glyph(ch, px, py, anchor::TOP | anchor::LEFT);
            px += font.char_width(ch);
        }
    }

    /// `drawGlyph`：确定性 hash 5 列点阵（`Graphics.java:440-462`）。
    ///
    /// hash = (int)(ch * 2654435761L)（long 乘法取低 32 位），位采样 `(hash >>> ((gx + gy*5) % 29)) & 1`。
    fn draw_glyph(&mut self, ch: u16, x: i32, y: i32, anchor: i32) {
        let font = self.font();
        let cw = font.char_width(ch);
        let chh = font.height;
        let mut adj = anchor_adjust(anchor, cw, chh);
        if anchor & anchor::BASELINE != 0 {
            adj[1] = font.baseline;
        }
        // 逻辑坐标（含 origin）——度量三元组/行高派生链不动（task-13 判据②）。
        let px = x + self.origin_x - adj[0];
        let py = y - adj[1];
        let c = self.opaque_color();
        // D2 字体缝：平台真字形钩子（game-desktop 注入系统 TTF 渲染）。
        // 【task-13 ②】钩子契约：入参=逻辑（含 origin）；put=**物理坐标**
        // （字形按物理尺寸光栅化=真 TTF 锐度；降采样路径由钩子侧撤除）。
        // None = 差分占位字形（下方哈希块——与 Java shim 同构，差分逐像素可比）。
        if let Some(h) = glyph_hook() {
            let s = self.scale;
            let mut put = |gx: i32, gy: i32, col: u32| self.set_pixel(gx, gy, col);
            h(ch, px, py, cw, chh, c, &mut put, s);
            return;
        }
        let gw = (cw * 4 / 5).max(2);
        let gh = (chh * 3 / 5).max(3);
        let hash = ((ch as i64).wrapping_mul(2654435761)) as i32 as u32;
        let s = self.scale;
        let (bx, by) = (self.phys_x(x - adj[0]), py * s);
        for gy in 0..gh {
            for gx in 0..gw {
                let bit = (hash >> ((gx + gy * 5) % 29)) & 1;
                if bit != 0 {
                    self.set_pixel_block(
                        bx + ((cw - gw) / 2 + gx) * s,
                        by + ((chh - gh) / 2 + gy) * s,
                        c,
                    );
                }
            }
        }
    }
}

/// `edge(x1,y1,x2,y2,y)`：double 插值，Java 表达式序 `(x2-x1)*(y-y1)/(y2-y1) + x1`。
fn edge(x1: i32, y1: i32, x2: i32, y2: i32, y: i32) -> f64 {
    if y1 == y2 {
        return (x1.min(x2)) as f64;
    }
    x1 as f64 + ((x2 - x1) as f64 * (y - y1) as f64 / (y2 - y1) as f64)
}

/// `inArc`（`Graphics.java:313-332`）。
fn in_arc(rad: f64, start_deg: i32, arc_deg: i32) -> bool {
    let mut deg = rad.to_degrees();
    if deg < 0.0 {
        deg += 360.0;
    }
    if arc_deg >= 360 {
        return true;
    }
    if arc_deg < 0 {
        // shim 同位置有 % 360（Graphics.java:322）；大角度下省略会因 f64
        // 精度丢失与 while 归一化产生 ULP 级漂移（对抗 review D-1）
        return !in_arc(rad, (start_deg + arc_deg) % 360, -arc_deg);
    }
    let mut rel = deg - start_deg as f64;
    while rel < 0.0 {
        rel += 360.0;
    }
    while rel >= 360.0 {
        rel -= 360.0;
    }
    rel <= arc_deg as f64
}

/// `anchorAdjust`（`Graphics.java:392-405`）：水平 HCENTER→w/2、RIGHT→w；垂直 VCENTER→h/2、BOTTOM→h。
fn anchor_adjust(anchor: i32, w: i32, h: i32) -> [i32; 2] {
    let mut ax = 0;
    let mut ay = 0;
    if anchor & anchor::HCENTER != 0 {
        ax = w / 2;
    } else if anchor & anchor::RIGHT != 0 {
        ax = w;
    }
    if anchor & anchor::VCENTER != 0 {
        ay = h / 2;
    } else if anchor & anchor::BOTTOM != 0 {
        ay = h;
    }
    [ax, ay]
}

/// 【R-3】transformed 落笔块内采样：目的子像素 (ix,iy)∈s² → 源子像素 (a,b)∈d²。
///
/// NN 相位 `p = i*d/s`（整数）按变换码做块内镜像/转置，与 [`transform`] 码表
/// 同构（同轴反演）——d==s 时双射 = 物理域精确正交变换；d=1 时恒 (0,0) =
/// `set_pixel_block` 块写（①恒等）；d≠s 的块内相位面【3/3 C1 未闭环】。
fn in_cell_src_sub(transform: i32, ix: i32, iy: i32, d: i32, s: i32) -> (i32, i32) {
    let (px, py) = (ix * d / s, iy * d / s);
    match transform {
        1 | 16384 => (px, d - 1 - py),
        2 | 8192 => (d - 1 - px, py),
        3 | 24576 => (d - 1 - px, d - 1 - py),
        4 => (py, px),
        5 => (py, d - 1 - px),
        6 => (d - 1 - py, px),
        7 => (d - 1 - py, d - 1 - px),
        _ => (px, py),
    }
}

/// OPS 文本里的字符串转义（shim `Graphics.q`，`Graphics.java:75-94`）。
fn quote_utf16(s: &[u16]) -> String {
    let mut out = String::from("\"");
    for &c in s {
        match c {
            0x22 => out.push_str("\\\""), // "
            0x5C => out.push_str("\\\\"), // backslash
            0x0A => out.push_str("\\n"),
            0x0D => out.push_str("\\r"),
            0x09 => out.push_str("\\t"),
            _ if (0x20..0x7F).contains(&c) => out.push(c as u8 as char),
            _ => out.push_str(&format!("\\u{c:04x}")),
        }
    }
    out.push('"');
    out
}


// ---------------- D2 字体缝（字形渲染钩子） ----------------

/// 平台字形渲染钩子。参数：(字符码, 格左上 x, 格上 y, 格宽, 格高, 颜色, 画点)。
/// None（默认）= 差分占位字形（哈希块，与 Java shim 同构——oracle 差分逐像素
/// 可比的锚，84 用例全绿基于此）；Some = 平台真字体（game-desktop 注入
/// 系统 TTF；差分测试不受影响——测试进程从不注入）。
/// 真字形钩子：(ch, px, py, cw, chh, color, put, scale)——入参逻辑坐标（含
/// origin）；put=**物理坐标**（scale 由图形层为真相源传入，钩子按 scale 光栅化
/// 直落物理=真 TTF 锐度；task-13 ② 替换降采样）。
pub type GlyphHook = fn(u16, i32, i32, i32, i32, u32, &mut dyn FnMut(i32, i32, u32), i32);

static mut GLYPH_HOOK: Option<GlyphHook> = None;

/// 注入平台字形渲染（启动时一次；None 恢复差分占位）。
pub fn set_glyph_hook(h: Option<GlyphHook>) {
    unsafe {
        GLYPH_HOOK = h;
    }
}

fn glyph_hook() -> Option<GlyphHook> {
    unsafe { GLYPH_HOOK }
}

#[cfg(test)]
/// task-13/14 判据（adversary 三面）：③平移×缩放可交换、④像素素材最近邻块写、
/// ②字形物理落点（占位面=块写；钩子面=shell font_inject 物理落点判据）。
/// classic 1×/origin=0 恒等=全既有套件+oracle ops 字节级（gate 护航）。
mod scale_origin_tests {
    use super::*;

    /// 【adversary③】平移×缩放可交换：origin 平移绘制的**像素** == 显式平移
    /// 参数绘制（同 scale）；ops 记调用方逻辑坐标（origin 隐式不改 ops 面）。
    #[test]
    fn translate_scale_commute() {
        let mut a = ArgbImage::create(80, 80);
        {
            let mut g = SoftGraphics::new(&mut a);
            g.set_scale(2);
            g.set_origin(10);
            g.set_color(0xFF0000);
            g.fill_rect(5, 5, 4, 3);
            assert_eq!(
                g.ops.last().map(String::as_str),
                Some("fillRect(5,5,4,3,#ff0000)"),
                "ops 记逻辑坐标（origin 隐式）"
            );
        }
        let mut b = ArgbImage::create(80, 80);
        {
            let mut g = SoftGraphics::new(&mut b);
            g.set_scale(2);
            g.set_color(0xFF0000);
            g.fill_rect(15, 5, 4, 3);
        }
        assert_eq!(a.argb, b.argb, "平移×缩放可交换（origin 平移 == 显式平移参数）");
    }

    /// 【task-13④】像素素材最近邻 ×scale：1 源像素 = scale×scale 硬边块。
    #[test]
    fn sprite_nearest_neighbor_blocks() {
        let mut img = ArgbImage::create(2, 1);
        img.argb[0] = 0xFF112233;
        img.argb[1] = 0xFF445566;
        let mut dst = ArgbImage::create(8, 4);
        {
            let mut g = SoftGraphics::new(&mut dst);
            g.set_scale(2);
            g.draw_image(&img, 0, 0, 0);
        }
        // 源 (0,0) → 物理 (0..2,0..2) 块；源 (1,0) → (2..4,0..2)
        assert_eq!(dst.argb[0], 0xFF112233);
        assert_eq!(dst.argb[1], 0xFF112233);
        assert_eq!(dst.argb[2], 0xFF445566);
        assert_eq!(dst.argb[1 * 8 + 1], 0xFF112233, "块写=垂直同色（最近邻非插值）");
    }

    /// 【task-13②·占位面】字形物理落点：scale=2 占位字形每点=2×2 硬边块
    /// （度量三元组/行高派生链不动——font_metrics_tests 钉）。
    #[test]
    fn placeholder_glyph_blocks_at_scale2() {
        let mut dst = ArgbImage::create(40, 20);
        {
            let mut g = SoftGraphics::new(&mut dst);
            g.set_scale(2);
            g.set_color(0xFFFFFFFF);
            g.draw_char('A' as u16, 0, 0, 0);
        }
        let s = 2;
        // 找一个已绘物理像素，断言其 2×2 块完整（块写语义）
        let mut found = None;
        for y in 0..20 {
            for x in 0..40 {
                if dst.argb[y * 40 + x] == 0xFFFFFFFF {
                    found = Some((x - x % s, y - y % s));
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }
        let (bx, by) = found.expect("占位字形须有笔画像素");
        for dy in 0..s {
            for dx in 0..s {
                assert_eq!(
                    dst.argb[((by + dy) * 40 + (bx + dx)) as usize],
                    0xFFFFFFFF,
                    "2×2 硬边块（{bx},{by}）"
                );
            }
        }
    }
}

#[cfg(test)]
mod font_metrics_tests {
    use super::*;

    /// task-25 ① 恒等断言（font-metrics-dependency-survey §3 legacy 恒等面）：
    /// classic 度量逐项=SoftFont 现值零偏离（6/14/11、8/14/11、12/20/16、
    /// BOLD aw+1、is_wide 2×、line_height=height+4）——度量换轨的钉死基准。
    #[test]
    fn classic_metrics_identity_matches_softfont_values() {
        let cases = [
            (font_size::SMALL, font_style::PLAIN, 6, 14, 11),
            (font_size::MEDIUM, font_style::PLAIN, 8, 14, 11),
            (font_size::LARGE, font_style::PLAIN, 12, 20, 16),
            (font_size::SMALL, font_style::BOLD, 7, 14, 11),
            (font_size::MEDIUM, font_style::BOLD, 9, 14, 11),
            (font_size::LARGE, font_style::BOLD, 13, 20, 16),
        ];
        for (size, style, aw, h, bl) in cases {
            let f = SoftFont::get_font(0, style, size);
            let m = ClassicMetrics { font: f };
            assert_eq!(m.height(), h, "size={size} style={style} height");
            assert_eq!(m.baseline(), bl, "size={size} style={style} baseline");
            assert_eq!(m.line_height(), h + 4, "行距=height+4（a.java:471）");
            assert_eq!(f.ascii_width, aw, "BOLD aw+1 步进");
            // is_wide 2×：ASCII=aw、CJK=2aw
            assert_eq!(m.char_advance(b'A' as u16), aw, "ASCII 定宽步进");
            assert_eq!(m.char_advance(0x4E2D), aw * 2, "CJK 2× 步进（is_wide）");
            // measure=(串宽, 行距)
            let s: Vec<u16> = "A中".encode_utf16().collect();
            assert_eq!(m.measure(&s), (aw + aw * 2, h + 4), "measure=(步进累加, 行距)");
        }
    }

    /// task-25 ③ 判据：变宽通道——hook 注入实测步进生效（ASCII/CJK 各自实测值
    /// 而非定宽 2×）；未注入回退 classic 定宽（零偏离）。
    #[test]
    fn modern_metrics_variable_advance_via_hook() {
        // 未注入 ⇒ 回退 classic 定宽（is_wide 2×）
        let m = ModernMetrics { size: font_size::MEDIUM, style: font_style::PLAIN };
        assert_eq!(m.char_advance(b'A' as u16), 8, "回退 classic ASCII 定宽");
        assert_eq!(m.char_advance(0x4E2D), 16, "回退 classic CJK 2×");
        // 注入实测步进（变宽：A=5 中=12——TTF 实测形）
        fn fake_advance(_size: i32, ch: u16) -> i32 {
            if ch == b'A' as u16 { 5 } else { 12 }
        }
        set_advance_hook(Some(fake_advance));
        assert_eq!(m.char_advance(b'A' as u16), 5, "实测 ASCII 变宽步进");
        assert_eq!(m.char_advance(0x4E2D), 12, "实测 CJK 变宽步进（非 2×）");
        let s: Vec<u16> = "A中".encode_utf16().collect();
        assert_eq!(m.measure(&s), (17, 18), "measure=实测步进累加（5+12），行距=height+4");
        // height/baseline 族=classic 同值（度量高族不动）
        assert_eq!(m.height(), 14);
        assert_eq!(m.line_height(), 18);
        assert_eq!(m.baseline(), 11);
        set_advance_hook(None);
    }

    /// task-25 ① 双实现恒等：SoftFont 直装 trait 与 ClassicMetrics 包装逐值同
    ///（68 处消费面两路接入的等价性基准）。
    #[test]
    fn softfont_trait_impl_matches_classic_wrapper() {
        let f = SoftFont::get_font(0, font_style::PLAIN, font_size::MEDIUM);
        let m = ClassicMetrics { font: f };
        for ch in [b'A' as u16, 0x4E2D, 0x3042] {
            assert_eq!(f.char_advance(ch), m.char_advance(ch), "char_advance 双实现同值");
        }
        let s: Vec<u16> = "测试ABC".encode_utf16().collect();
        assert_eq!(f.measure(&s), m.measure(&s), "measure 双实现同值");
        assert_eq!(f.line_height(), m.line_height());
        assert_eq!(f.baseline(), m.baseline());
    }
}
