//! paint 调用树（`a.java:2333` 起 876 行的 `paint(Graphics)` 的 Rust 端口）。
//!
//! # 结构
//!
//! `paint` 是按 `gameMode` 分发的巨型状态投影：每个分支把当前游戏状态绘制到
//! 画布。Rust 端保持同一分发结构与绘制顺序，绘制原语走
//! [`crate::render::SoftGraphics`]（shim 像素模型的逐字节复刻）。
//!
//! # 对拍裁判（三层）
//!
//! 1. **OPS 流**：每 tick 的绘制调用序列（trace 里 `  setColor(...)` 缩进行）
//! 2. **FRAME sha**：每帧全屏像素哈希（trace `FRAME sha=`，截断 16 字节 / 32 hex）
//! 3. 回放测试逐 tick 对拍（`game-oracle/tests/boot_replay.rs`）
//!
//! # 进度
//!
//! - [x] 入口：`setFont(f_Font_00)`（SIZE_SMALL，a.java:2336、24）
//! - [x] mode 0 全分支：白底+启动图居中（a.java:2338-2344）/ m_144 logo 层
//! - [x] mode 0 tick 逻辑：加载轮播 + logo 时间线（a.java:3330-3361）
//! - [ ] mode 21（logo 后过渡）、mode 4/19 主菜单、mode 1/2 加载、其余分支

use crate::render::{ArgbImage, SoftFont, SoftGraphics, font_size, font_style};

/// paint 入口持有的常量字体（a.java:24：`f_Font_00 = Font.getFont(0, 0, 8)`）。
pub fn paint_font() -> SoftFont {
    SoftFont::get_font(0, font_style::PLAIN, font_size::SMALL)
}

/// mode 0（启动屏）绘制：a.java:2338-2347。
///
/// 相位 <2：整屏白 + 启动图几何居中（Java `-` 优先于 `>>` ⇒ `(240-w)>>1`）；
/// 相位 >=2：`m_144` logo 动画层（[`crate::logo_anim::LogoAnim::paint`]）。
pub fn paint_boot(g: &mut SoftGraphics<'_>, boot: &BootPaintState) {
    if boot.phase < 2 {
        // setColor(-1)：Java int -1 位模式 & 0xFFFFFF = 0xFFFFFF
        g.set_color((-1i32) as u32);
        g.fill_rect(0, 0, 240, 320);
        if let Some(img) = &boot.image {
            let x = 240 - img.width >> 1;
            let y = 320 - img.height >> 1;
            g.draw_image(img, x, y, 0);
        }
    } else {
        match (&boot.logo, &boot.sflogo7) {
            (Some(logo), Some(sflogo7)) => logo.paint(g, sflogo7),
            // logic 已切 f_int_04>=2 但首个 m_143 未跑：f_byte_29=0 ⇒ m_144 仅白底
            // （A-boot-menu T32 硬锚：a682fa57=纯白）
            _ => {
                g.set_color(0xFFFFFF);
                g.fill_rect(0, 0, 240, 320);
            }
        }
    }
}

/// mode 0 启动状态机：tick 逻辑（a.java:3330-3361）+ paint 输入投影。
///
/// # 时序合同（差分实测，A-boot-menu 四硬锚交叉验证）
///
/// 每 tick **逻辑先 paint 后**：TICK n 的帧 = 第 n 次逻辑推进后的投影。
/// 硬锚：T17=l1（logic#17 加载即生效）、T31=l1/T32=白（f_int_04 切换
/// 发生在 31/32 帧之间）、T33=首个 logo 帧、T40=时间线 8 状态。
/// 计数细节：logic#1 走 `f_int_156==0` 加载分支（不加计数），`++` 自
/// logic#2 起拍——l0 显示 TICK 1-16（16 帧）、l1 显示 TICK 17-31（15 帧）。
pub struct BootMachine {
    /// f_int_04：加载相位（0=l0, 1=l1, 2+=logo 时间线）。
    pub phase: i32,
    /// f_int_156：相位内计数（轮播每张 16 tick：1..15 显示，16 切换；时间线 1..35）。
    pub counter: i32,
    /// f_Image_00：当前启动图（l0/l1）。
    pub image: Option<ArgbImage>,
    /// logo 动画层（f_int_04>=2 起参与绘制）。
    pub logo: Option<crate::logo_anim::LogoAnim>,
    /// l0/l1 预解码件（a.java:3335 的 `/l{n}.png`）。
    boot_images: Vec<ArgbImage>,
    /// sflogo 容器 8 张（m_001(0) 首次进 logo 相位时加载）。
    sflogo: Vec<ArgbImage>,
    /// paint 计数（对拍 TICK n 用）。
    pub paints: u32,
    /// mode 0 结束标志（logo 时间线 35 耗尽，a.java:3351-3360 切 gameMode=21）。
    pub finished: bool,
}

impl BootMachine {
    /// 构造：预解码三份资源（解码耗时与 tick 语义无关）。
    pub fn new(l0: ArgbImage, l1: ArgbImage, sflogo: Vec<ArgbImage>) -> BootMachine {
        assert_eq!(sflogo.len(), 8, "sflogo 容器 8 张（a.java:465-466 计数表）");
        BootMachine {
            phase: 0,
            counter: 0,
            image: None,
            logo: None,
            boot_images: vec![l0, l1],
            sflogo,
            paints: 0,
            finished: false,
        }
    }

    /// 一次 paint（serviceRepaints → a.paint，gameMode 0 分支）。
    pub fn paint(&mut self, g: &mut SoftGraphics<'_>) {
        self.paints += 1;
        let state = BootPaintState {
            phase: self.phase,
            image: self.image.clone(),
            logo: self.logo.clone(),
            sflogo7: Some(self.sflogo[7].clone()),
        };
        paint_boot(g, &state);
    }

    /// 一次逻辑 tick（a.java:3330-3361，gameMode 0 分支）。
    pub fn tick(&mut self) {
        if self.phase < 2 {
            if self.counter == 0 {
                // a.java:3335：加载 /l{phase}.png；夹具恒存在 ⇒ 恒成功路径
                let idx = self.phase as usize;
                self.image = self.boot_images.get(idx).cloned();
                if self.image.is_some() {
                    self.counter = 1;
                } else {
                    self.phase += 1; // 防御：a.java:3339-3341
                }
            } else {
                self.counter += 1;
                if self.counter > 15 {
                    self.phase += 1; // a.java:3346
                    self.counter = 0;
                }
            }
        } else {
            self.counter += 1; // a.java:3349
            if self.counter <= 35 {
                self.logo.get_or_insert_with(crate::logo_anim::LogoAnim::new)
                    .tick(&self.sflogo, self.counter);
            } else {
                // a.java:3352-3360：清理 + gameMode=21（mode 21 分支另行端口）
                self.logo.as_mut().unwrap().clear_all(); // m_147(-1)
                self.image = None;
                self.counter = 0;
                self.finished = true;
            }
        }
    }
}

/// mode 0 分支的绘制输入投影（a.java:2339 `f_int_04` 等的只读快照）。
#[derive(Clone)]
pub struct BootPaintState {
    /// 启动加载相位：0/1 → 轮播 `/l{n}.png`；>=2 → logo 动画层。
    pub phase: i32,
    /// 启动图（l0.png 164×117 / l1.png 176×138，加载线程 a.java:3332-3346）。
    pub image: Option<ArgbImage>,
    /// logo 动画层（phase>=2 时 Some）。
    pub logo: Option<crate::logo_anim::LogoAnim>,
    /// sflogo#7（m_144 字形槽源图，a.java:10434）。
    pub sflogo7: Option<ArgbImage>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_platform::hash::sha256_hex;
    use std::path::PathBuf;

    /// 与 TickHooks 同流的截断帧哈希（FRAME sha= 前 32 hex）。
    fn frame_sha(screen: &ArgbImage) -> String {
        sha256_hex(&screen.hash_stream())[..32].to_string()
    }

    fn repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn decode(name: &str) -> ArgbImage {
        let bytes = std::fs::read(repo().join("assets/raw").join(name)).unwrap();
        ArgbImage::from_decoded_png(game_data::decode_png(&bytes).unwrap())
    }

    fn load_sflogo() -> Vec<ArgbImage> {
        let data = std::fs::read(repo().join("assets/raw/sflogo")).unwrap();
        game_data::PackedPng::parse(&data, 8)
            .unwrap()
            .images
            .iter()
            .map(|sub| ArgbImage::from_decoded_png(game_data::decode_png(sub.bytes).unwrap()))
            .collect()
    }

    /// trace A-boot-menu 两相启动图与帧哈希逐一比对：
    /// - l0.png（164x117@38,101）→ TICK 8 帧 `57173976…`
    /// - l1.png（176x138@32,91）→ TICK 24 帧 `6a8aa7a0…`
    /// 证据：`reference/oracle/_out/A-boot-menu/trace.txt` T8/T24 的 OPS 行
    /// （白底 + drawImage）与 FRAME sha（`(240-w)>>1, (320-h)>>1` 居中算式）。
    #[test]
    fn boot_phase0_matches_trace_frames() {
        for (file, w, h, x, y, tick, sha) in [
            ("l0.png", 164, 117, 38, 101, 8, "57173976f11eeae0aed2c0029b6d5dd6"),
            ("l1.png", 176, 138, 32, 91, 24, "6a8aa7a0f744030761e35dac5b0c002b"),
        ] {
            let img = decode(file);
            assert_eq!((img.width, img.height), (w, h), "{file} 尺寸");

            let mut screen = ArgbImage::create(240, 320);
            let ops;
            {
                let mut g = SoftGraphics::new(&mut screen);
                g.set_font(Some(paint_font()));
                let boot = BootPaintState {
                    phase: 0,
                    image: Some(img),
                    logo: None,
                    sflogo7: None,
                };
                paint_boot(&mut g, &boot);
                ops = g.ops.clone();
            }
            assert_eq!(
                frame_sha(&screen),
                sha,
                "{file} 白底+居中必须与 Java 帧（TICK {tick}）逐字节一致"
            );
            assert_eq!(
                ops,
                vec![
                    "setFont(size=8)".to_string(),
                    "setColor(#ffffff)".to_string(),
                    "fillRect(0,0,240,320,#ffffff)".to_string(),
                    format!("drawImage({w}x{h},{x},{y},0)"),
                ],
                "{file} OPS 序列"
            );
        }
    }

    /// trace TICK 32：f_Image_00 已释放（null）⇒ 纯白帧。
    #[test]
    fn boot_phase0_no_image_matches_trace_t32() {
        let mut screen = ArgbImage::create(240, 320);
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_font(Some(paint_font()));
            let boot = BootPaintState { phase: 0, image: None, logo: None, sflogo7: None };
            paint_boot(&mut g, &boot);
        }
        // TICK 32 帧哈希 = 纯白画布（与 L1b blank_white golden 同值，交叉验证）
        assert_eq!(frame_sha(&screen), "a682fa570213181c0f6fd50f7da5ef6f");
    }
}
