//! 风格样张·实机效果（风格迁移 v2：Q 版形象×胧村正华丽画工）。
//!
//! 用户裁定（2026-10-22）：Q 版形象不动，迁移香草社"华丽画工"（柔喷薄影/鎏金
//! 质感/饱和石色/丰润光感）；高清分辨率素材（density 通路已验）。本件把
//! `assets/hd-demo/style-sample/wukong_v2_hos.png`（246×276 像素 = 逻辑 41×46
//! × density 6）经 **density 直采样** 插入真游戏场景（maplv5 相机锚同 hd_visual），
//! 产出实机截图给用户判读。**弃件/demo 用途**（画风定调样张，不进保真面）。
//!
//! 产出：`assets/hd-demo/style-sample/ingame_{hd,classic}_tiles.png`（scale=3）。

use std::path::{Path, PathBuf};

use game_core::game_view::{GameView, ViewDims};
use game_core::render::{anchor, ArgbImage, SoftGraphics};

const SCALE: i32 = 3;
const MAPLV: usize = 5;
const PLAYER_PX: i32 = 248;
const PLAYER_PY: i32 = 126;
const TILE_OFF: i32 = 20;

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel)
}

fn decode_png_file(path: &Path, hint: &str) -> ArgbImage {
    let bytes = std::fs::read(path)
        .unwrap_or_else(|e| panic!("缺 {path:?}（{hint}）：{e}"));
    ArgbImage::from_decoded_png(
        game_data::decode_png(&bytes).unwrap_or_else(|e| panic!("{path:?} 解码失败：{e}")),
    )
}

fn map_frames() -> Vec<ArgbImage> {
    let data = std::fs::read(repo("assets/raw/map"))
        .unwrap_or_else(|e| panic!("缺 assets/raw/map（先跑 tools/extract-assets/extract.py）：{e}"));
    game_data::PackedPng::parse(&data, 12)
        .unwrap_or_else(|e| panic!("map 容器解析失败：{e}"))
        .images
        .iter()
        .map(|sub| ArgbImage::from_decoded_png(game_data::decode_png(sub.bytes).unwrap()))
        .collect()
}

fn one_frame(name: &str, count: usize, idx: usize) -> ArgbImage {
    let data = std::fs::read(repo(&format!("assets/raw/{name}")))
        .unwrap_or_else(|e| panic!("缺 assets/raw/{name}：{e}"));
    let subs = game_data::PackedPng::parse(&data, count)
        .unwrap_or_else(|e| panic!("{name} 容器解析失败：{e}"));
    ArgbImage::from_decoded_png(game_data::decode_png(subs.images[idx].bytes).unwrap())
}

fn build_view() -> GameView {
    let data = std::fs::read(repo(&format!("assets/raw/maplv{MAPLV}")))
        .unwrap_or_else(|e| panic!("缺 assets/raw/maplv{MAPLV}：{e}"));
    let mut v = GameView::from_maplv(&data, PLAYER_PX, PLAYER_PY, ViewDims::classic())
        .unwrap_or_else(|e| panic!("maplv{MAPLV} 解析失败：{e}"));
    v.center_on_player();
    v
}

/// v2 风格样张悟空（Q 版×胧村正华丽）：246×276 像素=逻辑 41×46 × density 6。
fn wukong_v2() -> ArgbImage {
    decode_png_file(
        &repo("assets/hd-demo/style-sample/wukong_v2_hos.png"),
        "先跑 python3 tools/style-sample/muramasa_wukong_v2.py",
    )
    .with_density(6)
}

/// 场景渲染：真瓦片（classic d=1 或 demo d=3）+ 撒瓦片 + 新悟空立于画面中带。
/// （角色锚：BOTTOM|CENTER，落点=视口中带偏下——真机 paint_player 同款足底锚。）
fn render_scene(w: i32, h: i32, tileset: &ArgbImage, frames: &[ArgbImage], wukong: &ArgbImage) -> Vec<u32> {
    let mut canvas = ArgbImage::create(w * SCALE, h * SCALE); // 物理画布（scale 倍）
    {
        let mut g = SoftGraphics::new(&mut canvas);
        g.set_scale(SCALE);
        let v = build_view();
        // 瓦片层（同 hd_visual：行距 32 列距 2，tile_off=view_top）
        for row in 0..18 {
            for col in 0..12 {
                let idx = ((row * 12 + col) % 104) as usize; // 真帧序旋转取样（demo 语义不承诺）
                let _ = idx;
                let _ = frames;
                let tx = col * 32 - (v.cam_x % 32) - 32;
                let ty = row * 32 - (v.cam_y % 32) - 32 + TILE_OFF;
                g.draw_image(tileset, tx, ty, anchor::TOP | anchor::LEFT);
            }
        }
        // 数字条（非素材区装饰，同 hd_visual 电池节选）
        g.draw_image(&frames[3], 10, 4, anchor::TOP | anchor::LEFT);
        // —— 新悟空（Q 版×胧村正华丽，d=6 直采样）——
        // 足底锚：逻辑 (123, 236) = 画面中带偏下（角色立于瓦片上）
        let px = 123 - 41 / 2;
        let py = 236 - 46;
        // 影（同款柔影椭圆 sptprop[1] 或矢量椭圆——用矢量，弃件语义）
        // 足底影（fill_ellipse 不存在——用 fill_rect 阶梯近似，探索草稿件）
        for i in 0..5 {
            g.fill_rect(px + 12 + i, 238 - i / 2, 24 - 2 * i, 2 + i % 2);
        }
        g.draw_image(wukong, px, py, anchor::TOP | anchor::LEFT);
    }
    canvas.argb.clone()
}

// ---------------- 最小 PNG 写出（hd_visual 同构） ----------------

fn encode_png(width: i32, height: i32, pixels: &[u32]) -> Vec<u8> {
    let mut raw = Vec::with_capacity((height as usize) * (1 + width as usize * 4));
    for y in 0..height as usize {
        raw.push(0u8);
        for x in 0..width as usize {
            let p = pixels[y * width as usize + x];
            raw.push(((p >> 16) & 0xFF) as u8);
            raw.push(((p >> 8) & 0xFF) as u8);
            raw.push((p & 0xFF) as u8);
            raw.push(((p >> 24) & 0xFF) as u8);
        }
    }
    let mut z = vec![0x78, 0x01];
    let mut off = 0usize;
    while off < raw.len() {
        let n = (raw.len() - off).min(65535);
        let last = if off + n >= raw.len() { 1u8 } else { 0u8 };
        z.push(last);
        z.extend_from_slice(&(n as u16).to_le_bytes());
        z.extend_from_slice(&(!(n as u16)).to_le_bytes());
        z.extend_from_slice(&raw[off..off + n]);
        off += n;
    }
    let (mut a, mut b) = (0u32, 0u32);
    for &x in &raw {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let chunk = |out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]| {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend_from_slice(kind);
        out.extend_from_slice(data);
        let mut c = 0xFFFF_FFFFu32;
        for &x in kind.iter().chain(data.iter()) {
            c ^= x as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
            }
        }
        out.extend_from_slice(&(!c).to_be_bytes());
    };
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &z);
    chunk(&mut png, b"IEND", &[]);
    png
}

fn write_png(dir: &Path, name: &str, w: i32, h: i32, px: &[u32]) {
    std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("建 {dir:?} 失败：{e}"));
    let p = dir.join(name);
    std::fs::write(&p, encode_png(w, h, px)).unwrap_or_else(|e| panic!("写 {p:?} 失败：{e}"));
    eprintln!("[style-sample] {name}：{}×{} → {}", w, h, p.display());
}

#[test]
fn ingame_wukong_v2_hos() {
    let frames = map_frames();
    let wukong = wukong_v2();
    let out = repo("assets/hd-demo/style-sample");
    // ① classic 瓦片 + 新悟空（角色效果隔离）
    let px = render_scene(240, 320, &frames[0], &frames, &wukong);
    write_png(&out, "ingame_classic_tiles.png", 240 * SCALE, 320 * SCALE, &px);
    // ② HD demo 瓦片 + 新悟空（全高清预览）
    let hd = decode_png_file(&repo("assets/hd-demo/map_00_3x.png"), "先跑 tools/hd-demo/gen_hd_demo.py")
        .with_density(3);
    let px2 = render_scene(240, 320, &hd, &frames, &wukong);
    write_png(&out, "ingame_hd_tiles.png", 240 * SCALE, 320 * SCALE, &px2);
}
