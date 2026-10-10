//! 渲染光栅器 golden（L1b）：Rust 复刻（game-core::render）与 Java shim
//! 微驱动（reference/oracle/src/oracle/host/RenderGolden.java）对**同一组固定
//! 绘制用例**产出逐字节一致的像素哈希。
//!
//! 裁判：`data/golden/render-golden.json`（Java 侧 MessageDigest SHA-256，
//! 与 TickHooks.afterPaint 帧哈希同一输入流）。任何一侧像素模型变化导致
//! 本测试红 ⇒ FRAME sha 对拍必然破裂，先修层再动门禁（AGENTS §4）。
//!
//! 字符串用 UTF-16 码元序列（与 Java char[] 同域），CJK 用例覆盖宽字符度量。

use game_core::render::{
    anchor, font_size, font_style, ArgbImage, SoftFont, SoftGraphics,
};
use game_platform::hash::sha256_hex;

const W: i32 = 240;
const H: i32 = 320;

/// 与 Java 驱动同壳：白底画布 → 用例绘制 → 全屏 sha256。
fn render(paint: impl FnOnce(&mut SoftGraphics<'_>)) -> String {
    let mut screen = ArgbImage::create(W, H);
    let mut g = SoftGraphics::new(&mut screen);
    g.set_color(0xFFFFFF);
    g.fill_rect(0, 0, W, H);
    paint(&mut g);
    sha256_hex(&screen.hash_stream())
}

/// 位置梯度源（12x8），`RenderGolden.gradientSource` 同参。
fn gradient_source() -> ArgbImage {
    let mut px = vec![0u32; 12 * 8];
    for y in 0..8i32 {
        for x in 0..12i32 {
            px[(y * 12 + x) as usize] =
                0xFF000000 | ((x * 20) << 16) as u32 | ((y * 30) << 8) as u32 | ((x + y) * 10) as u32;
        }
    }
    ArgbImage::from_rgb(&px, 12, 8, true)
}

/// 半透明源（11x7），`RenderGolden.translucentSource` 同参。
fn translucent_source() -> ArgbImage {
    let mut px = vec![0u32; 11 * 7];
    for (i, p) in px.iter_mut().enumerate() {
        *p = if i % 3 == 0 { 0 } else { 0x80000000 | 0x0000FF };
    }
    ArgbImage::from_rgb(&px, 11, 7, true)
}

fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

#[test]
fn blank_white() {
    assert_eq!(
        render(|_g| {}),
        "a682fa570213181c0f6fd50f7da5ef6f263c855222c8f7441f19227d925383a3"
    );
}

#[test]
fn fill_clip() {
    assert_eq!(
        render(|g| {
            g.set_clip(20, 30, 100, 80);
            g.set_color(0xFF0000);
            g.fill_rect(0, 0, 240, 320);
        }),
        "ccf7b1571bb691696ace0afa9ca68f37fdaeeb3947f60257733efb580b5101e9"
    );
}

#[test]
fn clip_rect_chain() {
    assert_eq!(
        render(|g| {
            g.set_clip(10, 10, 50, 50);
            g.clip_rect(30, 30, 100, 100);
            g.set_color(0x00FF00);
            g.fill_rect(0, 0, 240, 320);
        }),
        "1cfe50fa8286d181dbba9395435ec88e76698e6cb81ba34e8141752768310107"
    );
}

#[test]
fn translate_clip() {
    // shim 语义：translate 只移动 clip（坐标原点不动）
    assert_eq!(
        render(|g| {
            g.set_clip(100, 100, 40, 40);
            g.translate(-50, -50);
            g.set_color(0x0000FF);
            g.fill_rect(100, 100, 20, 20);
            g.fill_rect(50, 50, 20, 20);
        }),
        "70b90c4eb1dce304bf67563c02bccc990d5dee4312ed3fd622b67ce8375cf265"
    );
}

#[test]
fn bresenham_octants() {
    assert_eq!(
        render(|g| {
            g.set_color(0x000000);
            let (cx, cy) = (120, 160);
            let pts: [(i32, i32); 11] = [
                (200, 160), (200, 260), (120, 260), (40, 260), (40, 160),
                (40, 60), (120, 60), (200, 60), (210, 30), (30, 290), (130, 20),
            ];
            for (x, y) in pts {
                g.draw_line(cx, cy, x, y);
            }
            g.draw_line(10, 10, 230, 310);
            g.draw_line(230, 10, 10, 310);
        }),
        "a6f7d0a50871ed438c8c84d81268a176d07de943c3b63b292ddd21c0df1176f5"
    );
}

#[test]
fn draw_rect_sizes() {
    assert_eq!(
        render(|g| {
            g.set_color(0xFF8800);
            g.draw_rect(10, 10, 50, 30);
            g.draw_rect(70, 10, 1, 1);
            g.draw_rect(80, 10, 0, 5);
            g.draw_rect(90, 10, 5, 0);
        }),
        "2e6f944e7657e14602e81573147526661b37e426247456ba7ccbcd0e0a8b2576"
    );
}

#[test]
fn fill_triangle_flat() {
    assert_eq!(
        render(|g| {
            g.set_color(0x00AAAA);
            g.fill_triangle(30, 250, 70, 250, 50, 230);
        }),
        "1a3cae9bd936f8165e33b98c5c1da9271db98f146633341da499c875a9309820"
    );
}

#[test]
fn fill_triangle_steep() {
    assert_eq!(
        render(|g| {
            g.set_color(0xAA00AA);
            g.fill_triangle(100, 230, 120, 280, 140, 230);
        }),
        "6f39a4b109234e2015719ac56fc2fa1eddc0f456180bda10099a33a5dc136f9c"
    );
}

#[test]
fn fill_triangle_degenerate() {
    assert_eq!(
        render(|g| {
            g.set_color(0xAA0000);
            g.fill_triangle(170, 250, 170, 250, 170, 250);
            g.fill_triangle(180, 250, 190, 250, 200, 250);
        }),
        "1da6de248e6f0675e53d95ca2c4672cd550b1bc03cec99d6cf5b62196d814eb0"
    );
}

#[test]
fn fill_arc_full() {
    // 游戏唯一 fillArc 调用形态（a.java:6635：32x32 全圆加载环）
    assert_eq!(
        render(|g| {
            g.set_color(0x0000FF);
            g.fill_arc(90, 90, 32, 32, 0, 360);
        }),
        "c67fd1174dd85e8ef20fd761dd6cfd091bd05d3798188f77262f5090a6f49e4d"
    );
}

#[test]
fn blend_over() {
    assert_eq!(
        render(|g| {
            g.set_color(0x00CC66);
            g.fill_rect(0, 0, 240, 320);
            let src = translucent_source();
            g.draw_image(&src, 40, 40, anchor::TOP | anchor::LEFT);
            g.draw_image(&src, 60, 60, anchor::TOP | anchor::LEFT);
        }),
        "e47f75d2b7d449a9e157b0764c9215a442f941980cd789585fc9ac0eaf9fb3cc"
    );
}

#[test]
fn draw_image_anchors() {
    assert_eq!(
        render(|g| {
            let src = gradient_source();
            g.draw_image(&src, 10, 10, anchor::TOP | anchor::LEFT);
            g.draw_image(&src, 120, 160, anchor::HCENTER | anchor::VCENTER);
            g.draw_image(&src, 230, 310, anchor::BOTTOM | anchor::RIGHT);
            g.draw_image(&src, 230, 10, anchor::RIGHT | anchor::TOP);
        }),
        "084fe0ae648b97058de78173e47727b19a78a006b4ca2550984fb1e6b3e09abf"
    );
}

#[test]
fn draw_image_transformed() {
    // Nokia DirectGraphics 8 变换码逐一落位（a.java:4539 的消费面）
    assert_eq!(
        render(|g| {
            let src = gradient_source();
            for t in 0..8 {
                g.draw_image_transformed(&src, 20 + t * 26, 40, anchor::TOP | anchor::LEFT, t);
            }
        }),
        "056c77c83f671d7924b4348f4962a0d7be8acd24e6c2ab7545f668f13dab503d"
    );
}

#[test]
fn font_glyphs() {
    assert_eq!(
        render(|g| {
            g.set_color(0x000000);
            g.draw_string(&utf16("囧囧西游 ABC 012"), 20, 40, anchor::TOP | anchor::LEFT);
            g.set_font(Some(SoftFont::get_font(0, font_style::BOLD, font_size::MEDIUM)));
            g.draw_string(&utf16("囧囧 BOLD"), 20, 80, anchor::TOP | anchor::LEFT);
            g.set_font(Some(SoftFont::get_font(0, font_style::PLAIN, font_size::SMALL)));
            g.draw_string(&utf16("small 123"), 20, 110, anchor::TOP | anchor::LEFT);
            g.set_font(Some(SoftFont::get_font(0, font_style::PLAIN, font_size::LARGE)));
            g.draw_string(&utf16("L囧"), 20, 140, anchor::TOP | anchor::LEFT);
        }),
        "e61db4095ac4ca36e17e9b97977bb45cd992e7eca2f807a67d407647c533dac6"
    );
}

#[test]
fn draw_substring() {
    assert_eq!(
        render(|g| {
            g.set_color(0x202020);
            g.draw_substring(&utf16("囧囧西游大闹天宫"), 2, 3, 100, 200, anchor::HCENTER | anchor::BASELINE);
        }),
        "2e509254d822727bb4683ae7b02b099f56dfeb8950148aef96c49a31e0778d20"
    );
}

#[test]
fn draw_char() {
    assert_eq!(
        render(|g| {
            g.set_color(0x101010);
            g.draw_char('囧' as u16, 50, 50, anchor::TOP | anchor::LEFT);
        }),
        "e2f0c447d20d7b69851070af6a549743c9044301680ee64ff480bb33959a86c3"
    );
}
