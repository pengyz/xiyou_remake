//! D2 字体缝共享实现（双壳消费不拷贝——2026-10-26 自 shell/win `font_inject` 迁入，
//! 供 Android 字体注入）：系统 TTF/TTC → [`game_core::render::set_glyph_hook`] 真字形。
//!
//! 布局仍用 SoftFont 格宽/格高（与差分验证过的排版一致），仅字形换真；无可用字体 =
//! **不装钩子**（game-core 差分占位字形原样保留=真回退）。测试进程从不注入全局钩子
//! （差分不受影响）；本模块测试只测纯光栅面 [`raster_glyph`]。

use ab_glyph::{point, Font, FontVec, PxScale};
use std::sync::OnceLock;

static FONT: OnceLock<Option<FontVec>> = OnceLock::new();

/// 字体候选：(路径, TTC 面索引；单字体文件=0)。
pub type Candidate = (String, u32);

/// Android 系统字体候选（AOSP /system/fonts；fonts.xml zh-Hans 族）。
/// `NotoSansCJK-Regular.ttc` 面索引 **2 = Noto Sans CJK SC**（实机 TTC name 表实测：
/// 0 JP / 1 KR / 2 SC / 3 TC / 4 HK——取 0 会用日文字形变体）。
pub fn android_candidates() -> Vec<Candidate> {
    [
        ("/system/fonts/NotoSansCJK-Regular.ttc", 2),
        ("/system/fonts/NotoSansSC-Regular.otf", 0),
        ("/system/fonts/DroidSansFallbackFull.ttf", 0),
        ("/system/fonts/DroidSansFallback.ttf", 0),
    ]
    .iter()
    .map(|(p, i)| (p.to_string(), *i))
    .collect()
}

/// 按序取首个可读且可解析的字体：Some((路径, 字体))；全不可用=None。
/// `log`=逐候选诊断（解析失败原因 / 选中源）。
pub fn load(cands: &[Candidate], log: &mut dyn FnMut(&str)) -> Option<(String, FontVec)> {
    for (path, index) in cands {
        if let Ok(bytes) = std::fs::read(path) {
            match FontVec::try_from_vec_and_index(bytes, *index) {
                Ok(f) => {
                    log(&format!("字形源：{path}#{index}"));
                    return Some((path.clone(), f));
                }
                Err(e) => log(&format!("字体 {path}#{index} 不可用（{e}），试下一个")),
            }
        }
    }
    log("未找到 CJK 字体——文字保持差分占位字形");
    None
}

/// 注入真字形钩子（进程一次；重复调用沿用首次结果）。返回选中路径；None=无字体
/// （不装钩子=占位真回退——旧"无条件装 hook 且 hook 内 return"=文字空白的假回退）。
pub fn install(cands: &[Candidate], log: &mut dyn FnMut(&str)) -> Option<String> {
    let mut chosen = None;
    let _ = FONT.get_or_init(|| {
        load(cands, log).map(|(p, f)| {
            chosen = Some(p);
            f
        })
    });
    if FONT.get().map(Option::is_some).unwrap_or(false) {
        game_core::render::set_glyph_hook(Some(hook_fn));
    }
    chosen
}

fn hook_fn(
    ch: u16,
    px: i32,
    py: i32,
    _cw: i32,
    chh: i32,
    color: u32,
    put: &mut dyn FnMut(i32, i32, u32),
    scale: i32,
) {
    let Some(font) = FONT.get().and_then(|f| f.as_ref()) else { return };
    for (x, y) in raster_glyph(font, ch, px, py, chh, scale) {
        put(x, y, color);
    }
}

/// 单字形 → 物理像素点（task-13 ②：按 **scale×格高** 光栅化、笔画直落物理坐标=真 TTF
/// 锐度，非 1× 落点放大色块）。入参逻辑坐标（格左上 px/py、格高 chh）；基线放格底
/// 上方 1px（CJK 无下伸部，格内视觉居中）。
pub fn raster_glyph(font: &FontVec, ch: u16, px: i32, py: i32, chh: i32, scale: i32) -> Vec<(i32, i32)> {
    let ss = scale.max(1);
    let s = PxScale { x: (chh * ss) as f32, y: (chh * ss) as f32 };
    let glyph = font
        .glyph_id(char::from_u32(ch as u32).unwrap_or(' '))
        .with_scale_and_position(s, point((px * ss) as f32, ((py + chh - 1) * ss) as f32));
    let Some(outline) = font.outline_glyph(glyph) else { return Vec::new() };
    // ab_glyph 回调坐标是 px_bounds **局部**坐标（outlined.rs draw() 文档："pixel
    // coordinate inside the bounds"），须平移 bbox 最小角才是画布绝对坐标——否则字形
    // 全堆在 (0,0)（mode21 文字不可见 bug）。
    let bb = outline.px_bounds();
    let (ox, oy) = (bb.min.x.round() as i32, bb.min.y.round() as i32);
    let mut dots: Vec<(i32, i32, f32)> = Vec::new();
    outline.draw(|gx, gy, a| dots.push((gx as i32, gy as i32, a)));
    dots_to_physical(&dots, ox, oy)
}

/// 字形笔画 → 物理像素（alpha>0.25 阈；局部坐标 + bbox 最小角平移）。
pub fn dots_to_physical(dots: &[(i32, i32, f32)], ox: i32, oy: i32) -> Vec<(i32, i32)> {
    let mut out: Vec<(i32, i32)> = dots
        .iter()
        .filter(|(_, _, a)| *a > 0.25)
        .map(|&(gx, gy, _)| (ox + gx, oy + gy))
        .collect();
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 宿主同族字体（Ubuntu fonts-noto-cjk；TTC 面序与 AOSP 同）。缺失=显式 SKIP。
    const HOST_NOTO: &str = "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc";

    fn host_font() -> Option<FontVec> {
        let f = load(&[(HOST_NOTO.to_string(), 2)], &mut |_| {}).map(|(_, f)| f);
        if f.is_none() {
            eprintln!("SKIP：宿主无 {HOST_NOTO}（apt fonts-noto-cjk）——光栅判据未执行");
        }
        f
    }

    #[test]
    fn android_candidates_prefer_noto_cjk_sc_face() {
        let c = android_candidates();
        assert_eq!(c[0], ("/system/fonts/NotoSansCJK-Regular.ttc".to_string(), 2), "首选 SC 面（非 0=JP）");
        assert!(c.iter().all(|(p, _)| p.starts_with("/system/fonts/")), "只读系统字体目录");
    }

    #[test]
    fn load_skips_missing_and_picks_first_valid() {
        let mut logs = Vec::new();
        assert!(load(&[("/nonexistent/x.ttf".into(), 0)], &mut |m| logs.push(m.to_string())).is_none());
        assert!(logs.last().unwrap().contains("未找到"), "全缺失=显式日志");
        if std::path::Path::new(HOST_NOTO).exists() {
            let got = load(&[("/nonexistent/x.ttf".into(), 0), (HOST_NOTO.into(), 2)], &mut |_| {});
            assert_eq!(got.map(|(p, _)| p).as_deref(), Some(HOST_NOTO), "跳过缺失取首个可用");
        }
    }

    #[test]
    fn raster_glyph_lands_in_cell_at_physical_scale() {
        let Some(font) = host_font() else { return };
        let (px, py, chh) = (10, 20, 12);
        let one = raster_glyph(&font, 0x5F00, px, py, chh, 1); // 开
        let two = raster_glyph(&font, 0x5F00, px, py, chh, 2);
        assert!(one.len() > 20, "1× 有笔画（实得 {}）", one.len());
        // 物理光栅化：2× 笔画像素≈1× 的 4 倍（非 1× 落点放大——那会恰为 4 倍块）
        let ratio = two.len() as f32 / one.len() as f32;
        assert!((2.5..6.0).contains(&ratio), "2× 物理光栅化（面积比 {ratio:.2}）");
        // 落点在格内（物理域，容 1px 抗锯齿溢出）
        for &(x, y) in &two {
            assert!(x >= px * 2 - 1 && x <= (px + chh) * 2 + 1, "x={x} 出格");
            assert!(y >= py * 2 - 1 && y <= (py + chh) * 2 + 1, "y={y} 出格");
        }
    }
}
