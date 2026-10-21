//! 验证件③：无头 scale=3 双截图对照 harness（HD 素材探针，R-8 验收两层）。
//!
//! 判据来源：`docs/superpowers/specs/2026-10-21-hd-asset-probe-design.md` v1.1/v1.2
//! （R-6 ops 隔离、R-7 弃件隔离、R-8 验收两层）+ `docs/findings/reviews/2026-10-21-hd-probe-review-fidelity.md`
//! C4（截图证据级别）。
//!
//! 同场景同相机同帧、scale=3 渲染两遍——①classic 原图集（assets/raw/map 帧00，d=1）
//! ②demo 图集（assets/hd-demo/map_00_3x.png，d=3）——**对照变量唯一**（仅贴图源不同，
//! 场景/相机/帧/overlay/底色逐字节同配置）。输出双 PNG 供人工判读。
//!
//! # 证据分级（R-8 / AGENTS §3）
//!
//! - **机器判据 = A 级**（本文件断言）：非素材区（HUD/精灵/文字）逐像素恒等、
//!   op 串全集相等、demo 图集帧序/帧数、d==s 直采样恒等、逐格差异⟺restyle 帧。
//! - **PNG 截图人工判读 = C 级**（对照说明 docs/reviews/2026-10-21-hd-visual-compare.md）：
//!   本件**不建闸**（visual 闸 SKIP），判读≠验证；"demo 画丑但管线对"与"管线错"的
//!   区分 = 机器判据全绿 ⇒ 管线对（R-8）。
//! - scale=3 **必选**：模糊根源=scale≥2 最近邻放大，scale=1 对照=假阴性（清晰度
//!   差异在 1x 不可见）。
//!
//! 无头纪律（AGENTS §5 验证不打扰）：纯渲染测试，零窗口/零用户桌面交互。
//! PNG 产出 = `assets/hd-demo/compare/`（gitignore 区，弃件 R-7）。

use std::path::{Path, PathBuf};

use game_core::game_view::{GameView, ViewDims};
use game_core::render::{anchor, ArgbImage, SoftGraphics};

// ---------------- 场景锚（同场景同相机同帧——常量断言钉住） ----------------

/// scale=3 必选（R-8：scale=1 对照=假阴性）。
const SCALE: i32 = 3;
/// 场景段 = maplv5 相机 (-144,-16) 窗口：16 restyle 高频帧全数在窗 + 24 个
/// NN 垫底帧（混密度交界预演）。窗口 = 可见格 rows 1..18、cols 9..25（272 格）。
const MAPLV: usize = 5;
const PLAYER_PX: i32 = 248;
const PLAYER_PY: i32 = 126;
/// 瓦片层 y 偏移 = layout::view_top_y()（classic=20，T544 锚）。
const TILE_OFF: i32 = 20;
const CAM_X: i32 = -144;
const CAM_Y: i32 = -16;
const COL0: i32 = 9;
const COL1: i32 = 25;
const ROW0: i32 = 1;
const ROW1: i32 = 18;
/// demo 弃件② 精绘帧（tools/hd-demo/gen_hd_demo.py EXPECTED_HAND_IDS，A 级锚）。
const HAND_IDS: [u8; 16] = [1, 22, 17, 10, 24, 46, 47, 2, 30, 15, 9, 45, 49, 38, 39, 44];

// ---------------- 素材装载 ----------------

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

/// classic 图集 = assets/raw/map 容器帧 00（128×208，d=1）。
fn classic_tileset(frames: &[ArgbImage]) -> ArgbImage {
    frames[0].clone()
}

/// demo 弃件图集 = assets/hd-demo/map_00_3x.png（384×624，d=3，R-7 弃件隔离）。
fn demo_tileset() -> ArgbImage {
    decode_png_file(&repo("assets/hd-demo/map_00_3x.png"), "先跑 python3 tools/hd-demo/gen_hd_demo.py")
        .with_density(3)
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

/// 非素材区 overlay 素材（全部 d=1 经典素材——两跑同源）。
struct OverlayImgs {
    /// map[3] 数字条（154×19，"文字/数字"面）。
    digits: ArgbImage,
    /// sptmap[0]（32×36，"精灵"面）。
    sprite: ArgbImage,
    /// actor[3]（90×33，"精灵"面二）。
    actor: ArgbImage,
    /// sptprop[1]（30×30，"道具精灵"面）。
    prop: ArgbImage,
}

fn load_overlays(frames: &[ArgbImage]) -> OverlayImgs {
    let one = |name: &str, count: usize, idx: usize| {
        let data = std::fs::read(repo(&format!("assets/raw/{name}")))
            .unwrap_or_else(|e| panic!("缺 assets/raw/{name}（先跑 tools/extract-assets/extract.py）：{e}"));
        let subs = game_data::PackedPng::parse(&data, count)
            .unwrap_or_else(|e| panic!("{name} 容器解析失败：{e}"));
        ArgbImage::from_decoded_png(game_data::decode_png(subs.images[idx].bytes).unwrap())
    };
    OverlayImgs {
        digits: frames[3].clone(),
        sprite: one("sptmap", 13, 0),
        actor: one("actor", 4, 3),
        prop: one("sptprop", 23, 1),
    }
}

fn build_view() -> GameView {
    let data = std::fs::read(repo(&format!("assets/raw/maplv{MAPLV}")))
        .unwrap_or_else(|e| panic!("缺 assets/raw/maplv{MAPLV}（先跑 tools/extract-assets/extract.py）：{e}"));
    let mut v = GameView::from_maplv(&data, PLAYER_PX, PLAYER_PY, ViewDims::classic())
        .unwrap_or_else(|e| panic!("maplv{MAPLV} 解析失败：{e}"));
    v.center_on_player(); // 加载步 11 居中相机（a.java:3508）
    assert_eq!(
        (v.cam_x, v.cam_y),
        (CAM_X, CAM_Y),
        "同相机锚：maplv{MAPLV} 玩家 ({PLAYER_PX},{PLAYER_PY}) 居中 ⇒ 相机 ({CAM_X},{CAM_Y})"
    );
    v
}

// ---------------- 场景渲染（对照变量唯一：仅 tileset 不同） ----------------

/// 非素材区 overlay 电池（HUD/精灵/文字）——双跑逐字节同配置。
/// 全部 d=1 素材 + 图元；alpha 均为 0/255 二值（见文件尾断言注）。
fn overlay_battery(g: &mut SoftGraphics<'_>, img: &OverlayImgs) {
    // HUD 带（状态栏区 y<view_top_y=20，不压瓦片）
    g.set_color(0x101820);
    g.fill_rect(0, 0, 240, 18);
    g.draw_string(&"HP 100".encode_utf16().collect::<Vec<u16>>(), 4, 2, anchor::TOP | anchor::LEFT);
    // 面板（压瓦片，"非素材区"锚——瓦片区上的恒等证明）
    g.set_color(0xE8E0C0);
    g.fill_rect(6, 26, 160, 24);
    // 数字条 map[3]（154×19，压瓦片）
    g.draw_image(&img.digits, 8, 28, anchor::TOP | anchor::LEFT);
    // 文字（差分占位字形，压瓦片）
    g.draw_string(&"HD-PROBE R-8".encode_utf16().collect::<Vec<u16>>(), 8, 56, anchor::TOP | anchor::LEFT);
    // 精灵 sptmap[0]（32×36，压瓦片）
    g.draw_image(&img.sprite, 150, 200, anchor::VCENTER | anchor::HCENTER);
    // 精灵 actor[3]（90×33，压瓦片）
    g.draw_image(&img.actor, 40, 300, anchor::BOTTOM | anchor::LEFT);
    // 道具精灵 sptprop[1]（30×30，压瓦片）
    g.draw_image(&img.prop, 180, 120, anchor::TOP | anchor::LEFT);
}

/// 全场景一帧：底色（两跑同）→ 瓦片层（唯一对照变量）→ overlay（两跑同）。
/// 返回 (画布, op 串)。
fn render_scene(view: &GameView, tileset: &ArgbImage, img: &OverlayImgs) -> (ArgbImage, Vec<String>) {
    let mut screen = ArgbImage::create(240 * SCALE, 320 * SCALE);
    let ops;
    {
        let mut g = SoftGraphics::new(&mut screen);
        g.set_scale(SCALE);
        g.set_color(0x202830); // 共享底色（非素材面，两跑同值）
        g.fill_rect(0, 0, 240, 320);
        view.paint_tiles(&mut g, tileset, TILE_OFF);
        overlay_battery(&mut g, img);
        ops = g.ops.clone();
    }
    (screen, ops)
}

/// overlay 单独一跑（透明底）：alpha==0xFF 掩码 = 非素材区（HUD/精灵/文字）。
fn overlay_mask(img: &OverlayImgs) -> Vec<bool> {
    let mut screen = ArgbImage::create(240 * SCALE, 320 * SCALE);
    {
        let mut g = SoftGraphics::new(&mut screen);
        g.set_scale(SCALE);
        overlay_battery(&mut g, img);
    }
    screen.argb.iter().map(|p| (p >> 24) & 0xFF == 0xFF).collect()
}

/// 瓦片层单独一跑（透明底）：非零 alpha 掩码 = 素材区（瓦片落笔面）。
fn tile_mask(view: &GameView, tileset: &ArgbImage) -> Vec<bool> {
    let mut screen = ArgbImage::create(240 * SCALE, 320 * SCALE);
    {
        let mut g = SoftGraphics::new(&mut screen);
        g.set_scale(SCALE);
        view.paint_tiles(&mut g, tileset, TILE_OFF);
    }
    screen.argb.iter().map(|p| (p >> 24) & 0xFF != 0).collect()
}

/// 格 (r,c) 的物理矩形（瓦片层 x=cam_x+16c、y=cam_y+TILE_OFF+16r，×SCALE）。
fn cell_rect(view: &GameView, r: i32, c: i32) -> (usize, usize, usize, usize) {
    let x = (view.cam_x + (c << 4)) * SCALE;
    let y = (view.cam_y + TILE_OFF + (r << 4)) * SCALE;
    (x.max(0) as usize, y.max(0) as usize, (16 * SCALE) as usize, (16 * SCALE) as usize)
}

// ---------------- 最小 PNG 写出（headless_shot::encode_png 同构；测试自含） ----------------

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
    let (mut a, mut b) = (1u32, 0u32);
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
    eprintln!("[hd-visual] {name}：{}×{} → {}", w, h, p.display());
}

// ---------------- R-8 机器判据（A 级） ----------------

/// 【判据①·R-6】classic/HD op 串全集相等 + 非素材区（HUD/精灵/文字）逐像素恒等
/// + 差异只落素材区（换素材零行为漂移）。产出双 PNG（C 级人工判读材料）。
#[test]
fn hd_compare_dual_png_scale3_non_asset_identity() {
    let frames = map_frames();
    let classic = classic_tileset(&frames);
    let demo = demo_tileset();
    let img = load_overlays(&frames);
    let view = build_view();

    let (a, ops_a) = render_scene(&view, &classic, &img); // ①classic 原图集
    let (b, ops_b) = render_scene(&view, &demo, &img); // ②demo 图集（d=3）

    // —— R-6：op 串全集相等（op 记逻辑尺寸，与 density 无关）——
    assert_eq!(ops_a, ops_b, "R-6：classic/HD 同场景同帧 op 串全集相等（换素材零行为漂移）");

    // —— 非素材区（HUD/精灵/文字）逐像素恒等（R-8 机器判据）——
    let mask = overlay_mask(&img);
    let (w, h) = (240 * SCALE as usize, 320 * SCALE as usize);
    let mut non_asset_px = 0usize;
    let mut zone_px = [0usize; 4]; // 0=HUD带 1=面板/数字条 2=文字 3=精灵
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if !mask[i] {
                continue;
            }
            non_asset_px += 1;
            let zone = match (x as i32 / SCALE, y as i32 / SCALE) {
                (_, ly) if ly < 18 => 0,
                (lx, ly) if (6..166).contains(&lx) && (26..50).contains(&ly) => 1,
                (lx, ly) if (8..120).contains(&lx) && (52..72).contains(&ly) => 2,
                _ => 3,
            };
            zone_px[zone] += 1;
            assert_eq!(a.argb[i], b.argb[i], "非素材区像素({x},{y}) 两图须逐像素恒等（R-8）");
        }
    }
    for (z, name) in ["HUD 带", "面板/数字条", "文字", "精灵"].iter().enumerate() {
        assert!(zone_px[z] > 0, "非素材区锚缺位：{name} 掩码 0 像素（overlay 电池漂了？）");
    }
    assert!(non_asset_px > 0, "非素材区掩码空 = 断言假阴性");

    // —— 差异只落素材区（瓦片层落笔面 ∩ 非素材掩码之外）——
    let tiles = tile_mask(&view, &classic).iter().zip(tile_mask(&view, &demo)).map(|(x, y)| *x || y).collect::<Vec<_>>();
    let mut diff_px = 0usize;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if a.argb[i] == b.argb[i] {
                continue;
            }
            diff_px += 1;
            assert!(!mask[i], "差异像素({x},{y}) 落在非素材区（HUD/精灵/文字）——行为漂移！");
            assert!(tiles[i], "差异像素({x},{y}) 不在瓦片层落笔面——对照变量不唯一！");
        }
    }
    assert!(diff_px > 0, "两图零差异 = 对照假阴性（demo 图集未改变画面？）");
    eprintln!(
        "[hd-visual] 非素材区恒等 {non_asset_px} px（HUD {} / 面板 {} / 文字 {} / 精灵 {}）；素材区差异 {diff_px} px",
        zone_px[0], zone_px[1], zone_px[2], zone_px[3]
    );

    // —— 双 PNG 产出（assets/hd-demo/compare/，gitignore 区；人工判读=C 级）——
    let dir = repo("assets/hd-demo/compare");
    write_png(&dir, "classic_scale3.png", 240 * SCALE, 320 * SCALE, &a.argb);
    write_png(&dir, "hd_scale3.png", 240 * SCALE, 320 * SCALE, &b.argb);
    let diff: Vec<u32> = a
        .argb
        .iter()
        .zip(b.argb.iter())
        .map(|(&x, &y)| if x == y { 0xFF00_0000 } else { 0xFFFF_FFFF })
        .collect();
    write_png(&dir, "diff_scale3.png", 240 * SCALE, 320 * SCALE, &diff);
}

/// 【判据②·逐格】同帧逐格对照（瓦片层单独一跑，无 overlay 遮挡）：
/// **transform=0 的可见格差异⟺restyle 帧**（NN 垫底帧两图逐像素恒等=混密度并存
/// 零漂移；restyle 帧逐格有差异=清晰度收益落点）。这是"哪个 tile 变清晰"的
/// A 级证据（人工判读只是复述它）。
#[test]
fn hd_compare_per_cell_diff_iff_restyle_tile() {
    let frames = map_frames();
    let classic = classic_tileset(&frames);
    let demo = demo_tileset();
    let view = build_view();
    // 瓦片层单独一跑（透明底）——逐格对照不受 HUD/精灵/文字遮挡影响
    let mut a = ArgbImage::create(240 * SCALE, 320 * SCALE);
    let mut b = ArgbImage::create(240 * SCALE, 320 * SCALE);
    for (screen, tileset) in [(&mut a, &classic), (&mut b, &demo)] {
        let mut g = SoftGraphics::new(screen);
        g.set_scale(SCALE);
        view.paint_tiles(&mut g, tileset, TILE_OFF);
    }
    let w = 240 * SCALE as usize;

    let (mut restyle_cells, mut nn_cells) = (0usize, 0usize);
    for r in ROW0..ROW1 {
        for c in COL0..COL1 {
            let v = view.terrain_at(r, c);
            if v == 0 || view.transform_at(r, c) != 0 {
                continue; // v=0 不落笔；transform≠0（kinds 4-7 shim 原样绘制）采样域越格，逐格锚不适用
            }
            let (cx, cy, cw, ch) = cell_rect(&view, r, c);
            let mut diff = 0usize;
            for y in cy..(cy + ch).min(320 * SCALE as usize) {
                for x in cx..(cx + cw).min(w) {
                    let i = y * w + x;
                    if a.argb[i] != b.argb[i] {
                        diff += 1;
                    }
                }
            }
            let restyle = HAND_IDS.contains(&v);
            if restyle {
                restyle_cells += 1;
                assert!(diff > 0, "restyle 格 (r{r},c{c}) v={v} 两图零差异——demo 精绘未生效");
            } else {
                nn_cells += 1;
                assert_eq!(
                    diff, 0,
                    "NN 垫底格 (r{r},c{c}) v={v} 有 {diff} px 差异——NN 帧非源帧 NN×3（帧序/串位嫌疑）"
                );
            }
        }
    }
    assert!(restyle_cells >= 20 && nn_cells >= 10, "场景锚漂移：restyle 格 {restyle_cells} / NN 格 {nn_cells}（须 ≥20/≥10）");
    eprintln!("[hd-visual] 逐格对照：restyle 格 {restyle_cells} 全部有差异、NN 格 {nn_cells} 全部恒等");
}

/// 【判据③·demo 图集帧序/帧数】104 帧（8×13 格网）、帧序 = tile id 升序：
/// 逐帧断言**NN 帧逐像素 == 源帧 NN×3（不串位）**、restyle 帧 ≠ NN 放大但
/// alpha 掩码 == 源帧 alpha NN×3（槽位同构）——帧序锚 = HAND_IDS 集合恰好 16 帧。
#[test]
fn demo_atlas_frame_order_104_frames() {
    let frames = map_frames();
    let classic = classic_tileset(&frames);
    let demo = demo_tileset();
    // R-4 图集切片锚：列数 8/行数 13（(v&7)/(v>>3) 索引语义，width/(16*d)==8）
    assert_eq!(
        (demo.width / (16 * demo.density), demo.height / (16 * demo.density)),
        (8, 13),
        "demo 图集格网 8×13（R-4 列数 8 索引语义锚）"
    );
    assert_eq!(demo.width * demo.height, 384 * 624, "demo 图集 384×624（=128×208 ×3）");

    let (cd, cw) = (classic.density, classic.width);
    assert_eq!(cd, 1, "classic 图集 d=1");
    let mut nn_frames = 0usize;
    let mut restyle_frames = 0usize;
    for id in 0..104u8 {
        let (col, row) = ((id & 7) as i32, (id >> 3) as i32);
        let mut nn_equal = true;
        for ly in 0..16 {
            for lx in 0..16 {
                let src = classic.argb[((row * 16 + ly) * cw + (col * 16 + lx)) as usize];
                for dy in 0..3 {
                    for dx in 0..3 {
                        let got = demo.argb[(((row * 16 + ly) * 3 + dy) * demo.width + (col * 16 + lx) * 3 + dx) as usize];
                        // alpha 掩码 == 源帧 alpha NN×3（restyle 帧槽位同构锚）
                        assert_eq!(
                            (got >> 24) & 0xFF,
                            (src >> 24) & 0xFF,
                            "帧 {id}（槽位 col{col},row{row}）alpha 掩码漂移——帧序串位嫌疑"
                        );
                        if got != src {
                            nn_equal = false;
                        }
                    }
                }
            }
        }
        let hand = HAND_IDS.contains(&id);
        if hand {
            restyle_frames += 1;
            assert!(!nn_equal, "restyle 帧 {id} 逐像素 == NN 放大——精绘未生效/帧序错位");
        } else {
            nn_frames += 1;
            assert!(nn_equal, "NN 垫底帧 {id} ≠ 源帧 NN×3——帧序串位（R-7 生成器断言 5 同锚）");
        }
    }
    assert_eq!((nn_frames, restyle_frames), (88, 16), "帧数 104 = NN 88 + restyle 16");
}

/// 【判据④·d==s 直采样像素级】真 demo 图集（d=3）@scale=3：1:1 直采
/// 逐像素 == 源图（零重采样零相位失真；density_draw `sampling_matrix_direct_copy_d_eq_s`
/// 同判据，此处用真弃件图集钉）。
#[test]
fn demo_atlas_direct_copy_d_eq_s() {
    let demo = demo_tileset();
    let (s, d) = (SCALE, demo.density);
    assert_eq!(s, d, "d==s 前提（scale=3 必选）");
    let mut screen = ArgbImage::create(demo.width + 8, demo.height + 8);
    {
        let mut g = SoftGraphics::new(&mut screen);
        g.set_scale(s);
        g.draw_image(&demo, 0, 0, anchor::TOP | anchor::LEFT);
    }
    for y in 0..demo.height {
        for x in 0..demo.width {
            let src = demo.argb[(y * demo.width + x) as usize];
            let got = screen.argb[(y * screen.width + x) as usize];
            if (src >> 24) & 0xFF == 0 {
                // alpha==0 跳过（set_pixel 语义）：画布留底（透明 0），源 RGB 不计
                assert_eq!(got, 0, "d==s 透明槽({x},{y}) 不落笔（画布留底）");
            } else {
                assert_eq!(got, src, "d==s 直采样像素({x},{y}) 须 == 源图（1:1 行拷）");
            }
        }
    }
    // bbox 外不落笔
    assert_eq!(screen.argb[(demo.height * screen.width) as usize], 0, "下沿外须空");
}

// ---------------- 标注版产物后处理（lead 追加 2026-10-21） ----------------
//
// 替换范围标注（用户诉求"把高清资源标注出来"）：按瓦片格画细亮色框——
//   **绿框 = 重绘高清帧**（16 个 restyle 精绘 tile，真正换画的资源）
//   **黄框 = 仅放大帧**（88 个 NN 垫底 tile：高清分辨率、像素内容为原图放大）
//   **无框 = 未替换**（d=1 精灵/挂幡/宝箱/HUD/文字 + 底色区——原素材）
// 纪律：**标注只画在产物副本上**——机器判据仍对拍无标注原图（上列测试零改动），
// 对照变量唯一原则不受标注破坏（两图框位置严格同格，见断言）。

const BOX_GREEN: u32 = 0xFF00_E050; // 亮绿（灰白石砖主色外）
const BOX_YELLOW: u32 = 0xFFFF_C400; // 琥珀黄
const BOX_THICK: i32 = 2; // scale=3 下每格 48px 物理、框 2px

fn put_px(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, c: u32) {
    if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
        buf[y as usize * w + x as usize] = c;
    }
}

fn fill_px(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, rw: i32, rh: i32, c: u32) {
    for dy in 0..rh {
        for dx in 0..rw {
            put_px(buf, w, h, x + dx, y + dy, c);
        }
    }
}

/// 矩形边框（t px 粗，四边）。
fn border_px(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, bw: i32, bh: i32, c: u32, t: i32) {
    fill_px(buf, w, h, x, y, bw, t, c);
    fill_px(buf, w, h, x, y + bh - t, bw, t, c);
    fill_px(buf, w, h, x, y, t, bh, c);
    fill_px(buf, w, h, x + bw - t, y, t, bh, c);
}

/// 3×5 点阵大写字模（测试自含，仅图例用；每行 3 bit，bit2=左列）。
fn glyph_rows(ch: char) -> [u8; 5] {
    match ch {
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        'C' => [0b011, 0b100, 0b100, 0b100, 0b011],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'G' => [0b011, 0b100, 0b101, 0b101, 0b011],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b110, 0b101, 0b101, 0b101, 0b101],
        'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
        'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
        'Q' => [0b010, 0b101, 0b101, 0b111, 0b011],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b011],
        'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b110, 0b001, 0b010, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b011, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b011, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b110],
        '=' => [0b000, 0b111, 0b000, 0b111, 0b000],
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        ',' => [0b000, 0b000, 0b000, 0b010, 0b100],
        _ => [0, 0, 0, 0, 0],
    }
}

fn text_px(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, s: &str, fs: i32, c: u32) {
    let mut cx = x;
    for ch in s.chars() {
        for (gy, row) in glyph_rows(ch).iter().enumerate() {
            for gx in 0..3 {
                if row & (0b100 >> gx) != 0 {
                    fill_px(buf, w, h, cx + gx as i32 * fs, y + gy as i32 * fs, fs, fs, c);
                }
            }
        }
        cx += 4 * fs;
    }
}

/// 角落小图例（三色框含义 + 场景/相机/scale 说明）。
fn legend_px(buf: &mut [u32], w: usize, h: usize) {
    let fs = 2;
    let lines: [(Option<u32>, &str); 4] = [
        (Some(BOX_GREEN), "GREEN=HD REDRAWN"),
        (Some(BOX_YELLOW), "YELLOW=NN UPSCALED"),
        (None, "NOBOX=ORIG D1"),
        (None, "MAPLV5 CAM-144,-16 S3"),
    ];
    let pw = 24 + 22 * 4 * fs;
    let ph = 6 + 4 * (5 * fs + 3) + 2;
    let (x0, y0) = (w as i32 - pw - 8, h as i32 - ph - 8);
    fill_px(buf, w, h, x0, y0, pw, ph, 0xF010_1820);
    border_px(buf, w, h, x0, y0, pw, ph, 0xFF60_7080, 1);
    for (i, (sample, txt)) in lines.iter().enumerate() {
        let ty = y0 + 6 + i as i32 * (5 * fs + 3);
        if let Some(c) = sample {
            fill_px(buf, w, h, x0 + 6, ty + 1, 10, 10, *c);
        }
        text_px(buf, w, h, x0 + 22, ty, txt, fs, 0xFFFF_FFFF);
    }
}

/// 按瓦片格标注替换范围（返回 (标注副本, 绿格数, 黄格数)）。
fn annotate_replacement_scope(view: &GameView, base: &ArgbImage) -> (Vec<u32>, usize, usize) {
    let (w, h) = (base.width as usize, base.height as usize);
    let mut buf = base.argb.clone();
    let (mut green_cells, mut yellow_cells) = (0usize, 0usize);
    for r in ROW0..ROW1 {
        for c in COL0..COL1 {
            let v = view.terrain_at(r, c);
            if v == 0 {
                continue; // 无框：v=0 不落笔（底色区）
            }
            let (x, y, cw, ch) = cell_rect(view, r, c);
            if x >= w {
                continue; // 右缘裁切格（c=24）整格在画布外
            }
            let color = if HAND_IDS.contains(&v) {
                green_cells += 1;
                BOX_GREEN
            } else {
                yellow_cells += 1;
                BOX_YELLOW
            };
            border_px(&mut buf, w, h, x as i32, y as i32, cw as i32, ch as i32, color, BOX_THICK);
        }
    }
    (buf, green_cells, yellow_cells)
}

/// 并排裁剪 + 底部图例条（交付首选图）：左 classic | 右 HD，下方图例。
fn compose_pair_with_legend(
    a: &[u32],
    b: &[u32],
    w: usize,
    x0: usize,
    y0: usize,
    cw: usize,
    ch: usize,
) -> (Vec<u32>, i32) {
    let strip_h = 56i32;
    let pw = cw * 2;
    let ph = ch + strip_h as usize;
    let mut out = vec![0u32; pw * ph];
    for y in 0..ch {
        for x in 0..cw {
            out[y * pw + x] = a[(y0 + y) * w + x0 + x];
            out[y * pw + cw + x] = b[(y0 + y) * w + x0 + x];
        }
    }
    // 图例条（fs=2）：三色框含义 + 左右说明 + 场景/相机/scale
    fill_px(&mut out, pw, ph, 0, ch as i32, pw as i32, strip_h, 0xF010_1820);
    let fs = 2;
    for (sx, sample, txt) in [
        (8, Some(BOX_GREEN), "GREEN=HD REDRAWN"),
        (170, Some(BOX_YELLOW), "YELLOW=NN UPSCALED"),
        (346, None, "NOBOX=ORIG D1"),
    ] {
        let ty = ch as i32 + 8;
        if let Some(c) = sample {
            fill_px(&mut out, pw, ph, sx, ty, 10, 10, c);
        }
        text_px(&mut out, pw, ph, sx + 14, ty, txt, fs, 0xFFFF_FFFF);
    }
    text_px(
        &mut out,
        pw,
        ph,
        8,
        ch as i32 + 32,
        "LEFT=CLASSIC RIGHT=HD MAPLV5 CAM-144,-16 SCALE3",
        fs,
        0xFFFF_FFFF,
    );
    (out, strip_h)
}

/// 【lead 追加·用户诉求】标注版对照图：替换范围一眼可见（绿=restyle 重绘 /
/// 黄=NN 仅放大 / 无框=未替换 d=1 素材）。产出 `assets/hd-demo/compare/annotated/`
/// 双图 + 并排裁剪（交付首选）。**标注仅画在产物副本**：机器判据仍对拍无标注
/// 原图，断言语义零改动；两图框位置严格同格（断言钉）。
#[test]
fn hd_annotated_replacement_scope_products() {
    let frames = map_frames();
    let classic = classic_tileset(&frames);
    let demo = demo_tileset();
    let img = load_overlays(&frames);
    let view = build_view();
    let (a, _) = render_scene(&view, &classic, &img);
    let (b, _) = render_scene(&view, &demo, &img);
    let (w, h) = (a.width, a.height);

    let (mut ac, g1, y1) = annotate_replacement_scope(&view, &a);
    let (mut bc, g2, y2) = annotate_replacement_scope(&view, &b);
    assert_eq!((g1, y1), (g2, y2), "两图框位置严格同格（标注不破坏对照变量唯一）");
    // 场景锚：可见窗 255 格 = 绿 219（restyle）+ 黄 36（NN）
    assert_eq!((g1, y1), (219, 36), "标注格数锚（maplv5 可见窗）：绿 {g1} / 黄 {y1}");
    legend_px(&mut ac, w as usize, h as usize);
    legend_px(&mut bc, w as usize, h as usize);

    let dir = repo("assets/hd-demo/compare/annotated");
    write_png(&dir, "classic_scale3_annotated.png", w, h, &ac);
    write_png(&dir, "hd_scale3_annotated.png", w, h, &bc);
    for (name, x0, y0, cw, ch) in [
        ("pair_torch_stone_annotated.png", 0, 300, 360, 360),
        ("pair_floor_mixed_annotated.png", 300, 540, 360, 360),
    ] {
        let (pair, strip_h) = compose_pair_with_legend(&ac, &bc, w as usize, x0, y0, cw, ch);
        write_png(&dir, name, (cw * 2) as i32, ch as i32 + strip_h, &pair);
    }
    eprintln!(
        "[hd-visual·标注] 绿框（restyle 重绘）{g1} 格 / 黄框（NN 仅放大）{y1} 格；无框=未替换（d=1 HUD/精灵/文字 + 底色区）"
    );
}
