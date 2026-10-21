//! HD 素材探针·留用件①：density-aware draw 路径（TDD 红证据先行）。
//!
//! 判据来源：`docs/superpowers/specs/2026-10-21-hd-asset-probe-design.md` v1.1
//! （R-0..R-10 吸收节）+ `docs/findings/reviews/2026-10-21-hd-probe-review-pipeline.md`
//! 条件 1-6。四组硬判据（任务卡）：
//!   ① d=1 逐字节恒等（帧哈希 + op 串双侧）——`d1_identity_pin_frame_and_ops`
//!   ② 采样矩阵 d==s / d>s / d<s 各断言像素——`sampling_matrix_*`
//!   ③ transformed × anchor 矩阵——`transformed_*`
//!   ④ op 串 classic/HD 全集相等（R-6）——`op_stream_classic_hd_identical`
//!
//! A 档模型（用户裁定）：素材像素尺寸 = 逻辑尺寸 × density；目标物理格 = 逻辑格 × scale；
//! 源 = 逻辑格 × density；NN 整数相位 `src = l*d/s`（禁浮点）。

use game_core::render::{anchor, transform, ArgbImage, SoftGraphics};
use game_platform::hash::sha256_hex;

// ---------------- 测试源构造 ----------------

/// 逻辑 lw×lh 的确定性内容图：像素 (x,y) → 0xFF000000 | x<<16 | y<<8 | (x*y+7)。
fn content(lw: i32, lh: i32, d: i32) -> ArgbImage {
    let (w, h) = (lw * d, lh * d);
    let mut px = vec![0u32; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let (lx, ly) = (x / d, y / d);
            px[(y * w + x) as usize] =
                0xFF00_0000 | ((lx * 40) << 16) as u32 | ((ly * 40) << 8) as u32 | (lx * ly + 7) as u32;
        }
    }
    ArgbImage::from_rgb(&px, w, h, true).with_density(d)
}

/// 半透明 + 全透明混合源（alpha 混合面）：逻辑 2×2。
fn mixed_alpha(lw: i32, lh: i32, d: i32) -> ArgbImage {
    let (w, h) = (lw * d, lh * d);
    let mut px = vec![0u32; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let (lx, ly) = (x / d, y / d);
            px[(y * w + x) as usize] = match (lx + ly) % 3 {
                0 => 0,                      // 全透明
                1 => 0x8000_FF00,            // 半透明
                _ => 0xFF10_2030,            // 不透明
            };
        }
    }
    ArgbImage::from_rgb(&px, w, h, true).with_density(d)
}

// ---------------- R-5 载体 ----------------

/// 【R-5】density 载体：logical_width()/logical_height() = w/d（默认 d=1 恒等）。
#[test]
fn logical_dims_divide_by_density() {
    let img = ArgbImage::from_rgb(&vec![0xFF00_0000; 24], 6, 4, true);
    assert_eq!((img.logical_width(), img.logical_height()), (6, 4), "默认 density=1 ⇒ 逻辑=像素");
    let img = img.with_density(2);
    assert_eq!((img.width, img.height), (6, 4), "像素尺寸不变");
    assert_eq!((img.logical_width(), img.logical_height()), (3, 2), "w/d、h/d");
    assert_eq!(img.width % img.density, 0, "width%d==0（R-5 断言锚）");
}

/// 【R-5】width%d!=0 拒绝（构造期断言，不许静默截断）。
#[test]
#[should_panic]
fn density_rejects_non_divisible_width() {
    let _ = ArgbImage::from_rgb(&vec![0xFF00_0000; 7], 7, 1, true).with_density(2);
}

// ---------------- R-6 ops 流隔离 ----------------

/// 绘制电池（ops 对拍 + 恒等钉共用）：覆盖 draw_image 锚全组合、transformed 0..8、
/// 半透明混合、clip 裁剪消费面。**只依赖逻辑尺寸**（R-2/R-6），与 density 无关。
fn battery(g: &mut SoftGraphics<'_>, src: &ArgbImage, mix: &ArgbImage) {
    g.set_color(0x204060);
    g.fill_rect(0, 0, 60, 40);
    let anchors = [
        anchor::TOP | anchor::LEFT,
        anchor::TOP | anchor::HCENTER,
        anchor::TOP | anchor::RIGHT,
        anchor::VCENTER | anchor::HCENTER,
        anchor::VCENTER | anchor::RIGHT,
        anchor::BOTTOM | anchor::LEFT,
        anchor::BOTTOM | anchor::HCENTER,
        anchor::BOTTOM | anchor::RIGHT,
    ];
    for (i, &a) in anchors.iter().enumerate() {
        g.draw_image(src, 4 + i as i32 * 5, 3 + i as i32 * 3, a);
    }
    for t in 0..=7 {
        g.draw_image_transformed(src, 10 + t * 3, 30, anchor::TOP | anchor::LEFT, t);
    }
    g.draw_image_transformed(mix, 30, 20, anchor::VCENTER | anchor::HCENTER, transform::ROT90);
    g.draw_image(mix, 2, 2, anchor::TOP | anchor::LEFT);
    g.set_clip(6, 6, 10, 8);
    g.draw_image(src, 4, 4, anchor::TOP | anchor::LEFT);
    g.reset_clip();
}

/// 【R-6】op 串 classic/HD 全集相等：同场景同帧两源（d=1 / d=3）跑出的 ops 序列
/// 逐条恒等 = 换素材零行为漂移证明。op 一律记**逻辑尺寸**。
#[test]
fn op_stream_classic_hd_identical() {
    let run = |src: &ArgbImage, mix: &ArgbImage| {
        let mut screen = ArgbImage::create(64, 48);
        let mut g = SoftGraphics::new(&mut screen);
        battery(&mut g, src, mix);
        g.ops.clone()
    };
    let (ops_c, ops_h) = (
        run(&content(4, 3, 1), &mixed_alpha(2, 2, 1)),
        run(&content(4, 3, 3), &mixed_alpha(2, 2, 3)),
    );
    assert_eq!(ops_c, ops_h, "R-6：classic/HD op 串全集相等");
    // op 记录逻辑尺寸（R-2 :669）——HD 源 12×9 像素仍记 4x3
    assert!(
        ops_h.iter().any(|s| s == "drawImage(4x3,4,3,20)"),
        "drawImage op 记逻辑尺寸 4x3（实际：{:?}）",
        ops_h.iter().find(|s| s.starts_with("drawImage("))
    );
}

// ---------------- R-2 换算收敛 · anchor ----------------

/// 【R-2/攻2】anchor 语义 = 逻辑尺寸×scale（勿混像素尺寸）：HD 源（逻辑 4×2 @d=2）
/// HCENTER|VCENTER 锚 (10,10)、scale=2 ⇒ 物理角点 (10-4/2)×2=16、(10-2/2)×2=18。
/// 旧实现按 img.width*s 入参 ⇒ 角点 (12,14)（偏 d 倍）——此测试在换算收敛前红。
#[test]
fn anchor_uses_logical_dims_at_density() {
    let src = content(4, 2, 2); // 8×4 像素，逻辑 4×2
    let mut screen = ArgbImage::create(64, 48);
    {
        let mut g = SoftGraphics::new(&mut screen);
        g.set_scale(2);
        g.draw_image(&src, 10, 10, anchor::VCENTER | anchor::HCENTER);
    }
    // 物理角点 (16,18)：内容像素 (0,0) 逻辑色 = 0xFF000000|0|0|7
    assert_eq!(screen.argb[18 * 64 + 16], 0xFF00_0007, "HCENTER|VCENTER 角点=逻辑尺寸×scale");
    // 对照：若按像素尺寸入参，角点会落在 (12,14)
    assert_eq!(screen.argb[14 * 64 + 12], 0, "像素尺寸混用的角点必须为空（旧实现错位点）");
}

// ---------------- ① d=1 恒等双侧锚（帧哈希 + op 串） ----------------

/// d=1 绘制电池（逻辑 64×48 画布）：返回 (帧哈希, op 串)。
fn battery_d1(scale: i32) -> (String, String) {
    let mut screen = ArgbImage::create(64 * scale, 48 * scale);
    let ops;
    {
        let mut g = SoftGraphics::new(&mut screen);
        g.set_scale(scale);
        battery(&mut g, &content(4, 3, 1), &mixed_alpha(2, 2, 1));
        ops = g.ops.join("\n");
    }
    (sha256_hex(&screen.hash_stream()), ops)
}

/// 【判据①】d=1 逐字节恒等：帧哈希 + op 串双侧钉住。
///
/// 常量采集自**换算收敛/采样器改造前**的旧实现（2026-10-21，set_pixel_block ×scale
/// 路径，`tmp_capture_pins` 同电池打印后入锚）；其后任何一侧行为漂移 ⇒ 此测试红
/// （R-1 "d=1 退化恒等 …… TDD 双侧帧哈希钉住"）。op 串共同段与 scale 无关
/// （R-6：op 记逻辑尺寸）；尾行 setClip 记画布物理尺寸（scale 换算域，密度无关）。
const D1_OPS_COMMON: &str = "\
setColor(#204060)
fillRect(0,0,60,40,#204060)
drawImage(4x3,4,3,20)
drawImage(4x3,9,6,17)
drawImage(4x3,14,9,24)
drawImage(4x3,19,12,3)
drawImage(4x3,24,15,10)
drawImage(4x3,29,18,36)
drawImage(4x3,34,21,33)
drawImage(4x3,39,24,40)
drawImageT(4x3,10,30,20,0)
drawImageT(4x3,13,30,20,1)
drawImageT(4x3,16,30,20,2)
drawImageT(4x3,19,30,20,3)
drawImageT(4x3,22,30,20,4)
drawImageT(4x3,25,30,20,5)
drawImageT(4x3,28,30,20,6)
drawImageT(4x3,31,30,20,7)
drawImageT(2x2,30,20,3,5)
drawImage(2x2,2,2,20)
setClip(6,6,10,8)
drawImage(4x3,4,4,20)";

#[test]
fn d1_identity_pin_frame_and_ops() {
    for (scale, want_frame) in [
        (1, "a1efb46fe7874bb9237d9188f3f1170e9bfbe9bd5a008ff8c4fa1003a24c2885"),
        (2, "67c022b07572c744ac5f2a75b83aa225059e1a6c247ddff7c5303fb73a5d1e05"),
        (3, "6d2aad8d1f3a518df4b45437a386f27d53be93e2de05e561cc12a549f360a433"),
    ] {
        let want_ops = format!("{D1_OPS_COMMON}\nsetClip(0,0,{},{})", 64 * scale, 48 * scale);
        let (frame, ops) = battery_d1(scale);
        assert_eq!(frame, want_frame, "d=1 scale={scale} 帧哈希（旧↔新双侧）");
        assert_eq!(ops, want_ops, "d=1 scale={scale} op 串（旧↔新双侧）");
    }
}

// ---------------- ② 采样矩阵（R-0 裁定 NN；R-1 整数相位 src=l*d/s） ----------------

/// 物理像素两两可辨的源（含透明槽位）：像素 (x,y) 唯一色。
fn distinct(lw: i32, lh: i32, d: i32) -> ArgbImage {
    let (w, h) = (lw * d, lh * d);
    let mut px = vec![0u32; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            px[(y * w + x) as usize] = if (x + y) % 7 == 3 {
                0
            } else {
                0xFF00_0000 | ((x * 17 + 3) << 16) as u32 | ((y * 29 + 5) << 8) as u32 | (x * y + 11) as u32
            };
        }
    }
    ArgbImage::from_rgb(&px, w, h, true).with_density(d)
}

/// 【②】d==s：1:1 直采零重采样——物理格逐格对齐，逐像素恒等。
#[test]
fn sampling_matrix_direct_copy_d_eq_s() {
    // 逻辑 2×2 @d=2（4×4 像素），scale=2 ⇒ 物理 4×4，源 4×4 直采
    let src = distinct(2, 2, 2);
    let mut screen = ArgbImage::create(32, 24);
    {
        let mut g = SoftGraphics::new(&mut screen);
        g.set_scale(2);
        g.draw_image(&src, 0, 0, anchor::TOP | anchor::LEFT);
    }
    for y in 0..4 {
        for x in 0..4 {
            assert_eq!(screen.argb[y * 32 + x], src.argb[y * 4 + x], "d==s 直采像素({x},{y})");
        }
    }
}

/// 【②】d>s 降采样：NN `src=l*d/s`（整数相位）。逻辑 2×1 @d=3（6×3 像素）、scale=2
/// ⇒ 物理 4×2；dest x∈0..4 → src x = x*3/2 = {0,1,3,4}（源列 2 不被采——R-0 NN
/// 裁定已登记的相位丢点，box 备选见设计件 R-0）。
#[test]
fn sampling_matrix_downsample_d_gt_s() {
    let src = distinct(2, 1, 3);
    let mut screen = ArgbImage::create(32, 24);
    {
        let mut g = SoftGraphics::new(&mut screen);
        g.set_scale(2);
        g.draw_image(&src, 0, 0, anchor::TOP | anchor::LEFT);
    }
    for (dx, sx) in [(0, 0), (1, 1), (2, 3), (3, 4)] {
        for dy in 0..2 {
            let sy = dy * 3 / 2; // 0..2 → {0,1}
            assert_eq!(
                screen.argb[dy * 32 + dx],
                src.argb[sy * 6 + sx],
                "d=3>s=2 降采样 dest({dx},{dy}) ← src({sx},{sy})"
            );
        }
    }
    // 空白区不落笔
    assert_eq!(screen.argb[4], 0, "画布 x≥4 无落笔");
}

/// 【②】d<s 升采样：NN 复制。逻辑 1×1 @d=2（2×2 像素）、scale=3 ⇒ 物理 3×3；
/// dest l∈0..3 → src l*2/3 = {0,0,1}（每轴）。
#[test]
fn sampling_matrix_upsample_d_lt_s() {
    let src = distinct(1, 1, 2);
    let mut screen = ArgbImage::create(32, 24);
    {
        let mut g = SoftGraphics::new(&mut screen);
        g.set_scale(3);
        g.draw_image(&src, 0, 0, anchor::TOP | anchor::LEFT);
    }
    let phase = |l: i32| (l * 2 / 3) as usize; // src=l*d/s
    for dy in 0..3i32 {
        for dx in 0..3i32 {
            assert_eq!(
                screen.argb[(dy * 32 + dx) as usize],
                src.argb[phase(dy) * 2 + phase(dx)],
                "d=2<s=3 升采样 dest({dx},{dy})"
            );
        }
    }
}

/// 【① 采样级】d=1 退化恒等 = `set_pixel_block ×scale`（旧 blit 语义参照实现，
/// 不透明/全透明像素域；半透明混合由 d1_identity_pin 帧哈希钉）。
#[test]
fn d1_degenerates_to_set_pixel_block() {
    for s in [1, 2, 3] {
        let src = distinct(5, 4, 1);
        let mut screen = ArgbImage::create(64 * s, 48 * s);
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_scale(s);
            g.draw_image(&src, 0, 0, anchor::TOP | anchor::LEFT);
        }
        // 旧 blit 参照：逐源像素 s×s 块（set_pixel_block 语义）
        let mut expect = vec![0u32; (64 * s * 48 * s) as usize];
        for sy in 0..4i32 {
            for sx in 0..5i32 {
                let p = src.argb[(sy * 5 + sx) as usize];
                if (p >> 24) & 0xFF == 0 {
                    continue;
                }
                for dy in 0..s {
                    for dx in 0..s {
                        expect[((sy * s + dy) * 64 * s + (sx * s + dx)) as usize] = p;
                    }
                }
            }
        }
        assert_eq!(screen.argb, expect, "d=1 ⇒ set_pixel_block ×scale 逐字节恒等（scale={s}）");
    }
}

// ---------------- R-1 性能门槛（条件 6：全屏重采样 ≤3× 现 blit） ----------------

use std::time::Instant;

/// 旧 blit + set_pixel **逐字节算法参照**（render.rs 改造前路径的测试内复刻）：
/// 逐源像素 s×s set_pixel_block；每物理像素 clip 判断 + alpha 0/255/混合三态
/// ——与新路径同口径的纯像素工作量（含 clip/alpha 阶梯，不偷跑）。
fn legacy_blit_ref(buf: &mut [u32], bw: i32, bh: i32, img: &ArgbImage, x: i32, y: i32, s: i32) {
    for sy in 0..img.height {
        for sx in 0..img.width {
            let p = img.argb[(sy * img.width + sx) as usize];
            if (p >> 24) & 0xFF == 0 {
                continue;
            }
            for dy in 0..s {
                for dx in 0..s {
                    let (px, py) = (x + sx * s + dx, y + sy * s + dy);
                    if px < 0 || py < 0 || px >= bw || py >= bh {
                        continue;
                    }
                    let a = (p >> 24) & 0xFF;
                    if a == 0 {
                        continue;
                    }
                    let idx = (py * bw + px) as usize;
                    if a == 255 {
                        buf[idx] = p;
                        continue;
                    }
                    let dst = buf[idx];
                    let ia = 255 - a;
                    let blend = |c1: u32, c2: u32| (c1 * a + c2 * ia) / 255;
                    let r = blend((p >> 16) & 0xFF, (dst >> 16) & 0xFF);
                    let g = blend((p >> 8) & 0xFF, (dst >> 8) & 0xFF);
                    let b = blend(p & 0xFF, dst & 0xFF);
                    let da = (((dst >> 24) & 0xFF) * ia) / 255 + a;
                    buf[idx] = (da << 24) | (r << 16) | (g << 8) | b;
                }
            }
        }
    }
}

fn bench(reps: usize, mut f: impl FnMut()) -> f64 {
    let mut best = f64::MAX;
    for _ in 0..reps {
        let t = Instant::now();
        f();
        best = best.min(t.elapsed().as_secs_f64());
    }
    best
}

/// 【R-1/条件 6】三段路径全屏耗时 ≤3× 现 blit（同落笔量 480×360 物理像素）：
/// d==s 直采（demo 主场景）/ d>s 降采样 / d<s 升采样；d=1 块写路径=零回退（≤2×）。
/// min-of-N 抗调度噪声；debug 放宽（未优化构建比值失真），release（`--release`
/// 必跑口径）严判。NN 目标 1×（实测见 commit 正文）。
#[test]
fn perf_resample_within_3x_of_legacy_blit() {
    let (lw, lh, s) = (160, 120, 3);
    let (pw, ph) = (lw * s, lh * s);
    let mut screen = ArgbImage::create(pw, ph);
    let mut buf = vec![0u32; (pw * ph) as usize];
    let legacy_t = bench(7, || {
        legacy_blit_ref(&mut buf, pw, ph, &distinct(lw, lh, 1), 0, 0, s)
    });
    for (d, what, bound) in [
        (1, "d=1 块写（classic 零回退）", if cfg!(debug_assertions) { 4.0 } else { 2.0 }),
        (3, "d==s=3 直采", if cfg!(debug_assertions) { 8.0 } else { 3.0 }),
        (2, "d=2<s=3 升采样", if cfg!(debug_assertions) { 8.0 } else { 3.0 }),
        (5, "d=5>s=3 降采样", if cfg!(debug_assertions) { 8.0 } else { 3.0 }),
    ] {
        let src = distinct(lw, lh, d);
        let t = bench(7, || {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_scale(s);
            g.draw_image(&src, 0, 0, anchor::TOP | anchor::LEFT);
        });
        let ratio = t / legacy_t;
        eprintln!("[perf] {what}: {:.3}ms vs legacy {:.3}ms = {ratio:.2}x", t * 1e3, legacy_t * 1e3);
        assert!(
            ratio <= bound,
            "{what}: 重采样/现 blit = {ratio:.2}×（上限 {bound}×；{:.3}ms vs {:.3}ms）",
            t * 1e3,
            legacy_t * 1e3
        );
    }
}
