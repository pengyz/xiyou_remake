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

/// `f_int_arr_01`（a.java:44）：`m_004` 的 Nokia 变换码表。
/// 全部值 >7 ⇒ shim `drawImageTransformed` 的 switch 落 default = 原样绘制
/// （op 文本仍记录原始码，`drawImageT`）。
pub const NOKIA_TRANSFORM_TABLE: [i32; 8] = [0, 8192, 16384, 24576, 8462, 270, 90, 8282];

/// `m_004`（a.java:4538-4540）：DirectGraphics 变换绘制（shim 下视觉 = 原样）。
fn m_004(g: &mut SoftGraphics<'_>, img: &ArgbImage, x: i32, y: i32, kind: i32) {
    let t = NOKIA_TRANSFORM_TABLE[kind as usize];
    g.draw_image_transformed(img, x, y, 0, t);
}

/// `m_002`（a.java:4526-4530）：源矩形 clip 绘制——clip(x,y,w,h) 后把图
/// 画在 `(x-sx, y-sy)`，只露出 `(sx,sy)` 起的子区。
fn m_002(g: &mut SoftGraphics<'_>, img: &ArgbImage, x: i32, y: i32, sx: i32, sy: i32, w: i32, h: i32) {
    g.set_clip(x, y, w, h);
    g.draw_image(img, x - sx, y - sy, 0);
    g.set_clip(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
}

/// `m_034`（a.java:5970-5980）：软键栏。`left`/`right` = f_byte_13/f_byte_14
/// （0 = 不绘制；kind 1..n 选 ui[11] 精灵条的第 n 个 12×10 图标）。
pub fn m_034_softkeys(g: &mut SoftGraphics<'_>, ui10: &ArgbImage, ui11: &ArgbImage, left: i8, right: i8) {
    if left != 0 {
        g.draw_image(ui10, 0, crate::layout::softkey_base_y(), 0);
        m_002(
            g,
            ui11,
            crate::layout::SOFTKEY_ICON_LEFT_X,
            crate::layout::softkey_icon_y(),
            (left as i32 - 1) * crate::layout::SOFTKEY_ICON_W,
            0,
            crate::layout::SOFTKEY_ICON_W,
            crate::layout::SOFTKEY_ICON_H,
        );
    }
    if right != 0 {
        m_004(g, ui10, crate::layout::softkey_right_x(), crate::layout::softkey_base_y(), 1);
        m_002(
            g,
            ui11,
            crate::layout::softkey_icon_right_x(),
            crate::layout::softkey_icon_y(),
            (right as i32 - 1) * crate::layout::SOFTKEY_ICON_W,
            0,
            crate::layout::SOFTKEY_ICON_W,
            crate::layout::SOFTKEY_ICON_H,
        );
    }
}

/// mode 21（声音询问）绘制：a.java:3047-3054 + m_034 软键栏。
///
/// 软键 (1,2) 由 m_000 case 21 设置（a.java:4457-4458）。
pub fn paint_sound_prompt(g: &mut SoftGraphics<'_>, ui10: &ArgbImage, ui11: &ArgbImage, softkeys: (i8, i8)) {
    g.set_color(0);
    g.fill_rect(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
    g.set_color((-1i32) as u32);
    g.draw_string(
        &crate::layout::SOUND_PROMPT_TEXT,
        crate::layout::prompt_cx(),
        crate::layout::prompt_cy(),
        anchor_top_hcenter(),
    );
    m_034_softkeys(g, ui10, ui11, softkeys.0, softkeys.1);
}

/// `Graphics.TOP | Graphics.HCENTER` = 17（drawString 第三参的常用组合）。
pub fn anchor_top_hcenter() -> i32 {
    crate::render::anchor::TOP | crate::render::anchor::HCENTER
}

/// mode 0（启动屏）绘制：a.java:2338-2347。
///
/// 相位 <2：整屏白 + 启动图几何居中（Java `-` 优先于 `>>` ⇒ `(240-w)>>1`）；
/// 相位 >=2：`m_144` logo 动画层（[`crate::logo_anim::LogoAnim::paint`]）。
pub fn paint_boot(g: &mut SoftGraphics<'_>, boot: &BootPaintState) {
    if boot.phase < 2 {
        // setColor(-1)：Java int -1 位模式 & 0xFFFFFF = 0xFFFFFF
        g.set_color((-1i32) as u32);
        g.fill_rect(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
        if let Some(img) = &boot.image {
            let x = crate::layout::SCREEN_W - img.width >> 1;
            let y = crate::layout::SCREEN_H - img.height >> 1;
            g.draw_image(img, x, y, 0);
        }
    } else {
        match (&boot.logo, &boot.sflogo7) {
            (Some(logo), Some(sflogo7)) => logo.paint(g, sflogo7),
            // logic 已切 f_int_04>=2 但首个 m_143 未跑：f_byte_29=0 ⇒ m_144 仅白底
            // （A-boot-menu T32 硬锚：a682fa57=纯白）
            _ => {
                g.set_color(0xFFFFFF);
                g.fill_rect(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
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
    /// gameMode（0=启动屏, 21=声音询问；1=title 菜单未端口）。
    pub mode: i32,
    /// keyValue（keyPressed 设置、tick 末清零，a.java:4639/4251）。
    pub key_value: i32,
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
    /// ui 容器 25 张（m_001(8)，m_000 case 21 a.java:4456 加载）。
    ui: Vec<ArgbImage>,
    /// f_byte_13/f_byte_14 软键栏状态（m_034 绘制输入）。
    pub softkeys: (i8, i8),
    /// paint 计数（对拍 TICK n 用）。
    pub paints: u32,
    /// mode 0 结束标志（logo 时间线 35 耗尽，a.java:3351-3360 切 gameMode=21）。
    pub finished: bool,
}

impl BootMachine {
    /// 构造：预解码全部依赖资源（解码耗时与 tick 语义无关）。
    pub fn new(l0: ArgbImage, l1: ArgbImage, sflogo: Vec<ArgbImage>, ui: Vec<ArgbImage>) -> BootMachine {
        assert_eq!(sflogo.len(), 8, "sflogo 容器 8 张（a.java:465-466 计数表）");
        assert_eq!(ui.len(), 25, "ui 容器 25 张（a.java:465-466 计数表）");
        BootMachine {
            mode: 0,
            key_value: 0,
            phase: 0,
            counter: 0,
            image: None,
            logo: None,
            boot_images: vec![l0, l1],
            sflogo,
            ui,
            softkeys: (0, 0),
            paints: 0,
            finished: false,
        }
    }

    /// 一次 paint（serviceRepaints → a.paint）。按 [`Self::mode`] 分发。
    pub fn paint(&mut self, g: &mut SoftGraphics<'_>) {
        self.paints += 1;
        match self.mode {
            21 => paint_sound_prompt(g, &self.ui[10], &self.ui[11], self.softkeys),
            _ => {
                let state = BootPaintState {
                    phase: self.phase,
                    image: self.image.clone(),
                    logo: self.logo.clone(),
                    sflogo7: Some(self.sflogo[7].clone()),
                };
                paint_boot(g, &state);
            }
        }
    }

    /// 一次逻辑 tick。`key` 为本 tick 边界投递的按键码（`keyValue`，无则 0）。
    ///
    /// - mode 0：a.java:3330-3361（加载轮播/logo 时间线/清理切换）
    /// - mode 21：a.java:4204-4223（-6 确认 / -7 否定 → gameMode=1）
    pub fn tick(&mut self, key: i32) {
        match self.mode {
            21 => {
                // a.java:4206-4223：switch (keyValue)
                match key {
                    -7 => {
                        // f_bool_29=false（声音关）… gameMode=1
                        self.mode = 1;
                        self.finished = true; // mode 1 未端口，对拍范围到切换为止
                    }
                    -6 => {
                        // f_bool_29=true；f_int_155==0 → 60；gameMode=1
                        self.mode = 1;
                        self.finished = true;
                    }
                    _ => {}
                }
                self.key_value = 0;
            }
            _ => self.tick_mode0(),
        }
    }

    /// gameMode 0 的逻辑 tick（a.java:3330-3361）。
    fn tick_mode0(&mut self) {
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
                // a.java:3352-3360：清理 + gameMode=21 + m_000()
                self.logo.as_mut().unwrap().clear_all(); // m_147(-1)
                self.image = None;
                self.counter = 0;
                self.mode = 21;
                // m_000 case 21（a.java:4457-4459）：软键 (1,2)；f_bool_29=false；
                // RMS "SKY_WAR" 读档失败路径（oracle 内存库恒空 ⇒ catch 删库重置）
                self.softkeys = (1, 2);
                self.finished = false; // mode 21 属对拍范围
            }
        }
        self.key_value = 0; // a.java:4251（tick 末 keyValue=0）
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
