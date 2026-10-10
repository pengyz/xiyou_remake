//! paint 调用树（`a.java:2333` 起 876 行的 `paint(Graphics)` 的 Rust 端口）。
//!
//! # 结构
//!
//! `paint` 是按 `gameMode`（[`GameMode`] 枚举）分发的巨型状态投影：每个分支把
//! 当前游戏状态绘制到画布。Rust 端保持同一分发结构与绘制顺序，绘制原语走
//! [`crate::render::SoftGraphics`]（shim 像素模型的逐字节复刻）。
//!
//! # 对拍裁判（三层）
//!
//! 1. **OPS 流**：每 tick 的绘制调用序列（trace 里 `  setColor(...)` 缩进行）
//! 2. **FRAME sha**：每帧全屏像素哈希（trace `FRAME sha=`，截断 16 字节 / 32 hex）
//! 3. 本文件内单测以真实 trace 帧哈希为锚（引用 tick 标注在断言行）
//!
//! # 进度
//!
//! - [x] 入口：`setFont(f_Font_00)`（SIZE_SMALL，a.java:2336、24）
//! - [x] mode 0 相位 <2：白底 + 启动图居中（a.java:2338-2344）
//! - [ ] mode 0 相位 >=2：m_144 logo 动画层（a.java:10423）——待动画子系统状态端口
//! - [ ] mode 4/19 主菜单、mode 1/2 加载/进度条、其余分支

use crate::render::{ArgbImage, SoftFont, SoftGraphics, font_size, font_style};

/// paint 入口持有的常量字体（a.java:24：`f_Font_00 = Font.getFont(0, 0, 8)`）。
pub fn paint_font() -> SoftFont {
    SoftFont::get_font(0, font_style::PLAIN, font_size::SMALL)
}

/// mode 0 分支的启动相位输入（a.java:2339 `f_int_04`）与启动图（`f_Image_00`）。
#[derive(Debug, Default, Clone)]
pub struct BootPaintState {
    /// 启动加载相位：0/1 → 轮播 `/l{n}.png`；>=2 → logo 动画层。
    pub phase: i32,
    /// 启动图（l0.png 164×117 / l1.png 176×138，加载线程 a.java:3332-3346）。
    pub image: Option<ArgbImage>,
}

/// mode 0（启动屏）绘制：a.java:2338-2347。
///
/// 相位 <2：整屏白 + 启动图几何居中（Java `-` 优先于 `>>` ⇒ `(240-w)>>1`）；
/// 相位 >=2：`m_144` logo 动画层（未端口，显式 todo 防静默漏绘）。
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
        // m_144 依赖 logo 动画登记子系统（m_145 注册表 + f_byte_arr_46 度量表），
        // 端口前不允许用空实现冒充通过（AGENTS §3：禁止未验证行为）。
        unimplemented!("m_144 logo 动画层待端口（a.java:10423）")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::ArgbImage;
    use game_platform::hash::sha256_hex;
    use std::path::PathBuf;

    /// 与 TickHooks 同流的截断帧哈希（FRAME sha= 前 32 hex）。
    fn frame_sha(screen: &ArgbImage) -> String {
        sha256_hex(&screen.hash_stream())[..32].to_string()
    }

    /// trace A-boot-menu 两相启动图与帧哈希逐一比对：
    /// - l0.png（164x117@38,101）→ TICK 8 帧帧哈希前缀 `57173976…`
    /// - l1.png（176x138@32,91）→ TICK 24 帧 `6a8aa7a0…`
    /// 证据：`reference/oracle/_out/A-boot-menu/trace.txt` T8/T24 的 OPS 行
    /// （白底 + drawImage）与 FRAME sha（`(240-w)>>1, (320-h)>>1` 居中算式）。
    #[test]
    fn boot_phase0_matches_trace_frames() {
        for (file, w, h, x, y, tick, sha) in [
            ("l0.png", 164, 117, 38, 101, 8, "57173976f11eeae0aed2c0029b6d5dd6"),
            ("l1.png", 176, 138, 32, 91, 24, "6a8aa7a0f744030761e35dac5b0c002b"),
        ] {
            let bytes = std::fs::read(fixtures().join(file)).unwrap();
            let img = ArgbImage::from_decoded_png(game_data::decode_png(&bytes).unwrap());
            assert_eq!((img.width, img.height), (w, h), "{file} 尺寸");

            let mut screen = ArgbImage::create(240, 320);
            let ops;
            {
                let mut g = SoftGraphics::new(&mut screen);
                g.set_font(Some(paint_font()));
                let boot = BootPaintState { phase: 0, image: Some(img) };
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
            let boot = BootPaintState { phase: 0, image: None };
            paint_boot(&mut g, &boot);
        }
        // TICK 32 帧哈希 = 纯白画布（与 L1b blank_white golden 同值，交叉验证）
        assert_eq!(frame_sha(&screen), "a682fa570213181c0f6fd50f7da5ef6f");
    }

    fn fixtures() -> PathBuf {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let raw = repo.join("assets/raw");
        assert!(raw.join("l0.png").exists(), "缺 assets/raw 夹具（先跑 tools/extract-assets）");
        raw
    }
}
