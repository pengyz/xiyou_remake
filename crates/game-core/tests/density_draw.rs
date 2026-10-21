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

/// 基准测度（2026-10-21 更正后口径）：**预热 2 轮 + N 轮逐对交替、每侧 3 连测
/// 取 min、对比率取中位数**。原实现=两段分离 min-of-7 wall-clock——并发负载下
/// 段间偏置抖动（集成闸实测 d=1 2.19× 红、67 负载下 d=5 尖峰 3.5-7× 复现）；
/// 3 连测 min 吞单次抢占、中位数吞坏轮、交替对计时共享负载条件。
fn median_ratio(rounds: usize, mut legacy: impl FnMut(), mut hd: impl FnMut()) -> (f64, f64, f64) {
    // 每轮每侧 3 次连测取 min（吞掉单次调度抢占），轮间交替（共享负载）。
    let triple = |f: &mut dyn FnMut()| {
        let mut best = f64::MAX;
        for _ in 0..3 {
            let t = Instant::now();
            f();
            best = best.min(t.elapsed().as_secs_f64().max(1e-12));
        }
        best
    };
    for _ in 0..2 {
        triple(&mut legacy);
        triple(&mut hd);
    }
    let (mut ratios, mut ls, mut hs) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..rounds {
        let lt = triple(&mut legacy);
        let ht = triple(&mut hd);
        ratios.push(ht / lt);
        ls.push(lt);
        hs.push(ht);
    }
    let mid = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    (mid(&mut ratios), mid(&mut ls), mid(&mut hs))
}

/// 【R-1/条件 6】分臂性能判据（同落笔量 480×360 物理像素）：
/// - **d=1 臂 ≤2×**：走旧 set_pixel_block 块写**原码**（R-1 明文保留），
///   与旧 blit 同码 ⇒ 稳态恒 1×，2× 为宽裕噪声带——R-1 判据① 恒等锚的
///   **性能投影**（classic 热路径零回退锐度；若放宽到 3× 则 d=1 退化 2 倍
///   都能过 = 判据失明）；
/// - **采样臂（d==s/d>s/d<s）≤3×**：2/3 条件 6/R-1 原文「性能门槛：全屏
///   重采样实测 ≤3× 现 blit（NN 目标 1×）」——该口径只属全屏重采样。
///
/// 【2026-10-21 判据勘误留痕·历史主张原地保留不删】①初版 d=1 臂 2×/其余
/// 3× 曾于集成闸并发负载下判红（实测 2.19×；非产品回退——d=1 与旧 blit
/// 同码恒等）；②lead 派工一度误用 3× 口径于 d=1 臂（统一 ≤3× 版本），
/// verifier 阶段二拦截更正回**分臂口径**（本注记=更正后措辞）。
/// 测量方法修正保留：预热 2 轮 + 11 轮逐对交替、每侧 3 连测取 min、
/// 对比率取中位数（治 flaky 根因）。
#[test]
fn perf_resample_within_3x_of_legacy_blit() {
    let (lw, lh, s) = (160, 120, 3);
    let (pw, ph) = (lw * s, lh * s);
    let mut screen = ArgbImage::create(pw, ph);
    let mut buf = vec![0u32; (pw * ph) as usize];
    for (d, what, bound) in [
        // d=1 臂 ≤2×：判据①恒等锚的性能投影（零回退锐度）
        (1, "d=1 块写（classic 路径）", 2.0),
        // 采样臂 ≤3×：2/3 条件 6（全屏重采样口径）
        (3, "d==s=3 直采", 3.0),
        (2, "d=2<s=3 升采样", 3.0),
        (5, "d=5>s=3 降采样", 3.0),
    ] {
        let src = distinct(lw, lh, d);
        let (ratio, lt, ht) = median_ratio(
            11,
            || legacy_blit_ref(&mut buf, pw, ph, &distinct(lw, lh, 1), 0, 0, s),
            || {
                let mut g = SoftGraphics::new(&mut screen);
                g.set_scale(s);
                g.draw_image(&src, 0, 0, anchor::TOP | anchor::LEFT);
            },
        );
        eprintln!(
            "[perf] {what}: 中位数 {ratio:.2}x（hd {:.3}ms / legacy {:.3}ms，11 轮中位）",
            ht * 1e3,
            lt * 1e3
        );
        assert!(
            ratio <= bound,
            "{what}: 重采样/现 blit 中位数 = {ratio:.2}×（分臂上限 {bound}×）"
        );
    }
}

// ---------------- ③ transformed × anchor 矩阵（R-3：8 变换表逻辑域不动） ----------------

/// 独立参照：正交变换的**物理域**逆映射——dest (u,v) ← src T⁻¹(u,v)。
/// d==s 时与"逻辑格网变换 + 块内相位"逐像素一致（R-3 判据）；d=1 退化块写恒等。
fn inv_transform_px(t: i32, u: i32, v: i32, lw_px: i32, lh_px: i32) -> (i32, i32) {
    match t {
        1 | 16384 => (u, lh_px - 1 - v),
        2 | 8192 => (lw_px - 1 - u, v),
        3 | 24576 => (lw_px - 1 - u, lh_px - 1 - v),
        4 => (v, u),
        5 => (v, lh_px - 1 - u),
        6 => (lw_px - 1 - v, u),
        7 => (lw_px - 1 - v, lh_px - 1 - u),
        _ => (u, v),
    }
}

/// anchor → 角点偏移（Graphics.anchorAdjust 语义：w/2、w、h/2、h）。
fn anchor_adj(a: i32, w: i32, h: i32) -> (i32, i32) {
    use game_core::render::anchor::*;
    let ax = if a & HCENTER != 0 { w / 2 } else if a & RIGHT != 0 { w } else { 0 };
    let ay = if a & VCENTER != 0 { h / 2 } else if a & BOTTOM != 0 { h } else { 0 };
    (ax, ay)
}

/// 【③】8 变换 × 8 锚全矩阵 @d==s：逐像素 = 物理域精确几何变换，
/// 角点 = 逻辑 bbox×scale（R-2/R-3）。d==s 直采下变换=正交同构零失真。
#[test]
fn transformed_anchor_matrix_d_eq_s_exact() {
    let (lw, lh, d, s) = (3, 2, 2, 2);
    let src = distinct(lw, lh, d); // 6×4 像素
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
    for t in 0..=7i32 {
        for &a in &anchors {
            let mut screen = ArgbImage::create(64, 48);
            {
                let mut g = SoftGraphics::new(&mut screen);
                g.set_scale(s);
                g.draw_image_transformed(&src, 10, 9, a, t);
            }
            let swap = (4..=7).contains(&t);
            let (bw, bh) = (if swap { lh } else { lw }, if swap { lw } else { lh });
            let (adj_x, adj_y) = anchor_adj(a, bw * s, bh * s);
            let (ox, oy) = (10 * s - adj_x, 9 * s - adj_y);
            for v in 0..bh * s {
                for u in 0..bw * s {
                    let (sx, sy) = inv_transform_px(t, u, v, lw * d, lh * d);
                    let want = src.argb[(sy * lw * d + sx) as usize];
                    assert_eq!(
                        screen.argb[((oy + v) * 64 + (ox + u)) as usize],
                        want,
                        "t={t} anchor={a} dest({u},{v}) @({},{})",
                        ox + u,
                        oy + v
                    );
                }
            }
            // bbox 外不落笔（角点错位旧病灶点）
            assert_eq!(
                screen.argb[((oy - 1) * 64 + ox) as usize],
                0,
                "t={t} anchor={a} bbox 上沿外须空"
            );
            assert_eq!(
                screen.argb[(oy * 64 + ox - 1) as usize],
                0,
                "t={t} anchor={a} bbox 左沿外须空"
            );
        }
    }
}

/// 【③】诺基亚 FLIP 常量（8192/16384/24576）与 2/1/3 同像（shim 同步实现，
/// a.java:44 f_int_arr_01 值域）。
#[test]
fn transformed_flip_constants_d_eq_s_exact() {
    let (lw, lh, d, s) = (3, 2, 2, 2);
    let src = distinct(lw, lh, d);
    for t in [8192i32, 16384, 24576] {
        let mut screen = ArgbImage::create(64, 48);
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_scale(s);
            g.draw_image_transformed(&src, 0, 0, anchor::TOP | anchor::LEFT, t);
        }
        for v in 0..lh * s {
            for u in 0..lw * s {
                let (sx, sy) = inv_transform_px(t, u, v, lw * d, lh * d);
                assert_eq!(
                    screen.argb[(v * 64 + u) as usize],
                    src.argb[(sy * lw * d + sx) as usize],
                    "FLIP t={t} dest({u},{v})"
                );
            }
        }
    }
}

/// 【③·未闭环 3/3 C1】d≠s transformed：落笔块内像素必须来自正确的源格
/// d×d 像素集（合法性覆盖，不钉块内相位——块内相位属未闭环面，待真素材复验）。
#[test]
fn transformed_density_validity_d_ne_s() {
    let (lw, lh, d, s) = (2, 3, 3, 2); // d>s；逻辑 2×3 ⇒ 源 6×9、落笔 4×6
    let src = distinct(lw, lh, d);
    for t in 0..=7i32 {
        let mut screen = ArgbImage::create(64, 48);
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_scale(s);
            g.draw_image_transformed(&src, 0, 0, anchor::TOP | anchor::LEFT, t);
        }
        let swap = (4..=7).contains(&t);
        let (bw, bh) = (if swap { lh } else { lw }, if swap { lw } else { lh });
        for v in 0..bh * s {
            for u in 0..bw * s {
                let got = screen.argb[(v * 64 + u) as usize];
                // 该目的格反查源格（逻辑域表），像素须落在该源格 d×d 集合内
                let (dx, dy) = (u / s, v / s);
                let (lx, ly) = match t {
                    1 | 16384 => (dx, lh - 1 - dy),
                    2 | 8192 => (lw - 1 - dx, dy),
                    3 | 24576 => (lw - 1 - dx, lh - 1 - dy),
                    4 => (dy, dx),
                    5 => (dy, lh - 1 - dx),
                    6 => (lw - 1 - dy, dx),
                    7 => (lw - 1 - dy, lh - 1 - dx),
                    _ => (dx, dy),
                };
                let mut ok = false;
                for b in 0..d {
                    for a in 0..d {
                        if src.argb[((ly * d + b) * lw * d + lx * d + a) as usize] == got {
                            ok = true;
                        }
                    }
                }
                assert!(ok, "t={t} dest({u},{v}) 落笔像素必须来自源格({lx},{ly}) 的 d×d 集合（未闭环面只验合法性）");
            }
        }
    }
}

// ---------------- ④ 图集切片消费面（R-4：索引语义逻辑域不动，仅像素边界 ×d） ----------------

use game_core::game_view::{GameView, ViewDims};
use game_core::menu_family::draw_edge_patch;

/// 逻辑 256×256 大图集（d=2 ⇒ 512×512 像素）：kind 5-7 的 (ih-sy-h)/(iw-sx-w)
/// 偏移在界内（128 宽 map 图集上 kind5 X 偏移 176 越界=shim 原样绘制既有形态）。
fn big_atlas(d: i32) -> ArgbImage {
    distinct(256, 256, d)
}

/// 【④】draw_edge_patch kind 0..7 @d==s：切片偏移逻辑域（(v&7)<<4 格距不动）、
/// 只对像素边界 ×d——窗口 16×16 逻辑（32×32 物理）逐像素 = 图集对应区域
/// （kind 1-3 = FLIP 码 8192/16384/24576 镜像；kind 4-7 = shim 原样绘制、
/// 仅坐标公式生效——转置偏移取 (sy,sx)/(ih-sy-h)/(iw-sx-w) 起点）。
#[test]
fn atlas_slice_edge_patch_kinds_d_eq_s() {
    let (d, s) = (2, 2);
    let ts = big_atlas(d);
    // (sx,sy) = (v&7)<<4, (v>>3)<<4 —— v=10 ⇒ (32,16)（陷阱①索引语义）
    let (sx, sy, w, h) = (32, 16, 16, 16);
    let (iw, ih) = (256, 256);
    for kind in 0..=7i32 {
        let mut screen = ArgbImage::create(128, 128);
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_scale(s);
            draw_edge_patch(&mut g, &ts, 10, 10, sx, sy, w, h, kind);
        }
        // 各 kind 的窗口物理 (u,v) ← 图集物理坐标（手推自 a.java:4577-4620 坐标式；
        // d==s ⇒ 1:1 直采零相位失真）
        for v in 0..h * s {
            for u in 0..w * s {
                let (ax, ay) = match kind {
                    1 => (64 + 31 - u, 32 + v),
                    2 => (64 + u, 32 + 31 - v),
                    3 => (64 + 31 - u, 32 + 31 - v),
                    4 => (32 + u, 64 + v),
                    5 => (448 + u, 64 + v),
                    6 => (32 + u, 416 + v),
                    7 => (448 + u, 416 + v),
                    _ => (64 + u, 32 + v),
                };
                assert_eq!(
                    screen.argb[((20 + v) * 128 + (20 + u)) as usize],
                    ts.argb[(ay * 512 + ax) as usize],
                    "kind={kind} 窗口({u},{v}) ← 图集物理({ax},{ay})"
                );
            }
        }
        // clip 外不落笔（切片=clip+整图偏移，密度只影响像素边界）
        assert_eq!(screen.argb[(19 * 128 + 20) as usize], 0, "kind={kind} clip 上沿外");
        assert_eq!(screen.argb[(20 * 128 + 19) as usize], 0, "kind={kind} clip 左沿外");
    }
}

/// 【④】paint_tiles 全链（(v&7)<<4 / (v>>3)<<4 切片 → draw_edge_patch → 密度
/// 直采）@d==s：可见瓦片逐像素 = 图集对应区域，v=0 跳过，transform=1 走 FLIP。
#[test]
fn atlas_paint_tiles_hd_tileset_pixel_exact() {
    // 4×4 格合成 maplv（头 u16LE=8 ⇒ wide=high=4；grid 64+64）
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&8u16.to_le_bytes());
    bytes.extend_from_slice(&8u16.to_le_bytes());
    let gl = (4 * 4 * 4) as usize;
    let mut terrain = vec![0u8; gl];
    let mut transform = vec![0u8; gl];
    terrain[0 * 8 + 0] = 2; // (r0,c0) v=2 ⇒ sx=32, sy=0
    terrain[0 * 8 + 1] = 9; // (r0,c1) v=9 ⇒ sx=16, sy=16
    terrain[1 * 8 + 0] = 0; // v=0 跳过
    terrain[1 * 8 + 1] = 3; // (r1,c1) v=3 ⇒ sx=48, sy=0；transform=1（FLIP）
    transform[1 * 8 + 1] = 1;
    bytes.extend_from_slice(&terrain);
    bytes.extend_from_slice(&transform);
    let v = GameView::from_maplv(&bytes, 0, 0, ViewDims::classic()).unwrap();

    let (d, s) = (2, 2);
    let ts = distinct(8 * 16, 13 * 16, d); // 逻辑 128×208 图集
    let mut screen = ArgbImage::create(240 * s, 252 * s);
    let (cam_x, cam_y) = {
        let mut g = SoftGraphics::new(&mut screen);
        g.set_scale(s);
        v.paint_tiles(&mut g, &ts, 0);
        (v.cam_x, v.cam_y)
    };
    for (r, c, v_tile, flipped) in [(0, 0, 2, false), (0, 1, 9, false), (1, 1, 3, true)] {
        let (tile_x, tile_y) = (cam_x + c * 16, cam_y + r * 16);
        let (sx, sy) = (((v_tile & 7) << 4) * d, ((v_tile >> 3) << 4) * d); // 像素边界 ×d
        for u in 0..16 * s {
            for w2 in 0..16 * s {
                let (ax, ay) = if flipped {
                    (sx + 31 - u, sy + w2) // transform=1（MIRROR_ROT180=垂直翻转）
                } else {
                    (sx + u, sy + w2)
                };
                assert_eq!(
                    screen.argb[((tile_y * s + w2) * 240 * s + (tile_x * s + u)) as usize],
                    ts.argb[(ay * 256 + ax) as usize],
                    "瓦片(r{r},c{c}) v={v_tile} 像素({u},{w2})"
                );
            }
        }
    }
    // v=0 格（r1,c0）不落笔（画布初值 0）
    assert_eq!(screen.argb[(((cam_y + 16) * s) * 240 * s + cam_x * s) as usize], 0, "v=0 格须空");
}

/// 【④/陷阱①】宽度型断言换算 width/(16*d)==8：列数 8 = (v&7) 索引语义，
/// 不随 d 缩放（列数 8 断言保持 legacy 语义）；只对像素边界 ×d。
#[test]
fn atlas_column8_index_semantics_unscaled() {
    for d in [1, 2, 3] {
        let ts = distinct(8 * 16, 13 * 16, d);
        assert_eq!(ts.width, 8 * 16 * d, "像素宽=8×16×d（边界 ×d）");
        assert_eq!(ts.width / (16 * ts.density), 8, "列数 8 锚（width/(16*d)==8）");
        assert_eq!(ts.height / (16 * ts.density), 13, "行数 13");
        // 陷阱①断言锚不动：索引语义 = 逻辑域常量
        let v = 2u8;
        assert_eq!(((v & 7) << 4, (v >> 3) << 4), (32, 0), "sx/sy 不随 d 缩放");
    }
}
